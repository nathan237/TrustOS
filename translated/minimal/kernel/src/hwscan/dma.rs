













use alloc::string::String;
use alloc::format;
use alloc::vec::Vec;
use super::{Ju, AccessLevel, RiskLevel};


#[cfg(target_arch = "aarch64")]
const BRW_: &[(u64, u64, &str)] = &[
    (0x0902_0000, 0x1000, "QEMU virt pl330 (if present)"),
    (0x7E00_7000, 0x1000, "BCM2711 DMA0-6 (Lite)"),
    (0x7E00_7B00, 0x0300, "BCM2711 DMA7-10 (Normal)"),
    (0x7EE0_B000, 0x1000, "BCM2711 DMA11-14 (4K capable)"),
    (0x7E00_E000, 0x1000, "BCM2711 DMA15 (bulk/fast)"),
    (0x0088_4000, 0x1000, "Snapdragon BAM DMA"),
    (0x0C80_0000, 0x4000, "Snapdragon GPI DMA (QUP)"),
];


#[cfg(target_arch = "aarch64")]
const CTN_: &[(u64, u64, &str)] = &[
    (0x0900_0000, 0x2_0000, "QEMU virt SMMUv3"),
    (0x1500_0000, 0x8_0000, "Snapdragon SMMU (apps)"),
    (0x0510_0000, 0x1_0000, "Snapdragon SMMU-500 (GPU)"),
    (0xFD50_0000, 0x1_0000, "BCM2711 IOMMU (if present)"),
];


#[cfg(target_arch = "x86_64")]
const CCF_: &[(u64, &str)] = &[
    (0xFED9_0000, "Intel VT-d DMAR unit (typical)"),
    (0xFED4_0000, "AMD-Vi IVHD base (typical)"),
];


const EAC_: u64 = 0x000; 
const EAB_: u64 = 0x004; 
const EAD_: u64 = 0x030; 
const EAE_: u64 = 0x038; 
const EAA_: u64 = 0xE00; 
const EAF_: u64 = 0xFE0; 


const DDJ_: u64 = 0x00; 
const DDL_: u64 = 0x08; 
const DDK_: u64 = 0x20; 

fn akp(ag: u64) -> Option<u32> {
    if ag == 0 { return None; }
    unsafe {
        let ptr = ag as *const u32;
        Some(core::ptr::read_volatile(ptr))
    }
}


#[cfg(target_arch = "aarch64")]
fn vmd(ar: u64) -> String {
    let mut bd = String::new();
    
    
    if let Some(gjj) = akp(ar) {
        let pev = (gjj >> 0) & 1;   
        let peu = (gjj >> 1) & 1;   
        let xne = (gjj >> 2) & 3;   
        let tqn = (gjj >> 6) & 3;  
        let rlp = (gjj >> 4) & 1; 
        
        bd.t(&format!("  IDR0 = 0x{:08X}\n", gjj));
        bd.t(&format!("    Stage 1: {}  Stage 2: {}\n",
            if peu == 1 { "YES" } else { "NO" },
            if pev == 1 { "YES" } else { "NO" }));
        bd.t(&format!("    Translation format: {}  HTTU: {}  Coherent: {}\n",
            xne, tqn, rlp != 0));
            
        if pev == 0 && peu == 0 {
            bd.t("    \x01R!! SMMU has NO translation stages — DMA is UNPROTECTED !!\x01W\n");
        }
    }
    
    
    if let Some(lde) = akp(ar + 4) {
        let bwj = (lde >> 8) & 0x1F;
        let wrz = (lde >> 6) & 0x1F;
        bd.t(&format!("  IDR1 = 0x{:08X} (queues: {}, SSID bits: {})\n",
            lde, bwj, wrz));
    }
    
    
    if let Some(akb) = akp(ar + 0x20) {
        let plo = akb & 1;
        let snu = (akb >> 2) & 1;
        let rky = (akb >> 3) & 1;
        
        bd.t(&format!("  CR0  = 0x{:08X}\n", akb));
        bd.t(&format!("    SMMU Enabled: {}  EventQ: {}  CmdQ: {}\n",
            plo == 1, snu == 1, rky == 1));
        
        if plo == 0 {
            bd.t("    \x01R!! SMMU is DISABLED — all DMA bypasses protection !!\x01W\n");
        }
    }
    
    bd
}


pub fn pga() -> String {
    let mut an = String::new();
    
    #[cfg(target_arch = "aarch64")]
    {
        an.t("\x01C== TrustProbe: DMA Engine & SMMU Scanner ==\x01W\n\n");
        
        
        an.t("\x01Y--- SMMU/IOMMU Status ---\x01W\n");
        let mut iaw = false;
        
        for &(ar, dds, j) in CTN_ {
            if let Some(ap) = akp(ar) {
                if ap != 0 && ap != 0xFFFFFFFF {
                    an.t(&format!("\x01G[FOUND]\x01W {} @ 0x{:08X}\n", j, ar));
                    an.t(&vmd(ar));
                    iaw = true;
                }
            } else {
                an.t(&format!("\x01R[FAULT]\x01W {} @ 0x{:08X}\n", j, ar));
            }
        }
        
        if !iaw {
            an.t("\x01R!! No SMMU/IOMMU found — DMA attacks possible from any bus master !!\x01W\n\n");
        }
        
        
        an.t("\n\x01Y--- DMA Controller Discovery ---\x01W\n");
        an.t(&format!("{:<16} {:<10} {:<12} {}\n",
            "ADDRESS", "STATUS", "TYPE", "NAME"));
        an.t(&format!("{}\n", "-".afd(65)));
        
        for &(ar, aw, j) in BRW_ {
            let ap = akp(ar);
            let status = match ap {
                Some(p) if p != 0 && p != 0xFFFFFFFF => {
                    
                    let vgt = akp(ar + 0xFE0);
                    let rzl = match vgt {
                        Some(0x30) => "PL330",
                        Some(ad) if ad & 0xFF == 0x41 => "ARM DMA",
                        _ => "Unknown",
                    };
                    format!("\x01G[ACTIVE]\x01W  {:<12} {}", rzl, j)
                },
                Some(_) => format!("\x01Y[IDLE]\x01W    {:<12} {}", "---", j),
                None => format!("\x01R[FAULT]\x01W   {:<12} {}", "---", j),
            };
            an.t(&format!("0x{:010X}   {}\n", ar, status));
        }
        
        
        an.t("\n\x01Y--- DMA Security Analysis ---\x01W\n");
        
        
        an.t("DMA attack surface assessment:\n");
        
        let vzk = [
            ("PCIe Bus Mastering", "Thunderbolt/PCIe devices can DMA", !iaw),
            ("USB DMA (DWC3/XHCI)", "USB devices with DMA capability", true),
            ("GPU DMA", "GPU can read/write system memory", true),
            ("WiFi/BT DMA", "Wireless chipset DMA access", true),
            ("eMMC/UFS ADMA", "Storage controller scatter-gather", true),
        ];
        
        for &(surface, desc, vup) in &vzk {
            if vup {
                let pa = if iaw { "\x01Y[MITIGATED]" } else { "\x01R[EXPOSED]" };
                an.t(&format!("  {}\x01W {} — {}\n", pa, surface, desc));
            }
        }
        
        if !iaw {
            an.t(&format!("\n\x01R!! CRITICAL: Without SMMU, any DMA-capable peripheral\n"));
            an.t(&format!("   can read/write ALL physical memory, including:\n"));
            an.t(&format!("   - Kernel code/data\n"));
            an.t(&format!("   - Encryption keys\n"));
            an.t(&format!("   - Page tables\n"));
            an.t(&format!("   - Secure World memory (if not TZ-protected) !!\x01W\n"));
        }
    }
    
    #[cfg(target_arch = "x86_64")]
    {
        an.t("\x01C== TrustProbe: x86 DMA & IOMMU Scanner ==\x01W\n\n");
        
        
        an.t("\x01Y--- Intel VT-d / AMD-Vi Detection ---\x01W\n");
        
        for &(ar, j) in CCF_ {
            if let Some(ap) = akp(ar) {
                if ap != 0 && ap != 0xFFFFFFFF {
                    an.t(&format!("\x01G[FOUND]\x01W {} @ 0x{:08X} = 0x{:08X}\n",
                        j, ar, ap));
                    
                    
                    if let Some(mh) = akp(ar + 0x08) {
                        let wcj = (mh >> 8) & 0x1F;
                        an.t(&format!("  Capability: 0x{:08X} SAGAW={}\n", mh, wcj));
                    }
                } else {
                    an.t(&format!("\x01Y[EMPTY]\x01W {} @ 0x{:08X}\n", j, ar));
                }
            }
        }
        
        
        an.t("\nPCIe DMA attack surfaces (Thunderbolt/NVMe/GPU):\n");
        an.t("  Use 'hwscan mmio 0xFE000000 0x1000000' to scan PCIe MMIO\n");
    }
    
    #[cfg(target_arch = "riscv64")]
    {
        an.t("\x01C== TrustProbe: RISC-V DMA Scanner ==\x01W\n\n");
        an.t("RISC-V typically uses PMP for DMA isolation.\n");
        an.t("See 'hwscan trustzone' for PMP boundary mapping.\n\n");
        
        
        let xrt = [
            (0x1000_1000u64, "VirtIO block (DMA)"),
            (0x1000_2000u64, "VirtIO net (DMA)"),
            (0x1000_3000u64, "VirtIO console"),
        ];
        
        for &(ar, j) in &xrt {
            if let Some(sj) = akp(ar) {
                if sj == 0x74726976 { 
                    an.t(&format!("\x01G[FOUND]\x01W {} @ 0x{:08X}\n", j, ar));
                }
            }
        }
    }
    
    an
}
