





















































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





static Ah: AtomicBool = AtomicBool::new(false);
static FU_: AtomicU64 = AtomicU64::new(0);
static BY_: AtomicU64 = AtomicU64::new(0);


static Ay: Mutex<Option<TransformerWeights>> = Mutex::new(None);


static Lr: Mutex<Option<InferenceEngine>> = Mutex::new(None);


static Mt: Mutex<Option<AdamState>> = Mutex::new(None);


static Tm: AtomicU8 = AtomicU8::new(0);


static COD_: AtomicBool = AtomicBool::new(false);


static JD_: Mutex<Option<MicroWeights>> = Mutex::new(None);


static WO_: Mutex<Option<MicroEngine>> = Mutex::new(None);


static FT_: AtomicBool = AtomicBool::new(false);









pub fn init() {
    crate::serial_println!("[JARVIS] Two-tier brain init starting...");

    
    static PD_: &[u8] = include_bytes!("jarvis_micro.bin");
    if PD_.len() >= 4 && PD_.len() % 4 == 0 {
        let dpy = PD_.len() / 4;
        let xn: &[f32] = unsafe {
            core::slice::from_raw_parts(PD_.as_ptr() as *const f32, dpy)
        };
        if let Some(micro_weights) = MicroWeights::byt(xn) {
            crate::serial_println!("[JARVIS] Micro sentinel loaded ({} params, {} KB)",
                micro_weights.param_count(), PD_.len() / 1024);
            *JD_.lock() = Some(micro_weights);
            *WO_.lock() = Some(MicroEngine::new());
        } else {
            crate::serial_println!("[JARVIS] WARN: micro sentinel deserialize failed, using random");
            *JD_.lock() = Some(MicroWeights::bns());
            *WO_.lock() = Some(MicroEngine::new());
        }
    } else {
        crate::serial_println!("[JARVIS] WARN: micro binary invalid, using random");
        *JD_.lock() = Some(MicroWeights::bns());
        *WO_.lock() = Some(MicroEngine::new());
    }

    
    let status = micro_model::mvn();
    crate::serial_println!("[JARVIS] Kernel validation: heap={} int={} fs={} serial={}",
        status.heap_ok, status.interrupts_ok, status.fs_ok, status.serial_ok);

    
    simd::igp();
    let backend = compute::hrv();
    match backend {
        compute::Backend::AmdGpu => crate::serial_println!("[JARVIS] Compute: AMD GPU detected"),
        compute::Backend::CpuSimd => crate::serial_println!("[JARVIS] Compute: CPU SSE2 SIMD"),
    }

    
    Ah.store(true, Ordering::Release);

    
    if status.fs_ok {
        match gfx() {
            Ok(bytes) => {
                crate::serial_println!("[JARVIS] Full brain loaded from FS ({} KB)", bytes / 1024);
                
                let _ = crate::ramfs::bh(|fs| fs.rm(IA_));
                
                
                Tm.store(3, Ordering::Relaxed);
                crate::serial_println!("[JARVIS] Maturity: {} (level {})",
                    ggr(), ggq());
            }
            Err(e) => {
                crate::serial_println!("[JARVIS] Full brain not available: {} — micro sentinel active", e);
            }
        }
    } else {
        crate::serial_println!("[JARVIS] FS not ready — micro sentinel only mode");
    }

    crate::serial_println!("[JARVIS] Init complete. Micro={}, Full={}",
        if JD_.lock().is_some() { "OK" } else { "FAIL" },
        if FT_.load(Ordering::Relaxed) { "LOADED" } else { "NOT LOADED" });

    
    if cki() && !mesh::is_active() {
        euj();
        crate::serial_println!("[JARVIS] Mesh auto-started (full brain available for peers)");
    }
}


pub fn gfx() -> Result<usize, &'static str> {
    let data = crate::ramfs::bh(|fs| {
        fs.read_file(IA_).map(|d| d.to_vec())
    }).map_err(|_| "weights.bin not found in FS")?;

    if data.len() % 4 != 0 || data.len() < 1024 {
        return Err("Invalid weight file");
    }

    let dpy = data.len() / 4;
    let xn: &[f32] = unsafe {
        core::slice::from_raw_parts(data.as_ptr() as *const f32, dpy)
    };

    let afx = model::TransformerWeights::byt(xn)
        .ok_or("Deserialization failed (wrong param count)")?;

    let param_count = afx.param_count();

    
    if compute::cyv() {
        let _ = compute::hat(&afx);
    }

    let engine = InferenceEngine::new();
    let app = AdamState::hck(param_count, 0.001);

    *Ay.lock() = Some(afx);
    *Lr.lock() = Some(engine);
    *Mt.lock() = Some(app);
    FT_.store(true, Ordering::Release);

    crate::serial_println!("[JARVIS] Full brain: {} params ({} KB)",
        param_count, data.len() / 1024);

    Ok(data.len())
}






fn dsc(data: &[u8], source: &str) -> Result<usize, &'static str> {
    if data.len() % 4 != 0 || data.len() < 1024 {
        return Err("Invalid weight data (bad size or alignment)");
    }

    let dpy = data.len() / 4;
    let xn: &[f32] = unsafe {
        core::slice::from_raw_parts(data.as_ptr() as *const f32, dpy)
    };

    let afx = model::TransformerWeights::byt(xn)
        .ok_or("Deserialization failed (wrong param count)")?;

    let param_count = afx.param_count();

    if compute::cyv() {
        let _ = compute::hat(&afx);
    }

    let engine = InferenceEngine::new();
    let app = AdamState::hck(param_count, 0.001);

    *Ay.lock() = Some(afx);
    *Lr.lock() = Some(engine);
    *Mt.lock() = Some(app);
    FT_.store(true, Ordering::Release);

    crate::serial_println!("[JARVIS] Full brain loaded from {}: {} params ({} KB)",
        source, param_count, data.len() / 1024);

    Ok(data.len())
}


pub fn mzu(path: &str) -> Result<usize, &'static str> {
    crate::serial_println!("[JARVIS] Loading brain from VFS: {}", path);
    let data = crate::vfs::read_file(path)
        .map_err(|_| "File not found or read error on VFS path")?;
    dsc(&data, path)
}


pub fn ikt(filename: Option<&str>) -> Result<usize, &'static str> {
    let path = match filename {
        Some(name) => format!("/mnt/fat32/{}", name),
        None => String::from("/mnt/fat32/jarvis_weights.bin"),
    };
    crate::serial_println!("[JARVIS] Loading brain from FAT32: {}", path);
    let data = crate::vfs::read_file(&path)
        .map_err(|_| "File not found on FAT32 (is a FAT32 disk attached?)")?;
    dsc(&data, &path)
}


pub fn mzt(url: &str) -> Result<usize, &'static str> {
    crate::serial_println!("[JARVIS] Downloading brain from: {}", url);
    let fa = crate::netstack::http::get(url)?;
    if fa.status_code != 200 {
        return Err("HTTP download failed (non-200 status)");
    }
    if fa.body.is_empty() {
        return Err("HTTP response body is empty");
    }
    crate::serial_println!("[JARVIS] Downloaded {} KB", fa.body.len() / 1024);
    dsc(&fa.body, url)
}


pub fn cuq() -> Result<usize, &'static str> {
    let aea = Ay.lock();
    let model = aea.as_ref().ok_or("No brain loaded to cache")?;
    let xn = model.serialize();
    let nb = xn.len() * 4;
    let bytes: &[u8] = unsafe {
        core::slice::from_raw_parts(xn.as_ptr() as *const u8, nb)
    };
    crate::ramfs::bh(|fs| {
        let _ = fs.mkdir(BLE_);
    });
    crate::ramfs::bh(|fs| {
        let _ = fs.touch(IA_);
        fs.write_file(IA_, bytes).map_err(|_| "Cache write failed")
    })?;
    crate::serial_println!("[JARVIS] Brain cached to RamFS ({} KB)", nb / 1024);
    Ok(nb)
}


pub fn cki() -> bool {
    FT_.load(Ordering::Acquire)
}


pub fn ggr() -> &'static str {
    match ggq() {
        0 => "Infant",
        1 => "Child",
        2 => "Teen",
        3 => "Adult",
        _ => "Unknown",
    }
}


fn qld() {
    crate::serial_println!("[JARVIS] Initializing random full brain...");

    let afx = TransformerWeights::bns();
    let param_count = afx.param_count();

    crate::serial_println!("[JARVIS] Model: {} layers, d_model={}, d_ff={}, {} heads",
        model::BB_, model::E_, model::Z_, model::GE_);
    crate::serial_println!("[JARVIS] Parameters: {} ({} KB FP32)",
        param_count, param_count * 4 / 1024);

    simd::igp();
    let backend = compute::hrv();
    match backend {
        compute::Backend::AmdGpu => {
            crate::serial_println!("[JARVIS] Compute: AMD GPU — GEMM available");
            match compute::hat(&afx) {
                Ok(bytes) => crate::serial_println!("[JARVIS] VRAM: {} KB", bytes / 1024),
                Err(e) => crate::serial_println!("[JARVIS] GPU fallback: {}", e),
            }
        }
        compute::Backend::CpuSimd => {
            crate::serial_println!("[JARVIS] Compute: CPU SSE2 SIMD");
        }
    }

    let engine = InferenceEngine::new();
    let app = AdamState::hck(param_count, 0.001);

    *Ay.lock() = Some(afx);
    *Lr.lock() = Some(engine);
    *Mt.lock() = Some(app);
    FT_.store(true, Ordering::Release);
    Ah.store(true, Ordering::Release);

    crate::serial_println!("[JARVIS] Random full brain ready.");
}


pub fn is_ready() -> bool {
    Ah.load(Ordering::Acquire)
}


pub fn ggq() -> u8 {
    Tm.load(Ordering::Relaxed)
}


pub fn iih() -> bool {
    COD_.load(Ordering::Relaxed)
}


fn jpf() {
    let ka = elq();
    let level = if ka < 2.0 { 3 }
        else if ka < 3.5 { 2 }
        else if ka < 5.0 { 1 }
        else { 0 };
    Tm.store(level, Ordering::Relaxed);
}







pub fn generate(nh: &str, alx: usize) -> String {
    if !is_ready() {
        return String::from("[JARVIS brain not initialized]");
    }

    
    if FT_.load(Ordering::Acquire) {
        let aea = Ay.lock();
        let model = match aea.as_ref() {
            Some(m) => m,
            None => return ghp(nh, alx),
        };

        let mut dor = Lr.lock();
        let engine = match dor.as_mut() {
            Some(e) => e,
            None => return ghp(nh, alx),
        };

        let tokens = tokenizer::bbj(nh);
        let fyj = engine.generate(model, &tokens, alx);
        FU_.fetch_add(1, Ordering::Relaxed);
        return tokenizer::dmo(&fyj);
    }

    
    ghp(nh, alx)
}


fn ghp(nh: &str, alx: usize) -> String {
    let nfg = JD_.lock();
    let dbh = match nfg.as_ref() {
        Some(m) => m,
        None => return String::from("[micro sentinel not loaded]"),
    };

    let mut dor = WO_.lock();
    let engine = match dor.as_mut() {
        Some(e) => e,
        None => return String::from("[micro engine not loaded]"),
    };

    let tokens: Vec<u8> = nh.bytes().collect();
    let fkv = alx.min(micro_model::DS_);
    let fyj = engine.generate(dbh, &tokens, fkv);
    FU_.fetch_add(1, Ordering::Relaxed);
    tokenizer::dmo(&fyj)
}


pub fn qqu(context: &[u8]) -> u8 {
    if !is_ready() { return b'?' }

    let aea = Ay.lock();
    let model = match aea.as_ref() {
        Some(m) => m,
        None => return b'?',
    };

    let mut dor = Lr.lock();
    let engine = match dor.as_mut() {
        Some(e) => e,
        None => return b'?',
    };

    engine.predict_next_token(model, context)
}








pub fn bwo(text: &str, bnh: f32) -> f32 {
    if !is_ready() { return f32::MAX; }

    
    if let Err(bk) = guardian::bxo(guardian::ProtectedOp::Train) {
        crate::serial_println!("[JARVIS] {}", bk);
        return f32::MAX;
    }

    let tokens = tokenizer::bbj(text);
    if tokens.len() < 2 { return f32::MAX; }

    
    let mut dvx = Mt.lock();
    if let Some(app) = dvx.as_mut() {
        let mut aea = Ay.lock();
        let model = match aea.as_mut() {
            Some(m) => m,
            None => return f32::MAX,
        };

        app.lr = bnh;
        let (ka, mut wg) = backprop::eng(model, &tokens);
        wg.clip_norm(1.0);
        app.step(model, &wg);
        BY_.fetch_add(1, Ordering::Relaxed);
        return ka;
    }
    drop(dvx);

    
    let mut aea = Ay.lock();
    let model = match aea.as_mut() {
        Some(m) => m,
        None => return f32::MAX,
    };
    let ka = training::jom(model, &tokens, bnh);
    BY_.fetch_add(1, Ordering::Relaxed);
    ka
}






pub fn info_lines() -> Vec<String> {
    let mut lines = Vec::new();
    lines.push(String::from("╔═══════════════════════════════════════════════════╗"));
    lines.push(String::from("║   J.A.R.V.I.S. Neural Brain v3.0 (Two-Tier)     ║"));
    lines.push(String::from("║   Micro Sentinel + Full Brain Architecture       ║"));
    lines.push(String::from("╠═══════════════════════════════════════════════════╣"));

    
    if let Some(dbh) = JD_.lock().as_ref() {
        lines.push(format!("║ Micro:  {}L × d{} × {}H  ({} params, {:.0} KB)     ║",
            micro_model::GC_, micro_model::N_,
            micro_model::AHG_, dbh.param_count(),
            dbh.param_count() as f64 * 4.0 / 1024.0));
        lines.push(String::from("║ Role:   Kernel sentinel, validation, fallback    ║"));
    } else {
        lines.push(String::from("║ Micro:  NOT LOADED                               ║"));
    }

    lines.push(String::from("╠═══════════════════════════════════════════════════╣"));

    
    let xo = FT_.load(Ordering::Relaxed);
    if xo {
        if let Some(model) = Ay.lock().as_ref() {
            let params = model.param_count();
            lines.push(format!("║ Full:   {}L × d{} × {}H × ff{}  ({:.1}M params)  ║",
                model::BB_, model::E_, model::GE_, model::Z_,
                params as f64 / 1_000_000.0));
            lines.push(format!("║ Memory: {:.1} MB FP32   Vocab: {} (byte)          ║",
                params as f64 * 4.0 / (1024.0 * 1024.0), model::BI_));
        }
        lines.push(format!("║ Status: LOADED from FS   Maturity: {} ({})    ║",
            ggq(), ggr()));
    } else {
        lines.push(String::from("║ Full:   NOT LOADED (use 'jarvis brain load')     ║"));
        lines.push(String::from("║ Status: MICRO ONLY MODE                          ║"));
    }

    lines.push(String::from("╠═══════════════════════════════════════════════════╣"));
    lines.push(format!("║ Generations:  {}                                  ║",
        FU_.load(Ordering::Relaxed)));
    lines.push(format!("║ Train steps:  {}                                  ║",
        BY_.load(Ordering::Relaxed)));
    let kwp = if compute::cyv() {
        "║ Compute: GPU (AMD RDNA GEMM)                      ║"
    } else {
        "║ Compute: CPU (SSE2 SIMD)                          ║"
    };
    lines.push(String::from(kwp));
    lines.push(String::from("╚═══════════════════════════════════════════════════╝"));

    lines
}


pub fn stats() -> String {
    let xo = if FT_.load(Ordering::Relaxed) {
        format!("full={}K", if let Some(m) = Ay.lock().as_ref() { m.param_count() / 1000 } else { 0 })
    } else {
        String::from("full=OFF")
    };
    let dbh = if JD_.lock().is_some() { "micro=OK" } else { "micro=OFF" };
    format!("Jarvis: {} {} gens={} steps={} ready={}",
        dbh, xo,
        FU_.load(Ordering::Relaxed),
        BY_.load(Ordering::Relaxed),
        is_ready())
}



pub fn qrg() -> bool {
    if !is_ready() { return false; }
    let audit = io_control::dqi();
    io_control::duz(&audit)
}


pub fn qrh() -> Vec<String> {
    let mut lines = Vec::new();

    lines.push(String::from("╔═══════════════════════════════════════════════════╗"));
    lines.push(String::from("║  JARVIS NETWORK PROPAGATION STATUS               ║"));
    lines.push(String::from("╠═══════════════════════════════════════════════════╣"));

    
    lines.push(format!("║ Brain:       {} ({} steps, {} gens){}",
        if is_ready() { "READY" } else { "NOT READY" },
        BY_.load(Ordering::Relaxed),
        FU_.load(Ordering::Relaxed),
        "         ║"));

    
    lines.push(format!("║ Federated:   {}{}",
        if federated::lq() { "ENABLED" } else { "DISABLED" },
        "                            ║"));
    lines.push(format!("║ Fed Rounds:  {}  Interval: {}ms{}",
        federated::oig(),
        federated::lal(),
        "              ║"));

    
    let bvq = federated::kgq();
    lines.push(format!("║ Bandwidth:   {} KB saved by compression{}",
        bvq / 1024, "          ║"));

    
    lines.push(format!("║ Mesh Peers:  {}  Role: {:?}{}",
        mesh::ayz(),
        mesh::dwa(),
        "                       ║"));

    
    let audit = io_control::dqi();
    let score = io_control::cvo(&audit);
    let caps = io_control::hjv(&audit);
    lines.push(format!("║ I/O Score:   {}%  Caps: 0x{:04X}{}",
        score, caps, "                   ║"));
    lines.push(format!("║ Net Ready:   {}  Full Ctrl: {}{}",
        if io_control::duz(&audit) { "YES" } else { "NO " },
        if io_control::iai(&audit) { "YES" } else { "NO " },
        "                  ║"));

    lines.push(String::from("╚═══════════════════════════════════════════════════╝"));
    lines
}


pub fn reset() {
    if let Some(model) = Ay.lock().as_mut() {
        model.reset();
        BY_.store(0, Ordering::Relaxed);
        FU_.store(0, Ordering::Relaxed);
    }
}





const BLE_: &str = "/jarvis";
const IA_: &str = "/jarvis/weights.bin";
const BLF_: &str = "/jarvis/meta.txt";


pub fn jco() -> Result<usize, &'static str> {
    let aea = Ay.lock();
    let model = aea.as_ref().ok_or("Model not loaded")?;

    let xn = model.serialize();
    let nb = xn.len() * 4;

    
    let bytes: &[u8] = unsafe {
        core::slice::from_raw_parts(xn.as_ptr() as *const u8, nb)
    };

    
    crate::ramfs::bh(|fs| {
        let _ = fs.mkdir(BLE_); 
    });

    
    crate::ramfs::bh(|fs| {
        let _ = fs.touch(IA_);
        fs.write_file(IA_, bytes).map_err(|_| "Write failed")
    })?;

    
    let meta = format!("params={}\nsteps={}\ngens={}\nbytes={}\n",
        model.param_count(),
        BY_.load(Ordering::Relaxed),
        FU_.load(Ordering::Relaxed),
        nb);
    crate::ramfs::bh(|fs| {
        let _ = fs.touch(BLF_);
        let _ = fs.write_file(BLF_, meta.as_bytes());
    });

    crate::serial_println!("[JARVIS] Saved {} floats ({} KB) to {}",
        xn.len(), nb / 1024, IA_);

    Ok(nb)
}


pub fn iky() -> Result<usize, &'static str> {
    gfx()
}


pub fn mkc() -> bool {
    crate::ramfs::bh(|fs| fs.exists(IA_))
}







pub fn nio(query: &str) -> Option<String> {
    if !is_ready() {
        init();
    }
    if !is_ready() { return None; }

    
    if !FT_.load(Ordering::Relaxed) && mkc() {
        let _ = gfx();
    }

    let output = generate(query, 64);
    if output.is_empty() { return None; }
    Some(output)
}


pub fn qng(user_input: &str, good_response: &str) {
    if !is_ready() { return; }
    
    let mut fdn = String::with_capacity(user_input.len() + good_response.len() + 1);
    fdn.push_str(user_input);
    fdn.push('\n');
    fdn.push_str(good_response);
    let _ = bwo(&fdn, 0.0005); 
}








pub fn ivx(ahx: usize, lr: f32) -> (usize, f32, u64) {
    if !is_ready() { init(); }
    if !is_ready() { return (0, f32::MAX, 0); }

    let start = crate::time::yf();
    let fdk = corpus::eci();
    let mut ix = fdk * ahx;
    let buc = lr * 0.1; 
    let mut step = 0u64;
    let mut aah = 0.0f32;
    let mut bhs = 0u32;

    
    const ALY_: usize = 4;

    
    let gbi = if crate::jarvis_hw::probe::mtq() {
        if let Some(ai) = crate::jarvis_hw::probe::cur() {
            let amt = hw_corpus::generate(&ai);
            crate::serial_println!("[JARVIS] Hardware corpus: {} sequences from live probe", amt.len());
            amt
        } else {
            alloc::vec::Vec::new()
        }
    } else {
        alloc::vec::Vec::new()
    };

    
    ix += gbi.len() * ahx;
    let cfb = (ix / 10).max(5) as u64; 

    crate::serial_println!("[JARVIS] Pre-training: {} phases + HW, {} sequences, {} epoch(s), lr_peak={}, warmup={}",
        corpus::dvp(), ix, ahx, lr, cfb);

    let mut dvx = Mt.lock();
    let mut aea = Ay.lock();

    if let (Some(app), Some(model)) = (dvx.as_mut(), aea.as_mut()) {
        for epoch in 0..ahx {
            for (phase_idx, phase) in corpus::Da.iter().enumerate() {
                let mut iun = 0.0f32;
                let mut ewp = 0u32;
                let mut ajd = backprop::ModelGrads::new();
                let mut avw = 0usize;
                let mut hds = 0.0f32;

                for &text in *phase {
                    let tokens = tokenizer::bbj(text);
                    if tokens.len() < 2 { step += 1; continue; }

                    
                    let dlz = optimizer::foo(
                        step, ix as u64, cfb, lr, buc
                    );
                    app.lr = dlz;

                    
                    let (ka, wg) = backprop::eng(model, &tokens);
                    if ka.is_finite() {
                        ajd.accumulate(&wg);
                        hds += ka;
                        avw += 1;
                        iun += ka;
                        ewp += 1;
                        aah += ka;
                        bhs += 1;
                    }

                    
                    if avw >= ALY_ {
                        ajd.scale(1.0 / avw as f32);
                        ajd.clip_norm(1.0);
                        app.step(model, &ajd);
                        BY_.fetch_add(1, Ordering::Relaxed);
                        ajd.zero();
                        avw = 0;
                        hds = 0.0;
                    }

                    step += 1;
                }

                
                if avw > 0 {
                    ajd.scale(1.0 / avw as f32);
                    ajd.clip_norm(1.0);
                    app.step(model, &ajd);
                    BY_.fetch_add(1, Ordering::Relaxed);
                    ajd.zero();
                }

                let ns = if ewp > 0 { iun / ewp as f32 } else { 0.0 };
                crate::serial_println!("[JARVIS] Epoch {}/{} Phase {} ({}) — {} seqs, avg loss={:.3}",
                    epoch + 1, ahx, phase_idx, corpus::ewq(phase_idx),
                    ewp, ns);
            }

            
            if !gbi.is_empty() {
                let mut ifh = 0.0f32;
                let mut epw = 0u32;
                let mut ajd = backprop::ModelGrads::new();
                let mut avw = 0usize;

                for text in &gbi {
                    let tokens = tokenizer::bbj(text);
                    if tokens.len() < 2 { step += 1; continue; }

                    let dlz = optimizer::foo(
                        step, ix as u64, cfb, lr, buc
                    );
                    app.lr = dlz;

                    let (ka, wg) = backprop::eng(model, &tokens);
                    if ka.is_finite() {
                        ajd.accumulate(&wg);
                        avw += 1;
                        ifh += ka;
                        epw += 1;
                        aah += ka;
                        bhs += 1;
                    }

                    if avw >= ALY_ {
                        ajd.scale(1.0 / avw as f32);
                        ajd.clip_norm(1.0);
                        app.step(model, &ajd);
                        BY_.fetch_add(1, Ordering::Relaxed);
                        ajd.zero();
                        avw = 0;
                    }

                    step += 1;
                }

                if avw > 0 {
                    ajd.scale(1.0 / avw as f32);
                    ajd.clip_norm(1.0);
                    app.step(model, &ajd);
                    BY_.fetch_add(1, Ordering::Relaxed);
                }

                let ns = if epw > 0 { ifh / epw as f32 } else { 0.0 };
                crate::serial_println!("[JARVIS] Epoch {}/{} Phase HW (Hardware Context) — {} seqs, avg loss={:.3}",
                    epoch + 1, ahx, epw, ns);
            }
        }
    } else {
        
        drop(dvx);
        drop(aea);
        for _epoch in 0..ahx {
            for phase in corpus::Da {
                for &text in *phase {
                    let ka = bwo(text, lr);
                    if ka.is_finite() { aah += ka; bhs += 1; }
                    step += 1;
                }
            }
        }
    }

    let bb = crate::time::yf().saturating_sub(start);
    let adh = if bhs > 0 { aah / bhs as f32 } else { f32::MAX };

    crate::serial_println!("[JARVIS] Pre-training done: {} steps, avg loss={:.3}, {} ms",
        step, adh, bb);

    (step as usize, adh, bb)
}



pub fn qqz(phase: usize, ahx: usize, lr: f32) -> (usize, f32, u64) {
    if !is_ready() { init(); }
    if !is_ready() { return (0, f32::MAX, 0); }
    if phase >= corpus::dvp() { return (0, f32::MAX, 0); }

    let start = crate::time::yf();
    let mut ix = 0usize;
    let mut aah = 0.0f32;
    let mut bhs = 0u32;

    let gub = corpus::Da[phase];
    let fdk = gub.len() * ahx;
    let ptv = (fdk / 10).max(2) as u64;
    let buc = lr * 0.1;
    let mut step = 0u64;

    crate::serial_println!("[JARVIS] Training phase {} ({}) — {} sequences, {} epoch(s)",
        phase, corpus::ewq(phase), gub.len(), ahx);

    for epoch in 0..ahx {
        for &text in gub {
            let dlz = optimizer::foo(step, fdk as u64, ptv, lr, buc);
            let ka = bwo(text, dlz);
            if ka.is_finite() {
                aah += ka;
                bhs += 1;
            }
            ix += 1;
            step += 1;
        }
        if ahx > 1 {
            let ns = if bhs > 0 { aah / bhs as f32 } else { 0.0 };
            crate::serial_println!("[JARVIS]   Epoch {}/{}: avg loss={:.3}", epoch + 1, ahx, ns);
        }
    }

    let bb = crate::time::yf().saturating_sub(start);
    let adh = if bhs > 0 { aah / bhs as f32 } else { f32::MAX };

    (ix, adh, bb)
}


pub fn fvj() -> f32 {
    if !is_ready() { return f32::MAX; }

    let aea = Ay.lock();
    let model = match aea.as_ref() {
        Some(m) => m,
        None => return f32::MAX,
    };

    let mut aah = 0.0f32;
    let mut count = 0u32;

    for phase in corpus::Da {
        for &text in *phase {
            let tokens = tokenizer::bbj(text);
            if tokens.len() < 2 { continue; }
            let (ka, _) = inference::atj(model, &tokens);
            if ka.is_finite() {
                aah += ka;
                count += 1;
            }
        }
    }

    let ns = if count > 0 { aah / count as f32 } else { f32::MAX };
    ns
}


pub fn elq() -> f32 {
    if !is_ready() { return f32::MAX; }

    let aea = Ay.lock();
    let model = match aea.as_ref() {
        Some(m) => m,
        None => return f32::MAX,
    };

    let mut aah = 0.0f32;
    let mut count = 0u32;

    for phase in corpus::Da {
        
        if let Some(&text) = phase.first() {
            let tokens = tokenizer::bbj(text);
            if tokens.len() < 2 { continue; }
            let (ka, _) = inference::atj(model, &tokens);
            if ka.is_finite() {
                aah += ka;
                count += 1;
            }
        }
    }

    let ns = if count > 0 { aah / count as f32 } else { f32::MAX };
    ns
}


pub fn training_steps() -> u64 {
    BY_.load(Ordering::Relaxed)
}


pub fn qhc() -> u64 {
    FU_.load(Ordering::Relaxed)
}






pub fn euj() {
    if !is_ready() {
        crate::serial_println!("[JARVIS] Brain not ready — cannot start mesh");
        return;
    }
    mesh::start();
    rpc::owj();
    consensus::init();
    crate::serial_println!("[JARVIS] Mesh networking active — discovery + RPC + consensus");
}


pub fn nez() {
    federated::bbc();
    mesh::stop();
    rpc::oxo();
    crate::serial_println!("[JARVIS] Mesh networking stopped");
}


pub fn inl() {
    if !mesh::is_active() {
        return;
    }
    crate::netstack::poll();
    mesh::poll();
    rpc::nwb();
    consensus::poll();
    federated::poll();
}


pub fn ney() -> String {
    if !mesh::is_active() {
        return String::from("JARVIS Mesh: inactive");
    }
    let neq = mesh::jis();
    let kxe = consensus::status();
    let luv = federated::stats();
    let (rpc_served, rpc_made, rpc_running) = rpc::get_stats();

    format!("{}\nConsensus: {}\nFederated: {}\nRPC: served={} made={} running={}",
        neq, kxe, luv,
        rpc_served, rpc_made, rpc_running)
}







pub fn pnz() -> Result<usize, &'static str> {
    if cki() {
        return Err("Full brain already loaded");
    }

    crate::serial_println!("[JARVIS] Searching mesh for brain donor...");

    
    if let Some(aid) = mesh::fyt() {
        crate::serial_println!("[JARVIS] Trying leader {}.{}.{}.{}:{}",
            aid.ip[0], aid.ip[1], aid.ip[2], aid.ip[3], aid.rpc_port);
        match rpc::fyy(aid.ip, aid.rpc_port) {
            Ok(baf) if baf.len() > 1024 => {
                let result = dsc(&baf, "mesh-leader");
                if result.is_ok() {
                    let _ = cuq();
                }
                return result;
            }
            Ok(_) => crate::serial_println!("[JARVIS] Leader has no weights"),
            Err(e) => crate::serial_println!("[JARVIS] Leader pull failed: {}", e),
        }
    }

    
    let lj = mesh::bgo();
    for peer in &lj {
        if peer.param_count > 0 {
            crate::serial_println!("[JARVIS] Trying peer {}.{}.{}.{}:{} ({} params)",
                peer.ip[0], peer.ip[1], peer.ip[2], peer.ip[3],
                peer.rpc_port, peer.param_count);
            match rpc::fyy(peer.ip, peer.rpc_port) {
                Ok(baf) if baf.len() > 1024 => {
                    crate::serial_println!("[JARVIS] Received {} KB from peer", baf.len() / 1024);
                    let result = dsc(&baf, "mesh-peer");
                    if result.is_ok() {
                        let _ = cuq();
                    }
                    return result;
                }
                Ok(baf) => {
                    crate::serial_println!("[JARVIS] Peer returned only {} bytes (need >1024)", baf.len());
                    continue;
                }
                Err(e) => {
                    crate::serial_println!("[JARVIS] Peer pull failed: {}", e);
                    continue;
                }
            }
        }
    }

    Err("No mesh peer has a brain to share")
}







pub fn hgd(cxd: bool) -> String {
    let mut report = String::new();

    
    if !is_ready() {
        init();
    }
    if !is_ready() {
        return String::from("FAIL: Brain init failed");
    }
    report.push_str("[1/5] Brain: micro sentinel OK\n");

    
    if !mesh::is_active() {
        euj();
    }
    report.push_str("[2/5] Mesh: active (UDP 7700 / TCP 7701)\n");

    
    let hsl = crate::time::uptime_ms();
    let mut ayz = 0;
    crate::serial_println!("[JARVIS] Discovering peers (up to 10s)...");
    loop {
        inl();
        for _ in 0..200_000 { core::hint::spin_loop(); }
        let lj = mesh::bgo();
        if !lj.is_empty() {
            ayz = lj.len();
            crate::serial_println!("[JARVIS] Found {} peer(s) after {}ms",
                ayz, crate::time::uptime_ms() - hsl);
            break;
        }
        let bb = crate::time::uptime_ms().wrapping_sub(hsl);
        if bb > 10_000 { break; }
    }
    report.push_str(&format!("[3/5] Peers: {} discovered\n", ayz));

    
    if !cki() && ayz > 0 {
        match pnz() {
            Ok(bytes) => {
                jpf();
                report.push_str(&format!("[4/5] Brain: DOWNLOADED {} KB from mesh ({})\n",
                    bytes / 1024, ggr()));
            }
            Err(e) => {
                report.push_str(&format!("[4/5] Brain: pull failed ({}) — micro only\n", e));
            }
        }
    } else if cki() {
        report.push_str("[4/5] Brain: full brain already loaded\n");
    } else {
        
        if let Ok(bytes) = ikt(None) {
            let _ = cuq();
            jpf();
            report.push_str(&format!("[4/5] Brain: loaded {} KB from FAT32\n", bytes / 1024));
        } else {
            report.push_str("[4/5] Brain: no source found — micro sentinel only\n");
        }
    }

    
    federated::enable();
    report.push_str("[5/5] Federated: enabled\n");

    if cxd {
        match pxe_replicator::start() {
            Ok(()) => report.push_str("[5/5] PXE: replication active — serving OS + brain\n"),
            Err(e) => report.push_str(&format!("[5/5] PXE: failed ({})\n", e)),
        }
    }

    
    let hip = if cki() { "FULL" } else { "MICRO" };
    report.push_str(&format!("\nPropagation complete. Brain={}, Peers={}, Federated=ON",
        hip, ayz));

    crate::serial_println!("[JARVIS] Auto-propagation complete: brain={} peers={}",
        hip, ayz);

    report
}
