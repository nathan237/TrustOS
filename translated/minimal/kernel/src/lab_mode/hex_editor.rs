














extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use super::{eh, ew, qu,
            P_, F_, M_, AC_, AK_, AN_,
            BG_, AU_, DN_};


fn hji(b: u8) -> u32 {
    match b {
        0x00 => F_,                   
        0x09 | 0x0A | 0x0D | 0x20 => AU_,  
        0x01..=0x1F => AK_,         
        0x20..=0x7E => AC_,          
        0x7F => AK_,                
        0xFF => AN_,                   
        0x80..=0xFE => BG_,         
    }
}


fn kgj(b: u8) -> char {
    if b >= 0x20 && b <= 0x7E { b as char } else { '.' }
}


pub struct HexEditorState {
    
    pub data: Vec<u8>,
    
    pub file_path: String,
    
    pub scroll: usize,
    
    pub cursor: usize,
    
    pub bytes_per_row: usize,
    
    pub frame: u64,
}

impl HexEditorState {
    pub fn new() -> Self {
        
        let mut j = Self {
            data: Vec::new(),
            file_path: String::new(),
            scroll: 0,
            cursor: 0,
            bytes_per_row: 16,
            frame: 0,
        };
        
        j.load_file("/home/welcome.txt");
        if j.data.is_empty() {
            j.load_sample();
        }
        j
    }

    
    pub fn load_file(&mut self, path: &str) {
        let data = crate::ramfs::bh(|fs| {
            fs.read_file(path).ok().map(|bytes| bytes.to_vec())
        });
        if let Some(bytes) = data {
            self.data = bytes;
            self.file_path = String::from(path);
            self.cursor = 0;
            self.scroll = 0;
        }
    }

    
    fn load_sample(&mut self) {
        self.file_path = String::from("<sample>");
        self.data.clear();
        
        self.data.extend_from_slice(&[0x7F, b'E', b'L', b'F', 0x02, 0x01, 0x01, 0x00]);
        self.data.extend_from_slice(&[0x00; 8]); 
        
        for i in 0u8..=255 {
            self.data.push(i);
        }
        
        self.data.extend_from_slice(b"TrustOS Kernel v0.1\x00");
        self.data.extend_from_slice(b"Built with Rust\x00\xFF\xFF");
        self.cursor = 0;
        self.scroll = 0;
    }

    pub fn handle_key(&mut self, key: u8) {
        use crate::keyboard::{T_, S_, AI_, AJ_, AM_, AO_};
        let awk = self.bytes_per_row;
        match key {
            T_ => {
                self.cursor = self.cursor.saturating_sub(awk);
            }
            S_ => {
                if self.cursor + awk < self.data.len() {
                    self.cursor += awk;
                }
            }
            AI_ => {
                self.cursor = self.cursor.saturating_sub(1);
            }
            AJ_ => {
                if self.cursor + 1 < self.data.len() {
                    self.cursor += 1;
                }
            }
            AM_ => {
                self.cursor = self.cursor.saturating_sub(awk * 8);
            }
            AO_ => {
                self.cursor = (self.cursor + awk * 8).min(self.data.len().saturating_sub(1));
            }
            _ => {}
        }
    }

    
    pub fn handle_click(&mut self, x: i32, y: i32, w: u32, h: u32) {
        let aq = ew();
        let ee = qu() + 1;
        if ee <= 0 || aq <= 0 { return; }

        
        let gc = ee + 2;
        if y < gc { return; }

        let row = ((y - gc) / ee) as usize;
        let pdh = self.scroll + row;

        
        let ieu = 10 * aq;
        
        if x >= ieu {
            let ot = (x - ieu) as usize;
            let dkn = ot / aq as usize;
            
            let kgl = if dkn >= 25 {
                (dkn - 1) / 3
            } else {
                dkn / 3
            };
            let yk = pdh * self.bytes_per_row + kgl.min(self.bytes_per_row - 1);
            if yk < self.data.len() {
                self.cursor = yk;
            }
        }
    }
}


pub fn draw(state: &HexEditorState, x: i32, y: i32, w: u32, h: u32) {
    let aq = ew();
    let ee = qu() + 1;
    if ee <= 0 || aq <= 0 { return; }

    
    let header = if state.file_path.is_empty() {
        String::from("No file loaded — use: hex <path>")
    } else {
        format!("{} ({} bytes)", state.file_path, state.data.len())
    };
    eh(x, y, &header, M_);

    
    let iry = format!("@{:04X}", state.cursor);
    let dbz = x + w as i32 - (iry.len() as i32 * aq) - 2;
    eh(dbz, y, &iry, F_);

    let gc = y + ee + 2;
    let abc = h as i32 - ee - 2;
    if abc <= 0 { return; }

    let yh = (abc / ee) as usize;
    if state.data.is_empty() {
        eh(x + 4, gc, "Empty — type 'hex <path>' to load a file", F_);
        return;
    }

    let awk = state.bytes_per_row;
    let crx = (state.data.len() + awk - 1) / awk;

    
    let cursor_row = state.cursor / awk;
    let scroll = if cursor_row >= state.scroll + yh {
        cursor_row - yh + 1
    } else if cursor_row < state.scroll {
        cursor_row
    } else {
        state.scroll
    };

    let fuv = (scroll + yh).min(crx);
    let mut u = gc;

    
    let mut kp = String::from("Offset    ");
    for i in 0..awk.min(16) {
        if i == 8 { kp.push(' '); }
        kp.push_str(&format!("{:02X} ", i));
    }
    kp.push_str(" ASCII");
    eh(x, u, &kp, F_);
    u += ee;

    for row in scroll..fuv {
        let offset = row * awk;
        
        let gkk = format!("{:08X}  ", offset);
        eh(x, u, &gkk, F_);

        let ckp = x + 10 * aq;
        let mut aib = ckp;

        
        let azm = (offset + awk).min(state.data.len());
        for i in offset..azm {
            if (i - offset) == 8 { aib += aq; } 

            let b = state.data[i];
            let col = if i == state.cursor { P_ } else { hji(b) };

            
            if i == state.cursor {
                crate::framebuffer::fill_rect(
                    aib as u32, u as u32, (2 * aq + 1) as u32, ee as u32,
                    0xFF1F6FEB,
                );
            }

            let ga = format!("{:02X}", b);
            eh(aib, u, &ga, col);
            aib += 3 * aq;
        }

        
        let efq = ckp + (awk as i32 * 3 + 2) * aq;

        
        eh(efq - 2 * aq, u, "|", F_);

        let mut ax = efq;
        for i in offset..azm {
            let b = state.data[i];
            let fle = alloc::format!("{}", kgj(b));
            let col = if i == state.cursor { P_ } else { hji(b) };
            eh(ax, u, &fle, col);
            ax += aq;
        }

        u += ee;
        if u > y + h as i32 { break; }
    }

    
    if crx > yh {
        let ada = abc;
        let zo = ((yh as i32 * ada) / crx as i32).max(8);
        let ebq = (scroll as i32 * (ada - zo)) / crx.saturating_sub(1).max(1) as i32;
        let yc = (x + w as i32 - 3) as u32;
        crate::framebuffer::fill_rect(yc, (gc) as u32, 2, ada as u32, 0xFF21262D);
        crate::framebuffer::fill_rect(yc, (gc + ebq) as u32, 2, zo as u32, M_);
    }

    
    let esu = u + 2;
    if esu + ee < y + h as i32 {
        crate::framebuffer::fill_rect(x as u32, esu as u32, w, 1, 0xFF30363D);
        let mut fe = x;
        let items: &[(&str, u32)] = &[
            ("ASCII", AC_), ("WS", AU_), ("CTRL", AK_),
            ("HIGH", BG_), ("NULL", F_), ("0xFF", AN_),
        ];
        for (label, color) in items {
            
            crate::framebuffer::fill_rect(fe as u32, (esu + 3) as u32, 6, 6, *color);
            fe += aq;
            eh(fe, esu + 2, label, *color);
            fe += (label.len() as i32 + 1) * aq;
            if fe > x + w as i32 - 10 { break; }
        }
    }
}
