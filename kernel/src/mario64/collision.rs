//! TrustMario64 — Collision System
//! Ray-triangle (Möller–Trumbore), AABB, sphere, terrain floor/wall detection
#![allow(dead_code)]

use alloc::vec::Vec;
use super::physics::{Vec3, clamp, fast_sqrt, MARIO_RADIUS, MARIO_HEIGHT};
use crate::math::fast_sqrt as _sqrt;

// ======================== Triangle ========================

#[derive(Clone, Copy, Debug)]
pub struct Triangle {
    pub v0: Vec3,
    pub v1: Vec3,
    pub v2: Vec3,
    pub normal: Vec3,
    pub color: u32,
    pub surface_type: SurfaceType,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SurfaceType {
    Default,
    Grass,
    Dirt,
    Stone,
    Water,
    Ice,
    Lava,
    DeathPlane,
    WallOnly, // not walkable
}

impl Triangle {
    pub fn new(v0: Vec3, v1: Vec3, v2: Vec3, color: u32, surface_type: SurfaceType) -> Self {
        let e1 = v1.sub(v0);
        let e2 = v2.sub(v0);
        let normal = e1.cross(e2).normalize();
        Self { v0, v1, v2, normal, color, surface_type }
    }

    pub fn center(&self) -> Vec3 {
        Vec3::new(
            (self.v0.x + self.v1.x + self.v2.x) / 3.0,
            (self.v0.y + self.v1.y + self.v2.y) / 3.0,
            (self.v0.z + self.v1.z + self.v2.z) / 3.0,
        )
    }
}

// ======================== AABB ========================

#[derive(Clone, Copy, Debug)]
pub struct AABB {
    pub min: Vec3,
    pub max: Vec3,
}

impl AABB {
    pub fn new(min: Vec3, max: Vec3) -> Self { Self { min, max } }

    pub fn from_center(center: Vec3, half: Vec3) -> Self {
        Self { min: center.sub(half), max: center.add(half) }
    }

    pub fn contains_point(&self, p: Vec3) -> bool {
        p.x >= self.min.x && p.x <= self.max.x
            && p.y >= self.min.y && p.y <= self.max.y
            && p.z >= self.min.z && p.z <= self.max.z
    }

    pub fn intersects(&self, other: &AABB) -> bool {
        self.min.x <= other.max.x && self.max.x >= other.min.x
            && self.min.y <= other.max.y && self.max.y >= other.min.y
            && self.min.z <= other.max.z && self.max.z >= other.min.z
    }

    pub fn expand(&self, margin: f32) -> Self {
        let m = Vec3::new(margin, margin, margin);
        Self { min: self.min.sub(m), max: self.max.add(m) }
    }
}

// ======================== Collision Mesh ========================

pub struct CollisionMesh {
    pub floors: Vec<Triangle>,   // walkable surfaces (normal.y > 0.5)
    pub walls: Vec<Triangle>,    // vertical surfaces
    pub ceilings: Vec<Triangle>, // overhead surfaces (normal.y < -0.5)
    pub bounds: AABB,
}

impl CollisionMesh {
    pub fn new() -> Self {
        Self {
            floors: Vec::new(),
            walls: Vec::new(),
            ceilings: Vec::new(),
            bounds: AABB::new(Vec3::ZERO, Vec3::ZERO),
        }
    }

    pub fn add_triangle(&mut self, tri: Triangle) {
        if tri.normal.y > 0.5 {
            self.floors.push(tri);
        } else if tri.normal.y < -0.5 {
            self.ceilings.push(tri);
        } else {
            self.walls.push(tri);
        }
    }

    pub fn build_from_triangles(tris: &[Triangle]) -> Self {
        let mut mesh = Self::new();
        let mut min = Vec3::new(f32::MAX, f32::MAX, f32::MAX);
        let mut max = Vec3::new(f32::MIN, f32::MIN, f32::MIN);

        for tri in tris {
            mesh.add_triangle(*tri);
            for v in [tri.v0, tri.v1, tri.v2] {
                if v.x < min.x { min.x = v.x; }
                if v.y < min.y { min.y = v.y; }
                if v.z < min.z { min.z = v.z; }
                if v.x > max.x { max.x = v.x; }
                if v.y > max.y { max.y = v.y; }
                if v.z > max.z { max.z = v.z; }
            }
        }
        mesh.bounds = AABB::new(min, max);
        mesh
    }
}

// ======================== Ray-Triangle (Möller–Trumbore) ========================

pub fn ray_triangle_intersect(
    origin: Vec3, dir: Vec3, tri: &Triangle,
) -> Option<(f32, f32, f32)> {
    // Returns (t, u, v) where t = distance along ray
    let e1 = tri.v1.sub(tri.v0);
    let e2 = tri.v2.sub(tri.v0);
    let h = dir.cross(e2);
    let a = e1.dot(h);

    if a > -0.00001 && a < 0.00001 { return None; }

    let f = 1.0 / a;
    let s = origin.sub(tri.v0);
    let u = f * s.dot(h);
    if u < 0.0 || u > 1.0 { return None; }

    let q = s.cross(e1);
    let v = f * dir.dot(q);
    if v < 0.0 || u + v > 1.0 { return None; }

    let t = f * e2.dot(q);
    if t > 0.0001 { Some((t, u, v)) } else { None }
}

// ======================== Floor detection (SM64 style) ========================

#[derive(Clone, Copy, Debug)]
pub struct FloorResult {
    pub height: f32,
    pub normal: Vec3,
    pub surface_type: SurfaceType,
    pub found: bool,
}

impl Default for FloorResult {
    fn default() -> Self {
        Self { height: f32::MIN, normal: Vec3::UP, surface_type: SurfaceType::Default, found: false }
    }
}

/// Find the floor triangle directly below the given position
/// Uses a downward ray from (x, y+100, z) in -Y direction
pub fn find_floor(pos: Vec3, mesh: &CollisionMesh) -> FloorResult {
    let origin = Vec3::new(pos.x, pos.y + 100.0, pos.z);
    let dir = Vec3::new(0.0, -1.0, 0.0);
    let mut best = FloorResult::default();

    for tri in &mesh.floors {
        // Quick XZ bounds check
        let cx = tri.center().x;
        let cz = tri.center().z;
        if (pos.x - cx).abs() > 50.0 || (pos.z - cz).abs() > 50.0 { continue; }

        if let Some((t, _u, _v)) = ray_triangle_intersect(origin, dir, tri) {
            let hit_y = origin.y - t;
            if hit_y <= pos.y + 0.1 && hit_y > best.height {
                best.height = hit_y;
                best.normal = tri.normal;
                best.surface_type = tri.surface_type;
                best.found = true;
            }
        }
    }
    best
}

// ======================== Wall collision (SM64 quarter-step style) ========================

#[derive(Clone, Copy, Debug)]
pub struct WallResult {
    pub pushed: bool,
    pub new_pos: Vec3,
    pub wall_normal: Vec3,
}

/// Check wall collision: try to move from old_pos to new_pos, push out of walls
pub fn check_wall_collision(old_pos: Vec3, new_pos: Vec3, radius: f32, mesh: &CollisionMesh) -> WallResult {
    let mut result = WallResult {
        pushed: false,
        new_pos,
        wall_normal: Vec3::ZERO,
    };

    for tri in &mesh.walls {
        // Project position onto wall plane
        let dist = result.new_pos.sub(tri.v0).dot(tri.normal);
        if dist.abs() < radius {
            // Check if we're actually near this triangle's area
            let proj = result.new_pos.sub(tri.normal.scale(dist));
            if point_near_triangle(proj, tri, radius * 2.0) {
                // Push out
                let push = radius - dist;
                result.new_pos = result.new_pos.add(tri.normal.scale(push));
                result.wall_normal = tri.normal;
                result.pushed = true;
            }
        }
    }
    result
}

/// Rough check if a point is near a triangle (within margin)
fn point_near_triangle(p: Vec3, tri: &Triangle, margin: f32) -> bool {
    let center = tri.center();
    // Simple distance check to triangle center
    let max_edge = {
        let d0 = tri.v0.dist(tri.v1);
        let d1 = tri.v1.dist(tri.v2);
        let d2 = tri.v2.dist(tri.v0);
        if d0 > d1 { if d0 > d2 { d0 } else { d2 } } else { if d1 > d2 { d1 } else { d2 } }
    };
    p.dist(center) < max_edge + margin
}

// ======================== Ceiling detection ========================

pub fn find_ceiling(pos: Vec3, mesh: &CollisionMesh) -> Option<f32> {
    let origin = Vec3::new(pos.x, pos.y, pos.z);
    let dir = Vec3::new(0.0, 1.0, 0.0);
    let mut best_height = f32::MAX;
    let mut found = false;

    for tri in &mesh.ceilings {
        if let Some((t, _u, _v)) = ray_triangle_intersect(origin, dir, tri) {
            let hit_y = origin.y + t;
            if hit_y < best_height {
                best_height = hit_y;
                found = true;
            }
        }
    }
    if found { Some(best_height) } else { None }
}

// ======================== Sphere-sphere collision ========================

pub fn sphere_overlap(pos_a: Vec3, rad_a: f32, pos_b: Vec3, rad_b: f32) -> bool {
    pos_a.dist(pos_b) < rad_a + rad_b
}

pub fn sphere_overlap_xz(pos_a: Vec3, rad_a: f32, pos_b: Vec3, rad_b: f32) -> bool {
    pos_a.dist_xz(pos_b) < rad_a + rad_b
}

// ======================== Cylinder collision (for trees, poles) ========================

pub fn cylinder_overlap(
    pos: Vec3, radius: f32,
    cyl_base: Vec3, cyl_radius: f32, cyl_height: f32,
) -> bool {
    if pos.y > cyl_base.y + cyl_height || pos.y + MARIO_HEIGHT < cyl_base.y {
        return false;
    }
    pos.dist_xz(cyl_base) < radius + cyl_radius
}

/// Push a position out of a cylinder
pub fn push_out_of_cylinder(
    pos: Vec3, radius: f32,
    cyl_center: Vec3, cyl_radius: f32,
) -> Vec3 {
    let dx = pos.x - cyl_center.x;
    let dz = pos.z - cyl_center.z;
    let dist = fast_sqrt(dx * dx + dz * dz);
    let min_dist = radius + cyl_radius;

    if dist < min_dist && dist > 0.001 {
        let push = (min_dist - dist) / dist;
        Vec3::new(pos.x + dx * push, pos.y, pos.z + dz * push)
    } else {
        pos
    }
}
