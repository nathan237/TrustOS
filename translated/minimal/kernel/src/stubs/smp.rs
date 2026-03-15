

use alloc::vec::Vec;

pub const AN_: usize = 64;

#[repr(C)]
pub struct PerCpuData {
    pub qq: u32,
    pub aed: u32,
    pub eoh: u64,
    pub flu: u32,
    pub bhg: u64,
    pub mni: u64,
    pub dng: u64,
}

impl PerCpuData {
    pub const fn new(qq: u32, aed: u32) -> Self {
        Self {
            qq, aed, eoh: 0,
            flu: 0, bhg: 0,
            mni: 0, dng: 0,
        }
    }
}

pub struct Aza {
    pub aao: u32,
    pub gbo: u32,
    pub iju: Vec<u32>,
}

impl Aza {
    pub fn dgf() -> Self {
        Self { aao: 1, gbo: 0, iju: Vec::new() }
    }
}

pub type Afv = fn(usize, usize, *mut u8);

pub struct WorkItem {
    pub ke: Option<Afv>,
    pub ay: usize,
    pub ci: usize,
    pub f: *mut u8,
}

impl WorkItem {
    pub const fn azs() -> Self {
        Self { ke: None, ay: 0, ci: 0, f: core::ptr::null_mut() }
    }
}

static mut Bck: PerCpuData = PerCpuData::new(0, 0);

pub fn init() {}
pub fn ead() -> u32 { 0 }
pub fn cv() -> &'static PerCpuData { unsafe { &Bck } }
pub fn rry() -> &'static mut PerCpuData { unsafe { &mut Bck } }
pub fn aao() -> u32 { 1 }
pub fn piv(jxu: u32) {}
pub fn boc() -> u32 { 1 }
pub fn lga(qbp: u32) -> bool { true }
pub fn xtm() {}
pub fn phx(ydk: u32) {}
pub fn lvm() {}
pub fn asx() -> (u32, u32, u64) { (1, 1, 0) }
pub fn isq() {}
pub fn kqd() {}
pub fn jbt() -> bool { false }
pub fn daj(ejz: usize, ke: Afv, f: *mut u8) {
    ke(0, ejz, f);
}


pub unsafe extern "C" fn mvx(ycy: &limine::smp::Cpu) -> ! {
    loop { crate::arch::bhd(); }
}
