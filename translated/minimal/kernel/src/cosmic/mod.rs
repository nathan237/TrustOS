










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
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }
    
    pub fn contains(&self, p: f32, o: f32) -> bool {
        p >= self.x && p < self.x + self.width &&
        o >= self.y && o < self.y + self.height
    }
    
    pub fn center(&self) -> (f32, f32) {
        (self.x + self.width / 2.0, self.y + self.height / 2.0)
    }
}


#[derive(Clone, Copy, Debug, Default)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}


#[derive(Clone, Copy, Debug, Default)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

impl Size {
    pub const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
}


#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }
    
    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }
    
    pub fn lzp(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a: a as f32 / 255.0,
        }
    }
    
    pub fn from_u32(abq: u32) -> Self {
        Self::lzp(
            ((abq >> 16) & 0xFF) as u8,
            ((abq >> 8) & 0xFF) as u8,
            (abq & 0xFF) as u8,
            ((abq >> 24) & 0xFF) as u8,
        )
    }
    
    pub fn to_u32(&self) -> u32 {
        let a = (self.a * 255.0) as u32;
        let r = (self.r * 255.0) as u32;
        let g = (self.g * 255.0) as u32;
        let b = (self.b * 255.0) as u32;
        (a << 24) | (r << 16) | (g << 8) | b
    }
    
    pub fn with_alpha(self, a: f32) -> Self {
        Self { a, ..self }
    }
    
    pub fn lighten(self, adg: f32) -> Self {
        Self {
            r: (self.r + adg).min(1.0),
            g: (self.g + adg).min(1.0),
            b: (self.b + adg).min(1.0),
            a: self.a,
        }
    }
    
    pub fn darken(self, adg: f32) -> Self {
        Self {
            r: (self.r - adg).max(0.0),
            g: (self.g - adg).max(0.0),
            b: (self.b - adg).max(0.0),
            a: self.a,
        }
    }
    
    
    pub fn blend(self, other: Color, t: f32) -> Self {
        Self {
            r: self.r + (other.r - self.r) * t,
            g: self.g + (other.g - self.g) * t,
            b: self.b + (other.b - self.b) * t,
            a: self.a + (other.a - self.a) * t,
        }
    }
    
    
    pub const TRANSPARENT: Color = Color::new(0.0, 0.0, 0.0, 0.0);
    pub const BLACK: Color = Color::rgb(0.0, 0.0, 0.0);
    pub const WHITE: Color = Color::rgb(1.0, 1.0, 1.0);
}






#[derive(Clone, Debug)]
pub enum Event {
    Mouse(MouseEvent),
    Keyboard(Amc),
    Window(Asa),
}

#[derive(Clone, Debug)]
pub enum MouseEvent {
    Move { x: f32, y: f32 },
    Press { x: f32, y: f32, button: MouseButton },
    Release { x: f32, y: f32, button: MouseButton },
    Scroll { delta_x: f32, delta_y: f32 },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

#[derive(Clone, Debug)]
pub enum Amc {
    Press { key: u8, modifiers: Abl },
    Release { key: u8, modifiers: Abl },
    Character { c: char },
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Abl {
    pub ctrl: bool,
    pub adf: bool,
    pub no: bool,
    pub super_key: bool,
}

#[derive(Clone, Debug)]
pub enum Asa {
    Resize { width: u32, height: u32 },
    Focus,
    Unfocus,
    Close,
}






pub type Az = u32;


pub trait Aw {
    
    fn size(&self) -> Size;
    
    
    fn draw(&self, renderer: &mut CosmicRenderer, bounds: Rect);
    
    
    fn on_event(&mut self, event: &Event, bounds: Rect) -> Option<Az>;
    
    
    fn children(&self) -> &[Box<dyn Aw>] {
        &[]
    }
}
