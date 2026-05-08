//! ACPI stub for non-x86_64 architectures
//!
//! Mirrors the public API of `kernel/src/acpi/` so consumers
//! (hwdiag, marionet, jarvis_hw) compile on aarch64/riscv.
//! All operations are no-op or return Default-like values.

use alloc::string::String;
use alloc::vec::Vec;

pub mod tables {
    pub fn find_table(_signature: &[u8; 4]) -> Option<u64> { None }
}

pub mod madt {
    #[derive(Debug, Clone, Default)]
    pub struct LocalApic {
        pub apic_id: u32,
        pub processor_id: u32,
        pub enabled: bool,
        pub online_capable: bool,
    }

    #[derive(Debug, Clone, Default)]
    pub struct IoApic {
        pub id: u8,
        pub address: u64,
        pub gsi_base: u32,
    }

    #[derive(Debug, Clone, Default)]
    pub struct IntSourceOverride {
        pub source: u8,
        pub gsi: u32,
        pub polarity: u8,
        pub trigger: u8,
    }

    #[derive(Debug, Clone, Default)]
    pub struct LocalApicNmiInfo {
        pub processor_uid: u8,
        pub lint: u8,
        pub polarity: u8,
        pub trigger: u8,
    }
}

pub mod fadt {
    /// Generic Address Structure (placeholder).
    #[derive(Debug, Clone, Copy, Default)]
    pub struct GenericAddress {
        pub address_space: u8,
        pub bit_width: u8,
        pub bit_offset: u8,
        pub access_size: u8,
        pub address: u64,
    }

    #[derive(Debug, Clone, Default)]
    pub struct FadtInfo {
        pub sci_int: u16,
        pub smi_cmd: u32,
        pub acpi_enable: u8,
        pub acpi_disable: u8,
        pub pm1a_evt_blk: u32,
        pub pm1b_evt_blk: u32,
        pub pm1a_cnt_blk: u32,
        pub pm1b_cnt_blk: u32,
        pub pm_tmr_blk: u32,
        pub century_reg: u8,
        pub reset_reg: GenericAddress,
        pub reset_value: u8,
        pub sleep_ctrl_reg: Option<GenericAddress>,
        pub sleep_status_reg: Option<GenericAddress>,
        pub flags: u32,
    }

    impl FadtInfo {
        pub const FLAG_HW_REDUCED: u32 = 1 << 20;
        pub const FLAG_LOW_POWER_S0: u32 = 1 << 21;
        pub const FLAG_WBINVD: u32 = 1 << 0;
        pub const FLAG_RESET_REG_SUP: u32 = 1 << 10;

        pub fn is_hw_reduced(&self) -> bool {
            (self.flags & Self::FLAG_HW_REDUCED) != 0
        }

        pub fn supports_reset(&self) -> bool {
            (self.flags & Self::FLAG_RESET_REG_SUP) != 0
        }
    }
}

pub mod mcfg {
    #[derive(Debug, Clone, Default)]
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
    #[derive(Debug, Clone, Default)]
    pub struct HpetInfo {
        pub base_address: u64,
        pub hpet_number: u8,
        pub min_tick: u16,
        pub num_comparators: u8,
        pub counter_64bit: bool,
        pub legacy_capable: bool,
        pub vendor_id: u16,
        pub period_fs: u32,
    }

    impl HpetInfo {
        /// Tick frequency (Hz) computed from period (femtoseconds).
        pub fn frequency(&self) -> u64 {
            if self.period_fs == 0 { 0 }
            else { 1_000_000_000_000_000u64 / self.period_fs as u64 }
        }
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
