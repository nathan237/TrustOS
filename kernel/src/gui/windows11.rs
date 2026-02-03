//! Windows 11 Style Renderer
//!
//! Modern UI rendering with:
//! - Rounded corners (Mica/Acrylic style)
//! - Drop shadows with blur simulation
//! - Fluent Design icons
//! - Smooth gradients
//! - Glass/transparency effects

use crate::framebuffer;

// ═══════════════════════════════════════════════════════════════════════════════
// WINDOWS 11 COLOR PALETTE
// ═══════════════════════════════════════════════════════════════════════════════

/// Windows 11 Dark theme colors
pub mod colors {
    // Backgrounds
    pub const MICA_DARK: u32 = 0xFF202020;           // Main window background
    pub const MICA_DARKER: u32 = 0xFF1A1A1A;         // Secondary background
    pub const ACRYLIC_BG: u32 = 0xE0282828;          // Semi-transparent acrylic
    pub const SURFACE: u32 = 0xFF2D2D2D;             // Card/surface background
    pub const SURFACE_HOVER: u32 = 0xFF383838;       // Hover state
    pub const SURFACE_PRESSED: u32 = 0xFF404040;    // Pressed state
    
    // Title bar
    pub const TITLEBAR_ACTIVE: u32 = 0xFF1F1F1F;    // Active window title
    pub const TITLEBAR_INACTIVE: u32 = 0xFF2B2B2B;  // Inactive window
    
    // Accent (Windows 11 default blue)
    pub const ACCENT: u32 = 0xFF0078D4;              // Primary accent
    pub const ACCENT_LIGHT: u32 = 0xFF60CDFF;        // Accent light
    pub const ACCENT_DARK: u32 = 0xFF005A9E;         // Accent dark
    
    // Text
    pub const TEXT_PRIMARY: u32 = 0xFFFFFFFF;        // Primary text
    pub const TEXT_SECONDARY: u32 = 0xFFB3B3B3;      // Secondary text
    pub const TEXT_DISABLED: u32 = 0xFF6E6E6E;       // Disabled text
    
    // Borders
    pub const BORDER_SUBTLE: u32 = 0xFF3D3D3D;       // Subtle border
    pub const BORDER_DEFAULT: u32 = 0xFF4D4D4D;      // Default border
    pub const BORDER_STRONG: u32 = 0xFF6B6B6B;       // Strong border
    
    // Window controls
    pub const CLOSE_HOVER: u32 = 0xFFC42B1C;         // Close button hover
    pub const CLOSE_PRESSED: u32 = 0xFFA31818;       // Close button pressed
    pub const CONTROL_HOVER: u32 = 0xFF404040;       // Min/Max hover
    
    // Shadows (with alpha)
    pub const SHADOW_AMBIENT: u32 = 0x40000000;      // Ambient shadow
    pub const SHADOW_KEY: u32 = 0x30000000;          // Key shadow
    
    // Taskbar
    pub const TASKBAR_BG: u32 = 0xF0202020;          // Taskbar with transparency
    pub const TASKBAR_HOVER: u32 = 0xFF383838;       // Taskbar item hover
    pub const TASKBAR_ACTIVE: u32 = 0xFF0078D4;      // Active app indicator
}

// ═══════════════════════════════════════════════════════════════════════════════
// ROUNDED RECTANGLE RENDERING
// ═══════════════════════════════════════════════════════════════════════════════

/// Draw a filled rounded rectangle
pub fn draw_rounded_rect(x: i32, y: i32, w: u32, h: u32, radius: u32, color: u32) {
    if w < radius * 2 || h < radius * 2 {
        // Too small for rounded corners
        fill_rect(x, y, w, h, color);
        return;
    }
    
    let r = radius as i32;
    let w = w as i32;
    let h = h as i32;
    
    // Main body (without corners)
    fill_rect(x + r, y, (w - r * 2) as u32, h as u32, color);
    fill_rect(x, y + r, r as u32, (h - r * 2) as u32, color);
    fill_rect(x + w - r, y + r, r as u32, (h - r * 2) as u32, color);
    
    // Draw corners with circle algorithm
    draw_corner_filled(x + r, y + r, radius, Corner::TopLeft, color);
    draw_corner_filled(x + w - r - 1, y + r, radius, Corner::TopRight, color);
    draw_corner_filled(x + r, y + h - r - 1, radius, Corner::BottomLeft, color);
    draw_corner_filled(x + w - r - 1, y + h - r - 1, radius, Corner::BottomRight, color);
}

/// Draw a rounded rectangle border
pub fn draw_rounded_rect_border(x: i32, y: i32, w: u32, h: u32, radius: u32, color: u32) {
    if radius == 0 {
        draw_rect_border(x, y, w, h, color);
        return;
    }
    
    let r = radius as i32;
    let wi = w as i32;
    let hi = h as i32;
    
    // Horizontal lines
    draw_hline(x + r, y, (wi - r * 2) as u32, color);
    draw_hline(x + r, y + hi - 1, (wi - r * 2) as u32, color);
    
    // Vertical lines
    draw_vline(x, y + r, (hi - r * 2) as u32, color);
    draw_vline(x + wi - 1, y + r, (hi - r * 2) as u32, color);
    
    // Corner arcs
    draw_corner_arc(x + r, y + r, radius, Corner::TopLeft, color);
    draw_corner_arc(x + wi - r - 1, y + r, radius, Corner::TopRight, color);
    draw_corner_arc(x + r, y + hi - r - 1, radius, Corner::BottomLeft, color);
    draw_corner_arc(x + wi - r - 1, y + hi - r - 1, radius, Corner::BottomRight, color);
}

#[derive(Clone, Copy)]
enum Corner {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

/// Draw a filled quarter circle for rounded corners
fn draw_corner_filled(cx: i32, cy: i32, radius: u32, corner: Corner, color: u32) {
    let r = radius as i32;
    
    for dy in 0..=r {
        for dx in 0..=r {
            // Check if point is inside the circle
            if dx * dx + dy * dy <= r * r {
                let (px, py) = match corner {
                    Corner::TopLeft => (cx - dx, cy - dy),
                    Corner::TopRight => (cx + dx, cy - dy),
                    Corner::BottomLeft => (cx - dx, cy + dy),
                    Corner::BottomRight => (cx + dx, cy + dy),
                };
                draw_pixel(px, py, color);
            }
        }
    }
}

/// Draw a quarter circle arc for rounded borders
fn draw_corner_arc(cx: i32, cy: i32, radius: u32, corner: Corner, color: u32) {
    let r = radius as i32;
    let mut x = 0;
    let mut y = r;
    let mut d = 3 - 2 * r;
    
    while x <= y {
        let points = match corner {
            Corner::TopLeft => [(cx - x, cy - y), (cx - y, cy - x)],
            Corner::TopRight => [(cx + x, cy - y), (cx + y, cy - x)],
            Corner::BottomLeft => [(cx - x, cy + y), (cx - y, cy + x)],
            Corner::BottomRight => [(cx + x, cy + y), (cx + y, cy + x)],
        };
        
        for (px, py) in points {
            draw_pixel(px, py, color);
        }
        
        if d < 0 {
            d += 4 * x + 6;
        } else {
            d += 4 * (x - y) + 10;
            y -= 1;
        }
        x += 1;
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// DROP SHADOW RENDERING
// ═══════════════════════════════════════════════════════════════════════════════

/// Draw a drop shadow behind a rounded rectangle
pub fn draw_shadow(x: i32, y: i32, w: u32, h: u32, radius: u32, offset_x: i32, offset_y: i32, blur: u32) {
    // Multi-layer shadow for soft effect
    let layers = blur.min(5);
    
    for i in 0..layers {
        let expand = i as i32;
        let alpha = 0x15 - (i as u32 * 0x04);
        let shadow_color = alpha << 24;
        
        draw_rounded_rect(
            x + offset_x - expand,
            y + offset_y - expand + i as i32,
            w + (expand * 2) as u32,
            h + (expand * 2) as u32,
            radius + i,
            shadow_color,
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// GRADIENT RENDERING
// ═══════════════════════════════════════════════════════════════════════════════

/// Draw a vertical gradient
pub fn draw_gradient_v(x: i32, y: i32, w: u32, h: u32, color_top: u32, color_bottom: u32) {
    for row in 0..h {
        let t = row as f32 / h as f32;
        let color = blend_colors(color_top, color_bottom, t);
        fill_rect(x, y + row as i32, w, 1, color);
    }
}

/// Draw a horizontal gradient
pub fn draw_gradient_h(x: i32, y: i32, w: u32, h: u32, color_left: u32, color_right: u32) {
    for col in 0..w {
        let t = col as f32 / w as f32;
        let color = blend_colors(color_left, color_right, t);
        fill_rect(x + col as i32, y, 1, h, color);
    }
}

/// Blend two colors
fn blend_colors(c1: u32, c2: u32, t: f32) -> u32 {
    let a1 = ((c1 >> 24) & 0xFF) as f32;
    let r1 = ((c1 >> 16) & 0xFF) as f32;
    let g1 = ((c1 >> 8) & 0xFF) as f32;
    let b1 = (c1 & 0xFF) as f32;
    
    let a2 = ((c2 >> 24) & 0xFF) as f32;
    let r2 = ((c2 >> 16) & 0xFF) as f32;
    let g2 = ((c2 >> 8) & 0xFF) as f32;
    let b2 = (c2 & 0xFF) as f32;
    
    let a = (a1 + (a2 - a1) * t) as u32;
    let r = (r1 + (r2 - r1) * t) as u32;
    let g = (g1 + (g2 - g1) * t) as u32;
    let b = (b1 + (b2 - b1) * t) as u32;
    
    (a << 24) | (r << 16) | (g << 8) | b
}

// ═══════════════════════════════════════════════════════════════════════════════
// WINDOWS 11 TITLE BAR
// ═══════════════════════════════════════════════════════════════════════════════

/// Draw a Windows 11 style title bar
pub fn draw_titlebar(x: i32, y: i32, w: u32, h: u32, title: &str, focused: bool, maximized: bool) {
    let bg = if focused { colors::TITLEBAR_ACTIVE } else { colors::TITLEBAR_INACTIVE };
    
    // Title bar background
    fill_rect(x, y, w, h, bg);
    
    // Title text (centered or left-aligned)
    let text_color = if focused { colors::TEXT_PRIMARY } else { colors::TEXT_SECONDARY };
    let title_x = x + 12; // Left aligned with padding
    let title_y = y + (h as i32 - 12) / 2;
    draw_text(title_x, title_y, title, text_color);
    
    // Window control buttons
    let btn_w = 46;
    let btn_h = h;
    let btn_y = y;
    
    // Close button (rightmost)
    let close_x = x + w as i32 - btn_w;
    draw_control_button(close_x, btn_y, btn_w as u32, btn_h, ControlButton::Close, false);
    
    // Maximize button
    let max_x = close_x - btn_w;
    draw_control_button(max_x, btn_y, btn_w as u32, btn_h, 
        if maximized { ControlButton::Restore } else { ControlButton::Maximize }, false);
    
    // Minimize button
    let min_x = max_x - btn_w;
    draw_control_button(min_x, btn_y, btn_w as u32, btn_h, ControlButton::Minimize, false);
}

#[derive(Clone, Copy)]
pub enum ControlButton {
    Close,
    Maximize,
    Restore,
    Minimize,
}

/// Draw a window control button
pub fn draw_control_button(x: i32, y: i32, w: u32, h: u32, btn_type: ControlButton, hovered: bool) {
    // Background on hover
    if hovered {
        let bg = match btn_type {
            ControlButton::Close => colors::CLOSE_HOVER,
            _ => colors::CONTROL_HOVER,
        };
        fill_rect(x, y, w, h, bg);
    }
    
    // Icon
    let icon_color = if hovered && matches!(btn_type, ControlButton::Close) {
        colors::TEXT_PRIMARY
    } else {
        colors::TEXT_SECONDARY
    };
    
    let cx = x + w as i32 / 2;
    let cy = y + h as i32 / 2;
    
    match btn_type {
        ControlButton::Close => {
            // X icon (10x10)
            for i in 0..10 {
                draw_pixel(cx - 5 + i, cy - 5 + i, icon_color);
                draw_pixel(cx - 5 + i, cy + 4 - i, icon_color);
            }
        }
        ControlButton::Maximize => {
            // Square icon (10x10)
            draw_rect_border(cx - 5, cy - 5, 10, 10, icon_color);
        }
        ControlButton::Restore => {
            // Overlapping squares (8x8 each)
            draw_rect_border(cx - 3, cy - 5, 8, 8, icon_color);
            draw_rect_border(cx - 5, cy - 3, 8, 8, icon_color);
            fill_rect(cx - 4, cy - 2, 6, 6, colors::TITLEBAR_ACTIVE);
        }
        ControlButton::Minimize => {
            // Horizontal line
            fill_rect(cx - 5, cy, 10, 1, icon_color);
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// MODERN WINDOW RENDERING
// ═══════════════════════════════════════════════════════════════════════════════

/// Draw a complete Windows 11 style window
pub fn draw_window(x: i32, y: i32, w: u32, h: u32, title: &str, focused: bool, maximized: bool) {
    let radius = if maximized { 0 } else { 8 };
    let titlebar_h = 32;
    
    // Drop shadow (only for non-maximized)
    if !maximized && focused {
        draw_shadow(x, y, w, h, radius, 0, 4, 4);
    }
    
    // Window background with rounded corners
    draw_rounded_rect(x, y, w, h, radius, colors::MICA_DARK);
    
    // Subtle border
    draw_rounded_rect_border(x, y, w, h, radius, colors::BORDER_SUBTLE);
    
    // Title bar (clipped to rounded corners at top)
    if radius > 0 {
        // Top corners only
        draw_corner_filled(x + radius as i32, y + radius as i32, radius, Corner::TopLeft, colors::TITLEBAR_ACTIVE);
        draw_corner_filled(x + w as i32 - radius as i32 - 1, y + radius as i32, radius, Corner::TopRight, colors::TITLEBAR_ACTIVE);
        fill_rect(x + radius as i32, y, (w - radius * 2) as u32, radius, 
            if focused { colors::TITLEBAR_ACTIVE } else { colors::TITLEBAR_INACTIVE });
    }
    draw_titlebar(x, y, w, titlebar_h, title, focused, maximized);
    
    // Content area (lighter)
    let content_y = y + titlebar_h as i32;
    let content_h = h - titlebar_h;
    fill_rect(x + 1, content_y, w - 2, content_h - 1, colors::MICA_DARKER);
}

// ═══════════════════════════════════════════════════════════════════════════════
// TASKBAR RENDERING
// ═══════════════════════════════════════════════════════════════════════════════

/// Draw Windows 11 centered taskbar
pub fn draw_taskbar(screen_w: u32, screen_h: u32, items: &[(&str, bool)]) {
    let taskbar_h = 48;
    let y = screen_h as i32 - taskbar_h as i32;
    
    // Taskbar background (acrylic effect simulation)
    fill_rect(0, y, screen_w, taskbar_h, colors::TASKBAR_BG);
    
    // Subtle top border
    draw_hline(0, y, screen_w, colors::BORDER_SUBTLE);
    
    // Center the icons
    let item_w = 48;
    let total_w = items.len() as u32 * item_w;
    let start_x = (screen_w - total_w) / 2;
    
    for (i, (name, active)) in items.iter().enumerate() {
        let ix = start_x as i32 + (i as i32 * item_w as i32);
        let iy = y + 4;
        
        // Hover background (rounded)
        draw_rounded_rect(ix + 2, iy, item_w - 4, 40, 4, colors::SURFACE);
        
        // Active indicator (accent line below)
        if *active {
            let indicator_w = 20;
            let indicator_x = ix + (item_w as i32 - indicator_w) / 2;
            draw_rounded_rect(indicator_x, y + taskbar_h as i32 - 4, indicator_w as u32, 3, 1, colors::ACCENT);
        }
        
        // Icon placeholder (would be real icons)
        let first_char = name.chars().next().unwrap_or(' ');
        let mut icon_buf = [0u8; 4];
        let icon_str = first_char.encode_utf8(&mut icon_buf);
        draw_text(ix + 16, iy + 12, icon_str, colors::TEXT_PRIMARY);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// FLUENT DESIGN BUTTONS
// ═══════════════════════════════════════════════════════════════════════════════

/// Draw a Windows 11 style button
pub fn draw_button(x: i32, y: i32, w: u32, h: u32, text: &str, primary: bool, hovered: bool, pressed: bool) {
    let bg = if pressed {
        if primary { colors::ACCENT_DARK } else { colors::SURFACE_PRESSED }
    } else if hovered {
        if primary { colors::ACCENT_LIGHT } else { colors::SURFACE_HOVER }
    } else {
        if primary { colors::ACCENT } else { colors::SURFACE }
    };
    
    // Button background
    draw_rounded_rect(x, y, w, h, 4, bg);
    
    // Border (subtle for non-primary)
    if !primary {
        draw_rounded_rect_border(x, y, w, h, 4, colors::BORDER_DEFAULT);
    }
    
    // Text (centered)
    let text_color = if primary { colors::TEXT_PRIMARY } else { colors::TEXT_PRIMARY };
    let text_w = text.len() as i32 * 8;
    let tx = x + (w as i32 - text_w) / 2;
    let ty = y + (h as i32 - 12) / 2;
    draw_text(tx, ty, text, text_color);
}

// ═══════════════════════════════════════════════════════════════════════════════
// HELPER FUNCTIONS (wrappers around framebuffer)
// ═══════════════════════════════════════════════════════════════════════════════

#[inline]
fn draw_pixel(x: i32, y: i32, color: u32) {
    if x >= 0 && y >= 0 {
        framebuffer::draw_pixel(x as u32, y as u32, color);
    }
}

#[inline]
fn fill_rect(x: i32, y: i32, w: u32, h: u32, color: u32) {
    if x >= 0 && y >= 0 {
        framebuffer::fill_rect(x as u32, y as u32, w, h, color);
    }
}

#[inline]
fn draw_hline(x: i32, y: i32, len: u32, color: u32) {
    if x >= 0 && y >= 0 {
        framebuffer::draw_hline(x as u32, y as u32, len, color);
    }
}

#[inline]
fn draw_vline(x: i32, y: i32, len: u32, color: u32) {
    if x >= 0 && y >= 0 {
        framebuffer::draw_vline(x as u32, y as u32, len, color);
    }
}

#[inline]
fn draw_rect_border(x: i32, y: i32, w: u32, h: u32, color: u32) {
    if x >= 0 && y >= 0 {
        framebuffer::draw_rect(x as u32, y as u32, w, h, color);
    }
}

#[inline]
fn draw_text(x: i32, y: i32, text: &str, color: u32) {
    for (i, c) in text.chars().enumerate() {
        let cx = x + (i * 8) as i32;
        if cx >= 0 && y >= 0 {
            framebuffer::draw_char_at(cx as u32, y as u32, c, color);
        }
    }
}
