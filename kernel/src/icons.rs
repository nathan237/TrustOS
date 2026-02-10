//! Pixel Art Icons for TrustOS Desktop
//!
//! 16x16 and 32x32 pixel icons for a professional look

use crate::framebuffer;
use libm::{cosf, sinf};

/// Icon size
pub const ICON_SIZE: u32 = 32;
pub const ICON_SMALL: u32 = 16;

/// Draw a 32x32 terminal icon
pub fn draw_terminal_icon(x: u32, y: u32, color: u32, bg: u32) {
    // Terminal window shape
    let dark = darken(color, 0.6);
    let light = lighten(color, 1.3);
    
    // Window frame
    framebuffer::fill_rect(x, y, 32, 32, bg);
    framebuffer::fill_rect(x + 2, y + 2, 28, 28, dark);
    
    // Title bar
    framebuffer::fill_rect(x + 2, y + 2, 28, 6, color);
    
    // Window buttons (3 dots)
    framebuffer::fill_rect(x + 4, y + 4, 2, 2, 0xFFFF5555); // red
    framebuffer::fill_rect(x + 8, y + 4, 2, 2, 0xFFFFAA00); // yellow
    framebuffer::fill_rect(x + 12, y + 4, 2, 2, 0xFF55FF55); // green
    
    // Terminal content area
    framebuffer::fill_rect(x + 4, y + 10, 24, 18, 0xFF0A0A0A);
    
    // Prompt: ">" and cursor "_"
    framebuffer::fill_rect(x + 6, y + 14, 2, 6, color);  // >
    framebuffer::fill_rect(x + 8, y + 17, 2, 2, color);  // > bottom
    framebuffer::fill_rect(x + 12, y + 20, 8, 2, light); // cursor _
}

/// Draw a 32x32 folder icon
pub fn draw_folder_icon(x: u32, y: u32, color: u32, _bg: u32) {
    let dark = darken(color, 0.7);
    let light = lighten(color, 1.2);
    
    // Folder tab
    framebuffer::fill_rect(x + 2, y + 6, 12, 4, light);
    
    // Folder body
    framebuffer::fill_rect(x + 2, y + 8, 28, 18, color);
    
    // Folder front (3D effect)
    framebuffer::fill_rect(x + 2, y + 12, 28, 14, dark);
    
    // Highlight edge
    framebuffer::fill_rect(x + 2, y + 12, 28, 2, light);
    
    // Shadow
    framebuffer::fill_rect(x + 4, y + 24, 26, 2, darken(dark, 0.5));
}

/// Draw a 32x32 file icon
pub fn draw_file_icon(x: u32, y: u32, color: u32, _bg: u32) {
    let dark = darken(color, 0.8);
    let light = lighten(color, 1.2);
    
    // Paper background
    framebuffer::fill_rect(x + 4, y + 2, 20, 28, 0xFFEEEEEE);
    
    // Folded corner
    framebuffer::fill_rect(x + 18, y + 2, 6, 6, 0xFFCCCCCC);
    framebuffer::fill_rect(x + 18, y + 2, 1, 6, 0xFFDDDDDD);
    
    // Content lines
    for i in 0..5 {
        framebuffer::fill_rect(x + 8, y + 10 + i * 4, 12, 2, dark);
    }
    
    // Border
    framebuffer::draw_rect(x + 4, y + 2, 20, 28, color);
}

/// Draw a 32x32 gear/settings icon
pub fn draw_settings_icon(x: u32, y: u32, color: u32, _bg: u32) {
    let cx = x + 16;
    let cy = y + 16;
    let light = lighten(color, 1.2);
    
    // Center circle
    draw_filled_circle(cx, cy, 6, color);
    draw_filled_circle(cx, cy, 3, 0xFF0A0A0A);
    
    // Gear teeth (8 teeth around)
    let teeth: [(i32, i32); 8] = [
        (0, -10), (7, -7), (10, 0), (7, 7),
        (0, 10), (-7, 7), (-10, 0), (-7, -7),
    ];
    for (dx, dy) in teeth {
        framebuffer::fill_rect(
            (cx as i32 + dx - 2) as u32,
            (cy as i32 + dy - 2) as u32,
            5, 5, color
        );
    }
}

/// Draw a 32x32 calculator icon
pub fn draw_calculator_icon(x: u32, y: u32, color: u32, _bg: u32) {
    let dark = darken(color, 0.6);
    
    // Calculator body
    framebuffer::fill_rect(x + 4, y + 2, 24, 28, dark);
    framebuffer::draw_rect(x + 4, y + 2, 24, 28, color);
    
    // Display
    framebuffer::fill_rect(x + 6, y + 4, 20, 8, 0xFF1A2A1A);
    framebuffer::fill_rect(x + 18, y + 6, 6, 4, color); // Number
    
    // Buttons grid (4x4)
    for row in 0..4 {
        for col in 0..4 {
            let bx = x + 6 + col * 5;
            let by = y + 14 + row * 4;
            let btn_color = if col == 3 { 0xFF44AA44 } else { 0xFF333333 };
            framebuffer::fill_rect(bx, by, 4, 3, btn_color);
        }
    }
}

/// Draw a 32x32 network icon
pub fn draw_network_icon(x: u32, y: u32, color: u32, _bg: u32) {
    let light = lighten(color, 1.2);
    
    // Globe circle
    draw_circle(x + 16, y + 16, 12, color);
    
    // Horizontal lines (latitude)
    framebuffer::fill_rect(x + 6, y + 11, 20, 1, color);
    framebuffer::fill_rect(x + 4, y + 16, 24, 1, color);
    framebuffer::fill_rect(x + 6, y + 21, 20, 1, color);
    
    // Vertical ellipse (longitude)
    draw_circle(x + 16, y + 16, 6, color);
    
    // Center meridian
    framebuffer::fill_rect(x + 15, y + 4, 2, 24, color);
}

/// Draw a 32x32 info/about icon
pub fn draw_about_icon(x: u32, y: u32, color: u32, _bg: u32) {
    let light = lighten(color, 1.3);
    
    // Circle
    draw_filled_circle(x + 16, y + 16, 12, color);
    draw_filled_circle(x + 16, y + 16, 10, 0xFF0A0A0A);
    
    // "i" letter
    framebuffer::fill_rect(x + 14, y + 10, 4, 4, light); // dot
    framebuffer::fill_rect(x + 14, y + 16, 4, 10, light); // stem
    framebuffer::fill_rect(x + 12, y + 24, 8, 2, light); // base
}

/// Draw a 32x32 game controller icon (for games like Snake)
pub fn draw_game_icon(x: u32, y: u32, color: u32, _bg: u32) {
    let dark = darken(color, 0.7);
    let light = lighten(color, 1.2);
    
    // Controller body
    framebuffer::fill_rect(x + 4, y + 10, 24, 14, dark);
    framebuffer::fill_rect(x + 2, y + 12, 4, 10, dark);
    framebuffer::fill_rect(x + 26, y + 12, 4, 10, dark);
    
    // D-pad
    framebuffer::fill_rect(x + 8, y + 14, 2, 6, color);
    framebuffer::fill_rect(x + 6, y + 16, 6, 2, color);
    
    // Buttons
    framebuffer::fill_rect(x + 22, y + 14, 3, 3, 0xFF55FF55);
    framebuffer::fill_rect(x + 18, y + 17, 3, 3, 0xFFFF5555);
}

/// Draw a 32x32 text editor icon
pub fn draw_editor_icon(x: u32, y: u32, color: u32, _bg: u32) {
    let dark = darken(color, 0.7);
    
    // Paper
    framebuffer::fill_rect(x + 4, y + 2, 24, 28, 0xFFEEEEEE);
    framebuffer::draw_rect(x + 4, y + 2, 24, 28, dark);
    
    // Text lines
    framebuffer::fill_rect(x + 8, y + 6, 16, 2, color);
    framebuffer::fill_rect(x + 8, y + 10, 14, 2, dark);
    framebuffer::fill_rect(x + 8, y + 14, 16, 2, dark);
    framebuffer::fill_rect(x + 8, y + 18, 10, 2, dark);
    framebuffer::fill_rect(x + 8, y + 22, 14, 2, dark);
    
    // Cursor
    framebuffer::fill_rect(x + 10, y + 22, 1, 4, color);
}

// ═══════════════════════════════════════════════════════════════════════════════
// HELPER FUNCTIONS
// ═══════════════════════════════════════════════════════════════════════════════

/// Darken a color by a factor (0.0 - 1.0)
fn darken(color: u32, factor: f32) -> u32 {
    let r = ((color >> 16) & 0xFF) as f32 * factor;
    let g = ((color >> 8) & 0xFF) as f32 * factor;
    let b = (color & 0xFF) as f32 * factor;
    0xFF000000 | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}

/// Lighten a color by a factor (> 1.0)
fn lighten(color: u32, factor: f32) -> u32 {
    let r = (((color >> 16) & 0xFF) as f32 * factor).min(255.0);
    let g = (((color >> 8) & 0xFF) as f32 * factor).min(255.0);
    let b = ((color & 0xFF) as f32 * factor).min(255.0);
    0xFF000000 | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}

/// Draw a filled circle using Bresenham
fn draw_filled_circle(cx: u32, cy: u32, r: u32, color: u32) {
    let r = r as i32;
    let cx = cx as i32;
    let cy = cy as i32;
    
    for dy in -r..=r {
        for dx in -r..=r {
            if dx * dx + dy * dy <= r * r {
                let px = cx + dx;
                let py = cy + dy;
                if px >= 0 && py >= 0 {
                    framebuffer::put_pixel(px as u32, py as u32, color);
                }
            }
        }
    }
}

/// Draw a circle outline
fn draw_circle(cx: u32, cy: u32, r: u32, color: u32) {
    let r = r as i32;
    let cx = cx as i32;
    let cy = cy as i32;
    
    let mut x = r;
    let mut y = 0;
    let mut err = 0;
    
    while x >= y {
        put_pixel_safe(cx + x, cy + y, color);
        put_pixel_safe(cx + y, cy + x, color);
        put_pixel_safe(cx - y, cy + x, color);
        put_pixel_safe(cx - x, cy + y, color);
        put_pixel_safe(cx - x, cy - y, color);
        put_pixel_safe(cx - y, cy - x, color);
        put_pixel_safe(cx + y, cy - x, color);
        put_pixel_safe(cx + x, cy - y, color);
        
        y += 1;
        err += 1 + 2 * y;
        if 2 * (err - x) + 1 > 0 {
            x -= 1;
            err += 1 - 2 * x;
        }
    }
}

fn put_pixel_safe(x: i32, y: i32, color: u32) {
    if x >= 0 && y >= 0 {
        framebuffer::put_pixel(x as u32, y as u32, color);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ICON TYPE ENUM
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Clone, Copy, PartialEq)]
pub enum IconType {
    Terminal,
    Folder,
    File,
    Settings,
    Calculator,
    Network,
    About,
    Game,
    Editor,
    OpenGL,
    Browser,
    ModelEditor,
}

/// Draw an icon by type
pub fn draw_icon(icon_type: IconType, x: u32, y: u32, color: u32, bg: u32) {
    match icon_type {
        IconType::Terminal => draw_terminal_icon(x, y, color, bg),
        IconType::Folder => draw_folder_icon(x, y, color, bg),
        IconType::File => draw_file_icon(x, y, color, bg),
        IconType::Settings => draw_settings_icon(x, y, color, bg),
        IconType::Calculator => draw_calculator_icon(x, y, color, bg),
        IconType::Network => draw_network_icon(x, y, color, bg),
        IconType::About => draw_about_icon(x, y, color, bg),
        IconType::Game => draw_game_icon(x, y, color, bg),
        IconType::Editor => draw_editor_icon(x, y, color, bg),
        IconType::OpenGL => draw_opengl_icon(x, y, color, bg),
        IconType::Browser => draw_browser_icon(x, y, color, bg),
        IconType::ModelEditor => draw_model_editor_icon(x, y, color, bg),
    }
}

/// Draw a 32x32 OpenGL/3D cube icon
pub fn draw_opengl_icon(x: u32, y: u32, color: u32, _bg: u32) {
    let dark = darken(color, 0.5);
    let light = lighten(color, 1.4);
    
    // Draw a 3D cube wireframe using i32 for calculations
    let cx = x as i32 + 16;
    let cy = y as i32 + 18;
    let size: i32 = 8;
    
    // Helper to put pixel with i32 coords
    let put = |px: i32, py: i32, c: u32| {
        if px >= 0 && py >= 0 {
            framebuffer::put_pixel(px as u32, py as u32, c);
        }
    };
    
    // Front square (bright)
    for i in 0..=size {
        put(cx - size + i, cy - size, light);
        put(cx + size, cy - size + i, light);
        put(cx + size - i, cy + size, light);
        put(cx - size, cy + size - i, light);
    }
    
    // Back square (dark)
    let offset: i32 = 5;
    for i in 0..=size {
        put(cx - size + i + offset, cy - size - offset, dark);
        put(cx + size + offset, cy - size + i - offset, dark);
        put(cx + size - i + offset, cy + size - offset, dark);
        put(cx - size + offset, cy + size - i - offset, dark);
    }
    
    // Connecting lines (color)
    for i in 0..offset {
        put(cx - size + i, cy - size - i, color);
        put(cx + size + i, cy - size - i, color);
        put(cx + size + i, cy + size - i, color);
        put(cx - size + i, cy + size - i, color);
    }
    
    // "GL" text at bottom
    let tx = x as i32;
    let ty = y as i32;
    // G
    put(tx + 10, ty + 26, light);
    put(tx + 11, ty + 26, light);
    put(tx + 12, ty + 26, light);
    put(tx + 9, ty + 27, light);
    put(tx + 9, ty + 28, light);
    put(tx + 10, ty + 29, light);
    put(tx + 11, ty + 29, light);
    put(tx + 12, ty + 29, light);
    put(tx + 12, ty + 28, light);
    
    // L
    put(tx + 15, ty + 26, light);
    put(tx + 15, ty + 27, light);
    put(tx + 15, ty + 28, light);
    put(tx + 15, ty + 29, light);
    put(tx + 16, ty + 29, light);
    put(tx + 17, ty + 29, light);
}

/// Draw a 32x32 browser icon (globe with navigation)
pub fn draw_browser_icon(x: u32, y: u32, color: u32, _bg: u32) {
    let dark = darken(color, 0.6);
    let light = lighten(color, 1.3);
    
    let cx = x as i32 + 16;
    let cy = y as i32 + 16;
    
    // Draw globe (outer circle)
    for angle in 0..360 {
        let rad = (angle as f32) * 3.14159 / 180.0;
        let px = cx + (cosf(rad) * 12.0) as i32;
        let py = cy + (sinf(rad) * 12.0) as i32;
        if px >= 0 && py >= 0 {
            framebuffer::put_pixel(px as u32, py as u32, color);
        }
    }
    
    // Inner globe (smaller circle)
    for angle in 0..360 {
        let rad = (angle as f32) * 3.14159 / 180.0;
        let px = cx + (cosf(rad) * 8.0) as i32;
        let py = cy + (sinf(rad) * 8.0) as i32;
        if px >= 0 && py >= 0 {
            framebuffer::put_pixel(px as u32, py as u32, dark);
        }
    }
    
    // Vertical meridian
    for dy in -12i32..=12 {
        let py = cy + dy;
        if py >= 0 {
            framebuffer::put_pixel(cx as u32, py as u32, light);
        }
    }
    
    // Horizontal equator
    for dx in -12i32..=12 {
        let px = cx + dx;
        if px >= 0 {
            framebuffer::put_pixel(px as u32, cy as u32, light);
        }
    }
    
    // Curved latitude lines (simplified as horizontal lines at offsets)
    for dx in -10i32..=10 {
        let px = cx + dx;
        if px >= 0 {
            framebuffer::put_pixel(px as u32, (cy - 6) as u32, dark);
            framebuffer::put_pixel(px as u32, (cy + 6) as u32, dark);
        }
    }
    
    // Navigation bar at bottom (address bar)
    framebuffer::fill_rect(x + 2, y + 26, 28, 4, dark);
    framebuffer::fill_rect(x + 4, y + 27, 24, 2, 0xFF202020);
}

/// Draw a 32x32 3D model editor icon (wireframe with cursor cross)
pub fn draw_model_editor_icon(x: u32, y: u32, color: u32, _bg: u32) {
    let dark = darken(color, 0.5);
    let light = lighten(color, 1.4);
    let accent = 0xFF00FFAA; // green wireframe color
    
    let cx = x as i32 + 16;
    let cy = y as i32 + 16;
    
    // Draw wireframe cube (simplified)
    let put = |px: i32, py: i32, c: u32| {
        if px >= 0 && py >= 0 {
            framebuffer::put_pixel(px as u32, py as u32, c);
        }
    };
    
    // Front face
    let s: i32 = 7;
    for i in 0..=s {
        put(cx - s + i - 2, cy - s + 2, accent);
        put(cx + s - 2, cy - s + i + 2, accent);
        put(cx + s - i - 2, cy + s + 2, accent);
        put(cx - s - 2, cy + s - i + 2, accent);
    }
    
    // Back face (offset)
    let o: i32 = 5;
    for i in 0..=s {
        put(cx - s + i + o - 2, cy - s - o + 2, dark);
        put(cx + s + o - 2, cy - s + i - o + 2, dark);
    }
    
    // Connect corners
    for i in 0..o {
        put(cx - s + i - 2, cy - s - i + 2, light);
        put(cx + s + i - 2, cy - s - i + 2, light);
        put(cx + s + i - 2, cy + s - i + 2, light);
    }
    
    // Crosshair cursor at bottom-right
    for i in 0..5 {
        put(cx + 8, cy + 6 + i, 0xFFFFFFFF);
        put(cx + 6 + i, cy + 8, 0xFFFFFFFF);
    }
    
    // Vertex dots
    put(cx - s - 2, cy - s + 2, 0xFFFFFF00);
    put(cx + s - 2, cy + s + 2, 0xFFFFFF00);
    put(cx + s + o - 2, cy - s - o + 2, 0xFFFFFF00);
}

