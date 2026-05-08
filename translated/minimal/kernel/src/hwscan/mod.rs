




























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


#[derive(Debug, Clone)]
pub struct Dy {
    pub category: &'static str,
    pub name: String,
    pub address: u64,
    pub size: u64,
    pub access: AccessLevel,
    pub details: String,
    pub risk: RiskLevel,
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AccessLevel {
    
    ReadWrite,
    
    ReadOnly,
    
    Faulted,
    
    Dead,
    
    Partial,
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RiskLevel {
    
    Info,
    
    Low,
    
    Medium,
    
    High,
    
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
            AccessLevel::ReadWrite => "\x01G",  
            AccessLevel::ReadOnly => "\x01Y",   
            AccessLevel::Faulted => "\x01R",    
            AccessLevel::Dead => "\x01W",       
            AccessLevel::Partial => "\x01M",    
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


pub struct Rs {
    pub results: Vec<Dy>,
    pub arch: &'static str,
    pub device_name: String,
    pub scan_time_ms: u64,
}

impl Rs {
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

        Rs {
            results: Vec::new(),
            arch,
            device_name: String::from("Unknown Device"),
            scan_time_ms: 0,
        }
    }

    pub fn add(&mut self, result: Dy) {
        self.results.push(result);
    }

    pub fn findings_by_risk(&self, level: RiskLevel) -> Vec<&Dy> {
        self.results.iter().filter(|r| r.risk == level).collect()
    }

    pub fn qbk(&self, category: &str) -> usize {
        self.results.iter().filter(|r| r.category == category).count()
    }

    pub fn summary(&self) -> String {
        let av = self.results.len();
        let aqb = self.findings_by_risk(RiskLevel::Critical).len();
        let high = self.findings_by_risk(RiskLevel::High).len();
        let dbd = self.findings_by_risk(RiskLevel::Medium).len();
        let low = self.findings_by_risk(RiskLevel::Low).len();
        let info = self.findings_by_risk(RiskLevel::Info).len();

        format!(
            "Device: {} ({})\n\
             Findings: {} total | {} CRITICAL | {} HIGH | {} MEDIUM | {} LOW | {} INFO\n\
             Scan time: {} ms",
            self.device_name, self.arch,
            av, aqb, high, dbd, low, info,
            self.scan_time_ms
        )
    }
}


pub fn idk(args: &[&str]) -> String {
    let je = args.first().map(|j| *j).unwrap_or("help");

    match je {
        "mmio" => {
            let base = args.get(1)
                .and_then(|j| itt(j))
                .unwrap_or(0);
            let size = args.get(2)
                .and_then(|j| itt(j))
                .unwrap_or(0);
            mmio::jdg(base, size)
        }
        "trustzone" | "tz" => {
            trustzone::iws()
        }
        "dma" => {
            dma::jda()
        }
        "irq" => {
            irq::jde()
        }
        "gpio" => {
            gpio::iwr()
        }
        "timing" => {
            let ef = args[1..].join(" ");
            timing::jbw(&ef)
        }
        "firmware" | "fw" => {
            let ef = args[1..].join(" ");
            firmware::jdb(&ef)
        }
        "report" => {
            report::fyi()
        }
        "dtb" => {
            knn()
        }
        "verify" => {
            ktp()
        }
        "auto" => {
            report::jyk()
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


fn itt(j: &str) -> Option<u64> {
    if j.starts_with("0x") || j.starts_with("0X") {
        u64::from_str_radix(&j[2..], 16).ok()
    } else {
        j.parse::<u64>().ok()
    }
}


fn knn() -> String {
    #[cfg(target_arch = "aarch64")]
    {
        let dtb_addr = crate::android_main::ftl();
        if dtb_addr == 0 {
            return String::from("\x01RDTB not available.\x01W No device tree was provided by the bootloader.\n\
                Tip: DTB is passed automatically on Android/ARM boot.\n");
        }
        unsafe {
            if let Some(parsed) = dtb_parser::ewg(dtb_addr as *const u8) {
                dtb_parser::hzo(&parsed)
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








fn ktp() -> String {
    let mut out = String::new();

    out.push_str("\x01C== TrustProbe: DTB vs Reality Verification ==\x01W\n\n");

    #[cfg(target_arch = "aarch64")]
    {
        let dtb_addr = crate::android_main::ftl();
        if dtb_addr == 0 {
            return String::from("\x01RNo DTB available.\x01W Cannot verify without Device Tree.\n");
        }

        unsafe {
            if let Some(parsed) = dtb_parser::ewg(dtb_addr as *const u8) {
                out.push_str(&format!("DTB Model: {}\n", parsed.model));
                out.push_str(&format!("DTB declares {} devices with MMIO registers\n\n", parsed.devices.len()));

                out.push_str(&format!("{:<35} {:<14} {:<10} {:<10} {}\n",
                    "DEVICE", "ADDRESS", "DTB STATUS", "MMIO READ", "VERDICT"));
                out.push_str(&format!("{}\n", "-".repeat(90)));

                let mut gkn = 0u32;
                let mut eog = 0u32;
                let mut epd = 0u32;
                let mut jjz = 0u32;

                for s in &parsed.devices {
                    if s.reg_base == 0 { continue; }

                    let ptr = s.reg_base as *const u32;
                    let bom = core::ptr::read_volatile(ptr);

                    let (edq, color) = if s.status == "okay" || s.status == "ok" {
                        if bom == 0 || bom == 0xFFFFFFFF {
                            eog += 1;
                            ("GHOST (no response)", "\x01Y")
                        } else {
                            gkn += 1;
                            ("OK", "\x01G")
                        }
                    } else {
                        
                        if bom != 0 && bom != 0xFFFFFFFF {
                            epd += 1;
                            ("HIDDEN ACTIVE!", "\x01R")
                        } else {
                            gkn += 1;
                            ("Disabled (confirmed)", "\x01W")
                        }
                    };

                    
                    let mtu = bom == 0xDEADBEEF 
                        || bom == 0xFEEDFACE
                        || (bom & 0xFF000000 == 0xAA000000);
                    if mtu {
                        jjz += 1;
                    }

                    let name = if s.compatible.len() > 34 {
                        &s.compatible[..34]
                    } else {
                        &s.compatible
                    };

                    out.push_str(&format!("{}{:<35}\x01W 0x{:010X}  {:<10} 0x{:08X} {}\n",
                        color, name, s.reg_base, s.status, bom, edq));
                }

                out.push_str(&format!("\n\x01C--- Verification Summary ---\x01W\n"));
                out.push_str(&format!("  \x01GConsistent:\x01W {}\n", gkn));
                out.push_str(&format!("  \x01YGhost (DTB says OK, no response):\x01W {}\n", eog));
                out.push_str(&format!("  \x01RHidden Active (disabled but responds):\x01W {}\n", epd));
                out.push_str(&format!("  \x01RSuspicious patterns:\x01W {}\n", jjz));

                if epd > 0 {
                    out.push_str(&format!("\n\x01R!! {} devices respond despite being marked disabled by firmware !!\x01W\n", epd));
                    out.push_str("These could be:\n");
                    out.push_str("  - Debug interfaces left active by manufacturer\n");
                    out.push_str("  - Hardware the vendor tried to hide from Linux\n");
                    out.push_str("  - Security-sensitive peripherals (crypto, secure storage)\n");
                }

                if eog > 0 {
                    out.push_str(&format!("\n\x01Y{} ghost devices: declared in DTB but don't respond.\x01W\n", eog));
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
