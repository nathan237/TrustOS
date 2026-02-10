//! UI Scaling Module for TrustOS
//!
//! Provides resolution-independent rendering by applying a global scale factor
//! to all UI dimensions. Supports 1x (native), 2x (HiDPI), and 3x scaling.
//!
//! # Architecture
//!
//! The scaling system works at the logical → physical level:
//! - All UI code works in **logical pixels** (design coordinates)
//! - The scaling module converts to **physical pixels** (framebuffer coordinates)
//! - Character rendering upscales 8×16 glyphs via nearest-neighbor
//!
//! # Usage
//!
//! ```rust
//! use crate::graphics::scaling;
//!
//! // Set scale factor (1 = native, 2 = double, 3 = triple)
//! scaling::set_scale_factor(2);
//!
//! // Scale a dimension
//! let height = scaling::scale(40); // → 80 at 2x
//!
//! // Get scaled character dimensions
//! let cw = scaling::char_width();  // → 16 at 2x
//! let ch = scaling::char_height(); // → 32 at 2x
//!
//! // Draw scaled text
//! scaling::draw_scaled_text(100, 50, "Hello", 0xFFFFFFFF);
//! ```

use core::sync::atomic::{AtomicU32, Ordering};

// ═══════════════════════════════════════════════════════════════════════════════
// GLOBAL SCALE STATE
// ═══════════════════════════════════════════════════════════════════════════════

/// Global UI scale factor (1 = native, 2 = HiDPI 2x, 3 = 3x)
static SCALE_FACTOR: AtomicU32 = AtomicU32::new(1);

/// Base character dimensions (unscaled 8×16 font)
const BASE_CHAR_WIDTH: u32 = 8;
const BASE_CHAR_HEIGHT: u32 = 16;

// ═══════════════════════════════════════════════════════════════════════════════
// SCALE FACTOR MANAGEMENT
// ═══════════════════════════════════════════════════════════════════════════════

/// Set the global UI scale factor.
///
/// Valid values: 1 (native), 2 (double/HiDPI), 3 (triple).
/// Invalid values are clamped to [1, 3].
pub fn set_scale_factor(factor: u32) {
    let clamped = factor.clamp(1, 3);
    SCALE_FACTOR.store(clamped, Ordering::SeqCst);
    crate::serial_println!("[Scaling] Scale factor set to {}x", clamped);
}

/// Get the current global scale factor.
#[inline]
pub fn get_scale_factor() -> u32 {
    SCALE_FACTOR.load(Ordering::Relaxed)
}

/// Auto-detect optimal scale factor based on framebuffer resolution.
///
/// Heuristics:
/// - Width >= 3840 (4K)  → 3x
/// - Width >= 2560 (QHD) → 2x
/// - Otherwise           → 1x
pub fn auto_detect_scale(fb_width: u32, fb_height: u32) -> u32 {
    let factor = if fb_width >= 3840 {
        3
    } else if fb_width >= 2560 {
        2
    } else {
        1
    };
    crate::serial_println!(
        "[Scaling] Auto-detected {}x scale for {}x{} framebuffer",
        factor, fb_width, fb_height
    );
    factor
}

/// Initialize the scaling system with auto-detection.
///
/// Call this after framebuffer is initialized, during desktop init.
pub fn init(fb_width: u32, fb_height: u32) {
    let factor = auto_detect_scale(fb_width, fb_height);
    set_scale_factor(factor);
}

// ═══════════════════════════════════════════════════════════════════════════════
// DIMENSION SCALING HELPERS
// ═══════════════════════════════════════════════════════════════════════════════

/// Scale a u32 dimension by the current global scale factor.
#[inline]
pub fn scale(value: u32) -> u32 {
    value * get_scale_factor()
}

/// Scale an i32 dimension by the current global scale factor.
#[inline]
pub fn scale_i32(value: i32) -> i32 {
    value * get_scale_factor() as i32
}

/// Unscale a physical pixel coordinate back to logical coordinate.
#[inline]
pub fn unscale(physical: u32) -> u32 {
    let f = get_scale_factor();
    if f == 0 { physical } else { physical / f }
}

/// Unscale a signed physical coordinate back to logical.
#[inline]
pub fn unscale_i32(physical: i32) -> i32 {
    let f = get_scale_factor() as i32;
    if f == 0 { physical } else { physical / f }
}

/// Get scaled character width.
#[inline]
pub fn char_width() -> u32 {
    BASE_CHAR_WIDTH * get_scale_factor()
}

/// Get scaled character height.
#[inline]
pub fn char_height() -> u32 {
    BASE_CHAR_HEIGHT * get_scale_factor()
}

// ═══════════════════════════════════════════════════════════════════════════════
// SCALED LAYOUT CONSTANTS
// ═══════════════════════════════════════════════════════════════════════════════

/// Provides scaled versions of all common UI layout constants.
///
/// Create with `ScaledLayout::current()` to snapshot the current scale factor.
#[derive(Clone, Copy, Debug)]
pub struct ScaledLayout {
    pub factor: u32,
    pub taskbar_height: u32,
    pub title_bar_height: u32,
    pub window_border_radius: u32,
    pub window_shadow_blur: u32,
    pub dock_icon_size: u32,
    pub dock_width: u32,
    pub char_width: u32,
    pub char_height: u32,
}

impl ScaledLayout {
    /// Base (unscaled) layout constants.
    const BASE_TASKBAR_HEIGHT: u32 = 40;
    const BASE_TITLE_BAR_HEIGHT: u32 = 28;
    const BASE_WINDOW_BORDER_RADIUS: u32 = 6;
    const BASE_WINDOW_SHADOW_BLUR: u32 = 12;
    const BASE_DOCK_ICON_SIZE: u32 = 24;
    const BASE_DOCK_WIDTH: u32 = 60;

    /// Create a ScaledLayout with the current global scale factor.
    pub fn current() -> Self {
        let f = get_scale_factor();
        ScaledLayout {
            factor: f,
            taskbar_height: Self::BASE_TASKBAR_HEIGHT * f,
            title_bar_height: Self::BASE_TITLE_BAR_HEIGHT * f,
            window_border_radius: Self::BASE_WINDOW_BORDER_RADIUS * f,
            window_shadow_blur: Self::BASE_WINDOW_SHADOW_BLUR * f,
            dock_icon_size: Self::BASE_DOCK_ICON_SIZE * f,
            dock_width: Self::BASE_DOCK_WIDTH * f,
            char_width: BASE_CHAR_WIDTH * f,
            char_height: BASE_CHAR_HEIGHT * f,
        }
    }

    /// Create a ScaledLayout for a specific factor.
    pub fn with_factor(f: u32) -> Self {
        let f = f.clamp(1, 3);
        ScaledLayout {
            factor: f,
            taskbar_height: Self::BASE_TASKBAR_HEIGHT * f,
            title_bar_height: Self::BASE_TITLE_BAR_HEIGHT * f,
            window_border_radius: Self::BASE_WINDOW_BORDER_RADIUS * f,
            window_shadow_blur: Self::BASE_WINDOW_SHADOW_BLUR * f,
            dock_icon_size: Self::BASE_DOCK_ICON_SIZE * f,
            dock_width: Self::BASE_DOCK_WIDTH * f,
            char_width: BASE_CHAR_WIDTH * f,
            char_height: BASE_CHAR_HEIGHT * f,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SCALED CHARACTER RENDERING
// ═══════════════════════════════════════════════════════════════════════════════

/// Draw a single character at (x, y) scaled by the current global factor.
///
/// Uses nearest-neighbor upscaling: each pixel of the 8×16 glyph is expanded
/// to a `factor × factor` block. No filtering — crisp pixel art look.
pub fn draw_scaled_char(x: u32, y: u32, c: char, color: u32) {
    let factor = get_scale_factor();

    // Fast path: 1x — delegate to normal framebuffer draw
    if factor == 1 {
        crate::framebuffer::draw_char_at(x, y, c, color);
        return;
    }

    let glyph = crate::framebuffer::font::get_glyph(c);
    let fb_width = crate::framebuffer::width();
    let fb_height = crate::framebuffer::height();

    // Bounds check: skip if entirely off-screen
    let total_w = BASE_CHAR_WIDTH * factor;
    let total_h = BASE_CHAR_HEIGHT * factor;
    if x >= fb_width || y >= fb_height {
        return;
    }

    // Clip extents
    let max_px = fb_width.min(x + total_w);
    let max_py = fb_height.min(y + total_h);

    for row in 0..BASE_CHAR_HEIGHT as usize {
        let bits = glyph[row];
        if bits == 0 {
            continue; // Skip blank glyph rows (common: top/bottom padding)
        }
        let base_py = y + (row as u32) * factor;
        if base_py >= max_py {
            break;
        }

        for col in 0..BASE_CHAR_WIDTH as usize {
            if (bits >> (7 - col)) & 1 == 1 {
                let base_px = x + (col as u32) * factor;
                if base_px >= max_px {
                    break;
                }

                // Fill a factor×factor block
                for sy in 0..factor {
                    let py = base_py + sy;
                    if py >= max_py {
                        break;
                    }
                    for sx in 0..factor {
                        let px = base_px + sx;
                        if px < max_px {
                            crate::framebuffer::put_pixel(px, py, color);
                        }
                    }
                }
            }
        }
    }
}

/// Draw a text string at (x, y) with scaling applied.
///
/// Characters are spaced by `char_width()` (8 × factor) pixels apart.
pub fn draw_scaled_text(x: i32, y: i32, text: &str, color: u32) {
    let cw = char_width() as i32;
    let fb_w = crate::framebuffer::width() as i32;
    let fb_h = crate::framebuffer::height() as i32;

    if y < 0 || y >= fb_h {
        return;
    }

    for (i, c) in text.chars().enumerate() {
        let px = x + (i as i32) * cw;
        if px >= fb_w {
            break; // Past right edge
        }
        if px + cw <= 0 {
            continue; // Left of screen
        }
        if px >= 0 {
            draw_scaled_char(px as u32, y as u32, c, color);
        }
    }
}

/// Draw a text string at (x, y) with a specific scale factor (not the global one).
///
/// Useful when a specific widget needs a different scale than the global default.
pub fn draw_text_at_scale(x: i32, y: i32, text: &str, color: u32, factor: u32) {
    let factor = factor.clamp(1, 3);
    let cw = (BASE_CHAR_WIDTH * factor) as i32;
    let fb_w = crate::framebuffer::width() as i32;
    let fb_h = crate::framebuffer::height() as i32;

    if y < 0 || y >= fb_h {
        return;
    }

    for (i, c) in text.chars().enumerate() {
        let px = x + (i as i32) * cw;
        if px >= fb_w {
            break;
        }
        if px + cw <= 0 {
            continue;
        }
        if px >= 0 {
            draw_char_at_scale(px as u32, y as u32, c, color, factor);
        }
    }
}

/// Draw a single character at a specific scale factor.
fn draw_char_at_scale(x: u32, y: u32, c: char, color: u32, factor: u32) {
    if factor == 1 {
        crate::framebuffer::draw_char_at(x, y, c, color);
        return;
    }

    let glyph = crate::framebuffer::font::get_glyph(c);
    let fb_width = crate::framebuffer::width();
    let fb_height = crate::framebuffer::height();

    let max_px = fb_width.min(x + BASE_CHAR_WIDTH * factor);
    let max_py = fb_height.min(y + BASE_CHAR_HEIGHT * factor);

    for row in 0..BASE_CHAR_HEIGHT as usize {
        let bits = glyph[row];
        if bits == 0 {
            continue;
        }
        let base_py = y + (row as u32) * factor;
        if base_py >= max_py {
            break;
        }

        for col in 0..BASE_CHAR_WIDTH as usize {
            if (bits >> (7 - col)) & 1 == 1 {
                let base_px = x + (col as u32) * factor;
                if base_px >= max_px {
                    break;
                }
                for sy in 0..factor {
                    let py = base_py + sy;
                    if py >= max_py {
                        break;
                    }
                    for sx in 0..factor {
                        let px = base_px + sx;
                        if px < max_px {
                            crate::framebuffer::put_pixel(px, py, color);
                        }
                    }
                }
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SCALED RECT PRIMITIVES
// ═══════════════════════════════════════════════════════════════════════════════

/// Draw a filled rectangle with scaled dimensions.
///
/// Position (x, y) is in physical pixels; width/height are logical and get scaled.
pub fn fill_scaled_rect(x: i32, y: i32, logical_w: u32, logical_h: u32, color: u32) {
    let f = get_scale_factor();
    let pw = logical_w * f;
    let ph = logical_h * f;

    if x >= 0 && y >= 0 {
        crate::framebuffer::fill_rect(x as u32, y as u32, pw, ph, color);
    }
}

/// Scale a rectangle: converts logical (x, y, w, h) to physical coordinates.
#[inline]
pub fn scale_rect(x: i32, y: i32, w: u32, h: u32) -> (i32, i32, u32, u32) {
    let f = get_scale_factor();
    (
        x * f as i32,
        y * f as i32,
        w * f,
        h * f,
    )
}

// ═══════════════════════════════════════════════════════════════════════════════
// SCALED CURSOR SUPPORT
// ═══════════════════════════════════════════════════════════════════════════════

/// Draw a scaled cursor pattern.
///
/// Takes a bitmap pattern (2D array) and draws it scaled by the current factor.
/// Pixel values: 0 = transparent, 1 = outline, 2 = fill.
pub fn draw_scaled_cursor(
    cursor_x: i32,
    cursor_y: i32,
    pattern: &[[u8; 12]],
    outline_color: u32,
    fill_color: u32,
) {
    let factor = get_scale_factor();
    let fb_w = crate::framebuffer::width();
    let fb_h = crate::framebuffer::height();

    for (cy, row) in pattern.iter().enumerate() {
        for (cx, &pixel) in row.iter().enumerate() {
            if pixel == 0 {
                continue;
            }
            let color = match pixel {
                1 => outline_color,
                2 => fill_color,
                _ => continue,
            };

            // Scale the cursor pixel to a factor×factor block
            for sy in 0..factor {
                for sx in 0..factor {
                    let px = cursor_x + (cx as u32 * factor + sx) as i32;
                    let py = cursor_y + (cy as u32 * factor + sy) as i32;
                    if px >= 0 && py >= 0 && (px as u32) < fb_w && (py as u32) < fb_h {
                        crate::framebuffer::put_pixel(px as u32, py as u32, color);
                    }
                }
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TEXT MEASUREMENT
// ═══════════════════════════════════════════════════════════════════════════════

/// Measure the width of a text string in physical pixels at the current scale.
#[inline]
pub fn measure_text_width(text: &str) -> u32 {
    text.len() as u32 * char_width()
}

/// Measure the height of a text line in physical pixels at the current scale.
#[inline]
pub fn measure_text_height() -> u32 {
    char_height()
}

/// Measure text width at a specific scale factor.
#[inline]
pub fn measure_text_width_at(text: &str, factor: u32) -> u32 {
    text.len() as u32 * BASE_CHAR_WIDTH * factor.clamp(1, 3)
}
