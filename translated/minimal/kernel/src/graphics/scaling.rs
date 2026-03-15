






























use core::sync::atomic::{AtomicU32, Ordering};






static BEZ_: AtomicU32 = AtomicU32::new(1);


const EY_: u32 = 8;
const HM_: u32 = 16;









pub fn wjo(pv: u32) {
    let feu = pv.qp(1, 3);
    BEZ_.store(feu, Ordering::SeqCst);
    crate::serial_println!("[Scaling] Scale factor set to {}x", feu);
}


#[inline]
pub fn ckv() -> u32 {
    BEZ_.load(Ordering::Relaxed)
}







pub fn qlj(lu: u32, qh: u32) -> u32 {
    let pv = if lu >= 3840 {
        3
    } else if lu >= 2560 {
        2
    } else {
        1
    };
    crate::serial_println!(
        "[Scaling] Auto-detected {}x scale for {}x{} framebuffer",
        pv, lu, qh
    );
    pv
}




pub fn init(lu: u32, qh: u32) {
    let pv = qlj(lu, qh);
    wjo(pv);
}






#[inline]
pub fn bv(bn: u32) -> u32 {
    bn * ckv()
}


#[inline]
pub fn zlr(bn: i32) -> i32 {
    bn * ckv() as i32
}


#[inline]
pub fn zuj(hvc: u32) -> u32 {
    let bb = ckv();
    if bb == 0 { hvc } else { hvc / bb }
}


#[inline]
pub fn zuk(hvc: i32) -> i32 {
    let bb = ckv() as i32;
    if bb == 0 { hvc } else { hvc / bb }
}


#[inline]
pub fn bmi() -> u32 {
    EY_ * ckv()
}


#[inline]
pub fn fep() -> u32 {
    HM_ * ckv()
}








#[derive(Clone, Copy, Debug)]
pub struct Ayj {
    pub pv: u32,
    pub psb: u32,
    pub ptl: u32,
    pub pzl: u32,
    pub pzn: u32,
    pub cxb: u32,
    pub nml: u32,
    pub bmi: u32,
    pub fep: u32,
}

impl Ayj {
    
    const ALQ_: u32 = 40;
    const ALR_: u32 = 28;
    const ALS_: u32 = 6;
    const ALT_: u32 = 12;
    const ALO_: u32 = 24;
    const ALP_: u32 = 60;

    
    pub fn cv() -> Self {
        let bb = ckv();
        Ayj {
            pv: bb,
            psb: Self::ALQ_ * bb,
            ptl: Self::ALR_ * bb,
            pzl: Self::ALS_ * bb,
            pzn: Self::ALT_ * bb,
            cxb: Self::ALO_ * bb,
            nml: Self::ALP_ * bb,
            bmi: EY_ * bb,
            fep: HM_ * bb,
        }
    }

    
    pub fn zwd(bb: u32) -> Self {
        let bb = bb.qp(1, 3);
        Ayj {
            pv: bb,
            psb: Self::ALQ_ * bb,
            ptl: Self::ALR_ * bb,
            pzl: Self::ALS_ * bb,
            pzn: Self::ALT_ * bb,
            cxb: Self::ALO_ * bb,
            nml: Self::ALP_ * bb,
            bmi: EY_ * bb,
            fep: HM_ * bb,
        }
    }
}









pub fn krh(b: u32, c: u32, r: char, s: u32) {
    let pv = ckv();

    
    if pv == 1 {
        crate::framebuffer::afn(b, c, r, s);
        return;
    }

    let ka = crate::framebuffer::font::ada(r);
    let lu = crate::framebuffer::z();
    let qh = crate::framebuffer::ac();

    
    let aza = EY_ * pv;
    let aku = HM_ * pv;
    if b >= lu || c >= qh {
        return;
    }

    
    let hrf = lu.v(b + aza);
    let hrg = qh.v(c + aku);

    for br in 0..HM_ as usize {
        let fs = ka[br];
        if fs == 0 {
            continue; 
        }
        let gzx = c + (br as u32) * pv;
        if gzx >= hrg {
            break;
        }

        for bj in 0..EY_ as usize {
            if (fs >> (7 - bj)) & 1 == 1 {
                let gzw = b + (bj as u32) * pv;
                if gzw >= hrf {
                    break;
                }

                
                for cq in 0..pv {
                    let x = gzx + cq;
                    if x >= hrg {
                        break;
                    }
                    for cr in 0..pv {
                        let y = gzw + cr;
                        if y < hrf {
                            crate::framebuffer::sf(y, x, s);
                        }
                    }
                }
            }
        }
    }
}




pub fn kri(b: i32, c: i32, text: &str, s: u32) {
    let dt = bmi() as i32;
    let gz = crate::framebuffer::z() as i32;
    let kc = crate::framebuffer::ac() as i32;

    if c < 0 || c >= kc {
        return;
    }

    for (a, r) in text.bw().cf() {
        let y = b + (a as i32) * dt;
        if y >= gz {
            break; 
        }
        if y + dt <= 0 {
            continue; 
        }
        if y >= 0 {
            krh(y as u32, c as u32, r, s);
        }
    }
}




pub fn azp(b: i32, c: i32, text: &str, s: u32, pv: u32) {
    let pv = pv.qp(1, 3);
    let dt = (EY_ * pv) as i32;
    let gz = crate::framebuffer::z() as i32;
    let kc = crate::framebuffer::ac() as i32;

    if c < 0 || c >= kc {
        return;
    }

    for (a, r) in text.bw().cf() {
        let y = b + (a as i32) * dt;
        if y >= gz {
            break;
        }
        if y + dt <= 0 {
            continue;
        }
        if y >= 0 {
            sce(y as u32, c as u32, r, s, pv);
        }
    }
}


fn sce(b: u32, c: u32, r: char, s: u32, pv: u32) {
    if pv == 1 {
        crate::framebuffer::afn(b, c, r, s);
        return;
    }

    let ka = crate::framebuffer::font::ada(r);
    let lu = crate::framebuffer::z();
    let qh = crate::framebuffer::ac();

    let hrf = lu.v(b + EY_ * pv);
    let hrg = qh.v(c + HM_ * pv);

    for br in 0..HM_ as usize {
        let fs = ka[br];
        if fs == 0 {
            continue;
        }
        let gzx = c + (br as u32) * pv;
        if gzx >= hrg {
            break;
        }

        for bj in 0..EY_ as usize {
            if (fs >> (7 - bj)) & 1 == 1 {
                let gzw = b + (bj as u32) * pv;
                if gzw >= hrf {
                    break;
                }
                for cq in 0..pv {
                    let x = gzx + cq;
                    if x >= hrg {
                        break;
                    }
                    for cr in 0..pv {
                        let y = gzw + cr;
                        if y < hrf {
                            crate::framebuffer::sf(y, x, s);
                        }
                    }
                }
            }
        }
    }
}








pub fn yqp(b: i32, c: i32, uia: u32, uhz: u32, s: u32) {
    let bb = ckv();
    let ars = uia * bb;
    let afv = uhz * bb;

    if b >= 0 && c >= 0 {
        crate::framebuffer::ah(b as u32, c as u32, ars, afv, s);
    }
}


#[inline]
pub fn zls(b: i32, c: i32, d: u32, i: u32) -> (i32, i32, u32, u32) {
    let bb = ckv();
    (
        b * bb as i32,
        c * bb as i32,
        d * bb,
        i * bb,
    )
}









pub fn ynj(
    lf: i32,
    ot: i32,
    pattern: &[[u8; 12]],
    dua: u32,
    ebo: u32,
) {
    let pv = ckv();
    let gz = crate::framebuffer::z();
    let kc = crate::framebuffer::ac();

    for (ae, br) in pattern.iter().cf() {
        for (cx, &il) in br.iter().cf() {
            if il == 0 {
                continue;
            }
            let s = match il {
                1 => dua,
                2 => ebo,
                _ => continue,
            };

            
            for cq in 0..pv {
                for cr in 0..pv {
                    let y = lf + (cx as u32 * pv + cr) as i32;
                    let x = ot + (ae as u32 * pv + cq) as i32;
                    if y >= 0 && x >= 0 && (y as u32) < gz && (x as u32) < kc {
                        crate::framebuffer::sf(y as u32, x as u32, s);
                    }
                }
            }
        }
    }
}






#[inline]
pub fn clj(text: &str) -> u32 {
    text.len() as u32 * bmi()
}


#[inline]
pub fn zcm() -> u32 {
    fep()
}


#[inline]
pub fn zcn(text: &str, pv: u32) -> u32 {
    text.len() as u32 * EY_ * pv.qp(1, 3)
}
