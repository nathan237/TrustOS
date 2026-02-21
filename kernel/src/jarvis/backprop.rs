//! Analytical Backpropagation for the Jarvis Transformer
//!
//! Replaces numerical gradients (~6000 forward passes per step) with
//! analytical gradient computation (1 forward + 1 backward = 2 passes).
//! This is a ~3000x speedup for the same training quality.
//!
//! Architecture matches model.rs / inference.rs:
//!   4 layers, d_model=64, n_heads=4, d_ff=256, vocab=256, max_seq=256
//!
//! Gradient flow (reverse order):
//!   loss → logits → final_rmsnorm → [layer N..0: FFN → attn → rmsnorm] → embeddings

use alloc::vec::Vec;
use alloc::vec;
use super::model::*;

// ═══════════════════════════════════════════════════════════════════════════════
// Gradient Buffers — stores ∂L/∂W for every weight matrix
// ═══════════════════════════════════════════════════════════════════════════════

/// Per-layer gradients (mirrors LayerWeights)
pub struct LayerGrads {
    pub d_rms_attn: Vec<f32>,   // [D_MODEL]
    pub d_wq: Vec<f32>,         // [D_MODEL × D_MODEL]
    pub d_wk: Vec<f32>,         // [D_MODEL × D_MODEL]
    pub d_wv: Vec<f32>,         // [D_MODEL × D_MODEL]
    pub d_wo: Vec<f32>,         // [D_MODEL × D_MODEL]
    pub d_rms_ffn: Vec<f32>,    // [D_MODEL]
    pub d_wgate: Vec<f32>,      // [D_MODEL × D_FF]
    pub d_wup: Vec<f32>,        // [D_MODEL × D_FF]
    pub d_wdown: Vec<f32>,      // [D_FF × D_MODEL]
}

impl LayerGrads {
    pub fn new() -> Self {
        LayerGrads {
            d_rms_attn: vec![0.0; D_MODEL],
            d_wq: vec![0.0; D_MODEL * D_MODEL],
            d_wk: vec![0.0; D_MODEL * D_MODEL],
            d_wv: vec![0.0; D_MODEL * D_MODEL],
            d_wo: vec![0.0; D_MODEL * D_MODEL],
            d_rms_ffn: vec![0.0; D_MODEL],
            d_wgate: vec![0.0; D_MODEL * D_FF],
            d_wup: vec![0.0; D_MODEL * D_FF],
            d_wdown: vec![0.0; D_FF * D_MODEL],
        }
    }

    pub fn zero(&mut self) {
        for v in self.d_rms_attn.iter_mut() { *v = 0.0; }
        for v in self.d_wq.iter_mut() { *v = 0.0; }
        for v in self.d_wk.iter_mut() { *v = 0.0; }
        for v in self.d_wv.iter_mut() { *v = 0.0; }
        for v in self.d_wo.iter_mut() { *v = 0.0; }
        for v in self.d_rms_ffn.iter_mut() { *v = 0.0; }
        for v in self.d_wgate.iter_mut() { *v = 0.0; }
        for v in self.d_wup.iter_mut() { *v = 0.0; }
        for v in self.d_wdown.iter_mut() { *v = 0.0; }
    }
}

/// All gradients for the full model
pub struct ModelGrads {
    pub d_token_embed: Vec<f32>,  // [VOCAB_SIZE × D_MODEL]
    pub d_pos_embed: Vec<f32>,    // [MAX_SEQ × D_MODEL]
    pub layers: Vec<LayerGrads>,
    pub d_rms_final: Vec<f32>,    // [D_MODEL]
    pub d_output: Vec<f32>,       // [D_MODEL × VOCAB_SIZE]
}

impl ModelGrads {
    pub fn new() -> Self {
        let mut layers = Vec::with_capacity(N_LAYERS);
        for _ in 0..N_LAYERS {
            layers.push(LayerGrads::new());
        }
        ModelGrads {
            d_token_embed: vec![0.0; VOCAB_SIZE * D_MODEL],
            d_pos_embed: vec![0.0; MAX_SEQ * D_MODEL],
            layers,
            d_rms_final: vec![0.0; D_MODEL],
            d_output: vec![0.0; D_MODEL * VOCAB_SIZE],
        }
    }

    pub fn zero(&mut self) {
        for v in self.d_token_embed.iter_mut() { *v = 0.0; }
        for v in self.d_pos_embed.iter_mut() { *v = 0.0; }
        for l in &mut self.layers { l.zero(); }
        for v in self.d_rms_final.iter_mut() { *v = 0.0; }
        for v in self.d_output.iter_mut() { *v = 0.0; }
    }

    /// Total number of gradient values
    pub fn count(&self) -> usize {
        self.d_token_embed.len() + self.d_pos_embed.len()
        + self.layers.iter().map(|l| {
            l.d_rms_attn.len() + l.d_wq.len() + l.d_wk.len() + l.d_wv.len()
            + l.d_wo.len() + l.d_rms_ffn.len() + l.d_wgate.len() + l.d_wup.len()
            + l.d_wdown.len()
        }).sum::<usize>()
        + self.d_rms_final.len() + self.d_output.len()
    }

    /// Compute total L2 norm of all gradients
    pub fn grad_norm(&self) -> f32 {
        let mut ss = 0.0f32;
        let add_ss = |ss: &mut f32, s: &[f32]| { for &g in s { *ss += g * g; } };
        add_ss(&mut ss, &self.d_token_embed);
        add_ss(&mut ss, &self.d_pos_embed);
        for l in &self.layers {
            add_ss(&mut ss, &l.d_rms_attn);
            add_ss(&mut ss, &l.d_wq); add_ss(&mut ss, &l.d_wk);
            add_ss(&mut ss, &l.d_wv); add_ss(&mut ss, &l.d_wo);
            add_ss(&mut ss, &l.d_rms_ffn);
            add_ss(&mut ss, &l.d_wgate); add_ss(&mut ss, &l.d_wup);
            add_ss(&mut ss, &l.d_wdown);
        }
        add_ss(&mut ss, &self.d_rms_final);
        add_ss(&mut ss, &self.d_output);
        approx_sqrt(ss)
    }

    /// Clip gradients by global L2 norm (if norm > max_norm, scale down)
    pub fn clip_norm(&mut self, max_norm: f32) {
        let norm = self.grad_norm();
        if norm > max_norm && norm > 0.0 {
            let s = max_norm / norm;
            let sc = |v: &mut [f32], s: f32| { for g in v.iter_mut() { *g *= s; } };
            sc(&mut self.d_token_embed, s);
            sc(&mut self.d_pos_embed, s);
            for l in &mut self.layers {
                sc(&mut l.d_rms_attn, s); sc(&mut l.d_wq, s); sc(&mut l.d_wk, s);
                sc(&mut l.d_wv, s); sc(&mut l.d_wo, s); sc(&mut l.d_rms_ffn, s);
                sc(&mut l.d_wgate, s); sc(&mut l.d_wup, s); sc(&mut l.d_wdown, s);
            }
            sc(&mut self.d_rms_final, s);
            sc(&mut self.d_output, s);
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Forward Pass with Saved Activations (needed for backward)
// ═══════════════════════════════════════════════════════════════════════════════

/// Activations saved during forward pass for one token position
struct PosActivations {
    /// Input to this position (after embed) [D_MODEL]
    x: Vec<f32>,
    /// Per-layer saved values
    layer_acts: Vec<LayerActivations>,
    /// After final rmsnorm [D_MODEL]
    x_final_norm: Vec<f32>,
    /// Logits [VOCAB_SIZE]
    logits: Vec<f32>,
}

struct LayerActivations {
    // Pre-attention
    x_in: Vec<f32>,       // [D_MODEL] residual input
    x_norm_attn: Vec<f32>,// [D_MODEL] after rmsnorm
    q: Vec<f32>,          // [D_MODEL]
    k: Vec<f32>,          // [D_MODEL]
    v: Vec<f32>,          // [D_MODEL]
    attn_weights: Vec<Vec<f32>>,  // [N_HEADS][n_pos] softmax outputs
    attn_out: Vec<f32>,   // [D_MODEL] weighted sum of V
    proj_out: Vec<f32>,   // [D_MODEL] after W_o
    // Pre-FFN
    x_mid: Vec<f32>,      // [D_MODEL] after attn residual
    x_norm_ffn: Vec<f32>, // [D_MODEL] after FFN rmsnorm
    gate_pre: Vec<f32>,   // [D_FF] before activation
    gate_act: Vec<f32>,   // [D_FF] SiLU(gate)
    up: Vec<f32>,         // [D_FF]
    gated: Vec<f32>,      // [D_FF] gate_act * up
    ffn_out: Vec<f32>,    // [D_MODEL] after W_down
}

// ═══════════════════════════════════════════════════════════════════════════════
// Math helpers (same as inference.rs but standalone)
// ═══════════════════════════════════════════════════════════════════════════════

fn matvec(out: &mut [f32], w: &[f32], x: &[f32], cols: usize, rows: usize) {
    for r in 0..rows {
        let mut sum = 0.0f32;
        let base = r * cols;
        for c in 0..cols {
            sum += w[base + c] * x[c];
        }
        out[r] = sum;
    }
}

fn rmsnorm(out: &mut [f32], x: &[f32], weight: &[f32]) -> f32 {
    let n = x.len();
    let mut ss = 0.0f32;
    for &v in x { ss += v * v; }
    let rms = approx_sqrt(ss / n as f32 + RMS_EPS);
    let inv = 1.0 / rms;
    for i in 0..n {
        out[i] = x[i] * inv * weight[i];
    }
    inv // return inv_rms for backward
}

fn softmax(data: &mut [f32]) {
    if data.is_empty() { return; }
    let max = data.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let mut sum = 0.0f32;
    for v in data.iter_mut() {
        *v = approx_exp(*v - max);
        sum += *v;
    }
    if sum > 0.0 {
        let inv = 1.0 / sum;
        for v in data.iter_mut() { *v *= inv; }
    }
}

fn approx_exp(x: f32) -> f32 {
    if x > 88.0 { return f32::MAX; }
    if x < -88.0 { return 0.0; }
    let a = (1 << 23) as f32 / core::f32::consts::LN_2;
    let b = (1 << 23) as f32 * (127.0 - 0.04368);
    let bits = ((a * x + b) as i32).max(0) as u32;
    f32::from_bits(bits)
}

fn approx_sqrt(x: f32) -> f32 {
    if x <= 0.0 { return 0.0; }
    let bits = x.to_bits();
    let guess = f32::from_bits((bits >> 1) + 0x1FBD_1DF5);
    (guess + x / guess) * 0.5
}

fn silu(x: f32) -> f32 {
    let sig = 1.0 / (1.0 + approx_exp(-x));
    x * sig
}

fn silu_grad(x: f32) -> f32 {
    let sig = 1.0 / (1.0 + approx_exp(-x));
    sig + x * sig * (1.0 - sig)
}

// ═══════════════════════════════════════════════════════════════════════════════
// Forward + Backward
// ═══════════════════════════════════════════════════════════════════════════════

/// Compute loss AND gradients in a single forward+backward pass.
/// Returns (avg_loss, grads).
///
/// This replaces ~6000 forward passes from numerical gradients with exactly
/// 1 forward + 1 backward pass.
pub fn forward_backward(model: &TransformerWeights, tokens: &[u8]) -> (f32, ModelGrads) {
    let seq_len = tokens.len().min(super::model::MAX_SEQ);
    if seq_len < 2 {
        return (f32::MAX, ModelGrads::new());
    }

    // ── FORWARD PASS (save activations) ─────────────────────────────────
    let mut all_acts: Vec<PosActivations> = Vec::with_capacity(seq_len);
    // KV cache: per-layer, all positions' K and V
    let mut all_k: Vec<Vec<Vec<f32>>> = vec![Vec::new(); N_LAYERS]; // [layer][pos][D_MODEL]
    let mut all_v: Vec<Vec<Vec<f32>>> = vec![Vec::new(); N_LAYERS];

    for t in 0..seq_len {
        let tok = tokens[t] as usize;
        let pos = t;

        // Embedding
        let mut x = vec![0.0f32; D_MODEL];
        for i in 0..D_MODEL {
            x[i] = model.token_embed[tok * D_MODEL + i] + model.pos_embed[pos * D_MODEL + i];
        }

        let mut layer_acts_vec = Vec::with_capacity(N_LAYERS);

        for l in 0..N_LAYERS {
            let layer = &model.layers[l];
            let x_in = x.clone();

            // RMSNorm (attn)
            let mut x_norm = vec![0.0f32; D_MODEL];
            let inv_rms_attn = rmsnorm(&mut x_norm, &x_in, &layer.rms_attn);
            let _ = inv_rms_attn; // used in backward

            // QKV
            let mut q = vec![0.0f32; D_MODEL];
            let mut k = vec![0.0f32; D_MODEL];
            let mut v = vec![0.0f32; D_MODEL];
            matvec(&mut q, &layer.w_q, &x_norm, D_MODEL, D_MODEL);
            matvec(&mut k, &layer.w_k, &x_norm, D_MODEL, D_MODEL);
            matvec(&mut v, &layer.w_v, &x_norm, D_MODEL, D_MODEL);

            // Store K, V
            all_k[l].push(k.clone());
            all_v[l].push(v.clone());

            // Multi-head attention
            let n_pos = t + 1;
            let mut attn_out = vec![0.0f32; D_MODEL];
            let d_k_sqrt = approx_sqrt(D_K as f32);
            let mut attn_weights_all_heads = Vec::with_capacity(N_HEADS);

            for h in 0..N_HEADS {
                let ho = h * D_K;
                let mut scores = vec![0.0f32; n_pos];
                for p in 0..n_pos {
                    let mut s = 0.0f32;
                    for d in 0..D_K {
                        s += q[ho + d] * all_k[l][p][ho + d];
                    }
                    scores[p] = s / d_k_sqrt;
                }
                softmax(&mut scores);

                for p in 0..n_pos {
                    let w = scores[p];
                    for d in 0..D_K {
                        attn_out[ho + d] += w * all_v[l][p][ho + d];
                    }
                }
                attn_weights_all_heads.push(scores);
            }

            // Output projection
            let mut proj = vec![0.0f32; D_MODEL];
            matvec(&mut proj, &layer.w_o, &attn_out, D_MODEL, D_MODEL);

            // Residual
            for i in 0..D_MODEL { x[i] = x_in[i] + proj[i]; }
            let x_mid = x.clone();

            // RMSNorm (FFN)
            let mut x_norm_ffn = vec![0.0f32; D_MODEL];
            let _inv_rms_ffn = rmsnorm(&mut x_norm_ffn, &x_mid, &layer.rms_ffn);

            // SwiGLU FFN
            let mut gate_pre = vec![0.0f32; D_FF];
            let mut up = vec![0.0f32; D_FF];
            matvec(&mut gate_pre, &layer.w_gate, &x_norm_ffn, D_MODEL, D_FF);
            matvec(&mut up, &layer.w_up, &x_norm_ffn, D_MODEL, D_FF);

            let mut gate_act = vec![0.0f32; D_FF];
            let mut gated = vec![0.0f32; D_FF];
            for i in 0..D_FF {
                gate_act[i] = silu(gate_pre[i]);
                gated[i] = gate_act[i] * up[i];
            }

            let mut ffn_out = vec![0.0f32; D_MODEL];
            matvec(&mut ffn_out, &layer.w_down, &gated, D_FF, D_MODEL);

            // Residual
            for i in 0..D_MODEL { x[i] = x_mid[i] + ffn_out[i]; }

            layer_acts_vec.push(LayerActivations {
                x_in, x_norm_attn: x_norm, q, k: all_k[l][t].clone(), v: all_v[l][t].clone(),
                attn_weights: attn_weights_all_heads, attn_out, proj_out: proj,
                x_mid, x_norm_ffn, gate_pre, gate_act, up, gated, ffn_out,
            });
        }

        // Final RMSNorm
        let mut x_final = vec![0.0f32; D_MODEL];
        rmsnorm(&mut x_final, &x, &model.rms_final);

        // Logits
        let mut logits = vec![0.0f32; VOCAB_SIZE];
        matvec(&mut logits, &model.w_output, &x_final, D_MODEL, VOCAB_SIZE);

        all_acts.push(PosActivations {
            x: x.clone(),
            layer_acts: layer_acts_vec,
            x_final_norm: x_final,
            logits,
        });
    }

    // ── COMPUTE LOSS + dL/dlogits ───────────────────────────────────────
    let mut total_loss = 0.0f32;
    let n_targets = seq_len - 1;
    let mut grads = ModelGrads::new();

    // For each position t, target is tokens[t+1]
    // We backprop from position seq_len-2 down to 0
    // (position seq_len-1 has no target)

    // We'll accumulate dL/dx for each position, then backprop through layers
    for t in 0..n_targets {
        let target = tokens[t + 1] as usize;
        let acts = &all_acts[t];

        // ── Softmax cross-entropy gradient ──
        // dL/dlogits = softmax(logits) - one_hot(target)
        let mut probs = acts.logits.clone();
        softmax(&mut probs);

        // Loss = -log(prob[target])
        let p_target = probs[target].max(1e-10);
        total_loss += -ln_approx(p_target);

        let mut d_logits = probs; // softmax output
        d_logits[target] -= 1.0;  // subtract one-hot
        // Scale by 1/n_targets for average loss
        let scale = 1.0 / n_targets as f32;
        for v in d_logits.iter_mut() { *v *= scale; }

        // ── dL/d_w_output ──
        // w_output is [VOCAB_SIZE rows × D_MODEL cols]: logits[r] = Σ_c w[r*D_MODEL+c]*x[c]
        for r in 0..VOCAB_SIZE {
            let base = r * D_MODEL;
            for c in 0..D_MODEL {
                grads.d_output[base + c] += d_logits[r] * acts.x_final_norm[c];
            }
        }

        // ── dL/d_x_final_norm = W_output^T @ d_logits ──
        let mut d_xfn = vec![0.0f32; D_MODEL];
        for c in 0..D_MODEL {
            let mut s = 0.0f32;
            for r in 0..VOCAB_SIZE {
                s += model.w_output[r * D_MODEL + c] * d_logits[r];
            }
            d_xfn[c] = s;
        }

        // ── Backward through final RMSNorm ──
        let mut d_x = backward_rmsnorm(&d_xfn, &acts.x, &model.rms_final, &mut grads.d_rms_final);

        // ── Backward through layers (reverse order) ──
        for l in (0..N_LAYERS).rev() {
            let la = &acts.layer_acts[l];
            let layer = &model.layers[l];
            let lg = &mut grads.layers[l];

            // ── Backward through FFN residual: d_x splits to d_x_mid and d_ffn_out ──
            let d_ffn_out = d_x.clone(); // gradient flows to both branches
            // d_x_mid gets same d_x (residual connection)

            // ── Backward through W_down ──
            // ffn_out[r] = Σ_c w_down[r*D_FF+c] * gated[c], r=0..D_MODEL
            let mut d_gated = vec![0.0f32; D_FF];
            for r in 0..D_MODEL {
                let base = r * D_FF;
                for c in 0..D_FF {
                    lg.d_wdown[base + c] += d_ffn_out[r] * la.gated[c];
                }
            }
            for c in 0..D_FF {
                let mut s = 0.0f32;
                for r in 0..D_MODEL {
                    s += layer.w_down[r * D_FF + c] * d_ffn_out[r];
                }
                d_gated[c] = s;
            }

            // ── Backward through SwiGLU: gated = silu(gate_pre) * up ──
            let mut d_gate_pre = vec![0.0f32; D_FF];
            let mut d_up = vec![0.0f32; D_FF];
            for i in 0..D_FF {
                // d_gate_act = d_gated * up
                let d_gate_act = d_gated[i] * la.up[i];
                // d_up = d_gated * gate_act
                d_up[i] = d_gated[i] * la.gate_act[i];
                // d_gate_pre = d_gate_act * silu'(gate_pre)
                d_gate_pre[i] = d_gate_act * silu_grad(la.gate_pre[i]);
            }

            // ── Backward through W_gate and W_up ──
            // gate_pre[r] = Σ_c w_gate[r*D_MODEL+c] * x[c], r=0..D_FF (same for w_up)
            let mut d_xnf = vec![0.0f32; D_MODEL];
            for r in 0..D_FF {
                let base = r * D_MODEL;
                for c in 0..D_MODEL {
                    lg.d_wgate[base + c] += d_gate_pre[r] * la.x_norm_ffn[c];
                    lg.d_wup[base + c] += d_up[r] * la.x_norm_ffn[c];
                    d_xnf[c] += layer.w_gate[base + c] * d_gate_pre[r];
                    d_xnf[c] += layer.w_up[base + c] * d_up[r];
                }
            }

            // ── Backward through FFN RMSNorm ──
            let d_x_mid = backward_rmsnorm(&d_xnf, &la.x_mid, &layer.rms_ffn, &mut lg.d_rms_ffn);

            // Add residual gradient
            let mut d_x_pre_ffn = vec![0.0f32; D_MODEL];
            for i in 0..D_MODEL {
                d_x_pre_ffn[i] = d_x[i] + d_x_mid[i]; // residual: both paths
            }

            // ── Backward through W_o: proj[r] = Σ_c W_o[r*D+c] * attn_out[c] ──
            for r in 0..D_MODEL {
                let base = r * D_MODEL;
                for c in 0..D_MODEL {
                    lg.d_wo[base + c] += d_x_pre_ffn[r] * la.attn_out[c];
                }
            }
            let mut d_attn_out = vec![0.0f32; D_MODEL];
            for c in 0..D_MODEL {
                let mut s = 0.0f32;
                for r in 0..D_MODEL {
                    s += layer.w_o[r * D_MODEL + c] * d_x_pre_ffn[r];
                }
                d_attn_out[c] = s;
            }

            // ── Backward through multi-head attention (Q + self-position K,V) ──
            let d_k_sqrt = approx_sqrt(D_K as f32);
            let n_pos = t + 1;
            let mut d_q = vec![0.0f32; D_MODEL];
            let mut d_k_self = vec![0.0f32; D_MODEL];
            let mut d_v_self = vec![0.0f32; D_MODEL];
            for h in 0..N_HEADS {
                let ho = h * D_K;
                let wts = &la.attn_weights[h];

                let mut d_wts = vec![0.0f32; n_pos];
                for p in 0..n_pos {
                    let mut s = 0.0f32;
                    for d in 0..D_K {
                        s += d_attn_out[ho + d] * all_v[l][p][ho + d];
                        if p == t { d_v_self[ho + d] += wts[p] * d_attn_out[ho + d]; }
                    }
                    d_wts[p] = s;
                }

                let dot: f32 = (0..n_pos).map(|p| d_wts[p] * wts[p]).sum();
                let mut d_scores = vec![0.0f32; n_pos];
                for p in 0..n_pos {
                    d_scores[p] = wts[p] * (d_wts[p] - dot);
                }

                for p in 0..n_pos {
                    let ds = d_scores[p] / d_k_sqrt;
                    for d in 0..D_K {
                        d_q[ho + d] += ds * all_k[l][p][ho + d];
                        if p == t { d_k_self[ho + d] += ds * la.q[ho + d]; }
                    }
                }
            }

            // ── Backward through Q, K, V projections ──
            let mut d_xna = vec![0.0f32; D_MODEL];
            for c in 0..D_MODEL {
                let mut s = 0.0f32;
                for r in 0..D_MODEL {
                    let idx = r * D_MODEL + c;
                    lg.d_wq[idx] += d_q[r] * la.x_norm_attn[c];
                    s += layer.w_q[idx] * d_q[r];
                    lg.d_wk[idx] += d_k_self[r] * la.x_norm_attn[c];
                    s += layer.w_k[idx] * d_k_self[r];
                    lg.d_wv[idx] += d_v_self[r] * la.x_norm_attn[c];
                    s += layer.w_v[idx] * d_v_self[r];
                }
                d_xna[c] = s;
            }

            // ── Backward through attention RMSNorm ──
            let d_x_in = backward_rmsnorm(&d_xna, &la.x_in, &layer.rms_attn, &mut lg.d_rms_attn);

            // Residual: d_x for next (lower) layer
            for i in 0..D_MODEL {
                d_x[i] = d_x_pre_ffn[i] + d_x_in[i];
            }
        }

        // ── Embedding gradients ──
        let tok = tokens[t] as usize;
        for i in 0..D_MODEL {
            grads.d_token_embed[tok * D_MODEL + i] += d_x[i];
            grads.d_pos_embed[t * D_MODEL + i] += d_x[i];
        }
    }

    let avg_loss = total_loss / n_targets as f32;
    (avg_loss, grads)
}

// ═══════════════════════════════════════════════════════════════════════════════
// RMSNorm backward
// ═══════════════════════════════════════════════════════════════════════════════

/// Backward through RMSNorm: out = (x / rms) * weight
/// Given d_out, returns d_x and accumulates into d_weight
fn backward_rmsnorm(d_out: &[f32], x: &[f32], weight: &[f32], d_weight: &mut [f32]) -> Vec<f32> {
    let n = x.len();
    let mut ss = 0.0f32;
    for &v in x { ss += v * v; }
    let rms = approx_sqrt(ss / n as f32 + RMS_EPS);
    let inv_rms = 1.0 / rms;

    // d_weight[i] += d_out[i] * x[i] * inv_rms
    for i in 0..n {
        d_weight[i] += d_out[i] * x[i] * inv_rms;
    }

    // d_x: chain rule through normalization
    // norm[i] = x[i] * inv_rms
    // d_norm[i] = d_out[i] * weight[i]
    let mut d_norm = vec![0.0f32; n];
    for i in 0..n {
        d_norm[i] = d_out[i] * weight[i];
    }

    // d_x[i] = inv_rms * (d_norm[i] - norm[i] * mean(norm * d_norm))
    let mut dot = 0.0f32;
    for i in 0..n {
        dot += x[i] * inv_rms * d_norm[i];
    }
    dot /= n as f32;

    let mut d_x = vec![0.0f32; n];
    for i in 0..n {
        d_x[i] = inv_rms * (d_norm[i] - x[i] * inv_rms * dot);
    }
    d_x
}

// ═══════════════════════════════════════════════════════════════════════════════
// Approximate ln (for loss computation)
// ═══════════════════════════════════════════════════════════════════════════════

fn ln_approx(x: f32) -> f32 {
    if x <= 0.0 { return -88.0; }
    let bits = x.to_bits();
    let e = ((bits >> 23) & 0xFF) as f32 - 127.0;
    let m = f32::from_bits((bits & 0x007FFFFF) | 0x3F800000);
    (e + (m - 1.0) * 1.4427) * core::f32::consts::LN_2
}
