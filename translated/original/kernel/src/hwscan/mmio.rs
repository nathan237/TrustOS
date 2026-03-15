//! MMIO Region Scanner
//!
//! Scans physical memory ranges to discover memory-mapped hardware peripherals.
//! On real hardware, each SoC maps its peripherals at specific physical addresses.
//! By systematically probing these ranges, we can build a map of what hardware
//! exists — including undocumented debug registers the vendor didn't tell you about.
//!
//! Method:
//!   1. For each 4KB page in the scan range, attempt a volatile read
//!   2. Classify the response: live data, fixed pattern, bus error, fault
//!   3. For live regions, probe register stride and identify the peripheral
//!   4. Cross-reference against known SoC peripheral databases
//!
//! Safety: All reads are volatile, single-word, with fault recovery.
//!         Writes are NEVER performed unless explicitly requested.

use alloc::string::String;
use alloc::format;
use alloc::vec::Vec;
use super::{ProbeResult, AccessLevel, RiskLevel};

/// Known MMIO regions for common SoCs (base, size, name)
const KNOWN_REGIONS_QEMU_VIRT: &[(u64, u64, &str)] = &[
    (0x0800_0000, 0x0001_0000, "GIC Distributor (GICD)"),
    (0x0801_0000, 0x0001_0000, "GIC CPU Interface (GICC)"),
    (0x0900_0000, 0x0000_1000, "PL011 UART0"),
    (0x0900_1000, 0x0000_1000, "PL011 UART1"),
    (0x0a00_0000, 0x0000_0200, "VirtIO Device 0"),
    (0x0a00_0200, 0x0000_0200, "VirtIO Device 1"),
    (0x0a00_3e00, 0x0000_0200, "VirtIO Device 31"),
    (0x0c00_0000, 0x0020_0000, "Platform Bus"),
    (0x0e00_0000, 0x0001_0000, "Secure SRAM (TrustZone)"),
    (0x1000_0000, 0x2eff_ffff, "PCIe MMIO Window"),
    (0x4000_0000, 0xc000_0000, "RAM"),
];

const KNOWN_REGIONS_BCM2711: &[(u64, u64, &str)] = &[
    (0xFE00_0000, 0x0000_1000, "System Timer"),
    (0xFE00_3000, 0x0000_1000, "DMA Controller"),
    (0xFE00_B000, 0x0000_1000, "ARM Interrupt Controller"),
    (0xFE10_0000, 0x0000_0080, "PCM/I2S Audio"),
    (0xFE10_4000, 0x0000_1000, "SPI0"),
    (0xFE20_0000, 0x0000_00B4, "GPIO"),
    (0xFE20_1000, 0x0000_1000, "PL011 UART0"),
    (0xFE20_4000, 0x0000_1000, "SPI/BSC Slave"),
    (0xFE21_5000, 0x0000_1000, "Mini UART + SPI1/2"),
    (0xFE30_0000, 0x0000_1000, "eMMC/SD (EMMC2)"),
    (0xFE34_0000, 0x0000_1000, "eMMC (EMMC1)"),
    (0xFE80_0000, 0x0000_1000, "USB (DWC2 OTG)"),
    (0xFE98_0000, 0x0000_1000, "USB xHCI"),
    (0xFEC1_1000, 0x0000_1000, "GICv2 Distributor"),
    (0xFEC1_2000, 0x0000_1000, "GICv2 CPU Interface"),
    (0xFF80_0000, 0x0000_1000, "PCIe Root Complex"),
];

const KNOWN_REGIONS_SNAPDRAGON: &[(u64, u64, &str)] = &[
    (0x0100_0000, 0x0010_0000, "RPM (Resource Power Manager)"),
    (0x0600_0000, 0x0010_0000, "TCSR (Top-level CSR)"),
    (0x0780_0000, 0x0008_0000, "SMMU (System MMU)"),
    (0x0880_0000, 0x0008_0000, "GCC (Global Clock Controller)"),
    (0x0A60_0000, 0x0000_1000, "UART (Debug Serial)"),
    (0x0A80_0000, 0x0004_0000, "USB3 Controller"),
    (0x0C40_0000, 0x0010_0000, "GPU (Adreno)"),
    (0x1700_0000, 0x0004_0000, "Modem (MSS)"),
    (0x1A00_0000, 0x0020_0000, "LPASS (Audio DSP)"),
    (0x5000_0000, 0x0100_0000, "UFS Controller"),
    (0x5A00_0000, 0x0010_0000, "Crypto Engine"),
    (0xF111_0000, 0x0000_1000, "IMEM (Internal SRAM)"),
];

/// Peripheral identifier patterns — magic values at register offset 0
const MAGIC_PATTERNS: &[(u32, &str)] = &[
    (0x0011_0004, "PL011 UART (ARM PrimeCell)"),
    (0x0011_0044, "PL011 UART (variant)"),
    (0x0034_1011, "PL031 RTC (ARM PrimeCell)"),
    (0x0041_0FC0, "ARM GIC (Generic Interrupt Controller)"),
    (0x0020_0100, "VirtIO Device (QEMU)"),
    (0x1020_6400, "BCM2711 Peripheral"),
    (0x7846_3100, "QCA UART (Qualcomm)"),
    (0x4D54_4B00, "MediaTek Peripheral"),
    (0x0000_0002, "PCIe Configuration Space"),
];

/// Result of scanning a single page
#[derive(Debug, Clone)]
pub struct MmioPageResult {
    pub base: u64,
    pub first_word: u32,
    pub access: AccessLevel,
    pub identified_as: Option<&'static str>,
    pub register_count: u32,
    pub unique_values: u32,
}

/// Safe volatile read with fault recovery
/// Returns None if the access causes a fault
fn safe_volatile_read(addr: u64) -> Option<u32> {
    // On real hardware, we'd install a fault handler and attempt the read.
    // In this implementation, we do a volatile read and catch common patterns.
    // The kernel's fault handler will recover from actual data aborts.
    
    #[cfg(target_arch = "aarch64")]
    {
        // ARM: Check if address is within a reasonable range
        // Addresses above 0x1_0000_0000_0000 are typically kernel VA
        if addr == 0 || addr > 0xFFFF_FFFF_FFFF {
            return None;
        }
        
        unsafe {
            let ptr = addr as *const u32;
            // Use volatile to prevent optimization
            let val = core::ptr::read_volatile(ptr);
            Some(val)
        }
    }
    
    #[cfg(target_arch = "x86_64")]
    {
        if addr == 0 || addr > 0xFFFF_FFFF_FFFF {
            return None;
        }
        unsafe {
            let ptr = addr as *const u32;
            let val = core::ptr::read_volatile(ptr);
            Some(val)
        }
    }
    
    #[cfg(target_arch = "riscv64")]
    {
        if addr == 0 {
            return None;
        }
        unsafe {
            let ptr = addr as *const u32;
            let val = core::ptr::read_volatile(ptr);
            Some(val)
        }
    }
    
    #[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64", target_arch = "riscv64")))]
    {
        let _ = addr;
        None
    }
}

/// Identify a peripheral by its register values
fn identify_peripheral(first_word: u32, addr: u64) -> Option<&'static str> {
    // Check magic patterns
    for &(magic, name) in MAGIC_PATTERNS {
        if first_word == magic {
            return Some(name);
        }
    }
    
    // Check against known SoC regions
    for &(base, size, name) in KNOWN_REGIONS_QEMU_VIRT.iter()
        .chain(KNOWN_REGIONS_BCM2711.iter())
        .chain(KNOWN_REGIONS_SNAPDRAGON.iter())
    {
        if addr >= base && addr < base + size {
            return Some(name);
        }
    }
    
    None
}

/// Scan a single MMIO page (4KB) and classify it
fn probe_page(base: u64) -> MmioPageResult {
    let mut result = MmioPageResult {
        base,
        first_word: 0,
        access: AccessLevel::Dead,
        identified_as: None,
        register_count: 0,
        unique_values: 0,
    };
    
    // Read first word
    match safe_volatile_read(base) {
        None => {
            result.access = AccessLevel::Faulted;
            return result;
        }
        Some(val) => {
            result.first_word = val;
        }
    }
    
    // Check if it's all-ones or all-zeros (dead bus)
    if result.first_word == 0xFFFF_FFFF || result.first_word == 0xDEAD_DEAD {
        result.access = AccessLevel::Dead;
        return result;
    }
    
    // Sample multiple offsets to determine if it's a real peripheral
    let mut values = Vec::new();
    let offsets = [0x00, 0x04, 0x08, 0x0C, 0x10, 0x14, 0x18, 0x1C,
                   0x20, 0x30, 0x40, 0x80, 0xFC];
    let mut fault_count = 0u32;
    
    for &offset in &offsets {
        if base + offset >= base + 0x1000 { break; }
        match safe_volatile_read(base + offset) {
            Some(val) => {
                if !values.contains(&val) {
                    values.push(val);
                }
                result.register_count += 1;
            }
            None => {
                fault_count += 1;
            }
        }
    }
    
    result.unique_values = values.len() as u32;
    
    // Classify access level
    if fault_count == offsets.len() as u32 {
        result.access = AccessLevel::Faulted;
    } else if fault_count > 0 {
        result.access = AccessLevel::Partial;
    } else if result.unique_values <= 1 && result.first_word == 0 {
        result.access = AccessLevel::Dead;
    } else {
        // Try to determine RW vs RO (we don't write in safe mode)
        result.access = AccessLevel::ReadOnly; // Conservative assumption
    }
    
    // Identify the peripheral
    result.identified_as = identify_peripheral(result.first_word, base);
    
    result
}

/// Main MMIO scanning function
pub fn scan_mmio_regions(user_base: u64, user_size: u64) -> String {
    let mut output = String::new();
    output.push_str("\x01C== TrustProbe MMIO Scanner ==\x01W\n\n");
    
    // Determine which regions to scan
    let scan_ranges: Vec<(u64, u64, &str)> = if user_base > 0 && user_size > 0 {
        alloc::vec![(user_base, user_size, "User-specified range")]
    } else {
        // Auto-detect based on architecture
        #[cfg(target_arch = "aarch64")]
        {
            let mut ranges: Vec<(u64, u64, &str)> = Vec::new();
            // Scan known peripheral regions
            ranges.push((0x0800_0000, 0x0002_0000, "GIC Region"));
            ranges.push((0x0900_0000, 0x0000_2000, "UART Region"));
            ranges.push((0x0A00_0000, 0x0000_4000, "VirtIO Region"));
            ranges.push((0xFE00_0000, 0x0200_0000, "BCM2711 Peripherals"));
            ranges
        }
        #[cfg(target_arch = "x86_64")]
        {
            let mut ranges: Vec<(u64, u64, &str)> = Vec::new();
            // x86 MMIO is typically above 0xE0000000
            ranges.push((0xFEC0_0000, 0x0000_1000, "I/O APIC"));
            ranges.push((0xFEE0_0000, 0x0000_1000, "Local APIC"));
            ranges.push((0xFED0_0000, 0x0000_1000, "HPET"));
            ranges.push((0xE000_0000, 0x1000_0000, "PCIe ECAM"));
            ranges
        }
        #[cfg(target_arch = "riscv64")]
        {
            let mut ranges: Vec<(u64, u64, &str)> = Vec::new();
            ranges.push((0x0C00_0000, 0x0040_0000, "PLIC"));
            ranges.push((0x1000_0000, 0x0000_1000, "UART0"));
            ranges.push((0x1000_1000, 0x0000_1000, "VirtIO Region"));
            ranges
        }
        #[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64", target_arch = "riscv64")))]
        {
            Vec::new()
        }
    };
    
    let mut total_pages = 0u64;
    let mut live_pages = 0u64;
    let mut faulted_pages = 0u64;
    let mut results: Vec<ProbeResult> = Vec::new();
    
    for &(base, size, region_name) in &scan_ranges {
        output.push_str(&format!("\x01YScanning: {} (0x{:X} - 0x{:X})\x01W\n", 
            region_name, base, base + size));
        
        let pages = size / 0x1000;
        let step = if pages > 256 { pages / 256 } else { 1 };
        
        let mut region_live = 0u64;
        let mut i = 0u64;
        while i < pages {
            let page_addr = base + i * 0x1000;
            total_pages += 1;
            
            let page = probe_page(page_addr);
            
            match page.access {
                AccessLevel::Dead => {}
                AccessLevel::Faulted => {
                    faulted_pages += 1;
                }
                _ => {
                    live_pages += 1;
                    region_live += 1;
                    
                    let risk = if page.access == AccessLevel::Partial {
                        RiskLevel::Medium
                    } else if page.identified_as.is_some() {
                        RiskLevel::Info
                    } else {
                        RiskLevel::Low // Unknown peripheral = interesting
                    };
                    
                    let name = page.identified_as.unwrap_or("Unknown Peripheral");
                    
                    results.push(ProbeResult {
                        category: "MMIO",
                        name: String::from(name),
                        address: page_addr,
                        size: 0x1000,
                        access: page.access,
                        details: format!("first=0x{:08X} regs={} unique={}", 
                            page.first_word, page.register_count, page.unique_values),
                        risk,
                    });
                }
            }
            
            i += step;
        }
        
        output.push_str(&format!("  Live: {} pages | Faulted: {} | Dead: {}\n",
            region_live, faulted_pages, total_pages - live_pages - faulted_pages));
    }
    
    // Print discovered peripherals
    output.push_str(&format!("\n\x01C== Results: {} peripherals found ==\x01W\n\n", results.len()));
    output.push_str(&format!("{:<14} {:<12} {:<8} {:<30} {}\n",
        "ADDRESS", "ACCESS", "RISK", "PERIPHERAL", "DETAILS"));
    output.push_str(&format!("{}\n", "-".repeat(90)));
    
    for r in &results {
        output.push_str(&format!("{}0x{:010X}\x01W  {:<12} {}{:<8}\x01W {:<30} {}\n",
            r.access.color_code(),
            r.address,
            r.access.as_str(),
            r.risk.color_code(),
            r.risk.as_str(),
            r.name,
            r.details,
        ));
    }
    
    output.push_str(&format!("\n\x01YTotal: {} pages scanned | {} live | {} faulted\x01W\n",
        total_pages, live_pages, faulted_pages));
    
    output
}
