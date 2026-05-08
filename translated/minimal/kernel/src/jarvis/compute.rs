



























use alloc::string::String;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Backend {
    
    CpuSimd,
    
    AmdGpu,
}


static AVT_: AtomicU64 = AtomicU64::new(0);
static DO_: AtomicU64 = AtomicU64::new(0);
static LA_: AtomicBool = AtomicBool::new(false);


pub fn hrv() -> Backend {
    
    if crate::drivers::amdgpu::aud() {
        LA_.store(true, Ordering::Relaxed);
        Backend::AmdGpu
    } else {
        Backend::CpuSimd
    }
}


pub fn cyv() -> bool {
    LA_.load(Ordering::Relaxed)
}


pub fn eex() -> Backend {
    if LA_.load(Ordering::Relaxed) {
        Backend::AmdGpu
    } else {
        Backend::CpuSimd
    }
}













#[inline]
pub fn tk(out: &mut [f32], w: &[f32], x: &[f32], cols: usize, rows: usize) {
    match eex() {
        Backend::AmdGpu => {
            
            
            
            mfu(out, w, x, cols, rows);
        }
        Backend::CpuSimd => {
            super::simd::tk(out, w, x, cols, rows);
            DO_.fetch_add(1, Ordering::Relaxed);
        }
    }
}



#[inline]
pub fn bnm(out: &mut [f32], w: &[f32], y: &[f32], cols: usize, rows: usize) {
    match eex() {
        Backend::AmdGpu => {
            super::simd::bnm(out, w, y, cols, rows);
            DO_.fetch_add(1, Ordering::Relaxed);
        }
        Backend::CpuSimd => {
            super::simd::bnm(out, w, y, cols, rows);
            DO_.fetch_add(1, Ordering::Relaxed);
        }
    }
}


#[inline]
pub fn cbq(out: &mut [f32], w: &[f32], y: &[f32], cols: usize, rows: usize) {
    match eex() {
        Backend::AmdGpu => {
            super::simd::cbq(out, w, y, cols, rows);
            DO_.fetch_add(1, Ordering::Relaxed);
        }
        Backend::CpuSimd => {
            super::simd::cbq(out, w, y, cols, rows);
            DO_.fetch_add(1, Ordering::Relaxed);
        }
    }
}



#[inline]
pub fn ayw(qx: &mut [f32], ad: &[f32], x: &[f32], cols: usize, rows: usize) {
    match eex() {
        Backend::AmdGpu => {
            super::simd::ayw(qx, ad, x, cols, rows);
            DO_.fetch_add(1, Ordering::Relaxed);
        }
        Backend::CpuSimd => {
            super::simd::ayw(qx, ad, x, cols, rows);
            DO_.fetch_add(1, Ordering::Relaxed);
        }
    }
}


#[inline]
pub fn aox(out: &mut [f32], x: &[f32], tv: &[f32]) -> f32 {
    let aeg = super::simd::aox(out, x, tv);
    DO_.fetch_add(1, Ordering::Relaxed);
    aeg
}







fn mfu(out: &mut [f32], w: &[f32], x: &[f32], cols: usize, rows: usize) {
    
    if !crate::drivers::amdgpu::compute::is_ready() {
        super::simd::tk(out, w, x, cols, rows);
        DO_.fetch_add(1, Ordering::Relaxed);
        return;
    }

    
    
    
    
    
    
    
    
    
    
    
    
    
    
    super::simd::tk(out, w, x, cols, rows);
    DO_.fetch_add(1, Ordering::Relaxed);
}



pub fn hat(_weights: &super::model::TransformerWeights) -> Result<usize, &'static str> {
    if !crate::drivers::amdgpu::aud() {
        return Err("No AMD GPU detected");
    }

    if !crate::drivers::amdgpu::sdma::is_ready() {
        return Err("SDMA not ready");
    }

    
    
    
    
    
    
    
    
    
    
    
    
    

    Err("GPU weight upload not yet implemented (need real hardware)")
}






pub fn summary() -> String {
    let backend = if cyv() { "GPU (AMD RDNA)" } else { "CPU (SSE2 SIMD)" };
    let mfw = AVT_.load(Ordering::Relaxed);
    let kyp = DO_.load(Ordering::Relaxed);
    alloc::format!("Backend: {}, GPU ops: {}, CPU ops: {}", backend, mfw, kyp)
}


pub fn info_lines() -> Vec<String> {
    let mut lines = Vec::new();
    let backend = if cyv() { "GPU (AMD RDNA)" } else { "CPU (SSE2 SIMD)" };
    lines.push(alloc::format!("Compute: {}", backend));
    lines.push(alloc::format!("  GPU ops:  {}", AVT_.load(Ordering::Relaxed)));
    lines.push(alloc::format!("  CPU ops:  {}", DO_.load(Ordering::Relaxed)));

    if cyv() {
        if let Some(info) = crate::drivers::amdgpu::rk() {
            lines.push(alloc::format!("  GPU: {} ({})", info.gpu_name(), info.vram_string()));
        }
    }

    lines
}
