



use alloc::string::String;
use alloc::vec::Vec;

pub mod tables {
    pub fn qfv(jss: &[u8; 4]) -> Option<u64> { None }
}

pub mod madt {
    #[derive(Clone)]
    pub struct Kc { pub processor_id: u8, pub apic_id: u8, pub flags: u32 }
    #[derive(Clone)]
    pub struct Mh { pub id: u8, pub address: u32, pub gsi_base: u32 }
    #[derive(Clone)]
    pub struct Kb { pub bus: u8, pub source: u8, pub gsi: u32, pub flags: u16 }
    #[derive(Clone)]
    pub struct Mn { pub processor_id: u8, pub flags: u16, pub lint: u8 }
}

pub mod fadt {
    #[derive(Clone)]
    pub struct FadtInfo {
        pub pm1a_control_block: u64,
        pub pm1b_control_block: u64,
        pub slp_typa: u8,
        pub slp_typb: u8,
        pub century: u8,
        pub reset_reg_addr: u64,
        pub reset_value: u8,
    }
}

pub mod mcfg {
    #[derive(Clone)]
    pub struct Ij {
        pub base_address: u64,
        pub segment: u16,
        pub start_bus: u8,
        pub end_bus: u8,
    }

    impl Ij {
        pub fn size(&self) -> u64 {
            let fkf = (self.end_bus - self.start_bus + 1) as u64;
            fkf << 20
        }

        pub fn config_address(&self, _bus: u8, _device: u8, _function: u8) -> Option<u64> {
            None
        }
    }
}

pub mod hpet {
    #[derive(Clone)]
    pub struct Jx {
        pub address: u64,
        pub minimum_tick: u16,
    }

    pub fn init() -> bool { false }
    pub fn is_initialized() -> bool { false }
    pub fn ocl() -> u64 { 0 }
    pub fn we() -> u64 { 0 }
    pub fn qph() -> u64 { 0 }
}

pub struct AcpiInfo {
    pub revision: u8,
    pub oem_id: String,
    pub local_apics: Vec<madt::Kc>,
    pub io_apics: Vec<madt::Mh>,
    pub int_overrides: Vec<madt::Kb>,
    pub local_apic_nmis: Vec<madt::Mn>,
    pub local_apic_addr: u64,
    pub fadt: Option<fadt::FadtInfo>,
    pub mcfg_regions: Vec<mcfg::Ij>,
    pub hpet: Option<hpet::Jx>,
    pub cpu_count: usize,
}

pub fn rk() -> Option<&'static AcpiInfo> { None }
pub fn init(_rsdp_phys: u64) -> bool { false }
pub fn mpf(_rsdp_addr: u64) -> bool { false }
pub fn igo(_rsdp_ptr: u64) -> bool { false }
pub fn cpu_count() -> usize { 1 }
pub fn ggc() -> u64 { 0 }
pub fn is_initialized() -> bool { false }

pub fn shutdown() -> ! { loop { crate::arch::acb(); } }
pub fn crf() -> bool { false }
pub fn eya() -> ! { loop { crate::arch::acb(); } }
