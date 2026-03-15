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
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const BG_DARK: u32        = 0x0A0A14;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const BG_MAIN: u32        = 0x0F0F1E;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const BG_PANEL: u32       = 0x141428;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const BG_HEADER: u32      = 0x1A1A30;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const BG_ELEVATED: u32    = 0x1E1E38;

    // Borders & Lines
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const BORDER: u32         = 0x2A2A4A;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const BORDER_BRIGHT: u32  = 0x4A4A6A;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const DIVIDER: u32        = 0x222240;

    // Text
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const TEXT_BRIGHT: u32    = 0xEEEEFF;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const TEXT_PRIMARY: u32   = 0xCCCCDD;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const TEXT_SECONDARY: u32 = 0x8888AA;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const TEXT_DIM: u32       = 0x555577;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const TEXT_ACCENT: u32    = 0x66BBFF;

    // Transport
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const PLAY_GREEN: u32     = 0x44DD66;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const STOP_GRAY: u32      = 0x666688;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const REC_RED: u32        = 0xFF3344;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const TRANSPORT_BG: u32   = 0x121228;

    // Step Sequencer
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const STEP_OFF: u32       = 0x1A1A30;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const STEP_ON: u32        = 0xFF6622;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const STEP_ON_ALT: u32    = 0xFF8844;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const STEP_CURSOR: u32    = 0x44FF88;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const STEP_PLAYHEAD: u32  = 0x66FF66;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const STEP_BORDER: u32    = 0x333355;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const STEP_BEAT_DIV: u32  = 0x444466;

    // Mixer
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const METER_GREEN: u32    = 0x44CC44;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const METER_YELLOW: u32   = 0xCCCC44;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const METER_RED: u32      = 0xCC4444;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const METER_BG: u32       = 0x0D0D1A;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const FADER_KNOB: u32     = 0xBBBBCC;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const MUTE_ORANGE: u32    = 0xFF8800;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SOLO_YELLOW: u32    = 0xFFDD00;

    // Keyboard
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const KEY_WHITE: u32      = 0xDDDDEE;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const KEY_BLACK: u32      = 0x222233;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const KEY_PRESSED: u32    = 0xFF6622;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const KEY_LABEL: u32      = 0x444455;

    // Scope & Spectrum
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SCOPE_LINE: u32     = 0x44DDFF;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SCOPE_BG: u32       = 0x0A0A18;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SPECTRUM_1: u32     = 0x22CC66;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SPECTRUM_2: u32     = 0x66DD44;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SPECTRUM_3: u32     = 0xCCCC22;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SPECTRUM_4: u32     = 0xDD6622;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SPECTRUM_5: u32     = 0xDD2222;

    // Track colors (one per track row)
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const TRACK_COLORS: [u32; 8] = [
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
const MAXIMUM_BEAT_TRACKS: usize = 8;
/// Maximum steps per track
const MAXIMUM_STEPS: usize = 32;
/// Default number of steps (16 = one bar of 16th notes)
const DEFAULT_STEPS: usize = 16;

/// One step in the beat grid
#[derive(Clone, Copy)]
// Structure publique — visible à l'extérieur de ce module.
pub struct BeatStep {
    /// Step is active (will sound)
    pub active: bool,
    /// Velocity (1-127, 0 = use track default)
    pub velocity: u8,
    /// Note offset from track base note (for melodic tracks)
    pub note_offset: i8,
}

// Bloc d'implémentation — définit les méthodes du type ci-dessus.
impl BeatStep {
        // Fonction publique — appelable depuis d'autres modules.
pub fn off() -> Self {
        Self { active: false, velocity: 100, note_offset: 0 }
    }

        // Fonction publique — appelable depuis d'autres modules.
pub fn on(velocity: u8) -> Self {
        Self { active: true, velocity, note_offset: 0 }
    }

        // Fonction publique — appelable depuis d'autres modules.
pub fn on_note(velocity: u8, offset: i8) -> Self {
        Self { active: true, velocity, note_offset: offset }
    }
}

/// One track in the beat studio (drum or melodic)
pub struct BeatTrack {
    /// Track name
    pub name: [u8; 16],
    pub name_length: usize,
    /// Step data
    pub steps: [BeatStep; MAXIMUM_STEPS],
    /// Number of active steps (16 or 32)
    pub number_steps: usize,
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

// Bloc d'implémentation — définit les méthodes du type ci-dessus.
impl BeatTrack {
        // Fonction publique — appelable depuis d'autres modules.
pub fn new(name: &str, base_note: u8, waveform: Waveform, color: u32, is_drum: bool) -> Self {
        let mut name_buffer = [0u8; 16];
        let bytes = name.as_bytes();
        let len = bytes.len().minimum(16);
        name_buffer[..len].copy_from_slice(&bytes[..len]);

        Self {
            name: name_buffer,
            name_length: len,
            steps: [BeatStep::off(); MAXIMUM_STEPS],
            number_steps: DEFAULT_STEPS,
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

        // Fonction publique — appelable depuis d'autres modules.
pub fn name_str(&self) -> &str {
        core::str::from_utf8(&self.name[..self.name_length]).unwrap_or("???")
    }

    /// Toggle a step on/off
    pub fn toggle_step(&mut self, step: usize) {
        if step < self.number_steps {
            self.steps[step].active = !self.steps[step].active;
        }
    }

    /// Get the MIDI note for a given step
    pub fn note_at(&self, step: usize) -> u8 {
        if step < self.number_steps && self.steps[step].active {
            let base = self.base_note as i16;
            let offset = self.steps[step].note_offset as i16;
            (base + offset).clamp(0, 127) as u8
        } else {
            0
        }
    }

    /// Count active steps
    pub fn active_count(&self) -> usize {
        self.steps[..self.number_steps].iter().filter(|s| s.active).count()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Beat Studio — Main State
// ═══════════════════════════════════════════════════════════════════════════════

/// The Beat Studio workspace
pub struct BeatStudio {
    /// 8 tracks (drums + melodic)
    pub tracks: [BeatTrack; MAXIMUM_BEAT_TRACKS],
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
    pub scope_position: usize,
    pub spectrum: [u8; 16],

    // Keyboard state (which keys are visually "pressed")
    pub keys_pressed: [bool; 128],
    pub octave: i8,
    pub velocity: u8,

    // Layout cache
    framebuffer_w: u32,
    framebuffer_h: u32,
}

// Bloc d'implémentation — définit les méthodes du type ci-dessus.
impl BeatStudio {
    /// Create a new Beat Studio with default demo beat
    pub fn new() -> Self {
        let framebuffer_w = crate::framebuffer::FRAMEBUFFER_WIDTH.load(Ordering::Relaxed) as u32;
        let framebuffer_h = crate::framebuffer::FRAMEBUFFER_HEIGHT.load(Ordering::Relaxed) as u32;

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
            scope_position: 0,
            spectrum: [0u8; 16],
            keys_pressed: [false; 128],
            octave: 0,
            velocity: 100,
            framebuffer_w,
            framebuffer_h,
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

    /// Load a deep house beat — 100 BPM, groovy, showcase-ready
    /// 32 steps = 2 bars, long notes with lookahead, echo FX in render
    pub fn load_funky_house(&mut self) {
        // Configure for deep funky house — slower, more groove
        self.bpm = 100;
        self.swing = 56; // subtle shuffle

        // Expand to 32 steps (2 bars) for variation
        for t in self.tracks.iterator_mut() {
            t.number_steps = 32;
            for s in 0..MAXIMUM_STEPS {
                t.steps[s] = BeatStep::off();
            }
        }

        // ─── Track configuration: 2 bass layers + melody ───
        self.tracks[0] = BeatTrack::new("Kick",    36, Waveform::Sine,     colors::TRACK_COLORS[0], true);
        self.tracks[1] = BeatTrack::new("Clap",    39, Waveform::Noise,    colors::TRACK_COLORS[1], true);
        self.tracks[2] = BeatTrack::new("HiHat",   42, Waveform::Noise,    colors::TRACK_COLORS[2], true);
        self.tracks[3] = BeatTrack::new("Sub Bass", 24, Waveform::Sine,    colors::TRACK_COLORS[3], false);
        self.tracks[4] = BeatTrack::new("Mid Bass", 36, Waveform::Square,  colors::TRACK_COLORS[4], false);
        self.tracks[5] = BeatTrack::new("Chords",  60, Waveform::Triangle, colors::TRACK_COLORS[5], false);
        self.tracks[6] = BeatTrack::new("Lead",    72, Waveform::Sawtooth, colors::TRACK_COLORS[6], false);
        self.tracks[7] = BeatTrack::new("Perc",    56, Waveform::Noise,    colors::TRACK_COLORS[7], true);

        for t in self.tracks.iterator_mut() {
            t.number_steps = 32;
        }

        // ─── Deep house envelopes (longer, smoother) ───
        self.tracks[0].envelope = Envelope::new(2, 200, 0, 80);    // Kick: deep 808 boom, long tail
        self.tracks[1].envelope = Envelope::new(1, 70, 0, 45);     // Clap: tight snap
        self.tracks[2].envelope = Envelope::new(1, 22, 0, 12);     // HiHat: crisp
        self.tracks[3].envelope = Envelope::new(8, 400, 85, 250);  // Sub Bass: very long sustained rumble
        self.tracks[4].envelope = Envelope::new(5, 150, 50, 100);  // Mid Bass: longer pluck
        self.tracks[5].envelope = Envelope::pad();                   // Chords: lush pad
        self.tracks[6].envelope = Envelope::new(10, 200, 70, 180); // Lead: very singing, slow attack
        self.tracks[7].envelope = Envelope::new(1, 30, 0, 18);     // Perc: snap

        // ══════════════════════════════════════════════
        // KICK — punchy four-on-the-floor + ghost notes
        // ══════════════════════════════════════════════
        for i in (0..32).step_by(4) {
            self.tracks[0].steps[i] = BeatStep::on(127);
        }
        self.tracks[0].steps[3]  = BeatStep::on(45);  // ghost
        self.tracks[0].steps[15] = BeatStep::on(40);  // ghost
        self.tracks[0].steps[27] = BeatStep::on(45);  // ghost

        // ══════════════════════════════════════════════
        // CLAP — beats 2 & 4 + ghost flams
        // ══════════════════════════════════════════════
        self.tracks[1].steps[4]  = BeatStep::on(120);
        self.tracks[1].steps[12] = BeatStep::on(120);
        self.tracks[1].steps[20] = BeatStep::on(120);
        self.tracks[1].steps[28] = BeatStep::on(120);
        self.tracks[1].steps[11] = BeatStep::on(50);  // flam
        self.tracks[1].steps[27] = BeatStep::on(55);  // flam

        // ══════════════════════════════════════════════
        // HIHAT — 16th note groove with velocity dynamics
        // ══════════════════════════════════════════════
        for i in 0..16 {
            let vel = // Correspondance de motifs — branchement exhaustif de Rust.
match i % 4 {
                0 => 85,
                2 => 105,  // off-beat accent
                1 => 35,
                3 => 50,
                _ => 45,
            };
            self.tracks[2].steps[i] = BeatStep::on(vel);
        }
        for i in 16..32 {
            let vel = // Correspondance de motifs — branchement exhaustif de Rust.
match i % 4 {
                0 => 80,
                2 => 110,  // bigger accent bar 2
                1 => 30,
                3 => 45,
                _ => 40,
            };
            self.tracks[2].steps[i] = BeatStep::on(vel);
        }
        self.tracks[2].steps[23] = BeatStep::off(); // breathing room
        self.tracks[2].steps[31] = BeatStep::off();

        // ══════════════════════════════════════════════
        // SUB BASS — deep sine, follows chord roots (C1=24)
        // C=0, Eb=3, F=5, G=7, Ab=8, Bb=10
        // Progression: Cm | Cm | Ab | Bb
        // ══════════════════════════════════════════════
        // Bar 1: C1 sustained (Cm), retrigger mid-bar
        self.tracks[3].steps[0]  = BeatStep::on_note(127, 0);   // C1 — root
        self.tracks[3].steps[6]  = BeatStep::on_note(100, 0);   // C1 retrigger
        self.tracks[3].steps[8]  = BeatStep::on_note(120, 0);   // C1
        self.tracks[3].steps[14] = BeatStep::on_note(100, 0);   // C1
        // Bar 2: Ab → Bb (chord change, tension & resolve)
        self.tracks[3].steps[16] = BeatStep::on_note(127, 8);   // Ab1
        self.tracks[3].steps[20] = BeatStep::on_note(110, 8);   // Ab1 retrigger
        self.tracks[3].steps[24] = BeatStep::on_note(127, 10);  // Bb1
        self.tracks[3].steps[28] = BeatStep::on_note(110, 10);  // Bb1 retrigger

        // ══════════════════════════════════════════════
        // MID BASS — funky syncopated pluck (C2=36)
        // Adds the rhythmic punch on top of the sub
        // ══════════════════════════════════════════════
        // Bar 1: C – C – Eb – G – F – Eb (syncopated funky line)
        self.tracks[4].steps[0]  = BeatStep::on_note(120, 0);   // C2
        self.tracks[4].steps[3]  = BeatStep::on_note(110, 0);   // C2 (syncopation!)
        self.tracks[4].steps[5]  = BeatStep::on_note(100, 3);   // Eb2
        self.tracks[4].steps[7]  = BeatStep::on_note(115, 7);   // G2 (bounce)
        self.tracks[4].steps[10] = BeatStep::on_note(105, 5);   // F2
        self.tracks[4].steps[13] = BeatStep::on_note(95, 3);    // Eb2 (pickup)
        // Bar 2: Bb – Ab – G – F – Eb – C (descending groove)
        self.tracks[4].steps[16] = BeatStep::on_note(120, 8);   // Ab2 (follows sub)
        self.tracks[4].steps[19] = BeatStep::on_note(110, 7);   // G2
        self.tracks[4].steps[21] = BeatStep::on_note(100, 5);   // F2
        self.tracks[4].steps[24] = BeatStep::on_note(120, 10);  // Bb2 (follows sub)
        self.tracks[4].steps[26] = BeatStep::on_note(105, 7);   // G2
        self.tracks[4].steps[29] = BeatStep::on_note(100, 5);   // F2
        self.tracks[4].steps[31] = BeatStep::on_note(90, 3);    // Eb2 (resolve)

        // ══════════════════════════════════════════════
        // CHORDS — sustained pads: Cm → Ab → Bb (C4=60)
        // ══════════════════════════════════════════════
        // Bar 1: Cm (C Eb G) — sustained
        self.tracks[5].steps[0]  = BeatStep::on_note(80, 0);    // C4
        self.tracks[5].steps[4]  = BeatStep::on_note(70, 3);    // Eb4
        self.tracks[5].steps[8]  = BeatStep::on_note(75, 7);    // G4
        self.tracks[5].steps[12] = BeatStep::on_note(70, 3);    // Eb4
        // Bar 2: Ab → Bb → Cm resolve
        self.tracks[5].steps[16] = BeatStep::on_note(80, 8);    // Ab4
        self.tracks[5].steps[20] = BeatStep::on_note(75, 7);    // G4
        self.tracks[5].steps[24] = BeatStep::on_note(80, 10);   // Bb4
        self.tracks[5].steps[28] = BeatStep::on_note(75, 7);    // G4

        // ══════════════════════════════════════════════
        // LEAD — catchy deep house melody (C5=72)
        // The hook! Ascending phrase → peak → descending response
        // ══════════════════════════════════════════════
        // Bar 1: Rising hook → G5 → Bb5 → C6 peak → descend
        self.tracks[6].steps[0]  = BeatStep::on_note(100, 7);   // G5
        self.tracks[6].steps[2]  = BeatStep::on_note(105, 10);  // Bb5
        self.tracks[6].steps[3]  = BeatStep::on_note(115, 12);  // C6 (peak!)
        self.tracks[6].steps[5]  = BeatStep::on_note(100, 10);  // Bb5
        self.tracks[6].steps[7]  = BeatStep::on_note(90, 7);    // G5
        self.tracks[6].steps[8]  = BeatStep::on_note(105, 5);   // F5
        self.tracks[6].steps[10] = BeatStep::on_note(100, 3);   // Eb5
        self.tracks[6].steps[12] = BeatStep::on_note(110, 5);   // F5 (bounce back up!)
        self.tracks[6].steps[14] = BeatStep::on_note(105, 7);   // G5
        // Bar 2: Call & response — higher, jazzier phrase
        self.tracks[6].steps[16] = BeatStep::on_note(115, 12);  // C6 (big!)
        self.tracks[6].steps[17] = BeatStep::on_note(100, 10);  // Bb5
        self.tracks[6].steps[19] = BeatStep::on_note(110, 12);  // C6
        self.tracks[6].steps[20] = BeatStep::on_note(120, 15);  // Eb6 (peak peak!)
        self.tracks[6].steps[22] = BeatStep::on_note(105, 12);  // C6
        self.tracks[6].steps[24] = BeatStep::on_note(100, 10);  // Bb5
        self.tracks[6].steps[25] = BeatStep::on_note(95, 7);    // G5
        self.tracks[6].steps[27] = BeatStep::on_note(90, 5);    // F5
        self.tracks[6].steps[29] = BeatStep::on_note(85, 3);    // Eb5
        self.tracks[6].steps[31] = BeatStep::on_note(80, 0);    // C5 (resolve home)

        // ══════════════════════════════════════════════
        // PERC — shakers, rides, fill buildup
        // ══════════════════════════════════════════════
        for i in (1..32).step_by(2) {
            self.tracks[7].steps[i] = BeatStep::on(55);
        }
        for i in (2..32).step_by(4) {
            self.tracks[7].steps[i] = BeatStep::on(90);
        }
        // Fill at end of bar 2 (buildup)
        self.tracks[7].steps[26] = BeatStep::on(90);
        self.tracks[7].steps[27] = BeatStep::on(100);
        self.tracks[7].steps[28] = BeatStep::on(105);
        self.tracks[7].steps[29] = BeatStep::on(110);
        self.tracks[7].steps[30] = BeatStep::on(120);
        self.tracks[7].steps[31] = BeatStep::on(127); // crash!

        // ─── Mix balance: heavy bass focus ───
        self.tracks[0].volume = 230; // Kick: dominant
        self.tracks[1].volume = 185; // Clap: present
        self.tracks[2].volume = 140; // HiHat: background
        self.tracks[3].volume = 255; // Sub Bass: LOUD, the foundation
        self.tracks[4].volume = 200; // Mid Bass: funky, present
        self.tracks[5].volume = 110; // Chords: bed, don't overpower
        self.tracks[6].volume = 175; // Lead: melodic focus
        self.tracks[7].volume = 120; // Perc: texture
    }

    // ═════════════════════════════════════════════════════════════════════════
    // TrustOS Anthem — "Renaissance Numérique"
    // ~3 min: Intro → Build → Drop → Stable → Outro
    // Key: C minor → C major  ·  106 BPM  ·  32 steps  ·  48kHz stereo
    // ═════════════════════════════════════════════════════════════════════════

    /// Base anthem track configuration (instruments, envelopes, volumes)
    fn anthem_initialize(&mut self) {
        self.bpm = 106;
        self.swing = 50; // straight for anthem clarity

        self.tracks[0] = BeatTrack::new("Kick",  36, Waveform::Sine,     colors::TRACK_COLORS[0], true);
        self.tracks[1] = BeatTrack::new("Snare", 38, Waveform::Noise,    colors::TRACK_COLORS[1], true);
        self.tracks[2] = BeatTrack::new("HiHat", 42, Waveform::Noise,    colors::TRACK_COLORS[2], true);
        self.tracks[3] = BeatTrack::new("Sub",   24, Waveform::Sine,     colors::TRACK_COLORS[3], false);
        self.tracks[4] = BeatTrack::new("Bass",  36, Waveform::Square,   colors::TRACK_COLORS[4], false);
        self.tracks[5] = BeatTrack::new("Pad",   60, Waveform::Triangle, colors::TRACK_COLORS[5], false);
        self.tracks[6] = BeatTrack::new("Lead",  72, Waveform::Sawtooth, colors::TRACK_COLORS[6], false);
        self.tracks[7] = BeatTrack::new("Arp",   72, Waveform::Triangle, colors::TRACK_COLORS[7], false);

        for t in self.tracks.iterator_mut() {
            t.number_steps = 32;
            for s in 0..MAXIMUM_STEPS { t.steps[s] = BeatStep::off(); }
        }

        // Cinematic anthem envelopes
        self.tracks[0].envelope = Envelope::new(2, 200, 0, 80);
        self.tracks[1].envelope = Envelope::new(1, 65, 0, 40);
        self.tracks[2].envelope = Envelope::new(1, 18, 0, 8);
        self.tracks[3].envelope = Envelope::new(15, 600, 90, 350);
        self.tracks[4].envelope = Envelope::new(5, 200, 45, 130);
        self.tracks[5].envelope = Envelope::pad();
        self.tracks[6].envelope = Envelope::new(10, 280, 75, 220);
        self.tracks[7].envelope = Envelope::new(3, 100, 25, 70);

        self.tracks[0].volume = 200;
        self.tracks[1].volume = 175;
        self.tracks[2].volume = 130;
        self.tracks[3].volume = 240;
        self.tracks[4].volume = 190;
        self.tracks[5].volume = 100;
        self.tracks[6].volume = 180;
        self.tracks[7].volume = 140;
    }

    /// INTRO — L'Éveil: heartbeat sub + floating pad + digital texture
    fn anthem_intro(&mut self) {
        self.anthem_initialize();
        // Only Sub + Pad + Texture active
        self.tracks[0].muted = true;  // Kick OFF
        self.tracks[1].muted = true;  // Snare OFF
        self.tracks[2].muted = true;  // HiHat OFF
        self.tracks[4].muted = true;  // Bass OFF
        self.tracks[6].muted = true;  // Lead OFF

        // Sub: heartbeat pulse — C1, very sparse
        self.tracks[3].volume = 180;
        self.tracks[3].steps[0]  = BeatStep::on_note(70, 0);   // C1 thump
        self.tracks[3].steps[16] = BeatStep::on_note(50, 0);   // C1 echo

        // Pad: floating Cm chord tones — ethereal
        self.tracks[5].volume = 75;
        self.tracks[5].steps[0]  = BeatStep::on_note(45, 0);   // C4
        self.tracks[5].steps[8]  = BeatStep::on_note(40, 3);   // Eb4
        self.tracks[5].steps[16] = BeatStep::on_note(45, 7);   // G4
        self.tracks[5].steps[24] = BeatStep::on_note(40, 3);   // Eb4

        // Arp → Texture: digital boot blips (noise percussion)
        self.tracks[7] = BeatTrack::new("Texture", 72, Waveform::Noise, colors::TRACK_COLORS[7], true);
        self.tracks[7].number_steps = 32;
        self.tracks[7].envelope = Envelope::new(1, 15, 0, 5);
        self.tracks[7].volume = 50;
        self.tracks[7].steps[4]  = BeatStep::on(25);
        self.tracks[7].steps[5]  = BeatStep::on(20);  // double tap = boot feel
        self.tracks[7].steps[11] = BeatStep::on(30);
        self.tracks[7].steps[18] = BeatStep::on(18);
        self.tracks[7].steps[19] = BeatStep::on(25);
        self.tracks[7].steps[26] = BeatStep::on(28);
    }

    /// BUILD — L'Espoir: soft kick, ascending arp, warm bass
    fn anthem_build(&mut self) {
        self.anthem_initialize();
        self.tracks[1].muted = true;  // Snare still OFF
        self.tracks[6].muted = true;  // Lead still OFF

        // Kick: soft quarter notes
        self.tracks[0].volume = 160;
        for i in (0..32).step_by(8) {
            self.tracks[0].steps[i] = BeatStep::on(80);
        }

        // HiHat: light pattern
        self.tracks[2].volume = 90;
        for i in (0..32).step_by(4) {
            self.tracks[2].steps[i] = BeatStep::on(30);
        }
        for i in (2..32).step_by(4) {
            self.tracks[2].steps[i] = BeatStep::on(55);
        }

        // Sub: gaining strength — Cm → Ab → Bb
        self.tracks[3].volume = 220;
        self.tracks[3].steps[0]  = BeatStep::on_note(100, 0);  // C1
        self.tracks[3].steps[8]  = BeatStep::on_note(90, 0);   // C1
        self.tracks[3].steps[16] = BeatStep::on_note(100, 8);  // Ab1
        self.tracks[3].steps[24] = BeatStep::on_note(95, 10);  // Bb1

        // Bass: warm groove entering
        self.tracks[4].volume = 150;
        self.tracks[4].steps[0]  = BeatStep::on_note(90, 0);   // C2
        self.tracks[4].steps[4]  = BeatStep::on_note(75, 0);   // C2
        self.tracks[4].steps[8]  = BeatStep::on_note(85, 3);   // Eb2
        self.tracks[4].steps[12] = BeatStep::on_note(80, 7);   // G2
        self.tracks[4].steps[16] = BeatStep::on_note(90, 8);   // Ab2
        self.tracks[4].steps[20] = BeatStep::on_note(80, 7);   // G2
        self.tracks[4].steps[24] = BeatStep::on_note(95, 10);  // Bb2
        self.tracks[4].steps[28] = BeatStep::on_note(85, 7);   // G2

        // Pad: richer chords Cm → Ab → Bb
        self.tracks[5].volume = 90;
        self.tracks[5].steps[0]  = BeatStep::on_note(55, 0);   // C4
        self.tracks[5].steps[4]  = BeatStep::on_note(50, 3);   // Eb4
        self.tracks[5].steps[8]  = BeatStep::on_note(55, 7);   // G4
        self.tracks[5].steps[12] = BeatStep::on_note(50, 3);   // Eb4
        self.tracks[5].steps[16] = BeatStep::on_note(55, 8);   // Ab4
        self.tracks[5].steps[20] = BeatStep::on_note(50, 7);   // G4
        self.tracks[5].steps[24] = BeatStep::on_note(55, 10);  // Bb4
        self.tracks[5].steps[28] = BeatStep::on_note(50, 7);   // G4

        // Arp: ascending Cm arpeggio — hope rising!
        self.tracks[7].volume = 120;
        // Bar 1: C5→Eb5→G5→C6 ascending
        self.tracks[7].steps[0]  = BeatStep::on_note(80, 0);   // C5
        self.tracks[7].steps[2]  = BeatStep::on_note(85, 3);   // Eb5
        self.tracks[7].steps[4]  = BeatStep::on_note(90, 7);   // G5
        self.tracks[7].steps[6]  = BeatStep::on_note(95, 12);  // C6
        self.tracks[7].steps[8]  = BeatStep::on_note(80, 0);   // C5
        self.tracks[7].steps[10] = BeatStep::on_note(85, 3);   // Eb5
        self.tracks[7].steps[12] = BeatStep::on_note(90, 7);   // G5
        self.tracks[7].steps[14] = BeatStep::on_note(100, 12); // C6 brighter
        // Bar 2: Ab arp → Bb arp
        self.tracks[7].steps[16] = BeatStep::on_note(80, 8);   // Ab5
        self.tracks[7].steps[18] = BeatStep::on_note(85, 0);   // C5
        self.tracks[7].steps[20] = BeatStep::on_note(90, 3);   // Eb5
        self.tracks[7].steps[22] = BeatStep::on_note(85, 8);   // Ab5
        self.tracks[7].steps[24] = BeatStep::on_note(85, 10);  // Bb5
        self.tracks[7].steps[26] = BeatStep::on_note(90, 2);   // D5
        self.tracks[7].steps[28] = BeatStep::on_note(95, 5);   // F5
        self.tracks[7].steps[30] = BeatStep::on_note(100, 10); // Bb5
    }

    /// DROP — La Révélation: full energy, punchy drums, the hook melody
    fn anthem_drop(&mut self) {
        self.anthem_initialize();
        // ALL tracks active

        // Kick: punchy four-on-floor + ghosts
        self.tracks[0].volume = 225;
        for i in (0..32).step_by(4) {
            self.tracks[0].steps[i] = BeatStep::on(120);
        }
        self.tracks[0].steps[3]  = BeatStep::on(40);
        self.tracks[0].steps[15] = BeatStep::on(35);
        self.tracks[0].steps[19] = BeatStep::on(40);
        self.tracks[0].steps[27] = BeatStep::on(35);

        // Snare: beats 2 & 4
        self.tracks[1].volume = 180;
        self.tracks[1].steps[4]  = BeatStep::on(115);
        self.tracks[1].steps[12] = BeatStep::on(115);
        self.tracks[1].steps[20] = BeatStep::on(115);
        self.tracks[1].steps[28] = BeatStep::on(115);
        self.tracks[1].steps[11] = BeatStep::on(45); // ghost

        // HiHat: 16th groove
        self.tracks[2].volume = 140;
        for i in 0..32 {
            let vel = // Correspondance de motifs — branchement exhaustif de Rust.
match i % 4 { 0 => 80, 2 => 100, 1 => 35, _ => 50 };
            self.tracks[2].steps[i] = BeatStep::on(vel);
        }
        self.tracks[2].steps[15] = BeatStep::off();
        self.tracks[2].steps[31] = BeatStep::off();

        // Sub: Cm → Ab → Bb
        self.tracks[3].volume = 250;
        self.tracks[3].steps[0]  = BeatStep::on_note(127, 0);
        self.tracks[3].steps[6]  = BeatStep::on_note(100, 0);
        self.tracks[3].steps[8]  = BeatStep::on_note(120, 0);
        self.tracks[3].steps[14] = BeatStep::on_note(95, 0);
        self.tracks[3].steps[16] = BeatStep::on_note(127, 8);  // Ab1
        self.tracks[3].steps[20] = BeatStep::on_note(105, 8);
        self.tracks[3].steps[24] = BeatStep::on_note(127, 10); // Bb1
        self.tracks[3].steps[28] = BeatStep::on_note(105, 10);

        // Bass: funky syncopated groove
        self.tracks[4].volume = 200;
        self.tracks[4].steps[0]  = BeatStep::on_note(115, 0);  // C2
        self.tracks[4].steps[3]  = BeatStep::on_note(105, 0);  // syncopation
        self.tracks[4].steps[5]  = BeatStep::on_note(95, 3);   // Eb2
        self.tracks[4].steps[7]  = BeatStep::on_note(110, 7);  // G2
        self.tracks[4].steps[10] = BeatStep::on_note(100, 5);  // F2
        self.tracks[4].steps[13] = BeatStep::on_note(90, 3);   // Eb2
        self.tracks[4].steps[16] = BeatStep::on_note(115, 8);  // Ab2
        self.tracks[4].steps[19] = BeatStep::on_note(105, 7);  // G2
        self.tracks[4].steps[21] = BeatStep::on_note(95, 5);   // F2
        self.tracks[4].steps[24] = BeatStep::on_note(115, 10); // Bb2
        self.tracks[4].steps[26] = BeatStep::on_note(100, 7);  // G2
        self.tracks[4].steps[29] = BeatStep::on_note(95, 5);   // F2
        self.tracks[4].steps[31] = BeatStep::on_note(85, 3);   // Eb2

        // Pad: full chords
        self.tracks[5].volume = 110;
        self.tracks[5].steps[0]  = BeatStep::on_note(70, 0);   // C4
        self.tracks[5].steps[4]  = BeatStep::on_note(65, 3);   // Eb4
        self.tracks[5].steps[8]  = BeatStep::on_note(70, 7);   // G4
        self.tracks[5].steps[12] = BeatStep::on_note(65, 3);   // Eb4
        self.tracks[5].steps[16] = BeatStep::on_note(70, 8);   // Ab4
        self.tracks[5].steps[20] = BeatStep::on_note(65, 7);   // G4
        self.tracks[5].steps[24] = BeatStep::on_note(70, 10);  // Bb4
        self.tracks[5].steps[28] = BeatStep::on_note(65, 7);   // G4

        // Lead: THE MELODY — ascending arc, peak, descend
        self.tracks[6].volume = 190;
        // Bar 1: G5→Bb5→C6(peak)→Bb5→Ab5→G5
        self.tracks[6].steps[0]  = BeatStep::on_note(100, 7);  // G5
        self.tracks[6].steps[3]  = BeatStep::on_note(110, 10); // Bb5
        self.tracks[6].steps[6]  = BeatStep::on_note(120, 12); // C6 PEAK
        self.tracks[6].steps[10] = BeatStep::on_note(105, 10); // Bb5
        self.tracks[6].steps[12] = BeatStep::on_note(100, 8);  // Ab5
        self.tracks[6].steps[14] = BeatStep::on_note(95, 7);   // G5
        // Bar 2: Eb5→G5→C6→Eb6(CLIMAX!)→C6→Bb5→G5→C5
        self.tracks[6].steps[16] = BeatStep::on_note(105, 3);  // Eb5
        self.tracks[6].steps[18] = BeatStep::on_note(110, 7);  // G5
        self.tracks[6].steps[20] = BeatStep::on_note(120, 12); // C6
        self.tracks[6].steps[22] = BeatStep::on_note(127, 15); // Eb6 CLIMAX!
        self.tracks[6].steps[24] = BeatStep::on_note(110, 12); // C6
        self.tracks[6].steps[26] = BeatStep::on_note(100, 10); // Bb5
        self.tracks[6].steps[28] = BeatStep::on_note(90, 7);   // G5
        self.tracks[6].steps[30] = BeatStep::on_note(85, 0);   // C5 resolve

        // Arp: staccato accents between lead notes
        self.tracks[7].volume = 130;
        self.tracks[7].steps[1]  = BeatStep::on_note(70, 12);  // C6
        self.tracks[7].steps[5]  = BeatStep::on_note(65, 7);   // G5
        self.tracks[7].steps[9]  = BeatStep::on_note(70, 12);  // C6
        self.tracks[7].steps[13] = BeatStep::on_note(65, 3);   // Eb5
        self.tracks[7].steps[17] = BeatStep::on_note(70, 8);   // Ab5
        self.tracks[7].steps[21] = BeatStep::on_note(75, 12);  // C6
        self.tracks[7].steps[25] = BeatStep::on_note(70, 10);  // Bb5
        self.tracks[7].steps[29] = BeatStep::on_note(65, 7);   // G5
    }

    /// STABLE — La Maîtrise: C minor → C MAJOR! The TrustOS identity.
    fn anthem_stable(&mut self) {
        self.anthem_initialize();

        // Kick: simplified, confident
        self.tracks[0].volume = 210;
        for i in (0..32).step_by(8) {
            self.tracks[0].steps[i] = BeatStep::on(110);
        }

        // Snare: clean backbeat
        self.tracks[1].volume = 165;
        self.tracks[1].steps[4]  = BeatStep::on(105);
        self.tracks[1].steps[12] = BeatStep::on(105);
        self.tracks[1].steps[20] = BeatStep::on(105);
        self.tracks[1].steps[28] = BeatStep::on(105);

        // HiHat: lighter
        self.tracks[2].volume = 100;
        for i in (2..32).step_by(4) {
            self.tracks[2].steps[i] = BeatStep::on(60);
        }

        // Sub: sustained
        self.tracks[3].volume = 230;
        self.tracks[3].steps[0]  = BeatStep::on_note(110, 0);  // C1
        self.tracks[3].steps[8]  = BeatStep::on_note(100, 0);
        self.tracks[3].steps[16] = BeatStep::on_note(110, 8);  // Ab1
        self.tracks[3].steps[24] = BeatStep::on_note(105, 10); // Bb1

        // Bass: stable groove — now with E natural (MAJOR!)
        self.tracks[4].volume = 180;
        self.tracks[4].steps[0]  = BeatStep::on_note(100, 0);  // C2
        self.tracks[4].steps[4]  = BeatStep::on_note(85, 7);   // G2
        self.tracks[4].steps[8]  = BeatStep::on_note(95, 4);   // E2 (MAJOR!)
        self.tracks[4].steps[12] = BeatStep::on_note(85, 0);   // C2
        self.tracks[4].steps[16] = BeatStep::on_note(100, 8);  // Ab2
        self.tracks[4].steps[20] = BeatStep::on_note(85, 7);   // G2
        self.tracks[4].steps[24] = BeatStep::on_note(100, 10); // Bb2
        self.tracks[4].steps[28] = BeatStep::on_note(90, 5);   // F2

        // Pad: C MAJOR tones — the shift!
        self.tracks[5].volume = 105;
        self.tracks[5].steps[0]  = BeatStep::on_note(60, 0);   // C4
        self.tracks[5].steps[8]  = BeatStep::on_note(55, 4);   // E4 (MAJOR!)
        self.tracks[5].steps[16] = BeatStep::on_note(60, 7);   // G4
        self.tracks[5].steps[24] = BeatStep::on_note(55, 12);  // C5

        // Lead: THE TRUSTOS MOTIF — C E G C (C MAJOR ARPEGGIO!)
        // This IS the TrustOS sound. Sovereign. Recognizable.
        self.tracks[6].volume = 200;
        // Bar 1: ascending C→E→G→C
        self.tracks[6].steps[0]  = BeatStep::on_note(110, 0);  // C5
        self.tracks[6].steps[4]  = BeatStep::on_note(115, 4);  // E5 ← MAJOR THIRD!
        self.tracks[6].steps[8]  = BeatStep::on_note(120, 7);  // G5
        self.tracks[6].steps[12] = BeatStep::on_note(127, 12); // C6 peak!
        // Bar 2: mirror descent C→G→E→C
        self.tracks[6].steps[16] = BeatStep::on_note(120, 12); // C6
        self.tracks[6].steps[20] = BeatStep::on_note(115, 7);  // G5
        self.tracks[6].steps[24] = BeatStep::on_note(110, 4);  // E5
        self.tracks[6].steps[28] = BeatStep::on_note(105, 0);  // C5 home

        // Arp: sparkling C major counter-melody (high register)
        self.tracks[7].volume = 115;
        self.tracks[7].steps[2]  = BeatStep::on_note(75, 12);  // C6
        self.tracks[7].steps[6]  = BeatStep::on_note(70, 16);  // E6
        self.tracks[7].steps[10] = BeatStep::on_note(75, 19);  // G6
        self.tracks[7].steps[14] = BeatStep::on_note(80, 12);  // C6
        self.tracks[7].steps[18] = BeatStep::on_note(70, 19);  // G6
        self.tracks[7].steps[22] = BeatStep::on_note(75, 16);  // E6
        self.tracks[7].steps[26] = BeatStep::on_note(70, 12);  // C6
        self.tracks[7].steps[30] = BeatStep::on_note(65, 7);   // G5
    }

    /// OUTRO — Le Futur Souverain: pad + motif fading into silence
    fn anthem_outro(&mut self) {
        self.anthem_initialize();
        // Only Sub + Pad + Lead
        self.tracks[0].muted = true;  // Kick OFF
        self.tracks[1].muted = true;  // Snare OFF
        self.tracks[2].muted = true;  // HiHat OFF
        self.tracks[4].muted = true;  // Bass OFF
        self.tracks[7].muted = true;  // Arp OFF

        // Sub: heartbeat fading
        self.tracks[3].volume = 150;
        self.tracks[3].steps[0]  = BeatStep::on_note(60, 0);   // C1
        self.tracks[3].steps[16] = BeatStep::on_note(40, 0);   // C1 soft

        // Pad: gentle C (no third — open, sovereign)
        self.tracks[5].volume = 80;
        self.tracks[5].steps[0]  = BeatStep::on_note(40, 0);   // C4
        self.tracks[5].steps[8]  = BeatStep::on_note(35, 7);   // G4
        self.tracks[5].steps[16] = BeatStep::on_note(40, 12);  // C5
        self.tracks[5].steps[24] = BeatStep::on_note(35, 7);   // G4

        // Lead: just the ascending motif, sparse and gentle
        self.tracks[6].volume = 140;
        self.tracks[6].steps[0]  = BeatStep::on_note(80, 0);   // C5
        self.tracks[6].steps[4]  = BeatStep::on_note(75, 4);   // E5
        self.tracks[6].steps[8]  = BeatStep::on_note(80, 7);   // G5
        self.tracks[6].steps[12] = BeatStep::on_note(85, 12);  // C6
        // Second half: silence (the motif floats away)
    }

    // ═════════════════════════════════════════════════════════════════════════
    // CYBERPUNK TRAP — "NEON PROTOCOL"
    // 100 BPM  ·  Eb minor  ·  Dark cyberpunk energy
    // Analyzed from Suno reference: Eb minor, 100 BPM, sub-bass 43-45Hz
    //
    // Structure: Intro / Drop / Breakdown / Build / Bridge / Rebuild /
    //            Final Drop / Outro
    // Key: Eb minor (Eb F Gb Ab Bb Cb Db) — cold, digital, neon-noir
    // Ref dominant freq: 43Hz (Eb1), centroid 2500-3500Hz (bright metal)
    // ═════════════════════════════════════════════════════════════════════════

    /// Common track/envelope/volume setup for all cyberpunk sections.
    /// Sharper envelopes for digital character. Sawtooth lead for edge.
    fn trap_base(&mut self) {
        self.bpm = 100;   // 100 BPM — dark, methodical pulse
        self.swing = 50;

        // Track layout: Sub, Snare, HiHat, OpenHat, Synth, Pad, Lead, Perc
        // Key: Eb minor — base notes tuned to Eb
        self.tracks[0] = BeatTrack::new("Sub",     27, Waveform::Sine,     colors::TRACK_COLORS[0], false);  // Eb1=27
        self.tracks[1] = BeatTrack::new("Snare",   38, Waveform::Noise,    colors::TRACK_COLORS[1], true);
        self.tracks[2] = BeatTrack::new("HiHat",   56, Waveform::Noise,    colors::TRACK_COLORS[2], true);
        self.tracks[3] = BeatTrack::new("OpenHat", 53, Waveform::Noise,    colors::TRACK_COLORS[3], true);
        self.tracks[4] = BeatTrack::new("Synth",   63, Waveform::Square,   colors::TRACK_COLORS[4], false);  // Eb4=63
        self.tracks[5] = BeatTrack::new("Pad",     51, Waveform::Sawtooth, colors::TRACK_COLORS[5], false);  // Eb3=51
        self.tracks[6] = BeatTrack::new("Lead",    75, Waveform::Sawtooth, colors::TRACK_COLORS[6], false);  // Eb5=75
        self.tracks[7] = BeatTrack::new("Perc",    63, Waveform::Noise,    colors::TRACK_COLORS[7], true);

        for t in self.tracks.iterator_mut() {
            t.number_steps = 32;
            for s in 0..MAXIMUM_STEPS { t.steps[s] = BeatStep::off(); }
            t.muted = false;
        }

        // Envelopes: sharper attacks for cyberpunk edge, long sub decay
        self.tracks[0].envelope = Envelope::new(1, 1800, 80, 600);  // Sub: instant punch, LONG sustain
        self.tracks[1].envelope = Envelope::new(1, 80, 0, 35);      // Snare: hard crack
        self.tracks[2].envelope = Envelope::new(1, 16, 0, 6);       // HiHat: metallic tick
        self.tracks[3].envelope = Envelope::new(1, 140, 0, 80);     // Open hat: digital sizzle
        self.tracks[4].envelope = Envelope::new(3, 380, 25, 260);   // Synth: sharp pluck, dark tail
        self.tracks[5].envelope = Envelope::pad();                   // Pad: atmospheric wash
        self.tracks[6].envelope = Envelope::new(4, 300, 50, 220);   // Lead: aggressive, cutting
        self.tracks[7].envelope = Envelope::new(1, 25, 0, 8);       // Perc: glitch click
    }

    // ──────────────────────────────────────────────────────────────────
    // INTRO  (4 loops)  —  "System Boot"
    // Dark ambience: sub drone, pad swell, distant synth motif.
    // No drums. Ref: 0-16s RMS=0.13, sub=674, mid=28 (atmosphere only)
    // ──────────────────────────────────────────────────────────────────
    pub fn load_trap_intro(&mut self) {
        self.trap_base();
        self.tracks[1].muted = true;   // no snare
        self.tracks[2].muted = true;   // no hats
        self.tracks[3].muted = true;   // no open hat
        self.tracks[7].muted = true;   // no perc

        // Sub: low drone — felt more than heard
        self.tracks[0].volume = 120;
        self.tracks[0].steps[0]  = BeatStep::on_note(60, 0);     // Eb1

        // Pad: cold digital swell
        self.tracks[5].volume = 50;
        self.tracks[5].steps[0]  = BeatStep::on_note(35, 0);     // Eb3
        self.tracks[5].steps[8]  = BeatStep::on_note(30, 5);     // Ab3
        self.tracks[5].steps[16] = BeatStep::on_note(35, 3);     // Gb3
        self.tracks[5].steps[24] = BeatStep::on_note(30, -2);    // Db3

        // Synth: distant motif — the "neon flicker"
        self.tracks[4].volume = 55;
        self.tracks[4].steps[4]  = BeatStep::on_note(40, 0);     // Eb4
        self.tracks[4].steps[12] = BeatStep::on_note(35, -3);    // B3 (chromatic tension)
        self.tracks[4].steps[20] = BeatStep::on_note(38, 5);     // Ab4

        // Lead: one ghostly phrase — high, thin
        self.tracks[6].volume = 35;
        self.tracks[6].steps[16] = BeatStep::on_note(30, 7);     // Bb5
        self.tracks[6].steps[24] = BeatStep::on_note(25, 0);     // Eb5
    }

    // ──────────────────────────────────────────────────────────────────
    // DROP / HOOK  (5 loops)  —  "Neon Protocol"
    // FULL ENERGY. Sub DOMINATES. Aggressive hats, hard snare.
    // Synth arpeggio Ebm→B→Ab→Gb. Lead melody: digital, angular.
    // Ref: 16-56s RMS=0.28, sub=1490, centroid=2900Hz, hi=24
    // ──────────────────────────────────────────────────────────────────
    pub fn load_trap_hook(&mut self) {
        self.trap_base();

        // ── Sub: MASSIVE — Eb1 dominance (ref 43Hz peak) ──
        self.tracks[0].volume = 255;
        self.tracks[0].steps[0]  = BeatStep::on_note(127, 0);    // Eb1 — hard downbeat
        self.tracks[0].steps[6]  = BeatStep::on_note(90, 0);     // Eb1 — ghost retrigger
        self.tracks[0].steps[8]  = BeatStep::on_note(118, 0);    // Eb1 — beat 3
        self.tracks[0].steps[16] = BeatStep::on_note(125, 5);    // Ab1
        self.tracks[0].steps[20] = BeatStep::on_note(105, 3);    // Gb1
        self.tracks[0].steps[24] = BeatStep::on_note(120, -2);   // Db1
        self.tracks[0].steps[28] = BeatStep::on_note(100, 0);    // Eb1

        // ── Snare: hard, mechanical (beats 3 & 7) ──
        self.tracks[1].volume = 185;
        self.tracks[1].steps[8]  = BeatStep::on(120);
        self.tracks[1].steps[24] = BeatStep::on(118);
        self.tracks[1].steps[6]  = BeatStep::on(32);             // ghost
        self.tracks[1].steps[22] = BeatStep::on(30);             // ghost
        self.tracks[1].steps[15] = BeatStep::on(40);             // off-beat snap

        // ── HiHat: aggressive 16ths — cyberpunk machine-gun ──
        // Ref shows high onset density (56 onsets / 8s) and high treble
        self.tracks[2].volume = 100;
        // Bar 1: driving 16ths with accent pattern
        self.tracks[2].steps[0]  = BeatStep::on(90);
        self.tracks[2].steps[1]  = BeatStep::on(38);
        self.tracks[2].steps[2]  = BeatStep::on(72);
        self.tracks[2].steps[3]  = BeatStep::on(35);
        self.tracks[2].steps[4]  = BeatStep::on(85);
        self.tracks[2].steps[5]  = BeatStep::on(32);
        self.tracks[2].steps[6]  = BeatStep::on(68);
        self.tracks[2].steps[7]  = BeatStep::on(30);
        self.tracks[2].steps[8]  = BeatStep::on(88);
        self.tracks[2].steps[9]  = BeatStep::on(36);
        self.tracks[2].steps[10] = BeatStep::on(70);
        self.tracks[2].steps[11] = BeatStep::on(33);
        self.tracks[2].steps[12] = BeatStep::on(82);
        self.tracks[2].steps[13] = BeatStep::on(35);
        self.tracks[2].steps[14] = BeatStep::on(75);
        self.tracks[2].steps[15] = BeatStep::on(40);
        // Bar 2: same pattern, roll at end
        self.tracks[2].steps[16] = BeatStep::on(90);
        self.tracks[2].steps[17] = BeatStep::on(38);
        self.tracks[2].steps[18] = BeatStep::on(72);
        self.tracks[2].steps[19] = BeatStep::on(35);
        self.tracks[2].steps[20] = BeatStep::on(85);
        self.tracks[2].steps[21] = BeatStep::on(32);
        self.tracks[2].steps[22] = BeatStep::on(68);
        self.tracks[2].steps[23] = BeatStep::on(30);
        // glitch roll (24-31)
        self.tracks[2].steps[24] = BeatStep::on(42);
        self.tracks[2].steps[25] = BeatStep::on(52);
        self.tracks[2].steps[26] = BeatStep::on(62);
        self.tracks[2].steps[27] = BeatStep::on(72);
        self.tracks[2].steps[28] = BeatStep::on(82);
        self.tracks[2].steps[29] = BeatStep::on(92);
        self.tracks[2].steps[30] = BeatStep::on(102);
        self.tracks[2].steps[31] = BeatStep::on(115);

        // ── Open hat: off-beat digital wash ──
        self.tracks[3].volume = 70;
        self.tracks[3].steps[4]  = BeatStep::on(65);
        self.tracks[3].steps[12] = BeatStep::on(60);
        self.tracks[3].steps[20] = BeatStep::on(58);

        // ── Synth: dark arpeggio Ebm→B→Ab→Gb ──
        self.tracks[4].volume = 95;
        // Bar 1: Ebm (Eb Gb Bb) → B (chromatic tension)
        self.tracks[4].steps[0]  = BeatStep::on_note(82, 0);     // Eb4
        self.tracks[4].steps[2]  = BeatStep::on_note(60, 3);     // Gb4
        self.tracks[4].steps[4]  = BeatStep::on_note(70, 7);     // Bb4
        self.tracks[4].steps[6]  = BeatStep::on_note(55, 12);    // Eb5
        self.tracks[4].steps[8]  = BeatStep::on_note(78, -4);    // B3 (chromatic!)
        self.tracks[4].steps[10] = BeatStep::on_note(58, 0);     // Eb4
        self.tracks[4].steps[12] = BeatStep::on_note(68, 3);     // Gb4
        // Bar 2: Ab → Gb
        self.tracks[4].steps[16] = BeatStep::on_note(80, 5);     // Ab4
        self.tracks[4].steps[18] = BeatStep::on_note(58, 9);     // C5
        self.tracks[4].steps[20] = BeatStep::on_note(72, 12);    // Eb5
        self.tracks[4].steps[24] = BeatStep::on_note(75, 3);     // Gb4
        self.tracks[4].steps[26] = BeatStep::on_note(55, 7);     // Bb4
        self.tracks[4].steps[28] = BeatStep::on_note(65, -2);    // Db4

        // ── Pad: cold Ebm wash ──
        self.tracks[5].volume = 42;
        self.tracks[5].steps[0]  = BeatStep::on_note(32, 0);     // Eb3
        self.tracks[5].steps[8]  = BeatStep::on_note(28, 5);     // Ab3
        self.tracks[5].steps[16] = BeatStep::on_note(32, 3);     // Gb3
        self.tracks[5].steps[24] = BeatStep::on_note(28, -2);    // Db3

        // ── Lead: THE HOOK — angular, digital, aggressive ──
        // Bar 1: Bb5→Gb5→Bb5→Eb6 (ascending peak)
        self.tracks[6].volume = 125;
        self.tracks[6].steps[0]  = BeatStep::on_note(105, 7);    // Bb5 — attack
        self.tracks[6].steps[2]  = BeatStep::on_note(88, 3);     // Gb5 — dip
        self.tracks[6].steps[4]  = BeatStep::on_note(95, 7);     // Bb5 — bounce
        self.tracks[6].steps[8]  = BeatStep::on_note(118, 12);   // Eb6! — PEAK
        // (silence 9-13: ring)
        self.tracks[6].steps[14] = BeatStep::on_note(80, 5);     // Ab5
        // Bar 2: descent — Ab5→Gb5→Db5→Eb5
        self.tracks[6].steps[16] = BeatStep::on_note(95, 5);     // Ab5
        self.tracks[6].steps[19] = BeatStep::on_note(85, 3);     // Gb5
        self.tracks[6].steps[22] = BeatStep::on_note(78, -2);    // Db5
        self.tracks[6].steps[26] = BeatStep::on_note(72, 0);     // Eb5 — resolve

        // ── Perc: glitch clicks — mechanical texture ──
        self.tracks[7].volume = 55;
        self.tracks[7].steps[3]  = BeatStep::on(42);
        self.tracks[7].steps[7]  = BeatStep::on(38);
        self.tracks[7].steps[11] = BeatStep::on(48);
        self.tracks[7].steps[15] = BeatStep::on(35);
        self.tracks[7].steps[19] = BeatStep::on(45);
        self.tracks[7].steps[23] = BeatStep::on(40);
        self.tracks[7].steps[27] = BeatStep::on(50);
        self.tracks[7].steps[31] = BeatStep::on(55);
    }

    // ──────────────────────────────────────────────────────────────────
    // VERSE / BREAKDOWN  (1 loop)  —  "Signal Lost"
    // Energy drops. Sub fades, hats sparse, synth echoes.
    // Ref: 56-64s RMS=0.14, centroid 2948Hz
    // ──────────────────────────────────────────────────────────────────
    pub fn load_trap_verse(&mut self) {
        self.trap_base();

        // Sub: quieter, just pulse
        self.tracks[0].volume = 160;
        self.tracks[0].steps[0]  = BeatStep::on_note(85, 0);     // Eb1
        self.tracks[0].steps[16] = BeatStep::on_note(75, 0);     // Eb1

        // Snare: sparse, off-beat
        self.tracks[1].volume = 140;
        self.tracks[1].steps[8]  = BeatStep::on(90);
        self.tracks[1].steps[24] = BeatStep::on(85);

        // HiHat: sparse — every other beat
        self.tracks[2].volume = 72;
        self.tracks[2].steps[0]  = BeatStep::on(65);
        self.tracks[2].steps[4]  = BeatStep::on(55);
        self.tracks[2].steps[8]  = BeatStep::on(60);
        self.tracks[2].steps[12] = BeatStep::on(50);
        self.tracks[2].steps[16] = BeatStep::on(65);
        self.tracks[2].steps[20] = BeatStep::on(55);
        self.tracks[2].steps[24] = BeatStep::on(60);
        self.tracks[2].steps[28] = BeatStep::on(50);

        // Open hat: one accent
        self.tracks[3].volume = 55;
        self.tracks[3].steps[14] = BeatStep::on(55);

        // Synth: echoing motif from intro
        self.tracks[4].volume = 70;
        self.tracks[4].steps[4]  = BeatStep::on_note(55, 0);     // Eb4
        self.tracks[4].steps[12] = BeatStep::on_note(48, -3);    // B3
        self.tracks[4].steps[20] = BeatStep::on_note(52, 5);     // Ab4
        self.tracks[4].steps[28] = BeatStep::on_note(45, 3);     // Gb4

        // Pad: atmospheric
        self.tracks[5].volume = 55;
        self.tracks[5].steps[0]  = BeatStep::on_note(32, 0);     // Eb3
        self.tracks[5].steps[16] = BeatStep::on_note(28, 7);     // Bb3

        // Lead: minimal — floating
        self.tracks[6].volume = 65;
        self.tracks[6].steps[0]  = BeatStep::on_note(55, 7);     // Bb5
        self.tracks[6].steps[12] = BeatStep::on_note(48, 0);     // Eb5

        // Perc: sparse
        self.tracks[7].volume = 35;
        self.tracks[7].steps[7]  = BeatStep::on(30);
        self.tracks[7].steps[23] = BeatStep::on(28);
    }

    // ──────────────────────────────────────────────────────────────────
    // BUILD  (4 loops)  —  "Recompile"
    // Energy rises gradually. Sub returns, hats accelerate.
    // Ref: 64-96s RMS=0.22→0.28, building onset density
    // ──────────────────────────────────────────────────────────────────
    pub fn load_trap_build(&mut self) {
        self.trap_base();

        // Sub: strong return
        self.tracks[0].volume = 235;
        self.tracks[0].steps[0]  = BeatStep::on_note(118, 0);    // Eb1
        self.tracks[0].steps[8]  = BeatStep::on_note(105, 0);    // Eb1
        self.tracks[0].steps[16] = BeatStep::on_note(115, 5);    // Ab1
        self.tracks[0].steps[20] = BeatStep::on_note(95, 3);     // Gb1
        self.tracks[0].steps[24] = BeatStep::on_note(110, 0);    // Eb1
        self.tracks[0].steps[28] = BeatStep::on_note(88, -2);    // Db1

        // Snare: half-time + crescendo ghosts
        self.tracks[1].volume = 175;
        self.tracks[1].steps[8]  = BeatStep::on(115);
        self.tracks[1].steps[24] = BeatStep::on(112);
        self.tracks[1].steps[4]  = BeatStep::on(28);
        self.tracks[1].steps[12] = BeatStep::on(35);
        self.tracks[1].steps[20] = BeatStep::on(30);

        // HiHat: building triplets
        self.tracks[2].volume = 90;
        // Bar 1: sparse start
        self.tracks[2].steps[0]  = BeatStep::on(82);
        self.tracks[2].steps[2]  = BeatStep::on(40);
        self.tracks[2].steps[4]  = BeatStep::on(75);
        self.tracks[2].steps[6]  = BeatStep::on(38);
        self.tracks[2].steps[8]  = BeatStep::on(80);
        self.tracks[2].steps[10] = BeatStep::on(42);
        self.tracks[2].steps[12] = BeatStep::on(72);
        self.tracks[2].steps[14] = BeatStep::on(35);
        // Bar 2: accelerate to full
        self.tracks[2].steps[16] = BeatStep::on(85);
        self.tracks[2].steps[17] = BeatStep::on(35);
        self.tracks[2].steps[18] = BeatStep::on(70);
        self.tracks[2].steps[19] = BeatStep::on(32);
        self.tracks[2].steps[20] = BeatStep::on(80);
        self.tracks[2].steps[21] = BeatStep::on(38);
        self.tracks[2].steps[22] = BeatStep::on(68);
        self.tracks[2].steps[23] = BeatStep::on(30);
        // roll crescendo
        self.tracks[2].steps[24] = BeatStep::on(40);
        self.tracks[2].steps[25] = BeatStep::on(48);
        self.tracks[2].steps[26] = BeatStep::on(58);
        self.tracks[2].steps[27] = BeatStep::on(68);
        self.tracks[2].steps[28] = BeatStep::on(78);
        self.tracks[2].steps[29] = BeatStep::on(88);
        self.tracks[2].steps[30] = BeatStep::on(98);
        self.tracks[2].steps[31] = BeatStep::on(110);

        // Open hat
        self.tracks[3].volume = 60;
        self.tracks[3].steps[6]  = BeatStep::on(60);
        self.tracks[3].steps[14] = BeatStep::on(55);
        self.tracks[3].steps[22] = BeatStep::on(58);

        // Synth: building arpeggio
        self.tracks[4].volume = 85;
        self.tracks[4].steps[0]  = BeatStep::on_note(72, 0);     // Eb4
        self.tracks[4].steps[4]  = BeatStep::on_note(58, 3);     // Gb4
        self.tracks[4].steps[8]  = BeatStep::on_note(68, 7);     // Bb4
        self.tracks[4].steps[12] = BeatStep::on_note(55, 5);     // Ab4
        self.tracks[4].steps[16] = BeatStep::on_note(75, 0);     // Eb4
        self.tracks[4].steps[20] = BeatStep::on_note(62, 3);     // Gb4
        self.tracks[4].steps[24] = BeatStep::on_note(70, -2);    // Db4
        self.tracks[4].steps[28] = BeatStep::on_note(60, 0);     // Eb4

        // Pad: growing
        self.tracks[5].volume = 48;
        self.tracks[5].steps[0]  = BeatStep::on_note(35, 0);     // Eb3
        self.tracks[5].steps[8]  = BeatStep::on_note(30, 5);     // Ab3
        self.tracks[5].steps[16] = BeatStep::on_note(35, 3);     // Gb3
        self.tracks[5].steps[24] = BeatStep::on_note(32, 7);     // Bb3

        // Lead: rising phrase
        self.tracks[6].volume = 100;
        self.tracks[6].steps[0]  = BeatStep::on_note(75, 0);     // Eb5
        self.tracks[6].steps[4]  = BeatStep::on_note(82, 3);     // Gb5
        self.tracks[6].steps[8]  = BeatStep::on_note(90, 7);     // Bb5 (rising)
        self.tracks[6].steps[16] = BeatStep::on_note(80, 5);     // Ab5
        self.tracks[6].steps[20] = BeatStep::on_note(88, 7);     // Bb5
        self.tracks[6].steps[24] = BeatStep::on_note(95, 12);    // Eb6! (pre-peak)

        // Perc: building texture
        self.tracks[7].volume = 48;
        self.tracks[7].steps[3]  = BeatStep::on(35);
        self.tracks[7].steps[7]  = BeatStep::on(32);
        self.tracks[7].steps[11] = BeatStep::on(40);
        self.tracks[7].steps[15] = BeatStep::on(30);
        self.tracks[7].steps[19] = BeatStep::on(38);
        self.tracks[7].steps[23] = BeatStep::on(35);
        self.tracks[7].steps[27] = BeatStep::on(42);
        self.tracks[7].steps[31] = BeatStep::on(48);
    }

    // ──────────────────────────────────────────────────────────────────
    // BRIDGE  (1 loop)  —  "Blackout"
    // NEAR SILENCE. Everything drops. Only pad + distant synth.
    // Ref: 96-104s RMS=0.09, sub=477 (almost gone), centroid=488Hz
    // Maximum contrast before final sections.
    // ──────────────────────────────────────────────────────────────────
    pub fn load_trap_bridge(&mut self) {
        self.trap_base();
        self.tracks[0].muted = true;   // NO sub — silence!
        self.tracks[1].muted = true;   // no snare
        self.tracks[2].muted = true;   // no hats
        self.tracks[3].muted = true;   // no open hat
        self.tracks[7].muted = true;   // no perc

        // Pad: cold, empty wash
        self.tracks[5].volume = 55;
        self.tracks[5].steps[0]  = BeatStep::on_note(35, 0);     // Eb3
        self.tracks[5].steps[8]  = BeatStep::on_note(28, 7);     // Bb3
        self.tracks[5].steps[16] = BeatStep::on_note(32, -2);    // Db3
        self.tracks[5].steps[24] = BeatStep::on_note(28, 0);     // Eb3

        // Synth: distant, broken signal
        self.tracks[4].volume = 45;
        self.tracks[4].steps[8]  = BeatStep::on_note(35, 0);     // Eb4
        self.tracks[4].steps[20] = BeatStep::on_note(30, -3);    // B3

        // Lead: barely there
        self.tracks[6].volume = 30;
        self.tracks[6].steps[0]  = BeatStep::on_note(28, 7);     // Bb5
        self.tracks[6].steps[16] = BeatStep::on_note(25, 0);     // Eb5
    }

    // ──────────────────────────────────────────────────────────────────
    // REBUILD  (3 loops)  —  "Reboot Sequence"
    // Gradually returning. Sub returns, hats come back.
    // Ref: 104-128s RMS=0.16→0.19, onset building
    // ──────────────────────────────────────────────────────────────────
    pub fn load_trap_rebuild(&mut self) {
        self.trap_base();

        // Sub: returning, moderate
        self.tracks[0].volume = 195;
        self.tracks[0].steps[0]  = BeatStep::on_note(100, 0);    // Eb1
        self.tracks[0].steps[8]  = BeatStep::on_note(88, 0);     // Eb1
        self.tracks[0].steps[16] = BeatStep::on_note(95, 5);     // Ab1
        self.tracks[0].steps[24] = BeatStep::on_note(85, 0);     // Eb1

        // Snare: half-time
        self.tracks[1].volume = 155;
        self.tracks[1].steps[8]  = BeatStep::on(105);
        self.tracks[1].steps[24] = BeatStep::on(100);
        self.tracks[1].steps[7]  = BeatStep::on(25);
        self.tracks[1].steps[23] = BeatStep::on(22);

        // HiHat: moderate 8ths (building back)
        self.tracks[2].volume = 78;
        self.tracks[2].steps[0]  = BeatStep::on(72);
        self.tracks[2].steps[2]  = BeatStep::on(38);
        self.tracks[2].steps[4]  = BeatStep::on(65);
        self.tracks[2].steps[6]  = BeatStep::on(35);
        self.tracks[2].steps[8]  = BeatStep::on(70);
        self.tracks[2].steps[10] = BeatStep::on(36);
        self.tracks[2].steps[12] = BeatStep::on(62);
        self.tracks[2].steps[14] = BeatStep::on(33);
        self.tracks[2].steps[16] = BeatStep::on(72);
        self.tracks[2].steps[18] = BeatStep::on(38);
        self.tracks[2].steps[20] = BeatStep::on(65);
        self.tracks[2].steps[22] = BeatStep::on(35);
        self.tracks[2].steps[24] = BeatStep::on(70);
        self.tracks[2].steps[26] = BeatStep::on(40);
        self.tracks[2].steps[28] = BeatStep::on(65);
        self.tracks[2].steps[30] = BeatStep::on(50);

        // Open hat
        self.tracks[3].volume = 55;
        self.tracks[3].steps[6]  = BeatStep::on(52);
        self.tracks[3].steps[22] = BeatStep::on(48);

        // Synth: motif building
        self.tracks[4].volume = 78;
        self.tracks[4].steps[0]  = BeatStep::on_note(65, 0);     // Eb4
        self.tracks[4].steps[4]  = BeatStep::on_note(52, 3);     // Gb4
        self.tracks[4].steps[8]  = BeatStep::on_note(60, 7);     // Bb4
        self.tracks[4].steps[16] = BeatStep::on_note(68, 5);     // Ab4
        self.tracks[4].steps[20] = BeatStep::on_note(55, 0);     // Eb4
        self.tracks[4].steps[24] = BeatStep::on_note(62, 3);     // Gb4

        // Pad: atmospheric
        self.tracks[5].volume = 45;
        self.tracks[5].steps[0]  = BeatStep::on_note(30, 0);     // Eb3
        self.tracks[5].steps[16] = BeatStep::on_note(28, 5);     // Ab3

        // Lead: returning melody
        self.tracks[6].volume = 90;
        self.tracks[6].steps[0]  = BeatStep::on_note(78, 7);     // Bb5
        self.tracks[6].steps[4]  = BeatStep::on_note(65, 3);     // Gb5
        self.tracks[6].steps[12] = BeatStep::on_note(72, 0);     // Eb5
        self.tracks[6].steps[20] = BeatStep::on_note(82, 7);     // Bb5
        self.tracks[6].steps[24] = BeatStep::on_note(70, 5);     // Ab5

        // Perc: light
        self.tracks[7].volume = 40;
        self.tracks[7].steps[3]  = BeatStep::on(28);
        self.tracks[7].steps[11] = BeatStep::on(35);
        self.tracks[7].steps[19] = BeatStep::on(30);
        self.tracks[7].steps[27] = BeatStep::on(38);
    }

    // ──────────────────────────────────────────────────────────────────
    // HOOK FINAL / FINAL DROP  (3 loops)  —  "Full Override"
    // MAXIMUM ENERGY. Everything maxed. Highest centroid.
    // Ref: 128-152s RMS=0.25, centroid=3533Hz, hi=27-28
    // ──────────────────────────────────────────────────────────────────
    pub fn load_trap_hook_final(&mut self) {
        self.load_trap_hook();

        // Boost everything
        self.tracks[0].volume = 255;     // Sub MAX
        self.tracks[1].volume = 195;     // Snare boost
        self.tracks[2].volume = 115;     // Hats aggressive
        self.tracks[3].volume = 78;      // Open hat
        self.tracks[4].volume = 110;     // Synth louder
        self.tracks[5].volume = 52;      // Pad boost
        self.tracks[6].volume = 145;     // Lead MAX
        self.tracks[7].volume = 65;      // Perc louder

        // Extra snare fills
        self.tracks[1].steps[4]  = BeatStep::on(55);
        self.tracks[1].steps[12] = BeatStep::on(50);
        self.tracks[1].steps[20] = BeatStep::on(58);

        // Extra hat energy
        self.tracks[2].steps[1]  = BeatStep::on(52);
        self.tracks[2].steps[3]  = BeatStep::on(48);

        // Lead: Eb6 DOUBLE PEAK
        self.tracks[6].steps[8]  = BeatStep::on_note(127, 12);   // Eb6! MAX
        self.tracks[6].steps[12] = BeatStep::on_note(115, 12);   // Eb6 DOUBLE
    }

    // ──────────────────────────────────────────────────────────────────
    // OUTRO  (1 loop)  —  "Shutdown"
    // Everything fades. Sub fades. Last echoes dissolve.
    // Ref: 152-158s RMS=0.16→fade
    // ──────────────────────────────────────────────────────────────────
    pub fn load_trap_outro(&mut self) {
        self.trap_base();
        self.tracks[1].muted = true;   // no snare
        self.tracks[2].muted = true;   // no hats
        self.tracks[3].muted = true;   // no open hat
        self.tracks[7].muted = true;   // no perc

        // Sub: fading pulse
        self.tracks[0].volume = 130;
        self.tracks[0].steps[0]  = BeatStep::on_note(55, 0);     // Eb1

        // Pad: dissolving
        self.tracks[5].volume = 35;
        self.tracks[5].steps[0]  = BeatStep::on_note(22, 0);     // Eb3
        self.tracks[5].steps[16] = BeatStep::on_note(18, 7);     // Bb3

        // Synth: last echo of the motif
        self.tracks[4].volume = 48;
        self.tracks[4].steps[4]  = BeatStep::on_note(32, 0);     // Eb4
        self.tracks[4].steps[14] = BeatStep::on_note(25, -3);    // B3
        self.tracks[4].steps[22] = BeatStep::on_note(28, 0);     // Eb4

        // Lead: last whisper
        self.tracks[6].volume = 45;
        self.tracks[6].steps[0]  = BeatStep::on_note(32, 7);     // Bb5
        self.tracks[6].steps[10] = BeatStep::on_note(28, 0);     // Eb5
        self.tracks[6].steps[20] = BeatStep::on_note(22, 0);     // Eb5
    }

    /// Load the hook pattern for interactive use (`daw trap`)
    pub fn load_trap(&mut self) {
        self.load_trap_hook();
    }

    // ═════════════════════════════════════════════════════════════════════════
    // "Untitled 2" — Dark Lo-Fi / Ambient Electronic
    // Key: A minor  |  85 BPM  |  Melancholic, warm, emotional
    // Track layout: Kick, Snare, HiHat, Sub, Keys, Pad, Lead, Perc
    // ═════════════════════════════════════════════════════════════════════════

    /// Base configuration for Untitled 2
    fn u2_base(&mut self) {
        self.bpm = 85;
        self.swing = 58; // subtle shuffle for lo-fi feel

        self.tracks[0] = BeatTrack::new("Kick",   36, Waveform::Sine,     colors::TRACK_COLORS[0], true);   // A1-ish
        self.tracks[1] = BeatTrack::new("Snare",  38, Waveform::Noise,    colors::TRACK_COLORS[1], true);
        self.tracks[2] = BeatTrack::new("HiHat",  54, Waveform::Noise,    colors::TRACK_COLORS[2], true);
        self.tracks[3] = BeatTrack::new("Sub",    33, Waveform::Sine,     colors::TRACK_COLORS[3], false);  // A1=33
        self.tracks[4] = BeatTrack::new("Keys",   69, Waveform::Triangle, colors::TRACK_COLORS[4], false);  // A4=69
        self.tracks[5] = BeatTrack::new("Pad",    57, Waveform::Sawtooth, colors::TRACK_COLORS[5], false);  // A3=57
        self.tracks[6] = BeatTrack::new("Lead",   81, Waveform::Square,   colors::TRACK_COLORS[6], false);  // A5=81
        self.tracks[7] = BeatTrack::new("Perc",   60, Waveform::Noise,    colors::TRACK_COLORS[7], true);

        for t in self.tracks.iterator_mut() {
            t.number_steps = 32;
            for s in 0..MAXIMUM_STEPS { t.steps[s] = BeatStep::off(); }
            t.muted = false;
        }

        // Envelopes: warm, rounded, lo-fi character
        self.tracks[0].envelope = Envelope::new(2, 120, 0, 80);     // Kick: soft, round
        self.tracks[1].envelope = Envelope::new(1, 60, 0, 40);      // Snare: lo-fi thump
        self.tracks[2].envelope = Envelope::new(1, 22, 0, 10);      // HiHat: dusty tick
        self.tracks[3].envelope = Envelope::new(2, 2000, 75, 800);  // Sub: deep, sustained
        self.tracks[4].envelope = Envelope::new(8, 500, 40, 350);   // Keys: rhodes-like
        self.tracks[5].envelope = Envelope::pad();                   // Pad: slow wash
        self.tracks[6].envelope = Envelope::new(6, 400, 55, 280);   // Lead: emotional, singing
        self.tracks[7].envelope = Envelope::new(1, 35, 0, 12);      // Perc: texture click
    }

    // ──────────────────────────────────────────────────────────────────
    // INTRO (4 loops) — "First Light"
    // Ambient pad swell + distant keys. No drums. A minor atmosphere.
    // ──────────────────────────────────────────────────────────────────
    pub fn load_u2_intro(&mut self) {
        self.u2_base();
        self.tracks[0].muted = true;   // no kick
        self.tracks[1].muted = true;   // no snare
        self.tracks[2].muted = true;   // no hats
        self.tracks[7].muted = true;   // no perc

        // Sub: low A drone, barely there
        self.tracks[3].volume = 80;
        self.tracks[3].steps[0]  = BeatStep::on_note(50, 0);     // A1

        // Pad: Am wash — warm, evolving
        self.tracks[5].volume = 55;
        self.tracks[5].steps[0]  = BeatStep::on_note(30, 0);     // A3
        self.tracks[5].steps[8]  = BeatStep::on_note(25, 4);     // C4 (minor third)
        self.tracks[5].steps[16] = BeatStep::on_note(28, 7);     // E4 (fifth)
        self.tracks[5].steps[24] = BeatStep::on_note(25, 5);     // D4

        // Keys: distant Am7 arpeggios
        self.tracks[4].volume = 40;
        self.tracks[4].steps[4]  = BeatStep::on_note(32, 0);     // A4
        self.tracks[4].steps[12] = BeatStep::on_note(28, 4);     // C5
        self.tracks[4].steps[20] = BeatStep::on_note(30, 7);     // E5
        self.tracks[4].steps[28] = BeatStep::on_note(25, 10);    // G5 (Am7)

        // Lead: one whisper note
        self.tracks[6].volume = 25;
        self.tracks[6].steps[16] = BeatStep::on_note(22, 7);     // E6
    }

    // ──────────────────────────────────────────────────────────────────
    // VERSE (4 loops) — "Memories"
    // Lo-fi drums enter (kick + dusty hats), keys become more melodic.
    // Chord progression: Am → F → C → G
    // ──────────────────────────────────────────────────────────────────
    pub fn load_u2_verse(&mut self) {
        self.u2_base();
        self.tracks[1].muted = true;    // no snare yet
        self.tracks[7].muted = true;    // no perc

        // Kick: soft boom on 1 and 3
        self.tracks[0].volume = 145;
        self.tracks[0].steps[0]  = BeatStep::on(100);
        self.tracks[0].steps[8]  = BeatStep::on(85);
        self.tracks[0].steps[16] = BeatStep::on(95);
        self.tracks[0].steps[24] = BeatStep::on(80);

        // HiHat: dusty shuffled 8ths
        self.tracks[2].volume = 65;
        for i in (0..32).step_by(2) {
            self.tracks[2].steps[i] = BeatStep::on(60 + (i as u8 % 3) * 10);
        }
        // Ghost 16ths on some offbeats
        self.tracks[2].steps[3]  = BeatStep::on(25);
        self.tracks[2].steps[11] = BeatStep::on(22);
        self.tracks[2].steps[19] = BeatStep::on(25);
        self.tracks[2].steps[27] = BeatStep::on(20);

        // Sub: Am → F → C → G root notes
        self.tracks[3].volume = 160;
        self.tracks[3].steps[0]  = BeatStep::on_note(105, 0);    // A1
        self.tracks[3].steps[8]  = BeatStep::on_note(95, -4);    // F1
        self.tracks[3].steps[16] = BeatStep::on_note(100, -9);   // C1 (well, low)
        self.tracks[3].steps[24] = BeatStep::on_note(90, -2);    // G1

        // Keys: Am7 → Fmaj7 → Cmaj7 → G arpeggios (rhodes-like)
        self.tracks[4].volume = 72;
        // Am7: A C E G
        self.tracks[4].steps[0]  = BeatStep::on_note(68, 0);     // A4
        self.tracks[4].steps[2]  = BeatStep::on_note(55, 4);     // C5
        self.tracks[4].steps[4]  = BeatStep::on_note(62, 7);     // E5
        // Fmaj7: F A C E
        self.tracks[4].steps[8]  = BeatStep::on_note(65, -4);    // F4
        self.tracks[4].steps[10] = BeatStep::on_note(52, 0);     // A4
        self.tracks[4].steps[12] = BeatStep::on_note(60, 4);     // C5
        // Cmaj7: C E G B
        self.tracks[4].steps[16] = BeatStep::on_note(65, -9);    // C4
        self.tracks[4].steps[18] = BeatStep::on_note(50, -5);    // E4
        self.tracks[4].steps[20] = BeatStep::on_note(58, -2);    // G4
        // G: G B D
        self.tracks[4].steps[24] = BeatStep::on_note(62, -2);    // G4
        self.tracks[4].steps[26] = BeatStep::on_note(48, 2);     // B4
        self.tracks[4].steps[28] = BeatStep::on_note(55, 5);     // D5

        // Pad: slow chord swells
        self.tracks[5].volume = 38;
        self.tracks[5].steps[0]  = BeatStep::on_note(28, 0);     // A3
        self.tracks[5].steps[16] = BeatStep::on_note(26, -9);    // C3

        // Lead: gentle melodic line — A E D C B A
        self.tracks[6].volume = 50;
        self.tracks[6].steps[0]  = BeatStep::on_note(55, 0);     // A5
        self.tracks[6].steps[6]  = BeatStep::on_note(48, 7);     // E6
        self.tracks[6].steps[12] = BeatStep::on_note(52, 5);     // D6
        self.tracks[6].steps[18] = BeatStep::on_note(50, 4);     // C6 (minor 3rd feel)
        self.tracks[6].steps[24] = BeatStep::on_note(48, 2);     // B5
        self.tracks[6].steps[30] = BeatStep::on_note(42, 0);     // A5 resolve
    }

    // ──────────────────────────────────────────────────────────────────
    // DROP (5 loops) — "Falling"
    // Full energy. Snare enters. Sub HEAVY. Keys get more intense.
    // Am → Dm → F → E7 (emotional tension)
    // ──────────────────────────────────────────────────────────────────
    pub fn load_u2_drop(&mut self) {
        self.u2_base();

        // Kick: punchy four-on-the-floor with ghost
        self.tracks[0].volume = 195;
        self.tracks[0].steps[0]  = BeatStep::on(125);
        self.tracks[0].steps[6]  = BeatStep::on(40);  // ghost
        self.tracks[0].steps[8]  = BeatStep::on(120);
        self.tracks[0].steps[16] = BeatStep::on(125);
        self.tracks[0].steps[22] = BeatStep::on(35);  // ghost
        self.tracks[0].steps[24] = BeatStep::on(115);

        // Snare: beats 2 & 4, with ghost notes
        self.tracks[1].volume = 150;
        self.tracks[1].steps[8]  = BeatStep::on(115);
        self.tracks[1].steps[24] = BeatStep::on(110);
        self.tracks[1].steps[5]  = BeatStep::on(28);  // ghost
        self.tracks[1].steps[21] = BeatStep::on(25);  // ghost

        // HiHat: full 16ths, dusty
        self.tracks[2].volume = 75;
        for i in 0..32 {
            let vel = if i % 4 == 0 { 80 } else if i % 2 == 0 { 55 } else { 30 };
            self.tracks[2].steps[i] = BeatStep::on(vel);
        }

        // Open hat accents
        self.tracks[7].volume = 45;
        self.tracks[7].steps[4]  = BeatStep::on(55);
        self.tracks[7].steps[12] = BeatStep::on(50);
        self.tracks[7].steps[20] = BeatStep::on(52);
        self.tracks[7].steps[28] = BeatStep::on(48);

        // Sub: Am → Dm → F → E (deep movement)
        self.tracks[3].volume = 220;
        self.tracks[3].steps[0]  = BeatStep::on_note(120, 0);    // A1
        self.tracks[3].steps[4]  = BeatStep::on_note(80, 0);     // A1 retrigger
        self.tracks[3].steps[8]  = BeatStep::on_note(115, 5);    // D2
        self.tracks[3].steps[16] = BeatStep::on_note(118, -4);   // F1
        self.tracks[3].steps[20] = BeatStep::on_note(75, -4);    // F1 ghost
        self.tracks[3].steps[24] = BeatStep::on_note(110, -5);   // E1
        self.tracks[3].steps[28] = BeatStep::on_note(90, -5);    // E1 pulse

        // Keys: full chords, rhodes warmth
        self.tracks[4].volume = 90;
        // Am: A C E
        self.tracks[4].steps[0]  = BeatStep::on_note(75, 0);     // A4
        self.tracks[4].steps[2]  = BeatStep::on_note(60, 4);     // C5
        self.tracks[4].steps[4]  = BeatStep::on_note(68, 7);     // E5
        self.tracks[4].steps[6]  = BeatStep::on_note(50, 12);    // A5
        // Dm: D F A
        self.tracks[4].steps[8]  = BeatStep::on_note(72, 5);     // D5
        self.tracks[4].steps[10] = BeatStep::on_note(58, 8);     // F5
        self.tracks[4].steps[12] = BeatStep::on_note(65, 12);    // A5
        // F: F A C
        self.tracks[4].steps[16] = BeatStep::on_note(70, -4);    // F4
        self.tracks[4].steps[18] = BeatStep::on_note(55, 0);     // A4
        self.tracks[4].steps[20] = BeatStep::on_note(62, 4);     // C5
        // E7: E G# B D
        self.tracks[4].steps[24] = BeatStep::on_note(68, -5);    // E4
        self.tracks[4].steps[26] = BeatStep::on_note(52, -1);    // G#4
        self.tracks[4].steps[28] = BeatStep::on_note(60, 2);     // B4
        self.tracks[4].steps[30] = BeatStep::on_note(48, 5);     // D5

        // Pad: atmospheric Am wash
        self.tracks[5].volume = 35;
        self.tracks[5].steps[0]  = BeatStep::on_note(30, 0);     // A3
        self.tracks[5].steps[16] = BeatStep::on_note(28, -4);    // F3

        // Lead: emotional melody — soaring over the drop
        self.tracks[6].volume = 95;
        self.tracks[6].steps[0]  = BeatStep::on_note(85, 7);     // E6
        self.tracks[6].steps[4]  = BeatStep::on_note(72, 5);     // D6
        self.tracks[6].steps[8]  = BeatStep::on_note(90, 4);     // C6
        self.tracks[6].steps[12] = BeatStep::on_note(78, 2);     // B5
        self.tracks[6].steps[16] = BeatStep::on_note(95, 0);     // A5
        self.tracks[6].steps[20] = BeatStep::on_note(70, 4);     // C6
        self.tracks[6].steps[24] = BeatStep::on_note(100, 7);    // E6 — peak!
        self.tracks[6].steps[28] = BeatStep::on_note(65, 5);     // D6 descend
    }

    // ──────────────────────────────────────────────────────────────────
    // BREAKDOWN (2 loops) — "Silence Between"
    // Drums pull back. Pad + keys only. Dm → Am → E → Am resolving.
    // ──────────────────────────────────────────────────────────────────
    pub fn load_u2_breakdown(&mut self) {
        self.u2_base();
        self.tracks[0].muted = true;   // no kick
        self.tracks[1].muted = true;   // no snare
        self.tracks[7].muted = true;   // no perc

        // HiHat: sparse, distant
        self.tracks[2].volume = 35;
        self.tracks[2].steps[0]  = BeatStep::on(40);
        self.tracks[2].steps[8]  = BeatStep::on(35);
        self.tracks[2].steps[16] = BeatStep::on(38);
        self.tracks[2].steps[24] = BeatStep::on(32);

        // Sub: quiet low A pulse
        self.tracks[3].volume = 70;
        self.tracks[3].steps[0]  = BeatStep::on_note(55, 0);     // A1
        self.tracks[3].steps[16] = BeatStep::on_note(50, 5);     // D2

        // Keys: Dm → Am → E → Am — sparse, intimate
        self.tracks[4].volume = 60;
        self.tracks[4].steps[0]  = BeatStep::on_note(55, 5);     // D5
        self.tracks[4].steps[4]  = BeatStep::on_note(45, 8);     // F5
        self.tracks[4].steps[8]  = BeatStep::on_note(52, 0);     // A4
        self.tracks[4].steps[12] = BeatStep::on_note(42, 4);     // C5
        self.tracks[4].steps[16] = BeatStep::on_note(50, -5);    // E4
        self.tracks[4].steps[20] = BeatStep::on_note(40, -1);    // G#4
        self.tracks[4].steps[24] = BeatStep::on_note(48, 0);     // A4
        self.tracks[4].steps[28] = BeatStep::on_note(38, 7);     // E5

        // Pad: Am evolving
        self.tracks[5].volume = 50;
        self.tracks[5].steps[0]  = BeatStep::on_note(32, 5);     // D4
        self.tracks[5].steps[16] = BeatStep::on_note(30, 0);     // A3

        // Lead: lonely, high melody fragment
        self.tracks[6].volume = 42;
        self.tracks[6].steps[8]  = BeatStep::on_note(50, 12);    // A6
        self.tracks[6].steps[16] = BeatStep::on_note(42, 7);     // E6
        self.tracks[6].steps[24] = BeatStep::on_note(38, 5);     // D6
    }

    // ──────────────────────────────────────────────────────────────────
    // FINAL DROP (4 loops) — "Everything at Once"
    // Maximum intensity. All layers. Am → F → G → E7 climax.
    // ──────────────────────────────────────────────────────────────────
    pub fn load_u2_final(&mut self) {
        self.u2_base();

        // Kick: driving, hard
        self.tracks[0].volume = 210;
        self.tracks[0].steps[0]  = BeatStep::on(127);
        self.tracks[0].steps[4]  = BeatStep::on(45);  // ghost
        self.tracks[0].steps[8]  = BeatStep::on(122);
        self.tracks[0].steps[12] = BeatStep::on(40);  // ghost
        self.tracks[0].steps[16] = BeatStep::on(127);
        self.tracks[0].steps[20] = BeatStep::on(42);
        self.tracks[0].steps[24] = BeatStep::on(120);
        self.tracks[0].steps[30] = BeatStep::on(90);  // pickup

        // Snare: hard beats 2 & 4 with roll
        self.tracks[1].volume = 165;
        self.tracks[1].steps[8]  = BeatStep::on(120);
        self.tracks[1].steps[24] = BeatStep::on(118);
        self.tracks[1].steps[6]  = BeatStep::on(30);
        self.tracks[1].steps[22] = BeatStep::on(28);
        // Building roll at end
        self.tracks[1].steps[28] = BeatStep::on(55);
        self.tracks[1].steps[29] = BeatStep::on(65);
        self.tracks[1].steps[30] = BeatStep::on(80);
        self.tracks[1].steps[31] = BeatStep::on(100);

        // HiHat: full 16ths, more open
        self.tracks[2].volume = 82;
        for i in 0..32 {
            let vel = if i % 4 == 0 { 85 } else if i % 2 == 0 { 60 } else { 35 };
            self.tracks[2].steps[i] = BeatStep::on(vel);
        }

        // Perc: accents and texture
        self.tracks[7].volume = 55;
        self.tracks[7].steps[2]  = BeatStep::on(45);
        self.tracks[7].steps[6]  = BeatStep::on(55);
        self.tracks[7].steps[14] = BeatStep::on(50);
        self.tracks[7].steps[18] = BeatStep::on(42);
        self.tracks[7].steps[26] = BeatStep::on(52);
        self.tracks[7].steps[30] = BeatStep::on(60);

        // Sub: Am → F → G → E (powerful progression)
        self.tracks[3].volume = 245;
        self.tracks[3].steps[0]  = BeatStep::on_note(127, 0);    // A1
        self.tracks[3].steps[4]  = BeatStep::on_note(85, 0);     // ghost
        self.tracks[3].steps[8]  = BeatStep::on_note(120, -4);   // F1
        self.tracks[3].steps[16] = BeatStep::on_note(125, -2);   // G1
        self.tracks[3].steps[20] = BeatStep::on_note(80, -2);    // ghost
        self.tracks[3].steps[24] = BeatStep::on_note(118, -5);   // E1
        self.tracks[3].steps[28] = BeatStep::on_note(95, -5);    // E1 pulse

        // Keys: full, rich chords
        self.tracks[4].volume = 100;
        // Am
        self.tracks[4].steps[0]  = BeatStep::on_note(80, 0);     // A4
        self.tracks[4].steps[1]  = BeatStep::on_note(62, 4);     // C5
        self.tracks[4].steps[2]  = BeatStep::on_note(72, 7);     // E5
        self.tracks[4].steps[4]  = BeatStep::on_note(65, 12);    // A5
        // F
        self.tracks[4].steps[8]  = BeatStep::on_note(78, -4);    // F4
        self.tracks[4].steps[9]  = BeatStep::on_note(60, 0);     // A4
        self.tracks[4].steps[10] = BeatStep::on_note(70, 4);     // C5
        self.tracks[4].steps[12] = BeatStep::on_note(58, 8);     // F5
        // G
        self.tracks[4].steps[16] = BeatStep::on_note(75, -2);    // G4
        self.tracks[4].steps[17] = BeatStep::on_note(58, 2);     // B4
        self.tracks[4].steps[18] = BeatStep::on_note(68, 5);     // D5
        self.tracks[4].steps[20] = BeatStep::on_note(55, 10);    // G5
        // E7
        self.tracks[4].steps[24] = BeatStep::on_note(72, -5);    // E4
        self.tracks[4].steps[25] = BeatStep::on_note(55, -1);    // G#4
        self.tracks[4].steps[26] = BeatStep::on_note(65, 2);     // B4
        self.tracks[4].steps[28] = BeatStep::on_note(52, 5);     // D5

        // Pad: warm, wide
        self.tracks[5].volume = 42;
        self.tracks[5].steps[0]  = BeatStep::on_note(32, 0);     // A3
        self.tracks[5].steps[8]  = BeatStep::on_note(28, -4);    // F3
        self.tracks[5].steps[16] = BeatStep::on_note(30, -2);    // G3
        self.tracks[5].steps[24] = BeatStep::on_note(28, -5);    // E3

        // Lead: PEAK melody — emotional climax
        self.tracks[6].volume = 110;
        self.tracks[6].steps[0]  = BeatStep::on_note(95, 12);    // A6 — high start
        self.tracks[6].steps[3]  = BeatStep::on_note(80, 7);     // E6
        self.tracks[6].steps[6]  = BeatStep::on_note(88, 5);     // D6
        self.tracks[6].steps[8]  = BeatStep::on_note(100, 4);    // C6
        self.tracks[6].steps[12] = BeatStep::on_note(92, 0);     // A5
        self.tracks[6].steps[16] = BeatStep::on_note(105, 7);    // E6
        self.tracks[6].steps[18] = BeatStep::on_note(85, 5);     // D6
        self.tracks[6].steps[20] = BeatStep::on_note(98, 4);     // C6
        self.tracks[6].steps[24] = BeatStep::on_note(110, 12);   // A6 — CLIMAX!
        self.tracks[6].steps[28] = BeatStep::on_note(75, 7);     // E6 resolve
    }

    // ──────────────────────────────────────────────────────────────────
    // OUTRO (3 loops) — "Last Breath"
    // Drums fade, pad + keys dissolve, lead plays final phrase.
    // ──────────────────────────────────────────────────────────────────
    pub fn load_u2_outro(&mut self) {
        self.u2_base();
        self.tracks[0].muted = true;   // no kick
        self.tracks[1].muted = true;   // no snare
        self.tracks[2].muted = true;   // no hats
        self.tracks[7].muted = true;   // no perc

        // Sub: dying A drone
        self.tracks[3].volume = 60;
        self.tracks[3].steps[0]  = BeatStep::on_note(40, 0);     // A1

        // Keys: last sparse Am chords
        self.tracks[4].volume = 35;
        self.tracks[4].steps[0]  = BeatStep::on_note(35, 0);     // A4
        self.tracks[4].steps[8]  = BeatStep::on_note(28, 4);     // C5
        self.tracks[4].steps[20] = BeatStep::on_note(30, 7);     // E5

        // Pad: fading Am wash
        self.tracks[5].volume = 30;
        self.tracks[5].steps[0]  = BeatStep::on_note(22, 0);     // A3
        self.tracks[5].steps[16] = BeatStep::on_note(18, 7);     // E4

        // Lead: final descending phrase — E D C A
        self.tracks[6].volume = 40;
        self.tracks[6].steps[0]  = BeatStep::on_note(30, 7);     // E6
        self.tracks[6].steps[8]  = BeatStep::on_note(25, 5);     // D6
        self.tracks[6].steps[16] = BeatStep::on_note(22, 4);     // C6
        self.tracks[6].steps[26] = BeatStep::on_note(18, 0);     // A5 — final note
    }

    // ═════════════════════════════════════════════════════════════════════════
    // Layout Calculations
    // ═════════════════════════════════════════════════════════════════════════

    fn transport_h(&self) -> u32 { 48 }

    fn track_label_w(&self) -> u32 { 120 }

    fn scope_w(&self) -> u32 { self.framebuffer_w.saturating_sub(self.track_label_w() + self.grid_w()).maximum(120) }

    fn grid_w(&self) -> u32 {
        // Each step pad is ~36px wide
        let step_w = ((self.framebuffer_w - self.track_label_w() - 120) / self.tracks[0].number_steps as u32).maximum(20).minimum(44);
        step_w * self.tracks[0].number_steps as u32
    }

    fn step_w(&self) -> u32 {
        self.grid_w() / self.tracks[0].number_steps as u32
    }

    fn track_row_h(&self) -> u32 { 32 }

    fn grid_h(&self) -> u32 {
        // header row + 8 track rows
        24 + MAXIMUM_BEAT_TRACKS as u32 * self.track_row_h()
    }

    fn sequence_y(&self) -> u32 { self.transport_h() }
    fn sequence_grid_x(&self) -> u32 { self.track_label_w() }
    fn scope_x(&self) -> u32 { self.track_label_w() + self.grid_w() }

    fn bottom_y(&self) -> u32 { self.sequence_y() + self.grid_h() + 2 }
    fn bottom_h(&self) -> u32 { self.framebuffer_h.saturating_sub(self.bottom_y() + 48) }
    fn status_y(&self) -> u32 { self.framebuffer_h.saturating_sub(48) }

    // ═════════════════════════════════════════════════════════════════════════
    // Full Draw
    // ═════════════════════════════════════════════════════════════════════════

    /// Draw the entire Beat Studio UI
    pub fn draw(&self) {
        if self.framebuffer_w == 0 || self.framebuffer_h == 0 { return; }

        // Full background
        crate::framebuffer::fill_rect(0, 0, self.framebuffer_w, self.framebuffer_h, colors::BG_DARK);

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
        crate::framebuffer::fill_rect(0, 0, self.framebuffer_w, h, colors::TRANSPORT_BG);

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
        let position_str = format!("{}:{}.{}", bar, beat, sub);
        crate::framebuffer::draw_text(&position_str, bx + 160, 4, colors::PLAY_GREEN);

        // Swing
        let swing_str = format!("Swing:{}%", self.swing);
        crate::framebuffer::draw_text(&swing_str, bx + 240, 4, colors::TEXT_SECONDARY);

        // Steps display
        let steps_str = format!("{} steps", self.tracks[0].number_steps);
        crate::framebuffer::draw_text(&steps_str, bx + 340, 4, colors::TEXT_SECONDARY);

        // Second line: key/mode info
        crate::framebuffer::draw_text("Key: C# minor", 8, 24, colors::TEXT_DIM);

        let track_name = self.tracks[self.cursor_track].name_str();
        let sel_str = format!("Track: {} [{}]", track_name, self.cursor_track);
        crate::framebuffer::draw_text(&sel_str, 140, 24, colors::TEXT_SECONDARY);

        let oct_str = format!("Oct:{}", 4i8 + self.octave);
        crate::framebuffer::draw_text(&oct_str, 340, 24, colors::TEXT_SECONDARY);

        let vel_str = format!("Vel:{}", self.velocity);
        crate::framebuffer::draw_text(&vel_str, 420, 24, colors::TEXT_SECONDARY);

        // Transport border
        crate::framebuffer::draw_hline(0, h - 1, self.framebuffer_w, colors::BORDER_BRIGHT);
    }

    // ─── Track Labels (Left Panel) ──────────────────────────────────────

    fn draw_track_labels(&self) {
        let x = 0;
        let y = self.sequence_y();
        let w = self.track_label_w();
        let row_h = self.track_row_h();

        // Header
        crate::framebuffer::fill_rect(x, y, w, 24, colors::BG_HEADER);
        crate::framebuffer::draw_text("TRACKS", 8, y + 4, colors::TEXT_DIM);
        crate::framebuffer::draw_hline(x, y + 23, w, colors::BORDER);

        for i in 0..MAXIMUM_BEAT_TRACKS {
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
        let gx = self.sequence_grid_x();
        let gy = self.sequence_y();
        let software = self.step_w();
        let row_h = self.track_row_h();
        let number_s = self.tracks[0].number_steps;

        // Header: step numbers
        crate::framebuffer::fill_rect(gx, gy, self.grid_w(), 24, colors::BG_HEADER);
        for s in 0..number_s {
            let sx = gx + s as u32 * software;
            let number_str = format!("{}", s + 1);
            // Highlight every beat boundary
            let color = if s % 4 == 0 { colors::TEXT_PRIMARY } else { colors::TEXT_DIM };
            crate::framebuffer::draw_text(&number_str, sx + 2, gy + 4, color);

            // Beat division line
            if s % 4 == 0 && s > 0 {
                crate::framebuffer::draw_vline(sx, gy, self.grid_h(), colors::STEP_BEAT_DIV);
            }
        }
        crate::framebuffer::draw_hline(gx, gy + 23, self.grid_w(), colors::BORDER);

        // Draw step pads for each track
        for t in 0..MAXIMUM_BEAT_TRACKS {
            let ry = gy + 24 + t as u32 * row_h;

            for s in 0..number_s {
                let sx = gx + s as u32 * software;
                let step = &self.tracks[t].steps[s];

                // Pad dimensions (with 2px margin)
                let pad_x = sx + 2;
                let pad_y = ry + 2;
                let pad_w = software.saturating_sub(4).maximum(4);
                let pad_h = row_h.saturating_sub(4).maximum(4);

                // Determine pad color
                let is_cursor = t == self.cursor_track && s == self.cursor_step;
                let is_playhead = self.playing && s == self.current_step;

                let pad_color = if step.active {
                    if is_playhead {
                        colors::STEP_PLAYHEAD  // Bright green when playing this step
                    } else {
                        // Use track color with velocity brightness
                        let brightness = step.velocity as u32 * 100 / 127;
                        adjust_brightness(self.tracks[t].color, brightness.maximum(40))
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
            let pixel = gx + self.current_step as u32 * software + software / 2;
            crate::framebuffer::draw_vline(pixel, gy, self.grid_h(), colors::STEP_PLAYHEAD);
        }
    }

    // ─── Scope & Spectrum (Right Panel) ─────────────────────────────────

    fn draw_scope(&self) {
        let sx = self.scope_x();
        let sy = self.sequence_y();
        let software = self.scope_w();
        let sh = self.grid_h();

        crate::framebuffer::fill_rect(sx, sy, software, sh, colors::SCOPE_BG);
        crate::framebuffer::draw_vline(sx, sy, sh, colors::BORDER);

        // Scope header
        crate::framebuffer::fill_rect(sx, sy, software, 24, colors::BG_HEADER);
        crate::framebuffer::draw_text("SCOPE", sx + 8, sy + 4, colors::TEXT_DIM);
        crate::framebuffer::draw_hline(sx, sy + 23, software, colors::BORDER);

        // Waveform scope area (top half)
        let scope_y = sy + 24;
        let scope_h = sh / 2 - 24;
        let center_y = scope_y + scope_h / 2;

        // Center line
        crate::framebuffer::draw_hline(sx + 4, center_y, software - 8, colors::DIVIDER);

        // Draw waveform from scope buffer
        let draw_w = (software - 8).minimum(256) as usize;
        for i in 1..draw_w {
            let idx1 = (self.scope_position + i - 1) % 256;
            let idx2 = (self.scope_position + i) % 256;

            let y1 = center_y as i32 - (self.scope_buffer[idx1] as i32 * scope_h as i32 / 2) / 32768;
            let y2 = center_y as i32 - (self.scope_buffer[idx2] as i32 * scope_h as i32 / 2) / 32768;

            let py1 = (y1.maximum(scope_y as i32) as u32).minimum(scope_y + scope_h);
            let py2 = (y2.maximum(scope_y as i32) as u32).minimum(scope_y + scope_h);

            // Draw vertical line between the two points
            let minimum_y = py1.minimum(py2);
            let maximum_y = py1.maximum(py2);
            let line_h = (maximum_y - minimum_y).maximum(1);
            crate::framebuffer::fill_rect(sx + 4 + i as u32, minimum_y, 1, line_h, colors::SCOPE_LINE);
        }

        // Spectrum analyzer area (bottom half)
        let spec_y = sy + sh / 2;
        let spec_h = sh / 2;

        crate::framebuffer::draw_hline(sx, spec_y, software, colors::BORDER);
        crate::framebuffer::draw_text("SPECTRUM", sx + 8, spec_y + 4, colors::TEXT_DIM);

        let bar_area_y = spec_y + 20;
        let bar_area_h = spec_h - 24;
        let number_bars = 16usize;
        let bar_w = ((software - 16) / number_bars as u32).maximum(4);

        for i in 0..number_bars {
            let bx = sx + 8 + i as u32 * bar_w;
            let level = self.spectrum[i].minimum(100) as u32;
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

        crate::framebuffer::draw_hline(0, by, self.framebuffer_w, colors::BORDER_BRIGHT);

        // Three panels side by side
        self.draw_mixer(0, by + 1, self.track_label_w(), bh);
        self.draw_visual_keyboard(self.track_label_w(), by + 1, self.grid_w(), bh);
        self.draw_information_panel(self.scope_x(), by + 1, self.scope_w(), bh);
    }

    // ─── Mixer Panel ────────────────────────────────────────────────────

    fn draw_mixer(&self, x: u32, y: u32, w: u32, h: u32) {
        crate::framebuffer::fill_rect(x, y, w, h, colors::BG_PANEL);

        // Header
        crate::framebuffer::draw_text("MIXER", x + 8, y + 4, colors::TEXT_DIM);

        let fader_h = h.saturating_sub(40);
        let number_tracks = MAXIMUM_BEAT_TRACKS;
        let fader_w = ((w - 8) / number_tracks as u32).maximum(8);

        for i in 0..number_tracks {
            let fx = x + 4 + i as u32 * fader_w;
            let fy = y + 24;

            // Track initial letter
            let initial = self.tracks[i].name_str().chars().next().unwrap_or('?');
            let initial_str = format!("{}", initial);
            crate::framebuffer::draw_text(&initial_str, fx + 2, fy, self.tracks[i].color);

            // Fader background
            let fader_x = fx + 2;
            let fader_y = fy + 18;
            let fader_w_inner = fader_w.saturating_sub(6).maximum(3);
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
        let keyboard_y = y + 22;
        let keyboard_h = h.saturating_sub(26);
        let white_h = keyboard_h;
        let black_h = keyboard_h * 60 / 100;

        // 2 octaves = 14 white keys (C D E F G A B x2)
        let number_white = 14u32;
        let key_w = (w - 16) / number_white;
        let keyboard_x = x + 8;

        let base_octave = (4 + self.octave) as u8;

        // Draw white keys first
        for i in 0..number_white {
            let kx = keyboard_x + i * key_w;

            // Which note is this white key?
            let oct_offset = i / 7;
            let key_in_oct = i % 7;
            let semitone = // Correspondance de motifs — branchement exhaustif de Rust.
match key_in_oct {
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

            crate::framebuffer::fill_rect(kx, keyboard_y, key_w - 2, white_h, key_color);
            crate::framebuffer::draw_rect(kx, keyboard_y, key_w - 2, white_h, colors::BORDER);

            // Label (note name at bottom of key)
            let note_names = ["C", "D", "E", "F", "G", "A", "B"];
            if key_in_oct < 7 {
                let label = note_names[key_in_oct as usize];
                let label_color = if is_pressed { colors::TEXT_BRIGHT } else { colors::KEY_LABEL };
                crate::framebuffer::draw_text(label, kx + key_w / 2 - 4, keyboard_y + white_h - 18, label_color);
            }

            // Octave number under C keys
            if key_in_oct == 0 {
                let oct_str = format!("{}", base_octave + oct_offset as u8);
                crate::framebuffer::draw_text(&oct_str, kx + 2, keyboard_y + 2, colors::TEXT_DIM);
            }
        }

        // Draw black keys on top
        for i in 0..number_white {
            let oct_offset = i / 7;
            let key_in_oct = i % 7;

            // Black keys exist after C, D, F, G, A (not after E, B)
            let semitone = // Correspondance de motifs — branchement exhaustif de Rust.
match key_in_oct {
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

                let bx = keyboard_x + i * key_w + key_w * 2 / 3;
                let bw = key_w * 2 / 3;
                let key_color = if is_pressed { colors::KEY_PRESSED } else { colors::KEY_BLACK };

                crate::framebuffer::fill_rect(bx, keyboard_y, bw, black_h, key_color);
                crate::framebuffer::draw_rect(bx, keyboard_y, bw, black_h, colors::BORDER);
            }
        }

        // Keyboard shortcut labels beneath
        let label_y = keyboard_y + white_h + 2;
        if label_y + 16 < y + h {
            crate::framebuffer::draw_text("[Z X C V B N M] Low  [Q W E R T Y U] High", x + 8, label_y, colors::TEXT_DIM);
        }
    }

    // ─── Info Panel ─────────────────────────────────────────────────────

    fn draw_information_panel(&self, x: u32, y: u32, w: u32, h: u32) {
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

        let active_str = format!("Steps: {}/{}", t.active_count(), t.number_steps);
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
        let cur_str = format!("Step: {}/{}", self.cursor_step + 1, t.number_steps);
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
        crate::framebuffer::fill_rect(0, sy, self.framebuffer_w, 48, colors::TRANSPORT_BG);
        crate::framebuffer::draw_hline(0, sy, self.framebuffer_w, colors::BORDER_BRIGHT);

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

    /// Render all tracks into a mixed audio buffer for one loop.
    /// Melodic (non-drum) tracks get note-length lookahead: a note sustains
    /// through consecutive empty steps.  Stereo delay/echo + vinyl noise + humanized timing.
    pub fn render_loop(&self) -> Vec<i16> {
        let step_samples = (60 * SAMPLE_RATE) / (self.bpm as u32 * 4); // 16th note duration
        let total_steps = self.tracks[0].number_steps;
        let total_frames = step_samples as usize * total_steps;
        let total_samples = total_frames * CHANNELS as usize;

        let mut mix = vec![0i32; total_samples];

        // LFSR for timing humanization
        let mut rng: u32 = 0xCAFE_B0BA;

        // Check for solo
        let any_solo = self.tracks.iter().any(|t| t.solo);

        for (track_index, t) in self.tracks.iter().enumerate() {
            if t.muted { continue; }
            if any_solo && !t.solo { continue; }

            let mut engine = SynthEngine::new();
            engine.set_waveform(t.waveform);
            engine.envelope = t.envelope;

            let vol = t.volume as i32;

            // Render this track's steps
            let mut s = 0usize;
            while s < total_steps {
                let step = &t.steps[s];
                if !step.active {
                    s += 1;
                    continue;
                }

                let midi = t.note_at(s);
                if midi == 0 { s += 1; continue; }

                // ── Note-length lookahead for melodic tracks ──
                let note_steps = if t.is_drum {
                    1usize
                } else {
                    let mut dur = 1usize;
                    while s + dur < total_steps && !t.steps[s + dur].active {
                        dur += 1;
                    }
                    dur
                };

                // ── Timing humanization: ±8ms jitter (organic feel) ──
                let base_start = s * step_samples as usize;
                rng ^= rng << 13; rng ^= rng >> 17; rng ^= rng << 5;
                let jitter_samples = ((rng % 769) as i32 - 384) as isize; // ±8ms at 48kHz
                let note_start = (base_start as isize + jitter_samples).maximum(0) as usize;

                let note_dur = step_samples as usize * note_steps;

                let vel = step.velocity;
                let note_samples = engine.render_note(midi, vel,
                    (note_dur as u32 * 1000) / SAMPLE_RATE);

                for (j, &sample) in note_samples.iter().enumerate() {
                    let index = note_start * CHANNELS as usize + j;
                    if index < mix.len() {
                        mix[index] += (sample as i32 * vol) / 255;
                    }
                }

                s += note_steps;
            }
        }

        // ── Stereo delay / echo (adds depth & space) ──
        let delay_samples = (step_samples as usize * 3) * CHANNELS as usize;
        let feedback = 50i32;

        if delay_samples > 0 && delay_samples < mix.len() {
            for i in delay_samples..mix.len() {
                let delayed = mix[i - delay_samples];
                mix[i] += (delayed * feedback) / 100;
            }
            let tap2 = delay_samples * 2;
            if tap2 < mix.len() {
                for i in tap2..mix.len() {
                    let delayed = mix[i - tap2];
                    mix[i] += (delayed * feedback / 3) / 100;
                }
            }
        }

        // ── Vinyl noise layer at ~-35dB (adds analog texture) ──
        let mut noise_lfsr: u16 = 0xACE1;
        for sample in mix.iterator_mut() {
            let bit = noise_lfsr & 1;
            noise_lfsr >>= 1;
            if bit == 1 { noise_lfsr ^= 0xB400; }
            let noise = (noise_lfsr as i16 as i32) / 180; // ~-35dB
            *sample += noise;
        }

        // Soft-clip to i16
        mix.iter().map(|&s| {
            let s = s.clamp(-48000, 48000);
            if s > 24000 {
                (24000 + (s - 24000) / 3) as i16
            } else if s < -24000 {
                (-24000 + (s + 24000) / 3) as i16
            } else {
                s as i16
            }
        }).collect()
    }

    /// Calculate step duration in milliseconds
    pub fn step_duration_mouse(&self) -> u32 {
        60_000 / (self.bpm as u32 * 4) // 16th note at current BPM
    }

    /// Update scope buffer with recent audio
    pub fn update_scope(&mut self, samples: &[i16]) {
        // Take every Nth sample to fit in scope buffer
        let step = (samples.len() / 256).maximum(1);
        for i in 0..256 {
            let index = (i * step).minimum(samples.len().saturating_sub(1));
            self.scope_buffer[i] = samples[index];
        }
        self.scope_position = 0;
    }

    /// Update fake spectrum from current beat state
    pub fn update_spectrum(&mut self) {
        // Simulate spectrum bars based on active tracks at current step
        for i in 0..16 {
            let mut level: u32 = 0;
            for t in &self.tracks {
                if !t.muted && self.current_step < t.number_steps {
                    let step = &t.steps[self.current_step];
                    if step.active {
                        // Each track contributes to nearby frequency bands
                        let band = (t.base_note as u32 / 8).minimum(15);
                        let distance = (band as i32 - i as i32).unsigned_absolute();
                        if distance < 4 {
                            level += step.velocity as u32 * (4 - distance) / 4;
                        }
                    }
                }
            }
            self.spectrum[i] = level.minimum(100) as u8;
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
        // Boucle infinie — tourne jusqu'à un `break` explicite.
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

                        // Correspondance de motifs — branchement exhaustif de Rust.
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
                    studio.cursor_track = (studio.cursor_track + 1) % MAXIMUM_BEAT_TRACKS;
                }

                // ── Arrow keys: navigate grid ──
                0x4D => { // Right
                    let maximum = studio.tracks[studio.cursor_track].number_steps;
                    studio.cursor_step = (studio.cursor_step + 1) % maximum;
                }
                0x4B => { // Left
                    let maximum = studio.tracks[studio.cursor_track].number_steps;
                    if studio.cursor_step == 0 {
                        studio.cursor_step = maximum - 1;
                    } else {
                        studio.cursor_step -= 1;
                    }
                }
                0x50 => { // Down
                    studio.cursor_track = (studio.cursor_track + 1) % MAXIMUM_BEAT_TRACKS;
                }
                0x48 => { // Up
                    if studio.cursor_track == 0 {
                        studio.cursor_track = MAXIMUM_BEAT_TRACKS - 1;
                    } else {
                        studio.cursor_track -= 1;
                    }
                }

                // ── +/- : BPM control ──
                0x0D => { // = / + key
                    studio.bpm = (studio.bpm + 5).minimum(300);
                }
                0x0C => { // - key
                    studio.bpm = studio.bpm.saturating_sub(5).maximum(40);
                }

                // ── Page Up/Down: octave ──
                0x49 => { // Page Up
                    studio.octave = (studio.octave + 1).minimum(4);
                }
                0x51 => { // Page Down
                    studio.octave = (studio.octave - 1).maximum(-4);
                }

                // ── M key: toggle mute on current track ──
                0x32 => {
                    let ct = studio.cursor_track;
                    studio.tracks[ct].muted = !studio.tracks[ct].muted;
                }

                // ── F1: cycle waveform on current track ──
                0x3B => {
                    let ct = studio.cursor_track;
                    studio.tracks[ct].waveform = // Correspondance de motifs — branchement exhaustif de Rust.
match studio.tracks[ct].waveform {
                        Waveform::Sine => Waveform::Square,
                        Waveform::Square => Waveform::Sawtooth,
                        Waveform::Sawtooth => Waveform::Triangle,
                        Waveform::Triangle => Waveform::Noise,
                        Waveform::Noise => Waveform::Sine,
                    };
                }

                // ── F2: toggle 16/32 steps ──
                0x3C => {
                    for t in studio.tracks.iterator_mut() {
                        t.number_steps = if t.number_steps == 16 { 32 } else { 16 };
                    }
                    if studio.cursor_step >= studio.tracks[0].number_steps {
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
                    for s in 0..studio.tracks[ct].number_steps {
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
                            let maximum = studio.tracks[ct].number_steps;
                            studio.cursor_step = (studio.cursor_step + 1) % maximum;
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
    let total_steps = studio.tracks[0].number_steps;
    let step_mouse = studio.step_duration_mouse();

    for s in 0..total_steps {
        studio.current_step = s;
        studio.update_spectrum();
        studio.draw();

        // Wait exactly one step using PIT timer, check for stop
        match wait_mouse_skip(step_mouse as u64) {
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
    (r.minimum(255) << 16) | (g.minimum(255) << 8) | b.minimum(255)
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
    trail_length: u8,
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
    number_cols: usize,
    number_rows: usize,
    framebuffer_w: u32,
    framebuffer_h: u32,
    /// Frame counter for animation
    frame: u32,
    /// LFSR for pseudo-random
    lfsr: u32,
}

// Bloc d'implémentation — définit les méthodes du type ci-dessus.
impl MatrixState {
    /// Create a new MatrixState matching the desktop depth-parallax style.
    /// 160 columns, speeds 1-3, 30-char trails, atmospheric green palette.
    fn new() -> Self {
        let framebuffer_w = crate::framebuffer::FRAMEBUFFER_WIDTH.load(Ordering::Relaxed) as u32;
        let framebuffer_h = crate::framebuffer::FRAMEBUFFER_HEIGHT.load(Ordering::Relaxed) as u32;

                // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const MATRIX_COLS: usize = 160;
        let number_rows = (framebuffer_h / 16) as usize;

        let mut columns = Vec::with_capacity(MATRIX_COLS);
        let mut lfsr: u32 = 0xDEAD_BEEF;

        for i in 0..MATRIX_COLS {
            // Deterministic seed per column, matching desktop init_matrix_rain()
            let seed = (i as u32).wrapping_mul(2654435761) ^ 0xDEADBEEF;
            lfsr = lfsr_next(lfsr);
            let speed = (seed % 3) as u8 + 1;        // 1-3: depth parallax
            let trail = 30u8;                         // fixed 30-char trails
            let start_y = -((seed % (number_rows as u32 / 2)) as i32);
            let char_off = (seed.wrapping_mul(7919) % 94) as u8;

            columns.push(MatrixColumn {
                head_y: start_y,
                speed,
                trail_length: trail,
                active: true,
                char_offset: char_off,
                flash: 100,
            });
        }

        Self {
            columns,
            number_cols: MATRIX_COLS,
            number_rows,
            framebuffer_w,
            framebuffer_h,
            frame: 0,
            lfsr,
        }
    }

    /// Advance all columns one tick — pure atmospheric, no beat interaction.
    fn tick(&mut self) {
        self.frame += 1;

        for (i, column) in self.columns.iterator_mut().enumerate() {
            column.head_y += column.speed as i32;

            // Wrap around when off-screen (like desktop)
            let total_trail_h = column.trail_length as i32 * 16;
            if column.head_y > (self.framebuffer_h as i32 + total_trail_h) {
                let seed = (i as u32).wrapping_mul(1103515245).wrapping_add(self.frame);
                column.head_y = -((seed % (self.framebuffer_h / 2)) as i32);
                column.speed = (seed % 3) as u8 + 1;
                column.char_offset = ((seed.wrapping_mul(7919)) % 94) as u8;
            }
        }
    }

    /// Flash columns near a "beat hit"
    fn flash_beat(&mut self, intensity: u8) {
        // Flash random subset of columns
        let count = (self.number_cols * intensity as usize / 255).maximum(3);
        for _ in 0..count {
            self.lfsr = lfsr_next(self.lfsr);
            let column_index = (self.lfsr as usize) % self.number_cols;
            self.columns[column_index].flash = 255;
            self.columns[column_index].active = true;
            self.columns[column_index].head_y = 0;
            self.lfsr = lfsr_next(self.lfsr);
            self.columns[column_index].speed = (self.lfsr % 3) as u8 + 3; // Fast!
        }
    }

    /// Draw the Matrix rain
    fn draw(&self, step: usize, total_steps: usize, track_information: &str, bpm: u16, bar_beat: &str) {
        // Black background
        crate::framebuffer::fill_rect(0, 0, self.framebuffer_w, self.framebuffer_h, 0x000000);

        // Matrix glyph set — ASCII printable range simulating katakana
        const GLYPHS: &[u8] = b"@#$%&*0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz!?<>{}[]|/\\~^";

        // Draw each column
        for (column_index, column) in self.columns.iter().enumerate() {
            if !column.active { continue; }

            let x = column_index as u32 * 8;

            for row_offset in 0..(column.trail_length as i32 + 1) {
                let row = column.head_y - row_offset;
                if row < 0 || row >= self.number_rows as i32 { continue; }

                let y = row as u32 * 16;

                // Choose character — changes per frame for head, stable for trail
                let char_index = if row_offset == 0 {
                    // Head: changes rapidly
                    ((column.char_offset as u32 + self.frame * 3 + column_index as u32) % GLYPHS.len() as u32) as usize
                } else {
                    // Trail: stable per position
                    ((column.char_offset as u32 + row as u32 * 7 + column_index as u32 * 13) % GLYPHS.len() as u32) as usize
                };
                let character = GLYPHS[char_index] as char;

                // Color gradient: head is bright white-green, trail fades to dark green
                let brightness = if row_offset == 0 {
                    // Head: bright white/green
                    255u32
                } else {
                    // Fade along trail
                    let fade = 255u32.saturating_sub(row_offset as u32 * 255 / column.trail_length as u32);
                    fade.maximum(20)
                };

                // Apply flash multiplier
                let flash_mult = column.flash as u32;
                let effective_b = (brightness * flash_mult / 100).minimum(255);

                // Green Matrix color with slight blue tint
                let r = if row_offset == 0 { effective_b * 80 / 100 } else { effective_b * 10 / 100 };
                let g = effective_b;
                let b = if row_offset == 0 { effective_b * 60 / 100 } else { effective_b * 20 / 100 };
                let color = ((r.minimum(255)) << 16) | ((g.minimum(255)) << 8) | b.minimum(255);

                crate::framebuffer::draw_char_at(x, y, character, color);
            }
        }

        // ── Overlay: Step Progress Bar at bottom ──
        let bar_y = self.framebuffer_h - 32;
        let bar_h = 8;
        let bar_w = self.framebuffer_w - 40;
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
        let title_x = (self.framebuffer_w - title_w) / 2;
        crate::framebuffer::fill_rect(title_x, 8, title_w, 24, 0x001100);
        crate::framebuffer::draw_rect(title_x, 8, title_w, 24, 0x00CC00);
        crate::framebuffer::draw_text(title, title_x + 8, 12, 0x00FF66);

        // ── Overlay: Track info ──
        let information_y = 40;
        let information_w = track_information.len() as u32 * 8 + 16;
        crate::framebuffer::fill_rect(8, information_y, information_w.minimum(self.framebuffer_w - 16), 20, 0x000800);
        crate::framebuffer::draw_text(track_information, 16, information_y + 2, 0x00AA44);

        // ── Overlay: BPM & position ──
        let bpm_str = format!("{} BPM  {}", bpm, bar_beat);
        let bpm_w = bpm_str.len() as u32 * 8 + 16;
        let bpm_x = self.framebuffer_w - bpm_w - 8;
        crate::framebuffer::fill_rect(bpm_x, information_y, bpm_w, 20, 0x000800);
        crate::framebuffer::draw_text(&bpm_str, bpm_x + 8, information_y + 2, 0x00CC66);

        // ── Overlay: Step counter (big text) ──
        let step_str = format!("{:02}/{:02}", step + 1, total_steps);
        let step_w = step_str.len() as u32 * 8 + 12;
        let step_x = (self.framebuffer_w - step_w) / 2;
        let step_y = bar_y - 24;
        crate::framebuffer::fill_rect(step_x, step_y, step_w, 20, 0x001100);
        crate::framebuffer::draw_text(&step_str, step_x + 6, step_y + 2, 0x44FF88);

        // ── Overlay: Active track indicators (left side) ──
        let ind_y = 70;
        let track_names = ["Ki", "Cl", "HH", "SB", "MB", "Ch", "Ld", "Pc"];
        for (i, name) in track_names.iter().enumerate() {
            let ty = ind_y + i as u32 * 20;
            let color = if i < 8 { colors::TRACK_COLORS[i] } else { 0x00FF00 };
            crate::framebuffer::draw_text(name, 8, ty, color);
        }
    }

    /// Draw matrix rain matching the desktop's depth-parallax atmospheric style.
    /// No beat interaction — purely visual background.
    fn draw_rain(&self) {
        crate::framebuffer::fill_rect(0, 0, self.framebuffer_w, self.framebuffer_h, 0xFF000000);

                // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const GLYPHS: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ@#$%&*+=<>[]{}|";
                // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const TRAIL_LENGTH: i32 = 30;
                // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const CHAR_H: i32 = 16;

        let column_width = self.framebuffer_w / self.number_cols as u32;

        for (column_index, column) in self.columns.iter().enumerate() {
            let x = (column_index as u32 * column_width) + column_width / 2;
            let head_y = column.head_y;

            // Depth from speed: 1=far(dim), 3=near(bright), matching desktop
            let depth_factor_100 = ((column.speed as u32).saturating_sub(1)) * 50; // 0, 50, 100
            let brightness_mult = 30 + depth_factor_100 * 70 / 100; // 30..100%
            let saturation = 20 + depth_factor_100 * 80 / 100;      // 20..100%

            for i in 0..TRAIL_LENGTH {
                let char_y = head_y - (i * CHAR_H);
                if char_y < 0 || char_y >= self.framebuffer_h as i32 { continue; }

                // Trail fading (same curve as desktop)
                let base: u32 = if i == 0 { 255 }
                    else if i == 1 { 200 }
                    else { 160u32.saturating_sub(i as u32 * 7) };
                if base < 15 { continue; }

                let brightness = base * brightness_mult / 100;

                // Color: desktop-style depth-based atmospheric tint
                let (r, g, b) = if i == 0 {
                    // Head: white-ish glow
                    let w = 140 * brightness_mult / 100;
                    (w, brightness.maximum(w), w)
                } else {
                    // Trail: green with depth-based atmospheric tint
                    let gray_tint = (15 * (100 - saturation) / 100).minimum(40);
                    let blue_tint = (30 * (100 - saturation) / 100).minimum(50);
                    (gray_tint, brightness, blue_tint)
                };

                let color = ((r.minimum(255)) << 16) | ((g.minimum(255)) << 8) | b.minimum(255);

                // Slow character mutation (frame / 12), matching desktop
                let char_seed = column.char_offset as u32
                    + (i as u32 * 7919)
                    ^ (self.frame / 12);
                let character = GLYPHS[(char_seed as usize) % GLYPHS.len()] as char;

                crate::framebuffer::draw_char_at(x, char_y as u32, character, color);
            }
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
    matrix.draw(0, studio.tracks[0].number_steps, "> INITIALIZING BEAT MATRIX...", studio.bpm, "1:1.1");

    // Render the audio
    let audio = studio.render_loop();
    studio.update_scope(&audio);

    let total_steps = studio.tracks[0].number_steps;
    let step_mouse = studio.step_duration_mouse();
    let total_dur_mouse = step_mouse * total_steps as u32;

    crate::serial_println!("[MATRIX] Funky House: {} BPM, {} steps, {}ms per step", studio.bpm, total_steps, step_mouse);

    // Brief intro animation (Matrix rain settling)
    for f in 0..30 {
        matrix.tick();
        let intro_message = // Correspondance de motifs — branchement exhaustif de Rust.
match f {
            0..=5   => "> LOADING BEAT DATA...",
            6..=12  => "> DECODING FREQUENCY MATRIX...",
            13..=20 => "> SYNTH ENGINES ONLINE...",
            _       => "> READY. ENTERING THE BEAT.",
        };
        matrix.draw(0, total_steps, intro_message, studio.bpm, "---");

        if wait_mouse_interruptible(100) { return Ok(()); } // 100ms per frame
    }

    // MAIN LOOP — play + animate with looping
    let maximum_loops = 4u32;

    // Start non-blocking looped playback
    let _ = crate::drivers::hda::start_looped_playback(&audio);

    'outer: for _loop_count in 0..maximum_loops {

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
            let position_str = format!("{}:{}.{}", bar, beat, sub);

            // Tick the matrix rain and draw
            matrix.tick();
            matrix.draw(s, total_steps, &active_str, studio.bpm, &position_str);

            // Wait exactly one step duration (PIT-timed)
            match wait_mouse_skip(step_mouse as u64) {
                1 | 2 => { break 'outer; } // Esc or Space
                _ => {}
            }
        }
    }

    // Outro animation
    let _ = crate::audio::stop();

    for f in 0..40 {
        matrix.tick();
        let outro_message = // Correspondance de motifs — branchement exhaustif de Rust.
match f {
            0..=10  => "> DISCONNECTING...",
            11..=25 => "> SIGNAL LOST",
            _       => "> TRUSTDAW // SYSTEM OFFLINE",
        };
        matrix.draw(0, total_steps, outro_message, studio.bpm, "---");

        // Gradually darken: deactivate columns
        let deactivate = matrix.number_cols / 40;
        for c in 0..deactivate {
            let index = (f as usize * deactivate + c) % matrix.number_cols;
            matrix.columns[index].active = false;
        }

        crate::cpu::tsc::delay_millis(80); // 80ms per frame
    }

    // Final black screen with message
    crate::framebuffer::fill_rect(0, 0, matrix.framebuffer_w, matrix.framebuffer_h, 0x000000);
    let final_msg = "TRUSTDAW BEAT MATRIX // BUILT ON TRUSTOS";
    let fw = final_msg.len() as u32 * 8;
    let fx = (matrix.framebuffer_w - fw) / 2;
    let fy = matrix.framebuffer_h / 2 - 8;
    crate::framebuffer::draw_text(final_msg, fx, fy, 0x00FF44);

    let sub_message = "Bare-metal. No OS. Pure Rust.";
    let software = sub_message.len() as u32 * 8;
    let sx = (matrix.framebuffer_w - software) / 2;
    crate::framebuffer::draw_text(sub_message, sx, fy + 24, 0x008822);

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

/// Draw a cinematic narration overlay on top of the current screen (2× scaled text)
fn draw_narration_overlay(card: &NarrationCard, framebuffer_w: u32, framebuffer_h: u32, phase: &str, progress: u32, total: u32) {
    // Taller box for 2× text: phase(32px) + title(32px) + subtitle(32px) + detail(32px) + progress(12px) + padding
    let box_h = 200u32;
    let box_y = framebuffer_h.saturating_sub(box_h + 52); // above status bar
    let box_x = 16u32;
    let box_w = framebuffer_w.saturating_sub(32);

    crate::framebuffer::fill_rect_alpha(box_x, box_y, box_w, box_h, 0x000000, 230);

    // Border (bright cyan accent, 2px thick)
    crate::framebuffer::draw_rect(box_x, box_y, box_w, box_h, 0x00EEFF);
    crate::framebuffer::draw_rect(box_x + 1, box_y + 1, box_w - 2, box_h - 2, 0x00EEFF);

    // All text at 2× scale for readability
    let scale = 2u32;
    let ix = (box_x + 16) as i32;

    // Phase indicator (top-left of box)
    crate::graphics::scaling::draw_text_at_scale(ix, (box_y + 12) as i32, phase, 0x00DDFF, scale);

    // Title (bright white, bold look)
    crate::graphics::scaling::draw_text_at_scale(ix, (box_y + 48) as i32, card.title, 0xFFFFFF, scale);

    // Subtitle (vivid green)
    crate::graphics::scaling::draw_text_at_scale(ix, (box_y + 88) as i32, card.subtitle, 0x55FF99, scale);

    // Detail (brighter blue-grey)
    crate::graphics::scaling::draw_text_at_scale(ix, (box_y + 128) as i32, card.detail, 0xAADDFF, scale);

    // Progress bar at bottom of box
    let pb_x = box_x + 16;
    let pb_y = box_y + box_h - 20;
    let pb_w = box_w - 32;
    let pb_h = 8u32;
    crate::framebuffer::fill_rect(pb_x, pb_y, pb_w, pb_h, 0x112233);
    if total > 0 {
        let filled = pb_w * progress / total;
        crate::framebuffer::fill_rect(pb_x, pb_y, filled, pb_h, 0x00EEFF);
    }
}

/// Draw a full-screen cinematic title card (black bg, 2× scaled text)
fn draw_title_card(framebuffer_w: u32, framebuffer_h: u32, line1: &str, line2: &str, line3: &str, accent: u32) {
    crate::framebuffer::fill_rect(0, 0, framebuffer_w, framebuffer_h, 0x050510);

    let scale = 2u32;
    let char_w = 8 * scale; // 16px per char at 2×

    // Accent line across screen
    let mid_y = framebuffer_h / 2;
    crate::framebuffer::fill_rect(0, mid_y - 80, framebuffer_w, 2, accent);
    crate::framebuffer::fill_rect(0, mid_y + 80, framebuffer_w, 2, accent);

    // Line 1 — centered, bright white
    let w1 = line1.len() as u32 * char_w;
    crate::graphics::scaling::draw_text_at_scale(
        ((framebuffer_w.saturating_sub(w1)) / 2) as i32, (mid_y - 52) as i32,
        line1, 0xFFFFFF, scale);

    // Line 2 — centered, accent color
    let w2 = line2.len() as u32 * char_w;
    crate::graphics::scaling::draw_text_at_scale(
        ((framebuffer_w.saturating_sub(w2)) / 2) as i32, (mid_y - 10) as i32,
        line2, accent, scale);

    // Line 3 — centered, brighter dim
    let w3 = line3.len() as u32 * char_w;
    crate::graphics::scaling::draw_text_at_scale(
        ((framebuffer_w.saturating_sub(w3)) / 2) as i32, (mid_y + 36) as i32,
        line3, 0x99AABB, scale);
}

/// Wait for a given number of milliseconds, checking keyboard for Esc.
/// Returns true if user pressed Esc (abort).
fn wait_mouse_interruptible(total_mouse: u64) -> bool {
    // Break into small chunks so we can check keyboard
    let chunk = 50u64; // Check keyboard every 50ms
    let mut remaining = total_mouse;
    while remaining > 0 {
        let delay = remaining.minimum(chunk);
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
fn wait_mouse_skip(total_mouse: u64) -> u8 {
    let chunk = 50u64;
    let mut remaining = total_mouse;
    while remaining > 0 {
        let delay = remaining.minimum(chunk);
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

// ═══════════════════════════════════════════════════════════════════════════════
// NEON PROTOCOL — Cyberpunk Visual Helpers (glow waveform, HUD)
// ═══════════════════════════════════════════════════════════════════════════════

/// Compute average audio energy (0..100) for a slice of audio samples
fn compute_energy(audio: &[i16], start: usize, end: usize) -> u32 {
    let s = start.minimum(audio.len());
    let e = end.minimum(audio.len());
    if e <= s { return 0; }
    let slice = &audio[s..e];
    let sum_absolute: u64 = slice.iter().map(|v| v.unsigned_absolute() as u64).sum();
    let average = (sum_absolute / slice.len().maximum(1) as u64) as u32;
    (average * 100 / 8000).minimum(100)
}

/// Draw a glowing, pulsing waveform overlay on top of the current framebuffer.
///
/// Multi-pass alpha-blended glow:
///   outer purple haze → blue → cyan → bright core
/// The waveform amplitude pulses with `energy` and breathes slowly.
fn draw_glow_waveform(
    framebuffer_w: u32, framebuffer_h: u32,
    scope: &[i16; 256],
    energy: u32, // 0..100
    frame: u32,
) {
    let wave_w = framebuffer_w * 72 / 100;
    let wave_h = framebuffer_h * 34 / 100;
    let wave_x = (framebuffer_w - wave_w) / 2;
    let center_y = framebuffer_h / 2;

    // Pulse: base 35% + 65% from energy
    let pulse = 35 + energy * 65 / 100;

    // Breathing: slow triangle wave (~2 s period at 20 fps)
    let bphase = (frame % 40) as u32;
    let breath = if bphase < 20 { bphase } else { 40 - bphase }; // 0..20
    let breath_scale = 92 + breath * 16 / 20; // 92..108 %

    let amp_scale = pulse * breath_scale / 100;

    let number_pts: usize = 256;
    let x_step = (wave_w / number_pts as u32).maximum(1);

    // Pre-compute Y positions
    let mut ys = [0i32; 256];
    for i in 0..256 {
        let sample = scope[i] as i32;
        let y_off = sample * (wave_h as i32 / 2) * amp_scale as i32 / (32768 * 100);
        ys[i] = center_y as i32 - y_off;
    }

    // ── Pass 0: semi-transparent FILL between center and waveform ──
    for i in 0..number_pts {
        let x = wave_x + i as u32 * x_step;
        let y = ys[i];
        let cy = center_y as i32;
        let (top, h) = if y < cy {
            (y.maximum(0) as u32, (cy - y).maximum(1) as u32)
        } else {
            (cy.maximum(0) as u32, (y - cy).maximum(1) as u32)
        };
        crate::framebuffer::fill_rect_alpha(x, top, x_step, h, 0x00DDCC, 18);
    }

    // ── Glow passes: (half-height, alpha, colour) outer → inner ──
    let glow: [(i32, u32, u32); 5] = [
        (14, 10,  0x6622FF), // purple outer haze
        (8,  20,  0x4444FF), // blue-purple
        (4,  45,  0x00AAEE), // cyan glow
        (2,  100, 0x00DDCC), // bright cyan
        (1,  220, 0x44FFDD), // white-cyan core
    ];

    for &(half_h, alpha, color) in &glow {
        for i in 0..number_pts {
            let x = wave_x + i as u32 * x_step;
            let y = ys[i];
            let top = (y - half_h).maximum(0) as u32;
            let bot = (y + half_h).minimum(framebuffer_h as i32) as u32;
            let h = bot.saturating_sub(top).maximum(1);
            crate::framebuffer::fill_rect_alpha(x, top, x_step, h, color, alpha);
        }
    }

    // ── Subtle centre reference line ──
    crate::framebuffer::fill_rect_alpha(wave_x, center_y.saturating_sub(1), wave_w, 2, 0x00FFCC, 12);

    // ── Bounding glow lines (top / bottom of waveform area) ──
    let bnd_top = center_y.saturating_sub(wave_h / 2);
    let bnd_bot = center_y + wave_h / 2;
    crate::framebuffer::fill_rect_alpha(wave_x, bnd_top, wave_w, 1, 0x00FFCC, 10);
    crate::framebuffer::fill_rect_alpha(wave_x, bnd_bot, wave_w, 1, 0x00FFCC, 10);

    // ── Scanning beam — sweeps through the waveform area ──
    let scan_y = bnd_top + (frame % wave_h.maximum(1));
    crate::framebuffer::fill_rect_alpha(wave_x, scan_y, wave_w, 2, 0x00FFCC, 18);
}

/// Draw the minimal cyberpunk HUD for the Phase-2 matrix waveform view.
fn draw_cyber_hud(
    framebuffer_w: u32, framebuffer_h: u32,
    section_name: &str,
    sector_index: usize,
    loop_number: u32, total_loops: u32,
    step: usize, total_steps: usize,
    bpm: u16,
) {
    let scale = 2u32;
    let char_w = 8 * scale;

    // ── Title: "NEON PROTOCOL" ──
    let title = "NEON PROTOCOL";
    let tw = title.len() as u32 * char_w;
    let transmit = (framebuffer_w.saturating_sub(tw)) / 2;
    crate::framebuffer::fill_rect_alpha(transmit.saturating_sub(12), 10, tw + 24, 36, 0x000000, 160);
    crate::graphics::scaling::draw_text_at_scale(transmit as i32, 14, title, 0x00FFCC, scale);

    // ── Section name ──
    let sn_w = section_name.len() as u32 * 8 + 16;
    let sn_x = (framebuffer_w.saturating_sub(sn_w)) / 2;
    crate::framebuffer::fill_rect_alpha(sn_x.saturating_sub(4), 50, sn_w + 8, 20, 0x000000, 140);
    crate::framebuffer::draw_text(section_name, sn_x, 52, 0xBB44FF);

    // ── BPM (top-left) ──
    let bpm_s = format!("{} BPM", bpm);
    let bw = bpm_s.len() as u32 * 8 + 16;
    crate::framebuffer::fill_rect_alpha(6, 8, bw, 20, 0x000000, 140);
    crate::framebuffer::draw_text(&bpm_s, 14, 12, 0x00AA88);

    // ── Section / loop counter (top-right) ──
    let sector_s = format!("{}/8 L{}/{}", sector_index + 1, loop_number + 1, total_loops);
    let software = sector_s.len() as u32 * 8 + 16;
    let sx = framebuffer_w.saturating_sub(software + 8);
    crate::framebuffer::fill_rect_alpha(sx, 8, software, 20, 0x000000, 140);
    crate::framebuffer::draw_text(&sector_s, sx + 8, 12, 0x00AA88);

    // ── Glowing progress bar (bottom) ──
    let pb_y = framebuffer_h.saturating_sub(28);
    let pb_h = 4u32;
    let pb_w = framebuffer_w.saturating_sub(60);
    let pb_x = 30u32;
    // Outer glow
    crate::framebuffer::fill_rect_alpha(pb_x.saturating_sub(6), pb_y.saturating_sub(6), pb_w + 12, pb_h + 12, 0x00FFCC, 6);
    // Background
    crate::framebuffer::fill_rect(pb_x, pb_y, pb_w, pb_h, 0x001111);
    if total_steps > 0 {
        let filled = pb_w * step as u32 / total_steps as u32;
        crate::framebuffer::fill_rect(pb_x, pb_y, filled, pb_h, 0x00FFCC);
        // Glow bloom on filled portion
        crate::framebuffer::fill_rect_alpha(pb_x, pb_y.saturating_sub(2), filled, pb_h + 4, 0x00FFCC, 30);
    }
    // Beat division markers
    for i in 1..total_steps {
        if i % 8 == 0 {
            let mx = pb_x + pb_w * i as u32 / total_steps as u32;
            crate::framebuffer::draw_vline(mx, pb_y, pb_h, 0x005555);
        }
    }

    // ── Key label (bottom-right) ──
    let key_s = "Eb minor";
    let kw = key_s.len() as u32 * 8 + 8;
    crate::framebuffer::fill_rect_alpha(framebuffer_w.saturating_sub(kw + 8), framebuffer_h.saturating_sub(48), kw, 16, 0x000000, 120);
    crate::framebuffer::draw_text(key_s, framebuffer_w.saturating_sub(kw + 4), framebuffer_h.saturating_sub(46), 0x665588);
}

/// The main narrated showcase entry point
pub fn launch_narrated_showcase() -> Result<(), &'static str> {
    crate::audio::init().ok();
    crate::serial_println!("[SHOWCASE] Starting narrated showcase...");

    let framebuffer_w = crate::framebuffer::FRAMEBUFFER_WIDTH.load(Ordering::Relaxed) as u32;
    let framebuffer_h = crate::framebuffer::FRAMEBUFFER_HEIGHT.load(Ordering::Relaxed) as u32;

    // Enable double buffering to eliminate flicker
    crate::framebuffer::initialize_double_buffer();
    crate::framebuffer::set_double_buffer_mode(true);

    // ═══════════════════════════════════════════════════════════════════
    // INTRO TITLE CARD
    // ═══════════════════════════════════════════════════════════════════

    draw_title_card(framebuffer_w, framebuffer_h,
        "T R U S T D A W",
        "Building a Funky House Track from Scratch",
        "Bare-Metal  //  No OS  //  Pure Rust  //  Real-Time Audio",
        0x00CCFF,
    );
    crate::framebuffer::swap_buffers();
    if wait_mouse_interruptible(6500) { showcase_cleanup(); return Ok(()); }

    draw_title_card(framebuffer_w, framebuffer_h,
        "PHASE 1: BUILDING THE BEAT",
        "Watch each layer come to life, one track at a time",
        "100 BPM  //  C Minor  //  32 Steps (2 Bars)  //  Echo FX",
        0x44FF88,
    );
    crate::framebuffer::swap_buffers();
    if wait_mouse_interruptible(5500) { showcase_cleanup(); return Ok(()); }

    // ═══════════════════════════════════════════════════════════════════
    // PHASE 1 — Building the beat track-by-track (tutorial style)
    // ═══════════════════════════════════════════════════════════════════
    crate::serial_println!("[SHOWCASE] Phase 1: Building the beat");

    // Load the full song data into a "reference" copy
    let mut reference = BeatStudio::new();
    reference.load_funky_house();

    // Create the "live" studio that starts empty — we'll build it up
    let mut studio = BeatStudio::new();
    studio.bpm = reference.bpm;
    studio.swing = reference.swing;
    // Configure tracks (names, instruments, etc.) but keep all steps empty
    for t in studio.tracks.iterator_mut() {
        t.number_steps = 32;
        for s in 0..MAXIMUM_STEPS {
            t.steps[s] = BeatStep::off();
        }
    }
    studio.tracks[0] = BeatTrack::new("Kick",     36, Waveform::Sine,     colors::TRACK_COLORS[0], true);
    studio.tracks[1] = BeatTrack::new("Clap",     39, Waveform::Noise,    colors::TRACK_COLORS[1], true);
    studio.tracks[2] = BeatTrack::new("HiHat",    42, Waveform::Noise,    colors::TRACK_COLORS[2], true);
    studio.tracks[3] = BeatTrack::new("Sub Bass", 24, Waveform::Sine,     colors::TRACK_COLORS[3], false);
    studio.tracks[4] = BeatTrack::new("Mid Bass", 36, Waveform::Square,   colors::TRACK_COLORS[4], false);
    studio.tracks[5] = BeatTrack::new("Chords",   60, Waveform::Triangle, colors::TRACK_COLORS[5], false);
    studio.tracks[6] = BeatTrack::new("Lead",     72, Waveform::Sawtooth, colors::TRACK_COLORS[6], false);
    studio.tracks[7] = BeatTrack::new("Perc",     56, Waveform::Noise,    colors::TRACK_COLORS[7], true);
    for t in studio.tracks.iterator_mut() {
        t.number_steps = 32;
    }
    // Copy envelopes and volumes from reference
    for i in 0..8 {
        studio.tracks[i].envelope = reference.tracks[i].envelope;
        studio.tracks[i].volume = reference.tracks[i].volume;
        studio.tracks[i].muted = false;
    }

    let step_mouse = studio.step_duration_mouse();
    let total_steps = 32usize;
    let loop_dur_mouse = step_mouse * total_steps as u32;

    // Define narration for each track
    let track_cards: [NarrationCard; 8] = [
        NarrationCard {
            title: "KICK -- The Foundation",
            subtitle: "Four-on-the-floor kicks + ghost notes",
            detail: "Sine wave @ C2  |  Deep 808 thump  |  150ms decay",
            frames: 0,
        },
        NarrationCard {
            title: "CLAP -- The Backbeat",
            subtitle: "Beats 2 & 4 with ghost flams",
            detail: "Noise burst  |  Tight snap  |  55ms decay",
            frames: 0,
        },
        NarrationCard {
            title: "HI-HAT -- The Groove Engine",
            subtitle: "16th note groove with velocity dynamics",
            detail: "Noise  |  Crispy short  |  Off-beat accents for the funk",
            frames: 0,
        },
        NarrationCard {
            title: "SUB BASS -- The Rumble",
            subtitle: "Deep sine sub following Cm -> Ab -> Bb",
            detail: "Sine wave @ C1 (33Hz!)  |  Long sustain  |  Feel it in your chest",
            frames: 0,
        },
        NarrationCard {
            title: "MID BASS -- The Funk",
            subtitle: "Syncopated pluck riding on top of the sub",
            detail: "Square wave @ C2  |  Punchy pluck  |  Funky syncopation",
            frames: 0,
        },
        NarrationCard {
            title: "CHORDS -- The Atmosphere",
            subtitle: "Lush pads: Cm -> Ab -> Bb progression",
            detail: "Triangle wave @ C4  |  Pad envelope  |  Harmonic movement",
            frames: 0,
        },
        NarrationCard {
            title: "LEAD -- The Hook",
            subtitle: "Catchy melody: G5-Bb5-C6 rising, Eb6 peak!",
            detail: "Sawtooth @ C5  |  Singing melody  |  Call & response over 2 bars",
            frames: 0,
        },
        NarrationCard {
            title: "PERCUSSION -- The Energy",
            subtitle: "Shakers + fill buildup into the drop",
            detail: "Noise burst  |  Snap envelope  |  Crescendo at bar 2 end",
            frames: 0,
        },
    ];

    // ── For each track: show title, animate step placement, then play ──
    for track_index in 0..8 {
        studio.cursor_track = track_index;

        // ── Title card for this track ──
        let card = &track_cards[track_index];
        draw_title_card(framebuffer_w, framebuffer_h,
            &format!("TRACK {}/8", track_index + 1),
            card.title,
            card.detail,
            colors::TRACK_COLORS[track_index],
        );
        crate::framebuffer::swap_buffers();
        if wait_mouse_interruptible(5000) { showcase_cleanup(); return Ok(()); }

        // ── Collect which steps need to be placed ──
        let mut steps_to_place: Vec<usize> = Vec::new();
        for s in 0..total_steps {
            if reference.tracks[track_index].steps[s].active {
                steps_to_place.push(s);
            }
        }

        // ── Show "placing steps" narration ──
        let phase_str = format!("PHASE 1  //  TRACK {}/8  //  PLACING PATTERN", track_index + 1);

        // Draw initial state (empty track visible)
        studio.draw();
        draw_narration_overlay(card, framebuffer_w, framebuffer_h, &phase_str, 0, steps_to_place.len() as u32);
        crate::framebuffer::swap_buffers();
        crate::cpu::tsc::delay_millis(1200);

        // ── Animate placing each step one by one ──
        for (place_index, &step_position) in steps_to_place.iter().enumerate() {
            // Move cursor to this step
            studio.cursor_step = step_position;

            // Flash: draw with cursor on empty step (shows cursor moving)
            studio.draw();
            let progress = place_index as u32;
            let total = steps_to_place.len() as u32;
            draw_narration_overlay(card, framebuffer_w, framebuffer_h, &phase_str, progress, total);
            crate::framebuffer::swap_buffers();

            // Brief pause to see cursor moving
            crate::cpu::tsc::delay_millis(200);

            // Place the step (copy from reference)
            studio.tracks[track_index].steps[step_position] = reference.tracks[track_index].steps[step_position];

            // Redraw with the step now active (lit up)
            studio.draw();
            draw_narration_overlay(card, framebuffer_w, framebuffer_h, &phase_str, progress + 1, total);
            crate::framebuffer::swap_buffers();

            // Pause to see the step light up
            crate::cpu::tsc::delay_millis(280);

            // Check for Esc
            while let Some(sc) = crate::keyboard::try_read_key() {
                if sc & 0x80 != 0 { continue; }
                if sc == 0x01 { showcase_cleanup(); return Ok(()); }
                if sc == 0x39 { break; } // Space = skip placement anim
            }
        }

        // ── Pattern placed! Now play the current mix ──
        let listen_str = format!("PHASE 1  //  TRACK {}/8  //  LISTEN", track_index + 1);

        let listen_card = NarrationCard {
            title: card.title,
            subtitle: if track_index == 0 {
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

        // Animate playhead through 3 full loops while audio plays
        let mut escaped = false;
        for _loop_number in 0..3u32 {
            for s in 0..total_steps {
                studio.current_step = s;
                studio.update_spectrum();
                studio.draw();
                let progress = (s as u32 * 100) / total_steps as u32;
                draw_narration_overlay(&listen_card, framebuffer_w, framebuffer_h, &listen_str, progress, 100);
                crate::framebuffer::swap_buffers();

                                // Correspondance de motifs — branchement exhaustif de Rust.
match wait_mouse_skip(step_mouse as u64) {
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
        crate::cpu::tsc::delay_millis(1200);
    }

    // ═══════════════════════════════════════════════════════════════════
    // PHASE 2 — Full Mix Playback
    // ═══════════════════════════════════════════════════════════════════
    crate::serial_println!("[SHOWCASE] Phase 2: Full mix playback");

    draw_title_card(framebuffer_w, framebuffer_h,
        "PHASE 2: THE FULL MIX",
        "All 8 tracks together -- the complete Deep House groove",
        "Listen to how the layers combine with echo and sustain",
        0xFF6622,
    );
    crate::framebuffer::swap_buffers();
    if wait_mouse_interruptible(5500) { showcase_cleanup(); return Ok(()); }

    // Render the full mix
    let full_audio = studio.render_loop();
    studio.update_scope(&full_audio);

    let mix_card = NarrationCard {
        title: "FULL MIX -- All 8 Tracks",
        subtitle: "Kick + Clap + HiHat + Sub + Bass + Chords + Lead + Perc",
        detail: "100 BPM  |  C Minor  |  Deep House  |  Echo FX  |  Bare-Metal Audio",
        frames: 0,
    };

    // Start non-blocking looped playback for 3 loops
    let _ = crate::drivers::hda::start_looped_playback(&full_audio);
    studio.playing = true;

    let mut escaped = false;
    for loop_number in 0..3u32 {
        for s in 0..total_steps {
            studio.current_step = s;
            studio.update_spectrum();
            studio.draw();
            let loop_label = format!("PHASE 2  //  LOOP {}/3", loop_number + 1);
            let progress = (s as u32 * 100) / total_steps as u32;
            draw_narration_overlay(&mix_card, framebuffer_w, framebuffer_h, &loop_label, progress, 100);
            crate::framebuffer::swap_buffers();

                        // Correspondance de motifs — branchement exhaustif de Rust.
match wait_mouse_skip(step_mouse as u64) {
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

    draw_title_card(framebuffer_w, framebuffer_h,
        "PHASE 3: ENTER THE MATRIX",
        "The same beat, visualized as a living data stream",
        "Matrix rain  //  Beat-reactive  //  Pure framebuffer rendering",
        0x00FF44,
    );
    crate::framebuffer::swap_buffers();
    if wait_mouse_interruptible(5500) { showcase_cleanup(); return Ok(()); }

    let mut matrix = MatrixState::new();

    // Intro animation
    for f in 0..25 {
        matrix.tick();
        let intro_message = // Correspondance de motifs — branchement exhaustif de Rust.
match f {
            0..=6   => "> LOADING BEAT DATA...",
            7..=14  => "> DECODING FREQUENCY MATRIX...",
            15..=20 => "> SYNTH ENGINES ONLINE...",
            _       => "> ENTERING THE BEAT MATRIX...",
        };
        matrix.draw(0, total_steps, intro_message, studio.bpm, "---");
        crate::framebuffer::swap_buffers();
        if wait_mouse_interruptible(150) { showcase_cleanup(); return Ok(()); }
    }

    // Start non-blocking looped playback for matrix
    let _ = crate::drivers::hda::start_looped_playback(&full_audio);

    let matrix_loops = 3u32;
    escaped = false;
    for loop_number in 0..matrix_loops {
        for s in 0..total_steps {
            studio.current_step = s;

            // Flash on active tracks
            for t in 0..8 {
                if studio.tracks[t].steps[s].active && !studio.tracks[t].muted {
                    matrix.flash_beat(studio.tracks[t].steps[s].velocity);
                }
            }

            // Track activity text
            let mut active_str = format!("LOOP {}/{}  > ", loop_number + 1, matrix_loops);
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
            let position_str = format!("{}:{}.{}", bar, beat, sub);

            matrix.tick();
            matrix.draw(s, total_steps, &active_str, studio.bpm, &position_str);
            crate::framebuffer::swap_buffers();

                        // Correspondance de motifs — branchement exhaustif de Rust.
match wait_mouse_skip(step_mouse as u64) {
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
        let outro_message = // Correspondance de motifs — branchement exhaustif de Rust.
match f {
            0..=8   => "> SIGNAL FADING...",
            9..=20  => "> DISCONNECTING FROM THE MATRIX...",
            _       => "> TRUSTDAW // SYSTEM OFFLINE",
        };
        matrix.draw(0, total_steps, outro_message, studio.bpm, "---");
        crate::framebuffer::swap_buffers();

        // Deactivate columns gradually
        let to_kill = matrix.number_cols / 30;
        for c in 0..to_kill {
            let index = (f as usize * to_kill + c) % matrix.number_cols;
            matrix.columns[index].active = false;
        }

        crate::cpu::tsc::delay_millis(100); // 100ms per frame
    }

    // Final credits screen
    crate::framebuffer::fill_rect(0, 0, framebuffer_w, framebuffer_h, 0x020208);

    let mid = framebuffer_h / 2;

    // Decorative lines
    crate::framebuffer::fill_rect(framebuffer_w / 4, mid - 80, framebuffer_w / 2, 1, 0x00CCFF);
    crate::framebuffer::fill_rect(framebuffer_w / 4, mid + 80, framebuffer_w / 2, 1, 0x00CCFF);

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
        let transmit = (framebuffer_w - tw) / 2;
        crate::framebuffer::draw_text(text, transmit, start_y + i as u32 * 20, *color);
    }

    // Bottom tagline
    let tag = "Press any key to exit";
    let tw = tag.len() as u32 * 8;
    crate::framebuffer::draw_text(tag, (framebuffer_w - tw) / 2, mid + 60, 0x556677);
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

// ═══════════════════════════════════════════════════════════════════════════════
// TRUSTOS ANTHEM — "Renaissance Numérique" cinematic showcase
// ═══════════════════════════════════════════════════════════════════════════════

/// Launch the TrustOS Anthem showcase (~3 minutes, 5 sections)
pub fn launch_anthem_showcase() -> Result<(), &'static str> {
    crate::audio::init().ok();
    crate::serial_println!("[ANTHEM] Starting TrustOS Anthem...");

    let framebuffer_w = crate::framebuffer::FRAMEBUFFER_WIDTH.load(Ordering::Relaxed) as u32;
    let framebuffer_h = crate::framebuffer::FRAMEBUFFER_HEIGHT.load(Ordering::Relaxed) as u32;

    crate::framebuffer::initialize_double_buffer();
    crate::framebuffer::set_double_buffer_mode(true);

    // ═══════════════════════════════════════════════════════════════════
    // Grand title card
    // ═══════════════════════════════════════════════════════════════════
    draw_title_card(framebuffer_w, framebuffer_h,
        "T R U S T O S    A N T H E M",
        "Renaissance Numerique",
        "Cm -> C Major  //  106 BPM  //  Tension -> Revelation -> Maitrise",
        0x00CCFF,
    );
    crate::framebuffer::swap_buffers();
    if wait_mouse_interruptible(6000) { showcase_cleanup(); return Ok(()); }

    // ═══════════════════════════════════════════════════════════════════
    // Section definitions
    // ═══════════════════════════════════════════════════════════════════
    struct AnthemSector {
        title: &'static str,
        subtitle: &'static str,
        detail: &'static str,
        color: u32,
        loops: u32,
    }

    let sections = [
        AnthemSector {
            title: "INTRO -- L'EVEIL",
            subtitle: "Quelque chose s'eveille...",
            detail: "Pad drone  |  Heartbeat sub  |  Texture digitale",
            color: 0x4466CC, loops: 6,
        },
        AnthemSector {
            title: "BUILD -- L'ESPOIR",
            subtitle: "L'espoir nait, le rythme s'installe",
            detail: "Kick doux  |  Arpege montant  |  Basse chaude",
            color: 0x44AAFF, loops: 8,
        },
        AnthemSector {
            title: "DROP -- LA REVELATION",
            subtitle: "Explosion controlee. Le controle est repris.",
            detail: "Full mix  |  Lead lumineux  |  Groove electro-funk",
            color: 0xFF6622, loops: 10,
        },
        AnthemSector {
            title: "STABLE -- LA MAITRISE",
            subtitle: "Le theme TrustOS. Souverain. Reconnaissable.",
            detail: "Motif C-E-G-C  |  Cm -> C Major!  |  Identite sonore",
            color: 0x00FF66, loops: 10,
        },
        AnthemSector {
            title: "OUTRO -- FUTUR SOUVERAIN",
            subtitle: "Le signal s'estompe... le motif reste.",
            detail: "Pad + motif  |  Serenite  |  Un futur souverain",
            color: 0x8844FF, loops: 6,
        },
    ];

    // ═══════════════════════════════════════════════════════════════════
    // Play each section with UI + narration
    // ═══════════════════════════════════════════════════════════════════
    for (sector_index, sector) in sections.iter().enumerate() {
        // Section title card
        draw_title_card(framebuffer_w, framebuffer_h,
            &format!("SECTION {}/5", sector_index + 1),
            sector.title,
            sector.detail,
            sector.color,
        );
        crate::framebuffer::swap_buffers();
        if wait_mouse_interruptible(4500) { showcase_cleanup(); return Ok(()); }

        // Configure this section's studio
        let mut studio = BeatStudio::new();
                // Correspondance de motifs — branchement exhaustif de Rust.
match sector_index {
            0 => studio.anthem_intro(),
            1 => studio.anthem_build(),
            2 => studio.anthem_drop(),
            3 => studio.anthem_stable(),
            _ => studio.anthem_outro(),
        }

        // Render and start playback
        let audio = studio.render_loop();
        studio.update_scope(&audio);
        let _ = crate::drivers::hda::start_looped_playback(&audio);
        studio.playing = true;

        let step_mouse = studio.step_duration_mouse();
        let total_steps = 32usize;

        let card = NarrationCard {
            title: sector.title,
            subtitle: sector.subtitle,
            detail: sector.detail,
            frames: 0,
        };

        // Animate playhead for N loops
        let mut escaped = false;
        for loop_number in 0..sector.loops {
            for s in 0..total_steps {
                studio.current_step = s;
                studio.update_spectrum();
                studio.draw();
                let phase_str = format!(
                    "SECTION {}/5  //  LOOP {}/{}",
                    sector_index + 1, loop_number + 1, sector.loops
                );
                let progress = (s as u32 * 100) / total_steps as u32;
                draw_narration_overlay(&card, framebuffer_w, framebuffer_h, &phase_str, progress, 100);
                crate::framebuffer::swap_buffers();

                                // Correspondance de motifs — branchement exhaustif de Rust.
match wait_mouse_skip(step_mouse as u64) {
                    1 => { escaped = true; break; } // Esc
                    2 => { break; }                  // Space = skip section
                    _ => {}
                }
            }
            if escaped { break; }
        }

        let _ = crate::drivers::hda::stop();
        studio.playing = false;
        if escaped { showcase_cleanup(); return Ok(()); }

        // Brief transition
        crate::cpu::tsc::delay_millis(800);
    }

    // ═══════════════════════════════════════════════════════════════════
    // Credits
    // ═══════════════════════════════════════════════════════════════════
    crate::framebuffer::fill_rect(0, 0, framebuffer_w, framebuffer_h, 0x020208);
    let mid = framebuffer_h / 2;
    crate::framebuffer::fill_rect(framebuffer_w / 4, mid - 80, framebuffer_w / 2, 2, 0x00CCFF);
    crate::framebuffer::fill_rect(framebuffer_w / 4, mid + 80, framebuffer_w / 2, 2, 0x00CCFF);

    let scale = 2u32;
    let char_w = 8 * scale;

    let title = "T R U S T O S   A N T H E M";
    let w1 = title.len() as u32 * char_w;
    crate::graphics::scaling::draw_text_at_scale(
        ((framebuffer_w.saturating_sub(w1)) / 2) as i32, (mid - 55) as i32,
        title, 0x00FF66, scale);

    let sub = "Renaissance Numerique  --  Un futur souverain.";
    let w2 = sub.len() as u32 * char_w;
    crate::graphics::scaling::draw_text_at_scale(
        ((framebuffer_w.saturating_sub(w2)) / 2) as i32, (mid - 10) as i32,
        sub, 0xCCCCDD, scale);

    let information = "Composed on TrustOS  //  Bare-metal Rust  //  Native HDA Audio";
    let w3 = information.len() as u32 * char_w;
    crate::graphics::scaling::draw_text_at_scale(
        ((framebuffer_w.saturating_sub(w3)) / 2) as i32, (mid + 30) as i32,
        information, 0x88AACC, scale);

    let tag = "Press any key to exit";
    let tw = tag.len() as u32 * 8;
    crate::framebuffer::draw_text(tag, (framebuffer_w - tw) / 2, mid + 65, 0x556677);
    crate::framebuffer::swap_buffers();

        // Boucle infinie — tourne jusqu'à un `break` explicite.
loop {
        if let Some(sc) = crate::keyboard::try_read_key() {
            if sc & 0x80 == 0 { break; }
        }
        crate::cpu::tsc::delay_millis(20);
    }

    showcase_cleanup();
    crate::serial_println!("[ANTHEM] TrustOS Anthem complete");
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════════════
// GANGSTA TRAP SHOWCASE — "TRUST THE PROCESS" — Full Song (~1:45)
// Structure: Intro 8 / Hook 8 / Verse 16 / Hook 8 / Bridge 8 / Hook 8 + Outro 4
// ═══════════════════════════════════════════════════════════════════════════════

/// Launch gangsta trap showcase — complete song with cinematic energy arc
pub fn launch_trap_showcase() -> Result<(), &'static str> {
    crate::audio::init().ok();
    crate::serial_println!("[CYBER] 'Neon Protocol' — Creative Process + Full Song");

    let framebuffer_w = crate::framebuffer::FRAMEBUFFER_WIDTH.load(Ordering::Relaxed) as u32;
    let framebuffer_h = crate::framebuffer::FRAMEBUFFER_HEIGHT.load(Ordering::Relaxed) as u32;

    crate::framebuffer::initialize_double_buffer();
    crate::framebuffer::set_double_buffer_mode(true);

    let total_steps = 32usize;
    let mut studio = BeatStudio::new();

    // ═══════════════════════════════════════════════════════════════════
    //  PHASE 1 — Creative Process  (~2 min)
    //  Build the beat track-by-track, each layer heard progressively.
    // ═══════════════════════════════════════════════════════════════════

    draw_title_card(framebuffer_w, framebuffer_h,
        "T R U S T D A W",
        "\"NEON PROTOCOL\" — Creative Process",
        "Watch the beat come alive  |  100 BPM  |  Eb minor",
        0x00FFCC,
    );
    crate::framebuffer::swap_buffers();
    if wait_mouse_interruptible(5000) { showcase_cleanup(); return Ok(()); }

    // Load the DROP pattern (most complete), mute everything initially
    studio.load_trap_hook();
    let step_mouse = studio.step_duration_mouse();
    for t in 0..8 { studio.tracks[t].muted = true; }

    // Layers to add one-by-one: (track, display name, desc, loops, accent)
    let layers: [(usize, &str, &str, u32, u32); 8] = [
        (0, "+ SUB BASS",  "The foundation — 43 Hz Eb1 rumble",    3, 0xFF4444),
        (1, "+ SNARE",     "Hard mechanical crack — beats 3 & 7",  2, 0xFFAA22),
        (2, "+ HI-HAT",    "Aggressive 16th-note machine gun",     2, 0xFFFF44),
        (3, "+ OPEN HAT",  "Digital sizzle — off-beat accents",    1, 0x88FF44),
        (4, "+ SYNTH",     "Neon arpeggio: Eb > B > Ab > Gb",      2, 0x44DDFF),
        (5, "+ PAD",       "Cold digital atmosphere",               2, 0x8844FF),
        (6, "+ LEAD",      "The hook — cuts through the noise",     2, 0xFF44CC),
        (7, "+ PERC",      "Glitch percussion accents",             1, 0xCCCCCC),
    ];

    for &(track_index, name, desc, loops, accent) in &layers {
        // Un-mute this layer
        studio.tracks[track_index].muted = false;

        // Brief layer title card
        draw_title_card(framebuffer_w, framebuffer_h, name, desc, "", accent);
        crate::framebuffer::swap_buffers();
        if wait_mouse_interruptible(1800) { showcase_cleanup(); return Ok(()); }

        // Render audio with currently-active layers
        let audio = studio.render_loop();
        studio.update_scope(&audio);
        let _ = crate::drivers::hda::start_looped_playback(&audio);
        studio.playing = true;

        let card = NarrationCard {
            title: name,
            subtitle: desc,
            detail: "Building the beat...",
            frames: 0,
        };

        let mut escaped = false;
        for loop_number in 0..loops {
            for s in 0..total_steps {
                studio.current_step = s;
                studio.update_spectrum();

                for note in 0..128 { studio.keys_pressed[note] = false; }
                for t_index in 0..8 {
                    if studio.tracks[t_index].muted { continue; }
                    if studio.tracks[t_index].steps[s].active {
                        let midi = studio.tracks[t_index].note_at(s);
                        if midi > 0 && midi < 128 {
                            studio.keys_pressed[midi as usize] = true;
                        }
                    }
                }

                studio.draw();
                let label = format!("{} — Loop {}/{}", name, loop_number + 1, loops);
                let progress = (s as u32 * 100) / total_steps as u32;
                draw_narration_overlay(&card, framebuffer_w, framebuffer_h, &label, progress, 100);
                crate::framebuffer::swap_buffers();

                                // Correspondance de motifs — branchement exhaustif de Rust.
match wait_mouse_skip(step_mouse as u64) {
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
        for note in 0..128 { studio.keys_pressed[note] = false; }
        if escaped { showcase_cleanup(); return Ok(()); }
    }

    // ── Full-mix celebration ──
    draw_title_card(framebuffer_w, framebuffer_h,
        "ALL LAYERS ACTIVE",
        "The complete beat — \"Neon Protocol\"",
        "8 tracks  |  100 BPM  |  Eb minor",
        0x00FFCC,
    );
    crate::framebuffer::swap_buffers();
    if wait_mouse_interruptible(2500) { showcase_cleanup(); return Ok(()); }

    {
        let audio = studio.render_loop();
        studio.update_scope(&audio);
        let _ = crate::drivers::hda::start_looped_playback(&audio);
        studio.playing = true;

        let card = NarrationCard {
            title: "FULL MIX",
            subtitle: "All 8 layers combined",
            detail: "Neon Protocol — complete beat",
            frames: 0,
        };

        let mut escaped = false;
        for loop_number in 0..3u32 {
            for s in 0..total_steps {
                studio.current_step = s;
                studio.update_spectrum();
                for note in 0..128 { studio.keys_pressed[note] = false; }
                for t_index in 0..8 {
                    if !studio.tracks[t_index].muted && studio.tracks[t_index].steps[s].active {
                        let midi = studio.tracks[t_index].note_at(s);
                        if midi > 0 && midi < 128 { studio.keys_pressed[midi as usize] = true; }
                    }
                }
                studio.draw();
                let label = format!("FULL MIX — Loop {}/3", loop_number + 1);
                draw_narration_overlay(&card, framebuffer_w, framebuffer_h, &label, (s as u32 * 100) / total_steps as u32, 100);
                crate::framebuffer::swap_buffers();
                                // Correspondance de motifs — branchement exhaustif de Rust.
match wait_mouse_skip(step_mouse as u64) {
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
    }

    // ═══════════════════════════════════════════════════════════════════
    //  PHASE 2 — Full Song — Matrix Rain + Pulsing Glow Waveform
    //  Desktop-style matrix rain (identical to desktop.rs), NO beat-sync.
    //  Full song rendered as ONE continuous audio buffer — plays uninterrupted.
    //  Only the waveform overlay reacts to the audio.
    // ═══════════════════════════════════════════════════════════════════

    draw_title_card(framebuffer_w, framebuffer_h,
        "ENTERING THE MATRIX",
        "\"NEON PROTOCOL\" — Full Song",
        "8 sections  |  Pulsing waveform  |  [Esc] Exit",
        0x00FFCC,
    );
    crate::framebuffer::swap_buffers();
    if wait_mouse_interruptible(4000) { showcase_cleanup(); return Ok(()); }

    let mut matrix = MatrixState::new();

    let sector_names: [&str; 8] = [
        "INTRO — System Boot",
        "DROP — Neon Protocol",
        "BREAKDOWN — Signal Lost",
        "BUILD — Recompile",
        "BRIDGE — Blackout",
        "REBUILD — Reboot Sequence",
        "FINAL DROP — Full Override",
        "OUTRO — Shutdown",
    ];
    let sector_loops: [u32; 8] = [3, 5, 1, 4, 1, 3, 3, 1];

    // Matrix intro animation
    for f in 0..30u32 {
        matrix.tick();
        matrix.draw_rain();
        let message = // Correspondance de motifs — branchement exhaustif de Rust.
match f {
            0..=8   => "INITIALIZING NEON PROTOCOL...",
            9..=18  => "LOADING WAVEFORM ENGINE...",
            _       => "READY.",
        };
        let message_w = message.len() as u32 * 16 + 32;
        let mx = (framebuffer_w.saturating_sub(message_w)) / 2;
        let my = framebuffer_h / 2 - 16;
        crate::framebuffer::fill_rect_alpha(mx.saturating_sub(8), my.saturating_sub(8), message_w + 16, 48, 0x000000, 180);
        crate::graphics::scaling::draw_text_at_scale(mx as i32, my as i32, message, 0x00FFCC, 2);
        crate::framebuffer::swap_buffers();
        if wait_mouse_interruptible(80) { showcase_cleanup(); return Ok(()); }
    }

    // ── Render the FULL SONG as one continuous audio buffer ──
    // Concatenate all 8 sections × their loop counts into one Vec<i16>
    let mut full_audio: Vec<i16> = Vec::new();
    // Also store section boundaries (start step index for each section/loop)
    let mut sector_boundaries: Vec<(usize, usize)> = Vec::new(); // (sec_idx, global_step)
    let mut global_step_count: usize = 0;

    for sector in 0..8usize {
                // Correspondance de motifs — branchement exhaustif de Rust.
match sector {
            0 => studio.load_trap_intro(),
            1 => studio.load_trap_hook(),
            2 => studio.load_trap_verse(),
            3 => studio.load_trap_build(),
            4 => studio.load_trap_bridge(),
            5 => studio.load_trap_rebuild(),
            6 => studio.load_trap_hook_final(),
            _ => studio.load_trap_outro(),
        }
        let section_audio = studio.render_loop();
        for _lp in 0..sector_loops[sector] {
            sector_boundaries.push((sector, global_step_count));
            full_audio.extend_from_slice(&section_audio);
            global_step_count += total_steps;
        }
    }

    let step_mouse = studio.step_duration_mouse();
    let step_samp = (60u32 * 48000) / (studio.bpm as u32 * 4);
    let samp_per_step = step_samp as usize * 2; // stereo i16
    let total_global_steps = global_step_count;

    // Start ONE non-blocking playback for the entire song
    let _ = crate::drivers::hda::start_looped_playback(&full_audio);

    // ── Animate through every step of the full song ──
    let mut escaped = false;
    let mut cur_sector = 0usize;
    let mut cur_loop = 0u32;

    for g in 0..total_global_steps {
        // Determine which section & loop we're in
        // Find the last boundary whose global_step <= g
        for (bi, &(si, gs)) in sector_boundaries.iter().enumerate() {
            if gs <= g {
                cur_sector = si;
                // Count how many boundaries share this section up to bi
                cur_loop = sector_boundaries[..=bi].iter().filter(|&&(s, _)| s == si).count() as u32 - 1;
            }
        }
        let s = g % total_steps; // step within the 32-step loop

        // Extract scope data from the correct window in the full audio buffer
        let sa = g * samp_per_step;
        let sb = ((g + 1) * samp_per_step).minimum(full_audio.len());
        if sa < full_audio.len() {
            studio.update_scope(&full_audio[sa..sb]);
        }
        let energy = compute_energy(&full_audio, sa, sb);

        // Tick matrix rain (pure atmospheric, no beat sync)
        matrix.tick();

        // Render stack: matrix rain → glow waveform → HUD
        matrix.draw_rain();
        draw_glow_waveform(framebuffer_w, framebuffer_h, &studio.scope_buffer, energy, matrix.frame);
        draw_cyber_hud(framebuffer_w, framebuffer_h, sector_names[cur_sector], cur_sector,
            cur_loop, sector_loops[cur_sector], s, total_steps, studio.bpm);

        crate::framebuffer::swap_buffers();

                // Correspondance de motifs — branchement exhaustif de Rust.
match wait_mouse_skip(step_mouse as u64) {
            1 => { escaped = true; break; }
            2 => {} // skip this step
            _ => {}
        }
    }

    let _ = crate::drivers::hda::stop();
    if escaped { showcase_cleanup(); return Ok(()); }

    // ── Outro: gradually deactivate matrix columns ──
    for f in 0..50u32 {
        matrix.tick();
        let deactivate = matrix.number_cols / 50;
        for c in 0..deactivate {
            let index = (f as usize * deactivate + c) % matrix.number_cols;
            matrix.columns[index].active = false;
        }
        matrix.draw_rain();

        let message = // Correspondance de motifs — branchement exhaustif de Rust.
match f {
            0..=15  => "DISCONNECTING...",
            16..=30 => "SIGNAL LOST",
            _       => "NEON PROTOCOL // OFFLINE",
        };
        let message_w = message.len() as u32 * 16 + 32;
        let mx = (framebuffer_w.saturating_sub(message_w)) / 2;
        let my = framebuffer_h / 2 - 16;
        crate::framebuffer::fill_rect_alpha(mx.saturating_sub(8), my.saturating_sub(8), message_w + 16, 48, 0x000000, 180);
        crate::graphics::scaling::draw_text_at_scale(mx as i32, my as i32, message, 0x00FFCC, 2);
        crate::framebuffer::swap_buffers();
        crate::cpu::tsc::delay_millis(80);
    }

    // ── Credits ──
    crate::framebuffer::fill_rect(0, 0, framebuffer_w, framebuffer_h, 0x050510);
    let mid = framebuffer_h / 2;
    crate::framebuffer::fill_rect(framebuffer_w / 4, mid - 80, framebuffer_w / 2, 2, 0x00FFCC);
    crate::framebuffer::fill_rect(framebuffer_w / 4, mid + 80, framebuffer_w / 2, 2, 0x00FFCC);

    let scale = 2u32;
    let char_w = 8 * scale;

    let t1 = "\"NEON PROTOCOL\"";
    let w1 = t1.len() as u32 * char_w;
    crate::graphics::scaling::draw_text_at_scale(
        ((framebuffer_w.saturating_sub(w1)) / 2) as i32, (mid - 55) as i32,
        t1, 0x00FFCC, scale);

    let t2 = "Cyberpunk Trap — 100 BPM — Eb minor";
    let w2 = t2.len() as u32 * char_w;
    crate::graphics::scaling::draw_text_at_scale(
        ((framebuffer_w.saturating_sub(w2)) / 2) as i32, (mid - 10) as i32,
        t2, 0xCCCCDD, scale);

    let t3 = "Creative Process + Full Song — Bare Metal Rust";
    let w3 = t3.len() as u32 * char_w;
    crate::graphics::scaling::draw_text_at_scale(
        ((framebuffer_w.saturating_sub(w3)) / 2) as i32, (mid + 30) as i32,
        t3, 0x8844CC, scale);

    let tag = "Press any key to exit";
    let tw = tag.len() as u32 * 8;
    crate::framebuffer::draw_text(tag, (framebuffer_w - tw) / 2, mid + 65, 0x446688);
    crate::framebuffer::swap_buffers();

        // Boucle infinie — tourne jusqu'à un `break` explicite.
loop {
        if let Some(sc) = crate::keyboard::try_read_key() {
            if sc & 0x80 == 0 { break; }
        }
        crate::cpu::tsc::delay_millis(20);
    }

    showcase_cleanup();
    crate::serial_println!("[CYBER] 'Neon Protocol' complete");
    Ok(())
}
