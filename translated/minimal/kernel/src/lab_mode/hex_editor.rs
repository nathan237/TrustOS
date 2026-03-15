














extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use super::{kw, nk, apm,
            T_, F_, O_, AK_, AO_, AW_,
            BO_, BB_, EZ_};


fn nax(o: u8) -> u32 {
    match o {
        0x00 => F_,                   
        0x09 | 0x0A | 0x0D | 0x20 => BB_,  
        0x01..=0x1F => AO_,         
        0x20..=0x7E => AK_,          
        0x7F => AO_,                
        0xFF => AW_,                   
        0x80..=0xFE => BO_,         
    }
}


fn quu(o: u8) -> char {
    if o >= 0x20 && o <= 0x7E { o as char } else { '.' }
}


pub struct HexEditorState {
    
    pub f: Vec<u8>,
    
    pub wn: String,
    
    pub jc: usize,
    
    pub gi: usize,
    
    pub hca: usize,
    
    pub frame: u64,
}

impl HexEditorState {
    pub fn new() -> Self {
        
        let mut e = Self {
            f: Vec::new(),
            wn: String::new(),
            jc: 0,
            gi: 0,
            hca: 16,
            frame: 0,
        };
        
        e.dsu("/home/welcome.txt");
        if e.f.is_empty() {
            e.uhg();
        }
        e
    }

    
    pub fn dsu(&mut self, path: &str) {
        let f = crate::ramfs::fh(|fs| {
            fs.mq(path).bq().map(|bf| bf.ip())
        });
        if let Some(bf) = f {
            self.f = bf;
            self.wn = String::from(path);
            self.gi = 0;
            self.jc = 0;
        }
    }

    
    fn uhg(&mut self) {
        self.wn = String::from("<sample>");
        self.f.clear();
        
        self.f.bk(&[0x7F, b'E', b'L', b'F', 0x02, 0x01, 0x01, 0x00]);
        self.f.bk(&[0x00; 8]); 
        
        for a in 0u8..=255 {
            self.f.push(a);
        }
        
        self.f.bk(b"TrustOS Kernel v0.1\x00");
        self.f.bk(b"Built with Rust\x00\xFF\xFF");
        self.gi = 0;
        self.jc = 0;
    }

    pub fn vr(&mut self, bs: u8) {
        use crate::keyboard::{V_, U_, AH_, AI_, AM_, AQ_};
        let cow = self.hca;
        match bs {
            V_ => {
                self.gi = self.gi.ao(cow);
            }
            U_ => {
                if self.gi + cow < self.f.len() {
                    self.gi += cow;
                }
            }
            AH_ => {
                self.gi = self.gi.ao(1);
            }
            AI_ => {
                if self.gi + 1 < self.f.len() {
                    self.gi += 1;
                }
            }
            AM_ => {
                self.gi = self.gi.ao(cow * 8);
            }
            AQ_ => {
                self.gi = (self.gi + cow * 8).v(self.f.len().ao(1));
            }
            _ => {}
        }
    }

    
    pub fn ago(&mut self, b: i32, c: i32, d: u32, i: u32) {
        let dt = nk();
        let kq = apm() + 1;
        if kq <= 0 || dt <= 0 { return; }

        
        let ou = kq + 2;
        if c < ou { return; }

        let br = ((c - ou) / kq) as usize;
        let xax = self.jc + br;

        
        let obt = 10 * dt;
        
        if b >= obt {
            let adj = (b - obt) as usize;
            let hcr = adj / dt as usize;
            
            let quw = if hcr >= 25 {
                (hcr - 1) / 3
            } else {
                hcr / 3
            };
            let avk = xax * self.hca + quw.v(self.hca - 1);
            if avk < self.f.len() {
                self.gi = avk;
            }
        }
    }
}


pub fn po(g: &HexEditorState, b: i32, c: i32, d: u32, i: u32) {
    let dt = nk();
    let kq = apm() + 1;
    if kq <= 0 || dt <= 0 { return; }

    
    let dh = if g.wn.is_empty() {
        String::from("No file loaded — use: hex <path>")
    } else {
        format!("{} ({} bytes)", g.wn, g.f.len())
    };
    kw(b, c, &dh, O_);

    
    let osc = format!("@{:04X}", g.gi);
    let lpw = b + d as i32 - (osc.len() as i32 * dt) - 2;
    kw(lpw, c, &osc, F_);

    let ou = c + kq + 2;
    let bae = i as i32 - kq - 2;
    if bae <= 0 { return; }

    let bpd = (bae / kq) as usize;
    if g.f.is_empty() {
        kw(b + 4, ou, "Empty — type 'hex <path>' to load a file", F_);
        return;
    }

    let cow = g.hca;
    let fxd = (g.f.len() + cow - 1) / cow;

    
    let qu = g.gi / cow;
    let jc = if qu >= g.jc + bpd {
        qu - bpd + 1
    } else if qu < g.jc {
        qu
    } else {
        g.jc
    };

    let ktn = (jc + bpd).v(fxd);
    let mut ae = ou;

    
    let mut zj = String::from("Offset    ");
    for a in 0..cow.v(16) {
        if a == 8 { zj.push(' '); }
        zj.t(&format!("{:02X} ", a));
    }
    zj.t(" ASCII");
    kw(b, ae, &zj, F_);
    ae += kq;

    for br in jc..ktn {
        let l = br * cow;
        
        let uxf = format!("{:08X}  ", l);
        kw(b, ae, &uxf, F_);

        let fkp = b + 10 * dt;
        let mut bng = fkp;

        
        let cub = (l + cow).v(g.f.len());
        for a in l..cub {
            if (a - l) == 8 { bng += dt; } 

            let o = g.f[a];
            let bj = if a == g.gi { T_ } else { nax(o) };

            
            if a == g.gi {
                crate::framebuffer::ah(
                    bng as u32, ae as u32, (2 * dt + 1) as u32, kq as u32,
                    0xFF1F6FEB,
                );
            }

            let nu = format!("{:02X}", o);
            kw(bng, ae, &nu, bj);
            bng += 3 * dt;
        }

        
        let ikb = fkp + (cow as i32 * 3 + 2) * dt;

        
        kw(ikb - 2 * dt, ae, "|", F_);

        let mut ax = ikb;
        for a in l..cub {
            let o = g.f[a];
            let khb = alloc::format!("{}", quu(o));
            let bj = if a == g.gi { T_ } else { nax(o) };
            kw(ax, ae, &khb, bj);
            ax += dt;
        }

        ae += kq;
        if ae > c + i as i32 { break; }
    }

    
    if fxd > bpd {
        let bdc = bae;
        let axd = ((bpd as i32 * bdc) / fxd as i32).am(8);
        let idk = (jc as i32 * (bdc - axd)) / fxd.ao(1).am(1) as i32;
        let auz = (b + d as i32 - 3) as u32;
        crate::framebuffer::ah(auz, (ou) as u32, 2, bdc as u32, 0xFF21262D);
        crate::framebuffer::ah(auz, (ou + idk) as u32, 2, axd as u32, O_);
    }

    
    let jdg = ae + 2;
    if jdg + kq < c + i as i32 {
        crate::framebuffer::ah(b as u32, jdg as u32, d, 1, 0xFF30363D);
        let mut mj = b;
        let pj: &[(&str, u32)] = &[
            ("ASCII", AK_), ("WS", BB_), ("CTRL", AO_),
            ("HIGH", BO_), ("NULL", F_), ("0xFF", AW_),
        ];
        for (cu, s) in pj {
            
            crate::framebuffer::ah(mj as u32, (jdg + 3) as u32, 6, 6, *s);
            mj += dt;
            kw(mj, jdg + 2, cu, *s);
            mj += (cu.len() as i32 + 1) * dt;
            if mj > b + d as i32 - 10 { break; }
        }
    }
}
