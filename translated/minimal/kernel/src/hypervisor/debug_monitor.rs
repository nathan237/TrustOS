











use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use spin::Mutex;


const CJD_: usize = 256;

const AHA_: usize = 512;

const DXB_: usize = 128;


static CZ_: Mutex<Option<DebugMonitor>> = Mutex::new(None);

static TT_: AtomicBool = AtomicBool::new(false);

static FJ_: AtomicU64 = AtomicU64::new(0);


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


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandleStatus {
    
    Handled,
    
    Stubbed,
    
    Unhandled,
    
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
            Self::Handled => "\x01G",    
            Self::Stubbed => "\x01Y",    
            Self::Unhandled => "\x01R",  
            Self::Fatal => "\x01M",      
        }
    }
}


#[derive(Debug, Clone)]
pub struct Xr {
    
    pub vm_id: u64,
    
    pub category: DebugCategory,
    
    pub identifier: u64,
    
    pub status: HandleStatus,
    
    pub guest_rip: u64,
    
    pub detail: String,
    
    pub exit_number: u64,
}


#[derive(Debug, Clone)]
pub struct Aad {
    
    pub count: u64,
    
    pub status: HandleStatus,
    
    pub first_rip: u64,
    
    pub last_rip: u64,
    
    pub name: String,
    
    pub last_detail: String,
}


#[derive(Debug, Clone)]
pub struct Afi {
    pub exit_number: u64,
    pub category: DebugCategory,
    pub identifier: u64,
    pub status: HandleStatus,
    pub guest_rip: u64,
}


pub struct DebugMonitor {
    
    pub stats: BTreeMap<(DebugCategory, u64), Aad>,
    
    pub timeline: Vec<Afi>,
    
    pub timeline_pos: usize,
    
    pub category_counts: BTreeMap<DebugCategory, u64>,
    
    pub unhandled_counts: BTreeMap<DebugCategory, u64>,
    
    pub gaps: Vec<(DebugCategory, u64, String, u64)>, 
    
    pub monitored_vms: Vec<u64>,
    
    pub serial_log: bool,
    
    pub start_exit: u64,
}

impl DebugMonitor {
    pub fn new() -> Self {
        Self {
            stats: BTreeMap::new(),
            timeline: Vec::with_capacity(AHA_),
            timeline_pos: 0,
            category_counts: BTreeMap::new(),
            unhandled_counts: BTreeMap::new(),
            gaps: Vec::new(),
            monitored_vms: Vec::new(),
            serial_log: false,
            start_exit: 0,
        }
    }

    
    pub fn record(&mut self, event: Xr) {
        
        *self.category_counts.entry(event.category).or_insert(0) += 1;

        
        if matches!(event.status, HandleStatus::Unhandled | HandleStatus::Fatal) {
            *self.unhandled_counts.entry(event.category).or_insert(0) += 1;
        }

        
        let key = (event.category, event.identifier);
        if let Some(stat) = self.stats.get_mut(&key) {
            stat.count += 1;
            stat.status = event.status;
            stat.last_rip = event.guest_rip;
            stat.last_detail = event.detail.clone();
        } else if self.stats.len() < CJD_ {
            let name = ifo(event.category, event.identifier);
            self.stats.insert(key, Aad {
                count: 1,
                status: event.status,
                first_rip: event.guest_rip,
                last_rip: event.guest_rip,
                name,
                last_detail: event.detail.clone(),
            });
        }

        
        let entry = Afi {
            exit_number: event.exit_number,
            category: event.category,
            identifier: event.identifier,
            status: event.status,
            guest_rip: event.guest_rip,
        };
        if self.timeline.len() < AHA_ {
            self.timeline.push(entry);
        } else {
            self.timeline[self.timeline_pos] = entry;
        }
        self.timeline_pos = (self.timeline_pos + 1) % AHA_;

        
        if self.serial_log && !matches!(event.status, HandleStatus::Handled) {
            crate::serial_println!(
                "[DBG] VM{} #{} {} 0x{:X} [{}] RIP=0x{:X} {}",
                event.vm_id, event.exit_number,
                event.category.as_str(), event.identifier,
                event.status.as_str(), event.guest_rip, event.detail
            );
        }

        FJ_.fetch_add(1, Ordering::Relaxed);
    }
}


pub fn init() {
    let mut alz = CZ_.lock();
    *alz = Some(DebugMonitor::new());
    TT_.store(true, Ordering::SeqCst);
    crate::serial_println!("[DEBUG_MONITOR] Initialized — recording all VM exits");
}


pub fn stop() {
    TT_.store(false, Ordering::SeqCst);
    crate::serial_println!("[DEBUG_MONITOR] Stopped");
}


pub fn is_active() -> bool {
    TT_.load(Ordering::Relaxed)
}


pub fn jfj(enabled: bool) {
    if let Some(ref mut nz) = *CZ_.lock() {
        nz.serial_log = enabled;
    }
}


pub fn akj(
    vm_id: u64,
    category: DebugCategory,
    identifier: u64,
    status: HandleStatus,
    guest_rip: u64,
    exit_number: u64,
    detail: &str,
) {
    if !TT_.load(Ordering::Relaxed) {
        return;
    }

    if let Some(ref mut nz) = *CZ_.lock() {
        
        if !nz.monitored_vms.is_empty() && !nz.monitored_vms.contains(&vm_id) {
            return;
        }

        nz.record(Xr {
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


pub fn fym() -> String {
    let alz = CZ_.lock();
    let nz = match alz.as_ref() {
        Some(m) => m,
        None => return String::from("Debug monitor not initialized. Run 'vm debug init' first."),
    };

    let av = FJ_.load(Ordering::Relaxed);
    let mut out = String::with_capacity(4096);

    out.push_str("\x01C╔══════════════════════════════════════════════════════════════╗\x01W\n");
    out.push_str("\x01C║\x01W   \x01GTRUST\x01WVM DEBUG MONITOR — Real-time VM Analysis            \x01C║\x01W\n");
    out.push_str("\x01C╚══════════════════════════════════════════════════════════════╝\x01W\n\n");

    
    out.push_str(&format!("  \x01YTotal events:\x01W {}    \x01YActive:\x01W {}\n\n",
        av, if is_active() { "\x01Gyes\x01W" } else { "\x01Rno\x01W" }));

    
    out.push_str("  \x01C── Category Breakdown ──────────────────────────────────────\x01W\n");
    out.push_str("  \x01YCategory      Total      Unhandled    Rate\x01W\n");

    let cgr = [
        DebugCategory::IoPortIn, DebugCategory::IoPortOut,
        DebugCategory::MsrRead, DebugCategory::MsrWrite,
        DebugCategory::CpuidLeaf, DebugCategory::NpfFault,
        DebugCategory::Interrupt, DebugCategory::Hypercall,
        DebugCategory::CrWrite, DebugCategory::Exception,
    ];

    for hx in &cgr {
        let count = nz.category_counts.get(hx).copied().unwrap_or(0);
        let dga = nz.unhandled_counts.get(hx).copied().unwrap_or(0);
        if count > 0 {
            let exq = if count > 0 { (dga * 100) / count } else { 0 };
            let color = if dga == 0 { "\x01G" } else if exq < 20 { "\x01Y" } else { "\x01R" };
            out.push_str(&format!("  {:<14}{:>8}    {}{:>8}\x01W    {}{}%\x01W\n",
                hx.as_str(), count, color, dga, color, exq));
        }
    }
    out.push('\n');

    
    let mut gaps: Vec<_> = nz.stats.iter()
        .filter(|((_, _), j)| matches!(j.status, HandleStatus::Unhandled | HandleStatus::Stubbed))
        .collect();
    gaps.sort_by(|a, b| b.1.count.cmp(&a.1.count));

    if !gaps.is_empty() {
        out.push_str("  \x01C── Missing/Stubbed Operations ──────────────────────────────\x01W\n");
        out.push_str("  \x01YCategory      ID             Name                   Count  Status\x01W\n");

        for ((hx, id), stat) in gaps.iter().take(30) {
            out.push_str(&format!("  {}{:<14}\x01W0x{:<12X} {:<22} {:>5}  {}{}\x01W\n",
                stat.status.color_code(),
                hx.as_str(), id,
                if stat.name.len() > 22 { &stat.name[..22] } else { &stat.name },
                stat.count, stat.status.color_code(), stat.status.as_str()));
        }
        out.push('\n');
    }

    
    let mut gdi: Vec<_> = nz.stats.iter()
        .filter(|((hx, _), _)| matches!(hx, DebugCategory::IoPortIn | DebugCategory::IoPortOut))
        .collect();
    gdi.sort_by(|a, b| b.1.count.cmp(&a.1.count));

    if !gdi.is_empty() {
        out.push_str("  \x01C── Top I/O Ports (by frequency) ────────────────────────────\x01W\n");
        out.push_str("  \x01YDir    Port       Name                   Count  Status\x01W\n");
        for ((hx, port), stat) in gdi.iter().take(20) {
            let it = if matches!(hx, DebugCategory::IoPortIn) { "IN " } else { "OUT" };
            out.push_str(&format!("  {}  0x{:04X}     {:<22} {:>6}  {}{}\x01W\n",
                it, port,
                if stat.name.len() > 22 { &stat.name[..22] } else { &stat.name },
                stat.count, stat.status.color_code(), stat.status.as_str()));
        }
        out.push('\n');
    }

    
    let mut gil: Vec<_> = nz.stats.iter()
        .filter(|((hx, _), _)| matches!(hx, DebugCategory::MsrRead | DebugCategory::MsrWrite))
        .collect();
    gil.sort_by(|a, b| b.1.count.cmp(&a.1.count));

    if !gil.is_empty() {
        out.push_str("  \x01C── MSR Access Log ──────────────────────────────────────────\x01W\n");
        out.push_str("  \x01YDir     MSR            Name                   Count  Status\x01W\n");
        for ((hx, msr), stat) in gil.iter().take(20) {
            let it = if matches!(hx, DebugCategory::MsrRead) { "READ " } else { "WRITE" };
            out.push_str(&format!("  {}  0x{:08X}     {:<22} {:>5}  {}{}\x01W\n",
                it, msr,
                if stat.name.len() > 22 { &stat.name[..22] } else { &stat.name },
                stat.count, stat.status.color_code(), stat.status.as_str()));
        }
        out.push('\n');
    }

    
    if !nz.timeline.is_empty() {
        out.push_str("  \x01C── Recent Timeline (last 20) ──────────────────────────────\x01W\n");
        out.push_str("  \x01YExit#      Category      ID             RIP              Status\x01W\n");

        let len = nz.timeline.len();
        let start = if len > 20 { len - 20 } else { 0 };
        for entry in &nz.timeline[start..] {
            out.push_str(&format!("  {:>8}   {:<14}0x{:<12X} 0x{:<14X} {}{}\x01W\n",
                entry.exit_number, entry.category.as_str(),
                entry.identifier, entry.guest_rip,
                entry.status.color_code(), entry.status.as_str()));
        }
        out.push('\n');
    }

    
    out.push_str("  \x01C── Recommendations ────────────────────────────────────────\x01W\n");
    let pmi: u64 = nz.unhandled_counts.values().sum();
    if pmi == 0 {
        out.push_str("  \x01G✓ All VM exits are handled! VM is fully functional.\x01W\n");
    } else {
        
        let har: Vec<_> = nz.stats.iter()
            .filter(|((hx, _), j)| 
                matches!(hx, DebugCategory::IoPortIn | DebugCategory::IoPortOut)
                && matches!(j.status, HandleStatus::Unhandled))
            .collect();
        if !har.is_empty() {
            out.push_str(&format!("  \x01R✗ {} unhandled I/O port(s)\x01W — implement handlers in handle_io()\n",
                har.len()));
            for ((_, port), stat) in har.iter().take(5) {
                out.push_str(&format!("    → 0x{:04X} {} ({}x)\n", port, stat.name, stat.count));
            }
        }

        let has: Vec<_> = nz.stats.iter()
            .filter(|((hx, _), j)| 
                matches!(hx, DebugCategory::MsrRead | DebugCategory::MsrWrite)
                && matches!(j.status, HandleStatus::Unhandled))
            .collect();
        if !has.is_empty() {
            out.push_str(&format!("  \x01R✗ {} unhandled MSR(s)\x01W — implement in handle_msr()\n",
                has.len()));
            for ((_, msr), stat) in has.iter().take(5) {
                out.push_str(&format!("    → 0x{:08X} {} ({}x)\n", msr, stat.name, stat.count));
            }
        }

        let gjw: Vec<_> = nz.stats.iter()
            .filter(|((hx, _), j)| 
                matches!(hx, DebugCategory::NpfFault)
                && matches!(j.status, HandleStatus::Unhandled | HandleStatus::Fatal))
            .collect();
        if !gjw.is_empty() {
            out.push_str(&format!("  \x01R✗ {} unhandled NPF fault address(es)\x01W — add MMIO region handlers\n",
                gjw.len()));
            for ((_, gm), stat) in gjw.iter().take(5) {
                out.push_str(&format!("    → GPA 0x{:X} ({}x, last RIP=0x{:X})\n", gm, stat.count, stat.last_rip));
            }
        }
    }

    out.push('\n');
    out
}


pub fn fyr() -> String {
    let alz = CZ_.lock();
    let nz = match alz.as_ref() {
        Some(m) => m,
        None => return String::from("Debug monitor not initialized."),
    };

    let mut out = String::with_capacity(2048);
    out.push_str("\x01C═══ VM Debug: Unhandled Operations Report ═══\x01W\n\n");

    let mut gaps: Vec<_> = nz.stats.iter()
        .filter(|((_, _), j)| !matches!(j.status, HandleStatus::Handled))
        .collect();
    gaps.sort_by(|a, b| b.1.count.cmp(&a.1.count));

    if gaps.is_empty() {
        out.push_str("  \x01G✓ No gaps detected — all operations handled!\x01W\n");
    } else {
        out.push_str(&format!("  \x01RFound {} unhandled/stubbed operations:\x01W\n\n", gaps.len()));
        out.push_str("  \x01YCategory      ID             Name                   Count  First RIP        Detail\x01W\n");
        for ((hx, id), stat) in &gaps {
            let duv = if stat.name.len() > 22 { &stat.name[..22] } else { &stat.name };
            let frt = if stat.last_detail.len() > 30 { &stat.last_detail[..30] } else { &stat.last_detail };
            out.push_str(&format!("  {}{:<14}\x01W0x{:<12X} {:<22} {:>5}  0x{:<14X} {}\n",
                stat.status.color_code(), hx.as_str(), id, duv,
                stat.count, stat.first_rip, frt));
        }
    }

    out.push('\n');
    out
}


pub fn ibo() -> String {
    let alz = CZ_.lock();
    let nz = match alz.as_ref() {
        Some(m) => m,
        None => return String::from("Debug monitor not initialized."),
    };

    let mut out = String::with_capacity(2048);
    out.push_str("\x01C═══ VM Debug: I/O Port Heatmap ═══\x01W\n\n");

    
    let aef = [
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

    for (start, end, name) in &aef {
        let ify: u64 = nz.stats.iter()
            .filter(|((hx, port), _)| matches!(hx, DebugCategory::IoPortIn) && *port >= *start && *port < *end)
            .map(|(_, j)| j.count)
            .sum();
        let isw: u64 = nz.stats.iter()
            .filter(|((hx, port), _)| matches!(hx, DebugCategory::IoPortOut) && *port >= *start && *port < *end)
            .map(|(_, j)| j.count)
            .sum();
        
        if ify > 0 || isw > 0 {
            let jwk = nz.stats.iter()
                .any(|((hx, port), j)| 
                    matches!(hx, DebugCategory::IoPortIn | DebugCategory::IoPortOut)
                    && *port >= *start && *port < *end
                    && matches!(j.status, HandleStatus::Unhandled));
            let status = if jwk { "\x01RMISS\x01W" } else { "\x01GOK\x01W" };
            out.push_str(&format!("  0x{:04X}-0x{:04X} {:<20} {:>8}   {:>8}   {}\n",
                start, end - 1, name, ify, isw, status));
        }
    }

    
    let jpc: Vec<_> = nz.stats.iter()
        .filter(|((hx, port), j)| 
            matches!(hx, DebugCategory::IoPortIn | DebugCategory::IoPortOut)
            && !aef.iter().any(|(start, end, _)| *port >= *start && *port < *end))
        .collect();
    
    if !jpc.is_empty() {
        out.push_str("\n  \x01R── Unknown Ports ──\x01W\n");
        for ((hx, port), stat) in &jpc {
            let it = if matches!(hx, DebugCategory::IoPortIn) { "IN " } else { "OUT" };
            out.push_str(&format!("  {} 0x{:04X}  {} ({}x) RIP=0x{:X}\n",
                it, port, stat.name, stat.count, stat.last_rip));
        }
    }

    out.push('\n');
    out
}


pub fn ibq() -> String {
    let alz = CZ_.lock();
    let nz = match alz.as_ref() {
        Some(m) => m,
        None => return String::from("Debug monitor not initialized."),
    };

    let mut out = String::with_capacity(2048);
    out.push_str("\x01C═══ VM Debug: MSR Access Report ═══\x01W\n\n");

    let mut gik: Vec<_> = nz.stats.iter()
        .filter(|((hx, _), _)| matches!(hx, DebugCategory::MsrRead | DebugCategory::MsrWrite))
        .collect();
    gik.sort_by_key(|((_, msr), _)| *msr);

    if gik.is_empty() {
        out.push_str("  No MSR accesses recorded.\n");
    } else {
        out.push_str("  \x01YDir     MSR            Name                        Count  Value/Detail           Status\x01W\n");
        for ((hx, msr), stat) in &gik {
            let it = if matches!(hx, DebugCategory::MsrRead) { "READ " } else { "WRITE" };
            let duv = if stat.name.len() > 26 { &stat.name[..26] } else { &stat.name };
            let frt = if stat.last_detail.len() > 20 { &stat.last_detail[..20] } else { &stat.last_detail };
            out.push_str(&format!("  {}  0x{:08X}     {:<26} {:>5}  {:<20}   {}{}\x01W\n",
                it, msr, duv, stat.count, frt,
                stat.status.color_code(), stat.status.as_str()));
        }
    }

    out.push('\n');
    out
}


pub fn ibz(count: usize) -> String {
    let alz = CZ_.lock();
    let nz = match alz.as_ref() {
        Some(m) => m,
        None => return String::from("Debug monitor not initialized."),
    };

    let mut out = String::with_capacity(2048);
    out.push_str("\x01C═══ VM Debug: Exit Timeline ═══\x01W\n\n");

    if nz.timeline.is_empty() {
        out.push_str("  No events recorded yet.\n");
    } else {
        let lfj = count.min(nz.timeline.len());
        let start = nz.timeline.len() - lfj;

        out.push_str("  \x01YExit#      Category      ID             RIP              Name                   Status\x01W\n");
        for entry in &nz.timeline[start..] {
            let name = ifo(entry.category, entry.identifier);
            let duv = if name.len() > 22 { &name[..22] } else { &name };
            out.push_str(&format!("  {:>8}   {:<14}0x{:<12X} 0x{:<14X} {:<22} {}{}\x01W\n",
                entry.exit_number, entry.category.as_str(),
                entry.identifier, entry.guest_rip,
                duv,
                entry.status.color_code(), entry.status.as_str()));
        }
    }

    out.push('\n');
    out
}


pub fn reset() {
    if let Some(ref mut nz) = *CZ_.lock() {
        nz.stats.clear();
        nz.timeline.clear();
        nz.timeline_pos = 0;
        nz.category_counts.clear();
        nz.unhandled_counts.clear();
        nz.gaps.clear();
        FJ_.store(0, Ordering::Relaxed);
    }
}


pub fn fdf() -> u64 {
    FJ_.load(Ordering::Relaxed)
}


pub fn fdw() -> u64 {
    let alz = CZ_.lock();
    match alz.as_ref() {
        Some(nz) => nz.unhandled_counts.values().sum(),
        None => 0,
    }
}






fn ifo(category: DebugCategory, id: u64) -> String {
    match category {
        DebugCategory::IoPortIn | DebugCategory::IoPortOut => mno(id as u16),
        DebugCategory::MsrRead | DebugCategory::MsrWrite => mnq(id as u32),
        DebugCategory::CpuidLeaf => format!("CPUID leaf 0x{:X}", id),
        DebugCategory::NpfFault => mnp(id),
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

fn mno(port: u16) -> String {
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

fn mnq(msr: u32) -> String {
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

fn mnp(gm: u64) -> String {
    match gm {
        0x0000..=0x0FFF => String::from("Real-mode IVT/BDA"),
        0xA0000..=0xBFFFF => String::from("VGA Frame Buffer"),
        0xC0000..=0xDFFFF => String::from("ROM/BIOS Shadow"),
        0xE0000..=0xFFFFF => String::from("High BIOS"),
        0xFEC0_0000..=0xFEC0_0FFF => String::from("I/O APIC"),
        0xFED0_0000..=0xFED0_03FF => String::from("HPET"),
        0xFEE0_0000..=0xFEE0_0FFF => String::from("Local APIC"),
        _ if gm >= 0x1_0000_0000 => String::from("High MMIO (>4GB)"),
        _ => format!("GPA 0x{:X}", gm),
    }
}






pub fn is_initialized() -> bool {
    CZ_.lock().is_some()
}


pub fn pzh(hx: DebugCategory) -> u64 {
    let alz = CZ_.lock();
    match alz.as_ref() {
        Some(nz) => nz.category_counts.get(&hx).copied().unwrap_or(0),
        None => 0,
    }
}


pub fn rav() -> usize {
    let alz = CZ_.lock();
    match alz.as_ref() {
        Some(nz) => nz.stats.len(),
        None => 0,
    }
}
