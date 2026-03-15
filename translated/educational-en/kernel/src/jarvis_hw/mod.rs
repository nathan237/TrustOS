//! JARVIS Hardware Intelligence — AI-Driven Hardware Awareness Layer
//!
//! This module transforms TrustOS from a static kernel into a **self-aware
//! intelligent system** that understands the hardware it's running on.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │              JARVIS Hardware Intelligence v1.0               │
//! │                                                             │
//! │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐   │
//! │  │  probe   │  │ analyzer │  │  media   │  │optimizer │   │
//! │  │          │  │          │  │          │  │          │   │
//! │  │ CPU/PCI  │  │ Insights │  │ ELF/PE   │  │ Boot opt │   │
//! │  │ RAM/GPU  │  │ Planning │  │ FS detect│  │ Live tune│   │
//! │  │ Storage  │  │ Scoring  │  │ RV xlat  │  │ Monitor  │   │
//! │  │ Network  │  │ Monitor  │  │ Strings  │  │ Adaptive │   │
//! │  └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘   │
//! │       │              │             │             │          │
//! │       └──────────────┴─────────────┴─────────────┘          │
//! │                          │                                  │
//! │                   ┌──────┴──────┐                            │
//! │                   │   Jarvis    │                            │
//! │                   │   Neural    │                            │
//! │                   │   Brain     │                            │
//! │                   └─────────────┘                            │
//! │                                                             │
//! │  Data flow: boot → scan → analyze → plan → optimize → loop  │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Boot Sequence
//!
//! When `jarvis boot` is called:
//! 1. **probe** scans every hardware subsystem (CPU, PCI, RAM, GPU, storage, net)
//! 2. **analyzer** generates insights and an execution plan
//! 3. **optimizer** applies initial optimizations and starts monitoring
//! 4. **Jarvis brain** receives hardware context for better reasoning
//!
//! # External Media Analysis
//!
//! When encountering unknown data:
//! 1. **media** detects format (ELF, PE, Mach-O, filesystem, partition table)
//! 2. **riscv_translator** decodes foreign binaries into universal IR
//! 3. **analyzer** assesses security and behavior
//! 4. Jarvis reports findings in natural language
//!
//! # Shell Commands
//!
//! ```text
//! jarvis boot              — Full hardware scan + AI analysis + optimization
//! jarvis hw                — Show cached hardware profile
//! jarvis analyze <file>    — Analyze a binary/media from VFS
//! jarvis insights          — Show AI-generated hardware insights
//! jarvis plan              — Show current execution plan
//! jarvis optimize          — Run one optimization cycle
//! jarvis status            — Show optimizer status
//! ```

pub mod probe;
pub mod analyzer;
pub mod media;
pub mod optimizer;
pub mod query;

use alloc::string::String;
use alloc::format;

/// Master boot command: scan → analyze → optimize → activate
/// Returns the full report as a terminal-displayable string
pub fn boot() -> String {
    optimizer::boot_scan_and_optimize()
}

/// Show the cached hardware profile
pub fn show_profile() -> String {
    if let Some(profile) = probe::cached_profile() {
        profile.format_report()
    } else {
        String::from("\x01RJARVIS hardware scan not yet performed.\x01W\nRun: \x01Cjarvis boot\x01W\n")
    }
}

/// Show AI insights
pub fn show_insights() -> String {
    if let Some(profile) = probe::cached_profile() {
        let insights = analyzer::analyze_hardware(&profile);
        analyzer::format_insights(&insights)
    } else {
        String::from("\x01RNo hardware data. Run: jarvis boot\x01W\n")
    }
}

/// Show current execution plan
pub fn show_plan() -> String {
    if let Some(plan) = optimizer::current_plan() {
        analyzer::format_plan(&plan)
    } else if let Some(profile) = probe::cached_profile() {
        let plan = analyzer::generate_plan(&profile);
        analyzer::format_plan(&plan)
    } else {
        String::from("\x01RNo hardware data. Run: jarvis boot\x01W\n")
    }
}

/// Run one optimization cycle
pub fn optimize_once() -> String {
    if let Some(output) = optimizer::run_optimization_cycle() {
        output
    } else {
        String::from("\x01ROptimizer not active. Run: jarvis boot\x01W\n")
    }
}

/// Show optimizer status
pub fn show_status() -> String {
    optimizer::status()
}

/// Analyze a binary blob (from VFS file read)
pub fn analyze_data(data: &[u8]) -> String {
    let analysis = media::analyze_binary(data);
    let mut output = analysis.format_report();

    // If it's a disk image, also parse partitions
    let format = media::detect_format(data);
    if matches!(format, media::BinaryFormat::Mbr | media::BinaryFormat::Gpt) {
        let parts = media::parse_partitions(data);
        if !parts.is_empty() {
            output.push_str(&media::format_partitions(&parts));
        }
    }

    output
}

/// Inject hardware context into Jarvis neural brain prompt
/// Called before generation to make Jarvis "aware" of its environment
pub fn hardware_context_for_jarvis() -> String {
    if let Some(profile) = probe::cached_profile() {
        let mut context = profile.to_ai_context();

        // Add execution plan info
        if let Some(plan) = optimizer::current_plan() {
            context.push_str(&format!("PLAN: simd={} batch={} gpu={} workers={}\n",
                plan.simd_tier.as_str(), plan.optimal_batch_size,
                plan.use_gpu, plan.worker_threads));
        }

        // Add recent optimization status
        if optimizer::is_active() {
            context.push_str("OPTIMIZER: active, self-tuning enabled\n");
        }

        context
    } else {
        String::from("HARDWARE: not yet scanned\n")
    }
}

/// Answer a natural language hardware query
/// "can you access the encrypted data on this disk?" → reasoned answer
pub fn hardware_query(question: &str) -> String {
    if let Some(profile) = probe::cached_profile() {
        let result = query::answer_query(question, &profile);
        query::format_query_result(&result)
    } else {
        String::from("\x01RHardware not scanned. Run: jarvis boot\x01W\n")
    }
}
