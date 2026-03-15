








use alloc::vec::Vec;
use spin::Mutex;
use core::sync::atomic::{AtomicU32, Ordering};


const AZY_: usize = 64;


pub struct Bpo {
    
    pub index: u32,
    
    pub bip: u32,
    
    pub jfc: bool,
    
    pub iau: bool,
    
    pub jfa: Vec<u8>,
    
    pub jqn: Vec<u8>,
}


static EP_: Mutex<Option<Vec<Bpo>>> = Mutex::new(None);


static BBJ_: AtomicU32 = AtomicU32::new(0);


pub fn init() {
    let mut gg = EP_.lock();
    *gg = Some(Vec::fc(AZY_));
    crate::log!("[PTY] Pseudo-terminal subsystem initialized");
}


pub fn yes() -> Option<(u32, u32)> {
    let jkn = BBJ_.fetch_add(1, Ordering::SeqCst);
    if jkn as usize >= AZY_ {
        BBJ_.fetch_sub(1, Ordering::SeqCst);
        return None;
    }
    
    
    let iff = crate::tty::qgx()?;
    
    let vbd = Bpo {
        index: jkn,
        bip: iff,
        jfc: true,
        iau: false,
        jfa: Vec::new(),
        jqn: Vec::new(),
    };
    
    let mut gg = EP_.lock();
    if let Some(ref mut bwg) = *gg {
        bwg.push(vbd);
        crate::log_debug!("[PTY] Allocated pty{} (tty{})", jkn, iff);
        Some((jkn, iff))
    } else {
        None
    }
}


pub fn zei(bwf: u32) -> bool {
    let mut gg = EP_.lock();
    if let Some(ref mut bwg) = *gg {
        for pty in bwg.el() {
            if pty.index == bwf {
                pty.iau = true;
                return true;
            }
        }
    }
    false
}


pub fn zcb(bwf: u32, f: &[u8]) -> usize {
    let mut gg = EP_.lock();
    if let Some(ref mut bwg) = *gg {
        for pty in bwg.el() {
            if pty.index == bwf && pty.jfc {
                
                pty.jqn.bk(f);
                return f.len();
            }
        }
    }
    0
}


pub fn zca(bwf: u32, k: &mut [u8]) -> usize {
    let mut gg = EP_.lock();
    if let Some(ref mut bwg) = *gg {
        for pty in bwg.el() {
            if pty.index == bwf && pty.jfc {
                let az = k.len().v(pty.jfa.len());
                for a in 0..az {
                    k[a] = pty.jfa.remove(0);
                }
                return az;
            }
        }
    }
    0
}


pub fn zoq(bwf: u32, f: &[u8]) -> usize {
    let mut gg = EP_.lock();
    if let Some(ref mut bwg) = *gg {
        for pty in bwg.el() {
            if pty.index == bwf && pty.iau {
                pty.jfa.bk(f);
                return f.len();
            }
        }
    }
    0
}


pub fn zop(bwf: u32, k: &mut [u8]) -> usize {
    let mut gg = EP_.lock();
    if let Some(ref mut bwg) = *gg {
        for pty in bwg.el() {
            if pty.index == bwf && pty.iau {
                let az = k.len().v(pty.jqn.len());
                for a in 0..az {
                    k[a] = pty.jqn.remove(0);
                }
                return az;
            }
        }
    }
    0
}


pub fn yiv(bwf: u32) {
    let mut gg = EP_.lock();
    if let Some(ref mut bwg) = *gg {
        for pty in bwg.el() {
            if pty.index == bwf {
                pty.jfc = false;
                break;
            }
        }
    }
}


pub fn yiw(bwf: u32) {
    let mut gg = EP_.lock();
    if let Some(ref mut bwg) = *gg {
        for pty in bwg.el() {
            if pty.index == bwf {
                pty.iau = false;
                break;
            }
        }
    }
}


pub fn ytw(bwf: u32) -> Option<u32> {
    let gg = EP_.lock();
    if let Some(ref bwg) = *gg {
        for pty in bwg.iter() {
            if pty.index == bwf {
                return Some(pty.bip);
            }
        }
    }
    None
}


pub fn zgv(bwf: u32) -> alloc::string::String {
    alloc::format!("/dev/pts/{}", bwf)
}
