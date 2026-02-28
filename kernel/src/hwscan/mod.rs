//! TrustProbe — Bare-Metal Hardware Security Research Toolkit
//!
//! When TrustOS boots on a device (via fastboot, SD card, JTAG, U-Boot),
//! it runs as the kernel (EL1 on ARM, Ring 0 on x86, S-mode on RISC-V).
//! At this privilege level, we have direct access to every memory-mapped
//! register, every interrupt controller, every DMA engine — without any
//! OS filter obscuring what the hardware actually does.
//!
//! This module provides a systematic toolkit for **hardware cartography**:
//! mapping every piece of silicon, probing every boundary, timing every
//! operation, and generating a complete device profile.
//!
//! Use cases:
//!   - Security research: find undocumented hardware, hidden debug ports
//!   - Reverse engineering: map a new SoC without vendor documentation
//!   - Vulnerability discovery: probe TrustZone/TEE boundaries, DMA paths
//!   - Quality assurance: verify hardware isolation claims
//!
//! Commands (from TrustOS shell):
//!   hwscan mmio [base] [size]    — Scan MMIO regions for responsive devices
//!   hwscan trustzone             — Map Secure/Normal World boundaries
//!   hwscan dma                   — Enumerate DMA engines and capabilities
//!   hwscan irq                   — Map interrupt topology (GIC/APIC)
//!   hwscan gpio                  — Probe GPIO pins for UART/JTAG
//!   hwscan timing [addr]         — Side-channel timing analysis
//!   hwscan firmware              — Scan memory for firmware residue
//!   hwscan report                — Full device reconnaissance report
//!   hwscan auto                  — Run all probes, generate complete map

pub mod mmio;
pub mod trustzone;
pub mod dma;
pub mod irq;
pub mod gpio;
pub mod timing;
pub mod firmware;
pub mod report;

use alloc::string::String;
use alloc::format;
use alloc::vec::Vec;

/// Result of a single hardware probe
#[derive(Debug, Clone)]
pub struct ProbeResult {
    pub category: &'static str,
    pub name: String,
    pub address: u64,
    pub size: u64,
    pub access: AccessLevel,
    pub details: String,
    pub risk: RiskLevel,
}

/// Access level observed during probing
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AccessLevel {
    /// Read + Write accessible
    ReadWrite,
    /// Read-only (writes silently ignored or fault)
    ReadOnly,
    /// Faults on any access (Secure World / reserved)
    Faulted,
    /// Returns fixed pattern (bus error / unmapped)
    Dead,
    /// Partially accessible (some registers fault)
    Partial,
}

/// Security risk assessment
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RiskLevel {
    /// Informational — normal hardware behavior
    Info,
    /// Low — expected but noteworthy access
    Low,
    /// Medium — unexpected access, potential attack surface
    Medium,
    /// High — direct security boundary violation
    High,
    /// Critical — exploitable vulnerability indicator
    Critical,
}

impl AccessLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            AccessLevel::ReadWrite => "RW",
            AccessLevel::ReadOnly => "RO",
            AccessLevel::Faulted => "FAULT",
            AccessLevel::Dead => "DEAD",
            AccessLevel::Partial => "PARTIAL",
        }
    }

    pub fn color_code(&self) -> &'static str {
        match self {
            AccessLevel::ReadWrite => "\x01G",  // Green
            AccessLevel::ReadOnly => "\x01Y",   // Yellow
            AccessLevel::Faulted => "\x01R",    // Red
            AccessLevel::Dead => "\x01W",       // White/dim
            AccessLevel::Partial => "\x01M",    // Magenta
        }
    }
}

impl RiskLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            RiskLevel::Info => "INFO",
            RiskLevel::Low => "LOW",
            RiskLevel::Medium => "MEDIUM",
            RiskLevel::High => "HIGH",
            RiskLevel::Critical => "CRITICAL",
        }
    }

    pub fn color_code(&self) -> &'static str {
        match self {
            RiskLevel::Info => "\x01W",
            RiskLevel::Low => "\x01G",
            RiskLevel::Medium => "\x01Y",
            RiskLevel::High => "\x01R",
            RiskLevel::Critical => "\x01R",
        }
    }
}

/// Accumulated results from all probes
pub struct DeviceMap {
    pub results: Vec<ProbeResult>,
    pub arch: &'static str,
    pub device_name: String,
    pub scan_time_ms: u64,
}

impl DeviceMap {
    pub fn new() -> Self {
        let arch = if cfg!(target_arch = "x86_64") {
            "x86_64"
        } else if cfg!(target_arch = "aarch64") {
            "aarch64"
        } else if cfg!(target_arch = "riscv64") {
            "riscv64"
        } else {
            "unknown"
        };

        DeviceMap {
            results: Vec::new(),
            arch,
            device_name: String::from("Unknown Device"),
            scan_time_ms: 0,
        }
    }

    pub fn add(&mut self, result: ProbeResult) {
        self.results.push(result);
    }

    pub fn findings_by_risk(&self, level: RiskLevel) -> Vec<&ProbeResult> {
        self.results.iter().filter(|r| r.risk == level).collect()
    }

    pub fn count_by_category(&self, category: &str) -> usize {
        self.results.iter().filter(|r| r.category == category).count()
    }

    pub fn summary(&self) -> String {
        let total = self.results.len();
        let critical = self.findings_by_risk(RiskLevel::Critical).len();
        let high = self.findings_by_risk(RiskLevel::High).len();
        let medium = self.findings_by_risk(RiskLevel::Medium).len();
        let low = self.findings_by_risk(RiskLevel::Low).len();
        let info = self.findings_by_risk(RiskLevel::Info).len();

        format!(
            "Device: {} ({})\n\
             Findings: {} total | {} CRITICAL | {} HIGH | {} MEDIUM | {} LOW | {} INFO\n\
             Scan time: {} ms",
            self.device_name, self.arch,
            total, critical, high, medium, low, info,
            self.scan_time_ms
        )
    }
}

/// Main dispatcher for `hwscan` shell command
pub fn handle_hwscan_command(args: &[&str]) -> String {
    let subcmd = args.first().map(|s| *s).unwrap_or("help");

    match subcmd {
        "mmio" => {
            let base = args.get(1)
                .and_then(|s| parse_hex_or_dec(s))
                .unwrap_or(0);
            let size = args.get(2)
                .and_then(|s| parse_hex_or_dec(s))
                .unwrap_or(0);
            mmio::scan_mmio_regions(base, size)
        }
        "trustzone" | "tz" => {
            trustzone::probe_secure_boundaries()
        }
        "dma" => {
            dma::scan_dma_engines()
        }
        "irq" => {
            irq::scan_irq_topology()
        }
        "gpio" => {
            gpio::probe_gpio_pins()
        }
        "timing" => {
            let rest = args[1..].join(" ");
            timing::run_timing_analysis(&rest)
        }
        "firmware" | "fw" => {
            let rest = args[1..].join(" ");
            firmware::scan_firmware_residue(&rest)
        }
        "report" => {
            report::generate_report()
        }
        "auto" => {
            report::auto_scan_all()
        }
        "help" | _ => {
            format!(
                "\x01C== TrustProbe - Hardware Security Research Toolkit ==\x01W\n\
                 \n\
                 \x01YUsage:\x01W hwscan <command> [options]\n\
                 \n\
                 \x01GCommands:\x01W\n\
                 \x01C  mmio\x01W [base] [size]  Scan MMIO regions for responsive devices\n\
                 \x01C  trustzone\x01W           Map Secure/Normal World boundaries (ARM)\n\
                 \x01C  dma\x01W                 Enumerate DMA engines and attack surface\n\
                 \x01C  irq\x01W                 Map interrupt controller topology\n\
                 \x01C  gpio\x01W [pin]           Probe GPIO for UART/JTAG interfaces\n\
                 \x01C  timing\x01W [addr]        Side-channel timing analysis\n\
                 \x01C  firmware\x01W             Scan memory for firmware/bootloader residue\n\
                 \x01C  report\x01W               Generate full device security report\n\
                 \x01C  auto\x01W                 Run ALL probes (comprehensive scan)\n\
                 \n\
                 \x01YExamples:\x01W\n\
                 \x01C  hwscan mmio 0xFE000000 0x2000000\x01W  Scan 32MB at peripheral base\n\
                 \x01C  hwscan trustzone\x01W                   Map TZ boundaries on this SoC\n\
                 \x01C  hwscan auto\x01W                        Full device reconnaissance\n\
                 \n\
                 \x01RNote:\x01W This tool requires bare-metal access (EL1/Ring 0).\n\
                 \x01R      \x01WFor Android devices, flash via fastboot first.\n\
                 \x01R      \x01WFor RPi, boot from SD card."
            )
        }
    }
}

/// Parse a value as hex (0x...) or decimal
fn parse_hex_or_dec(s: &str) -> Option<u64> {
    if s.starts_with("0x") || s.starts_with("0X") {
        u64::from_str_radix(&s[2..], 16).ok()
    } else {
        s.parse::<u64>().ok()
    }
}
