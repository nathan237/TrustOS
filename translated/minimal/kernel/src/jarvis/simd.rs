












#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

#[cfg(target_arch = "aarch64")]
use core::arch::aarch64::*;

use core::sync::atomic::{AtomicBool, Ordering};


static ANB_: AtomicBool = AtomicBool::new(false);



pub fn igp() {
    #[cfg(target_arch = "x86_64")]
    {
        let caps = crate::cpu::capabilities();
        let mjd = caps.map(|c| c.avx2 && c.fma).unwrap_or(false);

        if mjd {
            ANB_.store(true, Ordering::Release);
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
fn dgd() -> bool {
    ANB_.load(Ordering::Relaxed)
}









#[cfg(target_arch = "x86_64")]
#[inline]
fn lhc(a: &[f32], b: &[f32], len: usize) -> f32 {
    unsafe {
        let lf = a.as_ptr();
        let bp = b.as_ptr();

        let mut ke = _mm_setzero_ps();
        let mut abo = _mm_setzero_ps();
        let mut akv = _mm_setzero_ps();
        let mut bem = _mm_setzero_ps();

        
        let ym = len / 16;
        for i in 0..ym {
            let base = i * 16;
            let abn = _mm_loadu_ps(lf.add(base));
            let kl = _mm_loadu_ps(bp.add(base));
            ke = _mm_add_ps(ke, _mm_mul_ps(abn, kl));

            let eb = _mm_loadu_ps(lf.add(base + 4));
            let gf = _mm_loadu_ps(bp.add(base + 4));
            abo = _mm_add_ps(abo, _mm_mul_ps(eb, gf));

            let fy = _mm_loadu_ps(lf.add(base + 8));
            let iq = _mm_loadu_ps(bp.add(base + 8));
            akv = _mm_add_ps(akv, _mm_mul_ps(fy, iq));

            let kb = _mm_loadu_ps(lf.add(base + 12));
            let sc = _mm_loadu_ps(bp.add(base + 12));
            bem = _mm_add_ps(bem, _mm_mul_ps(kb, sc));
        }

        
        ke = _mm_add_ps(ke, abo);
        akv = _mm_add_ps(akv, bem);
        ke = _mm_add_ps(ke, akv);

        
        let jv = ym * 16;
        let aci = (len - jv) / 4;
        for i in 0..aci {
            let offset = jv + i * 4;
            let akw = _mm_loadu_ps(lf.add(offset));
            let lm = _mm_loadu_ps(bp.add(offset));
            ke = _mm_add_ps(ke, _mm_mul_ps(akw, lm));
        }

        
        let hi = _mm_movehl_ps(ke, ke);     
        let sum = _mm_add_ps(ke, hi);          
        let cqm = _mm_shuffle_ps(sum, sum, 1);  
        let av = _mm_add_ss(sum, cqm);       
        let mut result = _mm_cvtss_f32(av);

        
        let mg = jv + aci * 4;
        for i in mg..len {
            result += *lf.add(i) * *bp.add(i);
        }

        result
    }
}









#[cfg(target_arch = "aarch64")]
#[inline]
fn lhb(a: &[f32], b: &[f32], len: usize) -> f32 {
    unsafe {
        let lf = a.as_ptr();
        let bp = b.as_ptr();

        let mut ke = vdupq_n_f32(0.0);
        let mut abo = vdupq_n_f32(0.0);
        let mut akv = vdupq_n_f32(0.0);
        let mut bem = vdupq_n_f32(0.0);

        
        let ym = len / 16;
        for i in 0..ym {
            let base = i * 16;
            let abn = vld1q_f32(lf.add(base));
            let kl = vld1q_f32(bp.add(base));
            ke = vfmaq_f32(ke, abn, kl);

            let eb = vld1q_f32(lf.add(base + 4));
            let gf = vld1q_f32(bp.add(base + 4));
            abo = vfmaq_f32(abo, eb, gf);

            let fy = vld1q_f32(lf.add(base + 8));
            let iq = vld1q_f32(bp.add(base + 8));
            akv = vfmaq_f32(akv, fy, iq);

            let kb = vld1q_f32(lf.add(base + 12));
            let sc = vld1q_f32(bp.add(base + 12));
            bem = vfmaq_f32(bem, kb, sc);
        }

        
        ke = vaddq_f32(ke, abo);
        akv = vaddq_f32(akv, bem);
        ke = vaddq_f32(ke, akv);

        
        let jv = ym * 16;
        let aci = (len - jv) / 4;
        for i in 0..aci {
            let offset = jv + i * 4;
            let akw = vld1q_f32(lf.add(offset));
            let lm = vld1q_f32(bp.add(offset));
            ke = vfmaq_f32(ke, akw, lm);
        }

        
        let mut result = vaddvq_f32(ke);

        
        let mg = jv + aci * 4;
        for i in mg..len {
            result += *lf.add(i) * *bp.add(i);
        }

        result
    }
}












#[cfg(target_arch = "x86_64")]
pub fn tk(out: &mut [f32], w: &[f32], x: &[f32], cols: usize, rows: usize) {
    for r in 0..rows {
        let base = r * cols;
        out[r] = lhc(&w[base..base + cols], x, cols);
    }
}


#[cfg(target_arch = "aarch64")]
pub fn tk(out: &mut [f32], w: &[f32], x: &[f32], cols: usize, rows: usize) {
    for r in 0..rows {
        out[r] = lhb(&w[r * cols..r * cols + cols], x, cols);
    }
}


#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
pub fn tk(out: &mut [f32], w: &[f32], x: &[f32], cols: usize, rows: usize) {
    for r in 0..rows {
        let mut sum = 0.0f32;
        let base = r * cols;
        for c in 0..cols {
            sum += w[base + c] * x[c];
        }
        out[r] = sum;
    }
}













#[cfg(target_arch = "x86_64")]
pub fn bnm(out: &mut [f32], w: &[f32], y: &[f32], cols: usize, rows: usize) {
    if dgd() {
        unsafe { ncq(out, w, y, cols, rows); }
        return;
    }
    
    for v in out[..cols].iter_mut() { *v = 0.0; }

    unsafe {
        let ma = w.as_ptr();
        let op = out.as_mut_ptr();

        for r in 0..rows {
            let ade = y[r];
            if ade == 0.0 { continue; } 

            let so = _mm_set1_ps(ade);
            let base = r * cols;

            
            let ym = cols / 16;
            for i in 0..ym {
                let offset = base + i * 16;
                let hv = i * 16;

                let avr = _mm_loadu_ps(ma.add(offset));
                let bij = _mm_loadu_ps(op.add(hv));
                _mm_storeu_ps(op.add(hv), _mm_add_ps(bij, _mm_mul_ps(avr, so)));

                let ahg = _mm_loadu_ps(ma.add(offset + 4));
                let ayt = _mm_loadu_ps(op.add(hv + 4));
                _mm_storeu_ps(op.add(hv + 4), _mm_add_ps(ayt, _mm_mul_ps(ahg, so)));

                let aeo = _mm_loadu_ps(ma.add(offset + 8));
                let ayu = _mm_loadu_ps(op.add(hv + 8));
                _mm_storeu_ps(op.add(hv + 8), _mm_add_ps(ayu, _mm_mul_ps(aeo, so)));

                let ane = _mm_loadu_ps(ma.add(offset + 12));
                let ayv = _mm_loadu_ps(op.add(hv + 12));
                _mm_storeu_ps(op.add(hv + 12), _mm_add_ps(ayv, _mm_mul_ps(ane, so)));
            }

            
            let jv = ym * 16;
            let aci = (cols - jv) / 4;
            for i in 0..aci {
                let offset = base + jv + i * 4;
                let hv = jv + i * 4;
                let anh = _mm_loadu_ps(ma.add(offset));
                let bim = _mm_loadu_ps(op.add(hv));
                _mm_storeu_ps(op.add(hv), _mm_add_ps(bim, _mm_mul_ps(anh, so)));
            }

            
            let mg = jv + aci * 4;
            for c in mg..cols {
                *op.add(c) += *ma.add(base + c) * ade;
            }
        }
    }
}


#[cfg(target_arch = "aarch64")]
pub fn bnm(out: &mut [f32], w: &[f32], y: &[f32], cols: usize, rows: usize) {
    for v in out[..cols].iter_mut() { *v = 0.0; }
    unsafe {
        let ma = w.as_ptr();
        let op = out.as_mut_ptr();
        for r in 0..rows {
            let ade = y[r];
            if ade == 0.0 { continue; }
            let so = vdupq_n_f32(ade);
            let base = r * cols;
            let ym = cols / 16;
            for i in 0..ym {
                let offset = base + i * 16;
                let hv = i * 16;
                let avr = vld1q_f32(ma.add(offset));
                let bij = vld1q_f32(op.add(hv));
                vst1q_f32(op.add(hv), vfmaq_f32(bij, avr, so));
                let ahg = vld1q_f32(ma.add(offset + 4));
                let ayt = vld1q_f32(op.add(hv + 4));
                vst1q_f32(op.add(hv + 4), vfmaq_f32(ayt, ahg, so));
                let aeo = vld1q_f32(ma.add(offset + 8));
                let ayu = vld1q_f32(op.add(hv + 8));
                vst1q_f32(op.add(hv + 8), vfmaq_f32(ayu, aeo, so));
                let ane = vld1q_f32(ma.add(offset + 12));
                let ayv = vld1q_f32(op.add(hv + 12));
                vst1q_f32(op.add(hv + 12), vfmaq_f32(ayv, ane, so));
            }
            let jv = ym * 16;
            let aci = (cols - jv) / 4;
            for i in 0..aci {
                let offset = base + jv + i * 4;
                let hv = jv + i * 4;
                let anh = vld1q_f32(ma.add(offset));
                let bim = vld1q_f32(op.add(hv));
                vst1q_f32(op.add(hv), vfmaq_f32(bim, anh, so));
            }
            let mg = jv + aci * 4;
            for c in mg..cols {
                *op.add(c) += *ma.add(base + c) * ade;
            }
        }
    }
}


#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
pub fn bnm(out: &mut [f32], w: &[f32], y: &[f32], cols: usize, rows: usize) {
    for v in out[..cols].iter_mut() { *v = 0.0; }
    for r in 0..rows {
        let base = r * cols;
        for c in 0..cols {
            out[c] += w[base + c] * y[r];
        }
    }
}



#[cfg(target_arch = "x86_64")]
pub fn cbq(out: &mut [f32], w: &[f32], y: &[f32], cols: usize, rows: usize) {
    if dgd() {
        unsafe { ncp(out, w, y, cols, rows); }
        return;
    }
    unsafe {
        let ma = w.as_ptr();
        let op = out.as_mut_ptr();

        for r in 0..rows {
            let ade = y[r];
            if ade == 0.0 { continue; }

            let so = _mm_set1_ps(ade);
            let base = r * cols;

            let ym = cols / 16;
            for i in 0..ym {
                let offset = base + i * 16;
                let hv = i * 16;

                let avr = _mm_loadu_ps(ma.add(offset));
                let bij = _mm_loadu_ps(op.add(hv));
                _mm_storeu_ps(op.add(hv), _mm_add_ps(bij, _mm_mul_ps(avr, so)));

                let ahg = _mm_loadu_ps(ma.add(offset + 4));
                let ayt = _mm_loadu_ps(op.add(hv + 4));
                _mm_storeu_ps(op.add(hv + 4), _mm_add_ps(ayt, _mm_mul_ps(ahg, so)));

                let aeo = _mm_loadu_ps(ma.add(offset + 8));
                let ayu = _mm_loadu_ps(op.add(hv + 8));
                _mm_storeu_ps(op.add(hv + 8), _mm_add_ps(ayu, _mm_mul_ps(aeo, so)));

                let ane = _mm_loadu_ps(ma.add(offset + 12));
                let ayv = _mm_loadu_ps(op.add(hv + 12));
                _mm_storeu_ps(op.add(hv + 12), _mm_add_ps(ayv, _mm_mul_ps(ane, so)));
            }

            let jv = ym * 16;
            let aci = (cols - jv) / 4;
            for i in 0..aci {
                let offset = base + jv + i * 4;
                let hv = jv + i * 4;
                let anh = _mm_loadu_ps(ma.add(offset));
                let bim = _mm_loadu_ps(op.add(hv));
                _mm_storeu_ps(op.add(hv), _mm_add_ps(bim, _mm_mul_ps(anh, so)));
            }

            let mg = jv + aci * 4;
            for c in mg..cols {
                *op.add(c) += *ma.add(base + c) * ade;
            }
        }
    }
}


#[cfg(target_arch = "aarch64")]
pub fn cbq(out: &mut [f32], w: &[f32], y: &[f32], cols: usize, rows: usize) {
    unsafe {
        let ma = w.as_ptr();
        let op = out.as_mut_ptr();
        for r in 0..rows {
            let ade = y[r];
            if ade == 0.0 { continue; }
            let so = vdupq_n_f32(ade);
            let base = r * cols;
            let ym = cols / 16;
            for i in 0..ym {
                let offset = base + i * 16;
                let hv = i * 16;
                let avr = vld1q_f32(ma.add(offset));
                let bij = vld1q_f32(op.add(hv));
                vst1q_f32(op.add(hv), vfmaq_f32(bij, avr, so));
                let ahg = vld1q_f32(ma.add(offset + 4));
                let ayt = vld1q_f32(op.add(hv + 4));
                vst1q_f32(op.add(hv + 4), vfmaq_f32(ayt, ahg, so));
                let aeo = vld1q_f32(ma.add(offset + 8));
                let ayu = vld1q_f32(op.add(hv + 8));
                vst1q_f32(op.add(hv + 8), vfmaq_f32(ayu, aeo, so));
                let ane = vld1q_f32(ma.add(offset + 12));
                let ayv = vld1q_f32(op.add(hv + 12));
                vst1q_f32(op.add(hv + 12), vfmaq_f32(ayv, ane, so));
            }
            let jv = ym * 16;
            let aci = (cols - jv) / 4;
            for i in 0..aci {
                let offset = base + jv + i * 4;
                let hv = jv + i * 4;
                let anh = vld1q_f32(ma.add(offset));
                let bim = vld1q_f32(op.add(hv));
                vst1q_f32(op.add(hv), vfmaq_f32(bim, anh, so));
            }
            let mg = jv + aci * 4;
            for c in mg..cols {
                *op.add(c) += *ma.add(base + c) * ade;
            }
        }
    }
}

#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
pub fn cbq(out: &mut [f32], w: &[f32], y: &[f32], cols: usize, rows: usize) {
    for r in 0..rows {
        let base = r * cols;
        for c in 0..cols {
            out[c] += w[base + c] * y[r];
        }
    }
}









#[cfg(target_arch = "x86_64")]
pub fn ayw(qx: &mut [f32], ad: &[f32], x: &[f32], cols: usize, rows: usize) {
    if dgd() {
        unsafe { noo(qx, ad, x, cols, rows); }
        return;
    }
    unsafe {
        let sf = qx.as_mut_ptr();
        let mh = x.as_ptr();

        for r in 0..rows {
            let bgf = ad[r];
            if bgf == 0.0 { continue; }

            let alk = _mm_set1_ps(bgf);
            let base = r * cols;

            
            let ym = cols / 16;
            for i in 0..ym {
                let offset = base + i * 16;
                let aam = i * 16;

                let bm = _mm_loadu_ps(mh.add(aam));
                let aqd = _mm_loadu_ps(sf.add(offset));
                _mm_storeu_ps(sf.add(offset), _mm_add_ps(aqd, _mm_mul_ps(alk, bm)));

                let x1 = _mm_loadu_ps(mh.add(aam + 4));
                let vh = _mm_loadu_ps(sf.add(offset + 4));
                _mm_storeu_ps(sf.add(offset + 4), _mm_add_ps(vh, _mm_mul_ps(alk, x1)));

                let x2 = _mm_loadu_ps(mh.add(aam + 8));
                let jq = _mm_loadu_ps(sf.add(offset + 8));
                _mm_storeu_ps(sf.add(offset + 8), _mm_add_ps(jq, _mm_mul_ps(alk, x2)));

                let x3 = _mm_loadu_ps(mh.add(aam + 12));
                let aqe = _mm_loadu_ps(sf.add(offset + 12));
                _mm_storeu_ps(sf.add(offset + 12), _mm_add_ps(aqe, _mm_mul_ps(alk, x3)));
            }

            
            let jv = ym * 16;
            let aci = (cols - jv) / 4;
            for i in 0..aci {
                let offset = base + jv + i * 4;
                let aam = jv + i * 4;
                let bke = _mm_loadu_ps(mh.add(aam));
                let dny = _mm_loadu_ps(sf.add(offset));
                _mm_storeu_ps(sf.add(offset), _mm_add_ps(dny, _mm_mul_ps(alk, bke)));
            }

            
            let mg = jv + aci * 4;
            for c in mg..cols {
                *sf.add(base + c) += bgf * *mh.add(c);
            }
        }
    }
}


#[cfg(target_arch = "aarch64")]
pub fn ayw(qx: &mut [f32], ad: &[f32], x: &[f32], cols: usize, rows: usize) {
    unsafe {
        let sf = qx.as_mut_ptr();
        let mh = x.as_ptr();
        for r in 0..rows {
            let bgf = ad[r];
            if bgf == 0.0 { continue; }
            let alk = vdupq_n_f32(bgf);
            let base = r * cols;
            let ym = cols / 16;
            for i in 0..ym {
                let offset = base + i * 16;
                let aam = i * 16;
                let bm = vld1q_f32(mh.add(aam));
                let aqd = vld1q_f32(sf.add(offset));
                vst1q_f32(sf.add(offset), vfmaq_f32(aqd, alk, bm));
                let x1 = vld1q_f32(mh.add(aam + 4));
                let vh = vld1q_f32(sf.add(offset + 4));
                vst1q_f32(sf.add(offset + 4), vfmaq_f32(vh, alk, x1));
                let x2 = vld1q_f32(mh.add(aam + 8));
                let jq = vld1q_f32(sf.add(offset + 8));
                vst1q_f32(sf.add(offset + 8), vfmaq_f32(jq, alk, x2));
                let x3 = vld1q_f32(mh.add(aam + 12));
                let aqe = vld1q_f32(sf.add(offset + 12));
                vst1q_f32(sf.add(offset + 12), vfmaq_f32(aqe, alk, x3));
            }
            let jv = ym * 16;
            let aci = (cols - jv) / 4;
            for i in 0..aci {
                let offset = base + jv + i * 4;
                let aam = jv + i * 4;
                let bke = vld1q_f32(mh.add(aam));
                let dny = vld1q_f32(sf.add(offset));
                vst1q_f32(sf.add(offset), vfmaq_f32(dny, alk, bke));
            }
            let mg = jv + aci * 4;
            for c in mg..cols {
                *sf.add(base + c) += bgf * *mh.add(c);
            }
        }
    }
}


#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
pub fn ayw(qx: &mut [f32], ad: &[f32], x: &[f32], cols: usize, rows: usize) {
    for r in 0..rows {
        let base = r * cols;
        for c in 0..cols {
            qx[base + c] += ad[r] * x[c];
        }
    }
}









#[cfg(target_arch = "x86_64")]
pub fn aox(out: &mut [f32], x: &[f32], tv: &[f32]) -> f32 {
    let ae = x.len();

    
    let ss = unsafe {
        let mh = x.as_ptr();
        let mut ke = _mm_setzero_ps();
        let mut abo = _mm_setzero_ps();

        let ant = ae / 8;
        for i in 0..ant {
            let base = i * 8;
            let v0 = _mm_loadu_ps(mh.add(base));
            ke = _mm_add_ps(ke, _mm_mul_ps(v0, v0));
            let v1 = _mm_loadu_ps(mh.add(base + 4));
            abo = _mm_add_ps(abo, _mm_mul_ps(v1, v1));
        }
        ke = _mm_add_ps(ke, abo);

        
        let jv = ant * 8;
        for i in (jv..ae).step_by(4) {
            if i + 4 <= ae {
                let v = _mm_loadu_ps(mh.add(i));
                ke = _mm_add_ps(ke, _mm_mul_ps(v, v));
            }
        }

        let hi = _mm_movehl_ps(ke, ke);
        let sum = _mm_add_ps(ke, hi);
        let cqm = _mm_shuffle_ps(sum, sum, 1);
        let av = _mm_add_ss(sum, cqm);
        let mut result = _mm_cvtss_f32(av);

        
        let mg = (ae / 4) * 4;
        for i in mg..ae {
            result += *mh.add(i) * *mh.add(i);
        }
        result
    };

    let aeg = super::backprop::apq(ss / ae as f32 + super::model::HT_);
    let alu = 1.0 / aeg;

    
    unsafe {
        let mh = x.as_ptr();
        let ma = tv.as_ptr();
        let op = out.as_mut_ptr();
        let dsg = _mm_set1_ps(alu);

        let abu = ae / 4;
        for i in 0..abu {
            let off = i * 4;
            let bke = _mm_loadu_ps(mh.add(off));
            let anh = _mm_loadu_ps(ma.add(off));
            let bnv = _mm_mul_ps(bke, dsg);
            _mm_storeu_ps(op.add(off), _mm_mul_ps(bnv, anh));
        }

        
        let mg = abu * 4;
        for i in mg..ae {
            *op.add(i) = *mh.add(i) * alu * *ma.add(i);
        }
    }

    aeg
}


#[cfg(target_arch = "aarch64")]
pub fn aox(out: &mut [f32], x: &[f32], tv: &[f32]) -> f32 {
    let ae = x.len();
    let ss = unsafe {
        let mh = x.as_ptr();
        let mut ke = vdupq_n_f32(0.0);
        let mut abo = vdupq_n_f32(0.0);
        let ant = ae / 8;
        for i in 0..ant {
            let base = i * 8;
            let v0 = vld1q_f32(mh.add(base));
            ke = vfmaq_f32(ke, v0, v0);
            let v1 = vld1q_f32(mh.add(base + 4));
            abo = vfmaq_f32(abo, v1, v1);
        }
        ke = vaddq_f32(ke, abo);
        let mut result = vaddvq_f32(ke);
        let mg = (ae / 4) * 4;
        for i in mg..ae {
            result += *mh.add(i) * *mh.add(i);
        }
        result
    };
    let aeg = super::backprop::apq(ss / ae as f32 + super::model::HT_);
    let alu = 1.0 / aeg;
    unsafe {
        let mh = x.as_ptr();
        let ma = tv.as_ptr();
        let op = out.as_mut_ptr();
        let dsg = vdupq_n_f32(alu);
        let abu = ae / 4;
        for i in 0..abu {
            let off = i * 4;
            let bke = vld1q_f32(mh.add(off));
            let anh = vld1q_f32(ma.add(off));
            let bnv = vmulq_f32(bke, dsg);
            vst1q_f32(op.add(off), vmulq_f32(bnv, anh));
        }
        let mg = abu * 4;
        for i in mg..ae {
            *op.add(i) = *mh.add(i) * alu * *ma.add(i);
        }
    }
    aeg
}


#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
pub fn aox(out: &mut [f32], x: &[f32], tv: &[f32]) -> f32 {
    let ae = x.len();
    let mut ss = 0.0f32;
    for &v in x { ss += v * v; }
    let aeg = super::backprop::apq(ss / ae as f32 + super::model::HT_);
    let alu = 1.0 / aeg;
    for i in 0..ae {
        out[i] = x[i] * alu * tv[i];
    }
    aeg
}






#[cfg(target_arch = "x86_64")]
pub fn jpv(out: &mut [f32], a: &[f32], b: &[f32], len: usize) {
    if dgd() {
        unsafe { pri(out, a, b, len); }
        return;
    }
    unsafe {
        let lf = a.as_ptr();
        let bp = b.as_ptr();
        let op = out.as_mut_ptr();

        let abu = len / 4;
        for i in 0..abu {
            let off = i * 4;
            let akw = _mm_loadu_ps(lf.add(off));
            let lm = _mm_loadu_ps(bp.add(off));
            _mm_storeu_ps(op.add(off), _mm_add_ps(akw, lm));
        }

        let mg = abu * 4;
        for i in mg..len {
            *op.add(i) = *lf.add(i) + *bp.add(i);
        }
    }
}

#[cfg(target_arch = "aarch64")]
pub fn jpv(out: &mut [f32], a: &[f32], b: &[f32], len: usize) {
    unsafe {
        let lf = a.as_ptr();
        let bp = b.as_ptr();
        let op = out.as_mut_ptr();
        let abu = len / 4;
        for i in 0..abu {
            let off = i * 4;
            let akw = vld1q_f32(lf.add(off));
            let lm = vld1q_f32(bp.add(off));
            vst1q_f32(op.add(off), vaddq_f32(akw, lm));
        }
        let mg = abu * 4;
        for i in mg..len {
            *op.add(i) = *lf.add(i) + *bp.add(i);
        }
    }
}

#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
pub fn jpv(out: &mut [f32], a: &[f32], b: &[f32], len: usize) {
    for i in 0..len { out[i] = a[i] + b[i]; }
}


#[cfg(target_arch = "x86_64")]
pub fn jpw(a: &mut [f32], b: &[f32], len: usize) {
    if dgd() {
        unsafe { prj(a, b, len); }
        return;
    }
    unsafe {
        let lf = a.as_mut_ptr();
        let bp = b.as_ptr();

        let abu = len / 4;
        for i in 0..abu {
            let off = i * 4;
            let akw = _mm_loadu_ps(lf.add(off));
            let lm = _mm_loadu_ps(bp.add(off));
            _mm_storeu_ps(lf.add(off), _mm_add_ps(akw, lm));
        }

        let mg = abu * 4;
        for i in mg..len {
            *lf.add(i) += *bp.add(i);
        }
    }
}

#[cfg(target_arch = "aarch64")]
pub fn jpw(a: &mut [f32], b: &[f32], len: usize) {
    unsafe {
        let lf = a.as_mut_ptr();
        let bp = b.as_ptr();
        let abu = len / 4;
        for i in 0..abu {
            let off = i * 4;
            let akw = vld1q_f32(lf.add(off));
            let lm = vld1q_f32(bp.add(off));
            vst1q_f32(lf.add(off), vaddq_f32(akw, lm));
        }
        let mg = abu * 4;
        for i in mg..len {
            *lf.add(i) += *bp.add(i);
        }
    }
}

#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
pub fn jpw(a: &mut [f32], b: &[f32], len: usize) {
    for i in 0..len { a[i] += b[i]; }
}


#[cfg(target_arch = "x86_64")]
pub fn jpx(a: &mut [f32], aeh: f32, len: usize) {
    if dgd() {
        unsafe { prl(a, aeh, len); }
        return;
    }
    unsafe {
        let lf = a.as_mut_ptr();
        let amx = _mm_set1_ps(aeh);

        let abu = len / 4;
        for i in 0..abu {
            let off = i * 4;
            let v = _mm_loadu_ps(lf.add(off));
            _mm_storeu_ps(lf.add(off), _mm_mul_ps(v, amx));
        }

        let mg = abu * 4;
        for i in mg..len {
            *lf.add(i) *= aeh;
        }
    }
}

#[cfg(target_arch = "aarch64")]
pub fn jpx(a: &mut [f32], aeh: f32, len: usize) {
    unsafe {
        let lf = a.as_mut_ptr();
        let amx = vdupq_n_f32(aeh);
        let abu = len / 4;
        for i in 0..abu {
            let off = i * 4;
            let v = vld1q_f32(lf.add(off));
            vst1q_f32(lf.add(off), vmulq_f32(v, amx));
        }
        let mg = abu * 4;
        for i in mg..len {
            *lf.add(i) *= aeh;
        }
    }
}

#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
pub fn jpx(a: &mut [f32], aeh: f32, len: usize) {
    for i in 0..len { a[i] *= aeh; }
}















#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2,fma")]
#[inline]
unsafe fn lgy(a: &[f32], b: &[f32], len: usize) -> f32 {
    let lf = a.as_ptr();
    let bp = b.as_ptr();

    let mut ke = _mm256_setzero_ps();
    let mut abo = _mm256_setzero_ps();
    let mut akv = _mm256_setzero_ps();
    let mut bem = _mm256_setzero_ps();

    
    let bfg = len / 32;
    for i in 0..bfg {
        let base = i * 32;
        let abn = _mm256_loadu_ps(lf.add(base));
        let kl = _mm256_loadu_ps(bp.add(base));
        ke = _mm256_fmadd_ps(abn, kl, ke);

        let eb = _mm256_loadu_ps(lf.add(base + 8));
        let gf = _mm256_loadu_ps(bp.add(base + 8));
        abo = _mm256_fmadd_ps(eb, gf, abo);

        let fy = _mm256_loadu_ps(lf.add(base + 16));
        let iq = _mm256_loadu_ps(bp.add(base + 16));
        akv = _mm256_fmadd_ps(fy, iq, akv);

        let kb = _mm256_loadu_ps(lf.add(base + 24));
        let sc = _mm256_loadu_ps(bp.add(base + 24));
        bem = _mm256_fmadd_ps(kb, sc, bem);
    }

    
    ke = _mm256_add_ps(ke, abo);
    akv = _mm256_add_ps(akv, bem);
    ke = _mm256_add_ps(ke, akv);

    
    let jv = bfg * 32;
    let aut = (len - jv) / 8;
    for i in 0..aut {
        let offset = jv + i * 8;
        let akw = _mm256_loadu_ps(lf.add(offset));
        let lm = _mm256_loadu_ps(bp.add(offset));
        ke = _mm256_fmadd_ps(akw, lm, ke);
    }

    
    let gar = _mm256_extractf128_ps(ke, 1);
    let gfv = _mm256_castps256_ps128(ke);
    let cre = _mm_add_ps(gfv, gar);
    let hi = _mm_movehl_ps(cre, cre);
    let sum = _mm_add_ps(cre, hi);
    let cqm = _mm_shuffle_ps(sum, sum, 1);
    let av = _mm_add_ss(sum, cqm);
    let mut result = _mm_cvtss_f32(av);

    
    let mg = jv + aut * 8;
    for i in mg..len {
        result += *lf.add(i) * *bp.add(i);
    }

    result
}


#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2,fma")]
unsafe fn qot(out: &mut [f32], w: &[f32], x: &[f32], cols: usize, rows: usize) {
    for r in 0..rows {
        let base = r * cols;
        out[r] = lgy(&w[base..base + cols], x, cols);
    }
}


#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2,fma")]
unsafe fn ncq(out: &mut [f32], w: &[f32], y: &[f32], cols: usize, rows: usize) {
    for v in out[..cols].iter_mut() { *v = 0.0; }

    let ma = w.as_ptr();
    let op = out.as_mut_ptr();

    for r in 0..rows {
        let ade = y[r];
        if ade == 0.0 { continue; }

        let so = _mm256_set1_ps(ade);
        let base = r * cols;

        
        let bfg = cols / 32;
        for i in 0..bfg {
            let offset = base + i * 32;
            let hv = i * 32;

            let bij = _mm256_loadu_ps(op.add(hv));
            let avr = _mm256_loadu_ps(ma.add(offset));
            _mm256_storeu_ps(op.add(hv), _mm256_fmadd_ps(avr, so, bij));

            let ayt = _mm256_loadu_ps(op.add(hv + 8));
            let ahg = _mm256_loadu_ps(ma.add(offset + 8));
            _mm256_storeu_ps(op.add(hv + 8), _mm256_fmadd_ps(ahg, so, ayt));

            let ayu = _mm256_loadu_ps(op.add(hv + 16));
            let aeo = _mm256_loadu_ps(ma.add(offset + 16));
            _mm256_storeu_ps(op.add(hv + 16), _mm256_fmadd_ps(aeo, so, ayu));

            let ayv = _mm256_loadu_ps(op.add(hv + 24));
            let ane = _mm256_loadu_ps(ma.add(offset + 24));
            _mm256_storeu_ps(op.add(hv + 24), _mm256_fmadd_ps(ane, so, ayv));
        }

        
        let jv = bfg * 32;
        let aut = (cols - jv) / 8;
        for i in 0..aut {
            let offset = base + jv + i * 8;
            let hv = jv + i * 8;
            let bim = _mm256_loadu_ps(op.add(hv));
            let anh = _mm256_loadu_ps(ma.add(offset));
            _mm256_storeu_ps(op.add(hv), _mm256_fmadd_ps(anh, so, bim));
        }

        
        let mg = jv + aut * 8;
        for c in mg..cols {
            *op.add(c) += *ma.add(base + c) * ade;
        }
    }
}


#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2,fma")]
unsafe fn ncp(out: &mut [f32], w: &[f32], y: &[f32], cols: usize, rows: usize) {
    let ma = w.as_ptr();
    let op = out.as_mut_ptr();

    for r in 0..rows {
        let ade = y[r];
        if ade == 0.0 { continue; }

        let so = _mm256_set1_ps(ade);
        let base = r * cols;

        let bfg = cols / 32;
        for i in 0..bfg {
            let offset = base + i * 32;
            let hv = i * 32;

            let bij = _mm256_loadu_ps(op.add(hv));
            let avr = _mm256_loadu_ps(ma.add(offset));
            _mm256_storeu_ps(op.add(hv), _mm256_fmadd_ps(avr, so, bij));

            let ayt = _mm256_loadu_ps(op.add(hv + 8));
            let ahg = _mm256_loadu_ps(ma.add(offset + 8));
            _mm256_storeu_ps(op.add(hv + 8), _mm256_fmadd_ps(ahg, so, ayt));

            let ayu = _mm256_loadu_ps(op.add(hv + 16));
            let aeo = _mm256_loadu_ps(ma.add(offset + 16));
            _mm256_storeu_ps(op.add(hv + 16), _mm256_fmadd_ps(aeo, so, ayu));

            let ayv = _mm256_loadu_ps(op.add(hv + 24));
            let ane = _mm256_loadu_ps(ma.add(offset + 24));
            _mm256_storeu_ps(op.add(hv + 24), _mm256_fmadd_ps(ane, so, ayv));
        }

        let jv = bfg * 32;
        let aut = (cols - jv) / 8;
        for i in 0..aut {
            let offset = base + jv + i * 8;
            let hv = jv + i * 8;
            let bim = _mm256_loadu_ps(op.add(hv));
            let anh = _mm256_loadu_ps(ma.add(offset));
            _mm256_storeu_ps(op.add(hv), _mm256_fmadd_ps(anh, so, bim));
        }

        let mg = jv + aut * 8;
        for c in mg..cols {
            *op.add(c) += *ma.add(base + c) * ade;
        }
    }
}


#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2,fma")]
unsafe fn noo(qx: &mut [f32], ad: &[f32], x: &[f32], cols: usize, rows: usize) {
    let sf = qx.as_mut_ptr();
    let mh = x.as_ptr();

    for r in 0..rows {
        let bgf = ad[r];
        if bgf == 0.0 { continue; }

        let alk = _mm256_set1_ps(bgf);
        let base = r * cols;

        
        let bfg = cols / 32;
        for i in 0..bfg {
            let offset = base + i * 32;
            let aam = i * 32;

            let bm = _mm256_loadu_ps(mh.add(aam));
            let aqd = _mm256_loadu_ps(sf.add(offset));
            _mm256_storeu_ps(sf.add(offset), _mm256_fmadd_ps(alk, bm, aqd));

            let x1 = _mm256_loadu_ps(mh.add(aam + 8));
            let vh = _mm256_loadu_ps(sf.add(offset + 8));
            _mm256_storeu_ps(sf.add(offset + 8), _mm256_fmadd_ps(alk, x1, vh));

            let x2 = _mm256_loadu_ps(mh.add(aam + 16));
            let jq = _mm256_loadu_ps(sf.add(offset + 16));
            _mm256_storeu_ps(sf.add(offset + 16), _mm256_fmadd_ps(alk, x2, jq));

            let x3 = _mm256_loadu_ps(mh.add(aam + 24));
            let aqe = _mm256_loadu_ps(sf.add(offset + 24));
            _mm256_storeu_ps(sf.add(offset + 24), _mm256_fmadd_ps(alk, x3, aqe));
        }

        
        let jv = bfg * 32;
        let aut = (cols - jv) / 8;
        for i in 0..aut {
            let offset = base + jv + i * 8;
            let aam = jv + i * 8;
            let bke = _mm256_loadu_ps(mh.add(aam));
            let dny = _mm256_loadu_ps(sf.add(offset));
            _mm256_storeu_ps(sf.add(offset), _mm256_fmadd_ps(alk, bke, dny));
        }

        
        let mg = jv + aut * 8;
        for c in mg..cols {
            *sf.add(base + c) += bgf * *mh.add(c);
        }
    }
}


#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2,fma")]
unsafe fn qul(out: &mut [f32], x: &[f32], tv: &[f32]) -> f32 {
    let ae = x.len();
    let mh = x.as_ptr();

    
    let mut ke = _mm256_setzero_ps();
    let mut abo = _mm256_setzero_ps();

    let ym = ae / 16;
    for i in 0..ym {
        let base = i * 16;
        let v0 = _mm256_loadu_ps(mh.add(base));
        ke = _mm256_fmadd_ps(v0, v0, ke);
        let v1 = _mm256_loadu_ps(mh.add(base + 8));
        abo = _mm256_fmadd_ps(v1, v1, abo);
    }
    ke = _mm256_add_ps(ke, abo);

    
    let jv = ym * 16;
    let aut = (ae - jv) / 8;
    for i in 0..aut {
        let v = _mm256_loadu_ps(mh.add(jv + i * 8));
        ke = _mm256_fmadd_ps(v, v, ke);
    }

    
    let gar = _mm256_extractf128_ps(ke, 1);
    let gfv = _mm256_castps256_ps128(ke);
    let cre = _mm_add_ps(gfv, gar);
    let hi = _mm_movehl_ps(cre, cre);
    let sum = _mm_add_ps(cre, hi);
    let cqm = _mm_shuffle_ps(sum, sum, 1);
    let av = _mm_add_ss(sum, cqm);
    let mut ss = _mm_cvtss_f32(av);

    
    let mg = jv + aut * 8;
    for i in mg..ae {
        ss += *mh.add(i) * *mh.add(i);
    }

    let aeg = super::backprop::apq(ss / ae as f32 + super::model::HT_);
    let alu = 1.0 / aeg;

    
    let ma = tv.as_ptr();
    let op = out.as_mut_ptr();
    let dsg = _mm256_set1_ps(alu);

    let ant = ae / 8;
    for i in 0..ant {
        let off = i * 8;
        let bke = _mm256_loadu_ps(mh.add(off));
        let anh = _mm256_loadu_ps(ma.add(off));
        let bnv = _mm256_mul_ps(bke, dsg);
        _mm256_storeu_ps(op.add(off), _mm256_mul_ps(bnv, anh));
    }

    
    let okr = ant * 8;
    for i in okr..ae {
        *op.add(i) = *mh.add(i) * alu * *ma.add(i);
    }

    aeg
}


#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn pri(out: &mut [f32], a: &[f32], b: &[f32], len: usize) {
    let lf = a.as_ptr();
    let bp = b.as_ptr();
    let op = out.as_mut_ptr();

    let ant = len / 8;
    for i in 0..ant {
        let off = i * 8;
        let akw = _mm256_loadu_ps(lf.add(off));
        let lm = _mm256_loadu_ps(bp.add(off));
        _mm256_storeu_ps(op.add(off), _mm256_add_ps(akw, lm));
    }

    let mg = ant * 8;
    for i in mg..len {
        *op.add(i) = *lf.add(i) + *bp.add(i);
    }
}


#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn prj(a: &mut [f32], b: &[f32], len: usize) {
    let lf = a.as_mut_ptr();
    let bp = b.as_ptr();

    let ant = len / 8;
    for i in 0..ant {
        let off = i * 8;
        let akw = _mm256_loadu_ps(lf.add(off));
        let lm = _mm256_loadu_ps(bp.add(off));
        _mm256_storeu_ps(lf.add(off), _mm256_add_ps(akw, lm));
    }

    let mg = ant * 8;
    for i in mg..len {
        *lf.add(i) += *bp.add(i);
    }
}


#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn prl(a: &mut [f32], aeh: f32, len: usize) {
    let lf = a.as_mut_ptr();
    let amx = _mm256_set1_ps(aeh);

    let ant = len / 8;
    for i in 0..ant {
        let off = i * 8;
        let v = _mm256_loadu_ps(lf.add(off));
        _mm256_storeu_ps(lf.add(off), _mm256_mul_ps(v, amx));
    }

    let mg = ant * 8;
    for i in mg..len {
        *lf.add(i) *= aeh;
    }
}
