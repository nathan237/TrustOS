//! TrustMario64 — Player (Mario) State Machine
//! Action-based system faithfully adapted from SM64 decompilation (n64decomp/sm64)
#![allow(dead_code)]

use super::physics::*;
use super::collision::*;
use super::animation::{AnimId, AnimState, AnimLibrary};
use super::tas::{FrameInput, BTN_A, BTN_B, BTN_Z};
use crate::math::{fast_sin, fast_cos, fast_sqrt, fast_atan2};

// ======================== SM64 Actions ========================

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Action {
    Idle,
    Walking,
    Running,
    Turning,
    Crouching,
    CrouchSlide,
    Jumping,
    DoubleJump,
    TripleJump,
    LongJump,
    Backflip,
    SideFlip,
    WallKick,
    Freefall,
    GroundPound,
    GroundPoundLand,
    Dive,
    Swimming,
    WaterIdle,
    LedgeGrab,
    Damaged,
    DeathFall,
    Victory,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MarioFlags {
    OnGround,
    InAir,
    InWater,
    OnPole,
}

// ======================== Mario State ========================

pub struct MarioState {
    pub pos: Vec3,
    pub vel: Vec3,
    pub facing_yaw: f32,
    pub intended_yaw: f32,
    pub intended_mag: f32,

    pub action: Action,
    pub prev_action: Action,
    pub action_timer: u32,
    pub action_arg: u32,

    pub flags: MarioFlags,
    pub forward_vel: f32,
    pub peak_height: f32,

    // SM64 health system: 8 wedge power meter
    pub hp: u8,
    pub max_hp: u8,
    pub coins: u16,
    pub stars: u8,
    pub lives: u8,

    // Collision results (per-frame)
    pub floor_height: f32,
    pub floor_normal: Vec3,
    pub floor_type: SurfaceType,
    pub wall_normal: Vec3,
    pub wall_pushed: bool,

    // Jump tracking (for double/triple)
    pub jump_count: u8,
    pub double_jump_timer: u32,
    pub invincible_timer: u32,

    pub anim: AnimState,
    pub alive: bool,
}

impl MarioState {
    pub fn new(spawn: Vec3) -> Self {
        Self {
            pos: spawn,
            vel: Vec3::ZERO,
            facing_yaw: 0.0,
            intended_yaw: 0.0,
            intended_mag: 0.0,
            action: Action::Freefall,
            prev_action: Action::Idle,
            action_timer: 0,
            action_arg: 0,
            flags: MarioFlags::InAir,
            forward_vel: 0.0,
            peak_height: spawn.y,
            hp: 8,
            max_hp: 8,
            coins: 0,
            stars: 0,
            lives: 4,
            floor_height: 0.0,
            floor_normal: Vec3::UP,
            floor_type: SurfaceType::Default,
            wall_normal: Vec3::ZERO,
            wall_pushed: false,
            jump_count: 0,
            double_jump_timer: 0,
            invincible_timer: 0,
            anim: AnimState::new(),
            alive: true,
        }
    }

    pub fn set_action(&mut self, action: Action) {
        self.prev_action = self.action;
        self.action = action;
        self.action_timer = 0;
    }

    /// Convert keyboard/stick input into intended direction relative to camera
    pub fn process_input(&mut self, input: &FrameInput, cam_yaw: f32) {
        let sx = input.stick_x as f32 / 127.0;
        let sy = input.stick_y as f32 / 127.0;
        self.intended_mag = fast_sqrt(sx * sx + sy * sy).min(1.0);

        if self.intended_mag > 0.1 {
            self.intended_yaw = fast_atan2(sx, -sy) + cam_yaw;
            self.intended_yaw = wrap_angle(self.intended_yaw);
        }
    }

    /// Main update — called once per game frame
    pub fn update(
        &mut self, input: &FrameInput, mesh: &CollisionMesh,
        cam_yaw: f32, anim_lib: &AnimLibrary,
    ) {
        if !self.alive { return; }

        self.process_input(input, cam_yaw);
        self.action_timer += 1;
        if self.double_jump_timer > 0 { self.double_jump_timer -= 1; }
        if self.invincible_timer > 0 { self.invincible_timer -= 1; }

        // Sample floor below Mario
        let floor = find_floor(self.pos, mesh);
        if floor.found {
            self.floor_height = floor.height;
            self.floor_normal = floor.normal;
            self.floor_type = floor.surface_type;
        }

        // Dispatch to action handler
        match self.action {
            Action::Idle => self.act_idle(input),
            Action::Walking => self.act_walking(input),
            Action::Running => self.act_running(input),
            Action::Turning => self.act_turning(input),
            Action::Crouching => self.act_crouching(input),
            Action::CrouchSlide => self.act_crouch_slide(input),
            Action::Jumping => self.act_jumping(input),
            Action::DoubleJump => self.act_double_jump(input),
            Action::TripleJump => self.act_triple_jump(input),
            Action::LongJump => self.act_long_jump(input),
            Action::Backflip => self.act_backflip(input),
            Action::SideFlip => self.act_side_flip(input),
            Action::WallKick => self.act_wall_kick(input),
            Action::Freefall => self.act_freefall(input),
            Action::GroundPound => self.act_ground_pound(input),
            Action::GroundPoundLand => self.act_ground_pound_land(input),
            Action::Dive => self.act_dive(input),
            Action::Swimming | Action::WaterIdle => self.act_swimming(input),
            Action::LedgeGrab => self.act_ledge_grab(input),
            Action::Damaged => self.act_damaged(input),
            Action::DeathFall => self.act_death_fall(),
            Action::Victory => self.act_victory(),
        }

        // Move and collide
        self.apply_movement(mesh);

        // Death plane
        if self.pos.y < -20.0 {
            self.take_damage(8);
        }

        // Peak height (fall damage tracking)
        if self.pos.y > self.peak_height {
            self.peak_height = self.pos.y;
        }

        // Tick animation
        self.anim.update(1.0 / 30.0, anim_lib);
    }

    // ==================== Physics integration ====================

    fn apply_movement(&mut self, mesh: &CollisionMesh) {
        let new_pos = self.pos.add(self.vel.scale(1.0 / 30.0));

        // Wall collision
        let wall = check_wall_collision(self.pos, new_pos, MARIO_RADIUS, mesh);
        self.wall_pushed = wall.pushed;
        self.wall_normal = wall.wall_normal;
        self.pos = wall.new_pos;

        // Floor snap (on ground)
        if self.flags == MarioFlags::OnGround {
            let floor = find_floor(self.pos, mesh);
            if floor.found && (self.pos.y - floor.height).abs() < 1.0 {
                self.pos.y = floor.height;
                self.vel.y = 0.0;
            }
        }

        // Landing (in air)
        if self.flags == MarioFlags::InAir {
            let floor = find_floor(self.pos, mesh);
            if floor.found && self.pos.y <= floor.height + 0.05 && self.vel.y <= 0.0 {
                self.pos.y = floor.height;
                self.land();
            }
        }

        // Water transition
        if self.floor_type == SurfaceType::Water && self.pos.y < self.floor_height + 0.5 {
            if self.flags != MarioFlags::InWater {
                self.flags = MarioFlags::InWater;
                self.set_action(Action::Swimming);
            }
        }
    }

    fn land(&mut self) {
        self.flags = MarioFlags::OnGround;
        self.vel.y = 0.0;

        // Fall damage (SM64: if fall > 25 units from peak)
        let fall_dist = self.peak_height - self.pos.y;
        if fall_dist > 25.0 {
            self.take_damage(1);
        }

        // Special landing transitions
        match self.action {
            Action::GroundPound => {
                self.set_action(Action::GroundPoundLand);
                self.anim.play(AnimId::GroundPoundLand, 1.0);
                return;
            }
            Action::Dive => {
                self.forward_vel *= 0.5;
                self.set_action(Action::Idle);
                self.anim.play(AnimId::Land, 1.0);
                return;
            }
            _ => {}
        }

        self.peak_height = self.pos.y;

        if self.forward_vel.abs() > MAX_WALK_SPEED {
            self.set_action(Action::Running);
            self.anim.play(AnimId::Run, 1.0);
        } else if self.forward_vel.abs() > 0.5 {
            self.set_action(Action::Walking);
            self.anim.play(AnimId::Walk, 1.0);
        } else {
            self.set_action(Action::Idle);
            self.anim.play(AnimId::Land, 1.0);
        }

        self.double_jump_timer = 10;
    }

    // ==================== Ground Actions (SM64) ====================

    fn act_idle(&mut self, input: &FrameInput) {
        self.forward_vel = approach_f32(self.forward_vel, 0.0, GROUND_DECEL, GROUND_DECEL);
        self.apply_ground_vel();
        self.anim.play(AnimId::Idle, 1.0);

        if input.buttons & BTN_A != 0 {
            self.begin_jump();
        } else if input.buttons & BTN_Z != 0 {
            self.set_action(Action::Crouching);
            self.anim.play(AnimId::Crouch, 1.0);
        } else if self.intended_mag > 0.1 {
            self.set_action(Action::Walking);
        }
    }

    fn act_walking(&mut self, input: &FrameInput) {
        let diff = angle_diff(self.facing_yaw, self.intended_yaw);
        if diff.abs() > 0.1 {
            self.facing_yaw += clamp(diff, -TURN_SPEED_GROUND, TURN_SPEED_GROUND);
            self.facing_yaw = wrap_angle(self.facing_yaw);
        }

        let target = MAX_WALK_SPEED * self.intended_mag;
        self.forward_vel = approach_f32(self.forward_vel, target, GROUND_ACCEL, GROUND_DECEL);
        self.apply_ground_vel();
        self.anim.play(AnimId::Walk, self.intended_mag.max(0.3));

        if input.buttons & BTN_A != 0 {
            self.begin_jump();
        } else if input.buttons & BTN_Z != 0 {
            if self.forward_vel > 3.0 {
                self.set_action(Action::CrouchSlide);
            } else {
                self.set_action(Action::Crouching);
                self.anim.play(AnimId::Crouch, 1.0);
            }
        } else if self.intended_mag < 0.05 {
            self.set_action(Action::Idle);
        } else if self.forward_vel > MAX_WALK_SPEED * 0.8 {
            self.set_action(Action::Running);
        }
    }

    fn act_running(&mut self, input: &FrameInput) {
        let diff = angle_diff(self.facing_yaw, self.intended_yaw);

        // SM64: turning brake at >135°
        if diff.abs() > 2.3 && self.forward_vel > 4.0 {
            self.set_action(Action::Turning);
            return;
        }

        if diff.abs() > 0.05 {
            self.facing_yaw += clamp(diff, -TURN_SPEED_GROUND, TURN_SPEED_GROUND);
            self.facing_yaw = wrap_angle(self.facing_yaw);
        }

        let target = MAX_RUN_SPEED * self.intended_mag;
        self.forward_vel = approach_f32(self.forward_vel, target, GROUND_ACCEL, GROUND_DECEL);
        self.apply_ground_vel();
        self.anim.play(AnimId::Run, (self.forward_vel / MAX_RUN_SPEED).max(0.5));

        if input.buttons & BTN_A != 0 {
            self.begin_jump();
        } else if input.buttons & BTN_B != 0 {
            self.begin_dive();
        } else if input.buttons & BTN_Z != 0 {
            self.begin_long_jump();
        } else if self.intended_mag < 0.05 {
            self.set_action(Action::Idle);
        } else if self.forward_vel < MAX_WALK_SPEED * 0.5 {
            self.set_action(Action::Walking);
        }
    }

    fn act_turning(&mut self, input: &FrameInput) {
        self.forward_vel = approach_f32(self.forward_vel, 0.0, 1.5, 1.5);
        self.apply_ground_vel();

        let diff = angle_diff(self.facing_yaw, self.intended_yaw);
        self.facing_yaw += clamp(diff, -TURN_SPEED_GROUND * 3.0, TURN_SPEED_GROUND * 3.0);
        self.facing_yaw = wrap_angle(self.facing_yaw);

        if diff.abs() < 0.3 || self.action_timer > 15 {
            if input.buttons & BTN_A != 0 {
                // Side flip!
                self.set_action(Action::SideFlip);
                self.vel.y = SIDE_FLIP_VEL;
                self.flags = MarioFlags::InAir;
                self.peak_height = self.pos.y;
                self.anim.play(AnimId::DoubleJump, 1.5);
                return;
            }
            self.set_action(Action::Walking);
        }
    }

    fn act_crouching(&mut self, input: &FrameInput) {
        self.forward_vel = approach_f32(self.forward_vel, 0.0, GROUND_DECEL, GROUND_DECEL);
        self.apply_ground_vel();
        self.anim.play(AnimId::Crouch, 1.0);

        if input.buttons & BTN_Z == 0 {
            self.set_action(Action::Idle);
        } else if input.buttons & BTN_A != 0 {
            // Backflip from crouch! (iconic SM64 move)
            self.set_action(Action::Backflip);
            self.vel.y = BACKFLIP_VEL;
            self.forward_vel = -4.0;
            self.flags = MarioFlags::InAir;
            self.peak_height = self.pos.y;
            self.anim.play(AnimId::Backflip, 1.0);
        }
    }

    fn act_crouch_slide(&mut self, input: &FrameInput) {
        self.forward_vel = approach_f32(self.forward_vel, 0.0, 0.3, 0.3);
        self.apply_ground_vel();

        if self.forward_vel < 0.5 {
            self.set_action(Action::Crouching);
            self.anim.play(AnimId::Crouch, 1.0);
        } else if input.buttons & BTN_A != 0 {
            self.begin_long_jump();
        }
    }

    // ==================== Air Actions (SM64) ====================

    fn act_jumping(&mut self, input: &FrameInput) {
        self.apply_gravity();
        self.update_air_movement(input);
        self.anim.play(AnimId::Jump, 1.0);
        self.check_air_actions(input);
    }

    fn act_double_jump(&mut self, input: &FrameInput) {
        self.apply_gravity();
        self.update_air_movement(input);
        self.anim.play(AnimId::DoubleJump, 1.0);
        self.check_air_actions(input);
    }

    fn act_triple_jump(&mut self, input: &FrameInput) {
        self.apply_gravity();
        self.update_air_movement(input);
        self.anim.play(AnimId::DoubleJump, 1.5);
        self.check_air_actions(input);
    }

    fn act_freefall(&mut self, input: &FrameInput) {
        self.apply_gravity();
        self.update_air_movement(input);
        self.anim.play(AnimId::Fall, 1.0);
        self.check_air_actions(input);
    }

    fn act_long_jump(&mut self, input: &FrameInput) {
        self.apply_gravity();
        // Limited air control during long jump
        let fwd = angle_to_forward(self.facing_yaw);
        self.vel.x = fwd.x * self.forward_vel;
        self.vel.z = fwd.z * self.forward_vel;
        self.anim.play(AnimId::LongJump, 1.0);
    }

    fn act_backflip(&mut self, input: &FrameInput) {
        self.apply_gravity();
        let fwd = angle_to_forward(self.facing_yaw);
        self.vel.x = fwd.x * self.forward_vel;
        self.vel.z = fwd.z * self.forward_vel;
        self.anim.play(AnimId::Backflip, 1.0);
    }

    fn act_side_flip(&mut self, input: &FrameInput) {
        self.apply_gravity();
        self.update_air_movement(input);
    }

    fn act_wall_kick(&mut self, input: &FrameInput) {
        self.apply_gravity();
        self.update_air_movement(input);
        self.anim.play(AnimId::Jump, 1.0);
        self.check_air_actions(input);
    }

    fn act_ground_pound(&mut self, input: &FrameInput) {
        if self.action_timer < 5 {
            // SM64's ground pound hover pause
            self.vel = Vec3::ZERO;
        } else {
            self.vel = Vec3::new(0.0, GROUND_POUND_VEL, 0.0);
        }
        self.anim.play(AnimId::GroundPound, 1.0);
    }

    fn act_ground_pound_land(&mut self, input: &FrameInput) {
        self.forward_vel = 0.0;
        self.vel = Vec3::ZERO;
        self.anim.play(AnimId::GroundPoundLand, 1.0);

        if self.action_timer > 10 {
            if input.buttons & BTN_A != 0 {
                self.begin_special_jump(TRIPLE_JUMP_VEL, Action::TripleJump);
            } else {
                self.set_action(Action::Idle);
            }
        }
    }

    fn act_dive(&mut self, input: &FrameInput) {
        self.apply_gravity();
        let fwd = angle_to_forward(self.facing_yaw);
        self.vel.x = fwd.x * self.forward_vel;
        self.vel.z = fwd.z * self.forward_vel;
        self.anim.play(AnimId::LongJump, 1.2);
    }

    fn act_swimming(&mut self, input: &FrameInput) {
        self.vel.y += SWIM_GRAVITY;
        if input.buttons & BTN_A != 0 {
            self.vel.y = SWIM_SPEED;
        }
        let spd = SWIM_SPEED * self.intended_mag * 0.5;
        let fwd = angle_to_forward(self.intended_yaw);
        self.vel.x = fwd.x * spd;
        self.vel.z = fwd.z * spd;

        if self.intended_mag > 0.1 {
            self.facing_yaw = lerp_angle(self.facing_yaw, self.intended_yaw, 0.1);
        }
        self.anim.play(AnimId::Swim, 1.0);

        // Exit water
        if self.pos.y > self.floor_height + 1.5 {
            self.flags = MarioFlags::InAir;
            self.vel.y = JUMP_VEL * 0.8;
            self.set_action(Action::Freefall);
        }
    }

    fn act_ledge_grab(&mut self, input: &FrameInput) {
        self.vel = Vec3::ZERO;
        if input.buttons & BTN_A != 0 {
            self.vel.y = JUMP_VEL * 0.6;
            self.flags = MarioFlags::InAir;
            self.set_action(Action::Jumping);
            self.anim.play(AnimId::Jump, 1.0);
        } else if self.action_timer > 60 {
            self.set_action(Action::Freefall);
            self.flags = MarioFlags::InAir;
        }
    }

    fn act_damaged(&mut self, input: &FrameInput) {
        self.apply_gravity();
        self.forward_vel = approach_f32(self.forward_vel, 0.0, 0.5, 0.5);
        let fwd = angle_to_forward(self.facing_yaw);
        self.vel.x = fwd.x * self.forward_vel;
        self.vel.z = fwd.z * self.forward_vel;
        self.anim.play(AnimId::Damaged, 1.0);

        if self.flags == MarioFlags::OnGround && self.action_timer > 15 {
            self.set_action(Action::Idle);
        }
    }

    fn act_death_fall(&mut self) {
        self.apply_gravity();
        self.anim.play(AnimId::Damaged, 0.5);
    }

    fn act_victory(&mut self) {
        self.vel = Vec3::ZERO;
        self.anim.play(AnimId::Victory, 1.0);
    }

    // ==================== Jump + Air Helpers ====================

    fn check_air_actions(&mut self, input: &FrameInput) {
        // Wall kick
        if self.wall_pushed && input.buttons & BTN_A != 0 {
            self.set_action(Action::WallKick);
            self.vel.y = WALL_KICK_VEL;
            self.facing_yaw = forward_to_angle(self.wall_normal);
            self.forward_vel = 4.0;
            self.apply_ground_vel();
            self.peak_height = self.pos.y;
            self.anim.play(AnimId::Jump, 1.0);
            return;
        }

        // Ground pound
        if input.buttons & BTN_Z != 0 && self.action != Action::GroundPound {
            self.set_action(Action::GroundPound);
            self.forward_vel = 0.0;
            self.vel.x = 0.0;
            self.vel.z = 0.0;
            self.anim.play(AnimId::GroundPound, 1.0);
        }

        // Dive
        if input.buttons & BTN_B != 0 && self.action != Action::Dive {
            self.begin_dive();
        }
    }

    fn begin_jump(&mut self) {
        self.flags = MarioFlags::InAir;
        self.peak_height = self.pos.y;

        if self.double_jump_timer > 0 {
            self.jump_count += 1;
            if self.jump_count >= 3 && self.forward_vel > 3.0 {
                self.begin_special_jump(TRIPLE_JUMP_VEL, Action::TripleJump);
                return;
            } else if self.jump_count >= 2 {
                self.begin_special_jump(DOUBLE_JUMP_VEL, Action::DoubleJump);
                return;
            }
        }

        self.jump_count = 1;
        self.set_action(Action::Jumping);
        self.vel.y = JUMP_VEL;
        self.anim.play(AnimId::Jump, 1.0);
    }

    fn begin_special_jump(&mut self, vel_y: f32, action: Action) {
        self.flags = MarioFlags::InAir;
        self.peak_height = self.pos.y;
        self.set_action(action);
        self.vel.y = vel_y;
        self.anim.play(AnimId::DoubleJump, 1.0);
    }

    fn begin_long_jump(&mut self) {
        self.flags = MarioFlags::InAir;
        self.peak_height = self.pos.y;
        self.set_action(Action::LongJump);
        self.vel.y = LONG_JUMP_VEL_Y;
        self.forward_vel = self.forward_vel.max(LONG_JUMP_SPEED);
        let fwd = angle_to_forward(self.facing_yaw);
        self.vel.x = fwd.x * self.forward_vel;
        self.vel.z = fwd.z * self.forward_vel;
        self.anim.play(AnimId::LongJump, 1.0);
    }

    fn begin_dive(&mut self) {
        self.flags = MarioFlags::InAir;
        self.set_action(Action::Dive);
        self.vel.y = 3.0;
        self.forward_vel = self.forward_vel.max(8.0);
        let fwd = angle_to_forward(self.facing_yaw);
        self.vel.x = fwd.x * self.forward_vel;
        self.vel.z = fwd.z * self.forward_vel;
    }

    fn apply_gravity(&mut self) {
        self.vel.y += GRAVITY;
        if self.vel.y < TERMINAL_VEL {
            self.vel.y = TERMINAL_VEL;
        }
    }

    /// SM64's update_air_with_turn — stick controls facing + forward speed in air
    fn update_air_movement(&mut self, _input: &FrameInput) {
        if self.intended_mag > 0.1 {
            let diff = angle_diff(self.facing_yaw, self.intended_yaw);
            self.facing_yaw += clamp(diff, -TURN_SPEED_AIR, TURN_SPEED_AIR);
            self.facing_yaw = wrap_angle(self.facing_yaw);

            let accel = AIR_ACCEL * self.intended_mag;
            self.forward_vel = approach_f32(
                self.forward_vel,
                MAX_RUN_SPEED * self.intended_mag,
                accel, AIR_DRAG,
            );
        } else {
            self.forward_vel *= 0.99;
        }

        let fwd = angle_to_forward(self.facing_yaw);
        self.vel.x = fwd.x * self.forward_vel;
        self.vel.z = fwd.z * self.forward_vel;
    }

    fn apply_ground_vel(&mut self) {
        let fwd = angle_to_forward(self.facing_yaw);
        self.vel.x = fwd.x * self.forward_vel;
        self.vel.y = 0.0;
        self.vel.z = fwd.z * self.forward_vel;
    }

    // ==================== Health ====================

    pub fn take_damage(&mut self, amount: u8) {
        if self.invincible_timer > 0 { return; }
        self.hp = self.hp.saturating_sub(amount);
        self.invincible_timer = 60;

        if self.hp == 0 {
            self.alive = false;
            self.set_action(Action::DeathFall);
            self.vel = Vec3::new(0.0, 8.0, 0.0);
            return;
        }

        self.set_action(Action::Damaged);
        self.vel.y = DAMAGE_KNOCKBACK;
        self.forward_vel = -3.0;
        self.flags = MarioFlags::InAir;
        self.anim.play(AnimId::Damaged, 1.0);
    }

    pub fn heal(&mut self, amount: u8) {
        self.hp = (self.hp + amount).min(self.max_hp);
    }

    pub fn collect_coin(&mut self) {
        self.coins += 1;
        if self.coins % 50 == 0 { self.lives += 1; }
        if self.coins % 10 == 0 { self.heal(1); }
    }

    pub fn is_on_ground(&self) -> bool { self.flags == MarioFlags::OnGround }
    pub fn is_in_air(&self) -> bool { self.flags == MarioFlags::InAir }
}
