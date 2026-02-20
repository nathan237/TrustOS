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

// ═══════════════════════════════════════════════════════════════════════════════
// Persistence — Save/Load weights to disk (RamFS)
// ═══════════════════════════════════════════════════════════════════════════════

const WEIGHTS_DIR: &str = "/jarvis";
const WEIGHTS_PATH: &str = "/jarvis/weights.bin";
const WEIGHTS_META: &str = "/jarvis/meta.txt";

/// Save model weights to /jarvis/weights.bin in RamFS
pub fn save_weights() -> Result<usize, &'static str> {
    let model_guard = MODEL.lock();
    let model = model_guard.as_ref().ok_or("Model not loaded")?;

    let floats = model.serialize();
    let byte_count = floats.len() * 4;

    // Convert f32 slice to u8 slice (safe: same alignment guarantees in our allocator)
    let bytes: &[u8] = unsafe {
        core::slice::from_raw_parts(floats.as_ptr() as *const u8, byte_count)
    };

    // Ensure /jarvis directory exists
    crate::ramfs::with_fs(|fs| {
        let _ = fs.mkdir(WEIGHTS_DIR); // Ignore AlreadyExists
    });

    // Create file if needed, then write
    crate::ramfs::with_fs(|fs| {
        let _ = fs.touch(WEIGHTS_PATH);
        fs.write_file(WEIGHTS_PATH, bytes).map_err(|_| "Write failed")
    })?;

    // Save metadata
    let meta = format!("params={}\nsteps={}\ngens={}\nbytes={}\n",
        model.param_count(),
        TRAINING_STEPS.load(Ordering::Relaxed),
        GENERATION_COUNT.load(Ordering::Relaxed),
        byte_count);
    crate::ramfs::with_fs(|fs| {
        let _ = fs.touch(WEIGHTS_META);
        let _ = fs.write_file(WEIGHTS_META, meta.as_bytes());
    });

    crate::serial_println!("[JARVIS] Saved {} floats ({} KB) to {}",
        floats.len(), byte_count / 1024, WEIGHTS_PATH);

    Ok(byte_count)
}

/// Load model weights from /jarvis/weights.bin
pub fn load_weights() -> Result<usize, &'static str> {
    let data = crate::ramfs::with_fs(|fs| {
        fs.read_file(WEIGHTS_PATH).map(|d| d.to_vec())
    }).map_err(|_| "No saved weights found")?;

    if data.len() % 4 != 0 {
        return Err("Invalid weight file (size not multiple of 4)");
    }

    let float_count = data.len() / 4;
    let floats: &[f32] = unsafe {
        core::slice::from_raw_parts(data.as_ptr() as *const f32, float_count)
    };

    let new_model = model::TransformerWeights::deserialize(floats)
        .ok_or("Deserialization failed (wrong size)")?;

    let param_count = new_model.param_count();
    *MODEL.lock() = Some(new_model);

    if !INITIALIZED.load(Ordering::Acquire) {
        *ENGINE.lock() = Some(InferenceEngine::new());
        INITIALIZED.store(true, Ordering::Release);
    }

    crate::serial_println!("[JARVIS] Loaded {} params ({} KB) from {}",
        param_count, data.len() / 1024, WEIGHTS_PATH);

    Ok(data.len())
}

/// Check if saved weights exist on disk
pub fn has_saved_weights() -> bool {
    crate::ramfs::with_fs(|fs| fs.exists(WEIGHTS_PATH))
}

// ═══════════════════════════════════════════════════════════════════════════════
// Neural Fallback — Used by shell/jarvis.rs when rule-based has no answer
// ═══════════════════════════════════════════════════════════════════════════════

/// Generate a neural response for the shell Jarvis (auto-init if needed)
/// Returns None if brain is not available
pub fn neural_respond(query: &str) -> Option<String> {
    if !is_ready() {
        // Try loading saved weights first
        if has_saved_weights() {
            let _ = load_weights();
        } else {
            init();
        }
    }
    if !is_ready() { return None; }

    let output = generate(query, 64);
    if output.is_empty() { return None; }
    Some(output)
}

/// Train the neural brain on a conversation exchange (background learning)
pub fn learn_from_exchange(user_input: &str, good_response: &str) {
    if !is_ready() { return; }
    // Train on the pattern: <user_input>\n<good_response>
    let mut training_text = String::with_capacity(user_input.len() + good_response.len() + 1);
    training_text.push_str(user_input);
    training_text.push('\n');
    training_text.push_str(good_response);
    let _ = train_on_text(&training_text, 0.0005); // Lower LR for background learning
}
