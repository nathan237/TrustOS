



















pub mod track;
pub mod mixer;
pub mod piano_roll;
pub mod keyboard_midi;
pub mod recorder;
pub mod wav_export;
pub mod ui;
pub mod beat_studio;
pub mod audio_viz;
pub mod disk_audio;
pub mod live_viz;

use alloc::string::String;
use alloc::format;
use spin::Mutex;
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use track::{Track, Note, Project};
use mixer::Mixer;






pub static Fc: Mutex<Option<Project>> = Mutex::new(None);

static Lu: Mutex<Option<Mixer>> = Mutex::new(None);

static Be: AtomicBool = AtomicBool::new(false);

pub static Qi: AtomicBool = AtomicBool::new(false);

pub static Mf: AtomicBool = AtomicBool::new(false);

pub static FR_: AtomicU32 = AtomicU32::new(0);

pub static Hi: AtomicU32 = AtomicU32::new(120);


pub const AE_: u32 = 480;

pub const BR_: u32 = 48000;

pub const FM_: usize = 16;






pub fn init() -> Result<(), &'static str> {
    if Be.load(Ordering::Relaxed) {
        return Ok(());
    }

    
    crate::audio::init().jd(|_| "Failed to init audio subsystem")?;

    
    let nv = Project::new("Untitled", 120);
    *Fc.lock() = Some(nv);

    
    let mixer = Mixer::new(FM_);
    *Lu.lock() = Some(mixer);

    Hi.store(120, Ordering::Relaxed);
    Be.store(true, Ordering::Relaxed);

    crate::serial_println!("[TRUSTDAW] TrustDAW initialized — {} tracks max, {} ticks/quarter",
        FM_, AE_);
    Ok(())
}


pub fn aqz() -> Result<(), &'static str> {
    if !Be.load(Ordering::Relaxed) {
        init()?;
    }
    Ok(())
}






pub fn daq() -> Result<(), &'static str> {
    aqz()?;
    if Qi.load(Ordering::Relaxed) {
        return Ok(()); 
    }

    let nv = Fc.lock();
    let nv = nv.as_ref().ok_or("No project loaded")?;
    let uos = Lu.lock();
    let mixer = uos.as_ref().ok_or("Mixer not initialized")?;

    
    let kz = Hi.load(Ordering::Relaxed);
    let vb = FR_.load(Ordering::Relaxed);
    let un = mixer::pcb(nv, mixer, kz, vb);

    if un.is_empty() {
        return Err("No audio to play (add notes to tracks first)");
    }

    Qi.store(true, Ordering::Relaxed);

    
    let uk = (un.len() as u32 / 2) * 1000 / BR_;

    
    let result = crate::drivers::hda::ele(&un, uk);

    Qi.store(false, Ordering::Relaxed);
    result
}


pub fn qg() {
    Qi.store(false, Ordering::Relaxed);
    Mf.store(false, Ordering::Relaxed);
    let _ = crate::drivers::hda::qg();
}


pub fn lzz() {
    FR_.store(0, Ordering::Relaxed);
}


pub fn mef(kz: u32) {
    let kz = kz.qp(30, 300);
    Hi.store(kz, Ordering::Relaxed);
    if let Some(nv) = Fc.lock().as_mut() {
        nv.kz = kz as u16;
    }
}






pub fn jzi(j: &str) -> Result<usize, &'static str> {
    aqz()?;
    let mut nv = Fc.lock();
    let nv = nv.as_mut().ok_or("No project")?;
    nv.jzi(j)
}


pub fn lza(index: usize) -> Result<(), &'static str> {
    aqz()?;
    let mut nv = Fc.lock();
    let nv = nv.as_mut().ok_or("No project")?;
    nv.lza(index)
}


pub fn axn(zx: usize, jp: u8, qm: u8, vb: u32, bbn: u32) -> Result<(), &'static str> {
    aqz()?;
    let mut nv = Fc.lock();
    let nv = nv.as_mut().ok_or("No project")?;
    let track = nv.af.ds(zx).ok_or("Invalid track index")?;
    track.axn(Note::new(jp, qm, vb, bbn));
    Ok(())
}


pub fn wjw(zx: usize, ve: &str) -> Result<(), &'static str> {
    aqz()?;
    let azd = crate::audio::synth::Waveform::cko(ve)
        .ok_or("Unknown waveform (sine/square/saw/triangle/noise)")?;
    let mut nv = Fc.lock();
    let nv = nv.as_mut().ok_or("No project")?;
    let track = nv.af.ds(zx).ok_or("Invalid track index")?;
    track.ve = azd;
    Ok(())
}


pub fn wjv(zx: usize, hq: u8) -> Result<(), &'static str> {
    aqz()?;
    let mut mixer = Lu.lock();
    let mixer = mixer.as_mut().ok_or("Mixer not initialized")?;
    mixer.chv(zx, hq)
}


pub fn wju(zx: usize, arp: i8) -> Result<(), &'static str> {
    aqz()?;
    let mut mixer = Lu.lock();
    let mixer = mixer.as_mut().ok_or("Mixer not initialized")?;
    mixer.meq(zx, arp)
}


pub fn mlo(zx: usize) -> Result<bool, &'static str> {
    aqz()?;
    let mut mixer = Lu.lock();
    let mixer = mixer.as_mut().ok_or("Mixer not initialized")?;
    mixer.mlo(zx)
}


pub fn mlr(zx: usize) -> Result<bool, &'static str> {
    aqz()?;
    let mut mixer = Lu.lock();
    let mixer = mixer.as_mut().ok_or("Mixer not initialized")?;
    mixer.mlr(zx)
}






pub fn status() -> String {
    let nv = Fc.lock();
    let mixer = Lu.lock();
    let kz = Hi.load(Ordering::Relaxed);
    let u = FR_.load(Ordering::Relaxed);
    let uu = Qi.load(Ordering::Relaxed);
    let ehe = Mf.load(Ordering::Relaxed);

    let mut e = String::new();
    e.t("╔══════════════════════════════════════════════╗\n");
    e.t("║          TrustDAW — Digital Audio Workstation║\n");
    e.t("╠══════════════════════════════════════════════╣\n");

    if let Some(aci) = nv.as_ref() {
        e.t(&format!("║ Project: {:<36}║\n", aci.amj()));
        e.t(&format!("║ BPM: {:<3}  Tracks: {}/{:<22}║\n",
            kz, aci.af.len(), FM_));
        let bar = u / (AE_ * 4);
        let rf = (u % (AE_ * 4)) / AE_;
        let or = u % AE_;
        let g = if ehe { "REC" } else if uu { "PLAY" } else { "STOP" };
        e.t(&format!("║ Position: {}:{:02}:{:03}  State: {:<14}║\n",
            bar + 1, rf + 1, or, g));
        e.t("╠══════════════════════════════════════════════╣\n");

        if aci.af.is_empty() {
            e.t("║ No tracks. Use 'daw track add <name>'       ║\n");
        } else {
            e.t("║ # │ Name         │ Notes │ Wave  │ Vol │ Pan ║\n");
            e.t("║───┼──────────────┼───────┼───────┼─────┼─────║\n");
            for (a, track) in aci.af.iter().cf() {
                let api = if let Some(ef) = mixer.as_ref() {
                    ef.lq.get(a).map(|r| r.hq).unwrap_or(200)
                } else { 200 };
                let arp = if let Some(ef) = mixer.as_ref() {
                    ef.lq.get(a).map(|r| r.arp).unwrap_or(0)
                } else { 0 };
                let uqv = if let Some(ef) = mixer.as_ref() {
                    if ef.lq.get(a).map(|r| r.so).unwrap_or(false) { "M" } else { " " }
                } else { " " };
                e.t(&format!("║{}{:2}│ {:<12} │ {:>5} │ {:<5} │ {:>3} │ {:>+3} ║\n",
                    uqv, a, track.amj(), track.ts.len(),
                    track.ve.dbz(), api, arp));
            }
        }
    } else {
        e.t("║ No project loaded                            ║\n");
    }

    e.t("╚══════════════════════════════════════════════╝\n");
    e
}


pub fn uft(zx: usize) -> Result<String, &'static str> {
    aqz()?;
    let nv = Fc.lock();
    let nv = nv.as_ref().ok_or("No project")?;
    let track = nv.af.get(zx).ok_or("Invalid track index")?;

    let mut e = String::new();
    e.t(&format!("Track {}: \"{}\" — {} notes, {}\n",
        zx, track.amj(), track.ts.len(), track.ve.j()));

    if track.ts.is_empty() {
        e.t("  (no notes)\n");
    } else {
        e.t("  # │ Note │ Vel │ Start(tick) │ Dur(tick) │ Bar:Beat\n");
        e.t("  ──┼──────┼─────┼─────────────┼───────────┼─────────\n");
        for (a, jp) in track.ts.iter().cf().take(64) {
            let j = crate::audio::tables::dtf(jp.jb);
            let bvq = crate::audio::tables::efk(jp.jb);
            let bar = jp.vb / (AE_ * 4);
            let rf = (jp.vb % (AE_ * 4)) / AE_;
            e.t(&format!("  {:2}│ {}{:<2}  │ {:>3} │ {:>11} │ {:>9} │ {}:{}\n",
                a, j, bvq, jp.qm, jp.vb, jp.bbn,
                bar + 1, rf + 1));
        }
        if track.ts.len() > 64 {
            e.t(&format!("  ... and {} more notes\n", track.ts.len() - 64));
        }
    }
    Ok(e)
}






pub fn ugp() -> Result<(), &'static str> {
    aqz()?;

    let mut nv = Project::new("Demo Song", 120);

    
    let mut czq = Track::new("Melody");
    czq.ve = crate::audio::synth::Waveform::Dg;
    
    czq.axn(Note::new(60, 100, 0, 480));       
    czq.axn(Note::new(64, 100, 480, 480));     
    czq.axn(Note::new(67, 100, 960, 480));     
    czq.axn(Note::new(72, 100, 1440, 480));    
    czq.axn(Note::new(72, 90, 1920, 480));     
    czq.axn(Note::new(67, 90, 2400, 480));     
    czq.axn(Note::new(64, 90, 2880, 480));     
    czq.axn(Note::new(60, 90, 3360, 480));     
    nv.af.push(czq);

    
    let mut aee = Track::new("Bass");
    aee.ve = crate::audio::synth::Waveform::Ft;
    
    aee.axn(Note::new(36, 110, 0, 960));        
    aee.axn(Note::new(36, 110, 960, 960));      
    aee.axn(Note::new(36, 110, 1920, 960));     
    aee.axn(Note::new(43, 110, 2880, 960));     
    nv.af.push(aee);

    
    let mut irz = Track::new("Drums");
    irz.ve = crate::audio::synth::Waveform::Cr;
    
    for rf in 0..8 {
        irz.axn(Note::new(36, 127, rf * 480, 120));
    }
    
    for rf in [1, 3, 5, 7] {
        irz.axn(Note::new(60, 100, rf * 480 + 240, 120));
    }
    nv.af.push(irz);

    
    let mut enm = Track::new("Chords");
    enm.ve = crate::audio::synth::Waveform::Triangle;
    
    enm.axn(Note::new(60, 70, 0, 1920));      
    enm.axn(Note::new(64, 70, 0, 1920));      
    enm.axn(Note::new(67, 70, 0, 1920));      
    
    enm.axn(Note::new(55, 70, 1920, 1920));   
    enm.axn(Note::new(59, 70, 1920, 1920));   
    enm.axn(Note::new(62, 70, 1920, 1920));   
    nv.af.push(enm);

    *Fc.lock() = Some(nv);

    
    if let Some(mixer) = Lu.lock().as_mut() {
        mixer.chv(0, 200).bq();  
        mixer.chv(1, 160).bq();  
        mixer.chv(2, 140).bq();  
        mixer.chv(3, 120).bq();  
        mixer.meq(1, -40).bq();     
        mixer.meq(3, 40).bq();      
    }

    crate::serial_println!("[TRUSTDAW] Demo project loaded: 4 tracks, 2 bars");
    Ok(())
}


pub fn utn(j: &str, kz: u32) -> Result<(), &'static str> {
    aqz()?;
    let nv = Project::new(j, kz.qp(30, 300) as u16);
    *Fc.lock() = Some(nv);
    Hi.store(kz.qp(30, 300), Ordering::Relaxed);
    FR_.store(0, Ordering::Relaxed);
    Ok(())
}


pub fn hio(path: &str) -> Result<usize, &'static str> {
    aqz()?;
    let nv = Fc.lock();
    let nv = nv.as_ref().ok_or("No project")?;
    let mixer = Lu.lock();
    let mixer = mixer.as_ref().ok_or("Mixer not initialized")?;
    let kz = Hi.load(Ordering::Relaxed);

    let un = mixer::pcb(nv, mixer, kz, 0);
    if un.is_empty() {
        return Err("No audio to export");
    }

    wav_export::hio(path, &un, BR_, 2)
}
