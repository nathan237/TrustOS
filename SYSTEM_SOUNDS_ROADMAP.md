# TrustOS System Sounds Roadmap

## Current State
The HDA driver already has a `SoundEffect` enum with basic effects:
- **BootChime** — Ascending C5-E5-G5 triad (550ms)
- **Click** — 1kHz tick (15ms)
- **Error** — Double 400Hz beep (300ms)
- **Notification** — Ascending A5-C#6-E6 (400ms)
- **Warning** — Descending 880→660Hz (500ms)
- **Success** — C5-E5 major third (300ms)
- **Keypress** — 2kHz micro-tick (8ms)

All effects are **blocking** (busy-wait via `write_samples_and_play()`).

---

## Phase 1 — Essential UI Sounds (Priority: HIGH)

| Sound | Trigger | Design | Duration |
|-------|---------|--------|----------|
| **Window Open** | `create_window()` | Quick ascending sweep (400→800Hz) with soft attack | ~80ms |
| **Window Close** | Close button click | Short descending sweep (800→400Hz) with fade | ~80ms |
| **Window Minimize** | Minimize to taskbar | Gentle low pluck (300Hz, fast decay) | ~50ms |
| **Window Maximize** | Maximize/restore | Quick "whoosh" noise burst, filtered | ~60ms |
| **Start Menu Open** | Start button click | Soft chime — single pure tone + harmonic | ~120ms |
| **Start Menu Close** | Click away / ESC | Muted version of open (lower pitch) | ~80ms |
| **Context Menu** | Right-click | Very subtle tick, higher than Click | ~10ms |
| **Button Hover** | Mouse enters button | Whisper-quiet high tone (3kHz, very low amplitude) | ~5ms |
| **Dialog Confirm** | OK / Yes button | Same as Success | ~300ms |
| **Dialog Cancel** | Cancel / No button | Soft descending minor second | ~150ms |

## Phase 2 — Desktop & File Manager Sounds (Priority: MEDIUM)

| Sound | Trigger | Design | Duration |
|-------|---------|--------|----------|
| **File Select** | Click file/icon | Soft click, slightly lower than Click | ~12ms |
| **File Copy** | Ctrl+C / copy action | Quick double-tick | ~30ms |
| **File Paste** | Ctrl+V / paste action | Soft "thud" — low frequency pulse | ~40ms |
| **File Delete** | Delete / recycle | Crumple noise — filtered noise burst | ~100ms |
| **Drag Start** | Begin drag operation | Light "pick up" — ascending micro-sweep | ~30ms |
| **Drag Drop** | Release drag on target | Soft "place" — descending thud | ~40ms |
| **Icon Double-Click** | Open app from desktop | Punchy click + quick chime tail | ~100ms |
| **Empty Recycle** | Empty trash | Longer crumple/shred noise | ~300ms |

## Phase 3 — System & Status Sounds (Priority: MEDIUM)

| Sound | Trigger | Design | Duration |
|-------|---------|--------|----------|
| **Login / Unlock** | Lock screen unlock | Warm ascending chord (C-E-G-C) | ~600ms |
| **Lock Screen** | Win+L / auto-lock | Descending chord, echoed | ~400ms |
| **USB/Device Connect** | Hot-plug detection | Rising two-tone (like Windows plug-in) | ~250ms |
| **USB/Device Disconnect** | Hot-unplug | Falling two-tone (reverse of connect) | ~250ms |
| **Low Battery** | Battery < 15% | Urgent repeated beeps (3x 440Hz) | ~600ms |
| **Critical Battery** | Battery < 5% | Continuous alarm tone | ~1000ms |
| **Volume Change** | Volume slider adjust | Short tone at current volume level | ~50ms |
| **Screenshot** | Print Screen | Camera "shutter" click (noise burst + click) | ~150ms |
| **Timer/Alarm** | Calendar/clock app | Melodic repeating bell | ~2000ms |

## Phase 4 — Application-Specific Sounds (Priority: LOW)

| Sound | Trigger | Design | Duration |
|-------|---------|--------|----------|
| **Terminal Command Done** | Long command completes | Subtle bell (BEL character) | ~80ms |
| **Terminal Tab Complete** | Tab completion | Micro-tick | ~5ms |
| **Editor Save** | Ctrl+S in editor | Soft confirmation "ding" | ~60ms |
| **Editor Unsaved Warning** | Close unsaved file | Warning variant, gentler | ~200ms |
| **Game Boy Boot** | GB emulator starts | Classic Game Boy boot sound (sweep) | ~500ms |
| **Chess Move** | Piece placed | Wood "clack" — filtered noise + low tone | ~40ms |
| **Chess Capture** | Piece captures | Sharper clack, slightly louder | ~50ms |
| **Chess Check** | King in check | Alert tone | ~100ms |
| **Chat Message** | Message received | Bubble "pop" — quick sine with pitch bend | ~60ms |
| **Download Complete** | File download done | Happy ascending arpeggio | ~400ms |

## Phase 5 — Ambient & Accessibility (Priority: LOW)

| Sound | Trigger | Design | Duration |
|-------|---------|--------|----------|
| **Focus Change** | Alt+Tab, window focus | Very subtle "switch" pulse | ~15ms |
| **Scroll Boundary** | Scroll past end of list | Soft rubber-band "bounce" | ~30ms |
| **Text Select** | Shift+Arrow selection | Micro-tone per character | ~3ms |
| **Screen Reader** | Accessibility mode | TTS integration (future) | Variable |
| **Ambient Hum** | Desktop idle (optional) | Very faint machinery/data-center hum | Continuous |

---

## Architecture Requirements

### Non-Blocking Sound Playback
Currently all `play_effect()` calls are **blocking** — they occupy the DMA buffer and busy-wait.
This is problematic for UI sounds (you can't freeze the UI for 100ms on every click).

**Solution: Audio Mixer**
1. Create a global audio mixer that combines multiple sources into the DMA buffer
2. Sources: music player stream, system sound effects, Game Boy APU, synth engine
3. Each source writes to a private buffer; the mixer sums them into DMA at 48kHz
4. Use a ring buffer per source with lock-free push/pop
5. The mixer runs on the frame-tick cycle (every ~16ms at 60fps)

### Sound Theme System
```
kernel/src/sounds/
├── mod.rs          # SoundTheme trait, theme manager
├── default.rs      # Default cyberpunk theme (synthesized)
├── minimal.rs      # Minimal theme (fewer, quieter sounds)
├── silent.rs       # No sounds at all
└── generator.rs    # Procedural sound synthesis helpers
```

### User Settings
- Master volume (0-100)
- Sound theme selection
- Per-category volume: UI, System, Apps, Notifications
- Enable/disable specific sounds

### Game Boy APU Implementation
The Game Boy emulator currently **stubs out all APU registers** (read→0xFF, write→no-op).
To make GB audio reactive with the background:
1. Implement the 4 GB audio channels (2 pulse, 1 wave, 1 noise)
2. Mix GB audio output (44.1kHz native → resample to 48kHz)
3. Feed into the global audio mixer
4. The global DMA analyzer in `analyze_global_audio()` will automatically pick it up

---

## Priority Order
1. **Non-blocking audio mixer** (required for everything)
2. **Window Open/Close** sounds (most noticeable UX improvement)
3. **Click, Button, Start Menu** sounds (interactive feedback)
4. **Login/Lock** sounds (system polish)
5. **File operations** sounds (file manager UX)
6. **Game Boy APU** (emulator completeness + audio reactivity)
7. **App-specific** sounds (feature completeness)
8. **Ambient/Accessibility** (polish)
