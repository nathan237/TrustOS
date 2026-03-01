//! TrustDAW Beat Studio — YouTube Showcase Mode
//!
//! A visually impressive beat-making workstation designed for live demos.
//! Layout inspired by Adobe Audition / FL Studio:
//!
//! ╔═══════════════════════════════════════════════════════════════════════╗
//! ║  ♫ TrustDAW Beat Studio     ■ ►    ● REC    BPM: 128   1:02:120   ║
//! ╠════════╦════════════════════════════════════════════╦══════════════════╣
//! ║ TRACKS ║  S T E P   S E Q U E N C E R             ║     SCOPE       ║
//! ║ ────── ║  1  2  3  4 │ 5  6  7  8 │...            ║    ∿∿∿∿∿∿      ║
//! ║▌Kick   ║ [█][·][·][·]│[█][·][·][·]│               ║                 ║
//! ║▌Snare  ║ [·][·][·][·]│[█][·][·][·]│               ║   SPECTRUM      ║
//! ║▌HiHat  ║ [█][·][█][·]│[█][·][█][·]│               ║   █████████     ║
//! ║▌Bass   ║ [█][·][·][█]│[·][·][█][·]│               ║                 ║
//! ╠════════╬════════════════════════════╦═══════════════╬══════════════════╣
//! ║ MIXER  ║   VIRTUAL KEYBOARD        ║    INFO       ║                 ║
//! ║ faders ║   [piano keys visual]     ║  Oct/Vel/etc  ║                 ║
//! ╠════════╩════════════════════════════╩═══════════════╩══════════════════╣
//! ║ [Space] Play  [Enter] Toggle  [Tab] Track  [Z-M / Q-P] Piano keys   ║
//! ╚═══════════════════════════════════════════════════════════════════════╝

use alloc::vec::Vec;
use alloc::vec;
use alloc::format;
use alloc::string::String;
use core::sync::atomic::Ordering;

use crate::audio::synth::{Waveform, Envelope, SynthEngine, SAMPLE_RATE, CHANNELS};

// ═══════════════════════════════════════════════════════════════════════════════
// Color Palette — Dark professional theme
// ═══════════════════════════════════════════════════════════════════════════════

mod colors {
    // Background layers
    pub const BG_DARK: u32        = 0x0A0A14;
    pub const BG_MAIN: u32        = 0x0F0F1E;
    pub const BG_PANEL: u32       = 0x141428;
    pub const BG_HEADER: u32      = 0x1A1A30;
    pub const BG_ELEVATED: u32    = 0x1E1E38;

    // Borders & Lines
    pub const BORDER: u32         = 0x2A2A4A;
    pub const BORDER_BRIGHT: u32  = 0x4A4A6A;
    pub const DIVIDER: u32        = 0x222240;

    // Text
    pub const TEXT_BRIGHT: u32    = 0xEEEEFF;
    pub const TEXT_PRIMARY: u32   = 0xCCCCDD;
    pub const TEXT_SECONDARY: u32 = 0x8888AA;
    pub const TEXT_DIM: u32       = 0x555577;
    pub const TEXT_ACCENT: u32    = 0x66BBFF;

    // Transport
    pub const PLAY_GREEN: u32     = 0x44DD66;
    pub const STOP_GRAY: u32      = 0x666688;
    pub const REC_RED: u32        = 0xFF3344;
    pub const TRANSPORT_BG: u32   = 0x121228;

    // Step Sequencer
    pub const STEP_OFF: u32       = 0x1A1A30;
    pub const STEP_ON: u32        = 0xFF6622;
    pub const STEP_ON_ALT: u32    = 0xFF8844;
    pub const STEP_CURSOR: u32    = 0x44FF88;
    pub const STEP_PLAYHEAD: u32  = 0x66FF66;
    pub const STEP_BORDER: u32    = 0x333355;
    pub const STEP_BEAT_DIV: u32  = 0x444466;

    // Mixer
    pub const METER_GREEN: u32    = 0x44CC44;
    pub const METER_YELLOW: u32   = 0xCCCC44;
    pub const METER_RED: u32      = 0xCC4444;
    pub const METER_BG: u32       = 0x0D0D1A;
    pub const FADER_KNOB: u32     = 0xBBBBCC;
    pub const MUTE_ORANGE: u32    = 0xFF8800;
    pub const SOLO_YELLOW: u32    = 0xFFDD00;

    // Keyboard
    pub const KEY_WHITE: u32      = 0xDDDDEE;
    pub const KEY_BLACK: u32      = 0x222233;
    pub const KEY_PRESSED: u32    = 0xFF6622;
    pub const KEY_LABEL: u32      = 0x444455;

    // Scope & Spectrum
    pub const SCOPE_LINE: u32     = 0x44DDFF;
    pub const SCOPE_BG: u32       = 0x0A0A18;
    pub const SPECTRUM_1: u32     = 0x22CC66;
    pub const SPECTRUM_2: u32     = 0x66DD44;
    pub const SPECTRUM_3: u32     = 0xCCCC22;
    pub const SPECTRUM_4: u32     = 0xDD6622;
    pub const SPECTRUM_5: u32     = 0xDD2222;

    // Track colors (one per track row)
    pub const TRACK_COLORS: [u32; 8] = [
        0xFF4444, // Kick — red
        0xFFAA22, // Snare — orange
        0xFFDD44, // HiHat — yellow
        0x44DD66, // Bass — green
        0x44AAFF, // Lead — blue
        0x8844FF, // Pad — purple
        0xFF44AA, // FX — pink
        0x44DDDD, // Perc — cyan
    ];
}

// ═══════════════════════════════════════════════════════════════════════════════
// Step Sequencer Data Model
// ═══════════════════════════════════════════════════════════════════════════════

/// Maximum tracks in beat studio
const MAX_BEAT_TRACKS: usize = 8;
/// Maximum steps per track
const MAX_STEPS: usize = 32;
/// Default number of steps (16 = one bar of 16th notes)
const DEFAULT_STEPS: usize = 16;

/// One step in the beat grid
#[derive(Clone, Copy)]
pub struct BeatStep {
    /// Step is active (will sound)
    pub active: bool,
    /// Velocity (1-127, 0 = use track default)
    pub velocity: u8,
    /// Note offset from track base note (for melodic tracks)
    pub note_offset: i8,
}

impl BeatStep {
    pub fn off() -> Self {
        Self { active: false, velocity: 100, note_offset: 0 }
    }

    pub fn on(velocity: u8) -> Self {
        Self { active: true, velocity, note_offset: 0 }
    }

    pub fn on_note(velocity: u8, offset: i8) -> Self {
        Self { active: true, velocity, note_offset: offset }
    }
}

/// One track in the beat studio (drum or melodic)
pub struct BeatTrack {
    /// Track name
    pub name: [u8; 16],
    pub name_len: usize,
    /// Step data
    pub steps: [BeatStep; MAX_STEPS],
    /// Number of active steps (16 or 32)
    pub num_steps: usize,
    /// Base MIDI note (e.g., Kick=36, Snare=38)
    pub base_note: u8,
    /// Waveform
    pub waveform: Waveform,
    /// Envelope
    pub envelope: Envelope,
    /// Volume (0-255)
    pub volume: u8,
    /// Pan (-100..+100)
    pub pan: i8,
    /// Muted
    pub muted: bool,
    /// Solo
    pub solo: bool,
    /// Track color
    pub color: u32,
    /// Is this a drum track (single note per step vs melodic)
    pub is_drum: bool,
}

impl BeatTrack {
    pub fn new(name: &str, base_note: u8, waveform: Waveform, color: u32, is_drum: bool) -> Self {
        let mut name_buf = [0u8; 16];
        let bytes = name.as_bytes();
        let len = bytes.len().min(16);
        name_buf[..len].copy_from_slice(&bytes[..len]);

        Self {
            name: name_buf,
            name_len: len,
            steps: [BeatStep::off(); MAX_STEPS],
            num_steps: DEFAULT_STEPS,
            base_note,
            waveform,
            envelope: Envelope::pluck(),
            volume: 200,
            pan: 0,
            muted: false,
            solo: false,
            color,
            is_drum,
        }
    }

    pub fn name_str(&self) -> &str {
        core::str::from_utf8(&self.name[..self.name_len]).unwrap_or("???")
    }

    /// Toggle a step on/off
    pub fn toggle_step(&mut self, step: usize) {
        if step < self.num_steps {
            self.steps[step].active = !self.steps[step].active;
        }
    }

    /// Get the MIDI note for a given step
    pub fn note_at(&self, step: usize) -> u8 {
        if step < self.num_steps && self.steps[step].active {
            let base = self.base_note as i16;
            let offset = self.steps[step].note_offset as i16;
            (base + offset).clamp(0, 127) as u8
        } else {
            0
        }
    }

    /// Count active steps
    pub fn active_count(&self) -> usize {
        self.steps[..self.num_steps].iter().filter(|s| s.active).count()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Beat Studio — Main State
// ═══════════════════════════════════════════════════════════════════════════════

/// The Beat Studio workspace
pub struct BeatStudio {
    /// 8 tracks (drums + melodic)
    pub tracks: [BeatTrack; MAX_BEAT_TRACKS],
    /// Tempo
    pub bpm: u16,
    /// Swing amount (0-100, 50 = straight, 66 = triplet swing)
    pub swing: u8,
    /// Loop length in bars
    pub loop_bars: u8,

    // Transport state
    pub playing: bool,
    pub recording: bool,
    pub current_step: usize,

    // Cursor
    pub cursor_track: usize,
    pub cursor_step: usize,

    // Visualization
    pub scope_buffer: [i16; 256],
    pub scope_pos: usize,
    pub spectrum: [u8; 16],

    // Keyboard state (which keys are visually "pressed")
    pub keys_pressed: [bool; 128],
    pub octave: i8,
    pub velocity: u8,

    // Layout cache
    fb_w: u32,
    fb_h: u32,
}

impl BeatStudio {
    /// Create a new Beat Studio with default demo beat
    pub fn new() -> Self {
        let fb_w = crate::framebuffer::FB_WIDTH.load(Ordering::Relaxed) as u32;
        let fb_h = crate::framebuffer::FB_HEIGHT.load(Ordering::Relaxed) as u32;

        let tracks = [
            BeatTrack::new("Kick",  36, Waveform::Sine,     colors::TRACK_COLORS[0], true),
            BeatTrack::new("Snare", 38, Waveform::Noise,    colors::TRACK_COLORS[1], true),
            BeatTrack::new("HiHat", 42, Waveform::Noise,    colors::TRACK_COLORS[2], true),
            BeatTrack::new("Bass",  36, Waveform::Square,   colors::TRACK_COLORS[3], false),
            BeatTrack::new("Lead",  60, Waveform::Sawtooth, colors::TRACK_COLORS[4], false),
            BeatTrack::new("Pad",   60, Waveform::Triangle, colors::TRACK_COLORS[5], false),
            BeatTrack::new("FX",    48, Waveform::Sawtooth, colors::TRACK_COLORS[6], false),
            BeatTrack::new("Perc",  56, Waveform::Noise,    colors::TRACK_COLORS[7], true),
        ];

        let mut studio = Self {
            tracks,
            bpm: 128,
            swing: 50,
            loop_bars: 1,
            playing: false,
            recording: false,
            current_step: 0,
            cursor_track: 0,
            cursor_step: 0,
            scope_buffer: [0i16; 256],
            scope_pos: 0,
            spectrum: [0u8; 16],
            keys_pressed: [false; 128],
            octave: 0,
            velocity: 100,
            fb_w,
            fb_h,
        };

        // Pre-load a demo beat pattern
        studio.load_demo_beat();

        // Set appropriate envelopes per track
        studio.tracks[0].envelope = Envelope::new(2, 80, 0, 50);    // Kick: punchy
        studio.tracks[1].envelope = Envelope::new(1, 40, 0, 30);    // Snare: snappy
        studio.tracks[2].envelope = Envelope::new(1, 20, 0, 15);    // HiHat: very short
        studio.tracks[3].envelope = Envelope::pluck();                // Bass: pluck
        studio.tracks[4].envelope = Envelope::new(5, 100, 70, 80);  // Lead: medium
        studio.tracks[5].envelope = Envelope::pad();                  // Pad: slow
        studio.tracks[6].envelope = Envelope::new(1, 200, 0, 100);  // FX: decay
        studio.tracks[7].envelope = Envelope::new(1, 30, 0, 20);    // Perc: short

        studio
    }

    /// Load a classic demo beat pattern
    fn load_demo_beat(&mut self) {
        // Kick: four-on-the-floor
        for i in [0, 4, 8, 12] {
            self.tracks[0].steps[i] = BeatStep::on(127);
        }

        // Snare: beats 2 and 4
        for i in [4, 12] {
            self.tracks[1].steps[i] = BeatStep::on(110);
        }

        // HiHat: eighth notes
        for i in [0, 2, 4, 6, 8, 10, 12, 14] {
            self.tracks[2].steps[i] = BeatStep::on(80);
        }
        // Open hihat on off-beats (lighter)
        for i in [1, 3, 5, 7, 9, 11, 13, 15] {
            self.tracks[2].steps[i] = BeatStep::on(40);
        }

        // Bass: syncopated pattern
        self.tracks[3].steps[0] = BeatStep::on_note(120, 0);   // C
        self.tracks[3].steps[3] = BeatStep::on_note(100, 0);   // C
        self.tracks[3].steps[6] = BeatStep::on_note(110, 3);   // Eb
        self.tracks[3].steps[10] = BeatStep::on_note(100, 5);  // F
        self.tracks[3].steps[13] = BeatStep::on_note(90, 3);   // Eb

        // Lead: simple melody
        self.tracks[4].steps[0] = BeatStep::on_note(100, 0);   // C4
        self.tracks[4].steps[2] = BeatStep::on_note(90, 3);    // Eb4
        self.tracks[4].steps[4] = BeatStep::on_note(100, 7);   // G4
        self.tracks[4].steps[8] = BeatStep::on_note(110, 5);   // F4
        self.tracks[4].steps[11] = BeatStep::on_note(90, 3);   // Eb4

        // Pad: whole notes (only step 0)
        self.tracks[5].steps[0] = BeatStep::on_note(70, 0);    // C

        // FX: sparse hits
        self.tracks[6].steps[6] = BeatStep::on(60);
        self.tracks[6].steps[14] = BeatStep::on(50);

        // Perc: off-beat pattern
        self.tracks[7].steps[1] = BeatStep::on(80);
        self.tracks[7].steps[5] = BeatStep::on(90);
        self.tracks[7].steps[9] = BeatStep::on(80);
        self.tracks[7].steps[13] = BeatStep::on(70);
    }

    // ═════════════════════════════════════════════════════════════════════════
    // Layout Calculations
    // ═════════════════════════════════════════════════════════════════════════

    fn transport_h(&self) -> u32 { 48 }

    fn track_label_w(&self) -> u32 { 120 }

    fn scope_w(&self) -> u32 { self.fb_w.saturating_sub(self.track_label_w() + self.grid_w()).max(120) }

    fn grid_w(&self) -> u32 {
        // Each step pad is ~36px wide
        let step_w = ((self.fb_w - self.track_label_w() - 120) / self.tracks[0].num_steps as u32).max(20).min(44);
        step_w * self.tracks[0].num_steps as u32
    }

    fn step_w(&self) -> u32 {
        self.grid_w() / self.tracks[0].num_steps as u32
    }

    fn track_row_h(&self) -> u32 { 32 }

    fn grid_h(&self) -> u32 {
        // header row + 8 track rows
        24 + MAX_BEAT_TRACKS as u32 * self.track_row_h()
    }

    fn seq_y(&self) -> u32 { self.transport_h() }
    fn seq_grid_x(&self) -> u32 { self.track_label_w() }
    fn scope_x(&self) -> u32 { self.track_label_w() + self.grid_w() }

    fn bottom_y(&self) -> u32 { self.seq_y() + self.grid_h() + 2 }
    fn bottom_h(&self) -> u32 { self.fb_h.saturating_sub(self.bottom_y() + 48) }
    fn status_y(&self) -> u32 { self.fb_h.saturating_sub(48) }

    // ═════════════════════════════════════════════════════════════════════════
    // Full Draw
    // ═════════════════════════════════════════════════════════════════════════

    /// Draw the entire Beat Studio UI
    pub fn draw(&self) {
        if self.fb_w == 0 || self.fb_h == 0 { return; }

        // Full background
        crate::framebuffer::fill_rect(0, 0, self.fb_w, self.fb_h, colors::BG_DARK);

        self.draw_transport();
        self.draw_track_labels();
        self.draw_step_grid();
        self.draw_scope();
        self.draw_bottom_panel();
        self.draw_status_bar();
    }

    // ─── Transport Bar ──────────────────────────────────────────────────

    fn draw_transport(&self) {
        let h = self.transport_h();
        crate::framebuffer::fill_rect(0, 0, self.fb_w, h, colors::TRANSPORT_BG);

        // ♫ TrustDAW Beat Studio
        crate::framebuffer::draw_text("TrustDAW Beat Studio", 8, 4, colors::TEXT_ACCENT);

        // Transport buttons area
        let bx = 220;
        // Stop box
        let stop_c = if !self.playing { colors::TEXT_BRIGHT } else { colors::STOP_GRAY };
        crate::framebuffer::fill_rect(bx, 4, 14, 14, stop_c);

        // Play triangle (simple rectangle representation)
        let play_c = if self.playing { colors::PLAY_GREEN } else { colors::STOP_GRAY };
        crate::framebuffer::fill_rect(bx + 22, 4, 4, 14, play_c);
        crate::framebuffer::fill_rect(bx + 26, 6, 3, 10, play_c);
        crate::framebuffer::fill_rect(bx + 29, 8, 2, 6, play_c);

        // Record circle
        let rec_c = if self.recording { colors::REC_RED } else { colors::STOP_GRAY };
        crate::framebuffer::fill_circle(bx + 52, 11, 6, rec_c);

        // BPM display
        let bpm_str = format!("BPM:{}", self.bpm);
        crate::framebuffer::draw_text(&bpm_str, bx + 80, 4, colors::TEXT_BRIGHT);

        // Position display
        let bar = self.current_step / 16 + 1;
        let beat = (self.current_step % 16) / 4 + 1;
        let sub = self.current_step % 4 + 1;
        let pos_str = format!("{}:{}.{}", bar, beat, sub);
        crate::framebuffer::draw_text(&pos_str, bx + 160, 4, colors::PLAY_GREEN);

        // Swing
        let swing_str = format!("Swing:{}%", self.swing);
        crate::framebuffer::draw_text(&swing_str, bx + 240, 4, colors::TEXT_SECONDARY);

        // Steps display
        let steps_str = format!("{} steps", self.tracks[0].num_steps);
        crate::framebuffer::draw_text(&steps_str, bx + 340, 4, colors::TEXT_SECONDARY);

        // Second line: key/mode info
        crate::framebuffer::draw_text("Key: C minor", 8, 24, colors::TEXT_DIM);

        let track_name = self.tracks[self.cursor_track].name_str();
        let sel_str = format!("Track: {} [{}]", track_name, self.cursor_track);
        crate::framebuffer::draw_text(&sel_str, 140, 24, colors::TEXT_SECONDARY);

        let oct_str = format!("Oct:{}", 4i8 + self.octave);
        crate::framebuffer::draw_text(&oct_str, 340, 24, colors::TEXT_SECONDARY);

        let vel_str = format!("Vel:{}", self.velocity);
        crate::framebuffer::draw_text(&vel_str, 420, 24, colors::TEXT_SECONDARY);

        // Transport border
        crate::framebuffer::draw_hline(0, h - 1, self.fb_w, colors::BORDER_BRIGHT);
    }

    // ─── Track Labels (Left Panel) ──────────────────────────────────────

    fn draw_track_labels(&self) {
        let x = 0;
        let y = self.seq_y();
        let w = self.track_label_w();
        let row_h = self.track_row_h();

        // Header
        crate::framebuffer::fill_rect(x, y, w, 24, colors::BG_HEADER);
        crate::framebuffer::draw_text("TRACKS", 8, y + 4, colors::TEXT_DIM);
        crate::framebuffer::draw_hline(x, y + 23, w, colors::BORDER);

        for i in 0..MAX_BEAT_TRACKS {
            let ry = y + 24 + i as u32 * row_h;
            let is_selected = i == self.cursor_track;

            // Row background
            let bg = if is_selected { colors::BG_ELEVATED } else { colors::BG_PANEL };
            crate::framebuffer::fill_rect(x, ry, w, row_h, bg);

            // Color indicator (left edge, 4px wide)
            crate::framebuffer::fill_rect(x, ry, 4, row_h, self.tracks[i].color);

            // Selection arrow
            if is_selected {
                crate::framebuffer::draw_text(">", 6, ry + 8, colors::STEP_CURSOR);
            }

            // Track name
            let name = self.tracks[i].name_str();
            let name_c = if self.tracks[i].muted { colors::TEXT_DIM }
                         else { colors::TEXT_PRIMARY };
            crate::framebuffer::draw_text(name, 18, ry + 8, name_c);

            // Mute/Solo indicators
            if self.tracks[i].muted {
                crate::framebuffer::draw_text("M", 82, ry + 8, colors::MUTE_ORANGE);
            }
            if self.tracks[i].solo {
                crate::framebuffer::draw_text("S", 96, ry + 8, colors::SOLO_YELLOW);
            }

            // Bottom border
            crate::framebuffer::draw_hline(x, ry + row_h - 1, w, colors::DIVIDER);
        }

        // Right border of track panel
        crate::framebuffer::draw_vline(w - 1, y, self.grid_h(), colors::BORDER);
    }

    // ─── Step Sequencer Grid ────────────────────────────────────────────

    fn draw_step_grid(&self) {
        let gx = self.seq_grid_x();
        let gy = self.seq_y();
        let sw = self.step_w();
        let row_h = self.track_row_h();
        let num_s = self.tracks[0].num_steps;

        // Header: step numbers
        crate::framebuffer::fill_rect(gx, gy, self.grid_w(), 24, colors::BG_HEADER);
        for s in 0..num_s {
            let sx = gx + s as u32 * sw;
            let num_str = format!("{}", s + 1);
            // Highlight every beat boundary
            let color = if s % 4 == 0 { colors::TEXT_PRIMARY } else { colors::TEXT_DIM };
            crate::framebuffer::draw_text(&num_str, sx + 2, gy + 4, color);

            // Beat division line
            if s % 4 == 0 && s > 0 {
                crate::framebuffer::draw_vline(sx, gy, self.grid_h(), colors::STEP_BEAT_DIV);
            }
        }
        crate::framebuffer::draw_hline(gx, gy + 23, self.grid_w(), colors::BORDER);

        // Draw step pads for each track
        for t in 0..MAX_BEAT_TRACKS {
            let ry = gy + 24 + t as u32 * row_h;

            for s in 0..num_s {
                let sx = gx + s as u32 * sw;
                let step = &self.tracks[t].steps[s];

                // Pad dimensions (with 2px margin)
                let pad_x = sx + 2;
                let pad_y = ry + 2;
                let pad_w = sw.saturating_sub(4).max(4);
                let pad_h = row_h.saturating_sub(4).max(4);

                // Determine pad color
                let is_cursor = t == self.cursor_track && s == self.cursor_step;
                let is_playhead = self.playing && s == self.current_step;

                let pad_color = if step.active {
                    if is_playhead {
                        colors::STEP_PLAYHEAD  // Bright green when playing this step
                    } else {
                        // Use track color with velocity brightness
                        let brightness = step.velocity as u32 * 100 / 127;
                        adjust_brightness(self.tracks[t].color, brightness.max(40))
                    }
                } else if is_playhead {
                    colors::BG_ELEVATED  // Dimly lit when playhead passes
                } else {
                    colors::STEP_OFF
                };

                // Draw pad
                crate::framebuffer::fill_rect(pad_x, pad_y, pad_w, pad_h, pad_color);

                // Cursor highlight (green border)
                if is_cursor {
                    crate::framebuffer::draw_rect(pad_x.saturating_sub(1), pad_y.saturating_sub(1),
                        pad_w + 2, pad_h + 2, colors::STEP_CURSOR);
                }

                // Step border
                crate::framebuffer::draw_rect(pad_x, pad_y, pad_w, pad_h, colors::STEP_BORDER);
            }

            // Row divider
            crate::framebuffer::draw_hline(gx, ry + row_h - 1, self.grid_w(), colors::DIVIDER);
        }

        // Playhead vertical line across the entire grid
        if self.playing {
            let px = gx + self.current_step as u32 * sw + sw / 2;
            crate::framebuffer::draw_vline(px, gy, self.grid_h(), colors::STEP_PLAYHEAD);
        }
    }

    // ─── Scope & Spectrum (Right Panel) ─────────────────────────────────

    fn draw_scope(&self) {
        let sx = self.scope_x();
        let sy = self.seq_y();
        let sw = self.scope_w();
        let sh = self.grid_h();

        crate::framebuffer::fill_rect(sx, sy, sw, sh, colors::SCOPE_BG);
        crate::framebuffer::draw_vline(sx, sy, sh, colors::BORDER);

        // Scope header
        crate::framebuffer::fill_rect(sx, sy, sw, 24, colors::BG_HEADER);
        crate::framebuffer::draw_text("SCOPE", sx + 8, sy + 4, colors::TEXT_DIM);
        crate::framebuffer::draw_hline(sx, sy + 23, sw, colors::BORDER);

        // Waveform scope area (top half)
        let scope_y = sy + 24;
        let scope_h = sh / 2 - 24;
        let center_y = scope_y + scope_h / 2;

        // Center line
        crate::framebuffer::draw_hline(sx + 4, center_y, sw - 8, colors::DIVIDER);

        // Draw waveform from scope buffer
        let draw_w = (sw - 8).min(256) as usize;
        for i in 1..draw_w {
            let idx1 = (self.scope_pos + i - 1) % 256;
            let idx2 = (self.scope_pos + i) % 256;

            let y1 = center_y as i32 - (self.scope_buffer[idx1] as i32 * scope_h as i32 / 2) / 32768;
            let y2 = center_y as i32 - (self.scope_buffer[idx2] as i32 * scope_h as i32 / 2) / 32768;

            let py1 = (y1.max(scope_y as i32) as u32).min(scope_y + scope_h);
            let py2 = (y2.max(scope_y as i32) as u32).min(scope_y + scope_h);

            // Draw vertical line between the two points
            let min_y = py1.min(py2);
            let max_y = py1.max(py2);
            let line_h = (max_y - min_y).max(1);
            crate::framebuffer::fill_rect(sx + 4 + i as u32, min_y, 1, line_h, colors::SCOPE_LINE);
        }

        // Spectrum analyzer area (bottom half)
        let spec_y = sy + sh / 2;
        let spec_h = sh / 2;

        crate::framebuffer::draw_hline(sx, spec_y, sw, colors::BORDER);
        crate::framebuffer::draw_text("SPECTRUM", sx + 8, spec_y + 4, colors::TEXT_DIM);

        let bar_area_y = spec_y + 20;
        let bar_area_h = spec_h - 24;
        let num_bars = 16usize;
        let bar_w = ((sw - 16) / num_bars as u32).max(4);

        for i in 0..num_bars {
            let bx = sx + 8 + i as u32 * bar_w;
            let level = self.spectrum[i].min(100) as u32;
            let filled_h = bar_area_h * level / 100;
            let bar_y = bar_area_y + bar_area_h - filled_h;

            // Bar color based on level
            let bar_color = if level > 85 { colors::SPECTRUM_5 }
                           else if level > 70 { colors::SPECTRUM_4 }
                           else if level > 50 { colors::SPECTRUM_3 }
                           else if level > 30 { colors::SPECTRUM_2 }
                           else { colors::SPECTRUM_1 };

            // Dark background bar
            crate::framebuffer::fill_rect(bx, bar_area_y, bar_w - 2, bar_area_h, colors::METER_BG);

            // Filled bar
            if filled_h > 0 {
                crate::framebuffer::fill_rect(bx, bar_y, bar_w - 2, filled_h, bar_color);
            }
        }
    }

    // ─── Bottom Panel (Mixer + Keyboard + Info) ─────────────────────────

    fn draw_bottom_panel(&self) {
        let by = self.bottom_y();
        let bh = self.bottom_h();

        crate::framebuffer::draw_hline(0, by, self.fb_w, colors::BORDER_BRIGHT);

        // Three panels side by side
        self.draw_mixer(0, by + 1, self.track_label_w(), bh);
        self.draw_visual_keyboard(self.track_label_w(), by + 1, self.grid_w(), bh);
        self.draw_info_panel(self.scope_x(), by + 1, self.scope_w(), bh);
    }

    // ─── Mixer Panel ────────────────────────────────────────────────────

    fn draw_mixer(&self, x: u32, y: u32, w: u32, h: u32) {
        crate::framebuffer::fill_rect(x, y, w, h, colors::BG_PANEL);

        // Header
        crate::framebuffer::draw_text("MIXER", x + 8, y + 4, colors::TEXT_DIM);

        let fader_h = h.saturating_sub(40);
        let num_tracks = MAX_BEAT_TRACKS;
        let fader_w = ((w - 8) / num_tracks as u32).max(8);

        for i in 0..num_tracks {
            let fx = x + 4 + i as u32 * fader_w;
            let fy = y + 24;

            // Track initial letter
            let initial = self.tracks[i].name_str().chars().next().unwrap_or('?');
            let initial_str = format!("{}", initial);
            crate::framebuffer::draw_text(&initial_str, fx + 2, fy, self.tracks[i].color);

            // Fader background
            let fader_x = fx + 2;
            let fader_y = fy + 18;
            let fader_w_inner = fader_w.saturating_sub(6).max(3);
            let fader_h_inner = fader_h.saturating_sub(30);

            crate::framebuffer::fill_rect(fader_x, fader_y, fader_w_inner, fader_h_inner, colors::METER_BG);

            // Fader level
            let level = self.tracks[i].volume as u32 * fader_h_inner / 255;
            if level > 0 {
                let level_y = fader_y + fader_h_inner - level;
                let level_color = if self.tracks[i].muted { colors::TEXT_DIM }
                    else if level > fader_h_inner * 90 / 100 { colors::METER_RED }
                    else if level > fader_h_inner * 70 / 100 { colors::METER_YELLOW }
                    else { colors::METER_GREEN };
                crate::framebuffer::fill_rect(fader_x, level_y, fader_w_inner, level, level_color);
            }

            // Fader border
            crate::framebuffer::draw_rect(fader_x, fader_y, fader_w_inner, fader_h_inner, colors::BORDER);
        }

        // Right border
        crate::framebuffer::draw_vline(x + w - 1, y, h, colors::BORDER);
    }

    // ─── Visual Keyboard ────────────────────────────────────────────────

    fn draw_visual_keyboard(&self, x: u32, y: u32, w: u32, h: u32) {
        crate::framebuffer::fill_rect(x, y, w, h, colors::BG_MAIN);

        // Header
        crate::framebuffer::draw_text("KEYBOARD", x + 8, y + 4, colors::TEXT_DIM);

        // Draw 2 octaves of piano keys
        let kb_y = y + 22;
        let kb_h = h.saturating_sub(26);
        let white_h = kb_h;
        let black_h = kb_h * 60 / 100;

        // 2 octaves = 14 white keys (C D E F G A B x2)
        let num_white = 14u32;
        let key_w = (w - 16) / num_white;
        let kb_x = x + 8;

        let base_octave = (4 + self.octave) as u8;

        // Draw white keys first
        for i in 0..num_white {
            let kx = kb_x + i * key_w;

            // Which note is this white key?
            let oct_offset = i / 7;
            let key_in_oct = i % 7;
            let semitone = match key_in_oct {
                0 => 0,  // C
                1 => 2,  // D
                2 => 4,  // E
                3 => 5,  // F
                4 => 7,  // G
                5 => 9,  // A
                6 => 11, // B
                _ => 0,
            };
            let midi_note = (base_octave + oct_offset as u8) * 12 + semitone as u8;

            let is_pressed = midi_note < 128 && self.keys_pressed[midi_note as usize];
            let key_color = if is_pressed { colors::KEY_PRESSED } else { colors::KEY_WHITE };

            crate::framebuffer::fill_rect(kx, kb_y, key_w - 2, white_h, key_color);
            crate::framebuffer::draw_rect(kx, kb_y, key_w - 2, white_h, colors::BORDER);

            // Label (note name at bottom of key)
            let note_names = ["C", "D", "E", "F", "G", "A", "B"];
            if key_in_oct < 7 {
                let label = note_names[key_in_oct as usize];
                let label_color = if is_pressed { colors::TEXT_BRIGHT } else { colors::KEY_LABEL };
                crate::framebuffer::draw_text(label, kx + key_w / 2 - 4, kb_y + white_h - 18, label_color);
            }

            // Octave number under C keys
            if key_in_oct == 0 {
                let oct_str = format!("{}", base_octave + oct_offset as u8);
                crate::framebuffer::draw_text(&oct_str, kx + 2, kb_y + 2, colors::TEXT_DIM);
            }
        }

        // Draw black keys on top
        for i in 0..num_white {
            let oct_offset = i / 7;
            let key_in_oct = i % 7;

            // Black keys exist after C, D, F, G, A (not after E, B)
            let semitone = match key_in_oct {
                0 => Some(1),  // C#
                1 => Some(3),  // D#
                // No E#
                3 => Some(6),  // F#
                4 => Some(8),  // G#
                5 => Some(10), // A#
                _ => None,
            };

            if let Some(semi) = semitone {
                let midi_note = (base_octave + oct_offset as u8) * 12 + semi as u8;
                let is_pressed = midi_note < 128 && self.keys_pressed[midi_note as usize];

                let bx = kb_x + i * key_w + key_w * 2 / 3;
                let bw = key_w * 2 / 3;
                let key_color = if is_pressed { colors::KEY_PRESSED } else { colors::KEY_BLACK };

                crate::framebuffer::fill_rect(bx, kb_y, bw, black_h, key_color);
                crate::framebuffer::draw_rect(bx, kb_y, bw, black_h, colors::BORDER);
            }
        }

        // Keyboard shortcut labels beneath
        let label_y = kb_y + white_h + 2;
        if label_y + 16 < y + h {
            crate::framebuffer::draw_text("[Z X C V B N M] Low  [Q W E R T Y U] High", x + 8, label_y, colors::TEXT_DIM);
        }
    }

    // ─── Info Panel ─────────────────────────────────────────────────────

    fn draw_info_panel(&self, x: u32, y: u32, w: u32, h: u32) {
        crate::framebuffer::fill_rect(x, y, w, h, colors::BG_PANEL);
        crate::framebuffer::draw_vline(x, y, h, colors::BORDER);

        crate::framebuffer::draw_text("INFO", x + 8, y + 4, colors::TEXT_DIM);

        let mut ly = y + 24;
        let line_h = 18u32;

        // Track info
        let t = &self.tracks[self.cursor_track];
        let name_str = format!("Track: {}", t.name_str());
        crate::framebuffer::draw_text(&name_str, x + 8, ly, colors::TEXT_PRIMARY);
        ly += line_h;

        let wave_str = format!("Wave: {}", t.waveform.name());
        crate::framebuffer::draw_text(&wave_str, x + 8, ly, colors::TEXT_SECONDARY);
        ly += line_h;

        let type_str = if t.is_drum { "Type: Drum" } else { "Type: Melodic" };
        crate::framebuffer::draw_text(type_str, x + 8, ly, colors::TEXT_SECONDARY);
        ly += line_h;

        let note_name = crate::audio::tables::midi_to_note_name(t.base_note);
        let note_oct = crate::audio::tables::midi_octave(t.base_note);
        let note_str = format!("Note: {}{}", note_name, note_oct);
        crate::framebuffer::draw_text(&note_str, x + 8, ly, colors::TEXT_SECONDARY);
        ly += line_h;

        let active_str = format!("Steps: {}/{}", t.active_count(), t.num_steps);
        crate::framebuffer::draw_text(&active_str, x + 8, ly, colors::TEXT_SECONDARY);
        ly += line_h;

        let vol_str = format!("Vol: {}", t.volume);
        crate::framebuffer::draw_text(&vol_str, x + 8, ly, colors::TEXT_SECONDARY);
        ly += line_h;

        let pan_str = if t.pan == 0 { String::from("Pan: C") }
                     else if t.pan > 0 { format!("Pan: R{}", t.pan) }
                     else { format!("Pan: L{}", -t.pan) };
        crate::framebuffer::draw_text(&pan_str, x + 8, ly, colors::TEXT_SECONDARY);
        ly += line_h + 8;

        // Cursor position
        let cur_str = format!("Step: {}/{}", self.cursor_step + 1, t.num_steps);
        crate::framebuffer::draw_text(&cur_str, x + 8, ly, colors::TEXT_ACCENT);
        ly += line_h;

        // Step velocity at cursor
        let step = &t.steps[self.cursor_step];
        if step.active {
            let vel_str = format!("Hit Vel: {}", step.velocity);
            crate::framebuffer::draw_text(&vel_str, x + 8, ly, colors::TEXT_PRIMARY);
        } else {
            crate::framebuffer::draw_text("Hit: ---", x + 8, ly, colors::TEXT_DIM);
        }
    }

    // ─── Status Bar ─────────────────────────────────────────────────────

    fn draw_status_bar(&self) {
        let sy = self.status_y();
        crate::framebuffer::fill_rect(0, sy, self.fb_w, 48, colors::TRANSPORT_BG);
        crate::framebuffer::draw_hline(0, sy, self.fb_w, colors::BORDER_BRIGHT);

        crate::framebuffer::draw_text(
            "[Space] Play/Stop  [Enter] Toggle Step  [R] Record  [Tab] Track  [+/-] BPM",
            8, sy + 6, colors::TEXT_DIM
        );
        crate::framebuffer::draw_text(
            "[Arrows] Navigate  [Z-M] Low Piano  [Q-P] High Piano  [F8] Export  [Esc] Exit",
            8, sy + 24, colors::TEXT_DIM
        );
    }

    // ═════════════════════════════════════════════════════════════════════════
    // Playback — Render beat and play via HDA
    // ═════════════════════════════════════════════════════════════════════════

    /// Render all tracks into a mixed audio buffer for one loop
    pub fn render_loop(&self) -> Vec<i16> {
        let step_samples = (60 * SAMPLE_RATE) / (self.bpm as u32 * 4); // 16th note duration
        let total_steps = self.tracks[0].num_steps;
        let total_frames = step_samples as usize * total_steps;
        let total_samples = total_frames * CHANNELS as usize;

        let mut mix = vec![0i32; total_samples];

        // Check for solo
        let any_solo = self.tracks.iter().any(|t| t.solo);

        for t in &self.tracks {
            if t.muted { continue; }
            if any_solo && !t.solo { continue; }

            let mut engine = SynthEngine::new();
            engine.set_waveform(t.waveform);
            engine.envelope = t.envelope;

            // Render this track's steps
            for s in 0..total_steps {
                let step = &t.steps[s];
                if !step.active { continue; }

                let midi = t.note_at(s);
                if midi == 0 { continue; }

                // Note on at the start of this step
                let note_start = s * step_samples as usize;
                let note_dur = step_samples as usize; // One step worth

                // Render a note
                let vel = step.velocity;
                let note_samples = engine.render_note(midi, vel, 
                    (note_dur as u32 * 1000) / SAMPLE_RATE);

                // Mix into output buffer
                for (j, &sample) in note_samples.iter().enumerate() {
                    let idx = note_start * CHANNELS as usize + j;
                    if idx < mix.len() {
                        mix[idx] += sample as i32;
                    }
                }
            }
        }

        // Convert to i16 with soft clipping
        mix.iter().map(|&s| {
            let clamped = s.clamp(-32767, 32767);
            clamped as i16
        }).collect()
    }

    /// Calculate step duration in milliseconds
    pub fn step_duration_ms(&self) -> u32 {
        60_000 / (self.bpm as u32 * 4) // 16th note at current BPM
    }

    /// Update scope buffer with recent audio
    pub fn update_scope(&mut self, samples: &[i16]) {
        // Take every Nth sample to fit in scope buffer
        let step = (samples.len() / 256).max(1);
        for i in 0..256 {
            let idx = (i * step).min(samples.len().saturating_sub(1));
            self.scope_buffer[i] = samples[idx];
        }
        self.scope_pos = 0;
    }

    /// Update fake spectrum from current beat state
    pub fn update_spectrum(&mut self) {
        // Simulate spectrum bars based on active tracks at current step
        for i in 0..16 {
            let mut level: u32 = 0;
            for t in &self.tracks {
                if !t.muted && self.current_step < t.num_steps {
                    let step = &t.steps[self.current_step];
                    if step.active {
                        // Each track contributes to nearby frequency bands
                        let band = (t.base_note as u32 / 8).min(15);
                        let distance = (band as i32 - i as i32).unsigned_abs();
                        if distance < 4 {
                            level += step.velocity as u32 * (4 - distance) / 4;
                        }
                    }
                }
            }
            self.spectrum[i] = level.min(100) as u8;
        }
    }

    /// Trigger a note audibly (for live keyboard playing)
    pub fn trigger_note(&mut self, midi_note: u8, velocity: u8) {
        if midi_note < 128 {
            self.keys_pressed[midi_note as usize] = true;
        }
        // Get waveform from currently selected track
        let wf = self.tracks[self.cursor_track].waveform;
        let _ = crate::audio::set_waveform(wf);
        let _ = crate::audio::play_midi_note(midi_note, velocity, 150);
    }

    /// Release a note visually
    pub fn release_note(&mut self, midi_note: u8) {
        if midi_note < 128 {
            self.keys_pressed[midi_note as usize] = false;
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Interactive Launch — Main entry point for the Beat Studio
// ═══════════════════════════════════════════════════════════════════════════════

/// Launch the Beat Studio (blocking interactive mode)
pub fn launch() -> Result<(), &'static str> {
    // Ensure audio subsystem is ready
    crate::audio::init().ok(); // Non-fatal if already init

    let mut studio = BeatStudio::new();
    studio.draw();

    crate::serial_println!("[BEAT_STUDIO] Launched — press Esc to exit");

    loop {
        if let Some(scancode) = crate::keyboard::try_read_key() {
            let is_release = scancode & 0x80 != 0;
            let pressed_code = scancode & 0x7F;

            // Handle piano key releases (visual feedback)
            if is_release {
                if let Some(midi) = super::keyboard_midi::scancode_to_midi(pressed_code) {
                    studio.release_note(midi);
                    studio.draw();
                }
                continue;
            }

            let mut redraw = true;

            match scancode {
                // ── Escape: exit ──
                0x01 => break,

                // ── Space: play/stop ──
                0x39 => {
                    if studio.playing {
                        studio.playing = false;
                        studio.current_step = 0;
                        let _ = crate::audio::stop();
                    } else {
                        studio.playing = true;
                        // Render and play the loop
                        let audio = studio.render_loop();
                        studio.update_scope(&audio);
                        let dur_ms = studio.step_duration_ms() * studio.tracks[0].num_steps as u32;
                        let _ = crate::drivers::hda::write_samples_and_play(&audio, dur_ms);
                        // Animate playhead
                        animate_playhead(&mut studio);
                    }
                }

                // ── Enter: toggle step ──
                0x1C => {
                    studio.tracks[studio.cursor_track].toggle_step(studio.cursor_step);
                }

                // ── Tab: next track ──
                0x0F => {
                    studio.cursor_track = (studio.cursor_track + 1) % MAX_BEAT_TRACKS;
                }

                // ── Arrow keys: navigate grid ──
                0x4D => { // Right
                    let max = studio.tracks[studio.cursor_track].num_steps;
                    studio.cursor_step = (studio.cursor_step + 1) % max;
                }
                0x4B => { // Left
                    let max = studio.tracks[studio.cursor_track].num_steps;
                    if studio.cursor_step == 0 {
                        studio.cursor_step = max - 1;
                    } else {
                        studio.cursor_step -= 1;
                    }
                }
                0x50 => { // Down
                    studio.cursor_track = (studio.cursor_track + 1) % MAX_BEAT_TRACKS;
                }
                0x48 => { // Up
                    if studio.cursor_track == 0 {
                        studio.cursor_track = MAX_BEAT_TRACKS - 1;
                    } else {
                        studio.cursor_track -= 1;
                    }
                }

                // ── +/- : BPM control ──
                0x0D => { // = / + key
                    studio.bpm = (studio.bpm + 5).min(300);
                }
                0x0C => { // - key
                    studio.bpm = studio.bpm.saturating_sub(5).max(40);
                }

                // ── Page Up/Down: octave ──
                0x49 => { // Page Up
                    studio.octave = (studio.octave + 1).min(4);
                }
                0x51 => { // Page Down
                    studio.octave = (studio.octave - 1).max(-4);
                }

                // ── M key: toggle mute on current track ──
                0x32 => {
                    let ct = studio.cursor_track;
                    studio.tracks[ct].muted = !studio.tracks[ct].muted;
                }

                // ── F1: cycle waveform on current track ──
                0x3B => {
                    let ct = studio.cursor_track;
                    studio.tracks[ct].waveform = match studio.tracks[ct].waveform {
                        Waveform::Sine => Waveform::Square,
                        Waveform::Square => Waveform::Sawtooth,
                        Waveform::Sawtooth => Waveform::Triangle,
                        Waveform::Triangle => Waveform::Noise,
                        Waveform::Noise => Waveform::Sine,
                    };
                }

                // ── F2: toggle 16/32 steps ──
                0x3C => {
                    for t in studio.tracks.iter_mut() {
                        t.num_steps = if t.num_steps == 16 { 32 } else { 16 };
                    }
                    if studio.cursor_step >= studio.tracks[0].num_steps {
                        studio.cursor_step = 0;
                    }
                }

                // ── F8: export WAV ──
                0x42 => {
                    let audio = studio.render_loop();
                    let _ = super::wav_export::export_wav(
                        "/home/beat.wav", &audio, SAMPLE_RATE, CHANNELS as u16
                    );
                    crate::serial_println!("[BEAT_STUDIO] Exported to /home/beat.wav");
                }

                // ── R: record toggle ──
                0x13 => {
                    studio.recording = !studio.recording;
                }

                // ── Backspace: clear all steps on current track ──
                0x0E => {
                    let ct = studio.cursor_track;
                    for s in 0..studio.tracks[ct].num_steps {
                        studio.tracks[ct].steps[s] = BeatStep::off();
                    }
                }

                // ── Piano keys (live play) ──
                _ => {
                    if let Some(midi) = super::keyboard_midi::scancode_to_midi(scancode) {
                        studio.trigger_note(midi, studio.velocity);

                        // If recording, also place the note on the current step
                        if studio.recording {
                            let ct = studio.cursor_track;
                            let cs = studio.cursor_step;
                            let base = studio.tracks[ct].base_note;
                            let offset = midi as i8 - base as i8;
                            studio.tracks[ct].steps[cs] = BeatStep::on_note(studio.velocity, offset);
                            // Auto-advance cursor
                            let max = studio.tracks[ct].num_steps;
                            studio.cursor_step = (studio.cursor_step + 1) % max;
                        }
                    } else {
                        redraw = false;
                    }
                }
            }

            if redraw {
                studio.update_spectrum();
                studio.draw();
            }
        }

        // Spin loop to prevent 100% CPU
        for _ in 0..3000 {
            core::hint::spin_loop();
        }
    }

    // Cleanup
    let _ = crate::audio::stop();
    crate::serial_println!("[BEAT_STUDIO] Exited");
    Ok(())
}

/// Animate the playhead during playback (blocking)
fn animate_playhead(studio: &mut BeatStudio) {
    let total_steps = studio.tracks[0].num_steps;
    let step_ms = studio.step_duration_ms();

    for s in 0..total_steps {
        studio.current_step = s;
        studio.update_spectrum();
        studio.draw();

        // Check for stop (Escape or Space)
        let mut stop = false;
        // Simple timing: spin for step_ms
        let spin_iters = step_ms as u64 * 5000; // Rough approximation
        for _ in 0..spin_iters {
            if let Some(sc) = crate::keyboard::try_read_key() {
                if sc & 0x80 != 0 { continue; } // Ignore releases
                if sc == 0x01 || sc == 0x39 { // Esc or Space
                    stop = true;
                    break;
                }
            }
            core::hint::spin_loop();
        }

        if stop {
            studio.playing = false;
            studio.current_step = 0;
            let _ = crate::audio::stop();
            break;
        }
    }

    if studio.playing {
        // Loop completed — stop
        studio.playing = false;
        studio.current_step = 0;
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Helper functions
// ═══════════════════════════════════════════════════════════════════════════════

/// Adjust color brightness (0-100%)
fn adjust_brightness(color: u32, brightness: u32) -> u32 {
    let r = ((color >> 16) & 0xFF) * brightness / 100;
    let g = ((color >> 8) & 0xFF) * brightness / 100;
    let b = (color & 0xFF) * brightness / 100;
    (r.min(255) << 16) | (g.min(255) << 8) | b.min(255)
}
