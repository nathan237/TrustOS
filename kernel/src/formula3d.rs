// ═══════════════════════════════════════════════════════════════════════════
// FORMULA 3D — Tsoding-inspired wireframe 3D renderer
// Core idea: project({x,y,z}) = {x/z, y/z} — one formula for perspective
// Combined with Bresenham lines, depth coloring, matrix rain, glow vertices
// ═══════════════════════════════════════════════════════════════════════════

// ─── Math types ──────────────────────────────────────────────────────────

#[derive(Clone, Copy)]
pub struct V3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Clone, Copy)]
pub struct V2 {
    pub x: f32,
    pub y: f32,
}

// ─── Fast trig (Taylor series, no libm) ──────────────────────────────────

/// Normalize angle to [-PI, PI]
#[inline(always)]
fn wrap_angle(mut a: f32) -> f32 {
    const PI: f32 = 3.14159265;
    const TAU: f32 = 6.2831853;
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

/// sqrt via Newton's method (3 iterations)
#[inline(always)]
pub fn fast_sqrt(x: f32) -> f32 {
    if x <= 0.0 { return 0.0; }
    let mut g = x * 0.5;
    g = 0.5 * (g + x / g);
    g = 0.5 * (g + x / g);
    0.5 * (g + x / g)
}

/// Fast exp(x) approximation for x in [-4, 0] range (good for fog/falloff)
/// Uses Padé-like: e^x ≈ (1 + x/4)^4 clamped
#[inline(always)]
pub fn fast_exp(x: f32) -> f32 {
    if x < -6.0 { return 0.0; }
    if x > 0.0 { return 1.0; }
    // (1 + x/n)^n with n=8 for decent accuracy
    let t = 1.0 + x * 0.125; // x/8
    let t = if t < 0.0 { 0.0 } else { t };
    let t = t * t; // ^2
    let t = t * t; // ^4
    t * t           // ^8
}

// ─── 3D transforms ──────────────────────────────────────────────────────

#[inline(always)]
fn rotate_y(v: V3, a: f32) -> V3 {
    let c = fast_cos(a);
    let s = fast_sin(a);
    V3 { x: v.x * c + v.z * s, y: v.y, z: -v.x * s + v.z * c }
}

#[inline(always)]
fn rotate_x(v: V3, a: f32) -> V3 {
    let c = fast_cos(a);
    let s = fast_sin(a);
    V3 { x: v.x, y: v.y * c - v.z * s, z: v.y * s + v.z * c }
}

#[inline(always)]
fn translate_z(v: V3, dz: f32) -> V3 {
    V3 { x: v.x, y: v.y, z: v.z + dz }
}

/// THE FORMULA: project({x,y,z}) = {x/z, y/z}
#[inline(always)]
fn project(v: V3) -> V2 {
    if v.z.abs() < 0.001 {
        V2 { x: 0.0, y: 0.0 }
    } else {
        V2 { x: v.x / v.z, y: v.y / v.z }
    }
}

/// Map normalized coords to screen pixels
#[inline(always)]
fn to_screen(p: V2, w: usize, h: usize) -> (i32, i32) {
    let scale = w.min(h) as f32 * 0.45;
    let sx = (p.x * scale) + (w as f32 * 0.5);
    let sy = (-p.y * scale) + (h as f32 * 0.5);
    (sx as i32, sy as i32)
}

/// Full transform pipeline: rotate → translate → project → screen
#[inline(always)]
fn transform_vertex(v: V3, angle_y: f32, angle_x: f32, dz: f32, w: usize, h: usize) -> (i32, i32, f32) {
    let rotated = rotate_x(rotate_y(v, angle_y), angle_x);
    let translated = translate_z(rotated, dz);
    let screen = to_screen(project(translated), w, h);
    (screen.0, screen.1, translated.z)
}

// ─── Mesh ────────────────────────────────────────────────────────────────

pub struct Mesh {
    pub vertices: alloc::vec::Vec<V3>,
    pub edges: alloc::vec::Vec<(usize, usize)>,
}

pub fn mesh_cube() -> Mesh {
    let v = alloc::vec![
        V3 { x: -0.5, y: -0.5, z: -0.5 }, V3 { x:  0.5, y: -0.5, z: -0.5 },
        V3 { x:  0.5, y:  0.5, z: -0.5 }, V3 { x: -0.5, y:  0.5, z: -0.5 },
        V3 { x: -0.5, y: -0.5, z:  0.5 }, V3 { x:  0.5, y: -0.5, z:  0.5 },
        V3 { x:  0.5, y:  0.5, z:  0.5 }, V3 { x: -0.5, y:  0.5, z:  0.5 },
    ];
    let e = alloc::vec![
        (0,1),(1,2),(2,3),(3,0), // front
        (4,5),(5,6),(6,7),(7,4), // back
        (0,4),(1,5),(2,6),(3,7), // sides
    ];
    Mesh { vertices: v, edges: e }
}

pub fn mesh_pyramid() -> Mesh {
    let v = alloc::vec![
        V3 { x: -0.5, y: -0.5, z: -0.5 }, V3 { x:  0.5, y: -0.5, z: -0.5 },
        V3 { x:  0.5, y: -0.5, z:  0.5 }, V3 { x: -0.5, y: -0.5, z:  0.5 },
        V3 { x:  0.0, y:  0.7, z:  0.0 },
    ];
    let e = alloc::vec![
        (0,1),(1,2),(2,3),(3,0), // base
        (0,4),(1,4),(2,4),(3,4), // sides to apex
    ];
    Mesh { vertices: v, edges: e }
}

pub fn mesh_diamond() -> Mesh {
    let v = alloc::vec![
        V3 { x:  0.0, y:  0.7, z:  0.0 }, // top
        V3 { x: -0.5, y:  0.0, z: -0.5 }, V3 { x:  0.5, y:  0.0, z: -0.5 },
        V3 { x:  0.5, y:  0.0, z:  0.5 }, V3 { x: -0.5, y:  0.0, z:  0.5 },
        V3 { x:  0.0, y: -0.7, z:  0.0 }, // bottom
    ];
    let e = alloc::vec![
        (1,2),(2,3),(3,4),(4,1), // equator
        (0,1),(0,2),(0,3),(0,4), // top edges
        (5,1),(5,2),(5,3),(5,4), // bottom edges
    ];
    Mesh { vertices: v, edges: e }
}

pub fn mesh_torus(major_r: f32, minor_r: f32, major_seg: usize, minor_seg: usize) -> Mesh {
    let mut verts = alloc::vec::Vec::with_capacity(major_seg * minor_seg);
    let mut edges = alloc::vec::Vec::new();
    
    for i in 0..major_seg {
        let theta = (i as f32 / major_seg as f32) * 6.2831853;
        let ct = fast_cos(theta);
        let st = fast_sin(theta);
        for j in 0..minor_seg {
            let phi = (j as f32 / minor_seg as f32) * 6.2831853;
            let cp = fast_cos(phi);
            let sp = fast_sin(phi);
            let x = (major_r + minor_r * cp) * ct;
            let z = (major_r + minor_r * cp) * st;
            let y = minor_r * sp;
            verts.push(V3 { x, y, z });
            
            let idx = i * minor_seg + j;
            let next_j = i * minor_seg + (j + 1) % minor_seg;
            edges.push((idx, next_j));
            let next_i = ((i + 1) % major_seg) * minor_seg + j;
            edges.push((idx, next_i));
        }
    }
    Mesh { vertices: verts, edges }
}

pub fn mesh_icosphere(radius: f32) -> Mesh {
    let t = (1.0 + fast_sqrt(5.0)) / 2.0;
    let s = radius / fast_sqrt(1.0 + t * t);
    let a = s;
    let b = s * t;
    
    let v = alloc::vec![
        V3 { x: -a, y:  b, z: 0.0 }, V3 { x:  a, y:  b, z: 0.0 },
        V3 { x: -a, y: -b, z: 0.0 }, V3 { x:  a, y: -b, z: 0.0 },
        V3 { x: 0.0, y: -a, z:  b }, V3 { x: 0.0, y:  a, z:  b },
        V3 { x: 0.0, y: -a, z: -b }, V3 { x: 0.0, y:  a, z: -b },
        V3 { x:  b, y: 0.0, z: -a }, V3 { x:  b, y: 0.0, z:  a },
        V3 { x: -b, y: 0.0, z: -a }, V3 { x: -b, y: 0.0, z:  a },
    ];
    let e = alloc::vec![
        (0,1),(0,5),(0,7),(0,10),(0,11),
        (1,5),(1,7),(1,8),(1,9),
        (2,3),(2,4),(2,6),(2,10),(2,11),
        (3,4),(3,6),(3,8),(3,9),
        (4,5),(4,9),(4,11),
        (5,9),(5,11),
        (6,7),(6,8),(6,10),
        (7,8),(7,10),
        (8,9),
        (10,11),
    ];
    Mesh { vertices: v, edges: e }
}

pub fn mesh_grid(half: f32, divisions: usize) -> Mesh {
    let mut verts = alloc::vec::Vec::new();
    let mut edges = alloc::vec::Vec::new();
    let step = (half * 2.0) / divisions as f32;
    let n = divisions + 1;
    
    for i in 0..n {
        for j in 0..n {
            let x = -half + i as f32 * step;
            let z = -half + j as f32 * step;
            verts.push(V3 { x, y: -0.3, z });
            let idx = i * n + j;
            if j + 1 < n { edges.push((idx, idx + 1)); }
            if i + 1 < n { edges.push((idx, idx + n)); }
        }
    }
    Mesh { vertices: verts, edges }
}

/// Simplified Penger (blocky penguin meme character) — hand-crafted low-poly wireframe
/// Body parts: torso, head, belly patch, eyes, beak, feet, flippers
pub fn mesh_penger() -> Mesh {
    let mut verts = alloc::vec::Vec::with_capacity(80);
    let mut edges = alloc::vec::Vec::with_capacity(120);

    // ── BODY (torso) — slightly tapered rounded box ──
    // Bottom ring (y = -0.40), wider
    let body_bot = verts.len(); // 0..7
    let bw = 0.28; let bd = 0.22; let by = -0.40;
    verts.push(V3 { x: -bw, y: by, z: -bd }); // 0
    verts.push(V3 { x:  bw, y: by, z: -bd }); // 1
    verts.push(V3 { x:  bw, y: by, z:  bd }); // 2
    verts.push(V3 { x: -bw, y: by, z:  bd }); // 3
    // Mid ring (y = 0.0), same width
    let body_mid = verts.len(); // 4..7
    verts.push(V3 { x: -bw, y: 0.0, z: -bd }); // 4
    verts.push(V3 { x:  bw, y: 0.0, z: -bd }); // 5
    verts.push(V3 { x:  bw, y: 0.0, z:  bd }); // 6
    verts.push(V3 { x: -bw, y: 0.0, z:  bd }); // 7
    // Top ring (y = 0.30), slightly narrower (shoulder)
    let body_top = verts.len(); // 8..11
    let tw = 0.24; let td = 0.20;
    verts.push(V3 { x: -tw, y: 0.30, z: -td }); // 8
    verts.push(V3 { x:  tw, y: 0.30, z: -td }); // 9
    verts.push(V3 { x:  tw, y: 0.30, z:  td }); // 10
    verts.push(V3 { x: -tw, y: 0.30, z:  td }); // 11

    // Body edges — bottom ring
    edges.push((body_bot, body_bot+1)); edges.push((body_bot+1, body_bot+2));
    edges.push((body_bot+2, body_bot+3)); edges.push((body_bot+3, body_bot));
    // Body edges — mid ring
    edges.push((body_mid, body_mid+1)); edges.push((body_mid+1, body_mid+2));
    edges.push((body_mid+2, body_mid+3)); edges.push((body_mid+3, body_mid));
    // Body edges — top ring
    edges.push((body_top, body_top+1)); edges.push((body_top+1, body_top+2));
    edges.push((body_top+2, body_top+3)); edges.push((body_top+3, body_top));
    // Body vertical edges
    for i in 0..4 { edges.push((body_bot + i, body_mid + i)); }
    for i in 0..4 { edges.push((body_mid + i, body_top + i)); }

    // ── HEAD — wider box on top ──
    let head_bot = verts.len(); // 12..15
    let hw = 0.26; let hd = 0.22; let hby = 0.32;
    verts.push(V3 { x: -hw, y: hby, z: -hd }); // 12
    verts.push(V3 { x:  hw, y: hby, z: -hd }); // 13
    verts.push(V3 { x:  hw, y: hby, z:  hd }); // 14
    verts.push(V3 { x: -hw, y: hby, z:  hd }); // 15
    // Head top ring (y = 0.65)
    let head_top = verts.len(); // 16..19
    let htw = 0.24; let htd = 0.20;
    verts.push(V3 { x: -htw, y: 0.65, z: -htd }); // 16
    verts.push(V3 { x:  htw, y: 0.65, z: -htd }); // 17
    verts.push(V3 { x:  htw, y: 0.65, z:  htd }); // 18
    verts.push(V3 { x: -htw, y: 0.65, z:  htd }); // 19

    // Head edges
    edges.push((head_bot, head_bot+1)); edges.push((head_bot+1, head_bot+2));
    edges.push((head_bot+2, head_bot+3)); edges.push((head_bot+3, head_bot));
    edges.push((head_top, head_top+1)); edges.push((head_top+1, head_top+2));
    edges.push((head_top+2, head_top+3)); edges.push((head_top+3, head_top));
    for i in 0..4 { edges.push((head_bot + i, head_top + i)); }

    // ── BELLY PATCH — oval outline on the FRONT face ──
    // 8 vertices forming an ellipse on z = -bd (front)
    let belly_start = verts.len(); // 20..27
    let belly_cx = 0.0; let belly_cy = -0.05; let belly_z = -bd - 0.01;
    let belly_rx = 0.16; let belly_ry = 0.28;
    for i in 0..8u32 {
        let angle = i as f32 * 0.7853982; // PI/4 * i
        let x = belly_cx + belly_rx * fast_cos(angle);
        let y = belly_cy + belly_ry * fast_sin(angle);
        verts.push(V3 { x, y, z: belly_z });
    }
    for i in 0..8 { edges.push((belly_start + i, belly_start + (i + 1) % 8)); }

    // ── EYES — two small squares on front of head ──
    let eye_y = 0.50; let eye_z = -hd - 0.01; let eye_size = 0.04;
    // Left eye
    let el = verts.len(); // 28..31
    verts.push(V3 { x: -0.10 - eye_size, y: eye_y - eye_size, z: eye_z });
    verts.push(V3 { x: -0.10 + eye_size, y: eye_y - eye_size, z: eye_z });
    verts.push(V3 { x: -0.10 + eye_size, y: eye_y + eye_size, z: eye_z });
    verts.push(V3 { x: -0.10 - eye_size, y: eye_y + eye_size, z: eye_z });
    edges.push((el, el+1)); edges.push((el+1, el+2)); edges.push((el+2, el+3)); edges.push((el+3, el));
    // Right eye
    let er = verts.len(); // 32..35
    verts.push(V3 { x: 0.10 - eye_size, y: eye_y - eye_size, z: eye_z });
    verts.push(V3 { x: 0.10 + eye_size, y: eye_y - eye_size, z: eye_z });
    verts.push(V3 { x: 0.10 + eye_size, y: eye_y + eye_size, z: eye_z });
    verts.push(V3 { x: 0.10 - eye_size, y: eye_y + eye_size, z: eye_z });
    edges.push((er, er+1)); edges.push((er+1, er+2)); edges.push((er+2, er+3)); edges.push((er+3, er));
    // Eye cross-lines (X marks for cute look)
    edges.push((el, el+2)); edges.push((el+1, el+3));
    edges.push((er, er+2)); edges.push((er+1, er+3));

    // ── BEAK — small diamond/triangle on front ──
    let beak_y = 0.42; let beak_z = -hd - 0.03;
    let bk = verts.len(); // 36..39
    verts.push(V3 { x:  0.00, y: beak_y + 0.03, z: beak_z }); // top
    verts.push(V3 { x: -0.04, y: beak_y,        z: beak_z }); // left
    verts.push(V3 { x:  0.04, y: beak_y,        z: beak_z }); // right
    verts.push(V3 { x:  0.00, y: beak_y - 0.02, z: beak_z - 0.02 }); // tip (forward)
    edges.push((bk, bk+1)); edges.push((bk, bk+2)); edges.push((bk+1, bk+2));
    edges.push((bk+1, bk+3)); edges.push((bk+2, bk+3)); edges.push((bk, bk+3));

    // ── FEET — two flat boxes at the bottom ──
    let foot_y = -0.42; let foot_h = 0.04; let foot_w = 0.10; let foot_d = 0.14;
    // Left foot
    let fl = verts.len(); // 40..47
    verts.push(V3 { x: -0.16 - foot_w, y: foot_y,          z: -foot_d });
    verts.push(V3 { x: -0.16 + foot_w, y: foot_y,          z: -foot_d });
    verts.push(V3 { x: -0.16 + foot_w, y: foot_y,          z:  foot_d });
    verts.push(V3 { x: -0.16 - foot_w, y: foot_y,          z:  foot_d });
    verts.push(V3 { x: -0.16 - foot_w, y: foot_y - foot_h, z: -foot_d });
    verts.push(V3 { x: -0.16 + foot_w, y: foot_y - foot_h, z: -foot_d });
    verts.push(V3 { x: -0.16 + foot_w, y: foot_y - foot_h, z:  foot_d });
    verts.push(V3 { x: -0.16 - foot_w, y: foot_y - foot_h, z:  foot_d });
    edges.push((fl, fl+1)); edges.push((fl+1, fl+2)); edges.push((fl+2, fl+3)); edges.push((fl+3, fl));
    edges.push((fl+4, fl+5)); edges.push((fl+5, fl+6)); edges.push((fl+6, fl+7)); edges.push((fl+7, fl+4));
    for i in 0..4 { edges.push((fl + i, fl + 4 + i)); }
    // Right foot
    let fr = verts.len(); // 48..55
    verts.push(V3 { x: 0.16 - foot_w, y: foot_y,          z: -foot_d });
    verts.push(V3 { x: 0.16 + foot_w, y: foot_y,          z: -foot_d });
    verts.push(V3 { x: 0.16 + foot_w, y: foot_y,          z:  foot_d });
    verts.push(V3 { x: 0.16 - foot_w, y: foot_y,          z:  foot_d });
    verts.push(V3 { x: 0.16 - foot_w, y: foot_y - foot_h, z: -foot_d });
    verts.push(V3 { x: 0.16 + foot_w, y: foot_y - foot_h, z: -foot_d });
    verts.push(V3 { x: 0.16 + foot_w, y: foot_y - foot_h, z:  foot_d });
    verts.push(V3 { x: 0.16 - foot_w, y: foot_y - foot_h, z:  foot_d });
    edges.push((fr, fr+1)); edges.push((fr+1, fr+2)); edges.push((fr+2, fr+3)); edges.push((fr+3, fr));
    edges.push((fr+4, fr+5)); edges.push((fr+5, fr+6)); edges.push((fr+6, fr+7)); edges.push((fr+7, fr+4));
    for i in 0..4 { edges.push((fr + i, fr + 4 + i)); }

    // ── FLIPPERS/WINGS — triangular fins on each side ──
    // Left flipper
    let wl = verts.len(); // 56..59
    verts.push(V3 { x: -bw - 0.01, y:  0.20, z:  0.00 }); // top attach
    verts.push(V3 { x: -bw - 0.14, y: -0.05, z: -0.04 }); // outer mid
    verts.push(V3 { x: -bw - 0.10, y: -0.25, z: -0.02 }); // outer bottom
    verts.push(V3 { x: -bw - 0.01, y: -0.15, z:  0.00 }); // bottom attach
    edges.push((wl, wl+1)); edges.push((wl+1, wl+2)); edges.push((wl+2, wl+3)); edges.push((wl+3, wl));
    // Right flipper
    let wr = verts.len(); // 60..63
    verts.push(V3 { x: bw + 0.01, y:  0.20, z:  0.00 });
    verts.push(V3 { x: bw + 0.14, y: -0.05, z: -0.04 });
    verts.push(V3 { x: bw + 0.10, y: -0.25, z: -0.02 });
    verts.push(V3 { x: bw + 0.01, y: -0.15, z:  0.00 });
    edges.push((wr, wr+1)); edges.push((wr+1, wr+2)); edges.push((wr+2, wr+3)); edges.push((wr+3, wr));

    // ── HEAD CROWN — rounded top using 8-point arc ──
    let crown = verts.len(); // 64..71
    let crown_y = 0.65; let crown_r = 0.22;
    for i in 0..8u32 {
        let angle = i as f32 * 0.7853982; // PI/4
        let x = crown_r * fast_cos(angle);
        let z = crown_r * 0.9 * fast_sin(angle);
        let y_bump = 0.08 * fast_cos(angle * 2.0); // dome shape
        verts.push(V3 { x, y: crown_y + y_bump, z });
    }
    for i in 0..8 { edges.push((crown + i, crown + (i + 1) % 8)); }
    // Connect crown to head top corners
    edges.push((crown + 0, head_top + 1)); // front-right area
    edges.push((crown + 2, head_top + 2)); // back-right
    edges.push((crown + 4, head_top + 3)); // back-left area
    edges.push((crown + 6, head_top));     // front-left

    Mesh { vertices: verts, edges }
}

/// 3D block text "TRUSTOS" — wireframe letters with depth extrusion
/// Each letter is defined as polylines, then extruded front/back in Z for 3D depth
pub fn mesh_trustos_text() -> Mesh {
    let mut verts = alloc::vec::Vec::with_capacity(220);
    let mut edges = alloc::vec::Vec::with_capacity(350);

    let lw: f32 = 0.18;     // letter width
    let depth: f32 = 0.06;  // z extrusion half-depth
    let pitch: f32 = 0.25;  // distance between letter starts
    let start_x: f32 = -pitch * 3.0; // center 7 letters
    let top: f32 = 0.20;    // letter top
    let bot: f32 = -0.20;   // letter bottom
    let mid: f32 = 0.0;     // letter middle

    // Helper: add an extruded polyline (front z=-depth, back z=+depth)
    fn add_polyline(verts: &mut alloc::vec::Vec<V3>, edges: &mut alloc::vec::Vec<(usize, usize)>,
                    points: &[(f32, f32)], ox: f32, depth: f32) {
        let base = verts.len();
        for &(px, py) in points {
            verts.push(V3 { x: ox + px, y: py, z: -depth }); // front
            verts.push(V3 { x: ox + px, y: py, z: depth });  // back
        }
        for i in 0..points.len() {
            // Front-to-back edge at each vertex
            edges.push((base + i * 2, base + i * 2 + 1));
            // Sequential front edge
            if i + 1 < points.len() {
                edges.push((base + i * 2, base + (i + 1) * 2));
            }
            // Sequential back edge
            if i + 1 < points.len() {
                edges.push((base + i * 2 + 1, base + (i + 1) * 2 + 1));
            }
        }
    }

    // Letter T (index 0)
    let ox = start_x;
    add_polyline(&mut verts, &mut edges, &[(0.0, top), (lw, top)], ox, depth);
    add_polyline(&mut verts, &mut edges, &[(lw * 0.5, top), (lw * 0.5, bot)], ox, depth);

    // Letter R (index 1)
    let ox = start_x + pitch;
    add_polyline(&mut verts, &mut edges,
        &[(0.0, bot), (0.0, top), (lw, top), (lw, mid), (0.0, mid)], ox, depth);
    add_polyline(&mut verts, &mut edges,
        &[(0.04, mid), (lw, bot)], ox, depth); // diagonal leg

    // Letter U (index 2)
    let ox = start_x + pitch * 2.0;
    add_polyline(&mut verts, &mut edges,
        &[(0.0, top), (0.0, bot), (lw, bot), (lw, top)], ox, depth);

    // Letter S (index 3)
    let ox = start_x + pitch * 3.0;
    add_polyline(&mut verts, &mut edges,
        &[(lw, top), (0.0, top), (0.0, mid), (lw, mid), (lw, bot), (0.0, bot)], ox, depth);

    // Letter T (index 4)
    let ox = start_x + pitch * 4.0;
    add_polyline(&mut verts, &mut edges, &[(0.0, top), (lw, top)], ox, depth);
    add_polyline(&mut verts, &mut edges, &[(lw * 0.5, top), (lw * 0.5, bot)], ox, depth);

    // Letter O (index 5)
    let ox = start_x + pitch * 5.0;
    add_polyline(&mut verts, &mut edges,
        &[(0.0, top), (lw, top), (lw, bot), (0.0, bot), (0.0, top)], ox, depth);

    // Letter S (index 6)
    let ox = start_x + pitch * 6.0;
    add_polyline(&mut verts, &mut edges,
        &[(lw, top), (0.0, top), (0.0, mid), (lw, mid), (lw, bot), (0.0, bot)], ox, depth);

    Mesh { vertices: verts, edges }
}

pub fn mesh_helix(radius: f32, height: f32, turns: f32, seg: usize) -> Mesh {
    let mut verts = alloc::vec::Vec::new();
    let mut edges = alloc::vec::Vec::new();
    let total_angle = turns * 6.2831853;
    
    for strand in 0..2u32 {
        let offset = strand as f32 * 3.14159265;
        let base = verts.len();
        for i in 0..seg {
            let t = i as f32 / (seg - 1) as f32;
            let angle = t * total_angle + offset;
            let x = radius * fast_cos(angle);
            let z = radius * fast_sin(angle);
            let y = -height * 0.5 + t * height;
            verts.push(V3 { x, y, z });
            if i > 0 { edges.push((base + i - 1, base + i)); }
        }
    }
    // Cross-rungs
    let half = seg;
    for i in (0..seg).step_by(seg / 8) {
        edges.push((i, i + half));
    }
    Mesh { vertices: verts, edges }
}

// ─── Drawing ─────────────────────────────────────────────────────────────

/// Additive blend a pixel at (x,y)
#[inline(always)]
fn additive_blend(buf: &mut [u32], w: usize, h: usize, x: i32, y: i32, color: u32) {
    if x < 0 || y < 0 || x >= w as i32 || y >= h as i32 { return; }
    let idx = y as usize * w + x as usize;
    if idx >= buf.len() { return; }
    let dst = buf[idx];
    let r = ((dst >> 16) & 0xFF) + ((color >> 16) & 0xFF);
    let g = ((dst >> 8) & 0xFF) + ((color >> 8) & 0xFF);
    let b = (dst & 0xFF) + (color & 0xFF);
    buf[idx] = 0xFF000000 | (r.min(255) << 16) | (g.min(255) << 8) | b.min(255);
}

/// Bresenham line drawing
fn draw_line(buf: &mut [u32], w: usize, h: usize, x0: i32, y0: i32, x1: i32, y1: i32, color: u32) {
    let mut x0 = x0; let mut y0 = y0;
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx: i32 = if x0 < x1 { 1 } else { -1 };
    let sy: i32 = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    
    loop {
        additive_blend(buf, w, h, x0, y0, color);
        if x0 == x1 && y0 == y1 { break; }
        let e2 = 2 * err;
        if e2 >= dy { err += dy; x0 += sx; }
        if e2 <= dx { err += dx; y0 += sy; }
    }
}

/// Thick line (3px) for better visibility
fn draw_line_thick(buf: &mut [u32], w: usize, h: usize, x0: i32, y0: i32, x1: i32, y1: i32, color: u32) {
    draw_line(buf, w, h, x0, y0, x1, y1, color);
    draw_line(buf, w, h, x0 + 1, y0, x1 + 1, y1, color);
    draw_line(buf, w, h, x0, y0 + 1, x1, y1 + 1, color);
}

/// Dim entire buffer (fade to black effect)
fn dim(buf: &mut [u32], factor: u8) {
    for px in buf.iter_mut() {
        let r = ((*px >> 16) & 0xFF) * factor as u32 / 256;
        let g = ((*px >> 8) & 0xFF) * factor as u32 / 256;
        let b = (*px & 0xFF) * factor as u32 / 256;
        *px = 0xFF000000 | (r << 16) | (g << 8) | b;
    }
}

/// Color based on depth (closer = brighter green, further = darker)
fn depth_color(z: f32, base_color: u32) -> u32 {
    let intensity = ((1.0 / (z * 0.5 + 1.0)) * 255.0) as u32;
    let intensity = intensity.min(255);
    let r = ((base_color >> 16) & 0xFF) * intensity / 255;
    let g = ((base_color >> 8) & 0xFF) * intensity / 255;
    let b = (base_color & 0xFF) * intensity / 255;
    0xFF000000 | (r << 16) | (g << 8) | b
}

// ─── Matrix Rain background ─────────────────────────────────────────────

struct RainColumn {
    y: f32,
    speed: f32,
    len: u32,
    glyph: u8,
}

/// 3D rain column for holographic matrix
struct HoloRainCol3D {
    x: f32,
    z: f32,
    speed: f32,
    trail_len: u8,
    phase: f32,
}

// ─── Scene types ─────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq)]
pub enum FormulaScene {
    Cube,
    Pyramid,
    Diamond,
    Torus,
    Icosphere,
    Grid,
    Helix,
    Multi,
    Penger,
    TrustOs,
    HoloMatrix,
}

// ─── Main renderer ───────────────────────────────────────────────────────

pub struct FormulaRenderer {
    pub scene: FormulaScene,
    pub angle_y: f32,
    pub angle_x: f32,
    pub dz: f32,
    pub wire_color: u32,
    frame: u32,
    rain_columns: alloc::vec::Vec<RainColumn>,
    rain_inited: bool,
    // HoloMatrix 3D rain state
    holo_rain_inited: bool,
    holo_columns: alloc::vec::Vec<HoloRainCol3D>,
    // Meshes (lazily built)
    mesh_cube: Option<Mesh>,
    mesh_pyramid: Option<Mesh>,
    mesh_diamond: Option<Mesh>,
    mesh_torus: Option<Mesh>,
    mesh_icosphere: Option<Mesh>,
    mesh_grid: Option<Mesh>,
    mesh_helix: Option<Mesh>,
    mesh_penger: Option<Mesh>,
    mesh_trustos: Option<Mesh>,
}

impl FormulaRenderer {
    pub fn new() -> Self {
        Self {
            scene: FormulaScene::HoloMatrix,
            angle_y: 0.0,
            angle_x: 0.3,
            dz: 2.0,
            wire_color: 0xFF00FF66,
            frame: 0,
            rain_columns: alloc::vec::Vec::new(),
            rain_inited: false,
            holo_rain_inited: false,
            holo_columns: alloc::vec::Vec::new(),
            mesh_cube: None,
            mesh_pyramid: None,
            mesh_diamond: None,
            mesh_torus: None,
            mesh_icosphere: None,
            mesh_grid: None,
            mesh_helix: None,
            mesh_penger: None,
            mesh_trustos: None,
        }
    }

    pub fn set_scene(&mut self, scene: FormulaScene) {
        self.scene = scene;
        // Pre-build mesh for new scene
        self.ensure_mesh();
    }

    fn ensure_mesh(&mut self) {
        match self.scene {
            FormulaScene::Cube => { if self.mesh_cube.is_none() { self.mesh_cube = Some(mesh_cube()); } }
            FormulaScene::Pyramid => { if self.mesh_pyramid.is_none() { self.mesh_pyramid = Some(mesh_pyramid()); } }
            FormulaScene::Diamond => { if self.mesh_diamond.is_none() { self.mesh_diamond = Some(mesh_diamond()); } }
            FormulaScene::Torus => { if self.mesh_torus.is_none() { self.mesh_torus = Some(mesh_torus(0.5, 0.2, 16, 12)); } }
            FormulaScene::Icosphere => { if self.mesh_icosphere.is_none() { self.mesh_icosphere = Some(mesh_icosphere(0.6)); } }
            FormulaScene::Grid => { if self.mesh_grid.is_none() { self.mesh_grid = Some(mesh_grid(1.5, 10)); } }
            FormulaScene::Helix => { if self.mesh_helix.is_none() { self.mesh_helix = Some(mesh_helix(0.4, 1.2, 3.0, 60)); } }
            FormulaScene::Penger => { if self.mesh_penger.is_none() { self.mesh_penger = Some(mesh_penger()); } }
            FormulaScene::TrustOs => { if self.mesh_trustos.is_none() { self.mesh_trustos = Some(mesh_trustos_text()); } }
            FormulaScene::HoloMatrix => { /* no mesh — procedural */ }
            FormulaScene::Multi => {
                if self.mesh_cube.is_none() { self.mesh_cube = Some(mesh_cube()); }
                if self.mesh_pyramid.is_none() { self.mesh_pyramid = Some(mesh_pyramid()); }
                if self.mesh_diamond.is_none() { self.mesh_diamond = Some(mesh_diamond()); }
                if self.mesh_torus.is_none() { self.mesh_torus = Some(mesh_torus(0.5, 0.2, 16, 12)); }
            }
        }
    }

    fn get_mesh(&self) -> Option<&Mesh> {
        match self.scene {
            FormulaScene::Cube => self.mesh_cube.as_ref(),
            FormulaScene::Pyramid => self.mesh_pyramid.as_ref(),
            FormulaScene::Diamond => self.mesh_diamond.as_ref(),
            FormulaScene::Torus => self.mesh_torus.as_ref(),
            FormulaScene::Icosphere => self.mesh_icosphere.as_ref(),
            FormulaScene::Grid => self.mesh_grid.as_ref(),
            FormulaScene::Helix => self.mesh_helix.as_ref(),
            FormulaScene::Penger => self.mesh_penger.as_ref(),
            FormulaScene::TrustOs => self.mesh_trustos.as_ref(),
            FormulaScene::HoloMatrix => None, // procedural
            FormulaScene::Multi => None, // Multi renders multiple meshes
        }
    }

    /// Update animation state — call every frame (even skip frames)
    pub fn update(&mut self) {
        self.frame = self.frame.wrapping_add(1);
        match self.scene {
            FormulaScene::TrustOs => {
                self.angle_y += 0.012;
                self.angle_x = 0.15 + fast_sin(self.frame as f32 * 0.006) * 0.1;
                self.dz = 3.2 + fast_sin(self.frame as f32 * 0.005) * 0.2;
            }
            FormulaScene::HoloMatrix => {
                // NO rotation — camera fixed, staring straight into the infinite tunnel
                // Only very subtle drift for "floating" feel (human vestibular illusion)
                self.angle_y = fast_sin(self.frame as f32 * 0.0008) * 0.015;
                self.angle_x = fast_sin(self.frame as f32 * 0.0006) * 0.01;
                self.dz = 0.0;
            }
            _ => {
                self.angle_y += 0.025;
                self.angle_x = 0.3 + fast_sin(self.frame as f32 * 0.008) * 0.2;
                self.dz = 2.0 + fast_sin(self.frame as f32 * 0.005) * 0.3;
            }
        }
    }

    /// Full render to buffer
    pub fn render(&self, buf: &mut [u32], w: usize, h: usize) {
        // Fast black fill via SSE2
        #[cfg(target_arch = "x86_64")]
        unsafe {
            use core::arch::x86_64::*;
            let black = _mm_set1_epi32(0xFF000000u32 as i32);
            let ptr = buf.as_mut_ptr() as *mut __m128i;
            let count = buf.len() / 4;
            for i in 0..count {
                _mm_storeu_si128(ptr.add(i), black);
            }
            // Handle remaining pixels
            for i in (count * 4)..buf.len() {
                buf[i] = 0xFF000000;
            }
        }
        #[cfg(not(target_arch = "x86_64"))]
        buf.fill(0xFF000000);

        if self.scene == FormulaScene::HoloMatrix {
            // Full 3D holographic matrix — no 2D rain, no scanlines
            self.render_holo_matrix(buf, w, h);
            return;
        }

        // Render rain background
        self.render_rain(buf, w, h);
        
        // Render wireframe(s)
        if self.scene == FormulaScene::Multi {
            self.render_multi(buf, w, h);
        } else if let Some(mesh) = self.get_mesh() {
            self.render_wireframe(buf, w, h, mesh, self.angle_y, self.angle_x, self.dz, self.wire_color);
            self.render_vertices(buf, w, h, mesh, self.angle_y, self.angle_x, self.dz);
        }

        // Hologram scanline effect
        if self.scene == FormulaScene::TrustOs {
            self.render_scanlines(buf, w, h);
        }
    }

    fn render_wireframe(&self, buf: &mut [u32], w: usize, h: usize, mesh: &Mesh,
                        ay: f32, ax: f32, dz: f32, color: u32) {
        for &(a, b) in &mesh.edges {
            if a >= mesh.vertices.len() || b >= mesh.vertices.len() { continue; }
            let (x0, y0, z0) = transform_vertex(mesh.vertices[a], ay, ax, dz, w, h);
            let (x1, y1, z1) = transform_vertex(mesh.vertices[b], ay, ax, dz, w, h);
            let avg_z = (z0 + z1) * 0.5;
            let c = depth_color(avg_z, color);
            draw_line_thick(buf, w, h, x0, y0, x1, y1, c);
        }
    }

    fn render_vertices(&self, buf: &mut [u32], w: usize, h: usize, mesh: &Mesh,
                       ay: f32, ax: f32, dz: f32) {
        for v in &mesh.vertices {
            let (sx, sy, _z) = transform_vertex(*v, ay, ax, dz, w, h);
            // Glow: 3×3 bright center
            for dy in -1..=1i32 {
                for dx in -1..=1i32 {
                    additive_blend(buf, w, h, sx + dx, sy + dy, 0x00FFFFFF);
                }
            }
        }
    }

    fn render_multi(&self, buf: &mut [u32], w: usize, h: usize) {
        let t = self.frame as f32 * 0.012;
        let orbit_r = 1.2;
        
        // 4 shapes orbiting
        let shapes: [(f32, u32, FormulaScene); 4] = [
            (0.0, 0xFF00FF66, FormulaScene::Cube),
            (1.5707963, 0xFF6666FF, FormulaScene::Pyramid),
            (3.14159265, 0xFFFF6600, FormulaScene::Diamond),
            (4.7123889, 0xFFFF00FF, FormulaScene::Torus),
        ];
        
        for (angle_offset, color, scene) in &shapes {
            let orbit_angle = t + angle_offset;
            let ox = orbit_r * fast_cos(orbit_angle);
            let oz = orbit_r * fast_sin(orbit_angle);
            
            let mesh = match scene {
                FormulaScene::Cube => self.mesh_cube.as_ref(),
                FormulaScene::Pyramid => self.mesh_pyramid.as_ref(),
                FormulaScene::Diamond => self.mesh_diamond.as_ref(),
                FormulaScene::Torus => self.mesh_torus.as_ref(),
                _ => None,
            };
            
            if let Some(mesh) = mesh {
                // Render with offset — scale down and orbit
                for &(a, b) in &mesh.edges {
                    if a >= mesh.vertices.len() || b >= mesh.vertices.len() { continue; }
                    let va = V3 { x: mesh.vertices[a].x * 0.35 + ox, y: mesh.vertices[a].y * 0.35, z: mesh.vertices[a].z * 0.35 + oz };
                    let vb = V3 { x: mesh.vertices[b].x * 0.35 + ox, y: mesh.vertices[b].y * 0.35, z: mesh.vertices[b].z * 0.35 + oz };
                    let (x0, y0, z0) = transform_vertex(va, self.angle_y * 0.5, self.angle_x, self.dz + 1.0, w, h);
                    let (x1, y1, z1) = transform_vertex(vb, self.angle_y * 0.5, self.angle_x, self.dz + 1.0, w, h);
                    let avg_z = (z0 + z1) * 0.5;
                    let c = depth_color(avg_z, *color);
                    draw_line_thick(buf, w, h, x0, y0, x1, y1, c);
                }
            }
        }
    }

    /// ══════════════════════════════════════════════════════════════════════
    /// INFINITE HOLOGRAPHIC MATRIX — Demoscene tunnel + starfield technique
    /// ══════════════════════════════════════════════════════════════════════
    /// Perception tricks used:
    ///  1. Radial expansion from vanishing point = optic flow → brain perceives forward motion
    ///  2. Columns in cylindrical shell, looping in Z = infinite tunnel illusion
    ///  3. Exponential size scaling (1/z²) = foreshortening → strong depth cue
    ///  4. Center glow (bright vanishing point) = Gestalt figure/ground separation
    ///  5. Vignette darkening at edges = foveal attention → tunnel vision
    ///  6. No rotation (camera drift only) = vestibular stability → "flying through"
    ///  7. Demoscene distance table trick: z wraps modulo → seamless infinite repeat
    ///  8. Trail brightness squared falloff = Weber-Fechner logarithmic perception
    fn render_holo_matrix(&self, buf: &mut [u32], w: usize, h: usize) {
        // ── Glyph bitmaps: 16 katakana-inspired 5×7 patterns stored as u64 bitmasks ──
        const GLYPHS: [u64; 16] = [
            0b11111_10001_10001_11111_00001_00001_00001, // ア (a)
            0b00100_00100_11111_00100_01010_10001_00000, // キ (ki)
            0b11111_00001_00010_00100_01000_10000_11111, // ク (ku)
            0b01110_00010_00010_00010_00010_00010_11111, // ケ (ke)
            0b11111_10000_10000_11110_00001_00001_11110, // コ (ko)
            0b00100_01110_10101_00100_00100_00100_00100, // サ (sa)
            0b01010_01010_11111_01010_01010_00100_00100, // シ (shi)
            0b11111_00001_11111_10000_10000_10000_11111, // ス (su)
            0b10001_10001_01010_00100_01010_10001_10001, // セ (se)
            0b11111_10001_10001_10001_10001_10001_11111, // ソ (so)
            0b00100_01110_10101_10101_01110_00100_00100, // タ (ta)
            0b10001_01010_00100_11111_00100_00100_00100, // チ (chi)
            0b11111_00100_00100_01110_10001_10001_01110, // ツ (tsu)
            0b10100_10100_10100_10110_10001_10001_01110, // テ (te)
            0b01110_10001_10001_01110_00100_01010_10001, // ト (to)
            0b00100_01010_10001_11111_10001_01010_00100, // ナ (na)
        ];

        // ══ TUNNEL GEOMETRY ══
        // Columns placed in a cylindrical "tunnel" around the Z axis.
        // Camera at origin, looking down +Z. Columns wrap in Z → infinite.
        const NUM_RINGS: usize = 10;        // radial distribution rings
        const COLS_PER_RING: usize = 18;    // columns per ring
        const NUM_COLS: usize = NUM_RINGS * COLS_PER_RING; // 180 total
        const Z_DEPTH: f32 = 16.0;         // total tunnel depth (deeper for density)
        const Z_NEAR: f32 = 0.5;           // near clip
        const TUBE_RADIUS: f32 = 5.5;      // cylinder radius

        let frame = self.frame;
        // Camera advances through the tunnel — this shifts ALL columns toward viewer
        let cam_z_offset = frame as f32 * 0.018; // forward speed — slow drift, no nausea

        let cam_ay = self.angle_y; // very subtle drift
        let cam_ax = self.angle_x;

        let cx = (w as f32) * 0.5;
        let cy = (h as f32) * 0.5;

        // ── 1. Vanishing point glow — small, cheap, bright center ──
        {
            let max_rad = 40i32;
            let pulse = 0.85 + 0.15 * fast_sin(frame as f32 * 0.03);
            let mr2 = max_rad * max_rad;
            for ry in -max_rad..=max_rad {
                let py = cy as i32 + ry;
                if py < 0 || py >= h as i32 { continue; }
                let ry2 = ry * ry;
                for rx in -max_rad..=max_rad {
                    let d2 = rx * rx + ry2;
                    if d2 > mr2 { continue; }
                    let px_x = cx as i32 + rx;
                    if px_x < 0 || px_x >= w as i32 { continue; }
                    let t = 1.0 - (d2 as f32 / mr2 as f32);
                    let t = t * t * pulse;
                    let g = (t * 45.0) as u32;
                    let b = (t * 15.0) as u32;
                    if g > 0 {
                        let glow = 0xFF000000 | (g.min(255) << 8) | b.min(255);
                        let idx = py as usize * w + px_x as usize;
                        if idx < buf.len() {
                            let dst = buf[idx];
                            let rr = ((dst >> 16) & 0xFF) + ((glow >> 16) & 0xFF);
                            let gg = ((dst >> 8) & 0xFF) + ((glow >> 8) & 0xFF);
                            let bb = (dst & 0xFF) + (glow & 0xFF);
                            buf[idx] = 0xFF000000 | (rr.min(255) << 16) | (gg.min(255) << 8) | bb.min(255);
                        }
                    }
                }
            }
        }

        // ── 2. Render rain columns in tunnel ──
        for col_idx in 0..NUM_COLS {
            let ring = col_idx / COLS_PER_RING;
            let slot = col_idx % COLS_PER_RING;

            // Deterministic pseudo-random seed per column
            let seed = (col_idx as u32).wrapping_mul(2654435761).wrapping_add(374761393);
            let seed2 = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            let seed3 = seed2.wrapping_mul(1664525).wrapping_add(1013904223);

            // Polar angle around tunnel axis (evenly distributed with jitter)
            let base_angle = (slot as f32 / COLS_PER_RING as f32) * 6.2831853;
            let angle_jitter = (seed % 1000) as f32 / 1000.0 * 0.3 - 0.15;
            let theta = base_angle + angle_jitter;

            // Radial distance: rings at increasing radii (more columns further out for density)
            let ring_t = (ring as f32 + 0.3) / NUM_RINGS as f32;
            let radius = 0.3 + ring_t * TUBE_RADIUS;

            // Column position in cylinder
            let col_x = radius * fast_cos(theta);
            let col_y_center = radius * fast_sin(theta);

            // Z placement — spread along tunnel, offset by camera movement
            let col_z_base = (seed2 % 10000) as f32 / 10000.0 * Z_DEPTH;
            // Wrap Z modulo Z_DEPTH → infinite tunnel (demoscene distance-table trick)
            let col_z_raw = col_z_base - (cam_z_offset % Z_DEPTH);
            let col_z = ((col_z_raw % Z_DEPTH) + Z_DEPTH) % Z_DEPTH + Z_NEAR;

            // Rain parameters
            let speed = 0.012 + (seed3 % 1000) as f32 / 1000.0 * 0.02;
            let trail_len = 6 + (seed >> 12) as usize % 12;
            let char_spacing: f32 = 0.30;
            let phase = (seed2 >> 8) as f32 / 256.0;
            let y_range: f32 = 3.0 + radius * 0.5;

            // Head Y — drops fall downward within the column's local space
            let total_travel = y_range + trail_len as f32 * char_spacing + 2.0;
            let head_local_y = y_range * 0.5 - ((frame as f32 * speed + phase * y_range) % total_travel);

            for ci in 0..trail_len {
                let local_y = head_local_y + ci as f32 * char_spacing;
                if local_y > y_range || local_y < -y_range { continue; }

                // 3D position
                let vy = col_y_center + local_y;
                let v = V3 { x: col_x, y: vy, z: col_z };

                // Apply minimal camera drift (not full rotation!)
                let rotated = rotate_x(rotate_y(v, cam_ay), cam_ax);
                if rotated.z < Z_NEAR { continue; }

                let p = project(rotated);
                let (sx, sy) = to_screen(p, w, h);

                // ── Depth-based scaling: 1/z² foreshortening (stronger depth cue) ──
                let z_inv = 1.0 / rotated.z;
                let scale = (z_inv * z_inv * 30.0).max(1.0).min(4.0) as i32;

                let glyph_w = scale * 5;
                let glyph_h = scale * 7;
                if sx + glyph_w < -5 || sx - glyph_w > w as i32 + 5 { continue; }
                if sy + glyph_h < -5 || sy - glyph_h > h as i32 + 5 { continue; }

                // ── Brightness: head bright, trail fades squared ──
                let trail_t = ci as f32 / trail_len as f32;
                let trail_fade = (1.0 - trail_t) * (1.0 - trail_t);
                // Depth fog: far = dim (exponential for realism)
                let depth_fade = fast_exp(-rotated.z * 0.12);
                let brightness = trail_fade * depth_fade;

                if brightness < 0.015 { continue; }

                // ── Glyph selection with scramble ──
                let glyph_idx = (seed.wrapping_mul(ci as u32 + 1).wrapping_add(frame / 5)) as usize % 16;
                let glyph = GLYPHS[glyph_idx];

                // ── Color: green hologram palette ──
                let (cr, cg, cb) = if ci == 0 {
                    // Head: white-hot green (optic flow anchor point)
                    ((brightness * 200.0) as u32, (brightness * 255.0) as u32, (brightness * 210.0) as u32)
                } else if ci <= 2 {
                    ((brightness * 50.0) as u32, (brightness * 255.0) as u32, (brightness * 70.0) as u32)
                } else {
                    ((brightness * 8.0) as u32, (brightness * 230.0) as u32, (brightness * 20.0) as u32)
                };
                let cr = cr.min(255);
                let cg = cg.min(255);
                let cb = cb.min(255);
                let color = 0xFF000000 | (cr << 16) | (cg << 8) | cb;

                // ── Draw glyph bitmap, scaled — direct write for trail, additive for head ──
                let ox = sx - glyph_w / 2;
                let oy = sy - glyph_h / 2;
                let use_additive = ci <= 1; // only head chars need additive (might overlap glow)
                for row in 0..7i32 {
                    for col in 0..5i32 {
                        let bit_idx = row * 5 + col;
                        if (glyph >> bit_idx) & 1 == 0 { continue; }
                        for py in 0..scale {
                            let fy = oy + row * scale + py;
                            if fy < 0 || fy >= h as i32 { continue; }
                            let row_off = fy as usize * w;
                            for px in 0..scale {
                                let fx = ox + col * scale + px;
                                if fx < 0 || fx >= w as i32 { continue; }
                                let idx = row_off + fx as usize;
                                if use_additive {
                                    let dst = buf[idx];
                                    let rr = ((dst >> 16) & 0xFF) + ((color >> 16) & 0xFF);
                                    let gg = ((dst >> 8) & 0xFF) + ((color >> 8) & 0xFF);
                                    let bb = (dst & 0xFF) + (color & 0xFF);
                                    buf[idx] = 0xFF000000 | (rr.min(255) << 16) | (gg.min(255) << 8) | bb.min(255);
                                } else {
                                    buf[idx] = color;
                                }
                            }
                        }
                    }
                }

                // ── Green glow for head only (cheap: just 4 corner pixels) ──
                if ci == 0 && cg > 80 && scale >= 2 {
                    let glow_g = cg / 4;
                    let glow_color = 0xFF000000 | (glow_g << 8);
                    let corners: [(i32,i32); 4] = [
                        (ox - 1, oy - 1), (ox + glyph_w, oy - 1),
                        (ox - 1, oy + glyph_h), (ox + glyph_w, oy + glyph_h),
                    ];
                    for &(gx, gy) in &corners {
                        if gx >= 0 && gx < w as i32 && gy >= 0 && gy < h as i32 {
                            additive_blend(buf, w, h, gx, gy, glow_color);
                        }
                    }
                }
            }
        }

        // ── 3. Perspective ground grid — reinforces depth perception ──
        // Horizontal lines receding to vanishing point + vertical grid lines
        {
            let grid_color_base: u32 = 0x14; // dim green
            let horizon_y = cy as i32 + 30; // slightly below center
            let num_hz = 12; // horizontal depth lines
            let num_vt = 16; // vertical columns
            let grid_z_scroll = (frame as f32 * 0.35) % 48.0; // scroll with camera

            // Horizontal lines (receding — get closer together toward horizon)
            for i in 0..num_hz {
                let t = (i as f32 + grid_z_scroll / 48.0 * (num_hz as f32 / 4.0)) / num_hz as f32;
                let t = t.min(0.99);
                let screen_y = horizon_y + ((1.0 - t) * (1.0 - t) * (h as f32 - horizon_y as f32)) as i32;
                if screen_y < 0 || screen_y >= h as i32 { continue; }
                let depth_fade = (1.0 - t) * (1.0 - t);
                let g = (grid_color_base as f32 * depth_fade * 1.5) as u32;
                if g < 3 { continue; }
                let line_color = 0xFF000000 | (g.min(255) << 8) | (g.min(255) / 4);
                let row_off = screen_y as usize * w;
                // Draw horizontal line — skip every other pixel for speed
                let mut px = 0;
                while px < w {
                    let idx = row_off + px;
                    if idx < buf.len() {
                        let dst = buf[idx];
                        let gg = ((dst >> 8) & 0xFF) + ((line_color >> 8) & 0xFF);
                        let bb = (dst & 0xFF) + (line_color & 0xFF);
                        buf[idx] = 0xFF000000 | (gg.min(255) << 8) | bb.min(255);
                    }
                    px += 2; // skip pixel for perf
                }
            }

            // Vertical lines converging to vanishing point (sparse, every 4th pixel)
            for i in 0..num_vt {
                let vx = (i as f32 / num_vt as f32) * w as f32;
                let vx = vx as i32;
                let bottom_y = h as i32 - 1;
                let steps = (bottom_y - horizon_y).max(1);
                let mut s = 0;
                while s < steps {
                    let t = s as f32 / steps as f32;
                    let sy = horizon_y + s;
                    if sy >= h as i32 { break; }
                    if sy < 0 { s += 4; continue; }
                    let sx = cx as i32 + ((vx - cx as i32) as f32 * t * t) as i32;
                    if sx >= 0 && sx < w as i32 {
                        let depth_fade = t * t;
                        let g = (grid_color_base as f32 * depth_fade * 1.2) as u32;
                        if g >= 2 {
                            let idx = sy as usize * w + sx as usize;
                            if idx < buf.len() {
                                let line_color = 0xFF000000 | (g.min(255) << 8) | (g.min(255) / 4);
                                let dst = buf[idx];
                                let gg = ((dst >> 8) & 0xFF) + ((line_color >> 8) & 0xFF);
                                let bb = (dst & 0xFF) + (line_color & 0xFF);
                                buf[idx] = 0xFF000000 | (gg.min(255) << 8) | bb.min(255);
                            }
                        }
                    }
                    s += 3; // skip pixels for perf
                }
            }
        }

        // ── 4. Radial speed lines — optic flow streaks from center ──
        // These reinforce the perception of forward motion (like starfield trails)
        {
            let num_streaks = 30;
            for i in 0..num_streaks {
                let seed = (i as u32).wrapping_mul(2654435761).wrapping_add(frame.wrapping_mul(7));
                let seed2 = seed.wrapping_mul(1664525).wrapping_add(1013904223);
                let angle = (i as f32 / num_streaks as f32) * 6.2831853 + (seed % 1000) as f32 * 0.001;
                // Distance from center: pulsating
                let base_dist = 80.0 + (seed2 % 400) as f32;
                let life = ((frame.wrapping_mul(3).wrapping_add(seed)) % 120) as f32 / 120.0;
                let dist = base_dist + life * 400.0;
                let len = 3.0 + life * 15.0;

                let cos_a = fast_cos(angle);
                let sin_a = fast_sin(angle);
                let x0 = cx as i32 + (dist * cos_a) as i32;
                let y0 = cy as i32 + (dist * sin_a) as i32;
                let x1 = cx as i32 + ((dist + len) * cos_a) as i32;
                let y1 = cy as i32 + ((dist + len) * sin_a) as i32;

                let fade = ((1.0 - life) * 40.0) as u32;
                if fade < 2 { continue; }
                let streak_color = 0xFF000000 | (fade.min(255) << 8) | (fade.min(255) / 3);
                draw_line(buf, w, h, x0, y0, x1, y1, streak_color);
            }
        }

    }

    /// Hologram scanline effect — Star Wars-style horizontal sweep + CRT scanlines
    fn render_scanlines(&self, buf: &mut [u32], w: usize, h: usize) {
        // Sweeping bright beam that bounces up and down
        let cycle = (h as f32) * 2.0;
        let raw_y = (self.frame as f32 * 1.8) % cycle;
        let sweep_y = if raw_y > h as f32 { cycle - raw_y } else { raw_y };
        let sweep_y = sweep_y as i32;

        for y in 0..h {
            let row_start = y * w;
            let row_end = row_start + w;
            if row_end > buf.len() { break; }

            // CRT scanline: every 3rd row gets dimmed
            if y % 3 == 0 {
                for px in buf[row_start..row_end].iter_mut() {
                    let r = ((*px >> 16) & 0xFF) * 200 / 256;
                    let g = ((*px >> 8) & 0xFF) * 200 / 256;
                    let b = (*px & 0xFF) * 200 / 256;
                    *px = 0xFF000000 | (r << 16) | (g << 8) | b;
                }
            }

            // Bright sweep beam (±6 pixels around sweep_y)
            let dist = (y as i32 - sweep_y).unsigned_abs();
            if dist < 6 {
                let boost = (6 - dist) * 5;
                for px in buf[row_start..row_end].iter_mut() {
                    let r = (((*px >> 16) & 0xFF) + boost).min(255);
                    let g = (((*px >> 8) & 0xFF) + boost).min(255);
                    let b = ((*px & 0xFF) + boost).min(255);
                    *px = 0xFF000000 | (r << 16) | (g << 8) | b;
                }
            }

            // Hologram flicker: subtle random-ish horizontal offset jitter
            let jitter_seed = (y as u32).wrapping_mul(2654435761).wrapping_add(self.frame);
            if jitter_seed % 97 < 3 {
                // Shift row slightly (2px) for glitch effect
                let shift = 2;
                let row = &mut buf[row_start..row_end];
                if w > shift {
                    for x in (shift..w).rev() {
                        row[x] = row[x - shift];
                    }
                    for x in 0..shift {
                        row[x] = 0xFF000000;
                    }
                }
            }
        }
    }

    fn render_rain(&self, buf: &mut [u32], w: usize, h: usize) {
        // Simple fast rain: vertical streaks based on frame counter
        let num_drops = w / 12;
        let frame = self.frame;
        
        for i in 0..num_drops {
            // Deterministic pseudo-random per column
            let seed = (i as u32).wrapping_mul(2654435761);
            let x = (seed % w as u32) as i32;
            let speed = 2 + (seed >> 16) % 5;
            let len = 4 + (seed >> 8) % 12;
            let y_base = ((frame.wrapping_mul(speed)) % (h as u32 + len)) as i32;
            
            for j in 0..len as i32 {
                let y = y_base - j;
                if y >= 0 && y < h as i32 {
                    let fade = (len as i32 - j) as u32 * 255 / len as u32;
                    let g = (fade * 180 / 255).min(255);
                    let color = 0xFF000000 | (g << 8);
                    let idx = y as usize * w + x as usize;
                    if idx < buf.len() {
                        buf[idx] = color;
                    }
                }
            }
        }
    }
}
