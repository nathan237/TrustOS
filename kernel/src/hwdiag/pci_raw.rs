//! PCI Raw Config Space Dumper
//!
//! `hwdbg pciraw <bus:dev.fn>` — Hex dump of the full PCI/PCIe config space
//! (256 bytes legacy, 4096 bytes if ECAM available).

use alloc::format;
use super::dbg_out;

/// Run PCI raw config space dump
pub fn run(args: &[&str]) {
    let filter = args.first().copied().unwrap_or("");

    if filter.is_empty() {
        dbg_out!("Usage: hwdbg pciraw <bus:dev.fn>");
        dbg_out!("  Example: hwdbg pciraw 00:02.0");
        dbg_out!("  Dumps full PCI config space (256B or 4KB if PCIe ECAM available)");
        return;
    }

    let (bus, dev, func) = match parse_bdf(filter) {
        Some(bdf) => bdf,
        None => {
            dbg_out!("ERROR: Invalid BDF format '{}'. Use bus:dev.fn (e.g., 00:1f.3)", filter);
            return;
        }
    };

    // Check device exists
    let vendor = crate::pci::config_read16(bus, dev, func, 0x00);
    if vendor == 0xFFFF {
        dbg_out!("ERROR: No device at {:02x}:{:02x}.{}", bus, dev, func);
        return;
    }

    let device_id = crate::pci::config_read16(bus, dev, func, 0x02);
    let class = crate::pci::config_read8(bus, dev, func, 0x0B);
    let subclass = crate::pci::config_read8(bus, dev, func, 0x0A);

    super::section_header("PCI RAW CONFIG SPACE DUMP");
    dbg_out!("Device: {:02x}:{:02x}.{} [{:04x}:{:04x}] class={:02x}{:02x}",
        bus, dev, func, vendor, device_id, class, subclass);
    dbg_out!("");

    // Try ECAM first for full 4KB
    let has_ecam = crate::pci::ecam_config_read32(bus, dev, func, 0).is_some();
    let size = if has_ecam { 4096 } else { 256 };

    dbg_out!("Config space: {} bytes ({})", size, if has_ecam { "PCIe ECAM" } else { "Legacy PIO" });
    dbg_out!("");

    // Header row
    dbg_out!("         00 01 02 03 04 05 06 07 08 09 0A 0B 0C 0D 0E 0F");
    dbg_out!("         ── ── ── ── ── ── ── ── ── ── ── ── ── ── ── ──");

    let mut offset = 0u16;
    while offset < size {
        let mut line = format!("  {:04X}: ", offset);

        for col in 0..16u16 {
            let addr = offset + col;
            let byte = if has_ecam {
                let dword = crate::pci::ecam_config_read32(bus, dev, func, addr & 0xFFC)
                    .unwrap_or(0xFFFFFFFF);
                ((dword >> ((addr & 3) * 8)) & 0xFF) as u8
            } else {
                crate::pci::config_read8(bus, dev, func, addr as u8)
            };
            line.push_str(&format!("{:02X} ", byte));
        }

        // ASCII column
        line.push_str(" |");
        for col in 0..16u16 {
            let addr = offset + col;
            let byte = if has_ecam {
                let dword = crate::pci::ecam_config_read32(bus, dev, func, addr & 0xFFC)
                    .unwrap_or(0xFFFFFFFF);
                ((dword >> ((addr & 3) * 8)) & 0xFF) as u8
            } else {
                crate::pci::config_read8(bus, dev, func, addr as u8)
            };
            if byte >= 0x20 && byte < 0x7F {
                line.push(byte as char);
            } else {
                line.push('.');
            }
        }
        line.push('|');

        dbg_out!("{}", line);
        offset += 16;
    }

    // Decode key registers
    dbg_out!("");
    super::sub_header("KEY REGISTER DECODE");
    decode_header(bus, dev, func);
}

fn decode_header(bus: u8, dev: u8, func: u8) {
    let command = crate::pci::config_read16(bus, dev, func, 0x04);
    let status = crate::pci::config_read16(bus, dev, func, 0x06);
    let header_type = crate::pci::config_read8(bus, dev, func, 0x0E);
    let irq_line = crate::pci::config_read8(bus, dev, func, 0x3C);
    let irq_pin = crate::pci::config_read8(bus, dev, func, 0x3D);

    dbg_out!("  Command:     {:#06X}  [IO={} MEM={} BusMaster={} IntDis={}]",
        command,
        if command & 0x01 != 0 { "ON" } else { "off" },
        if command & 0x02 != 0 { "ON" } else { "off" },
        if command & 0x04 != 0 { "ON" } else { "off" },
        if command & 0x400 != 0 { "YES" } else { "no" });
    dbg_out!("  Status:      {:#06X}  [CapList={} 66MHz={}]",
        status,
        if status & 0x10 != 0 { "YES" } else { "no" },
        if status & 0x20 != 0 { "YES" } else { "no" });
    dbg_out!("  Header type: {:#04X}  ({})", header_type,
        match header_type & 0x7F {
            0 => "Type 0 (endpoint)",
            1 => "Type 1 (PCI-PCI bridge)",
            2 => "Type 2 (CardBus bridge)",
            _ => "unknown",
        });
    dbg_out!("  IRQ:         line={} pin={}", irq_line,
        match irq_pin { 1 => "INTA", 2 => "INTB", 3 => "INTC", 4 => "INTD", _ => "none" });

    // BARs (type 0 only)
    if header_type & 0x7F == 0 {
        for i in 0..6u8 {
            let bar_off = 0x10 + i * 4;
            let bar = crate::pci::config_read(bus, dev, func, bar_off);
            if bar == 0 { continue; }
            let is_io = bar & 1 != 0;
            if is_io {
                dbg_out!("  BAR{}:        I/O  {:#010X}", i, bar & !3);
            } else {
                let is_64 = (bar >> 1) & 3 == 2;
                let prefetch = bar & 0x08 != 0;
                if is_64 && i < 5 {
                    let bar_hi = crate::pci::config_read(bus, dev, func, bar_off + 4);
                    let addr = ((bar_hi as u64) << 32) | (bar as u64 & !0xF);
                    dbg_out!("  BAR{}-{}:      MMIO {:#018X} (64-bit{})", i, i + 1, addr,
                        if prefetch { ", prefetchable" } else { "" });
                } else {
                    dbg_out!("  BAR{}:        MMIO {:#010X} (32-bit{})", i, bar & !0xF,
                        if prefetch { ", prefetchable" } else { "" });
                }
            }
        }
    }

    // Capabilities chain
    if status & 0x10 != 0 {
        dbg_out!("");
        dbg_out!("  Capabilities chain:");
        let mut cap_ptr = crate::pci::config_read8(bus, dev, func, 0x34) & 0xFC;
        let mut count = 0;
        while cap_ptr != 0 && count < 32 {
            let cap_id = crate::pci::config_read8(bus, dev, func, cap_ptr);
            let cap_name = match cap_id {
                0x01 => "Power Management",
                0x03 => "VPD",
                0x05 => "MSI",
                0x07 => "PCI-X",
                0x09 => "Vendor Specific",
                0x0A => "Debug port",
                0x10 => "PCIe",
                0x11 => "MSI-X",
                0x12 => "SATA",
                0x13 => "AF",
                _ => "Unknown",
            };
            dbg_out!("    [{:#04X}] offset={:#04X} id={:#04X} ({})", cap_ptr, cap_ptr, cap_id, cap_name);
            cap_ptr = crate::pci::config_read8(bus, dev, func, cap_ptr + 1) & 0xFC;
            count += 1;
        }
    }
}

pub fn parse_bdf(s: &str) -> Option<(u8, u8, u8)> {
    // Parse "bus:dev.fn" format
    let colon = s.find(':')?;
    let dot = s.find('.')?;
    if dot <= colon { return None; }

    let bus = u8::from_str_radix(&s[..colon], 16).ok()?;
    let dev = u8::from_str_radix(&s[colon + 1..dot], 16).ok()?;
    let func = u8::from_str_radix(&s[dot + 1..], 16).ok()?;

    if dev > 31 || func > 7 { return None; }
    Some((bus, dev, func))
}
