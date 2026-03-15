















use alloc::string::String;
use alloc::vec::Vec;
use spin::RwLock;

pub mod config;
pub mod bmp;

pub use config::*;
pub use bmp::*;


pub static Ib: RwLock<Theme> = RwLock::new(Theme::eap());






#[derive(Clone)]
pub struct Theme {
    pub j: String,
    pub colors: ThemeColors,
    pub bou: TaskbarConfig,
    pub bh: WindowConfig,
    pub bsx: WallpaperConfig,
}


#[derive(Clone, Copy)]
pub struct ThemeColors {
    
    pub cop: u32,
    pub ivg: u32,
    pub mm: u32,
    pub cof: u32,
    pub iiq: u32,
    
    
    pub surface: u32,
    pub dwl: u32,
    pub fvv: u32,
    
    
    pub dcp: u32,
    pub dwr: u32,
    pub fwn: u32,
    
    
    pub idr: u32,
    pub jth: u32,
    pub acu: u32,
    pub dzc: u32,
    
    
    pub key: u32,
    pub kez: u32,
    pub kfb: u32,
    pub kfc: u32,
    
    
    pub ida: u32,
    pub mka: u32,
    
    
    pub zc: u32,
    pub gry: u32,
    pub vx: u32,
    pub ekt: u32,
    pub zt: u32,
}


#[derive(Clone, Copy)]
pub struct TaskbarConfig {
    pub ac: u32,
    pub qf: TaskbarPosition,
    pub gch: bool,
    pub iai: bool,
    pub jqa: bool,
    pub juc: u8, 
}

#[derive(Clone, Copy, PartialEq)]
pub enum TaskbarPosition {
    Hk,
    Jd,
    Ap,
    Ca,
}


#[derive(Clone, Copy)]
pub struct WindowConfig {
    pub ids: u32,
    pub avh: u32,
    pub iac: u32,
    pub dby: u8,
    pub dek: u32,
}


#[derive(Clone)]
pub struct WallpaperConfig {
    pub path: String,
    pub ev: WallpaperMode,
    pub hiv: u32,
    
    pub f: Option<Uy>,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum WallpaperMode {
    Uq,
    Eo,
    Azw,
    Bhc,
    Bhh,
    Aes, 
}


#[derive(Clone)]
pub struct Uy {
    pub z: u32,
    pub ac: u32,
    pub hz: Vec<u32>, 
}





impl Theme {
    
    pub const fn eap() -> Self {
        Self {
            j: String::new(),
            colors: ThemeColors::fgk(),
            bou: TaskbarConfig::eap(),
            bh: WindowConfig::eap(),
            bsx: WallpaperConfig::eap(),
        }
    }
    
    
    pub fn fgk() -> Self {
        Self {
            j: String::from("TrustOS Dark"),
            colors: ThemeColors::fgk(),
            bou: TaskbarConfig::default(),
            bh: WindowConfig::default(),
            bsx: WallpaperConfig::default(),
        }
    }
    
    
    pub fn jwx() -> Self {
        Self {
            j: String::from("Windows 11 Dark"),
            colors: ThemeColors::jwx(),
            bou: TaskbarConfig {
                ac: 48,
                qf: TaskbarPosition::Hk,
                gch: true,
                iai: true,
                jqa: true,
                juc: 200,
            },
            bh: WindowConfig {
                ids: 32,
                avh: 8,
                iac: 16,
                dby: 40,
                dek: 1,
            },
            bsx: WallpaperConfig::default(),
        }
    }
    
    
    pub fn ugv(path: &str) -> Option<Self> {
        config::vee(path)
    }
}

impl ThemeColors {
    
    pub const fn fgk() -> Self {
        Self {
            
            cop: 0xFF0A0E0B,
            ivg: 0xFFE0E0E0,
            mm: 0xFF00D26A,
            cof: 0xFF00FF7F,
            iiq: 0xFF009950,
            
            
            surface: 0xFF141A16,
            dwl: 0xFF1E2620,
            fvv: 0xFF0A0E0B,
            
            
            dcp: 0xFFE8E8E8,
            dwr: 0xFFA0A0A0,
            fwn: 0xFF606060,
            
            
            idr: 0xFF1A201C,
            jth: 0xFF141816,
            acu: 0xFF2A3A30,
            dzc: 0xFF00D26A,
            
            
            key: 0xFFE81123,
            kez: 0xFFFF4D5E,
            kfb: 0xFF00D26A,
            kfc: 0xFF606060,
            
            
            ida: 0xF0101410,
            mka: 0xFF1E2620,
            
            
            zc: 0x40000000,
            gry: 0xFF00D26A,
            vx: 0xFF00D26A,
            ekt: 0xFFFFAA00,
            zt: 0xFFE81123,
        }
    }
    
    
    pub const fn jwx() -> Self {
        Self {
            
            cop: 0xFF202020,
            ivg: 0xFFFFFFFF,
            mm: 0xFF0078D4,
            cof: 0xFF1A8CD8,
            iiq: 0xFF005A9E,
            
            
            surface: 0xFF2D2D2D,
            dwl: 0xFF3D3D3D,
            fvv: 0xFF1D1D1D,
            
            
            dcp: 0xFFFFFFFF,
            dwr: 0xFFAAAAAA,
            fwn: 0xFF666666,
            
            
            idr: 0xFF1F1F1F,
            jth: 0xFF2D2D2D,
            acu: 0xFF3D3D3D,
            dzc: 0xFF0078D4,
            
            
            key: 0xFFC42B1C,
            kez: 0xFFE81123,
            kfb: 0xFF666666,
            kfc: 0xFF666666,
            
            
            ida: 0xE61C1C1C,
            mka: 0xFF3D3D3D,
            
            
            zc: 0x30000000,
            gry: 0xFF0078D4,
            vx: 0xFF0F7B0F,
            ekt: 0xFFCA5010,
            zt: 0xFFC42B1C,
        }
    }
    
    
    pub const fn light() -> Self {
        Self {
            cop: 0xFFF3F3F3,
            ivg: 0xFF000000,
            mm: 0xFF0078D4,
            cof: 0xFF1A8CD8,
            iiq: 0xFF005A9E,
            
            surface: 0xFFFFFFFF,
            dwl: 0xFFE5E5E5,
            fvv: 0xFFCCCCCC,
            
            dcp: 0xFF000000,
            dwr: 0xFF666666,
            fwn: 0xFFAAAAAA,
            
            idr: 0xFFFFFFFF,
            jth: 0xFFF0F0F0,
            acu: 0xFFE0E0E0,
            dzc: 0xFF0078D4,
            
            key: 0xFFC42B1C,
            kez: 0xFFE81123,
            kfb: 0xFF666666,
            kfc: 0xFF666666,
            
            ida: 0xF0FFFFFF,
            mka: 0xFFE5E5E5,
            
            zc: 0x20000000,
            gry: 0xFF0078D4,
            vx: 0xFF0F7B0F,
            ekt: 0xFFCA5010,
            zt: 0xFFC42B1C,
        }
    }
}

impl TaskbarConfig {
    pub const fn eap() -> Self {
        Self {
            ac: 40,
            qf: TaskbarPosition::Hk,
            gch: false,
            iai: true,
            jqa: true,
            juc: 240,
        }
    }
}

impl Default for TaskbarConfig {
    fn default() -> Self {
        Self::eap()
    }
}

impl WindowConfig {
    pub const fn eap() -> Self {
        Self {
            ids: 28,
            avh: 4,
            iac: 8,
            dby: 30,
            dek: 1,
        }
    }
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self::eap()
    }
}

impl WallpaperConfig {
    pub const fn eap() -> Self {
        Self {
            path: String::new(),
            ev: WallpaperMode::Aes,
            hiv: 0xFF0A0E0B,
            f: None,
        }
    }
}

impl Default for WallpaperConfig {
    fn default() -> Self {
        Self {
            path: String::from("/usr/share/wallpapers/default.bmp"),
            ev: WallpaperMode::Uq,
            hiv: 0xFF0A0E0B,
            f: None,
        }
    }
}






pub fn init() {
    let mut theme = Ib.write();
    *theme = Theme::fgk();
    crate::serial_println!("[THEME] Initialized with default TrustOS theme");
}


pub fn uhi(path: &str) -> bool {
    if let Some(theme) = Theme::ugv(path) {
        let mut cv = Ib.write();
        *cv = theme;
        crate::serial_println!("[THEME] Loaded theme from {}", path);
        true
    } else {
        crate::serial_println!("[THEME] Failed to load theme from {}", path);
        false
    }
}


pub fn piq(j: &str) {
    let mut cv = Ib.write();
    match j {
        "dark" | "dark_green" | "trustos" => *cv = Theme::fgk(),
        "windows11" | "win11" | "fluent" => *cv = Theme::jwx(),
        _ => crate::serial_println!("[THEME] Unknown theme: {}", j),
    }
}


pub fn colors() -> ThemeColors {
    Ib.read().colors
}


pub fn bou() -> TaskbarConfig {
    Ib.read().bou
}


pub fn bh() -> WindowConfig {
    Ib.read().bh
}


pub fn vuq() {
    let mut theme = Ib.write();
    if !theme.bsx.path.is_empty() {
        if let Some(f) = bmp::jdu(&theme.bsx.path) {
            theme.bsx.f = Some(f);
            crate::serial_println!("[THEME] Wallpaper loaded: {}", theme.bsx.path);
        }
    }
}
