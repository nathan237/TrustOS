






use alloc::vec::Vec;
use super::tables::Bu;


#[repr(C, packed)]
struct Ams {
    
    local_apic_addr: u32,
    
    flags: u32,
}


#[repr(C, packed)]
#[derive(Clone, Copy)]
struct Ih {
    entry_type: u8,
    length: u8,
}


const BWR_: u8 = 0;
const BWQ_: u8 = 1;
const BWP_: u8 = 2;
const DOT_: u8 = 3;
const BWT_: u8 = 4;
const BWS_: u8 = 5;
const BWU_: u8 = 9;


#[repr(C, packed)]
struct Amk {
    header: Ih,
    
    acpi_processor_id: u8,
    
    apic_id: u8,
    
    flags: u32,
}


#[repr(C, packed)]
struct Alw {
    header: Ih,
    
    io_apic_id: u8,
    
    _reserved: u8,
    
    io_apic_addr: u32,
    
    gsi_base: u32,
}


#[repr(C, packed)]
struct Alv {
    header: Ih,
    
    bus: u8,
    
    source: u8,
    
    gsi: u32,
    
    flags: u16,
}


#[repr(C, packed)]
struct Aml {
    header: Ih,
    
    acpi_processor_uid: u8,
    
    flags: u16,
    
    lint: u8,
}


#[repr(C, packed)]
struct Amj {
    header: Ih,
    
    _reserved: u16,
    
    local_apic_addr: u64,
}


#[repr(C, packed)]
struct Asd {
    header: Ih,
    
    _reserved: u16,
    
    x2apic_id: u32,
    
    flags: u32,
    
    acpi_processor_uid: u32,
}


#[derive(Debug, Clone)]
pub struct Kc {
    
    pub apic_id: u32,
    
    pub processor_id: u32,
    
    pub enabled: bool,
    
    pub online_capable: bool,
}


#[derive(Debug, Clone)]
pub struct Mh {
    
    pub id: u8,
    
    pub address: u64,
    
    pub gsi_base: u32,
}


#[derive(Debug, Clone)]
pub struct Kb {
    
    pub source: u8,
    
    pub gsi: u32,
    
    pub polarity: u8,
    
    pub trigger: u8,
}


#[derive(Debug, Clone)]
pub struct Mn {
    
    pub processor_uid: u8,
    
    pub lint: u8,
    
    pub polarity: u8,
    
    pub trigger: u8,
}


pub fn parse(madt_virt: u64) -> Option<(u64, Vec<Kc>, Vec<Mh>, Vec<Kb>, Vec<Mn>)> {
    let header = unsafe { &*(madt_virt as *const Bu) };
    
    
    if &header.signature != b"APIC" {
        return None;
    }
    
    let iln = core::mem::size_of::<Bu>();
    let nbu = unsafe { 
        &*((madt_virt + iln as u64) as *const Ams) 
    };
    
    let mut local_apic_addr = unsafe { 
        core::ptr::read_unaligned(core::ptr::addr_of!(nbu.local_apic_addr)) 
    } as u64;
    
    let mut local_apics = Vec::new();
    let mut io_apics = Vec::new();
    let mut evx = Vec::new();
    let mut iqp = Vec::new();
    
    
    let ciy = madt_virt + iln as u64 + 8;
    let pcu = madt_virt + header.length as u64;
    let mut offset = ciy;
    
    while offset + 2 <= pcu {
        let aob = unsafe { &*(offset as *const Ih) };
        
        if aob.length < 2 {
            break;
        }
        
        match aob.entry_type {
            BWR_ => {
                if aob.length >= 8 {
                    let entry = unsafe { &*(offset as *const Amk) };
                    let flags = unsafe { core::ptr::read_unaligned(core::ptr::addr_of!(entry.flags)) };
                    
                    local_apics.push(Kc {
                        apic_id: entry.apic_id as u32,
                        processor_id: entry.acpi_processor_id as u32,
                        enabled: (flags & 1) != 0,
                        online_capable: (flags & 2) != 0,
                    });
                }
            }
            BWQ_ => {
                if aob.length >= 12 {
                    let entry = unsafe { &*(offset as *const Alw) };
                    let addr = unsafe { core::ptr::read_unaligned(core::ptr::addr_of!(entry.io_apic_addr)) };
                    let gsi_base = unsafe { core::ptr::read_unaligned(core::ptr::addr_of!(entry.gsi_base)) };
                    
                    io_apics.push(Mh {
                        id: entry.io_apic_id,
                        address: addr as u64,
                        gsi_base,
                    });
                }
            }
            BWP_ => {
                if aob.length >= 10 {
                    let entry = unsafe { &*(offset as *const Alv) };
                    let gsi = unsafe { core::ptr::read_unaligned(core::ptr::addr_of!(entry.gsi)) };
                    let flags = unsafe { core::ptr::read_unaligned(core::ptr::addr_of!(entry.flags)) };
                    
                    evx.push(Kb {
                        source: entry.source,
                        gsi,
                        polarity: (flags & 0x03) as u8,
                        trigger: ((flags >> 2) & 0x03) as u8,
                    });
                }
            }
            BWS_ => {
                if aob.length >= 12 {
                    let entry = unsafe { &*(offset as *const Amj) };
                    local_apic_addr = unsafe { 
                        core::ptr::read_unaligned(core::ptr::addr_of!(entry.local_apic_addr)) 
                    };
                }
            }
            BWT_ => {
                if aob.length >= 6 {
                    let entry = unsafe { &*(offset as *const Aml) };
                    let flags = unsafe { core::ptr::read_unaligned(core::ptr::addr_of!(entry.flags)) };
                    iqp.push(Mn {
                        processor_uid: entry.acpi_processor_uid,
                        lint: entry.lint,
                        polarity: (flags & 0x03) as u8,
                        trigger: ((flags >> 2) & 0x03) as u8,
                    });
                }
            }
            BWU_ => {
                if aob.length >= 16 {
                    let entry = unsafe { &*(offset as *const Asd) };
                    let x2apic_id = unsafe { core::ptr::read_unaligned(core::ptr::addr_of!(entry.x2apic_id)) };
                    let flags = unsafe { core::ptr::read_unaligned(core::ptr::addr_of!(entry.flags)) };
                    let uid = unsafe { core::ptr::read_unaligned(core::ptr::addr_of!(entry.acpi_processor_uid)) };
                    
                    local_apics.push(Kc {
                        apic_id: x2apic_id,
                        processor_id: uid,
                        enabled: (flags & 1) != 0,
                        online_capable: (flags & 2) != 0,
                    });
                }
            }
            _ => {
                
            }
        }
        
        offset += aob.length as u64;
    }
    
    Some((local_apic_addr, local_apics, io_apics, evx, iqp))
}
