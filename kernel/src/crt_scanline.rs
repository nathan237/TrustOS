// ═══════════════════════════════════════════════════════════════════════════
// CRT Progressive Scanline Engine
// ═══════════════════════════════════════════════════════════════════════════
//
// Single fast horizontal raster sweep — authentic CRT electron gun.
// Sweeps top→bottom in ~2 frames (imperceptible at 30fps).
// Characters under the beam get a subtle neutral-white phosphor boost.
// Fast decay behind the beam preserves natural rain appearance.
//
// All math is O(1) per query — single subtraction + range check.
// ═══════════════════════════════════════════════════════════════════════════


// ══════════════════════════════════════════
// State
// ══════════════════════════════════════════

pub struct CrtScanState {
    pub initialized: bool,
    pub w: f32,
    pub h: f32,
    pub frame: u64,
    // Vertical sweep position (pixels from top)
    pub sweep_pos: f32,
}

impl CrtScanState {
    pub const fn new() -> Self {
        Self {
            initialized: false,
            w: 0.0,
            h: 0.0,
            frame: 0,
            sweep_pos: 0.0,
        }
    }
}

// ══════════════════════════════════════════
// Query result
// ══════════════════════════════════════════

/// Effect returned per rain character. Applied as multiplier + additive color.
pub struct ScanEffect {
    /// Brightness multiplier (1.0 = unchanged, >1 = phosphor glow)
    pub brightness: f32,
    /// Additive color offsets (signed, per channel)
    pub color_r: i16,
    pub color_g: i16,
    pub color_b: i16,
}

impl ScanEffect {
    pub const NONE: Self = Self {
        brightness: 1.0,
        color_r: 0,
        color_g: 0,
        color_b: 0,
    };
}

// ══════════════════════════════════════════
// Constants
// ══════════════════════════════════════════

// Single fast raster — authentic CRT refresh (imperceptible sweep)
const SWEEP_FRAMES: f32 = 2.0;   // full screen in ~2 frames (~16ms at 60fps)
const BEAM_HALF: f32 = 3.0;      // narrow beam (like a real electron gun)
const PHOSPHOR_RANGE: f32 = 80.0; // short afterglow (fast decay)
const PRE_GLOW: f32 = 8.0;        // very subtle pre-glow ahead of beam

// ══════════════════════════════════════════
// Init / Update
// ══════════════════════════════════════════

pub fn init(s: &mut CrtScanState, w: u32, h: u32) {
    s.w = w as f32;
    s.h = h as f32;
    s.frame = 0;
    s.sweep_pos = -BEAM_HALF;
    s.initialized = true;
}

pub fn update(s: &mut CrtScanState) {
    if !s.initialized {
        return;
    }
    s.frame = s.frame.wrapping_add(1);

    // ── Single fast raster sweep (CRT-authentic) ──
    let sweep_speed = s.h / SWEEP_FRAMES;
    s.sweep_pos += sweep_speed;
    // Wrap: tight loop for near-instant refresh
    if s.sweep_pos > s.h + PHOSPHOR_RANGE + BEAM_HALF {
        s.sweep_pos = -(BEAM_HALF + PRE_GLOW);
    }
}

// ══════════════════════════════════════════
// Query — called per rain character
// ══════════════════════════════════════════

/// Query the scanline effect at pixel position (px, py).
/// Returns brightness multiplier + color offsets for the rain character.
pub fn query(s: &CrtScanState, _px: f32, py: f32) -> ScanEffect {
    if !s.initialized {
        return ScanEffect::NONE;
    }

    // Single horizontal raster — distance is just sweep_pos - py
    let dist = s.sweep_pos - py;
    phosphor(dist)
}

// ══════════════════════════════════════════
// Phosphor Decay Model
// ══════════════════════════════════════════

/// Convert signed distance-to-beam into phosphor glow effect.
///
/// Implements P31 green phosphor decay:
///   On beam  → white-hot flash (warm white-green, ×2.5)
///   Fresh    → bright phosphor green, rapid initial decay
///   Fading   → dim green, gradual falloff
///   Gone     → no effect (passthrough)
fn phosphor(dist: f32) -> ScanEffect {
    // ── Ahead of beam: very faint pre-glow ──
    if dist < -BEAM_HALF {
        let ahead = -dist - BEAM_HALF;
        if ahead < PRE_GLOW {
            let t = 1.0 - ahead / PRE_GLOW;
            let t2 = t * t;
            return ScanEffect {
                brightness: 1.0 + t2 * 0.06, // barely perceptible
                color_r: (t2 * 3.0) as i16,
                color_g: (t2 * 3.0) as i16,
                color_b: (t2 * 3.0) as i16,  // neutral white pre-glow
            };
        }
        return ScanEffect::NONE;
    }

    // ── On the active beam: subtle bright white flash ──
    if dist <= BEAM_HALF {
        let t = 1.0 - libm::fabsf(dist) / BEAM_HALF;
        let t2 = t * t;
        return ScanEffect {
            brightness: 1.10 + t2 * 0.15, // 1.10 → 1.25 at center (subtle)
            color_r: (8.0 + t2 * 10.0) as i16,   // neutral warm white
            color_g: (8.0 + t2 * 10.0) as i16,
            color_b: (8.0 + t2 * 10.0) as i16,
        };
    }

    // ── Behind beam: fast phosphor decay ──
    let age = dist - BEAM_HALF;
    if age > PHOSPHOR_RANGE {
        return ScanEffect::NONE;
    }

    let t = age / PHOSPHOR_RANGE; // 0.0 = just passed, 1.0 = faded
    let inv = 1.0 - t;
    let decay = inv * inv; // fast quadratic decay
    let brightness = 1.0 + decay * 0.08; // 1.08 → 1.0 (barely visible)

    // Neutral white afterglow — no green bias
    let glow = (decay * 5.0) as i16;
    ScanEffect {
        brightness,
        color_r: glow,
        color_g: glow,
        color_b: glow,
    }
}


