//! Keyboard-to-MIDI Mapping for TrustDAW
//!
//! Maps PS/2 keyboard scancodes to MIDI note events.
//! Uses a 2-row piano layout:
//!   Row 1 (Z-M):  White keys C3-B3
//!   Row 2 (Q-P):  White keys C4-B4
//!   With sharps/flats on the row above each (S,D,G,H,J / 2,3,5,6,7)
//!
//! Press = Note On, Release = Note Off

/// PC keyboard key → MIDI note mapping
/// Layout mimics a real piano keyboard:
///
/// ```text
///  PC Keys:    2  3     5  6  7     9  0
///              S  D     G  H  J     L  ;
/// MIDI:       C# D#   F# G# A#   C# D#   (sharps)
///
///  PC Keys:  Q  W  E  R  T  Y  U  I  O  P
///             Z  X  C  V  B  N  M  ,  .  /
/// MIDI:      C  D  E  F  G  A  B  C  D  E  (naturals)
/// ```

/// MIDI note mapping entry
#[derive(Debug, Clone, Copy)]
pub struct KeyMapping {
    /// PS/2 scancode (set 1)
    pub scancode: u8,
    /// MIDI note number
    pub midi_note: u8,
}

/// Lower row (Z-/ keys) → C3 octave (MIDI 48-64)
/// These are the PS/2 scan codes for set 1
static LOWER_ROW: &[KeyMapping] = &[
    // White keys (naturals)
    KeyMapping { scancode: 0x2C, midi_note: 48 }, // Z → C3
    KeyMapping { scancode: 0x2D, midi_note: 50 }, // X → D3
    KeyMapping { scancode: 0x2E, midi_note: 52 }, // C → E3
    KeyMapping { scancode: 0x2F, midi_note: 53 }, // V → F3
    KeyMapping { scancode: 0x30, midi_note: 55 }, // B → G3
    KeyMapping { scancode: 0x31, midi_note: 57 }, // N → A3
    KeyMapping { scancode: 0x32, midi_note: 59 }, // M → B3
    KeyMapping { scancode: 0x33, midi_note: 60 }, // , → C4
    KeyMapping { scancode: 0x34, midi_note: 62 }, // . → D4
    KeyMapping { scancode: 0x35, midi_note: 64 }, // / → E4
    // Black keys (sharps) — row above (S,D,F,G,H,J,K,L)
    KeyMapping { scancode: 0x1F, midi_note: 49 }, // S → C#3
    KeyMapping { scancode: 0x20, midi_note: 51 }, // D → D#3
    // F (0x21) is skipped — no E#
    KeyMapping { scancode: 0x22, midi_note: 54 }, // G → F#3
    KeyMapping { scancode: 0x23, midi_note: 56 }, // H → G#3
    KeyMapping { scancode: 0x24, midi_note: 58 }, // J → A#3
    // K (0x25) is skipped — no B#
    KeyMapping { scancode: 0x26, midi_note: 61 }, // L → C#4
    KeyMapping { scancode: 0x27, midi_note: 63 }, // ; → D#4
];

/// Upper row (Q-P keys) → C4 octave (MIDI 60-76)
static UPPER_ROW: &[KeyMapping] = &[
    // White keys (naturals)
    KeyMapping { scancode: 0x10, midi_note: 60 }, // Q → C4
    KeyMapping { scancode: 0x11, midi_note: 62 }, // W → D4
    KeyMapping { scancode: 0x12, midi_note: 64 }, // E → E4
    KeyMapping { scancode: 0x13, midi_note: 65 }, // R → F4
    KeyMapping { scancode: 0x14, midi_note: 67 }, // T → G4
    KeyMapping { scancode: 0x15, midi_note: 69 }, // Y → A4 (440 Hz!)
    KeyMapping { scancode: 0x16, midi_note: 71 }, // U → B4
    KeyMapping { scancode: 0x17, midi_note: 72 }, // I → C5
    KeyMapping { scancode: 0x18, midi_note: 74 }, // O → D5
    KeyMapping { scancode: 0x19, midi_note: 76 }, // P → E5
    // Black keys (sharps) — number row (2,3,4,5,6,7,8,9,0)
    KeyMapping { scancode: 0x03, midi_note: 61 }, // 2 → C#4
    KeyMapping { scancode: 0x04, midi_note: 63 }, // 3 → D#4
    // 4 (0x05) is skipped — no E#
    KeyMapping { scancode: 0x06, midi_note: 66 }, // 5 → F#4
    KeyMapping { scancode: 0x07, midi_note: 68 }, // 6 → G#4
    KeyMapping { scancode: 0x08, midi_note: 70 }, // 7 → A#4
    // 8 (0x09) is skipped — no B#
    KeyMapping { scancode: 0x0A, midi_note: 73 }, // 9 → C#5
    KeyMapping { scancode: 0x0B, midi_note: 75 }, // 0 → D#5
];

/// Current octave offset (can be shifted up/down)
static OCTAVE_OFFSET: core::sync::atomic::AtomicI8 = core::sync::atomic::AtomicI8::new(0);

/// Default velocity for keyboard input
static DEFAULT_VELOCITY: core::sync::atomic::AtomicU8 = core::sync::atomic::AtomicU8::new(100);

use core::sync::atomic::Ordering;

/// Look up a scancode in the mapping tables
/// Returns Some(midi_note) with octave offset applied
pub fn scancode_to_midi(scancode: u8) -> Option<u8> {
    let offset = OCTAVE_OFFSET.load(Ordering::Relaxed) as i16 * 12;

    // Search lower row first
    for mapping in LOWER_ROW {
        if mapping.scancode == scancode {
            let note = mapping.midi_note as i16 + offset;
            return if note >= 0 && note <= 127 { Some(note as u8) } else { None };
        }
    }

    // Then upper row
    for mapping in UPPER_ROW {
        if mapping.scancode == scancode {
            let note = mapping.midi_note as i16 + offset;
            return if note >= 0 && note <= 127 { Some(note as u8) } else { None };
        }
    }

    None
}

/// Shift octave up
pub fn octave_up() -> i8 {
    let current = OCTAVE_OFFSET.load(Ordering::Relaxed);
    if current < 4 {
        OCTAVE_OFFSET.store(current + 1, Ordering::Relaxed);
    }
    OCTAVE_OFFSET.load(Ordering::Relaxed)
}

/// Shift octave down
pub fn octave_down() -> i8 {
    let current = OCTAVE_OFFSET.load(Ordering::Relaxed);
    if current > -4 {
        OCTAVE_OFFSET.store(current - 1, Ordering::Relaxed);
    }
    OCTAVE_OFFSET.load(Ordering::Relaxed)
}

/// Get current octave offset
pub fn get_octave() -> i8 {
    OCTAVE_OFFSET.load(Ordering::Relaxed)
}

/// Set default velocity
pub fn set_velocity(vel: u8) {
    DEFAULT_VELOCITY.store(vel.min(127), Ordering::Relaxed);
}

/// Get default velocity
pub fn get_velocity() -> u8 {
    DEFAULT_VELOCITY.load(Ordering::Relaxed)
}

/// Check if a scancode is a mapped piano key (for filtering in piano mode)
pub fn is_piano_key(scancode: u8) -> bool {
    scancode_to_midi(scancode).is_some()
}

/// Get the display layout for the keyboard mapping
pub fn display_layout() -> &'static str {
    concat!(
        "TrustDAW Virtual Piano — PC Keyboard Layout\n",
        "═══════════════════════════════════════════════\n",
        "\n",
        " Number row (sharps/flats):\n",
        "  [2]  [3]      [5]  [6]  [7]      [9]  [0]\n",
        "  C#4  D#4      F#4  G#4  A#4      C#5  D#5\n",
        "\n",
        " QWERTY row (upper naturals, octave 4):\n",
        "  [Q]  [W]  [E] [R]  [T]  [Y]  [U] [I]  [O]  [P]\n",
        "   C4   D4   E4  F4   G4   A4   B4  C5   D5   E5\n",
        "\n",
        " Home row (sharps/flats):\n",
        "  [S]  [D]      [G]  [H]  [J]      [L]  [;]\n",
        "  C#3  D#3      F#3  G#3  A#3      C#4  D#4\n",
        "\n",
        " Bottom row (lower naturals, octave 3):\n",
        "  [Z]  [X]  [C] [V]  [B]  [N]  [M] [,]  [.]  [/]\n",
        "   C3   D3   E3  F3   G3   A3   B3  C4   D4   E4\n",
        "\n",
        " Controls:\n",
        "  [F1] Octave -  │  [F2] Octave +\n",
        "  [F3] Vel -     │  [F4] Vel +\n",
        "  [Esc] Exit piano mode\n",
    )
}
