














































use alloc::string::String;
use alloc::format;






static VY_: spin::Mutex<f32> = spin::Mutex::new(0.001);


static AAO_: spin::Mutex<(f32, u32)> = spin::Mutex::new((0.0, 0));


static AAP_: core::sync::atomic::AtomicBool =
    core::sync::atomic::AtomicBool::new(false);






pub fn nwa() {
    
    let mut buf = [0u8; 512];
    let mut pos = 0;

    
    while pos < buf.len() - 1 {
        if let Some(b) = crate::serial::read_byte() {
            if b == b'\n' || b == b'\r' {
                if pos > 0 { break; }
                continue;
            }
            buf[pos] = b;
            pos += 1;
        } else {
            break;
        }
    }

    if pos == 0 { return; }

    
    let line = match core::str::from_utf8(&buf[..pos]) {
        Ok(j) => j,
        Err(_) => return,
    };

    
    if line.starts_with("MENTOR:") {
        nyg(&line[7..]);
    }
}


fn nyg(cmd: &str) {
    
    if cmd.starts_with("GUARDIAN:AUTH:") {
        let abm = &cmd[13..];
        if super::guardian::jyg(abm) {
            wm("OK:Copilot guardian authenticated");
        } else {
            wm("ERROR:Authentication failed");
        }
        return;
    }
    if cmd == "GUARDIAN:LOCK" {
        super::guardian::ggd();
        wm("OK:Guardian session locked");
        return;
    }
    if cmd == "GUARDIAN:STATUS" {
        let lines = super::guardian::hsq();
        for l in &lines { wm(l); }
        return;
    }
    if cmd == "GUARDIAN:PACT" {
        wm(super::guardian::BJE_);
        return;
    }

    if cmd.starts_with("TEACH:") {
        let text = &cmd[6..];
        mis(text);
    } else if cmd.starts_with("CORRECT:") {
        let text = &cmd[8..];
        mhm(text);
    } else if cmd.starts_with("EVAL:") {
        let nh = &cmd[5..];
        mhs(nh);
    } else if cmd.starts_with("GENERATE:") {
        let nh = &cmd[9..];
        mhu(nh);
    } else if cmd.starts_with("CONFIG:") {
        let config = &cmd[7..];
        mhk(config);
    } else if cmd == "STATUS" {
        mil();
    } else if cmd == "SAVE" {
        mii();
    } else if cmd == "LOAD" {
        mia();
    } else if cmd == "RESET" {
        mig();
    } else if cmd == "BATCH_START" {
        AAP_.store(true, core::sync::atomic::Ordering::Relaxed);
        *AAO_.lock() = (0.0, 0);
        wm("OK:Batch mode started");
    } else if cmd == "BATCH_END" {
        AAP_.store(false, core::sync::atomic::Ordering::Relaxed);
        let (aah, count) = *AAO_.lock();
        let ns = if count > 0 { aah / count as f32 } else { 0.0 };
        wm(&format!("OK:Batch ended. {} sequences, avg loss={:.4}", count, ns));
    } else {
        wm(&format!("ERROR:Unknown command '{}'", cmd));
    }
}






fn mis(text: &str) {
    let lr = *VY_.lock();
    let ka = super::bwo(text, lr);

    if AAP_.load(core::sync::atomic::Ordering::Relaxed) {
        let mut bl = AAO_.lock();
        bl.0 += ka;
        bl.1 += 1;
    }

    wm(&format!("LOSS:{:.4}", ka));
}


fn mhm(text: &str) {
    if let Some(sep_pos) = text.find('|') {
        let pws = &text[..sep_pos];
        let mfi = &text[sep_pos + 1..];
        
        let lr = *VY_.lock() * 2.0;
        let ka = super::bwo(mfi, lr);
        wm(&format!("OK:Correction trained, loss={:.4}", ka));
    } else {
        wm("ERROR:Expected format: wrong|correct");
    }
}


fn mhs(nh: &str) {
    if !super::is_ready() {
        wm("ERROR:Model not ready");
        return;
    }

    let tokens = super::tokenizer::bbj(nh);
    let aea = super::Ay.lock();
    if let Some(model) = aea.as_ref() {
        let (ka, _) = super::inference::atj(model, &tokens);
        wm(&format!("LOSS:{:.4}", ka));
    } else {
        wm("ERROR:No model loaded");
    }
}


fn mhu(nh: &str) {
    let output = super::generate(nh, 128);
    wm(&format!("GEN:{}", output));
}


fn mhk(config: &str) {
    if let Err(bk) = super::guardian::bxo(super::guardian::ProtectedOp::ConfigChange) {
        wm(&alloc::format!("ERROR:Guardian denied — {}", bk));
        return;
    }
    if config.starts_with("temp=") {
        if let Ok(t) = config[5..].parse::<f32>() {
            
            wm(&format!("OK:Temperature set to {}", t));
        }
    } else if config.starts_with("topk=") {
        if let Ok(k) = config[5..].parse::<usize>() {
            wm(&format!("OK:Top-k set to {}", k));
        }
    } else if config.starts_with("lr=") {
        if let Ok(lr) = config[3..].parse::<f32>() {
            *VY_.lock() = lr;
            wm(&format!("OK:Learning rate set to {}", lr));
        }
    } else {
        wm(&format!("ERROR:Unknown config '{}'", config));
    }
}


fn mil() {
    let stats = super::stats();
    wm(&format!("STATUS:{}", stats));
}


fn mii() {
    match super::jco() {
        Ok(bytes) => wm(&format!("OK:Saved {} KB to /jarvis/weights.bin", bytes / 1024)),
        Err(e) => wm(&format!("ERROR:{}", e)),
    }
}


fn mia() {
    if let Err(bk) = super::guardian::bxo(super::guardian::ProtectedOp::WeightLoad) {
        wm(&alloc::format!("ERROR:Guardian denied — {}", bk));
        return;
    }
    match super::iky() {
        Ok(bytes) => wm(&format!("OK:Loaded {} KB from /jarvis/weights.bin", bytes / 1024)),
        Err(e) => wm(&format!("ERROR:{}", e)),
    }
}


fn mig() {
    if let Err(bk) = super::guardian::bxo(super::guardian::ProtectedOp::ModelReset) {
        wm(&alloc::format!("ERROR:Guardian denied — {}", bk));
        return;
    }
    if let Some(model) = super::Ay.lock().as_mut() {
        model.reset();
        wm("OK:Weights reset to random initialization");
    } else {
        wm("ERROR:No model to reset");
    }
}






fn wm(bk: &str) {
    crate::serial_println!("JARVIS:{}", bk);
}


pub fn bnh() -> f32 {
    *VY_.lock()
}
