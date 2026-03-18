//! Deep PCI Diagnostics — full PCI/PCIe tree with BARs, capabilities, power state
//!
//! Enumerates all PCI devices, dumps config space headers, decodes capabilities
//! (MSI, MSI-X, PCIe link, power management), and identifies unknown devices.

use alloc::string::String;
use alloc::format;
use alloc::vec::Vec;
use super::dbg_out;

/// Run PCI deep diagnostics
pub fn run(args: &[&str]) {
    dbg_out!("[PCI] === Deep PCI/PCIe Analysis ===");

    // Check if a specific device was requested
    let filter = args.first().copied();

    let devices = crate::pci::scan();
    if devices.is_empty() {
        dbg_out!("[PCI] No PCI devices found!");
        dbg_out!("[PCI] Check: PCI config mechanism (I/O ports 0xCF8/0xCFC) may not be available.");
        return;
    }

    dbg_out!("[PCI] Found {} devices", devices.len());
    dbg_out!("[PCI] ECam (PCIe MMIO config): {}", if crate::pci::ecam_available() { "available" } else { "not available (legacy I/O only)" });
    dbg_out!("");

    for dev in &devices {
        // Filter if specific device requested
        if let Some(f) = filter {
            let dev_str = alloc::format!("{:02x}:{:02x}.{}", dev.bus, dev.device, dev.function);
            if !dev_str.starts_with(f) && !f.contains("all") {
                continue;
            }
        }

        dump_device(dev);
    }

    // Summary by class
    dbg_out!("");
    dbg_out!("[PCI] === Device Summary by Class ===");
    dump_class_summary(&devices);
}

fn dump_device(dev: &crate::pci::PciDevice) {
    let bdf = alloc::format!("{:02x}:{:02x}.{}", dev.bus, dev.device, dev.function);
    dbg_out!("[PCI] ┌─ {} [{:04x}:{:04x}] ─────────────────────────────",
        bdf, dev.vendor_id, dev.device_id);
    dbg_out!("[PCI] │ Vendor:   {} (0x{:04X})", dev.vendor_name(), dev.vendor_id);
    dbg_out!("[PCI] │ Class:    {} / {} (0x{:02X}{:02X})",
        dev.class_name(), dev.subclass_name(), dev.class_code, dev.subclass);
    dbg_out!("[PCI] │ Rev:      0x{:02X}  ProgIF: 0x{:02X}",
        dev.revision, dev.prog_if);
    dbg_out!("[PCI] │ Header:   type {} {}",
        dev.header_type & 0x7F,
        if dev.is_multifunction() { "(multifunction)" } else { "" });

    // IRQ
    let irq_line = crate::pci::config_read8(dev.bus, dev.device, dev.function, 0x3C);
    let irq_pin = crate::pci::config_read8(dev.bus, dev.device, dev.function, 0x3D);
    if irq_pin > 0 {
        dbg_out!("[PCI] │ IRQ:      line={} pin=INT{}", irq_line, (b'A' + irq_pin - 1) as char);
    }

    // Status register
    let status = crate::pci::config_read16(dev.bus, dev.device, dev.function, 0x06);
    let command = crate::pci::config_read16(dev.bus, dev.device, dev.function, 0x04);
    dbg_out!("[PCI] │ Command:  0x{:04X} (IO={} MEM={} BusMaster={} IntDis={})",
        command,
        command & 1, (command >> 1) & 1, (command >> 2) & 1, (command >> 10) & 1);
    dbg_out!("[PCI] │ Status:   0x{:04X} (CapList={} 66MHz={} FastB2B={})",
        status, (status >> 4) & 1, (status >> 5) & 1, (status >> 7) & 1);

    // BARs
    dump_bars(dev);

    // Capabilities chain
    if status & (1 << 4) != 0 { // Capabilities bit set
        dump_capabilities(dev);
    }

    dbg_out!("[PCI] └────────────────────────────────────────────────────────");
    dbg_out!("");
}

fn dump_bars(dev: &crate::pci::PciDevice) {
    let max_bars = if (dev.header_type & 0x7F) == 0 { 6 } else { 2 };
    let mut bar_idx = 0;
    while bar_idx < max_bars {
        let offset = 0x10 + (bar_idx * 4) as u8;
        let bar_val = crate::pci::config_read(dev.bus, dev.device, dev.function, offset);

        if bar_val == 0 {
            bar_idx += 1;
            continue;
        }

        let is_io = bar_val & 1 != 0;
        if is_io {
            let addr = bar_val & 0xFFFF_FFFC;
            // Probe size
            crate::pci::config_write(dev.bus, dev.device, dev.function, offset, 0xFFFF_FFFF);
            let size_mask = crate::pci::config_read(dev.bus, dev.device, dev.function, offset);
            crate::pci::config_write(dev.bus, dev.device, dev.function, offset, bar_val);
            let size = !(size_mask & 0xFFFF_FFFC).wrapping_add(1);

            dbg_out!("[PCI] │ BAR{}: I/O  0x{:08X} (size: {} bytes)", bar_idx, addr, size & 0xFFFF);
            bar_idx += 1;
        } else {
            let bar_type = (bar_val >> 1) & 0x3;
            let prefetchable = bar_val & (1 << 3) != 0;
            let addr_lo = bar_val & 0xFFFF_FFF0;

            if bar_type == 2 && bar_idx + 1 < max_bars {
                // 64-bit BAR
                let offset_hi = 0x10 + ((bar_idx + 1) * 4) as u8;
                let bar_hi = crate::pci::config_read(dev.bus, dev.device, dev.function, offset_hi);
                let full_addr = ((bar_hi as u64) << 32) | (addr_lo as u64);

                // Probe size (save both BARs, write 0xFFFFFFFF, read back)
                crate::pci::config_write(dev.bus, dev.device, dev.function, offset, 0xFFFF_FFFF);
                crate::pci::config_write(dev.bus, dev.device, dev.function, offset_hi, 0xFFFF_FFFF);
                let size_lo = crate::pci::config_read(dev.bus, dev.device, dev.function, offset);
                let size_hi = crate::pci::config_read(dev.bus, dev.device, dev.function, offset_hi);
                crate::pci::config_write(dev.bus, dev.device, dev.function, offset, bar_val);
                crate::pci::config_write(dev.bus, dev.device, dev.function, offset_hi, bar_hi);

                let size_mask = ((size_hi as u64) << 32) | (size_lo as u64 & 0xFFFF_FFF0);
                let size = (!size_mask).wrapping_add(1);

                dbg_out!("[PCI] │ BAR{}-{}: MEM64 0x{:016X} (size: {} {}) {}",
                    bar_idx, bar_idx + 1, full_addr,
                    format_size(size),
                    if size > 1024*1024 { "MB" } else if size > 1024 { "KB" } else { "B" },
                    if prefetchable { "prefetchable" } else { "" });

                bar_idx += 2;
            } else {
                // 32-bit BAR
                crate::pci::config_write(dev.bus, dev.device, dev.function, offset, 0xFFFF_FFFF);
                let size_mask = crate::pci::config_read(dev.bus, dev.device, dev.function, offset);
                crate::pci::config_write(dev.bus, dev.device, dev.function, offset, bar_val);
                let size = (!(size_mask & 0xFFFF_FFF0)).wrapping_add(1);

                dbg_out!("[PCI] │ BAR{}: MEM32 0x{:08X} (size: {} {}) {}",
                    bar_idx, addr_lo,
                    format_size(size as u64),
                    if size > 1024*1024 { "MB" } else if size > 1024 { "KB" } else { "B" },
                    if prefetchable { "prefetchable" } else { "" });
                bar_idx += 1;
            }
        }
    }
}

fn format_size(bytes: u64) -> u64 {
    if bytes >= 1024 * 1024 {
        bytes / (1024 * 1024)
    } else if bytes >= 1024 {
        bytes / 1024
    } else {
        bytes
    }
}

fn dump_capabilities(dev: &crate::pci::PciDevice) {
    let mut cap_ptr = crate::pci::config_read8(dev.bus, dev.device, dev.function, 0x34) & 0xFC;
    let mut count = 0;

    while cap_ptr != 0 && count < 32 {
        let cap_id = crate::pci::config_read8(dev.bus, dev.device, dev.function, cap_ptr);
        let next = crate::pci::config_read8(dev.bus, dev.device, dev.function, cap_ptr + 1) & 0xFC;

        let cap_name = match cap_id {
            0x01 => "Power Management",
            0x02 => "AGP",
            0x03 => "VPD",
            0x04 => "Slot Numbering",
            0x05 => "MSI",
            0x06 => "CompactPCI Hot Swap",
            0x07 => "PCI-X",
            0x08 => "HyperTransport",
            0x09 => "Vendor Specific",
            0x0A => "Debug Port",
            0x0B => "CompactPCI Central Resource",
            0x0D => "PCI Bridge Subsystem Vendor",
            0x0E => "AGP 8x",
            0x0F => "Secure Device",
            0x10 => "PCI Express",
            0x11 => "MSI-X",
            0x12 => "SATA Config",
            0x13 => "AF (Advanced Features)",
            _ => "Unknown",
        };

        dbg_out!("[PCI] │ Cap @0x{:02X}: 0x{:02X} {}", cap_ptr, cap_id, cap_name);

        // Decode specific capabilities
        match cap_id {
            0x01 => decode_power_mgmt(dev, cap_ptr),
            0x05 => decode_msi(dev, cap_ptr),
            0x10 => decode_pcie(dev, cap_ptr),
            0x11 => decode_msix(dev, cap_ptr),
            _ => {}
        }

        cap_ptr = next;
        count += 1;
    }
}

fn decode_power_mgmt(dev: &crate::pci::PciDevice, offset: u8) {
    let pmcsr = crate::pci::config_read16(dev.bus, dev.device, dev.function, offset + 4);
    let power_state = pmcsr & 0x3;
    let state_str = match power_state {
        0 => "D0 (fully on)",
        1 => "D1 (light sleep)",
        2 => "D2 (deeper sleep)",
        3 => "D3 (off)",
        _ => "?",
    };
    dbg_out!("[PCI] │         Power state: {} (PMCSR=0x{:04X})", state_str, pmcsr);
}

fn decode_msi(dev: &crate::pci::PciDevice, offset: u8) {
    let msg_ctrl = crate::pci::config_read16(dev.bus, dev.device, dev.function, offset + 2);
    let enabled = msg_ctrl & 1 != 0;
    let multi_msg_cap = 1 << ((msg_ctrl >> 1) & 0x7);
    let multi_msg_en = 1 << ((msg_ctrl >> 4) & 0x7);
    let is_64bit = msg_ctrl & (1 << 7) != 0;
    dbg_out!("[PCI] │         MSI: {} vectors={}/{} 64bit={}",
        if enabled { "ENABLED" } else { "disabled" }, multi_msg_en, multi_msg_cap, is_64bit);
}

fn decode_pcie(dev: &crate::pci::PciDevice, offset: u8) {
    let pcie_caps = crate::pci::config_read16(dev.bus, dev.device, dev.function, offset + 2);
    let dev_type = (pcie_caps >> 4) & 0xF;
    let version = pcie_caps & 0xF;

    let type_str = match dev_type {
        0 => "Endpoint",
        1 => "Legacy Endpoint",
        4 => "Root Port",
        5 => "Upstream Switch Port",
        6 => "Downstream Switch Port",
        7 => "PCIe-to-PCI Bridge",
        8 => "PCI-to-PCIe Bridge",
        9 => "Root Complex Integrated EP",
        10 => "Root Complex Event Collector",
        _ => "Unknown",
    };

    dbg_out!("[PCI] │         PCIe: {} v{}", type_str, version);

    // Link status (offset + 0x12)
    let link_status = crate::pci::config_read16(dev.bus, dev.device, dev.function, offset + 0x12);
    let link_speed = link_status & 0xF;
    let link_width = (link_status >> 4) & 0x3F;

    let speed_str = match link_speed {
        1 => "2.5 GT/s (Gen1)",
        2 => "5.0 GT/s (Gen2)",
        3 => "8.0 GT/s (Gen3)",
        4 => "16.0 GT/s (Gen4)",
        5 => "32.0 GT/s (Gen5)",
        6 => "64.0 GT/s (Gen6)",
        _ => "unknown",
    };

    // Link capabilities (offset + 0x0C)
    let link_caps = crate::pci::config_read(dev.bus, dev.device, dev.function, offset + 0x0C);
    let max_speed = link_caps & 0xF;
    let max_width = (link_caps >> 4) & 0x3F;

    let max_speed_str = match max_speed {
        1 => "Gen1", 2 => "Gen2", 3 => "Gen3", 4 => "Gen4", 5 => "Gen5", 6 => "Gen6", _ => "?",
    };

    if link_width > 0 {
        dbg_out!("[PCI] │         Link: x{} {} (max: x{} {})",
            link_width, speed_str, max_width, max_speed_str);
    }
}

fn decode_msix(dev: &crate::pci::PciDevice, offset: u8) {
    let msg_ctrl = crate::pci::config_read16(dev.bus, dev.device, dev.function, offset + 2);
    let table_size = (msg_ctrl & 0x7FF) + 1;
    let enabled = msg_ctrl & (1 << 15) != 0;
    let masked = msg_ctrl & (1 << 14) != 0;
    dbg_out!("[PCI] │         MSI-X: {} vectors={} masked={}",
        if enabled { "ENABLED" } else { "disabled" }, table_size, masked);
}

fn dump_class_summary(devices: &[crate::pci::PciDevice]) {
    let mut class_counts = [(0u8, 0u32); 32]; // (class_code, count) — enough for all PCI classes
    let mut num_classes = 0usize;

    for dev in devices {
        let mut found = false;
        for i in 0..num_classes {
            if class_counts[i].0 == dev.class_code {
                class_counts[i].1 += 1;
                found = true;
                break;
            }
        }
        if !found && num_classes < 32 {
            class_counts[num_classes] = (dev.class_code, 1);
            num_classes += 1;
        }
    }

    for i in 0..num_classes {
        let (class, count) = class_counts[i];
        let name = match class {
            0x00 => "Unclassified",
            0x01 => "Storage Controller",
            0x02 => "Network Controller",
            0x03 => "Display Controller",
            0x04 => "Multimedia Controller",
            0x05 => "Memory Controller",
            0x06 => "Bridge",
            0x07 => "Communication Controller",
            0x08 => "System Peripheral",
            0x09 => "Input Device",
            0x0C => "Serial Bus (USB/FireWire)",
            0x0D => "Wireless Controller",
            0xFF => "Unassigned",
            _ => "Other",
        };
        dbg_out!("[PCI]   {} : {} device{}", name, count, if count > 1 { "s" } else { "" });
    }
}
