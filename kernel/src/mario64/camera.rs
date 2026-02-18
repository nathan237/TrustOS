//! TrustMario64 — Camera System (Lakitu)
//! SM64-style dynamic camera with orbit, follow, and debug modes
#![allow(dead_code)]

use super::physics::*;
use crate::math::{fast_sin, fast_cos};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CameraMode {
    Behind,  // SM64 default: orbits behind Mario, follows facing
    Lakitu,  // free-orbit, user-controlled yaw/pitch
    Fixed,   // fixed angle for specific areas
    Free,    // debug free-fly camera
}

pub struct Camera {
    pub pos: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub mode: CameraMode,

    pub yaw: f32,
    pub pitch: f32,
    pub dist: f32,
    pub fov: f32,

    // Smooth interpolation state
    smooth_pos: Vec3,
    smooth_target: Vec3,
    pub lerp_speed: f32,

    // Limits
    pub min_pitch: f32,
    pub max_pitch: f32,
    pub min_dist: f32,
    pub max_dist: f32,

    // Free cam
    pub free_pos: Vec3,
    pub free_yaw: f32,
    pub free_pitch: f32,
    pub free_speed: f32,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            pos: Vec3::new(0.0, 5.0, 10.0),
            target: Vec3::ZERO,
            up: Vec3::UP,
            mode: CameraMode::Behind,
            yaw: 0.0,
            pitch: 0.3,
            dist: 10.0,
            fov: 45.0 * DEG_TO_RAD,
            smooth_pos: Vec3::new(0.0, 5.0, 10.0),
            smooth_target: Vec3::ZERO,
            lerp_speed: 0.08,
            min_pitch: -0.2,
            max_pitch: 1.2,
            min_dist: 3.0,
            max_dist: 25.0,
            free_pos: Vec3::new(0.0, 10.0, 20.0),
            free_yaw: 0.0,
            free_pitch: 0.3,
            free_speed: 0.5,
        }
    }

    pub fn update(&mut self, mario_pos: Vec3, mario_facing: f32, _dt: f32) {
        match self.mode {
            CameraMode::Behind => self.update_behind(mario_pos, mario_facing),
            CameraMode::Lakitu => self.update_lakitu(mario_pos),
            CameraMode::Fixed => self.update_fixed(mario_pos),
            CameraMode::Free => {}, // updated by input only
        }
    }

    fn update_behind(&mut self, mario_pos: Vec3, mario_facing: f32) {
        // SM64: camera orbits behind Mario with smooth follow
        let target_yaw = mario_facing + PI;
        self.yaw = lerp_angle(self.yaw, target_yaw, self.lerp_speed);
        self.pitch = clamp(self.pitch, self.min_pitch, self.max_pitch);

        let look_at_point = mario_pos.add(Vec3::new(0.0, MARIO_HEIGHT * 0.7, 0.0));
        let offset = self.get_orbit_offset();
        let ideal_pos = look_at_point.add(offset);

        self.smooth_target = self.smooth_target.lerp(look_at_point, self.lerp_speed);
        self.smooth_pos = self.smooth_pos.lerp(ideal_pos, self.lerp_speed);
        self.pos = self.smooth_pos;
        self.target = self.smooth_target;
    }

    fn update_lakitu(&mut self, mario_pos: Vec3) {
        self.pitch = clamp(self.pitch, self.min_pitch, self.max_pitch);
        let look_at_point = mario_pos.add(Vec3::new(0.0, MARIO_HEIGHT * 0.7, 0.0));
        let offset = self.get_orbit_offset();
        let ideal_pos = look_at_point.add(offset);

        self.smooth_target = self.smooth_target.lerp(look_at_point, 0.1);
        self.smooth_pos = self.smooth_pos.lerp(ideal_pos, 0.1);
        self.pos = self.smooth_pos;
        self.target = self.smooth_target;
    }

    fn update_fixed(&mut self, mario_pos: Vec3) {
        self.target = mario_pos.add(Vec3::new(0.0, MARIO_HEIGHT * 0.5, 0.0));
    }

    fn get_orbit_offset(&self) -> Vec3 {
        Vec3::new(
            fast_sin(self.yaw) * fast_cos(self.pitch) * self.dist,
            fast_sin(self.pitch) * self.dist,
            -fast_cos(self.yaw) * fast_cos(self.pitch) * self.dist,
        )
    }

    /// Rotate camera by delta yaw/pitch
    pub fn rotate(&mut self, dyaw: f32, dpitch: f32) {
        self.yaw += dyaw;
        self.pitch = clamp(self.pitch + dpitch, self.min_pitch, self.max_pitch);
    }

    /// Zoom camera in/out
    pub fn zoom(&mut self, amount: f32) {
        self.dist = clamp(self.dist + amount, self.min_dist, self.max_dist);
    }

    /// Move free camera
    pub fn free_move(&mut self, forward: f32, right: f32, up: f32) {
        let fwd = Vec3::new(
            fast_sin(self.free_yaw) * fast_cos(self.free_pitch),
            fast_sin(self.free_pitch),
            -fast_cos(self.free_yaw) * fast_cos(self.free_pitch),
        );
        let rt = Vec3::new(fast_cos(self.free_yaw), 0.0, fast_sin(self.free_yaw));
        self.free_pos = self.free_pos
            .add(fwd.scale(forward * self.free_speed))
            .add(rt.scale(right * self.free_speed))
            .add(Vec3::UP.scale(up * self.free_speed));
        self.pos = self.free_pos;
        self.target = self.free_pos.add(fwd);
    }

    /// Cycle camera mode
    pub fn cycle_mode(&mut self) {
        self.mode = match self.mode {
            CameraMode::Behind => CameraMode::Lakitu,
            CameraMode::Lakitu => CameraMode::Fixed,
            CameraMode::Fixed => CameraMode::Free,
            CameraMode::Free => CameraMode::Behind,
        };
    }

    pub fn get_view_matrix(&self) -> Mat4 {
        Mat4::look_at(self.pos, self.target, self.up)
    }

    pub fn get_projection_matrix(&self, aspect: f32) -> Mat4 {
        Mat4::perspective(self.fov, aspect, 0.1, 200.0)
    }

    pub fn get_yaw(&self) -> f32 {
        match self.mode {
            CameraMode::Free => self.free_yaw,
            _ => self.yaw,
        }
    }
}
