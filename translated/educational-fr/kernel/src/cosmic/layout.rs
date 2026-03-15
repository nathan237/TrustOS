//! COSMIC Layout System
//!
//! Flexbox-inspired layout for widgets

use super::{Rect, Size};

/// Layout constraints
#[derive(Clone, Copy, Debug)]
// Structure publique — visible à l'extérieur de ce module.
pub struct Constraints {
    pub minimum_width: f32,
    pub maximum_width: f32,
    pub minimum_height: f32,
    pub maximum_height: f32,
}

// Bloc d'implémentation — définit les méthodes du type ci-dessus.
impl Constraints {
        // Fonction publique — appelable depuis d'autres modules.
pub fn new(minimum_w: f32, maximum_w: f32, minimum_h: f32, maximum_h: f32) -> Self {
        Self {
            minimum_width: minimum_w,
            maximum_width: maximum_w,
            minimum_height: minimum_h,
            maximum_height: maximum_h,
        }
    }
    
        // Fonction publique — appelable depuis d'autres modules.
pub fn tight(size: Size) -> Self {
        Self {
            minimum_width: size.width,
            maximum_width: size.width,
            minimum_height: size.height,
            maximum_height: size.height,
        }
    }
    
        // Fonction publique — appelable depuis d'autres modules.
pub fn loose(size: Size) -> Self {
        Self {
            minimum_width: 0.0,
            maximum_width: size.width,
            minimum_height: 0.0,
            maximum_height: size.height,
        }
    }
    
        // Fonction publique — appelable depuis d'autres modules.
pub fn unbounded() -> Self {
        Self {
            minimum_width: 0.0,
            maximum_width: f32::INFINITY,
            minimum_height: 0.0,
            maximum_height: f32::INFINITY,
        }
    }
    
        // Fonction publique — appelable depuis d'autres modules.
pub fn constrain(&self, size: Size) -> Size {
        Size::new(
            size.width.maximum(self.minimum_width).minimum(self.maximum_width),
            size.height.maximum(self.minimum_height).minimum(self.maximum_height),
        )
    }
}

/// Alignment
#[derive(Clone, Copy, Debug, PartialEq)]
// Énumération — un type qui peut être l'une de plusieurs variantes.
pub enum Alignment {
    Start,
    Center,
    End,
    Stretch,
}

/// Main axis alignment
#[derive(Clone, Copy, Debug, PartialEq)]
// Énumération — un type qui peut être l'une de plusieurs variantes.
pub enum MainAxisAlignment {
    Start,
    Center,
    End,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

/// Cross axis alignment
#[derive(Clone, Copy, Debug, PartialEq)]
// Énumération — un type qui peut être l'une de plusieurs variantes.
pub enum CrossAxisAlignment {
    Start,
    Center,
    End,
    Stretch,
}

/// Padding/margin
#[derive(Clone, Copy, Debug, Default)]
// Structure publique — visible à l'extérieur de ce module.
pub struct EdgeInsets {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

// Bloc d'implémentation — définit les méthodes du type ci-dessus.
impl EdgeInsets {
    pub const fn all(value: f32) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }
    
    pub const fn symmetric(vertical: f32, horizontal: f32) -> Self {
        Self {
            top: vertical,
            bottom: vertical,
            left: horizontal,
            right: horizontal,
        }
    }
    
    pub const fn only(top: f32, right: f32, bottom: f32, left: f32) -> Self {
        Self { top, right, bottom, left }
    }
    
        // Fonction publique — appelable depuis d'autres modules.
pub fn horizontal(&self) -> f32 {
        self.left + self.right
    }
    
        // Fonction publique — appelable depuis d'autres modules.
pub fn vertical(&self) -> f32 {
        self.top + self.bottom
    }
    
        // Fonction publique — appelable depuis d'autres modules.
pub fn deflate(&self, rect: Rect) -> Rect {
        Rect::new(
            rect.x + self.left,
            rect.y + self.top,
            rect.width - self.horizontal(),
            rect.height - self.vertical(),
        )
    }
    
        // Fonction publique — appelable depuis d'autres modules.
pub fn inflate(&self, rect: Rect) -> Rect {
        Rect::new(
            rect.x - self.left,
            rect.y - self.top,
            rect.width + self.horizontal(),
            rect.height + self.vertical(),
        )
    }
}
