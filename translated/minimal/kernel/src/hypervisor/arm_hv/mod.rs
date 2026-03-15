























pub mod stage2;
pub mod trap_handler;
pub mod mmio_spy;
pub mod vgic;
pub mod el2_entry;
pub mod guest_loader;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};


static CAR_: AtomicBool = AtomicBool::new(false);


static CGP_: AtomicU64 = AtomicU64::new(0);


static CTM_: AtomicU64 = AtomicU64::new(0);







pub mod hcr {
    
    pub const Cpb: u64      = 1 << 0;
    
    pub const Cmm: u64    = 1 << 1;
    
    pub const Cdc: u64     = 1 << 3;
    
    pub const Cfj: u64     = 1 << 4;
    
    pub const Bxi: u64     = 1 << 5;
    
    pub const Cnz: u64     = 1 << 13;
    
    pub const Cny: u64     = 1 << 14;
    
    pub const Djq: u64     = 1 << 26;
    
    pub const Cnx: u64     = 1 << 19;
    
    pub const Bqh: u64      = 1 << 31;
    
    pub const Cre: u64     = 1 << 40;
    
    pub const Crd: u64     = 1 << 41;
}


pub mod esr_class {
    
    pub const APT_: u32 = 0b100100;
    
    pub const Bin: u32 = 0b010110;
    
    pub const Brz: u32 = 0b010111;
    
    pub const BBB_: u32 = 0b011000;
    
    pub const Bwg: u32 = 0b000001;
    
    pub const AXA_: u32 = 0b100000;
}


#[repr(C)]
#[derive(Debug, Clone, Default)]
pub struct Atg {
    
    pub b: [u64; 31],
    
    pub wql: u64,
    
    pub bzm: u64,
    
    pub mgy: u64,
    
    pub wfb: u64,
    
    pub xnc: u64,
    
    pub xnd: u64,
    
    pub xbl: u64,
    
    pub ujj: u64,
    
    pub xqt: u64,
    
    pub snk: u64,
    
    pub sra: u64,
    
    pub tql: u64,
}


#[derive(Debug, Clone)]
pub struct Sw {
    
    pub hma: u64,
    
    pub ixj: u64,
    
    pub hmd: u64,
    
    pub hme: u64,
    
    pub iew: Vec<(u64, u64)>,
    
    pub fxj: bool,
    
    pub fxk: bool,
}

impl Default for Sw {
    fn default() -> Self {
        Self {
            hma: 0,
            ixj: 0,
            hmd: 0x4000_0000,
            hme: 512 * 1024 * 1024,
            iew: Vec::new(),
            fxj: true,
            fxk: false,
        }
    }
}






pub fn fma() -> bool {
    #[cfg(target_arch = "aarch64")]
    {
        let ij: u64;
        unsafe { core::arch::asm!("mrs {}, CurrentEL", bd(reg) ij) };
        (ij >> 2) & 3 == 2
    }
    #[cfg(not(target_arch = "aarch64"))]
    { false }
}


pub fn rl() -> bool {
    CAR_.load(Ordering::Relaxed)
}


pub fn onr() -> u64 {
    CGP_.load(Ordering::Relaxed)
}


pub fn plm() -> u64 {
    CTM_.load(Ordering::Relaxed)
}


pub fn nfh(config: &Sw) -> u64 {
    let mut hcr: u64 = 0;

    
    hcr |= hcr::Bqh;   
    hcr |= hcr::Cpb;   

    
    hcr |= hcr::Cfj;  
    hcr |= hcr::Cdc;  
    hcr |= hcr::Bxi;  

    
    if config.fxj {
        hcr |= hcr::Cnx;
    }

    
    if config.fxk {
        hcr |= hcr::Cnz;
        hcr |= hcr::Cny;
    }

    
    hcr |= hcr::Cmm;

    hcr
}


pub fn tcn() -> String {
    let mut bd = String::new();

    bd.t("\x01C== TrustOS EL2 Hypervisor — MMIO Spy Report ==\x01W\n\n");

    if !rl() {
        bd.t("Hypervisor is NOT active.\n");
        bd.t("Use 'hv start <kernel>' to boot a guest OS under EL2 surveillance.\n");
        return bd;
    }

    bd.t(&format!("Status: \x01GACTIVE\x01W (running at EL2)\n"));
    bd.t(&format!("MMIO traps: {}\n", onr()));
    bd.t(&format!("SMC intercepts: {}\n", plm()));
    bd.t("\n");

    
    let events = mmio_spy::paq(50);
    if events.is_empty() {
        bd.t("No MMIO events captured yet.\n");
    } else {
        bd.t(&format!("{:<14} {:<6} {:<14} {:<14} {}\n",
            "ADDRESS", "R/W", "VALUE", "SIZE", "DEVICE"));
        bd.t(&format!("{}\n", "-".afd(70)));

        for ebi in &events {
            let yq = if ebi.rm { "WRITE" } else { "READ" };
            bd.t(&format!("0x{:010X}  {:<6} 0x{:010X}  {} bytes   {}\n",
                ebi.akh, yq, ebi.bn, ebi.cct, ebi.dgg));
        }

        bd.t(&format!("\nTotal: {} events captured\n", events.len()));
    }

    
    let plk = mmio_spy::lyf(20);
    if !plk.is_empty() {
        bd.t("\n\x01Y--- SMC (Secure Monitor Calls) ---\x01W\n");
        bd.t(&format!("{:<14} {:<14} {:<14} {}\n",
            "FID", "X1", "X2", "MEANING"));
        bd.t(&format!("{}\n", "-".afd(60)));
        for bto in &plk {
            bd.t(&format!("0x{:08X}   0x{:08X}   0x{:08X}   {}\n",
                bto.aos, bto.dn, bto.hy, bto.jqo));
        }
    }

    bd
}
