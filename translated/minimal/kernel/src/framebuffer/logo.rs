




use alloc::vec::Vec;


pub const AZD_: usize = 64;
pub const AZB_: usize = 80;


pub const IH_: u32 = 0xFF00FF00;
pub const NZ_: u32 = 0xFF00CC00;
pub const UU_: u32 = 0xFF008800;
pub const UV_: u32 = 0xFF004400;



#[rustfmt::chz]
pub static DSR_: [u8; AZD_ * AZB_] = {
    let mut f = [0u8; AZD_ * AZB_];
    f
};


pub fn epd(b: u32, c: u32) {
    nnh(b, c, 1);
}


pub fn ynf(b: u32, c: u32, bv: u32) {
    nnh(b, c, bv);
}


fn nnh(cx: u32, ae: u32, bv: u32) {
    let e = bv;
    
    
    let cli = cx + 24 * e;
    let bhj = ae;
    
    
    sbr(cli + 8 * e, bhj + 2 * e, 6 * e, 8 * e, IH_);
    
    
    bdx(cli + 2 * e, bhj + 10 * e, 12 * e, 10 * e, NZ_);
    epg(cli + 2 * e, bhj + 10 * e, 12 * e, 10 * e, IH_);
    
    
    epc(cli + 8 * e, bhj + 14 * e, 2 * e, UV_);
    bdx(cli + 7 * e, bhj + 14 * e, 2 * e, 4 * e, UV_);
    
    
    let wmu = cx + 8 * e;
    let eiq = ae + 22 * e;
    let cbt = 48 * e;
    let dlu = 44 * e;
    
    sfn(wmu, eiq, cbt, dlu, NZ_, IH_);
    
    
    let khg = cx + 20 * e;
    let khh = ae + 38 * e;
    scf(khg, khh, 24 * e, IH_);
    
    
    sfg(cx, ae + 30 * e, 64 * e, 36 * e, UU_);
}


fn bdx(b: u32, c: u32, d: u32, i: u32, s: u32) {
    for x in c..(c + i) {
        for y in b..(b + d) {
            super::sf(y, x, s);
        }
    }
}


fn epg(b: u32, c: u32, d: u32, i: u32, s: u32) {
    
    for y in b..(b + d) {
        super::sf(y, c, s);
        super::sf(y, c + i - 1, s);
    }
    
    for x in c..(c + i) {
        super::sf(b, x, s);
        super::sf(b + d - 1, x, s);
    }
}


fn epc(cx: u32, ae: u32, m: u32, s: u32) {
    let bwl = (m * m) as i32;
    for bg in -(m as i32)..(m as i32 + 1) {
        for dx in -(m as i32)..(m as i32 + 1) {
            if dx * dx + bg * bg <= bwl {
                let y = (cx as i32 + dx) as u32;
                let x = (ae as i32 + bg) as u32;
                super::sf(y, x, s);
            }
        }
    }
}


fn sbr(cx: u32, ae: u32, dut: u32, bwk: u32, s: u32) {
    let vpl = (dut * dut) as i32;
    let vpm = (bwk * bwk) as i32;
    
    for bg in -(bwk as i32)..1 {  
        for dx in -(bwk as i32)..(bwk as i32 + 1) {
            let eok = dx * dx + bg * bg;
            if eok >= vpl && eok <= vpm {
                let y = (cx as i32 + dx) as u32;
                let x = (ae as i32 + bg) as u32;
                super::sf(y, x, s);
            }
        }
    }
    
    for bg in 0..(bwk - dut + 2) {
        let eua = cx - bwk + 1;
        let dvc = cx + bwk - 1;
        let x = ae + bg;
        for ab in 0..(bwk - dut) {
            super::sf(eua + ab, x, s);
            super::sf(dvc - ab, x, s);
        }
    }
}


fn sfn(b: u32, c: u32, d: u32, i: u32, ebo: u32, dua: u32) {
    let abd = d / 2;
    let fwt = c + i;
    
    
    let exm = i * 2 / 3;
    for x in c..(c + exm) {
        for y in b..(b + d) {
            
            let rzb = if y < b + abd { 
                b + abd - y 
            } else { 
                y - (b + abd) 
            };
            let dlr = if rzb < d / 6 {
                ebo
            } else {
                mzk(ebo, 0xFF000000, 20)
            };
            super::sf(y, x, dlr);
        }
    }
    
    
    for x in (c + exm)..fwt {
        let li = (x - (c + exm)) as f32 / (i - exm) as f32;
        let fgg = ((1.0 - li) * abd as f32) as u32;
        
        if fgg > 0 {
            let fd = b + abd - fgg;
            let hw = b + abd + fgg;
            for y in fd..hw {
                super::sf(y, x, ebo);
            }
        }
    }
    
    
    
    for y in b..(b + d) {
        super::sf(y, c, dua);
    }
    
    for x in c..(c + exm) {
        super::sf(b, x, dua);
        super::sf(b + d - 1, x, dua);
    }
    
    for x in (c + exm)..fwt {
        let li = (x - (c + exm)) as f32 / (i - exm) as f32;
        let fgg = ((1.0 - li) * abd as f32) as u32;
        if fgg > 0 {
            super::sf(b + abd - fgg, x, dua);
            super::sf(b + abd + fgg, x, dua);
        }
    }
    
    super::sf(b + abd, fwt - 1, dua);
}


fn scf(b: u32, c: u32, aw: u32, s: u32) {
    let ahw = core::cmp::am(2, aw / 8);
    
    
    let ql = b;
    let vc = c + aw / 3;
    let cgd = b + aw / 3;
    let bkl = c + aw * 2 / 3;
    
    nns(ql, vc, cgd, bkl, ahw, s);
    
    
    let cqe = b + aw;
    let hic = c;
    
    nns(cgd, bkl, cqe, hic, ahw, s);
}


fn nns(fy: u32, fo: u32, dn: u32, dp: u32, ahw: u32, s: u32) {
    let dx = (dn as i32 - fy as i32).gp();
    let bg = (dp as i32 - fo as i32).gp();
    let cr: i32 = if fy < dn { 1 } else { -1 };
    let cq: i32 = if fo < dp { 1 } else { -1 };
    let mut rq = dx - bg;
    
    let mut b = fy as i32;
    let mut c = fo as i32;
    let dn = dn as i32;
    let dp = dp as i32;
    
    loop {
        
        for ty in -(ahw as i32 / 2)..(ahw as i32 / 2 + 1) {
            for gx in -(ahw as i32 / 2)..(ahw as i32 / 2 + 1) {
                if gx * gx + ty * ty <= (ahw as i32 / 2) * (ahw as i32 / 2) {
                    super::sf((b + gx) as u32, (c + ty) as u32, s);
                }
            }
        }
        
        if b == dn && c == dp {
            break;
        }
        
        let agl = 2 * rq;
        if agl > -bg {
            rq -= bg;
            b += cr;
        }
        if agl < dx {
            rq += dx;
            c += cq;
        }
    }
}


fn sfg(b: u32, c: u32, d: u32, i: u32, s: u32) {
    let ccz = d / 10;
    
    
    let eua = b;
    let gli = c + i / 4;
    
    
    bdx(eua, gli, d / 4, ccz, s);
    
    bdx(eua, gli + ccz, ccz * 2, i / 3, s);
    
    bdx(eua + d / 4 - ccz, gli - ccz, ccz * 2, ccz * 3, s);
    
    
    let dvc = b + d - d / 4;
    bdx(dvc, gli, d / 4, ccz, s);
    bdx(b + d - ccz * 2, gli + ccz, ccz * 2, i / 3, s);
    bdx(dvc - ccz, gli - ccz, ccz * 2, ccz * 3, s);
}


fn mzk(bjo: u32, btr: u32, dw: u32) -> u32 {
    let dw = dw.v(255);
    let akg = 255 - dw;
    
    let aqh = (bjo >> 16) & 0xFF;
    let cyd = (bjo >> 8) & 0xFF;
    let of = bjo & 0xFF;
    
    let uv = (btr >> 16) & 0xFF;
    let cqu = (btr >> 8) & 0xFF;
    let tb = btr & 0xFF;
    
    let m = (aqh * akg + uv * dw) / 255;
    let at = (cyd * akg + cqu * dw) / 255;
    let o = (of * akg + tb * dw) / 255;
    
    0xFF000000 | (m << 16) | (at << 8) | o
}


pub fn nmt() {
    let (z, ac) = super::yn();

    
    super::ah(0, 0, z, ac, LC_);

    
    let cfz = crate::logo_bitmap::AY_ as u32;
    let cfy = crate::logo_bitmap::BL_ as u32;
    let euh = (z / 2).ao(cfz / 2);
    let eui = (ac / 2).ao(cfy / 2);
    crate::logo_bitmap::epd(euh, eui);
}


fn ynm(cx: u32, c: u32, qdp: u32) {
    
    let dq = "TRust-OS";
    let xhv = dq.len() as u32;
    let nk = 8u32;
    let ql = cx.ao(xhv * nk / 2);
    
    
    let bii = (ql / nk) as usize;
    let br = (c / 16) as usize;
    
    
    for (a, r) in dq.bw().cf() {
        let y = ql + (a as u32) * nk;
        afn(r, y as usize, c as usize, IH_, 0xFF000000);
    }
}


fn ynk(cx: u32, c: u32, qdp: u32) {
    let prs = "FAST . SECURE . RELIABLE";
    let xan = prs.len() as u32;
    let nk = 8u32;
    let ql = cx.ao(xan * nk / 2);
    
    for (a, r) in prs.bw().cf() {
        let y = ql + (a as u32) * nk;
        
        afn(r, y as usize, c as usize, NZ_, 0xFF000000);
    }
}


fn afn(r: char, b: usize, c: usize, lp: u32, ei: u32) {
    let ka = super::font::ada(r);
    
    for br in 0..16 {
        let fs = ka[br];
        for bj in 0..8 {
            let s = if (fs >> (7 - bj)) & 1 == 1 { lp } else { ei };
            if s != ei {  
                super::sf((b + bj) as u32, (c + br) as u32, s);
            }
        }
    }
}


fn ynh(z: u32, ac: u32) {
    
    let mut dv: u32 = 12345;
    
    let das = |e: &mut u32| -> u32 {
        *e = e.hx(1103515245).cn(12345);
        (*e >> 16) & 0x7FFF
    };
    
    
    let mft = z / 8;
    
    for _ in 0..200 {
        
        let b = das(&mut dv) % mft;
        let c = das(&mut dv) % ac;
        let hj = (das(&mut dv) % 4) as u8;
        let s = match hj {
            0 => UV_,
            1 => UU_,
            2 => NZ_,
            _ => IH_,
        };
        let r = (b'0' + (das(&mut dv) % 75) as u8) as char;
        afn(r, b as usize, c as usize, s, 0xFF000000);
        
        
        let b = z - mft + das(&mut dv) % mft;
        let c = das(&mut dv) % ac;
        let hj = (das(&mut dv) % 4) as u8;
        let s = match hj {
            0 => UV_,
            1 => UU_,
            2 => NZ_,
            _ => IH_,
        };
        let r = (b'0' + (das(&mut dv) % 75) as u8) as char;
        afn(r, b as usize, c as usize, s, 0xFF000000);
    }
}






const BLL_: u32 = 22;


const LC_: u32 = 0xFF050606;
const CTX_: u32 = 0xFF0A1A0E;
const BGE_: u32 = 0xFF00FF66;
const CTY_: u32 = 0xFF00CC55;
const CUA_: u32 = 0xFF558866;
const CTZ_: u32 = 0xFFCCEEDD;
const EGQ_: u32 = 0xFF00AA44;




pub fn led() {
    let (z, ac) = super::yn();
    if z == 0 || ac == 0 { return; }

    
    super::ah(0, 0, z, ac, LC_);

    
    let cfz = crate::logo_bitmap::AY_ as u32; 
    let cfy = crate::logo_bitmap::BL_ as u32; 
    let euh = (z / 2).ao(cfz / 2);
    let eui = (ac / 2).ao(cfy / 2);
    crate::logo_bitmap::epd(euh, eui);

    
    let lo: u32 = 200;
    let tn: u32 = 8;
    let ajx: u32 = 40;
    let pl = ac - 60;

    
    super::ah(ajx, pl, lo, tn, CTX_);
    
    super::lx(ajx.ao(1), pl.ao(1), lo + 2, tn + 2, UU_);

    
    let tua = "Initializing...";
    let tud = pl + tn + 8;
    for (a, r) in tua.bw().cf() {
        let y = ajx + (a as u32) * 8;
        afn(r, y as usize, tud as usize, CUA_, LC_);
    }
}




pub fn bir(ib: u32, message: &str) {
    let (jym, ac) = super::yn();
    if jym == 0 || ac == 0 { return; }

    
    let lo: u32 = 200;
    let tn: u32 = 8;
    let ajx: u32 = 40;
    let pl = ac - 60;

    
    let li = ((ib + 1) * 100) / BLL_;
    let kwa = (lo * li.v(100)) / 100;

    
    if kwa > 0 {
        super::ah(ajx, pl, kwa, tn, BGE_);
        super::ah(ajx, pl, kwa, 2, CTY_);
    }

    
    let dti = pl + tn + 8;
    super::ah(ajx, dti, 400, 18, LC_);

    
    for (a, r) in message.bw().cf() {
        let y = ajx + (a as u32) * 8;
        afn(r, y as usize, dti as usize, CTZ_, LC_);
    }

    
    let vfx = if li >= 100 {
        "100%"
    } else {
        static mut CIV_: [u8; 5] = [0; 5];
        let k = unsafe { &mut CIV_ };
        let xbp = (li / 10) as u8;
        let osn = (li % 10) as u8;
        if li >= 10 {
            k[0] = b' ';
            k[1] = b'0' + xbp;
            k[2] = b'0' + osn;
            k[3] = b'%';
            k[4] = 0;
        } else {
            k[0] = b' ';
            k[1] = b' ';
            k[2] = b'0' + osn;
            k[3] = b'%';
            k[4] = 0;
        }
        unsafe { core::str::nwj(&k[..4]) }
    };
    let vfy = ajx + lo + 8;
    for (a, r) in vfx.bw().cf() {
        let y = vfy + (a as u32) * 8;
        afn(r, y as usize, pl as usize, BGE_, LC_);
    }
}


pub fn kuv() {
    let (z, ac) = super::yn();
    if z == 0 || ac == 0 { return; }
    
    
    for gu in 0u32..8 {
        let dw = (gu + 1) * 32; 
        let dlr = if dw >= 255 { 0xFF000000 } else {
            
            let wq = 255 - dw;
            let at = (0x05 * wq) / 255;
            0xFF000000 | (at << 8)
        };
        super::ah(0, 0, z, ac, dlr);
        
        
        for _ in 0..2_000_000 { core::hint::hc(); }
    }
    
    
    super::ah(0, 0, z, ac, 0xFF000000);
    
    for _ in 0..3_000_000 { core::hint::hc(); }
}
