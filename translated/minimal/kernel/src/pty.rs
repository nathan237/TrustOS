








use alloc::vec::Vec;
use spin::Mutex;
use core::sync::atomic::{AtomicU32, Ordering};


const BCA_: usize = 64;


pub struct Acw {
    
    pub index: u32,
    
    pub tty_index: u32,
    
    pub master_open: bool,
    
    pub slave_open: bool,
    
    pub master_buf: Vec<u8>,
    
    pub slave_buf: Vec<u8>,
}


static FE_: Mutex<Option<Vec<Acw>>> = Mutex::new(None);


static BDM_: AtomicU32 = AtomicU32::new(0);


pub fn init() {
    let mut bs = FE_.lock();
    *bs = Some(Vec::with_capacity(BCA_));
    crate::log!("[PTY] Pseudo-terminal subsystem initialized");
}


pub fn pya() -> Option<(u32, u32)> {
    let exg = BDM_.fetch_add(1, Ordering::SeqCst);
    if exg as usize >= BCA_ {
        BDM_.fetch_sub(1, Ordering::SeqCst);
        return None;
    }
    
    
    let ect = crate::tty::juy()?;
    
    let npo = Acw {
        index: exg,
        tty_index: ect,
        master_open: true,
        slave_open: false,
        master_buf: Vec::new(),
        slave_buf: Vec::new(),
    };
    
    let mut bs = FE_.lock();
    if let Some(ref mut ptys) = *bs {
        ptys.push(npo);
        crate::log_debug!("[PTY] Allocated pty{} (tty{})", exg, ect);
        Some((exg, ect))
    } else {
        None
    }
}


pub fn qpy(pty_index: u32) -> bool {
    let mut bs = FE_.lock();
    if let Some(ref mut ptys) = *bs {
        for pty in ptys.iter_mut() {
            if pty.index == pty_index {
                pty.slave_open = true;
                return true;
            }
        }
    }
    false
}


pub fn qom(pty_index: u32, data: &[u8]) -> usize {
    let mut bs = FE_.lock();
    if let Some(ref mut ptys) = *bs {
        for pty in ptys.iter_mut() {
            if pty.index == pty_index && pty.master_open {
                
                pty.slave_buf.extend_from_slice(data);
                return data.len();
            }
        }
    }
    0
}


pub fn qol(pty_index: u32, buf: &mut [u8]) -> usize {
    let mut bs = FE_.lock();
    if let Some(ref mut ptys) = *bs {
        for pty in ptys.iter_mut() {
            if pty.index == pty_index && pty.master_open {
                let count = buf.len().min(pty.master_buf.len());
                for i in 0..count {
                    buf[i] = pty.master_buf.remove(0);
                }
                return count;
            }
        }
    }
    0
}


pub fn qxd(pty_index: u32, data: &[u8]) -> usize {
    let mut bs = FE_.lock();
    if let Some(ref mut ptys) = *bs {
        for pty in ptys.iter_mut() {
            if pty.index == pty_index && pty.slave_open {
                pty.master_buf.extend_from_slice(data);
                return data.len();
            }
        }
    }
    0
}


pub fn qxc(pty_index: u32, buf: &mut [u8]) -> usize {
    let mut bs = FE_.lock();
    if let Some(ref mut ptys) = *bs {
        for pty in ptys.iter_mut() {
            if pty.index == pty_index && pty.slave_open {
                let count = buf.len().min(pty.slave_buf.len());
                for i in 0..count {
                    buf[i] = pty.slave_buf.remove(0);
                }
                return count;
            }
        }
    }
    0
}


pub fn qag(pty_index: u32) {
    let mut bs = FE_.lock();
    if let Some(ref mut ptys) = *bs {
        for pty in ptys.iter_mut() {
            if pty.index == pty_index {
                pty.master_open = false;
                break;
            }
        }
    }
}


pub fn qah(pty_index: u32) {
    let mut bs = FE_.lock();
    if let Some(ref mut ptys) = *bs {
        for pty in ptys.iter_mut() {
            if pty.index == pty_index {
                pty.slave_open = false;
                break;
            }
        }
    }
}


pub fn qin(pty_index: u32) -> Option<u32> {
    let bs = FE_.lock();
    if let Some(ref ptys) = *bs {
        for pty in ptys.iter() {
            if pty.index == pty_index {
                return Some(pty.tty_index);
            }
        }
    }
    None
}


pub fn qri(pty_index: u32) -> alloc::string::String {
    alloc::format!("/dev/pts/{}", pty_index)
}
