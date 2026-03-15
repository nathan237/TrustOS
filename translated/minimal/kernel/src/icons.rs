



use crate::framebuffer;
use libm::{zq, st};


pub const BV_: u32 = 32;
pub const DQI_: u32 = 16;


pub fn sge(b: u32, c: u32, s: u32, ei: u32) {
    
    let dark = cdz(s, 0.6);
    let light = clh(s, 1.3);
    
    
    framebuffer::ah(b, c, 32, 32, ei);
    framebuffer::ah(b + 2, c + 2, 28, 28, dark);
    
    
    framebuffer::ah(b + 2, c + 2, 28, 6, s);
    
    
    framebuffer::ah(b + 4, c + 4, 2, 2, 0xFFFF5555); 
    framebuffer::ah(b + 8, c + 4, 2, 2, 0xFFFFAA00); 
    framebuffer::ah(b + 12, c + 4, 2, 2, 0xFF55FF55); 
    
    
    framebuffer::ah(b + 4, c + 10, 24, 18, 0xFF0A0A0A);
    
    
    framebuffer::ah(b + 6, c + 14, 2, 6, s);  
    framebuffer::ah(b + 8, c + 17, 2, 2, s);  
    framebuffer::ah(b + 12, c + 20, 8, 2, light); 
}


pub fn sdb(b: u32, c: u32, s: u32, cvl: u32) {
    let dark = cdz(s, 0.7);
    let light = clh(s, 1.2);
    
    
    framebuffer::ah(b + 2, c + 6, 12, 4, light);
    
    
    framebuffer::ah(b + 2, c + 8, 28, 18, s);
    
    
    framebuffer::ah(b + 2, c + 12, 28, 14, dark);
    
    
    framebuffer::ah(b + 2, c + 12, 28, 2, light);
    
    
    framebuffer::ah(b + 4, c + 24, 26, 2, cdz(dark, 0.5));
}


pub fn scu(b: u32, c: u32, s: u32, cvl: u32) {
    let dark = cdz(s, 0.8);
    let light = clh(s, 1.2);
    
    
    framebuffer::ah(b + 4, c + 2, 20, 28, 0xFFEEEEEE);
    
    
    framebuffer::ah(b + 18, c + 2, 6, 6, 0xFFCCCCCC);
    framebuffer::ah(b + 18, c + 2, 1, 6, 0xFFDDDDDD);
    
    
    for a in 0..5 {
        framebuffer::ah(b + 8, c + 10 + a * 4, 12, 2, dark);
    }
    
    
    framebuffer::lx(b + 4, c + 2, 20, 28, s);
}


pub fn sfk(b: u32, c: u32, s: u32, cvl: u32) {
    let cx = b + 16;
    let ae = c + 16;
    let light = clh(s, 1.2);
    
    
    epc(cx, ae, 6, s);
    epc(cx, ae, 3, 0xFF0A0A0A);
    
    
    let idb: [(i32, i32); 8] = [
        (0, -10), (7, -7), (10, 0), (7, 7),
        (0, 10), (-7, 7), (-10, 0), (-7, -7),
    ];
    for (dx, bg) in idb {
        framebuffer::ah(
            (cx as i32 + dx - 2) as u32,
            (ae as i32 + bg - 2) as u32,
            5, 5, s
        );
    }
}


pub fn scc(b: u32, c: u32, s: u32, cvl: u32) {
    let dark = cdz(s, 0.6);
    
    
    framebuffer::ah(b + 4, c + 2, 24, 28, dark);
    framebuffer::lx(b + 4, c + 2, 24, 28, s);
    
    
    framebuffer::ah(b + 6, c + 4, 20, 8, 0xFF1A2A1A);
    framebuffer::ah(b + 18, c + 6, 6, 4, s); 
    
    
    for br in 0..4 {
        for bj in 0..4 {
            let bx = b + 6 + bj * 5;
            let je = c + 14 + br * 4;
            let hbk = if bj == 3 { 0xFF44AA44 } else { 0xFF333333 };
            framebuffer::ah(bx, je, 4, 3, hbk);
        }
    }
}


pub fn sek(b: u32, c: u32, s: u32, cvl: u32) {
    let light = clh(s, 1.2);
    
    
    cxc(b + 16, c + 16, 12, s);
    
    
    framebuffer::ah(b + 6, c + 11, 20, 1, s);
    framebuffer::ah(b + 4, c + 16, 24, 1, s);
    framebuffer::ah(b + 6, c + 21, 20, 1, s);
    
    
    cxc(b + 16, c + 16, 6, s);
    
    
    framebuffer::ah(b + 15, c + 4, 2, 24, s);
}


pub fn sbb(b: u32, c: u32, s: u32, cvl: u32) {
    let light = clh(s, 1.3);
    
    
    epc(b + 16, c + 16, 12, s);
    epc(b + 16, c + 16, 10, 0xFF0A0A0A);
    
    
    framebuffer::ah(b + 14, c + 10, 4, 4, light); 
    framebuffer::ah(b + 14, c + 16, 4, 10, light); 
    framebuffer::ah(b + 12, c + 24, 8, 2, light); 
}


pub fn see(b: u32, c: u32, s: u32, cvl: u32) {
    let light = clh(s, 1.3);
    
    
    epc(b + 10, c + 24, 5, light);
    epc(b + 10, c + 24, 4, s);
    
    framebuffer::ah(b + 14, c + 6, 2, 19, light);
    
    framebuffer::ah(b + 16, c + 6, 2, 4, light);
    framebuffer::ah(b + 18, c + 8, 2, 4, light);
    framebuffer::ah(b + 20, c + 10, 2, 4, light);
}


pub fn kqz(b: u32, c: u32, s: u32, cvl: u32) {
    let dark = cdz(s, 0.7);
    let light = clh(s, 1.2);
    
    
    framebuffer::ah(b + 4, c + 10, 24, 14, dark);
    framebuffer::ah(b + 2, c + 12, 4, 10, dark);
    framebuffer::ah(b + 26, c + 12, 4, 10, dark);
    
    
    framebuffer::ah(b + 8, c + 14, 2, 6, s);
    framebuffer::ah(b + 6, c + 16, 6, 2, s);
    
    
    framebuffer::ah(b + 22, c + 14, 3, 3, 0xFF55FF55);
    framebuffer::ah(b + 18, c + 17, 3, 3, 0xFFFF5555);
}


pub fn scr(b: u32, c: u32, s: u32, cvl: u32) {
    let dark = cdz(s, 0.7);
    
    
    framebuffer::ah(b + 4, c + 2, 24, 28, 0xFFEEEEEE);
    framebuffer::lx(b + 4, c + 2, 24, 28, dark);
    
    
    framebuffer::ah(b + 8, c + 6, 16, 2, s);
    framebuffer::ah(b + 8, c + 10, 14, 2, dark);
    framebuffer::ah(b + 8, c + 14, 16, 2, dark);
    framebuffer::ah(b + 8, c + 18, 10, 2, dark);
    framebuffer::ah(b + 8, c + 22, 14, 2, dark);
    
    
    framebuffer::ah(b + 10, c + 22, 1, 4, s);
}






fn cdz(s: u32, pv: f32) -> u32 {
    let m = ((s >> 16) & 0xFF) as f32 * pv;
    let at = ((s >> 8) & 0xFF) as f32 * pv;
    let o = (s & 0xFF) as f32 * pv;
    0xFF000000 | ((m as u32) << 16) | ((at as u32) << 8) | (o as u32)
}


fn clh(s: u32, pv: f32) -> u32 {
    let m = (((s >> 16) & 0xFF) as f32 * pv).v(255.0);
    let at = (((s >> 8) & 0xFF) as f32 * pv).v(255.0);
    let o = ((s & 0xFF) as f32 * pv).v(255.0);
    0xFF000000 | ((m as u32) << 16) | ((at as u32) << 8) | (o as u32)
}


fn epc(cx: u32, ae: u32, m: u32, s: u32) {
    let m = m as i32;
    let cx = cx as i32;
    let ae = ae as i32;
    
    for bg in -m..=m {
        for dx in -m..=m {
            if dx * dx + bg * bg <= m * m {
                let y = cx + dx;
                let x = ae + bg;
                if y >= 0 && x >= 0 {
                    framebuffer::sf(y as u32, x as u32, s);
                }
            }
        }
    }
}


fn cxc(cx: u32, ae: u32, m: u32, s: u32) {
    let m = m as i32;
    let cx = cx as i32;
    let ae = ae as i32;
    
    let mut b = m;
    let mut c = 0;
    let mut rq = 0;
    
    while b >= c {
        exa(cx + b, ae + c, s);
        exa(cx + c, ae + b, s);
        exa(cx - c, ae + b, s);
        exa(cx - b, ae + c, s);
        exa(cx - b, ae - c, s);
        exa(cx - c, ae - b, s);
        exa(cx + c, ae - b, s);
        exa(cx + b, ae - c, s);
        
        c += 1;
        rq += 1 + 2 * c;
        if 2 * (rq - b) + 1 > 0 {
            b -= 1;
            rq += 1 - 2 * b;
        }
    }
}

fn exa(b: i32, c: i32, s: u32) {
    if b >= 0 && c >= 0 {
        framebuffer::sf(b as u32, c as u32, s);
    }
}





#[derive(Clone, Copy, PartialEq)]
pub enum IconType {
    Ay,
    Aig,
    Es,
    Gn,
    Calculator,
    As,
    Jf,
    Bmt,
    Io,
    Ahq,
    Ks,
    Browser,
    Fp,
    Aij,
    Lm,
    Gs,
}


pub fn ync(ecz: IconType, b: u32, c: u32, s: u32, ei: u32) {
    match ecz {
        IconType::Ay => sge(b, c, s, ei),
        IconType::Aig => sdb(b, c, s, ei),
        IconType::Es => scu(b, c, s, ei),
        IconType::Gn => sfk(b, c, s, ei),
        IconType::Calculator => scc(b, c, s, ei),
        IconType::As => sek(b, c, s, ei),
        IconType::Jf => sbb(b, c, s, ei),
        IconType::Bmt => see(b, c, s, ei),
        IconType::Io => kqz(b, c, s, ei),
        IconType::Ahq => scr(b, c, s, ei),
        IconType::Ks => sen(b, c, s, ei),
        IconType::Browser => sbz(b, c, s, ei),
        IconType::Fp => sea(b, c, s, ei),
        IconType::Aij => kqz(b, c, s, ei), 
        IconType::Lm => sde(b, c, s, ei),
        IconType::Gs => kqz(b, c, s, ei), 
    }
}


pub fn sen(b: u32, c: u32, s: u32, cvl: u32) {
    let dark = cdz(s, 0.5);
    let light = clh(s, 1.4);
    
    
    let cx = b as i32 + 16;
    let ae = c as i32 + 18;
    let aw: i32 = 8;
    
    
    let adi = |y: i32, x: i32, r: u32| {
        if y >= 0 && x >= 0 {
            framebuffer::sf(y as u32, x as u32, r);
        }
    };
    
    
    for a in 0..=aw {
        adi(cx - aw + a, ae - aw, light);
        adi(cx + aw, ae - aw + a, light);
        adi(cx + aw - a, ae + aw, light);
        adi(cx - aw, ae + aw - a, light);
    }
    
    
    let l: i32 = 5;
    for a in 0..=aw {
        adi(cx - aw + a + l, ae - aw - l, dark);
        adi(cx + aw + l, ae - aw + a - l, dark);
        adi(cx + aw - a + l, ae + aw - l, dark);
        adi(cx - aw + l, ae + aw - a - l, dark);
    }
    
    
    for a in 0..l {
        adi(cx - aw + a, ae - aw - a, s);
        adi(cx + aw + a, ae - aw - a, s);
        adi(cx + aw + a, ae + aw - a, s);
        adi(cx - aw + a, ae + aw - a, s);
    }
    
    
    let gx = b as i32;
    let ty = c as i32;
    
    adi(gx + 10, ty + 26, light);
    adi(gx + 11, ty + 26, light);
    adi(gx + 12, ty + 26, light);
    adi(gx + 9, ty + 27, light);
    adi(gx + 9, ty + 28, light);
    adi(gx + 10, ty + 29, light);
    adi(gx + 11, ty + 29, light);
    adi(gx + 12, ty + 29, light);
    adi(gx + 12, ty + 28, light);
    
    
    adi(gx + 15, ty + 26, light);
    adi(gx + 15, ty + 27, light);
    adi(gx + 15, ty + 28, light);
    adi(gx + 15, ty + 29, light);
    adi(gx + 16, ty + 29, light);
    adi(gx + 17, ty + 29, light);
}


pub fn sbz(b: u32, c: u32, s: u32, cvl: u32) {
    let dark = cdz(s, 0.6);
    let light = clh(s, 1.3);
    
    let cx = b as i32 + 16;
    let ae = c as i32 + 16;
    
    
    for hg in 0..360 {
        let bak = (hg as f32) * 3.14159 / 180.0;
        let y = cx + (zq(bak) * 12.0) as i32;
        let x = ae + (st(bak) * 12.0) as i32;
        if y >= 0 && x >= 0 {
            framebuffer::sf(y as u32, x as u32, s);
        }
    }
    
    
    for hg in 0..360 {
        let bak = (hg as f32) * 3.14159 / 180.0;
        let y = cx + (zq(bak) * 8.0) as i32;
        let x = ae + (st(bak) * 8.0) as i32;
        if y >= 0 && x >= 0 {
            framebuffer::sf(y as u32, x as u32, dark);
        }
    }
    
    
    for bg in -12i32..=12 {
        let x = ae + bg;
        if x >= 0 {
            framebuffer::sf(cx as u32, x as u32, light);
        }
    }
    
    
    for dx in -12i32..=12 {
        let y = cx + dx;
        if y >= 0 {
            framebuffer::sf(y as u32, ae as u32, light);
        }
    }
    
    
    for dx in -10i32..=10 {
        let y = cx + dx;
        if y >= 0 {
            framebuffer::sf(y as u32, (ae - 6) as u32, dark);
            framebuffer::sf(y as u32, (ae + 6) as u32, dark);
        }
    }
    
    
    framebuffer::ah(b + 2, c + 26, 28, 4, dark);
    framebuffer::ah(b + 4, c + 27, 24, 2, 0xFF202020);
}


pub fn sea(b: u32, c: u32, s: u32, cvl: u32) {
    let dark = cdz(s, 0.5);
    let light = clh(s, 1.4);
    let mm = 0xFF00FFAA; 
    
    let cx = b as i32 + 16;
    let ae = c as i32 + 16;
    
    
    let adi = |y: i32, x: i32, r: u32| {
        if y >= 0 && x >= 0 {
            framebuffer::sf(y as u32, x as u32, r);
        }
    };
    
    
    let e: i32 = 7;
    for a in 0..=e {
        adi(cx - e + a - 2, ae - e + 2, mm);
        adi(cx + e - 2, ae - e + a + 2, mm);
        adi(cx + e - a - 2, ae + e + 2, mm);
        adi(cx - e - 2, ae + e - a + 2, mm);
    }
    
    
    let dkb: i32 = 5;
    for a in 0..=e {
        adi(cx - e + a + dkb - 2, ae - e - dkb + 2, dark);
        adi(cx + e + dkb - 2, ae - e + a - dkb + 2, dark);
    }
    
    
    for a in 0..dkb {
        adi(cx - e + a - 2, ae - e - a + 2, light);
        adi(cx + e + a - 2, ae - e - a + 2, light);
        adi(cx + e + a - 2, ae + e - a + 2, light);
    }
    
    
    for a in 0..5 {
        adi(cx + 8, ae + 6 + a, 0xFFFFFFFF);
        adi(cx + 6 + a, ae + 8, 0xFFFFFFFF);
    }
    
    
    adi(cx - e - 2, ae - e + 2, 0xFFFFFF00);
    adi(cx + e - 2, ae + e + 2, 0xFFFFFF00);
    adi(cx + e + dkb - 2, ae - e - dkb + 2, 0xFFFFFF00);
}


pub fn sde(b: u32, c: u32, s: u32, cvl: u32) {
    use crate::framebuffer;
    let aaj = 0xFF00FF88u32;
    let tp = cdz(s, 0.5);
    
    
    framebuffer::ah(b + 8, c + 16, 16, 12, tp);
    framebuffer::ah(b + 6, c + 20, 20, 8, tp);
    
    
    framebuffer::ah(b + 13, c + 6, 6, 10, tp);
    
    
    framebuffer::ah(b + 11, c + 4, 10, 2, s);
    
    
    framebuffer::ah(b + 9, c + 22, 14, 4, aaj);
    framebuffer::ah(b + 10, c + 18, 12, 4, 0xFF00CC66);
    
    
    framebuffer::ah(b + 11, c + 19, 2, 2, 0xFFFFFFFF);
    framebuffer::ah(b + 16, c + 23, 2, 2, 0xFFFFFFFF);
    framebuffer::ah(b + 19, c + 20, 2, 2, 0xFFFFFFFF);
    
    
    framebuffer::ah(b + 7, c + 16, 1, 12, aaj);
    framebuffer::ah(b + 24, c + 16, 1, 12, aaj);
    framebuffer::ah(b + 6, c + 28, 20, 1, aaj);
}

