//! Audio lookup tables for TrustSynth
//!
//! All tables are integer-based (no floating-point) for bare-metal operation.
//! - Sine table: 256 entries, Q15 fixed-point (range ±32767)
//! - MIDI frequency table: 128 notes → Hz (integer)

/// Sine table — 256 entries, one full period, Q15 (±32767)
/// Generated from: round(sin(2π × i / 256) × 32767)
pub static SINE_TABLE: [i16; 256] = [
         0,    804,   1608,   2410,   3212,   4011,   4808,   5602,
      6393,   7179,   7962,   8739,   9512,  10278,  11039,  11793,
     12539,  13279,  14010,  14732,  15446,  16151,  16846,  17530,
     18204,  18868,  19519,  20159,  20787,  21403,  22005,  22594,
     23170,  23731,  24279,  24811,  25329,  25832,  26319,  26790,
     27245,  27683,  28105,  28510,  28898,  29268,  29621,  29956,
     30273,  30571,  30852,  31113,  31356,  31580,  31785,  31971,
     32137,  32285,  32412,  32521,  32609,  32678,  32728,  32757,
     32767,  32757,  32728,  32678,  32609,  32521,  32412,  32285,
     32137,  31971,  31785,  31580,  31356,  31113,  30852,  30571,
     30273,  29956,  29621,  29268,  28898,  28510,  28105,  27683,
     27245,  26790,  26319,  25832,  25329,  24811,  24279,  23731,
     23170,  22594,  22005,  21403,  20787,  20159,  19519,  18868,
     18204,  17530,  16846,  16151,  15446,  14732,  14010,  13279,
     12539,  11793,  11039,  10278,   9512,   8739,   7962,   7179,
      6393,   5602,   4808,   4011,   3212,   2410,   1608,    804,
         0,   -804,  -1608,  -2410,  -3212,  -4011,  -4808,  -5602,
     -6393,  -7179,  -7962,  -8739,  -9512, -10278, -11039, -11793,
    -12539, -13279, -14010, -14732, -15446, -16151, -16846, -17530,
    -18204, -18868, -19519, -20159, -20787, -21403, -22005, -22594,
    -23170, -23731, -24279, -24811, -25329, -25832, -26319, -26790,
    -27245, -27683, -28105, -28510, -28898, -29268, -29621, -29956,
    -30273, -30571, -30852, -31113, -31356, -31580, -31785, -31971,
    -32137, -32285, -32412, -32521, -32609, -32678, -32728, -32757,
    -32767, -32757, -32728, -32678, -32609, -32521, -32412, -32285,
    -32137, -31971, -31785, -31580, -31356, -31113, -30852, -30571,
    -30273, -29956, -29621, -29268, -28898, -28510, -28105, -27683,
    -27245, -26790, -26319, -25832, -25329, -24811, -24279, -23731,
    -23170, -22594, -22005, -21403, -20787, -20159, -19519, -18868,
    -18204, -17530, -16846, -16151, -15446, -14732, -14010, -13279,
    -12539, -11793, -11039, -10278,  -9512,  -8739,  -7962,  -7179,
     -6393,  -5602,  -4808,  -4011,  -3212,  -2410,  -1608,   -804,
];

/// MIDI note → frequency in Hz (integer approximation)
/// A4 (note 69) = 440 Hz, equal temperament tuning
/// Formula: f = 440 × 2^((n-69)/12)
pub static MIDI_FREQ: [u32; 128] = [
       8,     9,     9,    10,    10,    11,    12,    12, //   0-7   C-1 to G-1
      13,    14,    15,    15,    16,    17,    18,    19, //   8-15  G#-1 to D#0
      21,    22,    23,    25,    26,    28,    29,    31, //  16-23  E0 to B0
      33,    35,    37,    39,    41,    44,    46,    49, //  24-31  C1 to G1
      52,    55,    58,    62,    65,    69,    73,    78, //  32-39  G#1 to D#2
      82,    87,    92,    98,   104,   110,   117,   123, //  40-47  E2 to B2
     131,   139,   147,   156,   165,   175,   185,   196, //  48-55  C3 to G3
     208,   220,   233,   247,   262,   277,   294,   311, //  56-63  G#3 to D#4
     330,   349,   370,   392,   415,   440,   466,   494, //  64-71  E4 to B4
     523,   554,   587,   622,   659,   698,   740,   784, //  72-79  C5 to G5
     831,   880,   932,   988,  1047,  1109,  1175,  1245, //  80-87  G#5 to D#6
    1319,  1397,  1480,  1568,  1661,  1760,  1865,  1976, //  88-95  E6 to B6
    2093,  2217,  2349,  2489,  2637,  2794,  2960,  3136, //  96-103 C7 to G7
    3322,  3520,  3729,  3951,  4186,  4435,  4699,  4978, // 104-111 G#7 to D#8
    5274,  5588,  5920,  6272,  6645,  7040,  7459,  7902, // 112-119 E8 to B8
    8372,  8870,  9397,  9956, 10548, 11175, 11840, 12544, // 120-127 C9 to G9
];

/// Note name → MIDI note number lookup
/// Returns (midi_note, octave_offset) for common note names
pub fn note_name_to_midi(name: &str) -> Option<u8> {
    let bytes = name.as_bytes();
    if bytes.is_empty() {
        return None;
    }

    // Parse note letter
    let (base, rest) = match bytes[0] {
        b'C' | b'c' => (0u8, &bytes[1..]),
        b'D' | b'd' => (2, &bytes[1..]),
        b'E' | b'e' => (4, &bytes[1..]),
        b'F' | b'f' => (5, &bytes[1..]),
        b'G' | b'g' => (7, &bytes[1..]),
        b'A' | b'a' => (9, &bytes[1..]),
        b'B' | b'b' => (11, &bytes[1..]),
        _ => return None,
    };

    // Parse optional sharp/flat
    let (semitone_offset, rest) = if rest.first() == Some(&b'#') {
        (1i8, &rest[1..])
    } else if rest.first() == Some(&b'b') {
        (-1i8, &rest[1..])
    } else {
        (0i8, rest)
    };

    // Parse octave (0-9)
    if rest.is_empty() {
        return None;
    }
    let octave: u8 = {
        let mut val = 0u8;
        for &b in rest {
            if b >= b'0' && b <= b'9' {
                val = val * 10 + (b - b'0');
            } else {
                return None;
            }
        }
        val
    };

    // MIDI note = (octave + 1) × 12 + base + semitone
    let midi = (octave as i16 + 1) * 12 + base as i16 + semitone_offset as i16;
    if midi >= 0 && midi <= 127 {
        Some(midi as u8)
    } else {
        None
    }
}

/// MIDI note number → note name string (e.g., 69 → "A4")
pub fn midi_to_note_name(note: u8) -> &'static str {
    // Pre-computed names for the most common range (C2-C7 = notes 36-96)
    static NAMES: [&str; 12] = [
        "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"
    ];
    let _ = NAMES[note as usize % 12]; // just for bounds check
    // We return the base name; caller must append octave
    NAMES[note as usize % 12]
}

/// Get the octave of a MIDI note
pub fn midi_octave(note: u8) -> u8 {
    if note >= 12 { (note / 12) - 1 } else { 0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_note_parsing() {
        assert_eq!(note_name_to_midi("C4"), Some(60));
        assert_eq!(note_name_to_midi("A4"), Some(69));
        assert_eq!(note_name_to_midi("C#4"), Some(61));
        assert_eq!(note_name_to_midi("Bb3"), Some(58));
    }
}
