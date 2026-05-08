//! Audio-Sync Latency Calibration — Metronome Tap Test
//!
//! Measures end-to-end audio latency by:
//!
//! 1. **Kernel-side DMA probe**: Writes a click marker to the DMA buffer
//!    and monitors LPIB to see when the hardware plays it.  This gives the
//!    raw DMA pipeline delay (typically 10–50 ms).
//!
//! 2. **User tap calibration (metronome)**: Plays a steady metronome click
//!    at a known BPM.  The user taps a key on each beat they *hear*.
//!    Comparing tap timestamps to scheduled beat timestamps yields the
//!    perceived audio latency (DMA + DAC + speaker + perception).
//!
//! The resulting offset is stored in `MusicPlayerState::av_offset_ms`.

use alloc::vec::Vec;
use core::sync::atomic::{AtomicBool, AtomicI32, AtomicU32, AtomicU64, Ordering};

// ═══════════════════════════════════════════════════════════════════════════════
// Constants
// ═══════════════════════════════════════════════════════════════════════════════

/// Metronome BPM for the tap test (120 BPM = 500 ms per beat)
const METRONOME_BPM: u32 = 120;
/// Number of tap samples to collect before computing the offset
const TAP_SAMPLES_NEEDED: usize = 8;
/// Click tone frequency (Hz) — a sharp, audible tick
const CLICK_FREQUENCY_HZ: u32 = 1000;
/// Click tone duration (ms) — short and punchy
const CLICK_DURATION_MOUSE: u32 = 30;
/// Sample rate (must match HDA config)
const SAMPLE_RATE: u32 = 48_000;
/// Maximum acceptable offset (ms)
const MAXIMUM_OFFSET_MOUSE: i32 = 500;

// ═══════════════════════════════════════════════════════════════════════════════
// Shared state (accessible from interrupt & GUI contexts)
// ═══════════════════════════════════════════════════════════════════════════════

/// Whether a sync calibration session is active
static SYNC_ACTIVE: AtomicBool = AtomicBool::new(false);

/// Number of user taps collected so far
static TAP_COUNT: AtomicU32 = AtomicU32::new(0);

/// Beat index we're currently waiting for a tap on
static BEAT_INDEX: AtomicU32 = AtomicU32::new(0);

/// Microsecond timestamp of last beat played
static LAST_BEAT_US: AtomicU64 = AtomicU64::new(0);

/// Microsecond timestamp of first beat (session start)
static SESSION_START_US: AtomicU64 = AtomicU64::new(0);

/// Computed offset result (ms, positive = audio arrives late)
static COMPUTED_OFFSET_MOUSE: AtomicI32 = AtomicI32::new(0);

/// Whether the calibration produced a valid result
static RESULT_READY: AtomicBool = AtomicBool::new(false);

/// Whether there's a DMA probe result
static DMA_LATENCY_US: AtomicU64 = AtomicU64::new(0);

/// Ring buffer of measured deltas (tap_time - expected_beat_time) in µs.
/// Stored as i64 packed into pairs of u64 for atomic-free access.
/// We use a simple Mutex-protected Vec instead.
static TAP_DELTAS: spin::Mutex<Vec<i64>> = spin::Mutex::new(Vec::new());

// ═══════════════════════════════════════════════════════════════════════════════
// DMA Pipeline Latency Probe (fully automatic, no user input)
// ═══════════════════════════════════════════════════════════════════════════════

/// Measure the DMA pipeline latency by writing a short click to the
/// buffer and timing how long it takes for LPIB to reach it.
///
/// Call **outside** of active music playback (it briefly takes over HDA).
/// Returns the measured DMA latency in microseconds, or 0 on error.
pub fn probe_dma_latency() -> u64 {
    use crate::drivers::hda;

    // Generate a short click (1 ms of 1 kHz sine)
    let click = hda::generate_sine(CLICK_FREQUENCY_HZ, 2, 16000);
    if click.is_empty() { return 0; }

    // Stop any current playback
    let _ = hda::stop();
    hda::reset_stream();

    // Get DMA buffer pointer
    let (buf_ptr, buf_cap) = // Pattern matching — Rust's exhaustive branching construct.
match hda::get_dma_buffer_information() {
        Some(info) => info,
        None => return 0,
    };

    // Zero the entire buffer first
    unsafe {
        core::ptr::write_bytes(buf_ptr, 0, buf_cap);
    }

    // Write click at a known offset (quarter of buffer)
    let click_offset = buf_cap / 4;
    let click_length = click.len().min(buf_cap - click_offset);
        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
        core::ptr::copy_nonoverlapping(click.as_ptr(), buf_ptr.add(click_offset), click_length);
    }

    // Record the byte offset where the click starts
    let click_byte_offset = (click_offset * 2) as u32; // i16 → bytes

    // Start DMA and record the start time
    let start_us = crate::gui::engine::now_us();
    let _ = hda::start_dma();

    // Poll LPIB until it reaches or passes the click offset
    let timeout_us = 200_000; // 200 ms max
    loop {
        let now = crate::gui::engine::now_us();
        if now - start_us > timeout_us {
            let _ = hda::stop();
            crate::serial_println!("[SYNC] DMA probe timeout (200ms)");
            return 0;
        }

        let lpib = hda::get_playback_position();
        if lpib >= click_byte_offset {
            let latency = now - start_us;
            let _ = hda::stop();
            DMA_LATENCY_US.store(latency, Ordering::SeqCst);
            crate::serial_println!("[SYNC] DMA probe: click at byte {} reached after {} µs ({} ms)",
                click_byte_offset, latency, latency / 1000);
            return latency;
        }

        // Brief spin to avoid hammering the bus
        for _ in 0..100 {
            core::hint::spin_loop();
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Metronome Tap Calibration
// ═══════════════════════════════════════════════════════════════════════════════

/// Beat interval in microseconds
fn beat_interval_us() -> u64 {
    60_000_000 / METRONOME_BPM as u64  // 120 BPM → 500_000 µs
}

/// Generate a short click suitable for one metronome beat.
/// Returns stereo i16 samples at 48 kHz.
fn generate_click() -> Vec<i16> {
    crate::drivers::hda::generate_sine(CLICK_FREQUENCY_HZ, CLICK_DURATION_MOUSE, 20000)
}

/// Start a metronome tap calibration session.
///
/// This resets all state and begins playing clicks.
/// The GUI loop must call [`tick()`] every frame to advance beats,
/// and [`record_tap()`] when the user presses the sync key.
pub fn start_session() {
    // Reset
    SYNC_ACTIVE.store(true, Ordering::SeqCst);
    TAP_COUNT.store(0, Ordering::SeqCst);
    BEAT_INDEX.store(0, Ordering::SeqCst);
    RESULT_READY.store(false, Ordering::SeqCst);
    COMPUTED_OFFSET_MOUSE.store(0, Ordering::SeqCst);
    {
        let mut deltas = TAP_DELTAS.lock();
        deltas.clear();
    }

    let now = crate::gui::engine::now_us();
    SESSION_START_US.store(now, Ordering::SeqCst);
    LAST_BEAT_US.store(0, Ordering::SeqCst);

    crate::serial_println!("[SYNC] Metronome calibration started ({} BPM, {} taps needed)",
        METRONOME_BPM, TAP_SAMPLES_NEEDED);

    // Play the first click immediately
    play_click();
    LAST_BEAT_US.store(now, Ordering::SeqCst);
    BEAT_INDEX.store(1, Ordering::SeqCst);
}

/// Cancel an in-progress session.
pub fn cancel_session() {
    SYNC_ACTIVE.store(false, Ordering::SeqCst);
    let _ = crate::drivers::hda::stop();
    crate::serial_println!("[SYNC] Calibration cancelled");
}

/// Must be called each frame while a session is active.
/// Plays the next metronome click when it's time.
/// Returns `true` if the session is still running.
pub fn tick() -> bool {
    if !SYNC_ACTIVE.load(Ordering::SeqCst) {
        return false;
    }

    // Check if we've collected enough taps
    if TAP_COUNT.load(Ordering::SeqCst) as usize >= TAP_SAMPLES_NEEDED {
        finish_session();
        return false;
    }

    let now = crate::gui::engine::now_us();
    let session_start = SESSION_START_US.load(Ordering::SeqCst);
    let beat_index = BEAT_INDEX.load(Ordering::SeqCst);
    let expected_beat_us = session_start + beat_index as u64 * beat_interval_us();

    if now >= expected_beat_us {
        // Time for next click
        play_click();
        LAST_BEAT_US.store(expected_beat_us, Ordering::SeqCst);
        BEAT_INDEX.store(beat_index + 1, Ordering::SeqCst);
    }

    true
}

/// Record a user tap (call from keyboard handler when sync key is pressed).
/// The timestamp should be from `engine::now_us()`.
pub fn record_tap(tap_us: u64) {
    if !SYNC_ACTIVE.load(Ordering::SeqCst) {
        return;
    }

    // Find the nearest beat to this tap
    let session_start = SESSION_START_US.load(Ordering::SeqCst);
    let interval = beat_interval_us();

    if tap_us <= session_start {
        return;
    }

    // Which beat is closest?
    let elapsed = tap_us - session_start;
    let nearest_beat = ((elapsed + interval / 2) / interval) as u64;
    let nearest_beat_us = session_start + nearest_beat * interval;

    // Delta: positive = user tapped AFTER the beat played → audio perceived late
    // negative = user tapped BEFORE → unusual, probably early anticipation
    let delta_us = tap_us as i64 - nearest_beat_us as i64;

    {
        let mut deltas = TAP_DELTAS.lock();
        deltas.push(delta_us);
    }

    let count = TAP_COUNT.fetch_add(1, Ordering::SeqCst) + 1;
    crate::serial_println!("[SYNC] Tap {}/{}: delta = {} µs ({} ms)",
        count, TAP_SAMPLES_NEEDED, delta_us, delta_us / 1000);

    // Check if done
    if count as usize >= TAP_SAMPLES_NEEDED {
        finish_session();
    }
}

/// Play a single metronome click via HDA.
fn play_click() {
    let click = generate_click();
    if !click.is_empty() {
        // Use write_samples_and_play with the click duration
        // This is blocking for the click duration, but clicks are very short (30ms)
        let _ = crate::drivers::hda::write_samples_and_play(&click, CLICK_DURATION_MOUSE + 5);
    }
}

/// Compute the final offset from collected tap deltas.
fn finish_session() {
    SYNC_ACTIVE.store(false, Ordering::SeqCst);

    let deltas = {
        let d = TAP_DELTAS.lock();
        d.clone()
    };

    if deltas.is_empty() {
        crate::serial_println!("[SYNC] No taps recorded");
        return;
    }

    // Sort and take median (robust against outliers / early taps)
    let mut sorted = deltas.clone();
    sorted.sort();

    let median_us = if sorted.len() % 2 == 0 {
        (sorted[sorted.len() / 2 - 1] + sorted[sorted.len() / 2]) / 2
    } else {
        sorted[sorted.len() / 2]
    };

    // Also compute mean for reference
    let sum: i64 = sorted.iter().sum();
    let mean_us = sum / sorted.len() as i64;

    // Standard deviation for confidence (integer approximation, skip sqrt in no_std)
    let variance_us2: i64 = sorted.iter()
        .map(|&d| {
            let diff = d - mean_us;
            diff * diff
        })
        .sum::<i64>() / sorted.len() as i64;
    // Approximate stddev via integer sqrt (Newton's method)
    let stddev_us = isqrt_i64(variance_us2);
    let stddev_mouse = stddev_us / 1000;

    // Subtract a typical human reaction time (~80ms) to isolate audio latency.
    // This is approximate; the user will see the final value and can adjust.
    let human_reaction_us: i64 = 80_000;
    let audio_latency_us = median_us - human_reaction_us;
    let offset_mouse = (audio_latency_us / 1000).clamp(-MAXIMUM_OFFSET_MOUSE as i64, MAXIMUM_OFFSET_MOUSE as i64) as i32;

    COMPUTED_OFFSET_MOUSE.store(offset_mouse, Ordering::SeqCst);
    RESULT_READY.store(true, Ordering::SeqCst);

    crate::serial_println!("[SYNC] ──── Calibration Results ────");
    crate::serial_println!("[SYNC]   Taps collected: {}", sorted.len());
    crate::serial_println!("[SYNC]   Median tap delta: {} ms", median_us / 1000);
    crate::serial_println!("[SYNC]   Mean tap delta:   {} ms", mean_us / 1000);
    crate::serial_println!("[SYNC]   Std deviation:    {} ms", stddev_mouse);
    crate::serial_println!("[SYNC]   Human reaction:  -{} ms (subtracted)", human_reaction_us / 1000);
    crate::serial_println!("[SYNC]   → Computed A/V offset: {} ms", offset_mouse);

    // DMA latency for reference
    let dma_us = DMA_LATENCY_US.load(Ordering::SeqCst);
    if dma_us > 0 {
        crate::serial_println!("[SYNC]   DMA pipeline:    {} ms (measured)", dma_us / 1000);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Public query API
// ═══════════════════════════════════════════════════════════════════════════════

/// Is a calibration session currently active?
pub fn is_active() -> bool {
    SYNC_ACTIVE.load(Ordering::SeqCst)
}

/// Get the computed offset (valid only when `has_result()` is true).
pub fn get_offset_mouse() -> i32 {
    COMPUTED_OFFSET_MOUSE.load(Ordering::SeqCst)
}

/// Whether a result is available from the last session.
pub fn has_result() -> bool {
    RESULT_READY.load(Ordering::SeqCst)
}

/// Number of taps collected so far (0..TAP_SAMPLES_NEEDED).
pub fn taps_collected() -> u32 {
    TAP_COUNT.load(Ordering::SeqCst)
}

/// Total taps required.
pub fn taps_needed() -> u32 {
    TAP_SAMPLES_NEEDED as u32
}

/// Last measured DMA latency in µs (0 if not probed).
pub fn dma_latency_us() -> u64 {
    DMA_LATENCY_US.load(Ordering::SeqCst)
}

/// Integer square root (Newton's method). Returns floor(√n) for n ≥ 0.
fn isqrt_i64(n: i64) -> i64 {
    if n <= 0 { return 0; }
    let mut x = n;
    let mut y = (x + 1) / 2;
    while y < x {
        x = y;
        y = (x + n / x) / 2;
    }
    x
}
