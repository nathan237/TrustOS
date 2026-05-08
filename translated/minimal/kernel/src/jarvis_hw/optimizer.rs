














use alloc::string::String;
use alloc::format;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU32, AtomicU64, AtomicBool, Ordering};
use spin::Mutex;

use super::probe::N;
use super::analyzer::{Ht, Av, InsightCategory};






#[derive(Clone)]
pub struct Fw {
    pub cycle: u32,
    pub category: OptCategory,
    pub description: String,
    pub metric_before: f32,
    pub metric_after: f32,
    pub improvement_pct: f32,
    pub reverted: bool,
}

#[derive(Clone, Copy, PartialEq)]
pub enum OptCategory {
    AiInference,
    AiTraining,
    MemoryMgmt,
    IoPattern,
    Scheduling,
    Security,
}

impl OptCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            OptCategory::AiInference => "AI/inference",
            OptCategory::AiTraining => "AI/training",
            OptCategory::MemoryMgmt => "memory",
            OptCategory::IoPattern => "I/O",
            OptCategory::Scheduling => "scheduling",
            OptCategory::Security => "security",
        }
    }
}


pub struct Aca {
    pub cycle_count: u32,
    pub records: Vec<Fw>,
    pub current_plan: Option<Ht>,
    pub baseline_inference_us: u64,
    pub baseline_train_loss: f32,
    pub best_inference_us: u64,
    pub best_train_loss: f32,
    pub total_improvement_pct: f32,
}

static XG_: AtomicBool = AtomicBool::new(false);
static BDZ_: AtomicU32 = AtomicU32::new(0);
static AKV_: AtomicU32 = AtomicU32::new(0);
static EMB_: AtomicU64 = AtomicU64::new(0); 

static Dz: Mutex<Option<Aca>> = Mutex::new(None);












pub fn kdk() -> String {
    let mut output = String::new();

    output.push_str("\x01C╔══════════════════════════════════════════════════════════╗\n");
    output.push_str("║      J.A.R.V.I.S. — Autonomous Hardware Intelligence     ║\n");
    output.push_str("║           Boot Scan & Self-Optimization v1.0              ║\n");
    output.push_str("╚══════════════════════════════════════════════════════════╝\x01W\n\n");

    
    output.push_str("\x01Y[Phase 1/5] Hardware Discovery...\x01W\n");
    let ai = super::probe::olc();
    output.push_str(&format!("  {} — {} cores, {} MB RAM, {} PCI devices\n",
        ai.cpu_brand,
        ai.cpu_cores,
        ai.total_ram_bytes / (1024 * 1024),
        ai.pci_device_count));
    output.push_str(&format!("  Storage: {} dev(s), {} GB | GPU: {} | Net: {}\n\n",
        ai.storage_devices.len(),
        ai.total_storage_bytes / (1024 * 1024 * 1024),
        if ai.has_gpu { &ai.gpu_name } else { "none" },
        ai.has_network));

    
    output.push_str("\x01Y[Phase 2/5] AI Hardware Analysis...\x01W\n");
    let nu = super::analyzer::hfd(&ai);
    let aqb = nu.iter().filter(|i| i.severity == super::analyzer::InsightSeverity::Critical).count();
    let ckz = nu.iter().filter(|i| i.severity == super::analyzer::InsightSeverity::Important).count();
    output.push_str(&format!("  {} insights generated ({} critical, {} important)\n",
        nu.len(), aqb, ckz));

    for insight in nu.iter().take(5) {
        output.push_str(&format!("  {} [{}] {}\n",
            insight.category.color(), insight.category.as_str(), insight.title));
    }
    output.push('\n');

    
    output.push_str("\x01Y[Phase 3/5] Execution Plan Generation...\x01W\n");
    let vr = super::analyzer::ibc(&ai);
    output.push_str(&format!("  Strategy: {}\n\n", vr.strategy));

    
    output.push_str("\x01Y[Phase 4/5] Applying Optimizations...\x01W\n");
    let isk = jxc(&ai, &vr, &nu);
    for result in &isk {
        output.push_str(&format!("  \x01G✓\x01W [{}] {}\n", result.category.as_str(), result.description));
    }
    output.push('\n');

    
    output.push_str("\x01Y[Phase 5/5] Starting Adaptive Monitor...\x01W\n");

    let state = Aca {
        cycle_count: 0,
        records: isk,
        current_plan: Some(vr),
        baseline_inference_us: 0,
        baseline_train_loss: f32::MAX,
        best_inference_us: u64::MAX,
        best_train_loss: f32::MAX,
        total_improvement_pct: 0.0,
    };
    *Dz.lock() = Some(state);
    XG_.store(true, Ordering::Release);

    output.push_str("  Monitor active — Jarvis will continuously adapt\n\n");

    
    output.push_str("\x01C═══ Boot Scan Complete ═══\x01W\n");
    output.push_str(&format!("  Hardware score: \x01C{:.0}%\x01W\n", ai.overall_score * 100.0));
    output.push_str(&format!("  Optimizations applied: {}\n", AKV_.load(Ordering::Relaxed)));
    output.push_str("  Jarvis is now aware of its environment.\n");

    output
}





fn jxc(
    ai: &N,
    vr: &Ht,
    nu: &[Av],
) -> Vec<Fw> {
    let mut records = Vec::new();
    let cycle = 0u32;

    
    records.push(Fw {
        cycle,
        category: OptCategory::AiInference,
        description: format!("Set SIMD tier to {} for neural inference", vr.simd_tier.as_str()),
        metric_before: 0.0,
        metric_after: 0.0,
        improvement_pct: 0.0,
        reverted: false,
    });

    
    records.push(Fw {
        cycle,
        category: OptCategory::AiTraining,
        description: format!("Batch size → {} (based on {} MB RAM)", vr.optimal_batch_size,
            ai.total_ram_bytes / (1024 * 1024)),
        metric_before: 0.0,
        metric_after: 0.0,
        improvement_pct: 0.0,
        reverted: false,
    });

    
    let heap_pct = if ai.heap_size_bytes > 0 {
        ai.heap_used_bytes as f32 / ai.heap_size_bytes as f32
    } else { 0.0 };

    if heap_pct > 0.7 {
        records.push(Fw {
            cycle,
            category: OptCategory::MemoryMgmt,
            description: format!("Heap pressure {:.0}% — enable aggressive cache eviction", heap_pct * 100.0),
            metric_before: heap_pct,
            metric_after: 0.0,
            improvement_pct: 0.0,
            reverted: false,
        });
    }

    
    if ai.has_gpu {
        records.push(Fw {
            cycle,
            category: OptCategory::AiInference,
            description: format!("Enable GPU offload: {} ({} CUs)", ai.gpu_name, ai.gpu_compute_units),
            metric_before: 0.0,
            metric_after: 0.0,
            improvement_pct: 0.0,
            reverted: false,
        });
    }

    
    if ai.has_aesni {
        records.push(Fw {
            cycle,
            category: OptCategory::Security,
            description: String::from("Enable AES-NI for weight encryption at rest"),
            metric_before: 0.0,
            metric_after: 0.0,
            improvement_pct: 0.0,
            reverted: false,
        });
    }

    
    if vr.background_learning {
        records.push(Fw {
            cycle,
            category: OptCategory::AiTraining,
            description: format!("Enable background learning ({} spare cores)", ai.cpu_cores - 1),
            metric_before: 0.0,
            metric_after: 0.0,
            improvement_pct: 0.0,
            reverted: false,
        });
    }

    
    for insight in nu {
        if insight.severity == super::analyzer::InsightSeverity::Critical {
            records.push(Fw {
                cycle,
                category: match insight.category {
                    InsightCategory::Performance => OptCategory::AiInference,
                    InsightCategory::Bottleneck => OptCategory::MemoryMgmt,
                    InsightCategory::Security => OptCategory::Security,
                    _ => OptCategory::AiInference,
                },
                description: format!("Critical: {}", insight.action),
                metric_before: 0.0,
                metric_after: 0.0,
                improvement_pct: 0.0,
                reverted: false,
            });
        }
    }

    AKV_.store(records.len() as u32, Ordering::Relaxed);
    records
}






pub fn oje() -> Option<String> {
    if !XG_.load(Ordering::Acquire) { return None; }

    let mut jg = Dz.lock();
    let state = jg.as_mut()?;
    state.cycle_count += 1;
    let cycle = state.cycle_count;
    BDZ_.store(cycle, Ordering::Relaxed);

    let mut output = String::new();
    output.push_str(&format!("\x01C[Optimization Cycle #{}]\x01W\n", cycle));

    
    let bri = super::analyzer::hgg();
    let bqg = super::analyzer::hfg();

    
    if state.baseline_inference_us == 0 && bri > 0 {
        state.baseline_inference_us = bri;
        state.best_inference_us = bri;
        output.push_str(&format!("  Baseline established: {} µs/inference\n", bri));
    } else if bri > 0 && bri < state.best_inference_us {
        let moj = (state.best_inference_us as f32 - bri as f32)
            / state.best_inference_us as f32 * 100.0;
        state.best_inference_us = bri;
        output.push_str(&format!("  \x01GNew best:\x01W {} µs ({:.1}% faster)\n", bri, moj));
    }

    
    if bqg > 0 {
        output.push_str(&format!("  \x01RWARN:\x01W {} anomalies detected — investigating\n", bqg));
        
        let heap_used = crate::memory::heap::used();
        let heap_total = crate::memory::atz();
        let pressure = heap_used as f32 / heap_total as f32;
        if pressure > 0.9 {
            output.push_str("  → Heap critical: recommending emergency cache flush\n");
        }
    }

    
    if let Some(vr) = &state.current_plan {
        if cycle % 10 == 0 {
            output.push_str(&format!("  Current plan: {}\n", vr.strategy));
        }
    }

    Some(output)
}






pub fn status() -> String {
    let mut j = String::new();

    j.push_str("\x01C═══ Adaptive Optimizer Status ═══\x01W\n\n");
    j.push_str(&format!("  Active: {}\n", XG_.load(Ordering::Relaxed)));
    j.push_str(&format!("  Cycles: {}\n", BDZ_.load(Ordering::Relaxed)));
    j.push_str(&format!("  Optimizations: {}\n", AKV_.load(Ordering::Relaxed)));

    if let Some(state) = Dz.lock().as_ref() {
        if state.baseline_inference_us > 0 {
            j.push_str(&format!("  Baseline: {} µs/inference\n", state.baseline_inference_us));
            j.push_str(&format!("  Best:     {} µs/inference\n", state.best_inference_us));
            if state.baseline_inference_us > state.best_inference_us {
                let moi = (state.baseline_inference_us - state.best_inference_us) as f32
                    / state.baseline_inference_us as f32 * 100.0;
                j.push_str(&format!("  Improvement: \x01G{:.1}%\x01W\n", moi));
            }
        }

        j.push_str(&format!("\n  Recent optimizations:\n"));
        for dxn in state.records.iter().rev().take(10) {
            let status = if dxn.reverted { "\x01R⟲\x01W" } else { "\x01G✓\x01W" };
            j.push_str(&format!("    {} [{}] {}\n", status, dxn.category.as_str(), dxn.description));
        }
    }

    j
}


pub fn is_active() -> bool {
    XG_.load(Ordering::Acquire)
}


pub fn current_plan() -> Option<Ht> {
    Dz.lock().as_ref()?.current_plan.clone()
}
