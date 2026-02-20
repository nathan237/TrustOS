//! Inference Engine — Forward pass through the tiny transformer
//!
//! Implements autoregressive text generation:
//! 1. Embed input tokens
//! 2. Pass through N transformer layers
//! 3. Project to vocabulary logits
//! 4. Sample next token (temperature + top-k)
//! 5. Repeat until max_tokens or EOS
//!
//! Uses crate::drivers::amdgpu::neural for GEMM and activations
//! when GPU is available, falls back to direct matrix ops otherwise.

use alloc::vec::Vec;
use alloc::vec;
use super::model::*;
use super::tokenizer;

// ═══════════════════════════════════════════════════════════════════════════════
// Inference State
// ═══════════════════════════════════════════════════════════════════════════════

/// Key-Value cache for autoregressive generation
/// Stores K and V projections from previous positions to avoid recomputation
pub struct KVCache {
    /// Per-layer key cache: layer → [positions_so_far × D_MODEL]
    k: Vec<Vec<f32>>,
    /// Per-layer value cache: layer → [positions_so_far × D_MODEL]
    v: Vec<Vec<f32>>,
    /// Current position count
    pub len: usize,
}

impl KVCache {
    pub fn new() -> Self {
        KVCache {
            k: (0..N_LAYERS).map(|_| Vec::with_capacity(MAX_SEQ * D_MODEL)).collect(),
            v: (0..N_LAYERS).map(|_| Vec::with_capacity(MAX_SEQ * D_MODEL)).collect(),
            len: 0,
        }
    }

    pub fn clear(&mut self) {
        for layer_k in &mut self.k { layer_k.clear(); }
        for layer_v in &mut self.v { layer_v.clear(); }
        self.len = 0;
    }
}

/// Inference configuration
pub struct InferenceConfig {
    /// Temperature for sampling (0.0 = greedy, 1.0 = normal, >1 = creative)
    pub temperature: f32,
    /// Top-k sampling (0 = disabled, >0 = only top k tokens considered)
    pub top_k: usize,
    /// Maximum generation length
    pub max_tokens: usize,
}

impl Default for InferenceConfig {
    fn default() -> Self {
        InferenceConfig {
            temperature: 0.8,
            top_k: 40,
            max_tokens: 128,
        }
    }
}

/// The inference engine holds mutable state (caches, buffers)
pub struct InferenceEngine {
    /// KV cache for autoregressive generation
    kv_cache: KVCache,
    /// Configuration
    pub config: InferenceConfig,
    /// Scratch buffers (reused across calls)
    buf_x: Vec<f32>,      // [D_MODEL] — current hidden state
    buf_xn: Vec<f32>,     // [D_MODEL] — normalized hidden state
    buf_q: Vec<f32>,      // [D_MODEL]
    buf_k: Vec<f32>,      // [D_MODEL]
    buf_v: Vec<f32>,      // [D_MODEL]
    buf_attn: Vec<f32>,   // [MAX_SEQ] — attention scores
    buf_gate: Vec<f32>,   // [D_FF]
    buf_up: Vec<f32>,     // [D_FF]
    buf_logits: Vec<f32>, // [VOCAB_SIZE]
    /// PRNG state for sampling
    rng_state: u64,
}

impl InferenceEngine {
    pub fn new() -> Self {
        InferenceEngine {
            kv_cache: KVCache::new(),
            config: InferenceConfig::default(),
            buf_x: vec![0.0; D_MODEL],
            buf_xn: vec![0.0; D_MODEL],
            buf_q: vec![0.0; D_MODEL],
            buf_k: vec![0.0; D_MODEL],
            buf_v: vec![0.0; D_MODEL],
            buf_attn: vec![0.0; MAX_SEQ],
            buf_gate: vec![0.0; D_FF],
            buf_up: vec![0.0; D_FF],
            buf_logits: vec![0.0; VOCAB_SIZE],
            rng_state: crate::time::uptime_ticks().wrapping_add(0xDEAD_BEEF),
        }
    }

    /// Generate tokens autoregressively from a prompt
    pub fn generate(&mut self, model: &TransformerWeights, prompt: &[u8], max_tokens: usize) -> Vec<u8> {
        self.kv_cache.clear();
        let max = max_tokens.min(MAX_SEQ);
        let mut output = Vec::with_capacity(max);

        // Process prompt tokens (prefill)
        for &token in prompt.iter().take(MAX_SEQ - 1) {
            self.forward_one(model, token);
        }

        // Generate new tokens
        let mut next_token = if !prompt.is_empty() {
            self.sample_token()
        } else {
            tokenizer::BOS_TOKEN
        };

        for _ in 0..max {
            if next_token == tokenizer::EOS_TOKEN { break; }
            output.push(next_token);

            self.forward_one(model, next_token);
            next_token = self.sample_token();
        }

        output
    }

    /// Predict the most likely next token given context
    pub fn predict_next_token(&mut self, model: &TransformerWeights, context: &[u8]) -> u8 {
        self.kv_cache.clear();
        for &token in context.iter().take(MAX_SEQ) {
            self.forward_one(model, token);
        }
        // Greedy: take argmax
        argmax(&self.buf_logits)
    }

    /// Forward pass for a single token at the current position
    fn forward_one(&mut self, model: &TransformerWeights, token: u8) {
        let pos = self.kv_cache.len;
        if pos >= MAX_SEQ { return; }

        // ── Token + Position Embedding ──────────────────────────────────
        let tok_idx = token as usize;
        for i in 0..D_MODEL {
            self.buf_x[i] = model.token_embed[tok_idx * D_MODEL + i]
                           + model.pos_embed[pos * D_MODEL + i];
        }

        // ── Transformer Layers ──────────────────────────────────────────
        for layer_idx in 0..N_LAYERS {
            let layer = &model.layers[layer_idx];

            // ── Pre-attention RMSNorm ──
            rmsnorm(&mut self.buf_xn, &self.buf_x, &layer.rms_attn);

            // ── QKV Projections ── (matmul: [1 × D_MODEL] × [D_MODEL × D_MODEL])
            matvec(&mut self.buf_q, &layer.w_q, &self.buf_xn, D_MODEL, D_MODEL);
            matvec(&mut self.buf_k, &layer.w_k, &self.buf_xn, D_MODEL, D_MODEL);
            matvec(&mut self.buf_v, &layer.w_v, &self.buf_xn, D_MODEL, D_MODEL);

            // ── Store K,V in cache ──
            self.kv_cache.k[layer_idx].extend_from_slice(&self.buf_k);
            self.kv_cache.v[layer_idx].extend_from_slice(&self.buf_v);

            // ── Multi-head Attention ──
            // For each head: score = Q_h · K_h^T / sqrt(d_k), then softmax, then weighted V_h
            let n_pos = pos + 1; // Number of positions including current
            let mut attn_out = vec![0.0f32; D_MODEL];

            for h in 0..N_HEADS {
                let head_offset = h * D_K;

                // Compute attention scores for this head
                for t in 0..n_pos {
                    let mut score = 0.0f32;
                    for d in 0..D_K {
                        score += self.buf_q[head_offset + d]
                               * self.kv_cache.k[layer_idx][t * D_MODEL + head_offset + d];
                    }
                    self.buf_attn[t] = score / (D_K as f32).sqrt_approx();
                }

                // Causal masking is implicit: we only have positions 0..pos+1

                // Softmax over positions
                softmax_slice(&mut self.buf_attn[..n_pos]);

                // Weighted sum of values
                for t in 0..n_pos {
                    let w = self.buf_attn[t];
                    for d in 0..D_K {
                        attn_out[head_offset + d] +=
                            w * self.kv_cache.v[layer_idx][t * D_MODEL + head_offset + d];
                    }
                }
            }

            // ── Output Projection ──
            let mut proj_out = vec![0.0f32; D_MODEL];
            matvec(&mut proj_out, &layer.w_o, &attn_out, D_MODEL, D_MODEL);

            // ── Residual ──
            for i in 0..D_MODEL {
                self.buf_x[i] += proj_out[i];
            }

            // ── Pre-FFN RMSNorm ──
            rmsnorm(&mut self.buf_xn, &self.buf_x, &layer.rms_ffn);

            // ── SwiGLU FFN ──
            // gate = SiLU(x @ W_gate)
            matvec(&mut self.buf_gate, &layer.w_gate, &self.buf_xn, D_MODEL, D_FF);
            // up = x @ W_up
            matvec(&mut self.buf_up, &layer.w_up, &self.buf_xn, D_MODEL, D_FF);
            // gate = SiLU(gate) * up
            for i in 0..D_FF {
                let g = self.buf_gate[i];
                let sig = 1.0 / (1.0 + (-g).exp_approx());
                self.buf_gate[i] = g * sig * self.buf_up[i];
            }
            // down = gate @ W_down
            let mut ffn_out = vec![0.0f32; D_MODEL];
            matvec(&mut ffn_out, &layer.w_down, &self.buf_gate, D_FF, D_MODEL);

            // ── Residual ──
            for i in 0..D_MODEL {
                self.buf_x[i] += ffn_out[i];
            }
        }

        // ── Final RMSNorm ───────────────────────────────────────────────
        rmsnorm(&mut self.buf_xn, &self.buf_x, &model.rms_final);

        // ── Output Logits ── (project to vocabulary)
        matvec(&mut self.buf_logits, &model.w_output, &self.buf_xn, D_MODEL, VOCAB_SIZE);

        self.kv_cache.len = pos + 1;
    }

    /// Sample a token from the logit distribution
    fn sample_token(&mut self) -> u8 {
        let temp = self.config.temperature;

        if temp <= 0.01 {
            // Greedy decoding
            return argmax(&self.buf_logits);
        }

        // Apply temperature
        let mut logits = self.buf_logits.clone();
        for l in logits.iter_mut() {
            *l /= temp;
        }

        // Top-k filtering
        if self.config.top_k > 0 && self.config.top_k < VOCAB_SIZE {
            let mut indices: Vec<(f32, usize)> = logits.iter().copied()
                .enumerate().map(|(i, v)| (v, i)).collect();
            indices.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(core::cmp::Ordering::Equal));
            let threshold = indices[self.config.top_k.min(indices.len() - 1)].0;
            for l in logits.iter_mut() {
                if *l < threshold { *l = f32::NEG_INFINITY; }
            }
        }

        // Softmax
        softmax_slice(&mut logits);

        // Sample from distribution
        let r = self.rand_f32();
        let mut cum = 0.0f32;
        for (i, &p) in logits.iter().enumerate() {
            cum += p;
            if cum >= r {
                return i as u8;
            }
        }
        (VOCAB_SIZE - 1) as u8
    }

    /// Random f32 in [0, 1)
    fn rand_f32(&mut self) -> f32 {
        let mut x = self.rng_state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.rng_state = x;
        ((x >> 40) as u32 as f32) / ((1u32 << 24) as f32)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Math Primitives (optimized for single-vector operations)
// ═══════════════════════════════════════════════════════════════════════════════

/// Matrix-vector multiply: out = W × x
/// W is [rows × cols] stored row-major, x is [cols], out is [rows]
fn matvec(out: &mut [f32], w: &[f32], x: &[f32], cols: usize, rows: usize) {
    for r in 0..rows {
        let mut sum = 0.0f32;
        let row_start = r * cols;
        for c in 0..cols {
            sum += w[row_start + c] * x[c];
        }
        out[r] = sum;
    }
}

/// RMSNorm: out = (x / RMS(x)) * weight
fn rmsnorm(out: &mut [f32], x: &[f32], weight: &[f32]) {
    let n = x.len();
    let mut ss = 0.0f32;
    for &v in x { ss += v * v; }
    let rms = (ss / n as f32 + RMS_EPS).sqrt_approx();
    let inv_rms = 1.0 / rms;
    for i in 0..n {
        out[i] = x[i] * inv_rms * weight[i];
    }
}

/// Softmax in-place over a slice (numerically stable)
fn softmax_slice(data: &mut [f32]) {
    if data.is_empty() { return; }
    let mut max = data[0];
    for &v in data.iter() { if v > max { max = v; } }
    let mut sum = 0.0f32;
    for v in data.iter_mut() {
        *v = (*v - max).exp_approx();
        sum += *v;
    }
    if sum > 0.0 {
        let inv = 1.0 / sum;
        for v in data.iter_mut() { *v *= inv; }
    }
}

/// Argmax of a slice → index (as u8)
fn argmax(data: &[f32]) -> u8 {
    let mut best_i = 0u8;
    let mut best_v = f32::NEG_INFINITY;
    for (i, &v) in data.iter().enumerate() {
        if v > best_v {
            best_v = v;
            best_i = i as u8;
        }
    }
    best_i
}

// ═══════════════════════════════════════════════════════════════════════════════
// Approximate Math (duplicated from neural.rs for module independence)
// ═══════════════════════════════════════════════════════════════════════════════

trait ApproxMath {
    fn exp_approx(self) -> f32;
    fn sqrt_approx(self) -> f32;
}

impl ApproxMath for f32 {
    fn exp_approx(self) -> f32 {
        if self > 88.0 { return f32::MAX; }
        if self < -88.0 { return 0.0; }
        let a = (1 << 23) as f32 / core::f32::consts::LN_2;
        let b = (1 << 23) as f32 * (127.0 - 0.04368);
        let bits = ((a * self + b) as i32).max(0) as u32;
        f32::from_bits(bits)
    }

    fn sqrt_approx(self) -> f32 {
        if self <= 0.0 { return 0.0; }
        let bits = self.to_bits();
        let guess = f32::from_bits((bits >> 1) + 0x1FBD_1DF5);
        (guess + self / guess) * 0.5
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Diagnostic: compute loss on a sequence (for training.rs)
// ═══════════════════════════════════════════════════════════════════════════════

/// Compute cross-entropy loss on a token sequence (teacher forcing)
/// Returns average loss over tokens and the per-position logits
pub fn compute_loss(model: &TransformerWeights, tokens: &[u8]) -> (f32, Vec<Vec<f32>>) {
    let mut engine = InferenceEngine::new();
    engine.config.temperature = 0.0; // Doesn't matter for loss

    let mut total_loss = 0.0f32;
    let mut all_logits = Vec::new();

    for t in 0..tokens.len().saturating_sub(1) {
        engine.forward_one(model, tokens[t]);

        // Log-softmax for cross-entropy
        let target = tokens[t + 1] as usize;
        let mut logits = engine.buf_logits.clone();

        // Numerically stable log-softmax
        let max = logits.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        let mut sum_exp = 0.0f32;
        for l in logits.iter_mut() {
            *l = (*l - max).exp_approx();
            sum_exp += *l;
        }
        let log_sum = max + sum_exp.ln_approx();

        let loss = -(engine.buf_logits[target] - log_sum);
        total_loss += loss;

        all_logits.push(engine.buf_logits.clone());
    }

    let n = (tokens.len() - 1).max(1);
    (total_loss / n as f32, all_logits)
}

trait LnApprox {
    fn ln_approx(self) -> f32;
}
impl LnApprox for f32 {
    fn ln_approx(self) -> f32 {
        if self <= 0.0 { return -88.0; }
        let bits = self.to_bits();
        let e = ((bits >> 23) & 0xFF) as f32 - 127.0;
        let m = f32::from_bits((bits & 0x007FFFFF) | 0x3F800000);
        // ln(x) = (e + ln(m)) * ln(2)
        // ln(m) ≈ m - 1 for m ∈ [1, 2)
        (e + (m - 1.0) * 1.4427) * core::f32::consts::LN_2
    }
}
