//! SSE2-optimized math primitives for the Jarvis transformer
//!
//! Provides 2-4x speedup over scalar matvec/dot for the hot paths
//! in both forward (inference) and backward (backprop) passes.
//!
//! Uses 4-wide f32 SSE2 operations with 4 accumulators per dot product
//! for instruction-level parallelism. All dimensions (D_MODEL=256, D_FF=1024,
//! VOCAB_SIZE=256) are divisible by 16, so the fast path covers 100% of cases.
//!
//! ## Safety
//! SSE/SSE2 must be enabled via cpu::simd::enable_sse() before calling.
//! All functions are safe wrappers around unsafe SSE2 intrinsics.

#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

#[cfg(target_arch = "aarch64")]
use core::arch::aarch64::*;

use core::sync::atomic::{AtomicBool, Ordering};

/// Runtime dispatch flag — set once during init after CPUID + XCR0 setup
static AVX2_FMA: AtomicBool = AtomicBool::new(false);

/// Initialize SIMD dispatch based on detected CPU features.
/// Must be called AFTER cpu::init() has enabled AVX via XCR0.
pub fn initialize_dispatch() {
    #[cfg(target_arch = "x86_64")]
    {
        let caps = crate::cpu::capabilities();
        let has_avx2_fma = caps.map(|c| c.avx2 && c.fma).unwrap_or(false);

        if has_avx2_fma {
            AVX2_FMA.store(true, Ordering::Release);
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

/// Check if AVX2+FMA fast path is available
#[inline(always)]
fn use_avx2_fma() -> bool {
    AVX2_FMA.load(Ordering::Relaxed)
}

// ═══════════════════════════════════════════════════════════════════════════════
// SSE2 DOT PRODUCT (4 accumulators × 4 floats = 16 per iteration)
// ═══════════════════════════════════════════════════════════════════════════════

/// SSE2-accelerated dot product: Σ a[i]*b[i] for i in 0..len
///
/// Uses 4 independent accumulators to exploit out-of-order execution.
/// Fast path: processes 16 floats per iteration when len >= 16.
#[cfg(target_arch = "x86_64")]
#[inline]
fn dot_simd(a: &[f32], b: &[f32], len: usize) -> f32 {
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
        let ap = a.as_ptr();
        let bp = b.as_ptr();

        let mut acc0 = _mm_setzero_ps();
        let mut acc1 = _mm_setzero_ps();
        let mut acc2 = _mm_setzero_ps();
        let mut acc3 = _mm_setzero_ps();

        // Main loop: 16 floats per iteration (4 SSE2 ops × 4 lanes)
        let chunks16 = len / 16;
        for i in 0..chunks16 {
            let base = i * 16;
            let a0 = _mm_loadu_ps(ap.add(base));
            let b0 = _mm_loadu_ps(bp.add(base));
            acc0 = _mm_add_ps(acc0, _mm_mul_ps(a0, b0));

            let a1 = _mm_loadu_ps(ap.add(base + 4));
            let b1 = _mm_loadu_ps(bp.add(base + 4));
            acc1 = _mm_add_ps(acc1, _mm_mul_ps(a1, b1));

            let a2 = _mm_loadu_ps(ap.add(base + 8));
            let b2 = _mm_loadu_ps(bp.add(base + 8));
            acc2 = _mm_add_ps(acc2, _mm_mul_ps(a2, b2));

            let a3 = _mm_loadu_ps(ap.add(base + 12));
            let b3 = _mm_loadu_ps(bp.add(base + 12));
            acc3 = _mm_add_ps(acc3, _mm_mul_ps(a3, b3));
        }

        // Combine 4 accumulators into 1
        acc0 = _mm_add_ps(acc0, acc1);
        acc2 = _mm_add_ps(acc2, acc3);
        acc0 = _mm_add_ps(acc0, acc2);

        // Handle remaining 4-element chunks
        let rem_start = chunks16 * 16;
        let rem4 = (len - rem_start) / 4;
        for i in 0..rem4 {
            let offset = rem_start + i * 4;
            let av = _mm_loadu_ps(ap.add(offset));
            let bv = _mm_loadu_ps(bp.add(offset));
            acc0 = _mm_add_ps(acc0, _mm_mul_ps(av, bv));
        }

        // SSE2 horizontal sum: [a,b,c,d] → a+b+c+d
        let hi = _mm_movehl_ps(acc0, acc0);     // [c, d, c, d]
        let sum = _mm_add_ps(acc0, hi);          // [a+c, b+d, ...]
        let shuf = _mm_shuffle_ps(sum, sum, 1);  // [b+d, ...]
        let total = _mm_add_ss(sum, shuf);       // [a+b+c+d, ...]
        let mut result = _mm_cvtss_f32(total);

        // Scalar tail (len % 4)
        let scalar_start = rem_start + rem4 * 4;
        for i in scalar_start..len {
            result += *ap.add(i) * *bp.add(i);
        }

        result
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// NEON DOT PRODUCT (4 accumulators × 4 floats = 16 per iteration, with FMA)
// ═══════════════════════════════════════════════════════════════════════════════

/// NEON-accelerated dot product for aarch64: Σ a[i]*b[i] for i in 0..len
///
/// Uses 4 independent accumulators with fused multiply-add (vfmaq_f32)
/// for instruction-level parallelism. Processes 16 floats per iteration.
#[cfg(target_arch = "aarch64")]
#[inline]
fn dot_neon(a: &[f32], b: &[f32], len: usize) -> f32 {
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
        let ap = a.as_ptr();
        let bp = b.as_ptr();

        let mut acc0 = vdupq_n_f32(0.0);
        let mut acc1 = vdupq_n_f32(0.0);
        let mut acc2 = vdupq_n_f32(0.0);
        let mut acc3 = vdupq_n_f32(0.0);

        // Main loop: 16 floats per iteration (4 NEON ops × 4 lanes)
        let chunks16 = len / 16;
        for i in 0..chunks16 {
            let base = i * 16;
            let a0 = vld1q_f32(ap.add(base));
            let b0 = vld1q_f32(bp.add(base));
            acc0 = vfmaq_f32(acc0, a0, b0);

            let a1 = vld1q_f32(ap.add(base + 4));
            let b1 = vld1q_f32(bp.add(base + 4));
            acc1 = vfmaq_f32(acc1, a1, b1);

            let a2 = vld1q_f32(ap.add(base + 8));
            let b2 = vld1q_f32(bp.add(base + 8));
            acc2 = vfmaq_f32(acc2, a2, b2);

            let a3 = vld1q_f32(ap.add(base + 12));
            let b3 = vld1q_f32(bp.add(base + 12));
            acc3 = vfmaq_f32(acc3, a3, b3);
        }

        // Combine 4 accumulators
        acc0 = vaddq_f32(acc0, acc1);
        acc2 = vaddq_f32(acc2, acc3);
        acc0 = vaddq_f32(acc0, acc2);

        // Handle remaining 4-element chunks
        let rem_start = chunks16 * 16;
        let rem4 = (len - rem_start) / 4;
        for i in 0..rem4 {
            let offset = rem_start + i * 4;
            let av = vld1q_f32(ap.add(offset));
            let bv = vld1q_f32(bp.add(offset));
            acc0 = vfmaq_f32(acc0, av, bv);
        }

        // NEON horizontal sum: vaddvq_f32 sums all 4 lanes
        let mut result = vaddvq_f32(acc0);

        // Scalar tail (len % 4)
        let scalar_start = rem_start + rem4 * 4;
        for i in scalar_start..len {
            result += *ap.add(i) * *bp.add(i);
        }

        result
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// MATRIX-VECTOR MULTIPLY (SSE2 / NEON)
// ═══════════════════════════════════════════════════════════════════════════════

/// SSE2-accelerated matrix-vector multiply: out[r] = Σ_c W[r*cols+c] * x[c]
///
/// W is [rows × cols] row-major, x is [cols], out is [rows].
/// Each row's dot product uses 4 SSE2 accumulators for ILP.
///
/// This is the hottest function in training — called ~18 times per token
/// per forward+backward step (Q/K/V/O + gate/up/down per layer + output).
#[cfg(target_arch = "x86_64")]
// Fonction publique — appelable depuis d'autres modules.
pub fn matvec(out: &mut [f32], w: &[f32], x: &[f32], cols: usize, rows: usize) {
    for r in 0..rows {
        let base = r * cols;
        out[r] = dot_simd(&w[base..base + cols], x, cols);
    }
}

/// NEON-accelerated matvec for aarch64: 4-wide f32 with 4 accumulators
#[cfg(target_arch = "aarch64")]
// Fonction publique — appelable depuis d'autres modules.
pub fn matvec(out: &mut [f32], w: &[f32], x: &[f32], cols: usize, rows: usize) {
    for r in 0..rows {
        out[r] = dot_neon(&w[r * cols..r * cols + cols], x, cols);
    }
}

/// Scalar fallback for riscv64 / other architectures
#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
// Fonction publique — appelable depuis d'autres modules.
pub fn matvec(out: &mut [f32], w: &[f32], x: &[f32], cols: usize, rows: usize) {
    for r in 0..rows {
        let mut sum = 0.0f32;
        let base = r * cols;
        for c in 0..cols {
            sum += w[base + c] * x[c];
        }
        out[r] = sum;
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRANSPOSE MATRIX-VECTOR MULTIPLY (SSE2)
// ═══════════════════════════════════════════════════════════════════════════════

/// SSE2-accelerated transpose matvec: out[c] = Σ_r W[r*cols+c] * y[r]
///
/// This is W^T @ y — used in backward passes to propagate gradients
/// through weight matrices. Restructured as scatter-add for SIMD:
/// for each row r, broadcast y[r] and add W[r*cols..]*y[r] to out[].
///
/// Called ~10 times per token in backward (W_o^T, W_q^T, W_k^T, W_v^T,
/// W_gate^T, W_up^T, W_down^T, W_output^T).
#[cfg(target_arch = "x86_64")]
// Fonction publique — appelable depuis d'autres modules.
pub fn matvec_transpose(out: &mut [f32], w: &[f32], y: &[f32], cols: usize, rows: usize) {
    if use_avx2_fma() {
                // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { matvec_transpose_avx2(out, w, y, cols, rows); }
        return;
    }
    // Zero output first
    for v in out[..cols].iter_mut() { *v = 0.0; }

        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
        let wp = w.as_ptr();
        let op = out.as_mut_ptr();

        for r in 0..rows {
            let yr = y[r];
            if yr == 0.0 { continue; } // Skip zero gradients

            let yr_vec = _mm_set1_ps(yr);
            let base = r * cols;

            // Main loop: 16 floats per iteration
            let chunks16 = cols / 16;
            for i in 0..chunks16 {
                let offset = base + i * 16;
                let out_off = i * 16;

                let w0 = _mm_loadu_ps(wp.add(offset));
                let o0 = _mm_loadu_ps(op.add(out_off));
                _mm_storeu_ps(op.add(out_off), _mm_add_ps(o0, _mm_mul_ps(w0, yr_vec)));

                let w1 = _mm_loadu_ps(wp.add(offset + 4));
                let o1 = _mm_loadu_ps(op.add(out_off + 4));
                _mm_storeu_ps(op.add(out_off + 4), _mm_add_ps(o1, _mm_mul_ps(w1, yr_vec)));

                let w2 = _mm_loadu_ps(wp.add(offset + 8));
                let o2 = _mm_loadu_ps(op.add(out_off + 8));
                _mm_storeu_ps(op.add(out_off + 8), _mm_add_ps(o2, _mm_mul_ps(w2, yr_vec)));

                let w3 = _mm_loadu_ps(wp.add(offset + 12));
                let o3 = _mm_loadu_ps(op.add(out_off + 12));
                _mm_storeu_ps(op.add(out_off + 12), _mm_add_ps(o3, _mm_mul_ps(w3, yr_vec)));
            }

            // Remaining 4-element chunks
            let rem_start = chunks16 * 16;
            let rem4 = (cols - rem_start) / 4;
            for i in 0..rem4 {
                let offset = base + rem_start + i * 4;
                let out_off = rem_start + i * 4;
                let wv = _mm_loadu_ps(wp.add(offset));
                let ov = _mm_loadu_ps(op.add(out_off));
                _mm_storeu_ps(op.add(out_off), _mm_add_ps(ov, _mm_mul_ps(wv, yr_vec)));
            }

            // Scalar tail
            let scalar_start = rem_start + rem4 * 4;
            for c in scalar_start..cols {
                *op.add(c) += *wp.add(base + c) * yr;
            }
        }
    }
}

/// NEON-accelerated transpose matvec for aarch64
#[cfg(target_arch = "aarch64")]
// Fonction publique — appelable depuis d'autres modules.
pub fn matvec_transpose(out: &mut [f32], w: &[f32], y: &[f32], cols: usize, rows: usize) {
    for v in out[..cols].iter_mut() { *v = 0.0; }
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
        let wp = w.as_ptr();
        let op = out.as_mut_ptr();
        for r in 0..rows {
            let yr = y[r];
            if yr == 0.0 { continue; }
            let yr_vec = vdupq_n_f32(yr);
            let base = r * cols;
            let chunks16 = cols / 16;
            for i in 0..chunks16 {
                let offset = base + i * 16;
                let out_off = i * 16;
                let w0 = vld1q_f32(wp.add(offset));
                let o0 = vld1q_f32(op.add(out_off));
                vst1q_f32(op.add(out_off), vfmaq_f32(o0, w0, yr_vec));
                let w1 = vld1q_f32(wp.add(offset + 4));
                let o1 = vld1q_f32(op.add(out_off + 4));
                vst1q_f32(op.add(out_off + 4), vfmaq_f32(o1, w1, yr_vec));
                let w2 = vld1q_f32(wp.add(offset + 8));
                let o2 = vld1q_f32(op.add(out_off + 8));
                vst1q_f32(op.add(out_off + 8), vfmaq_f32(o2, w2, yr_vec));
                let w3 = vld1q_f32(wp.add(offset + 12));
                let o3 = vld1q_f32(op.add(out_off + 12));
                vst1q_f32(op.add(out_off + 12), vfmaq_f32(o3, w3, yr_vec));
            }
            let rem_start = chunks16 * 16;
            let rem4 = (cols - rem_start) / 4;
            for i in 0..rem4 {
                let offset = base + rem_start + i * 4;
                let out_off = rem_start + i * 4;
                let wv = vld1q_f32(wp.add(offset));
                let ov = vld1q_f32(op.add(out_off));
                vst1q_f32(op.add(out_off), vfmaq_f32(ov, wv, yr_vec));
            }
            let scalar_start = rem_start + rem4 * 4;
            for c in scalar_start..cols {
                *op.add(c) += *wp.add(base + c) * yr;
            }
        }
    }
}

/// Scalar fallback for riscv64 / other architectures
#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
// Fonction publique — appelable depuis d'autres modules.
pub fn matvec_transpose(out: &mut [f32], w: &[f32], y: &[f32], cols: usize, rows: usize) {
    for v in out[..cols].iter_mut() { *v = 0.0; }
    for r in 0..rows {
        let base = r * cols;
        for c in 0..cols {
            out[c] += w[base + c] * y[r];
        }
    }
}

/// SSE2 transpose matvec that ACCUMULATES into out (doesn't zero first).
/// Used when multiple W^T @ dy need to be summed into the same gradient buffer.
#[cfg(target_arch = "x86_64")]
// Fonction publique — appelable depuis d'autres modules.
pub fn matvec_transpose_accum(out: &mut [f32], w: &[f32], y: &[f32], cols: usize, rows: usize) {
    if use_avx2_fma() {
                // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { matvec_transpose_accum_avx2(out, w, y, cols, rows); }
        return;
    }
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
        let wp = w.as_ptr();
        let op = out.as_mut_ptr();

        for r in 0..rows {
            let yr = y[r];
            if yr == 0.0 { continue; }

            let yr_vec = _mm_set1_ps(yr);
            let base = r * cols;

            let chunks16 = cols / 16;
            for i in 0..chunks16 {
                let offset = base + i * 16;
                let out_off = i * 16;

                let w0 = _mm_loadu_ps(wp.add(offset));
                let o0 = _mm_loadu_ps(op.add(out_off));
                _mm_storeu_ps(op.add(out_off), _mm_add_ps(o0, _mm_mul_ps(w0, yr_vec)));

                let w1 = _mm_loadu_ps(wp.add(offset + 4));
                let o1 = _mm_loadu_ps(op.add(out_off + 4));
                _mm_storeu_ps(op.add(out_off + 4), _mm_add_ps(o1, _mm_mul_ps(w1, yr_vec)));

                let w2 = _mm_loadu_ps(wp.add(offset + 8));
                let o2 = _mm_loadu_ps(op.add(out_off + 8));
                _mm_storeu_ps(op.add(out_off + 8), _mm_add_ps(o2, _mm_mul_ps(w2, yr_vec)));

                let w3 = _mm_loadu_ps(wp.add(offset + 12));
                let o3 = _mm_loadu_ps(op.add(out_off + 12));
                _mm_storeu_ps(op.add(out_off + 12), _mm_add_ps(o3, _mm_mul_ps(w3, yr_vec)));
            }

            let rem_start = chunks16 * 16;
            let rem4 = (cols - rem_start) / 4;
            for i in 0..rem4 {
                let offset = base + rem_start + i * 4;
                let out_off = rem_start + i * 4;
                let wv = _mm_loadu_ps(wp.add(offset));
                let ov = _mm_loadu_ps(op.add(out_off));
                _mm_storeu_ps(op.add(out_off), _mm_add_ps(ov, _mm_mul_ps(wv, yr_vec)));
            }

            let scalar_start = rem_start + rem4 * 4;
            for c in scalar_start..cols {
                *op.add(c) += *wp.add(base + c) * yr;
            }
        }
    }
}

/// NEON-accelerated transpose matvec accum for aarch64
#[cfg(target_arch = "aarch64")]
// Fonction publique — appelable depuis d'autres modules.
pub fn matvec_transpose_accum(out: &mut [f32], w: &[f32], y: &[f32], cols: usize, rows: usize) {
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
        let wp = w.as_ptr();
        let op = out.as_mut_ptr();
        for r in 0..rows {
            let yr = y[r];
            if yr == 0.0 { continue; }
            let yr_vec = vdupq_n_f32(yr);
            let base = r * cols;
            let chunks16 = cols / 16;
            for i in 0..chunks16 {
                let offset = base + i * 16;
                let out_off = i * 16;
                let w0 = vld1q_f32(wp.add(offset));
                let o0 = vld1q_f32(op.add(out_off));
                vst1q_f32(op.add(out_off), vfmaq_f32(o0, w0, yr_vec));
                let w1 = vld1q_f32(wp.add(offset + 4));
                let o1 = vld1q_f32(op.add(out_off + 4));
                vst1q_f32(op.add(out_off + 4), vfmaq_f32(o1, w1, yr_vec));
                let w2 = vld1q_f32(wp.add(offset + 8));
                let o2 = vld1q_f32(op.add(out_off + 8));
                vst1q_f32(op.add(out_off + 8), vfmaq_f32(o2, w2, yr_vec));
                let w3 = vld1q_f32(wp.add(offset + 12));
                let o3 = vld1q_f32(op.add(out_off + 12));
                vst1q_f32(op.add(out_off + 12), vfmaq_f32(o3, w3, yr_vec));
            }
            let rem_start = chunks16 * 16;
            let rem4 = (cols - rem_start) / 4;
            for i in 0..rem4 {
                let offset = base + rem_start + i * 4;
                let out_off = rem_start + i * 4;
                let wv = vld1q_f32(wp.add(offset));
                let ov = vld1q_f32(op.add(out_off));
                vst1q_f32(op.add(out_off), vfmaq_f32(ov, wv, yr_vec));
            }
            let scalar_start = rem_start + rem4 * 4;
            for c in scalar_start..cols {
                *op.add(c) += *wp.add(base + c) * yr;
            }
        }
    }
}

#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
// Fonction publique — appelable depuis d'autres modules.
pub fn matvec_transpose_accum(out: &mut [f32], w: &[f32], y: &[f32], cols: usize, rows: usize) {
    for r in 0..rows {
        let base = r * cols;
        for c in 0..cols {
            out[c] += w[base + c] * y[r];
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// OUTER PRODUCT ACCUMULATE (for weight gradients)
// ═══════════════════════════════════════════════════════════════════════════════

/// SSE2-accelerated outer product: dW[r*cols+c] += dy[r] * x[c]
///
/// Used for weight gradient accumulation: dW += dy ⊗ x
/// Called for every weight matrix gradient (9 per layer × 4 layers + output).
#[cfg(target_arch = "x86_64")]
// Fonction publique — appelable depuis d'autres modules.
pub fn outer_product_accum(dw: &mut [f32], dy: &[f32], x: &[f32], cols: usize, rows: usize) {
    if use_avx2_fma() {
                // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { outer_product_accum_avx2(dw, dy, x, cols, rows); }
        return;
    }
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
        let dwp = dw.as_mut_ptr();
        let xp = x.as_ptr();

        for r in 0..rows {
            let dyr = dy[r];
            if dyr == 0.0 { continue; }

            let dyr_vec = _mm_set1_ps(dyr);
            let base = r * cols;

            // Main loop: 16 floats per iteration
            let chunks16 = cols / 16;
            for i in 0..chunks16 {
                let offset = base + i * 16;
                let x_off = i * 16;

                let x0 = _mm_loadu_ps(xp.add(x_off));
                let d0 = _mm_loadu_ps(dwp.add(offset));
                _mm_storeu_ps(dwp.add(offset), _mm_add_ps(d0, _mm_mul_ps(dyr_vec, x0)));

                let x1 = _mm_loadu_ps(xp.add(x_off + 4));
                let d1 = _mm_loadu_ps(dwp.add(offset + 4));
                _mm_storeu_ps(dwp.add(offset + 4), _mm_add_ps(d1, _mm_mul_ps(dyr_vec, x1)));

                let x2 = _mm_loadu_ps(xp.add(x_off + 8));
                let d2 = _mm_loadu_ps(dwp.add(offset + 8));
                _mm_storeu_ps(dwp.add(offset + 8), _mm_add_ps(d2, _mm_mul_ps(dyr_vec, x2)));

                let x3 = _mm_loadu_ps(xp.add(x_off + 12));
                let d3 = _mm_loadu_ps(dwp.add(offset + 12));
                _mm_storeu_ps(dwp.add(offset + 12), _mm_add_ps(d3, _mm_mul_ps(dyr_vec, x3)));
            }

            // Remaining 4-element chunks
            let rem_start = chunks16 * 16;
            let rem4 = (cols - rem_start) / 4;
            for i in 0..rem4 {
                let offset = base + rem_start + i * 4;
                let x_off = rem_start + i * 4;
                let xv = _mm_loadu_ps(xp.add(x_off));
                let dv = _mm_loadu_ps(dwp.add(offset));
                _mm_storeu_ps(dwp.add(offset), _mm_add_ps(dv, _mm_mul_ps(dyr_vec, xv)));
            }

            // Scalar tail
            let scalar_start = rem_start + rem4 * 4;
            for c in scalar_start..cols {
                *dwp.add(base + c) += dyr * *xp.add(c);
            }
        }
    }
}

/// NEON-accelerated outer product accum for aarch64
#[cfg(target_arch = "aarch64")]
// Fonction publique — appelable depuis d'autres modules.
pub fn outer_product_accum(dw: &mut [f32], dy: &[f32], x: &[f32], cols: usize, rows: usize) {
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
        let dwp = dw.as_mut_ptr();
        let xp = x.as_ptr();
        for r in 0..rows {
            let dyr = dy[r];
            if dyr == 0.0 { continue; }
            let dyr_vec = vdupq_n_f32(dyr);
            let base = r * cols;
            let chunks16 = cols / 16;
            for i in 0..chunks16 {
                let offset = base + i * 16;
                let x_off = i * 16;
                let x0 = vld1q_f32(xp.add(x_off));
                let d0 = vld1q_f32(dwp.add(offset));
                vst1q_f32(dwp.add(offset), vfmaq_f32(d0, dyr_vec, x0));
                let x1 = vld1q_f32(xp.add(x_off + 4));
                let d1 = vld1q_f32(dwp.add(offset + 4));
                vst1q_f32(dwp.add(offset + 4), vfmaq_f32(d1, dyr_vec, x1));
                let x2 = vld1q_f32(xp.add(x_off + 8));
                let d2 = vld1q_f32(dwp.add(offset + 8));
                vst1q_f32(dwp.add(offset + 8), vfmaq_f32(d2, dyr_vec, x2));
                let x3 = vld1q_f32(xp.add(x_off + 12));
                let d3 = vld1q_f32(dwp.add(offset + 12));
                vst1q_f32(dwp.add(offset + 12), vfmaq_f32(d3, dyr_vec, x3));
            }
            let rem_start = chunks16 * 16;
            let rem4 = (cols - rem_start) / 4;
            for i in 0..rem4 {
                let offset = base + rem_start + i * 4;
                let x_off = rem_start + i * 4;
                let xv = vld1q_f32(xp.add(x_off));
                let dv = vld1q_f32(dwp.add(offset));
                vst1q_f32(dwp.add(offset), vfmaq_f32(dv, dyr_vec, xv));
            }
            let scalar_start = rem_start + rem4 * 4;
            for c in scalar_start..cols {
                *dwp.add(base + c) += dyr * *xp.add(c);
            }
        }
    }
}

/// Scalar fallback for riscv64 / other architectures
#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
// Fonction publique — appelable depuis d'autres modules.
pub fn outer_product_accum(dw: &mut [f32], dy: &[f32], x: &[f32], cols: usize, rows: usize) {
    for r in 0..rows {
        let base = r * cols;
        for c in 0..cols {
            dw[base + c] += dy[r] * x[c];
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// RMSNORM (SSE2 / NEON)
// ═══════════════════════════════════════════════════════════════════════════════

/// SSE2-accelerated RMSNorm: out = (x / RMS(x)) * weight
///
/// Returns the RMS value for use in the backward pass.
/// Accelerates both the sum-of-squares and the normalize-multiply loop.
#[cfg(target_arch = "x86_64")]
// Fonction publique — appelable depuis d'autres modules.
pub fn rmsnorm(out: &mut [f32], x: &[f32], weight: &[f32]) -> f32 {
    let n = x.len();

    // SSE2 sum of squares
    let ss = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
        let xp = x.as_ptr();
        let mut acc0 = _mm_setzero_ps();
        let mut acc1 = _mm_setzero_ps();

        let chunks8 = n / 8;
        for i in 0..chunks8 {
            let base = i * 8;
            let v0 = _mm_loadu_ps(xp.add(base));
            acc0 = _mm_add_ps(acc0, _mm_mul_ps(v0, v0));
            let v1 = _mm_loadu_ps(xp.add(base + 4));
            acc1 = _mm_add_ps(acc1, _mm_mul_ps(v1, v1));
        }
        acc0 = _mm_add_ps(acc0, acc1);

        // Remaining
        let rem_start = chunks8 * 8;
        for i in (rem_start..n).step_by(4) {
            if i + 4 <= n {
                let v = _mm_loadu_ps(xp.add(i));
                acc0 = _mm_add_ps(acc0, _mm_mul_ps(v, v));
            }
        }

        let hi = _mm_movehl_ps(acc0, acc0);
        let sum = _mm_add_ps(acc0, hi);
        let shuf = _mm_shuffle_ps(sum, sum, 1);
        let total = _mm_add_ss(sum, shuf);
        let mut result = _mm_cvtss_f32(total);

        // Scalar tail
        let scalar_start = (n / 4) * 4;
        for i in scalar_start..n {
            result += *xp.add(i) * *xp.add(i);
        }
        result
    };

    let rms = super::backprop::approx_sqrt(ss / n as f32 + super::model::RMS_EPS);
    let inv_rms = 1.0 / rms;

    // SSE2 normalize + scale
    unsafe {
        let xp = x.as_ptr();
        let wp = weight.as_ptr();
        let op = out.as_mut_ptr();
        let inv_rms_vec = _mm_set1_ps(inv_rms);

        let chunks4 = n / 4;
        for i in 0..chunks4 {
            let off = i * 4;
            let xv = _mm_loadu_ps(xp.add(off));
            let wv = _mm_loadu_ps(wp.add(off));
            let normed = _mm_mul_ps(xv, inv_rms_vec);
            _mm_storeu_ps(op.add(off), _mm_mul_ps(normed, wv));
        }

        // Scalar tail
        let scalar_start = chunks4 * 4;
        for i in scalar_start..n {
            *op.add(i) = *xp.add(i) * inv_rms * *wp.add(i);
        }
    }

    rms
}

/// NEON-accelerated RMSNorm for aarch64
#[cfg(target_arch = "aarch64")]
// Fonction publique — appelable depuis d'autres modules.
pub fn rmsnorm(out: &mut [f32], x: &[f32], weight: &[f32]) -> f32 {
    let n = x.len();
    let ss = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
        let xp = x.as_ptr();
        let mut acc0 = vdupq_n_f32(0.0);
        let mut acc1 = vdupq_n_f32(0.0);
        let chunks8 = n / 8;
        for i in 0..chunks8 {
            let base = i * 8;
            let v0 = vld1q_f32(xp.add(base));
            acc0 = vfmaq_f32(acc0, v0, v0);
            let v1 = vld1q_f32(xp.add(base + 4));
            acc1 = vfmaq_f32(acc1, v1, v1);
        }
        acc0 = vaddq_f32(acc0, acc1);
        let mut result = vaddvq_f32(acc0);
        let scalar_start = (n / 4) * 4;
        for i in scalar_start..n {
            result += *xp.add(i) * *xp.add(i);
        }
        result
    };
    let rms = super::backprop::approx_sqrt(ss / n as f32 + super::model::RMS_EPS);
    let inv_rms = 1.0 / rms;
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
        let xp = x.as_ptr();
        let wp = weight.as_ptr();
        let op = out.as_mut_ptr();
        let inv_rms_vec = vdupq_n_f32(inv_rms);
        let chunks4 = n / 4;
        for i in 0..chunks4 {
            let off = i * 4;
            let xv = vld1q_f32(xp.add(off));
            let wv = vld1q_f32(wp.add(off));
            let normed = vmulq_f32(xv, inv_rms_vec);
            vst1q_f32(op.add(off), vmulq_f32(normed, wv));
        }
        let scalar_start = chunks4 * 4;
        for i in scalar_start..n {
            *op.add(i) = *xp.add(i) * inv_rms * *wp.add(i);
        }
    }
    rms
}

/// Scalar fallback for riscv64 / other architectures
#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
// Fonction publique — appelable depuis d'autres modules.
pub fn rmsnorm(out: &mut [f32], x: &[f32], weight: &[f32]) -> f32 {
    let n = x.len();
    let mut ss = 0.0f32;
    for &v in x { ss += v * v; }
    let rms = super::backprop::approx_sqrt(ss / n as f32 + super::model::RMS_EPS);
    let inv_rms = 1.0 / rms;
    for i in 0..n {
        out[i] = x[i] * inv_rms * weight[i];
    }
    rms
}

// ═══════════════════════════════════════════════════════════════════════════════
// VECTOR OPERATIONS (SSE2 / NEON)
// ═══════════════════════════════════════════════════════════════════════════════

/// SSE2-accelerated vector add: out[i] = a[i] + b[i]
#[cfg(target_arch = "x86_64")]
// Fonction publique — appelable depuis d'autres modules.
pub fn vec_add(out: &mut [f32], a: &[f32], b: &[f32], len: usize) {
    if use_avx2_fma() {
                // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { vec_add_avx2(out, a, b, len); }
        return;
    }
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
        let ap = a.as_ptr();
        let bp = b.as_ptr();
        let op = out.as_mut_ptr();

        let chunks4 = len / 4;
        for i in 0..chunks4 {
            let off = i * 4;
            let av = _mm_loadu_ps(ap.add(off));
            let bv = _mm_loadu_ps(bp.add(off));
            _mm_storeu_ps(op.add(off), _mm_add_ps(av, bv));
        }

        let scalar_start = chunks4 * 4;
        for i in scalar_start..len {
            *op.add(i) = *ap.add(i) + *bp.add(i);
        }
    }
}

#[cfg(target_arch = "aarch64")]
// Fonction publique — appelable depuis d'autres modules.
pub fn vec_add(out: &mut [f32], a: &[f32], b: &[f32], len: usize) {
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
        let ap = a.as_ptr();
        let bp = b.as_ptr();
        let op = out.as_mut_ptr();
        let chunks4 = len / 4;
        for i in 0..chunks4 {
            let off = i * 4;
            let av = vld1q_f32(ap.add(off));
            let bv = vld1q_f32(bp.add(off));
            vst1q_f32(op.add(off), vaddq_f32(av, bv));
        }
        let scalar_start = chunks4 * 4;
        for i in scalar_start..len {
            *op.add(i) = *ap.add(i) + *bp.add(i);
        }
    }
}

#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
// Fonction publique — appelable depuis d'autres modules.
pub fn vec_add(out: &mut [f32], a: &[f32], b: &[f32], len: usize) {
    for i in 0..len { out[i] = a[i] + b[i]; }
}

/// SSE2-accelerated in-place vector add: a[i] += b[i]
#[cfg(target_arch = "x86_64")]
// Fonction publique — appelable depuis d'autres modules.
pub fn vec_add_inplace(a: &mut [f32], b: &[f32], len: usize) {
    if use_avx2_fma() {
                // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { vec_add_inplace_avx2(a, b, len); }
        return;
    }
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
        let ap = a.as_mut_ptr();
        let bp = b.as_ptr();

        let chunks4 = len / 4;
        for i in 0..chunks4 {
            let off = i * 4;
            let av = _mm_loadu_ps(ap.add(off));
            let bv = _mm_loadu_ps(bp.add(off));
            _mm_storeu_ps(ap.add(off), _mm_add_ps(av, bv));
        }

        let scalar_start = chunks4 * 4;
        for i in scalar_start..len {
            *ap.add(i) += *bp.add(i);
        }
    }
}

#[cfg(target_arch = "aarch64")]
// Fonction publique — appelable depuis d'autres modules.
pub fn vec_add_inplace(a: &mut [f32], b: &[f32], len: usize) {
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
        let ap = a.as_mut_ptr();
        let bp = b.as_ptr();
        let chunks4 = len / 4;
        for i in 0..chunks4 {
            let off = i * 4;
            let av = vld1q_f32(ap.add(off));
            let bv = vld1q_f32(bp.add(off));
            vst1q_f32(ap.add(off), vaddq_f32(av, bv));
        }
        let scalar_start = chunks4 * 4;
        for i in scalar_start..len {
            *ap.add(i) += *bp.add(i);
        }
    }
}

#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
// Fonction publique — appelable depuis d'autres modules.
pub fn vec_add_inplace(a: &mut [f32], b: &[f32], len: usize) {
    for i in 0..len { a[i] += b[i]; }
}

/// SSE2-accelerated vector scale: out[i] = a[i] * scalar
#[cfg(target_arch = "x86_64")]
// Fonction publique — appelable depuis d'autres modules.
pub fn vec_scale(a: &mut [f32], scalar: f32, len: usize) {
    if use_avx2_fma() {
                // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { vec_scale_avx2(a, scalar, len); }
        return;
    }
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
        let ap = a.as_mut_ptr();
        let sv = _mm_set1_ps(scalar);

        let chunks4 = len / 4;
        for i in 0..chunks4 {
            let off = i * 4;
            let v = _mm_loadu_ps(ap.add(off));
            _mm_storeu_ps(ap.add(off), _mm_mul_ps(v, sv));
        }

        let scalar_start = chunks4 * 4;
        for i in scalar_start..len {
            *ap.add(i) *= scalar;
        }
    }
}

#[cfg(target_arch = "aarch64")]
// Fonction publique — appelable depuis d'autres modules.
pub fn vec_scale(a: &mut [f32], scalar: f32, len: usize) {
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
        let ap = a.as_mut_ptr();
        let sv = vdupq_n_f32(scalar);
        let chunks4 = len / 4;
        for i in 0..chunks4 {
            let off = i * 4;
            let v = vld1q_f32(ap.add(off));
            vst1q_f32(ap.add(off), vmulq_f32(v, sv));
        }
        let scalar_start = chunks4 * 4;
        for i in scalar_start..len {
            *ap.add(i) *= scalar;
        }
    }
}

#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
// Fonction publique — appelable depuis d'autres modules.
pub fn vec_scale(a: &mut [f32], scalar: f32, len: usize) {
    for i in 0..len { a[i] *= scalar; }
}

// ═══════════════════════════════════════════════════════════════════════════════
// AVX2 + FMA KERNELS (8-wide, fused multiply-add)
//
// 2-3× faster than SSE2 for all neural network hot paths:
// - 8 floats per instruction (vs 4 for SSE2)
// - FMA: a*b+c in 1 cycle (vs separate mul+add = 2 cycles)
// - 4 accumulators × 8 lanes = 32 floats per iteration in dot products
//
// Dispatch: checked at runtime via use_avx2_fma() flag, set during init.
// Safety: all AVX2 functions require AVX2+FMA to be enabled in XCR0.
// ═══════════════════════════════════════════════════════════════════════════════

/// AVX2+FMA dot product: Σ a[i]*b[i] for i in 0..len
/// 4 accumulators × 8 lanes = 32 floats per iteration.
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2,fma")]
#[inline]
// SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe fn dot_avx2(a: &[f32], b: &[f32], len: usize) -> f32 {
    let ap = a.as_ptr();
    let bp = b.as_ptr();

    let mut acc0 = _mm256_setzero_ps();
    let mut acc1 = _mm256_setzero_ps();
    let mut acc2 = _mm256_setzero_ps();
    let mut acc3 = _mm256_setzero_ps();

    // Main loop: 32 floats per iteration
    let chunks32 = len / 32;
    for i in 0..chunks32 {
        let base = i * 32;
        let a0 = _mm256_loadu_ps(ap.add(base));
        let b0 = _mm256_loadu_ps(bp.add(base));
        acc0 = _mm256_fmadd_ps(a0, b0, acc0);

        let a1 = _mm256_loadu_ps(ap.add(base + 8));
        let b1 = _mm256_loadu_ps(bp.add(base + 8));
        acc1 = _mm256_fmadd_ps(a1, b1, acc1);

        let a2 = _mm256_loadu_ps(ap.add(base + 16));
        let b2 = _mm256_loadu_ps(bp.add(base + 16));
        acc2 = _mm256_fmadd_ps(a2, b2, acc2);

        let a3 = _mm256_loadu_ps(ap.add(base + 24));
        let b3 = _mm256_loadu_ps(bp.add(base + 24));
        acc3 = _mm256_fmadd_ps(a3, b3, acc3);
    }

    // Combine 4 accumulators
    acc0 = _mm256_add_ps(acc0, acc1);
    acc2 = _mm256_add_ps(acc2, acc3);
    acc0 = _mm256_add_ps(acc0, acc2);

    // Remaining 8-element chunks
    let rem_start = chunks32 * 32;
    let rem8 = (len - rem_start) / 8;
    for i in 0..rem8 {
        let offset = rem_start + i * 8;
        let av = _mm256_loadu_ps(ap.add(offset));
        let bv = _mm256_loadu_ps(bp.add(offset));
        acc0 = _mm256_fmadd_ps(av, bv, acc0);
    }

    // AVX2 horizontal sum: 8 lanes → 1 float
    let hi128 = _mm256_extractf128_ps(acc0, 1);
    let lo128 = _mm256_castps256_ps128(acc0);
    let sum128 = _mm_add_ps(lo128, hi128);
    let hi = _mm_movehl_ps(sum128, sum128);
    let sum = _mm_add_ps(sum128, hi);
    let shuf = _mm_shuffle_ps(sum, sum, 1);
    let total = _mm_add_ss(sum, shuf);
    let mut result = _mm_cvtss_f32(total);

    // Scalar tail (len % 8)
    let scalar_start = rem_start + rem8 * 8;
    for i in scalar_start..len {
        result += *ap.add(i) * *bp.add(i);
    }

    result
}

/// AVX2+FMA matrix-vector multiply: out[r] = dot(W[r], x)
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2,fma")]
// SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe fn matvec_avx2(out: &mut [f32], w: &[f32], x: &[f32], cols: usize, rows: usize) {
    for r in 0..rows {
        let base = r * cols;
        out[r] = dot_avx2(&w[base..base + cols], x, cols);
    }
}

/// AVX2+FMA transpose matvec: out[c] = Σ_r W[r*cols+c] * y[r]
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2,fma")]
// SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe fn matvec_transpose_avx2(out: &mut [f32], w: &[f32], y: &[f32], cols: usize, rows: usize) {
    for v in out[..cols].iter_mut() { *v = 0.0; }

    let wp = w.as_ptr();
    let op = out.as_mut_ptr();

    for r in 0..rows {
        let yr = y[r];
        if yr == 0.0 { continue; }

        let yr_vec = _mm256_set1_ps(yr);
        let base = r * cols;

        // Main loop: 32 floats per iteration (4 × 8-wide FMA)
        let chunks32 = cols / 32;
        for i in 0..chunks32 {
            let offset = base + i * 32;
            let out_off = i * 32;

            let o0 = _mm256_loadu_ps(op.add(out_off));
            let w0 = _mm256_loadu_ps(wp.add(offset));
            _mm256_storeu_ps(op.add(out_off), _mm256_fmadd_ps(w0, yr_vec, o0));

            let o1 = _mm256_loadu_ps(op.add(out_off + 8));
            let w1 = _mm256_loadu_ps(wp.add(offset + 8));
            _mm256_storeu_ps(op.add(out_off + 8), _mm256_fmadd_ps(w1, yr_vec, o1));

            let o2 = _mm256_loadu_ps(op.add(out_off + 16));
            let w2 = _mm256_loadu_ps(wp.add(offset + 16));
            _mm256_storeu_ps(op.add(out_off + 16), _mm256_fmadd_ps(w2, yr_vec, o2));

            let o3 = _mm256_loadu_ps(op.add(out_off + 24));
            let w3 = _mm256_loadu_ps(wp.add(offset + 24));
            _mm256_storeu_ps(op.add(out_off + 24), _mm256_fmadd_ps(w3, yr_vec, o3));
        }

        // Remaining 8-element chunks
        let rem_start = chunks32 * 32;
        let rem8 = (cols - rem_start) / 8;
        for i in 0..rem8 {
            let offset = base + rem_start + i * 8;
            let out_off = rem_start + i * 8;
            let ov = _mm256_loadu_ps(op.add(out_off));
            let wv = _mm256_loadu_ps(wp.add(offset));
            _mm256_storeu_ps(op.add(out_off), _mm256_fmadd_ps(wv, yr_vec, ov));
        }

        // Scalar tail
        let scalar_start = rem_start + rem8 * 8;
        for c in scalar_start..cols {
            *op.add(c) += *wp.add(base + c) * yr;
        }
    }
}

/// AVX2+FMA transpose matvec with accumulation (out += W^T × y)
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2,fma")]
// SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe fn matvec_transpose_accum_avx2(out: &mut [f32], w: &[f32], y: &[f32], cols: usize, rows: usize) {
    let wp = w.as_ptr();
    let op = out.as_mut_ptr();

    for r in 0..rows {
        let yr = y[r];
        if yr == 0.0 { continue; }

        let yr_vec = _mm256_set1_ps(yr);
        let base = r * cols;

        let chunks32 = cols / 32;
        for i in 0..chunks32 {
            let offset = base + i * 32;
            let out_off = i * 32;

            let o0 = _mm256_loadu_ps(op.add(out_off));
            let w0 = _mm256_loadu_ps(wp.add(offset));
            _mm256_storeu_ps(op.add(out_off), _mm256_fmadd_ps(w0, yr_vec, o0));

            let o1 = _mm256_loadu_ps(op.add(out_off + 8));
            let w1 = _mm256_loadu_ps(wp.add(offset + 8));
            _mm256_storeu_ps(op.add(out_off + 8), _mm256_fmadd_ps(w1, yr_vec, o1));

            let o2 = _mm256_loadu_ps(op.add(out_off + 16));
            let w2 = _mm256_loadu_ps(wp.add(offset + 16));
            _mm256_storeu_ps(op.add(out_off + 16), _mm256_fmadd_ps(w2, yr_vec, o2));

            let o3 = _mm256_loadu_ps(op.add(out_off + 24));
            let w3 = _mm256_loadu_ps(wp.add(offset + 24));
            _mm256_storeu_ps(op.add(out_off + 24), _mm256_fmadd_ps(w3, yr_vec, o3));
        }

        let rem_start = chunks32 * 32;
        let rem8 = (cols - rem_start) / 8;
        for i in 0..rem8 {
            let offset = base + rem_start + i * 8;
            let out_off = rem_start + i * 8;
            let ov = _mm256_loadu_ps(op.add(out_off));
            let wv = _mm256_loadu_ps(wp.add(offset));
            _mm256_storeu_ps(op.add(out_off), _mm256_fmadd_ps(wv, yr_vec, ov));
        }

        let scalar_start = rem_start + rem8 * 8;
        for c in scalar_start..cols {
            *op.add(c) += *wp.add(base + c) * yr;
        }
    }
}

/// AVX2+FMA outer product: dW[r*cols+c] += dy[r] * x[c]
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2,fma")]
// SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe fn outer_product_accum_avx2(dw: &mut [f32], dy: &[f32], x: &[f32], cols: usize, rows: usize) {
    let dwp = dw.as_mut_ptr();
    let xp = x.as_ptr();

    for r in 0..rows {
        let dyr = dy[r];
        if dyr == 0.0 { continue; }

        let dyr_vec = _mm256_set1_ps(dyr);
        let base = r * cols;

        // Main loop: 32 floats per iteration
        let chunks32 = cols / 32;
        for i in 0..chunks32 {
            let offset = base + i * 32;
            let x_off = i * 32;

            let x0 = _mm256_loadu_ps(xp.add(x_off));
            let d0 = _mm256_loadu_ps(dwp.add(offset));
            _mm256_storeu_ps(dwp.add(offset), _mm256_fmadd_ps(dyr_vec, x0, d0));

            let x1 = _mm256_loadu_ps(xp.add(x_off + 8));
            let d1 = _mm256_loadu_ps(dwp.add(offset + 8));
            _mm256_storeu_ps(dwp.add(offset + 8), _mm256_fmadd_ps(dyr_vec, x1, d1));

            let x2 = _mm256_loadu_ps(xp.add(x_off + 16));
            let d2 = _mm256_loadu_ps(dwp.add(offset + 16));
            _mm256_storeu_ps(dwp.add(offset + 16), _mm256_fmadd_ps(dyr_vec, x2, d2));

            let x3 = _mm256_loadu_ps(xp.add(x_off + 24));
            let d3 = _mm256_loadu_ps(dwp.add(offset + 24));
            _mm256_storeu_ps(dwp.add(offset + 24), _mm256_fmadd_ps(dyr_vec, x3, d3));
        }

        // Remaining 8-element chunks
        let rem_start = chunks32 * 32;
        let rem8 = (cols - rem_start) / 8;
        for i in 0..rem8 {
            let offset = base + rem_start + i * 8;
            let x_off = rem_start + i * 8;
            let xv = _mm256_loadu_ps(xp.add(x_off));
            let dv = _mm256_loadu_ps(dwp.add(offset));
            _mm256_storeu_ps(dwp.add(offset), _mm256_fmadd_ps(dyr_vec, xv, dv));
        }

        // Scalar tail
        let scalar_start = rem_start + rem8 * 8;
        for c in scalar_start..cols {
            *dwp.add(base + c) += dyr * *xp.add(c);
        }
    }
}

/// AVX2+FMA RMSNorm: out = (x / RMS(x)) * weight
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2,fma")]
// SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe fn rmsnorm_avx2(out: &mut [f32], x: &[f32], weight: &[f32]) -> f32 {
    let n = x.len();
    let xp = x.as_ptr();

    // AVX2 sum of squares with FMA
    let mut acc0 = _mm256_setzero_ps();
    let mut acc1 = _mm256_setzero_ps();

    let chunks16 = n / 16;
    for i in 0..chunks16 {
        let base = i * 16;
        let v0 = _mm256_loadu_ps(xp.add(base));
        acc0 = _mm256_fmadd_ps(v0, v0, acc0);
        let v1 = _mm256_loadu_ps(xp.add(base + 8));
        acc1 = _mm256_fmadd_ps(v1, v1, acc1);
    }
    acc0 = _mm256_add_ps(acc0, acc1);

    // Remaining 8-element chunks
    let rem_start = chunks16 * 16;
    let rem8 = (n - rem_start) / 8;
    for i in 0..rem8 {
        let v = _mm256_loadu_ps(xp.add(rem_start + i * 8));
        acc0 = _mm256_fmadd_ps(v, v, acc0);
    }

    // Horizontal sum: 8 → 1
    let hi128 = _mm256_extractf128_ps(acc0, 1);
    let lo128 = _mm256_castps256_ps128(acc0);
    let sum128 = _mm_add_ps(lo128, hi128);
    let hi = _mm_movehl_ps(sum128, sum128);
    let sum = _mm_add_ps(sum128, hi);
    let shuf = _mm_shuffle_ps(sum, sum, 1);
    let total = _mm_add_ss(sum, shuf);
    let mut ss = _mm_cvtss_f32(total);

    // Scalar tail
    let scalar_start = rem_start + rem8 * 8;
    for i in scalar_start..n {
        ss += *xp.add(i) * *xp.add(i);
    }

    let rms = super::backprop::approx_sqrt(ss / n as f32 + super::model::RMS_EPS);
    let inv_rms = 1.0 / rms;

    // AVX2 normalize + scale
    let wp = weight.as_ptr();
    let op = out.as_mut_ptr();
    let inv_rms_vec = _mm256_set1_ps(inv_rms);

    let chunks8 = n / 8;
    for i in 0..chunks8 {
        let off = i * 8;
        let xv = _mm256_loadu_ps(xp.add(off));
        let wv = _mm256_loadu_ps(wp.add(off));
        let normed = _mm256_mul_ps(xv, inv_rms_vec);
        _mm256_storeu_ps(op.add(off), _mm256_mul_ps(normed, wv));
    }

    // Scalar tail
    let scalar_start2 = chunks8 * 8;
    for i in scalar_start2..n {
        *op.add(i) = *xp.add(i) * inv_rms * *wp.add(i);
    }

    rms
}

/// AVX2 vector add: out[i] = a[i] + b[i]
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
// SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe fn vec_add_avx2(out: &mut [f32], a: &[f32], b: &[f32], len: usize) {
    let ap = a.as_ptr();
    let bp = b.as_ptr();
    let op = out.as_mut_ptr();

    let chunks8 = len / 8;
    for i in 0..chunks8 {
        let off = i * 8;
        let av = _mm256_loadu_ps(ap.add(off));
        let bv = _mm256_loadu_ps(bp.add(off));
        _mm256_storeu_ps(op.add(off), _mm256_add_ps(av, bv));
    }

    let scalar_start = chunks8 * 8;
    for i in scalar_start..len {
        *op.add(i) = *ap.add(i) + *bp.add(i);
    }
}

/// AVX2 in-place vector add: a[i] += b[i]
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
// SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe fn vec_add_inplace_avx2(a: &mut [f32], b: &[f32], len: usize) {
    let ap = a.as_mut_ptr();
    let bp = b.as_ptr();

    let chunks8 = len / 8;
    for i in 0..chunks8 {
        let off = i * 8;
        let av = _mm256_loadu_ps(ap.add(off));
        let bv = _mm256_loadu_ps(bp.add(off));
        _mm256_storeu_ps(ap.add(off), _mm256_add_ps(av, bv));
    }

    let scalar_start = chunks8 * 8;
    for i in scalar_start..len {
        *ap.add(i) += *bp.add(i);
    }
}

/// AVX2 vector scale: a[i] *= scalar
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
// SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe fn vec_scale_avx2(a: &mut [f32], scalar: f32, len: usize) {
    let ap = a.as_mut_ptr();
    let sv = _mm256_set1_ps(scalar);

    let chunks8 = len / 8;
    for i in 0..chunks8 {
        let off = i * 8;
        let v = _mm256_loadu_ps(ap.add(off));
        _mm256_storeu_ps(ap.add(off), _mm256_mul_ps(v, sv));
    }

    let scalar_start = chunks8 * 8;
    for i in scalar_start..len {
        *ap.add(i) *= scalar;
    }
}
