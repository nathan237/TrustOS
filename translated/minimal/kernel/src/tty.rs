




use alloc::collections::VecDeque;
use alloc::string::String;
use alloc::vec::Vec;
use spin::Mutex;
use core::sync::atomic::{AtomicU32, Ordering};


const BAD_: usize = 16;


#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct WinSize {
    pub mrf: u16,
    pub mre: u16,
    pub mrg: u16,
    pub mrh: u16,
}

impl Default for WinSize {
    fn default() -> Self {
        Self { mrf: 25, mre: 80, mrg: 0, mrh: 0 }
    }
}


#[derive(Clone, Debug)]
pub struct Termios {
    
    pub dry: u32,
    
    pub htl: u32,
    
    pub hco: u32,
    
    pub eub: u32,
}


pub const Bff: u32   = 0x0008;
pub const Biz: u32 = 0x0002;
pub const Bjh: u32   = 0x0001;

impl Default for Termios {
    fn default() -> Self {
        Self {
            dry: 0,
            htl: 0,
            hco: 0,
            eub: Bff | Biz | Bjh, 
        }
    }
}


pub struct TtyDevice {
    pub index: u32,
    
    pub fuj: u32,
    
    pub dqv: u32,
    
    pub cnc: Termios,
    
    pub esq: VecDeque<u8>,
    
    pub fmz: Vec<u8>,
    
    pub ega: VecDeque<u8>,
    
    pub fbn: WinSize,
    
    pub gh: bool,
}

impl TtyDevice {
    pub fn new(index: u32) -> Self {
        Self {
            index,
            fuj: 0,
            dqv: 0,
            cnc: Termios::default(),
            esq: VecDeque::new(),
            fmz: Vec::new(),
            ega: VecDeque::new(),
            fbn: WinSize::default(),
            gh: false,
        }
    }

    
    pub fn yyc(&mut self, bm: u8) {
        let qvy = (self.cnc.eub & Biz) != 0;
        let gfw = (self.cnc.eub & Bff) != 0;
        let signals = (self.cnc.eub & Bjh) != 0;

        
        if signals {
            match bm {
                3 => {
                    
                    if self.dqv > 0 {
                        let _ = crate::signals::lhk(self.dqv, 2); 
                    }
                    return;
                }
                26 => {
                    
                    if self.dqv > 0 {
                        let _ = crate::signals::lhk(self.dqv, 20); 
                    }
                    return;
                }
                28 => {
                    
                    if self.dqv > 0 {
                        let _ = crate::signals::lhk(self.dqv, 3); 
                    }
                    return;
                }
                _ => {}
            }
        }

        if qvy {
            
            match bm {
                b'\n' | b'\r' => {
                    self.fmz.push(b'\n');
                    
                    for &o in &self.fmz {
                        self.esq.agt(o);
                    }
                    self.fmz.clear();
                    if gfw {
                        self.ega.agt(b'\n');
                    }
                }
                0x7F | 8 => {
                    
                    if !self.fmz.is_empty() {
                        self.fmz.pop();
                        if gfw {
                            self.ega.agt(8);
                            self.ega.agt(b' ');
                            self.ega.agt(8);
                        }
                    }
                }
                _ => {
                    self.fmz.push(bm);
                    if gfw {
                        self.ega.agt(bm);
                    }
                }
            }
        } else {
            
            self.esq.agt(bm);
            if gfw {
                self.ega.agt(bm);
            }
        }
    }

    
    pub fn read(&mut self, k: &mut [u8]) -> usize {
        let az = k.len().v(self.esq.len());
        for a in 0..az {
            k[a] = self.esq.awp().unwrap_or(0);
        }
        az
    }

    
    pub fn write(&mut self, f: &[u8]) -> usize {
        for &o in f {
            self.ega.agt(o);
        }
        
        for &o in f {
            crate::serial_print!("{}", o as char);
        }
        f.len()
    }

    
    pub fn yrb(&mut self) -> Vec<u8> {
        self.ega.bbk(..).collect()
    }

    
    pub fn yzg(&self, ary: u32) -> bool {
        self.gh && self.fuj == ary
    }
}


static AJG_: Mutex<Option<Vec<TtyDevice>>> = Mutex::new(None);


static AGA_: AtomicU32 = AtomicU32::new(0);


pub fn init() {
    let mut gg = AJG_.lock();
    let mut ik = Vec::fc(BAD_);
    
    
    let mut jui = TtyDevice::new(0);
    jui.gh = true;
    jui.fuj = 1; 
    jui.dqv = 1;
    ik.push(jui);
    
    AGA_.store(1, Ordering::SeqCst);
    *gg = Some(ik);
    
    crate::log!("[TTY] TTY subsystem initialized (tty0 = console)");
}


pub fn qgx() -> Option<u32> {
    let w = AGA_.fetch_add(1, Ordering::SeqCst);
    if w as usize >= BAD_ {
        AGA_.fetch_sub(1, Ordering::SeqCst);
        return None;
    }
    
    let mut gg = AJG_.lock();
    if let Some(ref mut ik) = *gg {
        let tty = TtyDevice::new(w);
        ik.push(tty);
        Some(w)
    } else {
        None
    }
}


pub fn fbp<G, Ac>(index: u32, bb: G) -> Option<Ac>
where
    G: FnOnce(&mut TtyDevice) -> Ac,
{
    let mut gg = AJG_.lock();
    if let Some(ref mut ik) = *gg {
        for tty in ik.el() {
            if tty.index == index {
                return Some(bb(tty));
            }
        }
    }
    None
}


pub fn wiv(bip: u32, bai: u32) {
    fbp(bip, |tty| {
        tty.dqv = bai;
    });
}


pub fn tdp(bip: u32) -> u32 {
    fbp(bip, |tty| tty.dqv).unwrap_or(0)
}


pub fn pit(bip: u32, fuj: u32) {
    fbp(bip, |tty| {
        tty.gh = true;
        tty.fuj = fuj;
    });
}


pub fn tff(bip: u32) -> WinSize {
    fbp(bip, |tty| tty.fbn).age()
}


pub fn wjy(bip: u32, ciw: WinSize) {
    fbp(bip, |tty| {
        tty.fbn = ciw;
    });
}





pub const Cnt: u64   = 0x540F;
pub const Cnw: u64   = 0x5410;
pub const Cnu: u64    = 0x5429;
pub const Aew: u64  = 0x5413;
pub const Btx: u64  = 0x5414;
pub const Cnv: u64   = 0x540E;
pub const Djk: u64   = 0x5422;
pub const Aev: u64      = 0x5401;
pub const Azq: u64      = 0x5402;


pub fn yvv(bip: u32, request: u64, ji: u64) -> i64 {
    match request {
        Cnt => {
            
            let bai = tdp(bip);
            if ji != 0 && crate::memory::sw(ji, 4, true) {
                unsafe { *(ji as *mut u32) = bai; }
            }
            0
        }
        Cnw => {
            
            if ji != 0 && crate::memory::sw(ji, 4, false) {
                let bai = unsafe { *(ji as *const u32) };
                wiv(bip, bai);
            }
            0
        }
        Cnu => {
            
            let ary = fbp(bip, |tty| tty.fuj).unwrap_or(0);
            if ji != 0 && crate::memory::sw(ji, 4, true) {
                unsafe { *(ji as *mut u32) = ary; }
            }
            0
        }
        Aew => {
            
            let ciw = tff(bip);
            if ji != 0 && crate::memory::sw(ji, 8, true) {
                unsafe { *(ji as *mut WinSize) = ciw; }
            }
            0
        }
        Btx => {
            
            if ji != 0 && crate::memory::sw(ji, 8, false) {
                let ciw = unsafe { *(ji as *const WinSize) };
                wjy(bip, ciw);
            }
            0
        }
        Cnv => {
            
            let ce = crate::process::aei();
            let ary = crate::process::nyo(ce);
            pit(bip, ary);
            0
        }
        Aev => {
            
            if ji != 0 && crate::memory::sw(ji, 16, true) {
                if let Some(cnc) = fbp(bip, |tty| tty.cnc.clone()) {
                    unsafe {
                        let ai = ji as *mut u32;
                        *ai = cnc.dry;
                        *ai.add(1) = cnc.htl;
                        *ai.add(2) = cnc.hco;
                        *ai.add(3) = cnc.eub;
                    }
                }
            }
            0
        }
        Azq => {
            
            if ji != 0 && crate::memory::sw(ji, 16, false) {
                let (dry, htl, hco, eub) = unsafe {
                    let ai = ji as *const u32;
                    (*ai, *ai.add(1), *ai.add(2), *ai.add(3))
                };
                fbp(bip, |tty| {
                    tty.cnc.dry = dry;
                    tty.cnc.htl = htl;
                    tty.cnc.hco = hco;
                    tty.cnc.eub = eub;
                });
            }
            0
        }
        _ => crate::syscall::errno::Cbi,
    }
}
