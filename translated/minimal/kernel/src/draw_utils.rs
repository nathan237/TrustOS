






#[inline]
pub fn put_pixel(buf: &mut [u32], w: usize, h: usize, x: i32, y: i32, color: u32) {
    if x >= 0 && y >= 0 && (x as usize) < w && (y as usize) < h {
        let idx = y as usize * w + x as usize;
        if idx < buf.len() {
            buf[idx] = color;
        }
    }
}


pub fn draw_line(buf: &mut [u32], w: usize, h: usize,
                 bm: i32, az: i32, x1: i32, y1: i32, color: u32) {
    let mut x = bm;
    let mut y = az;
    let dx = (x1 - bm).abs();
    let ad = -(y1 - az).abs();
    let am: i32 = if bm < x1 { 1 } else { -1 };
    let ak: i32 = if az < y1 { 1 } else { -1 };
    let mut err = dx + ad;

    let ayd = ((dx.abs() + ad.abs()) as usize + 1).min(8000);
    for _ in 0..ayd {
        put_pixel(buf, w, h, x, y, color);
        if x == x1 && y == y1 { break; }
        let pg = 2 * err;
        if pg >= ad { err += ad; x += am; }
        if pg <= dx { err += dx; y += ak; }
    }
}


pub fn fill_rect(buf: &mut [u32], buf_w: usize, buf_h: usize,
                 x: usize, y: usize, w: usize, h: usize, color: u32) {
    for ad in 0..h {
        let o = y + ad;
        if o >= buf_h { break; }
        for dx in 0..w {
            let p = x + dx;
            if p >= buf_w { break; }
            buf[o * buf_w + p] = color;
        }
    }
}


pub fn draw_rect(buf: &mut [u32], buf_w: usize, buf_h: usize,
                 x: usize, y: usize, w: usize, h: usize, color: u32) {
    if w == 0 || h == 0 { return; }
    
    for dx in 0..w {
        let p = x + dx;
        if p < buf_w {
            if y < buf_h { buf[y * buf_w + p] = color; }
            let dc = y + h - 1;
            if dc < buf_h { buf[dc * buf_w + p] = color; }
        }
    }
    
    for ad in 0..h {
        let o = y + ad;
        if o < buf_h {
            if x < buf_w { buf[o * buf_w + x] = color; }
            let da = x + w - 1;
            if da < buf_w { buf[o * buf_w + da] = color; }
        }
    }
}


pub fn fill_circle(buf: &mut [u32], w: usize, h: usize,
                   cx: i32, u: i32, r: i32, color: u32) {
    for ad in -r..=r {
        for dx in -r..=r {
            if dx * dx + ad * ad <= r * r {
                put_pixel(buf, w, h, cx + dx, u + ad, color);
            }
        }
    }
}



#[inline]
pub fn jsa(mut x: u32) -> u32 {
    x ^= x << 13;
    x ^= x >> 17;
    x ^= x << 5;
    x
}
