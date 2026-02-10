//! Developer Tools for TrustOS
//! 
//! Comprehensive tooling for OS developers:
//! - **perf**: TSC-precision profiler, IRQ stats, CPU utilization
//! - **dmesg**: Kernel ring buffer for post-boot log review
//! - **memdbg**: Allocation tracking, peak usage, fragmentation
//! - **peek/poke**: Memory inspector and register dump
//! - **devpanel**: Real-time overlay (FPS, heap, CPU%, IRQ/s)

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use core::sync::atomic::{AtomicU64, AtomicBool, AtomicUsize, Ordering};
use spin::Mutex;

// ═══════════════════════════════════════════════════════════════════════════════
// 1. DMESG — Kernel Ring Buffer
// ═══════════════════════════════════════════════════════════════════════════════

/// Ring buffer entry
#[derive(Clone)]
struct DmesgEntry {
    timestamp_ms: u64,
    level: u8,       // 0=trace..5=fatal
    message: String,
}

/// Ring buffer size (keep last N messages)
const DMESG_CAPACITY: usize = 2048;

struct DmesgBuffer {
    entries: Vec<DmesgEntry>,
    /// Write index (wraps around)
    write_idx: usize,
    /// Total messages ever written
    total_count: u64,
    /// Whether buffer has wrapped
    wrapped: bool,
}

impl DmesgBuffer {
    const fn new() -> Self {
        Self {
            entries: Vec::new(),
            write_idx: 0,
            total_count: 0,
            wrapped: false,
        }
    }
    
    fn push(&mut self, timestamp_ms: u64, level: u8, message: String) {
        if self.entries.len() < DMESG_CAPACITY {
            self.entries.push(DmesgEntry { timestamp_ms, level, message });
        } else {
            self.entries[self.write_idx] = DmesgEntry { timestamp_ms, level, message };
            self.wrapped = true;
        }
        self.write_idx = (self.write_idx + 1) % DMESG_CAPACITY;
        self.total_count += 1;
    }
    
    /// Iterate entries in chronological order
    fn iter_ordered(&self) -> impl Iterator<Item = &DmesgEntry> {
        let start = if self.wrapped { self.write_idx } else { 0 };
        let len = self.entries.len();
        (0..len).map(move |i| &self.entries[(start + i) % len])
    }
    
    fn len(&self) -> usize {
        self.entries.len()
    }
}

static DMESG: Mutex<DmesgBuffer> = Mutex::new(DmesgBuffer::new());

/// Record a message into the kernel ring buffer.
/// Called from the logging macros or boot code.
pub fn dmesg_write(level: u8, message: String) {
    // Safely get uptime (returns 0 if time not initialized yet)
    let ts = crate::time::uptime_ms();
    if let Some(mut buf) = DMESG.try_lock() {
        buf.push(ts, level, message);
    }
}

/// Record a message (from &str, avoids allocation in some paths)
pub fn dmesg_record(level: u8, msg: &str) {
    dmesg_write(level, String::from(msg));
}

/// Get the last N dmesg lines (or all if n=0)
pub fn dmesg_read(n: usize) -> Vec<String> {
    let buf = DMESG.lock();
    let entries: Vec<_> = buf.iter_ordered().collect();
    let start = if n > 0 && n < entries.len() { entries.len() - n } else { 0 };
    entries[start..]
        .iter()
        .map(|e| {
            let level = match e.level {
                0 => "TRACE",
                1 => "DEBUG",
                2 => "INFO ",
                3 => "WARN ",
                4 => "ERROR",
                5 => "FATAL",
                _ => "?????",
            };
            format!("[{:>10.3}] [{}] {}", 
                e.timestamp_ms as f64 / 1000.0, level, e.message)
        })
        .collect()
}

/// Get dmesg stats
pub fn dmesg_stats() -> (usize, u64) {
    let buf = DMESG.lock();
    (buf.len(), buf.total_count)
}

// ═══════════════════════════════════════════════════════════════════════════════
// 2. MEMDBG — Allocation Tracking
// ═══════════════════════════════════════════════════════════════════════════════

/// Allocation counters (updated by the global allocator wrapper)
static ALLOC_COUNT: AtomicU64 = AtomicU64::new(0);
static DEALLOC_COUNT: AtomicU64 = AtomicU64::new(0);
static ALLOC_BYTES_TOTAL: AtomicU64 = AtomicU64::new(0);
static DEALLOC_BYTES_TOTAL: AtomicU64 = AtomicU64::new(0);
static PEAK_HEAP_USED: AtomicUsize = AtomicUsize::new(0);
static CURRENT_LIVE_ALLOCS: AtomicU64 = AtomicU64::new(0);
/// Largest single allocation ever
static LARGEST_ALLOC: AtomicUsize = AtomicUsize::new(0);

/// Called when an allocation happens
pub fn track_alloc(size: usize) {
    ALLOC_COUNT.fetch_add(1, Ordering::Relaxed);
    ALLOC_BYTES_TOTAL.fetch_add(size as u64, Ordering::Relaxed);
    CURRENT_LIVE_ALLOCS.fetch_add(1, Ordering::Relaxed);
    
    // Update peak
    let used = crate::memory::heap::used();
    let _ = PEAK_HEAP_USED.fetch_max(used, Ordering::Relaxed);
    
    // Update largest alloc
    let _ = LARGEST_ALLOC.fetch_max(size, Ordering::Relaxed);
}

/// Called when a deallocation happens
pub fn track_dealloc(size: usize) {
    DEALLOC_COUNT.fetch_add(1, Ordering::Relaxed);
    DEALLOC_BYTES_TOTAL.fetch_add(size as u64, Ordering::Relaxed);
    CURRENT_LIVE_ALLOCS.fetch_sub(1, Ordering::Relaxed);
}

/// Memory debug stats
pub struct MemDbgStats {
    pub alloc_count: u64,
    pub dealloc_count: u64,
    pub alloc_bytes_total: u64,
    pub dealloc_bytes_total: u64,
    pub peak_heap_used: usize,
    pub current_heap_used: usize,
    pub current_heap_free: usize,
    pub heap_total: usize,
    pub live_allocs: u64,
    pub largest_alloc: usize,
    pub fragmentation_pct: f32,
}

/// Get comprehensive memory debug stats
pub fn memdbg_stats() -> MemDbgStats {
    let used = crate::memory::heap::used();
    let free = crate::memory::heap::free();
    let total = used + free;
    
    // Simple fragmentation estimate:
    // If we have lots of free space but can't allocate large blocks,
    // that's fragmentation. We approximate by checking dealloc/alloc ratio.
    let alloc_count = ALLOC_COUNT.load(Ordering::Relaxed);
    let dealloc_count = DEALLOC_COUNT.load(Ordering::Relaxed);
    let frag = if total > 0 && alloc_count > 100 {
        // Fragmentation heuristic: ratio of deallocated operations to total,
        // weighted by how much free space is scattered
        let churn = if alloc_count > 0 {
            (dealloc_count as f32) / (alloc_count as f32)
        } else {
            0.0
        };
        // High churn + low free = high fragmentation
        let free_ratio = free as f32 / total as f32;
        (churn * (1.0 - free_ratio) * 100.0).min(100.0)
    } else {
        0.0
    };
    
    MemDbgStats {
        alloc_count,
        dealloc_count,
        alloc_bytes_total: ALLOC_BYTES_TOTAL.load(Ordering::Relaxed),
        dealloc_bytes_total: DEALLOC_BYTES_TOTAL.load(Ordering::Relaxed),
        peak_heap_used: PEAK_HEAP_USED.load(Ordering::Relaxed),
        current_heap_used: used,
        current_heap_free: free,
        heap_total: total,
        live_allocs: CURRENT_LIVE_ALLOCS.load(Ordering::Relaxed),
        largest_alloc: LARGEST_ALLOC.load(Ordering::Relaxed),
        fragmentation_pct: frag,
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// 3. PERF — Profiler & System Stats
// ═══════════════════════════════════════════════════════════════════════════════

/// IRQ rate tracking
static LAST_IRQ_SNAPSHOT: AtomicU64 = AtomicU64::new(0);
static LAST_IRQ_TIME_MS: AtomicU64 = AtomicU64::new(0);
static IRQ_RATE: AtomicU64 = AtomicU64::new(0);

/// CPU idle time tracking (approximation via HLT cycles)
static IDLE_CYCLES: AtomicU64 = AtomicU64::new(0);
static BUSY_CYCLES: AtomicU64 = AtomicU64::new(0);

/// Record idle cycles (called from HLT loop / frame limiter)
pub fn record_idle_cycles(cycles: u64) {
    IDLE_CYCLES.fetch_add(cycles, Ordering::Relaxed);
}

/// Record busy cycles (called from frame render)
pub fn record_busy_cycles(cycles: u64) {
    BUSY_CYCLES.fetch_add(cycles, Ordering::Relaxed);
}

/// CPU utilization percentage (0-100)
pub fn cpu_utilization() -> u32 {
    let idle = IDLE_CYCLES.swap(0, Ordering::Relaxed);
    let busy = BUSY_CYCLES.swap(0, Ordering::Relaxed);
    let total = idle + busy;
    if total == 0 { return 0; }
    ((busy * 100) / total) as u32
}

/// Get IRQ rate snapshot (call periodically)
pub fn update_irq_rate() {
    let stats = crate::sync::percpu::all_cpu_stats();
    let total_irqs: u64 = stats.iter().map(|s| s.interrupts).sum();
    let now = crate::time::uptime_ms();
    
    let last_irqs = LAST_IRQ_SNAPSHOT.swap(total_irqs, Ordering::Relaxed);
    let last_time = LAST_IRQ_TIME_MS.swap(now, Ordering::Relaxed);
    
    let dt = now.saturating_sub(last_time);
    if dt > 0 {
        let rate = (total_irqs - last_irqs) * 1000 / dt;
        IRQ_RATE.store(rate, Ordering::Relaxed);
    }
}

/// Get current IRQ/sec rate
pub fn irq_rate() -> u64 {
    IRQ_RATE.load(Ordering::Relaxed)
}

/// Per-CPU detailed stats
pub struct PerfSnapshot {
    pub uptime_ms: u64,
    pub cpu_stats: Vec<crate::sync::percpu::CpuStats>,
    pub total_irqs: u64,
    pub total_syscalls: u64,
    pub total_ctx_switches: u64,
    pub irq_per_sec: u64,
    pub heap_used: usize,
    pub heap_free: usize,
    pub fps: u64,
}

/// Take a performance snapshot
pub fn perf_snapshot() -> PerfSnapshot {
    let stats = crate::sync::percpu::all_cpu_stats();
    let total_irqs: u64 = stats.iter().map(|s| s.interrupts).sum();
    let total_syscalls: u64 = stats.iter().map(|s| s.syscalls).sum();
    let total_ctx_switches: u64 = stats.iter().map(|s| s.context_switches).sum();
    
    PerfSnapshot {
        uptime_ms: crate::time::uptime_ms(),
        cpu_stats: stats,
        total_irqs,
        total_syscalls,
        total_ctx_switches,
        irq_per_sec: irq_rate(),
        heap_used: crate::memory::heap::used(),
        heap_free: crate::memory::heap::free(),
        fps: crate::gui::engine::get_fps(),
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// 4. PEEK/POKE — Memory Inspector
// ═══════════════════════════════════════════════════════════════════════════════

/// Read memory at virtual address, returning hex dump
pub fn peek(addr: usize, count: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let count = count.min(256); // Max 256 bytes per peek
    
    // Safety check: don't read from NULL or obviously bad addresses
    if addr == 0 {
        lines.push(String::from("Error: NULL pointer"));
        return lines;
    }
    
    let bytes_per_line = 16;
    let mut offset = 0;
    
    while offset < count {
        let line_bytes = (count - offset).min(bytes_per_line);
        let mut hex = String::new();
        let mut ascii = String::new();
        
        for i in 0..bytes_per_line {
            if i < line_bytes {
                let byte = unsafe {
                    // Read with volatile to avoid optimization
                    core::ptr::read_volatile((addr + offset + i) as *const u8)
                };
                hex.push_str(&format!("{:02x} ", byte));
                ascii.push(if byte >= 0x20 && byte < 0x7F { byte as char } else { '.' });
            } else {
                hex.push_str("   ");
                ascii.push(' ');
            }
            if i == 7 { hex.push(' '); }
        }
        
        lines.push(format!("  {:016x}  {}|{}|", addr + offset, hex, ascii));
        offset += bytes_per_line;
    }
    
    lines
}

/// Write a byte to a virtual address (very dangerous!)
pub fn poke(addr: usize, value: u8) -> Result<(), &'static str> {
    if addr == 0 {
        return Err("NULL pointer");
    }
    if addr < 0x1000 {
        return Err("Address too low (first page guard)");
    }
    
    unsafe {
        core::ptr::write_volatile(addr as *mut u8, value);
    }
    Ok(())
}

/// Dump CPU registers (snapshot of current state)
pub fn cpu_registers() -> Vec<String> {
    let mut regs = Vec::new();
    
    let rsp: u64;
    let rbp: u64;
    let rflags: u64;
    let cr0: u64;
    let cr3: u64;
    let cr4: u64;
    
    unsafe {
        core::arch::asm!("mov {}, rsp", out(reg) rsp);
        core::arch::asm!("mov {}, rbp", out(reg) rbp);
        core::arch::asm!("pushfq; pop {}", out(reg) rflags);
        core::arch::asm!("mov {}, cr0", out(reg) cr0);
        core::arch::asm!("mov {}, cr3", out(reg) cr3);
        core::arch::asm!("mov {}, cr4", out(reg) cr4);
    }
    
    regs.push(String::from("  CPU Register Snapshot"));
    regs.push(String::from("  ─────────────────────────────────────"));
    regs.push(format!("  RSP    = 0x{:016x}  (stack pointer)", rsp));
    regs.push(format!("  RBP    = 0x{:016x}  (base pointer)", rbp));
    regs.push(format!("  RFLAGS = 0x{:016x}", rflags));
    regs.push(String::from(""));
    regs.push(format!("  CR0    = 0x{:016x}", cr0));
    
    // Decode CR0 flags
    let mut cr0_flags = Vec::new();
    if cr0 & 1 != 0 { cr0_flags.push("PE"); }
    if cr0 & (1 << 1) != 0 { cr0_flags.push("MP"); }
    if cr0 & (1 << 4) != 0 { cr0_flags.push("ET"); }
    if cr0 & (1 << 5) != 0 { cr0_flags.push("NE"); }
    if cr0 & (1 << 16) != 0 { cr0_flags.push("WP"); }
    if cr0 & (1 << 31) != 0 { cr0_flags.push("PG"); }
    regs.push(format!("           [{}]", cr0_flags.join(" | ")));
    
    regs.push(format!("  CR3    = 0x{:016x}  (page table root)", cr3));
    regs.push(format!("  CR4    = 0x{:016x}", cr4));
    
    // Decode CR4 flags
    let mut cr4_flags = Vec::new();
    if cr4 & (1 << 5) != 0 { cr4_flags.push("PAE"); }
    if cr4 & (1 << 7) != 0 { cr4_flags.push("PGE"); }
    if cr4 & (1 << 9) != 0 { cr4_flags.push("OSFXSR"); }
    if cr4 & (1 << 10) != 0 { cr4_flags.push("OSXMMEXCPT"); }
    if cr4 & (1 << 13) != 0 { cr4_flags.push("VMXE"); }
    if cr4 & (1 << 18) != 0 { cr4_flags.push("OSXSAVE"); }
    if cr4 & (1 << 20) != 0 { cr4_flags.push("SMEP"); }
    if cr4 & (1 << 21) != 0 { cr4_flags.push("SMAP"); }
    regs.push(format!("           [{}]", cr4_flags.join(" | ")));
    
    // EFER MSR
    let efer: u64 = unsafe {
        let lo: u32;
        let hi: u32;
        core::arch::asm!(
            "rdmsr",
            in("ecx") 0xC000_0080u32,
            out("eax") lo,
            out("edx") hi,
        );
        (hi as u64) << 32 | lo as u64
    };
    regs.push(format!("  EFER   = 0x{:016x}", efer));
    let mut efer_flags = Vec::new();
    if efer & 1 != 0 { efer_flags.push("SCE"); }
    if efer & (1 << 8) != 0 { efer_flags.push("LME"); }
    if efer & (1 << 10) != 0 { efer_flags.push("LMA"); }
    if efer & (1 << 11) != 0 { efer_flags.push("NXE"); }
    regs.push(format!("           [{}]", efer_flags.join(" | ")));
    
    regs
}

// ═══════════════════════════════════════════════════════════════════════════════
// 5. DEVPANEL — Real-time overlay state
// ═══════════════════════════════════════════════════════════════════════════════

/// Whether the dev panel overlay is visible
static DEVPANEL_VISIBLE: AtomicBool = AtomicBool::new(false);

/// Toggle devpanel visibility (F12 or `devpanel` command)
pub fn toggle_devpanel() {
    let prev = DEVPANEL_VISIBLE.load(Ordering::Relaxed);
    DEVPANEL_VISIBLE.store(!prev, Ordering::Relaxed);
}

pub fn set_devpanel_visible(visible: bool) {
    DEVPANEL_VISIBLE.store(visible, Ordering::Relaxed);
}

pub fn is_devpanel_visible() -> bool {
    DEVPANEL_VISIBLE.load(Ordering::Relaxed)
}

/// DevPanel data snapshot (computed once per frame, displayed in overlay)
pub struct DevPanelData {
    pub fps: u64,
    pub frame_time_us: u64,
    pub heap_used_kb: usize,
    pub heap_total_kb: usize,
    pub heap_pct: u32,
    pub live_allocs: u64,
    pub irq_per_sec: u64,
    pub uptime_secs: u64,
    pub cpu_count: usize,
    pub total_irqs: u64,
}

/// Collect data for the devpanel overlay
pub fn devpanel_data() -> DevPanelData {
    let used = crate::memory::heap::used();
    let free = crate::memory::heap::free();
    let total = used + free;
    let stats = crate::sync::percpu::all_cpu_stats();
    let total_irqs: u64 = stats.iter().map(|s| s.interrupts).sum();
    
    DevPanelData {
        fps: crate::gui::engine::get_fps(),
        frame_time_us: 0, // will be filled by renderer
        heap_used_kb: used / 1024,
        heap_total_kb: total / 1024,
        heap_pct: if total > 0 { ((used * 100) / total) as u32 } else { 0 },
        live_allocs: CURRENT_LIVE_ALLOCS.load(Ordering::Relaxed),
        irq_per_sec: irq_rate(),
        uptime_secs: crate::time::uptime_ms() / 1000,
        cpu_count: stats.len().max(1),
        total_irqs,
    }
}

/// Render the devpanel overlay onto the framebuffer.
/// Called from desktop render loop when visible.
pub fn render_devpanel(width: u32, _height: u32, frame_time_us: u64) {
    if !is_devpanel_visible() {
        return;
    }
    
    // Update IRQ rate sampling
    update_irq_rate();
    
    let data = devpanel_data();
    
    // ── Panel dimensions ────────────────────────────────────
    let panel_w = 260u32;
    let panel_h = 180u32;
    let panel_x = (width - panel_w - 8) as i32;
    let panel_y = 8i32;
    
    // Semi-transparent dark background (approximate with solid dark)
    let bg_color: u32 = 0xFF101018;
    let border_color: u32 = 0xFF00FF88;
    let title_color: u32 = 0xFF00FFAA;
    let label_color: u32 = 0xFF88AACC;
    let value_color: u32 = 0xFFFFFFFF;
    let bar_bg: u32 = 0xFF333344;
    
    // Draw background
    crate::framebuffer::fill_rect(panel_x as u32, panel_y as u32, panel_w, panel_h, bg_color);
    
    // Border
    crate::framebuffer::draw_hline(panel_x as u32, panel_y as u32, panel_w, border_color);
    crate::framebuffer::draw_hline(panel_x as u32, (panel_y + panel_h as i32 - 1) as u32, panel_w, border_color);
    crate::framebuffer::draw_vline(panel_x as u32, panel_y as u32, panel_h, border_color);
    crate::framebuffer::draw_vline((panel_x + panel_w as i32 - 1) as u32, panel_y as u32, panel_h, border_color);
    
    let x = panel_x + 8;
    let mut y = panel_y + 6;
    
    // Title
    draw_overlay_text(x, y, "DEVPANEL [F12]", title_color);
    y += 14;
    
    // Separator
    crate::framebuffer::draw_hline((panel_x + 4) as u32, y as u32, panel_w - 8, 0xFF444466);
    y += 6;
    
    // FPS + frame time
    let ft = if frame_time_us > 0 { frame_time_us } else { 16666 };
    draw_overlay_text(x, y, &format!("FPS: {:<4}  Frame: {:.1}ms", data.fps, ft as f64 / 1000.0), value_color);
    y += 14;
    
    // Heap usage with bar
    draw_overlay_text(x, y, &format!("Heap: {} / {} KB ({}%)", 
        data.heap_used_kb, data.heap_total_kb, data.heap_pct), label_color);
    y += 12;
    
    // Heap bar
    let bar_w = (panel_w - 20) as u32;
    let bar_x = (x + 2) as u32;
    let bar_y = y as u32;
    crate::framebuffer::fill_rect(bar_x, bar_y, bar_w, 6, bar_bg);
    let filled = (bar_w * data.heap_pct) / 100;
    let bar_color = if data.heap_pct > 90 { 0xFFFF4444 }
        else if data.heap_pct > 70 { 0xFFFFAA44 }
        else { 0xFF44FF88 };
    crate::framebuffer::fill_rect(bar_x, bar_y, filled, 6, bar_color);
    y += 12;
    
    // Live allocations
    draw_overlay_text(x, y, &format!("Allocs: {} live", data.live_allocs), label_color);
    y += 14;
    
    // IRQ rate
    draw_overlay_text(x, y, &format!("IRQ/s: {}   Total: {}", data.irq_per_sec, data.total_irqs), label_color);
    y += 14;
    
    // CPUs
    draw_overlay_text(x, y, &format!("CPUs: {}   Uptime: {}s", data.cpu_count, data.uptime_secs), label_color);
    y += 14;
    
    // Per-CPU IRQ breakdown
    let stats = crate::sync::percpu::all_cpu_stats();
    for (i, s) in stats.iter().enumerate().take(4) {
        let idle_str = if s.is_idle { "idle" } else { "busy" };
        draw_overlay_text(x, y, &format!("  CPU{}: {} irqs [{}]", i, s.interrupts, idle_str), 
            if s.is_idle { 0xFF667788 } else { 0xFF88FFAA });
        y += 12;
    }
}

/// Draw text on the overlay (uses framebuffer's font renderer)
fn draw_overlay_text(x: i32, y: i32, text: &str, color: u32) {
    let x = x as u32;
    let y = y as u32;
    for (i, c) in text.chars().enumerate() {
        crate::framebuffer::draw_char_at(x + (i as u32 * 8), y, c, color);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// 6. BOOT MESSAGE CAPTURE — Hook into serial/println for dmesg
// ═══════════════════════════════════════════════════════════════════════════════

/// Capture a boot message into dmesg
pub fn capture_boot_message(msg: &str) {
    dmesg_record(2, msg); // level 2 = INFO
}

/// Macro-friendly capture — only captures after heap is available
pub fn capture_serial_line(msg: core::fmt::Arguments) {
    // Don't attempt to allocate before the heap is ready.
    // Check if the heap has been initialized (free > 0 means it's set up).
    if crate::memory::heap::free() == 0 {
        return;
    }
    let s = format!("{}", msg);
    dmesg_record(2, &s);
}
