








use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use core::sync::atomic::{AtomicU64, AtomicBool, AtomicUsize, Ordering};
use spin::Mutex;






#[derive(Clone)]
struct On {
    timestamp_ms: u64,
    level: u8,       
    message: String,
}


const ASN_: usize = 2048;

struct DmesgBuffer {
    entries: Vec<On>,
    
    write_idx: usize,
    
    total_count: u64,
    
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
        if self.entries.len() < ASN_ {
            self.entries.push(On { timestamp_ms, level, message });
        } else {
            self.entries[self.write_idx] = On { timestamp_ms, level, message };
            self.wrapped = true;
        }
        self.write_idx = (self.write_idx + 1) % ASN_;
        self.total_count += 1;
    }
    
    
    fn iter_ordered(&self) -> impl Iterator<Item = &On> {
        let start = if self.wrapped { self.write_idx } else { 0 };
        let len = self.entries.len();
        (0..len).map(move |i| &self.entries[(start + i) % len])
    }
    
    fn len(&self) -> usize {
        self.entries.len()
    }
}

static Rq: Mutex<DmesgBuffer> = Mutex::new(DmesgBuffer::new());



pub fn lgl(level: u8, message: String) {
    
    let jy = crate::time::uptime_ms();
    if let Some(mut buf) = Rq.try_lock() {
        buf.push(jy, level, message);
    }
}


pub fn hsx(level: u8, bk: &str) {
    lgl(level, String::from(bk));
}


pub fn lgj(ae: usize) -> Vec<String> {
    let buf = Rq.lock();
    let entries: Vec<_> = buf.iter_ordered().collect();
    let start = if ae > 0 && ae < entries.len() { entries.len() - ae } else { 0 };
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


pub fn lgk() -> (usize, u64) {
    let buf = Rq.lock();
    (buf.len(), buf.total_count)
}






static AMF_: AtomicU64 = AtomicU64::new(0);
static ARY_: AtomicU64 = AtomicU64::new(0);
static AME_: AtomicU64 = AtomicU64::new(0);
static ARX_: AtomicU64 = AtomicU64::new(0);
static BEX_: AtomicUsize = AtomicUsize::new(0);
static TQ_: AtomicU64 = AtomicU64::new(0);

static BAP_: AtomicUsize = AtomicUsize::new(0);


pub fn pmp(size: usize) {
    AMF_.fetch_add(1, Ordering::Relaxed);
    AME_.fetch_add(size as u64, Ordering::Relaxed);
    TQ_.fetch_add(1, Ordering::Relaxed);
    
    
    let used = crate::memory::heap::used();
    let _ = BEX_.fetch_max(used, Ordering::Relaxed);
    
    
    let _ = BAP_.fetch_max(size, Ordering::Relaxed);
    
    
    if size >= 4096 {
        crate::lab_mode::trace_bus::emit(
            crate::lab_mode::trace_bus::EventCategory::Memory,
            alloc::format!("alloc {} bytes", size),
            size as u64,
        );
    }
}


pub fn pmu(size: usize) {
    ARY_.fetch_add(1, Ordering::Relaxed);
    ARX_.fetch_add(size as u64, Ordering::Relaxed);
    TQ_.fetch_sub(1, Ordering::Relaxed);
    
    
    if size >= 4096 {
        crate::lab_mode::trace_bus::emit(
            crate::lab_mode::trace_bus::EventCategory::Memory,
            alloc::format!("dealloc {} bytes", size),
            size as u64,
        );
    }
}


pub struct Abh {
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


pub fn dbe() -> Abh {
    let used = crate::memory::heap::used();
    let free = crate::memory::heap::free();
    let av = used + free;
    
    
    
    
    let alloc_count = AMF_.load(Ordering::Relaxed);
    let dealloc_count = ARY_.load(Ordering::Relaxed);
    let lyh = if av > 0 && alloc_count > 100 {
        
        
        let kkm = if alloc_count > 0 {
            (dealloc_count as f32) / (alloc_count as f32)
        } else {
            0.0
        };
        
        let lyq = free as f32 / av as f32;
        (kkm * (1.0 - lyq) * 100.0).min(100.0)
    } else {
        0.0
    };
    
    Abh {
        alloc_count,
        dealloc_count,
        alloc_bytes_total: AME_.load(Ordering::Relaxed),
        dealloc_bytes_total: ARX_.load(Ordering::Relaxed),
        peak_heap_used: BEX_.load(Ordering::Relaxed),
        current_heap_used: used,
        current_heap_free: free,
        heap_total: av,
        live_allocs: TQ_.load(Ordering::Relaxed),
        largest_alloc: BAP_.load(Ordering::Relaxed),
        fragmentation_pct: lyh,
    }
}






static CGX_: AtomicU64 = AtomicU64::new(0);
static CGY_: AtomicU64 = AtomicU64::new(0);
static AZJ_: AtomicU64 = AtomicU64::new(0);


static AYX_: AtomicU64 = AtomicU64::new(0);
static APN_: AtomicU64 = AtomicU64::new(0);


pub fn qtd(cycles: u64) {
    AYX_.fetch_add(cycles, Ordering::Relaxed);
}


pub fn qtc(cycles: u64) {
    APN_.fetch_add(cycles, Ordering::Relaxed);
}


pub fn kyu() -> u32 {
    let ckv = AYX_.swap(0, Ordering::Relaxed);
    let ehi = APN_.swap(0, Ordering::Relaxed);
    let av = ckv + ehi;
    if av == 0 { return 0; }
    ((ehi * 100) / av) as u32
}


pub fn ppv() {
    let stats = crate::sync::percpu::dhj();
    let total_irqs: u64 = stats.iter().map(|j| j.interrupts).sum();
    let cy = crate::time::uptime_ms();
    
    let mwn = CGX_.swap(total_irqs, Ordering::Relaxed);
    let mws = CGY_.swap(cy, Ordering::Relaxed);
    
    let fm = cy.saturating_sub(mws);
    if fm > 0 {
        let exq = (total_irqs - mwn) * 1000 / fm;
        AZJ_.store(exq, Ordering::Relaxed);
    }
}


pub fn irq_rate() -> u64 {
    AZJ_.load(Ordering::Relaxed)
}


pub struct Acn {
    pub uptime_ms: u64,
    pub cpu_stats: Vec<crate::sync::percpu::Rl>,
    pub total_irqs: u64,
    pub total_syscalls: u64,
    pub total_ctx_switches: u64,
    pub irq_per_sec: u64,
    pub heap_used: usize,
    pub heap_free: usize,
    pub fps: u64,
}


pub fn nto() -> Acn {
    let stats = crate::sync::percpu::dhj();
    let total_irqs: u64 = stats.iter().map(|j| j.interrupts).sum();
    let total_syscalls: u64 = stats.iter().map(|j| j.syscalls).sum();
    let total_ctx_switches: u64 = stats.iter().map(|j| j.context_switches).sum();
    
    Acn {
        uptime_ms: crate::time::uptime_ms(),
        cpu_stats: stats,
        total_irqs,
        total_syscalls,
        total_ctx_switches,
        irq_per_sec: irq_rate(),
        heap_used: crate::memory::heap::used(),
        heap_free: crate::memory::heap::free(),
        fps: crate::gui::engine::fyp(),
    }
}







pub fn peek(addr: usize, count: usize) -> Vec<String> {
    let mut lines = Vec::new();

    
    if !crate::auth::is_root() {
        lines.push(String::from("Error: peek requires root privileges"));
        return lines;
    }

    let count = count.min(256); 
    
    
    if addr == 0 {
        lines.push(String::from("Error: NULL pointer"));
        return lines;
    }
    
    let fkh = 16;
    let mut offset = 0;
    
    while offset < count {
        let myk = (count - offset).min(fkh);
        let mut ga = String::new();
        let mut ascii = String::new();
        
        for i in 0..fkh {
            if i < myk {
                let byte = unsafe {
                    
                    core::ptr::read_volatile((addr + offset + i) as *const u8)
                };
                ga.push_str(&format!("{:02x} ", byte));
                ascii.push(if byte >= 0x20 && byte < 0x7F { byte as char } else { '.' });
            } else {
                ga.push_str("   ");
                ascii.push(' ');
            }
            if i == 7 { ga.push(' '); }
        }
        
        lines.push(format!("  {:016x}  {}|{}|", addr + offset, ga, ascii));
        offset += fkh;
    }
    
    lines
}



pub fn gnq(addr: usize, value: u8) -> Result<(), &'static str> {
    
    if !crate::auth::is_root() {
        return Err("poke requires root privileges");
    }
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


pub fn kyq() -> Vec<String> {
    let mut regs = Vec::new();
    
    let rsp: u64;
    let rbp: u64;
    let rflags: u64;
    let cr0: u64;
    let cr3: u64;
    let cr4: u64;
    
    #[cfg(target_arch = "x86_64")]
    unsafe {
        core::arch::asm!("mov {}, rsp", out(reg) rsp);
        core::arch::asm!("mov {}, rbp", out(reg) rbp);
        core::arch::asm!("pushfq; pop {}", out(reg) rflags);
        core::arch::asm!("mov {}, cr0", out(reg) cr0);
        core::arch::asm!("mov {}, cr3", out(reg) cr3);
        core::arch::asm!("mov {}, cr4", out(reg) cr4);
    }
    #[cfg(not(target_arch = "x86_64"))]
    { rsp = 0; rbp = 0; rflags = 0; cr0 = 0; cr3 = 0; cr4 = 0; }
    
    regs.push(String::from("  CPU Register Snapshot"));
    regs.push(String::from("  ─────────────────────────────────────"));
    regs.push(format!("  RSP    = 0x{:016x}  (stack pointer)", rsp));
    regs.push(format!("  RBP    = 0x{:016x}  (base pointer)", rbp));
    regs.push(format!("  RFLAGS = 0x{:016x}", rflags));
    regs.push(String::from(""));
    regs.push(format!("  CR0    = 0x{:016x}", cr0));
    
    
    let mut chu = Vec::new();
    if cr0 & 1 != 0 { chu.push("PE"); }
    if cr0 & (1 << 1) != 0 { chu.push("MP"); }
    if cr0 & (1 << 4) != 0 { chu.push("ET"); }
    if cr0 & (1 << 5) != 0 { chu.push("NE"); }
    if cr0 & (1 << 16) != 0 { chu.push("WP"); }
    if cr0 & (1 << 31) != 0 { chu.push("PG"); }
    regs.push(format!("           [{}]", chu.join(" | ")));
    
    regs.push(format!("  CR3    = 0x{:016x}  (page table root)", cr3));
    regs.push(format!("  CR4    = 0x{:016x}", cr4));
    
    
    let mut brf = Vec::new();
    if cr4 & (1 << 5) != 0 { brf.push("PAE"); }
    if cr4 & (1 << 7) != 0 { brf.push("PGE"); }
    if cr4 & (1 << 9) != 0 { brf.push("OSFXSR"); }
    if cr4 & (1 << 10) != 0 { brf.push("OSXMMEXCPT"); }
    if cr4 & (1 << 13) != 0 { brf.push("VMXE"); }
    if cr4 & (1 << 18) != 0 { brf.push("OSXSAVE"); }
    if cr4 & (1 << 20) != 0 { brf.push("SMEP"); }
    if cr4 & (1 << 21) != 0 { brf.push("SMAP"); }
    regs.push(format!("           [{}]", brf.join(" | ")));
    
    
    #[cfg(target_arch = "x86_64")]
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
    #[cfg(not(target_arch = "x86_64"))]
    let efer: u64 = 0;
    regs.push(format!("  EFER   = 0x{:016x}", efer));
    let mut dok = Vec::new();
    if efer & 1 != 0 { dok.push("SCE"); }
    if efer & (1 << 8) != 0 { dok.push("LME"); }
    if efer & (1 << 10) != 0 { dok.push("LMA"); }
    if efer & (1 << 11) != 0 { dok.push("NXE"); }
    regs.push(format!("           [{}]", dok.join(" | ")));
    
    regs
}






static TY_: AtomicBool = AtomicBool::new(false);


pub fn pks() {
    let prev = TY_.load(Ordering::Relaxed);
    TY_.store(!prev, Ordering::Relaxed);
}

pub fn qvt(visible: bool) {
    TY_.store(visible, Ordering::Relaxed);
}

pub fn ihv() -> bool {
    TY_.load(Ordering::Relaxed)
}


pub struct Xs {
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


pub fn leg() -> Xs {
    let used = crate::memory::heap::used();
    let free = crate::memory::heap::free();
    let av = used + free;
    let stats = crate::sync::percpu::dhj();
    let total_irqs: u64 = stats.iter().map(|j| j.interrupts).sum();
    
    Xs {
        fps: crate::gui::engine::fyp(),
        frame_time_us: 0, 
        heap_used_kb: used / 1024,
        heap_total_kb: av / 1024,
        heap_pct: if av > 0 { ((used * 100) / av) as u32 } else { 0 },
        live_allocs: TQ_.load(Ordering::Relaxed),
        irq_per_sec: irq_rate(),
        uptime_secs: crate::time::uptime_ms() / 1000,
        cpu_count: stats.len().max(1),
        total_irqs,
    }
}



pub fn ofi(width: u32, _height: u32, frame_time_us: u64) {
    if !ihv() {
        return;
    }
    
    
    ppv();
    
    let data = leg();
    
    
    let he = 260u32;
    let ug = 180u32;
    let zc = (width - he - 8) as i32;
    let xg = 8i32;
    
    
    let bg_color: u32 = 0xFF101018;
    let ri: u32 = 0xFF00FF88;
    let bwl: u32 = 0xFF00FFAA;
    let ace: u32 = 0xFF88AACC;
    let prd: u32 = 0xFFFFFFFF;
    let jzm: u32 = 0xFF333344;
    
    
    crate::framebuffer::fill_rect(zc as u32, xg as u32, he, ug, bg_color);
    
    
    crate::framebuffer::mn(zc as u32, xg as u32, he, ri);
    crate::framebuffer::mn(zc as u32, (xg + ug as i32 - 1) as u32, he, ri);
    crate::framebuffer::zv(zc as u32, xg as u32, ug, ri);
    crate::framebuffer::zv((zc + he as i32 - 1) as u32, xg as u32, ug, ri);
    
    let x = zc + 8;
    let mut y = xg + 6;
    
    
    cim(x, y, "DEVPANEL [F12]", bwl);
    y += 14;
    
    
    crate::framebuffer::mn((zc + 4) as u32, y as u32, he - 8, 0xFF444466);
    y += 6;
    
    
    let qk = if frame_time_us > 0 { frame_time_us } else { 16666 };
    cim(x, y, &format!("FPS: {:<4}  Frame: {:.1}ms", data.fps, qk as f64 / 1000.0), prd);
    y += 14;
    
    
    cim(x, y, &format!("Heap: {} / {} KB ({}%)", 
        data.heap_used_kb, data.heap_total_kb, data.heap_pct), ace);
    y += 12;
    
    
    let ek = (he - 20) as u32;
    let pv = (x + 2) as u32;
    let gk = y as u32;
    crate::framebuffer::fill_rect(pv, gk, ek, 6, jzm);
    let oz = (ek * data.heap_pct) / 100;
    let bxq = if data.heap_pct > 90 { 0xFFFF4444 }
        else if data.heap_pct > 70 { 0xFFFFAA44 }
        else { 0xFF44FF88 };
    crate::framebuffer::fill_rect(pv, gk, oz, 6, bxq);
    y += 12;
    
    
    cim(x, y, &format!("Allocs: {} live", data.live_allocs), ace);
    y += 14;
    
    
    cim(x, y, &format!("IRQ/s: {}   Total: {}", data.irq_per_sec, data.total_irqs), ace);
    y += 14;
    
    
    cim(x, y, &format!("CPUs: {}   Uptime: {}s", data.cpu_count, data.uptime_secs), ace);
    y += 14;
    
    
    let stats = crate::sync::percpu::dhj();
    for (i, j) in stats.iter().enumerate().take(4) {
        let mnt = if j.is_idle { "idle" } else { "busy" };
        cim(x, y, &format!("  CPU{}: {} irqs [{}]", i, j.interrupts, mnt), 
            if j.is_idle { 0xFF667788 } else { 0xFF88FFAA });
        y += 12;
    }
}


fn cim(x: i32, y: i32, text: &str, color: u32) {
    let x = x as u32;
    let y = y as u32;
    for (i, c) in text.chars().enumerate() {
        crate::framebuffer::px(x + (i as u32 * 8), y, c, color);
    }
}






pub fn pzg(bk: &str) {
    hsx(2, bk); 
}


pub fn khl(bk: core::fmt::Arguments) {
    
    
    if crate::memory::heap::free() == 0 {
        return;
    }
    let j = format!("{}", bk);
    hsx(2, &j);
}
