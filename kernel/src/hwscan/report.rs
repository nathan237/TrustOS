//! TrustProbe Report Generator & Auto-Scan Orchestrator
//!
//! This module ties together all hwscan submodules into:
//!   1. `auto_scan_all()` — runs every probe in sequence
//!   2. `generate_report()` — produces a structured security report
//!
//! The auto-scan is the "one button" entry point for a full device
//! security cartography. Flash TrustOS → boot → `hwscan auto` →
//! get a complete map of the hardware's security posture.

use alloc::string::String;
use alloc::format;
use alloc::vec::Vec;
use super::{DeviceMap, ProbeResult, RiskLevel};

/// Run all probes in sequence and collect results
pub fn auto_scan_all() -> String {
    let mut output = String::new();
    
    output.push_str("\x01C");
    output.push_str("╔══════════════════════════════════════════════════════════╗\n");
    output.push_str("║          TrustProbe — Full Device Cartography           ║\n");
    output.push_str("║            Automated Security Assessment                ║\n");
    output.push_str("╚══════════════════════════════════════════════════════════╝\n");
    output.push_str("\x01W\n");
    
    let arch = if cfg!(target_arch = "aarch64") {
        "aarch64 (ARM64)"
    } else if cfg!(target_arch = "x86_64") {
        "x86_64"
    } else if cfg!(target_arch = "riscv64") {
        "riscv64"
    } else {
        "unknown"
    };
    
    output.push_str(&format!("Architecture: {}\n", arch));
    output.push_str(&format!("TrustProbe version: 1.0.0\n\n"));
    
    // Phase 1: MMIO Discovery
    output.push_str("\x01Y[1/7] MMIO Peripheral Discovery...\x01W\n");
    output.push_str(&format!("{}\n", "=".repeat(60)));
    output.push_str(&super::mmio::scan_mmio_regions(0, 0));
    output.push_str("\n");
    
    // Phase 2: Security Boundaries
    output.push_str("\x01Y[2/7] Security Boundary Mapping...\x01W\n");
    output.push_str(&format!("{}\n", "=".repeat(60)));
    output.push_str(&super::trustzone::probe_secure_boundaries());
    output.push_str("\n");
    
    // Phase 3: DMA & IOMMU
    output.push_str("\x01Y[3/7] DMA Engine & IOMMU Audit...\x01W\n");
    output.push_str(&format!("{}\n", "=".repeat(60)));
    output.push_str(&super::dma::scan_dma_engines());
    output.push_str("\n");
    
    // Phase 4: Interrupt Topology
    output.push_str("\x01Y[4/7] Interrupt Controller Mapping...\x01W\n");
    output.push_str(&format!("{}\n", "=".repeat(60)));
    output.push_str(&super::irq::scan_irq_topology());
    output.push_str("\n");
    
    // Phase 5: GPIO / Debug Interfaces
    output.push_str("\x01Y[5/7] GPIO & Debug Interface Discovery...\x01W\n");
    output.push_str(&format!("{}\n", "=".repeat(60)));
    output.push_str(&super::gpio::probe_gpio_pins());
    output.push_str("\n");
    
    // Phase 6: Timing Analysis
    output.push_str("\x01Y[6/7] Timing Side-Channel Analysis...\x01W\n");
    output.push_str(&format!("{}\n", "=".repeat(60)));
    output.push_str(&super::timing::run_timing_analysis(""));
    output.push_str("\n");
    
    // Phase 7: Firmware Residue
    output.push_str("\x01Y[7/7] Firmware Residue Scan...\x01W\n");
    output.push_str(&format!("{}\n", "=".repeat(60)));
    output.push_str(&super::firmware::scan_firmware_residue(""));
    output.push_str("\n");
    
    // Final Report
    output.push_str(&generate_summary_report());
    
    output
}

/// Generate a concise summary report
fn generate_summary_report() -> String {
    let mut output = String::new();
    
    output.push_str("\x01C");
    output.push_str("╔══════════════════════════════════════════════════════════╗\n");
    output.push_str("║          TrustProbe — Executive Summary                 ║\n");
    output.push_str("╚══════════════════════════════════════════════════════════╝\n");
    output.push_str("\x01W\n");
    
    output.push_str("Scan complete. Key findings across all modules:\n\n");
    
    output.push_str("\x01YMMIO Discovery:\x01W\n");
    output.push_str("  Probed memory-mapped peripheral regions\n");
    output.push_str("  Identified controllers by reading hardware ID registers\n\n");
    
    output.push_str("\x01YSecurity Boundaries:\x01W\n");
    #[cfg(target_arch = "aarch64")]
    output.push_str("  Mapped TrustZone Normal/Secure World transitions\n");
    #[cfg(target_arch = "x86_64")]
    output.push_str("  Probed SMM and Ring 0/Ring -1 boundaries\n");
    #[cfg(target_arch = "riscv64")]
    output.push_str("  Mapped PMP (Physical Memory Protection) boundaries\n");
    output.push_str("  Any secure memory accessible from kernel = CRITICAL\n\n");
    
    output.push_str("\x01YDMA & IOMMU:\x01W\n");
    output.push_str("  Enumerated DMA-capable controllers\n");
    output.push_str("  Checked SMMU/IOMMU isolation status\n");
    output.push_str("  Missing IOMMU = all DMA devices can read kernel memory\n\n");
    
    output.push_str("\x01YInterrupts:\x01W\n");
    output.push_str("  Mapped interrupt controller topology\n");
    output.push_str("  Identified active/pending IRQ routing\n\n");
    
    output.push_str("\x01YGPIO & Debug:\x01W\n");
    output.push_str("  Scanned GPIO pin muxing for hidden interfaces\n");
    output.push_str("  Active UART/JTAG = direct debug access possible\n\n");
    
    output.push_str("\x01YTiming Analysis:\x01W\n");
    output.push_str("  Measured access latencies across memory regions\n");
    output.push_str("  Anomalies may indicate hidden secure boundaries\n\n");
    
    output.push_str("\x01YFirmware Residue:\x01W\n");
    output.push_str("  Searched for bootloader/firmware artifacts in memory\n");
    output.push_str("  Keys/certificates/debug tokens left by boot chain\n\n");
    
    output.push_str("\x01C--- Risk Assessment ---\x01W\n");
    output.push_str("Use individual scan commands for detailed results:\n");
    output.push_str("  hwscan mmio      — MMIO peripheral map\n");
    output.push_str("  hwscan trustzone — Security boundary map\n");
    output.push_str("  hwscan dma       — DMA/IOMMU audit\n");
    output.push_str("  hwscan irq       — Interrupt topology\n");
    output.push_str("  hwscan gpio      — Debug interface discovery\n");
    output.push_str("  hwscan timing    — Side-channel analysis\n");
    output.push_str("  hwscan firmware  — Firmware residue scan\n");
    output.push_str("  hwscan report    — This summary\n\n");
    
    output.push_str("\x01C[TrustOS TrustProbe — Bare-Metal Hardware Security Research Platform]\x01W\n");
    
    output
}

/// Generate the report (standalone command)
pub fn generate_report() -> String {
    generate_summary_report()
}
