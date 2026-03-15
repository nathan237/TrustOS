




extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use super::{kw, nk, apm,
            T_, F_, O_, BB_, AK_, AO_};


#[derive(Clone)]
struct Uv {
    
    j: String,
    
    path: String,
    
    ta: bool,
    
    eo: usize,
    
    tg: bool,
    
    aw: u64,
}


pub struct FileTreeState {
    
    pub xq: Vec<Uv>,
    
    pub na: usize,
    
    pub jc: usize,
    
    pub no: bool,
}

impl FileTreeState {
    pub fn new() -> Self {
        let mut e = Self {
            xq: Vec::new(),
            na: 0,
            jc: 0,
            no: true,
        };
        e.vsz();
        e
    }
    
    
    fn vsz(&mut self) {
        self.xq.clear();
        
        self.xq.push(Uv {
            j: String::from("/"),
            path: String::from("/"),
            ta: true,
            eo: 0,
            tg: true,
            aw: 0,
        });
        self.mtu("/", 0);
        self.no = false;
    }
    
    
    fn mtu(&mut self, path: &str, eo: usize) {
        if eo > 6 { return; }
        
        
        let ch = crate::ramfs::fh(|fs| {
            fs.awb(Some(path)).age()
        });
        
        if ch.is_empty() { return; }
        
        
        let mut dgh: Vec<_> = ch.iter()
            .hi(|(_, agm, _)| *agm == crate::ramfs::FileType::K)
            .collect();
        let mut sb: Vec<_> = ch.iter()
            .hi(|(_, agm, _)| *agm != crate::ramfs::FileType::K)
            .collect();
        dgh.bxe(|q, o| q.0.cmp(&o.0));
        sb.bxe(|q, o| q.0.cmp(&o.0));
        
        
        for (j, _, _) in &dgh {
            let enk = if path == "/" {
                format!("/{}", j)
            } else {
                format!("{}/{}", path, j)
            };
            
            let how = eo < 1; 
            self.xq.push(Uv {
                j: j.clone(),
                path: enk.clone(),
                ta: true,
                eo: eo + 1,
                tg: how,
                aw: 0,
            });
            if how {
                self.mtu(&enk, eo + 1);
            }
        }
        
        
        for (j, _, aw) in &sb {
            let kwx = if path == "/" {
                format!("/{}", j)
            } else {
                format!("{}/{}", path, j)
            };
            self.xq.push(Uv {
                j: j.clone(),
                path: kwx,
                ta: false,
                eo: eo + 1,
                tg: false,
                aw: *aw as u64,
            });
        }
    }
    
    pub fn vr(&mut self, bs: u8) {
        use crate::keyboard::{V_, U_, AM_, AQ_};
        match bs {
            V_ => {
                self.na = self.na.ao(1);
            }
            U_ => {
                if self.na + 1 < self.xq.len() {
                    self.na += 1;
                }
            }
            AM_ => {
                self.na = self.na.ao(10);
            }
            AQ_ => {
                self.na = (self.na + 10).v(self.xq.len().ao(1));
            }
            
            0x0D | 0x0A => {
                self.pua();
            }
            
            b'r' | b'R' => {
                self.no = true;
            }
            _ => {}
        }
        
        if self.no {
            self.vta();
        }
    }
    
    
    pub fn ago(&mut self, b: i32, c: i32, d: u32, i: u32) {
        let kq = super::apm() + 1;
        if kq <= 0 { return; }

        
        let ou = kq + 2;
        if c < ou { return; } 

        let br = ((c - ou) / kq) as usize;
        let cd = self.jc + br;
        if cd < self.xq.len() {
            self.na = cd;
            
            if self.xq[cd].ta {
                self.pua();
            }
        }
    }

    
    fn pua(&mut self) {
        if self.na >= self.xq.len() { return; }
        if !self.xq[self.na].ta { return; }
        
        self.xq[self.na].tg = !self.xq[self.na].tg;
        self.no = true;
    }
    
    
    fn vta(&mut self) {
        
        let nrw: Vec<String> = self.xq.iter()
            .hi(|bo| bo.ta && bo.tg)
            .map(|bo| bo.path.clone())
            .collect();
        
        let zef = self.na;
        self.xq.clear();
        
        
        let pdz = nrw.iter().any(|ai| ai == "/");
        self.xq.push(Uv {
            j: String::from("/"),
            path: String::from("/"),
            ta: true,
            eo: 0,
            tg: pdz,
            aw: 0,
        });
        
        if pdz {
            self.mtv("/", 0, &nrw);
        }
        
        self.no = false;
        
        
        if self.na >= self.xq.len() && !self.xq.is_empty() {
            self.na = self.xq.len() - 1;
        }
    }
    
    fn mtv(&mut self, path: &str, eo: usize, tg: &[String]) {
        if eo > 6 { return; }
        
        let ch = crate::ramfs::fh(|fs| {
            fs.awb(Some(path)).age()
        });
        
        let mut dgh: Vec<_> = ch.iter()
            .hi(|(_, agm, _)| *agm == crate::ramfs::FileType::K)
            .collect();
        let mut sb: Vec<_> = ch.iter()
            .hi(|(_, agm, _)| *agm != crate::ramfs::FileType::K)
            .collect();
        dgh.bxe(|q, o| q.0.cmp(&o.0));
        sb.bxe(|q, o| q.0.cmp(&o.0));
        
        for (j, _, _) in &dgh {
            let enk = if path == "/" {
                format!("/{}", j)
            } else {
                format!("{}/{}", path, j)
            };
            let how = tg.iter().any(|ai| ai == &enk);
            self.xq.push(Uv {
                j: j.clone(),
                path: enk.clone(),
                ta: true,
                eo: eo + 1,
                tg: how,
                aw: 0,
            });
            if how {
                self.mtv(&enk, eo + 1, tg);
            }
        }
        
        for (j, _, aw) in &sb {
            let kwx = if path == "/" {
                format!("/{}", j)
            } else {
                format!("{}/{}", path, j)
            };
            self.xq.push(Uv {
                j: j.clone(),
                path: kwx,
                ta: false,
                eo: eo + 1,
                tg: false,
                aw: *aw as u64,
            });
        }
    }
}


pub fn po(g: &FileTreeState, b: i32, c: i32, d: u32, i: u32) {
    let dt = nk();
    let kq = apm() + 1;
    if kq <= 0 || dt <= 0 { return; }
    
    
    let dh = format!("/ ({} items)", g.xq.len());
    kw(b, c, &dh, O_);
    
    let ou = c + kq + 2;
    let bae = i as i32 - kq - 2;
    if bae <= 0 { return; }
    
    let iw = (bae / kq) as usize;
    
    if g.xq.is_empty() {
        kw(b + 4, ou, "Empty filesystem", F_);
        return;
    }
    
    
    let jc = if g.na >= g.jc + iw {
        g.na - iw + 1
    } else if g.na < g.jc {
        g.na
    } else {
        g.jc
    };
    
    let ci = (jc + iw).v(g.xq.len());
    let mut ae = ou;
    
    for a in jc..ci {
        let anq = &g.xq[a];
        let qe = a == g.na;
        
        
        if qe {
            crate::framebuffer::ah(b as u32, ae as u32, d, kq as u32, 0xFF1F2937);
        }
        
        
        let crn = anq.eo as i32 * 2 * dt;
        let vt = b + crn;
        
        
        let (pa, xd) = if anq.ta {
            if anq.tg { ("v ", AO_) } else { ("> ", AO_) }
        } else {
            let spq = ntl(&anq.j);
            ("  ", spq)
        };
        kw(vt, ae, pa, xd);
        
        
        let dac = vt + 2 * dt;
        let csp = if anq.ta {
            BB_
        } else {
            ntl(&anq.j)
        };
        
        let ulp = d as i32 - (dac - b) - 10 * dt;
        let aem = if dt > 0 { (ulp / dt) as usize } else { 20 };
        let j = if anq.j.len() > aem && aem > 3 {
            &anq.j[..aem.ao(1)]
        } else {
            &anq.j
        };
        kw(dac, ae, j, csp);
        
        
        if !anq.ta && anq.aw > 0 {
            let als = cxz(anq.aw);
            let cr = b + d as i32 - (als.len() as i32 * dt) - 4;
            if cr > dac + (j.len() as i32 * dt) + dt {
                kw(cr, ae, &als, F_);
            }
        }
        
        ae += kq;
        if ae > c + i as i32 { break; }
    }
    
    
    if g.xq.len() > iw {
        let bdc = bae;
        let axd = ((iw as i32 * bdc) / g.xq.len() as i32).am(8);
        let idk = (jc as i32 * (bdc - axd)) / g.xq.len().ao(1).am(1) as i32;
        let auz = (b + d as i32 - 3) as u32;
        crate::framebuffer::ah(auz, ou as u32, 2, bdc as u32, 0xFF21262D);
        crate::framebuffer::ah(auz, (ou + idk) as u32, 2, axd as u32, O_);
    }
}


fn ntl(j: &str) -> u32 {
    if let Some(wm) = j.cmm('.').next() {
        match wm {
            "rs" => 0xFFD18616,     
            "toml" | "cfg" | "conf" => AK_,
            "md" | "txt" => T_,
            "sh" | "ps1" | "bat" => AO_,
            "elf" | "bin" => 0xFFF85149,  
            "tl" => 0xFFBC8CFF,     
            _ => F_,
        }
    } else {
        F_
    }
}


fn cxz(bf: u64) -> String {
    if bf >= 1024 * 1024 {
        format!("{:.1}M", bf as f64 / (1024.0 * 1024.0))
    } else if bf >= 1024 {
        format!("{:.1}K", bf as f64 / 1024.0)
    } else {
        format!("{}B", bf)
    }
}
