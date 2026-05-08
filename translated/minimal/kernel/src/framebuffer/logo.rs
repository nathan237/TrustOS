




use alloc::vec::Vec;


pub const BBE_: usize = 64;
pub const BBC_: usize = 80;


pub const JA_: u32 = 0xFF00FF00;
pub const OX_: u32 = 0xFF00CC00;
pub const WD_: u32 = 0xFF008800;
pub const WE_: u32 = 0xFF004400;



#[rustfmt::skip]
pub static DWJ_: [u8; BBE_ * BBC_] = {
    let mut data = [0u8; BBE_ * BBC_];
    data
};


pub fn draw_logo(x: u32, y: u32) {
    hto(x, y, 1);
}


pub fn qdu(x: u32, y: u32, scale: u32) {
    hto(x, y, scale);
}


fn hto(cx: u32, u: u32, scale: u32) {
    let j = scale;
    
    
    let auf = cx + 24 * j;
    let afi = u;
    
    
    lia(auf + 8 * j, afi + 2 * j, 6 * j, 8 * j, JA_);
    
    
    draw_filled_rect(auf + 2 * j, afi + 10 * j, 12 * j, 10 * j, OX_);
    draw_rect_outline(auf + 2 * j, afi + 10 * j, 12 * j, 10 * j, JA_);
    
    
    byw(auf + 8 * j, afi + 14 * j, 2 * j, WE_);
    draw_filled_rect(auf + 7 * j, afi + 14 * j, 2 * j, 4 * j, WE_);
    
    
    let orr = cx + 8 * j;
    let bvt = u + 22 * j;
    let apd = 48 * j;
    let bje = 44 * j;
    
    lkr(orr, bvt, apd, bje, OX_, JA_);
    
    
    let flj = cx + 20 * j;
    let flk = u + 38 * j;
    lih(flj, flk, 24 * j, JA_);
    
    
    lkn(cx, u + 30 * j, 64 * j, 36 * j, WD_);
}


fn draw_filled_rect(x: u32, y: u32, w: u32, h: u32, color: u32) {
    for o in y..(y + h) {
        for p in x..(x + w) {
            super::put_pixel(p, o, color);
        }
    }
}


fn draw_rect_outline(x: u32, y: u32, w: u32, h: u32, color: u32) {
    
    for p in x..(x + w) {
        super::put_pixel(p, y, color);
        super::put_pixel(p, y + h - 1, color);
    }
    
    for o in y..(y + h) {
        super::put_pixel(x, o, color);
        super::put_pixel(x + w - 1, o, color);
    }
}


fn byw(cx: u32, u: u32, r: u32, color: u32) {
    let amn = (r * r) as i32;
    for ad in -(r as i32)..(r as i32 + 1) {
        for dx in -(r as i32)..(r as i32 + 1) {
            if dx * dx + ad * ad <= amn {
                let p = (cx as i32 + dx) as u32;
                let o = (u as i32 + ad) as u32;
                super::put_pixel(p, o, color);
            }
        }
    }
}


fn lia(cx: u32, u: u32, boi: u32, amm: u32, color: u32) {
    let oas = (boi * boi) as i32;
    let oat = (amm * amm) as i32;
    
    for ad in -(amm as i32)..1 {  
        for dx in -(amm as i32)..(amm as i32 + 1) {
            let byn = dx * dx + ad * ad;
            if byn >= oas && byn <= oat {
                let p = (cx as i32 + dx) as u32;
                let o = (u as i32 + ad) as u32;
                super::put_pixel(p, o, color);
            }
        }
    }
    
    for ad in 0..(amm - boi + 2) {
        let aue = cx - amm + 1;
        let asa = cx + amm - 1;
        let o = u + ad;
        for t in 0..(amm - boi) {
            super::put_pixel(aue + t, o, color);
            super::put_pixel(asa - t, o, color);
        }
    }
}


fn lkr(x: u32, y: u32, w: u32, h: u32, bso: u32, bob: u32) {
    let nk = w / 2;
    let crq = y + h;
    
    
    let cde = h * 2 / 3;
    for o in y..(y + cde) {
        for p in x..(x + w) {
            
            let lfy = if p < x + nk { 
                x + nk - p 
            } else { 
                p - (x + nk) 
            };
            let shade = if lfy < w / 6 {
                bso
            } else {
                hia(bso, 0xFF000000, 20)
            };
            super::put_pixel(p, o, shade);
        }
    }
    
    
    for o in (y + cde)..crq {
        let progress = (o - (y + cde)) as f32 / (h - cde) as f32;
        let chx = ((1.0 - progress) * nk as f32) as u32;
        
        if chx > 0 {
            let left = x + nk - chx;
            let right = x + nk + chx;
            for p in left..right {
                super::put_pixel(p, o, bso);
            }
        }
    }
    
    
    
    for p in x..(x + w) {
        super::put_pixel(p, y, bob);
    }
    
    for o in y..(y + cde) {
        super::put_pixel(x, o, bob);
        super::put_pixel(x + w - 1, o, bob);
    }
    
    for o in (y + cde)..crq {
        let progress = (o - (y + cde)) as f32 / (h - cde) as f32;
        let chx = ((1.0 - progress) * nk as f32) as u32;
        if chx > 0 {
            super::put_pixel(x + nk - chx, o, bob);
            super::put_pixel(x + nk + chx, o, bob);
        }
    }
    
    super::put_pixel(x + nk, crq - 1, bob);
}


fn lih(x: u32, y: u32, size: u32, color: u32) {
    let rh = core::cmp::max(2, size / 8);
    
    
    let start_x = x;
    let start_y = y + size / 3;
    let arn = x + size / 3;
    let ags = y + size * 2 / 3;
    
    hts(start_x, start_y, arn, ags, rh, color);
    
    
    let awy = x + size;
    let doq = y;
    
    hts(arn, ags, awy, doq, rh, color);
}


fn hts(bm: u32, az: u32, x1: u32, y1: u32, rh: u32, color: u32) {
    let dx = (x1 as i32 - bm as i32).abs();
    let ad = (y1 as i32 - az as i32).abs();
    let am: i32 = if bm < x1 { 1 } else { -1 };
    let ak: i32 = if az < y1 { 1 } else { -1 };
    let mut err = dx - ad;
    
    let mut x = bm as i32;
    let mut y = az as i32;
    let x1 = x1 as i32;
    let y1 = y1 as i32;
    
    loop {
        
        for ty in -(rh as i32 / 2)..(rh as i32 / 2 + 1) {
            for bu in -(rh as i32 / 2)..(rh as i32 / 2 + 1) {
                if bu * bu + ty * ty <= (rh as i32 / 2) * (rh as i32 / 2) {
                    super::put_pixel((x + bu) as u32, (y + ty) as u32, color);
                }
            }
        }
        
        if x == x1 && y == y1 {
            break;
        }
        
        let pg = 2 * err;
        if pg > -ad {
            err -= ad;
            x += am;
        }
        if pg < dx {
            err += dx;
            y += ak;
        }
    }
}


fn lkn(x: u32, y: u32, w: u32, h: u32, color: u32) {
    let aps = w / 10;
    
    
    let aue = x;
    let dal = y + h / 4;
    
    
    draw_filled_rect(aue, dal, w / 4, aps, color);
    
    draw_filled_rect(aue, dal + aps, aps * 2, h / 3, color);
    
    draw_filled_rect(aue + w / 4 - aps, dal - aps, aps * 2, aps * 3, color);
    
    
    let asa = x + w - w / 4;
    draw_filled_rect(asa, dal, w / 4, aps, color);
    draw_filled_rect(x + w - aps * 2, dal + aps, aps * 2, h / 3, color);
    draw_filled_rect(asa - aps, dal - aps, aps * 2, aps * 3, color);
}


fn hia(agh: u32, ale: u32, alpha: u32) -> u32 {
    let alpha = alpha.min(255);
    let sg = 255 - alpha;
    
    let uh = (agh >> 16) & 0xFF;
    let bbu = (agh >> 8) & 0xFF;
    let gf = agh & 0xFF;
    
    let ju = (ale >> 16) & 0xFF;
    let axe = (ale >> 8) & 0xFF;
    let iq = ale & 0xFF;
    
    let r = (uh * sg + ju * alpha) / 255;
    let g = (bbu * sg + axe * alpha) / 255;
    let b = (gf * sg + iq * alpha) / 255;
    
    0xFF000000 | (r << 16) | (g << 8) | b
}


pub fn hti() {
    let (width, height) = super::kv();

    
    super::fill_rect(0, 0, width, height, LW_);

    
    let ark = crate::logo_bitmap::BA_ as u32;
    let arj = crate::logo_bitmap::BN_ as u32;
    let cbn = (width / 2).saturating_sub(ark / 2);
    let cbo = (height / 2).saturating_sub(arj / 2);
    crate::logo_bitmap::draw_logo(cbn, cbo);
}


fn qeb(cx: u32, y: u32, _scale: u32) {
    
    let title = "TRust-OS";
    let pkc = title.len() as u32;
    let ew = 8u32;
    let start_x = cx.saturating_sub(pkc * ew / 2);
    
    
    let afu = (start_x / ew) as usize;
    let row = (y / 16) as usize;
    
    
    for (i, c) in title.chars().enumerate() {
        let p = start_x + (i as u32) * ew;
        px(c, p as usize, y as usize, JA_, 0xFF000000);
    }
}


fn qdz(cx: u32, y: u32, _scale: u32) {
    let jlk = "FAST . SECURE . RELIABLE";
    let pcy = jlk.len() as u32;
    let ew = 8u32;
    let start_x = cx.saturating_sub(pcy * ew / 2);
    
    for (i, c) in jlk.chars().enumerate() {
        let p = start_x + (i as u32) * ew;
        
        px(c, p as usize, y as usize, OX_, 0xFF000000);
    }
}


fn px(c: char, x: usize, y: usize, fg: u32, bg: u32) {
    let du = super::font::ol(c);
    
    for row in 0..16 {
        let bits = du[row];
        for col in 0..8 {
            let color = if (bits >> (7 - col)) & 1 == 1 { fg } else { bg };
            if color != bg {  
                super::put_pixel((x + col) as u32, (y + row) as u32, color);
            }
        }
    }
}


fn qdw(width: u32, height: u32) {
    
    let mut seed: u32 = 12345;
    
    let pseudo_rand = |j: &mut u32| -> u32 {
        *j = j.wrapping_mul(1103515245).wrapping_add(12345);
        (*j >> 16) & 0x7FFF
    };
    
    
    let guz = width / 8;
    
    for _ in 0..200 {
        
        let x = pseudo_rand(&mut seed) % guz;
        let y = pseudo_rand(&mut seed) % height;
        let intensity = (pseudo_rand(&mut seed) % 4) as u8;
        let color = match intensity {
            0 => WE_,
            1 => WD_,
            2 => OX_,
            _ => JA_,
        };
        let c = (b'0' + (pseudo_rand(&mut seed) % 75) as u8) as char;
        px(c, x as usize, y as usize, color, 0xFF000000);
        
        
        let x = width - guz + pseudo_rand(&mut seed) % guz;
        let y = pseudo_rand(&mut seed) % height;
        let intensity = (pseudo_rand(&mut seed) % 4) as u8;
        let color = match intensity {
            0 => WE_,
            1 => WD_,
            2 => OX_,
            _ => JA_,
        };
        let c = (b'0' + (pseudo_rand(&mut seed) % 75) as u8) as char;
        px(c, x as usize, y as usize, color, 0xFF000000);
    }
}






const BOD_: u32 = 22;


const LW_: u32 = 0xFF050606;
const CXO_: u32 = 0xFF0A1A0E;
const BII_: u32 = 0xFF00FF66;
const CXP_: u32 = 0xFF00CC55;
const CXR_: u32 = 0xFF558866;
const CXQ_: u32 = 0xFFCCEEDD;
const EKJ_: u32 = 0xFF00AA44;




pub fn gcn() {
    let (width, height) = super::kv();
    if width == 0 || height == 0 { return; }

    
    super::fill_rect(0, 0, width, height, LW_);

    
    let ark = crate::logo_bitmap::BA_ as u32; 
    let arj = crate::logo_bitmap::BN_ as u32; 
    let cbn = (width / 2).saturating_sub(ark / 2);
    let cbo = (height / 2).saturating_sub(arj / 2);
    crate::logo_bitmap::draw_logo(cbn, cbo);

    
    let ek: u32 = 200;
    let hs: u32 = 8;
    let pv: u32 = 40;
    let gk = height - 60;

    
    super::fill_rect(pv, gk, ek, hs, CXO_);
    
    super::draw_rect(pv.saturating_sub(1), gk.saturating_sub(1), ek + 2, hs + 2, WD_);

    
    let mpp = "Initializing...";
    let mps = gk + hs + 8;
    for (i, c) in mpp.chars().enumerate() {
        let p = pv + (i as u32) * 8;
        px(c, p as usize, mps as usize, CXR_, LW_);
    }
}




pub fn afw(phase: u32, message: &str) {
    let (_width, height) = super::kv();
    if _width == 0 || height == 0 { return; }

    
    let ek: u32 = 200;
    let hs: u32 = 8;
    let pv: u32 = 40;
    let gk = height - 60;

    
    let progress = ((phase + 1) * 100) / BOD_;
    let fww = (ek * progress.min(100)) / 100;

    
    if fww > 0 {
        super::fill_rect(pv, gk, fww, hs, BII_);
        super::fill_rect(pv, gk, fww, 2, CXP_);
    }

    
    let bnq = gk + hs + 8;
    super::fill_rect(pv, bnq, 400, 18, LW_);

    
    for (i, c) in message.chars().enumerate() {
        let p = pv + (i as u32) * 8;
        px(c, p as usize, bnq as usize, CXQ_, LW_);
    }

    
    let nsw = if progress >= 100 {
        "100%"
    } else {
        static mut CME_: [u8; 5] = [0; 5];
        let buf = unsafe { &mut CME_ };
        let pdv = (progress / 10) as u8;
        let isg = (progress % 10) as u8;
        if progress >= 10 {
            buf[0] = b' ';
            buf[1] = b'0' + pdv;
            buf[2] = b'0' + isg;
            buf[3] = b'%';
            buf[4] = 0;
        } else {
            buf[0] = b' ';
            buf[1] = b' ';
            buf[2] = b'0' + isg;
            buf[3] = b'%';
            buf[4] = 0;
        }
        unsafe { core::str::from_utf8_unchecked(&buf[..4]) }
    };
    let nsx = pv + ek + 8;
    for (i, c) in nsw.chars().enumerate() {
        let p = nsx + (i as u32) * 8;
        px(c, p as usize, gk as usize, BII_, LW_);
    }
}


pub fn fvz() {
    let (width, height) = super::kv();
    if width == 0 || height == 0 { return; }
    
    
    for step in 0u32..8 {
        let alpha = (step + 1) * 32; 
        let shade = if alpha >= 255 { 0xFF000000 } else {
            
            let ki = 255 - alpha;
            let g = (0x05 * ki) / 255;
            0xFF000000 | (g << 8)
        };
        super::fill_rect(0, 0, width, height, shade);
        
        
        for _ in 0..2_000_000 { core::hint::spin_loop(); }
    }
    
    
    super::fill_rect(0, 0, width, height, 0xFF000000);
    
    for _ in 0..3_000_000 { core::hint::spin_loop(); }
}
