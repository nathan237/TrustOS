




























use alloc::vec::Vec;
use alloc::vec;
use super::model::*;
use super::inference;
use super::tokenizer;






const HE_: f32 = 0.001;


const BCD_: usize = 64;


const BET_: f32 = 0.01;


const UW_: f32 = 1.0;






enum WeightTarget {
    LayerWq(usize),
    LayerWk(usize),
    LayerWo(usize),
    LayerWgate(usize),
    Output,
}







pub fn jom(model: &mut TransformerWeights, tokens: &[u8], bnh: f32) -> f32 {
    let tokens = if tokens.len() > BCD_ { &tokens[..BCD_] } else { tokens };
    if tokens.len() < 2 { return f32::MAX; }

    
    let (base_loss, _) = inference::atj(model, tokens);

    
    
    let step = super::BY_.load(core::sync::atomic::Ordering::Relaxed);
    let chn = (step % 6) as usize;

    match chn {
        0 => {
            
            pna(model, tokens, bnh);
        }
        1 => {
            let xv = (step / 6) as usize % BB_;
            eco(model, WeightTarget::LayerWq(xv), tokens, bnh);
        }
        2 => {
            let xv = (step / 6) as usize % BB_;
            eco(model, WeightTarget::LayerWk(xv), tokens, bnh);
        }
        3 => {
            let xv = (step / 6) as usize % BB_;
            eco(model, WeightTarget::LayerWo(xv), tokens, bnh);
        }
        4 => {
            let xv = (step / 6) as usize % BB_;
            eco(model, WeightTarget::LayerWgate(xv), tokens, bnh);
        }
        5 => {
            eco(model, WeightTarget::Output, tokens, bnh);
        }
        _ => {}
    }

    base_loss
}


fn pna(model: &mut TransformerWeights, tokens: &[u8], lr: f32) {
    let (base_loss, _) = inference::atj(model, tokens);

    
    let mut jef = [false; BI_];
    for &t in tokens {
        if jef[t as usize] { continue; }
        jef[t as usize] = true;

        let base = t as usize * E_;
        
        for d in 0..E_ {
            if !orx(base + d) { continue; }

            
            model.token_embed[base + d] += HE_;
            let (loss_plus, _) = inference::atj(model, tokens);

            
            model.token_embed[base + d] -= 2.0 * HE_;
            let (loss_minus, _) = inference::atj(model, tokens);

            
            model.token_embed[base + d] += HE_; 

            let alp = (loss_plus - loss_minus) / (2.0 * HE_);
            let alp = hlm(alp);
            model.token_embed[base + d] -= lr * alp;
        }
    }
}








fn eco(model: &mut TransformerWeights, target: WeightTarget, tokens: &[u8], lr: f32) {
    
    
    let (ptr, ae) = match target {
        WeightTarget::LayerWq(l)    => (model.layers[l].w_q.as_mut_ptr(), model.layers[l].w_q.len()),
        WeightTarget::LayerWk(l)    => (model.layers[l].w_k.as_mut_ptr(), model.layers[l].w_k.len()),
        WeightTarget::LayerWo(l)    => (model.layers[l].w_o.as_mut_ptr(), model.layers[l].w_o.len()),
        WeightTarget::LayerWgate(l) => (model.layers[l].w_gate.as_mut_ptr(), model.layers[l].w_gate.len()),
        WeightTarget::Output        => (model.w_output.as_mut_ptr(), model.w_output.len()),
    };

    let ojy = ((ae as f32 * BET_) as usize).max(1);
    let step = super::BY_.load(core::sync::atomic::Ordering::Relaxed) as usize;

    for i in 0..ojy {
        let idx = (step * 7919 + i * 6271) % ae; 

        
        
        
        unsafe {
            
            *ptr.add(idx) += HE_;
        }
        let (loss_plus, _) = inference::atj(model, tokens);

        unsafe {
            
            *ptr.add(idx) -= 2.0 * HE_;
        }
        let (loss_minus, _) = inference::atj(model, tokens);

        unsafe {
            
            *ptr.add(idx) += HE_;
        }

        
        let alp = (loss_plus - loss_minus) / (2.0 * HE_);
        let alp = hlm(alp);
        unsafe {
            *ptr.add(idx) -= lr * alp;
        }
    }
}








pub fn pnb(model: &mut TransformerWeights, tokens: &[u8], lr: f32) -> f32 {
    if tokens.len() < 2 { return f32::MAX; }

    let (base_loss, _) = inference::atj(model, tokens);

    
    let mut rng = crate::time::yf().wrapping_mul(6364136223846793005);

    
    let gmt: Vec<f32> = (0..model.w_output.len()).map(|_| {
        rng ^= rng << 13; rng ^= rng >> 7; rng ^= rng << 17;
        let bits = (rng >> 40) as u32;
        (bits as f32 / (1u32 << 24) as f32) * 2.0 - 1.0
    }).collect();

    
    for (w, &aa) in model.w_output.iter_mut().zip(gmt.iter()) {
        *w += lr * aa;
    }

    
    let (new_loss, _) = inference::atj(model, tokens);

    if new_loss >= base_loss {
        
        for (w, &aa) in model.w_output.iter_mut().zip(gmt.iter()) {
            *w -= 2.0 * lr * aa; 
        }
        let (rev_loss, _) = inference::atj(model, tokens);
        if rev_loss >= base_loss {
            
            for (w, &aa) in model.w_output.iter_mut().zip(gmt.iter()) {
                *w += lr * aa;
            }
        }
    }

    base_loss
}






pub fn cdp() -> (u32, u32) {
    let mut gd = 0u32;
    let mut gv = 0u32;

    
    crate::serial_println!("[JARVIS-TRAIN] Test 1: Loss computation");
    {
        let model = TransformerWeights::bns();
        let tokens = tokenizer::bbj("Hello world");
        let (ka, logits) = inference::atj(&model, &tokens);
        if ka.is_finite() && logits.len() > 0 {
            crate::serial_println!("[JARVIS-TRAIN]   Loss = {:.4} (random weights)", ka);
            gd += 1;
        } else {
            crate::serial_println!("[JARVIS-TRAIN]   FAIL: loss={}", ka);
            gv += 1;
        }
    }

    
    crate::serial_println!("[JARVIS-TRAIN] Test 2: Training step");
    {
        let mut model = TransformerWeights::bns();
        let tokens = tokenizer::bbj("AB");
        let ka = jom(&mut model, &tokens, 0.01);
        if ka.is_finite() {
            crate::serial_println!("[JARVIS-TRAIN]   Initial loss = {:.4}", ka);
            gd += 1;
        } else {
            gv += 1;
        }
    }

    
    crate::serial_println!("[JARVIS-TRAIN] Test 3: Random perturbation training");
    {
        let mut model = TransformerWeights::bns();
        let tokens = tokenizer::bbj("Test");
        let ka = pnb(&mut model, &tokens, 0.001);
        if ka.is_finite() {
            crate::serial_println!("[JARVIS-TRAIN]   Loss = {:.4}", ka);
            gd += 1;
        } else {
            gv += 1;
        }
    }

    (gd, gv)
}






fn orx(idx: usize) -> bool {
    
    let h = idx.wrapping_mul(2654435761); 
    (h % 100) < (BET_ * 100.0) as usize
}


fn hlm(g: f32) -> f32 {
    if g > UW_ { UW_ }
    else if g < -UW_ { -UW_ }
    else { g }
}
