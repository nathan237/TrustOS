




























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
pub struct Ju {
    pub gb: &'static str,
    pub j: String,
    pub re: u64,
    pub aw: u64,
    pub vz: AccessLevel,
    pub yw: String,
    pub bhz: RiskLevel,
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AccessLevel {
    
    Jx,
    
    Bz,
    
    In,
    
    Ez,
    
    Adq,
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RiskLevel {
    
    V,
    
    Eg,
    
    Bc,
    
    Ao,
    
    Aj,
}

impl AccessLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            AccessLevel::Jx => "RW",
            AccessLevel::Bz => "RO",
            AccessLevel::In => "FAULT",
            AccessLevel::Ez => "DEAD",
            AccessLevel::Adq => "PARTIAL",
        }
    }

    pub fn cpk(&self) -> &'static str {
        match self {
            AccessLevel::Jx => "\x01G",  
            AccessLevel::Bz => "\x01Y",   
            AccessLevel::In => "\x01R",    
            AccessLevel::Ez => "\x01W",       
            AccessLevel::Adq => "\x01M",    
        }
    }
}

impl RiskLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            RiskLevel::V => "INFO",
            RiskLevel::Eg => "LOW",
            RiskLevel::Bc => "MEDIUM",
            RiskLevel::Ao => "HIGH",
            RiskLevel::Aj => "CRITICAL",
        }
    }

    pub fn cpk(&self) -> &'static str {
        match self {
            RiskLevel::V => "\x01W",
            RiskLevel::Eg => "\x01G",
            RiskLevel::Bc => "\x01Y",
            RiskLevel::Ao => "\x01R",
            RiskLevel::Aj => "\x01R",
        }
    }
}


pub struct Aqw {
    pub hd: Vec<Ju>,
    pub arch: &'static str,
    pub dgg: String,
    pub pgg: u64,
}

impl Aqw {
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

        Aqw {
            hd: Vec::new(),
            arch,
            dgg: String::from("Unknown Device"),
            pgg: 0,
        }
    }

    pub fn add(&mut self, result: Ju) {
        self.hd.push(result);
    }

    pub fn hjq(&self, jy: RiskLevel) -> Vec<&Ju> {
        self.hd.iter().hi(|m| m.bhz == jy).collect()
    }

    pub fn ykd(&self, gb: &str) -> usize {
        self.hd.iter().hi(|m| m.gb == gb).az()
    }

    pub fn awz(&self) -> String {
        let es = self.hd.len();
        let cpp = self.hjq(RiskLevel::Aj).len();
        let afq = self.hjq(RiskLevel::Ao).len();
        let gmm = self.hjq(RiskLevel::Bc).len();
        let ail = self.hjq(RiskLevel::Eg).len();
        let co = self.hjq(RiskLevel::V).len();

        format!(
            "Device: {} ({})\n\
             Findings: {} total | {} CRITICAL | {} HIGH | {} MEDIUM | {} LOW | {} INFO\n\
             Scan time: {} ms",
            self.dgg, self.arch,
            es, cpp, afq, gmm, ail, co,
            self.pgg
        )
    }
}


pub fn oaf(n: &[&str]) -> String {
    let air = n.fv().map(|e| *e).unwrap_or("help");

    match air {
        "mmio" => {
            let ar = n.get(1)
                .and_then(|e| ouh(e))
                .unwrap_or(0);
            let aw = n.get(2)
                .and_then(|e| ouh(e))
                .unwrap_or(0);
            mmio::pge(ar, aw)
        }
        "trustzone" | "tz" => {
            trustzone::oxy()
        }
        "dma" => {
            dma::pga()
        }
        "irq" => {
            irq::pgd()
        }
        "gpio" => {
            gpio::oxx()
        }
        "timing" => {
            let kr = n[1..].rr(" ");
            timing::per(&kr)
        }
        "firmware" | "fw" => {
            let kr = n[1..].rr(" ");
            firmware::pgb(&kr)
        }
        "report" => {
            report::tck()
        }
        "dtb" => {
            rdz()
        }
        "verify" => {
            rke()
        }
        "auto" => {
            report::qlk()
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


fn ouh(e: &str) -> Option<u64> {
    if e.cj("0x") || e.cj("0X") {
        u64::wa(&e[2..], 16).bq()
    } else {
        e.parse::<u64>().bq()
    }
}


fn rdz() -> String {
    #[cfg(target_arch = "aarch64")]
    {
        let bqh = crate::android_main::kry();
        if bqh == 0 {
            return String::from("\x01RDTB not available.\x01W No device tree was provided by the bootloader.\n\
                Tip: DTB is passed automatically on Android/ARM boot.\n");
        }
        unsafe {
            if let Some(bez) = dtb_parser::jis(bqh as *const u8) {
                dtb_parser::nvp(&bez)
            } else {
                format!("\x01RDTB parse error.\x01W The DTB at 0x{:X} appears corrupt.\n", bqh)
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








fn rke() -> String {
    let mut bd = String::new();

    bd.t("\x01C== TrustProbe: DTB vs Reality Verification ==\x01W\n\n");

    #[cfg(target_arch = "aarch64")]
    {
        let bqh = crate::android_main::kry();
        if bqh == 0 {
            return String::from("\x01RNo DTB available.\x01W Cannot verify without Device Tree.\n");
        }

        unsafe {
            if let Some(bez) = dtb_parser::jis(bqh as *const u8) {
                bd.t(&format!("DTB Model: {}\n", bez.model));
                bd.t(&format!("DTB declares {} devices with MMIO registers\n\n", bez.ik.len()));

                bd.t(&format!("{:<35} {:<14} {:<10} {:<10} {}\n",
                    "DEVICE", "ADDRESS", "DTB STATUS", "MMIO READ", "VERDICT"));
                bd.t(&format!("{}\n", "-".afd(90)));

                let mut lpx = 0u32;
                let mut iwy = 0u32;
                let mut iyh = 0u32;
                let mut pqd = 0u32;

                for ba in &bez.ik {
                    if ba.cbi == 0 { continue; }

                    let ptr = ba.cbi as *const u32;
                    let duz = core::ptr::read_volatile(ptr);

                    let (igj, s) = if ba.status == "okay" || ba.status == "ok" {
                        if duz == 0 || duz == 0xFFFFFFFF {
                            iwy += 1;
                            ("GHOST (no response)", "\x01Y")
                        } else {
                            lpx += 1;
                            ("OK", "\x01G")
                        }
                    } else {
                        
                        if duz != 0 && duz != 0xFFFFFFFF {
                            iyh += 1;
                            ("HIDDEN ACTIVE!", "\x01R")
                        } else {
                            lpx += 1;
                            ("Disabled (confirmed)", "\x01W")
                        }
                    };

                    
                    let tzc = duz == 0xDEADBEEF 
                        || duz == 0xFEEDFACE
                        || (duz & 0xFF000000 == 0xAA000000);
                    if tzc {
                        pqd += 1;
                    }

                    let j = if ba.bjp.len() > 34 {
                        &ba.bjp[..34]
                    } else {
                        &ba.bjp
                    };

                    bd.t(&format!("{}{:<35}\x01W 0x{:010X}  {:<10} 0x{:08X} {}\n",
                        s, j, ba.cbi, ba.status, duz, igj));
                }

                bd.t(&format!("\n\x01C--- Verification Summary ---\x01W\n"));
                bd.t(&format!("  \x01GConsistent:\x01W {}\n", lpx));
                bd.t(&format!("  \x01YGhost (DTB says OK, no response):\x01W {}\n", iwy));
                bd.t(&format!("  \x01RHidden Active (disabled but responds):\x01W {}\n", iyh));
                bd.t(&format!("  \x01RSuspicious patterns:\x01W {}\n", pqd));

                if iyh > 0 {
                    bd.t(&format!("\n\x01R!! {} devices respond despite being marked disabled by firmware !!\x01W\n", iyh));
                    bd.t("These could be:\n");
                    bd.t("  - Debug interfaces left active by manufacturer\n");
                    bd.t("  - Hardware the vendor tried to hide from Linux\n");
                    bd.t("  - Security-sensitive peripherals (crypto, secure storage)\n");
                }

                if iwy > 0 {
                    bd.t(&format!("\n\x01Y{} ghost devices: declared in DTB but don't respond.\x01W\n", iwy));
                    bd.t("Possible causes: removed in silicon revision, fused off, or clock-gated.\n");
                }
            } else {
                bd.t("\x01RDTB parse failed.\x01W\n");
            }
        }
    }
    #[cfg(not(target_arch = "aarch64"))]
    {
        bd.t("DTB verification is for ARM/RISC-V platforms.\n");
        bd.t("On x86_64, use 'hwscan mmio' to probe MMIO directly.\n");
    }

    bd
}
