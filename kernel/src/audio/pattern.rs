//! Pattern Sequencer for TrustSynth
//!
//! A pattern is a sequence of steps, each containing an optional note.
//! Patterns play in a loop at a configurable BPM.
//!
//! Architecture:
//!   Pattern → Steps → SynthEngine.render → DMA buffer → HDA playback

use alloc::vec::Vec;
use alloc::vec;
use alloc::string::String;
use alloc::format;

use super::synth::{Waveform, Envelope, SynthEngine, SAMPLE_RATE, CHANNELS};
use super::tables;

// ═══════════════════════════════════════════════════════════════════════════════
// Constants
// ═══════════════════════════════════════════════════════════════════════════════

/// Maximum number of stored patterns
pub const MAX_PATTERNS: usize = 16;
/// Maximum steps per pattern
pub const MAX_STEPS: usize = 64;
/// Default BPM
pub const DEFAULT_BPM: u16 = 120;
/// Steps per beat (sixteenth notes = 4 steps per beat)
pub const STEPS_PER_BEAT: u32 = 4;

// ═══════════════════════════════════════════════════════════════════════════════
// Step — one slot in a pattern
// ═══════════════════════════════════════════════════════════════════════════════

/// A single step in a pattern
#[derive(Debug, Clone, Copy)]
pub struct Step {
    /// MIDI note (0-127), 255 = rest/silence
    pub note: u8,
    /// Velocity (0-127), 0 = silent
    pub velocity: u8,
    /// Waveform override (None = use pattern default)
    pub waveform: Option<Waveform>,
}

impl Step {
    /// Empty/silent step
    pub fn rest() -> Self {
        Self { note: 255, velocity: 0, waveform: None }
    }

    /// Note step with default velocity
    pub fn note(midi_note: u8) -> Self {
        Self { note: midi_note, velocity: 100, waveform: None }
    }

    /// Note step with custom velocity
    pub fn note_vel(midi_note: u8, velocity: u8) -> Self {
        Self { note: midi_note, velocity, waveform: None }
    }

    /// Note step with custom waveform
    pub fn note_wf(midi_note: u8, velocity: u8, wf: Waveform) -> Self {
        Self { note: midi_note, velocity, waveform: Some(wf) }
    }

    /// Is this step a rest?
    pub fn is_rest(&self) -> bool {
        self.note == 255 || self.velocity == 0
    }

    /// Display name for the note
    pub fn display(&self) -> String {
        if self.is_rest() {
            String::from("--")
        } else {
            let name = tables::midi_to_note_name(self.note);
            let oct = tables::midi_octave(self.note);
            format!("{}{}", name, oct)
        }
    }

    /// Waveform short name
    pub fn wave_display(&self) -> &'static str {
        match self.waveform {
            Some(wf) => wf.short_name(),
            None => "..",
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Pattern — a loop of steps
// ═══════════════════════════════════════════════════════════════════════════════

/// A pattern is a named loop of steps at a given BPM
#[derive(Clone)]
pub struct Pattern {
    /// Pattern name (stored as bytes for no_std)
    pub name: [u8; 16],
    /// Name length
    pub name_len: usize,
    /// Steps in this pattern
    pub steps: Vec<Step>,
    /// Tempo in BPM
    pub bpm: u16,
    /// Default waveform for steps without override
    pub waveform: Waveform,
    /// Default envelope
    pub envelope: Envelope,
}

impl Pattern {
    /// Create a new empty pattern
    pub fn new(name: &str, num_steps: usize, bpm: u16) -> Self {
        let n = num_steps.min(MAX_STEPS).max(1);
        let mut name_buf = [0u8; 16];
        let name_bytes = name.as_bytes();
        let len = name_bytes.len().min(16);
        name_buf[..len].copy_from_slice(&name_bytes[..len]);

        Self {
            name: name_buf,
            name_len: len,
            steps: vec![Step::rest(); n],
            bpm,
            waveform: Waveform::Square,
            envelope: Envelope::pluck(),
        }
    }

    /// Get pattern name as &str
    pub fn name_str(&self) -> &str {
        core::str::from_utf8(&self.name[..self.name_len]).unwrap_or("???")
    }

    /// Number of steps
    pub fn len(&self) -> usize {
        self.steps.len()
    }

    /// Set a step by index
    pub fn set_step(&mut self, idx: usize, step: Step) {
        if idx < self.steps.len() {
            self.steps[idx] = step;
        }
    }

    /// Set a note at a step by name (e.g., "C4")
    pub fn set_note(&mut self, idx: usize, note_name: &str) -> Result<(), &'static str> {
        if idx >= self.steps.len() {
            return Err("Step index out of range");
        }
        if note_name == "--" || note_name == "." || note_name.is_empty() {
            self.steps[idx] = Step::rest();
            return Ok(());
        }
        let midi = tables::note_name_to_midi(note_name)
            .ok_or("Invalid note name")?;
        self.steps[idx] = Step::note(midi);
        Ok(())
    }

    /// Calculate duration of one step in samples
    pub fn step_duration_samples(&self) -> u32 {
        // step_duration = 60 * SAMPLE_RATE / (BPM * STEPS_PER_BEAT)
        (60 * SAMPLE_RATE) / (self.bpm as u32 * STEPS_PER_BEAT)
    }

    /// Calculate duration of one step in milliseconds
    pub fn step_duration_ms(&self) -> u32 {
        (60_000) / (self.bpm as u32 * STEPS_PER_BEAT)
    }

    /// Total pattern duration in milliseconds
    pub fn total_duration_ms(&self) -> u32 {
        self.step_duration_ms() * self.steps.len() as u32
    }

    /// Render one full loop of this pattern into stereo i16 samples
    pub fn render(&self, engine: &mut SynthEngine) -> Vec<i16> {
        let step_samples = self.step_duration_samples() as usize;
        let total_samples = step_samples * self.steps.len();
        let mut buffer = vec![0i16; total_samples * CHANNELS as usize];

        // Save and set engine state
        let saved_wf = engine.waveform;
        let saved_env = engine.envelope;
        engine.envelope = self.envelope;

        for (i, step) in self.steps.iter().enumerate() {
            if step.is_rest() {
                // Silence — already zeroed
                continue;
            }

            // Set waveform (step override or pattern default)
            let wf = step.waveform.unwrap_or(self.waveform);
            engine.set_waveform(wf);

            // Note on
            engine.note_on(step.note, step.velocity);

            // Render this step's portion of the buffer
            let offset = i * step_samples * CHANNELS as usize;
            let buf_slice = &mut buffer[offset..offset + step_samples * CHANNELS as usize];
            engine.render(buf_slice, step_samples);

            // Note off (let envelope release during next step or trail)
            engine.note_off(step.note);
        }

        // Restore engine state
        engine.set_waveform(saved_wf);
        engine.envelope = saved_env;

        buffer
    }

    /// Display the pattern as a text grid
    pub fn display(&self) -> String {
        let mut s = String::new();
        s.push_str(&format!("Pattern: \"{}\" | {} steps | {} BPM | {} | {}ms/step\n",
            self.name_str(), self.steps.len(), self.bpm,
            self.waveform.name(), self.step_duration_ms()));
        s.push_str(&format!("Total duration: {}ms\n\n", self.total_duration_ms()));

        // Step numbers header
        s.push_str(" Step: ");
        for i in 0..self.steps.len() {
            s.push_str(&format!("{:>3}", i + 1));
        }
        s.push('\n');

        // Notes row
        s.push_str(" Note: ");
        for step in &self.steps {
            s.push_str(&format!("{:>3}", step.display()));
        }
        s.push('\n');

        // Velocity row
        s.push_str("  Vel: ");
        for step in &self.steps {
            if step.is_rest() {
                s.push_str(" --");
            } else {
                s.push_str(&format!("{:>3}", step.velocity));
            }
        }
        s.push('\n');

        // Waveform row
        s.push_str(" Wave: ");
        for step in &self.steps {
            s.push_str(&format!("{:>3}", step.wave_display()));
        }
        s.push('\n');

        s
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Pattern Bank — manages all patterns
// ═══════════════════════════════════════════════════════════════════════════════

/// Storage for all patterns
pub struct PatternBank {
    pub patterns: Vec<Pattern>,
}

impl PatternBank {
    pub fn new() -> Self {
        Self { patterns: Vec::new() }
    }

    /// Add a new pattern, returns its index
    pub fn add(&mut self, pattern: Pattern) -> Result<usize, &'static str> {
        if self.patterns.len() >= MAX_PATTERNS {
            return Err("Maximum patterns reached (16)");
        }
        // Check for duplicate name
        let name = pattern.name_str();
        for p in &self.patterns {
            if p.name_str() == name {
                return Err("Pattern name already exists");
            }
        }
        self.patterns.push(pattern);
        Ok(self.patterns.len() - 1)
    }

    /// Find a pattern by name
    pub fn find(&self, name: &str) -> Option<usize> {
        self.patterns.iter().position(|p| p.name_str() == name)
    }

    /// Get a pattern by index
    pub fn get(&self, idx: usize) -> Option<&Pattern> {
        self.patterns.get(idx)
    }

    /// Get a mutable pattern by index
    pub fn get_mut(&mut self, idx: usize) -> Option<&mut Pattern> {
        self.patterns.get_mut(idx)
    }

    /// Get a pattern by name
    pub fn get_by_name(&self, name: &str) -> Option<&Pattern> {
        self.find(name).and_then(|i| self.get(i))
    }

    /// Get a mutable pattern by name
    pub fn get_by_name_mut(&mut self, name: &str) -> Option<&mut Pattern> {
        let idx = self.find(name)?;
        self.get_mut(idx)
    }

    /// Remove a pattern by name
    pub fn remove(&mut self, name: &str) -> Result<(), &'static str> {
        let idx = self.find(name).ok_or("Pattern not found")?;
        self.patterns.remove(idx);
        Ok(())
    }

    /// List all patterns
    pub fn list(&self) -> String {
        if self.patterns.is_empty() {
            return String::from("No patterns. Use 'synth pattern new <name>' to create one.\n");
        }
        let mut s = String::new();
        s.push_str(&format!("Patterns ({}/{}):\n", self.patterns.len(), MAX_PATTERNS));
        for (i, p) in self.patterns.iter().enumerate() {
            s.push_str(&format!("  [{}] \"{}\" — {} steps, {} BPM, {}\n",
                i, p.name_str(), p.steps.len(), p.bpm, p.waveform.name()));
        }
        s
    }

    /// Load built-in preset patterns
    pub fn load_presets(&mut self) {
        // ── Arpège C mineur ──
        let mut arp = Pattern::new("arp", 16, 140);
        arp.waveform = Waveform::Sine;
        arp.envelope = Envelope::pluck();
        let arp_notes = [60, 63, 67, 72, 67, 63, 60, 63, 67, 72, 67, 63, 60, 63, 67, 72]; // C Eb G C'
        for (i, &n) in arp_notes.iter().enumerate() {
            arp.steps[i] = Step::note_vel(n, 90);
        }
        let _ = self.add(arp);

        // ── Techno kick 4/4 ──
        let mut techno = Pattern::new("techno", 16, 128);
        techno.waveform = Waveform::Sine;
        techno.envelope = Envelope::new(1, 80, 0, 30);
        // Kick on beats 1, 5, 9, 13 (every 4th step) — low C2
        for i in (0..16).step_by(4) {
            techno.steps[i] = Step::note_vel(36, 127); // C2 — deep kick
        }
        let _ = self.add(techno);

        // ── Bass line (saw) ──
        let mut bass = Pattern::new("bass", 16, 120);
        bass.waveform = Waveform::Sawtooth;
        bass.envelope = Envelope::new(5, 100, 60, 50);
        let bass_notes: [u8; 16] = [36, 255, 36, 36, 39, 255, 39, 36, 43, 255, 43, 43, 41, 255, 41, 36];
        for (i, &n) in bass_notes.iter().enumerate() {
            if n != 255 {
                bass.steps[i] = Step::note_vel(n, 100);
            }
        }
        let _ = self.add(bass);

        // ── Chiptune melody ──
        let mut chip = Pattern::new("chiptune", 16, 150);
        chip.waveform = Waveform::Square;
        chip.envelope = Envelope::new(2, 30, 80, 20);
        let chip_notes: [u8; 16] = [72, 74, 76, 72, 79, 255, 79, 255, 76, 74, 72, 74, 76, 72, 71, 255];
        for (i, &n) in chip_notes.iter().enumerate() {
            if n != 255 {
                chip.steps[i] = Step::note_vel(n, 110);
            }
        }
        let _ = self.add(chip);

        // ── Pad chord (slow) ──
        let mut pad = Pattern::new("pad", 8, 80);
        pad.waveform = Waveform::Triangle;
        pad.envelope = Envelope::pad();
        // Long notes, fewer steps
        let pad_notes: [u8; 8] = [60, 255, 64, 255, 67, 255, 72, 255]; // C E G C'
        for (i, &n) in pad_notes.iter().enumerate() {
            if n != 255 {
                pad.steps[i] = Step::note_vel(n, 80);
            }
        }
        let _ = self.add(pad);
    }
}
