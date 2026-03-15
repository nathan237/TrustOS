










extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use super::{kw, gfh, nk, apm,
            T_, F_, O_, AK_, AO_, AW_,
            BO_, BB_, EZ_};
use super::trace_bus::{self, EventCategory, hxa};


#[derive(Clone, Copy, PartialEq, Eq)]
enum Stage {
    Jp,
    Parser,
    Scheduler,
    Cy,
    Cc,
    Lq,
    Dd,
}

impl Stage {
    fn cu(&self) -> &'static str {
        match self {
            Stage::Jp       => "USER INPUT",
            Stage::Parser      => "SHELL / PARSER",
            Stage::Scheduler   => "SCHEDULER",
            Stage::Cy      => "MEMORY MGR",
            Stage::Cc  => "FILE SYSTEM",
            Stage::Lq  => "IRQ / HW",
            Stage::Dd      => "DISPLAY OUT",
        }
    }
    
    fn s(&self) -> u32 {
        match self {
            Stage::Jp       => O_,
            Stage::Parser      => BO_,
            Stage::Scheduler   => AO_,
            Stage::Cy      => AK_,
            Stage::Cc  => BB_,
            Stage::Lq  => EZ_,
            Stage::Dd      => 0xFF3FB950,
        }
    }
    
    fn pa(&self) -> &'static str {
        match self {
            Stage::Jp       => ">>",
            Stage::Parser      => "{}", 
            Stage::Scheduler   => "<>",
            Stage::Cy      => "[]",
            Stage::Cc  => "//",
            Stage::Lq  => "!!",
            Stage::Dd      => "<-",
        }
    }
}

const RA_: [Stage; 7] = [
    Stage::Jp, Stage::Parser, Stage::Scheduler,
    Stage::Cy, Stage::Cc, Stage::Lq, Stage::Dd,
];


struct StageActivity {
    
    xc: u16,
    
    buw: u64,
    
    hps: String,
}

impl StageActivity {
    fn new() -> Self {
        Self { xc: 0, buw: 0, hps: String::new() }
    }
}


pub struct PipelineState {
    
    blh: [StageActivity; 7],
    
    pub cqq: Vec<Bhi>,
    
    lkx: usize,
    
    crz: u64,
    
    frame: u64,
    
    pub jc: usize,
}


pub struct Bhi {
    aet: u64,
    nwi: usize,
    pty: usize,
    cu: String,
}

impl PipelineState {
    pub fn new() -> Self {
        Self {
            blh: [
                StageActivity::new(), StageActivity::new(),
                StageActivity::new(), StageActivity::new(),
                StageActivity::new(), StageActivity::new(),
                StageActivity::new(),
            ],
            cqq: Vec::new(),
            lkx: 50,
            crz: 0,
            frame: 0,
            jc: 0,
        }
    }
    
    
    pub fn qs(&mut self) {
        self.frame += 1;
        
        
        if self.frame % 3 == 0 {
            for e in &mut self.blh {
                e.xc = e.xc.ao(3);
            }
        }
        
        
        if self.frame % 5 != 0 { return; }
        
        let (events, gnn) = hxa(self.crz, 50);
        if events.is_empty() {
            self.crz = gnn;
            return;
        }
        
        for aiz in &events {
            
            let (from, wh) = match aiz.gb {
                EventCategory::Hs => (0, 1),   
                EventCategory::Hg  => (1, 2),   
                EventCategory::Scheduler => (2, 3),  
                EventCategory::Cy   => (3, 6),   
                EventCategory::Cc => (1, 4), 
                EventCategory::Fv => (5, 2),  
                EventCategory::As  => (4, 6),   
                EventCategory::De => (1, 5),   
                EventCategory::Gv   => (0, 6),   
                EventCategory::Ee => (5, 6), 
            };
            
            
            self.blh[from].xc = 255;
            self.blh[from].buw += 1;
            self.blh[wh].xc = 200;
            self.blh[wh].buw += 1;
            
            
            if aiz.message.len() < 40 {
                self.blh[wh].hps = aiz.message.clone();
            } else {
                self.blh[wh].hps = String::from(&aiz.message[..37]);
                self.blh[wh].hps.t("...");
            }
            
            
            self.cqq.push(Bhi {
                aet: aiz.aet,
                nwi: from,
                pty: wh,
                cu: if aiz.message.len() > 25 {
                    let mut e = String::from(&aiz.message[..22]);
                    e.t("...");
                    e
                } else {
                    aiz.message.clone()
                },
            });
        }
        
        
        if self.cqq.len() > self.lkx {
            let bbk = self.cqq.len() - self.lkx;
            self.cqq.bbk(..bbk);
        }
        
        self.crz = gnn;
    }
    
    pub fn vr(&mut self, bs: u8) {
        use crate::keyboard::{V_, U_, AM_, AQ_};
        match bs {
            V_ => self.jc += 1,
            U_ => { if self.jc > 0 { self.jc -= 1; } }
            AM_ => self.jc += 5,
            AQ_ => self.jc = self.jc.ao(5),
            b'c' | b'C' => {
                self.cqq.clear();
                self.jc = 0;
                for e in &mut self.blh {
                    e.buw = 0;
                    e.xc = 0;
                    e.hps.clear();
                }
            }
            _ => {}
        }
    }

    
    pub fn ago(&mut self, bhi: i32, alk: i32, d: u32, dxv: u32) {
        let dt = nk();
        let kq = apm() + 1;
        if kq <= 0 || dt <= 0 { return; }

        
        let rxg = 3;
        let pnu = kq * rxg + 4;
        if alk < pnu {
            
            let bas = (d as i32 / 4).am(12 * dt);
            let qi = 2i32;
            
            if alk < kq + qi {
                let bj = bhi / (bas + dt);
                if bj < 3 {
                    let w = bj as usize;
                    self.blh[w].xc = 255; 
                }
            }
            
            else if alk < 2 * (kq + qi) {
                let bj = bhi / (bas + dt);
                if bj < 3 {
                    let w = 3 + bj as usize;
                    if w < 6 { self.blh[w].xc = 255; }
                }
            }
            return;
        }

        
        let dwf = pnu + 3 + kq + 2;
        if alk >= dwf {
            let br = ((alk - dwf) / kq) as usize;
            
            if br > 5 {
                self.jc = self.jc.ao(br - 5);
            }
        }
    }
}


pub fn po(g: &PipelineState, b: i32, c: i32, d: u32, i: u32) {
    let dt = nk();
    let kq = apm() + 1;
    if kq <= 0 || dt <= 0 { return; }
    
    
    
    let yma = kq * 4; 
    let bas = (d as i32 / 4).am(12 * dt); 
    let qi = 2i32;
    
    
    let hya = c;
    fha(g, 0, b, hya, bas as u32, kq, dt);
    irp(b + bas - dt, hya + kq / 2, dt, F_);
    fha(g, 1, b + bas + dt, hya, bas as u32, kq, dt);
    irp(b + 2 * bas, hya + kq / 2, dt, F_);
    fha(g, 2, b + 2 * (bas + dt), hya, bas as u32, kq, dt);
    
    
    let hyb = c + kq + qi;
    fha(g, 3, b, hyb, bas as u32, kq, dt);
    irp(b + bas - dt, hyb + kq / 2, dt, F_);
    fha(g, 4, b + bas + dt, hyb, bas as u32, kq, dt);
    irp(b + 2 * bas, hyb + kq / 2, dt, F_);
    fha(g, 5, b + 2 * (bas + dt), hyb, bas as u32, kq, dt);
    
    
    let pee = c + 2 * (kq + qi);
    let uzy = b + bas + dt;
    fha(g, 6, uzy, pee, bas as u32, kq, dt);
    
    
    let dwf = pee + kq + qi + 2;
    crate::framebuffer::ah(b as u32, dwf as u32, d, 1, 0xFF30363D);
    let poj = dwf + 3;
    
    let mut cr = b;
    for (a, eyz) in RA_.iter().cf() {
        let buw = g.blh[a].buw;
        let cu = format!("{}:{}", eyz.pa(), buw);
        let bj = if g.blh[a].xc > 50 { eyz.s() } else { F_ };
        kw(cr, poj, &cu, bj);
        cr += (cu.len() as i32 + 1) * dt;
        if cr > b + d as i32 - 10 { break; }
    }
    
    
    let eew = poj + kq + 2;
    crate::framebuffer::ah(b as u32, (eew - 1) as u32, d, 1, 0xFF30363D);
    
    kw(b, eew, "Pipeline Flow", O_);
    let ngh = format!("{} events", g.cqq.len());
    let rbz = b + d as i32 - (ngh.len() as i32 * dt) - 2;
    kw(rbz, eew, &ngh, F_);
    
    let ou = eew + kq;
    let bae = i as i32 - (ou - c);
    if bae <= 0 { return; }
    
    let iw = (bae / kq) as usize;
    
    if g.cqq.is_empty() {
        kw(b + 4, ou, "Waiting for events...", F_);
        return;
    }
    
    
    let es = g.cqq.len();
    let ci = es.ao(g.jc);
    let ay = ci.ao(iw);
    
    let mut ae = ou;
    for a in ay..ci {
        let eqm = &g.cqq[a];
        let from = &RA_[eqm.nwi];
        let wh = &RA_[eqm.pty];
        
        
        let wi = swc(eqm.aet);
        kw(b, ae, &wi, F_);
        
        
        let sxz = from.pa();
        let xil = wh.pa();
        let gx = b + 10 * dt;
        kw(gx, ae, sxz, from.s());
        kw(gx + 3 * dt, ae, ">", F_);
        kw(gx + 4 * dt, ae, xil, wh.s());
        
        
        let hfw = gx + 8 * dt;
        let fnu = ((d as i32 - (hfw - b)) / dt) as usize;
        let desc = if eqm.cu.len() > fnu && fnu > 3 {
            &eqm.cu[..fnu.ao(1)]
        } else {
            &eqm.cu
        };
        kw(hfw, ae, desc, T_);
        
        ae += kq;
        if ae > c + i as i32 { break; }
    }
}


fn fha(g: &PipelineState, w: usize, b: i32, c: i32, d: u32, i: i32, dt: i32) {
    let eyz = &RA_[w];
    let fzs = &g.blh[w];
    
    
    let ei = if fzs.xc > 150 {
        gbe(0xFF161B22, eyz.s(), fzs.xc as u32 / 4)
    } else if fzs.xc > 50 {
        gbe(0xFF161B22, eyz.s(), fzs.xc as u32 / 8)
    } else {
        0xFF161B22
    };
    crate::framebuffer::ah(b as u32, c as u32, d, i as u32, ei);
    
    
    let acu = if fzs.xc > 100 { eyz.s() } else { 0xFF30363D };
    
    crate::framebuffer::ah(b as u32, c as u32, d, 1, acu);
    crate::framebuffer::ah(b as u32, (c + i - 1) as u32, d, 1, acu);
    
    crate::framebuffer::ah(b as u32, c as u32, 1, i as u32, acu);
    crate::framebuffer::ah((b + d as i32 - 1) as u32, c as u32, 1, i as u32, acu);
    
    
    let cu = eyz.cu();
    let agx = if fzs.xc > 100 { eyz.s() } else { F_ };
    
    let dis = b + 2;
    kw(dis, c + 2, cu, agx);
}


fn irp(b: i32, c: i32, jxv: i32, s: u32) {
    kw(b, c, ">", s);
}


fn gbe(ar: u32, mm: u32, bdk: u32) -> u32 {
    let bdk = bdk.v(63);
    let wq = 63 - bdk;
    let m = (((ar >> 16) & 0xFF) * wq + ((mm >> 16) & 0xFF) * bdk) / 63;
    let at = (((ar >> 8) & 0xFF) * wq + ((mm >> 8) & 0xFF) * bdk) / 63;
    let o = ((ar & 0xFF) * wq + (mm & 0xFF) * bdk) / 63;
    0xFF000000 | (m << 16) | (at << 8) | o
}


fn swc(jn: u64) -> String {
    let e = jn / 1000;
    let ef = e / 60;
    let avw = jn % 1000;
    format!("{:02}:{:02}.{:03}", ef % 100, e % 60, avw)
}
