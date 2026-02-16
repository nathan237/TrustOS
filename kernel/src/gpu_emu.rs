//! GPU Emulation Layer
//!
//! Virtualizes CPU cores to behave like GPU compute units.
//! Provides a shader-like programming model for parallel computation.
//!
//! Key concepts:
//! - WorkGroup: A batch of work items processed together
//! - Shader: A function executed on each work item
//! - Dispatch: Launch parallel execution across all virtual cores

use core::sync::atomic::{AtomicU32, AtomicBool, Ordering};
use alloc::vec::Vec;

/// Number of virtual GPU cores (emulated via SIMD lanes + CPU cores)
/// Each CPU core can emulate 4-8 virtual cores via SSE2/AVX
pub const VIRTUAL_CORES: usize = 32;

/// Work items per workgroup (like GPU wavefront/warp)
pub const WORKGROUP_SIZE: usize = 64;

/// Maximum concurrent workgroups
pub const MAX_WORKGROUPS: usize = 256;

// ============================================================================
// SHADER TYPES - Functions that run on each pixel/item
// ============================================================================

/// Pixel shader input - position and uniforms
#[derive(Clone, Copy)]
#[repr(C)]
pub struct PixelInput {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub time: f32,
    pub frame: u32,
}

/// Pixel shader output - RGBA color
#[derive(Clone, Copy, Default)]
#[repr(C)]
pub struct PixelOutput {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl PixelOutput {
    #[inline]
    pub fn to_u32(&self) -> u32 {
        ((self.a as u32) << 24) | ((self.r as u32) << 16) | ((self.g as u32) << 8) | (self.b as u32)
    }
    
    #[inline]
    pub fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }
    
    #[inline]
    pub fn from_rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
}

/// Pixel shader function type
pub type PixelShaderFn = fn(input: PixelInput) -> PixelOutput;

/// Compute shader function type (generic work item processing)
pub type ComputeShaderFn = fn(global_id: u32, local_id: u32, uniforms: *const u8) -> u32;

// ============================================================================
// VIRTUAL GPU STATE
// ============================================================================

/// Virtual GPU state
pub struct VirtualGpu {
    /// Frame counter for animations
    frame: AtomicU32,
    /// Time accumulator (in ms)
    time_ms: AtomicU32,
    /// Active shader
    active_shader: Option<PixelShaderFn>,
    /// Framebuffer pointer
    framebuffer: *mut u32,
    /// Framebuffer dimensions
    width: u32,
    height: u32,
    /// Row stride in pixels (may differ from width on MMIO fb)
    stride: u32,
    /// Virtual core busy flags
    core_busy: [AtomicBool; VIRTUAL_CORES],
    /// Work items completed
    work_completed: AtomicU32,
}

unsafe impl Send for VirtualGpu {}
unsafe impl Sync for VirtualGpu {}

impl VirtualGpu {
    /// Create new virtual GPU
    pub const fn new() -> Self {
        const INIT_BOOL: AtomicBool = AtomicBool::new(false);
        Self {
            frame: AtomicU32::new(0),
            time_ms: AtomicU32::new(0),
            active_shader: None,
            framebuffer: core::ptr::null_mut(),
            width: 0,
            height: 0,
            stride: 0,
            core_busy: [INIT_BOOL; VIRTUAL_CORES],
            work_completed: AtomicU32::new(0),
        }
    }
    
    /// Initialize with framebuffer
    /// stride = row stride in pixels (width for backbuffer, pitch/4 for MMIO)
    pub fn init(&mut self, framebuffer: *mut u32, width: u32, height: u32, stride: u32) {
        self.framebuffer = framebuffer;
        self.width = width;
        self.height = height;
        self.stride = stride;
    }
    
    /// Set active pixel shader
    pub fn set_shader(&mut self, shader: PixelShaderFn) {
        self.active_shader = Some(shader);
    }
    
    /// Dispatch pixel shader across entire framebuffer
    /// This is the main "draw call" - like glDrawArrays
    pub fn dispatch_fullscreen(&self) {
        let Some(shader) = self.active_shader else { return };
        
        let width = self.width;
        let height = self.height;
        let total_pixels = (width * height) as usize;
        let time = self.time_ms.load(Ordering::Relaxed) as f32 / 1000.0;
        let frame = self.frame.load(Ordering::Relaxed);
        
        // Get number of CPU cores available
        let cpu_count = crate::cpu::smp::ready_cpu_count() as usize;
        let cpu_count = cpu_count.max(1);
        
        // Calculate pixels per core
        let pixels_per_core = (total_pixels + cpu_count - 1) / cpu_count;
        
        // Reset completion counter
        self.work_completed.store(0, Ordering::Release);
        
        // Create work context
        let ctx = ShaderContext {
            shader,
            framebuffer: self.framebuffer,
            width,
            height,
            stride: self.stride,
            time,
            frame,
        };
        
        // Dispatch to all cores using SMP parallel_for
        // Each core processes a horizontal band
        crate::cpu::smp::parallel_for(
            height as usize,
            dispatch_shader_row,
            &ctx as *const ShaderContext as *mut u8,
        );
        
        // Increment frame counter
        self.frame.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Dispatch with SIMD acceleration (4 pixels at a time)
    #[cfg(target_arch = "x86_64")]
    pub fn dispatch_fullscreen_simd(&self) {
        let Some(shader) = self.active_shader else { return };
        
        let width = self.width;
        let height = self.height;
        let time = self.time_ms.load(Ordering::Relaxed) as f32 / 1000.0;
        let frame = self.frame.load(Ordering::Relaxed);
        
        let ctx = ShaderContext {
            shader,
            framebuffer: self.framebuffer,
            width,
            height,
            stride: self.stride,
            time,
            frame,
        };
        
        // Dispatch using SMP + SIMD hybrid
        crate::cpu::smp::parallel_for(
            height as usize,
            dispatch_shader_row_simd,
            &ctx as *const ShaderContext as *mut u8,
        );
        
        self.frame.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Update time (call each frame with delta_ms)
    pub fn tick(&self, delta_ms: u32) {
        self.time_ms.fetch_add(delta_ms, Ordering::Relaxed);
    }
    
    /// Get current frame number
    pub fn frame(&self) -> u32 {
        self.frame.load(Ordering::Relaxed)
    }
    
    /// Get elapsed time in seconds
    pub fn time(&self) -> f32 {
        self.time_ms.load(Ordering::Relaxed) as f32 / 1000.0
    }
}

// ============================================================================
// STANDALONE DISPATCH FUNCTIONS - Easy to use without VirtualGpu instance
// ============================================================================

/// Dispatch a shader to the entire framebuffer (standalone function)
/// 
/// Parameters:
/// - `framebuffer`: Pointer to pixel buffer (ARGB u32)
/// - `width`: Screen width in pixels
/// - `height`: Screen height in pixels
/// - `time`: Animation time in seconds
/// - `frame`: Frame counter for animations
/// - `shader`: The shader function to execute
#[inline]
pub fn dispatch_fullscreen(
    framebuffer: *mut u32,
    width: u32,
    height: u32,
    time: f32,
    frame: u32,
    shader: PixelShaderFn,
) {
    dispatch_fullscreen_stride(framebuffer, width, height, width, time, frame, shader);
}

/// Dispatch a shader with explicit stride (pixels per row)
pub fn dispatch_fullscreen_stride(
    framebuffer: *mut u32,
    width: u32,
    height: u32,
    stride: u32,
    time: f32,
    frame: u32,
    shader: PixelShaderFn,
) {
    let ctx = ShaderContext {
        shader,
        framebuffer,
        width,
        height,
        stride,
        time,
        frame,
    };

    // Use SIMD dispatch on x86_64, fallback on other architectures
    #[cfg(target_arch = "x86_64")]
    {
        crate::cpu::smp::parallel_for(
            height as usize,
            dispatch_shader_row_simd,
            &ctx as *const ShaderContext as *mut u8,
        );
    }

    #[cfg(not(target_arch = "x86_64"))]
    {
        crate::cpu::smp::parallel_for(
            height as usize,
            dispatch_shader_row,
            &ctx as *const ShaderContext as *mut u8,
        );
    }
}

// ============================================================================
// SHADER DISPATCH INTERNALS
// ============================================================================

/// Context passed to shader dispatch functions
#[repr(C)]
struct ShaderContext {
    shader: PixelShaderFn,
    framebuffer: *mut u32,
    width: u32,
    height: u32,
    stride: u32,  // row stride in pixels (may differ from width for MMIO fb)
    time: f32,
    frame: u32,
}

unsafe impl Send for ShaderContext {}
unsafe impl Sync for ShaderContext {}

/// Dispatch shader for a range of rows (called by parallel_for)
fn dispatch_shader_row(start: usize, end: usize, data: *mut u8) {
    let ctx = unsafe { &*(data as *const ShaderContext) };
    let shader = ctx.shader;
    let fb = ctx.framebuffer;
    let width = ctx.width;
    let height = ctx.height;
    let stride = ctx.stride as usize;

    for y in start..end {
        let row_offset = y * stride;
        for x in 0..width as usize {
            let input = PixelInput {
                x: x as u32,
                y: y as u32,
                width,
                height,
                time: ctx.time,
                frame: ctx.frame,
            };

            let output = shader(input);
            unsafe {
                *fb.add(row_offset + x) = output.to_u32();
            }
        }
    }
}

/// SIMD-accelerated shader dispatch (4 pixels at a time)
#[cfg(target_arch = "x86_64")]
fn dispatch_shader_row_simd(start: usize, end: usize, data: *mut u8) {
    use core::arch::x86_64::*;
    
    let ctx = unsafe { &*(data as *const ShaderContext) };
    let shader = ctx.shader;
    let fb = ctx.framebuffer;
    let width = ctx.width as usize;
    let height = ctx.height;
    let stride = ctx.stride as usize;
    
    for y in start..end {
        let row_offset = y * stride;
        
        // Process 4 pixels at a time using SIMD
        let mut x = 0;
        while x + 4 <= width {
            // Call shader for 4 adjacent pixels
            let mut colors = [0u32; 4];
            
            for i in 0..4 {
                let input = PixelInput {
                    x: (x + i) as u32,
                    y: y as u32,
                    width: width as u32,
                    height,
                    time: ctx.time,
                    frame: ctx.frame,
                };
                colors[i] = shader(input).to_u32();
            }
            
            // Store 4 pixels at once using SSE2
            unsafe {
                let pixels = _mm_loadu_si128(colors.as_ptr() as *const __m128i);
                _mm_storeu_si128(fb.add(row_offset + x) as *mut __m128i, pixels);
            }
            
            x += 4;
        }
        
        // Handle remaining pixels
        while x < width {
            let input = PixelInput {
                x: x as u32,
                y: y as u32,
                width: width as u32,
                height,
                time: ctx.time,
                frame: ctx.frame,
            };
            unsafe {
                *fb.add(row_offset + x) = shader(input).to_u32();
            }
            x += 1;
        }
    }
}

// ============================================================================
// BUILT-IN SHADERS - Demo effects
// ============================================================================

/// Plasma shader - classic demoscene effect
pub fn shader_plasma(input: PixelInput) -> PixelOutput {
    let x = input.x as f32 / input.width as f32;
    let y = input.y as f32 / input.height as f32;
    let t = input.time;
    
    // Plasma formula using sin waves
    let v1 = fast_sin(x * 10.0 + t);
    let v2 = fast_sin(y * 10.0 + t * 1.5);
    let v3 = fast_sin((x + y) * 5.0 + t * 0.7);
    let v4 = fast_sin(fast_sqrt((x - 0.5) * (x - 0.5) + (y - 0.5) * (y - 0.5)) * 10.0 - t * 2.0);
    
    let v = (v1 + v2 + v3 + v4) / 4.0;
    
    // Map to colors
    let r = ((v + 1.0) * 0.5 * 255.0) as u8;
    let g = ((fast_sin(v * 3.14159 + t) + 1.0) * 0.5 * 255.0) as u8;
    let b = ((fast_sin(v * 3.14159 * 2.0 + t * 1.3) + 1.0) * 0.5 * 255.0) as u8;
    
    PixelOutput::from_rgb(r, g, b)
}

/// Matrix rain shader - Classic digital rain with glyph simulation
pub fn shader_matrix_rain(input: PixelInput) -> PixelOutput {
    let x = input.x;
    let y = input.y;
    let t = input.time;
    let w = input.width;
    let h = input.height;
    
    // Grid: 8x16 pixel cells (like characters)
    let cell_w = 8u32;
    let cell_h = 16u32;
    let col = x / cell_w;
    let row = y / cell_h;
    let local_x = x % cell_w;
    let local_y = y % cell_h;
    
    // Each column has unique parameters (using hash)
    let col_seed = col.wrapping_mul(2654435761);
    let col_hash = (col_seed & 0xFFFF) as f32 / 65535.0;
    let col_hash2 = ((col_seed >> 8) & 0xFFFF) as f32 / 65535.0;
    
    // Column properties
    let speed = 3.0 + col_hash * 8.0;           // Fall speed 3-11
    let offset = col_hash2 * 50.0;               // Start offset
    let trail_len = 8.0 + col_hash * 12.0;       // Trail length 8-20 cells
    
    // Head position (in cell units)
    let head_row = ((t * speed + offset) % ((h / cell_h + 30) as f32)) as i32 - 15;
    let row_i = row as i32;
    
    // Distance from head
    let dist = head_row - row_i;
    
    if dist < 0 || dist > trail_len as i32 {
        // Not in trail
        return PixelOutput::from_rgb(0, 0, 0);
    }
    
    // Calculate brightness based on position in trail
    let trail_pos = dist as f32 / trail_len;
    let brightness = if dist == 0 {
        1.0  // Head is brightest
    } else {
        (1.0 - trail_pos) * 0.7
    };
    
    // Glyph simulation: each cell has a "character" (pseudo-random pattern)
    let cell_seed = (col.wrapping_mul(31337) ^ row.wrapping_mul(7919) ^ (input.frame / 3)) as f32;
    let glyph_bit = pseudo_glyph(local_x, local_y, cell_seed as u32);
    
    if !glyph_bit {
        // Background of glyph cell - dim glow
        let glow = brightness * 0.15;
        let g = (glow * 255.0) as u8;
        return PixelOutput::from_rgb(0, g / 2, g / 4);
    }
    
    // Foreground of glyph
    let base_g = brightness * 255.0;
    let r = if dist == 0 { (base_g * 0.8) as u8 } else { 0 };  // Head has white tint
    let g = base_g as u8;
    let b = if dist == 0 { (base_g * 0.8) as u8 } else { (base_g * 0.2) as u8 };
    
    PixelOutput::from_rgb(r, g, b)
}

/// Generate pseudo-random glyph pattern (simulates Matrix characters)
#[inline]
fn pseudo_glyph(lx: u32, ly: u32, seed: u32) -> bool {
    // Simple hash to determine if this pixel is "on" in the glyph
    let hash = seed
        .wrapping_mul(2654435761)
        .wrapping_add(lx.wrapping_mul(7919))
        .wrapping_add(ly.wrapping_mul(31337));
    
    // Create patterns that look like katakana/symbols
    let pattern_type = (seed / 7) % 8;
    
    match pattern_type {
        0 => {
            // Horizontal bars
            ly % 4 < 2 && lx > 1 && lx < 6
        },
        1 => {
            // Vertical line with branches
            lx == 3 || lx == 4 || (ly % 5 == 0 && lx > 1)
        },
        2 => {
            // Box shape
            (ly == 2 || ly == 13) && lx > 1 && lx < 6 ||
            (lx == 2 || lx == 5) && ly > 2 && ly < 13
        },
        3 => {
            // Diagonal
            let diff = if lx > ly / 2 { lx - ly / 2 } else { ly / 2 - lx };
            diff < 2
        },
        4 => {
            // Cross
            (lx == 3 || lx == 4) && ly > 2 && ly < 14 ||
            (ly == 7 || ly == 8) && lx > 0 && lx < 7
        },
        5 => {
            // Scattered dots (noise)
            (hash % 3) == 0 && lx > 0 && lx < 7 && ly > 1 && ly < 14
        },
        6 => {
            // Triangle
            let mid = 4i32;
            let row_width = (ly as i32 - 2).max(0).min(6);
            let dist_from_mid = (lx as i32 - mid).abs();
            ly > 2 && ly < 14 && dist_from_mid <= row_width / 2
        },
        _ => {
            // Filled block with gaps
            lx > 0 && lx < 7 && ly > 1 && ly < 14 && (lx + ly) % 3 != 0
        }
    }
}

/// Mandelbrot fractal shader
pub fn shader_mandelbrot(input: PixelInput) -> PixelOutput {
    let zoom = 2.5 + fast_sin(input.time * 0.3) * 0.5;
    let cx = (input.x as f32 / input.width as f32 - 0.7) * zoom;
    let cy = (input.y as f32 / input.height as f32 - 0.5) * zoom;
    
    let mut zx = 0.0f32;
    let mut zy = 0.0f32;
    let mut iter = 0u32;
    const MAX_ITER: u32 = 64;
    
    while zx * zx + zy * zy < 4.0 && iter < MAX_ITER {
        let tmp = zx * zx - zy * zy + cx;
        zy = 2.0 * zx * zy + cy;
        zx = tmp;
        iter += 1;
    }
    
    if iter == MAX_ITER {
        PixelOutput::from_rgb(0, 0, 0)
    } else {
        let t = iter as f32 / MAX_ITER as f32;
        let r = (t * 255.0) as u8;
        let g = (fast_fract(t * 2.0) * 255.0) as u8;
        let b = ((1.0 - t) * 255.0) as u8;
        PixelOutput::from_rgb(r, g, b)
    }
}

/// Gradient shader (simple test)
pub fn shader_gradient(input: PixelInput) -> PixelOutput {
    let r = (input.x * 255 / input.width) as u8;
    let g = (input.y * 255 / input.height) as u8;
    let b = ((input.time * 50.0) as u32 % 256) as u8;
    PixelOutput::from_rgb(r, g, b)
}

/// Fire shader - classic demoscene fire effect
pub fn shader_fire(input: PixelInput) -> PixelOutput {
    let x = input.x as f32;
    let y = input.y as f32;
    let h = input.height as f32;
    let t = input.time;
    
    // Fire intensity based on y position and noise
    let noise1 = fast_sin(x * 0.1 + t * 3.0) * 0.5 + 0.5;
    let noise2 = fast_sin(x * 0.17 + t * 2.3) * 0.5 + 0.5;
    let noise3 = fast_sin(x * 0.23 + y * 0.1 + t * 1.7) * 0.5 + 0.5;
    
    let base_heat = 1.0 - (y / h);
    let heat = base_heat * (0.5 + noise1 * 0.2 + noise2 * 0.2 + noise3 * 0.1);
    let heat = heat.max(0.0).min(1.0);
    
    // Fire color gradient: black -> red -> orange -> yellow -> white
    let (r, g, b) = if heat < 0.2 {
        let t = heat / 0.2;
        ((t * 128.0) as u8, 0, 0)
    } else if heat < 0.5 {
        let t = (heat - 0.2) / 0.3;
        (128 + (t * 127.0) as u8, (t * 100.0) as u8, 0)
    } else if heat < 0.8 {
        let t = (heat - 0.5) / 0.3;
        (255, 100 + (t * 155.0) as u8, (t * 50.0) as u8)
    } else {
        let t = (heat - 0.8) / 0.2;
        (255, 255, 50 + (t * 205.0) as u8)
    };
    
    PixelOutput::from_rgb(r, g, b)
}

// ═══════════════════════════════════════════════════════════════════════════════
// HOLOMATRIX TUNNEL - 3D perspective Matrix effect (flying through the code)
// ═══════════════════════════════════════════════════════════════════════════════

/// Holographic Matrix tunnel - 3D flying through the digital rain
/// Creates the illusion of moving forward through a tunnel of glyphs
pub fn shader_holomatrix_tunnel(input: PixelInput) -> PixelOutput {
    let w = input.width as f32;
    let h = input.height as f32;
    let t = input.time;
    
    // Center-relative coordinates normalized to [-1, 1]
    let cx = (input.x as f32 - w * 0.5) / (h * 0.5);  // aspect-corrected
    let cy = (input.y as f32 - h * 0.5) / (h * 0.5);
    
    // Polar coordinates from center
    let radius = fast_sqrt(cx * cx + cy * cy).max(0.001);
    let angle = fast_atan2(cy, cx);
    
    // === TUNNEL DEPTH ===
    // Inverse radius = depth (center is far, edges are near)
    let depth = 1.0 / radius;
    
    // Forward motion - depth increases with time
    let z = depth + t * 2.5;
    
    // === TUNNEL WALL COORDINATES ===
    // Map angle to X position on tunnel wall (0 to 1, wrapping)
    let wall_x = (angle / 6.28318 + 0.5) % 1.0;
    let wall_x = if wall_x < 0.0 { wall_x + 1.0 } else { wall_x };
    
    // Z depth for Y position on wall (scrolling forward)
    let wall_y = z % 1.0;
    
    // === GRID CELLS (Glyph positions) ===
    let cell_size_x = 0.08;
    let cell_size_y = 0.12;  // Taller cells for glyphs
    
    let cell_x = (wall_x / cell_size_x) as u32;
    let cell_y = (z / cell_size_y) as u32;
    let local_x = (wall_x % cell_size_x) / cell_size_x;
    let local_y = (wall_y / cell_size_y) % 1.0;
    
    // === GLYPH PATTERN ===
    let cell_seed = cell_x.wrapping_mul(31337) ^ cell_y.wrapping_mul(7919);
    let glyph_visible = tunnel_glyph_pattern(local_x, local_y, cell_seed);
    
    // === DEPTH FOG / BRIGHTNESS ===
    // Near (high radius) = bright, Far (low radius / center) = dim
    let fog = (radius * 1.5).min(1.0);
    let depth_pulse = (fast_sin(z * 3.0 + t * 4.0) * 0.15 + 0.85);
    let brightness = fog * depth_pulse;
    
    // === SCANLINE EFFECT ===
    let scanline = if (input.y % 3) == 0 { 0.85 } else { 1.0 };
    
    // === HOLOGRAPHIC GRID LINES ===
    let grid_intensity = tunnel_grid_lines(wall_x, z, t);
    
    // === COLOR COMPOSITION ===
    let base_intensity = if glyph_visible {
        brightness * scanline
    } else {
        brightness * 0.08 * scanline  // Dim background
    };
    
    // Add grid glow
    let total_intensity = (base_intensity + grid_intensity * 0.4).min(1.0);
    
    // Matrix green with holographic tint
    // Closer = more cyan/white, farther = darker green
    let depth_hue = (1.0 - fog) * 0.3;  // Hue shift toward cyan at distance
    
    let r = (total_intensity * (80.0 + depth_hue * 100.0) * brightness) as u8;
    let g = (total_intensity * 255.0) as u8;
    let b = (total_intensity * (60.0 + depth_hue * 150.0 + grid_intensity * 80.0)) as u8;
    
    // Head glow (cells at certain Z depths flash white)
    let flash_phase = (z * 8.0 + t * 10.0) % 1.0;
    if glyph_visible && flash_phase < 0.05 && radius < 0.8 {
        let flash = (1.0 - flash_phase / 0.05) * brightness;
        let fr = (r as f32 + flash * 200.0).min(255.0) as u8;
        let fg = (g as f32 + flash * 50.0).min(255.0) as u8;
        let fb = (b as f32 + flash * 200.0).min(255.0) as u8;
        return PixelOutput::from_rgb(fr, fg, fb);
    }
    
    PixelOutput::from_rgb(r, g, b)
}

/// Create glyph-like patterns for tunnel walls
#[inline]
fn tunnel_glyph_pattern(lx: f32, ly: f32, seed: u32) -> bool {
    let pattern = seed % 12;
    let px = (lx * 8.0) as u32;
    let py = (ly * 12.0) as u32;
    
    match pattern {
        0 => py > 2 && py < 10 && (px == 2 || px == 5),  // ||
        1 => py == 3 || py == 8 || (px == 4 && py > 2 && py < 10),  // =|=
        2 => (px + py) % 3 == 0,  // Diagonal scatter
        3 => px > 1 && px < 6 && (py == 2 || py == 9),  // Top/bottom bars
        4 => (px == 3 || px == 4) && py > 1 && py < 11,  // Vertical line
        5 => py > 2 && py < 10 && px > 1 && px < 6 && (py - 2) % 2 == 0,  // Horizontal stripes
        6 => {  // Box outline
            (py == 2 || py == 9) && px > 1 && px < 6 ||
            (px == 2 || px == 5) && py > 2 && py < 9
        },
        7 => px == 3 && py > 1 && py < 11 || py == 6 && px > 0 && px < 7,  // Cross
        8 => (px + py / 2) % 4 == 0,  // Diagonal lines
        9 => py > 3 && py < 9 && ((px > 1 && px < 4) || (px > 4 && px < 7)),  // Two columns
        10 => {  // Triangle
            let center = 3.5;
            let dist = if px as f32 > center { px as f32 - center } else { center - px as f32 };
            py > 2 && py < 10 && dist < (py - 2) as f32 * 0.4
        },
        _ => (seed.wrapping_mul(px) ^ py) % 3 == 0,  // Random dots
    }
}

/// Perspective grid lines for holographic effect
#[inline]
fn tunnel_grid_lines(wall_x: f32, z: f32, t: f32) -> f32 {
    // Radial lines (converging to center)
    let radial_divisions = 16.0;
    let radial_pos = wall_x * radial_divisions;
    let radial_line = fast_abs(radial_pos - (radial_pos as i32) as f32 - 0.5);
    let radial_glow = if radial_line < 0.08 { (0.08 - radial_line) / 0.08 } else { 0.0 };
    
    // Depth rings (horizontal lines in tunnel)
    let ring_spacing = 0.3;
    let ring_z = z / ring_spacing;
    let ring_fract = ring_z - (ring_z as i32) as f32;
    let ring_line = fast_abs(ring_fract - 0.5);
    let ring_glow = if ring_line < 0.05 { (0.05 - ring_line) / 0.05 * 0.5 } else { 0.0 };
    
    // Pulse effect on rings
    let pulse = fast_sin(z * 2.0 - t * 8.0) * 0.3 + 0.7;
    
    (radial_glow * 0.6 + ring_glow * pulse) * 0.8
}

/// Fast atan2 approximation for angle calculation
#[inline(always)]
fn fast_atan2(y: f32, x: f32) -> f32 { crate::math::fast_atan2(y, x) }

// ════════════════════════════════════════════════════════════════════════════════
// HOLOMATRIX DEPTH LAYERS - Multiple parallax layers of falling glyphs
// ════════════════════════════════════════════════════════════════════════════════

/// Multi-layer parallax Matrix rain with depth
pub fn shader_holomatrix_parallax(input: PixelInput) -> PixelOutput {
    let x = input.x;
    let y = input.y;
    let t = input.time;
    let h = input.height;
    
    // Accumulate color from 4 depth layers
    let mut total_r = 0.0f32;
    let mut total_g = 0.0f32;
    let mut total_b = 0.0f32;
    
    // Layer 0: Far background (slow, dim, small)
    let (r0, g0, b0) = parallax_layer(x, y, t, h, 0.4, 0.15, 6, 12, 0);
    total_r += r0 * 0.3;
    total_g += g0 * 0.3;
    total_b += b0 * 0.5;  // More blue/cyan tint for depth
    
    // Layer 1: Mid-far (medium speed)
    let (r1, g1, b1) = parallax_layer(x, y, t, h, 0.7, 0.35, 7, 14, 100);
    total_r += r1 * 0.5;
    total_g += g1 * 0.5;
    total_b += b1 * 0.35;
    
    // Layer 2: Mid-near (faster)
    let (r2, g2, b2) = parallax_layer(x, y, t, h, 1.0, 0.65, 8, 16, 200);
    total_r += r2 * 0.7;
    total_g += g2 * 0.7;
    total_b += b2 * 0.25;
    
    // Layer 3: Foreground (fastest, brightest, largest)
    let (r3, g3, b3) = parallax_layer(x, y, t, h, 1.5, 1.0, 10, 20, 300);
    total_r += r3;
    total_g += g3;
    total_b += b3 * 0.2;
    
    // Scanlines
    let scanline = if (y % 2) == 0 { 0.92 } else { 1.0 };
    
    let r = (total_r * scanline).min(255.0) as u8;
    let g = (total_g * scanline).min(255.0) as u8;
    let b = (total_b * scanline).min(60.0) as u8;  // Keep it green-dominant
    
    PixelOutput::from_rgb(r, g, b)
}

/// Single parallax layer of matrix rain
fn parallax_layer(x: u32, y: u32, t: f32, h: u32, speed: f32, brightness: f32, 
                  cell_w: u32, cell_h: u32, seed_offset: u32) -> (f32, f32, f32) {
    let col = x / cell_w;
    let row = y / cell_h;
    let local_x = x % cell_w;
    let local_y = y % cell_h;
    
    // Column-based parameters
    let col_seed = col.wrapping_add(seed_offset).wrapping_mul(2654435761);
    let col_hash = (col_seed & 0xFFFF) as f32 / 65535.0;
    
    let col_speed = speed * (0.7 + col_hash * 0.6);
    let col_offset = ((col_seed >> 16) & 0xFFFF) as f32 / 65535.0 * 50.0;
    let trail_len = 12.0 + col_hash * 20.0;
    
    // Head position
    let head_row = ((t * col_speed * 8.0 + col_offset) % ((h / cell_h + 40) as f32)) as i32 - 20;
    let row_i = row as i32;
    let dist = head_row - row_i;
    
    if dist < 0 || dist > trail_len as i32 {
        return (0.0, 0.0, 0.0);
    }
    
    // Brightness based on trail position
    let trail_pos = dist as f32 / trail_len;
    let intensity = if dist == 0 {
        brightness * 255.0
    } else {
        brightness * (1.0 - trail_pos * trail_pos) * 180.0
    };
    
    // Glyph pattern
    let cell_seed = col.wrapping_mul(31337) ^ row.wrapping_mul(7919) ^ seed_offset;
    if !tunnel_glyph_pattern(local_x as f32 / cell_w as f32, 
                              local_y as f32 / cell_h as f32, cell_seed) {
        return (intensity * 0.05, intensity * 0.1, intensity * 0.03);
    }
    
    // Head flash
    let (r, g, b) = if dist == 0 {
        (intensity * 0.9, intensity, intensity * 0.9)  // White flash
    } else if dist < 3 {
        (intensity * 0.3, intensity, intensity * 0.1)  // Bright green
    } else {
        (0.0, intensity, intensity * 0.05)  // Pure matrix green
    };
    
    (r, g, b)
}

// ============================================================================
// MATH UTILITIES (Fast approximations for shaders)
// ============================================================================

/// Fast sine approximation (Taylor series, accurate to ~0.001)
/// Fast square root (delegates to shared math)
#[inline]
fn fast_sqrt(x: f32) -> f32 { crate::math::fast_sqrt(x) }

/// Fast absolute value
#[inline(always)]
fn fast_abs(x: f32) -> f32 {
    if x < 0.0 { -x } else { x }
}

/// Fast fractional part
#[inline(always)]
fn fast_fract(x: f32) -> f32 {
    x - (x as i32) as f32
}

// ═══════════════════════════════════════════════════════════════════════════════
// MATRIX SHAPES - 3D Objects emerging through the Matrix rain
// ═══════════════════════════════════════════════════════════════════════════════

/// Shader: 3D shapes floating through Matrix code rain
/// Creates cubes, spheres and tori that carve through the digital rain
pub fn shader_matrix_shapes(input: PixelInput) -> PixelOutput {
    let w = input.width as f32;
    let h = input.height as f32;
    let t = input.time;
    
    // Normalized coordinates centered at screen middle
    let u = (input.x as f32 / w - 0.5) * 2.0 * (w / h);  // aspect corrected
    let v = (input.y as f32 / h - 0.5) * 2.0;
    
    // Camera setup - looking down Z axis
    let cam_z = -3.0;
    let ray_dir_x = u;
    let ray_dir_y = v;
    let ray_dir_z = 1.5;  // focal length
    
    // Normalize ray direction
    let ray_len = fast_sqrt(ray_dir_x * ray_dir_x + ray_dir_y * ray_dir_y + ray_dir_z * ray_dir_z);
    let rd_x = ray_dir_x / ray_len;
    let rd_y = ray_dir_y / ray_len;
    let rd_z = ray_dir_z / ray_len;
    
    // Ray origin
    let ro_x = 0.0;
    let ro_y = 0.0;
    let ro_z = cam_z;
    
    // === RAY MARCH to find shapes === (reduced iterations for speed)
    let mut dist = 0.0f32;
    let mut hit_shape = 0u8;  // 0=none, 1=cube, 2=sphere, 3=torus
    let mut hit_pos = (0.0f32, 0.0f32, 0.0f32);
    
    for _step in 0..24 {  // Reduced from 64 to 24 for performance
        let px = ro_x + rd_x * dist;
        let py = ro_y + rd_y * dist;
        let pz = ro_z + rd_z * dist;
        
        // Get distance to all shapes
        let (shape_dist, shape_id) = sdf_scene(px, py, pz, t);
        
        if shape_dist < 0.02 {  // Slightly increased threshold
            hit_shape = shape_id;
            hit_pos = (px, py, pz);
            break;
        }
        
        dist += shape_dist;
        if dist > 20.0 { break; }
    }
    
    // === BACKGROUND: Matrix rain ===
    let (bg_r, bg_g, bg_b) = matrix_rain_background(input.x, input.y, w, h, t);
    
    if hit_shape == 0 {
        // No hit - show Matrix rain
        return PixelOutput::from_rgb(bg_r, bg_g, bg_b);
    }
    
    // === SIMPLIFIED SHAPE LIGHTING (no normal calculation for speed) ===
    // Use depth-based shading instead of true normals
    let depth_factor = (1.0 - (dist - 1.0) / 10.0).clamp(0.2, 1.0);
    let diffuse = depth_factor;
    
    // Simple edge detection based on position
    let edge_x = fast_abs(hit_pos.0) > 0.4;
    let edge_y = fast_abs(hit_pos.1) > 0.4;
    let edge_glow = if edge_x || edge_y { 0.5 } else { 0.0 };
    
    // Depth fog (farther = more Matrix visible through)
    let fog = ((dist - 1.0) / 6.0).clamp(0.0, 0.6);
    
    // Shape colors with Matrix tint
    let (shape_r, shape_g, shape_b) = match hit_shape {
        1 => {  // Cube - cyan/green glow
            let r = (80.0 * diffuse + edge_glow * 200.0) as u8;
            let g = (255.0 * diffuse + edge_glow * 100.0) as u8;
            let b = (180.0 * diffuse + edge_glow * 255.0) as u8;
            (r, g, b)
        },
        2 => {  // Sphere - glowing green orb
            let pulse = fast_sin(t * 3.0) * 0.2 + 0.8;
            let r = (60.0 * diffuse * pulse) as u8;
            let g = (255.0 * diffuse * pulse) as u8;
            let b = (100.0 * diffuse * pulse) as u8;
            (r, g, b)
        },
        _ => (bg_r, bg_g, bg_b)
    };
    
    // Blend shape with Matrix background (transparency effect)
    let opacity = 1.0 - fog;
    let r = lerp_u8(bg_r, shape_r, opacity);
    let g = lerp_u8(bg_g, shape_g, opacity);
    let b = lerp_u8(bg_b, shape_b, opacity);
    
    // Scanlines
    let scan = if input.y % 3 == 0 { 0.9 } else { 1.0 };
    let r = ((r as f32) * scan) as u8;
    let g = ((g as f32) * scan) as u8;
    let b = ((b as f32) * scan) as u8;
    
    PixelOutput::from_rgb(r, g, b)
}

/// SDF scene - all shapes combined (simplified for performance)
#[inline(always)]
fn sdf_scene(x: f32, y: f32, z: f32, t: f32) -> (f32, u8) {
    // Single rotating cube at center
    let cube_z = 2.0 + fast_sin(t * 0.5) * 0.3;
    
    // Rotate point around cube center
    let angle_y = t * 0.5;
    let (rx, ry, rz) = rotate_point(x, y, z - cube_z, 0.0, angle_y);
    let cube_dist = sdf_cube(rx, ry, rz, 0.6);
    
    // Orbiting sphere
    let sphere_x = fast_cos(t * 0.7) * 1.0;
    let sphere_y = fast_sin(t * 0.5) * 0.6;
    let sphere_z = 2.5;
    let sphere_dist = sdf_sphere(x - sphere_x, y - sphere_y, z - sphere_z, 0.4);
    
    // Find closest shape
    if cube_dist < sphere_dist {
        (cube_dist, 1)
    } else {
        (sphere_dist, 2)
    }
}

/// SDF Sphere
#[inline(always)]
fn sdf_sphere(x: f32, y: f32, z: f32, r: f32) -> f32 {
    fast_sqrt(x * x + y * y + z * z) - r
}

/// SDF Cube (box)
#[inline(always)]
fn sdf_cube(x: f32, y: f32, z: f32, s: f32) -> f32 {
    let dx = fast_abs(x) - s;
    let dy = fast_abs(y) - s;
    let dz = fast_abs(z) - s;
    
    let outside = fast_sqrt(
        dx.max(0.0) * dx.max(0.0) + 
        dy.max(0.0) * dy.max(0.0) + 
        dz.max(0.0) * dz.max(0.0)
    );
    let inside = dx.max(dy).max(dz).min(0.0);
    outside + inside
}

/// SDF Torus
#[inline(always)]
fn sdf_torus(x: f32, y: f32, z: f32, r_major: f32, r_minor: f32) -> f32 {
    let q = fast_sqrt(x * x + z * z) - r_major;
    fast_sqrt(q * q + y * y) - r_minor
}

/// Rotate point in 3D
#[inline(always)]
fn rotate_point(x: f32, y: f32, z: f32, ax: f32, ay: f32) -> (f32, f32, f32) {
    // Rotate around Y
    let cos_y = fast_cos(ay);
    let sin_y = fast_sin(ay);
    let x2 = x * cos_y - z * sin_y;
    let z2 = x * sin_y + z * cos_y;
    
    // Rotate around X
    let cos_x = fast_cos(ax);
    let sin_x = fast_sin(ax);
    let y2 = y * cos_x - z2 * sin_x;
    let z3 = y * sin_x + z2 * cos_x;
    
    (x2, y2, z3)
}

/// Calculate surface normal via gradient
fn sdf_normal(x: f32, y: f32, z: f32, t: f32) -> (f32, f32, f32) {
    let eps = 0.001;
    let (d, _) = sdf_scene(x, y, z, t);
    let (dx, _) = sdf_scene(x + eps, y, z, t);
    let (dy, _) = sdf_scene(x, y + eps, z, t);
    let (dz, _) = sdf_scene(x, y, z + eps, t);
    
    let nx = dx - d;
    let ny = dy - d;
    let nz = dz - d;
    let len = fast_sqrt(nx * nx + ny * ny + nz * nz).max(0.0001);
    (nx / len, ny / len, nz / len)
}

/// Edge glow for cube wireframe effect
fn edge_glow(x: f32, y: f32, z: f32, _t: f32) -> f32 {
    let edge_thresh = 0.02;
    let ax = fast_abs(x);
    let ay = fast_abs(y);
    let az = fast_abs(z);
    
    // Near edges if two coordinates are close to the cube size
    let on_edge_xy = (fast_abs(ax - 0.5) < edge_thresh) && (fast_abs(ay - 0.5) < edge_thresh);
    let on_edge_xz = (fast_abs(ax - 0.5) < edge_thresh) && (fast_abs(az - 0.5) < edge_thresh);
    let on_edge_yz = (fast_abs(ay - 0.5) < edge_thresh) && (fast_abs(az - 0.5) < edge_thresh);
    
    if on_edge_xy || on_edge_xz || on_edge_yz {
        1.0
    } else {
        0.0
    }
}

/// Matrix rain background (simplified version for compositing)
fn matrix_rain_background(x: u32, y: u32, w: f32, h: f32, t: f32) -> (u8, u8, u8) {
    let col = (x as f32 / 16.0) as u32;
    let row = (y as f32 / 18.0) as u32;
    
    // Column-based random seed
    let seed = col.wrapping_mul(31337) ^ 0xDEAD;
    let fall_speed = 0.3 + ((seed % 100) as f32 / 100.0) * 0.7;
    let phase_offset = (seed % 1000) as f32 / 100.0;
    
    // Falling position
    let fall_pos = (t * fall_speed + phase_offset) % 1.5;
    let row_norm = y as f32 / h;
    
    // Head position (brightest)
    let head_dist = (fall_pos - row_norm).abs();
    let is_head = head_dist < 0.03;
    
    // Trail fades from head
    let trail_length = 0.3;
    let in_trail = row_norm < fall_pos && (fall_pos - row_norm) < trail_length;
    let trail_fade = if in_trail { 1.0 - (fall_pos - row_norm) / trail_length } else { 0.0 };
    
    // Glyph visibility (pseudo-random based on cell)
    let cell_seed = col.wrapping_mul(7919) ^ row.wrapping_mul(31337);
    let glyph_on = (cell_seed % 3) != 0;
    
    if is_head && glyph_on {
        (220, 255, 220)  // White-green head
    } else if in_trail && glyph_on {
        let g = (200.0 * trail_fade) as u8;
        let r = (50.0 * trail_fade) as u8;
        let b = (80.0 * trail_fade) as u8;
        (r, g, b)
    } else {
        (5, 15, 8)  // Dark background
    }
}

/// Linear interpolate between u8 values
#[inline]
fn lerp_u8(a: u8, b: u8, t: f32) -> u8 {
    ((a as f32) * (1.0 - t) + (b as f32) * t) as u8
}

/// Fast sine approximation (delegates to shared math)
#[inline(always)]
fn fast_sin(x: f32) -> f32 { crate::math::fast_sin(x) }

/// Fast cosine (delegates to shared math)
#[inline(always)]
fn fast_cos(x: f32) -> f32 { crate::math::fast_cos(x) }

// ============================================================================
// GLOBAL VIRTUAL GPU INSTANCE
// ============================================================================

use spin::Mutex;

static VGPU: Mutex<VirtualGpu> = Mutex::new(VirtualGpu::new());

/// Initialize the virtual GPU with framebuffer
/// stride = row stride in pixels (width for backbuffer, pitch/4 for MMIO)
pub fn init(framebuffer: *mut u32, width: u32, height: u32) {
    VGPU.lock().init(framebuffer, width, height, width);
    crate::serial_println!("[VGPU] Initialized {}x{} virtual GPU ({} virtual cores)", 
        width, height, VIRTUAL_CORES);
}

/// Initialize with explicit stride
pub fn init_stride(framebuffer: *mut u32, width: u32, height: u32, stride: u32) {
    VGPU.lock().init(framebuffer, width, height, stride);
    crate::serial_println!("[VGPU] Initialized {}x{} stride={} virtual GPU ({} virtual cores)", 
        width, height, stride, VIRTUAL_CORES);
}

/// Set the active shader
pub fn set_shader(shader: PixelShaderFn) {
    VGPU.lock().set_shader(shader);
}

// ═══════════════════════════════════════════════════════════════════════════════
// MATRIX RAIN 3D - Glyphs flying TOWARD the viewer from depth
// ═══════════════════════════════════════════════════════════════════════════════

/// Matrix Rain 3D - Characters spawn far away and fly toward viewer
/// Uses inverse polar coords: center = far, edges = close
pub fn shader_matrix_rain_3d(input: PixelInput) -> PixelOutput {
    let w = input.width as f32;
    let h = input.height as f32;
    let t = input.time;
    
    // Center-relative coords
    let cx = (input.x as f32 - w * 0.5) / (h * 0.5);
    let cy = (input.y as f32 - h * 0.5) / (h * 0.5);
    
    // Polar coords
    let radius = fast_sqrt(cx * cx + cy * cy).max(0.001);
    let angle = fast_atan2(cy, cx);
    
    // Depth = inverse radius (center = far, edges = near)
    let depth = 1.0 / radius;
    
    // Forward motion through time
    let z = depth + t * 3.0;
    
    // === RAIN COLUMNS ===
    // Divide the circle into "columns" based on angle
    let num_columns = 32.0;
    let col_angle = (angle + 3.14159) / 6.28318;  // 0 to 1
    let column = (col_angle * num_columns) as u32;
    let in_column_pos = (col_angle * num_columns) % 1.0;  // Position within column
    
    // Column seed for random offset
    let col_seed = column.wrapping_mul(48271);
    let col_offset = (col_seed % 1000) as f32 / 1000.0 * 10.0;
    
    // === GLYPH GRID IN EACH COLUMN ===
    let glyph_spacing = 0.15;
    let adjusted_z = z + col_offset;
    let glyph_row = (adjusted_z / glyph_spacing) as u32;
    let glyph_local_z = (adjusted_z / glyph_spacing) % 1.0;
    
    // === CALCULATE IF PIXEL IS ON A GLYPH ===
    // Glyph center is when in_column_pos ~ 0.5 and glyph_local_z ~ 0.3 to 0.7
    let col_center_dist = fast_abs(in_column_pos - 0.5);
    let glyph_vertical = glyph_local_z > 0.2 && glyph_local_z < 0.8;
    
    // Column width expands outward (perspective)
    let col_width = 0.3 + radius * 0.2;
    let on_column = col_center_dist < col_width;
    
    // Generate glyph pattern
    let glyph_seed = glyph_row.wrapping_mul(31337) ^ column.wrapping_mul(48271);
    let glyph_pattern = matrix3d_glyph(in_column_pos, glyph_local_z, glyph_seed);
    
    // === BRIGHTNESS based on depth ===
    // Close (high radius) = bright, Far (low radius) = dim
    let fog = (radius * 1.8).min(1.0);
    
    // Head glow - leading edge of rain drops
    let is_head = glyph_local_z < 0.25;
    let trail_fade = if is_head { 1.0 } else { 1.0 - (glyph_local_z - 0.25) / 0.6 };
    
    // === SCANLINES ===
    let scanline = if input.y % 2 == 0 { 0.9 } else { 1.0 };
    
    // === COMPOSE COLOR ===
    if on_column && glyph_vertical && glyph_pattern {
        // On a glyph!
        let intensity = fog * trail_fade * scanline;
        
        if is_head {
            // White/bright green head
            let pulse = fast_sin(t * 8.0 + adjusted_z * 4.0) * 0.2 + 0.8;
            let r = (200.0 * intensity * pulse) as u8;
            let g = (255.0 * intensity) as u8;
            let b = (220.0 * intensity * pulse) as u8;
            PixelOutput::from_rgb(r, g, b)
        } else {
            // Green trail
            let r = (40.0 * intensity) as u8;
            let g = (255.0 * intensity * trail_fade) as u8;
            let b = (80.0 * intensity) as u8;
            PixelOutput::from_rgb(r, g, b)
        }
    } else {
        // Background - faint grid/glow
        let bg_glow = (fog * 0.05 * scanline) as f32;
        let bg = (bg_glow * 40.0) as u8;
        PixelOutput::from_rgb(0, bg, bg / 2)
    }
}

/// Glyph pattern for Matrix Rain 3D
#[inline]
fn matrix3d_glyph(lx: f32, ly: f32, seed: u32) -> bool {
    let px = (lx * 6.0) as u32;
    let py = (ly * 8.0) as u32;
    let pattern = seed % 10;
    
    match pattern {
        0 => px > 1 && px < 5,                              // Vertical bar
        1 => py == 2 || py == 5,                            // Horizontal lines
        2 => (px + py) % 2 == 0,                            // Checkerboard
        3 => px == 3 || py == 4,                            // Cross
        4 => py > 1 && py < 7 && px > 1 && px < 5,          // Block
        5 => (px == 2 || px == 4) && py > 1 && py < 7,      // Parallel bars
        6 => py == 3 || (px == 3 && py > 1 && py < 7),      // T shape
        7 => (px + py / 2) % 3 == 0,                        // Diagonal
        8 => py < 4 && px > 1 && px < 5,                    // Top half
        _ => (seed.wrapping_mul(px + 1) ^ (py + 1)) % 3 == 0, // Random dots
    }
}

/// Dispatch fullscreen shader
pub fn draw() {
    VGPU.lock().dispatch_fullscreen();
}

/// Dispatch with SIMD acceleration
#[cfg(target_arch = "x86_64")]
pub fn draw_simd() {
    VGPU.lock().dispatch_fullscreen_simd();
}

/// Update time
pub fn tick(delta_ms: u32) {
    VGPU.lock().tick(delta_ms);
}

/// Get frame count
pub fn frame() -> u32 {
    VGPU.lock().frame()
}

/// Get elapsed time
pub fn time() -> f32 {
    VGPU.lock().time()
}

// ============================================================================
// SHADER REGISTRY - Named shaders for easy switching
// ============================================================================

/// Cosmic Deformation shader — GLSL-golf style fractal vortex
/// Port of the complex "render_shader_frame" from video/player.rs
/// Iterative cosine deformation with radial/exponential color mapping
pub fn shader_cosmic_deform(input: PixelInput) -> PixelOutput {
    let w = input.width as f32;
    let h = input.height as f32;
    let t = input.time;
    
    // Normalized coords centered on screen
    let p_x = (input.x as f32 * 2.0 - w) / h;
    let p_y = (input.y as f32 * 2.0 - h) / h;
    
    let dot_pp = p_x * p_x + p_y * p_y;
    let l = fast_abs(0.7 - dot_pp);
    
    let s = (1.0 - l) * 5.0;
    let mut vx = p_x * s;
    let mut vy = p_y * s;
    
    let mut o_r: f32 = 0.0;
    let mut o_g: f32 = 0.0;
    let mut o_b: f32 = 0.0;
    
    // 6 iterations of cosine deformation
    let mut i: f32 = 1.0;
    while i <= 6.0 {
        let inv_i = 1.0 / i;
        vx += fast_cos(vy * i + t) * inv_i + 0.7;
        vy += fast_cos(vx * i + i + t) * inv_i + 0.7;
        
        let diff = fast_abs(vx - vy) * 0.2;
        o_r += (fast_sin(vx) + 1.0) * diff;
        o_g += (fast_sin(vy) + 1.0) * diff;
        o_b += (fast_sin(vy) + 1.0) * diff;
        i += 1.0;
    }
    
    // Radial + exponential color mapping
    let radial = fast_exp(-4.0 * l);
    let e_py1 = fast_exp(p_y);
    let e_pyn1 = fast_exp(-p_y);
    let e_pyn2 = fast_exp(p_y * -2.0);
    
    let fr = fast_tanh(e_py1 * radial / (o_r + 0.001));
    let fg = fast_tanh(e_pyn1 * radial / (o_g + 0.001));
    let fb = fast_tanh(e_pyn2 * radial / (o_b + 0.001));
    
    let r = (fast_abs(fr) * 255.0).min(255.0) as u8;
    let g = (fast_abs(fg) * 255.0).min(255.0) as u8;
    let b = (fast_abs(fb) * 255.0).min(255.0) as u8;
    PixelOutput::from_rgb(r, g, b)
}

/// Approximate tanh for shader math
#[inline(always)]
fn fast_tanh(x: f32) -> f32 {
    let x2 = x * x;
    x / (1.0 + x.abs() + x2 * 0.28)
}

/// Approximate exp for shader math (fast, rough)
#[inline(always)]
fn fast_exp(x: f32) -> f32 {
    let x = x.clamp(-10.0, 10.0);
    let t = 1.0 + x / 256.0;
    let mut r = t;
    // 8 squarings: (1+x/256)^256 ≈ e^x
    r = r * r; r = r * r; r = r * r; r = r * r;
    r = r * r; r = r * r; r = r * r; r = r * r;
    r
}

/// Get shader by name
pub fn get_shader(name: &str) -> Option<PixelShaderFn> {
    match name.to_lowercase().as_str() {
        "plasma" => Some(shader_plasma),
        "matrix" | "rain" => Some(shader_matrix_rain),
        "mandelbrot" | "fractal" => Some(shader_mandelbrot),
        "gradient" | "test" => Some(shader_gradient),
        "fire" => Some(shader_fire),
        "tunnel" | "holotunnel" | "3d" => Some(shader_holomatrix_tunnel),
        "parallax" | "holoparallax" | "depth" => Some(shader_holomatrix_parallax),
        "shapes" | "objects" | "cubes" | "matrix3dshapes" => Some(shader_matrix_shapes),
        "rain3d" | "matrix3d" | "matrixrain3d" | "fly" => Some(shader_matrix_rain_3d),
        "cosmic" | "deform" | "vortex" | "complex" => Some(shader_cosmic_deform),
        _ => None,
    }
}

/// List available shaders
pub fn list_shaders() -> &'static [&'static str] {
    &["plasma", "matrix", "mandelbrot", "gradient", "fire", "tunnel", "parallax", "shapes", "rain3d", "cosmic"]
}

// ============================================================================
// OPTIMIZED MATRIX RAIN ENGINE — Cell-based, Depth-parallax, SMP-ready
// ============================================================================
//
// WHY THIS EXISTS:
// The per-pixel shader approach calls shader_matrix_rain() ~1,024,000 times
// per frame (1280×800). Each call does floating-point hash, division, modulo.
// Matrix rain is 90%+ black background — massive waste.
//
// THIS ENGINE:
// - Only touches active trail cells (~12,000 glyph blits vs 1M shader calls)
// - Uses real MATRIX_GLYPHS_6X6 (sharp katakana, not pseudo_glyph noise)
// - Pre-computed COLOR_LUT for O(1) color mapping (zero float in render)
// - Per-column depth → parallax (brightness, speed, density variation)
// - SMP parallel by column bands (no false sharing)
// - SSE2 background fill
// - Result: 20-60× faster than per-pixel shader
//
// ============================================================================

/// Cell size in pixels (glyphs are 6×6 drawn inside 8×8 cells for spacing)
const MCELL: usize = 8;
/// Max grid columns (1920/8 = 240)
const MGRID_COLS: usize = 240;
/// Max grid rows (1200/8 = 150)
const MGRID_ROWS: usize = 150;
/// Rain drops per column (controls density)
const MDROPS: usize = 4;
/// Trail length bounds
const MTRAIL_MIN: usize = 10;
const MTRAIL_MAX: usize = 45;
/// Number of glyphs in shared table
const MNUM_GLYPHS: usize = 64;

/// Intensity LUT for trail fade (64 entries, never reaches 0)
static MINT_LUT: [u8; 64] = [
    255, 250, 244, 238, 232, 225, 218, 211,
    204, 196, 189, 181, 174, 166, 158, 150,
    143, 135, 128, 121, 114, 107, 100,  94,
     88,  82,  76,  71,  66,  61,  56,  52,
     48,  44,  40,  37,  34,  31,  28,  26,
     24,  22,  20,  18,  16,  15,  14,  13,
     12,  11,  10,   9,   8,   7,   6,   5,
      5,   4,   4,   3,   3,   3,   2,   2,
];

/// Pre-computed green color LUT: intensity → 0xAARRGGBB
/// Creates the iconic Matrix look: white head → bright lime → green → teal → dark
const fn gen_matrix_color_lut() -> [u32; 256] {
    let mut lut = [0xFF010201u32; 256]; // index 0 = near-black
    let mut i = 1u32;
    while i < 256 {
        let c = if i > 250 {
            // White-green head flash
            let w = 200 + ((i - 250) * 10) as u32;
            let w = if w > 255 { 255 } else { w };
            (0xFF << 24) | (w << 16) | (255 << 8) | w
        } else if i > 200 {
            // Bright lime
            let f = i - 200;
            let r = f * 3 / 2;
            let g = 200 + f;
            let b = f / 2;
            (0xFF << 24) | (r << 16) | (g << 8) | b
        } else if i > 140 {
            // Matrix green
            let g = 130 + (i - 140) * 7 / 6;
            let r = (i - 140) / 6;
            let b = (i - 140) / 8;
            (0xFF << 24) | (r << 16) | (g << 8) | b
        } else if i > 80 {
            // Teal-green (depth layer)
            let g = 60 + (i - 80) * 7 / 6;
            let b = (i - 80) / 4;
            (0xFF << 24) | (g << 8) | b
        } else if i > 30 {
            // Dark green
            let g = 20 + (i - 30) * 4 / 5;
            (0xFF << 24) | (g << 8)
        } else if i > 10 {
            // Very dark green (still visible for depth)
            let g = 6 + (i - 10) * 7 / 10;
            (0xFF << 24) | (g << 8)
        } else {
            // Minimum glow (never pure black for depth fog)
            let g = 2 + i / 2;
            (0xFF << 24) | (g << 8)
        };
        lut[i as usize] = c;
        i += 1;
    }
    lut
}

static MCOLOR_LUT: [u32; 256] = gen_matrix_color_lut();

/// O(1) color lookup
#[inline(always)]
fn mcolor(intensity: u8) -> u32 {
    MCOLOR_LUT[intensity as usize]
}

/// Pre-computed resonance glow LUT: intensity → 0xAARRGGBB
/// Locked/resonant glyphs emit cyan-white "data alignment" glow
const fn gen_matrix_glow_lut() -> [u32; 256] {
    let mut lut = [0xFF010201u32; 256];
    let mut i = 1u32;
    while i < 256 {
        // Cyan-white tint: more R and B than normal green LUT
        let c = if i > 240 {
            // Pure white flash for very bright resonance
            let w = 220 + ((i - 240) * 2);
            let w = if w > 255 { 255 } else { w };
            (0xFF << 24) | (w << 16) | (255 << 8) | w
        } else if i > 180 {
            // Bright cyan-white
            let f = i - 180;
            let r = 100 + f;
            let g = 200 + f / 2;
            let b = 140 + f;
            let r = if r > 255 { 255 } else { r };
            let g = if g > 255 { 255 } else { g };
            let b = if b > 255 { 255 } else { b };
            (0xFF << 24) | (r << 16) | (g << 8) | b
        } else if i > 120 {
            // Cyan-green
            let f = i - 120;
            let r = 30 + f / 2;
            let g = 130 + f;
            let b = 60 + f;
            (0xFF << 24) | (r << 16) | (g << 8) | b
        } else if i > 60 {
            // Teal glow
            let f = i - 60;
            let g = 60 + f;
            let b = 30 + f / 2;
            let r = f / 4;
            (0xFF << 24) | (r << 16) | (g << 8) | b
        } else if i > 20 {
            // Dark cyan
            let g = 20 + (i - 20);
            let b = 10 + (i - 20) / 2;
            (0xFF << 24) | (g << 8) | b
        } else {
            // Minimum cyan glow
            let g = 4 + i / 2;
            let b = 2 + i / 3;
            (0xFF << 24) | (g << 8) | b
        };
        lut[i as usize] = c;
        i += 1;
    }
    lut
}

static MGLOW_LUT: [u32; 256] = gen_matrix_glow_lut();

/// O(1) glow color lookup for resonant/locked cells
#[inline(always)]
fn mglow(intensity: u8) -> u32 {
    MGLOW_LUT[intensity as usize]
}

/// A single rain drop with resonance lock tracking
#[derive(Clone, Copy)]
struct MDrop {
    y: i16,              // Head row position (can be negative)
    speed: u8,           // Ticks per cell advance (1=fast, 8=slow)
    counter: u8,         // Current tick
    trail_len: u8,       // Trail length in cells
    glyph_seed: u32,     // Seed for glyph selection
    active: bool,
    // Resonance lock system: when adjacent trail cells share the same glyph,
    // they "lock" into that form for the rest of the drop's cycle and glow.
    locked_mask: u64,          // 1 bit per trail position (up to 64)
    locked_glyphs: [u8; 48],   // glyph index stored when locked
}

impl MDrop {
    const fn new() -> Self {
        Self {
            y: -100, speed: 2, counter: 0, trail_len: 20,
            glyph_seed: 0, active: false,
            locked_mask: 0, locked_glyphs: [0u8; 48],
        }
    }

    /// Compute glyph index for trail position (respects lock)
    #[inline(always)]
    fn glyph_at(&self, tp: usize) -> usize {
        if tp < 48 && (self.locked_mask >> tp) & 1 != 0 {
            self.locked_glyphs[tp] as usize
        } else {
            let gs = self.glyph_seed.wrapping_add(tp as u32 * 2654435761);
            (gs % MNUM_GLYPHS as u32) as usize
        }
    }

    /// Check if position is locked (resonant)
    #[inline(always)]
    fn is_locked(&self, tp: usize) -> bool {
        tp < 48 && (self.locked_mask >> tp) & 1 != 0
    }
}

/// Full matrix rain state (~60 KB total — fits in L2 cache)
pub struct ShaderMatrixState {
    drops: [[MDrop; MDROPS]; MGRID_COLS],
    col_depth: [u8; MGRID_COLS],   // 0=far (dim), 255=close (bright)
    num_cols: usize,
    num_rows: usize,
    frame: u32,
    rng: u32,
    initialized: bool,
}

impl ShaderMatrixState {
    pub const fn new() -> Self {
        Self {
            drops: [[MDrop::new(); MDROPS]; MGRID_COLS],
            col_depth: [128u8; MGRID_COLS],
            num_cols: 0,
            num_rows: 0,
            frame: 0,
            rng: 0xDEADBEEF,
            initialized: false,
        }
    }

    /// Initialize with screen dimensions
    pub fn init(&mut self, screen_w: usize, screen_h: usize) {
        self.num_cols = (screen_w / MCELL).min(MGRID_COLS);
        self.num_rows = (screen_h / MCELL).min(MGRID_ROWS);
        self.frame = 0;
        self.rng = 0xDEADBEEF;

        // Assign per-column depth using pseudo-noise for natural parallax
        for col in 0..self.num_cols {
            self.rng = self.rng.wrapping_mul(1103515245).wrapping_add(12345);
            // Mix position-based pattern with randomness for layered parallax
            let pattern = ((col * 17 + 53) % 97) as i32 - 48; // -48..+48
            let random = (self.rng % 100) as i32 - 50;          // -50..+50
            let depth = (145i32 + pattern + random).clamp(20, 255) as u8;
            self.col_depth[col] = depth;
        }

        // Initialize drops with staggered positions
        for col in 0..self.num_cols {
            let depth = self.col_depth[col];
            let df = depth as u32; // 0..255

            let mut next_offset: i32 = 0;
            for di in 0..MDROPS {
                self.rng = self.rng.wrapping_mul(1103515245).wrapping_add(12345);

                // Trail length: closer = longer
                let min_t = MTRAIL_MIN as u32 + df / 8;           // 10..41
                let max_t = (MTRAIL_MAX as u32).min(min_t + 20);  // min+20
                let trail = min_t + (self.rng % (max_t - min_t + 1));

                self.rng = self.rng.wrapping_mul(1103515245).wrapping_add(12345);

                // Gap: closer = smaller gaps (denser rain)
                let gap_base = 3u32.saturating_sub(df / 128);    // 1..3
                let gap_range = 2 + (255 - df) / 50;             // 2..7
                let gap = gap_base + (self.rng % gap_range);

                let start_y = next_offset - (self.rng % 6) as i32;
                next_offset = start_y - trail as i32 - gap as i32;

                self.rng = self.rng.wrapping_mul(1103515245).wrapping_add(12345);

                // Speed: closer = faster (1-2), far = slower (3-6)
                let speed_min = 1 + (255u32.saturating_sub(df)) / 128; // 1..2
                let speed_range = 1 + (255u32.saturating_sub(df)) / 80; // 1..4
                let speed = speed_min + (self.rng % speed_range);

                self.rng = self.rng.wrapping_mul(1103515245).wrapping_add(12345);

                self.drops[col][di] = MDrop {
                    y: start_y as i16,
                    speed: speed.min(8) as u8,
                    counter: (self.rng % speed) as u8,
                    trail_len: trail.min(MTRAIL_MAX as u32) as u8,
                    glyph_seed: self.rng,
                    active: true,
                    locked_mask: 0,
                    locked_glyphs: [0u8; 48],
                };
            }
        }

        self.initialized = true;
    }

    /// Advance all drops — integer-only, no floats
    pub fn update(&mut self) {
        self.frame = self.frame.wrapping_add(1);
        let max_y = self.num_rows as i32 + MTRAIL_MAX as i32 + 10;

        for col in 0..self.num_cols {
            let depth = self.col_depth[col];
            let df = depth as u32;

            for di in 0..MDROPS {
                let drop = &mut self.drops[col][di];
                if !drop.active { continue; }

                // Speed counter
                drop.counter = drop.counter.wrapping_add(1);
                if drop.counter >= drop.speed {
                    drop.counter = 0;
                    drop.y += 1;
                    // Animate glyph
                    drop.glyph_seed = drop.glyph_seed.wrapping_mul(1103515245).wrapping_add(12345);

                    // ── Resonance detection ──
                    // Scan trail for adjacent cells with identical glyph index.
                    // When found, lock both into that glyph for the rest of
                    // this drop's cycle. Already-locked positions keep their glyph.
                    let tlen = drop.trail_len as usize;
                    if tlen >= 2 && tlen <= 48 {
                        let mut prev_idx = drop.glyph_at(0) as u8;
                        for tp in 1..tlen {
                            let cur_idx = drop.glyph_at(tp) as u8;
                            if cur_idx == prev_idx {
                                // Resonance! Lock both positions
                                drop.locked_mask |= (1u64 << (tp - 1)) | (1u64 << tp);
                                drop.locked_glyphs[tp - 1] = cur_idx;
                                drop.locked_glyphs[tp] = cur_idx;
                            }
                            prev_idx = cur_idx;
                        }
                    }
                }

                // Reset if off-screen
                if drop.y as i32 > max_y {
                    self.rng = self.rng.wrapping_mul(1103515245).wrapping_add(12345);

                    let min_t = MTRAIL_MIN as u32 + df / 8;
                    let max_t = (MTRAIL_MAX as u32).min(min_t + 20);
                    let trail = min_t + (self.rng % (max_t - min_t + 1));

                    self.rng = self.rng.wrapping_mul(1103515245).wrapping_add(12345);
                    let gap = 2 + (self.rng % 6);
                    let new_y = -(trail as i32) - gap as i32 - (self.rng % 8) as i32;

                    self.rng = self.rng.wrapping_mul(1103515245).wrapping_add(12345);
                    let speed_min = 1 + (255u32.saturating_sub(df)) / 128;
                    let speed_range = 1 + (255u32.saturating_sub(df)) / 80;
                    let speed = speed_min + (self.rng % speed_range);

                    self.rng = self.rng.wrapping_mul(1103515245).wrapping_add(12345);
                    drop.y = new_y as i16;
                    drop.speed = speed.min(8) as u8;
                    drop.counter = 0;
                    drop.trail_len = trail.min(MTRAIL_MAX as u32) as u8;
                    drop.glyph_seed = self.rng;
                    // Clear resonance locks for new cycle
                    drop.locked_mask = 0;
                    drop.locked_glyphs = [0u8; 48];
                }
            }
        }
    }
}

// ── SMP render context ──────────────────────────────────────────────────────

#[repr(C)]
struct MatrixRenderCtx {
    state: *const ShaderMatrixState,
    fb: *mut u32,
    fb_width: usize,
    fb_height: usize,
}

unsafe impl Send for MatrixRenderCtx {}
unsafe impl Sync for MatrixRenderCtx {}

/// SMP worker: render column band [start..end)
/// Each column writes to non-overlapping horizontal strips → no data races.
fn matrix_col_worker(start: usize, end: usize, data: *mut u8) {
    let ctx = unsafe { &*(data as *const MatrixRenderCtx) };
    let state = unsafe { &*ctx.state };
    let fb = ctx.fb;
    let fw = ctx.fb_width;
    let fh = ctx.fb_height;
    let num_rows = state.num_rows;

    // Reference the glyph table from matrix_fast
    let glyphs = &crate::matrix_fast::MATRIX_GLYPHS_6X6;

    for col in start..end {
        let depth = state.col_depth[col] as u32;
        // Depth brightness: far=40%, close=100%
        let depth_brightness = 100 + (depth * 155 / 255);

        for di in 0..MDROPS {
            let drop = &state.drops[col][di];
            if !drop.active { continue; }

            let head_y = drop.y as i32;
            let trail_len = drop.trail_len as usize;

            for tp in 0..trail_len {
                let cell_y = head_y - tp as i32;
                if cell_y < 0 || cell_y >= num_rows as i32 { continue; }

                // Intensity from LUT (exponential fade along trail)
                let lut_idx = (tp * 63) / trail_len.max(1);
                let base_i = MINT_LUT[lut_idx.min(63)] as u32;
                let mut intensity = ((base_i * depth_brightness) / 255).min(255) as u8;
                if intensity < 2 { continue; }

                // Select glyph: locked (resonant) or dynamic
                let locked = drop.is_locked(tp);
                let glyph_idx = drop.glyph_at(tp);
                let glyph = &glyphs[glyph_idx];

                // Color selection:
                //  - Head cell: white-green flash
                //  - Locked/resonant cell: cyan-white glow (boosted intensity)
                //  - Normal cell: standard green fade
                let color = if tp == 0 {
                    mcolor(intensity.max(250))
                } else if locked {
                    // Resonance glow: boost intensity by 40-80 and use glow LUT
                    intensity = intensity.saturating_add(60).min(255);
                    mglow(intensity)
                } else {
                    mcolor(intensity)
                };

                // Pixel position (1px inset for spacing like BrailleMatrix)
                let px = col * MCELL + 1;
                let py = cell_y as usize * MCELL + 1;

                // Draw 6×6 glyph — unrolled, no bounds check in hot path
                if py + 6 <= fh && px + 6 <= fw {
                    // Fast path: fully on-screen
                    for row in 0..6 {
                        let bits = glyph[row];
                        if bits == 0 { continue; }
                        let row_base = (py + row) * fw + px;
                        unsafe {
                            if bits & 0b000001 != 0 { *fb.add(row_base)     = color; }
                            if bits & 0b000010 != 0 { *fb.add(row_base + 1) = color; }
                            if bits & 0b000100 != 0 { *fb.add(row_base + 2) = color; }
                            if bits & 0b001000 != 0 { *fb.add(row_base + 3) = color; }
                            if bits & 0b010000 != 0 { *fb.add(row_base + 4) = color; }
                            if bits & 0b100000 != 0 { *fb.add(row_base + 5) = color; }
                        }
                    }
                }
            }
        }
    }
}

// ── Static instance ─────────────────────────────────────────────────────────

static SHADER_MATRIX: spin::Mutex<ShaderMatrixState> =
    spin::Mutex::new(ShaderMatrixState::new());

/// Public entry point: update + render the optimized Matrix rain.
///
/// Call this once per frame. It:
///  1. Lazy-initializes on first call
///  2. Updates drop positions (integer math only)
///  3. SSE2-fills background to near-black
///  4. SMP-dispatches glyph rendering across column bands
///
/// Typical perf: ~12K glyph blits/frame vs 1M+ shader calls = 20-60× faster.
pub fn shader_matrix_render(fb: *mut u32, width: usize, height: usize) {
    let mut state = SHADER_MATRIX.lock();

    // Lazy init
    if !state.initialized || state.num_cols != width / MCELL || state.num_rows != height / MCELL {
        state.init(width, height);
    }

    // Update positions (integer-only, very fast)
    state.update();

    // Background fill: near-black (0xFF010201) using SSE2
    let total_pixels = width * height;
    unsafe {
        #[cfg(target_arch = "x86_64")]
        crate::graphics::simd::fill_row_sse2(fb, total_pixels, 0xFF010201);
        #[cfg(not(target_arch = "x86_64"))]
        {
            for i in 0..total_pixels {
                *fb.add(i) = 0xFF010201u32;
            }
        }
    }

    // SMP render: parallelize by column bands
    let num_cols = state.num_cols;
    let ctx = MatrixRenderCtx {
        state: &*state as *const ShaderMatrixState,
        fb,
        fb_width: width,
        fb_height: height,
    };

    crate::cpu::smp::parallel_for(
        num_cols,
        matrix_col_worker,
        &ctx as *const MatrixRenderCtx as *mut u8,
    );
}