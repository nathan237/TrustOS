//! AMD GPU SDMA Engine — Bare-metal DMA transfers for TrustOS
//!
//! The SDMA (System DMA) engines on Navi 10 operate independently of the
//! Graphics/Compute command processors and provide high-throughput async
//! memory transfers between system RAM and VRAM (and VRAM↔VRAM).
//!
//! Architecture:
//! ```
//! TrustOS CPU  →  SDMA Ring Buffer  →  SDMA Engine (hardware DMA)
//!                                            ↓
//!                                       System RAM ↔ VRAM
//!                                       (up to 448 GB/s to VRAM)
//! ```
//!
//! Navi 10 has 2 independent SDMA engines (SDMA0 + SDMA1).
//! Each engine has its own GFX ring buffer for command submission.
//!
//! SDMA packet format (different from PM4):
//!   DW0: [31:28]=0, [27:26]=sub_op, [25:8]=op, [7:0]=extra_info
//!
//! Supported operations:
//! - **LINEAR COPY**: Copy N bytes from src_addr to dst_addr
//! - **CONST FILL**: Fill N bytes at dst_addr with a 32-bit pattern
//! - **FENCE**: Write a 32-bit value to a memory address (completion signal)
//! - **TIMESTAMP**: Write GPU clock to memory address
//! - **NOP**: Ring buffer padding
//!
//! This module is the foundation for loading AI model weights into VRAM.
//! With 448 GB/s VRAM bandwidth, a 1.5 GB model loads in ~3.3ms to VRAM.
//!
//! References:
//! - Linux: drivers/gpu/drm/amd/amdgpu/sdma_v5_0.c
//! - Linux: drivers/gpu/drm/amd/amdgpu/navi10_sdma_pkt_open.h
//! - AMD SDMA Programming Guide (under NDA, register layout from Linux source)

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

/// Ring buffer size: 16KB = 4096 DWORDs
/// Larger than compute ring because DMA packets are bigger (7-8 DWORDs each)
const RING_SIZE_DWORDS: usize = 4096;
const RING_SIZE_BYTES: usize = RING_SIZE_DWORDS * 4;
/// Ring size as log2(dwords) for the CNTL register
const RING_SIZE_LOG2: u32 = 12; // 2^12 = 4096 dwords

/// Staging buffer for DMA data: 256KB
/// Used for CPU→GPU transfers: CPU writes here, SDMA copies to VRAM
const STAGING_BUFFER_SIZE: usize = 256 * 1024;

/// Fence/status area: 4KB page (contains fence values + RPTR writeback)
const STATUS_BUFFER_SIZE: usize = 4096;

/// Fence offset within status buffer (per engine)
const FENCE_OFFSET_E0: usize = 0x00;
const FENCE_OFFSET_E1: usize = 0x10;
/// RPTR writeback offset (per engine)
const RPTR_WB_OFFSET_E0: usize = 0x100;
const RPTR_WB_OFFSET_E1: usize = 0x110;

/// Timeout for SDMA operations (polling iterations)
const SDMA_TIMEOUT_ITERS: u64 = 10_000_000;

/// Performance counter: track total bytes transferred
static TOTAL_BYTES_TRANSFERRED: AtomicU64 = AtomicU64::new(0);
static TOTAL_TRANSFERS: AtomicU64 = AtomicU64::new(0);

// ═══════════════════════════════════════════════════════════════════════════════
// SDMA Packet Builders
// ═══════════════════════════════════════════════════════════════════════════════
//
// SDMA packets use a different format than PM4:
//   DW0: [31:28] = 0 (always)
//        [27:26] = sub_op
//        [25:8]  = opcode
//        [7:0]   = extra_info (varies by opcode)
//

/// Build SDMA packet header
#[inline]
fn sdma_header(op: u32, sub_op: u32) -> u32 {
    ((sub_op & 0x3) << 26) | ((op & 0x3FFFF) << 8)
}

/// Build a NOP packet (1 DWORD)
#[inline]
fn sdma_nop() -> u32 {
    sdma_header(regs::SDMA_OP_NOP, 0)
}

/// Build a LINEAR COPY packet (7 DWORDs)
///
/// Copies `byte_count` bytes from `src_addr` to `dst_addr`.
/// Both addresses must be GPU-visible (physical or GART).
///
/// Format:
///   DW0: header (op=COPY, sub_op=LINEAR)
///   DW1: byte_count (up to 2^26 = 64MB per packet)
///   DW2: 0 (parameter)
///   DW3: src_addr[31:0]
///   DW4: src_addr[63:32]
///   DW5: dst_addr[31:0]
///   DW6: dst_addr[63:32]
fn sdma_copy_linear(src_addr: u64, dst_addr: u64, byte_count: u32) -> [u32; 7] {
    [
        sdma_header(regs::SDMA_OP_COPY, regs::SDMA_COPY_SUB_LINEAR),
        byte_count,
        0, // parameter (src/dst array pitch for 2D — 0 for linear)
        (src_addr & 0xFFFFFFFF) as u32,
        ((src_addr >> 32) & 0xFFFFFFFF) as u32,
        (dst_addr & 0xFFFFFFFF) as u32,
        ((dst_addr >> 32) & 0xFFFFFFFF) as u32,
    ]
}

/// Build a CONST_FILL packet (5 DWORDs)
///
/// Fills `byte_count` bytes at `dst_addr` with `fill_value` (32-bit pattern).
/// byte_count must be a multiple of 4.
///
/// Format:
///   DW0: header (op=CONST_FILL, sub_op=0)
///   DW1: dst_addr[31:0]
///   DW2: dst_addr[63:32]
///   DW3: fill_value (32-bit constant)
///   DW4: byte_count
fn sdma_const_fill(dst_addr: u64, fill_value: u32, byte_count: u32) -> [u32; 5] {
    [
        sdma_header(regs::SDMA_OP_CONST_FILL, 0),
        (dst_addr & 0xFFFFFFFF) as u32,
        ((dst_addr >> 32) & 0xFFFFFFFF) as u32,
        fill_value,
        byte_count,
    ]
}

/// Build a FENCE packet (4 DWORDs)
///
/// Writes a 32-bit value to a memory address when all preceding commands complete.
/// This is how we know a DMA transfer is done.
///
/// Format:
///   DW0: header (op=FENCE, sub_op=0)
///   DW1: addr[31:0]
///   DW2: addr[63:32]
///   DW3: fence_value
fn sdma_fence(addr: u64, value: u32) -> [u32; 4] {
    [
        sdma_header(regs::SDMA_OP_FENCE, 0),
        (addr & 0xFFFFFFFF) as u32,
        ((addr >> 32) & 0xFFFFFFFF) as u32,
        value,
    ]
}

/// Build a TIMESTAMP packet (3 DWORDs)
///
/// Writes the GPU's 64-bit clock counter to a memory address.
/// Useful for measuring DMA transfer latency.
///
/// Format:
///   DW0: header (op=TIMESTAMP, sub_op=0)
///   DW1: addr[31:0]
///   DW2: addr[63:32]
fn sdma_timestamp(addr: u64) -> [u32; 3] {
    [
        sdma_header(regs::SDMA_OP_TIMESTAMP, 0),
        (addr & 0xFFFFFFFF) as u32,
        ((addr >> 32) & 0xFFFFFFFF) as u32,
    ]
}

/// Build a WRITE (immediate) packet (4+ DWORDs)
///
/// Writes one or more DWORDs directly to a GPU address.
/// Useful for small register-style writes via DMA engine.
///
/// Format:
///   DW0: header (op=WRITE, sub_op=LINEAR)
///   DW1: dst_addr[31:0]
///   DW2: dst_addr[63:32]
///   DW3: count-1 (number of DWORDs to write, minus 1)
///   DW4..N: data DWORDs
fn sdma_write_hdr(dst_addr: u64, count: u32) -> [u32; 4] {
    [
        sdma_header(regs::SDMA_OP_WRITE, regs::SDMA_WRITE_SUB_LINEAR),
        (dst_addr & 0xFFFFFFFF) as u32,
        ((dst_addr >> 32) & 0xFFFFFFFF) as u32,
        count.saturating_sub(1),
    ]
}

// ═══════════════════════════════════════════════════════════════════════════════
// Per-Engine State
// ═══════════════════════════════════════════════════════════════════════════════

/// State for a single SDMA engine
struct SdmaEngine {
    /// Engine index (0 or 1)
    index: usize,
    /// MMIO base address (kernel virtual)
    mmio_base: u64,
    /// Register base offset for this engine's GFX ring
    reg_base: u32,
    /// Ring buffer virtual address
    ring_virt: u64,
    /// Ring buffer physical address (GPU-visible)
    ring_phys: u64,
    /// Current write pointer (in DWORDs, wrapping at RING_SIZE_DWORDS)
    wptr: u32,
    /// Fence counter (monotonically increasing)
    fence_seq: u32,
    /// Transfer count
    transfers: u64,
    /// Bytes transferred
    bytes: u64,
}

/// Driver-wide SDMA state
struct SdmaState {
    initialized: bool,
    mmio_base: u64,
    /// Both SDMA engines
    engines: [Option<SdmaEngine>; 2],
    /// Status buffer virtual/physical (shared by both engines)
    status_virt: u64,
    status_phys: u64,
    /// Staging buffer virtual/physical (for CPU→GPU staging)
    staging_virt: u64,
    staging_phys: u64,
}

static SDMA_STATE: Mutex<SdmaState> = Mutex::new(SdmaState {
    initialized: false,
    mmio_base: 0,
    engines: [None, None],
    status_virt: 0,
    status_phys: 0,
    staging_virt: 0,
    staging_phys: 0,
});

static SDMA_READY: AtomicBool = AtomicBool::new(false);

// ═══════════════════════════════════════════════════════════════════════════════
// Ring Buffer Operations
// ═══════════════════════════════════════════════════════════════════════════════

/// Write DWORDs to an engine's ring buffer
fn ring_write(engine: &mut SdmaEngine, data: &[u32]) {
    let ring = engine.ring_virt as *mut u32;
    for (i, &dw) in data.iter().enumerate() {
        let idx = (engine.wptr as usize + i) % RING_SIZE_DWORDS;
        unsafe {
            core::ptr::write_volatile(ring.add(idx), dw);
        }
    }
    engine.wptr = ((engine.wptr as usize + data.len()) % RING_SIZE_DWORDS) as u32;
}

/// Submit the ring buffer by updating WPTR register
fn ring_submit(engine: &SdmaEngine) {
    unsafe {
        // SDMA WPTR is in bytes, not dwords
        let wptr_bytes = engine.wptr * 4;
        let wptr_reg = engine.reg_base + regs::SDMA_GFX_RB_WPTR;
        mmio_write32(engine.mmio_base, wptr_reg, wptr_bytes);
        mmio_write32(engine.mmio_base, wptr_reg + 4, 0); // WPTR_HI
    }
}

/// Read the hardware RPTR (in DWORDs)
fn ring_rptr(engine: &SdmaEngine) -> u32 {
    unsafe {
        let rptr_reg = engine.reg_base + regs::SDMA_GFX_RB_RPTR;
        let rptr_bytes = mmio_read32(engine.mmio_base, rptr_reg);
        rptr_bytes / 4
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Engine Initialization
// ═══════════════════════════════════════════════════════════════════════════════

/// Initialize a single SDMA engine
fn init_engine(
    mmio_base: u64,
    engine_idx: usize,
    ring_virt: u64,
    ring_phys: u64,
    status_phys: u64,
    rptr_wb_offset: usize,
) -> Option<SdmaEngine> {
    let base = if engine_idx == 0 {
        regs::SDMA0_BASE
    } else {
        regs::SDMA1_BASE
    };

    crate::log!("[SDMA{}] Initializing engine (reg_base={:#X})", engine_idx, base);

    unsafe {
        // Step 1: Read engine status
        let status_reg = if engine_idx == 0 {
            regs::SDMA0_STATUS_REG
        } else {
            regs::SDMA1_STATUS_REG
        };
        let status = mmio_read32(mmio_base, status_reg);
        crate::log!("[SDMA{}] STATUS={:#010X} (idle={})",
            engine_idx, status, (status & regs::SDMA_STATUS_IDLE) != 0);

        // Step 2: Halt engine by setting F32_CNTL halt bit
        let f32_reg = if engine_idx == 0 {
            regs::SDMA0_F32_CNTL
        } else {
            regs::SDMA1_F32_CNTL
        };
        mmio_write32(mmio_base, f32_reg, 1); // HALT=1

        // Small delay for halt to take effect
        for _ in 0..1000 {
            core::hint::spin_loop();
        }

        // Step 3: Disable GFX ring
        let rb_cntl_reg = base + regs::SDMA_GFX_RB_CNTL;
        mmio_write32(mmio_base, rb_cntl_reg, 0); // Disable ring

        // Step 4: Set ring buffer base address (256-byte aligned, store in 256B units)
        let rb_base_256 = ring_phys >> 8;
        let rb_base_reg = base + regs::SDMA_GFX_RB_BASE;
        let rb_base_hi_reg = base + regs::SDMA_GFX_RB_BASE_HI;
        mmio_write32(mmio_base, rb_base_reg, (rb_base_256 & 0xFFFFFFFF) as u32);
        mmio_write32(mmio_base, rb_base_hi_reg, ((rb_base_256 >> 32) & 0xFFFFFFFF) as u32);

        // Step 5: Clear RPTR/WPTR
        mmio_write32(mmio_base, base + regs::SDMA_GFX_RB_RPTR, 0);
        mmio_write32(mmio_base, base + regs::SDMA_GFX_RB_RPTR_HI, 0);
        mmio_write32(mmio_base, base + regs::SDMA_GFX_RB_WPTR, 0);
        mmio_write32(mmio_base, base + regs::SDMA_GFX_RB_WPTR_HI, 0);

        // Step 6: Set RPTR writeback address (GPU writes RPTR here so CPU can track)
        let rptr_wb_addr = status_phys + rptr_wb_offset as u64;
        mmio_write32(mmio_base, base + regs::SDMA_GFX_RB_RPTR_ADDR_LO,
            (rptr_wb_addr & 0xFFFFFFFF) as u32);
        mmio_write32(mmio_base, base + regs::SDMA_GFX_RB_RPTR_ADDR_HI,
            ((rptr_wb_addr >> 32) & 0xFFFFFFFF) as u32);

        // Step 7: Configure ring control and enable
        //  - RB_SIZE = log2(4096 dwords) = 12, shifted into bits [6:1]
        //  - RPTR_WRITEBACK_ENABLE = 1 (bit 12)
        //  - RB_ENABLE = 1 (bit 0)
        //  - VMID = 0 (bits [19:16], bare-metal, no IOMMU translation)
        let rb_cntl = regs::SDMA_RB_CNTL_RB_ENABLE
            | (RING_SIZE_LOG2 << regs::SDMA_RB_CNTL_RB_SIZE_SHIFT)
            | regs::SDMA_RB_CNTL_RPTR_WRITEBACK_ENABLE;
        mmio_write32(mmio_base, rb_cntl_reg, rb_cntl);

        // Step 8: Un-halt engine
        mmio_write32(mmio_base, f32_reg, 0); // HALT=0

        // Step 9: Verify engine came back alive
        for _ in 0..10000 {
            core::hint::spin_loop();
        }
        let status_after = mmio_read32(mmio_base, status_reg);
        crate::log!("[SDMA{}] Post-init STATUS={:#010X}", engine_idx, status_after);
    }

    Some(SdmaEngine {
        index: engine_idx,
        mmio_base,
        reg_base: base,
        ring_virt,
        ring_phys,
        wptr: 0,
        fence_seq: 1,
        transfers: 0,
        bytes: 0,
    })
}

// ═══════════════════════════════════════════════════════════════════════════════
// Public Initialization
// ═══════════════════════════════════════════════════════════════════════════════

/// Initialize both SDMA engines.
/// Must be called after amdgpu::init() has mapped MMIO.
pub fn init(mmio_base: u64) {
    crate::log!("[SDMA] ═══════════════════════════════════════════════════");
    crate::log!("[SDMA] SDMA Engine — Bare-metal DMA transfers (Navi 10)");
    crate::log!("[SDMA] ═══════════════════════════════════════════════════");

    if mmio_base == 0 {
        crate::log!("[SDMA] No MMIO base — skipping");
        return;
    }

    // Step 1: Read SDMA version
    let sdma_ver = unsafe { mmio_read32(mmio_base, regs::SDMA0_VERSION) };
    crate::log!("[SDMA] SDMA0 VERSION={:#010X}", sdma_ver);

    // Step 2: Allocate ring buffers (one per engine, 16KB each, page-aligned)
    let ring_layout = alloc::alloc::Layout::from_size_align(RING_SIZE_BYTES, 4096)
        .expect("sdma ring layout");

    let ring0_virt = unsafe { alloc::alloc::alloc_zeroed(ring_layout) } as u64;
    let ring0_phys = memory::virt_to_phys(ring0_virt).unwrap_or(0);

    let ring1_virt = unsafe { alloc::alloc::alloc_zeroed(ring_layout) } as u64;
    let ring1_phys = memory::virt_to_phys(ring1_virt).unwrap_or(0);

    if ring0_phys == 0 || ring1_phys == 0 {
        crate::log!("[SDMA] ERROR: Cannot get physical address for ring buffers");
        return;
    }

    crate::log!("[SDMA] Ring0: virt={:#X} phys={:#X} ({} dwords)",
        ring0_virt, ring0_phys, RING_SIZE_DWORDS);
    crate::log!("[SDMA] Ring1: virt={:#X} phys={:#X} ({} dwords)",
        ring1_virt, ring1_phys, RING_SIZE_DWORDS);

    // Step 3: Allocate status buffer (fence + RPTR writeback, 4KB)
    let status_layout = alloc::alloc::Layout::from_size_align(STATUS_BUFFER_SIZE, 4096)
        .expect("sdma status layout");
    let status_virt = unsafe { alloc::alloc::alloc_zeroed(status_layout) } as u64;
    let status_phys = memory::virt_to_phys(status_virt).unwrap_or(0);

    if status_phys == 0 {
        crate::log!("[SDMA] ERROR: Cannot get phys for status buffer");
        return;
    }
    crate::log!("[SDMA] Status: virt={:#X} phys={:#X}", status_virt, status_phys);

    // Step 4: Allocate staging buffer (CPU→GPU data staging, 256KB)
    let staging_layout = alloc::alloc::Layout::from_size_align(STAGING_BUFFER_SIZE, 4096)
        .expect("sdma staging layout");
    let staging_virt = unsafe { alloc::alloc::alloc_zeroed(staging_layout) } as u64;
    let staging_phys = memory::virt_to_phys(staging_virt).unwrap_or(0);

    if staging_phys == 0 {
        crate::log!("[SDMA] ERROR: Cannot get phys for staging buffer");
        return;
    }
    crate::log!("[SDMA] Staging: virt={:#X} phys={:#X} ({}KB)",
        staging_virt, staging_phys, STAGING_BUFFER_SIZE / 1024);

    // Step 5: Initialize both engines
    let engine0 = init_engine(
        mmio_base, 0, ring0_virt, ring0_phys, status_phys, RPTR_WB_OFFSET_E0,
    );
    let engine1 = init_engine(
        mmio_base, 1, ring1_virt, ring1_phys, status_phys, RPTR_WB_OFFSET_E1,
    );

    let e0_ok = engine0.is_some();
    let e1_ok = engine1.is_some();

    // Store state
    let mut state = SDMA_STATE.lock();
    state.initialized = true;
    state.mmio_base = mmio_base;
    state.engines[0] = engine0;
    state.engines[1] = engine1;
    state.status_virt = status_virt;
    state.status_phys = status_phys;
    state.staging_virt = staging_virt;
    state.staging_phys = staging_phys;
    drop(state);

    SDMA_READY.store(true, Ordering::SeqCst);

    crate::log!("[SDMA] ───────────────────────────────────────────────────");
    crate::log!("[SDMA] Engine 0: {}", if e0_ok { "READY" } else { "FAILED" });
    crate::log!("[SDMA] Engine 1: {}", if e1_ok { "READY" } else { "FAILED" });
    crate::log!("[SDMA] Staging: {}KB for CPU→GPU transfers", STAGING_BUFFER_SIZE / 1024);
    crate::log!("[SDMA] Commands: sdma copy|fill|test|bench|info");
    crate::log!("[SDMA] ───────────────────────────────────────────────────");
}

// ═══════════════════════════════════════════════════════════════════════════════
// DMA Transfer API
// ═══════════════════════════════════════════════════════════════════════════════

/// Copy `byte_count` bytes from `src_phys` to `dst_phys` using SDMA engine.
///
/// Both addresses must be GPU-visible physical addresses.
/// Uses engine 0 by default. For large transfers, consider using both engines.
///
/// Returns Ok(fence_seq) on success (verified via fence), Err on failure.
pub fn copy(src_phys: u64, dst_phys: u64, byte_count: u32, engine_idx: usize) -> Result<u32, &'static str> {
    if !SDMA_READY.load(Ordering::Relaxed) {
        return Err("SDMA not initialized");
    }
    if byte_count == 0 {
        return Ok(0);
    }
    // SDMA linear copy max is 2^26 = 64MB per packet
    if byte_count > (1 << 26) {
        return Err("Transfer too large (max 64MB per SDMA packet)");
    }
    let eidx = engine_idx.min(1);

    let mut state = SDMA_STATE.lock();
    let status_phys = state.status_phys;
    let status_virt = state.status_virt;
    let engine = state.engines[eidx].as_mut().ok_or("SDMA engine not ready")?;

    // Bump fence sequence
    let fence_val = engine.fence_seq;
    engine.fence_seq = engine.fence_seq.wrapping_add(1);
    if engine.fence_seq == 0 { engine.fence_seq = 1; }

    // Clear fence in status buffer
    let fence_offset = if eidx == 0 { FENCE_OFFSET_E0 } else { FENCE_OFFSET_E1 };
    let fence_virt = status_virt + fence_offset as u64;
    let fence_phys_addr = status_phys + fence_offset as u64;
    unsafe {
        core::ptr::write_volatile(fence_virt as *mut u32, 0);
    }

    // Build SDMA command stream: COPY + FENCE
    let copy_pkt = sdma_copy_linear(src_phys, dst_phys, byte_count);
    let fence_pkt = sdma_fence(fence_phys_addr, fence_val);

    ring_write(engine, &copy_pkt);
    ring_write(engine, &fence_pkt);

    // Submit
    ring_submit(engine);

    crate::serial_println!("[SDMA{}] COPY: {:#X} → {:#X} ({} bytes) fence={}",
        eidx, src_phys, dst_phys, byte_count, fence_val);

    // Poll for fence completion
    let mut elapsed = 0u64;
    loop {
        let current = unsafe { core::ptr::read_volatile(fence_virt as *const u32) };
        if current == fence_val {
            break;
        }
        elapsed += 1;
        if elapsed >= SDMA_TIMEOUT_ITERS {
            crate::serial_println!("[SDMA{}] TIMEOUT: fence expected {} got {}",
                eidx, fence_val, current);
            return Err("SDMA copy timed out");
        }
        if elapsed % 100 == 0 {
            core::hint::spin_loop();
        }
    }

    // Update stats
    engine.transfers += 1;
    engine.bytes += byte_count as u64;
    drop(state);

    TOTAL_BYTES_TRANSFERRED.fetch_add(byte_count as u64, Ordering::Relaxed);
    TOTAL_TRANSFERS.fetch_add(1, Ordering::Relaxed);

    crate::serial_println!("[SDMA{}] Copy complete in {} iters", eidx, elapsed);
    Ok(fence_val)
}

/// Fill `byte_count` bytes at `dst_phys` with `fill_value` using SDMA engine.
///
/// byte_count must be a multiple of 4.
///
/// Returns Ok(fence_seq) on success.
pub fn fill(dst_phys: u64, fill_value: u32, byte_count: u32, engine_idx: usize) -> Result<u32, &'static str> {
    if !SDMA_READY.load(Ordering::Relaxed) {
        return Err("SDMA not initialized");
    }
    if byte_count == 0 {
        return Ok(0);
    }
    if byte_count & 3 != 0 {
        return Err("byte_count must be a multiple of 4");
    }
    if byte_count > (1 << 26) {
        return Err("Fill too large (max 64MB per SDMA packet)");
    }
    let eidx = engine_idx.min(1);

    let mut state = SDMA_STATE.lock();
    let status_phys = state.status_phys;
    let status_virt = state.status_virt;
    let engine = state.engines[eidx].as_mut().ok_or("SDMA engine not ready")?;

    let fence_val = engine.fence_seq;
    engine.fence_seq = engine.fence_seq.wrapping_add(1);
    if engine.fence_seq == 0 { engine.fence_seq = 1; }

    let fence_offset = if eidx == 0 { FENCE_OFFSET_E0 } else { FENCE_OFFSET_E1 };
    let fence_virt_addr = status_virt + fence_offset as u64;
    let fence_phys_addr = status_phys + fence_offset as u64;
    unsafe {
        core::ptr::write_volatile(fence_virt_addr as *mut u32, 0);
    }

    let fill_pkt = sdma_const_fill(dst_phys, fill_value, byte_count);
    let fence_pkt = sdma_fence(fence_phys_addr, fence_val);

    ring_write(engine, &fill_pkt);
    ring_write(engine, &fence_pkt);
    ring_submit(engine);

    crate::serial_println!("[SDMA{}] FILL: {:#X} = {:#010X} x{} bytes, fence={}",
        eidx, dst_phys, fill_value, byte_count, fence_val);

    let mut elapsed = 0u64;
    loop {
        let current = unsafe { core::ptr::read_volatile(fence_virt_addr as *const u32) };
        if current == fence_val {
            break;
        }
        elapsed += 1;
        if elapsed >= SDMA_TIMEOUT_ITERS {
            crate::serial_println!("[SDMA{}] TIMEOUT: fence expected {} got {}",
                eidx, fence_val, current);
            return Err("SDMA fill timed out");
        }
        if elapsed % 100 == 0 {
            core::hint::spin_loop();
        }
    }

    engine.transfers += 1;
    engine.bytes += byte_count as u64;
    drop(state);

    TOTAL_BYTES_TRANSFERRED.fetch_add(byte_count as u64, Ordering::Relaxed);
    TOTAL_TRANSFERS.fetch_add(1, Ordering::Relaxed);

    crate::serial_println!("[SDMA{}] Fill complete in {} iters", eidx, elapsed);
    Ok(fence_val)
}

/// Upload CPU data to a GPU-visible physical address via staging buffer.
///
/// This is the primary mechanism for loading AI model weights:
///   CPU fills staging buffer → SDMA copies staging → VRAM
///
/// `data` = byte slice to upload (max STAGING_BUFFER_SIZE per call)
/// `dst_phys` = GPU-visible destination physical address
///
/// Returns Ok(bytes_transferred) on success.
pub fn upload(data: &[u8], dst_phys: u64, engine_idx: usize) -> Result<usize, &'static str> {
    if !SDMA_READY.load(Ordering::Relaxed) {
        return Err("SDMA not initialized");
    }
    if data.is_empty() {
        return Ok(0);
    }

    // Transfer in STAGING_BUFFER_SIZE chunks
    let mut offset = 0usize;
    while offset < data.len() {
        let chunk = (data.len() - offset).min(STAGING_BUFFER_SIZE);
        // Round up to 4-byte boundary for SDMA
        let aligned_chunk = (chunk + 3) & !3;

        let state = SDMA_STATE.lock();
        let staging_virt = state.staging_virt;
        let staging_phys = state.staging_phys;
        drop(state);

        // Copy chunk to staging buffer (CPU write)
        unsafe {
            let dst = staging_virt as *mut u8;
            let src = data.as_ptr().add(offset);
            core::ptr::copy_nonoverlapping(src, dst, chunk);
            // Zero padding bytes
            if aligned_chunk > chunk {
                core::ptr::write_bytes(dst.add(chunk), 0, aligned_chunk - chunk);
            }
        }

        // DMA from staging to destination
        copy(
            staging_phys,
            dst_phys + offset as u64,
            aligned_chunk as u32,
            engine_idx,
        )?;

        offset += chunk;
    }

    Ok(data.len())
}

/// Download data from a GPU-visible physical address to CPU buffer via staging.
///
/// `src_phys` = GPU-visible source physical address
/// `buf` = CPU buffer to receive data
///
/// Returns Ok(bytes_transferred) on success.
pub fn download(src_phys: u64, buf: &mut [u8], engine_idx: usize) -> Result<usize, &'static str> {
    if !SDMA_READY.load(Ordering::Relaxed) {
        return Err("SDMA not initialized");
    }
    if buf.is_empty() {
        return Ok(0);
    }

    let mut offset = 0usize;
    while offset < buf.len() {
        let chunk = (buf.len() - offset).min(STAGING_BUFFER_SIZE);
        let aligned_chunk = (chunk + 3) & !3;

        let state = SDMA_STATE.lock();
        let staging_virt = state.staging_virt;
        let staging_phys = state.staging_phys;
        drop(state);

        // DMA from source to staging buffer
        copy(
            src_phys + offset as u64,
            staging_phys,
            aligned_chunk as u32,
            engine_idx,
        )?;

        // Copy from staging to CPU buffer
        unsafe {
            let src = staging_virt as *const u8;
            let dst = buf.as_mut_ptr().add(offset);
            core::ptr::copy_nonoverlapping(src, dst, chunk);
        }

        offset += chunk;
    }

    Ok(buf.len())
}

// ═══════════════════════════════════════════════════════════════════════════════
// Self-Test & Benchmark
// ═══════════════════════════════════════════════════════════════════════════════

/// Run a comprehensive self-test of both SDMA engines.
///
/// Tests:
/// 1. Fill a buffer with a pattern via SDMA CONST_FILL → verify CPU readback
/// 2. Copy that buffer to a second buffer via SDMA LINEAR COPY → verify
/// 3. Upload CPU data via staging → verify readback
///
/// Returns (pass_count, fail_count)
pub fn self_test() -> (u32, u32) {
    if !SDMA_READY.load(Ordering::Relaxed) {
        return (0, 0);
    }

    let mut pass = 0u32;
    let mut fail = 0u32;

    // Allocate two test buffers (4KB each)
    let layout = alloc::alloc::Layout::from_size_align(4096, 4096).expect("test layout");
    let buf_a_virt = unsafe { alloc::alloc::alloc_zeroed(layout) } as u64;
    let buf_b_virt = unsafe { alloc::alloc::alloc_zeroed(layout) } as u64;
    let buf_a_phys = memory::virt_to_phys(buf_a_virt).unwrap_or(0);
    let buf_b_phys = memory::virt_to_phys(buf_b_virt).unwrap_or(0);

    if buf_a_phys == 0 || buf_b_phys == 0 {
        crate::serial_println!("[SDMA-TEST] FAIL: cannot allocate test buffers");
        return (0, 1);
    }

    // Test 1: CONST_FILL on engine 0
    crate::serial_println!("[SDMA-TEST] Test 1: CONST_FILL (engine 0, 1024 bytes, pattern=0xFACEFEED)");
    match fill(buf_a_phys, 0xFACE_FEED, 1024, 0) {
        Ok(_) => {
            // Verify first 256 DWORDs
            let ptr = buf_a_virt as *const u32;
            let mut ok = true;
            for i in 0..256 {
                let val = unsafe { core::ptr::read_volatile(ptr.add(i)) };
                if val != 0xFACE_FEED {
                    crate::serial_println!("[SDMA-TEST]   MISMATCH at [{}]: got {:#010X}", i, val);
                    ok = false;
                    break;
                }
            }
            if ok { pass += 1; } else { fail += 1; }
        }
        Err(e) => {
            crate::serial_println!("[SDMA-TEST]   Error: {}", e);
            fail += 1;
        }
    }

    // Test 2: LINEAR COPY engine 0 (buf_a → buf_b)
    crate::serial_println!("[SDMA-TEST] Test 2: LINEAR COPY (engine 0, 1024 bytes)");
    match copy(buf_a_phys, buf_b_phys, 1024, 0) {
        Ok(_) => {
            let ptr_b = buf_b_virt as *const u32;
            let mut ok = true;
            for i in 0..256 {
                let val = unsafe { core::ptr::read_volatile(ptr_b.add(i)) };
                if val != 0xFACE_FEED {
                    crate::serial_println!("[SDMA-TEST]   MISMATCH at [{}]: got {:#010X}", i, val);
                    ok = false;
                    break;
                }
            }
            if ok { pass += 1; } else { fail += 1; }
        }
        Err(e) => {
            crate::serial_println!("[SDMA-TEST]   Error: {}", e);
            fail += 1;
        }
    }

    // Test 3: CONST_FILL on engine 1
    crate::serial_println!("[SDMA-TEST] Test 3: CONST_FILL (engine 1, 512 bytes, pattern=0xBAAD_C0DE)");
    // Clear buf_b first
    unsafe {
        core::ptr::write_bytes(buf_b_virt as *mut u8, 0, 4096);
    }
    match fill(buf_b_phys, 0xBAAD_C0DE, 512, 1) {
        Ok(_) => {
            let ptr_b = buf_b_virt as *const u32;
            let mut ok = true;
            for i in 0..128 {
                let val = unsafe { core::ptr::read_volatile(ptr_b.add(i)) };
                if val != 0xBAAD_C0DE {
                    crate::serial_println!("[SDMA-TEST]   MISMATCH at [{}]: got {:#010X}", i, val);
                    ok = false;
                    break;
                }
            }
            if ok { pass += 1; } else { fail += 1; }
        }
        Err(e) => {
            crate::serial_println!("[SDMA-TEST]   Error: {}", e);
            fail += 1;
        }
    }

    // Test 4: Upload from CPU via staging
    crate::serial_println!("[SDMA-TEST] Test 4: CPU Upload via staging (256 bytes)");
    // Clear destination
    unsafe { core::ptr::write_bytes(buf_a_virt as *mut u8, 0, 4096); }
    let test_data: [u8; 256] = {
        let mut d = [0u8; 256];
        for i in 0..256 { d[i] = i as u8; }
        d
    };
    match upload(&test_data, buf_a_phys, 0) {
        Ok(_) => {
            let ptr = buf_a_virt as *const u8;
            let mut ok = true;
            for i in 0..256 {
                let val = unsafe { core::ptr::read_volatile(ptr.add(i)) };
                if val != i as u8 {
                    crate::serial_println!("[SDMA-TEST]   MISMATCH at [{}]: got {} expected {}", i, val, i);
                    ok = false;
                    break;
                }
            }
            if ok { pass += 1; } else { fail += 1; }
        }
        Err(e) => {
            crate::serial_println!("[SDMA-TEST]   Error: {}", e);
            fail += 1;
        }
    }

    // Test 5: Download to CPU via staging
    crate::serial_println!("[SDMA-TEST] Test 5: CPU Download via staging (256 bytes)");
    let mut readback = [0u8; 256];
    match download(buf_a_phys, &mut readback, 0) {
        Ok(_) => {
            let mut ok = true;
            for i in 0..256 {
                if readback[i] != i as u8 {
                    crate::serial_println!("[SDMA-TEST]   MISMATCH at [{}]: got {} expected {}", i, readback[i], i);
                    ok = false;
                    break;
                }
            }
            if ok { pass += 1; } else { fail += 1; }
        }
        Err(e) => {
            crate::serial_println!("[SDMA-TEST]   Error: {}", e);
            fail += 1;
        }
    }

    // Cleanup: deallocate test buffers
    unsafe {
        alloc::alloc::dealloc(buf_a_virt as *mut u8, layout);
        alloc::alloc::dealloc(buf_b_virt as *mut u8, layout);
    }

    (pass, fail)
}

/// Run a simple bandwidth benchmark using SDMA fill + copy.
///
/// Measures:
/// - Fill bandwidth: how fast SDMA can fill system memory
/// - Copy bandwidth: how fast SDMA can copy between system memory regions
///
/// Returns (fill_bw_mbps, copy_bw_mbps) or an error.
pub fn benchmark(size_kb: u32) -> Result<(u64, u64), &'static str> {
    if !SDMA_READY.load(Ordering::Relaxed) {
        return Err("SDMA not initialized");
    }
    let size_bytes = (size_kb as usize * 1024).min(STAGING_BUFFER_SIZE);
    let aligned = (size_bytes + 3) & !3;

    // Allocate test buffers
    let layout = alloc::alloc::Layout::from_size_align(aligned, 4096)
        .map_err(|_| "allocation error")?;
    let buf_a = unsafe { alloc::alloc::alloc_zeroed(layout) } as u64;
    let buf_b = unsafe { alloc::alloc::alloc_zeroed(layout) } as u64;
    let phys_a = memory::virt_to_phys(buf_a).ok_or("virt_to_phys failed")?;
    let phys_b = memory::virt_to_phys(buf_b).ok_or("virt_to_phys failed")?;

    // Warm up
    let _ = fill(phys_a, 0, aligned as u32, 0);

    // --- Fill benchmark: N iterations ---
    let iters = 16u32;
    let t_start_fill = crate::time::uptime_ticks();
    for _ in 0..iters {
        fill(phys_a, 0xAAAA_BBBB, aligned as u32, 0)?;
    }
    let t_end_fill = crate::time::uptime_ticks();

    // --- Copy benchmark: N iterations ---
    let t_start_copy = crate::time::uptime_ticks();
    for _ in 0..iters {
        copy(phys_a, phys_b, aligned as u32, 0)?;
    }
    let t_end_copy = crate::time::uptime_ticks();

    // Free buffers
    unsafe {
        alloc::alloc::dealloc(buf_a as *mut u8, layout);
        alloc::alloc::dealloc(buf_b as *mut u8, layout);
    }

    // Calculate bandwidth (approximate — using timer ticks)
    let fill_ticks = t_end_fill.saturating_sub(t_start_fill).max(1);
    let copy_ticks = t_end_copy.saturating_sub(t_start_copy).max(1);
    // ticks are ms (from crate::time), convert to approximate throughput

    // Total bytes = size * iters
    let total_bytes = aligned as u64 * iters as u64;

    // ticks are in ms, so MB/s = total_bytes / (ticks_ms / 1000) / 1_000_000
    //                             = total_bytes * 1000 / ticks_ms / 1_000_000
    //                             = total_bytes / (ticks_ms * 1000)
    // Return KB/s to avoid losing precision:
    // KB/s = total_bytes * 1000 / ticks_ms / 1024
    let fill_bw = if fill_ticks > 0 { (total_bytes * 1000) / (fill_ticks * 1024) } else { 0 };
    let copy_bw = if copy_ticks > 0 { (total_bytes * 1000) / (copy_ticks * 1024) } else { 0 };

    Ok((fill_bw, copy_bw))
}

// ═══════════════════════════════════════════════════════════════════════════════
// Public Query API
// ═══════════════════════════════════════════════════════════════════════════════

/// Whether SDMA is ready
pub fn is_ready() -> bool {
    SDMA_READY.load(Ordering::Relaxed)
}

/// Total bytes transferred across all engines
pub fn total_bytes() -> u64 {
    TOTAL_BYTES_TRANSFERRED.load(Ordering::Relaxed)
}

/// Total number of DMA transfers completed
pub fn total_transfers() -> u64 {
    TOTAL_TRANSFERS.load(Ordering::Relaxed)
}

/// Get a summary string
pub fn summary() -> String {
    if is_ready() {
        let bytes = total_bytes();
        let transfers = total_transfers();
        let kb = bytes / 1024;
        format!("SDMA: 2 engines, {} transfers, {} KB moved", transfers, kb)
    } else {
        String::from("SDMA: not initialized")
    }
}

/// Get engine-level info lines for display
pub fn info_lines() -> Vec<String> {
    let mut lines = Vec::new();

    if is_ready() {
        let state = SDMA_STATE.lock();
        lines.push(String::from("╔══════════════════════════════════════════════════╗"));
        lines.push(String::from("║       SDMA Engine — Bare-metal DMA Transfers     ║"));
        lines.push(String::from("╠══════════════════════════════════════════════════╣"));
        lines.push(format!("║ Status Buffer: {:#X}                      ║", state.status_phys));
        lines.push(format!("║ Staging:       {:#X} ({}KB)              ║",
            state.staging_phys, STAGING_BUFFER_SIZE / 1024));
        lines.push(format!("║ Total Bytes:   {} KB                           ║", total_bytes() / 1024));
        lines.push(format!("║ Total Xfers:   {}                              ║", total_transfers()));
        lines.push(String::from("╠══════════════════════════════════════════════════╣"));

        for i in 0..2 {
            if let Some(ref engine) = state.engines[i] {
                lines.push(format!("║ SDMA{}: ring@{:#X} wptr={} seq={} xfers={} bytes={}",
                    i, engine.ring_phys, engine.wptr, engine.fence_seq,
                    engine.transfers, engine.bytes));
            } else {
                lines.push(format!("║ SDMA{}: not initialized                        ║", i));
            }
        }
        lines.push(String::from("╚══════════════════════════════════════════════════╝"));
    } else {
        lines.push(String::from("SDMA not initialized (requires AMD GPU)"));
    }

    lines
}

/// Get staging buffer physical address (for external use, e.g., AI model loader)
pub fn staging_phys() -> Option<u64> {
    if !is_ready() { return None; }
    let state = SDMA_STATE.lock();
    Some(state.staging_phys)
}

/// Get staging buffer size
pub fn staging_size() -> usize {
    STAGING_BUFFER_SIZE
}
