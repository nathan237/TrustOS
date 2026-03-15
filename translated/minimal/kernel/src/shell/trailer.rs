





use alloc::vec;
use alloc::vec::Vec;

const BQ_: u64 = 33; 

use crate::draw_utils::qas as bpk;









const BL_: usize = 72;
const AY_: usize = 48;
const CES_: [u64; BL_] = [
    
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
fn djc(b: usize, c: usize) -> bool {
    if b >= AY_ || c >= BL_ { return false; }
    (CES_[c] >> (AY_ - 1 - b)) & 1 == 1
}


#[inline]
fn uic(b: usize, c: usize) -> bool {
    if !djc(b, c) { return false; }
    
    if b == 0 || !djc(b - 1, c) { return true; }
    if b >= AY_ - 1 || !djc(b + 1, c) { return true; }
    if c == 0 || !djc(b, c - 1) { return true; }
    if c >= BL_ - 1 || !djc(b, c + 1) { return true; }
    false
}




fn vwb(k: &mut [u32], nm: usize, adn: usize,
               cx: usize, ae: usize, bv: usize, frame: u32) {
    let okh = AY_ * bv;
    let okg = BL_ * bv;
    let mp = cx.ao(okh / 2);
    let qw = ae.ao(okg / 2);

    
    let ozf = frame as usize;

    for cq in 0..okg {
        let ct = cq / bv; 
        let x = qw + cq;
        if x >= adn { continue; }

        for cr in 0..okh {
            let mj = cr / bv; 
            let y = mp + cr;
            if y >= nm { continue; }

            if !djc(mj, ct) { continue; }

            let w = x * nm + y;

            if uic(mj, ct) {
                
                let buo = (ct as u32 * 80 / BL_ as u32) + 20;
                let ar = 140u32 + buo;
                
                let dx = if mj > AY_ / 2 { mj - AY_ / 2 } else { AY_ / 2 - mj };
                let tot = 30u32.ao(dx as u32 * 2);
                let p = (ar + tot).v(240);
                k[w] = 0xFF000000 | (p << 16) | (p << 8) | p;
            } else {
                
                
                let dpa = (mj.hx(7919) + 31) % 97;
                let ig = 1 + dpa % 3;
                let duv = (ct + ozf * ig + dpa * 5) % 17;
                
                let khd = (mj.hx(2654435761_usize.xvg(0))
                    .cn(ct.hx(40503))
                    .cn(ozf * ig)) % 37;

                
                let hms = duv;
                let hj = if hms < 2 {
                    200u32  
                } else if hms < 6 {
                    (120u32).ao(hms as u32 * 12)
                } else {
                    25u32 + (khd as u32 % 20)  
                };

                
                let dzm = cr % (bv * 4);  
                let bmg = cq % (bv * 6);
                let twx = khd < 20 && dzm > 0 && dzm < bv * 3
                    && bmg > 0 && bmg < bv * 5;

                if twx {
                    let at = hj.v(255);
                    let m = at / 8;
                    let o = at / 5;
                    k[w] = 0xFF000000 | (m << 16) | (at << 8) | o;
                } else {
                    
                    let at = 8u32 + (khd as u32 % 8);
                    k[w] = 0xFF000000 | (at << 8);
                }
            }
        }
    }
}





fn aej(k: &mut [u32], d: usize, i: usize,
                 cx: usize, ae: usize, r: char, s: u32, bv: usize) {
    let ka = crate::framebuffer::font::ada(r);
    for (br, &fs) in ka.iter().cf() {
        for ga in 0..8u32 {
            if fs & (0x80 >> ga) != 0 {
                for cq in 0..bv {
                    for cr in 0..bv {
                        let y = cx + ga as usize * bv + cr;
                        let x = ae + br * bv + cq;
                        if y < d && x < i { k[x * d + y] = s; }
                    }
                }
            }
        }
    }
}


fn nms(k: &mut [u32], d: usize, i: usize,
                      cx: usize, ae: usize, r: char, s: u32, bv: usize, ece: u32) {
    let ka = crate::framebuffer::font::ada(r);
    let btu = (s >> 16) & 0xFF;
    let bmh = (s >> 8) & 0xFF;
    let aiv = s & 0xFF;
    
    if ece > 0 {
        let eyw = ece as usize;
        for (br, &fs) in ka.iter().cf() {
            for ga in 0..8u32 {
                if fs & (0x80 >> ga) != 0 {
                    
                    let tam = cx + ga as usize * bv + bv / 2;
                    let tan = ae + br * bv + bv / 2;
                    
                    let gu = if eyw > 3 { 2 } else { 1 };
                    let mut bg = -(eyw as i32);
                    while bg <= eyw as i32 {
                        let mut dx = -(eyw as i32);
                        while dx <= eyw as i32 {
                            let us = (dx * dx + bg * bg) as u32;
                            let uv = ece * ece;
                            if us > 0 && us < uv {
                                let y = (tam as i32 + dx) as usize;
                                let x = (tan as i32 + bg) as usize;
                                if y < d && x < i {
                                    let ckj = 255u32.ao(us * 255 / uv) / 4;
                                    let w = x * d + y;
                                    let cs = k[w];
                                    let ahh = (cs >> 16) & 0xFF;
                                    let bgs = (cs >> 8) & 0xFF;
                                    let ng = cs & 0xFF;
                                    let nr = (ahh + btu * ckj / 255).v(255);
                                    let csu = (bgs + bmh * ckj / 255).v(255);
                                    let csq = (ng + aiv * ckj / 255).v(255);
                                    k[w] = 0xFF000000 | (nr << 16) | (csu << 8) | csq;
                                }
                            }
                            dx += gu;
                        }
                        bg += gu;
                    }
                }
            }
        }
    }
    
    for (br, &fs) in ka.iter().cf() {
        for ga in 0..8u32 {
            if fs & (0x80 >> ga) != 0 {
                for cq in 0..bv {
                    for cr in 0..bv {
                        let y = cx + ga as usize * bv + cr;
                        let x = ae + br * bv + cq;
                        if y < d && x < i { k[x * d + y] = s; }
                    }
                }
            }
        }
    }
}


fn eph(k: &mut [u32], d: usize, i: usize,
                  c: usize, text: &str, s: u32, bv: usize) {
    let qd = text.len() * 8 * bv;
    let cr = if qd < d { (d - qd) / 2 } else { 0 };
    
    let zc = bv.am(1);
    for (a, r) in text.bw().cf() {
        aej(k, d, i, cr + a * 8 * bv + zc, c + zc, r, 0xFF000000, bv);
    }
    
    let tq = (bv as u32 * 3).v(12);
    for (a, r) in text.bw().cf() {
        nms(k, d, i, cr + a * 8 * bv, c, r, s, bv, tq);
    }
}


fn gaf(k: &mut [u32], d: usize, i: usize, ccc: u32) {
    let cx = d / 2;
    let ae = i / 2;
    let ulw = (cx * cx + ae * ae) as u32;
    
    for c in (0..i).akt(2) {
        let bg = if c > ae { c - ae } else { ae - c };
        for b in (0..d).akt(2) {
            let dx = if b > cx { b - cx } else { cx - b };
            let us = (dx * dx + bg * bg) as u32;
            let pv = us * ccc / ulw;
            let tp = pv.v(200) as u32;
            
            for je in 0..2u32 {
                for bx in 0..2u32 {
                    let y = b + bx as usize;
                    let x = c + je as usize;
                    if y < d && x < i {
                        let w = x * d + y;
                        let r = k[w];
                        let m = ((r >> 16) & 0xFF).ao(tp);
                        let at = ((r >> 8) & 0xFF).ao(tp);
                        let o = (r & 0xFF).ao(tp);
                        k[w] = 0xFF000000 | (m << 16) | (at << 8) | o;
                    }
                }
            }
        }
    }
}


fn nnn(k: &mut [u32], d: usize, i: usize,
                    cx: usize, ae: usize, dy: usize,
                    qxr: u32, qxq: u32, qxp: u32, dw: u32) {
    let uv = (dy * dy) as u32;
    let bpl = ae.ao(dy);
    let dno = (ae + dy).v(i);
    let ihq = cx.ao(dy);
    let fza = (cx + dy).v(d);
    for c in bpl..dno {
        let bg = if c > ae { c - ae } else { ae - c };
        for b in ihq..fza {
            let dx = if b > cx { b - cx } else { cx - b };
            let us = (dx * dx + bg * bg) as u32;
            if us < uv {
                let ckj = dw * (uv - us) / uv;
                let ir = qxr * ckj / 255;
                let crl = qxq * ckj / 255;
                let gji = qxp * ckj / 255;
                let w = c * d + b;
                let cs = k[w];
                let ahh = (cs >> 16) & 0xFF;
                let bgs = (cs >> 8) & 0xFF;
                let ng = cs & 0xFF;
                k[w] = 0xFF000000
                    | ((ahh + ir).v(255) << 16)
                    | ((bgs + crl).v(255) << 8)
                    | (ng + gji).v(255);
            }
        }
    }
}


fn scx(k: &mut [u32], d: usize, i: usize,
                       cx: usize, ae: usize, kb: usize, ix: usize, s: u32) {
    let bpl = ae.ao(ix);
    let dno = (ae + ix).v(i);
    let ihq = cx.ao(kb);
    let fza = (cx + kb).v(d);
    let ftk = (kb * kb) as u64;
    let dbp = (ix * ix) as u64;
    for c in bpl..dno {
        let bg = if c > ae { c - ae } else { ae - c };
        for b in ihq..fza {
            let dx = if b > cx { b - cx } else { cx - b };
            if (dx as u64 * dx as u64) * dbp + (bg as u64 * bg as u64) * ftk < ftk * dbp {
                k[c * d + b] = s;
            }
        }
    }
}


fn qjq(k: &mut [u32], d: usize, i: usize) {
    let cx = d / 2;
    let ae = i / 2;
    for c in 0..i {
        
        let wdq = if c % 3 == 0 { 40u32 } else { 0 };
        
        let bg = if c > ae { c - ae } else { ae - c };
        let six = (bg * bg * 60 / (ae * ae).am(1)) as u32;
        for b in 0..d {
            let dx = if b > cx { b - cx } else { cx - b };
            let siw = (dx * dx * 60 / (cx * cx).am(1)) as u32;
            let jtp = wdq + six + siw;
            if jtp > 0 {
                let w = c * d + b;
                let r = k[w];
                let m = ((r >> 16) & 0xFF).ao(jtp);
                let at = ((r >> 8) & 0xFF).ao(jtp);
                let o = (r & 0xFF).ao(jtp);
                k[w] = 0xFF000000 | (m << 16) | (at << 8) | o;
            }
        }
    }
}

fn ri(k: &mut [u32], d: usize, i: usize,
                b: usize, c: usize, text: &str, s: u32, bv: usize) {
    for (a, r) in text.bw().cf() {
        aej(k, d, i, b + a * 8 * bv, c, r, s, bv);
    }
}

fn np(k: &mut [u32], d: usize, i: usize,
                      c: usize, text: &str, s: u32, bv: usize) {
    let qd = text.len() * 8 * bv;
    let cr = if qd < d { (d - qd) / 2 } else { 0 };
    ri(k, d, i, cr, c, text, s, bv);
}

fn ah(k: &mut [u32], d: usize, i: usize,
             kb: usize, ix: usize, yq: usize, aff: usize, s: u32) {
    for bg in 0..aff {
        for dx in 0..yq {
            let y = kb + dx;
            let x = ix + bg;
            if y < d && x < i { k[x * d + y] = s; }
        }
    }
}

fn aol(k: &mut [u32]) {
    
    #[cfg(target_arch = "x86_64")]
    unsafe {
        use core::arch::x86_64::*;
        let cot = els(0xFF000000u32 as i32);
        let ptr = k.mw() as *mut acb;
        let az = k.len() / 4;
        for a in 0..az {
            ccs(ptr.add(a), cot);
        }
        for a in (az * 4)..k.len() {
            k[a] = 0xFF000000;
        }
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        for ai in k.el() { *ai = 0xFF000000; }
    }
}

fn mb(k: &[u32], d: usize, i: usize) {
    
    crate::framebuffer::kdw(k.fq(), d, i);
}

fn apq(k: &mut [u32], d: usize, i: usize) {
    for _ in 0..40 {
        for y in k.el() {
            let m = ((*y >> 16) & 0xFF).ao(8);
            let at = ((*y >> 8) & 0xFF).ao(8);
            let o = (*y & 0xFF).ao(8);
            *y = 0xFF000000 | (m << 16) | (at << 8) | o;
        }
        mb(k, d, i);
        crate::cpu::tsc::rd(BQ_);
    }
    aol(k);
    mb(k, d, i);
    crate::cpu::tsc::rd(300);
}


fn hlq(k: &mut [u32], d: usize, i: usize) {
    let mut dv = 0xDEADBEEFu32;
    for _ in 0..3 {
        for y in k.el() {
            dv ^= dv << 13; dv ^= dv >> 17; dv ^= dv << 5;
            let p = dv & 0xFF;
            *y = 0xFF000000 | (p << 16) | (p << 8) | p;
        }
        mb(k, d, i);
        crate::cpu::tsc::rd(40);
    }
    aol(k);
    mb(k, d, i);
    crate::cpu::tsc::rd(80);
}


fn bmd() -> bool {
    if let Some(eh) = crate::keyboard::xw() {
        return eh == 0x1B || eh == b' ' || eh == b'\n' || eh == b'\r';
    }
    false
}






fn myx(k: &mut [u32], d: usize, i: usize, frame: u32) {
    let mut dv = frame.hx(2654435761);
    
    for y in k.el() {
        dv ^= dv << 13; dv ^= dv >> 17; dv ^= dv << 5;
        let bnq = (dv & 0x1F) as u32; 
        *y = 0xFF000000 | (bnq << 16) | (bnq << 8) | bnq;
    }
    
    for c in 0..i {
        if c % 3 == 0 {
            for b in 0..d {
                let w = c * d + b;
                let m = ((k[w] >> 16) & 0xFF) / 2;
                let at = ((k[w] >> 8) & 0xFF) / 2;
                let o = (k[w] & 0xFF) / 2;
                k[w] = 0xFF000000 | (m << 16) | (at << 8) | o;
            }
        }
    }
}


fn gaz(k: &mut [u32], d: usize, i: usize, frame: u32) {
    let jc = (frame as usize * 2) % i;
    for c in 0..i {
        let cq = (c + jc) % i;
        let mhv = (cq / 4) % 2 == 0;
        for b in 0..d {
            let bdm = if mhv { 35u32 } else { 15 };
            let ceq = if (cq % 60) < 2 { 30u32 } else { 0 };
            let m = (bdm + ceq).v(65);
            k[c * d + b] = 0xFF000000 | (m << 16) | 0x0205;
        }
    }
}


fn dyx(k: &mut [u32], d: usize, i: usize, frame: u32) {
    let ib = (frame % 160) as u32;
    let xg = if ib < 80 { ib / 2 } else { (160 - ib) / 2 };
    let fqp = ((frame + 40) % 120) as u32;
    let lwd = if fqp < 60 { fqp / 2 } else { (120 - fqp) / 2 };
    for c in 0..i {
        let dnr = (c as u32 * 40) / i as u32;
        for b in 0..d {
            let iht = (b as u32 * 10) / d as u32;
            let m = (dnr / 4 + lwd / 3).v(40);
            let at = (iht / 3).v(15);
            let o = (dnr + xg + iht / 2).v(80);
            k[c * d + b] = 0xFF000000 | (m << 16) | (at << 8) | o;
        }
    }
}


fn gbb(k: &mut [u32], d: usize, i: usize, frame: u32) {
    for y in k.el() {
        let at = ((*y >> 8) & 0xFF).ao(10);
        let m = ((*y >> 16) & 0xFF).ao(8);
        *y = 0xFF000000 | (m << 16) | (at << 8);
    }
    for a in 0..30u32 {
        let dv = (a.hx(2654435761).cn(frame.hx(37))) as usize;
        let y = (dv.hx(7919)) % d;
        let mal = (frame as usize + dv) % i;
        let x = i.ao(mal);
        let aaj = (50 + (dv % 50)) as u32;
        if y < d && x < i {
            k[x * d + y] = 0xFF000000 | (aaj / 4 << 16) | (aaj << 8) | (aaj / 3);
            if y + 1 < d { k[x * d + y + 1] = 0xFF000000 | (aaj << 8); }
        }
    }
}


fn hal(k: &mut [u32], d: usize, i: usize, frame: u32) {
    let fmy = frame.v(80);
    for c in 0..i {
        let dnr = c as u32 * 100 / i as u32;
        let fyn = if dnr > 50 { (dnr - 50).v(50) + fmy } else { fmy / 2 };
        let m = (fyn * 2).v(100);
        let at = (fyn * 3 / 4).v(50);
        let o = 20u32.ao(fyn / 3);
        for b in 0..d {
            k[c * d + b] = 0xFF000000 | (m << 16) | (at << 8) | o;
        }
    }
}


fn gaw(k: &mut [u32], d: usize, i: usize, frame: u32) {
    for ai in k.el() { *ai = 0xFF0A0A14; }
    let trace = 0xFF0F2818u32;
    for a in 0..16u32 {
        let ty = ((a.hx(7919) as usize) % i) & !3;
        let gx = ((a.hx(104729) as usize) % d) & !3;
        if ty < i { for b in 0..d { k[ty * d + b] = trace; } }
        if gx < d { for c in 0..i { k[c * d + gx] = trace; } }
    }
    let x = ((frame as usize * 3) % i) & !3;
    if x < i {
        let ars = (d / 4).v(120);
        let dav = (frame as usize * 5) % d;
        for dx in 0..ars {
            let y = (dav + dx) % d;
            k[x * d + y] = 0xFF00AA44;
            if x + 1 < i { k[(x + 1) * d + y] = 0xFF00AA44; }
        }
    }
}


fn qpj(k: &mut [u32], d: usize, i: usize,
           ec: &mut [u16], arz: &[u8], frame: u32) {
    
    for il in k.el() {
        let at = ((*il >> 8) & 0xFF) as u32;
        if at > 0 { *il = 0xFF000000 | (at.ao(6) << 8); }
        else { *il = 0xFF000000; }
    }
    for nc in 0..ec.len() {
        let b = nc * 8;
        if b >= d { continue; }
        ec[nc] = ec[nc].cn(arz[nc] as u16);
        if ec[nc] as usize >= i { ec[nc] = 0; }
        let c = ec[nc] as usize;
        let r = (((frame as usize + nc * 13) % 94) + 33) as u8 as char;
        let ka = crate::framebuffer::font::ada(r);
        for (br, &fs) in ka.iter().cf() {
            let x = c + br;
            if x >= i { break; }
            for ga in 0..8u32 {
                if fs & (0x80 >> ga) != 0 {
                    let y = b + ga as usize;
                    if y < d { k[x * d + y] = 0xFF00FF44; }
                }
            }
        }
    }
}


fn qpc(k: &mut [u32], d: usize, i: usize, frame: u32) {
    
    for y in k.el() {
        let m = ((*y >> 16) & 0xFF).ao(15);
        let at = ((*y >> 8) & 0xFF).ao(15);
        let o = (*y & 0xFF).ao(15);
        *y = 0xFF000000 | (m << 16) | (at << 8) | o;
    }
    
    for bj in 0..(d / 10) {
        let b = bj * 10 + 2;
        let dv = (bj.hx(7919).cn(frame as usize * 3)) % 97;
        let ig = 2 + dv % 4;
        let c = ((frame as usize * ig + bj * 23) % (i + 40)).nj(20);
        if c < i && b < d {
            let dpy = if (bj + frame as usize) % 2 == 0 { '0' } else { '1' };
            let ka = crate::framebuffer::font::ada(dpy);
            let aaj = 100u32 + (dv as u32 * 3) % 80;
            for (br, &fs) in ka.iter().cf() {
                let x = c + br;
                if x >= i { break; }
                for ga in 0..8u32 {
                    if fs & (0x80 >> ga) != 0 {
                        let y = b + ga as usize;
                        if y < d {
                            k[x * d + y] = 0xFF000000 | (aaj << 8) | (aaj / 4);
                        }
                    }
                }
            }
        }
    }
}



























































const BVK_: u32 = 14;


#[inline]
fn anb(bo: u32) -> u32 { bo * BVK_ }






fn mgd(k: &mut [u32], d: usize, i: usize) {
    aol(k);
    mb(k, d, i);
    crate::cpu::tsc::rd(100);
}



fn dhg(k: &mut [u32], d: usize, i: usize) {
    
    for ai in k.el() { *ai = 0xFFFFFFFF; }
    mb(k, d, i);
    crate::cpu::tsc::rd(33);
    
    for ai in k.el() { *ai = 0xFFB0B0B0; }
    mb(k, d, i);
    crate::cpu::tsc::rd(33);
    
    aol(k);
    mb(k, d, i);
    crate::cpu::tsc::rd(66);
}



fn ldp(k: &mut [u32], d: usize, i: usize, hj: usize) {
    let mut dv = 0xCAFEBABEu32;
    for a in 0..6u32 {
        let hfl = hj.ao(a as usize);
        if hfl == 0 { break; }
        dv = bpk(dv);
        let mp = (dv as usize % (hfl * 2 + 1)).nj(hfl);
        dv = bpk(dv);
        let qw = (dv as usize % (hfl * 2 + 1)).nj(hfl);
        
        let mut dvy = vec![0xFF000000u32; d * i];
        for c in 0..i {
            let cq = (c as isize + qw as isize).am(0) as usize;
            if cq >= i { continue; }
            for b in 0..d {
                let cr = (b as isize + mp as isize).am(0) as usize;
                if cr >= d { continue; }
                dvy[cq * d + cr] = k[c * d + b];
            }
        }
        mb(&dvy, d, i);
        crate::cpu::tsc::rd(33);
    }
}



fn xuh(k: &mut [u32], d: usize, i: usize) {
    for gu in 0..4u32 {
        let l = (gu + 1) as usize * d / 4;
        let mut dvy = vec![0xFF000000u32; d * i];
        for c in 0..i {
            for b in l..d {
                dvy[c * d + b - l] = k[c * d + b];
            }
        }
        
        for c in 0..i {
            let kss = d.ao(l);
            for b in kss..d {
                dvy[c * d + b] = 0xFF080808;
            }
        }
        mb(&dvy, d, i);
        crate::cpu::tsc::rd(25);
    }
    aol(k);
    mb(k, d, i);
    crate::cpu::tsc::rd(50);
}


fn jqj(k: &mut [u32], d: usize, i: usize, kcp: u32) {
    aol(k);
    for _ in 0..anb(kcp) {
        if bmd() { return; }
        mb(k, d, i);
        crate::cpu::tsc::rd(BQ_);
    }
}







fn xnt<G>(k: &mut [u32], d: usize, i: usize,
                 ak: &[(&str, u32, usize)],
                 hsc: u64, ukv: u32,
                 mut kcy: G)
where G: FnMut(&mut [u32], usize, usize, u32) {
    let aqo: usize = ak.iter().map(|(ab, _, _)| ab.len()).sum();
    let avv = (hsc / BQ_).am(1) as u32;
    let gvh = aqo as u32 * avv;
    let tpl = anb(ukv).ao(gvh);
    let es = gvh + tpl;

    for frame in 0..es {
        if bmd() { return; }

        kcy(k, d, i, frame);

        let qo = (frame / avv) as usize;
        let aku: usize = ak.iter().map(|(_, _, e)| 16 * e + 12).sum();
        let mut c = if aku < i { (i - aku) / 2 } else { 20 };
        let mut bbh = 0usize;

        for &(text, s, bv) in ak {
            let qd = text.len() * 8 * bv;
            let cr = if qd < d { (d - qd) / 2 } else { 0 };
            for (a, r) in text.bw().cf() {
                if bbh + a >= qo { break; }
                aej(k, d, i, cr + a * 8 * bv, c, r, s, bv);
            }
            
            if qo > bbh && qo < bbh + text.len() {
                let nc = qo - bbh;
                let cx = cr + nc * 8 * bv;
                if (frame / 8) % 2 == 0 {
                    for ae in c..c + 16 * bv {
                        if ae < i && cx + 2 < d {
                            k[ae * d + cx] = 0xFFFFFFFF;
                            k[ae * d + cx + 1] = 0xFFFFFFFF;
                        }
                    }
                }
            }
            bbh += text.len();
            c += 16 * bv + 12;
        }

        mb(k, d, i);
        crate::cpu::tsc::rd(BQ_);
    }
}


fn fkr<G>(k: &mut [u32], d: usize, i: usize,
                 ak: &[(&str, u32, usize)], kcp: u32,
                 mut kcy: G)
where G: FnMut(&mut [u32], usize, usize, u32) {
    for frame in 0..anb(kcp) {
        if bmd() { return; }
        kcy(k, d, i, frame);

        let aku: usize = ak.iter().map(|(_, _, e)| 16 * e + 12).sum();
        let mut c = if aku < i { (i - aku) / 2 } else { 20 };
        for &(text, s, bv) in ak {
            np(k, d, i, c, text, s, bv);
            c += 16 * bv + 12;
        }

        mb(k, d, i);
        crate::cpu::tsc::rd(BQ_);
    }
}








































pub(super) fn rjs() {
    let (kp, kl) = crate::framebuffer::yn();
    let d = kp as usize;
    let i = kl as usize;

    
    let afk = crate::framebuffer::bre();
    if !afk {
        crate::framebuffer::beo();
        crate::framebuffer::afi(true);
    }

    let mut k = vec![0xFF000000u32; d * i];

    
    let aur = d / 8 + 1;
    let mut ws: Vec<u16> = (0..aur).map(|a| ((a * 37 + 13) % i) as u16).collect();
    let yg: Vec<u8> = (0..aur).map(|a| (((a * 7 + 3) % 4) + 1) as u8).collect();

    crate::serial_println!("[TRAILER] TrustOS Trailer started (128 BPM beat-synced)");

    
    
    

    
    
    
    
    {
        let bv = if i > 600 { 6 } else { 4 };
        let dsw = i / 2 - 30;
        let sl = dsw + (BL_ * bv) / 2 + 20;
        let es = anb(16);

        for frame in 0..es {
            if bmd() { break; }
            aol(&mut k);

            
            let erj = (frame * 3).v(120);
            let tgb = 80 + (frame as usize * 2).v(i / 3);
            nnn(&mut k, d, i, d / 2, dsw, tgb,
                             20, 80, 40, erj);

            vwb(&mut k, d, i, d / 2, dsw, bv, frame);

            
            if frame > 20 {
                let xg = ((frame % 40) as i32 - 20).eki() as u32;
                let vzd = (AY_ * bv / 2 + 10 + xg as usize) as f32;
                let mah = 60u32.ao(xg);
                for q in 0..180 {
                    let hg = q as f32 * 0.0349; 
                    let aql = crate::formula3d::lz(hg);
                    let apn = crate::formula3d::rk(hg);
                    for ahw in 0..2 {
                        let m = vzd + ahw as f32;
                        let y = (d as f32 / 2.0 + apn * m) as usize;
                        let x = (dsw as f32 + aql * m * 0.75) as usize;
                        if y < d && x < i {
                            let w = x * d + y;
                            let at = ((k[w] >> 8) & 0xFF) + mah;
                            k[w] = 0xFF000000 | ((mah / 4) << 16) | (at.v(255) << 8) | (mah / 3);
                        }
                    }
                }
            }

            
            if frame < 60 {
                let tp = ((60 - frame) as u32 * 255 / 60) as u32;
                for y in k.el() {
                    if *y != 0xFF000000 {
                        let m = ((*y >> 16) & 0xFF).ao(tp);
                        let at = ((*y >> 8) & 0xFF).ao(tp);
                        let o = (*y & 0xFF).ao(tp);
                        *y = 0xFF000000 | (m << 16) | (at << 8) | o;
                    }
                }
            }

            
            gaf(&mut k, d, i, 180);

            
            if frame > anb(8) {
                let text = "TRUSTOS";
                let ezq = if i > 600 { 5 } else { 3 };
                let sub = frame - anb(8);
                let qo = (sub / 8).v(text.len() as u32) as usize;
                let qd = text.len() * 8 * ezq;
                let gx = if qd < d { (d - qd) / 2 } else { 0 };
                
                for (a, r) in text.bw().cf() {
                    if a >= qo { break; }
                    aej(&mut k, d, i,
                        gx + a * 8 * ezq + 2, sl + 2, r, 0xFF000000, ezq);
                }
                
                for (a, r) in text.bw().cf() {
                    if a >= qo { break; }
                    nms(&mut k, d, i,
                        gx + a * 8 * ezq, sl, r, 0xFFDDDDDD, ezq,
                        ezq as u32 * 3);
                }
            }

            mb(&k, d, i);
            crate::cpu::tsc::rd(BQ_);
        }
    }
    dhg(&mut k, d, i); 

    
    
    
    
    {
        let es = anb(8);
        for frame in 0..es {
            if bmd() { break; }
            myx(&mut k, d, i, frame);

            
            let gbw = if d > 800 { 200 } else { 160 };
            let han = 50;
            let bx = d / 2 - gbw;
            let je = i / 2 - han;
            
            ah(&mut k, d, i, bx + 3, je + 3, gbw * 2, han * 2, 0xFF050505);
            
            ah(&mut k, d, i, bx, je, gbw * 2, han * 2, 0xFF111111);
            
            for b in bx..bx + gbw * 2 {
                if b < d {
                    let buo = 0xFF555555 + ((b - bx) as u32 * 0x40 / (gbw as u32 * 2));
                    let drc = 0xFF000000 | (buo & 0xFF) << 16 | (buo & 0xFF) << 8 | (buo & 0xFF);
                    k[je * d + b] = drc;
                    k[(je + han * 2 - 1) * d + b] = drc;
                }
            }
            for c in je..je + han * 2 {
                if c < i {
                    k[c * d + bx] = 0xFF888888;
                    k[c * d + (bx + gbw * 2 - 1).v(d - 1)] = 0xFF888888;
                }
            }

            let xts = if (frame / 15) % 2 == 0 { 0xFFFFCC00 } else { 0xFFFF8800 };
            eph(&mut k, d, i, je + 12, "! WARNING !", xts, 2);
            eph(&mut k, d, i, je + 55, "PLEASE STAND BY", 0xFFCCCCCC, 2);

            
            qjq(&mut k, d, i);
            gaf(&mut k, d, i, 200);

            mb(&k, d, i);
            crate::cpu::tsc::rd(BQ_);
        }
    }
    hlq(&mut k, d, i); 

    
    
    
    
    {
        let his = d / 2;
        let ggo = i / 2 - 40;
        let kuu = d / 4;
        let hit = i / 8;
        let es = anb(12);

        for frame in 0..es {
            if bmd() { break; }
            dyx(&mut k, d, i, frame);

            
            let nze = (frame as u32 * 3).v(180);
            nnn(&mut k, d, i, his, ggo, kuu + 40, nze, 0x10, 0x00, nze);

            
            scx(&mut k, d, i, his, ggo, kuu, hit, 0xFFDDCCCC);

            
            let gkd = hit * 2 / 3;
            let ewz = hit / 3;
            for bg in 0..gkd*2+2 {
                for dx in 0..gkd*2+2 {
                    let ym = dx as i32 - gkd as i32;
                    let wl = bg as i32 - gkd as i32;
                    let us = ym*ym + wl*wl;
                    let ofn = (gkd as i32) * (gkd as i32);
                    let lvb = (ewz as i32) * (ewz as i32);
                    if us <= ofn && us >= lvb {
                        let y = (his as i32 + ym) as usize;
                        let x = (ggo as i32 + wl) as usize;
                        if y < d && x < i {
                            
                            let ab = (us - lvb) as u32 * 255 / (ofn - lvb).am(1) as u32;
                            let m = 0xFF;
                            let at = (0x66u32).ao(ab * 0x66 / 255) as u8;
                            k[x * d + y] = 0xFF000000 | (m as u32) << 16 | (at as u32) << 8;
                        }
                    }
                }
            }

            
            for bg in 0..ewz*2 {
                for dx in 0..ewz*2 {
                    let ym = dx as i32 - ewz as i32;
                    let wl = bg as i32 - ewz as i32;
                    if ym*ym + wl*wl < (ewz as i32 * ewz as i32) {
                        let y = (his as i32 + ym) as usize;
                        let x = (ggo as i32 + wl) as usize;
                        if y < d && x < i { k[x * d + y] = 0xFF080000; }
                    }
                }
            }

            
            for psx in 0..2i32 {
                let snw = kuu as i32 + psx;
                let sji = hit as i32 + psx;
                for hg in 0..720 {
                    let q = hg as f32 * 0.008727; 
                    let aql = crate::formula3d::lz(q);
                    let apn = crate::formula3d::rk(q);
                    let y = (his as f32 + apn * snw as f32) as usize;
                    let x = (ggo as f32 + aql * sji as f32) as usize;
                    if y < d && x < i {
                        k[x * d + y] = 0xFFFF3333;
                    }
                }
            }

            
            let bw = (frame / 3).v(28) as usize;
            let szb = "BIG BROTHER IS WATCHING YOU.";
            let wnr: alloc::string::String = szb.bw().take(bw).collect();
            eph(&mut k, d, i, ggo + hit + 50, &wnr, 0xFFFF4444, 3);

            gaf(&mut k, d, i, 160);

            mb(&k, d, i);
            crate::cpu::tsc::rd(BQ_);
        }
    }
    mgd(&mut k, d, i); 

    
    
    
    
    {
        let ufd = [
            "location: TRACKED",
            "camera: ACTIVE",
            "keystrokes: LOGGED",
            "microphone: RECORDING",
            "contacts: UPLOADED",
            "messages: SCANNED",
            "browsing: PROFILED",
            "identity: SOLD",
        ];
        let es = anb(10);
        for frame in 0..es {
            if bmd() { break; }
            gaz(&mut k, d, i, frame);

            
            let xg = (crate::formula3d::lz(frame as f32 * 0.15) * 40.0 + 60.0) as u8;
            let emn = 0xFF000000 | (xg as u32) << 16;
            ah(&mut k, d, i, 0, 0, d, 6, emn);
            ah(&mut k, d, i, 0, i.ao(6), d, 6, emn);

            let ufe = (frame / 12).v(8) as usize;
            for (a, &line) in ufd.iter().cf().take(ufe) {
                let c = 80 + a * 50;
                let wpo = ((frame as usize).ao(a * 12)).v(d);
                let b = d.ao(wpo);
                
                ri(&mut k, d, i, b + 2, c + 2, line, 0xFF220000, 2);
                
                let dfa = line.du(':').unwrap_or(line.len());
                let cu = &line[..dfa];
                let bn = &line[dfa..];
                let caj = cu.len() * 16;
                ri(&mut k, d, i, b, c, cu, 0xFFAA3333, 2);
                ri(&mut k, d, i, b + caj, c, bn, 0xFFFF4444, 2);
            }

            if frame > anb(6) {
                eph(&mut k, d, i, i - 60,
                    "Every keystroke. Every click.", 0xFFFF6666, 2);
            }
            if frame > anb(8) {
                eph(&mut k, d, i, i - 30,
                    "Every file. Every thought.", 0xFFFF4444, 2);
            }

            gaf(&mut k, d, i, 180);

            mb(&k, d, i);
            crate::cpu::tsc::rd(BQ_);
        }
    }
    xuh(&mut k, d, i); 

    
    
    
    
    xnt(&mut k, d, i,
        &[("Your OS has", 0xFFAAFFAA, 2),
          ("50,000,000 lines of code.", 0xFF00FF88, 3),
          ("", 0, 1),
          ("You can read ZERO of them.", 0xFFFF4444, 3)],
        40, 8, |k, d, i, bb| qpc(k, d, i, bb));
    mgd(&mut k, d, i); 

    
    
    
    
    {
        let hgm = [
            "Kernel source code ............",
            "Driver implementations ........",
            "Encryption keys ...............",
            "Telemetry endpoints ...........",
            "Backdoor protocols ............",
            "Data collection routines ......",
        ];
        let es = anb(8);
        for frame in 0..es {
            if bmd() { break; }
            
            for c in 0..i { for b in 0..d {
                let bdm: u32 = 0x0C + (c as u32 * 4 / i as u32);
                let bji: u32 = 0x14 + (c as u32 * 6 / i as u32);
                let cdd: u32 = 0x28 + (c as u32 * 8 / i as u32);
                let bbr = if (b % 20 < 1) || (c % 20 < 1) { 0x06u32 } else { 0u32 };
                let m = (bdm + bbr).v(255);
                let at = (bji + bbr).v(255);
                let o = (cdd + bbr).v(255);
                k[c * d + b] = 0xFF000000 | (m << 16) | (at << 8) | o;
            }}

            let dec = i / 2 - hgm.len() * 22;
            for (a, &line) in hgm.iter().cf() {
                let c = dec + a * 44;
                let qd = line.len() * 16;
                let gx = if qd < d { (d - qd) / 2 } else { 0 };
                
                ri(&mut k, d, i, gx + 1, c + 1, line, 0xFF223344, 2);
                ri(&mut k, d, i, gx, c, line, 0xFF7799BB, 2);

                let pax = 10 + a as u32 * 12;
                if frame > pax {
                    let ikr = ((frame - pax) as usize * 30).v(qd);
                    
                    ah(&mut k, d, i, gx, c, ikr, 30, 0xFF0A0A0A);
                    if ikr > 2 {
                        
                        ah(&mut k, d, i, gx, c, ikr, 1, 0xFF222222);
                    }
                    if ikr >= qd {
                        
                        let mhh = gx + qd / 2 - 64;
                        let mhi = c + 4;
                        
                        for ub in mhi.ao(4)..mhi + 30 {
                            for qz in mhh.ao(8)..mhh + 140 {
                                if qz < d && ub < i {
                                    let aft = k[ub * d + qz];
                                    let efx = ((aft >> 16) & 0xFF) as u32;
                                    let nr = (efx + 30).v(255);
                                    k[ub * d + qz] = (aft & 0xFF00FFFF) | (nr << 16);
                                }
                            }
                        }
                        ri(&mut k, d, i, mhh, mhi,
                            "[REDACTED]", 0xFFFF2222, 2);
                    }
                }
            }

            if frame > anb(6) {
                eph(&mut k, d, i, i - 40,
                    "You trust what you cannot see.", 0xFF8888CC, 2);
            }

            gaf(&mut k, d, i, 200);

            mb(&k, d, i);
            crate::cpu::tsc::rd(BQ_);
        }
    }
    dhg(&mut k, d, i); 

    
    
    
    
    
    jqj(&mut k, d, i, 4); 

    
    
    
    dhg(&mut k, d, i);
    ldp(&mut k, d, i, 12);

    
    
    
    
    {
        let es = anb(12);
        for frame in 0..es {
            if bmd() { break; }
            gbb(&mut k, d, i, frame);

            
            if frame > 5 && frame < anb(4) {
                let hj = if frame < anb(2) { frame - 5 } else { anb(4) - frame };
                let cx = d / 2;
                let ae = i / 2 - 20;
                for acj in 0..16 {
                    let hg = acj as f32 * 0.3927;
                    let aql = crate::formula3d::lz(hg);
                    let apn = crate::formula3d::rk(hg);
                    let len = hj as f32 * 8.0;
                    for ab in 0..len as usize {
                        let y = (cx as f32 + apn * ab as f32) as usize;
                        let x = (ae as f32 + aql * ab as f32) as usize;
                        if y < d && x < i {
                            let aaj = (200 - ab * 3).am(40) as u32;
                            k[x * d + y] = 0xFF000000 | (aaj / 4 << 16) | (aaj << 8) | (aaj / 2);
                        }
                    }
                }
            }

            
            if frame > 8 {
                let dmn = if i > 600 { 7 } else { 5 };
                let dw = ((frame - 8) * 12).v(255) as u32;
                let s = 0xFF000000 | (dw / 3 << 16) | (dw << 8) | (dw / 2);
                np(&mut k, d, i, i / 2 - 40, "TRUSTOS", s, dmn);

                if frame > anb(3) {
                    np(&mut k, d, i, i / 2 + 40,
                        "The OS you can read. All of it.", 0xFF88DDAA, 2);
                }
            }

            mb(&k, d, i);
            crate::cpu::tsc::rd(BQ_);
        }
    }
    dhg(&mut k, d, i); 

    
    
    
    
    {
        let rpi: [(u32, &str); 4] = [
            (131662, " lines of Rust"),
            (1, " author"),
            (0, " secrets"),
            (100, "% open source"),
        ];
        let es = anb(8);
        for frame in 0..es {
            if bmd() { break; }
            gaw(&mut k, d, i, frame);

            let ohe = (frame / anb(2)).v(4) as usize;
            let dec = i / 2 - ohe * 35;

            for (a, &(cd, cu)) in rpi.iter().cf().take(ohe) {
                let c = dec + a * 70;
                let wvm = frame.ao(a as u32 * anb(2));
                let li = (wvm * 6).v(anb(2));
                let cv = if cd == 0 { 0 }
                    else { (cd as u64 * li as u64 / anb(2) as u64) as u32 };

                let ajh = alloc::format!("{:>7}", cv);
                let sza = alloc::format!("{}{}", ajh, cu);

                let bv = 3;
                let qd = sza.len() * 8 * bv;
                let gx = if qd < d { (d - qd) / 2 } else { 0 };

                for (nc, r) in ajh.bw().cf() {
                    aej(&mut k, d, i, gx + nc * 8 * bv, c, r, 0xFF00FF88, bv);
                }
                for (nc, r) in cu.bw().cf() {
                    aej(&mut k, d, i, gx + (ajh.len() + nc) * 8 * bv, c, r, 0xFF44AA66, bv);
                }
            }

            mb(&k, d, i);
            crate::cpu::tsc::rd(BQ_);
        }
    }
    dhg(&mut k, d, i); 

    
    
    
    

    
    {
        let blh = ["hello", "a7 f3 0b 2e c1", "[HDR|DATA|CRC]", ">>> WIRE >>>"];
        let wse = [0xFF00FF88u32, 0xFF00CCFF, 0xFFFFAA00, 0xFF44FF44];
        let es = anb(4);

        for frame in 0..es {
            if bmd() { break; }
            gaw(&mut k, d, i, frame);

            let rsc = (frame * 4 / es).v(3) as usize;
            np(&mut k, d, i, 30, "NETWORK STACK", 0xFF00CCFF, 3);

            let uq = i / 2 - 80;
            for a in 0..=rsc {
                let c = uq + a * 50;
                let bx = d / 2 - 140;
                ah(&mut k, d, i, bx, c, 280, 35, 0xFF111122);
                np(&mut k, d, i, c + 4, blh[a], wse[a], 2);
            }

            mb(&k, d, i);
            crate::cpu::tsc::rd(BQ_);
        }
    }
    hlq(&mut k, d, i); 

    
    fkr(&mut k, d, i,
        &[("TLS 1.3", 0xFF00CCFF, 5),
          ("Full handshake. Real crypto.", 0xFF88BBDD, 2)],
        3, |k, d, i, bb| dyx(k, d, i, bb));
    dhg(&mut k, d, i); 

    
    fkr(&mut k, d, i,
        &[("GUI COMPOSITOR", 0xFFFFCC00, 4),
          ("Windows. Taskbar. Wallpaper.", 0xFFCCAA88, 2)],
        3, |k, d, i, bb| dyx(k, d, i, bb));
    hlq(&mut k, d, i); 

    
    fkr(&mut k, d, i,
        &[("TRUSTLANG", 0xFF00FF88, 5),
          ("Lexer > Parser > VM.", 0xFF88DDAA, 2)],
        3, |k, d, i, bb| gbb(k, d, i, bb));
    dhg(&mut k, d, i); 

    
    fkr(&mut k, d, i,
        &[("TRUSTFS", 0xFFFFAA00, 5),
          ("Journaled. Persistent.", 0xFFDDAA66, 2)],
        2, |k, d, i, bb| gaw(k, d, i, bb));
    hlq(&mut k, d, i); 

    
    fkr(&mut k, d, i,
        &[("WEB BROWSER", 0xFF4488FF, 4),
          ("HTML + CSS + HTTPS.", 0xFF88AADD, 2)],
        2, |k, d, i, bb| dyx(k, d, i, bb));
    dhg(&mut k, d, i); 

    
    {
        let unr = [
            (crate::formula3d::foe(), "PENGER", 0xFF00FF88u32),
            (crate::formula3d::czw(0.5, 0.2, 16, 12), "3D TORUS", 0xFF00CCFFu32),
            (crate::formula3d::onb(), "TRUSTOS 3D", 0xFFFFCC00u32),
        ];

        for (si, (mesh, cu, s)) in unr.iter().cf() {
            let unj = anb(4) / 3; 
            for frame in 0..unj {
                if bmd() { break; }
                aol(&mut k);

                let aev = frame as f32 * 0.08 + si as f32 * 2.0;
                
                crate::formula3d::vxb(
                    &mut k, d, i, &mesh, aev, 0.3, 3.0, *s
                );

                np(&mut k, d, i, 15, "3D ENGINE", 0xFFFFFFFF, 3);
                np(&mut k, d, i, i - 35, cu, *s, 2);

                mb(&k, d, i);
                crate::cpu::tsc::rd(BQ_);
            }
            
            if si < 2 {
                for ai in k.el() { *ai = 0xFFFFFFFF; }
                mb(&k, d, i);
                crate::cpu::tsc::rd(33);
            }
        }
    }
    hlq(&mut k, d, i); 

    
    
    {
        let aes = d / 3;
        let wp = i / 2;
        let mut xc = vec![0u8; aes * (wp + 2)];
        let mut ghd = 0x12345678u32;
        let olf = aes / 8 + 1;
        let mut gmd: Vec<u16> = (0..olf).map(|a| ((a * 37) % i) as u16).collect();
        let ukh: Vec<u8> = (0..olf).map(|a| ((a * 7 % 4) + 1) as u8).collect();

        let es = anb(4);
        for frame in 0..es {
            if bmd() { break; }
            aol(&mut k);

            
            for b in 0..aes {
                ghd = bpk(ghd);
                xc[(wp - 1) * aes + b] = (ghd & 0xFF) as u8;
                ghd = bpk(ghd);
                xc[wp.ao(2) * aes + b] = ((ghd & 0xFF) as u16).v(255) as u8;
            }
            for c in 0..wp.ao(2) {
                for b in 0..aes {
                    let def = xc[(c + 1) * aes + b] as u16;
                    let bl = if b > 0 { xc[(c + 1) * aes + b - 1] as u16 } else { def };
                    let avi = if b + 1 < aes { xc[(c + 1) * aes + b + 1] as u16 } else { def };
                    let aaa = xc[((c + 2).v(wp - 1)) * aes + b] as u16;
                    let abl = (def + bl + avi + aaa) / 4;
                    xc[c * aes + b] = if abl > 2 { (abl - 2).v(255) as u8 } else { 0 };
                }
            }
            for crj in 0..wp { for b in 0..aes {
                let ab = xc[crj * aes + b] as u32;
                let (m, at, o) = if ab < 64 { (ab * 4, 0u32, 0u32) }
                    else if ab < 128 { (255, (ab - 64) * 4, 0u32) }
                    else if ab < 192 { (255, 255, (ab - 128) * 4) }
                    else { (255u32, 255u32, 255u32) };
                let r = 0xFF000000 | (m.v(255) << 16) | (at.v(255) << 8) | o.v(255);
                let dp = crj * 2;
                let jz = dp + 1;
                if b < d && dp < i { k[dp * d + b] = r; }
                if b < d && jz < i { k[jz * d + b] = r; }
            }}

            
            
            let ab = frame as usize;
            for c in 0..i { for b in 0..aes {
                let y = aes + b;
                if y >= d { continue; }
                let agy = (b ^ c).cn(ab * 3) as u32;
                let apg = ((b.hx(3)) ^ (c.hx(7))).cn(ab * 5) as u32;
                let bdf = ((b + c + ab * 2) ^ (b.hx(c).xvg(4))) as u32;
                let m = (agy & 0xFF).v(255);
                let at = ((apg >> 1) & 0xFF).v(255);
                let o = ((bdf >> 2) & 0xFF).v(255);
                
                let uv = (m * 3 / 4 + at / 8).v(255);
                let cqu = (at / 3 + o / 3).v(255);
                let tb = (o * 3 / 4 + m / 4).v(255);
                k[c * d + y] = 0xFF000000 | (uv << 16) | (cqu << 8) | tb;
            }}

            
            for c in 0..i { for b in aes*2..d {
                let w = c * d + b;
                let at = ((k[w] >> 8) & 0xFF).ao(8);
                k[w] = 0xFF000000 | (at << 8);
            }}
            for nc in 0..gmd.len() {
                let b = aes * 2 + nc * 8;
                if b >= d { continue; }
                gmd[nc] = gmd[nc].cn(ukh[nc] as u16);
                if gmd[nc] as usize >= i { gmd[nc] = 0; }
                let c = gmd[nc] as usize;
                let r = (((frame as usize + nc * 13) % 94) + 33) as u8 as char;
                let ka = crate::framebuffer::font::ada(r);
                for (br, &fs) in ka.iter().cf() {
                    let x = c + br;
                    if x >= i { break; }
                    for ga in 0..8u32 {
                        if fs & (0x80 >> ga) != 0 {
                            let y = b + ga as usize;
                            if y < d { k[x * d + y] = 0xFF00FF44; }
                        }
                    }
                }
            }

            
            for c in 0..i { if aes < d { k[c * d + aes] = 0xFF333333; } if aes*2 < d { k[c * d + aes*2] = 0xFF333333; } }
            np(&mut k, d, i, 10, "VIDEO CODEC", 0xFFFFFFFF, 3);
            ri(&mut k, d, i, aes / 2 - 20, i - 25, "FIRE", 0xFFFF8844, 1);
            ri(&mut k, d, i, aes + aes / 2 - 28, i - 25, "PLASMA", 0xFFCC88FF, 1);
            ri(&mut k, d, i, aes * 2 + aes / 2 - 28, i - 25, "MATRIX", 0xFF00FF44, 1);

            mb(&k, d, i);
            crate::cpu::tsc::rd(BQ_);
        }
    }
    dhg(&mut k, d, i); 

    
    
    
    fkr(&mut k, d, i,
        &[("In 1984,", 0xFFFF4444, 4),
          ("Big Brother watched you.", 0xFFFF6666, 3),
          ("", 0, 1),
          ("In 2026,", 0xFF44FF88, 4),
          ("you watch the code.", 0xFF00FFAA, 3)],
        6, |k, d, i, bb| dyx(k, d, i, bb));
    mgd(&mut k, d, i); 

    
    
    
    
    {
        let cjf: [(u32, &str, u32); 4] = [
            (50_000, "Windows", 0xFF4455AA),
            (30_000, "macOS",   0xFF888888),
            (28_000, "Linux",   0xFFDDAA33),
            (131,    "TrustOS", 0xFF00FF66),
        ];
        let uku = 50_000u32;
        let tn = 35;
        let qmz = i / 2 - (cjf.len() * (tn + 15)) / 2;
        let es = anb(6);

        for frame in 0..es {
            if bmd() { break; }
            for ai in k.el() { *ai = 0xFF080810; }

            np(&mut k, d, i, 20, "LINES OF CODE", 0xFFCCCCCC, 3);

            for (a, &(aw, j, s)) in cjf.iter().cf() {
                let c = qmz + a * (tn + 15);
                let mvz = a as u32 * (es / 5);
                if frame < mvz { continue; }

                let li = ((frame - mvz) * 8).v(100);
                let gar = d * 3 / 4;
                let nm = (aw as u64 * gar as u64 / uku as u64) as usize;
                let geb = nm * li as usize / 100;

                ri(&mut k, d, i, 20, c + 8, j, 0xFF888888, 2);
                let ajx = 180;
                ah(&mut k, d, i, ajx, c, geb, tn, s);

                if li > 50 {
                    let cu = if aw >= 1000 { alloc::format!("{}M", aw / 1000) }
                        else { alloc::format!("{}K", aw) };
                    ri(&mut k, d, i, ajx + geb + 10, c + 8, &cu, 0xFFCCCCCC, 2);
                }
            }

            
            if frame > es * 3 / 5 && frame < es * 3 / 5 + 10 {
                let iad = (10 - (frame - es * 3 / 5)) as usize;
                if iad > 0 && iad < i {
                    k.ykc(0..(i - iad) * d, iad * d);
                    for c in 0..iad { for b in 0..d { k[c * d + b] = 0xFF080810; } }
                }
            }

            if frame > es * 4 / 5 {
                np(&mut k, d, i, i - 50,
                    "Small enough to understand.", 0xFF88DDAA, 2);
            }

            mb(&k, d, i);
            crate::cpu::tsc::rd(BQ_);
        }
    }
    ldp(&mut k, d, i, 8); 

    
    
    
    fkr(&mut k, d, i,
        &[("Your data. Your machine.", 0xFFFFCC44, 3),
          ("Your code.", 0xFFFFCC44, 3),
          ("", 0, 1),
          ("No backdoors. No telemetry.", 0xFF44FFAA, 2),
          ("No secrets.", 0xFF44FFAA, 2)],
        4, |k, d, i, bb| hal(k, d, i, bb));

    
    
    
    
    
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

        let bo = features.len();

        
        for (a, &(dq, sub, s, ei)) in features.iter().cf() {
            if bmd() { break; }

            
            for ai in k.el() { *ai = ei; }

            
            let abx = d / 6;
            let aha = i / 8;
            let aog = d * 2 / 3;
            let biz = i * 3 / 8;
            
            ah(&mut k, d, i, abx + 4, aha + 4, aog, biz, 0xFF020202);
            
            ah(&mut k, d, i, abx, aha, aog, biz, 0xFF111118);
            
            let xbi = ((s >> 16) & 0xFF) / 4;
            let xbh = ((s >> 8) & 0xFF) / 4;
            let xbf = (s & 0xFF) / 4;
            let xbg = 0xFF000000 | (xbi << 16) | (xbh << 8) | xbf;
            ah(&mut k, d, i, abx, aha, aog, 22, xbg);
            
            ri(&mut k, d, i, abx + 8, aha + 3, dq, s, 1);
            
            ah(&mut k, d, i, abx + aog - 18, aha + 5, 12, 12, 0xFFFF4444);
            ah(&mut k, d, i, abx + aog - 34, aha + 5, 12, 12, 0xFF888844);
            ah(&mut k, d, i, abx + aog - 50, aha + 5, 12, 12, 0xFF448844);

            
            let cx = abx + 16;
            let ae = aha + 30;
            let dt = aog - 32;
            match a {
                0 => { 
                    ah(&mut k, d, i, cx, ae, dt / 2 - 4, biz / 2 - 20, 0xFF1A2A4A);
                    ah(&mut k, d, i, cx, ae, dt / 2 - 4, 12, 0xFF3355AA);
                    ah(&mut k, d, i, cx + dt / 2 + 4, ae + 20, dt / 2 - 4, biz / 2 - 40, 0xFF1A3A2A);
                    ah(&mut k, d, i, cx + dt / 2 + 4, ae + 20, dt / 2 - 4, 12, 0xFF33AA55);
                    ah(&mut k, d, i, cx, ae + biz - 60, dt, 16, 0xFF222233);
                }
                1 => { 
                    for line in 0..5u32 {
                        let c = ae + 4 + line as usize * 18;
                        let oyg = ["> ls -la", "> cat readme.md", "> trust run app", "> netstat", "> _"];
                        if (line as usize) < oyg.len() {
                            ri(&mut k, d, i, cx + 4, c, oyg[line as usize], 0xFF00CC44, 1);
                        }
                    }
                }
                2 => { 
                    ah(&mut k, d, i, cx + 4, ae + 2, dt - 8, 14, 0xFF222233);
                    ri(&mut k, d, i, cx + 8, ae + 3, "https://trustos.dev", 0xFF4488FF, 1);
                    for line in 0..4u32 {
                        let c = ae + 24 + line as usize * 14;
                        let zv = dt - 40 - ((line as usize * 30) % 80);
                        ah(&mut k, d, i, cx + 12, c, zv, 8, 0xFF333344);
                    }
                }
                5 => { 
                    let hl = cx + dt / 2;
                    let ir = ae + 10;
                    let nf = (biz / 3).v(dt / 3);
                    
                    for ab in 0..nf {
                        let gqb = hl + ab - nf / 2;
                        let egz = ir + nf;
                        if gqb < d && egz < i { k[egz * d + gqb] = 0xFFFFCC00; }
                        let avw = ab as f32 / nf as f32;
                        let gqc = hl - (nf as f32 / 2.0 * (1.0 - avw)) as usize + (nf as f32 * avw / 2.0) as usize;
                        let eha = ir + nf - (nf as f32 * avw) as usize;
                        if gqc < d && eha < i { k[eha * d + gqc.v(d - 1)] = 0xFFFFCC00; }
                    }
                }
                6 => { 
                    let im = ((biz - 40) / 4).v(dt / 8);
                    let bx = cx + (dt - im * 4) / 2;
                    let je = ae + 4;
                    for br in 0..4u32 {
                        for bj in 0..4u32 {
                            let dark = (br + bj) % 2 == 0;
                            let jt = if dark { 0xFF886633 } else { 0xFFDDCC99 };
                            ah(&mut k, d, i, bx + bj as usize * im, je + br as usize * im, im, im, jt);
                        }
                    }
                }
                8 => { 
                    let iko = 12;
                    let lo = (dt - 20) / iko;
                    for o in 0..iko {
                        let hra = biz - 50;
                        let adn = ((o * 7 + 13) % hra).am(10);
                        let bx = cx + 10 + o * lo;
                        let je = ae + biz - 50 - adn;
                        let at = (0x88 + o as u32 * 0x08).v(0xFF);
                        ah(&mut k, d, i, bx, je, lo - 2, adn, 0xFF000000 | (at << 8) | 0x44);
                    }
                }
                _ => { 
                    for line in 0..5u32 {
                        let c = ae + 6 + line as usize * 16;
                        let zv = dt - 24 - ((line as usize * 40 + a * 17) % 100);
                        let kpx = ((s >> 16) & 0xFF) / 6;
                        let kpw = ((s >> 8) & 0xFF) / 6;
                        let kpv = (s & 0xFF) / 6;
                        ah(&mut k, d, i, cx + 12, c, zv, 8,
                            0xFF000000 | (kpx << 16) | (kpw << 8) | kpv);
                    }
                }
            }

            
            let cce = aha + biz + 30;
            eph(&mut k, d, i, cce, dq, s, 4);
            np(&mut k, d, i, cce + 50, sub, 0xFF888888, 2);

            
            let san = i - 35;
            let nmo = bo * 10;
            let sam = if nmo < d { (d - nmo) / 2 } else { 0 };
            for bc in 0..bo {
                let bmr = if bc <= a { s } else { 0xFF333333 };
                ah(&mut k, d, i, sam + bc * 10, san, 6, 6, bmr);
            }

            gaf(&mut k, d, i, 180);
            mb(&k, d, i);

            
            let bmv = 400u64.ao(a as u64 * 320 / (bo as u64 - 1).am(1));
            crate::cpu::tsc::rd(bmv);

            
            if a < bo - 1 {
                for ai in k.el() { *ai = 0xFFFFFFFF; }
                mb(&k, d, i);
                crate::cpu::tsc::rd(if bmv > 200 { 33 } else { 16 });
            }
        }

        
        for &(dq, _, s, ei) in features.iter() {
            if bmd() { break; }
            for ai in k.el() { *ai = ei; }
            eph(&mut k, d, i, i / 2 - 20, dq, s, 4);
            mb(&k, d, i);
            crate::cpu::tsc::rd(50);
        }

        
        for &(dq, _, s, ei) in features.iter() {
            if bmd() { break; }
            for ai in k.el() { *ai = ei; }
            np(&mut k, d, i, i / 2, dq, s, 5);
            mb(&k, d, i);
            crate::cpu::tsc::rd(BQ_);
        }
    }

    
    dhg(&mut k, d, i);

    
    
    
    
    
    jqj(&mut k, d, i, 2);

    
    
    
    dhg(&mut k, d, i);
    ldp(&mut k, d, i, 15); 

    
    
    
    
    {
        let es = anb(8);
        for frame in 0..es {
            if bmd() { break; }
            qpj(&mut k, d, i, &mut ws, &yg, frame);

            
            if frame > 8 {
                let cx = d / 2;
                let ae = i / 2;
                for mz in 0..3u32 {
                    let m = ((frame - 8 - mz * 6) as usize).hx(4);
                    if m > 0 && m < d {
                        for hg in 0..360 {
                            let aql = crate::formula3d::lz(hg as f32 * 0.01745);
                            let apn = crate::formula3d::rk(hg as f32 * 0.01745);
                            let y = (cx as f32 + apn * m as f32) as usize;
                            let x = (ae as f32 + aql * m as f32 / 1.5) as usize;
                            if y < d && x < i {
                                let yx = 255u32.ao(m as u32);
                                k[x * d + y] = 0xFF000000 | (yx / 4 << 16) | (yx << 8) | (yx / 3);
                            }
                        }
                    }
                }
            }

            
            if frame > anb(1) {
                np(&mut k, d, i, i / 2 - 50,
                    "TRUST THE CODE.", 0xFF00FFAA, 5);
            }
            if frame > anb(3) {
                np(&mut k, d, i, i / 2 + 30,
                    "github.com/nathan237/TrustOS", 0xFF00FF88, 2);
            }
            if frame > anb(5) {
                np(&mut k, d, i, i / 2 + 70,
                    "Written in Rust. By one person.", 0xFF88CCAA, 2);
                np(&mut k, d, i, i / 2 + 100,
                    "For everyone.", 0xFF88CCAA, 2);
            }

            mb(&k, d, i);
            crate::cpu::tsc::rd(BQ_);
        }
    }

    
    
    
    
    {
        let es = anb(4);
        for frame in 0..es {
            if bmd() { break; }
            myx(&mut k, d, i, frame);

            let bx = d / 2 - 180;
            let je = i / 2 - 40;
            ah(&mut k, d, i, bx, je, 360, 80, 0xFF111111);
            for b in bx..bx+360 { if b < d { k[je * d + b] = 0xFF888888; k[(je+79) * d + b] = 0xFF888888; } }
            for c in je..je+80 { if c < i { k[c * d + bx] = 0xFF888888; k[c * d + bx+359] = 0xFF888888; } }

            np(&mut k, d, i, je + 10, "PLEASE STAND BY", 0xFFCCCCCC, 2);
            if (frame / 15) % 2 == 0 {
                np(&mut k, d, i, je + 45,
                    "TRUSTOS v0.3.3 -- LOADING...", 0xFF00FF88, 2);
            }

            mb(&k, d, i);
            crate::cpu::tsc::rd(BQ_);
        }
    }

    
    apq(&mut k, d, i);

    
    
    
    aol(&mut k);
    mb(&k, d, i);
    if !afk {
        crate::framebuffer::afi(false);
    }
    crate::framebuffer::clear();
    crate::serial_println!("[TRAILER] TrustOS Trailer finished (145 beats)");
}
