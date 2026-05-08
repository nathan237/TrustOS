













use alloc::string::String;
use alloc::format;
use alloc::vec::Vec;
use super::{Dy, AccessLevel, RiskLevel};


#[cfg(target_arch = "aarch64")]
const DCV_: &[(u64, &str)] = &[
    (0x0E04_0000, "QEMU virt TZPC"),
    (0x8400_0000, "Qualcomm QSEE base"),
    (0xFE20_0000, "BCM2711 Peripheral base"),
    (0x1020_0000, "MediaTek TZ region"),
    (0x0E00_0000, "Secure SRAM (typical)"),
];


const CVS_: &[(u64, u64, &str)] = &[
    (0x0000_0000, 0x0001_0000, "Secure Boot ROM"),
    (0x0E00_0000, 0x0010_0000, "Secure SRAM"),
    (0x0E10_0000, 0x0010_0000, "Secure DRAM carveout"),
];


#[derive(Debug, Clone)]
pub struct Qr {
    pub address: u64,
    pub accessible: bool,
    pub read_value: Option<u32>,
    pub fault_type: &'static str,
}



fn coi(addr: u64) -> Qr {
    
    
    
    
    let result = unsafe {
        if addr == 0 || addr > 0xFFFF_FFFF_FFFF {
            return Qr {
                address: addr,
                accessible: false,
                read_value: None,
                fault_type: "invalid",
            };
        }
        
        let ptr = addr as *const u32;
        
        
        Some(core::ptr::read_volatile(ptr))
    };
    
    match result {
        Some(val) => Qr {
            address: addr,
            accessible: true,
            read_value: Some(val),
            fault_type: "none",
        },
        None => Qr {
            address: addr,
            accessible: false,
            read_value: None,
            fault_type: "data_abort",
        },
    }
}


fn lvr(start: u64, end: u64, start_accessible: bool) -> u64 {
    let mut lo = start;
    let mut hi = end;
    
    while hi - lo > 0x1000 {
        let mid = lo + ((hi - lo) / 2) & !0xFFF; 
        let result = coi(mid);
        
        if result.accessible == start_accessible {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    
    hi
}


pub fn iws() -> String {
    let mut output = String::new();

    #[cfg(target_arch = "aarch64")]
    {
        output.push_str("\x01C== TrustProbe: ARM TrustZone Boundary Mapper ==\x01W\n\n");
        
        
        output.push_str("\x01YNote: Running at EL1 (Normal World kernel)\x01W\n");
        output.push_str("Probing Secure World boundaries by fault analysis...\n\n");
        
        let mut fw: Vec<Dy> = Vec::new();
        
        
        output.push_str("\x01C--- Known Secure Region Tests ---\x01W\n");
        output.push_str(&format!("{:<16} {:<12} {:<12} {}\n",
            "ADDRESS", "ACCESS", "VALUE", "REGION"));
        output.push_str(&format!("{}\n", "-".repeat(70)));
        
        for &(base, size, name) in CVS_ {
            let result = coi(base);
            let access = if result.accessible { "NORMAL" } else { "SECURE" };
            let fef = match result.read_value {
                Some(v) => format!("0x{:08X}", v),
                None => String::from("FAULT"),
            };
            
            let risk = if result.accessible && name.contains("Secure") {
                RiskLevel::Critical 
            } else if !result.accessible {
                RiskLevel::Info 
            } else {
                RiskLevel::Low
            };
            
            output.push_str(&format!("0x{:010X}   {:<12} {:<12} {}{}\x01W\n",
                base, access, fef, risk.color_code(), name));
            
            fw.push(Dy {
                category: "TrustZone",
                name: String::from(name),
                address: base,
                size,
                access: if result.accessible { AccessLevel::ReadOnly } else { AccessLevel::Faulted },
                details: format!("{} - {}", access, fef),
                risk,
            });
        }
        
        
        output.push_str("\n\x01C--- TZPC Register Probing ---\x01W\n");
        for &(base, name) in DCV_ {
            let result = coi(base);
            let icon = if result.accessible { "\x01G[OK]" } else { "\x01R[FAULT]" };
            output.push_str(&format!("{}\x01W 0x{:010X} {}\n", icon, base, name));
            
            if result.accessible {
                
                for offset in [0x800, 0x804, 0x808, 0x80C].iter() {
                    let reg = coi(base + offset);
                    if let Some(val) = reg.read_value {
                        output.push_str(&format!("     +0x{:03X} = 0x{:08X} (decode protection {})\n",
                            offset, val, offset / 4));
                    }
                }
            }
        }
        
        
        output.push_str("\n\x01C--- Systematic Boundary Scan ---\x01W\n");
        output.push_str("Sweeping memory in 1MB steps to find Secure/Normal transitions...\n\n");
        
        let ozf = [
            (0x0000_0000u64, 0x1000_0000u64, "Low memory (0-256MB)"),
            (0x0E00_0000u64, 0x1000_0000u64, "Secure SRAM region"),
        ];
        
        for &(start, end, name) in &ozf {
            output.push_str(&format!("\x01Y{}: 0x{:X}-0x{:X}\x01W\n", name, start, end));
            
            let step = 0x10_0000u64; 
            let mut ivy = None;
            let mut addr = start;
            
            while addr < end {
                let result = coi(addr);
                
                if let Some(prev) = ivy {
                    if result.accessible != prev {
                        
                        let hil = lvr(addr - step, addr, prev);
                        let direction = if prev { "Normal->Secure" } else { "Secure->Normal" };
                        output.push_str(&format!("  \x01R** BOUNDARY at 0x{:010X}: {} **\x01W\n",
                            hil, direction));
                        
                        fw.push(Dy {
                            category: "TrustZone",
                            name: format!("TZ Boundary: {}", direction),
                            address: hil,
                            size: 0x1000,
                            access: AccessLevel::Partial,
                            details: format!("{} transition", direction),
                            risk: RiskLevel::High,
                        });
                    }
                }
                
                ivy = Some(result.accessible);
                addr += step;
            }
        }
        
        
        let omw = fw.iter().filter(|f| f.access == AccessLevel::Faulted).count();
        let nku = fw.iter().filter(|f| f.access != AccessLevel::Faulted).count();
        let kdr = fw.iter().filter(|f| f.name.contains("Boundary")).count();
        let aqb = fw.iter().filter(|f| f.risk == RiskLevel::Critical).count();
        
        output.push_str(&format!("\n\x01C== Summary ==\x01W\n"));
        output.push_str(&format!("  Secure regions: {}\n", omw));
        output.push_str(&format!("  Normal regions: {}\n", nku));
        output.push_str(&format!("  Boundaries found: {}\n", kdr));
        if aqb > 0 {
            output.push_str(&format!("  \x01R!! {} CRITICAL findings (secure memory accessible) !!\x01W\n", aqb));
        }
    }
    
    #[cfg(target_arch = "x86_64")]
    {
        output.push_str("\x01C== TrustProbe: x86_64 SMM/Ring Boundary Mapper ==\x01W\n\n");
        output.push_str("Probing System Management Mode (SMM) boundaries...\n\n");
        
        
        let oub = [
            (0x000A_0000u64, 0x0002_0000u64, "Legacy SMRAM (VGA hole)"),
            (0x000F_0000u64, 0x0001_0000u64, "High BIOS area"),
            (0xFFF0_0000u64, 0x0010_0000u64, "Flash region (4GB - 1MB)"),
        ];
        
        output.push_str(&format!("{:<16} {:<12} {:<12} {}\n",
            "ADDRESS", "ACCESS", "VALUE", "REGION"));
        output.push_str(&format!("{}\n", "-".repeat(60)));
        
        for &(base, bek, name) in &oub {
            let result = coi(base);
            let access = if result.accessible { "READABLE" } else { "LOCKED" };
            let fef = match result.read_value {
                Some(v) => format!("0x{:08X}", v),
                None => String::from("FAULT"),
            };
            output.push_str(&format!("0x{:010X}   {:<12} {:<12} {}\n",
                base, access, fef, name));
        }
    }
    
    #[cfg(target_arch = "riscv64")]
    {
        output.push_str("\x01C== TrustProbe: RISC-V PMP Boundary Mapper ==\x01W\n\n");
        output.push_str("Probing Physical Memory Protection (PMP) boundaries...\n\n");
        
        
        
        
        output.push_str("RISC-V PMP configuration (probed from S-mode):\n");
        output.push_str("Attempting to access M-mode regions...\n\n");
        
        let phi = [
            (0x0000_0000u64, "Reset vector"),
            (0x0000_1000u64, "Boot ROM"),
            (0x0200_0000u64, "CLINT (M-mode timer)"),
            (0x0C00_0000u64, "PLIC"),
            (0x8000_0000u64, "Main RAM"),
        ];
        
        for &(addr, name) in &phi {
            let result = coi(addr);
            let icon = if result.accessible { "\x01G[OK]" } else { "\x01R[PMP]" };
            output.push_str(&format!("{}\x01W 0x{:010X} {}\n", icon, addr, name));
        }
    }
    
    output
}
