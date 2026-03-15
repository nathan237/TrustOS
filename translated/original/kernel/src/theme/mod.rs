// ═══════════════════════════════════════════════════════════════════════════════
// TrustOS Dynamic Theme System
// ═══════════════════════════════════════════════════════════════════════════════
//
// Allows changing appearance without recompiling:
// - Load colors from /etc/theme.conf
// - Load wallpapers from /usr/share/wallpapers/
// - Hot-reload themes at runtime
//
// Theme Engine with security validation:
// - No pure white (#FFFFFF)
// - Contrast requirements (WCAG AA)
// - Controlled hue and saturation
//
// ═══════════════════════════════════════════════════════════════════════════════

use alloc::string::String;
use alloc::vec::Vec;
use spin::RwLock;

pub mod config;
pub mod bmp;

pub use config::*;
pub use bmp::*;

/// Global theme instance
pub static THEME: RwLock<Theme> = RwLock::new(Theme::default_const());

// ═══════════════════════════════════════════════════════════════════════════════
// THEME STRUCTURE
// ═══════════════════════════════════════════════════════════════════════════════

/// Complete theme definition
#[derive(Clone)]
pub struct Theme {
    pub name: String,
    pub colors: ThemeColors,
    pub taskbar: TaskbarConfig,
    pub window: WindowConfig,
    pub wallpaper: WallpaperConfig,
}

/// All theme colors
#[derive(Clone, Copy)]
pub struct ThemeColors {
    // Primary colors
    pub background: u32,
    pub foreground: u32,
    pub accent: u32,
    pub accent_hover: u32,
    pub accent_dark: u32,
    
    // Surface colors
    pub surface: u32,
    pub surface_hover: u32,
    pub surface_pressed: u32,
    
    // Text colors
    pub text_primary: u32,
    pub text_secondary: u32,
    pub text_disabled: u32,
    
    // Window colors
    pub titlebar_active: u32,
    pub titlebar_inactive: u32,
    pub border: u32,
    pub border_focused: u32,
    
    // Button colors
    pub btn_close: u32,
    pub btn_close_hover: u32,
    pub btn_maximize: u32,
    pub btn_minimize: u32,
    
    // Taskbar
    pub taskbar_bg: u32,
    pub taskbar_hover: u32,
    
    // Special
    pub shadow: u32,
    pub selection: u32,
    pub success: u32,
    pub warning: u32,
    pub error: u32,
}

/// Taskbar configuration
#[derive(Clone, Copy)]
pub struct TaskbarConfig {
    pub height: u32,
    pub position: TaskbarPosition,
    pub centered_icons: bool,
    pub show_clock: bool,
    pub show_date: bool,
    pub transparency: u8, // 0-255
}

#[derive(Clone, Copy, PartialEq)]
pub enum TaskbarPosition {
    Bottom,
    Top,
    Left,
    Right,
}

/// Window configuration
#[derive(Clone, Copy)]
pub struct WindowConfig {
    pub titlebar_height: u32,
    pub border_radius: u32,
    pub shadow_size: u32,
    pub shadow_opacity: u8,
    pub border_width: u32,
}

/// Wallpaper configuration
#[derive(Clone)]
pub struct WallpaperConfig {
    pub path: String,
    pub mode: WallpaperMode,
    pub fallback_color: u32,
    // Cached wallpaper data
    pub data: Option<WallpaperData>,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum WallpaperMode {
    Stretch,
    Center,
    Tile,
    Fill,
    Fit,
    Solid, // Just use fallback_color
}

/// Cached wallpaper pixels
#[derive(Clone)]
pub struct WallpaperData {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u32>, // ARGB pixels
}

// ═══════════════════════════════════════════════════════════════════════════════
// DEFAULT THEMES
// ═══════════════════════════════════════════════════════════════════════════════

impl Theme {
    /// Compile-time default (for static initialization)
    pub const fn default_const() -> Self {
        Self {
            name: String::new(),
            colors: ThemeColors::dark_green(),
            taskbar: TaskbarConfig::default_const(),
            window: WindowConfig::default_const(),
            wallpaper: WallpaperConfig::default_const(),
        }
    }
    
    /// Dark green theme (TrustOS default)
    pub fn dark_green() -> Self {
        Self {
            name: String::from("TrustOS Dark"),
            colors: ThemeColors::dark_green(),
            taskbar: TaskbarConfig::default(),
            window: WindowConfig::default(),
            wallpaper: WallpaperConfig::default(),
        }
    }
    
    /// Windows 11 dark theme
    pub fn windows11_dark() -> Self {
        Self {
            name: String::from("Windows 11 Dark"),
            colors: ThemeColors::windows11_dark(),
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
    
    /// Load theme from configuration file
    pub fn load_from_file(path: &str) -> Option<Self> {
        config::parse_theme_file(path)
    }
}

impl ThemeColors {
    /// TrustOS dark green theme
    pub const fn dark_green() -> Self {
        Self {
            // Primary
            background: 0xFF0A0E0B,
            foreground: 0xFFE0E0E0,
            accent: 0xFF00D26A,
            accent_hover: 0xFF00FF7F,
            accent_dark: 0xFF009950,
            
            // Surfaces
            surface: 0xFF141A16,
            surface_hover: 0xFF1E2620,
            surface_pressed: 0xFF0A0E0B,
            
            // Text
            text_primary: 0xFFE8E8E8,
            text_secondary: 0xFFA0A0A0,
            text_disabled: 0xFF606060,
            
            // Windows
            titlebar_active: 0xFF1A201C,
            titlebar_inactive: 0xFF141816,
            border: 0xFF2A3A30,
            border_focused: 0xFF00D26A,
            
            // Buttons
            btn_close: 0xFFE81123,
            btn_close_hover: 0xFFFF4D5E,
            btn_maximize: 0xFF00D26A,
            btn_minimize: 0xFF606060,
            
            // Taskbar
            taskbar_bg: 0xF0101410,
            taskbar_hover: 0xFF1E2620,
            
            // Special
            shadow: 0x40000000,
            selection: 0xFF00D26A,
            success: 0xFF00D26A,
            warning: 0xFFFFAA00,
            error: 0xFFE81123,
        }
    }
    
    /// Windows 11 dark theme colors
    pub const fn windows11_dark() -> Self {
        Self {
            // Primary
            background: 0xFF202020,
            foreground: 0xFFFFFFFF,
            accent: 0xFF0078D4,
            accent_hover: 0xFF1A8CD8,
            accent_dark: 0xFF005A9E,
            
            // Surfaces
            surface: 0xFF2D2D2D,
            surface_hover: 0xFF3D3D3D,
            surface_pressed: 0xFF1D1D1D,
            
            // Text
            text_primary: 0xFFFFFFFF,
            text_secondary: 0xFFAAAAAA,
            text_disabled: 0xFF666666,
            
            // Windows
            titlebar_active: 0xFF1F1F1F,
            titlebar_inactive: 0xFF2D2D2D,
            border: 0xFF3D3D3D,
            border_focused: 0xFF0078D4,
            
            // Buttons
            btn_close: 0xFFC42B1C,
            btn_close_hover: 0xFFE81123,
            btn_maximize: 0xFF666666,
            btn_minimize: 0xFF666666,
            
            // Taskbar
            taskbar_bg: 0xE61C1C1C,
            taskbar_hover: 0xFF3D3D3D,
            
            // Special
            shadow: 0x30000000,
            selection: 0xFF0078D4,
            success: 0xFF0F7B0F,
            warning: 0xFFCA5010,
            error: 0xFFC42B1C,
        }
    }
    
    /// Light theme
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
    pub const fn default_const() -> Self {
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
        Self::default_const()
    }
}

impl WindowConfig {
    pub const fn default_const() -> Self {
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
        Self::default_const()
    }
}

impl WallpaperConfig {
    pub const fn default_const() -> Self {
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

// ═══════════════════════════════════════════════════════════════════════════════
// PUBLIC API
// ═══════════════════════════════════════════════════════════════════════════════

/// Initialize theme system with default theme
pub fn init() {
    let mut theme = THEME.write();
    *theme = Theme::dark_green();
    crate::serial_println!("[THEME] Initialized with default TrustOS theme");
}

/// Load and apply theme from file
pub fn load_theme(path: &str) -> bool {
    if let Some(theme) = Theme::load_from_file(path) {
        let mut current = THEME.write();
        *current = theme;
        crate::serial_println!("[THEME] Loaded theme from {}", path);
        true
    } else {
        crate::serial_println!("[THEME] Failed to load theme from {}", path);
        false
    }
}

/// Switch to a built-in theme
pub fn set_builtin_theme(name: &str) {
    let mut current = THEME.write();
    match name {
        "dark" | "dark_green" | "trustos" => *current = Theme::dark_green(),
        "windows11" | "win11" | "fluent" => *current = Theme::windows11_dark(),
        _ => crate::serial_println!("[THEME] Unknown theme: {}", name),
    }
}

/// Get current theme colors (fast read access)
pub fn colors() -> ThemeColors {
    THEME.read().colors
}

/// Get taskbar config
pub fn taskbar() -> TaskbarConfig {
    THEME.read().taskbar
}

/// Get window config  
pub fn window() -> WindowConfig {
    THEME.read().window
}

/// Reload wallpaper from disk
pub fn reload_wallpaper() {
    let mut theme = THEME.write();
    if !theme.wallpaper.path.is_empty() {
        if let Some(data) = bmp::load_bmp(&theme.wallpaper.path) {
            theme.wallpaper.data = Some(data);
            crate::serial_println!("[THEME] Wallpaper loaded: {}", theme.wallpaper.path);
        }
    }
}
