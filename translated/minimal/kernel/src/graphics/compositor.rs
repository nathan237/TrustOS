








use alloc::vec::Vec;
use alloc::string::String;
use spin::Mutex;
use micromath::Wo;

use super::math3d::{Vec3, Vec4, Mat4};
use super::render2d::Color2D;
use super::opengl::*;
use crate::framebuffer;






#[derive(Clone, Copy, PartialEq)]
pub enum CompositorTheme {
    
    Aif,
    
    Xq,
    
    Ait,
    
    Tp,
    
    Gy,
}


#[derive(Clone, Copy)]
pub enum Easing {
    Cgp,
    Bfq,
    Arl,
    Cbn,
    Byr,
    Cbo,
}


#[derive(Clone)]
pub struct WindowSurface {
    pub ad: u32,
    pub b: f32,
    pub c: f32,
    pub z: f32,
    pub ac: f32,
    pub adh: f32,
    pub bv: f32,
    pub chh: f32,
    pub ell: i32,
    pub iw: bool,
    pub ja: bool,
    pub aat: bool,
    
    
    pub ca: Vec<u32>,
    pub rok: u32,
    pub eny: u32,
    
    
    pub ayw: f32,
    pub ayx: f32,
    pub jsm: f32,
    pub jso: f32,
    pub dyh: f32,
    pub gys: f32,
    pub gyt: Easing,
}

impl WindowSurface {
    pub fn new(ad: u32, b: f32, c: f32, z: f32, ac: f32) -> Self {
        let d = z as u32;
        let i = ac as u32;
        Self {
            ad,
            b,
            c,
            z,
            ac,
            adh: 1.0,
            bv: 1.0,
            chh: 0.0,
            ell: 0,
            iw: true,
            ja: false,
            aat: false,
            ca: alloc::vec![0xFF0A0E0B; (d * i) as usize],
            rok: d,
            eny: i,
            ayw: b,
            ayx: c,
            jsm: 1.0,
            jso: 1.0,
            dyh: 1.0,
            gys: 0.0,
            gyt: Easing::Arl,
        }
    }
    
    
    pub fn qs(&mut self, os: f32) {
        if self.dyh < 1.0 {
            self.dyh += os / self.gys.am(0.001);
            self.dyh = self.dyh.v(1.0);
            
            let ab = qjs(self.dyh, self.gyt);
            
            self.b = csb(self.b, self.ayw, ab);
            self.c = csb(self.c, self.ayx, ab);
            self.adh = csb(self.adh, self.jsm, ab);
            self.bv = csb(self.bv, self.jso, ab);
        }
    }
    
    
    pub fn yex(&mut self, b: f32, c: f32, avr: f32, ksn: Easing) {
        self.ayw = b;
        self.ayx = c;
        self.dyh = 0.0;
        self.gys = avr;
        self.gyt = ksn;
    }
    
    
    pub fn yqe(&mut self, avr: f32) {
        self.adh = 0.0;
        self.jsm = 1.0;
        self.bv = 0.95;
        self.jso = 1.0;
        self.dyh = 0.0;
        self.gys = avr;
        self.gyt = Easing::Arl;
    }
    
    
    pub fn cki(&mut self, avr: f32) {
        self.jsm = 0.0;
        self.jso = 0.95;
        self.dyh = 0.0;
        self.gys = avr;
        self.gyt = Easing::Bfq;
    }
}






pub struct Compositor {
    pub z: u32,
    pub ac: u32,
    pub axa: Vec<WindowSurface>,
    pub theme: CompositorTheme,
    pub cdb: u32,
    pub jr: bool,
    
    
    pub pkd: f32,
    pub gsl: f32,
    pub dby: f32,
    pub avn: f32,
    pub dek: f32,
    pub gbi: f32,
    
    
    pub time: f32,
    pub tz: f32,
    
    
    pub iln: u32,
    pub ilm: u32,
    pub haj: BackgroundPattern,
}

#[derive(Clone, Copy, PartialEq)]
pub enum BackgroundPattern {
    Aes,
    Bii,
    Pn,
    Cr,
    Bxx,
}

impl Compositor {
    pub const fn new() -> Self {
        Self {
            z: 1280,
            ac: 800,
            axa: Vec::new(),
            theme: CompositorTheme::Xq,
            cdb: 0xFF070707,
            jr: false,
            pkd: 8.0,
            gsl: 16.0,
            dby: 0.4,
            avn: 8.0,
            dek: 1.0,
            gbi: 0.0,
            time: 0.0,
            tz: 60.0,
            iln: 0xFF070707,
            ilm: 0xFF020303,
            haj: BackgroundPattern::Bii,
        }
    }
    
    
    pub fn init(&mut self, z: u32, ac: u32) {
        self.z = z;
        self.ac = ac;
        
        
        nyy(z, ac);
        kzd(TL_);
        kzd(ACV_);
        
        
        ixa(NF_);
        hlp();
        tft(0.0, z as f32, ac as f32, 0.0, -100.0, 100.0);
        
        ixa(ACY_);
        hlp();
        
        self.jr = true;
    }
    
    
    pub fn yep(&mut self, surface: WindowSurface) -> u32 {
        let ad = surface.ad;
        self.axa.push(surface);
        self.wqj();
        ad
    }
    
    
    pub fn zji(&mut self, ad: u32) {
        self.axa.ajm(|e| e.ad != ad);
    }
    
    
    pub fn teu(&mut self, ad: u32) -> Option<&mut WindowSurface> {
        self.axa.el().du(|e| e.ad == ad)
    }
    
    
    fn wqj(&mut self) {
        self.axa.bxe(|q, o| q.ell.cmp(&o.ell));
    }
    
    
    pub fn qs(&mut self, os: f32) {
        self.time += os;
        for surface in &mut self.axa {
            surface.qs(os);
        }
    }
    
    
    pub fn tj(&self) {
        if !self.jr {
            return;
        }
        
        
        nyx(
            ((self.cdb >> 16) & 0xFF) as f32 / 255.0,
            ((self.cdb >> 8) & 0xFF) as f32 / 255.0,
            (self.cdb & 0xFF) as f32 / 255.0,
            1.0,
        );
        nyw(ACW_ | ACX_);
        
        
        self.vvd();
        
        
        for surface in &self.axa {
            if !surface.iw || surface.adh <= 0.001 {
                continue;
            }
            self.vwq(surface);
        }
        
        
        tfr();
    }
    
    
    fn vvd(&self) {
        match self.haj {
            BackgroundPattern::Aes => {
                self.bdx(0.0, 0.0, self.z as f32, self.ac as f32, 
                                      self.cdb, -99.0);
            }
            BackgroundPattern::Bii => {
                self.krb(0.0, 0.0, self.z as f32, self.ac as f32,
                                        self.iln, self.ilm, -99.0);
            }
            BackgroundPattern::Pn => {
                self.krb(0.0, 0.0, self.z as f32, self.ac as f32,
                                        self.iln, self.ilm, -99.0);
                self.fgx(-98.0);
            }
            BackgroundPattern::Bxx => {
                self.sbc(-99.0);
            }
            _ => {
                self.bdx(0.0, 0.0, self.z as f32, self.ac as f32,
                                      self.cdb, -99.0);
            }
        }
    }
    
    
    fn sbc(&self, av: f32) {
        
        self.krb(0.0, 0.0, self.z as f32, self.ac as f32,
                                self.iln, self.ilm, av);
        
        
        let ab = self.time * 0.5;
        let tgd = (self.z as f32 / 2.0) + (ab.ayq() * 200.0);
        let tge = (self.ac as f32 / 3.0) + (ab.cjt() * 100.0);
        self.kra(tgd, tge, 300.0, 0x1000FF44, av + 0.1);
    }
    
    
    fn vwq(&self, surface: &WindowSurface) {
        let b = surface.b;
        let c = surface.c;
        let d = surface.z * surface.bv;
        let i = surface.ac * surface.bv;
        let av = surface.ell as f32;
        
        
        match self.theme {
            CompositorTheme::Xq => {
                
                if self.dby > 0.0 {
                    self.gfj(b, c, d, i, av - 0.5);
                }
                
                self.sgr(surface, av);
            }
            CompositorTheme::Ait => {
                
                self.sbw(b, c, d, i, av - 0.3);
                
                self.sdf(surface, av);
            }
            CompositorTheme::Tp => {
                
                self.sei(b, c, d, i, av - 0.5);
                
                self.seh(surface, av);
            }
            CompositorTheme::Gy => {
                
                self.sdw(surface, av);
            }
            CompositorTheme::Aif => {
                
                self.scy(surface, av);
            }
        }
        
        
        self.sfv(surface, av + 0.1);
    }
    
    
    fn gfj(&self, b: f32, c: f32, d: f32, i: f32, av: f32) {
        let l = self.pkd;
        let cou = self.gsl;
        let dw = (self.dby * 255.0) as u32;
        let dls = dw << 24;
        
        
        for a in 0..4 {
            let arb = cou * (a as f32 / 4.0);
            let udd = dw / (a + 1);
            let s = udd << 24;
            
            self.bdx(
                b + l - arb,
                c + l - arb,
                d + arb * 2.0,
                i + arb * 2.0,
                s,
                av - (a as f32 * 0.01),
            );
        }
    }
    
    
    fn sgr(&self, surface: &WindowSurface, av: f32) {
        let b = surface.b;
        let c = surface.c;
        let d = surface.z * surface.bv;
        let i = surface.ac * surface.bv;
        
        
        let bxn = 32.0;
        let ejy = if surface.ja { 0xFF0D120F } else { 0xFF0A0D0B };
        self.bdx(b, c, d, bxn, ejy, av);
        
        
        if surface.ja {
            self.bdx(b, c + bxn - 2.0, d, 2.0, 0xFF008844, av + 0.01);
        }
        
        
        let qqr = 0xFF0A0E0B;
        self.bdx(b, c + bxn, d, i - bxn, qqr, av);
        
        
        let aia = if surface.ja { 0xFF006633 } else { 0xFF004422 };
        self.epg(b, c, d, i, aia, av + 0.02);
        
        
        self.sgq(b + 8.0, c + 8.0, surface.ja, av + 0.03);
    }
    
    
    fn sdf(&self, surface: &WindowSurface, av: f32) {
        let b = surface.b;
        let c = surface.c;
        let d = surface.z * surface.bv;
        let i = surface.ac * surface.bv;
        
        
        let dw = (surface.adh * 0.85 * 255.0) as u32;
        let tfx = (dw << 24) | 0x0D1210;
        self.bdx(b, c, d, i, tfx, av);
        
        
        self.epg(b, c, d, i, 0x4000FF66, av + 0.01);
        
        
        self.bdx(b + 1.0, c + 1.0, d - 2.0, 1.0, 0x2000FF66, av + 0.02);
    }
    
    
    fn seh(&self, surface: &WindowSurface, av: f32) {
        let b = surface.b;
        let c = surface.c;
        let d = surface.z * surface.bv;
        let i = surface.ac * surface.bv;
        
        
        self.bdx(b, c, d, i, 0xFF050505, av);
        
        
        let bzv = if surface.ja { 0xFF00FF66 } else { 0xFF00AA44 };
        
        
        for a in 1..5 {
            let arb = a as f32 * 2.0;
            let dw = (60 - a * 15) as u32;
            let s = (dw << 24) | (bzv & 0x00FFFFFF);
            self.epg(b - arb, c - arb, d + arb * 2.0, i + arb * 2.0, s, av + 0.01);
        }
        
        
        self.epg(b, c, d, i, bzv, av + 0.05);
    }
    
    
    fn sdw(&self, surface: &WindowSurface, av: f32) {
        let b = surface.b;
        let c = surface.c;
        let d = surface.z * surface.bv;
        let i = surface.ac * surface.bv;
        
        
        self.bdx(b, c, d, i, 0xFF0A0E0B, av);
        
        
        let aia = if surface.ja { 0xFF00CC55 } else { 0xFF004422 };
        self.epg(b, c, d, i, aia, av + 0.01);
    }
    
    
    fn scy(&self, surface: &WindowSurface, av: f32) {
        let b = surface.b;
        let c = surface.c;
        let d = surface.z * surface.bv;
        let i = surface.ac * surface.bv;
        
        
        let bxn = 32.0;
        self.bdx(b, c, d, bxn, 0xFF0D120F, av);
        
        
        self.bdx(b, c + bxn, d, i - bxn, 0xFF0A0E0B, av);
        
        
        self.epg(b, c, d, i, 0xFF006633, av + 0.01);
    }
    
    
    fn sgq(&self, b: f32, c: f32, ja: bool, av: f32) {
        let aoa = 20.0;
        let dy = 6.0;
        
        
        let rbu = if ja { 0xFF4A3535 } else { 0xFF2A2020 };
        self.cxc(b, c + 8.0, dy, rbu, av);
        
        
        let uoj = if ja { 0xFF3A3A30 } else { 0xFF202010 };
        self.cxc(b + aoa, c + 8.0, dy, uoj, av);
        
        
        let ukx = if ja { 0xFF2A3A2F } else { 0xFF10201A };
        self.cxc(b + aoa * 2.0, c + 8.0, dy, ukx, av);
    }
    
    
    fn sfv(&self, surface: &WindowSurface, av: f32) {
        
        
        let b = surface.b;
        let c = surface.c + 32.0; 
        let d = surface.z * surface.bv;
        let i = (surface.ac - 32.0) * surface.bv;
        
        
    }
    
    
    
    
    
    
    fn bdx(&self, b: f32, c: f32, d: f32, i: f32, s: u32, av: f32) {
        let (m, at, o, q) = ent(s);
        
        cfa(KG_);
        erd(m, at, o, q);
        jx(b, c, av);
        jx(b + d, c, av);
        jx(b + d, c + i, av);
        jx(b, c + i, av);
        cfb();
    }
    
    
    fn krb(&self, b: f32, c: f32, d: f32, i: f32, 
                          idz: u32, hba: u32, av: f32) {
        let (aqh, cyd, of, km) = ent(idz);
        let (uv, cqu, tb, oe) = ent(hba);
        
        cfa(KG_);
        erd(aqh, cyd, of, km);
        jx(b, c, av);
        jx(b + d, c, av);
        erd(uv, cqu, tb, oe);
        jx(b + d, c + i, av);
        jx(b, c + i, av);
        cfb();
    }
    
    
    fn epg(&self, b: f32, c: f32, d: f32, i: f32, s: u32, av: f32) {
        let (m, at, o, q) = ent(s);
        
        cfa(NE_);
        erd(m, at, o, q);
        jx(b, c, av);
        jx(b + d, c, av);
        jx(b + d, c + i, av);
        jx(b, c + i, av);
        cfb();
    }
    
    
    fn cxc(&self, cx: f32, ae: f32, dy: f32, s: u32, av: f32) {
        let (m, at, o, q) = ent(s);
        let jq = 16;
        
        cfa(ATO_);
        erd(m, at, o, q);
        jx(cx, ae, av); 
        
        for a in 0..=jq {
            let hg = (a as f32 / jq as f32) * core::f32::consts::Eu * 2.0;
            let y = cx + hg.cjt() * dy;
            let x = ae + hg.ayq() * dy;
            jx(y, x, av);
        }
        cfb();
    }
    
    
    fn kra(&self, cx: f32, ae: f32, dy: f32, s: u32, av: f32) {
        let (m, at, o, _) = ent(s);
        
        
        for a in 0..8 {
            let ab = a as f32 / 8.0;
            let nij = dy * (0.3 + ab * 0.7);
            let dw = 0.3 * (1.0 - ab);
            
            cfa(ATO_);
            erd(m, at, o, dw);
            jx(cx, ae, av);
            
            let jq = 24;
            for fb in 0..=jq {
                let hg = (fb as f32 / jq as f32) * core::f32::consts::Eu * 2.0;
                let y = cx + hg.cjt() * nij;
                let x = ae + hg.ayq() * nij;
                jx(y, x, av);
            }
            cfb();
        }
    }
    
    
    fn sei(&self, b: f32, c: f32, d: f32, i: f32, av: f32) {
        let bzv = 0x00FF66u32;
        let (m, at, o, _) = ent(bzv);
        
        for a in 1..6 {
            let arb = a as f32 * 3.0;
            let dw = 0.4 / (a as f32);
            
            cfa(NE_);
            erd(m, at, o, dw);
            jx(b - arb, c - arb, av);
            jx(b + d + arb, c - arb, av);
            jx(b + d + arb, c + i + arb, av);
            jx(b - arb, c + i + arb, av);
            cfb();
        }
    }
    
    
    fn sbw(&self, b: f32, c: f32, d: f32, i: f32, av: f32) {
        
        self.bdx(b, c, d, i, 0x800D1210, av);
    }
    
    
    fn fgx(&self, av: f32) {
        let lak = 0x08004422u32;
        let (m, at, o, q) = ent(lak);
        let aoa = 40.0;
        
        cfa(TO_);
        erd(m, at, o, q);
        
        
        let mut b = 0.0;
        while b < self.z as f32 {
            jx(b, 0.0, av);
            jx(b, self.ac as f32, av);
            b += aoa;
        }
        
        
        let mut c = 0.0;
        while c < self.ac as f32 {
            jx(0.0, c, av);
            jx(self.z as f32, c, av);
            c += aoa;
        }
        cfb();
    }
    
    
    pub fn bxb(&mut self, theme: CompositorTheme) {
        self.theme = theme;
        
        
        match theme {
            CompositorTheme::Xq => {
                self.dby = 0.4;
                self.gsl = 16.0;
                self.avn = 8.0;
                self.gbi = 0.0;
            }
            CompositorTheme::Ait => {
                self.dby = 0.2;
                self.gsl = 24.0;
                self.avn = 12.0;
                self.gbi = 0.3;
            }
            CompositorTheme::Tp => {
                self.dby = 0.0;
                self.avn = 4.0;
                self.gbi = 1.0;
                self.haj = BackgroundPattern::Pn;
            }
            CompositorTheme::Gy => {
                self.dby = 0.0;
                self.gsl = 0.0;
                self.avn = 0.0;
                self.gbi = 0.0;
            }
            CompositorTheme::Aif => {
                self.dby = 0.0;
                self.gsl = 0.0;
                self.avn = 0.0;
                self.gbi = 0.0;
            }
        }
    }
}






fn ent(s: u32) -> (f32, f32, f32, f32) {
    let q = ((s >> 24) & 0xFF) as f32 / 255.0;
    let m = ((s >> 16) & 0xFF) as f32 / 255.0;
    let at = ((s >> 8) & 0xFF) as f32 / 255.0;
    let o = (s & 0xFF) as f32 / 255.0;
    (m, at, o, q)
}

use crate::math::csb;


fn qjs(ab: f32, ksn: Easing) -> f32 {
    match ksn {
        Easing::Cgp => ab,
        Easing::Bfq => ab * ab,
        Easing::Arl => 1.0 - (1.0 - ab) * (1.0 - ab),
        Easing::Cbn => {
            if ab < 0.5 {
                2.0 * ab * ab
            } else {
                1.0 - (-2.0 * ab + 2.0).zgc(2) / 2.0
            }
        }
        Easing::Byr => {
            let jgj = 7.5625;
            let apo = 2.75;
            let mut ab = ab;
            if ab < 1.0 / apo {
                jgj * ab * ab
            } else if ab < 2.0 / apo {
                ab -= 1.5 / apo;
                jgj * ab * ab + 0.75
            } else if ab < 2.5 / apo {
                ab -= 2.25 / apo;
                jgj * ab * ab + 0.9375
            } else {
                ab -= 2.625 / apo;
                jgj * ab * ab + 0.984375
            }
        }
        Easing::Cbo => {
            if ab == 0.0 || ab == 1.0 {
                ab
            } else {
                let ai = 0.3;
                let e = ai / 4.0;
                (2.0f32).zgb(-10.0 * ab) * ((ab - e) * (2.0 * core::f32::consts::Eu / ai)).ayq() + 1.0
            }
        }
    }
}





static Oz: Mutex<Compositor> = Mutex::new(Compositor::new());


pub fn compositor() -> spin::Aki<'static, Compositor> {
    Oz.lock()
}


pub fn ttg(z: u32, ac: u32) {
    compositor().init(z, ac);
}


pub fn pis(theme: CompositorTheme) {
    compositor().bxb(theme);
}


pub fn vvm() {
    compositor().tj();
}
