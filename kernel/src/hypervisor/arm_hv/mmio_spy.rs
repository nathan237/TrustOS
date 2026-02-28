//! MMIO Spy — Real-time Hardware Access Logger
//!
//! This is the "brain" of visibility. Every MMIO access and SMC call
//! intercepted by the EL2 hypervisor is logged here in a lock-free
//! ring buffer. The shell can query this to show real-time activity.
//!
//! Think of it as Wireshark, but for hardware register access.

use core::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

/// Maximum events in the ring buffer
const MAX_MMIO_EVENTS: usize = 512;
const MAX_SMC_EVENTS: usize = 128;

/// A single MMIO access event
#[derive(Debug, Clone, Copy)]
pub struct MmioEvent {
    /// IPA (Intermediate Physical Address) of the access
    pub ipa: u64,
    /// Virtual address (from FAR_EL2)
    pub va: u64,
    /// Value read or written
    pub value: u64,
    /// Access size in bytes (1, 2, 4, 8)
    pub access_size: u32,
    /// true = write, false = read
    pub is_write: bool,
    /// Was this an instruction fetch? (unusual for MMIO)
    pub was_inst_fetch: bool,
    /// Identified device name
    pub device_name: &'static str,
}

/// A single SMC (Secure Monitor Call) event
#[derive(Debug, Clone, Copy)]
pub struct SmcEvent {
    /// Function ID (x0)
    pub fid: u64,
    /// Argument 1 (x1)
    pub x1: u64,
    /// Argument 2 (x2)
    pub x2: u64,
    /// Argument 3 (x3)
    pub x3: u64,
    /// Decoded type name
    pub smc_type_name: &'static str,
}

/// Ring buffer entry with sequence number for lock-free operation
#[derive(Clone, Copy)]
struct MmioSlot {
    seq: u64,
    event: MmioEvent,
}

#[derive(Clone, Copy)]
struct SmcSlot {
    seq: u64,
    event: SmcEvent,
}

/// Default MmioEvent (zeroed)
const EMPTY_MMIO: MmioEvent = MmioEvent {
    ipa: 0,
    va: 0,
    value: 0,
    access_size: 0,
    is_write: false,
    was_inst_fetch: false,
    device_name: "",
};

/// Default SmcEvent (zeroed)
const EMPTY_SMC: SmcEvent = SmcEvent {
    fid: 0,
    x1: 0,
    x2: 0,
    x3: 0,
    smc_type_name: "",
};

/// Global MMIO event ring buffer (static, no heap needed at interrupt time)
static mut MMIO_RING: [MmioSlot; MAX_MMIO_EVENTS] = {
    let slot = MmioSlot { seq: 0, event: EMPTY_MMIO };
    [slot; MAX_MMIO_EVENTS]
};

static mut SMC_RING: [SmcSlot; MAX_SMC_EVENTS] = {
    let slot = SmcSlot { seq: 0, event: EMPTY_SMC };
    [slot; MAX_SMC_EVENTS]
};

/// Write cursor for MMIO events
static MMIO_WRITE_IDX: AtomicUsize = AtomicUsize::new(0);
/// Total MMIO events logged
static MMIO_TOTAL: AtomicU64 = AtomicU64::new(0);

/// Write cursor for SMC events
static SMC_WRITE_IDX: AtomicUsize = AtomicUsize::new(0);
/// Total SMC events logged
static SMC_TOTAL: AtomicU64 = AtomicU64::new(0);

/// Log an MMIO access event (called from trap handler at EL2)
pub fn log_event(event: MmioEvent) {
    let idx = MMIO_WRITE_IDX.fetch_add(1, Ordering::Relaxed) % MAX_MMIO_EVENTS;
    let seq = MMIO_TOTAL.fetch_add(1, Ordering::Relaxed) + 1;
    unsafe {
        MMIO_RING[idx] = MmioSlot { seq, event };
    }
}

/// Log an SMC event (called from trap handler at EL2)
pub fn log_smc(event: SmcEvent) {
    let idx = SMC_WRITE_IDX.fetch_add(1, Ordering::Relaxed) % MAX_SMC_EVENTS;
    let seq = SMC_TOTAL.fetch_add(1, Ordering::Relaxed) + 1;
    unsafe {
        SMC_RING[idx] = SmcSlot { seq, event };
    }
}

/// Total number of MMIO events recorded
pub fn total_mmio_events() -> u64 {
    MMIO_TOTAL.load(Ordering::Relaxed)
}

/// Total number of SMC events recorded
pub fn total_smc_events() -> u64 {
    SMC_TOTAL.load(Ordering::Relaxed)
}

/// Get recent MMIO events (newest first, up to `count`)
pub fn recent_mmio_events(count: usize) -> alloc::vec::Vec<MmioEvent> {
    let total = MMIO_TOTAL.load(Ordering::Acquire) as usize;
    if total == 0 {
        return alloc::vec::Vec::new();
    }

    let n = count.min(total).min(MAX_MMIO_EVENTS);
    let write_pos = MMIO_WRITE_IDX.load(Ordering::Acquire);
    let mut events = alloc::vec::Vec::with_capacity(n);

    for i in 0..n {
        let idx = (write_pos + MAX_MMIO_EVENTS - 1 - i) % MAX_MMIO_EVENTS;
        let slot = unsafe { &MMIO_RING[idx] };
        if slot.seq > 0 {
            events.push(slot.event);
        }
    }

    events
}

/// Get recent SMC events (newest first, up to `count`)
pub fn recent_smc_events(count: usize) -> alloc::vec::Vec<SmcEvent> {
    let total = SMC_TOTAL.load(Ordering::Acquire) as usize;
    if total == 0 {
        return alloc::vec::Vec::new();
    }

    let n = count.min(total).min(MAX_SMC_EVENTS);
    let write_pos = SMC_WRITE_IDX.load(Ordering::Acquire);
    let mut events = alloc::vec::Vec::with_capacity(n);

    for i in 0..n {
        let idx = (write_pos + MAX_SMC_EVENTS - 1 - i) % MAX_SMC_EVENTS;
        let slot = unsafe { &SMC_RING[idx] };
        if slot.seq > 0 {
            events.push(slot.event);
        }
    }

    events
}

/// Get per-device MMIO access statistics
pub fn device_stats() -> alloc::vec::Vec<(&'static str, u64, u64)> {
    // (device_name, read_count, write_count)
    let mut stats: alloc::vec::Vec<(&str, u64, u64)> = alloc::vec::Vec::new();

    let total = MMIO_TOTAL.load(Ordering::Acquire) as usize;
    let n = total.min(MAX_MMIO_EVENTS);

    for i in 0..n {
        let slot = unsafe { &MMIO_RING[i] };
        if slot.seq == 0 {
            continue;
        }

        let name = slot.event.device_name;
        if let Some(entry) = stats.iter_mut().find(|s| s.0 == name) {
            if slot.event.is_write {
                entry.2 += 1;
            } else {
                entry.1 += 1;
            }
        } else {
            if slot.event.is_write {
                stats.push((name, 0, 1));
            } else {
                stats.push((name, 1, 0));
            }
        }
    }

    stats
}

/// Identify a device by its IPA (physical address)
///
/// This covers common ARM SoC peripherals. On a real phone,
/// these ranges come from the DTB.
pub fn identify_device(ipa: u64) -> &'static str {
    match ipa {
        // QEMU virt machine peripherals
        0x0800_0000..=0x0800_FFFF => "GIC-Dist",
        0x0801_0000..=0x0801_FFFF => "GIC-Redist",
        0x0802_0000..=0x0803_FFFF => "GIC-ITS",
        0x0900_0000..=0x0900_0FFF => "PL011-UART",
        0x0901_0000..=0x0901_0FFF => "RTC (PL031)",
        0x0903_0000..=0x0903_0FFF => "GPIO",
        0x0A00_0000..=0x0A00_01FF => "VirtIO-0",
        0x0A00_0200..=0x0A00_03FF => "VirtIO-1",
        0x0A00_0400..=0x0A00_05FF => "VirtIO-2",
        0x0A00_0600..=0x0A00_07FF => "VirtIO-3",
        0x0C00_0000..=0x0C1F_FFFF => "PCIe-ECAM",
        0x1000_0000..=0x3EFF_FFFF => "PCIe-MMIO",
        0x4010_0000..=0x4010_0FFF => "Platform-Bus",

        // Qualcomm Snapdragon common ranges
        0x0B00_0000..=0x0B0F_FFFF => "QC-APCS-GIC",
        0x0B11_0000..=0x0B11_0FFF => "QC-Timer",
        0x0780_0000..=0x07FF_FFFF => "QC-BLSP-UART",
        0x0100_0000..=0x0100_FFFF => "QC-CLK-CTL",
        0x0050_0000..=0x005F_FFFF => "QC-CRYPTO",
        0x0080_0000..=0x008F_FFFF => "QC-IMEM",

        // Google Tensor / Samsung Exynos common ranges
        0x1000_0000..=0x100F_FFFF => "Exynos-CMU",
        0x1200_0000..=0x120F_FFFF => "Exynos-DMC",
        0x1385_0000..=0x1385_FFFF => "Exynos-UART",

        // Generic fallback by address range
        0x0000_0000..=0x07FF_FFFF => "LowPeripheral",
        0x0800_0000..=0x0FFF_FFFF => "MidPeripheral",
        _ => "Unknown-MMIO",
    }
}

/// Format a single MMIO event for display
pub fn format_mmio_event(event: &MmioEvent) -> alloc::string::String {
    use alloc::format;
    let direction = if event.is_write { "WR" } else { "RD" };
    let size_str = match event.access_size {
        1 => "B",
        2 => "H",
        4 => "W",
        8 => "D",
        _ => "?",
    };
    format!(
        "[{}] {} @0x{:08X} = 0x{:X} ({}{})",
        event.device_name,
        direction,
        event.ipa,
        event.value,
        size_str,
        if event.was_inst_fetch { " IFETCH!" } else { "" }
    )
}

/// Format an SMC event for display
pub fn format_smc_event(event: &SmcEvent) -> alloc::string::String {
    use alloc::format;
    format!(
        "SMC {} FID=0x{:08X} x1=0x{:X} x2=0x{:X} x3=0x{:X}",
        event.smc_type_name,
        event.fid,
        event.x1,
        event.x2,
        event.x3,
    )
}
