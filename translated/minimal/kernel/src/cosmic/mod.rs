










pub mod theme;
pub mod widgets;
pub mod renderer;
pub mod layout;

pub use theme::*;
pub use widgets::*;
pub use renderer::*;
pub use layout::*;

use alloc::boxed::Box;
use alloc::vec::Vec;
use alloc::string::String;


pub fn init() {
    crate::serial_println!("[COSMIC] UI Framework initialized (tiny-skia backend)");
}






#[derive(Clone, Copy, Debug, Default)]
pub struct Rect {
    pub b: f32,
    pub c: f32,
    pub z: f32,
    pub ac: f32,
}

impl Rect {
    pub const fn new(b: f32, c: f32, z: f32, ac: f32) -> Self {
        Self { b, c, z, ac }
    }
    
    pub fn contains(&self, y: f32, x: f32) -> bool {
        y >= self.b && y < self.b + self.z &&
        x >= self.c && x < self.c + self.ac
    }
    
    pub fn pn(&self) -> (f32, f32) {
        (self.b + self.z / 2.0, self.c + self.ac / 2.0)
    }
}


#[derive(Clone, Copy, Debug, Default)]
pub struct Point {
    pub b: f32,
    pub c: f32,
}

impl Point {
    pub const fn new(b: f32, c: f32) -> Self {
        Self { b, c }
    }
}


#[derive(Clone, Copy, Debug, Default)]
pub struct Size {
    pub z: f32,
    pub ac: f32,
}

impl Size {
    pub const fn new(z: f32, ac: f32) -> Self {
        Self { z, ac }
    }
}


#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color {
    pub m: f32,
    pub at: f32,
    pub o: f32,
    pub q: f32,
}

impl Color {
    pub const fn new(m: f32, at: f32, o: f32, q: f32) -> Self {
        Self { m, at, o, q }
    }
    
    pub const fn xt(m: f32, at: f32, o: f32) -> Self {
        Self { m, at, o, q: 1.0 }
    }
    
    pub fn syg(m: u8, at: u8, o: u8, q: u8) -> Self {
        Self {
            m: m as f32 / 255.0,
            at: at as f32 / 255.0,
            o: o as f32 / 255.0,
            q: q as f32 / 255.0,
        }
    }
    
    pub fn zi(bax: u32) -> Self {
        Self::syg(
            ((bax >> 16) & 0xFF) as u8,
            ((bax >> 8) & 0xFF) as u8,
            (bax & 0xFF) as u8,
            ((bax >> 24) & 0xFF) as u8,
        )
    }
    
    pub fn lv(&self) -> u32 {
        let q = (self.q * 255.0) as u32;
        let m = (self.m * 255.0) as u32;
        let at = (self.at * 255.0) as u32;
        let o = (self.o * 255.0) as u32;
        (q << 24) | (m << 16) | (at << 8) | o
    }
    
    pub fn fbo(self, q: f32) -> Self {
        Self { q, ..self }
    }
    
    pub fn clh(self, bdk: f32) -> Self {
        Self {
            m: (self.m + bdk).v(1.0),
            at: (self.at + bdk).v(1.0),
            o: (self.o + bdk).v(1.0),
            q: self.q,
        }
    }
    
    pub fn cdz(self, bdk: f32) -> Self {
        Self {
            m: (self.m - bdk).am(0.0),
            at: (self.at - bdk).am(0.0),
            o: (self.o - bdk).am(0.0),
            q: self.q,
        }
    }
    
    
    pub fn btk(self, gq: Color, ab: f32) -> Self {
        Self {
            m: self.m + (gq.m - self.m) * ab,
            at: self.at + (gq.at - self.at) * ab,
            o: self.o + (gq.o - self.o) * ab,
            q: self.q + (gq.q - self.q) * ab,
        }
    }
    
    
    pub const Anl: Color = Color::new(0.0, 0.0, 0.0, 0.0);
    pub const Ox: Color = Color::xt(0.0, 0.0, 0.0);
    pub const Zm: Color = Color::xt(1.0, 1.0, 1.0);
}






#[derive(Clone, Debug)]
pub enum Event {
    Cp(MouseEvent),
    Hs(Cgi),
    Window(Cql),
}

#[derive(Clone, Debug)]
pub enum MouseEvent {
    Fw { b: f32, c: f32 },
    Axb { b: f32, c: f32, bdp: MouseButton },
    Release { b: f32, c: f32, bdp: MouseButton },
    Yq { iqw: f32, iqx: f32 },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MouseButton {
    Ap,
    Ca,
    Chk,
}

#[derive(Clone, Debug)]
pub enum Cgi {
    Axb { bs: u8, modifiers: Bmn },
    Release { bs: u8, modifiers: Bmn },
    Kh { r: char },
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Bmn {
    pub db: bool,
    pub bdj: bool,
    pub acn: bool,
    pub zpx: bool,
}

#[derive(Clone, Debug)]
pub enum Cql {
    Ckm { z: u32, ac: u32 },
    Cdv,
    Dkr,
    Mx,
}






pub type Cj = u32;


pub trait Cf {
    
    fn aw(&self) -> Size;
    
    
    fn po(&self, renderer: &mut CosmicRenderer, eg: Rect);
    
    
    fn goi(&mut self, id: &Event, eg: Rect) -> Option<Cj>;
    
    
    fn zf(&self) -> &[Box<dyn Cf>] {
        &[]
    }
}
