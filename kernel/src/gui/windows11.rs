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
        if x >= 0 && y >= 0 {
            framebuffer::draw_rect(x as u32, y as u32, w, h, color);
        }
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
