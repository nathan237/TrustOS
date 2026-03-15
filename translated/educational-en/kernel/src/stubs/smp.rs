//! SMP stub for non-x86_64 architectures

use alloc::vec::Vec;

pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const MAXIMUM_CPUS: usize = 64;

#[repr(C)]
// Public structure — visible outside this module.
pub struct PerCpuData {
    pub cpu_id: u32,
    pub apic_id: u32,
    pub current_task: u64,
    pub interrupt_depth: u32,
    pub kernel_stack: u64,
    pub tsc_last: u64,
    pub work_completed: u64,
}

// Implementation block — defines methods for the type above.
impl PerCpuData {
    pub const fn new(cpu_id: u32, apic_id: u32) -> Self {
        Self {
            cpu_id, apic_id, current_task: 0,
            interrupt_depth: 0, kernel_stack: 0,
            tsc_last: 0, work_completed: 0,
        }
    }
}

// Public structure — visible outside this module.
pub struct SmpInformation {
    pub cpu_count: u32,
    pub bsp_apic_id: u32,
    pub ap_apic_ids: Vec<u32>,
}

// Implementation block — defines methods for the type above.
impl SmpInformation {
        // Public function — callable from other modules.
pub fn detect() -> Self {
        Self { cpu_count: 1, bsp_apic_id: 0, ap_apic_ids: Vec::new() }
    }
}

pub // Type alias — gives an existing type a new name for clarity.
type WorkFn = fn(usize, usize, *mut u8);

// Public structure — visible outside this module.
pub struct WorkItem {
    pub func: Option<WorkFn>,
    pub start: usize,
    pub end: usize,
    pub data: *mut u8,
}

// Implementation block — defines methods for the type above.
impl WorkItem {
    pub const fn empty() -> Self {
        Self { func: None, start: 0, end: 0, data: core::ptr::null_mut() }
    }
}

static mut BSP: PerCpuData = PerCpuData::new(0, 0);

// Public function — callable from other modules.
pub fn init() {}
// Public function — callable from other modules.
pub fn current_cpu_id() -> u32 { 0 }
// Public function — callable from other modules.
pub fn current() -> &'static PerCpuData { // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { &BSP } }
// Public function — callable from other modules.
pub fn current_mut() -> &'static mut PerCpuData { // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { &mut BSP } }
// Public function — callable from other modules.
pub fn cpu_count() -> u32 { 1 }
// Public function — callable from other modules.
pub fn set_cpu_count(_count: u32) {}
// Public function — callable from other modules.
pub fn ready_cpu_count() -> u32 { 1 }
// Public function — callable from other modules.
pub fn is_cpu_ready(_cpu_id: u32) -> bool { true }
// Public function — callable from other modules.
pub fn wake_all_aps() {}
// Public function — callable from other modules.
pub fn send_reschedule_ipi(_target_cpu: u32) {}
// Public function — callable from other modules.
pub fn print_status() {}
// Public function — callable from other modules.
pub fn get_stats() -> (u32, u32, u64) { (1, 1, 0) }
// Public function — callable from other modules.
pub fn enable_smp() {}
// Public function — callable from other modules.
pub fn disable_smp() {}
// Public function — callable from other modules.
pub fn is_smp_enabled() -> bool { false }
// Public function — callable from other modules.
pub fn parallel_for(total_items: usize, func: WorkFn, data: *mut u8) {
    func(0, total_items, data);
}

/// AP entry point stub (not used on non-x86)
pub // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe extern "C" fn ap_entry(_smp_information: &limine::smp::Cpu) -> ! {
        // Infinite loop — runs until an explicit `break`.
loop { crate::arch::halt(); }
}
