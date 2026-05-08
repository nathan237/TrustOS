



















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






pub static Ce: Mutex<Option<Project>> = Mutex::new(None);

static Ex: Mutex<Option<Mixer>> = Mutex::new(None);

static Ah: AtomicBool = AtomicBool::new(false);

pub static Gw: AtomicBool = AtomicBool::new(false);

pub static Fb: AtomicBool = AtomicBool::new(false);

pub static GG_: AtomicU32 = AtomicU32::new(0);

pub static Df: AtomicU32 = AtomicU32::new(120);


pub const AF_: u32 = 480;

pub const BT_: u32 = 48000;

pub const GB_: usize = 16;






pub fn init() -> Result<(), &'static str> {
    if Ah.load(Ordering::Relaxed) {
        return Ok(());
    }

    
    crate::audio::init().map_err(|_| "Failed to init audio subsystem")?;

    
    let project = Project::new("Untitled", 120);
    *Ce.lock() = Some(project);

    
    let mixer = Mixer::new(GB_);
    *Ex.lock() = Some(mixer);

    Df.store(120, Ordering::Relaxed);
    Ah.store(true, Ordering::Relaxed);

    crate::serial_println!("[TRUSTDAW] TrustDAW initialized — {} tracks max, {} ticks/quarter",
        GB_, AF_);
    Ok(())
}


pub fn ensure_init() -> Result<(), &'static str> {
    if !Ah.load(Ordering::Relaxed) {
        init()?;
    }
    Ok(())
}






pub fn play() -> Result<(), &'static str> {
    ensure_init()?;
    if Gw.load(Ordering::Relaxed) {
        return Ok(()); 
    }

    let project = Ce.lock();
    let project = project.as_ref().ok_or("No project loaded")?;
    let nfm = Ex.lock();
    let mixer = nfm.as_ref().ok_or("Mixer not initialized")?;

    
    let bpm = Df.load(Ordering::Relaxed);
    let start_tick = GG_.load(Ordering::Relaxed);
    let jo = mixer::izw(project, mixer, bpm, start_tick);

    if jo.is_empty() {
        return Err("No audio to play (add notes to tracks first)");
    }

    Gw.store(true, Ordering::Relaxed);

    
    let duration_ms = (jo.len() as u32 / 2) * 1000 / BT_;

    
    let result = crate::drivers::hda::bxb(&jo, duration_ms);

    Gw.store(false, Ordering::Relaxed);
    result
}


pub fn stop() {
    Gw.store(false, Ordering::Relaxed);
    Fb.store(false, Ordering::Relaxed);
    let _ = crate::drivers::hda::stop();
}


pub fn rewind() {
    GG_.store(0, Ordering::Relaxed);
}


pub fn guf(bpm: u32) {
    let bpm = bpm.clamp(30, 300);
    Df.store(bpm, Ordering::Relaxed);
    if let Some(project) = Ce.lock().as_mut() {
        project.bpm = bpm as u16;
    }
}






pub fn add_track(name: &str) -> Result<usize, &'static str> {
    ensure_init()?;
    let mut project = Ce.lock();
    let project = project.as_mut().ok_or("No project")?;
    project.add_track(name)
}


pub fn remove_track(index: usize) -> Result<(), &'static str> {
    ensure_init()?;
    let mut project = Ce.lock();
    let project = project.as_mut().ok_or("No project")?;
    project.remove_track(index)
}


pub fn add_note(mp: usize, note: u8, velocity: u8, start_tick: u32, duration_ticks: u32) -> Result<(), &'static str> {
    ensure_init()?;
    let mut project = Ce.lock();
    let project = project.as_mut().ok_or("No project")?;
    let track = project.tracks.get_mut(mp).ok_or("Invalid track index")?;
    track.add_note(Note::new(note, velocity, start_tick, duration_ticks));
    Ok(())
}


pub fn opr(mp: usize, waveform: &str) -> Result<(), &'static str> {
    ensure_init()?;
    let aal = crate::audio::synth::Waveform::atv(waveform)
        .ok_or("Unknown waveform (sine/square/saw/triangle/noise)")?;
    let mut project = Ce.lock();
    let project = project.as_mut().ok_or("No project")?;
    let track = project.tracks.get_mut(mp).ok_or("Invalid track index")?;
    track.waveform = aal;
    Ok(())
}


pub fn opq(mp: usize, volume: u8) -> Result<(), &'static str> {
    ensure_init()?;
    let mut mixer = Ex.lock();
    let mixer = mixer.as_mut().ok_or("Mixer not initialized")?;
    mixer.set_volume(mp, volume)
}


pub fn opp(mp: usize, pan: i8) -> Result<(), &'static str> {
    ensure_init()?;
    let mut mixer = Ex.lock();
    let mixer = mixer.as_mut().ok_or("Mixer not initialized")?;
    mixer.set_pan(mp, pan)
}


pub fn toggle_mute(mp: usize) -> Result<bool, &'static str> {
    ensure_init()?;
    let mut mixer = Ex.lock();
    let mixer = mixer.as_mut().ok_or("Mixer not initialized")?;
    mixer.toggle_mute(mp)
}


pub fn toggle_solo(mp: usize) -> Result<bool, &'static str> {
    ensure_init()?;
    let mut mixer = Ex.lock();
    let mixer = mixer.as_mut().ok_or("Mixer not initialized")?;
    mixer.toggle_solo(mp)
}






pub fn status() -> String {
    let project = Ce.lock();
    let mixer = Ex.lock();
    let bpm = Df.load(Ordering::Relaxed);
    let pos = GG_.load(Ordering::Relaxed);
    let playing = Gw.load(Ordering::Relaxed);
    let recording = Fb.load(Ordering::Relaxed);

    let mut j = String::new();
    j.push_str("╔══════════════════════════════════════════════╗\n");
    j.push_str("║          TrustDAW — Digital Audio Workstation║\n");
    j.push_str("╠══════════════════════════════════════════════╣\n");

    if let Some(oa) = project.as_ref() {
        j.push_str(&format!("║ Project: {:<36}║\n", oa.name_str()));
        j.push_str(&format!("║ BPM: {:<3}  Tracks: {}/{:<22}║\n",
            bpm, oa.tracks.len(), GB_));
        let bar = pos / (AF_ * 4);
        let beat = (pos % (AF_ * 4)) / AF_;
        let tick = pos % AF_;
        let state = if recording { "REC" } else if playing { "PLAY" } else { "STOP" };
        j.push_str(&format!("║ Position: {}:{:02}:{:03}  State: {:<14}║\n",
            bar + 1, beat + 1, tick, state));
        j.push_str("╠══════════════════════════════════════════════╣\n");

        if oa.tracks.is_empty() {
            j.push_str("║ No tracks. Use 'daw track add <name>'       ║\n");
        } else {
            j.push_str("║ # │ Name         │ Notes │ Wave  │ Vol │ Pan ║\n");
            j.push_str("║───┼──────────────┼───────┼───────┼─────┼─────║\n");
            for (i, track) in oa.tracks.iter().enumerate() {
                let vd = if let Some(m) = mixer.as_ref() {
                    m.channels.get(i).map(|c| c.volume).unwrap_or(200)
                } else { 200 };
                let pan = if let Some(m) = mixer.as_ref() {
                    m.channels.get(i).map(|c| c.pan).unwrap_or(0)
                } else { 0 };
                let nhh = if let Some(m) = mixer.as_ref() {
                    if m.channels.get(i).map(|c| c.muted).unwrap_or(false) { "M" } else { " " }
                } else { " " };
                j.push_str(&format!("║{}{:2}│ {:<12} │ {:>5} │ {:<5} │ {:>3} │ {:>+3} ║\n",
                    nhh, i, track.name_str(), track.notes.len(),
                    track.waveform.short_name(), vd, pan));
            }
        }
    } else {
        j.push_str("║ No project loaded                            ║\n");
    }

    j.push_str("╚══════════════════════════════════════════════╝\n");
    j
}


pub fn mzf(mp: usize) -> Result<String, &'static str> {
    ensure_init()?;
    let project = Ce.lock();
    let project = project.as_ref().ok_or("No project")?;
    let track = project.tracks.get(mp).ok_or("Invalid track index")?;

    let mut j = String::new();
    j.push_str(&format!("Track {}: \"{}\" — {} notes, {}\n",
        mp, track.name_str(), track.notes.len(), track.waveform.name()));

    if track.notes.is_empty() {
        j.push_str("  (no notes)\n");
    } else {
        j.push_str("  # │ Note │ Vel │ Start(tick) │ Dur(tick) │ Bar:Beat\n");
        j.push_str("  ──┼──────┼─────┼─────────────┼───────────┼─────────\n");
        for (i, note) in track.notes.iter().enumerate().take(64) {
            let name = crate::audio::tables::bno(note.pitch);
            let amb = crate::audio::tables::bui(note.pitch);
            let bar = note.start_tick / (AF_ * 4);
            let beat = (note.start_tick % (AF_ * 4)) / AF_;
            j.push_str(&format!("  {:2}│ {}{:<2}  │ {:>3} │ {:>11} │ {:>9} │ {}:{}\n",
                i, name, amb, note.velocity, note.start_tick, note.duration_ticks,
                bar + 1, beat + 1));
        }
        if track.notes.len() > 64 {
            j.push_str(&format!("  ... and {} more notes\n", track.notes.len() - 64));
        }
    }
    Ok(j)
}






pub fn mzw() -> Result<(), &'static str> {
    ensure_init()?;

    let mut project = Project::new("Demo Song", 120);

    
    let mut bcq = Track::new("Melody");
    bcq.waveform = crate::audio::synth::Waveform::Sine;
    
    bcq.add_note(Note::new(60, 100, 0, 480));       
    bcq.add_note(Note::new(64, 100, 480, 480));     
    bcq.add_note(Note::new(67, 100, 960, 480));     
    bcq.add_note(Note::new(72, 100, 1440, 480));    
    bcq.add_note(Note::new(72, 90, 1920, 480));     
    bcq.add_note(Note::new(67, 90, 2400, 480));     
    bcq.add_note(Note::new(64, 90, 2880, 480));     
    bcq.add_note(Note::new(60, 90, 3360, 480));     
    project.tracks.push(bcq);

    
    let mut bass = Track::new("Bass");
    bass.waveform = crate::audio::synth::Waveform::Sawtooth;
    
    bass.add_note(Note::new(36, 110, 0, 960));        
    bass.add_note(Note::new(36, 110, 960, 960));      
    bass.add_note(Note::new(36, 110, 1920, 960));     
    bass.add_note(Note::new(43, 110, 2880, 960));     
    project.tracks.push(bass);

    
    let mut eku = Track::new("Drums");
    eku.waveform = crate::audio::synth::Waveform::Noise;
    
    for beat in 0..8 {
        eku.add_note(Note::new(36, 127, beat * 480, 120));
    }
    
    for beat in [1, 3, 5, 7] {
        eku.add_note(Note::new(60, 100, beat * 480 + 240, 120));
    }
    project.tracks.push(eku);

    
    let mut bxy = Track::new("Chords");
    bxy.waveform = crate::audio::synth::Waveform::Triangle;
    
    bxy.add_note(Note::new(60, 70, 0, 1920));      
    bxy.add_note(Note::new(64, 70, 0, 1920));      
    bxy.add_note(Note::new(67, 70, 0, 1920));      
    
    bxy.add_note(Note::new(55, 70, 1920, 1920));   
    bxy.add_note(Note::new(59, 70, 1920, 1920));   
    bxy.add_note(Note::new(62, 70, 1920, 1920));   
    project.tracks.push(bxy);

    *Ce.lock() = Some(project);

    
    if let Some(mixer) = Ex.lock().as_mut() {
        mixer.set_volume(0, 200).ok();  
        mixer.set_volume(1, 160).ok();  
        mixer.set_volume(2, 140).ok();  
        mixer.set_volume(3, 120).ok();  
        mixer.set_pan(1, -40).ok();     
        mixer.set_pan(3, 40).ok();      
    }

    crate::serial_println!("[TRUSTDAW] Demo project loaded: 4 tracks, 2 bars");
    Ok(())
}


pub fn njo(name: &str, bpm: u32) -> Result<(), &'static str> {
    ensure_init()?;
    let project = Project::new(name, bpm.clamp(30, 300) as u16);
    *Ce.lock() = Some(project);
    Df.store(bpm.clamp(30, 300), Ordering::Relaxed);
    GG_.store(0, Ordering::Relaxed);
    Ok(())
}


pub fn dpb(path: &str) -> Result<usize, &'static str> {
    ensure_init()?;
    let project = Ce.lock();
    let project = project.as_ref().ok_or("No project")?;
    let mixer = Ex.lock();
    let mixer = mixer.as_ref().ok_or("Mixer not initialized")?;
    let bpm = Df.load(Ordering::Relaxed);

    let jo = mixer::izw(project, mixer, bpm, 0);
    if jo.is_empty() {
        return Err("No audio to export");
    }

    wav_export::dpb(path, &jo, BT_, 2)
}
