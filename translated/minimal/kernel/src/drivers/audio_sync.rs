














use alloc::vec::Vec;
use core::sync::atomic::{AtomicBool, AtomicI32, AtomicU32, AtomicU64, Ordering};






const BAM_: u32 = 120;

const PW_: usize = 8;

const ANV_: u32 = 1000;

const ANU_: u32 = 30;

const BR_: u32 = 48_000;

const AZU_: i32 = 500;






static LE_: AtomicBool = AtomicBool::new(false);


static XR_: AtomicU32 = AtomicU32::new(0);


static RI_: AtomicU32 = AtomicU32::new(0);


static AEB_: AtomicU64 = AtomicU64::new(0);


static AHR_: AtomicU64 = AtomicU64::new(0);


static AAN_: AtomicI32 = AtomicI32::new(0);


static AHF_: AtomicBool = AtomicBool::new(false);


static ABC_: AtomicU64 = AtomicU64::new(0);




static AIQ_: spin::Mutex<Vec<i64>> = spin::Mutex::new(Vec::new());










pub fn zgo() -> u64 {
    use crate::drivers::hda;

    
    let gcq = hda::ghw(ANV_, 2, 16000);
    if gcq.is_empty() { return 0; }

    
    let _ = hda::qg();
    hda::jmf();

    
    let (aeg, gbq) = match hda::gic() {
        Some(co) => co,
        None => return 0,
    };

    
    unsafe {
        core::ptr::ahx(aeg, 0, gbq);
    }

    
    let khy = gbq / 4;
    let rbn = gcq.len().v(gbq - khy);
    unsafe {
        core::ptr::copy_nonoverlapping(gcq.fq(), aeg.add(khy), rbn);
    }

    
    let ndj = (khy * 2) as u32; 

    
    let pog = crate::gui::engine::awf();
    let _ = hda::wsr();

    
    let mkv = 200_000; 
    loop {
        let iu = crate::gui::engine::awf();
        if iu - pog > mkv {
            let _ = hda::qg();
            crate::serial_println!("[SYNC] DMA probe timeout (200ms)");
            return 0;
        }

        let bvg = hda::hlj();
        if bvg >= ndj {
            let jcw = iu - pog;
            let _ = hda::qg();
            ABC_.store(jcw, Ordering::SeqCst);
            crate::serial_println!("[SYNC] DMA probe: click at byte {} reached after {} µs ({} ms)",
                ndj, jcw, jcw / 1000);
            return jcw;
        }

        
        for _ in 0..100 {
            core::hint::hc();
        }
    }
}






fn myk() -> u64 {
    60_000_000 / BAM_ as u64  
}



fn tby() -> Vec<i16> {
    crate::drivers::hda::ghw(ANV_, ANU_, 20000)
}






pub fn zpo() {
    
    LE_.store(true, Ordering::SeqCst);
    XR_.store(0, Ordering::SeqCst);
    RI_.store(0, Ordering::SeqCst);
    AHF_.store(false, Ordering::SeqCst);
    AAN_.store(0, Ordering::SeqCst);
    {
        let mut gek = AIQ_.lock();
        gek.clear();
    }

    let iu = crate::gui::engine::awf();
    AHR_.store(iu, Ordering::SeqCst);
    AEB_.store(0, Ordering::SeqCst);

    crate::serial_println!("[SYNC] Metronome calibration started ({} BPM, {} taps needed)",
        BAM_, PW_);

    
    ovz();
    AEB_.store(iu, Ordering::SeqCst);
    RI_.store(1, Ordering::SeqCst);
}


pub fn yhi() {
    LE_.store(false, Ordering::SeqCst);
    let _ = crate::drivers::hda::qg();
    crate::serial_println!("[SYNC] Calibration cancelled");
}




pub fn or() -> bool {
    if !LE_.load(Ordering::SeqCst) {
        return false;
    }

    
    if XR_.load(Ordering::SeqCst) as usize >= PW_ {
        nup();
        return false;
    }

    let iu = crate::gui::engine::awf();
    let hzv = AHR_.load(Ordering::SeqCst);
    let myj = RI_.load(Ordering::SeqCst);
    let nry = hzv + myj as u64 * myk();

    if iu >= nry {
        
        ovz();
        AEB_.store(nry, Ordering::SeqCst);
        RI_.store(myj + 1, Ordering::SeqCst);
    }

    true
}



pub fn ziu(mjv: u64) {
    if !LE_.load(Ordering::SeqCst) {
        return;
    }

    
    let hzv = AHR_.load(Ordering::SeqCst);
    let crp = myk();

    if mjv <= hzv {
        return;
    }

    
    let ez = mjv - hzv;
    let urq = ((ez + crp / 2) / crp) as u64;
    let urr = hzv + urq * crp;

    
    
    let koz = mjv as i64 - urr as i64;

    {
        let mut gek = AIQ_.lock();
        gek.push(koz);
    }

    let az = XR_.fetch_add(1, Ordering::SeqCst) + 1;
    crate::serial_println!("[SYNC] Tap {}/{}: delta = {} µs ({} ms)",
        az, PW_, koz, koz / 1000);

    
    if az as usize >= PW_ {
        nup();
    }
}


fn ovz() {
    let gcq = tby();
    if !gcq.is_empty() {
        
        
        let _ = crate::drivers::hda::ele(&gcq, ANU_ + 5);
    }
}


fn nup() {
    LE_.store(false, Ordering::SeqCst);

    let gek = {
        let bc = AIQ_.lock();
        bc.clone()
    };

    if gek.is_empty() {
        crate::serial_println!("[SYNC] No taps recorded");
        return;
    }

    
    let mut bcs = gek.clone();
    bcs.jqs();

    let oms = if bcs.len() % 2 == 0 {
        (bcs[bcs.len() / 2 - 1] + bcs[bcs.len() / 2]) / 2
    } else {
        bcs[bcs.len() / 2]
    };

    
    let sum: i64 = bcs.iter().sum();
    let omp = sum / bcs.len() as i64;

    
    let xqr: i64 = bcs.iter()
        .map(|&bc| {
            let wz = bc - omp;
            wz * wz
        })
        .sum::<i64>() / bcs.len() as i64;
    
    let wua = tzs(xqr);
    let wtz = wua / 1000;

    
    
    let ocm: i64 = 80_000;
    let qle = oms - ocm;
    let ose = (qle / 1000).qp(-AZU_ as i64, AZU_ as i64) as i32;

    AAN_.store(ose, Ordering::SeqCst);
    AHF_.store(true, Ordering::SeqCst);

    crate::serial_println!("[SYNC] ──── Calibration Results ────");
    crate::serial_println!("[SYNC]   Taps collected: {}", bcs.len());
    crate::serial_println!("[SYNC]   Median tap delta: {} ms", oms / 1000);
    crate::serial_println!("[SYNC]   Mean tap delta:   {} ms", omp / 1000);
    crate::serial_println!("[SYNC]   Std deviation:    {} ms", wtz);
    crate::serial_println!("[SYNC]   Human reaction:  -{} ms (subtracted)", ocm / 1000);
    crate::serial_println!("[SYNC]   → Computed A/V offset: {} ms", ose);

    
    let nmf = ABC_.load(Ordering::SeqCst);
    if nmf > 0 {
        crate::serial_println!("[SYNC]   DMA pipeline:    {} ms (measured)", nmf / 1000);
    }
}






pub fn rl() -> bool {
    LE_.load(Ordering::SeqCst)
}


pub fn ytk() -> i32 {
    AAN_.load(Ordering::SeqCst)
}


pub fn ywm() -> bool {
    AHF_.load(Ordering::SeqCst)
}


pub fn zqz() -> u32 {
    XR_.load(Ordering::SeqCst)
}


pub fn zra() -> u32 {
    PW_ as u32
}


pub fn yml() -> u64 {
    ABC_.load(Ordering::SeqCst)
}


fn tzs(bo: i64) -> i64 {
    if bo <= 0 { return 0; }
    let mut b = bo;
    let mut c = (b + 1) / 2;
    while c < b {
        b = c;
        c = (b + bo / b) / 2;
    }
    b
}
