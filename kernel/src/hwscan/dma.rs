//! DMA Engine Enumerator & IOMMU/SMMU Mapper
//!
//! DMA (Direct Memory Access) is one of the biggest attack surfaces
//! in modern SoCs. A DMA-capable peripheral can read/write physical
//! memory directly, bypassing the CPU's MMU. If the IOMMU/SMMU is
//! misconfigured or absent, this allows full memory access from
//! any bus-mastering device.
//!
//! This module:
//!   1. Discovers DMA controllers (PL330, SDMA, ADMA, etc.)
//!   2. Checks SMMU/IOMMU configuration
//!   3. Tests if DMA regions are properly isolated
//!   4. Identifies potential DMA attack vectors (Thunderbolt, PCIe, USB DMA)

use alloc::string::String;
use alloc::format;
use alloc::vec::Vec;
use super::{ProbeResult, AccessLevel, RiskLevel};

/// Known DMA controller bases per SoC
#[cfg(target_arch = "aarch64")]
const DMA_CONTROLLERS_ARM: &[(u64, u64, &str)] = &[
    (0x0902_0000, 0x1000, "QEMU virt pl330 (if present)"),
    (0x7E00_7000, 0x1000, "BCM2711 DMA0-6 (Lite)"),
    (0x7E00_7B00, 0x0300, "BCM2711 DMA7-10 (Normal)"),
    (0x7EE0_B000, 0x1000, "BCM2711 DMA11-14 (4K capable)"),
    (0x7E00_E000, 0x1000, "BCM2711 DMA15 (bulk/fast)"),
    (0x0088_4000, 0x1000, "Snapdragon BAM DMA"),
    (0x0C80_0000, 0x4000, "Snapdragon GPI DMA (QUP)"),
];

/// SMMU/IOMMU bases
#[cfg(target_arch = "aarch64")]
const SMMU_BASES_ARM: &[(u64, u64, &str)] = &[
    (0x0900_0000, 0x2_0000, "QEMU virt SMMUv3"),
    (0x1500_0000, 0x8_0000, "Snapdragon SMMU (apps)"),
    (0x0510_0000, 0x1_0000, "Snapdragon SMMU-500 (GPU)"),
    (0xFD50_0000, 0x1_0000, "BCM2711 IOMMU (if present)"),
];

/// x86 IOMMU (VT-d / AMD-Vi) detection locations
#[cfg(target_arch = "x86_64")]
const IOMMU_DETECT_X86: &[(u64, &str)] = &[
    (0xFED9_0000, "Intel VT-d DMAR unit (typical)"),
    (0xFED4_0000, "AMD-Vi IVHD base (typical)"),
];

/// DMA channel status register offsets (PL330)
const PL330_DSR: u64 = 0x000; // DMA Status Register
const PL330_DPC: u64 = 0x004; // DMA Program Counter
const PL330_FSRD: u64 = 0x030; // Fault Status DMA Manager
const PL330_FTRD: u64 = 0x038; // Fault Type DMA Manager
const PL330_CR0: u64 = 0xE00; // Configuration Register 0
const PL330_PERIPH_ID: u64 = 0xFE0; // Peripheral ID

/// BCM2711 DMA register offsets
const BCM_DMA_CS: u64 = 0x00; // Control & Status
const BCM_DMA_TI: u64 = 0x08; // Transfer Information
const BCM_DMA_DEBUG: u64 = 0x20; // Debug

fn safe_read(addr: u64) -> Option<u32> {
    if addr == 0 { return None; }
    unsafe {
        let ptr = addr as *const u32;
        Some(core::ptr::read_volatile(ptr))
    }
}

/// Detect SMMUv3 and read its configuration
#[cfg(target_arch = "aarch64")]
fn probe_smmuv3(base: u64) -> String {
    let mut out = String::new();
    
    // SMMUv3 IDR0 at offset 0x0
    if let Some(idr0) = safe_read(base) {
        let s2p = (idr0 >> 0) & 1;   // Stage 2 present
        let s1p = (idr0 >> 1) & 1;   // Stage 1 present
        let ttf = (idr0 >> 2) & 3;   // Translation Table Format
        let httu = (idr0 >> 6) & 3;  // Hardware TT Update
        let cohacc = (idr0 >> 4) & 1; // Coherent access
        
        out.push_str(&format!("  IDR0 = 0x{:08X}\n", idr0));
        out.push_str(&format!("    Stage 1: {}  Stage 2: {}\n",
            if s1p == 1 { "YES" } else { "NO" },
            if s2p == 1 { "YES" } else { "NO" }));
        out.push_str(&format!("    Translation format: {}  HTTU: {}  Coherent: {}\n",
            ttf, httu, cohacc != 0));
            
        if s2p == 0 && s1p == 0 {
            out.push_str("    \x01R!! SMMU has NO translation stages — DMA is UNPROTECTED !!\x01W\n");
        }
    }
    
    // SMMUv3 IDR1 at offset 0x4
    if let Some(idr1) = safe_read(base + 4) {
        let queues = (idr1 >> 8) & 0x1F;
        let ssidsize = (idr1 >> 6) & 0x1F;
        out.push_str(&format!("  IDR1 = 0x{:08X} (queues: {}, SSID bits: {})\n",
            idr1, queues, ssidsize));
    }
    
    // CR0 (Control Register)
    if let Some(cr0) = safe_read(base + 0x20) {
        let smmuen = cr0 & 1;
        let eventqen = (cr0 >> 2) & 1;
        let cmdqen = (cr0 >> 3) & 1;
        
        out.push_str(&format!("  CR0  = 0x{:08X}\n", cr0));
        out.push_str(&format!("    SMMU Enabled: {}  EventQ: {}  CmdQ: {}\n",
            smmuen == 1, eventqen == 1, cmdqen == 1));
        
        if smmuen == 0 {
            out.push_str("    \x01R!! SMMU is DISABLED — all DMA bypasses protection !!\x01W\n");
        }
    }
    
    out
}

/// Scan for DMA engines and check IOMMU protection
pub fn scan_dma_engines() -> String {
    let mut output = String::new();
    
    #[cfg(target_arch = "aarch64")]
    {
        output.push_str("\x01C== TrustProbe: DMA Engine & SMMU Scanner ==\x01W\n\n");
        
        // First: check SMMU/IOMMU status
        output.push_str("\x01Y--- SMMU/IOMMU Status ---\x01W\n");
        let mut smmu_found = false;
        
        for &(base, _size, name) in SMMU_BASES_ARM {
            if let Some(val) = safe_read(base) {
                if val != 0 && val != 0xFFFFFFFF {
                    output.push_str(&format!("\x01G[FOUND]\x01W {} @ 0x{:08X}\n", name, base));
                    output.push_str(&probe_smmuv3(base));
                    smmu_found = true;
                }
            } else {
                output.push_str(&format!("\x01R[FAULT]\x01W {} @ 0x{:08X}\n", name, base));
            }
        }
        
        if !smmu_found {
            output.push_str("\x01R!! No SMMU/IOMMU found — DMA attacks possible from any bus master !!\x01W\n\n");
        }
        
        // Scan DMA controllers
        output.push_str("\n\x01Y--- DMA Controller Discovery ---\x01W\n");
        output.push_str(&format!("{:<16} {:<10} {:<12} {}\n",
            "ADDRESS", "STATUS", "TYPE", "NAME"));
        output.push_str(&format!("{}\n", "-".repeat(65)));
        
        for &(base, size, name) in DMA_CONTROLLERS_ARM {
            let val = safe_read(base);
            let status = match val {
                Some(v) if v != 0 && v != 0xFFFFFFFF => {
                    // Try to identify the DMA type
                    let periph_id = safe_read(base + 0xFE0);
                    let dma_type = match periph_id {
                        Some(0x30) => "PL330",
                        Some(id) if id & 0xFF == 0x41 => "ARM DMA",
                        _ => "Unknown",
                    };
                    format!("\x01G[ACTIVE]\x01W  {:<12} {}", dma_type, name)
                },
                Some(_) => format!("\x01Y[IDLE]\x01W    {:<12} {}", "---", name),
                None => format!("\x01R[FAULT]\x01W   {:<12} {}", "---", name),
            };
            output.push_str(&format!("0x{:010X}   {}\n", base, status));
        }
        
        // Security analysis
        output.push_str("\n\x01Y--- DMA Security Analysis ---\x01W\n");
        
        // Check: Can we configure DMA to read kernel memory?
        output.push_str("DMA attack surface assessment:\n");
        
        let risks = [
            ("PCIe Bus Mastering", "Thunderbolt/PCIe devices can DMA", !smmu_found),
            ("USB DMA (DWC3/XHCI)", "USB devices with DMA capability", true),
            ("GPU DMA", "GPU can read/write system memory", true),
            ("WiFi/BT DMA", "Wireless chipset DMA access", true),
            ("eMMC/UFS ADMA", "Storage controller scatter-gather", true),
        ];
        
        for &(surface, desc, relevant) in &risks {
            if relevant {
                let icon = if smmu_found { "\x01Y[MITIGATED]" } else { "\x01R[EXPOSED]" };
                output.push_str(&format!("  {}\x01W {} — {}\n", icon, surface, desc));
            }
        }
        
        if !smmu_found {
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
        
        // Check VT-d / AMD-Vi
        output.push_str("\x01Y--- Intel VT-d / AMD-Vi Detection ---\x01W\n");
        
        for &(base, name) in IOMMU_DETECT_X86 {
            if let Some(val) = safe_read(base) {
                if val != 0 && val != 0xFFFFFFFF {
                    output.push_str(&format!("\x01G[FOUND]\x01W {} @ 0x{:08X} = 0x{:08X}\n",
                        name, base, val));
                    
                    // Read capability register
                    if let Some(cap) = safe_read(base + 0x08) {
                        let sagaw = (cap >> 8) & 0x1F;
                        output.push_str(&format!("  Capability: 0x{:08X} SAGAW={}\n", cap, sagaw));
                    }
                } else {
                    output.push_str(&format!("\x01Y[EMPTY]\x01W {} @ 0x{:08X}\n", name, base));
                }
            }
        }
        
        // PCIe DMA surfaces
        output.push_str("\nPCIe DMA attack surfaces (Thunderbolt/NVMe/GPU):\n");
        output.push_str("  Use 'hwscan mmio 0xFE000000 0x1000000' to scan PCIe MMIO\n");
    }
    
    #[cfg(target_arch = "riscv64")]
    {
        output.push_str("\x01C== TrustProbe: RISC-V DMA Scanner ==\x01W\n\n");
        output.push_str("RISC-V typically uses PMP for DMA isolation.\n");
        output.push_str("See 'hwscan trustzone' for PMP boundary mapping.\n\n");
        
        // VirtIO devices often have DMA
        let virtio_bases = [
            (0x1000_1000u64, "VirtIO block (DMA)"),
            (0x1000_2000u64, "VirtIO net (DMA)"),
            (0x1000_3000u64, "VirtIO console"),
        ];
        
        for &(base, name) in &virtio_bases {
            if let Some(magic) = safe_read(base) {
                if magic == 0x74726976 { // "virt"
                    output.push_str(&format!("\x01G[FOUND]\x01W {} @ 0x{:08X}\n", name, base));
                }
            }
        }
    }
    
    output
}
