













use alloc::string::String;
use alloc::format;
use alloc::vec::Vec;
use super::{Ju, AccessLevel, RiskLevel};


#[cfg(target_arch = "aarch64")]
const CZD_: &[(u64, &str)] = &[
    (0x0E04_0000, "QEMU virt TZPC"),
    (0x8400_0000, "Qualcomm QSEE base"),
    (0xFE20_0000, "BCM2711 Peripheral base"),
    (0x1020_0000, "MediaTek TZ region"),
    (0x0E00_0000, "Secure SRAM (typical)"),
];


const CSB_: &[(u64, u64, &str)] = &[
    (0x0000_0000, 0x0001_0000, "Secure Boot ROM"),
    (0x0E00_0000, 0x0010_0000, "Secure SRAM"),
    (0x0E10_0000, 0x0010_0000, "Secure DRAM carveout"),
];


#[derive(Debug, Clone)]
pub struct Anx {
    pub re: u64,
    pub ciy: bool,
    pub gqn: Option<u32>,
    pub kvh: &'static str,
}



fn frf(ag: u64) -> Anx {
    
    
    
    
    let result = unsafe {
        if ag == 0 || ag > 0xFFFF_FFFF_FFFF {
            return Anx {
                re: ag,
                ciy: false,
                gqn: None,
                kvh: "invalid",
            };
        }
        
        let ptr = ag as *const u32;
        
        
        Some(core::ptr::read_volatile(ptr))
    };
    
    match result {
        Some(ap) => Anx {
            re: ag,
            ciy: true,
            gqn: Some(ap),
            kvh: "none",
        },
        None => Anx {
            re: ag,
            ciy: false,
            gqn: None,
            kvh: "data_abort",
        },
    }
}


fn ssz(ay: u64, ci: u64, wsm: bool) -> u64 {
    let mut hh = ay;
    let mut gd = ci;
    
    while gd - hh > 0x1000 {
        let vs = hh + ((gd - hh) / 2) & !0xFFF; 
        let result = frf(vs);
        
        if result.ciy == wsm {
            hh = vs;
        } else {
            gd = vs;
        }
    }
    
    gd
}


pub fn oxy() -> String {
    let mut an = String::new();

    #[cfg(target_arch = "aarch64")]
    {
        an.t("\x01C== TrustProbe: ARM TrustZone Boundary Mapper ==\x01W\n\n");
        
        
        an.t("\x01YNote: Running at EL1 (Normal World kernel)\x01W\n");
        an.t("Probing Secure World boundaries by fault analysis...\n\n");
        
        let mut nq: Vec<Ju> = Vec::new();
        
        
        an.t("\x01C--- Known Secure Region Tests ---\x01W\n");
        an.t(&format!("{:<16} {:<12} {:<12} {}\n",
            "ADDRESS", "ACCESS", "VALUE", "REGION"));
        an.t(&format!("{}\n", "-".afd(70)));
        
        for &(ar, aw, j) in CSB_ {
            let result = frf(ar);
            let vz = if result.ciy { "NORMAL" } else { "SECURE" };
            let jvg = match result.gqn {
                Some(p) => format!("0x{:08X}", p),
                None => String::from("FAULT"),
            };
            
            let bhz = if result.ciy && j.contains("Secure") {
                RiskLevel::Aj 
            } else if !result.ciy {
                RiskLevel::V 
            } else {
                RiskLevel::Eg
            };
            
            an.t(&format!("0x{:010X}   {:<12} {:<12} {}{}\x01W\n",
                ar, vz, jvg, bhz.cpk(), j));
            
            nq.push(Ju {
                gb: "TrustZone",
                j: String::from(j),
                re: ar,
                aw,
                vz: if result.ciy { AccessLevel::Bz } else { AccessLevel::In },
                yw: format!("{} - {}", vz, jvg),
                bhz,
            });
        }
        
        
        an.t("\n\x01C--- TZPC Register Probing ---\x01W\n");
        for &(ar, j) in CZD_ {
            let result = frf(ar);
            let pa = if result.ciy { "\x01G[OK]" } else { "\x01R[FAULT]" };
            an.t(&format!("{}\x01W 0x{:010X} {}\n", pa, ar, j));
            
            if result.ciy {
                
                for l in [0x800, 0x804, 0x808, 0x80C].iter() {
                    let reg = frf(ar + l);
                    if let Some(ap) = reg.gqn {
                        an.t(&format!("     +0x{:03X} = 0x{:08X} (decode protection {})\n",
                            l, ap, l / 4));
                    }
                }
            }
        }
        
        
        an.t("\n\x01C--- Systematic Boundary Scan ---\x01W\n");
        an.t("Sweeping memory in 1MB steps to find Secure/Normal transitions...\n\n");
        
        let wwq = [
            (0x0000_0000u64, 0x1000_0000u64, "Low memory (0-256MB)"),
            (0x0E00_0000u64, 0x1000_0000u64, "Secure SRAM region"),
        ];
        
        for &(ay, ci, j) in &wwq {
            an.t(&format!("\x01Y{}: 0x{:X}-0x{:X}\x01W\n", j, ay, ci));
            
            let gu = 0x10_0000u64; 
            let mut oxf = None;
            let mut ag = ay;
            
            while ag < ci {
                let result = frf(ag);
                
                if let Some(vo) = oxf {
                    if result.ciy != vo {
                        
                        let mzx = ssz(ag - gu, ag, vo);
                        let sz = if vo { "Normal->Secure" } else { "Secure->Normal" };
                        an.t(&format!("  \x01R** BOUNDARY at 0x{:010X}: {} **\x01W\n",
                            mzx, sz));
                        
                        nq.push(Ju {
                            gb: "TrustZone",
                            j: format!("TZ Boundary: {}", sz),
                            re: mzx,
                            aw: 0x1000,
                            vz: AccessLevel::Adq,
                            yw: format!("{} transition", sz),
                            bhz: RiskLevel::Ao,
                        });
                    }
                }
                
                oxf = Some(result.ciy);
                ag += gu;
            }
        }
        
        
        let wgf = nq.iter().hi(|bb| bb.vz == AccessLevel::In).az();
        let uvc = nq.iter().hi(|bb| bb.vz != AccessLevel::In).az();
        let qrn = nq.iter().hi(|bb| bb.j.contains("Boundary")).az();
        let cpp = nq.iter().hi(|bb| bb.bhz == RiskLevel::Aj).az();
        
        an.t(&format!("\n\x01C== Summary ==\x01W\n"));
        an.t(&format!("  Secure regions: {}\n", wgf));
        an.t(&format!("  Normal regions: {}\n", uvc));
        an.t(&format!("  Boundaries found: {}\n", qrn));
        if cpp > 0 {
            an.t(&format!("  \x01R!! {} CRITICAL findings (secure memory accessible) !!\x01W\n", cpp));
        }
    }
    
    #[cfg(target_arch = "x86_64")]
    {
        an.t("\x01C== TrustProbe: x86_64 SMM/Ring Boundary Mapper ==\x01W\n\n");
        an.t("Probing System Management Mode (SMM) boundaries...\n\n");
        
        
        let wpz = [
            (0x000A_0000u64, 0x0002_0000u64, "Legacy SMRAM (VGA hole)"),
            (0x000F_0000u64, 0x0001_0000u64, "High BIOS area"),
            (0xFFF0_0000u64, 0x0010_0000u64, "Flash region (4GB - 1MB)"),
        ];
        
        an.t(&format!("{:<16} {:<12} {:<12} {}\n",
            "ADDRESS", "ACCESS", "VALUE", "REGION"));
        an.t(&format!("{}\n", "-".afd(60)));
        
        for &(ar, dds, j) in &wpz {
            let result = frf(ar);
            let vz = if result.ciy { "READABLE" } else { "LOCKED" };
            let jvg = match result.gqn {
                Some(p) => format!("0x{:08X}", p),
                None => String::from("FAULT"),
            };
            an.t(&format!("0x{:010X}   {:<12} {:<12} {}\n",
                ar, vz, jvg, j));
        }
    }
    
    #[cfg(target_arch = "riscv64")]
    {
        an.t("\x01C== TrustProbe: RISC-V PMP Boundary Mapper ==\x01W\n\n");
        an.t("Probing Physical Memory Protection (PMP) boundaries...\n\n");
        
        
        
        
        an.t("RISC-V PMP configuration (probed from S-mode):\n");
        an.t("Attempting to access M-mode regions...\n\n");
        
        let xex = [
            (0x0000_0000u64, "Reset vector"),
            (0x0000_1000u64, "Boot ROM"),
            (0x0200_0000u64, "CLINT (M-mode timer)"),
            (0x0C00_0000u64, "PLIC"),
            (0x8000_0000u64, "Main RAM"),
        ];
        
        for &(ag, j) in &xex {
            let result = frf(ag);
            let pa = if result.ciy { "\x01G[OK]" } else { "\x01R[PMP]" };
            an.t(&format!("{}\x01W 0x{:010X} {}\n", pa, ag, j));
        }
    }
    
    an
}
