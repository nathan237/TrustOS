//! AMD GPU Compute Agent — Bare-metal RDNA dispatch from TrustOS
//!
//! This module implements a portable GPU compute agent that can submit
//! compute workloads directly to AMD Navi 10 (RDNA 1) Compute Units,
//! without any userspace driver stack (no ROCm, no Vulkan, no OpenCL).
//!
//! Architecture (bare-metal compute dispatch):
//! ```
//! TrustOS CPU  →  PM4 Ring Buffer  →  Command Processor (CP/MEC)
//!                                           ↓
//!                                     RDNA Compute Units (40 CU)
//!                                           ↓
//!                                     Results in VRAM/system memory
//! ```
//!
//! Pipeline:
//! 1. Allocate GPU-visible ring buffer (4KB, 256-entry, naturally aligned)
//! 2. Initialize MEC (Micro Engine Compute) queue via HQD registers
//! 3. Encode compute kernel as raw RDNA ISA binary (hand-assembled)
//! 4. Submit PM4 DISPATCH_DIRECT packet to ring buffer
//! 5. Wait for completion via RELEASE_MEM fence writeback
//! 6. Read results from data buffer
//!
//! Pre-assembled RDNA kernels (agents):
//! - `AGENT_MEMFILL`: Fill a VRAM buffer with a constant u32 value
//! - `AGENT_MEMCOPY`: Copy N dwords from src to dst (GPU-speed memcpy)
//! - `AGENT_SHA256`:  SHA-256 hash one 64-byte block (parallel across wavefront)
//! - `AGENT_INCR`:    Increment each element — proves CUs are executing
//!
//! References:
//! - AMD RDNA ISA: https://developer.amd.com/wp-content/resources/RDNA_Shader_ISA.pdf
//! - PM4 format: AMD drivers/gpu/drm/amd/amdgpu/amdgpu_pm4.h
//! - GFX10 MEC: drivers/gpu/drm/amd/amdgpu/gfx_v10_0.c

use alloc::string::String;
use alloc::format;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use spin::Mutex;

use super::{mmio_read32, mmio_write32, GpuInfo};
use super::regs;
use crate::memory;

// ═══════════════════════════════════════════════════════════════════════════════
// Constants
// ═══════════════════════════════════════════════════════════════════════════════

/// Ring buffer size: 4KB = 1024 DWORDs = 256 PM4 packets (typical)
const RING_SIZE_DWORDS: usize = 1024;
const RING_SIZE_BYTES: usize = RING_SIZE_DWORDS * 4;

/// Data buffer for shader I/O: 64KB  
const DATA_BUFFER_SIZE: usize = 64 * 1024;

/// Fence address offset within data buffer (last 16 bytes)
const FENCE_OFFSET: usize = DATA_BUFFER_SIZE - 16;

/// Fence value to write on completion
const FENCE_SIGNAL_VALUE: u64 = 0xDEAD_BEEF_CAFE_F00D;

/// Timeout for GPU operations (in polling iterations)
const GPU_TIMEOUT_ITERS: u64 = 10_000_000;

// ═══════════════════════════════════════════════════════════════════════════════
// PM4 Packet Builder
// ═══════════════════════════════════════════════════════════════════════════════

/// Build a PM4 Type 3 packet header
/// Format: [31:30]=type(3), [23:16]=opcode, [15:14]=0, [13:0]=count-1
#[inline]
fn pm4_type3_header(opcode: u32, count: u32) -> u32 {
    (3 << 30) | ((opcode & 0xFF) << 8) | ((count - 1) & 0x3FFF)
}

/// Build a PM4 NOP packet (padding)
fn pm4_nop() -> [u32; 2] {
    [pm4_type3_header(regs::PM4_NOP, 1), 0]
}

/// Build SET_SH_REG packet to program one shader register
/// reg_offset is relative to SH_REG_BASE (0x2C00)
fn pm4_set_sh_reg(reg_offset: u32, value: u32) -> [u32; 3] {
    [
        pm4_type3_header(regs::PM4_SET_SH_REG, 2),
        (reg_offset - regs::SH_REG_BASE) >> 2, // Register index (dword offset from base)
        value,
    ]
}

/// Build SET_SH_REG packet for two consecutive registers
fn pm4_set_sh_reg2(reg_offset: u32, val0: u32, val1: u32) -> [u32; 4] {
    [
        pm4_type3_header(regs::PM4_SET_SH_REG, 3),
        (reg_offset - regs::SH_REG_BASE) >> 2,
        val0,
        val1,
    ]
}

/// Build DISPATCH_DIRECT packet — launches compute workgroups
/// thread_x/y/z = number of workgroups in each dimension
fn pm4_dispatch_direct(groups_x: u32, groups_y: u32, groups_z: u32) -> [u32; 5] {
    [
        pm4_type3_header(regs::PM4_DISPATCH_DIRECT, 4),
        groups_x,
        groups_y,
        groups_z,
        1, // dispatch_initiator: ordered_append=0, force_start_at_0=1
    ]
}

/// Build RELEASE_MEM packet — writes fence value to memory on completion
/// This tells us "all dispatched work before this point is done"
fn pm4_release_mem(fence_gpu_addr: u64, fence_value: u64) -> [u32; 7] {
    [
        pm4_type3_header(regs::PM4_RELEASE_MEM, 6),
        // event_type=CACHE_FLUSH_AND_INV_TS_EVENT(0x14), event_index=5 (TS)
        (0x14) | (5 << 8) | (0 << 12), // cache policy
        // data_sel=2 (64-bit data), int_sel=0 (no interrupt)
        (2 << 29), // Write 64-bit immediate data
        (fence_gpu_addr & 0xFFFFFFFF) as u32,          // address low
        ((fence_gpu_addr >> 32) & 0xFFFF) as u32,      // address high
        (fence_value & 0xFFFFFFFF) as u32,              // data low
        ((fence_value >> 32) & 0xFFFFFFFF) as u32,      // data high
    ]
}

// ═══════════════════════════════════════════════════════════════════════════════
// RDNA ISA — Pre-assembled Compute Kernels (Agents)
// ═══════════════════════════════════════════════════════════════════════════════

/// RDNA ISA encoding helpers
/// GFX10 (RDNA 1) instruction formats:
/// 
/// SOPP (Scalar-Only, no operands beyond imm16):
///   [31:23] = 0x17F (SOPP prefix), [22:16] = opcode, [15:0] = imm16
///
/// SOP1 (Scalar, 1 source):
///   [31:23] = 0x17D (SOP1 prefix), [15:8] = sdst, [7:0] = ssrc0
///
/// SMEM (Scalar Memory):
///   [31:26] = 0x30 (SMEM prefix), [25:22] = op, [21] = imm, [20:15] = sdata, [14:9] = sbase, [8:0] = offset
///
/// VOP1 (Vector, 1 source):
///   [31:25] = 0x3F (VOP1 prefix), [24:17] = vdst, [16:9] = opcode, [8:0] = src0
///
/// FLAT/GLOBAL (Flat/Global memory):
///   64-bit, [31:26]=0x37 (FLAT), [25:18]=op, [17:16]=seg(0=flat,1=scratch,2=global)
///
/// MTBUF/BUFFER (Buffer operations):
///   64-bit, [31:26]=0x38 (MTBUF) or [31:26]=0x3A (MUBUF)

/// Agent: INCR — Increment each u32 in a data buffer
///
/// This is the simplest possible "proof of life" compute kernel.
/// Each work item reads data[global_id], adds 1, writes it back.
///
/// Expected register setup (via USER_DATA SGPRs):
///   s[0:3] = Buffer descriptor (base_addr_lo, base_addr_hi, num_records, stride/flags)
///
/// RDNA 1 (GFX10) machine code:
/// ```
///   v_mov_b32     v1, 4                      ; stride = 4 bytes per u32
///   v_mul_lo_u32  v1, v0, v1                  ; byte_offset = global_id * 4
///   buffer_load_dword v2, v1, s[0:3], 0 offen ; v2 = data[global_id]
///   s_waitcnt     vmcnt(0)                    ; wait for load
///   v_add_u32     v2, v2, 1                   ; v2 += 1
///   buffer_store_dword v2, v1, s[0:3], 0 offen ; data[global_id] = v2
///   s_waitcnt     vmcnt(0)                    ; wait for store
///   s_endpgm                                  ; done
/// ```
pub static AGENT_INCR: &[u32] = &[
    // v_mov_b32 v1, 4
    // VOP1: [31:25]=0x7E, vdst=v1, op=V_MOV_B32(0x01), src0=inline_const_4(0x84)
    0x7E020284,
    // v_mul_lo_u32 v1, v0, v1
    // VOP3: op=V_MUL_LO_U32(0x169), vdst=v1, src0=v0(256), src1=v1(257)
    // Encoded as VOP3A: [31:26]=0x34, [25:17]=opcode[8:0], [16:0..]=...
    // For simplicity, use VOP2 v_lshlrev_b32 v1, 2, v0 (shift left by 2 = multiply by 4)
    // VOP2: [31]=0, [30:25]=opcode(V_LSHLREV_B32=0x12), [24:17]=vdst(1), [16:9]=vsrc1(v0=0x100), [8:0]=ssrc0(2=0x82)
    0x02020082 | (0x12 << 25),  // v_lshlrev_b32 v1, 2, v0
    // buffer_load_dword v2, v1, s[0:3], 0 offen
    // MUBUF: [31:26]=0x3A, [24:18]=op(BUFFER_LOAD_DWORD=0x14), [17]=idxen(0), [16]=offen(1)
    // Second dword: [7:0]=vdata(v2), [12:8]=vaddr(v1), [20:16]=soffset(0x80=off)
    0xE0502000, // MUBUF BUFFER_LOAD_DWORD offen
    0x80020100 | (1 << 8), // vdata=v2, vaddr=v1, srsrc=s[0:3], soffset=off
    // s_waitcnt vmcnt(0)
    // SOPP: prefix=0xBF, op=S_WAITCNT(0x0C), imm16 = 0x3F70 (vmcnt=0, expcnt=7, lgkmcnt=15)
    0xBF8C0070,
    // v_add_u32 v2, v2, 1
    // VOP2: [31]=0, [30:25]=V_ADD_U32(0x19), [24:17]=vdst(2), [16:9]=vsrc1(v2=0x102), [8:0]=ssrc0(1=0x81)
    // GFX10: V_ADD_NC_U32 opcode=0x25
    0x02040081 | (0x25 << 25), // v_add_nc_u32 v2, 1, v2  
    // buffer_store_dword v2, v1, s[0:3], 0 offen
    0xE0702000, // MUBUF BUFFER_STORE_DWORD offen  
    0x80020100 | (1 << 8), // vdata=v2, vaddr=v1, srsrc=s[0:3], soffset=off
    // s_waitcnt vmcnt(0)
    0xBF8C0070,
    // s_endpgm
    // SOPP: op=S_ENDPGM(0x01)
    0xBF810000,
];

/// Agent: MEMFILL — Fill buffer with constant value
///
/// Each work item writes a constant u32 to data[global_id].
///
/// Register setup:
///   s[0:3] = Buffer descriptor
///   s[4]   = Fill value (via USER_DATA_1)
///
/// ```
///   v_lshlrev_b32 v1, 2, v0                  ; byte_offset = global_id * 4
///   v_mov_b32     v2, s4                      ; v2 = fill value
///   buffer_store_dword v2, v1, s[0:3], 0 offen
///   s_waitcnt     vmcnt(0)
///   s_endpgm
/// ```
pub static AGENT_MEMFILL: &[u32] = &[
    // v_lshlrev_b32 v1, 2, v0
    0x02020082 | (0x12 << 25),
    // v_mov_b32 v2, s4  (s4 = inline SGPR #4 = 0x04)
    0x7E040204,
    // buffer_store_dword v2, v1, s[0:3], 0 offen
    0xE0702000,
    0x80020100 | (1 << 8),
    // s_waitcnt vmcnt(0)
    0xBF8C0070,
    // s_endpgm
    0xBF810000,
];

/// Agent: MEMCOPY — Copy from src buffer to dst buffer  
///
/// Register setup:
///   s[0:3] = Source buffer descriptor
///   s[4:7] = Destination buffer descriptor
///
/// ```
///   v_lshlrev_b32 v1, 2, v0                  ; byte offset
///   buffer_load_dword v2, v1, s[0:3], 0 offen ; load from src
///   s_waitcnt     vmcnt(0)
///   buffer_store_dword v2, v1, s[4:7], 0 offen ; store to dst
///   s_waitcnt     vmcnt(0)
///   s_endpgm
/// ```
pub static AGENT_MEMCOPY: &[u32] = &[
    // v_lshlrev_b32 v1, 2, v0
    0x02020082 | (0x12 << 25),
    // buffer_load_dword v2, v1, s[0:3], 0 offen
    0xE0502000,
    0x80020100 | (1 << 8),
    // s_waitcnt vmcnt(0)
    0xBF8C0070,
    // buffer_store_dword v2, v1, s[4:7], 0 offen (srsrc=s[4:7] → field = 1)
    0xE0702000,
    0x80020100 | (1 << 8) | (1 << 16), // srsrc=1 → s[4:7]
    // s_waitcnt vmcnt(0) 
    0xBF8C0070,
    // s_endpgm
    0xBF810000,
];

// ═══════════════════════════════════════════════════════════════════════════════
// Buffer Resource Descriptor (V# / T# — Buffer Descriptor for MUBUF)
// ═══════════════════════════════════════════════════════════════════════════════

/// Build a V# buffer resource descriptor (128-bit / 4 DWORDs)
/// Used by MUBUF instructions to access memory
///
/// Format (GFX10):
///   DW0: base_address[31:0]
///   DW1: base_address[47:32] | stride[29:16] 
///   DW2: num_records (number of elements)
///   DW3: dst_sel_x/y/z/w | num_format | data_format | element_size | ...
fn build_buffer_descriptor(gpu_addr: u64, num_elements: u32, stride: u32) -> [u32; 4] {
    let base_lo = (gpu_addr & 0xFFFFFFFF) as u32;
    let base_hi = ((gpu_addr >> 32) & 0xFFFF) as u32;
    // Stride in bits [29:16] of DW1
    let dw1 = base_hi | ((stride & 0x3FFF) << 16);
    // DW3: data_format=BUF_DATA_FORMAT_32(4), num_format=BUF_NUM_FORMAT_UINT(4)
    //      dst_sel = x,y,z,w = 4,5,6,7 (identity swizzle)
    //      For GFX10: dfmt=4(32bit), nfmt=4(uint) → bits [25:19]=nfmt, [18:15]=dfmt
    let dw3: u32 = (4 << 15) |  // data_format = 32-bit
                   (4 << 19) |  // num_format = UINT
                   (0 << 24) |  // element_size
                   (4 << 0)  |  // dst_sel_x = SEL_X(4)
                   (5 << 3)  |  // dst_sel_y = SEL_Y(5)  
                   (6 << 6)  |  // dst_sel_z = SEL_Z(6)
                   (7 << 9);    // dst_sel_w = SEL_W(7)
    [base_lo, dw1, num_elements, dw3]
}

// ═══════════════════════════════════════════════════════════════════════════════
// Compute Agent State
// ═══════════════════════════════════════════════════════════════════════════════

/// Named compute agent
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentKind {
    /// Increment each u32 in buffer by 1
    Incr,
    /// Fill buffer with constant value      
    MemFill,
    /// Copy from source buffer to destination buffer
    MemCopy,
}

impl AgentKind {
    pub fn name(&self) -> &'static str {
        match self {
            AgentKind::Incr => "incr",
            AgentKind::MemFill => "memfill",
            AgentKind::MemCopy => "memcopy",
        }
    }
    
    pub fn description(&self) -> &'static str {
        match self {
            AgentKind::Incr => "Increment each u32 by 1 (proof-of-life)",
            AgentKind::MemFill => "Fill buffer with constant u32 value",
            AgentKind::MemCopy => "GPU-speed buffer copy (src → dst)",
        }
    }

    /// Get the RDNA ISA binary for this agent
    pub fn shader_code(&self) -> &'static [u32] {
        match self {
            AgentKind::Incr => AGENT_INCR,
            AgentKind::MemFill => AGENT_MEMFILL,
            AgentKind::MemCopy => AGENT_MEMCOPY,
        }
    }
    
    /// Number of SGPRs used (for PGM_RSRC1)  
    pub fn sgpr_count(&self) -> u32 {
        match self {
            AgentKind::Incr => 4,     // s[0:3] buffer desc
            AgentKind::MemFill => 5,  // s[0:3] + s4 fill value
            AgentKind::MemCopy => 8,  // s[0:3] src + s[4:7] dst
        }
    }
    
    /// Number of VGPRs used (for PGM_RSRC1)
    pub fn vgpr_count(&self) -> u32 {
        match self {
            AgentKind::Incr => 3,    // v0=tid, v1=offset, v2=data
            AgentKind::MemFill => 3,
            AgentKind::MemCopy => 3,
        }
    }
    
    /// Number of USER_DATA SGPRs (buffer descriptors passed from CPU)
    pub fn user_sgpr_count(&self) -> u32 {
        match self {
            AgentKind::Incr => 4,
            AgentKind::MemFill => 5,
            AgentKind::MemCopy => 8,
        }
    }
}

/// All available agents
pub const ALL_AGENTS: &[AgentKind] = &[
    AgentKind::Incr,
    AgentKind::MemFill,
    AgentKind::MemCopy,
];

/// Compute dispatch state
struct ComputeState {
    initialized: bool,
    mmio_base: u64,
    /// Ring buffer virtual address
    ring_virt: u64,
    /// Ring buffer physical/GPU address
    ring_phys: u64,
    /// Data buffer virtual address (shader I/O + fence)
    data_virt: u64,
    /// Data buffer physical/GPU address  
    data_phys: u64,
    /// Shader code buffer virtual address
    code_virt: u64,
    /// Shader code buffer physical/GPU address
    code_phys: u64,
    /// Current ring write pointer (in DWORDs)  
    wptr: u32,
    /// Dispatches completed
    dispatch_count: u64,
}

static COMPUTE: Mutex<ComputeState> = Mutex::new(ComputeState {
    initialized: false,
    mmio_base: 0,
    ring_virt: 0,
    ring_phys: 0,
    data_virt: 0,
    data_phys: 0,
    code_virt: 0,
    code_phys: 0,
    wptr: 0,
    dispatch_count: 0,
});

static COMPUTE_READY: AtomicBool = AtomicBool::new(false);

// ═══════════════════════════════════════════════════════════════════════════════
// Ring Buffer Operations  
// ═══════════════════════════════════════════════════════════════════════════════

/// Write DWORDs to the ring buffer at the current write pointer
/// Returns the number of DWORDs written
fn ring_write(state: &mut ComputeState, data: &[u32]) -> usize {
    let ring = state.ring_virt as *mut u32;
    for (i, &dw) in data.iter().enumerate() {
        let idx = (state.wptr as usize + i) % RING_SIZE_DWORDS;
        unsafe {
            core::ptr::write_volatile(ring.add(idx), dw);
        }
    }
    state.wptr = (state.wptr + data.len() as u32) % RING_SIZE_DWORDS as u32;
    data.len()
}

/// Submit the ring buffer to the GPU by updating WPTR register
fn ring_submit(state: &ComputeState) {
    unsafe {
        // Write new WPTR to HQD register (in bytes, not dwords)
        let wptr_bytes = (state.wptr as u32) * 4;
        mmio_write32(state.mmio_base, regs::CP_HQD_PQ_WPTR_LO, wptr_bytes);
        mmio_write32(state.mmio_base, regs::CP_HQD_PQ_WPTR_HI, 0);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Initialization
// ═══════════════════════════════════════════════════════════════════════════════

/// Initialize the GPU compute engine
/// Must be called after amdgpu::init() has mapped MMIO
pub fn init(mmio_base: u64) {
    crate::log!("[GPU-COMPUTE] ═══════════════════════════════════════════════");
    crate::log!("[GPU-COMPUTE] Phase 3/4: Bare-metal RDNA Compute Agent");
    crate::log!("[GPU-COMPUTE] ═══════════════════════════════════════════════");
    
    if mmio_base == 0 {
        crate::log!("[GPU-COMPUTE] No MMIO base — skipping");
        return;
    }
    
    // Step 1: Allocate ring buffer (4KB, page-aligned → naturally GPU-aligned)
    let ring_layout = alloc::alloc::Layout::from_size_align(RING_SIZE_BYTES, 4096)
        .expect("ring layout");
    let ring_virt = unsafe { alloc::alloc::alloc_zeroed(ring_layout) } as u64;
    let ring_phys = memory::virt_to_phys(ring_virt).unwrap_or(0);
    
    if ring_phys == 0 {
        crate::log!("[GPU-COMPUTE] ERROR: Cannot get physical address for ring buffer");
        return;
    }
    crate::log!("[GPU-COMPUTE] Ring buffer: virt={:#X} phys={:#X} size={} dwords",
        ring_virt, ring_phys, RING_SIZE_DWORDS);
    
    // Step 2: Allocate data buffer (64KB — shader I/O + fence)
    let data_layout = alloc::alloc::Layout::from_size_align(DATA_BUFFER_SIZE, 4096)
        .expect("data layout");
    let data_virt = unsafe { alloc::alloc::alloc_zeroed(data_layout) } as u64;
    let data_phys = memory::virt_to_phys(data_virt).unwrap_or(0);
    
    if data_phys == 0 {
        crate::log!("[GPU-COMPUTE] ERROR: Cannot get physical address for data buffer");
        return;
    }
    crate::log!("[GPU-COMPUTE] Data buffer: virt={:#X} phys={:#X} size={}KB",
        data_virt, data_phys, DATA_BUFFER_SIZE / 1024);
    
    // Step 3: Allocate shader code buffer (4KB — holds RDNA ISA binaries)
    let code_layout = alloc::alloc::Layout::from_size_align(4096, 256)
        .expect("code layout");
    let code_virt = unsafe { alloc::alloc::alloc_zeroed(code_layout) } as u64;
    let code_phys = memory::virt_to_phys(code_virt).unwrap_or(0);
    
    if code_phys == 0 {
        crate::log!("[GPU-COMPUTE] ERROR: Cannot get physical address for code buffer");
        return;
    }
    crate::log!("[GPU-COMPUTE] Code buffer: virt={:#X} phys={:#X}", code_virt, code_phys);
    
    // Step 4: Read MEC/CP status
    let grbm_status = unsafe { mmio_read32(mmio_base, regs::GRBM_STATUS) };
    let cp_me_cntl = unsafe { mmio_read32(mmio_base, regs::CP_ME_CNTL) };
    crate::log!("[GPU-COMPUTE] GRBM_STATUS={:#010X} CP_ME_CNTL={:#010X}", grbm_status, cp_me_cntl);
    
    let gui_active = (grbm_status & regs::GRBM_STATUS_GUI_ACTIVE) != 0;
    let cp_busy = (grbm_status & regs::GRBM_STATUS_CP_BUSY) != 0;
    crate::log!("[GPU-COMPUTE] GUI_ACTIVE={} CP_BUSY={}", gui_active, cp_busy);
    
    // Step 5: Initialize HQD (Hardware Queue Descriptor) for compute queue 0
    crate::log!("[GPU-COMPUTE] Configuring HQD for compute queue...");
    unsafe {
        // Deactivate queue first
        mmio_write32(mmio_base, regs::CP_HQD_ACTIVE, 0);
        
        // Set ring buffer base address (in 256-byte units for GFX10)
        let rb_base_256 = ring_phys >> 8;
        mmio_write32(mmio_base, regs::CP_HQD_PQ_BASE_LO, (rb_base_256 & 0xFFFFFFFF) as u32);
        mmio_write32(mmio_base, regs::CP_HQD_PQ_BASE_HI, ((rb_base_256 >> 32) & 0xFF) as u32);
        
        // Set ring buffer control: size = log2(1024) = 10, rptr_block_size = 6
        // Bits: [5:0]=rptr_block_size(6), [7:6]=min_avail_size(0), [25:8]=rb_bufsz(10)
        let pq_control = (6 << 0) | (10 << 8);
        mmio_write32(mmio_base, regs::CP_HQD_PQ_CONTROL, pq_control);
        
        // Reset read/write pointers
        mmio_write32(mmio_base, regs::CP_HQD_PQ_RPTR, 0);
        mmio_write32(mmio_base, regs::CP_HQD_PQ_WPTR_LO, 0);
        mmio_write32(mmio_base, regs::CP_HQD_PQ_WPTR_HI, 0);
        
        // Activate the queue
        mmio_write32(mmio_base, regs::CP_HQD_ACTIVE, 1);
    }
    
    crate::log!("[GPU-COMPUTE] HQD configured: base={:#X} size={}dw", ring_phys, RING_SIZE_DWORDS);
    
    // Store state
    let mut state = COMPUTE.lock();
    state.initialized = true;
    state.mmio_base = mmio_base;
    state.ring_virt = ring_virt;
    state.ring_phys = ring_phys;
    state.data_virt = data_virt;
    state.data_phys = data_phys;
    state.code_virt = code_virt;
    state.code_phys = code_phys;
    state.wptr = 0;
    state.dispatch_count = 0;
    drop(state);
    
    COMPUTE_READY.store(true, Ordering::SeqCst);
    
    // Step 6: Report available agents
    crate::log!("[GPU-COMPUTE] ───────────────────────────────────────────────");
    crate::log!("[GPU-COMPUTE] Available agents:");
    for agent in ALL_AGENTS {
        crate::log!("[GPU-COMPUTE]   {} — {}", agent.name(), agent.description());
    }
    crate::log!("[GPU-COMPUTE] ───────────────────────────────────────────────");
    crate::log!("[GPU-COMPUTE] Compute engine ready — dispatch via `gpuexec`");
}

// ═══════════════════════════════════════════════════════════════════════════════
// Dispatch API
// ═══════════════════════════════════════════════════════════════════════════════

/// Dispatch a compute agent on the GPU
///
/// # Arguments
/// * `agent` — Which pre-assembled RDNA kernel to run
/// * `num_elements` — Number of u32 elements to process
/// * `fill_value` — Constant value (used by MemFill agent; ignored by others)
///
/// # Returns
/// Ok(elapsed_iters) on success, Err(description) on failure
pub fn dispatch(agent: AgentKind, num_elements: u32, fill_value: u32) -> Result<u64, &'static str> {
    if !COMPUTE_READY.load(Ordering::Relaxed) {
        return Err("GPU compute engine not initialized");
    }
    
    let mut state = COMPUTE.lock();
    let mmio = state.mmio_base;
    
    // Clamp to data buffer size (minus fence area)
    let max_elements = ((DATA_BUFFER_SIZE - 64) / 4) as u32;
    let num_elements = num_elements.min(max_elements);
    
    // Step 1: Initialize data buffer (for INCR agent, fill with sequential values)
    let data_ptr = state.data_virt as *mut u32;
    match agent {
        AgentKind::Incr => {
            for i in 0..num_elements {
                unsafe { core::ptr::write_volatile(data_ptr.add(i as usize), i); }
            }
        }
        AgentKind::MemFill => {
            // Buffer will be overwritten by GPU
            for i in 0..num_elements {
                unsafe { core::ptr::write_volatile(data_ptr.add(i as usize), 0); }
            }
        }
        AgentKind::MemCopy => {
            // First half = source data, second half = destination (zeroed)
            let half = num_elements / 2;
            for i in 0..half {
                unsafe { core::ptr::write_volatile(data_ptr.add(i as usize), 0xA0A0_0000 + i); }
            }
            for i in half..num_elements {
                unsafe { core::ptr::write_volatile(data_ptr.add(i as usize), 0); }
            }
        }
    }
    
    // Clear fence
    let fence_ptr = (state.data_virt + FENCE_OFFSET as u64) as *mut u64;
    unsafe { core::ptr::write_volatile(fence_ptr, 0); }
    
    // Step 2: Upload shader code to code buffer
    let shader = agent.shader_code();
    let code_ptr = state.code_virt as *mut u32;
    for (i, &insn) in shader.iter().enumerate() {
        unsafe { core::ptr::write_volatile(code_ptr.add(i), insn); }
    }
    
    // Step 3: Build buffer descriptor for data
    let buf_desc = build_buffer_descriptor(state.data_phys, num_elements, 4);
    
    // Step 4: Encode PGM_RSRC1 register
    // Bits: [5:0]=vgprs (granularity 8, so count/8 - 1), [11:6]=sgprs (granularity 8)
    let vgpr_blocks = (agent.vgpr_count() + 7) / 8;
    let sgpr_blocks = (agent.sgpr_count() + 7) / 8;
    let pgm_rsrc1 = ((vgpr_blocks.saturating_sub(1)) & 0x3F) |
                    (((sgpr_blocks.saturating_sub(1)) & 0xF) << 6) |
                    (3 << 24); // float_mode = 0x3 (default IEEE)
    
    // PGM_RSRC2: user_sgpr_count, no LDS, no scratch
    let pgm_rsrc2 = agent.user_sgpr_count() & 0x1F; // bits [4:0]
    
    // Shader address in 256-byte units
    let shader_addr_256 = state.code_phys >> 8;
    
    // Step 5: Build PM4 command stream
    // Reset ring write pointer for clean dispatch
    state.wptr = 0;
    
    // SET_SH_REG: COMPUTE_PGM_LO/HI (shader program address)
    let pgm_pkt = pm4_set_sh_reg2(
        regs::COMPUTE_PGM_LO,
        (shader_addr_256 & 0xFFFFFFFF) as u32,
        ((shader_addr_256 >> 32) & 0xFFFF) as u32,
    );
    ring_write(&mut state, &pgm_pkt);
    
    // SET_SH_REG: COMPUTE_PGM_RSRC1
    let rsrc1_pkt = pm4_set_sh_reg(regs::COMPUTE_PGM_RSRC1, pgm_rsrc1);
    ring_write(&mut state, &rsrc1_pkt);
    
    // SET_SH_REG: COMPUTE_PGM_RSRC2
    let rsrc2_pkt = pm4_set_sh_reg(regs::COMPUTE_PGM_RSRC2, pgm_rsrc2);
    ring_write(&mut state, &rsrc2_pkt);
    
    // SET_SH_REG: COMPUTE_NUM_THREAD_X/Y/Z (workgroup size = 64 threads, RDNA wavefront)
    let thread_x_pkt = pm4_set_sh_reg(regs::COMPUTE_NUM_THREAD_X, 64);
    ring_write(&mut state, &thread_x_pkt);
    let thread_y_pkt = pm4_set_sh_reg(regs::COMPUTE_NUM_THREAD_Y, 1);
    ring_write(&mut state, &thread_y_pkt);
    let thread_z_pkt = pm4_set_sh_reg(regs::COMPUTE_NUM_THREAD_Z, 1);
    ring_write(&mut state, &thread_z_pkt);
    
    // SET_SH_REG: USER_DATA[0:3] = buffer descriptor
    for (i, &dw) in buf_desc.iter().enumerate() {
        let reg = regs::COMPUTE_USER_DATA_0 + (i as u32) * 4;
        let pkt = pm4_set_sh_reg(reg, dw);
        ring_write(&mut state, &pkt);
    }
    
    // If MemFill, set USER_DATA[4] = fill value
    if agent == AgentKind::MemFill {
        let pkt = pm4_set_sh_reg(regs::COMPUTE_USER_DATA_0 + 16, fill_value);
        ring_write(&mut state, &pkt);
    }
    
    // DISPATCH_DIRECT: launch workgroups
    // num_workgroups = ceil(num_elements / 64)
    let num_groups = (num_elements + 63) / 64;
    let dispatch_pkt = pm4_dispatch_direct(num_groups, 1, 1);
    ring_write(&mut state, &dispatch_pkt);
    
    // RELEASE_MEM: write fence when done
    let fence_gpu_addr = state.data_phys + FENCE_OFFSET as u64;
    let release_pkt = pm4_release_mem(fence_gpu_addr, FENCE_SIGNAL_VALUE);
    ring_write(&mut state, &release_pkt);
    
    // Pad to alignment (NOP)
    let nop = pm4_nop();
    ring_write(&mut state, &nop);
    
    // Step 6: Submit to GPU
    crate::serial_println!("[GPU-COMPUTE] Submitting {} agent: {} elements, {} workgroups",
        agent.name(), num_elements, num_groups);
    crate::serial_println!("[GPU-COMPUTE]   Ring WPTR: {} dwords", state.wptr);
    crate::serial_println!("[GPU-COMPUTE]   Shader: {} insns at phys {:#X}", shader.len(), state.code_phys);
    
    ring_submit(&state);
    
    // Step 7: Poll for fence completion
    let mut elapsed = 0u64;
    loop {
        let current_fence = unsafe { core::ptr::read_volatile(fence_ptr) };
        if current_fence == FENCE_SIGNAL_VALUE {
            break;
        }
        elapsed += 1;
        if elapsed >= GPU_TIMEOUT_ITERS {
            crate::serial_println!("[GPU-COMPUTE] TIMEOUT after {} iterations (fence={:#X})",
                elapsed, current_fence);
            // Read GRBM status for diagnostics
            let grbm = unsafe { mmio_read32(mmio, regs::GRBM_STATUS) };
            let rptr = unsafe { mmio_read32(mmio, regs::CP_HQD_PQ_RPTR) };
            crate::serial_println!("[GPU-COMPUTE]   GRBM_STATUS={:#010X} RPTR={}", grbm, rptr);
            state.dispatch_count += 1;
            return Err("GPU dispatch timed out (fence not signaled)");
        }
        // Small delay between polls to not flood the bus
        if elapsed % 100 == 0 {
            core::hint::spin_loop();
        }
    }
    
    state.dispatch_count += 1;
    crate::serial_println!("[GPU-COMPUTE] Dispatch complete in {} poll iterations", elapsed);
    
    Ok(elapsed)
}

// ═══════════════════════════════════════════════════════════════════════════════
// Verification / Readback
// ═══════════════════════════════════════════════════════════════════════════════

/// Read back and verify results from the last dispatch
pub fn verify_results(agent: AgentKind, num_elements: u32, fill_value: u32) -> (u32, u32) {
    let state = COMPUTE.lock();
    let data_ptr = state.data_virt as *const u32;
    let mut pass = 0u32;
    let mut fail = 0u32;
    
    let check_count = num_elements.min(((DATA_BUFFER_SIZE - 64) / 4) as u32);
    
    for i in 0..check_count {
        let actual = unsafe { core::ptr::read_volatile(data_ptr.add(i as usize)) };
        let expected = match agent {
            AgentKind::Incr => i + 1, // Should be original value + 1
            AgentKind::MemFill => fill_value,
            AgentKind::MemCopy => {
                let half = check_count / 2;
                if i >= half {
                    // Destination half should contain source data
                    0xA0A0_0000 + (i - half)
                } else {
                    // Source half unchanged
                    0xA0A0_0000 + i
                }
            }
        };
        if actual == expected {
            pass += 1;
        } else {
            fail += 1;
            // Log first few failures
            if fail <= 8 {
                crate::serial_println!("[GPU-COMPUTE] VERIFY[{}]: expected {:#010X} got {:#010X}",
                    i, expected, actual);
            }
        }
    }
    
    (pass, fail)
}

/// Read a single element from the data buffer
pub fn read_data(index: u32) -> Option<u32> {
    let state = COMPUTE.lock();
    if !state.initialized {
        return None;
    }
    let max = ((DATA_BUFFER_SIZE - 64) / 4) as u32;
    if index >= max {
        return None;
    }
    let ptr = state.data_virt as *const u32;
    Some(unsafe { core::ptr::read_volatile(ptr.add(index as usize)) })
}

// ═══════════════════════════════════════════════════════════════════════════════
// Public API
// ═══════════════════════════════════════════════════════════════════════════════

/// Check if compute engine is ready
pub fn is_ready() -> bool {
    COMPUTE_READY.load(Ordering::Relaxed)
}

/// Get dispatch count
pub fn dispatch_count() -> u64 {
    COMPUTE.lock().dispatch_count
}

/// Get status summary  
pub fn summary() -> String {
    if is_ready() {
        let state = COMPUTE.lock();
        format!("GPU Compute: {} agents, {} dispatches, ring@{:#X}",
            ALL_AGENTS.len(), state.dispatch_count, state.ring_phys)
    } else {
        String::from("GPU Compute: not initialized")
    }
}

/// Get detailed info for terminal display
pub fn info_lines() -> Vec<String> {
    let mut lines = Vec::new();
    
    if is_ready() {
        let state = COMPUTE.lock();
        lines.push(String::from("╔══════════════════════════════════════════════════╗"));
        lines.push(String::from("║    GPU Compute Agent — Bare-metal RDNA Dispatch  ║"));
        lines.push(String::from("╠══════════════════════════════════════════════════╣"));
        lines.push(format!("║ Ring Buffer:  {:#X} ({} dwords)          ║", state.ring_phys, RING_SIZE_DWORDS));
        lines.push(format!("║ Data Buffer:  {:#X} ({}KB)              ║", state.data_phys, DATA_BUFFER_SIZE/1024));
        lines.push(format!("║ Code Buffer:  {:#X}                     ║", state.code_phys));
        lines.push(format!("║ Dispatches:   {}                                  ║", state.dispatch_count));
        lines.push(format!("║ Ring WPTR:    {}                                  ║", state.wptr));
        lines.push(String::from("╠══════════════════════════════════════════════════╣"));
        lines.push(String::from("║ Available Agents:                                ║"));
        for agent in ALL_AGENTS {
            lines.push(format!("║  {:10} — {}  ║", agent.name(), agent.description()));
        }
        lines.push(String::from("╚══════════════════════════════════════════════════╝"));
    } else {
        lines.push(String::from("GPU Compute Agent not initialized"));
        lines.push(String::from("(Requires AMD GPU with MMIO access)"));
    }
    
    lines
}
