//! Scale registry for the TrustStrudel DSL.
//!
//! All scales are stored as semitone offsets from the root.
//! `scale("g:minor")` is parsed into `(root_pc=7, intervals=&MINOR)`.
//!
//! Used by the `n(...)` builtin to map scale-degree integers
//! (negative or >= scale-len wrap with octave shifts) into MIDI notes.

#[derive(Debug, Clone, Copy)]
pub struct Scale {
    pub name: &'static str,
    /// Semitone offsets from the root (length is the scale size).
    pub steps: &'static [i32],
}

pub const CHROMATIC: &[i32] = &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];
pub const MAJOR: &[i32] = &[0, 2, 4, 5, 7, 9, 11];
pub const MINOR: &[i32] = &[0, 2, 3, 5, 7, 8, 10];
pub const HARMONIC_MINOR: &[i32] = &[0, 2, 3, 5, 7, 8, 11];
pub const MELODIC_MINOR: &[i32] = &[0, 2, 3, 5, 7, 9, 11];
pub const DORIAN: &[i32] = &[0, 2, 3, 5, 7, 9, 10];
pub const PHRYGIAN: &[i32] = &[0, 1, 3, 5, 7, 8, 10];
pub const LYDIAN: &[i32] = &[0, 2, 4, 6, 7, 9, 11];
pub const MIXOLYDIAN: &[i32] = &[0, 2, 4, 5, 7, 9, 10];
pub const LOCRIAN: &[i32] = &[0, 1, 3, 5, 6, 8, 10];
pub const PENTATONIC_MAJ: &[i32] = &[0, 2, 4, 7, 9];
pub const PENTATONIC_MIN: &[i32] = &[0, 3, 5, 7, 10];
pub const BLUES: &[i32] = &[0, 3, 5, 6, 7, 10];
pub const WHOLE_TONE: &[i32] = &[0, 2, 4, 6, 8, 10];
pub const HUNGARIAN_MIN: &[i32] = &[0, 2, 3, 6, 7, 8, 11];
pub const JAPANESE: &[i32] = &[0, 1, 5, 7, 8];

/// Resolve a mode name (case-insensitive) into its semitone steps.
pub fn lookup(mode: &str) -> Option<&'static [i32]> {
    let m = mode.trim();
    let mut buf = [0u8; 32];
    let mut n = 0usize;
    for b in m.as_bytes() {
        if n >= buf.len() { break; }
        buf[n] = b.to_ascii_lowercase();
        n += 1;
    }
    let lower = core::str::from_utf8(&buf[..n]).unwrap_or("");
    match lower {
        "major" | "maj" | "ionian"      => Some(MAJOR),
        "minor" | "min" | "aeolian"     => Some(MINOR),
        "harmonicminor" | "hmin" | "harmonic_minor" => Some(HARMONIC_MINOR),
        "melodicminor"  | "mmin" | "melodic_minor"  => Some(MELODIC_MINOR),
        "dorian"     => Some(DORIAN),
        "phrygian"   => Some(PHRYGIAN),
        "lydian"     => Some(LYDIAN),
        "mixolydian" => Some(MIXOLYDIAN),
        "locrian"    => Some(LOCRIAN),
        "pentatonic" | "majpent" | "pentatonicmajor" => Some(PENTATONIC_MAJ),
        "minpent" | "pentatonicminor" => Some(PENTATONIC_MIN),
        "blues" => Some(BLUES),
        "wholetone" | "whole_tone" => Some(WHOLE_TONE),
        "hungarian" | "hungarianminor" => Some(HUNGARIAN_MIN),
        "japanese" | "hirajoshi" => Some(JAPANESE),
        "chromatic" => Some(CHROMATIC),
        _ => None,
    }
}

/// Parse a root pitch-class letter ("c", "c#", "db", "g", "f#"…) into 0..11.
pub fn root_pc(s: &str) -> Option<i32> {
    let bytes = s.as_bytes();
    if bytes.is_empty() { return None; }
    let mut pc: i32 = match bytes[0].to_ascii_lowercase() as char {
        'c' => 0, 'd' => 2, 'e' => 4, 'f' => 5,
        'g' => 7, 'a' => 9, 'b' => 11,
        _ => return None,
    };
    if bytes.len() > 1 {
        match bytes[1] as char {
            '#' => pc += 1,
            'b' => pc -= 1,
            _ => {}
        }
    }
    Some(((pc % 12) + 12) % 12)
}

/// Parse `"<root>:<mode>"` (e.g. `"g:minor"`, `"f#:dorian"`).
/// Returns `(root_pc, scale_steps)`.
pub fn parse(spec: &str) -> Option<(i32, &'static [i32])> {
    let mut it = spec.splitn(2, ':');
    let root_part = it.next()?.trim();
    let mode_part = it.next()?.trim();
    let pc = root_pc(root_part)?;
    let steps = lookup(mode_part)?;
    Some((pc, steps))
}

/// Map a (possibly negative or large) scale degree into a MIDI note,
/// given the scale (`root_pc`, `steps`) and a base octave (default 4 = MIDI 60 for C).
pub fn degree_to_midi(degree: i32, root_pc: i32, steps: &[i32], base_octave: i32) -> u8 {
    let n = steps.len() as i32;
    if n == 0 { return 60; }
    // Wrap degree across octaves.
    let oct_extra = degree.div_euclid(n);
    let idx = degree.rem_euclid(n) as usize;
    let semis = root_pc + steps[idx] + 12 * (base_octave + oct_extra);
    // MIDI note 0..127.
    let midi = (semis + 12).clamp(0, 127); // shift so base_octave 4 => MIDI 60-ish
    midi as u8
}
