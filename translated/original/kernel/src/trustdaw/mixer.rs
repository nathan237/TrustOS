//! Multi-track Mixer for TrustDAW
//!
//! Renders all project tracks into a single stereo audio buffer.
//! Each track has independent volume, pan, mute, and solo controls.
//! Uses the existing SynthEngine for per-track rendering.

use alloc::vec::Vec;
use alloc::vec;

use crate::audio::synth::{SynthEngine, Envelope, SAMPLE_RATE};
use super::track::{Project, Track, Note, ticks_to_samples};

// ═══════════════════════════════════════════════════════════════════════════════
// Mixer Channel — per-track controls
// ═══════════════════════════════════════════════════════════════════════════════

/// A single mixer channel (one per track)
#[derive(Debug, Clone, Copy)]
pub struct MixerChannel {
    /// Volume (0-255)
    pub volume: u8,
    /// Pan: -100 (full left) to +100 (full right), 0 = center
    pub pan: i8,
    /// Muted?
    pub muted: bool,
    /// Soloed?
    pub solo: bool,
}

impl MixerChannel {
    pub fn new() -> Self {
        Self {
            volume: 200,
            pan: 0,
            muted: false,
            solo: false,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Mixer
// ═══════════════════════════════════════════════════════════════════════════════

/// Multi-track mixer
pub struct Mixer {
    /// Per-track channel strips
    pub channels: Vec<MixerChannel>,
    /// Master volume (0-255)
    pub master_volume: u8,
}

impl Mixer {
    /// Create a new mixer with N channels
    pub fn new(num_channels: usize) -> Self {
        Self {
            channels: vec![MixerChannel::new(); num_channels],
            master_volume: 220,
        }
    }

    /// Set volume for a channel
    pub fn set_volume(&mut self, ch: usize, volume: u8) -> Result<(), &'static str> {
        let channel = self.channels.get_mut(ch).ok_or("Invalid channel")?;
        channel.volume = volume;
        Ok(())
    }

    /// Set pan for a channel
    pub fn set_pan(&mut self, ch: usize, pan: i8) -> Result<(), &'static str> {
        let channel = self.channels.get_mut(ch).ok_or("Invalid channel")?;
        channel.pan = pan.clamp(-100, 100);
        Ok(())
    }

    /// Toggle mute for a channel
    pub fn toggle_mute(&mut self, ch: usize) -> Result<bool, &'static str> {
        let channel = self.channels.get_mut(ch).ok_or("Invalid channel")?;
        channel.muted = !channel.muted;
        Ok(channel.muted)
    }

    /// Toggle solo for a channel
    pub fn toggle_solo(&mut self, ch: usize) -> Result<bool, &'static str> {
        let channel = self.channels.get_mut(ch).ok_or("Invalid channel")?;
        channel.solo = !channel.solo;
        Ok(channel.solo)
    }

    /// Check if any channel is soloed
    pub fn has_solo(&self) -> bool {
        self.channels.iter().any(|c| c.solo)
    }

    /// Should a channel be audible? (considering mute/solo logic)
    pub fn is_audible(&self, ch: usize) -> bool {
        if let Some(channel) = self.channels.get(ch) {
            if channel.muted { return false; }
            if self.has_solo() { return channel.solo; }
            true
        } else {
            false
        }
    }

    /// Apply volume and pan to a stereo sample pair
    /// Returns (left, right)
    pub fn apply_channel(&self, ch: usize, left: i32, right: i32) -> (i32, i32) {
        if let Some(channel) = self.channels.get(ch) {
            let vol = channel.volume as i32;
            // Pan law: constant power approximation with integer math
            // pan = -100..+100
            // left_gain  = (100 - pan) / 100 (when pan > 0, reduce left)
            // right_gain = (100 + pan) / 100 (when pan < 0, reduce right)
            let pan = channel.pan as i32;
            let left_gain = (100 - pan).clamp(0, 200);
            let right_gain = (100 + pan).clamp(0, 200);

            let l = left * vol / 255 * left_gain / 100;
            let r = right * vol / 255 * right_gain / 100;
            (l, r)
        } else {
            (left, right)
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Rendering — render a full project to stereo PCM
// ═══════════════════════════════════════════════════════════════════════════════

/// Render a single track to a mono sample buffer
fn render_track(track: &Track, bpm: u32, start_tick: u32, total_samples: usize) -> Vec<i32> {
    let mut buffer = vec![0i32; total_samples];

    if track.notes.is_empty() {
        return buffer;
    }

    // Create a per-track synth engine
    let mut engine = SynthEngine::new();
    engine.set_waveform(track.waveform);
    engine.envelope = track.envelope;

    // Process notes: for each sample position, check which notes should be active
    // Convert all note events to sample-position events
    struct NoteEvent {
        sample_pos: usize,
        pitch: u8,
        velocity: u8,
        is_on: bool,
    }

    let mut events: Vec<NoteEvent> = Vec::new();

    for note in &track.notes {
        if note.end_tick() <= start_tick {
            continue; // Note is before our render window
        }

        let note_start_sample = if note.start_tick >= start_tick {
            ticks_to_samples(note.start_tick - start_tick, bpm) as usize
        } else {
            0 // Note started before our window, trigger immediately
        };

        let note_end_sample = ticks_to_samples(
            note.end_tick().saturating_sub(start_tick), bpm
        ) as usize;

        if note_start_sample < total_samples {
            events.push(NoteEvent {
                sample_pos: note_start_sample,
                pitch: note.pitch,
                velocity: note.velocity,
                is_on: true,
            });
        }

        if note_end_sample < total_samples {
            events.push(NoteEvent {
                sample_pos: note_end_sample,
                pitch: note.pitch,
                velocity: note.velocity,
                is_on: false,
            });
        }
    }

    // Sort events by sample position
    events.sort_by_key(|e| e.sample_pos);

    // Render sample by sample, processing events as we go
    let mut event_idx = 0;
    let mut temp_stereo = vec![0i16; 2]; // Single sample stereo buffer

    for sample in 0..total_samples {
        // Process all events at this sample position
        while event_idx < events.len() && events[event_idx].sample_pos <= sample {
            let ev = &events[event_idx];
            if ev.is_on {
                engine.note_on(ev.pitch, ev.velocity);
            } else {
                engine.note_off(ev.pitch);
            }
            event_idx += 1;
        }

        // Render one sample from the engine
        engine.render(&mut temp_stereo, 1);
        // Take the mono mix (average of L+R)
        buffer[sample] = (temp_stereo[0] as i32 + temp_stereo[1] as i32) / 2;
    }

    buffer
}

/// Render an entire project, mixing all tracks together
/// Returns stereo interleaved i16 buffer ready for HDA playback
pub fn render_project(project: &Project, mixer: &Mixer, bpm: u32, start_tick: u32) -> Vec<i16> {
    if project.tracks.is_empty() {
        return Vec::new();
    }

    // Calculate total length
    let total_ticks = project.length_ticks();
    if total_ticks <= start_tick {
        return Vec::new();
    }

    let render_ticks = total_ticks - start_tick;
    let total_samples = ticks_to_samples(render_ticks, bpm) as usize;

    if total_samples == 0 {
        return Vec::new();
    }

    // Render each track to its own mono buffer
    let track_buffers: Vec<Vec<i32>> = project.tracks.iter()
        .map(|track| render_track(track, bpm, start_tick, total_samples))
        .collect();

    // Mix all tracks into stereo output
    let mut output = vec![0i16; total_samples * 2];

    for sample in 0..total_samples {
        let mut left_mix: i32 = 0;
        let mut right_mix: i32 = 0;

        for (ch_idx, track_buf) in track_buffers.iter().enumerate() {
            if !mixer.is_audible(ch_idx) {
                continue;
            }

            let mono_sample = track_buf[sample];
            let (l, r) = mixer.apply_channel(ch_idx, mono_sample, mono_sample);
            left_mix += l;
            right_mix += r;
        }

        // Apply master volume
        left_mix = left_mix * mixer.master_volume as i32 / 255;
        right_mix = right_mix * mixer.master_volume as i32 / 255;

        // Soft clipping to prevent harsh distortion
        left_mix = soft_clip(left_mix);
        right_mix = soft_clip(right_mix);

        output[sample * 2] = left_mix.clamp(-32767, 32767) as i16;
        output[sample * 2 + 1] = right_mix.clamp(-32767, 32767) as i16;
    }

    output
}

/// Soft clipping function — smooth saturation instead of hard clamp
fn soft_clip(sample: i32) -> i32 {
    const THRESHOLD: i32 = 24000;
    if sample > THRESHOLD {
        let excess = sample - THRESHOLD;
        let compressed = excess * 8000 / (excess + 8000); // Asymptotic curve
        THRESHOLD + compressed
    } else if sample < -THRESHOLD {
        let excess = -sample - THRESHOLD;
        let compressed = excess * 8000 / (excess + 8000);
        -(THRESHOLD + compressed)
    } else {
        sample
    }
}
