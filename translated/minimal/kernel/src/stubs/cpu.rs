





use alloc::string::String;
use alloc::vec::Vec;


pub mod features;
pub mod tsc;
pub mod simd;
pub mod smp;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CpuVendor {
    Ef,
    Ct,
    F,
}


pub struct CpuCapabilities {
    pub acs: CpuVendor,
    pub family: u8,
    pub model: u8,
    pub bxi: u8,
    pub aed: u8,
    pub dem: [u8; 48],
    pub tsc: bool,
    pub fan: bool,
    pub ifc: bool,
    pub fsd: bool,
    pub eiw: bool,
    pub eix: bool,
    pub fvj: bool,
    pub fvl: bool,
    pub fvk: bool,
    pub eyy: bool,
    pub dof: bool,
    pub dog: bool,
    pub eml: bool,
    pub doa: bool,
    pub ewm: bool,
    pub eyl: bool,
    pub cbg: bool,
    pub cmc: bool,
    pub vt: bool,
    pub cia: bool,
    pub cul: bool,
    pub ddd: bool,
    pub vmx: bool,
    pub svm: bool,
    pub cau: u8,
    pub djk: u8,
    pub ekf: u64,
}

impl CpuCapabilities {
    pub fn dgf() -> Self {
        Self {
            acs: CpuVendor::F,
            family: 0,
            model: 0,
            bxi: 0,
            aed: 0,
            dem: [0; 48],
            tsc: false,
            fan: false,
            ifc: false,
            fsd: false,
            eiw: false,
            eix: false,
            fvj: false,
            fvl: false,
            fvk: false,
            eyy: false,
            dof: false,
            dog: false,
            eml: false,
            doa: false,
            ewm: false,
            eyl: false,
            cbg: false,
            cmc: false,
            vt: false,
            cia: false,
            cul: false,
            ddd: false,
            vmx: false,
            svm: false,
            cau: 1,
            djk: 1,
            ekf: 1_000_000_000,
        }
    }

    pub fn keu(&self) -> &str {
        crate::arch::kav()
    }
}

static mut Bcw: Option<CpuCapabilities> = None;

pub fn init() {
    unsafe { Bcw = Some(CpuCapabilities::dgf()); }
}

pub fn bme() -> Option<&'static CpuCapabilities> {
    unsafe { Bcw.as_ref() }
}

pub fn mnh() -> u64 {
    1_000_000_000 
}

pub fn cfe() -> bool { false }
pub fn crd() -> bool { false }
pub fn gdj() -> u8 { 1 }

pub fn cbg() -> Option<u64> { None }
pub fn cmc() -> Option<u64> { None }
