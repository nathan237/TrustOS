//! Remote Debug Protocol — structured serial output for machine parsing
//!
//! When Copilot / Nathan connects via serial (115200 8N1), this module
//! outputs structured key=value data that can be parsed programmatically.
//! Format: `[HWDBG:SECTION:KEY] value`
//!
//! Also supports interactive commands received over serial:
//!   HWDBG:CMD:AUTO       — Run full diagnostics
//!   HWDBG:CMD:CPU        — Run CPU diagnostics
//!   HWDBG:CMD:MEM:16     — Run 16MB memory test
//!   HWDBG:CMD:PCI        — Run PCI scan
//!   HWDBG:CMD:PING       — Heartbeat check

use alloc::string::String;
use alloc::format;
use alloc::vec::Vec;
use super::dbg_out;

/// Start an interactive remote debug session
pub fn start_remote_session() {
    dbg_out!("[REMOTE] === Remote Debug Session Started ===");
    dbg_out!("[REMOTE] Protocol: HWDBG structured serial (115200 8N1)");
    dbg_out!("[REMOTE] Send commands via serial in format: HWDBG:CMD:<command>");
    dbg_out!("[REMOTE] Available commands:");
    dbg_out!("[REMOTE]   HWDBG:CMD:PING      — Heartbeat");
    dbg_out!("[REMOTE]   HWDBG:CMD:AUTO      — Full diagnostics");
    dbg_out!("[REMOTE]   HWDBG:CMD:CPU       — CPU info");
    dbg_out!("[REMOTE]   HWDBG:CMD:MEM:<mb>  — Memory test");
    dbg_out!("[REMOTE]   HWDBG:CMD:PCI       — PCI scan");
    dbg_out!("[REMOTE]   HWDBG:CMD:ACPI      — ACPI tables");
    dbg_out!("[REMOTE]   HWDBG:CMD:STOR      — Storage info");
    dbg_out!("[REMOTE]   HWDBG:CMD:THERM     — Thermal info");
    dbg_out!("[REMOTE]   HWDBG:CMD:NET       — Network devices");
    dbg_out!("[REMOTE]   HWDBG:CMD:EXPORT    — Machine-parseable full dump");
    dbg_out!("[REMOTE]   HWDBG:CMD:EXIT      — End remote session");
    dbg_out!("[REMOTE]");
    dbg_out!("[REMOTE] Listening for serial input... (type 'exit' to quit)");

    // Send ready beacon
    crate::serial_println!("[HWDBG:STATUS:READY] TrustOS HwDbg Remote v1.0");

    // Simple serial command loop
    let mut line_buf = [0u8; 128];
    let mut line_pos = 0usize;

    loop {
        if let Some(byte) = crate::serial::read_byte() {
            match byte {
                b'\r' | b'\n' => {
                    if line_pos > 0 {
                        let cmd = core::str::from_utf8(&line_buf[..line_pos]).unwrap_or("");
                        if !process_remote_command(cmd) {
                            break; // EXIT command
                        }
                        line_pos = 0;
                    }
                }
                b if line_pos < line_buf.len() => {
                    line_buf[line_pos] = b;
                    line_pos += 1;
                }
                _ => {} // Buffer full, ignore
            }
        }

        // Also check for keyboard input (local exit)
        // Give CPU a break to avoid spinning
        for _ in 0..1000 {
            #[cfg(target_arch = "x86_64")]
            unsafe { core::arch::asm!("pause", options(nostack)); }
            #[cfg(not(target_arch = "x86_64"))]
            unsafe { core::arch::asm!("nop", options(nostack)); }
        }
    }

    crate::serial_println!("[HWDBG:STATUS:CLOSED] Remote session ended");
    dbg_out!("[REMOTE] Session closed.");
}

fn process_remote_command(cmd: &str) -> bool {
    let cmd = cmd.trim();
    crate::serial_println!("[HWDBG:CMD:RECEIVED] {}", cmd);

    if cmd.starts_with("HWDBG:CMD:") {
        let subcmd = &cmd[10..];
        match subcmd {
            "PING" => {
                crate::serial_println!("[HWDBG:STATUS:PONG] alive uptime_ms={}", crate::time::uptime_ms());
            }
            "AUTO" => {
                crate::serial_println!("[HWDBG:STATUS:RUNNING] auto");
                super::handle_hwdbg_command(&["auto"]);
                crate::serial_println!("[HWDBG:STATUS:DONE] auto");
            }
            "CPU" => {
                crate::serial_println!("[HWDBG:STATUS:RUNNING] cpu");
                super::cpu_deep::run(&[]);
                crate::serial_println!("[HWDBG:STATUS:DONE] cpu");
            }
            "PCI" => {
                crate::serial_println!("[HWDBG:STATUS:RUNNING] pci");
                super::pci_deep::run(&[]);
                crate::serial_println!("[HWDBG:STATUS:DONE] pci");
            }
            "ACPI" => {
                crate::serial_println!("[HWDBG:STATUS:RUNNING] acpi");
                super::acpi_dump::run(&[]);
                crate::serial_println!("[HWDBG:STATUS:DONE] acpi");
            }
            "STOR" => {
                crate::serial_println!("[HWDBG:STATUS:RUNNING] storage");
                super::storage::run();
                crate::serial_println!("[HWDBG:STATUS:DONE] storage");
            }
            "THERM" => {
                crate::serial_println!("[HWDBG:STATUS:RUNNING] thermal");
                super::thermal::run();
                crate::serial_println!("[HWDBG:STATUS:DONE] thermal");
            }
            "NET" => {
                crate::serial_println!("[HWDBG:STATUS:RUNNING] net");
                super::net_hw::run();
                crate::serial_println!("[HWDBG:STATUS:DONE] net");
            }
            "EXPORT" => {
                export_full_report();
            }
            "EXIT" | "QUIT" => {
                return false;
            }
            other => {
                if other.starts_with("MEM:") {
                    let mb_str = &other[4..];
                    let mb = mb_str.parse::<usize>().unwrap_or(16);
                    crate::serial_println!("[HWDBG:STATUS:RUNNING] mem {}", mb);
                    super::mem_test::run(mb);
                    crate::serial_println!("[HWDBG:STATUS:DONE] mem");
                } else {
                    crate::serial_println!("[HWDBG:ERROR] Unknown command: {}", other);
                }
            }
        }
    } else if cmd == "exit" || cmd == "quit" {
        return false;
    } else {
        crate::serial_println!("[HWDBG:ERROR] Expected format: HWDBG:CMD:<command>");
    }

    true
}

/// Export full machine report in structured key=value format for parsing
pub fn export_full_report() {
    crate::serial_println!("[HWDBG:REPORT:BEGIN]");
    crate::serial_println!("[HWDBG:META:VERSION] 1.0");
    crate::serial_println!("[HWDBG:META:ARCH] {}", if cfg!(target_arch = "x86_64") { "x86_64" }
        else if cfg!(target_arch = "aarch64") { "aarch64" }
        else if cfg!(target_arch = "riscv64") { "riscv64" }
        else { "unknown" });
    crate::serial_println!("[HWDBG:META:UPTIME_MS] {}", crate::time::uptime_ms());

    // CPU info
    #[cfg(target_arch = "x86_64")]
    export_cpu_info();

    // Memory
    export_mem_info();

    // PCI devices
    export_pci_info();

    // ACPI
    export_acpi_info();

    crate::serial_println!("[HWDBG:REPORT:END]");
}

#[cfg(target_arch = "x86_64")]
fn export_cpu_info() {
    // Vendor
    let r0 = unsafe { core::arch::x86_64::__cpuid(0) };
    let mut vendor = [0u8; 12];
    vendor[0..4].copy_from_slice(&r0.ebx.to_le_bytes());
    vendor[4..8].copy_from_slice(&r0.edx.to_le_bytes());
    vendor[8..12].copy_from_slice(&r0.ecx.to_le_bytes());
    let vendor_str = core::str::from_utf8(&vendor).unwrap_or("?");
    crate::serial_println!("[HWDBG:CPU:VENDOR] {}", vendor_str);

    // Family/model
    let r1 = unsafe { core::arch::x86_64::__cpuid(1) };
    let eax1 = r1.eax;
    let stepping = eax1 & 0xF;
    let model = ((eax1 >> 4) & 0xF) | (((eax1 >> 16) & 0xF) << 4);
    let family = ((eax1 >> 8) & 0xF) + ((eax1 >> 20) & 0xFF);
    crate::serial_println!("[HWDBG:CPU:FAMILY] {}", family);
    crate::serial_println!("[HWDBG:CPU:MODEL] {}", model);
    crate::serial_println!("[HWDBG:CPU:STEPPING] {}", stepping);
}

fn export_mem_info() {
    let free = crate::memory::heap::free();
    crate::serial_println!("[HWDBG:MEM:HEAP_FREE_KB] {}", free / 1024);

    let stats = crate::devtools::memdbg_stats();
    crate::serial_println!("[HWDBG:MEM:LIVE_ALLOCS] {}", stats.live_allocs);
    crate::serial_println!("[HWDBG:MEM:PEAK_HEAP_KB] {}", stats.peak_heap_used / 1024);
}

fn export_pci_info() {
    let devices = crate::pci::scan();
    crate::serial_println!("[HWDBG:PCI:COUNT] {}", devices.len());
    for dev in &devices {
        crate::serial_println!("[HWDBG:PCI:DEV] {:02x}:{:02x}.{} {:04x}:{:04x} class={:02x}{:02x} {}",
            dev.bus, dev.device, dev.function,
            dev.vendor_id, dev.device_id,
            dev.class_code, dev.subclass,
            dev.class_name());
    }
}

fn export_acpi_info() {
    if let Some(info) = crate::acpi::get_info() {
        crate::serial_println!("[HWDBG:ACPI:CPU_COUNT] {}", info.cpu_count);
        crate::serial_println!("[HWDBG:ACPI:IOAPIC_COUNT] {}", info.io_apics.len());
        crate::serial_println!("[HWDBG:ACPI:LAPIC_ADDR] 0x{:08X}", info.local_apic_addr);
    } else {
        crate::serial_println!("[HWDBG:ACPI:STATUS] unavailable");
    }
}
