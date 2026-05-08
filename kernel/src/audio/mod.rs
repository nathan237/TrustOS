//! TrustOS Audio Subsystem
//!
//! Provides:
//!   - `synth` — Multi-waveform synthesizer with ADSR envelopes  
//!   - `tables` — Sine LUT, MIDI frequency table, note name parser
//!   - High-level API for playing synthesized audio through Intel HDA
//!
//! Architecture:
//!   SynthEngine → render samples → write to HDA buffer → DMA playback

pub mod tables;
pub mod synth;
pub mod pattern;
pub mod player;
pub mod strudel;
#[cfg(feature = "strudel")]
pub mod strudel_dsl;
pub mod live_engine;

use spin::Mutex;
use alloc::string::String;
use alloc::format;

use synth::{SynthEngine, Waveform, Envelope};
pub use synth::{FilterMode, FilterSettings};
use pattern::{Pattern, PatternBank};
use player::PatternPlayer;
use strudel::LiveSession;
use live_engine::LiveEngine;

/// Global synth engine instance
static SYNTH: Mutex<Option<SynthEngine>> = Mutex::new(None);
/// Global pattern bank
static PATTERNS: Mutex<Option<PatternBank>> = Mutex::new(None);
/// Global player
static PLAYER: Mutex<Option<PatternPlayer>> = Mutex::new(None);
/// Global live session
static LIVE: Mutex<Option<LiveSession>> = Mutex::new(None);
/// Global multi-track live engine (TrustStrudel)
static ENGINE: Mutex<Option<LiveEngine>> = Mutex::new(None);

/// Initialize the audio subsystem (HDA driver + synth engine + pattern bank)
pub fn init() -> Result<(), &'static str> {
    // Ensure HDA driver is initialized
    if !crate::drivers::hda::is_initialized() {
        crate::drivers::hda::init()?;
    }

    // Create synth engine
    let engine = SynthEngine::new();
    *SYNTH.lock() = Some(engine);

    // Create pattern bank with presets
    let mut bank = PatternBank::new();
    bank.load_presets();
    *PATTERNS.lock() = Some(bank);

    // Create player
    *PLAYER.lock() = Some(PatternPlayer::new());

    crate::serial_println!("[AUDIO] TrustSynth engine + pattern bank initialized");
    Ok(())
}

/// Ensure synth is initialized, init if needed
fn ensure_init() -> Result<(), &'static str> {
    if SYNTH.lock().is_none() {
        init()?;
    }
    Ok(())
}

/// Play a single note by name (e.g., "C4", "A#3") for a duration
pub fn play_note(name: &str, duration_ms: u32) -> Result<(), &'static str> {
    ensure_init()?;

    let samples = {
        let mut synth = SYNTH.lock();
        let engine = synth.as_mut().ok_or("Synth not initialized")?;
        engine.play_note_by_name(name, duration_ms)?
    };

    // Write samples to HDA and play
    play_samples(&samples, duration_ms)?;
    Ok(())
}

/// Play a note by MIDI number
pub fn play_midi_note(note: u8, velocity: u8, duration_ms: u32) -> Result<(), &'static str> {
    ensure_init()?;

    let samples = {
        let mut synth = SYNTH.lock();
        let engine = synth.as_mut().ok_or("Synth not initialized")?;
        engine.render_note(note, velocity, duration_ms)
    };

    play_samples(&samples, duration_ms)?;
    Ok(())
}

/// Play a tone at a specific frequency
pub fn play_freq(freq_hz: u32, duration_ms: u32) -> Result<(), &'static str> {
    ensure_init()?;

    let samples = {
        let mut synth = SYNTH.lock();
        let engine = synth.as_mut().ok_or("Synth not initialized")?;
        engine.render_freq(freq_hz, duration_ms)
    };

    play_samples(&samples, duration_ms)?;
    Ok(())
}

/// Set the default waveform
pub fn set_waveform(wf: Waveform) -> Result<(), &'static str> {
    ensure_init()?;
    let mut synth = SYNTH.lock();
    let engine = synth.as_mut().ok_or("Synth not initialized")?;
    engine.set_waveform(wf);
    Ok(())
}

/// Set ADSR envelope parameters
pub fn set_adsr(attack_ms: u32, decay_ms: u32, sustain_pct: u32, release_ms: u32) -> Result<(), &'static str> {
    ensure_init()?;
    let mut synth = SYNTH.lock();
    let engine = synth.as_mut().ok_or("Synth not initialized")?;
    engine.set_adsr(attack_ms, decay_ms, sustain_pct, release_ms);
    Ok(())
}

/// Set envelope preset
pub fn set_envelope_preset(name: &str) -> Result<(), &'static str> {
    ensure_init()?;
    let env = match name {
        "default" => Envelope::default_env(),
        "organ" => Envelope::organ(),
        "pluck" => Envelope::pluck(),
        "pad" => Envelope::pad(),
        _ => return Err("Unknown preset (use: default, organ, pluck, pad)"),
    };
    let mut synth = SYNTH.lock();
    let engine = synth.as_mut().ok_or("Synth not initialized")?;
    engine.envelope = env;
    Ok(())
}

/// Set master volume (0-255)
pub fn set_volume(vol: u8) -> Result<(), &'static str> {
    ensure_init()?;
    let mut synth = SYNTH.lock();
    let engine = synth.as_mut().ok_or("Synth not initialized")?;
    engine.master_volume = vol;
    Ok(())
}

/// Get synth status
pub fn status() -> String {
    let synth = SYNTH.lock();
    match synth.as_ref() {
        Some(engine) => engine.status(),
        None => String::from("TrustSynth: not initialized\n"),
    }
}

/// Stop all audio
pub fn stop() -> Result<(), &'static str> {
    {
        let mut synth = SYNTH.lock();
        if let Some(engine) = synth.as_mut() {
            engine.all_notes_off();
        }
    }
    crate::drivers::hda::stop()
}

// ═══════════════════════════════════════════════════════════════════════════════
// Internal: write rendered samples to HDA buffer and play
// ═══════════════════════════════════════════════════════════════════════════════

/// Write rendered audio samples to the HDA DMA buffer and trigger playback
fn play_samples(samples: &[i16], duration_ms: u32) -> Result<(), &'static str> {
    // Access HDA driver internals to copy samples into the DMA buffer
    crate::drivers::hda::write_samples_and_play(samples, duration_ms)
}

// ═══════════════════════════════════════════════════════════════════════════════
// Pattern API
// ═══════════════════════════════════════════════════════════════════════════════

/// Ensure pattern bank is initialized
fn ensure_patterns() -> Result<(), &'static str> {
    ensure_init()?;
    if PATTERNS.lock().is_none() {
        let mut bank = PatternBank::new();
        bank.load_presets();
        *PATTERNS.lock() = Some(bank);
    }
    if PLAYER.lock().is_none() {
        *PLAYER.lock() = Some(PatternPlayer::new());
    }
    Ok(())
}

/// Create a new pattern
pub fn pattern_new(name: &str, steps: usize, bpm: u16) -> Result<(), &'static str> {
    ensure_patterns()?;
    let pattern = Pattern::new(name, steps, bpm);
    let mut bank = PATTERNS.lock();
    let bank = bank.as_mut().ok_or("Pattern bank not initialized")?;
    bank.add(pattern)?;
    Ok(())
}

/// Set a note in a pattern
pub fn pattern_set_note(name: &str, step: usize, note_name: &str) -> Result<(), &'static str> {
    ensure_patterns()?;
    let mut bank = PATTERNS.lock();
    let bank = bank.as_mut().ok_or("Pattern bank not initialized")?;
    let pat = bank.get_by_name_mut(name).ok_or("Pattern not found")?;
    pat.set_note(step, note_name)
}

/// Set BPM on a pattern
pub fn pattern_set_bpm(name: &str, bpm: u16) -> Result<(), &'static str> {
    ensure_patterns()?;
    let mut bank = PATTERNS.lock();
    let bank = bank.as_mut().ok_or("Pattern bank not initialized")?;
    let pat = bank.get_by_name_mut(name).ok_or("Pattern not found")?;
    pat.bpm = bpm;
    Ok(())
}

/// Set waveform on a pattern
pub fn pattern_set_wave(name: &str, wf: Waveform) -> Result<(), &'static str> {
    ensure_patterns()?;
    let mut bank = PATTERNS.lock();
    let bank = bank.as_mut().ok_or("Pattern bank not initialized")?;
    let pat = bank.get_by_name_mut(name).ok_or("Pattern not found")?;
    pat.waveform = wf;
    Ok(())
}

/// Display a pattern
pub fn pattern_show(name: &str) -> Result<String, &'static str> {
    ensure_patterns()?;
    let bank = PATTERNS.lock();
    let bank = bank.as_ref().ok_or("Pattern bank not initialized")?;
    let pat = bank.get_by_name(name).ok_or("Pattern not found")?;
    Ok(pat.display())
}

/// List all patterns
pub fn pattern_list() -> String {
    let bank = PATTERNS.lock();
    match bank.as_ref() {
        Some(b) => b.list(),
        None => String::from("Pattern bank not initialized\n"),
    }
}

/// Remove a pattern
pub fn pattern_remove(name: &str) -> Result<(), &'static str> {
    ensure_patterns()?;
    let mut bank = PATTERNS.lock();
    let bank = bank.as_mut().ok_or("Pattern bank not initialized")?;
    bank.remove(name)
}

/// Play a pattern by name
pub fn pattern_play(name: &str, loops: u32) -> Result<(), &'static str> {
    ensure_patterns()?;

    // Clone the pattern so we don't hold the lock during playback
    let pattern = {
        let bank = PATTERNS.lock();
        let bank = bank.as_ref().ok_or("Pattern bank not initialized")?;
        bank.get_by_name(name).ok_or("Pattern not found")?.clone()
    };

    // Get synth engine and player — need to drop locks carefully
    let mut synth_lock = SYNTH.lock();
    let engine = synth_lock.as_mut().ok_or("Synth not initialized")?;
    let mut player_lock = PLAYER.lock();
    let player = player_lock.as_mut().ok_or("Player not initialized")?;

    player.play_pattern_visual(&pattern, engine, loops)
}

/// Stop pattern playback
pub fn pattern_stop() -> Result<(), &'static str> {
    let mut player_lock = PLAYER.lock();
    if let Some(player) = player_lock.as_mut() {
        player.stop();
    }
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════════════
// Strudel / Live Coding API
// ═══════════════════════════════════════════════════════════════════════════════

fn ensure_live() -> Result<(), &'static str> {
    ensure_init()?;
    if LIVE.lock().is_none() {
        *LIVE.lock() = Some(LiveSession::new());
    }
    Ok(())
}

/// Parse and play a Strudel mini-notation pattern
pub fn live_play(notation: &str, loops: u32) -> Result<(), &'static str> {
    ensure_live()?;

    let (bpm, wf) = {
        let live = LIVE.lock();
        let session = live.as_ref().ok_or("Live session not initialized")?;
        (session.bpm, session.waveform)
    };

    let pattern = strudel::to_pattern(notation, "live", bpm, wf)?;

    {
        let mut live = LIVE.lock();
        let session = live.as_mut().ok_or("Live session not initialized")?;
        session.notation = String::from(notation);
        session.running = true;
    }

    let result = {
        let mut synth_lock = SYNTH.lock();
        let engine = synth_lock.as_mut().ok_or("Synth not initialized")?;
        let mut player_lock = PLAYER.lock();
        let player = player_lock.as_mut().ok_or("Player not initialized")?;
        player.play_pattern_visual(&pattern, engine, loops)
    };

    {
        let mut live = LIVE.lock();
        if let Some(session) = live.as_mut() {
            session.loops_played += loops;
            session.running = false;
        }
    }

    result
}

/// Set live session BPM
pub fn live_set_bpm(bpm: u16) -> Result<(), &'static str> {
    ensure_live()?;
    let mut live = LIVE.lock();
    let session = live.as_mut().ok_or("Live session not initialized")?;
    session.bpm = bpm.clamp(30, 300);
    Ok(())
}

/// Set live session waveform
pub fn live_set_wave(wf: Waveform) -> Result<(), &'static str> {
    ensure_live()?;
    let mut live = LIVE.lock();
    let session = live.as_mut().ok_or("Live session not initialized")?;
    session.waveform = wf;
    Ok(())
}

/// Preview a mini-notation (parse only, no playback)
pub fn live_preview(notation: &str) -> Result<String, &'static str> {
    strudel::display_parsed(notation)
}

/// Get live session status
pub fn live_status() -> String {
    let live = LIVE.lock();
    match live.as_ref() {
        Some(session) => session.status(),
        None => String::from("Live session not started. Use: live \"pattern\"\n"),
    }
}

/// Stop live playback
pub fn live_stop() -> Result<(), &'static str> {
    {
        let mut live = LIVE.lock();
        if let Some(session) = live.as_mut() {
            session.running = false;
        }
    }
    stop()
}

// ═══════════════════════════════════════════════════════════════════════════════
// TrustStrudel — Multi-track Live Engine API
// ═══════════════════════════════════════════════════════════════════════════════

fn ensure_engine() -> Result<(), &'static str> {
    ensure_init()?;
    if ENGINE.lock().is_none() {
        *ENGINE.lock() = Some(LiveEngine::new());
    }
    Ok(())
}

/// Set a track pattern: d<n> "notation"
pub fn strudel_set_track(track: usize, notation: &str) -> Result<(), &'static str> {
    ensure_engine()?;
    let mut eng = ENGINE.lock();
    let engine = eng.as_mut().ok_or("Engine not initialized")?;
    engine.set_track(track, notation)
}

/// Set track waveform
pub fn strudel_track_wave(track: usize, wf: Waveform) -> Result<(), &'static str> {
    ensure_engine()?;
    let mut eng = ENGINE.lock();
    let engine = eng.as_mut().ok_or("Engine not initialized")?;
    engine.set_track_wave(track, wf)
}

/// Set track volume
pub fn strudel_track_vol(track: usize, vol: u8) -> Result<(), &'static str> {
    ensure_engine()?;
    let mut eng = ENGINE.lock();
    let engine = eng.as_mut().ok_or("Engine not initialized")?;
    engine.set_track_volume(track, vol)
}

/// Set per-track filter mode (LP/HP/BP/Off).
pub fn strudel_track_filter_mode(track: usize, mode: FilterMode) -> Result<(), &'static str> {
    ensure_engine()?;
    let mut eng = ENGINE.lock();
    let engine = eng.as_mut().ok_or("Engine not initialized")?;
    engine.set_track_filter_mode(track, mode)
}

/// Set per-track filter cutoff (Hz). Auto-enables LP if filter was Off.
pub fn strudel_track_cutoff(track: usize, hz: u32) -> Result<(), &'static str> {
    ensure_engine()?;
    let mut eng = ENGINE.lock();
    let engine = eng.as_mut().ok_or("Engine not initialized")?;
    engine.set_track_cutoff(track, hz)
}

/// Set per-track filter resonance (0..=255).
pub fn strudel_track_q(track: usize, q: u8) -> Result<(), &'static str> {
    ensure_engine()?;
    let mut eng = ENGINE.lock();
    let engine = eng.as_mut().ok_or("Engine not initialized")?;
    engine.set_track_resonance(track, q)
}

/// Tone shortcut 0..127 → log-spaced LPF cutoff.
pub fn strudel_track_tone(track: usize, tone: u8) -> Result<(), &'static str> {
    ensure_engine()?;
    let mut eng = ENGINE.lock();
    let engine = eng.as_mut().ok_or("Engine not initialized")?;
    engine.set_track_tone(track, tone)
}

/// Disable per-track filter (returns to waveform-based warmth).
pub fn strudel_track_nofilter(track: usize) -> Result<(), &'static str> {
    ensure_engine()?;
    let mut eng = ENGINE.lock();
    let engine = eng.as_mut().ok_or("Engine not initialized")?;
    engine.clear_track_filter(track)
}

/// Mute/unmute track
pub fn strudel_mute(track: usize) -> Result<(), &'static str> {
    ensure_engine()?;
    let mut eng = ENGINE.lock();
    let engine = eng.as_mut().ok_or("Engine not initialized")?;
    engine.mute_track(track)
}

/// Clear a track
pub fn strudel_clear(track: usize) -> Result<(), &'static str> {
    ensure_engine()?;
    let mut eng = ENGINE.lock();
    let engine = eng.as_mut().ok_or("Engine not initialized")?;
    engine.clear_track(track)
}

/// Set global BPM
pub fn strudel_bpm(bpm: u16) -> Result<(), &'static str> {
    ensure_engine()?;
    let mut eng = ENGINE.lock();
    let engine = eng.as_mut().ok_or("Engine not initialized")?;
    engine.set_bpm(bpm);
    Ok(())
}

/// Play all active tracks (one cycle, blocking with visual)
pub fn strudel_play(loops: u32) -> Result<(), &'static str> {
    ensure_engine()?;
    let mut eng = ENGINE.lock();
    let engine = eng.as_mut().ok_or("Engine not initialized")?;
    let mut synth_lock = SYNTH.lock();
    let synth = synth_lock.as_mut().ok_or("Synth not initialized")?;
    for _ in 0..loops.max(1) {
        engine.play_cycle(synth)?;
    }
    Ok(())
}

/// Start looped non-blocking playback (hot-swappable)
pub fn strudel_loop() -> Result<(), &'static str> {
    ensure_engine()?;
    let mut eng = ENGINE.lock();
    let engine = eng.as_mut().ok_or("Engine not initialized")?;
    let mut synth_lock = SYNTH.lock();
    let synth = synth_lock.as_mut().ok_or("Synth not initialized")?;
    engine.start_loop(synth)
}

/// Re-render and swap the loop buffer (after track changes)
pub fn strudel_update() -> Result<(), &'static str> {
    ensure_engine()?;
    let mut eng = ENGINE.lock();
    let engine = eng.as_mut().ok_or("Engine not initialized")?;
    let mut synth_lock = SYNTH.lock();
    let synth = synth_lock.as_mut().ok_or("Synth not initialized")?;
    engine.update_loop(synth)
}

/// Silence everything
pub fn strudel_hush() -> Result<(), &'static str> {
    ensure_engine()?;
    let mut eng = ENGINE.lock();
    let engine = eng.as_mut().ok_or("Engine not initialized")?;
    engine.hush();
    Ok(())
}

/// Stop playback
pub fn strudel_stop() -> Result<(), &'static str> {
    ensure_engine()?;
    let mut eng = ENGINE.lock();
    let engine = eng.as_mut().ok_or("Engine not initialized")?;
    engine.stop();
    Ok(())
}

/// Get multi-track status
pub fn strudel_status() -> String {
    let eng = ENGINE.lock();
    match eng.as_ref() {
        Some(engine) => engine.status(),
        None => String::from("TrustStrudel not started. Use: d1 \"pattern\"\n"),
    }
}

/// Snapshot of all 8 tracks for UI rendering.
/// Returns empty Vec if engine not yet started.
/// Tuple: (notation, active, muted, volume, waveform_short_name,
///         filter_mode_short, cutoff_hz, resonance)
pub fn strudel_snapshot()
    -> alloc::vec::Vec<(String, bool, bool, u8, &'static str, &'static str, u32, u8)>
{
    let eng = ENGINE.lock();
    let mut out = alloc::vec::Vec::new();
    if let Some(engine) = eng.as_ref() {
        for t in engine.tracks.iter() {
            out.push((
                t.notation.clone(),
                t.active,
                t.muted,
                t.volume,
                t.waveform.short_name(),
                t.filter.mode.short_name(),
                t.filter.cutoff_hz,
                t.filter.resonance,
            ));
        }
    }
    out
}

/// Engine global state: (bpm, playing, cycles_played).
pub fn strudel_state() -> (u16, bool, u32) {
    let eng = ENGINE.lock();
    eng.as_ref()
        .map(|e| (e.bpm, e.playing, e.cycles_played))
        .unwrap_or((120, false, 0))
}


// -------------------------------------------------------------------------------
// TrustStrudel DSL (chained method-call syntax) � P1+
// Gated behind `feature = "strudel"` (included in trustos-audio edition).
// -------------------------------------------------------------------------------

/// Parse a DSL string and assign it to a track on the live engine.
///
/// Example: `dsl_set_track(0, "n(\"0 4 7\").scale(\"g:minor\").s(\"saw\").lpf(800)")`
#[cfg(feature = "strudel")]
pub fn dsl_set_track(idx: usize, src: &str) -> Result<(), String> {
    ensure_engine().map_err(String::from)?;
    let bundle = strudel_dsl::parse_eval(src)?;
    let mut eng = ENGINE.lock();
    let engine = eng.as_mut().ok_or_else(|| String::from("Engine not initialized"))?;
    if idx >= live_engine::MAX_TRACKS {
        return Err(String::from("Track index out of range (1..=8)"));
    }
    let track = &mut engine.tracks[idx];
    track.waveform = bundle.pattern.waveform;
    track.envelope = bundle.pattern.envelope;
    track.notation = String::from(src);
    let mut pat = bundle.pattern;
    pat.bpm = engine.bpm;
    track.pattern = Some(pat);
    track.active = true;
    track.muted = false;

    // Apply gain ? volume mapping (Q16.16 ? 0..255).
    let g = bundle.controls.gain.max(0);
    let vol = ((g.saturating_mul(255)) >> 16).clamp(0, 255) as u8;
    if vol > 0 { track.volume = vol; }

    // Apply filter override.
    if bundle.controls.lpf_hz > 0 {
        track.filter.mode = synth::FilterMode::LowPass;
        track.filter.cutoff_hz = bundle.controls.lpf_hz;
        track.filter.resonance = ((bundle.controls.lpq.max(0) * 255) >> 16).clamp(0, 255) as u8;
    } else if bundle.controls.hpf_hz > 0 {
        track.filter.mode = synth::FilterMode::HighPass;
        track.filter.cutoff_hz = bundle.controls.hpf_hz;
        track.filter.resonance = ((bundle.controls.hpq.max(0) * 255) >> 16).clamp(0, 255) as u8;
    } else if bundle.controls.bpf_hz > 0 {
        track.filter.mode = synth::FilterMode::BandPass;
        track.filter.cutoff_hz = bundle.controls.bpf_hz;
        track.filter.resonance = ((bundle.controls.bpq.max(0) * 255) >> 16).clamp(0, 255) as u8;
    }

    crate::serial_println!(
        "[strudel-dsl] track {} set: {} steps, {} BPM, wave={}",
        idx + 1, track.pattern.as_ref().map(|p| p.steps.len()).unwrap_or(0),
        engine.bpm, track.waveform.short_name()
    );
    Ok(())
}

/// One-shot: parse, evaluate, render one cycle, play it (no looping).
#[cfg(feature = "strudel")]
pub fn dsl_oneshot(src: &str) -> Result<(), String> {
    ensure_init().map_err(String::from)?;
    let bundle = strudel_dsl::parse_eval(src)?;
    let mut synth_lock = SYNTH.lock();
    let synth = synth_lock.as_mut().ok_or_else(|| String::from("Synth not initialized"))?;
    let saved_wf = synth.waveform;
    let saved_env = synth.envelope;
    let saved_filter = synth.filter;
    synth.waveform = bundle.pattern.waveform;
    synth.envelope = bundle.pattern.envelope;
    if bundle.controls.lpf_hz > 0 {
        synth.filter = synth::FilterSettings::explicit(
            synth::FilterMode::LowPass,
            bundle.controls.lpf_hz,
            ((bundle.controls.lpq.max(0) * 255) >> 16).clamp(0, 255) as u8,
        );
    }
    let samples = bundle.pattern.render(synth);
    synth.waveform = saved_wf;
    synth.envelope = saved_env;
    synth.filter = saved_filter;
    drop(synth_lock);

    let duration_ms = (samples.len() as u32) / (synth::SAMPLE_RATE * synth::CHANNELS / 1000);
    crate::drivers::hda::write_samples_and_play(&samples, duration_ms).map_err(String::from)?;
    Ok(())
}

/// Inspect a DSL source: returns a debug summary without playing it.
#[cfg(feature = "strudel")]
pub fn dsl_inspect(src: &str) -> Result<String, String> {
    let bundle = strudel_dsl::parse_eval(src)?;
    let c = &bundle.controls;
    let mut out = String::new();
    out.push_str(&format!(
        "wave={} | steps={} | bpm={}\n",
        bundle.pattern.waveform.short_name(),
        bundle.pattern.steps.len(),
        bundle.pattern.bpm,
    ));
    out.push_str(&format!(
        "gain=q{} detune={}c.q lpf={}Hz lpq=q{} hpf={}Hz bpf={}Hz\n",
        c.gain, c.detune_cents,
        c.lpf_hz, c.lpq,
        c.hpf_hz, c.bpf_hz,
    ));
    out.push_str(&format!(
        "lpenv=q{} lpa={}smp lpd={}smp lps=q{} room=q{} delay=q{} pan=q{} dist=q{} duck=q{} orbit={}\n",
        c.lpenv, c.lpa_samples, c.lpd_samples,
        c.lps_q, c.room, c.delay, c.pan, c.distort, c.duck_depth, c.orbit,
    ));
    out.push_str("(values in Q16.16 fixed-point; divide by 65536 to read as float)\n");
    Ok(out)
}
