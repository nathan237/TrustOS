




extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use super::{kw, nk, apm,
            T_, F_, O_, AK_, AO_, BB_, BO_};


struct Z {
    j: &'static str,
    gb: &'static str,
    dc: &'static str,
}

const Bda: &[Z] = &[
    
    Z { j: "ls", gb: "FS", dc: "List directory contents" },
    Z { j: "cd", gb: "FS", dc: "Change directory" },
    Z { j: "pwd", gb: "FS", dc: "Print working directory" },
    Z { j: "mkdir", gb: "FS", dc: "Create directory" },
    Z { j: "rmdir", gb: "FS", dc: "Remove directory" },
    Z { j: "touch", gb: "FS", dc: "Create empty file" },
    Z { j: "rm", gb: "FS", dc: "Remove file" },
    Z { j: "cp", gb: "FS", dc: "Copy file" },
    Z { j: "mv", gb: "FS", dc: "Move or rename file" },
    Z { j: "cat", gb: "FS", dc: "Display file contents" },
    Z { j: "head", gb: "FS", dc: "Show first lines of file" },
    Z { j: "tail", gb: "FS", dc: "Show last lines of file" },
    Z { j: "tree", gb: "FS", dc: "Show directory tree" },
    Z { j: "find", gb: "FS", dc: "Search for files" },
    Z { j: "stat", gb: "FS", dc: "Display file statistics" },
    Z { j: "hexdump", gb: "FS", dc: "Hex dump of file" },
    
    Z { j: "help", gb: "SYS", dc: "Show command help" },
    Z { j: "clear", gb: "SYS", dc: "Clear terminal screen" },
    Z { j: "time", gb: "SYS", dc: "Show current time" },
    Z { j: "uptime", gb: "SYS", dc: "Show system uptime" },
    Z { j: "date", gb: "SYS", dc: "Show current date" },
    Z { j: "whoami", gb: "SYS", dc: "Show current user" },
    Z { j: "uname", gb: "SYS", dc: "System information" },
    Z { j: "ps", gb: "SYS", dc: "List running processes" },
    Z { j: "free", gb: "SYS", dc: "Display memory usage" },
    Z { j: "top", gb: "SYS", dc: "System monitor" },
    Z { j: "dmesg", gb: "SYS", dc: "Kernel message buffer" },
    Z { j: "reboot", gb: "SYS", dc: "Reboot the system" },
    Z { j: "shutdown", gb: "SYS", dc: "Shut down the system" },
    
    Z { j: "ifconfig", gb: "NET", dc: "Network interface config" },
    Z { j: "ping", gb: "NET", dc: "Send ICMP echo request" },
    Z { j: "curl", gb: "NET", dc: "Transfer data from URL" },
    Z { j: "wget", gb: "NET", dc: "Download file from URL" },
    Z { j: "nslookup", gb: "NET", dc: "DNS lookup" },
    Z { j: "arp", gb: "NET", dc: "ARP table" },
    Z { j: "netstat", gb: "NET", dc: "Network statistics" },
    
    Z { j: "desktop", gb: "GUI", dc: "Launch graphical desktop" },
    Z { j: "open", gb: "GUI", dc: "Open file with GUI app" },
    Z { j: "trustedit", gb: "GUI", dc: "3D model editor" },
    
    Z { j: "trustview", gb: "DEV", dc: "Binary analysis viewer" },
    Z { j: "trustlang", gb: "DEV", dc: "TrustLang REPL" },
    Z { j: "transpile", gb: "DEV", dc: "Binary-to-Rust transpiler" },
    Z { j: "lab", gb: "DEV", dc: "TrustLab introspection" },
    
    Z { j: "lspci", gb: "HW", dc: "List PCI devices" },
    Z { j: "lshw", gb: "HW", dc: "List hardware" },
    Z { j: "disk", gb: "HW", dc: "Disk information" },
    Z { j: "fdisk", gb: "HW", dc: "Partition table" },
    Z { j: "audio", gb: "HW", dc: "Audio subsystem" },
    Z { j: "beep", gb: "HW", dc: "Play a beep tone" },
    
    Z { j: "neofetch", gb: "FUN", dc: "System info with ASCII art" },
    Z { j: "matrix", gb: "FUN", dc: "Matrix rain animation" },
    Z { j: "cowsay", gb: "FUN", dc: "ASCII cow says message" },
];


pub struct GuideState {
    
    pub anw: String,
    
    pub gi: usize,
    
    pub jc: usize,
    
    pub na: usize,
    
    pub imz: Option<&'static str>,
}

impl GuideState {
    pub fn new() -> Self {
        Self {
            anw: String::new(),
            gi: 0,
            jc: 0,
            na: 0,
            imz: None,
        }
    }
    
    pub fn vr(&mut self, bs: u8) {
        use crate::keyboard::{V_, U_, AM_, AQ_};
        match bs {
            V_ => {
                self.na = self.na.ao(1);
            }
            U_ => {
                self.na += 1;
            }
            AM_ => {
                self.na = self.na.ao(10);
            }
            AQ_ => {
                self.na += 10;
            }
            
            0x08 => {
                if self.gi > 0 {
                    self.gi -= 1;
                    self.anw.remove(self.gi);
                    self.na = 0;
                    self.jc = 0;
                }
            }
            
            0x15 => {
                self.anw.clear();
                self.gi = 0;
                self.na = 0;
                self.jc = 0;
            }
            _ => {}
        }
    }
    
    pub fn fka(&mut self, bm: char) {
        if bm.jbb() || bm == ' ' {
            self.anw.insert(self.gi, bm);
            self.gi += 1;
            self.na = 0;
            self.jc = 0;
        }
    }

    
    pub fn ago(&mut self, bhi: i32, alk: i32, d: u32, dxv: u32) {
        let dt = nk();
        let kq = apm() + 1;
        if kq <= 0 || dt <= 0 { return; }

        
        
        let kgv = kq;
        if alk >= kgv && alk < kgv + kq {
            let fek: [&str; 8] = ["ALL", "FS", "SYS", "NET", "GUI", "DEV", "HW", "FUN"];
            let qwo: [Option<&str>; 8] = [None, Some("FS"), Some("SYS"), Some("NET"), Some("GUI"), Some("DEV"), Some("HW"), Some("FUN")];
            let mut gx = 0i32;
            for (a, rx) in fek.iter().cf() {
                let hpo = gx + (rx.len() as i32 + 1) * dt;
                if bhi >= gx && bhi < hpo {
                    self.imz = qwo[a];
                    self.na = 0;
                    self.jc = 0;
                    return;
                }
                gx = hpo;
                if gx > d as i32 - 10 { break; }
            }
            return;
        }

        
        let ou = kgv + kq + 5; 
        if alk >= ou {
            let br = ((alk - ou) / kq) as usize;
            let dew = self.jc + br;
            let aud = self.nty();
            if dew < aud.len() {
                self.na = dew;
            }
        }
    }
    
    fn nty(&self) -> Vec<&Z> {
        Bda.iter()
            .hi(|r| {
                
                if let Some(rx) = self.imz {
                    if r.gb != rx { return false; }
                }
                
                if !self.anw.is_empty() {
                    let fm = self.anw.avd();
                    let urg = r.j.avd().contains(&fm);
                    let rwa = r.dc.avd().contains(&fm);
                    if !urg && !rwa { return false; }
                }
                true
            })
            .collect()
    }
}


pub fn po(g: &GuideState, b: i32, c: i32, d: u32, i: u32) {
    let dt = nk();
    let kq = apm() + 1;
    if kq <= 0 || dt <= 0 { return; }
    
    let mut ae = c;
    
    
    let wfo = "/ ";
    kw(b, ae, wfo, AK_);
    let cky = b + 2 * dt;
    if g.anw.is_empty() {
        kw(cky, ae, "type to search...", F_);
    } else {
        kw(cky, ae, &g.anw, T_);
    }
    ae += kq;
    
    
    let fek = ["ALL", "FS", "SYS", "NET", "GUI", "DEV", "HW", "FUN"];
    let qwm = [O_, BB_, AK_, AO_, BO_, O_, 0xFFD18616, 0xFFBC8CFF];
    let mut gx = b;
    for (a, rx) in fek.iter().cf() {
        let gh = match g.imz {
            None => a == 0,
            Some(r) => *rx == r,
        };
        let s = if gh { qwm[a] } else { F_ };
        kw(gx, ae, rx, s);
        gx += (rx.len() as i32 + 1) * dt;
        if gx > b + d as i32 - 10 { break; }
    }
    ae += kq + 2;
    
    
    crate::framebuffer::ah(b as u32, ae as u32, d, 1, 0xFF30363D);
    ae += 3;
    
    
    let aud = g.nty();
    let iw = ((i as i32 - (ae - c)) / kq) as usize;
    
    if aud.is_empty() {
        kw(b + 4, ae, "No matching commands", F_);
        return;
    }
    
    
    let na = g.na.v(aud.len().ao(1));
    
    
    let jc = if na >= g.jc + iw {
        na - iw + 1
    } else if na < g.jc {
        na
    } else {
        g.jc
    };
    
    let ci = (jc + iw).v(aud.len());
    
    for a in jc..ci {
        let cmd = aud[a];
        let qe = a == na;
        
        
        if qe {
            crate::framebuffer::ah(b as u32, ae as u32, d, kq as u32, 0xFF1F2937);
        }
        
        
        let qwl = match cmd.gb {
            "FS" => BB_,
            "SYS" => AK_,
            "NET" => AO_,
            "GUI" => BO_,
            "DEV" => O_,
            "HW" => 0xFFD18616,
            "FUN" => 0xFFBC8CFF,
            _ => F_,
        };
        kw(b + 2, ae, cmd.gb, qwl);
        
        
        let dac = b + 6 * dt;
        kw(dac, ae, cmd.j, if qe { O_ } else { T_ });
        
        
        let hfw = dac + 14 * dt;
        let fnu = if dt > 0 { ((d as i32 - (hfw - b)) / dt) as usize } else { 20 };
        let desc = if cmd.dc.len() > fnu && fnu > 0 {
            &cmd.dc[..fnu]
        } else {
            cmd.dc
        };
        kw(hfw, ae, desc, F_);
        
        ae += kq;
        if ae > c + i as i32 { break; }
    }
    
    
    let ffy = format!("{}/{}", aud.len(), Bda.len());
    let kkx = b + d as i32 - (ffy.len() as i32 * dt) - 4;
    kw(kkx, c, &ffy, F_);
}
