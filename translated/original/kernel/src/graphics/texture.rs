//! TrustGL Texture Support
//!
//! Provides OpenGL-style texture functionality inspired by TinyGL.
//! Supports 1D, 2D textures with mipmapping and various filters.

use alloc::vec::Vec;
use alloc::boxed::Box;
use spin::Mutex;
use micromath::F32Ext;

use super::render2d::Color2D;

// ═══════════════════════════════════════════════════════════════════════════════
// TEXTURE CONSTANTS
// ═══════════════════════════════════════════════════════════════════════════════

// Texture targets
pub const GL_TEXTURE_1D: u32 = 0x0DE0;
pub const GL_TEXTURE_2D: u32 = 0x0DE1;

// Texture parameters
pub const GL_TEXTURE_MAG_FILTER: u32 = 0x2800;
pub const GL_TEXTURE_MIN_FILTER: u32 = 0x2801;
pub const GL_TEXTURE_WRAP_S: u32 = 0x2802;
pub const GL_TEXTURE_WRAP_T: u32 = 0x2803;

// Texture filter modes
pub const GL_NEAREST: u32 = 0x2600;
pub const GL_LINEAR: u32 = 0x2601;
pub const GL_NEAREST_MIPMAP_NEAREST: u32 = 0x2700;
pub const GL_LINEAR_MIPMAP_NEAREST: u32 = 0x2701;
pub const GL_NEAREST_MIPMAP_LINEAR: u32 = 0x2702;
pub const GL_LINEAR_MIPMAP_LINEAR: u32 = 0x2703;

// Texture wrap modes
pub const GL_REPEAT: u32 = 0x2901;
pub const GL_CLAMP: u32 = 0x2900;
pub const GL_CLAMP_TO_EDGE: u32 = 0x812F;
pub const GL_MIRRORED_REPEAT: u32 = 0x8370;

// Texture formats
pub const GL_RGB: u32 = 0x1907;
pub const GL_RGBA: u32 = 0x1908;
pub const GL_LUMINANCE: u32 = 0x1909;
pub const GL_LUMINANCE_ALPHA: u32 = 0x190A;

// ═══════════════════════════════════════════════════════════════════════════════
// TEXTURE OBJECT
// ═══════════════════════════════════════════════════════════════════════════════

/// A single mipmap level
#[derive(Clone)]
pub struct TextureLevel {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u32>, // RGBA packed
}

impl TextureLevel {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            data: alloc::vec![0u32; (width * height) as usize],
        }
    }
    
    /// Sample at exact pixel coordinates
    #[inline]
    pub fn sample_pixel(&self, x: u32, y: u32) -> u32 {
        let x = x.min(self.width.saturating_sub(1));
        let y = y.min(self.height.saturating_sub(1));
        self.data[(y * self.width + x) as usize]
    }
    
    /// Sample with bilinear filtering
    pub fn sample_bilinear(&self, u: f32, v: f32) -> u32 {
        let x = u * (self.width as f32 - 1.0);
        let y = v * (self.height as f32 - 1.0);
        
        let x0 = x.floor() as u32;
        let y0 = y.floor() as u32;
        let x1 = (x0 + 1).min(self.width - 1);
        let y1 = (y0 + 1).min(self.height - 1);
        
        let fx = x.fract();
        let fy = y.fract();
        
        let c00 = self.sample_pixel(x0, y0);
        let c10 = self.sample_pixel(x1, y0);
        let c01 = self.sample_pixel(x0, y1);
        let c11 = self.sample_pixel(x1, y1);
        
        // Bilinear interpolation for each channel
        let r = Self::blerp(
            ((c00 >> 16) & 0xFF) as f32,
            ((c10 >> 16) & 0xFF) as f32,
            ((c01 >> 16) & 0xFF) as f32,
            ((c11 >> 16) & 0xFF) as f32,
            fx, fy
        ) as u32;
        let g = Self::blerp(
            ((c00 >> 8) & 0xFF) as f32,
            ((c10 >> 8) & 0xFF) as f32,
            ((c01 >> 8) & 0xFF) as f32,
            ((c11 >> 8) & 0xFF) as f32,
            fx, fy
        ) as u32;
        let b = Self::blerp(
            (c00 & 0xFF) as f32,
            (c10 & 0xFF) as f32,
            (c01 & 0xFF) as f32,
            (c11 & 0xFF) as f32,
            fx, fy
        ) as u32;
        let a = Self::blerp(
            ((c00 >> 24) & 0xFF) as f32,
            ((c10 >> 24) & 0xFF) as f32,
            ((c01 >> 24) & 0xFF) as f32,
            ((c11 >> 24) & 0xFF) as f32,
            fx, fy
        ) as u32;
        
        (a << 24) | (r << 16) | (g << 8) | b
    }
    
    #[inline]
    fn blerp(c00: f32, c10: f32, c01: f32, c11: f32, fx: f32, fy: f32) -> f32 {
        let c0 = c00 + (c10 - c00) * fx;
        let c1 = c01 + (c11 - c01) * fx;
        c0 + (c1 - c0) * fy
    }
}

/// Texture object with mipmaps
pub struct Texture {
    pub id: u32,
    pub target: u32,
    pub levels: Vec<TextureLevel>,
    pub mag_filter: u32,
    pub min_filter: u32,
    pub wrap_s: u32,
    pub wrap_t: u32,
}

impl Texture {
    pub fn new(id: u32) -> Self {
        Self {
            id,
            target: GL_TEXTURE_2D,
            levels: Vec::new(),
            mag_filter: GL_LINEAR,
            min_filter: GL_NEAREST_MIPMAP_LINEAR,
            wrap_s: GL_REPEAT,
            wrap_t: GL_REPEAT,
        }
    }
    
    /// Upload texture data (level 0)
    pub fn upload(&mut self, width: u32, height: u32, format: u32, data: &[u8]) {
        let mut level = TextureLevel::new(width, height);
        
        match format {
            GL_RGBA => {
                for (i, chunk) in data.chunks(4).enumerate() {
                    if chunk.len() == 4 && i < level.data.len() {
                        level.data[i] = ((chunk[3] as u32) << 24) 
                                      | ((chunk[0] as u32) << 16) 
                                      | ((chunk[1] as u32) << 8) 
                                      | (chunk[2] as u32);
                    }
                }
            }
            GL_RGB => {
                for (i, chunk) in data.chunks(3).enumerate() {
                    if chunk.len() == 3 && i < level.data.len() {
                        level.data[i] = 0xFF000000 
                                      | ((chunk[0] as u32) << 16) 
                                      | ((chunk[1] as u32) << 8) 
                                      | (chunk[2] as u32);
                    }
                }
            }
            GL_LUMINANCE => {
                for (i, &lum) in data.iter().enumerate() {
                    if i < level.data.len() {
                        level.data[i] = 0xFF000000 
                                      | ((lum as u32) << 16) 
                                      | ((lum as u32) << 8) 
                                      | (lum as u32);
                    }
                }
            }
            _ => {}
        }
        
        self.levels.clear();
        self.levels.push(level);
    }
    
    /// Generate mipmaps from level 0
    pub fn generate_mipmaps(&mut self) {
        if self.levels.is_empty() {
            return;
        }
        
        let mut w = self.levels[0].width / 2;
        let mut h = self.levels[0].height / 2;
        
        while w >= 1 && h >= 1 {
            let prev = &self.levels[self.levels.len() - 1];
            let mut level = TextureLevel::new(w, h);
            
            // 2x2 box filter
            for y in 0..h {
                for x in 0..w {
                    let sx = x * 2;
                    let sy = y * 2;
                    
                    let c00 = prev.sample_pixel(sx, sy);
                    let c10 = prev.sample_pixel(sx + 1, sy);
                    let c01 = prev.sample_pixel(sx, sy + 1);
                    let c11 = prev.sample_pixel(sx + 1, sy + 1);
                    
                    let r = (((c00 >> 16) & 0xFF) + ((c10 >> 16) & 0xFF) 
                           + ((c01 >> 16) & 0xFF) + ((c11 >> 16) & 0xFF)) / 4;
                    let g = (((c00 >> 8) & 0xFF) + ((c10 >> 8) & 0xFF) 
                           + ((c01 >> 8) & 0xFF) + ((c11 >> 8) & 0xFF)) / 4;
                    let b = ((c00 & 0xFF) + (c10 & 0xFF) 
                           + (c01 & 0xFF) + (c11 & 0xFF)) / 4;
                    let a = (((c00 >> 24) & 0xFF) + ((c10 >> 24) & 0xFF) 
                           + ((c01 >> 24) & 0xFF) + ((c11 >> 24) & 0xFF)) / 4;
                    
                    level.data[(y * w + x) as usize] = (a << 24) | (r << 16) | (g << 8) | b;
                }
            }
            
            self.levels.push(level);
            
            if w == 1 && h == 1 { break; }
            w = w.max(1) / 2;
            h = h.max(1) / 2;
            if w == 0 { w = 1; }
            if h == 0 { h = 1; }
        }
    }
    
    /// Sample texture at UV coordinates with current filters
    pub fn sample(&self, mut u: f32, mut v: f32) -> u32 {
        if self.levels.is_empty() {
            return 0xFFFFFFFF;
        }
        
        // Apply wrap mode
        u = self.apply_wrap(u, self.wrap_s);
        v = self.apply_wrap(v, self.wrap_t);
        
        let level = &self.levels[0]; // TODO: mipmap selection
        
        match self.mag_filter {
            GL_NEAREST => {
                let x = (u * level.width as f32) as u32;
                let y = (v * level.height as f32) as u32;
                level.sample_pixel(x, y)
            }
            _ => level.sample_bilinear(u, v),
        }
    }
    
    fn apply_wrap(&self, coord: f32, mode: u32) -> f32 {
        match mode {
            GL_CLAMP | GL_CLAMP_TO_EDGE => coord.clamp(0.0, 1.0),
            GL_MIRRORED_REPEAT => {
                let i = coord.floor() as i32;
                let f = coord.fract();
                if i % 2 == 0 { f } else { 1.0 - f }
            }
            _ => { // GL_REPEAT
                let f = coord.fract();
                if f < 0.0 { 1.0 + f } else { f }
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TEXTURE MANAGER
// ═══════════════════════════════════════════════════════════════════════════════

/// Global texture state
pub struct TextureState {
    textures: Vec<Option<Texture>>,
    next_id: u32,
    bound_2d: Option<u32>,
    texture_2d_enabled: bool,
}

impl TextureState {
    pub const fn new() -> Self {
        Self {
            textures: Vec::new(),
            next_id: 1,
            bound_2d: None,
            texture_2d_enabled: false,
        }
    }
}

static TEXTURE_STATE: Mutex<TextureState> = Mutex::new(TextureState::new());

// ═══════════════════════════════════════════════════════════════════════════════
// GL FUNCTIONS
// ═══════════════════════════════════════════════════════════════════════════════

/// Generate texture names
pub fn gl_gen_textures(n: i32, textures: &mut [u32]) {
    let mut state = TEXTURE_STATE.lock();
    for i in 0..(n as usize) {
        if i < textures.len() {
            let id = state.next_id;
            textures[i] = id;
            state.textures.push(Some(Texture::new(id)));
            state.next_id += 1;
        }
    }
}

/// Bind a texture to target
pub fn gl_bind_texture(target: u32, texture: u32) {
    let mut state = TEXTURE_STATE.lock();
    if target == GL_TEXTURE_2D {
        state.bound_2d = if texture == 0 { None } else { Some(texture) };
    }
}

/// Set texture parameter
pub fn gl_tex_parameteri(target: u32, pname: u32, param: u32) {
    let mut state = TEXTURE_STATE.lock();
    
    let tex_id = match target {
        GL_TEXTURE_2D => state.bound_2d,
        _ => None,
    };
    
    if let Some(id) = tex_id {
        if let Some(Some(tex)) = state.textures.iter_mut().find(|t| t.as_ref().map(|x| x.id) == Some(id)) {
            match pname {
                GL_TEXTURE_MAG_FILTER => tex.mag_filter = param,
                GL_TEXTURE_MIN_FILTER => tex.min_filter = param,
                GL_TEXTURE_WRAP_S => tex.wrap_s = param,
                GL_TEXTURE_WRAP_T => tex.wrap_t = param,
                _ => {}
            }
        }
    }
}

/// Upload texture image
pub fn gl_tex_image_2d(
    _target: u32,
    _level: i32,
    _internal_format: u32,
    width: u32,
    height: u32,
    _border: i32,
    format: u32,
    _type: u32,
    data: &[u8],
) {
    let mut state = TEXTURE_STATE.lock();
    
    if let Some(id) = state.bound_2d {
        if let Some(Some(tex)) = state.textures.iter_mut().find(|t| t.as_ref().map(|x| x.id) == Some(id)) {
            tex.upload(width, height, format, data);
        }
    }
}

/// Generate mipmaps for bound texture
pub fn gl_generate_mipmap(target: u32) {
    let mut state = TEXTURE_STATE.lock();
    
    let tex_id = match target {
        GL_TEXTURE_2D => state.bound_2d,
        _ => None,
    };
    
    if let Some(id) = tex_id {
        if let Some(Some(tex)) = state.textures.iter_mut().find(|t| t.as_ref().map(|x| x.id) == Some(id)) {
            tex.generate_mipmaps();
        }
    }
}

/// Enable texture target
pub fn gl_enable_texture(target: u32) {
    let mut state = TEXTURE_STATE.lock();
    if target == GL_TEXTURE_2D {
        state.texture_2d_enabled = true;
    }
}

/// Disable texture target
pub fn gl_disable_texture(target: u32) {
    let mut state = TEXTURE_STATE.lock();
    if target == GL_TEXTURE_2D {
        state.texture_2d_enabled = false;
    }
}

/// Sample currently bound texture at UV coords
pub fn sample_bound_texture(u: f32, v: f32) -> Option<u32> {
    let state = TEXTURE_STATE.lock();
    
    if !state.texture_2d_enabled {
        return None;
    }
    
    if let Some(id) = state.bound_2d {
        if let Some(Some(tex)) = state.textures.iter().find(|t| t.as_ref().map(|x| x.id) == Some(id)) {
            return Some(tex.sample(u, v));
        }
    }
    
    None
}

/// Check if texturing is enabled
pub fn is_texture_2d_enabled() -> bool {
    TEXTURE_STATE.lock().texture_2d_enabled
}

/// Delete textures
pub fn gl_delete_textures(n: i32, textures: &[u32]) {
    let mut state = TEXTURE_STATE.lock();
    for i in 0..(n as usize) {
        if i < textures.len() {
            let id = textures[i];
            if let Some(pos) = state.textures.iter().position(|t| t.as_ref().map(|x| x.id) == Some(id)) {
                state.textures[pos] = None;
            }
            if state.bound_2d == Some(id) {
                state.bound_2d = None;
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// PROCEDURAL TEXTURES
// ═══════════════════════════════════════════════════════════════════════════════

/// Create a checkerboard texture
pub fn create_checkerboard_texture(size: u32, color1: u32, color2: u32) -> Vec<u8> {
    let mut data = Vec::with_capacity((size * size * 4) as usize);
    let check_size = size / 8;
    
    for y in 0..size {
        for x in 0..size {
            let cx = x / check_size.max(1);
            let cy = y / check_size.max(1);
            let color = if (cx + cy) % 2 == 0 { color1 } else { color2 };
            
            data.push(((color >> 16) & 0xFF) as u8); // R
            data.push(((color >> 8) & 0xFF) as u8);  // G
            data.push((color & 0xFF) as u8);         // B
            data.push(((color >> 24) & 0xFF) as u8); // A
        }
    }
    
    data
}

/// Create a brick texture
pub fn create_brick_texture(size: u32) -> Vec<u8> {
    let mut data = Vec::with_capacity((size * size * 4) as usize);
    let brick_h = size / 4;
    let brick_w = size / 2;
    let mortar = 2u32;
    
    let brick_color: u32 = 0xFF8B4513; // Dark brown
    let mortar_color: u32 = 0xFFC0C0C0; // Gray
    
    for y in 0..size {
        for x in 0..size {
            let row = y / brick_h;
            let offset = if row % 2 == 0 { 0 } else { brick_w / 2 };
            let bx = (x + offset) % brick_w;
            let by = y % brick_h;
            
            let is_mortar = by < mortar || bx < mortar;
            let color = if is_mortar { mortar_color } else { brick_color };
            
            data.push(((color >> 16) & 0xFF) as u8);
            data.push(((color >> 8) & 0xFF) as u8);
            data.push((color & 0xFF) as u8);
            data.push(0xFF);
        }
    }
    
    data
}

/// Create a gradient texture
pub fn create_gradient_texture(size: u32, color1: Color2D, color2: Color2D, horizontal: bool) -> Vec<u8> {
    let mut data = Vec::with_capacity((size * size * 4) as usize);
    
    for y in 0..size {
        for x in 0..size {
            let t = if horizontal {
                x as f32 / size as f32
            } else {
                y as f32 / size as f32
            };
            
            let r = (color1.r as f32 + (color2.r as f32 - color1.r as f32) * t) as u8;
            let g = (color1.g as f32 + (color2.g as f32 - color1.g as f32) * t) as u8;
            let b = (color1.b as f32 + (color2.b as f32 - color1.b as f32) * t) as u8;
            
            data.push(r);
            data.push(g);
            data.push(b);
            data.push(0xFF);
        }
    }
    
    data
}
