//! SMP stub for non-x86_64 architectures

use alloc::vec::Vec;

pub const MAX_CPUS: usize = 64;

#[repr(C)]
pub struct PerCpuData {
    pub cpu_id: u32,
    pub apic_id: u32,
    pub current_task: u64,
    pub interrupt_depth: u32,
    pub kernel_stack: u64,
    pub tsc_last: u64,
    pub work_completed: u64,
}

impl PerCpuData {
    pub const fn new(cpu_id: u32, apic_id: u32) -> Self {
        Self {
            cpu_id, apic_id, current_task: 0,
            interrupt_depth: 0, kernel_stack: 0,
            tsc_last: 0, work_completed: 0,
        }
    }
}

pub struct SmpInfo {
    pub cpu_count: u32,
    pub bsp_apic_id: u32,
    pub ap_apic_ids: Vec<u32>,
}

impl SmpInfo {
    pub fn detect() -> Self {
        Self { cpu_count: 1, bsp_apic_id: 0, ap_apic_ids: Vec::new() }
    }
}

pub type WorkFn = fn(usize, usize, *mut u8);

pub struct WorkItem {
    pub func: Option<WorkFn>,
    pub start: usize,
    pub end: usize,
    pub data: *mut u8,
}

impl WorkItem {
    pub const fn empty() -> Self {
        Self { func: None, start: 0, end: 0, data: core::ptr::null_mut() }
    }
}

static mut BSP: PerCpuData = PerCpuData::new(0, 0);

pub fn init() {}
pub fn current_cpu_id() -> u32 { 0 }
pub fn current() -> &'static PerCpuData { unsafe { &BSP } }
pub fn current_mut() -> &'static mut PerCpuData { unsafe { &mut BSP } }
pub fn cpu_count() -> u32 { 1 }
pub fn set_cpu_count(_count: u32) {}
pub fn ready_cpu_count() -> u32 { 1 }
pub fn is_cpu_ready(_cpu_id: u32) -> bool { true }
pub fn wake_all_aps() {}
pub fn send_reschedule_ipi(_target_cpu: u32) {}
pub fn print_status() {}
pub fn get_stats() -> (u32, u32, u64) { (1, 1, 0) }
pub fn enable_smp() {}
pub fn disable_smp() {}
pub fn is_smp_enabled() -> bool { false }
pub fn parallel_for(total_items: usize, func: WorkFn, data: *mut u8) {
    func(0, total_items, data);
}

/// AP entry point stub (not used on non-x86)
pub unsafe extern "C" fn ap_entry(_smp_info: &limine::smp::Cpu) -> ! {
    loop { crate::arch::halt(); }
}
