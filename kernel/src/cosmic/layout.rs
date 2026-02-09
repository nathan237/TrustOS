//! COSMIC Layout System
//!
//! Flexbox-inspired layout for widgets

use super::{Rect, Size};

/// Layout constraints
#[derive(Clone, Copy, Debug)]
pub struct Constraints {
    pub min_width: f32,
    pub max_width: f32,
    pub min_height: f32,
    pub max_height: f32,
}

impl Constraints {
    pub fn new(min_w: f32, max_w: f32, min_h: f32, max_h: f32) -> Self {
        Self {
            min_width: min_w,
            max_width: max_w,
            min_height: min_h,
            max_height: max_h,
        }
    }
    
    pub fn tight(size: Size) -> Self {
        Self {
            min_width: size.width,
            max_width: size.width,
            min_height: size.height,
            max_height: size.height,
        }
    }
    
    pub fn loose(size: Size) -> Self {
        Self {
            min_width: 0.0,
            max_width: size.width,
            min_height: 0.0,
            max_height: size.height,
        }
    }
    
    pub fn unbounded() -> Self {
        Self {
            min_width: 0.0,
            max_width: f32::INFINITY,
            min_height: 0.0,
            max_height: f32::INFINITY,
        }
    }
    
    pub fn constrain(&self, size: Size) -> Size {
        Size::new(
            size.width.max(self.min_width).min(self.max_width),
            size.height.max(self.min_height).min(self.max_height),
        )
    }
}

/// Alignment
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Alignment {
    Start,
    Center,
    End,
    Stretch,
}

/// Main axis alignment
#[derive(Clone, Copy, Debug, PartialEq)]
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
pub enum CrossAxisAlignment {
    Start,
    Center,
    End,
    Stretch,
}

/// Padding/margin
#[derive(Clone, Copy, Debug, Default)]
pub struct EdgeInsets {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

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
    
    pub fn horizontal(&self) -> f32 {
        self.left + self.right
    }
    
    pub fn vertical(&self) -> f32 {
        self.top + self.bottom
    }
    
    pub fn deflate(&self, rect: Rect) -> Rect {
        Rect::new(
            rect.x + self.left,
            rect.y + self.top,
            rect.width - self.horizontal(),
            rect.height - self.vertical(),
        )
    }
    
    pub fn inflate(&self, rect: Rect) -> Rect {
        Rect::new(
            rect.x - self.left,
            rect.y - self.top,
            rect.width + self.horizontal(),
            rect.height + self.vertical(),
        )
    }
}
