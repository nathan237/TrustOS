//! ACPI stub for non-x86_64 architectures
//!
//! Provides the same public API as the real acpi module.

use alloc::string::String;
use alloc::vec::Vec;

pub mod tables {
        // Public function — callable from other modules.
pub fn find_table(_signature: &[u8; 4]) -> Option<u64> { None }
}

pub mod madt {
        // #[derive] — auto-generates trait implementations at compile time.
#[derive(Clone)]
        // Public structure — visible outside this module.
pub struct LocalApic { pub processor_id: u8, pub apic_id: u8, pub flags: u32 }
        // #[derive] — auto-generates trait implementations at compile time.
#[derive(Clone)]
        // Public structure — visible outside this module.
pub struct IoApic { pub id: u8, pub address: u32, pub gsi_base: u32 }
        // #[derive] — auto-generates trait implementations at compile time.
#[derive(Clone)]
        // Public structure — visible outside this module.
pub struct IntSourceOverride { pub bus: u8, pub source: u8, pub gsi: u32, pub flags: u16 }
        // #[derive] — auto-generates trait implementations at compile time.
#[derive(Clone)]
        // Public structure — visible outside this module.
pub struct LocalApicNmiInformation { pub processor_id: u8, pub flags: u16, pub lint: u8 }
}

pub mod fadt {
        // #[derive] — auto-generates trait implementations at compile time.
#[derive(Clone)]
        // Public structure — visible outside this module.
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
        // #[derive] — auto-generates trait implementations at compile time.
#[derive(Clone)]
        // Public structure — visible outside this module.
pub struct McfgEntry {
        pub base_address: u64,
        pub segment: u16,
        pub start_bus: u8,
        pub end_bus: u8,
    }

        // Implementation block — defines methods for the type above.
impl McfgEntry {
                // Public function — callable from other modules.
pub fn size(&self) -> u64 {
            let buses = (self.end_bus - self.start_bus + 1) as u64;
            buses << 20
        }

                // Public function — callable from other modules.
pub fn config_address(&self, _bus: u8, _device: u8, _function: u8) -> Option<u64> {
            None
        }
    }
}

pub mod hpet {
        // #[derive] — auto-generates trait implementations at compile time.
#[derive(Clone)]
        // Public structure — visible outside this module.
pub struct HpetInformation {
        pub address: u64,
        pub minimum_tick: u16,
    }

        // Public function — callable from other modules.
pub fn init() -> bool { false }
        // Public function — callable from other modules.
pub fn is_initialized() -> bool { false }
        // Public function — callable from other modules.
pub fn read_counter() -> u64 { 0 }
        // Public function — callable from other modules.
pub fn frequency_hz() -> u64 { 0 }
        // Public function — callable from other modules.
pub fn nanos_since_boot() -> u64 { 0 }
}

// Public structure — visible outside this module.
pub struct AcpiInfo {
    pub revision: u8,
    pub oem_id: String,
    pub local_apics: Vec<madt::LocalApic>,
    pub io_apics: Vec<madt::IoApic>,
    pub int_overrides: Vec<madt::IntSourceOverride>,
    pub local_apic_nmis: Vec<madt::LocalApicNmiInformation>,
    pub local_apic_addr: u64,
    pub fadt: Option<fadt::FadtInfo>,
    pub mcfg_regions: Vec<mcfg::McfgEntry>,
    pub hpet: Option<hpet::HpetInformation>,
    pub cpu_count: usize,
}

// Public function — callable from other modules.
pub fn get_information() -> Option<&'static AcpiInfo> { None }
// Public function — callable from other modules.
pub fn init(_rsdp_phys: u64) -> bool { false }
// Public function — callable from other modules.
pub fn initialize_from_virt(_rsdp_addr: u64) -> bool { false }
// Public function — callable from other modules.
pub fn initialize_direct(_rsdp_ptr: u64) -> bool { false }
// Public function — callable from other modules.
pub fn cpu_count() -> usize { 1 }
// Public function — callable from other modules.
pub fn local_apic_address() -> u64 { 0 }
// Public function — callable from other modules.
pub fn is_initialized() -> bool { false }

// Public function — callable from other modules.
pub fn shutdown() -> ! { // Infinite loop — runs until an explicit `break`.
loop { crate::arch::halt(); } }
// Public function — callable from other modules.
pub fn suspend() -> bool { false }
// Public function — callable from other modules.
pub fn reboot() -> ! { // Infinite loop — runs until an explicit `break`.
loop { crate::arch::halt(); } }
