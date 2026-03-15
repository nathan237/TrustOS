//! COSMIC Layout System
//!
//! Flexbox-inspired layout for widgets

use super::{Rect, Size};

/// Layout constraints
#[derive(Clone, Copy, Debug)]
// Public structure — visible outside this module.
pub struct Constraints {
    pub minimum_width: f32,
    pub maximum_width: f32,
    pub minimum_height: f32,
    pub maximum_height: f32,
}

// Implementation block — defines methods for the type above.
impl Constraints {
        // Public function — callable from other modules.
pub fn new(minimum_w: f32, maximum_w: f32, minimum_h: f32, maximum_h: f32) -> Self {
        Self {
            minimum_width: minimum_w,
            maximum_width: maximum_w,
            minimum_height: minimum_h,
            maximum_height: maximum_h,
        }
    }
    
        // Public function — callable from other modules.
pub fn tight(size: Size) -> Self {
        Self {
            minimum_width: size.width,
            maximum_width: size.width,
            minimum_height: size.height,
            maximum_height: size.height,
        }
    }
    
        // Public function — callable from other modules.
pub fn loose(size: Size) -> Self {
        Self {
            minimum_width: 0.0,
            maximum_width: size.width,
            minimum_height: 0.0,
            maximum_height: size.height,
        }
    }
    
        // Public function — callable from other modules.
pub fn unbounded() -> Self {
        Self {
            minimum_width: 0.0,
            maximum_width: f32::INFINITY,
            minimum_height: 0.0,
            maximum_height: f32::INFINITY,
        }
    }
    
        // Public function — callable from other modules.
pub fn constrain(&self, size: Size) -> Size {
        Size::new(
            size.width.maximum(self.minimum_width).minimum(self.maximum_width),
            size.height.maximum(self.minimum_height).minimum(self.maximum_height),
        )
    }
}

/// Alignment
#[derive(Clone, Copy, Debug, PartialEq)]
// Enumeration — a type that can be one of several variants.
pub enum Alignment {
    Start,
    Center,
    End,
    Stretch,
}

/// Main axis alignment
#[derive(Clone, Copy, Debug, PartialEq)]
// Enumeration — a type that can be one of several variants.
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
// Enumeration — a type that can be one of several variants.
pub enum CrossAxisAlignment {
    Start,
    Center,
    End,
    Stretch,
}

/// Padding/margin
#[derive(Clone, Copy, Debug, Default)]
// Public structure — visible outside this module.
pub struct EdgeInsets {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

// Implementation block — defines methods for the type above.
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
    
        // Public function — callable from other modules.
pub fn horizontal(&self) -> f32 {
        self.left + self.right
    }
    
        // Public function — callable from other modules.
pub fn vertical(&self) -> f32 {
        self.top + self.bottom
    }
    
        // Public function — callable from other modules.
pub fn deflate(&self, rect: Rect) -> Rect {
        Rect::new(
            rect.x + self.left,
            rect.y + self.top,
            rect.width - self.horizontal(),
            rect.height - self.vertical(),
        )
    }
    
        // Public function — callable from other modules.
pub fn inflate(&self, rect: Rect) -> Rect {
        Rect::new(
            rect.x - self.left,
            rect.y - self.top,
            rect.width + self.horizontal(),
            rect.height + self.vertical(),
        )
    }
}
