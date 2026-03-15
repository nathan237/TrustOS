







use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;

use super::{
    Cf, WidgetState, Color, Theme, Rect, Point, Size, EdgeInsets,
    UiEvent, MouseEvent, MouseButton, KeyEvent,
    bvo, cb,
};
use crate::drivers::virtio_gpu::GpuSurface;






pub struct Bsz {
    ad: u32,
    eg: Rect,
    g: WidgetState,
    pub bn: f32,        
    pub v: f32,
    pub am: f32,
    pub gu: f32,
    pub pko: bool,
    pub bks: Option<Box<dyn Fn(f32) + Send + Sync>>,
    cka: bool,
}

impl Bsz {
    pub fn new(v: f32, am: f32, bn: f32) -> Self {
        Self {
            ad: bvo(),
            eg: Rect::Dh,
            g: WidgetState::new(),
            bn: ((bn - v) / (am - v)).qp(0.0, 1.0),
            v,
            am,
            gu: 0.0,
            pko: true,
            bks: None,
            cka: false,
        }
    }
    
    pub fn zwi(mut self, gu: f32) -> Self {
        self.gu = gu;
        self
    }
    
    pub fn bks<G: Fn(f32) + Send + Sync + 'static>(mut self, bb: G) -> Self {
        self.bks = Some(Box::new(bb));
        self
    }
    
    pub fn mtt(&self) -> f32 {
        self.v + self.bn * (self.am - self.v)
    }
    
    fn pxe(&mut self, b: i32) {
        let pvr = self.eg.b + 8;
        let xlk = self.eg.b + self.eg.z as i32 - 8;
        let pvs = xlk - pvr;
        
        if pvs > 0 {
            let atj = (b - pvr) as f32 / pvs as f32;
            self.bn = atj.qp(0.0, 1.0);
            
            
            if self.gu > 0.0 {
                let cmb = self.am - self.v;
                let wue = self.bn * cmb / self.gu;
                let au = (wue + 0.5) as i32 as f32; 
                self.bn = (au * self.gu / cmb).qp(0.0, 1.0);
            }
            
            if let Some(ref bks) = self.bks {
                bks(self.mtt());
            }
        }
    }
}

impl Cf for Bsz {
    fn ad(&self) -> u32 { self.ad }
    fn eg(&self) -> Rect { self.eg }
    fn cbq(&mut self, eg: Rect) { self.eg = eg; }
    fn g(&self) -> WidgetState { self.g }
    fn cbr(&mut self, g: WidgetState) { self.g = g; }
    
    fn ctk(&self) -> Size {
        Size::new(200, 24)
    }
    
    fn ecj(&mut self, id: &UiEvent) -> bool {
        match id {
            UiEvent::Cp(MouseEvent::Fm { b, bdp: MouseButton::Ap, .. }) => {
                self.cka = true;
                self.pxe(*b);
                true
            }
            UiEvent::Cp(MouseEvent::Fw { b, .. }) if self.cka => {
                self.pxe(*b);
                true
            }
            UiEvent::Cp(MouseEvent::Ek { bdp: MouseButton::Ap, .. }) => {
                self.cka = false;
                true
            }
            _ => false
        }
    }
    
    fn tj(&self, surface: &mut GpuSurface, theme: &Theme) {
        if !self.g.iw { return; }
        
        let b = self.eg.b as u32;
        let c = self.eg.c as u32;
        let d = self.eg.z;
        let i = self.eg.ac;
        
        
        let ekc = c + i / 2 - 2;
        surface.afp(b + 4, ekc, d - 8, 4, 2, theme.ems.lv());
        
        
        let eqh = ((d - 16) as f32 * self.bn) as u32;
        if eqh > 0 {
            surface.afp(b + 4, ekc, eqh + 4, 4, 2, theme.mm.lv());
        }
        
        
        let xgq = b + 4 + eqh;
        let bsm = c + i / 2 - 8;
        let xgo = if self.cka { theme.mm } else if self.g.asy { theme.cof } else { theme.bui };
        surface.abc(xgq as i32 + 4, bsm as i32 + 8, 8, xgo.lv());
        
        
        if self.pko {
            let ap = self.mtt();
            let text = if self.gu >= 1.0 {
                format!("{}", ap as i32)
            } else {
                format!("{:.1}", ap)
            };
            let wg = b as i32 + d as i32 + 8;
            let sl = c as i32 + (i as i32 - 16) / 2;
            cb(surface, wg, sl, &text, theme.ebn.lv());
        }
    }
}






pub struct Bqi {
    ad: u32,
    eg: Rect,
    g: WidgetState,
    pub na: bool,
    pub cu: String,
    pub cyi: u32,
    pub goj: Option<Box<dyn Fn() + Send + Sync>>,
}

impl Bqi {
    pub fn new(cu: impl Into<String>, cyi: u32) -> Self {
        Self {
            ad: bvo(),
            eg: Rect::Dh,
            g: WidgetState::new(),
            na: false,
            cu: cu.into(),
            cyi,
            goj: None,
        }
    }
    
    pub fn na(mut self, bn: bool) -> Self {
        self.na = bn;
        self
    }
    
    pub fn goj<G: Fn() + Send + Sync + 'static>(mut self, bb: G) -> Self {
        self.goj = Some(Box::new(bb));
        self
    }
}

impl Cf for Bqi {
    fn ad(&self) -> u32 { self.ad }
    fn eg(&self) -> Rect { self.eg }
    fn cbq(&mut self, eg: Rect) { self.eg = eg; }
    fn g(&self) -> WidgetState { self.g }
    fn cbr(&mut self, g: WidgetState) { self.g = g; }
    
    fn ctk(&self) -> Size {
        Size::new(24 + self.cu.len() as u32 * 8 + 8, 24)
    }
    
    fn ecj(&mut self, id: &UiEvent) -> bool {
        match id {
            UiEvent::Cp(MouseEvent::Vy { bdp: MouseButton::Ap, .. }) => {
                self.na = true;
                if let Some(ref goj) = self.goj {
                    goj();
                }
                true
            }
            _ => false
        }
    }
    
    fn tj(&self, surface: &mut GpuSurface, theme: &Theme) {
        if !self.g.iw { return; }
        
        let nda = self.eg.b as i32 + 10;
        let ndb = self.eg.c as i32 + self.eg.ac as i32 / 2;
        
        
        surface.cxc(nda, ndb, 8, theme.acu.lv());
        
        
        if self.na {
            surface.abc(nda, ndb, 5, theme.mm.lv());
        }
        
        
        let wg = self.eg.b as i32 + 28;
        let sl = self.eg.c as i32 + (self.eg.ac as i32 - 16) / 2;
        cb(surface, wg, sl, &self.cu, theme.bui.lv());
    }
}






pub struct Bfa {
    ad: u32,
    eg: Rect,
    g: WidgetState,
    pub options: Vec<String>,
    pub acm: usize,
    pub tg: bool,
    pub bks: Option<Box<dyn Fn(usize) + Send + Sync>>,
}

impl Bfa {
    pub fn new(options: Vec<String>) -> Self {
        Self {
            ad: bvo(),
            eg: Rect::Dh,
            g: WidgetState::new(),
            options,
            acm: 0,
            tg: false,
            bks: None,
        }
    }
    
    pub fn na(mut self, index: usize) -> Self {
        self.acm = index.v(self.options.len().ao(1));
        self
    }
    
    pub fn bks<G: Fn(usize) + Send + Sync + 'static>(mut self, bb: G) -> Self {
        self.bks = Some(Box::new(bb));
        self
    }
    
    pub fn wgw(&self) -> Option<&String> {
        self.options.get(self.acm)
    }
}

impl Cf for Bfa {
    fn ad(&self) -> u32 { self.ad }
    fn eg(&self) -> Rect { self.eg }
    fn cbq(&mut self, eg: Rect) { self.eg = eg; }
    fn g(&self) -> WidgetState { self.g }
    fn cbr(&mut self, g: WidgetState) { self.g = g; }
    
    fn ctk(&self) -> Size {
        let cat = self.options.iter().map(|e| e.len()).am().unwrap_or(10);
        Size::new((cat as u32 * 8 + 32).am(120), 32)
    }
    
    fn ecj(&mut self, id: &UiEvent) -> bool {
        match id {
            UiEvent::Cp(MouseEvent::Vy { b, c, bdp: MouseButton::Ap }) => {
                if self.tg {
                    
                    let uyy = self.eg.c + self.eg.ac as i32;
                    let w = ((*c - uyy) / 28) as usize;
                    if w < self.options.len() {
                        self.acm = w;
                        if let Some(ref bks) = self.bks {
                            bks(w);
                        }
                    }
                    self.tg = false;
                } else {
                    self.tg = true;
                }
                true
            }
            UiEvent::Bct => {
                self.tg = false;
                true
            }
            _ => false
        }
    }
    
    fn tj(&self, surface: &mut GpuSurface, theme: &Theme) {
        if !self.g.iw { return; }
        
        let b = self.eg.b as u32;
        let c = self.eg.c as u32;
        let d = self.eg.z;
        let i = self.eg.ac;
        
        
        let ei = if self.g.asy { theme.dor } else { theme.dop };
        surface.afp(b, c, d, i, theme.avh, ei.lv());
        surface.mf(b, c, d, i, theme.avh, theme.acu.lv());
        
        
        if let Some(text) = self.wgw() {
            let wg = b as i32 + 12;
            let sl = c as i32 + (i as i32 - 16) / 2;
            cb(surface, wg, sl, text, theme.bui.lv());
        }
        
        
        let fct = b + d - 20;
        let ika = c + i / 2;
        surface.ahj(fct as i32, ika as i32 - 2, fct as i32 + 4, ika as i32 + 2, theme.ebn.lv());
        surface.ahj(fct as i32 + 4, ika as i32 + 2, fct as i32 + 8, ika as i32 - 2, theme.ebn.lv());
        
        
        if self.tg {
            let krp = c + i + 2;
            let nnw = (self.options.len() as u32 * 28).v(200);
            
            surface.afp(b, krp, d, nnw, theme.avh, theme.fdh.lv());
            surface.mf(b, krp, d, nnw, theme.avh, theme.acu.lv());
            
            for (a, option) in self.options.iter().cf() {
                let osu = krp + a as u32 * 28;
                
                if a == self.acm {
                    surface.ah(b + 2, osu + 2, d - 4, 24, theme.mm.fbo(40).lv());
                }
                
                let wg = b as i32 + 12;
                let sl = osu as i32 + 6;
                let s = if a == self.acm { theme.mm } else { theme.bui };
                cb(surface, wg, sl, option, s.lv());
            }
        }
    }
}






pub struct Bsh {
    ad: u32,
    eg: Rect,
    g: WidgetState,
    pub eny: u32,
    pub ug: i32,
    pub eie: i32,
}

impl Bsh {
    pub fn new(eny: u32) -> Self {
        Self {
            ad: bvo(),
            eg: Rect::Dh,
            g: WidgetState::new(),
            eny,
            ug: 0,
            eie: 20,
        }
    }
    
    pub fn wet(&mut self, c: i32) {
        let aye = (self.eny as i32 - self.eg.ac as i32).am(0);
        self.ug = c.qp(0, aye);
    }
    
    pub fn wer(&mut self, aaq: i32) {
        self.wet(self.ug + aaq);
    }
    
    
    pub fn zvl(&self) -> Rect {
        Rect::new(
            self.eg.b,
            self.eg.c,
            self.eg.z.ao(12), 
            self.eg.ac,
        )
    }
}

impl Cf for Bsh {
    fn ad(&self) -> u32 { self.ad }
    fn eg(&self) -> Rect { self.eg }
    fn cbq(&mut self, eg: Rect) { self.eg = eg; }
    fn g(&self) -> WidgetState { self.g }
    fn cbr(&mut self, g: WidgetState) { self.g = g; }
    
    fn ecj(&mut self, id: &UiEvent) -> bool {
        match id {
            UiEvent::Cp(MouseEvent::Yq { aaq, .. }) => {
                self.wer(-aaq * self.eie);
                true
            }
            _ => false
        }
    }
    
    fn tj(&self, surface: &mut GpuSurface, theme: &Theme) {
        if !self.g.iw { return; }
        
        let b = self.eg.b as u32;
        let c = self.eg.c as u32;
        let d = self.eg.z;
        let i = self.eg.ac;
        
        
        surface.ah(b, c, d, i, theme.gay.lv());
        
        
        let auz = b + d - 8;
        surface.ah(auz, c, 8, i, theme.ems.lv());
        
        
        if self.eny > i {
            let xsb = i as f32 / self.eny as f32;
            let psz = ((i as f32 * xsb) as u32).am(20);
            let mcp = self.ug as f32 / (self.eny - i) as f32;
            let bsm = c + ((i - psz) as f32 * mcp) as u32;
            
            surface.afp(auz, bsm, 8, psz, 4, theme.ebn.lv());
        }
    }
}






pub struct Bmk {
    ad: u32,
    eg: Rect,
    g: WidgetState,
    pub dq: String,
    pub z: u32,
    pub ac: u32,
    pub iw: bool,
}

impl Bmk {
    pub fn new(dq: impl Into<String>, z: u32, ac: u32) -> Self {
        Self {
            ad: bvo(),
            eg: Rect::Dh,
            g: WidgetState::new(),
            dq: dq.into(),
            z,
            ac,
            iw: false,
        }
    }
    
    pub fn iah(&mut self) {
        self.iw = true;
    }
    
    pub fn tos(&mut self) {
        self.iw = false;
    }
    
    
    pub fn roj(&self) -> Rect {
        Rect::new(
            self.eg.b + 16,
            self.eg.c + 48,
            self.z - 32,
            self.ac - 64,
        )
    }
}

impl Cf for Bmk {
    fn ad(&self) -> u32 { self.ad }
    fn eg(&self) -> Rect { self.eg }
    fn cbq(&mut self, eg: Rect) { 
        
        let wf = eg.z;
        let aav = eg.ac;
        self.eg = Rect::new(
            (wf as i32 - self.z as i32) / 2,
            (aav as i32 - self.ac as i32) / 2,
            self.z,
            self.ac,
        );
    }
    fn g(&self) -> WidgetState { self.g }
    fn cbr(&mut self, g: WidgetState) { self.g = g; }
    
    fn tj(&self, surface: &mut GpuSurface, theme: &Theme) {
        if !self.iw { return; }
        
        let (kp, kl) = (surface.z, surface.ac);
        
        
        for c in 0..kl {
            for b in 0..kp {
                let xy = surface.beg(b, c);
                let rxo = ((xy >> 16 & 0xFF) / 2) << 16 
                           | ((xy >> 8 & 0xFF) / 2) << 8 
                           | (xy & 0xFF) / 2
                           | 0xC0000000;
                surface.aht(b, c, rxo);
            }
        }
        
        let b = self.eg.b as u32;
        let c = self.eg.c as u32;
        let d = self.z;
        let i = self.ac;
        
        
        surface.afp(b + 4, c + 4, d, i, 8, 0x40000000); 
        surface.afp(b, c, d, i, 8, theme.fdh.lv());
        surface.mf(b, c, d, i, 8, theme.acu.lv());
        
        
        surface.ah(b + 1, c + 1, d - 2, 40, theme.ems.lv());
        
        
        let cnf = b as i32 + 16;
        let cce = c as i32 + 12;
        cb(surface, cnf, cce, &self.dq, theme.bui.lv());
        
        
        let bdr = b + d - 32;
        let ndq = c + 12;
        surface.afp(bdr, ndq, 20, 20, 4, theme.zt.fbo(40).lv());
        cb(surface, bdr as i32 + 6, ndq as i32 + 2, "×", theme.zt.lv());
    }
}






pub struct Dz {
    ad: u32,
    eg: Rect,
    g: WidgetState,
    pub dq: Option<String>,
    pub ob: EdgeInsets,
}

impl Dz {
    pub fn new() -> Self {
        Self {
            ad: bvo(),
            eg: Rect::Dh,
            g: WidgetState::new(),
            dq: None,
            ob: EdgeInsets::xx(12),
        }
    }
    
    pub fn zwj(mut self, dq: impl Into<String>) -> Self {
        self.dq = Some(dq.into());
        self
    }
    
    pub fn zwe(mut self, ob: EdgeInsets) -> Self {
        self.ob = ob;
        self
    }
    
    
    pub fn roj(&self) -> Rect {
        let ptm = if self.dq.is_some() { 32 } else { 0 };
        Rect::new(
            self.eg.b + self.ob.fd as i32,
            self.eg.c + self.ob.qc as i32 + ptm,
            self.eg.z - self.ob.fd - self.ob.hw,
            self.eg.ac - self.ob.qc - self.ob.abm - ptm as u32,
        )
    }
}

impl Cf for Dz {
    fn ad(&self) -> u32 { self.ad }
    fn eg(&self) -> Rect { self.eg }
    fn cbq(&mut self, eg: Rect) { self.eg = eg; }
    fn g(&self) -> WidgetState { self.g }
    fn cbr(&mut self, g: WidgetState) { self.g = g; }
    
    fn tj(&self, surface: &mut GpuSurface, theme: &Theme) {
        if !self.g.iw { return; }
        
        let b = self.eg.b as u32;
        let c = self.eg.c as u32;
        let d = self.eg.z;
        let i = self.eg.ac;
        
        
        surface.afp(b, c, d, i, theme.avh, theme.fdh.lv());
        surface.mf(b, c, d, i, theme.avh, theme.acu.lv());
        
        
        if let Some(ref dq) = self.dq {
            let cce = c + 8;
            cb(surface, b as i32 + 12, cce as i32, dq, theme.ebn.lv());
            
            
            surface.ahj(b as i32 + 8, (c + 28) as i32, (b + d - 8) as i32, (c + 28) as i32, theme.acu.lv());
        }
    }
}






pub struct Ber {
    ad: u32,
    eg: Rect,
    g: WidgetState,
    pub cns: bool,
}

impl Ber {
    pub fn dic() -> Self {
        Self {
            ad: bvo(),
            eg: Rect::Dh,
            g: WidgetState::new(),
            cns: false,
        }
    }
    
    pub fn cns() -> Self {
        Self {
            ad: bvo(),
            eg: Rect::Dh,
            g: WidgetState::new(),
            cns: true,
        }
    }
}

impl Cf for Ber {
    fn ad(&self) -> u32 { self.ad }
    fn eg(&self) -> Rect { self.eg }
    fn cbq(&mut self, eg: Rect) { self.eg = eg; }
    fn g(&self) -> WidgetState { self.g }
    fn cbr(&mut self, g: WidgetState) { self.g = g; }
    
    fn ctk(&self) -> Size {
        if self.cns {
            Size::new(1, 20)
        } else {
            Size::new(20, 1)
        }
    }
    
    fn tj(&self, surface: &mut GpuSurface, theme: &Theme) {
        if !self.g.iw { return; }
        
        let b = self.eg.b as u32;
        let c = self.eg.c as u32;
        
        if self.cns {
            surface.ah(b, c, 1, self.eg.ac, theme.acu.lv());
        } else {
            surface.ah(b, c, self.eg.z, 1, theme.acu.lv());
        }
    }
}






pub struct Bji {
    ad: u32,
    eg: Rect,
    g: WidgetState,
    pub pa: char,
    pub jtm: Option<String>,
    pub ctb: Option<Box<dyn Fn() + Send + Sync>>,
}

impl Bji {
    pub fn new(pa: char) -> Self {
        Self {
            ad: bvo(),
            eg: Rect::Dh,
            g: WidgetState::new(),
            pa,
            jtm: None,
            ctb: None,
        }
    }
    
    pub fn zwk(mut self, jtm: impl Into<String>) -> Self {
        self.jtm = Some(jtm.into());
        self
    }
    
    pub fn ctb<G: Fn() + Send + Sync + 'static>(mut self, bb: G) -> Self {
        self.ctb = Some(Box::new(bb));
        self
    }
}

impl Cf for Bji {
    fn ad(&self) -> u32 { self.ad }
    fn eg(&self) -> Rect { self.eg }
    fn cbq(&mut self, eg: Rect) { self.eg = eg; }
    fn g(&self) -> WidgetState { self.g }
    fn cbr(&mut self, g: WidgetState) { self.g = g; }
    
    fn ctk(&self) -> Size {
        Size::new(36, 36)
    }
    
    fn ecj(&mut self, id: &UiEvent) -> bool {
        match id {
            UiEvent::Cp(MouseEvent::Bfx) => {
                self.g.asy = true;
                true
            }
            UiEvent::Cp(MouseEvent::Tf) => {
                self.g.asy = false;
                self.g.vn = false;
                true
            }
            UiEvent::Cp(MouseEvent::Fm { bdp: MouseButton::Ap, .. }) => {
                self.g.vn = true;
                true
            }
            UiEvent::Cp(MouseEvent::Ek { bdp: MouseButton::Ap, .. }) => {
                if self.g.vn {
                    self.g.vn = false;
                    if let Some(ref ctb) = self.ctb {
                        ctb();
                    }
                }
                true
            }
            _ => false
        }
    }
    
    fn tj(&self, surface: &mut GpuSurface, theme: &Theme) {
        if !self.g.iw { return; }
        
        let b = self.eg.b as u32;
        let c = self.eg.c as u32;
        let aw = self.eg.z.v(self.eg.ac);
        
        let ei = if self.g.vn {
            theme.dzj
        } else if self.g.asy {
            theme.dor
        } else {
            Color::Anl
        };
        
        if ei.q > 0 {
            surface.afp(b, c, aw, aw, aw / 2, ei.lv());
        }
        
        
        let hnp = alloc::string::String::from(self.pa);
        let wg = b as i32 + (aw as i32 - 8) / 2;
        let sl = c as i32 + (aw as i32 - 16) / 2;
        cb(surface, wg, sl, &hnp, theme.bui.lv());
    }
}






pub struct Coe {
    pub text: String,
    pub b: i32,
    pub c: i32,
    pub iw: bool,
}

impl Coe {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            b: 0,
            c: 0,
            iw: false,
        }
    }
    
    pub fn zof(&mut self, b: i32, c: i32) {
        self.b = b;
        self.c = c;
        self.iw = true;
    }
    
    pub fn tos(&mut self) {
        self.iw = false;
    }
    
    pub fn tj(&self, surface: &mut GpuSurface, theme: &Theme) {
        if !self.iw { return; }
        
        let ob = 8;
        let d = (self.text.len() as u32 * 8 + ob * 2) as u32;
        let i = 24;
        
        let b = self.b as u32;
        let c = self.c as u32;
        
        
        surface.afp(b, c, d, i, 4, theme.ems.lv());
        surface.mf(b, c, d, i, 4, theme.acu.lv());
        
        
        cb(surface, b as i32 + ob as i32, c as i32 + 4, &self.text, theme.bui.lv());
    }
}
