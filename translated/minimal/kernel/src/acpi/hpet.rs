



use super::tables::Bu;


#[repr(C, packed)]
struct Ald {
    header: Bu,
    
    
    event_timer_block_id: u32,
    
    
    base_address_space_id: u8,
    base_register_bit_width: u8,
    base_register_bit_offset: u8,
    base_reserved: u8,
    base_address: u64,
    
    
    hpet_number: u8,
    
    minimum_tick: u16,
    
    page_protection: u8,
}


#[derive(Debug, Clone)]
pub struct Jx {
    
    pub base_address: u64,
    
    pub hpet_number: u8,
    
    pub min_tick: u16,
    
    pub num_comparators: u8,
    
    pub counter_64bit: bool,
    
    pub legacy_capable: bool,
    
    pub vendor_id: u16,
    
    pub period_fs: u32,
}


pub mod regs {
    
    pub const BON_: u64 = 0x000;
    
    pub const Gh: u64 = 0x010;
    
    pub const DVB_: u64 = 0x020;
    
    pub const Ahx: u64 = 0x0F0;
    
    pub const ELH_: u64 = 0x100;
    
    pub const ELG_: u64 = 0x108;
    
    pub const ELI_: u64 = 0x110;
}


pub fn parse(hpet_virt: u64) -> Option<Jx> {
    let header = unsafe { &*(hpet_virt as *const Bu) };
    
    
    if &header.signature != b"HPET" {
        return None;
    }
    
    let hpet = unsafe { &*(hpet_virt as *const Ald) };
    
    let els = unsafe { core::ptr::read_unaligned(core::ptr::addr_of!(hpet.event_timer_block_id)) };
    let base_addr = unsafe { core::ptr::read_unaligned(core::ptr::addr_of!(hpet.base_address)) };
    let min_tick = unsafe { core::ptr::read_unaligned(core::ptr::addr_of!(hpet.minimum_tick)) };
    
    
    let num_comparators = ((els >> 8) & 0x1F) as u8 + 1;
    let counter_64bit = (els & (1 << 13)) != 0;
    let legacy_capable = (els & (1 << 15)) != 0;
    let vendor_id = (els >> 16) as u16;
    
    
    let period_fs = if base_addr != 0 {
        
        match crate::memory::yv(base_addr, 4096) {
            Ok(virt_addr) => {
                let cap = unsafe { core::ptr::read_volatile((virt_addr + regs::BON_) as *const u64) };
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
    
    Some(Jx {
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

impl Jx {
    
    pub fn frequency(&self) -> u64 {
        if self.period_fs == 0 {
            return 0;
        }
        
        1_000_000_000_000_000u64 / self.period_fs as u64
    }
    
    
    pub fn ocl(&self) -> u64 {
        let bz = crate::memory::hhdm_offset();
        let addr = self.base_address + bz + regs::Ahx;
        unsafe { core::ptr::read_volatile(addr as *const u64) }
    }
    
    
    pub fn set_enabled(&self, enabled: bool) {
        let bz = crate::memory::hhdm_offset();
        let config_addr = self.base_address + bz + regs::Gh;
        
        unsafe {
            let mut config = core::ptr::read_volatile(config_addr as *const u64);
            if enabled {
                config |= 1; 
            } else {
                config &= !1;
            }
            core::ptr::write_volatile(config_addr as *mut u64, config);
        }
    }
    
    
    pub fn qwe(&self, enabled: bool) {
        if !self.legacy_capable {
            return;
        }
        
        let bz = crate::memory::hhdm_offset();
        let config_addr = self.base_address + bz + regs::Gh;
        
        unsafe {
            let mut config = core::ptr::read_volatile(config_addr as *const u64);
            if enabled {
                config |= 2; 
            } else {
                config &= !2;
            }
            core::ptr::write_volatile(config_addr as *mut u64, config);
        }
    }
    
    
    pub fn rah(&self, gx: u64) -> u64 {
        
        if self.period_fs == 0 {
            return 0;
        }
        (gx as u128 * self.period_fs as u128 / 1_000_000) as u64
    }
    
    
    pub fn qpi(&self, bul: u64) -> u64 {
        if self.period_fs == 0 {
            return 0;
        }
        (bul as u128 * 1_000_000 / self.period_fs as u128) as u64
    }
}


pub fn init() -> bool {
    let info = match super::rk() {
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
    
    
    hpet.set_enabled(true);
    
    true
}
