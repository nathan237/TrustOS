//! TrustMario64 — Level: Bob-omb Battlefield
//! Procedural terrain generation inspired by SM64's first course
#![allow(dead_code)]

use alloc::vec::Vec;
use super::physics::*;
use super::collision::*;
use super::objects::{ObjectType, GameObject};
use super::enemies::{EnemyType, Enemy};
use crate::math::{fast_sin, fast_cos, fast_sqrt};

// ======================== Level Data ========================

pub struct LevelData {
    pub terrain_tris: Vec<Triangle>,
    pub collision: CollisionMesh,
    pub objects: Vec<GameObject>,
    pub enemies: Vec<Enemy>,
    pub spawn_pos: Vec3,
    pub star_positions: Vec<Vec3>,
    pub boundaries: AABB,
    pub water_height: f32,
    // Heightmap
    pub heightmap: Vec<f32>,
    pub hm_size: usize,
    pub hm_scale: f32,
    pub hm_offset: Vec3,
}

// ======================== Heightmap ========================

const HM_SIZE: usize = 32;
const HM_SCALE: f32 = 5.0; // each cell is 5 units
const WORLD_SIZE: f32 = HM_SIZE as f32 * HM_SCALE; // 160 units

impl LevelData {
    /// Generate Bob-omb Battlefield
    pub fn bob_omb_battlefield() -> Self {
        let hm_offset = Vec3::new(-WORLD_SIZE * 0.5, 0.0, -WORLD_SIZE * 0.5);
        let mut heightmap = alloc::vec![0.0f32; HM_SIZE * HM_SIZE];

        // === Build heightmap ===
        // Base: mostly flat with gentle slopes
        for z in 0..HM_SIZE {
            for x in 0..HM_SIZE {
                let fx = x as f32 / HM_SIZE as f32;
                let fz = z as f32 / HM_SIZE as f32;

                // Center mountain (SM64's central mountain)
                let mx = fx - 0.6;
                let mz = fz - 0.6;
                let md = fast_sqrt(mx * mx + mz * mz);
                let mountain = if md < 0.25 {
                    (0.25 - md) * 80.0 // peak at ~20 units high
                } else {
                    0.0
                };

                // Gentle hills
                let hills = (fast_sin(fx * 6.0) * fast_cos(fz * 5.0) + 1.0) * 0.5;

                // Path (slight elevation along a winding route)
                let path_x = 0.3 + fast_sin(fz * 3.0) * 0.1;
                let path_dist = (fx - path_x).abs();
                let path = if path_dist < 0.08 { 0.3 } else { 0.0 };

                // Water depression (lower area in the corner)
                let wx = fx - 0.15;
                let wz = fz - 0.15;
                let water_area = if fast_sqrt(wx * wx + wz * wz) < 0.15 { -2.0 } else { 0.0 };

                // Combine
                let h = mountain + hills + path + water_area;
                heightmap[z * HM_SIZE + x] = h;
            }
        }

        // === Triangulate heightmap ===
        let mut terrain_tris = Vec::new();
        for z in 0..HM_SIZE - 1 {
            for x in 0..HM_SIZE - 1 {
                let v00 = heightmap_vertex(x, z, &heightmap, hm_offset);
                let v10 = heightmap_vertex(x + 1, z, &heightmap, hm_offset);
                let v01 = heightmap_vertex(x, z + 1, &heightmap, hm_offset);
                let v11 = heightmap_vertex(x + 1, z + 1, &heightmap, hm_offset);

                let h = heightmap[z * HM_SIZE + x];
                let (color, surface) = terrain_color(h);

                terrain_tris.push(Triangle::new(v00, v10, v01, color, surface));
                terrain_tris.push(Triangle::new(v10, v11, v01, color, surface));
            }
        }

        // === Bridge (elevated flat section) ===
        let bridge_y = 3.0;
        let bx0 = -10.0; let bx1 = 10.0;
        let bz0 = 20.0; let bz1 = 25.0;
        let bv = [
            Vec3::new(bx0, bridge_y, bz0), Vec3::new(bx1, bridge_y, bz0),
            Vec3::new(bx0, bridge_y, bz1), Vec3::new(bx1, bridge_y, bz1),
        ];
        terrain_tris.push(Triangle::new(bv[0], bv[1], bv[2], COLOR_DIRT, SurfaceType::Default));
        terrain_tris.push(Triangle::new(bv[1], bv[3], bv[2], COLOR_DIRT, SurfaceType::Default));

        // === Water surface ===
        let ws = 15.0;
        let wy = -1.5;
        terrain_tris.push(Triangle::new(
            Vec3::new(-ws - 60.0, wy, -ws - 60.0),
            Vec3::new(ws - 60.0, wy, -ws - 60.0),
            Vec3::new(-ws - 60.0, wy, ws - 60.0),
            COLOR_WATER, SurfaceType::Water,
        ));
        terrain_tris.push(Triangle::new(
            Vec3::new(ws - 60.0, wy, -ws - 60.0),
            Vec3::new(ws - 60.0, wy, ws - 60.0),
            Vec3::new(-ws - 60.0, wy, ws - 60.0),
            COLOR_WATER, SurfaceType::Water,
        ));

        // Build collision mesh
        let collision = CollisionMesh::build_from_triangles(&terrain_tris);

        // === Place objects ===
        let mut objects = Vec::new();

        // Coins scattered across the field
        let coin_positions = [
            (5.0, 0.5, 5.0), (10.0, 0.5, 8.0), (15.0, 0.5, 3.0),
            (-5.0, 0.5, 10.0), (-10.0, 0.5, 15.0), (0.0, 0.5, 20.0),
            (8.0, 0.5, -5.0), (-3.0, 0.5, -8.0), (20.0, 1.0, 20.0),
            (12.0, 0.5, 12.0), (-8.0, 0.5, 5.0), (3.0, 0.5, 15.0),
            (18.0, 0.5, -3.0), (-15.0, 0.5, -5.0), (0.0, 0.5, -10.0),
            // Coins along path
            (-20.0, 1.0, 0.0), (-20.0, 1.0, 5.0), (-20.0, 1.0, 10.0),
            (-20.0, 1.0, 15.0), (-20.0, 1.0, 20.0),
            // Coin ring
            (25.0, 1.0, 0.0), (27.0, 1.2, 2.0), (25.0, 1.4, 4.0),
            (23.0, 1.2, 2.0),
        ];
        for (x, y, z) in &coin_positions {
            objects.push(GameObject::new(ObjectType::Coin, Vec3::new(*x, *y, *z)));
        }

        // Red coins (8 total, SM64 standard)
        let red_coins = [
            (30.0, 1.0, 30.0), (-25.0, 1.0, -25.0),
            (40.0, 5.0, 10.0), (-30.0, 1.0, 40.0),
            (10.0, 4.0, -30.0), (-40.0, 2.0, 0.0),
            (0.0, bridge_y + 0.5, 22.5), (35.0, 2.0, -20.0),
        ];
        for (x, y, z) in &red_coins {
            objects.push(GameObject::new(ObjectType::RedCoin, Vec3::new(*x, *y, *z)));
        }

        // Yellow blocks
        objects.push(GameObject::new(ObjectType::YellowBlock, Vec3::new(15.0, 2.0, 0.0)));
        objects.push(GameObject::new(ObjectType::YellowBlock, Vec3::new(-15.0, 2.0, 30.0)));
        objects.push(GameObject::new(ObjectType::YellowBlock, Vec3::new(30.0, 5.0, -10.0)));

        // Trees (collision cylinders)
        let tree_positions = [
            (20.0, 0.0, -15.0), (-20.0, 0.0, -15.0), (35.0, 0.0, 25.0),
            (-35.0, 0.0, 10.0), (0.0, 0.0, 35.0), (-10.0, 0.0, -30.0),
            (25.0, 0.0, 40.0), (-40.0, 0.0, -20.0),
        ];
        for (x, y, z) in &tree_positions {
            objects.push(GameObject::new(ObjectType::Tree, Vec3::new(*x, *y, *z)));
        }

        // Star positions (7 stars like SM64)
        let star_positions = alloc::vec![
            Vec3::new(40.0, 22.0, 40.0),  // Top of mountain
            Vec3::new(-40.0, 3.0, 40.0),  // Behind chain chomp
            Vec3::new(0.0, bridge_y + 5.0, 22.5), // Above bridge
            Vec3::new(30.0, 2.0, -30.0),  // Island area
            Vec3::new(-30.0, 5.0, -30.0), // Red coin star
            Vec3::new(0.0, 15.0, 0.0),    // Cannon star (center, high up)
        ];

        // Spawn star objects
        for pos in &star_positions {
            objects.push(GameObject::new(ObjectType::Star, *pos));
        }

        // === Place enemies ===
        let mut enemies = Vec::new();

        // Goombas
        let goomba_pos = [
            (10.0, 0.2, 10.0), (-10.0, 0.2, 10.0), (5.0, 0.2, 20.0),
            (20.0, 0.2, -10.0), (-5.0, 0.2, 30.0),
        ];
        for (x, y, z) in &goomba_pos {
            enemies.push(Enemy::new(EnemyType::Goomba, Vec3::new(*x, *y, *z)));
        }

        // Bob-ombs
        let bobomb_pos = [
            (-15.0, 0.2, 20.0), (25.0, 0.2, 15.0), (-25.0, 0.2, -10.0),
        ];
        for (x, y, z) in &bobomb_pos {
            enemies.push(Enemy::new(EnemyType::BobOmb, Vec3::new(*x, *y, *z)));
        }

        // Chain Chomp (near the stake at a fixed position)
        enemies.push(Enemy::new(EnemyType::ChainChomp, Vec3::new(-35.0, 0.2, 35.0)));

        // Boundaries
        let bounds = AABB::new(
            Vec3::new(-WORLD_SIZE * 0.5 - 10.0, -30.0, -WORLD_SIZE * 0.5 - 10.0),
            Vec3::new(WORLD_SIZE * 0.5 + 10.0, 50.0, WORLD_SIZE * 0.5 + 10.0),
        );

        Self {
            terrain_tris,
            collision,
            objects,
            enemies,
            spawn_pos: Vec3::new(0.0, 2.0, -50.0),
            star_positions,
            boundaries: bounds,
            water_height: -1.5,
            heightmap,
            hm_size: HM_SIZE,
            hm_scale: HM_SCALE,
            hm_offset: hm_offset,
        }
    }

    /// Sample terrain height at world position
    pub fn terrain_height_at(&self, wx: f32, wz: f32) -> f32 {
        let lx = (wx - self.hm_offset.x) / self.hm_scale;
        let lz = (wz - self.hm_offset.z) / self.hm_scale;

        let ix = lx as usize;
        let iz = lz as usize;

        if ix >= self.hm_size - 1 || iz >= self.hm_size - 1 {
            return 0.0;
        }

        let fx = lx - ix as f32;
        let fz = lz - iz as f32;

        let h00 = self.heightmap[iz * self.hm_size + ix];
        let h10 = self.heightmap[iz * self.hm_size + ix + 1];
        let h01 = self.heightmap[(iz + 1) * self.hm_size + ix];
        let h11 = self.heightmap[(iz + 1) * self.hm_size + ix + 1];

        let h0 = h00 + (h10 - h00) * fx;
        let h1 = h01 + (h11 - h01) * fx;
        h0 + (h1 - h0) * fz
    }
}

fn heightmap_vertex(x: usize, z: usize, hm: &[f32], offset: Vec3) -> Vec3 {
    Vec3::new(
        offset.x + x as f32 * HM_SCALE,
        hm[z * HM_SIZE + x],
        offset.z + z as f32 * HM_SCALE,
    )
}

fn terrain_color(height: f32) -> (u32, SurfaceType) {
    if height < -1.0 {
        (COLOR_WATER, SurfaceType::Water)
    } else if height < 0.5 {
        (COLOR_GRASS, SurfaceType::Grass)
    } else if height < 5.0 {
        (COLOR_DIRT, SurfaceType::Dirt)
    } else {
        (COLOR_STONE, SurfaceType::Stone)
    }
}
