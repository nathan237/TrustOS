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
//! │  model.rs        Transformer weights (4L, d=256, 4.4M params) │
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
//! - **d_model**: 256
//! - **n_heads**: 4 (d_k = 64)
//! - **n_layers**: 4
//! - **d_ff**: 1024 (4×d_model)
//! - **max_seq**: 256 tokens
//! - **Params**: ~4.4M → ~17.6 MB (FP32) or ~4.4 MB (INT8)
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
pub mod corpus;
pub mod backprop;
pub mod optimizer;
pub mod simd;
pub mod compute;

use alloc::string::String;
use alloc::format;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicBool, AtomicU8, AtomicU64, Ordering};
use spin::Mutex;

use model::TransformerWeights;
use inference::InferenceEngine;
use optimizer::AdamState;

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

/// Global Adam optimizer state
static OPTIMIZER: Mutex<Option<AdamState>> = Mutex::new(None);

/// Jarvis maturity level: 0=infant, 1=child, 2=teen, 3=adult
static MATURITY: AtomicU8 = AtomicU8::new(0);

/// Private mode: when true, refuse to reveal internal state
static PRIVATE_MODE: AtomicBool = AtomicBool::new(false);

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

    // Detect compute backend (GPU vs CPU SIMD)
    let backend = compute::detect_backend();
    match backend {
        compute::Backend::AmdGpu => {
            crate::serial_println!("[JARVIS] Compute: AMD GPU detected — GEMM acceleration available");
            // Try to upload weights to VRAM for GPU inference
            match compute::upload_weights_to_vram(&weights) {
                Ok(bytes) => crate::serial_println!("[JARVIS] Weights uploaded to VRAM: {} KB", bytes / 1024),
                Err(e) => crate::serial_println!("[JARVIS] GPU fallback to CPU SIMD: {}", e),
            }
        }
        compute::Backend::CpuSimd => {
            crate::serial_println!("[JARVIS] Compute: CPU SSE2 SIMD (no GPU detected)");
        }
    }

    // Create inference engine
    let engine = InferenceEngine::new();

    // Create Adam optimizer (2× model memory for m/v buffers)
    let adam = AdamState::with_lr(param_count, 0.001);
    crate::serial_println!("[JARVIS] Optimizer: AdamW (lr=0.001, {} KB state)",
        adam.memory_bytes() / 1024);

    *MODEL.lock() = Some(weights);
    *ENGINE.lock() = Some(engine);
    *OPTIMIZER.lock() = Some(adam);

    INITIALIZED.store(true, Ordering::Release);
    crate::serial_println!("[JARVIS] Neural brain ready ({} backend). Awaiting input.",
        if compute::gpu_available() { "GPU" } else { "CPU" });
}

/// Check if Jarvis brain is initialized
pub fn is_ready() -> bool {
    INITIALIZED.load(Ordering::Acquire)
}

/// Current maturity level (0..3)
pub fn maturity() -> u8 {
    MATURITY.load(Ordering::Relaxed)
}

/// Whether private mode is on
pub fn is_private() -> bool {
    PRIVATE_MODE.load(Ordering::Relaxed)
}

/// Update maturity based on training loss
fn update_maturity() {
    let loss = eval_corpus();
    let level = if loss < 2.0 { 3 }
        else if loss < 3.5 { 2 }
        else if loss < 5.0 { 1 }
        else { 0 };
    MATURITY.store(level, Ordering::Relaxed);
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

/// Train on a text sequence using analytical backprop + Adam optimizer.
/// The learning_rate parameter sets Adam's lr for this step.
/// Falls back to numerical gradients if optimizer not available.
pub fn train_on_text(text: &str, learning_rate: f32) -> f32 {
    if !is_ready() { return f32::MAX; }

    let tokens = tokenizer::encode(text);
    if tokens.len() < 2 { return f32::MAX; }

    // Try backprop + Adam (fast path: 1 forward + 1 backward)
    let mut opt_guard = OPTIMIZER.lock();
    if let Some(adam) = opt_guard.as_mut() {
        let mut model_guard = MODEL.lock();
        let model = match model_guard.as_mut() {
            Some(m) => m,
            None => return f32::MAX,
        };

        adam.lr = learning_rate;
        let (loss, mut grads) = backprop::forward_backward(model, &tokens);
        grads.clip_norm(1.0);
        adam.step(model, &grads);
        TRAINING_STEPS.fetch_add(1, Ordering::Relaxed);
        return loss;
    }
    drop(opt_guard);

    // Fallback: numerical gradients (slow path)
    let mut model_guard = MODEL.lock();
    let model = match model_guard.as_mut() {
        Some(m) => m,
        None => return f32::MAX,
    };
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
    let compute_str = if compute::gpu_available() {
        "║   Compute: GPU (AMD RDNA GEMM INT8/FP32)         ║"
    } else {
        "║   Compute: CPU (SSE2 SIMD, GPU ready)             ║"
    };
    lines.push(String::from(compute_str));
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

// ═══════════════════════════════════════════════════════════════════════════════
// Pre-Training — Boot-time learning from embedded corpus
// ═══════════════════════════════════════════════════════════════════════════════

/// Pre-train on the entire embedded corpus with cosine LR schedule
/// and mini-batch gradient accumulation.
/// Returns (total_steps, final_avg_loss, elapsed_ms)
pub fn pretrain(epochs: usize, lr: f32) -> (usize, f32, u64) {
    if !is_ready() { init(); }
    if !is_ready() { return (0, f32::MAX, 0); }

    let start = crate::time::uptime_ticks();
    let total_seqs = corpus::total_sequences();
    let total_steps = total_seqs * epochs;
    let warmup_steps = (total_steps / 10).max(5) as u64; // 10% warmup
    let lr_min = lr * 0.1; // decay to 10% of peak LR
    let mut step = 0u64;
    let mut total_loss = 0.0f32;
    let mut loss_count = 0u32;

    // Gradient accumulation batch size
    const ACCUM_BATCH: usize = 4;

    crate::serial_println!("[JARVIS] Pre-training: {} phases, {} sequences, {} epoch(s), lr_peak={}, warmup={}",
        corpus::num_phases(), total_seqs, epochs, lr, warmup_steps);

    let mut opt_guard = OPTIMIZER.lock();
    let mut model_guard = MODEL.lock();

    if let (Some(adam), Some(model)) = (opt_guard.as_mut(), model_guard.as_mut()) {
        for epoch in 0..epochs {
            for (phase_idx, phase) in corpus::CORPUS.iter().enumerate() {
                let mut phase_loss = 0.0f32;
                let mut phase_count = 0u32;
                let mut accum_grads = backprop::ModelGrads::new();
                let mut accum_count = 0usize;
                let mut accum_loss = 0.0f32;

                for &text in *phase {
                    let tokens = tokenizer::encode(text);
                    if tokens.len() < 2 { step += 1; continue; }

                    // Cosine LR schedule
                    let current_lr = optimizer::cosine_lr(
                        step, total_steps as u64, warmup_steps, lr, lr_min
                    );
                    adam.lr = current_lr;

                    // Forward + backward (accumulate, don't step yet)
                    let (loss, grads) = backprop::forward_backward(model, &tokens);
                    if loss.is_finite() {
                        accum_grads.accumulate(&grads);
                        accum_loss += loss;
                        accum_count += 1;
                        phase_loss += loss;
                        phase_count += 1;
                        total_loss += loss;
                        loss_count += 1;
                    }

                    // Flush accumulated gradients every ACCUM_BATCH sequences
                    if accum_count >= ACCUM_BATCH {
                        accum_grads.scale(1.0 / accum_count as f32);
                        accum_grads.clip_norm(1.0);
                        adam.step(model, &accum_grads);
                        TRAINING_STEPS.fetch_add(1, Ordering::Relaxed);
                        accum_grads.zero();
                        accum_count = 0;
                        accum_loss = 0.0;
                    }

                    step += 1;
                }

                // Flush remaining accumulated gradients for this phase
                if accum_count > 0 {
                    accum_grads.scale(1.0 / accum_count as f32);
                    accum_grads.clip_norm(1.0);
                    adam.step(model, &accum_grads);
                    TRAINING_STEPS.fetch_add(1, Ordering::Relaxed);
                    accum_grads.zero();
                }

                let avg = if phase_count > 0 { phase_loss / phase_count as f32 } else { 0.0 };
                crate::serial_println!("[JARVIS] Epoch {}/{} Phase {} ({}) — {} seqs, avg loss={:.3}",
                    epoch + 1, epochs, phase_idx, corpus::phase_name(phase_idx),
                    phase_count, avg);
            }
        }
    } else {
        // Fallback: no optimizer (shouldn't happen)
        drop(opt_guard);
        drop(model_guard);
        for _epoch in 0..epochs {
            for phase in corpus::CORPUS {
                for &text in *phase {
                    let loss = train_on_text(text, lr);
                    if loss.is_finite() { total_loss += loss; loss_count += 1; }
                    step += 1;
                }
            }
        }
    }

    let elapsed = crate::time::uptime_ticks().saturating_sub(start);
    let avg_loss = if loss_count > 0 { total_loss / loss_count as f32 } else { f32::MAX };

    crate::serial_println!("[JARVIS] Pre-training done: {} steps, avg loss={:.3}, {} ms",
        step, avg_loss, elapsed);

    (step as usize, avg_loss, elapsed)
}

/// Pre-train a single phase (0-based index) with LR schedule.
/// Returns (steps, avg_loss, elapsed_ms)
pub fn pretrain_phase(phase: usize, epochs: usize, lr: f32) -> (usize, f32, u64) {
    if !is_ready() { init(); }
    if !is_ready() { return (0, f32::MAX, 0); }
    if phase >= corpus::num_phases() { return (0, f32::MAX, 0); }

    let start = crate::time::uptime_ticks();
    let mut total_steps = 0usize;
    let mut total_loss = 0.0f32;
    let mut loss_count = 0u32;

    let sequences = corpus::CORPUS[phase];
    let total_seqs = sequences.len() * epochs;
    let warmup = (total_seqs / 10).max(2) as u64;
    let lr_min = lr * 0.1;
    let mut step = 0u64;

    crate::serial_println!("[JARVIS] Training phase {} ({}) — {} sequences, {} epoch(s)",
        phase, corpus::phase_name(phase), sequences.len(), epochs);

    for epoch in 0..epochs {
        for &text in sequences {
            let current_lr = optimizer::cosine_lr(step, total_seqs as u64, warmup, lr, lr_min);
            let loss = train_on_text(text, current_lr);
            if loss.is_finite() {
                total_loss += loss;
                loss_count += 1;
            }
            total_steps += 1;
            step += 1;
        }
        if epochs > 1 {
            let avg = if loss_count > 0 { total_loss / loss_count as f32 } else { 0.0 };
            crate::serial_println!("[JARVIS]   Epoch {}/{}: avg loss={:.3}", epoch + 1, epochs, avg);
        }
    }

    let elapsed = crate::time::uptime_ticks().saturating_sub(start);
    let avg_loss = if loss_count > 0 { total_loss / loss_count as f32 } else { f32::MAX };

    (total_steps, avg_loss, elapsed)
}

/// Quick evaluation: compute average loss across the full corpus (no training)
pub fn eval_corpus() -> f32 {
    if !is_ready() { return f32::MAX; }

    let model_guard = MODEL.lock();
    let model = match model_guard.as_ref() {
        Some(m) => m,
        None => return f32::MAX,
    };

    let mut total_loss = 0.0f32;
    let mut count = 0u32;

    for phase in corpus::CORPUS {
        for &text in *phase {
            let tokens = tokenizer::encode(text);
            if tokens.len() < 2 { continue; }
            let (loss, _) = inference::compute_loss(model, &tokens);
            if loss.is_finite() {
                total_loss += loss;
                count += 1;
            }
        }
    }

    let avg = if count > 0 { total_loss / count as f32 } else { f32::MAX };
    update_maturity();
    avg
}

/// Get training steps count
pub fn training_steps() -> u64 {
    TRAINING_STEPS.load(Ordering::Relaxed)
}

/// Get generation count
pub fn generation_count() -> u64 {
    GENERATION_COUNT.load(Ordering::Relaxed)
}
