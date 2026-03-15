











use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use spin::Mutex;


const CFT_: usize = 256;

const AFG_: usize = 512;

const DTJ_: usize = 128;


static CT_: Mutex<Option<DebugMonitor>> = Mutex::new(None);

static SN_: AtomicBool = AtomicBool::new(false);

static ET_: AtomicU64 = AtomicU64::new(0);


#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DebugCategory {
    Iu,
    Lr,
    Hx,
    Jr,
    Ahg,
    Qe,
    Fv,
    Acd,
    Aqg,
    Ahu,
    Qg,
}

impl DebugCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Iu => "I/O IN",
            Self::Lr => "I/O OUT",
            Self::Hx => "RDMSR",
            Self::Jr => "WRMSR",
            Self::Ahg => "CPUID",
            Self::Qe => "NPF",
            Self::Fv => "INTR",
            Self::Acd => "VMCALL",
            Self::Aqg => "CR WRITE",
            Self::Ahu => "EXCEPTION",
            Self::Qg => "OTHER",
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandleStatus {
    
    Gw,
    
    Azk,
    
    Id,
    
    Nd,
}

impl HandleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Gw => "OK",
            Self::Azk => "STUB",
            Self::Id => "MISS",
            Self::Nd => "FATAL",
        }
    }

    pub fn cpk(&self) -> &'static str {
        match self {
            Self::Gw => "\x01G",    
            Self::Azk => "\x01Y",    
            Self::Id => "\x01R",  
            Self::Nd => "\x01M",      
        }
    }
}


#[derive(Debug, Clone)]
pub struct Bei {
    
    pub fk: u64,
    
    pub gb: DebugCategory,
    
    pub cys: u64,
    
    pub status: HandleStatus,
    
    pub wb: u64,
    
    pub eu: String,
    
    pub eqb: u64,
}


#[derive(Debug, Clone)]
pub struct Bjm {
    
    pub az: u64,
    
    pub status: HandleStatus,
    
    pub nuv: u64,
    
    pub jcp: u64,
    
    pub j: String,
    
    pub etx: String,
}


#[derive(Debug, Clone)]
pub struct Buj {
    pub eqb: u64,
    pub gb: DebugCategory,
    pub cys: u64,
    pub status: HandleStatus,
    pub wb: u64,
}


pub struct DebugMonitor {
    
    pub cm: BTreeMap<(DebugCategory, u64), Bjm>,
    
    pub dcr: Vec<Buj>,
    
    pub ido: usize,
    
    pub hcj: BTreeMap<DebugCategory, u64>,
    
    pub gvn: BTreeMap<DebugCategory, u64>,
    
    pub ckr: Vec<(DebugCategory, u64, String, u64)>, 
    
    pub lmp: Vec<u64>,
    
    pub mdy: bool,
    
    pub wst: u64,
}

impl DebugMonitor {
    pub fn new() -> Self {
        Self {
            cm: BTreeMap::new(),
            dcr: Vec::fc(AFG_),
            ido: 0,
            hcj: BTreeMap::new(),
            gvn: BTreeMap::new(),
            ckr: Vec::new(),
            lmp: Vec::new(),
            mdy: false,
            wst: 0,
        }
    }

    
    pub fn record(&mut self, id: Bei) {
        
        *self.hcj.bt(id.gb).gom(0) += 1;

        
        if oh!(id.status, HandleStatus::Id | HandleStatus::Nd) {
            *self.gvn.bt(id.gb).gom(0) += 1;
        }

        
        let bs = (id.gb, id.cys);
        if let Some(hm) = self.cm.ds(&bs) {
            hm.az += 1;
            hm.status = id.status;
            hm.jcp = id.wb;
            hm.etx = id.eu.clone();
        } else if self.cm.len() < CFT_ {
            let j = odb(id.gb, id.cys);
            self.cm.insert(bs, Bjm {
                az: 1,
                status: id.status,
                nuv: id.wb,
                jcp: id.wb,
                j,
                etx: id.eu.clone(),
            });
        }

        
        let bt = Buj {
            eqb: id.eqb,
            gb: id.gb,
            cys: id.cys,
            status: id.status,
            wb: id.wb,
        };
        if self.dcr.len() < AFG_ {
            self.dcr.push(bt);
        } else {
            self.dcr[self.ido] = bt;
        }
        self.ido = (self.ido + 1) % AFG_;

        
        if self.mdy && !oh!(id.status, HandleStatus::Gw) {
            crate::serial_println!(
                "[DBG] VM{} #{} {} 0x{:X} [{}] RIP=0x{:X} {}",
                id.fk, id.eqb,
                id.gb.as_str(), id.cys,
                id.status.as_str(), id.wb, id.eu
            );
        }

        ET_.fetch_add(1, Ordering::Relaxed);
    }
}


pub fn init() {
    let mut bvm = CT_.lock();
    *bvm = Some(DebugMonitor::new());
    SN_.store(true, Ordering::SeqCst);
    crate::serial_println!("[DEBUG_MONITOR] Initialized — recording all VM exits");
}


pub fn qg() {
    SN_.store(false, Ordering::SeqCst);
    crate::serial_println!("[DEBUG_MONITOR] Stopped");
}


pub fn rl() -> bool {
    SN_.load(Ordering::Relaxed)
}


pub fn pje(iq: bool) {
    if let Some(ref mut ach) = *CT_.lock() {
        ach.mdy = iq;
    }
}


pub fn bry(
    fk: u64,
    gb: DebugCategory,
    cys: u64,
    status: HandleStatus,
    wb: u64,
    eqb: u64,
    eu: &str,
) {
    if !SN_.load(Ordering::Relaxed) {
        return;
    }

    if let Some(ref mut ach) = *CT_.lock() {
        
        if !ach.lmp.is_empty() && !ach.lmp.contains(&fk) {
            return;
        }

        ach.record(Bei {
            fk,
            gb,
            cys,
            status,
            wb,
            eu: String::from(eu),
            eqb,
        });
    }
}


pub fn kym() -> String {
    let bvm = CT_.lock();
    let ach = match bvm.as_ref() {
        Some(ef) => ef,
        None => return String::from("Debug monitor not initialized. Run 'vm debug init' first."),
    };

    let es = ET_.load(Ordering::Relaxed);
    let mut bd = String::fc(4096);

    bd.t("\x01C╔══════════════════════════════════════════════════════════════╗\x01W\n");
    bd.t("\x01C║\x01W   \x01GTRUST\x01WVM DEBUG MONITOR — Real-time VM Analysis            \x01C║\x01W\n");
    bd.t("\x01C╚══════════════════════════════════════════════════════════════╝\x01W\n\n");

    
    bd.t(&format!("  \x01YTotal events:\x01W {}    \x01YActive:\x01W {}\n\n",
        es, if rl() { "\x01Gyes\x01W" } else { "\x01Rno\x01W" }));

    
    bd.t("  \x01C── Category Breakdown ──────────────────────────────────────\x01W\n");
    bd.t("  \x01YCategory      Total      Unhandled    Rate\x01W\n");

    let fej = [
        DebugCategory::Iu, DebugCategory::Lr,
        DebugCategory::Hx, DebugCategory::Jr,
        DebugCategory::Ahg, DebugCategory::Qe,
        DebugCategory::Fv, DebugCategory::Acd,
        DebugCategory::Aqg, DebugCategory::Ahu,
    ];

    for rx in &fej {
        let az = ach.hcj.get(rx).hu().unwrap_or(0);
        let gvm = ach.gvn.get(rx).hu().unwrap_or(0);
        if az > 0 {
            let jlc = if az > 0 { (gvm * 100) / az } else { 0 };
            let s = if gvm == 0 { "\x01G" } else if jlc < 20 { "\x01Y" } else { "\x01R" };
            bd.t(&format!("  {:<14}{:>8}    {}{:>8}\x01W    {}{}%\x01W\n",
                rx.as_str(), az, s, gvm, s, jlc));
        }
    }
    bd.push('\n');

    
    let mut ckr: Vec<_> = ach.cm.iter()
        .hi(|((_, _), e)| oh!(e.status, HandleStatus::Id | HandleStatus::Azk))
        .collect();
    ckr.bxe(|q, o| o.1.az.cmp(&q.1.az));

    if !ckr.is_empty() {
        bd.t("  \x01C── Missing/Stubbed Operations ──────────────────────────────\x01W\n");
        bd.t("  \x01YCategory      ID             Name                   Count  Status\x01W\n");

        for ((rx, ad), hm) in ckr.iter().take(30) {
            bd.t(&format!("  {}{:<14}\x01W0x{:<12X} {:<22} {:>5}  {}{}\x01W\n",
                hm.status.cpk(),
                rx.as_str(), ad,
                if hm.j.len() > 22 { &hm.j[..22] } else { &hm.j },
                hm.az, hm.status.cpk(), hm.status.as_str()));
        }
        bd.push('\n');
    }

    
    let mut lfn: Vec<_> = ach.cm.iter()
        .hi(|((rx, _), _)| oh!(rx, DebugCategory::Iu | DebugCategory::Lr))
        .collect();
    lfn.bxe(|q, o| o.1.az.cmp(&q.1.az));

    if !lfn.is_empty() {
        bd.t("  \x01C── Top I/O Ports (by frequency) ────────────────────────────\x01W\n");
        bd.t("  \x01YDir    Port       Name                   Count  Status\x01W\n");
        for ((rx, port), hm) in lfn.iter().take(20) {
            let te = if oh!(rx, DebugCategory::Iu) { "IN " } else { "OUT" };
            bd.t(&format!("  {}  0x{:04X}     {:<22} {:>6}  {}{}\x01W\n",
                te, port,
                if hm.j.len() > 22 { &hm.j[..22] } else { &hm.j },
                hm.az, hm.status.cpk(), hm.status.as_str()));
        }
        bd.push('\n');
    }

    
    let mut lna: Vec<_> = ach.cm.iter()
        .hi(|((rx, _), _)| oh!(rx, DebugCategory::Hx | DebugCategory::Jr))
        .collect();
    lna.bxe(|q, o| o.1.az.cmp(&q.1.az));

    if !lna.is_empty() {
        bd.t("  \x01C── MSR Access Log ──────────────────────────────────────────\x01W\n");
        bd.t("  \x01YDir     MSR            Name                   Count  Status\x01W\n");
        for ((rx, msr), hm) in lna.iter().take(20) {
            let te = if oh!(rx, DebugCategory::Hx) { "READ " } else { "WRITE" };
            bd.t(&format!("  {}  0x{:08X}     {:<22} {:>5}  {}{}\x01W\n",
                te, msr,
                if hm.j.len() > 22 { &hm.j[..22] } else { &hm.j },
                hm.az, hm.status.cpk(), hm.status.as_str()));
        }
        bd.push('\n');
    }

    
    if !ach.dcr.is_empty() {
        bd.t("  \x01C── Recent Timeline (last 20) ──────────────────────────────\x01W\n");
        bd.t("  \x01YExit#      Category      ID             RIP              Status\x01W\n");

        let len = ach.dcr.len();
        let ay = if len > 20 { len - 20 } else { 0 };
        for bt in &ach.dcr[ay..] {
            bd.t(&format!("  {:>8}   {:<14}0x{:<12X} 0x{:<14X} {}{}\x01W\n",
                bt.eqb, bt.gb.as_str(),
                bt.cys, bt.wb,
                bt.status.cpk(), bt.status.as_str()));
        }
        bd.push('\n');
    }

    
    bd.t("  \x01C── Recommendations ────────────────────────────────────────\x01W\n");
    let xkt: u64 = ach.gvn.alv().sum();
    if xkt == 0 {
        bd.t("  \x01G✓ All VM exits are handled! VM is fully functional.\x01W\n");
    } else {
        
        let moa: Vec<_> = ach.cm.iter()
            .hi(|((rx, _), e)| 
                oh!(rx, DebugCategory::Iu | DebugCategory::Lr)
                && oh!(e.status, HandleStatus::Id))
            .collect();
        if !moa.is_empty() {
            bd.t(&format!("  \x01R✗ {} unhandled I/O port(s)\x01W — implement handlers in handle_io()\n",
                moa.len()));
            for ((_, port), hm) in moa.iter().take(5) {
                bd.t(&format!("    → 0x{:04X} {} ({}x)\n", port, hm.j, hm.az));
            }
        }

        let mob: Vec<_> = ach.cm.iter()
            .hi(|((rx, _), e)| 
                oh!(rx, DebugCategory::Hx | DebugCategory::Jr)
                && oh!(e.status, HandleStatus::Id))
            .collect();
        if !mob.is_empty() {
            bd.t(&format!("  \x01R✗ {} unhandled MSR(s)\x01W — implement in handle_msr()\n",
                mob.len()));
            for ((_, msr), hm) in mob.iter().take(5) {
                bd.t(&format!("    → 0x{:08X} {} ({}x)\n", msr, hm.j, hm.az));
            }
        }

        let lpd: Vec<_> = ach.cm.iter()
            .hi(|((rx, _), e)| 
                oh!(rx, DebugCategory::Qe)
                && oh!(e.status, HandleStatus::Id | HandleStatus::Nd))
            .collect();
        if !lpd.is_empty() {
            bd.t(&format!("  \x01R✗ {} unhandled NPF fault address(es)\x01W — add MMIO region handlers\n",
                lpd.len()));
            for ((_, pe), hm) in lpd.iter().take(5) {
                bd.t(&format!("    → GPA 0x{:X} ({}x, last RIP=0x{:X})\n", pe, hm.az, hm.jcp));
            }
        }
    }

    bd.push('\n');
    bd
}


pub fn kyr() -> String {
    let bvm = CT_.lock();
    let ach = match bvm.as_ref() {
        Some(ef) => ef,
        None => return String::from("Debug monitor not initialized."),
    };

    let mut bd = String::fc(2048);
    bd.t("\x01C═══ VM Debug: Unhandled Operations Report ═══\x01W\n\n");

    let mut ckr: Vec<_> = ach.cm.iter()
        .hi(|((_, _), e)| !oh!(e.status, HandleStatus::Gw))
        .collect();
    ckr.bxe(|q, o| o.1.az.cmp(&q.1.az));

    if ckr.is_empty() {
        bd.t("  \x01G✓ No gaps detected — all operations handled!\x01W\n");
    } else {
        bd.t(&format!("  \x01RFound {} unhandled/stubbed operations:\x01W\n\n", ckr.len()));
        bd.t("  \x01YCategory      ID             Name                   Count  First RIP        Detail\x01W\n");
        for ((rx, ad), hm) in &ckr {
            let hsi = if hm.j.len() > 22 { &hm.j[..22] } else { &hm.j };
            let kpl = if hm.etx.len() > 30 { &hm.etx[..30] } else { &hm.etx };
            bd.t(&format!("  {}{:<14}\x01W0x{:<12X} {:<22} {:>5}  0x{:<14X} {}\n",
                hm.status.cpk(), rx.as_str(), ad, hsi,
                hm.az, hm.nuv, kpl));
        }
    }

    bd.push('\n');
    bd
}


pub fn nyd() -> String {
    let bvm = CT_.lock();
    let ach = match bvm.as_ref() {
        Some(ef) => ef,
        None => return String::from("Debug monitor not initialized."),
    };

    let mut bd = String::fc(2048);
    bd.t("\x01C═══ VM Debug: I/O Port Heatmap ═══\x01W\n\n");

    
    let bnz = [
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

    bd.t("  \x01YPort Range     Device               IN Count   OUT Count  Status\x01W\n");

    for (ay, ci, j) in &bnz {
        let odo: u64 = ach.cm.iter()
            .hi(|((rx, port), _)| oh!(rx, DebugCategory::Iu) && *port >= *ay && *port < *ci)
            .map(|(_, e)| e.az)
            .sum();
        let ote: u64 = ach.cm.iter()
            .hi(|((rx, port), _)| oh!(rx, DebugCategory::Lr) && *port >= *ay && *port < *ci)
            .map(|(_, e)| e.az)
            .sum();
        
        if odo > 0 || ote > 0 {
            let qjd = ach.cm.iter()
                .any(|((rx, port), e)| 
                    oh!(rx, DebugCategory::Iu | DebugCategory::Lr)
                    && *port >= *ay && *port < *ci
                    && oh!(e.status, HandleStatus::Id));
            let status = if qjd { "\x01RMISS\x01W" } else { "\x01GOK\x01W" };
            bd.t(&format!("  0x{:04X}-0x{:04X} {:<20} {:>8}   {:>8}   {}\n",
                ay, ci - 1, j, odo, ote, status));
        }
    }

    
    let pwz: Vec<_> = ach.cm.iter()
        .hi(|((rx, port), e)| 
            oh!(rx, DebugCategory::Iu | DebugCategory::Lr)
            && !bnz.iter().any(|(ay, ci, _)| *port >= *ay && *port < *ci))
        .collect();
    
    if !pwz.is_empty() {
        bd.t("\n  \x01R── Unknown Ports ──\x01W\n");
        for ((rx, port), hm) in &pwz {
            let te = if oh!(rx, DebugCategory::Iu) { "IN " } else { "OUT" };
            bd.t(&format!("  {} 0x{:04X}  {} ({}x) RIP=0x{:X}\n",
                te, port, hm.j, hm.az, hm.jcp));
        }
    }

    bd.push('\n');
    bd
}


pub fn nyf() -> String {
    let bvm = CT_.lock();
    let ach = match bvm.as_ref() {
        Some(ef) => ef,
        None => return String::from("Debug monitor not initialized."),
    };

    let mut bd = String::fc(2048);
    bd.t("\x01C═══ VM Debug: MSR Access Report ═══\x01W\n\n");

    let mut lmy: Vec<_> = ach.cm.iter()
        .hi(|((rx, _), _)| oh!(rx, DebugCategory::Hx | DebugCategory::Jr))
        .collect();
    lmy.bxf(|((_, msr), _)| *msr);

    if lmy.is_empty() {
        bd.t("  No MSR accesses recorded.\n");
    } else {
        bd.t("  \x01YDir     MSR            Name                        Count  Value/Detail           Status\x01W\n");
        for ((rx, msr), hm) in &lmy {
            let te = if oh!(rx, DebugCategory::Hx) { "READ " } else { "WRITE" };
            let hsi = if hm.j.len() > 26 { &hm.j[..26] } else { &hm.j };
            let kpl = if hm.etx.len() > 20 { &hm.etx[..20] } else { &hm.etx };
            bd.t(&format!("  {}  0x{:08X}     {:<26} {:>5}  {:<20}   {}{}\x01W\n",
                te, msr, hsi, hm.az, kpl,
                hm.status.cpk(), hm.status.as_str()));
        }
    }

    bd.push('\n');
    bd
}


pub fn nys(az: usize) -> String {
    let bvm = CT_.lock();
    let ach = match bvm.as_ref() {
        Some(ef) => ef,
        None => return String::from("Debug monitor not initialized."),
    };

    let mut bd = String::fc(2048);
    bd.t("\x01C═══ VM Debug: Exit Timeline ═══\x01W\n\n");

    if ach.dcr.is_empty() {
        bd.t("  No events recorded yet.\n");
    } else {
        let rym = az.v(ach.dcr.len());
        let ay = ach.dcr.len() - rym;

        bd.t("  \x01YExit#      Category      ID             RIP              Name                   Status\x01W\n");
        for bt in &ach.dcr[ay..] {
            let j = odb(bt.gb, bt.cys);
            let hsi = if j.len() > 22 { &j[..22] } else { &j };
            bd.t(&format!("  {:>8}   {:<14}0x{:<12X} 0x{:<14X} {:<22} {}{}\x01W\n",
                bt.eqb, bt.gb.as_str(),
                bt.cys, bt.wb,
                hsi,
                bt.status.cpk(), bt.status.as_str()));
        }
    }

    bd.push('\n');
    bd
}


pub fn apa() {
    if let Some(ref mut ach) = *CT_.lock() {
        ach.cm.clear();
        ach.dcr.clear();
        ach.ido = 0;
        ach.hcj.clear();
        ach.gvn.clear();
        ach.ckr.clear();
        ET_.store(0, Ordering::Relaxed);
    }
}


pub fn jtr() -> u64 {
    ET_.load(Ordering::Relaxed)
}


pub fn jup() -> u64 {
    let bvm = CT_.lock();
    match bvm.as_ref() {
        Some(ach) => ach.gvn.alv().sum(),
        None => 0,
    }
}






fn odb(gb: DebugCategory, ad: u64) -> String {
    match gb {
        DebugCategory::Iu | DebugCategory::Lr => trm(ad as u16),
        DebugCategory::Hx | DebugCategory::Jr => tro(ad as u32),
        DebugCategory::Ahg => format!("CPUID leaf 0x{:X}", ad),
        DebugCategory::Qe => trn(ad),
        DebugCategory::Fv => format!("IRQ {}", ad),
        DebugCategory::Acd => format!("VMCALL 0x{:X}", ad),
        DebugCategory::Aqg => match ad {
            0 => String::from("CR0"),
            3 => String::from("CR3"),
            4 => String::from("CR4"),
            _ => format!("CR{}", ad),
        },
        DebugCategory::Ahu => match ad {
            0 => String::from("#DE Divide Error"),
            1 => String::from("#DB Debug"),
            3 => String::from("#BP Breakpoint"),
            6 => String::from("#UD Invalid Opcode"),
            7 => String::from("#NM No Math"),
            8 => String::from("#DF Double Fault"),
            13 => String::from("#GP General Protection"),
            14 => String::from("#PF Page Fault"),
            _ => format!("Exception #{}", ad),
        },
        DebugCategory::Qg => format!("0x{:X}", ad),
    }
}

fn trm(port: u16) -> String {
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

fn tro(msr: u32) -> String {
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

fn trn(pe: u64) -> String {
    match pe {
        0x0000..=0x0FFF => String::from("Real-mode IVT/BDA"),
        0xA0000..=0xBFFFF => String::from("VGA Frame Buffer"),
        0xC0000..=0xDFFFF => String::from("ROM/BIOS Shadow"),
        0xE0000..=0xFFFFF => String::from("High BIOS"),
        0xFEC0_0000..=0xFEC0_0FFF => String::from("I/O APIC"),
        0xFED0_0000..=0xFED0_03FF => String::from("HPET"),
        0xFEE0_0000..=0xFEE0_0FFF => String::from("Local APIC"),
        _ if pe >= 0x1_0000_0000 => String::from("High MMIO (>4GB)"),
        _ => format!("GPA 0x{:X}", pe),
    }
}






pub fn ky() -> bool {
    CT_.lock().is_some()
}


pub fn yhl(rx: DebugCategory) -> u64 {
    let bvm = CT_.lock();
    match bvm.as_ref() {
        Some(ach) => ach.hcj.get(&rx).hu().unwrap_or(0),
        None => 0,
    }
}


pub fn ztm() -> usize {
    let bvm = CT_.lock();
    match bvm.as_ref() {
        Some(ach) => ach.cm.len(),
        None => 0,
    }
}
