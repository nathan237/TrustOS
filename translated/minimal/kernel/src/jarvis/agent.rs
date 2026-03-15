

















use alloc::string::String;
use alloc::format;
use alloc::vec::Vec;

use super::model;






#[derive(Debug)]
pub enum AgentAction {
    
    Cek { aau: String, bvi: usize },
    
    Ahw { ro: String },
    
    Ajg { jwq: IntrospectTarget },
    
    Zf { text: String, aad: f32 },
    
    Diw { message: String },
}


#[derive(Debug)]
pub enum IntrospectTarget {
    
    Agj,
    
    Bat,
    
    Bup,
    
    Ip,
    
    Btg { apz: String },
    
    Bv,
}






pub fn yig(input: &str) -> AgentAction {
    let pb = input.avd();

    
    if pb.cj("introspect") || pb.cj("describe yourself")
        || pb.contains("qui es tu") || pb.contains("architecture")
        || pb.cj("self") {
        return AgentAction::Ajg { jwq: IntrospectTarget::Agj };
    }

    if pb.cj("weights") || pb.contains("poids") || pb.contains("weight stats") {
        return AgentAction::Ajg { jwq: IntrospectTarget::Bat };
    }

    if pb.cj("hardware") || pb.contains("gpu") || pb.contains("materiel") {
        return AgentAction::Ajg { jwq: IntrospectTarget::Ip };
    }

    if pb.cj("source") || pb.cj("code") || pb.contains("ton code") {
        let apz = if pb.contains("model") { "model" }
            else if pb.contains("infer") { "inference" }
            else if pb.contains("agent") { "agent" }
            else if pb.contains("train") { "training" }
            else { "mod" };
        return AgentAction::Ajg {
            jwq: IntrospectTarget::Btg { apz: String::from(apz) }
        };
    }

    
    if pb.cj("train on ") || pb.cj("apprends ") {
        let text = if pb.cj("train on ") {
            &input[9..]
        } else {
            &input[9..]
        };
        return AgentAction::Zf { text: String::from(text), aad: 0.001 };
    }

    if pb.cj("learn ") || pb.cj("memorize ") {
        let text = &input[pb.du(' ').unwrap_or(0) + 1..];
        return AgentAction::Zf { text: String::from(text), aad: 0.001 };
    }

    
    if pb.cj("run ") || pb.cj("execute ") || pb.cj("exec ") {
        let cmd = &input[pb.du(' ').unwrap_or(0) + 1..];
        return AgentAction::Ahw { ro: String::from(cmd) };
    }

    
    AgentAction::Cek {
        aau: String::from(input),
        bvi: 128,
    }
}






pub fn flw(cd: &IntrospectTarget) -> Vec<String> {
    let mut ak = Vec::new();

    match cd {
        IntrospectTarget::Agj => {
            ak.push(String::from("= Jarvis Neural Architecture ="));
            ak.push(format!("Type: Decoder-only Transformer (LLaMA-style)"));
            ak.push(format!("Layers: {}", model::AZ_));
            ak.push(format!("d_model: {} (embedding dimension)", model::E_));
            ak.push(format!("n_heads: {} (d_k = {})", model::FP_, model::CU_));
            ak.push(format!("d_ff: {} (FFN inner dimension)", model::Y_));
            ak.push(format!("Vocab: {} (byte-level, no BPE)", model::BG_));
            ak.push(format!("Context: {} tokens max", model::CY_));
            ak.push(format!("Normalization: RMSNorm (eps={})", model::HC_));
            ak.push(format!("FFN: SwiGLU (SiLU gate × up, then down)"));
            ak.push(format!("Positional: Learned embeddings"));
            ak.push(format!("Total parameters: ~{}K", lsa() / 1000));
            ak.push(format!("Memory (FP32): ~{} KB", lsa() * 4 / 1024));
            ak.push(String::from(""));
            ak.push(String::from("Per-layer breakdown:"));
            ak.push(format!("  RMSNorm attn:   {} params", model::E_));
            ak.push(format!("  W_q,W_k,W_v,W_o: {} params each", model::E_ * model::E_));
            ak.push(format!("  RMSNorm FFN:    {} params", model::E_));
            ak.push(format!("  W_gate, W_up:   {} params each", model::E_ * model::Y_));
            ak.push(format!("  W_down:         {} params", model::Y_ * model::E_));
        }

        IntrospectTarget::Bat => {
            ak.push(String::from("= Weight Statistics ="));
            if let Some(model) = super::Ci.lock().as_ref() {
                ak.push(format!("Token embed: {}", dxl(&model.bpa)));
                ak.push(format!("Pos embed:   {}", dxl(&model.cgq)));
                for (a, fl) in model.my.iter().cf() {
                    ak.push(format!("Layer {}:", a));
                    ak.push(format!("  W_q:    {}", dxl(&fl.biw)));
                    ak.push(format!("  W_k:    {}", dxl(&fl.biu)));
                    ak.push(format!("  W_v:    {}", dxl(&fl.bpg)));
                    ak.push(format!("  W_o:    {}", dxl(&fl.biv)));
                    ak.push(format!("  W_gate: {}", dxl(&fl.bit)));
                    ak.push(format!("  W_up:   {}", dxl(&fl.bpf)));
                    ak.push(format!("  W_down: {}", dxl(&fl.bpe)));
                }
                ak.push(format!("Output:      {}", dxl(&model.bft)));
            } else {
                ak.push(String::from("  Model not loaded."));
            }
        }

        IntrospectTarget::Bup => {
            ak.push(String::from("= Training History ="));
            ak.push(format!("Steps completed: {}", super::BW_.load(core::sync::atomic::Ordering::Relaxed)));
            ak.push(format!("Generations:     {}", super::FF_.load(core::sync::atomic::Ordering::Relaxed)));
        }

        IntrospectTarget::Ip => {
            ak.push(String::from("= Hardware Status ="));
            let afa = crate::memory::heap::mr();
            let buv = crate::memory::heap::aez();
            ak.push(format!("Heap used: {} KB", afa / 1024));
            ak.push(format!("Heap free: {} KB", buv / 1024));
            ak.push(format!("CPUs: {}", crate::cpu::smp::aao()));
            ak.push(format!("GPU detected: {}", crate::drivers::amdgpu::clb()));
            ak.push(format!("GPU compute ready: {}", crate::drivers::amdgpu::compute::uc()));
            ak.push(format!("Model memory: {} KB",
                if let Some(ef) = super::Ci.lock().as_ref() { ef.omv() / 1024 } else { 0 }));
        }

        IntrospectTarget::Btg { apz } => {
            ak.push(format!("= Source: jarvis/{}.rs =", apz));
            
            let xrn = format!("/jarvis/src/{}.rs", apz);
            let iy = crate::ramfs::fh(|fs| {
                fs.mq(&xrn).map(|bc| bc.ip()).bq()
            });
            if let Some(f) = iy {
                ak.push(String::from("(Read from VFS)"));
                if let Ok(text) = core::str::jg(&f) {
                    
                    for (a, line) in text.ak().cf() {
                        if a >= 30 {
                            ak.push(format!("  ... ({} more lines)", text.ak().az() - 30));
                            break;
                        }
                        ak.push(format!("  {}", line));
                    }
                }
            } else {
                ak.push(String::from("(From embedded knowledge)"));
                match apz.as_str() {
                    "mod" => {
                        ak.push(String::from("// Module: jarvis/mod.rs"));
                        ak.push(String::from("// Public API: init(), generate(), train_on_text(), info_lines()"));
                        ak.push(String::from("// Global state: MODEL (Mutex<Option<TransformerWeights>>)"));
                        ak.push(String::from("// Persistence: save_weights(), load_weights() → /jarvis/weights.bin"));
                        ak.push(String::from("// Submodules: tokenizer, model, inference, agent, mentor, training"));
                    }
                    "model" => {
                        ak.push(format!("// TransformerWeights: {}L × d{} × {}H × ff{}",
                            model::AZ_, model::E_, model::FP_, model::Y_));
                        ak.push(format!("// LayerWeights: rms_attn, W_q/k/v/o, rms_ffn, W_gate/up/down"));
                        ak.push(format!("// Serialization: serialize() → Vec<f32>, deserialize(&[f32])"));
                        ak.push(format!("// Persistence: /jarvis/weights.bin ({} KB)",
                            lsa() * 4 / 1024));
                    }
                    "inference" => {
                        ak.push(String::from("// InferenceEngine: KV cache, scratch buffers, PRNG"));
                        ak.push(String::from("// forward_one(): embed → N×(attn + ffn) → logits"));
                        ak.push(String::from("// sample_token(): temperature + top-k sampling"));
                        ak.push(String::from("// compute_loss(): teacher forcing cross-entropy"));
                    }
                    "training" => {
                        ak.push(String::from("// train_step(): stochastic numerical gradients (1% params/step)"));
                        ak.push(String::from("// train_step_random(): evolution strategies"));
                        ak.push(String::from("// Rotates through: embeddings → W_q → W_k → W_o → W_gate → W_output"));
                        ak.push(format!("// Batch mode via MENTOR:BATCH_START/END"));
                    }
                    "agent" => {
                        ak.push(String::from("// classify_input(): route to Generate/Execute/Introspect/Train"));
                        ak.push(String::from("// introspect(): Architecture, WeightStats, Hardware, SourceCode"));
                        ak.push(String::from("// bench_inference(): measure tokens/sec"));
                    }
                    "mentor" => {
                        ak.push(String::from("// Serial mentoring: MENTOR:TEACH/CORRECT/EVAL/GENERATE/..."));
                        ak.push(String::from("// poll_serial(): non-blocking check on COM1"));
                        ak.push(String::from("// Batch training with loss accumulation"));
                        ak.push(String::from("// Save/load: /jarvis/weights.bin via ramfs"));
                    }
                    "tokenizer" => {
                        ak.push(String::from("// Byte-level tokenizer (vocab=256, zero-cost)"));
                        ak.push(String::from("// encode(text) → [BOS, byte0, byte1, ..., EOS]"));
                        ak.push(String::from("// decode(tokens) → String (lossy UTF-8)"));
                        ak.push(String::from("// Special: PAD=0x00, BOS=0x01, EOS=0x02, SEP=0x03"));
                    }
                    _ => {
                        ak.push(format!("// Module '{}' — use 'source model/inference/training/agent/mentor/tokenizer'", apz));
                    }
                }
            }
        }

        IntrospectTarget::Bv => {
            ak.lg(flw(&IntrospectTarget::Agj));
            ak.push(String::new());
            ak.lg(flw(&IntrospectTarget::Ip));
            ak.push(String::new());
            ak.lg(flw(&IntrospectTarget::Bup));
        }
    }

    ak
}






pub fn qox() -> (f32, u64) {
    if !super::uc() { return (0.0, 0); }

    let bet = super::Ci.lock();
    let model = match bet.as_ref() {
        Some(ef) => ef,
        None => return (0.0, 0),
    };

    let aau = b"The quick brown fox jumps over";
    let bvi = 32;

    let ay = crate::time::ave();

    let mut engine = super::inference::InferenceEngine::new();
    engine.config.fwj = 0.5;
    let _ = engine.cks(model, aau, bvi);

    let oz = crate::time::ave().ao(ay).am(1);
    let xjj = (bvi as f32 * 1000.0) / oz as f32;

    (xjj, oz)
}





fn lsa() -> usize {
    model::BG_ * model::E_ * 2  
    + model::AZ_ * (
        model::E_  
        + model::E_ * model::E_ * 4  
        + model::E_  
        + model::E_ * model::Y_ * 3  
    )
    + model::E_  
    + model::E_ * model::BG_  
}


fn dxl(p: &[f32]) -> String {
    if p.is_empty() { return String::from("empty"); }
    let bo = p.len() as f32;
    let llk = p.iter().sum::<f32>() / bo;
    let bfp = p.iter().map(|&b| (b - llk) * (b - llk)).sum::<f32>() / bo;
    let std = if bfp > 0.0 {
        let fs = bfp.bsr();
        let anj = f32::bhb((fs >> 1) + 0x1FBD_1DF5);
        (anj + bfp / anj) * 0.5
    } else {
        0.0
    };
    let v = p.iter().hu().cqs(f32::Att, f32::v);
    let am = p.iter().hu().cqs(f32::IP_, f32::am);
    format!("n={} mean={:.4} std={:.4} min={:.3} max={:.3}", p.len(), llk, std, v, am)
}
