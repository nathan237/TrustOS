













use alloc::string::String;
use alloc::format;
use alloc::vec::Vec;
use super::{RiskLevel};

fn akp(ag: u64) -> Option<u32> {
    if ag == 0 { return None; }
    unsafe {
        let ptr = ag as *const u32;
        Some(core::ptr::read_volatile(ptr))
    }
}

fn pfc(ag: u64) -> Option<u8> {
    if ag == 0 { return None; }
    unsafe {
        let ptr = ag as *const u8;
        Some(core::ptr::read_volatile(ptr))
    }
}


const BWE_: &[(u32, &str, RiskLevel)] = &[
    
    (0xAA64_0020, "ARM64 kernel/BL image header", RiskLevel::Ao),
    (0xE12F_FF1E, "ARM32 BX LR (function return)", RiskLevel::Bc),
    
    
    (0x4F42_4F4F, "Android BOOT magic ('BOOT')", RiskLevel::Ao),
    (0x4152_444E, "Android sparse image ('ANDR')", RiskLevel::Bc),
    (0x4454_5244, "Android DTB ('DTRD')", RiskLevel::Eg),
    
    
    (0x2705_1956, "U-Boot image header (legacy)", RiskLevel::Ao),
    (0xD00D_FEED, "Device Tree Blob (FDT magic)", RiskLevel::Eg),
    
    
    (0x4F50_5445, "OP-TEE header ('OPTE')", RiskLevel::Aj),
    (0x0000_000E, "SMC call convention (FID)", RiskLevel::Ao),
    
    
    (0x3082_0000, "ASN.1 DER header (possible certificate/key)", RiskLevel::Aj),
    (0x30820122, "RSA-2048 public key (DER)", RiskLevel::Aj),
    (0x30820282, "RSA-4096 public key (DER)", RiskLevel::Aj),
    (0x30770201, "EC private key (DER)", RiskLevel::Aj),
    
    
    (0x0000_0005, "Qualcomm SBL header type", RiskLevel::Ao),
    (0x7363_0000, "Qualcomm SCM call residue", RiskLevel::Ao),
    
    
    (0x7F45_4C46, "ELF header", RiskLevel::Bc),
    (0x5A4D_0000, "PE/COFF header (UEFI)", RiskLevel::Bc),
];


const CUN_: &[(&[u8], &str, RiskLevel)] = &[
    (b"-----BEGIN", "PEM key/certificate header", RiskLevel::Aj),
    (b"PRIVATE KEY", "Private key marker", RiskLevel::Aj),
    (b"ssh-rsa", "SSH public key", RiskLevel::Ao),
    (b"Secure boot", "Secure boot string", RiskLevel::Ao),
    (b"TZ_LOG", "TrustZone log buffer", RiskLevel::Ao),
    (b"QSEE", "Qualcomm Secure EE", RiskLevel::Ao),
    (b"trusty", "Google Trusty TEE", RiskLevel::Ao),
    (b"OP-TEE", "OP-TEE string", RiskLevel::Ao),
    (b"BL31", "TF-A BL31 string", RiskLevel::Ao),
    (b"BL2 ", "TF-A BL2 string", RiskLevel::Bc),
    (b"U-Boot", "U-Boot bootloader", RiskLevel::Bc),
    (b"coreboot", "coreboot firmware", RiskLevel::Bc),
    (b"UEFI", "UEFI firmware", RiskLevel::Eg),
    (b"fuse", "Fuse/OTP reference", RiskLevel::Ao),
    (b"rollback", "Anti-rollback reference", RiskLevel::Bc),
    (b"JTAG", "JTAG reference", RiskLevel::Ao),
    (b"debug", "Debug reference", RiskLevel::Eg),
    (b"password", "Password string", RiskLevel::Aj),
];


fn wdr(ay: u64, aw: u64) -> Vec<(u64, &'static str, RiskLevel)> {
    let mut nq = Vec::new();
    let ci = ay + aw;
    let mut ag = ay;
    
    while ag < ci {
        if let Some(od) = akp(ag) {
            for &(sj, j, ref bhz) in BWE_ {
                
                if od == sj || (sj & 0xFFFF_0000 != 0 && od & 0xFFFF_0000 == sj & 0xFFFF_0000) {
                    nq.push((ag, j, bhz.clone()));
                }
            }
        }
        ag += 4;
        
        
        if nq.len() > 100 {
            break;
        }
    }
    
    nq
}


fn wds(ay: u64, aw: u64) -> Vec<(u64, &'static str, RiskLevel)> {
    let mut nq = Vec::new();
    let ci = ay + aw;
    
    
    let mut ag = ay;
    while ag < ci {
        
        let mut bh = [0u8; 64];
        let mut blq = true;
        
        for a in 0..64u64 {
            match pfc(ag + a) {
                Some(o) => bh[a as usize] = o,
                None => { blq = false; break; }
            }
        }
        
        if blq {
            for &(pattern, j, ref bhz) in CUN_ {
                if pattern.len() <= 64 {
                    
                    for a in 0..=(64 - pattern.len()) {
                        if &bh[a..a + pattern.len()] == pattern {
                            nq.push((ag + a as u64, j, bhz.clone()));
                            break;
                        }
                    }
                }
            }
        }
        
        ag += 32; 
        
        if nq.len() > 50 {
            break;
        }
    }
    
    nq
}


pub fn pgb(n: &str) -> String {
    let mut an = String::new();
    
    an.t("\x01C== TrustProbe: Firmware Residue Scanner ==\x01W\n\n");
    an.t("Searching memory for bootloader/firmware artifacts...\n\n");
    
    
    #[cfg(target_arch = "aarch64")]
    let jnp: Vec<(u64, u64, &str)> = alloc::vec![
        (0x0000_0000, 0x0001_0000, "BootROM / Vector table"),
        (0x0E00_0000, 0x0010_0000, "Secure SRAM"),
        (0x4000_0000, 0x0010_0000, "Low DRAM (BL2 load area)"),
        (0x4020_0000, 0x0010_0000, "BL31 typical load address"),
        (0x6000_0000, 0x0010_0000, "TEE load area (typical)"),
        (0x8000_0000, 0x0010_0000, "Kernel load area"),
    ];
    
    #[cfg(target_arch = "x86_64")]
    let jnp: Vec<(u64, u64, &str)> = alloc::vec![
        (0x0000_0000, 0x0000_1000, "Real-mode IVT / BDA"),
        (0x000E_0000, 0x0002_0000, "BIOS ROM area"),
        (0x000F_0000, 0x0001_0000, "High BIOS"),
        (0x0010_0000, 0x0010_0000, "Extended memory (1MB+)"),
        (0xFFF0_0000, 0x0010_0000, "Flash region (top 1MB)"),
    ];
    
    #[cfg(target_arch = "riscv64")]
    let jnp: Vec<(u64, u64, &str)> = alloc::vec![
        (0x0000_0000, 0x0000_2000, "Reset / Boot ROM"),
        (0x2000_0000, 0x0010_0000, "Flash (typical)"),
        (0x8000_0000, 0x0010_0000, "RAM start (OpenSBI area)"),
        (0x8020_0000, 0x0010_0000, "Kernel load area"),
    ];
    
    let mut mme = 0;
    let mut hej = Vec::new();
    
    for (ar, aw, lyn) in &jnp {
        an.t(&format!("\x01Y--- {} (0x{:08X} - 0x{:08X}) ---\x01W\n",
            lyn, ar, ar + aw));
        
        
        let oks = wdr(*ar, *aw);
        let ppb = wds(*ar, *aw);
        
        if oks.is_empty() && ppb.is_empty() {
            an.t("  No artifacts found (clean/zeroed)\n\n");
            continue;
        }
        
        for (ag, j, bhz) in &oks {
            let mam = match bhz {
                RiskLevel::Aj => "\x01R[CRITICAL]",
                RiskLevel::Ao => "\x01R[HIGH]",
                RiskLevel::Bc => "\x01Y[MEDIUM]",
                _ => "\x01W[INFO]",
            };
            
            an.t(&format!("  {} 0x{:010X}: {}\x01W\n", mam, ag, j));
            
            
            if oh!(bhz, RiskLevel::Aj) {
                hej.push((*ag, *j));
                
                
                an.t("    Context: ");
                for a in 0..8u64 {
                    if let Some(od) = akp(ag + a * 4) {
                        an.t(&format!("{:08X} ", od));
                    }
                }
                an.t("\n");
            }
            mme += 1;
        }
        
        for (ag, j, bhz) in &ppb {
            let mam = match bhz {
                RiskLevel::Aj => "\x01R[CRITICAL]",
                RiskLevel::Ao => "\x01R[HIGH]",
                RiskLevel::Bc => "\x01Y[MEDIUM]",
                _ => "\x01W[INFO]",
            };
            
            an.t(&format!("  {} 0x{:010X}: {}\x01W\n", mam, ag, j));
            
            if oh!(bhz, RiskLevel::Aj | RiskLevel::Ao) {
                
                an.t("    String: \"");
                for a in 0..40u64 {
                    if let Some(o) = pfc(ag + a) {
                        if o >= 0x20 && o < 0x7F {
                            an.push(o as char);
                        } else {
                            an.push('.');
                        }
                    }
                }
                an.t("\"\n");
                
                if oh!(bhz, RiskLevel::Aj) {
                    hej.push((*ag, *j));
                }
            }
            mme += 1;
        }
        
        an.t("\n");
    }
    
    
    an.t(&format!("\x01C== Firmware Analysis Summary ==\x01W\n"));
    an.t(&format!("  Regions scanned: {}\n", jnp.len()));
    an.t(&format!("  Total artifacts: {}\n", mme));
    an.t(&format!("  Critical findings: {}\n", hej.len()));
    
    if !hej.is_empty() {
        an.t(&format!("\n\x01R!! CRITICAL: Sensitive data found in memory !!\x01W\n"));
        for (ag, j) in &hej {
            an.t(&format!("  0x{:010X}: {}\n", ag, j));
        }
        an.t("\nThis data was left by the bootloader/firmware and could include:\n");
        an.t("  - Signing keys (can forge firmware updates)\n");
        an.t("  - Secure World code (can find TEE vulnerabilities)\n");
        an.t("  - Debug tokens (can unlock JTAG/debug)\n");
    }
    
    an
}
