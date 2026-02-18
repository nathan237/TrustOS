//! TrustMario64 — Software Rasterizer
//! Z-buffered triangle rasterizer with flat shading, directional light, fog, shadows
//! Renders to a pixel buffer for desktop window integration
#![allow(dead_code)]

use alloc::vec::Vec;
use super::physics::*;
use super::collision::Triangle as ColTri;
use super::player::MarioState;
use super::animation::{AnimState, BonePose, BodyPart, NUM_BODY_PARTS};
use super::camera::Camera;
use super::objects::GameObject;
use super::enemies::Enemy;
use super::level::LevelData;
use super::tas::{TasEngine, GhostFrame};
use crate::math::{fast_sin, fast_cos, fast_sqrt};

// ======================== Render Config ========================

pub const RENDER_W: usize = 320;
pub const RENDER_H: usize = 240;
pub const RENDER_PIXELS: usize = RENDER_W * RENDER_H;

// Light direction (sun from top-right-front)
const LIGHT_DIR: Vec3 = Vec3 { x: 0.4, y: -0.7, z: -0.5 };
const AMBIENT: f32 = 0.35;
const FOG_START: f32 = 80.0;
const FOG_END: f32 = 150.0;
const FOG_COLOR: u32 = COLOR_SKY_BOT;

// ======================== Render State ========================

pub struct RenderState {
    pub color_buf: Vec<u32>,
    pub depth_buf: Vec<f32>,
    pub mvp: Mat4,
    pub view: Mat4,
    pub proj: Mat4,
    pub cam_pos: Vec3,
}

impl RenderState {
    pub fn new() -> Self {
        Self {
            color_buf: alloc::vec![0u32; RENDER_PIXELS],
            depth_buf: alloc::vec![f32::MAX; RENDER_PIXELS],
            mvp: Mat4::IDENTITY,
            view: Mat4::IDENTITY,
            proj: Mat4::IDENTITY,
            cam_pos: Vec3::ZERO,
        }
    }

    pub fn begin_frame(&mut self, camera: &Camera, aspect: f32) {
        // Clear buffers
        for p in self.color_buf.iter_mut() { *p = 0; }
        for d in self.depth_buf.iter_mut() { *d = f32::MAX; }

        self.view = camera.get_view_matrix();
        self.proj = camera.get_projection_matrix(aspect);
        self.mvp = self.proj.mul(&self.view);
        self.cam_pos = camera.pos;
    }

    /// Draw sky gradient
    pub fn draw_sky(&mut self) {
        for y in 0..RENDER_H {
            let t = y as f32 / RENDER_H as f32;
            let color = color_lerp(COLOR_SKY_TOP, COLOR_SKY_BOT, t);
            for x in 0..RENDER_W {
                self.color_buf[y * RENDER_W + x] = color;
            }
        }
    }

    /// Render terrain triangles
    pub fn draw_terrain(&mut self, level: &LevelData) {
        for tri in &level.terrain_tris {
            self.draw_world_triangle(tri.v0, tri.v1, tri.v2, tri.color, tri.normal);
        }
    }

    /// Render Mario as a low-poly colored model
    pub fn draw_mario(&mut self, mario: &MarioState) {
        let pos = mario.pos;
        let yaw = mario.facing_yaw;
        let root = Mat4::translation(pos.x, pos.y, pos.z)
            .mul(&Mat4::rotation_y(yaw));

        // Get bone poses from animation
        let poses = &mario.anim.current_poses;
        let root_off = mario.anim.current_root_offset;
        let root = root.mul(&Mat4::translation(root_off.x, root_off.y, root_off.z));

        // Invincibility flash
        if mario.invincible_timer > 0 && mario.invincible_timer % 4 < 2 {
            return; // flicker
        }

        // Draw body parts
        self.draw_mario_head(&root, poses);
        self.draw_mario_body(&root, poses);
        self.draw_mario_arms(&root, poses);
        self.draw_mario_legs(&root, poses);
    }

    fn draw_mario_head(&mut self, root: &Mat4, poses: &[BonePose; NUM_BODY_PARTS]) {
        let head_offset = Mat4::translation(0.0, 1.4, 0.0)
            .mul(&Mat4::rotation_x(poses[BodyPart::Head as usize].rx));

        let head = root.mul(&head_offset);
        let s = 0.2; // head half-size

        // Face (skin)
        self.draw_box(&head, Vec3::new(-s, -s, -s), Vec3::new(s, s, s), COLOR_MARIO_SKIN);

        // Cap (red, on top)
        let cap = head.mul(&Mat4::translation(0.0, s, 0.0));
        self.draw_box(&cap, Vec3::new(-s * 1.1, 0.0, -s * 1.1), Vec3::new(s * 1.1, s * 0.6, s * 1.1), COLOR_MARIO_RED);

        // Mustache
        let must = head.mul(&Mat4::translation(0.0, -s * 0.3, -s));
        self.draw_box(&must, Vec3::new(-s * 0.6, -s * 0.1, -0.02), Vec3::new(s * 0.6, s * 0.1, 0.02), COLOR_MARIO_BROWN);
    }

    fn draw_mario_body(&mut self, root: &Mat4, poses: &[BonePose; NUM_BODY_PARTS]) {
        let torso_p = &poses[BodyPart::Torso as usize];
        let torso = root.mul(&Mat4::translation(0.0, 0.7, 0.0))
            .mul(&Mat4::rotation_x(torso_p.rx));

        // Upper body (red shirt)
        self.draw_box(&torso, Vec3::new(-0.22, 0.2, -0.15), Vec3::new(0.22, 0.5, 0.15), COLOR_MARIO_RED);

        // Lower body (blue overalls)
        let hips_p = &poses[BodyPart::Hips as usize];
        let hips = root.mul(&Mat4::translation(0.0, 0.4, 0.0))
            .mul(&Mat4::rotation_x(hips_p.rx));
        self.draw_box(&hips, Vec3::new(-0.23, -0.1, -0.15), Vec3::new(0.23, 0.25, 0.15), COLOR_MARIO_BLUE);
    }

    fn draw_mario_arms(&mut self, root: &Mat4, poses: &[BonePose; NUM_BODY_PARTS]) {
        for side in 0..2 {
            let sign = if side == 0 { -1.0f32 } else { 1.0 };
            let upper_idx = if side == 0 { BodyPart::LeftUpperArm } else { BodyPart::RightUpperArm } as usize;
            let lower_idx = if side == 0 { BodyPart::LeftLowerArm } else { BodyPart::RightLowerArm } as usize;

            let upper_p = &poses[upper_idx];
            let shoulder = root.mul(&Mat4::translation(sign * 0.3, 1.1, 0.0))
                .mul(&Mat4::rotation_x(upper_p.rx))
                .mul(&Mat4::rotation_z(upper_p.rz));

            // Upper arm (red)
            self.draw_box(&shoulder, Vec3::new(-0.06, -0.25, -0.06), Vec3::new(0.06, 0.0, 0.06), COLOR_MARIO_RED);

            // Lower arm / glove (white)
            let lower_p = &poses[lower_idx];
            let elbow = shoulder.mul(&Mat4::translation(0.0, -0.25, 0.0))
                .mul(&Mat4::rotation_x(lower_p.rx));
            self.draw_box(&elbow, Vec3::new(-0.07, -0.22, -0.07), Vec3::new(0.07, 0.0, 0.07), COLOR_MARIO_WHITE);
        }
    }

    fn draw_mario_legs(&mut self, root: &Mat4, poses: &[BonePose; NUM_BODY_PARTS]) {
        for side in 0..2 {
            let sign = if side == 0 { -1.0f32 } else { 1.0 };
            let upper_idx = if side == 0 { BodyPart::LeftUpperLeg } else { BodyPart::RightUpperLeg } as usize;
            let lower_idx = if side == 0 { BodyPart::LeftLowerLeg } else { BodyPart::RightLowerLeg } as usize;

            let upper_p = &poses[upper_idx];
            let hip_joint = root.mul(&Mat4::translation(sign * 0.12, 0.4, 0.0))
                .mul(&Mat4::rotation_x(upper_p.rx));

            // Upper leg (blue overalls)
            self.draw_box(&hip_joint, Vec3::new(-0.08, -0.3, -0.08), Vec3::new(0.08, 0.0, 0.08), COLOR_MARIO_BLUE);

            // Lower leg / shoe (brown)
            let lower_p = &poses[lower_idx];
            let knee = hip_joint.mul(&Mat4::translation(0.0, -0.3, 0.0))
                .mul(&Mat4::rotation_x(lower_p.rx));
            self.draw_box(&knee, Vec3::new(-0.09, -0.2, -0.12), Vec3::new(0.09, 0.0, 0.08), COLOR_MARIO_BROWN);
        }
    }

    /// Render game objects (coins, stars, blocks, trees)
    pub fn draw_objects(&mut self, objects: &[GameObject]) {
        for obj in objects {
            if !obj.visible || obj.collected { continue; }

            let dp = obj.display_pos();
            let model = Mat4::translation(dp.x, dp.y, dp.z)
                .mul(&Mat4::rotation_y(obj.rotation));

            match obj.obj_type {
                super::objects::ObjectType::Coin | super::objects::ObjectType::RedCoin
                | super::objects::ObjectType::BlueCoin => {
                    // Flat circle (2 triangles)
                    let c = obj.get_color();
                    let s = 0.3;
                    self.draw_box(&model, Vec3::new(-s, -0.02, -s * 0.3), Vec3::new(s, 0.02, s * 0.3), c);
                }
                super::objects::ObjectType::Star => {
                    let c = obj.get_color();
                    // Star as a diamond shape
                    self.draw_box(&model, Vec3::new(-0.4, -0.4, -0.1), Vec3::new(0.4, 0.4, 0.1), c);
                    self.draw_box(&model, Vec3::new(-0.1, -0.1, -0.4), Vec3::new(0.1, 0.1, 0.4), c);
                }
                super::objects::ObjectType::YellowBlock | super::objects::ObjectType::BreakableBox => {
                    let c = obj.get_color();
                    self.draw_box(&model, Vec3::new(-0.5, 0.0, -0.5), Vec3::new(0.5, 1.0, 0.5), c);
                }
                super::objects::ObjectType::Tree => {
                    // Trunk
                    self.draw_box(&model, Vec3::new(-0.2, 0.0, -0.2), Vec3::new(0.2, 3.0, 0.2), COLOR_MARIO_BROWN);
                    // Foliage
                    let foliage = model.mul(&Mat4::translation(0.0, 3.5, 0.0));
                    self.draw_box(&foliage, Vec3::new(-1.2, -0.8, -1.2), Vec3::new(1.2, 0.8, 1.2), 0xFF228833);
                }
                super::objects::ObjectType::OneUp => {
                    self.draw_box(&model, Vec3::new(-0.2, 0.0, -0.2), Vec3::new(0.2, 0.4, 0.2), 0xFF00CC00);
                }
                _ => {
                    let c = obj.get_color();
                    self.draw_box(&model, Vec3::new(-0.3, 0.0, -0.3), Vec3::new(0.3, 0.6, 0.3), c);
                }
            }
        }
    }

    /// Render enemies
    pub fn draw_enemies(&mut self, enemies: &[Enemy]) {
        for enemy in enemies {
            let model = Mat4::translation(enemy.pos.x, enemy.pos.y, enemy.pos.z)
                .mul(&Mat4::rotation_y(enemy.facing));

            if !enemy.alive {
                if enemy.death_anim_timer < 30 {
                    // Flatten during death
                    let sq = 1.0 - enemy.death_anim_timer as f32 / 30.0;
                    let death_m = model.mul(&Mat4::scaling(1.0, sq.max(0.1), 1.0));
                    self.draw_box(&death_m, Vec3::new(-0.4, 0.0, -0.4), Vec3::new(0.4, 0.8, 0.4), enemy.get_color());
                }
                continue;
            }

            match enemy.enemy_type {
                super::enemies::EnemyType::Goomba => {
                    // Body (mushroom shape)
                    self.draw_box(&model, Vec3::new(-0.3, 0.0, -0.3), Vec3::new(0.3, 0.5, 0.3), COLOR_GOOMBA_BODY);
                    // Head (wider)
                    let head = model.mul(&Mat4::translation(0.0, 0.5, 0.0));
                    self.draw_box(&head, Vec3::new(-0.35, 0.0, -0.35), Vec3::new(0.35, 0.35, 0.35), COLOR_GOOMBA_BODY);
                    // Feet
                    self.draw_box(&model, Vec3::new(-0.35, -0.05, -0.12), Vec3::new(-0.05, 0.08, 0.12), COLOR_GOOMBA_FEET);
                    self.draw_box(&model, Vec3::new(0.05, -0.05, -0.12), Vec3::new(0.35, 0.08, 0.12), COLOR_GOOMBA_FEET);
                    // Eyes (white)
                    let eye = model.mul(&Mat4::translation(0.0, 0.55, -0.3));
                    self.draw_box(&eye, Vec3::new(-0.15, -0.05, -0.02), Vec3::new(-0.02, 0.1, 0.02), COLOR_MARIO_WHITE);
                    self.draw_box(&eye, Vec3::new(0.02, -0.05, -0.02), Vec3::new(0.15, 0.1, 0.02), COLOR_MARIO_WHITE);
                }
                super::enemies::EnemyType::BobOmb => {
                    // Body (round bomb)
                    self.draw_box(&model, Vec3::new(-0.3, 0.0, -0.3), Vec3::new(0.3, 0.6, 0.3), COLOR_BOBOMB_BODY);
                    // Eyes
                    let eyes = model.mul(&Mat4::translation(0.0, 0.35, -0.28));
                    self.draw_box(&eyes, Vec3::new(-0.1, -0.05, -0.02), Vec3::new(0.1, 0.1, 0.02), COLOR_BOBOMB_EYES);
                    // Fuse (on top)
                    if enemy.fuse_timer > 0 {
                        let fuse = model.mul(&Mat4::translation(0.0, 0.65, 0.0));
                        let fuse_color = if enemy.fuse_timer % 10 < 5 { COLOR_HUD_RED } else { COLOR_HUD_YELLOW };
                        self.draw_box(&fuse, Vec3::new(-0.03, 0.0, -0.03), Vec3::new(0.03, 0.15, 0.03), fuse_color);
                    }
                    // Feet
                    self.draw_box(&model, Vec3::new(-0.2, -0.08, -0.08), Vec3::new(-0.05, 0.05, 0.08), COLOR_GOOMBA_FEET);
                    self.draw_box(&model, Vec3::new(0.05, -0.08, -0.08), Vec3::new(0.2, 0.05, 0.08), COLOR_GOOMBA_FEET);
                }
                super::enemies::EnemyType::ChainChomp => {
                    // Big spherical body
                    self.draw_box(&model, Vec3::new(-0.8, 0.0, -0.8), Vec3::new(0.8, 1.6, 0.8), COLOR_CHAIN_BODY);
                    // Eyes
                    let eyes = model.mul(&Mat4::translation(0.0, 1.0, -0.7));
                    self.draw_box(&eyes, Vec3::new(-0.25, -0.1, -0.02), Vec3::new(-0.05, 0.15, 0.02), COLOR_MARIO_WHITE);
                    self.draw_box(&eyes, Vec3::new(0.05, -0.1, -0.02), Vec3::new(0.25, 0.15, 0.02), COLOR_MARIO_WHITE);
                    // Mouth
                    let mouth = model.mul(&Mat4::translation(0.0, 0.5, -0.75));
                    self.draw_box(&mouth, Vec3::new(-0.35, -0.1, -0.02), Vec3::new(0.35, 0.1, 0.02), COLOR_HUD_RED);
                    // Chain (dotted line to tether)
                    self.draw_chain_links(enemy);
                }
            }
        }
    }

    fn draw_chain_links(&mut self, enemy: &Enemy) {
        let segments = 6;
        for i in 0..segments {
            let t = i as f32 / segments as f32;
            let link_pos = enemy.pos.lerp(enemy.tether_pos, t);
            let model = Mat4::translation(link_pos.x, link_pos.y + 0.3, link_pos.z);
            self.draw_box(&model, Vec3::new(-0.08, -0.08, -0.08), Vec3::new(0.08, 0.08, 0.08), 0xFF555555);
        }
    }

    /// Render shadow blob under Mario
    pub fn draw_shadow(&mut self, mario: &MarioState) {
        let sx = mario.pos.x;
        let sz = mario.pos.z;
        let sy = mario.floor_height + 0.02;
        let height_diff = (mario.pos.y - sy).max(0.1);
        let shadow_size = clamp(0.8 / (height_diff * 0.1 + 1.0), 0.2, 0.8);

        let model = Mat4::translation(sx, sy, sz);
        let s = shadow_size;
        // Flat dark quad on ground
        let v0 = model.transform(Vec3::new(-s, 0.0, -s));
        let v1 = model.transform(Vec3::new(s, 0.0, -s));
        let v2 = model.transform(Vec3::new(-s, 0.0, s));
        let v3 = model.transform(Vec3::new(s, 0.0, s));

        self.draw_world_triangle(v0, v1, v2, 0x40000000, Vec3::UP);
        self.draw_world_triangle(v1, v3, v2, 0x40000000, Vec3::UP);
    }

    /// Draw ghost Mario (transparent)
    pub fn draw_ghost(&mut self, ghost: &GhostFrame) {
        let model = Mat4::translation(ghost.pos.x, ghost.pos.y, ghost.pos.z)
            .mul(&Mat4::rotation_y(ghost.facing));
        // Simplified translucent Mario
        let alpha_red = 0x60FF0000;
        let alpha_blue = 0x600000CC;
        let alpha_skin = 0x60FFCC88;
        self.draw_box(&model, Vec3::new(-0.22, 0.4, -0.15), Vec3::new(0.22, 1.4, 0.15), alpha_blue);
        self.draw_box(&model, Vec3::new(-0.22, 1.0, -0.15), Vec3::new(0.22, 1.4, 0.15), alpha_red);
        let head = model.mul(&Mat4::translation(0.0, 1.4, 0.0));
        self.draw_box(&head, Vec3::new(-0.2, 0.0, -0.2), Vec3::new(0.2, 0.4, 0.2), alpha_skin);
    }

    // ======================== Core Rasterizer ========================

    /// Draw a world-space triangle through the full MVP pipeline
    fn draw_world_triangle(&mut self, v0: Vec3, v1: Vec3, v2: Vec3, color: u32, normal: Vec3) {
        // Transform to clip space
        let c0 = self.mvp.transform(v0);
        let c1 = self.mvp.transform(v1);
        let c2 = self.mvp.transform(v2);

        // Simple clip: if all behind camera, skip
        if c0.z < -1.0 && c1.z < -1.0 && c2.z < -1.0 { return; }
        if c0.z > 1.0 && c1.z > 1.0 && c2.z > 1.0 { return; }

        // NDC to screen
        let hw = RENDER_W as f32 * 0.5;
        let hh = RENDER_H as f32 * 0.5;
        let sx0 = ((c0.x + 1.0) * hw) as i32;
        let sy0 = ((1.0 - c0.y) * hh) as i32;
        let sz0 = c0.z;
        let sx1 = ((c1.x + 1.0) * hw) as i32;
        let sy1 = ((1.0 - c1.y) * hh) as i32;
        let sz1 = c1.z;
        let sx2 = ((c2.x + 1.0) * hw) as i32;
        let sy2 = ((1.0 - c2.y) * hh) as i32;
        let sz2 = c2.z;

        // Backface culling (optional based on winding)
        let cross = (sx1 - sx0) * (sy2 - sy0) - (sy1 - sy0) * (sx2 - sx0);
        if cross > 0 { return; } // cull clockwise

        // Compute lighting
        let light = compute_lighting(normal);

        // Distance for fog
        let center = Vec3::new(
            (v0.x + v1.x + v2.x) / 3.0,
            (v0.y + v1.y + v2.y) / 3.0,
            (v0.z + v1.z + v2.z) / 3.0,
        );
        let dist = self.cam_pos.dist(center);
        let fog = clamp((dist - FOG_START) / (FOG_END - FOG_START), 0.0, 1.0);

        let lit_color = color_mul(color, light);
        let final_color = if fog > 0.01 { color_lerp(lit_color, FOG_COLOR, fog) } else { lit_color };

        self.fill_triangle(sx0, sy0, sz0, sx1, sy1, sz1, sx2, sy2, sz2, final_color);
    }

    /// Draw a box (6 faces, 12 triangles) with a transform matrix
    fn draw_box(&mut self, transform: &Mat4, min: Vec3, max: Vec3, color: u32) {
        let corners = [
            Vec3::new(min.x, min.y, min.z), // 0
            Vec3::new(max.x, min.y, min.z), // 1
            Vec3::new(max.x, max.y, min.z), // 2
            Vec3::new(min.x, max.y, min.z), // 3
            Vec3::new(min.x, min.y, max.z), // 4
            Vec3::new(max.x, min.y, max.z), // 5
            Vec3::new(max.x, max.y, max.z), // 6
            Vec3::new(min.x, max.y, max.z), // 7
        ];

        let world: [Vec3; 8] = core::array::from_fn(|i| transform.transform_dir(corners[i]).add(Vec3::new(transform.m[12], transform.m[13], transform.m[14])));

        let faces: [(usize, usize, usize, usize, Vec3); 6] = [
            (0, 1, 2, 3, Vec3::new(0.0, 0.0, -1.0)), // front
            (5, 4, 7, 6, Vec3::new(0.0, 0.0, 1.0)),  // back
            (4, 0, 3, 7, Vec3::new(-1.0, 0.0, 0.0)), // left
            (1, 5, 6, 2, Vec3::new(1.0, 0.0, 0.0)),  // right
            (3, 2, 6, 7, Vec3::new(0.0, 1.0, 0.0)),  // top
            (4, 5, 1, 0, Vec3::new(0.0, -1.0, 0.0)), // bottom
        ];

        for &(a, b, c, d, normal) in &faces {
            let world_normal = transform.transform_dir(normal).normalize();
            self.draw_world_triangle(world[a], world[b], world[c], color, world_normal);
            self.draw_world_triangle(world[a], world[c], world[d], color, world_normal);
        }
    }

    /// Scanline triangle rasterizer with z-buffer
    fn fill_triangle(&mut self, x0: i32, y0: i32, z0: f32, x1: i32, y1: i32, z1: f32, x2: i32, y2: i32, z2: f32, color: u32) {
        // Bounding box
        let min_x = x0.min(x1).min(x2).max(0);
        let max_x = x0.max(x1).max(x2).min(RENDER_W as i32 - 1);
        let min_y = y0.min(y1).min(y2).max(0);
        let max_y = y0.max(y1).max(y2).min(RENDER_H as i32 - 1);

        if min_x > max_x || min_y > max_y { return; }

        // Edge function rasterizer
        let area = edge_fn(x0, y0, x1, y1, x2, y2);
        if area == 0 { return; }
        let inv_area = 1.0 / area as f32;

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                let w0 = edge_fn(x1, y1, x2, y2, x, y);
                let w1 = edge_fn(x2, y2, x0, y0, x, y);
                let w2 = edge_fn(x0, y0, x1, y1, x, y);

                if (w0 >= 0 && w1 >= 0 && w2 >= 0) || (w0 <= 0 && w1 <= 0 && w2 <= 0) {
                    let b0 = w0 as f32 * inv_area;
                    let b1 = w1 as f32 * inv_area;
                    let b2 = w2 as f32 * inv_area;

                    let z = z0 * b0 + z1 * b1 + z2 * b2;
                    let idx = y as usize * RENDER_W + x as usize;

                    if z < self.depth_buf[idx] && z > -1.0 && z < 1.0 {
                        self.depth_buf[idx] = z;
                        self.color_buf[idx] = color;
                    }
                }
            }
        }
    }

    /// Upscale internal buffer to output buffer
    pub fn blit_to_output(&self, out: &mut [u32], out_w: usize, out_h: usize) {
        let scale_x = out_w as f32 / RENDER_W as f32;
        let scale_y = out_h as f32 / RENDER_H as f32;

        for oy in 0..out_h {
            let sy = ((oy as f32 / scale_y) as usize).min(RENDER_H - 1);
            for ox in 0..out_w {
                let sx = ((ox as f32 / scale_x) as usize).min(RENDER_W - 1);
                out[oy * out_w + ox] = self.color_buf[sy * RENDER_W + sx];
            }
        }
    }
}

// ======================== Helpers ========================

fn edge_fn(x0: i32, y0: i32, x1: i32, y1: i32, x2: i32, y2: i32) -> i32 {
    (x2 - x0) * (y1 - y0) - (y2 - y0) * (x1 - x0)
}

fn compute_lighting(normal: Vec3) -> f32 {
    let light_n = LIGHT_DIR.normalize().neg();
    let ndotl = normal.dot(light_n);
    clamp(AMBIENT + ndotl.max(0.0) * (1.0 - AMBIENT), AMBIENT, 1.0)
}
