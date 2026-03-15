//! Shared Buffer Drawing Utilities
//!
//! Common drawing primitives for off-screen `&mut [u32]` pixel buffers.
//! Used by model_editor, chess3d, trailer, and other modules that
//! render to intermediate buffers before compositing.

/// Write a single pixel to a buffer with bounds checking.
#[inline]
pub fn put_pixel(buf: &mut [u32], w: usize, h: usize, x: i32, y: i32, color: u32) {
    if x >= 0 && y >= 0 && (x as usize) < w && (y as usize) < h {
        let idx = y as usize * w + x as usize;
        if idx < buf.len() {
            buf[idx] = color;
        }
    }
}

/// Draw a line using Bresenham's algorithm into a pixel buffer.
pub fn draw_line(buf: &mut [u32], w: usize, h: usize,
                 x0: i32, y0: i32, x1: i32, y1: i32, color: u32) {
    let mut x = x0;
    let mut y = y0;
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx: i32 = if x0 < x1 { 1 } else { -1 };
    let sy: i32 = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;

    let max_steps = ((dx.abs() + dy.abs()) as usize + 1).min(8000);
    for _ in 0..max_steps {
        put_pixel(buf, w, h, x, y, color);
        if x == x1 && y == y1 { break; }
        let e2 = 2 * err;
        if e2 >= dy { err += dy; x += sx; }
        if e2 <= dx { err += dx; y += sy; }
    }
}

/// Fill a rectangle in a pixel buffer.
pub fn fill_rect(buf: &mut [u32], buf_w: usize, buf_h: usize,
                 x: usize, y: usize, w: usize, h: usize, color: u32) {
    for dy in 0..h {
        let py = y + dy;
        if py >= buf_h { break; }
        for dx in 0..w {
            let px = x + dx;
            if px >= buf_w { break; }
            buf[py * buf_w + px] = color;
        }
    }
}

/// Draw a rectangle outline in a pixel buffer.
pub fn draw_rect(buf: &mut [u32], buf_w: usize, buf_h: usize,
                 x: usize, y: usize, w: usize, h: usize, color: u32) {
    if w == 0 || h == 0 { return; }
    // Top and bottom edges
    for dx in 0..w {
        let px = x + dx;
        if px < buf_w {
            if y < buf_h { buf[y * buf_w + px] = color; }
            let by = y + h - 1;
            if by < buf_h { buf[by * buf_w + px] = color; }
        }
    }
    // Left and right edges
    for dy in 0..h {
        let py = y + dy;
        if py < buf_h {
            if x < buf_w { buf[py * buf_w + x] = color; }
            let rx = x + w - 1;
            if rx < buf_w { buf[py * buf_w + rx] = color; }
        }
    }
}

/// Fill a circle in a pixel buffer.
pub fn fill_circle(buf: &mut [u32], w: usize, h: usize,
                   cx: i32, cy: i32, r: i32, color: u32) {
    for dy in -r..=r {
        for dx in -r..=r {
            if dx * dx + dy * dy <= r * r {
                put_pixel(buf, w, h, cx + dx, cy + dy, color);
            }
        }
    }
}

/// XorShift32 pseudo-random number generator.
/// Common PRNG used for visual effects, dithering, etc.
#[inline]
pub fn xorshift32(mut x: u32) -> u32 {
    x ^= x << 13;
    x ^= x >> 17;
    x ^= x << 5;
    x
}
