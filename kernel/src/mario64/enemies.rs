//! TrustMario64 — Enemies
//! Goomba, Bob-omb, Chain Chomp — with SM64-faithful AI
#![allow(dead_code)]

use alloc::vec::Vec;
use super::physics::*;
use super::collision::*;
use crate::math::{fast_sin, fast_cos, fast_sqrt, fast_atan2};

// ======================== Enemy Types ========================

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EnemyType {
    Goomba,
    BobOmb,
    ChainChomp,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EnemyAI {
    Idle,
    Patrol,
    Chase,
    Flee,
    Stunned,
    Exploding,
    Dead,
}

// ======================== Enemy ========================

#[derive(Clone, Debug)]
pub struct Enemy {
    pub enemy_type: EnemyType,
    pub pos: Vec3,
    pub spawn_pos: Vec3,
    pub vel: Vec3,
    pub facing: f32,
    pub ai_state: EnemyAI,
    pub hp: u8,
    pub alive: bool,
    pub hitbox_radius: f32,
    pub hitbox_height: f32,

    // AI timers
    pub state_timer: u32,
    pub patrol_angle: f32,
    pub patrol_radius: f32,
    pub aggro_range: f32,
    pub deaggro_range: f32,

    // Chain chomp specific
    pub tether_pos: Vec3,
    pub tether_length: f32,
    pub lunge_timer: u32,

    // Bob-omb specific
    pub fuse_timer: u32,
    pub thrown: bool,

    // Respawn
    pub respawn_timer: u32,
    pub death_anim_timer: u32,
}

impl Enemy {
    pub fn new(enemy_type: EnemyType, pos: Vec3) -> Self {
        let (hp, radius, height, aggro, patrol_r) = match enemy_type {
            EnemyType::Goomba => (1, 0.5, 0.8, 15.0, 5.0),
            EnemyType::BobOmb => (1, 0.5, 0.9, 12.0, 8.0),
            EnemyType::ChainChomp => (3, 1.2, 1.5, 20.0, 0.0),
        };

        Self {
            enemy_type,
            pos,
            spawn_pos: pos,
            vel: Vec3::ZERO,
            facing: 0.0,
            ai_state: EnemyAI::Patrol,
            hp,
            alive: true,
            hitbox_radius: radius,
            hitbox_height: height,
            state_timer: 0,
            patrol_angle: 0.0,
            patrol_radius: patrol_r,
            aggro_range: aggro,
            deaggro_range: aggro * 1.5,
            tether_pos: pos,
            tether_length: 8.0,
            lunge_timer: 0,
            fuse_timer: 0,
            thrown: false,
            respawn_timer: 0,
            death_anim_timer: 0,
        }
    }

    /// Update enemy AI and physics
    pub fn update(&mut self, mario_pos: Vec3, mesh: &CollisionMesh) {
        if !self.alive {
            self.death_anim_timer += 1;
            if self.death_anim_timer > 90 { // 3 seconds
                self.respawn_timer += 1;
                if self.respawn_timer > 300 { // 10 seconds respawn
                    self.respawn();
                }
            }
            return;
        }

        self.state_timer += 1;
        let dist_to_mario = self.pos.dist_xz(mario_pos);

        match self.enemy_type {
            EnemyType::Goomba => self.update_goomba(mario_pos, dist_to_mario, mesh),
            EnemyType::BobOmb => self.update_bobomb(mario_pos, dist_to_mario, mesh),
            EnemyType::ChainChomp => self.update_chain_chomp(mario_pos, dist_to_mario),
        }

        // Apply gravity if not chain chomp
        if self.enemy_type != EnemyType::ChainChomp || self.ai_state == EnemyAI::Dead {
            self.vel.y += GRAVITY * 0.5;
            if self.vel.y < -10.0 { self.vel.y = -10.0; }
        }

        // Move
        self.pos = self.pos.add(self.vel.scale(1.0 / 30.0));

        // Floor snap
        let floor = find_floor(self.pos, mesh);
        if floor.found && self.pos.y <= floor.height + 0.05 {
            self.pos.y = floor.height;
            self.vel.y = 0.0;
        }
    }

    // ==================== Goomba AI ====================
    // SM64: walks in a line, turns around, charges at Mario when close

    fn update_goomba(&mut self, mario_pos: Vec3, dist: f32, _mesh: &CollisionMesh) {
        match self.ai_state {
            EnemyAI::Patrol => {
                // Walk in a circle around spawn
                self.patrol_angle += 0.02;
                let target_x = self.spawn_pos.x + fast_cos(self.patrol_angle) * self.patrol_radius;
                let target_z = self.spawn_pos.z + fast_sin(self.patrol_angle) * self.patrol_radius;
                let dir = Vec3::new(target_x - self.pos.x, 0.0, target_z - self.pos.z).normalize();
                self.vel.x = dir.x * 1.5;
                self.vel.z = dir.z * 1.5;
                self.facing = fast_atan2(dir.x, -dir.z);

                if dist < self.aggro_range {
                    self.ai_state = EnemyAI::Chase;
                    self.state_timer = 0;
                }
            }
            EnemyAI::Chase => {
                // Run toward Mario
                let dir = mario_pos.sub(self.pos);
                let dir_n = Vec3::new(dir.x, 0.0, dir.z).normalize();
                self.vel.x = dir_n.x * 3.0;
                self.vel.z = dir_n.z * 3.0;
                self.facing = fast_atan2(dir_n.x, -dir_n.z);

                if dist > self.deaggro_range {
                    self.ai_state = EnemyAI::Patrol;
                    self.state_timer = 0;
                }
            }
            EnemyAI::Stunned => {
                self.vel.x = 0.0;
                self.vel.z = 0.0;
                if self.state_timer > 60 {
                    self.ai_state = EnemyAI::Patrol;
                    self.state_timer = 0;
                }
            }
            _ => {}
        }
    }

    // ==================== Bob-omb AI ====================
    // SM64: patrols, chases, explodes after fuse

    fn update_bobomb(&mut self, mario_pos: Vec3, dist: f32, _mesh: &CollisionMesh) {
        match self.ai_state {
            EnemyAI::Patrol => {
                self.patrol_angle += 0.015;
                let target_x = self.spawn_pos.x + fast_cos(self.patrol_angle) * self.patrol_radius;
                let target_z = self.spawn_pos.z + fast_sin(self.patrol_angle) * self.patrol_radius;
                let dir = Vec3::new(target_x - self.pos.x, 0.0, target_z - self.pos.z).normalize();
                self.vel.x = dir.x * 1.2;
                self.vel.z = dir.z * 1.2;
                self.facing = fast_atan2(dir.x, -dir.z);

                if dist < self.aggro_range {
                    self.ai_state = EnemyAI::Chase;
                    self.fuse_timer = 120; // 4 second fuse
                    self.state_timer = 0;
                }
            }
            EnemyAI::Chase => {
                let dir = mario_pos.sub(self.pos);
                let dir_n = Vec3::new(dir.x, 0.0, dir.z).normalize();
                self.vel.x = dir_n.x * 2.5;
                self.vel.z = dir_n.z * 2.5;
                self.facing = fast_atan2(dir_n.x, -dir_n.z);

                self.fuse_timer = self.fuse_timer.saturating_sub(1);
                if self.fuse_timer == 0 {
                    self.ai_state = EnemyAI::Exploding;
                    self.state_timer = 0;
                } else if dist > self.deaggro_range {
                    self.ai_state = EnemyAI::Patrol;
                    self.fuse_timer = 0;
                }
            }
            EnemyAI::Exploding => {
                self.vel = Vec3::ZERO;
                if self.state_timer > 10 {
                    // Explosion! (damage Mario if close)
                    self.die();
                }
            }
            _ => {}
        }
    }

    // ==================== Chain Chomp AI ====================
    // SM64: tethered to stake, lunges at Mario periodically

    fn update_chain_chomp(&mut self, mario_pos: Vec3, dist: f32) {
        self.lunge_timer += 1;

        // Idle sway
        let sway_x = fast_sin(self.state_timer as f32 * 0.05) * 1.5;
        let sway_z = fast_cos(self.state_timer as f32 * 0.07) * 1.5;
        let base_pos = Vec3::new(
            self.tether_pos.x + sway_x,
            self.pos.y,
            self.tether_pos.z + sway_z,
        );

        if dist < self.aggro_range && self.lunge_timer > 90 {
            // LUNGE at Mario!
            let dir = mario_pos.sub(self.pos);
            let dir_n = Vec3::new(dir.x, 0.0, dir.z).normalize();
            self.vel.x = dir_n.x * 12.0; // fast lunge
            self.vel.z = dir_n.z * 12.0;
            self.vel.y = 3.0; // hop
            self.lunge_timer = 0;
            self.facing = fast_atan2(dir_n.x, -dir_n.z);
        } else {
            // Drift back toward base
            let to_base = base_pos.sub(self.pos);
            self.vel.x = to_base.x * 0.1;
            self.vel.z = to_base.z * 0.1;
        }

        // Tether constraint
        let dist_from_tether = self.pos.dist_xz(self.tether_pos);
        if dist_from_tether > self.tether_length {
            let pull = self.tether_pos.sub(self.pos);
            let pull_n = Vec3::new(pull.x, 0.0, pull.z).normalize();
            let excess = dist_from_tether - self.tether_length;
            self.pos.x += pull_n.x * excess;
            self.pos.z += pull_n.z * excess;
            // Bounce back effect
            self.vel.x = pull_n.x * 2.0;
            self.vel.z = pull_n.z * 2.0;
        }
    }

    // ==================== Interaction ====================

    /// Called when Mario stomps on this enemy
    pub fn stomp(&mut self) -> StompResult {
        match self.enemy_type {
            EnemyType::Goomba => {
                self.die();
                StompResult::Kill
            }
            EnemyType::BobOmb => {
                if self.ai_state == EnemyAI::Chase {
                    self.ai_state = EnemyAI::Stunned;
                    self.state_timer = 0;
                    self.vel = Vec3::ZERO;
                    StompResult::Stun
                } else {
                    self.die();
                    StompResult::Kill
                }
            }
            EnemyType::ChainChomp => {
                // Can't stomp chain chomp... well, you bounce
                StompResult::Bounce
            }
        }
    }

    /// Check if Mario is being hit by this enemy's attack
    pub fn is_attacking_mario(&self, mario_pos: Vec3, mario_radius: f32) -> bool {
        if !self.alive { return false; }

        let dist = self.pos.dist_xz(mario_pos);
        let y_overlap = mario_pos.y < self.pos.y + self.hitbox_height
            && mario_pos.y + MARIO_HEIGHT > self.pos.y;
        let in_range = dist < self.hitbox_radius + mario_radius;

        if !in_range || !y_overlap { return false; }

        match self.enemy_type {
            EnemyType::Goomba => self.ai_state == EnemyAI::Chase || self.ai_state == EnemyAI::Patrol,
            EnemyType::BobOmb => self.ai_state == EnemyAI::Exploding,
            EnemyType::ChainChomp => true,
        }
    }

    /// Check if Mario is stomping this enemy (above and coming down)
    pub fn can_be_stomped(&self, mario_pos: Vec3, mario_vel_y: f32, mario_radius: f32) -> bool {
        if !self.alive { return false; }
        if mario_vel_y > 0.0 { return false; } // must be falling

        let dist = self.pos.dist_xz(mario_pos);
        let above = mario_pos.y > self.pos.y + self.hitbox_height * 0.5;

        dist < self.hitbox_radius + mario_radius && above
    }

    /// Get explosion range (Bob-omb)
    pub fn explosion_range(&self) -> f32 {
        if self.enemy_type == EnemyType::BobOmb { 5.0 } else { 0.0 }
    }

    fn die(&mut self) {
        self.alive = false;
        self.ai_state = EnemyAI::Dead;
        self.vel = Vec3::new(0.0, 5.0, 0.0); // pop up
        self.death_anim_timer = 0;
        self.respawn_timer = 0;
    }

    fn respawn(&mut self) {
        self.alive = true;
        self.pos = self.spawn_pos;
        self.vel = Vec3::ZERO;
        self.ai_state = EnemyAI::Patrol;
        self.hp = match self.enemy_type {
            EnemyType::Goomba => 1,
            EnemyType::BobOmb => 1,
            EnemyType::ChainChomp => 3,
        };
        self.state_timer = 0;
        self.death_anim_timer = 0;
        self.respawn_timer = 0;
    }

    pub fn get_color(&self) -> u32 {
        match self.enemy_type {
            EnemyType::Goomba => COLOR_GOOMBA_BODY,
            EnemyType::BobOmb => COLOR_BOBOMB_BODY,
            EnemyType::ChainChomp => COLOR_CHAIN_BODY,
        }
    }
}

// ======================== Stomp Result ========================

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum StompResult {
    Kill,
    Stun,
    Bounce,
}
