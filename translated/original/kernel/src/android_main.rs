//! Android Boot Path — Rust entry point
//!
//! Called from `android_entry.S` after the assembly stub sets up:
//! - Stack pointer
//! - BSS zeroed
//! - Caches enabled
//! - DAIF masked (interrupts off)
//!
//! This module handles the transition from bare-metal Android boot
//! to the standard TrustOS kernel initialization path.
//!
//! The key difference from Limine boot: we have NO bootloader services.
//! Everything must come from the Device Tree Blob (DTB):
//!   - RAM layout → memory regions
//!   - UART address → serial console
//!   - SimpleFB → framebuffer display
//!   - SoC compatible → hardware identification
//!   - Reserved memory → firmware/TrustZone carveouts

use crate::android_boot::{self, FdtHeader, DtbInfo, SocFamily};
use crate::hwscan::dtb_parser;

// ═════════════════════════════════════════════════════════════════════════════
// Statics for Android boot state
// ═════════════════════════════════════════════════════════════════════════════

/// DTB physical address (saved from bootloader handoff)
static mut DTB_PHYS_ADDR: u64 = 0;

/// Whether we booted via Android path (vs Limine)
static mut ANDROID_BOOT: bool = false;

/// Parsed DTB info (legacy)
static mut DTB_INFO: Option<DtbInfo> = None;

// ═════════════════════════════════════════════════════════════════════════════
// PSCI interface (Power State Coordination Interface via SMC to BL31)
// ═════════════════════════════════════════════════════════════════════════════

// SMC Function IDs (PSCI v1.0+)
const PSCI_VERSION: u64 = 0x8400_0000;
const PSCI_SYSTEM_OFF: u64 = 0x8400_0008;
const PSCI_SYSTEM_RESET: u64 = 0x8400_0009;
const PSCI_CPU_ON_64: u64 = 0xC400_0003;
const PSCI_FEATURES: u64 = 0x8400_000A;

extern "C" {
    /// Call BL31 SMC (defined in android_entry.S)
    fn android_smc_call(fid: u64, x1: u64, x2: u64, x3: u64) -> u64;
    /// PSCI shutdown
    fn android_psci_off() -> !;
    /// PSCI reboot
    fn android_psci_reset() -> !;
}

/// Issue an SMC call to ARM Trusted Firmware (BL31)
pub fn smc(fid: u64, x1: u64, x2: u64, x3: u64) -> u64 {
    unsafe { android_smc_call(fid, x1, x2, x3) }
}

/// Get PSCI version from BL31
pub fn psci_version() -> (u32, u32) {
    let v = smc(PSCI_VERSION, 0, 0, 0);
    let major = (v >> 16) as u32;
    let minor = (v & 0xFFFF) as u32;
    (major, minor)
}

/// Power off the system via PSCI
pub fn system_off() -> ! {
    unsafe { android_psci_off() }
}

/// Reboot the system via PSCI
pub fn system_reset() -> ! {
    unsafe { android_psci_reset() }
}

/// Wake up a secondary CPU core via PSCI CPU_ON
pub fn cpu_on(target_cpu: u64, entry_point: u64, context_id: u64) -> i64 {
    smc(PSCI_CPU_ON_64, target_cpu, entry_point, context_id) as i64
}

// ═════════════════════════════════════════════════════════════════════════════
// Early UART (before serial module, before heap)
// ═════════════════════════════════════════════════════════════════════════════

/// Early UART write (before serial module is initialized)
/// Uses the PL011/GENI UART base set by the assembly stub.
unsafe fn early_uart_putc(c: u8) {
    extern "C" {
        static __uart_base: u64;
    }
    let base = __uart_base;
    if base != 0 {
        let ptr = base as *mut u32;
        // Wait for UART TX FIFO not full (PL011: UARTFR bit 5)
        for _ in 0..10000 {
            let fr = ptr.add(6).read_volatile(); // UARTFR offset = 0x18
            if fr & (1 << 5) == 0 {
                break;
            }
        }
        ptr.write_volatile(c as u32);
    }
}

unsafe fn early_uart_print(s: &[u8]) {
    for &c in s {
        early_uart_putc(c);
    }
}

/// Print a u64 as hex via early UART
unsafe fn early_uart_print_hex(val: u64) {
    let hex = b"0123456789ABCDEF";
    for i in (0..16).rev() {
        let nibble = ((val >> (i * 4)) & 0xF) as usize;
        early_uart_putc(hex[nibble]);
    }
}

/// Update the early UART base address (used after DTB parsing discovers the real UART)
unsafe fn update_uart_base(new_base: u64) {
    extern "C" {
        static mut __uart_base: u64;
    }
    if new_base != 0 {
        __uart_base = new_base;
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// Android main entry (called from assembly)
// ═════════════════════════════════════════════════════════════════════════════

/// Main Rust entry point for Android boot path.
///
/// Called from android_entry.S with:
/// - x0 (first arg) = DTB physical address  
/// - Stack set up, BSS zeroed, caches on, interrupts off
///
/// Boot sequence:
/// 1. Validate DTB (magic check)
/// 2. Set up 32MB heap (needed for alloc-based DTB parser)
/// 3. Parse DTB fully — discover RAM, UART, framebuffer, SoC, devices
/// 4. Update UART to real address from DTB
/// 5. Initialize SimpleFB framebuffer from DTB
/// 6. Initialize serial module
/// 7. Boot into standard TrustOS shell
#[no_mangle]
pub unsafe extern "C" fn android_main(dtb_ptr: u64) -> ! {
    // Mark that we booted via Android path
    ANDROID_BOOT = true;
    DTB_PHYS_ADDR = dtb_ptr;

    early_uart_print(b"[ANDROID] TrustOS android_main entered\r\n");

    // ── Step 1: Validate DTB (quick check before heap) ──
    if dtb_ptr != 0 {
        let valid = FdtHeader::validate(dtb_ptr as *const u8);
        if valid {
            early_uart_print(b"[ANDROID] DTB valid at 0x");
            early_uart_print_hex(dtb_ptr);
            early_uart_print(b"\r\n");
            DTB_INFO = DtbInfo::from_dtb_ptr(dtb_ptr as *const u8);
        } else {
            early_uart_print(b"[ANDROID] WARNING: DTB magic invalid!\r\n");
        }
    } else {
        early_uart_print(b"[ANDROID] No DTB provided\r\n");
    }

    // ── Step 2: Query PSCI version from BL31 ──
    let (psci_major, psci_minor) = psci_version();
    early_uart_print(b"[ANDROID] PSCI version: ");
    early_uart_putc(b'0' + psci_major as u8);
    early_uart_putc(b'.');
    early_uart_putc(b'0' + psci_minor as u8);
    early_uart_print(b"\r\n");

    // ── Step 3: Set up early heap (needed for alloc-based DTB parser) ──
    // Conservative: 32MB at a safe offset after kernel
    extern "C" {
        static __kernel_end: u8;
    }
    let kernel_end = &__kernel_end as *const u8 as u64;
    let heap_start = (kernel_end + 0xFFF) & !0xFFF; // Page-align
    let heap_size: usize = 32 * 1024 * 1024; // 32 MB

    early_uart_print(b"[ANDROID] Heap: 32 MB at 0x");
    early_uart_print_hex(heap_start);
    early_uart_print(b"\r\n");

    crate::memory::init_android_heap(heap_start, heap_size);
    early_uart_print(b"[ANDROID] Heap initialized\r\n");

    // ── Step 4: Full DTB parse (now that alloc works) ──
    let mut ram_base: u64 = 0x4000_0000; // fallback
    let mut ram_size: u64 = 128 * 1024 * 1024; // fallback
    let mut uart_from_dtb: u64 = 0;
    let mut has_simplefb = false;

    if dtb_ptr != 0 {
        if let Some(parsed) = dtb_parser::parse_dtb(dtb_ptr as *const u8) {
            early_uart_print(b"[DTB] Parsed ");
            early_uart_print_hex(parsed.node_count as u64);
            early_uart_print(b" nodes, ");
            early_uart_print_hex(parsed.devices.len() as u64);
            early_uart_print(b" devices\r\n");

            // ── Extract RAM info ──
            if let Some(&(base, size)) = parsed.memory.first() {
                ram_base = base;
                ram_size = size;
                early_uart_print(b"[DTB] RAM: 0x");
                early_uart_print_hex(base);
                early_uart_print(b" size 0x");
                early_uart_print_hex(size);
                early_uart_print(b"\r\n");
            }

            // ── Extract UART address ──
            if parsed.uart_base != 0 {
                uart_from_dtb = parsed.uart_base;
                early_uart_print(b"[DTB] UART: 0x");
                early_uart_print_hex(uart_from_dtb);
                early_uart_print(b"\r\n");
            } else {
                // Fallback: look for UART-compatible devices in the device list
                for dev in &parsed.devices {
                    if dev.compatible.contains("pl011")
                        || dev.compatible.contains("uart")
                        || dev.compatible.contains("serial")
                        || dev.compatible.contains("ns16550")
                        || dev.compatible.contains("geni")
                    {
                        if dev.status == "okay" || dev.status == "ok" {
                            uart_from_dtb = dev.reg_base;
                            early_uart_print(b"[DTB] UART found: 0x");
                            early_uart_print_hex(dev.reg_base);
                            early_uart_print(b"\r\n");
                            break;
                        }
                    }
                }
            }

            // ── Update UART base if DTB provided a real one ──
            if uart_from_dtb != 0 {
                update_uart_base(uart_from_dtb);
                early_uart_print(b"[ANDROID] UART switched to DTB address\r\n");
            }

            // ── Detect SoC from compatible strings ──
            let soc = detect_soc_from_compatible(&parsed.compatible);
            android_boot::SOC_INFO = soc;
            early_uart_print(b"[DTB] SoC detected: ");
            match soc {
                SocFamily::QemuVirt => early_uart_print(b"QEMU virt\r\n"),
                SocFamily::Qualcomm => early_uart_print(b"Qualcomm Snapdragon\r\n"),
                SocFamily::Tensor => early_uart_print(b"Google Tensor\r\n"),
                SocFamily::Broadcom => early_uart_print(b"Broadcom (RPi)\r\n"),
                SocFamily::Exynos => early_uart_print(b"Samsung Exynos\r\n"),
                SocFamily::MediaTek => early_uart_print(b"MediaTek\r\n"),
                SocFamily::Unknown => early_uart_print(b"Unknown\r\n"),
            }

            // ── SimpleFB framebuffer ──
            if let Some(ref sfb) = parsed.simplefb {
                early_uart_print(b"[DTB] SimpleFB: 0x");
                early_uart_print_hex(sfb.base);
                early_uart_print(b" ");
                early_uart_print_hex(sfb.width as u64);
                early_uart_print(b"x");
                early_uart_print_hex(sfb.height as u64);
                early_uart_print(b"\r\n");

                // Calculate BPP from format string
                let bpp = match sfb.format.as_str() {
                    "a8r8g8b8" | "x8r8g8b8" | "a8b8g8r8" => 32u16,
                    "r5g6b5" => 16u16,
                    _ => 32u16, // default to 32bpp
                };

                let pitch = if sfb.stride > 0 {
                    sfb.stride as u64
                } else {
                    sfb.width as u64 * (bpp as u64 / 8)
                };

                // Initialize the framebuffer subsystem
                crate::framebuffer::init(
                    sfb.base as *mut u8,
                    sfb.width as u64,
                    sfb.height as u64,
                    pitch,
                    bpp,
                );
                crate::framebuffer::init_scrollback();
                has_simplefb = true;
                early_uart_print(b"[ANDROID] Framebuffer initialized from DTB SimpleFB\r\n");
            } else {
                // No SimpleFB in DTB — try scanning for framebuffer-compatible nodes
                for dev in &parsed.devices {
                    if dev.compatible.contains("simple-framebuffer") {
                        early_uart_print(b"[DTB] Found simple-framebuffer device\r\n");
                        // Use reg as framebuffer base
                        if dev.reg_base != 0 {
                            // Default: 1080x1920 32bpp (common Android phone)
                            let w = 1080u64;
                            let h = 1920u64;
                            let bpp = 32u16;
                            let pitch = w * 4;
                            crate::framebuffer::init(
                                dev.reg_base as *mut u8, w, h, pitch, bpp,
                            );
                            crate::framebuffer::init_scrollback();
                            has_simplefb = true;
                        }
                        break;
                    }
                }
            }

            // ── Reserved memory (TrustZone carveouts) — log them ──
            if !parsed.reserved.is_empty() {
                early_uart_print(b"[DTB] Reserved memory: ");
                early_uart_print_hex(parsed.reserved.len() as u64);
                early_uart_print(b" regions (firmware/TZ)\r\n");
            }
        } else {
            early_uart_print(b"[DTB] Parse failed - using hardcoded defaults\r\n");
        }
    }

    // If no SoC was detected, default to QEMU virt
    if android_boot::SOC_INFO == SocFamily::Unknown {
        android_boot::SOC_INFO = SocFamily::QemuVirt;
    }

    // ── Step 5: Initialize serial module ──
    crate::serial::init();
    crate::serial_println!("[ANDROID] Serial module initialized");
    crate::serial_println!("[ANDROID] TrustOS v0.6.0-Android boot");

    // ── Step 6: Initialize interrupts ──
    crate::interrupts::init();
    crate::serial_println!("[ANDROID] Interrupts initialized");

    // ── Step 7: Enter TrustOS shell ──
    if has_simplefb {
        crate::serial_println!("[ANDROID] SimpleFB active — entering graphical shell");
        crate::println!();
        crate::println_color!(crate::framebuffer::COLOR_BRIGHT_GREEN, 
            "  TrustOS v0.6.0 — Android Boot Mode");
        crate::println_color!(crate::framebuffer::COLOR_GREEN, 
            "  Booted via fastboot | DTB-driven hardware discovery");
        crate::println!();
        crate::println_color!(crate::framebuffer::COLOR_WHITE,
            "  Type 'hwscan dtb' to see discovered hardware");
        crate::println_color!(crate::framebuffer::COLOR_WHITE,
            "  Type 'hwscan auto' for full security reconnaissance");
        crate::println!();
        crate::shell::run();
    } else {
        // No framebuffer — run serial-only shell
        crate::serial_println!("[ANDROID] No framebuffer — entering serial console");
        crate::serial_println!("TrustOS v0.6.0 — Serial Console (Android Boot)");
        crate::serial_println!("Type 'hwscan dtb' to see discovered hardware");
        crate::serial_println!("Type 'hwscan auto' for full security reconnaissance");
        // Even without a framebuffer, we can run hwscan via serial
        // The shell needs FB though, so we enter a serial-only command loop
        serial_shell_loop();
    }
}

/// Detect SoC family from DTB compatible strings
fn detect_soc_from_compatible(compatible: &[alloc::string::String]) -> SocFamily {
    for c in compatible {
        let lower = c.to_ascii_lowercase();
        if lower.contains("qemu") || lower.contains("virt") {
            return SocFamily::QemuVirt;
        }
        if lower.contains("qualcomm") || lower.contains("qcom") || lower.contains("sdm") 
            || lower.contains("sm8") || lower.contains("msm") {
            return SocFamily::Qualcomm;
        }
        if lower.contains("google,gs") || lower.contains("tensor") {
            return SocFamily::Tensor;
        }
        if lower.contains("samsung") || lower.contains("exynos") {
            return SocFamily::Exynos;
        }
        if lower.contains("mediatek") || lower.contains("mt6") || lower.contains("mt8") {
            return SocFamily::MediaTek;
        }
        if lower.contains("brcm") || lower.contains("broadcom") || lower.contains("bcm2") 
            || lower.contains("raspberrypi") {
            return SocFamily::Broadcom;
        }
    }
    SocFamily::Unknown
}

/// Minimal serial-only command loop for when no framebuffer is available
unsafe fn serial_shell_loop() -> ! {
    use alloc::string::String;
    use alloc::vec::Vec;

    crate::serial_println!("\nTrustOS> ");
    let mut cmd_buf = Vec::<u8>::new();

    loop {
        if let Some(byte) = crate::serial::try_read_byte() {
            match byte {
                b'\r' | b'\n' => {
                    crate::serial_println!("");
                    let cmd = String::from_utf8_lossy(&cmd_buf).into_owned();
                    let cmd = cmd.trim();
                    if !cmd.is_empty() {
                        handle_serial_command(cmd);
                    }
                    cmd_buf.clear();
                    crate::serial_print!("TrustOS> ");
                }
                0x7F | 0x08 => {
                    // Backspace
                    if !cmd_buf.is_empty() {
                        cmd_buf.pop();
                        crate::serial_print!("\x08 \x08");
                    }
                }
                _ => {
                    cmd_buf.push(byte);
                    unsafe { early_uart_putc(byte); }
                }
            }
        }
        // Small yield
        core::arch::asm!("yield", options(nomem, nostack));
    }
}

/// Handle a command in serial-only mode
fn handle_serial_command(cmd: &str) {
    let parts: alloc::vec::Vec<&str> = cmd.split_whitespace().collect();
    let command = parts.first().copied().unwrap_or("");
    let args = &parts[1..];

    match command {
        "hwscan" | "trustprobe" | "probe" => {
            let result = crate::hwscan::handle_hwscan_command(args);
            // Strip color codes for serial output
            let clean: alloc::string::String = result.chars()
                .filter(|&c| c != '\x01')
                .collect();
            crate::serial_println!("{}", clean);
        }
        "reboot" => {
            crate::serial_println!("Rebooting via PSCI...");
            system_reset();
        }
        "poweroff" | "halt" => {
            crate::serial_println!("Shutting down via PSCI...");
            system_off();
        }
        "help" => {
            crate::serial_println!("TrustOS Serial Console Commands:");
            crate::serial_println!("  hwscan <cmd>  - Hardware security scanner");
            crate::serial_println!("  reboot        - Reboot via PSCI");
            crate::serial_println!("  poweroff      - Shutdown via PSCI");
            crate::serial_println!("  help          - This message");
        }
        _ => {
            crate::serial_println!("Unknown command: {}", cmd);
            crate::serial_println!("Type 'help' for available commands");
        }
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// Public query API
// ═════════════════════════════════════════════════════════════════════════════

/// Check if we booted via Android bootloader (vs Limine)
pub fn is_android_boot() -> bool {
    unsafe { ANDROID_BOOT }
}

/// Get the DTB physical address (0 if not available)
pub fn dtb_address() -> u64 {
    unsafe { DTB_PHYS_ADDR }
}
