























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


static CEC_: AtomicBool = AtomicBool::new(false);


static CJZ_: AtomicU64 = AtomicU64::new(0);


static CXD_: AtomicU64 = AtomicU64::new(0);







pub mod hcr {
    
    pub const Ark: u64      = 1 << 0;
    
    pub const Aqa: u64    = 1 << 1;
    
    pub const Ajw: u64     = 1 << 3;
    
    pub const Ali: u64     = 1 << 4;
    
    pub const Agv: u64     = 1 << 5;
    
    pub const Aqp: u64     = 1 << 13;
    
    pub const Aqo: u64     = 1 << 14;
    
    pub const Bea: u64     = 1 << 26;
    
    pub const Aqn: u64     = 1 << 19;
    
    pub const Adh: u64      = 1 << 31;
    
    pub const Asp: u64     = 1 << 40;
    
    pub const Aso: u64     = 1 << 41;
}


pub mod esr_class {
    
    pub const ARV_: u32 = 0b100100;
    
    pub const Zp: u32 = 0b010110;
    
    pub const Aei: u32 = 0b010111;
    
    pub const BDE_: u32 = 0b011000;
    
    pub const Agi: u32 = 0b000001;
    
    pub const AZC_: u32 = 0b100000;
}


#[repr(C)]
#[derive(Debug, Clone, Default)]
pub struct Ss {
    
    pub x: [u64; 31],
    
    pub sp_el1: u64,
    
    pub elr_el1: u64,
    
    pub spsr_el1: u64,
    
    pub sctlr_el1: u64,
    
    pub ttbr0_el1: u64,
    
    pub ttbr1_el1: u64,
    
    pub tcr_el1: u64,
    
    pub mair_el1: u64,
    
    pub vbar_el1: u64,
    
    pub esr_el2: u64,
    
    pub far_el2: u64,
    
    pub hpfar_el2: u64,
}


#[derive(Debug, Clone)]
pub struct Ic {
    
    pub guest_entry: u64,
    
    pub guest_dtb: u64,
    
    pub guest_ram_base: u64,
    
    pub guest_ram_size: u64,
    
    pub trapped_mmio: Vec<(u64, u64)>,
    
    pub trap_smc: bool,
    
    pub trap_wfi: bool,
}

impl Default for Ic {
    fn default() -> Self {
        Self {
            guest_entry: 0,
            guest_dtb: 0,
            guest_ram_base: 0x4000_0000,
            guest_ram_size: 512 * 1024 * 1024,
            trapped_mmio: Vec::new(),
            trap_smc: true,
            trap_wfi: false,
        }
    }
}






pub fn cll() -> bool {
    #[cfg(target_arch = "aarch64")]
    {
        let el: u64;
        unsafe { core::arch::asm!("mrs {}, CurrentEL", out(reg) el) };
        (el >> 2) & 3 == 2
    }
    #[cfg(not(target_arch = "aarch64"))]
    { false }
}


pub fn is_active() -> bool {
    CEC_.load(Ordering::Relaxed)
}


pub fn ioa() -> u64 {
    CJZ_.load(Ordering::Relaxed)
}


pub fn jgr() -> u64 {
    CXD_.load(Ordering::Relaxed)
}


pub fn hnf(config: &Ic) -> u64 {
    let mut hcr: u64 = 0;

    
    hcr |= hcr::Adh;   
    hcr |= hcr::Ark;   

    
    hcr |= hcr::Ali;  
    hcr |= hcr::Ajw;  
    hcr |= hcr::Agv;  

    
    if config.trap_smc {
        hcr |= hcr::Aqn;
    }

    
    if config.trap_wfi {
        hcr |= hcr::Aqp;
        hcr |= hcr::Aqo;
    }

    
    hcr |= hcr::Aqa;

    hcr
}


pub fn mck() -> String {
    let mut out = String::new();

    out.push_str("\x01C== TrustOS EL2 Hypervisor — MMIO Spy Report ==\x01W\n\n");

    if !is_active() {
        out.push_str("Hypervisor is NOT active.\n");
        out.push_str("Use 'hv start <kernel>' to boot a guest OS under EL2 surveillance.\n");
        return out;
    }

    out.push_str(&format!("Status: \x01GACTIVE\x01W (running at EL2)\n"));
    out.push_str(&format!("MMIO traps: {}\n", ioa()));
    out.push_str(&format!("SMC intercepts: {}\n", jgr()));
    out.push_str("\n");

    
    let events = mmio_spy::iyt(50);
    if events.is_empty() {
        out.push_str("No MMIO events captured yet.\n");
    } else {
        out.push_str(&format!("{:<14} {:<6} {:<14} {:<14} {}\n",
            "ADDRESS", "R/W", "VALUE", "SIZE", "DEVICE"));
        out.push_str(&format!("{}\n", "-".repeat(70)));

        for bsj in &events {
            let lk = if bsj.is_write { "WRITE" } else { "READ" };
            out.push_str(&format!("0x{:010X}  {:<6} 0x{:010X}  {} bytes   {}\n",
                bsj.ipa, lk, bsj.value, bsj.access_size, bsj.device_name));
        }

        out.push_str(&format!("\nTotal: {} events captured\n", events.len()));
    }

    
    let jgp = mmio_spy::gqq(20);
    if !jgp.is_empty() {
        out.push_str("\n\x01Y--- SMC (Secure Monitor Calls) ---\x01W\n");
        out.push_str(&format!("{:<14} {:<14} {:<14} {}\n",
            "FID", "X1", "X2", "MEANING"));
        out.push_str(&format!("{}\n", "-".repeat(60)));
        for alb in &jgp {
            out.push_str(&format!("0x{:08X}   0x{:08X}   0x{:08X}   {}\n",
                alb.fid, alb.x1, alb.x2, alb.smc_type_name));
        }
    }

    out
}
