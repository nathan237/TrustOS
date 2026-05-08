//! AI Hardware Analyzer — Jarvis reasons about the hardware it's running on
//!
//! This is the "brain → eyes" connection. Given a HardwareProfile, Jarvis:
//!   1. Generates a natural-language understanding of the platform
//!   2. Identifies bottlenecks and optimization opportunities
//!   3. Builds an execution plan (which resources to use for what)
//!   4. Continuously monitors and adapts its behavior
//!
//! The AI doesn't just read hardware — it UNDERSTANDS it, builds a mental
//! model, and uses that model to make better decisions.

use alloc::string::String;
use alloc::format;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};

use super::probe::HardwareProfile;

// ═══════════════════════════════════════════════════════════════════════════════
// Hardware Understanding — AI-generated insights
// ═══════════════════════════════════════════════════════════════════════════════

/// A single insight that Jarvis has about the hardware
#[derive(Clone)]
// Public structure — visible outside this module.
pub struct HardwareInsight {
    pub category: InsightCategory,
    pub severity: InsightSeverity,
    pub title: String,
    pub detail: String,
    pub action: String,
}

// #[derive] — auto-generates trait implementations at compile time.
#[derive(Clone, Copy, PartialEq)]
// Enumeration — a type that can be one of several variants.
pub enum InsightCategory {
    Performance,
    Bottleneck,
    Security,
    Opportunity,
    Anomaly,
    Resource,
}

// #[derive] — auto-generates trait implementations at compile time.
#[derive(Clone, Copy, PartialEq, Ord, PartialOrd, Eq)]
// Enumeration — a type that can be one of several variants.
pub enum InsightSeverity {
    Info,
    Advisory,
    Important,
    Critical,
}

// Implementation block — defines methods for the type above.
impl InsightCategory {
        // Public function — callable from other modules.
pub fn as_str(&self) -> &'static str {
                // Pattern matching — Rust's exhaustive branching construct.
match self {
            InsightCategory::Performance => "PERF",
            InsightCategory::Bottleneck => "BOTTLENECK",
            InsightCategory::Security => "SECURITY",
            InsightCategory::Opportunity => "OPPORTUNITY",
            InsightCategory::Anomaly => "ANOMALY",
            InsightCategory::Resource => "RESOURCE",
        }
    }

        // Public function — callable from other modules.
pub fn color(&self) -> &'static str {
                // Pattern matching — Rust's exhaustive branching construct.
match self {
            InsightCategory::Performance => "\x01G",
            InsightCategory::Bottleneck => "\x01R",
            InsightCategory::Security => "\x01M",
            InsightCategory::Opportunity => "\x01C",
            InsightCategory::Anomaly => "\x01Y",
            InsightCategory::Resource => "\x01W",
        }
    }
}

// Implementation block — defines methods for the type above.
impl InsightSeverity {
        // Public function — callable from other modules.
pub fn as_str(&self) -> &'static str {
                // Pattern matching — Rust's exhaustive branching construct.
match self {
            InsightSeverity::Info => "INFO",
            InsightSeverity::Advisory => "NOTE",
            InsightSeverity::Important => "WARN",
            InsightSeverity::Critical => "CRIT",
        }
    }
}

/// Execution plan — how Jarvis will use the hardware
#[derive(Clone)]
// Public structure — visible outside this module.
pub struct ExecutionPlan {
    /// Which SIMD tier to use for inference
    pub simd_tier: SimdTier,
    /// Whether to offload to GPU
    pub use_gpu: bool,
    /// Optimal batch size for training
    pub optimal_batch_size: usize,
    /// Optimal tile size for matrix ops (cache-aware)
    pub tile_size: usize,
    /// Number of worker threads for parallel tasks
    pub worker_threads: u32,
    /// Maximum tokens per generation (memory-limited)
    pub max_gen_tokens: usize,
    /// Learning rate adjustment factor based on compute
    pub lr_factor: f32,
    /// Whether to enable background learning
    pub background_learning: bool,
    /// Whether to enable hardware monitoring
    pub hw_monitoring: bool,
    /// Strategy description
    pub strategy: String,
}

// #[derive] — auto-generates trait implementations at compile time.
#[derive(Clone, Copy, PartialEq)]
// Enumeration — a type that can be one of several variants.
pub enum SimdTier {
    Scalar,
    Sse2,
    Avx2,
    Avx512,
    Neon,   // AArch64
    GpuGemm,
}

// Implementation block — defines methods for the type above.
impl SimdTier {
        // Public function — callable from other modules.
pub fn as_str(&self) -> &'static str {
                // Pattern matching — Rust's exhaustive branching construct.
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

/// Running performance monitor state
static INFERENCE_COUNT: AtomicU64 = AtomicU64::new(0);
// Atomic variable — provides lock-free thread-safe access.
static TOTAL_INFERENCE_US: AtomicU64 = AtomicU64::new(0);
// Atomic variable — provides lock-free thread-safe access.
static ANOMALY_COUNT: AtomicU64 = AtomicU64::new(0);

// ═══════════════════════════════════════════════════════════════════════════════
// Analysis Engine — Generate insights from hardware profile
// ═══════════════════════════════════════════════════════════════════════════════

/// Analyze the hardware and generate AI insights
pub fn analyze_hardware(profile: &HardwareProfile) -> Vec<HardwareInsight> {
    let mut insights = Vec::new();

    analyze_compute(&mut insights, profile);
    analyze_memory(&mut insights, profile);
    analyze_storage(&mut insights, profile);
    analyze_network(&mut insights, profile);
    analyze_security(&mut insights, profile);
    analyze_architecture(&mut insights, profile);

    // Sort by severity (critical first)
    insights.sort_by(|a, b| b.severity.cmp(&a.severity));

    insights
}

fn analyze_compute(insights: &mut Vec<HardwareInsight>, profile: &HardwareProfile) {
    // SIMD capabilities
    if profile.has_avx512 {
        insights.push(HardwareInsight {
            category: InsightCategory::Performance,
            severity: InsightSeverity::Info,
            title: String::from("AVX-512 detected"),
            detail: String::from("CPU supports 512-bit SIMD — 8x FP32 lanes"),
            action: String::from("Enable AVX-512 matmul kernels for 8x neural throughput"),
        });
    } else if profile.has_avx2 {
        insights.push(HardwareInsight {
            category: InsightCategory::Opportunity,
            severity: InsightSeverity::Advisory,
            title: String::from("AVX2 available, no AVX-512"),
            detail: String::from("256-bit SIMD available — 4x FP32 lanes"),
            action: String::from("Use AVX2 GEMM with 8×4 tile blocking for inference"),
        });
    } else if !profile.has_sse2 {
        insights.push(HardwareInsight {
            category: InsightCategory::Bottleneck,
            severity: InsightSeverity::Important,
            title: String::from("No SIMD detected"),
            detail: String::from("Scalar-only compute — inference will be slow"),
            action: String::from("Use smallest model variant, reduce batch size"),
        });
    }

    // Core count
    if profile.cpu_cores >= 8 {
        insights.push(HardwareInsight {
            category: InsightCategory::Opportunity,
            severity: InsightSeverity::Info,
            title: format!("{}-core CPU detected", profile.cpu_cores),
            detail: String::from("Multi-core parallelism available for background tasks"),
            action: String::from("Enable parallel training + inference on separate cores"),
        });
    } else if profile.cpu_cores == 1 {
        insights.push(HardwareInsight {
            category: InsightCategory::Bottleneck,
            severity: InsightSeverity::Advisory,
            title: String::from("Single-core system"),
            detail: String::from("No parallelism — all tasks sequential"),
            action: String::from("Disable background learning, prioritize inference"),
        });
    }

    // GPU
    if profile.has_gpu {
        insights.push(HardwareInsight {
            category: InsightCategory::Performance,
            severity: InsightSeverity::Important,
            title: format!("GPU available: {}", profile.gpu_name),
            detail: format!("{} CUs, {} MB VRAM", profile.gpu_compute_units, profile.gpu_vram_mb),
            action: String::from("Offload matrix operations to GPU GEMM dispatch"),
        });
    }
}

fn analyze_memory(insights: &mut Vec<HardwareInsight>, profile: &HardwareProfile) {
    let ram_mb = profile.total_ram_bytes / (1024 * 1024);

    if ram_mb < 256 {
        insights.push(HardwareInsight {
            category: InsightCategory::Bottleneck,
            severity: InsightSeverity::Critical,
            title: format!("Very low RAM: {} MB", ram_mb),
            detail: String::from("Model + optimizer may not fit in memory"),
            action: String::from("Use INT8 quantization, disable Adam (use SGD), reduce context"),
        });
    } else if ram_mb < 1024 {
        insights.push(HardwareInsight {
            category: InsightCategory::Bottleneck,
            severity: InsightSeverity::Advisory,
            title: format!("Limited RAM: {} MB", ram_mb),
            detail: String::from("Model fits but optimizer state is tight"),
            action: String::from("Limit gradient accumulation buffer, reduce max_seq"),
        });
    } else {
        insights.push(HardwareInsight {
            category: InsightCategory::Resource,
            severity: InsightSeverity::Info,
            title: format!("Adequate RAM: {} MB", ram_mb),
            detail: String::from("Full model + Adam optimizer + large context fits"),
            action: String::from("Enable full training pipeline with gradient accumulation"),
        });
    }

    // Heap pressure
    let heap_pct = if profile.heap_size_bytes > 0 {
        profile.heap_used_bytes as f32 / profile.heap_size_bytes as f32
    } else { 0.0 };

    if heap_pct > 0.85 {
        insights.push(HardwareInsight {
            category: InsightCategory::Bottleneck,
            severity: InsightSeverity::Critical,
            title: format!("Heap pressure: {:.0}% used", heap_pct * 100.0),
            detail: format!("{} KB free of {} KB", profile.heap_free_bytes / 1024, profile.heap_size_bytes / 1024),
            action: String::from("Free cached data, reduce model context window"),
        });
    }
}

fn analyze_storage(insights: &mut Vec<HardwareInsight>, profile: &HardwareProfile) {
    if profile.storage_devices.is_empty() {
        insights.push(HardwareInsight {
            category: InsightCategory::Anomaly,
            severity: InsightSeverity::Advisory,
            title: String::from("No storage detected"),
            detail: String::from("Cannot persist weights or data — RAM-only mode"),
            action: String::from("Disable weight persistence, use RAM-only training"),
        });
    } else {
        let has_nvme = profile.storage_devices.iter().any(|s| s.kind == super::probe::StorageKind::Nvme);
        if has_nvme {
            insights.push(HardwareInsight {
                category: InsightCategory::Performance,
                severity: InsightSeverity::Info,
                title: String::from("NVMe storage available"),
                detail: String::from("Fast I/O for weight checkpoints and data loading"),
                action: String::from("Enable aggressive checkpointing (every 100 steps)"),
            });
        }
    }
}

fn analyze_network(insights: &mut Vec<HardwareInsight>, profile: &HardwareProfile) {
    if profile.has_network && profile.link_up {
        insights.push(HardwareInsight {
            category: InsightCategory::Opportunity,
            severity: InsightSeverity::Info,
            title: String::from("Network active"),
            detail: String::from("Could receive training data or firmware updates"),
            action: String::from("Enable network mentoring protocol alongside serial"),
        });
    } else if !profile.has_network {
        insights.push(HardwareInsight {
            category: InsightCategory::Resource,
            severity: InsightSeverity::Info,
            title: String::from("Isolated system (no network)"),
            detail: String::from("Air-gapped — good for security analysis"),
            action: String::from("Use serial mentoring only for external communication"),
        });
    }
}

fn analyze_security(insights: &mut Vec<HardwareInsight>, profile: &HardwareProfile) {
    if profile.has_aesni {
        insights.push(HardwareInsight {
            category: InsightCategory::Security,
            severity: InsightSeverity::Info,
            title: String::from("Hardware AES available"),
            detail: String::from("AES-NI for fast encryption of sensitive data"),
            action: String::from("Use HW AES for weight encryption at rest"),
        });
    }

    if !profile.has_rdrand {
        insights.push(HardwareInsight {
            category: InsightCategory::Security,
            severity: InsightSeverity::Advisory,
            title: String::from("No hardware RNG"),
            detail: String::from("RDRAND not available — using software PRNG"),
            action: String::from("Seed PRNG from TSC entropy, use longer initialization"),
        });
    }

    // Privilege level check
    insights.push(HardwareInsight {
        category: InsightCategory::Security,
        severity: InsightSeverity::Info,
        title: format!("Running at {} on {}", profile.privilege_level, profile.arch),
        detail: String::from("Full hardware access — no OS filtering"),
        action: String::from("Direct register access enabled for all probing"),
    });
}

fn analyze_architecture(insights: &mut Vec<HardwareInsight>, profile: &HardwareProfile) {
        // Pattern matching — Rust's exhaustive branching construct.
match profile.arch {
        "x86_64" => {
            insights.push(HardwareInsight {
                category: InsightCategory::Resource,
                severity: InsightSeverity::Info,
                title: String::from("x86_64 platform"),
                detail: String::from("Full CPUID, PCI, ACPI detection available"),
                action: String::from("Use native SSE2/AVX paths for max performance"),
            });
        }
        "aarch64" => {
            insights.push(HardwareInsight {
                category: InsightCategory::Opportunity,
                severity: InsightSeverity::Advisory,
                title: String::from("AArch64 platform"),
                detail: String::from("NEON SIMD + TrustZone probing available"),
                action: String::from("Scan TrustZone boundaries, use NEON for inference"),
            });
        }
        "riscv64" => {
            insights.push(HardwareInsight {
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

// ═══════════════════════════════════════════════════════════════════════════════
// Execution Plan Generator — Tell Jarvis how to use this hardware
// ═══════════════════════════════════════════════════════════════════════════════

/// Generate an optimal execution plan based on hardware analysis
pub fn generate_plan(profile: &HardwareProfile) -> ExecutionPlan {
    // SIMD tier selection
    let simd_tier = if profile.has_gpu {
        SimdTier::GpuGemm
    } else if profile.has_avx512 {
        SimdTier::Avx512
    } else if profile.has_avx2 {
        SimdTier::Avx2
    } else if profile.has_sse2 {
        SimdTier::Sse2
    } else if profile.arch == "aarch64" {
        SimdTier::Neon
    } else {
        SimdTier::Scalar
    };

    // Optimal tile size based on estimated L2 cache
    let tile_size = // Pattern matching — Rust's exhaustive branching construct.
match simd_tier {
        SimdTier::Avx512 => 32,
        SimdTier::Avx2 => 16,
        SimdTier::Sse2 => 8,
        SimdTier::GpuGemm => 64,
        _ => 4,
    };

    // Batch size based on available memory
    let ram_mb = profile.total_ram_bytes / (1024 * 1024);
    let optimal_batch = if ram_mb >= 4096 { 16 }
        else if ram_mb >= 2048 { 8 }
        else if ram_mb >= 1024 { 4 }
        else if ram_mb >= 512 { 2 }
        else { 1 };

    // Max generation tokens based on memory
    let maximum_generator = if ram_mb >= 2048 { 256 }
        else if ram_mb >= 512 { 128 }
        else { 64 };

    // Worker threads
    let workers = if profile.cpu_cores > 2 {
        profile.cpu_cores - 1 // Leave 1 core for main thread
    } else {
        1
    };

    // LR factor: faster hardware can use larger LR (more iterations/sec)
    let lr_factor = if profile.compute_score > 0.7 { 1.5 }
        else if profile.compute_score > 0.4 { 1.0 }
        else { 0.5 };

    // Background learning only if we have spare compute
    let bg_learning = profile.cpu_cores >= 4 && ram_mb >= 1024;

    let strategy = format!(
        "{}→{}tile, {}batch, {}workers, lr×{:.1}, gen≤{}tok, bg={}",
        simd_tier.as_str(), tile_size, optimal_batch, workers,
        lr_factor, maximum_generator, bg_learning
    );

    ExecutionPlan {
        simd_tier,
        use_gpu: profile.has_gpu,
        optimal_batch_size: optimal_batch,
        tile_size,
        worker_threads: workers,
        max_gen_tokens: maximum_generator,
        lr_factor,
        background_learning: bg_learning,
        hw_monitoring: true,
        strategy,
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Performance Monitoring — Live adaptation
// ═══════════════════════════════════════════════════════════════════════════════

/// Record an inference timing for adaptation
pub fn record_inference(elapsed_us: u64) {
    INFERENCE_COUNT.fetch_add(1, Ordering::Relaxed);
    TOTAL_INFERENCE_US.fetch_add(elapsed_us, Ordering::Relaxed);

    // Detect anomalies (sudden 3x slowdown)
    let count = INFERENCE_COUNT.load(Ordering::Relaxed);
    if count > 10 {
        let average = TOTAL_INFERENCE_US.load(Ordering::Relaxed) / count;
        if elapsed_us > average * 3 {
            ANOMALY_COUNT.fetch_add(1, Ordering::Relaxed);
            crate::serial_println!("[JARVIS-HW] Anomaly: inference took {}µs (avg={}µs)",
                elapsed_us, average);
        }
    }
}

/// Get average inference time in microseconds
pub fn average_inference_us() -> u64 {
    let count = INFERENCE_COUNT.load(Ordering::Relaxed);
    if count == 0 { return 0; }
    TOTAL_INFERENCE_US.load(Ordering::Relaxed) / count
}

/// Get anomaly count
pub fn anomaly_count() -> u64 {
    ANOMALY_COUNT.load(Ordering::Relaxed)
}

// ═══════════════════════════════════════════════════════════════════════════════
// Display
// ═══════════════════════════════════════════════════════════════════════════════

/// Format insights for terminal display
pub fn format_insights(insights: &[HardwareInsight]) -> String {
    let mut s = String::new();
    s.push_str("\x01C═══ JARVIS Hardware Analysis ═══\x01W\n\n");

    for insight in insights {
        s.push_str(&format!("{}{} [{}]\x01W {}\n",
            insight.category.color(),
            insight.category.as_str(),
            insight.severity.as_str(),
            insight.title));
        s.push_str(&format!("  {}\n", insight.detail));
        s.push_str(&format!("  \x01G→ {}\x01W\n\n", insight.action));
    }

    // Performance monitor
    let count = INFERENCE_COUNT.load(Ordering::Relaxed);
    if count > 0 {
        s.push_str(&format!("\x01Y═══ Live Monitor ═══\x01W\n"));
        s.push_str(&format!("  Inferences: {}\n", count));
        s.push_str(&format!("  Avg latency: {} µs\n", average_inference_us()));
        s.push_str(&format!("  Anomalies: {}\n", anomaly_count()));
    }

    s
}

/// Format execution plan for terminal display
pub fn format_plan(plan: &ExecutionPlan) -> String {
    let mut s = String::new();
    s.push_str("\x01C═══ JARVIS Execution Plan ═══\x01W\n\n");
    s.push_str(&format!("  SIMD:         {}\n", plan.simd_tier.as_str()));
    s.push_str(&format!("  GPU:          {}\n", if plan.use_gpu { "ENABLED" } else { "disabled" }));
    s.push_str(&format!("  Batch size:   {}\n", plan.optimal_batch_size));
    s.push_str(&format!("  Tile size:    {}×{}\n", plan.tile_size, plan.tile_size));
    s.push_str(&format!("  Workers:      {}\n", plan.worker_threads));
    s.push_str(&format!("  Max gen:      {} tokens\n", plan.max_gen_tokens));
    s.push_str(&format!("  LR factor:    ×{:.1}\n", plan.lr_factor));
    s.push_str(&format!("  Background:   {}\n", if plan.background_learning { "ON" } else { "OFF" }));
    s.push_str(&format!("  Monitoring:   {}\n", if plan.hw_monitoring { "ON" } else { "OFF" }));
    s.push_str(&format!("\n  \x01CStrategy:\x01W {}\n", plan.strategy));
    s
}
