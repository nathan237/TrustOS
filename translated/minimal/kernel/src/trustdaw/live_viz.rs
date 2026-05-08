






















use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use core::sync::atomic::{AtomicU32, AtomicBool, Ordering};
use spin::Mutex;





const CIQ_: usize = 8;

struct Aaw {
    name: String,
    source: String,
}

struct EffectRegistry {
    effects: Vec<Aaw>,
    active_idx: Option<usize>,
}

impl EffectRegistry {
    const fn new() -> Self {
        Self {
            effects: Vec::new(),
            active_idx: None,
        }
    }
}

static Ca: Mutex<EffectRegistry> = Mutex::new(EffectRegistry::new());



static AMV_: AtomicU32 = AtomicU32::new(0);
static AMU_: AtomicU32 = AtomicU32::new(0);
static AMZ_: AtomicU32 = AtomicU32::new(0);
static AMY_: AtomicU32 = AtomicU32::new(0);
static AMX_: AtomicU32 = AtomicU32::new(0);
static ANA_: AtomicU32 = AtomicU32::new(0);
static AMW_: AtomicU32 = AtomicU32::new(0);
static ADZ_: AtomicU32 = AtomicU32::new(0);
static IZ_: AtomicBool = AtomicBool::new(false);






pub fn oon(beat: f32, bass: f32, sub_bass: f32, mid: f32,
                      high_mid: f32, treble: f32, energy: f32, frame: u32) {
    AMV_.store(beat.to_bits(), Ordering::Relaxed);
    AMU_.store(bass.to_bits(), Ordering::Relaxed);
    AMZ_.store(sub_bass.to_bits(), Ordering::Relaxed);
    AMY_.store(mid.to_bits(), Ordering::Relaxed);
    AMX_.store(high_mid.to_bits(), Ordering::Relaxed);
    ANA_.store(treble.to_bits(), Ordering::Relaxed);
    AMW_.store(energy.to_bits(), Ordering::Relaxed);
    ADZ_.store(frame, Ordering::Relaxed);
}


pub fn mcr() -> f32 { f32::from_bits(AMV_.load(Ordering::Relaxed)) }
pub fn mcq() -> f32 { f32::from_bits(AMU_.load(Ordering::Relaxed)) }
pub fn mdw() -> f32 { f32::from_bits(AMZ_.load(Ordering::Relaxed)) }
pub fn mdm() -> f32 { f32::from_bits(AMY_.load(Ordering::Relaxed)) }
pub fn mde() -> f32 { f32::from_bits(AMX_.load(Ordering::Relaxed)) }
pub fn mea() -> f32 { f32::from_bits(ANA_.load(Ordering::Relaxed)) }
pub fn mdb() -> f32 { f32::from_bits(AMW_.load(Ordering::Relaxed)) }
pub fn mdd() -> u32 { ADZ_.load(Ordering::Relaxed) }


pub fn is_active() -> bool {
    IZ_.load(Ordering::Relaxed)
}






pub fn eez(name: &str, source: &str) -> Result<(), &'static str> {
    let mut reg = Ca.lock();
    if reg.effects.len() >= CIQ_ {
        return Err("Maximum 8 effects reached — remove one first");
    }
    if reg.effects.iter().any(|e| e.name == name) {
        return Err("Effect with this name already exists — use 'vizfx edit' to update");
    }
    
    if let Err(e) = crate::trustlang::cgv(source) {
        crate::serial_println!("[VIZFX] Syntax error: {}", e);
        return Err("TrustLang syntax error — check your script");
    }
    reg.effects.push(Aaw {
        name: String::from(name),
        source: String::from(source),
    });
    
    if reg.effects.len() == 1 {
        reg.active_idx = Some(0);
        IZ_.store(true, Ordering::Relaxed);
    }
    Ok(())
}


pub fn loh(name: &str, source: &str) -> Result<(), &'static str> {
    
    if let Err(e) = crate::trustlang::cgv(source) {
        crate::serial_println!("[VIZFX] Syntax error: {}", e);
        return Err("TrustLang syntax error — check your script");
    }
    let mut reg = Ca.lock();
    if let Some(eff) = reg.effects.iter_mut().find(|e| e.name == name) {
        eff.source = String::from(source);
        Ok(())
    } else {
        Err("Effect not found")
    }
}


pub fn oey(name: &str) -> Result<(), &'static str> {
    let mut reg = Ca.lock();
    let idx = reg.effects.iter().position(|e| e.name == name)
        .ok_or("Effect not found")?;
    reg.effects.remove(idx);
    
    match reg.active_idx {
        Some(i) if i == idx => {
            reg.active_idx = if reg.effects.is_empty() { None } else { Some(0) };
            if reg.active_idx.is_none() {
                IZ_.store(false, Ordering::Relaxed);
            }
        }
        Some(i) if i > idx => reg.active_idx = Some(i - 1),
        _ => {}
    }
    Ok(())
}


pub fn jej(name: &str) -> Result<(), &'static str> {
    let mut reg = Ca.lock();
    let idx = reg.effects.iter().position(|e| e.name == name)
        .ok_or("Effect not found")?;
    reg.active_idx = Some(idx);
    IZ_.store(true, Ordering::Relaxed);
    Ok(())
}


pub fn bbc() {
    IZ_.store(false, Ordering::Relaxed);
}


pub fn enable() -> Result<(), &'static str> {
    let reg = Ca.lock();
    if reg.active_idx.is_some() {
        IZ_.store(true, Ordering::Relaxed);
        Ok(())
    } else {
        Err("No effect selected")
    }
}


pub fn mzc() -> Vec<(String, bool)> {
    let reg = Ca.lock();
    reg.effects.iter().enumerate().map(|(i, e)| {
        (e.name.clone(), reg.active_idx == Some(i))
    }).collect()
}


pub fn mdu(name: &str) -> Option<String> {
    let reg = Ca.lock();
    reg.effects.iter().find(|e| e.name == name).map(|e| e.source.clone())
}








pub fn ojc() -> bool {
    if !IZ_.load(Ordering::Relaxed) {
        return false;
    }
    
    let source = {
        let reg = Ca.lock();
        match reg.active_idx {
            Some(idx) => reg.effects.get(idx).map(|e| e.source.clone()),
            None => None,
        }
    };

    if let Some(src) = source {
        
        
        match crate::trustlang::run(&src) {
            Ok(_) => {}
            Err(e) => {
                let frame = ADZ_.load(Ordering::Relaxed);
                
                if frame % 60 == 0 {
                    crate::serial_println!("[VIZFX] Script error (frame {}): {}", frame, e);
                }
            }
        }
        true
    } else {
        false
    }
}






pub fn mzy() -> Result<(), &'static str> {
    let source = r#"fn main() {
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
    eez("pulse-rings", source)
}


pub fn mzz() -> Result<(), &'static str> {
    let source = r#"fn main() {
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
    eez("spectrum-bars", source)
}


pub fn mzx() -> Result<(), &'static str> {
    let source = r#"fn main() {
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
    eez("beat-flash", source)
}
