//! GPIO Pin Prober — Hidden UART/JTAG Discovery
//!
//! On many SoCs, debug interfaces (UART, JTAG, SWD) are multiplexed
//! onto GPIO pins. Vendors often disable them in production firmware
//! but the hardware capability remains. By probing GPIO pin muxing
//! and function selection registers, we can discover:
//!   - Hidden UART consoles (firmware debug output)
//!   - JTAG/SWD interfaces for on-chip debugging
//!   - Test pins (factory test modes)
//!   - Undocumented peripheral connections
//!
//! This is how hardware security researchers find debug access on
//! "locked down" devices.

use alloc::string::String;
use alloc::format;
use alloc::vec::Vec;

fn safe_read(addr: u64) -> Option<u32> {
    if addr == 0 { return None; }
    unsafe {
        let ptr = addr as *const u32;
        Some(core::ptr::read_volatile(ptr))
    }
}

/// GPIO controller known bases
#[cfg(target_arch = "aarch64")]
const GPIO_CONTROLLERS: &[(u64, u32, &str)] = &[
    (0x0903_0000, 8,  "QEMU virt PL061 GPIO"),
    (0xFE20_0000, 58, "BCM2711 GPIO (58 pins)"),
    (0x0F10_0000, 150, "Snapdragon TLMM GPIO"),
    (0x1000_5000, 288, "MediaTek GPIO"),
];

/// BCM2711 GPIO function select values
const BCM_FSEL_NAMES: &[&str] = &[
    "INPUT", "OUTPUT", "ALT5", "ALT4", "ALT0", "ALT1", "ALT2", "ALT3"
];

/// Known GPIO alternate functions that indicate debug interfaces
const DEBUG_ALT_FUNCTIONS: &[(u32, u32, &str)] = &[
    // (pin, alt_func, description) for BCM2711
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

/// Qualcomm TLMM GPIO configuration
#[cfg(target_arch = "aarch64")]
const QCOM_TLMM_FUNC_NAMES: &[(u32, &str)] = &[
    (0, "GPIO"),
    (1, "UART_TX/BLSP"),
    (2, "I2C/QUP"),
    (3, "SPI"),
    (4, "JTAG/SDC"),
    (5, "RESERVED/TEST"),
];

/// Decode BCM2711 GPIO pin functions
fn decode_bcm_gpio(base: u64, num_pins: u32) -> String {
    let mut out = String::new();
    
    out.push_str(&format!("{:<6} {:<10} {:<8} {}\n",
        "PIN", "FUNCTION", "LEVEL", "NOTES"));
    out.push_str(&format!("{}\n", "-".repeat(55)));
    
    // Read pin levels
    let level0 = safe_read(base + 0x34).unwrap_or(0);
    let level1 = safe_read(base + 0x38).unwrap_or(0);
    
    let max_pins = core::cmp::min(num_pins, 54);
    let mut uart_pins = Vec::new();
    let mut jtag_pins = Vec::new();
    
    for pin in 0..max_pins {
        // Function select: GPFSEL0-5 at offset 0x00-0x14
        let fsel_reg = (pin / 10) as u64;
        let fsel_bit = ((pin % 10) * 3) as u32;
        
        if let Some(fsel) = safe_read(base + fsel_reg * 4) {
            let func = (fsel >> fsel_bit) & 0x7;
            let func_name = BCM_FSEL_NAMES.get(func as usize).unwrap_or(&"???");
            
            // Get pin level
            let level = if pin < 32 {
                (level0 >> pin) & 1
            } else {
                (level1 >> (pin - 32)) & 1
            };
            
            // Check if this is a debug pin
            let mut notes = String::new();
            for &(dpin, dalt, desc) in DEBUG_ALT_FUNCTIONS {
                // func values: 0=IN, 1=OUT, 2=ALT5, 3=ALT4, 4=ALT0, 5=ALT1, 6=ALT2, 7=ALT3
                let alt_to_func = match dalt {
                    0 => 4, // ALT0
                    1 => 5, // ALT1
                    2 => 6, // ALT2
                    3 => 7, // ALT3
                    4 => 3, // ALT4
                    5 => 2, // ALT5
                    _ => 0xFF,
                };
                
                if pin == dpin && func == alt_to_func {
                    notes = format!("\x01R{}\x01W", desc);
                    if desc.contains("UART") {
                        uart_pins.push(pin);
                    }
                    if desc.contains("JTAG") {
                        jtag_pins.push(pin);
                    }
                }
            }
            
            // Only print pins with non-input functions or debug notes
            if func != 0 || !notes.is_empty() {
                out.push_str(&format!("GPIO{:<3} {:<10} {:<8} {}\n",
                    pin, func_name, level, notes));
            }
        }
    }
    
    // Summary
    if !uart_pins.is_empty() {
        out.push_str(&format!("\n\x01R!! UART debug interface ACTIVE on pins: {:?} !!\x01W\n", uart_pins));
        out.push_str("   Connect a USB-UART adapter to capture firmware debug output\n");
    }
    if !jtag_pins.is_empty() {
        out.push_str(&format!("\n\x01R!! JTAG debug interface ACTIVE on pins: {:?} !!\x01W\n", jtag_pins));
        out.push_str("   Connect a JTAG probe for full on-chip debugging\n");
    }
    
    out
}

/// Decode Qualcomm TLMM GPIO
#[cfg(target_arch = "aarch64")]
fn decode_qcom_gpio(base: u64, num_pins: u32) -> String {
    let mut out = String::new();
    
    out.push_str(&format!("{:<6} {:<10} {:<8} {}\n",
        "PIN", "FUNCTION", "DIR", "FLAGS"));
    out.push_str(&format!("{}\n", "-".repeat(50)));
    
    let max = core::cmp::min(num_pins, 200);
    let mut debug_pins = Vec::new();
    
    for pin in 0..max {
        // Each GPIO has a config register at base + pin * 0x1000
        let cfg_addr = base + (pin as u64) * 0x1000;
        
        if let Some(cfg) = safe_read(cfg_addr) {
            let func = (cfg >> 2) & 0xF;
            let dir_out = (cfg >> 9) & 1;
            let pull = cfg & 0x3;
            
            let func_name = QCOM_TLMM_FUNC_NAMES.iter()
                .find(|&&(f, _)| f == func)
                .map(|&(_, name)| name)
                .unwrap_or("PERIPH");
            
            let direction = if dir_out == 1 { "OUT" } else { "IN" };
            let pull_str = match pull {
                0 => "",
                1 => "PD",
                2 => "KEEPER",
                3 => "PU",
                _ => "",
            };
            
            // Flag interesting functions
            if func >= 4 {
                debug_pins.push((pin, func_name));
            }
            
            // Only show non-GPIO (muxed) pins
            if func != 0 {
                out.push_str(&format!("GPIO{:<3} {:<10} {:<8} {}\n",
                    pin, func_name, direction, pull_str));
            }
        }
    }
    
    if !debug_pins.is_empty() {
        out.push_str(&format!("\n\x01Y[!] Potential debug/test pins found: {}\x01W\n", debug_pins.len()));
        for (pin, func) in &debug_pins {
            out.push_str(&format!("    GPIO{}: {}\n", pin, func));
        }
    }
    
    out
}

/// Main GPIO probe
pub fn probe_gpio_pins() -> String {
    let mut output = String::new();
    
    #[cfg(target_arch = "aarch64")]
    {
        output.push_str("\x01C== TrustProbe: GPIO Pin & Debug Interface Scanner ==\x01W\n\n");
        output.push_str("Scanning GPIO controllers for muxed debug interfaces...\n\n");
        
        for &(base, num_pins, name) in GPIO_CONTROLLERS {
            // Check if controller exists
            if let Some(val) = safe_read(base) {
                if val != 0xFFFFFFFF {
                    output.push_str(&format!("\x01G[FOUND]\x01W {} @ 0x{:08X} ({} pins)\n\n",
                        name, base, num_pins));
                    
                    if name.contains("BCM") {
                        output.push_str(&decode_bcm_gpio(base, num_pins));
                    } else if name.contains("Snapdragon") || name.contains("TLMM") {
                        output.push_str(&decode_qcom_gpio(base, num_pins));
                    } else {
                        // Generic PL061 decode
                        output.push_str(&format!("  Data: 0x{:08X}\n", val));
                        if let Some(dir) = safe_read(base + 0x400) {
                            output.push_str(&format!("  Direction: 0x{:08X}\n", dir));
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
        
        // Legacy COM ports
        let com_ports = [
            (0x3F8u64, "COM1"),
            (0x2F8u64, "COM2"),
            (0x3E8u64, "COM3"),
            (0x2E8u64, "COM4"),
        ];
        
        output.push_str("\nLegacy COM ports (I/O space):\n");
        for &(port, name) in &com_ports {
            output.push_str(&format!("  {} @ 0x{:03X}: ", name, port));
            // On real x86, we'd use in/out instructions
            // Here we just document the port addresses
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
        
        // SiFive/QEMU UART
        let uart_base = 0x1000_0000u64;
        if let Some(val) = safe_read(uart_base) {
            output.push_str(&format!("\x01G[FOUND]\x01W UART @ 0x{:08X}\n", uart_base));
        }
    }
    
    output
}
