

use alloc::string::String;
use alloc::vec::Vec;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SvmVmState {
    Cu,
    Ai,
    Cl,
    Af,
}

pub fn coa<G, Ac>(ddq: u64, xzc: G) -> Option<Ac>
where G: FnOnce(&mut Btr) -> Ac {
    None
}

pub fn hqc() -> Vec<(u64, String, SvmVmState)> { Vec::new() }

pub struct SvmVmStats {
    pub ait: u64,
    pub bmp: u64,
    pub ank: u64,
    pub bkn: u64,
    pub axz: u64,
    pub cay: u64,
    pub gwh: u64,
    pub jap: u64,
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
    pub iq: bool,
    pub bim: u32,
    pub guv: u32,
    pub atq: u32,
    pub bnh: u64,
    pub dgc: u32,
}
pub struct Vmcb;

impl Vmcb {
    pub fn xs(&self, dnv: usize) -> u64 { 0 }
    pub fn cgx(&self, dnv: usize) -> u64 { 0 }
    pub fn alp(&self, dnv: usize) -> u16 { 0 }
    pub fn za(&self, dnv: usize) -> u32 { 0 }
}

pub struct Btr {
    pub cm: SvmVmStats,
    pub ej: GuestRegs,
    pub apy: usize,
    pub ajv: u32,
    pub ku: LapicState,
    pub vmcb: Option<Vmcb>,
}

impl Btr {
    pub fn zvg(&self) -> String { String::new() }
    pub fn zcq(&self) -> String { String::new() }
    pub fn duy(&self, qbz: u64, jxx: usize) -> Option<&[u8]> { None }
    pub fn fvn(&mut self, qbl: &[u8], xyh: &str, xzz: Option<&[u8]>) -> Result<(), String> {
        Err(String::from("SVM not available on this architecture"))
    }
}
