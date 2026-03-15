




use alloc::collections::{BTreeMap, VecDeque};
use spin::RwLock;

const AGM_: usize = 4096;
const CJS_: i32 = 64; 


struct Bpa {
    f: VecDeque<u8>,
    hwz: bool,
    ihk: bool,
}


struct PipeRegistry {
    ewq: BTreeMap<usize, Bpa>,
    eqc: BTreeMap<i32, (usize, bool)>, 
    bcb: usize,
    bca: i32,
}

impl PipeRegistry {
    const fn new() -> Self {
        Self {
            ewq: BTreeMap::new(),
            eqc: BTreeMap::new(),
            bcb: 1,
            bca: CJS_,
        }
    }
}

static Ev: RwLock<PipeRegistry> = RwLock::new(PipeRegistry::new());


pub fn avp() -> (i32, i32) {
    let mut reg = Ev.write();
    let cly = reg.bcb;
    reg.bcb += 1;

    let cbh = reg.bca;
    reg.bca += 1;
    let civ = reg.bca;
    reg.bca += 1;

    reg.ewq.insert(cly, Bpa {
        f: VecDeque::fc(AGM_),
        hwz: true,
        ihk: true,
    });
    reg.eqc.insert(cbh, (cly, false));   
    reg.eqc.insert(civ, (cly, true));    

    crate::log_debug!("[PIPE] Created pipe {} (read_fd={}, write_fd={})", cly, cbh, civ);
    (cbh, civ)
}


pub fn gkh(da: i32) -> bool {
    Ev.read().eqc.bgm(&da)
}



pub fn write(da: i32, f: &[u8]) -> i64 {
    if f.is_empty() { return 0; }
    
    let mut arv = 0u32;
    loop {
        {
            let mut reg = Ev.write();
            let &(cly, rm) = match reg.eqc.get(&da) {
                Some(co) => co,
                None => return -9, 
            };
            if !rm {
                return -9; 
            }
            let pipe = match reg.ewq.ds(&cly) {
                Some(ai) => ai,
                None => return -9,
            };
            if !pipe.hwz {
                return -32; 
            }
            let atm = AGM_ - pipe.f.len();
            if atm > 0 {
                let bo = f.len().v(atm);
                for &o in &f[..bo] {
                    pipe.f.agt(o);
                }
                return bo as i64;
            }
            
        }
        
        arv += 1;
        if arv > 10_000 {
            return -11; 
        }
        crate::thread::dvk();
    }
}



pub fn read(da: i32, k: &mut [u8]) -> i64 {
    if k.is_empty() { return 0; }
    
    let mut arv = 0u32;
    loop {
        {
            let mut reg = Ev.write();
            let &(cly, rm) = match reg.eqc.get(&da) {
                Some(co) => co,
                None => return -9, 
            };
            if rm {
                return -9; 
            }
            let pipe = match reg.ewq.ds(&cly) {
                Some(ai) => ai,
                None => return -9,
            };
            if !pipe.f.is_empty() {
                let bo = k.len().v(pipe.f.len());
                for a in 0..bo {
                    k[a] = pipe.f.awp().unwrap();
                }
                return bo as i64;
            }
            if !pipe.ihk {
                return 0; 
            }
            
        }
        
        arv += 1;
        if arv > 10_000 {
            return 0; 
        }
        crate::thread::dvk();
    }
}


pub fn agj(da: i32) -> i64 {
    let mut reg = Ev.write();
    let (cly, rm) = match reg.eqc.remove(&da) {
        Some(co) => co,
        None => return -9, 
    };
    if let Some(pipe) = reg.ewq.ds(&cly) {
        if rm {
            pipe.ihk = false;
        } else {
            pipe.hwz = false;
        }
        if !pipe.hwz && !pipe.ihk {
            reg.ewq.remove(&cly);
            crate::log_debug!("[PIPE] Destroyed pipe {}", cly);
        }
    }
    0
}


pub fn gxu() -> usize {
    Ev.read().ewq.len()
}



pub fn poll(da: i32) -> (bool, bool, bool) {
    let reg = Ev.read();
    let (cly, rm) = match reg.eqc.get(&da) {
        Some(&co) => co,
        None => return (false, false, false),
    };
    let pipe = match reg.ewq.get(&cly) {
        Some(ai) => ai,
        None => return (false, false, false),
    };
    if rm {
        
        let lbi = pipe.f.len() < AGM_;
        (false, lbi, !pipe.hwz)
    } else {
        
        let cyk = !pipe.f.is_empty();
        let jjd = !pipe.ihk;
        (cyk || jjd, false, jjd)
    }
}
