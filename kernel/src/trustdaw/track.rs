//! Track and Note data models for TrustDAW
//!
//! A Project contains multiple Tracks, each with a list of Notes.
//! Notes are MIDI-style events with pitch, velocity, start time (ticks), and duration.

use alloc::vec::Vec;
use alloc::string::String;
use alloc::format;
use crate::audio::synth::{Waveform, Envelope};

use super::MAX_TRACKS;

// ═══════════════════════════════════════════════════════════════════════════════
// Note — a single MIDI-style note event
// ═══════════════════════════════════════════════════════════════════════════════

/// A note event in a track
#[derive(Debug, Clone, Copy)]
pub struct Note {
    /// MIDI pitch (0-127)
    pub pitch: u8,
    /// Velocity (0-127)
    pub velocity: u8,
    /// Start time in ticks (from beginning of track)
    pub start_tick: u32,
    /// Duration in ticks
    pub duration_ticks: u32,
}

impl Note {
    /// Create a new note
    pub fn new(pitch: u8, velocity: u8, start_tick: u32, duration_ticks: u32) -> Self {
        Self {
            pitch: pitch.min(127),
            velocity: velocity.min(127),
            start_tick,
            duration_ticks: duration_ticks.max(1),
        }
    }

    /// End tick (exclusive)
    pub fn end_tick(&self) -> u32 {
        self.start_tick + self.duration_ticks
    }

    /// Get the note name (e.g., "C4", "A#3")
    pub fn name(&self) -> String {
        let note_name = crate::audio::tables::midi_to_note_name(self.pitch);
        let octave = crate::audio::tables::midi_octave(self.pitch);
        format!("{}{}", note_name, octave)
    }

    /// Duration in milliseconds at a given BPM
    pub fn duration_ms(&self, bpm: u32) -> u32 {
        ticks_to_ms(self.duration_ticks, bpm)
    }

    /// Start time in milliseconds at a given BPM
    pub fn start_ms(&self, bpm: u32) -> u32 {
        ticks_to_ms(self.start_tick, bpm)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Track — a sequence of notes with instrument settings
// ═══════════════════════════════════════════════════════════════════════════════

/// A single track in the project
pub struct Track {
    /// Track name (fixed-size buffer for no_std)
    name: [u8; 32],
    /// Name length
    name_len: usize,
    /// Notes in this track (sorted by start_tick)
    pub notes: Vec<Note>,
    /// Waveform for this track
    pub waveform: Waveform,
    /// Envelope preset for this track
    pub envelope: Envelope,
    /// Track color (RGB for UI display)
    pub color: u32,
    /// Is this track armed for recording?
    pub armed: bool,
}

impl Track {
    /// Create a new empty track
    pub fn new(name: &str) -> Self {
        let mut name_buf = [0u8; 32];
        let bytes = name.as_bytes();
        let len = bytes.len().min(32);
        name_buf[..len].copy_from_slice(&bytes[..len]);

        Self {
            name: name_buf,
            name_len: len,
            notes: Vec::new(),
            waveform: Waveform::Sine,
            envelope: Envelope::default_env(),
            color: 0x4488FF, // Default blue
            armed: false,
        }
    }

    /// Get track name as &str
    pub fn name_str(&self) -> &str {
        core::str::from_utf8(&self.name[..self.name_len]).unwrap_or("???")
    }

    /// Add a note, keeping the list sorted by start_tick
    pub fn add_note(&mut self, note: Note) {
        // Binary search for insertion point
        let pos = self.notes.partition_point(|n| n.start_tick < note.start_tick);
        self.notes.insert(pos, note);
    }

    /// Remove a note by index
    pub fn remove_note(&mut self, index: usize) -> Option<Note> {
        if index < self.notes.len() {
            Some(self.notes.remove(index))
        } else {
            None
        }
    }

    /// Remove all notes in a tick range
    pub fn remove_notes_in_range(&mut self, start: u32, end: u32) {
        self.notes.retain(|n| n.start_tick < start || n.start_tick >= end);
    }

    /// Find notes that are active at a given tick
    pub fn notes_at_tick(&self, tick: u32) -> Vec<&Note> {
        self.notes.iter()
            .filter(|n| n.start_tick <= tick && tick < n.end_tick())
            .collect()
    }

    /// Find notes in a tick range (start inclusive, end exclusive)
    pub fn notes_in_range(&self, start: u32, end: u32) -> Vec<&Note> {
        self.notes.iter()
            .filter(|n| n.start_tick < end && n.end_tick() > start)
            .collect()
    }

    /// Get the last tick (end of last note)
    pub fn end_tick(&self) -> u32 {
        self.notes.iter().map(|n| n.end_tick()).max().unwrap_or(0)
    }

    /// Total number of notes
    pub fn note_count(&self) -> usize {
        self.notes.len()
    }

    /// Clear all notes
    pub fn clear(&mut self) {
        self.notes.clear();
    }

    /// Quantize all notes to the nearest grid size (in ticks)
    pub fn quantize(&mut self, grid_ticks: u32) {
        if grid_ticks == 0 { return; }
        for note in &mut self.notes {
            let remainder = note.start_tick % grid_ticks;
            if remainder > grid_ticks / 2 {
                note.start_tick += grid_ticks - remainder;
            } else {
                note.start_tick -= remainder;
            }
            // Also quantize duration
            let dur_remainder = note.duration_ticks % grid_ticks;
            if dur_remainder > grid_ticks / 2 {
                note.duration_ticks += grid_ticks - dur_remainder;
            } else if note.duration_ticks > dur_remainder {
                note.duration_ticks -= dur_remainder;
            }
            if note.duration_ticks == 0 {
                note.duration_ticks = grid_ticks;
            }
        }
    }

    /// Transpose all notes by semitones
    pub fn transpose(&mut self, semitones: i8) {
        for note in &mut self.notes {
            let new_pitch = note.pitch as i16 + semitones as i16;
            note.pitch = new_pitch.clamp(0, 127) as u8;
        }
    }

    /// Shift all notes by ticks
    pub fn shift(&mut self, ticks: i32) {
        for note in &mut self.notes {
            let new_start = note.start_tick as i64 + ticks as i64;
            note.start_tick = new_start.max(0) as u32;
        }
        // Re-sort after shifting
        self.notes.sort_by_key(|n| n.start_tick);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Project — the top-level container
// ═══════════════════════════════════════════════════════════════════════════════

/// A TrustDAW project containing multiple tracks
pub struct Project {
    /// Project name
    name: [u8; 64],
    /// Name length
    name_len: usize,
    /// Tracks
    pub tracks: Vec<Track>,
    /// Global BPM
    pub bpm: u16,
    /// Time signature numerator (e.g., 4 for 4/4)
    pub time_sig_num: u8,
    /// Time signature denominator (e.g., 4 for 4/4)
    pub time_sig_den: u8,
}

/// Track colors for UI
static TRACK_COLORS: [u32; 16] = [
    0x4488FF, // Blue
    0xFF4444, // Red 
    0x44FF44, // Green
    0xFFAA00, // Orange
    0xAA44FF, // Purple
    0x44FFFF, // Cyan
    0xFF44AA, // Pink
    0xFFFF44, // Yellow
    0x88FF88, // Light green
    0xFF8844, // Dark orange
    0x8888FF, // Light blue
    0xFF88FF, // Light pink
    0x44FFAA, // Teal
    0xAAAAFF, // Lavender
    0xFFAA88, // Peach
    0x88FFFF, // Light cyan
];

impl Project {
    /// Create a new project
    pub fn new(name: &str, bpm: u16) -> Self {
        let mut name_buf = [0u8; 64];
        let bytes = name.as_bytes();
        let len = bytes.len().min(64);
        name_buf[..len].copy_from_slice(&bytes[..len]);

        Self {
            name: name_buf,
            name_len: len,
            tracks: Vec::new(),
            bpm,
            time_sig_num: 4,
            time_sig_den: 4,
        }
    }

    /// Get project name as &str
    pub fn name_str(&self) -> &str {
        core::str::from_utf8(&self.name[..self.name_len]).unwrap_or("???")
    }

    /// Add a track, returns its index
    pub fn add_track(&mut self, name: &str) -> Result<usize, &'static str> {
        if self.tracks.len() >= MAX_TRACKS {
            return Err("Maximum tracks reached");
        }
        let idx = self.tracks.len();
        let mut track = Track::new(name);
        track.color = TRACK_COLORS[idx % TRACK_COLORS.len()];
        self.tracks.push(track);
        Ok(idx)
    }

    /// Remove a track by index
    pub fn remove_track(&mut self, index: usize) -> Result<(), &'static str> {
        if index >= self.tracks.len() {
            return Err("Invalid track index");
        }
        self.tracks.remove(index);
        Ok(())
    }

    /// Get the project length in ticks (end of the last note)
    pub fn length_ticks(&self) -> u32 {
        self.tracks.iter().map(|t| t.end_tick()).max().unwrap_or(0)
    }

    /// Get the project length in bars (rounded up)
    pub fn length_bars(&self) -> u32 {
        let ticks = self.length_ticks();
        let ticks_per_bar = super::TICKS_PER_QUARTER * self.time_sig_num as u32;
        (ticks + ticks_per_bar - 1) / ticks_per_bar
    }

    /// Get total track count
    pub fn track_count(&self) -> usize {
        self.tracks.len()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Utility functions
// ═══════════════════════════════════════════════════════════════════════════════

/// Convert ticks to milliseconds at a given BPM
pub fn ticks_to_ms(ticks: u32, bpm: u32) -> u32 {
    if bpm == 0 { return 0; }
    // ms = ticks * 60000 / (BPM * TICKS_PER_QUARTER)
    (ticks as u64 * 60_000 / (bpm as u64 * super::TICKS_PER_QUARTER as u64)) as u32
}

/// Convert milliseconds to ticks at a given BPM
pub fn ms_to_ticks(ms: u32, bpm: u32) -> u32 {
    if ms == 0 { return 0; }
    // ticks = ms * BPM * TICKS_PER_QUARTER / 60000
    (ms as u64 * bpm as u64 * super::TICKS_PER_QUARTER as u64 / 60_000) as u32
}

/// Convert ticks to samples at a given BPM
pub fn ticks_to_samples(ticks: u32, bpm: u32) -> u32 {
    if bpm == 0 { return 0; }
    // samples = ticks * SAMPLE_RATE * 60 / (BPM * TICKS_PER_QUARTER)
    (ticks as u64 * super::SAMPLE_RATE as u64 * 60 / (bpm as u64 * super::TICKS_PER_QUARTER as u64)) as u32
}

/// Convert samples to ticks at a given BPM
pub fn samples_to_ticks(samples: u32, bpm: u32) -> u32 {
    if samples == 0 { return 0; }
    (samples as u64 * bpm as u64 * super::TICKS_PER_QUARTER as u64 / (super::SAMPLE_RATE as u64 * 60)) as u32
}
