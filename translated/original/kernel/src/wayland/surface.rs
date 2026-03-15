//! Wayland Surface Management
//!
//! A wl_surface represents a rectangular area on the screen.
//! Clients attach buffers to surfaces and commit changes atomically.

use alloc::string::String;
use alloc::vec::Vec;

/// Damage region for partial updates
#[derive(Debug, Clone, Copy)]
pub struct DamageRect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

/// Surface buffer transform
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum BufferTransform {
    #[default]
    Normal = 0,
    Rotate90 = 1,
    Rotate180 = 2,
    Rotate270 = 3,
    Flipped = 4,
    FlippedRotate90 = 5,
    FlippedRotate180 = 6,
    FlippedRotate270 = 7,
}

/// A Wayland surface
#[derive(Debug, Clone)]
pub struct Surface {
    /// Unique surface ID
    pub id: u32,
    
    /// Position on screen
    pub x: i32,
    pub y: i32,
    
    /// Size in pixels
    pub width: u32,
    pub height: u32,
    
    /// Pixel buffer (ARGB8888)
    pub buffer: Vec<u32>,
    
    /// Pending buffer (not yet committed)
    pub pending_buffer: Option<Vec<u32>>,
    pub pending_width: u32,
    pub pending_height: u32,
    
    /// Buffer offset
    pub buffer_offset_x: i32,
    pub buffer_offset_y: i32,
    
    /// Buffer scale factor
    pub buffer_scale: i32,
    
    /// Buffer transform
    pub buffer_transform: BufferTransform,
    
    /// Damage regions for this frame
    pub damage: Vec<DamageRect>,
    
    /// Has the surface been committed at least once?
    pub committed: bool,
    
    /// Is the surface visible?
    pub visible: bool,
    
    /// Window title (for toplevels)
    pub title: String,
    
    /// Application ID
    pub app_id: String,
    
    /// Should compositor draw window decorations?
    pub has_decorations: bool,
    
    /// Is this a toplevel surface?
    pub is_toplevel: bool,
    
    /// Parent surface ID (for popups)
    pub parent: Option<u32>,
    
    /// Opaque region (for optimization)
    pub opaque_region: Option<DamageRect>,
    
    /// Input region (where input events are accepted)
    pub input_region: Option<DamageRect>,
    
    /// Frame callback pending
    pub frame_callback: Option<u32>,
    
    /// Surface state
    pub state: SurfaceState,
}

/// Surface window state (for xdg_toplevel)
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct SurfaceState {
    pub maximized: bool,
    pub fullscreen: bool,
    pub resizing: bool,
    pub activated: bool,
    pub tiled_left: bool,
    pub tiled_right: bool,
    pub tiled_top: bool,
    pub tiled_bottom: bool,
}

impl Surface {
    pub fn new(id: u32) -> Self {
        Self {
            id,
            x: 100,
            y: 100,
            width: 0,
            height: 0,
            buffer: Vec::new(),
            pending_buffer: None,
            pending_width: 0,
            pending_height: 0,
            buffer_offset_x: 0,
            buffer_offset_y: 0,
            buffer_scale: 1,
            buffer_transform: BufferTransform::Normal,
            damage: Vec::new(),
            committed: false,
            visible: true,
            title: String::new(),
            app_id: String::new(),
            has_decorations: true,
            is_toplevel: false,
            parent: None,
            opaque_region: None,
            input_region: None,
            frame_callback: None,
            state: SurfaceState::default(),
        }
    }
    
    /// Attach a buffer to the surface (pending until commit)
    pub fn attach(&mut self, buffer: Vec<u32>, width: u32, height: u32) {
        self.pending_buffer = Some(buffer);
        self.pending_width = width;
        self.pending_height = height;
    }
    
    /// Mark a region as damaged
    pub fn damage(&mut self, x: i32, y: i32, width: i32, height: i32) {
        self.damage.push(DamageRect { x, y, width, height });
    }
    
    /// Commit pending state
    pub fn commit(&mut self) {
        // Apply pending buffer
        if let Some(buffer) = self.pending_buffer.take() {
            self.buffer = buffer;
            self.width = self.pending_width;
            self.height = self.pending_height;
        }
        
        // Clear damage for next frame
        self.damage.clear();
        
        self.committed = true;
    }
    
    /// Set window title
    pub fn set_title(&mut self, title: &str) {
        self.title = String::from(title);
    }
    
    /// Set application ID
    pub fn set_app_id(&mut self, app_id: &str) {
        self.app_id = String::from(app_id);
    }
    
    /// Move surface to position
    pub fn set_position(&mut self, x: i32, y: i32) {
        self.x = x;
        self.y = y;
    }
    
    /// Check if point is inside surface (including decorations)
    pub fn contains(&self, px: i32, py: i32) -> bool {
        let title_height = if self.has_decorations { 28 } else { 0 };
        let x1 = self.x;
        let y1 = self.y - title_height;
        let x2 = self.x + self.width as i32;
        let y2 = self.y + self.height as i32;
        
        px >= x1 && px < x2 && py >= y1 && py < y2
    }
    
    /// Check if point is in title bar
    pub fn in_title_bar(&self, px: i32, py: i32) -> bool {
        if !self.has_decorations {
            return false;
        }
        let title_height = 28;
        px >= self.x 
            && px < self.x + self.width as i32 
            && py >= self.y - title_height 
            && py < self.y
    }
    
    /// Get content bounds (excluding decorations)
    pub fn content_bounds(&self) -> (i32, i32, u32, u32) {
        (self.x, self.y, self.width, self.height)
    }
    
    /// Make this a toplevel surface
    pub fn make_toplevel(&mut self) {
        self.is_toplevel = true;
        self.has_decorations = true;
        self.state.activated = true;
    }
    
    /// Maximize the surface
    pub fn maximize(&mut self, screen_width: u32, screen_height: u32) {
        self.state.maximized = true;
        self.x = 0;
        self.y = 28; // Below title bar
        // Would resize buffer in real impl
    }
    
    /// Restore from maximized state
    pub fn unmaximize(&mut self, prev_x: i32, prev_y: i32) {
        self.state.maximized = false;
        self.x = prev_x;
        self.y = prev_y;
    }
}

/// Factory for creating surfaces
pub struct SurfaceFactory {
    next_id: u32,
}

impl SurfaceFactory {
    pub fn new() -> Self {
        Self { next_id: 1 }
    }
    
    pub fn create(&mut self) -> Surface {
        let id = self.next_id;
        self.next_id += 1;
        Surface::new(id)
    }
}

impl Default for SurfaceFactory {
    fn default() -> Self {
        Self::new()
    }
}
