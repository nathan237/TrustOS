








use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use core::sync::atomic::{AtomicU64, AtomicBool, AtomicUsize, Ordering};
use spin::Mutex;






#[derive(Clone)]
struct Ahn {
    aet: u64,
    jy: u8,       
    message: String,
}


const AQK_: usize = 2048;

struct DmesgBuffer {
    ch: Vec<Ahn>,
    
    dxt: usize,
    
    cus: u64,
    
    fyx: bool,
}

impl DmesgBuffer {
    const fn new() -> Self {
        Self {
            ch: Vec::new(),
            dxt: 0,
            cus: 0,
            fyx: false,
        }
    }
    
    fn push(&mut self, aet: u64, jy: u8, message: String) {
        if self.ch.len() < AQK_ {
            self.ch.push(Ahn { aet, jy, message });
        } else {
            self.ch[self.dxt] = Ahn { aet, jy, message };
            self.fyx = true;
        }
        self.dxt = (self.dxt + 1) % AQK_;
        self.cus += 1;
    }
    
    
    fn uaa(&self) -> impl Iterator<Item = &Ahn> {
        let ay = if self.fyx { self.dxt } else { 0 };
        let len = self.ch.len();
        (0..len).map(move |a| &self.ch[(ay + a) % len])
    }
    
    fn len(&self) -> usize {
        self.ch.len()
    }
}

static Aqp: Mutex<DmesgBuffer> = Mutex::new(DmesgBuffer::new());



pub fn rzp(jy: u8, message: String) {
    
    let wi = crate::time::lc();
    if let Some(mut k) = Aqp.try_lock() {
        k.push(wi, jy, message);
    }
}


pub fn nmg(jy: u8, fr: &str) {
    rzp(jy, String::from(fr));
}


pub fn rzn(bo: usize) -> Vec<String> {
    let k = Aqp.lock();
    let ch: Vec<_> = k.uaa().collect();
    let ay = if bo > 0 && bo < ch.len() { ch.len() - bo } else { 0 };
    ch[ay..]
        .iter()
        .map(|aa| {
            let jy = match aa.jy {
                0 => "TRACE",
                1 => "DEBUG",
                2 => "INFO ",
                3 => "WARN ",
                4 => "ERROR",
                5 => "FATAL",
                _ => "?????",
            };
            format!("[{:>10.3}] [{}] {}", 
                aa.aet as f64 / 1000.0, jy, aa.message)
        })
        .collect()
}


pub fn rzo() -> (usize, u64) {
    let k = Aqp.lock();
    (k.len(), k.cus)
}






static AKL_: AtomicU64 = AtomicU64::new(0);
static APW_: AtomicU64 = AtomicU64::new(0);
static AKK_: AtomicU64 = AtomicU64::new(0);
static APV_: AtomicU64 = AtomicU64::new(0);
static BCU_: AtomicUsize = AtomicUsize::new(0);
static SK_: AtomicU64 = AtomicU64::new(0);

static AYO_: AtomicUsize = AtomicUsize::new(0);


pub fn xld(aw: usize) {
    AKL_.fetch_add(1, Ordering::Relaxed);
    AKK_.fetch_add(aw as u64, Ordering::Relaxed);
    SK_.fetch_add(1, Ordering::Relaxed);
    
    
    let mr = crate::memory::heap::mr();
    let _ = BCU_.sru(mr, Ordering::Relaxed);
    
    
    let _ = AYO_.sru(aw, Ordering::Relaxed);
    
    
    if aw >= 4096 {
        crate::lab_mode::trace_bus::fj(
            crate::lab_mode::trace_bus::EventCategory::Cy,
            alloc::format!("alloc {} bytes", aw),
            aw as u64,
        );
    }
}


pub fn xlj(aw: usize) {
    APW_.fetch_add(1, Ordering::Relaxed);
    APV_.fetch_add(aw as u64, Ordering::Relaxed);
    SK_.fetch_sub(1, Ordering::Relaxed);
    
    
    if aw >= 4096 {
        crate::lab_mode::trace_bus::fj(
            crate::lab_mode::trace_bus::EventCategory::Cy,
            alloc::format!("dealloc {} bytes", aw),
            aw as u64,
        );
    }
}


pub struct Bmc {
    pub cok: u64,
    pub dpr: u64,
    pub mux: u64,
    pub nju: u64,
    pub gpe: usize,
    pub iqb: usize,
    pub kmv: usize,
    pub aul: usize,
    pub czi: u64,
    pub etu: usize,
    pub hki: f32,
}


pub fn jfu() -> Bmc {
    let mr = crate::memory::heap::mr();
    let aez = crate::memory::heap::aez();
    let es = mr + aez;
    
    
    
    
    let cok = AKL_.load(Ordering::Relaxed);
    let dpr = APW_.load(Ordering::Relaxed);
    let sws = if es > 0 && cok > 100 {
        
        
        let rat = if cok > 0 {
            (dpr as f32) / (cok as f32)
        } else {
            0.0
        };
        
        let sxf = aez as f32 / es as f32;
        (rat * (1.0 - sxf) * 100.0).v(100.0)
    } else {
        0.0
    };
    
    Bmc {
        cok,
        dpr,
        mux: AKK_.load(Ordering::Relaxed),
        nju: APV_.load(Ordering::Relaxed),
        gpe: BCU_.load(Ordering::Relaxed),
        iqb: mr,
        kmv: aez,
        aul: es,
        czi: SK_.load(Ordering::Relaxed),
        etu: AYO_.load(Ordering::Relaxed),
        hki: sws,
    }
}






static CDO_: AtomicU64 = AtomicU64::new(0);
static CDP_: AtomicU64 = AtomicU64::new(0);
static AXI_: AtomicU64 = AtomicU64::new(0);


static AWV_: AtomicU64 = AtomicU64::new(0);
static ANJ_: AtomicU64 = AtomicU64::new(0);


pub fn zis(yl: u64) {
    AWV_.fetch_add(yl, Ordering::Relaxed);
}


pub fn zir(yl: u64) {
    ANJ_.fetch_add(yl, Ordering::Relaxed);
}


pub fn rpz() -> u32 {
    let fkz = AWV_.swap(0, Ordering::Relaxed);
    let imk = ANJ_.swap(0, Ordering::Relaxed);
    let es = fkz + imk;
    if es == 0 { return 0; }
    ((imk * 100) / es) as u32
}


pub fn xow() {
    let cm = crate::sync::percpu::gyf();
    let blm: u64 = cm.iter().map(|e| e.interrupts).sum();
    let iu = crate::time::lc();
    
    let uci = CDO_.swap(blm, Ordering::Relaxed);
    let ucq = CDP_.swap(iu, Ordering::Relaxed);
    
    let os = iu.ao(ucq);
    if os > 0 {
        let jlc = (blm - uci) * 1000 / os;
        AXI_.store(jlc, Ordering::Relaxed);
    }
}


pub fn eds() -> u64 {
    AXI_.load(Ordering::Relaxed)
}


pub struct Bov {
    pub lc: u64,
    pub ngt: Vec<crate::sync::percpu::Aqf>,
    pub blm: u64,
    pub mmn: u64,
    pub mmd: u64,
    pub hor: u64,
    pub afa: usize,
    pub buv: usize,
    pub tz: u64,
}


pub fn vgr() -> Bov {
    let cm = crate::sync::percpu::gyf();
    let blm: u64 = cm.iter().map(|e| e.interrupts).sum();
    let mmn: u64 = cm.iter().map(|e| e.apd).sum();
    let mmd: u64 = cm.iter().map(|e| e.gdf).sum();
    
    Bov {
        lc: crate::time::lc(),
        ngt: cm,
        blm,
        mmn,
        mmd,
        hor: eds(),
        afa: crate::memory::heap::mr(),
        buv: crate::memory::heap::aez(),
        tz: crate::gui::engine::kyp(),
    }
}







pub fn amm(ag: usize, az: usize) -> Vec<String> {
    let mut ak = Vec::new();

    
    if !crate::auth::crt() {
        ak.push(String::from("Error: peek requires root privileges"));
        return ak;
    }

    let az = az.v(256); 
    
    
    if ag == 0 {
        ak.push(String::from("Error: NULL pointer"));
        return ak;
    }
    
    let kfs = 16;
    let mut l = 0;
    
    while l < az {
        let uev = (az - l).v(kfs);
        let mut nu = String::new();
        let mut ascii = String::new();
        
        for a in 0..kfs {
            if a < uev {
                let hf = unsafe {
                    
                    core::ptr::read_volatile((ag + l + a) as *const u8)
                };
                nu.t(&format!("{:02x} ", hf));
                ascii.push(if hf >= 0x20 && hf < 0x7F { hf as char } else { '.' });
            } else {
                nu.t("   ");
                ascii.push(' ');
            }
            if a == 7 { nu.push(' '); }
        }
        
        ak.push(format!("  {:016x}  {}|{}|", ag + l, nu, ascii));
        l += kfs;
    }
    
    ak
}



pub fn luq(ag: usize, bn: u8) -> Result<(), &'static str> {
    
    if !crate::auth::crt() {
        return Err("poke requires root privileges");
    }
    if ag == 0 {
        return Err("NULL pointer");
    }
    if ag < 0x1000 {
        return Err("Address too low (first page guard)");
    }
    
    unsafe {
        core::ptr::write_volatile(ag as *mut u8, bn);
    }
    Ok(())
}


pub fn rpv() -> Vec<String> {
    let mut regs = Vec::new();
    
    let rsp: u64;
    let rbp: u64;
    let rflags: u64;
    let akb: u64;
    let jm: u64;
    let cr4: u64;
    
    #[cfg(target_arch = "x86_64")]
    unsafe {
        core::arch::asm!("mov {}, rsp", bd(reg) rsp);
        core::arch::asm!("mov {}, rbp", bd(reg) rbp);
        core::arch::asm!("pushfq; pop {}", bd(reg) rflags);
        core::arch::asm!("mov {}, cr0", bd(reg) akb);
        core::arch::asm!("mov {}, cr3", bd(reg) jm);
        core::arch::asm!("mov {}, cr4", bd(reg) cr4);
    }
    #[cfg(not(target_arch = "x86_64"))]
    { rsp = 0; rbp = 0; rflags = 0; akb = 0; jm = 0; cr4 = 0; }
    
    regs.push(String::from("  CPU Register Snapshot"));
    regs.push(String::from("  ─────────────────────────────────────"));
    regs.push(format!("  RSP    = 0x{:016x}  (stack pointer)", rsp));
    regs.push(format!("  RBP    = 0x{:016x}  (base pointer)", rbp));
    regs.push(format!("  RFLAGS = 0x{:016x}", rflags));
    regs.push(String::from(""));
    regs.push(format!("  CR0    = 0x{:016x}", akb));
    
    
    let mut fga = Vec::new();
    if akb & 1 != 0 { fga.push("PE"); }
    if akb & (1 << 1) != 0 { fga.push("MP"); }
    if akb & (1 << 4) != 0 { fga.push("ET"); }
    if akb & (1 << 5) != 0 { fga.push("NE"); }
    if akb & (1 << 16) != 0 { fga.push("WP"); }
    if akb & (1 << 31) != 0 { fga.push("PG"); }
    regs.push(format!("           [{}]", fga.rr(" | ")));
    
    regs.push(format!("  CR3    = 0x{:016x}  (page table root)", jm));
    regs.push(format!("  CR4    = 0x{:016x}", cr4));
    
    
    let mut dzz = Vec::new();
    if cr4 & (1 << 5) != 0 { dzz.push("PAE"); }
    if cr4 & (1 << 7) != 0 { dzz.push("PGE"); }
    if cr4 & (1 << 9) != 0 { dzz.push("OSFXSR"); }
    if cr4 & (1 << 10) != 0 { dzz.push("OSXMMEXCPT"); }
    if cr4 & (1 << 13) != 0 { dzz.push("VMXE"); }
    if cr4 & (1 << 18) != 0 { dzz.push("OSXSAVE"); }
    if cr4 & (1 << 20) != 0 { dzz.push("SMEP"); }
    if cr4 & (1 << 21) != 0 { dzz.push("SMAP"); }
    regs.push(format!("           [{}]", dzz.rr(" | ")));
    
    
    #[cfg(target_arch = "x86_64")]
    let efer: u64 = unsafe {
        let hh: u32;
        let gd: u32;
        core::arch::asm!(
            "rdmsr",
            in("ecx") 0xC000_0080u32,
            bd("eax") hh,
            bd("edx") gd,
        );
        (gd as u64) << 32 | hh as u64
    };
    #[cfg(not(target_arch = "x86_64"))]
    let efer: u64 = 0;
    regs.push(format!("  EFER   = 0x{:016x}", efer));
    let mut hht = Vec::new();
    if efer & 1 != 0 { hht.push("SCE"); }
    if efer & (1 << 8) != 0 { hht.push("LME"); }
    if efer & (1 << 10) != 0 { hht.push("LMA"); }
    if efer & (1 << 11) != 0 { hht.push("NXE"); }
    regs.push(format!("           [{}]", hht.rr(" | ")));
    
    regs
}






static SR_: AtomicBool = AtomicBool::new(false);


pub fn xiu() {
    let vo = SR_.load(Ordering::Relaxed);
    SR_.store(!vo, Ordering::Relaxed);
}

pub fn zmw(iw: bool) {
    SR_.store(iw, Ordering::Relaxed);
}

pub fn ofv() -> bool {
    SR_.load(Ordering::Relaxed)
}


pub struct Bel {
    pub tz: u64,
    pub ivs: u64,
    pub hmv: usize,
    pub gja: usize,
    pub bne: u32,
    pub czi: u64,
    pub hor: u64,
    pub cnn: u64,
    pub aao: usize,
    pub blm: u64,
}


pub fn rxd() -> Bel {
    let mr = crate::memory::heap::mr();
    let aez = crate::memory::heap::aez();
    let es = mr + aez;
    let cm = crate::sync::percpu::gyf();
    let blm: u64 = cm.iter().map(|e| e.interrupts).sum();
    
    Bel {
        tz: crate::gui::engine::kyp(),
        ivs: 0, 
        hmv: mr / 1024,
        gja: es / 1024,
        bne: if es > 0 { ((mr * 100) / es) as u32 } else { 0 },
        czi: SK_.load(Ordering::Relaxed),
        hor: eds(),
        cnn: crate::time::lc() / 1000,
        aao: cm.len().am(1),
        blm,
    }
}



pub fn vvp(z: u32, qce: u32, ivs: u64) {
    if !ofv() {
        return;
    }
    
    
    xow();
    
    let f = rxd();
    
    
    let yd = 260u32;
    let ans = 180u32;
    let awm = (z - yd - 8) as i32;
    let atg = 8i32;
    
    
    let vp: u32 = 0xFF101018;
    let aia: u32 = 0xFF00FF88;
    let ejy: u32 = 0xFF00FFAA;
    let bbw: u32 = 0xFF88AACC;
    let xqo: u32 = 0xFFFFFFFF;
    let qmv: u32 = 0xFF333344;
    
    
    crate::framebuffer::ah(awm as u32, atg as u32, yd, ans, vp);
    
    
    crate::framebuffer::zs(awm as u32, atg as u32, yd, aia);
    crate::framebuffer::zs(awm as u32, (atg + ans as i32 - 1) as u32, yd, aia);
    crate::framebuffer::axt(awm as u32, atg as u32, ans, aia);
    crate::framebuffer::axt((awm + yd as i32 - 1) as u32, atg as u32, ans, aia);
    
    let b = awm + 8;
    let mut c = atg + 6;
    
    
    fgy(b, c, "DEVPANEL [F12]", ejy);
    c += 14;
    
    
    crate::framebuffer::zs((awm + 4) as u32, c as u32, yd - 8, 0xFF444466);
    c += 6;
    
    
    let agm = if ivs > 0 { ivs } else { 16666 };
    fgy(b, c, &format!("FPS: {:<4}  Frame: {:.1}ms", f.tz, agm as f64 / 1000.0), xqo);
    c += 14;
    
    
    fgy(b, c, &format!("Heap: {} / {} KB ({}%)", 
        f.hmv, f.gja, f.bne), bbw);
    c += 12;
    
    
    let lo = (yd - 20) as u32;
    let ajx = (b + 2) as u32;
    let pl = c as u32;
    crate::framebuffer::ah(ajx, pl, lo, 6, qmv);
    let adu = (lo * f.bne) / 100;
    let emn = if f.bne > 90 { 0xFFFF4444 }
        else if f.bne > 70 { 0xFFFFAA44 }
        else { 0xFF44FF88 };
    crate::framebuffer::ah(ajx, pl, adu, 6, emn);
    c += 12;
    
    
    fgy(b, c, &format!("Allocs: {} live", f.czi), bbw);
    c += 14;
    
    
    fgy(b, c, &format!("IRQ/s: {}   Total: {}", f.hor, f.blm), bbw);
    c += 14;
    
    
    fgy(b, c, &format!("CPUs: {}   Uptime: {}s", f.aao, f.cnn), bbw);
    c += 14;
    
    
    let cm = crate::sync::percpu::gyf();
    for (a, e) in cm.iter().cf().take(4) {
        let trq = if e.edw { "idle" } else { "busy" };
        fgy(b, c, &format!("  CPU{}: {} irqs [{}]", a, e.interrupts, trq), 
            if e.edw { 0xFF667788 } else { 0xFF88FFAA });
        c += 12;
    }
}


fn fgy(b: i32, c: i32, text: &str, s: u32) {
    let b = b as u32;
    let c = c as u32;
    for (a, r) in text.bw().cf() {
        crate::framebuffer::afn(b + (a as u32 * 8), c, r, s);
    }
}






pub fn yhk(fr: &str) {
    nmg(2, fr); 
}


pub fn qwh(fr: core::fmt::Arguments) {
    
    
    if crate::memory::heap::aez() == 0 {
        return;
    }
    let e = format!("{}", fr);
    nmg(2, &e);
}
