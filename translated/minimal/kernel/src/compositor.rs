







use alloc::boxed::Box;
use alloc::vec::Vec;
use alloc::vec;
use spin::Mutex;
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};






#[repr(C)]
pub struct Ql {
    pub cy: *const u32,
    pub cs: *mut u32,
    pub cid: usize,
    pub epi: usize,
    pub z: usize,
    pub ac: usize,
}

unsafe impl Send for Ql {}
unsafe impl Sync for Ql {}


fn vkx(ay: usize, ci: usize, f: *mut u8) {
    let oi = unsafe { &*(f as *const Ql) };
    
    for c in ay..ci {
        if c >= oi.ac { break; }
        
        let cum = c * oi.cid;
        let bgu = c * oi.epi;
        
        unsafe {
            let cy = oi.cy.add(cum);
            let cs = oi.cs.add(bgu);
            
            #[cfg(target_arch = "x86_64")]
            crate::graphics::simd::dpd(cs, cy, oi.z);
            #[cfg(not(target_arch = "x86_64"))]
            core::ptr::copy_nonoverlapping(cy, cs, oi.z);
        }
    }
}


fn xvw(ay: usize, ci: usize, f: *mut u8) {
    let oi = unsafe { &*(f as *const Ql) };
    
    for c in ay..ci {
        if c >= oi.ac { break; }
        
        let cum = c * oi.cid;
        let bgu = c * oi.epi;
        
        unsafe {
            let cy = oi.cy.add(cum);
            let cs = oi.cs.add(bgu);
            
            #[cfg(target_arch = "x86_64")]
            crate::graphics::simd::dpd(cs, cy, oi.z);
            #[cfg(not(target_arch = "x86_64"))]
            core::ptr::copy_nonoverlapping(cy, cs, oi.z);
        }
    }
}






#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum LayerType {
    Apm = 0,    
    Caz = 1,          
    Cqm = 2,       
    Coa = 3,       
    Akx = 4,       
    Ctq = 5,        
}


pub struct Layer {
    pub eem: LayerType,
    pub b: u32,
    pub c: u32,
    pub z: u32,
    pub ac: u32,
    pub bi: Box<[u32]>,
    pub no: AtomicBool,
    pub iw: AtomicBool,
    pub adh: AtomicU32,  
}

impl Layer {
    
    pub fn new(eem: LayerType, b: u32, c: u32, z: u32, ac: u32) -> Self {
        let aw = (z * ac) as usize;
        Layer {
            eem,
            b,
            c,
            z,
            ac,
            bi: vec![0u32; aw].dsd(),
            no: AtomicBool::new(true),
            iw: AtomicBool::new(true),
            adh: AtomicU32::new(255),
        }
    }
    
    
    pub fn eyk(&mut self, b: u32, c: u32) {
        self.b = b;
        self.c = c;
        self.no.store(true, Ordering::SeqCst);
    }
    
    
    pub fn clear(&mut self, s: u32) {
        self.bi.vi(s);
        self.no.store(true, Ordering::SeqCst);
    }
    
    
    pub fn ah(&mut self, b: u32, c: u32, d: u32, i: u32, s: u32) {
        let dn = b.v(self.z);
        let dp = c.v(self.ac);
        let hy = (b + d).v(self.z);
        let jz = (c + i).v(self.ac);
        
        for x in dp..jz {
            let mu = (x * self.z + dn) as usize;
            let cub = (x * self.z + hy) as usize;
            if cub <= self.bi.len() {
                self.bi[mu..cub].vi(s);
            }
        }
        self.no.store(true, Ordering::SeqCst);
    }
    
    
    pub fn lx(&mut self, b: u32, c: u32, d: u32, i: u32, s: u32) {
        
        self.ah(b, c, d, 1, s);
        self.ah(b, c + i.ao(1), d, 1, s);
        
        self.ah(b, c, 1, i, s);
        self.ah(b + d.ao(1), c, 1, i, s);
    }
    
    
    #[inline]
    pub fn aht(&mut self, b: u32, c: u32, s: u32) {
        if b < self.z && c < self.ac {
            let w = (c * self.z + b) as usize;
            if w < self.bi.len() {
                self.bi[w] = s;
            }
        }
    }
    
    
    #[inline]
    pub fn beg(&self, b: u32, c: u32) -> u32 {
        if b < self.z && c < self.ac {
            let w = (c * self.z + b) as usize;
            if w < self.bi.len() {
                return self.bi[w];
            }
        }
        0
    }
    
    
    pub fn abc(&mut self, cx: u32, ae: u32, dy: u32, s: u32) {
        let m = dy as i32;
        let cx = cx as i32;
        let ae = ae as i32;
        
        for bg in -m..=m {
            for dx in -m..=m {
                if dx * dx + bg * bg <= m * m {
                    let y = cx + dx;
                    let x = ae + bg;
                    if y >= 0 && x >= 0 {
                        self.aht(y as u32, x as u32, s);
                    }
                }
            }
        }
        self.no.store(true, Ordering::SeqCst);
    }
    
    
    pub fn cb(&mut self, text: &str, b: u32, c: u32, s: u32) {
        let mut cx = b;
        for r in text.bw() {
            if r == ' ' {
                cx += 8;
                continue;
            }
            
            let ka = crate::framebuffer::font::ada(r);
            for (br, &fs) in ka.iter().cf() {
                for bj in 0..8 {
                    if fs & (0x80 >> bj) != 0 {
                        self.aht(cx + bj, c + br as u32, s);
                    }
                }
            }
            cx += 8;
        }
        self.no.store(true, Ordering::SeqCst);
    }
}


pub struct Compositor {
    my: Vec<Layer>,
    anv: u32,
    akr: u32,
    hdt: Box<[u32]>,
    
    
    fjw: usize,  
    laa: usize,
}

impl Compositor {
    
    pub fn new(z: u32, ac: u32) -> Self {
        let aw = (z * ac) as usize;
        Compositor {
            my: Vec::new(),
            anv: z,
            akr: ac,
            hdt: vec![0u32; aw].dsd(),
            fjw: 0,
            laa: 0,
        }
    }
    
    
    pub fn dyc(&mut self, eem: LayerType, b: u32, c: u32, d: u32, i: u32) -> usize {
        let fl = Layer::new(eem, b, c, d, i);
        self.my.push(fl);
        self.my.len() - 1
    }
    
    
    pub fn qfj(&mut self, eem: LayerType) -> usize {
        self.dyc(eem, 0, 0, self.anv, self.akr)
    }
    
    
    pub fn dhm(&mut self, index: usize) -> Option<&mut Layer> {
        self.my.ds(index)
    }
    
    
    pub fn iws(&self, index: usize) -> Option<&Layer> {
        self.my.get(index)
    }
    
    
    
    pub fn skz(&mut self) {
        if crate::drivers::virtio_gpu::anl() {
            if let Some((ptr, d, i)) = crate::drivers::virtio_gpu::iwv() {
                self.fjw = ptr as usize;
                self.laa = (d * i) as usize;
                crate::serial_println!("[COMPOSITOR] GPU direct mode: composite → GPU buffer (skip 4MB copy!)");
            }
        }
    }
    
    
    
    
    
    pub fn iov(&mut self) {
        
        let (ejp, dwp) = if self.fjw != 0 {
            (self.fjw as *mut u32, self.laa)
        } else {
            (self.hdt.mw(), self.hdt.len())
        };

        
        let mut lzg: Vec<usize> = (0..self.my.len()).collect();
        lzg.bxf(|&a| self.my[a].eem as u8);
        
        
        
        let wpi = if let Some(&iuv) = lzg.fv() {
            let fl = &self.my[iuv];
            fl.iw.load(Ordering::SeqCst) 
                && fl.eem == LayerType::Apm
                && fl.b == 0 && fl.c == 0
                && fl.z >= self.anv
                && fl.ac >= self.akr
                && fl.adh.load(Ordering::SeqCst) == 255
        } else {
            false
        };
        
        
        if !wpi {
            
            #[cfg(target_arch = "x86_64")]
            unsafe {
                crate::graphics::simd::bed(
                    ejp,
                    dwp,
                    0xFF000000
                );
            }
            #[cfg(not(target_arch = "x86_64"))]
            unsafe {
                for a in 0..dwp {
                    *ejp.add(a) = 0xFF000000;
                }
            }
        }
        
        
        for &aup in &lzg {
            let fl = &self.my[aup];
            if !fl.iw.load(Ordering::SeqCst) {
                continue;
            }
            
            let adh = fl.adh.load(Ordering::SeqCst);
            
            
            
            if adh == 255 && fl.b == 0 && fl.z == self.anv 
               && fl.eem == LayerType::Apm {
                let jrf = fl.ac.v(self.akr.ao(fl.c));
                
                
                let oi = Ql {
                    cy: fl.bi.fq(),
                    cs: ejp,
                    cid: fl.z as usize,
                    epi: self.anv as usize,
                    z: fl.z as usize,
                    ac: jrf as usize,
                };
                
                crate::cpu::smp::daj(
                    jrf as usize,
                    vkx,
                    &oi as *const Ql as *mut u8,
                );
                continue;
            }
            
            
            if adh == 255 {
                for ct in 0..fl.ac {
                    let abi = fl.c + ct;
                    if abi >= self.akr {
                        continue;
                    }
                    
                    
                    let big = (ct * fl.z) as usize;
                    let dqh = (abi * self.anv + fl.b) as usize;
                    let mau = fl.z.v(self.anv - fl.b) as usize;
                    
                    if fl.b < self.anv 
                       && big + mau <= fl.bi.len()
                       && dqh + mau <= dwp {
                        
                        for a in 0..mau {
                            let fvg = fl.bi[big + a];
                            let gsx = (fvg >> 24) & 0xFF;
                            if gsx > 200 { 
                                unsafe { *ejp.add(dqh + a) = fvg; }
                            } else if gsx > 0 {
                                
                                let krs = unsafe { *ejp.add(dqh + a) };
                                unsafe { *ejp.add(dqh + a) = gyl(fvg, krs, gsx); }
                            }
                        }
                    }
                }
                continue;
            }
            
            
            for ct in 0..fl.ac {
                let abi = fl.c + ct;
                if abi >= self.akr {
                    continue;
                }
                
                for mj in 0..fl.z {
                    let xu = fl.b + mj;
                    if xu >= self.anv {
                        continue;
                    }
                    
                    let blf = (ct * fl.z + mj) as usize;
                    let bbm = (abi * self.anv + xu) as usize;
                    
                    if blf >= fl.bi.len() || bbm >= dwp {
                        continue;
                    }
                    
                    let fvg = fl.bi[blf];
                    let gsx = ((fvg >> 24) & 0xFF) as u32;
                    
                    
                    if gsx == 0 {
                        continue;
                    }
                    
                    
                    let kwe = (gsx * adh) / 255;
                    
                    if kwe >= 255 {
                        
                        unsafe { *ejp.add(bbm) = fvg; }
                    } else if kwe > 0 {
                        
                        let krs = unsafe { *ejp.add(bbm) };
                        unsafe { *ejp.add(bbm) = gyl(fvg, krs, kwe); }
                    }
                }
            }
        }
        
    }
    
    
    
    
    
    
    
    
    
    
    pub fn brs(&self) {
        
        if self.fjw != 0 {
            
            let _ = crate::drivers::virtio_gpu::owx();
            
            self.qaf();
            return;
        }
        
        
        if crate::drivers::virtio_gpu::anl() {
            if let Some((hlu, erl, hlt)) = crate::drivers::virtio_gpu::iwv() {
                let aoo = (self.anv as usize).v(erl as usize);
                let bbg = (self.akr as usize).v(hlt as usize);
                
                unsafe {
                    let mha = self.hdt.fq();
                    let sgy = hlu;
                    
                    for c in 0..bbg {
                        let cy = mha.add(c * self.anv as usize);
                        let cs = sgy.add(c * erl as usize);
                        
                        #[cfg(target_arch = "x86_64")]
                        crate::graphics::simd::dpd(cs, cy, aoo);
                        #[cfg(not(target_arch = "x86_64"))]
                        core::ptr::copy_nonoverlapping(cy, cs, aoo);
                    }
                }
                
                let _ = crate::drivers::virtio_gpu::owx();
                return;
            }
        }
        
        
        
        self.qaf();
    }
    
    
    
    
    
    
    pub fn vkv(&self) {
        
        
    }
    
    
    
    
    
    
    
    
    fn qaf(&self) {
        use crate::framebuffer::{BJ_, AB_, Z_, CA_};
        
        let ag = BJ_.load(Ordering::SeqCst);
        if ag.abq() { return; }
        
        let lu = AB_.load(Ordering::SeqCst) as usize;
        let qh = Z_.load(Ordering::SeqCst) as usize;
        let jb = CA_.load(Ordering::SeqCst) as usize;
        let luc = jb / 4;
        
        let row = lu.v(self.anv as usize);
        let nfw = qh.v(self.akr as usize);
        
        
        let mha = if self.fjw != 0 {
            self.fjw as *const u32
        } else {
            self.hdt.fq()
        };
        
        
        let oi = Ql {
            cy: mha,
            cs: ag as *mut u32,
            cid: self.anv as usize,
            epi: luc,
            z: row,
            ac: nfw,
        };
        
        crate::cpu::smp::daj(
            nfw,
            xvw,
            &oi as *const Ql as *mut u8,
        );
    }
    
    
    pub fn ude(&self) -> usize {
        self.my.len()
    }
}


#[inline]
fn gyl(cy: u32, cs: u32, dw: u32) -> u32 {
    let akg = 255 - dw;
    
    let adz = (cy >> 16) & 0xFF;
    let bsi = (cy >> 8) & 0xFF;
    let is = cy & 0xFF;
    
    let ahh = (cs >> 16) & 0xFF;
    let bgs = (cs >> 8) & 0xFF;
    let ng = cs & 0xFF;
    
    let m = (adz * dw + ahh * akg) / 255;
    let at = (bsi * dw + bgs * akg) / 255;
    let o = (is * dw + ng * akg) / 255;
    
    0xFF000000 | (m << 16) | (at << 8) | o
}




static Oz: Mutex<Option<Compositor>> = Mutex::new(None);


pub fn init(z: u32, ac: u32) {
    let compositor = Compositor::new(z, ac);
    *Oz.lock() = Some(compositor);
    crate::serial_println!("[COMPOSITOR] Initialized {}x{}", z, ac);
}


pub fn dne<G, Ac>(bb: G) -> Option<Ac>
where
    G: FnOnce(&mut Compositor) -> Ac,
{
    Oz.lock().as_mut().map(bb)
}


pub fn zwb<G, Ac>(bb: G) -> Option<Ac>
where
    G: FnOnce(&Compositor) -> Ac,
{
    Oz.lock().as_ref().map(bb)
}
