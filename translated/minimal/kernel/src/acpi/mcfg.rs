



use alloc::vec::Vec;
use super::tables::Bu;


#[repr(C, packed)]
struct Azn {
    header: Bu,
    
    _reserved: u64,
    
}


#[repr(C, packed)]
#[derive(Clone, Copy)]
struct Abg {
    
    base_address: u64,
    
    segment_group: u16,
    
    start_bus: u8,
    
    end_bus: u8,
    
    _reserved: u32,
}


#[derive(Debug, Clone)]
pub struct Ij {
    
    pub base_address: u64,
    
    pub segment: u16,
    
    pub start_bus: u8,
    
    pub end_bus: u8,
}

impl Ij {
    
    pub fn config_address(&self, bus: u8, device: u8, function: u8) -> Option<u64> {
        if bus < self.start_bus || bus > self.end_bus {
            return None;
        }
        if device > 31 || function > 7 {
            return None;
        }
        
        
        
        let offset = ((bus.saturating_sub(self.start_bus)) as u64) << 20 
                   | (device as u64) << 15 
                   | (function as u64) << 12;
        
        Some(self.base_address + offset)
    }
    
    
    pub fn size(&self) -> u64 {
        let fkf = (self.end_bus.saturating_sub(self.start_bus) as u64).saturating_add(1);
        fkf << 20  
    }
}


pub fn parse(mcfg_virt: u64) -> Option<Vec<Ij>> {
    let header = unsafe { &*(mcfg_virt as *const Bu) };
    
    
    if &header.signature != b"MCFG" {
        return None;
    }
    
    
    let bms = core::mem::size_of::<Bu>() + 8; 
    let oi = core::mem::size_of::<Abg>();
    let bsg = (header.length as usize - bms) / oi;
    
    if bsg == 0 {
        return None;
    }
    
    let mut entries = Vec::with_capacity(bsg);
    let ciy = mcfg_virt + bms as u64;
    
    for i in 0..bsg {
        let cxg = ciy + (i * oi) as u64;
        let dm = unsafe { &*(cxg as *const Abg) };
        
        let base = unsafe { core::ptr::read_unaligned(core::ptr::addr_of!(dm.base_address)) };
        let gq = unsafe { core::ptr::read_unaligned(core::ptr::addr_of!(dm.segment_group)) };
        
        entries.push(Ij {
            base_address: base,
            segment: gq,
            start_bus: dm.start_bus,
            end_bus: dm.end_bus,
        });
    }
    
    Some(entries)
}


pub fn qib(segment: u16, bus: u8, device: u8, function: u8) -> Option<u64> {
    let info = super::rk()?;
    
    for entry in &info.mcfg_regions {
        if entry.segment == segment && bus >= entry.start_bus && bus <= entry.end_bus {
            return entry.config_address(bus, device, function);
        }
    }
    
    None
}


pub fn sw() -> bool {
    super::rk()
        .map(|i| !i.mcfg_regions.is_empty())
        .unwrap_or(false)
}
