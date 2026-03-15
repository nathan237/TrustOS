//! aarch64 Interrupt Control
//!
//! Uses DAIF (Debug/Abort/IRQ/FIQ) mask bits in PSTATE.

/// Enable interrupts (clear IRQ mask in DAIF)
#[inline(always)]
// Fonction publique — appelable depuis d'autres modules.
pub fn enable() {
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
        core::arch::asm!("msr DAIFClr, #0x2", options(nomem, nostack, preserves_flags));
    }
}

/// Disable interrupts (set IRQ mask in DAIF)
#[inline(always)]
// Fonction publique — appelable depuis d'autres modules.
pub fn disable() {
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
        core::arch::asm!("msr DAIFSet, #0x2", options(nomem, nostack, preserves_flags));
    }
}

/// Check if IRQ interrupts are enabled (DAIF.I bit clear)
#[inline(always)]
// Fonction publique — appelable depuis d'autres modules.
pub fn are_enabled() -> bool {
    let daif: u64;
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
        core::arch::asm!("mrs {}, DAIF", out(reg) daif, options(nomem, nostack, preserves_flags));
    }
    // DAIF bit 7 (I) = IRQ mask; 0 = enabled, 1 = masked
    daif & (1 << 7) == 0
}

/// Run a closure with interrupts disabled, restoring previous state after
#[inline(always)]
// Fonction publique — appelable depuis d'autres modules.
pub fn without_interrupts<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    let were_enabled = are_enabled();
    if were_enabled {
        disable();
    }
    let result = f();
    if were_enabled {
        enable();
    }
    result
}

/// Initialize aarch64 interrupt infrastructure (vectors + GIC)
/// NOTE: Timer is NOT enabled here — it's started later by set_bootstrap_ready()
/// to avoid spurious IRQs during boot init (VFS, threads, etc.)
pub fn initialize_platform() {
    super::vectors::init();
    super::gic::init();
    // Timer deferred to set_bootstrap_ready() — no IRQs during early boot
    crate::log!("aarch64 platform interrupts initialized (GICv2, timer deferred)");
}
