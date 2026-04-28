# TrustStrudel — Feature Parity Roadmap

> Goal: bring `kernel/src/audio/strudel.rs` + the in-kernel REPL to **full parity**
> with [strudel.cc](https://strudel.cc) so we can run the kind of trance live-coding
> demo seen in the reference screenshots (Switch Angel video) directly on bare metal.
>
> Reference scraped 2026-04-28 from `strudel.cc/learn/{mini-notation,synths,effects}`
> and `strudel.cc/functions/intro`. This file is the single source of truth for
> the parity work.

Legend:
- ✅ done in TrustOS today
- 🟡 partial (works but missing options/aliases)
- ❌ not started
- 🔥 visible in the target screenshots → high priority

---

## 0. Strategic phasing

| Phase | Goal | Visible to user |
|---|---|---|
| **P1** | Method-chaining DSL parser (no audio change) | `live` accepts `$: n("0 4 7").s("saw")` |
| **P2** | Mini-notation full coverage | All `[]` `<>` `*` `/` `!` `@` `?` `\|` `(n,k,o)` |
| **P3** | Synths parity (sawtooth / supersaw / fm / wavetable / noise) | Trance lead works |
| **P4** | Effects parity (lpenv, acidenv, duck, room, phaser, …) | Acid bass + sidechain |
| **P5** | Pattern combinators (stack, cat, jux, fast, slow, rev, …) | Compose tracks |
| **P6** | Visual REPL — multi-line editor in framebuffer | Looks like screenshots |
| **P7** | Inline visualisers `_pianoroll()` `_scope()` `_spectrum()` | Cyan grid + waveform |
| **P8** | Inline `slider()` widgets (mouse-clickable) | Live param tweak |
| **P9** | Hot-reload eval (Ctrl+Enter) without audio gap | Loop-quantised swap |

P1+P2+P3+P4+P5 = **audio parity**. P6+P7+P8+P9 = **visual parity**.

---

## 1. Mini-notation (P2)

| Token | Meaning | Status |
|---|---|---|
| `c4 d4 e4` | Sequence (events / cycle) | ✅ |
| `[a b]` | Sub-group (subdivide one slot) | ✅ |
| `<a b c>` | Alternation (one event per cycle) | ❌ 🔥 |
| `*N` | Repeat speed-up | ✅ (int only) |
| `*2.75` | Decimal multiply | ❌ |
| `/N` | Slowdown | ❌ |
| `~` `-` | Rest | ✅ |
| `_` | Tied note (extend previous) | ❌ |
| `,` | Polyphony (chord) | ❌ 🔥 |
| `@N` | Elongation (temporal weight) | ❌ |
| `!N` | Replication (no speed-up) | ❌ |
| `?` `?0.1` | Random degrade | ❌ |
| `a\|b\|c` | Random choice | ❌ |
| `(beats,segs[,offset])` | Euclidean rhythm | ❌ 🔥 |
| `bd:N` | Sample/voice index | ❌ |
| `>>` | Double-speed shorthand seen in screenshot (`*16`) | ❌ |

Visible in screenshots: `<0 4 0 9 7>*16`, `<7 _ _ 6 5 _ _ 6>*2`, chords, alternations.

---

## 2. Pattern factories & combinators (P5)

| Function | Purpose | Status |
|---|---|---|
| `note(p)` | Pitch from name (c4, eb3, …) | 🟡 (no flat/sharp variants) |
| `n(p)` | Scale-degree number | ❌ 🔥 |
| `s(name)` / `sound(name)` | Pick waveform / sample | ✅ via `wave()` |
| `freq(hz)` | Direct frequency | ✅ |
| `seq(a,b,c)` | Sequence | ❌ |
| `stack(a,b)` | Layer in parallel | ❌ |
| `cat(a,b)` | Concatenate (one per cycle) | ❌ |
| `irand(n)` | Random int 0..n | ❌ |
| `rand` `perlin` `sine` `tri` | Continuous signals | ❌ |
| `run(n)` | Ramp 0..n-1 | ❌ |
| `slow(n)` `fast(n)` | Time stretch | ❌ |
| `rev` | Reverse | ❌ |
| `jux(fn)` `juxBy(amt,fn)` | Stereo function-split | ❌ |
| `ply(n)` | Repeat per event | ❌ |
| `seg(n)` | Resample to n events | ❌ |
| `every(n,fn)` | Apply fn every n cycles | ❌ |
| `sometimes(fn)` `often(fn)` `rarely(fn)` | Probabilistic apply | ❌ |
| `add(p)` `sub(p)` `mul(p)` | Arithmetic on patterns | ❌ |
| `chord(name)` `voicing()` `dict('ireal')` | Jazz chord voicings | ❌ |

---

## 3. Tonal / scale system (P3-tonal)

Strudel:
```
n("0 4 7").scale("g:minor").trans(-12)
```
We need:

- [ ] Scale registry: major, minor, dorian, phrygian, lydian, mixolydian, locrian,
      harmonic_minor, melodic_minor, pentatonic (major/minor), blues, chromatic,
      whole-tone, hungarian, japanese, …
- [ ] `scale("root:mode")` parser (`"g:minor"`, `"C:major"`, `"f#:dorian"`)
- [ ] `n(degrees)` → scale-degree → MIDI note
- [ ] `trans(semitones)` — global transpose
- [ ] `octave(n)` / `o(n)` — set octave 🔥
- [ ] `add(p)` for adding intervals between voicings (the `.add("<7 _ 6 …>")` in screenshots) 🔥
- [ ] `detune(amt)` — small pitch shift on osc copies 🔥

---

## 4. Synths (P3)

| Synth | Strudel name | Status |
|---|---|---|
| Sine | `sine` | ✅ |
| Square | `square` | ✅ |
| Sawtooth | `sawtooth` / `saw` | ✅ 🔥 |
| Triangle | `triangle` / `tri` | ✅ |
| Pulse | `pulse` | ❌ |
| Noise white | `white` | ✅ |
| Noise pink | `pink` | ❌ 🔥 |
| Noise brown | `brown` | ❌ |
| Crackle | `crackle` (+ `density`) | ❌ |
| **Supersaw** (7 detuned saws) | `supersaw` | ❌ 🔥 |
| FM synth (`fm`, `fmh`, `fma`, `fmd`, `fms`, `fme`) | `fm` family | ❌ |
| Wavetable (`wt_*`) | `wt_flute`, … | ❌ |
| ZZFX engine | `z_*` | ❌ (skip, low priority) |
| Vibrato (`vib`, `vibmod`) | per-voice LFO | 🟡 (have `vibrato` global) |
| Partials (`partials([1,1,…])`) | additive | ❌ |
| Phases (`phases([…])`) | additive | ❌ |

**Supersaw** is THE trance sound in the screenshots → must implement (7 saws,
detune ±0.1..0.5%, mixed equally, soft-clip).

---

## 5. Filters (P4-filter)

| Function | Aliases | Status |
|---|---|---|
| `lpf(hz)` | `lp`, `cutoff`, `ctf` | ✅ |
| `lpq(q)` | `resonance` | ✅ |
| `hpf(hz)` | `hp`, `hcutoff` | ✅ |
| `hpq(q)` | `hresonance` | ✅ |
| `bpf(hz)` | `bandf`, `bp` | ✅ |
| `bpq(q)` | `bandq` | ✅ |
| `ftype` | `12db` / `ladder` / `24db` | ❌ |
| `vowel(a/e/i/o/u/…)` | formant filter | ❌ |

Filter mini-notation: `lpf("1000:10")` = cutoff:resonance combined → ❌

---

## 6. Envelopes (P4-env)

### Amplitude ADSR
| Function | Aliases | Status |
|---|---|---|
| `attack(s)` | `att`, `a` | ✅ |
| `decay(s)` | `dec`, `d` | ✅ |
| `sustain(level)` | `sus` | ✅ |
| `release(s)` | `rel`, `r` | ✅ |
| `adsr("a:d:s:r")` | combined | ❌ |
| `gain(amt)` | linear gain | ✅ |
| `velocity(v)` | `vel` | ❌ |
| `clip(amt)` | clip note duration | ❌ |

### Filter envelope (`lpenv`, `hpenv`, `bpenv`) 🔥
| Function | Aliases | Status |
|---|---|---|
| `lpattack(s)` | `lpa` | ❌ 🔥 |
| `lpdecay(s)` | `lpd` | ❌ 🔥 |
| `lpsustain(l)` | `lps` | ❌ 🔥 |
| `lprelease(s)` | `lpr` | ❌ |
| `lpenv(depth)` | `lpe` | ❌ 🔥 |
| same set for `hp*` and `bp*` | | ❌ |
| `acidenv(amt)` | TB-303-style preset (sets lpa+lpd+lpenv at once) | ❌ 🔥 |

**`acidenv` and `lpenv(x*9).lps(.2).lpd(.12)` are visible in EVERY screenshot.**

### Pitch envelope
| Function | Aliases | Status |
|---|---|---|
| `pattack` | `patt` | ❌ |
| `pdecay` | `pdec` | ❌ |
| `prelease` | `prel` | ❌ |
| `penv(semitones)` | depth | ❌ |
| `pcurve(0/1)` | linear/exp | ❌ |
| `panchor(0..1)` | range anchor | ❌ |

### FM envelope: `fmattack/fmdecay/fmsustain/fmenv` → ❌

---

## 7. Effects (P4-fx)

### Local effects (per-voice)
| Function | Aliases | Status |
|---|---|---|
| `coarse(n)` | sample-rate reducer | ❌ |
| `crush(bits)` | bit-crusher | ❌ |
| `shape(amt)` | waveshaper | ❌ |
| `distort(amt[:gain[:type]])` | `dist` | 🟡 (basic) |
| `tremolo` family (`tremsync`, `tremdepth`, `tremskew`, `tremphase`, `tremshape`) | AM | 🟡 (have global tremolo only) |
| `compressor("th:ratio:knee:a:r")` | dynamics | ❌ |
| `postgain(g)` | post-stage gain | ❌ |
| `pan(0..1)` | stereo placement | ❌ (kernel is mono) |
| `phaser(speed)` `phaserdepth` `phasercenter` `phasersweep` | `ph`, `phd`, `phc`, `phs` | ❌ |
| `vib(hz)` `vibmod(semi)` | per-voice vibrato | 🟡 |

### Global effects (per-orbit)
| Function | Aliases | Status |
|---|---|---|
| `delay(level[:time[:fb]])` | `dt`, `dfb` | 🟡 (basic, no time/fb mininotation) |
| `delaytime(s)` | `delayt`, `dt` | 🟡 |
| `delayfeedback(amt)` | `delayfb`, `dfb` | 🟡 |
| `room(level[:size])` | reverb level | ❌ 🔥 |
| `roomsize(n)` | `rsize`, `sz`, `size` | ❌ |
| `roomfade` `roomlp` `roomdim` | reverb tone | ❌ |
| `iresponse(sample)` | `ir` — convolution | ❌ |
| `orbit(n)` | `o` — orbit selector 🔥 | ❌ 🔥 |
| `duckorbit(n)` | `duck` — sidechain 🔥 | ❌ 🔥 |
| `duckattack(s)` | `duckatt`, `datt` | ❌ 🔥 |
| `duckdepth(n)` | sidechain depth | ❌ 🔥 |
| `xfade` | crossfade | ❌ |

`.duck("3:4:5:6")` in the trance screenshot = sidechain target orbits 3,4,5,6 from
the kick orbit. Must work.

---

## 8. Time / conditional / random modifiers (P5-modifiers)

| Function | Status |
|---|---|
| `fast(n)` `slow(n)` | ❌ |
| `rev` | ❌ |
| `early(n)` `late(n)` | ❌ |
| `iter(n)` `iterBack(n)` | ❌ |
| `chunk(n,fn)` | ❌ |
| `every(n,fn)` `whenmod` | ❌ |
| `sometimes` `often` `rarely` `almostNever` `almostAlways` | ❌ |
| `degrade` `degradeBy(p)` | ❌ |
| `range(lo,hi)` (on signals) | ❌ |
| `irand(n)` `choose([…])` | ❌ |
| `binaryL` `randL` (lists) | ❌ |
| `fit()` (auto-fit pattern length) 🔥 | ❌ 🔥 |

---

## 9. Method-chaining DSL (P1) — the fundamental blocker

Strudel patterns are JS objects with chained methods. We need a tiny interpreter
in the kernel that:

1. Tokenises a line like `n("0 4 7").scale("g:minor").s("sawtooth").lpenv(4)`.
2. Builds an AST: `MethodCall { receiver, method, args }`.
3. Evaluates left-to-right onto a `PatternBuilder` that accumulates control values.
4. Yields a final `Pattern` (already exists in `audio/pattern.rs`) with extra
   per-step control map (cutoff, lpenv depth, distort, etc.).

Files to add/modify:
- `kernel/src/audio/strudel/lexer.rs` — tokeniser (idents, strings, numbers, `.`, `(`, `)`, `,`)
- `kernel/src/audio/strudel/parser.rs` — recursive-descent for chains
- `kernel/src/audio/strudel/eval.rs` — apply each method to PatternBuilder
- `kernel/src/audio/strudel/builtins.rs` — registry of all `.method(args)` impls
- `kernel/src/audio/pattern.rs` — extend `Step` with control-map (cutoff, env params, fx sends)
- `kernel/src/audio/live_engine.rs` — render `Pattern` honouring control map

This is the single largest piece of work and unblocks everything else.

---

## 10. Multi-line REPL UI (P6) 🔥

Target visual (from screenshots):
- Cyan-on-black framebuffer terminal, monospaced font.
- Left pane: code editor with line numbers (1, 2, 3, …).
- Right pane: visualizer / video / album art (we have `audio_viz`).
- Each track header: `$:` (active) or `_$:` (muted).
- Comments rendered grey: `// LET US TRANCE ONCE MORE`.
- Argument highlighting: `n("...")`, `scale("...")`, `s("...")` → arg text wrapped
  in a thin cyan rectangle (token-based syntax highlight).
- Status bar at bottom: BPM, cycle, CPU%.

Modules to build:
- `kernel/src/audio/repl/editor.rs` — gap-buffer multi-line editor + cursor
- `kernel/src/audio/repl/render.rs` — framebuffer paint (uses `framebuffer/`
  + the existing 8×16 font); supports syntax highlight spans
- `kernel/src/audio/repl/keymap.rs` — Ctrl+Enter (eval all), Shift+Enter (eval
  block), Ctrl+. (panic = stop all), Ctrl+S (save to VFS)
- `kernel/src/audio/repl/mod.rs` — main loop, hooks into PS/2 keyboard
- Shell command: `strudel` (or `repl`) → enters fullscreen REPL mode

---

## 11. Inline visualisers (P7) 🔥

Strudel-style underscore-prefixed methods that don't change audio, only display:

| Method | Visual | Status |
|---|---|---|
| `._pianoroll()` | Step grid below the track (cyan blocks) 🔥 | ❌ 🔥 |
| `._scope()` | Live waveform | ❌ 🔥 (have `audio_viz` to reuse) |
| `._spectrum()` | FFT bars | ❌ |
| `._punchcard()` | Density histogram | ❌ |

Implementation: when parser hits `_pianoroll` it emits a `Visualiser::PianoRoll`
attached to the track; renderer reserves N lines under that track in the
editor framebuffer.

For `_pianoroll`:
- Width = editor pane width − margin.
- Grid = `cycle_steps × pitch_range` where pitch range auto-fits to the
  pattern's min/max note.
- Filled cell = note active at that step.
- Highlight current step (playhead) with a brighter column.

Reuse the existing `kernel/src/visualizer.rs` and `trustdaw/audio_viz.rs`
for `_scope` / `_spectrum`.

---

## 12. Inline interactive widgets (P8)

The screenshots show `slider(0.546)` rendered as an actual draggable slider
**inside the source code line**, not as a separate UI panel. Plan:

- Parser recognises `slider(initial[, min[, max]])` as a special expression.
- Each slider gets a stable id from `(line, column)` of its source.
- State is kept in `repl::widgets::SliderState { id, value, bounds }`.
- Renderer paints a horizontal bar with a knob at the slider's source column.
- Mouse click + drag (we already have `kernel/src/input/mouse.rs`) updates
  the value; eval re-runs with the new value but only for that one
  expression — no full re-evaluation needed.
- Also planned: `pick("a","b",…)`, `toggle()`, `xy()`.

Mouse already exists in TrustOS — we just need hit-testing per slider.

---

## 13. Hot-reload eval (P9)

When the user presses Ctrl+Enter:
- Parse all `$:` / `_$:` blocks in the buffer.
- Diff against previously running tracks.
- For each changed track:
  - Build new pattern.
  - **Wait for current cycle boundary** (no audio glitch).
  - Atomic swap into `live_engine` track slot.
- For removed tracks: fade out over 1 cycle.
- For added tracks: schedule start at next bar.

`audio/live_engine.rs` already supports per-track patterns; we just need the
quantised-swap discipline.

---

## 14. Suggested commit order

1. **P1** Method-chaining parser (lexer + parser + minimal eval, no audio change yet).
   Test: `live '$: s("bd sd")'` keeps working.
2. **P2 mini-notation completions**: `<>`, `,`, `(n,k)`, `?`, `|`, `!`, `@`, `_`, `:N`.
3. **P3 synths**: pink/brown noise, supersaw, pulse.
4. **P4 envelopes**: full `lpenv` family + `acidenv` macro.
5. **P4 effects**: `room`, `orbit`, `duck` (sidechain), `compressor`, `phaser`.
6. **P5 combinators**: `stack`, `cat`, `fast`/`slow`, `rev`, `every`, `sometimes`, `jux`.
7. **P3-tonal**: scales + `n()` + `trans` + `add` + `detune`.
8. **P6 REPL editor**: framebuffer multi-line buffer + syntax highlight + Ctrl+Enter.
9. **P7 visualisers**: `_pianoroll`, `_scope`.
10. **P8 sliders**: inline `slider()`.
11. **P9 hot-reload**: cycle-boundary swap.

After step 7 the screenshots' AUDIO is reproducible from the shell.
After step 11 the VISUAL is reproducible too.

---

## 15. Constraints / risks

- **`no_std` integer DSP only**: every Strudel example uses floats.
  We must port everything to Q16.16 or Q31 fixed-point (already the convention).
- **Heap usage**: a typical Strudel program builds dozens of patterns per cycle;
  must use small allocations + arenas (`bumpalo` not available, do a custom
  cycle arena).
- **No JS engine**: we deliberately implement only the chained-DSL subset, not
  arbitrary JavaScript. Lambdas (`fn => …`) inside `every`, `jux`, etc. become
  named operators only (`every(4, rev)` works; `every(4, n => n.fast(2))` does not).
- **Reverb / FFT**: needs serious DSP work in fixed-point. `room` can ship as
  a Schroeder reverb (4 combs + 2 allpass) → ~150 LOC, runs in real-time on
  G4400 easily.
- **Mouse interaction in fullscreen REPL**: PS/2 mouse only; USB HID (xhci) is
  partial. The board has PS/2, the laptop has USB → ship PS/2 first.

---

*Source of truth. Update statuses as features land.*
