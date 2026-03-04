// ═══════════════════════════════════════════════════════════════════════════
// Ghost Mesh v4 — 3D sphere revealed ONLY through matrix rain modulation
// ═══════════════════════════════════════════════════════════════════════════
//
// Nothing is drawn on top of the matrix.  The rain IS the renderer.
//
// A procedural sphere rotates invisibly.  The rain reveals it by:
//   • Edge glow    – chars near wireframe edges become bright white-green
//   • Volume fill  – chars inside the silhouette get a subtle brightness boost
//   • Z-depth      – front-face edges glow brighter than back-face edges
//   • Speed warp   – rain columns inside the shape slow down → density
//   • Trail extend – chars near edges get a brightness bonus → longer trails
//   • Beat ripple  – an expanding ring of light radiates from the center
//
// The sphere deforms per-frequency-band: bass pushes bottom vertices,
// treble pushes top.  Beat pulses spike everything outward.
//
// ═══════════════════════════════════════════════════════════════════════════

use alloc::vec::Vec;

// ═══════════════════════════════════════
// Types
// ═══════════════════════════════════════

#[derive(Clone, Copy)]
pub struct V3 { pub x: f32, pub y: f32, pub z: f32 }

impl V3 {
    pub const fn new(x: f32, y: f32, z: f32) -> Self { Self { x, y, z } }
}

#[derive(Clone, Copy)]
pub struct Edge(pub u16, pub u16);

// ═══════════════════════════════════════
// Rain modulation result
// ═══════════════════════════════════════

/// Returned by `check_rain_collision`.  The rain loop in desktop.rs
/// uses every field to modulate brightness, color, trail length, etc.
#[derive(Clone, Copy)]
pub struct RainEffect {
    /// 0 = outside shape,  1-80 = inside volume,  80-255 = near edge.
    pub glow: u8,
    /// 0 = back face,  255 = front face.  Scales edge brightness.
    pub depth: u8,
    /// Additive brightness for trail-extension effect (0-80).
    pub trail_boost: u8,
    /// Beat-ripple ring intensity at this position (0-120).
    pub ripple: u8,
    /// Dimming factor for contrast zone just outside shape (0=normal, 1-200=dim).
    pub dim: u8,
    /// Fresnel rim glow: 0=interior, 255=silhouette edge (0-255).
    pub fresnel: u8,
    /// Specular highlight intensity at this position (0-255).
    pub specular: u8,
    /// Ambient occlusion: 0=fully lit, 255=fully occluded (darkened in creases/poles).
    pub ao: u8,
    /// Bloom: bright glow bleed (0-255). High near bright edges, used for soft spread.
    pub bloom: u8,
    /// Scanline rays: volumetric light ray intensity (0-255) below the shape.
    pub scanline: u8,
    /// Inner glow: warm core emanation from sphere center (0-255).
    pub inner_glow: u8,
    /// Shadow: darkening below the shape (0=normal, 1-200=shadow).
    pub shadow: u8,
}

impl RainEffect {
    pub const NONE: Self = Self {
        glow: 0, depth: 128, trail_boost: 0, ripple: 0, dim: 0,
        fresnel: 0, specular: 0, ao: 0, bloom: 0, scanline: 0,
        inner_glow: 0, shadow: 0,
    };
}

// ═══════════════════════════════════════
// Sphere resolution
// ═══════════════════════════════════════

const NUM_LAT: usize = 8;   // latitude rings
const NUM_LON: usize = 12;  // longitude segments

// ═══════════════════════════════════════
// State
// ═══════════════════════════════════════

pub struct GhostMeshState {
    pub base_verts: Vec<V3>,
    pub deformed_verts: Vec<V3>,
    pub edges: Vec<Edge>,
    pub vert_bands: Vec<u8>,

    pub rot_x: i32,
    pub rot_y: i32,
    pub rot_z: i32,
    pub rot_speed_x: i32,
    pub rot_speed_y: i32,
    pub rot_speed_z: i32,
    pub scale: i32,
    pub scale_target: i32,
    pub center_x: i32,
    pub center_y: i32,

    /// Per-column edge hit map: (y_lo, y_hi, edge_idx, intensity, z_depth).
    pub column_hits: Vec<Vec<(i32, i32, u16, u8, i16)>>,
    /// Per-column bounding box of shape silhouette: (y_min, y_max).
    /// (-1, -1) means no shape in that column.
    pub column_bounds: Vec<(i32, i32)>,

    // ── v4: Z-depth, ripple, column metrics ──
    /// Z coordinate after rotation, per vertex.
    pub projected_z: Vec<i16>,
    /// Expanding beat-ripple ring radius (pixels from center).
    pub ripple_radius: f32,
    /// Min/max Z this frame for depth normalization.
    pub z_min: i16,
    pub z_max: i16,
    /// Pixel width of one matrix column (set each frame).
    pub col_w: i32,

    pub frame: u64,
    pub smooth_sub_bass: f32,
    pub smooth_bass: f32,
    pub smooth_mid: f32,
    pub smooth_treble: f32,
    pub beat_pulse: f32,
    pub initialized: bool,
    /// Specular highlight screen position (projected from light direction)
    pub spec_x: i32,
    pub spec_y: i32,
    /// Shadow zone: y range below the sphere where shadow falls
    pub shadow_y_start: i32,
    pub shadow_y_end: i32,
    /// Center of mass Y (for scanline rays origin)
    pub shape_center_y: i32,
}

impl GhostMeshState {
    pub const fn new() -> Self {
        Self {
            base_verts: Vec::new(),
            deformed_verts: Vec::new(),
            edges: Vec::new(),
            vert_bands: Vec::new(),
            rot_x: 0, rot_y: 0, rot_z: 0,
            rot_speed_x: 6,
            rot_speed_y: 10,
            rot_speed_z: 2,
            scale: 200,
            scale_target: 200,
            center_x: 0, center_y: 0,
            column_hits: Vec::new(),
            column_bounds: Vec::new(),
            projected_z: Vec::new(),
            ripple_radius: 999.0,
            z_min: 0,
            z_max: 0,
            col_w: 8,
            frame: 0,
            smooth_sub_bass: 0.0,
            smooth_bass: 0.0,
            smooth_mid: 0.0,
            smooth_treble: 0.0,
            beat_pulse: 0.0,
            initialized: false,
            spec_x: 0,
            spec_y: 0,
            shadow_y_start: 0,
            shadow_y_end: 0,
            shape_center_y: 0,
        }
    }

    fn ensure_init(&mut self) {
        if self.initialized { return; }
        let (bv, edges, bands) = generate_sphere();
        self.deformed_verts = bv.clone();
        self.base_verts = bv;
        self.edges = edges;
        self.vert_bands = bands;
        self.initialized = true;
    }
}

// ═══════════════════════════════════════
// Sphere Generation
// ═══════════════════════════════════════

fn generate_sphere() -> (Vec<V3>, Vec<Edge>, Vec<u8>) {
    let mut verts = Vec::with_capacity(NUM_LAT * NUM_LON + 2);
    let mut bands = Vec::with_capacity(NUM_LAT * NUM_LON + 2);
    let mut edges = Vec::new();
    let pi: f32 = 3.14159265;

    // South pole
    verts.push(V3::new(0.0, -1.0, 0.0));
    bands.push(0);

    for lat in 0..NUM_LAT {
        let frac = (lat as f32 + 1.0) / (NUM_LAT as f32 + 1.0);
        let angle = -pi / 2.0 + pi * frac;
        let y = sinf(angle);
        let r = cosf(angle);
        let band: u8 = match lat {
            0     => 0,
            1     => 1,
            2 | 3 => 2,
            4 | 5 => 2,
            _     => 3,
        };
        for lon in 0..NUM_LON {
            let la = 2.0 * pi * (lon as f32) / (NUM_LON as f32);
            verts.push(V3::new(r * cosf(la), y, r * sinf(la)));
            bands.push(band);
        }
    }

    // North pole
    verts.push(V3::new(0.0, 1.0, 0.0));
    bands.push(3);
    let np = verts.len() - 1;

    // Ring edges
    for lat in 0..NUM_LAT {
        let base = 1 + lat * NUM_LON;
        for lon in 0..NUM_LON {
            let next = if lon + 1 < NUM_LON { lon + 1 } else { 0 };
            edges.push(Edge((base + lon) as u16, (base + next) as u16));
        }
    }

    // Meridian edges
    for lon in 0..NUM_LON {
        edges.push(Edge(0, (1 + lon) as u16));
        for lat in 0..(NUM_LAT - 1) {
            let a = 1 + lat * NUM_LON + lon;
            let b = 1 + (lat + 1) * NUM_LON + lon;
            edges.push(Edge(a as u16, b as u16));
        }
        edges.push(Edge((1 + (NUM_LAT - 1) * NUM_LON + lon) as u16, np as u16));
    }

    (verts, edges, bands)
}

// ═══════════════════════════════════════
// Math
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

/// Integer square root (Newton's method).
fn isqrt(val: i32) -> i32 {
    if val <= 0 { return 0; }
    let mut x = val;
    let mut y = (x + 1) / 2;
    while y < x { x = y; y = (x + val / x) / 2; }
    x
}

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

// ═══════════════════════════════════════
// Update (once per frame, before rain)
// ═══════════════════════════════════════

pub fn update(
    state: &mut GhostMeshState,
    screen_w: u32, screen_h: u32,
    matrix_cols: usize,
    beat: f32, energy: f32,
    sub_bass: f32, bass: f32, mid: f32, treble: f32,
    playing: bool,
) {
    state.ensure_init();
    state.frame = state.frame.wrapping_add(1);
    state.center_x = screen_w as i32 / 2;
    state.center_y = screen_h as i32 / 3;

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

    // Beat pulse — fires on bass/kick hits, not just any transient
    let bass_beat = playing && beat > 0.5 && (sub_bass + bass) > 0.4;
    if bass_beat { state.beat_pulse = 1.0; }
    state.beat_pulse *= 0.90;  // slightly longer decay for heavier feel

    // ── Beat ripple: restart ring on strong beat ──
    if state.beat_pulse > 0.9 {
        state.ripple_radius = 0.0;
    }
    if state.ripple_radius < 600.0 {
        state.ripple_radius += 6.0;
    }

    // ── Rotation (bass/kick-driven) ──
    let bass_hit = if playing {
        ((state.smooth_sub_bass + state.smooth_bass) * 15.0 + beat * 8.0) as i32
    } else { 0 };
    state.rot_x += state.rot_speed_x + bass_hit / 4;
    state.rot_y += state.rot_speed_y + bass_hit;
    state.rot_z += state.rot_speed_z;
    state.rot_x %= 6283;
    state.rot_y %= 6283;
    state.rot_z %= 6283;

    // ── Scale (pumps with bass/kick, capped to stay on-screen) ──
    state.scale_target = if playing {
        140 + ((state.smooth_sub_bass + state.smooth_bass) * 30.0) as i32
            + (state.beat_pulse * 30.0) as i32
    } else { 140 };
    state.scale += (state.scale_target - state.scale) / 3;
    if state.scale < 80 { state.scale = 80; }
    if state.scale > 220 { state.scale = 220; }

    // ── Deform sphere (bass/drum-dominant) ──
    // Sub-bass & bass push 3-4× harder than mid/treble
    let amps = [
        state.smooth_sub_bass * 1.2,   // 0: sub_bass → strong push
        state.smooth_bass     * 1.0,   // 1: bass/kick → solid push
        state.smooth_mid      * 0.25,  // 2: mid → subtle
        state.smooth_treble   * 0.15,  // 3: treble → almost none
    ];
    let bass_pulse = state.beat_pulse * (0.3 + state.smooth_sub_bass * 0.3 + state.smooth_bass * 0.2);
    for i in 0..state.base_verts.len() {
        let bv = state.base_verts[i];
        let band = state.vert_bands[i] as usize;
        let amp = if band < 4 { amps[band] } else { energy * 0.2 };
        let r = 1.0 + 0.35 * amp + bass_pulse * 0.25;
        state.deformed_verts[i] = V3::new(bv.x * r, bv.y * r, bv.z * r);
    }

    // ── Project + store Z depth per vertex ──
    let (scale, rx, ry, rz) = (state.scale, state.rot_x, state.rot_y, state.rot_z);
    let (cx, cy) = (state.center_x, state.center_y);
    let vcount = state.deformed_verts.len();
    let mut pverts: Vec<(i32, i32)> = Vec::with_capacity(vcount);
    state.projected_z.clear();
    state.projected_z.reserve(vcount);
    let mut zmin: i16 = i16::MAX;
    let mut zmax: i16 = i16::MIN;
    for v in &state.deformed_verts {
        let (x3, y3, z3) = transform_vertex(*v, rx, ry, rz, scale);
        pverts.push(project(x3, y3, z3, cx, cy));
        let z16 = (z3 as i16).max(-500).min(500);
        state.projected_z.push(z16);
        if z16 < zmin { zmin = z16; }
        if z16 > zmax { zmax = z16; }
    }
    state.z_min = zmin;
    state.z_max = zmax;

    // ── Specular highlight: find vertex closest to light direction ──
    // Light comes from upper-right (direction: x=0.5, y=0.7, z=0.5)
    // We find which projected vertex is closest to where a specular
    // reflection would appear (vertex with normal most aligned to light)
    {
        let mut best_dot: i32 = i32::MIN;
        let mut best_sx: i32 = cx;
        let mut best_sy: i32 = cy;
        // Light direction in world space (fixed, upper-right-front)
        let light_x: i32 = 500;  // milli-units
        let light_y: i32 = 700;
        let light_z: i32 = 500;
        for (vi, v) in state.deformed_verts.iter().enumerate() {
            // Transform vertex normal (≈ vertex position for a sphere)
            let (nx, ny, nz) = transform_vertex(*v, rx, ry, rz, 1000);
            // Dot product with light direction
            let dot = (nx * light_x + ny * light_y + nz * light_z) / 1000;
            if dot > best_dot {
                best_dot = dot;
                if vi < pverts.len() {
                    best_sx = pverts[vi].0;
                    best_sy = pverts[vi].1;
                }
            }
        }
        state.spec_x = best_sx;
        state.spec_y = best_sy;
    }

    // ── Shadow zone: darkened area below the shape ──
    // Light from above → shadow cast downward
    {
        // Find global shape bounding box vertically from column_bounds
        // (computed at end of prev frame, good enough for smooth shadow)
        let mut global_bmax: i32 = -1;
        for &(_, bmax) in state.column_bounds.iter() {
            if bmax > global_bmax { global_bmax = bmax; }
        }
        if global_bmax > 0 {
            state.shadow_y_start = global_bmax;
            state.shadow_y_end = global_bmax + 120; // 120px shadow zone
        } else {
            state.shadow_y_start = 0;
            state.shadow_y_end = 0;
        }
        state.shape_center_y = cy;
    }

    // ── Column hit-map (edge proximity) ──
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
        let (x0, y0) = pverts[edge.0 as usize];
        let (x1, y1) = pverts[edge.1 as usize];
        // Average Z of the two vertices → depth hint for front/back glow
        let z_avg = if (edge.0 as usize) < state.projected_z.len()
                     && (edge.1 as usize) < state.projected_z.len() {
            ((state.projected_z[edge.0 as usize] as i32
            + state.projected_z[edge.1 as usize] as i32) / 2) as i16
        } else { 0 };
        rasterize_edge(x0, y0, x1, y1, ei as u16, z_avg, col_w, matrix_cols,
                       screen_h as i32, &mut state.column_hits, &mut state.column_bounds);
    }
}

fn rasterize_edge(
    x0: i32, y0: i32, x1: i32, y1: i32,
    eidx: u16, z_avg: i16, col_w: i32, ncols: usize, sh: i32,
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

    // Wider proximity: 8 pixels for thicker edge detection
    let prox = 8i32;

    let stride = 2;
    let mut s = 0;
    while s <= steps {
        let sx = px / 1024;
        let sy = py / 1024;
        let c = sx / col_w;
        if c >= 0 && (c as usize) < ncols && sy >= 0 && sy < sh {
            let ci = c as usize;
            // Edge hits (with z-depth)
            if hits[ci].len() < 64 {
                let y_lo = (sy - prox).max(0);
                let y_hi = (sy + prox).min(sh - 1);
                hits[ci].push((y_lo, y_hi, eidx, 255, z_avg));
            }
            // Expand bounding box for volume fill
            let (ref mut bmin, ref mut bmax) = bounds[ci];
            if *bmin < 0 || sy < *bmin { *bmin = sy; }
            if *bmax < 0 || sy > *bmax { *bmax = sy; }
        }
        px += step_x * stride;
        py += step_y * stride;
        s += stride;
    }
}

// ═══════════════════════════════════════
// Rain Collision (the ONLY visual effect)
// ═══════════════════════════════════════

/// Check rain char at (col, pixel_y) against the ghost shape.
/// Returns a full `RainEffect` with glow, z-depth, trail boost and ripple.
#[inline]
pub fn check_rain_collision(
    state: &GhostMeshState,
    col: usize,
    pixel_y: i32,
    beat_pulse: f32,
    energy: f32,
) -> RainEffect {
    if col >= state.column_hits.len() { return RainEffect::NONE; }

    // ── Beat ripple ring ──
    let pixel_x = col as i32 * state.col_w + state.col_w / 2;
    let dx = pixel_x - state.center_x;
    let dy = pixel_y - state.center_y;
    let dist = isqrt(dx * dx + dy * dy);
    let ring_dist = (dist - state.ripple_radius as i32).abs();
    let ripple_width = 35i32;
    let ripple_val = if state.ripple_radius > 0.0 && state.ripple_radius < 550.0
                      && ring_dist < ripple_width {
        let fade = ((ripple_width - ring_dist) * 255 / ripple_width) as u32;
        let life = ((550.0 - state.ripple_radius) / 550.0).max(0.0);
        (fade as f32 * life * 0.45).min(120.0) as u8
    } else { 0u8 };

    // ── Edge check (strong glow) ──
    let hits = &state.column_hits[col];
    let mut best_edge_glow: u8 = 0;
    let mut best_z: i16 = 0;
    for &(y_lo, y_hi, _eidx, intensity, z_val) in hits {
        if pixel_y >= y_lo && pixel_y <= y_hi {
            let center = (y_lo + y_hi) / 2;
            let dist = (pixel_y - center).abs();
            let half = ((y_hi - y_lo) / 2).max(1);
            let fade = ((half - dist) * intensity as i32 / half).max(0) as u8;
            if fade > best_edge_glow {
                best_edge_glow = fade;
                best_z = z_val;
            }
        }
    }

    if best_edge_glow > 0 {
        // Boost with beat pulse
        let boosted = (best_edge_glow as u32 + (beat_pulse * 60.0) as u32).min(255) as u8;

        // Z-depth → 0 (back) to 255 (front)
        let z_range = (state.z_max - state.z_min).max(1) as i32;
        let z_norm = ((state.z_max as i32 - best_z as i32) * 255 / z_range).max(0).min(255) as u8;

        // Fresnel: how close is this pixel to the silhouette boundary?
        // Pixels at bmin/bmax = max fresnel (rim), center = 0
        let fresnel_val = {
            let (bmin, bmax) = state.column_bounds[col];
            if bmin >= 0 && bmax > bmin {
                let half = ((bmax - bmin) / 2).max(1);
                let center_y = (bmin + bmax) / 2;
                let from_center = (pixel_y - center_y).abs();
                // Quadratic: rises fast near edges
                let rim_t = (from_center as f32 / half as f32).min(1.0);
                (rim_t * rim_t * 255.0) as u8
            } else { 0u8 }
        };

        // Specular: distance from specular hotspot
        let spec_val = {
            let sdx = pixel_x - state.spec_x;
            let sdy = pixel_y - state.spec_y;
            let sdist = isqrt(sdx * sdx + sdy * sdy);
            let spec_radius = 45i32;
            if sdist < spec_radius {
                let t = (spec_radius - sdist) as f32 / spec_radius as f32;
                (t * t * 255.0) as u8
            } else { 0u8 }
        };

        // Trail boost: near-edge chars get extra brightness
        let trail_b = (best_edge_glow as u32 * 80 / 255).min(80) as u8;

        // ── Ambient Occlusion: darken near poles (top/bottom of silhouette) ──
        let ao_val = {
            let (bmin, bmax) = state.column_bounds[col];
            if bmin >= 0 && bmax > bmin {
                let half = ((bmax - bmin) / 2).max(1);
                let center_y = (bmin + bmax) / 2;
                let from_center = (pixel_y - center_y).abs();
                // Near edges = more occlusion (creases/poles)
                let edge_t = (from_center as f32 / half as f32).min(1.0);
                // Also darken back faces more
                let back_t = 1.0 - (z_norm as f32 / 255.0); // 1=back, 0=front
                ((edge_t * 0.4 + back_t * 0.3) * 120.0).min(120.0) as u8
            } else { 0u8 }
        };

        // ── Bloom: bright edges bleed light ──
        let bloom_val = if boosted > 100 {
            ((boosted as u32 - 100) * 200 / 155).min(200) as u8
        } else { 0u8 };

        // ── Scanline rays: vertical light rays below shape ──
        let scanline_val = 0u8; // only active outside shape (below)

        // ── Inner glow: warm core emanation ──
        let inner_glow_val = {
            let dx_c = pixel_x - state.center_x;
            let dy_c = pixel_y - state.shape_center_y;
            let d_sq = dx_c * dx_c + dy_c * dy_c;
            let radius = 80i32;
            let r_sq = radius * radius;
            if d_sq < r_sq {
                let t = 1.0 - (d_sq as f32 / r_sq as f32);
                (t * 160.0) as u8
            } else { 0u8 }
        };

        return RainEffect {
            glow: boosted,
            depth: z_norm,
            trail_boost: trail_b,
            ripple: ripple_val,
            dim: 0,
            fresnel: fresnel_val,
            specular: spec_val,
            ao: ao_val,
            bloom: bloom_val,
            scanline: scanline_val,
            inner_glow: inner_glow_val,
            shadow: 0,
        };
    }

    // ── Volume fill check (inside silhouette) ──
    let (bmin, bmax) = state.column_bounds[col];
    if bmin >= 0 && bmax > bmin && pixel_y >= bmin && pixel_y <= bmax {
        let to_top = pixel_y - bmin;
        let to_bot = bmax - pixel_y;
        let to_edge = to_top.min(to_bot);
        let half = (bmax - bmin) / 2;
        if half <= 0 {
            return RainEffect { glow: 0, depth: 128, trail_boost: 0, ripple: ripple_val, dim: 0,
                fresnel: 0, specular: 0, ao: 0, bloom: 0, scanline: 0, inner_glow: 0, shadow: 0 };
        }
        // Fresnel for volume fill too
        let center_y = (bmin + bmax) / 2;
        let from_center = (pixel_y - center_y).abs();
        let rim_t = (from_center as f32 / half as f32).min(1.0);
        let fresnel_vol = (rim_t * rim_t * 180.0) as u8;

        // Specular for volume fill
        let spec_vol = {
            let sdx = pixel_x - state.spec_x;
            let sdy = pixel_y - state.spec_y;
            let sdist = isqrt(sdx * sdx + sdy * sdy);
            let spec_radius = 55i32; // wider for volume
            if sdist < spec_radius {
                let t = (spec_radius - sdist) as f32 / spec_radius as f32;
                (t * t * 200.0) as u8
            } else { 0u8 }
        };

        let base_fill = 15u32 + 25 * (half - to_edge).max(0) as u32 / half as u32;
        let boosted = (base_fill + (energy * 15.0) as u32 + (beat_pulse * 25.0) as u32).min(75) as u8;

        // ── AO for volume: near edges and back-face darken ──
        let ao_vol = {
            let edge_t = (from_center as f32 / half as f32).min(1.0);
            (edge_t * 0.5 * 100.0).min(100.0) as u8
        };

        // ── Bloom for volume ──
        let bloom_vol = if boosted > 50 {
            ((boosted as u32 - 50) * 80 / 25).min(80) as u8
        } else { 0u8 };

        // ── Inner glow: strongest at center of sphere ──
        let inner_glow_vol = {
            let dx_c = pixel_x - state.center_x;
            let dy_c = pixel_y - state.shape_center_y;
            let d_sq = dx_c * dx_c + dy_c * dy_c;
            let radius = 100i32;
            let r_sq = radius * radius;
            if d_sq < r_sq {
                let t = 1.0 - (d_sq as f32 / r_sq as f32);
                (t * 200.0) as u8
            } else { 0u8 }
        };

        return RainEffect {
            glow: boosted,
            depth: 128,
            trail_boost: 20,
            ripple: ripple_val,
            dim: 0,
            fresnel: fresnel_vol,
            specular: spec_vol,
            ao: ao_vol,
            bloom: bloom_vol,
            scanline: 0,
            inner_glow: inner_glow_vol,
            shadow: 0,
        };
    }

    // ── Contrast dim zone: chars just outside the shape boundary ──
    // Creates a dark "halo" that makes the bright shape pop
    if bmin >= 0 && bmax > bmin {
        let margin = 60i32; // pixels of dim zone outside boundary
        let dist_outside = if pixel_y < bmin {
            bmin - pixel_y
        } else if pixel_y > bmax {
            pixel_y - bmax
        } else { 0 };
        if dist_outside > 0 && dist_outside < margin {
            // Smooth fade: strongest dim nearest edge, fades to normal
            let dim_val = ((margin - dist_outside) * 160 / margin) as u8;
            return RainEffect {
                glow: 0, depth: 128, trail_boost: 0, ripple: ripple_val,
                dim: dim_val, fresnel: 0, specular: 0,
                ao: 0, bloom: 0, scanline: 0, inner_glow: 0, shadow: 0,
            };
        }
    }

    // ── Shadow zone: darkened area below the shape ──
    if state.shadow_y_start > 0 && pixel_y > state.shadow_y_start && pixel_y < state.shadow_y_end {
        // Check if this column is within the shape's horizontal span
        let (bmin, bmax) = state.column_bounds[col];
        // Shadow extends from columns that had shape above
        // Use adjacent columns' bounds for a wider, softer shadow
        let has_shape_above = bmin >= 0 || {
            let left = if col > 0 { state.column_bounds[col - 1].0 >= 0 } else { false };
            let right = if col + 1 < state.column_bounds.len() { state.column_bounds[col + 1].0 >= 0 } else { false };
            left || right
        };
        if has_shape_above {
            let shadow_depth = state.shadow_y_end - state.shadow_y_start;
            let from_start = pixel_y - state.shadow_y_start;
            let shadow_t = 1.0 - (from_start as f32 / shadow_depth as f32);
            let shadow_val = (shadow_t * shadow_t * 140.0) as u8; // quadratic fade
            if shadow_val > 5 {
                return RainEffect {
                    glow: 0, depth: 128, trail_boost: 0, ripple: ripple_val,
                    dim: 0, fresnel: 0, specular: 0,
                    ao: 0, bloom: 0, scanline: 0, inner_glow: 0, shadow: shadow_val,
                };
            }
        }
    }

    // ── Scanline rays: vertical light shafts below the shape ──
    if state.shadow_y_start > 0 && pixel_y > state.shadow_y_start && pixel_y < state.shadow_y_start + 200 {
        // Only a few columns get rays (every ~10th column near center)
        let col_center = state.center_x / state.col_w.max(1);
        let col_dist = (col as i32 - col_center).abs();
        if col_dist < 15 && (col % 4 == 0 || col % 4 == 1) {
            let shaft_depth = pixel_y - state.shadow_y_start;
            let fade = 1.0 - (shaft_depth as f32 / 200.0);
            let ray_val = (fade * fade * 80.0 * energy) as u8;
            if ray_val > 3 {
                return RainEffect {
                    glow: 0, depth: 128, trail_boost: 0, ripple: ripple_val,
                    dim: 0, fresnel: 0, specular: 0,
                    ao: 0, bloom: 0, scanline: ray_val, inner_glow: 0, shadow: 0,
                };
            }
        }
    }

    // Outside shape — ripple may still be visible
    if ripple_val > 0 {
        return RainEffect { glow: 0, depth: 128, trail_boost: 0, ripple: ripple_val, dim: 0,
            fresnel: 0, specular: 0, ao: 0, bloom: 0, scanline: 0, inner_glow: 0, shadow: 0 };
    }

    RainEffect::NONE
}

// ═══════════════════════════════════════
// Color modulation for rain chars
// ═══════════════════════════════════════

/// Modify a rain character's RGB based on ghost mesh proximity.
/// `depth` scales edge brightness: 255 = front (brightest), 0 = back (dimmest).
/// `ripple` adds a cyan-white flash from the beat-ripple ring.
#[inline]
pub fn modulate_rain_color(
    base_r: u8, base_g: u8, base_b: u8,
    glow: u8, depth: u8, ripple: u8,
    fresnel: u8, specular: u8,
    ao: u8, bloom: u8, scanline: u8, inner_glow: u8, shadow: u8,
    beat: f32, energy: f32,
) -> (u8, u8, u8) {
    let mut r = base_r as f32;
    let mut g = base_g as f32;
    let mut b = base_b as f32;

    // ── Shadow: darken rain below the shape ──
    if shadow > 5 {
        let s = 1.0 - (shadow as f32 / 255.0) * 0.6;
        r *= s; g *= s; b *= s;
    }

    // ── Scanline rays: vertical light shafts (green-tinted) ──
    if scanline > 3 {
        let ray = scanline as f32 / 255.0;
        r = (r + ray * 15.0).min(255.0);
        g = (g + ray * 50.0).min(255.0);
        b = (b + ray * 25.0).min(255.0);
    }

    // ── Ambient Occlusion: darken creases and back faces ──
    if ao > 0 && glow > 0 {
        let ao_factor = 1.0 - (ao as f32 / 255.0) * 0.45;
        r *= ao_factor;
        g *= ao_factor;
        b *= ao_factor;
    }

    if glow > 0 {
        let g_f = glow as f32 / 255.0;

        if glow > 80 {
            // ── Strong edge glow → shift toward bright white-green ──
            let depth_scale = 0.4 + 0.6 * (depth as f32 / 255.0);
            let g_f = g_f * depth_scale;
            let shift = beat * 0.2;
            let tr = (140.0 + shift * 80.0 + energy * 60.0).min(255.0);
            let tg = 255.0f32;
            let tb = (190.0 + shift * 40.0 + energy * 30.0).min(255.0);
            r = (r * (1.0 - g_f) + tr * g_f).min(255.0);
            g = (g * (1.0 - g_f) + tg * g_f).min(255.0);
            b = (b * (1.0 - g_f) + tb * g_f).min(255.0);
        } else {
            // ── Volume fill → subtle brightness boost ──
            let boost = g_f * 2.5;
            r = (r * (1.0 + boost)).min(255.0);
            g = (g * (1.0 + boost)).min(255.0);
            b = (b * (1.0 + boost)).min(255.0);
        }
    }

    // ── Inner glow: warm green-cyan core emanation ──
    if inner_glow > 20 {
        let ig = (inner_glow as f32 - 20.0) / 235.0;
        let ig = ig * ig; // quadratic for soft falloff
        // Warm green-cyan with a hint of white
        r = (r + ig * 30.0).min(255.0);
        g = (g + ig * 70.0).min(255.0);
        b = (b + ig * 45.0).min(255.0);
    }

    // ── Fresnel rim glow: bright cyan-white halo at silhouette edges ──
    if fresnel > 120 {
        let f_t = (fresnel as f32 - 120.0) / 135.0;
        r = (r + f_t * 80.0).min(255.0);
        g = (g + f_t * 120.0).min(255.0);
        b = (b + f_t * 100.0).min(255.0);
    }

    // ── Specular highlight: bright white hotspot ──
    if specular > 30 {
        let s_t = (specular as f32 - 30.0) / 225.0;
        let s_t = s_t * s_t;
        r = (r + s_t * 180.0).min(255.0);
        g = (g + s_t * 200.0).min(255.0);
        b = (b + s_t * 190.0).min(255.0);
    }

    // ── Bloom: soft glow bleed from bright areas ──
    if bloom > 10 {
        let bl = bloom as f32 / 255.0;
        // Bloom adds a gentle uniform bright wash
        r = (r + bl * 40.0).min(255.0);
        g = (g + bl * 55.0).min(255.0);
        b = (b + bl * 45.0).min(255.0);
    }

    // ── Beat ripple → gentle cyan tint ──
    if ripple > 0 {
        let rip = ripple as f32 / 255.0;
        r = (r + 20.0 * rip).min(255.0);
        g = (g + 35.0 * rip).min(255.0);
        b = (b + 30.0 * rip).min(255.0);
    }

    (r as u8, g as u8, b as u8)
}

// ═══════════════════════════════════════
// Per-column speed factor
// ═══════════════════════════════════════

/// Returns a speed percentage (45-100) for the given column.
/// Columns intersecting the shape slow down → rain accumulates → denser.
#[inline]
pub fn column_slow_factor(state: &GhostMeshState, col: usize) -> u8 {
    if col >= state.column_bounds.len() { return 100; }
    let (bmin, bmax) = state.column_bounds[col];
    if bmin < 0 || bmax <= bmin { return 100; }
    // More vertical coverage → slower.  Range: 100 (no overlap) → 45 (full).
    let coverage = ((bmax - bmin) as u32).min(400);
    let pct = 100u32.saturating_sub(coverage * 55 / 400);
    pct.max(45) as u8
}
