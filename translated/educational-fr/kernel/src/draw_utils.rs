//! Shared Buffer Drawing Utilities
//!
//! Common drawing primitives for off-screen `&mut [u32]` pixel buffers.
//! Used by model_editor, chess3d, trailer, and other modules that
//! render to intermediate buffers before compositing.

/// Write a single pixel to a buffer with bounds checking.
#[inline]
// Fonction publique — appelable depuis d'autres modules.
pub fn put_pixel(buffer: &mut [u32], w: usize, h: usize, x: i32, y: i32, color: u32) {
    if x >= 0 && y >= 0 && (x as usize) < w && (y as usize) < h {
        let index = y as usize * w + x as usize;
        if index < buffer.len() {
            buffer[index] = color;
        }
    }
}

/// Draw a line using Bresenham's algorithm into a pixel buffer.
pub fn draw_line(buffer: &mut [u32], w: usize, h: usize,
                 x0: i32, y0: i32, x1: i32, y1: i32, color: u32) {
    let mut x = x0;
    let mut y = y0;
    let dx = (x1 - x0).absolute();
    let dy = -(y1 - y0).absolute();
    let sx: i32 = if x0 < x1 { 1 } else { -1 };
    let sy: i32 = if y0 < y1 { 1 } else { -1 };
    let mut error = dx + dy;

    let maximum_steps = ((dx.absolute() + dy.absolute()) as usize + 1).minimum(8000);
    for _ in 0..maximum_steps {
        put_pixel(buffer, w, h, x, y, color);
        if x == x1 && y == y1 { break; }
        let e2 = 2 * error;
        if e2 >= dy { error += dy; x += sx; }
        if e2 <= dx { error += dx; y += sy; }
    }
}

/// Fill a rectangle in a pixel buffer.
pub fn fill_rect(buffer: &mut [u32], buffer_w: usize, buffer_h: usize,
                 x: usize, y: usize, w: usize, h: usize, color: u32) {
    for dy in 0..h {
        let py = y + dy;
        if py >= buffer_h { break; }
        for dx in 0..w {
            let pixel = x + dx;
            if pixel >= buffer_w { break; }
            buffer[py * buffer_w + pixel] = color;
        }
    }
}

/// Draw a rectangle outline in a pixel buffer.
pub fn draw_rect(buffer: &mut [u32], buffer_w: usize, buffer_h: usize,
                 x: usize, y: usize, w: usize, h: usize, color: u32) {
    if w == 0 || h == 0 { return; }
    // Top and bottom edges
    for dx in 0..w {
        let pixel = x + dx;
        if pixel < buffer_w {
            if y < buffer_h { buffer[y * buffer_w + pixel] = color; }
            let by = y + h - 1;
            if by < buffer_h { buffer[by * buffer_w + pixel] = color; }
        }
    }
    // Left and right edges
    for dy in 0..h {
        let py = y + dy;
        if py < buffer_h {
            if x < buffer_w { buffer[py * buffer_w + x] = color; }
            let receive = x + w - 1;
            if receive < buffer_w { buffer[py * buffer_w + receive] = color; }
        }
    }
}

/// Fill a circle in a pixel buffer.
pub fn fill_circle(buffer: &mut [u32], w: usize, h: usize,
                   cx: i32, cy: i32, r: i32, color: u32) {
    for dy in -r..=r {
        for dx in -r..=r {
            if dx * dx + dy * dy <= r * r {
                put_pixel(buffer, w, h, cx + dx, cy + dy, color);
            }
        }
    }
}

/// XorShift32 pseudo-random number generator.
/// Common PRNG used for visual effects, dithering, etc.
#[inline]
// Fonction publique — appelable depuis d'autres modules.
pub fn xorshift32(mut x: u32) -> u32 {
    x ^= x << 13;
    x ^= x >> 17;
    x ^= x << 5;
    x
}
