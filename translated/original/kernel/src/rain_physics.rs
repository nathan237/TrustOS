// ═══════════════════════════════════════════════════════════════════════════
// Rain Physics — SDF-based scene structures with gravity rain flow
// ═══════════════════════════════════════════════════════════════════════════
//
// Defines scene structures (tree, lake) as 2D Signed Distance Fields (SDFs).
// Rain glyphs query the SDF each frame to determine interaction:
//
//   • Surface flow  — deflect glyph x-position along surface tangent
//   • Gravity stream — rain "ruisselle" down convex canopy surfaces
//   • Silhouette     — denser/dimmer rain inside shapes reveals their form
//   • Lake shimmer  — bright ripple line at water surface
//   • Underwater dim — deep blue fade below lake
//   • Rain shadow   — subtle dimming below canopy
//   • Drip zone     — brightness pulse where drops fall from canopy edge
//
// All shapes are pure math (SDF): O(1) per query, zero geometry memory.
// Performance: ~100 float ops per query, ~17K queries/frame → <1ms.
//
// Mathematical foundation:
//   SDF(p) > 0  →  p is outside the shape
//   SDF(p) = 0  →  p is exactly on the surface
//   SDF(p) < 0  →  p is inside the shape
//   ∇SDF(p)     →  surface normal direction (for flow computation)
//   tangent     =  (-ny, nx), perpendicular to normal
//   flow        =  dot(gravity, tangent) = gravity_y * nx
//
// SDF primitives from Inigo Quilez (iquilezles.org/articles/distfunctions).
// ═══════════════════════════════════════════════════════════════════════════

// ═══════════════════════════════════════
// 2D SDF Primitives
// ═══════════════════════════════════════

/// Signed distance to a circle: negative inside, positive outside.
#[inline]
fn sd_circle(px: f32, py: f32, cx: f32, cy: f32, r: f32) -> f32 {
    let dx = px - cx;
    let dy = py - cy;
    libm::sqrtf(dx * dx + dy * dy) - r
}

/// Signed distance to a capsule (line segment with radius).
/// Perfect for tree trunks, branches, and roots.
#[inline]
fn sd_capsule(px: f32, py: f32, ax: f32, ay: f32, bx: f32, by: f32, r: f32) -> f32 {
    let pax = px - ax;
    let pay = py - ay;
    let bax = bx - ax;
    let bay = by - ay;
    let ba2 = bax * bax + bay * bay;
    let h = if ba2 > 0.001 {
        let t = (pax * bax + pay * bay) / ba2;
        if t < 0.0 { 0.0 } else if t > 1.0 { 1.0 } else { t }
    } else {
        0.0
    };
    let dx = pax - bax * h;
    let dy = pay - bay * h;
    libm::sqrtf(dx * dx + dy * dy) - r
}

/// Smooth union of two SDFs (polynomial, from Quilez).
/// k controls the blending radius in pixels — larger k = smoother join.
#[inline]
fn op_smooth_union(a: f32, b: f32, k: f32) -> f32 {
    let k4 = k * 4.0;
    let d = k4 - libm::fabsf(a - b);
    let h = if d > 0.0 { d } else { 0.0 };
    let m = if a < b { a } else { b };
    m - h * h * 0.25 / k4.max(0.001)
}

// ═══════════════════════════════════════
// Scene State
// ═══════════════════════════════════════

pub struct RainSceneState {
    pub initialized: bool,
    pub w: f32,
    pub h: f32,

    // ── Tree trunk (vertical capsule) ──
    pub trunk_cx: f32,
    pub trunk_top_y: f32,
    pub trunk_bot_y: f32,
    pub trunk_r: f32,

    // ── Canopy (3 smooth-unioned circles) ──
    pub can1_cx: f32, pub can1_cy: f32, pub can1_r: f32,  // main dome
    pub can2_cx: f32, pub can2_cy: f32, pub can2_r: f32,  // left lobe
    pub can3_cx: f32, pub can3_cy: f32, pub can3_r: f32,  // right lobe

    // ── Branches (capsules from trunk to outer lobes) ──
    pub br_l_ax: f32, pub br_l_ay: f32, pub br_l_bx: f32, pub br_l_by: f32, pub br_l_r: f32,
    pub br_r_ax: f32, pub br_r_ay: f32, pub br_r_bx: f32, pub br_r_by: f32, pub br_r_r: f32,

    // ── Roots (small capsules at trunk base) ──
    pub rt_l_ax: f32, pub rt_l_ay: f32, pub rt_l_bx: f32, pub rt_l_by: f32, pub rt_l_r: f32,
    pub rt_r_ax: f32, pub rt_r_ay: f32, pub rt_r_bx: f32, pub rt_r_by: f32, pub rt_r_r: f32,

    // ── Lake (horizontal water surface) ──
    pub lake_y: f32,
    pub lake_left: f32,
    pub lake_right: f32,

    // ── Bounding box for fast rejection ──
    pub tree_bb_left: f32,
    pub tree_bb_right: f32,
    pub tree_bb_top: f32,
    pub tree_bb_bot: f32,

    // ── Animation ──
    pub wind_phase: f32,
    pub frame: u64,
}

impl RainSceneState {
    pub const fn new() -> Self {
        Self {
            initialized: false,
            w: 0.0, h: 0.0,
            trunk_cx: 0.0, trunk_top_y: 0.0, trunk_bot_y: 0.0, trunk_r: 0.0,
            can1_cx: 0.0, can1_cy: 0.0, can1_r: 0.0,
            can2_cx: 0.0, can2_cy: 0.0, can2_r: 0.0,
            can3_cx: 0.0, can3_cy: 0.0, can3_r: 0.0,
            br_l_ax: 0.0, br_l_ay: 0.0, br_l_bx: 0.0, br_l_by: 0.0, br_l_r: 0.0,
            br_r_ax: 0.0, br_r_ay: 0.0, br_r_bx: 0.0, br_r_by: 0.0, br_r_r: 0.0,
            rt_l_ax: 0.0, rt_l_ay: 0.0, rt_l_bx: 0.0, rt_l_by: 0.0, rt_l_r: 0.0,
            rt_r_ax: 0.0, rt_r_ay: 0.0, rt_r_bx: 0.0, rt_r_by: 0.0, rt_r_r: 0.0,
            lake_y: 0.0, lake_left: 0.0, lake_right: 0.0,
            tree_bb_left: 0.0, tree_bb_right: 0.0, tree_bb_top: 0.0, tree_bb_bot: 0.0,
            wind_phase: 0.0, frame: 0,
        }
    }
}

// ═══════════════════════════════════════
// Scene Construction
// ═══════════════════════════════════════

/// Build the tree + lake scene proportional to screen dimensions.
/// Tree positioned at left-third to coexist with ghost mesh sphere at center.
pub fn init_scene(s: &mut RainSceneState, w: u32, h: u32) {
    let w = w as f32;
    let h = h as f32;
    s.w = w;
    s.h = h;

    // ── Tree trunk ──
    // Positioned at 30% of screen width (left third)
    s.trunk_cx   = w * 0.30;
    s.trunk_top_y = h * 0.48;      // where trunk meets canopy
    s.trunk_bot_y = h * 0.78;      // at ground/lake level
    s.trunk_r     = w * 0.010;     // ~19px at 1920

    // ── Canopy (3 overlapping circles for organic shape) ──
    // Main dome centered above trunk
    s.can1_cx = s.trunk_cx;
    s.can1_cy = h * 0.32;
    s.can1_r  = w * 0.080;         // ~154px at 1920

    // Left lobe (shifted left and slightly lower)
    s.can2_cx = s.trunk_cx - w * 0.063;   // ~121px left
    s.can2_cy = h * 0.38;
    s.can2_r  = w * 0.050;                // ~96px

    // Right lobe (shifted right and slightly lower)
    s.can3_cx = s.trunk_cx + w * 0.063;
    s.can3_cy = h * 0.38;
    s.can3_r  = w * 0.050;

    // ── Branches (capsules from upper trunk to outer lobes) ──
    let boff = s.trunk_r * 0.6;
    // Left branch
    s.br_l_ax = s.trunk_cx - boff;
    s.br_l_ay = s.trunk_top_y - h * 0.02;
    s.br_l_bx = s.can2_cx + s.can2_r * 0.3;
    s.br_l_by = s.can2_cy + s.can2_r * 0.4;
    s.br_l_r  = w * 0.005;                    // ~10px
    // Right branch
    s.br_r_ax = s.trunk_cx + boff;
    s.br_r_ay = s.trunk_top_y - h * 0.02;
    s.br_r_bx = s.can3_cx - s.can3_r * 0.3;
    s.br_r_by = s.can3_cy + s.can3_r * 0.4;
    s.br_r_r  = w * 0.005;

    // ── Roots (small capsules spreading from trunk base into ground/lake) ──
    s.rt_l_ax = s.trunk_cx - s.trunk_r * 0.3;
    s.rt_l_ay = s.trunk_bot_y;
    s.rt_l_bx = s.trunk_cx - w * 0.025;
    s.rt_l_by = s.trunk_bot_y + h * 0.025;
    s.rt_l_r  = w * 0.004;

    s.rt_r_ax = s.trunk_cx + s.trunk_r * 0.3;
    s.rt_r_ay = s.trunk_bot_y;
    s.rt_r_bx = s.trunk_cx + w * 0.025;
    s.rt_r_by = s.trunk_bot_y + h * 0.025;
    s.rt_r_r  = w * 0.004;

    // ── Lake (spans most of screen width) ──
    s.lake_y     = h * 0.82;
    s.lake_left  = w * 0.08;
    s.lake_right = w * 0.92;

    // ── Bounding box for tree (with generous margin for wind sway + influence) ──
    let margin = 60.0;
    s.tree_bb_left  = (s.can2_cx - s.can2_r - margin).max(0.0);
    s.tree_bb_right = (s.can3_cx + s.can3_r + margin).min(w);
    s.tree_bb_top   = (s.can1_cy - s.can1_r - margin).max(0.0);
    s.tree_bb_bot   = (s.rt_l_by.max(s.rt_r_by) + margin).min(h);

    s.initialized = true;
}

// ═══════════════════════════════════════
// SDF Evaluation
// ═══════════════════════════════════════

/// Evaluate tree SDF at (px, py). Wind sway applied to canopy.
/// Returns signed distance: negative = inside, positive = outside.
fn tree_sdf(px: f32, py: f32, s: &RainSceneState) -> f32 {
    // Gentle wind sway: canopy oscillates, trunk barely moves
    let wind_dx = libm::sinf(s.wind_phase) * s.can1_r * 0.04;
    let wind_dy = libm::sinf(s.wind_phase * 0.7 + 1.0) * s.can1_r * 0.015;

    // Trunk (base fixed, top sways slightly)
    let trunk = sd_capsule(
        px, py,
        s.trunk_cx, s.trunk_bot_y,
        s.trunk_cx + wind_dx * 0.1, s.trunk_top_y + wind_dy * 0.2,
        s.trunk_r,
    );

    // Canopy blobs (full wind sway)
    let c1 = sd_circle(px, py, s.can1_cx + wind_dx, s.can1_cy + wind_dy, s.can1_r);
    let c2 = sd_circle(px, py, s.can2_cx + wind_dx * 0.8, s.can2_cy + wind_dy * 0.8, s.can2_r);
    let c3 = sd_circle(px, py, s.can3_cx + wind_dx * 0.8, s.can3_cy + wind_dy * 0.8, s.can3_r);
    let canopy = op_smooth_union(c1, op_smooth_union(c2, c3, 25.0), 25.0);

    // Branches (partial sway)
    let bl = sd_capsule(
        px, py,
        s.br_l_ax + wind_dx * 0.1, s.br_l_ay + wind_dy * 0.2,
        s.br_l_bx + wind_dx * 0.8, s.br_l_by + wind_dy * 0.8,
        s.br_l_r,
    );
    let br = sd_capsule(
        px, py,
        s.br_r_ax + wind_dx * 0.1, s.br_r_ay + wind_dy * 0.2,
        s.br_r_bx + wind_dx * 0.8, s.br_r_by + wind_dy * 0.8,
        s.br_r_r,
    );
    let branches = op_smooth_union(bl, br, 15.0);

    // Roots (fixed, no wind)
    let rl = sd_capsule(px, py, s.rt_l_ax, s.rt_l_ay, s.rt_l_bx, s.rt_l_by, s.rt_l_r);
    let rr = sd_capsule(px, py, s.rt_r_ax, s.rt_r_ay, s.rt_r_bx, s.rt_r_by, s.rt_r_r);
    let roots = op_smooth_union(rl, rr, 10.0);

    // Combine: trunk ∪ canopy ∪ branches ∪ roots
    let tree = op_smooth_union(trunk, canopy, 18.0);
    let tree = op_smooth_union(tree, branches, 12.0);
    op_smooth_union(tree, roots, 10.0)
}

/// Compute 2D surface normal via central finite differences.
/// Returns normalized (nx, ny). The normal points AWAY from the surface.
fn tree_normal(px: f32, py: f32, s: &RainSceneState) -> (f32, f32) {
    let eps = 3.0;
    let dx = tree_sdf(px + eps, py, s) - tree_sdf(px - eps, py, s);
    let dy = tree_sdf(px, py + eps, s) - tree_sdf(px, py - eps, s);
    let len = libm::sqrtf(dx * dx + dy * dy);
    if len > 0.001 {
        (dx / len, dy / len)
    } else {
        (0.0, -1.0)
    }
}

// ═══════════════════════════════════════
// Rain Interaction
// ═══════════════════════════════════════

/// Result of querying the rain physics at a pixel position.
/// Applied to each rain glyph to create surface flow, dimming, and color effects.
pub struct RainInteraction {
    /// Horizontal pixel deflection (surface tangent flow).
    /// Positive = shift right, negative = shift left.
    pub x_offset: i32,
    /// Brightness multiplier (0.0 = invisible, 1.0 = normal, 2.0 = double).
    pub brightness: f32,
    /// Additive color shift (signed: can brighten or dim individual channels).
    pub color_r: i16,
    pub color_g: i16,
    pub color_b: i16,
    /// True if the glyph is on the tree surface (for special glyph selection).
    pub on_surface: bool,
    /// True if the glyph is in the lake area.
    pub in_lake: bool,
}

impl RainInteraction {
    pub const NONE: Self = Self {
        x_offset: 0,
        brightness: 1.0,
        color_r: 0, color_g: 0, color_b: 0,
        on_surface: false,
        in_lake: false,
    };
}

/// Update scene animation (call once per frame).
pub fn update(state: &mut RainSceneState) {
    if !state.initialized { return; }
    state.frame = state.frame.wrapping_add(1);
    // Gentle wind: full cycle every ~26 seconds at 30fps
    state.wind_phase = (state.frame as f32) * 0.008;
}

/// Query rain interaction at pixel position (px, py).
///
/// This is called for EVERY rain glyph each frame.
/// Uses bounding box culling to skip most queries cheaply.
pub fn query_rain(s: &RainSceneState, px: f32, py: f32) -> RainInteraction {
    if !s.initialized {
        return RainInteraction::NONE;
    }

    // ── Fast bounding box rejection for tree ──
    let near_tree = px >= s.tree_bb_left && px <= s.tree_bb_right
                 && py >= s.tree_bb_top  && py <= s.tree_bb_bot;

    // ── Fast check for lake ──
    let in_lake_band = py >= s.lake_y - 8.0
                    && px >= s.lake_left && px <= s.lake_right;

    if !near_tree && !in_lake_band {
        return RainInteraction::NONE;
    }

    // ────────────────────────────────────────
    // TREE INTERACTION
    // ────────────────────────────────────────
    if near_tree {
        let tree_d = tree_sdf(px, py, s);

        const SURFACE_BAND: f32  = 15.0;   // "on surface" thickness (pixels)
        const INNER_ZONE: f32    = 8.0;    // beyond this depth = deep interior
        const FLOW_STRENGTH: f32 = 8.0;    // max horizontal deflection (pixels)
        const INFLUENCE: f32     = 45.0;   // max range for any tree effect

        if tree_d < INFLUENCE {
            // ── Deep inside tree: dim rain for dark silhouette ──
            if tree_d < -INNER_ZONE {
                let depth = (-tree_d - INNER_ZONE).min(60.0);
                let dim = (depth / 60.0).min(0.80);  // up to 80% dimming
                return RainInteraction {
                    x_offset: 0,
                    brightness: 0.20 + (1.0 - dim) * 0.3,  // 0.20 - 0.50
                    color_r: -10,
                    color_g: 5,
                    color_b: -15,
                    on_surface: false,
                    in_lake: false,
                };
            }

            // ── On the surface: flow + glow ──
            if libm::fabsf(tree_d) <= SURFACE_BAND {
                let (nx, ny) = tree_normal(px, py, s);

                // Flow direction: project gravity onto surface tangent
                // tangent = (-ny, nx), gravity = (0, 1)
                // flow_x = dot((0,1), (-ny,nx)) = nx
                let flow = nx * FLOW_STRENGTH;

                // Brightness peaks at exact surface, fades at band edges
                let surface_t = 1.0 - libm::fabsf(tree_d) / SURFACE_BAND;
                let bright = 1.3 + surface_t * 0.7;  // 1.3 → 2.0 at surface

                // Color: warm water highlight (white-green-teal tint)
                let cr = (20.0 + surface_t * 30.0) as i16;
                let cg = (30.0 + surface_t * 40.0) as i16;
                let cb = (25.0 + surface_t * 30.0) as i16;

                return RainInteraction {
                    x_offset: flow as i32,
                    brightness: bright,
                    color_r: cr,
                    color_g: cg,
                    color_b: cb,
                    on_surface: true,
                    in_lake: false,
                };
            }

            // ── Influence zone: drip effect + rain shadow ──
            let falloff_t = (tree_d - SURFACE_BAND) / (INFLUENCE - SURFACE_BAND);
            let falloff_t = falloff_t.max(0.0).min(1.0);

            // Drip zone: just below canopy bottom edge
            // Rain that flowed along the dome drips off, creating bright pulses
            let canopy_bot = s.can1_cy + s.can1_r * 0.6;
            let above_trunk = py < s.trunk_top_y + 30.0;
            if py > canopy_bot && above_trunk && tree_d < 30.0 {
                let drip_t = 1.0 - falloff_t;
                return RainInteraction {
                    x_offset: 0,
                    brightness: 1.1 + drip_t * 0.3,
                    color_r: (8.0 * drip_t) as i16,
                    color_g: (15.0 * drip_t) as i16,
                    color_b: (10.0 * drip_t) as i16,
                    on_surface: false,
                    in_lake: false,
                };
            }

            // Rain shadow: subtle dimming below the canopy
            let below_canopy = py > s.can1_cy + s.can1_r * 0.3;
            if below_canopy && tree_d < 35.0 {
                let shadow = 0.12 * (1.0 - falloff_t);
                return RainInteraction {
                    x_offset: 0,
                    brightness: 1.0 - shadow,
                    color_r: 0,
                    color_g: 0,
                    color_b: 0,
                    on_surface: false,
                    in_lake: false,
                };
            }
        }
    }

    // ────────────────────────────────────────
    // LAKE INTERACTION
    // ────────────────────────────────────────
    if in_lake_band {
        let lake_d = py - s.lake_y;  // negative = above, positive = below

        const SURFACE_BAND: f32 = 6.0;  // lake surface highlight thickness

        // Lake surface: bright animated shimmer line
        if libm::fabsf(lake_d) < SURFACE_BAND {
            let surface_t = 1.0 - libm::fabsf(lake_d) / SURFACE_BAND;
            // Animated shimmer: sine wave along x + time
            let shimmer = libm::sinf(px * 0.05 + s.wind_phase * 3.0) * 0.3 + 0.7;
            let bright = 1.4 + surface_t * shimmer * 0.6;
            return RainInteraction {
                x_offset: 0,
                brightness: bright,
                color_r: -10,
                color_g: 10,
                color_b: 50,   // strong blue tint at water surface
                on_surface: false,
                in_lake: true,
            };
        }

        // Underwater: progressive deep blue dimming
        if lake_d > SURFACE_BAND {
            let depth = (lake_d - SURFACE_BAND).min(120.0) / 120.0;
            let dim = 0.12 + depth * 0.55;  // 12% to 67% dimming
            return RainInteraction {
                x_offset: 0,
                brightness: 1.0 - dim,
                color_r: -25,
                color_g: -8,
                color_b: 25,   // deep blue shift
                on_surface: false,
                in_lake: true,
            };
        }
    }

    RainInteraction::NONE
}
