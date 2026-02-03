//! Optimized GUI Renderer
//!
//! Uses the graphics engine for professional UI rendering with
//! anti-aliasing, gradients, shadows, and smooth animations.

use alloc::vec::Vec;
use alloc::string::String;
use micromath::F32Ext;

use super::render2d::{FramebufferTarget, Color2D, Renderer2D};
use super::math3d::Vec3;

// ═══════════════════════════════════════════════════════════════════════════════
// OPTIMIZED UI PRIMITIVES
// ═══════════════════════════════════════════════════════════════════════════════

/// High-performance rounded rectangle with optional shadow
pub fn draw_rounded_rect_shadow(
    target: &mut FramebufferTarget,
    x: i32, y: i32,
    width: u32, height: u32,
    radius: u32,
    fill_color: Color2D,
    shadow_blur: u32,
    shadow_color: Color2D,
) {
    // Draw shadow layers (blurred effect)
    if shadow_blur > 0 {
        for i in (1..=shadow_blur).rev() {
            let alpha = ((shadow_blur - i + 1) * 255 / (shadow_blur * 3)) as u8;
            let shadow = Color2D::new(shadow_color.r, shadow_color.g, shadow_color.b, alpha);
            draw_rounded_rect_fast(
                target,
                x + i as i32,
                y + i as i32 + 2,
                width,
                height,
                radius + i / 2,
                shadow,
            );
        }
    }
    
    // Draw main rectangle
    draw_rounded_rect_fast(target, x, y, width, height, radius, fill_color);
}

/// Fast rounded rectangle using optimized corner drawing
pub fn draw_rounded_rect_fast(
    target: &mut FramebufferTarget,
    x: i32, y: i32,
    width: u32, height: u32,
    radius: u32,
    color: Color2D,
) {
    let r = radius.min(width / 2).min(height / 2) as i32;
    let w = width as i32;
    let h = height as i32;
    let c = color.to_u32();
    
    // Fill main body (exclude corners)
    for row in r..(h - r) {
        let py = y + row;
        if py >= 0 && (py as u32) < target.height() {
            for col in 0..w {
                let px = x + col;
                if px >= 0 && (px as u32) < target.width() {
                    target.set_pixel(px as u32, py as u32, c);
                }
            }
        }
    }
    
    // Fill top and bottom strips (between corners)
    for row in 0..r {
        // Top strip
        for col in r..(w - r) {
            let px = x + col;
            let py = y + row;
            if px >= 0 && py >= 0 && (px as u32) < target.width() && (py as u32) < target.height() {
                target.set_pixel(px as u32, py as u32, c);
            }
        }
        // Bottom strip
        for col in r..(w - r) {
            let px = x + col;
            let py = y + h - 1 - row;
            if px >= 0 && py >= 0 && (px as u32) < target.width() && (py as u32) < target.height() {
                target.set_pixel(px as u32, py as u32, c);
            }
        }
    }
    
    // Draw corners using circle quadrants
    let r2 = (r * r) as i32;
    for oy in 0..r {
        for ox in 0..r {
            let dx = r - 1 - ox;
            let dy = r - 1 - oy;
            if dx * dx + dy * dy <= r2 {
                // Top-left
                let px = x + ox;
                let py = y + oy;
                if px >= 0 && py >= 0 && (px as u32) < target.width() && (py as u32) < target.height() {
                    target.set_pixel(px as u32, py as u32, c);
                }
                
                // Top-right
                let px = x + w - 1 - ox;
                if px >= 0 && py >= 0 && (px as u32) < target.width() && (py as u32) < target.height() {
                    target.set_pixel(px as u32, py as u32, c);
                }
                
                // Bottom-left
                let px = x + ox;
                let py = y + h - 1 - oy;
                if px >= 0 && py >= 0 && (px as u32) < target.width() && (py as u32) < target.height() {
                    target.set_pixel(px as u32, py as u32, c);
                }
                
                // Bottom-right
                let px = x + w - 1 - ox;
                if px >= 0 && py >= 0 && (px as u32) < target.width() && (py as u32) < target.height() {
                    target.set_pixel(px as u32, py as u32, c);
                }
            }
        }
    }
}

/// Draw a gradient rectangle (vertical)
pub fn draw_gradient_v(
    target: &mut FramebufferTarget,
    x: i32, y: i32,
    width: u32, height: u32,
    top_color: Color2D,
    bottom_color: Color2D,
) {
    for row in 0..height {
        let t = row as f32 / height as f32;
        let r = (top_color.r as f32 * (1.0 - t) + bottom_color.r as f32 * t) as u8;
        let g = (top_color.g as f32 * (1.0 - t) + bottom_color.g as f32 * t) as u8;
        let b = (top_color.b as f32 * (1.0 - t) + bottom_color.b as f32 * t) as u8;
        let color = Color2D::rgb(r, g, b).to_u32();
        
        let py = y + row as i32;
        if py >= 0 && (py as u32) < target.height() {
            for col in 0..width {
                let px = x + col as i32;
                if px >= 0 && (px as u32) < target.width() {
                    target.set_pixel(px as u32, py as u32, color);
                }
            }
        }
    }
}

/// Draw a gradient rectangle (horizontal)
pub fn draw_gradient_h(
    target: &mut FramebufferTarget,
    x: i32, y: i32,
    width: u32, height: u32,
    left_color: Color2D,
    right_color: Color2D,
) {
    for col in 0..width {
        let t = col as f32 / width as f32;
        let r = (left_color.r as f32 * (1.0 - t) + right_color.r as f32 * t) as u8;
        let g = (left_color.g as f32 * (1.0 - t) + right_color.g as f32 * t) as u8;
        let b = (left_color.b as f32 * (1.0 - t) + right_color.b as f32 * t) as u8;
        let color = Color2D::rgb(r, g, b).to_u32();
        
        let px = x + col as i32;
        if px >= 0 && (px as u32) < target.width() {
            for row in 0..height {
                let py = y + row as i32;
                if py >= 0 && (py as u32) < target.height() {
                    target.set_pixel(px as u32, py as u32, color);
                }
            }
        }
    }
}

/// Draw a radial gradient (for backgrounds)
pub fn draw_radial_gradient(
    target: &mut FramebufferTarget,
    center_x: i32, center_y: i32,
    radius: u32,
    inner_color: Color2D,
    outer_color: Color2D,
) {
    let r2 = (radius * radius) as f32;
    
    for dy in -(radius as i32)..=(radius as i32) {
        for dx in -(radius as i32)..=(radius as i32) {
            let dist2 = (dx * dx + dy * dy) as f32;
            if dist2 <= r2 {
                let t = (dist2 / r2).sqrt();
                let r = (inner_color.r as f32 * (1.0 - t) + outer_color.r as f32 * t) as u8;
                let g = (inner_color.g as f32 * (1.0 - t) + outer_color.g as f32 * t) as u8;
                let b = (inner_color.b as f32 * (1.0 - t) + outer_color.b as f32 * t) as u8;
                let color = Color2D::rgb(r, g, b).to_u32();
                
                let px = center_x + dx;
                let py = center_y + dy;
                if px >= 0 && py >= 0 && (px as u32) < target.width() && (py as u32) < target.height() {
                    target.set_pixel(px as u32, py as u32, color);
                }
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// UI COMPONENTS
// ═══════════════════════════════════════════════════════════════════════════════

/// Modern button styles
pub struct ButtonStyle {
    pub normal: Color2D,
    pub hover: Color2D,
    pub pressed: Color2D,
    pub text: Color2D,
    pub border_radius: u32,
    pub shadow: bool,
}

impl ButtonStyle {
    pub const PRIMARY: Self = Self {
        normal: Color2D { r: 0, g: 204, b: 85, a: 255 },
        hover: Color2D { r: 0, g: 255, b: 102, a: 255 },
        pressed: Color2D { r: 0, g: 170, b: 68, a: 255 },
        text: Color2D::BLACK,
        border_radius: 6,
        shadow: true,
    };
    
    pub const SECONDARY: Self = Self {
        normal: Color2D { r: 20, g: 26, b: 23, a: 255 },
        hover: Color2D { r: 30, g: 38, b: 34, a: 255 },
        pressed: Color2D { r: 15, g: 20, b: 18, a: 255 },
        text: Color2D { r: 0, g: 204, b: 85, a: 255 },
        border_radius: 6,
        shadow: false,
    };
    
    pub const DANGER: Self = Self {
        normal: Color2D { r: 74, g: 53, b: 53, a: 255 },
        hover: Color2D { r: 255, g: 107, b: 107, a: 255 },
        pressed: Color2D { r: 180, g: 60, b: 60, a: 255 },
        text: Color2D::WHITE,
        border_radius: 6,
        shadow: false,
    };
}

/// Draw a modern button
pub fn draw_button(
    target: &mut FramebufferTarget,
    x: i32, y: i32,
    width: u32, height: u32,
    style: &ButtonStyle,
    hovered: bool,
    pressed: bool,
) {
    let color = if pressed {
        style.pressed
    } else if hovered {
        style.hover
    } else {
        style.normal
    };
    
    if style.shadow && !pressed {
        draw_rounded_rect_shadow(
            target, x, y, width, height,
            style.border_radius,
            color,
            4,
            Color2D::rgba(0, 0, 0, 80),
        );
    } else {
        draw_rounded_rect_fast(target, x, y, width, height, style.border_radius, color);
    }
}

/// Window chrome style
pub struct WindowStyle {
    pub bg: Color2D,
    pub title_bar_active: Color2D,
    pub title_bar_inactive: Color2D,
    pub border: Color2D,
    pub title_text: Color2D,
    pub shadow_blur: u32,
    pub corner_radius: u32,
}

impl Default for WindowStyle {
    fn default() -> Self {
        Self {
            bg: Color2D::rgb(11, 15, 12),
            title_bar_active: Color2D::rgb(20, 26, 23),
            title_bar_inactive: Color2D::rgb(13, 18, 15),
            border: Color2D::rgb(0, 68, 34),
            title_text: Color2D::rgb(0, 204, 85),
            shadow_blur: 12,
            corner_radius: 8,
        }
    }
}

/// Draw modern window chrome
pub fn draw_window_frame(
    target: &mut FramebufferTarget,
    x: i32, y: i32,
    width: u32, height: u32,
    title_height: u32,
    style: &WindowStyle,
    focused: bool,
) {
    // Draw shadow
    draw_rounded_rect_shadow(
        target, x, y, width, height,
        style.corner_radius,
        style.bg,
        if focused { style.shadow_blur } else { style.shadow_blur / 2 },
        Color2D::rgba(0, 0, 0, if focused { 120 } else { 60 }),
    );
    
    // Draw title bar gradient
    let title_color = if focused { style.title_bar_active } else { style.title_bar_inactive };
    let title_darker = Color2D::rgb(
        (title_color.r as u32 * 3 / 4) as u8,
        (title_color.g as u32 * 3 / 4) as u8,
        (title_color.b as u32 * 3 / 4) as u8,
    );
    
    // Title bar with subtle gradient
    draw_gradient_v(target, x + 1, y + 1, width - 2, title_height, title_color, title_darker);
    
    // Content background
    let content_y = y + title_height as i32;
    let content_h = height - title_height;
    for row in 0..content_h {
        let py = content_y + row as i32;
        if py >= 0 && (py as u32) < target.height() {
            for col in 1..(width - 1) {
                let px = x + col as i32;
                if px >= 0 && (px as u32) < target.width() {
                    target.set_pixel(px as u32, py as u32, style.bg.to_u32());
                }
            }
        }
    }
    
    // Accent line under title bar
    if focused {
        let accent = Color2D::rgb(0, 136, 68);
        for col in 1..(width - 1) {
            let px = x + col as i32;
            let py = y + title_height as i32;
            if px >= 0 && py >= 0 && (px as u32) < target.width() && (py as u32) < target.height() {
                target.set_pixel(px as u32, py as u32, accent.to_u32());
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// VISUAL EFFECTS
// ═══════════════════════════════════════════════════════════════════════════════

/// Draw a glow effect around a rectangle
pub fn draw_glow(
    target: &mut FramebufferTarget,
    x: i32, y: i32,
    width: u32, height: u32,
    glow_size: u32,
    glow_color: Color2D,
) {
    for i in 1..=glow_size {
        let alpha = ((glow_size - i + 1) * glow_color.a as u32 / glow_size) as u8;
        let color = Color2D::new(glow_color.r, glow_color.g, glow_color.b, alpha);
        
        // Top edge
        for col in 0..width {
            let px = x + col as i32;
            let py = y - i as i32;
            if px >= 0 && py >= 0 && (px as u32) < target.width() && (py as u32) < target.height() {
                blend_pixel(target, px as u32, py as u32, color);
            }
        }
        
        // Bottom edge
        for col in 0..width {
            let px = x + col as i32;
            let py = y + height as i32 + i as i32 - 1;
            if px >= 0 && py >= 0 && (px as u32) < target.width() && (py as u32) < target.height() {
                blend_pixel(target, px as u32, py as u32, color);
            }
        }
        
        // Left edge
        for row in 0..height {
            let px = x - i as i32;
            let py = y + row as i32;
            if px >= 0 && py >= 0 && (px as u32) < target.width() && (py as u32) < target.height() {
                blend_pixel(target, px as u32, py as u32, color);
            }
        }
        
        // Right edge
        for row in 0..height {
            let px = x + width as i32 + i as i32 - 1;
            let py = y + row as i32;
            if px >= 0 && py >= 0 && (px as u32) < target.width() && (py as u32) < target.height() {
                blend_pixel(target, px as u32, py as u32, color);
            }
        }
    }
}

/// Blend a pixel with alpha
fn blend_pixel(target: &mut FramebufferTarget, x: u32, y: u32, color: Color2D) {
    let existing = target.get_pixel(x, y);
    let er = ((existing >> 16) & 0xFF) as u8;
    let eg = ((existing >> 8) & 0xFF) as u8;
    let eb = (existing & 0xFF) as u8;
    
    let alpha = color.a as f32 / 255.0;
    let inv_alpha = 1.0 - alpha;
    
    let nr = (color.r as f32 * alpha + er as f32 * inv_alpha) as u8;
    let ng = (color.g as f32 * alpha + eg as f32 * inv_alpha) as u8;
    let nb = (color.b as f32 * alpha + eb as f32 * inv_alpha) as u8;
    
    target.set_pixel(x, y, Color2D::rgb(nr, ng, nb).to_u32());
}

/// Draw animated pulse effect (for notifications)
pub fn draw_pulse_ring(
    target: &mut FramebufferTarget,
    center_x: i32, center_y: i32,
    radius: u32,
    ring_width: u32,
    color: Color2D,
    phase: f32, // 0.0 to 1.0 for animation
) {
    let outer_r = radius + (phase * ring_width as f32) as u32;
    let inner_r = radius + ((phase * ring_width as f32) as u32).saturating_sub(2);
    let alpha = ((1.0 - phase) * color.a as f32) as u8;
    let ring_color = Color2D::new(color.r, color.g, color.b, alpha);
    
    let outer_r2 = (outer_r * outer_r) as i32;
    let inner_r2 = (inner_r * inner_r) as i32;
    
    for dy in -(outer_r as i32)..=(outer_r as i32) {
        for dx in -(outer_r as i32)..=(outer_r as i32) {
            let dist2 = dx * dx + dy * dy;
            if dist2 <= outer_r2 && dist2 >= inner_r2 {
                let px = center_x + dx;
                let py = center_y + dy;
                if px >= 0 && py >= 0 && (px as u32) < target.width() && (py as u32) < target.height() {
                    blend_pixel(target, px as u32, py as u32, ring_color);
                }
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ANTI-ALIASED PRIMITIVES
// ═══════════════════════════════════════════════════════════════════════════════

/// Draw anti-aliased line (Wu's algorithm)
pub fn draw_aa_line(
    target: &mut FramebufferTarget,
    x0: i32, y0: i32,
    x1: i32, y1: i32,
    color: Color2D,
) {
    let steep = (y1 - y0).abs() > (x1 - x0).abs();
    
    let (x0, y0, x1, y1) = if steep {
        (y0, x0, y1, x1)
    } else {
        (x0, y0, x1, y1)
    };
    
    let (x0, y0, x1, y1) = if x0 > x1 {
        (x1, y1, x0, y0)
    } else {
        (x0, y0, x1, y1)
    };
    
    let dx = (x1 - x0) as f32;
    let dy = (y1 - y0) as f32;
    let gradient = if dx == 0.0 { 1.0 } else { dy / dx };
    
    // Handle first endpoint
    let xend = x0 as f32;
    let yend = y0 as f32;
    let xgap = 1.0 - (x0 as f32 + 0.5).fract();
    let xpxl1 = xend as i32;
    let ypxl1 = yend.floor() as i32;
    
    if steep {
        plot_aa(target, ypxl1, xpxl1, color, (1.0 - yend.fract()) * xgap);
        plot_aa(target, ypxl1 + 1, xpxl1, color, yend.fract() * xgap);
    } else {
        plot_aa(target, xpxl1, ypxl1, color, (1.0 - yend.fract()) * xgap);
        plot_aa(target, xpxl1, ypxl1 + 1, color, yend.fract() * xgap);
    }
    
    let mut intery = yend + gradient;
    
    // Handle second endpoint
    let xend = x1 as f32;
    let yend = y1 as f32;
    let xgap = (x1 as f32 + 0.5).fract();
    let xpxl2 = xend as i32;
    let ypxl2 = yend.floor() as i32;
    
    if steep {
        plot_aa(target, ypxl2, xpxl2, color, (1.0 - yend.fract()) * xgap);
        plot_aa(target, ypxl2 + 1, xpxl2, color, yend.fract() * xgap);
    } else {
        plot_aa(target, xpxl2, ypxl2, color, (1.0 - yend.fract()) * xgap);
        plot_aa(target, xpxl2, ypxl2 + 1, color, yend.fract() * xgap);
    }
    
    // Main loop
    for x in (xpxl1 + 1)..xpxl2 {
        if steep {
            plot_aa(target, intery.floor() as i32, x, color, 1.0 - intery.fract());
            plot_aa(target, intery.floor() as i32 + 1, x, color, intery.fract());
        } else {
            plot_aa(target, x, intery.floor() as i32, color, 1.0 - intery.fract());
            plot_aa(target, x, intery.floor() as i32 + 1, color, intery.fract());
        }
        intery += gradient;
    }
}

fn plot_aa(target: &mut FramebufferTarget, x: i32, y: i32, color: Color2D, brightness: f32) {
    if x >= 0 && y >= 0 && (x as u32) < target.width() && (y as u32) < target.height() {
        let alpha = (color.a as f32 * brightness) as u8;
        let aa_color = Color2D::new(color.r, color.g, color.b, alpha);
        blend_pixel(target, x as u32, y as u32, aa_color);
    }
}

/// Draw anti-aliased circle (Xiaolin Wu style)
pub fn draw_aa_circle(
    target: &mut FramebufferTarget,
    cx: i32, cy: i32,
    radius: u32,
    color: Color2D,
) {
    let r = radius as f32;
    let r2 = r * r;
    
    for y in -(radius as i32 + 1)..=(radius as i32 + 1) {
        for x in -(radius as i32 + 1)..=(radius as i32 + 1) {
            let dist = ((x * x + y * y) as f32).sqrt();
            let diff = (dist - r).abs();
            
            if diff < 1.5 {
                let alpha = (1.0 - diff / 1.5).max(0.0);
                let px = cx + x;
                let py = cy + y;
                if px >= 0 && py >= 0 && (px as u32) < target.width() && (py as u32) < target.height() {
                    let aa_color = Color2D::new(color.r, color.g, color.b, (color.a as f32 * alpha) as u8);
                    blend_pixel(target, px as u32, py as u32, aa_color);
                }
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// DESKTOP BACKGROUND EFFECTS
// ═══════════════════════════════════════════════════════════════════════════════

/// Draw modern desktop background with subtle effects
pub fn draw_modern_background(
    target: &mut FramebufferTarget,
    width: u32, height: u32,
    accent_color: Color2D,
) {
    // Base gradient
    let top = Color2D::rgb(7, 7, 7);
    let bottom = Color2D::rgb(2, 3, 3);
    draw_gradient_v(target, 0, 0, width, height, top, bottom);
    
    // Subtle radial glow at center
    let center_x = (width / 2) as i32;
    let center_y = (height / 2) as i32;
    let glow_radius = (width.min(height) / 2) as i32;
    
    for dy in -glow_radius..=glow_radius {
        for dx in -glow_radius..=glow_radius {
            let dist2 = dx * dx + dy * dy;
            let max_dist2 = glow_radius * glow_radius;
            if dist2 < max_dist2 {
                let t = 1.0 - (dist2 as f32 / max_dist2 as f32);
                let intensity = (t * t * 8.0) as u8; // Quadratic falloff
                if intensity > 0 {
                    let px = center_x + dx;
                    let py = center_y + dy;
                    if px >= 0 && py >= 0 && (px as u32) < width && (py as u32) < height {
                        let existing = target.get_pixel(px as u32, py as u32);
                        let er = ((existing >> 16) & 0xFF) as u8;
                        let eg = ((existing >> 8) & 0xFF) as u8;
                        let eb = (existing & 0xFF) as u8;
                        
                        // Tint toward accent color
                        let nr = er.saturating_add((intensity as u32 * accent_color.r as u32 / 255) as u8);
                        let ng = eg.saturating_add((intensity as u32 * accent_color.g as u32 / 255) as u8);
                        let nb = eb.saturating_add((intensity as u32 * accent_color.b as u32 / 255) as u8);
                        
                        target.set_pixel(px as u32, py as u32, Color2D::rgb(nr, ng, nb).to_u32());
                    }
                }
            }
        }
    }
    
    // Subtle grid pattern (optimized - only draw sparse dots)
    let grid_color = Color2D::rgb(0, 26, 13);
    for y in (48..height).step_by(48) {
        for x in (48..width).step_by(48) {
            target.set_pixel(x, y, grid_color.to_u32());
        }
    }
}

/// Draw vignette effect
pub fn draw_vignette(
    target: &mut FramebufferTarget,
    width: u32, height: u32,
    strength: f32, // 0.0 to 1.0
) {
    let cx = width as f32 / 2.0;
    let cy = height as f32 / 2.0;
    let max_dist = (cx * cx + cy * cy).sqrt();
    
    for y in 0..height {
        for x in 0..width {
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            let dist = (dx * dx + dy * dy).sqrt();
            let t = (dist / max_dist) * strength;
            
            if t > 0.3 {
                let darken = ((t - 0.3) * 0.7 * 255.0) as u8;
                let existing = target.get_pixel(x, y);
                let er = ((existing >> 16) & 0xFF) as u8;
                let eg = ((existing >> 8) & 0xFF) as u8;
                let eb = (existing & 0xFF) as u8;
                
                let nr = er.saturating_sub(darken);
                let ng = eg.saturating_sub(darken);
                let nb = eb.saturating_sub(darken);
                
                target.set_pixel(x, y, Color2D::rgb(nr, ng, nb).to_u32());
            }
        }
    }
}
