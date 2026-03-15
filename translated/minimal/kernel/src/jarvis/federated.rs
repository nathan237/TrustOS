




















use alloc::vec::Vec;
use alloc::format;
use alloc::string::String;
use core::sync::atomic::{AtomicU64, AtomicU32, AtomicBool, Ordering};
use spin::Mutex;






const CGN_: usize = 1;


const AZX_: usize = 16;



const DDI_: u64 = 30_000;


const AFQ_: u64 = 5_000;


const BAA_: u64 = 120_000;


const BVD_: f32 = 0.001;



const AHP_: f32 = 0.9;


const AVU_: f32 = 4.0;
const AER_: f32 = 2.0;



const BIC_: bool = true;






static NH_: Mutex<Vec<Vec<f32>>> = Mutex::new(Vec::new());


static ML_: Mutex<Vec<super::compression::Ii>> = Mutex::new(Vec::new());


static TD_: AtomicU64 = AtomicU64::new(0);


static ADB_: AtomicU64 = AtomicU64::new(0);


static AEI_: AtomicU64 = AtomicU64::new(0);


static TC_: AtomicBool = AtomicBool::new(false);


static AEA_: Mutex<f32> = Mutex::new(0.0);


static CSG_: Mutex<Vec<f32>> = Mutex::new(Vec::new());



static BCW_: Mutex<Vec<u64>> = Mutex::new(Vec::new());


static QY_: AtomicU64 = AtomicU64::new(30_000);


static RW_: AtomicU64 = AtomicU64::new(0);


static AAM_: AtomicU64 = AtomicU64::new(0);






pub fn aiy() {
    TC_.store(true, Ordering::SeqCst);
    AEI_.store(crate::time::lc(), Ordering::SeqCst);
    crate::serial_println!("[FED] Federated learning enabled");
}


pub fn cwz() {
    TC_.store(false, Ordering::SeqCst);
    crate::serial_println!("[FED] Federated learning disabled");
}


pub fn zu() -> bool {
    TC_.load(Ordering::SeqCst)
}


pub fn cm() -> String {
    let wag = TD_.load(Ordering::SeqCst);
    let vtd = ADB_.load(Ordering::SeqCst);
    let aln = NH_.lock().len();
    let rne = ML_.lock().len();
    let vl = *AEA_.lock();
    let crp = QY_.load(Ordering::SeqCst);
    let ehz = RW_.load(Ordering::SeqCst);
    let faf = AAM_.load(Ordering::SeqCst);

    format!("fed_rounds={} grads={} pending={}+{}c loss={:.4} interval={}ms saved={}KB transfers={}",
        wag, vtd, aln, rne, vl,
        crp, ehz / 1024, faf)
}


pub fn wah() -> u64 {
    TD_.load(Ordering::SeqCst)
}







pub fn pan(gqg: &[u8]) {
    if let Err(fr) = super::guardian::emj(super::guardian::ProtectedOp::Asf) {
        crate::serial_println!("[FED] Guardian denied gradient reception: {}", fr);
        return;
    }

    
    if gqg.len() >= 4 && &gqg[0..4] == b"JCMP" {
        if let Some(ahf) = super::compression::nks(gqg) {
            let mut dii = ML_.lock();
            if dii.len() < AZX_ {
                let ame = ahf.ch.len();
                dii.push(ahf);
                ADB_.fetch_add(1, Ordering::SeqCst);
                AAM_.fetch_add(1, Ordering::SeqCst);
                crate::serial_println!("[FED] Received compressed gradient ({} entries, {} pending)",
                    ame, dii.len());
            }
            return;
        }
    }

    
    let aue = super::rpc::kfu(gqg);
    if aue.is_empty() {
        return;
    }

    let mut dii = NH_.lock();
    if dii.len() < AZX_ {
        dii.push(aue);
        ADB_.fetch_add(1, Ordering::SeqCst);
        crate::serial_println!("[FED] Received raw gradient ({} pending)", dii.len());
    }
}


pub fn ziq(gqg: &[u8], vgn: u64) {
    let rrz = NH_.lock().len() + ML_.lock().len();
    pan(gqg);
    let utj = NH_.lock().len() + ML_.lock().len();
    if utj > rrz {
        BCW_.lock().push(vgn);
    }
}










pub fn poll() {
    if !TC_.load(Ordering::SeqCst) || !super::mesh::rl() {
        return;
    }

    let iu = crate::time::lc();
    let jct = AEI_.load(Ordering::SeqCst);
    let crp = QY_.load(Ordering::SeqCst);
    if iu.nj(jct) < crp {
        return;
    }

    if super::consensus::ogf() {
        oin();
    } else {
        pzr();
    }

    AEI_.store(iu, Ordering::SeqCst);

    
    xoq();
}












fn oin() {
    
    let jld = {
        let mut dii = NH_.lock();
        let at: Vec<Vec<f32>> = dii.bbk(..).collect();
        at
    };

    
    let iox = {
        let mut dii = ML_.lock();
        let ahf: Vec<super::compression::Ii> = dii.bbk(..).collect();
        ahf
    };

    
    let ovb = {
        let mut ars = BCW_.lock();
        let d: Vec<u64> = ars.bbk(..).collect();
        d
    };

    let cus = jld.len() + iox.len();
    if cus < CGN_ {
        return;
    }

    crate::serial_println!("[FED] Leader aggregating {} gradients ({} raw, {} compressed)",
        cus, jld.len(), iox.len());

    
    let vm = if let Some(fv) = jld.fv() {
        fv.len()
    } else if let Some(fv) = iox.fv() {
        fv.vm as usize
    } else {
        return;
    };

    
    let mut fyq = alloc::vec![0.0f32; vm];
    let mut jua: f64 = 0.0;
    let mut gin = 0usize;

    
    for buo in &jld {
        if buo.len() != vm {
            crate::serial_println!("[FED] Skipping mismatched gradient (got {} expected {})",
                buo.len(), vm);
            gin += 1;
            continue;
        }
        let amz = ovb.get(gin).hu().unwrap_or(1).am(1) as f64;
        for (a, &at) in buo.iter().cf() {
            fyq[a] += at * amz as f32;
        }
        jua += amz;
        gin += 1;
    }

    
    for ahf in &iox {
        let dpu = super::compression::rus(ahf);
        if dpu.len() != vm {
            gin += 1;
            continue;
        }
        let amz = ovb.get(gin).hu().unwrap_or(1).am(1) as f64;
        for (a, &at) in dpu.iter().cf() {
            fyq[a] += at * amz as f32;
        }
        jua += amz;
        gin += 1;
    }

    if jua <= 0.0 {
        return;
    }

    
    let tvy = 1.0 / jua as f32;
    for at in fyq.el() {
        *at *= tvy;
    }

    
    {
        let mut hrt = CSG_.lock();
        if hrt.len() != vm {
            
            *hrt = fyq.clone();
        } else {
            for a in 0..vm {
                hrt[a] = AHP_ * hrt[a]
                    + (1.0 - AHP_) * fyq[a];
            }
            
            for a in 0..vm {
                fyq[a] = hrt[a];
            }
        }
    }

    
    qjv(&fyq);

    TD_.fetch_add(1, Ordering::SeqCst);
    crate::serial_println!("[FED] Round {} complete (momentum={}, {} peers) — pushing delta weights",
        TD_.load(Ordering::SeqCst), AHP_, super::mesh::cti());

    
    oyn();
}


fn qjv(dhs: &[f32]) {
    let mut upi = super::Ci.lock();
    let model = match upi.as_mut() {
        Some(ef) => ef,
        None => return,
    };

    let cv = model.gsd();
    if cv.len() != dhs.len() {
        crate::serial_println!("[FED] Gradient size mismatch: model={} grad={}",
            cv.len(), dhs.len());
        return;
    }

    
    let mut juv = Vec::fc(cv.len());
    for (d, at) in cv.iter().fca(dhs.iter()) {
        juv.push(d - BVD_ * at);
    }

    
    if let Some(hst) = super::model::TransformerWeights::eos(&juv) {
        *model = hst;
    }
}




fn oyn() {
    let cps = {
        let model = super::Ci.lock();
        match model.as_ref() {
            Some(ef) => ef.gsd(),
            None => return,
        }
    };

    let yp = super::mesh::dhn();

    if BIC_ {
        
        let aaq = super::compression::rnm(&cps);
        let gdd = super::compression::pig(&aaq);
        let ghs = cps.len() * 4;
        let hdu = gdd.len();

        RW_.fetch_add((ghs - hdu) as u64, Ordering::Relaxed);

        crate::serial_println!("[FED] Delta compressed: {} entries, {} KB → {} KB ({}× compression)",
            aaq.ch.len(), ghs / 1024, hdu / 1024,
            if hdu > 0 { ghs / hdu } else { 0 });

        for ko in &yp {
            match super::rpc::oym(ko.ip, ko.bsb, &gdd) {
                Ok(()) => {
                    crate::serial_println!("[FED] Pushed delta to {}.{}.{}.{} ({} KB)",
                        ko.ip[0], ko.ip[1], ko.ip[2], ko.ip[3],
                        hdu / 1024);
                }
                Err(aa) => {
                    crate::serial_println!("[FED] Failed to push to {}.{}.{}.{}: {}",
                        ko.ip[0], ko.ip[1], ko.ip[2], ko.ip[3], aa);
                }
            }
        }
    } else {
        
        let mqk = super::rpc::nvc(&cps);
        for ko in &yp {
            match super::rpc::oym(ko.ip, ko.bsb, &mqk) {
                Ok(()) => {
                    crate::serial_println!("[FED] Pushed full weights to {}.{}.{}.{}",
                        ko.ip[0], ko.ip[1], ko.ip[2], ko.ip[3]);
                }
                Err(aa) => {
                    crate::serial_println!("[FED] Failed to push to {}.{}.{}.{}: {}",
                        ko.ip[0], ko.ip[1], ko.ip[2], ko.ip[3], aa);
                }
            }
        }
    }
}






fn pzr() {
    if !super::uc() {
        return;
    }

    
    let bnj = match super::mesh::kyu() {
        Some(dm) => dm,
        None => return, 
    };

    
    let ixf = rnj();
    if ixf.is_empty() {
        return;
    }

    
    let (ixe, twz) = if BIC_ {
        let ahf = super::compression::nfg(&ixf);
        let bf = super::compression::pig(&ahf);
        let ghs = ixf.len() * 4;
        let nfe = bf.len();
        RW_.fetch_add((ghs - nfe) as u64, Ordering::Relaxed);
        AAM_.fetch_add(1, Ordering::Relaxed);
        crate::serial_println!("[FED] Compressed gradient: {} entries, {} KB → {} KB",
            ahf.ch.len(), ghs / 1024, nfe / 1024);
        (bf, true)
    } else {
        (super::rpc::nvc(&ixf), false)
    };

    crate::serial_println!("[FED] Sending {} gradient to leader {}.{}.{}.{} ({} KB)",
        if twz { "compressed" } else { "raw" },
        bnj.ip[0], bnj.ip[1], bnj.ip[2], bnj.ip[3],
        ixe.len() / 1024);

    match super::rpc::voi(bnj.ip, bnj.bsb, &ixe) {
        Ok(()) => {
            crate::serial_println!("[FED] Gradients sent successfully");
        }
        Err(aa) => {
            crate::serial_println!("[FED] Failed to send gradients: {}", aa);
        }
    }
}



fn rnj() -> Vec<f32> {
    let model = super::Ci.lock();
    let upj = match model.as_ref() {
        Some(ef) => ef,
        None => return Vec::new(),
    };

    
    let yr = super::corpus::nyk();
    let pfh = yr.as_bytes();

    if pfh.len() < 2 {
        return Vec::new();
    }

    
    let (qcl, arg) = super::backprop::ivk(upj, pfh);

    *AEA_.lock() = qcl;

    
    why(&arg)
}


fn why(arg: &super::backprop::ModelGrads) -> Vec<f32> {
    let mut f = Vec::new();
    f.bk(&arg.dfs);
    f.bk(&arg.dfo);
    for fl in &arg.my {
        f.bk(&fl.dfp);
        f.bk(&fl.dfx);
        f.bk(&fl.dfv);
        f.bk(&fl.dfz);
        f.bk(&fl.dfw);
        f.bk(&fl.dfq);
        f.bk(&fl.dfu);
        f.bk(&fl.dfy);
        f.bk(&fl.dft);
    }
    f.bk(&arg.dfr);
    f.bk(&arg.dfn);
    f
}






pub fn svq() {
    if super::consensus::ogf() {
        oin();
    } else {
        pzr();
    }
}


pub fn vxk() {
    if !super::uc() {
        crate::serial_println!("[FED] Brain not ready");
        return;
    }

    
    let ma = super::io_control::hkp();
    let ol = super::io_control::gdg(&ma);
    if !super::io_control::hsn(&ma) {
        crate::serial_println!("[FED] Cannot replicate: I/O not network-ready (score={}%)", ol);
        return;
    }
    crate::serial_println!("[FED] I/O audit passed (score={}%), replicating model", ol);

    oyn();
}


pub fn vnz() -> Result<(), &'static str> {
    let bnj = super::mesh::kyu().ok_or("No leader found")?;

    let cvg = super::rpc::kyz(bnj.ip, bnj.bsb)?;

    
    if cvg.len() >= 4 && &cvg[0..4] == b"JCMP" {
        if let Some(aaq) = super::compression::nks(&cvg) {
            let mut model = super::Ci.lock();
            if let Some(ef) = model.as_mut() {
                let mut bix = ef.gsd();
                super::compression::qkc(&mut bix, &aaq);
                if let Some(juv) = super::model::TransformerWeights::eos(&bix) {
                    *ef = juv;
                    crate::serial_println!("[FED] Model synced from leader (compressed delta, {} entries)",
                        aaq.ch.len());
                    return Ok(());
                }
            }
            return Err("Failed to apply compressed delta");
        }
    }

    
    let aue = super::rpc::kfu(&cvg);
    match super::model::TransformerWeights::eos(&aue) {
        Some(hst) => {
            *super::Ci.lock() = Some(hst);
            crate::serial_println!("[FED] Model synced from leader (full weights)");
            Ok(())
        }
        None => Err("Failed to deserialize leader weights"),
    }
}









fn xoq() {
    let vl = *AEA_.lock();
    if vl <= 0.0 || !vl.dsg() {
        return; 
    }

    let lnv = if vl > AVU_ {
        AFQ_
    } else if vl < AER_ {
        BAA_
    } else {
        
        let ab = (vl - AER_) / (AVU_ - AER_);
        let cmb = BAA_ as f32 - AFQ_ as f32;
        (AFQ_ as f32 + cmb * (1.0 - ab)) as u64
    };

    let aft = QY_.swap(lnv, Ordering::SeqCst);
    if (lnv as i64 - aft as i64).eki() > 5000 {
        crate::serial_println!("[FED] Adaptive interval: {}ms → {}ms (loss={:.3})",
            aft, lnv, vl);
    }
}


pub fn rrx() -> u64 {
    QY_.load(Ordering::SeqCst)
}


pub fn qvc() -> u64 {
    RW_.load(Ordering::SeqCst)
}
