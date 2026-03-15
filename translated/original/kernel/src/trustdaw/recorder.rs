//! Real-time Note Recorder for TrustDAW
//!
//! Records keyboard input as MIDI notes into a track.
//! Uses system timer ticks for timing, converts to MIDI ticks on completion.

use alloc::vec::Vec;
use alloc::format;
use alloc::string::String;
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use super::track::{Note, ms_to_ticks};
use super::keyboard_midi;
use super::{RECORDING, PLAYBACK_POS, BPM, TICKS_PER_QUARTER};

// ═══════════════════════════════════════════════════════════════════════════════
// Recording State
// ═══════════════════════════════════════════════════════════════════════════════

/// A note being held (not yet released)
#[derive(Debug, Clone, Copy)]
struct ActiveNote {
    /// MIDI pitch
    pitch: u8,
    /// Velocity
    velocity: u8,
    /// Start time in ms (from recording start)
    start_ms: u32,
}

/// Recording session
pub struct RecordSession {
    /// Recorded notes (completed)
    pub notes: Vec<Note>,
    /// Currently held notes (waiting for release)
    active_notes: Vec<ActiveNote>,
    /// Recording start timestamp (ms from system uptime)
    start_time_ms: u32,
    /// BPM at recording time (for tick conversion)
    bpm: u32,
    /// Quantize grid (0 = no quantize)
    pub quantize_ticks: u32,
    /// Start tick offset (where in the timeline we started recording)
    pub start_tick_offset: u32,
}

impl RecordSession {
    /// Create a new recording session
    pub fn new(bpm: u32, start_tick: u32) -> Self {
        Self {
            notes: Vec::new(),
            active_notes: Vec::new(),
            start_time_ms: crate::time::uptime_ms() as u32,
            bpm,
            quantize_ticks: TICKS_PER_QUARTER / 4, // Default: sixteenth note
            start_tick_offset: start_tick,
        }
    }

    /// Handle a key press (note on)
    pub fn note_on(&mut self, scancode: u8) {
        if let Some(pitch) = keyboard_midi::scancode_to_midi(scancode) {
            // Check if this note is already active (don't duplicate)
            if self.active_notes.iter().any(|n| n.pitch == pitch) {
                return;
            }

            let elapsed_ms = self.elapsed_ms();
            let velocity = keyboard_midi::get_velocity();

            self.active_notes.push(ActiveNote {
                pitch,
                velocity,
                start_ms: elapsed_ms,
            });

            // Also trigger sound for real-time monitoring
            let _ = crate::audio::play_midi_note(pitch, velocity, 100);
        }
    }

    /// Handle a key release (note off)
    pub fn note_off(&mut self, scancode: u8) {
        if let Some(pitch) = keyboard_midi::scancode_to_midi(scancode) {
            let elapsed_ms = self.elapsed_ms();

            // Find and remove the active note
            if let Some(pos) = self.active_notes.iter().position(|n| n.pitch == pitch) {
                let active = self.active_notes.remove(pos);
                let duration_ms = elapsed_ms.saturating_sub(active.start_ms).max(10);

                // Convert to ticks
                let start_tick = self.start_tick_offset + ms_to_ticks(active.start_ms, self.bpm);
                let duration_ticks = ms_to_ticks(duration_ms, self.bpm).max(1);

                // Apply quantization
                let (start_tick, duration_ticks) = if self.quantize_ticks > 0 {
                    let q = self.quantize_ticks;
                    let snapped_start = ((start_tick + q / 2) / q) * q;
                    let snapped_dur = ((duration_ticks + q / 2) / q) * q;
                    (snapped_start, snapped_dur.max(q))
                } else {
                    (start_tick, duration_ticks)
                };

                self.notes.push(Note::new(pitch, active.velocity, start_tick, duration_ticks));
            }
        }
    }

    /// Get elapsed time since recording started
    fn elapsed_ms(&self) -> u32 {
        let now = crate::time::uptime_ms() as u32;
        now.saturating_sub(self.start_time_ms)
    }

    /// Finalize recording — release all held notes and return the recorded notes
    pub fn finalize(&mut self) -> Vec<Note> {
        let elapsed_ms = self.elapsed_ms();

        // Release all still-active notes
        for active in self.active_notes.drain(..) {
            let duration_ms = elapsed_ms.saturating_sub(active.start_ms).max(10);
            let start_tick = self.start_tick_offset + ms_to_ticks(active.start_ms, self.bpm);
            let duration_ticks = ms_to_ticks(duration_ms, self.bpm).max(1);

            self.notes.push(Note::new(active.pitch, active.velocity, start_tick, duration_ticks));
        }

        // Sort by start tick
        self.notes.sort_by_key(|n| n.start_tick);

        core::mem::take(&mut self.notes)
    }

    /// Get the number of completed notes
    pub fn note_count(&self) -> usize {
        self.notes.len()
    }

    /// Get the number of currently held notes
    pub fn active_count(&self) -> usize {
        self.active_notes.len()
    }

    /// Get recording duration in ms
    pub fn duration_ms(&self) -> u32 {
        self.elapsed_ms()
    }

    /// Get recording status string
    pub fn status(&self) -> String {
        let elapsed = self.elapsed_ms();
        let secs = elapsed / 1000;
        let ms = elapsed % 1000;
        format!("REC {:02}:{:02}.{:03} | Notes: {} | Active: {} | Quantize: {}",
            secs / 60, secs % 60, ms,
            self.notes.len(), self.active_notes.len(),
            if self.quantize_ticks > 0 {
                format!("1/{}", TICKS_PER_QUARTER * 4 / self.quantize_ticks)
            } else {
                String::from("off")
            }
        )
    }
}

/// Run an interactive recording session on the current armed track
/// This is a blocking function that reads keyboard input until Escape is pressed
pub fn record_interactive(track_idx: usize) -> Result<usize, &'static str> {
    super::ensure_init()?;

    let bpm = BPM.load(Ordering::Relaxed);
    let start_tick = PLAYBACK_POS.load(Ordering::Relaxed);

    RECORDING.store(true, Ordering::Relaxed);

    crate::println!("Recording on track {}...", track_idx);
    crate::println!("Play notes on keyboard. Press [Esc] to stop recording.\n");
    crate::println!("{}", keyboard_midi::display_layout());

    let mut session = RecordSession::new(bpm, start_tick);

    // Recording loop — read keyboard input
    loop {
        if !RECORDING.load(Ordering::Relaxed) {
            break; // External stop command
        }

        // Non-blocking key read
        if let Some(scancode) = crate::keyboard::try_read_key() {
            // Escape key (scancode 0x01) → stop recording
            if scancode == 0x01 {
                break;
            }

            // Check for octave/velocity controls
            match scancode {
                0x3B => { // F1 → octave down
                    let oct = keyboard_midi::octave_down();
                    crate::println!("Octave: {:+}", oct);
                    continue;
                }
                0x3C => { // F2 → octave up
                    let oct = keyboard_midi::octave_up();
                    crate::println!("Octave: {:+}", oct);
                    continue;
                }
                0x3D => { // F3 → velocity down
                    let v = keyboard_midi::get_velocity();
                    keyboard_midi::set_velocity(v.saturating_sub(10));
                    crate::println!("Velocity: {}", keyboard_midi::get_velocity());
                    continue;
                }
                0x3E => { // F4 → velocity up
                    let v = keyboard_midi::get_velocity();
                    keyboard_midi::set_velocity((v + 10).min(127));
                    crate::println!("Velocity: {}", keyboard_midi::get_velocity());
                    continue;
                }
                _ => {}
            }

            let is_release = scancode & 0x80 != 0;
            let key = scancode & 0x7F;

            if is_release {
                session.note_off(key);
            } else {
                session.note_on(key);
            }
        }

        // Brief yield to prevent busy-waiting
        // In a real preemptive OS we'd yield the CPU here
        for _ in 0..1000 {
            core::hint::spin_loop();
        }
    }

    RECORDING.store(false, Ordering::Relaxed);

    // Finalize and add notes to the track
    let notes = session.finalize();
    let count = notes.len();

    if count > 0 {
        let mut project = super::PROJECT.lock();
        let project = project.as_mut().ok_or("No project")?;
        let track = project.tracks.get_mut(track_idx).ok_or("Invalid track index")?;

        for note in notes {
            track.add_note(note);
        }

        crate::println!("\nRecording complete: {} notes added to track {}", count, track_idx);
    } else {
        crate::println!("\nRecording complete: no notes recorded");
    }

    Ok(count)
}
