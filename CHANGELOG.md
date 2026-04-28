# Changelog

All notable changes to TrustOS. Format loosely follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/); the project does not yet follow strict SemVer.

## [v0.12.0] — 2026-04-28 — Audio Edition

First dedicated **music edition** of TrustOS: a `no_std` bare-metal audio stack with synth, DAW, sequencer, live coding, and effects.

### Added
- **TrustOS Audio Edition** — new build profile `trustos-audio` (`hires-logo + daw + gui-stack`).
  - Standalone ISO published as a release asset: `trustos-audio.iso`.
  - Build: `.\scripts\build\build-trustos-edition.ps1 -Edition audio`.
- **TrustSynth** polyphonic synthesizer
  - 5 waveforms: sine, square, saw, triangle, noise.
  - Q16.16 phase accumulator (integer DSP, no FP).
  - 8-voice polyphony, full ADSR envelope + presets (organ, pluck, pad).
  - 128-note MIDI mapping, analog-style pitch micro-drift LFO.
- **Effects rack** — Chamberlin SVF (LP / HP / BP with resonance), delay, distortion, tremolo, vibrato, volume/gain/fade.
- **Pattern Sequencer** — 16 patterns × 64 steps, BPM 60–300.
- **TrustDAW** (feature `daw`) — multi-track, solo / mute / pan / volume, Piano Roll, PS/2 keyboard → MIDI, real-time recording, transport controls, Beat Studio grid, WAV export, audio visualizer / VU / spectrum, 480 PPQN.
- **Strudel / TidalCycles live coding** — mini-notation parser with sequences, sub-groups, repeats, rests, ~60 drum aliases.
- New shell commands: `beep`, `audio`, `synth`, `live`, `daw`, `vizfx`.

### Changed
- README "What's new" promoted to v0.12.0 Audio Edition with full feature list and download link.

## [v0.11.0] — 2026-04-22 — Audio, Userland, GPU, Benchmarks

### Added
- **AMD SDMA pipeline validated** on real silicon (RX 580X / Polaris 10): ring buffer in GART, GMC fully initialized (L1/L2 TLB, system aperture, VM flat mode), firmware responsive, RPTR/WPTR advancing.
- **Intel HDA audio driver** — complete `no_std` driver, working output on ThinkPad T61 / AD1984. 12-iteration debug saga; root cause was swapped amp gain bits + GPIO polarity inversion + i16 triangle-wave overflow.
- **Ring 3 userland** — protected user-mode processes, isolated address spaces, syscall interface, 85-check conformance audit.
- **Hardware diagnostics suite** — 6 new `hwdbg` modules: `pciraw`, `regdiff`, `ioscan`, `regwatch`, `aer`, `timing` (15+ subcommands total).
- **ThinkPad EC driver** — fan control, thermal readout, battery status. CPU frequency scaling via MSR.
- **CoreMark benchmark** — 25,000 iter/sec on bare metal Intel G4400 (EEMBC standard).
- **AMD PSP driver scaffolding** for Navi 10+.

### Fixed
- `build-std` CI compatibility (`rust-src` component, `-p trustos_kernel`).
- Preemptive vulnerability fixes from cross-OS security audit.

### Changed
- README rewritten around the project's three pillars (hardware X-ray, remote-first, embedded AI).
- Repository layout cleaned: screenshots and devlogs moved under `docs/`, jailbreak research split out.

## [v0.10.5] — 2026-03 — Userland milestone
- Full Ring 3 integration with conformance audit.

## [v0.10.4] — 2026-03 — Hardware diagnostic toolkit
- First wave of `hwdbg` modules.

## [v0.10.2] — 2026-02 — Laptop bring-up
- ThinkPad EC + CPU frequency control.

## [v0.10.1] — 2026-03-13 — Settings & Network
- Settings GUI, NetScan GUI, shell scrollback fix, ACPI shutdown hardening, T61 hardware optimization, matrix rain, GitHub Pages site.

## Earlier
See `git log` for the full history.
