//! JARVIS Neural Brain — Self-hosted AI with bare-metal GPU inference
//!
//! This module provides the neural network core for Jarvis, TrustOS's resident AI.
//! Unlike the shell/jarvis.rs rule-based NLU, this is a real tiny transformer that
//! learns, generates text, and self-optimizes.
//!
//! # Architecture
//!
//! ```text
//! ┌──────────────────────────────────────────────────────────────┐
//! │                    JARVIS NEURAL BRAIN                       │
//! │                                                              │
//! │  tokenizer.rs    Byte-level tokenizer (vocab=256)            │
//! │  model.rs        Transformer weights (4L, d=64, 312K params) │
//! │  inference.rs    Forward pass (CPU or GPU GEMM)              │
//! │  agent.rs        Self-aware agent: introspect, execute, learn│
//! │  mentor.rs       Serial mentoring protocol (QEMU link)       │
//! │  training.rs     On-device learning (loss + weight updates)  │
//! └──────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Model Specs
//!
//! - **Vocab**: 256 (byte-level — no dictionary, works on any text)
//! - **d_model**: 64
//! - **n_heads**: 4 (d_k = 16)
//! - **n_layers**: 4
//! - **d_ff**: 256 (4×d_model)
//! - **max_seq**: 256 tokens
//! - **Params**: ~312K → ~1.2 MB (FP32) or ~312 KB (INT8)
//!
//! # Self-Awareness
//!
//! Jarvis can:
//! - Read its own source code via VFS
//! - Describe its architecture (layers, params, weights)
//! - Execute shell commands and observe results
//! - Request GPU benchmarks and adapt tile sizes
//! - Save/load weights to disk (persists across reboots)
//! - Receive training data via serial from an external mentor
//!
//! # Mentoring Protocol (QEMU Serial → COM1)
//!
//! The mentor (a human or AI on the host) can:
//! ```text
//! MENTOR:TEACH:<text>                 → Train on text sequence
//! MENTOR:CORRECT:<bad>|<good>         → Correction pair
//! MENTOR:EVAL:<prompt>                → Evaluate and report loss
//! MENTOR:SAVE                         → Save weights to disk
//! MENTOR:LOAD                         → Load weights from disk
//! MENTOR:STATUS                       → Report model stats
//! MENTOR:RESET                        → Reinitialize weights
//! ```

pub mod tokenizer;
pub mod model;
pub mod inference;
pub mod agent;
pub mod mentor;
pub mod training;

use alloc::string::String;
use alloc::format;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use spin::Mutex;

use model::TransformerWeights;
use inference::InferenceEngine;

// ═══════════════════════════════════════════════════════════════════════════════
// Global State
// ═══════════════════════════════════════════════════════════════════════════════

static INITIALIZED: AtomicBool = AtomicBool::new(false);
static GENERATION_COUNT: AtomicU64 = AtomicU64::new(0);
static TRAINING_STEPS: AtomicU64 = AtomicU64::new(0);

/// Global model weights (heap-allocated)
static MODEL: Mutex<Option<TransformerWeights>> = Mutex::new(None);

/// Global inference engine
static ENGINE: Mutex<Option<InferenceEngine>> = Mutex::new(None);

// ═══════════════════════════════════════════════════════════════════════════════
// Initialization
// ═══════════════════════════════════════════════════════════════════════════════

/// Initialize Jarvis neural brain with random weights
pub fn init() {
    crate::serial_println!("[JARVIS] Initializing neural brain...");

    // Allocate and initialize model weights
    let weights = TransformerWeights::new_random();
    let param_count = weights.param_count();
    let memory_bytes = param_count * 4; // FP32

    crate::serial_println!("[JARVIS] Model: {} layers, d_model={}, d_ff={}, {} heads",
        model::N_LAYERS, model::D_MODEL, model::D_FF, model::N_HEADS);
    crate::serial_println!("[JARVIS] Parameters: {} ({} KB FP32)",
        param_count, memory_bytes / 1024);

    // Create inference engine
    let engine = InferenceEngine::new();

    *MODEL.lock() = Some(weights);
    *ENGINE.lock() = Some(engine);

    INITIALIZED.store(true, Ordering::Release);
    crate::serial_println!("[JARVIS] Neural brain ready. Awaiting input or mentoring.");
}

/// Check if Jarvis brain is initialized
pub fn is_ready() -> bool {
    INITIALIZED.load(Ordering::Acquire)
}

// ═══════════════════════════════════════════════════════════════════════════════
// Generation API
// ═══════════════════════════════════════════════════════════════════════════════

/// Generate text from a prompt using the neural model
pub fn generate(prompt: &str, max_tokens: usize) -> String {
    if !is_ready() {
        return String::from("[JARVIS brain not initialized]");
    }

    let model_guard = MODEL.lock();
    let model = match model_guard.as_ref() {
        Some(m) => m,
        None => return String::from("[JARVIS no model loaded]"),
    };

    let mut engine_guard = ENGINE.lock();
    let engine = match engine_guard.as_mut() {
        Some(e) => e,
        None => return String::from("[JARVIS no engine]"),
    };

    // Tokenize prompt (byte-level)
    let tokens = tokenizer::encode(prompt);

    // Generate tokens autoregressively
    let generated = engine.generate(model, &tokens, max_tokens);

    GENERATION_COUNT.fetch_add(1, Ordering::Relaxed);

    // Decode back to text
    tokenizer::decode(&generated)
}

/// Get a single next-token prediction (for interactive use)
pub fn predict_next(context: &[u8]) -> u8 {
    if !is_ready() { return b'?' }

    let model_guard = MODEL.lock();
    let model = match model_guard.as_ref() {
        Some(m) => m,
        None => return b'?',
    };

    let mut engine_guard = ENGINE.lock();
    let engine = match engine_guard.as_mut() {
        Some(e) => e,
        None => return b'?',
    };

    engine.predict_next_token(model, context)
}

// ═══════════════════════════════════════════════════════════════════════════════
// Training API
// ═══════════════════════════════════════════════════════════════════════════════

/// Train on a text sequence (teacher forcing)
pub fn train_on_text(text: &str, learning_rate: f32) -> f32 {
    if !is_ready() { return f32::MAX; }

    let mut model_guard = MODEL.lock();
    let model = match model_guard.as_mut() {
        Some(m) => m,
        None => return f32::MAX,
    };

    let tokens = tokenizer::encode(text);
    if tokens.len() < 2 { return f32::MAX; }

    let loss = training::train_step(model, &tokens, learning_rate);
    TRAINING_STEPS.fetch_add(1, Ordering::Relaxed);

    loss
}

// ═══════════════════════════════════════════════════════════════════════════════
// Status & Introspection
// ═══════════════════════════════════════════════════════════════════════════════

/// Get model info as formatted lines
pub fn info_lines() -> Vec<String> {
    let mut lines = Vec::new();
    lines.push(String::from("╔═══════════════════════════════════════════════════╗"));
    lines.push(String::from("║     J.A.R.V.I.S. Neural Brain v2.0               ║"));
    lines.push(String::from("║     Self-Hosted Tiny Transformer                  ║"));
    lines.push(String::from("╠═══════════════════════════════════════════════════╣"));
    
    if let Some(model) = MODEL.lock().as_ref() {
        let params = model.param_count();
        lines.push(format!("║ Architecture: {}L × d{} × {}H × ff{}     ║",
            model::N_LAYERS, model::D_MODEL, model::N_HEADS, model::D_FF));
        lines.push(format!("║ Parameters:   {} ({:.1} KB FP32)           ║",
            params, params as f64 * 4.0 / 1024.0));
        lines.push(format!("║ Vocab:        {} (byte-level)                   ║", model::VOCAB_SIZE));
        lines.push(format!("║ Context:      {} tokens                         ║", model::MAX_SEQ));
    } else {
        lines.push(String::from("║ Model: NOT LOADED                                 ║"));
    }

    lines.push(format!("║ Initialized:  {}                                  ║", is_ready()));
    lines.push(format!("║ Generations:  {}                                  ║",
        GENERATION_COUNT.load(Ordering::Relaxed)));
    lines.push(format!("║ Train steps:  {}                                  ║",
        TRAINING_STEPS.load(Ordering::Relaxed)));
    lines.push(String::from("╠═══════════════════════════════════════════════════╣"));
    lines.push(String::from("║ Capabilities:                                     ║"));
    lines.push(String::from("║   Text generation (autoregressive, temperature)   ║"));
    lines.push(String::from("║   On-device training (teacher forcing + SGD)       ║"));
    lines.push(String::from("║   Self-introspection (read own code/weights)       ║"));
    lines.push(String::from("║   Serial mentoring (learn from external AI)        ║"));
    lines.push(String::from("║   Shell command execution (agent mode)             ║"));
    lines.push(String::from("║   GPU dispatch ready (GEMM INT8/FP32 kernels)     ║"));
    lines.push(String::from("╚═══════════════════════════════════════════════════╝"));

    lines
}

/// Get compact stats
pub fn stats() -> String {
    format!("Jarvis: {}K params, {} gens, {} train steps, ready={}",
        if let Some(m) = MODEL.lock().as_ref() { m.param_count() / 1000 } else { 0 },
        GENERATION_COUNT.load(Ordering::Relaxed),
        TRAINING_STEPS.load(Ordering::Relaxed),
        is_ready())
}

/// Reset model weights to random initialization
pub fn reset() {
    if let Some(model) = MODEL.lock().as_mut() {
        model.reset();
        TRAINING_STEPS.store(0, Ordering::Relaxed);
        GENERATION_COUNT.store(0, Ordering::Relaxed);
    }
}
