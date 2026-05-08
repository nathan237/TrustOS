

use alloc::vec::Vec;

pub const AR_: usize = 64;

#[repr(C)]
pub struct PerCpuData {
    pub cpu_id: u32,
    pub apic_id: u32,
    pub byk: u64,
    pub interrupt_depth: u32,
    pub kernel_stack: u64,
    pub tsc_last: u64,
    pub work_completed: u64,
}

impl PerCpuData {
    pub const fn new(cpu_id: u32, apic_id: u32) -> Self {
        Self {
            cpu_id, apic_id, byk: 0,
            interrupt_depth: 0, kernel_stack: 0,
            tsc_last: 0, work_completed: 0,
        }
    }
}

pub struct Ve {
    pub cpu_count: u32,
    pub cuj: u32,
    pub ap_apic_ids: Vec<u32>,
}

impl Ve {
    pub fn bfx() -> Self {
        Self { cpu_count: 1, cuj: 0, ap_apic_ids: Vec::new() }
    }
}

pub type Nz = fn(usize, usize, *mut u8);

pub struct WorkItem {
    pub func: Option<Nz>,
    pub start: usize,
    pub end: usize,
    pub data: *mut u8,
}

impl WorkItem {
    pub const fn empty() -> Self {
        Self { func: None, start: 0, end: 0, data: core::ptr::null_mut() }
    }
}

static mut Wp: PerCpuData = PerCpuData::new(0, 0);

pub fn init() {}
pub fn bll() -> u32 { 0 }
pub fn current() -> &'static PerCpuData { unsafe { &Wp } }
pub fn lam() -> &'static mut PerCpuData { unsafe { &mut Wp } }
pub fn cpu_count() -> u32 { 1 }
pub fn jfb(_count: u32) {}
pub fn ail() -> u32 { 1 }
pub fn gds(_cpu_id: u32) -> bool { true }
pub fn ptm() {}
pub fn jeo(_target_cpu: u32) {}
pub fn gof() {}
pub fn get_stats() -> (u32, u32, u64) { (1, 1, 0) }
pub fn elh() {}
pub fn fsj() {}
pub fn eru() -> bool { false }
pub fn bcz(total_items: usize, func: Nz, data: *mut u8) {
    func(0, total_items, data);
}


pub unsafe extern "C" fn hfk(_smp_info: &limine::smp::Cpu) -> ! {
    loop { crate::arch::acb(); }
}
