









pub mod tables;
pub mod synth;
pub mod pattern;
pub mod player;

use spin::Mutex;
use alloc::string::String;
use alloc::format;

use synth::{SynthEngine, Waveform, Envelope};
use pattern::{Pattern, PatternBank};
use player::PatternPlayer;


static Ea: Mutex<Option<SynthEngine>> = Mutex::new(None);

static Ei: Mutex<Option<PatternBank>> = Mutex::new(None);

static Mu: Mutex<Option<PatternPlayer>> = Mutex::new(None);


pub fn init() -> Result<(), &'static str> {
    
    if !crate::drivers::hda::is_initialized() {
        crate::drivers::hda::init()?;
    }

    
    let engine = SynthEngine::new();
    *Ea.lock() = Some(engine);

    
    let mut gi = PatternBank::new();
    gi.load_presets();
    *Ei.lock() = Some(gi);

    
    *Mu.lock() = Some(PatternPlayer::new());

    crate::serial_println!("[AUDIO] TrustSynth engine + pattern bank initialized");
    Ok(())
}


fn ensure_init() -> Result<(), &'static str> {
    if Ea.lock().is_none() {
        init()?;
    }
    Ok(())
}


pub fn ivf(name: &str, duration_ms: u32) -> Result<(), &'static str> {
    ensure_init()?;

    let jo = {
        let mut synth = Ea.lock();
        let engine = synth.as_mut().ok_or("Synth not initialized")?;
        engine.play_note_by_name(name, duration_ms)?
    };

    
    gnh(&jo, duration_ms)?;
    Ok(())
}


pub fn ive(note: u8, velocity: u8, duration_ms: u32) -> Result<(), &'static str> {
    ensure_init()?;

    let jo = {
        let mut synth = Ea.lock();
        let engine = synth.as_mut().ok_or("Synth not initialized")?;
        engine.render_note(note, velocity, duration_ms)
    };

    gnh(&jo, duration_ms)?;
    Ok(())
}


pub fn nvj(freq_hz: u32, duration_ms: u32) -> Result<(), &'static str> {
    ensure_init()?;

    let jo = {
        let mut synth = Ea.lock();
        let engine = synth.as_mut().ok_or("Synth not initialized")?;
        engine.render_freq(freq_hz, duration_ms)
    };

    gnh(&jo, duration_ms)?;
    Ok(())
}


pub fn set_waveform(aal: Waveform) -> Result<(), &'static str> {
    ensure_init()?;
    let mut synth = Ea.lock();
    let engine = synth.as_mut().ok_or("Synth not initialized")?;
    engine.set_waveform(aal);
    Ok(())
}


pub fn set_adsr(attack_ms: u32, decay_ms: u32, sustain_pct: u32, release_ms: u32) -> Result<(), &'static str> {
    ensure_init()?;
    let mut synth = Ea.lock();
    let engine = synth.as_mut().ok_or("Synth not initialized")?;
    engine.set_adsr(attack_ms, decay_ms, sustain_pct, release_ms);
    Ok(())
}


pub fn oow(name: &str) -> Result<(), &'static str> {
    ensure_init()?;
    let env = match name {
        "default" => Envelope::eka(),
        "organ" => Envelope::nnt(),
        "pluck" => Envelope::dwp(),
        "pad" => Envelope::pad(),
        _ => return Err("Unknown preset (use: default, organ, pluck, pad)"),
    };
    let mut synth = Ea.lock();
    let engine = synth.as_mut().ok_or("Synth not initialized")?;
    engine.envelope = env;
    Ok(())
}


pub fn set_volume(vd: u8) -> Result<(), &'static str> {
    ensure_init()?;
    let mut synth = Ea.lock();
    let engine = synth.as_mut().ok_or("Synth not initialized")?;
    engine.master_volume = vd;
    Ok(())
}


pub fn status() -> String {
    let synth = Ea.lock();
    match synth.as_ref() {
        Some(engine) => engine.status(),
        None => String::from("TrustSynth: not initialized\n"),
    }
}


pub fn stop() -> Result<(), &'static str> {
    {
        let mut synth = Ea.lock();
        if let Some(engine) = synth.as_mut() {
            engine.all_notes_off();
        }
    }
    crate::drivers::hda::stop()
}






fn gnh(jo: &[i16], duration_ms: u32) -> Result<(), &'static str> {
    
    crate::drivers::hda::bxb(jo, duration_ms)
}






fn cix() -> Result<(), &'static str> {
    ensure_init()?;
    if Ei.lock().is_none() {
        let mut gi = PatternBank::new();
        gi.load_presets();
        *Ei.lock() = Some(gi);
    }
    if Mu.lock().is_none() {
        *Mu.lock() = Some(PatternPlayer::new());
    }
    Ok(())
}


pub fn nsc(name: &str, steps: usize, bpm: u16) -> Result<(), &'static str> {
    cix()?;
    let pattern = Pattern::new(name, steps, bpm);
    let mut gi = Ei.lock();
    let gi = gi.as_mut().ok_or("Pattern bank not initialized")?;
    gi.add(pattern)?;
    Ok(())
}


pub fn nsg(name: &str, step: usize, agu: &str) -> Result<(), &'static str> {
    cix()?;
    let mut gi = Ei.lock();
    let gi = gi.as_mut().ok_or("Pattern bank not initialized")?;
    let pat = gi.get_by_name_mut(name).ok_or("Pattern not found")?;
    pat.set_note(step, agu)
}


pub fn nsf(name: &str, bpm: u16) -> Result<(), &'static str> {
    cix()?;
    let mut gi = Ei.lock();
    let gi = gi.as_mut().ok_or("Pattern bank not initialized")?;
    let pat = gi.get_by_name_mut(name).ok_or("Pattern not found")?;
    pat.bpm = bpm;
    Ok(())
}


pub fn nsh(name: &str, aal: Waveform) -> Result<(), &'static str> {
    cix()?;
    let mut gi = Ei.lock();
    let gi = gi.as_mut().ok_or("Pattern bank not initialized")?;
    let pat = gi.get_by_name_mut(name).ok_or("Pattern not found")?;
    pat.waveform = aal;
    Ok(())
}


pub fn nsi(name: &str) -> Result<String, &'static str> {
    cix()?;
    let gi = Ei.lock();
    let gi = gi.as_ref().ok_or("Pattern bank not initialized")?;
    let pat = gi.get_by_name(name).ok_or("Pattern not found")?;
    Ok(pat.display())
}


pub fn nsb() -> String {
    let gi = Ei.lock();
    match gi.as_ref() {
        Some(b) => b.list(),
        None => String::from("Pattern bank not initialized\n"),
    }
}


pub fn nse(name: &str) -> Result<(), &'static str> {
    cix()?;
    let mut gi = Ei.lock();
    let gi = gi.as_mut().ok_or("Pattern bank not initialized")?;
    gi.remove(name)
}


pub fn nsd(name: &str, loops: u32) -> Result<(), &'static str> {
    cix()?;

    
    let pattern = {
        let gi = Ei.lock();
        let gi = gi.as_ref().ok_or("Pattern bank not initialized")?;
        gi.get_by_name(name).ok_or("Pattern not found")?.clone()
    };

    
    let mut ozo = Ea.lock();
    let engine = ozo.as_mut().ok_or("Synth not initialized")?;
    let mut gnj = Mu.lock();
    let player = gnj.as_mut().ok_or("Player not initialized")?;

    player.play_pattern_visual(&pattern, engine, loops)
}


pub fn nsj() -> Result<(), &'static str> {
    let mut gnj = Mu.lock();
    if let Some(player) = gnj.as_mut() {
        player.stop();
    }
    Ok(())
}
