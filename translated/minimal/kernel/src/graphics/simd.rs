








#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;







#[cfg(target_arch = "x86_64")]
#[inline]
pub unsafe fn bed(cs: *mut u32, az: usize, s: u32) {
    if az == 0 { return; }
    
    
    let dfb = els(s as i32);
    
    let mut ptr = cs;
    let mut ia = az;
    
    
    let muo = (ptr as usize) & 15; 
    if muo != 0 {
        let vik = ((16 - muo) / 4).v(ia);
        for _ in 0..vik {
            *ptr = s;
            ptr = ptr.add(1);
            ia -= 1;
        }
    }
    
    
    while ia >= 16 {
        iif(ptr as *mut acb, dfb);
        iif(ptr.add(4) as *mut acb, dfb);
        iif(ptr.add(8) as *mut acb, dfb);
        iif(ptr.add(12) as *mut acb, dfb);
        ptr = ptr.add(16);
        ia -= 16;
    }
    
    
    while ia >= 4 {
        iif(ptr as *mut acb, dfb);
        ptr = ptr.add(4);
        ia -= 4;
    }
    
    
    for _ in 0..ia {
        *ptr = s;
        ptr = ptr.add(1);
    }
}


#[cfg(target_arch = "x86_64")]
#[inline]
pub unsafe fn dpd(cs: *mut u32, cy: *const u32, az: usize) {
    if az == 0 { return; }
    
    let mut alc = cs;
    let mut aob = cy;
    let mut ia = az;
    
    
    while ia >= 16 {
        let abk = byb(aob as *const acb);
        let agy = byb(aob.add(4) as *const acb);
        let apg = byb(aob.add(8) as *const acb);
        let bdf = byb(aob.add(12) as *const acb);
        
        ccs(alc as *mut acb, abk);
        ccs(alc.add(4) as *mut acb, agy);
        ccs(alc.add(8) as *mut acb, apg);
        ccs(alc.add(12) as *mut acb, bdf);
        
        aob = aob.add(16);
        alc = alc.add(16);
        ia -= 16;
    }
    
    
    while ia >= 4 {
        let p = byb(aob as *const acb);
        ccs(alc as *mut acb, p);
        aob = aob.add(4);
        alc = alc.add(4);
        ia -= 4;
    }
    
    
    for _ in 0..ia {
        *alc = *aob;
        aob = aob.add(1);
        alc = alc.add(1);
    }
}






















#[cfg(target_arch = "x86_64")]
#[inline]
pub unsafe fn iph(cs: *mut u32, cy: *const u32, az: usize) {
    if az == 0 { return; }

    let hhb = cs as *mut u64;
    let pmp = cy as *const u64;
    let evx = az / 2;  
    let mut a = 0usize;

    
    while a + 8 <= evx {
        let e = pmp.add(a);
        let bc = hhb.add(a);
        let abk = core::ptr::md(e);
        let agy = core::ptr::md(e.add(1));
        let apg = core::ptr::md(e.add(2));
        let bdf = core::ptr::md(e.add(3));
        let cnq = core::ptr::md(e.add(4));
        let pxu = core::ptr::md(e.add(5));
        let pxv = core::ptr::md(e.add(6));
        let jvd = core::ptr::md(e.add(7));
        core::arch::asm!(
            "movnti [{d}], {v0}",
            "movnti [{d} + 8], {v1}",
            "movnti [{d} + 16], {v2}",
            "movnti [{d} + 24], {v3}",
            "movnti [{d} + 32], {v4}",
            "movnti [{d} + 40], {v5}",
            "movnti [{d} + 48], {v6}",
            "movnti [{d} + 56], {v7}",
            bc = in(reg) bc,
            abk = in(reg) abk,
            agy = in(reg) agy,
            apg = in(reg) apg,
            bdf = in(reg) bdf,
            cnq = in(reg) cnq,
            pxu = in(reg) pxu,
            pxv = in(reg) pxv,
            jvd = in(reg) jvd,
            options(nostack),
        );
        a += 8;
    }

    
    while a < evx {
        let p = core::ptr::md(pmp.add(a));
        core::arch::asm!(
            "movnti [{d}], {v}",
            bc = in(reg) hhb.add(a),
            p = in(reg) p,
            options(nostack),
        );
        a += 1;
    }

    
    if az & 1 != 0 {
        *cs.add(az - 1) = *cy.add(az - 1);
    }

    
    core::arch::asm!("sfence", options(nostack));
}


#[cfg(target_arch = "x86_64")]
#[inline]
pub unsafe fn yqo(cs: *mut u32, az: usize, s: u32) {
    if az == 0 { return; }

    let nex = (s as u64) | ((s as u64) << 32);
    let hhb = cs as *mut u64;
    let evx = az / 2;
    let mut a = 0usize;

    while a + 8 <= evx {
        let bc = hhb.add(a);
        core::arch::asm!(
            "movnti [{d}], {v}",
            "movnti [{d} + 8], {v}",
            "movnti [{d} + 16], {v}",
            "movnti [{d} + 24], {v}",
            "movnti [{d} + 32], {v}",
            "movnti [{d} + 40], {v}",
            "movnti [{d} + 48], {v}",
            "movnti [{d} + 56], {v}",
            bc = in(reg) bc,
            p = in(reg) nex,
            options(nostack),
        );
        a += 8;
    }

    while a < evx {
        core::arch::asm!(
            "movnti [{d}], {v}",
            bc = in(reg) hhb.add(a),
            p = in(reg) nex,
            options(nostack),
        );
        a += 1;
    }

    if az & 1 != 0 {
        *cs.add(az - 1) = s;
    }

    core::arch::asm!("sfence", options(nostack));
}








#[cfg(target_arch = "x86_64")]
#[inline]
pub unsafe fn kdu(cs: *mut u32, cy: *const u32, az: usize) {
    let mut alc = cs;
    let mut aob = cy;
    let mut ia = az;

    let ajs = mso();

    
    while ia >= 4 {
        let e = byb(aob as *const acb);
        let bc = byb(alc as *const acb);

        
        let mvd = gxj(e, 24);
        let jzy = qcy(mvd, ajs);
        if qcz(jzy) == 0xFFFF {
            
            aob = aob.add(4);
            alc = alc.add(4);
            ia -= 4;
            continue;
        }
        let jzx = qcy(mvd, els(255));
        if qcz(jzx) == 0xFFFF {
            
            ccs(alc as *mut acb, e);
            aob = aob.add(4);
            alc = alc.add(4);
            ia -= 4;
            continue;
        }

        
        
        let jnj = jyh(e, ajs); 
        let knt = jyh(bc, ajs); 

        
        
        let bfv = qdc(jnj, 0xFF); 
        let mtd = qdb(bfv, 0xFF);  
        let tvt = qdd(fzm(255), mtd);

        
        let wrt = fce(jnj, mtd);
        let shd = fce(knt, tvt);
        let mif = fcd(fcd(wrt, shd), fzm(128));
        let lzn = jyf(mif, 8);

        
        let jni = jyg(e, ajs);
        let knq = jyg(bc, ajs);

        let oe = qdc(jni, 0xFF);
        let mtb = qdb(oe, 0xFF);
        let tvs = qdd(fzm(255), mtb);

        let wru = fce(jni, mtb);
        let krv = fce(knq, tvs);
        let mie = fcd(fcd(wru, krv), fzm(128));
        let lzm = jyf(mie, 8);

        
        let result = qda(lzn, lzm);
        
        let result = iic(result, els(0xFF000000u32 as i32));
        ccs(alc as *mut acb, result);

        aob = aob.add(4);
        alc = alc.add(4);
        ia -= 4;
    }

    
    for _ in 0..ia {
        let dw = (*aob >> 24) as u32;
        if dw == 255 {
            *alc = *aob;
        } else if dw > 0 {
            *alc = kdt(*aob, *alc);
        }
        aob = aob.add(1);
        alc = alc.add(1);
    }
}


#[inline(always)]
pub fn kdt(cy: u32, cs: u32) -> u32 {
    let dw = (cy >> 24) as u32;
    if dw == 0 { return cs; }
    if dw == 255 { return cy; }
    
    let akg = 255 - dw;
    
    let adz = (cy >> 16) & 0xFF;
    let bsi = (cy >> 8) & 0xFF;
    let is = cy & 0xFF;
    
    let ahh = (cs >> 16) & 0xFF;
    let bgs = (cs >> 8) & 0xFF;
    let ng = cs & 0xFF;
    
    
    
    let m = ((adz * dw + ahh * akg + 128) >> 8).v(255);
    let at = ((bsi * dw + bgs * akg + 128) >> 8).v(255);
    let o = ((is * dw + ng * akg + 128) >> 8).v(255);
    
    0xFF000000 | (m << 16) | (at << 8) | o
}








#[cfg(target_arch = "x86_64")]
#[inline]
pub unsafe fn mzl(cs: *mut u32, az: usize, s: u32, dw: u32) {
    if az == 0 { return; }
    if dw == 0 { return; }
    if dw >= 255 {
        bed(cs, az, s | 0xFF000000);
        return;
    }

    let esy = 255 - dw;

    
    let fvi = els(s as i32);
    let mve = fzm(dw as i16);
    let ofd = fzm(esy as i16);
    let ped = fzm(128);
    let ajs = mso();
    let ijr = els(0xFF000000u32 as i32);

    
    let jnj = jyh(fvi, ajs);
    let jni = jyg(fvi, ajs);
    let wrq = fce(jnj, mve);
    let wrp = fce(jni, mve);

    let mut ptr = cs;
    let mut ia = az;

    
    while ia >= 4 {
        let bc = byb(ptr as *const acb);

        
        let knt = jyh(bc, ajs);
        let she = fce(knt, ofd);
        let mif = fcd(fcd(wrq, she), ped);
        let lzn = jyf(mif, 8);

        
        let knq = jyg(bc, ajs);
        let krv = fce(knq, ofd);
        let mie = fcd(fcd(wrp, krv), ped);
        let lzm = jyf(mie, 8);

        
        let result = qda(lzn, lzm);
        let result = iic(result, ijr);
        ccs(ptr as *mut acb, result);

        ptr = ptr.add(4);
        ia -= 4;
    }

    
    let adz = ((s >> 16) & 0xFF) as u32;
    let bsi = ((s >> 8) & 0xFF) as u32;
    let is = (s & 0xFF) as u32;
    for _ in 0..ia {
        let xy = *ptr;
        let ahh = ((xy >> 16) & 0xFF) as u32;
        let bgs = ((xy >> 8) & 0xFF) as u32;
        let ng = (xy & 0xFF) as u32;
        let m = ((adz * dw + ahh * esy + 128) >> 8).v(255);
        let at = ((bsi * dw + bgs * esy + 128) >> 8).v(255);
        let o = ((is * dw + ng * esy + 128) >> 8).v(255);
        *ptr = 0xFF000000 | (m << 16) | (at << 8) | o;
        ptr = ptr.add(1);
    }
}






#[cfg(target_arch = "x86_64")]
pub unsafe fn yik(pq: *mut u32, z: usize, ac: usize, luc: usize, s: u32) {
    for c in 0..ac {
        let br = pq.add(c * luc);
        bed(br, z, s);
    }
}


#[cfg(target_arch = "x86_64")]
pub unsafe fn ygo(
    pq: *mut u32,
    fij: usize,
    cy: *const u32,
    pmw: usize,
    jrf: usize,
    buc: usize,
    bqg: usize,
) {
    for c in 0..jrf {
        let bxg = cy.add(c * pmw);
        let hhd = pq.add((bqg + c) * fij + buc);
        dpd(hhd, bxg, pmw);
    }
}






pub struct Agz {
    
    pub hz: [u32; 128], 
    
    pub z: u8,
    
    pub ac: u8,
    
    pub axw: u32,
}


pub struct GlyphCache {
    
    cqz: [Option<Agz>; 128],
    
    btw: u32,
}

impl GlyphCache {
    pub const fn new() -> Self {
        const Cq: Option<Agz> = None;
        Self {
            cqz: [Cq; 128],
            btw: 0xFF00FF66, 
        }
    }
    
    
    pub fn dbv(&mut self, s: u32) {
        if self.btw != s {
            self.btw = s;
            
            for at in &mut self.cqz {
                *at = None;
            }
        }
    }
    
    
    pub fn ada(&mut self, r: char) -> &Agz {
        let w = (r as usize) & 127;
        
        if self.cqz[w].is_none() || 
           self.cqz[w].as_ref().map(|at| at.axw) != Some(self.btw) {
            
            let hlr = crate::framebuffer::font::ada(r);
            let mut hz = [0u32; 128];
            
            for (bwv, &br) in hlr.iter().cf() {
                for ga in 0..8 {
                    if (br >> (7 - ga)) & 1 == 1 {
                        hz[bwv * 8 + ga] = self.btw;
                    }
                }
            }
            
            self.cqz[w] = Some(Agz {
                hz,
                z: 8,
                ac: 16,
                axw: self.btw,
            });
        }
        
        self.cqz[w].as_ref().unwrap()
    }
    
    
    #[inline]
    pub fn sdl(
        &mut self,
        bi: &mut [u32],
        oq: usize,
        b: usize,
        c: usize,
        r: char,
        lp: u32,
        ei: u32,
    ) {
        let hlr = crate::framebuffer::font::ada(r);
        
        for (bwv, &br) in hlr.iter().cf() {
            let x = c + bwv;
            let mu = x * oq + b;
            
            if mu + 8 > bi.len() { continue; }
            
            
            for ga in 0..8u8 {
                let s = if (br >> (7 - ga)) & 1 == 1 { lp } else { ei };
                bi[mu + ga as usize] = s;
            }
        }
    }
}


use spin::Mutex;
pub static BXO_: Mutex<GlyphCache> = Mutex::new(GlyphCache::new());






pub fn vws(
    bi: &mut [u32],
    oq: usize,
    b: usize,
    c: usize,
    text: &str,
    lp: u32,
    ei: u32,
) {
    let mut bdq = BXO_.lock();
    let mut cx = b;
    
    for r in text.bw() {
        if cx + 8 > oq { break; }
        bdq.sdl(bi, oq, cx, c, r, lp, ei);
        cx += 8;
    }
}


pub fn zjn(
    bi: &mut [u32],
    oq: usize,
    b: usize,
    c: usize,
    ak: &[&str],
    lp: u32,
    ei: u32,
) {
    let mut ae = c;
    for line in ak {
        vws(bi, oq, b, ae, line, lp, ei);
        ae += 16;
    }
}






pub fn ntp(bi: &mut [u32], s: u32) {
    #[cfg(target_arch = "x86_64")]
    unsafe {
        if bi.len() >= 4 {
            bed(bi.mw(), bi.len(), s);
            return;
        }
    }
    
    bi.vi(s);
}


pub fn ror(cs: &mut [u32], cy: &[u32]) {
    let az = cs.len().v(cy.len());
    #[cfg(target_arch = "x86_64")]
    unsafe {
        if az >= 4 {
            dpd(cs.mw(), cy.fq(), az);
            return;
        }
    }
    
    cs[..az].dg(&cy[..az]);
}


pub fn ygk(cs: &mut [u32], cy: &[u32]) {
    let az = cs.len().v(cy.len());
    #[cfg(target_arch = "x86_64")]
    unsafe {
        if az >= 2 {
            kdu(cs.mw(), cy.fq(), az);
            return;
        }
    }
    
    for a in 0..az {
        cs[a] = kdt(cy[a], cs[a]);
    }
}





use core::sync::atomic::{AtomicU8, Ordering};


static ALH_: AtomicU8 = AtomicU8::new(0);


#[cfg(target_arch = "x86_64")]
fn bzx() -> bool {
    let ene = ALH_.load(Ordering::Relaxed);
    if ene != 0 {
        return ene == 2;
    }
    let result = unsafe {
        
        let ebx: u32;
        core::arch::asm!(
            "mov {tmp_rbx}, rbx",
            "cpuid",
            "mov {out}, ebx",
            "mov rbx, {tmp_rbx}",
            zst = bd(reg) _,
            bd = bd(reg) ebx,
            inout("eax") 7u32 => _,
            inout("ecx") 0u32 => _,
            bd("edx") _,
        );
        (ebx & (1 << 5)) != 0
    };
    ALH_.store(if result { 2 } else { 1 }, Ordering::Relaxed);
    result
}



#[cfg(target_arch = "x86_64")]
#[dmi(aiy = "avx2")]
pub unsafe fn ssl(cs: *mut u32, az: usize, s: u32) {
    if az == 0 { return; }

    let dfb = qcr(s as i32);
    let mut ptr = cs;
    let mut ia = az;

    
    while ia >= 32 {
        ddr(ptr as *mut bpm, dfb);
        ddr(ptr.add(8) as *mut bpm, dfb);
        ddr(ptr.add(16) as *mut bpm, dfb);
        ddr(ptr.add(24) as *mut bpm, dfb);
        ptr = ptr.add(32);
        ia -= 32;
    }
    
    while ia >= 8 {
        ddr(ptr as *mut bpm, dfb);
        ptr = ptr.add(8);
        ia -= 8;
    }
    
    for a in 0..ia {
        *ptr.add(a) = s;
    }
}



#[cfg(target_arch = "x86_64")]
#[dmi(aiy = "avx2")]
pub unsafe fn rou(cs: *mut u32, cy: *const u32, az: usize) {
    if az == 0 { return; }

    let mut bc = cs;
    let mut e = cy;
    let mut ia = az;

    
    while ia >= 32 {
        let q = fzl(e as *const bpm);
        let o = fzl(e.add(8) as *const bpm);
        let r = fzl(e.add(16) as *const bpm);
        let aa = fzl(e.add(24) as *const bpm);
        ddr(bc as *mut bpm, q);
        ddr(bc.add(8) as *mut bpm, o);
        ddr(bc.add(16) as *mut bpm, r);
        ddr(bc.add(24) as *mut bpm, aa);
        bc = bc.add(32);
        e = e.add(32);
        ia -= 32;
    }
    
    while ia >= 8 {
        ddr(bc as *mut bpm, fzl(e as *const bpm));
        bc = bc.add(8);
        e = e.add(8);
        ia -= 8;
    }
    
    for a in 0..ia {
        *bc.add(a) = *e.add(a);
    }
}



#[cfg(target_arch = "x86_64")]
#[dmi(aiy = "avx2")]
pub unsafe fn qpw(cs: *mut u32, cy: *const u32, az: usize) {
    if az == 0 { return; }

    let ajs = yba();
    let iv = msn(128);

    let mut bc = cs;
    let mut e = cy;
    let mut ia = az;

    
    while ia >= 8 {
        let fvi = fzl(e as *const bpm);
        let nod = fzl(bc as *const bpm);

        
        let ijr = ybb(fvi, 24);
        let jzx = qco(ijr, qcr(0xFF));
        let jzy = qco(ijr, ajs);

        if qcq(jzx) == -1i32 {
            ddr(bc as *mut bpm, fvi);
        } else if qcq(jzy) != -1i32 {
            
            let pms = qcx(fvi, ajs);
            let shc = qcx(nod, ajs);

            
            let mvc = qcs(
                qct(pms, 0xFF), 0xFF
            );
            let tvv = qcv(msn(255), mvc);

            let qpz = jxy(
                jxy(jxz(pms, mvc), iv),
                jxz(shc, tvv),
            );
            let vyg = qcu(qpz, 8);

            
            let pmr = qcw(fvi, ajs);
            let sha = qcw(nod, ajs);

            let mvb = qcs(
                qct(pmr, 0xFF), 0xFF
            );
            let tvu = qcv(msn(255), mvb);

            let qpy = jxy(
                jxy(jxz(pmr, mvb), iv),
                jxz(sha, tvu),
            );
            let vyf = qcu(qpy, 8);

            ddr(bc as *mut bpm, yaz(vyg, vyf));
        }
        

        bc = bc.add(8);
        e = e.add(8);
        ia -= 8;
    }
    
    for a in 0..ia {
        *bc.add(a) = kdt(*e.add(a), *bc.add(a));
    }
}






#[cfg(target_arch = "x86_64")]
#[inline]
pub unsafe fn yqn(cs: *mut u32, az: usize, s: u32) {
    if bzx() {
        ssl(cs, az, s);
    } else {
        bed(cs, az, s);
    }
}


#[cfg(target_arch = "x86_64")]
#[inline]
pub unsafe fn yka(cs: *mut u32, cy: *const u32, az: usize) {
    if bzx() {
        rou(cs, cy, az);
    } else {
        dpd(cs, cy, az);
    }
}


#[cfg(target_arch = "x86_64")]
#[inline]
pub unsafe fn ygl(cs: *mut u32, cy: *const u32, az: usize) {
    if bzx() {
        qpw(cs, cy, az);
    } else {
        kdu(cs, cy, az);
    }
}
