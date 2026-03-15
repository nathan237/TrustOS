//! GICv2 (Generic Interrupt Controller v2) Driver
//!
//! Supports the ARM GICv2 interrupt controller used in QEMU virt machine.
//! Base addresses for QEMU virt:
//!   - GICD (Distributor):  0x0800_0000
//!   - GICC (CPU Interface): 0x0801_0000
//!
//! These addresses are within the 0-1GB MMIO identity map set up by boot.rs.

use core::sync::atomic::{AtomicBool, Ordering};
use super::cpu;

// ============================================================================
// GICv2 Base Addresses (QEMU virt machine)
// ============================================================================

/// GIC Distributor base (GICD) — shared across all CPUs
const GICD_BASE: u64 = 0x0800_0000;
/// GIC CPU Interface base (GICC) — per-CPU banked
const GICC_BASE: u64 = 0x0801_0000;

// ============================================================================
// GICD Register Offsets
// ============================================================================

const GICD_CTLR: u64       = 0x000;  // Distributor Control
const GICD_TYPER: u64      = 0x004;  // Interrupt Controller Type
const GICD_ISENABLER: u64  = 0x100;  // Set-Enable (array, 32 IRQs per register)
const GICD_ICENABLER: u64  = 0x180;  // Clear-Enable
const GICD_ISPENDR: u64    = 0x200;  // Set-Pending
const GICD_ICPENDR: u64    = 0x280;  // Clear-Pending
const GICD_IPRIORITYR: u64 = 0x400;  // Priority (8 bits per IRQ)
const GICD_ITARGETSR: u64  = 0x800;  // Target CPU (8 bits per IRQ, bitmap)
const GICD_ICFGR: u64      = 0xC00;  // Configuration (2 bits per IRQ)

// ============================================================================
// GICC Register Offsets
// ============================================================================

const GICC_CTLR: u64  = 0x000;  // CPU Interface Control
const GICC_PMR: u64   = 0x004;  // Priority Mask
const GICC_BPR: u64   = 0x008;  // Binary Point
const GICC_IAR: u64   = 0x00C;  // Interrupt Acknowledge (read to ACK)
const GICC_EOIR: u64  = 0x010;  // End of Interrupt (write to signal done)

// ============================================================================
// Important IRQ Numbers
// ============================================================================

/// Non-secure Physical Timer PPI (CNTP_EL0) — used for preemptive scheduling
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const TIMER_PPI: u32 = 30;

/// Spurious interrupt ID (no pending interrupt)
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SPURIOUS_INTERRUPT_REQUEST: u32 = 1023;

/// Whether GIC has been initialized
static GIC_INITIALIZED: AtomicBool = AtomicBool::new(false);

// ============================================================================
// MMIO Helpers
// ============================================================================

#[inline(always)]
// SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe fn gicd_read(offset: u64) -> u32 {
    cpu::mmio_read32(GICD_BASE + offset)
}

#[inline(always)]
// SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe fn gicd_write(offset: u64, value: u32) {
    cpu::mmio_write32(GICD_BASE + offset, value);
}

#[inline(always)]
// SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe fn gicc_read(offset: u64) -> u32 {
    cpu::mmio_read32(GICC_BASE + offset)
}

#[inline(always)]
// SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe fn gicc_write(offset: u64, value: u32) {
    cpu::mmio_write32(GICC_BASE + offset, value);
}

// ============================================================================
// Public API
// ============================================================================

/// Initialize the GICv2 (Distributor + CPU Interface).
///
/// MUST be called after MMIO identity mapping is set up (boot.rs)
/// and BEFORE enabling interrupts.
pub fn init() -> bool {
        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
        // --- Distributor Init ---

        // Disable distributor while configuring
        gicd_write(GICD_CTLR, 0);

        // Read how many IRQ lines this GIC supports
        let typer = gicd_read(GICD_TYPER);
        let maximum_irqs = ((typer & 0x1F) + 1) * 32;

        crate::serial_println!("[GIC] Distributor: {} IRQ lines", maximum_irqs);

        // Disable all interrupts
        let register_count = maximum_irqs / 32;
        for i in 0..register_count {
            gicd_write(GICD_ICENABLER + (i as u64) * 4, 0xFFFF_FFFF);
        }

        // Clear all pending
        for i in 0..register_count {
            gicd_write(GICD_ICPENDR + (i as u64) * 4, 0xFFFF_FFFF);
        }

        // Set all SPIs (32+) to lowest priority (0xA0) and target CPU 0
        for i in 8..(maximum_irqs / 4) {
            // Priority: 4 IRQs per register (8 bits each)
            gicd_write(GICD_IPRIORITYR + (i as u64) * 4, 0xA0A0_A0A0);
        }
        for i in 8..(maximum_irqs / 4) {
            // Target: CPU 0 (bit 0) for all SPIs
            gicd_write(GICD_ITARGETSR + (i as u64) * 4, 0x0101_0101);
        }

        // Set all SPIs to level-triggered
        for i in 2..(maximum_irqs / 16) {
            gicd_write(GICD_ICFGR + (i as u64) * 4, 0);
        }

        // --- PPI Configuration (banked per CPU, IRQs 16-31) ---

        // Set PPI priorities to 0x90 (higher than default SPIs)
        for i in 4..8u64 {
            gicd_write(GICD_IPRIORITYR + i * 4, 0x9090_9090);
        }

        // Enable distributor (Group 0 + Group 1)
        gicd_write(GICD_CTLR, 0x3);

        // --- CPU Interface Init ---

        // Set priority mask to allow all priorities (0xFF = accept everything)
        gicc_write(GICC_PMR, 0xFF);

        // Binary point = 0 (no grouping, all bits used for priority comparison)
        gicc_write(GICC_BPR, 0);

        // Enable CPU interface (Group 0 + Group 1, EOI mode = standard)
        gicc_write(GICC_CTLR, 0x3);

        // Ensure all writes are visible
        cpu::dsb_sy();
        cpu::isb();

        GIC_INITIALIZED.store(true, Ordering::Release);
        crate::serial_println!("[GIC] Initialized (GICD + GICC)");
    }

    true
}

/// Initialize GIC CPU interface for an Application Processor
pub fn initialize_ap() {
        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
        // PPIs are banked per CPU, re-enable timer PPI
        enable_interrupt_request(TIMER_PPI);

        // CPU Interface
        gicc_write(GICC_PMR, 0xFF);
        gicc_write(GICC_BPR, 0);
        gicc_write(GICC_CTLR, 0x3);

        cpu::dsb_sy();
        cpu::isb();
    }
}

/// Enable a specific IRQ in the distributor
pub fn enable_interrupt_request(irq: u32) {
    let reg = irq / 32;
    let bit = irq % 32;
        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
        gicd_write(GICD_ISENABLER + (reg as u64) * 4, 1 << bit);
    }
}

/// Disable a specific IRQ in the distributor
pub fn disable_interrupt_request(irq: u32) {
    let reg = irq / 32;
    let bit = irq % 32;
        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
        gicd_write(GICD_ICENABLER + (reg as u64) * 4, 1 << bit);
    }
}

/// Acknowledge an interrupt (read IAR).
/// Returns the interrupt ID. If 1023, it's spurious.
pub fn acknowledge() -> u32 {
        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { gicc_read(GICC_IAR) & 0x3FF }
}

/// Signal End-of-Interrupt for the given IRQ.
pub fn eoi(irq: u32) {
        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
        gicc_write(GICC_EOIR, irq);
    }
}

/// Check if GIC is initialized
pub fn is_initialized() -> bool {
    GIC_INITIALIZED.load(Ordering::Acquire)
}

/// Enable the ARM Generic Timer interrupt (CNTP PPI 30) through the GIC.
///
/// Also arms the timer with the specified interval.
pub fn enable_timer(interval_mouse: u64) {
    // Enable PPI 30 (non-secure physical timer)
    enable_interrupt_request(TIMER_PPI);

    // Arm the timer
    let frequency = super::timer::frequency();
    let ticks = (interval_mouse * frequency) / 1000;
    super::timer::set_oneshot(ticks);

    crate::serial_println!("[GIC] Timer IRQ enabled (PPI {}, {}ms, {} ticks)", 
        TIMER_PPI, interval_mouse, ticks);
}

/// Start preemptive scheduling timer (10ms ticks = 100Hz)
pub fn start_timer(interval_mouse: u64) {
    enable_timer(interval_mouse);
}

/// Stop the timer
pub fn stop_timer() {
    disable_interrupt_request(TIMER_PPI);
    super::timer::disable_timer();
}

/// Re-arm the timer for next tick (called from IRQ handler)
pub fn rearm_timer(interval_mouse: u64) {
    let frequency = super::timer::frequency();
    let ticks = (interval_mouse * frequency) / 1000;
    super::timer::set_oneshot(ticks);
}
