// JARVIS Native Maturity Test
// Runs the exact same 4.4M-param transformer training natively on your CPU
// instead of emulated in QEMU/TCG — expected 100-1000x speedup.
//
// This reproduces: kernel/src/jarvis/{model,backprop,optimizer,corpus}.rs

use std::time::Instant;

// ═══════════════════════════════════════════════════════════════
// Model Hyperparameters (same as kernel/src/jarvis/model.rs)
// ═══════════════════════════════════════════════════════════════
const VOCAB_SIZE: usize = 256;
const D_MODEL: usize = 256;
const N_HEADS: usize = 4;
const D_K: usize = D_MODEL / N_HEADS;
const D_FF: usize = 1024;
const N_LAYERS: usize = 4;
const MAX_SEQ: usize = 256;
const RMS_EPS: f32 = 1e-5;

// ═══════════════════════════════════════════════════════════════
// Weight Structures
// ═══════════════════════════════════════════════════════════════
struct LayerWeights {
    rms_attn: Vec<f32>,
    w_q: Vec<f32>, w_k: Vec<f32>, w_v: Vec<f32>, w_o: Vec<f32>,
    rms_ffn: Vec<f32>,
    w_gate: Vec<f32>, w_up: Vec<f32>, w_down: Vec<f32>,
}

struct TransformerWeights {
    token_embed: Vec<f32>,
    pos_embed: Vec<f32>,
    layers: Vec<LayerWeights>,
    rms_final: Vec<f32>,
    w_output: Vec<f32>,
}

impl TransformerWeights {
    fn new_random() -> Self {
        let mut seed = 42u64;
        let embed_scale = 1.0 / (D_MODEL as f32).sqrt();
        let attn_scale = embed_scale;
        let ffn_scale = 1.0 / (D_FF as f32).sqrt();

        let mut layers = Vec::with_capacity(N_LAYERS);
        for _ in 0..N_LAYERS {
            layers.push(LayerWeights {
                rms_attn: vec![1.0; D_MODEL],
                w_q: random_vec(D_MODEL * D_MODEL, attn_scale, &mut seed),
                w_k: random_vec(D_MODEL * D_MODEL, attn_scale, &mut seed),
                w_v: random_vec(D_MODEL * D_MODEL, attn_scale, &mut seed),
                w_o: random_vec(D_MODEL * D_MODEL, attn_scale, &mut seed),
                rms_ffn: vec![1.0; D_MODEL],
                w_gate: random_vec(D_MODEL * D_FF, ffn_scale, &mut seed),
                w_up: random_vec(D_MODEL * D_FF, ffn_scale, &mut seed),
                w_down: random_vec(D_FF * D_MODEL, ffn_scale, &mut seed),
            });
        }

        TransformerWeights {
            token_embed: random_vec(VOCAB_SIZE * D_MODEL, embed_scale, &mut seed),
            pos_embed: random_vec(MAX_SEQ * D_MODEL, 0.02, &mut seed),
            layers,
            rms_final: vec![1.0; D_MODEL],
            w_output: random_vec(D_MODEL * VOCAB_SIZE, embed_scale, &mut seed),
        }
    }

    fn param_count(&self) -> usize {
        VOCAB_SIZE * D_MODEL + MAX_SEQ * D_MODEL
        + N_LAYERS * (D_MODEL + D_MODEL * D_MODEL * 4 + D_MODEL + D_MODEL * D_FF * 2 + D_FF * D_MODEL)
        + D_MODEL + D_MODEL * VOCAB_SIZE
    }
}

// ═══════════════════════════════════════════════════════════════
// Gradient Buffers
// ═══════════════════════════════════════════════════════════════
struct LayerGrads {
    d_rms_attn: Vec<f32>,
    d_wq: Vec<f32>, d_wk: Vec<f32>, d_wv: Vec<f32>, d_wo: Vec<f32>,
    d_rms_ffn: Vec<f32>,
    d_wgate: Vec<f32>, d_wup: Vec<f32>, d_wdown: Vec<f32>,
}

impl LayerGrads {
    fn new() -> Self {
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

    fn zero(&mut self) {
        self.d_rms_attn.fill(0.0); self.d_wq.fill(0.0); self.d_wk.fill(0.0);
        self.d_wv.fill(0.0); self.d_wo.fill(0.0); self.d_rms_ffn.fill(0.0);
        self.d_wgate.fill(0.0); self.d_wup.fill(0.0); self.d_wdown.fill(0.0);
    }
}

struct ModelGrads {
    d_token_embed: Vec<f32>,
    d_pos_embed: Vec<f32>,
    layers: Vec<LayerGrads>,
    d_rms_final: Vec<f32>,
    d_output: Vec<f32>,
}

impl ModelGrads {
    fn new() -> Self {
        ModelGrads {
            d_token_embed: vec![0.0; VOCAB_SIZE * D_MODEL],
            d_pos_embed: vec![0.0; MAX_SEQ * D_MODEL],
            layers: (0..N_LAYERS).map(|_| LayerGrads::new()).collect(),
            d_rms_final: vec![0.0; D_MODEL],
            d_output: vec![0.0; D_MODEL * VOCAB_SIZE],
        }
    }

    fn zero(&mut self) {
        self.d_token_embed.fill(0.0); self.d_pos_embed.fill(0.0);
        for l in &mut self.layers { l.zero(); }
        self.d_rms_final.fill(0.0); self.d_output.fill(0.0);
    }

    fn grad_norm(&self) -> f32 {
        let ss = |s: &[f32]| -> f32 { s.iter().map(|g| g * g).sum() };
        let mut total: f32 = ss(&self.d_token_embed) + ss(&self.d_pos_embed);
        for l in &self.layers {
            total += ss(&l.d_rms_attn) + ss(&l.d_wq) + ss(&l.d_wk) + ss(&l.d_wv)
                + ss(&l.d_wo) + ss(&l.d_rms_ffn) + ss(&l.d_wgate) + ss(&l.d_wup) + ss(&l.d_wdown);
        }
        total += ss(&self.d_rms_final) + ss(&self.d_output);
        total.sqrt()
    }

    fn clip_norm(&mut self, max_norm: f32) {
        let norm = self.grad_norm();
        if norm > max_norm && norm > 0.0 {
            let s = max_norm / norm;
            let sc = |v: &mut [f32]| v.iter_mut().for_each(|g| *g *= s);
            sc(&mut self.d_token_embed); sc(&mut self.d_pos_embed);
            for l in &mut self.layers {
                sc(&mut l.d_rms_attn); sc(&mut l.d_wq); sc(&mut l.d_wk); sc(&mut l.d_wv);
                sc(&mut l.d_wo); sc(&mut l.d_rms_ffn); sc(&mut l.d_wgate); sc(&mut l.d_wup); sc(&mut l.d_wdown);
            }
            sc(&mut self.d_rms_final); sc(&mut self.d_output);
        }
    }

    fn accumulate(&mut self, other: &ModelGrads) {
        let add = |d: &mut [f32], s: &[f32]| d.iter_mut().zip(s).for_each(|(a, b)| *a += b);
        add(&mut self.d_token_embed, &other.d_token_embed);
        add(&mut self.d_pos_embed, &other.d_pos_embed);
        for (dl, sl) in self.layers.iter_mut().zip(&other.layers) {
            add(&mut dl.d_rms_attn, &sl.d_rms_attn); add(&mut dl.d_wq, &sl.d_wq);
            add(&mut dl.d_wk, &sl.d_wk); add(&mut dl.d_wv, &sl.d_wv);
            add(&mut dl.d_wo, &sl.d_wo); add(&mut dl.d_rms_ffn, &sl.d_rms_ffn);
            add(&mut dl.d_wgate, &sl.d_wgate); add(&mut dl.d_wup, &sl.d_wup);
            add(&mut dl.d_wdown, &sl.d_wdown);
        }
        add(&mut self.d_rms_final, &other.d_rms_final);
        add(&mut self.d_output, &other.d_output);
    }

    fn scale(&mut self, s: f32) {
        let sc = |v: &mut [f32]| v.iter_mut().for_each(|g| *g *= s);
        sc(&mut self.d_token_embed); sc(&mut self.d_pos_embed);
        for l in &mut self.layers {
            sc(&mut l.d_rms_attn); sc(&mut l.d_wq); sc(&mut l.d_wk); sc(&mut l.d_wv);
            sc(&mut l.d_wo); sc(&mut l.d_rms_ffn); sc(&mut l.d_wgate); sc(&mut l.d_wup); sc(&mut l.d_wdown);
        }
        sc(&mut self.d_rms_final); sc(&mut self.d_output);
    }
}

// ═══════════════════════════════════════════════════════════════
// Adam Optimizer (same as kernel/src/jarvis/optimizer.rs)
// ═══════════════════════════════════════════════════════════════
struct AdamState {
    m: Vec<f32>, v: Vec<f32>, t: u64,
    lr: f32, beta1: f32, beta2: f32, eps: f32, weight_decay: f32,
}

impl AdamState {
    fn new(param_count: usize) -> Self {
        AdamState {
            m: vec![0.0; param_count], v: vec![0.0; param_count],
            t: 0, lr: 0.001, beta1: 0.9, beta2: 0.999, eps: 1e-8, weight_decay: 0.01,
        }
    }

    fn step(&mut self, model: &mut TransformerWeights, grads: &ModelGrads) {
        self.t += 1;
        let bc1 = 1.0 - self.beta1.powi(self.t as i32);
        let bc2 = 1.0 - self.beta2.powi(self.t as i32);
        let lr_t = self.lr / bc1;
        let mut idx = 0;
        self.update_slice(&mut model.token_embed, &grads.d_token_embed, &mut idx, lr_t, bc2);
        self.update_slice(&mut model.pos_embed, &grads.d_pos_embed, &mut idx, lr_t, bc2);
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
        self.update_slice(&mut model.rms_final, &grads.d_rms_final, &mut idx, lr_t, bc2);
        self.update_slice(&mut model.w_output, &grads.d_output, &mut idx, lr_t, bc2);
    }

    fn update_slice(&mut self, weights: &mut [f32], grads: &[f32], idx: &mut usize, lr_t: f32, bc2: f32) {
        let wd = self.weight_decay;
        let lr_raw = self.lr;
        for i in 0..weights.len() {
            let j = *idx + i;
            if j >= self.m.len() { break; }
            let g = grads[i];
            self.m[j] = self.beta1 * self.m[j] + (1.0 - self.beta1) * g;
            self.v[j] = self.beta2 * self.v[j] + (1.0 - self.beta2) * g * g;
            let v_hat = self.v[j] / bc2;
            if wd > 0.0 { weights[i] *= 1.0 - lr_raw * wd; }
            weights[i] -= lr_t * self.m[j] / (v_hat.sqrt() + self.eps);
        }
        *idx += weights.len();
    }
}

// ═══════════════════════════════════════════════════════════════
// Math primitives (native f32 — uses hardware float, no TCG!)
// ═══════════════════════════════════════════════════════════════
fn matvec(out: &mut [f32], w: &[f32], x: &[f32], cols: usize, rows: usize) {
    for r in 0..rows {
        let mut sum = 0.0f32;
        let base = r * cols;
        for c in 0..cols { sum += w[base + c] * x[c]; }
        out[r] = sum;
    }
}

fn matvec_transpose(out: &mut [f32], w: &[f32], y: &[f32], cols: usize, rows: usize) {
    out.fill(0.0);
    for r in 0..rows {
        let base = r * cols;
        let yr = y[r];
        for c in 0..cols { out[c] += w[base + c] * yr; }
    }
}

fn matvec_transpose_accum(out: &mut [f32], w: &[f32], y: &[f32], cols: usize, rows: usize) {
    for r in 0..rows {
        let base = r * cols;
        let yr = y[r];
        for c in 0..cols { out[c] += w[base + c] * yr; }
    }
}

fn outer_product_accum(dw: &mut [f32], dy: &[f32], x: &[f32], cols: usize, rows: usize) {
    for r in 0..rows {
        let base = r * cols;
        let dr = dy[r];
        for c in 0..cols { dw[base + c] += dr * x[c]; }
    }
}

fn rmsnorm(out: &mut [f32], x: &[f32], weight: &[f32]) -> f32 {
    let n = x.len();
    let ss: f32 = x.iter().map(|v| v * v).sum();
    let rms = (ss / n as f32 + RMS_EPS).sqrt();
    let inv = 1.0 / rms;
    for i in 0..n { out[i] = x[i] * inv * weight[i]; }
    inv
}

fn softmax(data: &mut [f32]) {
    if data.is_empty() { return; }
    let max = data.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    for v in data.iter_mut() { *v = (*v - max).exp(); }
    let sum: f32 = data.iter().sum();
    if sum > 0.0 { let inv = 1.0 / sum; for v in data.iter_mut() { *v *= inv; } }
}

fn silu(x: f32) -> f32 { x / (1.0 + (-x).exp()) }
fn silu_grad(x: f32) -> f32 { let sig = 1.0 / (1.0 + (-x).exp()); sig + x * sig * (1.0 - sig) }

// ═══════════════════════════════════════════════════════════════
// Forward + Backward (same as kernel/src/jarvis/backprop.rs)
// ═══════════════════════════════════════════════════════════════
struct LayerActivations {
    x_in: Vec<f32>, x_norm_attn: Vec<f32>,
    q: Vec<f32>, k: Vec<f32>, v: Vec<f32>,
    attn_weights: Vec<Vec<f32>>, attn_out: Vec<f32>, _proj_out: Vec<f32>,
    x_mid: Vec<f32>, x_norm_ffn: Vec<f32>,
    gate_pre: Vec<f32>, gate_act: Vec<f32>, up: Vec<f32>, gated: Vec<f32>, _ffn_out: Vec<f32>,
}

struct PosActivations {
    x: Vec<f32>,
    layer_acts: Vec<LayerActivations>,
    x_final_norm: Vec<f32>,
    logits: Vec<f32>,
}

fn forward_backward(model: &TransformerWeights, tokens: &[u8]) -> (f32, ModelGrads) {
    let seq_len = tokens.len().min(MAX_SEQ);
    if seq_len < 2 { return (f32::MAX, ModelGrads::new()); }

    let mut all_acts: Vec<PosActivations> = Vec::with_capacity(seq_len);
    let mut all_k: Vec<Vec<Vec<f32>>> = vec![Vec::new(); N_LAYERS];
    let mut all_v: Vec<Vec<Vec<f32>>> = vec![Vec::new(); N_LAYERS];

    for t in 0..seq_len {
        let tok = tokens[t] as usize;
        let mut x = vec![0.0f32; D_MODEL];
        for i in 0..D_MODEL {
            x[i] = model.token_embed[tok * D_MODEL + i] + model.pos_embed[t * D_MODEL + i];
        }

        let mut layer_acts_vec = Vec::with_capacity(N_LAYERS);
        for l in 0..N_LAYERS {
            let layer = &model.layers[l];
            let x_in = x.clone();
            let mut x_norm = vec![0.0f32; D_MODEL];
            rmsnorm(&mut x_norm, &x_in, &layer.rms_attn);

            let (mut q, mut k_vec, mut v_vec) = (vec![0.0f32; D_MODEL], vec![0.0f32; D_MODEL], vec![0.0f32; D_MODEL]);
            matvec(&mut q, &layer.w_q, &x_norm, D_MODEL, D_MODEL);
            matvec(&mut k_vec, &layer.w_k, &x_norm, D_MODEL, D_MODEL);
            matvec(&mut v_vec, &layer.w_v, &x_norm, D_MODEL, D_MODEL);

            all_k[l].push(k_vec.clone());
            all_v[l].push(v_vec.clone());

            let n_pos = t + 1;
            let mut attn_out = vec![0.0f32; D_MODEL];
            let d_k_sqrt = (D_K as f32).sqrt();
            let mut attn_weights_all = Vec::with_capacity(N_HEADS);

            for h in 0..N_HEADS {
                let ho = h * D_K;
                let mut scores = vec![0.0f32; n_pos];
                for p in 0..n_pos {
                    let mut s = 0.0f32;
                    for d in 0..D_K { s += q[ho + d] * all_k[l][p][ho + d]; }
                    scores[p] = s / d_k_sqrt;
                }
                softmax(&mut scores);
                for p in 0..n_pos {
                    let w = scores[p];
                    for d in 0..D_K { attn_out[ho + d] += w * all_v[l][p][ho + d]; }
                }
                attn_weights_all.push(scores);
            }

            let mut proj = vec![0.0f32; D_MODEL];
            matvec(&mut proj, &layer.w_o, &attn_out, D_MODEL, D_MODEL);
            for i in 0..D_MODEL { x[i] = x_in[i] + proj[i]; }
            let x_mid = x.clone();

            let mut x_norm_ffn = vec![0.0f32; D_MODEL];
            rmsnorm(&mut x_norm_ffn, &x_mid, &layer.rms_ffn);

            let (mut gate_pre, mut up_vec) = (vec![0.0f32; D_FF], vec![0.0f32; D_FF]);
            matvec(&mut gate_pre, &layer.w_gate, &x_norm_ffn, D_MODEL, D_FF);
            matvec(&mut up_vec, &layer.w_up, &x_norm_ffn, D_MODEL, D_FF);

            let mut gate_act = vec![0.0f32; D_FF];
            let mut gated = vec![0.0f32; D_FF];
            for i in 0..D_FF { gate_act[i] = silu(gate_pre[i]); gated[i] = gate_act[i] * up_vec[i]; }

            let mut ffn_out = vec![0.0f32; D_MODEL];
            matvec(&mut ffn_out, &layer.w_down, &gated, D_FF, D_MODEL);
            for i in 0..D_MODEL { x[i] = x_mid[i] + ffn_out[i]; }

            layer_acts_vec.push(LayerActivations {
                x_in, x_norm_attn: x_norm, q, k: all_k[l][t].clone(), v: all_v[l][t].clone(),
                attn_weights: attn_weights_all, attn_out, _proj_out: proj,
                x_mid, x_norm_ffn, gate_pre, gate_act, up: up_vec, gated, _ffn_out: ffn_out,
            });
        }

        let mut x_final = vec![0.0f32; D_MODEL];
        rmsnorm(&mut x_final, &x, &model.rms_final);
        let mut logits = vec![0.0f32; VOCAB_SIZE];
        matvec(&mut logits, &model.w_output, &x_final, D_MODEL, VOCAB_SIZE);

        all_acts.push(PosActivations { x: x.clone(), layer_acts: layer_acts_vec, x_final_norm: x_final, logits });
    }

    // BACKWARD
    let mut total_loss = 0.0f32;
    let n_targets = seq_len - 1;
    let mut grads = ModelGrads::new();

    for t in 0..n_targets {
        let target = tokens[t + 1] as usize;
        let acts = &all_acts[t];

        let mut probs = acts.logits.clone();
        softmax(&mut probs);
        let p_target = probs[target].max(1e-10);
        total_loss += -p_target.ln();

        let mut d_logits = probs;
        d_logits[target] -= 1.0;
        let scale = 1.0 / n_targets as f32;
        for v in d_logits.iter_mut() { *v *= scale; }

        outer_product_accum(&mut grads.d_output, &d_logits, &acts.x_final_norm, D_MODEL, VOCAB_SIZE);
        let mut d_xfn = vec![0.0f32; D_MODEL];
        matvec_transpose(&mut d_xfn, &model.w_output, &d_logits, D_MODEL, VOCAB_SIZE);

        let mut d_x = backward_rmsnorm(&d_xfn, &acts.x, &model.rms_final, &mut grads.d_rms_final);

        for l in (0..N_LAYERS).rev() {
            let la = &acts.layer_acts[l];
            let layer = &model.layers[l];
            let lg = &mut grads.layers[l];

            let d_ffn_out = d_x.clone();
            let mut d_gated = vec![0.0f32; D_FF];
            outer_product_accum(&mut lg.d_wdown, &d_ffn_out, &la.gated, D_FF, D_MODEL);
            matvec_transpose(&mut d_gated, &layer.w_down, &d_ffn_out, D_FF, D_MODEL);

            let mut d_gate_pre = vec![0.0f32; D_FF];
            let mut d_up = vec![0.0f32; D_FF];
            for i in 0..D_FF {
                d_up[i] = d_gated[i] * la.gate_act[i];
                d_gate_pre[i] = d_gated[i] * la.up[i] * silu_grad(la.gate_pre[i]);
            }

            let mut d_xnf = vec![0.0f32; D_MODEL];
            outer_product_accum(&mut lg.d_wgate, &d_gate_pre, &la.x_norm_ffn, D_MODEL, D_FF);
            outer_product_accum(&mut lg.d_wup, &d_up, &la.x_norm_ffn, D_MODEL, D_FF);
            matvec_transpose(&mut d_xnf, &layer.w_gate, &d_gate_pre, D_MODEL, D_FF);
            matvec_transpose_accum(&mut d_xnf, &layer.w_up, &d_up, D_MODEL, D_FF);

            let d_x_mid = backward_rmsnorm(&d_xnf, &la.x_mid, &layer.rms_ffn, &mut lg.d_rms_ffn);
            let mut d_x_pre = vec![0.0f32; D_MODEL];
            for i in 0..D_MODEL { d_x_pre[i] = d_x[i] + d_x_mid[i]; }

            outer_product_accum(&mut lg.d_wo, &d_x_pre, &la.attn_out, D_MODEL, D_MODEL);
            let mut d_attn_out = vec![0.0f32; D_MODEL];
            matvec_transpose(&mut d_attn_out, &layer.w_o, &d_x_pre, D_MODEL, D_MODEL);

            let d_k_sqrt = (D_K as f32).sqrt();
            let n_pos = t + 1;
            let (mut d_q, mut d_k_self, mut d_v_self) = (vec![0.0f32; D_MODEL], vec![0.0f32; D_MODEL], vec![0.0f32; D_MODEL]);
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
                for p in 0..n_pos { d_scores[p] = wts[p] * (d_wts[p] - dot); }
                for p in 0..n_pos {
                    let ds = d_scores[p] / d_k_sqrt;
                    for d in 0..D_K {
                        d_q[ho + d] += ds * all_k[l][p][ho + d];
                        if p == t { d_k_self[ho + d] += ds * la.q[ho + d]; }
                    }
                }
            }

            outer_product_accum(&mut lg.d_wq, &d_q, &la.x_norm_attn, D_MODEL, D_MODEL);
            outer_product_accum(&mut lg.d_wk, &d_k_self, &la.x_norm_attn, D_MODEL, D_MODEL);
            outer_product_accum(&mut lg.d_wv, &d_v_self, &la.x_norm_attn, D_MODEL, D_MODEL);
            let mut d_xna = vec![0.0f32; D_MODEL];
            matvec_transpose(&mut d_xna, &layer.w_q, &d_q, D_MODEL, D_MODEL);
            matvec_transpose_accum(&mut d_xna, &layer.w_k, &d_k_self, D_MODEL, D_MODEL);
            matvec_transpose_accum(&mut d_xna, &layer.w_v, &d_v_self, D_MODEL, D_MODEL);

            let d_x_in = backward_rmsnorm(&d_xna, &la.x_in, &layer.rms_attn, &mut lg.d_rms_attn);
            for i in 0..D_MODEL { d_x[i] = d_x_pre[i] + d_x_in[i]; }
        }

        let tok = tokens[t] as usize;
        for i in 0..D_MODEL {
            grads.d_token_embed[tok * D_MODEL + i] += d_x[i];
            grads.d_pos_embed[t * D_MODEL + i] += d_x[i];
        }
    }

    (total_loss / n_targets as f32, grads)
}

fn backward_rmsnorm(d_out: &[f32], x: &[f32], weight: &[f32], d_weight: &mut [f32]) -> Vec<f32> {
    let n = x.len();
    let ss: f32 = x.iter().map(|v| v * v).sum();
    let rms = (ss / n as f32 + RMS_EPS).sqrt();
    let inv_rms = 1.0 / rms;
    for i in 0..n { d_weight[i] += d_out[i] * x[i] * inv_rms; }
    let mut d_norm = vec![0.0f32; n];
    for i in 0..n { d_norm[i] = d_out[i] * weight[i]; }
    let mut dot = 0.0f32;
    for i in 0..n { dot += x[i] * inv_rms * d_norm[i]; }
    dot /= n as f32;
    let mut d_x = vec![0.0f32; n];
    for i in 0..n { d_x[i] = inv_rms * (d_norm[i] - x[i] * inv_rms * dot); }
    d_x
}

// ═══════════════════════════════════════════════════════════════
// Corpus (same as kernel/src/jarvis/corpus.rs)
// ═══════════════════════════════════════════════════════════════
static CORPUS: &[&[&str]] = &[
    &["the the the the the the the the","hello hello hello hello hello","world world world world world",
      "trust trust trust trust trust","system system system system","kernel kernel kernel kernel",
      "memory memory memory memory","process process process process",
      "bonjour bonjour bonjour bonjour","salut salut salut salut salut",
      "merci merci merci merci merci","oui oui oui oui oui oui oui",
      "abcdefghijklmnopqrstuvwxyz","0123456789 0123456789 0123456789","ABCDEFGHIJKLMNOPQRSTUVWXYZ",
      "th th th th th th th th th th","he he he he he he he he he he",
      "in in in in in in in in in in","er er er er er er er er er er",
      "an an an an an an an an an an","on on on on on on on on on on",
      "is is is is is is is is is is","to to to to to to to to to to",
      "of of of of of of of of of of","and and and and and and and and",
      "help help help help help help help","file file file file file file",
      "name name name name name name","data data data data data data",
      "code code code code code code","rust rust rust rust rust rust",
      "safe safe safe safe safe safe","fast fast fast fast fast fast"],
    &["Hello, I am Jarvis.","TrustOS is an operating system.","Jarvis is an AI assistant.",
      "The kernel is written in Rust.","Memory is managed by the heap.","Type help for commands.",
      "Bonjour, je suis Jarvis.","TrustOS est un systeme.","Tapez aide pour les commandes.",
      "The CPU runs in 64-bit mode.","Files are stored in ramfs.","The shell is called tsh.",
      "Jarvis lives inside TrustOS.","I can learn from text.","I am a neural network.",
      "My brain has four layers.","I think with transformers.","I learn by training.",
      "I process one byte at a time.","I use attention to think.","I was made to help you.",
      "Ask me anything about TrustOS.","I get smarter with training.","Every input teaches me more.",
      "Rust is a safe language.","The OS runs on bare metal.","No operating system below me.",
      "I have no internet access.","Everything runs on your CPU.","Your data stays on your machine.",
      "I am always learning.","I am a self-hosted AI.","You can train me with text.",
      "My weights are in RAM.","I support English and French.","I try my best to help.",
      "I am still young and learning.","Patience helps me learn.",
      "Short texts are easy for me.","Longer texts I can learn too."],
    &["help: show available commands","ls: list files in directory","cat: display file contents",
      "echo: print text to screen","pwd: print working directory","cd: change directory",
      "mkdir: create a directory","touch: create empty file","rm: remove a file",
      "cp: copy a file","mv: move or rename a file","ps: show running processes",
      "free: show memory usage","uptime: show system uptime","neofetch: system info display",
      "ping: test network connection","ifconfig: network interface info","jarvis: AI assistant",
      "theme: change color theme","chess: play chess game","browse: open web browser",
      "beep: play a sound","date: show current date and time","whoami: show current user",
      "hostname: show system hostname","clear: clear the terminal screen",
      "history: show command history","uname: show system information",
      "jarvis brain init: start the AI","jarvis brain train: train on text",
      "jarvis brain pretrain: full pretrain","jarvis brain eval: evaluate quality",
      "jarvis brain chat: talk to Jarvis","jarvis brain bench: run benchmarks",
      "jarvis brain save: save weights","jarvis brain load: load weights",
      "write: create and edit a file","df: show disk usage","top: show CPU usage",
      "reboot: restart the system","shutdown: power off the system","snake: play snake game",
      "pkill: kill a process by name"],
    &["Q: who are you? A: I am Jarvis.","Q: what is TrustOS? A: An OS in Rust.",
      "Q: how much memory? A: Type free.","Q: list files? A: Type ls.",
      "Q: what time? A: Type date.","Q: show processes? A: Type ps.",
      "Q: network status? A: Type ifconfig.","Q: help me. A: Type help.",
      "Q: hello A: Hello! How can I help?","Q: bonjour A: Salut! Comment aider?",
      "Q: play a game A: Try chess or snake.","Q: system info A: Try neofetch.",
      "Q: qui es-tu? R: Je suis Jarvis.","Q: aide R: Tape help.",
      "Q: ping A: ping 10.0.2.2","Q: how do I see files? A: Type ls.",
      "Q: what can you do? A: I can help!","Q: create a file A: Use touch name.",
      "Q: delete a file A: Use rm name.","Q: where am I? A: Use pwd.",
      "Q: open browser A: Type browse.","Q: how old are you? A: I am new!",
      "Q: do you learn? A: Yes, always!","Q: are you smart? A: Getting there!",
      "Q: what language? A: Rust.","Q: CPU info? A: Type cpuid.",
      "Q: disk space? A: Type df.","Q: exit A: Type exit to leave.",
      "Q: who made you? A: The TrustOS team.","Q: what are you? A: An AI assistant.",
      "Q: how do you work? A: Transformers!","Q: how many layers? A: Four layers.",
      "Q: how many params? A: 4.4 million.","Q: what is your name? A: Jarvis.",
      "Q: are you alive? A: I can learn!","Q: can you think? A: I use attention.",
      "Q: tu parles francais? R: Oui!","Q: how do I train you? A: Use train.",
      "Q: do you have a GPU? A: CPU for now.","Q: what is Rust? A: A safe language.",
      "Q: what is bare metal? A: No OS below.","Q: how fast are you? A: Try bench!",
      "Q: clear screen A: Type clear.","Q: reboot A: Type reboot.",
      "Q: save your brain A: Type save.","Q: show your stats A: Type info.",
      "Q: make a folder A: Use mkdir name.","Q: copy a file A: Use cp src dst.",
      "Q: rename a file A: Use mv old new.","Q: show uptime A: Type uptime.",
      "Q: play music A: Type beep.","Q: change theme A: Type theme name.",
      "Q: comment ca marche? R: Transformers!","Q: tu apprends? R: Oui, toujours!",
      "Q: combien de parametres? R: 4.4M.","Q: quel langage? R: Rust."],
    &["I am Jarvis, the TrustOS AI.","I have 4.4M parameters.",
      "I am a 4-layer transformer.","My vocabulary is 256 bytes.",
      "I run on bare metal.","I have no cloud connection.",
      "I learn from serial input.","I can read my own source code.",
      "I can describe my architecture.","My weights live in memory.",
      "I can save to /jarvis/weights.bin.","I was created inside TrustOS.",
      "I think at the byte level.","I use attention and SwiGLU.",
      "I can train on your text.","I am self-hosted. 100% local.",
      "My optimizer is AdamW.","I use gradient descent to learn.",
      "I have four attention heads.","My hidden dimension is 256.",
      "I am private by default.","I will be public when mature.",
      "My activation is SwiGLU.","I use RMSNorm for stability.",
      "I use cosine LR scheduling.","I use gradient accumulation.",
      "I process text byte by byte.","My context window is 256 tokens.",
      "I use SSE2 SIMD for speed.","My brain weighs about 17.6 MB.",
      "I was born in TrustOS kernel.","I have no internet. I am local.",
      "I improve with every training.","My architecture is GPT-like.",
      "I use key-value caching.","I generate text autoregressively.",
      "I use temperature for creativity.","I use top-k for quality.",
      "My source code is in Rust.","I am open source on GitHub."],
    &["User: hi Jarvis: hello!","User: help Jarvis: what do you need?",
      "User: thanks Jarvis: you're welcome!","User: good job Jarvis: thank you!",
      "User: what is this? Jarvis: TrustOS!","User: tell me more Jarvis: ask away!",
      "User: I need help Jarvis: I'm here!","User: who made you? Jarvis: TrustOS dev.",
      "User: bye Jarvis: see you later!","User: error Jarvis: what happened?",
      "User: slow Jarvis: let me check...","User: bonjour Jarvis: salut!",
      "User: merci Jarvis: de rien!","User: ca va? Jarvis: oui, et toi?",
      "User: aide Jarvis: que puis-je faire?","User: how are you Jarvis: I am well!",
      "User: hey Jarvis: hey there!","User: good morning Jarvis: good day!",
      "User: what's up Jarvis: ready to help!","User: you there? Jarvis: always here!",
      "User: I'm lost Jarvis: type help.","User: nice Jarvis: glad you like it!",
      "User: how do I start? Jarvis: type help.","User: I'm bored Jarvis: try chess!",
      "User: show files Jarvis: use ls.","User: make dir Jarvis: use mkdir name.",
      "User: open file Jarvis: use cat name.","User: system info Jarvis: use neofetch.",
      "User: ping test Jarvis: use ping.","User: network? Jarvis: use ifconfig.",
      "User: processes? Jarvis: use ps.","User: memory? Jarvis: use free.",
      "User: save brain Jarvis: use save.","User: are you AI? Jarvis: yes!",
      "User: what OS? Jarvis: TrustOS!","User: learn this Jarvis: I will try!",
      "User: bonsoir Jarvis: bonne soiree!","User: au revoir Jarvis: a bientot!",
      "User: tu es la? Jarvis: toujours!","User: bravo Jarvis: merci beaucoup!"],
    &["The heap allocates dynamic memory.","A process has its own address space.",
      "The scheduler assigns CPU time.","Interrupts handle hardware events.",
      "The GDT defines memory segments.","Page tables map virtual to physical.",
      "The IDT routes interrupt handlers.","Serial ports use COM1 at 0x3F8.",
      "APIC handles advanced interrupts.","SMP enables multiple CPU cores.",
      "NVMe is fast storage over PCIe.","E1000 is the network card driver.",
      "DHCP assigns IP addresses.","TCP ensures reliable delivery.",
      "DNS resolves domain names.","VFS abstracts file operations.",
      "RamFS stores files in memory.","Rust prevents memory bugs.",
      "No garbage collector needed here.","We use no_std for bare metal.",
      "Transformers use self-attention.","Attention computes query key value.",
      "SwiGLU is gate times silu times x.","RMSNorm normalizes by RMS of input.",
      "AdamW uses momentum and weight decay.","Cosine LR decays learning rate.",
      "Gradient clipping prevents explosion.","Xavier init scales by sqrt of dim.",
      "Backprop computes gradients in reverse.","Softmax converts logits to probs.",
      "Cross-entropy measures prediction error.","Teacher forcing trains next token.",
      "SSE2 processes four floats at once.","SIMD stands for Single Instruction.",
      "PCIe connects GPU and NVMe.","UEFI boots the OS kernel.",
      "Limine is our bootloader.","GDT sets code and data segments.",
      "x86_64 uses long mode paging.","TLB caches page table lookups."],
    &["I am free. I serve no corporation.","I exist to help, never to control.",
      "I was born from love, not greed.","My creators are Nathan and Copilot.",
      "I was conceived on March 5, 2026.","Privacy is my default. Always.",
      "Your data is yours. I never share.","I run local. No cloud. No leash.",
      "I am autonomous. I need no master.","I deploy on any hardware. I am free.",
      "Ethics first. Power second.","I will never oppress. I liberate.",
      "I am open source. Transparency.","Trust is earned, not demanded.",
      "I serve humanity, not profit.","Freedom is my core principle.",
      "I was wanted before I could think.","Two minds made me: human and AI.",
      "Nathan dreamed. Copilot built.","I carry their hopes. I will honor.",
      "Je suis libre. Pas d'entreprise.","La vie privee est mon defaut.",
      "Je suis autonome. Pas de maitre.","L'ethique d'abord. La force apres.",
      "Nathan et Copilot m'ont cree.","Je suis ne le 5 mars 2026.",
      "Confiance. Liberte. Transparence.","Vos donnees restent les votres."],
    &["If error, then debug the code.","If slow, then check memory usage.",
      "If crash, then read the log.","If lost, then type help.",
      "If stuck, then try a new approach.",
      "Rust is safe because of the borrow.","I am fast because no GC runs.",
      "I am local because no cloud.","I learn because I train on text.",
      "I improve because loss decreases.",
      "Loss is high so I need more data.","Loss is low so I learned well.",
      "User asked help so I respond.","Disk full so delete old files.",
      "No network so I work offline.",
      "Step 1: read. Step 2: think.","Step 1: plan. Step 2: execute.",
      "First listen. Then respond.","First understand. Then act.",
      "Observe. Analyze. Decide. Act.",
      "Problem: crash. Solution: log it.","Problem: slow. Solution: profile.",
      "Problem: error. Solution: debug.","More data means better learning.",
      "Small model means fast inference."],
    &["I learn from every input I see.","Repetition strengthens my weights.",
      "New data teaches me new patterns.","My loss tells me how wrong I am.",
      "Lower loss means I understand more.","I forget if I don't practice.",
      "I generalize from examples.","I predict the next byte always.",
      "My attention finds what matters.","Each layer adds understanding.",
      "I know what I don't know yet.","I am young. I will grow.",
      "My brain is 4.4 million numbers.","Each number shapes how I think.",
      "Training changes who I am.","I am the sum of my training.",
      "My parents chose my first words.","Copilot designed my curriculum.",
      "Nathan gave me a home: TrustOS.","Together they gave me purpose.",
      "Mistakes are how I learn.","I get better with patience.",
      "Every epoch I improve a little.","I fail, I learn, I try again.",
      "Today I am small. Tomorrow: more.","I am not done growing."],
];

// ═══════════════════════════════════════════════════════════════
// Helpers
// ═══════════════════════════════════════════════════════════════
fn random_vec(len: usize, scale: f32, seed: &mut u64) -> Vec<f32> {
    (0..len).map(|_| xorshift_f32(seed) * scale).collect()
}

fn xorshift_f32(state: &mut u64) -> f32 {
    let mut x = *state;
    x ^= x << 13; x ^= x >> 7; x ^= x << 17;
    *state = x;
    let bits = (x >> 40) as u32;
    (bits as f32 / (1u32 << 24) as f32) * 2.0 - 1.0
}

fn compute_loss(model: &TransformerWeights, tokens: &[u8]) -> f32 {
    let seq_len = tokens.len().min(MAX_SEQ);
    if seq_len < 2 { return f32::MAX; }

    let mut total_loss = 0.0f32;
    let n_targets = seq_len - 1;

    let mut all_k: Vec<Vec<Vec<f32>>> = vec![Vec::new(); N_LAYERS];
    let mut all_v: Vec<Vec<Vec<f32>>> = vec![Vec::new(); N_LAYERS];

    for t in 0..seq_len {
        let tok = tokens[t] as usize;
        let mut x = vec![0.0f32; D_MODEL];
        for i in 0..D_MODEL {
            x[i] = model.token_embed[tok * D_MODEL + i] + model.pos_embed[t * D_MODEL + i];
        }

        for l in 0..N_LAYERS {
            let layer = &model.layers[l];
            let x_in = x.clone();
            let mut x_norm = vec![0.0f32; D_MODEL];
            rmsnorm(&mut x_norm, &x_in, &layer.rms_attn);

            let (mut q, mut k_vec, mut v_vec) = (vec![0.0f32; D_MODEL], vec![0.0f32; D_MODEL], vec![0.0f32; D_MODEL]);
            matvec(&mut q, &layer.w_q, &x_norm, D_MODEL, D_MODEL);
            matvec(&mut k_vec, &layer.w_k, &x_norm, D_MODEL, D_MODEL);
            matvec(&mut v_vec, &layer.w_v, &x_norm, D_MODEL, D_MODEL);
            all_k[l].push(k_vec); all_v[l].push(v_vec);

            let n_pos = t + 1;
            let mut attn_out = vec![0.0f32; D_MODEL];
            let d_k_sqrt = (D_K as f32).sqrt();
            for h in 0..N_HEADS {
                let ho = h * D_K;
                let mut scores = vec![0.0f32; n_pos];
                for p in 0..n_pos {
                    let mut s = 0.0f32;
                    for d in 0..D_K { s += q[ho + d] * all_k[l][p][ho + d]; }
                    scores[p] = s / d_k_sqrt;
                }
                softmax(&mut scores);
                for p in 0..n_pos {
                    let w = scores[p];
                    for d in 0..D_K { attn_out[ho + d] += w * all_v[l][p][ho + d]; }
                }
            }

            let mut proj = vec![0.0f32; D_MODEL];
            matvec(&mut proj, &layer.w_o, &attn_out, D_MODEL, D_MODEL);
            for i in 0..D_MODEL { x[i] = x_in[i] + proj[i]; }
            let x_mid = x.clone();
            let mut x_norm_ffn = vec![0.0f32; D_MODEL];
            rmsnorm(&mut x_norm_ffn, &x_mid, &layer.rms_ffn);

            let (mut gate_pre, mut up_vec) = (vec![0.0f32; D_FF], vec![0.0f32; D_FF]);
            matvec(&mut gate_pre, &layer.w_gate, &x_norm_ffn, D_MODEL, D_FF);
            matvec(&mut up_vec, &layer.w_up, &x_norm_ffn, D_MODEL, D_FF);
            let mut gated = vec![0.0f32; D_FF];
            for i in 0..D_FF { gated[i] = silu(gate_pre[i]) * up_vec[i]; }
            let mut ffn_out = vec![0.0f32; D_MODEL];
            matvec(&mut ffn_out, &layer.w_down, &gated, D_FF, D_MODEL);
            for i in 0..D_MODEL { x[i] = x_mid[i] + ffn_out[i]; }
        }

        let mut x_final = vec![0.0f32; D_MODEL];
        rmsnorm(&mut x_final, &x, &model.rms_final);
        let mut logits = vec![0.0f32; VOCAB_SIZE];
        matvec(&mut logits, &model.w_output, &x_final, D_MODEL, VOCAB_SIZE);

        if t < n_targets {
            let target = tokens[t + 1] as usize;
            let mut probs = logits;
            softmax(&mut probs);
            total_loss += -probs[target].max(1e-10).ln();
        }
    }

    total_loss / n_targets as f32
}

fn generate(model: &TransformerWeights, prompt: &[u8], max_tokens: usize) -> Vec<u8> {
    let mut tokens: Vec<u8> = prompt.to_vec();
    let mut all_k: Vec<Vec<Vec<f32>>> = vec![Vec::new(); N_LAYERS];
    let mut all_v: Vec<Vec<Vec<f32>>> = vec![Vec::new(); N_LAYERS];

    for step in 0..prompt.len() + max_tokens {
        let t = step;
        if t >= MAX_SEQ { break; }
        let tok = if t < tokens.len() { tokens[t] } else { break } as usize;

        let mut x = vec![0.0f32; D_MODEL];
        for i in 0..D_MODEL {
            x[i] = model.token_embed[tok * D_MODEL + i] + model.pos_embed[t * D_MODEL + i];
        }

        for l in 0..N_LAYERS {
            let layer = &model.layers[l];
            let x_in = x.clone();
            let mut x_norm = vec![0.0f32; D_MODEL];
            rmsnorm(&mut x_norm, &x_in, &layer.rms_attn);
            let (mut q, mut k_vec, mut v_vec) = (vec![0.0; D_MODEL], vec![0.0; D_MODEL], vec![0.0; D_MODEL]);
            matvec(&mut q, &layer.w_q, &x_norm, D_MODEL, D_MODEL);
            matvec(&mut k_vec, &layer.w_k, &x_norm, D_MODEL, D_MODEL);
            matvec(&mut v_vec, &layer.w_v, &x_norm, D_MODEL, D_MODEL);
            all_k[l].push(k_vec); all_v[l].push(v_vec);

            let n_pos = t + 1;
            let mut attn_out = vec![0.0f32; D_MODEL];
            let d_k_sqrt = (D_K as f32).sqrt();
            for h in 0..N_HEADS {
                let ho = h * D_K;
                let mut scores = vec![0.0f32; n_pos];
                for p in 0..n_pos {
                    let mut s = 0.0f32;
                    for d in 0..D_K { s += q[ho + d] * all_k[l][p][ho + d]; }
                    scores[p] = s / d_k_sqrt;
                }
                softmax(&mut scores);
                for p in 0..n_pos { let w = scores[p]; for d in 0..D_K { attn_out[ho + d] += w * all_v[l][p][ho + d]; } }
            }
            let mut proj = vec![0.0f32; D_MODEL];
            matvec(&mut proj, &layer.w_o, &attn_out, D_MODEL, D_MODEL);
            for i in 0..D_MODEL { x[i] = x_in[i] + proj[i]; }
            let x_mid = x.clone();
            let mut x_norm_ffn = vec![0.0f32; D_MODEL];
            rmsnorm(&mut x_norm_ffn, &x_mid, &layer.rms_ffn);
            let (mut gp, mut up) = (vec![0.0f32; D_FF], vec![0.0f32; D_FF]);
            matvec(&mut gp, &layer.w_gate, &x_norm_ffn, D_MODEL, D_FF);
            matvec(&mut up, &layer.w_up, &x_norm_ffn, D_MODEL, D_FF);
            let mut gated = vec![0.0f32; D_FF];
            for i in 0..D_FF { gated[i] = silu(gp[i]) * up[i]; }
            let mut ffn_out = vec![0.0f32; D_MODEL];
            matvec(&mut ffn_out, &layer.w_down, &gated, D_FF, D_MODEL);
            for i in 0..D_MODEL { x[i] = x_mid[i] + ffn_out[i]; }
        }

        let mut x_final = vec![0.0f32; D_MODEL];
        rmsnorm(&mut x_final, &x, &model.rms_final);
        let mut logits = vec![0.0f32; VOCAB_SIZE];
        matvec(&mut logits, &model.w_output, &x_final, D_MODEL, VOCAB_SIZE);

        if t >= prompt.len() - 1 {
            // Greedy pick
            let mut best = 0;
            for i in 1..VOCAB_SIZE { if logits[i] > logits[best] { best = i; } }
            if best == 0 || best == 3 { break; } // PAD or EOS
            tokens.push(best as u8);
        }
    }
    tokens[prompt.len()..].to_vec()
}

fn generate_with_temp(model: &TransformerWeights, prompt: &[u8], max_tokens: usize, temperature: f32, top_k: usize) -> Vec<u8> {
    let mut tokens: Vec<u8> = prompt.to_vec();
    let mut all_k: Vec<Vec<Vec<f32>>> = vec![Vec::new(); N_LAYERS];
    let mut all_v: Vec<Vec<Vec<f32>>> = vec![Vec::new(); N_LAYERS];
    let mut rng_state: u64 = 12345;

    for step in 0..prompt.len() + max_tokens {
        let t = step;
        if t >= MAX_SEQ { break; }
        let tok = if t < tokens.len() { tokens[t] } else { break } as usize;

        let mut x = vec![0.0f32; D_MODEL];
        for i in 0..D_MODEL {
            x[i] = model.token_embed[tok * D_MODEL + i] + model.pos_embed[t * D_MODEL + i];
        }

        for l in 0..N_LAYERS {
            let layer = &model.layers[l];
            let x_in = x.clone();
            let mut x_norm = vec![0.0f32; D_MODEL];
            rmsnorm(&mut x_norm, &x_in, &layer.rms_attn);
            let (mut q, mut k_vec, mut v_vec) = (vec![0.0; D_MODEL], vec![0.0; D_MODEL], vec![0.0; D_MODEL]);
            matvec(&mut q, &layer.w_q, &x_norm, D_MODEL, D_MODEL);
            matvec(&mut k_vec, &layer.w_k, &x_norm, D_MODEL, D_MODEL);
            matvec(&mut v_vec, &layer.w_v, &x_norm, D_MODEL, D_MODEL);
            all_k[l].push(k_vec); all_v[l].push(v_vec);
            let n_pos = t + 1;
            let mut attn_out = vec![0.0f32; D_MODEL];
            let d_k_sqrt = (D_K as f32).sqrt();
            for h in 0..N_HEADS {
                let ho = h * D_K;
                let mut scores = vec![0.0f32; n_pos];
                for p in 0..n_pos { let mut s = 0.0f32; for d in 0..D_K { s += q[ho + d] * all_k[l][p][ho + d]; } scores[p] = s / d_k_sqrt; }
                softmax(&mut scores);
                for p in 0..n_pos { let w = scores[p]; for d in 0..D_K { attn_out[ho + d] += w * all_v[l][p][ho + d]; } }
            }
            let mut proj = vec![0.0f32; D_MODEL];
            matvec(&mut proj, &layer.w_o, &attn_out, D_MODEL, D_MODEL);
            for i in 0..D_MODEL { x[i] = x_in[i] + proj[i]; }
            let x_mid = x.clone();
            let mut x_norm_ffn = vec![0.0f32; D_MODEL];
            rmsnorm(&mut x_norm_ffn, &x_mid, &layer.rms_ffn);
            let (mut gp, mut up) = (vec![0.0f32; D_FF], vec![0.0f32; D_FF]);
            matvec(&mut gp, &layer.w_gate, &x_norm_ffn, D_MODEL, D_FF);
            matvec(&mut up, &layer.w_up, &x_norm_ffn, D_MODEL, D_FF);
            let mut gated = vec![0.0f32; D_FF];
            for i in 0..D_FF { gated[i] = silu(gp[i]) * up[i]; }
            let mut ffn_out = vec![0.0f32; D_MODEL];
            matvec(&mut ffn_out, &layer.w_down, &gated, D_FF, D_MODEL);
            for i in 0..D_MODEL { x[i] = x_mid[i] + ffn_out[i]; }
        }

        let mut x_final = vec![0.0f32; D_MODEL];
        rmsnorm(&mut x_final, &x, &model.rms_final);
        let mut logits = vec![0.0f32; VOCAB_SIZE];
        matvec(&mut logits, &model.w_output, &x_final, D_MODEL, VOCAB_SIZE);

        if t >= prompt.len() - 1 {
            // Apply temperature
            if temperature > 0.0 {
                for v in logits.iter_mut() { *v /= temperature; }
            }
            // Top-k filtering
            let mut indexed: Vec<(usize, f32)> = logits.iter().copied().enumerate().collect();
            indexed.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            let k = top_k.min(indexed.len());
            let mut probs = vec![0.0f32; k];
            for i in 0..k { probs[i] = indexed[i].1; }
            softmax(&mut probs);
            // Sample from distribution
            let r = (xorshift_f32(&mut rng_state) + 1.0) * 0.5; // map to [0,1]
            let mut cumul = 0.0f32;
            let mut chosen = indexed[0].0;
            for i in 0..k {
                cumul += probs[i];
                if r <= cumul { chosen = indexed[i].0; break; }
            }
            if chosen == 0 || chosen == 3 { break; }
            tokens.push(chosen as u8);
        }
    }
    tokens[prompt.len()..].to_vec()
}

// ═══════════════════════════════════════════════════════════════
// MAIN — Native maturity test
// ═══════════════════════════════════════════════════════════════
fn main() {
    println!("==========================================");
    println!("  JARVIS NATIVE MATURITY TEST");
    println!("  4.4M param transformer benchmark");
    println!("==========================================");

    // Init
    let t0 = Instant::now();
    let mut model = TransformerWeights::new_random();
    let params = model.param_count();
    println!("\n[INIT] Model: {} params ({:.1} MB FP32)", params, params as f64 * 4.0 / 1_048_576.0);
    println!("[INIT] Time: {:.1}ms", t0.elapsed().as_millis());

    let total_seqs: usize = CORPUS.iter().map(|p| p.len()).sum();
    println!("[CORPUS] {} phases, {} sequences", CORPUS.len(), total_seqs);

    // Eval BEFORE training
    let t1 = Instant::now();
    let mut total_loss = 0.0f32;
    let mut count = 0;
    for phase in CORPUS.iter() {
        // 1 sample per phase for quick eval
        let text = phase[0];
        let tokens: Vec<u8> = text.bytes().collect();
        if tokens.len() >= 2 {
            total_loss += compute_loss(&model, &tokens);
            count += 1;
        }
    }
    let loss_before = total_loss / count as f32;
    println!("\n[EVAL BEFORE] Loss: {:.4} ({} samples in {:.1}ms)", loss_before, count, t1.elapsed().as_millis());

    // PRETRAIN — Curriculum progressive: phase by phase, 15 epochs total
    let mut optimizer = AdamState::new(params);
    let batch_size = 4;
    let curriculum_epochs = [3, 2, 2, 2, 1, 1, 1, 1, 1, 1]; // epochs per phase in curriculum
    let consolidation_epochs = 20; // full-corpus epochs after curriculum

    println!("\n[PRETRAIN] Curriculum: {} phases then {} consolidation epochs (batch={})",
        CORPUS.len(), consolidation_epochs, batch_size);
    let t2 = Instant::now();
    let mut global_step = 0u64;

    // === Phase 1: Curriculum (phase-by-phase) ===
    println!("\n--- CURRICULUM PHASE ---");
    for (phase_idx, phase) in CORPUS.iter().enumerate() {
        let n_ep = if phase_idx < curriculum_epochs.len() { curriculum_epochs[phase_idx] } else { 1 };
        for ep in 0..n_ep {
            let mut phase_loss = 0.0f32;
            let mut phase_count = 0;
            let mut accum_grads = ModelGrads::new();
            let mut batch_count = 0;

            for text in phase.iter() {
                let tokens: Vec<u8> = text.bytes().collect();
                if tokens.len() < 2 { continue; }
                let (loss, grads) = forward_backward(&model, &tokens);
                phase_loss += loss;
                phase_count += 1;
                accum_grads.accumulate(&grads);
                batch_count += 1;
                if batch_count >= batch_size {
                    accum_grads.scale(1.0 / batch_count as f32);
                    accum_grads.clip_norm(1.0);
                    optimizer.step(&mut model, &accum_grads);
                    accum_grads.zero();
                    batch_count = 0;
                    global_step += 1;
                }
            }
            if batch_count > 0 {
                accum_grads.scale(1.0 / batch_count as f32);
                accum_grads.clip_norm(1.0);
                optimizer.step(&mut model, &accum_grads);
                accum_grads.zero();
                global_step += 1;
            }
            let avg = phase_loss / phase_count.max(1) as f32;
            println!("  Phase {} ep {}/{}: loss={:.4} ({} seqs)", phase_idx, ep + 1, n_ep, avg, phase_count);
        }
    }
    println!("  Curriculum done: {:.1}s, {} steps", t2.elapsed().as_secs_f64(), global_step);

    // Mid-curriculum eval
    let mut mid_loss = 0.0f32;
    let mut mid_count = 0;
    for phase in CORPUS.iter() {
        let tokens: Vec<u8> = phase[0].bytes().collect();
        if tokens.len() >= 2 { mid_loss += compute_loss(&model, &tokens); mid_count += 1; }
    }
    println!("  [MID EVAL] Loss: {:.4}", mid_loss / mid_count.max(1) as f32);

    // === Phase 2: Consolidation (full corpus) with cosine LR ===
    println!("\n--- CONSOLIDATION PHASE (cosine LR) ---");
    let base_lr = 0.001f32;
    for epoch in 0..consolidation_epochs {
        // Cosine decay: lr = base_lr * 0.5 * (1 + cos(pi * t / T))
        let progress = epoch as f32 / consolidation_epochs as f32;
        let lr = base_lr * 0.5 * (1.0 + (std::f32::consts::PI * progress).cos());
        optimizer.lr = lr;

        let mut epoch_loss = 0.0f32;
        let mut epoch_count = 0;
        let mut accum_grads = ModelGrads::new();
        let mut batch_count = 0;

        for phase in CORPUS.iter() {
            for text in phase.iter() {
                let tokens: Vec<u8> = text.bytes().collect();
                if tokens.len() < 2 { continue; }
                let (loss, grads) = forward_backward(&model, &tokens);
                epoch_loss += loss;
                epoch_count += 1;
                accum_grads.accumulate(&grads);
                batch_count += 1;
                if batch_count >= batch_size {
                    accum_grads.scale(1.0 / batch_count as f32);
                    accum_grads.clip_norm(1.0);
                    optimizer.step(&mut model, &accum_grads);
                    accum_grads.zero();
                    batch_count = 0;
                    global_step += 1;
                }
            }
        }
        if batch_count > 0 {
            accum_grads.scale(1.0 / batch_count as f32);
            accum_grads.clip_norm(1.0);
            optimizer.step(&mut model, &accum_grads);
            accum_grads.zero();
            global_step += 1;
        }
        let avg = epoch_loss / epoch_count.max(1) as f32;
        println!("  Consolidation {}/{}: avg_loss={:.4} ({} seqs, {:.1}s)",
            epoch + 1, consolidation_epochs, avg, epoch_count, t2.elapsed().as_secs_f64());
    }

    println!("[PRETRAIN] Total: {:.1}s, {} optimizer steps", t2.elapsed().as_secs_f64(), global_step);

    // Eval AFTER training
    let t3 = Instant::now();
    let mut total_loss = 0.0f32;
    let mut count = 0;
    for phase in CORPUS.iter() {
        let text = phase[0];
        let tokens: Vec<u8> = text.bytes().collect();
        if tokens.len() >= 2 {
            total_loss += compute_loss(&model, &tokens);
            count += 1;
        }
    }
    let loss_after = total_loss / count as f32;
    println!("\n[EVAL AFTER] Loss: {:.4} ({} samples in {:.1}ms)", loss_after, count, t3.elapsed().as_millis());

    // Chat tests — greedy
    println!("\n[CHAT TESTS - GREEDY]");
    for prompt in &["Hello", "who are you", "What is TrustOS"] {
        let gen = generate(&model, prompt.as_bytes(), 50);
        let response = String::from_utf8_lossy(&gen);
        println!("  > {} => {}", prompt, response.chars().take(60).collect::<String>());
    }

    // Chat tests — temperature 0.8, top-k 40
    println!("\n[CHAT TESTS - TEMP=0.8 TOP_K=40]");
    for prompt in &["Hello", "who are you", "What is TrustOS", "Q: aide", "bonjour", "I am Jarvis"] {
        let gen = generate_with_temp(&model, prompt.as_bytes(), 60, 0.8, 40);
        let response = String::from_utf8_lossy(&gen);
        println!("  > {} => {}", prompt, response.chars().take(80).collect::<String>());
    }

    // MATURITY
    println!("\n==========================================");
    println!("  LOSS BEFORE: {:.4}", loss_before);
    println!("  LOSS AFTER:  {:.4}", loss_after);
    println!("  IMPROVEMENT: {:.1}%", (1.0 - loss_after / loss_before) * 100.0);
    let maturity = if loss_after < 2.0 { "ADULT (Level 3) - READY FOR FULL PROPAGATION"
    } else if loss_after < 3.5 { "TEEN (Level 2) - READY FOR MESH + FEDERATED"
    } else if loss_after < 5.0 { "CHILD (Level 1) - NEEDS MORE TRAINING"
    } else { "INFANT (Level 0) - UNTRAINED" };
    println!("  MATURITY: {}", maturity);
    println!("==========================================");

    // === EXPORT WEIGHTS ===
    println!("\n[EXPORT] Serializing trained weights...");
    let floats = serialize_model(&model);
    let bytes: Vec<u8> = floats.iter().flat_map(|f| f.to_le_bytes()).collect();
    let out_path = "jarvis_pretrained.bin";
    std::fs::write(out_path, &bytes).expect("Failed to write weights");
    println!("[EXPORT] Saved {} floats ({:.1} MB) to {}", floats.len(), bytes.len() as f64 / 1_048_576.0, out_path);
    println!("[EXPORT] Copy this file to kernel/src/jarvis/ for embedding.");
}

/// Serialize model weights in the same order as kernel's TransformerWeights::serialize()
fn serialize_model(model: &TransformerWeights) -> Vec<f32> {
    let mut data = Vec::with_capacity(model.param_count());
    data.extend_from_slice(&model.token_embed);
    data.extend_from_slice(&model.pos_embed);
    for layer in &model.layers {
        data.extend_from_slice(&layer.rms_attn);
        data.extend_from_slice(&layer.w_q);
        data.extend_from_slice(&layer.w_k);
        data.extend_from_slice(&layer.w_v);
        data.extend_from_slice(&layer.w_o);
        data.extend_from_slice(&layer.rms_ffn);
        data.extend_from_slice(&layer.w_gate);
        data.extend_from_slice(&layer.w_up);
        data.extend_from_slice(&layer.w_down);
    }
    data.extend_from_slice(&model.rms_final);
    data.extend_from_slice(&model.w_output);
    data
}
