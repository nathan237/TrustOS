














use alloc::vec::Vec;
use core::sync::atomic::{AtomicBool, AtomicI32, AtomicU32, AtomicU64, Ordering};






const BCO_: u32 = 120;

const QT_: usize = 8;

const APZ_: u32 = 1000;

const APY_: u32 = 30;

const BT_: u32 = 48_000;

const BBW_: i32 = 500;






static LY_: AtomicBool = AtomicBool::new(false);


static YY_: AtomicU32 = AtomicU32::new(0);


static SK_: AtomicU32 = AtomicU32::new(0);


static AFV_: AtomicU64 = AtomicU64::new(0);


static AJO_: AtomicU64 = AtomicU64::new(0);


static ACA_: AtomicI32 = AtomicI32::new(0);


static AJB_: AtomicBool = AtomicBool::new(false);


static ACS_: AtomicU64 = AtomicU64::new(0);




static AKL_: spin::Mutex<Vec<i64>> = spin::Mutex::new(Vec::new());










pub fn qrd() -> u64 {
    use crate::drivers::hda;

    
    let cvd = hda::cyi(APZ_, 2, 16000);
    if cvd.is_empty() { return 0; }

    
    let _ = hda::stop();
    hda::eyn();

    
    let (buf_ptr, buf_cap) = match hda::cym() {
        Some(info) => info,
        None => return 0,
    };

    
    unsafe {
        core::ptr::write_bytes(buf_ptr, 0, buf_cap);
    }

    
    let flx = buf_cap / 4;
    let kla = cvd.len().min(buf_cap - flx);
    unsafe {
        core::ptr::copy_nonoverlapping(cvd.as_ptr(), buf_ptr.add(flx), kla);
    }

    
    let hlj = (flx * 2) as u32; 

    
    let jik = crate::gui::engine::yy();
    let _ = hda::owe();

    
    let gyv = 200_000; 
    loop {
        let cy = crate::gui::engine::yy();
        if cy - jik > gyv {
            let _ = hda::stop();
            crate::serial_println!("[SYNC] DMA probe timeout (200ms)");
            return 0;
        }

        let alw = hda::dqq();
        if alw >= hlj {
            let esk = cy - jik;
            let _ = hda::stop();
            ACS_.store(esk, Ordering::SeqCst);
            crate::serial_println!("[SYNC] DMA probe: click at byte {} reached after {} µs ({} ms)",
                hlj, esk, esk / 1000);
            return esk;
        }

        
        for _ in 0..100 {
            core::hint::spin_loop();
        }
    }
}






fn hhc() -> u64 {
    60_000_000 / BCO_ as u64  
}



fn mcf() -> Vec<i16> {
    crate::drivers::hda::cyi(APZ_, APY_, 20000)
}






pub fn qxq() {
    
    LY_.store(true, Ordering::SeqCst);
    YY_.store(0, Ordering::SeqCst);
    SK_.store(0, Ordering::SeqCst);
    AJB_.store(false, Ordering::SeqCst);
    ACA_.store(0, Ordering::SeqCst);
    {
        let mut cwh = AKL_.lock();
        cwh.clear();
    }

    let cy = crate::gui::engine::yy();
    AJO_.store(cy, Ordering::SeqCst);
    AFV_.store(0, Ordering::SeqCst);

    crate::serial_println!("[SYNC] Metronome calibration started ({} BPM, {} taps needed)",
        BCO_, QT_);

    
    ivc();
    AFV_.store(cy, Ordering::SeqCst);
    SK_.store(1, Ordering::SeqCst);
}


pub fn pzf() {
    LY_.store(false, Ordering::SeqCst);
    let _ = crate::drivers::hda::stop();
    crate::serial_println!("[SYNC] Calibration cancelled");
}




pub fn tick() -> bool {
    if !LY_.load(Ordering::SeqCst) {
        return false;
    }

    
    if YY_.load(Ordering::SeqCst) as usize >= QT_ {
        hza();
        return false;
    }

    let cy = crate::gui::engine::yy();
    let dze = AJO_.load(Ordering::SeqCst);
    let hhb = SK_.load(Ordering::SeqCst);
    let hxf = dze + hhb as u64 * hhc();

    if cy >= hxf {
        
        ivc();
        AFV_.store(hxf, Ordering::SeqCst);
        SK_.store(hhb + 1, Ordering::SeqCst);
    }

    true
}



pub fn qtf(tap_us: u64) {
    if !LY_.load(Ordering::SeqCst) {
        return;
    }

    
    let dze = AJO_.load(Ordering::SeqCst);
    let axr = hhc();

    if tap_us <= dze {
        return;
    }

    
    let bb = tap_us - dze;
    let nhw = ((bb + axr / 2) / axr) as u64;
    let nhx = dze + nhw * axr;

    
    
    let frk = tap_us as i64 - nhx as i64;

    {
        let mut cwh = AKL_.lock();
        cwh.push(frk);
    }

    let count = YY_.fetch_add(1, Ordering::SeqCst) + 1;
    crate::serial_println!("[SYNC] Tap {}/{}: delta = {} µs ({} ms)",
        count, QT_, frk, frk / 1000);

    
    if count as usize >= QT_ {
        hza();
    }
}


fn ivc() {
    let cvd = mcf();
    if !cvd.is_empty() {
        
        
        let _ = crate::drivers::hda::bxb(&cvd, APY_ + 5);
    }
}


fn hza() {
    LY_.store(false, Ordering::SeqCst);

    let cwh = {
        let d = AKL_.lock();
        d.clone()
    };

    if cwh.is_empty() {
        crate::serial_println!("[SYNC] No taps recorded");
        return;
    }

    
    let mut acq = cwh.clone();
    acq.sort();

    let inf = if acq.len() % 2 == 0 {
        (acq[acq.len() / 2 - 1] + acq[acq.len() / 2]) / 2
    } else {
        acq[acq.len() / 2]
    };

    
    let sum: i64 = acq.iter().sum();
    let inc = sum / acq.len() as i64;

    
    let prf: i64 = acq.iter()
        .map(|&d| {
            let jr = d - inc;
            jr * jr
        })
        .sum::<i64>() / acq.len() as i64;
    
    let oxd = muk(prf);
    let oxc = oxd / 1000;

    
    
    let ifg: i64 = 80_000;
    let jyf = inf - ifg;
    let isa = (jyf / 1000).clamp(-BBW_ as i64, BBW_ as i64) as i32;

    ACA_.store(isa, Ordering::SeqCst);
    AJB_.store(true, Ordering::SeqCst);

    crate::serial_println!("[SYNC] ──── Calibration Results ────");
    crate::serial_println!("[SYNC]   Taps collected: {}", acq.len());
    crate::serial_println!("[SYNC]   Median tap delta: {} ms", inf / 1000);
    crate::serial_println!("[SYNC]   Mean tap delta:   {} ms", inc / 1000);
    crate::serial_println!("[SYNC]   Std deviation:    {} ms", oxc);
    crate::serial_println!("[SYNC]   Human reaction:  -{} ms (subtracted)", ifg / 1000);
    crate::serial_println!("[SYNC]   → Computed A/V offset: {} ms", isa);

    
    let hsw = ACS_.load(Ordering::SeqCst);
    if hsw > 0 {
        crate::serial_println!("[SYNC]   DMA pipeline:    {} ms (measured)", hsw / 1000);
    }
}






pub fn is_active() -> bool {
    LY_.load(Ordering::SeqCst)
}


pub fn qia() -> i32 {
    ACA_.load(Ordering::SeqCst)
}


pub fn qkr() -> bool {
    AJB_.load(Ordering::SeqCst)
}


pub fn qyu() -> u32 {
    YY_.load(Ordering::SeqCst)
}


pub fn qyv() -> u32 {
    QT_ as u32
}


pub fn qdd() -> u64 {
    ACS_.load(Ordering::SeqCst)
}


fn muk(ae: i64) -> i64 {
    if ae <= 0 { return 0; }
    let mut x = ae;
    let mut y = (x + 1) / 2;
    while y < x {
        x = y;
        y = (x + ae / x) / 2;
    }
    x
}
