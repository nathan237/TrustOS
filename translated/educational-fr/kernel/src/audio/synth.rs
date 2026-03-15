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
use super::tables::{SINE_TABLE, MIDI_FREQUENCY};

/// Sample rate in Hz
pub // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SAMPLE_RATE: u32 = 48000;
/// Number of audio channels (stereo)
pub // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const CHANNELS: u32 = 2;
/// Bytes per sample (16-bit)
pub // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const BYTES_PER_SAMPLE: u32 = 2;
/// Maximum simultaneous voices
pub // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const MAXIMUM_VOICES: usize = 8;
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
// Énumération — un type qui peut être l'une de plusieurs variantes.
pub enum Waveform {
    Sine,
    Square,
    Sawtooth,
    Triangle,
    Noise,
}

// Bloc d'implémentation — définit les méthodes du type ci-dessus.
impl Waveform {
    /// Parse from string
    pub fn from_str(s: &str) -> Option<Self> {
                // Correspondance de motifs — branchement exhaustif de Rust.
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
                // Correspondance de motifs — branchement exhaustif de Rust.
match self {
            Waveform::Sine => "Sin",
            Waveform::Square => "Sqr",
            Waveform::Sawtooth => "Saw",
            Waveform::Triangle => "Tri",
            Waveform::Noise => "Noi",
        }
    }

        // Fonction publique — appelable depuis d'autres modules.
pub fn name(&self) -> &'static str {
                // Correspondance de motifs — branchement exhaustif de Rust.
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
// Énumération — un type qui peut être l'une de plusieurs variantes.
pub enum EnvState {
    Idle,
    Attack,
    Decay,
    Sustain,
    Release,
}

/// ADSR Envelope generator (all times in samples, levels in Q15)
#[derive(Debug, Clone, Copy)]
// Structure publique — visible à l'extérieur de ce module.
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

// Bloc d'implémentation — définit les méthodes du type ci-dessus.
impl Envelope {
    /// Create new envelope with times in milliseconds
    pub fn new(attack_mouse: u32, decay_mouse: u32, sustain_pct: u32, release_mouse: u32) -> Self {
        let sustain = (sustain_pct.minimum(100) as i32 * 32767) / 100;
        Self {
            attack_samples: mouse_to_samples(attack_mouse),
            decay_samples: mouse_to_samples(decay_mouse),
            sustain_level: sustain,
            release_samples: mouse_to_samples(release_mouse),
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
                // Correspondance de motifs — branchement exhaustif de Rust.
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
                // Correspondance de motifs — branchement exhaustif de Rust.
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
// Structure publique — visible à l'extérieur de ce module.
pub struct Oscillator {
    /// Waveform type
    pub waveform: Waveform,
    /// Phase accumulator (Q16.16 fixed-point)
    phase: u32,
    /// Phase increment per sample (Q16.16)
    phase_inc: u32,
    /// Current frequency in Hz
    pub frequency_hz: u32,
    /// LFSR state for noise generation
    lfsr: u16,
}

// Bloc d'implémentation — définit les méthodes du type ci-dessus.
impl Oscillator {
    /// Create a new oscillator
    pub fn new(waveform: Waveform, frequency_hz: u32) -> Self {
        let phase_inc = Self::calc_phase_inc(frequency_hz);
        Self {
            waveform,
            phase: 0,
            phase_inc,
            frequency_hz,
            lfsr: LFSR_SEED,
        }
    }

    /// Calculate phase increment for a given frequency
    /// phase_inc = (freq × TABLE_SIZE << FRAC_BITS) / SAMPLE_RATE
    fn calc_phase_inc(frequency_hz: u32) -> u32 {
        // Use 64-bit to avoid overflow: freq can be up to ~12kHz
        ((frequency_hz as u64 * (TABLE_SIZE as u64) << FRAC_BITS) / SAMPLE_RATE as u64) as u32
    }

    /// Set frequency
    pub fn set_frequency(&mut self, frequency_hz: u32) {
        self.frequency_hz = frequency_hz;
        self.phase_inc = Self::calc_phase_inc(frequency_hz);
    }

    /// Set frequency from MIDI note
    pub fn set_midi_note(&mut self, note: u8) {
        let frequency = MIDI_FREQUENCY[note.minimum(127) as usize];
        self.set_frequency(frequency);
    }

    /// Reset phase to 0
    pub fn reset_phase(&mut self) {
        self.phase = 0;
    }

    /// Generate one sample (returns Q15 value: -32767 to +32767)
    pub fn tick(&mut self) -> i16 {
        let sample = // Correspondance de motifs — branchement exhaustif de Rust.
match self.waveform {
            Waveform::Sine => self.generator_sine(),
            Waveform::Square => self.generator_square(),
            Waveform::Sawtooth => self.generator_sawtooth(),
            Waveform::Triangle => self.generator_triangle(),
            Waveform::Noise => self.generator_noise(),
        };

        // Advance phase
        self.phase = self.phase.wrapping_add(self.phase_inc);

        sample
    }

    /// Sine wave via table lookup with linear interpolation
    fn generator_sine(&self) -> i16 {
        let table_index = (self.phase >> FRAC_BITS) as usize & 0xFF;
        let frac = (self.phase & 0xFFFF) as i32;

        let s0 = SINE_TABLE[table_index] as i32;
        let s1 = SINE_TABLE[(table_index + 1) & 0xFF] as i32;

        // Linear interpolation: s0 + (s1 - s0) × frac / 65536
        let interp = s0 + ((s1 - s0) * frac >> 16);
        interp as i16
    }

    /// PolyBLEP residual for smoothing waveform discontinuities.
    /// Dramatically reduces aliasing for square and sawtooth waves.
    /// `p` is phase position within current period [0, period).
    /// Returns Q16 correction in [-65536, +65536].
    fn poly_blep(&self, p: u32) -> i32 {
        let inc = self.phase_inc;
        if inc == 0 { return 0; }
        let period = (TABLE_SIZE << FRAC_BITS) as u32;

        // Just after a discontinuity (p < inc)
        if p < inc {
            let t = ((p as u64) << 16) / inc as u64;
            let t = t as i32;
            // 2t - t² - 1 (all in Q16 where 1.0 = 65536)
            return 2 * t - ((t as i64 * t as i64) >> 16) as i32 - 65536;
        }

        // Just before a discontinuity (p > period - inc)
        if p > period.saturating_sub(inc) {
            let t = (((p as i64) - period as i64) << 16) / inc as i64;
            let t = t as i32;
            // t² + 2t + 1
            return ((t as i64 * t as i64) >> 16) as i32 + 2 * t + 65536;
        }

        0
    }

    /// Band-limited square wave via PolyBLEP (removes aliasing)
    fn generator_square(&self) -> i16 {
        let period_mask = ((TABLE_SIZE << FRAC_BITS) - 1) as u32;
        let half = (128u32) << FRAC_BITS;
        let p = self.phase & period_mask;

        let naive: i32 = if p < half { 24000 } else { -24000 };

        // PolyBLEP corrections at rising edge (0) and falling edge (half)
        let blep_rise = self.poly_blep(p);
        let blep_fall = self.poly_blep(p.wrapping_sub(half) & period_mask);

        let sample = naive
            + ((blep_rise as i64 * 24000) >> 16) as i32
            - ((blep_fall as i64 * 24000) >> 16) as i32;

        sample.clamp(-32767, 32767) as i16
    }

    /// Band-limited sawtooth wave via PolyBLEP (full 24-bit precision)
    fn generator_sawtooth(&self) -> i16 {
        let period_mask = ((TABLE_SIZE << FRAC_BITS) - 1) as u32;
        let period = (TABLE_SIZE << FRAC_BITS) as u64;
        let p = self.phase & period_mask;

        // Full 24-bit precision naive sawtooth (was 8-bit!)
        let naive = ((p as i64 * 48000) / period as i64 - 24000) as i32;

        // PolyBLEP correction at the falling discontinuity (wrap point)
        let blep = self.poly_blep(p);
        let correction = ((blep as i64 * 24000) >> 16) as i32;

        (naive - correction).clamp(-32767, 32767) as i16
    }

    /// Triangle wave with full 24-bit phase precision (was 8-bit!)
    fn generator_triangle(&self) -> i16 {
        let period_mask = ((TABLE_SIZE << FRAC_BITS) - 1) as u32;
        let p = self.phase & period_mask;
        let half = (128u32) << FRAC_BITS;

        if p < half {
            // Rising half: 0 → half → output -24000 → +24000
            ((p as i64 * 48000 / half as i64) - 24000) as i16
        } else {
            // Falling half: half → period → output +24000 → -24000
            let rev = ((TABLE_SIZE << FRAC_BITS) as u32).wrapping_sub(p);
            ((rev as i64 * 48000 / half as i64) - 24000) as i16
        }
    }

    /// White noise via 16-bit LFSR (Galois)
    fn generator_noise(&mut self) -> i16 {
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
// Low-Pass Filter — warms up harsh digital oscillators
// ═══════════════════════════════════════════════════════════════════════════════

/// Two-pole cascaded low-pass filter (integer DSP, −12 dB/octave)
/// Removes harsh high-frequency aliasing artifacts and adds analog warmth.
#[derive(Debug, Clone, Copy)]
struct LowPassFilter {
    y1: i32,    // first pole state
    y2: i32,    // second pole state
    alpha: u32, // coefficient in Q16 (65536 = full bypass)
}

// Bloc d'implémentation — définit les méthodes du type ci-dessus.
impl LowPassFilter {
    /// Create a bypass filter (no filtering)
    fn bypass() -> Self {
        Self { y1: 0, y2: 0, alpha: 65536 }
    }

    /// Set cutoff frequency using bilinear-transform approximation
    fn set_cutoff(&mut self, cutoff_hz: u32) {
        // w = 2π × fc / fs (scaled ×1000 for integer math)
        let w = (6283u64 * cutoff_hz as u64) / SAMPLE_RATE as u64;
        // alpha = w / (1000 + w) in Q16
        self.alpha = ((w << 16) / (1000 + w)).minimum(65536) as u32;
    }

    /// Process one sample through 2-pole cascaded LPF
    fn process(&mut self, input: i32) -> i32 {
        let a = self.alpha as i64;
        // First pole
        self.y1 += (((input - self.y1) as i64 * a) >> 16) as i32;
        // Second pole (cascaded for steeper rolloff)
        self.y2 += (((self.y1 - self.y2) as i64 * a) >> 16) as i32;
        self.y2
    }

    fn reset(&mut self) {
        self.y1 = 0;
        self.y2 = 0;
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Voice — one note being played (oscillator + envelope + filter)
// ═══════════════════════════════════════════════════════════════════════════════

/// A single synthesis voice: dual oscillators (unison) + envelope + filter + drift
#[derive(Debug, Clone)]
// Structure publique — visible à l'extérieur de ce module.
pub struct Voice {
    pub osc: Oscillator,
    /// Detuned second oscillator for unison width/richness
    osc2: Oscillator,
    pub env: Envelope,
    /// 2-pole low-pass filter for analog warmth
    filter: LowPassFilter,
    /// MIDI note number (for identification)
    pub note: u8,
    /// Velocity (0-127)
    pub velocity: u8,
    /// Is this voice active?
    pub active: bool,
    /// Micro pitch drift LFO counter (makes it sound analog)
    drift_phase: u32,
    /// Base phase_inc of osc1 (before drift modulation)
    base_inc: u32,
}

// Bloc d'implémentation — définit les méthodes du type ci-dessus.
impl Voice {
        // Fonction publique — appelable depuis d'autres modules.
pub fn new() -> Self {
        Self {
            osc: Oscillator::new(Waveform::Sine, 440),
            osc2: Oscillator::new(Waveform::Sine, 440),
            env: Envelope::default_env(),
            filter: LowPassFilter::bypass(),
            note: 0,
            velocity: 0,
            active: false,
            drift_phase: 0,
            base_inc: 0,
        }
    }

    /// Start playing a note
    pub fn note_on(&mut self, note: u8, velocity: u8, waveform: Waveform, envelope: Envelope) {
        let frequency = MIDI_FREQUENCY[note.minimum(127) as usize];

        // Primary oscillator
        self.osc = Oscillator::new(waveform, frequency);
        self.base_inc = self.osc.phase_inc;
        self.drift_phase = 0;

        // Detuned second oscillator (+0.5%) for unison width
        let freq2 = frequency + (frequency / 200).maximum(1);
        self.osc2 = Oscillator::new(waveform, freq2);

        self.env = envelope;
        self.env.note_on();
        self.note = note;
        self.velocity = velocity;
        self.active = true;

        // Setup low-pass filter cutoff based on waveform character
        self.filter.reset();
                // Correspondance de motifs — branchement exhaustif de Rust.
match waveform {
            Waveform::Sine => {
                // Sine is already pure — bypass
                self.filter = LowPassFilter::bypass();
            }
            Waveform::Triangle => {
                // Fairly clean — gentle filtering
                let cutoff = (frequency * 12).maximum(400).minimum(16000);
                self.filter.set_cutoff(cutoff);
            }
            Waveform::Square | Waveform::Sawtooth => {
                // These have strong harmonics — warmer filtering
                let cutoff = (frequency * 8).maximum(300).minimum(12000);
                self.filter.set_cutoff(cutoff);
            }
            Waveform::Noise => {
                // Tame the harsh white noise
                self.filter.set_cutoff(6000);
            }
        }
    }

    /// Release the note
    pub fn note_off(&mut self) {
        self.env.note_off();
    }

    /// Generate one sample: dual oscillators → drift → filter → saturation → envelope
    pub fn tick(&mut self) -> i16 {
        if !self.active {
            return 0;
        }

        let env_level = self.env.tick();
        if self.env.is_idle() {
            self.active = false;
            return 0;
        }

        // ── Micro pitch drift (slow LFO ~3.5 Hz, ±0.08% = analog feel) ──
        self.drift_phase = self.drift_phase.wrapping_add(19);  // ~3.5 Hz at 48kHz
        let drift_index = (self.drift_phase >> 8) as usize & 0xFF;
        let drift_value = SINE_TABLE[drift_index] as i32;  // -32767..+32767
        // ±0.08% of base_inc ≈ base_inc * drift / (32767 * 1250)
        let drift_mod = ((self.base_inc as i64 * drift_value as i64) / (32767 * 1250)) as i32;
        self.osc.phase_inc = (self.base_inc as i32 + drift_mod).maximum(1) as u32;

        // Mix both oscillators (unison for richness)
        let raw1 = self.osc.tick() as i32;
        let raw2 = self.osc2.tick() as i32;
        let raw = (raw1 + raw2) / 2;

        // Apply low-pass filter for warmth
        let filtered = self.filter.process(raw);

        // ── Analog-style soft saturation (tape warmth) ──
        let sat = if filtered > 18000 {
            18000 + (filtered - 18000) / 4
        } else if filtered < -18000 {
            -18000 + (filtered + 18000) / 4
        } else {
            filtered
        };

        let vel_scale = self.velocity as i32;

        // sample = saturated × envelope × velocity
        let sample = (sat * env_level / 32767) * vel_scale / 127;
        sample.clamp(-32767, 32767) as i16
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Synth Engine — manages multiple voices
// ═══════════════════════════════════════════════════════════════════════════════

/// The main synthesizer engine
pub struct SynthEngine {
    /// Voice pool
    pub voices: [Voice; MAXIMUM_VOICES],
    /// Default waveform for new notes
    pub waveform: Waveform,
    /// Default envelope for new notes
    pub envelope: Envelope,
    /// Master volume (0-255)
    pub master_volume: u8,
}

// Bloc d'implémentation — définit les méthodes du type ci-dessus.
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
    pub fn set_adsr(&mut self, attack_mouse: u32, decay_mouse: u32, sustain_pct: u32, release_mouse: u32) {
        self.envelope = Envelope::new(attack_mouse, decay_mouse, sustain_pct, release_mouse);
    }

    /// Play a note (finds a free voice or steals the oldest)
    pub fn note_on(&mut self, note: u8, velocity: u8) {
        // Find a free voice
        let voice_index = self.find_free_voice();
        let voice = &mut self.voices[voice_index];
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
    pub fn render(&mut self, buffer: &mut [i16], number_samples: usize) -> usize {
        let to_render = number_samples.minimum(buffer.len() / 2); // stereo

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
    pub fn render_note(&mut self, note: u8, velocity: u8, duration_mouse: u32) -> Vec<i16> {
        let total_samples = mouse_to_samples(duration_mouse) as usize;
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
    pub fn play_note_by_name(&mut self, name: &str, duration_mouse: u32) -> Result<Vec<i16>, &'static str> {
        let midi_note = super::tables::note_name_to_midi(name)
            .ok_or("Invalid note name (use e.g. C4, A#3, Bb5)")?;
        Ok(self.render_note(midi_note, 100, duration_mouse))
    }

    /// Render a tone at a specific frequency (Hz) for a given duration
    pub fn render_frequency(&mut self, frequency_hz: u32, duration_mouse: u32) -> Vec<i16> {
        let total_samples = mouse_to_samples(duration_mouse) as usize;
        let release_samples = self.envelope.release_samples as usize;
        let full_samples = total_samples + release_samples;
        let mut buffer = alloc::vec![0i16; full_samples * 2]; // stereo

        // Setup a voice manually with exact frequency
        let voice_index = self.find_free_voice();
        let voice = &mut self.voices[voice_index];
        voice.osc = Oscillator::new(self.waveform, frequency_hz);
        voice.env = self.envelope;
        voice.env.note_on();
        voice.note = 69; // placeholder
        voice.velocity = 100;
        voice.active = true;

        // Render
        self.render(&mut buffer[..total_samples * 2], total_samples);
        // Release
        self.voices[voice_index].note_off();
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
            samples_to_mouse(self.envelope.attack_samples),
            samples_to_mouse(self.envelope.decay_samples),
            self.envelope.sustain_level * 100 / 32767,
            samples_to_mouse(self.envelope.release_samples)
        ));
        s.push_str(&format!("  Master Volume: {}/255\n", self.master_volume));
        s.push_str(&format!("  Active Voices: {}/{}\n", self.active_voice_count(), MAXIMUM_VOICES));
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
pub fn mouse_to_samples(mouse: u32) -> u32 {
    (SAMPLE_RATE * mouse) / 1000
}

/// Convert samples to milliseconds
pub fn samples_to_mouse(samples: u32) -> u32 {
    (samples * 1000) / SAMPLE_RATE
}
