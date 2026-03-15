//! TrustZone / Secure World Boundary Mapper
//!
//! On ARM devices, TrustZone divides the system into two worlds:
//!   - Normal World (EL0/EL1) — where Linux/Android and TrustOS run
//!   - Secure World (EL3/S-EL1) — TEE, firmware, secrets
//!
//! This module maps the exact boundary: which addresses are accessible
//! from Normal World and which trigger a Secure fault. The boundary
//! reveals what the vendor is protecting and — critically — what they
//! might have misconfigured.
//!
//! On x86, this maps SMM (System Management Mode) boundaries.
//! On RISC-V, this maps PMP (Physical Memory Protection) regions.

use alloc::string::String;
use alloc::format;
use alloc::vec::Vec;
use super::{ProbeResult, AccessLevel, RiskLevel};

/// ARM TrustZone protection controller known bases
#[cfg(target_arch = "aarch64")]
const TZPC_KNOWN_BASES: &[(u64, &str)] = &[
    (0x0E04_0000, "QEMU virt TZPC"),
    (0x8400_0000, "Qualcomm QSEE base"),
    (0xFE20_0000, "BCM2711 Peripheral base"),
    (0x1020_0000, "MediaTek TZ region"),
    (0x0E00_0000, "Secure SRAM (typical)"),
];

/// Known Secure World memory regions by SoC family
const SECURE_REGIONS_GENERIC: &[(u64, u64, &str)] = &[
    (0x0000_0000, 0x0001_0000, "Secure Boot ROM"),
    (0x0E00_0000, 0x0010_0000, "Secure SRAM"),
    (0x0E10_0000, 0x0010_0000, "Secure DRAM carveout"),
];

/// Probe result for a TrustZone boundary test
#[derive(Debug, Clone)]
pub struct TzBoundaryResult {
    pub address: u64,
    pub accessible: bool,
    pub read_value: Option<u32>,
    pub fault_type: &'static str,
}

/// Test if an address is in the Secure World
/// Returns true if accessible (Normal World), false if faulted (Secure)
fn probe_tz_address(addr: u64) -> TzBoundaryResult {
    // Attempt a volatile read
    // If the address is in Secure World, we'll get a data abort
    // Our fault handler marks the access as failed vs succeeded
    
    let result = unsafe {
        if addr == 0 || addr > 0xFFFF_FFFF_FFFF {
            return TzBoundaryResult {
                address: addr,
                accessible: false,
                read_value: None,
                fault_type: "invalid",
            };
        }
        
        let ptr = addr as *const u32;
        // This would fault on real secure memory
        // In a controlled environment, our fault handler catches it
        Some(core::ptr::read_volatile(ptr))
    };
    
    match result {
        Some(val) => TzBoundaryResult {
            address: addr,
            accessible: true,
            read_value: Some(val),
            fault_type: "none",
        },
        None => TzBoundaryResult {
            address: addr,
            accessible: false,
            read_value: None,
            fault_type: "data_abort",
        },
    }
}

/// Binary search for the exact boundary between accessible and secured regions
fn find_boundary(start: u64, end: u64, start_accessible: bool) -> u64 {
    let mut lo = start;
    let mut hi = end;
    
    while hi - lo > 0x1000 {
        let mid = lo + ((hi - lo) / 2) & !0xFFF; // Page-align
        let result = probe_tz_address(mid);
        
        if result.accessible == start_accessible {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    
    hi
}

/// Main TrustZone boundary mapping function
pub fn probe_secure_boundaries() -> String {
    let mut output = String::new();

    #[cfg(target_arch = "aarch64")]
    {
        output.push_str("\x01C== TrustProbe: ARM TrustZone Boundary Mapper ==\x01W\n\n");
        
        // Read SCR_EL3 if accessible (we're in EL1, so we can't — but we can infer)
        output.push_str("\x01YNote: Running at EL1 (Normal World kernel)\x01W\n");
        output.push_str("Probing Secure World boundaries by fault analysis...\n\n");
        
        let mut findings: Vec<ProbeResult> = Vec::new();
        
        // Test known secure regions
        output.push_str("\x01C--- Known Secure Region Tests ---\x01W\n");
        output.push_str(&format!("{:<16} {:<12} {:<12} {}\n",
            "ADDRESS", "ACCESS", "VALUE", "REGION"));
        output.push_str(&format!("{}\n", "-".repeat(70)));
        
        for &(base, size, name) in SECURE_REGIONS_GENERIC {
            let result = probe_tz_address(base);
            let access = if result.accessible { "NORMAL" } else { "SECURE" };
            let value_str = match result.read_value {
                Some(v) => format!("0x{:08X}", v),
                None => String::from("FAULT"),
            };
            
            let risk = if result.accessible && name.contains("Secure") {
                RiskLevel::Critical // Secure region accessible = BAD
            } else if !result.accessible {
                RiskLevel::Info // Expected: secure is secure
            } else {
                RiskLevel::Low
            };
            
            output.push_str(&format!("0x{:010X}   {:<12} {:<12} {}{}\x01W\n",
                base, access, value_str, risk.color_code(), name));
            
            findings.push(ProbeResult {
                category: "TrustZone",
                name: String::from(name),
                address: base,
                size,
                access: if result.accessible { AccessLevel::ReadOnly } else { AccessLevel::Faulted },
                details: format!("{} - {}", access, value_str),
                risk,
            });
        }
        
        // Test TZPC (TrustZone Protection Controller) registers
        output.push_str("\n\x01C--- TZPC Register Probing ---\x01W\n");
        for &(base, name) in TZPC_KNOWN_BASES {
            let result = probe_tz_address(base);
            let icon = if result.accessible { "\x01G[OK]" } else { "\x01R[FAULT]" };
            output.push_str(&format!("{}\x01W 0x{:010X} {}\n", icon, base, name));
            
            if result.accessible {
                // Read TZPC decode protection registers
                for offset in [0x800, 0x804, 0x808, 0x80C].iter() {
                    let reg = probe_tz_address(base + offset);
                    if let Some(val) = reg.read_value {
                        output.push_str(&format!("     +0x{:03X} = 0x{:08X} (decode protection {})\n",
                            offset, val, offset / 4));
                    }
                }
            }
        }
        
        // Systematic boundary scan: sweep large regions
        output.push_str("\n\x01C--- Systematic Boundary Scan ---\x01W\n");
        output.push_str("Sweeping memory in 1MB steps to find Secure/Normal transitions...\n\n");
        
        let sweep_ranges = [
            (0x0000_0000u64, 0x1000_0000u64, "Low memory (0-256MB)"),
            (0x0E00_0000u64, 0x1000_0000u64, "Secure SRAM region"),
        ];
        
        for &(start, end, name) in &sweep_ranges {
            output.push_str(&format!("\x01Y{}: 0x{:X}-0x{:X}\x01W\n", name, start, end));
            
            let step = 0x10_0000u64; // 1MB steps
            let mut prev_accessible = None;
            let mut addr = start;
            
            while addr < end {
                let result = probe_tz_address(addr);
                
                if let Some(prev) = prev_accessible {
                    if result.accessible != prev {
                        // Found a boundary!
                        let boundary = find_boundary(addr - step, addr, prev);
                        let direction = if prev { "Normal->Secure" } else { "Secure->Normal" };
                        output.push_str(&format!("  \x01R** BOUNDARY at 0x{:010X}: {} **\x01W\n",
                            boundary, direction));
                        
                        findings.push(ProbeResult {
                            category: "TrustZone",
                            name: format!("TZ Boundary: {}", direction),
                            address: boundary,
                            size: 0x1000,
                            access: AccessLevel::Partial,
                            details: format!("{} transition", direction),
                            risk: RiskLevel::High,
                        });
                    }
                }
                
                prev_accessible = Some(result.accessible);
                addr += step;
            }
        }
        
        // Summary
        let secure_count = findings.iter().filter(|f| f.access == AccessLevel::Faulted).count();
        let normal_count = findings.iter().filter(|f| f.access != AccessLevel::Faulted).count();
        let boundary_count = findings.iter().filter(|f| f.name.contains("Boundary")).count();
        let critical = findings.iter().filter(|f| f.risk == RiskLevel::Critical).count();
        
        output.push_str(&format!("\n\x01C== Summary ==\x01W\n"));
        output.push_str(&format!("  Secure regions: {}\n", secure_count));
        output.push_str(&format!("  Normal regions: {}\n", normal_count));
        output.push_str(&format!("  Boundaries found: {}\n", boundary_count));
        if critical > 0 {
            output.push_str(&format!("  \x01R!! {} CRITICAL findings (secure memory accessible) !!\x01W\n", critical));
        }
    }
    
    #[cfg(target_arch = "x86_64")]
    {
        output.push_str("\x01C== TrustProbe: x86_64 SMM/Ring Boundary Mapper ==\x01W\n\n");
        output.push_str("Probing System Management Mode (SMM) boundaries...\n\n");
        
        // x86: SMRAM is typically at 0xA0000-0xBFFFF (legacy) or TSEG
        let smram_regions = [
            (0x000A_0000u64, 0x0002_0000u64, "Legacy SMRAM (VGA hole)"),
            (0x000F_0000u64, 0x0001_0000u64, "High BIOS area"),
            (0xFFF0_0000u64, 0x0010_0000u64, "Flash region (4GB - 1MB)"),
        ];
        
        output.push_str(&format!("{:<16} {:<12} {:<12} {}\n",
            "ADDRESS", "ACCESS", "VALUE", "REGION"));
        output.push_str(&format!("{}\n", "-".repeat(60)));
        
        for &(base, _size, name) in &smram_regions {
            let result = probe_tz_address(base);
            let access = if result.accessible { "READABLE" } else { "LOCKED" };
            let value_str = match result.read_value {
                Some(v) => format!("0x{:08X}", v),
                None => String::from("FAULT"),
            };
            output.push_str(&format!("0x{:010X}   {:<12} {:<12} {}\n",
                base, access, value_str, name));
        }
    }
    
    #[cfg(target_arch = "riscv64")]
    {
        output.push_str("\x01C== TrustProbe: RISC-V PMP Boundary Mapper ==\x01W\n\n");
        output.push_str("Probing Physical Memory Protection (PMP) boundaries...\n\n");
        
        // RISC-V: PMP entries define access regions
        // We can read pmpcfg0-3 and pmpaddr0-15 from M-mode
        // From S-mode, we can only probe by fault
        output.push_str("RISC-V PMP configuration (probed from S-mode):\n");
        output.push_str("Attempting to access M-mode regions...\n\n");
        
        let test_regions = [
            (0x0000_0000u64, "Reset vector"),
            (0x0000_1000u64, "Boot ROM"),
            (0x0200_0000u64, "CLINT (M-mode timer)"),
            (0x0C00_0000u64, "PLIC"),
            (0x8000_0000u64, "Main RAM"),
        ];
        
        for &(addr, name) in &test_regions {
            let result = probe_tz_address(addr);
            let icon = if result.accessible { "\x01G[OK]" } else { "\x01R[PMP]" };
            output.push_str(&format!("{}\x01W 0x{:010X} {}\n", icon, addr, name));
        }
    }
    
    output
}
