//! TrustGL Compositor - OpenGL-accelerated window compositor
//!
//! Uses TrustGL for all GUI rendering, enabling:
//! - Hardware-style compositing (software rendered)
//! - Window transparency and blur effects
//! - Smooth animations and transitions
//! - Drop shadows and glow effects
//! - Customizable visual themes via "shaders"

use alloc::vec::Vec;
use alloc::string::String;
use spin::Mutex;
use micromath::F32Ext;

use super::math3d::{Vec3, Vec4, Mat4};
use super::render2d::Color2D;
use super::opengl::*;
use crate::framebuffer;

// ═══════════════════════════════════════════════════════════════════════════════
// COMPOSITOR CONFIGURATION
// ═══════════════════════════════════════════════════════════════════════════════

/// Visual theme for the compositor
#[derive(Clone, Copy, PartialEq)]
pub enum CompositorTheme {
    /// Classic flat rendering (fastest)
    Flat,
    /// Modern with shadows and subtle effects
    Modern,
    /// Glass-like transparency effects
    Glass,
    /// Neon glow effects
    Neon,
    /// Minimal with thin borders
    Minimal,
}

/// Animation easing functions
#[derive(Clone, Copy)]
pub enum Easing {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    Bounce,
    Elastic,
}

/// Window visual state for compositor
#[derive(Clone)]
pub struct WindowSurface {
    pub id: u32,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub opacity: f32,
    pub scale: f32,
    pub rotation: f32,
    pub z_order: i32,
    pub visible: bool,
    pub focused: bool,
    pub minimized: bool,
    
    // Content buffer (RGBA pixels)
    pub content: Vec<u32>,
    pub content_width: u32,
    pub content_height: u32,
    
    // Animation state
    pub target_x: f32,
    pub target_y: f32,
    pub target_opacity: f32,
    pub target_scale: f32,
    pub animation_progress: f32,
    pub animation_duration: f32,
    pub animation_easing: Easing,
}

impl WindowSurface {
    pub fn new(id: u32, x: f32, y: f32, width: f32, height: f32) -> Self {
        let w = width as u32;
        let h = height as u32;
        Self {
            id,
            x,
            y,
            width,
            height,
            opacity: 1.0,
            scale: 1.0,
            rotation: 0.0,
            z_order: 0,
            visible: true,
            focused: false,
            minimized: false,
            content: alloc::vec![0xFF0A0E0B; (w * h) as usize],
            content_width: w,
            content_height: h,
            target_x: x,
            target_y: y,
            target_opacity: 1.0,
            target_scale: 1.0,
            animation_progress: 1.0,
            animation_duration: 0.0,
            animation_easing: Easing::EaseOut,
        }
    }
    
    /// Update animation state
    pub fn update(&mut self, dt: f32) {
        if self.animation_progress < 1.0 {
            self.animation_progress += dt / self.animation_duration.max(0.001);
            self.animation_progress = self.animation_progress.min(1.0);
            
            let t = apply_easing(self.animation_progress, self.animation_easing);
            
            self.x = lerp(self.x, self.target_x, t);
            self.y = lerp(self.y, self.target_y, t);
            self.opacity = lerp(self.opacity, self.target_opacity, t);
            self.scale = lerp(self.scale, self.target_scale, t);
        }
    }
    
    /// Start animation to new position
    pub fn animate_to(&mut self, x: f32, y: f32, duration: f32, easing: Easing) {
        self.target_x = x;
        self.target_y = y;
        self.animation_progress = 0.0;
        self.animation_duration = duration;
        self.animation_easing = easing;
    }
    
    /// Fade in effect
    pub fn fade_in(&mut self, duration: f32) {
        self.opacity = 0.0;
        self.target_opacity = 1.0;
        self.scale = 0.95;
        self.target_scale = 1.0;
        self.animation_progress = 0.0;
        self.animation_duration = duration;
        self.animation_easing = Easing::EaseOut;
    }
    
    /// Fade out effect
    pub fn fade_out(&mut self, duration: f32) {
        self.target_opacity = 0.0;
        self.target_scale = 0.95;
        self.animation_progress = 0.0;
        self.animation_duration = duration;
        self.animation_easing = Easing::EaseIn;
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// COMPOSITOR STATE
// ═══════════════════════════════════════════════════════════════════════════════

/// The main compositor state
pub struct Compositor {
    pub width: u32,
    pub height: u32,
    pub surfaces: Vec<WindowSurface>,
    pub theme: CompositorTheme,
    pub background_color: u32,
    pub initialized: bool,
    
    // Effect parameters
    pub shadow_offset: f32,
    pub shadow_blur: f32,
    pub shadow_opacity: f32,
    pub corner_radius: f32,
    pub border_width: f32,
    pub border_glow: f32,
    
    // Animation
    pub time: f32,
    pub fps: f32,
    
    // Background
    pub bg_gradient_top: u32,
    pub bg_gradient_bottom: u32,
    pub bg_pattern: BackgroundPattern,
}

#[derive(Clone, Copy, PartialEq)]
pub enum BackgroundPattern {
    Solid,
    Gradient,
    Grid,
    Noise,
    Animated,
}

impl Compositor {
    pub const fn new() -> Self {
        Self {
            width: 1280,
            height: 800,
            surfaces: Vec::new(),
            theme: CompositorTheme::Modern,
            background_color: 0xFF070707,
            initialized: false,
            shadow_offset: 8.0,
            shadow_blur: 16.0,
            shadow_opacity: 0.4,
            corner_radius: 8.0,
            border_width: 1.0,
            border_glow: 0.0,
            time: 0.0,
            fps: 60.0,
            bg_gradient_top: 0xFF070707,
            bg_gradient_bottom: 0xFF020303,
            bg_pattern: BackgroundPattern::Gradient,
        }
    }
    
    /// Initialize the compositor with TrustGL
    pub fn init(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        
        // Initialize TrustGL
        gl_init(width, height);
        gl_enable(GL_DEPTH_TEST);
        gl_enable(GL_BLEND);
        
        // Set up orthographic projection for 2D compositing
        gl_matrix_mode(GL_PROJECTION);
        gl_load_identity();
        gl_ortho(0.0, width as f32, height as f32, 0.0, -100.0, 100.0);
        
        gl_matrix_mode(GL_MODELVIEW);
        gl_load_identity();
        
        self.initialized = true;
    }
    
    /// Add a window surface
    pub fn add_surface(&mut self, surface: WindowSurface) -> u32 {
        let id = surface.id;
        self.surfaces.push(surface);
        self.sort_surfaces();
        id
    }
    
    /// Remove a surface
    pub fn remove_surface(&mut self, id: u32) {
        self.surfaces.retain(|s| s.id != id);
    }
    
    /// Get mutable surface by ID
    pub fn get_surface_mut(&mut self, id: u32) -> Option<&mut WindowSurface> {
        self.surfaces.iter_mut().find(|s| s.id == id)
    }
    
    /// Sort surfaces by z-order
    fn sort_surfaces(&mut self) {
        self.surfaces.sort_by(|a, b| a.z_order.cmp(&b.z_order));
    }
    
    /// Update all animations
    pub fn update(&mut self, dt: f32) {
        self.time += dt;
        for surface in &mut self.surfaces {
            surface.update(dt);
        }
    }
    
    /// Render the entire desktop using TrustGL
    pub fn render(&self) {
        if !self.initialized {
            return;
        }
        
        // Clear with background
        gl_clear_color(
            ((self.background_color >> 16) & 0xFF) as f32 / 255.0,
            ((self.background_color >> 8) & 0xFF) as f32 / 255.0,
            (self.background_color & 0xFF) as f32 / 255.0,
            1.0,
        );
        gl_clear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);
        
        // Draw background
        self.render_background();
        
        // Draw all surfaces (windows)
        for surface in &self.surfaces {
            if !surface.visible || surface.opacity <= 0.001 {
                continue;
            }
            self.render_surface(surface);
        }
        
        // Flush to framebuffer
        gl_flush();
    }
    
    /// Render background with effects
    fn render_background(&self) {
        match self.bg_pattern {
            BackgroundPattern::Solid => {
                self.draw_filled_rect(0.0, 0.0, self.width as f32, self.height as f32, 
                                      self.background_color, -99.0);
            }
            BackgroundPattern::Gradient => {
                self.draw_gradient_rect(0.0, 0.0, self.width as f32, self.height as f32,
                                        self.bg_gradient_top, self.bg_gradient_bottom, -99.0);
            }
            BackgroundPattern::Grid => {
                self.draw_gradient_rect(0.0, 0.0, self.width as f32, self.height as f32,
                                        self.bg_gradient_top, self.bg_gradient_bottom, -99.0);
                self.draw_grid(-98.0);
            }
            BackgroundPattern::Animated => {
                self.draw_animated_background(-99.0);
            }
            _ => {
                self.draw_filled_rect(0.0, 0.0, self.width as f32, self.height as f32,
                                      self.background_color, -99.0);
            }
        }
    }
    
    /// Draw animated background with subtle effects
    fn draw_animated_background(&self, z: f32) {
        // Base gradient
        self.draw_gradient_rect(0.0, 0.0, self.width as f32, self.height as f32,
                                self.bg_gradient_top, self.bg_gradient_bottom, z);
        
        // Animated glow spots
        let t = self.time * 0.5;
        let glow_x = (self.width as f32 / 2.0) + (t.sin() * 200.0);
        let glow_y = (self.height as f32 / 3.0) + (t.cos() * 100.0);
        self.draw_glow(glow_x, glow_y, 300.0, 0x1000FF44, z + 0.1);
    }
    
    /// Render a window surface with effects
    fn render_surface(&self, surface: &WindowSurface) {
        let x = surface.x;
        let y = surface.y;
        let w = surface.width * surface.scale;
        let h = surface.height * surface.scale;
        let z = surface.z_order as f32;
        
        // Apply theme-specific effects
        match self.theme {
            CompositorTheme::Modern => {
                // Drop shadow
                if self.shadow_opacity > 0.0 {
                    self.draw_shadow(x, y, w, h, z - 0.5);
                }
                // Window with rounded corners
                self.draw_window_frame(surface, z);
            }
            CompositorTheme::Glass => {
                // Blur background (simulated)
                self.draw_blur_rect(x, y, w, h, z - 0.3);
                // Glass frame
                self.draw_glass_frame(surface, z);
            }
            CompositorTheme::Neon => {
                // Glow effect
                self.draw_neon_glow(x, y, w, h, z - 0.5);
                // Neon frame
                self.draw_neon_frame(surface, z);
            }
            CompositorTheme::Minimal => {
                // Just thin border
                self.draw_minimal_frame(surface, z);
            }
            CompositorTheme::Flat => {
                // Simple flat frame
                self.draw_flat_frame(surface, z);
            }
        }
        
        // Draw window content
        self.draw_surface_content(surface, z + 0.1);
    }
    
    /// Draw drop shadow
    fn draw_shadow(&self, x: f32, y: f32, w: f32, h: f32, z: f32) {
        let offset = self.shadow_offset;
        let blur = self.shadow_blur;
        let alpha = (self.shadow_opacity * 255.0) as u32;
        let shadow_color = alpha << 24;
        
        // Multiple layers for blur effect
        for i in 0..4 {
            let expand = blur * (i as f32 / 4.0);
            let layer_alpha = alpha / (i + 1);
            let color = layer_alpha << 24;
            
            self.draw_filled_rect(
                x + offset - expand,
                y + offset - expand,
                w + expand * 2.0,
                h + expand * 2.0,
                color,
                z - (i as f32 * 0.01),
            );
        }
    }
    
    /// Draw window frame (Modern theme)
    fn draw_window_frame(&self, surface: &WindowSurface, z: f32) {
        let x = surface.x;
        let y = surface.y;
        let w = surface.width * surface.scale;
        let h = surface.height * surface.scale;
        
        // Title bar
        let title_h = 32.0;
        let title_color = if surface.focused { 0xFF0D120F } else { 0xFF0A0D0B };
        self.draw_filled_rect(x, y, w, title_h, title_color, z);
        
        // Title bar accent line
        if surface.focused {
            self.draw_filled_rect(x, y + title_h - 2.0, w, 2.0, 0xFF008844, z + 0.01);
        }
        
        // Window body
        let body_color = 0xFF0A0E0B;
        self.draw_filled_rect(x, y + title_h, w, h - title_h, body_color, z);
        
        // Window border
        let border_color = if surface.focused { 0xFF006633 } else { 0xFF004422 };
        self.draw_rect_outline(x, y, w, h, border_color, z + 0.02);
        
        // Window controls
        self.draw_window_controls(x + 8.0, y + 8.0, surface.focused, z + 0.03);
    }
    
    /// Draw glass-like frame
    fn draw_glass_frame(&self, surface: &WindowSurface, z: f32) {
        let x = surface.x;
        let y = surface.y;
        let w = surface.width * surface.scale;
        let h = surface.height * surface.scale;
        
        // Semi-transparent background
        let alpha = (surface.opacity * 0.85 * 255.0) as u32;
        let glass_color = (alpha << 24) | 0x0D1210;
        self.draw_filled_rect(x, y, w, h, glass_color, z);
        
        // Subtle border
        self.draw_rect_outline(x, y, w, h, 0x4000FF66, z + 0.01);
        
        // Top highlight
        self.draw_filled_rect(x + 1.0, y + 1.0, w - 2.0, 1.0, 0x2000FF66, z + 0.02);
    }
    
    /// Draw neon glowing frame
    fn draw_neon_frame(&self, surface: &WindowSurface, z: f32) {
        let x = surface.x;
        let y = surface.y;
        let w = surface.width * surface.scale;
        let h = surface.height * surface.scale;
        
        // Dark background
        self.draw_filled_rect(x, y, w, h, 0xFF050505, z);
        
        // Neon border with glow
        let glow_color = if surface.focused { 0xFF00FF66 } else { 0xFF00AA44 };
        
        // Outer glow
        for i in 1..5 {
            let expand = i as f32 * 2.0;
            let alpha = (60 - i * 15) as u32;
            let color = (alpha << 24) | (glow_color & 0x00FFFFFF);
            self.draw_rect_outline(x - expand, y - expand, w + expand * 2.0, h + expand * 2.0, color, z + 0.01);
        }
        
        // Core border
        self.draw_rect_outline(x, y, w, h, glow_color, z + 0.05);
    }
    
    /// Draw minimal frame
    fn draw_minimal_frame(&self, surface: &WindowSurface, z: f32) {
        let x = surface.x;
        let y = surface.y;
        let w = surface.width * surface.scale;
        let h = surface.height * surface.scale;
        
        // Just background
        self.draw_filled_rect(x, y, w, h, 0xFF0A0E0B, z);
        
        // Thin border
        let border_color = if surface.focused { 0xFF00CC55 } else { 0xFF004422 };
        self.draw_rect_outline(x, y, w, h, border_color, z + 0.01);
    }
    
    /// Draw flat frame (no effects)
    fn draw_flat_frame(&self, surface: &WindowSurface, z: f32) {
        let x = surface.x;
        let y = surface.y;
        let w = surface.width * surface.scale;
        let h = surface.height * surface.scale;
        
        // Title bar
        let title_h = 32.0;
        self.draw_filled_rect(x, y, w, title_h, 0xFF0D120F, z);
        
        // Body
        self.draw_filled_rect(x, y + title_h, w, h - title_h, 0xFF0A0E0B, z);
        
        // Border
        self.draw_rect_outline(x, y, w, h, 0xFF006633, z + 0.01);
    }
    
    /// Draw window control buttons (close, minimize, maximize)
    fn draw_window_controls(&self, x: f32, y: f32, focused: bool, z: f32) {
        let spacing = 20.0;
        let radius = 6.0;
        
        // Close button (red)
        let close_color = if focused { 0xFF4A3535 } else { 0xFF2A2020 };
        self.draw_circle(x, y + 8.0, radius, close_color, z);
        
        // Minimize button (yellow)
        let min_color = if focused { 0xFF3A3A30 } else { 0xFF202010 };
        self.draw_circle(x + spacing, y + 8.0, radius, min_color, z);
        
        // Maximize button (green)
        let max_color = if focused { 0xFF2A3A2F } else { 0xFF10201A };
        self.draw_circle(x + spacing * 2.0, y + 8.0, radius, max_color, z);
    }
    
    /// Draw surface content (window pixels)
    fn draw_surface_content(&self, surface: &WindowSurface, z: f32) {
        // In a real implementation, this would texture-map the window content
        // For now, we'll draw a content area placeholder
        let x = surface.x;
        let y = surface.y + 32.0; // Below title bar
        let w = surface.width * surface.scale;
        let h = (surface.height - 32.0) * surface.scale;
        
        // Content area is already drawn in frame, this would be for texturing
    }
    
    // ═══════════════════════════════════════════════════════════════════════════
    // PRIMITIVE DRAWING (using TrustGL)
    // ═══════════════════════════════════════════════════════════════════════════
    
    /// Draw filled rectangle
    fn draw_filled_rect(&self, x: f32, y: f32, w: f32, h: f32, color: u32, z: f32) {
        let (r, g, b, a) = color_to_rgba(color);
        
        gl_begin(GL_QUADS);
        gl_color4f(r, g, b, a);
        gl_vertex3f(x, y, z);
        gl_vertex3f(x + w, y, z);
        gl_vertex3f(x + w, y + h, z);
        gl_vertex3f(x, y + h, z);
        gl_end();
    }
    
    /// Draw gradient rectangle (vertical)
    fn draw_gradient_rect(&self, x: f32, y: f32, w: f32, h: f32, 
                          top_color: u32, bottom_color: u32, z: f32) {
        let (r1, g1, b1, a1) = color_to_rgba(top_color);
        let (r2, g2, b2, a2) = color_to_rgba(bottom_color);
        
        gl_begin(GL_QUADS);
        gl_color4f(r1, g1, b1, a1);
        gl_vertex3f(x, y, z);
        gl_vertex3f(x + w, y, z);
        gl_color4f(r2, g2, b2, a2);
        gl_vertex3f(x + w, y + h, z);
        gl_vertex3f(x, y + h, z);
        gl_end();
    }
    
    /// Draw rectangle outline
    fn draw_rect_outline(&self, x: f32, y: f32, w: f32, h: f32, color: u32, z: f32) {
        let (r, g, b, a) = color_to_rgba(color);
        
        gl_begin(GL_LINE_LOOP);
        gl_color4f(r, g, b, a);
        gl_vertex3f(x, y, z);
        gl_vertex3f(x + w, y, z);
        gl_vertex3f(x + w, y + h, z);
        gl_vertex3f(x, y + h, z);
        gl_end();
    }
    
    /// Draw filled circle (approximated with triangles)
    fn draw_circle(&self, cx: f32, cy: f32, radius: f32, color: u32, z: f32) {
        let (r, g, b, a) = color_to_rgba(color);
        let segments = 16;
        
        gl_begin(GL_TRIANGLE_FAN);
        gl_color4f(r, g, b, a);
        gl_vertex3f(cx, cy, z); // Center
        
        for i in 0..=segments {
            let angle = (i as f32 / segments as f32) * core::f32::consts::PI * 2.0;
            let px = cx + angle.cos() * radius;
            let py = cy + angle.sin() * radius;
            gl_vertex3f(px, py, z);
        }
        gl_end();
    }
    
    /// Draw glow effect
    fn draw_glow(&self, cx: f32, cy: f32, radius: f32, color: u32, z: f32) {
        let (r, g, b, _) = color_to_rgba(color);
        
        // Multiple expanding circles with decreasing alpha
        for i in 0..8 {
            let t = i as f32 / 8.0;
            let current_radius = radius * (0.3 + t * 0.7);
            let alpha = 0.3 * (1.0 - t);
            
            gl_begin(GL_TRIANGLE_FAN);
            gl_color4f(r, g, b, alpha);
            gl_vertex3f(cx, cy, z);
            
            let segments = 24;
            for j in 0..=segments {
                let angle = (j as f32 / segments as f32) * core::f32::consts::PI * 2.0;
                let px = cx + angle.cos() * current_radius;
                let py = cy + angle.sin() * current_radius;
                gl_vertex3f(px, py, z);
            }
            gl_end();
        }
    }
    
    /// Draw neon glow around rectangle
    fn draw_neon_glow(&self, x: f32, y: f32, w: f32, h: f32, z: f32) {
        let glow_color = 0x00FF66u32;
        let (r, g, b, _) = color_to_rgba(glow_color);
        
        for i in 1..6 {
            let expand = i as f32 * 3.0;
            let alpha = 0.4 / (i as f32);
            
            gl_begin(GL_LINE_LOOP);
            gl_color4f(r, g, b, alpha);
            gl_vertex3f(x - expand, y - expand, z);
            gl_vertex3f(x + w + expand, y - expand, z);
            gl_vertex3f(x + w + expand, y + h + expand, z);
            gl_vertex3f(x - expand, y + h + expand, z);
            gl_end();
        }
    }
    
    /// Draw blur rectangle (simulated)
    fn draw_blur_rect(&self, x: f32, y: f32, w: f32, h: f32, z: f32) {
        // Simulated blur with semi-transparent overlay
        self.draw_filled_rect(x, y, w, h, 0x800D1210, z);
    }
    
    /// Draw grid pattern
    fn draw_grid(&self, z: f32) {
        let grid_color = 0x08004422u32;
        let (r, g, b, a) = color_to_rgba(grid_color);
        let spacing = 40.0;
        
        gl_begin(GL_LINES);
        gl_color4f(r, g, b, a);
        
        // Vertical lines
        let mut x = 0.0;
        while x < self.width as f32 {
            gl_vertex3f(x, 0.0, z);
            gl_vertex3f(x, self.height as f32, z);
            x += spacing;
        }
        
        // Horizontal lines
        let mut y = 0.0;
        while y < self.height as f32 {
            gl_vertex3f(0.0, y, z);
            gl_vertex3f(self.width as f32, y, z);
            y += spacing;
        }
        gl_end();
    }
    
    /// Set theme
    pub fn set_theme(&mut self, theme: CompositorTheme) {
        self.theme = theme;
        
        // Adjust parameters based on theme
        match theme {
            CompositorTheme::Modern => {
                self.shadow_opacity = 0.4;
                self.shadow_blur = 16.0;
                self.corner_radius = 8.0;
                self.border_glow = 0.0;
            }
            CompositorTheme::Glass => {
                self.shadow_opacity = 0.2;
                self.shadow_blur = 24.0;
                self.corner_radius = 12.0;
                self.border_glow = 0.3;
            }
            CompositorTheme::Neon => {
                self.shadow_opacity = 0.0;
                self.corner_radius = 4.0;
                self.border_glow = 1.0;
                self.bg_pattern = BackgroundPattern::Grid;
            }
            CompositorTheme::Minimal => {
                self.shadow_opacity = 0.0;
                self.shadow_blur = 0.0;
                self.corner_radius = 0.0;
                self.border_glow = 0.0;
            }
            CompositorTheme::Flat => {
                self.shadow_opacity = 0.0;
                self.shadow_blur = 0.0;
                self.corner_radius = 0.0;
                self.border_glow = 0.0;
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// HELPER FUNCTIONS
// ═══════════════════════════════════════════════════════════════════════════════

/// Convert u32 ARGB color to RGBA floats
fn color_to_rgba(color: u32) -> (f32, f32, f32, f32) {
    let a = ((color >> 24) & 0xFF) as f32 / 255.0;
    let r = ((color >> 16) & 0xFF) as f32 / 255.0;
    let g = ((color >> 8) & 0xFF) as f32 / 255.0;
    let b = (color & 0xFF) as f32 / 255.0;
    (r, g, b, a)
}

/// Linear interpolation
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Apply easing function
fn apply_easing(t: f32, easing: Easing) -> f32 {
    match easing {
        Easing::Linear => t,
        Easing::EaseIn => t * t,
        Easing::EaseOut => 1.0 - (1.0 - t) * (1.0 - t),
        Easing::EaseInOut => {
            if t < 0.5 {
                2.0 * t * t
            } else {
                1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
            }
        }
        Easing::Bounce => {
            let n1 = 7.5625;
            let d1 = 2.75;
            let mut t = t;
            if t < 1.0 / d1 {
                n1 * t * t
            } else if t < 2.0 / d1 {
                t -= 1.5 / d1;
                n1 * t * t + 0.75
            } else if t < 2.5 / d1 {
                t -= 2.25 / d1;
                n1 * t * t + 0.9375
            } else {
                t -= 2.625 / d1;
                n1 * t * t + 0.984375
            }
        }
        Easing::Elastic => {
            if t == 0.0 || t == 1.0 {
                t
            } else {
                let p = 0.3;
                let s = p / 4.0;
                (2.0f32).powf(-10.0 * t) * ((t - s) * (2.0 * core::f32::consts::PI / p)).sin() + 1.0
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// GLOBAL COMPOSITOR INSTANCE
// ═══════════════════════════════════════════════════════════════════════════════

static COMPOSITOR: Mutex<Compositor> = Mutex::new(Compositor::new());

/// Get the global compositor
pub fn compositor() -> spin::MutexGuard<'static, Compositor> {
    COMPOSITOR.lock()
}

/// Initialize the global compositor
pub fn init_compositor(width: u32, height: u32) {
    compositor().init(width, height);
}

/// Set the compositor theme
pub fn set_compositor_theme(theme: CompositorTheme) {
    compositor().set_theme(theme);
}

/// Render frame using compositor
pub fn render_compositor_frame() {
    compositor().render();
}
