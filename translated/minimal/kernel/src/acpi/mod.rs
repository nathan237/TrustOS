







pub mod tables;
pub mod madt;
pub mod fadt;
pub mod mcfg;
pub mod hpet;

use alloc::vec::Vec;
use alloc::string::String;
use spin::Once;
use core::ptr;


#[derive(Debug, Clone)]
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

impl Default for AcpiInfo {
    fn default() -> Self {
        Self {
            revision: 0,
            oem_id: String::new(),
            local_apics: Vec::new(),
            io_apics: Vec::new(),
            int_overrides: Vec::new(),
            local_apic_nmis: Vec::new(),
            local_apic_addr: 0xFEE0_0000,
            fadt: None,
            mcfg_regions: Vec::new(),
            hpet: None,
            cpu_count: 1,
        }
    }
}

static GO_: Once<AcpiInfo> = Once::new();


pub fn rk() -> Option<&'static AcpiInfo> {
    GO_.get()
}


pub fn igo(ddr: u64) -> bool {
    if ddr == 0 {
        crate::serial_println!("[ACPI] No RSDP provided");
        return false;
    }
    
    let bz = crate::memory::hhdm_offset();
    
    
    crate::serial_println!("[ACPI] RSDP phys={:#x}, mapping...", ddr);
    
    
    match crate::memory::yv(ddr, 4096) {
        Ok(aoz) => {
            crate::serial_println!("[ACPI] RSDP mapped at virt={:#x}", aoz);
            gco(aoz, bz)
        }
        Err(e) => {
            crate::serial_println!("[ACPI] Failed to map RSDP: {}", e);
            false
        }
    }
}



pub fn mpf(biz: u64) -> bool {
    if biz == 0 {
        crate::serial_println!("[ACPI] No RSDP provided");
        return false;
    }
    
    let bz = crate::memory::hhdm_offset();
    
    
    
    let aoz = if biz >= bz {
        
        biz
    } else {
        
        biz + bz
    };
    
    let dyc = aoz - bz;
    crate::serial_println!("[ACPI] RSDP at phys={:#x} virt={:#x}", dyc, aoz);
    
    gco(aoz, bz)
}


pub fn init(dyc: u64) -> bool {
    if dyc == 0 {
        crate::serial_println!("[ACPI] No RSDP provided");
        return false;
    }
    
    let bz = crate::memory::hhdm_offset();
    let aoz = dyc + bz;
    
    crate::serial_println!("[ACPI] RSDP at phys={:#x} virt={:#x}", dyc, aoz);
    
    gco(aoz, bz)
}


fn gco(aoz: u64, bz: u64) -> bool {
    crate::serial_println!("[ACPI] About to read RSDP...");
    
    
    let ems = unsafe { core::ptr::read_volatile(aoz as *const u8) };
    crate::serial_println!("[ACPI] First byte: {:#x}", ems);
    
    
    let mut dzl = [0u8; 8];
    for i in 0..8 {
        dzl[i] = unsafe { core::ptr::read_volatile((aoz as *const u8).add(i)) };
    }
    
    
    let expected = b"RSD PTR ";
    let jgh = dzl == *expected;
    crate::serial_println!("[ACPI] Sig OK: {}", jgh);
    
    
    let asd = unsafe { &*(aoz as *const tables::Aoo) };
    
    if !jgh {
        crate::serial_println!("[ACPI] Invalid RSDP signature");
        return false;
    }
    
    
    let sum: u8 = unsafe {
        let bytes = core::slice::from_raw_parts(aoz as *const u8, 20);
        bytes.iter().fold(0u8, |aku, &b| aku.wrapping_add(b))
    };
    if sum != 0 {
        crate::serial_println!("[ACPI] Invalid RSDP checksum (sum={})", sum);
        return false;
    }
    
    let mut info = AcpiInfo::default();
    info.revision = asd.revision;
    info.oem_id = core::str::from_utf8(&asd.oem_id)
        .unwrap_or("Unknown")
        .trim()
        .into();
    
    crate::serial_println!("[ACPI] Revision: {}, OEM: {}", info.revision, info.oem_id);
    
    
    let ebf = if info.revision >= 2 {
        let jsb = unsafe { &*(aoz as *const tables::Ase) };
        
        
        let pvv = unsafe { ptr::read_unaligned(ptr::addr_of!(jsb.length)) };
        let xsdt_address = unsafe { ptr::read_unaligned(ptr::addr_of!(jsb.xsdt_address)) };
        let rsdt_address = unsafe { ptr::read_unaligned(ptr::addr_of!(asd.rsdt_address)) };
        
        
        let lti: u8 = unsafe {
            let bytes = core::slice::from_raw_parts(aoz as *const u8, pvv as usize);
            bytes.iter().fold(0u8, |aku, &b| aku.wrapping_add(b))
        };
        if lti != 0 {
            crate::serial_println!("[ACPI] Invalid XSDP extended checksum, falling back to RSDT");
            
            if let Err(e) = crate::memory::yv(rsdt_address as u64, 4096) {
                crate::serial_println!("[ACPI] Failed to map RSDT: {}", e);
                return false;
            }
            itx(rsdt_address as u64 + bz)
        } else {
            crate::serial_println!("[ACPI] Using XSDT at {:#x}", xsdt_address);
            
            if let Err(e) = crate::memory::yv(xsdt_address, 4096) {
                crate::serial_println!("[ACPI] Failed to map XSDT: {}", e);
                return false;
            }
            nrq(xsdt_address + bz)
        }
    } else {
        let rsdt_address = unsafe { ptr::read_unaligned(ptr::addr_of!(asd.rsdt_address)) };
        crate::serial_println!("[ACPI] Using RSDT at {:#x}", rsdt_address);
        
        if let Err(e) = crate::memory::yv(rsdt_address as u64, 4096) {
            crate::serial_println!("[ACPI] Failed to map RSDT: {}", e);
            return false;
        }
        itx(rsdt_address as u64 + bz)
    };
    
    crate::serial_println!("[ACPI] Found {} tables", ebf.len());
    
    
    for &asj in &ebf {
        
        if let Err(e) = crate::memory::yv(asj, 4096) {
            crate::serial_println!("[ACPI] Failed to map table at {:#x}: {}", asj, e);
            continue;
        }
        
        let bwf = asj + bz;
        let header = unsafe { &*(bwf as *const tables::Bu) };
        
        let sig = core::str::from_utf8(&header.signature).unwrap_or("????");
        let gai = unsafe { ptr::read_unaligned(ptr::addr_of!(header.length)) };
        
        
        if gai > 4096 {
            let ltm = gai as usize - 4096;
            if let Err(e) = crate::memory::yv(asj + 4096, ltm) {
                crate::serial_println!("[ACPI] Failed to map extended table: {}", e);
            }
        }
        
        crate::serial_println!("[ACPI] Table: {} at {:#x}, len={}", sig, asj, gai);
        
        match &header.signature {
            b"APIC" => {
                
                if let Some((ese, lapics, ioapics, evx, nmis)) = madt::parse(bwf) {
                    info.local_apic_addr = ese;
                    info.local_apics = lapics;
                    info.io_apics = ioapics;
                    info.int_overrides = evx;
                    info.local_apic_nmis = nmis;
                    info.cpu_count = info.local_apics.len();
                    crate::serial_println!("[ACPI] MADT: {} CPUs, {} I/O APICs, {} NMI entries", 
                        info.cpu_count, info.io_apics.len(), info.local_apic_nmis.len());
                }
            }
            b"FACP" => {
                
                if let Some(fadt_info) = fadt::parse(bwf, bz) {
                    crate::serial_println!("[ACPI] FADT: PM1a={:#x}, century={}", 
                        fadt_info.pm1a_evt_blk, fadt_info.century_reg);
                    info.fadt = Some(fadt_info);
                }
            }
            b"MCFG" => {
                
                if let Some(regions) = mcfg::parse(bwf) {
                    crate::serial_println!("[ACPI] MCFG: {} PCIe regions", regions.len());
                    info.mcfg_regions = regions;
                }
            }
            b"HPET" => {
                
                if let Some(hpet_info) = hpet::parse(bwf) {
                    crate::serial_println!("[ACPI] HPET: base={:#x}, min_tick={}", 
                        hpet_info.base_address, hpet_info.min_tick);
                    info.hpet = Some(hpet_info);
                }
            }
            _ => {
                
            }
        }
    }
    
    GO_.call_once(|| info);
    
    true
}


fn itx(rsdt_virt: u64) -> Vec<u64> {
    let header = unsafe { &*(rsdt_virt as *const tables::Bu) };
    
    
    if &header.signature != b"RSDT" {
        crate::serial_println!("[ACPI] Invalid RSDT signature");
        return Vec::new();
    }
    
    let ciy = rsdt_virt + core::mem::size_of::<tables::Bu>() as u64;
    let bsg = (header.length as usize - core::mem::size_of::<tables::Bu>()) / 4;
    
    let mut dhf = Vec::with_capacity(bsg);
    for i in 0..bsg {
        let addr = unsafe { ptr::read_unaligned((ciy + i as u64 * 4) as *const u32) };
        dhf.push(addr as u64);
    }
    
    dhf
}


fn nrq(xsdt_virt: u64) -> Vec<u64> {
    let header = unsafe { &*(xsdt_virt as *const tables::Bu) };
    
    
    if &header.signature != b"XSDT" {
        crate::serial_println!("[ACPI] Invalid XSDT signature");
        return Vec::new();
    }
    
    let ciy = xsdt_virt + core::mem::size_of::<tables::Bu>() as u64;
    let bsg = (header.length as usize - core::mem::size_of::<tables::Bu>()) / 8;
    
    let mut dhf = Vec::with_capacity(bsg);
    for i in 0..bsg {
        let addr = unsafe { ptr::read_unaligned((ciy + i as u64 * 8) as *const u64) };
        dhf.push(addr);
    }
    
    dhf
}


pub fn cpu_count() -> usize {
    GO_.get().map(|i| i.cpu_count).unwrap_or(1)
}


pub fn ggc() -> u64 {
    GO_.get().map(|i| i.local_apic_addr).unwrap_or(0xFEE0_0000)
}


pub fn is_initialized() -> bool {
    GO_.get().is_some()
}


pub fn shutdown() -> ! {
    if let Some(info) = GO_.get() {
        if let Some(ref fadt) = info.fadt {
            fadt::shutdown(fadt);
        }
    }
    
    
    crate::serial_println!("[ACPI] Shutdown failed, halting");
    loop {
        unsafe { core::arch::asm!("hlt"); }
    }
}


pub fn crf() -> bool {
    if let Some(info) = GO_.get() {
        if let Some(ref fadt) = info.fadt {
            return fadt::oyt(fadt);
        }
    }
    crate::serial_println!("[ACPI] No FADT available for S3 suspend");
    false
}


pub fn eya() -> ! {
    
    unsafe {
        
        for _ in 0..10000 {
            let status = x86_64::instructions::port::Port::<u8>::new(0x64).read();
            if (status & 0x02) == 0 {
                break;
            }
        }
        
        x86_64::instructions::port::Port::<u8>::new(0x64).write(0xFE);
    }
    
    
    if let Some(info) = GO_.get() {
        if let Some(ref fadt) = info.fadt {
            fadt::reset(fadt);
        }
    }
    
    
    crate::serial_println!("[ACPI] Reboot failed, triple faulting");
    unsafe {
        
        core::arch::asm!(
            "lidt [{}]",
            "int3",
            in(reg) &[0u64; 2],
            options(noreturn)
        );
    }
}
