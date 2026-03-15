

















pub mod widgets;

pub use widgets::*;

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use core::any::Eb;
use spin::Mutex;

use crate::drivers::virtio_gpu::{GpuSurface, Compositor};






#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color {
    pub m: u8,
    pub at: u8,
    pub o: u8,
    pub q: u8,
}

impl Color {
    pub const fn new(m: u8, at: u8, o: u8, q: u8) -> Self {
        Self { m, at, o, q }
    }
    
    pub const fn xt(m: u8, at: u8, o: u8) -> Self {
        Self { m, at, o, q: 255 }
    }
    
    pub const fn zi(s: u32) -> Self {
        Self {
            q: ((s >> 24) & 0xFF) as u8,
            m: ((s >> 16) & 0xFF) as u8,
            at: ((s >> 8) & 0xFF) as u8,
            o: (s & 0xFF) as u8,
        }
    }
    
    pub const fn lv(self) -> u32 {
        ((self.q as u32) << 24) | ((self.m as u32) << 16) | ((self.at as u32) << 8) | (self.o as u32)
    }
    
    pub fn fbo(self, q: u8) -> Self {
        Self { q, ..self }
    }
    
    pub fn clh(self, bdk: u8) -> Self {
        Self {
            m: self.m.akq(bdk),
            at: self.at.akq(bdk),
            o: self.o.akq(bdk),
            q: self.q,
        }
    }
    
    pub fn cdz(self, bdk: u8) -> Self {
        Self {
            m: self.m.ao(bdk),
            at: self.at.ao(bdk),
            o: self.o.ao(bdk),
            q: self.q,
        }
    }
    
    
    pub const Anl: Color = Color::new(0, 0, 0, 0);
    pub const Ox: Color = Color::xt(0, 0, 0);
    pub const Zm: Color = Color::xt(255, 255, 255);
    pub const Bqa: Color = Color::xt(255, 0, 0);
    pub const Bht: Color = Color::xt(0, 255, 0);
    pub const Bci: Color = Color::xt(0, 0, 255);
    pub const Cqs: Color = Color::xt(255, 255, 0);
    pub const Bzg: Color = Color::xt(0, 255, 255);
    pub const Cgz: Color = Color::xt(255, 0, 255);
}


#[derive(Clone)]
pub struct Theme {
    
    pub gay: Color,
    pub fdh: Color,
    pub ems: Color,
    
    
    pub bui: Color,
    pub ebn: Color,
    pub iui: Color,
    
    
    pub mm: Color,
    pub cof: Color,
    pub fzq: Color,
    
    
    pub dop: Color,
    pub dor: Color,
    pub dzj: Color,
    pub kfp: Color,
    
    
    pub acu: Color,
    pub imc: Color,
    
    
    pub vx: Color,
    pub ekt: Color,
    pub zt: Color,
    pub co: Color,
    
    
    pub avh: u32,
    pub dek: u32,
    pub ob: u32,
    pub aoa: u32,
    
    
    pub asv: u32,
    pub nvl: u32,
    pub nvk: u32,
}

impl Theme {
    
    pub fn dark() -> Self {
        Self {
            gay: Color::zi(0xFF0A0E0B),
            fdh: Color::zi(0xFF141A17),
            ems: Color::zi(0xFF1E2620),
            
            bui: Color::zi(0xFF00FF66),
            ebn: Color::zi(0xFF00CC55),
            iui: Color::zi(0xFF4A5A4E),
            
            mm: Color::zi(0xFF00FF66),
            cof: Color::zi(0xFF00CC55),
            fzq: Color::zi(0xFF00AA44),
            
            dop: Color::zi(0xFF1E2620),
            dor: Color::zi(0xFF2A3630),
            dzj: Color::zi(0xFF354540),
            kfp: Color::zi(0xFF1A1A1A),
            
            acu: Color::zi(0xFF2A3A2F),
            imc: Color::zi(0xFF00FF66),
            
            vx: Color::zi(0xFF00FF66),
            ekt: Color::zi(0xFFFFD166),
            zt: Color::zi(0xFFFF6B6B),
            co: Color::zi(0xFF4ECDC4),
            
            avh: 6,
            dek: 1,
            ob: 12,
            aoa: 8,
            
            asv: 14,
            nvl: 12,
            nvk: 18,
        }
    }
    
    
    pub fn light() -> Self {
        Self {
            gay: Color::zi(0xFFF5F5F5),
            fdh: Color::zi(0xFFFFFFFF),
            ems: Color::zi(0xFFE8E8E8),
            
            bui: Color::zi(0xFF1A1A1A),
            ebn: Color::zi(0xFF4A4A4A),
            iui: Color::zi(0xFFAAAAAA),
            
            mm: Color::zi(0xFF0066FF),
            cof: Color::zi(0xFF0055DD),
            fzq: Color::zi(0xFF0044BB),
            
            dop: Color::zi(0xFFE8E8E8),
            dor: Color::zi(0xFFDDDDDD),
            dzj: Color::zi(0xFFCCCCCC),
            kfp: Color::zi(0xFFF0F0F0),
            
            acu: Color::zi(0xFFCCCCCC),
            imc: Color::zi(0xFF0066FF),
            
            vx: Color::zi(0xFF22BB44),
            ekt: Color::zi(0xFFFFAA00),
            zt: Color::zi(0xFFDD3333),
            co: Color::zi(0xFF2299DD),
            
            avh: 6,
            dek: 1,
            ob: 12,
            aoa: 8,
            
            asv: 14,
            nvl: 12,
            nvk: 18,
        }
    }
}


static MR_: Mutex<Option<Theme>> = Mutex::new(None);

pub fn bxb(theme: Theme) {
    *MR_.lock() = Some(theme);
}

pub fn yua() -> Theme {
    MR_.lock().clone().unwrap_or_else(Theme::dark)
}






#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Point {
    pub b: i32,
    pub c: i32,
}

impl Point {
    pub const fn new(b: i32, c: i32) -> Self {
        Self { b, c }
    }
    
    pub const Dh: Point = Point::new(0, 0);
}


#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Size {
    pub z: u32,
    pub ac: u32,
}

impl Size {
    pub const fn new(z: u32, ac: u32) -> Self {
        Self { z, ac }
    }
    
    pub const Dh: Size = Size::new(0, 0);
}


#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rect {
    pub b: i32,
    pub c: i32,
    pub z: u32,
    pub ac: u32,
}

impl Rect {
    pub const fn new(b: i32, c: i32, z: u32, ac: u32) -> Self {
        Self { b, c, z, ac }
    }
    
    pub fn yrr(pr: Point, pf: Point) -> Self {
        let b = pr.b.v(pf.b);
        let c = pr.c.v(pf.c);
        let z = (pr.b - pf.b).eki();
        let ac = (pr.c - pf.c).eki();
        Self { b, c, z, ac }
    }
    
    pub fn contains(&self, nl: Point) -> bool {
        nl.b >= self.b && nl.b < self.b + self.z as i32 &&
        nl.c >= self.c && nl.c < self.c + self.ac as i32
    }
    
    pub fn jao(&self, gq: &Rect) -> bool {
        !(self.b + self.z as i32 <= gq.b ||
          gq.b + gq.z as i32 <= self.b ||
          self.c + self.ac as i32 <= gq.c ||
          gq.c + gq.ac as i32 <= self.c)
    }
    
    pub fn hw(&self) -> i32 {
        self.b + self.z as i32
    }
    
    pub fn abm(&self) -> i32 {
        self.c + self.ac as i32
    }
    
    pub const Dh: Rect = Rect::new(0, 0, 0, 0);
}


#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct EdgeInsets {
    pub qc: u32,
    pub hw: u32,
    pub abm: u32,
    pub fd: u32,
}

impl EdgeInsets {
    pub const fn xx(bn: u32) -> Self {
        Self { qc: bn, hw: bn, abm: bn, fd: bn }
    }
    
    pub const fn wwy(cns: u32, dic: u32) -> Self {
        Self { qc: cns, hw: dic, abm: cns, fd: dic }
    }
    
    pub const fn uyk(qc: u32, hw: u32, abm: u32, fd: u32) -> Self {
        Self { qc, hw, abm, fd }
    }
    
    pub const Dh: EdgeInsets = EdgeInsets::xx(0);
}






#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MouseButton {
    Ap,
    Ca,
    Chk,
}


#[derive(Clone, Debug)]
pub enum MouseEvent {
    Fw { b: i32, c: i32 },
    Fm { b: i32, c: i32, bdp: MouseButton },
    Ek { b: i32, c: i32, bdp: MouseButton },
    Vy { b: i32, c: i32, bdp: MouseButton },
    Cum { b: i32, c: i32, bdp: MouseButton },
    Yq { b: i32, c: i32, aaq: i32 },
    Bfx,
    Tf,
}


#[derive(Clone, Debug)]
pub enum KeyEvent {
    Fm { bs: char, modifiers: u8 },
    Ek { bs: char, modifiers: u8 },
    Bzj { r: char },
}


pub mod modifiers {
    pub const Bri: u8 = 0x01;
    pub const Bdf: u8 = 0x02;
    pub const Bbi: u8 = 0x04;
    pub const Dbp: u8 = 0x08;
}


#[derive(Clone, Debug)]
pub enum UiEvent {
    Cp(MouseEvent),
    Bki(KeyEvent),
    Cdv,
    Bct,
    Ckm(Size),
}






#[derive(Clone, Copy, Debug, Default)]
pub struct WidgetState {
    pub asy: bool,
    pub vn: bool,
    pub ja: bool,
    pub dqa: bool,
    pub iw: bool,
}

impl WidgetState {
    pub fn new() -> Self {
        Self {
            iw: true,
            ..Default::default()
        }
    }
}


pub trait Cf {
    
    fn ad(&self) -> u32;
    
    
    fn eg(&self) -> Rect;
    
    
    fn cbq(&mut self, eg: Rect);
    
    
    fn ctk(&self) -> Size {
        Size::new(100, 30)
    }
    
    
    fn zcu(&self) -> Size {
        Size::new(0, 0)
    }
    
    
    fn ate(&self) -> Size {
        Size::new(u32::O, u32::O)
    }
    
    
    fn g(&self) -> WidgetState;
    
    
    fn cbr(&mut self, g: WidgetState);
    
    
    fn ecj(&mut self, id: &UiEvent) -> bool {
        false 
    }
    
    
    fn tj(&self, surface: &mut GpuSurface, theme: &Theme);
    
    
    fn tpb(&self, nl: Point) -> bool {
        self.eg().contains(nl)
    }
}


static CHX_: Mutex<u32> = Mutex::new(1);

pub fn bvo() -> u32 {
    let mut ad = CHX_.lock();
    let result = *ad;
    *ad += 1;
    result
}






pub struct Dy {
    ad: u32,
    eg: Rect,
    g: WidgetState,
    pub text: String,
    pub s: Option<Color>,
    pub align: TextAlign,
}

#[derive(Clone, Copy, Debug, Default)]
pub enum TextAlign {
    #[default]
    Ap,
    Eo,
    Ca,
}

impl Dy {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            ad: bvo(),
            eg: Rect::Dh,
            g: WidgetState::new(),
            text: text.into(),
            s: None,
            align: TextAlign::Ap,
        }
    }
    
    pub fn zwa(mut self, s: Color) -> Self {
        self.s = Some(s);
        self
    }
    
    pub fn zvy(mut self, align: TextAlign) -> Self {
        self.align = align;
        self
    }
}

impl Cf for Dy {
    fn ad(&self) -> u32 { self.ad }
    fn eg(&self) -> Rect { self.eg }
    fn cbq(&mut self, eg: Rect) { self.eg = eg; }
    fn g(&self) -> WidgetState { self.g }
    fn cbr(&mut self, g: WidgetState) { self.g = g; }
    
    fn ctk(&self) -> Size {
        
        Size::new((self.text.len() as u32 * 8).am(10), 16)
    }
    
    fn tj(&self, surface: &mut GpuSurface, theme: &Theme) {
        if !self.g.iw { return; }
        
        let s = self.s.unwrap_or(theme.bui);
        
        
        let b = match self.align {
            TextAlign::Ap => self.eg.b,
            TextAlign::Eo => self.eg.b + (self.eg.z as i32 - self.text.len() as i32 * 8) / 2,
            TextAlign::Ca => self.eg.b + self.eg.z as i32 - self.text.len() as i32 * 8,
        };
        
        cb(surface, b, self.eg.c, &self.text, s.lv());
    }
}


pub struct Vs {
    ad: u32,
    eg: Rect,
    g: WidgetState,
    pub text: String,
    pub ctb: Option<Box<dyn Fn() + Send + Sync>>,
}

impl Vs {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            ad: bvo(),
            eg: Rect::Dh,
            g: WidgetState::new(),
            text: text.into(),
            ctb: None,
        }
    }
    
    pub fn ctb<G: Fn() + Send + Sync + 'static>(mut self, bb: G) -> Self {
        self.ctb = Some(Box::new(bb));
        self
    }
}

impl Cf for Vs {
    fn ad(&self) -> u32 { self.ad }
    fn eg(&self) -> Rect { self.eg }
    fn cbq(&mut self, eg: Rect) { self.eg = eg; }
    fn g(&self) -> WidgetState { self.g }
    fn cbr(&mut self, g: WidgetState) { self.g = g; }
    
    fn ctk(&self) -> Size {
        Size::new((self.text.len() as u32 * 8 + 24).am(80), 32)
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
        
        let ei = if self.g.dqa {
            theme.kfp
        } else if self.g.vn {
            theme.dzj
        } else if self.g.asy {
            theme.dor
        } else {
            theme.dop
        };
        
        let lp = if self.g.dqa {
            theme.iui
        } else {
            theme.bui
        };
        
        
        surface.afp(
            self.eg.b as u32,
            self.eg.c as u32,
            self.eg.z,
            self.eg.ac,
            theme.avh,
            ei.lv()
        );
        
        
        surface.mf(
            self.eg.b as u32,
            self.eg.c as u32,
            self.eg.z,
            self.eg.ac,
            theme.avh,
            if self.g.ja { theme.imc.lv() } else { theme.acu.lv() }
        );
        
        
        let wg = self.eg.b + (self.eg.z as i32 - self.text.len() as i32 * 8) / 2;
        let sl = self.eg.c + (self.eg.ac as i32 - 16) / 2;
        cb(surface, wg, sl, &self.text, lp.lv());
    }
}


pub struct Bue {
    ad: u32,
    eg: Rect,
    g: WidgetState,
    pub text: String,
    pub fqy: String,
    pub fgj: usize,
    pub olw: usize,
}

impl Bue {
    pub fn new() -> Self {
        Self {
            ad: bvo(),
            eg: Rect::Dh,
            g: WidgetState::new(),
            text: String::new(),
            fqy: String::new(),
            fgj: 0,
            olw: 256,
        }
    }
    
    pub fn zwg(mut self, fqy: impl Into<String>) -> Self {
        self.fqy = fqy.into();
        self
    }
}

impl Cf for Bue {
    fn ad(&self) -> u32 { self.ad }
    fn eg(&self) -> Rect { self.eg }
    fn cbq(&mut self, eg: Rect) { self.eg = eg; }
    fn g(&self) -> WidgetState { self.g }
    fn cbr(&mut self, g: WidgetState) { self.g = g; }
    
    fn ctk(&self) -> Size {
        Size::new(200, 32)
    }
    
    fn ecj(&mut self, id: &UiEvent) -> bool {
        match id {
            UiEvent::Bki(KeyEvent::Bzj { r }) if self.g.ja => {
                if self.text.len() < self.olw && !r.yzf() {
                    self.text.insert(self.fgj, *r);
                    self.fgj += 1;
                }
                true
            }
            UiEvent::Bki(KeyEvent::Fm { bs, .. }) if self.g.ja => {
                match *bs {
                    '\x08' => { 
                        if self.fgj > 0 {
                            self.fgj -= 1;
                            self.text.remove(self.fgj);
                        }
                        true
                    }
                    _ => false
                }
            }
            UiEvent::Cp(MouseEvent::Vy { .. }) => {
                self.g.ja = true;
                true
            }
            UiEvent::Bct => {
                self.g.ja = false;
                true
            }
            _ => false
        }
    }
    
    fn tj(&self, surface: &mut GpuSurface, theme: &Theme) {
        if !self.g.iw { return; }
        
        
        surface.afp(
            self.eg.b as u32,
            self.eg.c as u32,
            self.eg.z,
            self.eg.ac,
            theme.avh,
            theme.fdh.lv()
        );
        
        
        let aia = if self.g.ja {
            theme.imc
        } else if self.g.asy {
            theme.cof
        } else {
            theme.acu
        };
        
        surface.mf(
            self.eg.b as u32,
            self.eg.c as u32,
            self.eg.z,
            self.eg.ac,
            theme.avh,
            aia.lv()
        );
        
        
        let sl = self.eg.c + (self.eg.ac as i32 - 16) / 2;
        let wg = self.eg.b + theme.ob as i32;
        
        if self.text.is_empty() {
            cb(surface, wg, sl, &self.fqy, theme.iui.lv());
        } else {
            cb(surface, wg, sl, &self.text, theme.bui.lv());
        }
        
        
        if self.g.ja {
            let lf = wg + (self.fgj as i32 * 8);
            surface.ah(
                lf as u32,
                (sl + 2) as u32,
                2,
                12,
                theme.mm.lv()
            );
        }
    }
}


pub struct Bdk {
    ad: u32,
    eg: Rect,
    g: WidgetState,
    pub cpb: bool,
    pub cu: String,
    pub bks: Option<Box<dyn Fn(bool) + Send + Sync>>,
}

impl Bdk {
    pub fn new(cu: impl Into<String>) -> Self {
        Self {
            ad: bvo(),
            eg: Rect::Dh,
            g: WidgetState::new(),
            cpb: false,
            cu: cu.into(),
            bks: None,
        }
    }
    
    pub fn cpb(mut self, bn: bool) -> Self {
        self.cpb = bn;
        self
    }
    
    pub fn bks<G: Fn(bool) + Send + Sync + 'static>(mut self, bb: G) -> Self {
        self.bks = Some(Box::new(bb));
        self
    }
}

impl Cf for Bdk {
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
                self.cpb = !self.cpb;
                if let Some(ref bks) = self.bks {
                    bks(self.cpb);
                }
                true
            }
            _ => false
        }
    }
    
    fn tj(&self, surface: &mut GpuSurface, theme: &Theme) {
        if !self.g.iw { return; }
        
        
        let gbj = 20u32;
        let btm = self.eg.b as u32;
        let bjk = self.eg.c as u32 + (self.eg.ac - gbj) / 2;
        
        let ei = if self.cpb { theme.mm } else { theme.fdh };
        
        surface.afp(btm, bjk, gbj, gbj, 4, ei.lv());
        surface.mf(btm, bjk, gbj, gbj, 4, theme.acu.lv());
        
        
        if self.cpb {
            let cx = btm as i32 + 5;
            let ae = bjk as i32 + 10;
            surface.ahj(cx, ae, cx + 4, ae + 4, theme.gay.lv());
            surface.ahj(cx + 4, ae + 4, cx + 12, ae - 4, theme.gay.lv());
        }
        
        
        let wg = btm as i32 + gbj as i32 + 8;
        let sl = self.eg.c + (self.eg.ac as i32 - 16) / 2;
        cb(surface, wg, sl, &self.cu, theme.bui.lv());
    }
}


pub struct Bpl {
    ad: u32,
    eg: Rect,
    g: WidgetState,
    pub bn: f32,  
    pub pkn: bool,
}

impl Bpl {
    pub fn new(bn: f32) -> Self {
        Self {
            ad: bvo(),
            eg: Rect::Dh,
            g: WidgetState::new(),
            bn: bn.qp(0.0, 1.0),
            pkn: true,
        }
    }
    
    pub fn znt(&mut self, bn: f32) {
        self.bn = bn.qp(0.0, 1.0);
    }
}

impl Cf for Bpl {
    fn ad(&self) -> u32 { self.ad }
    fn eg(&self) -> Rect { self.eg }
    fn cbq(&mut self, eg: Rect) { self.eg = eg; }
    fn g(&self) -> WidgetState { self.g }
    fn cbr(&mut self, g: WidgetState) { self.g = g; }
    
    fn ctk(&self) -> Size {
        Size::new(200, 20)
    }
    
    fn tj(&self, surface: &mut GpuSurface, theme: &Theme) {
        if !self.g.iw { return; }
        
        let b = self.eg.b as u32;
        let c = self.eg.c as u32;
        let d = self.eg.z;
        let i = self.eg.ac;
        
        
        surface.afp(b, c, d, i, i / 2, theme.ems.lv());
        
        
        let eqh = ((d as f32 * self.bn) as u32).am(i);
        if self.bn > 0.0 {
            surface.afp(b, c, eqh, i, i / 2, theme.mm.lv());
        }
        
        
        if self.pkn {
            let egl = (self.bn * 100.0) as u32;
            let text = format!("{}%", egl);
            let wg = b as i32 + (d as i32 - text.len() as i32 * 8) / 2;
            let sl = c as i32 + (i as i32 - 16) / 2;
            cb(surface, wg, sl, &text, theme.bui.lv());
        }
    }
}






pub fn cb(surface: &mut GpuSurface, b: i32, c: i32, text: &str, s: u32) {
    let mut cx = b;
    for r in text.bw() {
        if cx >= 0 && (cx as u32) < surface.z && c >= 0 && (c as u32) < surface.ac {
            ahi(surface, cx, c, r, s);
        }
        cx += 8;
    }
}


fn ahi(surface: &mut GpuSurface, b: i32, c: i32, r: char, s: u32) {
    
    let ka = crate::framebuffer::font::ada(r);
    
    for br in 0..16 {
        let fs = ka[br];
        for bj in 0..8 {
            if (fs >> (7 - bj)) & 1 == 1 {
                let y = b + bj as i32;
                let x = c + br as i32;
                if y >= 0 && x >= 0 {
                    surface.aht(y as u32, x as u32, s);
                }
            }
        }
    }
}






#[derive(Clone, Copy, Debug, Default)]
pub enum FlexDirection {
    #[default]
    Aei,
    Aaq,
}


#[derive(Clone, Copy, Debug, Default)]
pub enum FlexAlign {
    #[default]
    Oe,
    Eo,
    Wm,
    Cnh,
    Cng,
    Cni,
}


pub struct Cdq {
    pub sz: FlexDirection,
    pub align: FlexAlign,
    pub ipq: FlexAlign,
    pub qi: u32,
    pub ob: EdgeInsets,
}

impl Cdq {
    pub fn br() -> Self {
        Self {
            sz: FlexDirection::Aei,
            align: FlexAlign::Oe,
            ipq: FlexAlign::Eo,
            qi: 8,
            ob: EdgeInsets::xx(8),
        }
    }
    
    pub fn column() -> Self {
        Self {
            sz: FlexDirection::Aaq,
            align: FlexAlign::Oe,
            ipq: FlexAlign::Oe,
            qi: 8,
            ob: EdgeInsets::xx(8),
        }
    }
    
    
    pub fn layout(&self, container: Rect, zf: &mut [&mut dyn Cf]) {
        if zf.is_empty() { return; }
        
        let yz = container.b + self.ob.fd as i32;
        let jae = container.c + self.ob.qc as i32;
        let aii = container.z.ao(self.ob.fd + self.ob.hw);
        let leq = container.ac.ao(self.ob.qc + self.ob.abm);
        
        
        let mut iek = 0u32;
        let mut jff = 0u32;
        
        for aeh in zf.iter() {
            let bwa = aeh.ctk();
            match self.sz {
                FlexDirection::Aei => {
                    iek += bwa.z;
                    jff = jff.am(bwa.ac);
                }
                FlexDirection::Aaq => {
                    iek += bwa.ac;
                    jff = jff.am(bwa.z);
                }
            }
        }
        iek += self.qi * (zf.len() as u32 - 1);
        
        
        let oku = match self.sz {
            FlexDirection::Aei => aii,
            FlexDirection::Aaq => leq,
        };
        
        let mut u = match self.align {
            FlexAlign::Oe => 0,
            FlexAlign::Eo => ((oku as i32 - iek as i32) / 2).am(0) as u32,
            FlexAlign::Wm => oku.ao(iek),
            _ => 0,
        };
        
        
        for aeh in zf.el() {
            let bwa = aeh.ctk();
            
            let (b, c, d, i) = match self.sz {
                FlexDirection::Aei => {
                    let kma = match self.ipq {
                        FlexAlign::Oe => 0,
                        FlexAlign::Eo => ((leq as i32 - bwa.ac as i32) / 2).am(0) as u32,
                        FlexAlign::Wm => leq.ao(bwa.ac),
                        _ => 0,
                    };
                    (yz + u as i32, jae + kma as i32, bwa.z, bwa.ac)
                }
                FlexDirection::Aaq => {
                    let kma = match self.ipq {
                        FlexAlign::Oe => 0,
                        FlexAlign::Eo => ((aii as i32 - bwa.z as i32) / 2).am(0) as u32,
                        FlexAlign::Wm => aii.ao(bwa.z),
                        _ => 0,
                    };
                    (yz + kma as i32, jae + u as i32, bwa.z, bwa.ac)
                }
            };
            
            aeh.cbq(Rect::new(b, c, d, i));
            
            u += match self.sz {
                FlexDirection::Aei => bwa.z + self.qi,
                FlexDirection::Aaq => bwa.ac + self.qi,
            };
        }
    }
}






pub fn init() {
    bxb(Theme::dark());
    crate::serial_println!("[UI] Toolkit initialized with dark theme");
}
