















use alloc::string::String;
use alloc::format;
use alloc::vec::Vec;
use super::{Ju, AccessLevel, RiskLevel};


const CDF_: &[(u64, u64, &str)] = &[
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

const CDE_: &[(u64, u64, &str)] = &[
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

const CDG_: &[(u64, u64, &str)] = &[
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


const CEY_: &[(u32, &str)] = &[
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
pub struct Bmi {
    pub ar: u64,
    pub cqo: u32,
    pub vz: AccessLevel,
    pub izh: Option<&'static str>,
    pub lyp: u32,
    pub juq: u32,
}



fn pfd(ag: u64) -> Option<u32> {
    
    
    
    
    #[cfg(target_arch = "aarch64")]
    {
        
        
        if ag == 0 || ag > 0xFFFF_FFFF_FFFF {
            return None;
        }
        
        unsafe {
            let ptr = ag as *const u32;
            
            let ap = core::ptr::read_volatile(ptr);
            Some(ap)
        }
    }
    
    #[cfg(target_arch = "x86_64")]
    {
        if ag == 0 || ag > 0xFFFF_FFFF_FFFF {
            return None;
        }
        unsafe {
            let ptr = ag as *const u32;
            let ap = core::ptr::read_volatile(ptr);
            Some(ap)
        }
    }
    
    #[cfg(target_arch = "riscv64")]
    {
        if ag == 0 {
            return None;
        }
        unsafe {
            let ptr = ag as *const u32;
            let ap = core::ptr::read_volatile(ptr);
            Some(ap)
        }
    }
    
    #[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64", target_arch = "riscv64")))]
    {
        let _ = ag;
        None
    }
}


fn trp(cqo: u32, ag: u64) -> Option<&'static str> {
    
    for &(sj, j) in CEY_ {
        if cqo == sj {
            return Some(j);
        }
    }
    
    
    for &(ar, aw, j) in CDF_.iter()
        .rh(CDE_.iter())
        .rh(CDG_.iter())
    {
        if ag >= ar && ag < ar + aw {
            return Some(j);
        }
    }
    
    None
}


fn vma(ar: u64) -> Bmi {
    let mut result = Bmi {
        ar,
        cqo: 0,
        vz: AccessLevel::Ez,
        izh: None,
        lyp: 0,
        juq: 0,
    };
    
    
    match pfd(ar) {
        None => {
            result.vz = AccessLevel::In;
            return result;
        }
        Some(ap) => {
            result.cqo = ap;
        }
    }
    
    
    if result.cqo == 0xFFFF_FFFF || result.cqo == 0xDEAD_DEAD {
        result.vz = AccessLevel::Ez;
        return result;
    }
    
    
    let mut alv = Vec::new();
    let bkr = [0x00, 0x04, 0x08, 0x0C, 0x10, 0x14, 0x18, 0x1C,
                   0x20, 0x30, 0x40, 0x80, 0xFC];
    let mut kvg = 0u32;
    
    for &l in &bkr {
        if ar + l >= ar + 0x1000 { break; }
        match pfd(ar + l) {
            Some(ap) => {
                if !alv.contains(&ap) {
                    alv.push(ap);
                }
                result.lyp += 1;
            }
            None => {
                kvg += 1;
            }
        }
    }
    
    result.juq = alv.len() as u32;
    
    
    if kvg == bkr.len() as u32 {
        result.vz = AccessLevel::In;
    } else if kvg > 0 {
        result.vz = AccessLevel::Adq;
    } else if result.juq <= 1 && result.cqo == 0 {
        result.vz = AccessLevel::Ez;
    } else {
        
        result.vz = AccessLevel::Bz; 
    }
    
    
    result.izh = trp(result.cqo, ar);
    
    result
}


pub fn pge(mom: u64, pxs: u64) -> String {
    let mut an = String::new();
    an.t("\x01C== TrustProbe MMIO Scanner ==\x01W\n\n");
    
    
    let wdw: Vec<(u64, u64, &str)> = if mom > 0 && pxs > 0 {
        alloc::vec![(mom, pxs, "User-specified range")]
    } else {
        
        #[cfg(target_arch = "aarch64")]
        {
            let mut bnz: Vec<(u64, u64, &str)> = Vec::new();
            
            bnz.push((0x0800_0000, 0x0002_0000, "GIC Region"));
            bnz.push((0x0900_0000, 0x0000_2000, "UART Region"));
            bnz.push((0x0A00_0000, 0x0000_4000, "VirtIO Region"));
            bnz.push((0xFE00_0000, 0x0200_0000, "BCM2711 Peripherals"));
            bnz
        }
        #[cfg(target_arch = "x86_64")]
        {
            let mut bnz: Vec<(u64, u64, &str)> = Vec::new();
            
            bnz.push((0xFEC0_0000, 0x0000_1000, "I/O APIC"));
            bnz.push((0xFEE0_0000, 0x0000_1000, "Local APIC"));
            bnz.push((0xFED0_0000, 0x0000_1000, "HPET"));
            bnz.push((0xE000_0000, 0x1000_0000, "PCIe ECAM"));
            bnz
        }
        #[cfg(target_arch = "riscv64")]
        {
            let mut bnz: Vec<(u64, u64, &str)> = Vec::new();
            bnz.push((0x0C00_0000, 0x0040_0000, "PLIC"));
            bnz.push((0x1000_0000, 0x0000_1000, "UART0"));
            bnz.push((0x1000_1000, 0x0000_1000, "VirtIO Region"));
            bnz
        }
        #[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64", target_arch = "riscv64")))]
        {
            Vec::new()
        }
    };
    
    let mut jtu = 0u64;
    let mut ljc = 0u64;
    let mut iub = 0u64;
    let mut hd: Vec<Ju> = Vec::new();
    
    for &(ar, aw, lyn) in &wdw {
        an.t(&format!("\x01YScanning: {} (0x{:X} - 0x{:X})\x01W\n", 
            lyn, ar, ar + aw));
        
        let bcd = aw / 0x1000;
        let gu = if bcd > 256 { bcd / 256 } else { 1 };
        
        let mut pbj = 0u64;
        let mut a = 0u64;
        while a < bcd {
            let dkk = ar + a * 0x1000;
            jtu += 1;
            
            let awl = vma(dkk);
            
            match awl.vz {
                AccessLevel::Ez => {}
                AccessLevel::In => {
                    iub += 1;
                }
                _ => {
                    ljc += 1;
                    pbj += 1;
                    
                    let bhz = if awl.vz == AccessLevel::Adq {
                        RiskLevel::Bc
                    } else if awl.izh.is_some() {
                        RiskLevel::V
                    } else {
                        RiskLevel::Eg 
                    };
                    
                    let j = awl.izh.unwrap_or("Unknown Peripheral");
                    
                    hd.push(Ju {
                        gb: "MMIO",
                        j: String::from(j),
                        re: dkk,
                        aw: 0x1000,
                        vz: awl.vz,
                        yw: format!("first=0x{:08X} regs={} unique={}", 
                            awl.cqo, awl.lyp, awl.juq),
                        bhz,
                    });
                }
            }
            
            a += gu;
        }
        
        an.t(&format!("  Live: {} pages | Faulted: {} | Dead: {}\n",
            pbj, iub, jtu - ljc - iub));
    }
    
    
    an.t(&format!("\n\x01C== Results: {} peripherals found ==\x01W\n\n", hd.len()));
    an.t(&format!("{:<14} {:<12} {:<8} {:<30} {}\n",
        "ADDRESS", "ACCESS", "RISK", "PERIPHERAL", "DETAILS"));
    an.t(&format!("{}\n", "-".afd(90)));
    
    for m in &hd {
        an.t(&format!("{}0x{:010X}\x01W  {:<12} {}{:<8}\x01W {:<30} {}\n",
            m.vz.cpk(),
            m.re,
            m.vz.as_str(),
            m.bhz.cpk(),
            m.bhz.as_str(),
            m.j,
            m.yw,
        ));
    }
    
    an.t(&format!("\n\x01YTotal: {} pages scanned | {} live | {} faulted\x01W\n",
        jtu, ljc, iub));
    
    an
}
