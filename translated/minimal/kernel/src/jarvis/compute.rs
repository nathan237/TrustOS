



























use alloc::string::String;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Backend {
    
    Pd,
    
    Ot,
}


static ATP_: AtomicU64 = AtomicU64::new(0);
static DG_: AtomicU64 = AtomicU64::new(0);
static KH_: AtomicBool = AtomicBool::new(false);


pub fn nky() -> Backend {
    
    if crate::drivers::amdgpu::clb() {
        KH_.store(true, Ordering::Relaxed);
        Backend::Ot
    } else {
        Backend::Pd
    }
}


pub fn gil() -> bool {
    KH_.load(Ordering::Relaxed)
}


pub fn iiu() -> Backend {
    if KH_.load(Ordering::Relaxed) {
        Backend::Ot
    } else {
        Backend::Pd
    }
}













#[inline]
pub fn ami(bd: &mut [f32], d: &[f32], b: &[f32], ec: usize, lk: usize) {
    match iiu() {
        Backend::Ot => {
            
            
            
            thc(bd, d, b, ec, lk);
        }
        Backend::Pd => {
            super::simd::ami(bd, d, b, ec, lk);
            DG_.fetch_add(1, Ordering::Relaxed);
        }
    }
}



#[inline]
pub fn dta(bd: &mut [f32], d: &[f32], c: &[f32], ec: usize, lk: usize) {
    match iiu() {
        Backend::Ot => {
            super::simd::dta(bd, d, c, ec, lk);
            DG_.fetch_add(1, Ordering::Relaxed);
        }
        Backend::Pd => {
            super::simd::dta(bd, d, c, ec, lk);
            DG_.fetch_add(1, Ordering::Relaxed);
        }
    }
}


#[inline]
pub fn euq(bd: &mut [f32], d: &[f32], c: &[f32], ec: usize, lk: usize) {
    match iiu() {
        Backend::Ot => {
            super::simd::euq(bd, d, c, ec, lk);
            DG_.fetch_add(1, Ordering::Relaxed);
        }
        Backend::Pd => {
            super::simd::euq(bd, d, c, ec, lk);
            DG_.fetch_add(1, Ordering::Relaxed);
        }
    }
}



#[inline]
pub fn ctd(aix: &mut [f32], bg: &[f32], b: &[f32], ec: usize, lk: usize) {
    match iiu() {
        Backend::Ot => {
            super::simd::ctd(aix, bg, b, ec, lk);
            DG_.fetch_add(1, Ordering::Relaxed);
        }
        Backend::Pd => {
            super::simd::ctd(aix, bg, b, ec, lk);
            DG_.fetch_add(1, Ordering::Relaxed);
        }
    }
}


#[inline]
pub fn cbl(bd: &mut [f32], b: &[f32], amz: &[f32]) -> f32 {
    let bfd = super::simd::cbl(bd, b, amz);
    DG_.fetch_add(1, Ordering::Relaxed);
    bfd
}







fn thc(bd: &mut [f32], d: &[f32], b: &[f32], ec: usize, lk: usize) {
    
    if !crate::drivers::amdgpu::compute::uc() {
        super::simd::ami(bd, d, b, ec, lk);
        DG_.fetch_add(1, Ordering::Relaxed);
        return;
    }

    
    
    
    
    
    
    
    
    
    
    
    
    
    
    super::simd::ami(bd, d, b, ec, lk);
    DG_.fetch_add(1, Ordering::Relaxed);
}



pub fn mog(ydw: &super::model::TransformerWeights) -> Result<usize, &'static str> {
    if !crate::drivers::amdgpu::clb() {
        return Err("No AMD GPU detected");
    }

    if !crate::drivers::amdgpu::sdma::uc() {
        return Err("SDMA not ready");
    }

    
    
    
    
    
    
    
    
    
    
    
    
    

    Err("GPU weight upload not yet implemented (need real hardware)")
}






pub fn awz() -> String {
    let backend = if gil() { "GPU (AMD RDNA)" } else { "CPU (SSE2 SIMD)" };
    let the = ATP_.load(Ordering::Relaxed);
    let rpt = DG_.load(Ordering::Relaxed);
    alloc::format!("Backend: {}, GPU ops: {}, CPU ops: {}", backend, the, rpt)
}


pub fn zl() -> Vec<String> {
    let mut ak = Vec::new();
    let backend = if gil() { "GPU (AMD RDNA)" } else { "CPU (SSE2 SIMD)" };
    ak.push(alloc::format!("Compute: {}", backend));
    ak.push(alloc::format!("  GPU ops:  {}", ATP_.load(Ordering::Relaxed)));
    ak.push(alloc::format!("  CPU ops:  {}", DG_.load(Ordering::Relaxed)));

    if gil() {
        if let Some(co) = crate::drivers::amdgpu::ani() {
            ak.push(alloc::format!("  GPU: {} ({})", co.beh(), co.jwb()));
        }
    }

    ak
}
