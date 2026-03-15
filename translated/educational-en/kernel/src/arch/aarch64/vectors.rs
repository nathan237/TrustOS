//! aarch64 Exception Vector Table + Trap Frame
//!
//! ARM64 exception vectors are installed at VBAR_EL1.
//! The table is 2048-byte aligned with 16 entries (4 groups × 4 types),
//! each entry 128 bytes (32 instructions).
//!
//! Groups:
//!   0) Current EL with SP_EL0 (not used — we always use SP_ELx at EL1)
//!   1) Current EL with SP_ELx (kernel exceptions/interrupts)
//!   2) Lower EL using AArch64 (user → kernel: syscalls, user IRQs)
//!   3) Lower EL using AArch32 (unused)

use core::sync::atomic::{AtomicBool, Ordering};

/// Whether exception vectors have been installed
static VECTORS_INSTALLED: AtomicBool = AtomicBool::new(false);

/// Trap frame saved on the stack during an exception.
/// Layout must exactly match the assembly save/restore macros.
#[repr(C)]
// Public structure — visible outside this module.
pub struct TrapFrame {
    /// General-purpose registers x0-x30 (31 × 8 = 248 bytes)
    pub regs: [u64; 31],
    /// Saved SP_EL0 (user stack pointer)
    pub sp_el0: u64,
    /// Exception Link Register (return address)
    pub elr_el1: u64,
    /// Saved Program Status Register
    pub spsr_el1: u64,
}

// TrapFrame size: 31 + 3 = 34 u64 = 272 bytes

/// Install the exception vector table.
/// Must be called before enabling interrupts.
pub fn init() {
        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
        // Get address of the vector table (defined in global_asm below)
        let vectors: u64;
        core::arch::asm!(
            "adrp {0}, __exception_vectors",
            "add {0}, {0}, :lo12:__exception_vectors",
            out(reg) vectors,
            options(nomem, nostack, preserves_flags)
        );

        // Install VBAR_EL1
        super::cpu::write_vbar_el1(vectors);

        VECTORS_INSTALLED.store(true, Ordering::Release);
        crate::serial_println!("[VECTORS] Exception vector table installed at {:#x}", vectors);
    }
}

// Public function — callable from other modules.
pub fn is_installed() -> bool {
    VECTORS_INSTALLED.load(Ordering::Acquire)
}

// ============================================================================
// Exception handlers (called from assembly with &TrapFrame in x0)
// ============================================================================

/// Handle synchronous exception from EL1 (kernel mode)
///
/// ESR_EL1 EC field meanings:
///   0x21 = Instruction Abort (same EL)
///   0x25 = Data Abort (same EL)
///   0x15 = SVC from AArch64 (shouldn't happen in kernel mode)
///   0x00 = Unknown
///   0x0E = Illegal Execution
///   0x22 = PC alignment fault
///   0x26 = SP alignment fault
#[no_mangle]
extern "C" fn el1_sync_handler(tf: &TrapFrame) {
    let esr = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { super::cpu::read_esr_el1() };
    let far = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { super::cpu::read_far_el1() };
    let ec = (esr >> 26) & 0x3F;
    let iss = esr & 0x1FF_FFFF;

        // Pattern matching — Rust's exhaustive branching construct.
match ec {
        0x25 => {
            // Data Abort from same EL
            let wnr = (iss >> 6) & 1; // Write-not-Read
            let dfsc = iss & 0x3F;    // Data Fault Status Code
            crate::serial_println!(
                "\n[EXCEPTION] Data Abort (EL1): FAR={:#x} WnR={} DFSC={:#x} ELR={:#x}",
                far, wnr, dfsc, tf.elr_el1
            );

            // Check if it's a translation fault (page not mapped) — potential page fault handling
            if dfsc & 0x3C == 0x04 {
                // Translation fault at level dfsc & 0x3
                crate::serial_println!("  Translation fault level {}", dfsc & 0x3);
            }

            // For now, panic on unhandled data aborts
            panic!("Unhandled Data Abort at ELR={:#x} FAR={:#x}", tf.elr_el1, far);
        }
        0x21 => {
            // Instruction Abort from same EL
            crate::serial_println!(
                "\n[EXCEPTION] Instruction Abort (EL1): FAR={:#x} ELR={:#x}",
                far, tf.elr_el1
            );
            panic!("Instruction Abort at ELR={:#x} FAR={:#x}", tf.elr_el1, far);
        }
        0x22 => {
            crate::serial_println!("\n[EXCEPTION] PC Alignment Fault: ELR={:#x}", tf.elr_el1);
            panic!("PC Alignment Fault at {:#x}", tf.elr_el1);
        }
        0x26 => {
            crate::serial_println!("\n[EXCEPTION] SP Alignment Fault: ELR={:#x}", tf.elr_el1);
            panic!("SP Alignment Fault at {:#x}", tf.elr_el1);
        }
        0x3C => {
            // BRK instruction (software breakpoint)
            crate::serial_println!("[EXCEPTION] Breakpoint (BRK) at ELR={:#x}", tf.elr_el1);
        }
        _ => {
            crate::serial_println!(
                "\n[EXCEPTION] Unhandled sync exception: EC={:#x} ISS={:#x} ELR={:#x} FAR={:#x}",
                ec, iss, tf.elr_el1, far
            );
            panic!("Unhandled sync exception EC={:#x}", ec);
        }
    }
}

/// Handle IRQ from EL1 (kernel mode) — main interrupt dispatch
///
/// Reads GIC IAR to determine which IRQ fired, dispatches to the
/// appropriate handler, then sends EOI.
#[no_mangle]
extern "C" fn el1_interrupt_request_handler(_tf: &TrapFrame) {
    let irq = super::gic::acknowledge();

    if irq == super::gic::SPURIOUS_INTERRUPT_REQUEST {
        // Spurious interrupt, ignore
        return;
    }

        // Pattern matching — Rust's exhaustive branching construct.
match irq {
        // Timer PPI (CNTP = 30) — preemptive scheduling
        super::gic::TIMER_PPI => {
            // Re-arm timer for next tick (10ms)
            super::gic::rearm_timer(10);

            // Only run tick logic after bootstrap is ready
            if crate::interrupts::is_bootstrap_ready() {
                // Update tick counters
                crate::logger::tick();
                crate::time::tick();

                // Record trace event
                crate::trace::record_event(crate::trace::EventType::TimerTick, 0);

                // Notify scheduler for preemptive context switching
                crate::thread::on_timer_tick();
            }
        }
        // Other IRQs can be added here (VirtIO, etc.)
        _ => {
            crate::serial_println!("[IRQ] Unhandled IRQ {}", irq);
        }
    }

    // Signal End-of-Interrupt
    super::gic::eoi(irq);
}

/// Handle synchronous exception from EL0 (user mode)
/// Primary use: SVC instruction (syscalls)
#[no_mangle]
extern "C" fn el0_sync_handler(tf: &TrapFrame) {
    let esr = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { super::cpu::read_esr_el1() };
    let ec = (esr >> 26) & 0x3F;

        // Pattern matching — Rust's exhaustive branching construct.
match ec {
        0x15 => {
            // SVC from AArch64 — system call
            // Syscall number in x8, args in x0-x5, return in x0
            let _syscall_number = tf.regs[8];
            let _arg0 = tf.regs[0];
            // TODO: dispatch to syscall handler
            crate::serial_println!("[SYSCALL] SVC from user: num={}", _syscall_number);
        }
        0x20 => {
            // Instruction Abort from lower EL
            let far = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { super::cpu::read_far_el1() };
            crate::serial_println!(
                "[EXCEPTION] User Instruction Abort: FAR={:#x} ELR={:#x}",
                far, tf.elr_el1
            );
        }
        0x24 => {
            // Data Abort from lower EL
            let far = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { super::cpu::read_far_el1() };
            crate::serial_println!(
                "[EXCEPTION] User Data Abort: FAR={:#x} ELR={:#x}",
                far, tf.elr_el1
            );
        }
        _ => {
            crate::serial_println!(
                "[EXCEPTION] Unhandled user sync exception: EC={:#x} ELR={:#x}",
                ec, tf.elr_el1
            );
        }
    }
}

/// Handle IRQ from EL0 (user mode)
/// Timer or device interrupt while in user space
#[no_mangle]
extern "C" fn el0_interrupt_request_handler(_tf: &TrapFrame) {
    // Same dispatch as EL1 IRQ — GIC doesn't distinguish source EL
    let irq = super::gic::acknowledge();

    if irq == super::gic::SPURIOUS_INTERRUPT_REQUEST {
        return;
    }

        // Pattern matching — Rust's exhaustive branching construct.
match irq {
        super::gic::TIMER_PPI => {
            super::gic::rearm_timer(10);
            if crate::interrupts::is_bootstrap_ready() {
                crate::logger::tick();
                crate::time::tick();
                crate::trace::record_event(crate::trace::EventType::TimerTick, 0);
                crate::thread::on_timer_tick();
            }
        }
        _ => {
            crate::serial_println!("[IRQ] Unhandled user IRQ {}", irq);
        }
    }

    super::gic::eoi(irq);
}

// ============================================================================
// Exception Vector Table (global_asm)
//
// Layout: 4 groups × 4 exception types = 16 entries, each 128 bytes (0x80)
//
// For each exception entry that we handle:
//   1. Save all GPRs + system regs to stack (TrapFrame)
//   2. Call Rust handler with x0 = &TrapFrame
//   3. Restore all GPRs + system regs from stack
//   4. ERET back to interrupted code
// ============================================================================

core::arch::global_asm!(
    // ========================================================================
    // Save/restore macros
    // ========================================================================

    // Save all registers to create a TrapFrame on the stack
    ".macro SAVE_REGS",
    "    sub sp, sp, #272",          // 34 × 8 = TrapFrame size
    "    stp x0,  x1,  [sp, #(0  * 16)]",
    "    stp x2,  x3,  [sp, #(1  * 16)]",
    "    stp x4,  x5,  [sp, #(2  * 16)]",
    "    stp x6,  x7,  [sp, #(3  * 16)]",
    "    stp x8,  x9,  [sp, #(4  * 16)]",
    "    stp x10, x11, [sp, #(5  * 16)]",
    "    stp x12, x13, [sp, #(6  * 16)]",
    "    stp x14, x15, [sp, #(7  * 16)]",
    "    stp x16, x17, [sp, #(8  * 16)]",
    "    stp x18, x19, [sp, #(9  * 16)]",
    "    stp x20, x21, [sp, #(10 * 16)]",
    "    stp x22, x23, [sp, #(11 * 16)]",
    "    stp x24, x25, [sp, #(12 * 16)]",
    "    stp x26, x27, [sp, #(13 * 16)]",
    "    stp x28, x29, [sp, #(14 * 16)]",
    // x30 (LR) at regs[30] offset 240
    "    str x30,       [sp, #240]",
    // Save system registers
    "    mrs x21, sp_el0",
    "    mrs x22, elr_el1",
    "    mrs x23, spsr_el1",
    "    str x21, [sp, #248]",      // sp_el0
    "    stp x22, x23, [sp, #256]", // elr_el1, spsr_el1
    ".endm",

    // Restore all registers from TrapFrame on the stack
    ".macro RESTORE_REGS",
    "    ldp x22, x23, [sp, #256]", // elr_el1, spsr_el1
    "    ldr x21, [sp, #248]",      // sp_el0
    "    msr sp_el0, x21",
    "    msr elr_el1, x22",
    "    msr spsr_el1, x23",
    "    ldr x30,       [sp, #240]",
    "    ldp x28, x29, [sp, #(14 * 16)]",
    "    ldp x26, x27, [sp, #(13 * 16)]",
    "    ldp x24, x25, [sp, #(12 * 16)]",
    "    ldp x22, x23, [sp, #(11 * 16)]",
    "    ldp x20, x21, [sp, #(10 * 16)]",
    "    ldp x18, x19, [sp, #(9  * 16)]",
    "    ldp x16, x17, [sp, #(8  * 16)]",
    "    ldp x14, x15, [sp, #(7  * 16)]",
    "    ldp x12, x13, [sp, #(6  * 16)]",
    "    ldp x10, x11, [sp, #(5  * 16)]",
    "    ldp x8,  x9,  [sp, #(4  * 16)]",
    "    ldp x6,  x7,  [sp, #(3  * 16)]",
    "    ldp x4,  x5,  [sp, #(2  * 16)]",
    "    ldp x2,  x3,  [sp, #(1  * 16)]",
    "    ldp x0,  x1,  [sp, #(0  * 16)]",
    "    add sp, sp, #272",
    "    eret",
    ".endm",

    // ========================================================================
    // Vector Table (must be 2048-byte aligned)
    //
    // Each entry is 128 bytes = 32 instructions max.
    // SAVE_REGS + handler + RESTORE_REGS = ~47 instructions total,
    // so we use a branch-out approach: each entry just branches to a
    // handler defined AFTER the vector table.
    // ========================================================================
    ".section .text",
    ".balign 2048",
    ".global __exception_vectors",
    "__exception_vectors:",

    // --- Group 0: Current EL with SP_EL0 (unused, we use SP_ELx) ---
    // 0x000: Synchronous
    "    b .",
    ".balign 128",
    // 0x080: IRQ
    "    b .",
    ".balign 128",
    // 0x100: FIQ
    "    b .",
    ".balign 128",
    // 0x180: SError
    "    b .",
    ".balign 128",

    // --- Group 1: Current EL with SP_ELx (kernel mode) ---
    // 0x200: Synchronous
    "    b __el1_sync_entry",
    ".balign 128",
    // 0x280: IRQ
    "    b __el1_irq_entry",
    ".balign 128",
    // 0x300: FIQ
    "    b .",
    ".balign 128",
    // 0x380: SError
    "    b .",
    ".balign 128",

    // --- Group 2: Lower EL using AArch64 (EL0 → EL1) ---
    // 0x400: Synchronous (syscalls)
    "    b __el0_sync_entry",
    ".balign 128",
    // 0x480: IRQ
    "    b __el0_irq_entry",
    ".balign 128",
    // 0x500: FIQ
    "    b .",
    ".balign 128",
    // 0x580: SError
    "    b .",
    ".balign 128",

    // --- Group 3: Lower EL using AArch32 (unused) ---
    // 0x600: Synchronous
    "    b .",
    ".balign 128",
    // 0x680: IRQ
    "    b .",
    ".balign 128",
    // 0x700: FIQ
    "    b .",
    ".balign 128",
    // 0x780: SError
    "    b .",
    ".balign 128",

    // ========================================================================
    // Exception entry points (outside the vector table, no size limit)
    // ========================================================================

    "__el1_sync_entry:",
    "    SAVE_REGS",
    "    mov x0, sp",
    "    bl el1_sync_handler",
    "    RESTORE_REGS",

    "__el1_irq_entry:",
    "    SAVE_REGS",
    "    mov x0, sp",
    "    bl el1_irq_handler",
    "    RESTORE_REGS",

    "__el0_sync_entry:",
    "    SAVE_REGS",
    "    mov x0, sp",
    "    bl el0_sync_handler",
    "    RESTORE_REGS",

    "__el0_irq_entry:",
    "    SAVE_REGS",
    "    mov x0, sp",
    "    bl el0_irq_handler",
    "    RESTORE_REGS",
);
