//! Live Register Monitor — Poll a register and display changes in real-time
//!
//! `hwdbg regwatch pci <bus>:<dev>.<fn> <offset> [count]`  — Watch PCI config register
//! `hwdbg regwatch msr <msr_hex> [count]`                  — Watch MSR
//! `hwdbg regwatch io <port_hex> [count]`                   — Watch I/O port
//!
//! count = number of polls (default: 50). Polls at ~100ms intervals.

use alloc::format;
use super::dbg_out;

/// Dispatcher
pub fn run(args: &[&str]) {
    let subcmd = args.first().copied().unwrap_or("help");

    match subcmd {
        "pci" => watch_pci(&args[1..]),
        "msr" => watch_msr(&args[1..]),
        "io" | "port" => watch_io(&args[1..]),
        _ => {
            dbg_out!("Usage:");
            dbg_out!("  hwdbg regwatch pci <B:D.F> <offset> [count]");
            dbg_out!("  hwdbg regwatch msr <msr_hex> [count]");
            dbg_out!("  hwdbg regwatch io <port_hex> [count]");
            dbg_out!("");
            dbg_out!("Polls register at ~100ms intervals, shows changes.");
            dbg_out!("count = number of polls (default: 50)");
        }
    }
}

/// Watch a PCI config space register
#[cfg(target_arch = "x86_64")]
fn watch_pci(args: &[&str]) {
    let bdf_str = match args.first() {
        Some(s) => s,
        None => { dbg_out!("Need BDF (e.g., 0:2.0)"); return; }
    };

    let offset_str = match args.get(1) {
        Some(s) => s,
        None => { dbg_out!("Need register offset"); return; }
    };

    let (bus, dev, func) = match super::pci_raw::parse_bdf(bdf_str) {
        Some(b) => b,
        None => { dbg_out!("Bad BDF format — use B:D.F"); return; }
    };

    let offset = match parse_hex_u16(offset_str) {
        Some(o) => o & !0x3, // Align to dword
        None => { dbg_out!("Bad offset"); return; }
    };

    let count: u32 = args.get(2)
        .and_then(|s| s.parse().ok())
        .unwrap_or(50);

    super::section_header("PCI REGISTER WATCH");
    dbg_out!("Target: {:02X}:{:02X}.{} offset {:#06X} — {} polls", bus, dev, func, offset, count);
    dbg_out!("");

    let mut prev = pci_read32(bus, dev, func, offset);
    dbg_out!("  [  0] {:#010X} (initial)", prev);

    let mut changes = 0u32;
    for i in 1..=count {
        busy_wait_ms(100);
        let cur = pci_read32(bus, dev, func, offset);
        if cur != prev {
            let diff = cur ^ prev;
            dbg_out!("  [{:3}] {:#010X}  delta={:#010X}  bits_changed={}",
                i, cur, diff, diff.count_ones());
            changes += 1;
            prev = cur;
        }
    }

    dbg_out!("");
    dbg_out!("{} changes over {} polls", changes, count);
}

#[cfg(not(target_arch = "x86_64"))]
fn watch_pci(_args: &[&str]) {
    dbg_out!("PCI register watch is x86_64 only");
}

/// Watch an MSR
#[cfg(target_arch = "x86_64")]
fn watch_msr(args: &[&str]) {
    let msr_str = match args.first() {
        Some(s) => s,
        None => { dbg_out!("Need MSR address (hex)"); return; }
    };

    let msr = match parse_hex_u32(msr_str) {
        Some(m) => m,
        None => { dbg_out!("Bad MSR address"); return; }
    };

    let count: u32 = args.get(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(50);

    super::section_header("MSR WATCH");
    dbg_out!("MSR: {:#010X}  {} — {} polls", msr, msr_name(msr), count);
    dbg_out!("");

    let mut prev = match crate::debug::read_msr_safe(msr) {
        Some(v) => v,
        None => { dbg_out!("ERROR: MSR {:#010X} not readable (GP fault)", msr); return; }
    };
    dbg_out!("  [  0] {:#018X} (initial)", prev);

    let mut changes = 0u32;
    for i in 1..=count {
        busy_wait_ms(100);
        let cur = match crate::debug::read_msr_safe(msr) {
            Some(v) => v,
            None => { dbg_out!("  [{:3}] READ FAULT — stopping", i); break; }
        };
        if cur != prev {
            let diff = cur ^ prev;
            dbg_out!("  [{:3}] {:#018X}  delta={:#018X}  bits={}",
                i, cur, diff, diff.count_ones());
            changes += 1;
            prev = cur;
        }
    }

    dbg_out!("");
    dbg_out!("{} changes over {} polls", changes, count);
}

#[cfg(not(target_arch = "x86_64"))]
fn watch_msr(_args: &[&str]) {
    dbg_out!("MSR watch is x86_64 only");
}

/// Watch an I/O port
#[cfg(target_arch = "x86_64")]
fn watch_io(args: &[&str]) {
    let port_str = match args.first() {
        Some(s) => s,
        None => { dbg_out!("Need I/O port address (hex)"); return; }
    };

    let port = match parse_hex_u16(port_str) {
        Some(p) => p,
        None => { dbg_out!("Bad port address"); return; }
    };

    let count: u32 = args.get(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(50);

    super::section_header("I/O PORT WATCH");
    dbg_out!("Port: {:#06X} — {} polls", port, count);
    dbg_out!("");

    let mut prev = crate::debug::inb(port);
    dbg_out!("  [  0] {:#04X} (initial)", prev);

    let mut changes = 0u32;
    for i in 1..=count {
        busy_wait_ms(100);
        let cur = crate::debug::inb(port);
        if cur != prev {
            dbg_out!("  [{:3}] {:#04X}  was={:#04X}  delta={:#04X}",
                i, cur, prev, cur ^ prev);
            changes += 1;
            prev = cur;
        }
    }

    dbg_out!("");
    dbg_out!("{} changes over {} polls", changes, count);
}

#[cfg(not(target_arch = "x86_64"))]
fn watch_io(_args: &[&str]) {
    dbg_out!("I/O port watch is x86_64 only");
}

/// Simple busy-wait loop (approximate milliseconds via TSC)
fn busy_wait_ms(ms: u64) {
    let start = crate::time::uptime_ms();
    while crate::time::uptime_ms() < start + ms {
        core::hint::spin_loop();
    }
}

/// Read PCI config dword supporting both legacy (offset<256) and ECAM (offset>=256)
fn pci_read32(bus: u8, dev: u8, func: u8, offset: u16) -> u32 {
    if offset < 0x100 {
        crate::pci::config_read(bus, dev, func, offset as u8)
    } else {
        crate::pci::ecam_config_read32(bus, dev, func, offset).unwrap_or(0xFFFFFFFF)
    }
}

fn msr_name(msr: u32) -> &'static str {
    match msr {
        0x10 => "TSC",
        0x1B => "APIC_BASE",
        0xC0000080 => "EFER",
        0xC0000081 => "STAR",
        0xC0000082 => "LSTAR",
        0xC0000084 => "SFMASK",
        0xC0000100 => "FS_BASE",
        0xC0000101 => "GS_BASE",
        0xC0000102 => "KERNEL_GS_BASE",
        0x174 => "SYSENTER_CS",
        0x175 => "SYSENTER_ESP",
        0x176 => "SYSENTER_EIP",
        0x1A0 => "MISC_ENABLE",
        0x19A => "CLOCK_MODULATION",
        0x198 => "PERF_STATUS",
        0x199 => "PERF_CTL",
        0xE2 => "POWER_CTL",
        _ => "(unknown)",
    }
}

fn parse_hex_u16(s: &str) -> Option<u16> {
    let s = s.strip_prefix("0x").unwrap_or(s);
    u16::from_str_radix(s, 16).ok()
}

fn parse_hex_u32(s: &str) -> Option<u32> {
    let s = s.strip_prefix("0x").unwrap_or(s);
    u32::from_str_radix(s, 16).ok()
}
