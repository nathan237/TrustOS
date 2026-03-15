













use alloc::string::String;
use alloc::format;
use alloc::vec::Vec;

fn akp(ag: u64) -> Option<u32> {
    if ag == 0 { return None; }
    unsafe {
        let ptr = ag as *const u32;
        Some(core::ptr::read_volatile(ptr))
    }
}


#[cfg(target_arch = "aarch64")]
const BYC_: &[(u64, u32, &str)] = &[
    (0x0903_0000, 8,  "QEMU virt PL061 GPIO"),
    (0xFE20_0000, 58, "BCM2711 GPIO (58 pins)"),
    (0x0F10_0000, 150, "Snapdragon TLMM GPIO"),
    (0x1000_5000, 288, "MediaTek GPIO"),
];


const BLA_: &[&str] = &[
    "INPUT", "OUTPUT", "ALT5", "ALT4", "ALT0", "ALT1", "ALT2", "ALT3"
];


const BRA_: &[(u32, u32, &str)] = &[
    
    (14, 0, "BCM UART0 TXD (ALT0)"),
    (15, 0, "BCM UART0 RXD (ALT0)"),
    (0, 2, "BCM UART2 TXD (ALT2 — hidden debug)"),
    (1, 2, "BCM UART2 RXD (ALT2 — hidden debug)"),
    (4, 2, "BCM JTAG TDI (ALT2)"),
    (5, 2, "BCM JTAG TDO (ALT2)"),
    (6, 2, "BCM JTAG RTCK (ALT2)"),
    (12, 2, "BCM JTAG TMS (ALT2)"),
    (13, 2, "BCM JTAG TCK (ALT2)"),
    (22, 3, "BCM ARM JTAG TRST (ALT3)"),
    (23, 3, "BCM ARM JTAG RTCK (ALT3)"),
    (24, 3, "BCM ARM JTAG TDO (ALT3)"),
    (25, 3, "BCM ARM JTAG TCK (ALT3)"),
    (26, 3, "BCM ARM JTAG TDI (ALT3)"),
    (27, 3, "BCM ARM JTAG TMS (ALT3)"),
];


#[cfg(target_arch = "aarch64")]
const CMU_: &[(u32, &str)] = &[
    (0, "GPIO"),
    (1, "UART_TX/BLSP"),
    (2, "I2C/QUP"),
    (3, "SPI"),
    (4, "JTAG/SDC"),
    (5, "RESERVED/TEST"),
];


fn rue(ar: u64, fpj: u32) -> String {
    let mut bd = String::new();
    
    bd.t(&format!("{:<6} {:<10} {:<8} {}\n",
        "PIN", "FUNCTION", "LEVEL", "NOTES"));
    bd.t(&format!("{}\n", "-".afd(55)));
    
    
    let uee = akp(ar + 0x34).unwrap_or(0);
    let uef = akp(ar + 0x38).unwrap_or(0);
    
    let ult = core::cmp::v(fpj, 54);
    let mut mnx = Vec::new();
    let mut lha = Vec::new();
    
    for pin in 0..ult {
        
        let syn = (pin / 10) as u64;
        let sym = ((pin % 10) * 3) as u32;
        
        if let Some(syl) = akp(ar + syn * 4) {
            let ke = (syl >> sym) & 0x7;
            let iwc = BLA_.get(ke as usize).unwrap_or(&"???");
            
            
            let jy = if pin < 32 {
                (uee >> pin) & 1
            } else {
                (uef >> (pin - 32)) & 1
            };
            
            
            let mut ts = String::new();
            for &(sau, rti, desc) in BRA_ {
                
                let qhm = match rti {
                    0 => 4, 
                    1 => 5, 
                    2 => 6, 
                    3 => 7, 
                    4 => 3, 
                    5 => 2, 
                    _ => 0xFF,
                };
                
                if pin == sau && ke == qhm {
                    ts = format!("\x01R{}\x01W", desc);
                    if desc.contains("UART") {
                        mnx.push(pin);
                    }
                    if desc.contains("JTAG") {
                        lha.push(pin);
                    }
                }
            }
            
            
            if ke != 0 || !ts.is_empty() {
                bd.t(&format!("GPIO{:<3} {:<10} {:<8} {}\n",
                    pin, iwc, jy, ts));
            }
        }
    }
    
    
    if !mnx.is_empty() {
        bd.t(&format!("\n\x01R!! UART debug interface ACTIVE on pins: {:?} !!\x01W\n", mnx));
        bd.t("   Connect a USB-UART adapter to capture firmware debug output\n");
    }
    if !lha.is_empty() {
        bd.t(&format!("\n\x01R!! JTAG debug interface ACTIVE on pins: {:?} !!\x01W\n", lha));
        bd.t("   Connect a JTAG probe for full on-chip debugging\n");
    }
    
    bd
}


#[cfg(target_arch = "aarch64")]
fn ruq(ar: u64, fpj: u32) -> String {
    let mut bd = String::new();
    
    bd.t(&format!("{:<6} {:<10} {:<8} {}\n",
        "PIN", "FUNCTION", "DIR", "FLAGS"));
    bd.t(&format!("{}\n", "-".afd(50)));
    
    let am = core::cmp::v(fpj, 200);
    let mut iqr = Vec::new();
    
    for pin in 0..am {
        
        let qxx = ar + (pin as u64) * 0x1000;
        
        if let Some(cfg) = akp(qxx) {
            let ke = (cfg >> 2) & 0xF;
            let rxq = (cfg >> 9) & 1;
            let vny = cfg & 0x3;
            
            let iwc = CMU_.iter()
                .du(|&&(bb, _)| bb == ke)
                .map(|&(_, j)| j)
                .unwrap_or("PERIPH");
            
            let sz = if rxq == 1 { "OUT" } else { "IN" };
            let voa = match vny {
                0 => "",
                1 => "PD",
                2 => "KEEPER",
                3 => "PU",
                _ => "",
            };
            
            
            if ke >= 4 {
                iqr.push((pin, iwc));
            }
            
            
            if ke != 0 {
                bd.t(&format!("GPIO{:<3} {:<10} {:<8} {}\n",
                    pin, iwc, sz, voa));
            }
        }
    }
    
    if !iqr.is_empty() {
        bd.t(&format!("\n\x01Y[!] Potential debug/test pins found: {}\x01W\n", iqr.len()));
        for (pin, ke) in &iqr {
            bd.t(&format!("    GPIO{}: {}\n", pin, ke));
        }
    }
    
    bd
}


pub fn oxx() -> String {
    let mut an = String::new();
    
    #[cfg(target_arch = "aarch64")]
    {
        an.t("\x01C== TrustProbe: GPIO Pin & Debug Interface Scanner ==\x01W\n\n");
        an.t("Scanning GPIO controllers for muxed debug interfaces...\n\n");
        
        for &(ar, fpj, j) in BYC_ {
            
            if let Some(ap) = akp(ar) {
                if ap != 0xFFFFFFFF {
                    an.t(&format!("\x01G[FOUND]\x01W {} @ 0x{:08X} ({} pins)\n\n",
                        j, ar, fpj));
                    
                    if j.contains("BCM") {
                        an.t(&rue(ar, fpj));
                    } else if j.contains("Snapdragon") || j.contains("TLMM") {
                        an.t(&ruq(ar, fpj));
                    } else {
                        
                        an.t(&format!("  Data: 0x{:08X}\n", ap));
                        if let Some(te) = akp(ar + 0x400) {
                            an.t(&format!("  Direction: 0x{:08X}\n", te));
                        }
                    }
                    an.t("\n");
                }
            }
        }
        
        an.t("\x01Y--- Debug Interface Summary ---\x01W\n");
        an.t("TrustProbe can identify:\n");
        an.t("  - UART consoles (vendor debug output / bootloader shell)\n");
        an.t("  - JTAG/SWD (full on-chip debug access)\n");
        an.t("  - Factory test pins (unlocks hidden features)\n");
        an.t("  - I2C/SPI debug buses (sensor/PMIC access)\n");
    }
    
    #[cfg(target_arch = "x86_64")]
    {
        an.t("\x01C== TrustProbe: x86 Debug Port Scanner ==\x01W\n\n");
        an.t("x86 debug interfaces:\n");
        
        
        let rmo = [
            (0x3F8u64, "COM1"),
            (0x2F8u64, "COM2"),
            (0x3E8u64, "COM3"),
            (0x2E8u64, "COM4"),
        ];
        
        an.t("\nLegacy COM ports (I/O space):\n");
        for &(port, j) in &rmo {
            an.t(&format!("  {} @ 0x{:03X}: ", j, port));
            
            
            an.t("(use port I/O to probe)\n");
        }
        
        an.t("\nDebug interfaces to check:\n");
        an.t("  - Intel DCI (Direct Connect Interface)\n");
        an.t("  - USB Debug Port (EHCI debug capability)\n");
        an.t("  - SPI flash (BIOS/UEFI image access)\n");
    }
    
    #[cfg(target_arch = "riscv64")]
    {
        an.t("\x01C== TrustProbe: RISC-V GPIO Scanner ==\x01W\n\n");
        an.t("Scanning for debug interfaces...\n\n");
        
        
        let cnl = 0x1000_0000u64;
        if let Some(ap) = akp(cnl) {
            an.t(&format!("\x01G[FOUND]\x01W UART @ 0x{:08X}\n", cnl));
        }
    }
    
    an
}
