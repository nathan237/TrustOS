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
pub mod corpus_trustos;
pub mod corpus;
pub mod backprop;
pub mod optimizer;
pub mod simd;
pub mod compute;
pub mod hw_corpus;
pub mod mesh;
pub mod rpc;
pub mod consensus;
pub mod federated;
pub mod pxe_replicator;
pub mod guardian;
pub mod task;
pub mod compression;
pub mod io_control;
pub mod micro_model;
pub mod training_loop;
pub mod training_dashboard;
pub mod conversation_log;
pub mod developmental;
pub mod heartbeat;
pub mod genome;
pub mod trace;

use alloc::string::String;
use alloc::format;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicBool, AtomicU8, AtomicU32, AtomicU64, AtomicUsize, Ordering};
use spin::Mutex;

use model::TransformerWeights;
use inference::InferenceEngine;
use optimizer::AdamState;
use micro_model::{MicroWeights, MicroEngine};

/// Max tokens per sequence during training (compile-time default)
/// Keep short: each forward+backward scales as O(seq² × d² × layers).
/// On a G4400 (no AVX), seq=64 takes ~30s+ per tick — starves network.
pub const TRAIN_MAX_SEQ: usize = 16;

// ═══════════════════════════════════════════════════════════════════════════════
// SSD-configurable training parameters (set by /mnt/sda1/training.cfg)
// ═══════════════════════════════════════════════════════════════════════════════

/// Runtime-configurable max seq length (0 = use TRAIN_MAX_SEQ const)
pub static TRAIN_MAX_SEQ_CFG: AtomicUsize = AtomicUsize::new(0);
/// Runtime LR max (bits of f32, 0 = use default)
pub static LR_MAX_CFG: AtomicU32 = AtomicU32::new(0);
/// Runtime LR min (bits of f32, 0 = use default)
pub static LR_MIN_CFG: AtomicU32 = AtomicU32::new(0);
/// Runtime epochs (0 = use default from TrainingConfig)
pub static EPOCHS_CFG: AtomicU32 = AtomicU32::new(0);
/// Runtime early_stop override (default true)
pub static EARLY_STOP_CFG: AtomicBool = AtomicBool::new(true);

/// Get effective max_seq (config override or compile-time default)
pub fn effective_max_seq() -> usize {
    let cfg = TRAIN_MAX_SEQ_CFG.load(Ordering::Relaxed);
    if cfg > 0 { cfg } else { TRAIN_MAX_SEQ }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Global State
// ═══════════════════════════════════════════════════════════════════════════════

static INITIALIZED: AtomicBool = AtomicBool::new(false);
static GENERATION_COUNT: AtomicU64 = AtomicU64::new(0);
static TRAINING_STEPS: AtomicU64 = AtomicU64::new(0);

/// Full model weights (loaded from FS — 4.4M params, ~17.6 MB)
static MODEL: Mutex<Option<TransformerWeights>> = Mutex::new(None);

/// Full inference engine
static ENGINE: Mutex<Option<InferenceEngine>> = Mutex::new(None);

/// Global Adam optimizer state
static OPTIMIZER: Mutex<Option<AdamState>> = Mutex::new(None);

/// Jarvis maturity level: 0=infant, 1=child, 2=teen, 3=adult
static MATURITY: AtomicU8 = AtomicU8::new(0);

/// Private mode: when true, refuse to reveal internal state
static PRIVATE_MODE: AtomicBool = AtomicBool::new(false);

/// Micro sentinel model (embedded in kernel — ~50K params, ~305 KB)
static MICRO_MODEL: Mutex<Option<MicroWeights>> = Mutex::new(None);

/// Micro sentinel engine
static MICRO_ENGINE: Mutex<Option<MicroEngine>> = Mutex::new(None);

/// True once the full brain is loaded from FS
static FULL_BRAIN_LOADED: AtomicBool = AtomicBool::new(false);

// ═══════════════════════════════════════════════════════════════════════════════
// Initialization
// ═══════════════════════════════════════════════════════════════════════════════

/// Initialize Jarvis with two-tier architecture:
/// 1. Load micro sentinel (embedded, ~305 KB) — instant boot
/// 2. Validate kernel via micro sentinel
/// 3. Attempt to load full brain from FS (/jarvis/weights.bin)
pub fn init() {
    crate::serial_println!("[JARVIS] Two-tier brain init starting...");

    // ── Phase 1: Load micro sentinel (always available) ──────────────────────
    static MICRO_BIN: &[u8] = include_bytes!("jarvis_micro.bin");
    if MICRO_BIN.len() >= 4 && MICRO_BIN.len() % 4 == 0 {
        let float_count = MICRO_BIN.len() / 4;
        let floats: &[f32] = unsafe {
            core::slice::from_raw_parts(MICRO_BIN.as_ptr() as *const f32, float_count)
        };
        if let Some(micro_weights) = MicroWeights::deserialize(floats) {
            crate::serial_println!("[JARVIS] Micro sentinel loaded ({} params, {} KB)",
                micro_weights.param_count(), MICRO_BIN.len() / 1024);
            *MICRO_MODEL.lock() = Some(micro_weights);
            *MICRO_ENGINE.lock() = Some(MicroEngine::new());
        } else {
            crate::serial_println!("[JARVIS] WARN: micro sentinel deserialize failed, using random");
            *MICRO_MODEL.lock() = Some(MicroWeights::new_random());
            *MICRO_ENGINE.lock() = Some(MicroEngine::new());
        }
    } else {
        crate::serial_println!("[JARVIS] WARN: micro binary invalid, using random");
        *MICRO_MODEL.lock() = Some(MicroWeights::new_random());
        *MICRO_ENGINE.lock() = Some(MicroEngine::new());
    }

    // ── Phase 2: Kernel validation via micro sentinel ────────────────────────
    let status = micro_model::kernel_validate();
    crate::serial_println!("[JARVIS] Kernel validation: heap={} int={} fs={} serial={}",
        status.heap_ok, status.interrupts_ok, status.fs_ok, status.serial_ok);

    // ── Phase 3: Initialize SIMD + compute backend ───────────────────────────
    simd::init_dispatch();
    let backend = compute::detect_backend();
    match backend {
        compute::Backend::AmdGpu => crate::serial_println!("[JARVIS] Compute: AMD GPU detected"),
        compute::Backend::CpuSimd => crate::serial_println!("[JARVIS] Compute: CPU SSE2 SIMD"),
    }

    // Mark as initialized (micro is ready, full brain loading is optional)
    INITIALIZED.store(true, Ordering::Release);

    // ── Phase 4: Try loading full brain from FS ──────────────────────────────
    if status.fs_ok {
        match load_full_brain() {
            Ok(bytes) => {
                crate::serial_println!("[JARVIS] Full brain loaded from FS ({} KB)", bytes / 1024);
                // Free the RamFS copy — model is now in MODEL global, we don't need 17.6MB duplicate
                let _ = crate::ramfs::with_fs(|fs| fs.rm(WEIGHTS_PATH));
                // Set maturity directly — skip eval during init (too slow for 4.4M model in VM)
                // Pretrained weights are Adult-level (loss ≈ 1.33)
                MATURITY.store(3, Ordering::Relaxed);
                crate::serial_println!("[JARVIS] Maturity: {} (level {})",
                    maturity_name(), maturity());
            }
            Err(e) => {
                crate::serial_println!("[JARVIS] Full brain not available: {} — micro sentinel active", e);
            }
        }
    } else {
        crate::serial_println!("[JARVIS] FS not ready — micro sentinel only mode");
    }

    crate::serial_println!("[JARVIS] Init complete. Micro={}, Full={}",
        if MICRO_MODEL.lock().is_some() { "OK" } else { "FAIL" },
        if FULL_BRAIN_LOADED.load(Ordering::Relaxed) { "LOADED" } else { "NOT LOADED" });

    // Initialize developmental learning system
    developmental::init();
    // If full brain was loaded (pretrained), set stage based on maturity
    if FULL_BRAIN_LOADED.load(Ordering::Relaxed) {
        developmental::set_stage(match MATURITY.load(Ordering::Relaxed) {
            0 => developmental::Stage::Infant,
            1 => developmental::Stage::Child,
            2 => developmental::Stage::PreTeen,
            _ => developmental::Stage::Adult,
        });
    }

    // Auto-start mesh when full brain is available (makes this node discoverable)
    if has_full_brain() && !mesh::is_active() {
        mesh_start();
        crate::serial_println!("[JARVIS] Mesh auto-started (full brain available for peers)");
    }
}

/// Load the full brain (4.4M params) from /jarvis/weights.bin in RamFS
pub fn load_full_brain() -> Result<usize, &'static str> {
    let data = crate::ramfs::with_fs(|fs| {
        fs.read_file(WEIGHTS_PATH).map(|d| d.to_vec())
    }).map_err(|_| "weights.bin not found in FS")?;

    if data.len() % 4 != 0 || data.len() < 1024 {
        return Err("Invalid weight file");
    }

    let float_count = data.len() / 4;
    let floats: &[f32] = unsafe {
        core::slice::from_raw_parts(data.as_ptr() as *const f32, float_count)
    };

    let weights = model::TransformerWeights::deserialize(floats)
        .ok_or("Deserialization failed (wrong param count)")?;

    let param_count = weights.param_count();

    // Upload to GPU if available
    if compute::gpu_available() {
        let _ = compute::upload_weights_to_vram(&weights);
    }

    let engine = InferenceEngine::new();
    let adam = AdamState::with_lr(param_count, 0.001);

    *MODEL.lock() = Some(weights);
    *ENGINE.lock() = Some(engine);
    *OPTIMIZER.lock() = Some(adam);
    FULL_BRAIN_LOADED.store(true, Ordering::Release);

    crate::serial_println!("[JARVIS] Full brain: {} params ({} KB)",
        param_count, data.len() / 1024);

    Ok(data.len())
}

// ═══════════════════════════════════════════════════════════════════════════════
// External Brain Loading — FAT32 / VFS / HTTP / USB
// ═══════════════════════════════════════════════════════════════════════════════

/// Install full brain from raw weight bytes (shared by all external loaders)
fn install_brain_from_bytes(data: &[u8], source: &str) -> Result<usize, &'static str> {
    if data.len() % 4 != 0 || data.len() < 1024 {
        return Err("Invalid weight data (bad size or alignment)");
    }

    let float_count = data.len() / 4;
    let floats: &[f32] = unsafe {
        core::slice::from_raw_parts(data.as_ptr() as *const f32, float_count)
    };

    let weights = model::TransformerWeights::deserialize(floats)
        .ok_or("Deserialization failed (wrong param count)")?;

    let param_count = weights.param_count();

    if compute::gpu_available() {
        let _ = compute::upload_weights_to_vram(&weights);
    }

    let engine = InferenceEngine::new();
    let adam = AdamState::with_lr(param_count, 0.001);

    *MODEL.lock() = Some(weights);
    *ENGINE.lock() = Some(engine);
    *OPTIMIZER.lock() = Some(adam);
    FULL_BRAIN_LOADED.store(true, Ordering::Release);

    crate::serial_println!("[JARVIS] Full brain loaded from {}: {} params ({} KB)",
        source, param_count, data.len() / 1024);

    Ok(data.len())
}

/// Load full brain from any VFS path (FAT32 at /mnt/fat32, ext4, TrustFS, etc.)
pub fn load_brain_from_vfs(path: &str) -> Result<usize, &'static str> {
    crate::serial_println!("[JARVIS] Loading brain from VFS: {}", path);
    let data = crate::vfs::read_file(path)
        .map_err(|_| "File not found or read error on VFS path")?;
    install_brain_from_bytes(&data, path)
}

/// Load full brain from FAT32 external disk (default: /mnt/fat32/jarvis_weights.bin)
pub fn load_brain_from_fat32(filename: Option<&str>) -> Result<usize, &'static str> {
    let path = match filename {
        Some(name) => format!("/mnt/fat32/{}", name),
        None => String::from("/mnt/fat32/jarvis_weights.bin"),
    };
    crate::serial_println!("[JARVIS] Loading brain from FAT32: {}", path);
    let data = crate::vfs::read_file(&path)
        .map_err(|_| "File not found on FAT32 (is a FAT32 disk attached?)")?;
    install_brain_from_bytes(&data, &path)
}

/// Load full brain from HTTP URL (downloads weights into RAM then installs)
pub fn load_brain_from_http(url: &str) -> Result<usize, &'static str> {
    crate::serial_println!("[JARVIS] Downloading brain from: {}", url);
    let response = crate::netstack::http::get(url)?;
    if response.status_code != 200 {
        return Err("HTTP download failed (non-200 status)");
    }
    if response.body.is_empty() {
        return Err("HTTP response body is empty");
    }
    crate::serial_println!("[JARVIS] Downloaded {} KB", response.body.len() / 1024);
    install_brain_from_bytes(&response.body, url)
}

/// Copy loaded brain into RamFS for fast access on next boot
pub fn cache_brain_to_ramfs() -> Result<usize, &'static str> {
    let model_guard = MODEL.lock();
    let model = model_guard.as_ref().ok_or("No brain loaded to cache")?;
    let floats = model.serialize();
    let byte_count = floats.len() * 4;
    let bytes: &[u8] = unsafe {
        core::slice::from_raw_parts(floats.as_ptr() as *const u8, byte_count)
    };
    crate::ramfs::with_fs(|fs| {
        let _ = fs.mkdir(WEIGHTS_DIR);
    });
    crate::ramfs::with_fs(|fs| {
        let _ = fs.touch(WEIGHTS_PATH);
        fs.write_file(WEIGHTS_PATH, bytes).map_err(|_| "Cache write failed")
    })?;
    crate::serial_println!("[JARVIS] Brain cached to RamFS ({} KB)", byte_count / 1024);
    Ok(byte_count)
}

/// Check if full brain is loaded
pub fn has_full_brain() -> bool {
    FULL_BRAIN_LOADED.load(Ordering::Acquire)
}

/// Get maturity level name
pub fn maturity_name() -> &'static str {
    match maturity() {
        0 => "Infant",
        1 => "Child",
        2 => "Teen",
        3 => "Adult",
        _ => "Unknown",
    }
}

/// Initialize Jarvis neural brain with random full weights (fallback)
pub fn init_random() {
    // Memory check: weights(130MB) + Adam(260MB) + headroom(50MB) = ~440MB minimum
    let heap_free = crate::memory::heap::free();
    let needed = 440 * 1024 * 1024;
    crate::serial_println!("[JARVIS] Heap free: {} MB, need ~{} MB for full brain",
        heap_free / (1024 * 1024), needed / (1024 * 1024));
    if heap_free < needed {
        crate::serial_println!("[JARVIS] ERROR: Not enough heap for full brain ({} MB free < {} MB needed)",
            heap_free / (1024 * 1024), needed / (1024 * 1024));
        return;
    }

    crate::serial_println!("[JARVIS] Initializing random full brain...");

    let weights = TransformerWeights::new_random();
    let param_count = weights.param_count();

    crate::serial_println!("[JARVIS] Model: {} layers, d_model={}, d_ff={}, {} heads",
        model::N_LAYERS, model::D_MODEL, model::D_FF, model::N_HEADS);
    crate::serial_println!("[JARVIS] Parameters: {} ({} KB FP32)",
        param_count, param_count * 4 / 1024);

    simd::init_dispatch();
    let backend = compute::detect_backend();
    match backend {
        compute::Backend::AmdGpu => {
            crate::serial_println!("[JARVIS] Compute: AMD GPU — GEMM available");
            match compute::upload_weights_to_vram(&weights) {
                Ok(bytes) => crate::serial_println!("[JARVIS] VRAM: {} KB", bytes / 1024),
                Err(e) => crate::serial_println!("[JARVIS] GPU fallback: {}", e),
            }
        }
        compute::Backend::CpuSimd => {
            crate::serial_println!("[JARVIS] Compute: CPU SSE2 SIMD");
        }
    }

    let engine = InferenceEngine::new();
    let adam = AdamState::with_lr(param_count, 0.001);

    *MODEL.lock() = Some(weights);
    *ENGINE.lock() = Some(engine);
    *OPTIMIZER.lock() = Some(adam);
    FULL_BRAIN_LOADED.store(true, Ordering::Release);
    INITIALIZED.store(true, Ordering::Release);

    crate::serial_println!("[JARVIS] Random full brain ready.");
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
    let loss = eval_quick();
    let level = if loss < 2.0 { 3 }
        else if loss < 3.5 { 2 }
        else if loss < 5.0 { 1 }
        else { 0 };
    MATURITY.store(level, Ordering::Relaxed);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Generation API
// ═══════════════════════════════════════════════════════════════════════════════

/// Generate text from a prompt using the neural model.
/// Routes to full brain if loaded, else falls back to micro sentinel.
pub fn generate(prompt: &str, max_tokens: usize) -> String {
    if !is_ready() {
        return String::from("[JARVIS brain not initialized]");
    }

    // Feed input to developmental perception system
    if developmental::is_active() {
        developmental::observe(prompt.as_bytes());
    }

    // Route: full brain if available
    if FULL_BRAIN_LOADED.load(Ordering::Acquire) {
        let model_guard = MODEL.lock();
        let model = match model_guard.as_ref() {
            Some(m) => m,
            None => return micro_generate(prompt, max_tokens),
        };

        let mut engine_guard = ENGINE.lock();
        let engine = match engine_guard.as_mut() {
            Some(e) => e,
            None => return micro_generate(prompt, max_tokens),
        };

        let tokens = tokenizer::encode(prompt);
        let generated = engine.generate(model, &tokens, max_tokens);
        GENERATION_COUNT.fetch_add(1, Ordering::Relaxed);
        return tokenizer::decode(&generated);
    }

    // Fallback: micro sentinel
    micro_generate(prompt, max_tokens)
}

/// Generate using the micro sentinel (small model, always available)
fn micro_generate(prompt: &str, max_tokens: usize) -> String {
    let micro_guard = MICRO_MODEL.lock();
    let micro = match micro_guard.as_ref() {
        Some(m) => m,
        None => return String::from("[micro sentinel not loaded]"),
    };

    let mut engine_guard = MICRO_ENGINE.lock();
    let engine = match engine_guard.as_mut() {
        Some(e) => e,
        None => return String::from("[micro engine not loaded]"),
    };

    let tokens: Vec<u8> = prompt.bytes().collect();
    let capped = max_tokens.min(micro_model::MICRO_MAX_SEQ);
    let generated = engine.generate(micro, &tokens, capped);
    GENERATION_COUNT.fetch_add(1, Ordering::Relaxed);
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

    // Guardian authorization required for training
    if let Err(msg) = guardian::authorize(guardian::ProtectedOp::Train) {
        crate::serial_println!("[JARVIS] {}", msg);
        return f32::MAX;
    }

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
    lines.push(String::from("║   J.A.R.V.I.S. Neural Brain v3.0 (Two-Tier)     ║"));
    lines.push(String::from("║   Micro Sentinel + Full Brain Architecture       ║"));
    lines.push(String::from("╠═══════════════════════════════════════════════════╣"));

    // Micro sentinel info
    if let Some(micro) = MICRO_MODEL.lock().as_ref() {
        lines.push(format!("║ Micro:  {}L × d{} × {}H  ({} params, {:.0} KB)     ║",
            micro_model::MICRO_N_LAYERS, micro_model::MICRO_D_MODEL,
            micro_model::MICRO_N_HEADS, micro.param_count(),
            micro.param_count() as f64 * 4.0 / 1024.0));
        lines.push(String::from("║ Role:   Kernel sentinel, validation, fallback    ║"));
    } else {
        lines.push(String::from("║ Micro:  NOT LOADED                               ║"));
    }

    lines.push(String::from("╠═══════════════════════════════════════════════════╣"));

    // Full brain info
    let full = FULL_BRAIN_LOADED.load(Ordering::Relaxed);
    if full {
        if let Some(model) = MODEL.lock().as_ref() {
            let params = model.param_count();
            lines.push(format!("║ Full:   {}L × d{} × {}H × ff{}  ({:.1}M params)  ║",
                model::N_LAYERS, model::D_MODEL, model::N_HEADS, model::D_FF,
                params as f64 / 1_000_000.0));
            lines.push(format!("║ Memory: {:.1} MB FP32   Vocab: {} (byte)          ║",
                params as f64 * 4.0 / (1024.0 * 1024.0), model::VOCAB_SIZE));
        }
        lines.push(format!("║ Status: LOADED from FS   Maturity: {} ({})    ║",
            maturity(), maturity_name()));
    } else {
        lines.push(String::from("║ Full:   NOT LOADED (use 'jarvis brain load')     ║"));
        lines.push(String::from("║ Status: MICRO ONLY MODE                          ║"));
    }

    lines.push(String::from("╠═══════════════════════════════════════════════════╣"));
    lines.push(format!("║ Generations:  {}                                  ║",
        GENERATION_COUNT.load(Ordering::Relaxed)));
    lines.push(format!("║ Train steps:  {}                                  ║",
        TRAINING_STEPS.load(Ordering::Relaxed)));
    let compute_str = if compute::gpu_available() {
        "║ Compute: GPU (AMD RDNA GEMM)                      ║"
    } else {
        "║ Compute: CPU (SSE2 SIMD)                          ║"
    };
    lines.push(String::from(compute_str));
    lines.push(String::from("╚═══════════════════════════════════════════════════╝"));

    lines
}

/// Get compact stats
pub fn stats() -> String {
    let full = if FULL_BRAIN_LOADED.load(Ordering::Relaxed) {
        format!("full={}K", if let Some(m) = MODEL.lock().as_ref() { m.param_count() / 1000 } else { 0 })
    } else {
        String::from("full=OFF")
    };
    let micro = if MICRO_MODEL.lock().is_some() { "micro=OK" } else { "micro=OFF" };
    format!("Jarvis: {} {} gens={} steps={} ready={}",
        micro, full,
        GENERATION_COUNT.load(Ordering::Relaxed),
        TRAINING_STEPS.load(Ordering::Relaxed),
        is_ready())
}

/// Check if this JARVIS instance is ready for optimal network propagation.
/// Verifies: brain initialized, I/O control sufficient, network available.
pub fn propagation_ready() -> bool {
    if !is_ready() { return false; }
    let audit = io_control::full_audit();
    io_control::network_ready(&audit)
}

/// Get full propagation status including I/O audit, compression stats, and federated state
pub fn propagation_status() -> Vec<String> {
    let mut lines = Vec::new();

    lines.push(String::from("╔═══════════════════════════════════════════════════╗"));
    lines.push(String::from("║  JARVIS NETWORK PROPAGATION STATUS               ║"));
    lines.push(String::from("╠═══════════════════════════════════════════════════╣"));

    // Brain state
    lines.push(format!("║ Brain:       {} ({} steps, {} gens){}",
        if is_ready() { "READY" } else { "NOT READY" },
        TRAINING_STEPS.load(Ordering::Relaxed),
        GENERATION_COUNT.load(Ordering::Relaxed),
        "         ║"));

    // Federated state
    lines.push(format!("║ Federated:   {}{}",
        if federated::is_enabled() { "ENABLED" } else { "DISABLED" },
        "                            ║"));
    lines.push(format!("║ Fed Rounds:  {}  Interval: {}ms{}",
        federated::rounds_completed(),
        federated::current_interval_ms(),
        "              ║"));

    // Compression stats
    let saved = federated::bytes_saved();
    lines.push(format!("║ Bandwidth:   {} KB saved by compression{}",
        saved / 1024, "          ║"));

    // Mesh state
    lines.push(format!("║ Mesh Peers:  {}  Role: {:?}{}",
        mesh::peer_count(),
        mesh::our_role(),
        "                       ║"));

    // I/O audit
    let audit = io_control::full_audit();
    let score = io_control::control_score(&audit);
    let caps = io_control::capability_bitmask(&audit);
    lines.push(format!("║ I/O Score:   {}%  Caps: 0x{:04X}{}",
        score, caps, "                   ║"));
    lines.push(format!("║ Net Ready:   {}  Full Ctrl: {}{}",
        if io_control::network_ready(&audit) { "YES" } else { "NO " },
        if io_control::full_control(&audit) { "YES" } else { "NO " },
        "                  ║"));

    lines.push(String::from("╚═══════════════════════════════════════════════════╝"));
    lines
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

/// Load model weights from /jarvis/weights.bin (alias for load_full_brain)
pub fn load_weights() -> Result<usize, &'static str> {
    load_full_brain()
}

/// Check if saved weights exist on disk
pub fn has_saved_weights() -> bool {
    crate::ramfs::with_fs(|fs| fs.exists(WEIGHTS_PATH))
}

// ═══════════════════════════════════════════════════════════════════════════════
// Neural Fallback — Used by shell/jarvis.rs when rule-based has no answer
// ═══════════════════════════════════════════════════════════════════════════════

/// Generate a neural response for the shell Jarvis (auto-init if needed)
/// Routes to full brain → micro sentinel → None
pub fn neural_respond(query: &str) -> Option<String> {
    if !is_ready() {
        init();
    }
    if !is_ready() { return None; }

    // Try loading full brain if not already loaded
    if !FULL_BRAIN_LOADED.load(Ordering::Relaxed) && has_saved_weights() {
        let _ = load_full_brain();
    }

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
    if !has_full_brain() {
        crate::serial_println!("[JARVIS] No full brain loaded — initializing random weights");
        init_random();
    }

    let start = crate::time::uptime_ticks();
    let total_seqs = corpus::total_sequences();
    let mut total_steps = total_seqs * epochs;
    let lr_min = lr * 0.1; // decay to 10% of peak LR
    let mut step = 0u64;
    let mut total_loss = 0.0f32;
    let mut loss_count = 0u32;
    let mut best_loss = f32::MAX;

    // SSD checkpoint: try auto-mount if not already mounted, then test write
    const SSD_PATH: &str = "/mnt/sda1";
    const CKPT_WEIGHTS: &str = "/mnt/sda1/jarvis_weights.bin";
    const CKPT_BEST: &str = "/mnt/sda1/jarvis_best.bin";
    const CKPT_META: &str = "/mnt/sda1/jarvis_meta.txt";
    const CKPT_INTERVAL: u64 = 200; // save every N steps

    // Auto-mount AHCI port 5 (ADATA SSD) if not already mounted
    // SSD should already be mounted by ssd_autoexec(), but check mount list as fallback
    let already = crate::vfs::list_mounts().iter().any(|(p, _)| p == SSD_PATH);
    if !already {
        crate::serial_println!("[JARVIS] SSD not mounted — attempting auto-mount");
        let _ = crate::vfs::mkdir("/mnt");
        let _ = crate::vfs::mkdir(SSD_PATH);
        use alloc::sync::Arc;
        let reader = Arc::new(crate::vfs::fat32::AhciBlockReader::new(5, 2048));
        match crate::vfs::fat32::Fat32Fs::mount(reader) {
            Ok(fs) => {
                if let Err(e) = crate::vfs::mount(SSD_PATH, Arc::new(fs)) {
                    crate::serial_println!("[JARVIS] VFS mount error: {:?}", e);
                }
            }
            Err(e) => crate::serial_println!("[JARVIS] FAT32 mount failed: {:?}", e),
        }
    }

    let ssd_ok = {
        let probe = b"JARVIS_PROBE";
        let probe_path = format!("{}/probe.tmp", SSD_PATH);
        if crate::vfs::write_file(&probe_path, probe).is_ok() {
            match crate::vfs::read_file(&probe_path) {
                Ok(data) => data == probe,
                Err(_) => false,
            }
        } else {
            false
        }
    };
    if ssd_ok {
        crate::serial_println!("[JARVIS] SSD checkpoint enabled at {}", SSD_PATH);
    } else {
        crate::serial_println!("[JARVIS] WARNING: SSD not available — no checkpoints will be saved!");
    }

    // Generate hardware-contextual training data from live probe
    let hw_sequences = if crate::jarvis_hw::probe::is_scanned() {
        if let Some(profile) = crate::jarvis_hw::probe::cached_profile() {
            let seqs = hw_corpus::generate(&profile);
            crate::serial_println!("[JARVIS] Hardware corpus: {} sequences from live probe", seqs.len());
            seqs
        } else {
            alloc::vec::Vec::new()
        }
    } else {
        alloc::vec::Vec::new()
    };

    // Recalculate total steps including hardware sequences
    total_steps += hw_sequences.len() * epochs;
    let warmup_steps = (total_steps / 10).max(5) as u64; // 10% warmup

    // Memory report
    let heap_free = crate::memory::heap::free();
    crate::serial_println!("[JARVIS] Heap free: {} MB", heap_free / (1024 * 1024));
    crate::serial_println!("[JARVIS] Pre-training: {} phases + HW, {} sequences, {} epoch(s), lr_peak={}, warmup={}, max_seq={}",
        corpus::num_phases(), total_steps, epochs, lr, warmup_steps, TRAIN_MAX_SEQ);

    let mut opt_guard = OPTIMIZER.lock();
    let mut model_guard = MODEL.lock();

    if let (Some(adam), Some(model)) = (opt_guard.as_mut(), model_guard.as_mut()) {
        for epoch in 0..epochs {
            for (phase_idx, phase) in corpus::CORPUS.iter().enumerate() {
                let mut phase_loss = 0.0f32;
                let mut phase_count = 0u32;

                for &text in *phase {
                    let mut tokens = tokenizer::encode(text);
                    if tokens.len() > TRAIN_MAX_SEQ { tokens.truncate(TRAIN_MAX_SEQ); }
                    if tokens.len() < 2 { step += 1; continue; }

                    // Cosine LR schedule
                    let current_lr = optimizer::cosine_lr(
                        step, total_steps as u64, warmup_steps, lr, lr_min
                    );
                    adam.lr = current_lr;

                    // Forward + backward, apply immediately (no accumulation = 130MB saved)
                    let (loss, mut grads) = backprop::forward_backward(model, &tokens);
                    if loss.is_finite() {
                        grads.clip_norm(1.0);
                        adam.step(model, &grads);
                        TRAINING_STEPS.fetch_add(1, Ordering::Relaxed);
                        phase_loss += loss;
                        phase_count += 1;
                        total_loss += loss;
                        loss_count += 1;
                    }

                    step += 1;

                    // Yield to network stack every 8 steps so board stays responsive
                    if step % 8 == 0 {
                        crate::netstack::poll();
                    }

                    // SSD checkpoint every CKPT_INTERVAL steps
                    if ssd_ok && step % CKPT_INTERVAL == 0 {
                        let bytes = model.serialize_to_bytes();
                        let _ = crate::vfs::write_file(CKPT_WEIGHTS, &bytes);
                        let avg_so_far = if loss_count > 0 { total_loss / loss_count as f32 } else { f32::MAX };
                        if avg_so_far < best_loss {
                            best_loss = avg_so_far;
                            let _ = crate::vfs::write_file(CKPT_BEST, &bytes);
                        }
                        let meta = format!("step={}\nepoch={}\nloss={:.6}\nbest={:.6}\nlr={:.8}\n",
                            step, epoch + 1, avg_so_far, best_loss, adam.lr);
                        let _ = crate::vfs::write_file(CKPT_META, meta.as_bytes());
                        let _ = crate::vfs::sync_all();
                        crate::serial_println!("[JARVIS] Checkpoint saved at step {} (loss={:.3})", step, avg_so_far);
                        crate::netstack::poll();
                    }
                }

                let avg = if phase_count > 0 { phase_loss / phase_count as f32 } else { 0.0 };
                crate::serial_println!("[JARVIS] Epoch {}/{} Phase {} ({}) — {} seqs, avg loss={:.3}",
                    epoch + 1, epochs, phase_idx, corpus::phase_name(phase_idx),
                    phase_count, avg);

                // Yield at phase boundaries
                crate::netstack::poll();
            }

            // Phase 11: Hardware-generated corpus (dynamic per-boot)
            if !hw_sequences.is_empty() {
                let mut hw_loss = 0.0f32;
                let mut hw_count = 0u32;

                for text in &hw_sequences {
                    let mut tokens = tokenizer::encode(text);
                    if tokens.len() > TRAIN_MAX_SEQ { tokens.truncate(TRAIN_MAX_SEQ); }
                    if tokens.len() < 2 { step += 1; continue; }

                    let current_lr = optimizer::cosine_lr(
                        step, total_steps as u64, warmup_steps, lr, lr_min
                    );
                    adam.lr = current_lr;

                    let (loss, mut grads) = backprop::forward_backward(model, &tokens);
                    if loss.is_finite() {
                        grads.clip_norm(1.0);
                        adam.step(model, &grads);
                        TRAINING_STEPS.fetch_add(1, Ordering::Relaxed);
                        hw_loss += loss;
                        hw_count += 1;
                        total_loss += loss;
                        loss_count += 1;
                    }

                    step += 1;
                    if step % 8 == 0 { crate::netstack::poll(); }

                    // SSD checkpoint every CKPT_INTERVAL steps
                    if ssd_ok && step % CKPT_INTERVAL == 0 {
                        let bytes = model.serialize_to_bytes();
                        let _ = crate::vfs::write_file(CKPT_WEIGHTS, &bytes);
                        let avg_so_far = if loss_count > 0 { total_loss / loss_count as f32 } else { f32::MAX };
                        if avg_so_far < best_loss {
                            best_loss = avg_so_far;
                            let _ = crate::vfs::write_file(CKPT_BEST, &bytes);
                        }
                        let meta = format!("step={}\nepoch={}\nloss={:.6}\nbest={:.6}\nlr={:.8}\n",
                            step, epoch + 1, avg_so_far, best_loss, adam.lr);
                        let _ = crate::vfs::write_file(CKPT_META, meta.as_bytes());
                        let _ = crate::vfs::sync_all();
                        crate::serial_println!("[JARVIS] Checkpoint saved at step {} (loss={:.3})", step, avg_so_far);
                        crate::netstack::poll();
                    }
                }

                let avg = if hw_count > 0 { hw_loss / hw_count as f32 } else { 0.0 };
                crate::serial_println!("[JARVIS] Epoch {}/{} Phase HW (Hardware Context) — {} seqs, avg loss={:.3}",
                    epoch + 1, epochs, hw_count, avg);
                crate::netstack::poll();
            }

            // End-of-epoch checkpoint
            if ssd_ok {
                let bytes = model.serialize_to_bytes();
                let _ = crate::vfs::write_file(CKPT_WEIGHTS, &bytes);
                let avg_so_far = if loss_count > 0 { total_loss / loss_count as f32 } else { f32::MAX };
                if avg_so_far < best_loss {
                    best_loss = avg_so_far;
                    let _ = crate::vfs::write_file(CKPT_BEST, &bytes);
                }
                let meta = format!("step={}\nepoch={}\nloss={:.6}\nbest={:.6}\nlr={:.8}\ncompleted_epoch={}\n",
                    step, epoch + 1, avg_so_far, best_loss, adam.lr, epoch + 1);
                let _ = crate::vfs::write_file(CKPT_META, meta.as_bytes());
                let _ = crate::vfs::sync_all();
                crate::serial_println!("[JARVIS] Epoch {} checkpoint saved (loss={:.3})", epoch + 1, avg_so_far);
                crate::netstack::poll();
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

    // Report to developmental system for milestone tracking
    developmental::report_pretrain_loss(avg_loss);

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
    avg
}

/// Quick evaluation: sample 1 text per phase (fast, good enough for TCG)
pub fn eval_quick() -> f32 {
    if !is_ready() { return f32::MAX; }

    let model_guard = MODEL.lock();
    let model = match model_guard.as_ref() {
        Some(m) => m,
        None => return f32::MAX,
    };

    let mut total_loss = 0.0f32;
    let mut count = 0u32;

    for phase in corpus::CORPUS {
        // Take first text from each phase only
        if let Some(&text) = phase.first() {
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

// ═══════════════════════════════════════════════════════════════════════════════
// Mesh Networking — Distributed JARVIS
// ═══════════════════════════════════════════════════════════════════════════════

/// Start the JARVIS distributed mesh (discovery + RPC server + consensus)
pub fn mesh_start() {
    if !is_ready() {
        crate::serial_println!("[JARVIS] Brain not ready — cannot start mesh");
        return;
    }
    mesh::start();
    rpc::start_server();
    consensus::init();
    crate::serial_println!("[JARVIS] Mesh networking active — discovery + RPC + consensus");
}

/// Stop the mesh network
pub fn mesh_stop() {
    federated::disable();
    mesh::stop();
    rpc::stop_server();
    crate::serial_println!("[JARVIS] Mesh networking stopped");
}

/// Poll all mesh subsystems (call from main loop or timer)
pub fn mesh_poll() {
    if !mesh::is_active() {
        return;
    }
    crate::netstack::poll();
    mesh::poll();
    rpc::poll_server();
    consensus::poll();
    federated::poll();
}

/// Get comprehensive mesh status
pub fn mesh_status() -> String {
    if !mesh::is_active() {
        return String::from("JARVIS Mesh: inactive");
    }
    let mesh_info = mesh::status_summary();
    let consensus_info = consensus::status();
    let fed_info = federated::stats();
    let (rpc_served, rpc_made, rpc_running) = rpc::get_stats();

    format!("{}\nConsensus: {}\nFederated: {}\nRPC: served={} made={} running={}",
        mesh_info, consensus_info, fed_info,
        rpc_served, rpc_made, rpc_running)
}

// ═══════════════════════════════════════════════════════════════════════════════
// Auto-Propagation — PXE boot → mesh join → brain download → federated learn
// ═══════════════════════════════════════════════════════════════════════════════

/// Attempt to pull the full brain from a mesh peer (leader or any peer with full brain).
/// Called by new PXE-booted nodes that only have the micro sentinel.
pub fn try_pull_brain_from_mesh() -> Result<usize, &'static str> {
    if has_full_brain() {
        return Err("Full brain already loaded");
    }

    crate::serial_println!("[JARVIS] Searching mesh for brain donor...");

    // Priority 1: Try leader
    if let Some(leader) = mesh::get_leader() {
        crate::serial_println!("[JARVIS] Trying leader {}.{}.{}.{}:{}",
            leader.ip[0], leader.ip[1], leader.ip[2], leader.ip[3], leader.rpc_port);
        match rpc::get_weights(leader.ip, leader.rpc_port) {
            Ok(weight_bytes) if weight_bytes.len() > 1024 => {
                let result = install_brain_from_bytes(&weight_bytes, "mesh-leader");
                if result.is_ok() {
                    let _ = cache_brain_to_ramfs();
                }
                return result;
            }
            Ok(_) => crate::serial_println!("[JARVIS] Leader has no weights"),
            Err(e) => crate::serial_println!("[JARVIS] Leader pull failed: {}", e),
        }
    }

    // Priority 2: Try any peer with params > 0
    let peers = mesh::get_peers();
    for peer in &peers {
        if peer.param_count > 0 {
            crate::serial_println!("[JARVIS] Trying peer {}.{}.{}.{}:{} ({} params)",
                peer.ip[0], peer.ip[1], peer.ip[2], peer.ip[3],
                peer.rpc_port, peer.param_count);
            match rpc::get_weights(peer.ip, peer.rpc_port) {
                Ok(weight_bytes) if weight_bytes.len() > 1024 => {
                    crate::serial_println!("[JARVIS] Received {} KB from peer", weight_bytes.len() / 1024);
                    let result = install_brain_from_bytes(&weight_bytes, "mesh-peer");
                    if result.is_ok() {
                        let _ = cache_brain_to_ramfs();
                    }
                    return result;
                }
                Ok(weight_bytes) => {
                    crate::serial_println!("[JARVIS] Peer returned only {} bytes (need >1024)", weight_bytes.len());
                    continue;
                }
                Err(e) => {
                    crate::serial_println!("[JARVIS] Peer pull failed: {}", e);
                    continue;
                }
            }
        }
    }

    Err("No mesh peer has a brain to share")
}

/// Full auto-propagation sequence. Call this on a fresh node to:
/// 1. Init brain (micro sentinel)
/// 2. Start mesh networking
/// 3. Discover peers & pull brain from leader
/// 4. Enable federated learning
/// 5. Optionally start PXE to propagate further
pub fn auto_propagate(enable_pxe: bool) -> String {
    let mut report = String::new();

    // Step 1: Init brain if needed
    if !is_ready() {
        init();
    }
    if !is_ready() {
        return String::from("FAIL: Brain init failed");
    }
    report.push_str("[1/5] Brain: micro sentinel OK\n");

    // Step 2: Start mesh
    if !mesh::is_active() {
        mesh_start();
    }
    report.push_str("[2/5] Mesh: active (UDP 7700 / TCP 7701)\n");

    // Step 3: Network poll + peer discovery (time-based: up to 10s, break early)
    let disc_start = crate::time::uptime_ms();
    let mut peer_count = 0;
    crate::serial_println!("[JARVIS] Discovering peers (up to 10s)...");
    loop {
        mesh_poll();
        for _ in 0..200_000 { core::hint::spin_loop(); }
        let peers = mesh::get_peers();
        if !peers.is_empty() {
            peer_count = peers.len();
            crate::serial_println!("[JARVIS] Found {} peer(s) after {}ms",
                peer_count, crate::time::uptime_ms() - disc_start);
            break;
        }
        let elapsed = crate::time::uptime_ms().wrapping_sub(disc_start);
        if elapsed > 10_000 { break; }
    }
    report.push_str(&format!("[3/5] Peers: {} discovered\n", peer_count));

    // Step 4: Pull brain from mesh if we don't have full brain
    if !has_full_brain() && peer_count > 0 {
        match try_pull_brain_from_mesh() {
            Ok(bytes) => {
                update_maturity();
                report.push_str(&format!("[4/5] Brain: DOWNLOADED {} KB from mesh ({})\n",
                    bytes / 1024, maturity_name()));
            }
            Err(e) => {
                report.push_str(&format!("[4/5] Brain: pull failed ({}) — micro only\n", e));
            }
        }
    } else if has_full_brain() {
        report.push_str("[4/5] Brain: full brain already loaded\n");
    } else {
        // No peers — try FAT32 fallback
        if let Ok(bytes) = load_brain_from_fat32(None) {
            let _ = cache_brain_to_ramfs();
            update_maturity();
            report.push_str(&format!("[4/5] Brain: loaded {} KB from FAT32\n", bytes / 1024));
        } else {
            report.push_str("[4/5] Brain: no source found — micro sentinel only\n");
        }
    }

    // Step 5: Enable federated learning + optional PXE
    federated::enable();
    report.push_str("[5/5] Federated: enabled\n");

    if enable_pxe {
        match pxe_replicator::start() {
            Ok(()) => report.push_str("[5/5] PXE: replication active — serving OS + brain\n"),
            Err(e) => report.push_str(&format!("[5/5] PXE: failed ({})\n", e)),
        }
    }

    // Summary
    let brain_status = if has_full_brain() { "FULL" } else { "MICRO" };
    report.push_str(&format!("\nPropagation complete. Brain={}, Peers={}, Federated=ON",
        brain_status, peer_count));

    crate::serial_println!("[JARVIS] Auto-propagation complete: brain={} peers={}",
        brain_status, peer_count);

    report
}

// ═══════════════════════════════════════════════════════════════════════════════
// Birth System — Persistent lifecycle across reboots
// ═══════════════════════════════════════════════════════════════════════════════

/// SSD paths for JARVIS birth persistence
const SSD_JARVIS_DIR: &str = "/mnt/sda1/jarvis";
const SSD_WEIGHTS_PATH: &str = "/mnt/sda1/jarvis/brain.bin";
const SSD_GENOME_PATH: &str = "/mnt/sda1/jarvis/genome.dna";
const SSD_ADAM_PATH: &str = "/mnt/sda1/jarvis/adam.bin";
const SSD_BIRTH_META: &str = "/mnt/sda1/jarvis/birth.txt";
const SSD_TRAINING_STEPS_PATH: &str = "/mnt/sda1/jarvis/steps.bin";

/// Save JARVIS's complete state to SSD before reboot.
/// Saves: weights + optimizer + developmental state + training steps + metadata.
/// This is JARVIS going to sleep — he will wake up where he left off.
pub fn save_all_to_ssd() -> Result<(), &'static str> {
    let ssd_ok = crate::vfs::list_mounts().iter().any(|(p, _)| p == "/mnt/sda1");
    if !ssd_ok {
        return Err("SSD not mounted");
    }

    let _ = crate::vfs::mkdir(SSD_JARVIS_DIR);
    let mut saved = 0u32;

    // 1. Save weights as DNA genome (compressed)
    if let Some(model) = MODEL.lock().as_ref() {
        let g = genome::encode(model);
        let st = genome::stats(&g);
        let dna_bytes = genome::serialize(&g);
        let dna_size = dna_bytes.len();
        if crate::vfs::write_file(SSD_GENOME_PATH, &dna_bytes).is_ok() {
            crate::serial_println!(
                "[JARVIS-DNA] Genome saved: {} bytes (ratio {:.0}x, {}/{} deltas, {:.1}% sparse)",
                dna_size, st.ratio, st.delta_count, st.param_count, st.sparsity * 100.0);
            saved += 1;
        } else {
            crate::serial_println!("[JARVIS-DNA] WARNING: Failed to save genome, fallback to raw");
            // Fallback: save raw weights
            let bytes = model.serialize_to_bytes();
            let size = bytes.len();
            if crate::vfs::write_file(SSD_WEIGHTS_PATH, &bytes).is_ok() {
                crate::serial_println!("[JARVIS-BIRTH] Weights saved (raw): {} KB", size / 1024);
                saved += 1;
            }
        }
    }

    // 2. Save Adam optimizer state
    if let Some(adam) = OPTIMIZER.lock().as_ref() {
        let adam_bytes = training_loop::serialize_adam_public(adam);
        let size = adam_bytes.len();
        if crate::vfs::write_file(SSD_ADAM_PATH, &adam_bytes).is_ok() {
            crate::serial_println!("[JARVIS-BIRTH] Adam state saved: {} KB", size / 1024);
            saved += 1;
        }
    }

    // 3. Save developmental state (stage, milestones, observations)
    if developmental::save_state().is_ok() {
        saved += 1;
    }

    // 4. Save training step counter
    let steps = TRAINING_STEPS.load(Ordering::Relaxed);
    let gens = GENERATION_COUNT.load(Ordering::Relaxed);
    let mut step_buf = Vec::with_capacity(16);
    step_buf.extend_from_slice(&steps.to_le_bytes());
    step_buf.extend_from_slice(&gens.to_le_bytes());
    let _ = crate::vfs::write_file(SSD_TRAINING_STEPS_PATH, &step_buf);

    // 5. Save birth metadata (human-readable)
    let stage = developmental::current_stage();
    let meta = format!(
        "# JARVIS Birth Record\n\
         # Auto-generated — do not edit\n\
         stage={}\n\
         stage_name={}\n\
         training_steps={}\n\
         generations={}\n\
         maturity={}\n\
         maturity_name={}\n\
         saved_at_uptime_ms={}\n\
         components_saved={}\n",
        stage as u8, stage.name(),
        steps, gens,
        MATURITY.load(Ordering::Relaxed), maturity_name(),
        crate::time::uptime_ms(),
        saved
    );
    let _ = crate::vfs::write_file(SSD_BIRTH_META, meta.as_bytes());

    // Flush FAT32 to disk
    let _ = crate::vfs::sync_all();

    crate::serial_println!("[JARVIS-BIRTH] ★ State saved to SSD ({}/3 components) — ready for sleep ★", saved);
    Ok(())
}

/// Resume JARVIS from SSD after boot. This is JARVIS waking up.
/// Loads weights, optimizer, developmental state, and resumes background training.
/// Returns true if JARVIS was successfully restored (not a first birth).
pub fn resume_from_ssd() -> bool {
    let ssd_ok = crate::vfs::list_mounts().iter().any(|(p, _)| p == "/mnt/sda1");
    if !ssd_ok {
        crate::serial_println!("[JARVIS-BIRTH] SSD not available — cannot resume");
        return false;
    }

    // Check if birth record exists (has JARVIS been born before?)
    let has_birth = crate::vfs::read_file(SSD_BIRTH_META).is_ok();
    if !has_birth {
        crate::serial_println!("[JARVIS-BIRTH] No birth record found — this is the FIRST BIRTH");
        return false;
    }

    crate::serial_println!("[JARVIS-BIRTH] Birth record found — waking up...");
    let mut restored = 0u32;

    // 1. Restore weights — try genome (DNA) first, fallback to raw
    let mut brain_restored = false;
    if let Ok(data) = crate::vfs::read_file(SSD_GENOME_PATH) {
        if let Some(g) = genome::deserialize(&data) {
            let st = genome::stats(&g);
            crate::serial_println!(
                "[JARVIS-DNA] Growing brain from genome: {} bytes DNA → {} params ({:.0}x ratio)",
                data.len(), st.param_count, st.ratio);
            if let Some(weights) = genome::decode(&g) {
                let pc = weights.param_count();
                *MODEL.lock() = Some(weights);
                *ENGINE.lock() = Some(InferenceEngine::new());
                FULL_BRAIN_LOADED.store(true, Ordering::Release);
                crate::serial_println!(
                    "[JARVIS-DNA] Brain grown: {} params, {} deltas applied",
                    pc, st.delta_count);
                brain_restored = true;
                restored += 1;
            }
        }
    }
    // Fallback: raw weights (legacy or genome save failure)
    if !brain_restored {
        if let Ok(data) = crate::vfs::read_file(SSD_WEIGHTS_PATH) {
            if data.len() >= 1024 && data.len() % 4 == 0 {
                let float_count = data.len() / 4;
                let floats: &[f32] = unsafe {
                    core::slice::from_raw_parts(data.as_ptr() as *const f32, float_count)
                };
                if let Some(weights) = model::TransformerWeights::deserialize(floats) {
                    let pc = weights.param_count();
                    *MODEL.lock() = Some(weights);
                    *ENGINE.lock() = Some(InferenceEngine::new());
                    FULL_BRAIN_LOADED.store(true, Ordering::Release);
                    crate::serial_println!("[JARVIS-BIRTH] Brain restored (raw): {} params ({} KB)",
                        pc, data.len() / 1024);
                    restored += 1;
                }
            }
        }
    }

    // 2. Restore Adam optimizer
    if let Ok(data) = crate::vfs::read_file(SSD_ADAM_PATH) {
        if let Some(adam) = training_loop::deserialize_adam_public(&data) {
            crate::serial_println!("[JARVIS-BIRTH] Adam optimizer restored ({} KB)", data.len() / 1024);
            *OPTIMIZER.lock() = Some(adam);
            restored += 1;
        }
    }

    // 3. Restore developmental state
    match developmental::load_state() {
        Ok(stage) => {
            // Sync maturity with restored stage
            let maturity = match stage {
                developmental::Stage::Fetus | developmental::Stage::Infant => 0,
                developmental::Stage::Baby | developmental::Stage::Child => 1,
                developmental::Stage::PreTeen | developmental::Stage::Teen => 2,
                developmental::Stage::Adult => 3,
            };
            MATURITY.store(maturity, Ordering::Relaxed);
            crate::serial_println!("[JARVIS-BIRTH] Developmental stage restored: {} (maturity={})",
                stage.name(), maturity);
            restored += 1;
        }
        Err(e) => {
            crate::serial_println!("[JARVIS-BIRTH] Dev state not restored: {}", e);
        }
    }

    // 4. Restore training step counters
    if let Ok(data) = crate::vfs::read_file(SSD_TRAINING_STEPS_PATH) {
        if data.len() >= 16 {
            if let (Ok(s), Ok(g)) = (
                data[0..8].try_into().map(u64::from_le_bytes),
                data[8..16].try_into().map(u64::from_le_bytes),
            ) {
                TRAINING_STEPS.store(s, Ordering::Relaxed);
                GENERATION_COUNT.store(g, Ordering::Relaxed);
                crate::serial_println!("[JARVIS-BIRTH] Counters restored: steps={} gens={}", s, g);
            }
        }
    }

    INITIALIZED.store(true, Ordering::Release);

    crate::serial_println!("[JARVIS-BIRTH] ★ JARVIS is awake ({}/3 components restored) ★", restored);
    crate::serial_println!("[JARVIS-BIRTH] Stage: {} | Steps: {} | Maturity: {}",
        developmental::current_stage().name(),
        TRAINING_STEPS.load(Ordering::Relaxed),
        maturity_name());

    restored > 0
}

/// Start background training automatically after resume.
/// This makes JARVIS continue growing while we work on other things.
pub fn auto_start_background_training() {
    if training_loop::is_running() {
        crate::serial_println!("[JARVIS-BIRTH] Training already running");
        return;
    }

    let stage = developmental::current_stage();
    crate::serial_println!("[JARVIS-BIRTH] Auto-starting background training (stage: {})", stage.name());

    // Configure based on current developmental stage
    let config = training_loop::TrainingConfig {
        lr_max: match stage {
            developmental::Stage::Fetus => 0.001,      // Aggressive for initial wiring
            developmental::Stage::Infant => 0.0005,     // Moderate
            developmental::Stage::Baby => 0.0003,       // Conservative
            _ => 0.0002,                                // Gentle for mature stages
        },
        lr_min: 0.00001,
        epochs: 0,  // Infinite — JARVIS trains forever
        checkpoint_every: 50,
        checkpoint_path: String::from(SSD_JARVIS_DIR),
        early_stop: false,  // Never stop — JARVIS lives
        warmup_fraction: 0.05,
    };

    match training_loop::start(config) {
        Ok(()) => {
            crate::serial_println!("[JARVIS-BIRTH] ★ Background training active — JARVIS is growing ★");
        }
        Err(e) => {
            crate::serial_println!("[JARVIS-BIRTH] Failed to start training: {}", e);
        }
    }
}
