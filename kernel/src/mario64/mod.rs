//! TrustMario64 — Main Module
//! SM64 clone with integrated TAS engine — runs inside TrustOS desktop
//! This is the game's entry point, tick loop, input handler, and desktop integration
#![allow(dead_code)]

pub mod physics;
pub mod collision;
pub mod animation;
pub mod tas;
pub mod camera;
pub mod player;
pub mod level;
pub mod objects;
pub mod enemies;
pub mod hud;
pub mod renderer;

use alloc::vec::Vec;
use physics::*;
use player::{MarioState, Action};
use camera::{Camera, CameraMode};
use level::LevelData;
use animation::AnimLibrary;
use tas::{TasEngine, TasMode, FrameInput, SaveState, BTN_A, BTN_B, BTN_Z, BTN_START, BTN_L};
use renderer::{RenderState, RENDER_W, RENDER_H};
use enemies::StompResult;

// ======================== Game State ========================

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GamePhase {
    TitleScreen,
    Playing,
    Paused,
    StarGet,
    GameOver,
}

pub struct Mario64Game {
    pub phase: GamePhase,
    pub mario: MarioState,
    pub camera: Camera,
    pub level: LevelData,
    pub anim_lib: AnimLibrary,
    pub tas: TasEngine,
    pub render: RenderState,

    // Input state (keyboard -> virtual N64 controller)
    key_w: bool, key_a: bool, key_s: bool, key_d: bool,
    key_space: bool, key_shift: bool, key_ctrl: bool,
    key_e: bool, key_q: bool, key_r: bool,
    key_tab: bool,
    mouse_dx: i16, mouse_dy: i16,

    // Game timing
    pub frame_count: u64,
    pub star_get_timer: u32,
    pub game_over_timer: u32,
    pub pause_selection: u8,

    // Red coin tracking
    pub red_coins_collected: u8,
}

impl Mario64Game {
    pub fn new() -> Self {
        let level = LevelData::bob_omb_battlefield();
        let spawn = level.spawn_pos;

        Self {
            phase: GamePhase::Playing,
            mario: MarioState::new(spawn),
            camera: Camera::new(),
            level,
            anim_lib: AnimLibrary::new(),
            tas: TasEngine::new(),
            render: RenderState::new(),
            key_w: false, key_a: false, key_s: false, key_d: false,
            key_space: false, key_shift: false, key_ctrl: false,
            key_e: false, key_q: false, key_r: false,
            key_tab: false,
            mouse_dx: 0, mouse_dy: 0,
            frame_count: 0,
            star_get_timer: 0,
            game_over_timer: 0,
            pause_selection: 0,
            red_coins_collected: 0,
        }
    }

    // ======================== Input ========================

    pub fn handle_key(&mut self, key: u8) {
        match key {
            b'w' | b'W' => self.key_w = true,
            b'a' | b'A' => self.key_a = true,
            b's' | b'S' => self.key_s = true,
            b'd' | b'D' => self.key_d = true,
            b' ' => self.key_space = true,
            b'e' | b'E' => self.key_e = true,
            b'q' | b'Q' => self.key_q = true,
            b'r' | b'R' => self.key_r = true,

            // TAS controls
            0x3B => self.tas_save_state(),           // F1: save
            0x3C => self.tas_load_state(),           // F2: load
            0x3D => self.tas.toggle_frame_advance(), // F3: frame advance
            0x3E => self.tas.step_frame(),           // F4: step
            0x3F => {                                // F5: record
                if self.tas.mode == TasMode::Recording {
                    self.tas.stop_recording();
                } else {
                    self.tas.start_recording();
                }
            }
            0x40 => self.tas.start_replay(),         // F6: replay
            0x41 => self.tas_rewind(),               // F7: rewind
            0x42 => {                                // F8: toggle ghost
                self.tas.ghost_active = !self.tas.ghost_active;
            }
            0x43 => {                                // F9: toggle TAS info
                self.tas.show_info_panel = !self.tas.show_info_panel;
            }
            0x44 => {                                // F10: toggle hitboxes
                self.tas.show_hitboxes = !self.tas.show_hitboxes;
            }

            // Camera mode
            b'c' | b'C' => self.camera.cycle_mode(),

            // Pause
            27 => { // ESC
                if self.phase == GamePhase::Playing {
                    self.phase = GamePhase::Paused;
                } else if self.phase == GamePhase::Paused {
                    self.phase = GamePhase::Playing;
                }
            }
            _ => {}
        }

        // Shift/Ctrl via scancode (these may come as special values)
        if key == 0x2A || key == 0x36 { self.key_shift = true; }
        if key == 0x1D { self.key_ctrl = true; }
        if key == 0x0F { self.key_tab = true; }
    }

    pub fn handle_key_release(&mut self, key: u8) {
        match key {
            b'w' | b'W' => self.key_w = false,
            b'a' | b'A' => self.key_a = false,
            b's' | b'S' => self.key_s = false,
            b'd' | b'D' => self.key_d = false,
            b' ' => self.key_space = false,
            b'e' | b'E' => self.key_e = false,
            b'q' | b'Q' => self.key_q = false,
            b'r' | b'R' => self.key_r = false,
            _ => {}
        }
        if key == 0x2A || key == 0x36 { self.key_shift = false; }
        if key == 0x1D { self.key_ctrl = false; }
        if key == 0x0F { self.key_tab = false; }
    }

    pub fn handle_mouse(&mut self, dx: i16, dy: i16) {
        self.mouse_dx += dx;
        self.mouse_dy += dy;
    }

    pub fn handle_scroll(&mut self, delta: i8) {
        self.camera.zoom(delta as f32 * -1.0);
    }

    // ======================== Keyboard -> N64 Controller mapping ========================

    fn build_input(&self) -> FrameInput {
        let mut buttons: u16 = 0;
        if self.key_space { buttons |= BTN_A; }
        if self.key_shift { buttons |= BTN_B; }
        if self.key_ctrl { buttons |= BTN_Z; }
        if self.key_tab { buttons |= BTN_START; }
        if self.key_q { buttons |= BTN_L; }

        // WASD -> analog stick
        let mut sx: i8 = 0;
        let mut sy: i8 = 0;
        if self.key_w { sy = 127; }
        if self.key_s { sy = -127; }
        if self.key_a { sx = -127; }
        if self.key_d { sx = 127; }
        // Diagonal normalization
        if sx != 0 && sy != 0 {
            sx = (sx as f32 * 0.707) as i8;
            sy = (sy as f32 * 0.707) as i8;
        }

        FrameInput {
            frame: self.frame_count,
            stick_x: sx,
            stick_y: sy,
            buttons,
            mouse_dx: self.mouse_dx,
            mouse_dy: self.mouse_dy,
        }
    }

    // ======================== Game Tick ========================

    pub fn tick(&mut self) {
        match self.phase {
            GamePhase::TitleScreen => { /* wait for input */ }
            GamePhase::Paused => { /* frozen */ }
            GamePhase::StarGet => self.tick_star_get(),
            GamePhase::GameOver => self.tick_game_over(),
            GamePhase::Playing => self.tick_playing(),
        }
    }

    fn tick_playing(&mut self) {
        if !self.tas.should_tick() { return; }

        // Build input from keyboard state
        let mut input = self.build_input();

        // TAS: override with replay input if replaying
        if let Some(replay_input) = self.tas.get_replay_input() {
            input = replay_input;
        }

        // Record input
        self.tas.record_input(input);

        // Camera mouse control (Lakitu mode)
        if self.mouse_dx != 0 || self.mouse_dy != 0 {
            self.camera.rotate(
                self.mouse_dx as f32 * -0.003,
                self.mouse_dy as f32 * 0.003,
            );
            self.mouse_dx = 0;
            self.mouse_dy = 0;
        }

        // E/Q for camera rotation (keyboard players)
        if self.key_e { self.camera.rotate(-0.04, 0.0); }
        if self.key_q { self.camera.rotate(0.04, 0.0); }

        // Update Mario
        let cam_yaw = self.camera.get_yaw();
        self.mario.update(&input, &self.level.collision, cam_yaw, &self.anim_lib);

        // Update camera
        self.camera.update(self.mario.pos, self.mario.facing_yaw, 1.0 / 30.0);

        // Update enemies
        for enemy in &mut self.level.enemies {
            enemy.update(self.mario.pos, &self.level.collision);
        }

        // Update objects
        for obj in &mut self.level.objects {
            obj.update(self.frame_count);
        }

        // Check object collection
        self.check_collections();

        // Check enemy interactions
        self.check_enemy_interactions();

        // Mario death check
        if !self.mario.alive {
            self.mario.lives = self.mario.lives.saturating_sub(1);
            if self.mario.lives == 0 {
                self.phase = GamePhase::GameOver;
                self.game_over_timer = 0;
            } else {
                // Respawn
                self.mario = MarioState::new(self.level.spawn_pos);
            }
        }

        // TAS: push to rewind buffer (every 2 frames)
        if self.frame_count % 2 == 0 {
            let state = self.create_save_state();
            self.tas.push_rewind(state);
        }

        // TAS: record ghost
        self.tas.record_ghost(
            self.mario.pos,
            self.mario.facing_yaw,
            self.mario.action as u8,
            self.mario.anim.time,
        );

        self.tas.advance_frame();
        self.frame_count += 1;
    }

    fn tick_star_get(&mut self) {
        self.star_get_timer += 1;
        self.mario.set_action(Action::Victory);
        self.mario.anim.update(1.0 / 30.0, &self.anim_lib);
        if self.star_get_timer > 120 { // 4 seconds celebration
            self.phase = GamePhase::Playing;
        }
    }

    fn tick_game_over(&mut self) {
        self.game_over_timer += 1;
    }

    // ======================== Collisions & Interactions ========================

    fn check_collections(&mut self) {
        for obj in &mut self.level.objects {
            if obj.check_collection(self.mario.pos, MARIO_RADIUS) {
                let obj_type = obj.collect();
                match obj_type {
                    objects::ObjectType::Coin | objects::ObjectType::BlueCoin => {
                        self.mario.collect_coin();
                    }
                    objects::ObjectType::RedCoin => {
                        self.mario.collect_coin();
                        self.mario.collect_coin(); // Red coins = 2 coins
                        self.red_coins_collected += 1;
                    }
                    objects::ObjectType::Star => {
                        self.mario.stars += 1;
                        self.phase = GamePhase::StarGet;
                        self.star_get_timer = 0;
                    }
                    objects::ObjectType::OneUp => {
                        self.mario.lives += 1;
                    }
                    objects::ObjectType::HeartSpinner => {
                        self.mario.heal(8);
                    }
                    _ => {}
                }
            }
        }

        // Check for solid object collisions (trees, blocks)
        for obj in &self.level.objects {
            if obj.is_solid() && !obj.collected {
                let dist = self.mario.pos.dist_xz(obj.pos);
                if dist < MARIO_RADIUS + obj.hitbox_radius {
                    self.mario.pos = collision::push_out_of_cylinder(
                        self.mario.pos, MARIO_RADIUS,
                        obj.pos, obj.hitbox_radius,
                    );
                }
            }
        }
    }

    fn check_enemy_interactions(&mut self) {
        for enemy in &mut self.level.enemies {
            if !enemy.alive { continue; }

            // Check stomp
            if enemy.can_be_stomped(self.mario.pos, self.mario.vel.y, MARIO_RADIUS) {
                match enemy.stomp() {
                    StompResult::Kill => {
                        self.mario.vel.y = JUMP_VEL * 0.6; // bounce
                        self.mario.collect_coin(); // coins from enemy
                    }
                    StompResult::Stun => {
                        self.mario.vel.y = JUMP_VEL * 0.4;
                    }
                    StompResult::Bounce => {
                        self.mario.vel.y = JUMP_VEL * 0.8;
                    }
                }
                continue;
            }

            // Check if enemy attacks Mario
            if enemy.is_attacking_mario(self.mario.pos, MARIO_RADIUS) {
                self.mario.take_damage(1);
            }
        }
    }

    // ======================== TAS Operations ========================

    fn tas_save_state(&mut self) {
        let state = self.create_save_state();
        self.tas.save_state(state);
    }

    fn tas_load_state(&mut self) {
        if let Some(state) = self.tas.load_state().cloned() {
            self.apply_save_state(&state);
        }
    }

    fn tas_rewind(&mut self) {
        if let Some(state) = self.tas.pop_rewind() {
            self.apply_save_state(&state);
        }
    }

    fn create_save_state(&self) -> SaveState {
        let mut enemy_data = Vec::new();
        for e in &self.level.enemies {
            enemy_data.push((e.pos, e.ai_state as u8));
        }
        let mut object_data = Vec::new();
        for o in &self.level.objects {
            object_data.push((o.pos, o.collected));
        }

        SaveState {
            frame: self.frame_count,
            mario_pos: self.mario.pos,
            mario_vel: self.mario.vel,
            mario_facing: self.mario.facing_yaw,
            mario_action: self.mario.action as u32,
            mario_action_timer: self.mario.action_timer,
            mario_hp: self.mario.hp,
            mario_forward_vel: self.mario.forward_vel,
            mario_peak_height: self.mario.peak_height,
            mario_on_ground: self.mario.is_on_ground(),
            coins: self.mario.coins,
            stars: self.mario.stars,
            lives: self.mario.lives,
            rng_state: self.frame_count, // use frame as RNG seed
            enemy_data,
            object_data,
            cam_pos: self.camera.pos,
            cam_yaw: self.camera.yaw,
            cam_pitch: self.camera.pitch,
            cam_dist: self.camera.dist,
        }
    }

    fn apply_save_state(&mut self, state: &SaveState) {
        self.frame_count = state.frame;
        self.mario.pos = state.mario_pos;
        self.mario.vel = state.mario_vel;
        self.mario.facing_yaw = state.mario_facing;
        self.mario.action_timer = state.mario_action_timer;
        self.mario.hp = state.mario_hp;
        self.mario.forward_vel = state.mario_forward_vel;
        self.mario.peak_height = state.mario_peak_height;
        self.mario.coins = state.coins;
        self.mario.stars = state.stars;
        self.mario.lives = state.lives;
        self.camera.pos = state.cam_pos;
        self.camera.yaw = state.cam_yaw;
        self.camera.pitch = state.cam_pitch;
        self.camera.dist = state.cam_dist;

        // Restore enemies
        for (i, e) in self.level.enemies.iter_mut().enumerate() {
            if i < state.enemy_data.len() {
                e.pos = state.enemy_data[i].0;
            }
        }
        // Restore objects
        for (i, o) in self.level.objects.iter_mut().enumerate() {
            if i < state.object_data.len() {
                o.pos = state.object_data[i].0;
                o.collected = state.object_data[i].1;
                o.visible = !o.collected;
            }
        }
    }

    // ======================== Rendering ========================

    pub fn render(&mut self, out_buf: &mut [u32], w: usize, h: usize) {
        let aspect = RENDER_W as f32 / RENDER_H as f32;
        self.render.begin_frame(&self.camera, aspect);

        // Sky
        self.render.draw_sky();

        // Terrain
        self.render.draw_terrain(&self.level);

        // Objects
        self.render.draw_objects(&self.level.objects);

        // Enemies
        self.render.draw_enemies(&self.level.enemies);

        // Shadow
        self.render.draw_shadow(&self.mario);

        // Mario
        self.render.draw_mario(&self.mario);

        // Ghost (TAS)
        if let Some(ghost) = self.tas.get_ghost_frame() {
            self.render.draw_ghost(&ghost);
        }

        // Upscale to output
        self.render.blit_to_output(out_buf, w, h);

        // HUD (drawn at output resolution)
        hud::draw_hud(out_buf, w, h, &self.mario, &self.tas);

        // Paused overlay
        if self.phase == GamePhase::Paused {
            draw_pause_overlay(out_buf, w, h);
        }

        // Game Over overlay
        if self.phase == GamePhase::GameOver {
            draw_game_over(out_buf, w, h, self.game_over_timer);
        }

        // Star Get overlay
        if self.phase == GamePhase::StarGet {
            draw_star_get(out_buf, w, h, self.star_get_timer, self.mario.stars);
        }
    }
}

// ======================== Overlay Screens ========================

fn draw_pause_overlay(buf: &mut [u32], w: usize, h: usize) {
    // Dim the screen
    for i in 0..w * h {
        let c = buf[i];
        let r = ((c >> 16) & 0xFF) / 2;
        let g = ((c >> 8) & 0xFF) / 2;
        let b = (c & 0xFF) / 2;
        buf[i] = 0xFF000000 | (r << 16) | (g << 8) | b;
    }
    // PAUSED text (centered, using simple box letters)
    let text = b"PAUSED";
    let scale = if w > 400 { 4 } else { 2 };
    let tw = text.len() as i32 * 4 * scale;
    let tx = (w as i32 - tw) / 2;
    let ty = h as i32 / 2 - 3 * scale;
    hud::draw_hud(buf, w, h, &MarioState::new(Vec3::ZERO), &TasEngine::new()); // just reuse for demo
}

fn draw_game_over(buf: &mut [u32], w: usize, h: usize, timer: u32) {
    let fade = ((timer as f32 / 60.0).min(1.0) * 200.0) as u32;
    for i in 0..w * h {
        let c = buf[i];
        let r = (((c >> 16) & 0xFF) as u32).saturating_sub(fade);
        let g = (((c >> 8) & 0xFF) as u32).saturating_sub(fade);
        let b = ((c & 0xFF) as u32).saturating_sub(fade);
        buf[i] = 0xFF000000 | (r << 16) | (g << 8) | b;
    }
}

fn draw_star_get(buf: &mut [u32], w: usize, h: usize, timer: u32, stars: u8) {
    // Flash white briefly
    if timer < 10 {
        let flash = ((10 - timer) as f32 / 10.0 * 100.0) as u32;
        for i in 0..w * h {
            let c = buf[i];
            let r = (((c >> 16) & 0xFF) + flash).min(255);
            let g = (((c >> 8) & 0xFF) + flash).min(255);
            let b = ((c & 0xFF) + flash).min(255);
            buf[i] = 0xFF000000 | (r << 16) | (g << 8) | b;
        }
    }
}
