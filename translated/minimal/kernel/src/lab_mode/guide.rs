




extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use super::{eh, ew, qu,
            P_, F_, M_, AC_, AK_, AU_, BG_};


struct R {
    name: &'static str,
    category: &'static str,
    description: &'static str,
}

const Wx: &[R] = &[
    
    R { name: "ls", category: "FS", description: "List directory contents" },
    R { name: "cd", category: "FS", description: "Change directory" },
    R { name: "pwd", category: "FS", description: "Print working directory" },
    R { name: "mkdir", category: "FS", description: "Create directory" },
    R { name: "rmdir", category: "FS", description: "Remove directory" },
    R { name: "touch", category: "FS", description: "Create empty file" },
    R { name: "rm", category: "FS", description: "Remove file" },
    R { name: "cp", category: "FS", description: "Copy file" },
    R { name: "mv", category: "FS", description: "Move or rename file" },
    R { name: "cat", category: "FS", description: "Display file contents" },
    R { name: "head", category: "FS", description: "Show first lines of file" },
    R { name: "tail", category: "FS", description: "Show last lines of file" },
    R { name: "tree", category: "FS", description: "Show directory tree" },
    R { name: "find", category: "FS", description: "Search for files" },
    R { name: "stat", category: "FS", description: "Display file statistics" },
    R { name: "hexdump", category: "FS", description: "Hex dump of file" },
    
    R { name: "help", category: "SYS", description: "Show command help" },
    R { name: "clear", category: "SYS", description: "Clear terminal screen" },
    R { name: "time", category: "SYS", description: "Show current time" },
    R { name: "uptime", category: "SYS", description: "Show system uptime" },
    R { name: "date", category: "SYS", description: "Show current date" },
    R { name: "whoami", category: "SYS", description: "Show current user" },
    R { name: "uname", category: "SYS", description: "System information" },
    R { name: "ps", category: "SYS", description: "List running processes" },
    R { name: "free", category: "SYS", description: "Display memory usage" },
    R { name: "top", category: "SYS", description: "System monitor" },
    R { name: "dmesg", category: "SYS", description: "Kernel message buffer" },
    R { name: "reboot", category: "SYS", description: "Reboot the system" },
    R { name: "shutdown", category: "SYS", description: "Shut down the system" },
    
    R { name: "ifconfig", category: "NET", description: "Network interface config" },
    R { name: "ping", category: "NET", description: "Send ICMP echo request" },
    R { name: "curl", category: "NET", description: "Transfer data from URL" },
    R { name: "wget", category: "NET", description: "Download file from URL" },
    R { name: "nslookup", category: "NET", description: "DNS lookup" },
    R { name: "arp", category: "NET", description: "ARP table" },
    R { name: "netstat", category: "NET", description: "Network statistics" },
    
    R { name: "desktop", category: "GUI", description: "Launch graphical desktop" },
    R { name: "open", category: "GUI", description: "Open file with GUI app" },
    R { name: "trustedit", category: "GUI", description: "3D model editor" },
    
    R { name: "trustview", category: "DEV", description: "Binary analysis viewer" },
    R { name: "trustlang", category: "DEV", description: "TrustLang REPL" },
    R { name: "transpile", category: "DEV", description: "Binary-to-Rust transpiler" },
    R { name: "lab", category: "DEV", description: "TrustLab introspection" },
    
    R { name: "lspci", category: "HW", description: "List PCI devices" },
    R { name: "lshw", category: "HW", description: "List hardware" },
    R { name: "disk", category: "HW", description: "Disk information" },
    R { name: "fdisk", category: "HW", description: "Partition table" },
    R { name: "audio", category: "HW", description: "Audio subsystem" },
    R { name: "beep", category: "HW", description: "Play a beep tone" },
    
    R { name: "neofetch", category: "FUN", description: "System info with ASCII art" },
    R { name: "matrix", category: "FUN", description: "Matrix rain animation" },
    R { name: "cowsay", category: "FUN", description: "ASCII cow says message" },
];


pub struct GuideState {
    
    pub search: String,
    
    pub cursor: usize,
    
    pub scroll: usize,
    
    pub selected: usize,
    
    pub category_filter: Option<&'static str>,
}

impl GuideState {
    pub fn new() -> Self {
        Self {
            search: String::new(),
            cursor: 0,
            scroll: 0,
            selected: 0,
            category_filter: None,
        }
    }
    
    pub fn handle_key(&mut self, key: u8) {
        use crate::keyboard::{T_, S_, AM_, AO_};
        match key {
            T_ => {
                self.selected = self.selected.saturating_sub(1);
            }
            S_ => {
                self.selected += 1;
            }
            AM_ => {
                self.selected = self.selected.saturating_sub(10);
            }
            AO_ => {
                self.selected += 10;
            }
            
            0x08 => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                    self.search.remove(self.cursor);
                    self.selected = 0;
                    self.scroll = 0;
                }
            }
            
            0x15 => {
                self.search.clear();
                self.cursor = 0;
                self.selected = 0;
                self.scroll = 0;
            }
            _ => {}
        }
    }
    
    pub fn handle_char(&mut self, ch: char) {
        if ch.is_ascii_graphic() || ch == ' ' {
            self.search.insert(self.cursor, ch);
            self.cursor += 1;
            self.selected = 0;
            self.scroll = 0;
        }
    }

    
    pub fn handle_click(&mut self, afh: i32, ta: i32, w: u32, _h: u32) {
        let aq = ew();
        let ee = qu() + 1;
        if ee <= 0 || aq <= 0 { return; }

        
        
        let fla = ee;
        if ta >= fla && ta < fla + ee {
            let cgs: [&str; 8] = ["ALL", "FS", "SYS", "NET", "GUI", "DEV", "HW", "FUN"];
            let khr: [Option<&str>; 8] = [None, Some("FS"), Some("SYS"), Some("NET"), Some("GUI"), Some("DEV"), Some("HW"), Some("FUN")];
            let mut bu = 0i32;
            for (i, hx) in cgs.iter().enumerate() {
                let dte = bu + (hx.len() as i32 + 1) * aq;
                if afh >= bu && afh < dte {
                    self.category_filter = khr[i];
                    self.selected = 0;
                    self.scroll = 0;
                    return;
                }
                bu = dte;
                if bu > w as i32 - 10 { break; }
            }
            return;
        }

        
        let gc = fla + ee + 5; 
        if ta >= gc {
            let row = ((ta - gc) / ee) as usize;
            let bfi = self.scroll + row;
            let filtered = self.filtered_commands();
            if bfi < filtered.len() {
                self.selected = bfi;
            }
        }
    }
    
    fn filtered_commands(&self) -> Vec<&R> {
        Wx.iter()
            .filter(|c| {
                
                if let Some(hx) = self.category_filter {
                    if c.category != hx { return false; }
                }
                
                if !self.search.is_empty() {
                    let q = self.search.to_ascii_lowercase();
                    let nho = c.name.to_ascii_lowercase().contains(&q);
                    let ldo = c.description.to_ascii_lowercase().contains(&q);
                    if !nho && !ldo { return false; }
                }
                true
            })
            .collect()
    }
}


pub fn draw(state: &GuideState, x: i32, y: i32, w: u32, h: u32) {
    let aq = ew();
    let ee = qu() + 1;
    if ee <= 0 || aq <= 0 { return; }
    
    let mut u = y;
    
    
    let omm = "/ ";
    eh(x, u, omm, AC_);
    let aua = x + 2 * aq;
    if state.search.is_empty() {
        eh(aua, u, "type to search...", F_);
    } else {
        eh(aua, u, &state.search, P_);
    }
    u += ee;
    
    
    let cgs = ["ALL", "FS", "SYS", "NET", "GUI", "DEV", "HW", "FUN"];
    let khp = [M_, AU_, AC_, AK_, BG_, M_, 0xFFD18616, 0xFFBC8CFF];
    let mut bu = x;
    for (i, hx) in cgs.iter().enumerate() {
        let active = match state.category_filter {
            None => i == 0,
            Some(c) => *hx == c,
        };
        let color = if active { khp[i] } else { F_ };
        eh(bu, u, hx, color);
        bu += (hx.len() as i32 + 1) * aq;
        if bu > x + w as i32 - 10 { break; }
    }
    u += ee + 2;
    
    
    crate::framebuffer::fill_rect(x as u32, u as u32, w, 1, 0xFF30363D);
    u += 3;
    
    
    let filtered = state.filtered_commands();
    let visible = ((h as i32 - (u - y)) / ee) as usize;
    
    if filtered.is_empty() {
        eh(x + 4, u, "No matching commands", F_);
        return;
    }
    
    
    let selected = state.selected.min(filtered.len().saturating_sub(1));
    
    
    let scroll = if selected >= state.scroll + visible {
        selected - visible + 1
    } else if selected < state.scroll {
        selected
    } else {
        state.scroll
    };
    
    let end = (scroll + visible).min(filtered.len());
    
    for i in scroll..end {
        let cmd = filtered[i];
        let hd = i == selected;
        
        
        if hd {
            crate::framebuffer::fill_rect(x as u32, u as u32, w, ee as u32, 0xFF1F2937);
        }
        
        
        let kho = match cmd.category {
            "FS" => AU_,
            "SYS" => AC_,
            "NET" => AK_,
            "GUI" => BG_,
            "DEV" => M_,
            "HW" => 0xFFD18616,
            "FUN" => 0xFFBC8CFF,
            _ => F_,
        };
        eh(x + 2, u, cmd.category, kho);
        
        
        let bcv = x + 6 * aq;
        eh(bcv, u, cmd.name, if hd { M_ } else { P_ });
        
        
        let dmu = bcv + 14 * aq;
        let cmu = if aq > 0 { ((w as i32 - (dmu - x)) / aq) as usize } else { 20 };
        let desc = if cmd.description.len() > cmu && cmu > 0 {
            &cmd.description[..cmu]
        } else {
            cmd.description
        };
        eh(dmu, u, desc, F_);
        
        u += ee;
        if u > y + h as i32 { break; }
    }
    
    
    let cht = format!("{}/{}", filtered.len(), Wx.len());
    let foq = x + w as i32 - (cht.len() as i32 * aq) - 4;
    eh(foq, y, &cht, F_);
}
