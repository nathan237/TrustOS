




use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use alloc::collections::VecDeque;
use spin::Mutex;


const MM_: usize = 4096;


pub struct VirtConsole {
    
    fk: u64,
    
    dkh: VecDeque<u8>,
    
    xn: VecDeque<u8>,
    
    gfw: bool,
    
    ueu: String,
    
    j: String,
}

impl VirtConsole {
    pub fn new(fk: u64, j: &str) -> Self {
        VirtConsole {
            fk,
            dkh: VecDeque::fc(MM_),
            xn: VecDeque::fc(MM_),
            gfw: true,
            ueu: String::new(),
            j: String::from(j),
        }
    }
    
    
    pub fn cco(&mut self, hf: u8) {
        if self.dkh.len() < MM_ {
            self.dkh.agt(hf);
        }
        
        
        let r = hf as char;
        crate::serial_print!("{}", r);
    }
    
    
    pub fn write_str(&mut self, e: &str) {
        for hf in e.bf() {
            self.cco(hf);
        }
    }
    
    
    pub fn dlb(&mut self) -> Option<u8> {
        self.xn.awp()
    }
    
    
    pub fn hmo(&self) -> bool {
        !self.xn.is_empty()
    }
    
    
    pub fn yye(&self) -> u8 {
        if self.xn.is_empty() {
            0x00 
        } else {
            0x01 
        }
    }
    
    
    pub fn zel(&self) -> u8 {
        if self.dkh.len() < MM_ {
            0x20 
        } else {
            0x00 
        }
    }
    
    
    pub fn hoa(&mut self, f: &[u8]) {
        for &hf in f {
            if self.xn.len() < MM_ {
                self.xn.agt(hf);
            }
        }
    }
    
    
    pub fn yxz(&mut self, line: &str) {
        self.hoa(line.as_bytes());
        self.hoa(b"\n");
    }
    
    
    pub fn kqu(&mut self) -> Vec<u8> {
        self.dkh.bbk(..).collect()
    }
    
    
    pub fn saz(&mut self) -> String {
        let bf: Vec<u8> = self.kqu();
        String::azw(&bf).bkc()
    }
    
    
    pub fn zez(&self) -> String {
        let bf: Vec<u8> = self.dkh.iter().hu().collect();
        String::azw(&bf).bkc()
    }
}


pub struct ConsoleManager {
    byx: Vec<VirtConsole>,
}

impl ConsoleManager {
    pub const fn new() -> Self {
        ConsoleManager {
            byx: Vec::new(),
        }
    }
    
    pub fn fgb(&mut self, fk: u64, j: &str) -> usize {
        let console = VirtConsole::new(fk, j);
        let w = self.byx.len();
        self.byx.push(console);
        w
    }
    
    pub fn ghy(&mut self, fk: u64) -> Option<&mut VirtConsole> {
        self.byx.el().du(|r| r.fk == fk)
    }
    
    pub fn vuu(&mut self, fk: u64) {
        self.byx.ajm(|r| r.fk != fk);
    }
}


static JS_: Mutex<ConsoleManager> = Mutex::new(ConsoleManager::new());


pub fn ghy(fk: u64) -> Option<spin::Aki<'static, ConsoleManager>> {
    let aas = JS_.lock();
    Some(aas)
}


pub fn fgb(fk: u64, j: &str) -> usize {
    JS_.lock().fgb(fk, j)
}


pub fn write_char(bjq: usize, bm: char) {
    let mut aas = JS_.lock();
    if bjq < aas.byx.len() {
        aas.byx[bjq].cco(bm as u8);
    }
}


pub fn oac(fk: u64, port: u16, rm: bool, bn: u8) -> u8 {
    let mut aas = JS_.lock();
    
    if let Some(console) = aas.ghy(fk) {
        match port {
            
            0x3F8 => {
                if rm {
                    console.cco(bn);
                    0
                } else {
                    console.dlb().unwrap_or(0)
                }
            }
            
            0x3FD => {
                let mut status = 0x60; 
                if console.hmo() {
                    status |= 0x01; 
                }
                status
            }
            
            0xE9 => {
                if rm {
                    console.cco(bn);
                }
                0
            }
            _ => 0xFF,
        }
    } else {
        0xFF
    }
}


pub fn hoa(fk: u64, f: &[u8]) {
    let mut aas = JS_.lock();
    if let Some(console) = aas.ghy(fk) {
        console.hoa(f);
    }
}


pub fn teh(fk: u64) -> String {
    let mut aas = JS_.lock();
    if let Some(console) = aas.ghy(fk) {
        console.saz()
    } else {
        String::new()
    }
}
