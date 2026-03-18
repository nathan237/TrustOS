//! PCIe Advanced Error Reporting (AER) — Read and decode AER capability
//!
//! `hwdbg aer`              — Scan all devices for AER capability and show errors
//! `hwdbg aer <B:D.F>`      — Show AER for specific device
//! `hwdbg aer clear <B:D.F>` — Clear error registers on a device

use alloc::format;
use super::dbg_out;

// AER Extended Capability ID = 0x0001
const AER_CAP_ID: u16 = 0x0001;

// AER register offsets (relative to AER cap base)
const AER_UNCORRECTABLE_STATUS: u16 = 0x04;
const AER_UNCORRECTABLE_MASK: u16 = 0x08;
const AER_UNCORRECTABLE_SEVERITY: u16 = 0x0C;
const AER_CORRECTABLE_STATUS: u16 = 0x10;
const AER_CORRECTABLE_MASK: u16 = 0x14;
const AER_CAP_CONTROL: u16 = 0x18;
const AER_HEADER_LOG: u16 = 0x1C; // 16 bytes (4 dwords)

/// Dispatcher
pub fn run(args: &[&str]) {
    let subcmd = args.first().copied().unwrap_or("all");

    match subcmd {
        "all" | "scan" => scan_all_aer(),
        "clear" => {
            if let Some(bdf) = args.get(1) {
                if let Some((bus, dev, func)) = super::pci_raw::parse_bdf(bdf) {
                    clear_aer(bus, dev, func);
                } else {
                    dbg_out!("Bad BDF format — use B:D.F");
                }
            } else {
                dbg_out!("Usage: hwdbg aer clear <B:D.F>");
            }
        }
        _ => {
            if let Some((bus, dev, func)) = super::pci_raw::parse_bdf(subcmd) {
                show_device_aer(bus, dev, func);
            } else {
                dbg_out!("Usage:");
                dbg_out!("  hwdbg aer              Scan all devices for AER errors");
                dbg_out!("  hwdbg aer <B:D.F>      Show AER for specific device");
                dbg_out!("  hwdbg aer clear <B:D.F> Clear AER error registers");
            }
        }
    }
}

/// Scan all PCI devices for AER capability
fn scan_all_aer() {
    super::section_header("PCIe AER SCAN");

    let devices = crate::pci::scan();
    let mut aer_count = 0;
    let mut error_count = 0;

    for dev in &devices {
        if let Some(aer_base) = find_aer_cap(dev.bus, dev.device, dev.function) {
            aer_count += 1;
            let uncorr = read_ecam(dev.bus, dev.device, dev.function, aer_base + AER_UNCORRECTABLE_STATUS);
            let corr = read_ecam(dev.bus, dev.device, dev.function, aer_base + AER_CORRECTABLE_STATUS);

            let has_errors = uncorr != 0 || corr != 0;
            if has_errors { error_count += 1; }

            let tag = if has_errors { "ERR" } else { " ok" };
            dbg_out!("  [{}] {:02X}:{:02X}.{}  {:04X}:{:04X}  AER@{:#05X}  uncorr={:#010X} corr={:#010X}",
                tag, dev.bus, dev.device, dev.function,
                dev.vendor_id, dev.device_id, aer_base, uncorr, corr);

            if has_errors {
                if uncorr != 0 {
                    decode_uncorrectable(uncorr, "    ");
                }
                if corr != 0 {
                    decode_correctable(corr, "    ");
                }
            }
        }
    }

    dbg_out!("");
    if aer_count == 0 {
        dbg_out!("No devices with AER capability found (need PCIe ECAM)");
    } else {
        dbg_out!("{} devices with AER, {} reporting errors", aer_count, error_count);
    }
}

/// Show detailed AER info for one device
fn show_device_aer(bus: u8, dev: u8, func: u8) {
    super::section_header("PCIe AER DETAIL");
    dbg_out!("Device: {:02X}:{:02X}.{}", bus, dev, func);
    dbg_out!("");

    let aer_base = match find_aer_cap(bus, dev, func) {
        Some(b) => b,
        None => {
            dbg_out!("No AER capability found on this device.");
            dbg_out!("Check: Is it a PCIe device? Is ECAM available?");
            return;
        }
    };

    dbg_out!("AER Capability at offset {:#05X}", aer_base);
    dbg_out!("");

    let uncorr_status = read_ecam(bus, dev, func, aer_base + AER_UNCORRECTABLE_STATUS);
    let uncorr_mask = read_ecam(bus, dev, func, aer_base + AER_UNCORRECTABLE_MASK);
    let uncorr_sev = read_ecam(bus, dev, func, aer_base + AER_UNCORRECTABLE_SEVERITY);
    let corr_status = read_ecam(bus, dev, func, aer_base + AER_CORRECTABLE_STATUS);
    let corr_mask = read_ecam(bus, dev, func, aer_base + AER_CORRECTABLE_MASK);
    let cap_ctrl = read_ecam(bus, dev, func, aer_base + AER_CAP_CONTROL);

    super::sub_header("Uncorrectable Error Status");
    dbg_out!("  Status:   {:#010X}", uncorr_status);
    dbg_out!("  Mask:     {:#010X}", uncorr_mask);
    dbg_out!("  Severity: {:#010X}", uncorr_sev);
    if uncorr_status != 0 {
        decode_uncorrectable(uncorr_status, "  ");
    }

    dbg_out!("");
    super::sub_header("Correctable Error Status");
    dbg_out!("  Status:   {:#010X}", corr_status);
    dbg_out!("  Mask:     {:#010X}", corr_mask);
    if corr_status != 0 {
        decode_correctable(corr_status, "  ");
    }

    dbg_out!("");
    super::sub_header("AER Capabilities & Control");
    dbg_out!("  Control:  {:#010X}", cap_ctrl);
    let first_err = (cap_ctrl >> 5) & 0x1F;
    dbg_out!("  First Error Pointer: {}", first_err);
    let ecrc_gen_cap = cap_ctrl & (1 << 5) != 0;
    let ecrc_gen_en = cap_ctrl & (1 << 6) != 0;
    let ecrc_chk_cap = cap_ctrl & (1 << 7) != 0;
    let ecrc_chk_en = cap_ctrl & (1 << 8) != 0;
    dbg_out!("  ECRC Gen: cap={} en={}", ecrc_gen_cap, ecrc_gen_en);
    dbg_out!("  ECRC Chk: cap={} en={}", ecrc_chk_cap, ecrc_chk_en);

    // Header log
    dbg_out!("");
    super::sub_header("Header Log");
    for i in 0..4u16 {
        let dw = read_ecam(bus, dev, func, aer_base + AER_HEADER_LOG + i * 4);
        dbg_out!("  DW{}: {:#010X}", i, dw);
    }
}

/// Clear AER error registers
fn clear_aer(bus: u8, dev: u8, func: u8) {
    let aer_base = match find_aer_cap(bus, dev, func) {
        Some(b) => b,
        None => {
            dbg_out!("No AER capability on {:02X}:{:02X}.{}", bus, dev, func);
            return;
        }
    };

    let uncorr = read_ecam(bus, dev, func, aer_base + AER_UNCORRECTABLE_STATUS);
    let corr = read_ecam(bus, dev, func, aer_base + AER_CORRECTABLE_STATUS);

    // Write-1-to-clear (W1C) semantics for AER status registers
    write_ecam(bus, dev, func, aer_base + AER_UNCORRECTABLE_STATUS, uncorr);
    write_ecam(bus, dev, func, aer_base + AER_CORRECTABLE_STATUS, corr);

    dbg_out!("Cleared AER on {:02X}:{:02X}.{}", bus, dev, func);
    dbg_out!("  Uncorrectable was: {:#010X}", uncorr);
    dbg_out!("  Correctable was:   {:#010X}", corr);

    // Verify
    let new_uncorr = read_ecam(bus, dev, func, aer_base + AER_UNCORRECTABLE_STATUS);
    let new_corr = read_ecam(bus, dev, func, aer_base + AER_CORRECTABLE_STATUS);
    dbg_out!("  After clear: uncorr={:#010X} corr={:#010X}", new_uncorr, new_corr);
}

/// Find AER extended capability in PCIe extended config space (0x100+)
fn find_aer_cap(bus: u8, dev: u8, func: u8) -> Option<u16> {
    let mut offset: u16 = 0x100;

    // Walk extended capability list (max 48 iterations for safety)
    for _ in 0..48 {
        let header = read_ecam(bus, dev, func, offset);

        if header == 0 || header == 0xFFFFFFFF {
            return None;
        }

        let cap_id = (header & 0xFFFF) as u16;
        let next = ((header >> 20) & 0xFFC) as u16;

        if cap_id == AER_CAP_ID {
            return Some(offset);
        }

        if next == 0 || next < 0x100 {
            return None;
        }
        offset = next;
    }

    None
}

/// Read from PCIe extended config space via ECAM
fn read_ecam(bus: u8, dev: u8, func: u8, offset: u16) -> u32 {
    crate::pci::ecam_config_read32(bus, dev, func, offset).unwrap_or(0xFFFFFFFF)
}

/// Write to PCIe extended config space via ECAM
fn write_ecam(bus: u8, dev: u8, func: u8, offset: u16, value: u32) {
    if offset < 0x100 {
        crate::pci::config_write(bus, dev, func, offset as u8, value);
    } else {
        if !crate::pci::ecam_config_write32(bus, dev, func, offset, value) {
            dbg_out!("WARN: ECAM write failed for offset {:#06X}", offset);
        }
    }
}

/// Decode uncorrectable error status bits
fn decode_uncorrectable(status: u32, prefix: &str) {
    let bits: &[(u8, &str)] = &[
        (4, "Data Link Protocol Error"),
        (5, "Surprise Down Error"),
        (12, "Poisoned TLP"),
        (13, "Flow Control Protocol Error"),
        (14, "Completion Timeout"),
        (15, "Completer Abort"),
        (16, "Unexpected Completion"),
        (17, "Receiver Overflow"),
        (18, "Malformed TLP"),
        (19, "ECRC Error"),
        (20, "Unsupported Request"),
        (21, "ACS Violation"),
        (22, "Internal Error"),
        (23, "MC Blocked TLP"),
        (24, "AtomicOp Egress Blocked"),
        (25, "TLP Prefix Blocked"),
        (26, "Poisoned TLP Egress Blocked"),
    ];

    for &(bit, name) in bits {
        if status & (1 << bit) != 0 {
            dbg_out!("{}  [bit {}] {}", prefix, bit, name);
        }
    }
}

/// Decode correctable error status bits
fn decode_correctable(status: u32, prefix: &str) {
    let bits: &[(u8, &str)] = &[
        (0, "Receiver Error"),
        (6, "Bad TLP"),
        (7, "Bad DLLP"),
        (8, "REPLAY_NUM Rollover"),
        (12, "Replay Timer Timeout"),
        (13, "Advisory Non-Fatal Error"),
        (14, "Corrected Internal Error"),
        (15, "Header Log Overflow"),
    ];

    for &(bit, name) in bits {
        if status & (1 << bit) != 0 {
            dbg_out!("{}  [bit {}] {}", prefix, bit, name);
        }
    }
}
