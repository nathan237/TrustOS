//! Wayland Compositor Interface
//!
//! The wl_compositor global interface creates surfaces and regions.

use super::surface::Surface;
use super::new_object_id;
use alloc::vec::Vec;

/// Region - a set of rectangles for clipping
#[derive(Debug, Clone)]
pub struct Region {
    pub id: u32,
    pub rects: Vec<RegionRect>,
}

#[derive(Debug, Clone, Copy)]
pub struct RegionRect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub add: bool, // true = add, false = subtract
}

impl Region {
    pub fn new(id: u32) -> Self {
        Self {
            id,
            rects: Vec::new(),
        }
    }
    
    pub fn add(&mut self, x: i32, y: i32, width: i32, height: i32) {
        self.rects.push(RegionRect { x, y, width, height, add: true });
    }
    
    pub fn subtract(&mut self, x: i32, y: i32, width: i32, height: i32) {
        self.rects.push(RegionRect { x, y, width, height, add: false });
    }
    
    /// Check if a point is in the region
    pub fn contains(&self, px: i32, py: i32) -> bool {
        let mut inside = false;
        
        for rect in &self.rects {
            let in_rect = px >= rect.x 
                && px < rect.x + rect.width 
                && py >= rect.y 
                && py < rect.y + rect.height;
            
            if rect.add && in_rect {
                inside = true;
            } else if !rect.add && in_rect {
                inside = false;
            }
        }
        
        inside
    }
}

/// Compositor request handler
pub trait CompositorHandler {
    /// Create a new surface
    fn create_surface(&mut self) -> u32;
    
    /// Create a new region
    fn create_region(&mut self) -> u32;
}
