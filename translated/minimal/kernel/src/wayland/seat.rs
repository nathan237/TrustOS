



use alloc::string::String;
use alloc::vec::Vec;


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Amb {
    Released = 0,
    Pressed = 1,
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ButtonState {
    Released = 0,
    Pressed = 1,
}


#[derive(Debug, Clone, Copy, Default)]
pub struct ModifierState {
    pub depressed: u32,
    pub latched: u32,
    pub locked: u32,
    pub bbz: u32,
}


pub struct Aqd {
    
    pub name: String,
    
    
    pub capabilities: u32,
    
    
    pub pointer: PointerState,
    
    
    pub keyboard: KeyboardState,
    
    
    pub focused_surface: Option<u32>,
    
    
    serial: u32,
}

impl Aqd {
    pub fn new(name: &str) -> Self {
        Self {
            name: String::from(name),
            capabilities: 0b11, 
            pointer: PointerState::new(),
            keyboard: KeyboardState::new(),
            focused_surface: None,
            serial: 1,
        }
    }
    
    
    pub fn qpo(&mut self) -> u32 {
        let j = self.serial;
        self.serial = self.serial.wrapping_add(1);
        j
    }
    
    
    pub fn qwd(&mut self, avh: Option<u32>) {
        self.focused_surface = avh;
    }
    
    
    pub fn qkq(&self) -> bool {
        self.capabilities & 1 != 0
    }
    
    
    pub fn idr(&self) -> bool {
        self.capabilities & 2 != 0
    }
    
    
    pub fn qks(&self) -> bool {
        self.capabilities & 4 != 0
    }
}


pub struct PointerState {
    
    pub x: f64,
    
    
    pub y: f64,
    
    
    pub focus: Option<u32>,
    
    
    pub surface_x: f64,
    pub surface_y: f64,
    
    
    pub buttons: u32,
    
    
    pub cursor_surface: Option<u32>,
    pub cursor_hotspot_x: i32,
    pub cursor_hotspot_y: i32,
}

impl PointerState {
    pub fn new() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            focus: None,
            surface_x: 0.0,
            surface_y: 0.0,
            buttons: 0,
            cursor_surface: None,
            cursor_hotspot_x: 0,
            cursor_hotspot_y: 0,
        }
    }
    
    
    pub fn move_to(&mut self, x: f64, y: f64) {
        self.x = x;
        self.y = y;
    }
    
    
    pub fn qvl(&mut self, button: u32, pressed: bool) {
        if pressed {
            self.buttons |= 1 << button;
        } else {
            self.buttons &= !(1 << button);
        }
    }
    
    
    pub fn qme(&self, button: u32) -> bool {
        self.buttons & (1 << button) != 0
    }
    
    
    pub fn afr(&mut self, surface: Option<u32>, hotspot_x: i32, hotspot_y: i32) {
        self.cursor_surface = surface;
        self.cursor_hotspot_x = hotspot_x;
        self.cursor_hotspot_y = hotspot_y;
    }
}

impl Default for PointerState {
    fn default() -> Self {
        Self::new()
    }
}


pub struct KeyboardState {
    
    pub pressed_keys: Vec<u32>,
    
    
    pub modifiers: ModifierState,
    
    
    pub repeat_rate: i32,
    
    
    pub repeat_delay: i32,
    
    
    pub focus: Option<u32>,
}

impl KeyboardState {
    pub fn new() -> Self {
        Self {
            pressed_keys: Vec::new(),
            modifiers: ModifierState::default(),
            repeat_rate: 25,
            repeat_delay: 400,
            focus: None,
        }
    }
    
    
    pub fn qnd(&mut self, key: u32) {
        if !self.pressed_keys.contains(&key) {
            self.pressed_keys.push(key);
        }
    }
    
    
    pub fn qne(&mut self, key: u32) {
        self.pressed_keys.retain(|&k| k != key);
    }
    
    
    pub fn sx(&self, key: u32) -> bool {
        self.pressed_keys.contains(&key)
    }
    
    
    pub fn qwh(&mut self, depressed: u32, latched: u32, locked: u32, bbz: u32) {
        self.modifiers = ModifierState {
            depressed,
            latched,
            locked,
            bbz,
        };
    }
    
    
    pub fn clear(&mut self) {
        self.pressed_keys.clear();
    }
}

impl Default for KeyboardState {
    fn default() -> Self {
        Self::new()
    }
}






#[derive(Debug, Clone)]
pub struct Bav {
    pub time: u32,
    pub surface_x: f64,
    pub surface_y: f64,
}


#[derive(Debug, Clone)]
pub struct Bau {
    pub serial: u32,
    pub time: u32,
    pub button: u32,
    pub state: ButtonState,
}


#[derive(Debug, Clone)]
pub struct Bat {
    pub time: u32,
    pub ctt: u32, 
    pub value: f64,
}


#[derive(Debug, Clone)]
pub struct Ayg {
    pub serial: u32,
    pub time: u32,
    pub key: u32,
    pub state: Amb,
}


#[derive(Debug, Clone)]
pub struct Ayh {
    pub serial: u32,
    pub mods_depressed: u32,
    pub mods_latched: u32,
    pub mods_locked: u32,
    pub bbz: u32,
}
