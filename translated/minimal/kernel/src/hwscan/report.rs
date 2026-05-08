









use alloc::string::String;
use alloc::format;
use alloc::vec::Vec;
use super::{Rs, Dy, RiskLevel};


pub fn jyk() -> String {
    let mut output = String::new();
    
    output.push_str("\x01C");
    output.push_str("╔══════════════════════════════════════════════════════════╗\n");
    output.push_str("║          TrustProbe — Full Device Cartography           ║\n");
    output.push_str("║            Automated Security Assessment                ║\n");
    output.push_str("╚══════════════════════════════════════════════════════════╝\n");
    output.push_str("\x01W\n");
    
    let arch = if cfg!(target_arch = "aarch64") {
        "aarch64 (ARM64)"
    } else if cfg!(target_arch = "x86_64") {
        "x86_64"
    } else if cfg!(target_arch = "riscv64") {
        "riscv64"
    } else {
        "unknown"
    };
    
    output.push_str(&format!("Architecture: {}\n", arch));
    output.push_str(&format!("TrustProbe version: 1.0.0\n\n"));
    
    
    #[cfg(target_arch = "aarch64")]
    {
        let dtb_addr = crate::android_main::ftl();
        if dtb_addr != 0 {
            output.push_str("\x01Y[0/8] Device Tree Blob (DTB) Analysis...\x01W\n");
            output.push_str(&format!("{}\n", "=".repeat(60)));
            unsafe {
                if let Some(parsed) = super::dtb_parser::ewg(dtb_addr as *const u8) {
                    output.push_str(&super::dtb_parser::hzo(&parsed));
                    
                    
                    output.push_str("\n\x01C--- DTB vs Reality Cross-Reference ---\x01W\n");
                    let mut haq = 0u32;
                    for s in &parsed.devices {
                        if s.reg_base == 0 { continue; }
                        let ptr = s.reg_base as *const u32;
                        let readable = core::ptr::read_volatile(ptr);
                        
                        if s.status == "disabled" && readable != 0 && readable != 0xFFFFFFFF {
                            output.push_str(&format!("  \x01R[!] {} (0x{:X}): DTB says disabled, but RESPONDS (0x{:08X})\x01W\n",
                                s.compatible, s.reg_base, readable));
                            haq += 1;
                        }
                    }
                    if haq == 0 {
                        output.push_str("  All DTB entries consistent with MMIO probing.\n");
                    } else {
                        output.push_str(&format!("  \x01R{} devices respond despite being marked disabled!\x01W\n", haq));
                    }
                }
            }
            output.push_str("\n");
        }
    }
    
    
    output.push_str("\x01Y[1/7] MMIO Peripheral Discovery...\x01W\n");
    output.push_str(&format!("{}\n", "=".repeat(60)));
    output.push_str(&super::mmio::jdg(0, 0));
    output.push_str("\n");
    
    
    output.push_str("\x01Y[2/7] Security Boundary Mapping...\x01W\n");
    output.push_str(&format!("{}\n", "=".repeat(60)));
    output.push_str(&super::trustzone::iws());
    output.push_str("\n");
    
    
    output.push_str("\x01Y[3/7] DMA Engine & IOMMU Audit...\x01W\n");
    output.push_str(&format!("{}\n", "=".repeat(60)));
    output.push_str(&super::dma::jda());
    output.push_str("\n");
    
    
    output.push_str("\x01Y[4/7] Interrupt Controller Mapping...\x01W\n");
    output.push_str(&format!("{}\n", "=".repeat(60)));
    output.push_str(&super::irq::jde());
    output.push_str("\n");
    
    
    output.push_str("\x01Y[5/7] GPIO & Debug Interface Discovery...\x01W\n");
    output.push_str(&format!("{}\n", "=".repeat(60)));
    output.push_str(&super::gpio::iwr());
    output.push_str("\n");
    
    
    output.push_str("\x01Y[6/7] Timing Side-Channel Analysis...\x01W\n");
    output.push_str(&format!("{}\n", "=".repeat(60)));
    output.push_str(&super::timing::jbw(""));
    output.push_str("\n");
    
    
    output.push_str("\x01Y[7/7] Firmware Residue Scan...\x01W\n");
    output.push_str(&format!("{}\n", "=".repeat(60)));
    output.push_str(&super::firmware::jdb(""));
    output.push_str("\n");
    
    
    output.push_str(&ibe());
    
    output
}


fn ibe() -> String {
    let mut output = String::new();
    
    output.push_str("\x01C");
    output.push_str("╔══════════════════════════════════════════════════════════╗\n");
    output.push_str("║          TrustProbe — Executive Summary                 ║\n");
    output.push_str("╚══════════════════════════════════════════════════════════╝\n");
    output.push_str("\x01W\n");
    
    output.push_str("Scan complete. Key findings across all modules:\n\n");
    
    output.push_str("\x01YMMIO Discovery:\x01W\n");
    output.push_str("  Probed memory-mapped peripheral regions\n");
    output.push_str("  Identified controllers by reading hardware ID registers\n\n");
    
    output.push_str("\x01YSecurity Boundaries:\x01W\n");
    #[cfg(target_arch = "aarch64")]
    output.push_str("  Mapped TrustZone Normal/Secure World transitions\n");
    #[cfg(target_arch = "x86_64")]
    output.push_str("  Probed SMM and Ring 0/Ring -1 boundaries\n");
    #[cfg(target_arch = "riscv64")]
    output.push_str("  Mapped PMP (Physical Memory Protection) boundaries\n");
    output.push_str("  Any secure memory accessible from kernel = CRITICAL\n\n");
    
    output.push_str("\x01YDMA & IOMMU:\x01W\n");
    output.push_str("  Enumerated DMA-capable controllers\n");
    output.push_str("  Checked SMMU/IOMMU isolation status\n");
    output.push_str("  Missing IOMMU = all DMA devices can read kernel memory\n\n");
    
    output.push_str("\x01YInterrupts:\x01W\n");
    output.push_str("  Mapped interrupt controller topology\n");
    output.push_str("  Identified active/pending IRQ routing\n\n");
    
    output.push_str("\x01YGPIO & Debug:\x01W\n");
    output.push_str("  Scanned GPIO pin muxing for hidden interfaces\n");
    output.push_str("  Active UART/JTAG = direct debug access possible\n\n");
    
    output.push_str("\x01YTiming Analysis:\x01W\n");
    output.push_str("  Measured access latencies across memory regions\n");
    output.push_str("  Anomalies may indicate hidden secure boundaries\n\n");
    
    output.push_str("\x01YFirmware Residue:\x01W\n");
    output.push_str("  Searched for bootloader/firmware artifacts in memory\n");
    output.push_str("  Keys/certificates/debug tokens left by boot chain\n\n");
    
    output.push_str("\x01C--- Risk Assessment ---\x01W\n");
    output.push_str("Use individual scan commands for detailed results:\n");
    output.push_str("  hwscan mmio      — MMIO peripheral map\n");
    output.push_str("  hwscan trustzone — Security boundary map\n");
    output.push_str("  hwscan dma       — DMA/IOMMU audit\n");
    output.push_str("  hwscan irq       — Interrupt topology\n");
    output.push_str("  hwscan gpio      — Debug interface discovery\n");
    output.push_str("  hwscan timing    — Side-channel analysis\n");
    output.push_str("  hwscan firmware  — Firmware residue scan\n");
    output.push_str("  hwscan report    — This summary\n\n");
    
    output.push_str("\x01C[TrustOS TrustProbe — Bare-Metal Hardware Security Research Platform]\x01W\n");
    
    output
}


pub fn fyi() -> String {
    ibe()
}
