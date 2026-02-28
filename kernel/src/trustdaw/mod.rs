//! TrustDAW — Bare-metal Digital Audio Workstation for TrustOS
//!
//! A full-featured DAW running directly on hardware with no OS dependencies.
//! Built on top of TrustOS's existing audio infrastructure:
//!   - SynthEngine (8-voice polyphonic, 5 waveforms, ADSR)
//!   - Intel HDA driver (48kHz/16-bit/stereo DMA playback)
//!   - Framebuffer (double-buffered 2D drawing)
//!   - PS/2 keyboard (press/release detection)
//!
//! Features:
//!   - Multi-track sequencer with per-track waveform/envelope
//!   - Graphical piano roll editor
//!   - PC keyboard → MIDI note mapping (virtual piano)
//!   - Real-time recording from keyboard
//!   - Multi-track mixer with volume/pan
//!   - WAV file export to VFS
//!   - Transport controls (play/stop/record/loop)
//!
//! Shell command: `daw <subcommand>`

pub mod track;
pub mod mixer;
pub mod piano_roll;
pub mod keyboard_midi;
pub mod recorder;
pub mod wav_export;
pub mod ui;

use alloc::string::String;
use alloc::format;
use spin::Mutex;
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use track::{Track, Note, Project};
use mixer::Mixer;

// ═══════════════════════════════════════════════════════════════════════════════
// Global DAW State
// ═══════════════════════════════════════════════════════════════════════════════

/// Global project
pub static PROJECT: Mutex<Option<Project>> = Mutex::new(None);
/// Global mixer
static MIXER: Mutex<Option<Mixer>> = Mutex::new(None);
/// Is the DAW initialized?
static INITIALIZED: AtomicBool = AtomicBool::new(false);
/// Is playback active?
pub static PLAYING: AtomicBool = AtomicBool::new(false);
/// Is recording active?
pub static RECORDING: AtomicBool = AtomicBool::new(false);
/// Current playback position in ticks (1 tick = 1/480 quarter note)
pub static PLAYBACK_POS: AtomicU32 = AtomicU32::new(0);
/// Current BPM
pub static BPM: AtomicU32 = AtomicU32::new(120);

/// Ticks per quarter note (standard MIDI resolution)
pub const TICKS_PER_QUARTER: u32 = 480;
/// Sample rate (matches HDA)
pub const SAMPLE_RATE: u32 = 48000;
/// Max tracks in a project
pub const MAX_TRACKS: usize = 16;

// ═══════════════════════════════════════════════════════════════════════════════
// Initialization
// ═══════════════════════════════════════════════════════════════════════════════

/// Initialize TrustDAW
pub fn init() -> Result<(), &'static str> {
    if INITIALIZED.load(Ordering::Relaxed) {
        return Ok(());
    }

    // Ensure audio subsystem is ready
    crate::audio::init().map_err(|_| "Failed to init audio subsystem")?;

    // Create default project
    let project = Project::new("Untitled", 120);
    *PROJECT.lock() = Some(project);

    // Create mixer
    let mixer = Mixer::new(MAX_TRACKS);
    *MIXER.lock() = Some(mixer);

    BPM.store(120, Ordering::Relaxed);
    INITIALIZED.store(true, Ordering::Relaxed);

    crate::serial_println!("[TRUSTDAW] TrustDAW initialized — {} tracks max, {} ticks/quarter",
        MAX_TRACKS, TICKS_PER_QUARTER);
    Ok(())
}

/// Ensure DAW is initialized
pub fn ensure_init() -> Result<(), &'static str> {
    if !INITIALIZED.load(Ordering::Relaxed) {
        init()?;
    }
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════════════
// Transport Controls
// ═══════════════════════════════════════════════════════════════════════════════

/// Start playback from current position
pub fn play() -> Result<(), &'static str> {
    ensure_init()?;
    if PLAYING.load(Ordering::Relaxed) {
        return Ok(()); // Already playing
    }

    let project = PROJECT.lock();
    let project = project.as_ref().ok_or("No project loaded")?;
    let mixer_lock = MIXER.lock();
    let mixer = mixer_lock.as_ref().ok_or("Mixer not initialized")?;

    // Render all tracks from current position
    let bpm = BPM.load(Ordering::Relaxed);
    let start_tick = PLAYBACK_POS.load(Ordering::Relaxed);
    let samples = mixer::render_project(project, mixer, bpm, start_tick);

    if samples.is_empty() {
        return Err("No audio to play (add notes to tracks first)");
    }

    PLAYING.store(true, Ordering::Relaxed);

    // Calculate duration
    let duration_ms = (samples.len() as u32 / 2) * 1000 / SAMPLE_RATE;

    // Play through HDA
    let result = crate::drivers::hda::write_samples_and_play(&samples, duration_ms);

    PLAYING.store(false, Ordering::Relaxed);
    result
}

/// Stop playback/recording
pub fn stop() {
    PLAYING.store(false, Ordering::Relaxed);
    RECORDING.store(false, Ordering::Relaxed);
    let _ = crate::drivers::hda::stop();
}

/// Rewind to beginning
pub fn rewind() {
    PLAYBACK_POS.store(0, Ordering::Relaxed);
}

/// Set BPM
pub fn set_bpm(bpm: u32) {
    let bpm = bpm.clamp(30, 300);
    BPM.store(bpm, Ordering::Relaxed);
    if let Some(project) = PROJECT.lock().as_mut() {
        project.bpm = bpm as u16;
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Track Management
// ═══════════════════════════════════════════════════════════════════════════════

/// Add a new track to the project
pub fn add_track(name: &str) -> Result<usize, &'static str> {
    ensure_init()?;
    let mut project = PROJECT.lock();
    let project = project.as_mut().ok_or("No project")?;
    project.add_track(name)
}

/// Remove a track by index
pub fn remove_track(index: usize) -> Result<(), &'static str> {
    ensure_init()?;
    let mut project = PROJECT.lock();
    let project = project.as_mut().ok_or("No project")?;
    project.remove_track(index)
}

/// Add a note to a track
pub fn add_note(track_idx: usize, note: u8, velocity: u8, start_tick: u32, duration_ticks: u32) -> Result<(), &'static str> {
    ensure_init()?;
    let mut project = PROJECT.lock();
    let project = project.as_mut().ok_or("No project")?;
    let track = project.tracks.get_mut(track_idx).ok_or("Invalid track index")?;
    track.add_note(Note::new(note, velocity, start_tick, duration_ticks));
    Ok(())
}

/// Set a track's waveform
pub fn set_track_waveform(track_idx: usize, waveform: &str) -> Result<(), &'static str> {
    ensure_init()?;
    let wf = crate::audio::synth::Waveform::from_str(waveform)
        .ok_or("Unknown waveform (sine/square/saw/triangle/noise)")?;
    let mut project = PROJECT.lock();
    let project = project.as_mut().ok_or("No project")?;
    let track = project.tracks.get_mut(track_idx).ok_or("Invalid track index")?;
    track.waveform = wf;
    Ok(())
}

/// Set track volume in the mixer
pub fn set_track_volume(track_idx: usize, volume: u8) -> Result<(), &'static str> {
    ensure_init()?;
    let mut mixer = MIXER.lock();
    let mixer = mixer.as_mut().ok_or("Mixer not initialized")?;
    mixer.set_volume(track_idx, volume)
}

/// Set track pan in the mixer (-100 = full left, 0 = center, +100 = full right)
pub fn set_track_pan(track_idx: usize, pan: i8) -> Result<(), &'static str> {
    ensure_init()?;
    let mut mixer = MIXER.lock();
    let mixer = mixer.as_mut().ok_or("Mixer not initialized")?;
    mixer.set_pan(track_idx, pan)
}

/// Mute/unmute a track
pub fn toggle_mute(track_idx: usize) -> Result<bool, &'static str> {
    ensure_init()?;
    let mut mixer = MIXER.lock();
    let mixer = mixer.as_mut().ok_or("Mixer not initialized")?;
    mixer.toggle_mute(track_idx)
}

/// Solo a track
pub fn toggle_solo(track_idx: usize) -> Result<bool, &'static str> {
    ensure_init()?;
    let mut mixer = MIXER.lock();
    let mixer = mixer.as_mut().ok_or("Mixer not initialized")?;
    mixer.toggle_solo(track_idx)
}

// ═══════════════════════════════════════════════════════════════════════════════
// Project Info
// ═══════════════════════════════════════════════════════════════════════════════

/// Get project status
pub fn status() -> String {
    let project = PROJECT.lock();
    let mixer = MIXER.lock();
    let bpm = BPM.load(Ordering::Relaxed);
    let pos = PLAYBACK_POS.load(Ordering::Relaxed);
    let playing = PLAYING.load(Ordering::Relaxed);
    let recording = RECORDING.load(Ordering::Relaxed);

    let mut s = String::new();
    s.push_str("╔══════════════════════════════════════════════╗\n");
    s.push_str("║          TrustDAW — Digital Audio Workstation║\n");
    s.push_str("╠══════════════════════════════════════════════╣\n");

    if let Some(proj) = project.as_ref() {
        s.push_str(&format!("║ Project: {:<36}║\n", proj.name_str()));
        s.push_str(&format!("║ BPM: {:<3}  Tracks: {}/{:<22}║\n",
            bpm, proj.tracks.len(), MAX_TRACKS));
        let bar = pos / (TICKS_PER_QUARTER * 4);
        let beat = (pos % (TICKS_PER_QUARTER * 4)) / TICKS_PER_QUARTER;
        let tick = pos % TICKS_PER_QUARTER;
        let state = if recording { "REC" } else if playing { "PLAY" } else { "STOP" };
        s.push_str(&format!("║ Position: {}:{:02}:{:03}  State: {:<14}║\n",
            bar + 1, beat + 1, tick, state));
        s.push_str("╠══════════════════════════════════════════════╣\n");

        if proj.tracks.is_empty() {
            s.push_str("║ No tracks. Use 'daw track add <name>'       ║\n");
        } else {
            s.push_str("║ # │ Name         │ Notes │ Wave  │ Vol │ Pan ║\n");
            s.push_str("║───┼──────────────┼───────┼───────┼─────┼─────║\n");
            for (i, track) in proj.tracks.iter().enumerate() {
                let vol = if let Some(m) = mixer.as_ref() {
                    m.channels.get(i).map(|c| c.volume).unwrap_or(200)
                } else { 200 };
                let pan = if let Some(m) = mixer.as_ref() {
                    m.channels.get(i).map(|c| c.pan).unwrap_or(0)
                } else { 0 };
                let mute_str = if let Some(m) = mixer.as_ref() {
                    if m.channels.get(i).map(|c| c.muted).unwrap_or(false) { "M" } else { " " }
                } else { " " };
                s.push_str(&format!("║{}{:2}│ {:<12} │ {:>5} │ {:<5} │ {:>3} │ {:>+3} ║\n",
                    mute_str, i, track.name_str(), track.notes.len(),
                    track.waveform.short_name(), vol, pan));
            }
        }
    } else {
        s.push_str("║ No project loaded                            ║\n");
    }

    s.push_str("╚══════════════════════════════════════════════╝\n");
    s
}

/// List notes in a track
pub fn list_notes(track_idx: usize) -> Result<String, &'static str> {
    ensure_init()?;
    let project = PROJECT.lock();
    let project = project.as_ref().ok_or("No project")?;
    let track = project.tracks.get(track_idx).ok_or("Invalid track index")?;

    let mut s = String::new();
    s.push_str(&format!("Track {}: \"{}\" — {} notes, {}\n",
        track_idx, track.name_str(), track.notes.len(), track.waveform.name()));

    if track.notes.is_empty() {
        s.push_str("  (no notes)\n");
    } else {
        s.push_str("  # │ Note │ Vel │ Start(tick) │ Dur(tick) │ Bar:Beat\n");
        s.push_str("  ──┼──────┼─────┼─────────────┼───────────┼─────────\n");
        for (i, note) in track.notes.iter().enumerate().take(64) {
            let name = crate::audio::tables::midi_to_note_name(note.pitch);
            let oct = crate::audio::tables::midi_octave(note.pitch);
            let bar = note.start_tick / (TICKS_PER_QUARTER * 4);
            let beat = (note.start_tick % (TICKS_PER_QUARTER * 4)) / TICKS_PER_QUARTER;
            s.push_str(&format!("  {:2}│ {}{:<2}  │ {:>3} │ {:>11} │ {:>9} │ {}:{}\n",
                i, name, oct, note.velocity, note.start_tick, note.duration_ticks,
                bar + 1, beat + 1));
        }
        if track.notes.len() > 64 {
            s.push_str(&format!("  ... and {} more notes\n", track.notes.len() - 64));
        }
    }
    Ok(s)
}

// ═══════════════════════════════════════════════════════════════════════════════
// Demo / Presets
// ═══════════════════════════════════════════════════════════════════════════════

/// Load a demo project with pre-made tracks
pub fn load_demo() -> Result<(), &'static str> {
    ensure_init()?;

    let mut project = Project::new("Demo Song", 120);

    // ── Track 0: Melody (sine) ──
    let mut melody = Track::new("Melody");
    melody.waveform = crate::audio::synth::Waveform::Sine;
    // C4 E4 G4 C5 — ascending arpeggio over 2 bars
    melody.add_note(Note::new(60, 100, 0, 480));       // C4, bar 1 beat 1
    melody.add_note(Note::new(64, 100, 480, 480));     // E4, bar 1 beat 2
    melody.add_note(Note::new(67, 100, 960, 480));     // G4, bar 1 beat 3
    melody.add_note(Note::new(72, 100, 1440, 480));    // C5, bar 1 beat 4
    melody.add_note(Note::new(72, 90, 1920, 480));     // C5, bar 2 beat 1
    melody.add_note(Note::new(67, 90, 2400, 480));     // G4, bar 2 beat 2
    melody.add_note(Note::new(64, 90, 2880, 480));     // E4, bar 2 beat 3
    melody.add_note(Note::new(60, 90, 3360, 480));     // C4, bar 2 beat 4
    project.tracks.push(melody);

    // ── Track 1: Bass (sawtooth) ──
    let mut bass = Track::new("Bass");
    bass.waveform = crate::audio::synth::Waveform::Sawtooth;
    // Root notes on beats 1 and 3
    bass.add_note(Note::new(36, 110, 0, 960));        // C2, bar 1 beats 1-2
    bass.add_note(Note::new(36, 110, 960, 960));      // C2, bar 1 beats 3-4
    bass.add_note(Note::new(36, 110, 1920, 960));     // C2, bar 2 beats 1-2
    bass.add_note(Note::new(43, 110, 2880, 960));     // G2, bar 2 beats 3-4
    project.tracks.push(bass);

    // ── Track 2: Drums (noise) ──
    let mut drums = Track::new("Drums");
    drums.waveform = crate::audio::synth::Waveform::Noise;
    // Kick pattern: every beat
    for beat in 0..8 {
        drums.add_note(Note::new(36, 127, beat * 480, 120));
    }
    // Snare on 2 and 4
    for beat in [1, 3, 5, 7] {
        drums.add_note(Note::new(60, 100, beat * 480 + 240, 120));
    }
    project.tracks.push(drums);

    // ── Track 3: Chords (triangle) ──
    let mut chords = Track::new("Chords");
    chords.waveform = crate::audio::synth::Waveform::Triangle;
    // C major chord: C4+E4+G4 held for full bars
    chords.add_note(Note::new(60, 70, 0, 1920));      // C4, bars 1-2
    chords.add_note(Note::new(64, 70, 0, 1920));      // E4, bars 1-2
    chords.add_note(Note::new(67, 70, 0, 1920));      // G4, bars 1-2
    // G major: G3+B3+D4
    chords.add_note(Note::new(55, 70, 1920, 1920));   // G3, bars 3-4
    chords.add_note(Note::new(59, 70, 1920, 1920));   // B3, bars 3-4
    chords.add_note(Note::new(62, 70, 1920, 1920));   // D4, bars 3-4
    project.tracks.push(chords);

    *PROJECT.lock() = Some(project);

    // Set mixer volumes
    if let Some(mixer) = MIXER.lock().as_mut() {
        mixer.set_volume(0, 200).ok();  // Melody
        mixer.set_volume(1, 160).ok();  // Bass
        mixer.set_volume(2, 140).ok();  // Drums
        mixer.set_volume(3, 120).ok();  // Chords
        mixer.set_pan(1, -40).ok();     // Bass left
        mixer.set_pan(3, 40).ok();      // Chords right
    }

    crate::serial_println!("[TRUSTDAW] Demo project loaded: 4 tracks, 2 bars");
    Ok(())
}

/// Create a new empty project
pub fn new_project(name: &str, bpm: u32) -> Result<(), &'static str> {
    ensure_init()?;
    let project = Project::new(name, bpm.clamp(30, 300) as u16);
    *PROJECT.lock() = Some(project);
    BPM.store(bpm.clamp(30, 300), Ordering::Relaxed);
    PLAYBACK_POS.store(0, Ordering::Relaxed);
    Ok(())
}

/// Export project to WAV file
pub fn export_wav(path: &str) -> Result<usize, &'static str> {
    ensure_init()?;
    let project = PROJECT.lock();
    let project = project.as_ref().ok_or("No project")?;
    let mixer = MIXER.lock();
    let mixer = mixer.as_ref().ok_or("Mixer not initialized")?;
    let bpm = BPM.load(Ordering::Relaxed);

    let samples = mixer::render_project(project, mixer, bpm, 0);
    if samples.is_empty() {
        return Err("No audio to export");
    }

    wav_export::export_wav(path, &samples, SAMPLE_RATE, 2)
}
