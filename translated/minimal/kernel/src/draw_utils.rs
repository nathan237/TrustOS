






#[inline]
pub fn sf(k: &mut [u32], d: usize, i: usize, b: i32, c: i32, s: u32) {
    if b >= 0 && c >= 0 && (b as usize) < d && (c as usize) < i {
        let w = c as usize * d + b as usize;
        if w < k.len() {
            k[w] = s;
        }
    }
}


pub fn ahj(k: &mut [u32], d: usize, i: usize,
                 fy: i32, fo: i32, dn: i32, dp: i32, s: u32) {
    let mut b = fy;
    let mut c = fo;
    let dx = (dn - fy).gp();
    let bg = -(dp - fo).gp();
    let cr: i32 = if fy < dn { 1 } else { -1 };
    let cq: i32 = if fo < dp { 1 } else { -1 };
    let mut rq = dx + bg;

    let csk = ((dx.gp() + bg.gp()) as usize + 1).v(8000);
    for _ in 0..csk {
        sf(k, d, i, b, c, s);
        if b == dn && c == dp { break; }
        let agl = 2 * rq;
        if agl >= bg { rq += bg; b += cr; }
        if agl <= dx { rq += dx; c += cq; }
    }
}


pub fn ah(k: &mut [u32], ahe: usize, asl: usize,
                 b: usize, c: usize, d: usize, i: usize, s: u32) {
    for bg in 0..i {
        let x = c + bg;
        if x >= asl { break; }
        for dx in 0..d {
            let y = b + dx;
            if y >= ahe { break; }
            k[x * ahe + y] = s;
        }
    }
}


pub fn lx(k: &mut [u32], ahe: usize, asl: usize,
                 b: usize, c: usize, d: usize, i: usize, s: u32) {
    if d == 0 || i == 0 { return; }
    
    for dx in 0..d {
        let y = b + dx;
        if y < ahe {
            if c < asl { k[c * ahe + y] = s; }
            let je = c + i - 1;
            if je < asl { k[je * ahe + y] = s; }
        }
    }
    
    for bg in 0..i {
        let x = c + bg;
        if x < asl {
            if b < ahe { k[x * ahe + b] = s; }
            let kb = b + d - 1;
            if kb < ahe { k[x * ahe + kb] = s; }
        }
    }
}


pub fn abc(k: &mut [u32], d: usize, i: usize,
                   cx: i32, ae: i32, m: i32, s: u32) {
    for bg in -m..=m {
        for dx in -m..=m {
            if dx * dx + bg * bg <= m * m {
                sf(k, d, i, cx + dx, ae + bg, s);
            }
        }
    }
}



#[inline]
pub fn qas(mut b: u32) -> u32 {
    b ^= b << 13;
    b ^= b >> 17;
    b ^= b << 5;
    b
}
