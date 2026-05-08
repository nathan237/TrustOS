




use spin::Mutex;
use alloc::string::String;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicBool, Ordering};
use crate::serial;
use crate::arch::Port;


const HR_: u16 = 0x60;
const HS_: u16 = 0x64;
const AIM_: u16 = 0x64;






pub fn mpg() {
    crate::serial_println!("[i8042] Initializing PS/2 keyboard controller...");

    
    cks(0xAD); 
    cks(0xA7); 

    
    {
        let mut data = Port::<u8>::new(HR_);
        let mut status = Port::<u8>::new(HS_);
        for _ in 0..64 {
            if unsafe { status.read() } & 0x01 == 0 { break; }
            unsafe { data.read(); }
        }
    }

    
    cks(0x20);
    let cfg = epx();
    crate::serial_println!("[i8042] Config byte: {:#04x}", cfg);

    
    
    let ipr = (cfg | 0x41) & !0x10;
    cks(0x60); 
    gbk(ipr);

    
    cks(0xAA);
    let test = epx();
    if test == 0x55 {
        crate::serial_println!("[i8042] Self-test PASSED");
    } else {
        crate::serial_println!("[i8042] Self-test returned {:#04x} (expected 0x55) — continuing anyway", test);
        
    }

    
    cks(0x60);
    gbk(ipr);

    
    cks(0xAE);

    
    gbk(0xFF);
    let ack = epx();
    if ack == 0xFA {
        let kal = epx();
        crate::serial_println!("[i8042] Keyboard reset: ACK={:#04x} BAT={:#04x}", ack, kal);
    } else {
        crate::serial_println!("[i8042] Keyboard reset: response {:#04x} (no ACK)", ack);
    }

    
    {
        let mut data = Port::<u8>::new(HR_);
        let mut status = Port::<u8>::new(HS_);
        for _ in 0..64 {
            if unsafe { status.read() } & 0x01 == 0 { break; }
            unsafe { data.read(); }
        }
    }

    crate::serial_println!("[i8042] PS/2 keyboard controller ready");
}


fn cks(cmd: u8) {
    let mut status = Port::<u8>::new(HS_);
    let mut command = Port::<u8>::new(AIM_);
    
    
    for _ in 0..1_000_000 {
        if unsafe { status.read() } & 0x02 == 0 { break; }
        core::hint::spin_loop();
    }
    unsafe { command.write(cmd); }
}


fn gbk(data: u8) {
    let mut status = Port::<u8>::new(HS_);
    let mut port = Port::<u8>::new(HR_);
    for _ in 0..1_000_000 {
        if unsafe { status.read() } & 0x02 == 0 { break; }
        core::hint::spin_loop();
    }
    unsafe { port.write(data); }
}


fn epx() -> u8 {
    let mut status = Port::<u8>::new(HS_);
    let mut port = Port::<u8>::new(HR_);
    for _ in 0..1_000_000 {
        if unsafe { status.read() } & 0x01 != 0 {
            return unsafe { port.read() };
        }
        core::hint::spin_loop();
    }
    0xFF 
}


const SV_: usize = 256;


pub const T_: u8 = 0xF0;
pub const S_: u8 = 0xF1;
pub const AI_: u8 = 0xF2;
pub const AJ_: u8 = 0xF3;
pub const CW_: u8 = 0xF4;
pub const CV_: u8 = 0xF5;
pub const DE_: u8 = 0xF6;
pub const AM_: u8 = 0xF7;
pub const AO_: u8 = 0xF8;


pub const BAB_: u8 = 0xD0;      
pub const BAA_: u8 = 0xD1;    
pub const AZZ_: u8 = 0xD2;    
pub const AZW_: u8 = 0xD3;          
pub const AZV_: u8 = 0xD4;        
pub const AZX_: u8 = 0xD5;       
pub const AZY_: u8 = 0xD6;      


struct KeyboardBuffer {
    buffer: [u8; SV_],
    read_pos: usize,
    write_pos: usize,
}

impl KeyboardBuffer {
    const fn new() -> Self {
        Self {
            buffer: [0; SV_],
            read_pos: 0,
            write_pos: 0,
        }
    }

    fn push(&mut self, byte: u8) {
        let iqo = (self.write_pos + 1) % SV_;
        if iqo != self.read_pos {
            self.buffer[self.write_pos] = byte;
            self.write_pos = iqo;
        }
    }

    fn pop(&mut self) -> Option<u8> {
        if self.read_pos == self.write_pos {
            None
        } else {
            let byte = self.buffer[self.read_pos];
            self.read_pos = (self.read_pos + 1) % SV_;
            Some(byte)
        }
    }

    fn is_empty(&self) -> bool {
        self.read_pos == self.write_pos
    }
}


const FW_: usize = 32;

struct CommandHistory {
    entries: [Option<String>; FW_],
    write_pos: usize,
    browse_pos: usize,
    count: usize,
}

impl CommandHistory {
    const fn new() -> Self {
        
        Self {
            entries: [
                None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None,
            ],
            write_pos: 0,
            browse_pos: 0,
            count: 0,
        }
    }
    
    fn add(&mut self, cmd: &str) {
        if cmd.is_empty() {
            return;
        }
        
        if self.count > 0 {
            let mwp = if self.write_pos == 0 { FW_ - 1 } else { self.write_pos - 1 };
            if let Some(ref last) = self.entries[mwp] {
                if last == cmd {
                    self.browse_pos = self.write_pos;
                    return;
                }
            }
        }
        
        self.entries[self.write_pos] = Some(String::from(cmd));
        self.write_pos = (self.write_pos + 1) % FW_;
        if self.count < FW_ {
            self.count += 1;
        }
        self.browse_pos = self.write_pos;
    }
    
    fn get_prev(&mut self) -> Option<&str> {
        if self.count == 0 {
            return None;
        }
        
        let gjf = if self.browse_pos == 0 { 
            FW_ - 1 
        } else { 
            self.browse_pos - 1 
        };
        
        
        let ise = if self.count < FW_ {
            0
        } else {
            self.write_pos
        };
        
        if gjf == ise && self.browse_pos == ise {
            
            return self.entries[self.browse_pos].as_deref();
        }
        
        if self.entries[gjf].is_some() {
            self.browse_pos = gjf;
            self.entries[self.browse_pos].as_deref()
        } else {
            None
        }
    }
    
    fn get_next(&mut self) -> Option<&str> {
        if self.browse_pos == self.write_pos {
            return None; 
        }
        
        self.browse_pos = (self.browse_pos + 1) % FW_;
        
        if self.browse_pos == self.write_pos {
            None 
        } else {
            self.entries[self.browse_pos].as_deref()
        }
    }
    
    fn reset_browse(&mut self) {
        self.browse_pos = self.write_pos;
    }
    
    fn iter(&self) -> impl Iterator<Item = (usize, &str)> {
        let count = self.count;
        let start = if count < FW_ { 0 } else { self.write_pos };
        
        (0..count).map(move |i| {
            let idx = (start + i) % FW_;
            (i + 1, self.entries[idx].as_deref().unwrap_or(""))
        })
    }
}


static VR_: Mutex<KeyboardBuffer> = Mutex::new(KeyboardBuffer::new());


static NJ_: Mutex<CommandHistory> = Mutex::new(CommandHistory::new());

static Ww: Mutex<Option<String>> = Mutex::new(None);


static UG_: AtomicBool = AtomicBool::new(false);


static ADG_: core::sync::atomic::AtomicU8 = core::sync::atomic::AtomicU8::new(0);


static RW_: AtomicBool = AtomicBool::new(false);


static LU_: AtomicBool = AtomicBool::new(false);

static ABH_: AtomicBool = AtomicBool::new(false);

static AHX_: AtomicBool = AtomicBool::new(false);

static BR_: AtomicBool = AtomicBool::new(false);


static BAD_: Mutex<[u8; 32]> = Mutex::new([0u8; 32]);


static DWC_: core::sync::atomic::AtomicU8 = core::sync::atomic::AtomicU8::new(0xFF);

static EFZ_: core::sync::atomic::AtomicU8 = core::sync::atomic::AtomicU8::new(0);


const CTW_: [u8; 128] = [
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


const CTX_: [u8; 128] = [
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
fn ajh(byte: u8) {
    if let Some(mut buf) = VR_.try_lock() {
        buf.push(byte);
    }
    
}


pub fn handle_scancode(scancode: u8) {
    
    
    
    
    let skip = ADG_.load(Ordering::SeqCst);
    if skip > 0 {
        ADG_.store(skip - 1, Ordering::SeqCst);
        return;
    }
    if scancode == 0xE1 {
        
        ADG_.store(5, Ordering::SeqCst);
        return;
    }
    
    
    
    
    
    if scancode == 0x00 || scancode == 0xFF || scancode == 0xFA 
        || scancode == 0xFE || scancode == 0xFC
        || scancode == 0xEE || scancode == 0xAB {
        return;
    }
    
    
    
    
    let geo = scancode & 0x7F;
    if geo >= 0x47 && geo <= 0x53 {
        if !UG_.load(Ordering::SeqCst) && !AHX_.load(Ordering::SeqCst) {
            
            
            
            let adx = scancode & 0x80 != 0;
            if !adx {
                let cqw = match geo {
                    0x47 => Some(CW_),
                    0x48 => Some(T_),
                    0x49 => Some(AM_),
                    0x4B => Some(AI_),
                    0x4D => Some(AJ_),
                    0x4F => Some(CV_),
                    0x50 => Some(S_),
                    0x51 => Some(AO_),
                    0x53 => Some(DE_),
                    _ => None,
                };
                if let Some(k) = cqw {
                    ajh(k);
                }
            }
            return;
        }
    }
    
    
    if scancode == 0xE0 {
        UG_.store(true, Ordering::SeqCst);
        return;
    }
    
    let ihw = UG_.load(Ordering::SeqCst);
    UG_.store(false, Ordering::SeqCst);
    
    
    let adx = scancode & 0x80 != 0;
    let key = scancode & 0x7F;
    
    
    ppw(key, !adx);
    
    
    if ihw {
        
        if key == 0x1D {
            BR_.store(!adx, Ordering::SeqCst);
            if !adx {
                crate::accessibility::eak(crate::accessibility::StickyModifier::Ctrl);
            }
            return;
        }
        if key == 0x38 {
            RW_.store(!adx, Ordering::SeqCst);
            if !adx {
                crate::accessibility::eak(crate::accessibility::StickyModifier::Alt);
            }
            return;
        }
        
        
        if !adx {
            let adf = RW_.load(Ordering::SeqCst);
            let ctrl = BR_.load(Ordering::SeqCst);
            
            let cqw = match key {
                0x48 if adf  => Some(AZW_),
                0x50 if adf  => Some(AZV_),
                0x4B if ctrl => Some(AZX_),
                0x4D if ctrl => Some(AZY_),
                0x48 => Some(T_),
                0x50 => Some(S_),
                0x4B => Some(AI_),
                0x4D => Some(AJ_),
                0x47 => Some(CW_),
                0x4F => Some(CV_),
                0x53 => Some(DE_),
                0x71 => Some(DE_), 
                0x75 => Some(T_),      
                0x72 => Some(S_),    
                0x6B => Some(AI_),    
                0x74 => Some(AJ_),   
                0x6C => Some(CW_),    
                0x69 => Some(CV_),     
                0x49 => Some(AM_),
                0x51 => Some(AO_),
                _ => None,
            };
            if let Some(k) = cqw {
                ajh(k);
            }
        }
        return;
    }

    
    if !ihw && !adx && key == 0x29 {
        ajh(b' ');
        return;
    }
    
    
    if key == 0x1D {
        BR_.store(!adx, Ordering::SeqCst);
        
        if !adx {
            crate::accessibility::eak(crate::accessibility::StickyModifier::Ctrl);
        }
        return;
    }
    
    
    if key == 0x2A || key == 0x36 {
        
        LU_.store(!adx, Ordering::SeqCst);
        if !adx {
            crate::accessibility::eak(crate::accessibility::StickyModifier::Shift);
        }
        return;
    }
    
    
    if key == 0x3A && !adx {
        let current = ABH_.load(Ordering::SeqCst);
        ABH_.store(!current, Ordering::SeqCst);
        return;
    }
    
    
    if key == 0x45 && !adx {
        let current = AHX_.load(Ordering::SeqCst);
        AHX_.store(!current, Ordering::SeqCst);
        return;
    }
    
    
    if adx {
        return;
    }
    
    
    if BR_.load(Ordering::SeqCst) && key == 0x1E {
        ajh(1); 
        return;
    }

    
    if BR_.load(Ordering::SeqCst) && key == 0x2E {
        ajh(3); 
        return;
    }

    
    if BR_.load(Ordering::SeqCst) && key == 0x2F {
        ajh(0x16); 
        return;
    }

    
    if BR_.load(Ordering::SeqCst) && key == 0x2D {
        ajh(0x18); 
        return;
    }

    
    if BR_.load(Ordering::SeqCst) && key == 0x26 {
        ajh(12); 
        return;
    }

    
    if BR_.load(Ordering::SeqCst) && key == 0x1F {
        ajh(0x13); 
        return;
    }

    
    if BR_.load(Ordering::SeqCst) && key == 0x22 {
        ajh(0x07); 
        return;
    }

    
    if BR_.load(Ordering::SeqCst) && key == 0x21 {
        ajh(0x06); 
        return;
    }

    
    if BR_.load(Ordering::SeqCst) && key == 0x23 {
        ajh(0x12); 
        return;
    }

    
    if BR_.load(Ordering::SeqCst) && key == 0x2C {
        ajh(0x1A); 
        return;
    }

    
    if BR_.load(Ordering::SeqCst) && key == 0x15 {
        ajh(0x19); 
        return;
    }

    
    if BR_.load(Ordering::SeqCst) && key == 0x35 {
        ajh(BAB_);
        return;
    }

    
    if BR_.load(Ordering::SeqCst) && LU_.load(Ordering::SeqCst) && key == 0x25 {
        ajh(BAA_);
        return;
    }

    
    if BR_.load(Ordering::SeqCst) && LU_.load(Ordering::SeqCst) && key == 0x20 {
        ajh(AZZ_);
        return;
    }

    
    let no = LU_.load(Ordering::SeqCst)
        || crate::accessibility::erv(crate::accessibility::StickyModifier::Shift);
    let caps = ABH_.load(Ordering::SeqCst);
    
    let ascii = if key < 128 {
        let base = if no {
            CTX_[key as usize]
        } else {
            CTW_[key as usize]
        };
        
        
        if caps && base >= b'a' && base <= b'z' {
            base - 32 
        } else if caps && base >= b'A' && base <= b'Z' {
            base + 32 
        } else {
            base
        }
    } else {
        0
    };
    
    
    if ascii != 0 {
        
        ajh(ascii);
        
        crate::accessibility::oxi();
    }
}


pub fn ya() -> Option<u8> {
    
    
    let result = crate::arch::bag(|| {
        VR_.lock().pop()
    });
    if let Some(b) = result {
        return Some(b);
    }
    serial::read_byte()
}


pub fn nzs(ascii: u8) {
    if ascii != 0 {
        crate::arch::bag(|| {
            VR_.lock().push(ascii);
        });
    }
}


pub fn has_input() -> bool {
    crate::arch::bag(|| {
        !VR_.lock().is_empty()
    })
}



pub fn sx(scancode: u8) -> bool {
    
    
    crate::arch::bag(|| {
        match scancode {
            0x38 => RW_.load(Ordering::Relaxed)
                || crate::accessibility::erv(crate::accessibility::StickyModifier::Alt),
            0x1D => BR_.load(Ordering::Relaxed)
                || crate::accessibility::erv(crate::accessibility::StickyModifier::Ctrl),
            0x2A | 0x36 => LU_.load(Ordering::Relaxed)
                || crate::accessibility::erv(crate::accessibility::StickyModifier::Shift),
            _ => {
                let state = BAD_.lock();
                let yk = (scancode / 8) as usize;
                let bew = scancode % 8;
                if yk < 32 {
                    (state[yk] & (1 << bew)) != 0
                } else {
                    false
                }
            }
        }
    })
}


fn ppw(scancode: u8, pressed: bool) {
    
    match scancode {
        0x38 => {
            RW_.store(pressed, Ordering::Relaxed);
            
            if pressed {
                crate::accessibility::eak(crate::accessibility::StickyModifier::Alt);
            }
        }
        0x1D => BR_.store(pressed, Ordering::Relaxed),
        0x2A | 0x36 => LU_.store(pressed, Ordering::Relaxed),
        _ => {}
    }
    
    
    let mut state = BAD_.lock();
    let yk = (scancode / 8) as usize;
    let bew = scancode % 8;
    if yk < 32 {
        if pressed {
            state[yk] |= 1 << bew;
        } else {
            state[yk] &= !(1 << bew);
        }
    }
}


pub fn fgf(cmd: &str) {
    NJ_.lock().add(cmd);
}


pub fn gat() -> Option<String> {
    NJ_.lock().get_prev().map(String::from)
}


pub fn gas() -> Option<String> {
    NJ_.lock().get_next().map(String::from)
}


pub fn gau() {
    NJ_.lock().reset_browse();
}


pub fn mlp() -> Vec<(usize, String)> {
    NJ_.lock().iter().map(|(i, j)| (i, String::from(j))).collect()
}

pub fn byb(text: &str) {
    *Ww.lock() = Some(String::from(text));
}

pub fn hln() -> Option<String> {
    Ww.lock().as_ref().map(|j| j.clone())
}


pub fn ocu(buffer: &mut [u8]) -> usize {
    let mut pos = 0;
    let mut cursor = 0; 
    let mut qcc = String::new(); 
    let mut amr = false;
    
    
    gau();
    
    loop {
        if let Some(c) = ya() {
            match c {
                b'\n' | b'\r' => {
                    crate::println!();
                    
                    let cmd = core::str::from_utf8(&buffer[..pos]).unwrap_or("");
                    if !cmd.trim().is_empty() {
                        fgf(cmd);
                    }
                    break;
                }
                0x01 => {
                    
                    amr = true;
                }
                0x08 => {
                    
                    if amr {
                        
                        while cursor > 0 {
                            crate::print!("\x08");
                            cursor -= 1;
                        }
                        for _ in 0..pos {
                            crate::print!(" ");
                        }
                        for _ in 0..pos {
                            crate::print!("\x08");
                        }
                        pos = 0;
                        cursor = 0;
                        amr = false;
                    } else if cursor > 0 {
                        
                        for i in cursor..pos {
                            buffer[i - 1] = buffer[i];
                        }
                        pos = pos.saturating_sub(1);
                        cursor = cursor.saturating_sub(1);
                        
                        
                        crate::print!("\x08");
                        for i in cursor..pos {
                            crate::print!("{}", buffer[i] as char);
                        }
                        crate::print!(" ");
                        for _ in cursor..=pos {
                            crate::print!("\x08");
                        }
                    }
                }
                T_ => {
                    
                    if let Some(prev) = gat() {
                        amr = false;
                        
                        while cursor > 0 {
                            crate::print!("\x08");
                            cursor -= 1;
                        }
                        for _ in 0..pos {
                            crate::print!(" ");
                        }
                        for _ in 0..pos {
                            crate::print!("\x08");
                        }
                        
                        let bytes = prev.as_bytes();
                        let len = bytes.len().min(buffer.len() - 1);
                        buffer[..len].copy_from_slice(&bytes[..len]);
                        pos = len;
                        cursor = len;
                        crate::print!("{}", &prev[..len]);
                    }
                }
                S_ => {
                    
                    let next = gas();
                    amr = false;
                    
                    while cursor > 0 {
                        crate::print!("\x08");
                        cursor -= 1;
                    }
                    for _ in 0..pos {
                        crate::print!(" ");
                    }
                    for _ in 0..pos {
                        crate::print!("\x08");
                    }
                    
                    if let Some(next_cmd) = next {
                        let bytes = next_cmd.as_bytes();
                        let len = bytes.len().min(buffer.len() - 1);
                        buffer[..len].copy_from_slice(&bytes[..len]);
                        pos = len;
                        cursor = len;
                        crate::print!("{}", &next_cmd[..len]);
                    } else {
                        pos = 0;
                        cursor = 0;
                    }
                }
                AI_ => {
                    amr = false;
                    if cursor > 0 {
                        cursor -= 1;
                        crate::print!("\x08");
                    }
                }
                AJ_ => {
                    amr = false;
                    if cursor < pos {
                        crate::print!("{}", buffer[cursor] as char);
                        cursor += 1;
                    }
                }
                CW_ => {
                    amr = false;
                    while cursor > 0 {
                        crate::print!("\x08");
                        cursor -= 1;
                    }
                }
                CV_ => {
                    amr = false;
                    while cursor < pos {
                        crate::print!("{}", buffer[cursor] as char);
                        cursor += 1;
                    }
                }
                DE_ => {
                    
                    if amr {
                        
                        while cursor > 0 {
                            crate::print!("\x08");
                            cursor -= 1;
                        }
                        for _ in 0..pos {
                            crate::print!(" ");
                        }
                        for _ in 0..pos {
                            crate::print!("\x08");
                        }
                        pos = 0;
                        cursor = 0;
                        amr = false;
                    } else if cursor < pos {
                        
                        for i in cursor..pos.saturating_sub(1) {
                            buffer[i] = buffer[i + 1];
                        }
                        pos = pos.saturating_sub(1);
                        
                        
                        for i in cursor..pos {
                            crate::print!("{}", buffer[i] as char);
                        }
                        crate::print!(" ");
                        
                        for _ in cursor..=pos {
                            crate::print!("\x08");
                        }
                    }
                }
                12 => {
                    
                    crate::framebuffer::clear();
                    
                    crate::bq!(crate::framebuffer::G_, "trustos");
                    crate::bq!(crate::framebuffer::B_, "> ");
                    for i in 0..pos {
                        crate::print!("{}", buffer[i] as char);
                    }
                    
                    for _ in cursor..pos {
                        crate::print!("\x08");
                    }
                    amr = false;
                }
                3 => {
                    
                    if let Ok(text) = core::str::from_utf8(&buffer[..pos]) {
                        byb(text);
                    }
                    amr = false;
                }
                0x16 => {
                    
                    if let Some(text) = hln() {
                        if amr {
                            
                            while cursor > 0 {
                                crate::print!("\x08");
                                cursor -= 1;
                            }
                            for _ in 0..pos {
                                crate::print!(" ");
                            }
                            for _ in 0..pos {
                                crate::print!("\x08");
                            }
                            pos = 0;
                            cursor = 0;
                            amr = false;
                        }
                        for b in text.bytes() {
                            if b < 0x20 || b >= 0x7F || pos >= buffer.len() - 1 {
                                continue;
                            }
                            if cursor < pos {
                                for i in (cursor..pos).rev() {
                                    buffer[i + 1] = buffer[i];
                                }
                            }
                            buffer[cursor] = b;
                            pos += 1;
                            cursor += 1;

                            for i in cursor - 1..pos {
                                crate::print!("{}", buffer[i] as char);
                            }
                            for _ in cursor..pos {
                                crate::print!("\x08");
                            }
                        }
                    }
                }
                _ if c >= 0x20 && c < 0x7F && pos < buffer.len() - 1 => {
                    
                    if amr {
                        
                        while cursor > 0 {
                            crate::print!("\x08");
                            cursor -= 1;
                        }
                        for _ in 0..pos {
                            crate::print!(" ");
                        }
                        for _ in 0..pos {
                            crate::print!("\x08");
                        }
                        pos = 0;
                        cursor = 0;
                        amr = false;
                    }
                    if cursor < pos {
                        
                        for i in (cursor..pos).rev() {
                            buffer[i + 1] = buffer[i];
                        }
                    }
                    buffer[cursor] = c;
                    pos += 1;
                    cursor += 1;
                    
                    
                    for i in cursor - 1..pos {
                        crate::print!("{}", buffer[i] as char);
                    }
                    
                    for _ in cursor..pos {
                        crate::print!("\x08");
                    }
                }
                _ => {}
            }
        } else {
            
            crate::arch::acb();
        }
    }
    
    buffer[pos] = 0; 
    pos
}


pub fn read_line(buffer: &mut [u8]) -> usize {
    ocu(buffer)
}


pub fn cpb(buffer: &mut [u8]) -> usize {
    let mut pos = 0;
    
    loop {
        if let Some(c) = ya() {
            match c {
                b'\n' | b'\r' => {
                    
                    break;
                }
                0x08 => {
                    
                    if pos > 0 {
                        pos -= 1;
                        buffer[pos] = 0;
                        
                        crate::print!("\x08 \x08");
                    }
                }
                0x03 => {
                    
                    pos = 0;
                    break;
                }
                0x15 => {
                    
                    for _ in 0..pos {
                        crate::print!("\x08 \x08");
                    }
                    pos = 0;
                }
                c if c >= 0x20 && c < 0x7F => {
                    
                    if pos < buffer.len() - 1 {
                        buffer[pos] = c;
                        pos += 1;
                        
                        crate::print!("*");
                    }
                }
                _ => {}
            }
        }
    }
    
    pos
}







pub fn kr() -> Option<u8> {
    ya()
}


pub fn ptj() -> u8 {
    loop {
        if let Some(key) = ya() {
            return key;
        }
        
        for _ in 0..1000 {
            core::hint::spin_loop();
        }
    }
}
