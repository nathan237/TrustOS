//! TrustOS 3D Game Engine — First-Person Maze Game
//!
//! A raycasting-based 3D game engine using the TrustGL pipeline.
//! Renders textured walls, floor and ceiling in first-person perspective.
//! Inspired by Wolfenstein 3D / early 90s FPS engines.
//!
//! Features:
//! - Raycasting with DDA algorithm for wall detection
//! - Textured walls via affine mapping (procedural textures)
//! - Floor/ceiling rendering
//! - Minimap overlay
//! - Collision detection
//! - Multiple levels
//! - Collectible items
//! - HUD with health, score, compass

use alloc::vec::Vec;
use alloc::format;
use micromath::F32Ext;

// ═══════════════════════════════════════════════════════════════════════════════
// CONSTANTS
// ═══════════════════════════════════════════════════════════════════════════════

/// Map tile types
const TILE_EMPTY: u8 = 0;
const TILE_WALL_BRICK: u8 = 1;
const TILE_WALL_STONE: u8 = 2;
const TILE_WALL_METAL: u8 = 3;
const TILE_WALL_GREEN: u8 = 4;
const TILE_DOOR: u8 = 5;
const TILE_EXIT: u8 = 9;

/// Colors
const COLOR_CEIL: u32 = 0xFF0A1A0A;  // Dark green ceiling
const COLOR_FLOOR: u32 = 0xFF0F0F0F; // Very dark floor
const COLOR_HUD_BG: u32 = 0xCC0A0F0A;
const COLOR_HUD_GREEN: u32 = 0xFF00DD55;
const COLOR_HUD_DIM: u32 = 0xFF006633;
const COLOR_MINIMAP_WALL: u32 = 0xFF00AA44;
const COLOR_MINIMAP_EMPTY: u32 = 0xFF050A05;
const COLOR_MINIMAP_PLAYER: u32 = 0xFF00FF88;
const COLOR_MINIMAP_EXIT: u32 = 0xFFFFFF00;
const COLOR_ITEM: u32 = 0xFF00FFAA;
const COLOR_HEALTH: u32 = 0xFF44FF44;
const COLOR_DAMAGE: u32 = 0xFFFF4444;

// Texture size (power of 2 for fast modulo)
const TEX_SIZE: usize = 64;

// ═══════════════════════════════════════════════════════════════════════════════
// PROCEDURAL TEXTURES
// ═══════════════════════════════════════════════════════════════════════════════

/// Pre-rendered texture as Vec of u32 (ARGB) — heap allocated
struct WallTexture {
    pixels: Vec<u32>,
}

impl WallTexture {
    /// Generate brick wall texture
    fn brick() -> Self {
        let mut pixels = alloc::vec![0u32; TEX_SIZE * TEX_SIZE];
        let brick_h = 16;
        let brick_w = 32;
        let mortar = 2;

        for y in 0..TEX_SIZE {
            for x in 0..TEX_SIZE {
                let row = y / brick_h;
                let offset = if row % 2 == 0 { 0 } else { brick_w / 2 };
                let bx = (x + offset) % brick_w;
                let by = y % brick_h;

                if by < mortar || bx < mortar {
                    // Mortar: dark gray
                    pixels[y * TEX_SIZE + x] = 0xFF333333;
                } else {
                    // Brick with slight variation
                    let noise = ((x * 7 + y * 13) % 20) as u32;
                    let r = 140u32.saturating_add(noise).min(180);
                    let g = 60u32.saturating_add(noise / 2).min(80);
                    let b = 30u32.saturating_add(noise / 3).min(50);
                    pixels[y * TEX_SIZE + x] = 0xFF000000 | (r << 16) | (g << 8) | b;
                }
            }
        }
        WallTexture { pixels }
    }

    /// Generate stone wall texture
    fn stone() -> Self {
        let mut pixels = alloc::vec![0u32; TEX_SIZE * TEX_SIZE];
        for y in 0..TEX_SIZE {
            for x in 0..TEX_SIZE {
                // Stone pattern with pseudo-random noise
                let noise = ((x * 31 + y * 17 + (x ^ y) * 7) % 40) as u32;
                let base = 90u32;
                let v = base.saturating_add(noise).min(140);
                pixels[y * TEX_SIZE + x] = 0xFF000000 | (v << 16) | (v << 8) | v;
            }
        }
        // Add cracks
        for i in 0..TEX_SIZE {
            let cx = (i * 3 + 7) % TEX_SIZE;
            let cy = i;
            if cx < TEX_SIZE && cy < TEX_SIZE {
                pixels[cy * TEX_SIZE + cx] = 0xFF222222;
            }
        }
        WallTexture { pixels }
    }

    /// Generate metal/tech wall texture (green tint)
    fn metal() -> Self {
        let mut pixels = alloc::vec![0u32; TEX_SIZE * TEX_SIZE];
        for y in 0..TEX_SIZE {
            for x in 0..TEX_SIZE {
                // Horizontal bands with rivets
                let band = y % 16;
                let is_edge = band == 0 || band == 15;
                let is_rivet = (x % 16 == 8) && (y % 16 == 8);

                if is_edge {
                    pixels[y * TEX_SIZE + x] = 0xFF556655;
                } else if is_rivet {
                    pixels[y * TEX_SIZE + x] = 0xFF889988;
                } else {
                    let noise = ((x * 11 + y * 23) % 15) as u32;
                    let v = 50u32 + noise;
                    pixels[y * TEX_SIZE + x] = 0xFF000000 | (v / 2 << 16) | (v << 8) | (v / 2);
                }
            }
        }
        WallTexture { pixels }
    }

    /// Generate green matrix-style wall
    fn matrix_wall() -> Self {
        let mut pixels = alloc::vec![0u32; TEX_SIZE * TEX_SIZE];
        for y in 0..TEX_SIZE {
            for x in 0..TEX_SIZE {
                let noise = ((x * 37 + y * 53 + (x * y) % 97) % 50) as u32;
                let col_bright = ((x * 13) % TEX_SIZE) < 4;
                let g = if col_bright {
                    80u32 + noise * 2
                } else {
                    10u32 + noise / 2
                };
                pixels[y * TEX_SIZE + x] = 0xFF000000 | ((g / 4) << 16) | (g.min(200) << 8) | (g / 6);
            }
        }
        WallTexture { pixels }
    }

    /// Generate door texture
    fn door() -> Self {
        let mut pixels = alloc::vec![0u32; TEX_SIZE * TEX_SIZE];
        for y in 0..TEX_SIZE {
            for x in 0..TEX_SIZE {
                let border = x < 3 || x >= TEX_SIZE - 3 || y < 3 || y >= TEX_SIZE - 3;
                let handle_area = x > 48 && x < 56 && y > 26 && y < 38;

                if border {
                    pixels[y * TEX_SIZE + x] = 0xFF008844;
                } else if handle_area {
                    pixels[y * TEX_SIZE + x] = 0xFF00FFAA;
                } else {
                    // Wood-like pattern
                    let noise = ((x * 3 + y * 7) % 20) as u32;
                    let r = 60u32 + noise;
                    let g = 40u32 + noise / 2;
                    let b = 20u32;
                    pixels[y * TEX_SIZE + x] = 0xFF000000 | (r << 16) | (g << 8) | b;
                }
            }
        }
        WallTexture { pixels }
    }

    #[inline]
    fn sample(&self, u: usize, v: usize) -> u32 {
        self.pixels[(v & (TEX_SIZE - 1)) * TEX_SIZE + (u & (TEX_SIZE - 1))]
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ITEM / COLLECTIBLE
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Clone, Copy, PartialEq)]
pub enum ItemType {
    HealthPack,
    DataChip,   // Score item
    KeyCard,    // Opens doors
}

#[derive(Clone, Copy)]
pub struct Item {
    pub x: f32,
    pub y: f32,
    pub item_type: ItemType,
    pub collected: bool,
}

// ═══════════════════════════════════════════════════════════════════════════════
// LEVEL DATA
// ═══════════════════════════════════════════════════════════════════════════════

const MAP_W: usize = 16;
const MAP_H: usize = 16;

/// Level definition
struct Level {
    map: [[u8; MAP_W]; MAP_H],
    spawn_x: f32,
    spawn_y: f32,
    spawn_angle: f32,
    items: Vec<Item>,
}

fn create_level_1() -> Level {
    #[rustfmt::skip]
    let map: [[u8; MAP_W]; MAP_H] = [
        [1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1],
        [1,0,0,0,1,0,0,0,0,0,0,0,0,0,0,1],
        [1,0,0,0,1,0,0,0,0,0,3,0,0,0,0,1],
        [1,0,0,0,5,0,0,0,0,0,3,0,0,0,0,1],
        [1,1,1,1,1,0,0,0,0,0,3,0,0,0,0,1],
        [1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
        [1,0,2,2,0,0,0,4,4,4,0,0,0,0,0,1],
        [1,0,2,2,0,0,0,4,0,4,0,0,0,0,0,1],
        [1,0,0,0,0,0,0,4,0,4,0,0,2,2,0,1],
        [1,0,0,0,0,0,0,0,0,0,0,0,2,0,0,1],
        [1,0,0,0,1,1,5,1,1,0,0,0,2,0,0,1],
        [1,0,0,0,1,0,0,0,1,0,0,0,0,0,0,1],
        [1,0,0,0,1,0,0,0,1,0,0,0,0,0,0,1],
        [1,0,0,0,1,0,0,9,1,0,0,0,3,3,3,1],
        [1,0,0,0,1,1,1,1,1,0,0,0,0,0,0,1],
        [1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1],
    ];

    let items = alloc::vec![
        Item { x: 1.5, y: 1.5, item_type: ItemType::HealthPack, collected: false },
        Item { x: 5.5, y: 2.5, item_type: ItemType::DataChip, collected: false },
        Item { x: 8.5, y: 8.5, item_type: ItemType::DataChip, collected: false },
        Item { x: 14.5, y: 1.5, item_type: ItemType::DataChip, collected: false },
        Item { x: 1.5, y: 13.5, item_type: ItemType::KeyCard, collected: false },
        Item { x: 13.5, y: 9.5, item_type: ItemType::DataChip, collected: false },
    ];

    Level {
        map,
        spawn_x: 2.5,
        spawn_y: 2.5,
        spawn_angle: 0.0,
        items,
    }
}

fn create_level_2() -> Level {
    #[rustfmt::skip]
    let map: [[u8; MAP_W]; MAP_H] = [
        [2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2],
        [2,0,0,0,0,0,0,0,0,0,0,0,0,0,0,2],
        [2,0,3,3,3,0,0,0,0,0,3,3,3,0,0,2],
        [2,0,3,0,0,0,0,0,0,0,0,0,3,0,0,2],
        [2,0,3,0,4,4,4,0,4,4,4,0,3,0,0,2],
        [2,0,0,0,4,0,0,0,0,0,4,0,0,0,0,2],
        [2,0,0,0,4,0,1,1,1,0,4,0,0,0,0,2],
        [2,0,0,0,0,0,1,0,1,0,0,0,0,0,0,2],
        [2,0,0,0,0,0,1,9,1,0,0,0,0,0,0,2],
        [2,0,0,0,4,0,1,1,1,0,4,0,0,0,0,2],
        [2,0,0,0,4,0,0,0,0,0,4,0,0,0,0,2],
        [2,0,3,0,4,4,4,0,4,4,4,0,3,0,0,2],
        [2,0,3,0,0,0,0,0,0,0,0,0,3,0,0,2],
        [2,0,3,3,3,0,0,0,0,0,3,3,3,0,0,2],
        [2,0,0,0,0,0,0,0,0,0,0,0,0,0,0,2],
        [2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2],
    ];

    let items = alloc::vec![
        Item { x: 1.5, y: 1.5, item_type: ItemType::DataChip, collected: false },
        Item { x: 14.5, y: 1.5, item_type: ItemType::DataChip, collected: false },
        Item { x: 1.5, y: 14.5, item_type: ItemType::DataChip, collected: false },
        Item { x: 14.5, y: 14.5, item_type: ItemType::DataChip, collected: false },
        Item { x: 7.5, y: 5.5, item_type: ItemType::HealthPack, collected: false },
        Item { x: 7.5, y: 10.5, item_type: ItemType::KeyCard, collected: false },
    ];

    Level {
        map,
        spawn_x: 1.5,
        spawn_y: 1.5,
        spawn_angle: 0.0,
        items,
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// GAME STATE
// ═══════════════════════════════════════════════════════════════════════════════

pub struct Game3DState {
    // Player
    pub player_x: f32,
    pub player_y: f32,
    pub player_angle: f32,  // radians
    pub player_health: i32,
    pub player_score: u32,
    pub has_keycard: bool,

    // Movement
    move_forward: bool,
    move_back: bool,
    strafe_left: bool,
    strafe_right: bool,
    turn_left: bool,
    turn_right: bool,

    // Level
    map: [[u8; MAP_W]; MAP_H],
    items: Vec<Item>,
    current_level: u32,

    // Textures (Vec-based to avoid stack overflow)
    tex_brick: WallTexture,
    tex_stone: WallTexture,
    tex_metal: WallTexture,
    tex_matrix: WallTexture,
    tex_door: WallTexture,

    // Game state
    frame: u32,
    pub game_won: bool,
    pub game_over: bool,
    flash_timer: u32,          // Damage flash
    pickup_flash_timer: u32,   // Item pickup flash
    message: Option<(alloc::string::String, u32)>, // (text, frames remaining)

    // RNG
    rng_state: u32,
}

impl Game3DState {
    pub fn new() -> Self {
        let level = create_level_1();
        Self {
            player_x: level.spawn_x,
            player_y: level.spawn_y,
            player_angle: level.spawn_angle,
            player_health: 100,
            player_score: 0,
            has_keycard: false,

            move_forward: false,
            move_back: false,
            strafe_left: false,
            strafe_right: false,
            turn_left: false,
            turn_right: false,

            map: level.map,
            items: level.items,
            current_level: 1,

            tex_brick: WallTexture::brick(),
            tex_stone: WallTexture::stone(),
            tex_metal: WallTexture::metal(),
            tex_matrix: WallTexture::matrix_wall(),
            tex_door: WallTexture::door(),

            frame: 0,
            game_won: false,
            game_over: false,
            flash_timer: 0,
            pickup_flash_timer: 0,
            message: None,

            rng_state: 12345,
        }
    }

    fn next_rng(&mut self) -> u32 {
        self.rng_state ^= self.rng_state << 13;
        self.rng_state ^= self.rng_state >> 17;
        self.rng_state ^= self.rng_state << 5;
        self.rng_state
    }

    /// Load next level
    fn load_level(&mut self, level_num: u32) {
        let level = match level_num {
            2 => create_level_2(),
            _ => create_level_1(),
        };
        self.map = level.map;
        self.items = level.items;
        self.player_x = level.spawn_x;
        self.player_y = level.spawn_y;
        self.player_angle = level.spawn_angle;
        self.current_level = level_num;
        self.has_keycard = false;
        self.message = Some((format!("Level {}", level_num), 120));
    }

    /// Handle keyboard input
    pub fn handle_key(&mut self, key: u8) {
        use crate::keyboard::{KEY_UP, KEY_DOWN, KEY_LEFT, KEY_RIGHT};

        if self.game_won || self.game_over {
            if key == b' ' || key == 0x0D {
                // Restart
                *self = Game3DState::new();
            }
            return;
        }

        match key {
            // WASD movement
            b'w' | b'W' | KEY_UP => self.move_forward = true,
            b's' | b'S' | KEY_DOWN => self.move_back = true,
            b'a' | b'A' => self.strafe_left = true,
            b'd' | b'D' => self.strafe_right = true,
            KEY_LEFT => self.turn_left = true,
            KEY_RIGHT => self.turn_right = true,
            b'e' | b'E' => self.try_interact(),
            _ => {}
        }
    }

    /// Handle key release (for continuous movement)
    pub fn handle_key_release(&mut self, key: u8) {
        use crate::keyboard::{KEY_UP, KEY_DOWN, KEY_LEFT, KEY_RIGHT};
        match key {
            b'w' | b'W' | KEY_UP => self.move_forward = false,
            b's' | b'S' | KEY_DOWN => self.move_back = false,
            b'a' | b'A' => self.strafe_left = false,
            b'd' | b'D' => self.strafe_right = false,
            KEY_LEFT => self.turn_left = false,
            KEY_RIGHT => self.turn_right = false,
            _ => {}
        }
    }

    /// Try to interact with nearby objects
    fn try_interact(&mut self) {
        // Check tile in front of player
        let check_x = self.player_x + self.player_angle.cos() * 1.2;
        let check_y = self.player_y + self.player_angle.sin() * 1.2;
        let mx = check_x as usize;
        let my = check_y as usize;

        if mx < MAP_W && my < MAP_H {
            match self.map[my][mx] {
                TILE_DOOR => {
                    if self.has_keycard {
                        self.map[my][mx] = TILE_EMPTY;
                        self.message = Some((alloc::string::String::from("Door opened!"), 90));
                    } else {
                        self.message = Some((alloc::string::String::from("Need keycard!"), 90));
                    }
                }
                TILE_EXIT => {
                    if self.current_level < 2 {
                        self.current_level += 1;
                        self.load_level(self.current_level);
                        self.player_score += 500;
                    } else {
                        self.game_won = true;
                        self.message = Some((alloc::string::String::from("YOU WIN!"), 9999));
                    }
                }
                _ => {}
            }
        }
    }

    /// Game tick — called each frame
    pub fn tick(&mut self) {
        if self.game_won || self.game_over {
            return;
        }

        self.frame += 1;

        // Turning
        let turn_speed = 0.06;
        if self.turn_left { self.player_angle -= turn_speed; }
        if self.turn_right { self.player_angle += turn_speed; }

        // Movement
        let move_speed = 0.06;
        let cos_a = self.player_angle.cos();
        let sin_a = self.player_angle.sin();
        let mut dx = 0.0f32;
        let mut dy = 0.0f32;

        if self.move_forward { dx += cos_a * move_speed; dy += sin_a * move_speed; }
        if self.move_back { dx -= cos_a * move_speed; dy -= sin_a * move_speed; }
        if self.strafe_left { dx += sin_a * move_speed; dy -= cos_a * move_speed; }
        if self.strafe_right { dx -= sin_a * move_speed; dy += cos_a * move_speed; }

        // Collision detection with sliding
        let margin = 0.25;
        let new_x = self.player_x + dx;
        let new_y = self.player_y + dy;

        // X movement
        if !self.is_wall(new_x + margin * dx.signum(), self.player_y) {
            self.player_x = new_x;
        }
        // Y movement
        if !self.is_wall(self.player_x, new_y + margin * dy.signum()) {
            self.player_y = new_y;
        }

        // Item pickup
        self.check_pickups();

        // Auto-interact with exit when touching
        let mx = self.player_x as usize;
        let my = self.player_y as usize;
        if mx < MAP_W && my < MAP_H && self.map[my][mx] == TILE_EXIT {
            self.try_interact();
        }

        // Update timers
        if self.flash_timer > 0 { self.flash_timer -= 1; }
        if self.pickup_flash_timer > 0 { self.pickup_flash_timer -= 1; }
        if let Some((_, ref mut frames)) = self.message {
            if *frames > 0 { *frames -= 1; }
            else { self.message = None; }
        }
    }

    #[inline]
    fn is_wall(&self, x: f32, y: f32) -> bool {
        let mx = x as usize;
        let my = y as usize;
        if mx >= MAP_W || my >= MAP_H { return true; }
        self.map[my][mx] != TILE_EMPTY
    }

    fn check_pickups(&mut self) {
        for item in &mut self.items {
            if item.collected { continue; }
            let dx = item.x - self.player_x;
            let dy = item.y - self.player_y;
            if dx * dx + dy * dy < 0.5 {
                item.collected = true;
                self.pickup_flash_timer = 15;
                match item.item_type {
                    ItemType::HealthPack => {
                        self.player_health = (self.player_health + 25).min(100);
                        self.message = Some((alloc::string::String::from("+25 HP"), 60));
                    }
                    ItemType::DataChip => {
                        self.player_score += 100;
                        self.message = Some((alloc::string::String::from("+100 pts"), 60));
                    }
                    ItemType::KeyCard => {
                        self.has_keycard = true;
                        self.message = Some((alloc::string::String::from("KEYCARD acquired!"), 90));
                    }
                }
            }
        }
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // RENDERING — Raycasting Engine
    // ═══════════════════════════════════════════════════════════════════════════

    /// Render the full game view into a pixel buffer
    pub fn render(&self, buf: &mut [u32], w: usize, h: usize) {
        if w < 80 || h < 60 { return; }

        let hud_h = 40; // Bottom HUD height
        let view_h = h.saturating_sub(hud_h);

        // Clear buffer
        for i in 0..w * h {
            buf[i] = 0xFF000000;
        }

        // Render ceiling and floor
        self.render_floor_ceiling(buf, w, view_h);

        // Raycasting — render walls
        self.render_walls(buf, w, view_h);

        // Render item sprites (simple billboard circles)
        self.render_items(buf, w, view_h);

        // Minimap
        self.render_minimap(buf, w, h);

        // HUD
        self.render_hud(buf, w, h, view_h);

        // Damage flash overlay
        if self.flash_timer > 0 {
            let alpha = (self.flash_timer as u32 * 8).min(80);
            for i in 0..w * view_h {
                let r = ((buf[i] >> 16) & 0xFF).saturating_add(alpha);
                buf[i] = (buf[i] & 0xFF00FFFF) | (r.min(255) << 16);
            }
        }

        // Pickup flash
        if self.pickup_flash_timer > 0 {
            let alpha = (self.pickup_flash_timer as u32 * 5).min(40);
            for i in 0..w * view_h {
                let g = ((buf[i] >> 8) & 0xFF).saturating_add(alpha);
                buf[i] = (buf[i] & 0xFFFF00FF) | (g.min(255) << 8);
            }
        }

        // Game over / win overlay
        if self.game_won || self.game_over {
            // Darken
            for i in 0..w * h {
                let r = ((buf[i] >> 16) & 0xFF) / 3;
                let g = ((buf[i] >> 8) & 0xFF) / 3;
                let b = (buf[i] & 0xFF) / 3;
                buf[i] = 0xFF000000 | (r << 16) | (g << 8) | b;
            }
        }

        // Messages
        if let Some((ref msg, _)) = self.message {
            self.draw_text_centered(buf, w, view_h / 4, msg, COLOR_HUD_GREEN);
        }
    }

    fn render_floor_ceiling(&self, buf: &mut [u32], w: usize, h: usize) {
        let half = h / 2;

        for y in 0..h {
            if y < half {
                // Ceiling - gradient from dark to slightly lighter
                let t = y as u32 * 20 / half as u32;
                let g = 8u32 + t;
                let color = 0xFF000000 | ((g / 3) << 16) | (g << 8) | (g / 4);
                for x in 0..w {
                    buf[y * w + x] = color;
                }
            } else {
                // Floor - gradient from lighter near horizon to dark at bottom
                let dist = (y - half) as u32;
                let max_dist = half as u32;
                let t = dist * 25 / max_dist.max(1);
                let base = 6u32 + t;
                let color = 0xFF000000 | ((base / 2) << 16) | ((base / 2) << 8) | (base / 2);
                for x in 0..w {
                    buf[y * w + x] = color;
                }
            }
        }
    }

    fn render_walls(&self, buf: &mut [u32], w: usize, h: usize) {
        let fov = core::f32::consts::FRAC_PI_3; // 60 degrees
        let half_h = h as f32 / 2.0;

        for col in 0..w {
            // Ray angle for this column
            let ray_offset = (col as f32 / w as f32 - 0.5) * 2.0;
            let ray_angle = self.player_angle + ray_offset * (fov / 2.0);

            let ray_dx = ray_angle.cos();
            let ray_dy = ray_angle.sin();

            // DDA Raycasting
            let mut map_x = self.player_x as i32;
            let mut map_y = self.player_y as i32;

            let delta_dist_x = if ray_dx.abs() < 1e-8 { 1e8 } else { (1.0 / ray_dx).abs() };
            let delta_dist_y = if ray_dy.abs() < 1e-8 { 1e8 } else { (1.0 / ray_dy).abs() };

            let (step_x, mut side_dist_x) = if ray_dx < 0.0 {
                (-1i32, (self.player_x - map_x as f32) * delta_dist_x)
            } else {
                (1i32, ((map_x + 1) as f32 - self.player_x) * delta_dist_x)
            };

            let (step_y, mut side_dist_y) = if ray_dy < 0.0 {
                (-1i32, (self.player_y - map_y as f32) * delta_dist_y)
            } else {
                (1i32, ((map_y + 1) as f32 - self.player_y) * delta_dist_y)
            };

            // Step through grid
            let mut hit = false;
            let mut side = 0; // 0 = X side, 1 = Y side
            let mut wall_type = TILE_WALL_BRICK;

            for _ in 0..64 {
                if side_dist_x < side_dist_y {
                    side_dist_x += delta_dist_x;
                    map_x += step_x;
                    side = 0;
                } else {
                    side_dist_y += delta_dist_y;
                    map_y += step_y;
                    side = 1;
                }

                if map_x < 0 || map_y < 0 || map_x >= MAP_W as i32 || map_y >= MAP_H as i32 {
                    break;
                }

                let tile = self.map[map_y as usize][map_x as usize];
                if tile != TILE_EMPTY {
                    hit = true;
                    wall_type = tile;
                    break;
                }
            }

            if !hit { continue; }

            // Perpendicular distance (fixes fisheye)
            let perp_dist = if side == 0 {
                (map_x as f32 - self.player_x + (1.0 - step_x as f32) / 2.0) / ray_dx
            } else {
                (map_y as f32 - self.player_y + (1.0 - step_y as f32) / 2.0) / ray_dy
            };

            if perp_dist <= 0.0 { continue; }

            // Wall height on screen
            let wall_height = (h as f32 / perp_dist).min(h as f32 * 4.0);
            let draw_start = ((half_h - wall_height / 2.0) as i32).max(0) as usize;
            let draw_end = ((half_h + wall_height / 2.0) as i32).min(h as i32 - 1) as usize;

            // Texture coordinate (where on the wall face was hit)
            let wall_x = if side == 0 {
                self.player_y + perp_dist * ray_dy
            } else {
                self.player_x + perp_dist * ray_dx
            };
            let wall_x = wall_x - wall_x.floor();  // fractional part [0..1)
            let tex_x = (wall_x * TEX_SIZE as f32) as usize;

            // Select texture
            let tex = match wall_type {
                TILE_WALL_BRICK => &self.tex_brick,
                TILE_WALL_STONE => &self.tex_stone,
                TILE_WALL_METAL => &self.tex_metal,
                TILE_WALL_GREEN => &self.tex_matrix,
                TILE_DOOR => &self.tex_door,
                TILE_EXIT => &self.tex_matrix,
                _ => &self.tex_brick,
            };

            // Draw wall column with texture
            let inv_wall_h = TEX_SIZE as f32 / wall_height;
            for y in draw_start..=draw_end {
                let tex_y = ((y as f32 - (half_h - wall_height / 2.0)) * inv_wall_h) as usize;
                let mut pixel = tex.sample(tex_x, tex_y);

                // Distance fog (darken far walls)
                let fog = (1.0 - (perp_dist / 12.0).min(1.0)).max(0.15);
                let r = (((pixel >> 16) & 0xFF) as f32 * fog) as u32;
                let g = (((pixel >> 8) & 0xFF) as f32 * fog) as u32;
                let b = ((pixel & 0xFF) as f32 * fog) as u32;
                pixel = 0xFF000000 | (r << 16) | (g << 8) | b;

                // Darken Y-side slightly for depth cue
                if side == 1 {
                    let r = ((pixel >> 16) & 0xFF) * 3 / 4;
                    let g = ((pixel >> 8) & 0xFF) * 3 / 4;
                    let b = (pixel & 0xFF) * 3 / 4;
                    pixel = 0xFF000000 | (r << 16) | (g << 8) | b;
                }

                // Exit tile pulsing green tint
                if wall_type == TILE_EXIT {
                    let pulse = ((self.frame as f32 * 0.1).sin() * 30.0 + 30.0) as u32;
                    let g_val = ((pixel >> 8) & 0xFF).saturating_add(pulse).min(255);
                    pixel = (pixel & 0xFFFF00FF) | (g_val << 8);
                }

                buf[y * w + col] = pixel;
            }
        }
    }

    fn render_items(&self, buf: &mut [u32], w: usize, h: usize) {
        let half_h = h as f32 / 2.0;

        for item in &self.items {
            if item.collected { continue; }

            // Relative position to player
            let dx = item.x - self.player_x;
            let dy = item.y - self.player_y;

            // Transform to player-relative coordinates
            let cos_a = self.player_angle.cos();
            let sin_a = self.player_angle.sin();
            let tx = dx * cos_a + dy * sin_a;
            let ty = -dx * sin_a + dy * cos_a;

            // Behind player
            if ty < 0.2 { continue; }

            // Convert to screen space
            let fov = core::f32::consts::FRAC_PI_3;
            let screen_x = (0.5 + tx / (ty * (fov / 2.0).tan() * 2.0)) * w as f32;
            let sprite_size = (h as f32 / ty * 0.3) as i32;

            if sprite_size < 2 { continue; }

            let sx = screen_x as i32 - sprite_size / 2;
            let sy = (half_h as i32) - sprite_size / 2;

            let color = match item.item_type {
                ItemType::HealthPack => COLOR_HEALTH,
                ItemType::DataChip => COLOR_ITEM,
                ItemType::KeyCard => 0xFFFFAA00,
            };

            // Draw as a diamond shape
            let half = sprite_size / 2;
            for dy_off in -half..=half {
                let row = sy + half + dy_off;
                if row < 0 || row >= h as i32 { continue; }
                let row_width = half - dy_off.abs();
                for dx_off in -row_width..=row_width {
                    let cx = sx + half + dx_off;
                    if cx < 0 || cx >= w as i32 { continue; }

                    // Pulsing brightness
                    let pulse = ((self.frame as f32 * 0.15 + item.x * 3.0).sin() * 0.3 + 0.7) as f32;
                    let r = (((color >> 16) & 0xFF) as f32 * pulse) as u32;
                    let g = (((color >> 8) & 0xFF) as f32 * pulse) as u32;
                    let b = ((color & 0xFF) as f32 * pulse) as u32;

                    buf[row as usize * w + cx as usize] = 0xFF000000 | (r << 16) | (g << 8) | b;
                }
            }
        }
    }

    fn render_minimap(&self, buf: &mut [u32], w: usize, h: usize) {
        let cell = 5;
        let map_w_px = MAP_W * cell;
        let map_h_px = MAP_H * cell;
        let offset_x = w - map_w_px - 8;
        let offset_y = 8;

        // Background
        for y in 0..map_h_px + 4 {
            for x in 0..map_w_px + 4 {
                let px = offset_x - 2 + x;
                let py = offset_y - 2 + y;
                if px < w && py < h {
                    buf[py * w + px] = 0xAA000000;
                }
            }
        }

        // Map tiles
        for my in 0..MAP_H {
            for mx in 0..MAP_W {
                let color = match self.map[my][mx] {
                    TILE_EMPTY => COLOR_MINIMAP_EMPTY,
                    TILE_EXIT => COLOR_MINIMAP_EXIT,
                    TILE_DOOR => 0xFF884400,
                    _ => COLOR_MINIMAP_WALL,
                };
                for dy in 0..cell {
                    for dx in 0..cell {
                        let px = offset_x + mx * cell + dx;
                        let py = offset_y + my * cell + dy;
                        if px < w && py < h {
                            buf[py * w + px] = color;
                        }
                    }
                }
            }
        }

        // Player dot
        let px = offset_x + (self.player_x * cell as f32) as usize;
        let py = offset_y + (self.player_y * cell as f32) as usize;
        for dy in 0..3usize {
            for dx in 0..3usize {
                let x = px + dx;
                let y = py + dy;
                if x < w && y < h {
                    buf[y * w + x] = COLOR_MINIMAP_PLAYER;
                }
            }
        }

        // Direction line
        let dir_len = 6.0;
        let ex = px as f32 + self.player_angle.cos() * dir_len;
        let ey = py as f32 + self.player_angle.sin() * dir_len;
        let steps = 8;
        for i in 0..steps {
            let t = i as f32 / steps as f32;
            let lx = (px as f32 + (ex - px as f32) * t) as usize;
            let ly = (py as f32 + (ey - py as f32) * t) as usize;
            if lx < w && ly < h {
                buf[ly * w + lx] = COLOR_MINIMAP_PLAYER;
            }
        }

        // Uncollected items on minimap
        for item in &self.items {
            if item.collected { continue; }
            let ix = offset_x + (item.x * cell as f32) as usize;
            let iy = offset_y + (item.y * cell as f32) as usize;
            let ic = match item.item_type {
                ItemType::HealthPack => COLOR_HEALTH,
                ItemType::DataChip => COLOR_ITEM,
                ItemType::KeyCard => 0xFFFFAA00,
            };
            if ix > 0 && ix + 1 < w && iy > 0 && iy + 1 < h {
                buf[iy * w + ix] = ic;
                buf[iy * w + ix + 1] = ic;
                buf[(iy + 1) * w + ix] = ic;
                buf[(iy + 1) * w + ix + 1] = ic;
            }
        }
    }

    fn render_hud(&self, buf: &mut [u32], w: usize, h: usize, view_h: usize) {
        // HUD background
        for y in view_h..h {
            for x in 0..w {
                buf[y * w + x] = 0xFF0A120A;
            }
        }

        // Top border
        for x in 0..w {
            if view_h < h {
                buf[view_h * w + x] = COLOR_HUD_GREEN;
            }
        }

        let hud_y = view_h + 4;
        let text_scale = 1;

        // Health bar
        self.draw_text_at(buf, w, h, 8, hud_y, "HP", COLOR_HUD_DIM);
        let bar_x = 28;
        let bar_w = 80;
        let bar_h = 10;
        // Background
        for y in 0..bar_h {
            for x in 0..bar_w {
                let px = bar_x + x;
                let py = hud_y + 4 + y;
                if px < w && py < h {
                    buf[py * w + px] = 0xFF1A1A1A;
                }
            }
        }
        // Fill
        let fill_w = (self.player_health as usize * bar_w / 100).min(bar_w);
        let hp_color = if self.player_health > 60 { COLOR_HEALTH }
                       else if self.player_health > 30 { 0xFFAAAA00 }
                       else { COLOR_DAMAGE };
        for y in 0..bar_h {
            for x in 0..fill_w {
                let px = bar_x + x;
                let py = hud_y + 4 + y;
                if px < w && py < h {
                    buf[py * w + px] = hp_color;
                }
            }
        }

        // Score
        let score_str = format!("SCORE:{}", self.player_score);
        self.draw_text_at(buf, w, h, 120, hud_y, &score_str, COLOR_HUD_GREEN);

        // Level
        let level_str = format!("LVL:{}", self.current_level);
        self.draw_text_at(buf, w, h, 240, hud_y, &level_str, COLOR_HUD_DIM);

        // Keycard indicator
        if self.has_keycard {
            self.draw_text_at(buf, w, h, 310, hud_y, "[KEY]", 0xFFFFAA00);
        }

        // Compass
        let compass_x = w - 60;
        let dirs = ["N", "E", "S", "W"];
        let angle_normalized = (self.player_angle + core::f32::consts::PI * 2.0) % (core::f32::consts::PI * 2.0);
        let dir_idx = ((angle_normalized + core::f32::consts::FRAC_PI_4) / core::f32::consts::FRAC_PI_2) as usize % 4;
        self.draw_text_at(buf, w, h, compass_x, hud_y, dirs[dir_idx], COLOR_HUD_GREEN);

        // Controls hint
        self.draw_text_at(buf, w, h, 8, hud_y + 18, "WASD:Move Arrows:Turn E:Use", COLOR_HUD_DIM);
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // TEXT RENDERING (simple built-in font to pixel buffer)
    // ═══════════════════════════════════════════════════════════════════════════

    fn draw_text_at(&self, buf: &mut [u32], w: usize, h: usize, x: usize, y: usize, text: &str, color: u32) {
        for (i, ch) in text.chars().enumerate() {
            let cx = x + i * 7;
            if cx + 6 >= w { break; }
            self.draw_char(buf, w, h, cx, y, ch, color);
        }
    }

    fn draw_text_centered(&self, buf: &mut [u32], w: usize, y: usize, text: &str, color: u32) {
        let text_w = text.len() * 7;
        let x = if text_w < w { (w - text_w) / 2 } else { 0 };
        self.draw_text_at(buf, w, w * (w / w), x, y, text, color); // h approximation
        // Actually need h, use a safe bound
        for (i, ch) in text.chars().enumerate() {
            let cx = x + i * 7;
            if cx + 6 >= w { break; }
            // Just write if in bounds (buf size covers it)
            let max_y = buf.len() / w;
            self.draw_char(buf, w, max_y, cx, y, ch, color);
        }
    }

    fn draw_char(&self, buf: &mut [u32], w: usize, h: usize, x: usize, y: usize, ch: char, color: u32) {
        // Tiny 5x7 bitmap font for HUD text
        let bitmap = match ch {
            'A' => [0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001],
            'B' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10001, 0b10001, 0b11110],
            'C' => [0b01110, 0b10001, 0b10000, 0b10000, 0b10000, 0b10001, 0b01110],
            'D' => [0b11110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b11110],
            'E' => [0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111],
            'F' => [0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b10000],
            'G' => [0b01110, 0b10001, 0b10000, 0b10111, 0b10001, 0b10001, 0b01110],
            'H' => [0b10001, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001],
            'I' => [0b01110, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110],
            'J' => [0b00111, 0b00010, 0b00010, 0b00010, 0b10010, 0b10010, 0b01100],
            'K' => [0b10001, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010, 0b10001],
            'L' => [0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111],
            'M' => [0b10001, 0b11011, 0b10101, 0b10101, 0b10001, 0b10001, 0b10001],
            'N' => [0b10001, 0b11001, 0b10101, 0b10011, 0b10001, 0b10001, 0b10001],
            'O' => [0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110],
            'P' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000],
            'Q' => [0b01110, 0b10001, 0b10001, 0b10001, 0b10101, 0b10010, 0b01101],
            'R' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10100, 0b10010, 0b10001],
            'S' => [0b01110, 0b10001, 0b10000, 0b01110, 0b00001, 0b10001, 0b01110],
            'T' => [0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100],
            'U' => [0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110],
            'V' => [0b10001, 0b10001, 0b10001, 0b10001, 0b01010, 0b01010, 0b00100],
            'W' => [0b10001, 0b10001, 0b10001, 0b10101, 0b10101, 0b11011, 0b10001],
            'X' => [0b10001, 0b10001, 0b01010, 0b00100, 0b01010, 0b10001, 0b10001],
            'Y' => [0b10001, 0b10001, 0b01010, 0b00100, 0b00100, 0b00100, 0b00100],
            'Z' => [0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b11111],
            '0' => [0b01110, 0b10011, 0b10101, 0b10101, 0b11001, 0b10001, 0b01110],
            '1' => [0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110],
            '2' => [0b01110, 0b10001, 0b00001, 0b00110, 0b01000, 0b10000, 0b11111],
            '3' => [0b01110, 0b10001, 0b00001, 0b00110, 0b00001, 0b10001, 0b01110],
            '4' => [0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010],
            '5' => [0b11111, 0b10000, 0b11110, 0b00001, 0b00001, 0b10001, 0b01110],
            '6' => [0b01110, 0b10000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110],
            '7' => [0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000],
            '8' => [0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110],
            '9' => [0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00001, 0b01110],
            ':' => [0b00000, 0b00100, 0b00100, 0b00000, 0b00100, 0b00100, 0b00000],
            '+' => [0b00000, 0b00100, 0b00100, 0b11111, 0b00100, 0b00100, 0b00000],
            '-' => [0b00000, 0b00000, 0b00000, 0b11111, 0b00000, 0b00000, 0b00000],
            '!' => [0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00000, 0b00100],
            '.' => [0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00100],
            '[' => [0b01100, 0b01000, 0b01000, 0b01000, 0b01000, 0b01000, 0b01100],
            ']' => [0b00110, 0b00010, 0b00010, 0b00010, 0b00010, 0b00010, 0b00110],
            ' ' => [0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000],
            _   => [0b11111, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b11111],
        };

        for row in 0..7 {
            for col in 0..5 {
                if bitmap[row] & (1 << (4 - col)) != 0 {
                    let px = x + col;
                    let py = y + row;
                    if px < w && py < h {
                        buf[py * w + px] = color;
                    }
                }
            }
        }
    }
}
