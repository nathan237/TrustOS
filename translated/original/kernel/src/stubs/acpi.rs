//! ACPI stub for non-x86_64 architectures
//!
//! Provides the same public API as the real acpi module.

use alloc::string::String;
use alloc::vec::Vec;

pub mod tables {
    pub fn find_table(_signature: &[u8; 4]) -> Option<u64> { None }
}

pub mod madt {
    #[derive(Clone)]
    pub struct LocalApic { pub processor_id: u8, pub apic_id: u8, pub flags: u32 }
    #[derive(Clone)]
    pub struct IoApic { pub id: u8, pub address: u32, pub gsi_base: u32 }
    #[derive(Clone)]
    pub struct IntSourceOverride { pub bus: u8, pub source: u8, pub gsi: u32, pub flags: u16 }
    #[derive(Clone)]
    pub struct LocalApicNmiInfo { pub processor_id: u8, pub flags: u16, pub lint: u8 }
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
    pub struct McfgEntry {
        pub base_address: u64,
        pub segment: u16,
        pub start_bus: u8,
        pub end_bus: u8,
    }

    impl McfgEntry {
        pub fn size(&self) -> u64 {
            let buses = (self.end_bus - self.start_bus + 1) as u64;
            buses << 20
        }

        pub fn config_address(&self, _bus: u8, _device: u8, _function: u8) -> Option<u64> {
            None
        }
    }
}

pub mod hpet {
    #[derive(Clone)]
    pub struct HpetInfo {
        pub address: u64,
        pub minimum_tick: u16,
    }

    pub fn init() -> bool { false }
    pub fn is_initialized() -> bool { false }
    pub fn read_counter() -> u64 { 0 }
    pub fn frequency_hz() -> u64 { 0 }
    pub fn nanos_since_boot() -> u64 { 0 }
}

pub struct AcpiInfo {
    pub revision: u8,
    pub oem_id: String,
    pub local_apics: Vec<madt::LocalApic>,
    pub io_apics: Vec<madt::IoApic>,
    pub int_overrides: Vec<madt::IntSourceOverride>,
    pub local_apic_nmis: Vec<madt::LocalApicNmiInfo>,
    pub local_apic_addr: u64,
    pub fadt: Option<fadt::FadtInfo>,
    pub mcfg_regions: Vec<mcfg::McfgEntry>,
    pub hpet: Option<hpet::HpetInfo>,
    pub cpu_count: usize,
}

pub fn get_info() -> Option<&'static AcpiInfo> { None }
pub fn init(_rsdp_phys: u64) -> bool { false }
pub fn init_from_virt(_rsdp_addr: u64) -> bool { false }
pub fn init_direct(_rsdp_ptr: u64) -> bool { false }
pub fn cpu_count() -> usize { 1 }
pub fn local_apic_address() -> u64 { 0 }
pub fn is_initialized() -> bool { false }

pub fn shutdown() -> ! { loop { crate::arch::halt(); } }
pub fn suspend() -> bool { false }
pub fn reboot() -> ! { loop { crate::arch::halt(); } }
