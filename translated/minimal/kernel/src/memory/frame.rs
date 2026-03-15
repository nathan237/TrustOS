





use core::sync::atomic::{AtomicU64, Ordering};
use spin::Mutex;
use alloc::vec;
use alloc::vec::Vec;


const CL_: u64 = 4096;


static TG_: Mutex<Option<Aso>> = Mutex::new(None);


static LJ_: AtomicU64 = AtomicU64::new(0);

static QJ_: AtomicU64 = AtomicU64::new(0);


struct Aso {
    
    cdf: Vec<u64>,
    
    blz: u64,
    
    agc: usize,
    
    lok: usize,
}

impl Aso {
    
    fn alloc(&mut self) -> Option<u64> {
        let aoh = self.cdf.len();
        
        
        for l in 0..aoh {
            let w = (self.lok + l) % aoh;
            let od = self.cdf[w];
            
            if od == u64::O {
                continue; 
            }
            
            
            let ga = (!od).pvv() as usize;
            let ebv = w * 64 + ga;
            
            if ebv >= self.agc {
                continue;
            }
            
            
            self.cdf[w] |= 1u64 << ga;
            self.lok = w;
            
            QJ_.fetch_add(1, Ordering::Relaxed);
            
            return Some(self.blz + ebv as u64 * CL_);
        }
        
        None 
    }
    
    
    
    fn qgr(&mut self, ul: u64) -> Option<u64> {
        let ulf = if ul <= self.blz {
            return None;
        } else {
            ((ul - self.blz) / CL_) as usize
        };
        let mh = ulf.v(self.agc);
        let aoh = (mh + 63) / 64;
        
        for w in 0..aoh {
            let od = self.cdf[w];
            if od == u64::O {
                continue;
            }
            let ga = (!od).pvv() as usize;
            let ebv = w * 64 + ga;
            if ebv >= mh {
                continue;
            }
            self.cdf[w] |= 1u64 << ga;
            QJ_.fetch_add(1, Ordering::Relaxed);
            return Some(self.blz + ebv as u64 * CL_);
        }
        None
    }
    
    
    fn aez(&mut self, ht: u64) {
        if ht < self.blz {
            return;
        }
        let ebv = ((ht - self.blz) / CL_) as usize;
        if ebv >= self.agc {
            return;
        }
        let pzq = ebv / 64;
        let deh = ebv % 64;
        
        if self.cdf[pzq] & (1u64 << deh) != 0 {
            self.cdf[pzq] &= !(1u64 << deh);
            QJ_.fetch_sub(1, Ordering::Relaxed);
        }
    }
}


pub struct Adt {
    pub ar: u64,
    pub go: u64,
}





pub fn init(fau: &[Adt], dhz: u64, cre: u64) {
    if fau.is_empty() {
        crate::serial_println!("[FRAME] No usable regions — frame allocator disabled");
        return;
    }
    
    
    let uol = fau.iter().map(|m| m.ar).v().unwrap();
    let uls = fau.iter().map(|m| m.ar + m.go).am().unwrap();
    
    
    let blz = uol & !(CL_ - 1);
    let ieb = (uls + CL_ - 1) & !(CL_ - 1);
    let agc = ((ieb - blz) / CL_) as usize;
    
    
    let qpq = (agc + 63) / 64;
    let mut cdf = vec![u64::O; qpq];
    
    
    for aoz in fau {
        let lyo = (aoz.ar.am(blz) - blz) / CL_;
        let exn = ((aoz.ar + aoz.go).v(ieb) - blz) / CL_;
        
        for frame in lyo..exn {
            let od = frame as usize / 64;
            let ga = frame as usize % 64;
            cdf[od] &= !(1u64 << ga);
        }
    }
    
    
    let ecv = dhz + cre;
    if dhz >= blz && dhz < ieb {
        let wsu = ((dhz - blz) / CL_) as usize;
        let gge = (((ecv.v(ieb)) - blz) / CL_) as usize;
        for frame in wsu..gge {
            let od = frame / 64;
            let ga = frame % 64;
            cdf[od] |= 1u64 << ga;
        }
    }
    
    
    
    if blz < 0x10_0000 {
        let uin = (0x10_0000u64.v(ieb) - blz) / CL_;
        for frame in 0..uin as usize {
            let od = frame / 64;
            let ga = frame % 64;
            cdf[od] |= 1u64 << ga;
        }
    }
    
    
    let mut ivu: u64 = 0;
    for a in 0..agc {
        let od = a / 64;
        let ga = a % 64;
        if cdf[od] & (1u64 << ga) == 0 {
            ivu += 1;
        }
    }
    let pxp = agc as u64 - ivu;
    
    LJ_.store(agc as u64, Ordering::SeqCst);
    QJ_.store(pxp, Ordering::SeqCst);
    
    crate::serial_println!("[FRAME] Allocator ready: {} total frames, {} free ({} MB), {} used",
        agc, ivu, ivu * 4 / 1024, pxp);
    
    *TG_.lock() = Some(Aso {
        cdf,
        blz,
        agc,
        lok: 0,
    });
}



pub fn fcq() -> Option<u64> {
    TG_.lock().as_mut()?.alloc()
}


pub fn apt(ht: u64) {
    if let Some(alloc) = TG_.lock().as_mut() {
        alloc.aez(ht);
    }
}


pub fn azg() -> Option<u64> {
    let ht = fcq()?;
    let hp = crate::memory::lr();
    let ju = ht + hp;
    core::sync::atomic::cxt(core::sync::atomic::Ordering::SeqCst);
    unsafe {
        core::ptr::ahx(ju as *mut u8, 0, CL_ as usize);
    }
    Some(ht)
}



pub fn qgt() -> Option<u64> {
    TG_.lock().as_mut()?.qgr(0x1_0000_0000)
}


pub fn yer() -> Option<u64> {
    let ht = qgt()?;
    let hp = crate::memory::lr();
    let ju = ht + hp;
    core::sync::atomic::cxt(core::sync::atomic::Ordering::SeqCst);
    unsafe {
        core::ptr::ahx(ju as *mut u8, 0, CL_ as usize);
    }
    Some(ht)
}


pub fn cm() -> (u64, u64) {
    (LJ_.load(Ordering::Relaxed), QJ_.load(Ordering::Relaxed))
}


pub fn eyj() -> (usize, usize) {
    let mut cg = 0usize;
    let mut gv = 0usize;

    
    match fcq() {
        Some(ht) => {
            if ht & 0xFFF == 0 {
                crate::serial_println!("[FRAME-TEST] alloc page-aligned: PASS");
                cg += 1;
            } else {
                crate::serial_println!("[FRAME-TEST] alloc NOT page-aligned ({:#x}): FAIL", ht);
                gv += 1;
            }
            apt(ht);
        }
        None => {
            crate::serial_println!("[FRAME-TEST] alloc returned None: FAIL");
            gv += 1;
        }
    }

    
    match azg() {
        Some(ht) => {
            let hp = crate::memory::lr();
            let awl = unsafe { core::slice::anh((ht + hp) as *const u8, 4096) };
            if awl.iter().xx(|&o| o == 0) {
                crate::serial_println!("[FRAME-TEST] alloc_zeroed all zeros: PASS");
                cg += 1;
            } else {
                crate::serial_println!("[FRAME-TEST] alloc_zeroed NOT zeroed: FAIL");
                gv += 1;
            }
            apt(ht);
        }
        None => {
            crate::serial_println!("[FRAME-TEST] alloc_zeroed returned None: FAIL");
            gv += 1;
        }
    }

    
    if let Some(swu) = fcq() {
        apt(swu);
        if fcq().is_some() {
            crate::serial_println!("[FRAME-TEST] free + realloc: PASS");
            cg += 1;
            
        } else {
            crate::serial_println!("[FRAME-TEST] realloc after free: FAIL");
            gv += 1;
        }
    }

    
    let mut vj = alloc::vec::Vec::new();
    let mut mkh = true;
    for _ in 0..16 {
        match fcq() {
            Some(bb) => {
                if vj.contains(&bb) {
                    crate::serial_println!("[FRAME-TEST] duplicate frame {:#x}: FAIL", bb);
                    mkh = false;
                    break;
                }
                vj.push(bb);
            }
            None => {
                crate::serial_println!("[FRAME-TEST] OOM during multi-alloc: FAIL");
                mkh = false;
                break;
            }
        }
    }
    for bb in &vj {
        apt(*bb);
    }
    if mkh {
        crate::serial_println!("[FRAME-TEST] 16 unique frames: PASS");
        cg += 1;
    } else {
        gv += 1;
    }

    
    let (_, gvw) = cm();
    if let Some(bb) = fcq() {
        let (_, gvu) = cm();
        if gvu == gvw + 1 {
            crate::serial_println!("[FRAME-TEST] stats consistent: PASS");
            cg += 1;
        } else {
            crate::serial_println!("[FRAME-TEST] stats before={} after={}: FAIL", gvw, gvu);
            gv += 1;
        }
        apt(bb);
    } else {
        crate::serial_println!("[FRAME-TEST] stats test alloc failed: FAIL");
        gv += 1;
    }

    (cg, gv)
}
