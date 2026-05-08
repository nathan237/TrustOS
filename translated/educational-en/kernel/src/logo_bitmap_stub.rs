//! TrustOS Logo Bitmap — Slim build stub (feature "hires-logo" disabled)
//!
//! Provides the same public API as logo_bitmap.rs but with no embedded pixel data.
//! All draw functions are no-ops; pixel/mask queries return transparent/false.

/// Logo width in pixels
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const LOGO_W: usize = 400;
/// Logo height in pixels
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const LOGO_H: usize = 400;

/// Check if pixel (x, y) is in the logo silhouette — always false in slim build
#[inline]
// Public function — callable from other modules.
pub fn logo_mask_pixel(_x: usize, _y: usize) -> bool {
    false
}

/// Check if pixel is on the edge of the logo — always false in slim build
#[inline]
// Public function — callable from other modules.
pub fn logo_edge_pixel(_x: usize, _y: usize) -> bool {
    false
}

/// Draw the full-color logo — no-op in slim build
pub fn draw_logo(_px: u32, _py: u32) {}

/// Draw the logo centered — no-op in slim build
pub fn draw_logo_centered(_cx: u32, _cy: u32) {}

/// Draw the logo with a green glow effect — no-op in slim build
pub fn draw_logo_glow(_px: u32, _py: u32, _glow_intensity: u8) {}

/// Access a single pixel as ARGB u32 — returns transparent in slim build
#[inline]
// Public function — callable from other modules.
pub fn logo_pixel(_x: usize, _y: usize) -> u32 {
    0x00000000
}

/// Access a single pixel by flat index — returns transparent in slim build
#[inline]
// Public function — callable from other modules.
pub fn logo_pixel_flat(_i: usize) -> u32 {
    0x00000000
}
