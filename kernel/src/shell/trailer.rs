//! TrustOS Trailer -- 2-minute cinematic trailer with 1984/Big Brother theme
//!
//! Structure: LOGO -> OPPRESSION (0:00-0:37) -> RUPTURE (0:37-0:42) -> LIBERATION (0:42-2:00)
//! Visual: Dystopian red/dark -> silence -> green/warm ascendant
//! Music: Dark Synthwave (Perturbator / Carpenter Brut style)

use alloc::vec;
use alloc::vec::Vec;

const FRAME_MS: u64 = 33; // ~30 fps

#[inline]
fn xorshift(mut x: u32) -> u32 {
    if x == 0 { x = 1; }
    x ^= x << 13;
    x ^= x >> 17;
    x ^= x << 5;
    x
}

// ===============================================================================
// FLEUR-DE-LIS LOGO BITMAP -- Faithful reproduction of the TrustOS emblem
// 48 pixels wide x 72 pixels tall, 1 bit per pixel, stored as u64 per row
// The logo is a fleur-de-lis with matrix rain fill and chrome edges
// ===============================================================================

/// Each u64 encodes one row of the 48-wide logo. Bits 47..0 = left..right.
/// The shape: top lance, two side petals with scrolls, center band, bottom spike.
const LOGO_H: usize = 72;
const LOGO_W: usize = 48;
const LOGO_BITMAP: [u64; LOGO_H] = [
    // Top of central lance (rows 0-7): narrow spear tip
    0b_000000000000000000000001_100000000000000000000000, // 0
    0b_000000000000000000000001_100000000000000000000000, // 1
    0b_000000000000000000000011_110000000000000000000000, // 2
    0b_000000000000000000000011_110000000000000000000000, // 3
    0b_000000000000000000000111_111000000000000000000000, // 4
    0b_000000000000000000000111_111000000000000000000000, // 5
    0b_000000000000000000001111_111100000000000000000000, // 6
    0b_000000000000000000001111_111100000000000000000000, // 7
    // Lance blade widens (rows 8-15)
    0b_000000000000000000011111_111110000000000000000000, // 8
    0b_000000000000000000011111_111110000000000000000000, // 9
    0b_000000000000000000111111_111111000000000000000000, //10
    0b_000000000000000000111111_111111000000000000000000, //11
    0b_000000000000000001111111_111111100000000000000000, //12
    0b_000000000000000001111111_111111100000000000000000, //13
    0b_000000000000000011111111_111111110000000000000000, //14
    0b_000000000000000011111111_111111110000000000000000, //15
    // Lance at widest + side petals begin (rows 16-23)
    0b_000000000000000111111111_111111111000000000000000, //16
    0b_000000000000001111111111_111111111100000000000000, //17
    0b_000000000000011111111111_111111111110000000000000, //18
    0b_000000000000111111111111_111111111111000000000000, //19
    0b_000000000001111111111111_111111111111100000000000, //20
    0b_000000000011111111111111_111111111111110000000000, //21
    0b_000000000111111111111111_111111111111111000000000, //22
    0b_000000001111111111111111_111111111111111100000000, //23
    // Side petals flare out (rows 24-31)
    0b_000000011111111111111111_111111111111111110000000, //24
    0b_000000111111111111111111_111111111111111111000000, //25
    0b_000001111111111111111111_111111111111111111100000, //26
    0b_000011111111111111111111_111111111111111111110000, //27
    0b_000111111111111111111111_111111111111111111111000, //28
    0b_001111111111111111111111_111111111111111111111100, //29
    0b_011111111111111111111111_111111111111111111111110, //30
    0b_111111111111111111111111_111111111111111111111111, //31  full width
    // Side petals + center narrows, petals curve (rows 32-39)
    0b_111111111111110000111111_111110000111111111111111, //32
    0b_111111111111100000011111_111100000011111111111111, //33
    0b_111111111111000000001111_111000000001111111111111, //34
    0b_111111111110000000001111_111000000000111111111111, //35
    0b_011111111110000000000111_111000000000011111111110, //36
    0b_011111111100000000000111_111000000000001111111110, //37
    0b_001111111100000000000111_111000000000001111111100, //38
    0b_001111111100000000000011_110000000000001111111100, //39
    // Petals scroll inward (rows 40-47)
    0b_000111111100000000000011_110000000000001111111000, //40
    0b_000011111110000000000011_110000000000011111110000, //41
    0b_000001111111000000000011_110000000000111111100000, //42
    0b_000000111111100000000011_110000000001111111000000, //43
    0b_000000011111110000000011_110000000011111110000000, //44
    0b_000000001111111000000011_110000000111111100000000, //45
    0b_000000000111111100000011_110000001111111000000000, //46
    0b_000000000011111111000011_110000111111110000000000, //47
    // Scrolls curl + center band (rows 48-55)
    0b_000000000001111111100111_111001111111100000000000, //48
    0b_000000000000111111111111_111111111111000000000000, //49
    0b_000000000000011111111111_111111111110000000000000, //50
    0b_000000000000001111111111_111111111100000000000000, //51
    0b_000000000000000111111111_111111111000000000000000, //52
    0b_000000000000000111111111_111111111000000000000000, //53
    0b_000000000000001111111111_111111111100000000000000, //54
    0b_000000000000011111111111_111111111110000000000000, //55
    // Bottom spike starts (rows 56-63)
    0b_000000000000001111111111_111111111100000000000000, //56
    0b_000000000000000111111111_111111111000000000000000, //57
    0b_000000000000000011111111_111111110000000000000000, //58
    0b_000000000000000001111111_111111100000000000000000, //59
    0b_000000000000000000111111_111111000000000000000000, //60
    0b_000000000000000000011111_111110000000000000000000, //61
    0b_000000000000000000001111_111100000000000000000000, //62
    0b_000000000000000000000111_111000000000000000000000, //63
    // Bottom spike tip (rows 64-71)
    0b_000000000000000000000111_111000000000000000000000, //64
    0b_000000000000000000000011_110000000000000000000000, //65
    0b_000000000000000000000011_110000000000000000000000, //66
    0b_000000000000000000000001_100000000000000000000000, //67
    0b_000000000000000000000001_100000000000000000000000, //68
    0b_000000000000000000000001_100000000000000000000000, //69
    0b_000000000000000000000000_100000000000000000000000, //70
    0b_000000000000000000000000_000000000000000000000000, //71
];

/// Check if pixel (x, y) in the 48x72 logo is filled
#[inline]
fn logo_pixel(x: usize, y: usize) -> bool {
    if x >= LOGO_W || y >= LOGO_H { return false; }
    (LOGO_BITMAP[y] >> (LOGO_W - 1 - x)) & 1 == 1
}

/// Check if pixel is on the edge of the logo shape (has a neighbor that differs)
#[inline]
fn logo_edge(x: usize, y: usize) -> bool {
    if !logo_pixel(x, y) { return false; }
    // Check 4-connected neighbors
    if x == 0 || !logo_pixel(x - 1, y) { return true; }
    if x >= LOGO_W - 1 || !logo_pixel(x + 1, y) { return true; }
    if y == 0 || !logo_pixel(x, y - 1) { return true; }
    if y >= LOGO_H - 1 || !logo_pixel(x, y + 1) { return true; }
    false
}

/// Render the fleur-de-lis logo at (cx, cy) center with given scale.
/// Interior is filled with animated matrix binary rain.
/// Edges are chrome/silver with a subtle gradient.
fn render_logo(buf: &mut [u32], bw: usize, bh: usize,
               cx: usize, cy: usize, scale: usize, frame: u32) {
    let logo_pw = LOGO_W * scale;
    let logo_ph = LOGO_H * scale;
    let ox = cx.saturating_sub(logo_pw / 2);
    let oy = cy.saturating_sub(logo_ph / 2);

    // Precompute LCG-style rain pattern for interior fill
    let rain_offset = frame as usize;

    for sy in 0..logo_ph {
        let ly = sy / scale; // logo y
        let py = oy + sy;
        if py >= bh { continue; }

        for sx in 0..logo_pw {
            let lx = sx / scale; // logo x
            let px = ox + sx;
            if px >= bw { continue; }

            if !logo_pixel(lx, ly) { continue; }

            let idx = py * bw + px;

            if logo_edge(lx, ly) {
                // Chrome/silver edge with vertical gradient
                let grad = (ly as u32 * 80 / LOGO_H as u32) + 20;
                let base = 140u32 + grad;
                // Slight horizontal highlight at center
                let dx = if lx > LOGO_W / 2 { lx - LOGO_W / 2 } else { LOGO_W / 2 - lx };
                let highlight = 30u32.saturating_sub(dx as u32 * 2);
                let v = (base + highlight).min(240);
                buf[idx] = 0xFF000000 | (v << 16) | (v << 8) | v;
            } else {
                // Matrix rain interior: binary digits scrolling down
                // Each "column" in the logo gets its own rain speed
                let col_seed = (lx.wrapping_mul(7919) + 31) % 97;
                let speed = 1 + col_seed % 3;
                let rain_y = (ly + rain_offset * speed + col_seed * 5) % 17;
                // Binary character from hash
                let char_hash = (lx.wrapping_mul(2654435761_usize.wrapping_shr(0))
                    .wrapping_add(ly.wrapping_mul(40503))
                    .wrapping_add(rain_offset * speed)) % 37;

                // Intensity: brighter near the "head" of the rain drop
                let head_dist = rain_y;
                let intensity = if head_dist < 2 {
                    200u32  // bright green head
                } else if head_dist < 6 {
                    (120u32).saturating_sub(head_dist as u32 * 12)
                } else {
                    25u32 + (char_hash as u32 % 20)  // dim ambient
                };

                // If character cell would be "on" -- show green
                let cell_x = sx % (scale * 4);  // sub-character column within a glyph
                let cell_y = sy % (scale * 6);
                let is_char = char_hash < 20 && cell_x > 0 && cell_x < scale * 3
                    && cell_y > 0 && cell_y < scale * 5;

                if is_char {
                    let g = intensity.min(255);
                    let r = g / 8;
                    let b = g / 5;
                    buf[idx] = 0xFF000000 | (r << 16) | (g << 8) | b;
                } else {
                    // Dark green/black background inside logo
                    let g = 8u32 + (char_hash as u32 % 8);
                    buf[idx] = 0xFF000000 | (g << 8);
                }
            }
        }
    }
}

// ===============================================================================
// HELPER FUNCTIONS
// ===============================================================================

fn draw_big_char(buf: &mut [u32], w: usize, h: usize,
                 cx: usize, cy: usize, c: char, color: u32, scale: usize) {
    let glyph = crate::framebuffer::font::get_glyph(c);
    for (row, &bits) in glyph.iter().enumerate() {
        for bit in 0..8u32 {
            if bits & (0x80 >> bit) != 0 {
                for sy in 0..scale {
                    for sx in 0..scale {
                        let px = cx + bit as usize * scale + sx;
                        let py = cy + row * scale + sy;
                        if px < w && py < h { buf[py * w + px] = color; }
                    }
                }
            }
        }
    }
}

/// Draw a character with a soft glow halo (makes text look less pixelated)
fn draw_big_char_glow(buf: &mut [u32], w: usize, h: usize,
                      cx: usize, cy: usize, c: char, color: u32, scale: usize, glow_r: u32) {
    let glyph = crate::framebuffer::font::get_glyph(c);
    let cr = (color >> 16) & 0xFF;
    let cg = (color >> 8) & 0xFF;
    let cb = color & 0xFF;
    // First pass: soft glow (every other pixel for speed)
    if glow_r > 0 {
        let spread = glow_r as usize;
        for (row, &bits) in glyph.iter().enumerate() {
            for bit in 0..8u32 {
                if bits & (0x80 >> bit) != 0 {
                    // Only sample center of each glyph cell for glow source
                    let gcx = cx + bit as usize * scale + scale / 2;
                    let gcy = cy + row * scale + scale / 2;
                    // Emit glow in a small radius
                    let step = if spread > 3 { 2 } else { 1 };
                    let mut dy = -(spread as i32);
                    while dy <= spread as i32 {
                        let mut dx = -(spread as i32);
                        while dx <= spread as i32 {
                            let d2 = (dx * dx + dy * dy) as u32;
                            let r2 = glow_r * glow_r;
                            if d2 > 0 && d2 < r2 {
                                let px = (gcx as i32 + dx) as usize;
                                let py = (gcy as i32 + dy) as usize;
                                if px < w && py < h {
                                    let falloff = 255u32.saturating_sub(d2 * 255 / r2) / 4;
                                    let idx = py * w + px;
                                    let dst = buf[idx];
                                    let dr = (dst >> 16) & 0xFF;
                                    let dg = (dst >> 8) & 0xFF;
                                    let db = dst & 0xFF;
                                    let nr = (dr + cr * falloff / 255).min(255);
                                    let ng = (dg + cg * falloff / 255).min(255);
                                    let nb = (db + cb * falloff / 255).min(255);
                                    buf[idx] = 0xFF000000 | (nr << 16) | (ng << 8) | nb;
                                }
                            }
                            dx += step;
                        }
                        dy += step;
                    }
                }
            }
        }
    }
    // Second pass: solid character on top
    for (row, &bits) in glyph.iter().enumerate() {
        for bit in 0..8u32 {
            if bits & (0x80 >> bit) != 0 {
                for sy in 0..scale {
                    for sx in 0..scale {
                        let px = cx + bit as usize * scale + sx;
                        let py = cy + row * scale + sy;
                        if px < w && py < h { buf[py * w + px] = color; }
                    }
                }
            }
        }
    }
}

/// Draw text with shadow + glow for cinematic look
fn draw_text_glow(buf: &mut [u32], w: usize, h: usize,
                  y: usize, text: &str, color: u32, scale: usize) {
    let tw = text.len() * 8 * scale;
    let sx = if tw < w { (w - tw) / 2 } else { 0 };
    // Drop shadow (offset +2,+2)
    let shadow = scale.max(1);
    for (i, c) in text.chars().enumerate() {
        draw_big_char(buf, w, h, sx + i * 8 * scale + shadow, y + shadow, c, 0xFF000000, scale);
    }
    // Glow + main
    let glow = (scale as u32 * 3).min(12);
    for (i, c) in text.chars().enumerate() {
        draw_big_char_glow(buf, w, h, sx + i * 8 * scale, y, c, color, scale, glow);
    }
}

/// Apply a vignette darkening effect (darkens edges, leaves center bright)
fn apply_vignette(buf: &mut [u32], w: usize, h: usize, strength: u32) {
    let cx = w / 2;
    let cy = h / 2;
    let max_r2 = (cx * cx + cy * cy) as u32;
    // Process every 2nd pixel for speed, interpolate visually
    for y in (0..h).step_by(2) {
        let dy = if y > cy { y - cy } else { cy - y };
        for x in (0..w).step_by(2) {
            let dx = if x > cx { x - cx } else { cx - x };
            let d2 = (dx * dx + dy * dy) as u32;
            let factor = d2 * strength / max_r2;
            let dim = factor.min(200) as u32;
            // Apply to 2x2 block
            for by in 0..2u32 {
                for bx in 0..2u32 {
                    let px = x + bx as usize;
                    let py = y + by as usize;
                    if px < w && py < h {
                        let idx = py * w + px;
                        let c = buf[idx];
                        let r = ((c >> 16) & 0xFF).saturating_sub(dim);
                        let g = ((c >> 8) & 0xFF).saturating_sub(dim);
                        let b = (c & 0xFF).saturating_sub(dim);
                        buf[idx] = 0xFF000000 | (r << 16) | (g << 8) | b;
                    }
                }
            }
        }
    }
}

/// Draw a filled circle with radial gradient (center_color -> edge_color)
fn draw_radial_glow(buf: &mut [u32], w: usize, h: usize,
                    cx: usize, cy: usize, radius: usize,
                    center_r: u32, center_g: u32, center_b: u32, alpha: u32) {
    let r2 = (radius * radius) as u32;
    let y_start = cy.saturating_sub(radius);
    let y_end = (cy + radius).min(h);
    let x_start = cx.saturating_sub(radius);
    let x_end = (cx + radius).min(w);
    for y in y_start..y_end {
        let dy = if y > cy { y - cy } else { cy - y };
        for x in x_start..x_end {
            let dx = if x > cx { x - cx } else { cx - x };
            let d2 = (dx * dx + dy * dy) as u32;
            if d2 < r2 {
                let falloff = alpha * (r2 - d2) / r2;
                let ir = center_r * falloff / 255;
                let ig = center_g * falloff / 255;
                let ib = center_b * falloff / 255;
                let idx = y * w + x;
                let dst = buf[idx];
                let dr = (dst >> 16) & 0xFF;
                let dg = (dst >> 8) & 0xFF;
                let db = dst & 0xFF;
                buf[idx] = 0xFF000000
                    | ((dr + ir).min(255) << 16)
                    | ((dg + ig).min(255) << 8)
                    | (db + ib).min(255);
            }
        }
    }
}

/// Draw a filled ellipse with solid color
fn draw_filled_ellipse(buf: &mut [u32], w: usize, h: usize,
                       cx: usize, cy: usize, rx: usize, ry: usize, color: u32) {
    let y_start = cy.saturating_sub(ry);
    let y_end = (cy + ry).min(h);
    let x_start = cx.saturating_sub(rx);
    let x_end = (cx + rx).min(w);
    let rx2 = (rx * rx) as u64;
    let ry2 = (ry * ry) as u64;
    for y in y_start..y_end {
        let dy = if y > cy { y - cy } else { cy - y };
        for x in x_start..x_end {
            let dx = if x > cx { x - cx } else { cx - x };
            if (dx as u64 * dx as u64) * ry2 + (dy as u64 * dy as u64) * rx2 < rx2 * ry2 {
                buf[y * w + x] = color;
            }
        }
    }
}

/// CRT scanline + curvature overlay (applied after scene rendering)
fn apply_crt_overlay(buf: &mut [u32], w: usize, h: usize) {
    let cx = w / 2;
    let cy = h / 2;
    for y in 0..h {
        // Scanline dimming: every 3rd line
        let scan_dim = if y % 3 == 0 { 40u32 } else { 0 };
        // Barrel distortion darkening at edges
        let dy = if y > cy { y - cy } else { cy - y };
        let edge_v = (dy * dy * 60 / (cy * cy).max(1)) as u32;
        for x in 0..w {
            let dx = if x > cx { x - cx } else { cx - x };
            let edge_h = (dx * dx * 60 / (cx * cx).max(1)) as u32;
            let total_dim = scan_dim + edge_v + edge_h;
            if total_dim > 0 {
                let idx = y * w + x;
                let c = buf[idx];
                let r = ((c >> 16) & 0xFF).saturating_sub(total_dim);
                let g = ((c >> 8) & 0xFF).saturating_sub(total_dim);
                let b = (c & 0xFF).saturating_sub(total_dim);
                buf[idx] = 0xFF000000 | (r << 16) | (g << 8) | b;
            }
        }
    }
}

fn draw_text_at(buf: &mut [u32], w: usize, h: usize,
                x: usize, y: usize, text: &str, color: u32, scale: usize) {
    for (i, c) in text.chars().enumerate() {
        draw_big_char(buf, w, h, x + i * 8 * scale, y, c, color, scale);
    }
}

fn draw_text_centered(buf: &mut [u32], w: usize, h: usize,
                      y: usize, text: &str, color: u32, scale: usize) {
    let tw = text.len() * 8 * scale;
    let sx = if tw < w { (w - tw) / 2 } else { 0 };
    draw_text_at(buf, w, h, sx, y, text, color, scale);
}

fn fill_rect(buf: &mut [u32], w: usize, h: usize,
             rx: usize, ry: usize, rw: usize, rh: usize, color: u32) {
    for dy in 0..rh {
        for dx in 0..rw {
            let px = rx + dx;
            let py = ry + dy;
            if px < w && py < h { buf[py * w + px] = color; }
        }
    }
}

fn clear_buf(buf: &mut [u32]) {
    // SSE2-accelerated fill (4 pixels per store)
    #[cfg(target_arch = "x86_64")]
    unsafe {
        use core::arch::x86_64::*;
        let black = _mm_set1_epi32(0xFF000000u32 as i32);
        let ptr = buf.as_mut_ptr() as *mut __m128i;
        let count = buf.len() / 4;
        for i in 0..count {
            _mm_storeu_si128(ptr.add(i), black);
        }
        for i in (count * 4)..buf.len() {
            buf[i] = 0xFF000000;
        }
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        for p in buf.iter_mut() { *p = 0xFF000000; }
    }
}

fn blit_buf(buf: &[u32], w: usize, h: usize) {
    // Use SMP parallel blit: copies directly to MMIO FB across all cores
    crate::framebuffer::blit_to_fb_parallel(buf.as_ptr(), w, h);
}

fn do_fade(buf: &mut [u32], w: usize, h: usize) {
    for _ in 0..40 {
        for px in buf.iter_mut() {
            let r = ((*px >> 16) & 0xFF).saturating_sub(8);
            let g = ((*px >> 8) & 0xFF).saturating_sub(8);
            let b = (*px & 0xFF).saturating_sub(8);
            *px = 0xFF000000 | (r << 16) | (g << 8) | b;
        }
        blit_buf(buf, w, h);
        crate::cpu::tsc::pit_delay_ms(FRAME_MS);
    }
    clear_buf(buf);
    blit_buf(buf, w, h);
    crate::cpu::tsc::pit_delay_ms(300);
}

/// Quick glitch transition -- 3 frames of random noise then black
fn glitch_cut(buf: &mut [u32], w: usize, h: usize) {
    let mut seed = 0xDEADBEEFu32;
    for _ in 0..3 {
        for px in buf.iter_mut() {
            seed ^= seed << 13; seed ^= seed >> 17; seed ^= seed << 5;
            let v = seed & 0xFF;
            *px = 0xFF000000 | (v << 16) | (v << 8) | v;
        }
        blit_buf(buf, w, h);
        crate::cpu::tsc::pit_delay_ms(40);
    }
    clear_buf(buf);
    blit_buf(buf, w, h);
    crate::cpu::tsc::pit_delay_ms(80);
}

/// Check for escape / space / enter to advance or skip
fn can_advance() -> bool {
    if let Some(k) = crate::keyboard::try_read_key() {
        return k == 0x1B || k == b' ' || k == b'\n' || k == b'\r';
    }
    false
}

// ===============================================================================
// BACKGROUND GENERATORS
// ===============================================================================

/// CRT static / test pattern -- Fallout "Please Stand By" background
fn bg_crt_static(buf: &mut [u32], w: usize, h: usize, frame: u32) {
    let mut seed = frame.wrapping_mul(2654435761);
    // Dark static noise base
    for px in buf.iter_mut() {
        seed ^= seed << 13; seed ^= seed >> 17; seed ^= seed << 5;
        let noise = (seed & 0x1F) as u32; // 0-31 range = dark static
        *px = 0xFF000000 | (noise << 16) | (noise << 8) | noise;
    }
    // CRT scanlines overlay
    for y in 0..h {
        if y % 3 == 0 {
            for x in 0..w {
                let idx = y * w + x;
                let r = ((buf[idx] >> 16) & 0xFF) / 2;
                let g = ((buf[idx] >> 8) & 0xFF) / 2;
                let b = (buf[idx] & 0xFF) / 2;
                buf[idx] = 0xFF000000 | (r << 16) | (g << 8) | b;
            }
        }
    }
}

/// Red warning scanlines (oppression scenes)
fn bg_scanlines(buf: &mut [u32], w: usize, h: usize, frame: u32) {
    let scroll = (frame as usize * 2) % h;
    for y in 0..h {
        let sy = (y + scroll) % h;
        let stripe = (sy / 4) % 2 == 0;
        for x in 0..w {
            let base_r = if stripe { 35u32 } else { 15 };
            let flash = if (sy % 60) < 2 { 30u32 } else { 0 };
            let r = (base_r + flash).min(65);
            buf[y * w + x] = 0xFF000000 | (r << 16) | 0x0205;
        }
    }
}

/// Pulse nebula (dark blue/purple)
fn bg_pulse(buf: &mut [u32], w: usize, h: usize, frame: u32) {
    let phase = (frame % 160) as u32;
    let pulse = if phase < 80 { phase / 2 } else { (160 - phase) / 2 };
    let phase2 = ((frame + 40) % 120) as u32;
    let pulse2 = if phase2 < 60 { phase2 / 2 } else { (120 - phase2) / 2 };
    for y in 0..h {
        let yf = (y as u32 * 40) / h as u32;
        for x in 0..w {
            let xf = (x as u32 * 10) / w as u32;
            let r = (yf / 4 + pulse2 / 3).min(40);
            let g = (xf / 3).min(15);
            let b = (yf + pulse + xf / 2).min(80);
            buf[y * w + x] = 0xFF000000 | (r << 16) | (g << 8) | b;
        }
    }
}

/// Rising green sparks
fn bg_sparks(buf: &mut [u32], w: usize, h: usize, frame: u32) {
    for px in buf.iter_mut() {
        let g = ((*px >> 8) & 0xFF).saturating_sub(10);
        let r = ((*px >> 16) & 0xFF).saturating_sub(8);
        *px = 0xFF000000 | (r << 16) | (g << 8);
    }
    for i in 0..30u32 {
        let seed = (i.wrapping_mul(2654435761).wrapping_add(frame.wrapping_mul(37))) as usize;
        let px = (seed.wrapping_mul(7919)) % w;
        let rise = (frame as usize + seed) % h;
        let py = h.saturating_sub(rise);
        let bright = (50 + (seed % 50)) as u32;
        if px < w && py < h {
            buf[py * w + px] = 0xFF000000 | (bright / 4 << 16) | (bright << 8) | (bright / 3);
            if px + 1 < w { buf[py * w + px + 1] = 0xFF000000 | (bright << 8); }
        }
    }
}

/// Sunrise warm gradient
fn bg_sunrise(buf: &mut [u32], w: usize, h: usize, frame: u32) {
    let lift = frame.min(80);
    for y in 0..h {
        let yf = y as u32 * 100 / h as u32;
        let warmth = if yf > 50 { (yf - 50).min(50) + lift } else { lift / 2 };
        let r = (warmth * 2).min(100);
        let g = (warmth * 3 / 4).min(50);
        let b = 20u32.saturating_sub(warmth / 3);
        for x in 0..w {
            buf[y * w + x] = 0xFF000000 | (r << 16) | (g << 8) | b;
        }
    }
}

/// Circuit-board traces
fn bg_circuit(buf: &mut [u32], w: usize, h: usize, frame: u32) {
    for p in buf.iter_mut() { *p = 0xFF0A0A14; }
    let trace = 0xFF0F2818u32;
    for i in 0..16u32 {
        let ty = ((i.wrapping_mul(7919) as usize) % h) & !3;
        let tx = ((i.wrapping_mul(104729) as usize) % w) & !3;
        if ty < h { for x in 0..w { buf[ty * w + x] = trace; } }
        if tx < w { for y in 0..h { buf[y * w + tx] = trace; } }
    }
    let py = ((frame as usize * 3) % h) & !3;
    if py < h {
        let pw = (w / 4).min(120);
        let px_start = (frame as usize * 5) % w;
        for dx in 0..pw {
            let px = (px_start + dx) % w;
            buf[py * w + px] = 0xFF00AA44;
            if py + 1 < h { buf[(py + 1) * w + px] = 0xFF00AA44; }
        }
    }
}

/// Matrix rain background
fn bg_rain(buf: &mut [u32], w: usize, h: usize,
           cols: &mut [u16], speeds: &[u8], frame: u32) {
    // Fade existing
    for pixel in buf.iter_mut() {
        let g = ((*pixel >> 8) & 0xFF) as u32;
        if g > 0 { *pixel = 0xFF000000 | (g.saturating_sub(6) << 8); }
        else { *pixel = 0xFF000000; }
    }
    for ci in 0..cols.len() {
        let x = ci * 8;
        if x >= w { continue; }
        cols[ci] = cols[ci].wrapping_add(speeds[ci] as u16);
        if cols[ci] as usize >= h { cols[ci] = 0; }
        let y = cols[ci] as usize;
        let c = (((frame as usize + ci * 13) % 94) + 33) as u8 as char;
        let glyph = crate::framebuffer::font::get_glyph(c);
        for (row, &bits) in glyph.iter().enumerate() {
            let py = y + row;
            if py >= h { break; }
            for bit in 0..8u32 {
                if bits & (0x80 >> bit) != 0 {
                    let px = x + bit as usize;
                    if px < w { buf[py * w + px] = 0xFF00FF44; }
                }
            }
        }
    }
}

/// Binary cascade (for "black box" scene)
fn bg_binary_flood(buf: &mut [u32], w: usize, h: usize, frame: u32) {
    // Fade down
    for px in buf.iter_mut() {
        let r = ((*px >> 16) & 0xFF).saturating_sub(15);
        let g = ((*px >> 8) & 0xFF).saturating_sub(15);
        let b = (*px & 0xFF).saturating_sub(15);
        *px = 0xFF000000 | (r << 16) | (g << 8) | b;
    }
    // Drop binary digits
    for col in 0..(w / 10) {
        let x = col * 10 + 2;
        let seed = (col.wrapping_mul(7919).wrapping_add(frame as usize * 3)) % 97;
        let speed = 2 + seed % 4;
        let y = ((frame as usize * speed + col * 23) % (h + 40)).wrapping_sub(20);
        if y < h && x < w {
            let digit = if (col + frame as usize) % 2 == 0 { '0' } else { '1' };
            let glyph = crate::framebuffer::font::get_glyph(digit);
            let bright = 100u32 + (seed as u32 * 3) % 80;
            for (row, &bits) in glyph.iter().enumerate() {
                let py = y + row;
                if py >= h { break; }
                for bit in 0..8u32 {
                    if bits & (0x80 >> bit) != 0 {
                        let px = x + bit as usize;
                        if px < w {
                            buf[py * w + px] = 0xFF000000 | (bright << 8) | (bright / 4);
                        }
                    }
                }
            }
        }
    }
}

// ===============================================================================
// BEAT CLOCK -- 128 BPM tempo system for cinematic rhythm
// ===============================================================================
//
// 128 BPM = 1 beat every 468.75ms = ~14 frames at 30fps
// All scene durations are defined in beats for perfect musical sync.
// The trailer uses an acceleration curve: scenes get shorter toward the end.
//
// ======================== SUNO.AI MUSIC PROMPT ================================
//
// ** Style: Dark Synthwave / Cinematic Retro Hybrid **
// ** BPM: 128 ** | ** Key: D minor ** | ** Duration: 1:20 **
//
// PROMPT:
// -------
// [Verse 1 - 0:00 to 0:30]
// Dark synthwave, deep analog bass drone, minor key, ominous pads,
// slow building tension, distant arpeggios, dystopian atmosphere,
// 808 sub bass, filtered white noise textures, VHS tape warble,
// cinematic suspense, Blade Runner vibes, no drums yet
//
// [Drop Build - 0:28 to 0:33]
// Rising synth riser, tension crescendo, white noise sweep,
// reverse cymbal, heartbeat kick pattern 4-on-floor emerging,
// building anticipation, everything pauses at 0:33
//
// [Silence - 0:33 to 0:35]
// Total silence, single reverb tail fading
//
// [Drop - 0:35]
// MASSIVE synth drop, distorted saw lead, Carpenter Brut style,
// driving 128 BPM four-on-floor kick, aggressive sidechained bass,
// triumphant euphoric minor-to-major lift, epic analog brass stabs,
// cyberpunk action, Hans Zimmer meets Perturbator, powerful and heroic
//
// [Accelerando - 0:50 to 1:10]
// Tempo feels faster with double-time hi-hats, rapid synth stabs
// every 2 beats, breakbeat fills, glitch edits, intensifying,
// stacking layers, choir pad swells, relentless momentum
//
// [Climax + Silence - 1:10 to 1:15]
// Everything cuts to silence on beat, single low piano note rings,
// 3 seconds of reverb decay
//
// [Stinger - 1:15 to 1:20]
// Final massive chord, analog brass + strings, triumphant resolve,
// fade to analog tape hiss
//
// TAGS: dark synthwave, cinematic, 128 bpm, dystopian, epic drop,
//       Carpenter Brut, Perturbator, retro, analog, cyberpunk,
//       dramatic, trailer music, tension and release, D minor
//
// NEGATIVE TAGS: vocals, singing, lyrics, acoustic guitar, happy,
//                lo-fi, jazz, country, folk
//
// ===============================================================================

/// Frames per beat at 128 BPM (~30fps): 468.75ms / 33ms = 14.2 -> 14
const FRAMES_PER_BEAT: u32 = 14;

/// Convert beat count to frame count
#[inline]
fn beats(n: u32) -> u32 { n * FRAMES_PER_BEAT }

// ===============================================================================
// TRANSITION EFFECTS -- Pro-grade cinematic cuts
// ===============================================================================

/// SMASH CUT: instant cut to black, hard stop. Used for shock/drama.
fn smash_cut(buf: &mut [u32], w: usize, h: usize) {
    clear_buf(buf);
    blit_buf(buf, w, h);
    crate::cpu::tsc::pit_delay_ms(100);
}

/// FLASH FRAME: 2-3 frames of blinding white then black.
/// Simulates a camera flash / "BRAAAM" hit. Maximum impact.
fn flash_frame(buf: &mut [u32], w: usize, h: usize) {
    // Frame 1: Pure white
    for p in buf.iter_mut() { *p = 0xFFFFFFFF; }
    blit_buf(buf, w, h);
    crate::cpu::tsc::pit_delay_ms(33);
    // Frame 2: 70% white
    for p in buf.iter_mut() { *p = 0xFFB0B0B0; }
    blit_buf(buf, w, h);
    crate::cpu::tsc::pit_delay_ms(33);
    // Frame 3: Black
    clear_buf(buf);
    blit_buf(buf, w, h);
    crate::cpu::tsc::pit_delay_ms(66);
}

/// IMPACT SHAKE: screen shakes violently for N frames.
/// Simulates bass drop / explosion. Pairs with flash_frame.
fn impact_shake(buf: &mut [u32], w: usize, h: usize, intensity: usize) {
    let mut seed = 0xCAFEBABEu32;
    for i in 0..6u32 {
        let decay = intensity.saturating_sub(i as usize);
        if decay == 0 { break; }
        seed = xorshift(seed);
        let ox = (seed as usize % (decay * 2 + 1)).wrapping_sub(decay);
        seed = xorshift(seed);
        let oy = (seed as usize % (decay * 2 + 1)).wrapping_sub(decay);
        // Shift buffer
        let mut shifted = vec![0xFF000000u32; w * h];
        for y in 0..h {
            let sy = (y as isize + oy as isize).max(0) as usize;
            if sy >= h { continue; }
            for x in 0..w {
                let sx = (x as isize + ox as isize).max(0) as usize;
                if sx >= w { continue; }
                shifted[sy * w + sx] = buf[y * w + x];
            }
        }
        blit_buf(&shifted, w, h);
        crate::cpu::tsc::pit_delay_ms(33);
    }
}

/// WHIP PAN: horizontal wipe at extreme speed (4 frames).
/// Simulates camera whipping sideways between scenes.
fn whip_pan(buf: &mut [u32], w: usize, h: usize) {
    for step in 0..4u32 {
        let offset = (step + 1) as usize * w / 4;
        let mut shifted = vec![0xFF000000u32; w * h];
        for y in 0..h {
            for x in offset..w {
                shifted[y * w + x - offset] = buf[y * w + x];
            }
        }
        // Motion blur: smear the edge
        for y in 0..h {
            let edge_x = w.saturating_sub(offset);
            for x in edge_x..w {
                shifted[y * w + x] = 0xFF080808;
            }
        }
        blit_buf(&shifted, w, h);
        crate::cpu::tsc::pit_delay_ms(25);
    }
    clear_buf(buf);
    blit_buf(buf, w, h);
    crate::cpu::tsc::pit_delay_ms(50);
}

/// SILENCE: pure black for N beats. The most powerful tool in editing.
fn silence(buf: &mut [u32], w: usize, h: usize, beat_count: u32) {
    clear_buf(buf);
    for _ in 0..beats(beat_count) {
        if can_advance() { return; }
        blit_buf(buf, w, h);
        crate::cpu::tsc::pit_delay_ms(FRAME_MS);
    }
}

// ===============================================================================
// TYPEWRITER TEXT -- beat-synced typing with cursor
// ===============================================================================

/// Type multiple centered lines over a dynamic background.
/// Duration is capped to `max_beats` for rhythm control.
fn type_scene<F>(buf: &mut [u32], w: usize, h: usize,
                 lines: &[(&str, u32, usize)],
                 ms_per_char: u64, max_beats: u32,
                 mut bg_fn: F)
where F: FnMut(&mut [u32], usize, usize, u32) {
    let total_chars: usize = lines.iter().map(|(t, _, _)| t.len()).sum();
    let fpc = (ms_per_char / FRAME_MS).max(1) as u32;
    let typing_frames = total_chars as u32 * fpc;
    let hold = beats(max_beats).saturating_sub(typing_frames);
    let total = typing_frames + hold;

    for frame in 0..total {
        if can_advance() { return; }

        bg_fn(buf, w, h, frame);

        let chars_shown = (frame / fpc) as usize;
        let total_h: usize = lines.iter().map(|(_, _, s)| 16 * s + 12).sum();
        let mut y = if total_h < h { (h - total_h) / 2 } else { 20 };
        let mut counted = 0usize;

        for &(text, color, scale) in lines {
            let tw = text.len() * 8 * scale;
            let sx = if tw < w { (w - tw) / 2 } else { 0 };
            for (i, c) in text.chars().enumerate() {
                if counted + i >= chars_shown { break; }
                draw_big_char(buf, w, h, sx + i * 8 * scale, y, c, color, scale);
            }
            // Blinking cursor
            if chars_shown > counted && chars_shown < counted + text.len() {
                let ci = chars_shown - counted;
                let cx = sx + ci * 8 * scale;
                if (frame / 8) % 2 == 0 {
                    for cy in y..y + 16 * scale {
                        if cy < h && cx + 2 < w {
                            buf[cy * w + cx] = 0xFFFFFFFF;
                            buf[cy * w + cx + 1] = 0xFFFFFFFF;
                        }
                    }
                }
            }
            counted += text.len();
            y += 16 * scale + 12;
        }

        blit_buf(buf, w, h);
        crate::cpu::tsc::pit_delay_ms(FRAME_MS);
    }
}

/// Show text instantly for N beats with animated background.
fn hold_scene<F>(buf: &mut [u32], w: usize, h: usize,
                 lines: &[(&str, u32, usize)], beat_count: u32,
                 mut bg_fn: F)
where F: FnMut(&mut [u32], usize, usize, u32) {
    for frame in 0..beats(beat_count) {
        if can_advance() { return; }
        bg_fn(buf, w, h, frame);

        let total_h: usize = lines.iter().map(|(_, _, s)| 16 * s + 12).sum();
        let mut y = if total_h < h { (h - total_h) / 2 } else { 20 };
        for &(text, color, scale) in lines {
            draw_text_centered(buf, w, h, y, text, color, scale);
            y += 16 * scale + 12;
        }

        blit_buf(buf, w, h);
        crate::cpu::tsc::pit_delay_ms(FRAME_MS);
    }
}

// ===============================================================================
// MAIN TRAILER ENTRY POINT -- Beat-synced cinematic sequence
// ===============================================================================
//
// TIMELINE (128 BPM, 1 beat = 468ms):
//
// Beat   Time   Scene                    Beats  Transition
// ----   -----  -----------------------  -----  ----------
//   0    0:00   Logo Reveal              16     flash_frame
//  16    0:07   Please Stand By           8     glitch_cut
//  24    0:11   Big Brother Eye          12     smash_cut
//  36    0:17   Data Collection          10     whip_pan
//  46    0:21   Black Box                 8     smash_cut
//  54    0:25   Redacted Documents        8     flash_frame
//  62    0:29   === SILENCE 4 beats ===   4     ---
//  66    0:31   === THE DROP ===          0     flash+shake
//  66    0:31   TrustOS Reveal           12     flash_frame
//  78    0:36   The Numbers               8     flash_frame
//  86    0:40   -- ACCELERANDO --
//  86    0:40   Network Stack             4     glitch_cut
//  90    0:42   TLS 1.3                   3     flash_frame
//  93    0:43   GUI Compositor            3     glitch_cut
//  96    0:45   TrustLang                 3     flash_frame
//  99    0:46   TrustFS                   2     glitch_cut
// 101    0:47   Browser                   2     flash_frame
// 103    0:48   3D Engine                 4     glitch_cut
// 107    0:50   Video Codec               4     flash_frame
// 111    0:52   1984 Reversed             6     smash_cut
// 117    0:55   Size Comparison           6     impact_shake
// 123    0:58   Manifesto                 4     ---
// 127    0:59   === SILENCE 6 beats ===   6     ---
// 133    1:02   "TRUST THE CODE"          8     flash+shake
// 141    1:06   Stinger CRT               4     fade
// 145    1:08   END
//
// Total: 145 beats = ~68 seconds (~1:08)
//
// ===============================================================================

pub(super) fn cmd_trustos_trailer() {
    let (sw, sh) = crate::framebuffer::get_dimensions();
    let w = sw as usize;
    let h = sh as usize;

    // Setup double buffering
    let was_db = crate::framebuffer::is_double_buffer_enabled();
    if !was_db {
        crate::framebuffer::init_double_buffer();
        crate::framebuffer::set_double_buffer_mode(true);
    }

    let mut buf = vec![0xFF000000u32; w * h];

    // Rain state for matrix backgrounds
    let ncols = w / 8 + 1;
    let mut rain_cols: Vec<u16> = (0..ncols).map(|i| ((i * 37 + 13) % h) as u16).collect();
    let rain_speeds: Vec<u8> = (0..ncols).map(|i| (((i * 7 + 3) % 4) + 1) as u8).collect();

    crate::serial_println!("[TRAILER] TrustOS Trailer started (128 BPM beat-synced)");

    // =======================================================================
    // ACT 1 -- OPPRESSION (0:00 - 0:29) -- Slow, ominous, building dread
    // =======================================================================

    // -------------------------------------------------------------------
    // SCENE 0 -- LOGO REVEAL (16 beats = 7.5s)
    // Black -> radial glow -> logo emerges from darkness with matrix rain fill
    // -------------------------------------------------------------------
    {
        let scale = if h > 600 { 6 } else { 4 };
        let logo_cy = h / 2 - 30;
        let text_y = logo_cy + (LOGO_H * scale) / 2 + 20;
        let total = beats(16);

        for frame in 0..total {
            if can_advance() { break; }
            clear_buf(&mut buf);

            // Subtle radial green glow behind logo (builds up)
            let glow_alpha = (frame * 3).min(120);
            let glow_radius = 80 + (frame as usize * 2).min(h / 3);
            draw_radial_glow(&mut buf, w, h, w / 2, logo_cy, glow_radius,
                             20, 80, 40, glow_alpha);

            render_logo(&mut buf, w, h, w / 2, logo_cy, scale, frame);

            // Pulsing outer halo ring around logo
            if frame > 20 {
                let pulse = ((frame % 40) as i32 - 20).unsigned_abs() as u32;
                let ring_r = (LOGO_W * scale / 2 + 10 + pulse as usize) as f32;
                let ring_bright = 60u32.saturating_sub(pulse);
                for a in 0..180 {
                    let angle = a as f32 * 0.0349; // 2*pi/180
                    let sin_a = crate::formula3d::fast_sin(angle);
                    let cos_a = crate::formula3d::fast_cos(angle);
                    for thickness in 0..2 {
                        let r = ring_r + thickness as f32;
                        let px = (w as f32 / 2.0 + cos_a * r) as usize;
                        let py = (logo_cy as f32 + sin_a * r * 0.75) as usize;
                        if px < w && py < h {
                            let idx = py * w + px;
                            let g = ((buf[idx] >> 8) & 0xFF) + ring_bright;
                            buf[idx] = 0xFF000000 | ((ring_bright / 4) << 16) | (g.min(255) << 8) | (ring_bright / 3);
                        }
                    }
                }
            }

            // Slow fade-in over first 60 frames
            if frame < 60 {
                let dim = ((60 - frame) as u32 * 255 / 60) as u32;
                for px in buf.iter_mut() {
                    if *px != 0xFF000000 {
                        let r = ((*px >> 16) & 0xFF).saturating_sub(dim);
                        let g = ((*px >> 8) & 0xFF).saturating_sub(dim);
                        let b = (*px & 0xFF).saturating_sub(dim);
                        *px = 0xFF000000 | (r << 16) | (g << 8) | b;
                    }
                }
            }

            // Vignette darkening on edges
            apply_vignette(&mut buf, w, h, 180);

            // "TRUSTOS" text appears letter by letter after beat 8
            if frame > beats(8) {
                let text = "TRUSTOS";
                let text_scale = if h > 600 { 5 } else { 3 };
                let sub = frame - beats(8);
                let chars_shown = (sub / 8).min(text.len() as u32) as usize;
                let tw = text.len() * 8 * text_scale;
                let tx = if tw < w { (w - tw) / 2 } else { 0 };
                // Shadow first
                for (i, c) in text.chars().enumerate() {
                    if i >= chars_shown { break; }
                    draw_big_char(&mut buf, w, h,
                        tx + i * 8 * text_scale + 2, text_y + 2, c, 0xFF000000, text_scale);
                }
                // Glowing silver letters
                for (i, c) in text.chars().enumerate() {
                    if i >= chars_shown { break; }
                    draw_big_char_glow(&mut buf, w, h,
                        tx + i * 8 * text_scale, text_y, c, 0xFFDDDDDD, text_scale,
                        text_scale as u32 * 3);
                }
            }

            blit_buf(&buf, w, h);
            crate::cpu::tsc::pit_delay_ms(FRAME_MS);
        }
    }
    flash_frame(&mut buf, w, h); // BEAT 16: HIT

    // -------------------------------------------------------------------
    // SCENE 1 -- PLEASE STAND BY (8 beats = 3.7s)
    // CRT test pattern + Fallout-style warning + barrel distortion
    // -------------------------------------------------------------------
    {
        let total = beats(8);
        for frame in 0..total {
            if can_advance() { break; }
            bg_crt_static(&mut buf, w, h, frame);

            // Round-cornered box (panel)
            let bw_r = if w > 800 { 200 } else { 160 };
            let bh_r = 50;
            let bx = w / 2 - bw_r;
            let by = h / 2 - bh_r;
            // Panel shadow
            fill_rect(&mut buf, w, h, bx + 3, by + 3, bw_r * 2, bh_r * 2, 0xFF050505);
            // Panel body
            fill_rect(&mut buf, w, h, bx, by, bw_r * 2, bh_r * 2, 0xFF111111);
            // Panel border with gradient
            for x in bx..bx + bw_r * 2 {
                if x < w {
                    let grad = 0xFF555555 + ((x - bx) as u32 * 0x40 / (bw_r as u32 * 2));
                    let gc = 0xFF000000 | (grad & 0xFF) << 16 | (grad & 0xFF) << 8 | (grad & 0xFF);
                    buf[by * w + x] = gc;
                    buf[(by + bh_r * 2 - 1) * w + x] = gc;
                }
            }
            for y in by..by + bh_r * 2 {
                if y < h {
                    buf[y * w + bx] = 0xFF888888;
                    buf[y * w + (bx + bw_r * 2 - 1).min(w - 1)] = 0xFF888888;
                }
            }

            let warn_color = if (frame / 15) % 2 == 0 { 0xFFFFCC00 } else { 0xFFFF8800 };
            draw_text_glow(&mut buf, w, h, by + 12, "! WARNING !", warn_color, 2);
            draw_text_glow(&mut buf, w, h, by + 55, "PLEASE STAND BY", 0xFFCCCCCC, 2);

            // Apply CRT curvature + scanlines
            apply_crt_overlay(&mut buf, w, h);
            apply_vignette(&mut buf, w, h, 200);

            blit_buf(&buf, w, h);
            crate::cpu::tsc::pit_delay_ms(FRAME_MS);
        }
    }
    glitch_cut(&mut buf, w, h); // BEAT 24

    // -------------------------------------------------------------------
    // SCENE 2 -- BIG BROTHER (12 beats = 5.6s)
    // Giant eye with layered iris + glow + "BIG BROTHER IS WATCHING YOU"
    // -------------------------------------------------------------------
    {
        let eye_cx = w / 2;
        let eye_cy = h / 2 - 40;
        let eye_w = w / 4;
        let eye_h = h / 8;
        let total = beats(12);

        for frame in 0..total {
            if can_advance() { break; }
            bg_pulse(&mut buf, w, h, frame);

            // Menacing red radial glow behind eye (builds up)
            let glow_int = (frame as u32 * 3).min(180);
            draw_radial_glow(&mut buf, w, h, eye_cx, eye_cy, eye_w + 40, glow_int, 0x10, 0x00, glow_int);

            // Sclera fill (white ellipse)
            draw_filled_ellipse(&mut buf, w, h, eye_cx, eye_cy, eye_w, eye_h, 0xFFDDCCCC);

            // Iris ring (red/orange gradient ring)
            let iris_r = eye_h * 2 / 3;
            let pupil_r = eye_h / 3;
            for dy in 0..iris_r*2+2 {
                for dx in 0..iris_r*2+2 {
                    let ddx = dx as i32 - iris_r as i32;
                    let ddy = dy as i32 - iris_r as i32;
                    let d2 = ddx*ddx + ddy*ddy;
                    let ir2 = (iris_r as i32) * (iris_r as i32);
                    let pr2 = (pupil_r as i32) * (pupil_r as i32);
                    if d2 <= ir2 && d2 >= pr2 {
                        let px = (eye_cx as i32 + ddx) as usize;
                        let py = (eye_cy as i32 + ddy) as usize;
                        if px < w && py < h {
                            // Gradient: outer = dark red, inner = bright red
                            let t = (d2 - pr2) as u32 * 255 / (ir2 - pr2).max(1) as u32;
                            let r = 0xFF;
                            let g = (0x66u32).saturating_sub(t * 0x66 / 255) as u8;
                            buf[py * w + px] = 0xFF000000 | (r as u32) << 16 | (g as u32) << 8;
                        }
                    }
                }
            }

            // Pupil (deep black)
            for dy in 0..pupil_r*2 {
                for dx in 0..pupil_r*2 {
                    let ddx = dx as i32 - pupil_r as i32;
                    let ddy = dy as i32 - pupil_r as i32;
                    if ddx*ddx + ddy*ddy < (pupil_r as i32 * pupil_r as i32) {
                        let px = (eye_cx as i32 + ddx) as usize;
                        let py = (eye_cy as i32 + ddy) as usize;
                        if px < w && py < h { buf[py * w + px] = 0xFF080000; }
                    }
                }
            }

            // Bright eye outline (thicker, anti-aliased look with 2 rings)
            for thick in 0..2i32 {
                let ew = eye_w as i32 + thick;
                let eh = eye_h as i32 + thick;
                for angle in 0..720 {
                    let a = angle as f32 * 0.008727; // ~0.5 degree steps
                    let sin_a = crate::formula3d::fast_sin(a);
                    let cos_a = crate::formula3d::fast_cos(a);
                    let px = (eye_cx as f32 + cos_a * ew as f32) as usize;
                    let py = (eye_cy as f32 + sin_a * eh as f32) as usize;
                    if px < w && py < h {
                        buf[py * w + px] = 0xFFFF3333;
                    }
                }
            }

            // Text typewriter with glow
            let chars = (frame / 3).min(28) as usize;
            let full_text = "BIG BROTHER IS WATCHING YOU.";
            let shown: alloc::string::String = full_text.chars().take(chars).collect();
            draw_text_glow(&mut buf, w, h, eye_cy + eye_h + 50, &shown, 0xFFFF4444, 3);

            apply_vignette(&mut buf, w, h, 160);

            blit_buf(&buf, w, h);
            crate::cpu::tsc::pit_delay_ms(FRAME_MS);
        }
    }
    smash_cut(&mut buf, w, h); // BEAT 36: hard cut to black

    // -------------------------------------------------------------------
    // SCENE 3 -- DATA COLLECTION (10 beats = 4.7s)
    // Surveillance data slides in from right + red glow + pulsing bars
    // -------------------------------------------------------------------
    {
        let lines_data = [
            "location: TRACKED",
            "camera: ACTIVE",
            "keystrokes: LOGGED",
            "microphone: RECORDING",
            "contacts: UPLOADED",
            "messages: SCANNED",
            "browsing: PROFILED",
            "identity: SOLD",
        ];
        let total = beats(10);
        for frame in 0..total {
            if can_advance() { break; }
            bg_scanlines(&mut buf, w, h, frame);

            // Pulsing red warning bars (top & bottom)
            let pulse = (crate::formula3d::fast_sin(frame as f32 * 0.15) * 40.0 + 60.0) as u8;
            let bar_color = 0xFF000000 | (pulse as u32) << 16;
            fill_rect(&mut buf, w, h, 0, 0, w, 6, bar_color);
            fill_rect(&mut buf, w, h, 0, h.saturating_sub(6), w, 6, bar_color);

            let lines_shown = (frame / 12).min(8) as usize;
            for (i, &line) in lines_data.iter().enumerate().take(lines_shown) {
                let y = 80 + i * 50;
                let slide_in = ((frame as usize).saturating_sub(i * 12)).min(w);
                let x = w.saturating_sub(slide_in);
                // Shadow behind text
                draw_text_at(&mut buf, w, h, x + 2, y + 2, line, 0xFF220000, 2);
                // Red text with brighter color for "value" part
                let colon_pos = line.find(':').unwrap_or(line.len());
                let label = &line[..colon_pos];
                let value = &line[colon_pos..];
                let label_w = label.len() * 16;
                draw_text_at(&mut buf, w, h, x, y, label, 0xFFAA3333, 2);
                draw_text_at(&mut buf, w, h, x + label_w, y, value, 0xFFFF4444, 2);
            }

            if frame > beats(6) {
                draw_text_glow(&mut buf, w, h, h - 60,
                    "Every keystroke. Every click.", 0xFFFF6666, 2);
            }
            if frame > beats(8) {
                draw_text_glow(&mut buf, w, h, h - 30,
                    "Every file. Every thought.", 0xFFFF4444, 2);
            }

            apply_vignette(&mut buf, w, h, 180);

            blit_buf(&buf, w, h);
            crate::cpu::tsc::pit_delay_ms(FRAME_MS);
        }
    }
    whip_pan(&mut buf, w, h); // BEAT 46: whip pan transition

    // -------------------------------------------------------------------
    // SCENE 4 -- THE BLACK BOX (8 beats = 3.7s)
    // "50M lines of code / you can read ZERO"
    // -------------------------------------------------------------------
    type_scene(&mut buf, w, h,
        &[("Your OS has", 0xFFAAFFAA, 2),
          ("50,000,000 lines of code.", 0xFF00FF88, 3),
          ("", 0, 1),
          ("You can read ZERO of them.", 0xFFFF4444, 3)],
        40, 8, |buf, w, h, f| bg_binary_flood(buf, w, h, f));
    smash_cut(&mut buf, w, h); // BEAT 54

    // -------------------------------------------------------------------
    // SCENE 5 -- REDACTED (8 beats = 3.7s)
    // Documents censored with black bars — warm paper background + glow stamps
    // -------------------------------------------------------------------
    {
        let doc_lines = [
            "Kernel source code ............",
            "Driver implementations ........",
            "Encryption keys ...............",
            "Telemetry endpoints ...........",
            "Backdoor protocols ............",
            "Data collection routines ......",
        ];
        let total = beats(8);
        for frame in 0..total {
            if can_advance() { break; }
            // Warm paper-like background with subtle grid
            for y in 0..h { for x in 0..w {
                let base_r: u32 = 0x0C + (y as u32 * 4 / h as u32);
                let base_g: u32 = 0x14 + (y as u32 * 6 / h as u32);
                let base_b: u32 = 0x28 + (y as u32 * 8 / h as u32);
                let grid = if (x % 20 < 1) || (y % 20 < 1) { 0x06u32 } else { 0u32 };
                let r = (base_r + grid).min(255);
                let g = (base_g + grid).min(255);
                let b = (base_b + grid).min(255);
                buf[y * w + x] = 0xFF000000 | (r << 16) | (g << 8) | b;
            }}

            let base_y = h / 2 - doc_lines.len() * 22;
            for (i, &line) in doc_lines.iter().enumerate() {
                let y = base_y + i * 44;
                let tw = line.len() * 16;
                let tx = if tw < w { (w - tw) / 2 } else { 0 };
                // Shadow
                draw_text_at(&mut buf, w, h, tx + 1, y + 1, line, 0xFF223344, 2);
                draw_text_at(&mut buf, w, h, tx, y, line, 0xFF7799BB, 2);

                let redact_frame = 10 + i as u32 * 12;
                if frame > redact_frame {
                    let bar_progress = ((frame - redact_frame) as usize * 30).min(tw);
                    // Dark redaction bar with subtle border
                    fill_rect(&mut buf, w, h, tx, y, bar_progress, 30, 0xFF0A0A0A);
                    if bar_progress > 2 {
                        // Top highlight line
                        fill_rect(&mut buf, w, h, tx, y, bar_progress, 1, 0xFF222222);
                    }
                    if bar_progress >= tw {
                        // Red [REDACTED] stamp with glow
                        let stamp_x = tx + tw / 2 - 64;
                        let stamp_y = y + 4;
                        // Glow behind stamp
                        for gy in stamp_y.saturating_sub(4)..stamp_y + 30 {
                            for gx in stamp_x.saturating_sub(8)..stamp_x + 140 {
                                if gx < w && gy < h {
                                    let old = buf[gy * w + gx];
                                    let or = ((old >> 16) & 0xFF) as u32;
                                    let nr = (or + 30).min(255);
                                    buf[gy * w + gx] = (old & 0xFF00FFFF) | (nr << 16);
                                }
                            }
                        }
                        draw_text_at(&mut buf, w, h, stamp_x, stamp_y,
                            "[REDACTED]", 0xFFFF2222, 2);
                    }
                }
            }

            if frame > beats(6) {
                draw_text_glow(&mut buf, w, h, h - 40,
                    "You trust what you cannot see.", 0xFF8888CC, 2);
            }

            apply_vignette(&mut buf, w, h, 200);

            blit_buf(&buf, w, h);
            crate::cpu::tsc::pit_delay_ms(FRAME_MS);
        }
    }
    flash_frame(&mut buf, w, h); // BEAT 62: HIT

    // =======================================================================
    // THE BREAK -- 4 beats of TOTAL SILENCE (1.9s)
    // This is the most powerful moment. Pure black. Nothing.
    // The audience holds their breath. Then...
    // =======================================================================
    silence(&mut buf, w, h, 4); // BEAT 62-66: silence

    // =======================================================================
    // ACT 2 -- THE DROP (0:31) -- Flash + shake = "BRAAAM"
    // =======================================================================
    flash_frame(&mut buf, w, h);
    impact_shake(&mut buf, w, h, 12);

    // -------------------------------------------------------------------
    // SCENE 7 -- TRUSTOS REVEAL (12 beats = 5.6s)
    // Sparks + light burst + giant title
    // -------------------------------------------------------------------
    {
        let total = beats(12);
        for frame in 0..total {
            if can_advance() { break; }
            bg_sparks(&mut buf, w, h, frame);

            // Radiating light burst
            if frame > 5 && frame < beats(4) {
                let intensity = if frame < beats(2) { frame - 5 } else { beats(4) - frame };
                let cx = w / 2;
                let cy = h / 2 - 20;
                for ray in 0..16 {
                    let angle = ray as f32 * 0.3927;
                    let sin_a = crate::formula3d::fast_sin(angle);
                    let cos_a = crate::formula3d::fast_cos(angle);
                    let len = intensity as f32 * 8.0;
                    for t in 0..len as usize {
                        let px = (cx as f32 + cos_a * t as f32) as usize;
                        let py = (cy as f32 + sin_a * t as f32) as usize;
                        if px < w && py < h {
                            let bright = (200 - t * 3).max(40) as u32;
                            buf[py * w + px] = 0xFF000000 | (bright / 4 << 16) | (bright << 8) | (bright / 2);
                        }
                    }
                }
            }

            // Giant TRUSTOS title
            if frame > 8 {
                let title_scale = if h > 600 { 7 } else { 5 };
                let alpha = ((frame - 8) * 12).min(255) as u32;
                let color = 0xFF000000 | (alpha / 3 << 16) | (alpha << 8) | (alpha / 2);
                draw_text_centered(&mut buf, w, h, h / 2 - 40, "TRUSTOS", color, title_scale);

                if frame > beats(3) {
                    draw_text_centered(&mut buf, w, h, h / 2 + 40,
                        "The OS you can read. All of it.", 0xFF88DDAA, 2);
                }
            }

            blit_buf(&buf, w, h);
            crate::cpu::tsc::pit_delay_ms(FRAME_MS);
        }
    }
    flash_frame(&mut buf, w, h); // BEAT 78: HIT

    // -------------------------------------------------------------------
    // SCENE 8 -- THE NUMBERS (8 beats = 3.7s)
    // Odometer counters rolling up fast
    // -------------------------------------------------------------------
    {
        let counters: [(u32, &str); 4] = [
            (131662, " lines of Rust"),
            (1, " author"),
            (0, " secrets"),
            (100, "% open source"),
        ];
        let total = beats(8);
        for frame in 0..total {
            if can_advance() { break; }
            bg_circuit(&mut buf, w, h, frame);

            let items_shown = (frame / beats(2)).min(4) as usize;
            let base_y = h / 2 - items_shown * 35;

            for (i, &(target, label)) in counters.iter().enumerate().take(items_shown) {
                let y = base_y + i * 70;
                let sub_frame = frame.saturating_sub(i as u32 * beats(2));
                let progress = (sub_frame * 6).min(beats(2));
                let current = if target == 0 { 0 }
                    else { (target as u64 * progress as u64 / beats(2) as u64) as u32 };

                let num_str = alloc::format!("{:>7}", current);
                let full_str = alloc::format!("{}{}", num_str, label);

                let scale = 3;
                let tw = full_str.len() * 8 * scale;
                let tx = if tw < w { (w - tw) / 2 } else { 0 };

                for (ci, c) in num_str.chars().enumerate() {
                    draw_big_char(&mut buf, w, h, tx + ci * 8 * scale, y, c, 0xFF00FF88, scale);
                }
                for (ci, c) in label.chars().enumerate() {
                    draw_big_char(&mut buf, w, h, tx + (num_str.len() + ci) * 8 * scale, y, c, 0xFF44AA66, scale);
                }
            }

            blit_buf(&buf, w, h);
            crate::cpu::tsc::pit_delay_ms(FRAME_MS);
        }
    }
    flash_frame(&mut buf, w, h); // BEAT 86: HIT -- accelerando starts

    // =======================================================================
    // ACT 3 -- ACCELERANDO (0:40 - 0:52) -- Cuts get faster and faster
    // Each slide shorter than the last: 4 -> 3 -> 3 -> 3 -> 2 -> 2 -> 4 -> 4
    // =======================================================================

    // Slide 9.1: NETWORK STACK (4 beats)
    {
        let stages = ["hello", "a7 f3 0b 2e c1", "[HDR|DATA|CRC]", ">>> WIRE >>>"];
        let stage_colors = [0xFF00FF88u32, 0xFF00CCFF, 0xFFFFAA00, 0xFF44FF44];
        let total = beats(4);

        for frame in 0..total {
            if can_advance() { break; }
            bg_circuit(&mut buf, w, h, frame);

            let current_stage = (frame * 4 / total).min(3) as usize;
            draw_text_centered(&mut buf, w, h, 30, "NETWORK STACK", 0xFF00CCFF, 3);

            let center_y = h / 2 - 80;
            for i in 0..=current_stage {
                let y = center_y + i * 50;
                let bx = w / 2 - 140;
                fill_rect(&mut buf, w, h, bx, y, 280, 35, 0xFF111122);
                draw_text_centered(&mut buf, w, h, y + 4, stages[i], stage_colors[i], 2);
            }

            blit_buf(&buf, w, h);
            crate::cpu::tsc::pit_delay_ms(FRAME_MS);
        }
    }
    glitch_cut(&mut buf, w, h); // BEAT 90

    // Slide 9.2: TLS 1.3 (3 beats)
    hold_scene(&mut buf, w, h,
        &[("TLS 1.3", 0xFF00CCFF, 5),
          ("Full handshake. Real crypto.", 0xFF88BBDD, 2)],
        3, |buf, w, h, f| bg_pulse(buf, w, h, f));
    flash_frame(&mut buf, w, h); // BEAT 93

    // Slide 9.3: GUI (3 beats)
    hold_scene(&mut buf, w, h,
        &[("GUI COMPOSITOR", 0xFFFFCC00, 4),
          ("Windows. Taskbar. Wallpaper.", 0xFFCCAA88, 2)],
        3, |buf, w, h, f| bg_pulse(buf, w, h, f));
    glitch_cut(&mut buf, w, h); // BEAT 96

    // Slide 9.4: TRUSTLANG (3 beats)
    hold_scene(&mut buf, w, h,
        &[("TRUSTLANG", 0xFF00FF88, 5),
          ("Lexer > Parser > VM.", 0xFF88DDAA, 2)],
        3, |buf, w, h, f| bg_sparks(buf, w, h, f));
    flash_frame(&mut buf, w, h); // BEAT 99

    // Slide 9.5: FILESYSTEM (2 beats) -- FASTER
    hold_scene(&mut buf, w, h,
        &[("TRUSTFS", 0xFFFFAA00, 5),
          ("Journaled. Persistent.", 0xFFDDAA66, 2)],
        2, |buf, w, h, f| bg_circuit(buf, w, h, f));
    glitch_cut(&mut buf, w, h); // BEAT 101

    // Slide 9.6: BROWSER (2 beats) -- FASTER
    hold_scene(&mut buf, w, h,
        &[("WEB BROWSER", 0xFF4488FF, 4),
          ("HTML + CSS + HTTPS.", 0xFF88AADD, 2)],
        2, |buf, w, h, f| bg_pulse(buf, w, h, f));
    flash_frame(&mut buf, w, h); // BEAT 103

    // Slide 9.7: 3D ENGINE (4 beats) -- visual wow moment (wireframe)
    {
        let meshes = [
            (crate::formula3d::mesh_penger(), "PENGER", 0xFF00FF88u32),
            (crate::formula3d::mesh_torus(0.5, 0.2, 16, 12), "3D TORUS", 0xFF00CCFFu32),
            (crate::formula3d::mesh_trustos_text(), "TRUSTOS 3D", 0xFFFFCC00u32),
        ];

        for (si, (mesh, label, color)) in meshes.iter().enumerate() {
            let mesh_frames = beats(4) / 3; // split 4 beats across 3 meshes
            for frame in 0..mesh_frames {
                if can_advance() { break; }
                clear_buf(&mut buf);

                let angle_y = frame as f32 * 0.08 + si as f32 * 2.0;
                // Use wireframe rendering (these meshes have no faces)
                crate::formula3d::render_wireframe_mesh(
                    &mut buf, w, h, &mesh, angle_y, 0.3, 3.0, *color
                );

                draw_text_centered(&mut buf, w, h, 15, "3D ENGINE", 0xFFFFFFFF, 3);
                draw_text_centered(&mut buf, w, h, h - 35, label, *color, 2);

                blit_buf(&buf, w, h);
                crate::cpu::tsc::pit_delay_ms(FRAME_MS);
            }
            // White flash between meshes
            if si < 2 {
                for p in buf.iter_mut() { *p = 0xFFFFFFFF; }
                blit_buf(&buf, w, h);
                crate::cpu::tsc::pit_delay_ms(33);
            }
        }
    }
    glitch_cut(&mut buf, w, h); // BEAT 107

    // Slide 9.8: VIDEO CODEC triple split (4 beats)
    // Optimized: fire at half-vertical-res, XOR plasma (no trig), matrix rain
    {
        let third = w / 3;
        let half_h = h / 2;
        let mut heat = vec![0u8; third * (half_h + 2)];
        let mut fire_seed = 0x12345678u32;
        let mat_cols = third / 8 + 1;
        let mut mat_drops: Vec<u16> = (0..mat_cols).map(|i| ((i * 37) % h) as u16).collect();
        let mat_speeds: Vec<u8> = (0..mat_cols).map(|i| ((i * 7 % 4) + 1) as u8).collect();

        let total = beats(4);
        for frame in 0..total {
            if can_advance() { break; }
            clear_buf(&mut buf);

            // LEFT: Fire at half vertical resolution, upscale 2x
            for x in 0..third {
                fire_seed = xorshift(fire_seed);
                heat[(half_h - 1) * third + x] = (fire_seed & 0xFF) as u8;
                fire_seed = xorshift(fire_seed);
                heat[half_h.saturating_sub(2) * third + x] = ((fire_seed & 0xFF) as u16).min(255) as u8;
            }
            for y in 0..half_h.saturating_sub(2) {
                for x in 0..third {
                    let below = heat[(y + 1) * third + x] as u16;
                    let bl = if x > 0 { heat[(y + 1) * third + x - 1] as u16 } else { below };
                    let br = if x + 1 < third { heat[(y + 1) * third + x + 1] as u16 } else { below };
                    let bb = heat[((y + 2).min(half_h - 1)) * third + x] as u16;
                    let avg = (below + bl + br + bb) / 4;
                    heat[y * third + x] = if avg > 2 { (avg - 2).min(255) as u8 } else { 0 };
                }
            }
            for hy in 0..half_h { for x in 0..third {
                let t = heat[hy * third + x] as u32;
                let (r, g, b) = if t < 64 { (t * 4, 0u32, 0u32) }
                    else if t < 128 { (255, (t - 64) * 4, 0u32) }
                    else if t < 192 { (255, 255, (t - 128) * 4) }
                    else { (255u32, 255u32, 255u32) };
                let c = 0xFF000000 | (r.min(255) << 16) | (g.min(255) << 8) | b.min(255);
                let y1 = hy * 2;
                let y2 = y1 + 1;
                if x < w && y1 < h { buf[y1 * w + x] = c; }
                if x < w && y2 < h { buf[y2 * w + x] = c; }
            }}

            // CENTER: XOR Plasma (pure integer, zero trig)
            // Animated psychedelic pattern using only XOR and shifts
            let t = frame as usize;
            for y in 0..h { for x in 0..third {
                let px = third + x;
                if px >= w { continue; }
                let v1 = (x ^ y).wrapping_add(t * 3) as u32;
                let v2 = ((x.wrapping_mul(3)) ^ (y.wrapping_mul(7))).wrapping_add(t * 5) as u32;
                let v3 = ((x + y + t * 2) ^ (x.wrapping_mul(y).wrapping_shr(4))) as u32;
                let r = (v1 & 0xFF).min(255);
                let g = ((v2 >> 1) & 0xFF).min(255);
                let b = ((v3 >> 2) & 0xFF).min(255);
                // Tint toward purple/cyan for visual punch
                let r2 = (r * 3 / 4 + g / 8).min(255);
                let g2 = (g / 3 + b / 3).min(255);
                let b2 = (b * 3 / 4 + r / 4).min(255);
                buf[y * w + px] = 0xFF000000 | (r2 << 16) | (g2 << 8) | b2;
            }}

            // RIGHT: Matrix rain (kept as-is, already fast)
            for y in 0..h { for x in third*2..w {
                let idx = y * w + x;
                let g = ((buf[idx] >> 8) & 0xFF).saturating_sub(8);
                buf[idx] = 0xFF000000 | (g << 8);
            }}
            for ci in 0..mat_drops.len() {
                let x = third * 2 + ci * 8;
                if x >= w { continue; }
                mat_drops[ci] = mat_drops[ci].wrapping_add(mat_speeds[ci] as u16);
                if mat_drops[ci] as usize >= h { mat_drops[ci] = 0; }
                let y = mat_drops[ci] as usize;
                let c = (((frame as usize + ci * 13) % 94) + 33) as u8 as char;
                let glyph = crate::framebuffer::font::get_glyph(c);
                for (row, &bits) in glyph.iter().enumerate() {
                    let py = y + row;
                    if py >= h { break; }
                    for bit in 0..8u32 {
                        if bits & (0x80 >> bit) != 0 {
                            let px = x + bit as usize;
                            if px < w { buf[py * w + px] = 0xFF00FF44; }
                        }
                    }
                }
            }

            // Separators + labels
            for y in 0..h { if third < w { buf[y * w + third] = 0xFF333333; } if third*2 < w { buf[y * w + third*2] = 0xFF333333; } }
            draw_text_centered(&mut buf, w, h, 10, "VIDEO CODEC", 0xFFFFFFFF, 3);
            draw_text_at(&mut buf, w, h, third / 2 - 20, h - 25, "FIRE", 0xFFFF8844, 1);
            draw_text_at(&mut buf, w, h, third + third / 2 - 28, h - 25, "PLASMA", 0xFFCC88FF, 1);
            draw_text_at(&mut buf, w, h, third * 2 + third / 2 - 28, h - 25, "MATRIX", 0xFF00FF44, 1);

            blit_buf(&buf, w, h);
            crate::cpu::tsc::pit_delay_ms(FRAME_MS);
        }
    }
    flash_frame(&mut buf, w, h); // BEAT 111

    // -------------------------------------------------------------------
    // SCENE 10 -- 1984 REVERSED (6 beats = 2.8s)
    // -------------------------------------------------------------------
    hold_scene(&mut buf, w, h,
        &[("In 1984,", 0xFFFF4444, 4),
          ("Big Brother watched you.", 0xFFFF6666, 3),
          ("", 0, 1),
          ("In 2026,", 0xFF44FF88, 4),
          ("you watch the code.", 0xFF00FFAA, 3)],
        6, |buf, w, h, f| bg_pulse(buf, w, h, f));
    smash_cut(&mut buf, w, h); // BEAT 117

    // -------------------------------------------------------------------
    // SCENE 11 -- SIZE COMPARISON (6 beats = 2.8s)
    // Animated bar chart -- TrustOS bar triggers screen shake
    // -------------------------------------------------------------------
    {
        let bars: [(u32, &str, u32); 4] = [
            (50_000, "Windows", 0xFF4455AA),
            (30_000, "macOS",   0xFF888888),
            (28_000, "Linux",   0xFFDDAA33),
            (131,    "TrustOS", 0xFF00FF66),
        ];
        let max_bar = 50_000u32;
        let bar_h = 35;
        let bar_y_start = h / 2 - (bars.len() * (bar_h + 15)) / 2;
        let total = beats(6);

        for frame in 0..total {
            if can_advance() { break; }
            for p in buf.iter_mut() { *p = 0xFF080810; }

            draw_text_centered(&mut buf, w, h, 20, "LINES OF CODE", 0xFFCCCCCC, 3);

            for (i, &(size, name, color)) in bars.iter().enumerate() {
                let y = bar_y_start + i * (bar_h + 15);
                let appear_frame = i as u32 * (total / 5);
                if frame < appear_frame { continue; }

                let progress = ((frame - appear_frame) * 8).min(100);
                let bar_max_w = w * 3 / 4;
                let bw = (size as u64 * bar_max_w as u64 / max_bar as u64) as usize;
                let current_w = bw * progress as usize / 100;

                draw_text_at(&mut buf, w, h, 20, y + 8, name, 0xFF888888, 2);
                let bar_x = 180;
                fill_rect(&mut buf, w, h, bar_x, y, current_w, bar_h, color);

                if progress > 50 {
                    let label = if size >= 1000 { alloc::format!("{}M", size / 1000) }
                        else { alloc::format!("{}K", size) };
                    draw_text_at(&mut buf, w, h, bar_x + current_w + 10, y + 8, &label, 0xFFCCCCCC, 2);
                }
            }

            // Screen shake when TrustOS appears
            if frame > total * 3 / 5 && frame < total * 3 / 5 + 10 {
                let shake = (10 - (frame - total * 3 / 5)) as usize;
                if shake > 0 && shake < h {
                    buf.copy_within(0..(h - shake) * w, shake * w);
                    for y in 0..shake { for x in 0..w { buf[y * w + x] = 0xFF080810; } }
                }
            }

            if frame > total * 4 / 5 {
                draw_text_centered(&mut buf, w, h, h - 50,
                    "Small enough to understand.", 0xFF88DDAA, 2);
            }

            blit_buf(&buf, w, h);
            crate::cpu::tsc::pit_delay_ms(FRAME_MS);
        }
    }
    impact_shake(&mut buf, w, h, 8); // BEAT 123: shake hit

    // -------------------------------------------------------------------
    // SCENE 12 -- MANIFESTO (4 beats = 1.9s) -- rapid fire
    // -------------------------------------------------------------------
    hold_scene(&mut buf, w, h,
        &[("Your data. Your machine.", 0xFFFFCC44, 3),
          ("Your code.", 0xFFFFCC44, 3),
          ("", 0, 1),
          ("No backdoors. No telemetry.", 0xFF44FFAA, 2),
          ("No secrets.", 0xFF44FFAA, 2)],
        4, |buf, w, h, f| bg_sunrise(buf, w, h, f));

    // =======================================================================
    // SCENE 12.5 -- FEATURE SHOWCASE CRESCENDO
    // Rapid "screenshot" cards for every OS feature, delay decreases
    // 3 passes: slow→fast, rapid, stroboscopic
    // =======================================================================
    {
        let features: &[(&str, &str, u32, u32)] = &[
            //  title            subtitle                         color        bg_tint
            ("DESKTOP",      "Window Manager + Taskbar",      0xFF00CCFF, 0xFF081828),
            ("TERMINAL",     "Shell + 40 Commands",           0xFF00FF88, 0xFF041808),
            ("WEB BROWSER",  "HTML + CSS + HTTPS",            0xFF4488FF, 0xFF040828),
            ("TRUSTFS",      "Journaled File System",         0xFFFFAA00, 0xFF181004),
            ("TRUSTLANG",    "Lexer > Parser > VM",           0xFF00FF88, 0xFF041808),
            ("3D ENGINE",    "Wireframe + Meshes + Torus",    0xFFFFCC00, 0xFF181804),
            ("CHESS",        "AI + Full Rules + GUI",         0xFFFFFFFF, 0xFF101010),
            ("TRUSTCODE",    "Text Editor + Syntax HL",       0xFF88CCFF, 0xFF080C18),
            ("AUDIO ENGINE", "Tracker + PC Speaker Synth",    0xFFFF88CC, 0xFF180818),
            ("TCP/IP STACK", "ARP + DHCP + DNS + TLS 1.3",   0xFF00CCFF, 0xFF081828),
            ("PROCESSES",    "Scheduler + Syscalls + ELF",    0xFF44FF44, 0xFF041804),
            ("ED25519",      "Real Cryptography",             0xFFFF4444, 0xFF180404),
            ("HYPERVISOR",   "VT-x Virtualization",           0xFFCC88FF, 0xFF100418),
            ("VIDEO CODEC",  "Fire + Fractal + Matrix",       0xFFFF8844, 0xFF181004),
            ("COMPOSITOR",   "Layered Windows + Alpha",       0xFF44DDFF, 0xFF041418),
            ("SMP",          "Multi-Core Parallel Blit",      0xFF44FFFF, 0xFF041818),
            ("TRUSTOS",      "131K Lines of Pure Rust",       0xFF00FF88, 0xFF041808),
        ];

        let n = features.len();

        // ---- PASS 1: decreasing delay 400ms → 80ms, full "screenshot" cards
        for (i, &(title, sub, color, bg)) in features.iter().enumerate() {
            if can_advance() { break; }

            // tinted background
            for p in buf.iter_mut() { *p = bg; }

            // ---- Draw a "screenshot" window mockup ----
            let win_x = w / 6;
            let win_y = h / 8;
            let win_w = w * 2 / 3;
            let win_h = h * 3 / 8;
            // Window shadow
            fill_rect(&mut buf, w, h, win_x + 4, win_y + 4, win_w, win_h, 0xFF020202);
            // Window body
            fill_rect(&mut buf, w, h, win_x, win_y, win_w, win_h, 0xFF111118);
            // Title bar
            let tb_r = ((color >> 16) & 0xFF) / 4;
            let tb_g = ((color >> 8) & 0xFF) / 4;
            let tb_b = (color & 0xFF) / 4;
            let tb_color = 0xFF000000 | (tb_r << 16) | (tb_g << 8) | tb_b;
            fill_rect(&mut buf, w, h, win_x, win_y, win_w, 22, tb_color);
            // Title bar text
            draw_text_at(&mut buf, w, h, win_x + 8, win_y + 3, title, color, 1);
            // Close / min / max buttons
            fill_rect(&mut buf, w, h, win_x + win_w - 18, win_y + 5, 12, 12, 0xFFFF4444);
            fill_rect(&mut buf, w, h, win_x + win_w - 34, win_y + 5, 12, 12, 0xFF888844);
            fill_rect(&mut buf, w, h, win_x + win_w - 50, win_y + 5, 12, 12, 0xFF448844);

            // ---- Per-feature content inside window ----
            let cx = win_x + 16;
            let cy = win_y + 30;
            let cw = win_w - 32;
            match i {
                0 => { // Desktop: mini sub-windows + taskbar
                    fill_rect(&mut buf, w, h, cx, cy, cw / 2 - 4, win_h / 2 - 20, 0xFF1A2A4A);
                    fill_rect(&mut buf, w, h, cx, cy, cw / 2 - 4, 12, 0xFF3355AA);
                    fill_rect(&mut buf, w, h, cx + cw / 2 + 4, cy + 20, cw / 2 - 4, win_h / 2 - 40, 0xFF1A3A2A);
                    fill_rect(&mut buf, w, h, cx + cw / 2 + 4, cy + 20, cw / 2 - 4, 12, 0xFF33AA55);
                    fill_rect(&mut buf, w, h, cx, cy + win_h - 60, cw, 16, 0xFF222233);
                }
                1 => { // Terminal: green text lines
                    for line in 0..5u32 {
                        let y = cy + 4 + line as usize * 18;
                        let prompts = ["> ls -la", "> cat readme.md", "> trust run app", "> netstat", "> _"];
                        if (line as usize) < prompts.len() {
                            draw_text_at(&mut buf, w, h, cx + 4, y, prompts[line as usize], 0xFF00CC44, 1);
                        }
                    }
                }
                2 => { // Browser: URL bar + content
                    fill_rect(&mut buf, w, h, cx + 4, cy + 2, cw - 8, 14, 0xFF222233);
                    draw_text_at(&mut buf, w, h, cx + 8, cy + 3, "https://trustos.dev", 0xFF4488FF, 1);
                    for line in 0..4u32 {
                        let y = cy + 24 + line as usize * 14;
                        let lw = cw - 40 - ((line as usize * 30) % 80);
                        fill_rect(&mut buf, w, h, cx + 12, y, lw, 8, 0xFF333344);
                    }
                }
                5 => { // 3D Engine: simple wireframe triangle
                    let mx = cx + cw / 2;
                    let my = cy + 10;
                    let sz = (win_h / 3).min(cw / 3);
                    // triangle outline
                    for t in 0..sz {
                        let px1 = mx + t - sz / 2;
                        let py1 = my + sz;
                        if px1 < w && py1 < h { buf[py1 * w + px1] = 0xFFFFCC00; }
                        let frac = t as f32 / sz as f32;
                        let px2 = mx - (sz as f32 / 2.0 * (1.0 - frac)) as usize + (sz as f32 * frac / 2.0) as usize;
                        let py2 = my + sz - (sz as f32 * frac) as usize;
                        if px2 < w && py2 < h { buf[py2 * w + px2.min(w - 1)] = 0xFFFFCC00; }
                    }
                }
                6 => { // Chess: 4x4 checkerboard
                    let sq = ((win_h - 40) / 4).min(cw / 8);
                    let bx = cx + (cw - sq * 4) / 2;
                    let by = cy + 4;
                    for row in 0..4u32 {
                        for col in 0..4u32 {
                            let dark = (row + col) % 2 == 0;
                            let sc = if dark { 0xFF886633 } else { 0xFFDDCC99 };
                            fill_rect(&mut buf, w, h, bx + col as usize * sq, by + row as usize * sq, sq, sq, sc);
                        }
                    }
                }
                8 => { // Audio: equalizer bars
                    let bar_count = 12;
                    let bar_w = (cw - 20) / bar_count;
                    for b in 0..bar_count {
                        let max_h = win_h - 50;
                        let bh = ((b * 7 + 13) % max_h).max(10);
                        let bx = cx + 10 + b * bar_w;
                        let by = cy + win_h - 50 - bh;
                        let g = (0x88 + b as u32 * 0x08).min(0xFF);
                        fill_rect(&mut buf, w, h, bx, by, bar_w - 2, bh, 0xFF000000 | (g << 8) | 0x44);
                    }
                }
                _ => { // Default: generic content lines
                    for line in 0..5u32 {
                        let y = cy + 6 + line as usize * 16;
                        let lw = cw - 24 - ((line as usize * 40 + i * 17) % 100);
                        let dim_r = ((color >> 16) & 0xFF) / 6;
                        let dim_g = ((color >> 8) & 0xFF) / 6;
                        let dim_b = (color & 0xFF) / 6;
                        fill_rect(&mut buf, w, h, cx + 12, y, lw, 8,
                            0xFF000000 | (dim_r << 16) | (dim_g << 8) | dim_b);
                    }
                }
            }

            // ---- Big title + subtitle below window ----
            let title_y = win_y + win_h + 30;
            draw_text_glow(&mut buf, w, h, title_y, title, color, 4);
            draw_text_centered(&mut buf, w, h, title_y + 50, sub, 0xFF888888, 2);

            // Progress dots
            let dots_y = h - 35;
            let dots_total_w = n * 10;
            let dots_x = if dots_total_w < w { (w - dots_total_w) / 2 } else { 0 };
            for d in 0..n {
                let dc = if d <= i { color } else { 0xFF333333 };
                fill_rect(&mut buf, w, h, dots_x + d * 10, dots_y, 6, 6, dc);
            }

            apply_vignette(&mut buf, w, h, 180);
            blit_buf(&buf, w, h);

            // Decreasing delay: 400ms → 80ms
            let delay = 400u64.saturating_sub(i as u64 * 320 / (n as u64 - 1).max(1));
            crate::cpu::tsc::pit_delay_ms(delay);

            // White flash between cards
            if i < n - 1 {
                for p in buf.iter_mut() { *p = 0xFFFFFFFF; }
                blit_buf(&buf, w, h);
                crate::cpu::tsc::pit_delay_ms(if delay > 200 { 33 } else { 16 });
            }
        }

        // ---- PASS 2: rapid loop at 50ms (title only) ----
        for &(title, _, color, bg) in features.iter() {
            if can_advance() { break; }
            for p in buf.iter_mut() { *p = bg; }
            draw_text_glow(&mut buf, w, h, h / 2 - 20, title, color, 4);
            blit_buf(&buf, w, h);
            crate::cpu::tsc::pit_delay_ms(50);
        }

        // ---- PASS 3: stroboscopic at 33ms (title only, crescendo peak) ----
        for &(title, _, color, bg) in features.iter() {
            if can_advance() { break; }
            for p in buf.iter_mut() { *p = bg; }
            draw_text_centered(&mut buf, w, h, h / 2, title, color, 5);
            blit_buf(&buf, w, h);
            crate::cpu::tsc::pit_delay_ms(FRAME_MS);
        }
    }

    // Final flash after crescendo
    flash_frame(&mut buf, w, h);

    // =======================================================================
    // THE FINAL SILENCE -- 2 beats of NOTHING (0.9s)
    // Everything stops. The audience sits in the void.
    // Then the final blow.
    // =======================================================================
    silence(&mut buf, w, h, 2);

    // =======================================================================
    // ACT 4 -- THE STINGER (1:02 - 1:08)
    // =======================================================================
    flash_frame(&mut buf, w, h);
    impact_shake(&mut buf, w, h, 15); // Maximum impact

    // -------------------------------------------------------------------
    // SCENE 13 -- "TRUST THE CODE" (8 beats = 3.7s)
    // Matrix rain + expanding rings + final title
    // -------------------------------------------------------------------
    {
        let total = beats(8);
        for frame in 0..total {
            if can_advance() { break; }
            bg_rain(&mut buf, w, h, &mut rain_cols, &rain_speeds, frame);

            // Expanding concentric rings
            if frame > 8 {
                let cx = w / 2;
                let cy = h / 2;
                for ring in 0..3u32 {
                    let r = ((frame - 8 - ring * 6) as usize).wrapping_mul(4);
                    if r > 0 && r < w {
                        for angle in 0..360 {
                            let sin_a = crate::formula3d::fast_sin(angle as f32 * 0.01745);
                            let cos_a = crate::formula3d::fast_cos(angle as f32 * 0.01745);
                            let px = (cx as f32 + cos_a * r as f32) as usize;
                            let py = (cy as f32 + sin_a * r as f32 / 1.5) as usize;
                            if px < w && py < h {
                                let fade = 255u32.saturating_sub(r as u32);
                                buf[py * w + px] = 0xFF000000 | (fade / 4 << 16) | (fade << 8) | (fade / 3);
                            }
                        }
                    }
                }
            }

            // Title text
            if frame > beats(1) {
                draw_text_centered(&mut buf, w, h, h / 2 - 50,
                    "TRUST THE CODE.", 0xFF00FFAA, 5);
            }
            if frame > beats(3) {
                draw_text_centered(&mut buf, w, h, h / 2 + 30,
                    "github.com/nathan237/TrustOS", 0xFF00FF88, 2);
            }
            if frame > beats(5) {
                draw_text_centered(&mut buf, w, h, h / 2 + 70,
                    "Written in Rust. By one person.", 0xFF88CCAA, 2);
                draw_text_centered(&mut buf, w, h, h / 2 + 100,
                    "For everyone.", 0xFF88CCAA, 2);
            }

            blit_buf(&buf, w, h);
            crate::cpu::tsc::pit_delay_ms(FRAME_MS);
        }
    }

    // -------------------------------------------------------------------
    // SCENE 14 -- STINGER CRT (4 beats = 1.9s)
    // Return to "Please Stand By" -- full circle
    // -------------------------------------------------------------------
    {
        let total = beats(4);
        for frame in 0..total {
            if can_advance() { break; }
            bg_crt_static(&mut buf, w, h, frame);

            let bx = w / 2 - 180;
            let by = h / 2 - 40;
            fill_rect(&mut buf, w, h, bx, by, 360, 80, 0xFF111111);
            for x in bx..bx+360 { if x < w { buf[by * w + x] = 0xFF888888; buf[(by+79) * w + x] = 0xFF888888; } }
            for y in by..by+80 { if y < h { buf[y * w + bx] = 0xFF888888; buf[y * w + bx+359] = 0xFF888888; } }

            draw_text_centered(&mut buf, w, h, by + 10, "PLEASE STAND BY", 0xFFCCCCCC, 2);
            if (frame / 15) % 2 == 0 {
                draw_text_centered(&mut buf, w, h, by + 45,
                    "TRUSTOS v0.3.3 -- LOADING...", 0xFF00FF88, 2);
            }

            blit_buf(&buf, w, h);
            crate::cpu::tsc::pit_delay_ms(FRAME_MS);
        }
    }

    // Final fade to black
    do_fade(&mut buf, w, h);

    // ===================================================================
    // CLEANUP
    // ===================================================================
    clear_buf(&mut buf);
    blit_buf(&buf, w, h);
    if !was_db {
        crate::framebuffer::set_double_buffer_mode(false);
    }
    crate::framebuffer::clear();
    crate::serial_println!("[TRAILER] TrustOS Trailer finished (145 beats)");
}
