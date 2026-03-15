




























use alloc::vec::Vec;
use alloc::vec;
use super::model::*;
use super::inference;
use super::tokenizer;






const GN_: f32 = 0.001;


const BAB_: usize = 64;


const BCQ_: f32 = 0.01;


const TQ_: f32 = 1.0;






enum WeightTarget {
    Bkx(usize),
    Bkv(usize),
    Bkw(usize),
    Bku(usize),
    Dd,
}







pub fn pvw(model: &mut TransformerWeights, eb: &[u8], dsr: f32) -> f32 {
    let eb = if eb.len() > BAB_ { &eb[..BAB_] } else { eb };
    if eb.len() < 2 { return f32::O; }

    
    let (gas, _) = inference::cjq(model, eb);

    
    
    let gu = super::BW_.load(core::sync::atomic::Ordering::Relaxed);
    let ffm = (gu % 6) as usize;

    match ffm {
        0 => {
            
            xlq(model, eb, dsr);
        }
        1 => {
            let aup = (gu / 6) as usize % AZ_;
            iet(model, WeightTarget::Bkx(aup), eb, dsr);
        }
        2 => {
            let aup = (gu / 6) as usize % AZ_;
            iet(model, WeightTarget::Bkv(aup), eb, dsr);
        }
        3 => {
            let aup = (gu / 6) as usize % AZ_;
            iet(model, WeightTarget::Bkw(aup), eb, dsr);
        }
        4 => {
            let aup = (gu / 6) as usize % AZ_;
            iet(model, WeightTarget::Bku(aup), eb, dsr);
        }
        5 => {
            iet(model, WeightTarget::Dd, eb, dsr);
        }
        _ => {}
    }

    gas
}


fn xlq(model: &mut TransformerWeights, eb: &[u8], aad: f32) {
    let (gas, _) = inference::cjq(model, eb);

    
    let mut phm = [false; BG_];
    for &ab in eb {
        if phm[ab as usize] { continue; }
        phm[ab as usize] = true;

        let ar = ab as usize * E_;
        
        for bc in 0..E_ {
            if !wnd(ar + bc) { continue; }

            
            model.bpa[ar + bc] += GN_;
            let (ljx, _) = inference::cjq(model, eb);

            
            model.bpa[ar + bc] -= 2.0 * GN_;
            let (ljw, _) = inference::cjq(model, eb);

            
            model.bpa[ar + bc] += GN_; 

            let buo = (ljx - ljw) / (2.0 * GN_);
            let buo = ndm(buo);
            model.bpa[ar + bc] -= aad * buo;
        }
    }
}








fn iet(model: &mut TransformerWeights, cd: WeightTarget, eb: &[u8], aad: f32) {
    
    
    let (ptr, bo) = match cd {
        WeightTarget::Bkx(dm)    => (model.my[dm].biw.mw(), model.my[dm].biw.len()),
        WeightTarget::Bkv(dm)    => (model.my[dm].biu.mw(), model.my[dm].biu.len()),
        WeightTarget::Bkw(dm)    => (model.my[dm].biv.mw(), model.my[dm].biv.len()),
        WeightTarget::Bku(dm) => (model.my[dm].bit.mw(), model.my[dm].bit.len()),
        WeightTarget::Dd        => (model.bft.mw(), model.bft.len()),
    };

    let wcl = ((bo as f32 * BCQ_) as usize).am(1);
    let gu = super::BW_.load(core::sync::atomic::Ordering::Relaxed) as usize;

    for a in 0..wcl {
        let w = (gu * 7919 + a * 6271) % bo; 

        
        
        
        unsafe {
            
            *ptr.add(w) += GN_;
        }
        let (ljx, _) = inference::cjq(model, eb);

        unsafe {
            
            *ptr.add(w) -= 2.0 * GN_;
        }
        let (ljw, _) = inference::cjq(model, eb);

        unsafe {
            
            *ptr.add(w) += GN_;
        }

        
        let buo = (ljx - ljw) / (2.0 * GN_);
        let buo = ndm(buo);
        unsafe {
            *ptr.add(w) -= aad * buo;
        }
    }
}








pub fn xlr(model: &mut TransformerWeights, eb: &[u8], aad: f32) -> f32 {
    if eb.len() < 2 { return f32::O; }

    let (gas, _) = inference::cjq(model, eb);

    
    let mut rng = crate::time::ave().hx(6364136223846793005);

    
    let ltp: Vec<f32> = (0..model.bft.len()).map(|_| {
        rng ^= rng << 13; rng ^= rng >> 7; rng ^= rng << 17;
        let fs = (rng >> 40) as u32;
        (fs as f32 / (1u32 << 24) as f32) * 2.0 - 1.0
    }).collect();

    
    for (d, &ai) in model.bft.el().fca(ltp.iter()) {
        *d += aad * ai;
    }

    
    let (uta, _) = inference::cjq(model, eb);

    if uta >= gas {
        
        for (d, &ai) in model.bft.el().fca(ltp.iter()) {
            *d -= 2.0 * aad * ai; 
        }
        let (vyn, _) = inference::cjq(model, eb);
        if vyn >= gas {
            
            for (d, &ai) in model.bft.el().fca(ltp.iter()) {
                *d += aad * ai;
            }
        }
    }

    gas
}






pub fn eyj() -> (u32, u32) {
    let mut afu = 0u32;
    let mut ace = 0u32;

    
    crate::serial_println!("[JARVIS-TRAIN] Test 1: Loss computation");
    {
        let model = TransformerWeights::dtm();
        let eb = tokenizer::cxj("Hello world");
        let (vl, auq) = inference::cjq(&model, &eb);
        if vl.dsg() && auq.len() > 0 {
            crate::serial_println!("[JARVIS-TRAIN]   Loss = {:.4} (random weights)", vl);
            afu += 1;
        } else {
            crate::serial_println!("[JARVIS-TRAIN]   FAIL: loss={}", vl);
            ace += 1;
        }
    }

    
    crate::serial_println!("[JARVIS-TRAIN] Test 2: Training step");
    {
        let mut model = TransformerWeights::dtm();
        let eb = tokenizer::cxj("AB");
        let vl = pvw(&mut model, &eb, 0.01);
        if vl.dsg() {
            crate::serial_println!("[JARVIS-TRAIN]   Initial loss = {:.4}", vl);
            afu += 1;
        } else {
            ace += 1;
        }
    }

    
    crate::serial_println!("[JARVIS-TRAIN] Test 3: Random perturbation training");
    {
        let mut model = TransformerWeights::dtm();
        let eb = tokenizer::cxj("Test");
        let vl = xlr(&mut model, &eb, 0.001);
        if vl.dsg() {
            crate::serial_println!("[JARVIS-TRAIN]   Loss = {:.4}", vl);
            afu += 1;
        } else {
            ace += 1;
        }
    }

    (afu, ace)
}






fn wnd(w: usize) -> bool {
    
    let i = w.hx(2654435761); 
    (i % 100) < (BCQ_ * 100.0) as usize
}


fn ndm(at: f32) -> f32 {
    if at > TQ_ { TQ_ }
    else if at < -TQ_ { -TQ_ }
    else { at }
}
