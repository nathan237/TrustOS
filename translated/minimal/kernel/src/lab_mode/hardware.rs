









extern crate alloc;

use alloc::string::String;
use alloc::format;
use super::{eh, bly, ew, qu,
            P_, F_, AC_, AK_, AN_, M_, NF_};


pub struct HardwareState {
    
    pub scroll: usize,
    
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
    
    
    pub fn update(&mut self) {
        self.refresh_counter += 1;
        if self.refresh_counter % 15 != 0 {
            return;
        }
        
        
        self.cpu_pct = crate::devtools::kyu();
        
        
        let mem = crate::devtools::dbe();
        self.heap_used = mem.current_heap_used;
        self.heap_total = mem.heap_total;
        self.alloc_count = mem.alloc_count;
        self.dealloc_count = mem.dealloc_count;
        self.live_allocs = mem.live_allocs;
        self.peak_heap = mem.peak_heap_used;
        self.largest_alloc = mem.largest_alloc;
        self.frag_pct = mem.fragmentation_pct;
        
        
        self.total_phys_mb = crate::memory::ceo() / (1024 * 1024);
        
        
        self.irq_rate = crate::devtools::irq_rate();
        
        
        self.uptime_secs = crate::time::uptime_secs();
        
        
        let pmn = crate::trace::stats();
        self.ctx_switches = pmn.events_recorded;
        
        
        self.task_count = crate::scheduler::stats().ready_count;
    }
    
    
    pub fn force_refresh(&mut self) {
        self.refresh_counter = 14;
    }
    
    pub fn handle_key(&mut self, key: u8) {
        use crate::keyboard::{T_, S_};
        match key {
            T_ => self.scroll = self.scroll.saturating_sub(1),
            S_ => self.scroll += 1,
            _ => {}
        }
    }

    
    pub fn handle_click(&mut self, _local_x: i32, ta: i32, _w: u32, _h: u32) {
        let ee = qu() + 2;
        if ee <= 0 { return; }
        
        let row = (ta / ee) as usize;
        if row > 2 {
            
            self.scroll = row.saturating_sub(3);
        }
    }
}


pub fn draw(state: &HardwareState, x: i32, y: i32, w: u32, h: u32) {
    let aq = ew();
    let ee = qu() + 2; 
    let hs = 8u32;
    
    let nd = if aq > 0 { (w as i32 / aq) as usize } else { 30 };
    let mut u = y;
    
    
    eh(x, u, "CPU", M_);
    let hof = format!("{}%", state.cpu_pct);
    let hoe = if state.cpu_pct > 80 { AN_ } else if state.cpu_pct > 50 { AK_ } else { AC_ };
    let bhn = x + w as i32 - (hof.len() as i32 * aq) - 4;
    eh(bhn, u, &hof, hoe);
    u += ee;
    bly(x, u, w.saturating_sub(4), hs, state.cpu_pct, hoe, 0xFF21262D);
    u += hs as i32 + ee / 2;
    
    
    eh(x, u, "Heap", M_);
    let haw = state.heap_used / (1024 * 1024);
    let total_mb = state.heap_total / (1024 * 1024);
    let heap_pct = if state.heap_total > 0 {
        ((state.heap_used as u64 * 100) / state.heap_total as u64) as u32
    } else { 0 };
    let iem = format!("{}/{}MB ({}%)", haw, total_mb, heap_pct);
    let mlv = x + w as i32 - (iem.len() as i32 * aq) - 4;
    let iel = if heap_pct > 85 { AN_ } else if heap_pct > 60 { AK_ } else { AC_ };
    eh(mlv, u, &iem, iel);
    u += ee;
    bly(x, u, w.saturating_sub(4), hs, heap_pct, iel, 0xFF21262D);
    u += hs as i32 + ee / 2;
    
    
    let stats: [(&str, String, u32); 10] = [
        ("Uptime", hzr(state.uptime_secs), P_),
        ("Physical RAM", format!("{} MB", state.total_phys_mb), P_),
        ("IRQ Rate", format!("{}/sec", state.irq_rate), AK_),
        ("Tasks", format!("{}", state.task_count), P_),
        ("Trace Events", format!("{}", state.ctx_switches), F_),
        ("Live Allocs", format!("{}", state.live_allocs), AC_),
        ("Total Allocs", format!("{}", state.alloc_count), F_),
        ("Peak Heap", format!("{} KB", state.peak_heap / 1024), P_),
        ("Largest Alloc", cxz(state.largest_alloc), P_),
        ("Fragmentation", format!("{:.1}%", state.frag_pct), 
            if state.frag_pct > 50.0 { AN_ } else { F_ }),
    ];
    
    let visible = ((h as i32 - (u - y)) / ee) as usize;
    let start = state.scroll.min(stats.len().saturating_sub(1));
    let end = (start + visible).min(stats.len());
    
    for i in start..end {
        let (label, ref value, color) = stats[i];
        eh(x + 4, u, label, F_);
        let vx = x + w as i32 - (value.len() as i32 * aq) - 4;
        eh(vx, u, value, color);
        u += ee;
    }
}

fn hzr(im: u64) -> String {
    let h = im / 3600;
    let m = (im % 3600) / 60;
    let j = im % 60;
    if h > 0 {
        format!("{}h {:02}m {:02}s", h, m, j)
    } else if m > 0 {
        format!("{}m {:02}s", m, j)
    } else {
        format!("{}s", j)
    }
}

fn cxz(bytes: usize) -> String {
    if bytes >= 1024 * 1024 {
        format!("{} MB", bytes / (1024 * 1024))
    } else if bytes >= 1024 {
        format!("{} KB", bytes / 1024)
    } else {
        format!("{} B", bytes)
    }
}
