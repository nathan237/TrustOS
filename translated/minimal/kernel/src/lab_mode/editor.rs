








extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use super::{kw, nk, apm,
            T_, F_, O_, AK_, AO_, AW_, BO_, BB_};


pub struct EditorState {
    
    pub ak: Vec<String>,
    
    pub gn: usize,
    
    pub hn: usize,
    
    pub jc: usize,
    
    pub an: Vec<String>,
    
    pub bhp: bool,
    
    pub evv: usize,
    
    pub frame: u64,
}

impl EditorState {
    pub fn new() -> Self {
        Self {
            ak: alloc::vec![
                String::from("// TrustLang — write code here"),
                String::from("fn main() {"),
                String::from("    let x = 42;"),
                String::from("    print(\"Hello from TrustLab!\");"),
                String::from("    print(x * 2);"),
                String::from("}"),
            ],
            gn: 1,
            hn: 0,
            jc: 0,
            an: alloc::vec![String::from("Press Ctrl+R to run")],
            bhp: false,
            evv: 0,
            frame: 0,
        }
    }
    
    pub fn vr(&mut self, bs: u8) {
        use crate::keyboard::{V_, U_, AH_, AI_, AM_, AQ_};
        
        self.frame += 1;
        
        match bs {
            V_ => {
                if self.bhp {
                    self.evv = self.evv.ao(1);
                } else if self.gn > 0 {
                    self.gn -= 1;
                    self.inu();
                }
            }
            U_ => {
                if self.bhp {
                    self.evv += 1;
                } else if self.gn + 1 < self.ak.len() {
                    self.gn += 1;
                    self.inu();
                }
            }
            AH_ => {
                if !self.bhp && self.hn > 0 {
                    self.hn -= 1;
                }
            }
            AI_ => {
                if !self.bhp {
                    let ark = self.ak.get(self.gn)
                        .map(|dm| dm.len()).unwrap_or(0);
                    if self.hn < ark {
                        self.hn += 1;
                    }
                }
            }
            AM_ => {
                if self.bhp {
                    self.evv = self.evv.ao(5);
                } else {
                    self.gn = self.gn.ao(10);
                    self.inu();
                }
            }
            AQ_ => {
                if self.bhp {
                    self.evv += 5;
                } else {
                    self.gn = (self.gn + 10).v(self.ak.len().ao(1));
                    self.inu();
                }
            }
            
            0x12 => {
                self.pep();
            }
            
            0x13 => {
                self.ftm();
            }
            
            0x0D | 0x0A => {
                if !self.bhp {
                    
                    if self.gn < self.ak.len() {
                        let bj = self.hn.v(self.ak[self.gn].len());
                        let kr = self.ak[self.gn].pmk(bj);
                        self.gn += 1;
                        self.ak.insert(self.gn, kr);
                        self.hn = 0;
                    } else {
                        self.ak.push(String::new());
                        self.gn = self.ak.len() - 1;
                        self.hn = 0;
                    }
                }
            }
            
            0x08 => {
                if !self.bhp {
                    if self.hn > 0 {
                        self.hn -= 1;
                        if self.gn < self.ak.len() {
                            self.ak[self.gn].remove(self.hn);
                        }
                    } else if self.gn > 0 {
                        
                        let cv = self.ak.remove(self.gn);
                        self.gn -= 1;
                        self.hn = self.ak[self.gn].len();
                        self.ak[self.gn].t(&cv);
                    }
                }
            }
            
            0x0F => {
                self.bhp = !self.bhp;
            }
            _ => {}
        }
    }
    
    pub fn fka(&mut self, bm: char) {
        if self.bhp { return; }
        
        self.frame += 1;
        
        if self.gn >= self.ak.len() {
            self.ak.push(String::new());
            self.gn = self.ak.len() - 1;
        }
        
        self.ak[self.gn].insert(self.hn, bm);
        self.hn += 1;
    }
    
    
    pub fn ago(&mut self, b: i32, c: i32, d: u32, i: u32) {
        let dt = super::nk();
        let kq = super::apm() + 1;
        if kq <= 0 || dt <= 0 { return; }

        
        let hhs = (i as i32 * 60 / 100).am(kq * 3);
        let hua = hhs + 2;

        if c >= hua {
            
            self.bhp = true;
        } else {
            
            self.bhp = false;

            
            let bdt = kq;
            if c < bdt { return; } 

            let bqy = 4 * dt;
            let br = ((c - bdt) / kq) as usize;
            let bj = ((b - bqy).am(0) / dt) as usize;

            self.gn = (self.jc + br).v(self.ak.len().ao(1));
            let ark = self.ak.get(self.gn).map(|dm| dm.len()).unwrap_or(0);
            self.hn = bj.v(ark);
            self.frame += 1; 
        }
    }

    fn inu(&mut self) {
        let ark = self.ak.get(self.gn)
            .map(|dm| dm.len()).unwrap_or(0);
        if self.hn > ark {
            self.hn = ark;
        }
    }
    
    
    pub fn ftm(&mut self) {
        let iy: String = self.ak.iter()
            .map(|dm| dm.as_str())
            .collect::<Vec<_>>()
            .rr("\n");
        
        match crate::vfs::ns("/mnt/trustfs/editor.tl", iy.as_bytes()) {
            Ok(()) => {
                
                let _ = crate::vfs::wxb();
                self.an.clear();
                self.an.push(String::from("=== Saved ==="));
                self.an.push(format!("Wrote {} bytes to /mnt/trustfs/editor.tl", iy.len()));
                self.an.push(String::from("File persisted to disk (WAL protected)"));
            }
            Err(aa) => {
                self.an.clear();
                self.an.push(format!("Save error: {:?}", aa));
                self.an.push(String::from("Try: file saved to ramfs as fallback"));
                
                let _ = crate::ramfs::fh(|fs| {
                    if !fs.aja("editor.tl") { let _ = fs.touch("editor.tl"); }
                    fs.ns("editor.tl", iy.as_bytes())
                });
            }
        }
    }

    
    pub fn pep(&mut self) {
        self.an.clear();
        self.an.push(String::from("=== Running ==="));
        
        
        let iy: String = self.ak.iter()
            .map(|dm| dm.as_str())
            .collect::<Vec<_>>()
            .rr("\n");
        
        
        self.an.push(format!("Source: {} lines, {} bytes", self.ak.len(), iy.len()));
        
        
        match crate::trustlang::vw(&iy) {
            Ok(result) => {
                self.an.push(format!("=> {}", result));
            }
            Err(aa) => {
                self.an.push(format!("Error: {}", aa));
            }
        }
        
        self.an.push(String::from("=== Done ==="));
        self.bhp = true;
    }
}


pub fn po(g: &EditorState, b: i32, c: i32, d: u32, i: u32) {
    let dt = nk();
    let kq = apm() + 1;
    if kq <= 0 || dt <= 0 { return; }
    
    
    let hhs = (i as i32 * 60 / 100).am(kq * 3);
    let hua = c + hhs + 2;
    let oti = i as i32 - hhs - 4;
    
    
    
    let dh = if g.bhp { "Editor (Ctrl+O)" } else { "Editor [active]" };
    let tnw = if !g.bhp { O_ } else { F_ };
    kw(b, c, dh, tnw);
    
    
    let hint = "[Ctrl+S] save [Ctrl+R] run";
    let hmy = b + d as i32 - (hint.len() as i32 * dt) - 2;
    kw(hmy, c, hint, AK_);
    
    let bdt = c + kq;
    let byr = hhs - kq;
    let mpk = (byr / kq) as usize;
    
    
    let jc = if g.gn >= g.jc + mpk {
        g.gn - mpk + 1
    } else if g.gn < g.jc {
        g.gn
    } else {
        g.jc
    };
    
    let bqy = 4 * dt; 
    let bds = b + bqy;
    
    let ci = (jc + mpk).v(g.ak.len());
    let mut ae = bdt;
    
    for a in jc..ci {
        
        let csd = format!("{:>3}", a + 1);
        kw(b, ae, &csd, F_);
        
        
        if a == g.gn && !g.bhp {
            crate::framebuffer::ah(
                bds as u32, ae as u32,
                d.ao(bqy as u32), kq as u32,
                0xFF1C2128,
            );
        }
        
        
        irv(bds, ae, &g.ak[a], d.ao(bqy as u32));
        
        
        if a == g.gn && !g.bhp {
            if (g.frame / 25) % 2 == 0 {
                let lf = bds + (g.hn as i32 * dt);
                crate::framebuffer::ah(
                    lf as u32, ae as u32,
                    2, kq as u32,
                    O_,
                );
            }
        }
        
        ae += kq;
    }
    
    
    crate::framebuffer::ah(b as u32, (hua - 1) as u32, d, 1, 0xFF30363D);
    
    
    if oti <= 0 { return; }
    
    let uzt = if g.bhp { "Output [active]" } else { "Output (Ctrl+O)" };
    let uzq = if g.bhp { AK_ } else { F_ };
    kw(b, hua, uzt, uzq);
    
    let uzv = hua + kq;
    let xsa = ((oti - kq) / kq) as usize;
    let otf = g.evv.v(g.an.len().ao(1));
    let uzs = (otf + xsa).v(g.an.len());
    
    let mut qw = uzv;
    for a in otf..uzs {
        let line = &g.an[a];
        let s = if line.cj("Error:") {
            AW_
        } else if line.cj("=>") {
            AK_
        } else if line.cj("===") {
            AO_
        } else {
            T_
        };
        
        kw(b + 4, qw, line, s);
        qw += kq;
    }
}


fn irv(b: i32, c: i32, line: &str, yaw: u32) {
    let dt = nk();
    let fmj = ["fn", "let", "mut", "if", "else", "for", "while", "return", 
                     "true", "false", "struct", "enum", "match", "pub", "use",
                     "const", "static", "impl", "self", "loop", "break", "continue"];
    
    let mut cx = b;
    let bw: Vec<char> = line.bw().collect();
    let len = bw.len();
    let mut a = 0;
    
    while a < len {
        let bm = bw[a];
        
        
        if bm == '/' && a + 1 < len && bw[a + 1] == '/' {
            
            let kr: String = bw[a..].iter().collect();
            kw(cx, c, &kr, F_);
            return;
        }
        
        
        if bm == '"' {
            let ay = a;
            a += 1;
            while a < len && bw[a] != '"' {
                if bw[a] == '\\' { a += 1; } 
                a += 1;
            }
            if a < len { a += 1; } 
            let e: String = bw[ay..a].iter().collect();
            kw(cx, c, &e, AK_);
            cx += e.len() as i32 * dt;
            continue;
        }
        
        
        if bm.atb() {
            let ay = a;
            while a < len && (bw[a].atb() || bw[a] == '.' || bw[a] == 'x') {
                a += 1;
            }
            let num: String = bw[ay..a].iter().collect();
            kw(cx, c, &num, BB_);
            cx += num.len() as i32 * dt;
            continue;
        }
        
        
        if bm.bvb() || bm == '_' {
            let ay = a;
            while a < len && (bw[a].bvb() || bw[a] == '_') {
                a += 1;
            }
            let od: String = bw[ay..a].iter().collect();
            let s = if fmj.contains(&od.as_str()) {
                BO_
            } else if od.bw().next().map(|r| r.crs()).unwrap_or(false) {
                AO_ 
            } else {
                T_
            };
            kw(cx, c, &od, s);
            cx += od.len() as i32 * dt;
            continue;
        }
        
        
        let s = match bm {
            '(' | ')' | '{' | '}' | '[' | ']' => AO_,
            '=' | '+' | '-' | '*' | '/' | '<' | '>' | '!' | '&' | '|' => O_,
            ';' | ':' | ',' | '.' => F_,
            _ => T_,
        };
        let e = alloc::format!("{}", bm);
        kw(cx, c, &e, s);
        cx += dt;
        a += 1;
    }
}
