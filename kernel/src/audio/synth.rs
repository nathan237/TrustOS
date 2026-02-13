//! TrustSynth — Multi-waveform synthesizer engine
//!
//! Pure integer DSP (no floating-point) for bare-metal operation.
//! Features:
//!   - 5 waveforms: Sine, Square, Sawtooth, Triangle, Noise
//!   - ADSR envelope generator
//!   - Polyphonic voice engine (up to 8 simultaneous voices)
//!   - Phase accumulator with Q16.16 fixed-point precision
//!
//! Audio format: 48 kHz, 16-bit signed, stereo interleaved

use alloc::vec::Vec;
use alloc::string::String;
use alloc::format;
use super::tables::{SINE_TABLE, MIDI_FREQ};

/// Sample rate in Hz
pub const SAMPLE_RATE: u32 = 48000;
/// Number of audio channels (stereo)
pub const CHANNELS: u32 = 2;
/// Bytes per sample (16-bit)
pub const BYTES_PER_SAMPLE: u32 = 2;
/// Maximum simultaneous voices
pub const MAX_VOICES: usize = 8;
/// Fixed-point fractional bits for phase accumulator
const FRAC_BITS: u32 = 16;
/// Sine table size
const TABLE_SIZE: u32 = 256;
/// LFSR seed for noise generator
const LFSR_SEED: u16 = 0xACE1;

// ═══════════════════════════════════════════════════════════════════════════════
// Waveform Types
// ═══════════════════════════════════════════════════════════════════════════════

/// Available waveform shapes
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Waveform {
    Sine,
    Square,
    Sawtooth,
    Triangle,
    Noise,
}

impl Waveform {
    /// Parse from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "sine" | "sin" | "s" => Some(Waveform::Sine),
            "square" | "sq" | "q" => Some(Waveform::Square),
            "saw" | "sawtooth" | "w" => Some(Waveform::Sawtooth),
            "triangle" | "tri" | "t" => Some(Waveform::Triangle),
            "noise" | "n" => Some(Waveform::Noise),
            _ => None,
        }
    }

    /// Short display name
    pub fn short_name(&self) -> &'static str {
        match self {
            Waveform::Sine => "Sin",
            Waveform::Square => "Sqr",
            Waveform::Sawtooth => "Saw",
            Waveform::Triangle => "Tri",
            Waveform::Noise => "Noi",
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Waveform::Sine => "Sine",
            Waveform::Square => "Square",
            Waveform::Sawtooth => "Sawtooth",
            Waveform::Triangle => "Triangle",
            Waveform::Noise => "Noise",
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ADSR Envelope
// ═══════════════════════════════════════════════════════════════════════════════

/// Envelope state machine
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EnvState {
    Idle,
    Attack,
    Decay,
    Sustain,
    Release,
}

/// ADSR Envelope generator (all times in samples, levels in Q15)
#[derive(Debug, Clone, Copy)]
pub struct Envelope {
    /// Attack time in samples
    pub attack_samples: u32,
    /// Decay time in samples
    pub decay_samples: u32,
    /// Sustain level (0-32767, Q15)
    pub sustain_level: i32,
    /// Release time in samples
    pub release_samples: u32,
    /// Current state
    state: EnvState,
    /// Current output level (0-32767, Q15)
    level: i32,
    /// Counter within current state
    counter: u32,
}

impl Envelope {
    /// Create new envelope with times in milliseconds
    pub fn new(attack_ms: u32, decay_ms: u32, sustain_pct: u32, release_ms: u32) -> Self {
        let sustain = (sustain_pct.min(100) as i32 * 32767) / 100;
        Self {
            attack_samples: ms_to_samples(attack_ms),
            decay_samples: ms_to_samples(decay_ms),
            sustain_level: sustain,
            release_samples: ms_to_samples(release_ms),
            state: EnvState::Idle,
            level: 0,
            counter: 0,
        }
    }

    /// Default envelope: quick attack, medium release
    pub fn default_env() -> Self {
        Self::new(10, 50, 70, 100)
    }

    /// Organ-style: instant attack, full sustain, quick release
    pub fn organ() -> Self {
        Self::new(1, 1, 100, 10)
    }

    /// Pluck-style: instant attack, quick decay, no sustain
    pub fn pluck() -> Self {
        Self::new(2, 200, 0, 50)
    }

    /// Pad-style: slow attack, full sustain, long release
    pub fn pad() -> Self {
        Self::new(300, 100, 80, 500)
    }

    /// Trigger note-on
    pub fn note_on(&mut self) {
        self.state = EnvState::Attack;
        self.counter = 0;
        // Don't reset level — allows retriggering smoothly
    }

    /// Trigger note-off → start release
    pub fn note_off(&mut self) {
        if self.state != EnvState::Idle {
            self.state = EnvState::Release;
            self.counter = 0;
        }
    }

    /// Process one sample, return envelope value (0-32767)
    pub fn tick(&mut self) -> i32 {
        match self.state {
            EnvState::Idle => {
                self.level = 0;
            }
            EnvState::Attack => {
                if self.attack_samples == 0 {
                    self.level = 32767;
                    self.state = EnvState::Decay;
                    self.counter = 0;
                } else {
                    self.level = ((self.counter as i64 * 32767) / self.attack_samples as i64) as i32;
                    self.counter += 1;
                    if self.counter >= self.attack_samples {
                        self.level = 32767;
                        self.state = EnvState::Decay;
                        self.counter = 0;
                    }
                }
            }
            EnvState::Decay => {
                if self.decay_samples == 0 {
                    self.level = self.sustain_level;
                    self.state = EnvState::Sustain;
                } else {
                    let delta = 32767 - self.sustain_level;
                    self.level = 32767 - ((self.counter as i64 * delta as i64) / self.decay_samples as i64) as i32;
                    self.counter += 1;
                    if self.counter >= self.decay_samples {
                        self.level = self.sustain_level;
                        self.state = EnvState::Sustain;
                        self.counter = 0;
                    }
                }
            }
            EnvState::Sustain => {
                self.level = self.sustain_level;
                // Stay here until note_off
            }
            EnvState::Release => {
                if self.release_samples == 0 {
                    self.level = 0;
                    self.state = EnvState::Idle;
                } else {
                    let start_level = if self.counter == 0 { self.level } else {
                        // We need to store the level at the start of release
                        // Approximate: use sustain_level (most common case)
                        self.sustain_level
                    };
                    self.level = start_level - ((self.counter as i64 * start_level as i64) / self.release_samples as i64) as i32;
                    if self.level < 0 { self.level = 0; }
                    self.counter += 1;
                    if self.counter >= self.release_samples {
                        self.level = 0;
                        self.state = EnvState::Idle;
                    }
                }
            }
        }
        self.level
    }

    /// Is the envelope finished (idle)?
    pub fn is_idle(&self) -> bool {
        self.state == EnvState::Idle
    }

    /// Get current state name
    pub fn state_name(&self) -> &'static str {
        match self.state {
            EnvState::Idle => "Idle",
            EnvState::Attack => "Atk",
            EnvState::Decay => "Dec",
            EnvState::Sustain => "Sus",
            EnvState::Release => "Rel",
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Oscillator
// ═══════════════════════════════════════════════════════════════════════════════

/// Single oscillator with phase accumulator
#[derive(Debug, Clone)]
pub struct Oscillator {
    /// Waveform type
    pub waveform: Waveform,
    /// Phase accumulator (Q16.16 fixed-point)
    phase: u32,
    /// Phase increment per sample (Q16.16)
    phase_inc: u32,
    /// Current frequency in Hz
    pub freq_hz: u32,
    /// LFSR state for noise generation
    lfsr: u16,
}

impl Oscillator {
    /// Create a new oscillator
    pub fn new(waveform: Waveform, freq_hz: u32) -> Self {
        let phase_inc = Self::calc_phase_inc(freq_hz);
        Self {
            waveform,
            phase: 0,
            phase_inc,
            freq_hz,
            lfsr: LFSR_SEED,
        }
    }

    /// Calculate phase increment for a given frequency
    /// phase_inc = (freq × TABLE_SIZE << FRAC_BITS) / SAMPLE_RATE
    fn calc_phase_inc(freq_hz: u32) -> u32 {
        // Use 64-bit to avoid overflow: freq can be up to ~12kHz
        ((freq_hz as u64 * (TABLE_SIZE as u64) << FRAC_BITS) / SAMPLE_RATE as u64) as u32
    }

    /// Set frequency
    pub fn set_freq(&mut self, freq_hz: u32) {
        self.freq_hz = freq_hz;
        self.phase_inc = Self::calc_phase_inc(freq_hz);
    }

    /// Set frequency from MIDI note
    pub fn set_midi_note(&mut self, note: u8) {
        let freq = MIDI_FREQ[note.min(127) as usize];
        self.set_freq(freq);
    }

    /// Reset phase to 0
    pub fn reset_phase(&mut self) {
        self.phase = 0;
    }

    /// Generate one sample (returns Q15 value: -32767 to +32767)
    pub fn tick(&mut self) -> i16 {
        let sample = match self.waveform {
            Waveform::Sine => self.gen_sine(),
            Waveform::Square => self.gen_square(),
            Waveform::Sawtooth => self.gen_sawtooth(),
            Waveform::Triangle => self.gen_triangle(),
            Waveform::Noise => self.gen_noise(),
        };

        // Advance phase
        self.phase = self.phase.wrapping_add(self.phase_inc);

        sample
    }

    /// Sine wave via table lookup with linear interpolation
    fn gen_sine(&self) -> i16 {
        let table_idx = (self.phase >> FRAC_BITS) as usize & 0xFF;
        let frac = (self.phase & 0xFFFF) as i32;

        let s0 = SINE_TABLE[table_idx] as i32;
        let s1 = SINE_TABLE[(table_idx + 1) & 0xFF] as i32;

        // Linear interpolation: s0 + (s1 - s0) × frac / 65536
        let interp = s0 + ((s1 - s0) * frac >> 16);
        interp as i16
    }

    /// Square wave: +max for first half, -max for second half
    fn gen_square(&self) -> i16 {
        let table_pos = (self.phase >> FRAC_BITS) & 0xFF;
        if table_pos < 128 { 24000 } else { -24000 }
    }

    /// Sawtooth: linear ramp from -max to +max over one period
    fn gen_sawtooth(&self) -> i16 {
        // phase goes 0→(256<<16), map to -32767→+32767
        let pos = (self.phase >> FRAC_BITS) & 0xFF;
        // pos: 0→255 → sample: -24000→+24000
        ((pos as i32 * 48000 / 256) - 24000) as i16
    }

    /// Triangle wave: ramp up then down
    fn gen_triangle(&self) -> i16 {
        let pos = (self.phase >> FRAC_BITS) & 0xFF;
        if pos < 128 {
            // Rising: 0→127 → -24000→+24000
            ((pos as i32 * 48000 / 128) - 24000) as i16
        } else {
            // Falling: 128→255 → +24000→-24000
            (((255 - pos) as i32 * 48000 / 128) - 24000) as i16
        }
    }

    /// White noise via 16-bit LFSR (Galois)
    fn gen_noise(&mut self) -> i16 {
        // Galois LFSR with taps at bits 16, 14, 13, 11
        let bit = self.lfsr & 1;
        self.lfsr >>= 1;
        if bit == 1 {
            self.lfsr ^= 0xB400; // Polynomial
        }
        // Map LFSR (0-65535) to audio range
        (self.lfsr as i16).wrapping_mul(3) / 4 // Scale down a bit
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Voice — one note being played (oscillator + envelope)
// ═══════════════════════════════════════════════════════════════════════════════

/// A single synthesis voice (oscillator + envelope)
#[derive(Debug, Clone)]
pub struct Voice {
    pub osc: Oscillator,
    pub env: Envelope,
    /// MIDI note number (for identification)
    pub note: u8,
    /// Velocity (0-127)
    pub velocity: u8,
    /// Is this voice active?
    pub active: bool,
}

impl Voice {
    pub fn new() -> Self {
        Self {
            osc: Oscillator::new(Waveform::Sine, 440),
            env: Envelope::default_env(),
            note: 0,
            velocity: 0,
            active: false,
        }
    }

    /// Start playing a note
    pub fn note_on(&mut self, note: u8, velocity: u8, waveform: Waveform, envelope: Envelope) {
        let freq = MIDI_FREQ[note.min(127) as usize];
        self.osc = Oscillator::new(waveform, freq);
        self.env = envelope;
        self.env.note_on();
        self.note = note;
        self.velocity = velocity;
        self.active = true;
    }

    /// Release the note
    pub fn note_off(&mut self) {
        self.env.note_off();
    }

    /// Generate one sample
    pub fn tick(&mut self) -> i16 {
        if !self.active {
            return 0;
        }

        let env_level = self.env.tick();
        if self.env.is_idle() {
            self.active = false;
            return 0;
        }

        let raw = self.osc.tick() as i32;
        let vel_scale = self.velocity as i32;

        // sample = oscillator × envelope × velocity
        // raw is Q15 (±32767), env is Q15 (0-32767), velocity is 0-127
        // result = raw × env / 32767 × vel / 127
        let sample = (raw * env_level / 32767) * vel_scale / 127;
        sample.clamp(-32767, 32767) as i16
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Synth Engine — manages multiple voices
// ═══════════════════════════════════════════════════════════════════════════════

/// The main synthesizer engine
pub struct SynthEngine {
    /// Voice pool
    pub voices: [Voice; MAX_VOICES],
    /// Default waveform for new notes
    pub waveform: Waveform,
    /// Default envelope for new notes
    pub envelope: Envelope,
    /// Master volume (0-255)
    pub master_volume: u8,
}

impl SynthEngine {
    /// Create a new synth engine
    pub fn new() -> Self {
        let voices = core::array::from_fn(|_| Voice::new());
        Self {
            voices,
            waveform: Waveform::Sine,
            envelope: Envelope::default_env(),
            master_volume: 200,
        }
    }

    /// Set the default waveform
    pub fn set_waveform(&mut self, wf: Waveform) {
        self.waveform = wf;
    }

    /// Set the default envelope (ADSR in ms, sustain in %)
    pub fn set_adsr(&mut self, attack_ms: u32, decay_ms: u32, sustain_pct: u32, release_ms: u32) {
        self.envelope = Envelope::new(attack_ms, decay_ms, sustain_pct, release_ms);
    }

    /// Play a note (finds a free voice or steals the oldest)
    pub fn note_on(&mut self, note: u8, velocity: u8) {
        // Find a free voice
        let voice_idx = self.find_free_voice();
        let voice = &mut self.voices[voice_idx];
        voice.note_on(note, velocity, self.waveform, self.envelope);
    }

    /// Release a note (by MIDI note number)
    pub fn note_off(&mut self, note: u8) {
        for voice in &mut self.voices {
            if voice.active && voice.note == note {
                voice.note_off();
                break;
            }
        }
    }

    /// Release all notes
    pub fn all_notes_off(&mut self) {
        for voice in &mut self.voices {
            voice.note_off();
        }
    }

    /// Generate audio samples into a buffer (stereo interleaved i16)
    /// Returns the number of samples written (per channel)
    pub fn render(&mut self, buffer: &mut [i16], num_samples: usize) -> usize {
        let to_render = num_samples.min(buffer.len() / 2); // stereo

        for i in 0..to_render {
            // Mix all active voices
            let mut mix: i32 = 0;
            for voice in &mut self.voices {
                if voice.active {
                    mix += voice.tick() as i32;
                }
            }

            // Apply master volume
            mix = mix * self.master_volume as i32 / 255;

            // Clamp to 16-bit range
            let sample = mix.clamp(-32767, 32767) as i16;

            // Write stereo (same on both channels for now)
            buffer[i * 2] = sample;
            buffer[i * 2 + 1] = sample;
        }

        to_render
    }

    /// Generate a fixed-duration note and return the samples
    pub fn render_note(&mut self, note: u8, velocity: u8, duration_ms: u32) -> Vec<i16> {
        let total_samples = ms_to_samples(duration_ms) as usize;
        // Add release tail
        let release_samples = self.envelope.release_samples as usize;
        let full_samples = total_samples + release_samples;
        let mut buffer = alloc::vec![0i16; full_samples * 2]; // stereo

        // Note on
        self.note_on(note, velocity);

        // Render note-on portion
        self.render(&mut buffer[..total_samples * 2], total_samples);

        // Note off (start release)
        self.note_off(note);

        // Render release tail
        if release_samples > 0 {
            self.render(&mut buffer[total_samples * 2..], release_samples);
        }

        buffer
    }

    /// Play a note by name, e.g., "C4", "A#3"
    pub fn play_note_by_name(&mut self, name: &str, duration_ms: u32) -> Result<Vec<i16>, &'static str> {
        let midi_note = super::tables::note_name_to_midi(name)
            .ok_or("Invalid note name (use e.g. C4, A#3, Bb5)")?;
        Ok(self.render_note(midi_note, 100, duration_ms))
    }

    /// Render a tone at a specific frequency (Hz) for a given duration
    pub fn render_freq(&mut self, freq_hz: u32, duration_ms: u32) -> Vec<i16> {
        let total_samples = ms_to_samples(duration_ms) as usize;
        let release_samples = self.envelope.release_samples as usize;
        let full_samples = total_samples + release_samples;
        let mut buffer = alloc::vec![0i16; full_samples * 2]; // stereo

        // Setup a voice manually with exact frequency
        let voice_idx = self.find_free_voice();
        let voice = &mut self.voices[voice_idx];
        voice.osc = Oscillator::new(self.waveform, freq_hz);
        voice.env = self.envelope;
        voice.env.note_on();
        voice.note = 69; // placeholder
        voice.velocity = 100;
        voice.active = true;

        // Render
        self.render(&mut buffer[..total_samples * 2], total_samples);
        // Release
        self.voices[voice_idx].note_off();
        if release_samples > 0 {
            self.render(&mut buffer[total_samples * 2..], release_samples);
        }

        buffer
    }

    /// Get info about active voices
    pub fn active_voice_count(&self) -> usize {
        self.voices.iter().filter(|v| v.active).count()
    }

    /// Get synth status string
    pub fn status(&self) -> String {
        let mut s = String::new();
        s.push_str(&format!("TrustSynth Engine\n"));
        s.push_str(&format!("  Waveform: {}\n", self.waveform.name()));
        s.push_str(&format!("  ADSR: A={}ms D={}ms S={}% R={}ms\n",
            samples_to_ms(self.envelope.attack_samples),
            samples_to_ms(self.envelope.decay_samples),
            self.envelope.sustain_level * 100 / 32767,
            samples_to_ms(self.envelope.release_samples)
        ));
        s.push_str(&format!("  Master Volume: {}/255\n", self.master_volume));
        s.push_str(&format!("  Active Voices: {}/{}\n", self.active_voice_count(), MAX_VOICES));
        for (i, v) in self.voices.iter().enumerate() {
            if v.active {
                let note_name = super::tables::midi_to_note_name(v.note);
                let octave = super::tables::midi_octave(v.note);
                s.push_str(&format!("    Voice {}: {}{} vel={} env={} wf={}\n",
                    i, note_name, octave, v.velocity, v.env.state_name(), v.osc.waveform.short_name()));
            }
        }
        s
    }

    // ─── Internal helpers ─────────────────────────────────────────────────

    /// Find a free voice, or steal the one with lowest envelope level
    fn find_free_voice(&self) -> usize {
        // First: find an idle voice
        for (i, v) in self.voices.iter().enumerate() {
            if !v.active {
                return i;
            }
        }
        // Second: find the voice in Release with lowest level
        let mut best = 0;
        let mut best_level = i32::MAX;
        for (i, v) in self.voices.iter().enumerate() {
            if v.env.state == EnvState::Release && (v.env.level as i32) < best_level {
                best_level = v.env.level as i32;
                best = i;
            }
        }
        if best_level < i32::MAX {
            return best;
        }
        // Last resort: steal voice 0
        0
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Utility functions
// ═══════════════════════════════════════════════════════════════════════════════

/// Convert milliseconds to samples
pub fn ms_to_samples(ms: u32) -> u32 {
    (SAMPLE_RATE * ms) / 1000
}

/// Convert samples to milliseconds
pub fn samples_to_ms(samples: u32) -> u32 {
    (samples * 1000) / SAMPLE_RATE
}
