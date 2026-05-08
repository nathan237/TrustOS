















use alloc::string::String;
use alloc::vec::Vec;
use spin::RwLock;

pub mod config;
pub mod bmp;

pub use config::*;
pub use bmp::*;


pub static Dj: RwLock<Theme> = RwLock::new(Theme::brt());






#[derive(Clone)]
pub struct Theme {
    pub name: String,
    pub colors: ThemeColors,
    pub taskbar: TaskbarConfig,
    pub window: WindowConfig,
    pub wallpaper: WallpaperConfig,
}


#[derive(Clone, Copy)]
pub struct ThemeColors {
    
    pub background: u32,
    pub foreground: u32,
    pub accent: u32,
    pub accent_hover: u32,
    pub accent_dark: u32,
    
    
    pub surface: u32,
    pub surface_hover: u32,
    pub surface_pressed: u32,
    
    
    pub text_primary: u32,
    pub text_secondary: u32,
    pub text_disabled: u32,
    
    
    pub titlebar_active: u32,
    pub titlebar_inactive: u32,
    pub border: u32,
    pub border_focused: u32,
    
    
    pub btn_close: u32,
    pub btn_close_hover: u32,
    pub btn_maximize: u32,
    pub btn_minimize: u32,
    
    
    pub taskbar_bg: u32,
    pub taskbar_hover: u32,
    
    
    pub shadow: u32,
    pub selection: u32,
    pub success: u32,
    pub warning: u32,
    pub error: u32,
}


#[derive(Clone, Copy)]
pub struct TaskbarConfig {
    pub height: u32,
    pub position: TaskbarPosition,
    pub centered_icons: bool,
    pub show_clock: bool,
    pub show_date: bool,
    pub transparency: u8, 
}

#[derive(Clone, Copy, PartialEq)]
pub enum TaskbarPosition {
    Bottom,
    Top,
    Left,
    Right,
}


#[derive(Clone, Copy)]
pub struct WindowConfig {
    pub titlebar_height: u32,
    pub border_radius: u32,
    pub shadow_size: u32,
    pub shadow_opacity: u8,
    pub border_width: u32,
}


#[derive(Clone)]
pub struct WallpaperConfig {
    pub path: String,
    pub mode: WallpaperMode,
    pub fallback_color: u32,
    
    pub data: Option<Jg>,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum WallpaperMode {
    Stretch,
    Center,
    Tile,
    Fill,
    Fit,
    Solid, 
}


#[derive(Clone)]
pub struct Jg {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u32>, 
}





impl Theme {
    
    pub const fn brt() -> Self {
        Self {
            name: String::new(),
            colors: ThemeColors::cia(),
            taskbar: TaskbarConfig::brt(),
            window: WindowConfig::brt(),
            wallpaper: WallpaperConfig::brt(),
        }
    }
    
    
    pub fn cia() -> Self {
        Self {
            name: String::from("TrustOS Dark"),
            colors: ThemeColors::cia(),
            taskbar: TaskbarConfig::default(),
            window: WindowConfig::default(),
            wallpaper: WallpaperConfig::default(),
        }
    }
    
    
    pub fn ffg() -> Self {
        Self {
            name: String::from("Windows 11 Dark"),
            colors: ThemeColors::ffg(),
            taskbar: TaskbarConfig {
                height: 48,
                position: TaskbarPosition::Bottom,
                centered_icons: true,
                show_clock: true,
                show_date: true,
                transparency: 200,
            },
            window: WindowConfig {
                titlebar_height: 32,
                border_radius: 8,
                shadow_size: 16,
                shadow_opacity: 40,
                border_width: 1,
            },
            wallpaper: WallpaperConfig::default(),
        }
    }
    
    
    pub fn naa(path: &str) -> Option<Self> {
        config::nrj(path)
    }
}

impl ThemeColors {
    
    pub const fn cia() -> Self {
        Self {
            
            background: 0xFF0A0E0B,
            foreground: 0xFFE0E0E0,
            accent: 0xFF00D26A,
            accent_hover: 0xFF00FF7F,
            accent_dark: 0xFF009950,
            
            
            surface: 0xFF141A16,
            surface_hover: 0xFF1E2620,
            surface_pressed: 0xFF0A0E0B,
            
            
            text_primary: 0xFFE8E8E8,
            text_secondary: 0xFFA0A0A0,
            text_disabled: 0xFF606060,
            
            
            titlebar_active: 0xFF1A201C,
            titlebar_inactive: 0xFF141816,
            border: 0xFF2A3A30,
            border_focused: 0xFF00D26A,
            
            
            btn_close: 0xFFE81123,
            btn_close_hover: 0xFFFF4D5E,
            btn_maximize: 0xFF00D26A,
            btn_minimize: 0xFF606060,
            
            
            taskbar_bg: 0xF0101410,
            taskbar_hover: 0xFF1E2620,
            
            
            shadow: 0x40000000,
            selection: 0xFF00D26A,
            success: 0xFF00D26A,
            warning: 0xFFFFAA00,
            error: 0xFFE81123,
        }
    }
    
    
    pub const fn ffg() -> Self {
        Self {
            
            background: 0xFF202020,
            foreground: 0xFFFFFFFF,
            accent: 0xFF0078D4,
            accent_hover: 0xFF1A8CD8,
            accent_dark: 0xFF005A9E,
            
            
            surface: 0xFF2D2D2D,
            surface_hover: 0xFF3D3D3D,
            surface_pressed: 0xFF1D1D1D,
            
            
            text_primary: 0xFFFFFFFF,
            text_secondary: 0xFFAAAAAA,
            text_disabled: 0xFF666666,
            
            
            titlebar_active: 0xFF1F1F1F,
            titlebar_inactive: 0xFF2D2D2D,
            border: 0xFF3D3D3D,
            border_focused: 0xFF0078D4,
            
            
            btn_close: 0xFFC42B1C,
            btn_close_hover: 0xFFE81123,
            btn_maximize: 0xFF666666,
            btn_minimize: 0xFF666666,
            
            
            taskbar_bg: 0xE61C1C1C,
            taskbar_hover: 0xFF3D3D3D,
            
            
            shadow: 0x30000000,
            selection: 0xFF0078D4,
            success: 0xFF0F7B0F,
            warning: 0xFFCA5010,
            error: 0xFFC42B1C,
        }
    }
    
    
    pub const fn light() -> Self {
        Self {
            background: 0xFFF3F3F3,
            foreground: 0xFF000000,
            accent: 0xFF0078D4,
            accent_hover: 0xFF1A8CD8,
            accent_dark: 0xFF005A9E,
            
            surface: 0xFFFFFFFF,
            surface_hover: 0xFFE5E5E5,
            surface_pressed: 0xFFCCCCCC,
            
            text_primary: 0xFF000000,
            text_secondary: 0xFF666666,
            text_disabled: 0xFFAAAAAA,
            
            titlebar_active: 0xFFFFFFFF,
            titlebar_inactive: 0xFFF0F0F0,
            border: 0xFFE0E0E0,
            border_focused: 0xFF0078D4,
            
            btn_close: 0xFFC42B1C,
            btn_close_hover: 0xFFE81123,
            btn_maximize: 0xFF666666,
            btn_minimize: 0xFF666666,
            
            taskbar_bg: 0xF0FFFFFF,
            taskbar_hover: 0xFFE5E5E5,
            
            shadow: 0x20000000,
            selection: 0xFF0078D4,
            success: 0xFF0F7B0F,
            warning: 0xFFCA5010,
            error: 0xFFC42B1C,
        }
    }
}

impl TaskbarConfig {
    pub const fn brt() -> Self {
        Self {
            height: 40,
            position: TaskbarPosition::Bottom,
            centered_icons: false,
            show_clock: true,
            show_date: true,
            transparency: 240,
        }
    }
}

impl Default for TaskbarConfig {
    fn default() -> Self {
        Self::brt()
    }
}

impl WindowConfig {
    pub const fn brt() -> Self {
        Self {
            titlebar_height: 28,
            border_radius: 4,
            shadow_size: 8,
            shadow_opacity: 30,
            border_width: 1,
        }
    }
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self::brt()
    }
}

impl WallpaperConfig {
    pub const fn brt() -> Self {
        Self {
            path: String::new(),
            mode: WallpaperMode::Solid,
            fallback_color: 0xFF0A0E0B,
            data: None,
        }
    }
}

impl Default for WallpaperConfig {
    fn default() -> Self {
        Self {
            path: String::from("/usr/share/wallpapers/default.bmp"),
            mode: WallpaperMode::Stretch,
            fallback_color: 0xFF0A0E0B,
            data: None,
        }
    }
}






pub fn init() {
    let mut theme = Dj.write();
    *theme = Theme::cia();
    crate::serial_println!("[THEME] Initialized with default TrustOS theme");
}


pub fn nag(path: &str) -> bool {
    if let Some(theme) = Theme::naa(path) {
        let mut current = Dj.write();
        *current = theme;
        crate::serial_println!("[THEME] Loaded theme from {}", path);
        true
    } else {
        crate::serial_println!("[THEME] Failed to load theme from {}", path);
        false
    }
}


pub fn jex(name: &str) {
    let mut current = Dj.write();
    match name {
        "dark" | "dark_green" | "trustos" => *current = Theme::cia(),
        "windows11" | "win11" | "fluent" => *current = Theme::ffg(),
        _ => crate::serial_println!("[THEME] Unknown theme: {}", name),
    }
}


pub fn colors() -> ThemeColors {
    Dj.read().colors
}


pub fn taskbar() -> TaskbarConfig {
    Dj.read().taskbar
}


pub fn window() -> WindowConfig {
    Dj.read().window
}


pub fn oes() {
    let mut theme = Dj.write();
    if !theme.wallpaper.path.is_empty() {
        if let Some(data) = bmp::ete(&theme.wallpaper.path) {
            theme.wallpaper.data = Some(data);
            crate::serial_println!("[THEME] Wallpaper loaded: {}", theme.wallpaper.path);
        }
    }
}
