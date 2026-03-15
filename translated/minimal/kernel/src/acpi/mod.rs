







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
    
    pub afe: u8,
    
    pub clo: String,
    
    pub dja: Vec<madt::Xl>,
    
    pub cyx: Vec<madt::Ach>,
    
    pub gka: Vec<madt::Xc>,
    
    pub fne: Vec<madt::Acs>,
    
    pub cap: u64,
    
    pub fadt: Option<fadt::FadtInfo>,
    
    pub eut: Vec<mcfg::Tl>,
    
    pub hpet: Option<hpet::Wy>,
    
    pub aao: usize,
}

impl Default for AcpiInfo {
    fn default() -> Self {
        Self {
            afe: 0,
            clo: String::new(),
            dja: Vec::new(),
            cyx: Vec::new(),
            gka: Vec::new(),
            fne: Vec::new(),
            cap: 0xFEE0_0000,
            fadt: None,
            eut: Vec::new(),
            hpet: None,
            aao: 1,
        }
    }
}

static FZ_: Once<AcpiInfo> = Once::new();


pub fn ani() -> Option<&'static AcpiInfo> {
    FZ_.get()
}


pub fn oeh(gre: u64) -> bool {
    if gre == 0 {
        crate::serial_println!("[ACPI] No RSDP provided");
        return false;
    }
    
    let hp = crate::memory::lr();
    
    
    crate::serial_println!("[ACPI] RSDP phys={:#x}, mapping...", gre);
    
    
    match crate::memory::bki(gre, 4096) {
        Ok(cbo) => {
            crate::serial_println!("[ACPI] RSDP mapped at virt={:#x}", cbo);
            lee(cbo, hp)
        }
        Err(aa) => {
            crate::serial_println!("[ACPI] Failed to map RSDP: {}", aa);
            false
        }
    }
}



pub fn ttm(dll: u64) -> bool {
    if dll == 0 {
        crate::serial_println!("[ACPI] No RSDP provided");
        return false;
    }
    
    let hp = crate::memory::lr();
    
    
    
    let cbo = if dll >= hp {
        
        dll
    } else {
        
        dll + hp
    };
    
    let hyd = cbo - hp;
    crate::serial_println!("[ACPI] RSDP at phys={:#x} virt={:#x}", hyd, cbo);
    
    lee(cbo, hp)
}


pub fn init(hyd: u64) -> bool {
    if hyd == 0 {
        crate::serial_println!("[ACPI] No RSDP provided");
        return false;
    }
    
    let hp = crate::memory::lr();
    let cbo = hyd + hp;
    
    crate::serial_println!("[ACPI] RSDP at phys={:#x} virt={:#x}", hyd, cbo);
    
    lee(cbo, hp)
}


fn lee(cbo: u64, hp: u64) -> bool {
    crate::serial_println!("[ACPI] About to read RSDP...");
    
    
    let iuu = unsafe { core::ptr::read_volatile(cbo as *const u8) };
    crate::serial_println!("[ACPI] First byte: {:#x}", iuu);
    
    
    let mut iap = [0u8; 8];
    for a in 0..8 {
        iap[a] = unsafe { core::ptr::read_volatile((cbo as *const u8).add(a)) };
    }
    
    
    let qy = b"RSD PTR ";
    let pkt = iap == *qy;
    crate::serial_println!("[ACPI] Sig OK: {}", pkt);
    
    
    let chl = unsafe { &*(cbo as *const tables::Cky) };
    
    if !pkt {
        crate::serial_println!("[ACPI] Invalid RSDP signature");
        return false;
    }
    
    
    let sum: u8 = unsafe {
        let bf = core::slice::anh(cbo as *const u8, 20);
        bf.iter().cqs(0u8, |btc, &o| btc.cn(o))
    };
    if sum != 0 {
        crate::serial_println!("[ACPI] Invalid RSDP checksum (sum={})", sum);
        return false;
    }
    
    let mut co = AcpiInfo::default();
    co.afe = chl.afe;
    co.clo = core::str::jg(&chl.clo)
        .unwrap_or("Unknown")
        .em()
        .into();
    
    crate::serial_println!("[ACPI] Revision: {}, OEM: {}", co.afe, co.clo);
    
    
    let icw = if co.afe >= 2 {
        let qat = unsafe { &*(cbo as *const tables::Cqr) };
        
        
        let xwq = unsafe { ptr::md(ptr::vf!(qat.go)) };
        let ihx = unsafe { ptr::md(ptr::vf!(qat.ihx)) };
        let dvi = unsafe { ptr::md(ptr::vf!(chl.dvi)) };
        
        
        let spu: u8 = unsafe {
            let bf = core::slice::anh(cbo as *const u8, xwq as usize);
            bf.iter().cqs(0u8, |btc, &o| btc.cn(o))
        };
        if spu != 0 {
            crate::serial_println!("[ACPI] Invalid XSDP extended checksum, falling back to RSDT");
            
            if let Err(aa) = crate::memory::bki(dvi as u64, 4096) {
                crate::serial_println!("[ACPI] Failed to map RSDT: {}", aa);
                return false;
            }
            oul(dvi as u64 + hp)
        } else {
            crate::serial_println!("[ACPI] Using XSDT at {:#x}", ihx);
            
            if let Err(aa) = crate::memory::bki(ihx, 4096) {
                crate::serial_println!("[ACPI] Failed to map XSDT: {}", aa);
                return false;
            }
            ven(ihx + hp)
        }
    } else {
        let dvi = unsafe { ptr::md(ptr::vf!(chl.dvi)) };
        crate::serial_println!("[ACPI] Using RSDT at {:#x}", dvi);
        
        if let Err(aa) = crate::memory::bki(dvi as u64, 4096) {
            crate::serial_println!("[ACPI] Failed to map RSDT: {}", aa);
            return false;
        }
        oul(dvi as u64 + hp)
    };
    
    crate::serial_println!("[ACPI] Found {} tables", icw.len());
    
    
    for &cig in &icw {
        
        if let Err(aa) = crate::memory::bki(cig, 4096) {
            crate::serial_println!("[ACPI] Failed to map table at {:#x}: {}", cig, aa);
            continue;
        }
        
        let ejk = cig + hp;
        let dh = unsafe { &*(ejk as *const tables::Ei) };
        
        let sig = core::str::jg(&dh.signature).unwrap_or("????");
        let lbs = unsafe { ptr::md(ptr::vf!(dh.go)) };
        
        
        if lbs > 4096 {
            let sqa = lbs as usize - 4096;
            if let Err(aa) = crate::memory::bki(cig + 4096, sqa) {
                crate::serial_println!("[ACPI] Failed to map extended table: {}", aa);
            }
        }
        
        crate::serial_println!("[ACPI] Table: {} at {:#x}, len={}", sig, cig, lbs);
        
        match &dh.signature {
            b"APIC" => {
                
                if let Some((jch, uby, twh, jif, uut)) = madt::parse(ejk) {
                    co.cap = jch;
                    co.dja = uby;
                    co.cyx = twh;
                    co.gka = jif;
                    co.fne = uut;
                    co.aao = co.dja.len();
                    crate::serial_println!("[ACPI] MADT: {} CPUs, {} I/O APICs, {} NMI entries", 
                        co.aao, co.cyx.len(), co.fne.len());
                }
            }
            b"FACP" => {
                
                if let Some(kva) = fadt::parse(ejk, hp) {
                    crate::serial_println!("[ACPI] FADT: PM1a={:#x}, century={}", 
                        kva.gpl, kva.nca);
                    co.fadt = Some(kva);
                }
            }
            b"MCFG" => {
                
                if let Some(afx) = mcfg::parse(ejk) {
                    crate::serial_println!("[ACPI] MCFG: {} PCIe regions", afx.len());
                    co.eut = afx;
                }
            }
            b"HPET" => {
                
                if let Some(lcs) = hpet::parse(ejk) {
                    crate::serial_println!("[ACPI] HPET: base={:#x}, min_tick={}", 
                        lcs.bps, lcs.llx);
                    co.hpet = Some(lcs);
                }
            }
            _ => {
                
            }
        }
    }
    
    FZ_.nbm(|| co);
    
    true
}


fn oul(pel: u64) -> Vec<u64> {
    let dh = unsafe { &*(pel as *const tables::Ei) };
    
    
    if &dh.signature != b"RSDT" {
        crate::serial_println!("[ACPI] Invalid RSDT signature");
        return Vec::new();
    }
    
    let fhu = pel + core::mem::size_of::<tables::Ei>() as u64;
    let ebf = (dh.go as usize - core::mem::size_of::<tables::Ei>()) / 4;
    
    let mut gya = Vec::fc(ebf);
    for a in 0..ebf {
        let ag = unsafe { ptr::md((fhu + a as u64 * 4) as *const u32) };
        gya.push(ag as u64);
    }
    
    gya
}


fn ven(qau: u64) -> Vec<u64> {
    let dh = unsafe { &*(qau as *const tables::Ei) };
    
    
    if &dh.signature != b"XSDT" {
        crate::serial_println!("[ACPI] Invalid XSDT signature");
        return Vec::new();
    }
    
    let fhu = qau + core::mem::size_of::<tables::Ei>() as u64;
    let ebf = (dh.go as usize - core::mem::size_of::<tables::Ei>()) / 8;
    
    let mut gya = Vec::fc(ebf);
    for a in 0..ebf {
        let ag = unsafe { ptr::md((fhu + a as u64 * 8) as *const u64) };
        gya.push(ag);
    }
    
    gya
}


pub fn aao() -> usize {
    FZ_.get().map(|a| a.aao).unwrap_or(1)
}


pub fn ljo() -> u64 {
    FZ_.get().map(|a| a.cap).unwrap_or(0xFEE0_0000)
}


pub fn ky() -> bool {
    FZ_.get().is_some()
}


pub fn cbu() -> ! {
    if let Some(co) = FZ_.get() {
        if let Some(ref fadt) = co.fadt {
            fadt::cbu(fadt);
        }
    }
    
    
    crate::serial_println!("[ACPI] Shutdown failed, halting");
    loop {
        unsafe { core::arch::asm!("hlt"); }
    }
}


pub fn fvw() -> bool {
    if let Some(co) = FZ_.get() {
        if let Some(ref fadt) = co.fadt {
            return fadt::wwc(fadt);
        }
    }
    crate::serial_println!("[ACPI] No FADT available for S3 suspend");
    false
}


pub fn jlq() -> ! {
    
    unsafe {
        
        for _ in 0..10000 {
            let status = x86_64::instructions::port::Port::<u8>::new(0x64).read();
            if (status & 0x02) == 0 {
                break;
            }
        }
        
        x86_64::instructions::port::Port::<u8>::new(0x64).write(0xFE);
    }
    
    
    if let Some(co) = FZ_.get() {
        if let Some(ref fadt) = co.fadt {
            fadt::apa(fadt);
        }
    }
    
    
    crate::serial_println!("[ACPI] Reboot failed, triple faulting");
    unsafe {
        
        core::arch::asm!(
            "lidt [{}]",
            "int3",
            in(reg) &[0u64; 2],
            options(jhe)
        );
    }
}
