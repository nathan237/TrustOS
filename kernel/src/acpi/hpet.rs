//! HPET (High Precision Event Timer) Parser
//!
//! The HPET provides high-resolution timing capabilities.

use super::tables::SdtHeader;

/// HPET table structure
#[repr(C, packed)]
struct HpetTable {
    header: SdtHeader,
    
    /// Hardware ID of Event Timer Block
    event_timer_block_id: u32,
    
    /// Base address (GAS format, but fixed layout for HPET)
    base_address_space_id: u8,
    base_register_bit_width: u8,
    base_register_bit_offset: u8,
    base_reserved: u8,
    base_address: u64,
    
    /// HPET sequence number
    hpet_number: u8,
    /// Minimum tick in periodic mode
    minimum_tick: u16,
    /// Page protection and OEM attribute
    page_protection: u8,
}

/// Parsed HPET information
#[derive(Debug, Clone)]
pub struct HpetInfo {
    /// HPET base address (memory-mapped)
    pub base_address: u64,
    /// HPET number (for systems with multiple HPETs)
    pub hpet_number: u8,
    /// Minimum tick value for periodic mode
    pub min_tick: u16,
    /// Number of comparators (from hardware ID)
    pub num_comparators: u8,
    /// Counter size (true = 64-bit, false = 32-bit)
    pub counter_64bit: bool,
    /// Supports legacy replacement (IRQ0/IRQ8)
    pub legacy_capable: bool,
    /// Vendor ID
    pub vendor_id: u16,
    /// Period in femtoseconds
    pub period_fs: u32,
}

/// HPET Register offsets
pub mod regs {
    /// General Capabilities and ID
    pub const CAP_ID: u64 = 0x000;
    /// General Configuration
    pub const CONFIG: u64 = 0x010;
    /// General Interrupt Status
    pub const INT_STATUS: u64 = 0x020;
    /// Main Counter Value
    pub const COUNTER: u64 = 0x0F0;
    /// Timer 0 Configuration and Capabilities
    pub const TIMER0_CONFIG: u64 = 0x100;
    /// Timer 0 Comparator
    pub const TIMER0_COMPARATOR: u64 = 0x108;
    /// Timer 0 FSB Interrupt Route
    pub const TIMER0_FSB: u64 = 0x110;
}

/// Parse HPET table
pub fn parse(hpet_virt: u64) -> Option<HpetInfo> {
    let header = unsafe { &*(hpet_virt as *const SdtHeader) };
    
    // Verify signature
    if &header.signature != b"HPET" {
        return None;
    }
    
    let hpet = unsafe { &*(hpet_virt as *const HpetTable) };
    
    let event_id = unsafe { core::ptr::read_unaligned(core::ptr::addr_of!(hpet.event_timer_block_id)) };
    let base_addr = unsafe { core::ptr::read_unaligned(core::ptr::addr_of!(hpet.base_address)) };
    let min_tick = unsafe { core::ptr::read_unaligned(core::ptr::addr_of!(hpet.minimum_tick)) };
    
    // Parse hardware ID
    let num_comparators = ((event_id >> 8) & 0x1F) as u8 + 1;
    let counter_64bit = (event_id & (1 << 13)) != 0;
    let legacy_capable = (event_id & (1 << 15)) != 0;
    let vendor_id = (event_id >> 16) as u16;
    
    // Read period from hardware registers if accessible
    let period_fs = if base_addr != 0 {
        // Map the HPET MMIO region before accessing
        match crate::memory::map_mmio(base_addr, 4096) {
            Ok(virt_addr) => {
                let cap = unsafe { core::ptr::read_volatile((virt_addr + regs::CAP_ID) as *const u64) };
                (cap >> 32) as u32
            }
            Err(e) => {
                crate::serial_println!("[HPET] Failed to map HPET MMIO at {:#x}: {}", base_addr, e);
                0
            }
        }
    } else {
        0
    };
    
    Some(HpetInfo {
        base_address: base_addr,
        hpet_number: hpet.hpet_number,
        min_tick,
        num_comparators,
        counter_64bit,
        legacy_capable,
        vendor_id,
        period_fs,
    })
}

impl HpetInfo {
    /// Get frequency in Hz
    pub fn frequency(&self) -> u64 {
        if self.period_fs == 0 {
            return 0;
        }
        // frequency = 10^15 / period_fs
        1_000_000_000_000_000u64 / self.period_fs as u64
    }
    
    /// Read current counter value
    pub fn read_counter(&self) -> u64 {
        let hhdm = crate::memory::hhdm_offset();
        let addr = self.base_address + hhdm + regs::COUNTER;
        unsafe { core::ptr::read_volatile(addr as *const u64) }
    }
    
    /// Enable/disable HPET
    pub fn set_enabled(&self, enabled: bool) {
        let hhdm = crate::memory::hhdm_offset();
        let config_addr = self.base_address + hhdm + regs::CONFIG;
        
        unsafe {
            let mut config = core::ptr::read_volatile(config_addr as *const u64);
            if enabled {
                config |= 1; // ENABLE_CNF
            } else {
                config &= !1;
            }
            core::ptr::write_volatile(config_addr as *mut u64, config);
        }
    }
    
    /// Enable/disable legacy replacement mode
    pub fn set_legacy_mode(&self, enabled: bool) {
        if !self.legacy_capable {
            return;
        }
        
        let hhdm = crate::memory::hhdm_offset();
        let config_addr = self.base_address + hhdm + regs::CONFIG;
        
        unsafe {
            let mut config = core::ptr::read_volatile(config_addr as *const u64);
            if enabled {
                config |= 2; // LEG_RT_CNF
            } else {
                config &= !2;
            }
            core::ptr::write_volatile(config_addr as *mut u64, config);
        }
    }
    
    /// Convert HPET ticks to nanoseconds
    pub fn ticks_to_nanos(&self, ticks: u64) -> u64 {
        // nanos = ticks * period_fs / 10^6
        if self.period_fs == 0 {
            return 0;
        }
        (ticks as u128 * self.period_fs as u128 / 1_000_000) as u64
    }
    
    /// Convert nanoseconds to HPET ticks
    pub fn nanos_to_ticks(&self, nanos: u64) -> u64 {
        if self.period_fs == 0 {
            return 0;
        }
        (nanos as u128 * 1_000_000 / self.period_fs as u128) as u64
    }
}

/// Initialize HPET if available
pub fn init() -> bool {
    let info = match super::get_info() {
        Some(i) => i,
        None => return false,
    };
    
    let hpet = match &info.hpet {
        Some(h) => h,
        None => {
            crate::serial_println!("[HPET] No HPET table found");
            return false;
        }
    };
    
    crate::serial_println!("[HPET] Initializing: base={:#x}, freq={} Hz", 
        hpet.base_address, hpet.frequency());
    
    // Enable HPET counter
    hpet.set_enabled(true);
    
    true
}
