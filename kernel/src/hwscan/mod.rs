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
pub mod dtb_parser;

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
        "dtb" => {
            cmd_dtb_info()
        }
        "verify" => {
            cmd_verify_hardware()
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
                 \x01C  dtb\x01W                  Parse Device Tree Blob (ARM/RISC-V)\n\
                 \x01C  verify\x01W               Cross-reference DTB vs real MMIO\n\
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

/// Parse and display the Device Tree Blob (if available)
fn cmd_dtb_info() -> String {
    #[cfg(target_arch = "aarch64")]
    {
        let dtb_addr = crate::android_main::dtb_address();
        if dtb_addr == 0 {
            return String::from("\x01RDTB not available.\x01W No device tree was provided by the bootloader.\n\
                Tip: DTB is passed automatically on Android/ARM boot.\n");
        }
        unsafe {
            if let Some(parsed) = dtb_parser::parse_dtb(dtb_addr as *const u8) {
                dtb_parser::format_dtb_report(&parsed)
            } else {
                format!("\x01RDTB parse error.\x01W The DTB at 0x{:X} appears corrupt.\n", dtb_addr)
            }
        }
    }
    #[cfg(not(target_arch = "aarch64"))]
    {
        String::from("\x01YDTB: Not applicable on this architecture.\x01W\n\
            Device Tree is used on aarch64 and riscv64 platforms.\n\
            On x86_64, hardware is discovered via ACPI tables.\n")
    }
}

/// Cross-reference DTB declarations against real MMIO probing
///
/// This is the key research value: compare what the firmware SAYS exists
/// vs what ACTUALLY responds. Discrepancies reveal:
///   - Undocumented hardware (responds but not in DTB)
///   - Disabled-but-active devices (DTB says off, MMIO says alive)
///   - Ghost peripherals (in DTB but doesn't respond — removed/fused)
fn cmd_verify_hardware() -> String {
    let mut out = String::new();

    out.push_str("\x01C== TrustProbe: DTB vs Reality Verification ==\x01W\n\n");

    #[cfg(target_arch = "aarch64")]
    {
        let dtb_addr = crate::android_main::dtb_address();
        if dtb_addr == 0 {
            return String::from("\x01RNo DTB available.\x01W Cannot verify without Device Tree.\n");
        }

        unsafe {
            if let Some(parsed) = dtb_parser::parse_dtb(dtb_addr as *const u8) {
                out.push_str(&format!("DTB Model: {}\n", parsed.model));
                out.push_str(&format!("DTB declares {} devices with MMIO registers\n\n", parsed.devices.len()));

                out.push_str(&format!("{:<35} {:<14} {:<10} {:<10} {}\n",
                    "DEVICE", "ADDRESS", "DTB STATUS", "MMIO READ", "VERDICT"));
                out.push_str(&format!("{}\n", "-".repeat(90)));

                let mut ok_count = 0u32;
                let mut ghost_count = 0u32;
                let mut hidden_count = 0u32;
                let mut suspicious_count = 0u32;

                for dev in &parsed.devices {
                    if dev.reg_base == 0 { continue; }

                    let ptr = dev.reg_base as *const u32;
                    let read_result = core::ptr::read_volatile(ptr);

                    let (verdict, color) = if dev.status == "okay" || dev.status == "ok" {
                        if read_result == 0 || read_result == 0xFFFFFFFF {
                            ghost_count += 1;
                            ("GHOST (no response)", "\x01Y")
                        } else {
                            ok_count += 1;
                            ("OK", "\x01G")
                        }
                    } else {
                        // Disabled in DTB
                        if read_result != 0 && read_result != 0xFFFFFFFF {
                            hidden_count += 1;
                            ("HIDDEN ACTIVE!", "\x01R")
                        } else {
                            ok_count += 1;
                            ("Disabled (confirmed)", "\x01W")
                        }
                    };

                    // Detect suspicious patterns
                    let is_suspicious = read_result == 0xDEADBEEF 
                        || read_result == 0xFEEDFACE
                        || (read_result & 0xFF000000 == 0xAA000000);
                    if is_suspicious {
                        suspicious_count += 1;
                    }

                    let name = if dev.compatible.len() > 34 {
                        &dev.compatible[..34]
                    } else {
                        &dev.compatible
                    };

                    out.push_str(&format!("{}{:<35}\x01W 0x{:010X}  {:<10} 0x{:08X} {}\n",
                        color, name, dev.reg_base, dev.status, read_result, verdict));
                }

                out.push_str(&format!("\n\x01C--- Verification Summary ---\x01W\n"));
                out.push_str(&format!("  \x01GConsistent:\x01W {}\n", ok_count));
                out.push_str(&format!("  \x01YGhost (DTB says OK, no response):\x01W {}\n", ghost_count));
                out.push_str(&format!("  \x01RHidden Active (disabled but responds):\x01W {}\n", hidden_count));
                out.push_str(&format!("  \x01RSuspicious patterns:\x01W {}\n", suspicious_count));

                if hidden_count > 0 {
                    out.push_str(&format!("\n\x01R!! {} devices respond despite being marked disabled by firmware !!\x01W\n", hidden_count));
                    out.push_str("These could be:\n");
                    out.push_str("  - Debug interfaces left active by manufacturer\n");
                    out.push_str("  - Hardware the vendor tried to hide from Linux\n");
                    out.push_str("  - Security-sensitive peripherals (crypto, secure storage)\n");
                }

                if ghost_count > 0 {
                    out.push_str(&format!("\n\x01Y{} ghost devices: declared in DTB but don't respond.\x01W\n", ghost_count));
                    out.push_str("Possible causes: removed in silicon revision, fused off, or clock-gated.\n");
                }
            } else {
                out.push_str("\x01RDTB parse failed.\x01W\n");
            }
        }
    }
    #[cfg(not(target_arch = "aarch64"))]
    {
        out.push_str("DTB verification is for ARM/RISC-V platforms.\n");
        out.push_str("On x86_64, use 'hwscan mmio' to probe MMIO directly.\n");
    }

    out
}
