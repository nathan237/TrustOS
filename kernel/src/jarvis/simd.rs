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
// MATRIX-VECTOR MULTIPLY (SSE2)
// ═══════════════════════════════════════════════════════════════════════════════

/// SSE2-accelerated matrix-vector multiply: out[r] = Σ_c W[r*cols+c] * x[c]
///
/// W is [rows × cols] row-major, x is [cols], out is [rows].
/// Each row's dot product uses 4 SSE2 accumulators for ILP.
///
/// This is the hottest function in training — called ~18 times per token
/// per forward+backward step (Q/K/V/O + gate/up/down per layer + output).
#[cfg(target_arch = "x86_64")]
pub fn matvec(out: &mut [f32], w: &[f32], x: &[f32], cols: usize, rows: usize) {
    for r in 0..rows {
        let base = r * cols;
        out[r] = dot_simd(&w[base..base + cols], x, cols);
    }
}

/// Scalar fallback for non-x86_64 (should never be used in practice)
#[cfg(not(target_arch = "x86_64"))]
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
pub fn matvec_transpose(out: &mut [f32], w: &[f32], y: &[f32], cols: usize, rows: usize) {
    // Zero output first
    for v in out[..cols].iter_mut() { *v = 0.0; }

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

/// Scalar fallback for non-x86_64
#[cfg(not(target_arch = "x86_64"))]
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
pub fn matvec_transpose_accum(out: &mut [f32], w: &[f32], y: &[f32], cols: usize, rows: usize) {
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

#[cfg(not(target_arch = "x86_64"))]
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
pub fn outer_product_accum(dw: &mut [f32], dy: &[f32], x: &[f32], cols: usize, rows: usize) {
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

/// Scalar fallback for non-x86_64
#[cfg(not(target_arch = "x86_64"))]
pub fn outer_product_accum(dw: &mut [f32], dy: &[f32], x: &[f32], cols: usize, rows: usize) {
    for r in 0..rows {
        let base = r * cols;
        for c in 0..cols {
            dw[base + c] += dy[r] * x[c];
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// RMSNORM (SSE2)
// ═══════════════════════════════════════════════════════════════════════════════

/// SSE2-accelerated RMSNorm: out = (x / RMS(x)) * weight
///
/// Returns the RMS value for use in the backward pass.
/// Accelerates both the sum-of-squares and the normalize-multiply loop.
#[cfg(target_arch = "x86_64")]
pub fn rmsnorm(out: &mut [f32], x: &[f32], weight: &[f32]) -> f32 {
    let n = x.len();

    // SSE2 sum of squares
    let ss = unsafe {
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

/// Scalar fallback
#[cfg(not(target_arch = "x86_64"))]
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
// VECTOR OPERATIONS (SSE2)
// ═══════════════════════════════════════════════════════════════════════════════

/// SSE2-accelerated vector add: out[i] = a[i] + b[i]
#[cfg(target_arch = "x86_64")]
pub fn vec_add(out: &mut [f32], a: &[f32], b: &[f32], len: usize) {
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

#[cfg(not(target_arch = "x86_64"))]
pub fn vec_add(out: &mut [f32], a: &[f32], b: &[f32], len: usize) {
    for i in 0..len { out[i] = a[i] + b[i]; }
}

/// SSE2-accelerated in-place vector add: a[i] += b[i]
#[cfg(target_arch = "x86_64")]
pub fn vec_add_inplace(a: &mut [f32], b: &[f32], len: usize) {
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

#[cfg(not(target_arch = "x86_64"))]
pub fn vec_add_inplace(a: &mut [f32], b: &[f32], len: usize) {
    for i in 0..len { a[i] += b[i]; }
}

/// SSE2-accelerated vector scale: out[i] = a[i] * scalar
#[cfg(target_arch = "x86_64")]
pub fn vec_scale(a: &mut [f32], scalar: f32, len: usize) {
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

#[cfg(not(target_arch = "x86_64"))]
pub fn vec_scale(a: &mut [f32], scalar: f32, len: usize) {
    for i in 0..len { a[i] *= scalar; }
}
