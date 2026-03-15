





pub mod font;
pub mod logo;

use core::fmt;
use spin::Mutex;
use alloc::vec::Vec;
use alloc::boxed::Box;
use alloc::format;
use core::sync::atomic::{AtomicPtr, AtomicU64, AtomicBool, Ordering};
use crate::math::ahn;


struct Cxg {
    ag: *mut u8,
    z: u64,
    ac: u64,
    jb: u64,
    cwa: u16,
}


struct Rw {
    lf: usize,
    ot: usize,
    axw: u32,
    vp: u32,
}


pub static BJ_: AtomicPtr<u8> = AtomicPtr::new(core::ptr::null_mut());
pub static AB_: AtomicU64 = AtomicU64::new(0);
pub static Z_: AtomicU64 = AtomicU64::new(0);
pub static CA_: AtomicU64 = AtomicU64::new(0);
pub static BVC_: AtomicU64 = AtomicU64::new(32); 


static Ec: Mutex<Option<Box<[u32]>>> = Mutex::new(None);
static BDM_: Mutex<Option<Box<[u32]>>> = Mutex::new(None);
static FX_: AtomicBool = AtomicBool::new(false);




static HZ_: AtomicPtr<u32> = AtomicPtr::new(core::ptr::null_mut());
static KD_: AtomicU64 = AtomicU64::new(0);
static TH_: AtomicU64 = AtomicU64::new(0);





use core::sync::atomic::AtomicU32;

static AAC_: AtomicBool = AtomicBool::new(false);
static ANW_: AtomicU32 = AtomicU32::new(0);
static ANY_: AtomicU32 = AtomicU32::new(0);
static ANX_: AtomicU32 = AtomicU32::new(u32::O);
static ANZ_: AtomicU32 = AtomicU32::new(u32::O);


pub fn pir(b: u32, c: u32, d: u32, i: u32) {
    ANW_.store(b, Ordering::Relaxed);
    ANY_.store(c, Ordering::Relaxed);
    ANX_.store(b.akq(d), Ordering::Relaxed);
    ANZ_.store(c.akq(i), Ordering::Relaxed);
    AAC_.store(true, Ordering::Release);
}


pub fn nde() {
    AAC_.store(false, Ordering::Release);
}


#[inline(always)]
pub fn iob(b: u32, c: u32) -> bool {
    if !AAC_.load(Ordering::Relaxed) {
        return true;
    }
    b >= ANW_.load(Ordering::Relaxed) && b < ANX_.load(Ordering::Relaxed) &&
    c >= ANY_.load(Ordering::Relaxed) && c < ANZ_.load(Ordering::Relaxed)
}




pub fn ili() {
    if let Some(ref mut k) = *Ec.lock() {
        HZ_.store(k.mw(), Ordering::Release);
        KD_.store(AB_.load(Ordering::Relaxed), Ordering::Release);
        TH_.store(Z_.load(Ordering::Relaxed), Ordering::Release);
    }
}


pub fn gge() {
    HZ_.store(core::ptr::null_mut(), Ordering::Release);
}




#[inline(always)]
pub fn sww() -> (*mut u32, u32, u32) {
    let ptr = HZ_.load(Ordering::Relaxed);
    let oq = KD_.load(Ordering::Relaxed) as u32;
    let ac = TH_.load(Ordering::Relaxed) as u32;
    (ptr, oq, ac)
}



#[inline(always)]
pub fn ii(b: u32, c: u32, s: u32) {
    let ptr = HZ_.load(Ordering::Relaxed);
    if ptr.abq() { sf(b, c, s); return; }
    let oq = KD_.load(Ordering::Relaxed) as u32;
    let ac = TH_.load(Ordering::Relaxed) as u32;
    if b >= oq || c >= ac { return; }
    if !iob(b, c) { return; }
    unsafe { *ptr.add(c as usize * oq as usize + b as usize) = s; }
}


#[inline(always)]
pub fn iwt(b: u32, c: u32) -> u32 {
    let ptr = HZ_.load(Ordering::Relaxed);
    if ptr.abq() { return beg(b, c); }
    let oq = KD_.load(Ordering::Relaxed) as u32;
    let ac = TH_.load(Ordering::Relaxed) as u32;
    if b >= oq || c >= ac { return 0; }
    unsafe { *ptr.add(c as usize * oq as usize + b as usize) }
}



const AHN_: usize = 1000;  
const PH_: usize = 256; 


#[derive(Clone)]
struct ScrollbackLine {
    bw: [char; PH_],
    colors: [(u32, u32); PH_], 
    len: usize,
}

impl ScrollbackLine {
    const fn new() -> Self {
        ScrollbackLine {
            bw: [' '; PH_],
            colors: [(0xFFFFFFFF, 0xFF000000); PH_],
            len: 0,
        }
    }
}


struct ScrollbackBuffer {
    ak: Vec<ScrollbackLine>,
    bgp: ScrollbackLine,
    px: usize,  
    eti: bool,     
}

impl ScrollbackBuffer {
    fn new() -> Self {
        ScrollbackBuffer {
            ak: Vec::fc(AHN_),
            bgp: ScrollbackLine::new(),
            px: 0,
            eti: false,
        }
    }
    
    
    fn voh(&mut self, r: char, lp: u32, ei: u32) {
        if r == '\n' {
            
            self.rmt();
        } else if r == '\r' {
            
            self.bgp.len = 0;
        } else if r == '\x08' {
            
            if self.bgp.len > 0 {
                self.bgp.len -= 1;
            }
        } else if r.jbb() || r == ' ' {
            if self.bgp.len < PH_ {
                self.bgp.bw[self.bgp.len] = r;
                self.bgp.colors[self.bgp.len] = (lp, ei);
                self.bgp.len += 1;
            }
        }
    }
    
    
    fn rmt(&mut self) {
        if self.ak.len() >= AHN_ {
            self.ak.remove(0); 
        }
        self.ak.push(self.bgp.clone());
        self.bgp = ScrollbackLine::new();
    }
    
    
    fn bss(&self) -> usize {
        self.ak.len()
    }
}

static Ny: Mutex<Option<ScrollbackBuffer>> = Mutex::new(None);
static WU_: AtomicBool = AtomicBool::new(false);



static RH_: Mutex<Option<Box<[u32]>>> = Mutex::new(None);
static LV_: AtomicBool = AtomicBool::new(false);



pub const GQ_: usize = 32;

#[derive(Clone, Copy, Default)]
pub struct DirtyRect {
    pub b: u32,
    pub c: u32,
    pub d: u32,
    pub i: u32,
}

impl DirtyRect {
    pub const fn new(b: u32, c: u32, d: u32, i: u32) -> Self {
        DirtyRect { b, c, d, i }
    }
    
    
    pub fn cld(&self) -> bool {
        self.d > 0 && self.i > 0
    }
    
    
    pub fn unf(&self, gq: &DirtyRect) -> DirtyRect {
        if !self.cld() { return *gq; }
        if !gq.cld() { return *self; }
        
        let dn = self.b.v(gq.b);
        let dp = self.c.v(gq.c);
        let hy = (self.b + self.d).am(gq.b + gq.d);
        let jz = (self.c + self.i).am(gq.c + gq.i);
        
        DirtyRect { b: dn, c: dp, d: hy - dn, i: jz - dp }
    }
    
    
    pub fn vag(&self, gq: &DirtyRect) -> bool {
        !(self.b + self.d <= gq.b || gq.b + gq.d <= self.b ||
          self.c + self.i <= gq.c || gq.c + gq.i <= self.c)
    }
}

struct DirtyRectList {
    akn: [DirtyRect; GQ_],
    az: usize,
    asw: bool,
}

impl DirtyRectList {
    const fn new() -> Self {
        DirtyRectList {
            akn: [DirtyRect { b: 0, c: 0, d: 0, i: 0 }; GQ_],
            az: 0,
            asw: true, 
        }
    }
    
    fn add(&mut self, ha: DirtyRect) {
        if self.asw { return; } 
        if !ha.cld() { return; }
        
        
        for a in 0..self.az {
            if self.akn[a].vag(&ha) {
                self.akn[a] = self.akn[a].unf(&ha);
                return;
            }
        }
        
        
        if self.az < GQ_ {
            self.akn[self.az] = ha;
            self.az += 1;
        } else {
            
            self.asw = true;
        }
    }
    
    fn clear(&mut self) {
        self.az = 0;
        self.asw = false;
    }
    
    fn olc(&mut self) {
        self.asw = true;
    }
}

static MT_: Mutex<DirtyRectList> = Mutex::new(DirtyRectList::new());

static Fk: Mutex<Rw> = Mutex::new(Rw {
    lf: 0,
    ot: 0,
    axw: 0xFFFFFFFF, 
    vp: 0xFF000000, 
});


const CH_: usize = 8;
const BN_: usize = 16;


const AZL_: usize = 16 * 1024 * 1024;



pub fn init(ag: *mut u8, z: u64, ac: u64, jb: u64, cwa: u16) {
    
    if cwa != 32 {
        crate::serial_println!("[FB] WARNING: unsupported bpp={}, forcing 32bpp interpretation (may have color artifacts)", cwa);
    }

    BJ_.store(ag, Ordering::SeqCst);
    AB_.store(z, Ordering::SeqCst);
    Z_.store(ac, Ordering::SeqCst);
    CA_.store(jb, Ordering::SeqCst);
    BVC_.store(cwa as u64, Ordering::SeqCst);
    
    
    
    
    
    clear();
}


pub fn leh() {
    *Ny.lock() = Some(ScrollbackBuffer::new());
    WU_.store(true, Ordering::SeqCst);
    crate::serial_println!("[FB] Scrollback buffer initialized ({} lines max)", AHN_);
}


pub fn ky() -> bool {
    !BJ_.load(Ordering::SeqCst).abq()
}


pub fn z() -> u32 {
    AB_.load(Ordering::SeqCst) as u32
}


pub fn ac() -> u32 {
    Z_.load(Ordering::SeqCst) as u32
}


pub fn kyq() -> *mut u32 {
    BJ_.load(Ordering::SeqCst) as *mut u32
}


pub fn clear() {
    let ag = BJ_.load(Ordering::SeqCst);
    if ag.abq() {
        return;
    }
    
    let ac = Z_.load(Ordering::SeqCst) as usize;
    let jb = CA_.load(Ordering::SeqCst) as usize;
    let vp = Fk.lock().vp;
    
    for c in 0..ac {
        let br = unsafe { ag.add(c * jb) };
        for b in 0..(jb / 4) {
            unsafe {
                br.add(b * 4).cast::<u32>().write_volatile(vp);
            }
        }
    }
    
    let mut console = Fk.lock();
    console.lf = 0;
    console.ot = 0;
}


pub fn dbv(s: u32) {
    Fk.lock().axw = s;
}


pub fn zmn(s: u32) {
    Fk.lock().vp = s;
}


pub fn sf(b: u32, c: u32, s: u32) {
    let ag = BJ_.load(Ordering::SeqCst);
    if ag.abq() {
        return;
    }
    
    let z = AB_.load(Ordering::SeqCst) as u32;
    let ac = Z_.load(Ordering::SeqCst) as u32;
    let jb = CA_.load(Ordering::SeqCst) as usize;
    
    if b >= z || c >= ac {
        return;
    }
    if !iob(b, c) { return; }
    
    
    if FX_.load(Ordering::SeqCst) {
        if let Some(ref mut k) = *Ec.lock() {
            let w = c as usize * z as usize + b as usize;
            if w < k.len() {
                k[w] = s;
            }
        }
    } else {
        let l = c as usize * jb + b as usize * 4;
        unsafe {
            ag.add(l).cast::<u32>().write_volatile(s);
        }
    }
}


pub fn beg(b: u32, c: u32) -> u32 {
    let ag = BJ_.load(Ordering::SeqCst);
    if ag.abq() {
        return 0;
    }
    
    let z = AB_.load(Ordering::SeqCst) as u32;
    let ac = Z_.load(Ordering::SeqCst) as u32;
    let jb = CA_.load(Ordering::SeqCst) as usize;
    
    if b >= z || c >= ac {
        return 0;
    }
    
    
    if FX_.load(Ordering::SeqCst) {
        if let Some(ref k) = *Ec.lock() {
            let w = c as usize * z as usize + b as usize;
            if w < k.len() {
                return k[w];
            }
        }
        0
    } else {
        let l = c as usize * jb + b as usize * 4;
        unsafe {
            ag.add(l).cast::<u32>().read_volatile()
        }
    }
}







pub struct FastPixelContext {
    pub ag: *mut u8,
    pub z: usize,
    pub ac: usize,
    pub jb: usize,
    pub gap: bool,
}

impl FastPixelContext {
    
    #[inline]
    pub fn new() -> Self {
        FastPixelContext {
            ag: BJ_.load(Ordering::SeqCst),
            z: AB_.load(Ordering::SeqCst) as usize,
            ac: Z_.load(Ordering::SeqCst) as usize,
            jb: CA_.load(Ordering::SeqCst) as usize,
            gap: FX_.load(Ordering::SeqCst),
        }
    }
    
    
    
    #[inline(always)]
    pub unsafe fn vom(&self, b: usize, c: usize, s: u32) {
        if self.gap {
            
            if let Some(ref mut k) = *Ec.lock() {
                let w = c * self.z + b;
                *k.yuc(w) = s;
            }
        } else {
            let l = c * self.jb + b * 4;
            (self.ag.add(l) as *mut u32).write_volatile(s);
        }
    }
    
    
    #[inline(always)]
    pub fn sf(&self, b: usize, c: usize, s: u32) {
        if b >= self.z || c >= self.ac { return; }
        unsafe { self.vom(b, c, s); }
    }
    
    
    #[inline]
    pub fn yqm(&self, b: usize, c: usize, len: usize, s: u32) {
        if c >= self.ac || b >= self.z { return; }
        let fck = len.v(self.z - b);
        
        if self.gap {
            if let Some(ref mut k) = *Ec.lock() {
                let ay = c * self.z + b;
                #[cfg(target_arch = "x86_64")]
                unsafe {
                    crate::graphics::simd::bed(
                        k.mw().add(ay),
                        fck,
                        s
                    );
                }
                #[cfg(not(target_arch = "x86_64"))]
                {
                    k[ay..ay + fck].vi(s);
                }
            }
        } else {
            unsafe {
                let ptr = (self.ag.add(c * self.jb + b * 4)) as *mut u32;
                #[cfg(target_arch = "x86_64")]
                {
                    crate::graphics::simd::bed(ptr, fck, s);
                }
                #[cfg(not(target_arch = "x86_64"))]
                {
                    for a in 0..fck {
                        ptr.add(a).write_volatile(s);
                    }
                }
            }
        }
    }
    
    
    
    pub fn yso(&self) -> Option<alloc::boxed::Box<[u32]>> {
        if self.gap {
            if let Some(ref k) = *Ec.lock() {
                
                return Some(k.clone());
            }
        }
        None
    }
}


pub fn yn() -> (u32, u32) {
    (AB_.load(Ordering::SeqCst) as u32, Z_.load(Ordering::SeqCst) as u32)
}







#[inline]
pub fn zvz<G: FnOnce(*mut u32, usize, usize, usize)>(bb: G) -> bool {
    let z = AB_.load(Ordering::SeqCst) as usize;
    let ac = Z_.load(Ordering::SeqCst) as usize;
    if z == 0 || ac == 0 { return false; }
    if let Some(ref mut k) = *Ec.lock() {
        bb(k.mw(), z, ac, z);
        true
    } else {
        false
    }
}


pub fn iwp() -> *mut u8 {
    BJ_.load(Ordering::SeqCst)
}


pub fn kyo() -> usize {
    CA_.load(Ordering::SeqCst) as usize
}




pub fn beo() {
    let z = AB_.load(Ordering::SeqCst) as usize;
    let ac = Z_.load(Ordering::SeqCst) as usize;
    
    if z == 0 || ac == 0 {
        return;
    }
    
    let aw = z * ac;
    let afz = aw * 4; 
    
    if afz > AZL_ {
        crate::serial_println!("[FB] WARNING: Framebuffer {}x{} = {} KB too large for backbuffer (max {} KB), disabling double buffer",
            z, ac, afz / 1024, AZL_ / 1024);
        return;
    }
    
    
    let mut bi = alloc::vec::Vec::new();
    if bi.jug(aw).is_err() {
        crate::serial_println!("[FB] WARNING: Failed to allocate backbuffer {} KB — OOM, desktop will use direct mode",
            afz / 1024);
        return;
    }
    bi.cmg(aw, 0u32);
    let bi = bi.dsd();
    
    *Ec.lock() = Some(bi);
    
    
    let mut lvh = alloc::vec::Vec::new();
    if lvh.jug(aw).is_ok() {
        lvh.cmg(aw, 0u32);
        *BDM_.lock() = Some(lvh.dsd());
        crate::serial_println!("[FB] Row-diff shadow buffer allocated: {} KB", afz / 1024);
    }
    
    crate::serial_println!("[FB] Double buffer allocated: {}x{} ({} KB)", z, ac, afz / 1024);
}


pub fn afi(iq: bool) {
    FX_.store(iq, Ordering::SeqCst);
}


pub fn bre() -> bool {
    FX_.load(Ordering::SeqCst)
}



pub fn cey() -> Option<(*mut u8, u32, u32, u32)> {
    let z = AB_.load(Ordering::SeqCst) as u32;
    let ac = Z_.load(Ordering::SeqCst) as u32;
    
    if z == 0 || ac == 0 {
        return None;
    }
    
    let gap = Ec.lock();
    if let Some(ref k) = *gap {
        
        let ptr = k.fq() as *mut u8;
        Some((ptr, z, ac, z)) 
    } else {
        None
    }
}







pub fn sv() {
    let ag = BJ_.load(Ordering::Relaxed);
    let z = AB_.load(Ordering::Relaxed) as usize;
    let ac = Z_.load(Ordering::Relaxed) as usize;
    let jb = CA_.load(Ordering::Relaxed) as usize;
    
    if z == 0 || ac == 0 { return; }
    
    
    
    
    if crate::drivers::virtio_gpu::anl() {
        
        let tgz = crate::drivers::virtio_gpu::kyi()
            .or_else(|| crate::drivers::virtio_gpu::iwv());
        if let Some((hlu, erl, hlt)) = tgz {
            if let Some(ref k) = *Ec.lock() {
                let aoo = z.v(erl as usize);
                let bbg = ac.v(hlt as usize);
                unsafe {
                    for c in 0..bbg {
                        let cy = k.fq().add(c * z);
                        let cs = hlu.add(c * erl as usize);
                        #[cfg(target_arch = "x86_64")]
                        crate::graphics::simd::dpd(cs, cy, aoo);
                        #[cfg(not(target_arch = "x86_64"))]
                        core::ptr::copy_nonoverlapping(cy, cs, aoo);
                    }
                }
            }
            
            let _ = crate::drivers::virtio_gpu::vku();
            
            
            return;
        }
    }
    
    
    if ag.abq() { return; }
    
    wwi(ag, z, ac, jb);
}




fn wwi(ag: *mut u8, z: usize, ac: usize, jb: usize) {
    let qob = Ec.lock();
    let mut vgw = BDM_.lock();
    
    let (aaa, ewp) = match (qob.as_ref(), vgw.as_mut()) {
        (Some(o), Some(ai)) => (o, ai),
        
        (Some(o), None) => {
            for c in 0..ac {
                let cum = c * z;
                let bgu = c * jb;
                unsafe {
                    let cy = o.fq().add(cum);
                    let cs = ag.add(bgu) as *mut u32;
                    #[cfg(target_arch = "x86_64")]
                    crate::graphics::simd::iph(cs, cy, z);
                    #[cfg(not(target_arch = "x86_64"))]
                    core::ptr::copy_nonoverlapping(cy, cs, z);
                }
            }
            return;
        }
        _ => return,
    };
    
    for c in 0..ac {
        let l = c * z;
        let ile = &aaa[l..l + z];
        let ltq = &mut ewp[l..l + z];
        
        
        let mut cpa = false;
        let dyr = ile.fq() as *const u64;
        let egn = ltq.fq() as *const u64;
        let evx = z / 2;
        
        
        let btq = evx / 8;
        let mut a = 0usize;
        unsafe {
            for _ in 0..btq {
                if *dyr.add(a) != *egn.add(a)
                    || *dyr.add(a+1) != *egn.add(a+1)
                    || *dyr.add(a+2) != *egn.add(a+2)
                    || *dyr.add(a+3) != *egn.add(a+3)
                    || *dyr.add(a+4) != *egn.add(a+4)
                    || *dyr.add(a+5) != *egn.add(a+5)
                    || *dyr.add(a+6) != *egn.add(a+6)
                    || *dyr.add(a+7) != *egn.add(a+7)
                {
                    cpa = true;
                    break;
                }
                a += 8;
            }
            
            if !cpa {
                while a < evx {
                    if *dyr.add(a) != *egn.add(a) {
                        cpa = true;
                        break;
                    }
                    a += 1;
                }
            }
            
            if !cpa && (z & 1) != 0 {
                if ile[z - 1] != ltq[z - 1] {
                    cpa = true;
                }
            }
        }
        
        if cpa {
            
            let bgu = c * jb;
            unsafe {
                let cy = ile.fq();
                let cs = ag.add(bgu) as *mut u32;
                #[cfg(target_arch = "x86_64")]
                crate::graphics::simd::iph(cs, cy, z);
                #[cfg(not(target_arch = "x86_64"))]
                core::ptr::copy_nonoverlapping(cy, cs, z);
            }
            
            ltq.dg(ile);
        }
    }
}




fn wwh(ag: *mut u8, z: usize, ac: usize, jb: usize) {
    if let Some(ref k) = *Ec.lock() {
        for c in 0..ac {
            let cum = c * z;
            let bgu = c * jb;
            unsafe {
                let cy = k.fq().add(cum);
                let cs = ag.add(bgu) as *mut u32;
                #[cfg(target_arch = "x86_64")]
                crate::graphics::simd::iph(cs, cy, z);
                #[cfg(not(target_arch = "x86_64"))]
                core::ptr::copy_nonoverlapping(cy, cs, z);
            }
        }
    }
}



pub fn nxr() -> Option<*const u32> {
    let aaa = Ec.lock();
    aaa.as_ref().map(|k| k.fq())
}



pub fn wwj() {
    let ag = BJ_.load(Ordering::Relaxed);
    if ag.abq() { return; }
    let z = AB_.load(Ordering::Relaxed) as usize;
    let ac = Z_.load(Ordering::Relaxed) as usize;
    let jb = CA_.load(Ordering::Relaxed) as usize;
    if z == 0 || ac == 0 { return; }
    wwh(ag, z, ac, jb);
}


pub fn cwe(s: u32) {
    if let Some(ref mut k) = *Ec.lock() {
        #[cfg(target_arch = "x86_64")]
        unsafe {
            crate::graphics::simd::bed(k.mw(), k.len(), s);
        }
        #[cfg(not(target_arch = "x86_64"))]
        {
            k.vi(s);
        }
    }
}






#[repr(C)]
struct Xz {
    cy: *const u32,
    cs: *mut u8,
    cid: usize,   
    noc: usize,    
    z: usize,        
}

unsafe impl Send for Xz {}
unsafe impl Sync for Xz {}


fn mzn(ay: usize, ci: usize, f: *mut u8) {
    let be = unsafe { &*(f as *const Xz) };
    for c in ay..ci {
        unsafe {
            let cy = be.cy.add(c * be.cid);
            let cs = be.cs.add(c * be.noc) as *mut u32;
            #[cfg(target_arch = "x86_64")]
            {
                crate::graphics::simd::iph(cs, cy, be.z);
            }
            #[cfg(not(target_arch = "x86_64"))]
            {
                core::ptr::copy_nonoverlapping(cy, cs, be.z);
            }
        }
    }
}









pub fn kdw(cy: *const u32, d: usize, i: usize) {
    let ag = BJ_.load(Ordering::SeqCst);
    if ag.abq() { return; }

    let gz = AB_.load(Ordering::SeqCst) as usize;
    let jb = CA_.load(Ordering::SeqCst) as usize;
    let kc = Z_.load(Ordering::SeqCst) as usize;

    let aoo = d.v(gz);
    let bbg = i.v(kc);

    let be = Xz {
        cy,
        cs: ag,
        cid: d,
        noc: jb,
        z: aoo,
    };

    
    crate::cpu::smp::daj(
        bbg,
        mzn,
        &be as *const Xz as *mut u8,
    );

    
    
    
    mzn(0, bbg, &be as *const Xz as *mut u8);
}


pub fn ah(b: u32, c: u32, d: u32, i: u32, s: u32) {
    let z = AB_.load(Ordering::SeqCst) as u32;
    let ac = Z_.load(Ordering::SeqCst) as u32;
    
    let dn = b.v(z);
    let dp = c.v(ac);
    let hy = (b + d).v(z);
    let jz = (c + i).v(ac);
    
    if hy <= dn || jz <= dp { return; }
    
    
    let ptr = HZ_.load(Ordering::Relaxed);
    if !ptr.abq() {
        let oq = KD_.load(Ordering::Relaxed) as usize;
        let gqq = (hy - dn) as usize;
        for x in dp..jz {
            let mu = x as usize * oq + dn as usize;
            #[cfg(target_arch = "x86_64")]
            unsafe {
                crate::graphics::simd::bed(
                    ptr.add(mu),
                    gqq,
                    s
                );
            }
            #[cfg(not(target_arch = "x86_64"))]
            unsafe {
                for a in 0..gqq {
                    *ptr.add(mu + a) = s;
                }
            }
        }
        return;
    }
    
    if FX_.load(Ordering::SeqCst) {
        if let Some(ref mut k) = *Ec.lock() {
            let gqq = (hy - dn) as usize;
            for x in dp..jz {
                let mu = x as usize * z as usize + dn as usize;
                if mu + gqq <= k.len() {
                    
                    #[cfg(target_arch = "x86_64")]
                    unsafe {
                        crate::graphics::simd::bed(
                            k.mw().add(mu),
                            gqq,
                            s
                        );
                    }
                    #[cfg(not(target_arch = "x86_64"))]
                    {
                        k[mu..mu + gqq].vi(s);
                    }
                }
            }
        }
    } else {
        for x in dp..jz {
            for y in dn..hy {
                sf(y, x, s);
            }
        }
    }
}


pub fn draw_pixel(b: u32, c: u32, s: u32) {
    let z = AB_.load(Ordering::SeqCst) as u32;
    let ac = Z_.load(Ordering::SeqCst) as u32;
    
    if b >= z || c >= ac { return; }
    
    
    if FX_.load(Ordering::SeqCst) {
        if let Some(ref mut k) = *Ec.lock() {
            let l = c as usize * z as usize + b as usize;
            if l < k.len() {
                k[l] = s;
            }
        }
    } else {
        let ag = BJ_.load(Ordering::SeqCst);
        if ag.abq() { return; }
        let jb = CA_.load(Ordering::SeqCst) as usize;
        let l = c as usize * jb + b as usize * 4;
        unsafe {
            let ptr = ag.add(l) as *mut u32;
            *ptr = s;
        }
    }
}


pub fn zs(b: u32, c: u32, len: u32, s: u32) {
    ah(b, c, len, 1, s);
}



pub fn ih(b: u32, c: u32, d: u32, i: u32, s: u32, dw: u32) {
    let z = AB_.load(Ordering::SeqCst) as u32;
    let ac = Z_.load(Ordering::SeqCst) as u32;
    
    let dn = b.v(z);
    let dp = c.v(ac);
    let hy = (b + d).v(z);
    let jz = (c + i).v(ac);
    if hy <= dn || jz <= dp { return; }
    
    let dw = dw.v(255);
    let wq = 255 - dw;
    let adz = (s >> 16) & 0xFF;
    let bsi = (s >> 8) & 0xFF;
    let is = s & 0xFF;
    
    
    let ptr = HZ_.load(Ordering::Relaxed);
    if !ptr.abq() {
        let oq = KD_.load(Ordering::Relaxed) as usize;
        for x in dp..jz {
            let mav = unsafe { ptr.add(x as usize * oq + dn as usize) };
            let jmy = (hy - dn) as usize;
            #[cfg(target_arch = "x86_64")]
            unsafe { crate::graphics::simd::mzl(mav, jmy, s, dw); }
            #[cfg(not(target_arch = "x86_64"))]
            for y in dn..hy {
                let w = x as usize * oq + y as usize;
                unsafe {
                    let xy = *ptr.add(w);
                    let ahh = (xy >> 16) & 0xFF;
                    let bgs = (xy >> 8) & 0xFF;
                    let ng = xy & 0xFF;
                    let m = ((adz * dw + ahh * wq + 128) >> 8).v(255);
                    let at = ((bsi * dw + bgs * wq + 128) >> 8).v(255);
                    let o = ((is * dw + ng * wq + 128) >> 8).v(255);
                    *ptr.add(w) = 0xFF000000 | (m << 16) | (at << 8) | o;
                }
            }
        }
        return;
    }
    
    
    if FX_.load(Ordering::SeqCst) {
        if let Some(ref mut k) = *Ec.lock() {
            let bjl = k.len();
            for x in dp..jz {
                let br = x as usize * z as usize;
                let mu = br + dn as usize;
                let jmy = (hy - dn) as usize;
                if mu + jmy <= bjl {
                    #[cfg(target_arch = "x86_64")]
                    unsafe { crate::graphics::simd::mzl(k.mw().add(mu), jmy, s, dw); }
                    #[cfg(not(target_arch = "x86_64"))]
                    for y in dn..hy {
                        let w = br + y as usize;
                        let xy = k[w];
                        let ahh = (xy >> 16) & 0xFF;
                        let bgs = (xy >> 8) & 0xFF;
                        let ng = xy & 0xFF;
                        let m = ((adz * dw + ahh * wq + 128) >> 8).v(255);
                        let at = ((bsi * dw + bgs * wq + 128) >> 8).v(255);
                        let o = ((is * dw + ng * wq + 128) >> 8).v(255);
                        k[w] = 0xFF000000 | (m << 16) | (at << 8) | o;
                    }
                }
            }
        }
    } else {
        for x in dp..jz {
            for y in dn..hy {
                let xy = beg(y, x);
                let ahh = (xy >> 16) & 0xFF;
                let bgs = (xy >> 8) & 0xFF;
                let ng = xy & 0xFF;
                let m = ((adz * dw + ahh * wq + 128) >> 8).v(255);
                let at = ((bsi * dw + bgs * wq + 128) >> 8).v(255);
                let o = ((is * dw + ng * wq + 128) >> 8).v(255);
                sf(y, x, 0xFF000000 | (m << 16) | (at << 8) | o);
            }
        }
    }
}


pub fn axt(b: u32, c: u32, len: u32, s: u32) {
    ah(b, c, 1, len, s);
}


pub fn lx(b: u32, c: u32, d: u32, i: u32, s: u32) {
    zs(b, c, d, s);
    zs(b, c + i - 1, d, s);
    axt(b, c, i, s);
    axt(b + d - 1, c, i, s);
}


pub fn abc(cx: u32, ae: u32, dy: u32, s: u32) {
    if dy == 0 { return; }
    
    
    let uv = (dy * dy) as i32;
    for bg in 0..=dy {
        let dx = ahn((uv - (bg * bg) as i32) as f32) as u32;
        if dx > 0 {
            
            if ae >= bg {
                ah(cx.ao(dx), ae - bg, dx * 2 + 1, 1, s);
            }
            
            ah(cx.ao(dx), ae + bg, dx * 2 + 1, 1, s);
        }
    }
}


pub fn afp(b: u32, c: u32, d: u32, i: u32, dy: u32, s: u32) {
    if d == 0 || i == 0 { return; }
    
    let m = dy.v(d / 2).v(i / 2);
    
    if m == 0 {
        ah(b, c, d, i, s);
        return;
    }
    
    
    ah(b, c + m, d, i - m * 2, s);
    
    ah(b + m, c, d - m * 2, m, s);
    
    ah(b + m, c + i - m, d - m * 2, m, s);
    
    
    let uv = (m * m) as i32;
    for bg in 0..m {
        let dx = ahn((uv - (bg * bg) as i32) as f32) as u32;
        if dx > 0 {
            
            ah(b + m - dx, c + m - bg - 1, dx, 1, s);
            
            ah(b + d - m, c + m - bg - 1, dx, 1, s);
            
            ah(b + m - dx, c + i - m + bg, dx, 1, s);
            
            ah(b + d - m, c + i - m + bg, dx, 1, s);
        }
    }
}


pub fn gtn(b: u32, c: u32, d: u32, i: u32, dy: u32, s: u32) {
    if d == 0 || i == 0 { return; }
    
    let m = dy.v(d / 2).v(i / 2);
    
    if m == 0 {
        lx(b, c, d, i, s);
        return;
    }
    
    
    zs(b + m, c, d - m * 2, s);           
    zs(b + m, c + i - 1, d - m * 2, s);   
    
    axt(b, c + m, i - m * 2, s);           
    axt(b + d - 1, c + m, i - m * 2, s);   
    
    
    let mut y = m as i32;
    let mut x = 0i32;
    let mut rq = 0i32;
    
    while y >= x {
        
        draw_pixel(b + m - y as u32, c + m - x as u32, s);
        draw_pixel(b + m - x as u32, c + m - y as u32, s);
        
        draw_pixel(b + d - 1 - m + y as u32, c + m - x as u32, s);
        draw_pixel(b + d - 1 - m + x as u32, c + m - y as u32, s);
        
        draw_pixel(b + m - y as u32, c + i - 1 - m + x as u32, s);
        draw_pixel(b + m - x as u32, c + i - 1 - m + y as u32, s);
        
        draw_pixel(b + d - 1 - m + y as u32, c + i - 1 - m + x as u32, s);
        draw_pixel(b + d - 1 - m + x as u32, c + i - 1 - m + y as u32, s);
        
        x += 1;
        rq += 1 + 2 * x;
        if 2 * (rq - y) + 1 > 0 {
            y -= 1;
            rq += 1 - 2 * y;
        }
    }
}


pub fn cb(text: &str, b: u32, c: u32, s: u32) {
    let mut cx = b;
    for r in text.bw() {
        afn(cx, c, r, s);
        cx += CH_ as u32;
    }
}


fn ahi(r: char, b: usize, c: usize, lp: u32, ei: u32) {
    let ka = font::ada(r);
    
    for br in 0..BN_ {
        let fs = ka[br];
        for bj in 0..CH_ {
            let s = if (fs >> (7 - bj)) & 1 == 1 { lp } else { ei };
            sf((b + bj) as u32, (c + br) as u32, s);
        }
    }
}



pub fn afn(b: u32, c: u32, r: char, s: u32) {
    let ka = font::ada(r);
    let z = AB_.load(Ordering::SeqCst) as u32;
    let ac = Z_.load(Ordering::SeqCst) as u32;
    
    if b >= z || c >= ac { return; }
    
    
    if FX_.load(Ordering::SeqCst) {
        if let Some(ref mut k) = *Ec.lock() {
            let oq = z as usize;
            for br in 0..BN_ {
                let x = c as usize + br;
                if x >= ac as usize { break; }
                let fs = ka[br];
                let afg = x * oq;
                for bj in 0..CH_ {
                    if (fs >> (7 - bj)) & 1 == 1 {
                        let y = b as usize + bj;
                        if y < z as usize && iob(y as u32, x as u32) {
                            let l = afg + y;
                            if l < k.len() {
                                k[l] = s;
                            }
                        }
                    }
                }
            }
        }
    } else {
        
        let ag = BJ_.load(Ordering::SeqCst);
        if ag.abq() { return; }
        let jb = CA_.load(Ordering::SeqCst) as usize;
        for br in 0..BN_ {
            let x = c as usize + br;
            if x >= ac as usize { break; }
            let fs = ka[br];
            for bj in 0..CH_ {
                if (fs >> (7 - bj)) & 1 == 1 {
                    let y = b as usize + bj;
                    if y < z as usize && iob(y as u32, x as u32) {
                        let l = x * jb + y * 4;
                        unsafe {
                            let ptr = ag.add(l) as *mut u32;
                            *ptr = s;
                        }
                    }
                }
            }
        }
    }
}


fn write_char(r: char) {
    let z = AB_.load(Ordering::SeqCst) as usize;
    let ac = Z_.load(Ordering::SeqCst) as usize;
    
    if z == 0 || ac == 0 {
        return;
    }
    
    let ec = z / CH_;
    let lk = ac / BN_;
    
    let mut console = Fk.lock();
    let lp = console.axw;
    let ei = console.vp;
    
    
    if WU_.load(Ordering::SeqCst) {
        if let Some(ref mut bsf) = *Ny.lock() {
            if !bsf.eti {
                bsf.voh(r, lp, ei);
            }
        }
    }
    
    match r {
        '\x08' => {
            if console.lf > 0 {
                console.lf -= 1;
            } else if console.ot > 0 {
                console.ot -= 1;
                console.lf = ec.ao(1);
            } else {
                return;
            }

            let y = console.lf * CH_;
            let x = console.ot * BN_;
            let lp = console.axw;
            let ei = console.vp;
            drop(console);
            ahi(' ', y, x, lp, ei);
            return;
        }
        '\n' => {
            console.lf = 0;
            console.ot += 1;
        }
        '\r' => {
            console.lf = 0;
        }
        '\t' => {
            console.lf = (console.lf + 4) & !3;
        }
        _ => {
            if console.lf >= ec {
                console.lf = 0;
                console.ot += 1;
            }
            
            let y = console.lf * CH_;
            let x = console.ot * BN_;
            
            
            let b = console.lf;
            console.lf += 1;
            drop(console);
            
            ahi(r, y, x, lp, ei);
            return;
        }
    }
    
    
    if console.ot >= lk {
        drop(console);
        dlm();
        Fk.lock().ot = lk - 1;
    }
}


pub fn dlm() {
    let ag = BJ_.load(Ordering::SeqCst);
    if ag.abq() {
        return;
    }
    
    let ac = Z_.load(Ordering::SeqCst) as usize;
    let jb = CA_.load(Ordering::SeqCst) as usize;
    let vp = Fk.lock().vp;
    
    
    for c in BN_..ac {
        unsafe {
            let cy = ag.add(c * jb);
            let cs = ag.add((c - BN_) * jb);
            core::ptr::bdu(cy, cs, jb);
        }
    }
    
    
    for c in (ac - BN_)..ac {
        let br = unsafe { ag.add(c * jb) };
        for b in 0..(jb / 4) {
            unsafe {
                br.add(b * 4).cast::<u32>().write_volatile(vp);
            }
        }
    }
}


pub struct Bau;

impl fmt::Write for Bau {
    fn write_str(&mut self, e: &str) -> fmt::Result {
        for r in e.bw() {
            write_char(r);
        }
        Ok(())
    }
}



static BFJ_: core::sync::atomic::AtomicBool =
    core::sync::atomic::AtomicBool::new(false);


pub fn pjf(ea: bool) {
    BFJ_.store(ea, core::sync::atomic::Ordering::Relaxed);
}


#[doc(hidden)]
pub fn elt(n: fmt::Arguments) {
    use core::fmt::Write;
    
    if crate::shell::edu() {
        let mut e = alloc::string::String::new();
        let _ = core::fmt::write(&mut e, n);
        crate::shell::qwj(&e);
        return;
    }
    if !BFJ_.load(core::sync::atomic::Ordering::Relaxed) {
        Bau.write_fmt(n).unwrap();
    }
    crate::serial::elt(n);
}



#[doc(hidden)]
pub fn qdk(n: fmt::Arguments) {
    use core::fmt::Write;
    Bau.write_fmt(n).unwrap();
}


#[macro_export]
macro_rules! print {
    ($($ji:tt)*) => {
        $crate::framebuffer::elt(format_args!($($ji)*))
    };
}


#[macro_export]
macro_rules! cgs {
    ($($ji:tt)*) => {
        $crate::framebuffer::qdk(format_args!($($ji)*))
    };
}


#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($fmt:expr) => ($crate::print!(concat!($fmt, "\n")));
    ($fmt:expr, $($ji:tt)*) => ($crate::print!(concat!($fmt, "\n"), $($ji)*));
}


pub const MG_: u32 = 0xFF000000;
pub const Q_: u32 = 0xFFFFFFFF;
pub const B_: u32 = 0xFF00FF00;
pub const G_: u32 = 0xFF00FF66;
pub const AU_: u32 = 0xFF00AA00;
pub const A_: u32 = 0xFFFF0000;
pub const CD_: u32 = 0xFF0000FF;
pub const D_: u32 = 0xFFFFFF00;
pub const C_: u32 = 0xFF00FFFF;
pub const DF_: u32 = 0xFFFF00FF;
pub const L_: u32 = 0xFF888888;


#[macro_export]
macro_rules! gr {
    ($s:expr, $($ji:tt)*) => {{
        let aft = $crate::framebuffer::hlh();
        $crate::framebuffer::dbv($s);
        $crate::print!($($ji)*);
        $crate::framebuffer::dbv(aft);
    }};
}


#[macro_export]
macro_rules! h {
    ($s:expr, $($ji:tt)*) => {{
        let aft = $crate::framebuffer::hlh();
        $crate::framebuffer::dbv($s);
        $crate::println!($($ji)*);
        $crate::framebuffer::dbv(aft);
    }};
}


pub fn hlh() -> u32 {
    Fk.lock().axw
}


pub fn meo() {
    let mut console = Fk.lock();
    console.axw = B_;
    console.vp = MG_;
}


pub fn zmq() {
    let mut console = Fk.lock();
    console.axw = Q_;
    console.vp = MG_;
}




pub fn ri(text: &str, b: u32, c: u32, lp: u32, ei: u32) {
    for (a, r) in text.bw().cf() {
        let y = b + (a as u32) * CH_ as u32;
        ahi(r, y as usize, c as usize, lp, ei);
    }
}


pub fn np(text: &str, c: u32, lp: u32) {
    let (z, _) = yn();
    let idh = text.len() as u32 * CH_ as u32;
    let b = (z.ao(idh)) / 2;
    ri(text, b, c, lp, MG_);
}


pub fn krj(c: u32, s: u32) {
    let (z, _) = yn();
    let adf = z / 10;
    zs(adf, c, z - 2 * adf, s);
}



pub fn ndd(br: usize) {
    let z = AB_.load(Ordering::SeqCst) as usize;
    let ac = Z_.load(Ordering::SeqCst) as usize;
    if z == 0 || ac == 0 { return; }
    let x = br * BN_;
    if x + BN_ > ac { return; }
    let ei = Fk.lock().vp;
    ah(0, x as u32, z as u32, BN_ as u32, ei);
}



pub fn krm(bj: usize, br: usize, text: &str, lp: u32, ei: u32) -> usize {
    let z = AB_.load(Ordering::SeqCst) as usize;
    let ac = Z_.load(Ordering::SeqCst) as usize;
    if z == 0 || ac == 0 { return 0; }
    let ec = z / CH_;
    let x = br * BN_;
    if x + BN_ > ac { return 0; }
    let mut az = 0;
    for (a, bm) in text.bw().cf() {
        let r = bj + a;
        if r >= ec { break; }
        ahi(bm, r * CH_, x, lp, ei);
        az += 1;
    }
    az
}


pub fn bld(bj: usize, br: usize) {
    let mut console = Fk.lock();
    console.lf = bj;
    console.ot = br;
}


pub fn gia() -> (usize, usize) {
    let console = Fk.lock();
    (console.lf, console.ot)
}


pub fn gfh(b: u32, c: u32, z: u32, li: u32, lp: u32, ei: u32) {
    let adu = (z * li.v(100)) / 100;
    
    
    ah(b, c, z, 16, ei);
    
    
    if adu > 0 {
        ah(b, c, adu, 16, lp);
    }
    
    
    lx(b, c, z, 16, lp);
}


pub fn sd(fr: &str, status: BootStatus) {
    let (ejb, s) = match status {
        BootStatus::Ok => ("[OK]", B_),
        BootStatus::Ej => ("[--]", L_),
        BootStatus::Cdj => ("[!!]", A_),
        BootStatus::V => ("[..]", C_),
    };
    
    
    let lpy = hlh();
    dbv(s);
    crate::print!("{} ", ejb);
    dbv(lpy);
    crate::println!("{}", fr);
}


#[derive(Clone, Copy)]
pub enum BootStatus {
    Ok,
    Ej,
    Cdj,
    V,
}


pub fn nmt() {
    logo::nmt();
}


pub fn led() {
    logo::led();
}


pub fn bir(ib: u32, message: &str) {
    logo::bir(ib, message);
}


pub fn kuv() {
    logo::kuv();
}


pub fn zog() {
    clear();
    meo();
    
    let (z, qce) = yn();
    
    
    krj(0, B_);
    
    
    let cjd = 16u32;
    np("╔════════════════════════════════════════════════════════════╗", cjd, G_);
    np("║                                                            ║", cjd + 16, G_);
    np("║   ████████╗██████╗ ██╗   ██╗███████╗████████╗ ██████╗ ███████╗  ║", cjd + 32, B_);
    np("║   ╚══██╔══╝██╔══██╗██║   ██║██╔════╝╚══██╔══╝██╔═══██╗██╔════╝  ║", cjd + 48, B_);
    np("║      ██║   ██████╔╝██║   ██║███████╗   ██║   ██║   ██║███████╗  ║", cjd + 64, G_);
    np("║      ██║   ██╔══██╗██║   ██║╚════██║   ██║   ██║   ██║╚════██║  ║", cjd + 80, B_);
    np("║      ██║   ██║  ██║╚██████╔╝███████║   ██║   ╚██████╔╝███████║  ║", cjd + 96, G_);
    np("║      ╚═╝   ╚═╝  ╚═╝ ╚═════╝ ╚══════╝   ╚═╝    ╚═════╝ ╚══════╝  ║", cjd + 112, B_);
    np("║                                                            ║", cjd + 128, G_);
    np("║            FAST  •  SECURE  •  RELIABLE                    ║", cjd + 144, AU_);
    np("║                                                            ║", cjd + 160, G_);
    np("╚════════════════════════════════════════════════════════════╝", cjd + 176, G_);
    
    
    krj(cjd + 200, B_);
    
    
    let dwe = ((cjd + 220) / 16) as usize;
    bld(0, dwe);
}


pub fn zoi() {
    clear();
    meo();
    
    crate::println!("╔══════════════════════════════════════════════════════════════╗");
    crate::println!("║                      T R U S T - O S                         ║");
    crate::println!("║                 FAST • SECURE • RELIABLE                     ║");
    crate::println!("╚══════════════════════════════════════════════════════════════╝");
    crate::println!();
}




pub fn tte() {
    let z = AB_.load(Ordering::SeqCst) as usize;
    let ac = Z_.load(Ordering::SeqCst) as usize;
    
    if z == 0 || ac == 0 {
        return;
    }
    
    let aw = z * ac;
    
    let mut bi = alloc::vec::Vec::new();
    if bi.jug(aw).is_err() {
        crate::serial_println!("[FB] WARNING: Failed to allocate background cache {} KB — OOM",
            aw * 4 / 1024);
        return;
    }
    bi.cmg(aw, 0u32);
    let bi = bi.dsd();
    
    *RH_.lock() = Some(bi);
    LV_.store(false, Ordering::SeqCst);
    crate::serial_println!("[FB] Background cache allocated: {} KB", aw * 4 / 1024);
}


pub fn zvc() {
    LV_.store(true, Ordering::SeqCst);
}


pub fn yyr() {
    LV_.store(false, Ordering::SeqCst);
}


pub fn yza() -> bool {
    LV_.load(Ordering::SeqCst)
}


pub fn zka() {
    let z = AB_.load(Ordering::SeqCst) as usize;
    
    let gax = RH_.lock();
    if let Some(ref emr) = *gax {
        if let Some(ref mut emm) = *Ec.lock() {
            
            let len = emr.len().v(emm.len());
            unsafe {
                core::ptr::copy_nonoverlapping(emr.fq(), emm.mw(), len);
            }
        }
    }
}


pub fn zjz(b: u32, c: u32, d: u32, i: u32) {
    let z = AB_.load(Ordering::SeqCst) as u32;
    let ac = Z_.load(Ordering::SeqCst) as u32;
    
    let dn = b.v(z);
    let dp = c.v(ac);
    let hy = (b + d).v(z);
    let jz = (c + i).v(ac);
    
    let gax = RH_.lock();
    if let Some(ref emr) = *gax {
        if let Some(ref mut emm) = *Ec.lock() {
            for x in dp..jz {
                let big = x as usize * z as usize + dn as usize;
                let pmq = x as usize * z as usize + hy as usize;
                let dqh = big;
                
                if pmq <= emr.len() && pmq <= emm.len() {
                    unsafe {
                        let cy = emr.fq().add(big);
                        let cs = emm.mw().add(dqh);
                        core::ptr::copy_nonoverlapping(cy, cs, (hy - dn) as usize);
                    }
                }
            }
        }
    }
}


pub fn ynn<G: FnOnce()>(sda: G) {
    
    sda();
    
    
    qvg();
}


pub fn qvg() {
    
    let z = AB_.load(Ordering::SeqCst) as usize;
    let ac = Z_.load(Ordering::SeqCst) as usize;
    let aw = z * ac;
    
    if aw == 0 { return; }
    
    
    
    let mut gax = RH_.lock();
    let mxj = Ec.lock();
    
    if let (Some(ref mut emr), Some(ref emm)) = (&mut *gax, &*mxj) {
        let len = emm.len().v(emr.len());
        unsafe {
            core::ptr::copy_nonoverlapping(emm.fq(), emr.mw(), len);
        }
    }
    
    drop(mxj);
    drop(gax);
    
    LV_.store(true, Ordering::SeqCst);
}




pub fn fzt(b: u32, c: u32, d: u32, i: u32) {
    MT_.lock().add(DirtyRect::new(b, c, d, i));
}


pub fn olc() {
    MT_.lock().olc();
}


pub fn ndf() {
    MT_.lock().clear();
}


pub fn bex() -> bool {
    MT_.lock().asw
}


pub fn tdj() -> ([DirtyRect; GQ_], usize, bool) {
    let adb = MT_.lock();
    (adb.akn, adb.az, adb.asw)
}


pub fn zqg() {
    let ag = BJ_.load(Ordering::SeqCst);
    if ag.abq() {
        return;
    }
    
    let z = AB_.load(Ordering::SeqCst) as usize;
    let ac = Z_.load(Ordering::SeqCst) as usize;
    let jb = CA_.load(Ordering::SeqCst) as usize;
    
    let (akn, az, asw) = tdj();
    
    if asw {
        
        sv();
        ndf();
        return;
    }
    
    if let Some(ref k) = *Ec.lock() {
        for a in 0..az {
            let ha = &akn[a];
            if !ha.cld() { continue; }
            
            let dn = (ha.b as usize).v(z);
            let dp = (ha.c as usize).v(ac);
            let hy = ((ha.b + ha.d) as usize).v(z);
            let jz = ((ha.c + ha.i) as usize).v(ac);
            
            for c in dp..jz {
                let cum = c * z + dn;
                let bgu = c * jb / 4 + dn; 
                let len = hy - dn;
                
                unsafe {
                    let cy = k.fq().add(cum);
                    let cs = (ag as *mut u32).add(c * jb / 4 + dn);
                    core::ptr::copy_nonoverlapping(cy, cs, len);
                }
            }
        }
    }
    
    ndf();
}


pub fn ysz() -> usize {
    Fk.lock().ot
}


pub fn ysy() -> usize {
    Fk.lock().lf
}


pub fn wev(ak: usize) {
    if !WU_.load(Ordering::SeqCst) {
        return;
    }
    
    let z = AB_.load(Ordering::SeqCst) as usize;
    let ac = Z_.load(Ordering::SeqCst) as usize;
    if z == 0 || ac == 0 {
        return;
    }
    
    let bpd = ac / BN_;
    
    let mut bsf = Ny.lock();
    if let Some(ref mut is) = *bsf {
        let aye = is.bss().ao(bpd);
        is.px = (is.px + ak).v(aye);
        is.eti = is.px > 0;
        
        if is.eti {
            
            let l = is.px;
            let es = is.bss();
            drop(bsf);
            lyj(l, es, bpd);
        }
    }
}


pub fn eid(ak: usize) {
    if !WU_.load(Ordering::SeqCst) {
        return;
    }
    
    let z = AB_.load(Ordering::SeqCst) as usize;
    let ac = Z_.load(Ordering::SeqCst) as usize;
    if z == 0 || ac == 0 {
        return;
    }
    
    let bpd = ac / BN_;
    
    let mut bsf = Ny.lock();
    if let Some(ref mut is) = *bsf {
        is.px = is.px.ao(ak);
        is.eti = is.px > 0;
        
        let l = is.px;
        let es = is.bss();
        drop(bsf);
        
        lyj(l, es, bpd);
    }
}


pub fn weu() {
    if let Some(ref mut is) = *Ny.lock() {
        is.px = 0;
        is.eti = false;
    }
}



pub fn pcw() -> (usize, usize) {
    let es = {
        let mut bsf = Ny.lock();
        if let Some(ref mut is) = *bsf {
            is.px = 0;
            is.eti = false;
            is.bss()
        } else {
            return (0, 0);
        }
    };

    let ac = Z_.load(Ordering::SeqCst) as usize;
    if ac == 0 {
        return (0, 0);
    }
    let bpd = ac / BN_;
    lyj(0, es, bpd);

    let console = Fk.lock();
    (console.lf, console.ot)
}


pub fn lgl() -> bool {
    if let Some(ref is) = *Ny.lock() {
        is.eti
    } else {
        false
    }
}


pub fn ytu() -> (usize, usize) {
    if let Some(ref is) = *Ny.lock() {
        (is.px, is.bss())
    } else {
        (0, 0)
    }
}


fn lyj(px: usize, bss: usize, bpd: usize) {
    let z = AB_.load(Ordering::SeqCst) as usize;
    let ec = z / CH_;
    
    
    let ei = Fk.lock().vp;
    rbi(ei);
    
    
    
    
    
    
    
    
    let tpa = if px == 0 {
        bpd.ao(1)
    } else {
        bpd
    };
    let pob = bss.ao(tpa + px);
    let nqa = bss.ao(px);
    
    let mut lja = 0usize;
    let mut ljb = 0usize;
    let mut lbc = false;
    
    let bsf = Ny.lock();
    if let Some(ref is) = *bsf {
        for (dvl, atd) in (pob..nqa).cf() {
            if atd >= is.ak.len() {
                continue;
            }
            let line = &is.ak[atd];
            for (bj, a) in (0..line.len.v(ec)).cf() {
                let r = line.bw[a];
                let (lp, ei) = line.colors[a];
                let y = bj * CH_;
                let x = dvl * BN_;
                ahi(r, y, x, lp, ei);
            }
        }
        
        
        
        if px == 0 {
            let dvl = nqa.ao(pob);
            if dvl < bpd && is.bgp.len > 0 {
                for bj in 0..is.bgp.len.v(ec) {
                    let r = is.bgp.bw[bj];
                    let (lp, ei) = is.bgp.colors[bj];
                    let y = bj * CH_;
                    let x = dvl * BN_;
                    ahi(r, y, x, lp, ei);
                }
                ljb = dvl;
                lja = is.bgp.len.v(ec);
                lbc = true;
            } else if dvl < bpd {
                
                ljb = dvl;
                lja = 0;
                lbc = true;
            }
        }
    }
    drop(bsf);
    
    
    if px == 0 && lbc {
        let mut console = Fk.lock();
        console.ot = ljb;
        console.lf = lja;
    }
    
    
    if px > 0 {
        let oec = format!("-- SCROLL: +{} lines --", px);
        let bii = ec.ao(oec.len() + 2);
        for (a, bm) in oec.bw().cf() {
            let y = (bii + a) * CH_;
            ahi(bm, y, 0, 0xFFFFFF00, 0xFF000000); 
        }
    }
}


fn rbi(s: u32) {
    let ag = BJ_.load(Ordering::SeqCst);
    if ag.abq() {
        return;
    }
    
    let ac = Z_.load(Ordering::SeqCst) as usize;
    let jb = CA_.load(Ordering::SeqCst) as usize;
    
    for c in 0..ac {
        let br = unsafe { ag.add(c * jb) };
        for b in 0..(jb / 4) {
            unsafe {
                br.add(b * 4).cast::<u32>().write_volatile(s);
            }
        }
    }
}
