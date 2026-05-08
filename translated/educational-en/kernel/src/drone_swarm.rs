// ═══════════════════════════════════════════════════════════════════════════
// Drone Swarm — Holographic wireframe formations rendered through matrix rain
// ═══════════════════════════════════════════════════════════════════════════
//
// A swarm of "virtual drones" arranged in 3D formations.
// Each drone is a point in 3D space. Edges connect drones to form wireframes.
// The swarm smoothly transitions between choreographed formations.
//
// Rendering pipeline (per frame):
//   1. Advance choreography timer; on timeout, switch formation
//   2. Lerp all drone positions toward their target positions (exponential ease)
//   3. Apply slow global scene rotation (Y tumble + X nod)
//   4. Perspective-project all drones to 2D screen space
//   5. Rasterize edges + vertices into a low-res "glow buffer" (~96×54 cells)
//   6. Per-glyph: sample glow buffer → O(1) brightness + color modulation
//
// Color depends on choreography time: slowly cycles through a holographic palette.
// Depth: far edges project dimmer; perspective projection gives convergence.
//
// Formations (all procedurally generated):
//   0: DNA Double Helix     — two intertwined spirals with connecting rungs
//   1: Tesseract            — 4D hypercube projected to 3D (16 vertices, 32 edges)
//   2: Orbital Atom         — 3 interlocking elliptical orbits with nucleus
//   3: Spiral Galaxy        — logarithmic spiral arms with central core
//   4: Crystal Lattice      — 3D grid of connected nodes
//   5: Infinity Lemniscate  — figure-8 Möbius-like flowing form
//
// Memory: ~26 KB fixed arrays. No heap allocation. const fn constructible.
// Performance: ~6K cell writes/frame for glow buffer + O(1) per glyph query.
// ═══════════════════════════════════════════════════════════════════════════

pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const MAXIMUM_DRONES: usize = 128;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const MAXIMUM_EDGES: usize = 256;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const NUMBER_FORMATIONS: usize = 6;

// Low-resolution glow buffer for O(1) per-glyph queries
const GLOW_W: usize = 96;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const GLOW_H: usize = 54;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const GLOW_SIZE: usize = 5184; // GLOW_W * GLOW_H

// Choreography timing (in frames, ~30fps)
const HOLD_FRAMES: f32 = 300.0;   // 10s hold per formation
const LERP_SPEED: f32 = 0.04;     // exponential ease toward target

// ═══════════════════════════════════════
// State
// ═══════════════════════════════════════

pub struct DroneSwarmState {
    pub initialized: bool,
    screen_w: f32,
    screen_h: f32,

    // Current drone positions (normalized -1..1, lerping toward target)
    drone_x: [f32; MAXIMUM_DRONES],
    drone_y: [f32; MAXIMUM_DRONES],
    drone_z: [f32; MAXIMUM_DRONES],

    // Target positions (current formation endpoints)
    target_x: [f32; MAXIMUM_DRONES],
    target_y: [f32; MAXIMUM_DRONES],
    target_z: [f32; MAXIMUM_DRONES],

    // 2D projected screen positions (recomputed each frame)
    proj_x: [f32; MAXIMUM_DRONES],
    proj_y: [f32; MAXIMUM_DRONES],
    proj_depth: [f32; MAXIMUM_DRONES],

    num_drones: usize,

    // Edge connectivity (index pairs)
    edge_a: [u8; MAXIMUM_EDGES],
    edge_b: [u8; MAXIMUM_EDGES],
    num_edges: usize,

    // Global scene rotation (slow continuous tumble)
    scene_rot_y: f32,
    scene_rot_x: f32,

    // Choreography state
    formation_idx: u8,
    state_timer: f32,

    // Total elapsed frames for color cycling
    total_time: f32,

    // Low-res glow buffer (cleared+rebuilt each frame)
    glow: [f32; GLOW_SIZE],

    // Glow buffer cell dimensions (pixels per cell)
    cell_w: f32,
    cell_h: f32,

    frame: u64,
}

// Implementation block — defines methods for the type above.
impl DroneSwarmState {
    pub const fn new() -> Self {
        Self {
            initialized: false,
            screen_w: 0.0,
            screen_h: 0.0,
            drone_x: [0.0; MAXIMUM_DRONES],
            drone_y: [0.0; MAXIMUM_DRONES],
            drone_z: [0.0; MAXIMUM_DRONES],
            target_x: [0.0; MAXIMUM_DRONES],
            target_y: [0.0; MAXIMUM_DRONES],
            target_z: [0.0; MAXIMUM_DRONES],
            proj_x: [0.0; MAXIMUM_DRONES],
            proj_y: [0.0; MAXIMUM_DRONES],
            proj_depth: [0.0; MAXIMUM_DRONES],
            num_drones: 0,
            edge_a: [0; MAXIMUM_EDGES],
            edge_b: [0; MAXIMUM_EDGES],
            num_edges: 0,
            scene_rot_y: 0.0,
            scene_rot_x: 0.0,
            formation_idx: 0,
            state_timer: 0.0,
            total_time: 0.0,
            glow: [0.0; GLOW_SIZE],
            cell_w: 20.0,
            cell_h: 20.0,
            frame: 0,
        }
    }
}

// ═══════════════════════════════════════
// Initialization
// ═══════════════════════════════════════

pub fn init(s: &mut DroneSwarmState, w: u32, h: u32) {
    s.screen_w = w as f32;
    s.screen_h = h as f32;
    s.cell_w = s.screen_w / GLOW_W as f32;
    s.cell_h = s.screen_h / GLOW_H as f32;

    // Start with first formation
    s.formation_idx = 0;
    build_formation(s, 0);

    // Snap drones to targets instantly
    for i in 0..s.num_drones {
        s.drone_x[i] = s.target_x[i];
        s.drone_y[i] = s.target_y[i];
        s.drone_z[i] = s.target_z[i];
    }

    s.initialized = true;
}

// ═══════════════════════════════════════
// Formation Builders
// ═══════════════════════════════════════

fn build_formation(s: &mut DroneSwarmState, idx: u8) {
    // Clear targets for unused drones
    for i in 0..MAXIMUM_DRONES {
        s.target_x[i] = 0.0;
        s.target_y[i] = 0.0;
        s.target_z[i] = 0.0;
    }
    for i in 0..MAXIMUM_EDGES {
        s.edge_a[i] = 0;
        s.edge_b[i] = 0;
    }
        // Pattern matching — Rust's exhaustive branching construct.
match idx % NUMBER_FORMATIONS as u8 {
        0 => build_helix(s),
        1 => build_tesseract(s),
        2 => build_atom(s),
        3 => build_galaxy(s),
        4 => build_crystal(s),
        _ => build_infinity(s),
    }
}

/// DNA Double Helix — two intertwined spirals with connecting rungs.
fn build_helix(s: &mut DroneSwarmState) {
        // Compile-time constant — evaluated at compilation, zero runtime cost.
const STRAND_N: usize = 32;
    let mut ne = 0usize;

    let turns = 2.5f32;
    let radius = 0.35f32;
    let height = 1.6f32;

    for i in 0..STRAND_N {
        let t = i as f32 / (STRAND_N - 1) as f32;
        let angle = t * turns * 6.2831853;
        let y = -0.8 + t * height;

        // Strand A
        s.target_x[i] = radius * libm::cosf(angle);
        s.target_y[i] = y;
        s.target_z[i] = radius * libm::sinf(angle);

        // Strand B (180° offset)
        s.target_x[STRAND_N + i] = radius * libm::cosf(angle + 3.1415927);
        s.target_y[STRAND_N + i] = y;
        s.target_z[STRAND_N + i] = radius * libm::sinf(angle + 3.1415927);

        // Edges along strands
        if i > 0 {
            s.edge_a[ne] = (i - 1) as u8;
            s.edge_b[ne] = i as u8;
            ne += 1;
            s.edge_a[ne] = (STRAND_N + i - 1) as u8;
            s.edge_b[ne] = (STRAND_N + i) as u8;
            ne += 1;
        }
        // Rungs every 4 drones
        if i % 4 == 0 {
            s.edge_a[ne] = i as u8;
            s.edge_b[ne] = (STRAND_N + i) as u8;
            ne += 1;
        }
    }

    s.num_drones = STRAND_N * 2;
    s.num_edges = ne;
}

/// Tesseract — 4D hypercube projected to 3D (16 vertices, 32 edges).
fn build_tesseract(s: &mut DroneSwarmState) {
    let scale = 0.55f32;

    for bits in 0u8..16 {
        let x4 = if bits & 1 != 0 { 1.0f32 } else { -1.0 };
        let y4 = if bits & 2 != 0 { 1.0f32 } else { -1.0 };
        let z4 = if bits & 4 != 0 { 1.0f32 } else { -1.0 };
        let w4 = if bits & 8 != 0 { 1.0f32 } else { -1.0 };

        let denom = w4 * 0.4 + 1.8;
        s.target_x[bits as usize] = (x4 / denom) * scale;
        s.target_y[bits as usize] = (y4 / denom) * scale;
        s.target_z[bits as usize] = (z4 / denom) * scale;
    }
    s.num_drones = 16;

    // Edges: connect vertices differing in exactly 1 bit
    let mut ne = 0usize;
    for i in 0u8..16 {
        for bit in 0u8..4 {
            let j = i ^ (1 << bit);
            if j > i {
                s.edge_a[ne] = i;
                s.edge_b[ne] = j;
                ne += 1;
            }
        }
    }
    s.num_edges = ne;
}

/// Orbital Atom — 3 interlocking elliptical orbits with nucleus.
fn build_atom(s: &mut DroneSwarmState) {
        // Compile-time constant — evaluated at compilation, zero runtime cost.
const RING_N: usize = 32;
    let mut nd = 0usize;
    let mut ne = 0usize;

    let ring_radius = 0.7f32;

    for ring in 0u8..3 {
        let tilt = // Pattern matching — Rust's exhaustive branching construct.
match ring {
            0 => 0.0f32,
            1 => 1.047f32,  // 60°
            _ => -1.047f32,  // -60°
        };

        let base = nd;
        let cos_t = libm::cosf(tilt);
        let sin_t = libm::sinf(tilt);

        for i in 0..RING_N {
            let a = (i as f32 / RING_N as f32) * 6.2831853;
            let x = ring_radius * libm::cosf(a);
            let y = ring_radius * libm::sinf(a);

            // Rotate ring around X axis by tilt
            s.target_x[nd] = x;
            s.target_y[nd] = y * cos_t;
            s.target_z[nd] = y * sin_t;

            // Edge to next drone in ring
            if i > 0 {
                s.edge_a[ne] = (nd - 1) as u8;
                s.edge_b[ne] = nd as u8;
                ne += 1;
            }
            nd += 1;
        }
        // Close the ring
        s.edge_a[ne] = (nd - 1) as u8;
        s.edge_b[ne] = base as u8;
        ne += 1;
    }

    // 4 nucleus drones
    let nuc_r = 0.08f32;
    let nuc_base = nd;
    let nuc_position: [(f32, f32, f32); 4] = [
        (nuc_r, 0.0, 0.0),
        (-nuc_r, 0.0, 0.0),
        (0.0, nuc_r, 0.0),
        (0.0, -nuc_r, 0.0),
    ];
    for &(nx, ny, nz) in nuc_position.iter() {
        s.target_x[nd] = nx;
        s.target_y[nd] = ny;
        s.target_z[nd] = nz;
        nd += 1;
    }
    // Connect nucleus drones
    for i in 0u8..4 {
        for j in (i + 1)..4 {
            s.edge_a[ne] = (nuc_base as u8) + i;
            s.edge_b[ne] = (nuc_base as u8) + j;
            ne += 1;
        }
    }

    s.num_drones = nd;
    s.num_edges = ne;
}

/// Spiral Galaxy — 4 logarithmic spiral arms with central core.
fn build_galaxy(s: &mut DroneSwarmState) {
        // Compile-time constant — evaluated at compilation, zero runtime cost.
const ARM_N: usize = 24;
        // Compile-time constant — evaluated at compilation, zero runtime cost.
const NUMBER_ARMS: usize = 4;
    let mut nd = 0usize;
    let mut ne = 0usize;

    let base_r = 0.1f32;
    let maximum_r = 0.8f32;

    for arm in 0..NUMBER_ARMS {
        let offset = (arm as f32 / NUMBER_ARMS as f32) * 6.2831853;

        for i in 0..ARM_N {
            let t = i as f32 / (ARM_N - 1) as f32;
            let r = base_r + t * (maximum_r - base_r);
            let theta = offset + t * 3.0;
            let z_wave = libm::sinf(t * 4.0) * 0.1;

            s.target_x[nd] = r * libm::cosf(theta);
            s.target_y[nd] = z_wave;
            s.target_z[nd] = r * libm::sinf(theta);

            if i > 0 {
                s.edge_a[ne] = (nd - 1) as u8;
                s.edge_b[ne] = nd as u8;
                ne += 1;
            }
            nd += 1;
        }
    }

    // Central core: 8 drones
    let core_base = nd;
    for i in 0..8usize {
        let a = (i as f32 / 8.0) * 6.2831853;
        s.target_x[nd] = 0.08 * libm::cosf(a);
        s.target_y[nd] = 0.08 * libm::sinf(a * 1.5);
        s.target_z[nd] = 0.08 * libm::sinf(a);
        nd += 1;
    }
    for i in 0u8..8 {
        s.edge_a[ne] = (core_base as u8) + i;
        s.edge_b[ne] = (core_base as u8) + ((i + 1) % 8);
        ne += 1;
    }

    s.num_drones = nd;
    s.num_edges = ne;
}

/// Crystal Lattice — 4×4×3 grid of connected nodes.
fn build_crystal(s: &mut DroneSwarmState) {
        // Compile-time constant — evaluated at compilation, zero runtime cost.
const NX: usize = 4;
        // Compile-time constant — evaluated at compilation, zero runtime cost.
const NY: usize = 4;
        // Compile-time constant — evaluated at compilation, zero runtime cost.
const NZ: usize = 3;
    let mut nd = 0usize;
    let mut ne = 0usize;

    let spacing = 0.45f32;
    let off_x = -((NX - 1) as f32) * spacing * 0.5;
    let off_y = -((NY - 1) as f32) * spacing * 0.5;
    let off_z = -((NZ - 1) as f32) * spacing * 0.5;

    for iz in 0..NZ {
        for iy in 0..NY {
            for ix in 0..NX {
                s.target_x[nd] = off_x + ix as f32 * spacing;
                s.target_y[nd] = off_y + iy as f32 * spacing;
                s.target_z[nd] = off_z + iz as f32 * spacing;
                nd += 1;
            }
        }
    }

    // Connect to +X, +Y, +Z neighbors
    for iz in 0..NZ {
        for iy in 0..NY {
            for ix in 0..NX {
                let idx = iz * NY * NX + iy * NX + ix;
                if ix + 1 < NX && ne < MAXIMUM_EDGES {
                    s.edge_a[ne] = idx as u8;
                    s.edge_b[ne] = (idx + 1) as u8;
                    ne += 1;
                }
                if iy + 1 < NY && ne < MAXIMUM_EDGES {
                    s.edge_a[ne] = idx as u8;
                    s.edge_b[ne] = (idx + NX) as u8;
                    ne += 1;
                }
                if iz + 1 < NZ && ne < MAXIMUM_EDGES {
                    s.edge_a[ne] = idx as u8;
                    s.edge_b[ne] = (idx + NY * NX) as u8;
                    ne += 1;
                }
            }
        }
    }

    s.num_drones = nd;
    s.num_edges = ne;
}

/// Infinity Lemniscate — figure-8 Möbius-like flowing form.
fn build_infinity(s: &mut DroneSwarmState) {
        // Compile-time constant — evaluated at compilation, zero runtime cost.
const N: usize = 64;
    let r = 0.7f32;

    for i in 0..N {
        let t = (i as f32 / N as f32) * 6.2831853;
        // Lemniscate of Bernoulli in 3D
        s.target_x[i] = r * libm::sinf(t);
        s.target_y[i] = r * libm::sinf(t) * libm::cosf(t);
        s.target_z[i] = r * libm::sinf(2.0 * t) * 0.3;
    }

    let mut ne = 0usize;
    for i in 0..N {
        s.edge_a[ne] = i as u8;
        s.edge_b[ne] = ((i + 1) % N) as u8;
        ne += 1;
    }

    s.num_drones = N;
    s.num_edges = ne;
}

// ═══════════════════════════════════════
// Frame Update
// ═══════════════════════════════════════

pub fn update(s: &mut DroneSwarmState) {
    if !s.initialized {
        return;
    }
    s.frame += 1;
    s.total_time += 1.0;
    s.state_timer += 1.0;

    // ── Slow global scene rotation ──
    s.scene_rot_y += 0.006; // ~360° per 17s
    s.scene_rot_x = libm::sinf(s.total_time * 0.0015) * 0.35; // gentle nod

    // ── Choreography: switch formation when timer expires ──
    if s.state_timer >= HOLD_FRAMES {
        let old_number = s.num_drones;
        s.formation_idx = (s.formation_idx + 1) % NUMBER_FORMATIONS as u8;
        build_formation(s, s.formation_idx);
        // New drones beyond old count: start from center so they "fly out"
        for i in old_number..s.num_drones {
            s.drone_x[i] = 0.0;
            s.drone_y[i] = 0.0;
            s.drone_z[i] = 0.0;
        }
        s.state_timer = 0.0;
    }

    // ── Lerp all drones toward targets (exponential ease) ──
    for i in 0..MAXIMUM_DRONES {
        s.drone_x[i] += (s.target_x[i] - s.drone_x[i]) * LERP_SPEED;
        s.drone_y[i] += (s.target_y[i] - s.drone_y[i]) * LERP_SPEED;
        s.drone_z[i] += (s.target_z[i] - s.drone_z[i]) * LERP_SPEED;
    }

    // ── Project 3D → 2D ──
    project_drones(s);

    // ── Render glow buffer ──
    render_glow(s);
}

// ═══════════════════════════════════════
// Projection
// ═══════════════════════════════════════

fn project_drones(s: &mut DroneSwarmState) {
    let cos_y = libm::cosf(s.scene_rot_y);
    let sin_y = libm::sinf(s.scene_rot_y);
    let cos_x = libm::cosf(s.scene_rot_x);
    let sin_x = libm::sinf(s.scene_rot_x);

    let cx = s.screen_w * 0.5;
    let cy = s.screen_h * 0.45;
    let scale = s.screen_w * 0.28; // projection scale
    let cam_dist = 3.5f32;

    for i in 0..MAXIMUM_DRONES {
        let x = s.drone_x[i];
        let y = s.drone_y[i];
        let z = s.drone_z[i];

        // Rotate around Y axis
        let x2 = x * cos_y + z * sin_y;
        let z2 = -x * sin_y + z * cos_y;

        // Rotate around X axis
        let y2 = y * cos_x - z2 * sin_x;
        let z3 = y * sin_x + z2 * cos_x;

        // Perspective projection
        let w = (cam_dist + z3).max(0.15);

        s.proj_x[i] = cx + (x2 / w) * scale;
        s.proj_y[i] = cy + (y2 / w) * scale;
        s.proj_depth[i] = w;
    }
}

// ═══════════════════════════════════════
// Glow Buffer Rendering
// ═══════════════════════════════════════

/// Add brightness to a glow cell and its neighbors with smooth falloff.
#[inline]
fn add_glow(glow: &mut [f32; GLOW_SIZE], x: f32, y: f32, bright: f32, radius: i32) {
    let ix = x as i32;
    let iy = y as i32;
    let r_maximum = radius as f32 + 0.5;

    let mut dy = -radius;
    while dy <= radius {
        let mut dx = -radius;
        while dx <= radius {
            let cx = ix + dx;
            let cy = iy + dy;
            if cx >= 0 && (cx as usize) < GLOW_W && cy >= 0 && (cy as usize) < GLOW_H {
                let fx = x - cx as f32 - 0.5;
                let fy = y - cy as f32 - 0.5;
                let dist = libm::sqrtf(fx * fx + fy * fy);
                if dist < r_maximum {
                    let falloff = 1.0 - dist / r_maximum;
                    let falloff = falloff * falloff; // quadratic for soft neon glow
                    let idx = cy as usize * GLOW_W + cx as usize;
                    glow[idx] += bright * falloff;
                    if glow[idx] > 5.0 {
                        glow[idx] = 5.0;
                    }
                }
            }
            dx += 1;
        }
        dy += 1;
    }
}

fn render_glow(s: &mut DroneSwarmState) {
    // Clear glow buffer
    for i in 0..GLOW_SIZE {
        s.glow[i] = 0.0;
    }

    let cw = s.cell_w;
    let ch = s.cell_h;

    // ── Render edges as thick glowing lines ──
    for e in 0..s.num_edges {
        let a = s.edge_a[e] as usize;
        let b = s.edge_b[e] as usize;

        let x0 = s.proj_x[a] / cw;
        let y0 = s.proj_y[a] / ch;
        let x1 = s.proj_x[b] / cw;
        let y1 = s.proj_y[b] / ch;

        // Average depth → brightness (nearer = brighter)
        let average_depth = (s.proj_depth[a] + s.proj_depth[b]) * 0.5;
        let depth_bright = (1.8 / average_depth).min(1.5);
        let edge_bright = depth_bright * 0.7;

        // DDA line rasterization into glow buffer
        let dx = x1 - x0;
        let dy = y1 - y0;
        let len = libm::sqrtf(dx * dx + dy * dy);
        let steps = ((len * 1.5) as usize).max(1).min(300);

        for step in 0..=steps {
            let t = step as f32 / steps as f32;
            let x = x0 + dx * t;
            let y = y0 + dy * t;
            add_glow(&mut s.glow, x, y, edge_bright, 1);
        }
    }

    // ── Render vertices as bright point lights ──
    for i in 0..s.num_drones {
        let gx = s.proj_x[i] / cw;
        let gy = s.proj_y[i] / ch;
        let depth_bright = (2.5 / s.proj_depth[i]).min(3.0);
        add_glow(&mut s.glow, gx, gy, depth_bright, 2);
    }
}

// ═══════════════════════════════════════
// Rain Query
// ═══════════════════════════════════════

/// Result of querying the drone swarm at a pixel position.
pub struct DroneInteraction {
    pub brightness: f32,
    pub color_r: i16,
    pub color_g: i16,
    pub color_b: i16,
}

// Implementation block — defines methods for the type above.
impl DroneInteraction {
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const NONE: Self = Self {
        brightness: 1.0,
        color_r: 0,
        color_g: 0,
        color_b: 0,
    };
}

/// Query drone swarm interaction at pixel position (px, py).
/// O(1) — just samples the pre-computed glow buffer.
pub fn query(s: &DroneSwarmState, px: f32, py: f32) -> DroneInteraction {
    if !s.initialized {
        return DroneInteraction::NONE;
    }

    let gx = (px / s.cell_w) as usize;
    let gy = (py / s.cell_h) as usize;
    if gx >= GLOW_W || gy >= GLOW_H {
        return DroneInteraction::NONE;
    }

    let val = s.glow[gy * GLOW_W + gx];
    if val < 0.03 {
        return DroneInteraction::NONE;
    }

    // Choreography color based on formation + time
    let (cr, cg, cb) = choreo_color(s.total_time, s.formation_idx);

    // Brightness from glow intensity
    let brightness = if val > 2.0 {
        // Vertex zone: strong holographic glow
        1.4 + (val - 2.0).min(2.0) * 0.4
    } else if val > 0.5 {
        // Edge zone: moderate neon glow
        1.1 + (val - 0.5) * 0.25
    } else {
        // Halo zone: subtle ambient
        1.0 + val * 0.2
    };

    // Color contribution scales with glow value
    let color_t = (val / 2.5).min(1.0);

    DroneInteraction {
        brightness,
        color_r: (cr as f32 * color_t) as i16,
        color_g: (cg as f32 * color_t) as i16,
        color_b: (cb as f32 * color_t) as i16,
    }
}

// ═══════════════════════════════════════
// Choreography Color Palette
// ═══════════════════════════════════════

/// Compute holographic color based on elapsed time and current formation.
///
/// Each formation has a characteristic hue; during transitions the colors blend.
/// The overall palette cycles through: cyan → electric blue → gold → purple → ice → pink.
fn choreo_color(time: f32, formation: u8) -> (i16, i16, i16) {
    // Base color tint per formation (additive RGB shifts)
    let (br, bg, bb) = // Pattern matching — Rust's exhaustive branching construct.
match formation % NUMBER_FORMATIONS as u8 {
        0 => (0i16, 60, 80),      // DNA Helix: teal/cyan
        1 => (20i16, 30, 90),      // Tesseract: electric blue
        2 => (70i16, 50, -10),     // Atom: warm gold
        3 => (50i16, -5, 70),      // Galaxy: purple
        4 => (-10i16, 40, 70),     // Crystal: ice blue
        _ => (60i16, 20, 50),      // Infinity: magenta/pink
    };

    // Slow holographic shimmer — color oscillates gently
    let shimmer = libm::sinf(time * 0.018) * 0.2 + 1.0;
    // Secondary shimmer at different frequency for depth feel
    let shimmer2 = libm::sinf(time * 0.011 + 1.5) * 0.1 + 1.0;

    let r = (br as f32 * shimmer) as i16;
    let g = (bg as f32 * shimmer2) as i16;
    let b = (bb as f32 * shimmer) as i16;

    (r, g, b)
}
