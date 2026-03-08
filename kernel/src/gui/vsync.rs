//! VSync Module — Smooth frame pacing for TrustOS desktop
//!
//! Provides:
//! - Adaptive frame timing with HLT-based waiting
//! - Frame budget tracking (how long each frame took)
//! - Automatic frame skip when overloaded
//! - Smooth FPS averaging over multiple frames
//! - Frame deadline enforcement to prevent stutter

use core::sync::atomic::{AtomicU64, AtomicBool, Ordering};

/// Target refresh rate in Hz
const TARGET_FPS: u64 = 60;

/// Frame period in microseconds: 1_000_000 / 60 = 16_666
const FRAME_PERIOD_US: u64 = 16_666;

/// Maximum frame time before we skip frames (33ms = 30fps minimum)
const MAX_FRAME_US: u64 = 33_333;

/// Number of frames to average for smooth FPS display
const FPS_WINDOW: usize = 16;

/// Frame deadline: the TSC timestamp when the next frame should begin
static NEXT_FRAME_DEADLINE: AtomicU64 = AtomicU64::new(0);

/// Accumulated frame times for averaging (ring buffer, in microseconds)
static FRAME_TIMES: [AtomicU64; FPS_WINDOW] = {
    const INIT: AtomicU64 = AtomicU64::new(16_666);
    [INIT; FPS_WINDOW]
};
static FRAME_IDX: AtomicU64 = AtomicU64::new(0);

/// Smoothed FPS (updated every frame)
static SMOOTH_FPS: AtomicU64 = AtomicU64::new(60);

/// Dropped frame counter
static DROPPED_FRAMES: AtomicU64 = AtomicU64::new(0);

/// Total frame counter
static TOTAL_FRAMES: AtomicU64 = AtomicU64::new(0);

/// Last frame's render time in microseconds (for budget display)
static LAST_RENDER_US: AtomicU64 = AtomicU64::new(0);

/// Whether vsync is enabled
static VSYNC_ENABLED: AtomicBool = AtomicBool::new(true);

/// Initialize vsync timing. Call once at desktop start.
pub fn init() {
    let now = super::engine::now_us();
    NEXT_FRAME_DEADLINE.store(now + FRAME_PERIOD_US, Ordering::SeqCst);
    DROPPED_FRAMES.store(0, Ordering::Relaxed);
    TOTAL_FRAMES.store(0, Ordering::Relaxed);
    FRAME_IDX.store(0, Ordering::Relaxed);
    crate::serial_println!("[VSYNC] Initialized: target {}fps ({}us/frame)", TARGET_FPS, FRAME_PERIOD_US);
}

/// Call at the START of each frame to get the frame-start timestamp.
#[inline]
pub fn frame_begin() -> u64 {
    super::engine::now_us()
}

/// Call at the END of each frame (after swap_buffers).
/// Records render time, enforces frame deadline, sleeps until next frame.
pub fn frame_end(frame_start_us: u64) {
    let now = super::engine::now_us();
    let render_time = now.saturating_sub(frame_start_us);
    LAST_RENDER_US.store(render_time, Ordering::Relaxed);
    
    // Record in ring buffer
    let idx = FRAME_IDX.fetch_add(1, Ordering::Relaxed) as usize % FPS_WINDOW;
    FRAME_TIMES[idx].store(render_time, Ordering::Relaxed);
    
    TOTAL_FRAMES.fetch_add(1, Ordering::Relaxed);
    
    if !VSYNC_ENABLED.load(Ordering::Relaxed) {
        // No waiting — free-run mode
        update_smooth_fps();
        return;
    }
    
    let deadline = NEXT_FRAME_DEADLINE.load(Ordering::Relaxed);
    
    if now >= deadline {
        // We missed the deadline — frame took too long
        DROPPED_FRAMES.fetch_add(1, Ordering::Relaxed);
        // Reset deadline to avoid cascading delays
        let new_deadline = now + FRAME_PERIOD_US;
        NEXT_FRAME_DEADLINE.store(new_deadline, Ordering::Relaxed);
    } else {
        // We have time left — sleep until deadline
        let remaining = deadline - now;
        adaptive_sleep(remaining);
        // Advance deadline by exactly one frame period (keeps cadence)
        NEXT_FRAME_DEADLINE.store(deadline + FRAME_PERIOD_US, Ordering::Relaxed);
    }
    
    update_smooth_fps();
}

/// Adaptive sleep: HLT loop for long waits, spin for short waits.
/// More precise than a single HLT — loops until the target time.
fn adaptive_sleep(target_us: u64) {
    let start = super::engine::now_us();
    let end = start + target_us;
    
    // Phase 1: HLT loop for waits > 2ms (each HLT wakes on timer IRQ ~1ms)
    while super::engine::now_us() + 2000 < end {
        unsafe {
            #[cfg(target_arch = "x86_64")]
            core::arch::asm!("sti; hlt", options(nomem, nostack));
            #[cfg(not(target_arch = "x86_64"))]
            {
                core::arch::asm!("wfe", options(nomem, nostack));
            }
        }
    }
    
    // Phase 2: Spin-wait for the last ~2ms (precise timing)
    while super::engine::now_us() < end {
        core::hint::spin_loop();
    }
}

/// Update smoothed FPS from ring buffer
fn update_smooth_fps() {
    let mut total: u64 = 0;
    for i in 0..FPS_WINDOW {
        total += FRAME_TIMES[i].load(Ordering::Relaxed);
    }
    let avg_us = total / FPS_WINDOW as u64;
    let fps = if avg_us > 0 { 1_000_000 / avg_us } else { 0 };
    SMOOTH_FPS.store(fps.min(999), Ordering::Relaxed);
}

/// Get smoothed FPS value
#[inline]
pub fn fps() -> u64 {
    SMOOTH_FPS.load(Ordering::Relaxed)
}

/// Get last frame's render time in microseconds
#[inline]
pub fn render_time_us() -> u64 {
    LAST_RENDER_US.load(Ordering::Relaxed)
}

/// Get render time as percentage of frame budget (0-100+)
#[inline]
pub fn frame_budget_pct() -> u64 {
    (LAST_RENDER_US.load(Ordering::Relaxed) * 100) / FRAME_PERIOD_US
}

/// Get total dropped frames since init
#[inline]
pub fn dropped_frames() -> u64 {
    DROPPED_FRAMES.load(Ordering::Relaxed)
}

/// Get total frames rendered since init
#[inline]
pub fn total_frames() -> u64 {
    TOTAL_FRAMES.load(Ordering::Relaxed)
}

/// Enable or disable vsync
pub fn set_enabled(enabled: bool) {
    VSYNC_ENABLED.store(enabled, Ordering::SeqCst);
    if enabled {
        // Reset deadline to now to avoid immediate frame-skip
        let now = super::engine::now_us();
        NEXT_FRAME_DEADLINE.store(now + FRAME_PERIOD_US, Ordering::SeqCst);
    }
}

/// Check if vsync is enabled
#[inline]
pub fn is_enabled() -> bool {
    VSYNC_ENABLED.load(Ordering::Relaxed)
}

/// Get frame period in microseconds
#[inline]
pub fn frame_period_us() -> u64 {
    FRAME_PERIOD_US
}
