










use alloc::string::String;
use alloc::format;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};

use super::probe::T;






#[derive(Clone)]
pub struct Cd {
    pub gb: InsightCategory,
    pub qj: InsightSeverity,
    pub dq: String,
    pub eu: String,
    pub hr: String,
}

#[derive(Clone, Copy, PartialEq)]
pub enum InsightCategory {
    Yb,
    Oy,
    De,
    Tv,
    Aoz,
    Nx,
}

#[derive(Clone, Copy, PartialEq, Ord, PartialOrd, Eq)]
pub enum InsightSeverity {
    V,
    Os,
    Ajd,
    Aj,
}

impl InsightCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            InsightCategory::Yb => "PERF",
            InsightCategory::Oy => "BOTTLENECK",
            InsightCategory::De => "SECURITY",
            InsightCategory::Tv => "OPPORTUNITY",
            InsightCategory::Aoz => "ANOMALY",
            InsightCategory::Nx => "RESOURCE",
        }
    }

    pub fn s(&self) -> &'static str {
        match self {
            InsightCategory::Yb => "\x01G",
            InsightCategory::Oy => "\x01R",
            InsightCategory::De => "\x01M",
            InsightCategory::Tv => "\x01C",
            InsightCategory::Aoz => "\x01Y",
            InsightCategory::Nx => "\x01W",
        }
    }
}

impl InsightSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            InsightSeverity::V => "INFO",
            InsightSeverity::Os => "NOTE",
            InsightSeverity::Ajd => "WARN",
            InsightSeverity::Aj => "CRIT",
        }
    }
}


#[derive(Clone)]
pub struct Si {
    
    pub fut: SimdTier,
    
    pub mok: bool,
    
    pub jhx: usize,
    
    pub bll: usize,
    
    pub mqw: u32,
    
    pub olu: usize,
    
    pub jea: f32,
    
    pub kbu: bool,
    
    pub ocp: bool,
    
    pub ibx: String,
}

#[derive(Clone, Copy, PartialEq)]
pub enum SimdTier {
    Bsf,
    Qu,
    Ow,
    Apj,
    Tp,   
    Ate,
}

impl SimdTier {
    pub fn as_str(&self) -> &'static str {
        match self {
            SimdTier::Bsf => "scalar",
            SimdTier::Qu => "SSE2",
            SimdTier::Ow => "AVX2",
            SimdTier::Apj => "AVX-512",
            SimdTier::Tp => "NEON",
            SimdTier::Ate => "GPU-GEMM",
        }
    }
}


static UC_: AtomicU64 = AtomicU64::new(0);
static AIY_: AtomicU64 = AtomicU64::new(0);
static AKP_: AtomicU64 = AtomicU64::new(0);






pub fn mvq(cc: &T) -> Vec<Cd> {
    let mut abp = Vec::new();

    qhx(&mut abp, cc);
    qia(&mut abp, cc);
    qid(&mut abp, cc);
    qib(&mut abp, cc);
    qic(&mut abp, cc);
    qhv(&mut abp, cc);

    
    abp.bxe(|q, o| o.qj.cmp(&q.qj));

    abp
}

fn qhx(abp: &mut Vec<Cd>, cc: &T) {
    
    if cc.drm {
        abp.push(Cd {
            gb: InsightCategory::Yb,
            qj: InsightSeverity::V,
            dq: String::from("AVX-512 detected"),
            eu: String::from("CPU supports 512-bit SIMD — 8x FP32 lanes"),
            hr: String::from("Enable AVX-512 matmul kernels for 8x neural throughput"),
        });
    } else if cc.bzx {
        abp.push(Cd {
            gb: InsightCategory::Tv,
            qj: InsightSeverity::Os,
            dq: String::from("AVX2 available, no AVX-512"),
            eu: String::from("256-bit SIMD available — 4x FP32 lanes"),
            hr: String::from("Use AVX2 GEMM with 8×4 tile blocking for inference"),
        });
    } else if !cc.dro {
        abp.push(Cd {
            gb: InsightCategory::Oy,
            qj: InsightSeverity::Ajd,
            dq: String::from("No SIMD detected"),
            eu: String::from("Scalar-only compute — inference will be slow"),
            hr: String::from("Use smallest model variant, reduce batch size"),
        });
    }

    
    if cc.azj >= 8 {
        abp.push(Cd {
            gb: InsightCategory::Tv,
            qj: InsightSeverity::V,
            dq: format!("{}-core CPU detected", cc.azj),
            eu: String::from("Multi-core parallelism available for background tasks"),
            hr: String::from("Enable parallel training + inference on separate cores"),
        });
    } else if cc.azj == 1 {
        abp.push(Cd {
            gb: InsightCategory::Oy,
            qj: InsightSeverity::Os,
            dq: String::from("Single-core system"),
            eu: String::from("No parallelism — all tasks sequential"),
            hr: String::from("Disable background learning, prioritize inference"),
        });
    }

    
    if cc.bqz {
        abp.push(Cd {
            gb: InsightCategory::Yb,
            qj: InsightSeverity::Ajd,
            dq: format!("GPU available: {}", cc.beh),
            eu: format!("{} CUs, {} MB VRAM", cc.erk, cc.dhr),
            hr: String::from("Offload matrix operations to GPU GEMM dispatch"),
        });
    }
}

fn qia(abp: &mut Vec<Cd>, cc: &T) {
    let amo = cc.ccf / (1024 * 1024);

    if amo < 256 {
        abp.push(Cd {
            gb: InsightCategory::Oy,
            qj: InsightSeverity::Aj,
            dq: format!("Very low RAM: {} MB", amo),
            eu: String::from("Model + optimizer may not fit in memory"),
            hr: String::from("Use INT8 quantization, disable Adam (use SGD), reduce context"),
        });
    } else if amo < 1024 {
        abp.push(Cd {
            gb: InsightCategory::Oy,
            qj: InsightSeverity::Os,
            dq: format!("Limited RAM: {} MB", amo),
            eu: String::from("Model fits but optimizer state is tight"),
            hr: String::from("Limit gradient accumulation buffer, reduce max_seq"),
        });
    } else {
        abp.push(Cd {
            gb: InsightCategory::Nx,
            qj: InsightSeverity::V,
            dq: format!("Adequate RAM: {} MB", amo),
            eu: String::from("Full model + Adam optimizer + large context fits"),
            hr: String::from("Enable full training pipeline with gradient accumulation"),
        });
    }

    
    let bne = if cc.drr > 0 {
        cc.ecw as f32 / cc.drr as f32
    } else { 0.0 };

    if bne > 0.85 {
        abp.push(Cd {
            gb: InsightCategory::Oy,
            qj: InsightSeverity::Aj,
            dq: format!("Heap pressure: {:.0}% used", bne * 100.0),
            eu: format!("{} KB free of {} KB", cc.erx / 1024, cc.drr / 1024),
            hr: String::from("Free cached data, reduce model context window"),
        });
    }
}

fn qid(abp: &mut Vec<Cd>, cc: &T) {
    if cc.aqm.is_empty() {
        abp.push(Cd {
            gb: InsightCategory::Aoz,
            qj: InsightSeverity::Os,
            dq: String::from("No storage detected"),
            eu: String::from("Cannot persist weights or data — RAM-only mode"),
            hr: String::from("Disable weight persistence, use RAM-only training"),
        });
    } else {
        let lbe = cc.aqm.iter().any(|e| e.kk == super::probe::StorageKind::Xv);
        if lbe {
            abp.push(Cd {
                gb: InsightCategory::Yb,
                qj: InsightSeverity::V,
                dq: String::from("NVMe storage available"),
                eu: String::from("Fast I/O for weight checkpoints and data loading"),
                hr: String::from("Enable aggressive checkpointing (every 100 steps)"),
            });
        }
    }
}

fn qib(abp: &mut Vec<Cd>, cc: &T) {
    if cc.bzz && cc.aik {
        abp.push(Cd {
            gb: InsightCategory::Tv,
            qj: InsightSeverity::V,
            dq: String::from("Network active"),
            eu: String::from("Could receive training data or firmware updates"),
            hr: String::from("Enable network mentoring protocol alongside serial"),
        });
    } else if !cc.bzz {
        abp.push(Cd {
            gb: InsightCategory::Nx,
            qj: InsightSeverity::V,
            dq: String::from("Isolated system (no network)"),
            eu: String::from("Air-gapped — good for security analysis"),
            hr: String::from("Use serial mentoring only for external communication"),
        });
    }
}

fn qic(abp: &mut Vec<Cd>, cc: &T) {
    if cc.cfe {
        abp.push(Cd {
            gb: InsightCategory::De,
            qj: InsightSeverity::V,
            dq: String::from("Hardware AES available"),
            eu: String::from("AES-NI for fast encryption of sensitive data"),
            hr: String::from("Use HW AES for weight encryption at rest"),
        });
    }

    if !cc.crd {
        abp.push(Cd {
            gb: InsightCategory::De,
            qj: InsightSeverity::Os,
            dq: String::from("No hardware RNG"),
            eu: String::from("RDRAND not available — using software PRNG"),
            hr: String::from("Seed PRNG from TSC entropy, use longer initialization"),
        });
    }

    
    abp.push(Cd {
        gb: InsightCategory::De,
        qj: InsightSeverity::V,
        dq: format!("Running at {} on {}", cc.jjz, cc.arch),
        eu: String::from("Full hardware access — no OS filtering"),
        hr: String::from("Direct register access enabled for all probing"),
    });
}

fn qhv(abp: &mut Vec<Cd>, cc: &T) {
    match cc.arch {
        "x86_64" => {
            abp.push(Cd {
                gb: InsightCategory::Nx,
                qj: InsightSeverity::V,
                dq: String::from("x86_64 platform"),
                eu: String::from("Full CPUID, PCI, ACPI detection available"),
                hr: String::from("Use native SSE2/AVX paths for max performance"),
            });
        }
        "aarch64" => {
            abp.push(Cd {
                gb: InsightCategory::Tv,
                qj: InsightSeverity::Os,
                dq: String::from("AArch64 platform"),
                eu: String::from("NEON SIMD + TrustZone probing available"),
                hr: String::from("Scan TrustZone boundaries, use NEON for inference"),
            });
        }
        "riscv64" => {
            abp.push(Cd {
                gb: InsightCategory::Tv,
                qj: InsightSeverity::Os,
                dq: String::from("RISC-V platform"),
                eu: String::from("Native RISC-V — no translation needed"),
                hr: String::from("Run binaries natively, use V-extension if available"),
            });
        }
        _ => {}
    }
}






pub fn nxn(cc: &T) -> Si {
    
    let fut = if cc.bqz {
        SimdTier::Ate
    } else if cc.drm {
        SimdTier::Apj
    } else if cc.bzx {
        SimdTier::Ow
    } else if cc.dro {
        SimdTier::Qu
    } else if cc.arch == "aarch64" {
        SimdTier::Tp
    } else {
        SimdTier::Bsf
    };

    
    let bll = match fut {
        SimdTier::Apj => 32,
        SimdTier::Ow => 16,
        SimdTier::Qu => 8,
        SimdTier::Ate => 64,
        _ => 4,
    };

    
    let amo = cc.ccf / (1024 * 1024);
    let osv = if amo >= 4096 { 16 }
        else if amo >= 2048 { 8 }
        else if amo >= 1024 { 4 }
        else if amo >= 512 { 2 }
        else { 1 };

    
    let olt = if amo >= 2048 { 256 }
        else if amo >= 512 { 128 }
        else { 64 };

    
    let pzs = if cc.azj > 2 {
        cc.azj - 1 
    } else {
        1
    };

    
    let jea = if cc.cwl > 0.7 { 1.5 }
        else if cc.cwl > 0.4 { 1.0 }
        else { 0.5 };

    
    let myz = cc.azj >= 4 && amo >= 1024;

    let ibx = format!(
        "{}→{}tile, {}batch, {}workers, lr×{:.1}, gen≤{}tok, bg={}",
        fut.as_str(), bll, osv, pzs,
        jea, olt, myz
    );

    Si {
        fut,
        mok: cc.bqz,
        jhx: osv,
        bll,
        mqw: pzs,
        olu: olt,
        jea,
        kbu: myz,
        ocp: true,
        ibx,
    }
}






pub fn zit(fhl: u64) {
    UC_.fetch_add(1, Ordering::Relaxed);
    AIY_.fetch_add(fhl, Ordering::Relaxed);

    
    let az = UC_.load(Ordering::Relaxed);
    if az > 10 {
        let abl = AIY_.load(Ordering::Relaxed) / az;
        if fhl > abl * 3 {
            AKP_.fetch_add(1, Ordering::Relaxed);
            crate::serial_println!("[JARVIS-HW] Anomaly: inference took {}µs (avg={}µs)",
                fhl, abl);
        }
    }
}


pub fn mxc() -> u64 {
    let az = UC_.load(Ordering::Relaxed);
    if az == 0 { return 0; }
    AIY_.load(Ordering::Relaxed) / az
}


pub fn mvu() -> u64 {
    AKP_.load(Ordering::Relaxed)
}






pub fn svu(abp: &[Cd]) -> String {
    let mut e = String::new();
    e.t("\x01C═══ JARVIS Hardware Analysis ═══\x01W\n\n");

    for ckz in abp {
        e.t(&format!("{}{} [{}]\x01W {}\n",
            ckz.gb.s(),
            ckz.gb.as_str(),
            ckz.qj.as_str(),
            ckz.dq));
        e.t(&format!("  {}\n", ckz.eu));
        e.t(&format!("  \x01G→ {}\x01W\n\n", ckz.hr));
    }

    
    let az = UC_.load(Ordering::Relaxed);
    if az > 0 {
        e.t(&format!("\x01Y═══ Live Monitor ═══\x01W\n"));
        e.t(&format!("  Inferences: {}\n", az));
        e.t(&format!("  Avg latency: {} µs\n", mxc()));
        e.t(&format!("  Anomalies: {}\n", mvu()));
    }

    e
}


pub fn nvr(aqg: &Si) -> String {
    let mut e = String::new();
    e.t("\x01C═══ JARVIS Execution Plan ═══\x01W\n\n");
    e.t(&format!("  SIMD:         {}\n", aqg.fut.as_str()));
    e.t(&format!("  GPU:          {}\n", if aqg.mok { "ENABLED" } else { "disabled" }));
    e.t(&format!("  Batch size:   {}\n", aqg.jhx));
    e.t(&format!("  Tile size:    {}×{}\n", aqg.bll, aqg.bll));
    e.t(&format!("  Workers:      {}\n", aqg.mqw));
    e.t(&format!("  Max gen:      {} tokens\n", aqg.olu));
    e.t(&format!("  LR factor:    ×{:.1}\n", aqg.jea));
    e.t(&format!("  Background:   {}\n", if aqg.kbu { "ON" } else { "OFF" }));
    e.t(&format!("  Monitoring:   {}\n", if aqg.ocp { "ON" } else { "OFF" }));
    e.t(&format!("\n  \x01CStrategy:\x01W {}\n", aqg.ibx));
    e
}
