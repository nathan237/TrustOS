















use alloc::vec::Vec;
use alloc::format;
use micromath::F32Ext;






const QZ_: u8 = 0;
const BJG_: u8 = 1;
const DBU_: u8 = 2;
const DBT_: u8 = 3;
const DBS_: u8 = 4;
const AKQ_: u8 = 5;
const RA_: u8 = 9;


const DIY_: u32 = 0xFF0A1A0A;  
const DJA_: u32 = 0xFF0F0F0F; 
const DJC_: u32 = 0xCC0A0F0A;
const TD_: u32 = 0xFF00DD55;
const ABT_: u32 = 0xFF006633;
const BQA_: u32 = 0xFF00AA44;
const BPY_: u32 = 0xFF050A05;
const AQJ_: u32 = 0xFF00FF88;
const BPZ_: u32 = 0xFFFFFF00;
const AQG_: u32 = 0xFF00FFAA;
const ABS_: u32 = 0xFF44FF44;
const AQD_: u32 = 0xFFFF4444;


const AA_: usize = 64;






struct WallTexture {
    pixels: Vec<u32>,
}

impl WallTexture {
    
    fn kdx() -> Self {
        let mut pixels = alloc::vec![0u32; AA_ * AA_];
        let djo = 16;
        let djp = 32;
        let duo = 2;

        for y in 0..AA_ {
            for x in 0..AA_ {
                let row = y / djo;
                let offset = if row % 2 == 0 { 0 } else { djp / 2 };
                let bx = (x + offset) % djp;
                let dc = y % djo;

                if dc < duo || bx < duo {
                    
                    pixels[y * AA_ + x] = 0xFF333333;
                } else {
                    
                    let aig = ((x * 7 + y * 13) % 20) as u32;
                    let r = 140u32.saturating_add(aig).min(180);
                    let g = 60u32.saturating_add(aig / 2).min(80);
                    let b = 30u32.saturating_add(aig / 3).min(50);
                    pixels[y * AA_ + x] = 0xFF000000 | (r << 16) | (g << 8) | b;
                }
            }
        }
        WallTexture { pixels }
    }

    
    fn oxk() -> Self {
        let mut pixels = alloc::vec![0u32; AA_ * AA_];
        for y in 0..AA_ {
            for x in 0..AA_ {
                
                let aig = ((x * 31 + y * 17 + (x ^ y) * 7) % 40) as u32;
                let base = 90u32;
                let v = base.saturating_add(aig).min(140);
                pixels[y * AA_ + x] = 0xFF000000 | (v << 16) | (v << 8) | v;
            }
        }
        
        for i in 0..AA_ {
            let cx = (i * 3 + 7) % AA_;
            let u = i;
            if cx < AA_ && u < AA_ {
                pixels[u * AA_ + cx] = 0xFF222222;
            }
        }
        WallTexture { pixels }
    }

    
    fn nfb() -> Self {
        let mut pixels = alloc::vec![0u32; AA_ * AA_];
        for y in 0..AA_ {
            for x in 0..AA_ {
                
                let akx = y % 16;
                let msk = akx == 0 || akx == 15;
                let mtp = (x % 16 == 8) && (y % 16 == 8);

                if msk {
                    pixels[y * AA_ + x] = 0xFF556655;
                } else if mtp {
                    pixels[y * AA_ + x] = 0xFF889988;
                } else {
                    let aig = ((x * 11 + y * 23) % 15) as u32;
                    let v = 50u32 + aig;
                    pixels[y * AA_ + x] = 0xFF000000 | (v / 2 << 16) | (v << 8) | (v / 2);
                }
            }
        }
        WallTexture { pixels }
    }

    
    fn ncn() -> Self {
        let mut pixels = alloc::vec![0u32; AA_ * AA_];
        for y in 0..AA_ {
            for x in 0..AA_ {
                let aig = ((x * 37 + y * 53 + (x * y) % 97) % 50) as u32;
                let kvb = ((x * 13) % AA_) < 4;
                let g = if kvb {
                    80u32 + aig * 2
                } else {
                    10u32 + aig / 2
                };
                pixels[y * AA_ + x] = 0xFF000000 | ((g / 4) << 16) | (g.min(200) << 8) | (g / 6);
            }
        }
        WallTexture { pixels }
    }

    
    fn lgw() -> Self {
        let mut pixels = alloc::vec![0u32; AA_ * AA_];
        for y in 0..AA_ {
            for x in 0..AA_ {
                let border = x < 3 || x >= AA_ - 3 || y < 3 || y >= AA_ - 3;
                let mhi = x > 48 && x < 56 && y > 26 && y < 38;

                if border {
                    pixels[y * AA_ + x] = 0xFF008844;
                } else if mhi {
                    pixels[y * AA_ + x] = 0xFF00FFAA;
                } else {
                    
                    let aig = ((x * 3 + y * 7) % 20) as u32;
                    let r = 60u32 + aig;
                    let g = 40u32 + aig / 2;
                    let b = 20u32;
                    pixels[y * AA_ + x] = 0xFF000000 | (r << 16) | (g << 8) | b;
                }
            }
        }
        WallTexture { pixels }
    }

    #[inline]
    fn sample(&self, iy: usize, v: usize) -> u32 {
        self.pixels[(v & (AA_ - 1)) * AA_ + (iy & (AA_ - 1))]
    }
}





#[derive(Clone, Copy, PartialEq)]
pub enum ItemType {
    HealthPack,
    DataChip,   
    KeyCard,    
}

#[derive(Clone, Copy)]
pub struct Item {
    pub x: f32,
    pub y: f32,
    pub item_type: ItemType,
    pub collected: bool,
}





#[derive(Clone, Copy, PartialEq)]
pub enum EnemyState {
    Idle,
    Chasing,
    Attacking,
    Dead,
}

#[derive(Clone, Copy)]
pub struct Fo {
    pub x: f32,
    pub y: f32,
    pub health: i32,
    pub max_health: i32,
    pub state: EnemyState,
    pub attack_cooldown: u32,
    pub damage: i32,
    pub speed: f32,
    pub sight_range: f32,
    pub attack_range: f32,
}





const EH_: usize = 16;
const EG_: usize = 16;


struct Gp {
    map: [[u8; EH_]; EG_],
    spawn_x: f32,
    spawn_y: f32,
    spawn_angle: f32,
    items: Vec<Item>,
    enemies: Vec<Fo>,
}

fn hoq() -> Gp {
    #[rustfmt::skip]
    let map: [[u8; EH_]; EG_] = [
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

    Gp {
        map,
        spawn_x: 2.5,
        spawn_y: 2.5,
        spawn_angle: 0.0,
        items,
        enemies: alloc::vec![
            Fo { x: 8.5, y: 3.5, health: 30, max_health: 30, state: EnemyState::Idle, attack_cooldown: 0, damage: 8, speed: 0.02, sight_range: 6.0, attack_range: 1.5 },
            Fo { x: 3.5, y: 8.5, health: 30, max_health: 30, state: EnemyState::Idle, attack_cooldown: 0, damage: 8, speed: 0.02, sight_range: 6.0, attack_range: 1.5 },
            Fo { x: 13.5, y: 5.5, health: 40, max_health: 40, state: EnemyState::Idle, attack_cooldown: 0, damage: 12, speed: 0.025, sight_range: 8.0, attack_range: 1.5 },
        ],
    }
}

fn kzi() -> Gp {
    #[rustfmt::skip]
    let map: [[u8; EH_]; EG_] = [
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

    Gp {
        map,
        spawn_x: 1.5,
        spawn_y: 1.5,
        spawn_angle: 0.0,
        items,
        enemies: alloc::vec![
            Fo { x: 5.5, y: 5.5, health: 40, max_health: 40, state: EnemyState::Idle, attack_cooldown: 0, damage: 10, speed: 0.025, sight_range: 7.0, attack_range: 1.5 },
            Fo { x: 10.5, y: 5.5, health: 40, max_health: 40, state: EnemyState::Idle, attack_cooldown: 0, damage: 10, speed: 0.025, sight_range: 7.0, attack_range: 1.5 },
            Fo { x: 5.5, y: 10.5, health: 40, max_health: 40, state: EnemyState::Idle, attack_cooldown: 0, damage: 10, speed: 0.025, sight_range: 7.0, attack_range: 1.5 },
            Fo { x: 10.5, y: 10.5, health: 50, max_health: 50, state: EnemyState::Idle, attack_cooldown: 0, damage: 15, speed: 0.03, sight_range: 8.0, attack_range: 1.5 },
        ],
    }
}





pub struct Game3DState {
    
    pub player_x: f32,
    pub player_y: f32,
    pub player_angle: f32,  
    pub player_health: i32,
    pub player_score: u32,
    pub has_keycard: bool,

    
    move_forward: bool,
    move_back: bool,
    strafe_left: bool,
    strafe_right: bool,
    turn_left: bool,
    turn_right: bool,

    
    map: [[u8; EH_]; EG_],
    items: Vec<Item>,
    enemies: Vec<Fo>,
    current_level: u32,

    
    tex_brick: WallTexture,
    tex_stone: WallTexture,
    tex_metal: WallTexture,
    tex_matrix: WallTexture,
    tex_door: WallTexture,

    
    frame: u32,
    pub game_won: bool,
    pub game_over: bool,
    flash_timer: u32,          
    pickup_flash_timer: u32,   
    message: Option<(alloc::string::String, u32)>, 

    
    shoot_cooldown: u32,
    shoot_flash: u32,
    weapon_bob: f32,
    kills: u32,
    z_buffer: Vec<f32>,  

    
    rng_state: u32,

    
    ray_cos_table: Vec<f32>,
    ray_sin_table: Vec<f32>,
    last_table_width: usize,
    last_table_angle: f32,
}

impl Game3DState {
    pub fn new() -> Self {
        let level = hoq();
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
            enemies: level.enemies,
            current_level: 1,

            tex_brick: WallTexture::kdx(),
            tex_stone: WallTexture::oxk(),
            tex_metal: WallTexture::nfb(),
            tex_matrix: WallTexture::ncn(),
            tex_door: WallTexture::lgw(),

            frame: 0,
            game_won: false,
            game_over: false,
            flash_timer: 0,
            pickup_flash_timer: 0,
            message: None,

            shoot_cooldown: 0,
            shoot_flash: 0,
            weapon_bob: 0.0,
            kills: 0,
            z_buffer: Vec::new(),

            rng_state: 12345,

            ray_cos_table: Vec::new(),
            ray_sin_table: Vec::new(),
            last_table_width: 0,
            last_table_angle: f32::NAN,
        }
    }

    
    fn rebuild_ray_table(&mut self, w: usize) {
        if w == self.last_table_width && self.player_angle == self.last_table_angle {
            return; 
        }
        let fov = core::f32::consts::FRAC_PI_3;
        self.ray_cos_table.resize(w, 0.0);
        self.ray_sin_table.resize(w, 0.0);
        for col in 0..w {
            let obr = (col as f32 / w as f32 - 0.5) * 2.0;
            let iyb = self.player_angle + obr * (fov / 2.0);
            self.ray_cos_table[col] = iyb.cos();
            self.ray_sin_table[col] = iyb.sin();
        }
        self.last_table_width = w;
        self.last_table_angle = self.player_angle;
    }

    fn next_rng(&mut self) -> u32 {
        self.rng_state ^= self.rng_state << 13;
        self.rng_state ^= self.rng_state >> 17;
        self.rng_state ^= self.rng_state << 5;
        self.rng_state
    }

    
    fn load_level(&mut self, level_num: u32) {
        let level = match level_num {
            2 => kzi(),
            _ => hoq(),
        };
        self.map = level.map;
        self.items = level.items;
        self.enemies = level.enemies;
        self.player_x = level.spawn_x;
        self.player_y = level.spawn_y;
        self.player_angle = level.spawn_angle;
        self.current_level = level_num;
        self.has_keycard = false;
        self.message = Some((format!("Level {}", level_num), 120));
    }

    
    pub fn handle_key(&mut self, key: u8) {
        use crate::keyboard::{T_, S_, AI_, AJ_};

        if self.game_won || self.game_over {
            if key == b' ' || key == 0x0D {
                
                *self = Game3DState::new();
            }
            return;
        }

        match key {
            
            b'w' | b'W' | T_ => self.move_forward = true,
            b's' | b'S' | S_ => self.move_back = true,
            b'a' | b'A' => self.strafe_left = true,
            b'd' | b'D' => self.strafe_right = true,
            AI_ => self.turn_left = true,
            AJ_ => self.turn_right = true,
            b'e' | b'E' => self.try_interact(),
            b' ' => self.shoot(), 
            _ => {}
        }
    }

    
    pub fn handle_key_release(&mut self, key: u8) {
        use crate::keyboard::{T_, S_, AI_, AJ_};
        match key {
            b'w' | b'W' | T_ => self.move_forward = false,
            b's' | b'S' | S_ => self.move_back = false,
            b'a' | b'A' => self.strafe_left = false,
            b'd' | b'D' => self.strafe_right = false,
            AI_ => self.turn_left = false,
            AJ_ => self.turn_right = false,
            _ => {}
        }
    }

    
    fn try_interact(&mut self) {
        
        let flj = self.player_x + self.player_angle.cos() * 1.2;
        let flk = self.player_y + self.player_angle.sin() * 1.2;
        let cg = flj as usize;
        let cr = flk as usize;

        if cg < EH_ && cr < EG_ {
            match self.map[cr][cg] {
                AKQ_ => {
                    if self.has_keycard {
                        self.map[cr][cg] = QZ_;
                        self.message = Some((alloc::string::String::from("Door opened!"), 90));
                    } else {
                        self.message = Some((alloc::string::String::from("Need keycard!"), 90));
                    }
                }
                RA_ => {
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

    
    pub fn tick(&mut self) {
        if self.game_won || self.game_over {
            return;
        }

        self.frame += 1;

        
        let jou = 0.06;
        if self.turn_left { self.player_angle -= jou; }
        if self.turn_right { self.player_angle += jou; }

        
        let cbt = 0.06;
        let vg = self.player_angle.cos();
        let vt = self.player_angle.sin();
        let mut dx = 0.0f32;
        let mut ad = 0.0f32;

        if self.move_forward { dx += vg * cbt; ad += vt * cbt; }
        if self.move_back { dx -= vg * cbt; ad -= vt * cbt; }
        if self.strafe_left { dx += vt * cbt; ad -= vg * cbt; }
        if self.strafe_right { dx -= vt * cbt; ad += vg * cbt; }

        
        let oq = 0.25;
        let cbw = self.player_x + dx;
        let afk = self.player_y + ad;

        
        if !self.is_wall(cbw + oq * dx.signum(), self.player_y) {
            self.player_x = cbw;
        }
        
        if !self.is_wall(self.player_x, afk + oq * ad.signum()) {
            self.player_y = afk;
        }

        
        self.check_pickups();

        
        self.update_enemies();

        
        if self.shoot_cooldown > 0 { self.shoot_cooldown -= 1; }
        if self.shoot_flash > 0 { self.shoot_flash -= 1; }

        
        if self.move_forward || self.move_back || self.strafe_left || self.strafe_right {
            self.weapon_bob += 0.12;
        } else {
            self.weapon_bob *= 0.9; 
        }

        
        if self.player_health <= 0 {
            self.game_over = true;
            self.message = Some((alloc::string::String::from("YOU DIED"), 9999));
        }

        
        let cg = self.player_x as usize;
        let cr = self.player_y as usize;
        if cg < EH_ && cr < EG_ && self.map[cr][cg] == RA_ {
            self.try_interact();
        }

        
        if self.flash_timer > 0 { self.flash_timer -= 1; }
        if self.pickup_flash_timer > 0 { self.pickup_flash_timer -= 1; }
        if let Some((_, ref mut frames)) = self.message {
            if *frames > 0 { *frames -= 1; }
            else { self.message = None; }
        }
    }

    #[inline]
    fn is_wall(&self, x: f32, y: f32) -> bool {
        let cg = x as usize;
        let cr = y as usize;
        if cg >= EH_ || cr >= EG_ { return true; }
        self.map[cr][cg] != QZ_
    }

    fn check_pickups(&mut self) {
        for item in &mut self.items {
            if item.collected { continue; }
            let dx = item.x - self.player_x;
            let ad = item.y - self.player_y;
            if dx * dx + ad * ad < 0.5 {
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

    
    fn shoot(&mut self) {
        if self.shoot_cooldown > 0 { return; }
        self.shoot_cooldown = 15; 
        self.shoot_flash = 4;    

        
        let coy = self.player_angle.cos();
        let coz = self.player_angle.sin();
        let mut da = self.player_x;
        let mut cm = self.player_y;
        let step = 0.1;

        for _ in 0..120 { 
            da += coy * step;
            cm += coz * step;

            
            let cg = da as usize;
            let cr = cm as usize;
            if cg >= EH_ || cr >= EG_ { break; }
            if self.map[cr][cg] != QZ_ { break; }

            
            for enemy in &mut self.enemies {
                if enemy.state == EnemyState::Dead { continue; }
                let edx = enemy.x - da;
                let hvb = enemy.y - cm;
                if edx * edx + hvb * hvb < 0.3 {
                    
                    enemy.health -= 25;
                    if enemy.health <= 0 {
                        enemy.state = EnemyState::Dead;
                        self.kills += 1;
                        self.player_score += 200;
                        self.message = Some((alloc::string::String::from("Enemy eliminated!"), 60));
                    } else {
                        enemy.state = EnemyState::Chasing; 
                        self.message = Some((alloc::string::String::from("Hit!"), 30));
                    }
                    return;
                }
            }
        }
    }

    
    fn update_enemies(&mut self) {
        let p = self.player_x;
        let o = self.player_y;

        for i in 0..self.enemies.len() {
            if self.enemies[i].state == EnemyState::Dead { continue; }

            let ajq = self.enemies[i].x;
            let qz = self.enemies[i].y;
            let dx = p - ajq;
            let ad = o - qz;
            let em = (dx * dx + ad * ad).sqrt();

            
            if em < self.enemies[i].attack_range {
                self.enemies[i].state = EnemyState::Attacking;
            } else if em < self.enemies[i].sight_range {
                self.enemies[i].state = EnemyState::Chasing;
            } else if self.enemies[i].state == EnemyState::Chasing {
                
                self.enemies[i].state = EnemyState::Idle;
            }

            match self.enemies[i].state {
                EnemyState::Chasing => {
                    
                    if em > 0.1 {
                        let nx = dx / em * self.enemies[i].speed;
                        let re = ad / em * self.enemies[i].speed;
                        let ipv = self.enemies[i].x + nx;
                        let ipw = self.enemies[i].y + re;
                        
                        if !self.is_wall(ipv, ipw) {
                            self.enemies[i].x = ipv;
                            self.enemies[i].y = ipw;
                        }
                    }
                }
                EnemyState::Attacking => {
                    if self.enemies[i].attack_cooldown == 0 {
                        
                        self.player_health -= self.enemies[i].damage;
                        self.flash_timer = 10;
                        self.enemies[i].attack_cooldown = 45; 
                    }
                }
                _ => {}
            }

            
            if self.enemies[i].attack_cooldown > 0 {
                self.enemies[i].attack_cooldown -= 1;
            }
        }
    }

    
    
    

    
    pub fn render(&mut self, buf: &mut [u32], w: usize, h: usize) {
        if w < 80 || h < 60 { return; }

        let mmq = 40; 
        let aak = h.saturating_sub(mmq);

        
        #[cfg(target_arch = "x86_64")]
        unsafe {
            crate::graphics::simd::adq(buf.as_mut_ptr(), w * h, 0xFF000000);
        }
        #[cfg(not(target_arch = "x86_64"))]
        {
            buf[..w * h].fill(0xFF000000);
        }

        
        self.z_buffer.clear();
        self.z_buffer.resize(w, f32::MAX);

        
        self.rebuild_ray_table(w);

        
        self.render_floor_ceiling(buf, w, aak);

        
        self.render_walls(buf, w, aak);

        
        self.render_enemies(buf, w, aak);

        
        self.render_items(buf, w, aak);

        
        self.render_crosshair(buf, w, aak);

        
        self.render_weapon(buf, w, aak);

        
        self.render_minimap(buf, w, h);

        
        self.render_hud(buf, w, h, aak);

        
        if self.flash_timer > 0 {
            let alpha = (self.flash_timer as u32 * 8).min(80);
            for i in 0..w * aak {
                let r = ((buf[i] >> 16) & 0xFF).saturating_add(alpha);
                buf[i] = (buf[i] & 0xFF00FFFF) | (r.min(255) << 16);
            }
        }

        
        if self.pickup_flash_timer > 0 {
            let alpha = (self.pickup_flash_timer as u32 * 5).min(40);
            for i in 0..w * aak {
                let g = ((buf[i] >> 8) & 0xFF).saturating_add(alpha);
                buf[i] = (buf[i] & 0xFFFF00FF) | (g.min(255) << 8);
            }
        }

        
        if self.game_won || self.game_over {
            
            for i in 0..w * h {
                let r = ((buf[i] >> 16) & 0xFF) / 3;
                let g = ((buf[i] >> 8) & 0xFF) / 3;
                let b = (buf[i] & 0xFF) / 3;
                buf[i] = 0xFF000000 | (r << 16) | (g << 8) | b;
            }
        }

        
        if let Some((ref bk, _)) = self.message {
            self.draw_text_centered(buf, w, aak / 4, bk, TD_);
        }
    }

    fn render_floor_ceiling(&self, buf: &mut [u32], w: usize, h: usize) {
        let cw = h / 2;

        for y in 0..h {
            let color = if y < cw {
                
                let t = y as u32 * 20 / cw as u32;
                let g = 8u32 + t;
                0xFF000000 | ((g / 3) << 16) | (g << 8) | (g / 4)
            } else {
                
                let em = (y - cw) as u32;
                let ggv = cw as u32;
                let t = em * 25 / ggv.max(1);
                let base = 6u32 + t;
                0xFF000000 | ((base / 2) << 16) | ((base / 2) << 8) | (base / 2)
            };
            
            #[cfg(target_arch = "x86_64")]
            unsafe {
                crate::graphics::simd::adq(
                    buf.as_mut_ptr().add(y * w),
                    w,
                    color,
                );
            }
            #[cfg(not(target_arch = "x86_64"))]
            {
                for x in 0..w {
                    buf[y * w + x] = color;
                }
            }
        }
    }

    fn render_walls(&mut self, buf: &mut [u32], w: usize, h: usize) {
        let fov = core::f32::consts::FRAC_PI_3; 
        let kh = h as f32 / 2.0;

        for col in 0..w {
            
            let coy = self.ray_cos_table[col];
            let coz = self.ray_sin_table[col];

            
            let mut cmr = self.player_x as i32;
            let mut cms = self.player_y as i32;

            let fri = if coy.abs() < 1e-8 { 1e8 } else { (1.0 / coy).abs() };
            let frj = if coz.abs() < 1e-8 { 1e8 } else { (1.0 / coz).abs() };

            let (avf, mut side_dist_x) = if coy < 0.0 {
                (-1i32, (self.player_x - cmr as f32) * fri)
            } else {
                (1i32, ((cmr + 1) as f32 - self.player_x) * fri)
            };

            let (bwa, mut side_dist_y) = if coz < 0.0 {
                (-1i32, (self.player_y - cms as f32) * frj)
            } else {
                (1i32, ((cms + 1) as f32 - self.player_y) * frj)
            };

            
            let mut hit = false;
            let mut den = 0; 
            let mut hcd = BJG_;

            for _ in 0..64 {
                if side_dist_x < side_dist_y {
                    side_dist_x += fri;
                    cmr += avf;
                    den = 0;
                } else {
                    side_dist_y += frj;
                    cms += bwa;
                    den = 1;
                }

                if cmr < 0 || cms < 0 || cmr >= EH_ as i32 || cms >= EG_ as i32 {
                    break;
                }

                let apf = self.map[cms as usize][cmr as usize];
                if apf != QZ_ {
                    hit = true;
                    hcd = apf;
                    break;
                }
            }

            if !hit { continue; }

            
            let dcp = if den == 0 {
                (cmr as f32 - self.player_x + (1.0 - avf as f32) / 2.0) / coy
            } else {
                (cms as f32 - self.player_y + (1.0 - bwa as f32) / 2.0) / coz
            };

            if dcp <= 0.0 { continue; }

            
            let fex = (h as f32 / dcp).min(h as f32 * 4.0);
            let lkv = ((kh - fex / 2.0) as i32).max(0) as usize;
            let liq = ((kh + fex / 2.0) as i32).min(h as i32 - 1) as usize;

            
            let avu = if den == 0 {
                self.player_y + dcp * coz
            } else {
                self.player_x + dcp * coy
            };
            let avu = avu - avu.floor();  
            let pia = (avu * AA_ as f32) as usize;

            
            let bdz = match hcd {
                BJG_ => &self.tex_brick,
                DBU_ => &self.tex_stone,
                DBT_ => &self.tex_metal,
                DBS_ => &self.tex_matrix,
                AKQ_ => &self.tex_door,
                RA_ => &self.tex_matrix,
                _ => &self.tex_brick,
            };

            
            let mrj = AA_ as f32 / fex;
            for y in lkv..=liq {
                let pib = ((y as f32 - (kh - fex / 2.0)) * mrj) as usize;
                let mut ct = bdz.sample(pia, pib);

                
                let aqm = (1.0 - (dcp / 12.0).min(1.0)).max(0.15);
                let r = (((ct >> 16) & 0xFF) as f32 * aqm) as u32;
                let g = (((ct >> 8) & 0xFF) as f32 * aqm) as u32;
                let b = ((ct & 0xFF) as f32 * aqm) as u32;
                ct = 0xFF000000 | (r << 16) | (g << 8) | b;

                
                if den == 1 {
                    let r = ((ct >> 16) & 0xFF) * 3 / 4;
                    let g = ((ct >> 8) & 0xFF) * 3 / 4;
                    let b = (ct & 0xFF) * 3 / 4;
                    ct = 0xFF000000 | (r << 16) | (g << 8) | b;
                }

                
                if hcd == RA_ {
                    let kq = ((self.frame as f32 * 0.1).sin() * 30.0 + 30.0) as u32;
                    let cyh = ((ct >> 8) & 0xFF).saturating_add(kq).min(255);
                    ct = (ct & 0xFFFF00FF) | (cyh << 8);
                }

                buf[y * w + col] = ct;
            }

            
            if col < self.z_buffer.len() {
                self.z_buffer[col] = dcp;
            }
        }
    }

    fn render_items(&self, buf: &mut [u32], w: usize, h: usize) {
        let kh = h as f32 / 2.0;

        for item in &self.items {
            if item.collected { continue; }

            
            let dx = item.x - self.player_x;
            let ad = item.y - self.player_y;

            
            let vg = self.player_angle.cos();
            let vt = self.player_angle.sin();
            let bu = dx * vg + ad * vt;
            let ty = -dx * vt + ad * vg;

            
            if ty < 0.2 { continue; }

            
            let fov = core::f32::consts::FRAC_PI_3;
            let lw = (0.5 + bu / (ty * (fov / 2.0).tan() * 2.0)) * w as f32;
            let fbh = (h as f32 / ty * 0.3) as i32;

            if fbh < 2 { continue; }

            let am = lw as i32 - fbh / 2;
            let ak = (kh as i32) - fbh / 2;

            let color = match item.item_type {
                ItemType::HealthPack => ABS_,
                ItemType::DataChip => AQG_,
                ItemType::KeyCard => 0xFFFFAA00,
            };

            
            let cw = fbh / 2;
            for dy_off in -cw..=cw {
                let row = ak + cw + dy_off;
                if row < 0 || row >= h as i32 { continue; }
                let auv = cw - dy_off.abs();
                for dx_off in -auv..=auv {
                    let cx = am + cw + dx_off;
                    if cx < 0 || cx >= w as i32 { continue; }

                    
                    let kq = ((self.frame as f32 * 0.15 + item.x * 3.0).sin() * 0.3 + 0.7) as f32;
                    let r = (((color >> 16) & 0xFF) as f32 * kq) as u32;
                    let g = (((color >> 8) & 0xFF) as f32 * kq) as u32;
                    let b = ((color & 0xFF) as f32 * kq) as u32;

                    buf[row as usize * w + cx as usize] = 0xFF000000 | (r << 16) | (g << 8) | b;
                }
            }
        }
    }

    fn render_enemies(&self, buf: &mut [u32], w: usize, h: usize) {
        let kh = h as f32 / 2.0;
        let fov = core::f32::consts::FRAC_PI_3;

        for enemy in &self.enemies {
            if enemy.state == EnemyState::Dead { continue; }

            
            let dx = enemy.x - self.player_x;
            let ad = enemy.y - self.player_y;

            
            let vg = self.player_angle.cos();
            let vt = self.player_angle.sin();
            let bu = dx * vg + ad * vt;
            let ty = -dx * vt + ad * vg;

            
            if ty < 0.3 { continue; }

            
            let lw = (0.5 + bu / (ty * (fov / 2.0).tan() * 2.0)) * w as f32;
            let ape = (h as f32 / ty * 0.6) as i32;
            let cqz = (ape as f32 * 0.5) as i32;

            if ape < 2 { continue; }

            let am = lw as i32 - cqz / 2;
            let ak = kh as i32 - ape / 2;

            
            let qf: u32 = match enemy.state {
                EnemyState::Idle => 0xFFCC2222,      
                EnemyState::Chasing => 0xFFFF3333,   
                EnemyState::Attacking => 0xFFFF6600,  
                EnemyState::Dead => continue,
            };

            
            let aqm = (1.0 - (ty / 12.0).min(1.0)).max(0.2);

            
            for dy_off in 0..ape {
                let row = ak + dy_off;
                if row < 0 || row >= h as i32 { continue; }

                
                let t = dy_off as f32 / ape as f32; 
                let hch = if t < 0.2 {
                    
                    0.4 + t
                } else if t < 0.7 {
                    
                    0.7
                } else {
                    
                    0.5
                };
                let jbm = (cqz as f32 * hch * 0.5) as i32;

                for dx_off in -jbm..=jbm {
                    let cx = am + cqz / 2 + dx_off;
                    if cx < 0 || cx >= w as i32 { continue; }

                    
                    if (cx as usize) < self.z_buffer.len() && ty >= self.z_buffer[cx as usize] {
                        continue; 
                    }

                    
                    if t > 0.75 && dx_off.abs() < 2 {
                        continue; 
                    }

                    
                    let r = (((qf >> 16) & 0xFF) as f32 * aqm) as u32;
                    let g = (((qf >> 8) & 0xFF) as f32 * aqm) as u32;
                    let b = ((qf & 0xFF) as f32 * aqm) as u32;
                    let ct = 0xFF000000 | (r << 16) | (g << 8) | b;

                    buf[row as usize * w + cx as usize] = ct;
                }
            }

            
            if ape > 8 {
                let atr = ak + ape / 8;
                let hxo = cqz / 6;
                for &ex_off in &[-hxo, hxo] {
                    let cje = am + cqz / 2 + ex_off;
                    if cje >= 0 && cje < w as i32 && atr >= 0 && atr < h as i32 {
                        if (cje as usize) < self.z_buffer.len() && ty < self.z_buffer[cje as usize] {
                            buf[atr as usize * w + cje as usize] = 0xFFFFFF00; 
                        }
                    }
                }
            }

            
            if enemy.health < enemy.max_health && ape > 10 {
                let gk = ak - 4;
                if gk >= 0 && gk < h as i32 {
                    let ek = cqz.min(20) as usize;
                    let rb = (enemy.health as f32 / enemy.max_health as f32 * ek as f32) as usize;
                    let egh = (am + cqz / 2 - ek as i32 / 2).max(0) as usize;
                    for bx in 0..ek {
                        let p = egh + bx;
                        if p >= w { break; }
                        if (p as usize) < self.z_buffer.len() && ty >= self.z_buffer[p] {
                            continue;
                        }
                        let color = if bx < rb { 0xFFFF0000 } else { 0xFF440000 };
                        buf[gk as usize * w + p] = color;
                    }
                }
            }
        }
    }

    fn render_crosshair(&self, buf: &mut [u32], w: usize, h: usize) {
        let cx = w / 2;
        let u = h / 2;
        let size = 4;
        let color = if self.shoot_flash > 0 { 0xFFFFFF00 } else { 0xAA00FF88 };

        
        for x in (cx.saturating_sub(size))..=(cx + size).min(w - 1) {
            if x != cx { 
                buf[u * w + x] = color;
            }
        }
        
        for y in (u.saturating_sub(size))..=(u + size).min(h - 1) {
            if y != u {
                buf[y * w + cx] = color;
            }
        }
        
        buf[u * w + cx] = if self.shoot_flash > 0 { 0xFFFFFFFF } else { 0xFF00FF88 };
    }

    fn render_weapon(&self, buf: &mut [u32], w: usize, h: usize) {
        
        let kcz = if self.move_forward || self.move_back || self.strafe_left || self.strafe_right {
            (self.weapon_bob.sin() * 3.0) as i32
        } else {
            0
        };

        let cgf = (w as i32 / 2 + 30) as usize;
        let bet = (h as i32 - 20 + kcz) as usize;

        
        let mgq = if self.shoot_flash > 0 { 0xFFFFDD44 } else { 0xFF666666 };
        let jzr = if self.shoot_flash > 0 { 0xFFFFFF88 } else { 0xFF444444 };

        
        for y in 0..3usize {
            for x in 0..12usize {
                let p = cgf + x;
                let o = bet.saturating_sub(8) + y;
                if p < w && o < h {
                    buf[o * w + p] = jzr;
                }
            }
        }
        
        for y in 0..8usize {
            for x in 0..6usize {
                let p = cgf + 3 + x;
                let o = bet.saturating_sub(5) + y;
                if p < w && o < h {
                    buf[o * w + p] = mgq;
                }
            }
        }

        
        if self.shoot_flash > 0 {
            let lwu = cgf + 12;
            let lwv = bet.saturating_sub(7);
            for ad in 0..5usize {
                for dx in 0..4usize {
                    let p = lwu + dx;
                    let o = lwv.saturating_sub(1) + ad;
                    if p < w && o < h {
                        buf[o * w + p] = 0xFFFFFF88;
                    }
                }
            }
        }
    }

    fn render_minimap(&self, buf: &mut [u32], w: usize, h: usize) {
        let cell = 5;
        let ilv = EH_ * cell;
        let nbz = EG_ * cell;
        let bny = w - ilv - 8;
        let bnz = 8;

        
        for y in 0..nbz + 4 {
            for x in 0..ilv + 4 {
                let p = bny - 2 + x;
                let o = bnz - 2 + y;
                if p < w && o < h {
                    buf[o * w + p] = 0xAA000000;
                }
            }
        }

        
        for cr in 0..EG_ {
            for cg in 0..EH_ {
                let color = match self.map[cr][cg] {
                    QZ_ => BPY_,
                    RA_ => BPZ_,
                    AKQ_ => 0xFF884400,
                    _ => BQA_,
                };
                for ad in 0..cell {
                    for dx in 0..cell {
                        let p = bny + cg * cell + dx;
                        let o = bnz + cr * cell + ad;
                        if p < w && o < h {
                            buf[o * w + p] = color;
                        }
                    }
                }
            }
        }

        
        let p = bny + (self.player_x * cell as f32) as usize;
        let o = bnz + (self.player_y * cell as f32) as usize;
        for ad in 0..3usize {
            for dx in 0..3usize {
                let x = p + dx;
                let y = o + ad;
                if x < w && y < h {
                    buf[y * w + x] = AQJ_;
                }
            }
        }

        
        let hsi = 6.0;
        let ajq = p as f32 + self.player_angle.cos() * hsi;
        let qz = o as f32 + self.player_angle.sin() * hsi;
        let steps = 8;
        for i in 0..steps {
            let t = i as f32 / steps as f32;
            let fe = (p as f32 + (ajq - p as f32) * t) as usize;
            let ly = (o as f32 + (qz - o as f32) * t) as usize;
            if fe < w && ly < h {
                buf[ly * w + fe] = AQJ_;
            }
        }

        
        for item in &self.items {
            if item.collected { continue; }
            let bi = bny + (item.x * cell as f32) as usize;
            let gg = bnz + (item.y * cell as f32) as usize;
            let epy = match item.item_type {
                ItemType::HealthPack => ABS_,
                ItemType::DataChip => AQG_,
                ItemType::KeyCard => 0xFFFFAA00,
            };
            if bi > 0 && bi + 1 < w && gg > 0 && gg + 1 < h {
                buf[gg * w + bi] = epy;
                buf[gg * w + bi + 1] = epy;
                buf[(gg + 1) * w + bi] = epy;
                buf[(gg + 1) * w + bi + 1] = epy;
            }
        }

        
        for enemy in &self.enemies {
            if enemy.state == EnemyState::Dead { continue; }
            let ajq = bny + (enemy.x * cell as f32) as usize;
            let qz = bnz + (enemy.y * cell as f32) as usize;
            let ec = 0xFFFF2222;
            if ajq > 0 && ajq + 1 < w && qz > 0 && qz + 1 < h {
                buf[qz * w + ajq] = ec;
                buf[qz * w + ajq + 1] = ec;
                buf[(qz + 1) * w + ajq] = ec;
                buf[(qz + 1) * w + ajq + 1] = ec;
            }
        }
    }

    fn render_hud(&self, buf: &mut [u32], w: usize, h: usize, aak: usize) {
        
        for y in aak..h {
            for x in 0..w {
                buf[y * w + x] = 0xFF0A120A;
            }
        }

        
        for x in 0..w {
            if aak < h {
                buf[aak * w + x] = TD_;
            }
        }

        let btf = aak + 4;
        let ceh = 1;

        
        self.draw_text_at(buf, w, h, 8, btf, "HP", ABT_);
        let pv = 28;
        let ek = 80;
        let hs = 10;
        
        for y in 0..hs {
            for x in 0..ek {
                let p = pv + x;
                let o = btf + 4 + y;
                if p < w && o < h {
                    buf[o * w + p] = 0xFF1A1A1A;
                }
            }
        }
        
        let rb = (self.player_health as usize * ek / 100).min(ek);
        let mmj = if self.player_health > 60 { ABS_ }
                       else if self.player_health > 30 { 0xFFAAAA00 }
                       else { AQD_ };
        for y in 0..hs {
            for x in 0..rb {
                let p = pv + x;
                let o = btf + 4 + y;
                if p < w && o < h {
                    buf[o * w + p] = mmj;
                }
            }
        }

        
        let dyq = format!("SCORE:{}", self.player_score);
        self.draw_text_at(buf, w, h, 120, btf, &dyq, TD_);

        
        let myd = format!("LVL:{}", self.current_level);
        self.draw_text_at(buf, w, h, 240, btf, &myd, ABT_);

        
        if self.has_keycard {
            self.draw_text_at(buf, w, h, 310, btf, "[KEY]", 0xFFFFAA00);
        }

        
        let mvt = format!("KILLS:{}", self.kills);
        self.draw_text_at(buf, w, h, 370, btf, &mvt, AQD_);

        
        let kwb = w - 60;
        let bfy = ["N", "E", "S", "W"];
        let jwd = (self.player_angle + core::f32::consts::PI * 2.0) % (core::f32::consts::PI * 2.0);
        let les = ((jwd + core::f32::consts::FRAC_PI_4) / core::f32::consts::FRAC_PI_2) as usize % 4;
        self.draw_text_at(buf, w, h, kwb, btf, bfy[les], TD_);

        
        self.draw_text_at(buf, w, h, 8, btf + 18, "WASD:Move Arrows:Turn E:Use Space:Shoot", ABT_);
    }

    
    
    

    fn draw_text_at(&self, buf: &mut [u32], w: usize, h: usize, x: usize, y: usize, text: &str, color: u32) {
        for (i, ch) in text.chars().enumerate() {
            let cx = x + i * 7;
            if cx + 6 >= w { break; }
            self.draw_char(buf, w, h, cx, y, ch, color);
        }
    }

    fn draw_text_centered(&self, buf: &mut [u32], w: usize, y: usize, text: &str, color: u32) {
        let acy = text.len() * 7;
        let x = if acy < w { (w - acy) / 2 } else { 0 };
        self.draw_text_at(buf, w, w * (w / w), x, y, text, color); 
        
        for (i, ch) in text.chars().enumerate() {
            let cx = x + i * 7;
            if cx + 6 >= w { break; }
            
            let aye = buf.len() / w;
            self.draw_char(buf, w, aye, cx, y, ch, color);
        }
    }

    fn draw_char(&self, buf: &mut [u32], w: usize, h: usize, x: usize, y: usize, ch: char, color: u32) {
        
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
                    let p = x + col;
                    let o = y + row;
                    if p < w && o < h {
                        buf[o * w + p] = color;
                    }
                }
            }
        }
    }
}
