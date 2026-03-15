//! TrustVM Debug Monitor — Real-time VM analysis and gap detection
//!
//! Records all VM exits, categorizes handled vs unhandled operations,
//! tracks I/O port access patterns, MSR usage, CPUID leaves, and NPF faults
//! to identify exactly what's missing for full VM functionality.
//!
//! Usage: `vm debug <id>` — show real-time debug dashboard
//!        `vm debug gaps` — show only unhandled/missing operations
//!        `vm debug io` — show I/O port access heatmap
//!        `vm debug msr` — show MSR access log
//!        `vm debug timeline` — show recent exit timeline

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use spin::Mutex;

/// Maximum number of unique events to track per category
const MAX_TRACKED_ENTRIES: usize = 256;
/// Maximum timeline entries
const MAX_TIMELINE: usize = 512;
/// Maximum gap entries per category
const MAX_GAPS: usize = 128;

/// Global debug monitor state — shared across all VMs
static DEBUG_MONITOR: Mutex<Option<DebugMonitor>> = Mutex::new(None);
/// Whether the debug monitor is actively recording
static DEBUG_ACTIVE: AtomicBool = AtomicBool::new(false);
/// Total events recorded
static TOTAL_EVENTS: AtomicU64 = AtomicU64::new(0);

/// Debug event categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DebugCategory {
    IoPortIn,
    IoPortOut,
    MsrRead,
    MsrWrite,
    CpuidLeaf,
    NpfFault,
    Interrupt,
    Hypercall,
    CrWrite,
    Exception,
    Other,
}

impl DebugCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::IoPortIn => "I/O IN",
            Self::IoPortOut => "I/O OUT",
            Self::MsrRead => "RDMSR",
            Self::MsrWrite => "WRMSR",
            Self::CpuidLeaf => "CPUID",
            Self::NpfFault => "NPF",
            Self::Interrupt => "INTR",
            Self::Hypercall => "VMCALL",
            Self::CrWrite => "CR WRITE",
            Self::Exception => "EXCEPTION",
            Self::Other => "OTHER",
        }
    }
}

/// Status of a recorded event — was it handled or not?
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandleStatus {
    /// Fully handled by the hypervisor
    Handled,
    /// Partially handled (default/stub response)
    Stubbed,
    /// Not handled — returned default/error
    Unhandled,
    /// Caused a VM crash or stop
    Fatal,
}

impl HandleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Handled => "OK",
            Self::Stubbed => "STUB",
            Self::Unhandled => "MISS",
            Self::Fatal => "FATAL",
        }
    }

    pub fn color_code(&self) -> &'static str {
        match self {
            Self::Handled => "\x01G",    // Green
            Self::Stubbed => "\x01Y",    // Yellow
            Self::Unhandled => "\x01R",  // Red
            Self::Fatal => "\x01M",      // Magenta
        }
    }
}

/// A single recorded debug event
#[derive(Debug, Clone)]
pub struct DebugEvent {
    /// VM ID
    pub vm_id: u64,
    /// Event category
    pub category: DebugCategory,
    /// Unique identifier (port number, MSR address, CPUID leaf, etc.)
    pub identifier: u64,
    /// Handle status
    pub status: HandleStatus,
    /// Guest RIP when the event occurred
    pub guest_rip: u64,
    /// Additional detail string
    pub detail: String,
    /// VMEXIT number when this occurred
    pub exit_number: u64,
}

/// Per-identifier statistics
#[derive(Debug, Clone)]
pub struct IdentifierStats {
    /// Number of times this identifier was accessed
    pub count: u64,
    /// Handle status (most recent)
    pub status: HandleStatus,
    /// First guest RIP that accessed this
    pub first_rip: u64,
    /// Last guest RIP that accessed this
    pub last_rip: u64,
    /// Human-readable name if known
    pub name: String,
    /// Last detail/value
    pub last_detail: String,
}

/// Timeline entry (compact)
#[derive(Debug, Clone)]
pub struct TimelineEntry {
    pub exit_number: u64,
    pub category: DebugCategory,
    pub identifier: u64,
    pub status: HandleStatus,
    pub guest_rip: u64,
}

/// The main debug monitor struct
pub struct DebugMonitor {
    /// Per-category per-identifier stats
    pub stats: BTreeMap<(DebugCategory, u64), IdentifierStats>,
    /// Recent timeline
    pub timeline: Vec<TimelineEntry>,
    /// Timeline write position (circular)
    pub timeline_pos: usize,
    /// Total events per category
    pub category_counts: BTreeMap<DebugCategory, u64>,
    /// Total unhandled events per category
    pub unhandled_counts: BTreeMap<DebugCategory, u64>,
    /// Gap report: identifiers that were unhandled/stubbed
    pub gaps: Vec<(DebugCategory, u64, String, u64)>, // (cat, id, name, count)
    /// VM IDs being monitored (empty = all)
    pub monitored_vms: Vec<u64>,
    /// Whether to log to serial in real-time
    pub serial_log: bool,
    /// Capture start VMEXIT count
    pub start_exit: u64,
}

impl DebugMonitor {
    pub fn new() -> Self {
        Self {
            stats: BTreeMap::new(),
            timeline: Vec::with_capacity(MAX_TIMELINE),
            timeline_pos: 0,
            category_counts: BTreeMap::new(),
            unhandled_counts: BTreeMap::new(),
            gaps: Vec::new(),
            monitored_vms: Vec::new(),
            serial_log: false,
            start_exit: 0,
        }
    }

    /// Record an event
    pub fn record(&mut self, event: DebugEvent) {
        // Update category count
        *self.category_counts.entry(event.category).or_insert(0) += 1;

        // Update unhandled count
        if matches!(event.status, HandleStatus::Unhandled | HandleStatus::Fatal) {
            *self.unhandled_counts.entry(event.category).or_insert(0) += 1;
        }

        // Update per-identifier stats
        let key = (event.category, event.identifier);
        if let Some(stat) = self.stats.get_mut(&key) {
            stat.count += 1;
            stat.status = event.status;
            stat.last_rip = event.guest_rip;
            stat.last_detail = event.detail.clone();
        } else if self.stats.len() < MAX_TRACKED_ENTRIES {
            let name = identify_name(event.category, event.identifier);
            self.stats.insert(key, IdentifierStats {
                count: 1,
                status: event.status,
                first_rip: event.guest_rip,
                last_rip: event.guest_rip,
                name,
                last_detail: event.detail.clone(),
            });
        }

        // Update timeline (circular buffer)
        let entry = TimelineEntry {
            exit_number: event.exit_number,
            category: event.category,
            identifier: event.identifier,
            status: event.status,
            guest_rip: event.guest_rip,
        };
        if self.timeline.len() < MAX_TIMELINE {
            self.timeline.push(entry);
        } else {
            self.timeline[self.timeline_pos] = entry;
        }
        self.timeline_pos = (self.timeline_pos + 1) % MAX_TIMELINE;

        // Serial log if enabled (only non-handled)
        if self.serial_log && !matches!(event.status, HandleStatus::Handled) {
            crate::serial_println!(
                "[DBG] VM{} #{} {} 0x{:X} [{}] RIP=0x{:X} {}",
                event.vm_id, event.exit_number,
                event.category.as_str(), event.identifier,
                event.status.as_str(), event.guest_rip, event.detail
            );
        }

        TOTAL_EVENTS.fetch_add(1, Ordering::Relaxed);
    }
}

/// Initialize and start the debug monitor
pub fn init() {
    let mut monitor = DEBUG_MONITOR.lock();
    *monitor = Some(DebugMonitor::new());
    DEBUG_ACTIVE.store(true, Ordering::SeqCst);
    crate::serial_println!("[DEBUG_MONITOR] Initialized — recording all VM exits");
}

/// Stop the debug monitor
pub fn stop() {
    DEBUG_ACTIVE.store(false, Ordering::SeqCst);
    crate::serial_println!("[DEBUG_MONITOR] Stopped");
}

/// Check if debug monitor is active
pub fn is_active() -> bool {
    DEBUG_ACTIVE.load(Ordering::Relaxed)
}

/// Enable/disable serial logging of unhandled events
pub fn set_serial_log(enabled: bool) {
    if let Some(ref mut mon) = *DEBUG_MONITOR.lock() {
        mon.serial_log = enabled;
    }
}

/// Record a debug event (called from VM exit handlers)
pub fn record_event(
    vm_id: u64,
    category: DebugCategory,
    identifier: u64,
    status: HandleStatus,
    guest_rip: u64,
    exit_number: u64,
    detail: &str,
) {
    if !DEBUG_ACTIVE.load(Ordering::Relaxed) {
        return;
    }

    if let Some(ref mut mon) = *DEBUG_MONITOR.lock() {
        // Check VM filter
        if !mon.monitored_vms.is_empty() && !mon.monitored_vms.contains(&vm_id) {
            return;
        }

        mon.record(DebugEvent {
            vm_id,
            category,
            identifier,
            status,
            guest_rip,
            detail: String::from(detail),
            exit_number,
        });
    }
}

/// Get the full debug dashboard as formatted text
pub fn get_dashboard() -> String {
    let monitor = DEBUG_MONITOR.lock();
    let mon = match monitor.as_ref() {
        Some(m) => m,
        None => return String::from("Debug monitor not initialized. Run 'vm debug init' first."),
    };

    let total = TOTAL_EVENTS.load(Ordering::Relaxed);
    let mut out = String::with_capacity(4096);

    out.push_str("\x01C╔══════════════════════════════════════════════════════════════╗\x01W\n");
    out.push_str("\x01C║\x01W   \x01GTRUST\x01WVM DEBUG MONITOR — Real-time VM Analysis            \x01C║\x01W\n");
    out.push_str("\x01C╚══════════════════════════════════════════════════════════════╝\x01W\n\n");

    // Summary
    out.push_str(&format!("  \x01YTotal events:\x01W {}    \x01YActive:\x01W {}\n\n",
        total, if is_active() { "\x01Gyes\x01W" } else { "\x01Rno\x01W" }));

    // Category breakdown
    out.push_str("  \x01C── Category Breakdown ──────────────────────────────────────\x01W\n");
    out.push_str("  \x01YCategory      Total      Unhandled    Rate\x01W\n");

    let categories = [
        DebugCategory::IoPortIn, DebugCategory::IoPortOut,
        DebugCategory::MsrRead, DebugCategory::MsrWrite,
        DebugCategory::CpuidLeaf, DebugCategory::NpfFault,
        DebugCategory::Interrupt, DebugCategory::Hypercall,
        DebugCategory::CrWrite, DebugCategory::Exception,
    ];

    for cat in &categories {
        let count = mon.category_counts.get(cat).copied().unwrap_or(0);
        let unhandled = mon.unhandled_counts.get(cat).copied().unwrap_or(0);
        if count > 0 {
            let rate = if count > 0 { (unhandled * 100) / count } else { 0 };
            let color = if unhandled == 0 { "\x01G" } else if rate < 20 { "\x01Y" } else { "\x01R" };
            out.push_str(&format!("  {:<14}{:>8}    {}{:>8}\x01W    {}{}%\x01W\n",
                cat.as_str(), count, color, unhandled, color, rate));
        }
    }
    out.push('\n');

    // Gaps (unhandled/stubbed identifiers)
    let mut gaps: Vec<_> = mon.stats.iter()
        .filter(|((_, _), s)| matches!(s.status, HandleStatus::Unhandled | HandleStatus::Stubbed))
        .collect();
    gaps.sort_by(|a, b| b.1.count.cmp(&a.1.count));

    if !gaps.is_empty() {
        out.push_str("  \x01C── Missing/Stubbed Operations ──────────────────────────────\x01W\n");
        out.push_str("  \x01YCategory      ID             Name                   Count  Status\x01W\n");

        for ((cat, id), stat) in gaps.iter().take(30) {
            out.push_str(&format!("  {}{:<14}\x01W0x{:<12X} {:<22} {:>5}  {}{}\x01W\n",
                stat.status.color_code(),
                cat.as_str(), id,
                if stat.name.len() > 22 { &stat.name[..22] } else { &stat.name },
                stat.count, stat.status.color_code(), stat.status.as_str()));
        }
        out.push('\n');
    }

    // Top I/O ports
    let mut io_stats: Vec<_> = mon.stats.iter()
        .filter(|((cat, _), _)| matches!(cat, DebugCategory::IoPortIn | DebugCategory::IoPortOut))
        .collect();
    io_stats.sort_by(|a, b| b.1.count.cmp(&a.1.count));

    if !io_stats.is_empty() {
        out.push_str("  \x01C── Top I/O Ports (by frequency) ────────────────────────────\x01W\n");
        out.push_str("  \x01YDir    Port       Name                   Count  Status\x01W\n");
        for ((cat, port), stat) in io_stats.iter().take(20) {
            let dir = if matches!(cat, DebugCategory::IoPortIn) { "IN " } else { "OUT" };
            out.push_str(&format!("  {}  0x{:04X}     {:<22} {:>6}  {}{}\x01W\n",
                dir, port,
                if stat.name.len() > 22 { &stat.name[..22] } else { &stat.name },
                stat.count, stat.status.color_code(), stat.status.as_str()));
        }
        out.push('\n');
    }

    // Top MSRs
    let mut msr_stats: Vec<_> = mon.stats.iter()
        .filter(|((cat, _), _)| matches!(cat, DebugCategory::MsrRead | DebugCategory::MsrWrite))
        .collect();
    msr_stats.sort_by(|a, b| b.1.count.cmp(&a.1.count));

    if !msr_stats.is_empty() {
        out.push_str("  \x01C── MSR Access Log ──────────────────────────────────────────\x01W\n");
        out.push_str("  \x01YDir     MSR            Name                   Count  Status\x01W\n");
        for ((cat, msr), stat) in msr_stats.iter().take(20) {
            let dir = if matches!(cat, DebugCategory::MsrRead) { "READ " } else { "WRITE" };
            out.push_str(&format!("  {}  0x{:08X}     {:<22} {:>5}  {}{}\x01W\n",
                dir, msr,
                if stat.name.len() > 22 { &stat.name[..22] } else { &stat.name },
                stat.count, stat.status.color_code(), stat.status.as_str()));
        }
        out.push('\n');
    }

    // Recent timeline
    if !mon.timeline.is_empty() {
        out.push_str("  \x01C── Recent Timeline (last 20) ──────────────────────────────\x01W\n");
        out.push_str("  \x01YExit#      Category      ID             RIP              Status\x01W\n");

        let len = mon.timeline.len();
        let start = if len > 20 { len - 20 } else { 0 };
        for entry in &mon.timeline[start..] {
            out.push_str(&format!("  {:>8}   {:<14}0x{:<12X} 0x{:<14X} {}{}\x01W\n",
                entry.exit_number, entry.category.as_str(),
                entry.identifier, entry.guest_rip,
                entry.status.color_code(), entry.status.as_str()));
        }
        out.push('\n');
    }

    // Recommendations
    out.push_str("  \x01C── Recommendations ────────────────────────────────────────\x01W\n");
    let total_unhandled: u64 = mon.unhandled_counts.values().sum();
    if total_unhandled == 0 {
        out.push_str("  \x01G✓ All VM exits are handled! VM is fully functional.\x01W\n");
    } else {
        // Count unique unhandled I/O ports
        let unhandled_io: Vec<_> = mon.stats.iter()
            .filter(|((cat, _), s)| 
                matches!(cat, DebugCategory::IoPortIn | DebugCategory::IoPortOut)
                && matches!(s.status, HandleStatus::Unhandled))
            .collect();
        if !unhandled_io.is_empty() {
            out.push_str(&format!("  \x01R✗ {} unhandled I/O port(s)\x01W — implement handlers in handle_io()\n",
                unhandled_io.len()));
            for ((_, port), stat) in unhandled_io.iter().take(5) {
                out.push_str(&format!("    → 0x{:04X} {} ({}x)\n", port, stat.name, stat.count));
            }
        }

        let unhandled_msr: Vec<_> = mon.stats.iter()
            .filter(|((cat, _), s)| 
                matches!(cat, DebugCategory::MsrRead | DebugCategory::MsrWrite)
                && matches!(s.status, HandleStatus::Unhandled))
            .collect();
        if !unhandled_msr.is_empty() {
            out.push_str(&format!("  \x01R✗ {} unhandled MSR(s)\x01W — implement in handle_msr()\n",
                unhandled_msr.len()));
            for ((_, msr), stat) in unhandled_msr.iter().take(5) {
                out.push_str(&format!("    → 0x{:08X} {} ({}x)\n", msr, stat.name, stat.count));
            }
        }

        let npf_faults: Vec<_> = mon.stats.iter()
            .filter(|((cat, _), s)| 
                matches!(cat, DebugCategory::NpfFault)
                && matches!(s.status, HandleStatus::Unhandled | HandleStatus::Fatal))
            .collect();
        if !npf_faults.is_empty() {
            out.push_str(&format!("  \x01R✗ {} unhandled NPF fault address(es)\x01W — add MMIO region handlers\n",
                npf_faults.len()));
            for ((_, gpa), stat) in npf_faults.iter().take(5) {
                out.push_str(&format!("    → GPA 0x{:X} ({}x, last RIP=0x{:X})\n", gpa, stat.count, stat.last_rip));
            }
        }
    }

    out.push('\n');
    out
}

/// Get just the gaps report
pub fn get_gaps_report() -> String {
    let monitor = DEBUG_MONITOR.lock();
    let mon = match monitor.as_ref() {
        Some(m) => m,
        None => return String::from("Debug monitor not initialized."),
    };

    let mut out = String::with_capacity(2048);
    out.push_str("\x01C═══ VM Debug: Unhandled Operations Report ═══\x01W\n\n");

    let mut gaps: Vec<_> = mon.stats.iter()
        .filter(|((_, _), s)| !matches!(s.status, HandleStatus::Handled))
        .collect();
    gaps.sort_by(|a, b| b.1.count.cmp(&a.1.count));

    if gaps.is_empty() {
        out.push_str("  \x01G✓ No gaps detected — all operations handled!\x01W\n");
    } else {
        out.push_str(&format!("  \x01RFound {} unhandled/stubbed operations:\x01W\n\n", gaps.len()));
        out.push_str("  \x01YCategory      ID             Name                   Count  First RIP        Detail\x01W\n");
        for ((cat, id), stat) in &gaps {
            let name_display = if stat.name.len() > 22 { &stat.name[..22] } else { &stat.name };
            let detail_display = if stat.last_detail.len() > 30 { &stat.last_detail[..30] } else { &stat.last_detail };
            out.push_str(&format!("  {}{:<14}\x01W0x{:<12X} {:<22} {:>5}  0x{:<14X} {}\n",
                stat.status.color_code(), cat.as_str(), id, name_display,
                stat.count, stat.first_rip, detail_display));
        }
    }

    out.push('\n');
    out
}

/// Get I/O port heatmap
pub fn get_io_heatmap() -> String {
    let monitor = DEBUG_MONITOR.lock();
    let mon = match monitor.as_ref() {
        Some(m) => m,
        None => return String::from("Debug monitor not initialized."),
    };

    let mut out = String::with_capacity(2048);
    out.push_str("\x01C═══ VM Debug: I/O Port Heatmap ═══\x01W\n\n");

    // Group by port ranges
    let ranges = [
        (0x000u64, 0x020, "DMA Controller"),
        (0x020, 0x040, "PIC 8259A"),
        (0x040, 0x064, "PIT 8254 Timer"),
        (0x060, 0x068, "Keyboard (8042)"),
        (0x070, 0x080, "CMOS/RTC"),
        (0x080, 0x0A0, "DMA Page Regs"),
        (0x0A0, 0x0C0, "PIC Slave"),
        (0x0C0, 0x0E0, "DMA Controller 2"),
        (0x0E9, 0x0EA, "Debug Port"),
        (0x0ED, 0x0EE, "I/O Delay"),
        (0x2F8, 0x300, "COM2 Serial"),
        (0x3B0, 0x3E0, "VGA Registers"),
        (0x3F8, 0x400, "COM1 Serial"),
        (0xB000, 0xB040, "ACPI PM"),
        (0xC000, 0xC040, "VirtIO Console"),
        (0xC040, 0xC080, "VirtIO Block"),
        (0xCF8, 0xD00, "PCI Config"),
    ];

    out.push_str("  \x01YPort Range     Device               IN Count   OUT Count  Status\x01W\n");

    for (start, end, name) in &ranges {
        let in_count: u64 = mon.stats.iter()
            .filter(|((cat, port), _)| matches!(cat, DebugCategory::IoPortIn) && *port >= *start && *port < *end)
            .map(|(_, s)| s.count)
            .sum();
        let out_count: u64 = mon.stats.iter()
            .filter(|((cat, port), _)| matches!(cat, DebugCategory::IoPortOut) && *port >= *start && *port < *end)
            .map(|(_, s)| s.count)
            .sum();
        
        if in_count > 0 || out_count > 0 {
            let any_unhandled = mon.stats.iter()
                .any(|((cat, port), s)| 
                    matches!(cat, DebugCategory::IoPortIn | DebugCategory::IoPortOut)
                    && *port >= *start && *port < *end
                    && matches!(s.status, HandleStatus::Unhandled));
            let status = if any_unhandled { "\x01RMISS\x01W" } else { "\x01GOK\x01W" };
            out.push_str(&format!("  0x{:04X}-0x{:04X} {:<20} {:>8}   {:>8}   {}\n",
                start, end - 1, name, in_count, out_count, status));
        }
    }

    // Show unknown ports
    let unknown: Vec<_> = mon.stats.iter()
        .filter(|((cat, port), s)| 
            matches!(cat, DebugCategory::IoPortIn | DebugCategory::IoPortOut)
            && !ranges.iter().any(|(start, end, _)| *port >= *start && *port < *end))
        .collect();
    
    if !unknown.is_empty() {
        out.push_str("\n  \x01R── Unknown Ports ──\x01W\n");
        for ((cat, port), stat) in &unknown {
            let dir = if matches!(cat, DebugCategory::IoPortIn) { "IN " } else { "OUT" };
            out.push_str(&format!("  {} 0x{:04X}  {} ({}x) RIP=0x{:X}\n",
                dir, port, stat.name, stat.count, stat.last_rip));
        }
    }

    out.push('\n');
    out
}

/// Get MSR access report
pub fn get_msr_report() -> String {
    let monitor = DEBUG_MONITOR.lock();
    let mon = match monitor.as_ref() {
        Some(m) => m,
        None => return String::from("Debug monitor not initialized."),
    };

    let mut out = String::with_capacity(2048);
    out.push_str("\x01C═══ VM Debug: MSR Access Report ═══\x01W\n\n");

    let mut msr_entries: Vec<_> = mon.stats.iter()
        .filter(|((cat, _), _)| matches!(cat, DebugCategory::MsrRead | DebugCategory::MsrWrite))
        .collect();
    msr_entries.sort_by_key(|((_, msr), _)| *msr);

    if msr_entries.is_empty() {
        out.push_str("  No MSR accesses recorded.\n");
    } else {
        out.push_str("  \x01YDir     MSR            Name                        Count  Value/Detail           Status\x01W\n");
        for ((cat, msr), stat) in &msr_entries {
            let dir = if matches!(cat, DebugCategory::MsrRead) { "READ " } else { "WRITE" };
            let name_display = if stat.name.len() > 26 { &stat.name[..26] } else { &stat.name };
            let detail_display = if stat.last_detail.len() > 20 { &stat.last_detail[..20] } else { &stat.last_detail };
            out.push_str(&format!("  {}  0x{:08X}     {:<26} {:>5}  {:<20}   {}{}\x01W\n",
                dir, msr, name_display, stat.count, detail_display,
                stat.status.color_code(), stat.status.as_str()));
        }
    }

    out.push('\n');
    out
}

/// Get timeline of recent events
pub fn get_timeline(count: usize) -> String {
    let monitor = DEBUG_MONITOR.lock();
    let mon = match monitor.as_ref() {
        Some(m) => m,
        None => return String::from("Debug monitor not initialized."),
    };

    let mut out = String::with_capacity(2048);
    out.push_str("\x01C═══ VM Debug: Exit Timeline ═══\x01W\n\n");

    if mon.timeline.is_empty() {
        out.push_str("  No events recorded yet.\n");
    } else {
        let display_count = count.min(mon.timeline.len());
        let start = mon.timeline.len() - display_count;

        out.push_str("  \x01YExit#      Category      ID             RIP              Name                   Status\x01W\n");
        for entry in &mon.timeline[start..] {
            let name = identify_name(entry.category, entry.identifier);
            let name_display = if name.len() > 22 { &name[..22] } else { &name };
            out.push_str(&format!("  {:>8}   {:<14}0x{:<12X} 0x{:<14X} {:<22} {}{}\x01W\n",
                entry.exit_number, entry.category.as_str(),
                entry.identifier, entry.guest_rip,
                name_display,
                entry.status.color_code(), entry.status.as_str()));
        }
    }

    out.push('\n');
    out
}

/// Reset all debug data
pub fn reset() {
    if let Some(ref mut mon) = *DEBUG_MONITOR.lock() {
        mon.stats.clear();
        mon.timeline.clear();
        mon.timeline_pos = 0;
        mon.category_counts.clear();
        mon.unhandled_counts.clear();
        mon.gaps.clear();
        TOTAL_EVENTS.store(0, Ordering::Relaxed);
    }
}

/// Get total events recorded
pub fn total_events() -> u64 {
    TOTAL_EVENTS.load(Ordering::Relaxed)
}

/// Get count of unhandled events
pub fn unhandled_count() -> u64 {
    let monitor = DEBUG_MONITOR.lock();
    match monitor.as_ref() {
        Some(mon) => mon.unhandled_counts.values().sum(),
        None => 0,
    }
}

// ═══════════════════════════════════════════════════════════════
// Known name lookups for I/O ports, MSRs, CPUID leaves
// ═══════════════════════════════════════════════════════════════

/// Look up a human-readable name for a given category + identifier
fn identify_name(category: DebugCategory, id: u64) -> String {
    match category {
        DebugCategory::IoPortIn | DebugCategory::IoPortOut => identify_io_port(id as u16),
        DebugCategory::MsrRead | DebugCategory::MsrWrite => identify_msr(id as u32),
        DebugCategory::CpuidLeaf => format!("CPUID leaf 0x{:X}", id),
        DebugCategory::NpfFault => identify_mmio_region(id),
        DebugCategory::Interrupt => format!("IRQ {}", id),
        DebugCategory::Hypercall => format!("VMCALL 0x{:X}", id),
        DebugCategory::CrWrite => match id {
            0 => String::from("CR0"),
            3 => String::from("CR3"),
            4 => String::from("CR4"),
            _ => format!("CR{}", id),
        },
        DebugCategory::Exception => match id {
            0 => String::from("#DE Divide Error"),
            1 => String::from("#DB Debug"),
            3 => String::from("#BP Breakpoint"),
            6 => String::from("#UD Invalid Opcode"),
            7 => String::from("#NM No Math"),
            8 => String::from("#DF Double Fault"),
            13 => String::from("#GP General Protection"),
            14 => String::from("#PF Page Fault"),
            _ => format!("Exception #{}", id),
        },
        DebugCategory::Other => format!("0x{:X}", id),
    }
}

fn identify_io_port(port: u16) -> String {
    match port {
        0x00..=0x0F => String::from("DMA Controller 1"),
        0x20 => String::from("PIC Master CMD"),
        0x21 => String::from("PIC Master Data"),
        0x40 => String::from("PIT Channel 0"),
        0x41 => String::from("PIT Channel 1"),
        0x42 => String::from("PIT Channel 2"),
        0x43 => String::from("PIT Control"),
        0x60 => String::from("Keyboard Data"),
        0x61 => String::from("NMI/Speaker"),
        0x64 => String::from("Keyboard Status/Cmd"),
        0x70 => String::from("CMOS Index"),
        0x71 => String::from("CMOS Data"),
        0x80..=0x8F => String::from("DMA Page Regs"),
        0x92 => String::from("Fast A20 Gate"),
        0xA0 => String::from("PIC Slave CMD"),
        0xA1 => String::from("PIC Slave Data"),
        0xC0..=0xDF => String::from("DMA Controller 2"),
        0xE9 => String::from("Debug Port"),
        0xED => String::from("I/O Delay"),
        0x2F8 => String::from("COM2 Data"),
        0x2F9..=0x2FF => format!("COM2 +{}", port - 0x2F8),
        0x3B0..=0x3BF => String::from("VGA MDA"),
        0x3C0..=0x3CF => String::from("VGA Attr/Seq"),
        0x3D0..=0x3DF => String::from("VGA CRT/Status"),
        0x3F8 => String::from("COM1 Data"),
        0x3F9 => String::from("COM1 IER"),
        0x3FA => String::from("COM1 IIR/FCR"),
        0x3FB => String::from("COM1 LCR"),
        0x3FC => String::from("COM1 MCR"),
        0x3FD => String::from("COM1 LSR"),
        0x3FE => String::from("COM1 MSR"),
        0x3FF => String::from("COM1 Scratch"),
        0xB000 => String::from("ACPI PM1a EVT STS"),
        0xB002 => String::from("ACPI PM1a EVT EN"),
        0xB004 => String::from("ACPI PM1a CNT"),
        0xB008..=0xB00B => String::from("ACPI PM Timer"),
        0xC000..=0xC03F => format!("VirtIO Console +0x{:02X}", port - 0xC000),
        0xC040..=0xC07F => format!("VirtIO Block +0x{:02X}", port - 0xC040),
        0xCF8 => String::from("PCI Config Addr"),
        0xCFC => String::from("PCI Config Data"),
        0xCFD => String::from("PCI Config Data+1"),
        0xCFE => String::from("PCI Config Data+2"),
        0xCFF => String::from("PCI Config Data+3"),
        _ => format!("Port 0x{:04X}", port),
    }
}

fn identify_msr(msr: u32) -> String {
    match msr {
        0x001B => String::from("IA32_APIC_BASE"),
        0x00FE => String::from("IA32_MTRRCAP"),
        0x0174 => String::from("IA32_SYSENTER_CS"),
        0x0175 => String::from("IA32_SYSENTER_ESP"),
        0x0176 => String::from("IA32_SYSENTER_EIP"),
        0x0179 => String::from("IA32_MCG_CAP"),
        0x017A => String::from("IA32_MCG_STATUS"),
        0x01A0 => String::from("IA32_MISC_ENABLE"),
        0x0200..=0x020F => format!("IA32_MTRR_{:X}", msr),
        0x0250 => String::from("IA32_MTRR_FIX64K_00000"),
        0x0258 => String::from("IA32_MTRR_FIX16K_80000"),
        0x0259 => String::from("IA32_MTRR_FIX16K_A0000"),
        0x0268..=0x026F => format!("IA32_MTRR_FIX4K_{:X}", msr),
        0x0277 => String::from("IA32_PAT"),
        0x02FF => String::from("IA32_MTRR_DEF_TYPE"),
        0x0400..=0x047F => format!("IA32_MC{}_{}", (msr - 0x400) / 4, 
            match (msr - 0x400) % 4 { 0 => "CTL", 1 => "STATUS", 2 => "ADDR", _ => "MISC" }),
        0x0480 => String::from("IA32_VMX_BASIC"),
        0x048B => String::from("IA32_VMX_TRUE_PINBASED"),
        0x048D => String::from("IA32_VMX_TRUE_ENTRY"),
        0x0802 => String::from("IA32_X2APIC_EOI"),
        0xC000_0080 => String::from("IA32_EFER"),
        0xC000_0081 => String::from("MSR_STAR"),
        0xC000_0082 => String::from("MSR_LSTAR"),
        0xC000_0083 => String::from("MSR_CSTAR"),
        0xC000_0084 => String::from("MSR_SFMASK"),
        0xC000_0100 => String::from("MSR_FS_BASE"),
        0xC000_0101 => String::from("MSR_GS_BASE"),
        0xC000_0102 => String::from("MSR_KERNEL_GS_BASE"),
        0xC000_0103 => String::from("MSR_TSC_AUX"),
        _ => format!("MSR 0x{:X}", msr),
    }
}

fn identify_mmio_region(gpa: u64) -> String {
    match gpa {
        0x0000..=0x0FFF => String::from("Real-mode IVT/BDA"),
        0xA0000..=0xBFFFF => String::from("VGA Frame Buffer"),
        0xC0000..=0xDFFFF => String::from("ROM/BIOS Shadow"),
        0xE0000..=0xFFFFF => String::from("High BIOS"),
        0xFEC0_0000..=0xFEC0_0FFF => String::from("I/O APIC"),
        0xFED0_0000..=0xFED0_03FF => String::from("HPET"),
        0xFEE0_0000..=0xFEE0_0FFF => String::from("Local APIC"),
        _ if gpa >= 0x1_0000_0000 => String::from("High MMIO (>4GB)"),
        _ => format!("GPA 0x{:X}", gpa),
    }
}

// ═══════════════════════════════════════════════════════════════
// Public test API for integration tests
// ═══════════════════════════════════════════════════════════════

/// Check if debug monitor is initialized
pub fn is_initialized() -> bool {
    DEBUG_MONITOR.lock().is_some()
}

/// Get category count
pub fn category_count(cat: DebugCategory) -> u64 {
    let monitor = DEBUG_MONITOR.lock();
    match monitor.as_ref() {
        Some(mon) => mon.category_counts.get(&cat).copied().unwrap_or(0),
        None => 0,
    }
}

/// Get number of tracked unique identifiers
pub fn tracked_identifiers() -> usize {
    let monitor = DEBUG_MONITOR.lock();
    match monitor.as_ref() {
        Some(mon) => mon.stats.len(),
        None => 0,
    }
}
