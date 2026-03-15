//! Wayland Display
//!
//! The wl_display is the core global object representing the connection.

use alloc::vec::Vec;
use alloc::string::String;
use super::protocol::{WlGlobal, WlMessage, get_globals};

/// Display connection state
pub struct Display {
    /// Registered globals
    pub globals: Vec<WlGlobal>,
    
    /// Pending events to send to client
    pub pending_events: Vec<WlMessage>,
    
    /// Last sync callback serial
    pub last_sync: u32,
}

impl Display {
    pub fn new() -> Self {
        Self {
            globals: get_globals(),
            pending_events: Vec::new(),
            last_sync: 0,
        }
    }
    
    /// Handle sync request (creates a callback)
    pub fn sync(&mut self, callback_id: u32) {
        // In real impl, we'd queue a done event for the callback
        self.last_sync = callback_id;
    }
    
    /// Get the list of globals for registry
    pub fn get_registry(&self) -> &[WlGlobal] {
        &self.globals
    }
    
    /// Queue an event to send
    pub fn queue_event(&mut self, event: WlMessage) {
        self.pending_events.push(event);
    }
    
    /// Flush pending events
    pub fn flush(&mut self) -> Vec<WlMessage> {
        core::mem::take(&mut self.pending_events)
    }
}

impl Default for Display {
    fn default() -> Self {
        Self::new()
    }
}

/// Output information (wl_output)
#[derive(Debug, Clone)]
pub struct Output {
    pub id: u32,
    pub name: String,
    pub make: String,
    pub model: String,
    pub x: i32,
    pub y: i32,
    pub physical_width: i32,  // mm
    pub physical_height: i32, // mm
    pub subpixel: Subpixel,
    pub transform: Transform,
    pub scale: i32,
    pub modes: Vec<OutputMode>,
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
pub struct OutputMode {
    pub width: i32,
    pub height: i32,
    pub refresh: i32, // mHz
    pub flags: u32,   // 1 = current, 2 = preferred
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
            physical_width: (width * 254 / 96) as i32, // Approximate mm at 96 DPI
            physical_height: (height * 254 / 96) as i32,
            subpixel: Subpixel::Unknown,
            transform: Transform::Normal,
            scale: 1,
            modes: alloc::vec![
                OutputMode {
                    width: width as i32,
                    height: height as i32,
                    refresh: 60000, // 60 Hz
                    flags: 3, // current + preferred
                }
            ],
            current_mode: 0,
        }
    }
    
    pub fn current_mode(&self) -> &OutputMode {
        &self.modes[self.current_mode]
    }
}
