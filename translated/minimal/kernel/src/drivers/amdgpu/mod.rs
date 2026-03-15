
















pub mod regs;
pub mod dcn;
pub mod compute;
pub mod sdma;
pub mod neural;
pub mod firmware;

use alloc::string::String;
use alloc::format;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicBool, Ordering};
use spin::Mutex;

use crate::pci::{self, S};
use crate::memory;






pub const AKN_: u16 = 0x1002;


pub const CHJ_: &[u16] = &[
    0x7310, 
    0x7312, 
    0x7318, 
    0x7319, 
    0x731A, 
    0x731B, 
    0x731E, 
    0x731F, 
    0x7340, 
    0x7341, 
    0x7347, 
    0x734F, 
];


pub mod family {
    pub const BUZ_:  u32 = 0;
    pub const ARO_:       u32 = 143; 
    pub const ARP_:     u32 = 144; 
    pub const BUX_:       u32 = 141; 
    pub const BUY_:       u32 = 142; 
}


pub mod bar {
    
    pub const BAS_: usize = 0;
    
    pub const BIT_: usize = 2;
    
    pub const Bec: usize = 4;
}


pub mod pcie_cap {
    pub const CIT_: u8 = 0x10;
    pub const Acu: u8 = 0x05;
    pub const Akb: u8 = 0x11;
    pub const EAY_: u8 = 0x01;
}






#[derive(Debug, Clone)]
pub struct Sr {
    
    pub ml: u16,
    pub mx: u16,
    
    pub aq: u8,
    pub de: u8,
    pub gw: u8,
    
    pub afe: u8,
    
    pub kbh: u32,
    
    pub raj: u32,
    
    pub itx: &'static str,
    
    pub cnu: u64,
    
    pub igz: &'static str,
    
    pub uou: u64,
    
    pub lmf: u64,
    
    pub bkm: u64,
    
    pub pyw: u64,
    
    pub igx: u64,
    
    pub ouz: u8,
    
    pub ova: u8,
    
    pub ixr: bool,
    
    pub ixs: bool,
    
    pub cwm: u32,
    
    pub tha: u32,
}

impl Sr {
    
    pub fn beh(&self) -> &'static str {
        match self.mx {
            0x731F => match self.afe {
                0xC1 => "AMD Radeon RX 5700 XT",
                0xC0 | 0xC4 => "AMD Radeon RX 5700",
                0xC2 | 0xC3 => "AMD Radeon RX 5600 XT",
                _ => "AMD Radeon Navi 10",
            },
            0x7340 => "AMD Radeon RX 5500 XT",
            0x7341 => "AMD Radeon Pro W5500",
            0x7312 => "AMD Radeon Pro W5700X",
            _ => "AMD Radeon (Unknown Navi)",
        }
    }

    
    pub fn jwb(&self) -> String {
        if self.cnu >= 1024 * 1024 * 1024 {
            format!("{} GB", self.cnu / (1024 * 1024 * 1024))
        } else if self.cnu >= 1024 * 1024 {
            format!("{} MB", self.cnu / (1024 * 1024))
        } else if self.cnu > 0 {
            format!("{} KB", self.cnu / 1024)
        } else {
            String::from("Unknown")
        }
    }

    
    pub fn ltg(&self) -> String {
        let ig = match self.ouz {
            1 => "2.5 GT/s (Gen1)",
            2 => "5.0 GT/s (Gen2)",
            3 => "8.0 GT/s (Gen3)",
            4 => "16.0 GT/s (Gen4)",
            _ => "Unknown",
        };
        format!("PCIe x{} {}", self.ova, ig)
    }
}


struct Bbp {
    
    jr: bool,
    
    fjv: Option<Sr>,
}

static ATQ_: Mutex<Bbp> = Mutex::new(Bbp {
    jr: false,
    fjv: None,
});

static NG_: AtomicBool = AtomicBool::new(false);









#[inline]
pub unsafe fn wr(hv: u64, l: u32) -> u32 {
    let ag = hv + l as u64;
    core::ptr::read_volatile(ag as *const u32)
}





#[inline]
pub unsafe fn sk(hv: u64, l: u32, bn: u32) {
    let ag = hv + l as u64;
    core::ptr::write_volatile(ag as *mut u32, bn);
}






#[inline]
pub unsafe fn onq(hv: u64, reg: u32) -> u32 {
    
    sk(hv, regs::BAV_, reg);
    
    wr(hv, regs::BAU_)
}





#[inline]
pub unsafe fn uox(hv: u64, reg: u32, bn: u32) {
    sk(hv, regs::BAV_, reg);
    sk(hv, regs::BAU_, bn);
}







fn kpm(ba: &S, fda: usize) -> u64 {
    let doi = 0x10 + (fda as u8 * 4);
    
    
    let evs = pci::aon(ba.aq, ba.de, ba.gw, doi);
    
    
    pci::aso(ba.aq, ba.de, ba.gw, doi, 0xFFFFFFFF);
    
    
    let bky = pci::aon(ba.aq, ba.de, ba.gw, doi);
    
    
    pci::aso(ba.aq, ba.de, ba.gw, doi, evs);
    
    if bky == 0 {
        return 0;
    }
    
    
    if evs & 1 == 0 {
        
        let hs = bky & 0xFFFFFFF0;
        if hs == 0 {
            return 0;
        }
        let aw = (!hs).cn(1) as u64;
        
        
        let gzq = (evs >> 1) & 0x3;
        if gzq == 2 && fda < 5 {
            
            let ikp = doi + 4;
            let lqw = pci::aon(ba.aq, ba.de, ba.gw, ikp);
            pci::aso(ba.aq, ba.de, ba.gw, ikp, 0xFFFFFFFF);
            let lxy = pci::aon(ba.aq, ba.de, ba.gw, ikp);
            pci::aso(ba.aq, ba.de, ba.gw, ikp, lqw);
            
            let nws = ((lxy as u64) << 32) | (hs as u64);
            if nws == 0 {
                return aw;
            }
            (!nws).cn(1)
        } else {
            aw
        }
    } else {
        
        let hs = bky & 0xFFFFFFFC;
        if hs == 0 {
            return 0;
        }
        ((!hs).cn(1) & 0xFFFF) as u64
    }
}






fn vsh(ba: &S) -> (u8, u8) {
    if let Some(vfv) = pci::ebr(ba, pcie_cap::CIT_) {
        
        let hpw = pci::byw(ba.aq, ba.de, ba.gw, vfv + 0x12);
        let ig = (hpw & 0xF) as u8;        
        let z = ((hpw >> 4) & 0x3F) as u8; 
        (ig, z)
    } else {
        (0, 0)
    }
}







fn lxo(hv: u64) -> (u32, u32, u32, u32, u32) {
    let family;
    let nsg;
    let cwm;
    let gim;
    let pyx;

    unsafe {
        
        
        
        let iwi = wr(hv, regs::BWO_);
        crate::serial_println!("[AMDGPU] GC_VERSION raw: {:#010X}", iwi);
        
        
        let vzm = wr(hv, regs::COV_);
        crate::serial_println!("[AMDGPU] RLC_PG_CNTL raw: {:#010X}", vzm);
        
        
        let tai = wr(hv, regs::BWM_);
        crate::serial_println!("[AMDGPU] GC_CAC raw: {:#010X}", tai);
        
        
        let nbv = wr(hv, regs::BLZ_);
        crate::serial_println!("[AMDGPU] CC_GC_SHADER_ARRAY_CONFIG: {:#010X}", nbv);
        
        
        let taj = wr(hv, regs::BWN_);
        crate::serial_println!("[AMDGPU] GC_USER_SHADER_ARRAY_CONFIG: {:#010X}", taj);
        
        
        let hlx = wr(hv, regs::KI_);
        crate::serial_println!("[AMDGPU] GRBM_STATUS: {:#010X}", hlx);
        
        
        let wpw = wr(hv, regs::BAZ_);
        crate::serial_println!("[AMDGPU] MP1 firmware version: {:#010X}", wpw);
        
        
        let weo = wr(hv, regs::CQP_);
        let wep = wr(hv, regs::CQQ_);
        let weq = wr(hv, regs::CQR_);
        crate::serial_println!("[AMDGPU] SCRATCH[0]={:#010X} [1]={:#010X} [7]={:#010X}", 
            weo, wep, weq);
        
        
        let omi = wr(hv, regs::CGA_);
        crate::serial_println!("[AMDGPU] MC_ARB_RAMCFG: {:#010X}", omi);
        pyx = ((omi >> 8) & 0xFF) as u32;
        
        
        
        let nxe = (iwi >> 16) & 0xFF;
        let nxf = (iwi >> 8) & 0xFF;
        
        family = if nxe == 10 && nxf == 1 {
            family::ARO_ 
        } else if nxe == 10 && nxf == 3 {
            family::ARP_ 
        } else {
            family::BUZ_
        };
        
        nsg = iwi; 
        
        
        
        let kmg = nbv & 0xFFFF;
        cwm = if kmg != 0xFFFF && kmg != 0xFFFFFFFF {
            
            let dqa = kmg.ipi();
            40u32.ao(dqa)
        } else {
            0 
        };

        
        let jnt = wr(hv, regs::BMF_);
        crate::serial_println!("[AMDGPU] CG_CLKPIN_CNTL_2: {:#010X}", jnt);
        gim = if jnt != 0 && jnt != 0xFFFFFFFF {
            jnt & 0xFFFF 
        } else {
            0
        };
    }

    (family, nsg, cwm, gim, pyx)
}


fn lxx(hv: u64) -> u64 {
    unsafe {
        
        let jfq = wr(hv, regs::CGC_);
        let lli = wr(hv, regs::CGD_);
        crate::serial_println!("[AMDGPU] MC_VM_FB_LOCATION BASE={:#010X} TOP={:#010X}", 
            jfq, lli);
        
        if lli > jfq && jfq != 0xFFFFFFFF {
            
            let qno = (jfq & 0xFFFF) as u64;
            let xjp = (lli & 0xFFFF) as u64;
            let aw = (xjp - qno + 1) * 1024 * 1024;
            if aw > 0 && aw <= 32 * 1024 * 1024 * 1024 {
                return aw;
            }
        }
        
        
        let ioz = wr(hv, regs::BOR_);
        crate::serial_println!("[AMDGPU] CONFIG_MEMSIZE: {:#010X}", ioz);
        if ioz != 0 && ioz != 0xFFFFFFFF {
            return (ioz as u64) * 1024 * 1024; 
        }
        
        
        0
    }
}


fn xsy(hv: u64) -> &'static str {
    unsafe {
        let omj = wr(hv, regs::CGB_);
        crate::serial_println!("[AMDGPU] MC_SEQ_MISC0: {:#010X}", omj);
        
        
        match (omj >> 28) & 0xF {
            1 => "DDR2",
            2 => "DDR3",
            3 => "GDDR3",
            4 => "GDDR4",
            5 => "GDDR5",
            6 => "HBM",
            7 => "HBM2",
            8 | 9 => "GDDR6",
            _ => {
                
                "GDDR6 (assumed)"
            }
        }
    }
}







pub fn probe() -> Option<S> {
    
    for &rwz in CHJ_ {
        if let Some(ba) = pci::sta(AKN_, rwz) {
            crate::log!("[AMDGPU] Found Navi GPU: {:04X}:{:04X} at {:02X}:{:02X}.{}", 
                ba.ml, ba.mx, ba.aq, ba.de, ba.gw);
            return Some(ba);
        }
    }
    
    
    let cxa = pci::ebq(pci::class::Ji);
    for ba in cxa {
        if ba.ml == AKN_ {
            crate::log!("[AMDGPU] Found AMD Display: {:04X}:{:04X} at {:02X}:{:02X}.{}", 
                ba.ml, ba.mx, ba.aq, ba.de, ba.gw);
            return Some(ba);
        }
    }
    
    None
}









pub fn init() {
    crate::log!("[AMDGPU] ═══════════════════════════════════════════════════");
    crate::log!("[AMDGPU] AMD GPU Driver — Phase 1: PCIe Discovery & MMIO");
    crate::log!("[AMDGPU] ═══════════════════════════════════════════════════");
    
    
    let ba = match probe() {
        Some(bc) => bc,
        None => {
            crate::log!("[AMDGPU] No AMD GPU detected on PCI bus");
            crate::log!("[AMDGPU] (This is normal in VMs without GPU passthrough)");
            
            let cxa = pci::ebq(pci::class::Ji);
            if cxa.is_empty() {
                crate::log!("[AMDGPU] No display controllers found at all");
            } else {
                for bc in &cxa {
                    crate::log!("[AMDGPU] Display: {:04X}:{:04X} {} at {:02X}:{:02X}.{}", 
                        bc.ml, bc.mx, bc.cip(),
                        bc.aq, bc.de, bc.gw);
                }
            }
            return;
        }
    };
    
    
    crate::log!("[AMDGPU] Enabling PCI bus mastering and memory space...");
    pci::fhp(&ba);
    pci::fhq(&ba);
    
    
    let cmd = pci::byw(ba.aq, ba.de, ba.gw, 0x04);
    let status = pci::byw(ba.aq, ba.de, ba.gw, 0x06);
    crate::log!("[AMDGPU] PCI Command: {:#06X}  Status: {:#06X}", cmd, status);
    
    
    crate::log!("[AMDGPU] Detecting BARs...");
    
    let euv = ba.cje(bar::BAS_).unwrap_or(0);
    let bkm = kpm(&ba, bar::BAS_);
    
    let gwk = ba.cje(bar::BIT_).unwrap_or(0);
    let fyl = kpm(&ba, bar::BIT_);
    
    let saf = ba.cje(bar::Bec).unwrap_or(0);
    let nmn = kpm(&ba, bar::Bec);
    
    crate::log!("[AMDGPU] BAR0 (MMIO):     phys={:#014X} size={:#X} ({} KB)", 
        euv, bkm, bkm / 1024);
    crate::log!("[AMDGPU] BAR2 (VRAM):     phys={:#014X} size={:#X} ({} MB)", 
        gwk, fyl, fyl / (1024 * 1024));
    crate::log!("[AMDGPU] BAR4 (Doorbell): phys={:#014X} size={:#X} ({} KB)", 
        saf, nmn, nmn / 1024);
    
    if euv == 0 {
        crate::log!("[AMDGPU] ERROR: BAR0 (MMIO registers) not available!");
        return;
    }
    
    
    let gmb = if bkm > 0 { bkm as usize } else { 512 * 1024 }; 
    crate::log!("[AMDGPU] Mapping MMIO: {:#X} -> {} pages...", euv, gmb / 4096);
    
    let brj = match memory::bki(euv, gmb) {
        Ok(ju) => {
            crate::log!("[AMDGPU] MMIO mapped at virtual {:#014X}", ju);
            ju
        }
        Err(aa) => {
            crate::log!("[AMDGPU] ERROR: Failed to map MMIO: {}", aa);
            return;
        }
    };
    
    
    let (gll, ojk) = vsh(&ba);
    let ixr = pci::ebr(&ba, pcie_cap::Acu).is_some();
    let ixs = pci::ebr(&ba, pcie_cap::Akb).is_some();
    
    crate::log!("[AMDGPU] PCIe: Gen{} x{}  MSI:{}  MSI-X:{}", 
        gll, ojk,
        if ixr { "yes" } else { "no" },
        if ixs { "yes" } else { "no" });
    
    
    crate::log!("[AMDGPU] Reading GPU identity registers...");
    let (kbh, rai, cwm, gim, ydv) = 
        lxo(brj);
    
    let itx = match kbh {
        family::ARO_ => "Navi 10 (RDNA 1)",
        family::ARP_ => "Navi 14 (RDNA 1)",
        family::BUX_ => "Vega 10 (GCN 5)",
        family::BUY_ => "Raven (Vega APU)",
        _ => "Unknown",
    };
    
    
    let cnu = lxx(brj);
    let igz = xsy(brj);
    
    
    let co = Sr {
        ml: ba.ml,
        mx: ba.mx,
        aq: ba.aq,
        de: ba.de,
        gw: ba.gw,
        afe: ba.afe,
        kbh,
        raj: rai,
        itx,
        cnu,
        igz,
        uou: euv,
        lmf: brj,
        bkm: bkm,
        pyw: gwk,
        igx: fyl,
        ouz: gll,
        ova: ojk,
        ixr,
        ixs,
        cwm,
        tha: gim,
    };
    
    
    crate::log!("[AMDGPU] ───────────────────────────────────────────────────");
    crate::log!("[AMDGPU] GPU: {}", co.beh());
    crate::log!("[AMDGPU] PCI: {:04X}:{:04X} rev {:02X} at {:02X}:{:02X}.{}", 
        co.ml, co.mx, co.afe,
        co.aq, co.de, co.gw);
    crate::log!("[AMDGPU] Family: {}", itx);
    crate::log!("[AMDGPU] VRAM: {} ({})", co.jwb(), igz);
    crate::log!("[AMDGPU] {}", co.ltg());
    if cwm > 0 {
        crate::log!("[AMDGPU] Compute Units: {}", cwm);
    }
    if gim > 0 {
        crate::log!("[AMDGPU] GPU Clock: {} MHz", gim);
    }
    crate::log!("[AMDGPU] MMIO: {:#X} ({} KB mapped)", brj, gmb / 1024);
    crate::log!("[AMDGPU] ───────────────────────────────────────────────────");
    crate::log!("[AMDGPU] Phase 1 complete — GPU discovered and identified");
    
    
    let mut g = ATQ_.lock();
    g.jr = true;
    g.fjv = Some(co);
    NG_.store(true, Ordering::SeqCst);
    drop(g);
    
    
    firmware::init(brj);
    
    
    dcn::init(brj);
    
    
    sdma::init(brj);
    
    
    compute::init(brj);
}






pub fn clb() -> bool {
    NG_.load(Ordering::Relaxed)
}


pub fn ani() -> Option<Sr> {
    ATQ_.lock().fjv.clone()
}


pub fn awz() -> String {
    if let Some(co) = ani() {
        format!("{} | {} {} | {} | CU:{}", 
            co.beh(),
            co.jwb(),
            co.igz,
            co.ltg(),
            if co.cwm > 0 { 
                format!("{}", co.cwm) 
            } else { 
                String::from("?") 
            })
    } else {
        String::from("No AMD GPU detected")
    }
}


pub fn zl() -> Vec<String> {
    let mut ak = Vec::new();
    
    if let Some(co) = ani() {
        ak.push(format!("╔══════════════════════════════════════════════╗"));
        ak.push(format!("║        AMD GPU — {}        ║", co.beh()));
        ak.push(format!("╠══════════════════════════════════════════════╣"));
        ak.push(format!("║ PCI ID:    {:04X}:{:04X} rev {:02X}                 ║", 
            co.ml, co.mx, co.afe));
        ak.push(format!("║ Location:  {:02X}:{:02X}.{}                          ║",
            co.aq, co.de, co.gw));
        ak.push(format!("║ Family:    {}               ║", co.itx));
        ak.push(format!("║ VRAM:      {} ({})            ║", co.jwb(), co.igz));
        ak.push(format!("║ PCIe:      {}              ║", co.ltg()));
        if co.cwm > 0 {
            ak.push(format!("║ CUs:       {}                               ║", co.cwm));
        }
        ak.push(format!("║ MMIO:      {:#X} ({} KB)        ║", 
            co.lmf, co.bkm / 1024));
        ak.push(format!("║ VRAM Apt:  {:#X} ({} MB)     ║",
            co.pyw, co.igx / (1024 * 1024)));
        ak.push(format!("║ MSI: {}  MSI-X: {}                        ║",
            if co.ixr { "Yes" } else { "No " },
            if co.ixs { "Yes" } else { "No " }));
        ak.push(format!("╚══════════════════════════════════════════════╝"));
    } else {
        ak.push(String::from("No AMD GPU detected"));
        ak.push(String::from("(GPU passthrough or bare metal required)"));
    }
    
    ak
}
