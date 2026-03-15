//! SMP stub for non-x86_64 architectures

use alloc::vec::Vec;

pub // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const MAXIMUM_CPUS: usize = 64;

#[repr(C)]
// Structure publique — visible à l'extérieur de ce module.
pub struct PerCpuData {
    pub cpu_id: u32,
    pub apic_id: u32,
    pub current_task: u64,
    pub interrupt_depth: u32,
    pub kernel_stack: u64,
    pub tsc_last: u64,
    pub work_completed: u64,
}

// Bloc d'implémentation — définit les méthodes du type ci-dessus.
impl PerCpuData {
    pub const fn new(cpu_id: u32, apic_id: u32) -> Self {
        Self {
            cpu_id, apic_id, current_task: 0,
            interrupt_depth: 0, kernel_stack: 0,
            tsc_last: 0, work_completed: 0,
        }
    }
}

// Structure publique — visible à l'extérieur de ce module.
pub struct SmpInformation {
    pub cpu_count: u32,
    pub bsp_apic_id: u32,
    pub ap_apic_ids: Vec<u32>,
}

// Bloc d'implémentation — définit les méthodes du type ci-dessus.
impl SmpInformation {
        // Fonction publique — appelable depuis d'autres modules.
pub fn detect() -> Self {
        Self { cpu_count: 1, bsp_apic_id: 0, ap_apic_ids: Vec::new() }
    }
}

pub // Alias de type — donne un nouveau nom à un type existant pour la clarté.
type WorkFn = fn(usize, usize, *mut u8);

// Structure publique — visible à l'extérieur de ce module.
pub struct WorkItem {
    pub func: Option<WorkFn>,
    pub start: usize,
    pub end: usize,
    pub data: *mut u8,
}

// Bloc d'implémentation — définit les méthodes du type ci-dessus.
impl WorkItem {
    pub const fn empty() -> Self {
        Self { func: None, start: 0, end: 0, data: core::ptr::null_mut() }
    }
}

static mut BSP: PerCpuData = PerCpuData::new(0, 0);

// Fonction publique — appelable depuis d'autres modules.
pub fn init() {}
// Fonction publique — appelable depuis d'autres modules.
pub fn current_cpu_id() -> u32 { 0 }
// Fonction publique — appelable depuis d'autres modules.
pub fn current() -> &'static PerCpuData { // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { &BSP } }
// Fonction publique — appelable depuis d'autres modules.
pub fn current_mut() -> &'static mut PerCpuData { // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { &mut BSP } }
// Fonction publique — appelable depuis d'autres modules.
pub fn cpu_count() -> u32 { 1 }
// Fonction publique — appelable depuis d'autres modules.
pub fn set_cpu_count(_count: u32) {}
// Fonction publique — appelable depuis d'autres modules.
pub fn ready_cpu_count() -> u32 { 1 }
// Fonction publique — appelable depuis d'autres modules.
pub fn is_cpu_ready(_cpu_id: u32) -> bool { true }
// Fonction publique — appelable depuis d'autres modules.
pub fn wake_all_aps() {}
// Fonction publique — appelable depuis d'autres modules.
pub fn send_reschedule_ipi(_target_cpu: u32) {}
// Fonction publique — appelable depuis d'autres modules.
pub fn print_status() {}
// Fonction publique — appelable depuis d'autres modules.
pub fn get_stats() -> (u32, u32, u64) { (1, 1, 0) }
// Fonction publique — appelable depuis d'autres modules.
pub fn enable_smp() {}
// Fonction publique — appelable depuis d'autres modules.
pub fn disable_smp() {}
// Fonction publique — appelable depuis d'autres modules.
pub fn is_smp_enabled() -> bool { false }
// Fonction publique — appelable depuis d'autres modules.
pub fn parallel_for(total_items: usize, func: WorkFn, data: *mut u8) {
    func(0, total_items, data);
}

/// AP entry point stub (not used on non-x86)
pub // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe extern "C" fn ap_entry(_smp_information: &limine::smp::Cpu) -> ! {
        // Boucle infinie — tourne jusqu'à un `break` explicite.
loop { crate::arch::halt(); }
}
