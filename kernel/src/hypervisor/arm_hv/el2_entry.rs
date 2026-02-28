//! EL2 Entry/Exit — Guest World Switch
//!
//! This module contains the Rust-side wrappers for entering and exiting
//! the guest (EL1). The actual register save/restore would ideally be
//! in assembly, but we use inline asm for portability.
//!
//! ## World Switch Flow
//!
//! ```text
//! TrustOS (EL2)                     Guest/Android (EL1)
//!     │                                    │
//!     ├── configure HCR_EL2 ───────┐       │
//!     ├── set VTTBR_EL2 ──────────┤       │
//!     ├── set VBAR_EL2 ──────────┤       │
//!     ├── load guest context ──────┤       │
//!     ├── eret ────────────────────┼──────→│ (guest runs)
//!     │                            │       │
//!     │  (guest hits trapped MMIO) │       │
//!     │                            │       ├── data abort
//!     │←──────────────────────────┼───────┤ (trap to EL2)
//!     ├── save guest context       │       │
//!     ├── handle trap              │       │
//!     ├── restore + eret ──────────┼──────→│ (guest resumes)
//! ```

use super::{GuestContext, HypervisorConfig, hcr, compute_hcr};
use super::stage2::Stage2Tables;
use super::trap_handler;
use super::vgic::VirtualGic;
use super::mmio_spy;

use core::sync::atomic::{AtomicBool, Ordering};

/// Whether the hypervisor is currently running a guest
static GUEST_RUNNING: AtomicBool = AtomicBool::new(false);

/// EL2 exception vector table setup
///
/// The vector table has 16 entries (4 groups × 4 types):
/// - Current EL with SP0: Sync, IRQ, FIQ, SError
/// - Current EL with SPx: Sync, IRQ, FIQ, SError  
/// - Lower EL using AArch64: Sync, IRQ, FIQ, SError  ← our guest traps land here
/// - Lower EL using AArch32: Sync, IRQ, FIQ, SError
pub fn install_vector_table() {
    #[cfg(target_arch = "aarch64")]
    unsafe {
        // The vector table is defined in assembly (el2_vectors.S)
        // For now, use a Rust-based minimal approach:
        // We store a function pointer that the assembly stub will call
        core::arch::asm!(
            "adr {tmp}, 2f",
            "msr vbar_el2, {tmp}",
            "isb",
            "b 3f",

            // Minimal vector table (2KB aligned in real impl)
            // For now this is a placeholder — the real vector table
            // needs to be in a separate .S file with proper alignment
            ".balign 2048",
            "2:",
            // Current EL, SP0 (shouldn't happen)
            "b .",           // Sync
            ".balign 128",
            "b .",           // IRQ  
            ".balign 128",
            "b .",           // FIQ
            ".balign 128",
            "b .",           // SError
            ".balign 128",

            // Current EL, SPx
            "b .",           // Sync
            ".balign 128",
            "b .",           // IRQ
            ".balign 128",
            "b .",           // FIQ
            ".balign 128",
            "b .",           // SError
            ".balign 128",

            // Lower EL, AArch64 — THIS IS WHERE GUEST TRAPS GO
            "b 4f",          // Sync (MMIO, SMC, etc.)
            ".balign 128",
            "b 5f",          // IRQ (hardware interrupt)
            ".balign 128",
            "b .",           // FIQ (not used)
            ".balign 128",
            "b .",           // SError
            ".balign 128",

            // Lower EL, AArch32 (not used)
            "b .",
            ".balign 128",
            "b .",
            ".balign 128",
            "b .",
            ".balign 128",
            "b .",

            // Sync handler from Lower EL
            "4:",
            "stp x29, x30, [sp, #-16]!",
            "bl {sync_handler}",
            "ldp x29, x30, [sp], #16",
            "eret",

            // IRQ handler from Lower EL
            "5:",
            "stp x29, x30, [sp, #-16]!",
            "bl {irq_handler}",
            "ldp x29, x30, [sp], #16",
            "eret",

            "3:",

            tmp = out(reg) _,
            sync_handler = sym el2_sync_entry,
            irq_handler = sym el2_irq_entry,
            options(nostack)
        );
    }
}

/// Rust entry point for EL2 synchronous exceptions
#[cfg(target_arch = "aarch64")]
#[no_mangle]
extern "C" fn el2_sync_entry() {
    // Read syndrome registers
    let esr: u64;
    let far: u64;
    let hpfar: u64;

    unsafe {
        core::arch::asm!(
            "mrs {esr}, esr_el2",
            "mrs {far}, far_el2",
            "mrs {hpfar}, hpfar_el2",
            esr = out(reg) esr,
            far = out(reg) far,
            hpfar = out(reg) hpfar,
            options(nomem, nostack)
        );
    }

    // For a full implementation, we'd save/restore all 31 guest registers
    // from the stack frame. For now, use a static context.
    unsafe {
        let regs = &mut GUEST_CTX.x;
        let action = trap_handler::handle_sync_trap(esr, far, hpfar, regs);

        match action {
            trap_handler::TrapAction::Handled => {
                // Advance guest PC past the faulting instruction
                let elr: u64;
                core::arch::asm!("mrs {e}, elr_el2", e = out(reg) elr, options(nomem, nostack));
                let il = if (esr >> 25) & 1 != 0 { 4u64 } else { 2u64 };
                core::arch::asm!("msr elr_el2, {e}", e = in(reg) elr + il, options(nomem, nostack));
            }
            trap_handler::TrapAction::ForwardSmc => {
                // Forward SMC to real firmware (EL3)
                // Re-execute the SMC instruction
                // The guest registers are already set up
            }
            trap_handler::TrapAction::InjectFault => {
                // Inject a data abort back into the guest
                // Set ESR_EL1, FAR_EL1, redirect to guest's exception vector
            }
            trap_handler::TrapAction::GuestHalt => {
                GUEST_RUNNING.store(false, Ordering::Release);
            }
        }
    }
}

/// Rust entry point for EL2 IRQ exceptions
#[cfg(target_arch = "aarch64")]
#[no_mangle]
extern "C" fn el2_irq_entry() {
    unsafe {
        super::vgic::handle_el2_irq(&mut VGIC_INSTANCE);
    }
}

/// Static guest context (single-core for now)
static mut GUEST_CTX: GuestContext = GuestContext {
    x: [0; 31],
    sp_el1: 0,
    elr_el1: 0,
    spsr_el1: 0,
    sctlr_el1: 0,
    ttbr0_el1: 0,
    ttbr1_el1: 0,
    tcr_el1: 0,
    mair_el1: 0,
    vbar_el1: 0,
    esr_el2: 0,
    far_el2: 0,
    hpfar_el2: 0,
};

/// Static VGIC instance
static mut VGIC_INSTANCE: VirtualGic = VirtualGic::new();

/// Configure and enter the hypervisor
///
/// This is the main entry point called from android_main when
/// hypervisor mode is requested. It:
/// 1. Builds Stage-2 page tables
/// 2. Configures HCR_EL2 to trap everything we want
/// 3. Sets up the EL2 vector table
/// 4. Enters the guest (Android kernel) at EL1
pub fn enter_hypervisor(config: &HypervisorConfig) -> ! {
    // Verify we're at EL2
    if !super::is_el2() {
        panic!("ARM hypervisor requires EL2! Current EL is lower.");
    }

    // 1. Build Stage-2 page tables
    let mut s2 = Stage2Tables::new(1); // VMID = 1

    // Identity-map guest RAM
    s2.map_ram(config.guest_ram_base, config.guest_ram_size);

    // Trap requested MMIO regions
    for &(base, size) in &config.trapped_mmio {
        let label = mmio_spy::identify_device(base);
        s2.trap_mmio(base, size, label);
    }

    // 2. Compute HCR_EL2
    let hcr_val = compute_hcr(config);

    // 3. Configure EL2 system registers
    #[cfg(target_arch = "aarch64")]
    unsafe {
        // Set HCR_EL2
        core::arch::asm!(
            "msr hcr_el2, {hcr}",
            "isb",
            hcr = in(reg) hcr_val,
            options(nomem, nostack)
        );

        // Set VTTBR_EL2 (Stage-2 page table base)
        let vttbr = s2.vttbr();
        core::arch::asm!(
            "msr vttbr_el2, {vttbr}",
            "isb",
            vttbr = in(reg) vttbr,
            options(nomem, nostack)
        );

        // Configure VTCR_EL2 (Stage-2 translation control)
        // T0SZ=24 (40-bit IPA), SL0=01 (start at level 1), IRGN0/ORGN0=WB
        let vtcr: u64 = (24 << 0)     // T0SZ = 24 → 40-bit IPA
                       | (0b01 << 6)   // SL0 = 1 (start at L1)
                       | (0b01 << 8)   // IRGN0 = Write-Back
                       | (0b01 << 10)  // ORGN0 = Write-Back
                       | (0b11 << 12)  // SH0 = Inner Shareable
                       | (0b00 << 14)  // TG0 = 4KB granule
                       | (1 << 31);    // RES1
        core::arch::asm!(
            "msr vtcr_el2, {vtcr}",
            "isb",
            vtcr = in(reg) vtcr,
            options(nomem, nostack)
        );
    }

    // 4. Install EL2 vector table
    install_vector_table();

    // 5. Initialize virtual GIC
    unsafe {
        VGIC_INSTANCE.init();
    }

    // 6. Set up guest initial state
    unsafe {
        GUEST_CTX.x[0] = config.guest_dtb;    // x0 = DTB pointer (Linux convention)
        GUEST_CTX.elr_el1 = config.guest_entry;  // Entry point
        GUEST_CTX.spsr_el1 = 0x3C5;              // EL1h, DAIF masked
    }

    GUEST_RUNNING.store(true, Ordering::Release);

    // 7. Enter guest!
    #[cfg(target_arch = "aarch64")]
    unsafe {
        core::arch::asm!(
            // Set where the guest will start executing
            "msr elr_el2, {entry}",
            // Set guest's saved program status
            "msr spsr_el2, {spsr}",
            // Pass DTB in x0
            "mov x0, {dtb}",
            // Clear other argument registers
            "mov x1, xzr",
            "mov x2, xzr",
            "mov x3, xzr",
            // Invalidate TLBs
            "tlbi vmalls12e1is",
            "dsb ish",
            "isb",
            // Enter guest!
            "eret",
            entry = in(reg) config.guest_entry,
            spsr = in(reg) 0x3C5u64,
            dtb = in(reg) config.guest_dtb,
            options(noreturn)
        );
    }

    #[cfg(not(target_arch = "aarch64"))]
    loop {
        core::hint::spin_loop();
    }
}

/// Check if a guest is currently running
pub fn is_guest_running() -> bool {
    GUEST_RUNNING.load(Ordering::Acquire)
}

/// Get a snapshot of the current spy data for the shell
pub fn get_spy_summary() -> alloc::string::String {
    use alloc::format;
    let mmio_count = mmio_spy::total_mmio_events();
    let smc_count = mmio_spy::total_smc_events();

    let mut s = format!(
        "=== TrustOS EL2 Hypervisor Spy Report ===\n\
         MMIO accesses intercepted: {}\n\
         SMC calls intercepted: {}\n",
        mmio_count, smc_count
    );

    // Per-device stats
    let stats = mmio_spy::device_stats();
    if !stats.is_empty() {
        s.push_str("\n--- Device Activity ---\n");
        for (name, reads, writes) in &stats {
            s.push_str(&format!("  {:<20} R:{:<6} W:{}\n", name, reads, writes));
        }
    }

    // Recent MMIO events
    let recent = mmio_spy::recent_mmio_events(10);
    if !recent.is_empty() {
        s.push_str("\n--- Recent MMIO (newest first) ---\n");
        for ev in &recent {
            s.push_str(&format!("  {}\n", mmio_spy::format_mmio_event(ev)));
        }
    }

    // Recent SMC events
    let smc_recent = mmio_spy::recent_smc_events(5);
    if !smc_recent.is_empty() {
        s.push_str("\n--- Recent SMC Calls ---\n");
        for ev in &smc_recent {
            s.push_str(&format!("  {}\n", mmio_spy::format_smc_event(ev)));
        }
    }

    s
}
