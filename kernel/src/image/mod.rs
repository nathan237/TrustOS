// ═══════════════════════════════════════════════════════════════════════════════
// TrustOS Image System
// ═══════════════════════════════════════════════════════════════════════════════
//
// Load and display images (BMP, PPM, raw formats)
// Draw images to framebuffer with scaling, transparency, etc.
//
// ═══════════════════════════════════════════════════════════════════════════════

use alloc::string::String;
use alloc::vec::Vec;

pub mod bmp;
pub mod ppm;

pub use bmp::*;
pub use ppm::*;

/// Image data structure
#[derive(Clone)]
pub struct Image {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u32>,  // ARGB format
}

impl Image {
    /// Create a new empty image
    pub fn new(width: u32, height: u32) -> Self {
        let size = (width * height) as usize;
        Self {
            width,
            height,
            pixels: alloc::vec![0xFF000000; size],
        }
    }
    
    /// Create from raw ARGB pixels
    pub fn from_pixels(width: u32, height: u32, pixels: Vec<u32>) -> Self {
        Self { width, height, pixels }
    }
    
    /// Get pixel at (x, y)
    pub fn get_pixel(&self, x: u32, y: u32) -> u32 {
        if x < self.width && y < self.height {
            self.pixels[(y * self.width + x) as usize]
        } else {
            0
        }
    }
    
    /// Set pixel at (x, y)
    pub fn set_pixel(&mut self, x: u32, y: u32, color: u32) {
        if x < self.width && y < self.height {
            self.pixels[(y * self.width + x) as usize] = color;
        }
    }
    
    /// Fill with solid color
    pub fn fill(&mut self, color: u32) {
        for pixel in &mut self.pixels {
            *pixel = color;
        }
    }
    
    /// Draw to framebuffer at position
    pub fn draw(&self, x: i32, y: i32) {
        self.draw_scaled(x, y, self.width, self.height);
    }
    
    /// Draw with scaling
    pub fn draw_scaled(&self, x: i32, y: i32, dest_w: u32, dest_h: u32) {
        let (fb_width, fb_height) = crate::framebuffer::get_dimensions();
        
        for dy in 0..dest_h {
            let screen_y = y + dy as i32;
            if screen_y < 0 || screen_y >= fb_height as i32 { continue; }
            
            // Source Y coordinate (scaled)
            let src_y = (dy as u64 * self.height as u64 / dest_h as u64) as u32;
            
            for dx in 0..dest_w {
                let screen_x = x + dx as i32;
                if screen_x < 0 || screen_x >= fb_width as i32 { continue; }
                
                // Source X coordinate (scaled)
                let src_x = (dx as u64 * self.width as u64 / dest_w as u64) as u32;
                
                let pixel = self.get_pixel(src_x, src_y);
                let alpha = (pixel >> 24) & 0xFF;
                
                if alpha == 255 {
                    // Opaque - direct draw
                    crate::framebuffer::put_pixel(screen_x as u32, screen_y as u32, pixel);
                } else if alpha > 0 {
                    // Semi-transparent - blend
                    let bg = crate::framebuffer::get_pixel(screen_x as u32, screen_y as u32);
                    let blended = blend_pixels(pixel, bg);
                    crate::framebuffer::put_pixel(screen_x as u32, screen_y as u32, blended);
                }
                // alpha == 0 means fully transparent, skip
            }
        }
    }
    
    /// Draw with alpha blending (for overlays)
    pub fn draw_blended(&self, x: i32, y: i32, global_alpha: u8) {
        let (fb_width, fb_height) = crate::framebuffer::get_dimensions();
        
        for sy in 0..self.height {
            let screen_y = y + sy as i32;
            if screen_y < 0 || screen_y >= fb_height as i32 { continue; }
            
            for sx in 0..self.width {
                let screen_x = x + sx as i32;
                if screen_x < 0 || screen_x >= fb_width as i32 { continue; }
                
                let pixel = self.get_pixel(sx, sy);
                let pixel_alpha = ((pixel >> 24) & 0xFF) as u32;
                let combined_alpha = (pixel_alpha * global_alpha as u32) / 255;
                
                if combined_alpha > 0 {
                    let modified = (combined_alpha << 24) | (pixel & 0x00FFFFFF);
                    let bg = crate::framebuffer::get_pixel(screen_x as u32, screen_y as u32);
                    let blended = blend_pixels(modified, bg);
                    crate::framebuffer::put_pixel(screen_x as u32, screen_y as u32, blended);
                }
            }
        }
    }
    
    /// Create a scaled copy of the image
    pub fn scale(&self, new_width: u32, new_height: u32) -> Image {
        let mut scaled = Image::new(new_width, new_height);
        
        for dy in 0..new_height {
            let src_y = (dy as u64 * self.height as u64 / new_height as u64) as u32;
            for dx in 0..new_width {
                let src_x = (dx as u64 * self.width as u64 / new_width as u64) as u32;
                scaled.set_pixel(dx, dy, self.get_pixel(src_x, src_y));
            }
        }
        
        scaled
    }
    
    /// Crop a region of the image
    pub fn crop(&self, x: u32, y: u32, w: u32, h: u32) -> Image {
        let mut cropped = Image::new(w, h);
        
        for dy in 0..h {
            for dx in 0..w {
                cropped.set_pixel(dx, dy, self.get_pixel(x + dx, y + dy));
            }
        }
        
        cropped
    }
}

/// Blend foreground pixel over background with alpha
fn blend_pixels(fg: u32, bg: u32) -> u32 {
    let alpha = ((fg >> 24) & 0xFF) as u32;
    if alpha == 0 { return bg; }
    if alpha == 255 { return fg; }
    
    let inv_alpha = 255 - alpha;
    
    let fg_r = ((fg >> 16) & 0xFF) as u32;
    let fg_g = ((fg >> 8) & 0xFF) as u32;
    let fg_b = (fg & 0xFF) as u32;
    
    let bg_r = ((bg >> 16) & 0xFF) as u32;
    let bg_g = ((bg >> 8) & 0xFF) as u32;
    let bg_b = (bg & 0xFF) as u32;
    
    let r = (fg_r * alpha + bg_r * inv_alpha) / 255;
    let g = (fg_g * alpha + bg_g * inv_alpha) / 255;
    let b = (fg_b * alpha + bg_b * inv_alpha) / 255;
    
    0xFF000000 | (r << 16) | (g << 8) | b
}

// ═══════════════════════════════════════════════════════════════════════════════
// IMAGE LOADING
// ═══════════════════════════════════════════════════════════════════════════════

/// Load image from file (auto-detect format)
pub fn load(path: &str) -> Option<Image> {
    // Detect format by extension
    let lower_path = path.to_lowercase_simple();
    
    if lower_path.ends_with(".bmp") {
        bmp::load_bmp(path)
    } else if lower_path.ends_with(".ppm") || lower_path.ends_with(".pnm") {
        ppm::load_ppm(path)
    } else if lower_path.ends_with(".raw") || lower_path.ends_with(".rgba") {
        // Try to detect from content
        if let Ok(data) = crate::vfs::read_file(path) {
            // Raw format: first 8 bytes are width, height as u32
            if data.len() >= 8 {
                let width = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
                let height = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
                let pixel_data = &data[8..];
                
                if pixel_data.len() >= (width * height * 4) as usize {
                    let mut pixels = Vec::with_capacity((width * height) as usize);
                    for i in 0..(width * height) as usize {
                        let offset = i * 4;
                        let pixel = u32::from_le_bytes([
                            pixel_data[offset],
                            pixel_data[offset + 1],
                            pixel_data[offset + 2],
                            pixel_data[offset + 3],
                        ]);
                        pixels.push(pixel);
                    }
                    return Some(Image::from_pixels(width, height, pixels));
                }
            }
        }
        None
    } else {
        // Try BMP first, then PPM
        bmp::load_bmp(path).or_else(|| ppm::load_ppm(path))
    }
}

/// Load image from raw bytes with known format
pub fn load_from_bytes(data: &[u8], format: ImageFormat) -> Option<Image> {
    match format {
        ImageFormat::Bmp => bmp::load_bmp_from_bytes(data),
        ImageFormat::Ppm => ppm::load_ppm_from_bytes(data),
        ImageFormat::Raw { width, height } => {
            if data.len() >= (width * height * 4) as usize {
                let mut pixels = Vec::with_capacity((width * height) as usize);
                for i in 0..(width * height) as usize {
                    let offset = i * 4;
                    let pixel = u32::from_le_bytes([
                        data[offset],
                        data[offset + 1],
                        data[offset + 2],
                        data[offset + 3],
                    ]);
                    pixels.push(pixel);
                }
                Some(Image::from_pixels(width, height, pixels))
            } else {
                None
            }
        }
    }
}

#[derive(Clone, Copy)]
pub enum ImageFormat {
    Bmp,
    Ppm,
    Raw { width: u32, height: u32 },
}

// Simple lowercase for no_std (ASCII only)
trait ToLowercaseSimple {
    fn to_lowercase_simple(&self) -> String;
}

impl ToLowercaseSimple for str {
    fn to_lowercase_simple(&self) -> String {
        self.chars().map(|c| {
            if c >= 'A' && c <= 'Z' {
                (c as u8 + 32) as char
            } else {
                c
            }
        }).collect()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SIMPLE IMAGE CREATION HELPERS
// ═══════════════════════════════════════════════════════════════════════════════

/// Create a solid color image
pub fn create_solid(width: u32, height: u32, color: u32) -> Image {
    let mut img = Image::new(width, height);
    img.fill(color);
    img
}

/// Create a gradient image (vertical)
pub fn create_gradient_v(width: u32, height: u32, top_color: u32, bottom_color: u32) -> Image {
    let mut img = Image::new(width, height);
    
    let tr = ((top_color >> 16) & 0xFF) as i32;
    let tg = ((top_color >> 8) & 0xFF) as i32;
    let tb = (top_color & 0xFF) as i32;
    
    let br = ((bottom_color >> 16) & 0xFF) as i32;
    let bg = ((bottom_color >> 8) & 0xFF) as i32;
    let bb = (bottom_color & 0xFF) as i32;
    
    for y in 0..height {
        let t = y as f32 / height as f32;
        let r = (tr as f32 * (1.0 - t) + br as f32 * t) as u32;
        let g = (tg as f32 * (1.0 - t) + bg as f32 * t) as u32;
        let b = (tb as f32 * (1.0 - t) + bb as f32 * t) as u32;
        let color = 0xFF000000 | (r << 16) | (g << 8) | b;
        
        for x in 0..width {
            img.set_pixel(x, y, color);
        }
    }
    
    img
}

/// Create a checkerboard pattern
pub fn create_checkerboard(width: u32, height: u32, size: u32, color1: u32, color2: u32) -> Image {
    let mut img = Image::new(width, height);
    
    for y in 0..height {
        for x in 0..width {
            let checker = ((x / size) + (y / size)) % 2 == 0;
            img.set_pixel(x, y, if checker { color1 } else { color2 });
        }
    }
    
    img
}
