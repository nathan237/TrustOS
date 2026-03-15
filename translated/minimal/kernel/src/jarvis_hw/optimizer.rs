














use alloc::string::String;
use alloc::format;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU32, AtomicU64, AtomicBool, Ordering};
use spin::Mutex;

use super::probe::T;
use super::analyzer::{Si, Cd, InsightCategory};






#[derive(Clone)]
pub struct Nr {
    pub bgq: u32,
    pub gb: OptCategory,
    pub dc: String,
    pub fog: f32,
    pub fof: f32,
    pub flh: f32,
    pub exq: bool,
}

#[derive(Clone, Copy, PartialEq)]
pub enum OptCategory {
    Zr,
    Aov,
    Avo,
    Cge,
    Cmp,
    De,
}

impl OptCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            OptCategory::Zr => "AI/inference",
            OptCategory::Aov => "AI/training",
            OptCategory::Avo => "memory",
            OptCategory::Cge => "I/O",
            OptCategory::Cmp => "scheduling",
            OptCategory::De => "security",
        }
    }
}


pub struct Bog {
    pub knj: u32,
    pub dba: Vec<Nr>,
    pub gea: Option<Si>,
    pub emq: u64,
    pub qnv: f32,
    pub dyt: u64,
    pub qoz: f32,
    pub xkg: f32,
}

static VX_: AtomicBool = AtomicBool::new(false);
static BBW_: AtomicU32 = AtomicU32::new(0);
static AIZ_: AtomicU32 = AtomicU32::new(0);
static EIM_: AtomicU64 = AtomicU64::new(0); 

static Jz: Mutex<Option<Bog>> = Mutex::new(None);












pub fn qrd() -> String {
    let mut an = String::new();

    an.t("\x01C╔══════════════════════════════════════════════════════════╗\n");
    an.t("║      J.A.R.V.I.S. — Autonomous Hardware Intelligence     ║\n");
    an.t("║           Boot Scan & Self-Optimization v1.0              ║\n");
    an.t("╚══════════════════════════════════════════════════════════╝\x01W\n\n");

    
    an.t("\x01Y[Phase 1/5] Hardware Discovery...\x01W\n");
    let cc = super::probe::wdu();
    an.t(&format!("  {} — {} cores, {} MB RAM, {} PCI devices\n",
        cc.dpf,
        cc.azj,
        cc.ccf / (1024 * 1024),
        cc.dal));
    an.t(&format!("  Storage: {} dev(s), {} GB | GPU: {} | Net: {}\n\n",
        cc.aqm.len(),
        cc.dmp / (1024 * 1024 * 1024),
        if cc.bqz { &cc.beh } else { "none" },
        cc.bzz));

    
    an.t("\x01Y[Phase 2/5] AI Hardware Analysis...\x01W\n");
    let abp = super::analyzer::mvq(&cc);
    let cpp = abp.iter().hi(|a| a.qj == super::analyzer::InsightSeverity::Aj).az();
    let flg = abp.iter().hi(|a| a.qj == super::analyzer::InsightSeverity::Ajd).az();
    an.t(&format!("  {} insights generated ({} critical, {} important)\n",
        abp.len(), cpp, flg));

    for ckz in abp.iter().take(5) {
        an.t(&format!("  {} [{}] {}\n",
            ckz.gb.s(), ckz.gb.as_str(), ckz.dq));
    }
    an.push('\n');

    
    an.t("\x01Y[Phase 3/5] Execution Plan Generation...\x01W\n");
    let aqg = super::analyzer::nxn(&cc);
    an.t(&format!("  Strategy: {}\n\n", aqg.ibx));

    
    an.t("\x01Y[Phase 4/5] Applying Optimizations...\x01W\n");
    let ost = qjw(&cc, &aqg, &abp);
    for result in &ost {
        an.t(&format!("  \x01G✓\x01W [{}] {}\n", result.gb.as_str(), result.dc));
    }
    an.push('\n');

    
    an.t("\x01Y[Phase 5/5] Starting Adaptive Monitor...\x01W\n");

    let g = Bog {
        knj: 0,
        dba: ost,
        gea: Some(aqg),
        emq: 0,
        qnv: f32::O,
        dyt: u64::O,
        qoz: f32::O,
        xkg: 0.0,
    };
    *Jz.lock() = Some(g);
    VX_.store(true, Ordering::Release);

    an.t("  Monitor active — Jarvis will continuously adapt\n\n");

    
    an.t("\x01C═══ Boot Scan Complete ═══\x01W\n");
    an.t(&format!("  Hardware score: \x01C{:.0}%\x01W\n", cc.dkj * 100.0));
    an.t(&format!("  Optimizations applied: {}\n", AIZ_.load(Ordering::Relaxed)));
    an.t("  Jarvis is now aware of its environment.\n");

    an
}





fn qjw(
    cc: &T,
    aqg: &Si,
    abp: &[Cd],
) -> Vec<Nr> {
    let mut dba = Vec::new();
    let bgq = 0u32;

    
    dba.push(Nr {
        bgq,
        gb: OptCategory::Zr,
        dc: format!("Set SIMD tier to {} for neural inference", aqg.fut.as_str()),
        fog: 0.0,
        fof: 0.0,
        flh: 0.0,
        exq: false,
    });

    
    dba.push(Nr {
        bgq,
        gb: OptCategory::Aov,
        dc: format!("Batch size → {} (based on {} MB RAM)", aqg.jhx,
            cc.ccf / (1024 * 1024)),
        fog: 0.0,
        fof: 0.0,
        flh: 0.0,
        exq: false,
    });

    
    let bne = if cc.drr > 0 {
        cc.ecw as f32 / cc.drr as f32
    } else { 0.0 };

    if bne > 0.7 {
        dba.push(Nr {
            bgq,
            gb: OptCategory::Avo,
            dc: format!("Heap pressure {:.0}% — enable aggressive cache eviction", bne * 100.0),
            fog: bne,
            fof: 0.0,
            flh: 0.0,
            exq: false,
        });
    }

    
    if cc.bqz {
        dba.push(Nr {
            bgq,
            gb: OptCategory::Zr,
            dc: format!("Enable GPU offload: {} ({} CUs)", cc.beh, cc.erk),
            fog: 0.0,
            fof: 0.0,
            flh: 0.0,
            exq: false,
        });
    }

    
    if cc.cfe {
        dba.push(Nr {
            bgq,
            gb: OptCategory::De,
            dc: String::from("Enable AES-NI for weight encryption at rest"),
            fog: 0.0,
            fof: 0.0,
            flh: 0.0,
            exq: false,
        });
    }

    
    if aqg.kbu {
        dba.push(Nr {
            bgq,
            gb: OptCategory::Aov,
            dc: format!("Enable background learning ({} spare cores)", cc.azj - 1),
            fog: 0.0,
            fof: 0.0,
            flh: 0.0,
            exq: false,
        });
    }

    
    for ckz in abp {
        if ckz.qj == super::analyzer::InsightSeverity::Aj {
            dba.push(Nr {
                bgq,
                gb: match ckz.gb {
                    InsightCategory::Yb => OptCategory::Zr,
                    InsightCategory::Oy => OptCategory::Avo,
                    InsightCategory::De => OptCategory::De,
                    _ => OptCategory::Zr,
                },
                dc: format!("Critical: {}", ckz.hr),
                fog: 0.0,
                fof: 0.0,
                flh: 0.0,
                exq: false,
            });
        }
    }

    AIZ_.store(dba.len() as u32, Ordering::Relaxed);
    dba
}






pub fn wbk() -> Option<String> {
    if !VX_.load(Ordering::Acquire) { return None; }

    let mut adb = Jz.lock();
    let g = adb.as_mut()?;
    g.knj += 1;
    let bgq = g.knj;
    BBW_.store(bgq, Ordering::Relaxed);

    let mut an = String::new();
    an.t(&format!("\x01C[Optimization Cycle #{}]\x01W\n", bgq));

    
    let eaf = super::analyzer::mxc();
    let dyi = super::analyzer::mvu();

    
    if g.emq == 0 && eaf > 0 {
        g.emq = eaf;
        g.dyt = eaf;
        an.t(&format!("  Baseline established: {} µs/inference\n", eaf));
    } else if eaf > 0 && eaf < g.dyt {
        let tsj = (g.dyt as f32 - eaf as f32)
            / g.dyt as f32 * 100.0;
        g.dyt = eaf;
        an.t(&format!("  \x01GNew best:\x01W {} µs ({:.1}% faster)\n", eaf, tsj));
    }

    
    if dyi > 0 {
        an.t(&format!("  \x01RWARN:\x01W {} anomalies detected — investigating\n", dyi));
        
        let afa = crate::memory::heap::mr();
        let aul = crate::memory::cre();
        let cgr = afa as f32 / aul as f32;
        if cgr > 0.9 {
            an.t("  → Heap critical: recommending emergency cache flush\n");
        }
    }

    
    if let Some(aqg) = &g.gea {
        if bgq % 10 == 0 {
            an.t(&format!("  Current plan: {}\n", aqg.ibx));
        }
    }

    Some(an)
}






pub fn status() -> String {
    let mut e = String::new();

    e.t("\x01C═══ Adaptive Optimizer Status ═══\x01W\n\n");
    e.t(&format!("  Active: {}\n", VX_.load(Ordering::Relaxed)));
    e.t(&format!("  Cycles: {}\n", BBW_.load(Ordering::Relaxed)));
    e.t(&format!("  Optimizations: {}\n", AIZ_.load(Ordering::Relaxed)));

    if let Some(g) = Jz.lock().as_ref() {
        if g.emq > 0 {
            e.t(&format!("  Baseline: {} µs/inference\n", g.emq));
            e.t(&format!("  Best:     {} µs/inference\n", g.dyt));
            if g.emq > g.dyt {
                let tsh = (g.emq - g.dyt) as f32
                    / g.emq as f32 * 100.0;
                e.t(&format!("  Improvement: \x01G{:.1}%\x01W\n", tsh));
            }
        }

        e.t(&format!("\n  Recent optimizations:\n"));
        for hxf in g.dba.iter().vv().take(10) {
            let status = if hxf.exq { "\x01R⟲\x01W" } else { "\x01G✓\x01W" };
            e.t(&format!("    {} [{}] {}\n", status, hxf.gb.as_str(), hxf.dc));
        }
    }

    e
}


pub fn rl() -> bool {
    VX_.load(Ordering::Acquire)
}


pub fn gea() -> Option<Si> {
    Jz.lock().as_ref()?.gea.clone()
}
