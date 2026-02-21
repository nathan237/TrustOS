//! Transformer Model — Weight storage & architecture definition
//!
//! Implements a GPT/LLaMA-style decoder-only transformer.
//! All weights are stored as flat `Vec<f32>` arrays on the heap.
//!
//! Architecture (LLaMA-2 style):
//! - Pre-norm (RMSNorm before each sublayer)
//! - Rotary Position Embeddings (RoPE) — simplified as learned positional
//! - SwiGLU FFN (gated feed-forward with SiLU activation)
//! - Multi-head attention with causal mask
//!
//! Memory layout:
//! ```text
//! token_embed:  [VOCAB_SIZE × D_MODEL]     = [256 × 128] = 128 KB
//! pos_embed:    [MAX_SEQ × D_MODEL]         = [256 × 128] = 128 KB
//! layers[0..3]: each ~1 MB (see LayerWeights)              = 4 MB
//! rms_final:    [D_MODEL]                    = [128]       = 512 B
//! w_output:     [D_MODEL × VOCAB_SIZE]       = [128 × 256] = 128 KB
//! ─────────────────────────────────────────────────────────────────
//! Total: ~1.15M params → ~4.5 MB at FP32
//! ```

use alloc::vec::Vec;
use alloc::vec;

// ═══════════════════════════════════════════════════════════════════════════════
// Model Hyperparameters
// ═══════════════════════════════════════════════════════════════════════════════

/// Vocabulary size (byte-level: 256 possible byte values)
pub const VOCAB_SIZE: usize = 256;

/// Model embedding dimension
pub const D_MODEL: usize = 128;

/// Number of attention heads
pub const N_HEADS: usize = 4;

/// Key/Query dimension per head
pub const D_K: usize = D_MODEL / N_HEADS; // = 32

/// Feed-forward inner dimension (4× d_model for SwiGLU)
pub const D_FF: usize = 512;

/// Number of transformer layers
pub const N_LAYERS: usize = 4;

/// Maximum sequence length (context window)
pub const MAX_SEQ: usize = 256;

/// RMSNorm epsilon
pub const RMS_EPS: f32 = 1e-5;

/// Fast approximate square root (no_std compatible)
/// Uses bit manipulation + one Newton-Raphson iteration
fn approx_sqrt(x: f32) -> f32 {
    if x <= 0.0 { return 0.0; }
    let bits = x.to_bits();
    let guess = f32::from_bits((bits >> 1) + 0x1FBD_1DF5);
    (guess + x / guess) * 0.5
}

// ═══════════════════════════════════════════════════════════════════════════════
// Weight Structures
// ═══════════════════════════════════════════════════════════════════════════════

/// Weights for a single transformer layer
pub struct LayerWeights {
    /// RMSNorm weights for pre-attention normalization [D_MODEL]
    pub rms_attn: Vec<f32>,
    /// Query projection [D_MODEL × D_MODEL]
    pub w_q: Vec<f32>,
    /// Key projection [D_MODEL × D_MODEL]
    pub w_k: Vec<f32>,
    /// Value projection [D_MODEL × D_MODEL]
    pub w_v: Vec<f32>,
    /// Output projection [D_MODEL × D_MODEL]
    pub w_o: Vec<f32>,
    /// RMSNorm weights for pre-FFN normalization [D_MODEL]
    pub rms_ffn: Vec<f32>,
    /// FFN gate projection (SwiGLU) [D_MODEL × D_FF]
    pub w_gate: Vec<f32>,
    /// FFN up projection [D_MODEL × D_FF]
    pub w_up: Vec<f32>,
    /// FFN down projection [D_FF × D_MODEL]
    pub w_down: Vec<f32>,
}

impl LayerWeights {
    /// Create a new layer with Xavier-initialized random weights
    fn new_random(seed: &mut u64) -> Self {
        let attn_scale = 1.0 / approx_sqrt(D_MODEL as f32);
        let ffn_scale = 1.0 / approx_sqrt(D_FF as f32);

        LayerWeights {
            rms_attn: vec![1.0f32; D_MODEL],
            w_q: random_vec(D_MODEL * D_MODEL, attn_scale, seed),
            w_k: random_vec(D_MODEL * D_MODEL, attn_scale, seed),
            w_v: random_vec(D_MODEL * D_MODEL, attn_scale, seed),
            w_o: random_vec(D_MODEL * D_MODEL, attn_scale, seed),
            rms_ffn: vec![1.0f32; D_MODEL],
            w_gate: random_vec(D_MODEL * D_FF, ffn_scale, seed),
            w_up: random_vec(D_MODEL * D_FF, ffn_scale, seed),
            w_down: random_vec(D_FF * D_MODEL, ffn_scale, seed),
        }
    }

    /// Parameter count for one layer
    pub fn param_count(&self) -> usize {
        D_MODEL  // rms_attn
        + D_MODEL * D_MODEL * 4  // w_q, w_k, w_v, w_o
        + D_MODEL  // rms_ffn
        + D_MODEL * D_FF * 2  // w_gate, w_up
        + D_FF * D_MODEL  // w_down
    }
}

/// Complete transformer model weights
pub struct TransformerWeights {
    /// Token embedding matrix [VOCAB_SIZE × D_MODEL]
    pub token_embed: Vec<f32>,
    /// Learned positional embedding [MAX_SEQ × D_MODEL]
    pub pos_embed: Vec<f32>,
    /// Per-layer weights
    pub layers: Vec<LayerWeights>,
    /// Final RMSNorm [D_MODEL]
    pub rms_final: Vec<f32>,
    /// Output projection (logits) [D_MODEL × VOCAB_SIZE]
    pub w_output: Vec<f32>,
}

impl TransformerWeights {
    /// Create a new model with random (Xavier) initialization
    pub fn new_random() -> Self {
        let mut seed = 42_u64; // Deterministic seed for reproducibility

        let embed_scale = 1.0 / approx_sqrt(D_MODEL as f32);

        let mut layers = Vec::with_capacity(N_LAYERS);
        for _ in 0..N_LAYERS {
            layers.push(LayerWeights::new_random(&mut seed));
        }

        TransformerWeights {
            token_embed: random_vec(VOCAB_SIZE * D_MODEL, embed_scale, &mut seed),
            pos_embed: random_vec(MAX_SEQ * D_MODEL, 0.02, &mut seed),
            layers,
            rms_final: vec![1.0f32; D_MODEL],
            w_output: random_vec(D_MODEL * VOCAB_SIZE, embed_scale, &mut seed),
        }
    }

    /// Total parameter count
    pub fn param_count(&self) -> usize {
        VOCAB_SIZE * D_MODEL           // token_embed
        + MAX_SEQ * D_MODEL            // pos_embed
        + self.layers.iter().map(|l| l.param_count()).sum::<usize>()
        + D_MODEL                       // rms_final
        + D_MODEL * VOCAB_SIZE         // w_output
    }

    /// Total memory in bytes (FP32)
    pub fn memory_bytes(&self) -> usize {
        self.param_count() * 4
    }

    /// Get a flat snapshot of all weights (for serialization)
    pub fn serialize(&self) -> Vec<f32> {
        let mut data = Vec::with_capacity(self.param_count());
        data.extend_from_slice(&self.token_embed);
        data.extend_from_slice(&self.pos_embed);
        for layer in &self.layers {
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
        data.extend_from_slice(&self.rms_final);
        data.extend_from_slice(&self.w_output);
        data
    }

    /// Load weights from a flat array (deserialization)
    pub fn deserialize(data: &[f32]) -> Option<Self> {
        let mut pos = 0;

        let token_embed = slice_vec(data, &mut pos, VOCAB_SIZE * D_MODEL)?;
        let pos_embed = slice_vec(data, &mut pos, MAX_SEQ * D_MODEL)?;

        let mut layers = Vec::with_capacity(N_LAYERS);
        for _ in 0..N_LAYERS {
            let rms_attn = slice_vec(data, &mut pos, D_MODEL)?;
            let w_q = slice_vec(data, &mut pos, D_MODEL * D_MODEL)?;
            let w_k = slice_vec(data, &mut pos, D_MODEL * D_MODEL)?;
            let w_v = slice_vec(data, &mut pos, D_MODEL * D_MODEL)?;
            let w_o = slice_vec(data, &mut pos, D_MODEL * D_MODEL)?;
            let rms_ffn = slice_vec(data, &mut pos, D_MODEL)?;
            let w_gate = slice_vec(data, &mut pos, D_MODEL * D_FF)?;
            let w_up = slice_vec(data, &mut pos, D_MODEL * D_FF)?;
            let w_down = slice_vec(data, &mut pos, D_FF * D_MODEL)?;

            layers.push(LayerWeights {
                rms_attn, w_q, w_k, w_v, w_o, rms_ffn, w_gate, w_up, w_down,
            });
        }

        let rms_final = slice_vec(data, &mut pos, D_MODEL)?;
        let w_output = slice_vec(data, &mut pos, D_MODEL * VOCAB_SIZE)?;

        Some(TransformerWeights {
            token_embed, pos_embed, layers, rms_final, w_output,
        })
    }

    /// Reset all weights to random (reinitialize)
    pub fn reset(&mut self) {
        let fresh = Self::new_random();
        *self = fresh;
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Utility Functions
// ═══════════════════════════════════════════════════════════════════════════════

/// Extract a slice from data and advance position
fn slice_vec(data: &[f32], pos: &mut usize, len: usize) -> Option<Vec<f32>> {
    if *pos + len > data.len() { return None; }
    let v = data[*pos..*pos + len].to_vec();
    *pos += len;
    Some(v)
}

/// Generate a Vec of random f32 values using xorshift64
fn random_vec(len: usize, scale: f32, seed: &mut u64) -> Vec<f32> {
    let mut v = Vec::with_capacity(len);
    for _ in 0..len {
        v.push(xorshift_f32(seed) * scale);
    }
    v
}

/// Xorshift64 PRNG → f32 in [-1, 1]
fn xorshift_f32(state: &mut u64) -> f32 {
    let mut x = *state;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    *state = x;
    // Convert to [-1, 1]
    let bits = (x >> 40) as u32; // 24 bits
    (bits as f32 / (1u32 << 24) as f32) * 2.0 - 1.0
}
