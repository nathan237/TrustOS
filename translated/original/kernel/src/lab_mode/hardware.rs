//! Hardware Status Panel — Real-time CPU, memory, IRQ, and timer stats
//!
//! Shows live hardware telemetry:
//! - CPU utilization gauge  
//! - Heap usage bar (used / total)
//! - IRQ rate (interrupts/sec)
//! - Uptime
//! - Scheduler info (tasks, context switches)
//! - Memory allocator stats

extern crate alloc;

use alloc::string::String;
use alloc::format;
use super::{draw_lab_text, draw_progress_bar, char_w, char_h,
            COL_TEXT, COL_DIM, COL_GREEN, COL_YELLOW, COL_RED, COL_ACCENT, COL_PANEL_BG};

/// Hardware panel state
pub struct HardwareState {
    /// Scroll offset for stats list
    pub scroll: usize,
    /// Cached values (refreshed each tick)
    pub cpu_pct: u32,
    pub heap_used: usize,
    pub heap_total: usize,
    pub irq_rate: u64,
    pub uptime_secs: u64,
    pub task_count: usize,
    pub alloc_count: u64,
    pub dealloc_count: u64,
    pub live_allocs: u64,
    pub peak_heap: usize,
    pub largest_alloc: usize,
    pub frag_pct: f32,
    pub total_phys_mb: u64,
    pub ctx_switches: u64,
    /// Refresh counter
    refresh_counter: u64,
}

impl HardwareState {
    pub fn new() -> Self {
        Self {
            scroll: 0,
            cpu_pct: 0,
            heap_used: 0,
            heap_total: 0,
            irq_rate: 0,
            uptime_secs: 0,
            task_count: 0,
            alloc_count: 0,
            dealloc_count: 0,
            live_allocs: 0,
            peak_heap: 0,
            largest_alloc: 0,
            frag_pct: 0.0,
            total_phys_mb: 0,
            ctx_switches: 0,
            refresh_counter: 0,
        }
    }
    
    /// Update cached values (called each frame, but only refresh every ~15 frames)
    pub fn update(&mut self) {
        self.refresh_counter += 1;
        if self.refresh_counter % 15 != 0 {
            return;
        }
        
        // CPU
        self.cpu_pct = crate::devtools::cpu_utilization();
        
        // Memory
        let mem = crate::devtools::memdbg_stats();
        self.heap_used = mem.current_heap_used;
        self.heap_total = mem.heap_total;
        self.alloc_count = mem.alloc_count;
        self.dealloc_count = mem.dealloc_count;
        self.live_allocs = mem.live_allocs;
        self.peak_heap = mem.peak_heap_used;
        self.largest_alloc = mem.largest_alloc;
        self.frag_pct = mem.fragmentation_pct;
        
        // Physical memory
        self.total_phys_mb = crate::memory::total_physical_memory() / (1024 * 1024);
        
        // IRQ
        self.irq_rate = crate::devtools::irq_rate();
        
        // Uptime
        self.uptime_secs = crate::time::uptime_secs();
        
        // Scheduler stats
        let trace_stats = crate::trace::stats();
        self.ctx_switches = trace_stats.events_recorded;
        
        // Tasks
        self.task_count = crate::scheduler::stats().ready_count;
    }
    
    /// Force an immediate refresh next tick
    pub fn force_refresh(&mut self) {
        self.refresh_counter = 14;
    }
    
    pub fn handle_key(&mut self, key: u8) {
        use crate::keyboard::{KEY_UP, KEY_DOWN};
        match key {
            KEY_UP => self.scroll = self.scroll.saturating_sub(1),
            KEY_DOWN => self.scroll += 1,
            _ => {}
        }
    }

    /// Handle mouse click inside the panel content area
    pub fn handle_click(&mut self, _local_x: i32, local_y: i32, _w: u32, _h: u32) {
        let lh = char_h() + 2;
        if lh <= 0 { return; }
        // Click on a stat row → scroll to make that row visible at top
        let row = (local_y / lh) as usize;
        if row > 2 {
            // Skip CPU + Heap bars (first ~3 visual rows), scroll stats
            self.scroll = row.saturating_sub(3);
        }
    }
}

/// Draw the hardware status panel
pub fn draw(state: &HardwareState, x: i32, y: i32, w: u32, h: u32) {
    let cw = char_w();
    let lh = char_h() + 2; // line height with spacing
    let bar_h = 8u32;
    
    let max_chars = if cw > 0 { (w as i32 / cw) as usize } else { 30 };
    let mut cy = y;
    
    // ── CPU ──────────────────────────────────
    draw_lab_text(x, cy, "CPU", COL_ACCENT);
    let cpu_label = format!("{}%", state.cpu_pct);
    let cpu_color = if state.cpu_pct > 80 { COL_RED } else if state.cpu_pct > 50 { COL_YELLOW } else { COL_GREEN };
    let label_x = x + w as i32 - (cpu_label.len() as i32 * cw) - 4;
    draw_lab_text(label_x, cy, &cpu_label, cpu_color);
    cy += lh;
    draw_progress_bar(x, cy, w.saturating_sub(4), bar_h, state.cpu_pct, cpu_color, 0xFF21262D);
    cy += bar_h as i32 + lh / 2;
    
    // ── Heap ─────────────────────────────────
    draw_lab_text(x, cy, "Heap", COL_ACCENT);
    let used_mb = state.heap_used / (1024 * 1024);
    let total_mb = state.heap_total / (1024 * 1024);
    let heap_pct = if state.heap_total > 0 {
        ((state.heap_used as u64 * 100) / state.heap_total as u64) as u32
    } else { 0 };
    let heap_label = format!("{}/{}MB ({}%)", used_mb, total_mb, heap_pct);
    let hl_x = x + w as i32 - (heap_label.len() as i32 * cw) - 4;
    let heap_color = if heap_pct > 85 { COL_RED } else if heap_pct > 60 { COL_YELLOW } else { COL_GREEN };
    draw_lab_text(hl_x, cy, &heap_label, heap_color);
    cy += lh;
    draw_progress_bar(x, cy, w.saturating_sub(4), bar_h, heap_pct, heap_color, 0xFF21262D);
    cy += bar_h as i32 + lh / 2;
    
    // ── Stats table ──────────────────────────
    let stats: [(&str, String, u32); 10] = [
        ("Uptime", format_uptime(state.uptime_secs), COL_TEXT),
        ("Physical RAM", format!("{} MB", state.total_phys_mb), COL_TEXT),
        ("IRQ Rate", format!("{}/sec", state.irq_rate), COL_YELLOW),
        ("Tasks", format!("{}", state.task_count), COL_TEXT),
        ("Trace Events", format!("{}", state.ctx_switches), COL_DIM),
        ("Live Allocs", format!("{}", state.live_allocs), COL_GREEN),
        ("Total Allocs", format!("{}", state.alloc_count), COL_DIM),
        ("Peak Heap", format!("{} KB", state.peak_heap / 1024), COL_TEXT),
        ("Largest Alloc", format_bytes(state.largest_alloc), COL_TEXT),
        ("Fragmentation", format!("{:.1}%", state.frag_pct), 
            if state.frag_pct > 50.0 { COL_RED } else { COL_DIM }),
    ];
    
    let visible = ((h as i32 - (cy - y)) / lh) as usize;
    let start = state.scroll.min(stats.len().saturating_sub(1));
    let end = (start + visible).min(stats.len());
    
    for i in start..end {
        let (label, ref value, color) = stats[i];
        draw_lab_text(x + 4, cy, label, COL_DIM);
        let vx = x + w as i32 - (value.len() as i32 * cw) - 4;
        draw_lab_text(vx, cy, value, color);
        cy += lh;
    }
}

fn format_uptime(secs: u64) -> String {
    let h = secs / 3600;
    let m = (secs % 3600) / 60;
    let s = secs % 60;
    if h > 0 {
        format!("{}h {:02}m {:02}s", h, m, s)
    } else if m > 0 {
        format!("{}m {:02}s", m, s)
    } else {
        format!("{}s", s)
    }
}

fn format_bytes(bytes: usize) -> String {
    if bytes >= 1024 * 1024 {
        format!("{} MB", bytes / (1024 * 1024))
    } else if bytes >= 1024 {
        format!("{} KB", bytes / 1024)
    } else {
        format!("{} B", bytes)
    }
}
