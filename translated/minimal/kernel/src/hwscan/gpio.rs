













use alloc::string::String;
use alloc::format;
use alloc::vec::Vec;

fn sm(addr: u64) -> Option<u32> {
    if addr == 0 { return None; }
    unsafe {
        let ptr = addr as *const u32;
        Some(core::ptr::read_volatile(ptr))
    }
}


#[cfg(target_arch = "aarch64")]
const CBI_: &[(u64, u32, &str)] = &[
    (0x0903_0000, 8,  "QEMU virt PL061 GPIO"),
    (0xFE20_0000, 58, "BCM2711 GPIO (58 pins)"),
    (0x0F10_0000, 150, "Snapdragon TLMM GPIO"),
    (0x1000_5000, 288, "MediaTek GPIO"),
];


const BNS_: &[&str] = &[
    "INPUT", "OUTPUT", "ALT5", "ALT4", "ALT0", "ALT1", "ALT2", "ALT3"
];


const BTV_: &[(u32, u32, &str)] = &[
    
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
const CQD_: &[(u32, &str)] = &[
    (0, "GPIO"),
    (1, "UART_TX/BLSP"),
    (2, "I2C/QUP"),
    (3, "SPI"),
    (4, "JTAG/SDC"),
    (5, "RESERVED/TEST"),
];


fn lch(base: u64, num_pins: u32) -> String {
    let mut out = String::new();
    
    out.push_str(&format!("{:<6} {:<10} {:<8} {}\n",
        "PIN", "FUNCTION", "LEVEL", "NOTES"));
    out.push_str(&format!("{}\n", "-".repeat(55)));
    
    
    let mya = sm(base + 0x34).unwrap_or(0);
    let myb = sm(base + 0x38).unwrap_or(0);
    
    let ndg = core::cmp::min(num_pins, 54);
    let mut ham = Vec::new();
    let mut gek = Vec::new();
    
    for pin in 0..ndg {
        
        let lzv = (pin / 10) as u64;
        let lzu = ((pin % 10) * 3) as u32;
        
        if let Some(fsel) = sm(base + lzv * 4) {
            let func = (fsel >> lzu) & 0x7;
            let enp = BNS_.get(func as usize).unwrap_or(&"???");
            
            
            let level = if pin < 32 {
                (mya >> pin) & 1
            } else {
                (myb >> (pin - 32)) & 1
            };
            
            
            let mut notes = String::new();
            for &(dpin, dalt, desc) in BTV_ {
                
                let jvk = match dalt {
                    0 => 4, 
                    1 => 5, 
                    2 => 6, 
                    3 => 7, 
                    4 => 3, 
                    5 => 2, 
                    _ => 0xFF,
                };
                
                if pin == dpin && func == jvk {
                    notes = format!("\x01R{}\x01W", desc);
                    if desc.contains("UART") {
                        ham.push(pin);
                    }
                    if desc.contains("JTAG") {
                        gek.push(pin);
                    }
                }
            }
            
            
            if func != 0 || !notes.is_empty() {
                out.push_str(&format!("GPIO{:<3} {:<10} {:<8} {}\n",
                    pin, enp, level, notes));
            }
        }
    }
    
    
    if !ham.is_empty() {
        out.push_str(&format!("\n\x01R!! UART debug interface ACTIVE on pins: {:?} !!\x01W\n", ham));
        out.push_str("   Connect a USB-UART adapter to capture firmware debug output\n");
    }
    if !gek.is_empty() {
        out.push_str(&format!("\n\x01R!! JTAG debug interface ACTIVE on pins: {:?} !!\x01W\n", gek));
        out.push_str("   Connect a JTAG probe for full on-chip debugging\n");
    }
    
    out
}


#[cfg(target_arch = "aarch64")]
fn lcs(base: u64, num_pins: u32) -> String {
    let mut out = String::new();
    
    out.push_str(&format!("{:<6} {:<10} {:<8} {}\n",
        "PIN", "FUNCTION", "DIR", "FLAGS"));
    out.push_str(&format!("{}\n", "-".repeat(50)));
    
    let max = core::cmp::min(num_pins, 200);
    let mut ejz = Vec::new();
    
    for pin in 0..max {
        
        let kih = base + (pin as u64) * 0x1000;
        
        if let Some(cfg) = sm(kih) {
            let func = (cfg >> 2) & 0xF;
            let leu = (cfg >> 9) & 1;
            let nzi = cfg & 0x3;
            
            let enp = CQD_.iter()
                .find(|&&(f, _)| f == func)
                .map(|&(_, name)| name)
                .unwrap_or("PERIPH");
            
            let direction = if leu == 1 { "OUT" } else { "IN" };
            let nzk = match nzi {
                0 => "",
                1 => "PD",
                2 => "KEEPER",
                3 => "PU",
                _ => "",
            };
            
            
            if func >= 4 {
                ejz.push((pin, enp));
            }
            
            
            if func != 0 {
                out.push_str(&format!("GPIO{:<3} {:<10} {:<8} {}\n",
                    pin, enp, direction, nzk));
            }
        }
    }
    
    if !ejz.is_empty() {
        out.push_str(&format!("\n\x01Y[!] Potential debug/test pins found: {}\x01W\n", ejz.len()));
        for (pin, func) in &ejz {
            out.push_str(&format!("    GPIO{}: {}\n", pin, func));
        }
    }
    
    out
}


pub fn iwr() -> String {
    let mut output = String::new();
    
    #[cfg(target_arch = "aarch64")]
    {
        output.push_str("\x01C== TrustProbe: GPIO Pin & Debug Interface Scanner ==\x01W\n\n");
        output.push_str("Scanning GPIO controllers for muxed debug interfaces...\n\n");
        
        for &(base, num_pins, name) in CBI_ {
            
            if let Some(val) = sm(base) {
                if val != 0xFFFFFFFF {
                    output.push_str(&format!("\x01G[FOUND]\x01W {} @ 0x{:08X} ({} pins)\n\n",
                        name, base, num_pins));
                    
                    if name.contains("BCM") {
                        output.push_str(&lch(base, num_pins));
                    } else if name.contains("Snapdragon") || name.contains("TLMM") {
                        output.push_str(&lcs(base, num_pins));
                    } else {
                        
                        output.push_str(&format!("  Data: 0x{:08X}\n", val));
                        if let Some(it) = sm(base + 0x400) {
                            output.push_str(&format!("  Direction: 0x{:08X}\n", it));
                        }
                    }
                    output.push_str("\n");
                }
            }
        }
        
        output.push_str("\x01Y--- Debug Interface Summary ---\x01W\n");
        output.push_str("TrustProbe can identify:\n");
        output.push_str("  - UART consoles (vendor debug output / bootloader shell)\n");
        output.push_str("  - JTAG/SWD (full on-chip debug access)\n");
        output.push_str("  - Factory test pins (unlocks hidden features)\n");
        output.push_str("  - I2C/SPI debug buses (sensor/PMIC access)\n");
    }
    
    #[cfg(target_arch = "x86_64")]
    {
        output.push_str("\x01C== TrustProbe: x86 Debug Port Scanner ==\x01W\n\n");
        output.push_str("x86 debug interfaces:\n");
        
        
        let kvy = [
            (0x3F8u64, "COM1"),
            (0x2F8u64, "COM2"),
            (0x3E8u64, "COM3"),
            (0x2E8u64, "COM4"),
        ];
        
        output.push_str("\nLegacy COM ports (I/O space):\n");
        for &(port, name) in &kvy {
            output.push_str(&format!("  {} @ 0x{:03X}: ", name, port));
            
            
            output.push_str("(use port I/O to probe)\n");
        }
        
        output.push_str("\nDebug interfaces to check:\n");
        output.push_str("  - Intel DCI (Direct Connect Interface)\n");
        output.push_str("  - USB Debug Port (EHCI debug capability)\n");
        output.push_str("  - SPI flash (BIOS/UEFI image access)\n");
    }
    
    #[cfg(target_arch = "riscv64")]
    {
        output.push_str("\x01C== TrustProbe: RISC-V GPIO Scanner ==\x01W\n\n");
        output.push_str("Scanning for debug interfaces...\n\n");
        
        
        let uart_base = 0x1000_0000u64;
        if let Some(val) = sm(uart_base) {
            output.push_str(&format!("\x01G[FOUND]\x01W UART @ 0x{:08X}\n", uart_base));
        }
    }
    
    output
}
