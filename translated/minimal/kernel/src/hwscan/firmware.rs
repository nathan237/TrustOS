













use alloc::string::String;
use alloc::format;
use alloc::vec::Vec;
use super::{RiskLevel};

fn sm(addr: u64) -> Option<u32> {
    if addr == 0 { return None; }
    unsafe {
        let ptr = addr as *const u32;
        Some(core::ptr::read_volatile(ptr))
    }
}

fn jcf(addr: u64) -> Option<u8> {
    if addr == 0 { return None; }
    unsafe {
        let ptr = addr as *const u8;
        Some(core::ptr::read_volatile(ptr))
    }
}


const BZK_: &[(u32, &str, RiskLevel)] = &[
    
    (0xAA64_0020, "ARM64 kernel/BL image header", RiskLevel::High),
    (0xE12F_FF1E, "ARM32 BX LR (function return)", RiskLevel::Medium),
    
    
    (0x4F42_4F4F, "Android BOOT magic ('BOOT')", RiskLevel::High),
    (0x4152_444E, "Android sparse image ('ANDR')", RiskLevel::Medium),
    (0x4454_5244, "Android DTB ('DTRD')", RiskLevel::Low),
    
    
    (0x2705_1956, "U-Boot image header (legacy)", RiskLevel::High),
    (0xD00D_FEED, "Device Tree Blob (FDT magic)", RiskLevel::Low),
    
    
    (0x4F50_5445, "OP-TEE header ('OPTE')", RiskLevel::Critical),
    (0x0000_000E, "SMC call convention (FID)", RiskLevel::High),
    
    
    (0x3082_0000, "ASN.1 DER header (possible certificate/key)", RiskLevel::Critical),
    (0x30820122, "RSA-2048 public key (DER)", RiskLevel::Critical),
    (0x30820282, "RSA-4096 public key (DER)", RiskLevel::Critical),
    (0x30770201, "EC private key (DER)", RiskLevel::Critical),
    
    
    (0x0000_0005, "Qualcomm SBL header type", RiskLevel::High),
    (0x7363_0000, "Qualcomm SCM call residue", RiskLevel::High),
    
    
    (0x7F45_4C46, "ELF header", RiskLevel::Medium),
    (0x5A4D_0000, "PE/COFF header (UEFI)", RiskLevel::Medium),
];


const CYF_: &[(&[u8], &str, RiskLevel)] = &[
    (b"-----BEGIN", "PEM key/certificate header", RiskLevel::Critical),
    (b"PRIVATE KEY", "Private key marker", RiskLevel::Critical),
    (b"ssh-rsa", "SSH public key", RiskLevel::High),
    (b"Secure boot", "Secure boot string", RiskLevel::High),
    (b"TZ_LOG", "TrustZone log buffer", RiskLevel::High),
    (b"QSEE", "Qualcomm Secure EE", RiskLevel::High),
    (b"trusty", "Google Trusty TEE", RiskLevel::High),
    (b"OP-TEE", "OP-TEE string", RiskLevel::High),
    (b"BL31", "TF-A BL31 string", RiskLevel::High),
    (b"BL2 ", "TF-A BL2 string", RiskLevel::Medium),
    (b"U-Boot", "U-Boot bootloader", RiskLevel::Medium),
    (b"coreboot", "coreboot firmware", RiskLevel::Medium),
    (b"UEFI", "UEFI firmware", RiskLevel::Low),
    (b"fuse", "Fuse/OTP reference", RiskLevel::High),
    (b"rollback", "Anti-rollback reference", RiskLevel::Medium),
    (b"JTAG", "JTAG reference", RiskLevel::High),
    (b"debug", "Debug reference", RiskLevel::Low),
    (b"password", "Password string", RiskLevel::Critical),
];


fn okz(start: u64, size: u64) -> Vec<(u64, &'static str, RiskLevel)> {
    let mut fw = Vec::new();
    let end = start + size;
    let mut addr = start;
    
    while addr < end {
        if let Some(fx) = sm(addr) {
            for &(magic, name, ref risk) in BZK_ {
                
                if fx == magic || (magic & 0xFFFF_0000 != 0 && fx & 0xFFFF_0000 == magic & 0xFFFF_0000) {
                    fw.push((addr, name, risk.clone()));
                }
            }
        }
        addr += 4;
        
        
        if fw.len() > 100 {
            break;
        }
    }
    
    fw
}


fn ola(start: u64, size: u64) -> Vec<(u64, &'static str, RiskLevel)> {
    let mut fw = Vec::new();
    let end = start + size;
    
    
    let mut addr = start;
    while addr < end {
        
        let mut window = [0u8; 64];
        let mut valid = true;
        
        for i in 0..64u64 {
            match jcf(addr + i) {
                Some(b) => window[i as usize] = b,
                None => { valid = false; break; }
            }
        }
        
        if valid {
            for &(pattern, name, ref risk) in CYF_ {
                if pattern.len() <= 64 {
                    
                    for i in 0..=(64 - pattern.len()) {
                        if &window[i..i + pattern.len()] == pattern {
                            fw.push((addr + i as u64, name, risk.clone()));
                            break;
                        }
                    }
                }
            }
        }
        
        addr += 32; 
        
        if fw.len() > 50 {
            break;
        }
    }
    
    fw
}


pub fn jdb(args: &str) -> String {
    let mut output = String::new();
    
    output.push_str("\x01C== TrustProbe: Firmware Residue Scanner ==\x01W\n\n");
    output.push_str("Searching memory for bootloader/firmware artifacts...\n\n");
    
    
    #[cfg(target_arch = "aarch64")]
    let ezl: Vec<(u64, u64, &str)> = alloc::vec![
        (0x0000_0000, 0x0001_0000, "BootROM / Vector table"),
        (0x0E00_0000, 0x0010_0000, "Secure SRAM"),
        (0x4000_0000, 0x0010_0000, "Low DRAM (BL2 load area)"),
        (0x4020_0000, 0x0010_0000, "BL31 typical load address"),
        (0x6000_0000, 0x0010_0000, "TEE load area (typical)"),
        (0x8000_0000, 0x0010_0000, "Kernel load area"),
    ];
    
    #[cfg(target_arch = "x86_64")]
    let ezl: Vec<(u64, u64, &str)> = alloc::vec![
        (0x0000_0000, 0x0000_1000, "Real-mode IVT / BDA"),
        (0x000E_0000, 0x0002_0000, "BIOS ROM area"),
        (0x000F_0000, 0x0001_0000, "High BIOS"),
        (0x0010_0000, 0x0010_0000, "Extended memory (1MB+)"),
        (0xFFF0_0000, 0x0010_0000, "Flash region (top 1MB)"),
    ];
    
    #[cfg(target_arch = "riscv64")]
    let ezl: Vec<(u64, u64, &str)> = alloc::vec![
        (0x0000_0000, 0x0000_2000, "Reset / Boot ROM"),
        (0x2000_0000, 0x0010_0000, "Flash (typical)"),
        (0x8000_0000, 0x0010_0000, "RAM start (OpenSBI area)"),
        (0x8020_0000, 0x0010_0000, "Kernel load area"),
    ];
    
    let mut gzo = 0;
    let mut dlr = Vec::new();
    
    for (base, size, region_name) in &ezl {
        output.push_str(&format!("\x01Y--- {} (0x{:08X} - 0x{:08X}) ---\x01W\n",
            region_name, base, base + size));
        
        
        let ilo = okz(*base, *size);
        let jjc = ola(*base, *size);
        
        if ilo.is_empty() && jjc.is_empty() {
            output.push_str("  No artifacts found (clean/zeroed)\n\n");
            continue;
        }
        
        for (addr, name, risk) in &ilo {
            let grv = match risk {
                RiskLevel::Critical => "\x01R[CRITICAL]",
                RiskLevel::High => "\x01R[HIGH]",
                RiskLevel::Medium => "\x01Y[MEDIUM]",
                _ => "\x01W[INFO]",
            };
            
            output.push_str(&format!("  {} 0x{:010X}: {}\x01W\n", grv, addr, name));
            
            
            if matches!(risk, RiskLevel::Critical) {
                dlr.push((*addr, *name));
                
                
                output.push_str("    Context: ");
                for i in 0..8u64 {
                    if let Some(fx) = sm(addr + i * 4) {
                        output.push_str(&format!("{:08X} ", fx));
                    }
                }
                output.push_str("\n");
            }
            gzo += 1;
        }
        
        for (addr, name, risk) in &jjc {
            let grv = match risk {
                RiskLevel::Critical => "\x01R[CRITICAL]",
                RiskLevel::High => "\x01R[HIGH]",
                RiskLevel::Medium => "\x01Y[MEDIUM]",
                _ => "\x01W[INFO]",
            };
            
            output.push_str(&format!("  {} 0x{:010X}: {}\x01W\n", grv, addr, name));
            
            if matches!(risk, RiskLevel::Critical | RiskLevel::High) {
                
                output.push_str("    String: \"");
                for i in 0..40u64 {
                    if let Some(b) = jcf(addr + i) {
                        if b >= 0x20 && b < 0x7F {
                            output.push(b as char);
                        } else {
                            output.push('.');
                        }
                    }
                }
                output.push_str("\"\n");
                
                if matches!(risk, RiskLevel::Critical) {
                    dlr.push((*addr, *name));
                }
            }
            gzo += 1;
        }
        
        output.push_str("\n");
    }
    
    
    output.push_str(&format!("\x01C== Firmware Analysis Summary ==\x01W\n"));
    output.push_str(&format!("  Regions scanned: {}\n", ezl.len()));
    output.push_str(&format!("  Total artifacts: {}\n", gzo));
    output.push_str(&format!("  Critical findings: {}\n", dlr.len()));
    
    if !dlr.is_empty() {
        output.push_str(&format!("\n\x01R!! CRITICAL: Sensitive data found in memory !!\x01W\n"));
        for (addr, name) in &dlr {
            output.push_str(&format!("  0x{:010X}: {}\n", addr, name));
        }
        output.push_str("\nThis data was left by the bootloader/firmware and could include:\n");
        output.push_str("  - Signing keys (can forge firmware updates)\n");
        output.push_str("  - Secure World code (can find TEE vulnerabilities)\n");
        output.push_str("  - Debug tokens (can unlock JTAG/debug)\n");
    }
    
    output
}
