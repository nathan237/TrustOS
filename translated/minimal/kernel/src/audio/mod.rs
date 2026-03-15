









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


static Ka: Mutex<Option<SynthEngine>> = Mutex::new(None);

static Kt: Mutex<Option<PatternBank>> = Mutex::new(None);

static Adp: Mutex<Option<PatternPlayer>> = Mutex::new(None);


pub fn init() -> Result<(), &'static str> {
    
    if !crate::drivers::hda::ky() {
        crate::drivers::hda::init()?;
    }

    
    let engine = SynthEngine::new();
    *Ka.lock() = Some(engine);

    
    let mut om = PatternBank::new();
    om.ojw();
    *Kt.lock() = Some(om);

    
    *Adp.lock() = Some(PatternPlayer::new());

    crate::serial_println!("[AUDIO] TrustSynth engine + pattern bank initialized");
    Ok(())
}


fn aqz() -> Result<(), &'static str> {
    if Ka.lock().is_none() {
        init()?;
    }
    Ok(())
}


pub fn owc(j: &str, uk: u32) -> Result<(), &'static str> {
    aqz()?;

    let un = {
        let mut synth = Ka.lock();
        let engine = synth.as_mut().ok_or("Synth not initialized")?;
        engine.viz(j, uk)?
    };

    
    lug(&un, uk)?;
    Ok(())
}


pub fn owb(jp: u8, qm: u8, uk: u32) -> Result<(), &'static str> {
    aqz()?;

    let un = {
        let mut synth = Ka.lock();
        let engine = synth.as_mut().ok_or("Synth not initialized")?;
        engine.lzf(jp, qm, uk)
    };

    lug(&un, uk)?;
    Ok(())
}


pub fn viw(auf: u32, uk: u32) -> Result<(), &'static str> {
    aqz()?;

    let un = {
        let mut synth = Ka.lock();
        let engine = synth.as_mut().ok_or("Synth not initialized")?;
        engine.vvv(auf, uk)
    };

    lug(&un, uk)?;
    Ok(())
}


pub fn dvs(azd: Waveform) -> Result<(), &'static str> {
    aqz()?;
    let mut synth = Ka.lock();
    let engine = synth.as_mut().ok_or("Synth not initialized")?;
    engine.dvs(azd);
    Ok(())
}


pub fn med(gzc: u32, hfm: u32, icg: u32, hxk: u32) -> Result<(), &'static str> {
    aqz()?;
    let mut synth = Ka.lock();
    let engine = synth.as_mut().ok_or("Synth not initialized")?;
    engine.med(gzc, hfm, icg, hxk);
    Ok(())
}


pub fn wiu(j: &str) -> Result<(), &'static str> {
    aqz()?;
    let env = match j {
        "default" => Envelope::iqt(),
        "organ" => Envelope::uza(),
        "pluck" => Envelope::hvi(),
        "pad" => Envelope::ov(),
        _ => return Err("Unknown preset (use: default, organ, pluck, pad)"),
    };
    let mut synth = Ka.lock();
    let engine = synth.as_mut().ok_or("Synth not initialized")?;
    engine.qr = env;
    Ok(())
}


pub fn chv(api: u8) -> Result<(), &'static str> {
    aqz()?;
    let mut synth = Ka.lock();
    let engine = synth.as_mut().ok_or("Synth not initialized")?;
    engine.euo = api;
    Ok(())
}


pub fn status() -> String {
    let synth = Ka.lock();
    match synth.as_ref() {
        Some(engine) => engine.status(),
        None => String::from("TrustSynth: not initialized\n"),
    }
}


pub fn qg() -> Result<(), &'static str> {
    {
        let mut synth = Ka.lock();
        if let Some(engine) = synth.as_mut() {
            engine.qgm();
        }
    }
    crate::drivers::hda::qg()
}






fn lug(un: &[i16], uk: u32) -> Result<(), &'static str> {
    
    crate::drivers::hda::ele(un, uk)
}






fn fht() -> Result<(), &'static str> {
    aqz()?;
    if Kt.lock().is_none() {
        let mut om = PatternBank::new();
        om.ojw();
        *Kt.lock() = Some(om);
    }
    if Adp.lock().is_none() {
        *Adp.lock() = Some(PatternPlayer::new());
    }
    Ok(())
}


pub fn vfd(j: &str, au: usize, kz: u16) -> Result<(), &'static str> {
    fht()?;
    let pattern = Pattern::new(j, au, kz);
    let mut om = Kt.lock();
    let om = om.as_mut().ok_or("Pattern bank not initialized")?;
    om.add(pattern)?;
    Ok(())
}


pub fn vfh(j: &str, gu: usize, bkp: &str) -> Result<(), &'static str> {
    fht()?;
    let mut om = Kt.lock();
    let om = om.as_mut().ok_or("Pattern bank not initialized")?;
    let pat = om.kyj(j).ok_or("Pattern not found")?;
    pat.wjh(gu, bkp)
}


pub fn vfg(j: &str, kz: u16) -> Result<(), &'static str> {
    fht()?;
    let mut om = Kt.lock();
    let om = om.as_mut().ok_or("Pattern bank not initialized")?;
    let pat = om.kyj(j).ok_or("Pattern not found")?;
    pat.kz = kz;
    Ok(())
}


pub fn vfi(j: &str, azd: Waveform) -> Result<(), &'static str> {
    fht()?;
    let mut om = Kt.lock();
    let om = om.as_mut().ok_or("Pattern bank not initialized")?;
    let pat = om.kyj(j).ok_or("Pattern not found")?;
    pat.ve = azd;
    Ok(())
}


pub fn vfj(j: &str) -> Result<String, &'static str> {
    fht()?;
    let om = Kt.lock();
    let om = om.as_ref().ok_or("Pattern bank not initialized")?;
    let pat = om.nxt(j).ok_or("Pattern not found")?;
    Ok(pat.display())
}


pub fn vfc() -> String {
    let om = Kt.lock();
    match om.as_ref() {
        Some(o) => o.aoy(),
        None => String::from("Pattern bank not initialized\n"),
    }
}


pub fn vff(j: &str) -> Result<(), &'static str> {
    fht()?;
    let mut om = Kt.lock();
    let om = om.as_mut().ok_or("Pattern bank not initialized")?;
    om.remove(j)
}


pub fn vfe(j: &str, bkh: u32) -> Result<(), &'static str> {
    fht()?;

    
    let pattern = {
        let om = Kt.lock();
        let om = om.as_ref().ok_or("Pattern bank not initialized")?;
        om.nxt(j).ok_or("Pattern not found")?.clone()
    };

    
    let mut wxd = Ka.lock();
    let engine = wxd.as_mut().ok_or("Synth not initialized")?;
    let mut lui = Adp.lock();
    let player = lui.as_mut().ok_or("Player not initialized")?;

    player.vja(&pattern, engine, bkh)
}


pub fn vfk() -> Result<(), &'static str> {
    let mut lui = Adp.lock();
    if let Some(player) = lui.as_mut() {
        player.qg();
    }
    Ok(())
}
