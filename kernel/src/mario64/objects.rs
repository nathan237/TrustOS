//! TrustMario64 — Game Objects
//! Coins, stars, blocks, trees, warps — the collectibles of Bob-omb Battlefield
#![allow(dead_code)]

use alloc::vec::Vec;
use super::physics::*;
use crate::math::{fast_sin, fast_cos};

// ======================== Object Types ========================

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ObjectType {
    Coin,
    RedCoin,
    BlueCoin,
    Star,
    YellowBlock,
    Tree,
    Warp,
    BreakableBox,
    OneUp,        // 1-Up Mushroom
    HeartSpinner, // health restore
}

// ======================== Game Object ========================

#[derive(Clone, Debug)]
pub struct GameObject {
    pub obj_type: ObjectType,
    pub pos: Vec3,
    pub spawn_pos: Vec3,
    pub rotation: f32,     // Y-axis spin
    pub bob_offset: f32,   // vertical bobbing
    pub collected: bool,
    pub visible: bool,
    pub hitbox_radius: f32,
    pub hitbox_height: f32,
    pub respawn_timer: u32,
    pub anim_time: f32,
}

impl GameObject {
    pub fn new(obj_type: ObjectType, pos: Vec3) -> Self {
        let (radius, height) = match obj_type {
            ObjectType::Coin | ObjectType::RedCoin | ObjectType::BlueCoin => (0.5, 0.5),
            ObjectType::Star => (1.0, 1.0),
            ObjectType::YellowBlock | ObjectType::BreakableBox => (1.0, 1.0),
            ObjectType::Tree => (0.8, 6.0),
            ObjectType::Warp => (1.5, 2.0),
            ObjectType::OneUp | ObjectType::HeartSpinner => (0.5, 0.5),
        };

        Self {
            obj_type,
            pos,
            spawn_pos: pos,
            rotation: 0.0,
            bob_offset: 0.0,
            collected: false,
            visible: true,
            hitbox_radius: radius,
            hitbox_height: height,
            respawn_timer: 0,
            anim_time: 0.0,
        }
    }

    /// Update object animation (rotation, bobbing, respawn)
    pub fn update(&mut self, frame: u64) {
        self.anim_time += 1.0 / 30.0;

        match self.obj_type {
            ObjectType::Coin | ObjectType::RedCoin | ObjectType::BlueCoin => {
                // Rotate and bob like SM64
                self.rotation += 0.08;
                self.bob_offset = fast_sin(self.anim_time * 2.0) * 0.2;
            }
            ObjectType::Star => {
                // Rotate slower, bigger bob
                self.rotation += 0.05;
                self.bob_offset = fast_sin(self.anim_time * 1.5) * 0.4;
            }
            ObjectType::HeartSpinner => {
                self.rotation += 0.1;
            }
            ObjectType::OneUp => {
                self.bob_offset = fast_sin(self.anim_time * 2.5) * 0.15;
            }
            _ => {}
        }

        // Respawn timer
        if self.collected && self.respawn_timer > 0 {
            self.respawn_timer -= 1;
            if self.respawn_timer == 0 {
                self.collected = false;
                self.visible = true;
                self.pos = self.spawn_pos;
            }
        }
    }

    /// Get the effective display position (with bob offset)
    pub fn display_pos(&self) -> Vec3 {
        Vec3::new(self.pos.x, self.pos.y + self.bob_offset, self.pos.z)
    }

    /// Check if Mario is close enough to collect this object
    pub fn check_collection(&self, mario_pos: Vec3, mario_radius: f32) -> bool {
        if self.collected || !self.visible { return false; }
        match self.obj_type {
            ObjectType::Tree => false, // trees aren't collectible
            _ => {
                let dist = mario_pos.dist_xz(self.pos);
                let y_overlap = mario_pos.y < self.pos.y + self.hitbox_height
                    && mario_pos.y + MARIO_HEIGHT > self.pos.y;
                dist < self.hitbox_radius + mario_radius && y_overlap
            }
        }
    }

    /// Collect the object, returning the type
    pub fn collect(&mut self) -> ObjectType {
        self.collected = true;
        self.visible = false;

        // Set respawn timer (coins respawn, stars don't)
        match self.obj_type {
            ObjectType::Coin | ObjectType::RedCoin | ObjectType::BlueCoin => {
                self.respawn_timer = 300; // 10 seconds
            }
            ObjectType::OneUp | ObjectType::HeartSpinner => {
                self.respawn_timer = 600; // 20 seconds
            }
            _ => {
                self.respawn_timer = 0; // permanent collection
            }
        }

        self.obj_type
    }

    /// Is this a solid object (blocks Mario's movement)?
    pub fn is_solid(&self) -> bool {
        matches!(self.obj_type, ObjectType::Tree | ObjectType::YellowBlock | ObjectType::BreakableBox)
    }

    /// Break a yellow block (when punched/ground pounded)
    pub fn try_break(&mut self, from_above: bool) -> bool {
        match self.obj_type {
            ObjectType::YellowBlock | ObjectType::BreakableBox => {
                if from_above || self.obj_type == ObjectType::BreakableBox {
                    self.collected = true;
                    self.visible = false;
                    self.respawn_timer = 600;
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    pub fn get_color(&self) -> u32 {
        match self.obj_type {
            ObjectType::Coin => COLOR_COIN,
            ObjectType::RedCoin => COLOR_HUD_RED,
            ObjectType::BlueCoin => 0xFF4488FF,
            ObjectType::Star => COLOR_STAR,
            ObjectType::YellowBlock => 0xFFDDAA00,
            ObjectType::BreakableBox => COLOR_DIRT,
            ObjectType::Tree => 0xFF226611,
            ObjectType::Warp => 0xFF8800FF,
            ObjectType::OneUp => 0xFF00CC00,
            ObjectType::HeartSpinner => COLOR_HEALTH_FULL,
        }
    }
}
