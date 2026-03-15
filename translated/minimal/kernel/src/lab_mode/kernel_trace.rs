











extern crate alloc;

use alloc::vec::Vec;
use alloc::string::String;
use alloc::format;
use super::{kw, nk, apm,
            T_, F_, O_, AAH_};
use super::trace_bus::{Jq, EventCategory, vsi, cus, hxa};


pub struct KernelTraceState {
    
    pub events: Vec<Jq>,
    
    pub jc: usize,
    
    pub dyp: bool,
    
    pub crz: u64,
    
    pub ckk: [bool; 9],
    
    pub lgf: bool,
    
    ehh: u64,
    
    pub ant: bool,
    
    pub fuc: Option<usize>,
}

impl KernelTraceState {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            jc: 0,
            dyp: false,
            crz: 0,
            ckk: [true; 9],
            lgf: false,
            ehh: 0,
            ant: false,
            fuc: None,
        }
    }
    
    pub fn zdk() -> Self {
        Self {
            events: Vec::new(),
            jc: 0,
            dyp: true,
            crz: 0,
            ckk: [true; 9],
            lgf: true,
            ehh: 0,
            ant: false,
            fuc: None,
        }
    }
    
    
    pub fn qs(&mut self) {
        self.ehh += 1;
        if self.ehh % 10 != 0 || self.ant {
            return;
        }
        
        
        let (jgs, gnn) = hxa(self.crz, 100);
        if !jgs.is_empty() {
            self.events.lg(jgs);
            self.crz = gnn;
            
            
            if self.events.len() > 500 {
                let bbk = self.events.len() - 500;
                self.events.bbk(..bbk);
            }
            
            
            if self.dyp {
                self.jc = 0;
            }
        }
    }
    
    pub fn vr(&mut self, bs: u8) {
        use crate::keyboard::{V_, U_, AM_, AQ_};
        match bs {
            V_ => {
                self.jc += 1;
                self.dyp = false;
            }
            U_ => {
                if self.jc > 0 {
                    self.jc -= 1;
                }
                if self.jc == 0 {
                    self.dyp = true;
                }
            }
            AM_ => {
                self.jc += 10;
                self.dyp = false;
            }
            AQ_ => {
                self.jc = self.jc.ao(10);
                if self.jc == 0 {
                    self.dyp = true;
                }
            }
            
            b'p' | b'P' => {
                self.ant = !self.ant;
            }
            
            b'c' | b'C' => {
                self.events.clear();
                self.jc = 0;
            }
            
            b'1'..=b'9' => {
                let w = (bs - b'1') as usize;
                if w < self.ckk.len() {
                    self.ckk[w] = !self.ckk[w];
                }
            }
            _ => {}
        }
    }

    
    pub fn ago(&mut self, bhi: i32, alk: i32, d: u32, dxv: u32) {
        let dt = nk();
        let kq = apm() + 1;
        if kq <= 0 || dt <= 0 { return; }

        let bfm = kq;
        let kwd = bfm;
        let okc = kwd + kq + 2;

        
        if alk >= kwd && alk < kwd + kq {
            let fek = [
                EventCategory::Fv,
                EventCategory::Scheduler,
                EventCategory::Cy,
                EventCategory::Cc,
                EventCategory::Hg,
                EventCategory::Hs,
            ];
            let mut jf = 0i32;
            for (a, rx) in fek.iter().cf() {
                let jce = rx.cu().len() as i32 + 1;
                let hpo = jf + jce * dt;
                if bhi >= jf && bhi < hpo {
                    let w = *rx as usize;
                    if w < self.ckk.len() {
                        self.ckk[w] = !self.ckk[w];
                    }
                    return;
                }
                jf = hpo;
                if jf > d as i32 - 20 { break; }
            }
            return;
        }

        
        if alk >= okc {
            let br = ((alk - okc) / kq) as usize;
            
            let aud: Vec<usize> = self.events.iter().cf()
                .hi(|(_, aa)| self.ckk[aa.gb as usize])
                .map(|(a, _)| a)
                .collect();
            let fxb = aud.len();
            let ci = fxb.ao(self.jc);
            let act = 20usize; 
            let ay = ci.ao(act);
            let ndl = ay + br;
            if ndl < ci {
                let fia = aud[ndl];
                self.fuc = if self.fuc == Some(fia) {
                    None 
                } else {
                    Some(fia)
                };
            }
            return;
        }
    }
}


pub fn po(g: &KernelTraceState, b: i32, c: i32, d: u32, i: u32) {
    let dt = nk();
    let kq = apm() + 1;
    
    if kq <= 0 || dt <= 0 { return; }
    
    
    let bfm = kq;
    let es = cus();
    let status = if g.ant {
        format!("PAUSED | {} events", es)
    } else if g.lgf {
        format!("LIVE | {} events", es)
    } else {
        format!("{} events | scroll: {}", es, g.jc)
    };
    kw(b, c, &status, if g.ant { super::AO_ } else { F_ });
    
    
    let ntx = c + bfm;
    let fek = [
        EventCategory::Fv,
        EventCategory::Scheduler,
        EventCategory::Cy,
        EventCategory::Cc,
        EventCategory::Hg,
        EventCategory::Hs,
    ];
    let mut jf = b;
    for (a, rx) in fek.iter().cf() {
        let iq = g.ckk[*rx as usize];
        let s = if iq { rx.s() } else { 0xFF30363D };
        let cu = rx.cu();
        kw(jf, ntx, cu, s);
        jf += (cu.len() as i32 + 1) * dt;
        if jf > b + d as i32 - 20 { break; }
    }
    
    
    let eew = ntx + kq + 2;
    let ljq = i as i32 - (eew - c);
    if ljq <= 0 { return; }
    
    let act = (ljq / kq) as usize;
    
    
    let aud: Vec<&Jq> = g.events.iter()
        .hi(|aa| g.ckk[aa.gb as usize])
        .collect();
    
    if aud.is_empty() {
        kw(b + 4, eew + kq, "Waiting for events...", F_);
        return;
    }
    
    
    let fxb = aud.len();
    let ci = fxb.ao(g.jc);

    
    let nkw = if g.fuc.is_some() { 4 } else { 0 };
    let uji = act.ao(nkw);
    let ay = ci.ao(uji);
    
    
    let ntz: Vec<usize> = g.events.iter().cf()
        .hi(|(_, aa)| g.ckk[aa.gb as usize])
        .map(|(a, _)| a)
        .collect();
    
    let mut ae = eew;
    for a in ay..ci {
        let id = aud[a];
        let uzc = if a < ntz.len() { ntz[a] } else { a };
        let qe = g.fuc == Some(uzc);
        
        
        if qe {
            crate::framebuffer::ah(b as u32, ae as u32, d, kq as u32, 0xFF1F2937);
        }
        
        
        let tv = id.aet / 1000;
        let jn = id.aet % 1000;
        let wi = format!("{:02}:{:02}.{:03}", tv / 60, tv % 60, jn);
        kw(b, ae, &wi, F_);
        
        
        let nbu = b + (wi.len() as i32 + 1) * dt;
        let qwn = id.gb.cu();
        kw(nbu, ae, qwn, id.gb.s());
        
        
        let lmx = nbu + (6 * dt);
        let jgh;
        if let Some(nr) = id.fvy {
            let orf = format!("#{}", nr);
            kw(lmx, ae, &orf, super::BO_);
            jgh = lmx + ((orf.len() as i32 + 1) * dt);
        } else {
            jgh = lmx;
        }
        
        
        let ulo = d as i32 - (jgh - b);
        let aem = if dt > 0 { (ulo / dt) as usize } else { 20 };
        let fr = if id.message.len() > aem && aem > 3 {
            &id.message[..aem.ao(3)]
        } else {
            &id.message
        };
        kw(jgh, ae, fr, if qe { O_ } else { T_ });
        
        ae += kq;
        if ae > c + i as i32 { break; }
    }
    
    
    if let Some(mdj) = g.fuc {
        if mdj < g.events.len() {
            let id = &g.events[mdj];
            let nkx = c + i as i32 - (nkw as i32 * kq);
            
            crate::framebuffer::ah(b as u32, (nkx - 2) as u32, d, 1, super::O_);
            
            let mut bg = nkx;
            
            let dh = format!("[{}] {}", id.gb.cu(), id.message);
            kw(b + 2, bg, &dh, id.gb.s());
            bg += kq;
            
            
            if let Some(nr) = id.fvy {
                let j = super::trace_bus::gty(nr);
                let eu = if let Some(n) = id.mjg {
                    format!("Syscall #{} ({}) args=[{:#x}, {:#x}, {:#x}]",
                        nr, j, n[0], n[1], n[2])
                } else {
                    format!("Syscall #{} ({})", nr, j)
                };
                kw(b + 2, bg, &eu, super::BO_);
                bg += kq;
                
                
                if let Some(aux) = id.mjh {
                    let vyj = if aux < 0 {
                        format!("Return: {} (error)", aux)
                    } else {
                        format!("Return: {} ({:#x})", aux, aux)
                    };
                    kw(b + 2, bg, &vyj, if aux < 0 { super::AW_ } else { super::AK_ });
                }
            } else {
                
                let vfp = format!("Payload: {} ({:#x}) | Timestamp: {}ms",
                    id.ew, id.ew, id.aet);
                kw(b + 2, bg, &vfp, F_);
            }
        }
    }
    
    
    if fxb > act {
        let ekc = eew;
        let bdc = ljq.am(1);
        let axd = ((act as i32 * bdc) / fxb as i32).am(8);
        let idk = if fxb > act {
            let pgs = fxb - act;
            let u = pgs.ao(g.jc);
            (u as i32 * (bdc - axd)) / pgs.am(1) as i32
        } else { 0 };
        
        let auz = (b + d as i32 - 3) as u32;
        
        crate::framebuffer::ah(auz, ekc as u32, 2, bdc as u32, 0xFF21262D);
        
        crate::framebuffer::ah(auz, (ekc + idk) as u32, 2, axd as u32, O_);
    }
}
