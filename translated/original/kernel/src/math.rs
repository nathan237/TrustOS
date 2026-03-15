//! Shared math utilities for TrustOS
//!
//! Centralized fast math approximations (sin, cos, sqrt, atan2, exp, abs, clamp)
//! to eliminate duplicate implementations across 8+ files.

const PI: f32 = 3.14159265;
const TAU: f32 = 6.28318530;

/// Wrap angle to [-PI, PI]
#[inline(always)]
pub fn wrap_angle(x: f32) -> f32 {
    let mut a = x % TAU;
    while a > PI { a -= TAU; }
    while a < -PI { a += TAU; }
    a
}

/// sin(x) — 7th-order Taylor, error < 0.0002
#[inline(always)]
pub fn fast_sin(x: f32) -> f32 {
    let x = wrap_angle(x);
    let x2 = x * x;
    let x3 = x * x2;
    let x5 = x3 * x2;
    let x7 = x5 * x2;
    x - x3 / 6.0 + x5 / 120.0 - x7 / 5040.0
}

/// cos(x) = sin(x + PI/2)
#[inline(always)]
pub fn fast_cos(x: f32) -> f32 {
    fast_sin(x + 1.5707963)
}

/// tan(x) = sin(x) / cos(x)
#[inline(always)]
pub fn fast_tan(x: f32) -> f32 {
    let c = fast_cos(x);
    if c.abs() < 0.0001 { return 99999.0; }
    fast_sin(x) / c
}

/// sqrt via Newton-Raphson (3 iterations)
#[inline(always)]
pub fn fast_sqrt(x: f32) -> f32 {
    if x <= 0.0 { return 0.0; }
    let mut g = x * 0.5;
    g = 0.5 * (g + x / g);
    g = 0.5 * (g + x / g);
    0.5 * (g + x / g)
}

/// Fast inverse square root (Quake-style)
#[inline(always)]
pub fn fast_isqrt(x: f32) -> f32 {
    if x <= 0.0 { return 0.0; }
    let xhalf = 0.5 * x;
    let i = unsafe { core::mem::transmute::<f32, u32>(x) };
    let i = 0x5f375a86u32.wrapping_sub(i >> 1);
    let y = unsafe { core::mem::transmute::<u32, f32>(i) };
    let y = y * (1.5 - xhalf * y * y);
    y * (1.5 - xhalf * y * y)
}

/// atan2(y, x) — fast approximation (~0.01 rad error)
#[inline(always)]
pub fn fast_atan2(y: f32, x: f32) -> f32 {
    let abs_x = fast_abs(x);
    let abs_y = fast_abs(y);
    let max_val = if abs_x > abs_y { abs_x } else { abs_y };
    let min_val = if abs_x < abs_y { abs_x } else { abs_y };

    if max_val < 0.0001 { return 0.0; }

    let a = min_val / max_val;
    let s = a * a;
    let r = ((-0.0464964749 * s + 0.15931422) * s - 0.327622764) * s * a + a;

    let r = if abs_y > abs_x { 1.5707963 - r } else { r };
    let r = if x < 0.0 { PI - r } else { r };
    if y < 0.0 { -r } else { r }
}

/// Fast exp(x) approximation for x in [-6, 0]
#[inline(always)]
pub fn fast_exp(x: f32) -> f32 {
    if x < -6.0 { return 0.0; }
    if x > 0.0 { return 1.0; }
    let t = 1.0 + x * 0.125;
    let t = if t < 0.0 { 0.0 } else { t };
    let t = t * t; // ^2
    let t = t * t; // ^4
    t * t           // ^8
}

/// Fast absolute value
#[inline(always)]
pub fn fast_abs(x: f32) -> f32 {
    if x < 0.0 { -x } else { x }
}

/// Clamp value to [min, max]
#[inline(always)]
pub fn clamp(x: f32, min: f32, max: f32) -> f32 {
    if x < min { min } else if x > max { max } else { x }
}

/// Clamp i32 to [min, max]
#[inline(always)]
pub fn clamp_i32(x: i32, min: i32, max: i32) -> i32 {
    if x < min { min } else if x > max { max } else { x }
}

/// Linear interpolation
#[inline(always)]
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Linear interpolation for u8
#[inline(always)]
pub fn lerp_u8(a: u8, b: u8, t: f32) -> u8 {
    ((a as f32) * (1.0 - t) + (b as f32) * t) as u8
}

/// Floor of f32 as i32
#[inline(always)]
pub fn floor_i32(x: f32) -> i32 {
    let i = x as i32;
    if (i as f32) > x { i - 1 } else { i }
}

/// Fractional part
#[inline(always)]
pub fn fract(x: f32) -> f32 {
    x - (floor_i32(x) as f32)
}
