//! COSMIC-style UI Framework for TrustOS
//!
//! A `no_std` implementation inspired by System76's libcosmic.
//! Uses tiny-skia for software rendering directly to framebuffer.
//!
//! Features:
//! - Modern widgets (Button, Label, Container, etc.)
//! - COSMIC color palette (Pop!_OS style)
//! - Rounded corners, shadows, blur effects
//! - Event-driven architecture

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

/// Initialize the COSMIC UI system
pub fn init() {
    crate::serial_println!("[COSMIC] UI Framework initialized (tiny-skia backend)");
}

// ═══════════════════════════════════════════════════════════════════════════════
// CORE TYPES
// ═══════════════════════════════════════════════════════════════════════════════

/// Rectangle bounds
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
    
    pub fn contains(&self, px: f32, py: f32) -> bool {
        px >= self.x && px < self.x + self.width &&
        py >= self.y && py < self.y + self.height
    }
    
    pub fn center(&self) -> (f32, f32) {
        (self.x + self.width / 2.0, self.y + self.height / 2.0)
    }
}

/// Point
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

/// Size
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

/// Color in ARGB format
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
    
    pub fn from_rgba8(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a: a as f32 / 255.0,
        }
    }
    
    pub fn from_u32(argb: u32) -> Self {
        Self::from_rgba8(
            ((argb >> 16) & 0xFF) as u8,
            ((argb >> 8) & 0xFF) as u8,
            (argb & 0xFF) as u8,
            ((argb >> 24) & 0xFF) as u8,
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
    
    pub fn lighten(self, amount: f32) -> Self {
        Self {
            r: (self.r + amount).min(1.0),
            g: (self.g + amount).min(1.0),
            b: (self.b + amount).min(1.0),
            a: self.a,
        }
    }
    
    pub fn darken(self, amount: f32) -> Self {
        Self {
            r: (self.r - amount).max(0.0),
            g: (self.g - amount).max(0.0),
            b: (self.b - amount).max(0.0),
            a: self.a,
        }
    }
    
    /// Blend with another color
    pub fn blend(self, other: Color, t: f32) -> Self {
        Self {
            r: self.r + (other.r - self.r) * t,
            g: self.g + (other.g - self.g) * t,
            b: self.b + (other.b - self.b) * t,
            a: self.a + (other.a - self.a) * t,
        }
    }
    
    // Predefined colors
    pub const TRANSPARENT: Color = Color::new(0.0, 0.0, 0.0, 0.0);
    pub const BLACK: Color = Color::rgb(0.0, 0.0, 0.0);
    pub const WHITE: Color = Color::rgb(1.0, 1.0, 1.0);
}

// ═══════════════════════════════════════════════════════════════════════════════
// MESSAGE/EVENT SYSTEM (inspired by Elm/iced)
// ═══════════════════════════════════════════════════════════════════════════════

/// UI Events
#[derive(Clone, Debug)]
pub enum Event {
    Mouse(MouseEvent),
    Keyboard(KeyboardEvent),
    Window(WindowEvent),
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
pub enum KeyboardEvent {
    Press { key: u8, modifiers: Modifiers },
    Release { key: u8, modifiers: Modifiers },
    Character { c: char },
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Modifiers {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub super_key: bool,
}

#[derive(Clone, Debug)]
pub enum WindowEvent {
    Resize { width: u32, height: u32 },
    Focus,
    Unfocus,
    Close,
}

// ═══════════════════════════════════════════════════════════════════════════════
// WIDGET TRAIT (core abstraction)
// ═══════════════════════════════════════════════════════════════════════════════

/// Message type for widget callbacks
pub type Message = u32;

/// Widget trait - all UI elements implement this
pub trait Widget {
    /// Calculate the natural size of this widget
    fn size(&self) -> Size;
    
    /// Render the widget to the given renderer
    fn draw(&self, renderer: &mut CosmicRenderer, bounds: Rect);
    
    /// Handle an event, optionally returning a message
    fn on_event(&mut self, event: &Event, bounds: Rect) -> Option<Message>;
    
    /// Get child widgets (for containers)
    fn children(&self) -> &[Box<dyn Widget>] {
        &[]
    }
}
