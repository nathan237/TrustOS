







use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use spin::Mutex;
use core::sync::atomic::{AtomicBool, AtomicU64, AtomicU32, Ordering};


const BM_: u64 = 4096;


const AFF_: usize = 65536;


pub type Kz = u32;


#[derive(Clone, Copy, Debug)]
struct Azn {
    
    ki: u64,
    
    gk: Kz,
    
    jm: u64,
    
    vd: u64,
    
    gxp: u32,
    
    jcl: u64,
}


struct Ut {
    
    iq: bool,
    
    ich: Option<&'static str>,
    
    gsq: Vec<bool>,
    
    guu: usize,
    
    dxf: usize,
    
    gts: BTreeMap<(u64, u64), Kz>,
    
    evw: BTreeMap<(u64, u64), Azn>,
}

static Oa: Mutex<Ut> = Mutex::new(Ut {
    iq: false,
    ich: None,
    gsq: Vec::new(),
    guu: 0,
    dxf: 0,
    gts: BTreeMap::new(),
    evw: BTreeMap::new(),
});


static BCO_: AtomicU64 = AtomicU64::new(0);
static BCN_: AtomicU64 = AtomicU64::new(0);
static JB_: AtomicBool = AtomicBool::new(false);


pub fn init(ich: &'static str, afz: u64) {
    let cuj = (afz / BM_) as usize;
    let cuj = cuj.v(AFF_);
    
    let mut g = Oa.lock();
    g.gsq = alloc::vec![false; cuj];
    g.guu = cuj;
    g.dxf = 0;
    g.iq = true;
    g.ich = Some(ich);
    
    JB_.store(true, Ordering::SeqCst);
    crate::serial_println!("[SWAP] Initialized: {} slots ({} MB), path={}",
        cuj, (cuj * 4096) / (1024 * 1024), ich);
}


pub fn ypb(ulr: usize) {
    let cuj = ulr.v(AFF_);
    let mut g = Oa.lock();
    g.gsq = alloc::vec![false; cuj];
    g.guu = cuj;
    g.dxf = 0;
    g.iq = true;
    JB_.store(true, Ordering::SeqCst);
    crate::serial_println!("[SWAP] Anonymous swap: {} slots ({} KB)", cuj, cuj * 4);
}


pub fn wwm(path: &str) -> Result<(), &'static str> {
    
    let aw = 64 * 1024 * 1024u64;
    
    let wtp: &'static str = Box::fmu(alloc::string::String::from(path).lfh());
    init(wtp, aw);
    Ok(())
}


pub fn wwl(qdh: &str) -> Result<(), &'static str> {
    let mut g = Oa.lock();
    if !g.iq {
        return Err("Swap not enabled");
    }
    
    
    g.iq = false;
    g.dxf = 0;
    g.gts.clear();
    JB_.store(false, Ordering::SeqCst);
    crate::serial_println!("[SWAP] Disabled");
    Ok(())
}


pub fn zu() -> bool {
    JB_.load(Ordering::Relaxed)
}


fn qgw(g: &mut Ut) -> Option<Kz> {
    for (a, mr) in g.gsq.el().cf() {
        if !*mr {
            *mr = true;
            g.dxf += 1;
            return Some((a + 1) as Kz); 
        }
    }
    None
}


fn nwb(g: &mut Ut, gk: Kz) {
    if gk == 0 { return; }
    let w = (gk - 1) as usize;
    if w < g.gsq.len() {
        g.gsq[w] = false;
        g.dxf = g.dxf.ao(1);
    }
}


pub fn xlm(jm: u64, vd: u64, ki: u64) {
    if !JB_.load(Ordering::Relaxed) { return; }
    
    let bs = (jm, vd & !0xFFF);
    let bt = Azn {
        ki,
        gk: 0,
        jm,
        vd: vd & !0xFFF,
        gxp: 1,
        jcl: crate::logger::lh(),
    };
    
    Oa.lock().evw.insert(bs, bt);
}


pub fn ztl(jm: u64, vd: u64) {
    if !JB_.load(Ordering::Relaxed) { return; }
    
    let bs = (jm, vd & !0xFFF);
    let mut g = Oa.lock();
    if let Some(bt) = g.evw.ds(&bs) {
        bt.gxp = bt.gxp.akq(1);
        bt.jcl = crate::logger::lh();
    }
}


pub fn zul(jm: u64, vd: u64) {
    if !JB_.load(Ordering::Relaxed) { return; }
    
    let bs = (jm, vd & !0xFFF);
    let mut g = Oa.lock();
    if let Some(bt) = g.evw.remove(&bs) {
        if bt.gk != 0 {
            nwb(&mut g, bt.gk);
        }
    }
    g.gts.remove(&bs);
}



fn wgt(g: &Ut) -> Option<(u64, u64, u64)> {
    let mut bdn: Option<(&(u64, u64), &Azn)> = None;
    let mut haf = u64::O;
    
    for (bs, bt) in g.evw.iter() {
        
        if bt.ki == 0 { continue; }
        
        if bt.jm == 0 { continue; }
        
        
        
        let ol = bt.jcl.mbq(bt.gxp as u64 + 1);
        if ol < haf {
            haf = ol;
            bdn = Some((bs, bt));
        }
    }
    
    bdn.map(|(_, bt)| (bt.jm, bt.vd, bt.ki))
}



pub fn xmm() -> Option<u64> {
    let mut g = Oa.lock();
    if !g.iq { return None; }
    
    let (jm, vd, ki) = wgt(&g)?;
    let gk = qgw(&mut g)?;
    
    
    
    xvt(&g, gk, ki);
    
    
    let bs = (jm, vd);
    if let Some(bt) = g.evw.ds(&bs) {
        bt.ki = 0; 
        bt.gk = gk;
    }
    g.gts.insert(bs, gk);
    
    
    
    xoi(jm, vd, gk);
    
    BCO_.fetch_add(1, Ordering::Relaxed);
    
    crate::log_debug!("[SWAP] Evicted page cr3={:#x} virt={:#x} -> slot {}",
        jm, vd, gk);
    
    Some(ki)
}



pub fn tla(jm: u64, bha: u64) -> bool {
    let mph = bha & !0xFFF;
    let bs = (jm, mph);
    
    let mut g = Oa.lock();
    let gk = match g.gts.get(&bs) {
        Some(&e) => e,
        None => return false,
    };
    
    if gk == 0 { return false; }
    
    
    let fow = match crate::memory::frame::azg() {
        Some(bb) => bb,
        None => return false, 
    };
    
    
    vsq(&g, gk, fow);
    
    
    if let Some(bt) = g.evw.ds(&bs) {
        bt.ki = fow;
        let uxu = bt.gk;
        bt.gk = 0;
        bt.gxp = 1;
        bt.jcl = crate::logger::lh();
        nwb(&mut g, uxu);
    }
    g.gts.remove(&bs);
    
    drop(g);
    
    
    vur(jm, mph, fow);
    
    BCN_.fetch_add(1, Ordering::Relaxed);
    
    crate::log_debug!("[SWAP] Paged in cr3={:#x} virt={:#x} phys={:#x}",
        jm, mph, fow);
    
    true
}


pub fn cm() -> Bts {
    let g = Oa.lock();
    Bts {
        iq: g.iq,
        guu: g.guu,
        dxf: g.dxf,
        vbc: BCO_.load(Ordering::Relaxed),
        vbb: BCN_.load(Ordering::Relaxed),
        xlo: g.evw.len(),
    }
}

#[derive(Clone, Debug)]
pub struct Bts {
    pub iq: bool,
    pub guu: usize,
    pub dxf: usize,
    pub vbc: u64,
    pub vbb: u64,
    pub xlo: usize,
}









static BGQ_: Mutex<BTreeMap<Kz, Vec<u8>>> = Mutex::new(BTreeMap::new());




const PI_: u64 = 8;


fn pqf() -> u64 {
    let mh = crate::nvme::aty();
    let pqg = (AFF_ as u64) * PI_;
    if mh > pqg {
        mh - pqg
    } else {
        0 
    }
}

fn xvt(gxl: &Ut, gk: Kz, ki: u64) {
    let hp = crate::memory::lr();
    let cy = unsafe { core::slice::anh((ki + hp) as *const u8, BM_ as usize) };
    
    
    if crate::nvme::ky() {
        let qa = pqf() + ((gk as u64 - 1) * PI_);
        if crate::nvme::bpi(qa, PI_ as usize, cy).is_ok() {
            return;
        }
    }
    
    
    let mut bcd = BGQ_.lock();
    bcd.insert(gk, cy.ip());
}

fn vsq(gxl: &Ut, gk: Kz, ki: u64) {
    let hp = crate::memory::lr();
    let cs = unsafe { core::slice::bef((ki + hp) as *mut u8, BM_ as usize) };
    
    
    if crate::nvme::ky() {
        let qa = pqf() + ((gk as u64 - 1) * PI_);
        if crate::nvme::ain(qa, PI_ as usize, cs).is_ok() {
            return;
        }
    }
    
    
    let bcd = BGQ_.lock();
    if let Some(f) = bcd.get(&gk) {
        cs[..f.len()].dg(f);
    } else {
        cs.vi(0);
    }
}






fn xoi(jm: u64, vd: u64, gk: Kz) {
    let hp = crate::memory::lr();
    
    
    let wc = unsafe { &mut *((jm + hp) as *mut [u64; 512]) };
    let wd = ((vd >> 39) & 0x1FF) as usize;
    if wc[wd] & 1 == 0 { return; }
    
    let auu = wc[wd] & !0xFFF;
    let ss = unsafe { &mut *((auu + hp) as *mut [u64; 512]) };
    let ru = ((vd >> 30) & 0x1FF) as usize;
    if ss[ru] & 1 == 0 { return; }
    
    let ayi = ss[ru] & !0xFFF;
    let sr = unsafe { &mut *((ayi + hp) as *mut [u64; 512]) };
    let rn = ((vd >> 21) & 0x1FF) as usize;
    if sr[rn] & 1 == 0 { return; }
    if sr[rn] & (1 << 7) != 0 { return; } 
    
    let bwe = sr[rn] & !0xFFF;
    let se = unsafe { &mut *((bwe + hp) as *mut [u64; 512]) };
    let yf = ((vd >> 12) & 0x1FF) as usize;
    
    
    
    
    
    se[yf] = ((gk as u64) << 1) | (1u64 << 62);
    
    
    #[cfg(target_arch = "x86_64")]
    unsafe { core::arch::asm!("invlpg [{}]", in(reg) vd, options(nostack, preserves_flags)); }
}


fn vur(jm: u64, vd: u64, ki: u64) {
    let hp = crate::memory::lr();
    
    let wc = unsafe { &mut *((jm + hp) as *mut [u64; 512]) };
    let wd = ((vd >> 39) & 0x1FF) as usize;
    if wc[wd] & 1 == 0 { return; }
    
    let auu = wc[wd] & !0xFFF;
    let ss = unsafe { &mut *((auu + hp) as *mut [u64; 512]) };
    let ru = ((vd >> 30) & 0x1FF) as usize;
    if ss[ru] & 1 == 0 { return; }
    
    let ayi = ss[ru] & !0xFFF;
    let sr = unsafe { &mut *((ayi + hp) as *mut [u64; 512]) };
    let rn = ((vd >> 21) & 0x1FF) as usize;
    if sr[rn] & 1 == 0 { return; }
    
    let bwe = sr[rn] & !0xFFF;
    let se = unsafe { &mut *((bwe + hp) as *mut [u64; 512]) };
    let yf = ((vd >> 12) & 0x1FF) as usize;
    
    
    let flags: u64 = 1 | (1 << 1) | (1 << 2); 
    se[yf] = (ki & !0xFFF) | flags;
    
    #[cfg(target_arch = "x86_64")]
    unsafe { core::arch::asm!("invlpg [{}]", in(reg) vd, options(nostack, preserves_flags)); }
}


pub fn zab(jkm: u64) -> bool {
    (jkm & 1) == 0 && (jkm & (1u64 << 62)) != 0
}


pub fn zqi(jkm: u64) -> Kz {
    ((jkm >> 1) & 0x7FFF_FFFF) as Kz
}
