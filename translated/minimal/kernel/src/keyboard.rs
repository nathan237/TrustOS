




use spin::Mutex;
use alloc::string::String;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicBool, Ordering};
use crate::serial;
use crate::arch::Port;


const HA_: u16 = 0x60;
const HB_: u16 = 0x64;
const AGS_: u16 = 0x64;






pub fn tto() {
    crate::serial_println!("[i8042] Initializing PS/2 keyboard controller...");

    
    fku(0xAD); 
    fku(0xA7); 

    
    {
        let mut f = Port::<u8>::new(HA_);
        let mut status = Port::<u8>::new(HB_);
        for _ in 0..64 {
            if unsafe { status.read() } & 0x01 == 0 { break; }
            unsafe { f.read(); }
        }
    }

    
    fku(0x20);
    let cfg = izc();
    crate::serial_println!("[i8042] Config byte: {:#04x}", cfg);

    
    
    let opm = (cfg | 0x41) & !0x10;
    fku(0x60); 
    lcy(opm);

    
    fku(0xAA);
    let test = izc();
    if test == 0x55 {
        crate::serial_println!("[i8042] Self-test PASSED");
    } else {
        crate::serial_println!("[i8042] Self-test returned {:#04x} (expected 0x55) — continuing anyway", test);
        
    }

    
    fku(0x60);
    lcy(opm);

    
    fku(0xAE);

    
    lcy(0xFF);
    let alx = izc();
    if alx == 0xFA {
        let qnx = izc();
        crate::serial_println!("[i8042] Keyboard reset: ACK={:#04x} BAT={:#04x}", alx, qnx);
    } else {
        crate::serial_println!("[i8042] Keyboard reset: response {:#04x} (no ACK)", alx);
    }

    
    {
        let mut f = Port::<u8>::new(HA_);
        let mut status = Port::<u8>::new(HB_);
        for _ in 0..64 {
            if unsafe { status.read() } & 0x01 == 0 { break; }
            unsafe { f.read(); }
        }
    }

    crate::serial_println!("[i8042] PS/2 keyboard controller ready");
}


fn fku(cmd: u8) {
    let mut status = Port::<u8>::new(HB_);
    let mut ro = Port::<u8>::new(AGS_);
    
    
    for _ in 0..1_000_000 {
        if unsafe { status.read() } & 0x02 == 0 { break; }
        core::hint::hc();
    }
    unsafe { ro.write(cmd); }
}


fn lcy(f: u8) {
    let mut status = Port::<u8>::new(HB_);
    let mut port = Port::<u8>::new(HA_);
    for _ in 0..1_000_000 {
        if unsafe { status.read() } & 0x02 == 0 { break; }
        core::hint::hc();
    }
    unsafe { port.write(f); }
}


fn izc() -> u8 {
    let mut status = Port::<u8>::new(HB_);
    let mut port = Port::<u8>::new(HA_);
    for _ in 0..1_000_000 {
        if unsafe { status.read() } & 0x01 != 0 {
            return unsafe { port.read() };
        }
        core::hint::hc();
    }
    0xFF 
}


const RT_: usize = 256;


pub const V_: u8 = 0xF0;
pub const U_: u8 = 0xF1;
pub const AH_: u8 = 0xF2;
pub const AI_: u8 = 0xF3;
pub const CQ_: u8 = 0xF4;
pub const CP_: u8 = 0xF5;
pub const CX_: u8 = 0xF6;
pub const AM_: u8 = 0xF7;
pub const AQ_: u8 = 0xF8;


pub const AYA_: u8 = 0xD0;      
pub const AXZ_: u8 = 0xD1;    
pub const AXY_: u8 = 0xD2;    
pub const AXV_: u8 = 0xD3;          
pub const AXU_: u8 = 0xD4;        
pub const AXW_: u8 = 0xD5;       
pub const AXX_: u8 = 0xD6;      


struct KeyboardBuffer {
    bi: [u8; RT_],
    fsh: usize,
    bau: usize,
}

impl KeyboardBuffer {
    const fn new() -> Self {
        Self {
            bi: [0; RT_],
            fsh: 0,
            bau: 0,
        }
    }

    fn push(&mut self, hf: u8) {
        let oqp = (self.bau + 1) % RT_;
        if oqp != self.fsh {
            self.bi[self.bau] = hf;
            self.bau = oqp;
        }
    }

    fn pop(&mut self) -> Option<u8> {
        if self.fsh == self.bau {
            None
        } else {
            let hf = self.bi[self.fsh];
            self.fsh = (self.fsh + 1) % RT_;
            Some(hf)
        }
    }

    fn is_empty(&self) -> bool {
        self.fsh == self.bau
    }
}


const FH_: usize = 32;

struct CommandHistory {
    ch: [Option<String>; FH_],
    bau: usize,
    cdi: usize,
    az: usize,
}

impl CommandHistory {
    const fn new() -> Self {
        
        Self {
            ch: [
                None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None,
            ],
            bau: 0,
            cdi: 0,
            az: 0,
        }
    }
    
    fn add(&mut self, cmd: &str) {
        if cmd.is_empty() {
            return;
        }
        
        if self.az > 0 {
            let uck = if self.bau == 0 { FH_ - 1 } else { self.bau - 1 };
            if let Some(ref qv) = self.ch[uck] {
                if qv == cmd {
                    self.cdi = self.bau;
                    return;
                }
            }
        }
        
        self.ch[self.bau] = Some(String::from(cmd));
        self.bau = (self.bau + 1) % FH_;
        if self.az < FH_ {
            self.az += 1;
        }
        self.cdi = self.bau;
    }
    
    fn tek(&mut self) -> Option<&str> {
        if self.az == 0 {
            return None;
        }
        
        let loa = if self.cdi == 0 { 
            FH_ - 1 
        } else { 
            self.cdi - 1 
        };
        
        
        let osk = if self.az < FH_ {
            0
        } else {
            self.bau
        };
        
        if loa == osk && self.cdi == osk {
            
            return self.ch[self.cdi].ahz();
        }
        
        if self.ch[loa].is_some() {
            self.cdi = loa;
            self.ch[self.cdi].ahz()
        } else {
            None
        }
    }
    
    fn tee(&mut self) -> Option<&str> {
        if self.cdi == self.bau {
            return None; 
        }
        
        self.cdi = (self.cdi + 1) % FH_;
        
        if self.cdi == self.bau {
            None 
        } else {
            self.ch[self.cdi].ahz()
        }
    }
    
    fn vxt(&mut self) {
        self.cdi = self.bau;
    }
    
    fn iter(&self) -> impl Iterator<Item = (usize, &str)> {
        let az = self.az;
        let ay = if az < FH_ { 0 } else { self.bau };
        
        (0..az).map(move |a| {
            let w = (ay + a) % FH_;
            (a + 1, self.ch[w].ahz().unwrap_or(""))
        })
    }
}


static UI_: Mutex<KeyboardBuffer> = Mutex::new(KeyboardBuffer::new());


static MK_: Mutex<CommandHistory> = Mutex::new(CommandHistory::new());

static Bcz: Mutex<Option<String>> = Mutex::new(None);


static TA_: AtomicBool = AtomicBool::new(false);


static ABQ_: core::sync::atomic::AtomicU8 = core::sync::atomic::AtomicU8::new(0);


static RB_: AtomicBool = AtomicBool::new(false);


static LB_: AtomicBool = AtomicBool::new(false);

static ZV_: AtomicBool = AtomicBool::new(false);

static AGD_: AtomicBool = AtomicBool::new(false);

static BP_: AtomicBool = AtomicBool::new(false);


static AYC_: Mutex<[u8; 32]> = Mutex::new([0u8; 32]);


static DSJ_: core::sync::atomic::AtomicU8 = core::sync::atomic::AtomicU8::new(0xFF);

static ECJ_: core::sync::atomic::AtomicU8 = core::sync::atomic::AtomicU8::new(0);


const CQH_: [u8; 128] = [
    0, 27, 
    b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'0', b'-', b'=', 
    0x08, 
    b'\t', 
    b'q', b'w', b'e', b'r', b't', b'y', b'u', b'i', b'o', b'p', b'[', b']', 
    b'\n', 
    0, 
    b'a', b's', b'd', b'f', b'g', b'h', b'j', b'k', b'l', b';', b'\'', b'`', 
    0, 
    b'\\', b'z', b'x', b'c', b'v', b'b', b'n', b'm', b',', b'.', b'/', 
    0, 
    b'*', 
    0, 
    b' ', 
    0, 
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 
    0, 
    0, 
    b'7', b'8', b'9', b'-', 
    b'4', b'5', b'6', b'+', 
    b'1', b'2', b'3', 
    b'0', b'.', 
    0, 0, 0, 
    0, 0, 
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 
    0, 0, 0, 0, 0, 0, 0, 
];


const CQI_: [u8; 128] = [
    0, 27, 
    b'!', b'@', b'#', b'$', b'%', b'^', b'&', b'*', b'(', b')', b'_', b'+', 
    0x08, 
    b'\t', 
    b'Q', b'W', b'E', b'R', b'T', b'Y', b'U', b'I', b'O', b'P', b'{', b'}', 
    b'\n', 
    0, 
    b'A', b'S', b'D', b'F', b'G', b'H', b'J', b'K', b'L', b':', b'"', b'~', 
    0, 
    b'|', b'Z', b'X', b'C', b'V', b'B', b'N', b'M', b'<', b'>', b'?', 
    0, 
    b'*', 
    0, 
    b' ', 
    0, 
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 
    0, 
    0, 
    b'7', b'8', b'9', b'-', 
    b'4', b'5', b'6', b'+', 
    b'1', b'2', b'3', 
    b'0', b'.', 
    0, 0, 0, 
    0, 0, 
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 
    0, 0, 0, 0, 0, 0, 0, 
];




#[inline]
fn bpv(hf: u8) {
    if let Some(mut k) = UI_.try_lock() {
        k.push(hf);
    }
    
}


pub fn crc(scancode: u8) {
    
    
    
    
    let chz = ABQ_.load(Ordering::SeqCst);
    if chz > 0 {
        ABQ_.store(chz - 1, Ordering::SeqCst);
        return;
    }
    if scancode == 0xE1 {
        
        ABQ_.store(5, Ordering::SeqCst);
        return;
    }
    
    
    
    
    
    if scancode == 0x00 || scancode == 0xFF || scancode == 0xFA 
        || scancode == 0xFE || scancode == 0xFC
        || scancode == 0xEE || scancode == 0xAB {
        return;
    }
    
    
    
    
    let lhe = scancode & 0x7F;
    if lhe >= 0x47 && lhe <= 0x53 {
        if !TA_.load(Ordering::SeqCst) && !AGD_.load(Ordering::SeqCst) {
            
            
            
            let bep = scancode & 0x80 != 0;
            if !bep {
                let fvb = match lhe {
                    0x47 => Some(CQ_),
                    0x48 => Some(V_),
                    0x49 => Some(AM_),
                    0x4B => Some(AH_),
                    0x4D => Some(AI_),
                    0x4F => Some(CP_),
                    0x50 => Some(U_),
                    0x51 => Some(AQ_),
                    0x53 => Some(CX_),
                    _ => None,
                };
                if let Some(eh) = fvb {
                    bpv(eh);
                }
            }
            return;
        }
    }
    
    
    if scancode == 0xE0 {
        TA_.store(true, Ordering::SeqCst);
        return;
    }
    
    let ofx = TA_.load(Ordering::SeqCst);
    TA_.store(false, Ordering::SeqCst);
    
    
    let bep = scancode & 0x80 != 0;
    let bs = scancode & 0x7F;
    
    
    xox(bs, !bep);
    
    
    if ofx {
        
        if bs == 0x1D {
            BP_.store(!bep, Ordering::SeqCst);
            if !bep {
                crate::accessibility::ibv(crate::accessibility::StickyModifier::Wd);
            }
            return;
        }
        if bs == 0x38 {
            RB_.store(!bep, Ordering::SeqCst);
            if !bep {
                crate::accessibility::ibv(crate::accessibility::StickyModifier::Vh);
            }
            return;
        }
        
        
        if !bep {
            let bdj = RB_.load(Ordering::SeqCst);
            let db = BP_.load(Ordering::SeqCst);
            
            let fvb = match bs {
                0x48 if bdj  => Some(AXV_),
                0x50 if bdj  => Some(AXU_),
                0x4B if db => Some(AXW_),
                0x4D if db => Some(AXX_),
                0x48 => Some(V_),
                0x50 => Some(U_),
                0x4B => Some(AH_),
                0x4D => Some(AI_),
                0x47 => Some(CQ_),
                0x4F => Some(CP_),
                0x53 => Some(CX_),
                0x71 => Some(CX_), 
                0x75 => Some(V_),      
                0x72 => Some(U_),    
                0x6B => Some(AH_),    
                0x74 => Some(AI_),   
                0x6C => Some(CQ_),    
                0x69 => Some(CP_),     
                0x49 => Some(AM_),
                0x51 => Some(AQ_),
                _ => None,
            };
            if let Some(eh) = fvb {
                bpv(eh);
            }
        }
        return;
    }

    
    if !ofx && !bep && bs == 0x29 {
        bpv(b' ');
        return;
    }
    
    
    if bs == 0x1D {
        BP_.store(!bep, Ordering::SeqCst);
        
        if !bep {
            crate::accessibility::ibv(crate::accessibility::StickyModifier::Wd);
        }
        return;
    }
    
    
    if bs == 0x2A || bs == 0x36 {
        
        LB_.store(!bep, Ordering::SeqCst);
        if !bep {
            crate::accessibility::ibv(crate::accessibility::StickyModifier::Yv);
        }
        return;
    }
    
    
    if bs == 0x3A && !bep {
        let cv = ZV_.load(Ordering::SeqCst);
        ZV_.store(!cv, Ordering::SeqCst);
        return;
    }
    
    
    if bs == 0x45 && !bep {
        let cv = AGD_.load(Ordering::SeqCst);
        AGD_.store(!cv, Ordering::SeqCst);
        return;
    }
    
    
    if bep {
        return;
    }
    
    
    if BP_.load(Ordering::SeqCst) && bs == 0x1E {
        bpv(1); 
        return;
    }

    
    if BP_.load(Ordering::SeqCst) && bs == 0x2E {
        bpv(3); 
        return;
    }

    
    if BP_.load(Ordering::SeqCst) && bs == 0x2F {
        bpv(0x16); 
        return;
    }

    
    if BP_.load(Ordering::SeqCst) && bs == 0x2D {
        bpv(0x18); 
        return;
    }

    
    if BP_.load(Ordering::SeqCst) && bs == 0x26 {
        bpv(12); 
        return;
    }

    
    if BP_.load(Ordering::SeqCst) && bs == 0x1F {
        bpv(0x13); 
        return;
    }

    
    if BP_.load(Ordering::SeqCst) && bs == 0x22 {
        bpv(0x07); 
        return;
    }

    
    if BP_.load(Ordering::SeqCst) && bs == 0x21 {
        bpv(0x06); 
        return;
    }

    
    if BP_.load(Ordering::SeqCst) && bs == 0x23 {
        bpv(0x12); 
        return;
    }

    
    if BP_.load(Ordering::SeqCst) && bs == 0x2C {
        bpv(0x1A); 
        return;
    }

    
    if BP_.load(Ordering::SeqCst) && bs == 0x15 {
        bpv(0x19); 
        return;
    }

    
    if BP_.load(Ordering::SeqCst) && bs == 0x35 {
        bpv(AYA_);
        return;
    }

    
    if BP_.load(Ordering::SeqCst) && LB_.load(Ordering::SeqCst) && bs == 0x25 {
        bpv(AXZ_);
        return;
    }

    
    if BP_.load(Ordering::SeqCst) && LB_.load(Ordering::SeqCst) && bs == 0x20 {
        bpv(AXY_);
        return;
    }

    
    let acn = LB_.load(Ordering::SeqCst)
        || crate::accessibility::jbu(crate::accessibility::StickyModifier::Yv);
    let dr = ZV_.load(Ordering::SeqCst);
    
    let ascii = if bs < 128 {
        let ar = if acn {
            CQI_[bs as usize]
        } else {
            CQH_[bs as usize]
        };
        
        
        if dr && ar >= b'a' && ar <= b'z' {
            ar - 32 
        } else if dr && ar >= b'A' && ar <= b'Z' {
            ar + 32 
        } else {
            ar
        }
    } else {
        0
    };
    
    
    if ascii != 0 {
        
        bpv(ascii);
        
        crate::accessibility::wuh();
    }
}


pub fn auw() -> Option<u8> {
    
    
    let result = crate::arch::cvh(|| {
        UI_.lock().pop()
    });
    if let Some(o) = result {
        return Some(o);
    }
    serial::dlb()
}


pub fn voj(ascii: u8) {
    if ascii != 0 {
        crate::arch::cvh(|| {
            UI_.lock().push(ascii);
        });
    }
}


pub fn hmo() -> bool {
    crate::arch::cvh(|| {
        !UI_.lock().is_empty()
    })
}



pub fn alh(scancode: u8) -> bool {
    
    
    crate::arch::cvh(|| {
        match scancode {
            0x38 => RB_.load(Ordering::Relaxed)
                || crate::accessibility::jbu(crate::accessibility::StickyModifier::Vh),
            0x1D => BP_.load(Ordering::Relaxed)
                || crate::accessibility::jbu(crate::accessibility::StickyModifier::Wd),
            0x2A | 0x36 => LB_.load(Ordering::Relaxed)
                || crate::accessibility::jbu(crate::accessibility::StickyModifier::Yv),
            _ => {
                let g = AYC_.lock();
                let avk = (scancode / 8) as usize;
                let deh = scancode % 8;
                if avk < 32 {
                    (g[avk] & (1 << deh)) != 0
                } else {
                    false
                }
            }
        }
    })
}


fn xox(scancode: u8, vn: bool) {
    
    match scancode {
        0x38 => {
            RB_.store(vn, Ordering::Relaxed);
            
            if vn {
                crate::accessibility::ibv(crate::accessibility::StickyModifier::Vh);
            }
        }
        0x1D => BP_.store(vn, Ordering::Relaxed),
        0x2A | 0x36 => LB_.store(vn, Ordering::Relaxed),
        _ => {}
    }
    
    
    let mut g = AYC_.lock();
    let avk = (scancode / 8) as usize;
    let deh = scancode % 8;
    if avk < 32 {
        if vn {
            g[avk] |= 1 << deh;
        } else {
            g[avk] &= !(1 << deh);
        }
    }
}


pub fn jzh(cmd: &str) {
    MK_.lock().add(cmd);
}


pub fn lcd() -> Option<String> {
    MK_.lock().tek().map(String::from)
}


pub fn lcc() -> Option<String> {
    MK_.lock().tee().map(String::from)
}


pub fn lce() {
    MK_.lock().vxt();
}


pub fn toz() -> Vec<(usize, String)> {
    MK_.lock().iter().map(|(a, e)| (a, String::from(e))).collect()
}

pub fn eno(text: &str) {
    *Bcz.lock() = Some(String::from(text));
}

pub fn ndn() -> Option<String> {
    Bcz.lock().as_ref().map(|e| e.clone())
}


pub fn vrz(bi: &mut [u8]) -> usize {
    let mut u = 0;
    let mut gi = 0; 
    let mut ylb = String::new(); 
    let mut bww = false;
    
    
    lce();
    
    loop {
        if let Some(r) = auw() {
            match r {
                b'\n' | b'\r' => {
                    crate::println!();
                    
                    let cmd = core::str::jg(&bi[..u]).unwrap_or("");
                    if !cmd.em().is_empty() {
                        jzh(cmd);
                    }
                    break;
                }
                0x01 => {
                    
                    bww = true;
                }
                0x08 => {
                    
                    if bww {
                        
                        while gi > 0 {
                            crate::print!("\x08");
                            gi -= 1;
                        }
                        for _ in 0..u {
                            crate::print!(" ");
                        }
                        for _ in 0..u {
                            crate::print!("\x08");
                        }
                        u = 0;
                        gi = 0;
                        bww = false;
                    } else if gi > 0 {
                        
                        for a in gi..u {
                            bi[a - 1] = bi[a];
                        }
                        u = u.ao(1);
                        gi = gi.ao(1);
                        
                        
                        crate::print!("\x08");
                        for a in gi..u {
                            crate::print!("{}", bi[a] as char);
                        }
                        crate::print!(" ");
                        for _ in gi..=u {
                            crate::print!("\x08");
                        }
                    }
                }
                V_ => {
                    
                    if let Some(vo) = lcd() {
                        bww = false;
                        
                        while gi > 0 {
                            crate::print!("\x08");
                            gi -= 1;
                        }
                        for _ in 0..u {
                            crate::print!(" ");
                        }
                        for _ in 0..u {
                            crate::print!("\x08");
                        }
                        
                        let bf = vo.as_bytes();
                        let len = bf.len().v(bi.len() - 1);
                        bi[..len].dg(&bf[..len]);
                        u = len;
                        gi = len;
                        crate::print!("{}", &vo[..len]);
                    }
                }
                U_ => {
                    
                    let next = lcc();
                    bww = false;
                    
                    while gi > 0 {
                        crate::print!("\x08");
                        gi -= 1;
                    }
                    for _ in 0..u {
                        crate::print!(" ");
                    }
                    for _ in 0..u {
                        crate::print!("\x08");
                    }
                    
                    if let Some(oqk) = next {
                        let bf = oqk.as_bytes();
                        let len = bf.len().v(bi.len() - 1);
                        bi[..len].dg(&bf[..len]);
                        u = len;
                        gi = len;
                        crate::print!("{}", &oqk[..len]);
                    } else {
                        u = 0;
                        gi = 0;
                    }
                }
                AH_ => {
                    bww = false;
                    if gi > 0 {
                        gi -= 1;
                        crate::print!("\x08");
                    }
                }
                AI_ => {
                    bww = false;
                    if gi < u {
                        crate::print!("{}", bi[gi] as char);
                        gi += 1;
                    }
                }
                CQ_ => {
                    bww = false;
                    while gi > 0 {
                        crate::print!("\x08");
                        gi -= 1;
                    }
                }
                CP_ => {
                    bww = false;
                    while gi < u {
                        crate::print!("{}", bi[gi] as char);
                        gi += 1;
                    }
                }
                CX_ => {
                    
                    if bww {
                        
                        while gi > 0 {
                            crate::print!("\x08");
                            gi -= 1;
                        }
                        for _ in 0..u {
                            crate::print!(" ");
                        }
                        for _ in 0..u {
                            crate::print!("\x08");
                        }
                        u = 0;
                        gi = 0;
                        bww = false;
                    } else if gi < u {
                        
                        for a in gi..u.ao(1) {
                            bi[a] = bi[a + 1];
                        }
                        u = u.ao(1);
                        
                        
                        for a in gi..u {
                            crate::print!("{}", bi[a] as char);
                        }
                        crate::print!(" ");
                        
                        for _ in gi..=u {
                            crate::print!("\x08");
                        }
                    }
                }
                12 => {
                    
                    crate::framebuffer::clear();
                    
                    crate::gr!(crate::framebuffer::G_, "trustos");
                    crate::gr!(crate::framebuffer::B_, "> ");
                    for a in 0..u {
                        crate::print!("{}", bi[a] as char);
                    }
                    
                    for _ in gi..u {
                        crate::print!("\x08");
                    }
                    bww = false;
                }
                3 => {
                    
                    if let Ok(text) = core::str::jg(&bi[..u]) {
                        eno(text);
                    }
                    bww = false;
                }
                0x16 => {
                    
                    if let Some(text) = ndn() {
                        if bww {
                            
                            while gi > 0 {
                                crate::print!("\x08");
                                gi -= 1;
                            }
                            for _ in 0..u {
                                crate::print!(" ");
                            }
                            for _ in 0..u {
                                crate::print!("\x08");
                            }
                            u = 0;
                            gi = 0;
                            bww = false;
                        }
                        for o in text.bf() {
                            if o < 0x20 || o >= 0x7F || u >= bi.len() - 1 {
                                continue;
                            }
                            if gi < u {
                                for a in (gi..u).vv() {
                                    bi[a + 1] = bi[a];
                                }
                            }
                            bi[gi] = o;
                            u += 1;
                            gi += 1;

                            for a in gi - 1..u {
                                crate::print!("{}", bi[a] as char);
                            }
                            for _ in gi..u {
                                crate::print!("\x08");
                            }
                        }
                    }
                }
                _ if r >= 0x20 && r < 0x7F && u < bi.len() - 1 => {
                    
                    if bww {
                        
                        while gi > 0 {
                            crate::print!("\x08");
                            gi -= 1;
                        }
                        for _ in 0..u {
                            crate::print!(" ");
                        }
                        for _ in 0..u {
                            crate::print!("\x08");
                        }
                        u = 0;
                        gi = 0;
                        bww = false;
                    }
                    if gi < u {
                        
                        for a in (gi..u).vv() {
                            bi[a + 1] = bi[a];
                        }
                    }
                    bi[gi] = r;
                    u += 1;
                    gi += 1;
                    
                    
                    for a in gi - 1..u {
                        crate::print!("{}", bi[a] as char);
                    }
                    
                    for _ in gi..u {
                        crate::print!("\x08");
                    }
                }
                _ => {}
            }
        } else {
            
            crate::arch::bhd();
        }
    }
    
    bi[u] = 0; 
    u
}


pub fn cts(bi: &mut [u8]) -> usize {
    vrz(bi)
}


pub fn fsf(bi: &mut [u8]) -> usize {
    let mut u = 0;
    
    loop {
        if let Some(r) = auw() {
            match r {
                b'\n' | b'\r' => {
                    
                    break;
                }
                0x08 => {
                    
                    if u > 0 {
                        u -= 1;
                        bi[u] = 0;
                        
                        crate::print!("\x08 \x08");
                    }
                }
                0x03 => {
                    
                    u = 0;
                    break;
                }
                0x15 => {
                    
                    for _ in 0..u {
                        crate::print!("\x08 \x08");
                    }
                    u = 0;
                }
                r if r >= 0x20 && r < 0x7F => {
                    
                    if u < bi.len() - 1 {
                        bi[u] = r;
                        u += 1;
                        
                        crate::print!("*");
                    }
                }
                _ => {}
            }
        }
    }
    
    u
}







pub fn xw() -> Option<u8> {
    auw()
}


pub fn xtj() -> u8 {
    loop {
        if let Some(bs) = auw() {
            return bs;
        }
        
        for _ in 0..1000 {
            core::hint::hc();
        }
    }
}
