

















use alloc::string::String;
use alloc::format;
use alloc::vec::Vec;

use super::model;






#[derive(Debug)]
pub enum AgentAction {
    
    Generate { nh: String, alx: usize },
    
    Execute { command: String },
    
    Introspect { what: IntrospectTarget },
    
    Train { text: String, lr: f32 },
    
    StaticResponse { message: String },
}


#[derive(Debug)]
pub enum IntrospectTarget {
    
    Architecture,
    
    WeightStats,
    
    TrainingHistory,
    
    Hardware,
    
    SourceCode { vn: String },
    
    Full,
}






pub fn pzu(input: &str) -> AgentAction {
    let gj = input.to_ascii_lowercase();

    
    if gj.starts_with("introspect") || gj.starts_with("describe yourself")
        || gj.contains("qui es tu") || gj.contains("architecture")
        || gj.starts_with("self") {
        return AgentAction::Introspect { what: IntrospectTarget::Architecture };
    }

    if gj.starts_with("weights") || gj.contains("poids") || gj.contains("weight stats") {
        return AgentAction::Introspect { what: IntrospectTarget::WeightStats };
    }

    if gj.starts_with("hardware") || gj.contains("gpu") || gj.contains("materiel") {
        return AgentAction::Introspect { what: IntrospectTarget::Hardware };
    }

    if gj.starts_with("source") || gj.starts_with("code") || gj.contains("ton code") {
        let vn = if gj.contains("model") { "model" }
            else if gj.contains("infer") { "inference" }
            else if gj.contains("agent") { "agent" }
            else if gj.contains("train") { "training" }
            else { "mod" };
        return AgentAction::Introspect {
            what: IntrospectTarget::SourceCode { vn: String::from(vn) }
        };
    }

    
    if gj.starts_with("train on ") || gj.starts_with("apprends ") {
        let text = if gj.starts_with("train on ") {
            &input[9..]
        } else {
            &input[9..]
        };
        return AgentAction::Train { text: String::from(text), lr: 0.001 };
    }

    if gj.starts_with("learn ") || gj.starts_with("memorize ") {
        let text = &input[gj.find(' ').unwrap_or(0) + 1..];
        return AgentAction::Train { text: String::from(text), lr: 0.001 };
    }

    
    if gj.starts_with("run ") || gj.starts_with("execute ") || gj.starts_with("exec ") {
        let cmd = &input[gj.find(' ').unwrap_or(0) + 1..];
        return AgentAction::Execute { command: String::from(cmd) };
    }

    
    AgentAction::Generate {
        nh: String::from(input),
        alx: 128,
    }
}






pub fn cli(target: &IntrospectTarget) -> Vec<String> {
    let mut lines = Vec::new();

    match target {
        IntrospectTarget::Architecture => {
            lines.push(String::from("= Jarvis Neural Architecture ="));
            lines.push(format!("Type: Decoder-only Transformer (LLaMA-style)"));
            lines.push(format!("Layers: {}", model::BB_));
            lines.push(format!("d_model: {} (embedding dimension)", model::E_));
            lines.push(format!("n_heads: {} (d_k = {})", model::GE_, model::DA_));
            lines.push(format!("d_ff: {} (FFN inner dimension)", model::Z_));
            lines.push(format!("Vocab: {} (byte-level, no BPE)", model::BI_));
            lines.push(format!("Context: {} tokens max", model::DF_));
            lines.push(format!("Normalization: RMSNorm (eps={})", model::HT_));
            lines.push(format!("FFN: SwiGLU (SiLU gate × up, then down)"));
            lines.push(format!("Positional: Learned embeddings"));
            lines.push(format!("Total parameters: ~{}K", gmb() / 1000));
            lines.push(format!("Memory (FP32): ~{} KB", gmb() * 4 / 1024));
            lines.push(String::from(""));
            lines.push(String::from("Per-layer breakdown:"));
            lines.push(format!("  RMSNorm attn:   {} params", model::E_));
            lines.push(format!("  W_q,W_k,W_v,W_o: {} params each", model::E_ * model::E_));
            lines.push(format!("  RMSNorm FFN:    {} params", model::E_));
            lines.push(format!("  W_gate, W_up:   {} params each", model::E_ * model::Z_));
            lines.push(format!("  W_down:         {} params", model::Z_ * model::E_));
        }

        IntrospectTarget::WeightStats => {
            lines.push(String::from("= Weight Statistics ="));
            if let Some(model) = super::Ay.lock().as_ref() {
                lines.push(format!("Token embed: {}", bpv(&model.token_embed)));
                lines.push(format!("Pos embed:   {}", bpv(&model.pos_embed)));
                for (i, bj) in model.layers.iter().enumerate() {
                    lines.push(format!("Layer {}:", i));
                    lines.push(format!("  W_q:    {}", bpv(&bj.w_q)));
                    lines.push(format!("  W_k:    {}", bpv(&bj.w_k)));
                    lines.push(format!("  W_v:    {}", bpv(&bj.w_v)));
                    lines.push(format!("  W_o:    {}", bpv(&bj.w_o)));
                    lines.push(format!("  W_gate: {}", bpv(&bj.w_gate)));
                    lines.push(format!("  W_up:   {}", bpv(&bj.w_up)));
                    lines.push(format!("  W_down: {}", bpv(&bj.w_down)));
                }
                lines.push(format!("Output:      {}", bpv(&model.w_output)));
            } else {
                lines.push(String::from("  Model not loaded."));
            }
        }

        IntrospectTarget::TrainingHistory => {
            lines.push(String::from("= Training History ="));
            lines.push(format!("Steps completed: {}", super::BY_.load(core::sync::atomic::Ordering::Relaxed)));
            lines.push(format!("Generations:     {}", super::FU_.load(core::sync::atomic::Ordering::Relaxed)));
        }

        IntrospectTarget::Hardware => {
            lines.push(String::from("= Hardware Status ="));
            let heap_used = crate::memory::heap::used();
            let heap_free = crate::memory::heap::free();
            lines.push(format!("Heap used: {} KB", heap_used / 1024));
            lines.push(format!("Heap free: {} KB", heap_free / 1024));
            lines.push(format!("CPUs: {}", crate::cpu::smp::cpu_count()));
            lines.push(format!("GPU detected: {}", crate::drivers::amdgpu::aud()));
            lines.push(format!("GPU compute ready: {}", crate::drivers::amdgpu::compute::is_ready()));
            lines.push(format!("Model memory: {} KB",
                if let Some(m) = super::Ay.lock().as_ref() { m.memory_bytes() / 1024 } else { 0 }));
        }

        IntrospectTarget::SourceCode { vn } => {
            lines.push(format!("= Source: jarvis/{}.rs =", vn));
            
            let pry = format!("/jarvis/src/{}.rs", vn);
            let source = crate::ramfs::bh(|fs| {
                fs.read_file(&pry).map(|d| d.to_vec()).ok()
            });
            if let Some(data) = source {
                lines.push(String::from("(Read from VFS)"));
                if let Ok(text) = core::str::from_utf8(&data) {
                    
                    for (i, line) in text.lines().enumerate() {
                        if i >= 30 {
                            lines.push(format!("  ... ({} more lines)", text.lines().count() - 30));
                            break;
                        }
                        lines.push(format!("  {}", line));
                    }
                }
            } else {
                lines.push(String::from("(From embedded knowledge)"));
                match vn.as_str() {
                    "mod" => {
                        lines.push(String::from("// Module: jarvis/mod.rs"));
                        lines.push(String::from("// Public API: init(), generate(), train_on_text(), info_lines()"));
                        lines.push(String::from("// Global state: MODEL (Mutex<Option<TransformerWeights>>)"));
                        lines.push(String::from("// Persistence: save_weights(), load_weights() → /jarvis/weights.bin"));
                        lines.push(String::from("// Submodules: tokenizer, model, inference, agent, mentor, training"));
                    }
                    "model" => {
                        lines.push(format!("// TransformerWeights: {}L × d{} × {}H × ff{}",
                            model::BB_, model::E_, model::GE_, model::Z_));
                        lines.push(format!("// LayerWeights: rms_attn, W_q/k/v/o, rms_ffn, W_gate/up/down"));
                        lines.push(format!("// Serialization: serialize() → Vec<f32>, deserialize(&[f32])"));
                        lines.push(format!("// Persistence: /jarvis/weights.bin ({} KB)",
                            gmb() * 4 / 1024));
                    }
                    "inference" => {
                        lines.push(String::from("// InferenceEngine: KV cache, scratch buffers, PRNG"));
                        lines.push(String::from("// forward_one(): embed → N×(attn + ffn) → logits"));
                        lines.push(String::from("// sample_token(): temperature + top-k sampling"));
                        lines.push(String::from("// compute_loss(): teacher forcing cross-entropy"));
                    }
                    "training" => {
                        lines.push(String::from("// train_step(): stochastic numerical gradients (1% params/step)"));
                        lines.push(String::from("// train_step_random(): evolution strategies"));
                        lines.push(String::from("// Rotates through: embeddings → W_q → W_k → W_o → W_gate → W_output"));
                        lines.push(format!("// Batch mode via MENTOR:BATCH_START/END"));
                    }
                    "agent" => {
                        lines.push(String::from("// classify_input(): route to Generate/Execute/Introspect/Train"));
                        lines.push(String::from("// introspect(): Architecture, WeightStats, Hardware, SourceCode"));
                        lines.push(String::from("// bench_inference(): measure tokens/sec"));
                    }
                    "mentor" => {
                        lines.push(String::from("// Serial mentoring: MENTOR:TEACH/CORRECT/EVAL/GENERATE/..."));
                        lines.push(String::from("// poll_serial(): non-blocking check on COM1"));
                        lines.push(String::from("// Batch training with loss accumulation"));
                        lines.push(String::from("// Save/load: /jarvis/weights.bin via ramfs"));
                    }
                    "tokenizer" => {
                        lines.push(String::from("// Byte-level tokenizer (vocab=256, zero-cost)"));
                        lines.push(String::from("// encode(text) → [BOS, byte0, byte1, ..., EOS]"));
                        lines.push(String::from("// decode(tokens) → String (lossy UTF-8)"));
                        lines.push(String::from("// Special: PAD=0x00, BOS=0x01, EOS=0x02, SEP=0x03"));
                    }
                    _ => {
                        lines.push(format!("// Module '{}' — use 'source model/inference/training/agent/mentor/tokenizer'", vn));
                    }
                }
            }
        }

        IntrospectTarget::Full => {
            lines.extend(cli(&IntrospectTarget::Architecture));
            lines.push(String::new());
            lines.extend(cli(&IntrospectTarget::Hardware));
            lines.push(String::new());
            lines.extend(cli(&IntrospectTarget::TrainingHistory));
        }
    }

    lines
}






pub fn kbk() -> (f32, u64) {
    if !super::is_ready() { return (0.0, 0); }

    let aea = super::Ay.lock();
    let model = match aea.as_ref() {
        Some(m) => m,
        None => return (0.0, 0),
    };

    let nh = b"The quick brown fox jumps over";
    let alx = 32;

    let start = crate::time::yf();

    let mut engine = super::inference::InferenceEngine::new();
    engine.config.temperature = 0.5;
    let _ = engine.generate(model, nh, alx);

    let elapsed_ms = crate::time::yf().saturating_sub(start).max(1);
    let plb = (alx as f32 * 1000.0) / elapsed_ms as f32;

    (plb, elapsed_ms)
}





fn gmb() -> usize {
    model::BI_ * model::E_ * 2  
    + model::BB_ * (
        model::E_  
        + model::E_ * model::E_ * 4  
        + model::E_  
        + model::E_ * model::Z_ * 3  
    )
    + model::E_  
    + model::E_ * model::BI_  
}


fn bpv(v: &[f32]) -> String {
    if v.is_empty() { return String::from("empty"); }
    let ae = v.len() as f32;
    let ghg = v.iter().sum::<f32>() / ae;
    let ael = v.iter().map(|&x| (x - ghg) * (x - ghg)).sum::<f32>() / ae;
    let std = if ael > 0.0 {
        let bits = ael.to_bits();
        let uc = f32::from_bits((bits >> 1) + 0x1FBD_1DF5);
        (uc + ael / uc) * 0.5
    } else {
        0.0
    };
    let min = v.iter().copied().fold(f32::INFINITY, f32::min);
    let max = v.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    format!("n={} mean={:.4} std={:.4} min={:.3} max={:.3}", v.len(), ghg, std, min, max)
}
