//! GDT stub for non-x86_64 architectures
//!
//! Provides segment selector constants and no-op functions.

pub // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const KERNEL_CODE_SELECTOR: u16 = 0x08;
pub // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const KERNEL_DATA_SELECTOR: u16 = 0x10;
pub // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const USER_DATA_SELECTOR: u16 = 0x1B;
pub // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const USER_CODE_SELECTOR: u16 = 0x23;
pub // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const TSS_SELECTOR: u16 = 0x28;

// Fonction publique — appelable depuis d'autres modules.
pub fn init() {}
// Fonction publique — appelable depuis d'autres modules.
pub fn initialize_ap(_cpu_id: u32) {}
// Fonction publique — appelable depuis d'autres modules.
pub fn set_kernel_stack(_stack_top: u64) {}
// Fonction publique — appelable depuis d'autres modules.
pub fn current_ring() -> u8 { 0 }
// Fonction publique — appelable depuis d'autres modules.
pub fn is_kernel_mode() -> bool { true }
// Fonction publique — appelable depuis d'autres modules.
pub fn is_user_mode() -> bool { false }
