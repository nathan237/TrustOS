





use alloc::vec;
use alloc::vec::Vec;

const BS_: u64 = 33; 

use crate::draw_utils::jsa as xorshift;









const BN_: usize = 72;
const BA_: usize = 48;
const CIB_: [u64; BN_] = [
    
    0b_000000000000000000000001_100000000000000000000000, 
    0b_000000000000000000000001_100000000000000000000000, 
    0b_000000000000000000000011_110000000000000000000000, 
    0b_000000000000000000000011_110000000000000000000000, 
    0b_000000000000000000000111_111000000000000000000000, 
    0b_000000000000000000000111_111000000000000000000000, 
    0b_000000000000000000001111_111100000000000000000000, 
    0b_000000000000000000001111_111100000000000000000000, 
    
    0b_000000000000000000011111_111110000000000000000000, 
    0b_000000000000000000011111_111110000000000000000000, 
    0b_000000000000000000111111_111111000000000000000000, 
    0b_000000000000000000111111_111111000000000000000000, 
    0b_000000000000000001111111_111111100000000000000000, 
    0b_000000000000000001111111_111111100000000000000000, 
    0b_000000000000000011111111_111111110000000000000000, 
    0b_000000000000000011111111_111111110000000000000000, 
    
    0b_000000000000000111111111_111111111000000000000000, 
    0b_000000000000001111111111_111111111100000000000000, 
    0b_000000000000011111111111_111111111110000000000000, 
    0b_000000000000111111111111_111111111111000000000000, 
    0b_000000000001111111111111_111111111111100000000000, 
    0b_000000000011111111111111_111111111111110000000000, 
    0b_000000000111111111111111_111111111111111000000000, 
    0b_000000001111111111111111_111111111111111100000000, 
    
    0b_000000011111111111111111_111111111111111110000000, 
    0b_000000111111111111111111_111111111111111111000000, 
    0b_000001111111111111111111_111111111111111111100000, 
    0b_000011111111111111111111_111111111111111111110000, 
    0b_000111111111111111111111_111111111111111111111000, 
    0b_001111111111111111111111_111111111111111111111100, 
    0b_011111111111111111111111_111111111111111111111110, 
    0b_111111111111111111111111_111111111111111111111111, 
    
    0b_111111111111110000111111_111110000111111111111111, 
    0b_111111111111100000011111_111100000011111111111111, 
    0b_111111111111000000001111_111000000001111111111111, 
    0b_111111111110000000001111_111000000000111111111111, 
    0b_011111111110000000000111_111000000000011111111110, 
    0b_011111111100000000000111_111000000000001111111110, 
    0b_001111111100000000000111_111000000000001111111100, 
    0b_001111111100000000000011_110000000000001111111100, 
    
    0b_000111111100000000000011_110000000000001111111000, 
    0b_000011111110000000000011_110000000000011111110000, 
    0b_000001111111000000000011_110000000000111111100000, 
    0b_000000111111100000000011_110000000001111111000000, 
    0b_000000011111110000000011_110000000011111110000000, 
    0b_000000001111111000000011_110000000111111100000000, 
    0b_000000000111111100000011_110000001111111000000000, 
    0b_000000000011111111000011_110000111111110000000000, 
    
    0b_000000000001111111100111_111001111111100000000000, 
    0b_000000000000111111111111_111111111111000000000000, 
    0b_000000000000011111111111_111111111110000000000000, 
    0b_000000000000001111111111_111111111100000000000000, 
    0b_000000000000000111111111_111111111000000000000000, 
    0b_000000000000000111111111_111111111000000000000000, 
    0b_000000000000001111111111_111111111100000000000000, 
    0b_000000000000011111111111_111111111110000000000000, 
    
    0b_000000000000001111111111_111111111100000000000000, 
    0b_000000000000000111111111_111111111000000000000000, 
    0b_000000000000000011111111_111111110000000000000000, 
    0b_000000000000000001111111_111111100000000000000000, 
    0b_000000000000000000111111_111111000000000000000000, 
    0b_000000000000000000011111_111110000000000000000000, 
    0b_000000000000000000001111_111100000000000000000000, 
    0b_000000000000000000000111_111000000000000000000000, 
    
    0b_000000000000000000000111_111000000000000000000000, 
    0b_000000000000000000000011_110000000000000000000000, 
    0b_000000000000000000000011_110000000000000000000000, 
    0b_000000000000000000000001_100000000000000000000000, 
    0b_000000000000000000000001_100000000000000000000000, 
    0b_000000000000000000000001_100000000000000000000000, 
    0b_000000000000000000000000_100000000000000000000000, 
    0b_000000000000000000000000_000000000000000000000000, 
];


#[inline]
fn bhr(x: usize, y: usize) -> bool {
    if x >= BA_ || y >= BN_ { return false; }
    (CIB_[y] >> (BA_ - 1 - x)) & 1 == 1
}


#[inline]
fn nao(x: usize, y: usize) -> bool {
    if !bhr(x, y) { return false; }
    
    if x == 0 || !bhr(x - 1, y) { return true; }
    if x >= BA_ - 1 || !bhr(x + 1, y) { return true; }
    if y == 0 || !bhr(x, y - 1) { return true; }
    if y >= BN_ - 1 || !bhr(x, y + 1) { return true; }
    false
}




fn ofn(buf: &mut [u32], fv: usize, ov: usize,
               cx: usize, u: usize, scale: usize, frame: u32) {
    let ile = BA_ * scale;
    let ild = BN_ * scale;
    let fh = cx.saturating_sub(ile / 2);
    let hk = u.saturating_sub(ild / 2);

    
    let ixt = frame as usize;

    for ak in 0..ild {
        let ly = ak / scale; 
        let o = hk + ak;
        if o >= ov { continue; }

        for am in 0..ile {
            let fe = am / scale; 
            let p = fh + am;
            if p >= fv { continue; }

            if !bhr(fe, ly) { continue; }

            let idx = o * fv + p;

            if nao(fe, ly) {
                
                let alp = (ly as u32 * 80 / BN_ as u32) + 20;
                let base = 140u32 + alp;
                
                let dx = if fe > BA_ / 2 { fe - BA_ / 2 } else { BA_ / 2 - fe };
                let mll = 30u32.saturating_sub(dx as u32 * 2);
                let v = (base + mll).min(240);
                buf[idx] = 0xFF000000 | (v << 16) | (v << 8) | v;
            } else {
                
                
                let blf = (fe.wrapping_mul(7919) + 31) % 97;
                let speed = 1 + blf % 3;
                let bok = (ly + ixt * speed + blf * 5) % 17;
                
                let flg = (fe.wrapping_mul(2654435761_usize.wrapping_shr(0))
                    .wrapping_add(ly.wrapping_mul(40503))
                    .wrapping_add(ixt * speed)) % 37;

                
                let drh = bok;
                let intensity = if drh < 2 {
                    200u32  
                } else if drh < 6 {
                    (120u32).saturating_sub(drh as u32 * 12)
                } else {
                    25u32 + (flg as u32 % 20)  
                };

                
                let bqx = am % (scale * 4);  
                let aho = ak % (scale * 6);
                let msc = flg < 20 && bqx > 0 && bqx < scale * 3
                    && aho > 0 && aho < scale * 5;

                if msc {
                    let g = intensity.min(255);
                    let r = g / 8;
                    let b = g / 5;
                    buf[idx] = 0xFF000000 | (r << 16) | (g << 8) | b;
                } else {
                    
                    let g = 8u32 + (flg as u32 % 8);
                    buf[idx] = 0xFF000000 | (g << 8);
                }
            }
        }
    }
}





fn pf(buf: &mut [u32], w: usize, h: usize,
                 cx: usize, u: usize, c: char, color: u32, scale: usize) {
    let du = crate::framebuffer::font::ol(c);
    for (row, &bits) in du.iter().enumerate() {
        for bf in 0..8u32 {
            if bits & (0x80 >> bf) != 0 {
                for ak in 0..scale {
                    for am in 0..scale {
                        let p = cx + bf as usize * scale + am;
                        let o = u + row * scale + ak;
                        if p < w && o < h { buf[o * w + p] = color; }
                    }
                }
            }
        }
    }
}


fn hth(buf: &mut [u32], w: usize, h: usize,
                      cx: usize, u: usize, c: char, color: u32, scale: usize, bsz: u32) {
    let du = crate::framebuffer::font::ol(c);
    let alg = (color >> 16) & 0xFF;
    let ahp = (color >> 8) & 0xFF;
    let cb = color & 0xFF;
    
    if bsz > 0 {
        let cdv = bsz as usize;
        for (row, &bits) in du.iter().enumerate() {
            for bf in 0..8u32 {
                if bits & (0x80 >> bf) != 0 {
                    
                    let mbk = cx + bf as usize * scale + scale / 2;
                    let mbl = u + row * scale + scale / 2;
                    
                    let step = if cdv > 3 { 2 } else { 1 };
                    let mut ad = -(cdv as i32);
                    while ad <= cdv as i32 {
                        let mut dx = -(cdv as i32);
                        while dx <= cdv as i32 {
                            let jq = (dx * dx + ad * ad) as u32;
                            let ju = bsz * bsz;
                            if jq > 0 && jq < ju {
                                let p = (mbk as i32 + dx) as usize;
                                let o = (mbl as i32 + ad) as usize;
                                if p < w && o < h {
                                    let att = 255u32.saturating_sub(jq * 255 / ju) / 4;
                                    let idx = o * w + p;
                                    let dst = buf[idx];
                                    let qw = (dst >> 16) & 0xFF;
                                    let afb = (dst >> 8) & 0xFF;
                                    let fu = dst & 0xFF;
                                    let nr = (qw + alg * att / 255).min(255);
                                    let ayn = (afb + ahp * att / 255).min(255);
                                    let ayj = (fu + cb * att / 255).min(255);
                                    buf[idx] = 0xFF000000 | (nr << 16) | (ayn << 8) | ayj;
                                }
                            }
                            dx += step;
                        }
                        ad += step;
                    }
                }
            }
        }
    }
    
    for (row, &bits) in du.iter().enumerate() {
        for bf in 0..8u32 {
            if bits & (0x80 >> bf) != 0 {
                for ak in 0..scale {
                    for am in 0..scale {
                        let p = cx + bf as usize * scale + am;
                        let o = u + row * scale + ak;
                        if p < w && o < h { buf[o * w + p] = color; }
                    }
                }
            }
        }
    }
}


fn byy(buf: &mut [u32], w: usize, h: usize,
                  y: usize, text: &str, color: u32, scale: usize) {
    let gr = text.len() * 8 * scale;
    let am = if gr < w { (w - gr) / 2 } else { 0 };
    
    let shadow = scale.max(1);
    for (i, c) in text.chars().enumerate() {
        pf(buf, w, h, am + i * 8 * scale + shadow, y + shadow, c, 0xFF000000, scale);
    }
    
    let glow = (scale as u32 * 3).min(12);
    for (i, c) in text.chars().enumerate() {
        hth(buf, w, h, am + i * 8 * scale, y, c, color, scale, glow);
    }
}


fn ctp(buf: &mut [u32], w: usize, h: usize, strength: u32) {
    let cx = w / 2;
    let u = h / 2;
    let ndi = (cx * cx + u * u) as u32;
    
    for y in (0..h).step_by(2) {
        let ad = if y > u { y - u } else { u - y };
        for x in (0..w).step_by(2) {
            let dx = if x > cx { x - cx } else { cx - x };
            let jq = (dx * dx + ad * ad) as u32;
            let ha = jq * strength / ndi;
            let dim = ha.min(200) as u32;
            
            for dc in 0..2u32 {
                for bx in 0..2u32 {
                    let p = x + bx as usize;
                    let o = y + dc as usize;
                    if p < w && o < h {
                        let idx = o * w + p;
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


fn htp(buf: &mut [u32], w: usize, h: usize,
                    cx: usize, u: usize, radius: usize,
                    center_r: u32, center_g: u32, center_b: u32, alpha: u32) {
    let ju = (radius * radius) as u32;
    let ajb = u.saturating_sub(radius);
    let bkg = (u + radius).min(h);
    let eek = cx.saturating_sub(radius);
    let csz = (cx + radius).min(w);
    for y in ajb..bkg {
        let ad = if y > u { y - u } else { u - y };
        for x in eek..csz {
            let dx = if x > cx { x - cx } else { cx - x };
            let jq = (dx * dx + ad * ad) as u32;
            if jq < ju {
                let att = alpha * (ju - jq) / ju;
                let ir = center_r * att / 255;
                let axo = center_g * att / 255;
                let czj = center_b * att / 255;
                let idx = y * w + x;
                let dst = buf[idx];
                let qw = (dst >> 16) & 0xFF;
                let afb = (dst >> 8) & 0xFF;
                let fu = dst & 0xFF;
                buf[idx] = 0xFF000000
                    | ((qw + ir).min(255) << 16)
                    | ((afb + axo).min(255) << 8)
                    | (fu + czj).min(255);
            }
        }
    }
}


fn liu(buf: &mut [u32], w: usize, h: usize,
                       cx: usize, u: usize, da: usize, cm: usize, color: u32) {
    let ajb = u.saturating_sub(cm);
    let bkg = (u + cm).min(h);
    let eek = cx.saturating_sub(da);
    let csz = (cx + da).min(w);
    let bja = (da * da) as u64;
    let apa = (cm * cm) as u64;
    for y in ajb..bkg {
        let ad = if y > u { y - u } else { u - y };
        for x in eek..csz {
            let dx = if x > cx { x - cx } else { cx - x };
            if (dx as u64 * dx as u64) * apa + (ad as u64 * ad as u64) * bja < bja * apa {
                buf[y * w + x] = color;
            }
        }
    }
}


fn jwy(buf: &mut [u32], w: usize, h: usize) {
    let cx = w / 2;
    let u = h / 2;
    for y in 0..h {
        
        let oky = if y % 3 == 0 { 40u32 } else { 0 };
        
        let ad = if y > u { y - u } else { u - y };
        let loe = (ad * ad * 60 / (u * u).max(1)) as u32;
        for x in 0..w {
            let dx = if x > cx { x - cx } else { cx - x };
            let lod = (dx * dx * 60 / (cx * cx).max(1)) as u32;
            let fdd = oky + loe + lod;
            if fdd > 0 {
                let idx = y * w + x;
                let c = buf[idx];
                let r = ((c >> 16) & 0xFF).saturating_sub(fdd);
                let g = ((c >> 8) & 0xFF).saturating_sub(fdd);
                let b = (c & 0xFF).saturating_sub(fdd);
                buf[idx] = 0xFF000000 | (r << 16) | (g << 8) | b;
            }
        }
    }
}

fn draw_text_at(buf: &mut [u32], w: usize, h: usize,
                x: usize, y: usize, text: &str, color: u32, scale: usize) {
    for (i, c) in text.chars().enumerate() {
        pf(buf, w, h, x + i * 8 * scale, y, c, color, scale);
    }
}

fn draw_text_centered(buf: &mut [u32], w: usize, h: usize,
                      y: usize, text: &str, color: u32, scale: usize) {
    let gr = text.len() * 8 * scale;
    let am = if gr < w { (w - gr) / 2 } else { 0 };
    draw_text_at(buf, w, h, am, y, text, color, scale);
}

fn fill_rect(buf: &mut [u32], w: usize, h: usize,
             da: usize, cm: usize, lk: usize, pp: usize, color: u32) {
    for ad in 0..pp {
        for dx in 0..lk {
            let p = da + dx;
            let o = cm + ad;
            if p < w && o < h { buf[o * w + p] = color; }
        }
    }
}

fn uq(buf: &mut [u32]) {
    
    #[cfg(target_arch = "x86_64")]
    unsafe {
        use core::arch::x86_64::*;
        let awh = _mm_set1_epi32(0xFF000000u32 as i32);
        let ptr = buf.as_mut_ptr() as *mut __m128i;
        let count = buf.len() / 4;
        for i in 0..count {
            _mm_storeu_si128(ptr.add(i), awh);
        }
        for i in (count * 4)..buf.len() {
            buf[i] = 0xFF000000;
        }
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        for aa in buf.iter_mut() { *aa = 0xFF000000; }
    }
}

fn ev(buf: &[u32], w: usize, h: usize) {
    
    crate::framebuffer::fjl(buf.as_ptr(), w, h);
}

fn vi(buf: &mut [u32], w: usize, h: usize) {
    for _ in 0..40 {
        for p in buf.iter_mut() {
            let r = ((*p >> 16) & 0xFF).saturating_sub(8);
            let g = ((*p >> 8) & 0xFF).saturating_sub(8);
            let b = (*p & 0xFF).saturating_sub(8);
            *p = 0xFF000000 | (r << 16) | (g << 8) | b;
        }
        ev(buf, w, h);
        crate::cpu::tsc::hq(BS_);
    }
    uq(buf);
    ev(buf, w, h);
    crate::cpu::tsc::hq(300);
}


fn dqv(buf: &mut [u32], w: usize, h: usize) {
    let mut seed = 0xDEADBEEFu32;
    for _ in 0..3 {
        for p in buf.iter_mut() {
            seed ^= seed << 13; seed ^= seed >> 17; seed ^= seed << 5;
            let v = seed & 0xFF;
            *p = 0xFF000000 | (v << 16) | (v << 8) | v;
        }
        ev(buf, w, h);
        crate::cpu::tsc::hq(40);
    }
    uq(buf);
    ev(buf, w, h);
    crate::cpu::tsc::hq(80);
}


fn ahm() -> bool {
    if let Some(k) = crate::keyboard::kr() {
        return k == 0x1B || k == b' ' || k == b'\n' || k == b'\r';
    }
    false
}






fn hho(buf: &mut [u32], w: usize, h: usize, frame: u32) {
    let mut seed = frame.wrapping_mul(2654435761);
    
    for p in buf.iter_mut() {
        seed ^= seed << 13; seed ^= seed >> 17; seed ^= seed << 5;
        let aig = (seed & 0x1F) as u32; 
        *p = 0xFF000000 | (aig << 16) | (aig << 8) | aig;
    }
    
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


fn cub(buf: &mut [u32], w: usize, h: usize, frame: u32) {
    let scroll = (frame as usize * 2) % h;
    for y in 0..h {
        let ak = (y + scroll) % h;
        let gwn = (ak / 4) % 2 == 0;
        for x in 0..w {
            let adi = if gwn { 35u32 } else { 15 };
            let flash = if (ak % 60) < 2 { 30u32 } else { 0 };
            let r = (adi + flash).min(65);
            buf[y * w + x] = 0xFF000000 | (r << 16) | 0x0205;
        }
    }
}


fn bqq(buf: &mut [u32], w: usize, h: usize, frame: u32) {
    let phase = (frame % 160) as u32;
    let kq = if phase < 80 { phase / 2 } else { (160 - phase) / 2 };
    let cnz = ((frame + 40) % 120) as u32;
    let gox = if cnz < 60 { cnz / 2 } else { (120 - cnz) / 2 };
    for y in 0..h {
        let bkj = (y as u32 * 40) / h as u32;
        for x in 0..w {
            let eem = (x as u32 * 10) / w as u32;
            let r = (bkj / 4 + gox / 3).min(40);
            let g = (eem / 3).min(15);
            let b = (bkj + kq + eem / 2).min(80);
            buf[y * w + x] = 0xFF000000 | (r << 16) | (g << 8) | b;
        }
    }
}


fn cud(buf: &mut [u32], w: usize, h: usize, frame: u32) {
    for p in buf.iter_mut() {
        let g = ((*p >> 8) & 0xFF).saturating_sub(10);
        let r = ((*p >> 16) & 0xFF).saturating_sub(8);
        *p = 0xFF000000 | (r << 16) | (g << 8);
    }
    for i in 0..30u32 {
        let seed = (i.wrapping_mul(2654435761).wrapping_add(frame.wrapping_mul(37))) as usize;
        let p = (seed.wrapping_mul(7919)) % w;
        let gru = (frame as usize + seed) % h;
        let o = h.saturating_sub(gru);
        let na = (50 + (seed % 50)) as u32;
        if p < w && o < h {
            buf[o * w + p] = 0xFF000000 | (na / 4 << 16) | (na << 8) | (na / 3);
            if p + 1 < w { buf[o * w + p + 1] = 0xFF000000 | (na << 8); }
        }
    }
}


fn djf(buf: &mut [u32], w: usize, h: usize, frame: u32) {
    let cmf = frame.min(80);
    for y in 0..h {
        let bkj = y as u32 * 100 / h as u32;
        let csq = if bkj > 50 { (bkj - 50).min(50) + cmf } else { cmf / 2 };
        let r = (csq * 2).min(100);
        let g = (csq * 3 / 4).min(50);
        let b = 20u32.saturating_sub(csq / 3);
        for x in 0..w {
            buf[y * w + x] = 0xFF000000 | (r << 16) | (g << 8) | b;
        }
    }
}


fn ctz(buf: &mut [u32], w: usize, h: usize, frame: u32) {
    for aa in buf.iter_mut() { *aa = 0xFF0A0A14; }
    let trace = 0xFF0F2818u32;
    for i in 0..16u32 {
        let ty = ((i.wrapping_mul(7919) as usize) % h) & !3;
        let bu = ((i.wrapping_mul(104729) as usize) % w) & !3;
        if ty < h { for x in 0..w { buf[ty * w + x] = trace; } }
        if bu < w { for y in 0..h { buf[y * w + bu] = trace; } }
    }
    let o = ((frame as usize * 3) % h) & !3;
    if o < h {
        let wl = (w / 4).min(120);
        let bdd = (frame as usize * 5) % w;
        for dx in 0..wl {
            let p = (bdd + dx) % w;
            buf[o * w + p] = 0xFF00AA44;
            if o + 1 < h { buf[(o + 1) * w + p] = 0xFF00AA44; }
        }
    }
}


fn kbu(buf: &mut [u32], w: usize, h: usize,
           cols: &mut [u16], speeds: &[u8], frame: u32) {
    
    for ct in buf.iter_mut() {
        let g = ((*ct >> 8) & 0xFF) as u32;
        if g > 0 { *ct = 0xFF000000 | (g.saturating_sub(6) << 8); }
        else { *ct = 0xFF000000; }
    }
    for ci in 0..cols.len() {
        let x = ci * 8;
        if x >= w { continue; }
        cols[ci] = cols[ci].wrapping_add(speeds[ci] as u16);
        if cols[ci] as usize >= h { cols[ci] = 0; }
        let y = cols[ci] as usize;
        let c = (((frame as usize + ci * 13) % 94) + 33) as u8 as char;
        let du = crate::framebuffer::font::ol(c);
        for (row, &bits) in du.iter().enumerate() {
            let o = y + row;
            if o >= h { break; }
            for bf in 0..8u32 {
                if bits & (0x80 >> bf) != 0 {
                    let p = x + bf as usize;
                    if p < w { buf[o * w + p] = 0xFF00FF44; }
                }
            }
        }
    }
}


fn kbo(buf: &mut [u32], w: usize, h: usize, frame: u32) {
    
    for p in buf.iter_mut() {
        let r = ((*p >> 16) & 0xFF).saturating_sub(15);
        let g = ((*p >> 8) & 0xFF).saturating_sub(15);
        let b = (*p & 0xFF).saturating_sub(15);
        *p = 0xFF000000 | (r << 16) | (g << 8) | b;
    }
    
    for col in 0..(w / 10) {
        let x = col * 10 + 2;
        let seed = (col.wrapping_mul(7919).wrapping_add(frame as usize * 3)) % 97;
        let speed = 2 + seed % 4;
        let y = ((frame as usize * speed + col * 23) % (h + 40)).wrapping_sub(20);
        if y < h && x < w {
            let blu = if (col + frame as usize) % 2 == 0 { '0' } else { '1' };
            let du = crate::framebuffer::font::ol(blu);
            let na = 100u32 + (seed as u32 * 3) % 80;
            for (row, &bits) in du.iter().enumerate() {
                let o = y + row;
                if o >= h { break; }
                for bf in 0..8u32 {
                    if bits & (0x80 >> bf) != 0 {
                        let p = x + bf as usize;
                        if p < w {
                            buf[o * w + p] = 0xFF000000 | (na << 8) | (na / 4);
                        }
                    }
                }
            }
        }
    }
}



























































const BYQ_: u32 = 14;


#[inline]
fn tw(ae: u32) -> u32 { ae * BYQ_ }






fn gvj(buf: &mut [u32], w: usize, h: usize) {
    uq(buf);
    ev(buf, w, h);
    crate::cpu::tsc::hq(100);
}



fn bgl(buf: &mut [u32], w: usize, h: usize) {
    
    for aa in buf.iter_mut() { *aa = 0xFFFFFFFF; }
    ev(buf, w, h);
    crate::cpu::tsc::hq(33);
    
    for aa in buf.iter_mut() { *aa = 0xFFB0B0B0; }
    ev(buf, w, h);
    crate::cpu::tsc::hq(33);
    
    uq(buf);
    ev(buf, w, h);
    crate::cpu::tsc::hq(66);
}



fn gca(buf: &mut [u32], w: usize, h: usize, intensity: usize) {
    let mut seed = 0xCAFEBABEu32;
    for i in 0..6u32 {
        let dmn = intensity.saturating_sub(i as usize);
        if dmn == 0 { break; }
        seed = xorshift(seed);
        let fh = (seed as usize % (dmn * 2 + 1)).wrapping_sub(dmn);
        seed = xorshift(seed);
        let hk = (seed as usize % (dmn * 2 + 1)).wrapping_sub(dmn);
        
        let mut bow = vec![0xFF000000u32; w * h];
        for y in 0..h {
            let ak = (y as isize + hk as isize).max(0) as usize;
            if ak >= h { continue; }
            for x in 0..w {
                let am = (x as isize + fh as isize).max(0) as usize;
                if am >= w { continue; }
                bow[ak * w + am] = buf[y * w + x];
            }
        }
        ev(&bow, w, h);
        crate::cpu::tsc::hq(33);
    }
}



fn pul(buf: &mut [u32], w: usize, h: usize) {
    for step in 0..4u32 {
        let offset = (step + 1) as usize * w / 4;
        let mut bow = vec![0xFF000000u32; w * h];
        for y in 0..h {
            for x in offset..w {
                bow[y * w + x - offset] = buf[y * w + x];
            }
        }
        
        for y in 0..h {
            let fuc = w.saturating_sub(offset);
            for x in fuc..w {
                bow[y * w + x] = 0xFF080808;
            }
        }
        ev(&bow, w, h);
        crate::cpu::tsc::hq(25);
    }
    uq(buf);
    ev(buf, w, h);
    crate::cpu::tsc::hq(50);
}


fn fba(buf: &mut [u32], w: usize, h: usize, beat_count: u32) {
    uq(buf);
    for _ in 0..tw(beat_count) {
        if ahm() { return; }
        ev(buf, w, h);
        crate::cpu::tsc::hq(BS_);
    }
}







fn ppb<F>(buf: &mut [u32], w: usize, h: usize,
                 lines: &[(&str, u32, usize)],
                 ms_per_char: u64, max_beats: u32,
                 mut bg_fn: F)
where F: FnMut(&mut [u32], usize, usize, u32) {
    let vu: usize = lines.iter().map(|(t, _, _)| t.len()).sum();
    let yr = (ms_per_char / BS_).max(1) as u32;
    let dfz = vu as u32 * yr;
    let mma = tw(max_beats).saturating_sub(dfz);
    let av = dfz + mma;

    for frame in 0..av {
        if ahm() { return; }

        bg_fn(buf, w, h, frame);

        let hh = (frame / yr) as usize;
        let sn: usize = lines.iter().map(|(_, _, j)| 16 * j + 12).sum();
        let mut y = if sn < h { (h - sn) / 2 } else { 20 };
        let mut abx = 0usize;

        for &(text, color, scale) in lines {
            let gr = text.len() * 8 * scale;
            let am = if gr < w { (w - gr) / 2 } else { 0 };
            for (i, c) in text.chars().enumerate() {
                if abx + i >= hh { break; }
                pf(buf, w, h, am + i * 8 * scale, y, c, color, scale);
            }
            
            if hh > abx && hh < abx + text.len() {
                let ci = hh - abx;
                let cx = am + ci * 8 * scale;
                if (frame / 8) % 2 == 0 {
                    for u in y..y + 16 * scale {
                        if u < h && cx + 2 < w {
                            buf[u * w + cx] = 0xFFFFFFFF;
                            buf[u * w + cx + 1] = 0xFFFFFFFF;
                        }
                    }
                }
            }
            abx += text.len();
            y += 16 * scale + 12;
        }

        ev(buf, w, h);
        crate::cpu::tsc::hq(BS_);
    }
}


fn ckq<F>(buf: &mut [u32], w: usize, h: usize,
                 lines: &[(&str, u32, usize)], beat_count: u32,
                 mut bg_fn: F)
where F: FnMut(&mut [u32], usize, usize, u32) {
    for frame in 0..tw(beat_count) {
        if ahm() { return; }
        bg_fn(buf, w, h, frame);

        let sn: usize = lines.iter().map(|(_, _, j)| 16 * j + 12).sum();
        let mut y = if sn < h { (h - sn) / 2 } else { 20 };
        for &(text, color, scale) in lines {
            draw_text_centered(buf, w, h, y, text, color, scale);
            y += 16 * scale + 12;
        }

        ev(buf, w, h);
        crate::cpu::tsc::hq(BS_);
    }
}








































pub(super) fn ktd() {
    let (dy, dw) = crate::framebuffer::kv();
    let w = dy as usize;
    let h = dw as usize;

    
    let pu = crate::framebuffer::ajy();
    if !pu {
        crate::framebuffer::adw();
        crate::framebuffer::pr(true);
    }

    let mut buf = vec![0xFF000000u32; w * h];

    
    let xx = w / 8 + 1;
    let mut kk: Vec<u16> = (0..xx).map(|i| ((i * 37 + 13) % h) as u16).collect();
    let la: Vec<u8> = (0..xx).map(|i| (((i * 7 + 3) % 4) + 1) as u8).collect();

    crate::serial_println!("[TRAILER] TrustOS Trailer started (128 BPM beat-synced)");

    
    
    

    
    
    
    
    {
        let scale = if h > 600 { 6 } else { 4 };
        let bnk = h / 2 - 30;
        let ie = bnk + (BN_ * scale) / 2 + 20;
        let av = tw(16);

        for frame in 0..av {
            if ahm() { break; }
            uq(&mut buf);

            
            let caf = (frame * 3).min(120);
            let mey = 80 + (frame as usize * 2).min(h / 3);
            htp(&mut buf, w, h, w / 2, bnk, mey,
                             20, 80, 40, caf);

            ofn(&mut buf, w, h, w / 2, bnk, scale, frame);

            
            if frame > 20 {
                let kq = ((frame % 40) as i32 - 20).unsigned_abs() as u32;
                let ohg = (BA_ * scale / 2 + 10 + kq as usize) as f32;
                let grq = 60u32.saturating_sub(kq);
                for a in 0..180 {
                    let cc = a as f32 * 0.0349; 
                    let vt = crate::formula3d::eu(cc);
                    let vg = crate::formula3d::hr(cc);
                    for rh in 0..2 {
                        let r = ohg + rh as f32;
                        let p = (w as f32 / 2.0 + vg * r) as usize;
                        let o = (bnk as f32 + vt * r * 0.75) as usize;
                        if p < w && o < h {
                            let idx = o * w + p;
                            let g = ((buf[idx] >> 8) & 0xFF) + grq;
                            buf[idx] = 0xFF000000 | ((grq / 4) << 16) | (g.min(255) << 8) | (grq / 3);
                        }
                    }
                }
            }

            
            if frame < 60 {
                let dim = ((60 - frame) as u32 * 255 / 60) as u32;
                for p in buf.iter_mut() {
                    if *p != 0xFF000000 {
                        let r = ((*p >> 16) & 0xFF).saturating_sub(dim);
                        let g = ((*p >> 8) & 0xFF).saturating_sub(dim);
                        let b = (*p & 0xFF).saturating_sub(dim);
                        *p = 0xFF000000 | (r << 16) | (g << 8) | b;
                    }
                }
            }

            
            ctp(&mut buf, w, h, 180);

            
            if frame > tw(8) {
                let text = "TRUSTOS";
                let ceh = if h > 600 { 5 } else { 3 };
                let sub = frame - tw(8);
                let hh = (sub / 8).min(text.len() as u32) as usize;
                let gr = text.len() * 8 * ceh;
                let bu = if gr < w { (w - gr) / 2 } else { 0 };
                
                for (i, c) in text.chars().enumerate() {
                    if i >= hh { break; }
                    pf(&mut buf, w, h,
                        bu + i * 8 * ceh + 2, ie + 2, c, 0xFF000000, ceh);
                }
                
                for (i, c) in text.chars().enumerate() {
                    if i >= hh { break; }
                    hth(&mut buf, w, h,
                        bu + i * 8 * ceh, ie, c, 0xFFDDDDDD, ceh,
                        ceh as u32 * 3);
                }
            }

            ev(&buf, w, h);
            crate::cpu::tsc::hq(BS_);
        }
    }
    bgl(&mut buf, w, h); 

    
    
    
    
    {
        let av = tw(8);
        for frame in 0..av {
            if ahm() { break; }
            hho(&mut buf, w, h, frame);

            
            let cup = if w > 800 { 200 } else { 160 };
            let djh = 50;
            let bx = w / 2 - cup;
            let dc = h / 2 - djh;
            
            fill_rect(&mut buf, w, h, bx + 3, dc + 3, cup * 2, djh * 2, 0xFF050505);
            
            fill_rect(&mut buf, w, h, bx, dc, cup * 2, djh * 2, 0xFF111111);
            
            for x in bx..bx + cup * 2 {
                if x < w {
                    let alp = 0xFF555555 + ((x - bx) as u32 * 0x40 / (cup as u32 * 2));
                    let bmk = 0xFF000000 | (alp & 0xFF) << 16 | (alp & 0xFF) << 8 | (alp & 0xFF);
                    buf[dc * w + x] = bmk;
                    buf[(dc + djh * 2 - 1) * w + x] = bmk;
                }
            }
            for y in dc..dc + djh * 2 {
                if y < h {
                    buf[y * w + bx] = 0xFF888888;
                    buf[y * w + (bx + cup * 2 - 1).min(w - 1)] = 0xFF888888;
                }
            }

            let ptw = if (frame / 15) % 2 == 0 { 0xFFFFCC00 } else { 0xFFFF8800 };
            byy(&mut buf, w, h, dc + 12, "! WARNING !", ptw, 2);
            byy(&mut buf, w, h, dc + 55, "PLEASE STAND BY", 0xFFCCCCCC, 2);

            
            jwy(&mut buf, w, h);
            ctp(&mut buf, w, h, 200);

            ev(&buf, w, h);
            crate::cpu::tsc::hq(BS_);
        }
    }
    dqv(&mut buf, w, h); 

    
    
    
    
    {
        let dpe = w / 2;
        let cxl = h / 2 - 40;
        let fvy = w / 4;
        let dpf = h / 8;
        let av = tw(12);

        for frame in 0..av {
            if ahm() { break; }
            bqq(&mut buf, w, h, frame);

            
            let ici = (frame as u32 * 3).min(180);
            htp(&mut buf, w, h, dpe, cxl, fvy + 40, ici, 0x10, 0x00, ici);

            
            liu(&mut buf, w, h, dpe, cxl, fvy, dpf, 0xFFDDCCCC);

            
            let czw = dpf * 2 / 3;
            let ccw = dpf / 3;
            for ad in 0..czw*2+2 {
                for dx in 0..czw*2+2 {
                    let lh = dx as i32 - czw as i32;
                    let kf = ad as i32 - czw as i32;
                    let jq = lh*lh + kf*kf;
                    let ihq = (czw as i32) * (czw as i32);
                    let gnw = (ccw as i32) * (ccw as i32);
                    if jq <= ihq && jq >= gnw {
                        let p = (dpe as i32 + lh) as usize;
                        let o = (cxl as i32 + kf) as usize;
                        if p < w && o < h {
                            
                            let t = (jq - gnw) as u32 * 255 / (ihq - gnw).max(1) as u32;
                            let r = 0xFF;
                            let g = (0x66u32).saturating_sub(t * 0x66 / 255) as u8;
                            buf[o * w + p] = 0xFF000000 | (r as u32) << 16 | (g as u32) << 8;
                        }
                    }
                }
            }

            
            for ad in 0..ccw*2 {
                for dx in 0..ccw*2 {
                    let lh = dx as i32 - ccw as i32;
                    let kf = ad as i32 - ccw as i32;
                    if lh*lh + kf*kf < (ccw as i32 * ccw as i32) {
                        let p = (dpe as i32 + lh) as usize;
                        let o = (cxl as i32 + kf) as usize;
                        if p < w && o < h { buf[o * w + p] = 0xFF080000; }
                    }
                }
            }

            
            for thick in 0..2i32 {
                let lrr = fvy as i32 + thick;
                let lom = dpf as i32 + thick;
                for cc in 0..720 {
                    let a = cc as f32 * 0.008727; 
                    let vt = crate::formula3d::eu(a);
                    let vg = crate::formula3d::hr(a);
                    let p = (dpe as f32 + vg * lrr as f32) as usize;
                    let o = (cxl as f32 + vt * lom as f32) as usize;
                    if p < w && o < h {
                        buf[o * w + p] = 0xFFFF3333;
                    }
                }
            }

            
            let chars = (frame / 3).min(28) as usize;
            let mah = "BIG BROTHER IS WATCHING YOU.";
            let osg: alloc::string::String = mah.chars().take(chars).collect();
            byy(&mut buf, w, h, cxl + dpf + 50, &osg, 0xFFFF4444, 3);

            ctp(&mut buf, w, h, 160);

            ev(&buf, w, h);
            crate::cpu::tsc::hq(BS_);
        }
    }
    gvj(&mut buf, w, h); 

    
    
    
    
    {
        let myq = [
            "location: TRACKED",
            "camera: ACTIVE",
            "keystrokes: LOGGED",
            "microphone: RECORDING",
            "contacts: UPLOADED",
            "messages: SCANNED",
            "browsing: PROFILED",
            "identity: SOLD",
        ];
        let av = tw(10);
        for frame in 0..av {
            if ahm() { break; }
            cub(&mut buf, w, h, frame);

            
            let kq = (crate::formula3d::eu(frame as f32 * 0.15) * 40.0 + 60.0) as u8;
            let bxq = 0xFF000000 | (kq as u32) << 16;
            fill_rect(&mut buf, w, h, 0, 0, w, 6, bxq);
            fill_rect(&mut buf, w, h, 0, h.saturating_sub(6), w, 6, bxq);

            let myr = (frame / 12).min(8) as usize;
            for (i, &line) in myq.iter().enumerate().take(myr) {
                let y = 80 + i * 50;
                let otq = ((frame as usize).saturating_sub(i * 12)).min(w);
                let x = w.saturating_sub(otq);
                
                draw_text_at(&mut buf, w, h, x + 2, y + 2, line, 0xFF220000, 2);
                
                let bfk = line.find(':').unwrap_or(line.len());
                let label = &line[..bfk];
                let value = &line[bfk..];
                let aok = label.len() * 16;
                draw_text_at(&mut buf, w, h, x, y, label, 0xFFAA3333, 2);
                draw_text_at(&mut buf, w, h, x + aok, y, value, 0xFFFF4444, 2);
            }

            if frame > tw(6) {
                byy(&mut buf, w, h, h - 60,
                    "Every keystroke. Every click.", 0xFFFF6666, 2);
            }
            if frame > tw(8) {
                byy(&mut buf, w, h, h - 30,
                    "Every file. Every thought.", 0xFFFF4444, 2);
            }

            ctp(&mut buf, w, h, 180);

            ev(&buf, w, h);
            crate::cpu::tsc::hq(BS_);
        }
    }
    pul(&mut buf, w, h); 

    
    
    
    
    ppb(&mut buf, w, h,
        &[("Your OS has", 0xFFAAFFAA, 2),
          ("50,000,000 lines of code.", 0xFF00FF88, 3),
          ("", 0, 1),
          ("You can read ZERO of them.", 0xFFFF4444, 3)],
        40, 8, |buf, w, h, f| kbo(buf, w, h, f));
    gvj(&mut buf, w, h); 

    
    
    
    
    {
        let dnk = [
            "Kernel source code ............",
            "Driver implementations ........",
            "Encryption keys ...............",
            "Telemetry endpoints ...........",
            "Backdoor protocols ............",
            "Data collection routines ......",
        ];
        let av = tw(8);
        for frame in 0..av {
            if ahm() { break; }
            
            for y in 0..h { for x in 0..w {
                let adi: u32 = 0x0C + (y as u32 * 4 / h as u32);
                let agd: u32 = 0x14 + (y as u32 * 6 / h as u32);
                let apu: u32 = 0x28 + (y as u32 * 8 / h as u32);
                let grid = if (x % 20 < 1) || (y % 20 < 1) { 0x06u32 } else { 0u32 };
                let r = (adi + grid).min(255);
                let g = (agd + grid).min(255);
                let b = (apu + grid).min(255);
                buf[y * w + x] = 0xFF000000 | (r << 16) | (g << 8) | b;
            }}

            let bet = h / 2 - dnk.len() * 22;
            for (i, &line) in dnk.iter().enumerate() {
                let y = bet + i * 44;
                let gr = line.len() * 16;
                let bu = if gr < w { (w - gr) / 2 } else { 0 };
                
                draw_text_at(&mut buf, w, h, bu + 1, y + 1, line, 0xFF223344, 2);
                draw_text_at(&mut buf, w, h, bu, y, line, 0xFF7799BB, 2);

                let iyy = 10 + i as u32 * 12;
                if frame > iyy {
                    let egf = ((frame - iyy) as usize * 30).min(gr);
                    
                    fill_rect(&mut buf, w, h, bu, y, egf, 30, 0xFF0A0A0A);
                    if egf > 2 {
                        
                        fill_rect(&mut buf, w, h, bu, y, egf, 1, 0xFF222222);
                    }
                    if egf >= gr {
                        
                        let gwb = bu + gr / 2 - 64;
                        let gwc = y + 4;
                        
                        for jh in gwc.saturating_sub(4)..gwc + 30 {
                            for hc in gwb.saturating_sub(8)..gwb + 140 {
                                if hc < w && jh < h {
                                    let qb = buf[jh * w + hc];
                                    let or = ((qb >> 16) & 0xFF) as u32;
                                    let nr = (or + 30).min(255);
                                    buf[jh * w + hc] = (qb & 0xFF00FFFF) | (nr << 16);
                                }
                            }
                        }
                        draw_text_at(&mut buf, w, h, gwb, gwc,
                            "[REDACTED]", 0xFFFF2222, 2);
                    }
                }
            }

            if frame > tw(6) {
                byy(&mut buf, w, h, h - 40,
                    "You trust what you cannot see.", 0xFF8888CC, 2);
            }

            ctp(&mut buf, w, h, 200);

            ev(&buf, w, h);
            crate::cpu::tsc::hq(BS_);
        }
    }
    bgl(&mut buf, w, h); 

    
    
    
    
    
    fba(&mut buf, w, h, 4); 

    
    
    
    bgl(&mut buf, w, h);
    gca(&mut buf, w, h, 12);

    
    
    
    
    {
        let av = tw(12);
        for frame in 0..av {
            if ahm() { break; }
            cud(&mut buf, w, h, frame);

            
            if frame > 5 && frame < tw(4) {
                let intensity = if frame < tw(2) { frame - 5 } else { tw(4) - frame };
                let cx = w / 2;
                let u = h / 2 - 20;
                for ob in 0..16 {
                    let cc = ob as f32 * 0.3927;
                    let vt = crate::formula3d::eu(cc);
                    let vg = crate::formula3d::hr(cc);
                    let len = intensity as f32 * 8.0;
                    for t in 0..len as usize {
                        let p = (cx as f32 + vg * t as f32) as usize;
                        let o = (u as f32 + vt * t as f32) as usize;
                        if p < w && o < h {
                            let na = (200 - t * 3).max(40) as u32;
                            buf[o * w + p] = 0xFF000000 | (na / 4 << 16) | (na << 8) | (na / 2);
                        }
                    }
                }
            }

            
            if frame > 8 {
                let bjp = if h > 600 { 7 } else { 5 };
                let alpha = ((frame - 8) * 12).min(255) as u32;
                let color = 0xFF000000 | (alpha / 3 << 16) | (alpha << 8) | (alpha / 2);
                draw_text_centered(&mut buf, w, h, h / 2 - 40, "TRUSTOS", color, bjp);

                if frame > tw(3) {
                    draw_text_centered(&mut buf, w, h, h / 2 + 40,
                        "The OS you can read. All of it.", 0xFF88DDAA, 2);
                }
            }

            ev(&buf, w, h);
            crate::cpu::tsc::hq(BS_);
        }
    }
    bgl(&mut buf, w, h); 

    
    
    
    
    {
        let kyg: [(u32, &str); 4] = [
            (131662, " lines of Rust"),
            (1, " author"),
            (0, " secrets"),
            (100, "% open source"),
        ];
        let av = tw(8);
        for frame in 0..av {
            if ahm() { break; }
            ctz(&mut buf, w, h, frame);

            let iir = (frame / tw(2)).min(4) as usize;
            let bet = h / 2 - iir * 35;

            for (i, &(target, label)) in kyg.iter().enumerate().take(iir) {
                let y = bet + i * 70;
                let oyj = frame.saturating_sub(i as u32 * tw(2));
                let progress = (oyj * 6).min(tw(2));
                let current = if target == 0 { 0 }
                    else { (target as u64 * progress as u64 / tw(2) as u64) as u32 };

                let rw = alloc::format!("{:>7}", current);
                let mag = alloc::format!("{}{}", rw, label);

                let scale = 3;
                let gr = mag.len() * 8 * scale;
                let bu = if gr < w { (w - gr) / 2 } else { 0 };

                for (ci, c) in rw.chars().enumerate() {
                    pf(&mut buf, w, h, bu + ci * 8 * scale, y, c, 0xFF00FF88, scale);
                }
                for (ci, c) in label.chars().enumerate() {
                    pf(&mut buf, w, h, bu + (rw.len() + ci) * 8 * scale, y, c, 0xFF44AA66, scale);
                }
            }

            ev(&buf, w, h);
            crate::cpu::tsc::hq(BS_);
        }
    }
    bgl(&mut buf, w, h); 

    
    
    
    

    
    {
        let stages = ["hello", "a7 f3 0b 2e c1", "[HDR|DATA|CRC]", ">>> WIRE >>>"];
        let ovv = [0xFF00FF88u32, 0xFF00CCFF, 0xFFFFAA00, 0xFF44FF44];
        let av = tw(4);

        for frame in 0..av {
            if ahm() { break; }
            ctz(&mut buf, w, h, frame);

            let laq = (frame * 4 / av).min(3) as usize;
            draw_text_centered(&mut buf, w, h, 30, "NETWORK STACK", 0xFF00CCFF, 3);

            let center_y = h / 2 - 80;
            for i in 0..=laq {
                let y = center_y + i * 50;
                let bx = w / 2 - 140;
                fill_rect(&mut buf, w, h, bx, y, 280, 35, 0xFF111122);
                draw_text_centered(&mut buf, w, h, y + 4, stages[i], ovv[i], 2);
            }

            ev(&buf, w, h);
            crate::cpu::tsc::hq(BS_);
        }
    }
    dqv(&mut buf, w, h); 

    
    ckq(&mut buf, w, h,
        &[("TLS 1.3", 0xFF00CCFF, 5),
          ("Full handshake. Real crypto.", 0xFF88BBDD, 2)],
        3, |buf, w, h, f| bqq(buf, w, h, f));
    bgl(&mut buf, w, h); 

    
    ckq(&mut buf, w, h,
        &[("GUI COMPOSITOR", 0xFFFFCC00, 4),
          ("Windows. Taskbar. Wallpaper.", 0xFFCCAA88, 2)],
        3, |buf, w, h, f| bqq(buf, w, h, f));
    dqv(&mut buf, w, h); 

    
    ckq(&mut buf, w, h,
        &[("TRUSTLANG", 0xFF00FF88, 5),
          ("Lexer > Parser > VM.", 0xFF88DDAA, 2)],
        3, |buf, w, h, f| cud(buf, w, h, f));
    bgl(&mut buf, w, h); 

    
    ckq(&mut buf, w, h,
        &[("TRUSTFS", 0xFFFFAA00, 5),
          ("Journaled. Persistent.", 0xFFDDAA66, 2)],
        2, |buf, w, h, f| ctz(buf, w, h, f));
    dqv(&mut buf, w, h); 

    
    ckq(&mut buf, w, h,
        &[("WEB BROWSER", 0xFF4488FF, 4),
          ("HTML + CSS + HTTPS.", 0xFF88AADD, 2)],
        2, |buf, w, h, f| bqq(buf, w, h, f));
    bgl(&mut buf, w, h); 

    
    {
        let nfa = [
            (crate::formula3d::mesh_penger(), "PENGER", 0xFF00FF88u32),
            (crate::formula3d::mesh_torus(0.5, 0.2, 16, 12), "3D TORUS", 0xFF00CCFFu32),
            (crate::formula3d::inm(), "TRUSTOS 3D", 0xFFFFCC00u32),
        ];

        for (si, (mesh, label, color)) in nfa.iter().enumerate() {
            let nep = tw(4) / 3; 
            for frame in 0..nep {
                if ahm() { break; }
                uq(&mut buf);

                let angle_y = frame as f32 * 0.08 + si as f32 * 2.0;
                
                crate::formula3d::ofv(
                    &mut buf, w, h, &mesh, angle_y, 0.3, 3.0, *color
                );

                draw_text_centered(&mut buf, w, h, 15, "3D ENGINE", 0xFFFFFFFF, 3);
                draw_text_centered(&mut buf, w, h, h - 35, label, *color, 2);

                ev(&buf, w, h);
                crate::cpu::tsc::hq(BS_);
            }
            
            if si < 2 {
                for aa in buf.iter_mut() { *aa = 0xFFFFFFFF; }
                ev(&buf, w, h);
                crate::cpu::tsc::hq(33);
            }
        }
    }
    dqv(&mut buf, w, h); 

    
    
    {
        let pn = w / 3;
        let kh = h / 2;
        let mut heat = vec![0u8; pn * (kh + 2)];
        let mut cxw = 0x12345678u32;
        let ily = pn / 8 + 1;
        let mut dax: Vec<u16> = (0..ily).map(|i| ((i * 37) % h) as u16).collect();
        let ncg: Vec<u8> = (0..ily).map(|i| ((i * 7 % 4) + 1) as u8).collect();

        let av = tw(4);
        for frame in 0..av {
            if ahm() { break; }
            uq(&mut buf);

            
            for x in 0..pn {
                cxw = xorshift(cxw);
                heat[(kh - 1) * pn + x] = (cxw & 0xFF) as u8;
                cxw = xorshift(cxw);
                heat[kh.saturating_sub(2) * pn + x] = ((cxw & 0xFF) as u16).min(255) as u8;
            }
            for y in 0..kh.saturating_sub(2) {
                for x in 0..pn {
                    let bev = heat[(y + 1) * pn + x] as u16;
                    let bl = if x > 0 { heat[(y + 1) * pn + x - 1] as u16 } else { bev };
                    let yi = if x + 1 < pn { heat[(y + 1) * pn + x + 1] as u16 } else { bev };
                    let mq = heat[((y + 2).min(kh - 1)) * pn + x] as u16;
                    let ns = (bev + bl + yi + mq) / 4;
                    heat[y * pn + x] = if ns > 2 { (ns - 2).min(255) as u8 } else { 0 };
                }
            }
            for axm in 0..kh { for x in 0..pn {
                let t = heat[axm * pn + x] as u32;
                let (r, g, b) = if t < 64 { (t * 4, 0u32, 0u32) }
                    else if t < 128 { (255, (t - 64) * 4, 0u32) }
                    else if t < 192 { (255, 255, (t - 128) * 4) }
                    else { (255u32, 255u32, 255u32) };
                let c = 0xFF000000 | (r.min(255) << 16) | (g.min(255) << 8) | b.min(255);
                let y1 = axm * 2;
                let y2 = y1 + 1;
                if x < w && y1 < h { buf[y1 * w + x] = c; }
                if x < w && y2 < h { buf[y2 * w + x] = c; }
            }}

            
            
            let t = frame as usize;
            for y in 0..h { for x in 0..pn {
                let p = pn + x;
                if p >= w { continue; }
                let v1 = (x ^ y).wrapping_add(t * 3) as u32;
                let v2 = ((x.wrapping_mul(3)) ^ (y.wrapping_mul(7))).wrapping_add(t * 5) as u32;
                let v3 = ((x + y + t * 2) ^ (x.wrapping_mul(y).wrapping_shr(4))) as u32;
                let r = (v1 & 0xFF).min(255);
                let g = ((v2 >> 1) & 0xFF).min(255);
                let b = ((v3 >> 2) & 0xFF).min(255);
                
                let ju = (r * 3 / 4 + g / 8).min(255);
                let axe = (g / 3 + b / 3).min(255);
                let iq = (b * 3 / 4 + r / 4).min(255);
                buf[y * w + p] = 0xFF000000 | (ju << 16) | (axe << 8) | iq;
            }}

            
            for y in 0..h { for x in pn*2..w {
                let idx = y * w + x;
                let g = ((buf[idx] >> 8) & 0xFF).saturating_sub(8);
                buf[idx] = 0xFF000000 | (g << 8);
            }}
            for ci in 0..dax.len() {
                let x = pn * 2 + ci * 8;
                if x >= w { continue; }
                dax[ci] = dax[ci].wrapping_add(ncg[ci] as u16);
                if dax[ci] as usize >= h { dax[ci] = 0; }
                let y = dax[ci] as usize;
                let c = (((frame as usize + ci * 13) % 94) + 33) as u8 as char;
                let du = crate::framebuffer::font::ol(c);
                for (row, &bits) in du.iter().enumerate() {
                    let o = y + row;
                    if o >= h { break; }
                    for bf in 0..8u32 {
                        if bits & (0x80 >> bf) != 0 {
                            let p = x + bf as usize;
                            if p < w { buf[o * w + p] = 0xFF00FF44; }
                        }
                    }
                }
            }

            
            for y in 0..h { if pn < w { buf[y * w + pn] = 0xFF333333; } if pn*2 < w { buf[y * w + pn*2] = 0xFF333333; } }
            draw_text_centered(&mut buf, w, h, 10, "VIDEO CODEC", 0xFFFFFFFF, 3);
            draw_text_at(&mut buf, w, h, pn / 2 - 20, h - 25, "FIRE", 0xFFFF8844, 1);
            draw_text_at(&mut buf, w, h, pn + pn / 2 - 28, h - 25, "PLASMA", 0xFFCC88FF, 1);
            draw_text_at(&mut buf, w, h, pn * 2 + pn / 2 - 28, h - 25, "MATRIX", 0xFF00FF44, 1);

            ev(&buf, w, h);
            crate::cpu::tsc::hq(BS_);
        }
    }
    bgl(&mut buf, w, h); 

    
    
    
    ckq(&mut buf, w, h,
        &[("In 1984,", 0xFFFF4444, 4),
          ("Big Brother watched you.", 0xFFFF6666, 3),
          ("", 0, 1),
          ("In 2026,", 0xFF44FF88, 4),
          ("you watch the code.", 0xFF00FFAA, 3)],
        6, |buf, w, h, f| bqq(buf, w, h, f));
    gvj(&mut buf, w, h); 

    
    
    
    
    {
        let bars: [(u32, &str, u32); 4] = [
            (50_000, "Windows", 0xFF4455AA),
            (30_000, "macOS",   0xFF888888),
            (28_000, "Linux",   0xFFDDAA33),
            (131,    "TrustOS", 0xFF00FF66),
        ];
        let daz = 50_000u32;
        let hs = 35;
        let jzq = h / 2 - (bars.len() * (hs + 15)) / 2;
        let av = tw(6);

        for frame in 0..av {
            if ahm() { break; }
            for aa in buf.iter_mut() { *aa = 0xFF080810; }

            draw_text_centered(&mut buf, w, h, 20, "LINES OF CODE", 0xFFCCCCCC, 3);

            for (i, &(size, name, color)) in bars.iter().enumerate() {
                let y = jzq + i * (hs + 15);
                let hfm = i as u32 * (av / 5);
                if frame < hfm { continue; }

                let progress = ((frame - hfm) * 8).min(100);
                let ctv = w * 3 / 4;
                let fv = (size as u64 * ctv as u64 / daz as u64) as usize;
                let cwc = fv * progress as usize / 100;

                draw_text_at(&mut buf, w, h, 20, y + 8, name, 0xFF888888, 2);
                let pv = 180;
                fill_rect(&mut buf, w, h, pv, y, cwc, hs, color);

                if progress > 50 {
                    let label = if size >= 1000 { alloc::format!("{}M", size / 1000) }
                        else { alloc::format!("{}K", size) };
                    draw_text_at(&mut buf, w, h, pv + cwc + 10, y + 8, &label, 0xFFCCCCCC, 2);
                }
            }

            
            if frame > av * 3 / 5 && frame < av * 3 / 5 + 10 {
                let dzg = (10 - (frame - av * 3 / 5)) as usize;
                if dzg > 0 && dzg < h {
                    buf.copy_within(0..(h - dzg) * w, dzg * w);
                    for y in 0..dzg { for x in 0..w { buf[y * w + x] = 0xFF080810; } }
                }
            }

            if frame > av * 4 / 5 {
                draw_text_centered(&mut buf, w, h, h - 50,
                    "Small enough to understand.", 0xFF88DDAA, 2);
            }

            ev(&buf, w, h);
            crate::cpu::tsc::hq(BS_);
        }
    }
    gca(&mut buf, w, h, 8); 

    
    
    
    ckq(&mut buf, w, h,
        &[("Your data. Your machine.", 0xFFFFCC44, 3),
          ("Your code.", 0xFFFFCC44, 3),
          ("", 0, 1),
          ("No backdoors. No telemetry.", 0xFF44FFAA, 2),
          ("No secrets.", 0xFF44FFAA, 2)],
        4, |buf, w, h, f| djf(buf, w, h, f));

    
    
    
    
    
    {
        let features: &[(&str, &str, u32, u32)] = &[
            
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

        let ae = features.len();

        
        for (i, &(title, sub, color, bg)) in features.iter().enumerate() {
            if ahm() { break; }

            
            for aa in buf.iter_mut() { *aa = bg; }

            
            let nw = w / 6;
            let qr = h / 8;
            let ul = w * 2 / 3;
            let afy = h * 3 / 8;
            
            fill_rect(&mut buf, w, h, nw + 4, qr + 4, ul, afy, 0xFF020202);
            
            fill_rect(&mut buf, w, h, nw, qr, ul, afy, 0xFF111118);
            
            let pdp = ((color >> 16) & 0xFF) / 4;
            let pdo = ((color >> 8) & 0xFF) / 4;
            let pdm = (color & 0xFF) / 4;
            let pdn = 0xFF000000 | (pdp << 16) | (pdo << 8) | pdm;
            fill_rect(&mut buf, w, h, nw, qr, ul, 22, pdn);
            
            draw_text_at(&mut buf, w, h, nw + 8, qr + 3, title, color, 1);
            
            fill_rect(&mut buf, w, h, nw + ul - 18, qr + 5, 12, 12, 0xFFFF4444);
            fill_rect(&mut buf, w, h, nw + ul - 34, qr + 5, 12, 12, 0xFF888844);
            fill_rect(&mut buf, w, h, nw + ul - 50, qr + 5, 12, 12, 0xFF448844);

            
            let cx = nw + 16;
            let u = qr + 30;
            let aq = ul - 32;
            match i {
                0 => { 
                    fill_rect(&mut buf, w, h, cx, u, aq / 2 - 4, afy / 2 - 20, 0xFF1A2A4A);
                    fill_rect(&mut buf, w, h, cx, u, aq / 2 - 4, 12, 0xFF3355AA);
                    fill_rect(&mut buf, w, h, cx + aq / 2 + 4, u + 20, aq / 2 - 4, afy / 2 - 40, 0xFF1A3A2A);
                    fill_rect(&mut buf, w, h, cx + aq / 2 + 4, u + 20, aq / 2 - 4, 12, 0xFF33AA55);
                    fill_rect(&mut buf, w, h, cx, u + afy - 60, aq, 16, 0xFF222233);
                }
                1 => { 
                    for line in 0..5u32 {
                        let y = u + 4 + line as usize * 18;
                        let iwx = ["> ls -la", "> cat readme.md", "> trust run app", "> netstat", "> _"];
                        if (line as usize) < iwx.len() {
                            draw_text_at(&mut buf, w, h, cx + 4, y, iwx[line as usize], 0xFF00CC44, 1);
                        }
                    }
                }
                2 => { 
                    fill_rect(&mut buf, w, h, cx + 4, u + 2, aq - 8, 14, 0xFF222233);
                    draw_text_at(&mut buf, w, h, cx + 8, u + 3, "https://trustos.dev", 0xFF4488FF, 1);
                    for line in 0..4u32 {
                        let y = u + 24 + line as usize * 14;
                        let mo = aq - 40 - ((line as usize * 30) % 80);
                        fill_rect(&mut buf, w, h, cx + 12, y, mo, 8, 0xFF333344);
                    }
                }
                5 => { 
                    let cg = cx + aq / 2;
                    let cr = u + 10;
                    let fq = (afy / 3).min(aq / 3);
                    
                    for t in 0..fq {
                        let ddb = cg + t - fq / 2;
                        let bvc = cr + fq;
                        if ddb < w && bvc < h { buf[bvc * w + ddb] = 0xFFFFCC00; }
                        let yt = t as f32 / fq as f32;
                        let ddc = cg - (fq as f32 / 2.0 * (1.0 - yt)) as usize + (fq as f32 * yt / 2.0) as usize;
                        let bvd = cr + fq - (fq as f32 * yt) as usize;
                        if ddc < w && bvd < h { buf[bvd * w + ddc.min(w - 1)] = 0xFFFFCC00; }
                    }
                }
                6 => { 
                    let cu = ((afy - 40) / 4).min(aq / 8);
                    let bx = cx + (aq - cu * 4) / 2;
                    let dc = u + 4;
                    for row in 0..4u32 {
                        for col in 0..4u32 {
                            let dark = (row + col) % 2 == 0;
                            let dr = if dark { 0xFF886633 } else { 0xFFDDCC99 };
                            fill_rect(&mut buf, w, h, bx + col as usize * cu, dc + row as usize * cu, cu, cu, dr);
                        }
                    }
                }
                8 => { 
                    let egb = 12;
                    let ek = (aq - 20) / egb;
                    for b in 0..egb {
                        let dua = afy - 50;
                        let ov = ((b * 7 + 13) % dua).max(10);
                        let bx = cx + 10 + b * ek;
                        let dc = u + afy - 50 - ov;
                        let g = (0x88 + b as u32 * 0x08).min(0xFF);
                        fill_rect(&mut buf, w, h, bx, dc, ek - 2, ov, 0xFF000000 | (g << 8) | 0x44);
                    }
                }
                _ => { 
                    for line in 0..5u32 {
                        let y = u + 6 + line as usize * 16;
                        let mo = aq - 24 - ((line as usize * 40 + i * 17) % 100);
                        let fse = ((color >> 16) & 0xFF) / 6;
                        let fsd = ((color >> 8) & 0xFF) / 6;
                        let fsc = (color & 0xFF) / 6;
                        fill_rect(&mut buf, w, h, cx + 12, y, mo, 8,
                            0xFF000000 | (fse << 16) | (fsd << 8) | fsc);
                    }
                }
            }

            
            let apg = qr + afy + 30;
            byy(&mut buf, w, h, apg, title, color, 4);
            draw_text_centered(&mut buf, w, h, apg + 50, sub, 0xFF888888, 2);

            
            let lhf = h - 35;
            let hte = ae * 10;
            let lhe = if hte < w { (w - hte) / 2 } else { 0 };
            for d in 0..ae {
                let aht = if d <= i { color } else { 0xFF333333 };
                fill_rect(&mut buf, w, h, lhe + d * 10, lhf, 6, 6, aht);
            }

            ctp(&mut buf, w, h, 180);
            ev(&buf, w, h);

            
            let delay = 400u64.saturating_sub(i as u64 * 320 / (ae as u64 - 1).max(1));
            crate::cpu::tsc::hq(delay);

            
            if i < ae - 1 {
                for aa in buf.iter_mut() { *aa = 0xFFFFFFFF; }
                ev(&buf, w, h);
                crate::cpu::tsc::hq(if delay > 200 { 33 } else { 16 });
            }
        }

        
        for &(title, _, color, bg) in features.iter() {
            if ahm() { break; }
            for aa in buf.iter_mut() { *aa = bg; }
            byy(&mut buf, w, h, h / 2 - 20, title, color, 4);
            ev(&buf, w, h);
            crate::cpu::tsc::hq(50);
        }

        
        for &(title, _, color, bg) in features.iter() {
            if ahm() { break; }
            for aa in buf.iter_mut() { *aa = bg; }
            draw_text_centered(&mut buf, w, h, h / 2, title, color, 5);
            ev(&buf, w, h);
            crate::cpu::tsc::hq(BS_);
        }
    }

    
    bgl(&mut buf, w, h);

    
    
    
    
    
    fba(&mut buf, w, h, 2);

    
    
    
    bgl(&mut buf, w, h);
    gca(&mut buf, w, h, 15); 

    
    
    
    
    {
        let av = tw(8);
        for frame in 0..av {
            if ahm() { break; }
            kbu(&mut buf, w, h, &mut kk, &la, frame);

            
            if frame > 8 {
                let cx = w / 2;
                let u = h / 2;
                for dq in 0..3u32 {
                    let r = ((frame - 8 - dq * 6) as usize).wrapping_mul(4);
                    if r > 0 && r < w {
                        for cc in 0..360 {
                            let vt = crate::formula3d::eu(cc as f32 * 0.01745);
                            let vg = crate::formula3d::hr(cc as f32 * 0.01745);
                            let p = (cx as f32 + vg * r as f32) as usize;
                            let o = (u as f32 + vt * r as f32 / 1.5) as usize;
                            if p < w && o < h {
                                let ln = 255u32.saturating_sub(r as u32);
                                buf[o * w + p] = 0xFF000000 | (ln / 4 << 16) | (ln << 8) | (ln / 3);
                            }
                        }
                    }
                }
            }

            
            if frame > tw(1) {
                draw_text_centered(&mut buf, w, h, h / 2 - 50,
                    "TRUST THE CODE.", 0xFF00FFAA, 5);
            }
            if frame > tw(3) {
                draw_text_centered(&mut buf, w, h, h / 2 + 30,
                    "github.com/nathan237/TrustOS", 0xFF00FF88, 2);
            }
            if frame > tw(5) {
                draw_text_centered(&mut buf, w, h, h / 2 + 70,
                    "Written in Rust. By one person.", 0xFF88CCAA, 2);
                draw_text_centered(&mut buf, w, h, h / 2 + 100,
                    "For everyone.", 0xFF88CCAA, 2);
            }

            ev(&buf, w, h);
            crate::cpu::tsc::hq(BS_);
        }
    }

    
    
    
    
    {
        let av = tw(4);
        for frame in 0..av {
            if ahm() { break; }
            hho(&mut buf, w, h, frame);

            let bx = w / 2 - 180;
            let dc = h / 2 - 40;
            fill_rect(&mut buf, w, h, bx, dc, 360, 80, 0xFF111111);
            for x in bx..bx+360 { if x < w { buf[dc * w + x] = 0xFF888888; buf[(dc+79) * w + x] = 0xFF888888; } }
            for y in dc..dc+80 { if y < h { buf[y * w + bx] = 0xFF888888; buf[y * w + bx+359] = 0xFF888888; } }

            draw_text_centered(&mut buf, w, h, dc + 10, "PLEASE STAND BY", 0xFFCCCCCC, 2);
            if (frame / 15) % 2 == 0 {
                draw_text_centered(&mut buf, w, h, dc + 45,
                    "TRUSTOS v0.3.3 -- LOADING...", 0xFF00FF88, 2);
            }

            ev(&buf, w, h);
            crate::cpu::tsc::hq(BS_);
        }
    }

    
    vi(&mut buf, w, h);

    
    
    
    uq(&mut buf);
    ev(&buf, w, h);
    if !pu {
        crate::framebuffer::pr(false);
    }
    crate::framebuffer::clear();
    crate::serial_println!("[TRAILER] TrustOS Trailer finished (145 beats)");
}
