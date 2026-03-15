
















pub mod regs;

use alloc::string::String;
use alloc::format;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use spin::Mutex;

use crate::pci::{self, S};
use crate::memory;






pub const CIG_: u16 = 0x10DE;



const BGO_: &[(u16, u16, &str)] = &[
    
    (0x0190, 0x019F, "GeForce 8800"),
    
    (0x0400, 0x040F, "GeForce 8600"),
    
    (0x0420, 0x042F, "Quadro NVS 140M / GeForce 8500"),
    
    (0x0600, 0x060F, "GeForce 8800/9800"),
    
    (0x0620, 0x063F, "GeForce 9600"),
    
    (0x0640, 0x065F, "GeForce 9500/9400"),
    
    (0x06E0, 0x06EF, "GeForce G100/G105M"),
    
    (0x05E0, 0x05EF, "GeForce GTX 260/280"),
    
    (0x0840, 0x084F, "GeForce 8200M"),
    (0x0860, 0x086F, "GeForce 8100/8200"),
];


mod bar {
    
    pub const Chc: usize = 0;
    
    pub const Bvq: usize = 1;
    
    pub const Dff: usize = 3;
}






#[derive(Debug, Clone)]
pub struct Ako {
    pub ml: u16,
    pub mx: u16,
    pub afe: u8,
    pub aq: u8,
    pub de: u8,
    pub gw: u8,
    
    pub enl: u8,
    
    pub bxi: u8,
    
    pub beh: &'static str,
    
    pub cnu: u64,
    
    pub hv: u64,
    
    pub bkm: u64,
    
    pub igy: u64,
    
    pub igx: u64,
    
    pub hut: u8,
    
    pub huu: u8,
}

impl Ako {
    pub fn khn(&self) -> &'static str {
        match self.enl {
            0x50 => "G80",
            0x84 => "G84",
            0x86 => "G86",
            0x92 => "G92",
            0x94 => "G94",
            0x96 => "G96",
            0x98 => "G98",
            0xA0 => "GT200",
            _ => "NV50-unknown",
        }
    }
    
    pub fn wvy(&self) -> String {
        format!("{} ({}) | {} MB VRAM | PCIe Gen{} x{}",
            self.beh, self.khn(),
            self.cnu / (1024 * 1024),
            self.hut, self.huu)
    }
}


struct Ash {
    
    vok: u64,
    
    lwi: u64,
    
    lwh: usize,
    
    adi: u32,
    
    channel: u32,
    
    xnn: bool,
}


struct Bno {
    jr: bool,
    fjv: Option<Ako>,
    hje: Option<Ash>,
    
    mth: bool,
}

static Jz: Mutex<Bno> = Mutex::new(Bno {
    jr: false,
    fjv: None,
    hje: None,
    mth: false,
});

static NG_: AtomicBool = AtomicBool::new(false);
static AKC_: AtomicBool = AtomicBool::new(false);

static CGO_: AtomicU64 = AtomicU64::new(0);

static AJS_: AtomicU64 = AtomicU64::new(0);






#[inline]
unsafe fn dth(ar: u64, l: u32) -> u32 {
    core::ptr::read_volatile((ar + l as u64) as *const u32)
}


#[inline]
unsafe fn cgf(ar: u64, l: u32, ap: u32) {
    core::ptr::write_volatile((ar + l as u64) as *mut u32, ap);
}


unsafe fn zcw(ar: u64, l: u32, hs: u32, bn: u32, mkv: u32) -> bool {
    for _ in 0..mkv {
        if dth(ar, l) & hs == bn {
            return true;
        }
        
        core::arch::asm!("out 0x80, al", in("al") 0u8, options(nostack, preserves_flags));
    }
    false
}






fn gpv() -> Option<S> {
    let ik = pci::ebq(pci::class::Ji);
    for ba in ik {
        if ba.ml != CIG_ {
            continue;
        }
        
        for &(hh, gd, blu) in BGO_ {
            if ba.mx >= hh && ba.mx <= gd {
                return Some(ba);
            }
        }
        
        crate::serial_println!("[NVIDIA] Unrecognized NVIDIA GPU: {:04X}:{:04X}",
            ba.ml, ba.mx);
    }
    None
}


fn thd(mx: u16) -> &'static str {
    for &(hh, gd, j) in BGO_ {
        if mx >= hh && mx <= gd {
            return j;
        }
    }
    "NVIDIA (Unknown)"
}






unsafe fn lxo(mmio: u64) -> (u8, u8) {
    let ked = dth(mmio, regs::CJZ_);
    let enl = ((ked >> 20) & 0xFF) as u8;
    let bxi = (ked & 0xFF) as u8;
    crate::serial_println!("[NVIDIA] PMC_BOOT_0 = {:#010X} → chipset={:#04X} stepping={:#04X}",
        ked, enl, bxi);
    (enl, bxi)
}


unsafe fn lxx(mmio: u64) -> u64 {
    let qxw = dth(mmio, regs::CIY_);
    let ncc = dth(mmio, regs::CIZ_);
    crate::serial_println!("[NVIDIA] PFB_CFG0={:#010X} PFB_CFG1={:#010X}", qxw, ncc);
    
    
    
    
    let aga = match ncc & 0xFFF {
        e if e > 0 => e as u64,
        _ => {
            
            128 
        }
    };
    
    let xsx = aga * 1024 * 1024;
    crate::serial_println!("[NVIDIA] VRAM: {} MB", aga);
    xsx
}


fn vsg(ba: &S) -> (u8, u8) {
    if let Some(kgl) = pci::ebr(ba, 0x10) {
        let hpw = pci::byw(ba.aq, ba.de, ba.gw, 
            kgl as u8 + 0x12);
        let ig = (hpw & 0xF) as u8;
        let z = ((hpw >> 4) & 0x3F) as u8;
        (ig, z)
    } else {
        (1, 16) 
    }
}


unsafe fn thb(mmio: u64) -> bool {
    crate::serial_println!("[NVIDIA] Initializing GPU engines...");
    
    
    let aiy = dth(mmio, regs::AGN_);
    crate::serial_println!("[NVIDIA] Current PMC_ENABLE = {:#010X}", aiy);
    
    
    let ust = aiy | regs::CKC_ | regs::CKD_ 
                           | regs::CKB_ | regs::CKA_;
    cgf(mmio, regs::AGN_, ust);
    
    
    let xrf = dth(mmio, regs::AGN_);
    crate::serial_println!("[NVIDIA] PMC_ENABLE after write = {:#010X}", xrf);
    
    
    cgf(mmio, regs::BDH_, 0xFFFFFFFF);
    cgf(mmio, regs::CJG_, 0xFFFFFFFF);
    cgf(mmio, regs::CJJ_, 0xC0000000);
    cgf(mmio, regs::CJB_, 0xFFFFFFFF);
    
    
    cgf(mmio, regs::CKE_, 0);
    cgf(mmio, regs::CJH_, 0);
    cgf(mmio, regs::CJC_, 0);
    
    
    cgf(mmio, regs::CJA_, 1);
    
    
    cgf(mmio, regs::CLC_, 0x00000008);
    cgf(mmio, regs::CLB_, 0x00000003);
    
    
    let edo = dth(mmio, regs::BDH_);
    let vhb = dth(mmio, regs::CJI_);
    crate::serial_println!("[NVIDIA] PMC_INTR = {:#010X}, PGRAPH status = {:#010X}", 
        edo, vhb);
    
    true
}


unsafe fn wku(mmio: u64, igy: u64) -> Option<Ash> {
    crate::serial_println!("[NVIDIA] Setting up FIFO channel 0...");
    
    
    
    
    let lwh: usize = 4096;     
    let lwj: u64 = 0;   
    
    
    let lwi = igy + lwj;
    
    
    let ptr = lwi as *mut u8;
    for a in 0..lwh {
        core::ptr::write_volatile(ptr.add(a), 0);
    }
    
    
    
    let ev = dth(mmio, regs::BDA_);
    cgf(mmio, regs::BDA_, ev | 1); 
    
    
    let dma = dth(mmio, regs::BCZ_);
    cgf(mmio, regs::BCZ_, dma | 1);
    
    
    
    let mom = mmio + regs::AGF_ as u64;
    
    
    cgf(mmio, regs::AGF_ + regs::CIF_, 0);
    cgf(mmio, regs::AGF_ + regs::CIE_, 0);
    
    crate::serial_println!("[NVIDIA] FIFO channel 0 configured (pushbuf at VRAM offset {:#X})", 
        lwj);
    
    Some(Ash {
        vok: lwj,
        lwi,
        lwh,
        adi: 0,
        channel: 0,
        xnn: false,
    })
}








unsafe fn znw(mmio: u64, hiz: u64, z: u32, ac: u32, jb: u32) {
    crate::serial_println!("[NVIDIA] Setting up 2D surface: {}x{} pitch={} fb_phys={:#X}",
        z, ac, jb, hiz);
    
    
    
    
    
    
    
    
    
    
    
}





pub fn yeb(b: u32, c: u32, d: u32, i: u32, s: u32) {
    let aof = AJS_.load(Ordering::Relaxed);
    if aof == 0 {
        return;
    }
    
    
    let (gz, kc) = crate::framebuffer::yn();
    if gz == 0 || kc == 0 || b >= gz || c >= kc {
        return;
    }
    
    let fza = (b + d).v(gz);
    let dno = (c + i).v(kc);
    let jb = gz; 
    
    
    
    unsafe {
        let ar = aof as *mut u32;
        for br in c..dno {
            let afg = (br * jb + b) as isize;
            let cml = (fza - b) as usize;
            let mav = ar.l(afg);
            
            
            for bj in 0..cml {
                core::ptr::write_volatile(mav.add(bj), s);
            }
        }
    }
}


pub fn yea(blg: u32, bih: u32, buc: u32, bqg: u32, d: u32, i: u32) {
    let aof = AJS_.load(Ordering::Relaxed);
    if aof == 0 {
        return;
    }
    
    let (gz, kc) = crate::framebuffer::yn();
    if gz == 0 || kc == 0 {
        return;
    }
    
    let jb = gz;
    
    unsafe {
        let ar = aof as *mut u32;
        
        
        if bqg > bih || (bqg == bih && buc > blg) {
            
            for br in (0..i).vv() {
                let cq = bih + br;
                let bg = bqg + br;
                if cq >= kc || bg >= kc { continue; }
                
                for bj in (0..d).vv() {
                    let cr = blg + bj;
                    let dx = buc + bj;
                    if cr >= gz || dx >= gz { continue; }
                    
                    let ap = core::ptr::read_volatile(ar.l((cq * jb + cr) as isize));
                    core::ptr::write_volatile(ar.l((bg * jb + dx) as isize), ap);
                }
            }
        } else {
            
            for br in 0..i {
                let cq = bih + br;
                let bg = bqg + br;
                if cq >= kc || bg >= kc { continue; }
                
                for bj in 0..d {
                    let cr = blg + bj;
                    let dx = buc + bj;
                    if cr >= gz || dx >= gz { continue; }
                    
                    let ap = core::ptr::read_volatile(ar.l((cq * jb + cr) as isize));
                    core::ptr::write_volatile(ar.l((bg * jb + dx) as isize), ap);
                }
            }
        }
    }
}






pub fn init() {
    crate::serial_println!("[NVIDIA] ═══════════════════════════════════════════════════");
    crate::serial_println!("[NVIDIA] NVIDIA GPU Driver — NV50 (Tesla) Family");
    crate::serial_println!("[NVIDIA] ═══════════════════════════════════════════════════");
    
    
    let ba = match gpv() {
        Some(bc) => bc,
        None => {
            crate::serial_println!("[NVIDIA] No supported NVIDIA GPU found on PCI bus");
            
            let cxa = pci::ebq(pci::class::Ji);
            if cxa.is_empty() {
                crate::serial_println!("[NVIDIA] No display controllers found at all");
            } else {
                for bc in &cxa {
                    crate::serial_println!("[NVIDIA] Display: {:04X}:{:04X} at {:02X}:{:02X}.{}",
                        bc.ml, bc.mx, bc.aq, bc.de, bc.gw);
                }
            }
            return;
        }
    };
    
    crate::serial_println!("[NVIDIA] Found: {:04X}:{:04X} rev {:02X} at {:02X}:{:02X}.{}",
        ba.ml, ba.mx, ba.afe, ba.aq, ba.de, ba.gw);
    
    
    pci::fhp(&ba);
    pci::fhq(&ba);
    
    
    let euv = match ba.cje(bar::Chc) {
        Some(ag) if ag > 0 => ag,
        _ => {
            crate::serial_println!("[NVIDIA] ERROR: BAR0 (MMIO) not available");
            return;
        }
    };
    
    let bkm = 16 * 1024 * 1024; 
    crate::serial_println!("[NVIDIA] BAR0 (MMIO): phys={:#010X} size={}MB", euv, bkm / (1024*1024));
    
    let brj = match memory::bki(euv, bkm) {
        Ok(p) => p,
        Err(aa) => {
            crate::serial_println!("[NVIDIA] ERROR: Failed to map BAR0: {}", aa);
            return;
        }
    };
    crate::serial_println!("[NVIDIA] BAR0 mapped at virt={:#014X}", brj);
    CGO_.store(brj, Ordering::SeqCst);
    
    
    let gwk = ba.cje(bar::Bvq).unwrap_or(0);
    let mut fbc: u64 = 0;
    let mut fyl: u64 = 0;
    
    if gwk > 0 {
        
        let yfk = ba.bar[bar::Bvq];
        
        fyl = 256 * 1024 * 1024; 
        
        crate::serial_println!("[NVIDIA] BAR1 (VRAM): phys={:#010X} aperture={}MB", 
            gwk, fyl / (1024*1024));
        
        
        
        let gmb = 16 * 1024 * 1024;
        match memory::bki(gwk, gmb) {
            Ok(p) => {
                fbc = p;
                crate::serial_println!("[NVIDIA] BAR1 mapped at virt={:#014X} ({}MB)", p, gmb / (1024*1024));
                AJS_.store(fbc, Ordering::SeqCst);
            }
            Err(aa) => {
                crate::serial_println!("[NVIDIA] WARNING: Failed to map BAR1 VRAM: {}", aa);
                
            }
        }
    }
    
    
    let (enl, bxi) = unsafe { lxo(brj) };
    
    
    let cnu = unsafe { lxx(brj) };
    
    
    let (hut, huu) = vsg(&ba);
    
    
    let sls = unsafe { thb(brj) };
    if !sls {
        crate::serial_println!("[NVIDIA] WARNING: Engine init had issues, continuing anyway");
    }
    
    
    let hje = if fbc > 0 {
        unsafe { wku(brj, fbc) }
    } else {
        None
    };
    
    let jyw = hje.is_some() && fbc > 0;
    
    
    let co = Ako {
        ml: ba.ml,
        mx: ba.mx,
        afe: ba.afe,
        aq: ba.aq,
        de: ba.de,
        gw: ba.gw,
        enl,
        bxi,
        beh: thd(ba.mx),
        cnu,
        hv: brj,
        bkm: bkm as u64,
        igy: fbc,
        igx: fyl,
        hut,
        huu,
    };
    
    
    crate::serial_println!("[NVIDIA] ───────────────────────────────────────────────────");
    crate::serial_println!("[NVIDIA] GPU: {} ({})", co.beh, co.khn());
    crate::serial_println!("[NVIDIA] PCI: {:04X}:{:04X} rev {:02X} at {:02X}:{:02X}.{}", 
        co.ml, co.mx, co.afe,
        co.aq, co.de, co.gw);
    crate::serial_println!("[NVIDIA] Chipset: {:#04X} stepping {:#04X}", enl, bxi);
    crate::serial_println!("[NVIDIA] VRAM: {} MB", co.cnu / (1024 * 1024));
    crate::serial_println!("[NVIDIA] PCIe: Gen{} x{}", hut, huu);
    crate::serial_println!("[NVIDIA] MMIO: {:#X} ({}MB)", brj, bkm / (1024*1024));
    if fbc > 0 {
        crate::serial_println!("[NVIDIA] VRAM aperture: {:#X}", fbc);
    }
    crate::serial_println!("[NVIDIA] 2D Acceleration: {}", if jyw { "READY" } else { "UNAVAILABLE" });
    crate::serial_println!("[NVIDIA] ───────────────────────────────────────────────────");
    
    
    let mut g = Jz.lock();
    g.jr = true;
    g.fjv = Some(co);
    g.hje = hje;
    g.mth = jyw;
    NG_.store(true, Ordering::SeqCst);
    AKC_.store(jyw, Ordering::SeqCst);
    drop(g);
}


pub fn clb() -> bool {
    NG_.load(Ordering::Relaxed)
}


pub fn twp() -> bool {
    AKC_.load(Ordering::Relaxed)
}


pub fn ani() -> Option<Ako> {
    Jz.lock().fjv.clone()
}


pub fn awz() -> String {
    if let Some(co) = ani() {
        co.wvy()
    } else {
        String::from("No NVIDIA GPU detected")
    }
}
