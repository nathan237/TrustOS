













use alloc::string::String;
use alloc::format;
use alloc::vec::Vec;
use super::{Dy, AccessLevel, RiskLevel};


#[cfg(target_arch = "aarch64")]
const BUS_: &[(u64, u64, &str)] = &[
    (0x0902_0000, 0x1000, "QEMU virt pl330 (if present)"),
    (0x7E00_7000, 0x1000, "BCM2711 DMA0-6 (Lite)"),
    (0x7E00_7B00, 0x0300, "BCM2711 DMA7-10 (Normal)"),
    (0x7EE0_B000, 0x1000, "BCM2711 DMA11-14 (4K capable)"),
    (0x7E00_E000, 0x1000, "BCM2711 DMA15 (bulk/fast)"),
    (0x0088_4000, 0x1000, "Snapdragon BAM DMA"),
    (0x0C80_0000, 0x4000, "Snapdragon GPI DMA (QUP)"),
];


#[cfg(target_arch = "aarch64")]
const CXE_: &[(u64, u64, &str)] = &[
    (0x0900_0000, 0x2_0000, "QEMU virt SMMUv3"),
    (0x1500_0000, 0x8_0000, "Snapdragon SMMU (apps)"),
    (0x0510_0000, 0x1_0000, "Snapdragon SMMU-500 (GPU)"),
    (0xFD50_0000, 0x1_0000, "BCM2711 IOMMU (if present)"),
];


#[cfg(target_arch = "x86_64")]
const CFQ_: &[(u64, &str)] = &[
    (0xFED9_0000, "Intel VT-d DMAR unit (typical)"),
    (0xFED4_0000, "AMD-Vi IVHD base (typical)"),
];


const EDT_: u64 = 0x000; 
const EDS_: u64 = 0x004; 
const EDU_: u64 = 0x030; 
const EDV_: u64 = 0x038; 
const EDR_: u64 = 0xE00; 
const EDW_: u64 = 0xFE0; 


const DHD_: u64 = 0x00; 
const DHF_: u64 = 0x08; 
const DHE_: u64 = 0x20; 

fn sm(addr: u64) -> Option<u32> {
    if addr == 0 { return None; }
    unsafe {
        let ptr = addr as *const u32;
        Some(core::ptr::read_volatile(ptr))
    }
}


#[cfg(target_arch = "aarch64")]
fn nxx(base: u64) -> String {
    let mut out = String::new();
    
    
    if let Some(idr0) = sm(base) {
        let jcb = (idr0 >> 0) & 1;   
        let jca = (idr0 >> 1) & 1;   
        let poh = (idr0 >> 2) & 3;   
        let mmp = (idr0 >> 6) & 3;  
        let kva = (idr0 >> 4) & 1; 
        
        out.push_str(&format!("  IDR0 = 0x{:08X}\n", idr0));
        out.push_str(&format!("    Stage 1: {}  Stage 2: {}\n",
            if jca == 1 { "YES" } else { "NO" },
            if jcb == 1 { "YES" } else { "NO" }));
        out.push_str(&format!("    Translation format: {}  HTTU: {}  Coherent: {}\n",
            poh, mmp, kva != 0));
            
        if jcb == 0 && jca == 0 {
            out.push_str("    \x01R!! SMMU has NO translation stages — DMA is UNPROTECTED !!\x01W\n");
        }
    }
    
    
    if let Some(idr1) = sm(base + 4) {
        let zg = (idr1 >> 8) & 0x1F;
        let ovq = (idr1 >> 6) & 0x1F;
        out.push_str(&format!("  IDR1 = 0x{:08X} (queues: {}, SSID bits: {})\n",
            idr1, zg, ovq));
    }
    
    
    if let Some(cr0) = sm(base + 0x20) {
        let jgt = cr0 & 1;
        let lrq = (cr0 >> 2) & 1;
        let kuk = (cr0 >> 3) & 1;
        
        out.push_str(&format!("  CR0  = 0x{:08X}\n", cr0));
        out.push_str(&format!("    SMMU Enabled: {}  EventQ: {}  CmdQ: {}\n",
            jgt == 1, lrq == 1, kuk == 1));
        
        if jgt == 0 {
            out.push_str("    \x01R!! SMMU is DISABLED — all DMA bypasses protection !!\x01W\n");
        }
    }
    
    out
}


pub fn jda() -> String {
    let mut output = String::new();
    
    #[cfg(target_arch = "aarch64")]
    {
        output.push_str("\x01C== TrustProbe: DMA Engine & SMMU Scanner ==\x01W\n\n");
        
        
        output.push_str("\x01Y--- SMMU/IOMMU Status ---\x01W\n");
        let mut dzr = false;
        
        for &(base, bek, name) in CXE_ {
            if let Some(val) = sm(base) {
                if val != 0 && val != 0xFFFFFFFF {
                    output.push_str(&format!("\x01G[FOUND]\x01W {} @ 0x{:08X}\n", name, base));
                    output.push_str(&nxx(base));
                    dzr = true;
                }
            } else {
                output.push_str(&format!("\x01R[FAULT]\x01W {} @ 0x{:08X}\n", name, base));
            }
        }
        
        if !dzr {
            output.push_str("\x01R!! No SMMU/IOMMU found — DMA attacks possible from any bus master !!\x01W\n\n");
        }
        
        
        output.push_str("\n\x01Y--- DMA Controller Discovery ---\x01W\n");
        output.push_str(&format!("{:<16} {:<10} {:<12} {}\n",
            "ADDRESS", "STATUS", "TYPE", "NAME"));
        output.push_str(&format!("{}\n", "-".repeat(65)));
        
        for &(base, size, name) in BUS_ {
            let val = sm(base);
            let status = match val {
                Some(v) if v != 0 && v != 0xFFFFFFFF => {
                    
                    let ntq = sm(base + 0xFE0);
                    let lgh = match ntq {
                        Some(0x30) => "PL330",
                        Some(id) if id & 0xFF == 0x41 => "ARM DMA",
                        _ => "Unknown",
                    };
                    format!("\x01G[ACTIVE]\x01W  {:<12} {}", lgh, name)
                },
                Some(_) => format!("\x01Y[IDLE]\x01W    {:<12} {}", "---", name),
                None => format!("\x01R[FAULT]\x01W   {:<12} {}", "---", name),
            };
            output.push_str(&format!("0x{:010X}   {}\n", base, status));
        }
        
        
        output.push_str("\n\x01Y--- DMA Security Analysis ---\x01W\n");
        
        
        output.push_str("DMA attack surface assessment:\n");
        
        let ohm = [
            ("PCIe Bus Mastering", "Thunderbolt/PCIe devices can DMA", !dzr),
            ("USB DMA (DWC3/XHCI)", "USB devices with DMA capability", true),
            ("GPU DMA", "GPU can read/write system memory", true),
            ("WiFi/BT DMA", "Wireless chipset DMA access", true),
            ("eMMC/UFS ADMA", "Storage controller scatter-gather", true),
        ];
        
        for &(surface, desc, relevant) in &ohm {
            if relevant {
                let icon = if dzr { "\x01Y[MITIGATED]" } else { "\x01R[EXPOSED]" };
                output.push_str(&format!("  {}\x01W {} — {}\n", icon, surface, desc));
            }
        }
        
        if !dzr {
            output.push_str(&format!("\n\x01R!! CRITICAL: Without SMMU, any DMA-capable peripheral\n"));
            output.push_str(&format!("   can read/write ALL physical memory, including:\n"));
            output.push_str(&format!("   - Kernel code/data\n"));
            output.push_str(&format!("   - Encryption keys\n"));
            output.push_str(&format!("   - Page tables\n"));
            output.push_str(&format!("   - Secure World memory (if not TZ-protected) !!\x01W\n"));
        }
    }
    
    #[cfg(target_arch = "x86_64")]
    {
        output.push_str("\x01C== TrustProbe: x86 DMA & IOMMU Scanner ==\x01W\n\n");
        
        
        output.push_str("\x01Y--- Intel VT-d / AMD-Vi Detection ---\x01W\n");
        
        for &(base, name) in CFQ_ {
            if let Some(val) = sm(base) {
                if val != 0 && val != 0xFFFFFFFF {
                    output.push_str(&format!("\x01G[FOUND]\x01W {} @ 0x{:08X} = 0x{:08X}\n",
                        name, base, val));
                    
                    
                    if let Some(cap) = sm(base + 0x08) {
                        let ojx = (cap >> 8) & 0x1F;
                        output.push_str(&format!("  Capability: 0x{:08X} SAGAW={}\n", cap, ojx));
                    }
                } else {
                    output.push_str(&format!("\x01Y[EMPTY]\x01W {} @ 0x{:08X}\n", name, base));
                }
            }
        }
        
        
        output.push_str("\nPCIe DMA attack surfaces (Thunderbolt/NVMe/GPU):\n");
        output.push_str("  Use 'hwscan mmio 0xFE000000 0x1000000' to scan PCIe MMIO\n");
    }
    
    #[cfg(target_arch = "riscv64")]
    {
        output.push_str("\x01C== TrustProbe: RISC-V DMA Scanner ==\x01W\n\n");
        output.push_str("RISC-V typically uses PMP for DMA isolation.\n");
        output.push_str("See 'hwscan trustzone' for PMP boundary mapping.\n\n");
        
        
        let pse = [
            (0x1000_1000u64, "VirtIO block (DMA)"),
            (0x1000_2000u64, "VirtIO net (DMA)"),
            (0x1000_3000u64, "VirtIO console"),
        ];
        
        for &(base, name) in &pse {
            if let Some(magic) = sm(base) {
                if magic == 0x74726976 { 
                    output.push_str(&format!("\x01G[FOUND]\x01W {} @ 0x{:08X}\n", name, base));
                }
            }
        }
    }
    
    output
}
