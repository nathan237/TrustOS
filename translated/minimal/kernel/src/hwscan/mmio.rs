















use alloc::string::String;
use alloc::format;
use alloc::vec::Vec;
use super::{Dy, AccessLevel, RiskLevel};


const CGO_: &[(u64, u64, &str)] = &[
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

const CGN_: &[(u64, u64, &str)] = &[
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

const CGP_: &[(u64, u64, &str)] = &[
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


const CIH_: &[(u32, &str)] = &[
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


#[derive(Debug, Clone)]
pub struct Abj {
    pub base: u64,
    pub first_word: u32,
    pub access: AccessLevel,
    pub identified_as: Option<&'static str>,
    pub register_count: u32,
    pub unique_values: u32,
}



fn jcg(addr: u64) -> Option<u32> {
    
    
    
    
    #[cfg(target_arch = "aarch64")]
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


fn mns(first_word: u32, addr: u64) -> Option<&'static str> {
    
    for &(magic, name) in CIH_ {
        if first_word == magic {
            return Some(name);
        }
    }
    
    
    for &(base, size, name) in CGO_.iter()
        .chain(CGN_.iter())
        .chain(CGP_.iter())
    {
        if addr >= base && addr < base + size {
            return Some(name);
        }
    }
    
    None
}


fn nxu(base: u64) -> Abj {
    let mut result = Abj {
        base,
        first_word: 0,
        access: AccessLevel::Dead,
        identified_as: None,
        register_count: 0,
        unique_values: 0,
    };
    
    
    match jcg(base) {
        None => {
            result.access = AccessLevel::Faulted;
            return result;
        }
        Some(val) => {
            result.first_word = val;
        }
    }
    
    
    if result.first_word == 0xFFFF_FFFF || result.first_word == 0xDEAD_DEAD {
        result.access = AccessLevel::Dead;
        return result;
    }
    
    
    let mut values = Vec::new();
    let agv = [0x00, 0x04, 0x08, 0x0C, 0x10, 0x14, 0x18, 0x1C,
                   0x20, 0x30, 0x40, 0x80, 0xFC];
    let mut fwg = 0u32;
    
    for &offset in &agv {
        if base + offset >= base + 0x1000 { break; }
        match jcg(base + offset) {
            Some(val) => {
                if !values.contains(&val) {
                    values.push(val);
                }
                result.register_count += 1;
            }
            None => {
                fwg += 1;
            }
        }
    }
    
    result.unique_values = values.len() as u32;
    
    
    if fwg == agv.len() as u32 {
        result.access = AccessLevel::Faulted;
    } else if fwg > 0 {
        result.access = AccessLevel::Partial;
    } else if result.unique_values <= 1 && result.first_word == 0 {
        result.access = AccessLevel::Dead;
    } else {
        
        result.access = AccessLevel::ReadOnly; 
    }
    
    
    result.identified_as = mns(result.first_word, base);
    
    result
}


pub fn jdg(hax: u64, user_size: u64) -> String {
    let mut output = String::new();
    output.push_str("\x01C== TrustProbe MMIO Scanner ==\x01W\n\n");
    
    
    let ole: Vec<(u64, u64, &str)> = if hax > 0 && user_size > 0 {
        alloc::vec![(hax, user_size, "User-specified range")]
    } else {
        
        #[cfg(target_arch = "aarch64")]
        {
            let mut aef: Vec<(u64, u64, &str)> = Vec::new();
            
            aef.push((0x0800_0000, 0x0002_0000, "GIC Region"));
            aef.push((0x0900_0000, 0x0000_2000, "UART Region"));
            aef.push((0x0A00_0000, 0x0000_4000, "VirtIO Region"));
            aef.push((0xFE00_0000, 0x0200_0000, "BCM2711 Peripherals"));
            aef
        }
        #[cfg(target_arch = "x86_64")]
        {
            let mut aef: Vec<(u64, u64, &str)> = Vec::new();
            
            aef.push((0xFEC0_0000, 0x0000_1000, "I/O APIC"));
            aef.push((0xFEE0_0000, 0x0000_1000, "Local APIC"));
            aef.push((0xFED0_0000, 0x0000_1000, "HPET"));
            aef.push((0xE000_0000, 0x1000_0000, "PCIe ECAM"));
            aef
        }
        #[cfg(target_arch = "riscv64")]
        {
            let mut aef: Vec<(u64, u64, &str)> = Vec::new();
            aef.push((0x0C00_0000, 0x0040_0000, "PLIC"));
            aef.push((0x1000_0000, 0x0000_1000, "UART0"));
            aef.push((0x1000_1000, 0x0000_1000, "VirtIO Region"));
            aef
        }
        #[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64", target_arch = "riscv64")))]
        {
            Vec::new()
        }
    };
    
    let mut fdh = 0u64;
    let mut gfu = 0u64;
    let mut emh = 0u64;
    let mut results: Vec<Dy> = Vec::new();
    
    for &(base, size, region_name) in &ole {
        output.push_str(&format!("\x01YScanning: {} (0x{:X} - 0x{:X})\x01W\n", 
            region_name, base, base + size));
        
        let acg = size / 0x1000;
        let step = if acg > 256 { acg / 256 } else { 1 };
        
        let mut izi = 0u64;
        let mut i = 0u64;
        while i < acg {
            let page_addr = base + i * 0x1000;
            fdh += 1;
            
            let za = nxu(page_addr);
            
            match za.access {
                AccessLevel::Dead => {}
                AccessLevel::Faulted => {
                    emh += 1;
                }
                _ => {
                    gfu += 1;
                    izi += 1;
                    
                    let risk = if za.access == AccessLevel::Partial {
                        RiskLevel::Medium
                    } else if za.identified_as.is_some() {
                        RiskLevel::Info
                    } else {
                        RiskLevel::Low 
                    };
                    
                    let name = za.identified_as.unwrap_or("Unknown Peripheral");
                    
                    results.push(Dy {
                        category: "MMIO",
                        name: String::from(name),
                        address: page_addr,
                        size: 0x1000,
                        access: za.access,
                        details: format!("first=0x{:08X} regs={} unique={}", 
                            za.first_word, za.register_count, za.unique_values),
                        risk,
                    });
                }
            }
            
            i += step;
        }
        
        output.push_str(&format!("  Live: {} pages | Faulted: {} | Dead: {}\n",
            izi, emh, fdh - gfu - emh));
    }
    
    
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
        fdh, gfu, emh));
    
    output
}
