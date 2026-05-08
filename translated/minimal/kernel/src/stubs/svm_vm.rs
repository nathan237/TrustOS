

use alloc::string::String;
use alloc::vec::Vec;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SvmVmState {
    Created,
    Running,
    Paused,
    Stopped,
}

pub fn avv<F, U>(_id: u64, _f: F) -> Option<U>
where F: FnOnce(&mut Afc) -> U {
    None
}

pub fn dtn() -> Vec<(u64, String, SvmVmState)> { Vec::new() }

pub struct SvmVmStats {
    pub vmexits: u64,
    pub cpuid_exits: u64,
    pub io_exits: u64,
    pub msr_exits: u64,
    pub hlt_exits: u64,
    pub npf_exits: u64,
    pub vmmcall_exits: u64,
    pub intr_exits: u64,
}

pub struct GuestRegs {
    pub rax: u64,
    pub rbx: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub rbp: u64,
    pub rsp: u64,
    pub r8: u64,
    pub r9: u64,
    pub r10: u64,
    pub r11: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
}

pub struct LapicState {
    pub enabled: bool,
    pub svr: u32,
    pub tpr: u32,
    pub timer_lvt: u32,
    pub icr: u64,
    pub dcr: u32,
}
pub struct Vmcb;

impl Vmcb {
    pub fn read_state(&self, bkm: usize) -> u64 { 0 }
    pub fn read_control(&self, bkm: usize) -> u64 { 0 }
    pub fn read_u16(&self, bkm: usize) -> u16 { 0 }
    pub fn read_u32(&self, bkm: usize) -> u32 { 0 }
}

pub struct Afc {
    pub stats: SvmVmStats,
    pub guest_regs: GuestRegs,
    pub memory_size: usize,
    pub asid: u32,
    pub lapic: LapicState,
    pub vmcb: Option<Vmcb>,
}

impl Afc {
    pub fn rby(&self) -> String { String::new() }
    pub fn qoz(&self) -> String { String::new() }
    pub fn read_guest_memory(&self, _gpa: u64, _len: usize) -> Option<&[u8]> { None }
    pub fn start_linux(&mut self, _bzimage: &[u8], _cmdline: &str, _initrd: Option<&[u8]>) -> Result<(), String> {
        Err(String::from("SVM not available on this architecture"))
    }
}
