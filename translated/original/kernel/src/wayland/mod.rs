//! TrustOS Wayland Compositor
//!
//! A native Wayland compositor implementation for TrustOS.
//! This provides a modern display server protocol for GUI applications.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    Wayland Clients                          │
//! │   (Native TrustOS apps, Linux apps via subsystem)          │
//! ├─────────────────────────────────────────────────────────────┤
//! │                  Wayland Protocol Layer                     │
//! │   wl_compositor │ wl_surface │ wl_shm │ wl_seat            │
//! ├─────────────────────────────────────────────────────────────┤
//! │                  TrustOS Compositor                         │
//! │   Surface management, damage tracking, compositing         │
//! ├─────────────────────────────────────────────────────────────┤
//! │                  Framebuffer / VirtIO-GPU                   │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Implemented Protocols
//! - `wl_display` - Core display connection
//! - `wl_compositor` - Surface creation
//! - `wl_surface` - Window surfaces
//! - `wl_shm` - Shared memory buffers
//! - `wl_seat` - Input devices (keyboard, pointer)

pub mod protocol;
pub mod compositor;
pub mod surface;
pub mod shm;
pub mod seat;
pub mod display;
pub mod terminal;

use alloc::vec::Vec;
use alloc::string::String;
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use spin::Mutex;
use core::sync::atomic::{AtomicU32, Ordering};

pub use protocol::*;
pub use compositor::*;
pub use surface::*;
pub use shm::*;
pub use seat::*;
pub use display::*;

// ═══════════════════════════════════════════════════════════════════════════════
// GLOBAL COMPOSITOR STATE
// ═══════════════════════════════════════════════════════════════════════════════

/// Global Wayland compositor instance
static COMPOSITOR: Mutex<Option<WaylandCompositor>> = Mutex::new(None);

/// Next object ID for Wayland protocol
static NEXT_OBJECT_ID: AtomicU32 = AtomicU32::new(1);

/// Generate a new unique object ID
pub fn new_object_id() -> u32 {
    NEXT_OBJECT_ID.fetch_add(1, Ordering::SeqCst)
}

/// Initialize the Wayland compositor
pub fn init() -> Result<(), &'static str> {
    let mut compositor = COMPOSITOR.lock();
    if compositor.is_some() {
        return Err("Wayland compositor already initialized");
    }
    
    let (width, height) = crate::framebuffer::get_dimensions();
    
    let wc = WaylandCompositor::new(width, height);
    *compositor = Some(wc);
    
    crate::serial_println!("[WAYLAND] Compositor initialized ({}x{})", width, height);
    Ok(())
}

/// Get a reference to the compositor
pub fn with_compositor<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&mut WaylandCompositor) -> R,
{
    let mut guard = COMPOSITOR.lock();
    guard.as_mut().map(f)
}

/// Main compositor loop - compose all surfaces and render to framebuffer
pub fn compose_frame() {
    with_compositor(|compositor| {
        compositor.compose();
    });
}

/// Process input events and dispatch to focused client
pub fn process_input() {
    with_compositor(|compositor| {
        compositor.process_input();
    });
}

// ═══════════════════════════════════════════════════════════════════════════════
// WAYLAND COMPOSITOR CORE
// ═══════════════════════════════════════════════════════════════════════════════

/// The main Wayland compositor
pub struct WaylandCompositor {
    /// Screen dimensions
    pub width: u32,
    pub height: u32,
    
    /// All surfaces managed by the compositor
    pub surfaces: BTreeMap<u32, Surface>,
    
    /// Surface stacking order (bottom to top)
    pub surface_order: Vec<u32>,
    
    /// Currently focused surface
    pub focused_surface: Option<u32>,
    
    /// Pointer position
    pub pointer_x: i32,
    pub pointer_y: i32,
    
    /// Shared memory pools
    pub shm_pools: BTreeMap<u32, ShmPool>,
    
    /// Connected clients
    pub clients: BTreeMap<u32, WaylandClient>,
    
    /// Frame number for sync
    pub frame_number: u64,
    
    /// Background color (ARGB)
    pub background_color: u32,
}

impl WaylandCompositor {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            surfaces: BTreeMap::new(),
            surface_order: Vec::new(),
            focused_surface: None,
            pointer_x: (width / 2) as i32,
            pointer_y: (height / 2) as i32,
            shm_pools: BTreeMap::new(),
            clients: BTreeMap::new(),
            frame_number: 0,
            background_color: 0xFF0A0F0C, // Dark green-tinted background
        }
    }
    
    /// Create a new surface
    pub fn create_surface(&mut self) -> u32 {
        let id = new_object_id();
        let surface = Surface::new(id);
        self.surfaces.insert(id, surface);
        self.surface_order.push(id);
        crate::serial_println!("[WAYLAND] Created surface {}", id);
        id
    }
    
    /// Destroy a surface
    pub fn destroy_surface(&mut self, id: u32) {
        self.surfaces.remove(&id);
        self.surface_order.retain(|&x| x != id);
        if self.focused_surface == Some(id) {
            self.focused_surface = self.surface_order.last().copied();
        }
        crate::serial_println!("[WAYLAND] Destroyed surface {}", id);
    }
    
    /// Bring a surface to the top
    pub fn raise_surface(&mut self, id: u32) {
        self.surface_order.retain(|&x| x != id);
        self.surface_order.push(id);
        self.focused_surface = Some(id);
    }
    
    /// Create a shared memory pool
    pub fn create_shm_pool(&mut self, size: usize) -> u32 {
        let id = new_object_id();
        let pool = ShmPool::new(id, size);
        self.shm_pools.insert(id, pool);
        crate::serial_println!("[WAYLAND] Created SHM pool {} ({} bytes)", id, size);
        id
    }
    
    /// Compose all surfaces to the framebuffer
    pub fn compose(&mut self) {
        self.frame_number += 1;
        
        // Clear with background color
        let (width, height) = crate::framebuffer::get_dimensions();
        
        // Draw background gradient
        self.draw_background(width, height);
        
        // Draw surfaces in order (bottom to top)
        for &surface_id in &self.surface_order {
            if let Some(surface) = self.surfaces.get(&surface_id) {
                if surface.visible && surface.committed {
                    self.draw_surface(surface);
                }
            }
        }
        
        // Draw cursor on top
        self.draw_cursor();
    }
    
    fn draw_background(&self, width: u32, height: u32) {
        // Simple dark background with subtle pattern
        for y in 0..height {
            for x in 0..width {
                let pattern: u32 = if (x + y) % 32 < 16 { 0x00 } else { 0x02 };
                let _color: u32 = 0xFF000000u32 | (pattern << 16) | ((pattern + 0x08) << 8) | pattern;
                crate::framebuffer::put_pixel(x, y, self.background_color);
            }
        }
    }
    
    fn draw_surface(&self, surface: &Surface) {
        if surface.buffer.is_empty() {
            return;
        }
        
        let x = surface.x;
        let y = surface.y;
        let w = surface.width;
        let h = surface.height;
        
        // Draw window decorations if this is a toplevel
        if surface.has_decorations {
            self.draw_decorations(surface);
        }
        
        // Draw surface content
        for sy in 0..h {
            for sx in 0..w {
                let idx = (sy * w + sx) as usize;
                if idx < surface.buffer.len() {
                    let pixel = surface.buffer[idx];
                    let px = x + sx as i32;
                    let py = y + sy as i32;
                    if px >= 0 && py >= 0 {
                        crate::framebuffer::put_pixel(px as u32, py as u32, pixel);
                    }
                }
            }
        }
    }
    
    fn draw_decorations(&self, surface: &Surface) {
        let x = surface.x;
        let y = surface.y;
        let w = surface.width as i32;
        let title_height = 28;
        
        // Title bar background
        let title_color = if self.focused_surface == Some(surface.id) {
            0xFF1A2A20 // Focused - brighter
        } else {
            0xFF0D1310 // Unfocused - darker
        };
        
        for ty in 0..title_height {
            for tx in 0..w {
                let px = x + tx;
                let py = y - title_height + ty;
                if px >= 0 && py >= 0 {
                    crate::framebuffer::put_pixel(px as u32, py as u32, title_color);
                }
            }
        }
        
        // Window buttons (close, minimize, maximize)
        let btn_y = y - title_height + 6;
        let btn_size = 14;
        
        // Close button (red)
        self.draw_circle(x + 12, btn_y + 7, btn_size / 2, 0xFF3A2828);
        // Minimize button
        self.draw_circle(x + 32, btn_y + 7, btn_size / 2, 0xFF2A3028);
        // Maximize button  
        self.draw_circle(x + 52, btn_y + 7, btn_size / 2, 0xFF2A2A20);
        
        // Title text
        if !surface.title.is_empty() {
            // Simple title rendering (would use font in real impl)
            let title_x = x + 70;
            let title_y = btn_y + 4;
            // crate::framebuffer::draw_string(&surface.title, title_x as usize, title_y as usize, 0xFFE0E8E4);
        }
    }
    
    fn draw_circle(&self, cx: i32, cy: i32, r: i32, color: u32) {
        for dy in -r..=r {
            for dx in -r..=r {
                if dx * dx + dy * dy <= r * r {
                    let px = cx + dx;
                    let py = cy + dy;
                    if px >= 0 && py >= 0 {
                        crate::framebuffer::put_pixel(px as u32, py as u32, color);
                    }
                }
            }
        }
    }
    
    fn draw_cursor(&self) {
        // Simple arrow cursor
        let cursor = [
            0b11000000u8,
            0b11100000u8,
            0b11110000u8,
            0b11111000u8,
            0b11111100u8,
            0b11111110u8,
            0b11111111u8,
            0b11111100u8,
            0b11111100u8,
            0b11001100u8,
            0b10000110u8,
            0b00000110u8,
            0b00000011u8,
            0b00000011u8,
            0b00000000u8,
        ];
        
        for (y, row) in cursor.iter().enumerate() {
            for x in 0..8 {
                if (row >> (7 - x)) & 1 == 1 {
                    let px = self.pointer_x + x;
                    let py = self.pointer_y + y as i32;
                    if px >= 0 && py >= 0 && (px as u32) < self.width && (py as u32) < self.height {
                        crate::framebuffer::put_pixel(px as u32, py as u32, 0xFFFFFFFF);
                    }
                }
            }
        }
    }
    
    /// Move pointer
    pub fn move_pointer(&mut self, dx: i32, dy: i32) {
        self.pointer_x = (self.pointer_x + dx).clamp(0, self.width as i32 - 1);
        self.pointer_y = (self.pointer_y + dy).clamp(0, self.height as i32 - 1);
    }
    
    /// Process input events
    pub fn process_input(&mut self) {
        // Would read from keyboard/mouse queues and dispatch to clients
    }
    
    /// Find surface at coordinates
    pub fn surface_at(&self, x: i32, y: i32) -> Option<u32> {
        // Search from top to bottom
        for &id in self.surface_order.iter().rev() {
            if let Some(surface) = self.surfaces.get(&id) {
                if surface.contains(x, y) {
                    return Some(id);
                }
            }
        }
        None
    }
}

/// A connected Wayland client
pub struct WaylandClient {
    pub id: u32,
    pub surfaces: Vec<u32>,
}
