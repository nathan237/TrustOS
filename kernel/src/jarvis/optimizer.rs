//! Adam Optimizer for Jarvis Neural Brain
//!
//! Adam (Adaptive Moment Estimation) converges 5-10x faster than plain SGD.
//! Maintains per-parameter momentum (m) and squared gradient (v) buffers.
//!
//! Update rule:
//!   m = β1 * m + (1 - β1) * grad
//!   v = β2 * v + (1 - β2) * grad²
//!   m_hat = m / (1 - β1^t)
//!   v_hat = v / (1 - β2^t)
//!   w -= lr * m_hat / (sqrt(v_hat) + ε)

use alloc::vec::Vec;
use alloc::vec;
use super::model::*;
use super::backprop::ModelGrads;

/// Adam optimizer state
pub struct AdamState {
    /// First moment (momentum) — same layout as model weights
    pub m: Vec<f32>,
    /// Second moment (squared grads)
    pub v: Vec<f32>,
    /// Timestep (for bias correction)
    pub t: u64,
    /// Learning rate
    pub lr: f32,
    /// β1 (momentum decay)
    pub beta1: f32,
    /// β2 (squared grad decay)
    pub beta2: f32,
    /// Epsilon (numerical stability)
    pub eps: f32,
    /// Gradient clipping threshold
    pub grad_clip: f32,
}

impl AdamState {
    /// Create new Adam optimizer with default hyperparams
    pub fn new(param_count: usize) -> Self {
        AdamState {
            m: vec![0.0; param_count],
            v: vec![0.0; param_count],
            t: 0,
            lr: 0.001,
            beta1: 0.9,
            beta2: 0.999,
            eps: 1e-8,
            grad_clip: 1.0,
        }
    }

    /// Create with custom learning rate
    pub fn with_lr(param_count: usize, lr: f32) -> Self {
        let mut s = Self::new(param_count);
        s.lr = lr;
        s
    }

    /// Apply one Adam step: updates model weights using gradients
    pub fn step(&mut self, model: &mut TransformerWeights, grads: &ModelGrads) {
        self.t += 1;

        // Bias correction factors
        let bc1 = 1.0 - pow_approx(self.beta1, self.t);
        let bc2 = 1.0 - pow_approx(self.beta2, self.t);
        let lr_t = self.lr / bc1; // effective LR with bias correction

        let mut idx = 0;

        // Token embeddings
        self.update_slice(&mut model.token_embed, &grads.d_token_embed, &mut idx, lr_t, bc2);

        // Position embeddings
        self.update_slice(&mut model.pos_embed, &grads.d_pos_embed, &mut idx, lr_t, bc2);

        // Layer weights
        for l in 0..N_LAYERS {
            let lg = &grads.layers[l];
            let lw = &mut model.layers[l];

            self.update_slice(&mut lw.rms_attn, &lg.d_rms_attn, &mut idx, lr_t, bc2);
            self.update_slice(&mut lw.w_q, &lg.d_wq, &mut idx, lr_t, bc2);
            self.update_slice(&mut lw.w_k, &lg.d_wk, &mut idx, lr_t, bc2);
            self.update_slice(&mut lw.w_v, &lg.d_wv, &mut idx, lr_t, bc2);
            self.update_slice(&mut lw.w_o, &lg.d_wo, &mut idx, lr_t, bc2);
            self.update_slice(&mut lw.rms_ffn, &lg.d_rms_ffn, &mut idx, lr_t, bc2);
            self.update_slice(&mut lw.w_gate, &lg.d_wgate, &mut idx, lr_t, bc2);
            self.update_slice(&mut lw.w_up, &lg.d_wup, &mut idx, lr_t, bc2);
            self.update_slice(&mut lw.w_down, &lg.d_wdown, &mut idx, lr_t, bc2);
        }

        // Final RMSNorm
        self.update_slice(&mut model.rms_final, &grads.d_rms_final, &mut idx, lr_t, bc2);

        // Output projection
        self.update_slice(&mut model.w_output, &grads.d_output, &mut idx, lr_t, bc2);
    }

    /// Update a weight slice using Adam
    fn update_slice(&mut self, weights: &mut [f32], grads: &[f32], idx: &mut usize, lr_t: f32, bc2: f32) {
        for i in 0..weights.len() {
            let j = *idx + i;
            if j >= self.m.len() { break; }

            let mut g = grads[i];

            // Gradient clipping
            if g > self.grad_clip { g = self.grad_clip; }
            if g < -self.grad_clip { g = -self.grad_clip; }

            // Update moments
            self.m[j] = self.beta1 * self.m[j] + (1.0 - self.beta1) * g;
            self.v[j] = self.beta2 * self.v[j] + (1.0 - self.beta2) * g * g;

            // Bias-corrected second moment
            let v_hat = self.v[j] / bc2;

            // Update weight
            weights[i] -= lr_t * self.m[j] / (approx_sqrt(v_hat) + self.eps);
        }
        *idx += weights.len();
    }

    /// Reset optimizer state (keep hyperparams)
    pub fn reset(&mut self) {
        for v in self.m.iter_mut() { *v = 0.0; }
        for v in self.v.iter_mut() { *v = 0.0; }
        self.t = 0;
    }

    /// Memory usage in bytes
    pub fn memory_bytes(&self) -> usize {
        (self.m.len() + self.v.len()) * 4
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Helpers
// ═══════════════════════════════════════════════════════════════════════════════

fn approx_sqrt(x: f32) -> f32 {
    if x <= 0.0 { return 0.0; }
    let bits = x.to_bits();
    let guess = f32::from_bits((bits >> 1) + 0x1FBD_1DF5);
    (guess + x / guess) * 0.5
}

/// Approximate power: base^exp for small positive values
fn pow_approx(base: f32, exp: u64) -> f32 {
    let mut result = 1.0f32;
    let mut b = base;
    let mut e = exp;
    while e > 0 {
        if e & 1 == 1 { result *= b; }
        b *= b;
        e >>= 1;
    }
    result
}
