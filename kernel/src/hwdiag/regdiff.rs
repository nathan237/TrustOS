//! Register Snapshot & Diff — capture hardware state, compare after an operation
//!
//! `hwdbg regdiff snap [name]`  — Save current PCI + MSR + IO state
//! `hwdbg regdiff diff [name]`  — Show what changed since snapshot
//! `hwdbg regdiff clear`        — Clear stored snapshot

use alloc::vec::Vec;
use alloc::string::String;
use alloc::format;
use spin::Mutex;

use super::dbg_out;

/// A register snapshot entry
#[derive(Clone)]
struct RegEntry {
    source: RegSource,
    addr: u64,
    value: u32,
}

#[derive(Clone, Copy, PartialEq)]
enum RegSource {
    PciConfig,    // bus:dev.fn + offset
    Msr,          // MSR index
    IoPort,       // I/O port address
}

struct Snapshot {
    name: String,
    entries: Vec<RegEntry>,
    timestamp_ms: u64,
}

static SAVED_SNAPSHOT: Mutex<Option<Snapshot>> = Mutex::new(None);

/// Dispatcher
pub fn run(args: &[&str]) {
    let subcmd = args.first().copied().unwrap_or("help");
    match subcmd {
        "snap" | "snapshot" | "save" => {
            let name = args.get(1).copied().unwrap_or("default");
            cmd_snap(name);
        }
        "diff" | "compare" | "cmp" => {
            cmd_diff();
        }
        "clear" | "reset" => {
            *SAVED_SNAPSHOT.lock() = None;
            dbg_out!("Snapshot cleared.");
        }
        _ => {
            dbg_out!("Usage:");
            dbg_out!("  hwdbg regdiff snap [name]  — Capture register snapshot");
            dbg_out!("  hwdbg regdiff diff         — Show changes since snapshot");
            dbg_out!("  hwdbg regdiff clear        — Clear stored snapshot");
        }
    }
}

fn cmd_snap(name: &str) {
    super::section_header("REGISTER SNAPSHOT");
    dbg_out!("Capturing state: '{}'...", name);

    let mut entries = Vec::new();
    let ts = crate::time::uptime_ms();

    // 1. PCI config: scan all existing devices, capture first 64 bytes
    let devices = crate::pci::scan();
    let pci_count = devices.len();
    for dev in &devices {
        for offset in (0..64u16).step_by(4) {
            let val = crate::pci::config_read(dev.bus, dev.device, dev.function, offset as u8);
            let addr = encode_pci_addr(dev.bus, dev.device, dev.function, offset);
            entries.push(RegEntry { source: RegSource::PciConfig, addr, value: val });
        }
    }
    dbg_out!("  PCI: {} devices × 16 dwords = {} registers", pci_count, pci_count * 16);

    // 2. Key MSRs (x86_64 only)
    #[cfg(target_arch = "x86_64")]
    {
        let msr_list: &[u32] = &[
            0x10,   // IA32_TIME_STAMP_COUNTER (low 32 unused, but we capture it)
            0x1B,   // IA32_APIC_BASE
            0xFE,   // IA32_MTRRCAP
            0x174,  // IA32_SYSENTER_CS
            0x176,  // IA32_SYSENTER_EIP
            0x179,  // IA32_MCG_CAP
            0x198,  // IA32_PERF_STATUS
            0x199,  // IA32_PERF_CTL
            0x19A,  // IA32_CLOCK_MODULATION
            0x19C,  // IA32_THERM_STATUS
            0x1A2,  // IA32_TEMPERATURE_TARGET
            0x1B1,  // IA32_PACKAGE_THERM_STATUS
            0xC0000080, // IA32_EFER
            0xC0000081, // IA32_STAR
            0xC0000082, // IA32_LSTAR
            0xC0000084, // IA32_FMASK
        ];
        let mut msr_count = 0;
        for &msr in msr_list {
            if let Some(val) = crate::debug::read_msr_safe(msr) {
                entries.push(RegEntry { source: RegSource::Msr, addr: msr as u64, value: val as u32 });
                // Also capture high 32 bits
                entries.push(RegEntry { source: RegSource::Msr, addr: (msr as u64) | (1u64 << 32), value: (val >> 32) as u32 });
                msr_count += 1;
            }
        }
        dbg_out!("  MSR: {} registers captured", msr_count);
    }

    // 3. Key I/O ports
    #[cfg(target_arch = "x86_64")]
    {
        let io_ports: &[u16] = &[
            0x20, 0x21,       // PIC1
            0xA0, 0xA1,       // PIC2
            0x40, 0x41, 0x42, 0x43, // PIT
            0x60, 0x64,       // Keyboard controller
            0x70, 0x71,       // CMOS/RTC
            0x3F8, 0x3F9, 0x3FA, 0x3FB, 0x3FC, 0x3FD, // COM1
            0x2F8, 0x2F9, 0x2FA, 0x2FB, 0x2FC, 0x2FD, // COM2
            0x1F0, 0x1F1, 0x1F2, 0x1F7, // IDE primary status
            0x170, 0x171, 0x172, 0x177, // IDE secondary status
            0xCF8, // PCI config address
        ];
        for &port in io_ports {
            let val = crate::debug::inb(port);
            entries.push(RegEntry { source: RegSource::IoPort, addr: port as u64, value: val as u32 });
        }
        dbg_out!("  I/O: {} ports captured", io_ports.len());
    }

    let total = entries.len();
    *SAVED_SNAPSHOT.lock() = Some(Snapshot {
        name: String::from(name),
        entries,
        timestamp_ms: ts,
    });

    dbg_out!("  Total: {} registers saved at {}ms", total, ts);
    dbg_out!("");
    dbg_out!("Now perform your operation, then run: hwdbg regdiff diff");
}

fn cmd_diff() {
    let snap_guard = SAVED_SNAPSHOT.lock();
    let snap = match snap_guard.as_ref() {
        Some(s) => s,
        None => {
            dbg_out!("ERROR: No snapshot saved. Run: hwdbg regdiff snap");
            return;
        }
    };

    super::section_header("REGISTER DIFF");
    let now_ms = crate::time::uptime_ms();
    dbg_out!("Comparing against '{}' (taken at {}ms, {}ms ago)",
        snap.name, snap.timestamp_ms, now_ms - snap.timestamp_ms);
    dbg_out!("");

    let mut changes = 0u32;

    // Re-read all entries and compare
    for entry in &snap.entries {
        let current = match entry.source {
            RegSource::PciConfig => {
                let (bus, dev, func, offset) = decode_pci_addr(entry.addr);
                crate::pci::config_read(bus, dev, func, offset as u8)
            }
            #[cfg(target_arch = "x86_64")]
            RegSource::Msr => {
                let msr = (entry.addr & 0xFFFFFFFF) as u32;
                let is_high = (entry.addr >> 32) & 1 != 0;
                match crate::debug::read_msr_safe(msr) {
                    Some(val) => if is_high { (val >> 32) as u32 } else { val as u32 },
                    None => 0xDEAD_DEAD,
                }
            }
            #[cfg(target_arch = "x86_64")]
            RegSource::IoPort => {
                crate::debug::inb(entry.addr as u16) as u32
            }
            #[cfg(not(target_arch = "x86_64"))]
            _ => entry.value, // No change detected on non-x86
        };

        if current != entry.value {
            changes += 1;
            let changed_bits = current ^ entry.value;
            match entry.source {
                RegSource::PciConfig => {
                    let (bus, dev, func, offset) = decode_pci_addr(entry.addr);
                    dbg_out!("  PCI {:02x}:{:02x}.{} +{:#04X}: {:#010X} → {:#010X}  (Δ bits: {:#010X})",
                        bus, dev, func, offset, entry.value, current, changed_bits);
                }
                RegSource::Msr => {
                    let msr = (entry.addr & 0xFFFFFFFF) as u32;
                    let half = if (entry.addr >> 32) & 1 != 0 { "[hi]" } else { "[lo]" };
                    dbg_out!("  MSR {:#010X}{}: {:#010X} → {:#010X}  (Δ bits: {:#010X})",
                        msr, half, entry.value, current, changed_bits);
                }
                RegSource::IoPort => {
                    dbg_out!("  I/O {:#06X}: {:#04X} → {:#04X}  (Δ bits: {:#04X})",
                        entry.addr, entry.value & 0xFF, current & 0xFF, changed_bits & 0xFF);
                }
            }
        }
    }

    dbg_out!("");
    if changes == 0 {
        dbg_out!("No changes detected ({} registers checked).", snap.entries.len());
    } else {
        dbg_out!("{} register(s) changed out of {} total.", changes, snap.entries.len());
    }
}

// Encode bus:dev.fn:offset into a single u64
fn encode_pci_addr(bus: u8, dev: u8, func: u8, offset: u16) -> u64 {
    ((bus as u64) << 24) | ((dev as u64) << 16) | ((func as u64) << 8) | (offset as u64)
}

fn decode_pci_addr(addr: u64) -> (u8, u8, u8, u16) {
    let bus = ((addr >> 24) & 0xFF) as u8;
    let dev = ((addr >> 16) & 0xFF) as u8;
    let func = ((addr >> 8) & 0xFF) as u8;
    let offset = (addr & 0xFF) as u16;
    (bus, dev, func, offset)
}
