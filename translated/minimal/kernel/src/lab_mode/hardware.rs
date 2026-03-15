









extern crate alloc;

use alloc::string::String;
use alloc::format;
use super::{kw, gfh, nk, apm,
            T_, F_, AK_, AO_, AW_, O_, AAH_};


pub struct HardwareState {
    
    pub jc: usize,
    
    pub gdn: u32,
    pub afa: usize,
    pub aul: usize,
    pub eds: u64,
    pub cnn: u64,
    pub dmj: usize,
    pub cok: u64,
    pub dpr: u64,
    pub czi: u64,
    pub ltj: usize,
    pub etu: usize,
    pub ivq: f32,
    pub mmk: u64,
    pub kmf: u64,
    
    ehh: u64,
}

impl HardwareState {
    pub fn new() -> Self {
        Self {
            jc: 0,
            gdn: 0,
            afa: 0,
            aul: 0,
            eds: 0,
            cnn: 0,
            dmj: 0,
            cok: 0,
            dpr: 0,
            czi: 0,
            ltj: 0,
            etu: 0,
            ivq: 0.0,
            mmk: 0,
            kmf: 0,
            ehh: 0,
        }
    }
    
    
    pub fn qs(&mut self) {
        self.ehh += 1;
        if self.ehh % 15 != 0 {
            return;
        }
        
        
        self.gdn = crate::devtools::rpz();
        
        
        let mem = crate::devtools::jfu();
        self.afa = mem.iqb;
        self.aul = mem.aul;
        self.cok = mem.cok;
        self.dpr = mem.dpr;
        self.czi = mem.czi;
        self.ltj = mem.gpe;
        self.etu = mem.etu;
        self.ivq = mem.hki;
        
        
        self.mmk = crate::memory::fxc() / (1024 * 1024);
        
        
        self.eds = crate::devtools::eds();
        
        
        self.cnn = crate::time::cnn();
        
        
        let xkz = crate::trace::cm();
        self.kmf = xkz.nrh;
        
        
        self.dmj = crate::scheduler::cm().exk;
    }
    
    
    pub fn nvo(&mut self) {
        self.ehh = 14;
    }
    
    pub fn vr(&mut self, bs: u8) {
        use crate::keyboard::{V_, U_};
        match bs {
            V_ => self.jc = self.jc.ao(1),
            U_ => self.jc += 1,
            _ => {}
        }
    }

    
    pub fn ago(&mut self, yap: i32, alk: i32, dxx: u32, dxv: u32) {
        let kq = apm() + 2;
        if kq <= 0 { return; }
        
        let br = (alk / kq) as usize;
        if br > 2 {
            
            self.jc = br.ao(3);
        }
    }
}


pub fn po(g: &HardwareState, b: i32, c: i32, d: u32, i: u32) {
    let dt = nk();
    let kq = apm() + 2; 
    let tn = 8u32;
    
    let aem = if dt > 0 { (d as i32 / dt) as usize } else { 30 };
    let mut ae = c;
    
    
    kw(b, ae, "CPU", O_);
    let ngn = format!("{}%", g.gdn);
    let ngm = if g.gdn > 80 { AW_ } else if g.gdn > 50 { AO_ } else { AK_ };
    let dis = b + d as i32 - (ngn.len() as i32 * dt) - 4;
    kw(dis, ae, &ngn, ngm);
    ae += kq;
    gfh(b, ae, d.ao(4), tn, g.gdn, ngm, 0xFF21262D);
    ae += tn as i32 + kq / 2;
    
    
    kw(b, ae, "Heap", O_);
    let mol = g.afa / (1024 * 1024);
    let jtt = g.aul / (1024 * 1024);
    let bne = if g.aul > 0 {
        ((g.afa as u64 * 100) / g.aul as u64) as u32
    } else { 0 };
    let obk = format!("{}/{}MB ({}%)", mol, jtt, bne);
    let tpf = b + d as i32 - (obk.len() as i32 * dt) - 4;
    let obj = if bne > 85 { AW_ } else if bne > 60 { AO_ } else { AK_ };
    kw(tpf, ae, &obk, obj);
    ae += kq;
    gfh(b, ae, d.ao(4), tn, bne, obj, 0xFF21262D);
    ae += tn as i32 + kq / 2;
    
    
    let cm: [(&str, String, u32); 10] = [
        ("Uptime", nvt(g.cnn), T_),
        ("Physical RAM", format!("{} MB", g.mmk), T_),
        ("IRQ Rate", format!("{}/sec", g.eds), AO_),
        ("Tasks", format!("{}", g.dmj), T_),
        ("Trace Events", format!("{}", g.kmf), F_),
        ("Live Allocs", format!("{}", g.czi), AK_),
        ("Total Allocs", format!("{}", g.cok), F_),
        ("Peak Heap", format!("{} KB", g.ltj / 1024), T_),
        ("Largest Alloc", svs(g.etu), T_),
        ("Fragmentation", format!("{:.1}%", g.ivq), 
            if g.ivq > 50.0 { AW_ } else { F_ }),
    ];
    
    let iw = ((i as i32 - (ae - c)) / kq) as usize;
    let ay = g.jc.v(cm.len().ao(1));
    let ci = (ay + iw).v(cm.len());
    
    for a in ay..ci {
        let (cu, ref bn, s) = cm[a];
        kw(b + 4, ae, cu, F_);
        let fp = b + d as i32 - (bn.len() as i32 * dt) - 4;
        kw(fp, ae, bn, s);
        ae += kq;
    }
}

fn nvt(tv: u64) -> String {
    let i = tv / 3600;
    let ef = (tv % 3600) / 60;
    let e = tv % 60;
    if i > 0 {
        format!("{}h {:02}m {:02}s", i, ef, e)
    } else if ef > 0 {
        format!("{}m {:02}s", ef, e)
    } else {
        format!("{}s", e)
    }
}

fn svs(bf: usize) -> String {
    if bf >= 1024 * 1024 {
        format!("{} MB", bf / (1024 * 1024))
    } else if bf >= 1024 {
        format!("{} KB", bf / 1024)
    } else {
        format!("{} B", bf)
    }
}
