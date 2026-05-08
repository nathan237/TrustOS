//! CPU TSC stub for non-x86_64 architectures
//! Provides timing functions using the architecture's timer.

static mut FREQUENCY: u64 = 1_000_000_000;

// Structure publique — visible à l'extérieur de ce module.
pub struct Stopwatch {
    start: u64,
}

// Bloc d'implémentation — définit les méthodes du type ci-dessus.
impl Stopwatch {
        // Fonction publique — appelable depuis d'autres modules.
pub fn start() -> Self {
        Self { start: crate::arch::timestamp() }
    }
        // Fonction publique — appelable depuis d'autres modules.
pub fn elapsed_nanos(&self) -> u64 {
        let elapsed = crate::arch::timestamp().wrapping_sub(self.start);
        cycles_to_nanos(elapsed)
    }
        // Fonction publique — appelable depuis d'autres modules.
pub fn elapsed_micros(&self) -> u64 { self.elapsed_nanos() / 1_000 }
        // Fonction publique — appelable depuis d'autres modules.
pub fn elapsed_millis(&self) -> u64 { self.elapsed_nanos() / 1_000_000 }
        // Fonction publique — appelable depuis d'autres modules.
pub fn elapsed_cycles(&self) -> u64 { crate::arch::timestamp().wrapping_sub(self.start) }
        // Fonction publique — appelable depuis d'autres modules.
pub fn lap_nanos(&mut self) -> u64 {
        let now = crate::arch::timestamp();
        let elapsed = cycles_to_nanos(now.wrapping_sub(self.start));
        self.start = now;
        elapsed
    }
}

// Fonction publique — appelable depuis d'autres modules.
pub fn init(frequency_hz: u64) {
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { FREQUENCY = frequency_hz; }
}

// Fonction publique — appelable depuis d'autres modules.
pub fn read_tsc() -> u64 { crate::arch::timestamp() }
// Fonction publique — appelable depuis d'autres modules.
pub fn read_tsc_serialized() -> u64 { crate::arch::timestamp() }
// Fonction publique — appelable depuis d'autres modules.
pub fn read_tscp() -> (u64, u32) { (crate::arch::timestamp(), 0) }
// Fonction publique — appelable depuis d'autres modules.
pub fn frequency_hz() -> u64 { // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { FREQUENCY } }

// Fonction publique — appelable depuis d'autres modules.
pub fn cycles_to_nanos(cycles: u64) -> u64 {
    let freq = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { FREQUENCY };
    if freq == 0 { return 0; }
    (cycles as u128 * 1_000_000_000 / freq as u128) as u64
}
// Fonction publique — appelable depuis d'autres modules.
pub fn cycles_to_micros(cycles: u64) -> u64 { cycles_to_nanos(cycles) / 1_000 }
// Fonction publique — appelable depuis d'autres modules.
pub fn cycles_to_millis(cycles: u64) -> u64 { cycles_to_nanos(cycles) / 1_000_000 }

// Fonction publique — appelable depuis d'autres modules.
pub fn now_nanos() -> u64 { cycles_to_nanos(crate::arch::timestamp()) }
// Fonction publique — appelable depuis d'autres modules.
pub fn now_micros() -> u64 { now_nanos() / 1_000 }
// Fonction publique — appelable depuis d'autres modules.
pub fn now_millis() -> u64 { now_nanos() / 1_000_000 }

// Fonction publique — appelable depuis d'autres modules.
pub fn delay_nanos(nanos: u64) {
    let start = now_nanos();
    while now_nanos().wrapping_sub(start) < nanos {
        core::hint::spin_loop();
    }
}
// Fonction publique — appelable depuis d'autres modules.
pub fn delay_micros(micros: u64) { delay_nanos(micros * 1_000); }
// Fonction publique — appelable depuis d'autres modules.
pub fn delay_millis(millis: u64) { delay_nanos(millis * 1_000_000); }
// Fonction publique — appelable depuis d'autres modules.
pub fn pit_delay_mouse(millis: u64) { delay_millis(millis); }
// Fonction publique — appelable depuis d'autres modules.
pub fn calibrate_tsc() -> u64 { // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { FREQUENCY } }
