




















use alloc::vec::Vec;
use alloc::format;
use alloc::string::String;
use core::sync::atomic::{AtomicU64, AtomicU32, AtomicBool, Ordering};
use spin::Mutex;






const CJX_: usize = 1;


const BBZ_: usize = 16;



const DHC_: u64 = 30_000;


const AHK_: u64 = 5_000;


const BCC_: u64 = 120_000;


const BXZ_: f32 = 0.001;



const AJM_: f32 = 0.9;


const AXY_: f32 = 4.0;
const AGL_: f32 = 2.0;



const BKJ_: bool = true;






static OG_: Mutex<Vec<Vec<f32>>> = Mutex::new(Vec::new());


static NK_: Mutex<Vec<super::compression::Dm>> = Mutex::new(Vec::new());


static UJ_: AtomicU64 = AtomicU64::new(0);


static AER_: AtomicU64 = AtomicU64::new(0);


static AGC_: AtomicU64 = AtomicU64::new(0);


static UI_: AtomicBool = AtomicBool::new(false);


static AFU_: Mutex<f32> = Mutex::new(0.0);


static CVX_: Mutex<Vec<f32>> = Mutex::new(Vec::new());



static BEZ_: Mutex<Vec<u64>> = Mutex::new(Vec::new());


static RT_: AtomicU64 = AtomicU64::new(30_000);


static SY_: AtomicU64 = AtomicU64::new(0);


static ABZ_: AtomicU64 = AtomicU64::new(0);






pub fn enable() {
    UI_.store(true, Ordering::SeqCst);
    AGC_.store(crate::time::uptime_ms(), Ordering::SeqCst);
    crate::serial_println!("[FED] Federated learning enabled");
}


pub fn bbc() {
    UI_.store(false, Ordering::SeqCst);
    crate::serial_println!("[FED] Federated learning disabled");
}


pub fn lq() -> bool {
    UI_.load(Ordering::SeqCst)
}


pub fn stats() -> String {
    let oif = UJ_.load(Ordering::SeqCst);
    let odp = AER_.load(Ordering::SeqCst);
    let pending = OG_.lock().len();
    let kwj = NK_.lock().len();
    let ka = *AFU_.lock();
    let axr = RT_.load(Ordering::SeqCst);
    let bvq = SY_.load(Ordering::SeqCst);
    let transfers = ABZ_.load(Ordering::SeqCst);

    format!("fed_rounds={} grads={} pending={}+{}c loss={:.4} interval={}ms saved={}KB transfers={}",
        oif, odp, pending, kwj, ka,
        axr, bvq / 1024, transfers)
}


pub fn oig() -> u64 {
    UJ_.load(Ordering::SeqCst)
}







pub fn iyq(raw_bytes: &[u8]) {
    if let Err(bk) = super::guardian::bxo(super::guardian::ProtectedOp::FederatedSync) {
        crate::serial_println!("[FED] Guardian denied gradient reception: {}", bk);
        return;
    }

    
    if raw_bytes.len() >= 4 && &raw_bytes[0..4] == b"JCMP" {
        if let Some(qv) = super::compression::hro(raw_bytes) {
            let mut bhd = NK_.lock();
            if bhd.len() < BBZ_ {
                let entry_count = qv.entries.len();
                bhd.push(qv);
                AER_.fetch_add(1, Ordering::SeqCst);
                ABZ_.fetch_add(1, Ordering::SeqCst);
                crate::serial_println!("[FED] Received compressed gradient ({} entries, {} pending)",
                    entry_count, bhd.len());
            }
            return;
        }
    }

    
    let xn = super::rpc::fkj(raw_bytes);
    if xn.is_empty() {
        return;
    }

    let mut bhd = OG_.lock();
    if bhd.len() < BBZ_ {
        bhd.push(xn);
        AER_.fetch_add(1, Ordering::SeqCst);
        crate::serial_println!("[FED] Received raw gradient ({} pending)", bhd.len());
    }
}


pub fn qtb(raw_bytes: &[u8], peer_steps: u64) {
    let lan = OG_.lock().len() + NK_.lock().len();
    iyq(raw_bytes);
    let njk = OG_.lock().len() + NK_.lock().len();
    if njk > lan {
        BEZ_.lock().push(peer_steps);
    }
}










pub fn poll() {
    if !UI_.load(Ordering::SeqCst) || !super::mesh::is_active() {
        return;
    }

    let cy = crate::time::uptime_ms();
    let last_sync = AGC_.load(Ordering::SeqCst);
    let axr = RT_.load(Ordering::SeqCst);
    if cy.wrapping_sub(last_sync) < axr {
        return;
    }

    if super::consensus::iia() {
        ijr();
    } else {
        jrh();
    }

    AGC_.store(cy, Ordering::SeqCst);

    
    ppt();
}












fn ijr() {
    
    let exr = {
        let mut bhd = OG_.lock();
        let g: Vec<Vec<f32>> = bhd.drain(..).collect();
        g
    };

    
    let eir = {
        let mut bhd = NK_.lock();
        let qv: Vec<super::compression::Dm> = bhd.drain(..).collect();
        qv
    };

    
    let iui = {
        let mut wl = BEZ_.lock();
        let w: Vec<u64> = wl.drain(..).collect();
        w
    };

    let total_count = exr.len() + eir.len();
    if total_count < CJX_ {
        return;
    }

    crate::serial_println!("[FED] Leader aggregating {} gradients ({} raw, {} compressed)",
        total_count, exr.len(), eir.len());

    
    let param_count = if let Some(first) = exr.first() {
        first.len()
    } else if let Some(first) = eir.first() {
        first.param_count as usize
    } else {
        return;
    };

    
    let mut cst = alloc::vec![0.0f32; param_count];
    let mut fdm: f64 = 0.0;
    let mut cyx = 0usize;

    
    for alp in &exr {
        if alp.len() != param_count {
            crate::serial_println!("[FED] Skipping mismatched gradient (got {} expected {})",
                alp.len(), param_count);
            cyx += 1;
            continue;
        }
        let tv = iui.get(cyx).copied().unwrap_or(1).max(1) as f64;
        for (i, &g) in alp.iter().enumerate() {
            cst[i] += g * tv as f32;
        }
        fdm += tv;
        cyx += 1;
    }

    
    for qv in &eir {
        let blr = super::compression::lct(qv);
        if blr.len() != param_count {
            cyx += 1;
            continue;
        }
        let tv = iui.get(cyx).copied().unwrap_or(1).max(1) as f64;
        for (i, &g) in blr.iter().enumerate() {
            cst[i] += g * tv as f32;
        }
        fdm += tv;
        cyx += 1;
    }

    if fdm <= 0.0 {
        return;
    }

    
    let mrk = 1.0 / fdm as f32;
    for g in cst.iter_mut() {
        *g *= mrk;
    }

    
    {
        let mut dum = CVX_.lock();
        if dum.len() != param_count {
            
            *dum = cst.clone();
        } else {
            for i in 0..param_count {
                dum[i] = AJM_ * dum[i]
                    + (1.0 - AJM_) * cst[i];
            }
            
            for i in 0..param_count {
                cst[i] = dum[i];
            }
        }
    }

    
    jxb(&cst);

    UJ_.fetch_add(1, Ordering::SeqCst);
    crate::serial_println!("[FED] Round {} complete (momentum={}, {} peers) — pushing delta weights",
        UJ_.load(Ordering::SeqCst), AJM_, super::mesh::ayz());

    
    ixd();
}


fn jxb(bgs: &[f32]) {
    let mut nfz = super::Ay.lock();
    let model = match nfz.as_mut() {
        Some(m) => m,
        None => return,
    };

    let current = model.serialize();
    if current.len() != bgs.len() {
        crate::serial_println!("[FED] Gradient size mismatch: model={} grad={}",
            current.len(), bgs.len());
        return;
    }

    
    let mut fdz = Vec::with_capacity(current.len());
    for (w, g) in current.iter().zip(bgs.iter()) {
        fdz.push(w - BXZ_ * g);
    }

    
    if let Some(new_weights) = super::model::TransformerWeights::byt(&fdz) {
        *model = new_weights;
    }
}




fn ixd() {
    let awt = {
        let model = super::Ay.lock();
        match model.as_ref() {
            Some(m) => m.serialize(),
            None => return,
        }
    };

    let lj = super::mesh::bgo();

    if BKJ_ {
        
        let mk = super::compression::kwq(&awt);
        let cvl = super::compression::jet(&mk);
        let cyg = awt.len() * 4;
        let dlg = cvl.len();

        SY_.fetch_add((cyg - dlg) as u64, Ordering::Relaxed);

        crate::serial_println!("[FED] Delta compressed: {} entries, {} KB → {} KB ({}× compression)",
            mk.entries.len(), cyg / 1024, dlg / 1024,
            if dlg > 0 { cyg / dlg } else { 0 });

        for peer in &lj {
            match super::rpc::ixc(peer.ip, peer.rpc_port, &cvl) {
                Ok(()) => {
                    crate::serial_println!("[FED] Pushed delta to {}.{}.{}.{} ({} KB)",
                        peer.ip[0], peer.ip[1], peer.ip[2], peer.ip[3],
                        dlg / 1024);
                }
                Err(e) => {
                    crate::serial_println!("[FED] Failed to push to {}.{}.{}.{}: {}",
                        peer.ip[0], peer.ip[1], peer.ip[2], peer.ip[3], e);
                }
            }
        }
    } else {
        
        let hcg = super::rpc::hzf(&awt);
        for peer in &lj {
            match super::rpc::ixc(peer.ip, peer.rpc_port, &hcg) {
                Ok(()) => {
                    crate::serial_println!("[FED] Pushed full weights to {}.{}.{}.{}",
                        peer.ip[0], peer.ip[1], peer.ip[2], peer.ip[3]);
                }
                Err(e) => {
                    crate::serial_println!("[FED] Failed to push to {}.{}.{}.{}: {}",
                        peer.ip[0], peer.ip[1], peer.ip[2], peer.ip[3], e);
                }
            }
        }
    }
}






fn jrh() {
    if !super::is_ready() {
        return;
    }

    
    let aid = match super::mesh::fyt() {
        Some(l) => l,
        None => return, 
    };

    
    let eol = kwn();
    if eol.is_empty() {
        return;
    }

    
    let (grad_bytes, is_compressed) = if BKJ_ {
        let qv = super::compression::hne(&eol);
        let bytes = super::compression::jet(&qv);
        let cyg = eol.len() * 4;
        let hnd = bytes.len();
        SY_.fetch_add((cyg - hnd) as u64, Ordering::Relaxed);
        ABZ_.fetch_add(1, Ordering::Relaxed);
        crate::serial_println!("[FED] Compressed gradient: {} entries, {} KB → {} KB",
            qv.entries.len(), cyg / 1024, hnd / 1024);
        (bytes, true)
    } else {
        (super::rpc::hzf(&eol), false)
    };

    crate::serial_println!("[FED] Sending {} gradient to leader {}.{}.{}.{} ({} KB)",
        if is_compressed { "compressed" } else { "raw" },
        aid.ip[0], aid.ip[1], aid.ip[2], aid.ip[3],
        grad_bytes.len() / 1024);

    match super::rpc::nzr(aid.ip, aid.rpc_port, &grad_bytes) {
        Ok(()) => {
            crate::serial_println!("[FED] Gradients sent successfully");
        }
        Err(e) => {
            crate::serial_println!("[FED] Failed to send gradients: {}", e);
        }
    }
}



fn kwn() -> Vec<f32> {
    let model = super::Ay.lock();
    let nga = match model.as_ref() {
        Some(m) => m,
        None => return Vec::new(),
    };

    
    let sample = super::corpus::ibu();
    let jck = sample.as_bytes();

    if jck.len() < 2 {
        return Vec::new();
    }

    
    let (_loss, wg) = super::backprop::eng(nga, jck);

    *AFU_.lock() = _loss;

    
    ooj(&wg)
}


fn ooj(wg: &super::backprop::ModelGrads) -> Vec<f32> {
    let mut data = Vec::new();
    data.extend_from_slice(&wg.d_token_embed);
    data.extend_from_slice(&wg.d_pos_embed);
    for bj in &wg.layers {
        data.extend_from_slice(&bj.d_rms_attn);
        data.extend_from_slice(&bj.d_wq);
        data.extend_from_slice(&bj.d_wk);
        data.extend_from_slice(&bj.d_wv);
        data.extend_from_slice(&bj.d_wo);
        data.extend_from_slice(&bj.d_rms_ffn);
        data.extend_from_slice(&bj.d_wgate);
        data.extend_from_slice(&bj.d_wup);
        data.extend_from_slice(&bj.d_wdown);
    }
    data.extend_from_slice(&wg.d_rms_final);
    data.extend_from_slice(&wg.d_output);
    data
}






pub fn lxj() {
    if super::consensus::iia() {
        ijr();
    } else {
        jrh();
    }
}


pub fn oga() {
    if !super::is_ready() {
        crate::serial_println!("[FED] Brain not ready");
        return;
    }

    
    let audit = super::io_control::dqi();
    let score = super::io_control::cvo(&audit);
    if !super::io_control::duz(&audit) {
        crate::serial_println!("[FED] Cannot replicate: I/O not network-ready (score={}%)", score);
        return;
    }
    crate::serial_println!("[FED] I/O audit passed (score={}%), replicating model", score);

    ixd();
}


pub fn nzj() -> Result<(), &'static str> {
    let aid = super::mesh::fyt().ok_or("No leader found")?;

    let baf = super::rpc::fyy(aid.ip, aid.rpc_port)?;

    
    if baf.len() >= 4 && &baf[0..4] == b"JCMP" {
        if let Some(mk) = super::compression::hro(&baf) {
            let mut model = super::Ay.lock();
            if let Some(m) = model.as_mut() {
                let mut afx = m.serialize();
                super::compression::jxg(&mut afx, &mk);
                if let Some(fdz) = super::model::TransformerWeights::byt(&afx) {
                    *m = fdz;
                    crate::serial_println!("[FED] Model synced from leader (compressed delta, {} entries)",
                        mk.entries.len());
                    return Ok(());
                }
            }
            return Err("Failed to apply compressed delta");
        }
    }

    
    let xn = super::rpc::fkj(&baf);
    match super::model::TransformerWeights::byt(&xn) {
        Some(new_weights) => {
            *super::Ay.lock() = Some(new_weights);
            crate::serial_println!("[FED] Model synced from leader (full weights)");
            Ok(())
        }
        None => Err("Failed to deserialize leader weights"),
    }
}









fn ppt() {
    let ka = *AFU_.lock();
    if ka <= 0.0 || !ka.is_finite() {
        return; 
    }

    let gjb = if ka > AXY_ {
        AHK_
    } else if ka < AGL_ {
        BCC_
    } else {
        
        let t = (ka - AGL_) / (AXY_ - AGL_);
        let range = BCC_ as f32 - AHK_ as f32;
        (AHK_ as f32 + range * (1.0 - t)) as u64
    };

    let qb = RT_.swap(gjb, Ordering::SeqCst);
    if (gjb as i64 - qb as i64).unsigned_abs() > 5000 {
        crate::serial_println!("[FED] Adaptive interval: {}ms → {}ms (loss={:.3})",
            qb, gjb, ka);
    }
}


pub fn lal() -> u64 {
    RT_.load(Ordering::SeqCst)
}


pub fn kgq() -> u64 {
    SY_.load(Ordering::SeqCst)
}
