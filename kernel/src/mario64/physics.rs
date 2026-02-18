//! TrustMario64 — Physics Engine
//! Constants and math adapted from the SM64 decompilation (n64decomp/sm64)
#![allow(dead_code)]

pub use crate::math::{fast_sin, fast_cos, fast_sqrt, fast_atan2, fast_tan};

// ======================== SM64-authentic constants ========================

pub const PI: f32 = 3.14159265;
pub const TWO_PI: f32 = 6.2831853;
pub const HALF_PI: f32 = 1.5707963;
pub const DEG_TO_RAD: f32 = 0.01745329;

// Gravity & movement
pub const GRAVITY: f32 = -0.8;
pub const TERMINAL_VEL: f32 = -15.0;
pub const MAX_WALK_SPEED: f32 = 4.0;
pub const MAX_RUN_SPEED: f32 = 8.0;
pub const JUMP_VEL: f32 = 10.0;
pub const DOUBLE_JUMP_VEL: f32 = 12.0;
pub const TRIPLE_JUMP_VEL: f32 = 14.0;
pub const WALL_KICK_VEL: f32 = 10.0;
pub const LONG_JUMP_VEL_Y: f32 = 5.0;
pub const LONG_JUMP_SPEED: f32 = 12.0;
pub const BACKFLIP_VEL: f32 = 13.0;
pub const GROUND_POUND_VEL: f32 = -15.0;
pub const SIDE_FLIP_VEL: f32 = 11.0;
pub const GROUND_ACCEL: f32 = 0.8;
pub const GROUND_DECEL: f32 = 0.5;
pub const GROUND_FRICTION: f32 = 0.92;
pub const AIR_ACCEL: f32 = 0.3;
pub const AIR_DRAG: f32 = 0.35;
pub const TURN_SPEED_GROUND: f32 = 0.15;
pub const TURN_SPEED_AIR: f32 = 0.06;
pub const MARIO_HEIGHT: f32 = 1.7;
pub const MARIO_RADIUS: f32 = 0.5;
pub const SWIM_GRAVITY: f32 = -0.2;
pub const SWIM_SPEED: f32 = 4.0;
pub const DAMAGE_KNOCKBACK: f32 = 8.0;

// ======================== Vec3 ========================

#[derive(Clone, Copy, Debug, Default)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub const ZERO: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    pub const UP: Vec3 = Vec3 { x: 0.0, y: 1.0, z: 0.0 };
    pub const FORWARD: Vec3 = Vec3 { x: 0.0, y: 0.0, z: -1.0 };
    pub const RIGHT: Vec3 = Vec3 { x: 1.0, y: 0.0, z: 0.0 };

    pub const fn new(x: f32, y: f32, z: f32) -> Self { Self { x, y, z } }
    pub fn length(&self) -> f32 { fast_sqrt(self.x * self.x + self.y * self.y + self.z * self.z) }
    pub fn length_sq(&self) -> f32 { self.x * self.x + self.y * self.y + self.z * self.z }
    pub fn length_xz(&self) -> f32 { fast_sqrt(self.x * self.x + self.z * self.z) }

    pub fn normalize(&self) -> Self {
        let l = self.length();
        if l < 0.0001 { return Self::ZERO; }
        Self { x: self.x / l, y: self.y / l, z: self.z / l }
    }

    pub fn dot(&self, o: Vec3) -> f32 { self.x * o.x + self.y * o.y + self.z * o.z }

    pub fn cross(&self, o: Vec3) -> Vec3 {
        Vec3 {
            x: self.y * o.z - self.z * o.y,
            y: self.z * o.x - self.x * o.z,
            z: self.x * o.y - self.y * o.x,
        }
    }

    pub fn lerp(&self, o: Vec3, t: f32) -> Vec3 {
        Vec3 {
            x: self.x + (o.x - self.x) * t,
            y: self.y + (o.y - self.y) * t,
            z: self.z + (o.z - self.z) * t,
        }
    }

    pub fn scale(&self, s: f32) -> Vec3 { Vec3 { x: self.x * s, y: self.y * s, z: self.z * s } }
    pub fn add(&self, o: Vec3) -> Vec3 { Vec3 { x: self.x + o.x, y: self.y + o.y, z: self.z + o.z } }
    pub fn sub(&self, o: Vec3) -> Vec3 { Vec3 { x: self.x - o.x, y: self.y - o.y, z: self.z - o.z } }
    pub fn neg(&self) -> Vec3 { Vec3 { x: -self.x, y: -self.y, z: -self.z } }
    pub fn dist(&self, o: Vec3) -> f32 { self.sub(o).length() }

    pub fn dist_xz(&self, o: Vec3) -> f32 {
        let dx = self.x - o.x;
        let dz = self.z - o.z;
        fast_sqrt(dx * dx + dz * dz)
    }

    pub fn with_y(&self, y: f32) -> Vec3 { Vec3 { x: self.x, y, z: self.z } }
}

// ======================== Mat4 (column-major) ========================

#[derive(Clone, Copy, Debug)]
pub struct Mat4 {
    pub m: [f32; 16],
}

impl Mat4 {
    pub const IDENTITY: Mat4 = Mat4 {
        m: [
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0,
        ],
    };

    pub fn translation(x: f32, y: f32, z: f32) -> Self {
        let mut r = Self::IDENTITY;
        r.m[12] = x; r.m[13] = y; r.m[14] = z;
        r
    }

    pub fn scaling(x: f32, y: f32, z: f32) -> Self {
        let mut r = Self::IDENTITY;
        r.m[0] = x; r.m[5] = y; r.m[10] = z;
        r
    }

    pub fn rotation_y(angle: f32) -> Self {
        let (s, c) = (fast_sin(angle), fast_cos(angle));
        let mut r = Self::IDENTITY;
        r.m[0] = c;  r.m[2] = s;
        r.m[8] = -s; r.m[10] = c;
        r
    }

    pub fn rotation_x(angle: f32) -> Self {
        let (s, c) = (fast_sin(angle), fast_cos(angle));
        let mut r = Self::IDENTITY;
        r.m[5] = c;  r.m[6] = -s;
        r.m[9] = s;  r.m[10] = c;
        r
    }

    pub fn rotation_z(angle: f32) -> Self {
        let (s, c) = (fast_sin(angle), fast_cos(angle));
        let mut r = Self::IDENTITY;
        r.m[0] = c;  r.m[1] = -s;
        r.m[4] = s;  r.m[5] = c;
        r
    }

    pub fn perspective(fov: f32, aspect: f32, near: f32, far: f32) -> Self {
        let f = 1.0 / fast_tan(fov * 0.5);
        let nf = 1.0 / (near - far);
        Mat4 {
            m: [
                f / aspect, 0.0,  0.0,                  0.0,
                0.0,        f,    0.0,                  0.0,
                0.0,        0.0,  (far + near) * nf,   -1.0,
                0.0,        0.0,  2.0 * far * near * nf, 0.0,
            ],
        }
    }

    pub fn look_at(eye: Vec3, target: Vec3, up: Vec3) -> Self {
        let f = target.sub(eye).normalize();
        let r = f.cross(up).normalize();
        let u = r.cross(f);
        Mat4 {
            m: [
                r.x, u.x, -f.x, 0.0,
                r.y, u.y, -f.y, 0.0,
                r.z, u.z, -f.z, 0.0,
                -r.dot(eye), -u.dot(eye), f.dot(eye), 1.0,
            ],
        }
    }

    pub fn mul(&self, b: &Mat4) -> Mat4 {
        let mut r = [0.0f32; 16];
        for i in 0..4 {
            for j in 0..4 {
                r[j * 4 + i] = self.m[i] * b.m[j * 4]
                    + self.m[4 + i] * b.m[j * 4 + 1]
                    + self.m[8 + i] * b.m[j * 4 + 2]
                    + self.m[12 + i] * b.m[j * 4 + 3];
            }
        }
        Mat4 { m: r }
    }

    /// Transform a point through the full MVP pipeline (with perspective divide)
    pub fn transform(&self, v: Vec3) -> Vec3 {
        let w = self.m[3] * v.x + self.m[7] * v.y + self.m[11] * v.z + self.m[15];
        let inv_w = if w.abs() > 0.0001 { 1.0 / w } else { 1.0 };
        Vec3 {
            x: (self.m[0] * v.x + self.m[4] * v.y + self.m[8] * v.z + self.m[12]) * inv_w,
            y: (self.m[1] * v.x + self.m[5] * v.y + self.m[9] * v.z + self.m[13]) * inv_w,
            z: (self.m[2] * v.x + self.m[6] * v.y + self.m[10] * v.z + self.m[14]) * inv_w,
        }
    }

    /// Transform without perspective divide (for normals, directions)
    pub fn transform_dir(&self, v: Vec3) -> Vec3 {
        Vec3 {
            x: self.m[0] * v.x + self.m[4] * v.y + self.m[8] * v.z,
            y: self.m[1] * v.x + self.m[5] * v.y + self.m[9] * v.z,
            z: self.m[2] * v.x + self.m[6] * v.y + self.m[10] * v.z,
        }
    }
}

// ======================== SM64 approach_f32 ========================
// Smoothly moves current toward target with asymmetric inc/dec rates

pub fn approach_f32(current: f32, target: f32, inc: f32, dec: f32) -> f32 {
    if current < target {
        let r = current + inc;
        if r > target { target } else { r }
    } else {
        let r = current - dec;
        if r < target { target } else { r }
    }
}

pub fn clamp(x: f32, min: f32, max: f32) -> f32 {
    if x < min { min } else if x > max { max } else { x }
}

pub fn clamp_i32(x: i32, min: i32, max: i32) -> i32 {
    if x < min { min } else if x > max { max } else { x }
}

/// Convert facing angle to XZ forward direction
pub fn angle_to_forward(angle: f32) -> Vec3 {
    Vec3::new(fast_sin(angle), 0.0, -fast_cos(angle))
}

/// Convert XZ direction to angle (0 = -Z, PI/2 = +X)
pub fn forward_to_angle(dir: Vec3) -> f32 {
    fast_atan2(dir.x, -dir.z)
}

/// Wrap angle to [-PI, PI]
pub fn wrap_angle(a: f32) -> f32 {
    let mut r = a;
    while r > PI { r -= TWO_PI; }
    while r < -PI { r += TWO_PI; }
    r
}

/// Shortest angular difference
pub fn angle_diff(from: f32, to: f32) -> f32 {
    wrap_angle(to - from)
}

/// Linearly interpolate angle (shortest path)
pub fn lerp_angle(from: f32, to: f32, t: f32) -> f32 {
    from + angle_diff(from, to) * t
}

// ======================== Color helpers ========================

pub fn rgb(r: u8, g: u8, b: u8) -> u32 {
    0xFF000000 | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}

pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> u32 {
    ((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}

pub fn color_mul(c: u32, factor: f32) -> u32 {
    let r = (((c >> 16) & 0xFF) as f32 * factor) as u32;
    let g = (((c >> 8) & 0xFF) as f32 * factor) as u32;
    let b = ((c & 0xFF) as f32 * factor) as u32;
    0xFF000000 | (r.min(255) << 16) | (g.min(255) << 8) | b.min(255)
}

pub fn color_lerp(a: u32, b: u32, t: f32) -> u32 {
    let ra = ((a >> 16) & 0xFF) as f32;
    let ga = ((a >> 8) & 0xFF) as f32;
    let ba = (a & 0xFF) as f32;
    let rb = ((b >> 16) & 0xFF) as f32;
    let gb = ((b >> 8) & 0xFF) as f32;
    let bb = (b & 0xFF) as f32;
    let r = (ra + (rb - ra) * t) as u32;
    let g = (ga + (gb - ga) * t) as u32;
    let bi = (ba + (bb - ba) * t) as u32;
    0xFF000000 | (r.min(255) << 16) | (g.min(255) << 8) | bi.min(255)
}

// SM64 colors
pub const COLOR_MARIO_RED: u32 = 0xFFFF0000;
pub const COLOR_MARIO_BLUE: u32 = 0xFF0000CC;
pub const COLOR_MARIO_SKIN: u32 = 0xFFFFCC88;
pub const COLOR_MARIO_BROWN: u32 = 0xFF664422;
pub const COLOR_MARIO_WHITE: u32 = 0xFFFFFFFF;
pub const COLOR_GRASS: u32 = 0xFF44AA22;
pub const COLOR_DIRT: u32 = 0xFF886644;
pub const COLOR_STONE: u32 = 0xFF888888;
pub const COLOR_WATER: u32 = 0xFF2266CC;
pub const COLOR_SKY_TOP: u32 = 0xFF4488FF;
pub const COLOR_SKY_BOT: u32 = 0xFFAADDFF;
pub const COLOR_COIN: u32 = 0xFFFFCC00;
pub const COLOR_STAR: u32 = 0xFFFFFF00;
pub const COLOR_GOOMBA_BODY: u32 = 0xFFBB8844;
pub const COLOR_GOOMBA_FEET: u32 = 0xFF442200;
pub const COLOR_BOBOMB_BODY: u32 = 0xFF222222;
pub const COLOR_BOBOMB_EYES: u32 = 0xFFFFFFFF;
pub const COLOR_CHAIN_BODY: u32 = 0xFF333333;
pub const COLOR_HUD_BG: u32 = 0x80000000;
pub const COLOR_HUD_WHITE: u32 = 0xFFFFFFFF;
pub const COLOR_HUD_YELLOW: u32 = 0xFFFFCC00;
pub const COLOR_HUD_RED: u32 = 0xFFFF0000;
pub const COLOR_HEALTH_FULL: u32 = 0xFF00FF00;
pub const COLOR_HEALTH_MED: u32 = 0xFFFFCC00;
pub const COLOR_HEALTH_LOW: u32 = 0xFFFF0000;
