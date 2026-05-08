




use alloc::collections::VecDeque;
use alloc::string::String;
use alloc::vec::Vec;
use spin::Mutex;
use core::sync::atomic::{AtomicU32, Ordering};


const BCF_: usize = 16;


#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct WinSize {
    pub ws_row: u16,
    pub ws_col: u16,
    pub ws_xpixel: u16,
    pub ws_ypixel: u16,
}

impl Default for WinSize {
    fn default() -> Self {
        Self { ws_row: 25, ws_col: 80, ws_xpixel: 0, ws_ypixel: 0 }
    }
}


#[derive(Clone, Debug)]
pub struct Termios {
    
    pub iflag: u32,
    
    pub oflag: u32,
    
    pub cflag: u32,
    
    pub lflag: u32,
}


pub const Yb: u32   = 0x0008;
pub const Zs: u32 = 0x0002;
pub const Aaa: u32   = 0x0001;

impl Default for Termios {
    fn default() -> Self {
        Self {
            iflag: 0,
            oflag: 0,
            cflag: 0,
            lflag: Yb | Zs | Aaa, 
        }
    }
}


pub struct TtyDevice {
    pub index: u32,
    
    pub session_id: u32,
    
    pub foreground_pgid: u32,
    
    pub termios: Termios,
    
    pub input_buf: VecDeque<u8>,
    
    pub line_buf: Vec<u8>,
    
    pub output_buf: VecDeque<u8>,
    
    pub winsize: WinSize,
    
    pub active: bool,
}

impl TtyDevice {
    pub fn new(index: u32) -> Self {
        Self {
            index,
            session_id: 0,
            foreground_pgid: 0,
            termios: Termios::default(),
            input_buf: VecDeque::new(),
            line_buf: Vec::new(),
            output_buf: VecDeque::new(),
            winsize: WinSize::default(),
            active: false,
        }
    }

    
    pub fn qll(&mut self, ch: u8) {
        let khg = (self.termios.lflag & Zs) != 0;
        let cxa = (self.termios.lflag & Yb) != 0;
        let signals = (self.termios.lflag & Aaa) != 0;

        
        if signals {
            match ch {
                3 => {
                    
                    if self.foreground_pgid > 0 {
                        let _ = crate::signals::geu(self.foreground_pgid, 2); 
                    }
                    return;
                }
                26 => {
                    
                    if self.foreground_pgid > 0 {
                        let _ = crate::signals::geu(self.foreground_pgid, 20); 
                    }
                    return;
                }
                28 => {
                    
                    if self.foreground_pgid > 0 {
                        let _ = crate::signals::geu(self.foreground_pgid, 3); 
                    }
                    return;
                }
                _ => {}
            }
        }

        if khg {
            
            match ch {
                b'\n' | b'\r' => {
                    self.line_buf.push(b'\n');
                    
                    for &b in &self.line_buf {
                        self.input_buf.push_back(b);
                    }
                    self.line_buf.clear();
                    if cxa {
                        self.output_buf.push_back(b'\n');
                    }
                }
                0x7F | 8 => {
                    
                    if !self.line_buf.is_empty() {
                        self.line_buf.pop();
                        if cxa {
                            self.output_buf.push_back(8);
                            self.output_buf.push_back(b' ');
                            self.output_buf.push_back(8);
                        }
                    }
                }
                _ => {
                    self.line_buf.push(ch);
                    if cxa {
                        self.output_buf.push_back(ch);
                    }
                }
            }
        } else {
            
            self.input_buf.push_back(ch);
            if cxa {
                self.output_buf.push_back(ch);
            }
        }
    }

    
    pub fn read(&mut self, buf: &mut [u8]) -> usize {
        let count = buf.len().min(self.input_buf.len());
        for i in 0..count {
            buf[i] = self.input_buf.pop_front().unwrap_or(0);
        }
        count
    }

    
    pub fn write(&mut self, data: &[u8]) -> usize {
        for &b in data {
            self.output_buf.push_back(b);
        }
        
        for &b in data {
            crate::serial_print!("{}", b as char);
        }
        data.len()
    }

    
    pub fn qfy(&mut self) -> Vec<u8> {
        self.output_buf.drain(..).collect()
    }

    
    pub fn qmg(&self, sid: u32) -> bool {
        self.active && self.session_id == sid
    }
}


static ALB_: Mutex<Option<Vec<TtyDevice>>> = Mutex::new(None);


static AHU_: AtomicU32 = AtomicU32::new(0);


pub fn init() {
    let mut bs = ALB_.lock();
    let mut devices = Vec::with_capacity(BCF_);
    
    
    let mut fdq = TtyDevice::new(0);
    fdq.active = true;
    fdq.session_id = 1; 
    fdq.foreground_pgid = 1;
    devices.push(fdq);
    
    AHU_.store(1, Ordering::SeqCst);
    *bs = Some(devices);
    
    crate::log!("[TTY] TTY subsystem initialized (tty0 = console)");
}


pub fn juy() -> Option<u32> {
    let idx = AHU_.fetch_add(1, Ordering::SeqCst);
    if idx as usize >= BCF_ {
        AHU_.fetch_sub(1, Ordering::SeqCst);
        return None;
    }
    
    let mut bs = ALB_.lock();
    if let Some(ref mut devices) = *bs {
        let tty = TtyDevice::new(idx);
        devices.push(tty);
        Some(idx)
    } else {
        None
    }
}


pub fn cfj<F, U>(index: u32, f: F) -> Option<U>
where
    F: FnOnce(&mut TtyDevice) -> U,
{
    let mut bs = ALB_.lock();
    if let Some(ref mut devices) = *bs {
        for tty in devices.iter_mut() {
            if tty.index == index {
                return Some(f(tty));
            }
        }
    }
    None
}


pub fn ooy(tty_index: u32, pgid: u32) {
    cfj(tty_index, |tty| {
        tty.foreground_pgid = pgid;
    });
}


pub fn mdc(tty_index: u32) -> u32 {
    cfj(tty_index, |tty| tty.foreground_pgid).unwrap_or(0)
}


pub fn jfa(tty_index: u32, session_id: u32) {
    cfj(tty_index, |tty| {
        tty.active = true;
        tty.session_id = session_id;
    });
}


pub fn med(tty_index: u32) -> WinSize {
    cfj(tty_index, |tty| tty.winsize).unwrap_or_default()
}


pub fn opu(tty_index: u32, asv: WinSize) {
    cfj(tty_index, |tty| {
        tty.winsize = asv;
    });
}





pub const Aqj: u64   = 0x540F;
pub const Aqm: u64   = 0x5410;
pub const Aqk: u64    = 0x5429;
pub const Nl: u64  = 0x5413;
pub const Aff: u64  = 0x5414;
pub const Aql: u64   = 0x540E;
pub const Bdu: u64   = 0x5422;
pub const Nk: u64      = 0x5401;
pub const Vi: u64      = 0x5402;


pub fn qkd(tty_index: u32, request: u64, db: u64) -> i64 {
    match request {
        Aqj => {
            
            let pgid = mdc(tty_index);
            if db != 0 && crate::memory::ij(db, 4, true) {
                unsafe { *(db as *mut u32) = pgid; }
            }
            0
        }
        Aqm => {
            
            if db != 0 && crate::memory::ij(db, 4, false) {
                let pgid = unsafe { *(db as *const u32) };
                ooy(tty_index, pgid);
            }
            0
        }
        Aqk => {
            
            let sid = cfj(tty_index, |tty| tty.session_id).unwrap_or(0);
            if db != 0 && crate::memory::ij(db, 4, true) {
                unsafe { *(db as *mut u32) = sid; }
            }
            0
        }
        Nl => {
            
            let asv = med(tty_index);
            if db != 0 && crate::memory::ij(db, 8, true) {
                unsafe { *(db as *mut WinSize) = asv; }
            }
            0
        }
        Aff => {
            
            if db != 0 && crate::memory::ij(db, 8, false) {
                let asv = unsafe { *(db as *const WinSize) };
                opu(tty_index, asv);
            }
            0
        }
        Aql => {
            
            let pid = crate::process::pe();
            let sid = crate::process::ibv(pid);
            jfa(tty_index, sid);
            0
        }
        Nk => {
            
            if db != 0 && crate::memory::ij(db, 16, true) {
                if let Some(termios) = cfj(tty_index, |tty| tty.termios.clone()) {
                    unsafe {
                        let aa = db as *mut u32;
                        *aa = termios.iflag;
                        *aa.add(1) = termios.oflag;
                        *aa.add(2) = termios.cflag;
                        *aa.add(3) = termios.lflag;
                    }
                }
            }
            0
        }
        Vi => {
            
            if db != 0 && crate::memory::ij(db, 16, false) {
                let (iflag, oflag, cflag, lflag) = unsafe {
                    let aa = db as *const u32;
                    (*aa, *aa.add(1), *aa.add(2), *aa.add(3))
                };
                cfj(tty_index, |tty| {
                    tty.termios.iflag = iflag;
                    tty.termios.oflag = oflag;
                    tty.termios.cflag = cflag;
                    tty.termios.lflag = lflag;
                });
            }
            0
        }
        _ => crate::syscall::errno::Aja,
    }
}
