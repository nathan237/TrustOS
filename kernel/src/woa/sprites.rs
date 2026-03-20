//! WOA Sprites — Pixel art sprite data (ARGB u32 const arrays)
//!
//! Sprites are 128×128 ARGB at 640×400 internal resolution.
//! Transparent pixels = 0x00000000.
//! Convention: sprites face RIGHT.
//!
//! To regenerate from PNG:
//!   python tools/sprite_converter.py media/woa/militant.png --size 128 --name MILITANT_IDLE -o media/woa/militant_128.rs

use super::militant_sprite_data;

/// Sprite dimensions
pub const SPRITE_W: u32 = 128;
pub const SPRITE_H: u32 = 128;
pub const SPRITE_PIXELS: usize = (SPRITE_W * SPRITE_H) as usize; // 16384

/// Militant Idle sprite — 128×128 ARGB, facing right (converted from PixelLab PNG)
pub use militant_sprite_data::MILITANT_IDLE;

/// Flipped version (facing LEFT) — generated at runtime to avoid doubling const data
pub fn flip_sprite_h(src: &[u32], w: u32, h: u32, dst: &mut [u32]) {
    for row in 0..h {
        for col in 0..w {
            let src_idx = (row * w + col) as usize;
            let dst_idx = (row * w + (w - 1 - col)) as usize;
            if src_idx < src.len() && dst_idx < dst.len() {
                dst[dst_idx] = src[src_idx];
            }
        }
    }
}

/// Flash white effect — replace all non-transparent pixels with white
pub fn flash_sprite(src: &[u32], dst: &mut [u32]) {
    for (i, &px) in src.iter().enumerate() {
        if i < dst.len() {
            dst[i] = if px & 0xFF000000 != 0 { 0xFFFFFFFF } else { 0x00000000 };
        }
    }
}

/// Tint sprite with a color (multiply blend) — used for ghost/halo effect
pub fn tint_sprite(src: &[u32], dst: &mut [u32], tint: u32) {
    let tr = ((tint >> 16) & 0xFF) as u32;
    let tg = ((tint >> 8) & 0xFF) as u32;
    let tb = (tint & 0xFF) as u32;
    for (i, &px) in src.iter().enumerate() {
        if i < dst.len() {
            if px & 0xFF000000 != 0 {
                let r = (((px >> 16) & 0xFF) * tr) / 255;
                let g = (((px >> 8) & 0xFF) * tg) / 255;
                let b = ((px & 0xFF) * tb) / 255;
                dst[i] = 0x80000000 | (r << 16) | (g << 8) | b; // 50% alpha marker
            } else {
                dst[i] = 0x00000000;
            }
        }
    }
}
