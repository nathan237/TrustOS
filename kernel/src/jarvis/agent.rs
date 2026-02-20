//! Self-Aware Agent — Jarvis can introspect, execute, and evolve
//!
//! This module gives Jarvis the ability to:
//! - Execute shell commands and observe the output
//! - Read its own source code via the VFS
//! - Describe its own architecture and state
//! - Benchmark itself and adapt parameters
//! - Suggest improvements to its own training
//!
//! Agent loop:
//! ```text
//! 1. Receive input (from user or mentor)
//! 2. Classify: generation | introspection | command | training
//! 3. Execute appropriate action
//! 4. Observe result
//! 5. Optionally: learn from the interaction
//! ```

use alloc::string::String;
use alloc::format;
use alloc::vec::Vec;

use super::model;

// ═══════════════════════════════════════════════════════════════════════════════
// Agent Actions
// ═══════════════════════════════════════════════════════════════════════════════

/// Actions the agent can take
#[derive(Debug)]
pub enum AgentAction {
    /// Generate text using the neural model
    Generate { prompt: String, max_tokens: usize },
    /// Execute a shell command and capture output
    Execute { command: String },
    /// Introspect: report own state
    Introspect { what: IntrospectTarget },
    /// Train on provided text
    Train { text: String, lr: f32 },
    /// Respond with a static message (no model needed)
    StaticResponse { message: String },
}

/// What to introspect
#[derive(Debug)]
pub enum IntrospectTarget {
    /// Model architecture and parameters
    Architecture,
    /// Current weights statistics (mean, std per layer)
    WeightStats,
    /// Training history and loss
    TrainingHistory,
    /// Available hardware (GPU, memory)
    Hardware,
    /// Source code of a module
    SourceCode { module: String },
    /// Everything
    Full,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Intent Classification (for agent routing)
// ═══════════════════════════════════════════════════════════════════════════════

/// Classify user input into an agent action
pub fn classify_input(input: &str) -> AgentAction {
    let lower = input.to_ascii_lowercase();

    // ── Introspection commands ──
    if lower.starts_with("introspect") || lower.starts_with("describe yourself")
        || lower.contains("qui es tu") || lower.contains("architecture")
        || lower.starts_with("self") {
        return AgentAction::Introspect { what: IntrospectTarget::Architecture };
    }

    if lower.starts_with("weights") || lower.contains("poids") || lower.contains("weight stats") {
        return AgentAction::Introspect { what: IntrospectTarget::WeightStats };
    }

    if lower.starts_with("hardware") || lower.contains("gpu") || lower.contains("materiel") {
        return AgentAction::Introspect { what: IntrospectTarget::Hardware };
    }

    if lower.starts_with("source") || lower.starts_with("code") || lower.contains("ton code") {
        let module = if lower.contains("model") { "model" }
            else if lower.contains("infer") { "inference" }
            else if lower.contains("agent") { "agent" }
            else if lower.contains("train") { "training" }
            else { "mod" };
        return AgentAction::Introspect {
            what: IntrospectTarget::SourceCode { module: String::from(module) }
        };
    }

    // ── Training commands ──
    if lower.starts_with("train on ") || lower.starts_with("apprends ") {
        let text = if lower.starts_with("train on ") {
            &input[9..]
        } else {
            &input[9..]
        };
        return AgentAction::Train { text: String::from(text), lr: 0.001 };
    }

    if lower.starts_with("learn ") || lower.starts_with("memorize ") {
        let text = &input[lower.find(' ').unwrap_or(0) + 1..];
        return AgentAction::Train { text: String::from(text), lr: 0.001 };
    }

    // ── Shell execution ──
    if lower.starts_with("run ") || lower.starts_with("execute ") || lower.starts_with("exec ") {
        let cmd = &input[lower.find(' ').unwrap_or(0) + 1..];
        return AgentAction::Execute { command: String::from(cmd) };
    }

    // ── Default: generate with neural model ──
    AgentAction::Generate {
        prompt: String::from(input),
        max_tokens: 128,
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Introspection
// ═══════════════════════════════════════════════════════════════════════════════

/// Generate introspection report
pub fn introspect(target: &IntrospectTarget) -> Vec<String> {
    let mut lines = Vec::new();

    match target {
        IntrospectTarget::Architecture => {
            lines.push(String::from("= Jarvis Neural Architecture ="));
            lines.push(format!("Type: Decoder-only Transformer (LLaMA-style)"));
            lines.push(format!("Layers: {}", model::N_LAYERS));
            lines.push(format!("d_model: {} (embedding dimension)", model::D_MODEL));
            lines.push(format!("n_heads: {} (d_k = {})", model::N_HEADS, model::D_K));
            lines.push(format!("d_ff: {} (FFN inner dimension)", model::D_FF));
            lines.push(format!("Vocab: {} (byte-level, no BPE)", model::VOCAB_SIZE));
            lines.push(format!("Context: {} tokens max", model::MAX_SEQ));
            lines.push(format!("Normalization: RMSNorm (eps={})", model::RMS_EPS));
            lines.push(format!("FFN: SwiGLU (SiLU gate × up, then down)"));
            lines.push(format!("Positional: Learned embeddings"));
            lines.push(format!("Total parameters: ~{}K", param_count_estimate() / 1000));
            lines.push(format!("Memory (FP32): ~{} KB", param_count_estimate() * 4 / 1024));
            lines.push(String::from(""));
            lines.push(String::from("Per-layer breakdown:"));
            lines.push(format!("  RMSNorm attn:   {} params", model::D_MODEL));
            lines.push(format!("  W_q,W_k,W_v,W_o: {} params each", model::D_MODEL * model::D_MODEL));
            lines.push(format!("  RMSNorm FFN:    {} params", model::D_MODEL));
            lines.push(format!("  W_gate, W_up:   {} params each", model::D_MODEL * model::D_FF));
            lines.push(format!("  W_down:         {} params", model::D_FF * model::D_MODEL));
        }

        IntrospectTarget::WeightStats => {
            lines.push(String::from("= Weight Statistics ="));
            if let Some(model) = super::MODEL.lock().as_ref() {
                lines.push(format!("Token embed: {}", vec_stats(&model.token_embed)));
                lines.push(format!("Pos embed:   {}", vec_stats(&model.pos_embed)));
                for (i, layer) in model.layers.iter().enumerate() {
                    lines.push(format!("Layer {}:", i));
                    lines.push(format!("  W_q:    {}", vec_stats(&layer.w_q)));
                    lines.push(format!("  W_k:    {}", vec_stats(&layer.w_k)));
                    lines.push(format!("  W_v:    {}", vec_stats(&layer.w_v)));
                    lines.push(format!("  W_o:    {}", vec_stats(&layer.w_o)));
                    lines.push(format!("  W_gate: {}", vec_stats(&layer.w_gate)));
                    lines.push(format!("  W_up:   {}", vec_stats(&layer.w_up)));
                    lines.push(format!("  W_down: {}", vec_stats(&layer.w_down)));
                }
                lines.push(format!("Output:      {}", vec_stats(&model.w_output)));
            } else {
                lines.push(String::from("  Model not loaded."));
            }
        }

        IntrospectTarget::TrainingHistory => {
            lines.push(String::from("= Training History ="));
            lines.push(format!("Steps completed: {}", super::TRAINING_STEPS.load(core::sync::atomic::Ordering::Relaxed)));
            lines.push(format!("Generations:     {}", super::GENERATION_COUNT.load(core::sync::atomic::Ordering::Relaxed)));
        }

        IntrospectTarget::Hardware => {
            lines.push(String::from("= Hardware Status ="));
            let heap_used = crate::memory::heap::used();
            let heap_free = crate::memory::heap::free();
            lines.push(format!("Heap used: {} KB", heap_used / 1024));
            lines.push(format!("Heap free: {} KB", heap_free / 1024));
            lines.push(format!("CPUs: {}", crate::cpu::smp::cpu_count()));
            lines.push(format!("GPU detected: {}", crate::drivers::amdgpu::is_detected()));
            lines.push(format!("GPU compute ready: {}", crate::drivers::amdgpu::compute::is_ready()));
            lines.push(format!("Model memory: {} KB",
                if let Some(m) = super::MODEL.lock().as_ref() { m.memory_bytes() / 1024 } else { 0 }));
        }

        IntrospectTarget::SourceCode { module } => {
            lines.push(format!("= Source: jarvis/{}.rs =", module));
            lines.push(String::from("(Reading from embedded knowledge — VFS integration pending)"));
            match module.as_str() {
                "mod" => {
                    lines.push(String::from("// Module: jarvis/mod.rs"));
                    lines.push(String::from("// Public API: init(), generate(), train_on_text(), info_lines()"));
                    lines.push(String::from("// Global state: MODEL (Mutex<Option<TransformerWeights>>)"));
                    lines.push(String::from("// Submodules: tokenizer, model, inference, agent, mentor, training"));
                }
                "model" => {
                    lines.push(format!("// TransformerWeights: {}L × d{} × {}H × ff{}",
                        model::N_LAYERS, model::D_MODEL, model::N_HEADS, model::D_FF));
                    lines.push(format!("// LayerWeights: rms_attn, W_q/k/v/o, rms_ffn, W_gate/up/down"));
                    lines.push(format!("// Serialization: serialize() → Vec<f32>, deserialize(&[f32])"));
                }
                "inference" => {
                    lines.push(String::from("// InferenceEngine: KV cache, scratch buffers, PRNG"));
                    lines.push(String::from("// forward_one(): embed → N×(attn + ffn) → logits"));
                    lines.push(String::from("// sample_token(): temperature + top-k sampling"));
                    lines.push(String::from("// compute_loss(): teacher forcing cross-entropy"));
                }
                _ => {
                    lines.push(format!("// Module '{}' — structure available via introspect", module));
                }
            }
        }

        IntrospectTarget::Full => {
            lines.extend(introspect(&IntrospectTarget::Architecture));
            lines.push(String::new());
            lines.extend(introspect(&IntrospectTarget::Hardware));
            lines.push(String::new());
            lines.extend(introspect(&IntrospectTarget::TrainingHistory));
        }
    }

    lines
}

// ═══════════════════════════════════════════════════════════════════════════════
// Self-Benchmark
// ═══════════════════════════════════════════════════════════════════════════════

/// Benchmark inference speed — tokens per second
pub fn bench_inference() -> (f32, u64) {
    if !super::is_ready() { return (0.0, 0); }

    let model_guard = super::MODEL.lock();
    let model = match model_guard.as_ref() {
        Some(m) => m,
        None => return (0.0, 0),
    };

    let prompt = b"The quick brown fox jumps over";
    let max_tokens = 32;

    let start = crate::time::uptime_ticks();

    let mut engine = super::inference::InferenceEngine::new();
    engine.config.temperature = 0.5;
    let _ = engine.generate(model, prompt, max_tokens);

    let elapsed_ms = crate::time::uptime_ticks().saturating_sub(start).max(1);
    let tokens_per_sec = (max_tokens as f32 * 1000.0) / elapsed_ms as f32;

    (tokens_per_sec, elapsed_ms)
}

// ═══════════════════════════════════════════════════════════════════════════════
// Helpers
// ═══════════════════════════════════════════════════════════════════════════════

fn param_count_estimate() -> usize {
    model::VOCAB_SIZE * model::D_MODEL * 2  // Token + pos embed
    + model::N_LAYERS * (
        model::D_MODEL  // rms_attn
        + model::D_MODEL * model::D_MODEL * 4  // QKV + O
        + model::D_MODEL  // rms_ffn
        + model::D_MODEL * model::D_FF * 3  // gate + up + down
    )
    + model::D_MODEL  // rms_final
    + model::D_MODEL * model::VOCAB_SIZE  // w_output
}

/// Compute basic statistics for a weight vector
fn vec_stats(v: &[f32]) -> String {
    if v.is_empty() { return String::from("empty"); }
    let n = v.len() as f32;
    let mean = v.iter().sum::<f32>() / n;
    let var = v.iter().map(|&x| (x - mean) * (x - mean)).sum::<f32>() / n;
    let std = if var > 0.0 {
        let bits = var.to_bits();
        let guess = f32::from_bits((bits >> 1) + 0x1FBD_1DF5);
        (guess + var / guess) * 0.5
    } else {
        0.0
    };
    let min = v.iter().copied().fold(f32::INFINITY, f32::min);
    let max = v.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    format!("n={} mean={:.4} std={:.4} min={:.3} max={:.3}", v.len(), mean, std, min, max)
}
