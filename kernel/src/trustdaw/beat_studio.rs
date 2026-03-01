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

    /// Load a funky house beat — 124 BPM, groovy, showcase-ready
    /// 32 steps = 2 bars of variation for extra groove
    pub fn load_funky_house(&mut self) {
        // Configure for funky house
        self.bpm = 124;
        self.swing = 58; // slight shuffle feel

        // Expand to 32 steps (2 bars) for variation
        for t in self.tracks.iter_mut() {
            t.num_steps = 32;
            // Clear all steps first
            for s in 0..MAX_STEPS {
                t.steps[s] = BeatStep::off();
            }
        }

        // ─── Rename & reconfigure tracks for house ───
        self.tracks[0] = BeatTrack::new("Kick",    36, Waveform::Sine,     colors::TRACK_COLORS[0], true);
        self.tracks[1] = BeatTrack::new("Clap",    39, Waveform::Noise,    colors::TRACK_COLORS[1], true);
        self.tracks[2] = BeatTrack::new("HiHat",   42, Waveform::Noise,    colors::TRACK_COLORS[2], true);
        self.tracks[3] = BeatTrack::new("Bass",    36, Waveform::Square,   colors::TRACK_COLORS[3], false);
        self.tracks[4] = BeatTrack::new("Stab",    60, Waveform::Sawtooth, colors::TRACK_COLORS[4], false);
        self.tracks[5] = BeatTrack::new("Chord",   60, Waveform::Triangle, colors::TRACK_COLORS[5], false);
        self.tracks[6] = BeatTrack::new("Lead",    72, Waveform::Sawtooth, colors::TRACK_COLORS[6], false);
        self.tracks[7] = BeatTrack::new("Perc",    56, Waveform::Noise,    colors::TRACK_COLORS[7], true);

        // Expand to 32 steps for all tracks
        for t in self.tracks.iter_mut() {
            t.num_steps = 32;
        }

        // ─── House envelopes ───
        self.tracks[0].envelope = Envelope::new(1, 120, 0, 60);    // Kick: deep thump
        self.tracks[1].envelope = Envelope::new(1, 60, 0, 40);     // Clap: tight snap
        self.tracks[2].envelope = Envelope::new(1, 15, 0, 10);     // HiHat: crispy short
        self.tracks[3].envelope = Envelope::new(3, 100, 60, 80);   // Bass: funky pluck
        self.tracks[4].envelope = Envelope::new(2, 50, 0, 40);     // Stab: quick punch
        self.tracks[5].envelope = Envelope::pad();                   // Chord: lush pad
        self.tracks[6].envelope = Envelope::new(5, 150, 50, 120);  // Lead: singing
        self.tracks[7].envelope = Envelope::new(1, 25, 0, 15);     // Perc: snap

        // ══════════════════════════════════════════════
        // KICK — four-on-the-floor with ghost notes
        // ══════════════════════════════════════════════
        // Main kicks: every quarter note (steps 0, 4, 8, 12, 16, 20, 24, 28)
        for i in (0..32).step_by(4) {
            self.tracks[0].steps[i] = BeatStep::on(127);
        }
        // Ghost kick before beat 2 (bar 1) and beat 4 (bar 2)
        self.tracks[0].steps[3]  = BeatStep::on(50);
        self.tracks[0].steps[27] = BeatStep::on(45);

        // ══════════════════════════════════════════════
        // CLAP — beats 2 & 4, with funky pre-clap
        // ══════════════════════════════════════════════
        self.tracks[1].steps[4]  = BeatStep::on(120);   // Bar 1 beat 2
        self.tracks[1].steps[12] = BeatStep::on(120);   // Bar 1 beat 4
        self.tracks[1].steps[20] = BeatStep::on(120);   // Bar 2 beat 2
        self.tracks[1].steps[28] = BeatStep::on(120);   // Bar 2 beat 4
        // Funky flam/drag before beat 4 of bar 2
        self.tracks[1].steps[27] = BeatStep::on(60);    // Ghost clap
        self.tracks[1].steps[15] = BeatStep::on(50);    // Grace note

        // ══════════════════════════════════════════════
        // HIHAT — 16th note groove with velocity dynamics
        // ══════════════════════════════════════════════
        // Bar 1: driving 16th hats with off-beat accents
        for i in 0..16 {
            let vel = match i % 4 {
                0 => 90,   // downbeat
                2 => 100,  // off-beat accent (funky!)
                1 => 40,   // ghost
                3 => 55,   // ghost slightly louder
                _ => 50,
            };
            self.tracks[2].steps[i] = BeatStep::on(vel);
        }
        // Bar 2: variation — skip some hats for breathing room
        for i in 16..32 {
            let vel = match i % 4 {
                0 => 85,
                2 => 105,  // accented off-beat
                1 => 35,
                3 => 50,
                _ => 45,
            };
            self.tracks[2].steps[i] = BeatStep::on(vel);
        }
        // Open hat feel: remove some in bar 2 for variation
        self.tracks[2].steps[23] = BeatStep::off();
        self.tracks[2].steps[31] = BeatStep::off();

        // ══════════════════════════════════════════════
        // BASS — funky syncopated bassline in C minor
        // C=0, D=2, Eb=3, F=5, G=7, Ab=8, Bb=10
        // ══════════════════════════════════════════════
        // Bar 1: C – rest – C – Eb – rest – G – F – rest
        self.tracks[3].steps[0]  = BeatStep::on_note(120, 0);   // C
        self.tracks[3].steps[3]  = BeatStep::on_note(110, 0);   // C (syncopation!)
        self.tracks[3].steps[5]  = BeatStep::on_note(100, 3);   // Eb
        self.tracks[3].steps[8]  = BeatStep::on_note(115, 7);   // G
        self.tracks[3].steps[10] = BeatStep::on_note(105, 5);   // F
        self.tracks[3].steps[13] = BeatStep::on_note(95, 3);    // Eb (pickup)

        // Bar 2: Variation — Bb – rest – Ab – G – rest – F – Eb – C
        self.tracks[3].steps[16] = BeatStep::on_note(120, 10);  // Bb
        self.tracks[3].steps[19] = BeatStep::on_note(100, 8);   // Ab
        self.tracks[3].steps[21] = BeatStep::on_note(110, 7);   // G
        self.tracks[3].steps[24] = BeatStep::on_note(115, 5);   // F
        self.tracks[3].steps[26] = BeatStep::on_note(105, 3);   // Eb
        self.tracks[3].steps[29] = BeatStep::on_note(100, 0);   // C (resolve)

        // ══════════════════════════════════════════════
        // STAB — house chord stabs (off-beat hits)
        // Cm7 = C+Eb+G+Bb
        // ══════════════════════════════════════════════
        // Classic house off-beat stab pattern
        self.tracks[4].steps[2]  = BeatStep::on_note(100, 0);   // C stab
        self.tracks[4].steps[6]  = BeatStep::on_note(110, 0);   // C stab (accented)
        self.tracks[4].steps[10] = BeatStep::on_note(95, 3);    // Eb stab
        self.tracks[4].steps[14] = BeatStep::on_note(100, 0);   // C stab
        // Bar 2: more variation
        self.tracks[4].steps[18] = BeatStep::on_note(105, 7);   // G stab
        self.tracks[4].steps[22] = BeatStep::on_note(110, 5);   // F stab (tension!)
        self.tracks[4].steps[26] = BeatStep::on_note(100, 3);   // Eb stab
        self.tracks[4].steps[30] = BeatStep::on_note(95, 0);    // C resolve

        // ══════════════════════════════════════════════
        // CHORD — sustained house chord (Cm7 → Fm7)
        // ══════════════════════════════════════════════
        // Bar 1: Cm7 (C Eb G Bb) — long sustain
        self.tracks[5].steps[0]  = BeatStep::on_note(75, 0);    // C
        self.tracks[5].steps[8]  = BeatStep::on_note(70, 0);    // C re-trigger
        // Bar 2: Fm7 (F Ab C Eb) — chord change!
        self.tracks[5].steps[16] = BeatStep::on_note(75, 5);    // F
        self.tracks[5].steps[24] = BeatStep::on_note(70, 5);    // F re-trigger

        // ══════════════════════════════════════════════
        // LEAD — funky disco-house riff
        // ══════════════════════════════════════════════
        // Bar 1: C5 – Eb5 – G5 – F5 descend (classic house lick)
        self.tracks[6].steps[0]  = BeatStep::on_note(100, 0);   // C5
        self.tracks[6].steps[2]  = BeatStep::on_note(95, 3);    // Eb5
        self.tracks[6].steps[4]  = BeatStep::on_note(110, 7);   // G5 (peak!)
        self.tracks[6].steps[6]  = BeatStep::on_note(95, 5);    // F5
        self.tracks[6].steps[9]  = BeatStep::on_note(90, 3);    // Eb5
        self.tracks[6].steps[11] = BeatStep::on_note(85, 0);    // C5
        // Bar 2: call & response — higher phrase
        self.tracks[6].steps[16] = BeatStep::on_note(110, 7);   // G5
        self.tracks[6].steps[18] = BeatStep::on_note(100, 10);  // Bb5 (jazzy!)
        self.tracks[6].steps[20] = BeatStep::on_note(115, 12);  // C6 (peak octave!)
        self.tracks[6].steps[22] = BeatStep::on_note(100, 10);  // Bb5
        self.tracks[6].steps[24] = BeatStep::on_note(95, 7);    // G5
        self.tracks[6].steps[27] = BeatStep::on_note(90, 5);    // F5
        self.tracks[6].steps[29] = BeatStep::on_note(85, 3);    // Eb5 (resolve down)

        // ══════════════════════════════════════════════
        // PERC — shakers, rides, fills
        // ══════════════════════════════════════════════
        // Shaker on off-beats (every other 16th)
        for i in (1..32).step_by(2) {
            self.tracks[7].steps[i] = BeatStep::on(60);
        }
        // Accent the "and" of each beat
        for i in (2..32).step_by(4) {
            self.tracks[7].steps[i] = BeatStep::on(90);
        }
        // Fill at end of bar 2 (steps 28-31)
        self.tracks[7].steps[28] = BeatStep::on(100);
        self.tracks[7].steps[29] = BeatStep::on(110);
        self.tracks[7].steps[30] = BeatStep::on(120);
        self.tracks[7].steps[31] = BeatStep::on(127); // crash into next bar!

        // Set volumes for good mix balance
        self.tracks[0].volume = 220; // Kick: dominant
        self.tracks[1].volume = 190; // Clap: present
        self.tracks[2].volume = 150; // HiHat: background groove
        self.tracks[3].volume = 210; // Bass: fat
        self.tracks[4].volume = 160; // Stab: cutting through
        self.tracks[5].volume = 120; // Chord: bed
        self.tracks[6].volume = 170; // Lead: melodic focus
        self.tracks[7].volume = 130; // Perc: texture
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

    launch_interactive(&mut studio)
}

/// Launch the Beat Studio pre-loaded with Funky House beat
pub fn launch_funky() -> Result<(), &'static str> {
    crate::audio::init().ok();

    let mut studio = BeatStudio::new();
    studio.load_funky_house();
    studio.draw();

    crate::serial_println!("[BEAT_STUDIO] Funky House loaded — press Esc to exit");

    launch_interactive(&mut studio)
}

/// Internal: interactive loop shared by all launch modes
fn launch_interactive(studio: &mut BeatStudio) -> Result<(), &'static str> {
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
                        let _ = crate::drivers::hda::start_looped_playback(&audio);
                        // Animate playhead
                        animate_playhead(studio);
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

        // Wait exactly one step using PIT timer, check for stop
        match wait_ms_skip(step_ms as u64) {
            1 | 2 => {  // Esc or Space
                studio.playing = false;
                studio.current_step = 0;
                let _ = crate::audio::stop();
                return;
            }
            _ => {}
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

// ═══════════════════════════════════════════════════════════════════════════════
// MATRIX VISUALIZER — "Enter the Beat" showcase mode
// ═══════════════════════════════════════════════════════════════════════════════
//
// Fullscreen Matrix-style rain of characters, beat-reactive:
//   - Green falling glyphs (katakana-styled from ASCII)
//   - Beat hits cause bright flashes / column bursts
//   - Track info displayed as "decoded" Matrix text
//   - Step position shown as a glowing bar at bottom
//   - BPM pulse makes the whole screen breathe

/// Matrix rain column state
struct MatrixColumn {
    /// Current head Y position (pixels row index)
    head_y: i32,
    /// Speed (pixels per tick)
    speed: u8,
    /// Trail length (characters)
    trail_len: u8,
    /// Active (visible on screen)
    active: bool,
    /// Random character offset (for variety)
    char_offset: u8,
    /// Brightness multiplier (100 = normal, 200 = flash)
    flash: u8,
}

/// Matrix visual state
struct MatrixState {
    columns: Vec<MatrixColumn>,
    num_cols: usize,
    num_rows: usize,
    fb_w: u32,
    fb_h: u32,
    /// Frame counter for animation
    frame: u32,
    /// LFSR for pseudo-random
    lfsr: u32,
}

impl MatrixState {
    fn new() -> Self {
        let fb_w = crate::framebuffer::FB_WIDTH.load(Ordering::Relaxed) as u32;
        let fb_h = crate::framebuffer::FB_HEIGHT.load(Ordering::Relaxed) as u32;

        let num_cols = (fb_w / 8) as usize;   // 8px font width
        let num_rows = (fb_h / 16) as usize;  // 16px font height

        let mut columns = Vec::with_capacity(num_cols);
        let mut lfsr: u32 = 0xDEAD_BEEF;

        for i in 0..num_cols {
            lfsr = lfsr_next(lfsr);
            let speed = (lfsr % 4) as u8 + 1;
            lfsr = lfsr_next(lfsr);
            let trail = (lfsr % 12) as u8 + 4;
            lfsr = lfsr_next(lfsr);
            let start_y = -((lfsr % (num_rows as u32 * 2)) as i32);
            lfsr = lfsr_next(lfsr);
            let char_off = (lfsr % 94) as u8;

            columns.push(MatrixColumn {
                head_y: start_y,
                speed,
                trail_len: trail,
                active: i % 3 != 2, // ~66% active initially
                char_offset: char_off,
                flash: 100,
            });
        }

        Self {
            columns,
            num_cols,
            num_rows,
            fb_w,
            fb_h,
            frame: 0,
            lfsr,
        }
    }

    /// Advance all columns one tick
    fn tick(&mut self) {
        self.frame += 1;

        for col in self.columns.iter_mut() {
            if !col.active {
                // Randomly reactivate
                if self.frame % 7 == 0 {
                    self.lfsr = lfsr_next(self.lfsr);
                    if self.lfsr % 5 == 0 {
                        col.active = true;
                        col.head_y = 0;
                        self.lfsr = lfsr_next(self.lfsr);
                        col.speed = (self.lfsr % 4) as u8 + 1;
                        self.lfsr = lfsr_next(self.lfsr);
                        col.trail_len = (self.lfsr % 12) as u8 + 4;
                        self.lfsr = lfsr_next(self.lfsr);
                        col.char_offset = (self.lfsr % 94) as u8;
                    }
                }
                continue;
            }

            col.head_y += col.speed as i32;

            // Deactivate when fully off screen
            if col.head_y > (self.num_rows as i32 + col.trail_len as i32 + 4) {
                col.active = false;
            }

            // Decay flash
            if col.flash > 100 {
                col.flash = col.flash.saturating_sub(15);
                if col.flash < 100 { col.flash = 100; }
            }
        }
    }

    /// Flash columns near a "beat hit"
    fn flash_beat(&mut self, intensity: u8) {
        // Flash random subset of columns
        let count = (self.num_cols * intensity as usize / 255).max(3);
        for _ in 0..count {
            self.lfsr = lfsr_next(self.lfsr);
            let col_idx = (self.lfsr as usize) % self.num_cols;
            self.columns[col_idx].flash = 255;
            self.columns[col_idx].active = true;
            self.columns[col_idx].head_y = 0;
            self.lfsr = lfsr_next(self.lfsr);
            self.columns[col_idx].speed = (self.lfsr % 3) as u8 + 3; // Fast!
        }
    }

    /// Draw the Matrix rain
    fn draw(&self, step: usize, total_steps: usize, track_info: &str, bpm: u16, bar_beat: &str) {
        // Black background
        crate::framebuffer::fill_rect(0, 0, self.fb_w, self.fb_h, 0x000000);

        // Matrix glyph set — ASCII printable range simulating katakana
        const GLYPHS: &[u8] = b"@#$%&*0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz!?<>{}[]|/\\~^";

        // Draw each column
        for (col_idx, col) in self.columns.iter().enumerate() {
            if !col.active { continue; }

            let x = col_idx as u32 * 8;

            for row_offset in 0..(col.trail_len as i32 + 1) {
                let row = col.head_y - row_offset;
                if row < 0 || row >= self.num_rows as i32 { continue; }

                let y = row as u32 * 16;

                // Choose character — changes per frame for head, stable for trail
                let char_idx = if row_offset == 0 {
                    // Head: changes rapidly
                    ((col.char_offset as u32 + self.frame * 3 + col_idx as u32) % GLYPHS.len() as u32) as usize
                } else {
                    // Trail: stable per position
                    ((col.char_offset as u32 + row as u32 * 7 + col_idx as u32 * 13) % GLYPHS.len() as u32) as usize
                };
                let ch = GLYPHS[char_idx] as char;

                // Color gradient: head is bright white-green, trail fades to dark green
                let brightness = if row_offset == 0 {
                    // Head: bright white/green
                    255u32
                } else {
                    // Fade along trail
                    let fade = 255u32.saturating_sub(row_offset as u32 * 255 / col.trail_len as u32);
                    fade.max(20)
                };

                // Apply flash multiplier
                let flash_mult = col.flash as u32;
                let effective_b = (brightness * flash_mult / 100).min(255);

                // Green Matrix color with slight blue tint
                let r = if row_offset == 0 { effective_b * 80 / 100 } else { effective_b * 10 / 100 };
                let g = effective_b;
                let b = if row_offset == 0 { effective_b * 60 / 100 } else { effective_b * 20 / 100 };
                let color = ((r.min(255)) << 16) | ((g.min(255)) << 8) | b.min(255);

                crate::framebuffer::draw_char_at(x, y, ch, color);
            }
        }

        // ── Overlay: Step Progress Bar at bottom ──
        let bar_y = self.fb_h - 32;
        let bar_h = 8;
        let bar_w = self.fb_w - 40;
        let bar_x = 20;

        // Dark bar background
        crate::framebuffer::fill_rect(bar_x, bar_y, bar_w, bar_h, 0x002200);
        // Border
        crate::framebuffer::draw_rect(bar_x, bar_y, bar_w, bar_h, 0x00AA00);

        // Fill based on step position
        if total_steps > 0 {
            let filled = bar_w * step as u32 / total_steps as u32;
            crate::framebuffer::fill_rect(bar_x + 1, bar_y + 1, filled, bar_h - 2, 0x00FF44);

            // Step markers (beat divisions)
            for i in 1..total_steps {
                if i % 4 == 0 {
                    let mx = bar_x + bar_w * i as u32 / total_steps as u32;
                    crate::framebuffer::draw_vline(mx, bar_y, bar_h, 0x00CC00);
                }
            }
        }

        // ── Overlay: Title text (Matrix-green on black box) ──
        let title = "TRUSTDAW // BEAT MATRIX";
        let title_w = title.len() as u32 * 8 + 16;
        let title_x = (self.fb_w - title_w) / 2;
        crate::framebuffer::fill_rect(title_x, 8, title_w, 24, 0x001100);
        crate::framebuffer::draw_rect(title_x, 8, title_w, 24, 0x00CC00);
        crate::framebuffer::draw_text(title, title_x + 8, 12, 0x00FF66);

        // ── Overlay: Track info ──
        let info_y = 40;
        let info_w = track_info.len() as u32 * 8 + 16;
        crate::framebuffer::fill_rect(8, info_y, info_w.min(self.fb_w - 16), 20, 0x000800);
        crate::framebuffer::draw_text(track_info, 16, info_y + 2, 0x00AA44);

        // ── Overlay: BPM & position ──
        let bpm_str = format!("{} BPM  {}", bpm, bar_beat);
        let bpm_w = bpm_str.len() as u32 * 8 + 16;
        let bpm_x = self.fb_w - bpm_w - 8;
        crate::framebuffer::fill_rect(bpm_x, info_y, bpm_w, 20, 0x000800);
        crate::framebuffer::draw_text(&bpm_str, bpm_x + 8, info_y + 2, 0x00CC66);

        // ── Overlay: Step counter (big text) ──
        let step_str = format!("{:02}/{:02}", step + 1, total_steps);
        let step_w = step_str.len() as u32 * 8 + 12;
        let step_x = (self.fb_w - step_w) / 2;
        let step_y = bar_y - 24;
        crate::framebuffer::fill_rect(step_x, step_y, step_w, 20, 0x001100);
        crate::framebuffer::draw_text(&step_str, step_x + 6, step_y + 2, 0x44FF88);

        // ── Overlay: Active track indicators (left side) ──
        let ind_y = 70;
        let track_names = ["Ki", "Cl", "HH", "Ba", "St", "Ch", "Ld", "Pc"];
        for (i, name) in track_names.iter().enumerate() {
            let ty = ind_y + i as u32 * 20;
            let color = if i < 8 { colors::TRACK_COLORS[i] } else { 0x00FF00 };
            crate::framebuffer::draw_text(name, 8, ty, color);
        }
    }
}

/// Launch the Matrix visualizer with the funky house beat
pub fn launch_matrix() -> Result<(), &'static str> {
    crate::audio::init().ok();

    let mut studio = BeatStudio::new();
    studio.load_funky_house();

    let mut matrix = MatrixState::new();

    // Initial draw
    matrix.draw(0, studio.tracks[0].num_steps, "> INITIALIZING BEAT MATRIX...", studio.bpm, "1:1.1");

    // Render the audio
    let audio = studio.render_loop();
    studio.update_scope(&audio);

    let total_steps = studio.tracks[0].num_steps;
    let step_ms = studio.step_duration_ms();
    let total_dur_ms = step_ms * total_steps as u32;

    crate::serial_println!("[MATRIX] Funky House: {} BPM, {} steps, {}ms per step", studio.bpm, total_steps, step_ms);

    // Brief intro animation (Matrix rain settling)
    for f in 0..30 {
        matrix.tick();
        let intro_msg = match f {
            0..=5   => "> LOADING BEAT DATA...",
            6..=12  => "> DECODING FREQUENCY MATRIX...",
            13..=20 => "> SYNTH ENGINES ONLINE...",
            _       => "> READY. ENTERING THE BEAT.",
        };
        matrix.draw(0, total_steps, intro_msg, studio.bpm, "---");

        if wait_ms_interruptible(100) { return Ok(()); } // 100ms per frame
    }

    // MAIN LOOP — play + animate with looping
    let max_loops = 4u32;

    // Start non-blocking looped playback
    let _ = crate::drivers::hda::start_looped_playback(&audio);

    'outer: for _loop_count in 0..max_loops {

        // Animate through each step
        for s in 0..total_steps {
            studio.current_step = s;

            // Check which tracks are active at this step → flash
            for t in 0..8 {
                if studio.tracks[t].steps[s].active && !studio.tracks[t].muted {
                    let vel = studio.tracks[t].steps[s].velocity;
                    matrix.flash_beat(vel);
                }
            }

            // Build track activity string
            let mut active_str = String::from("> ");
            for t in 0..8 {
                if studio.tracks[t].steps[s].active && !studio.tracks[t].muted {
                    active_str.push_str(studio.tracks[t].name_str());
                    active_str.push(' ');
                }
            }
            if active_str.len() <= 2 {
                active_str.push_str("...");
            }

            // Position string
            let bar = s / 16 + 1;
            let beat = (s % 16) / 4 + 1;
            let sub = s % 4 + 1;
            let pos_str = format!("{}:{}.{}", bar, beat, sub);

            // Tick the matrix rain and draw
            matrix.tick();
            matrix.draw(s, total_steps, &active_str, studio.bpm, &pos_str);

            // Wait exactly one step duration (PIT-timed)
            match wait_ms_skip(step_ms as u64) {
                1 | 2 => { break 'outer; } // Esc or Space
                _ => {}
            }
        }
    }

    // Outro animation
    let _ = crate::audio::stop();

    for f in 0..40 {
        matrix.tick();
        let outro_msg = match f {
            0..=10  => "> DISCONNECTING...",
            11..=25 => "> SIGNAL LOST",
            _       => "> TRUSTDAW // SYSTEM OFFLINE",
        };
        matrix.draw(0, total_steps, outro_msg, studio.bpm, "---");

        // Gradually darken: deactivate columns
        let deactivate = matrix.num_cols / 40;
        for c in 0..deactivate {
            let idx = (f as usize * deactivate + c) % matrix.num_cols;
            matrix.columns[idx].active = false;
        }

        crate::cpu::tsc::delay_millis(80); // 80ms per frame
    }

    // Final black screen with message
    crate::framebuffer::fill_rect(0, 0, matrix.fb_w, matrix.fb_h, 0x000000);
    let final_msg = "TRUSTDAW BEAT MATRIX // BUILT ON TRUSTOS";
    let fw = final_msg.len() as u32 * 8;
    let fx = (matrix.fb_w - fw) / 2;
    let fy = matrix.fb_h / 2 - 8;
    crate::framebuffer::draw_text(final_msg, fx, fy, 0x00FF44);

    let sub_msg = "Bare-metal. No OS. Pure Rust.";
    let sw = sub_msg.len() as u32 * 8;
    let sx = (matrix.fb_w - sw) / 2;
    crate::framebuffer::draw_text(sub_msg, sx, fy + 24, 0x008822);

    // Wait for key
    loop {
        if let Some(sc) = crate::keyboard::try_read_key() {
            if sc & 0x80 == 0 { break; }
        }
        crate::cpu::tsc::delay_millis(20);
    }

    crate::serial_println!("[MATRIX] Showcase complete");
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════════════
// NARRATED SHOWCASE — Automated beat creation + playback for YouTube
// ═══════════════════════════════════════════════════════════════════════════════
//
// A fully automated, cinematic demo in 3 phases:
//
// PHASE 1 — "Building the Beat"
//   Shows the Beat Studio UI. Adds tracks one by one with narration text.
//   Each track is solo'd and played so the viewer hears each layer.
//
// PHASE 2 — "The Full Mix"
//   Unmutes all tracks, plays the complete beat in the studio UI.
//   Shows the mixer moving, scope, spectrum — all alive.
//
// PHASE 3 — "Enter the Matrix"
//   Transitions to the Matrix rain visualizer with the full beat.
//   Cinematic outro.

/// Narration card: text lines + display duration (in ~frames)
struct NarrationCard {
    title: &'static str,
    subtitle: &'static str,
    detail: &'static str,
    frames: u32,
}

/// Draw a cinematic narration overlay on top of the current screen
fn draw_narration_overlay(card: &NarrationCard, fb_w: u32, fb_h: u32, phase: &str, progress: u32, total: u32) {
    // Semi-transparent dark box at bottom third of screen
    let box_h = 120u32;
    let box_y = fb_h.saturating_sub(box_h + 52); // above status bar
    let box_x = 16u32;
    let box_w = fb_w.saturating_sub(32);

    crate::framebuffer::fill_rect_alpha(box_x, box_y, box_w, box_h, 0x000000, 200);

    // Border (cyan accent)
    crate::framebuffer::draw_rect(box_x, box_y, box_w, box_h, 0x00CCFF);

    // Phase indicator (top-left of box)
    crate::framebuffer::draw_text(phase, box_x + 12, box_y + 8, 0x00AACC);

    // Title (large — bright white)
    crate::framebuffer::draw_text(card.title, box_x + 12, box_y + 28, 0xFFFFFF);

    // Subtitle (accent green)
    crate::framebuffer::draw_text(card.subtitle, box_x + 12, box_y + 50, 0x44FF88);

    // Detail (dim)
    crate::framebuffer::draw_text(card.detail, box_x + 12, box_y + 72, 0x88AACC);

    // Progress bar at bottom of box
    let pb_x = box_x + 12;
    let pb_y = box_y + box_h - 16;
    let pb_w = box_w - 24;
    let pb_h = 6u32;
    crate::framebuffer::fill_rect(pb_x, pb_y, pb_w, pb_h, 0x112233);
    if total > 0 {
        let filled = pb_w * progress / total;
        crate::framebuffer::fill_rect(pb_x, pb_y, filled, pb_h, 0x00CCFF);
    }
}

/// Draw a full-screen cinematic title card (black bg)
fn draw_title_card(fb_w: u32, fb_h: u32, line1: &str, line2: &str, line3: &str, accent: u32) {
    crate::framebuffer::fill_rect(0, 0, fb_w, fb_h, 0x050510);

    // Accent line across screen
    let mid_y = fb_h / 2;
    crate::framebuffer::fill_rect(0, mid_y - 60, fb_w, 1, accent);
    crate::framebuffer::fill_rect(0, mid_y + 60, fb_w, 1, accent);

    // Line 1 — centered, bright
    let w1 = line1.len() as u32 * 8;
    crate::framebuffer::draw_text(line1, (fb_w - w1) / 2, mid_y - 40, 0xFFFFFF);

    // Line 2 — centered, accent
    let w2 = line2.len() as u32 * 8;
    crate::framebuffer::draw_text(line2, (fb_w - w2) / 2, mid_y - 8, accent);

    // Line 3 — centered, dim
    let w3 = line3.len() as u32 * 8;
    crate::framebuffer::draw_text(line3, (fb_w - w3) / 2, mid_y + 24, 0x667788);
}

/// Wait for a given number of milliseconds, checking keyboard for Esc.
/// Returns true if user pressed Esc (abort).
fn wait_ms_interruptible(total_ms: u64) -> bool {
    // Break into small chunks so we can check keyboard
    let chunk = 50u64; // Check keyboard every 50ms
    let mut remaining = total_ms;
    while remaining > 0 {
        let delay = remaining.min(chunk);
        crate::cpu::tsc::delay_millis(delay);
        remaining -= delay;
        // Drain keyboard
        while let Some(sc) = crate::keyboard::try_read_key() {
            if sc & 0x80 != 0 { continue; }
            if sc == 0x01 { return true; } // Esc
        }
    }
    false
}

/// Wait for a given number of milliseconds, checking for Esc OR Space.
/// Returns 0=ok, 1=esc, 2=space.
fn wait_ms_skip(total_ms: u64) -> u8 {
    let chunk = 50u64;
    let mut remaining = total_ms;
    while remaining > 0 {
        let delay = remaining.min(chunk);
        crate::cpu::tsc::delay_millis(delay);
        remaining -= delay;
        while let Some(sc) = crate::keyboard::try_read_key() {
            if sc & 0x80 != 0 { continue; }
            if sc == 0x01 { return 1; } // Esc
            if sc == 0x39 { return 2; } // Space
        }
    }
    0
}

/// The main narrated showcase entry point
pub fn launch_narrated_showcase() -> Result<(), &'static str> {
    crate::audio::init().ok();
    crate::serial_println!("[SHOWCASE] Starting narrated showcase...");

    let fb_w = crate::framebuffer::FB_WIDTH.load(Ordering::Relaxed) as u32;
    let fb_h = crate::framebuffer::FB_HEIGHT.load(Ordering::Relaxed) as u32;

    // Enable double buffering to eliminate flicker
    crate::framebuffer::init_double_buffer();
    crate::framebuffer::set_double_buffer_mode(true);

    // ═══════════════════════════════════════════════════════════════════
    // INTRO TITLE CARD
    // ═══════════════════════════════════════════════════════════════════

    draw_title_card(fb_w, fb_h,
        "T R U S T D A W",
        "Building a Funky House Track from Scratch",
        "Bare-Metal  //  No OS  //  Pure Rust  //  Real-Time Audio",
        0x00CCFF,
    );
    crate::framebuffer::swap_buffers();
    if wait_ms_interruptible(5000) { showcase_cleanup(); return Ok(()); }

    draw_title_card(fb_w, fb_h,
        "PHASE 1: BUILDING THE BEAT",
        "Watch each layer come to life, one track at a time",
        "124 BPM  //  C Minor  //  32 Steps (2 Bars)",
        0x44FF88,
    );
    crate::framebuffer::swap_buffers();
    if wait_ms_interruptible(4000) { showcase_cleanup(); return Ok(()); }

    // ═══════════════════════════════════════════════════════════════════
    // PHASE 1 — Building the beat track-by-track (tutorial style)
    // ═══════════════════════════════════════════════════════════════════
    crate::serial_println!("[SHOWCASE] Phase 1: Building the beat");

    // Load the full song data into a "reference" copy
    let mut reference = BeatStudio::new();
    reference.load_funky_house();

    // Create the "live" studio that starts empty — we'll build it up
    let mut studio = BeatStudio::new();
    studio.bpm = 124;
    studio.swing = 58;
    // Configure tracks (names, instruments, etc.) but keep all steps empty
    for t in studio.tracks.iter_mut() {
        t.num_steps = 32;
        for s in 0..MAX_STEPS {
            t.steps[s] = BeatStep::off();
        }
    }
    studio.tracks[0] = BeatTrack::new("Kick",  36, Waveform::Sine,     colors::TRACK_COLORS[0], true);
    studio.tracks[1] = BeatTrack::new("Clap",  39, Waveform::Noise,    colors::TRACK_COLORS[1], true);
    studio.tracks[2] = BeatTrack::new("HiHat", 42, Waveform::Noise,    colors::TRACK_COLORS[2], true);
    studio.tracks[3] = BeatTrack::new("Bass",  36, Waveform::Square,   colors::TRACK_COLORS[3], false);
    studio.tracks[4] = BeatTrack::new("Stab",  60, Waveform::Sawtooth, colors::TRACK_COLORS[4], false);
    studio.tracks[5] = BeatTrack::new("Chord", 60, Waveform::Triangle, colors::TRACK_COLORS[5], false);
    studio.tracks[6] = BeatTrack::new("Lead",  72, Waveform::Sawtooth, colors::TRACK_COLORS[6], false);
    studio.tracks[7] = BeatTrack::new("Perc",  56, Waveform::Noise,    colors::TRACK_COLORS[7], true);
    for t in studio.tracks.iter_mut() {
        t.num_steps = 32;
    }
    // Copy envelopes and volumes from reference
    for i in 0..8 {
        studio.tracks[i].envelope = reference.tracks[i].envelope;
        studio.tracks[i].volume = reference.tracks[i].volume;
        studio.tracks[i].muted = false;
    }

    let step_ms = studio.step_duration_ms();
    let total_steps = 32usize;
    let loop_dur_ms = step_ms * total_steps as u32;

    // Define narration for each track
    let track_cards: [NarrationCard; 8] = [
        NarrationCard {
            title: "KICK -- The Foundation",
            subtitle: "Placing four-on-the-floor kicks + ghost notes",
            detail: "Sine wave @ 36Hz  |  Punchy thump (1ms atk, 120ms decay)",
            frames: 0,
        },
        NarrationCard {
            title: "CLAP -- The Backbeat",
            subtitle: "Adding claps on beats 2 & 4 with ghost flams",
            detail: "Noise burst @ 39Hz  |  Tight snap (1ms atk, 60ms decay)",
            frames: 0,
        },
        NarrationCard {
            title: "HI-HAT -- The Groove Engine",
            subtitle: "Programming 16th notes with velocity dynamics",
            detail: "Noise @ 42Hz  |  Crispy short (1ms atk, 15ms decay)  |  Accented off-beats",
            frames: 0,
        },
        NarrationCard {
            title: "BASS -- The Low End",
            subtitle: "Drawing syncopated C minor bassline: C-Eb-G-F-Bb-Ab",
            detail: "Square wave  |  Funky pluck envelope  |  2-bar variation",
            frames: 0,
        },
        NarrationCard {
            title: "STAB -- House Chord Hits",
            subtitle: "Placing off-beat Cm7 stabs cutting through the mix",
            detail: "Sawtooth wave  |  Quick punch (2ms atk, 50ms decay)",
            frames: 0,
        },
        NarrationCard {
            title: "CHORD -- The Harmonic Bed",
            subtitle: "Adding sustained pads: Cm7 -> Fm7 chord progression",
            detail: "Triangle wave  |  Lush pad envelope  |  Chord changes at bar 2",
            frames: 0,
        },
        NarrationCard {
            title: "LEAD -- The Melody",
            subtitle: "Drawing a disco-house riff: ascending C-Eb-G, call & response",
            detail: "Sawtooth @ C5  |  Singing envelope  |  Peak at C6 in bar 2",
            frames: 0,
        },
        NarrationCard {
            title: "PERCUSSION -- The Texture",
            subtitle: "Laying down shakers on off-beats + fill at end of bar 2",
            detail: "Noise burst  |  Snap envelope  |  Building energy into the drop",
            frames: 0,
        },
    ];

    // ── For each track: show title, animate step placement, then play ──
    for track_idx in 0..8 {
        studio.cursor_track = track_idx;

        // ── Title card for this track ──
        let card = &track_cards[track_idx];
        draw_title_card(fb_w, fb_h,
            &format!("TRACK {}/8", track_idx + 1),
            card.title,
            card.detail,
            colors::TRACK_COLORS[track_idx],
        );
        crate::framebuffer::swap_buffers();
        if wait_ms_interruptible(3500) { showcase_cleanup(); return Ok(()); }

        // ── Collect which steps need to be placed ──
        let mut steps_to_place: Vec<usize> = Vec::new();
        for s in 0..total_steps {
            if reference.tracks[track_idx].steps[s].active {
                steps_to_place.push(s);
            }
        }

        // ── Show "placing steps" narration ──
        let phase_str = format!("PHASE 1  //  TRACK {}/8  //  PLACING PATTERN", track_idx + 1);

        // Draw initial state (empty track visible)
        studio.draw();
        draw_narration_overlay(card, fb_w, fb_h, &phase_str, 0, steps_to_place.len() as u32);
        crate::framebuffer::swap_buffers();
        crate::cpu::tsc::delay_millis(800);

        // ── Animate placing each step one by one ──
        for (place_idx, &step_pos) in steps_to_place.iter().enumerate() {
            // Move cursor to this step
            studio.cursor_step = step_pos;

            // Flash: draw with cursor on empty step (shows cursor moving)
            studio.draw();
            let progress = place_idx as u32;
            let total = steps_to_place.len() as u32;
            draw_narration_overlay(card, fb_w, fb_h, &phase_str, progress, total);
            crate::framebuffer::swap_buffers();

            // Brief pause to see cursor moving
            crate::cpu::tsc::delay_millis(150);

            // Place the step (copy from reference)
            studio.tracks[track_idx].steps[step_pos] = reference.tracks[track_idx].steps[step_pos];

            // Redraw with the step now active (lit up)
            studio.draw();
            draw_narration_overlay(card, fb_w, fb_h, &phase_str, progress + 1, total);
            crate::framebuffer::swap_buffers();

            // Pause to see the step light up
            crate::cpu::tsc::delay_millis(200);

            // Check for Esc
            while let Some(sc) = crate::keyboard::try_read_key() {
                if sc & 0x80 != 0 { continue; }
                if sc == 0x01 { showcase_cleanup(); return Ok(()); }
                if sc == 0x39 { break; } // Space = skip placement anim
            }
        }

        // ── Pattern placed! Now play the current mix ──
        let listen_str = format!("PHASE 1  //  TRACK {}/8  //  LISTEN", track_idx + 1);

        let listen_card = NarrationCard {
            title: card.title,
            subtitle: if track_idx == 0 {
                "Listening to the kick pattern..."
            } else {
                "Hear how this layer adds to the mix..."
            },
            detail: card.detail,
            frames: 0,
        };

        // Render audio with all tracks placed so far
        let audio = studio.render_loop();
        studio.update_scope(&audio);

        // Start non-blocking looped playback
        let _ = crate::drivers::hda::start_looped_playback(&audio);
        studio.playing = true;

        // Animate playhead through 2 full loops while audio plays
        let mut escaped = false;
        for _loop_num in 0..2u32 {
            for s in 0..total_steps {
                studio.current_step = s;
                studio.update_spectrum();
                studio.draw();
                let progress = (s as u32 * 100) / total_steps as u32;
                draw_narration_overlay(&listen_card, fb_w, fb_h, &listen_str, progress, 100);
                crate::framebuffer::swap_buffers();

                match wait_ms_skip(step_ms as u64) {
                    1 => { escaped = true; break; }
                    2 => { break; } // Space = skip to next track
                    _ => {}
                }
            }
            if escaped { break; }
        }

        // Stop audio, reset
        let _ = crate::drivers::hda::stop();
        studio.playing = false;
        studio.current_step = 0;

        if escaped { showcase_cleanup(); return Ok(()); }

        // Brief pause between tracks
        crate::cpu::tsc::delay_millis(800);
    }

    // ═══════════════════════════════════════════════════════════════════
    // PHASE 2 — Full Mix Playback
    // ═══════════════════════════════════════════════════════════════════
    crate::serial_println!("[SHOWCASE] Phase 2: Full mix playback");

    draw_title_card(fb_w, fb_h,
        "PHASE 2: THE FULL MIX",
        "All 8 tracks together -- the complete Funky House groove",
        "Listen to how the layers combine into a unified track",
        0xFF6622,
    );
    crate::framebuffer::swap_buffers();
    if wait_ms_interruptible(4000) { showcase_cleanup(); return Ok(()); }

    // Render the full mix
    let full_audio = studio.render_loop();
    studio.update_scope(&full_audio);

    let mix_card = NarrationCard {
        title: "FULL MIX -- All 8 Tracks",
        subtitle: "Kick + Clap + HiHat + Bass + Stab + Chord + Lead + Perc",
        detail: "124 BPM  |  C Minor  |  Funky House  |  Bare-Metal Audio Engine",
        frames: 0,
    };

    // Start non-blocking looped playback for 3 loops
    let _ = crate::drivers::hda::start_looped_playback(&full_audio);
    studio.playing = true;

    let mut escaped = false;
    for loop_num in 0..3u32 {
        for s in 0..total_steps {
            studio.current_step = s;
            studio.update_spectrum();
            studio.draw();
            let loop_label = format!("PHASE 2  //  LOOP {}/3", loop_num + 1);
            let progress = (s as u32 * 100) / total_steps as u32;
            draw_narration_overlay(&mix_card, fb_w, fb_h, &loop_label, progress, 100);
            crate::framebuffer::swap_buffers();

            match wait_ms_skip(step_ms as u64) {
                1 => { escaped = true; break; }
                2 => { break; }
                _ => {}
            }
        }
        if escaped { break; }
    }

    let _ = crate::drivers::hda::stop();
    studio.playing = false;
    studio.current_step = 0;
    if escaped { showcase_cleanup(); return Ok(()); }

    // ═══════════════════════════════════════════════════════════════════
    // PHASE 3 — Matrix Visualizer
    // ═══════════════════════════════════════════════════════════════════
    crate::serial_println!("[SHOWCASE] Phase 3: Matrix visualizer");

    draw_title_card(fb_w, fb_h,
        "PHASE 3: ENTER THE MATRIX",
        "The same beat, visualized as a living data stream",
        "Matrix rain  //  Beat-reactive  //  Pure framebuffer rendering",
        0x00FF44,
    );
    crate::framebuffer::swap_buffers();
    if wait_ms_interruptible(4000) { showcase_cleanup(); return Ok(()); }

    let mut matrix = MatrixState::new();

    // Intro animation
    for f in 0..25 {
        matrix.tick();
        let intro_msg = match f {
            0..=6   => "> LOADING BEAT DATA...",
            7..=14  => "> DECODING FREQUENCY MATRIX...",
            15..=20 => "> SYNTH ENGINES ONLINE...",
            _       => "> ENTERING THE BEAT MATRIX...",
        };
        matrix.draw(0, total_steps, intro_msg, studio.bpm, "---");
        crate::framebuffer::swap_buffers();
        if wait_ms_interruptible(150) { showcase_cleanup(); return Ok(()); }
    }

    // Start non-blocking looped playback for matrix
    let _ = crate::drivers::hda::start_looped_playback(&full_audio);

    let matrix_loops = 3u32;
    escaped = false;
    for loop_num in 0..matrix_loops {
        for s in 0..total_steps {
            studio.current_step = s;

            // Flash on active tracks
            for t in 0..8 {
                if studio.tracks[t].steps[s].active && !studio.tracks[t].muted {
                    matrix.flash_beat(studio.tracks[t].steps[s].velocity);
                }
            }

            // Track activity text
            let mut active_str = format!("LOOP {}/{}  > ", loop_num + 1, matrix_loops);
            for t in 0..8 {
                if studio.tracks[t].steps[s].active && !studio.tracks[t].muted {
                    active_str.push_str(studio.tracks[t].name_str());
                    active_str.push(' ');
                }
            }
            if active_str.ends_with("> ") { active_str.push_str("..."); }

            let bar = s / 16 + 1;
            let beat = (s % 16) / 4 + 1;
            let sub = s % 4 + 1;
            let pos_str = format!("{}:{}.{}", bar, beat, sub);

            matrix.tick();
            matrix.draw(s, total_steps, &active_str, studio.bpm, &pos_str);
            crate::framebuffer::swap_buffers();

            match wait_ms_skip(step_ms as u64) {
                1 => { escaped = true; break; }
                2 => { break; }
                _ => {}
            }
        }
        if escaped { break; }
    }

    let _ = crate::drivers::hda::stop();

    // ═══════════════════════════════════════════════════════════════════
    // OUTRO
    // ═══════════════════════════════════════════════════════════════════
    crate::serial_println!("[SHOWCASE] Outro");

    // Fade out matrix
    for f in 0..35 {
        matrix.tick();
        let outro_msg = match f {
            0..=8   => "> SIGNAL FADING...",
            9..=20  => "> DISCONNECTING FROM THE MATRIX...",
            _       => "> TRUSTDAW // SYSTEM OFFLINE",
        };
        matrix.draw(0, total_steps, outro_msg, studio.bpm, "---");
        crate::framebuffer::swap_buffers();

        // Deactivate columns gradually
        let to_kill = matrix.num_cols / 30;
        for c in 0..to_kill {
            let idx = (f as usize * to_kill + c) % matrix.num_cols;
            matrix.columns[idx].active = false;
        }

        crate::cpu::tsc::delay_millis(100); // 100ms per frame
    }

    // Final credits screen
    crate::framebuffer::fill_rect(0, 0, fb_w, fb_h, 0x020208);

    let mid = fb_h / 2;

    // Decorative lines
    crate::framebuffer::fill_rect(fb_w / 4, mid - 80, fb_w / 2, 1, 0x00CCFF);
    crate::framebuffer::fill_rect(fb_w / 4, mid + 80, fb_w / 2, 1, 0x00CCFF);

    let credits: [(&str, u32); 8] = [
        ("T R U S T D A W",                          0x00FF66),
        ("",                                          0x000000),
        ("A bare-metal beat production studio",       0xCCCCDD),
        ("running on TrustOS -- written in Rust",     0xCCCCDD),
        ("",                                          0x000000),
        ("No operating system. No libraries.",         0x88AACC),
        ("Just raw hardware, a framebuffer, and HDA audio.", 0x88AACC),
        ("",                                          0x000000),
    ];

    let start_y = mid - 60;
    for (i, (text, color)) in credits.iter().enumerate() {
        if text.is_empty() { continue; }
        let tw = text.len() as u32 * 8;
        let tx = (fb_w - tw) / 2;
        crate::framebuffer::draw_text(text, tx, start_y + i as u32 * 20, *color);
    }

    // Bottom tagline
    let tag = "Press any key to exit";
    let tw = tag.len() as u32 * 8;
    crate::framebuffer::draw_text(tag, (fb_w - tw) / 2, mid + 60, 0x556677);
    crate::framebuffer::swap_buffers();

    // Wait for any key
    loop {
        if let Some(sc) = crate::keyboard::try_read_key() {
            if sc & 0x80 == 0 { break; }
        }
        crate::cpu::tsc::delay_millis(20);
    }

    showcase_cleanup();
    crate::serial_println!("[SHOWCASE] Narrated showcase complete");
    Ok(())
}

/// Restore framebuffer state after showcase exits
fn showcase_cleanup() {
    crate::framebuffer::set_double_buffer_mode(false);
}

/// Simple LFSR pseudo-random number generator (xorshift32)
fn lfsr_next(state: u32) -> u32 {
    let mut s = state;
    if s == 0 { s = 0xDEAD_BEEF; }
    s ^= s << 13;
    s ^= s >> 17;
    s ^= s << 5;
    s
}
