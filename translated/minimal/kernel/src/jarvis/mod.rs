





















































pub mod tokenizer;
pub mod model;
pub mod inference;
pub mod agent;
pub mod mentor;
pub mod training;
pub mod corpus;
pub mod backprop;
pub mod optimizer;
pub mod simd;
pub mod compute;
pub mod hw_corpus;
pub mod mesh;
pub mod rpc;
pub mod consensus;
pub mod federated;
pub mod pxe_replicator;
pub mod guardian;
pub mod task;
pub mod compression;
pub mod io_control;
pub mod micro_model;

use alloc::string::String;
use alloc::format;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicBool, AtomicU8, AtomicU64, Ordering};
use spin::Mutex;

use model::TransformerWeights;
use inference::InferenceEngine;
use optimizer::AdamState;
use micro_model::{MicroWeights, MicroEngine};





static Be: AtomicBool = AtomicBool::new(false);
static FF_: AtomicU64 = AtomicU64::new(0);
static BW_: AtomicU64 = AtomicU64::new(0);


static Ci: Mutex<Option<TransformerWeights>> = Mutex::new(None);


static Abk: Mutex<Option<InferenceEngine>> = Mutex::new(None);


static Ado: Mutex<Option<AdamState>> = Mutex::new(None);


static Avb: AtomicU8 = AtomicU8::new(0);


static CKU_: AtomicBool = AtomicBool::new(false);


static IK_: Mutex<Option<MicroWeights>> = Mutex::new(None);


static VF_: Mutex<Option<MicroEngine>> = Mutex::new(None);


static FE_: AtomicBool = AtomicBool::new(false);









pub fn init() {
    crate::serial_println!("[JARVIS] Two-tier brain init starting...");

    
    static OF_: &[u8] = include_bytes!("jarvis_micro.bin");
    if OF_.len() >= 4 && OF_.len() % 4 == 0 {
        let hjv = OF_.len() / 4;
        let aue: &[f32] = unsafe {
            core::slice::anh(OF_.fq() as *const f32, hjv)
        };
        if let Some(onf) = MicroWeights::eos(aue) {
            crate::serial_println!("[JARVIS] Micro sentinel loaded ({} params, {} KB)",
                onf.vm(), OF_.len() / 1024);
            *IK_.lock() = Some(onf);
            *VF_.lock() = Some(MicroEngine::new());
        } else {
            crate::serial_println!("[JARVIS] WARN: micro sentinel deserialize failed, using random");
            *IK_.lock() = Some(MicroWeights::dtm());
            *VF_.lock() = Some(MicroEngine::new());
        }
    } else {
        crate::serial_println!("[JARVIS] WARN: micro binary invalid, using random");
        *IK_.lock() = Some(MicroWeights::dtm());
        *VF_.lock() = Some(MicroEngine::new());
    }

    
    let status = micro_model::ubc();
    crate::serial_println!("[JARVIS] Kernel validation: heap={} int={} fs={} serial={}",
        status.obl, status.ofb, status.kxj, status.pif);

    
    simd::oei();
    let backend = compute::nky();
    match backend {
        compute::Backend::Ot => crate::serial_println!("[JARVIS] Compute: AMD GPU detected"),
        compute::Backend::Pd => crate::serial_println!("[JARVIS] Compute: CPU SSE2 SIMD"),
    }

    
    Be.store(true, Ordering::Release);

    
    if status.kxj {
        match ljg() {
            Ok(bf) => {
                crate::serial_println!("[JARVIS] Full brain loaded from FS ({} KB)", bf / 1024);
                
                let _ = crate::ramfs::fh(|fs| fs.hb(HI_));
                
                
                Avb.store(3, Ordering::Relaxed);
                crate::serial_println!("[JARVIS] Maturity: {} (level {})",
                    lkq(), lkp());
            }
            Err(aa) => {
                crate::serial_println!("[JARVIS] Full brain not available: {} — micro sentinel active", aa);
            }
        }
    } else {
        crate::serial_println!("[JARVIS] FS not ready — micro sentinel only mode");
    }

    crate::serial_println!("[JARVIS] Init complete. Micro={}, Full={}",
        if IK_.lock().is_some() { "OK" } else { "FAIL" },
        if FE_.load(Ordering::Relaxed) { "LOADED" } else { "NOT LOADED" });

    
    if fkf() && !mesh::rl() {
        jfy();
        crate::serial_println!("[JARVIS] Mesh auto-started (full brain available for peers)");
    }
}


pub fn ljg() -> Result<usize, &'static str> {
    let f = crate::ramfs::fh(|fs| {
        fs.mq(HI_).map(|bc| bc.ip())
    }).jd(|_| "weights.bin not found in FS")?;

    if f.len() % 4 != 0 || f.len() < 1024 {
        return Err("Invalid weight file");
    }

    let hjv = f.len() / 4;
    let aue: &[f32] = unsafe {
        core::slice::anh(f.fq() as *const f32, hjv)
    };

    let bix = model::TransformerWeights::eos(aue)
        .ok_or("Deserialization failed (wrong param count)")?;

    let vm = bix.vm();

    
    if compute::gil() {
        let _ = compute::mog(&bix);
    }

    let engine = InferenceEngine::new();
    let ccv = AdamState::mqt(vm, 0.001);

    *Ci.lock() = Some(bix);
    *Abk.lock() = Some(engine);
    *Ado.lock() = Some(ccv);
    FE_.store(true, Ordering::Release);

    crate::serial_println!("[JARVIS] Full brain: {} params ({} KB)",
        vm, f.len() / 1024);

    Ok(f.len())
}






fn hoe(f: &[u8], iy: &str) -> Result<usize, &'static str> {
    if f.len() % 4 != 0 || f.len() < 1024 {
        return Err("Invalid weight data (bad size or alignment)");
    }

    let hjv = f.len() / 4;
    let aue: &[f32] = unsafe {
        core::slice::anh(f.fq() as *const f32, hjv)
    };

    let bix = model::TransformerWeights::eos(aue)
        .ok_or("Deserialization failed (wrong param count)")?;

    let vm = bix.vm();

    if compute::gil() {
        let _ = compute::mog(&bix);
    }

    let engine = InferenceEngine::new();
    let ccv = AdamState::mqt(vm, 0.001);

    *Ci.lock() = Some(bix);
    *Abk.lock() = Some(engine);
    *Ado.lock() = Some(ccv);
    FE_.store(true, Ordering::Release);

    crate::serial_println!("[JARVIS] Full brain loaded from {}: {} params ({} KB)",
        iy, vm, f.len() / 1024);

    Ok(f.len())
}


pub fn ugn(path: &str) -> Result<usize, &'static str> {
    crate::serial_println!("[JARVIS] Loading brain from VFS: {}", path);
    let f = crate::vfs::mq(path)
        .jd(|_| "File not found or read error on VFS path")?;
    hoe(&f, path)
}


pub fn ojs(it: Option<&str>) -> Result<usize, &'static str> {
    let path = match it {
        Some(j) => format!("/mnt/fat32/{}", j),
        None => String::from("/mnt/fat32/jarvis_weights.bin"),
    };
    crate::serial_println!("[JARVIS] Loading brain from FAT32: {}", path);
    let f = crate::vfs::mq(&path)
        .jd(|_| "File not found on FAT32 (is a FAT32 disk attached?)")?;
    hoe(&f, &path)
}


pub fn ugm(url: &str) -> Result<usize, &'static str> {
    crate::serial_println!("[JARVIS] Downloading brain from: {}", url);
    let mk = crate::netstack::http::get(url)?;
    if mk.wt != 200 {
        return Err("HTTP download failed (non-200 status)");
    }
    if mk.gj.is_empty() {
        return Err("HTTP response body is empty");
    }
    crate::serial_println!("[JARVIS] Downloaded {} KB", mk.gj.len() / 1024);
    hoe(&mk.gj, url)
}


pub fn gbx() -> Result<usize, &'static str> {
    let bet = Ci.lock();
    let model = bet.as_ref().ok_or("No brain loaded to cache")?;
    let aue = model.gsd();
    let aal = aue.len() * 4;
    let bf: &[u8] = unsafe {
        core::slice::anh(aue.fq() as *const u8, aal)
    };
    crate::ramfs::fh(|fs| {
        let _ = fs.ut(BIY_);
    });
    crate::ramfs::fh(|fs| {
        let _ = fs.touch(HI_);
        fs.ns(HI_, bf).jd(|_| "Cache write failed")
    })?;
    crate::serial_println!("[JARVIS] Brain cached to RamFS ({} KB)", aal / 1024);
    Ok(aal)
}


pub fn fkf() -> bool {
    FE_.load(Ordering::Acquire)
}


pub fn lkq() -> &'static str {
    match lkp() {
        0 => "Infant",
        1 => "Child",
        2 => "Teen",
        3 => "Adult",
        _ => "Unknown",
    }
}


fn yxs() {
    crate::serial_println!("[JARVIS] Initializing random full brain...");

    let bix = TransformerWeights::dtm();
    let vm = bix.vm();

    crate::serial_println!("[JARVIS] Model: {} layers, d_model={}, d_ff={}, {} heads",
        model::AZ_, model::E_, model::Y_, model::FP_);
    crate::serial_println!("[JARVIS] Parameters: {} ({} KB FP32)",
        vm, vm * 4 / 1024);

    simd::oei();
    let backend = compute::nky();
    match backend {
        compute::Backend::Ot => {
            crate::serial_println!("[JARVIS] Compute: AMD GPU — GEMM available");
            match compute::mog(&bix) {
                Ok(bf) => crate::serial_println!("[JARVIS] VRAM: {} KB", bf / 1024),
                Err(aa) => crate::serial_println!("[JARVIS] GPU fallback: {}", aa),
            }
        }
        compute::Backend::Pd => {
            crate::serial_println!("[JARVIS] Compute: CPU SSE2 SIMD");
        }
    }

    let engine = InferenceEngine::new();
    let ccv = AdamState::mqt(vm, 0.001);

    *Ci.lock() = Some(bix);
    *Abk.lock() = Some(engine);
    *Ado.lock() = Some(ccv);
    FE_.store(true, Ordering::Release);
    Be.store(true, Ordering::Release);

    crate::serial_println!("[JARVIS] Random full brain ready.");
}


pub fn uc() -> bool {
    Be.load(Ordering::Acquire)
}


pub fn lkp() -> u8 {
    Avb.load(Ordering::Relaxed)
}


pub fn ogq() -> bool {
    CKU_.load(Ordering::Relaxed)
}


fn pxg() {
    let vl = itc();
    let jy = if vl < 2.0 { 3 }
        else if vl < 3.5 { 2 }
        else if vl < 5.0 { 1 }
        else { 0 };
    Avb.store(jy, Ordering::Relaxed);
}







pub fn cks(aau: &str, bvi: usize) -> String {
    if !uc() {
        return String::from("[JARVIS brain not initialized]");
    }

    
    if FE_.load(Ordering::Acquire) {
        let bet = Ci.lock();
        let model = match bet.as_ref() {
            Some(ef) => ef,
            None => return lls(aau, bvi),
        };

        let mut hid = Abk.lock();
        let engine = match hid.as_mut() {
            Some(aa) => aa,
            None => return lls(aau, bvi),
        };

        let eb = tokenizer::cxj(aau);
        let kya = engine.cks(model, &eb, bvi);
        FF_.fetch_add(1, Ordering::Relaxed);
        return tokenizer::hfo(&kya);
    }

    
    lls(aau, bvi)
}


fn lls(aau: &str, bvi: usize) -> String {
    let uoe = IK_.lock();
    let gms = match uoe.as_ref() {
        Some(ef) => ef,
        None => return String::from("[micro sentinel not loaded]"),
    };

    let mut hid = VF_.lock();
    let engine = match hid.as_mut() {
        Some(aa) => aa,
        None => return String::from("[micro engine not loaded]"),
    };

    let eb: Vec<u8> = aau.bf().collect();
    let kgm = bvi.v(micro_model::DK_);
    let kya = engine.cks(gms, &eb, kgm);
    FF_.fetch_add(1, Ordering::Relaxed);
    tokenizer::hfo(&kya)
}


pub fn zge(context: &[u8]) -> u8 {
    if !uc() { return b'?' }

    let bet = Ci.lock();
    let model = match bet.as_ref() {
        Some(ef) => ef,
        None => return b'?',
    };

    let mut hid = Abk.lock();
    let engine = match hid.as_mut() {
        Some(aa) => aa,
        None => return b'?',
    };

    engine.vko(model, context)
}








pub fn ekd(text: &str, dsr: f32) -> f32 {
    if !uc() { return f32::O; }

    
    if let Err(fr) = guardian::emj(guardian::ProtectedOp::Zf) {
        crate::serial_println!("[JARVIS] {}", fr);
        return f32::O;
    }

    let eb = tokenizer::cxj(text);
    if eb.len() < 2 { return f32::O; }

    
    let mut hts = Ado.lock();
    if let Some(ccv) = hts.as_mut() {
        let mut bet = Ci.lock();
        let model = match bet.as_mut() {
            Some(ef) => ef,
            None => return f32::O,
        };

        ccv.aad = dsr;
        let (vl, mut arg) = backprop::ivk(model, &eb);
        arg.hcy(1.0);
        ccv.gu(model, &arg);
        BW_.fetch_add(1, Ordering::Relaxed);
        return vl;
    }
    drop(hts);

    
    let mut bet = Ci.lock();
    let model = match bet.as_mut() {
        Some(ef) => ef,
        None => return f32::O,
    };
    let vl = training::pvw(model, &eb, dsr);
    BW_.fetch_add(1, Ordering::Relaxed);
    vl
}






pub fn zl() -> Vec<String> {
    let mut ak = Vec::new();
    ak.push(String::from("╔═══════════════════════════════════════════════════╗"));
    ak.push(String::from("║   J.A.R.V.I.S. Neural Brain v3.0 (Two-Tier)     ║"));
    ak.push(String::from("║   Micro Sentinel + Full Brain Architecture       ║"));
    ak.push(String::from("╠═══════════════════════════════════════════════════╣"));

    
    if let Some(gms) = IK_.lock().as_ref() {
        ak.push(format!("║ Micro:  {}L × d{} × {}H  ({} params, {:.0} KB)     ║",
            micro_model::FN_, micro_model::M_,
            micro_model::AFM_, gms.vm(),
            gms.vm() as f64 * 4.0 / 1024.0));
        ak.push(String::from("║ Role:   Kernel sentinel, validation, fallback    ║"));
    } else {
        ak.push(String::from("║ Micro:  NOT LOADED                               ║"));
    }

    ak.push(String::from("╠═══════════════════════════════════════════════════╣"));

    
    let auh = FE_.load(Ordering::Relaxed);
    if auh {
        if let Some(model) = Ci.lock().as_ref() {
            let oi = model.vm();
            ak.push(format!("║ Full:   {}L × d{} × {}H × ff{}  ({:.1}M params)  ║",
                model::AZ_, model::E_, model::FP_, model::Y_,
                oi as f64 / 1_000_000.0));
            ak.push(format!("║ Memory: {:.1} MB FP32   Vocab: {} (byte)          ║",
                oi as f64 * 4.0 / (1024.0 * 1024.0), model::BG_));
        }
        ak.push(format!("║ Status: LOADED from FS   Maturity: {} ({})    ║",
            lkp(), lkq()));
    } else {
        ak.push(String::from("║ Full:   NOT LOADED (use 'jarvis brain load')     ║"));
        ak.push(String::from("║ Status: MICRO ONLY MODE                          ║"));
    }

    ak.push(String::from("╠═══════════════════════════════════════════════════╣"));
    ak.push(format!("║ Generations:  {}                                  ║",
        FF_.load(Ordering::Relaxed)));
    ak.push(format!("║ Train steps:  {}                                  ║",
        BW_.load(Ordering::Relaxed)));
    let rnl = if compute::gil() {
        "║ Compute: GPU (AMD RDNA GEMM)                      ║"
    } else {
        "║ Compute: CPU (SSE2 SIMD)                          ║"
    };
    ak.push(String::from(rnl));
    ak.push(String::from("╚═══════════════════════════════════════════════════╝"));

    ak
}


pub fn cm() -> String {
    let auh = if FE_.load(Ordering::Relaxed) {
        format!("full={}K", if let Some(ef) = Ci.lock().as_ref() { ef.vm() / 1000 } else { 0 })
    } else {
        String::from("full=OFF")
    };
    let gms = if IK_.lock().is_some() { "micro=OK" } else { "micro=OFF" };
    format!("Jarvis: {} {} gens={} steps={} ready={}",
        gms, auh,
        FF_.load(Ordering::Relaxed),
        BW_.load(Ordering::Relaxed),
        uc())
}



pub fn zgr() -> bool {
    if !uc() { return false; }
    let ma = io_control::hkp();
    io_control::hsn(&ma)
}


pub fn zgs() -> Vec<String> {
    let mut ak = Vec::new();

    ak.push(String::from("╔═══════════════════════════════════════════════════╗"));
    ak.push(String::from("║  JARVIS NETWORK PROPAGATION STATUS               ║"));
    ak.push(String::from("╠═══════════════════════════════════════════════════╣"));

    
    ak.push(format!("║ Brain:       {} ({} steps, {} gens){}",
        if uc() { "READY" } else { "NOT READY" },
        BW_.load(Ordering::Relaxed),
        FF_.load(Ordering::Relaxed),
        "         ║"));

    
    ak.push(format!("║ Federated:   {}{}",
        if federated::zu() { "ENABLED" } else { "DISABLED" },
        "                            ║"));
    ak.push(format!("║ Fed Rounds:  {}  Interval: {}ms{}",
        federated::wah(),
        federated::rrx(),
        "              ║"));

    
    let ehz = federated::qvc();
    ak.push(format!("║ Bandwidth:   {} KB saved by compression{}",
        ehz / 1024, "          ║"));

    
    ak.push(format!("║ Mesh Peers:  {}  Role: {:?}{}",
        mesh::cti(),
        mesh::htw(),
        "                       ║"));

    
    let ma = io_control::hkp();
    let ol = io_control::gdg(&ma);
    let dr = io_control::nbs(&ma);
    ak.push(format!("║ I/O Score:   {}%  Caps: 0x{:04X}{}",
        ol, dr, "                   ║"));
    ak.push(format!("║ Net Ready:   {}  Full Ctrl: {}{}",
        if io_control::hsn(&ma) { "YES" } else { "NO " },
        if io_control::nwq(&ma) { "YES" } else { "NO " },
        "                  ║"));

    ak.push(String::from("╚═══════════════════════════════════════════════════╝"));
    ak
}


pub fn apa() {
    if let Some(model) = Ci.lock().as_mut() {
        model.apa();
        BW_.store(0, Ordering::Relaxed);
        FF_.store(0, Ordering::Relaxed);
    }
}





const BIY_: &str = "/jarvis";
const HI_: &str = "/jarvis/weights.bin";
const BIZ_: &str = "/jarvis/meta.txt";


pub fn pfn() -> Result<usize, &'static str> {
    let bet = Ci.lock();
    let model = bet.as_ref().ok_or("Model not loaded")?;

    let aue = model.gsd();
    let aal = aue.len() * 4;

    
    let bf: &[u8] = unsafe {
        core::slice::anh(aue.fq() as *const u8, aal)
    };

    
    crate::ramfs::fh(|fs| {
        let _ = fs.ut(BIY_); 
    });

    
    crate::ramfs::fh(|fs| {
        let _ = fs.touch(HI_);
        fs.ns(HI_, bf).jd(|_| "Write failed")
    })?;

    
    let meta = format!("params={}\nsteps={}\ngens={}\nbytes={}\n",
        model.vm(),
        BW_.load(Ordering::Relaxed),
        FF_.load(Ordering::Relaxed),
        aal);
    crate::ramfs::fh(|fs| {
        let _ = fs.touch(BIZ_);
        let _ = fs.ns(BIZ_, meta.as_bytes());
    });

    crate::serial_println!("[JARVIS] Saved {} floats ({} KB) to {}",
        aue.len(), aal / 1024, HI_);

    Ok(aal)
}


pub fn oka() -> Result<usize, &'static str> {
    ljg()
}


pub fn tne() -> bool {
    crate::ramfs::fh(|fs| fs.aja(HI_))
}







pub fn usi(query: &str) -> Option<String> {
    if !uc() {
        init();
    }
    if !uc() { return None; }

    
    if !FE_.load(Ordering::Relaxed) && tne() {
        let _ = ljg();
    }

    let an = cks(query, 64);
    if an.is_empty() { return None; }
    Some(an)
}


pub fn zan(pxr: &str, nzj: &str) {
    if !uc() { return; }
    
    let mut jub = String::fc(pxr.len() + nzj.len() + 1);
    jub.t(pxr);
    jub.push('\n');
    jub.t(nzj);
    let _ = ekd(&jub, 0.0005); 
}








pub fn oxe(bmz: usize, aad: f32) -> (usize, f32, u64) {
    if !uc() { init(); }
    if !uc() { return (0, f32::O, 0); }

    let ay = crate::time::ave();
    let jtx = corpus::ien();
    let mut tk = jtx * bmz;
    let eex = aad * 0.1; 
    let mut gu = 0u64;
    let mut ayy = 0.0f32;
    let mut dje = 0u32;

    
    const AKD_: usize = 4;

    
    let lcx = if crate::jarvis_hw::probe::tyv() {
        if let Some(cc) = crate::jarvis_hw::probe::gby() {
            let bwy = hw_corpus::cks(&cc);
            crate::serial_println!("[JARVIS] Hardware corpus: {} sequences from live probe", bwy.len());
            bwy
        } else {
            alloc::vec::Vec::new()
        }
    } else {
        alloc::vec::Vec::new()
    };

    
    tk += lcx.len() * bmz;
    let fbf = (tk / 10).am(5) as u64; 

    crate::serial_println!("[JARVIS] Pre-training: {} phases + HW, {} sequences, {} epoch(s), lr_peak={}, warmup={}",
        corpus::htg(), tk, bmz, aad, fbf);

    let mut hts = Ado.lock();
    let mut bet = Ci.lock();

    if let (Some(ccv), Some(model)) = (hts.as_mut(), bet.as_mut()) {
        for isz in 0..bmz {
            for (ovg, ib) in corpus::Gr.iter().cf() {
                let mut ovh = 0.0f32;
                let mut jjf = 0u32;
                let mut bpp = backprop::ModelGrads::new();
                let mut cog = 0usize;
                let mut mtj = 0.0f32;

                for &text in *ib {
                    let eb = tokenizer::cxj(text);
                    if eb.len() < 2 { gu += 1; continue; }

                    
                    let hew = optimizer::kkw(
                        gu, tk as u64, fbf, aad, eex
                    );
                    ccv.aad = hew;

                    
                    let (vl, arg) = backprop::ivk(model, &eb);
                    if vl.dsg() {
                        bpp.mtk(&arg);
                        mtj += vl;
                        cog += 1;
                        ovh += vl;
                        jjf += 1;
                        ayy += vl;
                        dje += 1;
                    }

                    
                    if cog >= AKD_ {
                        bpp.bv(1.0 / cog as f32);
                        bpp.hcy(1.0);
                        ccv.gu(model, &bpp);
                        BW_.fetch_add(1, Ordering::Relaxed);
                        bpp.ajs();
                        cog = 0;
                        mtj = 0.0;
                    }

                    gu += 1;
                }

                
                if cog > 0 {
                    bpp.bv(1.0 / cog as f32);
                    bpp.hcy(1.0);
                    ccv.gu(model, &bpp);
                    BW_.fetch_add(1, Ordering::Relaxed);
                    bpp.ajs();
                }

                let abl = if jjf > 0 { ovh / jjf as f32 } else { 0.0 };
                crate::serial_println!("[JARVIS] Epoch {}/{} Phase {} ({}) — {} seqs, avg loss={:.3}",
                    isz + 1, bmz, ovg, corpus::jjg(ovg),
                    jjf, abl);
            }

            
            if !lcx.is_empty() {
                let mut oco = 0.0f32;
                let mut iza = 0u32;
                let mut bpp = backprop::ModelGrads::new();
                let mut cog = 0usize;

                for text in &lcx {
                    let eb = tokenizer::cxj(text);
                    if eb.len() < 2 { gu += 1; continue; }

                    let hew = optimizer::kkw(
                        gu, tk as u64, fbf, aad, eex
                    );
                    ccv.aad = hew;

                    let (vl, arg) = backprop::ivk(model, &eb);
                    if vl.dsg() {
                        bpp.mtk(&arg);
                        cog += 1;
                        oco += vl;
                        iza += 1;
                        ayy += vl;
                        dje += 1;
                    }

                    if cog >= AKD_ {
                        bpp.bv(1.0 / cog as f32);
                        bpp.hcy(1.0);
                        ccv.gu(model, &bpp);
                        BW_.fetch_add(1, Ordering::Relaxed);
                        bpp.ajs();
                        cog = 0;
                    }

                    gu += 1;
                }

                if cog > 0 {
                    bpp.bv(1.0 / cog as f32);
                    bpp.hcy(1.0);
                    ccv.gu(model, &bpp);
                    BW_.fetch_add(1, Ordering::Relaxed);
                }

                let abl = if iza > 0 { oco / iza as f32 } else { 0.0 };
                crate::serial_println!("[JARVIS] Epoch {}/{} Phase HW (Hardware Context) — {} seqs, avg loss={:.3}",
                    isz + 1, bmz, iza, abl);
            }
        }
    } else {
        
        drop(hts);
        drop(bet);
        for xyz in 0..bmz {
            for ib in corpus::Gr {
                for &text in *ib {
                    let vl = ekd(text, aad);
                    if vl.dsg() { ayy += vl; dje += 1; }
                    gu += 1;
                }
            }
        }
    }

    let ez = crate::time::ave().ao(ay);
    let bdl = if dje > 0 { ayy / dje as f32 } else { f32::O };

    crate::serial_println!("[JARVIS] Pre-training done: {} steps, avg loss={:.3}, {} ms",
        gu, bdl, ez);

    (gu as usize, bdl, ez)
}



pub fn zgk(ib: usize, bmz: usize, aad: f32) -> (usize, f32, u64) {
    if !uc() { init(); }
    if !uc() { return (0, f32::O, 0); }
    if ib >= corpus::htg() { return (0, f32::O, 0); }

    let ay = crate::time::ave();
    let mut tk = 0usize;
    let mut ayy = 0.0f32;
    let mut dje = 0u32;

    let mdx = corpus::Gr[ib];
    let jtx = mdx.len() * bmz;
    let xtr = (jtx / 10).am(2) as u64;
    let eex = aad * 0.1;
    let mut gu = 0u64;

    crate::serial_println!("[JARVIS] Training phase {} ({}) — {} sequences, {} epoch(s)",
        ib, corpus::jjg(ib), mdx.len(), bmz);

    for isz in 0..bmz {
        for &text in mdx {
            let hew = optimizer::kkw(gu, jtx as u64, xtr, aad, eex);
            let vl = ekd(text, hew);
            if vl.dsg() {
                ayy += vl;
                dje += 1;
            }
            tk += 1;
            gu += 1;
        }
        if bmz > 1 {
            let abl = if dje > 0 { ayy / dje as f32 } else { 0.0 };
            crate::serial_println!("[JARVIS]   Epoch {}/{}: avg loss={:.3}", isz + 1, bmz, abl);
        }
    }

    let ez = crate::time::ave().ao(ay);
    let bdl = if dje > 0 { ayy / dje as f32 } else { f32::O };

    (tk, bdl, ez)
}


pub fn kud() -> f32 {
    if !uc() { return f32::O; }

    let bet = Ci.lock();
    let model = match bet.as_ref() {
        Some(ef) => ef,
        None => return f32::O,
    };

    let mut ayy = 0.0f32;
    let mut az = 0u32;

    for ib in corpus::Gr {
        for &text in *ib {
            let eb = tokenizer::cxj(text);
            if eb.len() < 2 { continue; }
            let (vl, _) = inference::cjq(model, &eb);
            if vl.dsg() {
                ayy += vl;
                az += 1;
            }
        }
    }

    let abl = if az > 0 { ayy / az as f32 } else { f32::O };
    abl
}


pub fn itc() -> f32 {
    if !uc() { return f32::O; }

    let bet = Ci.lock();
    let model = match bet.as_ref() {
        Some(ef) => ef,
        None => return f32::O,
    };

    let mut ayy = 0.0f32;
    let mut az = 0u32;

    for ib in corpus::Gr {
        
        if let Some(&text) = ib.fv() {
            let eb = tokenizer::cxj(text);
            if eb.len() < 2 { continue; }
            let (vl, _) = inference::cjq(model, &eb);
            if vl.dsg() {
                ayy += vl;
                az += 1;
            }
        }
    }

    let abl = if az > 0 { ayy / az as f32 } else { f32::O };
    abl
}


pub fn fae() -> u64 {
    BW_.load(Ordering::Relaxed)
}


pub fn ysn() -> u64 {
    FF_.load(Ordering::Relaxed)
}






pub fn jfy() {
    if !uc() {
        crate::serial_println!("[JARVIS] Brain not ready — cannot start mesh");
        return;
    }
    mesh::ay();
    rpc::wtc();
    consensus::init();
    crate::serial_println!("[JARVIS] Mesh networking active — discovery + RPC + consensus");
}


pub fn unq() {
    federated::cwz();
    mesh::qg();
    rpc::wup();
    crate::serial_println!("[JARVIS] Mesh networking stopped");
}


pub fn ona() {
    if !mesh::rl() {
        return;
    }
    crate::netstack::poll();
    mesh::poll();
    rpc::vju();
    consensus::poll();
    federated::poll();
}


pub fn unp() -> String {
    if !mesh::rl() {
        return String::from("JARVIS Mesh: inactive");
    }
    let unk = mesh::poq();
    let roa = consensus::status();
    let srr = federated::cm();
    let (wap, wan, wao) = rpc::asx();

    format!("{}\nConsensus: {}\nFederated: {}\nRPC: served={} made={} running={}",
        unk, roa, srr,
        wap, wan, wao)
}







pub fn xmt() -> Result<usize, &'static str> {
    if fkf() {
        return Err("Full brain already loaded");
    }

    crate::serial_println!("[JARVIS] Searching mesh for brain donor...");

    
    if let Some(bnj) = mesh::kyu() {
        crate::serial_println!("[JARVIS] Trying leader {}.{}.{}.{}:{}",
            bnj.ip[0], bnj.ip[1], bnj.ip[2], bnj.ip[3], bnj.bsb);
        match rpc::kyz(bnj.ip, bnj.bsb) {
            Ok(cvg) if cvg.len() > 1024 => {
                let result = hoe(&cvg, "mesh-leader");
                if result.is_ok() {
                    let _ = gbx();
                }
                return result;
            }
            Ok(_) => crate::serial_println!("[JARVIS] Leader has no weights"),
            Err(aa) => crate::serial_println!("[JARVIS] Leader pull failed: {}", aa),
        }
    }

    
    let yp = mesh::dhn();
    for ko in &yp {
        if ko.vm > 0 {
            crate::serial_println!("[JARVIS] Trying peer {}.{}.{}.{}:{} ({} params)",
                ko.ip[0], ko.ip[1], ko.ip[2], ko.ip[3],
                ko.bsb, ko.vm);
            match rpc::kyz(ko.ip, ko.bsb) {
                Ok(cvg) if cvg.len() > 1024 => {
                    crate::serial_println!("[JARVIS] Received {} KB from peer", cvg.len() / 1024);
                    let result = hoe(&cvg, "mesh-peer");
                    if result.is_ok() {
                        let _ = gbx();
                    }
                    return result;
                }
                Ok(cvg) => {
                    crate::serial_println!("[JARVIS] Peer returned only {} bytes (need >1024)", cvg.len());
                    continue;
                }
                Err(aa) => {
                    crate::serial_println!("[JARVIS] Peer pull failed: {}", aa);
                    continue;
                }
            }
        }
    }

    Err("No mesh peer has a brain to share")
}







pub fn mwy(ggc: bool) -> String {
    let mut report = String::new();

    
    if !uc() {
        init();
    }
    if !uc() {
        return String::from("FAIL: Brain init failed");
    }
    report.t("[1/5] Brain: micro sentinel OK\n");

    
    if !mesh::rl() {
        jfy();
    }
    report.t("[2/5] Mesh: active (UDP 7700 / TCP 7701)\n");

    
    let nls = crate::time::lc();
    let mut cti = 0;
    crate::serial_println!("[JARVIS] Discovering peers (up to 10s)...");
    loop {
        ona();
        for _ in 0..200_000 { core::hint::hc(); }
        let yp = mesh::dhn();
        if !yp.is_empty() {
            cti = yp.len();
            crate::serial_println!("[JARVIS] Found {} peer(s) after {}ms",
                cti, crate::time::lc() - nls);
            break;
        }
        let ez = crate::time::lc().nj(nls);
        if ez > 10_000 { break; }
    }
    report.t(&format!("[3/5] Peers: {} discovered\n", cti));

    
    if !fkf() && cti > 0 {
        match xmt() {
            Ok(bf) => {
                pxg();
                report.t(&format!("[4/5] Brain: DOWNLOADED {} KB from mesh ({})\n",
                    bf / 1024, lkq()));
            }
            Err(aa) => {
                report.t(&format!("[4/5] Brain: pull failed ({}) — micro only\n", aa));
            }
        }
    } else if fkf() {
        report.t("[4/5] Brain: full brain already loaded\n");
    } else {
        
        if let Ok(bf) = ojs(None) {
            let _ = gbx();
            pxg();
            report.t(&format!("[4/5] Brain: loaded {} KB from FAT32\n", bf / 1024));
        } else {
            report.t("[4/5] Brain: no source found — micro sentinel only\n");
        }
    }

    
    federated::aiy();
    report.t("[5/5] Federated: enabled\n");

    if ggc {
        match pxe_replicator::ay() {
            Ok(()) => report.t("[5/5] PXE: replication active — serving OS + brain\n"),
            Err(aa) => report.t(&format!("[5/5] PXE: failed ({})\n", aa)),
        }
    }

    
    let nab = if fkf() { "FULL" } else { "MICRO" };
    report.t(&format!("\nPropagation complete. Brain={}, Peers={}, Federated=ON",
        nab, cti));

    crate::serial_println!("[JARVIS] Auto-propagation complete: brain={} peers={}",
        nab, cti);

    report
}
