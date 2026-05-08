
















pub mod regs;

use alloc::string::String;
use alloc::format;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use spin::Mutex;

use crate::pci::{self, L};
use crate::memory;






pub const CLP_: u16 = 0x10DE;



const BIS_: &[(u16, u16, &str)] = &[
    
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
    
    pub const Amp: usize = 0;
    
    pub const Agb: usize = 1;
    
    pub const Bay: usize = 3;
}






#[derive(Debug, Clone)]
pub struct Pr {
    pub vendor_id: u16,
    pub device_id: u16,
    pub revision: u8,
    pub bus: u8,
    pub device: u8,
    pub function: u8,
    
    pub chipset_id: u8,
    
    pub stepping: u8,
    
    pub gpu_name: &'static str,
    
    pub vram_size: u64,
    
    pub mmio_base: u64,
    
    pub mmio_size: u64,
    
    pub vram_base: u64,
    
    pub vram_aperture_size: u64,
    
    pub pcie_gen: u8,
    
    pub pcie_width: u8,
}

impl Pr {
    pub fn chipset_name(&self) -> &'static str {
        match self.chipset_id {
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
    
    pub fn summary_string(&self) -> String {
        format!("{} ({}) | {} MB VRAM | PCIe Gen{} x{}",
            self.gpu_name, self.chipset_name(),
            self.vram_size / (1024 * 1024),
            self.pcie_gen, self.pcie_width)
    }
}


struct Sg {
    
    pushbuf_phys: u64,
    
    gpb: u64,
    
    gpa: usize,
    
    os: u32,
    
    channel: u32,
    
    twod_bound: bool,
}


struct Abv {
    initialized: bool,
    gpu_info: Option<Pr>,
    fifo: Option<Sg>,
    
    accel_2d_ready: bool,
}

static Dz: Mutex<Abv> = Mutex::new(Abv {
    initialized: false,
    gpu_info: None,
    fifo: None,
    accel_2d_ready: false,
});

static OF_: AtomicBool = AtomicBool::new(false);
static ALX_: AtomicBool = AtomicBool::new(false);

static CJY_: AtomicU64 = AtomicU64::new(0);

static ALN_: AtomicU64 = AtomicU64::new(0);






#[inline]
unsafe fn bnp(base: u64, offset: u32) -> u32 {
    core::ptr::read_volatile((base + offset as u64) as *const u32)
}


#[inline]
unsafe fn arq(base: u64, offset: u32, val: u32) {
    core::ptr::write_volatile((base + offset as u64) as *mut u32, val);
}


unsafe fn qpc(base: u64, offset: u32, mask: u32, value: u32, gyv: u32) -> bool {
    for _ in 0..gyv {
        if bnp(base, offset) & mask == value {
            return true;
        }
        
        core::arch::asm!("out 0x80, al", in("al") 0u8, options(nostack, preserves_flags));
    }
    false
}






fn ccv() -> Option<L> {
    let devices = pci::bsp(pci::class::Du);
    for s in devices {
        if s.vendor_id != CLP_ {
            continue;
        }
        
        for &(lo, hi, _name) in BIS_ {
            if s.device_id >= lo && s.device_id <= hi {
                return Some(s);
            }
        }
        
        crate::serial_println!("[NVIDIA] Unrecognized NVIDIA GPU: {:04X}:{:04X}",
            s.vendor_id, s.device_id);
    }
    None
}


fn mfv(device_id: u16) -> &'static str {
    for &(lo, hi, name) in BIS_ {
        if device_id >= lo && device_id <= hi {
            return name;
        }
    }
    "NVIDIA (Unknown)"
}






unsafe fn gqe(mmio: u64) -> (u8, u8) {
    let fjo = bnp(mmio, regs::CNI_);
    let chipset_id = ((fjo >> 20) & 0xFF) as u8;
    let stepping = (fjo & 0xFF) as u8;
    crate::serial_println!("[NVIDIA] PMC_BOOT_0 = {:#010X} → chipset={:#04X} stepping={:#04X}",
        fjo, chipset_id, stepping);
    (chipset_id, stepping)
}


unsafe fn gqj(mmio: u64) -> u64 {
    let kig = bnp(mmio, regs::CMH_);
    let hkd = bnp(mmio, regs::CMI_);
    crate::serial_println!("[NVIDIA] PFB_CFG0={:#010X} PFB_CFG1={:#010X}", kig, hkd);
    
    
    
    
    let size_mb = match hkd & 0xFFF {
        j if j > 0 => j as u64,
        _ => {
            
            128 
        }
    };
    
    let pte = size_mb * 1024 * 1024;
    crate::serial_println!("[NVIDIA] VRAM: {} MB", size_mb);
    pte
}


fn ocw(s: &L) -> (u8, u8) {
    if let Some(cap_offset) = pci::bsq(s, 0x10) {
        let cbl = pci::vf(s.bus, s.device, s.function, 
            cap_offset as u8 + 0x12);
        let speed = (cbl & 0xF) as u8;
        let width = ((cbl >> 4) & 0x3F) as u8;
        (speed, width)
    } else {
        (1, 16) 
    }
}


unsafe fn mft(mmio: u64) -> bool {
    crate::serial_println!("[NVIDIA] Initializing GPU engines...");
    
    
    let enable = bnp(mmio, regs::AIH_);
    crate::serial_println!("[NVIDIA] Current PMC_ENABLE = {:#010X}", enable);
    
    
    let niy = enable | regs::CNL_ | regs::CNM_ 
                           | regs::CNK_ | regs::CNJ_;
    arq(mmio, regs::AIH_, niy);
    
    
    let prp = bnp(mmio, regs::AIH_);
    crate::serial_println!("[NVIDIA] PMC_ENABLE after write = {:#010X}", prp);
    
    
    arq(mmio, regs::BFK_, 0xFFFFFFFF);
    arq(mmio, regs::CMP_, 0xFFFFFFFF);
    arq(mmio, regs::CMS_, 0xC0000000);
    arq(mmio, regs::CMK_, 0xFFFFFFFF);
    
    
    arq(mmio, regs::CNN_, 0);
    arq(mmio, regs::CMQ_, 0);
    arq(mmio, regs::CML_, 0);
    
    
    arq(mmio, regs::CMJ_, 1);
    
    
    arq(mmio, regs::COL_, 0x00000008);
    arq(mmio, regs::COK_, 0x00000003);
    
    
    let btn = bnp(mmio, regs::BFK_);
    let ntx = bnp(mmio, regs::CMR_);
    crate::serial_println!("[NVIDIA] PMC_INTR = {:#010X}, PGRAPH status = {:#010X}", 
        btn, ntx);
    
    true
}


unsafe fn oqh(mmio: u64, vram_base: u64) -> Option<Sg> {
    crate::serial_println!("[NVIDIA] Setting up FIFO channel 0...");
    
    
    
    
    let gpa: usize = 4096;     
    let gpc: u64 = 0;   
    
    
    let gpb = vram_base + gpc;
    
    
    let ptr = gpb as *mut u8;
    for i in 0..gpa {
        core::ptr::write_volatile(ptr.add(i), 0);
    }
    
    
    
    let mode = bnp(mmio, regs::BFD_);
    arq(mmio, regs::BFD_, mode | 1); 
    
    
    let dma = bnp(mmio, regs::BFC_);
    arq(mmio, regs::BFC_, dma | 1);
    
    
    
    let hax = mmio + regs::AHZ_ as u64;
    
    
    arq(mmio, regs::AHZ_ + regs::CLO_, 0);
    arq(mmio, regs::AHZ_ + regs::CLN_, 0);
    
    crate::serial_println!("[NVIDIA] FIFO channel 0 configured (pushbuf at VRAM offset {:#X})", 
        gpc);
    
    Some(Sg {
        pushbuf_phys: gpc,
        gpb,
        gpa,
        os: 0,
        channel: 0,
        twod_bound: false,
    })
}








unsafe fn qws(mmio: u64, dpj: u64, width: u32, height: u32, pitch: u32) {
    crate::serial_println!("[NVIDIA] Setting up 2D surface: {}x{} pitch={} fb_phys={:#X}",
        width, height, pitch, dpj);
    
    
    
    
    
    
    
    
    
    
    
}





pub fn pxm(x: u32, y: u32, w: u32, h: u32, color: u32) {
    let vram = ALN_.load(Ordering::Relaxed);
    if vram == 0 {
        return;
    }
    
    
    let (fb_w, fb_h) = crate::framebuffer::kv();
    if fb_w == 0 || fb_h == 0 || x >= fb_w || y >= fb_h {
        return;
    }
    
    let csz = (x + w).min(fb_w);
    let bkg = (y + h).min(fb_h);
    let pitch = fb_w; 
    
    
    
    unsafe {
        let base = vram as *mut u32;
        for row in y..bkg {
            let pq = (row * pitch + x) as isize;
            let auv = (csz - x) as usize;
            let grz = base.offset(pq);
            
            
            for col in 0..auv {
                core::ptr::write_volatile(grz.add(col), color);
            }
        }
    }
}


pub fn pxl(ahc: u32, aft: u32, dst_x: u32, dst_y: u32, w: u32, h: u32) {
    let vram = ALN_.load(Ordering::Relaxed);
    if vram == 0 {
        return;
    }
    
    let (fb_w, fb_h) = crate::framebuffer::kv();
    if fb_w == 0 || fb_h == 0 {
        return;
    }
    
    let pitch = fb_w;
    
    unsafe {
        let base = vram as *mut u32;
        
        
        if dst_y > aft || (dst_y == aft && dst_x > ahc) {
            
            for row in (0..h).rev() {
                let ak = aft + row;
                let ad = dst_y + row;
                if ak >= fb_h || ad >= fb_h { continue; }
                
                for col in (0..w).rev() {
                    let am = ahc + col;
                    let dx = dst_x + col;
                    if am >= fb_w || dx >= fb_w { continue; }
                    
                    let val = core::ptr::read_volatile(base.offset((ak * pitch + am) as isize));
                    core::ptr::write_volatile(base.offset((ad * pitch + dx) as isize), val);
                }
            }
        } else {
            
            for row in 0..h {
                let ak = aft + row;
                let ad = dst_y + row;
                if ak >= fb_h || ad >= fb_h { continue; }
                
                for col in 0..w {
                    let am = ahc + col;
                    let dx = dst_x + col;
                    if am >= fb_w || dx >= fb_w { continue; }
                    
                    let val = core::ptr::read_volatile(base.offset((ak * pitch + am) as isize));
                    core::ptr::write_volatile(base.offset((ad * pitch + dx) as isize), val);
                }
            }
        }
    }
}






pub fn init() {
    crate::serial_println!("[NVIDIA] ═══════════════════════════════════════════════════");
    crate::serial_println!("[NVIDIA] NVIDIA GPU Driver — NV50 (Tesla) Family");
    crate::serial_println!("[NVIDIA] ═══════════════════════════════════════════════════");
    
    
    let s = match ccv() {
        Some(d) => d,
        None => {
            crate::serial_println!("[NVIDIA] No supported NVIDIA GPU found on PCI bus");
            
            let bbd = pci::bsp(pci::class::Du);
            if bbd.is_empty() {
                crate::serial_println!("[NVIDIA] No display controllers found at all");
            } else {
                for d in &bbd {
                    crate::serial_println!("[NVIDIA] Display: {:04X}:{:04X} at {:02X}:{:02X}.{}",
                        d.vendor_id, d.device_id, d.bus, d.device, d.function);
                }
            }
            return;
        }
    };
    
    crate::serial_println!("[NVIDIA] Found: {:04X}:{:04X} rev {:02X} at {:02X}:{:02X}.{}",
        s.vendor_id, s.device_id, s.revision, s.bus, s.device, s.function);
    
    
    pci::bzi(&s);
    pci::bzj(&s);
    
    
    let cbs = match s.bar_address(bar::Amp) {
        Some(addr) if addr > 0 => addr,
        _ => {
            crate::serial_println!("[NVIDIA] ERROR: BAR0 (MMIO) not available");
            return;
        }
    };
    
    let mmio_size = 16 * 1024 * 1024; 
    crate::serial_println!("[NVIDIA] BAR0 (MMIO): phys={:#010X} size={}MB", cbs, mmio_size / (1024*1024));
    
    let akb = match memory::yv(cbs, mmio_size) {
        Ok(v) => v,
        Err(e) => {
            crate::serial_println!("[NVIDIA] ERROR: Failed to map BAR0: {}", e);
            return;
        }
    };
    crate::serial_println!("[NVIDIA] BAR0 mapped at virt={:#014X}", akb);
    CJY_.store(akb, Ordering::SeqCst);
    
    
    let dgk = s.bar_address(bar::Agb).unwrap_or(0);
    let mut cez: u64 = 0;
    let mut csn: u64 = 0;
    
    if dgk > 0 {
        
        let pyj = s.bar[bar::Agb];
        
        csn = 256 * 1024 * 1024; 
        
        crate::serial_println!("[NVIDIA] BAR1 (VRAM): phys={:#010X} aperture={}MB", 
            dgk, csn / (1024*1024));
        
        
        
        let daw = 16 * 1024 * 1024;
        match memory::yv(dgk, daw) {
            Ok(v) => {
                cez = v;
                crate::serial_println!("[NVIDIA] BAR1 mapped at virt={:#014X} ({}MB)", v, daw / (1024*1024));
                ALN_.store(cez, Ordering::SeqCst);
            }
            Err(e) => {
                crate::serial_println!("[NVIDIA] WARNING: Failed to map BAR1 VRAM: {}", e);
                
            }
        }
    }
    
    
    let (chipset_id, stepping) = unsafe { gqe(akb) };
    
    
    let vram_size = unsafe { gqj(akb) };
    
    
    let (pcie_gen, pcie_width) = ocw(&s);
    
    
    let lqh = unsafe { mft(akb) };
    if !lqh {
        crate::serial_println!("[NVIDIA] WARNING: Engine init had issues, continuing anyway");
    }
    
    
    let fifo = if cez > 0 {
        unsafe { oqh(akb, cez) }
    } else {
        None
    };
    
    let ffz = fifo.is_some() && cez > 0;
    
    
    let info = Pr {
        vendor_id: s.vendor_id,
        device_id: s.device_id,
        revision: s.revision,
        bus: s.bus,
        device: s.device,
        function: s.function,
        chipset_id,
        stepping,
        gpu_name: mfv(s.device_id),
        vram_size,
        mmio_base: akb,
        mmio_size: mmio_size as u64,
        vram_base: cez,
        vram_aperture_size: csn,
        pcie_gen,
        pcie_width,
    };
    
    
    crate::serial_println!("[NVIDIA] ───────────────────────────────────────────────────");
    crate::serial_println!("[NVIDIA] GPU: {} ({})", info.gpu_name, info.chipset_name());
    crate::serial_println!("[NVIDIA] PCI: {:04X}:{:04X} rev {:02X} at {:02X}:{:02X}.{}", 
        info.vendor_id, info.device_id, info.revision,
        info.bus, info.device, info.function);
    crate::serial_println!("[NVIDIA] Chipset: {:#04X} stepping {:#04X}", chipset_id, stepping);
    crate::serial_println!("[NVIDIA] VRAM: {} MB", info.vram_size / (1024 * 1024));
    crate::serial_println!("[NVIDIA] PCIe: Gen{} x{}", pcie_gen, pcie_width);
    crate::serial_println!("[NVIDIA] MMIO: {:#X} ({}MB)", akb, mmio_size / (1024*1024));
    if cez > 0 {
        crate::serial_println!("[NVIDIA] VRAM aperture: {:#X}", cez);
    }
    crate::serial_println!("[NVIDIA] 2D Acceleration: {}", if ffz { "READY" } else { "UNAVAILABLE" });
    crate::serial_println!("[NVIDIA] ───────────────────────────────────────────────────");
    
    
    let mut state = Dz.lock();
    state.initialized = true;
    state.gpu_info = Some(info);
    state.fifo = fifo;
    state.accel_2d_ready = ffz;
    OF_.store(true, Ordering::SeqCst);
    ALX_.store(ffz, Ordering::SeqCst);
    drop(state);
}


pub fn aud() -> bool {
    OF_.load(Ordering::Relaxed)
}


pub fn mrv() -> bool {
    ALX_.load(Ordering::Relaxed)
}


pub fn rk() -> Option<Pr> {
    Dz.lock().gpu_info.clone()
}


pub fn summary() -> String {
    if let Some(info) = rk() {
        info.summary_string()
    } else {
        String::from("No NVIDIA GPU detected")
    }
}
