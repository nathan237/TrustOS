












#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

#[cfg(target_arch = "aarch64")]
use core::arch::aarch64::*;

use core::sync::atomic::{AtomicBool, Ordering};


static ALG_: AtomicBool = AtomicBool::new(false);



pub fn oei() {
    #[cfg(target_arch = "x86_64")]
    {
        let dr = crate::cpu::bme();
        let tmd = dr.map(|r| r.dog && r.hka).unwrap_or(false);

        if tmd {
            ALG_.store(true, Ordering::Release);
            crate::serial_println!("[SIMD] Jarvis dispatch: AVX2+FMA (8-wide, fused multiply-add)");
        } else {
            crate::serial_println!("[SIMD] Jarvis dispatch: SSE2 (4-wide)");
        }
    }

    #[cfg(target_arch = "aarch64")]
    {
        crate::serial_println!("[SIMD] Jarvis dispatch: NEON (4-wide, fused multiply-add)");
    }

    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    {
        crate::serial_println!("[SIMD] Jarvis dispatch: scalar fallback (no SIMD)");
    }
}


#[inline(always)]
fn gvt() -> bool {
    ALG_.load(Ordering::Relaxed)
}









#[cfg(target_arch = "x86_64")]
#[inline]
fn sak(q: &[f32], o: &[f32], len: usize) -> f32 {
    unsafe {
        let yj = q.fq();
        let bp = o.fq();

        let mut wk = gxi();
        let mut bav = gxi();
        let mut btd = gxi();
        let mut ddu = gxi();

        
        let avm = len / 16;
        for a in 0..avm {
            let ar = a * 16;
            let bfv = zz(yj.add(ar));
            let wu = zz(bp.add(ar));
            wk = aky(wk, axl(bfv, wu));

            let km = zz(yj.add(ar + 4));
            let of = zz(bp.add(ar + 4));
            bav = aky(bav, axl(km, of));

            let oe = zz(yj.add(ar + 8));
            let tb = zz(bp.add(ar + 8));
            btd = aky(btd, axl(oe, tb));

            let vy = zz(yj.add(ar + 12));
            let ajw = zz(bp.add(ar + 12));
            ddu = aky(ddu, axl(vy, ajw));
        }

        
        wk = aky(wk, bav);
        btd = aky(btd, ddu);
        wk = aky(wk, btd);

        
        let uw = avm * 16;
        let bch = (len - uw) / 4;
        for a in 0..bch {
            let l = uw + a * 4;
            let btg = zz(yj.add(l));
            let yu = zz(bp.add(l));
            wk = aky(wk, axl(btg, yu));
        }

        
        let gd = jyd(wk, wk);     
        let sum = aky(wk, gd);          
        let fuq = jye(sum, sum, 1);  
        let es = jya(sum, fuq);       
        let mut result = jyc(es);

        
        let zm = uw + bch * 4;
        for a in zm..len {
            result += *yj.add(a) * *bp.add(a);
        }

        result
    }
}









#[cfg(target_arch = "aarch64")]
#[inline]
fn saj(q: &[f32], o: &[f32], len: usize) -> f32 {
    unsafe {
        let yj = q.fq();
        let bp = o.fq();

        let mut wk = dxk(0.0);
        let mut bav = dxk(0.0);
        let mut btd = dxk(0.0);
        let mut ddu = dxk(0.0);

        
        let avm = len / 16;
        for a in 0..avm {
            let ar = a * 16;
            let bfv = aba(yj.add(ar));
            let wu = aba(bp.add(ar));
            wk = bis(wk, bfv, wu);

            let km = aba(yj.add(ar + 4));
            let of = aba(bp.add(ar + 4));
            bav = bis(bav, km, of);

            let oe = aba(yj.add(ar + 8));
            let tb = aba(bp.add(ar + 8));
            btd = bis(btd, oe, tb);

            let vy = aba(yj.add(ar + 12));
            let ajw = aba(bp.add(ar + 12));
            ddu = bis(ddu, vy, ajw);
        }

        
        wk = igf(wk, bav);
        btd = igf(btd, ddu);
        wk = igf(wk, btd);

        
        let uw = avm * 16;
        let bch = (len - uw) / 4;
        for a in 0..bch {
            let l = uw + a * 4;
            let btg = aba(yj.add(l));
            let yu = aba(bp.add(l));
            wk = bis(wk, btg, yu);
        }

        
        let mut result = xqg(wk);

        
        let zm = uw + bch * 4;
        for a in zm..len {
            result += *yj.add(a) * *bp.add(a);
        }

        result
    }
}












#[cfg(target_arch = "x86_64")]
pub fn ami(bd: &mut [f32], d: &[f32], b: &[f32], ec: usize, lk: usize) {
    for m in 0..lk {
        let ar = m * ec;
        bd[m] = sak(&d[ar..ar + ec], b, ec);
    }
}


#[cfg(target_arch = "aarch64")]
pub fn ami(bd: &mut [f32], d: &[f32], b: &[f32], ec: usize, lk: usize) {
    for m in 0..lk {
        bd[m] = saj(&d[m * ec..m * ec + ec], b, ec);
    }
}


#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
pub fn ami(bd: &mut [f32], d: &[f32], b: &[f32], ec: usize, lk: usize) {
    for m in 0..lk {
        let mut sum = 0.0f32;
        let ar = m * ec;
        for r in 0..ec {
            sum += d[ar + r] * b[r];
        }
        bd[m] = sum;
    }
}













#[cfg(target_arch = "x86_64")]
pub fn dta(bd: &mut [f32], d: &[f32], c: &[f32], ec: usize, lk: usize) {
    if gvt() {
        unsafe { ukr(bd, d, c, ec, lk); }
        return;
    }
    
    for p in bd[..ec].el() { *p = 0.0; }

    unsafe {
        let zd = d.fq();
        let op = bd.mw();

        for m in 0..lk {
            let bdh = c[m];
            if bdh == 0.0 { continue; } 

            let akx = iid(bdh);
            let ar = m * ec;

            
            let avm = ec / 16;
            for a in 0..avm {
                let l = ar + a * 16;
                let rt = a * 16;

                let cnv = zz(zd.add(l));
                let dkc = zz(op.add(rt));
                bpo(op.add(rt), aky(dkc, axl(cnv, akx)));

                let blt = zz(zd.add(l + 4));
                let csy = zz(op.add(rt + 4));
                bpo(op.add(rt + 4), aky(csy, axl(blt, akx)));

                let bfs = zz(zd.add(l + 8));
                let csz = zz(op.add(rt + 8));
                bpo(op.add(rt + 8), aky(csz, axl(bfs, akx)));

                let bxu = zz(zd.add(l + 12));
                let cta = zz(op.add(rt + 12));
                bpo(op.add(rt + 12), aky(cta, axl(bxu, akx)));
            }

            
            let uw = avm * 16;
            let bch = (ec - uw) / 4;
            for a in 0..bch {
                let l = ar + uw + a * 4;
                let rt = uw + a * 4;
                let bxx = zz(zd.add(l));
                let dki = zz(op.add(rt));
                bpo(op.add(rt), aky(dki, axl(bxx, akx)));
            }

            
            let zm = uw + bch * 4;
            for r in zm..ec {
                *op.add(r) += *zd.add(ar + r) * bdh;
            }
        }
    }
}


#[cfg(target_arch = "aarch64")]
pub fn dta(bd: &mut [f32], d: &[f32], c: &[f32], ec: usize, lk: usize) {
    for p in bd[..ec].el() { *p = 0.0; }
    unsafe {
        let zd = d.fq();
        let op = bd.mw();
        for m in 0..lk {
            let bdh = c[m];
            if bdh == 0.0 { continue; }
            let akx = dxk(bdh);
            let ar = m * ec;
            let avm = ec / 16;
            for a in 0..avm {
                let l = ar + a * 16;
                let rt = a * 16;
                let cnv = aba(zd.add(l));
                let dkc = aba(op.add(rt));
                bsv(op.add(rt), bis(dkc, cnv, akx));
                let blt = aba(zd.add(l + 4));
                let csy = aba(op.add(rt + 4));
                bsv(op.add(rt + 4), bis(csy, blt, akx));
                let bfs = aba(zd.add(l + 8));
                let csz = aba(op.add(rt + 8));
                bsv(op.add(rt + 8), bis(csz, bfs, akx));
                let bxu = aba(zd.add(l + 12));
                let cta = aba(op.add(rt + 12));
                bsv(op.add(rt + 12), bis(cta, bxu, akx));
            }
            let uw = avm * 16;
            let bch = (ec - uw) / 4;
            for a in 0..bch {
                let l = ar + uw + a * 4;
                let rt = uw + a * 4;
                let bxx = aba(zd.add(l));
                let dki = aba(op.add(rt));
                bsv(op.add(rt), bis(dki, bxx, akx));
            }
            let zm = uw + bch * 4;
            for r in zm..ec {
                *op.add(r) += *zd.add(ar + r) * bdh;
            }
        }
    }
}


#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
pub fn dta(bd: &mut [f32], d: &[f32], c: &[f32], ec: usize, lk: usize) {
    for p in bd[..ec].el() { *p = 0.0; }
    for m in 0..lk {
        let ar = m * ec;
        for r in 0..ec {
            bd[r] += d[ar + r] * c[m];
        }
    }
}



#[cfg(target_arch = "x86_64")]
pub fn euq(bd: &mut [f32], d: &[f32], c: &[f32], ec: usize, lk: usize) {
    if gvt() {
        unsafe { ukq(bd, d, c, ec, lk); }
        return;
    }
    unsafe {
        let zd = d.fq();
        let op = bd.mw();

        for m in 0..lk {
            let bdh = c[m];
            if bdh == 0.0 { continue; }

            let akx = iid(bdh);
            let ar = m * ec;

            let avm = ec / 16;
            for a in 0..avm {
                let l = ar + a * 16;
                let rt = a * 16;

                let cnv = zz(zd.add(l));
                let dkc = zz(op.add(rt));
                bpo(op.add(rt), aky(dkc, axl(cnv, akx)));

                let blt = zz(zd.add(l + 4));
                let csy = zz(op.add(rt + 4));
                bpo(op.add(rt + 4), aky(csy, axl(blt, akx)));

                let bfs = zz(zd.add(l + 8));
                let csz = zz(op.add(rt + 8));
                bpo(op.add(rt + 8), aky(csz, axl(bfs, akx)));

                let bxu = zz(zd.add(l + 12));
                let cta = zz(op.add(rt + 12));
                bpo(op.add(rt + 12), aky(cta, axl(bxu, akx)));
            }

            let uw = avm * 16;
            let bch = (ec - uw) / 4;
            for a in 0..bch {
                let l = ar + uw + a * 4;
                let rt = uw + a * 4;
                let bxx = zz(zd.add(l));
                let dki = zz(op.add(rt));
                bpo(op.add(rt), aky(dki, axl(bxx, akx)));
            }

            let zm = uw + bch * 4;
            for r in zm..ec {
                *op.add(r) += *zd.add(ar + r) * bdh;
            }
        }
    }
}


#[cfg(target_arch = "aarch64")]
pub fn euq(bd: &mut [f32], d: &[f32], c: &[f32], ec: usize, lk: usize) {
    unsafe {
        let zd = d.fq();
        let op = bd.mw();
        for m in 0..lk {
            let bdh = c[m];
            if bdh == 0.0 { continue; }
            let akx = dxk(bdh);
            let ar = m * ec;
            let avm = ec / 16;
            for a in 0..avm {
                let l = ar + a * 16;
                let rt = a * 16;
                let cnv = aba(zd.add(l));
                let dkc = aba(op.add(rt));
                bsv(op.add(rt), bis(dkc, cnv, akx));
                let blt = aba(zd.add(l + 4));
                let csy = aba(op.add(rt + 4));
                bsv(op.add(rt + 4), bis(csy, blt, akx));
                let bfs = aba(zd.add(l + 8));
                let csz = aba(op.add(rt + 8));
                bsv(op.add(rt + 8), bis(csz, bfs, akx));
                let bxu = aba(zd.add(l + 12));
                let cta = aba(op.add(rt + 12));
                bsv(op.add(rt + 12), bis(cta, bxu, akx));
            }
            let uw = avm * 16;
            let bch = (ec - uw) / 4;
            for a in 0..bch {
                let l = ar + uw + a * 4;
                let rt = uw + a * 4;
                let bxx = aba(zd.add(l));
                let dki = aba(op.add(rt));
                bsv(op.add(rt), bis(dki, bxx, akx));
            }
            let zm = uw + bch * 4;
            for r in zm..ec {
                *op.add(r) += *zd.add(ar + r) * bdh;
            }
        }
    }
}

#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
pub fn euq(bd: &mut [f32], d: &[f32], c: &[f32], ec: usize, lk: usize) {
    for m in 0..lk {
        let ar = m * ec;
        for r in 0..ec {
            bd[r] += d[ar + r] * c[m];
        }
    }
}









#[cfg(target_arch = "x86_64")]
pub fn ctd(aix: &mut [f32], bg: &[f32], b: &[f32], ec: usize, lk: usize) {
    if gvt() {
        unsafe { uzz(aix, bg, b, ec, lk); }
        return;
    }
    unsafe {
        let akc = aix.mw();
        let zp = b.fq();

        for m in 0..lk {
            let dgw = bg[m];
            if dgw == 0.0 { continue; }

            let bud = iid(dgw);
            let ar = m * ec;

            
            let avm = ec / 16;
            for a in 0..avm {
                let l = ar + a * 16;
                let aze = a * 16;

                let fy = zz(zp.add(aze));
                let cdx = zz(akc.add(l));
                bpo(akc.add(l), aky(cdx, axl(bud, fy)));

                let dn = zz(zp.add(aze + 4));
                let apo = zz(akc.add(l + 4));
                bpo(akc.add(l + 4), aky(apo, axl(bud, dn)));

                let hy = zz(zp.add(aze + 8));
                let us = zz(akc.add(l + 8));
                bpo(akc.add(l + 8), aky(us, axl(bud, hy)));

                let ajr = zz(zp.add(aze + 12));
                let cdy = zz(akc.add(l + 12));
                bpo(akc.add(l + 12), aky(cdy, axl(bud, ajr)));
            }

            
            let uw = avm * 16;
            let bch = (ec - uw) / 4;
            for a in 0..bch {
                let l = ar + uw + a * 4;
                let aze = uw + a * 4;
                let dnm = zz(zp.add(aze));
                let hhg = zz(akc.add(l));
                bpo(akc.add(l), aky(hhg, axl(bud, dnm)));
            }

            
            let zm = uw + bch * 4;
            for r in zm..ec {
                *akc.add(ar + r) += dgw * *zp.add(r);
            }
        }
    }
}


#[cfg(target_arch = "aarch64")]
pub fn ctd(aix: &mut [f32], bg: &[f32], b: &[f32], ec: usize, lk: usize) {
    unsafe {
        let akc = aix.mw();
        let zp = b.fq();
        for m in 0..lk {
            let dgw = bg[m];
            if dgw == 0.0 { continue; }
            let bud = dxk(dgw);
            let ar = m * ec;
            let avm = ec / 16;
            for a in 0..avm {
                let l = ar + a * 16;
                let aze = a * 16;
                let fy = aba(zp.add(aze));
                let cdx = aba(akc.add(l));
                bsv(akc.add(l), bis(cdx, bud, fy));
                let dn = aba(zp.add(aze + 4));
                let apo = aba(akc.add(l + 4));
                bsv(akc.add(l + 4), bis(apo, bud, dn));
                let hy = aba(zp.add(aze + 8));
                let us = aba(akc.add(l + 8));
                bsv(akc.add(l + 8), bis(us, bud, hy));
                let ajr = aba(zp.add(aze + 12));
                let cdy = aba(akc.add(l + 12));
                bsv(akc.add(l + 12), bis(cdy, bud, ajr));
            }
            let uw = avm * 16;
            let bch = (ec - uw) / 4;
            for a in 0..bch {
                let l = ar + uw + a * 4;
                let aze = uw + a * 4;
                let dnm = aba(zp.add(aze));
                let hhg = aba(akc.add(l));
                bsv(akc.add(l), bis(hhg, bud, dnm));
            }
            let zm = uw + bch * 4;
            for r in zm..ec {
                *akc.add(ar + r) += dgw * *zp.add(r);
            }
        }
    }
}


#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
pub fn ctd(aix: &mut [f32], bg: &[f32], b: &[f32], ec: usize, lk: usize) {
    for m in 0..lk {
        let ar = m * ec;
        for r in 0..ec {
            aix[ar + r] += bg[m] * b[r];
        }
    }
}









#[cfg(target_arch = "x86_64")]
pub fn cbl(bd: &mut [f32], b: &[f32], amz: &[f32]) -> f32 {
    let bo = b.len();

    
    let rv = unsafe {
        let zp = b.fq();
        let mut wk = gxi();
        let mut bav = gxi();

        let byq = bo / 8;
        for a in 0..byq {
            let ar = a * 8;
            let abk = zz(zp.add(ar));
            wk = aky(wk, axl(abk, abk));
            let agy = zz(zp.add(ar + 4));
            bav = aky(bav, axl(agy, agy));
        }
        wk = aky(wk, bav);

        
        let uw = byq * 8;
        for a in (uw..bo).akt(4) {
            if a + 4 <= bo {
                let p = zz(zp.add(a));
                wk = aky(wk, axl(p, p));
            }
        }

        let gd = jyd(wk, wk);
        let sum = aky(wk, gd);
        let fuq = jye(sum, sum, 1);
        let es = jya(sum, fuq);
        let mut result = jyc(es);

        
        let zm = (bo / 4) * 4;
        for a in zm..bo {
            result += *zp.add(a) * *zp.add(a);
        }
        result
    };

    let bfd = super::backprop::ccw(rv / bo as f32 + super::model::HC_);
    let bva = 1.0 / bfd;

    
    unsafe {
        let zp = b.fq();
        let zd = amz.fq();
        let op = bd.mw();
        let hol = iid(bva);

        let bbe = bo / 4;
        for a in 0..bbe {
            let dz = a * 4;
            let dnm = zz(zp.add(dz));
            let bxx = zz(zd.add(dz));
            let dtp = axl(dnm, hol);
            bpo(op.add(dz), axl(dtp, bxx));
        }

        
        let zm = bbe * 4;
        for a in zm..bo {
            *op.add(a) = *zp.add(a) * bva * *zd.add(a);
        }
    }

    bfd
}


#[cfg(target_arch = "aarch64")]
pub fn cbl(bd: &mut [f32], b: &[f32], amz: &[f32]) -> f32 {
    let bo = b.len();
    let rv = unsafe {
        let zp = b.fq();
        let mut wk = dxk(0.0);
        let mut bav = dxk(0.0);
        let byq = bo / 8;
        for a in 0..byq {
            let ar = a * 8;
            let abk = aba(zp.add(ar));
            wk = bis(wk, abk, abk);
            let agy = aba(zp.add(ar + 4));
            bav = bis(bav, agy, agy);
        }
        wk = igf(wk, bav);
        let mut result = xqg(wk);
        let zm = (bo / 4) * 4;
        for a in zm..bo {
            result += *zp.add(a) * *zp.add(a);
        }
        result
    };
    let bfd = super::backprop::ccw(rv / bo as f32 + super::model::HC_);
    let bva = 1.0 / bfd;
    unsafe {
        let zp = b.fq();
        let zd = amz.fq();
        let op = bd.mw();
        let hol = dxk(bva);
        let bbe = bo / 4;
        for a in 0..bbe {
            let dz = a * 4;
            let dnm = aba(zp.add(dz));
            let bxx = aba(zd.add(dz));
            let dtp = pyq(dnm, hol);
            bsv(op.add(dz), pyq(dtp, bxx));
        }
        let zm = bbe * 4;
        for a in zm..bo {
            *op.add(a) = *zp.add(a) * bva * *zd.add(a);
        }
    }
    bfd
}


#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
pub fn cbl(bd: &mut [f32], b: &[f32], amz: &[f32]) -> f32 {
    let bo = b.len();
    let mut rv = 0.0f32;
    for &p in b { rv += p * p; }
    let bfd = super::backprop::ccw(rv / bo as f32 + super::model::HC_);
    let bva = 1.0 / bfd;
    for a in 0..bo {
        bd[a] = b[a] * bva * amz[a];
    }
    bfd
}






#[cfg(target_arch = "x86_64")]
pub fn pxz(bd: &mut [f32], q: &[f32], o: &[f32], len: usize) {
    if gvt() {
        unsafe { xqw(bd, q, o, len); }
        return;
    }
    unsafe {
        let yj = q.fq();
        let bp = o.fq();
        let op = bd.mw();

        let bbe = len / 4;
        for a in 0..bbe {
            let dz = a * 4;
            let btg = zz(yj.add(dz));
            let yu = zz(bp.add(dz));
            bpo(op.add(dz), aky(btg, yu));
        }

        let zm = bbe * 4;
        for a in zm..len {
            *op.add(a) = *yj.add(a) + *bp.add(a);
        }
    }
}

#[cfg(target_arch = "aarch64")]
pub fn pxz(bd: &mut [f32], q: &[f32], o: &[f32], len: usize) {
    unsafe {
        let yj = q.fq();
        let bp = o.fq();
        let op = bd.mw();
        let bbe = len / 4;
        for a in 0..bbe {
            let dz = a * 4;
            let btg = aba(yj.add(dz));
            let yu = aba(bp.add(dz));
            bsv(op.add(dz), igf(btg, yu));
        }
        let zm = bbe * 4;
        for a in zm..len {
            *op.add(a) = *yj.add(a) + *bp.add(a);
        }
    }
}

#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
pub fn pxz(bd: &mut [f32], q: &[f32], o: &[f32], len: usize) {
    for a in 0..len { bd[a] = q[a] + o[a]; }
}


#[cfg(target_arch = "x86_64")]
pub fn pya(q: &mut [f32], o: &[f32], len: usize) {
    if gvt() {
        unsafe { xqx(q, o, len); }
        return;
    }
    unsafe {
        let yj = q.mw();
        let bp = o.fq();

        let bbe = len / 4;
        for a in 0..bbe {
            let dz = a * 4;
            let btg = zz(yj.add(dz));
            let yu = zz(bp.add(dz));
            bpo(yj.add(dz), aky(btg, yu));
        }

        let zm = bbe * 4;
        for a in zm..len {
            *yj.add(a) += *bp.add(a);
        }
    }
}

#[cfg(target_arch = "aarch64")]
pub fn pya(q: &mut [f32], o: &[f32], len: usize) {
    unsafe {
        let yj = q.mw();
        let bp = o.fq();
        let bbe = len / 4;
        for a in 0..bbe {
            let dz = a * 4;
            let btg = aba(yj.add(dz));
            let yu = aba(bp.add(dz));
            bsv(yj.add(dz), igf(btg, yu));
        }
        let zm = bbe * 4;
        for a in zm..len {
            *yj.add(a) += *bp.add(a);
        }
    }
}

#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
pub fn pya(q: &mut [f32], o: &[f32], len: usize) {
    for a in 0..len { q[a] += o[a]; }
}


#[cfg(target_arch = "x86_64")]
pub fn pyb(q: &mut [f32], bfe: f32, len: usize) {
    if gvt() {
        unsafe { xqy(q, bfe, len); }
        return;
    }
    unsafe {
        let yj = q.mw();
        let bxk = iid(bfe);

        let bbe = len / 4;
        for a in 0..bbe {
            let dz = a * 4;
            let p = zz(yj.add(dz));
            bpo(yj.add(dz), axl(p, bxk));
        }

        let zm = bbe * 4;
        for a in zm..len {
            *yj.add(a) *= bfe;
        }
    }
}

#[cfg(target_arch = "aarch64")]
pub fn pyb(q: &mut [f32], bfe: f32, len: usize) {
    unsafe {
        let yj = q.mw();
        let bxk = dxk(bfe);
        let bbe = len / 4;
        for a in 0..bbe {
            let dz = a * 4;
            let p = aba(yj.add(dz));
            bsv(yj.add(dz), pyq(p, bxk));
        }
        let zm = bbe * 4;
        for a in zm..len {
            *yj.add(a) *= bfe;
        }
    }
}

#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
pub fn pyb(q: &mut [f32], bfe: f32, len: usize) {
    for a in 0..len { q[a] *= bfe; }
}















#[cfg(target_arch = "x86_64")]
#[dmi(aiy = "avx2,fma")]
#[inline]
unsafe fn sag(q: &[f32], o: &[f32], len: usize) -> f32 {
    let yj = q.fq();
    let bp = o.fq();

    let mut wk = gxh();
    let mut bav = gxh();
    let mut btd = gxh();
    let mut ddu = gxh();

    
    let deu = len / 32;
    for a in 0..deu {
        let ar = a * 32;
        let bfv = zy(yj.add(ar));
        let wu = zy(bp.add(ar));
        wk = bdi(bfv, wu, wk);

        let km = zy(yj.add(ar + 8));
        let of = zy(bp.add(ar + 8));
        bav = bdi(km, of, bav);

        let oe = zy(yj.add(ar + 16));
        let tb = zy(bp.add(ar + 16));
        btd = bdi(oe, tb, btd);

        let vy = zy(yj.add(ar + 24));
        let ajw = zy(bp.add(ar + 24));
        ddu = bdi(vy, ajw, ddu);
    }

    
    wk = gxg(wk, bav);
    btd = gxg(btd, ddu);
    wk = gxg(wk, btd);

    
    let uw = deu * 32;
    let cmf = (len - uw) / 8;
    for a in 0..cmf {
        let l = uw + a * 8;
        let btg = zy(yj.add(l));
        let yu = zy(bp.add(l));
        wk = bdi(btg, yu, wk);
    }

    
    let lcb = qcp(wk, 1);
    let ljd = qcn(wk);
    let fvu = aky(ljd, lcb);
    let gd = jyd(fvu, fvu);
    let sum = aky(fvu, gd);
    let fuq = jye(sum, sum, 1);
    let es = jya(sum, fuq);
    let mut result = jyc(es);

    
    let zm = uw + cmf * 8;
    for a in zm..len {
        result += *yj.add(a) * *bp.add(a);
    }

    result
}


#[cfg(target_arch = "x86_64")]
#[dmi(aiy = "avx2,fma")]
unsafe fn zci(bd: &mut [f32], d: &[f32], b: &[f32], ec: usize, lk: usize) {
    for m in 0..lk {
        let ar = m * ec;
        bd[m] = sag(&d[ar..ar + ec], b, ec);
    }
}


#[cfg(target_arch = "x86_64")]
#[dmi(aiy = "avx2,fma")]
unsafe fn ukr(bd: &mut [f32], d: &[f32], c: &[f32], ec: usize, lk: usize) {
    for p in bd[..ec].el() { *p = 0.0; }

    let zd = d.fq();
    let op = bd.mw();

    for m in 0..lk {
        let bdh = c[m];
        if bdh == 0.0 { continue; }

        let akx = iib(bdh);
        let ar = m * ec;

        
        let deu = ec / 32;
        for a in 0..deu {
            let l = ar + a * 32;
            let rt = a * 32;

            let dkc = zy(op.add(rt));
            let cnv = zy(zd.add(l));
            bpn(op.add(rt), bdi(cnv, akx, dkc));

            let csy = zy(op.add(rt + 8));
            let blt = zy(zd.add(l + 8));
            bpn(op.add(rt + 8), bdi(blt, akx, csy));

            let csz = zy(op.add(rt + 16));
            let bfs = zy(zd.add(l + 16));
            bpn(op.add(rt + 16), bdi(bfs, akx, csz));

            let cta = zy(op.add(rt + 24));
            let bxu = zy(zd.add(l + 24));
            bpn(op.add(rt + 24), bdi(bxu, akx, cta));
        }

        
        let uw = deu * 32;
        let cmf = (ec - uw) / 8;
        for a in 0..cmf {
            let l = ar + uw + a * 8;
            let rt = uw + a * 8;
            let dki = zy(op.add(rt));
            let bxx = zy(zd.add(l));
            bpn(op.add(rt), bdi(bxx, akx, dki));
        }

        
        let zm = uw + cmf * 8;
        for r in zm..ec {
            *op.add(r) += *zd.add(ar + r) * bdh;
        }
    }
}


#[cfg(target_arch = "x86_64")]
#[dmi(aiy = "avx2,fma")]
unsafe fn ukq(bd: &mut [f32], d: &[f32], c: &[f32], ec: usize, lk: usize) {
    let zd = d.fq();
    let op = bd.mw();

    for m in 0..lk {
        let bdh = c[m];
        if bdh == 0.0 { continue; }

        let akx = iib(bdh);
        let ar = m * ec;

        let deu = ec / 32;
        for a in 0..deu {
            let l = ar + a * 32;
            let rt = a * 32;

            let dkc = zy(op.add(rt));
            let cnv = zy(zd.add(l));
            bpn(op.add(rt), bdi(cnv, akx, dkc));

            let csy = zy(op.add(rt + 8));
            let blt = zy(zd.add(l + 8));
            bpn(op.add(rt + 8), bdi(blt, akx, csy));

            let csz = zy(op.add(rt + 16));
            let bfs = zy(zd.add(l + 16));
            bpn(op.add(rt + 16), bdi(bfs, akx, csz));

            let cta = zy(op.add(rt + 24));
            let bxu = zy(zd.add(l + 24));
            bpn(op.add(rt + 24), bdi(bxu, akx, cta));
        }

        let uw = deu * 32;
        let cmf = (ec - uw) / 8;
        for a in 0..cmf {
            let l = ar + uw + a * 8;
            let rt = uw + a * 8;
            let dki = zy(op.add(rt));
            let bxx = zy(zd.add(l));
            bpn(op.add(rt), bdi(bxx, akx, dki));
        }

        let zm = uw + cmf * 8;
        for r in zm..ec {
            *op.add(r) += *zd.add(ar + r) * bdh;
        }
    }
}


#[cfg(target_arch = "x86_64")]
#[dmi(aiy = "avx2,fma")]
unsafe fn uzz(aix: &mut [f32], bg: &[f32], b: &[f32], ec: usize, lk: usize) {
    let akc = aix.mw();
    let zp = b.fq();

    for m in 0..lk {
        let dgw = bg[m];
        if dgw == 0.0 { continue; }

        let bud = iib(dgw);
        let ar = m * ec;

        
        let deu = ec / 32;
        for a in 0..deu {
            let l = ar + a * 32;
            let aze = a * 32;

            let fy = zy(zp.add(aze));
            let cdx = zy(akc.add(l));
            bpn(akc.add(l), bdi(bud, fy, cdx));

            let dn = zy(zp.add(aze + 8));
            let apo = zy(akc.add(l + 8));
            bpn(akc.add(l + 8), bdi(bud, dn, apo));

            let hy = zy(zp.add(aze + 16));
            let us = zy(akc.add(l + 16));
            bpn(akc.add(l + 16), bdi(bud, hy, us));

            let ajr = zy(zp.add(aze + 24));
            let cdy = zy(akc.add(l + 24));
            bpn(akc.add(l + 24), bdi(bud, ajr, cdy));
        }

        
        let uw = deu * 32;
        let cmf = (ec - uw) / 8;
        for a in 0..cmf {
            let l = ar + uw + a * 8;
            let aze = uw + a * 8;
            let dnm = zy(zp.add(aze));
            let hhg = zy(akc.add(l));
            bpn(akc.add(l), bdi(bud, dnm, hhg));
        }

        
        let zm = uw + cmf * 8;
        for r in zm..ec {
            *akc.add(ar + r) += dgw * *zp.add(r);
        }
    }
}


#[cfg(target_arch = "x86_64")]
#[dmi(aiy = "avx2,fma")]
unsafe fn zkg(bd: &mut [f32], b: &[f32], amz: &[f32]) -> f32 {
    let bo = b.len();
    let zp = b.fq();

    
    let mut wk = gxh();
    let mut bav = gxh();

    let avm = bo / 16;
    for a in 0..avm {
        let ar = a * 16;
        let abk = zy(zp.add(ar));
        wk = bdi(abk, abk, wk);
        let agy = zy(zp.add(ar + 8));
        bav = bdi(agy, agy, bav);
    }
    wk = gxg(wk, bav);

    
    let uw = avm * 16;
    let cmf = (bo - uw) / 8;
    for a in 0..cmf {
        let p = zy(zp.add(uw + a * 8));
        wk = bdi(p, p, wk);
    }

    
    let lcb = qcp(wk, 1);
    let ljd = qcn(wk);
    let fvu = aky(ljd, lcb);
    let gd = jyd(fvu, fvu);
    let sum = aky(fvu, gd);
    let fuq = jye(sum, sum, 1);
    let es = jya(sum, fuq);
    let mut rv = jyc(es);

    
    let zm = uw + cmf * 8;
    for a in zm..bo {
        rv += *zp.add(a) * *zp.add(a);
    }

    let bfd = super::backprop::ccw(rv / bo as f32 + super::model::HC_);
    let bva = 1.0 / bfd;

    
    let zd = amz.fq();
    let op = bd.mw();
    let hol = iib(bva);

    let byq = bo / 8;
    for a in 0..byq {
        let dz = a * 8;
        let dnm = zy(zp.add(dz));
        let bxx = zy(zd.add(dz));
        let dtp = msm(dnm, hol);
        bpn(op.add(dz), msm(dtp, bxx));
    }

    
    let wdi = byq * 8;
    for a in wdi..bo {
        *op.add(a) = *zp.add(a) * bva * *zd.add(a);
    }

    bfd
}


#[cfg(target_arch = "x86_64")]
#[dmi(aiy = "avx2")]
unsafe fn xqw(bd: &mut [f32], q: &[f32], o: &[f32], len: usize) {
    let yj = q.fq();
    let bp = o.fq();
    let op = bd.mw();

    let byq = len / 8;
    for a in 0..byq {
        let dz = a * 8;
        let btg = zy(yj.add(dz));
        let yu = zy(bp.add(dz));
        bpn(op.add(dz), gxg(btg, yu));
    }

    let zm = byq * 8;
    for a in zm..len {
        *op.add(a) = *yj.add(a) + *bp.add(a);
    }
}


#[cfg(target_arch = "x86_64")]
#[dmi(aiy = "avx2")]
unsafe fn xqx(q: &mut [f32], o: &[f32], len: usize) {
    let yj = q.mw();
    let bp = o.fq();

    let byq = len / 8;
    for a in 0..byq {
        let dz = a * 8;
        let btg = zy(yj.add(dz));
        let yu = zy(bp.add(dz));
        bpn(yj.add(dz), gxg(btg, yu));
    }

    let zm = byq * 8;
    for a in zm..len {
        *yj.add(a) += *bp.add(a);
    }
}


#[cfg(target_arch = "x86_64")]
#[dmi(aiy = "avx2")]
unsafe fn xqy(q: &mut [f32], bfe: f32, len: usize) {
    let yj = q.mw();
    let bxk = iib(bfe);

    let byq = len / 8;
    for a in 0..byq {
        let dz = a * 8;
        let p = zy(yj.add(dz));
        bpn(yj.add(dz), msm(p, bxk));
    }

    let zm = byq * 8;
    for a in zm..len {
        *yj.add(a) *= bfe;
    }
}
