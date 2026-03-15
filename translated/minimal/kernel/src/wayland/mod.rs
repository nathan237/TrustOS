




























pub mod protocol;
pub mod compositor;
pub mod surface;
pub mod shm;
pub mod seat;
pub mod display;
pub mod terminal;

use alloc::vec::Vec;
use alloc::string::String;
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use spin::Mutex;
use core::sync::atomic::{AtomicU32, Ordering};

pub use protocol::*;
pub use compositor::*;
pub use surface::*;
pub use shm::*;
pub use seat::*;
pub use display::*;






static Oz: Mutex<Option<WaylandCompositor>> = Mutex::new(None);


static CHS_: AtomicU32 = AtomicU32::new(1);


pub fn lny() -> u32 {
    CHS_.fetch_add(1, Ordering::SeqCst)
}


pub fn init() -> Result<(), &'static str> {
    let mut compositor = Oz.lock();
    if compositor.is_some() {
        return Err("Wayland compositor already initialized");
    }
    
    let (z, ac) = crate::framebuffer::yn();
    
    let ekv = WaylandCompositor::new(z, ac);
    *compositor = Some(ekv);
    
    crate::serial_println!("[WAYLAND] Compositor initialized ({}x{})", z, ac);
    Ok(())
}


pub fn dne<G, Ac>(bb: G) -> Option<Ac>
where
    G: FnOnce(&mut WaylandCompositor) -> Ac,
{
    let mut adb = Oz.lock();
    adb.as_mut().map(bb)
}


pub fn ffn() {
    dne(|compositor| {
        compositor.nff();
    });
}


pub fn oyb() {
    dne(|compositor| {
        compositor.oyb();
    });
}






pub struct WaylandCompositor {
    
    pub z: u32,
    pub ac: u32,
    
    
    pub axa: BTreeMap<u32, Surface>,
    
    
    pub ezh: Vec<u32>,
    
    
    pub eqq: Option<u32>,
    
    
    pub hvo: i32,
    pub hvp: i32,
    
    
    pub mfn: BTreeMap<u32, ShmPool>,
    
    
    pub rbr: BTreeMap<u32, Cqi>,
    
    
    pub kxa: u64,
    
    
    pub cdb: u32,
}

impl WaylandCompositor {
    pub fn new(z: u32, ac: u32) -> Self {
        Self {
            z,
            ac,
            axa: BTreeMap::new(),
            ezh: Vec::new(),
            eqq: None,
            hvo: (z / 2) as i32,
            hvp: (ac / 2) as i32,
            mfn: BTreeMap::new(),
            rbr: BTreeMap::new(),
            kxa: 0,
            cdb: 0xFF0A0F0C, 
        }
    }
    
    
    pub fn fgc(&mut self) -> u32 {
        let ad = lny();
        let surface = Surface::new(ad);
        self.axa.insert(ad, surface);
        self.ezh.push(ad);
        crate::serial_println!("[WAYLAND] Created surface {}", ad);
        ad
    }
    
    
    pub fn ylu(&mut self, ad: u32) {
        self.axa.remove(&ad);
        self.ezh.ajm(|&b| b != ad);
        if self.eqq == Some(ad) {
            self.eqq = self.ezh.qv().hu();
        }
        crate::serial_println!("[WAYLAND] Destroyed surface {}", ad);
    }
    
    
    pub fn zhg(&mut self, ad: u32) {
        self.ezh.ajm(|&b| b != ad);
        self.ezh.push(ad);
        self.eqq = Some(ad);
    }
    
    
    pub fn ykq(&mut self, aw: usize) -> u32 {
        let ad = lny();
        let lut = ShmPool::new(ad, aw);
        self.mfn.insert(ad, lut);
        crate::serial_println!("[WAYLAND] Created SHM pool {} ({} bytes)", ad, aw);
        ad
    }
    
    
    pub fn nff(&mut self) {
        self.kxa += 1;
        
        
        let (z, ac) = crate::framebuffer::yn();
        
        
        self.gff(z, ac);
        
        
        for &cmz in &self.ezh {
            if let Some(surface) = self.axa.get(&cmz) {
                if surface.iw && surface.gda {
                    self.sfu(surface);
                }
            }
        }
        
        
        self.dqf();
    }
    
    fn gff(&self, z: u32, ac: u32) {
        
        for c in 0..ac {
            for b in 0..z {
                let pattern: u32 = if (b + c) % 32 < 16 { 0x00 } else { 0x02 };
                let xyj: u32 = 0xFF000000u32 | (pattern << 16) | ((pattern + 0x08) << 8) | pattern;
                crate::framebuffer::sf(b, c, self.cdb);
            }
        }
    }
    
    fn sfu(&self, surface: &Surface) {
        if surface.bi.is_empty() {
            return;
        }
        
        let b = surface.b;
        let c = surface.c;
        let d = surface.z;
        let i = surface.ac;
        
        
        if surface.hmn {
            self.sco(surface);
        }
        
        
        for cq in 0..i {
            for cr in 0..d {
                let w = (cq * d + cr) as usize;
                if w < surface.bi.len() {
                    let il = surface.bi[w];
                    let y = b + cr as i32;
                    let x = c + cq as i32;
                    if y >= 0 && x >= 0 {
                        crate::framebuffer::sf(y as u32, x as u32, il);
                    }
                }
            }
        }
    }
    
    fn sco(&self, surface: &Surface) {
        let b = surface.b;
        let c = surface.c;
        let d = surface.z as i32;
        let fwu = 28;
        
        
        let ejy = if self.eqq == Some(surface.ad) {
            0xFF1A2A20 
        } else {
            0xFF0D1310 
        };
        
        for ty in 0..fwu {
            for gx in 0..d {
                let y = b + gx;
                let x = c - fwu + ty;
                if y >= 0 && x >= 0 {
                    crate::framebuffer::sf(y as u32, x as u32, ejy);
                }
            }
        }
        
        
        let kn = c - fwu + 6;
        let ask = 14;
        
        
        self.cxc(b + 12, kn + 7, ask / 2, 0xFF3A2828);
        
        self.cxc(b + 32, kn + 7, ask / 2, 0xFF2A3028);
        
        self.cxc(b + 52, kn + 7, ask / 2, 0xFF2A2A20);
        
        
        if !surface.dq.is_empty() {
            
            let cnf = b + 70;
            let cce = kn + 4;
            
        }
    }
    
    fn cxc(&self, cx: i32, ae: i32, m: i32, s: u32) {
        for bg in -m..=m {
            for dx in -m..=m {
                if dx * dx + bg * bg <= m * m {
                    let y = cx + dx;
                    let x = ae + bg;
                    if y >= 0 && x >= 0 {
                        crate::framebuffer::sf(y as u32, x as u32, s);
                    }
                }
            }
        }
    }
    
    fn dqf(&self) {
        
        let gi = [
            0b11000000u8,
            0b11100000u8,
            0b11110000u8,
            0b11111000u8,
            0b11111100u8,
            0b11111110u8,
            0b11111111u8,
            0b11111100u8,
            0b11111100u8,
            0b11001100u8,
            0b10000110u8,
            0b00000110u8,
            0b00000011u8,
            0b00000011u8,
            0b00000000u8,
        ];
        
        for (c, br) in gi.iter().cf() {
            for b in 0..8 {
                if (br >> (7 - b)) & 1 == 1 {
                    let y = self.hvo + b;
                    let x = self.hvp + c as i32;
                    if y >= 0 && x >= 0 && (y as u32) < self.z && (x as u32) < self.ac {
                        crate::framebuffer::sf(y as u32, x as u32, 0xFFFFFFFF);
                    }
                }
            }
        }
    }
    
    
    pub fn zdc(&mut self, dx: i32, bg: i32) {
        self.hvo = (self.hvo + dx).qp(0, self.z as i32 - 1);
        self.hvp = (self.hvp + bg).qp(0, self.ac as i32 - 1);
    }
    
    
    pub fn oyb(&mut self) {
        
    }
    
    
    pub fn zqc(&self, b: i32, c: i32) -> Option<u32> {
        
        for &ad in self.ezh.iter().vv() {
            if let Some(surface) = self.axa.get(&ad) {
                if surface.contains(b, c) {
                    return Some(ad);
                }
            }
        }
        None
    }
}


pub struct Cqi {
    pub ad: u32,
    pub axa: Vec<u32>,
}
