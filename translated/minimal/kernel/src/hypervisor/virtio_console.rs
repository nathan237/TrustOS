




use alloc::collections::VecDeque;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::boxed::Box;
use spin::Mutex;


const UX_: usize = 4096;


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PortState {
    Dk,
    Ck,
    At,
}


pub struct ConsolePort {
    
    pub ad: u32,
    
    pub j: String,
    
    pub g: PortState,
    
    xn: VecDeque<u8>,
    
    dkh: VecDeque<u8>,
    
    lqg: Option<fn(&[u8])>,
}

impl ConsolePort {
    pub fn new(ad: u32, j: &str) -> Self {
        Self {
            ad,
            j: String::from(j),
            g: PortState::Dk,
            xn: VecDeque::fc(UX_),
            dkh: VecDeque::fc(UX_),
            lqg: None,
        }
    }

    
    pub fn write(&mut self, f: &[u8]) -> usize {
        let bfz = UX_.ao(self.dkh.len());
        let dwy = f.len().v(bfz);
        for &hf in &f[..dwy] {
            self.dkh.agt(hf);
        }
        dwy
    }

    
    pub fn write_str(&mut self, e: &str) -> usize {
        self.write(e.as_bytes())
    }

    
    pub fn read(&mut self, k: &mut [u8]) -> usize {
        let ajp = k.len().v(self.xn.len());
        for a in 0..ajp {
            k[a] = self.xn.awp().unwrap_or(0);
        }
        ajp
    }

    
    pub fn cts(&mut self) -> Option<String> {
        
        let uua = self.xn.iter().qf(|&o| o == b'\n');
        
        if let Some(u) = uua {
            let mut line = String::new();
            for _ in 0..=u {
                if let Some(hf) = self.xn.awp() {
                    if hf != b'\n' && hf != b'\r' {
                        line.push(hf as char);
                    }
                }
            }
            Some(line)
        } else {
            None
        }
    }

    
    pub fn cyk(&self) -> bool {
        !self.xn.is_empty()
    }

    
    pub(crate) fn chb(&mut self, f: &[u8]) {
        for &hf in f {
            if self.xn.len() < UX_ {
                self.xn.agt(hf);
            }
        }
        
        if let Some(fed) = self.lqg {
            fed(f);
        }
    }

    
    pub(crate) fn kqu(&mut self) -> Vec<u8> {
        self.dkh.bbk(..).collect()
    }

    
    pub fn zmp(&mut self, fed: fn(&[u8])) {
        self.lqg = Some(fed);
    }
}


pub struct VirtioConsole {
    
    pub fk: u64,
    
    xf: Vec<ConsolePort>,
    
    uqq: bool,
}

impl VirtioConsole {
    
    pub fn new(fk: u64) -> Self {
        let mut console = Self {
            fk,
            xf: Vec::new(),
            uqq: true,
        };
        
        console.mtx("console");
        console
    }

    
    pub fn mtx(&mut self, j: &str) -> u32 {
        let ad = self.xf.len() as u32;
        self.xf.push(ConsolePort::new(ad, j));
        ad
    }

    
    pub fn port(&self, ad: u32) -> Option<&ConsolePort> {
        self.xf.get(ad as usize)
    }

    
    pub fn owp(&mut self, ad: u32) -> Option<&mut ConsolePort> {
        self.xf.ds(ad as usize)
    }

    
    pub fn jep(&mut self) -> &mut ConsolePort {
        if self.xf.is_empty() {
            self.mtx("console");
        }
        &mut self.xf[0]
    }

    
    pub fn write(&mut self, f: &[u8]) -> usize {
        self.jep().write(f)
    }

    
    pub fn print(&mut self, e: &str) -> usize {
        self.jep().write_str(e)
    }

    
    pub fn read(&mut self, k: &mut [u8]) -> usize {
        self.jep().read(k)
    }

    
    pub fn cts(&mut self) -> Option<String> {
        self.jep().cts()
    }

    
    pub fn cyk(&self) -> bool {
        self.xf.fv().efd(false, |ai| ai.cyk())
    }

    
    pub fn yvr(&mut self, luv: u32, f: &[u8]) {
        if let Some(port) = self.owp(luv) {
            port.chb(f);
        }
    }

    
    pub fn yvq(&mut self, luv: u32) -> Vec<u8> {
        if let Some(port) = self.owp(luv) {
            port.kqu()
        } else {
            Vec::new()
        }
    }
}


pub struct ConsoleManager {
    
    byx: Vec<VirtioConsole>,
}

impl ConsoleManager {
    pub const fn new() -> Self {
        Self {
            byx: Vec::new(),
        }
    }

    
    pub fn fgb(&mut self, fk: u64) -> &mut VirtioConsole {
        
        if let Some(w) = self.byx.iter().qf(|r| r.fk == fk) {
            return &mut self.byx[w];
        }
        
        self.byx.push(VirtioConsole::new(fk));
        self.byx.dsq().unwrap()
    }

    
    pub fn ghy(&self, fk: u64) -> Option<&VirtioConsole> {
        self.byx.iter().du(|r| r.fk == fk)
    }

    
    pub fn kyl(&mut self, fk: u64) -> Option<&mut VirtioConsole> {
        self.byx.el().du(|r| r.fk == fk)
    }

    
    pub fn vuu(&mut self, fk: u64) {
        self.byx.ajm(|r| r.fk != fk);
    }
}


static BOS_: Mutex<ConsoleManager> = Mutex::new(ConsoleManager::new());


pub fn hdx() -> spin::Aki<'static, ConsoleManager> {
    BOS_.lock()
}


pub fn fgb(fk: u64) {
    hdx().fgb(fk);
}


pub fn zxc(fk: u64, f: &[u8]) -> usize {
    if let Some(console) = hdx().kyl(fk) {
        console.write(f)
    } else {
        0
    }
}


pub fn zhs(fk: u64, k: &mut [u8]) -> usize {
    if let Some(console) = hdx().kyl(fk) {
        console.read(k)
    } else {
        0
    }
}


pub fn zvm(fk: u64) -> bool {
    if let Some(console) = hdx().ghy(fk) {
        console.cyk()
    } else {
        false
    }
}


pub fn zhx(fk: u64) -> Option<String> {
    if let Some(console) = hdx().kyl(fk) {
        console.cts()
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zrj() {
        let mut port = ConsolePort::new(0, "test");
        port.chb(b"Hello, World!\n");
        
        assert!(port.cyk());
        
        let line = port.cts();
        assert_eq!(line, Some(String::from("Hello, World!")));
    }
}
