



use alloc::string::String;
use alloc::vec::Vec;

pub mod tables {
    pub fn yqt(qdq: &[u8; 4]) -> Option<u64> { None }
}

pub mod madt {
    #[derive(Clone)]
    pub struct Xl { pub bny: u8, pub aed: u8, pub flags: u32 }
    #[derive(Clone)]
    pub struct Ach { pub ad: u8, pub re: u32, pub ech: u32 }
    #[derive(Clone)]
    pub struct Xc { pub aq: u8, pub iy: u8, pub bup: u32, pub flags: u16 }
    #[derive(Clone)]
    pub struct Acs { pub bny: u8, pub flags: u16, pub gln: u8 }
}

pub mod fadt {
    #[derive(Clone)]
    pub struct FadtInfo {
        pub zfn: u64,
        pub zfo: u64,
        pub zos: u8,
        pub zot: u8,
        pub hcn: u8,
        pub zjy: u64,
        pub hxp: u8,
    }
}

pub mod mcfg {
    #[derive(Clone)]
    pub struct Tl {
        pub bps: u64,
        pub ie: u16,
        pub cca: u8,
        pub cej: u8,
    }

    impl Tl {
        pub fn aw(&self) -> u64 {
            let kfo = (self.cej - self.cca + 1) as u64;
            kfo << 20
        }

        pub fn nfk(&self, xyd: u8, xyt: u8, xzl: u8) -> Option<u64> {
            None
        }
    }
}

pub mod hpet {
    #[derive(Clone)]
    pub struct Wy {
        pub re: u64,
        pub onn: u16,
    }

    pub fn init() -> bool { false }
    pub fn ky() -> bool { false }
    pub fn vrl() -> u64 { 0 }
    pub fn ard() -> u64 { 0 }
    pub fn zdg() -> u64 { 0 }
}

pub struct AcpiInfo {
    pub afe: u8,
    pub clo: String,
    pub dja: Vec<madt::Xl>,
    pub cyx: Vec<madt::Ach>,
    pub gka: Vec<madt::Xc>,
    pub fne: Vec<madt::Acs>,
    pub cap: u64,
    pub fadt: Option<fadt::FadtInfo>,
    pub eut: Vec<mcfg::Tl>,
    pub hpet: Option<hpet::Wy>,
    pub aao: usize,
}

pub fn ani() -> Option<&'static AcpiInfo> { None }
pub fn init(ycn: u64) -> bool { false }
pub fn ttm(ycm: u64) -> bool { false }
pub fn oeh(yco: u64) -> bool { false }
pub fn aao() -> usize { 1 }
pub fn ljo() -> u64 { 0 }
pub fn ky() -> bool { false }

pub fn cbu() -> ! { loop { crate::arch::bhd(); } }
pub fn fvw() -> bool { false }
pub fn jlq() -> ! { loop { crate::arch::bhd(); } }
