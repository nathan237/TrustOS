//! Micro-JARVIS — Tiny kernel sentinel model
//!
//! A minimal transformer that lives permanently in the kernel binary.
//! Handles kernel validation, basic diagnostics, and orchestrates
//! loading the full JARVIS brain from the filesystem.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────┐
//! │         MICRO-JARVIS (kernel sentinel)       │
//! │  1 layer, d=64, 2 heads, ~50K params         │
//! │  Always available — instant boot              │
//! │                                               │
//! │  Roles:                                       │
//! │    • Kernel command validation                 │
//! │    • Basic diagnostics & status                │
//! │    • Bridge to full brain via FS               │
//! │    • Fallback when full brain unavailable      │
//! └──────────┬──────────────────────────────────┘
//!            │ load_full_brain()
//!            ▼
//! ┌─────────────────────────────────────────────┐
//! │         FULL JARVIS (filesystem brain)        │
//! │  4 layers, d=256, 4 heads, 4.4M params       │
//! │  Loaded from /jarvis/weights.bin              │
//! │                                               │
//! │  Roles:                                       │
//! │    • Chat, reasoning, learning                 │
//! │    • Federated training & mesh sync            │
//! │    • Full corpus understanding                 │
//! └─────────────────────────────────────────────┘
//! ```

use alloc::vec::Vec;
use alloc::vec;

// ═══════════════════════════════════════════════════════════════════════════════
// Micro-model hyperparameters
// ═══════════════════════════════════════════════════════════════════════════════

pub // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const MICRO_VOCAB: usize = 256;    // byte-level (same as full)
pub // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const MICRO_D_MODEL: usize = 64;
pub // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const MICRO_N_HEADS: usize = 2;
pub // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const MICRO_D_K: usize = MICRO_D_MODEL / MICRO_N_HEADS; // 32
pub // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const MICRO_D_FF: usize = 128;     // 2× d_model
pub // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const MICRO_N_LAYERS: usize = 1;
pub // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const MICRO_MAXIMUM_SEQUENCE: usize = 64;
pub // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const MICRO_RMS_EPS: f32 = 1e-5;

// ═══════════════════════════════════════════════════════════════════════════════
// Weight Structures
// ═══════════════════════════════════════════════════════════════════════════════

pub struct MicroLayerWeights {
    pub rms_attn: Vec<f32>,      // [MICRO_D_MODEL]
    pub w_q: Vec<f32>,           // [MICRO_D_MODEL × MICRO_D_MODEL]
    pub w_k: Vec<f32>,
    pub w_v: Vec<f32>,
    pub w_o: Vec<f32>,
    pub rms_ffn: Vec<f32>,       // [MICRO_D_MODEL]
    pub w_gate: Vec<f32>,        // [MICRO_D_MODEL × MICRO_D_FF]
    pub w_up: Vec<f32>,
    pub w_down: Vec<f32>,        // [MICRO_D_FF × MICRO_D_MODEL]
}

// Structure publique — visible à l'extérieur de ce module.
pub struct MicroWeights {
    pub token_embed: Vec<f32>,   // [MICRO_VOCAB × MICRO_D_MODEL]
    pub pos_embed: Vec<f32>,     // [MICRO_MAX_SEQ × MICRO_D_MODEL]
    pub layers: Vec<MicroLayerWeights>,
    pub rms_final: Vec<f32>,     // [MICRO_D_MODEL]
    pub w_output: Vec<f32>,      // [MICRO_D_MODEL × MICRO_VOCAB]
}

// Bloc d'implémentation — définit les méthodes du type ci-dessus.
impl MicroWeights {
        // Fonction publique — appelable depuis d'autres modules.
pub fn param_count(&self) -> usize {
        MICRO_VOCAB * MICRO_D_MODEL              // token_embed
        + MICRO_MAXIMUM_SEQUENCE * MICRO_D_MODEL          // pos_embed
        + MICRO_N_LAYERS * (
            MICRO_D_MODEL                        // rms_attn
            + MICRO_D_MODEL * MICRO_D_MODEL * 4  // Q,K,V,O
            + MICRO_D_MODEL                      // rms_ffn
            + MICRO_D_MODEL * MICRO_D_FF * 2     // gate, up
            + MICRO_D_FF * MICRO_D_MODEL         // down
        )
        + MICRO_D_MODEL                          // rms_final
        + MICRO_D_MODEL * MICRO_VOCAB            // w_output
    }

    /// Serialize to flat f32 array (same order as full model)
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

    /// Deserialize from flat f32 array
    pub fn deserialize(data: &[f32]) -> Option<Self> {
        let mut pos = 0;
        let token_embed = slice_vec(data, &mut pos, MICRO_VOCAB * MICRO_D_MODEL)?;
        let pos_embed = slice_vec(data, &mut pos, MICRO_MAXIMUM_SEQUENCE * MICRO_D_MODEL)?;

        let mut layers = Vec::with_capacity(MICRO_N_LAYERS);
        for _ in 0..MICRO_N_LAYERS {
            layers.push(MicroLayerWeights {
                rms_attn: slice_vec(data, &mut pos, MICRO_D_MODEL)?,
                w_q: slice_vec(data, &mut pos, MICRO_D_MODEL * MICRO_D_MODEL)?,
                w_k: slice_vec(data, &mut pos, MICRO_D_MODEL * MICRO_D_MODEL)?,
                w_v: slice_vec(data, &mut pos, MICRO_D_MODEL * MICRO_D_MODEL)?,
                w_o: slice_vec(data, &mut pos, MICRO_D_MODEL * MICRO_D_MODEL)?,
                rms_ffn: slice_vec(data, &mut pos, MICRO_D_MODEL)?,
                w_gate: slice_vec(data, &mut pos, MICRO_D_MODEL * MICRO_D_FF)?,
                w_up: slice_vec(data, &mut pos, MICRO_D_MODEL * MICRO_D_FF)?,
                w_down: slice_vec(data, &mut pos, MICRO_D_FF * MICRO_D_MODEL)?,
            });
        }

        let rms_final = slice_vec(data, &mut pos, MICRO_D_MODEL)?;
        let w_output = slice_vec(data, &mut pos, MICRO_D_MODEL * MICRO_VOCAB)?;

        Some(MicroWeights {
            token_embed, pos_embed, layers, rms_final, w_output,
        })
    }

    /// Random initialization (Xavier-like)
    pub fn new_random() -> Self {
        let mut seed = 77u64;
        let emb_scale = 1.0 / (MICRO_D_MODEL as f32).sqrt_approx();
        let ffn_scale = 1.0 / (MICRO_D_FF as f32).sqrt_approx();

        let mut layers = Vec::with_capacity(MICRO_N_LAYERS);
        for _ in 0..MICRO_N_LAYERS {
            layers.push(MicroLayerWeights {
                rms_attn: vec![1.0; MICRO_D_MODEL],
                w_q: random_vec(MICRO_D_MODEL * MICRO_D_MODEL, emb_scale, &mut seed),
                w_k: random_vec(MICRO_D_MODEL * MICRO_D_MODEL, emb_scale, &mut seed),
                w_v: random_vec(MICRO_D_MODEL * MICRO_D_MODEL, emb_scale, &mut seed),
                w_o: random_vec(MICRO_D_MODEL * MICRO_D_MODEL, emb_scale, &mut seed),
                rms_ffn: vec![1.0; MICRO_D_MODEL],
                w_gate: random_vec(MICRO_D_MODEL * MICRO_D_FF, ffn_scale, &mut seed),
                w_up: random_vec(MICRO_D_MODEL * MICRO_D_FF, ffn_scale, &mut seed),
                w_down: random_vec(MICRO_D_FF * MICRO_D_MODEL, ffn_scale, &mut seed),
            });
        }

        MicroWeights {
            token_embed: random_vec(MICRO_VOCAB * MICRO_D_MODEL, emb_scale, &mut seed),
            pos_embed: random_vec(MICRO_MAXIMUM_SEQUENCE * MICRO_D_MODEL, 0.02, &mut seed),
            layers,
            rms_final: vec![1.0; MICRO_D_MODEL],
            w_output: random_vec(MICRO_D_MODEL * MICRO_VOCAB, emb_scale, &mut seed),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Micro Inference Engine
// ═══════════════════════════════════════════════════════════════════════════════

pub struct MicroEngine {
    /// KV cache: keys[layer][pos * MICRO_D_MODEL .. (pos+1) * MICRO_D_MODEL]
    cache_k: Vec<Vec<f32>>,
    cache_v: Vec<Vec<f32>>,
    cache_len: usize,
    /// Scratch buffers
    buf_x: Vec<f32>,
    buf_xn: Vec<f32>,
    buf_q: Vec<f32>,
    buf_k: Vec<f32>,
    buf_v: Vec<f32>,
    buf_attn: Vec<f32>,
    buf_gate: Vec<f32>,
    buf_up: Vec<f32>,
    buf_logits: Vec<f32>,
    rng: u64,
}

// Bloc d'implémentation — définit les méthodes du type ci-dessus.
impl MicroEngine {
        // Fonction publique — appelable depuis d'autres modules.
pub fn new() -> Self {
        MicroEngine {
            cache_k: (0..MICRO_N_LAYERS).map(|_| Vec::with_capacity(MICRO_MAXIMUM_SEQUENCE * MICRO_D_MODEL)).collect(),
            cache_v: (0..MICRO_N_LAYERS).map(|_| Vec::with_capacity(MICRO_MAXIMUM_SEQUENCE * MICRO_D_MODEL)).collect(),
            cache_len: 0,
            buf_x: vec![0.0; MICRO_D_MODEL],
            buf_xn: vec![0.0; MICRO_D_MODEL],
            buf_q: vec![0.0; MICRO_D_MODEL],
            buf_k: vec![0.0; MICRO_D_MODEL],
            buf_v: vec![0.0; MICRO_D_MODEL],
            buf_attn: vec![0.0; MICRO_MAXIMUM_SEQUENCE],
            buf_gate: vec![0.0; MICRO_D_FF],
            buf_up: vec![0.0; MICRO_D_FF],
            buf_logits: vec![0.0; MICRO_VOCAB],
            rng: crate::time::uptime_ticks().wrapping_add(0xBEEF_CAFE),
        }
    }

    /// Generate text from prompt
    pub fn generate(&mut self, model: &MicroWeights, prompt: &[u8], maximum_tokens: usize) -> Vec<u8> {
        self.clear_cache();
        let max = maximum_tokens.min(MICRO_MAXIMUM_SEQUENCE);
        let mut output = Vec::with_capacity(max);

        for &token in prompt.iter().take(MICRO_MAXIMUM_SEQUENCE - 1) {
            self.forward_one(model, token);
        }

        let mut next = self.sample(0.7, 20, &output);
        for _ in 0..max {
            if next == 0 || next == 3 { break; }
            output.push(next);
            self.forward_one(model, next);
            next = self.sample(0.7, 20, &output);
        }
        output
    }

    /// Classify: run forward pass, return argmax logit index
    pub fn classify(&mut self, model: &MicroWeights, input: &[u8]) -> u8 {
        self.clear_cache();
        for &token in input.iter().take(MICRO_MAXIMUM_SEQUENCE) {
            self.forward_one(model, token);
        }
        argmax(&self.buf_logits)
    }

    /// Compute loss on a token sequence (teacher forcing)
    pub fn compute_loss(&mut self, model: &MicroWeights, tokens: &[u8]) -> f32 {
        self.clear_cache();
        let sequence_length = tokens.len().min(MICRO_MAXIMUM_SEQUENCE);
        if sequence_length < 2 { return f32::MAX; }

        let mut total_loss = 0.0f32;
        let n_targets = sequence_length - 1;

        for t in 0..sequence_length {
            self.forward_one(model, tokens[t]);
            if t < n_targets {
                let target = tokens[t + 1] as usize;
                let mut probs = self.buf_logits.clone();
                softmax(&mut probs);
                let p = probs[target].max(1e-10);
                total_loss += -p.ln_approx();
            }
        }
        total_loss / n_targets as f32
    }

    fn clear_cache(&mut self) {
        for k in &mut self.cache_k { k.clear(); }
        for v in &mut self.cache_v { v.clear(); }
        self.cache_len = 0;
    }

    fn forward_one(&mut self, model: &MicroWeights, token: u8) {
        let pos = self.cache_len;
        if pos >= MICRO_MAXIMUM_SEQUENCE { return; }

        let tok = token as usize;
        for i in 0..MICRO_D_MODEL {
            self.buf_x[i] = model.token_embed[tok * MICRO_D_MODEL + i]
                           + model.pos_embed[pos * MICRO_D_MODEL + i];
        }

        for l in 0..MICRO_N_LAYERS {
            let layer = &model.layers[l];

            // Pre-attn RMSNorm
            rmsnorm(&mut self.buf_xn, &self.buf_x, &layer.rms_attn);

            // QKV projections
            matvec(&mut self.buf_q, &layer.w_q, &self.buf_xn, MICRO_D_MODEL, MICRO_D_MODEL);
            matvec(&mut self.buf_k, &layer.w_k, &self.buf_xn, MICRO_D_MODEL, MICRO_D_MODEL);
            matvec(&mut self.buf_v, &layer.w_v, &self.buf_xn, MICRO_D_MODEL, MICRO_D_MODEL);

            self.cache_k[l].extend_from_slice(&self.buf_k);
            self.cache_v[l].extend_from_slice(&self.buf_v);

            let n_position = pos + 1;
            let mut attn_out = vec![0.0f32; MICRO_D_MODEL];
            let dk_sqrt = (MICRO_D_K as f32).sqrt_approx();

            for h in 0..MICRO_N_HEADS {
                let ho = h * MICRO_D_K;
                for t in 0..n_position {
                    let mut score = 0.0f32;
                    for d in 0..MICRO_D_K {
                        score += self.buf_q[ho + d] * self.cache_k[l][t * MICRO_D_MODEL + ho + d];
                    }
                    self.buf_attn[t] = score / dk_sqrt;
                }
                softmax(&mut self.buf_attn[..n_position]);
                for t in 0..n_position {
                    let w = self.buf_attn[t];
                    for d in 0..MICRO_D_K {
                        attn_out[ho + d] += w * self.cache_v[l][t * MICRO_D_MODEL + ho + d];
                    }
                }
            }

            // Output projection + residual
            let mut proj = vec![0.0f32; MICRO_D_MODEL];
            matvec(&mut proj, &layer.w_o, &attn_out, MICRO_D_MODEL, MICRO_D_MODEL);
            for i in 0..MICRO_D_MODEL { self.buf_x[i] += proj[i]; }

            // Pre-FFN RMSNorm
            rmsnorm(&mut self.buf_xn, &self.buf_x, &layer.rms_ffn);

            // SwiGLU FFN
            matvec(&mut self.buf_gate, &layer.w_gate, &self.buf_xn, MICRO_D_MODEL, MICRO_D_FF);
            matvec(&mut self.buf_up, &layer.w_up, &self.buf_xn, MICRO_D_MODEL, MICRO_D_FF);
            for i in 0..MICRO_D_FF {
                let g = self.buf_gate[i];
                let sig = 1.0 / (1.0 + (-g).exp_approx());
                self.buf_gate[i] = g * sig * self.buf_up[i];
            }
            let mut ffn_out = vec![0.0f32; MICRO_D_MODEL];
            matvec(&mut ffn_out, &layer.w_down, &self.buf_gate, MICRO_D_FF, MICRO_D_MODEL);
            for i in 0..MICRO_D_MODEL { self.buf_x[i] += ffn_out[i]; }
        }

        // Final norm + logits
        rmsnorm(&mut self.buf_xn, &self.buf_x, &model.rms_final);
        matvec(&mut self.buf_logits, &model.w_output, &self.buf_xn, MICRO_D_MODEL, MICRO_VOCAB);
        self.cache_len = pos + 1;
    }

    fn sample(&mut self, temperature: f32, top_k: usize, recent: &[u8]) -> u8 {
        if temperature <= 0.01 { return argmax(&self.buf_logits); }

        let mut logits = self.buf_logits.clone();
        for l in logits.iter_mut() { *l /= temperature; }

        // Repetition penalty on last 16 tokens
        let window = recent.len().min(16);
        if window > 0 {
            let start = recent.len() - window;
            for &tok in &recent[start..] {
                let idx = tok as usize;
                if idx < MICRO_VOCAB {
                    if logits[idx] > 0.0 { logits[idx] /= 1.3; }
                    else { logits[idx] *= 1.3; }
                }
            }
        }

        // Top-k
        if top_k > 0 && top_k < MICRO_VOCAB {
            let mut indexed: Vec<(f32, usize)> = logits.iter().copied()
                .enumerate().map(|(i, v)| (v, i)).collect();
            indexed.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(core::cmp::Ordering::Equal));
            let threshold = indexed[top_k.min(indexed.len() - 1)].0;
            for l in logits.iter_mut() { if *l < threshold { *l = f32::NEG_INFINITY; } }
        }

        softmax(&mut logits);
        let r = self.rand_f32();
        let mut cum = 0.0f32;
        for (i, &p) in logits.iter().enumerate() {
            cum += p;
            if cum >= r { return i as u8; }
        }
        (MICRO_VOCAB - 1) as u8
    }

    fn rand_f32(&mut self) -> f32 {
        let mut x = self.rng;
        x ^= x << 13; x ^= x >> 7; x ^= x << 17;
        self.rng = x;
        ((x >> 40) as u32 as f32) / ((1u32 << 24) as f32)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Kernel validation — micro-JARVIS core duties
// ═══════════════════════════════════════════════════════════════════════════════

/// Kernel health check results
pub struct KernelStatus {
    pub heap_ok: bool,
    pub interrupts_ok: bool,
    pub fs_ok: bool,
    pub serial_ok: bool,
    pub full_brain_available: bool,
    pub full_brain_loaded: bool,
}

/// Run kernel validation checks
pub fn kernel_validate() -> KernelStatus {
    KernelStatus {
        heap_ok: check_heap(),
        interrupts_ok: check_interrupts(),
        fs_ok: check_filesystem(),
        serial_ok: check_serial(),
        full_brain_available: check_full_brain_file(),
        full_brain_loaded: false, // Updated by mod.rs
    }
}

fn check_heap() -> bool {
    // Try a small allocation to verify heap works
    let test: Vec<u8> = vec![42u8; 64];
    test.len() == 64
}

fn check_interrupts() -> bool {
    // Verify interrupt flag is enabled
    let flags: u64;
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { core::arch::asm!("pushfq; pop {}", out(reg) flags); }
    (flags & (1 << 9)) != 0 // IF bit
}

fn check_filesystem() -> bool {
    crate::ramfs::with_filesystem(|fs| fs.exists("/"))
}

fn check_serial() -> bool {
    // COM1 base port check
    let status: u8;
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { core::arch::asm!("in al, dx", out("al") status, in("dx") 0x3FDu16); }
    status != 0xFF
}

/// Check if the full brain weights file exists in RamFS
pub fn check_full_brain_file() -> bool {
    crate::ramfs::with_filesystem(|fs| fs.exists("/jarvis/weights.bin"))
}

// ═══════════════════════════════════════════════════════════════════════════════
// Micro corpus — kernel-focused training data
// ═══════════════════════════════════════════════════════════════════════════════

pub static MICRO_CORPUS: &[&str] = &[
    // Kernel commands
    "help: show commands",
    "ls: list files",
    "ps: show processes",
    "free: memory usage",
    "uptime: system uptime",
    "reboot: restart system",
    "shutdown: power off",
    "clear: clear screen",
    // Kernel validation
    "heap OK",
    "interrupts enabled",
    "filesystem ready",
    "serial port active",
    "kernel healthy",
    "all checks passed",
    // Brain management
    "brain loading from fs",
    "brain loaded OK",
    "brain not found",
    "brain save to disk",
    "brain init random",
    "micro sentinel active",
    "full brain connected",
    "full brain offline",
    // Status
    "I am micro-Jarvis",
    "kernel sentinel ready",
    "checking kernel state",
    "validating memory",
    "validating interrupts",
    "filesystem check OK",
    "loading full brain",
    "full brain available",
    "micro mode active",
    "sentinel watching",
    // Diagnostics
    "error: heap corrupt",
    "error: interrupt fault",
    "error: fs unavailable",
    "warning: brain large",
    "status: all nominal",
    "status: degraded",
];

// ═══════════════════════════════════════════════════════════════════════════════
// Math primitives (scalar — micro model is tiny enough)
// ═══════════════════════════════════════════════════════════════════════════════

fn matvec(out: &mut [f32], w: &[f32], x: &[f32], cols: usize, rows: usize) {
    for r in 0..rows {
        let mut sum = 0.0f32;
        let base = r * cols;
        for c in 0..cols { sum += w[base + c] * x[c]; }
        out[r] = sum;
    }
}

fn rmsnorm(out: &mut [f32], x: &[f32], weight: &[f32]) {
    let n = x.len();
    let ss: f32 = x.iter().map(|v| v * v).sum();
    let inv = 1.0 / (ss / n as f32 + MICRO_RMS_EPS).sqrt_approx();
    for i in 0..n { out[i] = x[i] * inv * weight[i]; }
}

fn softmax(data: &mut [f32]) {
    if data.is_empty() { return; }
    let max = data.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    for v in data.iter_mut() { *v = (*v - max).exp_approx(); }
    let sum: f32 = data.iter().sum();
    if sum > 0.0 { let inv = 1.0 / sum; for v in data.iter_mut() { *v *= inv; } }
}

fn argmax(data: &[f32]) -> u8 {
    let mut best = 0;
    for i in 1..data.len() { if data[i] > data[best] { best = i; } }
    best as u8
}

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

fn slice_vec(data: &[f32], pos: &mut usize, count: usize) -> Option<Vec<f32>> {
    if *pos + count > data.len() { return None; }
    let v = data[*pos..*pos + count].to_vec();
    *pos += count;
    Some(v)
}

// ═══════════════════════════════════════════════════════════════════════════════
// f32 extension for no_std math approximations
// ═══════════════════════════════════════════════════════════════════════════════

trait F32Ext {
    fn exp_approx(self) -> f32;
    fn sqrt_approx(self) -> f32;
    fn ln_approx(self) -> f32;
}

// Implémentation de trait — remplit un contrat comportemental.
impl F32Ext for f32 {
    fn exp_approx(self) -> f32 {
        let x = self.clamp(-88.0, 88.0);
        let a = 12102203.0f32;
        let b = 1065353216.0f32;
        let bits = ((a * x + b) as i32).max(0) as u32;
        f32::from_bits(bits)
    }

    fn sqrt_approx(self) -> f32 {
        if self <= 0.0 { return 0.0; }
        let bits = self.to_bits();
        let guess = f32::from_bits((bits >> 1) + 0x1FBD_1DF5);
        (guess + self / guess) * 0.5
    }

    fn ln_approx(self) -> f32 {
        if self <= 0.0 { return -88.0; }
        let bits = self.to_bits();
        let e = ((bits >> 23) & 0xFF) as f32 - 127.0;
        let m = f32::from_bits((bits & 0x007FFFFF) | 0x3F800000);
        (e + (m - 1.0) * 1.4427) * core::f32::consts::LN_2
    }
}
