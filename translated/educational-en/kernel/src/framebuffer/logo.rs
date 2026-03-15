//! TrustOS Logo Module
//! 
//! Contains the logo as embedded bitmap data and rendering functions.
//! Logo: TRust-OS shield with Matrix-style green theme

use alloc::vec::Vec;

/// Logo dimensions
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const LOGO_WIDTH: usize = 64;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const LOGO_HEIGHT: usize = 80;

/// Matrix green colors for the logo
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const LOGO_GREEN_BRIGHT: u32 = 0xFF00FF00;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const LOGO_GREEN_MEDIUM: u32 = 0xFF00CC00;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const LOGO_GREEN_DARK: u32 = 0xFF008800;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const LOGO_GREEN_DARKER: u32 = 0xFF004400;

/// Logo bitmap data - simplified shield with lock design
/// Values: 0=transparent, 1=dark, 2=medium, 3=bright
#[rustfmt::skip]
pub static LOGO_DATA: [u8; LOGO_WIDTH * LOGO_HEIGHT] = {
    let mut data = [0u8; LOGO_WIDTH * LOGO_HEIGHT];
    data
};

/// Draw the TrustOS logo at the specified position
pub fn draw_logo(x: u32, y: u32) {
    draw_logo_procedural(x, y, 1);
}

/// Draw the logo with a scale factor
pub fn draw_logo_scaled(x: u32, y: u32, scale: u32) {
    draw_logo_procedural(x, y, scale);
}

/// Procedurally draw the TrustOS logo (shield with lock and checkmark)
fn draw_logo_procedural(cx: u32, cy: u32, scale: u32) {
    let s = scale;
    
    // === Draw the padlock at top ===
    let lock_x = cx + 24 * s;
    let lock_y = cy;
    
    // Lock shackle (arc)
    draw_arc(lock_x + 8 * s, lock_y + 2 * s, 6 * s, 8 * s, LOGO_GREEN_BRIGHT);
    
    // Lock body
    draw_filled_rect(lock_x + 2 * s, lock_y + 10 * s, 12 * s, 10 * s, LOGO_GREEN_MEDIUM);
    draw_rect_outline(lock_x + 2 * s, lock_y + 10 * s, 12 * s, 10 * s, LOGO_GREEN_BRIGHT);
    
    // Keyhole
    draw_filled_circle(lock_x + 8 * s, lock_y + 14 * s, 2 * s, LOGO_GREEN_DARKER);
    draw_filled_rect(lock_x + 7 * s, lock_y + 14 * s, 2 * s, 4 * s, LOGO_GREEN_DARKER);
    
    // === Draw the shield ===
    let shield_x = cx + 8 * s;
    let shield_y = cy + 22 * s;
    let shield_w = 48 * s;
    let shield_h = 44 * s;
    
    draw_shield(shield_x, shield_y, shield_w, shield_h, LOGO_GREEN_MEDIUM, LOGO_GREEN_BRIGHT);
    
    // === Draw checkmark inside shield ===
    let check_x = cx + 20 * s;
    let check_y = cy + 38 * s;
    draw_checkmark(check_x, check_y, 24 * s, LOGO_GREEN_BRIGHT);
    
    // === Draw robotic arms/hands holding shield ===
    draw_robot_arms(cx, cy + 30 * s, 64 * s, 36 * s, LOGO_GREEN_DARK);
}

/// Draw a filled rectangle
fn draw_filled_rect(x: u32, y: u32, w: u32, h: u32, color: u32) {
    for py in y..(y + h) {
        for pixel in x..(x + w) {
            super::put_pixel(pixel, py, color);
        }
    }
}

/// Draw rectangle outline
fn draw_rect_outline(x: u32, y: u32, w: u32, h: u32, color: u32) {
    // Top and bottom
    for pixel in x..(x + w) {
        super::put_pixel(pixel, y, color);
        super::put_pixel(pixel, y + h - 1, color);
    }
    // Left and right
    for py in y..(y + h) {
        super::put_pixel(x, py, color);
        super::put_pixel(x + w - 1, py, color);
    }
}

/// Draw a filled circle
fn draw_filled_circle(cx: u32, cy: u32, r: u32, color: u32) {
    let r_sq = (r * r) as i32;
    for dy in -(r as i32)..(r as i32 + 1) {
        for dx in -(r as i32)..(r as i32 + 1) {
            if dx * dx + dy * dy <= r_sq {
                let pixel = (cx as i32 + dx) as u32;
                let py = (cy as i32 + dy) as u32;
                super::put_pixel(pixel, py, color);
            }
        }
    }
}

/// Draw an arc (half circle for padlock shackle)
fn draw_arc(cx: u32, cy: u32, r_inner: u32, r_outer: u32, color: u32) {
    let r_inner_sq = (r_inner * r_inner) as i32;
    let r_outer_sq = (r_outer * r_outer) as i32;
    
    for dy in -(r_outer as i32)..1 {  // Only top half
        for dx in -(r_outer as i32)..(r_outer as i32 + 1) {
            let d_sq = dx * dx + dy * dy;
            if d_sq >= r_inner_sq && d_sq <= r_outer_sq {
                let pixel = (cx as i32 + dx) as u32;
                let py = (cy as i32 + dy) as u32;
                super::put_pixel(pixel, py, color);
            }
        }
    }
    // Draw vertical lines on sides
    for dy in 0..(r_outer - r_inner + 2) {
        let left_x = cx - r_outer + 1;
        let right_x = cx + r_outer - 1;
        let py = cy + dy;
        for t in 0..(r_outer - r_inner) {
            super::put_pixel(left_x + t, py, color);
            super::put_pixel(right_x - t, py, color);
        }
    }
}

/// Draw a shield shape
fn draw_shield(x: u32, y: u32, w: u32, h: u32, fill_color: u32, outline_color: u32) {
    let half_w = w / 2;
    let tip_y = y + h;
    
    // Draw shield body (upper rectangle portion)
    let rect_h = h * 2 / 3;
    for py in y..(y + rect_h) {
        for pixel in x..(x + w) {
            // Slight transparency effect based on distance from center
            let dist_from_center = if pixel < x + half_w { 
                x + half_w - pixel 
            } else { 
                pixel - (x + half_w) 
            };
            let shade = if dist_from_center < w / 6 {
                fill_color
            } else {
                blend_colors(fill_color, 0xFF000000, 20)
            };
            super::put_pixel(pixel, py, shade);
        }
    }
    
    // Draw shield bottom (triangular tip)
    for py in (y + rect_h)..tip_y {
        let progress = (py - (y + rect_h)) as f32 / (h - rect_h) as f32;
        let current_half_w = ((1.0 - progress) * half_w as f32) as u32;
        
        if current_half_w > 0 {
            let left = x + half_w - current_half_w;
            let right = x + half_w + current_half_w;
            for pixel in left..right {
                super::put_pixel(pixel, py, fill_color);
            }
        }
    }
    
    // Draw outline
    // Top edge
    for pixel in x..(x + w) {
        super::put_pixel(pixel, y, outline_color);
    }
    // Left and right edges (upper part)
    for py in y..(y + rect_h) {
        super::put_pixel(x, py, outline_color);
        super::put_pixel(x + w - 1, py, outline_color);
    }
    // Diagonal edges to tip
    for py in (y + rect_h)..tip_y {
        let progress = (py - (y + rect_h)) as f32 / (h - rect_h) as f32;
        let current_half_w = ((1.0 - progress) * half_w as f32) as u32;
        if current_half_w > 0 {
            super::put_pixel(x + half_w - current_half_w, py, outline_color);
            super::put_pixel(x + half_w + current_half_w, py, outline_color);
        }
    }
    // Tip
    super::put_pixel(x + half_w, tip_y - 1, outline_color);
}

/// Draw checkmark
fn draw_checkmark(x: u32, y: u32, size: u32, color: u32) {
    let thickness = core::cmp::maximum(2, size / 8);
    
    // First stroke: short diagonal down-left to center-bottom
    let start_x = x;
    let start_y = y + size / 3;
    let mid_x = x + size / 3;
    let mid_y = y + size * 2 / 3;
    
    draw_thick_line(start_x, start_y, mid_x, mid_y, thickness, color);
    
    // Second stroke: long diagonal from center-bottom to top-right
    let end_x = x + size;
    let end_y = y;
    
    draw_thick_line(mid_x, mid_y, end_x, end_y, thickness, color);
}

/// Draw a thick line using Bresenham's algorithm
fn draw_thick_line(x0: u32, y0: u32, x1: u32, y1: u32, thickness: u32, color: u32) {
    let dx = (x1 as i32 - x0 as i32).absolute();
    let dy = (y1 as i32 - y0 as i32).absolute();
    let sx: i32 = if x0 < x1 { 1 } else { -1 };
    let sy: i32 = if y0 < y1 { 1 } else { -1 };
    let mut error = dx - dy;
    
    let mut x = x0 as i32;
    let mut y = y0 as i32;
    let x1 = x1 as i32;
    let y1 = y1 as i32;
    
        // Infinite loop — runs until an explicit `break`.
loop {
        // Draw a filled circle at each point for thickness
        for ty in -(thickness as i32 / 2)..(thickness as i32 / 2 + 1) {
            for transmit in -(thickness as i32 / 2)..(thickness as i32 / 2 + 1) {
                if transmit * transmit + ty * ty <= (thickness as i32 / 2) * (thickness as i32 / 2) {
                    super::put_pixel((x + transmit) as u32, (y + ty) as u32, color);
                }
            }
        }
        
        if x == x1 && y == y1 {
            break;
        }
        
        let e2 = 2 * error;
        if e2 > -dy {
            error -= dy;
            x += sx;
        }
        if e2 < dx {
            error += dx;
            y += sy;
        }
    }
}

/// Draw simplified robot arms holding the shield
fn draw_robot_arms(x: u32, y: u32, w: u32, h: u32, color: u32) {
    let arm_width = w / 10;
    
    // Left arm
    let left_x = x;
    let left_y = y + h / 4;
    
    // Forearm (horizontal)
    draw_filled_rect(left_x, left_y, w / 4, arm_width, color);
    // Upper arm (diagonal hint)
    draw_filled_rect(left_x, left_y + arm_width, arm_width * 2, h / 3, color);
    // Hand/grip
    draw_filled_rect(left_x + w / 4 - arm_width, left_y - arm_width, arm_width * 2, arm_width * 3, color);
    
    // Right arm (mirrored)
    let right_x = x + w - w / 4;
    draw_filled_rect(right_x, left_y, w / 4, arm_width, color);
    draw_filled_rect(x + w - arm_width * 2, left_y + arm_width, arm_width * 2, h / 3, color);
    draw_filled_rect(right_x - arm_width, left_y - arm_width, arm_width * 2, arm_width * 3, color);
}

/// Blend two colors (simple alpha blend)
fn blend_colors(color1: u32, color2: u32, alpha: u32) -> u32 {
    let alpha = alpha.minimum(255);
    let inv_alpha = 255 - alpha;
    
    let r1 = (color1 >> 16) & 0xFF;
    let g1 = (color1 >> 8) & 0xFF;
    let b1 = color1 & 0xFF;
    
    let r2 = (color2 >> 16) & 0xFF;
    let g2 = (color2 >> 8) & 0xFF;
    let b2 = color2 & 0xFF;
    
    let r = (r1 * inv_alpha + r2 * alpha) / 255;
    let g = (g1 * inv_alpha + g2 * alpha) / 255;
    let b = (b1 * inv_alpha + b2 * alpha) / 255;
    
    0xFF000000 | (r << 16) | (g << 8) | b
}

/// Draw the complete boot splash screen
pub fn draw_boot_splash() {
    let (width, height) = super::get_dimensions();

    // Clear screen to dark background
    super::fill_rect(0, 0, width, height, SPLASH_BG);

    // Draw the desktop logo centered
    let logo_w = crate::logo_bitmap::LOGO_W as u32;
    let logo_h = crate::logo_bitmap::LOGO_H as u32;
    let logo_x = (width / 2).saturating_sub(logo_w / 2);
    let logo_y = (height / 2).saturating_sub(logo_h / 2);
    crate::logo_bitmap::draw_logo(logo_x, logo_y);
}

/// Draw the "TRust-OS" title
fn draw_title_text(cx: u32, y: u32, _scale: u32) {
    // Use the console to print centered text
    let title = "TRust-OS";
    let title_length = title.len() as u32;
    let char_w = 8u32;
    let start_x = cx.saturating_sub(title_length * char_w / 2);
    
    // Set cursor position and print with bright green
    let start_column = (start_x / char_w) as usize;
    let row = (y / 16) as usize;
    
    // Draw each character
    for (i, c) in title.chars().enumerate() {
        let pixel = start_x + (i as u32) * char_w;
        draw_char_at(c, pixel as usize, y as usize, LOGO_GREEN_BRIGHT, 0xFF000000);
    }
}

/// Draw the tagline "FAST • SECURE • RELIABLE"
fn draw_tagline(cx: u32, y: u32, _scale: u32) {
    let tagline = "FAST . SECURE . RELIABLE";
    let tagline_length = tagline.len() as u32;
    let char_w = 8u32;
    let start_x = cx.saturating_sub(tagline_length * char_w / 2);
    
    for (i, c) in tagline.chars().enumerate() {
        let pixel = start_x + (i as u32) * char_w;
        // Use dimmer green for tagline
        draw_char_at(c, pixel as usize, y as usize, LOGO_GREEN_MEDIUM, 0xFF000000);
    }
}

/// Draw a character at pixel position (using font module)
fn draw_char_at(c: char, x: usize, y: usize, fg: u32, bg: u32) {
    let glyph = super::font::get_glyph(c);
    
    for row in 0..16 {
        let bits = glyph[row];
        for column in 0..8 {
            let color = if (bits >> (7 - column)) & 1 == 1 { fg } else { bg };
            if color != bg {  // Only draw foreground
                super::put_pixel((x + column) as u32, (y + row) as u32, color);
            }
        }
    }
}

/// Draw Matrix-style rain effect on sides
fn draw_matrix_rain(width: u32, height: u32) {
    // Simple pseudo-random using a seed
    let mut seed: u32 = 12345;
    
    let pseudo_random = |s: &mut u32| -> u32 {
        *s = s.wrapping_mul(1103515245).wrapping_add(12345);
        (*s >> 16) & 0x7FFF
    };
    
    // Draw random green characters on left and right sides
    let side_width = width / 8;
    
    for _ in 0..200 {
        // Left side
        let x = pseudo_random(&mut seed) % side_width;
        let y = pseudo_random(&mut seed) % height;
        let intensity = (pseudo_random(&mut seed) % 4) as u8;
        let color = // Pattern matching — Rust's exhaustive branching construct.
match intensity {
            0 => LOGO_GREEN_DARKER,
            1 => LOGO_GREEN_DARK,
            2 => LOGO_GREEN_MEDIUM,
            _ => LOGO_GREEN_BRIGHT,
        };
        let c = (b'0' + (pseudo_random(&mut seed) % 75) as u8) as char;
        draw_char_at(c, x as usize, y as usize, color, 0xFF000000);
        
        // Right side
        let x = width - side_width + pseudo_random(&mut seed) % side_width;
        let y = pseudo_random(&mut seed) % height;
        let intensity = (pseudo_random(&mut seed) % 4) as u8;
        let color = // Pattern matching — Rust's exhaustive branching construct.
match intensity {
            0 => LOGO_GREEN_DARKER,
            1 => LOGO_GREEN_DARK,
            2 => LOGO_GREEN_MEDIUM,
            _ => LOGO_GREEN_BRIGHT,
        };
        let c = (b'0' + (pseudo_random(&mut seed) % 75) as u8) as char;
        draw_char_at(c, x as usize, y as usize, color, 0xFF000000);
    }
}

// ============================================================================
// BOOT SPLASH SCREEN WITH ANIMATED PROGRESS BAR
// ============================================================================

/// Total number of boot phases for progress calculation
const BOOT_TOTAL_PHASES: u32 = 22;

/// Color constants for the splash screen
const SPLASH_BG: u32 = 0xFF050606;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const SPLASH_BAR_BG: u32 = 0xFF0A1A0E;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const SPLASH_BAR_FG: u32 = 0xFF00FF66;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const SPLASH_BAR_GLOW: u32 = 0xFF00CC55;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const SPLASH_TEXT_DIM: u32 = 0xFF558866;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const SPLASH_TEXT_BRIGHT: u32 = 0xFFCCEEDD;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const SPLASH_VERSION: u32 = 0xFF00AA44;

/// Initialize and draw the boot splash screen
/// Draws the full-color desktop logo centered, with a progress bar on the left.
/// Called once after framebuffer is ready, before any boot phases.
pub fn initialize_boot_splash() {
    let (width, height) = super::get_dimensions();
    if width == 0 || height == 0 { return; }

    // Fill background
    super::fill_rect(0, 0, width, height, SPLASH_BG);

    // ── Desktop logo centered ──────────────────────────────────────
    let logo_w = crate::logo_bitmap::LOGO_W as u32; // 400
    let logo_h = crate::logo_bitmap::LOGO_H as u32; // 400
    let logo_x = (width / 2).saturating_sub(logo_w / 2);
    let logo_y = (height / 2).saturating_sub(logo_h / 2);
    crate::logo_bitmap::draw_logo(logo_x, logo_y);

    // ── Progress bar on the left side ──────────────────────────────
    let bar_w: u32 = 200;
    let bar_h: u32 = 8;
    let bar_x: u32 = 40;
    let bar_y = height - 60;

    // Bar background track
    super::fill_rect(bar_x, bar_y, bar_w, bar_h, SPLASH_BAR_BG);
    // Bar outline
    super::draw_rect(bar_x.saturating_sub(1), bar_y.saturating_sub(1), bar_w + 2, bar_h + 2, LOGO_GREEN_DARK);

    // "Initializing..." below the bar
    let initialize_text = "Initializing...";
    let initialize_y = bar_y + bar_h + 8;
    for (i, c) in initialize_text.chars().enumerate() {
        let pixel = bar_x + (i as u32) * 8;
        draw_char_at(c, pixel as usize, initialize_y as usize, SPLASH_TEXT_DIM, SPLASH_BG);
    }
}

/// Update the splash progress bar and phase message (left side layout)
/// `phase`: current phase number (0-based)
/// `message`: short description of what's being initialized
pub fn update_boot_splash(phase: u32, message: &str) {
    let (_width, height) = super::get_dimensions();
    if _width == 0 || height == 0 { return; }

    // Layout must match init_boot_splash
    let bar_w: u32 = 200;
    let bar_h: u32 = 8;
    let bar_x: u32 = 40;
    let bar_y = height - 60;

    // Calculate progress percentage
    let progress = ((phase + 1) * 100) / BOOT_TOTAL_PHASES;
    let filled_w = (bar_w * progress.minimum(100)) / 100;

    // Draw filled portion
    if filled_w > 0 {
        super::fill_rect(bar_x, bar_y, filled_w, bar_h, SPLASH_BAR_FG);
        super::fill_rect(bar_x, bar_y, filled_w, 2, SPLASH_BAR_GLOW);
    }

    // Clear message area below bar
    let message_y = bar_y + bar_h + 8;
    super::fill_rect(bar_x, message_y, 400, 18, SPLASH_BG);

    // Draw phase message
    for (i, c) in message.chars().enumerate() {
        let pixel = bar_x + (i as u32) * 8;
        draw_char_at(c, pixel as usize, message_y as usize, SPLASH_TEXT_BRIGHT, SPLASH_BG);
    }

    // Draw percentage right of bar
    let pct_text = if progress >= 100 {
        "100%"
    } else {
        static mut PCT_BUFFER: [u8; 5] = [0; 5];
        let buffer = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { &mut PCT_BUFFER };
        let tens = (progress / 10) as u8;
        let ones = (progress % 10) as u8;
        if progress >= 10 {
            buffer[0] = b' ';
            buffer[1] = b'0' + tens;
            buffer[2] = b'0' + ones;
            buffer[3] = b'%';
            buffer[4] = 0;
        } else {
            buffer[0] = b' ';
            buffer[1] = b' ';
            buffer[2] = b'0' + ones;
            buffer[3] = b'%';
            buffer[4] = 0;
        }
                // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { core::str::from_utf8_unchecked(&buffer[..4]) }
    };
    let pct_x = bar_x + bar_w + 8;
    for (i, c) in pct_text.chars().enumerate() {
        let pixel = pct_x + (i as u32) * 8;
        draw_char_at(c, pixel as usize, bar_y as usize, SPLASH_BAR_FG, SPLASH_BG);
    }
}

/// Fade out the splash screen to black before transitioning to shell
pub fn fade_out_splash() {
    let (width, height) = super::get_dimensions();
    if width == 0 || height == 0 { return; }
    
    // 8-step fade: overlay increasingly opaque black rectangles
    for step in 0u32..8 {
        let alpha = (step + 1) * 32; // 32, 64, 96 ... 256
        let shade = if alpha >= 255 { 0xFF000000 } else {
            // Blend: darken existing pixels
            let inv = 255 - alpha;
            let g = (0x05 * inv) / 255;
            0xFF000000 | (g << 8)
        };
        super::fill_rect(0, 0, width, height, shade);
        
        // Small delay between frames (~20ms per step)
        for _ in 0..2_000_000 { core::hint::spin_loop(); }
    }
    
    // Final: full black
    super::fill_rect(0, 0, width, height, 0xFF000000);
    // Brief pause before shell appears
    for _ in 0..3_000_000 { core::hint::spin_loop(); }
}
