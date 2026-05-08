



use alloc::vec::Vec;
use alloc::string::String;
use super::protocol::{Gd, Ny, fys};


pub struct Display {
    
    pub globals: Vec<Gd>,
    
    
    pub pending_events: Vec<Ny>,
    
    
    pub last_sync: u32,
}

impl Display {
    pub fn new() -> Self {
        Self {
            globals: fys(),
            pending_events: Vec::new(),
            last_sync: 0,
        }
    }
    
    
    pub fn sync(&mut self, callback_id: u32) {
        
        self.last_sync = callback_id;
    }
    
    
    pub fn qih(&self) -> &[Gd] {
        &self.globals
    }
    
    
    pub fn qrp(&mut self, event: Ny) {
        self.pending_events.push(event);
    }
    
    
    pub fn flush(&mut self) -> Vec<Ny> {
        core::mem::take(&mut self.pending_events)
    }
}

impl Default for Display {
    fn default() -> Self {
        Self::new()
    }
}


#[derive(Debug, Clone)]
pub struct Output {
    pub id: u32,
    pub name: String,
    pub make: String,
    pub model: String,
    pub x: i32,
    pub y: i32,
    pub physical_width: i32,  
    pub physical_height: i32, 
    pub subpixel: Subpixel,
    pub transform: Transform,
    pub scale: i32,
    pub modes: Vec<Ue>,
    pub current_mode: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Subpixel {
    Unknown = 0,
    None = 1,
    HorizontalRgb = 2,
    HorizontalBgr = 3,
    VerticalRgb = 4,
    VerticalBgr = 5,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Transform {
    Normal = 0,
    Rotate90 = 1,
    Rotate180 = 2,
    Rotate270 = 3,
    Flipped = 4,
    FlippedRotate90 = 5,
    FlippedRotate180 = 6,
    FlippedRotate270 = 7,
}

#[derive(Debug, Clone)]
pub struct Ue {
    pub width: i32,
    pub height: i32,
    pub refresh: i32, 
    pub flags: u32,   
}

impl Output {
    pub fn new(id: u32, width: u32, height: u32) -> Self {
        Self {
            id,
            name: String::from("TrustOS-1"),
            make: String::from("TrustOS"),
            model: String::from("Virtual Display"),
            x: 0,
            y: 0,
            physical_width: (width * 254 / 96) as i32, 
            physical_height: (height * 254 / 96) as i32,
            subpixel: Subpixel::Unknown,
            transform: Transform::Normal,
            scale: 1,
            modes: alloc::vec![
                Ue {
                    width: width as i32,
                    height: height as i32,
                    refresh: 60000, 
                    flags: 3, 
                }
            ],
            current_mode: 0,
        }
    }
    
    pub fn current_mode(&self) -> &Ue {
        &self.modes[self.current_mode]
    }
}
