






















use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use core::sync::atomic::{AtomicU32, AtomicBool, Ordering};
use spin::Mutex;





const CFH_: usize = 8;

struct Bll {
    j: String,
    iy: String,
}

struct EffectRegistry {
    bzl: Vec<Bll>,
    dnx: Option<usize>,
}

impl EffectRegistry {
    const fn new() -> Self {
        Self {
            bzl: Vec::new(),
            dnx: None,
        }
    }
}

static Ev: Mutex<EffectRegistry> = Mutex::new(EffectRegistry::new());



static ALA_: AtomicU32 = AtomicU32::new(0);
static AKZ_: AtomicU32 = AtomicU32::new(0);
static ALE_: AtomicU32 = AtomicU32::new(0);
static ALD_: AtomicU32 = AtomicU32::new(0);
static ALC_: AtomicU32 = AtomicU32::new(0);
static ALF_: AtomicU32 = AtomicU32::new(0);
static ALB_: AtomicU32 = AtomicU32::new(0);
static ACJ_: AtomicU32 = AtomicU32::new(0);
static IF_: AtomicBool = AtomicBool::new(false);






pub fn wif(rf: f32, aee: f32, ato: f32, vs: f32,
                      fkq: f32, axg: f32, abo: f32, frame: u32) {
    ALA_.store(rf.bsr(), Ordering::Relaxed);
    AKZ_.store(aee.bsr(), Ordering::Relaxed);
    ALE_.store(ato.bsr(), Ordering::Relaxed);
    ALD_.store(vs.bsr(), Ordering::Relaxed);
    ALC_.store(fkq.bsr(), Ordering::Relaxed);
    ALF_.store(axg.bsr(), Ordering::Relaxed);
    ALB_.store(abo.bsr(), Ordering::Relaxed);
    ACJ_.store(frame, Ordering::Relaxed);
}


pub fn tcw() -> f32 { f32::bhb(ALA_.load(Ordering::Relaxed)) }
pub fn tcv() -> f32 { f32::bhb(AKZ_.load(Ordering::Relaxed)) }
pub fn tet() -> f32 { f32::bhb(ALE_.load(Ordering::Relaxed)) }
pub fn teb() -> f32 { f32::bhb(ALD_.load(Ordering::Relaxed)) }
pub fn tdr() -> f32 { f32::bhb(ALC_.load(Ordering::Relaxed)) }
pub fn tez() -> f32 { f32::bhb(ALF_.load(Ordering::Relaxed)) }
pub fn tdo() -> f32 { f32::bhb(ALB_.load(Ordering::Relaxed)) }
pub fn tdq() -> u32 { ACJ_.load(Ordering::Relaxed) }


pub fn rl() -> bool {
    IF_.load(Ordering::Relaxed)
}






pub fn iix(j: &str, iy: &str) -> Result<(), &'static str> {
    let mut reg = Ev.lock();
    if reg.bzl.len() >= CFH_ {
        return Err("Maximum 8 effects reached — remove one first");
    }
    if reg.bzl.iter().any(|aa| aa.j == j) {
        return Err("Effect with this name already exists — use 'vizfx edit' to update");
    }
    
    if let Err(aa) = crate::trustlang::feq(iy) {
        crate::serial_println!("[VIZFX] Syntax error: {}", aa);
        return Err("TrustLang syntax error — check your script");
    }
    reg.bzl.push(Bll {
        j: String::from(j),
        iy: String::from(iy),
    });
    
    if reg.bzl.len() == 1 {
        reg.dnx = Some(0);
        IF_.store(true, Ordering::Relaxed);
    }
    Ok(())
}


pub fn sja(j: &str, iy: &str) -> Result<(), &'static str> {
    
    if let Err(aa) = crate::trustlang::feq(iy) {
        crate::serial_println!("[VIZFX] Syntax error: {}", aa);
        return Err("TrustLang syntax error — check your script");
    }
    let mut reg = Ev.lock();
    if let Some(sjg) = reg.bzl.el().du(|aa| aa.j == j) {
        sjg.iy = String::from(iy);
        Ok(())
    } else {
        Err("Effect not found")
    }
}


pub fn vuw(j: &str) -> Result<(), &'static str> {
    let mut reg = Ev.lock();
    let w = reg.bzl.iter().qf(|aa| aa.j == j)
        .ok_or("Effect not found")?;
    reg.bzl.remove(w);
    
    match reg.dnx {
        Some(a) if a == w => {
            reg.dnx = if reg.bzl.is_empty() { None } else { Some(0) };
            if reg.dnx.is_none() {
                IF_.store(false, Ordering::Relaxed);
            }
        }
        Some(a) if a > w => reg.dnx = Some(a - 1),
        _ => {}
    }
    Ok(())
}


pub fn phq(j: &str) -> Result<(), &'static str> {
    let mut reg = Ev.lock();
    let w = reg.bzl.iter().qf(|aa| aa.j == j)
        .ok_or("Effect not found")?;
    reg.dnx = Some(w);
    IF_.store(true, Ordering::Relaxed);
    Ok(())
}


pub fn cwz() {
    IF_.store(false, Ordering::Relaxed);
}


pub fn aiy() -> Result<(), &'static str> {
    let reg = Ev.lock();
    if reg.dnx.is_some() {
        IF_.store(true, Ordering::Relaxed);
        Ok(())
    } else {
        Err("No effect selected")
    }
}


pub fn ufq() -> Vec<(String, bool)> {
    let reg = Ev.lock();
    reg.bzl.iter().cf().map(|(a, aa)| {
        (aa.j.clone(), reg.dnx == Some(a))
    }).collect()
}


pub fn teq(j: &str) -> Option<String> {
    let reg = Ev.lock();
    reg.bzl.iter().du(|aa| aa.j == j).map(|aa| aa.iy.clone())
}








pub fn wbh() -> bool {
    if !IF_.load(Ordering::Relaxed) {
        return false;
    }
    
    let iy = {
        let reg = Ev.lock();
        match reg.dnx {
            Some(w) => reg.bzl.get(w).map(|aa| aa.iy.clone()),
            None => None,
        }
    };

    if let Some(cy) = iy {
        
        
        match crate::trustlang::vw(&cy) {
            Ok(_) => {}
            Err(aa) => {
                let frame = ACJ_.load(Ordering::Relaxed);
                
                if frame % 60 == 0 {
                    crate::serial_println!("[VIZFX] Script error (frame {}): {}", frame, aa);
                }
            }
        }
        true
    } else {
        false
    }
}






pub fn ugs() -> Result<(), &'static str> {
    let iy = r#"fn main() {
    let w = screen_w();
    let h = screen_h();
    let cx = w / 2;
    let cy = h / 2;
    let b = to_int(beat() * 255.0);
    let e = to_int(energy() * 200.0);
    let ba = to_int(bass() * 255.0);
    let t = frame_num();

    let i = 0;
    while i < 8 {
        let base_r = 30 + i * 40;
        let pulse = to_int(to_float(base_r) + beat() * 50.0);
        let r = to_int(to_float(ba) * 0.3);
        let g = to_int(to_float(b) * 0.8 + to_float(i * 20));
        let blue = to_int(to_float(e) * 0.5);
        draw_circle(cx, cy, pulse, r, g, blue);
        i = i + 1;
    }
}"#;
    iix("pulse-rings", iy)
}


pub fn ugt() -> Result<(), &'static str> {
    let iy = r#"fn main() {
    let w = screen_w();
    let h = screen_h();
    let bar_w = w / 8;
    let t = frame_num();

    let sb = to_int(sub_bass() * to_float(h));
    let ba = to_int(bass() * to_float(h));
    let lm = to_int(mid() * to_float(h) * 0.8);
    let mi = to_int(mid() * to_float(h) * 0.6);
    let hm = to_int(high_mid() * to_float(h) * 0.7);
    let tr = to_int(treble() * to_float(h) * 0.5);

    fill_rect(0 * bar_w, h - sb, bar_w - 2, sb, 255, 0, 50);
    fill_rect(1 * bar_w, h - ba, bar_w - 2, ba, 255, 50, 0);
    fill_rect(2 * bar_w, h - lm, bar_w - 2, lm, 200, 200, 0);
    fill_rect(3 * bar_w, h - mi, bar_w - 2, mi, 0, 255, 100);
    fill_rect(4 * bar_w, h - hm, bar_w - 2, hm, 0, 150, 255);
    fill_rect(5 * bar_w, h - tr, bar_w - 2, tr, 150, 0, 255);
}"#;
    iix("spectrum-bars", iy)
}


pub fn ugr() -> Result<(), &'static str> {
    let iy = r#"fn main() {
    let w = screen_w();
    let h = screen_h();
    let b = beat();
    if b > 0.3 {
        let intensity = to_int(b * 255.0);
        let thickness = to_int(b * 20.0) + 2;
        fill_rect(0, 0, w, thickness, 0, intensity, intensity / 3);
        fill_rect(0, h - thickness, w, thickness, 0, intensity, intensity / 3);
        fill_rect(0, 0, thickness, h, 0, intensity, intensity / 3);
        fill_rect(w - thickness, 0, thickness, h, 0, intensity, intensity / 3);
    }
}"#;
    iix("beat-flash", iy)
}
