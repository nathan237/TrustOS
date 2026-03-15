














































use alloc::string::String;
use alloc::format;






static UP_: spin::Mutex<f32> = spin::Mutex::new(0.001);


static ZH_: spin::Mutex<(f32, u32)> = spin::Mutex::new((0.0, 0));


static ZI_: core::sync::atomic::AtomicBool =
    core::sync::atomic::AtomicBool::new(false);






pub fn vjt() {
    
    let mut k = [0u8; 512];
    let mut u = 0;

    
    while u < k.len() - 1 {
        if let Some(o) = crate::serial::dlb() {
            if o == b'\n' || o == b'\r' {
                if u > 0 { break; }
                continue;
            }
            k[u] = o;
            u += 1;
        } else {
            break;
        }
    }

    if u == 0 { return; }

    
    let line = match core::str::jg(&k[..u]) {
        Ok(e) => e,
        Err(_) => return,
    };

    
    if line.cj("MENTOR:") {
        vmm(&line[7..]);
    }
}


fn vmm(cmd: &str) {
    
    if cmd.cj("GUARDIAN:AUTH:") {
        let bat = &cmd[13..];
        if super::guardian::qlg(bat) {
            aru("OK:Copilot guardian authenticated");
        } else {
            aru("ERROR:Authentication failed");
        }
        return;
    }
    if cmd == "GUARDIAN:LOCK" {
        super::guardian::ljp();
        aru("OK:Guardian session locked");
        return;
    }
    if cmd == "GUARDIAN:STATUS" {
        let ak = super::guardian::nly();
        for dm in &ak { aru(dm); }
        return;
    }
    if cmd == "GUARDIAN:PACT" {
        aru(super::guardian::BHA_);
        return;
    }

    if cmd.cj("TEACH:") {
        let text = &cmd[6..];
        tli(text);
    } else if cmd.cj("CORRECT:") {
        let text = &cmd[8..];
        tje(text);
    } else if cmd.cj("EVAL:") {
        let aau = &cmd[5..];
        tjn(aau);
    } else if cmd.cj("GENERATE:") {
        let aau = &cmd[9..];
        tjs(aau);
    } else if cmd.cj("CONFIG:") {
        let config = &cmd[7..];
        tjc(config);
    } else if cmd == "STATUS" {
        tkz();
    } else if cmd == "SAVE" {
        tku();
    } else if cmd == "LOAD" {
        tkh();
    } else if cmd == "RESET" {
        tks();
    } else if cmd == "BATCH_START" {
        ZI_.store(true, core::sync::atomic::Ordering::Relaxed);
        *ZH_.lock() = (0.0, 0);
        aru("OK:Batch mode started");
    } else if cmd == "BATCH_END" {
        ZI_.store(false, core::sync::atomic::Ordering::Relaxed);
        let (ayy, az) = *ZH_.lock();
        let abl = if az > 0 { ayy / az as f32 } else { 0.0 };
        aru(&format!("OK:Batch ended. {} sequences, avg loss={:.4}", az, abl));
    } else {
        aru(&format!("ERROR:Unknown command '{}'", cmd));
    }
}






fn tli(text: &str) {
    let aad = *UP_.lock();
    let vl = super::ekd(text, aad);

    if ZI_.load(core::sync::atomic::Ordering::Relaxed) {
        let mut bl = ZH_.lock();
        bl.0 += vl;
        bl.1 += 1;
    }

    aru(&format!("LOSS:{:.4}", vl));
}


fn tje(text: &str) {
    if let Some(phz) = text.du('|') {
        let xxy = &text[..phz];
        let tgq = &text[phz + 1..];
        
        let aad = *UP_.lock() * 2.0;
        let vl = super::ekd(tgq, aad);
        aru(&format!("OK:Correction trained, loss={:.4}", vl));
    } else {
        aru("ERROR:Expected format: wrong|correct");
    }
}


fn tjn(aau: &str) {
    if !super::uc() {
        aru("ERROR:Model not ready");
        return;
    }

    let eb = super::tokenizer::cxj(aau);
    let bet = super::Ci.lock();
    if let Some(model) = bet.as_ref() {
        let (vl, _) = super::inference::cjq(model, &eb);
        aru(&format!("LOSS:{:.4}", vl));
    } else {
        aru("ERROR:No model loaded");
    }
}


fn tjs(aau: &str) {
    let an = super::cks(aau, 128);
    aru(&format!("GEN:{}", an));
}


fn tjc(config: &str) {
    if let Err(fr) = super::guardian::emj(super::guardian::ProtectedOp::Pc) {
        aru(&alloc::format!("ERROR:Guardian denied — {}", fr));
        return;
    }
    if config.cj("temp=") {
        if let Ok(ab) = config[5..].parse::<f32>() {
            
            aru(&format!("OK:Temperature set to {}", ab));
        }
    } else if config.cj("topk=") {
        if let Ok(eh) = config[5..].parse::<usize>() {
            aru(&format!("OK:Top-k set to {}", eh));
        }
    } else if config.cj("lr=") {
        if let Ok(aad) = config[3..].parse::<f32>() {
            *UP_.lock() = aad;
            aru(&format!("OK:Learning rate set to {}", aad));
        }
    } else {
        aru(&format!("ERROR:Unknown config '{}'", config));
    }
}


fn tkz() {
    let cm = super::cm();
    aru(&format!("STATUS:{}", cm));
}


fn tku() {
    match super::pfn() {
        Ok(bf) => aru(&format!("OK:Saved {} KB to /jarvis/weights.bin", bf / 1024)),
        Err(aa) => aru(&format!("ERROR:{}", aa)),
    }
}


fn tkh() {
    if let Err(fr) = super::guardian::emj(super::guardian::ProtectedOp::Bwu) {
        aru(&alloc::format!("ERROR:Guardian denied — {}", fr));
        return;
    }
    match super::oka() {
        Ok(bf) => aru(&format!("OK:Loaded {} KB from /jarvis/weights.bin", bf / 1024)),
        Err(aa) => aru(&format!("ERROR:{}", aa)),
    }
}


fn tks() {
    if let Err(fr) = super::guardian::emj(super::guardian::ProtectedOp::Bmm) {
        aru(&alloc::format!("ERROR:Guardian denied — {}", fr));
        return;
    }
    if let Some(model) = super::Ci.lock().as_mut() {
        model.apa();
        aru("OK:Weights reset to random initialization");
    } else {
        aru("ERROR:No model to reset");
    }
}






fn aru(fr: &str) {
    crate::serial_println!("JARVIS:{}", fr);
}


pub fn dsr() -> f32 {
    *UP_.lock()
}
