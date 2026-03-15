









use alloc::string::String;
use alloc::format;
use alloc::vec::Vec;
use super::{Aqw, Ju, RiskLevel};


pub fn qlk() -> String {
    let mut an = String::new();
    
    an.t("\x01C");
    an.t("╔══════════════════════════════════════════════════════════╗\n");
    an.t("║          TrustProbe — Full Device Cartography           ║\n");
    an.t("║            Automated Security Assessment                ║\n");
    an.t("╚══════════════════════════════════════════════════════════╝\n");
    an.t("\x01W\n");
    
    let arch = if cfg!(target_arch = "aarch64") {
        "aarch64 (ARM64)"
    } else if cfg!(target_arch = "x86_64") {
        "x86_64"
    } else if cfg!(target_arch = "riscv64") {
        "riscv64"
    } else {
        "unknown"
    };
    
    an.t(&format!("Architecture: {}\n", arch));
    an.t(&format!("TrustProbe version: 1.0.0\n\n"));
    
    
    #[cfg(target_arch = "aarch64")]
    {
        let bqh = crate::android_main::kry();
        if bqh != 0 {
            an.t("\x01Y[0/8] Device Tree Blob (DTB) Analysis...\x01W\n");
            an.t(&format!("{}\n", "=".afd(60)));
            unsafe {
                if let Some(bez) = super::dtb_parser::jis(bqh as *const u8) {
                    an.t(&super::dtb_parser::nvp(&bez));
                    
                    
                    an.t("\n\x01C--- DTB vs Reality Cross-Reference ---\x01W\n");
                    let mut mnz = 0u32;
                    for ba in &bez.ik {
                        if ba.cbi == 0 { continue; }
                        let ptr = ba.cbi as *const u32;
                        let bob = core::ptr::read_volatile(ptr);
                        
                        if ba.status == "disabled" && bob != 0 && bob != 0xFFFFFFFF {
                            an.t(&format!("  \x01R[!] {} (0x{:X}): DTB says disabled, but RESPONDS (0x{:08X})\x01W\n",
                                ba.bjp, ba.cbi, bob));
                            mnz += 1;
                        }
                    }
                    if mnz == 0 {
                        an.t("  All DTB entries consistent with MMIO probing.\n");
                    } else {
                        an.t(&format!("  \x01R{} devices respond despite being marked disabled!\x01W\n", mnz));
                    }
                }
            }
            an.t("\n");
        }
    }
    
    
    an.t("\x01Y[1/7] MMIO Peripheral Discovery...\x01W\n");
    an.t(&format!("{}\n", "=".afd(60)));
    an.t(&super::mmio::pge(0, 0));
    an.t("\n");
    
    
    an.t("\x01Y[2/7] Security Boundary Mapping...\x01W\n");
    an.t(&format!("{}\n", "=".afd(60)));
    an.t(&super::trustzone::oxy());
    an.t("\n");
    
    
    an.t("\x01Y[3/7] DMA Engine & IOMMU Audit...\x01W\n");
    an.t(&format!("{}\n", "=".afd(60)));
    an.t(&super::dma::pga());
    an.t("\n");
    
    
    an.t("\x01Y[4/7] Interrupt Controller Mapping...\x01W\n");
    an.t(&format!("{}\n", "=".afd(60)));
    an.t(&super::irq::pgd());
    an.t("\n");
    
    
    an.t("\x01Y[5/7] GPIO & Debug Interface Discovery...\x01W\n");
    an.t(&format!("{}\n", "=".afd(60)));
    an.t(&super::gpio::oxx());
    an.t("\n");
    
    
    an.t("\x01Y[6/7] Timing Side-Channel Analysis...\x01W\n");
    an.t(&format!("{}\n", "=".afd(60)));
    an.t(&super::timing::per(""));
    an.t("\n");
    
    
    an.t("\x01Y[7/7] Firmware Residue Scan...\x01W\n");
    an.t(&format!("{}\n", "=".afd(60)));
    an.t(&super::firmware::pgb(""));
    an.t("\n");
    
    
    an.t(&nxp());
    
    an
}


fn nxp() -> String {
    let mut an = String::new();
    
    an.t("\x01C");
    an.t("╔══════════════════════════════════════════════════════════╗\n");
    an.t("║          TrustProbe — Executive Summary                 ║\n");
    an.t("╚══════════════════════════════════════════════════════════╝\n");
    an.t("\x01W\n");
    
    an.t("Scan complete. Key findings across all modules:\n\n");
    
    an.t("\x01YMMIO Discovery:\x01W\n");
    an.t("  Probed memory-mapped peripheral regions\n");
    an.t("  Identified controllers by reading hardware ID registers\n\n");
    
    an.t("\x01YSecurity Boundaries:\x01W\n");
    #[cfg(target_arch = "aarch64")]
    an.t("  Mapped TrustZone Normal/Secure World transitions\n");
    #[cfg(target_arch = "x86_64")]
    an.t("  Probed SMM and Ring 0/Ring -1 boundaries\n");
    #[cfg(target_arch = "riscv64")]
    an.t("  Mapped PMP (Physical Memory Protection) boundaries\n");
    an.t("  Any secure memory accessible from kernel = CRITICAL\n\n");
    
    an.t("\x01YDMA & IOMMU:\x01W\n");
    an.t("  Enumerated DMA-capable controllers\n");
    an.t("  Checked SMMU/IOMMU isolation status\n");
    an.t("  Missing IOMMU = all DMA devices can read kernel memory\n\n");
    
    an.t("\x01YInterrupts:\x01W\n");
    an.t("  Mapped interrupt controller topology\n");
    an.t("  Identified active/pending IRQ routing\n\n");
    
    an.t("\x01YGPIO & Debug:\x01W\n");
    an.t("  Scanned GPIO pin muxing for hidden interfaces\n");
    an.t("  Active UART/JTAG = direct debug access possible\n\n");
    
    an.t("\x01YTiming Analysis:\x01W\n");
    an.t("  Measured access latencies across memory regions\n");
    an.t("  Anomalies may indicate hidden secure boundaries\n\n");
    
    an.t("\x01YFirmware Residue:\x01W\n");
    an.t("  Searched for bootloader/firmware artifacts in memory\n");
    an.t("  Keys/certificates/debug tokens left by boot chain\n\n");
    
    an.t("\x01C--- Risk Assessment ---\x01W\n");
    an.t("Use individual scan commands for detailed results:\n");
    an.t("  hwscan mmio      — MMIO peripheral map\n");
    an.t("  hwscan trustzone — Security boundary map\n");
    an.t("  hwscan dma       — DMA/IOMMU audit\n");
    an.t("  hwscan irq       — Interrupt topology\n");
    an.t("  hwscan gpio      — Debug interface discovery\n");
    an.t("  hwscan timing    — Side-channel analysis\n");
    an.t("  hwscan firmware  — Firmware residue scan\n");
    an.t("  hwscan report    — This summary\n\n");
    
    an.t("\x01C[TrustOS TrustProbe — Bare-Metal Hardware Security Research Platform]\x01W\n");
    
    an
}


pub fn tck() -> String {
    nxp()
}
