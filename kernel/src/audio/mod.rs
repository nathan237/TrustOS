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

use spin::Mutex;
use alloc::string::String;
use alloc::format;

use synth::{SynthEngine, Waveform, Envelope};

/// Global synth engine instance
static SYNTH: Mutex<Option<SynthEngine>> = Mutex::new(None);

/// Initialize the audio subsystem (HDA driver + synth engine)
pub fn init() -> Result<(), &'static str> {
    // Ensure HDA driver is initialized
    if !crate::drivers::hda::is_initialized() {
        crate::drivers::hda::init()?;
    }

    // Create synth engine
    let engine = SynthEngine::new();
    *SYNTH.lock() = Some(engine);

    crate::serial_println!("[AUDIO] TrustSynth engine initialized");
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
