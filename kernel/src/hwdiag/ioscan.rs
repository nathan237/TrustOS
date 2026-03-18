//! I/O Port Scanner — Detect devices responding on x86 I/O ports
//!
//! `hwdbg ioscan [start] [end]` — Scan a range of I/O ports
//! `hwdbg ioscan legacy`        — Scan all known legacy device ports
//! `hwdbg ioscan com`           — Scan serial COM ports specifically

use alloc::format;
use super::dbg_out;

/// Dispatcher
pub fn run(args: &[&str]) {
    let subcmd = args.first().copied().unwrap_or("legacy");

    match subcmd {
        "legacy" | "all" => scan_legacy(),
        "com" | "serial" => scan_com_ports(),
        "ide" | "ata" => scan_ide(),
        "range" => {
            let start = args.get(1).and_then(|s| parse_port(s)).unwrap_or(0x00);
            let end = args.get(2).and_then(|s| parse_port(s)).unwrap_or(0xFF);
            scan_range(start, end);
        }
        _ => {
            // Try to parse as hex range directly: hwdbg ioscan 3f8 400
            if let Some(start) = parse_port(subcmd) {
                let end = args.get(1).and_then(|s| parse_port(s)).unwrap_or(start + 0x10);
                scan_range(start, end);
            } else {
                dbg_out!("Usage:");
                dbg_out!("  hwdbg ioscan legacy           Scan all known legacy I/O ports");
                dbg_out!("  hwdbg ioscan com              Scan COM1-COM8 serial ports");
                dbg_out!("  hwdbg ioscan ide              Scan IDE/ATA controllers");
                dbg_out!("  hwdbg ioscan <start> [end]    Scan hex range (e.g., 3f0 400)");
            }
        }
    }
}

/// Scan all known legacy I/O port ranges
#[cfg(target_arch = "x86_64")]
fn scan_legacy() {
    super::section_header("LEGACY I/O PORT SCAN");

    struct PortRange {
        start: u16,
        end: u16,
        name: &'static str,
    }

    let ranges = [
        PortRange { start: 0x000, end: 0x00F, name: "DMA Controller 1" },
        PortRange { start: 0x020, end: 0x021, name: "PIC 1 (8259A master)" },
        PortRange { start: 0x040, end: 0x043, name: "PIT (8254 timer)" },
        PortRange { start: 0x060, end: 0x064, name: "Keyboard/PS2 (8042)" },
        PortRange { start: 0x070, end: 0x071, name: "CMOS/RTC" },
        PortRange { start: 0x080, end: 0x08F, name: "DMA Page Registers" },
        PortRange { start: 0x0A0, end: 0x0A1, name: "PIC 2 (8259A slave)" },
        PortRange { start: 0x0C0, end: 0x0DF, name: "DMA Controller 2" },
        PortRange { start: 0x170, end: 0x177, name: "IDE Secondary" },
        PortRange { start: 0x1F0, end: 0x1F7, name: "IDE Primary" },
        PortRange { start: 0x278, end: 0x27F, name: "LPT2 (Parallel)" },
        PortRange { start: 0x2E8, end: 0x2EF, name: "COM4" },
        PortRange { start: 0x2F8, end: 0x2FF, name: "COM2" },
        PortRange { start: 0x378, end: 0x37F, name: "LPT1 (Parallel)" },
        PortRange { start: 0x3B0, end: 0x3BB, name: "VGA (MDA compat)" },
        PortRange { start: 0x3C0, end: 0x3DF, name: "VGA Registers" },
        PortRange { start: 0x3E8, end: 0x3EF, name: "COM3" },
        PortRange { start: 0x3F0, end: 0x3F7, name: "Floppy Controller" },
        PortRange { start: 0x3F8, end: 0x3FF, name: "COM1" },
        PortRange { start: 0x4D0, end: 0x4D1, name: "ELCR (Edge/Level)" },
        PortRange { start: 0xCF8, end: 0xCFF, name: "PCI Config Space" },
    ];

    let mut alive_count = 0;

    for range in &ranges {
        let mut has_data = false;
        let mut values = alloc::vec::Vec::new();

        for port in range.start..=range.end {
            let val = crate::debug::inb(port);
            values.push(val);
            if val != 0xFF && val != 0x00 {
                has_data = true;
            }
        }

        let status = if has_data { "LIVE" } else { "dead" };
        if has_data { alive_count += 1; }

        let color_tag = if has_data { "+" } else { " " };

        let hex_vals: alloc::string::String = values.iter()
            .map(|v| format!("{:02X}", v))
            .collect::<alloc::vec::Vec<_>>()
            .join(" ");

        dbg_out!("{} {:#06X}-{:#06X}  {:4}  {:<24} [{}]",
            color_tag, range.start, range.end, status, range.name, hex_vals);
    }

    dbg_out!("");
    dbg_out!("{} / {} port ranges show activity", alive_count, ranges.len());
}

#[cfg(not(target_arch = "x86_64"))]
fn scan_legacy() {
    dbg_out!("I/O port scanning is x86_64 only");
}

/// Scan COM serial ports and detect UART type
#[cfg(target_arch = "x86_64")]
fn scan_com_ports() {
    super::section_header("SERIAL PORT SCAN");

    struct ComPort {
        base: u16,
        name: &'static str,
    }

    let ports = [
        ComPort { base: 0x3F8, name: "COM1" },
        ComPort { base: 0x2F8, name: "COM2" },
        ComPort { base: 0x3E8, name: "COM3" },
        ComPort { base: 0x2E8, name: "COM4" },
        ComPort { base: 0x4E8, name: "COM5" },
        ComPort { base: 0x5E8, name: "COM6" },
        ComPort { base: 0x5F8, name: "COM7" },
        ComPort { base: 0x4F8, name: "COM8" },
    ];

    for port in &ports {
        let iir = crate::debug::inb(port.base + 2); // Interrupt ID Register
        let lcr = crate::debug::inb(port.base + 3); // Line Control Register
        let mcr = crate::debug::inb(port.base + 4); // Modem Control Register
        let lsr = crate::debug::inb(port.base + 5); // Line Status Register

        if iir == 0xFF {
            dbg_out!("  {} ({:#06X}): NOT PRESENT", port.name, port.base);
            continue;
        }

        // Detect UART type via scratch register + FIFO
        let uart_type = detect_uart_type(port.base);

        let data_ready = lsr & 0x01 != 0;
        let tx_empty = lsr & 0x20 != 0;
        let bits = match lcr & 0x03 { 0 => 5, 1 => 6, 2 => 7, _ => 8 };
        let stop = if lcr & 0x04 != 0 { 2 } else { 1 };
        let parity = match (lcr >> 3) & 0x07 {
            0 => "none",
            1 => "odd",
            3 => "even",
            5 => "mark",
            7 => "space",
            _ => "?",
        };
        let loopback = mcr & 0x10 != 0;

        dbg_out!("  {} ({:#06X}): {} — {}{}{}  DR={} TxE={} loopback={}",
            port.name, port.base, uart_type,
            bits, if stop == 2 { "S2" } else { "S1" }, 
            match parity { "none" => "N", "odd" => "O", "even" => "E", _ => "?" },
            data_ready, tx_empty, loopback);
    }
}

#[cfg(not(target_arch = "x86_64"))]
fn scan_com_ports() {
    dbg_out!("COM port scanning is x86_64 only");
}

/// Detect UART chip type
#[cfg(target_arch = "x86_64")]
fn detect_uart_type(base: u16) -> &'static str {
    // Save and test scratch register
    let old_scratch = crate::debug::inb(base + 7);
    crate::debug::outb(base + 7, 0xA5);
    let test = crate::debug::inb(base + 7);
    crate::debug::outb(base + 7, old_scratch);

    if test != 0xA5 {
        return "8250 (no scratch)";
    }

    // Test FIFO
    let old_fcr = crate::debug::inb(base + 2);
    crate::debug::outb(base + 2, 0xE7); // Enable FIFO, 14-byte trigger
    let iir = crate::debug::inb(base + 2);
    crate::debug::outb(base + 2, old_fcr);

    match (iir >> 6) & 0x03 {
        0 => "16450",
        1 => "16550 (broken FIFO)",
        2 => "16550A/compatible",
        3 => "16750 (64-byte FIFO)",
        _ => "unknown",
    }
}

/// Scan IDE/ATA controllers
#[cfg(target_arch = "x86_64")]
fn scan_ide() {
    super::section_header("IDE/ATA PORT SCAN");

    struct IdeChannel {
        base: u16,
        ctrl: u16,
        name: &'static str,
    }

    let channels = [
        IdeChannel { base: 0x1F0, ctrl: 0x3F6, name: "Primary" },
        IdeChannel { base: 0x170, ctrl: 0x376, name: "Secondary" },
    ];

    for ch in &channels {
        let status = crate::debug::inb(ch.base + 7);
        let alt_status = crate::debug::inb(ch.ctrl);

        if status == 0xFF && alt_status == 0xFF {
            dbg_out!("  {} ({:#06X}): NOT PRESENT", ch.name, ch.base);
            continue;
        }

        let bsy = status & 0x80 != 0;
        let drdy = status & 0x40 != 0;
        let drq = status & 0x08 != 0;
        let err = status & 0x01 != 0;

        dbg_out!("  {} ({:#06X}): status={:#04X} [BSY={} DRDY={} DRQ={} ERR={}]",
            ch.name, ch.base, status, bsy, drdy, drq, err);

        // Try to detect master/slave
        for drive in 0..2u8 {
            crate::debug::outb(ch.base + 6, 0xA0 | (drive << 4)); // Select drive
            // Small delay
            for _ in 0..15 { let _ = crate::debug::inb(ch.ctrl); }

            let s = crate::debug::inb(ch.base + 7);
            let cl = crate::debug::inb(ch.base + 4);
            let ch_val = crate::debug::inb(ch.base + 5);

            let dev_type = match (cl, ch_val) {
                (0x14, 0xEB) => "ATAPI",
                (0x69, 0x96) => "SATAPI",
                (0x00, 0x00) if s != 0xFF => "ATA/PATA",
                (0x3C, 0xC3) => "SATA",
                _ if s == 0xFF => continue,
                _ => "Unknown",
            };

            dbg_out!("    {} {}: type={} sig={:02X}{:02X} status={:#04X}",
                ch.name, if drive == 0 { "Master" } else { "Slave" },
                dev_type, ch_val, cl, s);
        }
    }
}

#[cfg(not(target_arch = "x86_64"))]
fn scan_ide() {
    dbg_out!("IDE scanning is x86_64 only");
}

/// Scan arbitrary I/O port range
#[cfg(target_arch = "x86_64")]
fn scan_range(start: u16, end: u16) {
    if end < start {
        dbg_out!("ERROR: end ({:#06X}) < start ({:#06X})", end, start);
        return;
    }
    let count = (end as u32 - start as u32 + 1).min(256) as u16;
    let actual_end = start + count - 1;

    super::section_header("I/O PORT RANGE SCAN");
    dbg_out!("Range: {:#06X} - {:#06X} ({} ports)", start, actual_end, count);
    dbg_out!("");

    // Group by lines of 16
    let aligned_start = start & !0xF;

    let mut port = aligned_start;
    while port <= actual_end {
        let mut line = format!("  {:#06X}: ", port);
        let mut has_data = false;

        for col in 0..16u16 {
            let addr = port + col;
            if addr >= start && addr <= actual_end {
                let val = crate::debug::inb(addr);
                line.push_str(&format!("{:02X} ", val));
                if val != 0xFF && val != 0x00 {
                    has_data = true;
                }
            } else {
                line.push_str("   ");
            }
        }

        if has_data {
            line.push_str(" <<<");
        }
        dbg_out!("{}", line);

        port = match port.checked_add(16) {
            Some(p) => p,
            None => break,
        };
    }
}

#[cfg(not(target_arch = "x86_64"))]
fn scan_range(_start: u16, _end: u16) {
    dbg_out!("I/O port scanning is x86_64 only");
}

fn parse_port(s: &str) -> Option<u16> {
    let s = s.strip_prefix("0x").unwrap_or(s);
    u16::from_str_radix(s, 16).ok()
}
