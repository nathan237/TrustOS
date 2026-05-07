//! Evaluator for the chained DSL.
//!
//! Walks an `Expr` tree produced by `parser::parse` and yields a
//! `Bundle` describing one playable pattern, plus a global control map.
//!
//! Today's coverage (P1 + P3-tonal + a slice of P5):
//!   - sources: `s("...")`, `sound("...")`, `note("...")`, `n("...")`, `freq(N)`
//!   - synth selection: `.s("name")`, `.sound("name")`
//!   - tonal: `.scale("g:minor")`, `.octave(N)`, `.trans(N)`, `.add(N)`
//!   - dynamics: `.gain(x)`, `.volume(x)` (0..1 in Q16.16 or 0..127 int)
//!   - tempo: `.bpm(N)`, `.cps(x)`, `.fast(N)`, `.slow(N)`, `.rev()`
//!   - filter: `.lpf(hz)`, `.lpq(q)`, `.hpf(hz)`, `.hpq(q)`, `.bpf(hz)`, `.bpq(q)`
//!   - envelope: `.attack(s)`/`.att`/`.a`, `.decay`/`.dec`/`.d`,
//!               `.sustain`/`.sus`, `.release`/`.rel`/`.r`, `.adsr("a:d:s:r")`
//!   - structure: `.detune(amt)`, `.room(amt)`, `.delay(amt)` (stored on bundle)
//!
//! Anything we don't recognise yet is **silently ignored** (with a debug log)
//! so demos can keep typing forward-compatible patches.

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;

use super::parser::Expr;
use super::scales;
use super::super::pattern::{Pattern, Step};
use super::super::strudel;
use super::super::synth::{Waveform, Envelope};
use super::super::tables;

/// Q16.16 unit value (1.0).
const Q: i64 = 1 << 16;

/// Per-pattern global controls extracted from the chain.
#[derive(Debug, Clone, Copy)]
pub struct Controls {
    /// Linear gain (Q16.16). 1.0 = unity.
    pub gain: i64,
    /// Detune in cents (Q16.16). 0 = no detune.
    pub detune_cents: i64,
    /// Low-pass filter cutoff (Hz). 0 = bypass.
    pub lpf_hz: u32,
    /// LPF resonance (Q16.16, 0..1).
    pub lpq: i64,
    /// High-pass filter cutoff (Hz). 0 = bypass.
    pub hpf_hz: u32,
    pub hpq: i64,
    /// Band-pass filter centre (Hz). 0 = bypass.
    pub bpf_hz: u32,
    pub bpq: i64,
    /// Reverb send level (Q16.16). 0 = dry.
    pub room: i64,
    /// Delay send level (Q16.16). 0 = dry.
    pub delay: i64,
    /// Pan (-1..+1 in Q16.16).
    pub pan: i64,
    /// Distortion amount (Q16.16).
    pub distort: i64,
    /// Filter envelope depth (Q16.16) — `lpenv`.
    pub lpenv: i64,
    /// Filter envelope attack (samples). 0 = use synth default.
    pub lpa_samples: u32,
    pub lpd_samples: u32,
    pub lps_q: i64,
    pub lpr_samples: u32,
    /// Sidechain duck depth (Q16.16). 0 = no duck.
    pub duck_depth: i64,
    /// Orbit selector for global routing (default 0).
    pub orbit: u8,
}

impl Default for Controls {
    fn default() -> Self {
        Self {
            gain: Q, detune_cents: 0,
            lpf_hz: 0, lpq: 0, hpf_hz: 0, hpq: 0, bpf_hz: 0, bpq: 0,
            room: 0, delay: 0, pan: 0, distort: 0,
            lpenv: 0, lpa_samples: 0, lpd_samples: 0, lps_q: 0, lpr_samples: 0,
            duck_depth: 0,
            orbit: 0,
        }
    }
}

/// Final result of evaluating a chain: a Pattern + extra global controls.
#[derive(Clone)]
pub struct Bundle {
    pub pattern: Pattern,
    pub controls: Controls,
}

/// Builder used during evaluation.
struct Builder {
    /// MIDI note steps (or rests). Authoritative once `materialise()` runs.
    steps: Vec<Step>,
    /// Default waveform for the pattern.
    waveform: Waveform,
    /// Envelope (overrides defaults).
    envelope: Envelope,
    /// Controls being accumulated.
    controls: Controls,
    /// Current scale `(root_pc, intervals)`. None = chromatic / use note names.
    scale: Option<(i32, &'static [i32])>,
    /// Octave offset applied to scale-degree resolution and to note names.
    octave_offset: i32,
    /// Semitone transposition applied at the end.
    transpose: i32,
    /// Source kind: how to interpret upcoming `n("...")` etc.
    pending_source: SourceKind,
    /// BPM for the pattern.
    bpm: u16,
}

#[derive(Clone, Debug)]
enum SourceKind {
    /// `s("bd sd")` — drum aliases / synth name lookup
    Drums,
    /// `note("c4 e4")` — note names
    Notes,
    /// `n("0 4 7")` — scale-degree integers
    Degrees,
    /// `freq(N)` — single frequency, one-step pattern
    Freq(u32),
}

impl Builder {
    fn new() -> Self {
        Self {
            steps: Vec::new(),
            waveform: Waveform::Sawtooth,
            envelope: Envelope::pluck(),
            controls: Controls::default(),
            scale: None,
            octave_offset: 0,
            transpose: 0,
            pending_source: SourceKind::Drums,
            bpm: 120,
        }
    }

    /// Once steps are filled, apply transpose/scale/octave shifts.
    fn finalise(mut self) -> Bundle {
        if self.steps.is_empty() {
            self.steps.push(Step::rest());
        }

        let trans = self.transpose;
        let oct = self.octave_offset;
        for step in self.steps.iter_mut() {
            if step.is_rest() { continue; }
            let mut n = step.note as i32 + trans + 12 * oct;
            n = n.clamp(0, 127);
            step.note = n as u8;
        }

        let mut pat = Pattern::new("dsl", self.steps.len(), self.bpm);
        pat.steps = self.steps;
        pat.waveform = self.waveform;
        pat.envelope = self.envelope;

        Bundle { pattern: pat, controls: self.controls }
    }
}

// ─── Helpers ────────────────────────────────────────────────────────────────

fn arg_str(args: &[Expr], idx: usize) -> Option<&str> {
    match args.get(idx) {
        Some(Expr::Str(s)) => Some(s.as_str()),
        Some(Expr::Ident(s)) => Some(s.as_str()),
        _ => None,
    }
}

fn arg_int(args: &[Expr], idx: usize) -> Option<i64> {
    match args.get(idx) {
        Some(Expr::Int(n)) => Some(*n),
        Some(Expr::Fixed(q)) => Some(q >> 16),
        _ => None,
    }
}

/// Read arg as Q16.16, accepting either int or fixed.
fn arg_q(args: &[Expr], idx: usize) -> Option<i64> {
    match args.get(idx) {
        Some(Expr::Int(n)) => Some((*n) << 16),
        Some(Expr::Fixed(q)) => Some(*q),
        _ => None,
    }
}

fn waveform_from_name(s: &str) -> Option<Waveform> {
    Waveform::from_str(s).or_else(|| match s {
        // Common Strudel aliases.
        "sawtooth" | "saw" => Some(Waveform::Sawtooth),
        "square" | "pulse" => Some(Waveform::Square),
        "triangle" | "tri" => Some(Waveform::Triangle),
        "sine" | "sin" => Some(Waveform::Sine),
        "noise" | "white" | "pink" | "brown" | "crackle" => Some(Waveform::Noise),
        // Future synth names map to closest existing waveform until we add them.
        "supersaw" | "fm" | "wt_flute" => Some(Waveform::Sawtooth),
        _ => None,
    })
}

/// Parse a mini-notation string as a sequence of MIDI notes / rests,
/// respecting the current scale (for "0 4 7"-style degree input).
fn parse_steps(text: &str, source: &SourceKind, scale: &Option<(i32, &'static [i32])>) -> Result<Vec<Step>, &'static str> {
    match source {
        SourceKind::Freq(_) => Ok(alloc::vec![Step::rest()]),
        SourceKind::Drums => strudel::parse(text),
        SourceKind::Notes => strudel::parse(text),
        SourceKind::Degrees => parse_degrees(text, scale),
    }
}

fn parse_degrees(text: &str, scale: &Option<(i32, &'static [i32])>) -> Result<Vec<Step>, &'static str> {
    let (root, steps) = scale.unwrap_or((0, scales::MAJOR));
    let mut out = Vec::new();
    for tok in text.split_ascii_whitespace() {
        if tok == "~" || tok == "." || tok == "-" || tok == "_" {
            out.push(Step::rest());
            continue;
        }
        // Optional `+N` or `-N` octave nudges via brackets `[3+12]` are not yet supported here.
        let deg: i32 = match tok.parse() {
            Ok(v) => v,
            Err(_) => {
                // Allow alphabetic note names too as a fallback.
                if let Some(midi) = tables::note_name_to_midi(tok) {
                    out.push(Step::note(midi));
                    continue;
                }
                return Err("bad degree");
            }
        };
        let midi = scales::degree_to_midi(deg, root, steps, 4);
        out.push(Step::note(midi));
    }
    if out.is_empty() {
        return Err("empty degree pattern");
    }
    Ok(out)
}

// ─── Method dispatch ────────────────────────────────────────────────────────

fn apply_method(b: &mut Builder, name: &str, args: &[Expr]) -> Result<(), &'static str> {
    match name {
        // Synth / sound override.
        "s" | "sound" => {
            if let Some(s) = arg_str(args, 0) {
                if let Some(wf) = waveform_from_name(s) { b.waveform = wf; }
            }
        }
        // Volume / gain.
        "gain" | "volume" | "amp" => {
            if let Some(q) = arg_q(args, 0) {
                b.controls.gain = q;
            }
        }
        // Tempo controls.
        "bpm" => { if let Some(n) = arg_int(args, 0) { b.bpm = n.clamp(30, 300) as u16; } }
        "cps" => {
            // cycles-per-second: bpm = cps * 60 * 4 (one cycle = 1 bar of 4 beats by default).
            if let Some(q) = arg_q(args, 0) {
                let cps_x256 = (q >> 8) as i64; // approx
                let bpm = ((cps_x256 * 60 * 4) >> 8) as i64;
                b.bpm = bpm.clamp(30, 300) as u16;
            }
        }
        "fast" => {
            // fast(N) duplicates the pattern N times in the same cycle.
            if let Some(n) = arg_int(args, 0) {
                if n >= 2 {
                    let original = b.steps.clone();
                    for _ in 1..n { b.steps.extend(original.iter().copied()); }
                }
            }
        }
        "slow" => {
            // slow(N) extends each step into N copies of (note + rests).
            if let Some(n) = arg_int(args, 0) {
                if n >= 2 {
                    let mut out = Vec::with_capacity(b.steps.len() * n as usize);
                    for s in &b.steps {
                        out.push(*s);
                        for _ in 1..n { out.push(Step::rest()); }
                    }
                    b.steps = out;
                }
            }
        }
        "rev" => { b.steps.reverse(); }

        // Tonal.
        "scale" => {
            if let Some(spec) = arg_str(args, 0) {
                if let Some(parsed) = scales::parse(spec) {
                    b.scale = Some(parsed);
                }
            }
        }
        "octave" | "o" => {
            if let Some(n) = arg_int(args, 0) {
                b.octave_offset = (n as i32).clamp(-4, 4);
            }
        }
        "trans" | "transpose" => {
            if let Some(n) = arg_int(args, 0) {
                b.transpose = b.transpose.saturating_add(n as i32);
            }
        }
        "add" => {
            // Single-int form: shift all notes by N semitones (degree-aware via transpose).
            if let Some(n) = arg_int(args, 0) {
                b.transpose = b.transpose.saturating_add(n as i32);
            }
        }
        "detune" => {
            if let Some(q) = arg_q(args, 0) { b.controls.detune_cents = q; }
        }

        // Filter.
        "lpf" | "lp" | "cutoff" | "ctf" => {
            if let Some(n) = arg_int(args, 0) { b.controls.lpf_hz = n.max(0) as u32; }
        }
        "lpq" | "resonance" => { if let Some(q) = arg_q(args, 0) { b.controls.lpq = q; } }
        "hpf" | "hp" | "hcutoff" => { if let Some(n) = arg_int(args, 0) { b.controls.hpf_hz = n.max(0) as u32; } }
        "hpq" | "hresonance" => { if let Some(q) = arg_q(args, 0) { b.controls.hpq = q; } }
        "bpf" | "bp" | "bandf" => { if let Some(n) = arg_int(args, 0) { b.controls.bpf_hz = n.max(0) as u32; } }
        "bpq" | "bandq" => { if let Some(q) = arg_q(args, 0) { b.controls.bpq = q; } }

        // Envelope.
        "attack" | "att" | "a" => { if let Some(q) = arg_q(args, 0) { b.envelope.attack_samples = seconds_q_to_samples(q); } }
        "decay"  | "dec" | "d" => { if let Some(q) = arg_q(args, 0) { b.envelope.decay_samples  = seconds_q_to_samples(q); } }
        "sustain" | "sus" => { if let Some(q) = arg_q(args, 0) { b.envelope.sustain_level = q_to_q15(q); } }
        "release" | "rel" | "r" => { if let Some(q) = arg_q(args, 0) { b.envelope.release_samples = seconds_q_to_samples(q); } }
        "adsr" => {
            if let Some(spec) = arg_str(args, 0) { apply_adsr(b, spec); }
        }

        // Filter envelope (P4-env).
        "lpenv" | "lpe" => { if let Some(q) = arg_q(args, 0) { b.controls.lpenv = q; } }
        "lpattack" | "lpa" => { if let Some(q) = arg_q(args, 0) { b.controls.lpa_samples = seconds_q_to_samples(q); } }
        "lpdecay"  | "lpd" => { if let Some(q) = arg_q(args, 0) { b.controls.lpd_samples = seconds_q_to_samples(q); } }
        "lpsustain"| "lps" => { if let Some(q) = arg_q(args, 0) { b.controls.lps_q = q; } }
        "lprelease"| "lpr" => { if let Some(q) = arg_q(args, 0) { b.controls.lpr_samples = seconds_q_to_samples(q); } }
        "acidenv" => {
            // Macro: 303-style preset. Optional Q16.16 amount scales the depth.
            let amt = arg_q(args, 0).unwrap_or(Q);
            b.controls.lpenv = (Q * 6 * amt) >> 16; // depth ~6.0 * amt
            b.controls.lpa_samples = seconds_q_to_samples(Q / 1000); // 1 ms
            b.controls.lpd_samples = seconds_q_to_samples((Q * 12) / 100); // 0.12 s
            b.controls.lps_q = Q / 5; // 0.2
        }

        // Effects.
        "room" => { if let Some(q) = arg_q(args, 0) { b.controls.room = q; } }
        "delay" | "echo" => { if let Some(q) = arg_q(args, 0) { b.controls.delay = q; } }
        "pan" => { if let Some(q) = arg_q(args, 0) { b.controls.pan = q; } }
        "distort" | "dist" | "shape" | "crush" => { if let Some(q) = arg_q(args, 0) { b.controls.distort = q; } }
        "duckdepth" | "duck" => { if let Some(q) = arg_q(args, 0) { b.controls.duck_depth = q; } }
        "orbit" | "ob" => { if let Some(n) = arg_int(args, 0) { b.controls.orbit = n.clamp(0, 15) as u8; } }

        // Display-only / no-op for now; these are picked up by the REPL renderer later.
        "_pianoroll" | "_scope" | "_spectrum" | "_punchcard" => { /* visualiser markers */ }

        // Anything else: ignored silently. We keep the logs short.
        _ => {
            crate::serial_println!("[strudel-dsl] ignored .{}", name);
        }
    }
    Ok(())
}

fn apply_adsr(b: &mut Builder, spec: &str) {
    let mut it = spec.split(':');
    if let Some(s) = it.next() { if let Some(q) = parse_q(s) { b.envelope.attack_samples = seconds_q_to_samples(q); } }
    if let Some(s) = it.next() { if let Some(q) = parse_q(s) { b.envelope.decay_samples = seconds_q_to_samples(q); } }
    if let Some(s) = it.next() { if let Some(q) = parse_q(s) { b.envelope.sustain_level = q_to_q15(q); } }
    if let Some(s) = it.next() { if let Some(q) = parse_q(s) { b.envelope.release_samples = seconds_q_to_samples(q); } }
}

fn parse_q(s: &str) -> Option<i64> {
    let s = s.trim();
    if let Some(dot) = s.find('.') {
        let int_part: i64 = s[..dot].parse().ok()?;
        let frac_str = &s[dot + 1..];
        let mut frac_q = 0i64;
        let mut div = 1i64;
        for c in frac_str.chars().take(6) {
            if let Some(d) = c.to_digit(10) {
                frac_q = frac_q * 10 + d as i64;
                div *= 10;
            }
        }
        let frac_q16 = (frac_q << 16) / div.max(1);
        Some((int_part.abs() << 16 | frac_q16) * if int_part < 0 || s.starts_with('-') { -1 } else { 1 })
    } else {
        s.parse::<i64>().ok().map(|n| n << 16)
    }
}

fn seconds_q_to_samples(q: i64) -> u32 {
    let s = (q.max(0) * super::super::synth::SAMPLE_RATE as i64) >> 16;
    s.clamp(0, super::super::synth::SAMPLE_RATE as i64 * 30) as u32
}

fn q_to_q15(q: i64) -> i32 {
    // input Q16.16 in [0,1] → Q15
    let v = (q.max(0) * 32767) >> 16;
    v.clamp(0, 32767) as i32
}

// ─── Public API ─────────────────────────────────────────────────────────────

/// Evaluate a parsed `Expr` chain into a finished `Bundle`.
pub fn evaluate(expr: &Expr) -> Result<Bundle, &'static str> {
    let mut b = Builder::new();
    eval_into(&mut b, expr)?;
    // Materialise steps from the source if not already done by `freq()`.
    if b.steps.is_empty() {
        if let SourceKind::Freq(_hz) = b.pending_source {
            b.steps.push(Step::note(69)); // placeholder; freq routed via controls later
        }
    }
    Ok(b.finalise())
}

fn eval_into(b: &mut Builder, expr: &Expr) -> Result<(), &'static str> {
    match expr {
        Expr::Method { receiver, name, args } => {
            eval_into(b, receiver)?;
            apply_method(b, name, args)?;
            Ok(())
        }
        Expr::Call { name, args } => {
            apply_source(b, name, args)?;
            Ok(())
        }
        Expr::Ident(name) => {
            // Bare identifier as source (e.g. just `sawtooth`).
            if let Some(wf) = waveform_from_name(name) {
                b.waveform = wf;
                Ok(())
            } else {
                Err("unknown identifier as source")
            }
        }
        _ => Err("expected a chain or call as the root expression"),
    }
}

fn apply_source(b: &mut Builder, name: &str, args: &[Expr]) -> Result<(), &'static str> {
    match name {
        "s" | "sound" => {
            if let Some(text) = arg_str(args, 0) {
                // If the text matches a single known waveform name, just set the waveform.
                if let Some(wf) = waveform_from_name(text.trim()) {
                    b.waveform = wf;
                    if b.steps.is_empty() {
                        b.steps.push(Step::note(60)); // simple one-shot
                    }
                    return Ok(());
                }
                b.pending_source = SourceKind::Drums;
                b.steps = parse_steps(text, &SourceKind::Drums, &b.scale)?;
            }
            Ok(())
        }
        "note" | "n_name" => {
            if let Some(text) = arg_str(args, 0) {
                b.pending_source = SourceKind::Notes;
                b.steps = parse_steps(text, &SourceKind::Notes, &b.scale)?;
            }
            Ok(())
        }
        "n" => {
            if let Some(text) = arg_str(args, 0) {
                b.pending_source = SourceKind::Degrees;
                b.steps = parse_degrees(text, &b.scale)?;
            } else if let Some(v) = arg_int(args, 0) {
                let (root, steps) = b.scale.unwrap_or((0, scales::MAJOR));
                let midi = scales::degree_to_midi(v as i32, root, steps, 4);
                b.steps = alloc::vec![Step::note(midi)];
            }
            Ok(())
        }
        "freq" => {
            if let Some(v) = arg_int(args, 0) {
                b.pending_source = SourceKind::Freq(v.max(20) as u32);
                // Approximate MIDI note from frequency: midi = 69 + 12*log2(f/440)
                let midi = freq_to_midi(v as u32);
                b.steps = alloc::vec![Step::note(midi)];
            }
            Ok(())
        }
        "stack" | "cat" | "seq" => {
            // Minimal: concatenate child patterns one after the other.
            for a in args {
                let bundle = evaluate(a)?;
                b.steps.extend(bundle.pattern.steps.iter().copied());
                if b.bpm == 120 { b.bpm = bundle.pattern.bpm; }
                if matches!(b.waveform, Waveform::Sawtooth) {
                    b.waveform = bundle.pattern.waveform;
                }
            }
            Ok(())
        }
        _ => Err("unknown source function"),
    }
}

fn freq_to_midi(hz: u32) -> u8 {
    // Linear search over the MIDI table is fine: 128 entries, runs once per pattern.
    let mut best = 69u8;
    let mut best_d: i64 = i64::MAX;
    for (i, f) in tables::MIDI_FREQ.iter().enumerate() {
        let d = (*f as i64 - hz as i64).abs();
        if d < best_d { best_d = d; best = i as u8; }
    }
    best
}

/// One-shot helper: parse + evaluate.
pub fn parse_eval(src: &str) -> Result<Bundle, String> {
    let expr = super::parser::parse(src).map_err(|e| format!("parse error: {:?}", e))?;
    evaluate(&expr).map_err(|m| String::from(m))
}
