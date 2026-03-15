//! Live Visualizer Effects — TrustLang-scripted visual effects for audio playback
//!
//! Allows creating, listing, selecting and running TrustLang scripts as
//! real-time visualizer effects that react to audio data (FFT bands, beat).
//!
//! The active effect is called each frame during `play` / `daw viz` playback.
//! No reboot, no recompile — write the script, add it, play a song.
//!
//! Shell: `vizfx new <name>`, `vizfx list`, `vizfx select <name>`, `vizfx edit <name>`
//!
//! Scripts can use all TrustLang graphics builtins (pixel, fill_rect, draw_circle,
//! draw_line, draw_text, clear_screen, flush, screen_w, screen_h) plus:
//!   beat()    → 0.0–1.0 current beat pulse
//!   bass()    → 0.0–1.0 bass band energy
//!   mid()     → 0.0–1.0 mid band energy
//!   treble()  → 0.0–1.0 treble band energy
//!   energy()  → 0.0–1.5 overall energy
//!   sub_bass()→ 0.0–1.0 sub-bass energy
//!   high_mid()→ 0.0–1.0 high-mid energy
//!   frame_num() → current frame number (int)
//!   sin_f(x)  → sin(x) (float)
//!   cos_f(x)  → cos(x) (float)

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use core::sync::atomic::{AtomicU32, AtomicBool, Ordering};
use spin::Mutex;

// ═══════════════════════════════════════════════════════════════════════════════
// Live Effect Storage
// ═══════════════════════════════════════════════════════════════════════════════

const MAXIMUM_EFFECTS: usize = 8;

struct LiveEffect {
    name: String,
    source: String,
}

struct EffectRegistry {
    effects: Vec<LiveEffect>,
    active_index: Option<usize>,
}

// Implementation block — defines methods for the type above.
impl EffectRegistry {
    const fn new() -> Self {
        Self {
            effects: Vec::new(),
            active_index: None,
        }
    }
}

// Global shared state guarded by a Mutex (mutual exclusion lock).
static REGISTRY: Mutex<EffectRegistry> = Mutex::new(EffectRegistry::new());

/// Global audio data injected each frame by the visualizer loop.
/// The TrustLang builtins read from these.
static AUDIO_BEAT: AtomicU32 = AtomicU32::new(0);
// Atomic variable — provides lock-free thread-safe access.
static AUDIO_BASS: AtomicU32 = AtomicU32::new(0);
// Atomic variable — provides lock-free thread-safe access.
static AUDIO_SUB_BASS: AtomicU32 = AtomicU32::new(0);
// Atomic variable — provides lock-free thread-safe access.
static AUDIO_MID: AtomicU32 = AtomicU32::new(0);
// Atomic variable — provides lock-free thread-safe access.
static AUDIO_HIGH_MID: AtomicU32 = AtomicU32::new(0);
// Atomic variable — provides lock-free thread-safe access.
static AUDIO_TREBLE: AtomicU32 = AtomicU32::new(0);
// Atomic variable — provides lock-free thread-safe access.
static AUDIO_ENERGY: AtomicU32 = AtomicU32::new(0);
// Atomic variable — provides lock-free thread-safe access.
static FRAME_NUMBER: AtomicU32 = AtomicU32::new(0);
// Atomic variable — provides lock-free thread-safe access.
static LIVE_VIZ_ACTIVE: AtomicBool = AtomicBool::new(false);

// ═══════════════════════════════════════════════════════════════════════════════
// Audio Data Injection (called by visualizer loop each frame)
// ═══════════════════════════════════════════════════════════════════════════════

/// Update audio data globals (called from the playback loop each frame)
pub fn set_audio_data(beat: f32, bass: f32, sub_bass: f32, mid: f32,
                      high_mid: f32, treble: f32, energy: f32, frame: u32) {
    AUDIO_BEAT.store(beat.to_bits(), Ordering::Relaxed);
    AUDIO_BASS.store(bass.to_bits(), Ordering::Relaxed);
    AUDIO_SUB_BASS.store(sub_bass.to_bits(), Ordering::Relaxed);
    AUDIO_MID.store(mid.to_bits(), Ordering::Relaxed);
    AUDIO_HIGH_MID.store(high_mid.to_bits(), Ordering::Relaxed);
    AUDIO_TREBLE.store(treble.to_bits(), Ordering::Relaxed);
    AUDIO_ENERGY.store(energy.to_bits(), Ordering::Relaxed);
    FRAME_NUMBER.store(frame, Ordering::Relaxed);
}

/// Read audio builtins (called from TrustLang VM)
pub fn get_beat() -> f32 { f32::from_bits(AUDIO_BEAT.load(Ordering::Relaxed)) }
// Public function — callable from other modules.
pub fn get_bass() -> f32 { f32::from_bits(AUDIO_BASS.load(Ordering::Relaxed)) }
// Public function — callable from other modules.
pub fn get_sub_bass() -> f32 { f32::from_bits(AUDIO_SUB_BASS.load(Ordering::Relaxed)) }
// Public function — callable from other modules.
pub fn get_mid() -> f32 { f32::from_bits(AUDIO_MID.load(Ordering::Relaxed)) }
// Public function — callable from other modules.
pub fn get_high_mid() -> f32 { f32::from_bits(AUDIO_HIGH_MID.load(Ordering::Relaxed)) }
// Public function — callable from other modules.
pub fn get_treble() -> f32 { f32::from_bits(AUDIO_TREBLE.load(Ordering::Relaxed)) }
// Public function — callable from other modules.
pub fn get_energy() -> f32 { f32::from_bits(AUDIO_ENERGY.load(Ordering::Relaxed)) }
// Public function — callable from other modules.
pub fn get_frame_number() -> u32 { FRAME_NUMBER.load(Ordering::Relaxed) }

/// Check if a live viz effect is active
pub fn is_active() -> bool {
    LIVE_VIZ_ACTIVE.load(Ordering::Relaxed)
}

// ═══════════════════════════════════════════════════════════════════════════════
// Effect Management
// ═══════════════════════════════════════════════════════════════════════════════

/// Add a new effect. Returns Ok or error if name exists or limit reached.
pub fn add_effect(name: &str, source: &str) -> Result<(), &'static str> {
    let mut reg = REGISTRY.lock();
    if reg.effects.len() >= MAXIMUM_EFFECTS {
        return Err("Maximum 8 effects reached — remove one first");
    }
    if reg.effects.iter().any(|e| e.name == name) {
        return Err("Effect with this name already exists — use 'vizfx edit' to update");
    }
    // Syntax-check before accepting
    if let Err(e) = crate::trustlang::check(source) {
        crate::serial_println!("[VIZFX] Syntax error: {}", e);
        return Err("TrustLang syntax error — check your script");
    }
    reg.effects.push(LiveEffect {
        name: String::from(name),
        source: String::from(source),
    });
    // Auto-select if first effect
    if reg.effects.len() == 1 {
        reg.active_index = Some(0);
        LIVE_VIZ_ACTIVE.store(true, Ordering::Relaxed);
    }
    Ok(())
}

/// Replace the source of an existing effect.
pub fn edit_effect(name: &str, source: &str) -> Result<(), &'static str> {
    // Syntax-check before accepting
    if let Err(e) = crate::trustlang::check(source) {
        crate::serial_println!("[VIZFX] Syntax error: {}", e);
        return Err("TrustLang syntax error — check your script");
    }
    let mut reg = REGISTRY.lock();
    if let Some(eff) = reg.effects.iterator_mut().find(|e| e.name == name) {
        eff.source = String::from(source);
        Ok(())
    } else {
        Err("Effect not found")
    }
}

/// Remove an effect by name.
pub fn remove_effect(name: &str) -> Result<(), &'static str> {
    let mut reg = REGISTRY.lock();
    let index = reg.effects.iter().position(|e| e.name == name)
        .ok_or("Effect not found")?;
    reg.effects.remove(index);
    // Fix active index
    match reg.active_index {
        Some(i) if i == index => {
            reg.active_index = if reg.effects.is_empty() { None } else { Some(0) };
            if reg.active_index.is_none() {
                LIVE_VIZ_ACTIVE.store(false, Ordering::Relaxed);
            }
        }
        Some(i) if i > index => reg.active_index = Some(i - 1),
        _ => {}
    }
    Ok(())
}

/// Select an effect by name.
pub fn select_effect(name: &str) -> Result<(), &'static str> {
    let mut reg = REGISTRY.lock();
    let index = reg.effects.iter().position(|e| e.name == name)
        .ok_or("Effect not found")?;
    reg.active_index = Some(index);
    LIVE_VIZ_ACTIVE.store(true, Ordering::Relaxed);
    Ok(())
}

/// Disable live viz (go back to default overlay).
pub fn disable() {
    LIVE_VIZ_ACTIVE.store(false, Ordering::Relaxed);
}

/// Enable live viz (re-enable the selected effect).
pub fn enable() -> Result<(), &'static str> {
    let reg = REGISTRY.lock();
    if reg.active_index.is_some() {
        LIVE_VIZ_ACTIVE.store(true, Ordering::Relaxed);
        Ok(())
    } else {
        Err("No effect selected")
    }
}

/// List all effects. Returns (name, is_active) pairs.
pub fn list_effects() -> Vec<(String, bool)> {
    let reg = REGISTRY.lock();
    reg.effects.iter().enumerate().map(|(i, e)| {
        (e.name.clone(), reg.active_index == Some(i))
    }).collect()
}

/// Get the source code of an effect.
pub fn get_source(name: &str) -> Option<String> {
    let reg = REGISTRY.lock();
    reg.effects.iter().find(|e| e.name == name).map(|e| e.source.clone())
}

// ═══════════════════════════════════════════════════════════════════════════════
// Per-Frame Execution (called from the visualizer render loop)
// ═══════════════════════════════════════════════════════════════════════════════

/// Run the active live effect for one frame.
/// Called from the visualizer playback loop after overlay rendering.
/// Returns true if an effect was executed.
pub fn run_frame() -> bool {
    if !LIVE_VIZ_ACTIVE.load(Ordering::Relaxed) {
        return false;
    }
    // Get the active effect source (clone to release lock before execution)
    let source = {
        let reg = REGISTRY.lock();
                // Pattern matching — Rust's exhaustive branching construct.
match reg.active_index {
            Some(index) => reg.effects.get(index).map(|e| e.source.clone()),
            None => None,
        }
    };

    if let Some(source) = source {
        // Execute the TrustLang script
        // Errors are logged to serial but don't crash
        match crate::trustlang::run(&source) {
            Ok(_) => {}
            Err(e) => {
                let frame = FRAME_NUMBER.load(Ordering::Relaxed);
                // Only log errors every 60 frames to avoid serial spam
                if frame % 60 == 0 {
                    crate::serial_println!("[VIZFX] Script error (frame {}): {}", frame, e);
                }
            }
        }
        true
    } else {
        false
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Built-In Demo Effects
// ═══════════════════════════════════════════════════════════════════════════════

/// Load a demo "Pulse Rings" effect — concentric circles that pulse with the beat.
pub fn load_demo_pulse_rings() -> Result<(), &'static str> {
    let source = r#"fn main() {
    let w = screen_w();
    let h = screen_h();
    let cx = w / 2;
    let cy = h / 2;
    let b = to_int(beat() * 255.0);
    let e = to_int(energy() * 200.0);
    let ba = to_int(bass() * 255.0);
    let t = frame_num();

    let i = 0;
    while i < 8 {
        let base_r = 30 + i * 40;
        let pulse = to_int(to_float(base_r) + beat() * 50.0);
        let r = to_int(to_float(ba) * 0.3);
        let g = to_int(to_float(b) * 0.8 + to_float(i * 20));
        let blue = to_int(to_float(e) * 0.5);
        draw_circle(cx, cy, pulse, r, g, blue);
        i = i + 1;
    }
}"#;
    add_effect("pulse-rings", source)
}

/// Load a demo "Spectrum Bars" effect — vertical bars for each frequency band.
pub fn load_demo_spectrum_bars() -> Result<(), &'static str> {
    let source = r#"fn main() {
    let w = screen_w();
    let h = screen_h();
    let bar_w = w / 8;
    let t = frame_num();

    let sb = to_int(sub_bass() * to_float(h));
    let ba = to_int(bass() * to_float(h));
    let lm = to_int(mid() * to_float(h) * 0.8);
    let mi = to_int(mid() * to_float(h) * 0.6);
    let hm = to_int(high_mid() * to_float(h) * 0.7);
    let tr = to_int(treble() * to_float(h) * 0.5);

    fill_rect(0 * bar_w, h - sb, bar_w - 2, sb, 255, 0, 50);
    fill_rect(1 * bar_w, h - ba, bar_w - 2, ba, 255, 50, 0);
    fill_rect(2 * bar_w, h - lm, bar_w - 2, lm, 200, 200, 0);
    fill_rect(3 * bar_w, h - mi, bar_w - 2, mi, 0, 255, 100);
    fill_rect(4 * bar_w, h - hm, bar_w - 2, hm, 0, 150, 255);
    fill_rect(5 * bar_w, h - tr, bar_w - 2, tr, 150, 0, 255);
}"#;
    add_effect("spectrum-bars", source)
}

/// Load a demo "Beat Flash" effect — screen border flashes on beat.
pub fn load_demo_beat_flash() -> Result<(), &'static str> {
    let source = r#"fn main() {
    let w = screen_w();
    let h = screen_h();
    let b = beat();
    if b > 0.3 {
        let intensity = to_int(b * 255.0);
        let thickness = to_int(b * 20.0) + 2;
        fill_rect(0, 0, w, thickness, 0, intensity, intensity / 3);
        fill_rect(0, h - thickness, w, thickness, 0, intensity, intensity / 3);
        fill_rect(0, 0, thickness, h, 0, intensity, intensity / 3);
        fill_rect(w - thickness, 0, thickness, h, 0, intensity, intensity / 3);
    }
}"#;
    add_effect("beat-flash", source)
}
