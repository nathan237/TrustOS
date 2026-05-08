




























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






static Gg: Mutex<Option<WaylandCompositor>> = Mutex::new(None);


static CLB_: AtomicU32 = AtomicU32::new(1);


pub fn gjd() -> u32 {
    CLB_.fetch_add(1, Ordering::SeqCst)
}


pub fn init() -> Result<(), &'static str> {
    let mut compositor = Gg.lock();
    if compositor.is_some() {
        return Err("Wayland compositor already initialized");
    }
    
    let (width, height) = crate::framebuffer::kv();
    
    let bww = WaylandCompositor::new(width, height);
    *compositor = Some(bww);
    
    crate::serial_println!("[WAYLAND] Compositor initialized ({}x{})", width, height);
    Ok(())
}


pub fn bjz<F, U>(f: F) -> Option<U>
where
    F: FnOnce(&mut WaylandCompositor) -> U,
{
    let mut jg = Gg.lock();
    jg.as_mut().map(f)
}


pub fn cho() {
    bjz(|compositor| {
        compositor.compose();
    });
}


pub fn process_input() {
    bjz(|compositor| {
        compositor.process_input();
    });
}






pub struct WaylandCompositor {
    
    pub width: u32,
    pub height: u32,
    
    
    pub surfaces: BTreeMap<u32, Surface>,
    
    
    pub surface_order: Vec<u32>,
    
    
    pub focused_surface: Option<u32>,
    
    
    pub pointer_x: i32,
    pub pointer_y: i32,
    
    
    pub shm_pools: BTreeMap<u32, ShmPool>,
    
    
    pub clients: BTreeMap<u32, Arz>,
    
    
    pub frame_number: u64,
    
    
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
            background_color: 0xFF0A0F0C, 
        }
    }
    
    
    pub fn create_surface(&mut self) -> u32 {
        let id = gjd();
        let surface = Surface::new(id);
        self.surfaces.insert(id, surface);
        self.surface_order.push(id);
        crate::serial_println!("[WAYLAND] Created surface {}", id);
        id
    }
    
    
    pub fn qct(&mut self, id: u32) {
        self.surfaces.remove(&id);
        self.surface_order.retain(|&x| x != id);
        if self.focused_surface == Some(id) {
            self.focused_surface = self.surface_order.last().copied();
        }
        crate::serial_println!("[WAYLAND] Destroyed surface {}", id);
    }
    
    
    pub fn qrs(&mut self, id: u32) {
        self.surface_order.retain(|&x| x != id);
        self.surface_order.push(id);
        self.focused_surface = Some(id);
    }
    
    
    pub fn qbx(&mut self, size: usize) -> u32 {
        let id = gjd();
        let gnr = ShmPool::new(id, size);
        self.shm_pools.insert(id, gnr);
        crate::serial_println!("[WAYLAND] Created SHM pool {} ({} bytes)", id, size);
        id
    }
    
    
    pub fn compose(&mut self) {
        self.frame_number += 1;
        
        
        let (width, height) = crate::framebuffer::kv();
        
        
        self.draw_background(width, height);
        
        
        for &avh in &self.surface_order {
            if let Some(surface) = self.surfaces.get(&avh) {
                if surface.visible && surface.committed {
                    self.draw_surface(surface);
                }
            }
        }
        
        
        self.draw_cursor();
    }
    
    fn draw_background(&self, width: u32, height: u32) {
        
        for y in 0..height {
            for x in 0..width {
                let pattern: u32 = if (x + y) % 32 < 16 { 0x00 } else { 0x02 };
                let pwv: u32 = 0xFF000000u32 | (pattern << 16) | ((pattern + 0x08) << 8) | pattern;
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
        
        
        if surface.has_decorations {
            self.draw_decorations(surface);
        }
        
        
        for ak in 0..h {
            for am in 0..w {
                let idx = (ak * w + am) as usize;
                if idx < surface.buffer.len() {
                    let ct = surface.buffer[idx];
                    let p = x + am as i32;
                    let o = y + ak as i32;
                    if p >= 0 && o >= 0 {
                        crate::framebuffer::put_pixel(p as u32, o as u32, ct);
                    }
                }
            }
        }
    }
    
    fn draw_decorations(&self, surface: &Surface) {
        let x = surface.x;
        let y = surface.y;
        let w = surface.width as i32;
        let crr = 28;
        
        
        let bwl = if self.focused_surface == Some(surface.id) {
            0xFF1A2A20 
        } else {
            0xFF0D1310 
        };
        
        for ty in 0..crr {
            for bu in 0..w {
                let p = x + bu;
                let o = y - crr + ty;
                if p >= 0 && o >= 0 {
                    crate::framebuffer::put_pixel(p as u32, o as u32, bwl);
                }
            }
        }
        
        
        let ed = y - crr + 6;
        let wv = 14;
        
        
        self.draw_circle(x + 12, ed + 7, wv / 2, 0xFF3A2828);
        
        self.draw_circle(x + 32, ed + 7, wv / 2, 0xFF2A3028);
        
        self.draw_circle(x + 52, ed + 7, wv / 2, 0xFF2A2A20);
        
        
        if !surface.title.is_empty() {
            
            let avk = x + 70;
            let apg = ed + 4;
            
        }
    }
    
    fn draw_circle(&self, cx: i32, u: i32, r: i32, color: u32) {
        for ad in -r..=r {
            for dx in -r..=r {
                if dx * dx + ad * ad <= r * r {
                    let p = cx + dx;
                    let o = u + ad;
                    if p >= 0 && o >= 0 {
                        crate::framebuffer::put_pixel(p as u32, o as u32, color);
                    }
                }
            }
        }
    }
    
    fn draw_cursor(&self) {
        
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
                    let p = self.pointer_x + x;
                    let o = self.pointer_y + y as i32;
                    if p >= 0 && o >= 0 && (p as u32) < self.width && (o as u32) < self.height {
                        crate::framebuffer::put_pixel(p as u32, o as u32, 0xFFFFFFFF);
                    }
                }
            }
        }
    }
    
    
    pub fn qpe(&mut self, dx: i32, ad: i32) {
        self.pointer_x = (self.pointer_x + dx).clamp(0, self.width as i32 - 1);
        self.pointer_y = (self.pointer_y + ad).clamp(0, self.height as i32 - 1);
    }
    
    
    pub fn process_input(&mut self) {
        
    }
    
    
    pub fn qya(&self, x: i32, y: i32) -> Option<u32> {
        
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


pub struct Arz {
    pub id: u32,
    pub surfaces: Vec<u32>,
}
