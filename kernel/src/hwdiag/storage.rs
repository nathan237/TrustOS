//! Storage Diagnostics — detect and identify disks, NVMe, AHCI/SATA controllers
//!
//! Enumerates storage controllers via PCI, identifies drives,
//! reads basic metadata (model, serial, capacity, firmware).

use alloc::string::String;
use alloc::format;
use alloc::vec::Vec;
use super::dbg_out;

/// Run storage diagnostics
pub fn run() {
    dbg_out!("[STOR] === Storage Controller Diagnostics ===");

    // Find storage controllers via PCI
    let devices = crate::pci::scan();
    let storage_devs: Vec<_> = devices.iter().filter(|d| d.class_code == 0x01).collect();

    if storage_devs.is_empty() {
        dbg_out!("[STOR] No storage controllers found via PCI!");
        return;
    }

    dbg_out!("[STOR] Found {} storage controller(s)", storage_devs.len());
    dbg_out!("");

    for dev in &storage_devs {
        let bdf = alloc::format!("{:02x}:{:02x}.{}", dev.bus, dev.device, dev.function);
        let subclass_name = match dev.subclass {
            0x00 => "SCSI Bus Controller",
            0x01 => "IDE Controller",
            0x02 => "Floppy Controller",
            0x04 => "RAID Controller",
            0x05 => "ATA Controller",
            0x06 => match dev.prog_if {
                0x00 => "SATA (vendor specific)",
                0x01 => "SATA (AHCI 1.0)",
                0x02 => "SATA (Serial Storage Bus)",
                _ => "SATA (unknown interface)",
            },
            0x07 => "SAS (Serial Attached SCSI)",
            0x08 => match dev.prog_if {
                0x01 => "NVMe (NVM Express)",
                0x02 => "NVMe (NVM Express over Fabrics)",
                _ => "Non-Volatile Memory Controller",
            },
            0x09 => "UFS (Universal Flash Storage)",
            _ => "Unknown Storage",
        };

        dbg_out!("[STOR] ┌─ {} [{:04x}:{:04x}] {} ──────────────────",
            bdf, dev.vendor_id, dev.device_id, subclass_name);
        dbg_out!("[STOR] │ Vendor: {} (0x{:04X})", dev.vendor_name(), dev.vendor_id);

        // For AHCI controllers (subclass 0x06, progif 0x01)
        if dev.subclass == 0x06 && dev.prog_if == 0x01 {
            probe_ahci(dev);
        }

        // For NVMe controllers (subclass 0x08)
        if dev.subclass == 0x08 {
            probe_nvme(dev);
        }

        // For IDE controllers
        if dev.subclass == 0x01 {
            probe_ide(dev);
        }

        dbg_out!("[STOR] └──────────────────────────────────────────────────");
        dbg_out!("");
    }
}

fn probe_ahci(dev: &crate::pci::PciDevice) {
    // AHCI BAR5 contains ABAR (AHCI Base Address Register)
    if let Some(abar) = dev.bar_address(5) {
        dbg_out!("[STOR] │ AHCI ABAR: 0x{:016X}", abar);

        // Read AHCI Generic Host Control registers
        let virt = crate::memory::phys_to_virt(abar);
        if virt != 0 {
            let cap = unsafe { *(virt as *const u32) };
            let ghc = unsafe { *((virt + 4) as *const u32) };
            let pi = unsafe { *((virt + 0x0C) as *const u32) };  // Ports Implemented
            let vs = unsafe { *((virt + 0x10) as *const u32) };  // Version

            let num_ports = ((cap >> 0) & 0x1F) + 1;
            let num_cmd_slots = ((cap >> 8) & 0x1F) + 1;
            let supports_64bit = cap & (1 << 31) != 0;
            let supports_ncq = cap & (1 << 30) != 0;
            let ahci_enabled = ghc & (1 << 31) != 0;
            let version_major = (vs >> 16) & 0xFFFF;
            let version_minor = vs & 0xFFFF;

            dbg_out!("[STOR] │ AHCI Version: {}.{}", version_major, version_minor);
            dbg_out!("[STOR] │ Ports: {} (implemented: 0b{:032b})", num_ports, pi);
            dbg_out!("[STOR] │ Command slots: {}  64-bit: {}  NCQ: {}  Enabled: {}",
                num_cmd_slots, supports_64bit, supports_ncq, ahci_enabled);

            // Check each implemented port
            for port in 0..32u32 {
                if pi & (1 << port) == 0 { continue; }

                let port_base = virt + 0x100 + (port as u64 * 0x80);
                let ssts = unsafe { *((port_base + 0x28) as *const u32) }; // SStatus
                let sig = unsafe { *((port_base + 0x24) as *const u32) };  // Signature
                let serr = unsafe { *((port_base + 0x30) as *const u32) }; // SError

                let det = ssts & 0xF;      // Device detection
                let spd = (ssts >> 4) & 0xF; // Speed
                let ipm = (ssts >> 8) & 0xF; // Interface power management

                let det_str = match det {
                    0 => "No device",
                    1 => "Device, no PHY",
                    3 => "Device + PHY active",
                    4 => "PHY offline",
                    _ => "Unknown",
                };

                let spd_str = match spd {
                    0 => "no negotiation",
                    1 => "Gen1 (1.5 Gbps)",
                    2 => "Gen2 (3.0 Gbps)",
                    3 => "Gen3 (6.0 Gbps)",
                    _ => "unknown",
                };

                let sig_str = match sig {
                    0x0000_0101 => "SATA drive",
                    0xEB14_0101 => "SATAPI (ATAPI)",
                    0xC33C_0101 => "SATA Enclosure Management Bridge",
                    0x9669_0101 => "Port Multiplier",
                    _ => "Unknown/empty",
                };

                if det == 3 {
                    dbg_out!("[STOR] │   Port {}: {} — {} @ {}",
                        port, det_str, sig_str, spd_str);
                    if serr != 0 {
                        dbg_out!("[STOR] │     SError: 0x{:08X} (errors present!)", serr);
                    }
                } else if det != 0 {
                    dbg_out!("[STOR] │   Port {}: {}", port, det_str);
                }
            }
        }
    } else {
        dbg_out!("[STOR] │ AHCI: BAR5 not available");
    }
}

fn probe_nvme(dev: &crate::pci::PciDevice) {
    // NVMe BAR0 is the MMIO register set
    if let Some(bar0) = dev.bar_address(0) {
        dbg_out!("[STOR] │ NVMe BAR0: 0x{:016X}", bar0);

        let virt = crate::memory::phys_to_virt(bar0);
        if virt != 0 {
            let cap = unsafe { *(virt as *const u64) };       // Controller Capabilities
            let vs = unsafe { *((virt + 0x08) as *const u32) }; // Version
            let cc = unsafe { *((virt + 0x14) as *const u32) }; // Controller Configuration
            let csts = unsafe { *((virt + 0x1C) as *const u32) }; // Controller Status

            let mqes = (cap & 0xFFFF) + 1;             // Max Queue Entries
            let cqr = (cap >> 16) & 1;                 // Contiguous Queues Required
            let dstrd = (cap >> 32) & 0xF;             // Doorbell Stride
            let mpsmin = (cap >> 48) & 0xF;            // Memory Page Size Minimum
            let mpsmax = (cap >> 52) & 0xF;            // Memory Page Size Maximum
            let css = (cap >> 37) & 0xFF;               // Command Sets Supported

            let major = (vs >> 16) & 0xFFFF;
            let minor = (vs >> 8) & 0xFF;
            let tertiary = vs & 0xFF;

            let ready = csts & 1 != 0;
            let fatal = csts & (1 << 1) != 0;
            let enabled = cc & 1 != 0;

            dbg_out!("[STOR] │ NVMe Version: {}.{}.{}", major, minor, tertiary);
            dbg_out!("[STOR] │ Max Queue Entries: {}", mqes);
            dbg_out!("[STOR] │ Page Size: {}-{} (min 2^{}, max 2^{})",
                1 << (12 + mpsmin), 1 << (12 + mpsmax), 12 + mpsmin, 12 + mpsmax);
            dbg_out!("[STOR] │ Doorbell Stride: {} bytes", 4 << dstrd);
            dbg_out!("[STOR] │ Enabled: {}  Ready: {}  Fatal: {}", enabled, ready, fatal);
            dbg_out!("[STOR] │ NVM Command Set: {}", if css & 1 != 0 { "supported" } else { "NOT supported" });
        }
    } else {
        dbg_out!("[STOR] │ NVMe: BAR0 not available");
    }
}

fn probe_ide(_dev: &crate::pci::PciDevice) {
    dbg_out!("[STOR] │ Legacy IDE controller detected");

    // Check standard IDE I/O ports
    #[cfg(target_arch = "x86_64")]
    {
        // Primary channel: 0x1F0-0x1F7 (data), 0x3F6 (control)
        let status_primary = crate::debug::inb(0x1F7);
        let status_secondary = crate::debug::inb(0x177);

        dbg_out!("[STOR] │   Primary channel (0x1F0): status=0x{:02X} {}",
            status_primary,
            if status_primary == 0xFF { "(not present)" }
            else if status_primary & 0x40 != 0 { "(drive ready)" }
            else { "(busy or not ready)" });

        dbg_out!("[STOR] │   Secondary channel (0x170): status=0x{:02X} {}",
            status_secondary,
            if status_secondary == 0xFF { "(not present)" }
            else if status_secondary & 0x40 != 0 { "(drive ready)" }
            else { "(busy or not ready)" });
    }

    #[cfg(not(target_arch = "x86_64"))]
    {
        dbg_out!("[STOR] │   IDE port probing only available on x86_64");
    }
}
