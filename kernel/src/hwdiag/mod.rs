//! HwDbg — Universal Hardware Debugger Toolkit
//!
//! Designed for PXE-booting TrustOS on any machine and running comprehensive
//! hardware diagnostics. All output goes to both screen and serial (115200 8N1)
//! so a remote operator (or Copilot via serial capture) can analyze results.
//!
//! Commands (from TrustOS shell):
//!   hwdbg auto                  — Run ALL diagnostics, full machine profile
//!   hwdbg cpu                   — Deep CPU analysis (all CPUID, topology, µcode)
//!   hwdbg mem [size_mb]         — Memory test (walking 1s, pattern, stress)
//!   hwdbg pci [bus:dev.fn]      — Full PCI tree with BARs & capabilities
//!   hwdbg acpi                  — Dump all ACPI tables (RSDP→XSDT→*)
//!   hwdbg storage               — Disk/NVMe detection & identify
//!   hwdbg thermal               — Thermal sensors, power state, fans
//!   hwdbg net                   — Network interface inventory
//!   hwdbg stress [seconds]      — CPU + memory stress test
//!   hwdbg remote                — Start structured serial debug protocol
//!   hwdbg export                — Dump everything as machine-parseable report
//!   hwdbg help                  — Show this help

pub mod cpu_deep;
pub mod mem_test;
pub mod pci_deep;
pub mod acpi_dump;
pub mod storage;
pub mod thermal;
pub mod net_hw;
pub mod stress;
pub mod remote;
pub mod pci_raw;
pub mod regdiff;
pub mod ioscan;
pub mod regwatch;
pub mod pci_aer;
pub mod boot_timing;

use alloc::string::String;
use alloc::format;
use alloc::vec::Vec;

/// Dual-output: print to screen AND serial
macro_rules! dbg_out {
    ($($arg:tt)*) => {{
        let s = alloc::format!($($arg)*);
        crate::println!("{}", s);
        crate::serial_println!("{}", s);
    }};
}

pub(crate) use dbg_out;

/// Section header for diagnostic output
fn section_header(title: &str) {
    let bar = "═".repeat(62);
    dbg_out!("╔{}╗", bar);
    let pad = (62i32 - title.len() as i32) / 2;
    let pad = if pad < 1 { 1 } else { pad as usize };
    let right_pad = 62 - pad - title.len();
    let right_pad = if right_pad < 0 { 0usize } else { right_pad as usize };
    dbg_out!("║{}{}{}", " ".repeat(pad), title, " ".repeat(right_pad));
    dbg_out!("╚{}╝", bar);
}

fn sub_header(title: &str) {
    dbg_out!("━━━ {} ━━━", title);
}

/// Main dispatcher for `hwdbg` shell command
pub fn handle_hwdbg_command(args: &[&str]) {
    let subcmd = args.first().copied().unwrap_or("help");

    match subcmd {
        "auto" | "all" | "full" => cmd_auto(),
        "cpu" => cpu_deep::run(args.get(1..).unwrap_or(&[])),
        "mem" | "memory" => {
            let size_mb = args.get(1).and_then(|s| s.parse::<usize>().ok()).unwrap_or(16);
            mem_test::run(size_mb);
        }
        "pci" => pci_deep::run(args.get(1..).unwrap_or(&[])),
        "acpi" => acpi_dump::run(args.get(1..).unwrap_or(&[])),
        "storage" | "disk" | "nvme" => storage::run(),
        "thermal" | "temp" | "power" => thermal::run(),
        "net" | "network" | "nic" => net_hw::run(),
        "stress" | "burn" => {
            let secs = args.get(1).and_then(|s| s.parse::<u64>().ok()).unwrap_or(10);
            stress::run(secs);
        }
        "remote" | "serial" => remote::start_remote_session(),
        "pciraw" | "rawpci" => pci_raw::run(args.get(1..).unwrap_or(&[])),
        "regdiff" | "rdiff" => regdiff::run(args.get(1..).unwrap_or(&[])),
        "ioscan" | "io" => ioscan::run(args.get(1..).unwrap_or(&[])),
        "regwatch" | "watch" => regwatch::run(args.get(1..).unwrap_or(&[])),
        "aer" => pci_aer::run(args.get(1..).unwrap_or(&[])),
        "timing" | "boottiming" => boot_timing::run(args.get(1..).unwrap_or(&[])),
        "export" | "dump" => cmd_export(),
        "help" | _ => cmd_help(),
    }
}

/// Run ALL diagnostics — the "PXE boot and tell me everything" command
fn cmd_auto() {
    section_header("TRUSTOS UNIVERSAL HARDWARE DEBUGGER");
    dbg_out!("Timestamp: boot + {} ms", crate::time::uptime_ms());
    dbg_out!("Architecture: {}", if cfg!(target_arch = "x86_64") { "x86_64" }
             else if cfg!(target_arch = "aarch64") { "aarch64" }
             else if cfg!(target_arch = "riscv64") { "riscv64" }
             else { "unknown" });
    dbg_out!("");

    // Phase 1: CPU
    sub_header("PHASE 1: CPU IDENTIFICATION");
    cpu_deep::run(&[]);
    dbg_out!("");

    // Phase 2: Memory
    sub_header("PHASE 2: MEMORY MAP & TEST");
    mem_test::run(8); // Quick 8MB test for auto mode
    dbg_out!("");

    // Phase 3: ACPI
    sub_header("PHASE 3: ACPI TABLES");
    acpi_dump::run(&[]);
    dbg_out!("");

    // Phase 4: PCI
    sub_header("PHASE 4: PCI DEVICE TREE");
    pci_deep::run(&[]);
    dbg_out!("");

    // Phase 5: Storage
    sub_header("PHASE 5: STORAGE CONTROLLERS");
    storage::run();
    dbg_out!("");

    // Phase 6: Thermal
    sub_header("PHASE 6: THERMAL & POWER");
    thermal::run();
    dbg_out!("");

    // Phase 7: Network
    sub_header("PHASE 7: NETWORK INTERFACES");
    net_hw::run();
    dbg_out!("");

    // Phase 8: PCIe AER
    sub_header("PHASE 8: PCIe AER ERRORS");
    pci_aer::run(&[]);
    dbg_out!("");

    // Phase 9: Boot Timing
    sub_header("PHASE 9: BOOT TIMING");
    boot_timing::run(&[]);
    dbg_out!("");

    section_header("HARDWARE DEBUG COMPLETE");
    dbg_out!("Capture this output via serial cable (115200 8N1) for analysis.");
    dbg_out!("Or run: hwdbg export   for machine-parseable format.");
}

/// Export structured machine-parseable report
fn cmd_export() {
    // Use remote protocol format for machine parsing
    remote::export_full_report();
}

fn cmd_help() {
    dbg_out!("╔══════════════════════════════════════════════════════════════╗");
    dbg_out!("║  HwDbg — Universal Hardware Debugger for TrustOS           ║");
    dbg_out!("╚══════════════════════════════════════════════════════════════╝");
    dbg_out!("");
    dbg_out!("  hwdbg auto                  Run ALL diagnostics (full machine profile)");
    dbg_out!("  hwdbg cpu                   Deep CPU analysis (CPUID, topology, µcode)");
    dbg_out!("  hwdbg mem [size_mb]         Memory test (default: 16 MB)");
    dbg_out!("  hwdbg pci [bus:dev.fn]      Full PCI tree with BARs & capabilities");
    dbg_out!("  hwdbg acpi                  Dump all ACPI tables (RSDP → XSDT → *)");
    dbg_out!("  hwdbg storage               Disk/NVMe detection & identify");
    dbg_out!("  hwdbg thermal               Thermal sensors, power state, fans");
    dbg_out!("  hwdbg net                   Network interface inventory");
    dbg_out!("  hwdbg stress [seconds]      CPU + memory stress test (default: 10s)");
    dbg_out!("  hwdbg remote                Start structured serial debug protocol");
    dbg_out!("  hwdbg export                Full report in machine-parseable format");
    dbg_out!("");
    dbg_out!("  --- New Hardware Debug Tools ---");
    dbg_out!("  hwdbg pciraw <B:D.F>        Raw hex dump of PCI config space (256B/4KB)");
    dbg_out!("  hwdbg regdiff snap [name]   Snapshot registers, then `regdiff diff` to compare");
    dbg_out!("  hwdbg ioscan [legacy|com|range] Scan I/O port ranges for active devices");
    dbg_out!("  hwdbg regwatch pci|msr|io   Live register monitor (polls for changes)");
    dbg_out!("  hwdbg aer [B:D.F]           PCIe Advanced Error Reporting");
    dbg_out!("  hwdbg timing [slow]         Boot timing profiler (checkpoint deltas)");
    dbg_out!("");
    dbg_out!("Tip: All output is mirrored to serial (115200 8N1) for remote capture.");
    dbg_out!("     PXE boot TrustOS, connect serial cable, run `hwdbg auto`.");
}
