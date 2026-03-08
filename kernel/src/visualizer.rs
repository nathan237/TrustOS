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

impl V3 {
    const fn new(x: f32, y: f32, z: f32) -> Self { Self { x, y, z } }
}

#[derive(Clone, Copy)]
struct Edge(u16, u16);

/// Rain interaction result — identical interface to ghost_mesh::RainEffect
#[derive(Clone, Copy)]
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

impl RainEffect {
    pub const NONE: Self = Self {
        glow: 0, depth: 128, trail_boost: 0, ripple: 0, dim: 0,
        fresnel: 0, specular: 0, ao: 0, bloom: 0, scanline: 0,
        inner_glow: 0, shadow: 0,
        target_r: 0, target_g: 0, target_b: 0, target_blend: 0,
    };
}

// ═══════════════════════════════════════
// Visualizer mode enum
// ═══════════════════════════════════════

pub const NUM_MODES: u8 = 8;

pub const MODE_NAMES: [&str; 8] = [
    "Sphere",
    "Morphing",
    "Lorenz",
    "Spectrum",
    "Ribbon",
    "Starburst",
    "Image",
    "FlowField",
];

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

    pub initialized: bool,
}

struct Particle {
    x: f32, y: f32, z: f32,
    vx: f32, vy: f32, vz: f32,
    life: f32,
}

impl VisualizerState {
    pub const fn new() -> Self {
        Self {
            mode: 0,
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
const SPHERE_LON: usize = 12;

fn gen_sphere() -> (Vec<V3>, Vec<Edge>, Vec<u8>) {
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
        let band: u8 = match lat { 0 => 0, 1 => 1, 2..=5 => 2, _ => 3 };
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
fn gen_icosahedron() -> (Vec<V3>, Vec<Edge>) {
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
fn gen_cube() -> (Vec<V3>, Vec<Edge>) {
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
fn gen_diamond() -> (Vec<V3>, Vec<Edge>) {
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
fn gen_star() -> (Vec<V3>, Vec<Edge>) {
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
// Initialization
// ═══════════════════════════════════════

fn ensure_init(s: &mut VisualizerState) {
    if s.initialized { return; }

    // Mode 0 & 3: Sphere + Spectrum Globe (same base geometry)
    let (sv, se, sb) = gen_sphere();
    s.spec_base = sv.clone();
    s.spec_bands = sb.clone();
    s.spec_edges = se.clone();
    s.sphere_base = sv;
    s.sphere_edges = se;
    s.sphere_bands = sb;

    // Mode 1: Morph targets — pad all to same vertex count (max 14)
    let (ico_v, ico_e) = gen_icosahedron();
    let (cube_v, cube_e) = gen_cube();
    let (dia_v, dia_e) = gen_diamond();
    let (star_v, star_e) = gen_star();

    // Find max vertex count
    let max_v = ico_v.len().max(cube_v.len()).max(dia_v.len()).max(star_v.len());

    // Pad shapes to max_v vertices (duplicate last vertex)
    fn pad_verts(v: &[V3], target: usize) -> Vec<V3> {
        let mut out = v.to_vec();
        while out.len() < target {
            out.push(*out.last().unwrap_or(&V3::new(0.0, 0.0, 0.0)));
        }
        out
    }

    s.morph_shapes[0] = pad_verts(&ico_v, max_v);
    s.morph_shapes[1] = pad_verts(&cube_v, max_v);
    s.morph_shapes[2] = pad_verts(&dia_v, max_v);
    s.morph_shapes[3] = pad_verts(&star_v, max_v);
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
    const MAX_TRAIL: usize = 350;
    while s.lorenz_trail.len() > MAX_TRAIL {
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
    let disp_w = logo_w * 3 / 2;
    let disp_h = logo_h * 3 / 2;
    s.image_display_w = disp_w;
    s.image_display_h = disp_h;
    s.image_offset_x = s.center_x - disp_w as i32 / 2;
    s.image_offset_y = s.center_y - disp_h as i32 / 2;

    // Beat makes image pulse slightly
    let pulse = (s.beat_pulse * 40.0) as i32;
    s.image_offset_x -= pulse / 2;
    s.image_offset_y -= pulse / 2;
    s.image_display_w = (disp_w as i32 + pulse) as u32;
    s.image_display_h = (disp_h as i32 + pulse) as u32;

    // Set column bounds based on image extent so fill layers + slow factor work
    let col_w = s.col_w;
    if col_w > 0 {
        let ncols = s.column_bounds.len();
        for col in 0..ncols {
            let col_px = col as i32 * col_w + col_w / 2;
            if col_px >= s.image_offset_x && col_px < s.image_offset_x + s.image_display_w as i32 {
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

    // Beat pulse
    let bass_beat = playing && beat > 0.5 && (sub_bass + bass) > 0.4;
    if bass_beat { state.beat_pulse = 1.0; }
    state.beat_pulse *= 0.90;

    // Beat ripple
    if state.beat_pulse > 0.9 { state.ripple_radius = 0.0; }
    if state.ripple_radius < 600.0 { state.ripple_radius += 6.0; }

    // ── Rotation ──
    let bass_hit = if playing {
        ((state.smooth_sub_bass + state.smooth_bass) * 15.0 + beat * 8.0) as i32
    } else { 0 };
    state.rot_x += state.rot_speed_x + bass_hit / 4;
    state.rot_y += state.rot_speed_y + bass_hit;
    state.rot_z += state.rot_speed_z;
    state.rot_x %= 6283; state.rot_y %= 6283; state.rot_z %= 6283;

    // ── Scale ──
    state.scale_target = if playing {
        150 + ((state.smooth_sub_bass + state.smooth_bass) * 25.0) as i32
            + (state.beat_pulse * 25.0) as i32
    } else { 150 };
    state.scale += (state.scale_target - state.scale) / 3;
    state.scale = state.scale.max(80).min(220);

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
        let z_avg = if a < state.proj_z.len() && b < state.proj_z.len() {
            ((state.proj_z[a] as i32 + state.proj_z[b] as i32) / 2) as i16
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
                hits[ci].push((y_lo, y_hi, eidx, intensity, z_avg));
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
        let rel_x = screen_x - state.image_offset_x;
        let rel_y = y - state.image_offset_y;

        if rel_x >= 0 && rel_x < state.image_display_w as i32
            && rel_y >= 0 && rel_y < state.image_display_h as i32
        {
            let logo_w = crate::logo_bitmap::LOGO_W;
            let logo_h = crate::logo_bitmap::LOGO_H;
            // Nearest-neighbor sample from display size → logo native size
            let img_x = (rel_x as u32 * logo_w as u32 / state.image_display_w) as usize;
            let img_y = (rel_y as u32 * logo_h as u32 / state.image_display_h) as usize;

            if img_x < logo_w && img_y < logo_h {
                let pixel = crate::logo_bitmap::logo_pixel(img_x, img_y);
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
        let rel_x = screen_x - state.image_offset_x;
        let rel_y = y - state.image_offset_y;
        let dx_edge = if rel_x < 0 { -rel_x }
            else if rel_x >= state.image_display_w as i32 { rel_x - state.image_display_w as i32 + 1 }
            else { 0 };
        let dy_edge = if rel_y < 0 { -rel_y }
            else if rel_y >= state.image_display_h as i32 { rel_y - state.image_display_h as i32 + 1 }
            else { 0 };
        let edge_dist = dx_edge.max(dy_edge);
        if edge_dist > 0 && edge_dist < 40 {
            let dim = ((1.0 - edge_dist as f32 / 40.0) * 50.0) as u8;
            return RainEffect { dim, ..RainEffect::NONE };
        }

        return RainEffect::NONE;
    }

    // ═══════════════════════════════════════
    // Modes 0–5: 3D wireframe collision
    // ═══════════════════════════════════════

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
                let max_r = ((bmax - bmin) as f32 / 2.0).max(1.0);
                let t = (1.0 - r / max_r).max(0.0);
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
                target_r: 0, target_g: 0, target_b: 0, target_blend: 0,
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
            let max_r = ((bmax - bmin) as f32 / 2.0).max(1.0);
            let t = (1.0 - r / max_r).max(0.0);
            (t * t * 120.0) as u8
        };

        return RainEffect {
            glow, depth, trail_boost: 5,
            ripple, dim: 0,
            fresnel, specular: 0, ao: 0, bloom: 0,
            scanline: 0, inner_glow, shadow: 0,
            target_r: 0, target_g: 0, target_b: 0, target_blend: 0,
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
) -> (u8, u8, u8) {
    let mut r = base_r as f32;
    let mut g = base_g as f32;
    let mut b = base_b as f32;

    if shadow > 5 {
        let s = 1.0 - (shadow as f32 / 255.0) * 0.6;
        r *= s; g *= s; b *= s;
    }
    if scanline > 3 {
        let ray = scanline as f32 / 255.0;
        r = (r + ray * 5.0).min(255.0);
        g = (g + ray * 50.0).min(255.0);
        b = (b + ray * 5.0).min(255.0);
    }
    if ao > 0 && glow > 0 {
        let ao_factor = 1.0 - (ao as f32 / 255.0) * 0.45;
        r *= ao_factor; g *= ao_factor; b *= ao_factor;
    }
    if glow > 0 {
        let g_f = glow as f32 / 255.0;
        if glow > 80 {
            let depth_scale = 0.4 + 0.6 * (depth as f32 / 255.0);
            let g_f = g_f * depth_scale;
            // Matrix palette: glow is green/white, NEVER yellow
            let tr = (100.0 + energy * 40.0).min(180.0);
            let tg = 255.0f32;
            let tb = (100.0 + energy * 30.0).min(180.0);
            r = (r * (1.0 - g_f) + tr * g_f).min(255.0);
            g = (g * (1.0 - g_f) + tg * g_f).min(255.0);
            b = (b * (1.0 - g_f) + tb * g_f).min(255.0);
        } else {
            let boost = g_f * 2.5;
            // Only boost green channel significantly
            r = (r * (1.0 + boost * 0.3)).min(255.0);
            g = (g * (1.0 + boost)).min(255.0);
            b = (b * (1.0 + boost * 0.3)).min(255.0);
        }
    }
    if inner_glow > 20 {
        let ig = (inner_glow as f32 - 20.0) / 235.0;
        let ig = ig * ig;
        // Interior color: deep inside the shape gets a vivid color tint
        // Color varies by depth: near-face = bright cyan, far-face = deep green
        let depth_t = depth as f32 / 255.0;
        // Interior color palette: cyan-green gradient based on depth
        let int_r = 0.0 + depth_t * 20.0;       // very little red
        let int_g = 120.0 + depth_t * 100.0;     // strong green
        let int_b = 80.0 + (1.0 - depth_t) * 80.0; // cyan bias for near faces
        // Blend toward interior color (stronger for deeper inner_glow)
        let blend = (ig * 0.65).min(0.65); // max 65% color blend
        r = (r * (1.0 - blend) + int_r * blend).min(255.0);
        g = (g * (1.0 - blend) + int_g * blend).min(255.0);
        b = (b * (1.0 - blend) + int_b * blend).min(255.0);
        // Also boost overall brightness inside shape
        let bright_boost = ig * 40.0;
        g = (g + bright_boost).min(255.0);
        b = (b + bright_boost * 0.5).min(255.0);
    }
    if fresnel > 120 {
        let f_t = (fresnel as f32 - 120.0) / 135.0;
        // Edges push strongly toward white (contour effect)
        let white_push = f_t * f_t; // quadratic for sharp edge
        r = (r * (1.0 - white_push) + 220.0 * white_push).min(255.0);
        g = (g * (1.0 - white_push) + 255.0 * white_push).min(255.0);
        b = (b * (1.0 - white_push) + 220.0 * white_push).min(255.0);
    }
    if specular > 30 {
        let s_t = (specular as f32 - 30.0) / 225.0;
        let s_t = s_t * s_t;
        // Specular highlights: near-white hot spots
        r = (r + s_t * 160.0).min(255.0);
        g = (g + s_t * 200.0).min(255.0);
        b = (b + s_t * 160.0).min(255.0);
    }
    if bloom > 10 {
        let bl = bloom as f32 / 255.0;
        r = (r + bl * 15.0).min(255.0);
        g = (g + bl * 55.0).min(255.0);
        b = (b + bl * 15.0).min(255.0);
    }
    if ripple > 0 {
        let rip = ripple as f32 / 255.0;
        r = (r + 5.0 * rip).min(255.0);
        g = (g + 35.0 * rip).min(255.0);
        b = (b + 5.0 * rip).min(255.0);
    }

    // Matrix palette enforcement: R and B must never exceed G
    // This guarantees pure white/green/dark-green/black — no yellow ever
    let g_final = g as u8;
    let r_final = (r as u8).min(g_final);
    let b_final = (b as u8).min(g_final);
    (r_final, g_final, b_final)
}

// ═══════════════════════════════════════
// Column slow factor
// ═══════════════════════════════════════

#[inline]
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
