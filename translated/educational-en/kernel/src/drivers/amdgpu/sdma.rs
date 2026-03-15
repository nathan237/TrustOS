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

use super::{mmio_read32, mmio_write32, GpuInformation};
use super::regs;
use crate::memory;

// ═══════════════════════════════════════════════════════════════════════════════
// Constants
// ═══════════════════════════════════════════════════════════════════════════════

/// Ring buffer size: 16KB = 4096 DWORDs
/// Larger than compute ring because DMA packets are bigger (7-8 DWORDs each)
const RING_SIZE_DWORDS: usize = 4096;
// Compile-time constant — evaluated at compilation, zero runtime cost.
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
// Compile-time constant — evaluated at compilation, zero runtime cost.
const FENCE_OFFSET_E1: usize = 0x10;
/// RPTR writeback offset (per engine)
const RPTR_WB_OFFSET_E0: usize = 0x100;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const RPTR_WB_OFFSET_E1: usize = 0x110;

/// Timeout for SDMA operations (polling iterations)
const SDMA_TIMEOUT_ITERS: u64 = 10_000_000;

/// Performance counter: track total bytes transferred
static TOTAL_BYTES_TRANSFERRED: AtomicU64 = AtomicU64::new(0);
// Atomic variable — provides lock-free thread-safe access.
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
fn sdma_header(op: u32, sub_operation: u32) -> u32 {
    ((sub_operation & 0x3) << 26) | ((op & 0x3FFFF) << 8)
}

/// Build a NOP packet (1 DWORD)
#[inline]
fn sdma_nop() -> u32 {
    sdma_header(regs::SDMA_OPERATION_NOP, 0)
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
fn sdma_copy_linear(source_address: u64, destination_address: u64, byte_count: u32) -> [u32; 7] {
    [
        sdma_header(regs::SDMA_OPERATION_COPY, regs::SDMA_COPY_SUB_LINEAR),
        byte_count,
        0, // parameter (src/dst array pitch for 2D — 0 for linear)
        (source_address & 0xFFFFFFFF) as u32,
        ((source_address >> 32) & 0xFFFFFFFF) as u32,
        (destination_address & 0xFFFFFFFF) as u32,
        ((destination_address >> 32) & 0xFFFFFFFF) as u32,
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
fn sdma_const_fill(destination_address: u64, fill_value: u32, byte_count: u32) -> [u32; 5] {
    [
        sdma_header(regs::SDMA_OPERATION_CONST_FILL, 0),
        (destination_address & 0xFFFFFFFF) as u32,
        ((destination_address >> 32) & 0xFFFFFFFF) as u32,
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
fn sdma_fence(address: u64, value: u32) -> [u32; 4] {
    [
        sdma_header(regs::SDMA_OPERATION_FENCE, 0),
        (address & 0xFFFFFFFF) as u32,
        ((address >> 32) & 0xFFFFFFFF) as u32,
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
fn sdma_timestamp(address: u64) -> [u32; 3] {
    [
        sdma_header(regs::SDMA_OPERATION_TIMESTAMP, 0),
        (address & 0xFFFFFFFF) as u32,
        ((address >> 32) & 0xFFFFFFFF) as u32,
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
fn sdma_write_header(destination_address: u64, count: u32) -> [u32; 4] {
    [
        sdma_header(regs::SDMA_OPERATION_WRITE, regs::SDMA_WRITE_SUB_LINEAR),
        (destination_address & 0xFFFFFFFF) as u32,
        ((destination_address >> 32) & 0xFFFFFFFF) as u32,
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
    register_base: u32,
    /// Ring buffer virtual address
    ring_virt: u64,
    /// Ring buffer physical address (GPU-visible)
    ring_physical: u64,
    /// Current write pointer (in DWORDs, wrapping at RING_SIZE_DWORDS)
    wptr: u32,
    /// Fence counter (monotonically increasing)
    fence_sequence: u32,
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
    status_physical: u64,
    /// Staging buffer virtual/physical (for CPU→GPU staging)
    staging_virt: u64,
    staging_physical: u64,
}

// Global shared state guarded by a Mutex (mutual exclusion lock).
static SDMA_STATE: Mutex<SdmaState> = Mutex::new(SdmaState {
    initialized: false,
    mmio_base: 0,
    engines: [None, None],
    status_virt: 0,
    status_physical: 0,
    staging_virt: 0,
    staging_physical: 0,
});

// Atomic variable — provides lock-free thread-safe access.
static SDMA_READY: AtomicBool = AtomicBool::new(false);

// ═══════════════════════════════════════════════════════════════════════════════
// Ring Buffer Operations
// ═══════════════════════════════════════════════════════════════════════════════

/// Write DWORDs to an engine's ring buffer
fn ring_write(engine: &mut SdmaEngine, data: &[u32]) {
    let ring = engine.ring_virt as *mut u32;
    for (i, &dw) in data.iter().enumerate() {
        let index = (engine.wptr as usize + i) % RING_SIZE_DWORDS;
                // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
            core::ptr::write_volatile(ring.add(index), dw);
        }
    }
    engine.wptr = ((engine.wptr as usize + data.len()) % RING_SIZE_DWORDS) as u32;
}

/// Submit the ring buffer by updating WPTR register
fn ring_submit(engine: &SdmaEngine) {
        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
        // SDMA WPTR is in bytes, not dwords
        let wptr_bytes = engine.wptr * 4;
        let wptr_register = engine.register_base + regs::SDMA_GFX_RB_WPTR;
        mmio_write32(engine.mmio_base, wptr_register, wptr_bytes);
        mmio_write32(engine.mmio_base, wptr_register + 4, 0); // WPTR_HI
    }
}

/// Read the hardware RPTR (in DWORDs)
fn ring_rptr(engine: &SdmaEngine) -> u32 {
        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
        let rptr_register = engine.register_base + regs::SDMA_GFX_RB_RPTR;
        let rptr_bytes = mmio_read32(engine.mmio_base, rptr_register);
        rptr_bytes / 4
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Engine Initialization
// ═══════════════════════════════════════════════════════════════════════════════

/// Initialize a single SDMA engine
fn initialize_engine(
    mmio_base: u64,
    engine_index: usize,
    ring_virt: u64,
    ring_physical: u64,
    status_physical: u64,
    rptr_wb_offset: usize,
) -> Option<SdmaEngine> {
    let base = if engine_index == 0 {
        regs::SDMA0_BASE
    } else {
        regs::SDMA1_BASE
    };

    crate::log!("[SDMA{}] Initializing engine (reg_base={:#X})", engine_index, base);

        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
        // Step 1: Read engine status
        let status_register = if engine_index == 0 {
            regs::SDMA0_STATUS_REGISTER
        } else {
            regs::SDMA1_STATUS_REGISTER
        };
        let status = mmio_read32(mmio_base, status_register);
        crate::log!("[SDMA{}] STATUS={:#010X} (idle={})",
            engine_index, status, (status & regs::SDMA_STATUS_IDLE) != 0);

        // Step 2: Halt engine by setting F32_CNTL halt bit
        let f32_register = if engine_index == 0 {
            regs::SDMA0_F32_CNTL
        } else {
            regs::SDMA1_F32_CNTL
        };
        mmio_write32(mmio_base, f32_register, 1); // HALT=1

        // Small delay for halt to take effect
        for _ in 0..1000 {
            core::hint::spin_loop();
        }

        // Step 3: Disable GFX ring
        let rb_cntl_register = base + regs::SDMA_GFX_RB_CNTL;
        mmio_write32(mmio_base, rb_cntl_register, 0); // Disable ring

        // Step 4: Set ring buffer base address (256-byte aligned, store in 256B units)
        let rb_base_256 = ring_physical >> 8;
        let rb_base_register = base + regs::SDMA_GFX_RB_BASE;
        let rb_base_hi_register = base + regs::SDMA_GFX_RB_BASE_HI;
        mmio_write32(mmio_base, rb_base_register, (rb_base_256 & 0xFFFFFFFF) as u32);
        mmio_write32(mmio_base, rb_base_hi_register, ((rb_base_256 >> 32) & 0xFFFFFFFF) as u32);

        // Step 5: Clear RPTR/WPTR
        mmio_write32(mmio_base, base + regs::SDMA_GFX_RB_RPTR, 0);
        mmio_write32(mmio_base, base + regs::SDMA_GFX_RB_RPTR_HI, 0);
        mmio_write32(mmio_base, base + regs::SDMA_GFX_RB_WPTR, 0);
        mmio_write32(mmio_base, base + regs::SDMA_GFX_RB_WPTR_HI, 0);

        // Step 6: Set RPTR writeback address (GPU writes RPTR here so CPU can track)
        let rptr_wb_address = status_physical + rptr_wb_offset as u64;
        mmio_write32(mmio_base, base + regs::SDMA_GFX_RB_RPTR_ADDRESS_LO,
            (rptr_wb_address & 0xFFFFFFFF) as u32);
        mmio_write32(mmio_base, base + regs::SDMA_GFX_RB_RPTR_ADDRESS_HI,
            ((rptr_wb_address >> 32) & 0xFFFFFFFF) as u32);

        // Step 7: Configure ring control and enable
        //  - RB_SIZE = log2(4096 dwords) = 12, shifted into bits [6:1]
        //  - RPTR_WRITEBACK_ENABLE = 1 (bit 12)
        //  - RB_ENABLE = 1 (bit 0)
        //  - VMID = 0 (bits [19:16], bare-metal, no IOMMU translation)
        let rb_cntl = regs::SDMA_RB_CNTL_RB_ENABLE
            | (RING_SIZE_LOG2 << regs::SDMA_RB_CNTL_RB_SIZE_SHIFT)
            | regs::SDMA_RB_CNTL_RPTR_WRITEBACK_ENABLE;
        mmio_write32(mmio_base, rb_cntl_register, rb_cntl);

        // Step 8: Un-halt engine
        mmio_write32(mmio_base, f32_register, 0); // HALT=0

        // Step 9: Verify engine came back alive
        for _ in 0..10000 {
            core::hint::spin_loop();
        }
        let status_after = mmio_read32(mmio_base, status_register);
        crate::log!("[SDMA{}] Post-init STATUS={:#010X}", engine_index, status_after);
    }

    Some(SdmaEngine {
        index: engine_index,
        mmio_base,
        register_base: base,
        ring_virt,
        ring_physical,
        wptr: 0,
        fence_sequence: 1,
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
    let sdma_ver = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { mmio_read32(mmio_base, regs::SDMA0_VERSION) };
    crate::log!("[SDMA] SDMA0 VERSION={:#010X}", sdma_ver);

    // Step 2: Allocate ring buffers (one per engine, 16KB each, page-aligned)
    let ring_layout = alloc::alloc::Layout::from_size_align(RING_SIZE_BYTES, 4096)
        .expect("sdma ring layout");

    let ring0_virt = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { alloc::alloc::alloc_zeroed(ring_layout) } as u64;
    let ring0_physical = memory::virt_to_physical(ring0_virt).unwrap_or(0);

    let ring1_virt = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { alloc::alloc::alloc_zeroed(ring_layout) } as u64;
    let ring1_physical = memory::virt_to_physical(ring1_virt).unwrap_or(0);

    if ring0_physical == 0 || ring1_physical == 0 {
        crate::log!("[SDMA] ERROR: Cannot get physical address for ring buffers");
        return;
    }

    crate::log!("[SDMA] Ring0: virt={:#X} phys={:#X} ({} dwords)",
        ring0_virt, ring0_physical, RING_SIZE_DWORDS);
    crate::log!("[SDMA] Ring1: virt={:#X} phys={:#X} ({} dwords)",
        ring1_virt, ring1_physical, RING_SIZE_DWORDS);

    // Step 3: Allocate status buffer (fence + RPTR writeback, 4KB)
    let status_layout = alloc::alloc::Layout::from_size_align(STATUS_BUFFER_SIZE, 4096)
        .expect("sdma status layout");
    let status_virt = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { alloc::alloc::alloc_zeroed(status_layout) } as u64;
    let status_physical = memory::virt_to_physical(status_virt).unwrap_or(0);

    if status_physical == 0 {
        crate::log!("[SDMA] ERROR: Cannot get phys for status buffer");
        return;
    }
    crate::log!("[SDMA] Status: virt={:#X} phys={:#X}", status_virt, status_physical);

    // Step 4: Allocate staging buffer (CPU→GPU data staging, 256KB)
    let staging_layout = alloc::alloc::Layout::from_size_align(STAGING_BUFFER_SIZE, 4096)
        .expect("sdma staging layout");
    let staging_virt = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { alloc::alloc::alloc_zeroed(staging_layout) } as u64;
    let staging_physical = memory::virt_to_physical(staging_virt).unwrap_or(0);

    if staging_physical == 0 {
        crate::log!("[SDMA] ERROR: Cannot get phys for staging buffer");
        return;
    }
    crate::log!("[SDMA] Staging: virt={:#X} phys={:#X} ({}KB)",
        staging_virt, staging_physical, STAGING_BUFFER_SIZE / 1024);

    // Step 5: Initialize both engines
    let engine0 = initialize_engine(
        mmio_base, 0, ring0_virt, ring0_physical, status_physical, RPTR_WB_OFFSET_E0,
    );
    let engine1 = initialize_engine(
        mmio_base, 1, ring1_virt, ring1_physical, status_physical, RPTR_WB_OFFSET_E1,
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
    state.status_physical = status_physical;
    state.staging_virt = staging_virt;
    state.staging_physical = staging_physical;
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
pub fn copy(source_physical: u64, destination_physical: u64, byte_count: u32, engine_index: usize) -> Result<u32, &'static str> {
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
    let eidx = engine_index.minimum(1);

    let mut state = SDMA_STATE.lock();
    let status_physical = state.status_physical;
    let status_virt = state.status_virt;
    let engine = state.engines[eidx].as_mut().ok_or("SDMA engine not ready")?;

    // Bump fence sequence
    let fence_value = engine.fence_sequence;
    engine.fence_sequence = engine.fence_sequence.wrapping_add(1);
    if engine.fence_sequence == 0 { engine.fence_sequence = 1; }

    // Clear fence in status buffer
    let fence_offset = if eidx == 0 { FENCE_OFFSET_E0 } else { FENCE_OFFSET_E1 };
    let fence_virt = status_virt + fence_offset as u64;
    let fence_physical_address = status_physical + fence_offset as u64;
        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
        core::ptr::write_volatile(fence_virt as *mut u32, 0);
    }

    // Build SDMA command stream: COPY + FENCE
    let copy_packet = sdma_copy_linear(source_physical, destination_physical, byte_count);
    let fence_packet = sdma_fence(fence_physical_address, fence_value);

    ring_write(engine, &copy_packet);
    ring_write(engine, &fence_packet);

    // Submit
    ring_submit(engine);

    crate::serial_println!("[SDMA{}] COPY: {:#X} → {:#X} ({} bytes) fence={}",
        eidx, source_physical, destination_physical, byte_count, fence_value);

    // Poll for fence completion
    let mut elapsed = 0u64;
        // Infinite loop — runs until an explicit `break`.
loop {
        let current = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { core::ptr::read_volatile(fence_virt as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const u32) };
        if current == fence_value {
            break;
        }
        elapsed += 1;
        if elapsed >= SDMA_TIMEOUT_ITERS {
            crate::serial_println!("[SDMA{}] TIMEOUT: fence expected {} got {}",
                eidx, fence_value, current);
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
    Ok(fence_value)
}

/// Fill `byte_count` bytes at `dst_phys` with `fill_value` using SDMA engine.
///
/// byte_count must be a multiple of 4.
///
/// Returns Ok(fence_seq) on success.
pub fn fill(destination_physical: u64, fill_value: u32, byte_count: u32, engine_index: usize) -> Result<u32, &'static str> {
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
    let eidx = engine_index.minimum(1);

    let mut state = SDMA_STATE.lock();
    let status_physical = state.status_physical;
    let status_virt = state.status_virt;
    let engine = state.engines[eidx].as_mut().ok_or("SDMA engine not ready")?;

    let fence_value = engine.fence_sequence;
    engine.fence_sequence = engine.fence_sequence.wrapping_add(1);
    if engine.fence_sequence == 0 { engine.fence_sequence = 1; }

    let fence_offset = if eidx == 0 { FENCE_OFFSET_E0 } else { FENCE_OFFSET_E1 };
    let fence_virt_address = status_virt + fence_offset as u64;
    let fence_physical_address = status_physical + fence_offset as u64;
        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
        core::ptr::write_volatile(fence_virt_address as *mut u32, 0);
    }

    let fill_packet = sdma_const_fill(destination_physical, fill_value, byte_count);
    let fence_packet = sdma_fence(fence_physical_address, fence_value);

    ring_write(engine, &fill_packet);
    ring_write(engine, &fence_packet);
    ring_submit(engine);

    crate::serial_println!("[SDMA{}] FILL: {:#X} = {:#010X} x{} bytes, fence={}",
        eidx, destination_physical, fill_value, byte_count, fence_value);

    let mut elapsed = 0u64;
        // Infinite loop — runs until an explicit `break`.
loop {
        let current = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { core::ptr::read_volatile(fence_virt_address as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const u32) };
        if current == fence_value {
            break;
        }
        elapsed += 1;
        if elapsed >= SDMA_TIMEOUT_ITERS {
            crate::serial_println!("[SDMA{}] TIMEOUT: fence expected {} got {}",
                eidx, fence_value, current);
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
    Ok(fence_value)
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
pub fn upload(data: &[u8], destination_physical: u64, engine_index: usize) -> Result<usize, &'static str> {
    if !SDMA_READY.load(Ordering::Relaxed) {
        return Err("SDMA not initialized");
    }
    if data.is_empty() {
        return Ok(0);
    }

    // Transfer in STAGING_BUFFER_SIZE chunks
    let mut offset = 0usize;
    while offset < data.len() {
        let chunk = (data.len() - offset).minimum(STAGING_BUFFER_SIZE);
        // Round up to 4-byte boundary for SDMA
        let aligned_chunk = (chunk + 3) & !3;

        let state = SDMA_STATE.lock();
        let staging_virt = state.staging_virt;
        let staging_physical = state.staging_physical;
        drop(state);

        // Copy chunk to staging buffer (CPU write)
        unsafe {
            let destination = staging_virt as *mut u8;
            let source = data.as_pointer().add(offset);
            core::ptr::copy_nonoverlapping(source, destination, chunk);
            // Zero padding bytes
            if aligned_chunk > chunk {
                core::ptr::write_bytes(destination.add(chunk), 0, aligned_chunk - chunk);
            }
        }

        // DMA from staging to destination
        copy(
            staging_physical,
            destination_physical + offset as u64,
            aligned_chunk as u32,
            engine_index,
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
pub fn download(source_physical: u64, buffer: &mut [u8], engine_index: usize) -> Result<usize, &'static str> {
    if !SDMA_READY.load(Ordering::Relaxed) {
        return Err("SDMA not initialized");
    }
    if buffer.is_empty() {
        return Ok(0);
    }

    let mut offset = 0usize;
    while offset < buffer.len() {
        let chunk = (buffer.len() - offset).minimum(STAGING_BUFFER_SIZE);
        let aligned_chunk = (chunk + 3) & !3;

        let state = SDMA_STATE.lock();
        let staging_virt = state.staging_virt;
        let staging_physical = state.staging_physical;
        drop(state);

        // DMA from source to staging buffer
        copy(
            source_physical + offset as u64,
            staging_physical,
            aligned_chunk as u32,
            engine_index,
        )?;

        // Copy from staging to CPU buffer
        unsafe {
            let source = staging_virt as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const u8;
            let destination = buffer.as_mut_pointer().add(offset);
            core::ptr::copy_nonoverlapping(source, destination, chunk);
        }

        offset += chunk;
    }

    Ok(buffer.len())
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
    let buffer_a_virt = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { alloc::alloc::alloc_zeroed(layout) } as u64;
    let buffer_b_virt = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { alloc::alloc::alloc_zeroed(layout) } as u64;
    let buffer_a_physical = memory::virt_to_physical(buffer_a_virt).unwrap_or(0);
    let buffer_b_physical = memory::virt_to_physical(buffer_b_virt).unwrap_or(0);

    if buffer_a_physical == 0 || buffer_b_physical == 0 {
        crate::serial_println!("[SDMA-TEST] FAIL: cannot allocate test buffers");
        return (0, 1);
    }

    // Test 1: CONST_FILL on engine 0
    crate::serial_println!("[SDMA-TEST] Test 1: CONST_FILL (engine 0, 1024 bytes, pattern=0xFACEFEED)");
        // Pattern matching — Rust's exhaustive branching construct.
match fill(buffer_a_physical, 0xFACE_FEED, 1024, 0) {
        Ok(_) => {
            // Verify first 256 DWORDs
            let ptr = buffer_a_virt as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const u32;
            let mut ok = true;
            for i in 0..256 {
                let value = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { core::ptr::read_volatile(ptr.add(i)) };
                if value != 0xFACE_FEED {
                    crate::serial_println!("[SDMA-TEST]   MISMATCH at [{}]: got {:#010X}", i, value);
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
        // Pattern matching — Rust's exhaustive branching construct.
match copy(buffer_a_physical, buffer_b_physical, 1024, 0) {
        Ok(_) => {
            let pointer_b = buffer_b_virt as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const u32;
            let mut ok = true;
            for i in 0..256 {
                let value = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { core::ptr::read_volatile(pointer_b.add(i)) };
                if value != 0xFACE_FEED {
                    crate::serial_println!("[SDMA-TEST]   MISMATCH at [{}]: got {:#010X}", i, value);
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
        core::ptr::write_bytes(buffer_b_virt as *mut u8, 0, 4096);
    }
        // Pattern matching — Rust's exhaustive branching construct.
match fill(buffer_b_physical, 0xBAAD_C0DE, 512, 1) {
        Ok(_) => {
            let pointer_b = buffer_b_virt as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const u32;
            let mut ok = true;
            for i in 0..128 {
                let value = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { core::ptr::read_volatile(pointer_b.add(i)) };
                if value != 0xBAAD_C0DE {
                    crate::serial_println!("[SDMA-TEST]   MISMATCH at [{}]: got {:#010X}", i, value);
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
    unsafe { core::ptr::write_bytes(buffer_a_virt as *mut u8, 0, 4096); }
    let test_data: [u8; 256] = {
        let mut d = [0u8; 256];
        for i in 0..256 { d[i] = i as u8; }
        d
    };
        // Pattern matching — Rust's exhaustive branching construct.
match upload(&test_data, buffer_a_physical, 0) {
        Ok(_) => {
            let ptr = buffer_a_virt as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const u8;
            let mut ok = true;
            for i in 0..256 {
                let value = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { core::ptr::read_volatile(ptr.add(i)) };
                if value != i as u8 {
                    crate::serial_println!("[SDMA-TEST]   MISMATCH at [{}]: got {} expected {}", i, value, i);
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
        // Pattern matching — Rust's exhaustive branching construct.
match download(buffer_a_physical, &mut readback, 0) {
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
        alloc::alloc::dealloc(buffer_a_virt as *mut u8, layout);
        alloc::alloc::dealloc(buffer_b_virt as *mut u8, layout);
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
pub fn benchmark(size_keyboard: u32) -> Result<(u64, u64), &'static str> {
    if !SDMA_READY.load(Ordering::Relaxed) {
        return Err("SDMA not initialized");
    }
    let size_bytes = (size_keyboard as usize * 1024).minimum(STAGING_BUFFER_SIZE);
    let aligned = (size_bytes + 3) & !3;

    // Allocate test buffers
    let layout = alloc::alloc::Layout::from_size_align(aligned, 4096)
        .map_error(|_| "allocation error")?;
    let buffer_a = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { alloc::alloc::alloc_zeroed(layout) } as u64;
    let buffer_b = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { alloc::alloc::alloc_zeroed(layout) } as u64;
    let physical_a = memory::virt_to_physical(buffer_a).ok_or("virt_to_phys failed")?;
    let physical_b = memory::virt_to_physical(buffer_b).ok_or("virt_to_phys failed")?;

    // Warm up
    let _ = fill(physical_a, 0, aligned as u32, 0);

    // --- Fill benchmark: N iterations ---
    let iters = 16u32;
    let t_start_fill = crate::time::uptime_ticks();
    for _ in 0..iters {
        fill(physical_a, 0xAAAA_BBBB, aligned as u32, 0)?;
    }
    let t_end_fill = crate::time::uptime_ticks();

    // --- Copy benchmark: N iterations ---
    let t_start_copy = crate::time::uptime_ticks();
    for _ in 0..iters {
        copy(physical_a, physical_b, aligned as u32, 0)?;
    }
    let t_end_copy = crate::time::uptime_ticks();

    // Free buffers
    unsafe {
        alloc::alloc::dealloc(buffer_a as *mut u8, layout);
        alloc::alloc::dealloc(buffer_b as *mut u8, layout);
    }

    // Calculate bandwidth (approximate — using timer ticks)
    let fill_ticks = t_end_fill.saturating_sub(t_start_fill).maximum(1);
    let copy_ticks = t_end_copy.saturating_sub(t_start_copy).maximum(1);
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
        let keyboard = bytes / 1024;
        format!("SDMA: 2 engines, {} transfers, {} KB moved", transfers, keyboard)
    } else {
        String::from("SDMA: not initialized")
    }
}

/// Get engine-level info lines for display
pub fn information_lines() -> Vec<String> {
    let mut lines = Vec::new();

    if is_ready() {
        let state = SDMA_STATE.lock();
        lines.push(String::from("╔══════════════════════════════════════════════════╗"));
        lines.push(String::from("║       SDMA Engine — Bare-metal DMA Transfers     ║"));
        lines.push(String::from("╠══════════════════════════════════════════════════╣"));
        lines.push(format!("║ Status Buffer: {:#X}                      ║", state.status_physical));
        lines.push(format!("║ Staging:       {:#X} ({}KB)              ║",
            state.staging_physical, STAGING_BUFFER_SIZE / 1024));
        lines.push(format!("║ Total Bytes:   {} KB                           ║", total_bytes() / 1024));
        lines.push(format!("║ Total Xfers:   {}                              ║", total_transfers()));
        lines.push(String::from("╠══════════════════════════════════════════════════╣"));

        for i in 0..2 {
            if let Some(ref engine) = state.engines[i] {
                lines.push(format!("║ SDMA{}: ring@{:#X} wptr={} seq={} xfers={} bytes={}",
                    i, engine.ring_physical, engine.wptr, engine.fence_sequence,
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
pub fn staging_physical() -> Option<u64> {
    if !is_ready() { return None; }
    let state = SDMA_STATE.lock();
    Some(state.staging_physical)
}

/// Get staging buffer size
pub fn staging_size() -> usize {
    STAGING_BUFFER_SIZE
}
