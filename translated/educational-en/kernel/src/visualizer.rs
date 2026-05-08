// ═══════════════════════════════════════════════════════════════════════════
// Music Visualizer — Multi-mode 3D wireframe shapes revealed by matrix rain
// ═══════════════════════════════════════════════════════════════════════════
//
// 6 selectable modes, all sharing the same projection + rain-modulation API:
//   0. Sphere         — Classic UV-sphere wireframe (original ghost mesh)
//   1. Morphing Poly  — Smooth transitions: icosahedron → cube → diamond → star
//   2. Lorenz         — Chaotic attractor with audio-driven parameters
//   3. Spectrum Globe  — Sphere with FFT-driven vertex displacement per band
//   4. Waveform Ribbon — 3D ribbon whose cross-section follows the waveform
//   5. Starburst      — Particle explosion on every beat
//
// Rain rendering interface (same as ghost_mesh):
//   update()                → advance animation, project, rasterize column hits
//   check_rain_collision()  → returns RainEffect per rain character
//   modulate_rain_color()   → applies RainEffect to base rain RGB
//   column_slow_factor()    → rain slows inside the shape
// ═══════════════════════════════════════════════════════════════════════════

extern crate alloc;
use alloc::vec::Vec;

// ═══════════════════════════════════════
// Data types
// ═══════════════════════════════════════

#[derive(Clone, Copy)]
struct V3 { x: f32, y: f32, z: f32 }

// Implementation block — defines methods for the type above.
impl V3 {
    const fn new(x: f32, y: f32, z: f32) -> Self { Self { x, y, z } }
}

// #[derive] — auto-generates trait implementations at compile time.
#[derive(Clone, Copy)]
struct Edge(u16, u16);

/// Rain interaction result — identical interface to ghost_mesh::RainEffect
#[derive(Clone, Copy)]
// Public structure — visible outside this module.
pub struct RainEffect {
    pub glow: u8,
    pub depth: u8,
    pub trail_boost: u8,
    pub ripple: u8,
    pub dim: u8,
    pub fresnel: u8,
    pub specular: u8,
    pub ao: u8,
    pub bloom: u8,
    pub scanline: u8,
    pub inner_glow: u8,
    pub shadow: u8,
    /// Image mode: target pixel color to blend toward
    pub target_r: u8,
    pub target_g: u8,
    pub target_b: u8,
    /// 0 = no image blend, 255 = fully image color
    pub target_blend: u8,
}

// Implementation block — defines methods for the type above.
impl RainEffect {
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const NONE: Self = Self {
        glow: 0, depth: 128, trail_boost: 0, ripple: 0, dim: 0,
        fresnel: 0, specular: 0, ao: 0, bloom: 0, scanline: 0,
        inner_glow: 0, shadow: 0,
        target_r: 0, target_g: 0, target_b: 0, target_blend: 0,
    };
}

// ═══════════════════════════════════════
// Visualizer mode enum
// ═══════════════════════════════════════

pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const NUMBER_MODES: u8 = 14;

pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const MODE_NAMES: [&str; 14] = [
    "Sphere",
    "Morphing",
    "Lorenz",
    "Spectrum",
    "Ribbon",
    "Starburst",
    "Image",
    "TorusKnot",
    "DNA Helix",
    "Tesseract",
    "Vortex",
    "PlasmaSphere",
    "Galaxy",
    "Subscribe",
];

// ═══════════════════════════════════════
// Color palettes
// ═══════════════════════════════════════

pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const NUMBER_PALETTES: u8 = 24;

pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const PALETTE_NAMES: [&str; 24] = [
    // ── Base single-color (0-12) ──
    "Matrix",     // 0  green
    "Cyber",      // 1  cyan/magenta
    "Fire",       // 2  red/orange/yellow
    "Ocean",      // 3  blue/teal
    "Aurora",     // 4  green/purple
    "Gold",       // 5  gold/amber
    "Red",        // 6
    "Blue",       // 7
    "Purple",     // 8
    "Pink",       // 9
    "Yellow",     // 10
    "Cyan",       // 11
    "White",      // 12
    // ── Multi-color combos (13-22) ──
    "Rainbow",    // 13
    "Neon Mix",   // 14  Cyber + Fire + Aurora
    "Lava Sea",   // 15  Fire + Ocean + Gold
    "Prism",      // 16  Cyber + Aurora + Ocean
    "Sunset",     // 17  Red + Gold + Purple
    "Arctic",     // 18  Cyan + Blue + White
    "Toxic",      // 19  Matrix + Yellow + Cyan
    "Vampire",    // 20  Red + Purple + Pink
    "Nebula",     // 21  Blue + Purple + Pink
    "Inferno",    // 22  Red + Fire + Yellow
    // ── Special ──
    "Random",     // 23  random color per character
];

/// Returns (r, g, b) target colors for a palette given frequency band dominance
/// t = 0.0..1.0 interpolation within the palette gradient
pub fn palette_color(palette: u8, t: f32) -> (f32, f32, f32) {
    let t = t.max(0.0).min(1.0);
        // Pattern matching — Rust's exhaustive branching construct.
match palette {
        0 => { // Matrix — classic green
            let r = 20.0 + t * 80.0;
            let g = 120.0 + t * 135.0;
            let b = 20.0 + t * 60.0;
            (r, g, b)
        }
        1 => { // Cyber — cyan / magenta / purple
            if t < 0.33 {
                let s = t / 0.33;
                (20.0 + s * 60.0, 180.0 + s * 75.0, 255.0) // deep cyan → bright cyan
            } else if t < 0.66 {
                let s = (t - 0.33) / 0.33;
                (80.0 + s * 175.0, 255.0 - s * 175.0, 255.0 - s * 55.0) // cyan → magenta
            } else {
                let s = (t - 0.66) / 0.34;
                (255.0, 80.0 - s * 40.0, 200.0 + s * 55.0) // magenta → pink-white
            }
        }
        2 => { // Fire — red / orange / yellow
            if t < 0.4 {
                let s = t / 0.4;
                (180.0 + s * 75.0, 20.0 + s * 60.0, 0.0) // dark red → orange-red
            } else if t < 0.75 {
                let s = (t - 0.4) / 0.35;
                (255.0, 80.0 + s * 120.0, s * 30.0) // orange-red → yellow
            } else {
                let s = (t - 0.75) / 0.25;
                (255.0, 200.0 + s * 55.0, 30.0 + s * 200.0) // yellow → white-hot
            }
        }
        3 => { // Ocean — deep blue / teal / aqua
            if t < 0.5 {
                let s = t / 0.5;
                (0.0, 30.0 + s * 100.0, 120.0 + s * 135.0) // deep blue → teal
            } else {
                let s = (t - 0.5) / 0.5;
                (s * 80.0, 130.0 + s * 125.0, 255.0) // teal → bright aqua
            }
        }
        4 => { // Aurora — green / purple / pink
            if t < 0.33 {
                let s = t / 0.33;
                (30.0 + s * 40.0, 200.0 + s * 55.0, 80.0 + s * 40.0) // green
            } else if t < 0.66 {
                let s = (t - 0.33) / 0.33;
                (70.0 + s * 100.0, 255.0 - s * 155.0, 120.0 + s * 80.0) // green → purple
            } else {
                let s = (t - 0.66) / 0.34;
                (170.0 + s * 85.0, 100.0 + s * 50.0, 200.0 + s * 55.0) // purple → pink
            }
        }
        5 => { // Gold — gold / amber / white
            if t < 0.5 {
                let s = t / 0.5;
                (180.0 + s * 75.0, 130.0 + s * 50.0, 10.0 + s * 20.0)
            } else {
                let s = (t - 0.5) / 0.5;
                (255.0, 180.0 + s * 75.0, 30.0 + s * 225.0)
            }
        }
        6 => { // Red — dark red → crimson → pinkish white
            if t < 0.4 {
                let s = t / 0.4;
                (100.0 + s * 100.0, 5.0 + s * 15.0, 5.0 + s * 10.0)
            } else if t < 0.75 {
                let s = (t - 0.4) / 0.35;
                (200.0 + s * 55.0, 20.0 + s * 40.0, 15.0 + s * 30.0)
            } else {
                let s = (t - 0.75) / 0.25;
                (255.0, 60.0 + s * 160.0, 45.0 + s * 180.0)
            }
        }
        7 => { // Blue — navy → electric blue → ice
            if t < 0.4 {
                let s = t / 0.4;
                (5.0 + s * 15.0, 10.0 + s * 40.0, 100.0 + s * 100.0)
            } else if t < 0.75 {
                let s = (t - 0.4) / 0.35;
                (20.0 + s * 40.0, 50.0 + s * 80.0, 200.0 + s * 55.0)
            } else {
                let s = (t - 0.75) / 0.25;
                (60.0 + s * 160.0, 130.0 + s * 125.0, 255.0)
            }
        }
        8 => { // Purple — deep purple → violet → lavender
            if t < 0.4 {
                let s = t / 0.4;
                (60.0 + s * 50.0, 5.0 + s * 15.0, 100.0 + s * 60.0)
            } else if t < 0.75 {
                let s = (t - 0.4) / 0.35;
                (110.0 + s * 60.0, 20.0 + s * 40.0, 160.0 + s * 60.0)
            } else {
                let s = (t - 0.75) / 0.25;
                (170.0 + s * 70.0, 60.0 + s * 140.0, 220.0 + s * 35.0)
            }
        }
        9 => { // Pink — deep magenta → hot pink → light pink
            if t < 0.4 {
                let s = t / 0.4;
                (140.0 + s * 60.0, 10.0 + s * 20.0, 80.0 + s * 40.0)
            } else if t < 0.75 {
                let s = (t - 0.4) / 0.35;
                (200.0 + s * 55.0, 30.0 + s * 50.0, 120.0 + s * 40.0)
            } else {
                let s = (t - 0.75) / 0.25;
                (255.0, 80.0 + s * 130.0, 160.0 + s * 80.0)
            }
        }
        10 => { // Yellow — dark amber → bright yellow → white-yellow
            if t < 0.4 {
                let s = t / 0.4;
                (140.0 + s * 60.0, 100.0 + s * 50.0, 5.0 + s * 10.0)
            } else if t < 0.75 {
                let s = (t - 0.4) / 0.35;
                (200.0 + s * 55.0, 150.0 + s * 75.0, 15.0 + s * 20.0)
            } else {
                let s = (t - 0.75) / 0.25;
                (255.0, 225.0 + s * 30.0, 35.0 + s * 200.0)
            }
        }
        11 => { // Cyan — deep teal → bright cyan → ice-white
            if t < 0.4 {
                let s = t / 0.4;
                (5.0 + s * 10.0, 80.0 + s * 70.0, 100.0 + s * 60.0)
            } else if t < 0.75 {
                let s = (t - 0.4) / 0.35;
                (15.0 + s * 30.0, 150.0 + s * 80.0, 160.0 + s * 95.0)
            } else {
                let s = (t - 0.75) / 0.25;
                (45.0 + s * 180.0, 230.0 + s * 25.0, 255.0)
            }
        }
        _ => { // White (12) — dark grey → silver → bright white
            if t < 0.4 {
                let s = t / 0.4;
                let v = 60.0 + s * 60.0;
                (v, v, v)
            } else if t < 0.75 {
                let s = (t - 0.4) / 0.35;
                let v = 120.0 + s * 80.0;
                (v, v + s * 10.0, v + s * 15.0) // slight cool tint
            } else {
                let s = (t - 0.75) / 0.25;
                let v = 200.0 + s * 55.0;
                (v, v, v)
            }
        }
    }
}

/// Combo palette: blends 3 base palettes across the gradient
/// Each palette occupies 1/3 of the t range with smooth crossfade
fn combo_palette(p1: u8, p2: u8, p3: u8, t: f32) -> (f32, f32, f32) {
    let t = t.max(0.0).min(1.0);
    if t < 0.3 {
        // Pure p1
        palette_color(p1, t / 0.3)
    } else if t < 0.45 {
        // Crossfade p1 → p2
        let s = (t - 0.3) / 0.15;
        let (r1, g1, b1) = palette_color(p1, 0.8 + s * 0.2);
        let (r2, g2, b2) = palette_color(p2, s * 0.2);
        (r1 * (1.0 - s) + r2 * s, g1 * (1.0 - s) + g2 * s, b1 * (1.0 - s) + b2 * s)
    } else if t < 0.65 {
        // Pure p2
        let s = (t - 0.45) / 0.2;
        palette_color(p2, s)
    } else if t < 0.8 {
        // Crossfade p2 → p3
        let s = (t - 0.65) / 0.15;
        let (r1, g1, b1) = palette_color(p2, 0.8 + s * 0.2);
        let (r2, g2, b2) = palette_color(p3, s * 0.2);
        (r1 * (1.0 - s) + r2 * s, g1 * (1.0 - s) + g2 * s, b1 * (1.0 - s) + b2 * s)
    } else {
        // Pure p3
        let s = (t - 0.8) / 0.2;
        palette_color(p3, s)
    }
}

/// Rainbow: full HSV color wheel across t=0..1
fn rainbow_color(t: f32) -> (f32, f32, f32) {
    let t = t.max(0.0).min(1.0);
    // HSV to RGB: hue = t * 360°, full saturation, full value
    let h = t * 6.0; // 0..6 (6 sextants)
    let i = h as u8;
    let f = h - i as f32;
        // Pattern matching — Rust's exhaustive branching construct.
match i {
        0 => (255.0, f * 255.0, 0.0),                    // red → yellow
        1 => (255.0 * (1.0 - f), 255.0, 0.0),            // yellow → green
        2 => (0.0, 255.0, f * 255.0),                     // green → cyan
        3 => (0.0, 255.0 * (1.0 - f), 255.0),            // cyan → blue
        4 => (f * 255.0, 0.0, 255.0),                     // blue → magenta
        _ => (255.0, 0.0, 255.0 * (1.0 - f)),            // magenta → red
    }
}

/// Random: hash t into a pseudo-random vivid color (high saturation)
fn random_color(t: f32) -> (f32, f32, f32) {
    // Use t bits as a hash seed for a pseudo-random hue
    let bits = t.to_bits();
    let hash = bits.wrapping_mul(2654435761); // Knuth multiplicative hash
    let hue = (hash % 360) as f32 / 360.0;
    rainbow_color(hue)
}

/// Public random color for rain characters — uses col+row+frame as seed
pub fn rain_random_color(col: usize, row: usize, frame: u32) -> (u8, u8, u8) {
    let seed = (col as u32).wrapping_mul(2654435761)
        ^ (row as u32).wrapping_mul(1103515245)
        ^ frame.wrapping_mul(214013).wrapping_add(2531011);
    let hue = (seed % 360) as f32 / 360.0;
    let (r, g, b) = rainbow_color(hue);
    (r.min(255.0) as u8, g.min(255.0) as u8, b.min(255.0) as u8)
}

/// Get color for any palette index (base 0-12, multi 13-23)
pub fn get_palette_color(palette: u8, t: f32) -> (f32, f32, f32) {
        // Pattern matching — Rust's exhaustive branching construct.
match palette {
        0..=12 => palette_color(palette, t),
        13 => rainbow_color(t),                     // Rainbow
        14 => combo_palette(1, 2, 4, t),            // Neon Mix: Cyber + Fire + Aurora
        15 => combo_palette(2, 3, 5, t),            // Lava Sea: Fire + Ocean + Gold
        16 => combo_palette(1, 4, 3, t),            // Prism: Cyber + Aurora + Ocean
        17 => combo_palette(6, 5, 8, t),            // Sunset: Red + Gold + Purple
        18 => combo_palette(11, 7, 12, t),          // Arctic: Cyan + Blue + White
        19 => combo_palette(0, 10, 11, t),          // Toxic: Matrix + Yellow + Cyan
        20 => combo_palette(6, 8, 9, t),            // Vampire: Red + Purple + Pink
        21 => combo_palette(7, 8, 9, t),            // Nebula: Blue + Purple + Pink
        22 => combo_palette(6, 2, 10, t),           // Inferno: Red + Fire + Yellow
        _  => random_color(t),                      // Random: per-character random color
    }
}

// ═══════════════════════════════════════
// State
// ═══════════════════════════════════════

pub struct VisualizerState {
    // ── Shape mode ──
    pub mode: u8,

    // ── Shared geometry buffers ──
    verts: Vec<V3>,           // current frame vertices (world space)
    edges: Vec<Edge>,         // edge index pairs
    pverts: Vec<(i32, i32)>,  // projected 2D coords
    proj_z: Vec<i16>,         // Z depth per vertex

    // ── Projection ──
    rot_x: i32, rot_y: i32, rot_z: i32,
    rot_speed_x: i32, rot_speed_y: i32, rot_speed_z: i32,
    scale: i32, scale_target: i32,
    center_x: i32, center_y: i32,

    // ── Column hit map (same structure as ghost_mesh) ──
    pub column_hits: Vec<Vec<(i32, i32, u16, u8, i16)>>,
    pub column_bounds: Vec<(i32, i32)>,

    // ── Audio state ──
    pub z_min: i16, pub z_max: i16,
    pub col_w: i32,
    pub frame: u64,
    pub smooth_sub_bass: f32,
    pub smooth_bass: f32,
    pub smooth_mid: f32,
    pub smooth_treble: f32,
    pub beat_pulse: f32,
    pub ripple_radius: f32,
    pub spec_x: i32, pub spec_y: i32,
    pub shadow_y_start: i32, pub shadow_y_end: i32,
    pub shape_center_y: i32,

    // ── Mode 0: Sphere ──
    sphere_base: Vec<V3>,
    sphere_edges: Vec<Edge>,
    sphere_bands: Vec<u8>,

    // ── Mode 1: Morphing Polyhedra ──
    morph_shapes: [Vec<V3>; 4],     // target shapes: icosa, cube, diamond, star
    morph_edges_all: [Vec<Edge>; 4],
    morph_current: Vec<V3>,
    morph_phase: f32,               // 0.0–4.0 cycling through shapes
    morph_idx: usize,               // current target index

    // ── Mode 2: Lorenz Attractor ──
    lorenz_trail: Vec<V3>,          // trail points
    lorenz_state: [f32; 3],         // x,y,z state

    // ── Mode 3: Spectrum Globe ──
    spec_base: Vec<V3>,
    spec_bands: Vec<u8>,
    spec_edges: Vec<Edge>,

    // ── Mode 4: Waveform Ribbon ──
    ribbon_spine: Vec<V3>,

    // ── Mode 5: Starburst ──
    particles: Vec<Particle>,
    particle_timer: u32,

    // ── Mode 6: Image Reveal ──
    image_offset_x: i32,
    image_offset_y: i32,
    image_display_w: u32,
    image_display_h: u32,

    // ── Mode 7: Torus Knot ──
    torus_knot_base: Vec<V3>,
    torus_knot_edges: Vec<Edge>,

    // ── Mode 8: DNA Helix ──
    dna_base: Vec<V3>,
    dna_edges: Vec<Edge>,

    // ── Mode 9: Tesseract (4D Hypercube) ──
    tesseract_base: Vec<V3>,   // 16 verts of 4D cube projected to 3D
    tesseract_edges: Vec<Edge>,
    tesseract_w_angle: f32,    // 4th-dimension rotation angle

    // ── Mode 10: Vortex Tunnel ──
    vortex_base: Vec<V3>,
    vortex_edges: Vec<Edge>,

    // ── Mode 11: Plasma Sphere ──
    plasma_base: Vec<V3>,
    plasma_edges: Vec<Edge>,
    plasma_tendrils: Vec<V3>,
    plasma_tendril_edges: Vec<Edge>,

    // ── Mode 12: Galaxy Spiral ──
    galaxy_base: Vec<V3>,
    galaxy_edges: Vec<Edge>,

    // ── Mode 13: Subscribe DVD Bounce ──
    dvd_x: f32,           // bounce position X (pixels)
    dvd_y: f32,           // bounce position Y (pixels)
    dvd_vx: f32,          // velocity X (pixels/frame)
    dvd_vy: f32,          // velocity Y (pixels/frame)
    dvd_flash: f32,       // red flash intensity (0.0-1.0)
    subscribe_base: Vec<V3>,
    subscribe_edges: Vec<Edge>,

    // ── Shared: color palette ──
    pub palette: u8,

    pub initialized: bool,
}

struct Particle {
    x: f32, y: f32, z: f32,
    vx: f32, vy: f32, vz: f32,
    life: f32,
}

// Implementation block — defines methods for the type above.
impl VisualizerState {
    pub const fn new() -> Self {
        Self {
            mode: 9,
            verts: Vec::new(), edges: Vec::new(),
            pverts: Vec::new(), proj_z: Vec::new(),
            rot_x: 0, rot_y: 0, rot_z: 0,
            rot_speed_x: 6, rot_speed_y: 10, rot_speed_z: 2,
            scale: 180, scale_target: 180,
            center_x: 0, center_y: 0,
            column_hits: Vec::new(), column_bounds: Vec::new(),
            z_min: 0, z_max: 0, col_w: 8,
            frame: 0,
            smooth_sub_bass: 0.0, smooth_bass: 0.0,
            smooth_mid: 0.0, smooth_treble: 0.0,
            beat_pulse: 0.0, ripple_radius: 999.0,
            spec_x: 0, spec_y: 0,
            shadow_y_start: 0, shadow_y_end: 0, shape_center_y: 0,
            sphere_base: Vec::new(), sphere_edges: Vec::new(), sphere_bands: Vec::new(),
            morph_shapes: [Vec::new(), Vec::new(), Vec::new(), Vec::new()],
            morph_edges_all: [Vec::new(), Vec::new(), Vec::new(), Vec::new()],
            morph_current: Vec::new(),
            morph_phase: 0.0, morph_idx: 0,
            lorenz_trail: Vec::new(), lorenz_state: [1.0, 1.0, 1.0],
            spec_base: Vec::new(), spec_bands: Vec::new(), spec_edges: Vec::new(),
            ribbon_spine: Vec::new(),
            particles: Vec::new(), particle_timer: 0,
            image_offset_x: 0, image_offset_y: 0,
            image_display_w: 0, image_display_h: 0,
            torus_knot_base: Vec::new(), torus_knot_edges: Vec::new(),
            dna_base: Vec::new(), dna_edges: Vec::new(),
            tesseract_base: Vec::new(), tesseract_edges: Vec::new(), tesseract_w_angle: 0.0,
            vortex_base: Vec::new(), vortex_edges: Vec::new(),
            plasma_base: Vec::new(), plasma_edges: Vec::new(),
            plasma_tendrils: Vec::new(), plasma_tendril_edges: Vec::new(),
            galaxy_base: Vec::new(), galaxy_edges: Vec::new(),
            dvd_x: 200.0, dvd_y: 150.0, dvd_vx: 1.8, dvd_vy: 1.2,
            dvd_flash: 0.0,
            subscribe_base: Vec::new(), subscribe_edges: Vec::new(),
            palette: 0,
            initialized: false,
        }
    }
}

// ═══════════════════════════════════════
// Math helpers (integer trig from ghost_mesh)
// ═══════════════════════════════════════

fn sin_i(mrad: i32) -> i32 {
    let two_pi: i32 = 6283;
    let mut a = mrad % two_pi;
    if a < 0 { a += two_pi; }
    let pi_i: i32 = 3141;
    let (a_n, sign) = if a > pi_i { (a - pi_i, -1i32) } else { (a, 1) };
    let p = pi_i as i64;
    let x = a_n as i64;
    let xp = x * (p - x);
    let den = 5 * p * p - 4 * xp;
    if den == 0 { return 0; }
    (16 * xp * 1000 / den) as i32 * sign
}

fn cos_i(mrad: i32) -> i32 { sin_i(mrad + 1571) }
fn sinf(r: f32) -> f32 { sin_i((r * 1000.0) as i32) as f32 / 1000.0 }
fn cosf(r: f32) -> f32 { cos_i((r * 1000.0) as i32) as f32 / 1000.0 }

fn transform_vertex(v: V3, rx: i32, ry: i32, rz: i32, s: i32) -> (i32, i32, i32) {
    let vx = (v.x * s as f32) as i32;
    let vy = (v.y * s as f32) as i32;
    let vz = (v.z * s as f32) as i32;
    let (sx, cx) = (sin_i(rx), cos_i(rx));
    let y1 = (vy * cx - vz * sx) / 1000;
    let z1 = (vy * sx + vz * cx) / 1000;
    let (sy, cy) = (sin_i(ry), cos_i(ry));
    let x2 = (vx * cy + z1 * sy) / 1000;
    let z2 = (-vx * sy + z1 * cy) / 1000;
    let (sz, cz) = (sin_i(rz), cos_i(rz));
    let x3 = (x2 * cz - y1 * sz) / 1000;
    let y3 = (x2 * sz + y1 * cz) / 1000;
    (x3, y3, z2)
}

fn project(x: i32, y: i32, z: i32, cx: i32, cy: i32) -> (i32, i32) {
    let d: i32 = 600;
    let den = d + z;
    if den <= 10 { return (cx, cy); }
    (cx + x * d / den, cy - y * d / den)
}

fn lerp(a: f32, b: f32, t: f32) -> f32 { a + (b - a) * t }

// ═══════════════════════════════════════
// Shape generators
// ═══════════════════════════════════════

const SPHERE_LAT: usize = 8;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const SPHERE_LON: usize = 12;

fn generator_sphere() -> (Vec<V3>, Vec<Edge>, Vec<u8>) {
    let mut verts = Vec::with_capacity(SPHERE_LAT * SPHERE_LON + 2);
    let mut bands = Vec::with_capacity(SPHERE_LAT * SPHERE_LON + 2);
    let mut edges = Vec::new();
    let pi: f32 = 3.14159265;
    verts.push(V3::new(0.0, -1.0, 0.0)); bands.push(0);
    for lat in 0..SPHERE_LAT {
        let frac = (lat as f32 + 1.0) / (SPHERE_LAT as f32 + 1.0);
        let angle = -pi / 2.0 + pi * frac;
        let y = sinf(angle);
        let r = cosf(angle);
        let band: u8 = // Pattern matching — Rust's exhaustive branching construct.
match lat { 0 => 0, 1 => 1, 2..=5 => 2, _ => 3 };
        for lon in 0..SPHERE_LON {
            let la = 2.0 * pi * (lon as f32) / (SPHERE_LON as f32);
            verts.push(V3::new(r * cosf(la), y, r * sinf(la)));
            bands.push(band);
        }
    }
    verts.push(V3::new(0.0, 1.0, 0.0)); bands.push(3);
    let np = verts.len() - 1;
    for lat in 0..SPHERE_LAT {
        let base = 1 + lat * SPHERE_LON;
        for lon in 0..SPHERE_LON {
            let next = if lon + 1 < SPHERE_LON { lon + 1 } else { 0 };
            edges.push(Edge((base + lon) as u16, (base + next) as u16));
        }
    }
    for lon in 0..SPHERE_LON {
        edges.push(Edge(0, (1 + lon) as u16));
        for lat in 0..(SPHERE_LAT - 1) {
            let a = 1 + lat * SPHERE_LON + lon;
            let b = 1 + (lat + 1) * SPHERE_LON + lon;
            edges.push(Edge(a as u16, b as u16));
        }
        edges.push(Edge((1 + (SPHERE_LAT - 1) * SPHERE_LON + lon) as u16, np as u16));
    }
    (verts, edges, bands)
}

/// Icosahedron (12 vertices, 30 edges)
fn generator_icosahedron() -> (Vec<V3>, Vec<Edge>) {
    let phi: f32 = 1.618034; // golden ratio
    let s = 1.0 / libm::sqrtf(1.0 + phi * phi);
    let l = phi * s;
    let verts = alloc::vec![
        V3::new(-s,  l, 0.0), V3::new( s,  l, 0.0),
        V3::new(-s, -l, 0.0), V3::new( s, -l, 0.0),
        V3::new(0.0, -s,  l), V3::new(0.0,  s,  l),
        V3::new(0.0, -s, -l), V3::new(0.0,  s, -l),
        V3::new( l, 0.0, -s), V3::new( l, 0.0,  s),
        V3::new(-l, 0.0, -s), V3::new(-l, 0.0,  s),
    ];
    #[allow(clippy::identity_op)]
    let edges = alloc::vec![
        Edge(0,1), Edge(0,5), Edge(0,7), Edge(0,10), Edge(0,11),
        Edge(1,5), Edge(1,7), Edge(1,8), Edge(1,9),
        Edge(2,3), Edge(2,4), Edge(2,6), Edge(2,10), Edge(2,11),
        Edge(3,4), Edge(3,6), Edge(3,8), Edge(3,9),
        Edge(4,5), Edge(4,9), Edge(4,11),
        Edge(5,9), Edge(5,11),
        Edge(6,7), Edge(6,8), Edge(6,10),
        Edge(7,8), Edge(7,10),
        Edge(8,9), Edge(10,11),
    ];
    (verts, edges)
}

/// Cube (8 vertices, 12 edges)
fn generator_cube() -> (Vec<V3>, Vec<Edge>) {
    let s: f32 = 0.82;
    let verts = alloc::vec![
        V3::new(-s, -s, -s), V3::new( s, -s, -s),
        V3::new( s,  s, -s), V3::new(-s,  s, -s),
        V3::new(-s, -s,  s), V3::new( s, -s,  s),
        V3::new( s,  s,  s), V3::new(-s,  s,  s),
    ];
    let edges = alloc::vec![
        Edge(0,1), Edge(1,2), Edge(2,3), Edge(3,0),
        Edge(4,5), Edge(5,6), Edge(6,7), Edge(7,4),
        Edge(0,4), Edge(1,5), Edge(2,6), Edge(3,7),
    ];
    (verts, edges)
}

/// Diamond / Octahedron (6 vertices, 12 edges)
fn generator_diamond() -> (Vec<V3>, Vec<Edge>) {
    let verts = alloc::vec![
        V3::new(0.0,  1.2, 0.0),  // top
        V3::new(0.0, -1.2, 0.0),  // bottom
        V3::new( 1.0, 0.0, 0.0),  // right
        V3::new(-1.0, 0.0, 0.0),  // left
        V3::new(0.0, 0.0,  1.0),  // front
        V3::new(0.0, 0.0, -1.0),  // back
    ];
    let edges = alloc::vec![
        Edge(0,2), Edge(0,3), Edge(0,4), Edge(0,5),
        Edge(1,2), Edge(1,3), Edge(1,4), Edge(1,5),
        Edge(2,4), Edge(4,3), Edge(3,5), Edge(5,2),
    ];
    (verts, edges)
}

/// Star / Stellated octahedron (14 vertices, 36 edges)
fn generator_star() -> (Vec<V3>, Vec<Edge>) {
    // 6 inner (octahedron) + 8 outer spikes
    let mut verts = Vec::with_capacity(14);
    // Inner octahedron vertices
    let s: f32 = 0.55;
    verts.push(V3::new(0.0,  s, 0.0));   // 0 top
    verts.push(V3::new(0.0, -s, 0.0));   // 1 bottom
    verts.push(V3::new( s, 0.0, 0.0));   // 2 right
    verts.push(V3::new(-s, 0.0, 0.0));   // 3 left
    verts.push(V3::new(0.0, 0.0,  s));   // 4 front
    verts.push(V3::new(0.0, 0.0, -s));   // 5 back
    // 8 outer spike tips (cube corners, extended)
    let t: f32 = 1.1;
    verts.push(V3::new( t,  t,  t));     // 6
    verts.push(V3::new(-t,  t,  t));     // 7
    verts.push(V3::new( t, -t,  t));     // 8
    verts.push(V3::new(-t, -t,  t));     // 9
    verts.push(V3::new( t,  t, -t));     // 10
    verts.push(V3::new(-t,  t, -t));     // 11
    verts.push(V3::new( t, -t, -t));     // 12
    verts.push(V3::new(-t, -t, -t));     // 13
    let mut edges = Vec::new();
    // Inner edges
    edges.push(Edge(0,2)); edges.push(Edge(0,3)); edges.push(Edge(0,4)); edges.push(Edge(0,5));
    edges.push(Edge(1,2)); edges.push(Edge(1,3)); edges.push(Edge(1,4)); edges.push(Edge(1,5));
    edges.push(Edge(2,4)); edges.push(Edge(4,3)); edges.push(Edge(3,5)); edges.push(Edge(5,2));
    // Spikes from inner to outer
    for spike in 6..14u16 {
        // Connect each spike to 3 nearest inner vertices
        let sv = verts[spike as usize];
        let mut dists: Vec<(u16, f32)> = (0..6u16).map(|i| {
            let iv = verts[i as usize];
            let dx = sv.x - iv.x; let dy = sv.y - iv.y; let dz = sv.z - iv.z;
            (i, dx*dx + dy*dy + dz*dz)
        }).collect();
        dists.sort_by(|a, b| {
            if a.1 < b.1 { core::cmp::Ordering::Less }
            else if a.1 > b.1 { core::cmp::Ordering::Greater }
            else { core::cmp::Ordering::Equal }
        });
        for k in 0..3 {
            edges.push(Edge(spike, dists[k].0));
        }
    }
    (verts, edges)
}

// ═══════════════════════════════════════
// New 3D shape generators
// ═══════════════════════════════════════

/// Torus Knot — parametric trefoil knot on a torus
fn generator_torus_knot(p_k: u32, q_k: u32, segments: usize) -> (Vec<V3>, Vec<Edge>) {
    let mut verts = Vec::with_capacity(segments);
    let mut edges = Vec::new();
    let pi2: f32 = 6.28318;
    let r_major: f32 = 0.7; // torus major radius
    let r_minor: f32 = 0.3; // tube radius

    for i in 0..segments {
        let t = pi2 * i as f32 / segments as f32;
        let r = r_major + r_minor * cosf(q_k as f32 * t);
        let x = r * cosf(p_k as f32 * t);
        let y = r * sinf(p_k as f32 * t);
        let z = r_minor * sinf(q_k as f32 * t);
        verts.push(V3::new(x, y, z));
    }
    // Connect sequential + wrap
    for i in 0..segments {
        let next = (i + 1) % segments;
        edges.push(Edge(i as u16, next as u16));
    }
    // Cross-connections for web effect
    for i in (0..segments).step_by(4) {
        let cross = (i + segments / 3) % segments;
        edges.push(Edge(i as u16, cross as u16));
    }
    (verts, edges)
}

/// DNA Double Helix
fn generator_dna_helix(segments: usize, turns: f32) -> (Vec<V3>, Vec<Edge>) {
    // Two helices offset by pi, connected by rungs
    let mut verts = Vec::with_capacity(segments * 2);
    let mut edges = Vec::new();
    let pi2: f32 = 6.28318;
    let helix_r: f32 = 0.5;
    let height: f32 = 2.0;

    // Strand A
    for i in 0..segments {
        let t = i as f32 / segments as f32;
        let angle = pi2 * turns * t;
        let y = -height / 2.0 + height * t;
        verts.push(V3::new(helix_r * cosf(angle), y, helix_r * sinf(angle)));
    }
    // Strand B (offset by pi)
    for i in 0..segments {
        let t = i as f32 / segments as f32;
        let angle = pi2 * turns * t + 3.14159;
        let y = -height / 2.0 + height * t;
        verts.push(V3::new(helix_r * cosf(angle), y, helix_r * sinf(angle)));
    }
    // Helix edges
    for i in 0..(segments - 1) {
        edges.push(Edge(i as u16, (i + 1) as u16)); // strand A
        edges.push(Edge((segments + i) as u16, (segments + i + 1) as u16)); // strand B
    }
    // Rungs connecting the two strands
    for i in (0..segments).step_by(3) {
        edges.push(Edge(i as u16, (segments + i) as u16));
    }
    (verts, edges)
}

/// 4D Hypercube (Tesseract) — 16 vertices in 4D, projected to 3D
fn generator_tesseract() -> (Vec<[f32; 4]>, Vec<Edge>) {
    // 16 vertices: all combinations of ±1 in 4D
    let mut verts4d: Vec<[f32; 4]> = Vec::with_capacity(16);
    let s: f32 = 0.6;
    for i in 0..16u8 {
        let x = if i & 1 != 0 { s } else { -s };
        let y = if i & 2 != 0 { s } else { -s };
        let z = if i & 4 != 0 { s } else { -s };
        let w = if i & 8 != 0 { s } else { -s };
        verts4d.push([x, y, z, w]);
    }
    // 32 edges: connect vertices that differ in exactly one coordinate
    let mut edges = Vec::new();
    for i in 0..16u16 {
        for bit in 0..4u16 {
            let j = i ^ (1 << bit);
            if j > i {
                edges.push(Edge(i, j));
            }
        }
    }
    (verts4d, edges)
}

/// Project 4D point to 3D using perspective on the W axis
fn project_4d_to_3d(v: [f32; 4], w_angle: f32) -> V3 {
    // Rotate in XW plane
    let cw = cosf(w_angle);
    let software = sinf(w_angle);
    let x = v[0] * cw - v[3] * software;
    let w = v[0] * software + v[3] * cw;
    // Also rotate in YW plane for more 4D feel
    let cw2 = cosf(w_angle * 0.7);
    let sw2 = sinf(w_angle * 0.7);
    let y = v[1] * cw2 - w * sw2;
    let w2 = v[1] * sw2 + w * cw2;
    // Perspective projection from 4D to 3D
    let d4: f32 = 2.5;
    let scale = d4 / (d4 + w2);
    V3::new(x * scale, y * scale, v[2] * scale)
}

/// Vortex Tunnel — concentric rings at different depths, spiraling
fn generator_vortex_tunnel(rings: usize, segments: usize) -> (Vec<V3>, Vec<Edge>) {
    let mut verts = Vec::with_capacity(rings * segments);
    let mut edges = Vec::new();
    let pi2: f32 = 6.28318;

    for ring in 0..rings {
        let t = ring as f32 / rings as f32;
        let z = -1.0 + 2.0 * t;
        let r = 0.2 + 0.6 * (1.0 - t); // wider at front, narrower at back
        let twist = t * 2.0; // spiral twist
        for seg in 0..segments {
            let angle = pi2 * seg as f32 / segments as f32 + twist;
            let x = r * cosf(angle);
            let y = r * sinf(angle);
            verts.push(V3::new(x, y, z));
        }
    }
    // Ring edges
    for ring in 0..rings {
        let base = ring * segments;
        for seg in 0..segments {
            let next = (seg + 1) % segments;
            edges.push(Edge((base + seg) as u16, (base + next) as u16));
        }
    }
    // Longitudinal edges connecting rings
    for ring in 0..(rings - 1) {
        let base = ring * segments;
        let next_base = (ring + 1) * segments;
        for seg in (0..segments).step_by(2) {
            edges.push(Edge((base + seg) as u16, (next_base + seg) as u16));
        }
    }
    (verts, edges)
}

/// Plasma Sphere — base sphere + animated tendrils that extend outward
fn generator_plasma_sphere() -> (Vec<V3>, Vec<Edge>) {
    // Core sphere (lower res)
    let lat: usize = 6;
    let lon: usize = 10;
    let mut verts = Vec::with_capacity(lat * lon + 2);
    let mut edges = Vec::new();
    let pi: f32 = 3.14159;

    verts.push(V3::new(0.0, -0.5, 0.0));
    for la in 0..lat {
        let frac = (la as f32 + 1.0) / (lat as f32 + 1.0);
        let angle = -pi / 2.0 + pi * frac;
        let y = sinf(angle) * 0.5;
        let r = cosf(angle) * 0.5;
        for lo in 0..lon {
            let a = 6.28318 * lo as f32 / lon as f32;
            verts.push(V3::new(r * cosf(a), y, r * sinf(a)));
        }
    }
    verts.push(V3::new(0.0, 0.5, 0.0));
    // Ring edges
    for la in 0..lat {
        let base = 1 + la * lon;
        for lo in 0..lon {
            let next = if lo + 1 < lon { lo + 1 } else { 0 };
            edges.push(Edge((base + lo) as u16, (base + next) as u16));
        }
    }
    // Meridian edges
    for lo in 0..lon {
        edges.push(Edge(0, (1 + lo) as u16));
        for la in 0..(lat - 1) {
            let a = 1 + la * lon + lo;
            let b = 1 + (la + 1) * lon + lo;
            edges.push(Edge(a as u16, b as u16));
        }
        let last = 1 + (lat - 1) * lon + lo;
        edges.push(Edge(last as u16, (verts.len() - 1) as u16));
    }
    (verts, edges)
}

/// Galaxy Spiral — logarithmic spiral arms
fn generator_galaxy(arms: usize, points_per_arm: usize) -> (Vec<V3>, Vec<Edge>) {
    let mut verts = Vec::with_capacity(arms * points_per_arm + 1);
    let mut edges = Vec::new();
    let pi2: f32 = 6.28318;

    // Central core
    verts.push(V3::new(0.0, 0.0, 0.0));

    for arm in 0..arms {
        let arm_offset = pi2 * arm as f32 / arms as f32;
        for i in 0..points_per_arm {
            let t = (i as f32 + 1.0) / points_per_arm as f32;
            let r = 0.1 + t * 0.9;
            let angle = arm_offset + t * pi2 * 1.2; // 1.2 turns per arm
            let x = r * cosf(angle);
            let z = r * sinf(angle);
            // Slight vertical variation for 3D depth
            let y = sinf(angle * 2.0) * 0.08 * r;
            verts.push(V3::new(x, y, z));
        }
    }
    // Connect within each arm
    for arm in 0..arms {
        let base = 1 + arm * points_per_arm;
        // Core to first point
        edges.push(Edge(0, base as u16));
        for i in 0..(points_per_arm - 1) {
            edges.push(Edge((base + i) as u16, (base + i + 1) as u16));
        }
    }
    // Cross-arm connections (dust lanes)
    for arm in 0..arms {
        let next_arm = (arm + 1) % arms;
        for i in (0..points_per_arm).step_by(4) {
            let a = 1 + arm * points_per_arm + i;
            let b = 1 + next_arm * points_per_arm + i;
            if a < verts.len() && b < verts.len() {
                edges.push(Edge(a as u16, b as u16));
            }
        }
    }
    (verts, edges)
}

// ═══════════════════════════════════════
// Mode 13: Subscribe — Diamond Play Button + SUBSCRIBE text
// ═══════════════════════════════════════

/// Generate a YouTube Diamond Play Button with "SUBSCRIBE" text as 3D wireframe.
/// The shape is flat-ish (slight Z depth) for a recognizable icon look.
fn generator_subscribe_button() -> (Vec<V3>, Vec<Edge>) {
    let mut verts = Vec::with_capacity(256);
    let mut edges = Vec::new();

    // ── Diamond Play Button outline (rhombus shape) ──
    // Outer diamond: 4 corners
    let dw = 0.9f32;   // half-width
    let dh = 0.55f32;  // half-height
    let dz = 0.08f32;  // slight depth for 3D look

    // Front face corners (clockwise from top)
    let v0 = verts.len();
    verts.push(V3::new(0.0, dh, dz));           // top
    verts.push(V3::new(dw, 0.0, dz));           // right
    verts.push(V3::new(0.0, -dh, dz));          // bottom
    verts.push(V3::new(-dw, 0.0, dz));          // left

    // Back face corners
    verts.push(V3::new(0.0, dh, -dz));          // top back
    verts.push(V3::new(dw, 0.0, -dz));          // right back
    verts.push(V3::new(0.0, -dh, -dz));         // bottom back
    verts.push(V3::new(-dw, 0.0, -dz));         // left back

    // Front face edges
    edges.push(Edge(v0 as u16, (v0 + 1) as u16));
    edges.push(Edge((v0 + 1) as u16, (v0 + 2) as u16));
    edges.push(Edge((v0 + 2) as u16, (v0 + 3) as u16));
    edges.push(Edge((v0 + 3) as u16, v0 as u16));
    // Back face edges
    edges.push(Edge((v0 + 4) as u16, (v0 + 5) as u16));
    edges.push(Edge((v0 + 5) as u16, (v0 + 6) as u16));
    edges.push(Edge((v0 + 6) as u16, (v0 + 7) as u16));
    edges.push(Edge((v0 + 7) as u16, (v0 + 4) as u16));
    // Connecting edges (front to back)
    for i in 0..4 {
        edges.push(Edge((v0 + i) as u16, (v0 + 4 + i) as u16));
    }

    // ── Play triangle (inside diamond) ──
    let pw = 0.22f32;   // play triangle dimensions
    let ph = 0.28f32;
    let poff = 0.05f32; // slight right offset (YouTube style)
    let pz = dz + 0.02;

    let pt = verts.len();
    // Front play triangle
    verts.push(V3::new(-pw + poff, ph, pz));       // top-left
    verts.push(V3::new(pw * 1.2 + poff, 0.0, pz)); // right point
    verts.push(V3::new(-pw + poff, -ph, pz));      // bottom-left
    // Back play triangle
    verts.push(V3::new(-pw + poff, ph, -pz));
    verts.push(V3::new(pw * 1.2 + poff, 0.0, -pz));
    verts.push(V3::new(-pw + poff, -ph, -pz));

    // Front triangle edges
    edges.push(Edge(pt as u16, (pt + 1) as u16));
    edges.push(Edge((pt + 1) as u16, (pt + 2) as u16));
    edges.push(Edge((pt + 2) as u16, pt as u16));
    // Back triangle edges
    edges.push(Edge((pt + 3) as u16, (pt + 4) as u16));
    edges.push(Edge((pt + 4) as u16, (pt + 5) as u16));
    edges.push(Edge((pt + 5) as u16, (pt + 3) as u16));
    // Connecting
    for i in 0..3 {
        edges.push(Edge((pt + i) as u16, (pt + 3 + i) as u16));
    }

    // ── "SUBSCRIBE" text below the diamond ──
    // Each letter is a small 3D wireframe glyph at y offset below diamond
    let text_y = -dh - 0.25;  // below diamond
    let letter_w = 0.14f32;    // width per letter
    let letter_h = 0.15f32;    // letter height
    let text_z = 0.04f32;      // slight depth
    let total_w = 9.0 * letter_w; // "SUBSCRIBE" = 9 chars
    let start_x = -total_w / 2.0;

    // Helper: add a letter's front face vertices and edges
    // Letters are drawn as simple wireframe strokes
    let letters: [&[(f32, f32, f32, f32)]; 9] = [
        // S: 3 horizontal + 2 vertical pieces
        &[(0.0, 1.0, 0.8, 1.0), (0.0, 1.0, 0.0, 0.5), (0.0, 0.5, 0.8, 0.5),
          (0.8, 0.5, 0.8, 0.0), (0.0, 0.0, 0.8, 0.0)],
        // U: left side down, bottom, right side up
        &[(0.0, 1.0, 0.0, 0.0), (0.0, 0.0, 0.8, 0.0), (0.8, 0.0, 0.8, 1.0)],
        // B: left side + 3 horizontal + right bumps
        &[(0.0, 0.0, 0.0, 1.0), (0.0, 1.0, 0.6, 1.0), (0.6, 1.0, 0.7, 0.75),
          (0.7, 0.75, 0.6, 0.5), (0.0, 0.5, 0.6, 0.5), (0.6, 0.5, 0.7, 0.25),
          (0.7, 0.25, 0.6, 0.0), (0.0, 0.0, 0.6, 0.0)],
        // S (duplicate)
        &[(0.0, 1.0, 0.8, 1.0), (0.0, 1.0, 0.0, 0.5), (0.0, 0.5, 0.8, 0.5),
          (0.8, 0.5, 0.8, 0.0), (0.0, 0.0, 0.8, 0.0)],
        // C: top + left + bottom
        &[(0.8, 1.0, 0.0, 1.0), (0.0, 1.0, 0.0, 0.0), (0.0, 0.0, 0.8, 0.0)],
        // R: left + top + right-top + mid + diagonal
        &[(0.0, 0.0, 0.0, 1.0), (0.0, 1.0, 0.6, 1.0), (0.6, 1.0, 0.7, 0.75),
          (0.7, 0.75, 0.6, 0.5), (0.0, 0.5, 0.6, 0.5), (0.4, 0.5, 0.8, 0.0)],
        // I: top + middle + bottom
        &[(0.2, 1.0, 0.6, 1.0), (0.4, 1.0, 0.4, 0.0), (0.2, 0.0, 0.6, 0.0)],
        // B (duplicate)
        &[(0.0, 0.0, 0.0, 1.0), (0.0, 1.0, 0.6, 1.0), (0.6, 1.0, 0.7, 0.75),
          (0.7, 0.75, 0.6, 0.5), (0.0, 0.5, 0.6, 0.5), (0.6, 0.5, 0.7, 0.25),
          (0.7, 0.25, 0.6, 0.0), (0.0, 0.0, 0.6, 0.0)],
        // E: left + top + mid + bottom
        &[(0.0, 0.0, 0.0, 1.0), (0.0, 1.0, 0.8, 1.0), (0.0, 0.5, 0.6, 0.5),
          (0.0, 0.0, 0.8, 0.0)],
    ];

    for (li, strokes) in letters.iter().enumerate() {
        let ox = start_x + li as f32 * letter_w;
        for &(x1, y1, x2, y2) in *strokes {
            let sx1 = ox + x1 * (letter_w * 0.85);
            let sy1 = text_y + y1 * letter_h;
            let sx2 = ox + x2 * (letter_w * 0.85);
            let sy2 = text_y + y2 * letter_h;

            let vi = verts.len();
            // Front points
            verts.push(V3::new(sx1, sy1, text_z));
            verts.push(V3::new(sx2, sy2, text_z));
            // Back points
            verts.push(V3::new(sx1, sy1, -text_z));
            verts.push(V3::new(sx2, sy2, -text_z));
            // Front edge
            edges.push(Edge(vi as u16, (vi + 1) as u16));
            // Back edge
            edges.push(Edge((vi + 2) as u16, (vi + 3) as u16));
            // Depth connectors
            edges.push(Edge(vi as u16, (vi + 2) as u16));
            edges.push(Edge((vi + 1) as u16, (vi + 3) as u16));
        }
    }

    (verts, edges)
}

fn build_mode_13(s: &mut VisualizerState) {
    // Subscribe DVD Bounce: use pre-built geometry, no audio deformation
    s.verts.clear();
    s.verts.extend_from_slice(&s.subscribe_base);
    s.edges.clear();
    s.edges.extend_from_slice(&s.subscribe_edges);
}

// ═══════════════════════════════════════
// Initialization
// ═══════════════════════════════════════

fn ensure_init(s: &mut VisualizerState) {
    if s.initialized { return; }

    // Mode 0 & 3: Sphere + Spectrum Globe (same base geometry)
    let (sv, se, sb) = generator_sphere();
    s.spec_base = sv.clone();
    s.spec_bands = sb.clone();
    s.spec_edges = se.clone();
    s.sphere_base = sv;
    s.sphere_edges = se;
    s.sphere_bands = sb;

    // Mode 1: Morph targets — pad all to same vertex count (max 14)
    let (ico_v, ico_e) = generator_icosahedron();
    let (cube_v, cube_e) = generator_cube();
    let (dia_v, dia_e) = generator_diamond();
    let (star_v, star_e) = generator_star();

    // Find max vertex count
    let maximum_v = ico_v.len().max(cube_v.len()).max(dia_v.len()).max(star_v.len());

    // Pad shapes to max_v vertices (duplicate last vertex)
    fn pad_verts(v: &[V3], target: usize) -> Vec<V3> {
        let mut out = v.to_vec();
        while out.len() < target {
            out.push(*out.last().unwrap_or(&V3::new(0.0, 0.0, 0.0)));
        }
        out
    }

    s.morph_shapes[0] = pad_verts(&ico_v, maximum_v);
    s.morph_shapes[1] = pad_verts(&cube_v, maximum_v);
    s.morph_shapes[2] = pad_verts(&dia_v, maximum_v);
    s.morph_shapes[3] = pad_verts(&star_v, maximum_v);
    s.morph_edges_all[0] = ico_e;
    s.morph_edges_all[1] = cube_e;
    s.morph_edges_all[2] = dia_e;
    s.morph_edges_all[3] = star_e;
    s.morph_current = s.morph_shapes[0].clone();

    // Mode 2: Lorenz trail
    s.lorenz_trail = Vec::with_capacity(400);
    s.lorenz_state = [1.0, 1.0, 1.0];

    // Mode 4: Ribbon spine
    s.ribbon_spine = Vec::with_capacity(128);

    // Mode 5: Particles
    s.particles = Vec::with_capacity(200);

    // Mode 7: Torus Knot (2,3 trefoil)
    let (tk_v, tk_e) = generator_torus_knot(2, 3, 120);
    s.torus_knot_base = tk_v;
    s.torus_knot_edges = tk_e;

    // Mode 8: DNA Helix
    let (dna_v, dna_e) = generator_dna_helix(60, 3.0);
    s.dna_base = dna_v;
    s.dna_edges = dna_e;

    // Mode 9: Tesseract — store 4D verts, project per-frame
    // (initialized in build_mode_9)

    // Mode 10: Vortex Tunnel
    let (vt_v, vt_e) = generator_vortex_tunnel(10, 12);
    s.vortex_base = vt_v;
    s.vortex_edges = vt_e;

    // Mode 11: Plasma Sphere
    let (ps_v, ps_e) = generator_plasma_sphere();
    s.plasma_base = ps_v;
    s.plasma_edges = ps_e;

    // Mode 12: Galaxy
    let (gal_v, gal_e) = generator_galaxy(4, 20);
    s.galaxy_base = gal_v;
    s.galaxy_edges = gal_e;

    // Mode 13: Subscribe DVD Bounce
    let (sub_v, sub_e) = generator_subscribe_button();
    s.subscribe_base = sub_v;
    s.subscribe_edges = sub_e;

    s.initialized = true;
}

// ═══════════════════════════════════════
// Mode-specific shape generation
// ═══════════════════════════════════════

fn build_mode_0(s: &mut VisualizerState) {
    // Deform sphere with audio
    let amps = [
        s.smooth_sub_bass * 1.2,
        s.smooth_bass * 1.0,
        s.smooth_mid * 0.25,
        s.smooth_treble * 0.15,
    ];
    let bass_pulse = s.beat_pulse * (0.3 + s.smooth_sub_bass * 0.3 + s.smooth_bass * 0.2);

    s.verts.clear();
    for i in 0..s.sphere_base.len() {
        let bv = s.sphere_base[i];
        let band = s.sphere_bands[i] as usize;
        let amp = if band < 4 { amps[band] } else { 0.0 };
        let r = 1.0 + 0.35 * amp + bass_pulse * 0.25;
        s.verts.push(V3::new(bv.x * r, bv.y * r, bv.z * r));
    }
    s.edges.clear();
    s.edges.extend_from_slice(&s.sphere_edges);
}

fn build_mode_1(s: &mut VisualizerState) {
    // Morph between shapes — transition on beat
    let from_idx = s.morph_idx;
    let to_idx = (s.morph_idx + 1) % 4;
    let t = s.morph_phase - libm::floorf(s.morph_phase); // 0.0–1.0 within current transition

    // Smooth ease-in-out: t² * (3 - 2t)
    let smooth_t = t * t * (3.0 - 2.0 * t);

    let from = &s.morph_shapes[from_idx];
    let to = &s.morph_shapes[to_idx];
    let count = from.len().min(to.len());

    // Audio-reactive vertex jitter
    let jitter = s.beat_pulse * 0.15;

    s.verts.clear();
    for i in 0..count {
        let x = lerp(from[i].x, to[i].x, smooth_t) + jitter * sinf(i as f32 * 2.1);
        let y = lerp(from[i].y, to[i].y, smooth_t) + jitter * cosf(i as f32 * 1.7);
        let z = lerp(from[i].z, to[i].z, smooth_t) + jitter * sinf(i as f32 * 3.3);
        s.verts.push(V3::new(x, y, z));
    }

    // Use edges from the shape we're transitioning toward
    s.edges.clear();
    s.edges.extend_from_slice(&s.morph_edges_all[to_idx]);
}

fn build_mode_2(s: &mut VisualizerState) {
    // Lorenz attractor, audio-driven parameters
    let sigma = 10.0 + s.smooth_bass * 5.0;
    let rho = 28.0 + s.smooth_mid * 10.0;
    let beta = 2.667 + s.smooth_treble * 1.0;
    let dt: f32 = 0.006;

    let [x, y, z] = s.lorenz_state;
    let dx = sigma * (y - x);
    let dy = x * (rho - z) - y;
    let dz = x * y - beta * z;
    let nx = x + dx * dt;
    let ny = y + dy * dt;
    let nz = z + dz * dt;
    s.lorenz_state = [nx, ny, nz];

    // Add to trail
    // Normalize to unit-ish range (Lorenz goes roughly ±20, ±30, 0–50)
    let scale = 0.04;
    let pt = V3::new(nx * scale, (nz - 25.0) * scale, ny * scale);
    s.lorenz_trail.push(pt);
        // Compile-time constant — evaluated at compilation, zero runtime cost.
const MAXIMUM_TRAIL: usize = 350;
    while s.lorenz_trail.len() > MAXIMUM_TRAIL {
        s.lorenz_trail.remove(0);
    }

    // Build verts + edges from trail
    s.verts.clear();
    s.edges.clear();
    for p in &s.lorenz_trail {
        s.verts.push(*p);
    }
    for i in 0..(s.verts.len().saturating_sub(1)) {
        if i < 65535 {
            s.edges.push(Edge(i as u16, (i + 1) as u16));
        }
    }
}

fn build_mode_3(s: &mut VisualizerState) {
    // Spectrum globe: sphere with per-band radial displacement
    let amps = [
        s.smooth_sub_bass * 0.8,
        s.smooth_bass * 0.6,
        s.smooth_mid * 0.5,
        s.smooth_treble * 0.35,
    ];
    let bass_pulse = s.beat_pulse * 0.4;

    s.verts.clear();
    for i in 0..s.spec_base.len() {
        let bv = s.spec_base[i];
        let band = s.spec_bands[i] as usize;
        let amp = if band < 4 { amps[band] } else { 0.0 };
        // Stronger displacement: each band pushes radially
        let r = 1.0 + 0.6 * amp + bass_pulse * 0.3;
        s.verts.push(V3::new(bv.x * r, bv.y * r, bv.z * r));
    }
    s.edges.clear();
    s.edges.extend_from_slice(&s.spec_edges);
}

fn build_mode_4(s: &mut VisualizerState) {
    // Waveform ribbon: spine along Z, cross-section displaced by audio
    const SEGMENTS: usize = 80;
        // Compile-time constant — evaluated at compilation, zero runtime cost.
const WIDTH: f32 = 0.6;

    // Shift spine forward each frame (scrolling ribbon)
    if s.ribbon_spine.len() < SEGMENTS {
        // Initialize spine as straight line
        s.ribbon_spine.clear();
        for i in 0..SEGMENTS {
            let t = i as f32 / SEGMENTS as f32;
            s.ribbon_spine.push(V3::new(0.0, 0.0, -1.5 + 3.0 * t));
        }
    }

    // Scroll: shift all spine points forward, add new one at end
    s.ribbon_spine.remove(0);
    let last_z = s.ribbon_spine.last().map_or(1.5, |v| v.z);
    let dy = (s.smooth_mid * 0.6 + s.smooth_treble * 0.3) * sinf(s.frame as f32 * 0.1);
    let dx = s.smooth_bass * 0.4 * cosf(s.frame as f32 * 0.07);
    s.ribbon_spine.push(V3::new(dx, dy, last_z));

    // Build ribbon mesh: 2 vertices per segment (left/right)
    s.verts.clear();
    s.edges.clear();
    for (i, sp) in s.ribbon_spine.iter().enumerate() {
        // Cross-section offset perpendicular to Z
        let w = WIDTH + s.smooth_sub_bass * 0.3;
        let angle = s.frame as f32 * 0.02 + i as f32 * 0.08;
        let nx = cosf(angle);
        let ny = sinf(angle);
        s.verts.push(V3::new(sp.x - nx * w, sp.y - ny * w, sp.z));
        s.verts.push(V3::new(sp.x + nx * w, sp.y + ny * w, sp.z));
    }
    // Edge: connect pairs and sequential
    let n = s.ribbon_spine.len();
    for i in 0..n {
        let base = (i * 2) as u16;
        // Cross edge (left-right)
        if base + 1 < s.verts.len() as u16 {
            s.edges.push(Edge(base, base + 1));
        }
        // Longitudinal edges
        if i + 1 < n {
            let next = ((i + 1) * 2) as u16;
            s.edges.push(Edge(base, next));
            s.edges.push(Edge(base + 1, next + 1));
        }
    }
}

fn build_mode_5(s: &mut VisualizerState) {
    // Starburst: emit particles on beat, fade with life
    s.particle_timer = s.particle_timer.wrapping_add(1);

    // Emit burst on beat pulse
    if s.beat_pulse > 0.8 {
        let count = 30 + (s.smooth_bass * 20.0) as usize;
        let seed = s.frame as u32;
        for i in 0..count.min(50) {
            let hash = seed.wrapping_mul(2654435761).wrapping_add(i as u32);
            let fx = ((hash & 0xFF) as f32 / 128.0 - 1.0) * 0.8;
            let fy = (((hash >> 8) & 0xFF) as f32 / 128.0 - 1.0) * 0.8;
            let fz = (((hash >> 16) & 0xFF) as f32 / 128.0 - 1.0) * 0.8;
            let speed = 0.03 + (s.smooth_sub_bass + s.smooth_bass) * 0.02;
            s.particles.push(Particle {
                x: 0.0, y: 0.0, z: 0.0,
                vx: fx * speed, vy: fy * speed, vz: fz * speed,
                life: 1.0,
            });
        }
    }

    // Update particles
    for p in s.particles.iter_mut() {
        p.x += p.vx;
        p.y += p.vy;
        p.z += p.vz;
        p.vy -= 0.0003; // slight gravity
        p.life -= 0.012;
    }
    s.particles.retain(|p| p.life > 0.0);
    while s.particles.len() > 200 { s.particles.remove(0); }

    // Build wireframe: connect nearby particles
    s.verts.clear();
    s.edges.clear();
    for p in &s.particles {
        s.verts.push(V3::new(p.x, p.y, p.z));
    }
    let n = s.verts.len();
    // Connect sequential pairs (burst order creates natural clusters)
    for i in (0..n.saturating_sub(1)).step_by(1) {
        if i + 1 < n && i < 65535 {
            s.edges.push(Edge(i as u16, (i + 1) as u16));
        }
    }
    // Also connect every 3rd for web effect
    for i in (0..n.saturating_sub(3)).step_by(3) {
        if i + 3 < n && i + 3 < 65535 {
            s.edges.push(Edge(i as u16, (i + 3) as u16));
        }
    }
}

fn build_mode_6(s: &mut VisualizerState) {
    // Image reveal mode — no 3D geometry; rain samples the logo bitmap directly.
    // We just need to compute image placement and set column_bounds so that
    // the fill layers and column_slow_factor know where the image is.
    let logo_w = crate::logo_bitmap::LOGO_W as u32;
    let logo_h = crate::logo_bitmap::LOGO_H as u32;

    // Display the image at ~1.5× native size, centered
    let display_w = logo_w * 3 / 2;
    let disp_h = logo_h * 3 / 2;
    s.image_display_w = display_w;
    s.image_display_h = disp_h;
    s.image_offset_x = s.center_x - display_w as i32 / 2;
    s.image_offset_y = s.center_y - disp_h as i32 / 2;

    // Beat makes image pulse slightly
    let pulse = (s.beat_pulse * 40.0) as i32;
    s.image_offset_x -= pulse / 2;
    s.image_offset_y -= pulse / 2;
    s.image_display_w = (display_w as i32 + pulse) as u32;
    s.image_display_h = (disp_h as i32 + pulse) as u32;

    // Set column bounds based on image extent so fill layers + slow factor work
    let col_w = s.col_w;
    if col_w > 0 {
        let ncols = s.column_bounds.len();
        for col in 0..ncols {
            let column_pixel = col as i32 * col_w + col_w / 2;
            if column_pixel >= s.image_offset_x && column_pixel < s.image_offset_x + s.image_display_w as i32 {
                s.column_bounds[col] = (
                    s.image_offset_y.max(0),
                    (s.image_offset_y + s.image_display_h as i32).max(0),
                );
            }
        }
    }

    // No verts/edges — image sampling happens entirely in check_rain_collision
    s.verts.clear();
    s.edges.clear();
}

fn build_mode_7(s: &mut VisualizerState) {
    // Torus Knot — audio-reactive radius pulsation
    let bass_pulse = s.beat_pulse * 0.3 + s.smooth_bass * 0.2;
    let treble_jitter = s.smooth_treble * 0.15;
    let time = s.frame as f32 * 0.02;

    s.verts.clear();
    for (i, bv) in s.torus_knot_base.iter().enumerate() {
        let t = i as f32 / s.torus_knot_base.len().max(1) as f32;
        // Audio-reactive radial breathing
        let r = 1.0 + bass_pulse * 0.4 + sinf(t * 6.28 * 3.0 + time) * treble_jitter;
        s.verts.push(V3::new(bv.x * r, bv.y * r, bv.z * r));
    }
    s.edges.clear();
    s.edges.extend_from_slice(&s.torus_knot_edges);
}

fn build_mode_8(s: &mut VisualizerState) {
    // DNA Helix — audio-reactive twist and breathing
    let bass_pulse = s.beat_pulse * 0.25;
    let mid_twist = s.smooth_mid * 0.3;
    let time = s.frame as f32 * 0.015;
    let segments = s.dna_base.len() / 2;

    s.verts.clear();
    for (i, bv) in s.dna_base.iter().enumerate() {
        let t = (i % segments.max(1)) as f32 / segments.max(1) as f32;
        // Breathing radius
        let r = 1.0 + bass_pulse * 0.3;
        // Extra twist based on mid frequencies
        let extra_twist = mid_twist * sinf(t * 6.28 + time);
        let ca = cosf(extra_twist);
        let sa = sinf(extra_twist);
        let nx = bv.x * ca - bv.z * sa;
        let nz = bv.x * sa + bv.z * ca;
        s.verts.push(V3::new(nx * r, bv.y * r, nz * r));
    }
    s.edges.clear();
    s.edges.extend_from_slice(&s.dna_edges);
}

fn build_mode_9(s: &mut VisualizerState) {
    // Tesseract (4D Hypercube) — rotate in 4D, project to 3D
    s.tesseract_w_angle += 0.02 + s.smooth_bass * 0.05 + s.beat_pulse * 0.1;

    let (verts4d, edges4d) = generator_tesseract();

    s.verts.clear();
    for v4 in &verts4d {
        let v3 = project_4d_to_3d(*v4, s.tesseract_w_angle);
        // Audio-reactive scale
        let pulse = 1.0 + s.beat_pulse * 0.3;
        s.verts.push(V3::new(v3.x * pulse, v3.y * pulse, v3.z * pulse));
    }
    s.edges.clear();
    s.edges.extend_from_slice(&edges4d);
}

fn build_mode_10(s: &mut VisualizerState) {
    // Vortex Tunnel — spinning rings with depth
    let time = s.frame as f32 * 0.03;
    let bass_pulse = s.beat_pulse * 0.4;
    let rings = 10usize;
    let segments = 12usize;

    s.verts.clear();
    for (i, bv) in s.vortex_base.iter().enumerate() {
        let ring = i / segments;
        let t = ring as f32 / rings as f32;
        // Rings pulse on beat, inner rings more
        let rpulse = 1.0 + bass_pulse * (1.0 - t) + s.smooth_sub_bass * 0.2;
        // Spinning twist increases with time
        let spin = time * (1.0 + t * 2.0);
        let ca = cosf(spin);
        let sa = sinf(spin);
        let nx = bv.x * ca - bv.y * sa;
        let ny = bv.x * sa + bv.y * ca;
        s.verts.push(V3::new(nx * rpulse, ny * rpulse, bv.z));
    }
    s.edges.clear();
    s.edges.extend_from_slice(&s.vortex_edges);
}

fn build_mode_11(s: &mut VisualizerState) {
    // Plasma Sphere — core sphere with animated tendrils
    let time = s.frame as f32 * 0.02;
    let bass_pulse = s.beat_pulse * 0.5;
    let energy = s.smooth_sub_bass + s.smooth_bass;

    s.verts.clear();
    // Core sphere breathing
    for bv in s.plasma_base.iter() {
        let r = 1.0 + bass_pulse * 0.3;
        s.verts.push(V3::new(bv.x * r, bv.y * r, bv.z * r));
    }
    let core_count = s.plasma_base.len();

    // Generate tendrils — lightning-like arms extending from sphere
    let number_tendrils: usize = 6;
    let tendril_segments: usize = 8;
    let pi2: f32 = 6.28318;
    for t in 0..number_tendrils {
        let base_angle = pi2 * t as f32 / number_tendrils as f32 + time * 0.5;
        let elev = sinf(time * 0.7 + t as f32 * 1.5) * 0.6;
        let base_x = cosf(base_angle) * cosf(elev) * 0.5;
        let base_y = sinf(elev) * 0.5;
        let base_z = sinf(base_angle) * cosf(elev) * 0.5;

        for seg in 0..tendril_segments {
            let st = (seg as f32 + 1.0) / tendril_segments as f32;
            // Tendril extends outward with audio-reactive length
            let length = 0.5 + energy * 0.4 + bass_pulse * 0.3;
            let jitter_x = sinf(time * 3.0 + seg as f32 * 2.0 + t as f32) * 0.08 * st;
            let jitter_y = cosf(time * 2.5 + seg as f32 * 1.7 + t as f32) * 0.08 * st;
            let jitter_z = sinf(time * 2.0 + seg as f32 * 2.3 + t as f32) * 0.08 * st;
            s.verts.push(V3::new(
                base_x * (1.0 + st * length) + jitter_x,
                base_y * (1.0 + st * length) + jitter_y,
                base_z * (1.0 + st * length) + jitter_z,
            ));
        }
    }

    s.edges.clear();
    s.edges.extend_from_slice(&s.plasma_edges);
    // Tendril edges
    for t in 0..number_tendrils {
        let base_index = core_count + t * tendril_segments;
        // Connect first tendril vertex to nearest sphere surface vertex
        // (find closest core vertex)
        if base_index < s.verts.len() && core_count > 0 {
            let tv = s.verts[base_index];
            let mut best_d = f32::MAX;
            let mut best_i = 0u16;
            for ci in 0..core_count {
                let cv = s.verts[ci];
                let dx = tv.x - cv.x; let dy = tv.y - cv.y; let dz = tv.z - cv.z;
                let d = dx*dx + dy*dy + dz*dz;
                if d < best_d { best_d = d; best_i = ci as u16; }
            }
            s.edges.push(Edge(best_i, base_index as u16));
        }
        // Connect tendril segments
        for seg in 0..(tendril_segments - 1) {
            let a = base_index + seg;
            let b = base_index + seg + 1;
            if a < s.verts.len() && b < s.verts.len() {
                s.edges.push(Edge(a as u16, b as u16));
            }
        }
    }
}

fn build_mode_12(s: &mut VisualizerState) {
    // Galaxy Spiral — audio-reactive arm extension and rotation
    let time = s.frame as f32 * 0.01;
    let bass_pulse = s.beat_pulse * 0.3;
    let pi2: f32 = 6.28318;
    let arms: usize = 4;
    let points_per_arm: usize = 20;

    s.verts.clear();
    // Core
    s.verts.push(V3::new(0.0, sinf(time * 2.0) * 0.05, 0.0));

    for arm in 0..arms {
        let arm_offset = pi2 * arm as f32 / arms as f32;
        for i in 0..points_per_arm {
            let t = (i as f32 + 1.0) / points_per_arm as f32;
            let r = 0.1 + t * (0.9 + bass_pulse * 0.3);
            let angle = arm_offset + t * pi2 * 1.2 + time;
            let x = r * cosf(angle);
            let z = r * sinf(angle);
            // Vertical wave
            let y = sinf(angle * 2.0 + time * 1.5) * 0.1 * r;
            // Audio: treble makes outer stars twinkle
            let jitter = s.smooth_treble * 0.05 * t;
            let jx = sinf(time * 4.0 + i as f32) * jitter;
            let jz = cosf(time * 3.5 + i as f32) * jitter;
            s.verts.push(V3::new(x + jx, y, z + jz));
        }
    }

    s.edges.clear();
    // Arms
    for arm in 0..arms {
        let base = 1 + arm * points_per_arm;
        s.edges.push(Edge(0, base as u16));
        for i in 0..(points_per_arm - 1) {
            s.edges.push(Edge((base + i) as u16, (base + i + 1) as u16));
        }
    }
    // Cross-arm dust lanes
    for arm in 0..arms {
        let next_arm = (arm + 1) % arms;
        for i in (0..points_per_arm).step_by(4) {
            let a = 1 + arm * points_per_arm + i;
            let b = 1 + next_arm * points_per_arm + i;
            if a < s.verts.len() && b < s.verts.len() {
                s.edges.push(Edge(a as u16, b as u16));
            }
        }
    }
}

// ═══════════════════════════════════════
// Main update (called once per frame)
// ═══════════════════════════════════════

pub fn update(
    state: &mut VisualizerState,
    screen_w: u32, screen_h: u32,
    matrix_cols: usize,
    beat: f32, energy: f32,
    sub_bass: f32, bass: f32, mid: f32, treble: f32,
    playing: bool,
) {
    ensure_init(state);
    state.frame = state.frame.wrapping_add(1);
    state.center_x = screen_w as i32 / 2;
    state.center_y = screen_h as i32 / 2;

    // ── Smooth audio ──
    let sm = 0.15f32;
    if playing {
        state.smooth_sub_bass += (sub_bass - state.smooth_sub_bass) * sm;
        state.smooth_bass     += (bass     - state.smooth_bass)     * sm;
        state.smooth_mid      += (mid      - state.smooth_mid)      * sm;
        state.smooth_treble   += (treble   - state.smooth_treble)   * sm;
    } else {
        state.smooth_sub_bass *= 0.95;
        state.smooth_bass     *= 0.95;
        state.smooth_mid      *= 0.95;
        state.smooth_treble   *= 0.95;
    }

    // Beat pulse
    let bass_beat = playing && beat > 0.5 && (sub_bass + bass) > 0.4;
    if bass_beat { state.beat_pulse = 1.0; }
    state.beat_pulse *= 0.90;

    // Beat ripple
    if state.beat_pulse > 0.9 { state.ripple_radius = 0.0; }
    if state.ripple_radius < 600.0 { state.ripple_radius += 6.0; }

    // ── Rotation ──
    if state.mode == 13 {
        // Subscribe mode: slow constant rotation, NOT audio-reactive
        state.rot_x += 3;
        state.rot_y += 5;
        state.rot_z += 1;
        state.rot_x %= 6283; state.rot_y %= 6283; state.rot_z %= 6283;
    } else {
        let bass_hit = if playing {
            ((state.smooth_sub_bass + state.smooth_bass) * 15.0 + beat * 8.0) as i32
        } else { 0 };
        state.rot_x += state.rot_speed_x + bass_hit / 4;
        state.rot_y += state.rot_speed_y + bass_hit;
        state.rot_z += state.rot_speed_z;
        state.rot_x %= 6283; state.rot_y %= 6283; state.rot_z %= 6283;
    }

    // ── Scale ──
    if state.mode == 13 {
        // Subscribe mode: fixed scale, no audio
        state.scale = 160;
        state.scale_target = 160;
    } else {
        state.scale_target = if playing {
            150 + ((state.smooth_sub_bass + state.smooth_bass) * 25.0) as i32
                + (state.beat_pulse * 25.0) as i32
        } else { 150 };
        state.scale += (state.scale_target - state.scale) / 3;
        state.scale = state.scale.max(80).min(220);
    }

    // ── Mode 13: DVD bounce movement ──
    if state.mode == 13 {
        // Compute shape bounding box radius in screen coords (approximate)
        let shape_radius_x = (state.scale as f32 * 0.9).max(80.0);
        let shape_radius_y = (state.scale as f32 * 0.7).max(60.0);

        // Move position
        state.dvd_x += state.dvd_vx;
        state.dvd_y += state.dvd_vy;

        // Bounce off edges
        let margin_x = shape_radius_x;
        let margin_y = shape_radius_y;
        let software = screen_w as f32;
        let sh = screen_h as f32;

        if state.dvd_x - margin_x < 0.0 {
            state.dvd_x = margin_x;
            state.dvd_vx = state.dvd_vx.abs();
        } else if state.dvd_x + margin_x > software {
            state.dvd_x = software - margin_x;
            state.dvd_vx = -(state.dvd_vx.abs());
        }
        if state.dvd_y - margin_y < 0.0 {
            state.dvd_y = margin_y;
            state.dvd_vy = state.dvd_vy.abs();
        } else if state.dvd_y + margin_y > sh {
            state.dvd_y = sh - margin_y;
            state.dvd_vy = -(state.dvd_vy.abs());
        }

        // Override center to bounce position
        state.center_x = state.dvd_x as i32;
        state.center_y = state.dvd_y as i32;

        // Red flash on beat — trigger and decay
        if bass_beat {
            state.dvd_flash = 1.0;
        }
        state.dvd_flash *= 0.92;
        if state.dvd_flash < 0.01 { state.dvd_flash = 0.0; }
    }

    // ── Mode 1: advance morph phase ──
    if state.mode == 1 {
        // Advance morph: beat triggers transition, slow continuous otherwise
        let speed = if state.beat_pulse > 0.5 { 0.08 } else { 0.004 };
        state.morph_phase += speed;
        if state.morph_phase >= 1.0 {
            state.morph_phase -= 1.0;
            state.morph_idx = (state.morph_idx + 1) % 4;
        }
    }

    // ── Build shape for current mode ──
    match state.mode {
        0 => build_mode_0(state),
        1 => build_mode_1(state),
        2 => build_mode_2(state),
        3 => build_mode_3(state),
        4 => build_mode_4(state),
        5 => build_mode_5(state),
        6 => build_mode_6(state),
        7 => build_mode_7(state),
        8 => build_mode_8(state),
        9 => build_mode_9(state),
        10 => build_mode_10(state),
        11 => build_mode_11(state),
        12 => build_mode_12(state),
        13 => build_mode_13(state),
        _ => build_mode_0(state),
    }

    // ── Project all vertices ──
    let (scale, rx, ry, rz) = (state.scale, state.rot_x, state.rot_y, state.rot_z);
    let (cx, cy) = (state.center_x, state.center_y);

    state.pverts.clear();
    state.proj_z.clear();
    let mut zmin: i16 = i16::MAX;
    let mut zmax: i16 = i16::MIN;
    for v in &state.verts {
        let (x3, y3, z3) = transform_vertex(*v, rx, ry, rz, scale);
        state.pverts.push(project(x3, y3, z3, cx, cy));
        let z16 = (z3 as i16).max(-500).min(500);
        state.proj_z.push(z16);
        if z16 < zmin { zmin = z16; }
        if z16 > zmax { zmax = z16; }
    }
    state.z_min = zmin;
    state.z_max = zmax;

    // ── Specular highlight ──
    {
        let mut best_dot: i32 = i32::MIN;
        let mut best_sx: i32 = cx;
        let mut best_sy: i32 = cy;
        let (lx, ly, lz) = (500i32, 700, 500);
        for (vi, v) in state.verts.iter().enumerate() {
            let (nx, ny, nz) = transform_vertex(*v, rx, ry, rz, 1000);
            let dot = (nx * lx + ny * ly + nz * lz) / 1000;
            if dot > best_dot {
                best_dot = dot;
                if vi < state.pverts.len() {
                    best_sx = state.pverts[vi].0;
                    best_sy = state.pverts[vi].1;
                }
            }
        }
        state.spec_x = best_sx;
        state.spec_y = best_sy;
    }

    // ── Shadow zone ──
    {
        let mut global_bmax: i32 = -1;
        for &(_, bmax) in state.column_bounds.iter() {
            if bmax > global_bmax { global_bmax = bmax; }
        }
        if global_bmax > 0 {
            state.shadow_y_start = global_bmax;
            state.shadow_y_end = global_bmax + 120;
        } else {
            state.shadow_y_start = 0; state.shadow_y_end = 0;
        }
        state.shape_center_y = cy;
    }

    // ── Column hit-map ──
    let col_w = if matrix_cols > 0 { screen_w as i32 / matrix_cols as i32 } else { 8 };
    state.col_w = col_w;

    if state.column_hits.len() != matrix_cols {
        state.column_hits.clear();
        state.column_bounds.clear();
        for _ in 0..matrix_cols {
            state.column_hits.push(Vec::new());
            state.column_bounds.push((-1, -1));
        }
    } else {
        for h in state.column_hits.iter_mut() { h.clear(); }
        for b in state.column_bounds.iter_mut() { *b = (-1, -1); }
    }

    for (ei, edge) in state.edges.iter().enumerate() {
        let a = edge.0 as usize;
        let b = edge.1 as usize;
        if a >= state.pverts.len() || b >= state.pverts.len() { continue; }
        let (x0, y0) = state.pverts[a];
        let (x1, y1) = state.pverts[b];
        let z_average = if a < state.proj_z.len() && b < state.proj_z.len() {
            ((state.proj_z[a] as i32 + state.proj_z[b] as i32) / 2) as i16
        } else { 0 };
        rasterize_edge(x0, y0, x1, y1, ei as u16, z_average, col_w, matrix_cols,
                       screen_h as i32, &mut state.column_hits, &mut state.column_bounds);
    }
}

fn rasterize_edge(
    x0: i32, y0: i32, x1: i32, y1: i32,
    eidx: u16, z_average: i16, col_w: i32, ncols: usize, sh: i32,
    hits: &mut [Vec<(i32, i32, u16, u8, i16)>],
    bounds: &mut [(i32, i32)],
) {
    if col_w <= 0 { return; }
    let dx = (x1 - x0).abs();
    let dy = (y1 - y0).abs();
    let steps = dx.max(dy).max(1).min(2048);
    let step_x = ((x1 - x0) * 1024) / steps;
    let step_y = ((y1 - y0) * 1024) / steps;
    let mut px = x0 * 1024;
    let mut py = y0 * 1024;
    let prox = 8i32;
    let stride = 2;
    let mut s = 0;
    while s <= steps {
        let sx = px / 1024;
        let sy = py / 1024;
        let c = sx / col_w;
        if c >= 0 && (c as usize) < ncols && sy >= 0 && sy < sh {
            let ci = c as usize;
            if hits[ci].len() < 64 {
                let y_lo = (sy - prox).max(0);
                let y_hi = (sy + prox).min(sh - 1);
                let dist = ((sx - c * col_w - col_w / 2).abs() as u32 * 255 / (prox as u32 + 1)).min(255) as u8;
                let intensity = 255u8.saturating_sub(dist);
                hits[ci].push((y_lo, y_hi, eidx, intensity, z_average));
            }
            let (ref mut bmin, ref mut bmax) = bounds[ci];
            let yy = sy;
            if *bmin < 0 || yy < *bmin { *bmin = yy; }
            if yy > *bmax { *bmax = yy; }
        }
        px += step_x * stride;
        py += step_y * stride;
        s += stride;
    }
}

// ═══════════════════════════════════════
// Rain collision query (same API as ghost_mesh)
// ═══════════════════════════════════════

pub fn check_rain_collision(
    state: &VisualizerState, col: usize, y: i32,
    beat_pulse: f32, energy: f32,
) -> RainEffect {
    if !state.initialized { return RainEffect::NONE; }
    let ncols = state.column_hits.len();
    if col >= ncols { return RainEffect::NONE; }

    let (cx, cy) = (state.center_x, state.center_y);

    // ═══════════════════════════════════════
    // Mode 6: Image Reveal — sample logo bitmap directly
    // ═══════════════════════════════════════
    if state.mode == 6 && state.image_display_w > 0 && state.image_display_h > 0 {
        let screen_x = col as i32 * state.col_w + state.col_w / 2;

        // Map screen position to image pixel coordinates
        let relative_x = screen_x - state.image_offset_x;
        let relative_y = y - state.image_offset_y;

        if relative_x >= 0 && relative_x < state.image_display_w as i32
            && relative_y >= 0 && relative_y < state.image_display_h as i32
        {
            let logo_w = crate::logo_bitmap::LOGO_W;
            let logo_h = crate::logo_bitmap::LOGO_H;
            // Nearest-neighbor sample from display size → logo native size
            let image_x = (relative_x as u32 * logo_w as u32 / state.image_display_w) as usize;
            let image_y = (relative_y as u32 * logo_h as u32 / state.image_display_h) as usize;

            if image_x < logo_w && image_y < logo_h {
                let pixel = crate::logo_bitmap::logo_pixel(image_x, image_y);
                let a = (pixel >> 24) & 0xFF;
                let pr = (pixel >> 16) & 0xFF;
                let pg = (pixel >> 8) & 0xFF;
                let pb = pixel & 0xFF;

                // Only reveal non-transparent pixels with some brightness
                let brightness = (pr.max(pg).max(pb)) as u8;
                if a > 30 && brightness > 15 {
                    // Blend strength: brighter pixels → stronger reveal
                    // Beat pulse intensifies the image
                    let base_blend = 140u8 + (brightness / 4);
                    let beat_boost = (beat_pulse * 60.0).min(60.0) as u8;
                    let blend = base_blend.saturating_add(beat_boost).min(230);

                    // Beat ripple for image mode
                    let mut ripple: u8 = 0;
                    {
                        let dx = (screen_x - cx) as f32;
                        let dy = (y - cy) as f32;
                        let dist = libm::sqrtf(dx * dx + dy * dy);
                        let ring_dist = libm::fabsf(dist - state.ripple_radius);
                        if ring_dist < 35.0 {
                            let life = (1.0 - state.ripple_radius / 500.0).max(0.0);
                            let t = (1.0 - ring_dist / 35.0) * life;
                            ripple = (t * 120.0).min(120.0) as u8;
                        }
                    }

                    // Slight glow on brighter pixels (makes edges pop)
                    let glow = if brightness > 100 { (brightness - 60) / 2 } else { 0 };

                    return RainEffect {
                        glow,
                        depth: 128,
                        trail_boost: 10,
                        ripple,
                        dim: 0,
                        fresnel: 0,
                        specular: 0,
                        ao: 0,
                        bloom: if brightness > 180 { brightness / 3 } else { 0 },
                        scanline: 0,
                        inner_glow: 0,
                        shadow: 0,
                        target_r: pr as u8,
                        target_g: pg as u8,
                        target_b: pb as u8,
                        target_blend: blend,
                    };
                }
            }
        }

        // Outside image or on transparent pixel — slight dim near image edge
        let relative_x = screen_x - state.image_offset_x;
        let relative_y = y - state.image_offset_y;
        let dx_edge = if relative_x < 0 { -relative_x }
            else if relative_x >= state.image_display_w as i32 { relative_x - state.image_display_w as i32 + 1 }
            else { 0 };
        let dy_edge = if relative_y < 0 { -relative_y }
            else if relative_y >= state.image_display_h as i32 { relative_y - state.image_display_h as i32 + 1 }
            else { 0 };
        let edge_dist = dx_edge.max(dy_edge);
        if edge_dist > 0 && edge_dist < 40 {
            let dim = ((1.0 - edge_dist as f32 / 40.0) * 50.0) as u8;
            return RainEffect { dim, ..RainEffect::NONE };
        }

        return RainEffect::NONE;
    }

    // ═══════════════════════════════════════
    // Modes 0–5, 7–13: 3D wireframe collision
    // ═══════════════════════════════════════

    // ── Red flash for Subscribe mode (mode 13) ──
    let red_flash = if state.mode == 13 { state.dvd_flash } else { 0.0 };

    // ── Beat ripple ring ──
    let mut ripple: u8 = 0;
    {
        let dx = (col as i32 * state.col_w + state.col_w / 2 - cx) as f32;
        let dy = (y - cy) as f32;
        let dist = libm::sqrtf(dx * dx + dy * dy);
        let ring_dist = libm::fabsf(dist - state.ripple_radius);
        if ring_dist < 35.0 {
            let life = (1.0 - state.ripple_radius / 500.0).max(0.0);
            let t = (1.0 - ring_dist / 35.0) * life;
            ripple = (t * 120.0).min(120.0) as u8;
        }
    }

    // ── Edge glow (strongest effect) ──
    let hits = &state.column_hits[col];
    for &(y_lo, y_hi, _eidx, intensity, z_depth) in hits.iter() {
        if y >= y_lo && y <= y_hi {
            let edge_dist = (y - (y_lo + y_hi) / 2).abs();
            let half = (y_hi - y_lo) / 2;
            let glow_t = if half > 0 {
                1.0 - (edge_dist as f32 / half as f32).min(1.0)
            } else { 1.0 };
            let glow = (80.0 + glow_t * 175.0 * (intensity as f32 / 255.0)) as u8;

            // Depth normalization
            let z_range = (state.z_max as i32 - state.z_min as i32).max(1);
            let depth = ((z_depth as i32 - state.z_min as i32) * 255 / z_range).max(0).min(255) as u8;

            // Fresnel: brighter at silhouette edges
            let (bmin, bmax) = state.column_bounds[col];
            let fresnel = if bmax > bmin {
                let cy_mid = (bmin + bmax) / 2;
                let edge_from_mid = (y - cy_mid).abs() as f32;
                let half_h = ((bmax - bmin) / 2) as f32;
                if half_h > 0.0 {
                    let f = (edge_from_mid / half_h).min(1.0);
                    (f * f * 255.0) as u8
                } else { 0 }
            } else { 0 };

            // Specular
            let sdx = (col as i32 * state.col_w - state.spec_x).abs();
            let sdy = (y - state.spec_y).abs();
            let spec_dist = sdx + sdy;
            let specular = if spec_dist < 60 {
                ((1.0 - spec_dist as f32 / 60.0) * 255.0) as u8
            } else { 0 };

            // AO: darker at poles (top/bottom of silhouette)
            let ao = if bmax > bmin {
                let t = (y - bmin) as f32 / (bmax - bmin).max(1) as f32;
                let pole = (0.5 - t).abs() * 2.0; // 1.0 at poles, 0.0 at equator
                (pole * pole * 100.0) as u8
            } else { 0 };

            // Bloom
            let bloom = (glow as u16 * depth as u16 / 512).min(200) as u8;

            // Inner glow
            let inner_glow = if bmax > bmin {
                let dx = col as i32 * state.col_w + state.col_w / 2 - cx;
                let dy = y - cy;
                let r = libm::sqrtf((dx * dx + dy * dy) as f32);
                let maximum_r = ((bmax - bmin) as f32 / 2.0).max(1.0);
                let t = (1.0 - r / maximum_r).max(0.0);
                (t * t * 180.0) as u8
            } else { 0 };

            return RainEffect {
                glow, depth,
                trail_boost: (glow / 4).min(80),
                ripple,
                dim: 0,
                fresnel, specular, ao, bloom,
                scanline: 0,
                inner_glow,
                shadow: 0,
                target_r: if red_flash > 0.01 { 255 } else { 0 },
                target_g: 0,
                target_b: 0,
                target_blend: (red_flash * 220.0).min(220.0) as u8,
            };
        }
    }

    // ── Inside silhouette (volume fill) ──
    let (bmin, bmax) = state.column_bounds[col];
    if bmin >= 0 && y >= bmin && y <= bmax {
        let glow = 20u8;
        let z_range = (state.z_max as i32 - state.z_min as i32).max(1);
        let depth = 128u8;

        let fresnel = {
            let cy_mid = (bmin + bmax) / 2;
            let edge_from_mid = (y - cy_mid).abs() as f32;
            let half_h = ((bmax - bmin) / 2) as f32;
            if half_h > 0.0 {
                let f = (edge_from_mid / half_h).min(1.0);
                (f * f * 200.0) as u8
            } else { 0 }
        };

        let inner_glow = {
            let dx = col as i32 * state.col_w + state.col_w / 2 - cx;
            let dy = y - cy;
            let r = libm::sqrtf((dx * dx + dy * dy) as f32);
            let maximum_r = ((bmax - bmin) as f32 / 2.0).max(1.0);
            let t = (1.0 - r / maximum_r).max(0.0);
            (t * t * 120.0) as u8
        };

        return RainEffect {
            glow, depth, trail_boost: 5,
            ripple, dim: 0,
            fresnel, specular: 0, ao: 0, bloom: 0,
            scanline: 0, inner_glow, shadow: 0,
            target_r: if red_flash > 0.01 { 255 } else { 0 },
            target_g: 0,
            target_b: 0,
            target_blend: (red_flash * 180.0).min(180.0) as u8,
        };
    }

    // ── Contrast dim zone outside shape ──
    if bmin >= 0 {
        let outside_dist = if y < bmin { bmin - y } else if y > bmax { y - bmax } else { 0 };
        if outside_dist > 0 && outside_dist < 60 {
            let dim = ((1.0 - outside_dist as f32 / 60.0) * 80.0) as u8;
            return RainEffect {
                glow: 0, depth: 128, trail_boost: 0,
                ripple, dim, fresnel: 0, specular: 0, ao: 0,
                bloom: 0, scanline: 0, inner_glow: 0, shadow: 0,
                target_r: 0, target_g: 0, target_b: 0, target_blend: 0,
            };
        }
    }

    // ── Shadow below shape ──
    if y > state.shadow_y_start && y < state.shadow_y_end {
        let t = (y - state.shadow_y_start) as f32 / (state.shadow_y_end - state.shadow_y_start) as f32;
        let shadow = ((1.0 - t) * (1.0 - t) * 120.0) as u8;
        return RainEffect {
            glow: 0, depth: 128, trail_boost: 0,
            ripple, dim: 0, fresnel: 0, specular: 0, ao: 0,
            bloom: 0, scanline: 0, inner_glow: 0, shadow,
            target_r: 0, target_g: 0, target_b: 0, target_blend: 0,
        };
    }

    // ── Scanline rays below center ──
    if y > state.shape_center_y && y < state.shape_center_y + 200 {
        let px = col as i32 * state.col_w + state.col_w / 2;
        let dx = (px - cx).abs();
        if dx < 80 && col % 4 == 0 {
            let ray_t = (1.0 - dx as f32 / 80.0) * (1.0 - (y - state.shape_center_y) as f32 / 200.0);
            if ray_t > 0.05 {
                let scanline = (ray_t * 150.0).min(200.0) as u8;
                return RainEffect {
                    glow: 0, depth: 128, trail_boost: 0,
                    ripple, dim: 0, fresnel: 0, specular: 0, ao: 0,
                    bloom: 0, scanline, inner_glow: 0, shadow: 0,
                    target_r: 0, target_g: 0, target_b: 0, target_blend: 0,
                };
            }
        }
    }

    // ── Ripple only ──
    if ripple > 0 {
        return RainEffect { ripple, ..RainEffect::NONE };
    }

    RainEffect::NONE
}

// ═══════════════════════════════════════
// Color modulation (same as ghost_mesh)
// ═══════════════════════════════════════

pub fn modulate_rain_color(
    base_r: u8, base_g: u8, base_b: u8,
    glow: u8, depth: u8, ripple: u8,
    fresnel: u8, specular: u8,
    ao: u8, bloom: u8, scanline: u8, inner_glow: u8, shadow: u8,
    beat: f32, energy: f32,
    sub_bass: f32, bass: f32, mid: f32, treble: f32,
    palette: u8,
) -> (u8, u8, u8) {
    let mut r = base_r as f32;
    let mut g = base_g as f32;
    let mut b = base_b as f32;

    // Track zones
    let on_edge = glow > 80;
    let inside = inner_glow > 20 || glow > 40;

    // ── depth_t: 0.0 = far face (dark), 1.0 = near face (bright) ──
    let depth_t = depth as f32 / 255.0;

    // ══════════════════════════════════════════════════════════════
    // 1. Shadow below shape (floor shadow)
    // ══════════════════════════════════════════════════════════════
    if shadow > 5 {
        let s = 1.0 - (shadow as f32 / 255.0) * 0.6;
        r *= s; g *= s; b *= s;
    }

    // ══════════════════════════════════════════════════════════════
    // 2. Scanline rays below center
    // ══════════════════════════════════════════════════════════════
    if scanline > 3 {
        let ray = scanline as f32 / 255.0;
        // Tint scanlines with palette color
        let (pr, pg, pb) = get_palette_color(palette, 0.3);
        r = (r + ray * pr * 0.3).min(255.0);
        g = (g + ray * pg * 0.3).min(255.0);
        b = (b + ray * pb * 0.3).min(255.0);
    }

    // ══════════════════════════════════════════════════════════════
    // 3. EDGES → push toward WHITE (bright contours)
    //    Wireframe edges are always white/near-white regardless of palette
    // ══════════════════════════════════════════════════════════════
    if on_edge {
        let g_f = glow as f32 / 255.0;

        // Depth-based edge brightness: near edges brighter, far edges dimmer
        let edge_brightness = 0.55 + 0.45 * depth_t; // 0.55..1.0

        // Strong push toward white for edges
        let white_blend = g_f * edge_brightness;
        r = (r * (1.0 - white_blend) + 255.0 * white_blend).min(255.0);
        g = (g * (1.0 - white_blend) + 255.0 * white_blend).min(255.0);
        b = (b * (1.0 - white_blend) + 255.0 * white_blend).min(255.0);

        // Subtle palette tint on edges (just a hint, 15%) so they're not pure white
        let (pr, pg, pb) = get_palette_color(palette, depth_t);
        let tint = 0.15 * g_f;
        r = (r * (1.0 - tint) + pr * tint).min(255.0);
        g = (g * (1.0 - tint) + pg * tint).min(255.0);
        b = (b * (1.0 - tint) + pb * tint).min(255.0);

        // AO: darken at poles
        if ao > 0 {
            let ao_factor = 1.0 - (ao as f32 / 255.0) * 0.3;
            r *= ao_factor; g *= ao_factor; b *= ao_factor;
        }
    }
    // ══════════════════════════════════════════════════════════════
    // 4. FACES → Palette gradient colored by DEPTH (shadow illusion)
    //    Far faces (depth_t~0) = dark saturated palette start
    //    Near faces (depth_t~1) = bright desaturated palette end
    // ══════════════════════════════════════════════════════════════
    else if inner_glow > 20 {
        let ig = (inner_glow as f32 - 20.0) / 235.0;
        let ig = ig * ig; // quadratic falloff

        // ── Depth-based shadow: simulate light coming from front-top ──
        // depth_t: 0 = far (shadows), 1 = near (lit)
        // shadow_factor: 0.3 (deep shadow) to 1.0 (fully lit)
        let shadow_factor = 0.3 + 0.7 * depth_t;

        // ── Palette gradient across depth ──
        // Near faces get bright end of palette, far faces get dark end
        let (pr, pg, pb) = get_palette_color(palette, depth_t);

        // Apply shadow factor to palette color
        let lit_r = pr * shadow_factor;
        let lit_g = pg * shadow_factor;
        let lit_b = pb * shadow_factor;

        // Blend strength: deeper inside = stronger color
        let blend = (ig * 0.80).min(0.80);
        r = (r * (1.0 - blend) + lit_r * blend).min(255.0);
        g = (g * (1.0 - blend) + lit_g * blend).min(255.0);
        b = (b * (1.0 - blend) + lit_b * blend).min(255.0);

        // ── AO: extra darkening at top/bottom poles (ambient occlusion) ──
        if ao > 0 {
            let ao_factor = 1.0 - (ao as f32 / 255.0) * 0.5;
            r *= ao_factor; g *= ao_factor; b *= ao_factor;
        }

        // ── Subtle brightness boost for very near faces (highlight rim) ──
        if depth_t > 0.8 {
            let near_boost = (depth_t - 0.8) / 0.2 * 30.0;
            r = (r + near_boost).min(255.0);
            g = (g + near_boost).min(255.0);
            b = (b + near_boost).min(255.0);
        }
    }
    // ══════════════════════════════════════════════════════════════
    // 5. Weak glow (near edges but not ON edge) — soft palette tint
    // ══════════════════════════════════════════════════════════════
    else if glow > 0 {
        let g_f = glow as f32 / 255.0;
        let (pr, pg, pb) = get_palette_color(palette, depth_t * 0.5 + 0.3);
        let boost = g_f * 0.5;
        r = (r + pr * boost).min(255.0);
        g = (g + pg * boost).min(255.0);
        b = (b + pb * boost).min(255.0);
    }

    // ══════════════════════════════════════════════════════════════
    // 6. FRESNEL → bright white silhouette edges (always white)
    // ══════════════════════════════════════════════════════════════
    if fresnel > 120 {
        let f_t = (fresnel as f32 - 120.0) / 135.0;
        let white_push = f_t * f_t;
        r = (r * (1.0 - white_push) + 252.0 * white_push).min(255.0);
        g = (g * (1.0 - white_push) + 255.0 * white_push).min(255.0);
        b = (b * (1.0 - white_push) + 252.0 * white_push).min(255.0);
    }

    // ══════════════════════════════════════════════════════════════
    // 7. SPECULAR → white hot spots (simulated light reflection)
    // ══════════════════════════════════════════════════════════════
    if specular > 30 {
        let s_t = (specular as f32 - 30.0) / 225.0;
        let s_t = s_t * s_t;
        // Specular goes toward white but with slight palette tint
        let (pr, pg, pb) = get_palette_color(palette, 0.9);
        r = (r + s_t * (200.0 + pr * 0.2)).min(255.0);
        g = (g + s_t * (220.0 + pg * 0.1)).min(255.0);
        b = (b + s_t * (200.0 + pb * 0.2)).min(255.0);
    }

    // ══════════════════════════════════════════════════════════════
    // 8. BLOOM → soft glow halo with palette tint
    // ══════════════════════════════════════════════════════════════
    if bloom > 10 {
        let bl = bloom as f32 / 255.0;
        let (pr, pg, pb) = get_palette_color(palette, 0.5);
        r = (r + bl * pr * 0.15).min(255.0);
        g = (g + bl * pg * 0.15).min(255.0);
        b = (b + bl * pb * 0.15).min(255.0);
    }

    // ══════════════════════════════════════════════════════════════
    // 9. RIPPLE → beat ring with palette color
    // ══════════════════════════════════════════════════════════════
    if ripple > 0 {
        let rip = ripple as f32 / 255.0;
        let (pr, pg, pb) = get_palette_color(palette, 0.6);
        r = (r + rip * pr * 0.2).min(255.0);
        g = (g + rip * pg * 0.2).min(255.0);
        b = (b + rip * pb * 0.2).min(255.0);
    }

    // ══════════════════════════════════════════════════════════════
    // 10. Music-reactive color boost — intensifies palette on beat
    // ══════════════════════════════════════════════════════════════
    if inside && energy > 0.03 {
        let maximum_band = sub_bass.max(bass).max(mid).max(treble);
        if maximum_band > 0.10 {
            let intensity = (energy * 0.8 + 0.1).min(0.55);

            // Map frequency band to palette position
            let band_t = if sub_bass >= maximum_band - 0.05 {
                0.1f32
            } else if bass >= maximum_band - 0.05 {
                0.35
            } else if mid >= maximum_band - 0.05 {
                0.65
            } else {
                0.9
            };

            // Blend band color with depth position for richer variation
            let mix_t = band_t * 0.6 + depth_t * 0.4;
            let (tr, tg, tb) = get_palette_color(palette, mix_t);

            let pulse = 1.0 + beat * 0.5;
            let blend = (intensity * pulse).min(0.65);
            r = (r * (1.0 - blend) + tr * blend).min(255.0);
            g = (g * (1.0 - blend) + tg * blend).min(255.0);
            b = (b * (1.0 - blend) + tb * blend).min(255.0);

            // Beat kick: flash brighter
            if beat > 0.5 {
                let kick = (beat - 0.5) * 45.0;
                r = (r + kick).min(255.0);
                g = (g + kick).min(255.0);
                b = (b + kick).min(255.0);
            }
        }
    }

    // ══════════════════════════════════════════════════════════════
    // 11. Final output with palette-aware background tinting
    // ══════════════════════════════════════════════════════════════
    if inside {
        (r.min(255.0) as u8, g.min(255.0) as u8, b.min(255.0) as u8)
    } else if palette == 0 {
        // Matrix: green-dominant outside
        let g_final = g as u8;
        let r_final = (r as u8).min(g_final);
        let b_final = (b as u8).min(g_final);
        (r_final, g_final, b_final)
    } else {
        // Other palettes: subtle tint on rain
        let (pr, pg, pb) = get_palette_color(palette, 0.15);
        let tint = 0.18f32;
        let r_final = (r * (1.0 - tint) + pr * tint).min(255.0) as u8;
        let g_final = (g * (1.0 - tint) + pg * tint).min(255.0) as u8;
        let b_final = (b * (1.0 - tint) + pb * tint).min(255.0) as u8;
        (r_final, g_final, b_final)
    }
}

// ═══════════════════════════════════════
// Column slow factor
// ═══════════════════════════════════════

#[inline]
// Public function — callable from other modules.
pub fn column_slow_factor(state: &VisualizerState, col: usize) -> u8 {
    if col < state.column_bounds.len() {
        let (bmin, bmax) = state.column_bounds[col];
        if bmin >= 0 && bmax > bmin {
            let span = (bmax - bmin) as u32;
            let slow = (45 + (55 * span / 400).min(55)) as u8;
            return 100u8.saturating_sub(100 - slow);
        }
    }
    100
}
