










use alloc::string::String;
use alloc::format;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};

use super::probe::N;






#[derive(Clone)]
pub struct Av {
    pub category: InsightCategory,
    pub severity: InsightSeverity,
    pub title: String,
    pub detail: String,
    pub action: String,
}

#[derive(Clone, Copy, PartialEq)]
pub enum InsightCategory {
    Performance,
    Bottleneck,
    Security,
    Opportunity,
    Anomaly,
    Fx,
}

#[derive(Clone, Copy, PartialEq, Ord, PartialOrd, Eq)]
pub enum InsightSeverity {
    Info,
    Advisory,
    Important,
    Critical,
}

impl InsightCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            InsightCategory::Performance => "PERF",
            InsightCategory::Bottleneck => "BOTTLENECK",
            InsightCategory::Security => "SECURITY",
            InsightCategory::Opportunity => "OPPORTUNITY",
            InsightCategory::Anomaly => "ANOMALY",
            InsightCategory::Fx => "RESOURCE",
        }
    }

    pub fn color(&self) -> &'static str {
        match self {
            InsightCategory::Performance => "\x01G",
            InsightCategory::Bottleneck => "\x01R",
            InsightCategory::Security => "\x01M",
            InsightCategory::Opportunity => "\x01C",
            InsightCategory::Anomaly => "\x01Y",
            InsightCategory::Fx => "\x01W",
        }
    }
}

impl InsightSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            InsightSeverity::Info => "INFO",
            InsightSeverity::Advisory => "NOTE",
            InsightSeverity::Important => "WARN",
            InsightSeverity::Critical => "CRIT",
        }
    }
}


#[derive(Clone)]
pub struct Ht {
    
    pub simd_tier: SimdTier,
    
    pub use_gpu: bool,
    
    pub optimal_batch_size: usize,
    
    pub tile_size: usize,
    
    pub worker_threads: u32,
    
    pub max_gen_tokens: usize,
    
    pub lr_factor: f32,
    
    pub background_learning: bool,
    
    pub hw_monitoring: bool,
    
    pub strategy: String,
}

#[derive(Clone, Copy, PartialEq)]
pub enum SimdTier {
    Scalar,
    Sse2,
    Avx2,
    Avx512,
    Neon,   
    GpuGemm,
}

impl SimdTier {
    pub fn as_str(&self) -> &'static str {
        match self {
            SimdTier::Scalar => "scalar",
            SimdTier::Sse2 => "SSE2",
            SimdTier::Avx2 => "AVX2",
            SimdTier::Avx512 => "AVX-512",
            SimdTier::Neon => "NEON",
            SimdTier::GpuGemm => "GPU-GEMM",
        }
    }
}


static VK_: AtomicU64 = AtomicU64::new(0);
static AKU_: AtomicU64 = AtomicU64::new(0);
static AMJ_: AtomicU64 = AtomicU64::new(0);






pub fn hfd(ai: &N) -> Vec<Av> {
    let mut nu = Vec::new();

    jvv(&mut nu, ai);
    jvx(&mut nu, ai);
    jwa(&mut nu, ai);
    jvy(&mut nu, ai);
    jvz(&mut nu, ai);
    jvt(&mut nu, ai);

    
    nu.sort_by(|a, b| b.severity.cmp(&a.severity));

    nu
}

fn jvv(nu: &mut Vec<Av>, ai: &N) {
    
    if ai.has_avx512 {
        nu.push(Av {
            category: InsightCategory::Performance,
            severity: InsightSeverity::Info,
            title: String::from("AVX-512 detected"),
            detail: String::from("CPU supports 512-bit SIMD — 8x FP32 lanes"),
            action: String::from("Enable AVX-512 matmul kernels for 8x neural throughput"),
        });
    } else if ai.has_avx2 {
        nu.push(Av {
            category: InsightCategory::Opportunity,
            severity: InsightSeverity::Advisory,
            title: String::from("AVX2 available, no AVX-512"),
            detail: String::from("256-bit SIMD available — 4x FP32 lanes"),
            action: String::from("Use AVX2 GEMM with 8×4 tile blocking for inference"),
        });
    } else if !ai.has_sse2 {
        nu.push(Av {
            category: InsightCategory::Bottleneck,
            severity: InsightSeverity::Important,
            title: String::from("No SIMD detected"),
            detail: String::from("Scalar-only compute — inference will be slow"),
            action: String::from("Use smallest model variant, reduce batch size"),
        });
    }

    
    if ai.cpu_cores >= 8 {
        nu.push(Av {
            category: InsightCategory::Opportunity,
            severity: InsightSeverity::Info,
            title: format!("{}-core CPU detected", ai.cpu_cores),
            detail: String::from("Multi-core parallelism available for background tasks"),
            action: String::from("Enable parallel training + inference on separate cores"),
        });
    } else if ai.cpu_cores == 1 {
        nu.push(Av {
            category: InsightCategory::Bottleneck,
            severity: InsightSeverity::Advisory,
            title: String::from("Single-core system"),
            detail: String::from("No parallelism — all tasks sequential"),
            action: String::from("Disable background learning, prioritize inference"),
        });
    }

    
    if ai.has_gpu {
        nu.push(Av {
            category: InsightCategory::Performance,
            severity: InsightSeverity::Important,
            title: format!("GPU available: {}", ai.gpu_name),
            detail: format!("{} CUs, {} MB VRAM", ai.gpu_compute_units, ai.gpu_vram_mb),
            action: String::from("Offload matrix operations to GPU GEMM dispatch"),
        });
    }
}

fn jvx(nu: &mut Vec<Av>, ai: &N) {
    let ram_mb = ai.total_ram_bytes / (1024 * 1024);

    if ram_mb < 256 {
        nu.push(Av {
            category: InsightCategory::Bottleneck,
            severity: InsightSeverity::Critical,
            title: format!("Very low RAM: {} MB", ram_mb),
            detail: String::from("Model + optimizer may not fit in memory"),
            action: String::from("Use INT8 quantization, disable Adam (use SGD), reduce context"),
        });
    } else if ram_mb < 1024 {
        nu.push(Av {
            category: InsightCategory::Bottleneck,
            severity: InsightSeverity::Advisory,
            title: format!("Limited RAM: {} MB", ram_mb),
            detail: String::from("Model fits but optimizer state is tight"),
            action: String::from("Limit gradient accumulation buffer, reduce max_seq"),
        });
    } else {
        nu.push(Av {
            category: InsightCategory::Fx,
            severity: InsightSeverity::Info,
            title: format!("Adequate RAM: {} MB", ram_mb),
            detail: String::from("Full model + Adam optimizer + large context fits"),
            action: String::from("Enable full training pipeline with gradient accumulation"),
        });
    }

    
    let heap_pct = if ai.heap_size_bytes > 0 {
        ai.heap_used_bytes as f32 / ai.heap_size_bytes as f32
    } else { 0.0 };

    if heap_pct > 0.85 {
        nu.push(Av {
            category: InsightCategory::Bottleneck,
            severity: InsightSeverity::Critical,
            title: format!("Heap pressure: {:.0}% used", heap_pct * 100.0),
            detail: format!("{} KB free of {} KB", ai.heap_free_bytes / 1024, ai.heap_size_bytes / 1024),
            action: String::from("Free cached data, reduce model context window"),
        });
    }
}

fn jwa(nu: &mut Vec<Av>, ai: &N) {
    if ai.storage_devices.is_empty() {
        nu.push(Av {
            category: InsightCategory::Anomaly,
            severity: InsightSeverity::Advisory,
            title: String::from("No storage detected"),
            detail: String::from("Cannot persist weights or data — RAM-only mode"),
            action: String::from("Disable weight persistence, use RAM-only training"),
        });
    } else {
        let fzx = ai.storage_devices.iter().any(|j| j.kind == super::probe::StorageKind::Nvme);
        if fzx {
            nu.push(Av {
                category: InsightCategory::Performance,
                severity: InsightSeverity::Info,
                title: String::from("NVMe storage available"),
                detail: String::from("Fast I/O for weight checkpoints and data loading"),
                action: String::from("Enable aggressive checkpointing (every 100 steps)"),
            });
        }
    }
}

fn jvy(nu: &mut Vec<Av>, ai: &N) {
    if ai.has_network && ai.link_up {
        nu.push(Av {
            category: InsightCategory::Opportunity,
            severity: InsightSeverity::Info,
            title: String::from("Network active"),
            detail: String::from("Could receive training data or firmware updates"),
            action: String::from("Enable network mentoring protocol alongside serial"),
        });
    } else if !ai.has_network {
        nu.push(Av {
            category: InsightCategory::Fx,
            severity: InsightSeverity::Info,
            title: String::from("Isolated system (no network)"),
            detail: String::from("Air-gapped — good for security analysis"),
            action: String::from("Use serial mentoring only for external communication"),
        });
    }
}

fn jvz(nu: &mut Vec<Av>, ai: &N) {
    if ai.has_aesni {
        nu.push(Av {
            category: InsightCategory::Security,
            severity: InsightSeverity::Info,
            title: String::from("Hardware AES available"),
            detail: String::from("AES-NI for fast encryption of sensitive data"),
            action: String::from("Use HW AES for weight encryption at rest"),
        });
    }

    if !ai.has_rdrand {
        nu.push(Av {
            category: InsightCategory::Security,
            severity: InsightSeverity::Advisory,
            title: String::from("No hardware RNG"),
            detail: String::from("RDRAND not available — using software PRNG"),
            action: String::from("Seed PRNG from TSC entropy, use longer initialization"),
        });
    }

    
    nu.push(Av {
        category: InsightCategory::Security,
        severity: InsightSeverity::Info,
        title: format!("Running at {} on {}", ai.privilege_level, ai.arch),
        detail: String::from("Full hardware access — no OS filtering"),
        action: String::from("Direct register access enabled for all probing"),
    });
}

fn jvt(nu: &mut Vec<Av>, ai: &N) {
    match ai.arch {
        "x86_64" => {
            nu.push(Av {
                category: InsightCategory::Fx,
                severity: InsightSeverity::Info,
                title: String::from("x86_64 platform"),
                detail: String::from("Full CPUID, PCI, ACPI detection available"),
                action: String::from("Use native SSE2/AVX paths for max performance"),
            });
        }
        "aarch64" => {
            nu.push(Av {
                category: InsightCategory::Opportunity,
                severity: InsightSeverity::Advisory,
                title: String::from("AArch64 platform"),
                detail: String::from("NEON SIMD + TrustZone probing available"),
                action: String::from("Scan TrustZone boundaries, use NEON for inference"),
            });
        }
        "riscv64" => {
            nu.push(Av {
                category: InsightCategory::Opportunity,
                severity: InsightSeverity::Advisory,
                title: String::from("RISC-V platform"),
                detail: String::from("Native RISC-V — no translation needed"),
                action: String::from("Run binaries natively, use V-extension if available"),
            });
        }
        _ => {}
    }
}






pub fn ibc(ai: &N) -> Ht {
    
    let simd_tier = if ai.has_gpu {
        SimdTier::GpuGemm
    } else if ai.has_avx512 {
        SimdTier::Avx512
    } else if ai.has_avx2 {
        SimdTier::Avx2
    } else if ai.has_sse2 {
        SimdTier::Sse2
    } else if ai.arch == "aarch64" {
        SimdTier::Neon
    } else {
        SimdTier::Scalar
    };

    
    let tile_size = match simd_tier {
        SimdTier::Avx512 => 32,
        SimdTier::Avx2 => 16,
        SimdTier::Sse2 => 8,
        SimdTier::GpuGemm => 64,
        _ => 4,
    };

    
    let ram_mb = ai.total_ram_bytes / (1024 * 1024);
    let ism = if ram_mb >= 4096 { 16 }
        else if ram_mb >= 2048 { 8 }
        else if ram_mb >= 1024 { 4 }
        else if ram_mb >= 512 { 2 }
        else { 1 };

    
    let imk = if ram_mb >= 2048 { 256 }
        else if ram_mb >= 512 { 128 }
        else { 64 };

    
    let jri = if ai.cpu_cores > 2 {
        ai.cpu_cores - 1 
    } else {
        1
    };

    
    let lr_factor = if ai.compute_score > 0.7 { 1.5 }
        else if ai.compute_score > 0.4 { 1.0 }
        else { 0.5 };

    
    let hhq = ai.cpu_cores >= 4 && ram_mb >= 1024;

    let strategy = format!(
        "{}→{}tile, {}batch, {}workers, lr×{:.1}, gen≤{}tok, bg={}",
        simd_tier.as_str(), tile_size, ism, jri,
        lr_factor, imk, hhq
    );

    Ht {
        simd_tier,
        use_gpu: ai.has_gpu,
        optimal_batch_size: ism,
        tile_size,
        worker_threads: jri,
        max_gen_tokens: imk,
        lr_factor,
        background_learning: hhq,
        hw_monitoring: true,
        strategy,
    }
}






pub fn qte(cis: u64) {
    VK_.fetch_add(1, Ordering::Relaxed);
    AKU_.fetch_add(cis, Ordering::Relaxed);

    
    let count = VK_.load(Ordering::Relaxed);
    if count > 10 {
        let ns = AKU_.load(Ordering::Relaxed) / count;
        if cis > ns * 3 {
            AMJ_.fetch_add(1, Ordering::Relaxed);
            crate::serial_println!("[JARVIS-HW] Anomaly: inference took {}µs (avg={}µs)",
                cis, ns);
        }
    }
}


pub fn hgg() -> u64 {
    let count = VK_.load(Ordering::Relaxed);
    if count == 0 { return 0; }
    AKU_.load(Ordering::Relaxed) / count
}


pub fn hfg() -> u64 {
    AMJ_.load(Ordering::Relaxed)
}






pub fn lxm(nu: &[Av]) -> String {
    let mut j = String::new();
    j.push_str("\x01C═══ JARVIS Hardware Analysis ═══\x01W\n\n");

    for insight in nu {
        j.push_str(&format!("{}{} [{}]\x01W {}\n",
            insight.category.color(),
            insight.category.as_str(),
            insight.severity.as_str(),
            insight.title));
        j.push_str(&format!("  {}\n", insight.detail));
        j.push_str(&format!("  \x01G→ {}\x01W\n\n", insight.action));
    }

    
    let count = VK_.load(Ordering::Relaxed);
    if count > 0 {
        j.push_str(&format!("\x01Y═══ Live Monitor ═══\x01W\n"));
        j.push_str(&format!("  Inferences: {}\n", count));
        j.push_str(&format!("  Avg latency: {} µs\n", hgg()));
        j.push_str(&format!("  Anomalies: {}\n", hfg()));
    }

    j
}


pub fn hzp(vr: &Ht) -> String {
    let mut j = String::new();
    j.push_str("\x01C═══ JARVIS Execution Plan ═══\x01W\n\n");
    j.push_str(&format!("  SIMD:         {}\n", vr.simd_tier.as_str()));
    j.push_str(&format!("  GPU:          {}\n", if vr.use_gpu { "ENABLED" } else { "disabled" }));
    j.push_str(&format!("  Batch size:   {}\n", vr.optimal_batch_size));
    j.push_str(&format!("  Tile size:    {}×{}\n", vr.tile_size, vr.tile_size));
    j.push_str(&format!("  Workers:      {}\n", vr.worker_threads));
    j.push_str(&format!("  Max gen:      {} tokens\n", vr.max_gen_tokens));
    j.push_str(&format!("  LR factor:    ×{:.1}\n", vr.lr_factor));
    j.push_str(&format!("  Background:   {}\n", if vr.background_learning { "ON" } else { "OFF" }));
    j.push_str(&format!("  Monitoring:   {}\n", if vr.hw_monitoring { "ON" } else { "OFF" }));
    j.push_str(&format!("\n  \x01CStrategy:\x01W {}\n", vr.strategy));
    j
}
