//! GUI Window Manager
//! 
//! Modern windowing system with double-buffered rendering,
//! taskbar, mouse cursor, context menus, and scroll wheel support.
//! 
//! Design: TrustOS Modern Dark Theme
//! Inspired by: macOS, Windows 11, Terminal apps
//! 
//! Now using the TrustOS Graphics Engine for optimized rendering

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use spin::Mutex;
use crate::framebuffer::{self, COLOR_GREEN, COLOR_BRIGHT_GREEN, COLOR_DARK_GREEN, COLOR_WHITE, COLOR_BLACK};
use crate::graphics::desktop_gfx;

// ═══════════════════════════════════════════════════════════════════════════════
// 🧮 MATH UTILITIES for no_std environment
// ═══════════════════════════════════════════════════════════════════════════════

/// Fast approximate square root using Newton-Raphson method
fn fast_sqrt(x: f32) -> f32 {
    if x <= 0.0 { return 0.0; }
    let mut guess = x / 2.0;
    for _ in 0..5 {
        guess = (guess + x / guess) / 2.0;
    }
    guess
}

/// Approximate x^2
#[allow(dead_code)]
fn square(x: f32) -> f32 {
    x * x
}

// ═══════════════════════════════════════════════════════════════════════════════
// 🎨 TRUSTOS OFFICIAL PALETTE - Matrix-like professional dark theme
// ═══════════════════════════════════════════════════════════════════════════════
// STRICT COMPLIANCE: No pure white (#FFFFFF), green accent hierarchy
// ═══════════════════════════════════════════════════════════════════════════════

// Primary backgrounds
const BG_DEEPEST: u32 = 0xFF050606;          // Deepest black
const BG_DARK: u32 = 0xFF070B09;             // Panel background
const BG_MEDIUM: u32 = 0xFF0A0F0C;           // Window background
const BG_LIGHT: u32 = 0xFF0D1310;            // Alternate/hover

// Green hierarchy (stratified)
const GREEN_PRIMARY: u32 = 0xFF00FF66;       // Bright accent, focus
const GREEN_SECONDARY: u32 = 0xFF00CC55;     // Normal text, icons
const GREEN_TERTIARY: u32 = 0xFF00AA44;      // Intermediate
const GREEN_MUTED: u32 = 0xFF008844;         // Borders, separators  
const GREEN_SUBTLE: u32 = 0xFF006633;        // Subtle elements
const GREEN_GHOST: u32 = 0xFF003B1A;         // Shadows, grid

// Status colors (minimal use)
const ACCENT_AMBER: u32 = 0xFFFFD166;        // Warnings
const ACCENT_RED: u32 = 0xFFFF5555;          // Errors (rare)
const ACCENT_BLUE: u32 = 0xFF4ECDC4;         // Info/links (minimal)

// UI Elements
const WINDOW_BG: u32 = 0xFF0A0F0C;
const WINDOW_CONTENT_BG: u32 = 0xFF070B09;
const TITLE_BAR_ACTIVE: u32 = 0xFF080C09;
const TITLE_BAR_INACTIVE: u32 = 0xFF060908;
const TITLE_BAR_ACCENT: u32 = GREEN_MUTED;

// Dock (left sidebar)
const DOCK_BG: u32 = 0xFF060908;
const DOCK_ITEM_HOVER: u32 = 0xFF0D1310;
const DOCK_ITEM_ACTIVE: u32 = 0xFF101815;

// Context menu
const MENU_BG: u32 = 0xFF080C09;
const MENU_HOVER: u32 = 0xFF0D1310;
const MENU_BORDER: u32 = 0xFF1A2A20;
const MENU_SEPARATOR: u32 = 0xFF1A2A20;

// Window control buttons (desaturated - professional)
const BTN_CLOSE: u32 = 0xFF3A2828;
const BTN_CLOSE_HOVER: u32 = 0xFFFF5555;
const BTN_MAXIMIZE: u32 = 0xFF2A2A20;
const BTN_MAXIMIZE_HOVER: u32 = 0xFFFFD166;
const BTN_MINIMIZE: u32 = 0xFF283028;
const BTN_MINIMIZE_HOVER: u32 = 0xFF00CC55;

// Text colors
const TEXT_PRIMARY: u32 = 0xFFE0E8E4;
const TEXT_SECONDARY: u32 = 0xFF8A9890;
const TEXT_ACCENT: u32 = 0xFF00CC55;

// Legacy aliases for compatibility
const TASKBAR_BG: u32 = DOCK_BG;
const TASKBAR_ACCENT: u32 = DOCK_ITEM_HOVER;
const BUTTON_RED: u32 = BTN_CLOSE;
const BUTTON_YELLOW: u32 = BTN_MAXIMIZE;
const BUTTON_GREEN_BTN: u32 = BTN_MINIMIZE;
const CONTEXT_MENU_BG: u32 = MENU_BG;
const CONTEXT_MENU_HOVER: u32 = MENU_HOVER;
const CONTEXT_MENU_BORDER: u32 = MENU_BORDER;
const DESKTOP_BG_TOP: u32 = BG_DEEPEST;
const DESKTOP_BG_BOTTOM: u32 = 0xFF020303;

// Layout constants (official spec)
const TASKBAR_HEIGHT: u32 = 40;
const TITLE_BAR_HEIGHT: u32 = 28;             // Official: 28px
const WINDOW_BORDER_RADIUS: u32 = 6;          // Official: 6px rounded corners
const WINDOW_SHADOW_BLUR: u32 = 12;
const DOCK_ICON_SIZE: u32 = 24;               // Official icon size
const DOCK_WIDTH: u32 = 60;                   // Official: 56-64px

// Animation state (minimal - no flashy effects)
const FADE_STEPS: u8 = 8;

/// Window ID counter
static NEXT_WINDOW_ID: Mutex<u32> = Mutex::new(1);

/// Context menu item
#[derive(Clone)]
pub struct ContextMenuItem {
    pub label: String,
    pub action: ContextAction,
}

/// Context menu actions
#[derive(Clone, Copy, PartialEq)]
pub enum ContextAction {
    Open,
    OpenWith,
    Delete,
    Rename,
    Properties,
    Refresh,
    NewFile,
    NewFolder,
    CopyPath,
    Cut,
    Copy,
    Paste,
    ViewLargeIcons,
    ViewSmallIcons,
    ViewList,
    SortByName,
    SortByDate,
    SortBySize,
    Personalize,
    TerminalHere,
    Cancel,
}

/// Context menu structure
#[derive(Clone)]
pub struct ContextMenu {
    pub visible: bool,
    pub x: i32,
    pub y: i32,
    pub items: Vec<ContextMenuItem>,
    pub selected_index: usize,
    pub target_icon: Option<usize>,  // Index of icon right-clicked
    pub target_file: Option<String>, // File path right-clicked
}

/// Desktop icon structure
#[derive(Clone)]
pub struct DesktopIcon {
    pub name: String,
    pub icon_type: crate::icons::IconType,
    pub x: u32,
    pub y: u32,
    pub action: IconAction,
}

#[derive(Clone, Copy, PartialEq)]
pub enum IconAction {
    OpenTerminal,
    OpenFileManager,
    OpenSettings,
    OpenAbout,
    OpenCalculator,
    OpenNetwork,
    OpenGame,
    OpenEditor,
    OpenGL3D,
}

/// Window type for content
#[derive(Clone, Copy, PartialEq)]
pub enum WindowType {
    Terminal,
    SystemInfo,
    About,
    Empty,
    Calculator,
    FileManager,
    TextEditor,
    NetworkInfo,
    Settings,
    ImageViewer,
    HexViewer,
    FileAssociations,
    Demo3D,  // New: 3D graphics demo
    Game,    // Snake game
}

/// Window structure
#[derive(Clone)]
pub struct Window {
    pub id: u32,
    pub title: String,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub min_width: u32,
    pub min_height: u32,
    pub visible: bool,
    pub focused: bool,
    pub minimized: bool,
    pub maximized: bool,
    pub dragging: bool,
    pub resizing: ResizeEdge,
    pub drag_offset_x: i32,
    pub drag_offset_y: i32,
    // Saved position before maximize
    pub saved_x: i32,
    pub saved_y: i32,
    pub saved_width: u32,
    pub saved_height: u32,
    pub window_type: WindowType,
    pub content: Vec<String>,
    pub file_path: Option<String>,
    pub selected_index: usize,
    pub scroll_offset: usize,
}

/// Resize edge being dragged
#[derive(Clone, Copy, PartialEq)]
pub enum ResizeEdge {
    None,
    Right,
    Bottom,
    BottomRight,
}

impl Window {
    pub fn new(title: &str, x: i32, y: i32, width: u32, height: u32, wtype: WindowType) -> Self {
        let mut id_lock = NEXT_WINDOW_ID.lock();
        let id = *id_lock;
        *id_lock += 1;
        
        Window {
            id,
            title: String::from(title),
            x,
            y,
            width,
            height,
            min_width: 200,
            min_height: 150,
            visible: true,
            focused: false,
            minimized: false,
            maximized: false,
            dragging: false,
            resizing: ResizeEdge::None,
            drag_offset_x: 0,
            drag_offset_y: 0,
            saved_x: x,
            saved_y: y,
            saved_width: width,
            saved_height: height,
            window_type: wtype,
            content: Vec::new(),
            file_path: None,
            selected_index: 0,
            scroll_offset: 0,
        }
    }
    
    /// Check if point is inside window
    pub fn contains(&self, px: i32, py: i32) -> bool {
        if self.minimized { return false; }
        px >= self.x && px < self.x + self.width as i32 &&
        py >= self.y && py < self.y + self.height as i32
    }
    
    /// Check if point is in title bar
    pub fn in_title_bar(&self, px: i32, py: i32) -> bool {
        px >= self.x && px < self.x + self.width as i32 - 60 &&
        py >= self.y && py < self.y + TITLE_BAR_HEIGHT as i32
    }
    
    /// Check if point is on close button
    pub fn on_close_button(&self, px: i32, py: i32) -> bool {
        let btn_x = self.x + self.width as i32 - 24;
        px >= btn_x && px < btn_x + 18 &&
        py >= self.y + 5 && py < self.y + 23
    }
    
    /// Check if point is on maximize button
    pub fn on_maximize_button(&self, px: i32, py: i32) -> bool {
        let btn_x = self.x + self.width as i32 - 46;
        px >= btn_x && px < btn_x + 18 &&
        py >= self.y + 5 && py < self.y + 23
    }
    
    /// Check if point is on minimize button
    pub fn on_minimize_button(&self, px: i32, py: i32) -> bool {
        let btn_x = self.x + self.width as i32 - 68;
        px >= btn_x && px < btn_x + 18 &&
        py >= self.y + 5 && py < self.y + 23
    }
    
    /// Check if point is on resize edge
    pub fn on_resize_edge(&self, px: i32, py: i32) -> ResizeEdge {
        if self.maximized { return ResizeEdge::None; }
        
        let resize_margin = 8i32;
        let right_edge = self.x + self.width as i32;
        let bottom_edge = self.y + self.height as i32;
        
        let on_right = px >= right_edge - resize_margin && px < right_edge;
        let on_bottom = py >= bottom_edge - resize_margin && py < bottom_edge;
        
        if on_right && on_bottom {
            ResizeEdge::BottomRight
        } else if on_right {
            ResizeEdge::Right
        } else if on_bottom {
            ResizeEdge::Bottom
        } else {
            ResizeEdge::None
        }
    }
    
    /// Toggle maximize state
    pub fn toggle_maximize(&mut self, screen_width: u32, screen_height: u32) {
        if self.maximized {
            // Restore
            self.x = self.saved_x;
            self.y = self.saved_y;
            self.width = self.saved_width;
            self.height = self.saved_height;
            self.maximized = false;
        } else {
            // Save current position
            self.saved_x = self.x;
            self.saved_y = self.y;
            self.saved_width = self.width;
            self.saved_height = self.height;
            // Maximize
            self.x = 0;
            self.y = 0;
            self.width = screen_width;
            self.height = screen_height - TASKBAR_HEIGHT;
            self.maximized = true;
        }
    }
}

/// Desktop manager
use crate::graphics::{compositor, Compositor, CompositorTheme, WindowSurface, Easing};

// ═══════════════════════════════════════════════════════════════════════════════
// 🎨 RENDER MODE - Choose between classic and OpenGL compositor
// ═══════════════════════════════════════════════════════════════════════════════

/// Rendering backend for the desktop
#[derive(Clone, Copy, PartialEq)]
pub enum RenderMode {
    /// Classic framebuffer rendering (stable, fast)
    Classic,
    /// OpenGL compositor with effects (modern, customizable)
    OpenGL,
}

pub struct Desktop {
    pub windows: Vec<Window>,
    pub icons: Vec<DesktopIcon>,
    pub cursor_x: i32,
    pub cursor_y: i32,
    pub cursor_visible: bool,
    pub width: u32,
    pub height: u32,
    frame_count: u64,
    start_menu_open: bool,
    pub input_buffer: String,
    pub cursor_blink: bool,
    pub context_menu: ContextMenu,
    // Performance: cached background
    background_cached: bool,
    needs_full_redraw: bool,
    last_cursor_x: i32,
    last_cursor_y: i32,
    // Track state changes
    last_window_count: usize,
    last_start_menu_open: bool,
    last_context_menu_visible: bool,
    // OpenGL compositor
    pub render_mode: RenderMode,
    pub compositor_theme: CompositorTheme,
}

impl Desktop {
    pub const fn new() -> Self {
        Desktop {
            windows: Vec::new(),
            icons: Vec::new(),
            cursor_x: 640,
            cursor_y: 400,
            cursor_visible: true,
            width: 1280,
            height: 800,
            frame_count: 0,
            start_menu_open: false,
            input_buffer: String::new(),
            cursor_blink: true,
            context_menu: ContextMenu {
                visible: false,
                x: 0,
                y: 0,
                items: Vec::new(),
                selected_index: 0,
                target_icon: None,
                target_file: None,
            },
            background_cached: false,
            needs_full_redraw: true,
            last_cursor_x: 640,
            last_cursor_y: 400,
            last_window_count: 0,
            last_start_menu_open: false,
            last_context_menu_visible: false,
            render_mode: RenderMode::Classic,
            compositor_theme: CompositorTheme::Modern,
        }
    }
    
    /// Initialize desktop with double buffering
    pub fn init(&mut self, width: u32, height: u32) {
        crate::serial_println!("[Desktop] init start: {}x{}", width, height);
        self.width = width;
        self.height = height;
        self.cursor_x = (width / 2) as i32;
        self.cursor_y = (height / 2) as i32;
        
        // Initialize double buffering
        crate::serial_println!("[Desktop] init_double_buffer...");
        framebuffer::init_double_buffer();
        framebuffer::set_double_buffer_mode(true);
        
        // Initialize background cache for fast redraws
        crate::serial_println!("[Desktop] init_background_cache...");
        framebuffer::init_background_cache();
        
        // Initialize OpenGL compositor
        crate::serial_println!("[Desktop] init_compositor...");
        compositor::init_compositor(width, height);
        compositor::set_compositor_theme(self.compositor_theme);
        
        // Create desktop icons like Windows
        crate::serial_println!("[Desktop] init_desktop_icons...");
        self.init_desktop_icons();
        
        // Mark that we need to render background on first frame
        self.background_cached = false;
        self.needs_full_redraw = true;
        crate::serial_println!("[Desktop] init complete");
    }
    
    /// Set render mode (Classic or OpenGL)
    pub fn set_render_mode(&mut self, mode: RenderMode) {
        self.render_mode = mode;
        self.needs_full_redraw = true;
        self.background_cached = false;
        
        if mode == RenderMode::OpenGL {
            // Sync windows to compositor
            self.sync_compositor_surfaces();
        }
    }
    
    /// Set compositor visual theme
    pub fn set_theme(&mut self, theme: CompositorTheme) {
        self.compositor_theme = theme;
        compositor::set_compositor_theme(theme);
        self.needs_full_redraw = true;
    }
    
    /// Sync window state to compositor surfaces
    fn sync_compositor_surfaces(&self) {
        let mut comp = compositor::compositor();
        comp.surfaces.clear();
        
        for window in &self.windows {
            if window.visible {
                let mut surface = WindowSurface::new(
                    window.id,
                    window.x as f32,
                    window.y as f32,
                    window.width as f32,
                    window.height as f32,
                );
                surface.z_order = 0;
                surface.focused = window.focused;
                surface.visible = !window.minimized;
                comp.surfaces.push(surface);
            }
        }
    }
    
    /// Initialize desktop icons
    fn init_desktop_icons(&mut self) {
        use crate::icons::IconType;
        
        let icon_spacing = 80;
        let start_x = 20;
        let start_y = 20;
        
        self.icons.push(DesktopIcon {
            name: String::from("Terminal"),
            icon_type: IconType::Terminal,
            x: start_x,
            y: start_y,
            action: IconAction::OpenTerminal,
        });
        
        self.icons.push(DesktopIcon {
            name: String::from("Files"),
            icon_type: IconType::Folder,
            x: start_x,
            y: start_y + icon_spacing,
            action: IconAction::OpenFileManager,
        });
        
        self.icons.push(DesktopIcon {
            name: String::from("Editor"),
            icon_type: IconType::Editor,
            x: start_x,
            y: start_y + icon_spacing * 2,
            action: IconAction::OpenEditor,
        });
        
        self.icons.push(DesktopIcon {
            name: String::from("Calculator"),
            icon_type: IconType::Calculator,
            x: start_x,
            y: start_y + icon_spacing * 3,
            action: IconAction::OpenCalculator,
        });
        
        self.icons.push(DesktopIcon {
            name: String::from("Network"),
            icon_type: IconType::Network,
            x: start_x,
            y: start_y + icon_spacing * 4,
            action: IconAction::OpenNetwork,
        });
        
        self.icons.push(DesktopIcon {
            name: String::from("Games"),
            icon_type: IconType::Game,
            x: start_x,
            y: start_y + icon_spacing * 5,
            action: IconAction::OpenGame,
        });
        
        self.icons.push(DesktopIcon {
            name: String::from("Settings"),
            icon_type: IconType::Settings,
            x: start_x,
            y: start_y + icon_spacing * 6,
            action: IconAction::OpenSettings,
        });
        
        self.icons.push(DesktopIcon {
            name: String::from("About"),
            icon_type: IconType::About,
            x: start_x,
            y: start_y + icon_spacing * 7,
            action: IconAction::OpenAbout,
        });
        
        self.icons.push(DesktopIcon {
            name: String::from("OpenGL"),
            icon_type: IconType::OpenGL,
            x: start_x + 80,
            y: start_y,
            action: IconAction::OpenGL3D,
        });
    }
    
    /// Check if click is on a desktop icon
    fn check_icon_click(&self, x: i32, y: i32) -> Option<IconAction> {
        for icon in &self.icons {
            let icon_x = icon.x as i32;
            let icon_y = icon.y as i32;
            if x >= icon_x && x < icon_x + 64 && y >= icon_y && y < icon_y + 72 {
                return Some(icon.action);
            }
        }
        None
    }
    
    /// Create a new window with type
    pub fn create_window(&mut self, title: &str, x: i32, y: i32, width: u32, height: u32, wtype: WindowType) -> u32 {
        let mut window = Window::new(title, x, y, width, height, wtype);
        
        // Initialize content based on type
        match wtype {
            WindowType::Terminal => {
                window.content.push(String::from("TrustOS Terminal v1.0"));
                window.content.push(String::from(""));
                window.content.push(String::from("root@trustos:~$ _"));
            },
            WindowType::SystemInfo => {
                window.content.push(String::from("=== System Information ==="));
                window.content.push(String::from(""));
                window.content.push(format!("OS: TrustOS v0.1.0"));
                window.content.push(format!("Arch: x86_64"));
                window.content.push(format!("Display: {}x{}", self.width, self.height));
                window.content.push(String::from("Kernel: trustos_kernel"));
            },
            WindowType::About => {
                window.content.push(String::from("TrustOS"));
                window.content.push(String::from(""));
                window.content.push(String::from("A modern operating system"));
                window.content.push(String::from("written in Rust"));
                window.content.push(String::from(""));
                window.content.push(String::from("(c) 2026 Nathan"));
            },
            WindowType::Calculator => {
                window.content.push(String::from("[ Calculator ]"));
                window.content.push(String::from(""));
                window.content.push(String::from("  Display: 0"));
                window.content.push(String::from(""));
                window.content.push(String::from(" [7] [8] [9] [/]"));
                window.content.push(String::from(" [4] [5] [6] [*]"));
                window.content.push(String::from(" [1] [2] [3] [-]"));
                window.content.push(String::from(" [0] [.] [=] [+]"));
            },
            WindowType::FileManager => {
                window.content.push(String::from("=== File Manager ==="));
                window.content.push(String::from("Path: /"));
                window.content.push(String::from(""));
                window.content.push(String::from("  Name              Type       Size    Program"));
                window.content.push(String::from("  ────────────────────────────────────────────"));
                window.file_path = Some(String::from("/"));
                // List actual files from ramfs with file type info
                if let Ok(entries) = crate::ramfs::with_fs(|fs| fs.ls(Some("/"))) {
                    for (name, ftype, size) in entries.iter().take(12) {
                        let icon = if *ftype == crate::ramfs::FileType::Directory { 
                            "[D]" 
                        } else { 
                            crate::file_assoc::get_file_icon(name)
                        };
                        let prog = if *ftype == crate::ramfs::FileType::Directory {
                            String::from("---")
                        } else {
                            String::from(crate::file_assoc::get_program_for_file(name).name())
                        };
                        let ftype_str = if *ftype == crate::ramfs::FileType::Directory { "DIR" } else { "FILE" };
                        window.content.push(format!("  {} {:<14} {:<10} {:<7} {}", icon, name, ftype_str, size, prog));
                    }
                }
                if window.content.len() <= 5 {
                    window.content.push(String::from("  (empty directory)"));
                }
                window.content.push(String::from(""));
                window.content.push(String::from("  [Enter] Open | [Up/Down] Navigate"));
            },
            WindowType::TextEditor => {
                window.content.push(String::from("=== Text Editor ==="));
                window.content.push(String::from(""));
                window.content.push(String::from("Enter text below:"));
                window.content.push(String::from("___________________"));
                window.content.push(String::from(""));
                window.content.push(String::from("|"));
            },
            WindowType::NetworkInfo => {
                window.content.push(String::from("=== Network Status ==="));
                window.content.push(String::from(""));
                if let Some(mac) = crate::network::get_mac_address() {
                    window.content.push(format!("MAC: {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
                        mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]));
                } else {
                    window.content.push(String::from("MAC: Not available"));
                }
                window.content.push(String::from("IP: Waiting for DHCP..."));
                window.content.push(String::from(""));
                if crate::virtio_net::is_initialized() {
                    window.content.push(String::from("Driver: virtio-net (active)"));
                } else {
                    window.content.push(String::from("Driver: none"));
                }
            },
            WindowType::Settings => {
                window.content.push(String::from("=== Settings ==="));
                window.content.push(String::from(""));
                window.content.push(format!("Resolution: {}x{}", self.width, self.height));
                window.content.push(String::from("Theme: Dark Green"));
                window.content.push(String::from(""));
                window.content.push(String::from("[1] File Associations"));
                window.content.push(String::from("[2] Display Settings"));
                window.content.push(String::from("[3] About System"));
            },
            WindowType::ImageViewer => {
                window.content.push(String::from("=== Image Viewer ==="));
                window.content.push(String::from(""));
                window.content.push(String::from("No image loaded"));
                window.content.push(String::from(""));
                window.content.push(String::from("Supported: PNG, JPG, BMP, GIF"));
            },
            WindowType::HexViewer => {
                window.content.push(String::from("=== Hex Viewer ==="));
                window.content.push(String::from(""));
                window.content.push(String::from("No file loaded"));
            },
            WindowType::Demo3D => {
                window.content.push(String::from("=== 3D Graphics Demo ==="));
                window.content.push(String::from(""));
                window.content.push(String::from("TrustOS Graphics Engine"));
                window.content.push(String::from("Software 3D Renderer"));
                window.content.push(String::from(""));
                window.content.push(String::from("Features:"));
                window.content.push(String::from("- Wireframe/Solid/Mixed modes"));
                window.content.push(String::from("- Z-buffer depth testing"));
                window.content.push(String::from("- Flat shading with lighting"));
                window.content.push(String::from("- Perspective projection"));
                window.content.push(String::from("- Backface culling"));
                window.content.push(String::from(""));
                window.content.push(String::from("[Rotating Cube Demo Below]"));
            },
            WindowType::FileAssociations => {
                window.content.push(String::from("=== File Associations ==="));
                window.content.push(String::from(""));
                window.content.push(String::from("Extension | Program       | Type"));
                window.content.push(String::from("----------|---------------|-------------"));
                // Load associations
                let assocs = crate::file_assoc::list_associations();
                for (ext, prog, desc) in assocs.iter().take(15) {
                    window.content.push(format!(".{:<8} | {:<13} | {}", ext, prog, desc));
                }
                window.content.push(String::from(""));
                window.content.push(String::from("Click extension to change program"));
            },
            _ => {}
        }
        
        let id = window.id;
        self.windows.push(window);
        id
    }
    
    /// Close a window
    pub fn close_window(&mut self, id: u32) {
        self.windows.retain(|w| w.id != id);
    }
    
    /// Minimize/restore a window
    pub fn minimize_window(&mut self, id: u32) {
        if let Some(w) = self.windows.iter_mut().find(|w| w.id == id) {
            w.minimized = !w.minimized;
        }
    }
    
    /// Focus a window (bring to front)
    pub fn focus_window(&mut self, id: u32) {
        for w in &mut self.windows {
            w.focused = false;
        }
        if let Some(idx) = self.windows.iter().position(|w| w.id == id) {
            let mut window = self.windows.remove(idx);
            window.focused = true;
            window.minimized = false;
            self.windows.push(window);
        }
    }
    
    // ═══════════════════════════════════════════════════════════════════════════════
    // NEW: Windows-like features for hotkeys and GUI engine
    // ═══════════════════════════════════════════════════════════════════════════════
    
    /// Get screen dimensions
    pub fn screen_width(&self) -> u32 { self.width }
    pub fn screen_height(&self) -> u32 { self.height }
    
    /// Close the currently focused window (Alt+F4)
    pub fn close_focused_window(&mut self) {
        if let Some(id) = self.windows.iter().rev().find(|w| w.focused).map(|w| w.id) {
            self.close_window(id);
        }
    }
    
    /// Minimize the currently focused window (Win+Down)
    pub fn minimize_focused_window(&mut self) {
        if let Some(id) = self.windows.iter().rev().find(|w| w.focused).map(|w| w.id) {
            self.minimize_window(id);
        }
    }
    
    /// Toggle maximize on focused window (Win+Up)
    pub fn toggle_maximize_focused(&mut self) {
        if let Some(id) = self.windows.iter().rev().find(|w| w.focused).map(|w| w.id) {
            let (sw, sh) = (self.width, self.height);
            if let Some(w) = self.windows.iter_mut().find(|w| w.id == id) {
                w.toggle_maximize(sw, sh);
            }
        }
    }
    
    /// Snap focused window to left/right (Win+Arrow)
    pub fn snap_focused_window(&mut self, dir: SnapDir) {
        if let Some(w) = self.windows.iter_mut().rev().find(|w| w.focused) {
            let work_height = self.height - TASKBAR_HEIGHT;
            
            match dir {
                SnapDir::Left => {
                    w.x = 0;
                    w.y = 0;
                    w.width = self.width / 2;
                    w.height = work_height;
                }
                SnapDir::Right => {
                    w.x = (self.width / 2) as i32;
                    w.y = 0;
                    w.width = self.width / 2;
                    w.height = work_height;
                }
            }
            w.maximized = false;
        }
    }
    
    /// Toggle show desktop (Win+D) - minimize/restore all windows
    pub fn toggle_show_desktop(&mut self) {
        // Check if all windows are minimized
        let all_minimized = self.windows.iter().all(|w| w.minimized);
        
        // Toggle all windows
        for w in &mut self.windows {
            w.minimized = !all_minimized;
        }
    }
    
    /// Focus window by index (for Alt+Tab)
    pub fn focus_window_by_index(&mut self, index: usize) {
        if index < self.windows.len() {
            // Get visible (non-minimized) windows
            let visible: Vec<u32> = self.windows.iter()
                .filter(|w| !w.minimized)
                .map(|w| w.id)
                .collect();
            
            if index < visible.len() {
                self.focus_window(visible[index]);
            }
        }
    }
    
    /// Get list of window titles for Alt+Tab display
    pub fn get_window_titles(&self) -> Vec<String> {
        self.windows.iter()
            .filter(|w| !w.minimized)
            .map(|w| w.title.clone())
            .collect()
    }
    
    /// Open a new terminal window
    pub fn open_terminal(&mut self) {
        let id = self.create_window("Terminal", 100, 100, 700, 500, WindowType::Terminal);
        self.focus_window(id);
    }

    /// Handle mouse click
    pub fn handle_click(&mut self, x: i32, y: i32, pressed: bool) {
        if pressed {
            // Close context menu on any left click
            if self.context_menu.visible {
                if let Some(action) = self.check_context_menu_click(x, y) {
                    self.execute_context_action(action);
                }
                self.context_menu.visible = false;
                return;
            }
            
            // Record click for double-click detection
            crate::mouse::record_click();
            
            // Check start menu clicks first
            if self.start_menu_open {
                if let Some(action) = self.check_start_menu_click(x, y) {
                    self.start_menu_open = false;
                    self.handle_menu_action(action);
                    return;
                }
            }
            
            // Check taskbar first
            if y >= (self.height - TASKBAR_HEIGHT) as i32 {
                self.handle_taskbar_click(x, y);
                return;
            }
            
            // Check windows from top to bottom
            for i in (0..self.windows.len()).rev() {
                if self.windows[i].contains(x, y) {
                    let id = self.windows[i].id;
                    
                    if self.windows[i].on_close_button(x, y) {
                        self.close_window(id);
                        return;
                    }
                    
                    if self.windows[i].on_maximize_button(x, y) {
                        let (sw, sh) = (self.width, self.height);
                        if let Some(w) = self.windows.iter_mut().find(|w| w.id == id) {
                            w.toggle_maximize(sw, sh);
                        }
                        return;
                    }
                    
                    if self.windows[i].on_minimize_button(x, y) {
                        self.minimize_window(id);
                        return;
                    }
                    
                    // Check for resize edge
                    let resize_edge = self.windows[i].on_resize_edge(x, y);
                    if resize_edge != ResizeEdge::None {
                        self.windows[i].resizing = resize_edge;
                        self.windows[i].drag_offset_x = x;
                        self.windows[i].drag_offset_y = y;
                        self.focus_window(id);
                        return;
                    }
                    
                    if self.windows[i].in_title_bar(x, y) {
                        // Double-click to maximize
                        if crate::mouse::is_double_click() {
                            crate::mouse::reset_click_count();
                            let (sw, sh) = (self.width, self.height);
                            if let Some(w) = self.windows.iter_mut().find(|w| w.id == id) {
                                w.toggle_maximize(sw, sh);
                            }
                            return;
                        }
                        
                        let win_x = self.windows[i].x;
                        let win_y = self.windows[i].y;
                        self.windows[i].dragging = true;
                        self.windows[i].drag_offset_x = x - win_x;
                        self.windows[i].drag_offset_y = y - win_y;
                    }
                    
                    self.focus_window(id);
                    return;
                }
            }
            
            // Check desktop icons - handle double-click
            if let Some(idx) = self.check_icon_index(x, y) {
                if crate::mouse::is_double_click() {
                    crate::mouse::reset_click_count();
                    let action = self.icons[idx].action;
                    self.handle_icon_action(action);
                }
                return;
            }
            
            self.start_menu_open = false;
        } else {
            // Mouse released - stop dragging and resizing
            for w in &mut self.windows {
                w.dragging = false;
                w.resizing = ResizeEdge::None;
            }
        }
    }
    
    /// Handle right-click (context menu)
    pub fn handle_right_click(&mut self, x: i32, y: i32, pressed: bool) {
        if !pressed {
            return; // Only handle press, not release
        }
        
        // Close any existing context menu
        self.context_menu.visible = false;
        self.start_menu_open = false;
        
        // Check if right-click on desktop icon
        if let Some(idx) = self.check_icon_index(x, y) {
            self.show_icon_context_menu(x, y, idx);
            return;
        }
        
        // Check if right-click on desktop (empty area)
        if y < (self.height - TASKBAR_HEIGHT) as i32 {
            self.show_desktop_context_menu(x, y);
        }
    }
    
    /// Show context menu for a desktop icon
    fn show_icon_context_menu(&mut self, x: i32, y: i32, icon_index: usize) {
        self.context_menu = ContextMenu {
            visible: true,
            x,
            y,
            items: alloc::vec![
                ContextMenuItem { label: String::from("  Open"), action: ContextAction::Open },
                ContextMenuItem { label: String::from("  Open With..."), action: ContextAction::OpenWith },
                ContextMenuItem { label: String::from("─────────────────────"), action: ContextAction::Cancel },
                ContextMenuItem { label: String::from("  Cut"), action: ContextAction::Cut },
                ContextMenuItem { label: String::from("  Copy"), action: ContextAction::Copy },
                ContextMenuItem { label: String::from("─────────────────────"), action: ContextAction::Cancel },
                ContextMenuItem { label: String::from("  Rename"), action: ContextAction::Rename },
                ContextMenuItem { label: String::from("  Delete"), action: ContextAction::Delete },
                ContextMenuItem { label: String::from("─────────────────────"), action: ContextAction::Cancel },
                ContextMenuItem { label: String::from("  Properties"), action: ContextAction::Properties },
            ],
            selected_index: 0,
            target_icon: Some(icon_index),
            target_file: None,
        };
    }
    
    /// Show context menu for desktop (empty area) - Windows style
    fn show_desktop_context_menu(&mut self, x: i32, y: i32) {
        self.context_menu = ContextMenu {
            visible: true,
            x,
            y,
            items: alloc::vec![
                ContextMenuItem { label: String::from("  View              >"), action: ContextAction::ViewLargeIcons },
                ContextMenuItem { label: String::from("  Sort by           >"), action: ContextAction::SortByName },
                ContextMenuItem { label: String::from("  Refresh"), action: ContextAction::Refresh },
                ContextMenuItem { label: String::from("─────────────────────"), action: ContextAction::Cancel },
                ContextMenuItem { label: String::from("  Paste"), action: ContextAction::Paste },
                ContextMenuItem { label: String::from("─────────────────────"), action: ContextAction::Cancel },
                ContextMenuItem { label: String::from("  New               >"), action: ContextAction::NewFile },
                ContextMenuItem { label: String::from("─────────────────────"), action: ContextAction::Cancel },
                ContextMenuItem { label: String::from("  Open in Terminal"), action: ContextAction::TerminalHere },
                ContextMenuItem { label: String::from("  Personalize"), action: ContextAction::Personalize },
                ContextMenuItem { label: String::from("  Properties"), action: ContextAction::Properties },
            ],
            selected_index: 0,
            target_icon: None,
            target_file: None,
        };
    }
    
    /// Check if click is on context menu and return action
    fn check_context_menu_click(&self, x: i32, y: i32) -> Option<ContextAction> {
        if !self.context_menu.visible {
            return None;
        }
        
        let menu_x = self.context_menu.x;
        let menu_y = self.context_menu.y;
        let menu_width = 150;
        let item_height = 22;
        let menu_height = self.context_menu.items.len() as i32 * item_height;
        
        if x >= menu_x && x < menu_x + menu_width && y >= menu_y && y < menu_y + menu_height {
            let idx = ((y - menu_y) / item_height) as usize;
            if idx < self.context_menu.items.len() {
                return Some(self.context_menu.items[idx].action);
            }
        }
        
        None
    }
    
    /// Execute context menu action
    fn execute_context_action(&mut self, action: ContextAction) {
        let offset = (self.windows.len() as i32 * 25) % 200;
        
        match action {
            ContextAction::Open => {
                if let Some(idx) = self.context_menu.target_icon {
                    let icon_action = self.icons[idx].action;
                    self.handle_icon_action(icon_action);
                }
            },
            ContextAction::OpenWith => {
                // Show file associations window
                self.create_window("Open With", 300 + offset, 200 + offset, 400, 300, WindowType::FileAssociations);
            },
            ContextAction::Refresh => {
                // Redraw desktop - just logs for now
                crate::serial_println!("[GUI] Desktop refreshed");
            },
            ContextAction::NewFile => {
                // Create a new file in ramfs
                let filename = format!("/desktop/newfile_{}.txt", self.frame_count);
                crate::ramfs::with_fs(|fs| {
                    let _ = fs.write_file(&filename, b"New file created from desktop");
                });
                crate::serial_println!("[GUI] Created new file: {}", filename);
            },
            ContextAction::NewFolder => {
                let dirname = format!("/desktop/folder_{}", self.frame_count);
                crate::ramfs::with_fs(|fs| {
                    let _ = fs.mkdir(&dirname);
                });
                crate::serial_println!("[GUI] Created new folder: {}", dirname);
            },
            ContextAction::Properties => {
                let (w, h) = (self.width, self.height);
                let win_count = self.windows.len();
                let icon_count = self.icons.len();
                let win_id = self.create_window("Properties", 350 + offset, 250 + offset, 320, 220, WindowType::About);
                if let Some(window) = self.windows.iter_mut().find(|wnd| wnd.id == win_id) {
                    window.content.clear();
                    window.content.push(String::from("═══════ System Properties ═══════"));
                    window.content.push(String::new());
                    window.content.push(format!("Display: {}x{}", w, h));
                    window.content.push(format!("Windows open: {}", win_count + 1));
                    window.content.push(format!("Desktop icons: {}", icon_count));
                    window.content.push(String::new());
                    window.content.push(String::from("Theme: GitHub Dark"));
                    window.content.push(String::from("OS: TrustOS v0.1.0"));
                }
            },
            ContextAction::Cut => {
                crate::serial_println!("[GUI] Cut (not implemented)");
            },
            ContextAction::Copy => {
                crate::serial_println!("[GUI] Copy (not implemented)");
            },
            ContextAction::Paste => {
                crate::serial_println!("[GUI] Paste (not implemented)");
            },
            ContextAction::ViewLargeIcons | ContextAction::ViewSmallIcons | ContextAction::ViewList => {
                crate::serial_println!("[GUI] View mode changed");
            },
            ContextAction::SortByName | ContextAction::SortByDate | ContextAction::SortBySize => {
                crate::serial_println!("[GUI] Sort order changed");
            },
            ContextAction::Personalize => {
                self.create_window("Personalization", 250 + offset, 150 + offset, 400, 300, WindowType::Settings);
            },
            ContextAction::TerminalHere => {
                self.create_window("Terminal", 200 + offset, 120 + offset, 500, 350, WindowType::Terminal);
            },
            ContextAction::Delete | ContextAction::Rename | ContextAction::CopyPath => {
                crate::serial_println!("[GUI] Action not implemented yet");
            },
            ContextAction::Cancel => {},
        }
    }
    
    /// Get icon index at position (for context menu)
    fn check_icon_index(&self, x: i32, y: i32) -> Option<usize> {
        for (idx, icon) in self.icons.iter().enumerate() {
            let icon_x = icon.x as i32;
            let icon_y = icon.y as i32;
            if x >= icon_x && x < icon_x + 64 && y >= icon_y && y < icon_y + 72 {
                return Some(idx);
            }
        }
        None
    }
    
    /// Handle desktop icon action
    fn handle_icon_action(&mut self, action: IconAction) {
        let offset = (self.windows.len() as i32 * 25) % 200;
        match action {
            IconAction::OpenTerminal => {
                self.create_window("Terminal", 150 + offset, 100 + offset, 500, 350, WindowType::Terminal);
            },
            IconAction::OpenFileManager => {
                self.create_window("Files", 180 + offset, 120 + offset, 400, 350, WindowType::FileManager);
            },
            IconAction::OpenCalculator => {
                self.create_window("Calculator", 450 + offset, 150 + offset, 200, 220, WindowType::Calculator);
            },
            IconAction::OpenNetwork => {
                self.create_window("Network", 200 + offset, 140 + offset, 320, 200, WindowType::NetworkInfo);
            },
            IconAction::OpenSettings => {
                self.create_window("Settings", 300 + offset, 160 + offset, 350, 250, WindowType::Settings);
            },
            IconAction::OpenAbout => {
                self.create_window("About TrustOS", 350 + offset, 180 + offset, 350, 200, WindowType::About);
            },
            IconAction::OpenGame => {
                self.create_window("Snake Game", 250 + offset, 120 + offset, 320, 320, WindowType::Game);
            },
            IconAction::OpenEditor => {
                self.create_window("Text Editor", 200 + offset, 100 + offset, 450, 350, WindowType::TextEditor);
            },
            IconAction::OpenGL3D => {
                self.create_window("TrustGL 3D Demo", 150 + offset, 80 + offset, 400, 350, WindowType::Demo3D);
            },
        }
    }
    
    fn handle_taskbar_click(&mut self, x: i32, _y: i32) {
        if x < 48 {
            self.start_menu_open = !self.start_menu_open;
            return;
        }
        
        let mut btn_x = 56;
        for w in &self.windows {
            if x >= btn_x && x < btn_x + 120 {
                let id = w.id;
                self.focus_window(id);
                return;
            }
            btn_x += 124;
        }
    }
    
    /// Menu actions enum
    fn check_start_menu_click(&self, x: i32, y: i32) -> Option<u8> {
        let menu_x = 2i32;
        let menu_y = (self.height - TASKBAR_HEIGHT - 200) as i32;
        let menu_w = 180;
        let menu_h = 200;
        
        // Check if click is inside menu
        if x < menu_x || x >= menu_x + menu_w || y < menu_y || y >= menu_y + menu_h {
            return None;
        }
        
        // Check which item was clicked (skip header)
        let item_start_y = menu_y + 28;
        let item_height = 26;
        
        if y >= item_start_y {
            let item_index = ((y - item_start_y) / item_height) as u8;
            if item_index < 7 {
                return Some(item_index);
            }
        }
        
        None
    }
    
    fn handle_menu_action(&mut self, action: u8) {
        match action {
            0 => { // Terminal
                let x = 100 + (self.windows.len() as i32 * 30);
                let y = 80 + (self.windows.len() as i32 * 20);
                self.create_window("Terminal", x, y, 500, 350, WindowType::Terminal);
            },
            1 => { // File Manager
                self.create_window("Files", 150, 100, 400, 350, WindowType::FileManager);
            },
            2 => { // Calculator
                self.create_window("Calculator", 400, 150, 200, 220, WindowType::Calculator);
            },
            3 => { // Network
                self.create_window("Network", 200, 120, 320, 200, WindowType::NetworkInfo);
            },
            4 => { // Settings
                self.create_window("Settings", 250, 140, 350, 250, WindowType::Settings);
            },
            5 => { // About
                self.create_window("About TrustOS", 300, 180, 350, 200, WindowType::About);
            },
            6 => { // Shutdown
                crate::println!("\n\n=== SYSTEM SHUTDOWN ===");
                crate::println!("Goodbye!");
                // In a real OS we'd ACPI shutdown here
                // For now, just halt
                loop { x86_64::instructions::hlt(); }
            },
            _ => {}
        }
    }
    
    /// Handle keyboard input for the focused window
    pub fn handle_keyboard_input(&mut self, key: u8) {
        // Find focused window
        if let Some(window) = self.windows.iter_mut().find(|w| w.focused) {
            match window.window_type {
                WindowType::Terminal => {
                    self.handle_terminal_key(key);
                },
                WindowType::FileManager => {
                    self.handle_filemanager_key(key);
                },
                WindowType::FileAssociations => {
                    self.handle_fileassoc_key(key);
                },
                WindowType::Settings => {
                    self.handle_settings_key(key);
                },
                WindowType::TextEditor => {
                    // For text editor, just add characters
                    if key == 0x08 { // Backspace
                        if !self.input_buffer.is_empty() {
                            self.input_buffer.pop();
                        }
                    } else if key == 0x0D || key == 0x0A { // Enter
                        // Add line to content
                        if let Some(w) = self.windows.iter_mut().find(|w| w.focused) {
                            w.content.push(self.input_buffer.clone());
                            self.input_buffer.clear();
                        }
                    } else if key >= 0x20 && key < 0x7F {
                        self.input_buffer.push(key as char);
                    }
                },
                _ => {}
            }
        }
    }
    
    /// Handle file manager keyboard input
    fn handle_filemanager_key(&mut self, key: u8) {
        use crate::keyboard::{KEY_UP, KEY_DOWN};
        
        let mut action: Option<(String, bool)> = None; // (filename, is_dir)
        
        if let Some(window) = self.windows.iter_mut().find(|w| w.focused && w.window_type == WindowType::FileManager) {
            // Get file list (skip header lines - first 5 lines)
            let file_count = window.content.len().saturating_sub(7); // header + footer
            
            if key == KEY_UP {
                if window.selected_index > 0 {
                    window.selected_index -= 1;
                }
            } else if key == KEY_DOWN {
                if window.selected_index < file_count.saturating_sub(1) {
                    window.selected_index += 1;
                }
            } else if key == 0x0D || key == 0x0A { // Enter - open file
                // Get selected file
                let idx = window.selected_index + 5; // Skip header
                if idx < window.content.len() - 2 { // Skip footer
                    let line = &window.content[idx];
                    // Parse filename from line format: "  [icon] filename..."
                    if let Some(name_start) = line.find(']') {
                        let rest = &line[name_start + 2..];
                        if let Some(name_end) = rest.find(' ') {
                            let filename = String::from(rest[..name_end].trim());
                            let is_dir = line.contains("[D]");
                            action = Some((filename, is_dir));
                        }
                    }
                }
            }
        }
        
        // Handle action outside the borrow
        if let Some((filename, is_dir)) = action {
            if is_dir {
                // TODO: Navigate into directory
                crate::serial_println!("[FM] Navigate to dir: {}", filename);
            } else {
                self.open_file(&filename);
            }
        }
    }
    
    /// Open file with associated program
    fn open_file(&mut self, filename: &str) {
        use crate::file_assoc::{get_program_for_file, Program};
        
        let program = get_program_for_file(filename);
        let offset = (self.windows.len() as i32 * 25) % 150;
        
        match program {
            Program::TextEditor => {
                let mut id = self.create_window(&format!("Edit: {}", filename), 200 + offset, 120 + offset, 450, 350, WindowType::TextEditor);
                // Load file content
                if let Some(window) = self.windows.iter_mut().find(|w| w.id == id) {
                    window.file_path = Some(String::from(filename));
                    window.content.clear();
                    window.content.push(format!("=== {} ===", filename));
                    window.content.push(String::new());
                    // Try to read file (copy to avoid lifetime issue)
                    let file_path = format!("/{}", filename);
                    if let Ok(content) = crate::ramfs::with_fs(|fs| {
                        fs.read_file(&file_path).map(|d| d.to_vec())
                    }) {
                        if let Ok(text) = core::str::from_utf8(&content) {
                            for line in text.lines().take(20) {
                                window.content.push(String::from(line));
                            }
                        } else {
                            window.content.push(String::from("(binary file)"));
                        }
                    } else {
                        window.content.push(String::from("(could not read file)"));
                    }
                }
            },
            Program::ImageViewer => {
                let id = self.create_window(&format!("View: {}", filename), 180 + offset, 100 + offset, 400, 320, WindowType::ImageViewer);
                if let Some(window) = self.windows.iter_mut().find(|w| w.id == id) {
                    window.file_path = Some(String::from(filename));
                    window.content.clear();
                    window.content.push(format!("=== Image: {} ===", filename));
                    window.content.push(String::new());
                    // For now, just show file info
                    let file_path = format!("/{}", filename);
                    if let Ok(content) = crate::ramfs::with_fs(|fs| fs.read_file(&file_path).map(|d| d.to_vec())) {
                        window.content.push(format!("Size: {} bytes", content.len()));
                        window.content.push(String::from("Format: Detected from header"));
                        // Check PNG signature
                        if content.len() >= 8 && &content[0..8] == b"\x89PNG\r\n\x1a\n" {
                            window.content.push(String::from("Type: PNG Image"));
                        } else if content.len() >= 2 && &content[0..2] == b"\xFF\xD8" {
                            window.content.push(String::from("Type: JPEG Image"));
                        } else if content.len() >= 2 && &content[0..2] == b"BM" {
                            window.content.push(String::from("Type: BMP Image"));
                        } else {
                            window.content.push(String::from("Type: Unknown"));
                        }
                        window.content.push(String::new());
                        window.content.push(String::from("(Image preview not implemented)"));
                    }
                }
            },
            Program::HexViewer => {
                let id = self.create_window(&format!("Hex: {}", filename), 160 + offset, 80 + offset, 500, 350, WindowType::HexViewer);
                if let Some(window) = self.windows.iter_mut().find(|w| w.id == id) {
                    window.file_path = Some(String::from(filename));
                    window.content.clear();
                    window.content.push(format!("=== Hex View: {} ===", filename));
                    window.content.push(String::new());
                    window.content.push(String::from("Offset   00 01 02 03 04 05 06 07  ASCII"));
                    window.content.push(String::from("──────── ─────────────────────── ────────"));
                    // Load and show hex
                    let file_path = format!("/{}", filename);
                    if let Ok(content) = crate::ramfs::with_fs(|fs| fs.read_file(&file_path).map(|d| d.to_vec())) {
                        for (i, chunk) in content.chunks(8).enumerate().take(16) {
                            let offset = i * 8;
                            let hex: String = chunk.iter()
                                .map(|b| format!("{:02X} ", b))
                                .collect();
                            let ascii: String = chunk.iter()
                                .map(|&b| if b >= 0x20 && b < 0x7F { b as char } else { '.' })
                                .collect();
                            window.content.push(format!("{:08X} {:<24} {}", offset, hex, ascii));
                        }
                    }
                }
            },
            Program::Terminal => {
                // Execute file
                crate::serial_println!("[EXEC] Would execute: {}", filename);
                let id = self.create_window("Execution", 200 + offset, 150 + offset, 400, 200, WindowType::Terminal);
                if let Some(window) = self.windows.iter_mut().find(|w| w.id == id) {
                    window.content.clear();
                    window.content.push(format!("Executing: {}", filename));
                    window.content.push(String::from(""));
                    window.content.push(String::from("(ELF execution not yet integrated in GUI)"));
                }
            },
            _ => {
                // Unknown - open with hex viewer
                crate::serial_println!("[OPEN] No handler for: {}", filename);
            }
        }
    }
    
    /// Handle settings keyboard input
    fn handle_settings_key(&mut self, key: u8) {
        if key == b'1' {
            // Open file associations
            let offset = (self.windows.len() as i32 * 20) % 100;
            self.create_window("File Associations", 250 + offset, 130 + offset, 500, 400, WindowType::FileAssociations);
        } else if key == b'2' {
            // Display settings - just show info for now
        } else if key == b'3' {
            // About
            let offset = (self.windows.len() as i32 * 20) % 100;
            self.create_window("About TrustOS", 280 + offset, 150 + offset, 350, 200, WindowType::About);
        }
    }
    
    /// Handle file associations keyboard input
    fn handle_fileassoc_key(&mut self, key: u8) {
        use crate::keyboard::{KEY_UP, KEY_DOWN};
        
        if let Some(window) = self.windows.iter_mut().find(|w| w.focused && w.window_type == WindowType::FileAssociations) {
            let list_start = 4; // After header
            let list_end = window.content.len().saturating_sub(2);
            let list_len = list_end.saturating_sub(list_start);
            
            if key == KEY_UP && window.selected_index > 0 {
                window.selected_index -= 1;
            } else if key == KEY_DOWN && window.selected_index < list_len.saturating_sub(1) {
                window.selected_index += 1;
            } else if key == 0x0D || key == 0x0A {
                // Cycle through programs for selected extension
                let idx = list_start + window.selected_index;
                if idx < list_end {
                    // Get extension from line
                    let line = &window.content[idx];
                    if let Some(ext_end) = line.find('|') {
                        let ext = line[1..ext_end].trim().trim_start_matches('.');
                        // Cycle to next program
                        use crate::file_assoc::{Program, set_program, get_program_for_file};
                        let current = get_program_for_file(&format!("test.{}", ext));
                        let next = match current {
                            Program::TextEditor => Program::ImageViewer,
                            Program::ImageViewer => Program::HexViewer,
                            Program::HexViewer => Program::Terminal,
                            Program::Terminal => Program::TextEditor,
                            _ => Program::TextEditor,
                        };
                        set_program(ext, next.clone());
                        // Update display
                        crate::serial_println!("[ASSOC] {} -> {}", ext, next.name());
                    }
                }
            }
        }
    }
    
    /// Handle terminal keyboard input
    fn handle_terminal_key(&mut self, key: u8) {
        if key == 0x08 { // Backspace
            if !self.input_buffer.is_empty() {
                self.input_buffer.pop();
            }
        } else if key == 0x0D || key == 0x0A { // Enter
            let cmd = self.input_buffer.clone();
            self.input_buffer.clear();
            
            // Execute command first (before borrowing windows)
            let output = Self::execute_command_static(&cmd);
            
            // Add command to terminal output
            if let Some(window) = self.windows.iter_mut().find(|w| w.focused && w.window_type == WindowType::Terminal) {
                // Remove cursor line
                if window.content.last().map(|s| s.ends_with("$ _")).unwrap_or(false) {
                    window.content.pop();
                }
                window.content.push(format!("root@trustos:~$ {}", cmd));
                
                for line in output {
                    window.content.push(line);
                }
                
                // Add new prompt
                window.content.push(String::from("root@trustos:~$ _"));
                
                // Scroll if needed (keep last 20 lines visible)
                while window.content.len() > 18 {
                    window.content.remove(0);
                }
            }
        } else if key >= 0x20 && key < 0x7F {
            self.input_buffer.push(key as char);
            
            // Update terminal display
            if let Some(window) = self.windows.iter_mut().find(|w| w.focused && w.window_type == WindowType::Terminal) {
                // Update prompt line with current input
                if let Some(last) = window.content.last_mut() {
                    *last = format!("root@trustos:~$ {}_", self.input_buffer);
                }
            }
        }
    }
    
    /// Execute terminal command (static to avoid borrow issues)
    fn execute_command_static(cmd: &str) -> Vec<String> {
        let mut output = Vec::new();
        let cmd = cmd.trim();
        
        if cmd.is_empty() {
            return output;
        }
        
        match cmd {
            "help" => {
                output.push(String::from("TrustOS GUI Terminal - Commands:"));
                output.push(String::from("  help      - Show this help"));
                output.push(String::from("  ls [dir]  - List directory contents"));
                output.push(String::from("  pwd       - Print working directory"));
                output.push(String::from("  date      - Show current date/time"));
                output.push(String::from("  uname     - System information"));
                output.push(String::from("  free      - Memory usage"));
                output.push(String::from("  net       - Network status"));
                output.push(String::from("  cat <file>- Show file contents"));
                output.push(String::from("  clear     - Clear terminal"));
                output.push(String::from("  exit      - Close terminal"));
            },
            "ls" | "ls /" => {
                output.push(String::from("Directory: /"));
                if let Ok(entries) = crate::ramfs::with_fs(|fs| fs.ls(Some("/"))) {
                    for (name, ftype, size) in entries.iter().take(15) {
                        let icon = if *ftype == crate::ramfs::FileType::Directory { "📁" } else { "📄" };
                        output.push(format!("  {} {} ({} bytes)", icon, name, size));
                    }
                    if entries.is_empty() {
                        output.push(String::from("  (empty directory)"));
                    }
                }
            },
            _ if cmd.starts_with("ls ") => {
                let path = &cmd[3..];
                output.push(format!("Directory: {}", path));
                if let Ok(entries) = crate::ramfs::with_fs(|fs| fs.ls(Some(path))) {
                    for (name, ftype, size) in entries.iter().take(15) {
                        let icon = if *ftype == crate::ramfs::FileType::Directory { "📁" } else { "📄" };
                        output.push(format!("  {} {} ({} bytes)", icon, name, size));
                    }
                } else {
                    output.push(format!("  Error: cannot access '{}'", path));
                }
            },
            "pwd" => {
                output.push(String::from("/"));
            },
            "clear" => {
                // Will be handled specially
            },
            "date" => {
                let dt = crate::rtc::read_rtc();
                output.push(format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02}", 
                    dt.year, dt.month, dt.day, dt.hour, dt.minute, dt.second));
            },
            "uname" | "uname -a" => {
                output.push(String::from("TrustOS 0.1.0 x86_64 Rust Kernel"));
                output.push(format!("Heap: {} MB", crate::memory::HEAP_SIZE / 1024 / 1024));
            },
            "whoami" => {
                output.push(String::from("root"));
            },
            "free" | "mem" => {
                let heap_mb = crate::memory::HEAP_SIZE / 1024 / 1024;
                output.push(String::from("Memory Usage:"));
                output.push(format!("  Heap Size: {} MB", heap_mb));
                output.push(String::from("  Kernel:   Active"));
            },
            "net" | "ifconfig" | "ip" => {
                output.push(String::from("Network Status:"));
                if crate::network::is_available() {
                    if let Some((mac, ip, _state)) = crate::network::get_interface() {
                        output.push(format!("  MAC: {}", mac));
                        if let Some(ip) = ip {
                            output.push(format!("  IP:  {}", ip));
                        }
                        output.push(String::from("  Status: Connected"));
                    }
                } else {
                    output.push(String::from("  Status: No network"));
                }
            },
            _ if cmd.starts_with("cat ") => {
                let filename = &cmd[4..];
                output.push(format!("cat: {} - use shell for file operations", filename));
            },
            _ if cmd.starts_with("echo ") => {
                output.push(String::from(&cmd[5..]));
            },
            "exit" | "quit" => {
                output.push(String::from("Use the X button to close the terminal"));
            },
            _ => {
                output.push(format!("Command not found: {}", cmd));
                output.push(String::from("Type 'help' for available commands"));
            },
        }
        
        output
    }
    
    /// Handle mouse move
    pub fn handle_move(&mut self, x: i32, y: i32) {
        self.cursor_x = x.clamp(0, self.width as i32 - 1);
        self.cursor_y = y.clamp(0, self.height as i32 - 1);
        
        for w in &mut self.windows {
            // Handle window dragging
            if w.dragging && !w.maximized {
                w.x = (x - w.drag_offset_x).max(0).min(self.width as i32 - 50);
                w.y = (y - w.drag_offset_y).max(0).min(self.height as i32 - TASKBAR_HEIGHT as i32 - TITLE_BAR_HEIGHT as i32);
            }
            
            // Handle window resizing
            if w.resizing != ResizeEdge::None {
                let dx = x - w.drag_offset_x;
                let dy = y - w.drag_offset_y;
                
                match w.resizing {
                    ResizeEdge::Right | ResizeEdge::BottomRight => {
                        let new_width = (w.width as i32 + dx).max(w.min_width as i32) as u32;
                        w.width = new_width.min(self.width - w.x as u32);
                        w.drag_offset_x = x;
                    }
                    _ => {}
                }
                
                match w.resizing {
                    ResizeEdge::Bottom | ResizeEdge::BottomRight => {
                        let new_height = (w.height as i32 + dy).max(w.min_height as i32) as u32;
                        w.height = new_height.min(self.height - TASKBAR_HEIGHT - w.y as u32);
                        w.drag_offset_y = y;
                    }
                    _ => {}
                }
            }
        }
    }
    
    /// Handle scroll wheel
    pub fn handle_scroll(&mut self, delta: i8) {
        // Scroll focused window content if it's a scrollable type
        if let Some(window) = self.windows.iter_mut().rev().find(|w| w.focused && !w.minimized) {
            match window.window_type {
                WindowType::FileManager | WindowType::TextEditor | WindowType::HexViewer | 
                WindowType::FileAssociations | WindowType::Terminal => {
                    let max_scroll = if window.content.len() > 10 {
                        window.content.len() - 10
                    } else {
                        0
                    };
                    
                    if delta > 0 {
                        // Scroll up
                        if window.scroll_offset > 0 {
                            window.scroll_offset = window.scroll_offset.saturating_sub(3);
                        }
                    } else if delta < 0 {
                        // Scroll down
                        window.scroll_offset = (window.scroll_offset + 3).min(max_scroll);
                    }
                },
                _ => {}
            }
        }
    }
    
    /// Draw the desktop with double buffering
    pub fn draw(&mut self) {
        self.frame_count += 1;
        
        // Toggle cursor blink every ~30 frames
        if self.frame_count % 30 == 0 {
            self.cursor_blink = !self.cursor_blink;
        }
        
        // Get mouse state
        let mouse = crate::mouse::get_state();
        
        // Use OpenGL compositor if enabled
        if self.render_mode == RenderMode::OpenGL {
            self.draw_opengl();
            return;
        }
        
        // === CLASSIC RENDERING MODE ===
        
        // Detect state changes that require taskbar/background recache
        let windows_changed = self.windows.len() != self.last_window_count;
        let menu_changed = self.start_menu_open != self.last_start_menu_open 
                        || self.context_menu.visible != self.last_context_menu_visible;
        
        // If windows changed, invalidate taskbar in background cache
        if windows_changed {
            self.needs_full_redraw = true;
        }
        
        // OPTIMIZATION 1: Background caching
        // Only draw background once, then cache it
        if !self.background_cached || self.needs_full_redraw {
            // First time: draw everything to backbuffer
            framebuffer::clear_backbuffer(DESKTOP_BG_TOP);
            self.draw_background();
            self.draw_logo_watermark();
            self.draw_desktop_icons();
            self.draw_taskbar();
            
            // Cache the static background (everything except windows and cursor)
            framebuffer::cache_current_background();
            self.background_cached = true;
            self.needs_full_redraw = false;
        } else {
            // OPTIMIZATION 2: Restore background from cache (very fast memcpy)
            framebuffer::restore_background_to_backbuffer();
        }
        
        // Draw windows (these change, so always redraw)
        let has_visible_windows = self.windows.iter().any(|w| w.visible && !w.minimized);
        for window in &self.windows {
            if window.visible && !window.minimized {
                self.draw_window(window);
            }
        }
        
        // Only redraw taskbar if there are windows (to show active indicators)
        // Otherwise the cached taskbar is already in the background
        if has_visible_windows || self.start_menu_open {
            self.draw_taskbar();
        }
        
        // Draw start menu if open
        if self.start_menu_open {
            self.draw_start_menu();
        }
        
        // Draw context menu if visible
        if self.context_menu.visible {
            self.draw_context_menu();
        }
        
        // Draw cursor last
        self.draw_cursor();
        
        // Update tracking state
        self.last_cursor_x = mouse.x;
        self.last_cursor_y = mouse.y;
        self.last_window_count = self.windows.len();
        self.last_start_menu_open = self.start_menu_open;
        self.last_context_menu_visible = self.context_menu.visible;
        
        // Swap buffers for flicker-free display
        framebuffer::swap_buffers();
    }
    
    /// Draw the desktop using OpenGL compositor
    fn draw_opengl(&mut self) {
        use crate::graphics::opengl::*;
        
        // Update compositor state
        let dt = 1.0 / 60.0; // Assume 60 FPS
        
        // Sync windows to compositor surfaces
        {
            let mut comp = compositor::compositor();
            comp.update(dt);
            
            // Update surface positions from windows
            for window in &self.windows {
                if let Some(surface) = comp.get_surface_mut(window.id) {
                    surface.x = window.x as f32;
                    surface.y = window.y as f32;
                    surface.width = window.width as f32;
                    surface.height = window.height as f32;
                    surface.focused = window.focused;
                    surface.visible = window.visible && !window.minimized;
                }
            }
        }
        
        // Render using compositor
        compositor::render_compositor_frame();
        
        // Draw additional UI elements on top (taskbar, menus, cursor)
        // These are rendered classically on top of the OpenGL content
        self.draw_taskbar();
        
        if self.start_menu_open {
            self.draw_start_menu();
        }
        
        if self.context_menu.visible {
            self.draw_context_menu();
        }
        
        // Draw desktop icons
        self.draw_desktop_icons();
        
        // Draw cursor
        self.draw_cursor();
        
        // Update state
        self.last_window_count = self.windows.len();
        self.last_start_menu_open = self.start_menu_open;
        self.last_context_menu_visible = self.context_menu.visible;
        
        // Swap buffers
        framebuffer::swap_buffers();
    }
    
    /// Draw context menu - Windows 11 style
    fn draw_context_menu(&self) {
        let menu_x = self.context_menu.x;
        let menu_y = self.context_menu.y;
        let menu_width = 200;
        let item_height = 28;
        let menu_height = self.context_menu.items.len() as i32 * item_height + 8;
        let padding = 4;
        
        // Soft multi-layer shadow
        for i in (1..=8).rev() {
            let alpha = (20 - i * 2).max(4) as u32;
            let shadow_color = alpha << 24;
            framebuffer::fill_rect(
                (menu_x + i) as u32, (menu_y + i + 2) as u32,
                menu_width as u32, menu_height as u32,
                shadow_color
            );
        }
        
        // Background with glassmorphism
        framebuffer::fill_rect(
            menu_x as u32, menu_y as u32,
            menu_width as u32, menu_height as u32,
            BG_MEDIUM
        );
        
        // Subtle gradient overlay
        for row in 0..menu_height {
            let alpha = (row * 10 / menu_height) as u32;
            let overlay = alpha << 24;
            framebuffer::fill_rect(
                menu_x as u32, (menu_y + row) as u32,
                menu_width as u32, 1,
                overlay
            );
        }
        
        // Border with gradient effect
        framebuffer::draw_rect(
            menu_x as u32, menu_y as u32,
            menu_width as u32, menu_height as u32,
            GREEN_MUTED
        );
        
        // Brighter top edge
        framebuffer::fill_rect(menu_x as u32, menu_y as u32, menu_width as u32, 1, GREEN_TERTIARY);
        
        // Draw items
        for (idx, item) in self.context_menu.items.iter().enumerate() {
            let item_y = menu_y + padding + idx as i32 * item_height;
            
            // Check if cursor is hovering this item
            let is_hovered = self.cursor_x >= menu_x && self.cursor_x < menu_x + menu_width
                && self.cursor_y >= item_y && self.cursor_y < item_y + item_height;
            
            if is_hovered && item.action != ContextAction::Cancel && !item.label.starts_with("─") {
                // Hover with glow effect
                framebuffer::fill_rect(
                    (menu_x + 4) as u32, item_y as u32,
                    (menu_width - 8) as u32, (item_height - 2) as u32,
                    GREEN_GHOST
                );
                framebuffer::fill_rect(
                    (menu_x + 6) as u32, (item_y + 1) as u32,
                    (menu_width - 12) as u32, (item_height - 4) as u32,
                    BG_LIGHT
                );
                // Left accent bar
                framebuffer::fill_rect(
                    (menu_x + 4) as u32, (item_y + 4) as u32,
                    2, (item_height - 10) as u32,
                    GREEN_PRIMARY
                );
            }
            
            // Separator line or text
            if item.label.starts_with("─") {
                framebuffer::fill_rect(
                    (menu_x + 12) as u32, (item_y + item_height / 2) as u32,
                    (menu_width - 24) as u32, 1,
                    GREEN_GHOST
                );
            } else {
                let text_color = if is_hovered { GREEN_SECONDARY } else { GREEN_TERTIARY };
                self.draw_text(menu_x + 16, item_y + 6, &item.label, text_color);
            }
        }
    }
    
    fn draw_background(&self) {
        let height = self.height - TASKBAR_HEIGHT;
        let theme_colors = crate::theme::colors();
        let wallpaper_cfg = &crate::theme::THEME.read().wallpaper;
        
        // ═══════════════════════════════════════════════════════════════
        // DYNAMIC THEME BACKGROUND
        // ═══════════════════════════════════════════════════════════════
        
        // Check if we have a loaded wallpaper
        if let Some(ref wp_data) = wallpaper_cfg.data {
            // Draw wallpaper from loaded image
            self.draw_wallpaper_image(wp_data, height);
        } else {
            // Fallback: gradient from theme colors
            let bg = theme_colors.background;
            let surface = theme_colors.surface;
            
            // Extract RGB components
            let (tr, tg, tb) = (
                ((bg >> 16) & 0xFF) as u32,
                ((bg >> 8) & 0xFF) as u32,
                (bg & 0xFF) as u32
            );
            let (br, bgg, bb) = (
                ((surface >> 16) & 0xFF) as u32,
                ((surface >> 8) & 0xFF) as u32,
                (surface & 0xFF) as u32
            );
            
            // Draw gradient with larger steps for performance
            let mut y = 0u32;
            while y < height {
                let t = y as f32 / height as f32;
                let r = (tr as f32 * (1.0 - t) + br as f32 * t) as u32;
                let g = (tg as f32 * (1.0 - t) + bgg as f32 * t) as u32;
                let b = (tb as f32 * (1.0 - t) + bb as f32 * t) as u32;
                let color = 0xFF000000 | (r << 16) | (g << 8) | b;
                framebuffer::draw_hline(0, y, self.width, color);
                if y + 1 < height {
                    framebuffer::draw_hline(0, y + 1, self.width, color);
                }
                y += 2;
            }
            
            // Accent glow in center
            let cx = self.width / 2;
            let cy = height / 2;
            let accent = theme_colors.accent;
            
            let sizes: [(u32, u32); 3] = [(200, 150), (120, 90), (60, 45)];
            for (i, (w, h)) in sizes.iter().enumerate() {
                let alpha = (3 - i as u32) * 6;
                let color = (alpha << 24) | (accent & 0x00FFFFFF);
                let rx = cx.saturating_sub(*w / 2);
                let ry = cy.saturating_sub(*h / 2);
                framebuffer::fill_rect(rx, ry, *w, *h, color);
            }
        }
    }
    
    /// Draw wallpaper from loaded image data
    fn draw_wallpaper_image(&self, wp_data: &crate::theme::WallpaperData, screen_height: u32) {
        use crate::theme::WallpaperMode;
        let mode = crate::theme::THEME.read().wallpaper.mode;
        
        match mode {
            WallpaperMode::Stretch => {
                // Scale to fit screen
                for sy in 0..screen_height {
                    let src_y = (sy as u64 * wp_data.height as u64 / screen_height as u64) as u32;
                    for sx in 0..self.width {
                        let src_x = (sx as u64 * wp_data.width as u64 / self.width as u64) as u32;
                        let idx = (src_y * wp_data.width + src_x) as usize;
                        if idx < wp_data.pixels.len() {
                            framebuffer::put_pixel(sx, sy, wp_data.pixels[idx]);
                        }
                    }
                }
            }
            WallpaperMode::Center => {
                // Center the image
                let bg = crate::theme::colors().background;
                framebuffer::fill_rect(0, 0, self.width, screen_height, bg);
                
                let offset_x = self.width.saturating_sub(wp_data.width) / 2;
                let offset_y = screen_height.saturating_sub(wp_data.height) / 2;
                
                for y in 0..wp_data.height.min(screen_height) {
                    for x in 0..wp_data.width.min(self.width) {
                        let idx = (y * wp_data.width + x) as usize;
                        if idx < wp_data.pixels.len() {
                            framebuffer::put_pixel(offset_x + x, offset_y + y, wp_data.pixels[idx]);
                        }
                    }
                }
            }
            WallpaperMode::Tile => {
                // Tile the image
                let mut dy = 0;
                while dy < screen_height {
                    let mut dx = 0;
                    while dx < self.width {
                        for y in 0..wp_data.height {
                            if dy + y >= screen_height { break; }
                            for x in 0..wp_data.width {
                                if dx + x >= self.width { break; }
                                let idx = (y * wp_data.width + x) as usize;
                                if idx < wp_data.pixels.len() {
                                    framebuffer::put_pixel(dx + x, dy + y, wp_data.pixels[idx]);
                                }
                            }
                        }
                        dx += wp_data.width;
                    }
                    dy += wp_data.height;
                }
            }
            _ => {
                // Solid color fallback
                let color = crate::theme::THEME.read().wallpaper.fallback_color;
                framebuffer::fill_rect(0, 0, self.width, screen_height, color);
            }
        }
    }
    
    /// Draw TrustOS logo watermark - Windows 11 style
    fn draw_logo_watermark(&self) {
        let center_x = self.width / 2;
        let center_y = (self.height - TASKBAR_HEIGHT) / 2;
        
        // Very subtle color - barely visible
        let logo_color = 0xFF0A0E0B;
        let logo_outline = 0xFF0C100D;
        
        // Shield shape
        let shield_w = 60u32;
        let shield_h = 75u32;
        let sx = center_x - shield_w / 2;
        let sy = center_y - shield_h / 2 - 10;
        
        // Draw shield with gradient
        for y in 0..shield_h {
            let ratio = y as f32 / shield_h as f32;
            let width_factor = if ratio < 0.4 {
                1.0
            } else {
                let t = (ratio - 0.4) / 0.6;
                1.0 - t * t // Smoother curve
            };
            let w = (shield_w as f32 * width_factor).max(2.0) as u32;
            let x_offset = (shield_w - w) / 2;
            
            // Fill with slight gradient
            let brightness = ((1.0 - ratio * 0.3) * 12.0) as u32;
            let fill = 0xFF000000 | (brightness << 16) | ((brightness + 2) << 8) | brightness;
            framebuffer::draw_hline(sx + x_offset, sy + y, w, fill);
        }
        
        // Outline
        for y in 0..shield_h {
            let ratio = y as f32 / shield_h as f32;
            let t = if ratio < 0.4 { 0.0 } else { (ratio - 0.4) / 0.6 };
            let width_factor = if ratio < 0.4 { 1.0 } else { 1.0 - square(t) };
            let w = (shield_w as f32 * width_factor).max(2.0) as u32;
            let x_offset = (shield_w - w) / 2;
            framebuffer::put_pixel(sx + x_offset, sy + y, logo_outline);
            if w > 1 {
                framebuffer::put_pixel(sx + x_offset + w - 1, sy + y, logo_outline);
            }
        }
        
        // Lock icon inside shield
        let lock_x = center_x - 8;
        let lock_y = sy + 20;
        // Lock body
        framebuffer::fill_rect(lock_x, lock_y + 8, 16, 12, logo_outline);
        // Lock shackle (arc)
        framebuffer::draw_rect(lock_x + 3, lock_y, 10, 10, logo_outline);
        framebuffer::fill_rect(lock_x + 5, lock_y + 2, 6, 6, logo_color);
        
        // "TrustOS" text - subtle
        let text = "TrustOS";
        let text_x = (center_x - text.len() as u32 * 4) as i32;
        let text_y = (sy + shield_h + 12) as i32;
        self.draw_text(text_x, text_y, text, logo_outline);
        
        // Version even more subtle
        let version = "v0.1.0";
        let ver_x = (center_x - version.len() as u32 * 4) as i32;
        let ver_y = text_y + 14;
        self.draw_text(ver_x, ver_y, version, 0xFF080A09);
    }
    
    fn draw_desktop_icons(&self) {
        use crate::icons;
        
        // Draw dock-style icons on the left
        for icon in self.icons.iter() {
            let x = icon.x;
            let y = icon.y;
            
            // Check if cursor is over this icon (48x64 hit area)
            let is_hovered = self.cursor_x >= x as i32 && self.cursor_x < (x + 64) as i32
                && self.cursor_y >= y as i32 && self.cursor_y < (y + 72) as i32;
            
            // Background container
            let icon_bg = if is_hovered { BG_LIGHT } else { 0x00000000 };
            if is_hovered {
                // Halo/glow effect
                self.draw_icon_glow(x, y, 64, 52);
                framebuffer::fill_rect(x + 8, y + 4, 48, 48, icon_bg);
                framebuffer::draw_rect(x + 8, y + 4, 48, 48, GREEN_MUTED);
            }
            
            // Draw the actual pixel-art icon
            let icon_color = if is_hovered { GREEN_PRIMARY } else { GREEN_SECONDARY };
            icons::draw_icon(icon.icon_type, x + 16, y + 10, icon_color, icon_bg);
            
            // Label below - smaller, muted
            let name = &icon.name;
            let name_x = x + 32 - (name.len() as u32 * 4);
            let label_color = if is_hovered { GREEN_SECONDARY } else { GREEN_TERTIARY };
            self.draw_text(name_x as i32, (y + 56) as i32, name, label_color);
        }
    }
    
    /// Draw a subtle glow effect around an icon
    fn draw_icon_glow(&self, x: u32, y: u32, w: u32, h: u32) {
        // Multiple layers of increasingly transparent green
        let layers = [
            (0xFF001108, 6),
            (0xFF001A0D, 4),
            (0xFF002211, 2),
        ];
        
        for (color, offset) in layers {
            let ox = if x > offset { x - offset } else { 0 };
            let oy = if y > offset { y - offset } else { 0 };
            framebuffer::draw_rect(ox + 8, oy + 4, w + offset * 2 - 16, h + offset * 2 - 8, color);
        }
    }
    
    fn draw_taskbar(&self) {
        use crate::gui::windows11::colors;
        
        let y = self.height - TASKBAR_HEIGHT;
        
        // ═══════════════════════════════════════════════════════════════
        // Windows 11 Style Taskbar (centered icons)
        // ═══════════════════════════════════════════════════════════════
        
        // Acrylic-like background
        framebuffer::fill_rect(0, y, self.width, TASKBAR_HEIGHT, colors::TASKBAR_BG);
        
        // Subtle top border
        framebuffer::draw_hline(0, y, self.width, colors::BORDER_SUBTLE);
        
        // Start button (left side for now, Win11 centers it but left is more familiar)
        let start_hover = self.cursor_x >= 4 && self.cursor_x < 52 && self.cursor_y >= y as i32;
        if start_hover || self.start_menu_open {
            // Rounded hover background
            crate::gui::windows11::draw_rounded_rect(4, (y + 4) as i32, 48, TASKBAR_HEIGHT - 8, 4, colors::SURFACE_HOVER);
        }
        
        // Windows logo (simplified as 4 squares)
        let logo_x = 16;
        let logo_y = (y + 12) as i32;
        let logo_color = if start_hover { colors::ACCENT_LIGHT } else { colors::ACCENT };
        framebuffer::fill_rect(logo_x, logo_y as u32, 7, 7, logo_color);
        framebuffer::fill_rect(logo_x + 9, logo_y as u32, 7, 7, logo_color);
        framebuffer::fill_rect(logo_x, (logo_y + 9) as u32, 7, 7, logo_color);
        framebuffer::fill_rect(logo_x + 9, (logo_y + 9) as u32, 7, 7, logo_color);
        
        // Window buttons (centered in taskbar)
        let total_btns = self.windows.len();
        let btn_w = 48u32;
        let btn_gap = 4u32;
        let total_w = total_btns as u32 * (btn_w + btn_gap);
        let start_x = (self.width - total_w) / 2;
        
        for (i, w) in self.windows.iter().enumerate() {
            let btn_x = start_x + i as u32 * (btn_w + btn_gap);
            let btn_y = y + 4;
            
            let is_hover = self.cursor_x >= btn_x as i32 && self.cursor_x < (btn_x + btn_w) as i32
                && self.cursor_y >= y as i32;
            
            // Button background
            let bg = if w.focused { 
                colors::SURFACE_PRESSED
            } else if is_hover { 
                colors::SURFACE_HOVER
            } else { 
                colors::SURFACE
            };
            crate::gui::windows11::draw_rounded_rect(btn_x as i32, btn_y as i32, btn_w, TASKBAR_HEIGHT - 8, 4, bg);
            
            // Active indicator (accent line below)
            if w.focused {
                let indicator_w = 20u32;
                let indicator_x = btn_x + (btn_w - indicator_w) / 2;
                crate::gui::windows11::draw_rounded_rect(indicator_x as i32, (y + TASKBAR_HEIGHT - 5) as i32, indicator_w, 3, 1, colors::ACCENT);
            } else if !w.minimized {
                // Small dot for open windows
                let dot_x = btn_x + btn_w / 2 - 2;
                framebuffer::fill_rect(dot_x, y + TASKBAR_HEIGHT - 4, 4, 2, colors::TEXT_SECONDARY);
            }
            
            // App icon (first letter)
            let first_char = w.title.chars().next().unwrap_or(' ');
            let mut icon_buf = [0u8; 4];
            let icon_str = first_char.encode_utf8(&mut icon_buf);
            self.draw_text((btn_x + btn_w / 2 - 4) as i32, (btn_y + 8) as i32, icon_str, colors::TEXT_PRIMARY);
        }
        
        // System tray (right side)
        let tray_x = self.width - 120;
        
        // Clock
        let time = self.get_time_string();
        let date = self.get_date_string();
        self.draw_text((tray_x + 20) as i32, (y + 6) as i32, &time, colors::TEXT_PRIMARY);
        self.draw_text((tray_x + 12) as i32, (y + 20) as i32, &date, colors::TEXT_SECONDARY);
        
        // Notification area hover
        let tray_hover = self.cursor_x >= tray_x as i32 && self.cursor_y >= y as i32;
        if tray_hover {
            crate::gui::windows11::draw_rounded_rect((tray_x - 4) as i32, (y + 2) as i32, 120, TASKBAR_HEIGHT - 4, 4, colors::SURFACE_HOVER);
        }
    }
    
    fn get_time_string(&self) -> String {
        let dt = crate::rtc::read_rtc();
        format!("{:02}:{:02}", dt.hour, dt.minute)
    }
    
    fn get_date_string(&self) -> String {
        let dt = crate::rtc::read_rtc();
        // Format: MM/DD/YYYY (US style) or DD/MM (shorter)
        format!("{:02}/{:02}", dt.month, dt.day)
    }
    
    fn draw_start_menu(&self) {
        use crate::gui::windows11::colors;
        
        let menu_w = 320u32;
        let menu_h = 450u32;
        let menu_x = 8i32;
        let menu_y = (self.height - TASKBAR_HEIGHT - menu_h - 8) as i32;
        
        // ═══════════════════════════════════════════════════════════════
        // Windows 11 Style Start Menu
        // ═══════════════════════════════════════════════════════════════
        
        // Drop shadow
        crate::gui::windows11::draw_shadow(menu_x, menu_y, menu_w, menu_h, 8, 0, 8, 5);
        
        // Main background with rounded corners
        crate::gui::windows11::draw_rounded_rect(menu_x, menu_y, menu_w, menu_h, 8, colors::MICA_DARK);
        crate::gui::windows11::draw_rounded_rect_border(menu_x, menu_y, menu_w, menu_h, 8, colors::BORDER_SUBTLE);
        
        // Search bar at top
        let search_y = menu_y + 16;
        crate::gui::windows11::draw_rounded_rect(menu_x + 16, search_y, menu_w - 32, 36, 4, colors::SURFACE);
        crate::gui::windows11::draw_rounded_rect_border(menu_x + 16, search_y, menu_w - 32, 36, 4, colors::BORDER_DEFAULT);
        self.draw_text(menu_x + 28, search_y + 10, "Type to search", colors::TEXT_DISABLED);
        
        // "Pinned" section
        let pinned_y = search_y + 52;
        self.draw_text(menu_x + 16, pinned_y, "Pinned", colors::TEXT_PRIMARY);
        
        // App grid (2 rows x 4 columns)
        let apps = [
            ("Terminal", ">_"),
            ("Files", "[]"),
            ("Settings", "@@"),
            ("Editor", "Ed"),
            ("Network", "~~"),
            ("About", "?i"),
            ("Calc", "##"),
            ("3D Demo", "3D"),
        ];
        
        let grid_y = pinned_y + 24;
        let cell_w = 72;
        let cell_h = 70;
        
        for (i, (name, icon)) in apps.iter().enumerate() {
            let col = i % 4;
            let row = i / 4;
            let cx = menu_x + 16 + (col as i32 * cell_w);
            let cy = grid_y + (row as i32 * cell_h);
            
            // Hover detection
            let hover = self.cursor_x >= cx && self.cursor_x < cx + cell_w
                && self.cursor_y >= cy && self.cursor_y < cy + cell_h;
            
            if hover {
                crate::gui::windows11::draw_rounded_rect(cx + 2, cy + 2, (cell_w - 4) as u32, (cell_h - 4) as u32, 4, colors::SURFACE_HOVER);
            }
            
            // Icon (larger)
            self.draw_text(cx + 24, cy + 12, icon, colors::ACCENT);
            
            // Name (centered)
            let name_w = name.len() as i32 * 8;
            let name_x = cx + (cell_w - name_w) / 2;
            self.draw_text(name_x, cy + 44, name, colors::TEXT_PRIMARY);
        }
        
        // "Recommended" section  
        let rec_y = grid_y + cell_h * 2 + 16;
        self.draw_text(menu_x + 16, rec_y, "Recommended", colors::TEXT_PRIMARY);
        
        // Separator line
        framebuffer::draw_hline((menu_x + 16) as u32, (rec_y + 24) as u32, menu_w - 32, colors::BORDER_SUBTLE);
        
        // User profile at bottom
        let user_y = menu_y + menu_h as i32 - 60;
        framebuffer::draw_hline((menu_x + 16) as u32, user_y as u32, menu_w - 32, colors::BORDER_SUBTLE);
        
        // User icon (circle with initial)
        let user_icon_x = menu_x + 24;
        let user_icon_y = user_y + 16;
        crate::gui::windows11::draw_rounded_rect(user_icon_x, user_icon_y, 32, 32, 16, colors::ACCENT);
        self.draw_text(user_icon_x + 10, user_icon_y + 8, "U", colors::TEXT_PRIMARY);
        
        // User name
        self.draw_text(user_icon_x + 44, user_icon_y + 8, "User", colors::TEXT_PRIMARY);
        
        // Power button (right side)
        let power_x = menu_x + menu_w as i32 - 48;
        let power_hover = self.cursor_x >= power_x && self.cursor_x < power_x + 32
            && self.cursor_y >= user_icon_y && self.cursor_y < user_icon_y + 32;
        
        if power_hover {
            crate::gui::windows11::draw_rounded_rect(power_x, user_icon_y, 32, 32, 4, colors::SURFACE_HOVER);
        }
        self.draw_text(power_x + 8, user_icon_y + 8, "O", colors::TEXT_SECONDARY);
        
        // TrustOS branding in header
        self.draw_text((menu_x + 12) as i32, (menu_y + 6) as i32, "T", GREEN_PRIMARY);
        self.draw_text((menu_x + 24) as i32, (menu_y + 6) as i32, "TrustOS", GREEN_SECONDARY);
        self.draw_text((menu_x + 12) as i32, (menu_y + 22) as i32, "v0.1.0", GREEN_SUBTLE);
        
        // Separator line
        framebuffer::fill_rect((menu_x + 12) as u32, (menu_y + 42) as u32, menu_w - 24, 1, GREEN_GHOST);
        
        // Menu items with modern styling
        let items = [
            (">_", "Terminal", false),
            ("[]", "Files", false),
            ("##", "Calculator", false),
            ("~~", "Network", false),
            ("@)", "Settings", false),
            ("(i)", "About", false),
            ("!!", "Shutdown", true),
        ];
        
        for (i, (icon, label, is_danger)) in items.iter().enumerate() {
            let item_y = menu_y + 50 + (i as i32 * 28);
            
            // Hover detection
            let is_hovered = self.cursor_x >= menu_x 
                && self.cursor_x < menu_x + menu_w as i32
                && self.cursor_y >= item_y 
                && self.cursor_y < item_y + 26;
            
            if is_hovered {
                // Hover glow effect
                framebuffer::fill_rect((menu_x + 6) as u32, item_y as u32, menu_w - 12, 26, GREEN_GHOST);
                framebuffer::fill_rect((menu_x + 8) as u32, (item_y + 1) as u32, menu_w - 16, 24, BG_LIGHT);
                
                // Left accent bar on hover
                framebuffer::fill_rect((menu_x + 6) as u32, (item_y + 4) as u32, 2, 18, 
                    if *is_danger { BTN_CLOSE } else { GREEN_PRIMARY });
            }
            
            // Icon with muted color
            let icon_color = if is_hovered {
                if *is_danger { BTN_CLOSE_HOVER } else { GREEN_PRIMARY }
            } else {
                if *is_danger { BTN_CLOSE } else { GREEN_TERTIARY }
            };
            self.draw_text((menu_x + 16) as i32, (item_y + 6) as i32, icon, icon_color);
            
            // Label
            let label_color = if is_hovered {
                if *is_danger { BTN_CLOSE_HOVER } else { GREEN_SECONDARY }
            } else {
                if *is_danger { 0xFF994444 } else { GREEN_SECONDARY }
            };
            self.draw_text((menu_x + 44) as i32, (item_y + 6) as i32, label, label_color);
        }
    }
    
    fn draw_window(&self, window: &Window) {
        use crate::gui::windows11::{self as w11, colors};
        
        let x = window.x;
        let y = window.y;
        let w = window.width;
        let h = window.height;
        let radius = if window.maximized { 0 } else { 8 };
        
        // ═══════════════════════════════════════════════════════════════
        // Windows 11 Style Window Rendering
        // ═══════════════════════════════════════════════════════════════
        
        // Drop shadow (only for focused, non-maximized windows)
        if !window.maximized && window.focused {
            w11::draw_shadow(x, y, w, h, radius, 0, 6, 4);
        } else if !window.maximized {
            // Subtle shadow for unfocused
            w11::draw_shadow(x, y, w, h, radius, 0, 2, 2);
        }
        
        // Window background with rounded corners
        w11::draw_rounded_rect(x, y, w, h, radius, colors::MICA_DARK);
        
        // Accent border for focused window
        let border_color = if window.focused { colors::ACCENT } else { colors::BORDER_SUBTLE };
        w11::draw_rounded_rect_border(x, y, w, h, radius, border_color);
        
        // Title bar
        let titlebar_h = TITLE_BAR_HEIGHT;
        let title_bg = if window.focused { colors::TITLEBAR_ACTIVE } else { colors::TITLEBAR_INACTIVE };
        
        // Draw title bar (respecting rounded corners)
        if radius > 0 {
            // Fill top corners
            for dy in 0..radius {
                for dx in 0..radius {
                    if dx * dx + dy * dy <= radius * radius {
                        // Top left corner
                        framebuffer::draw_pixel((x + radius as i32 - dx as i32 - 1) as u32, (y + radius as i32 - dy as i32 - 1) as u32, title_bg);
                        // Top right corner  
                        framebuffer::draw_pixel((x + w as i32 - radius as i32 + dx as i32) as u32, (y + radius as i32 - dy as i32 - 1) as u32, title_bg);
                    }
                }
            }
            framebuffer::fill_rect((x + radius as i32) as u32, y as u32, w - radius * 2, radius, title_bg);
        }
        framebuffer::fill_rect((x + 1) as u32, (y + radius as i32) as u32, w - 2, titlebar_h - radius, title_bg);
        
        // Window icon and title
        let text_color = if window.focused { colors::TEXT_PRIMARY } else { colors::TEXT_SECONDARY };
        let icon = match window.window_type {
            WindowType::Terminal => "⌘",
            WindowType::FileManager => "📁",
            WindowType::Calculator => "🔢",
            WindowType::NetworkInfo => "🌐",
            WindowType::About => "ℹ",
            WindowType::Settings => "⚙",
            WindowType::FileAssociations => "📎",
            WindowType::SystemInfo => "💻",
            WindowType::Empty => "◻",
            WindowType::TextEditor => "📝",
            WindowType::ImageViewer => "🖼",
            WindowType::HexViewer => "🔍",
            WindowType::Demo3D => "🎮",
            WindowType::Game => "🎯",
        };
        
        // Icon (simple 2-char representation for now)
        let icon_str = match window.window_type {
            WindowType::Terminal => ">_",
            WindowType::FileManager => "[]",
            WindowType::Calculator => "##",
            _ => "::",
        };
        self.draw_text(x + 12, y + 8, icon_str, colors::ACCENT);
        
        // Title text
        self.draw_text(x + 36, y + 8, &window.title, text_color);
        
        // ═══════════════════════════════════════════════════════════════
        // Window Control Buttons (Windows 11 style)
        // ═══════════════════════════════════════════════════════════════
        let btn_w = 46;
        let btn_h = titlebar_h;
        let btn_y = y;
        let mx = self.cursor_x;
        let my = self.cursor_y;
        
        // Close button (rightmost) - red on hover
        let close_x = x + w as i32 - btn_w as i32;
        let close_hover = mx >= close_x && mx < close_x + btn_w as i32 
            && my >= btn_y && my < btn_y + btn_h as i32;
        if close_hover {
            framebuffer::fill_rect(close_x as u32, btn_y as u32, btn_w as u32, btn_h, colors::CLOSE_HOVER);
        }
        // X icon
        let cx = close_x + btn_w as i32 / 2;
        let cy = btn_y + btn_h as i32 / 2;
        let icon_color = if close_hover { colors::TEXT_PRIMARY } else { text_color };
        for i in -4..=4i32 {
            framebuffer::draw_pixel((cx + i) as u32, (cy + i) as u32, icon_color);
            framebuffer::draw_pixel((cx + i) as u32, (cy - i) as u32, icon_color);
        }
        
        // Maximize/Restore button
        let max_x = close_x - btn_w as i32;
        let max_hover = mx >= max_x && mx < max_x + btn_w as i32
            && my >= btn_y && my < btn_y + btn_h as i32;
        if max_hover {
            framebuffer::fill_rect(max_x as u32, btn_y as u32, btn_w as u32, btn_h, colors::CONTROL_HOVER);
        }
        let icon_color = if max_hover { colors::TEXT_PRIMARY } else { text_color };
        if window.maximized {
            // Restore icon (two overlapping squares)
            framebuffer::draw_rect((cx - btn_w as i32 - 2) as u32, (cy - 4) as u32, 8, 8, icon_color);
            framebuffer::draw_rect((cx - btn_w as i32 - 4) as u32, (cy - 2) as u32, 8, 8, icon_color);
        } else {
            // Maximize icon (square)
            framebuffer::draw_rect((cx - btn_w as i32 - 4) as u32, (cy - 4) as u32, 10, 10, icon_color);
        }
        
        // Minimize button
        let min_x = max_x - btn_w as i32;
        let min_hover = mx >= min_x && mx < min_x + btn_w as i32
            && my >= btn_y && my < btn_y + btn_h as i32;
        if min_hover {
            framebuffer::fill_rect(min_x as u32, btn_y as u32, btn_w as u32, btn_h, colors::CONTROL_HOVER);
        }
        let icon_color = if min_hover { colors::TEXT_PRIMARY } else { text_color };
        framebuffer::fill_rect((min_x + btn_w as i32 / 2 - 5) as u32, (cy) as u32, 10, 1, icon_color);
        
        // ═══════════════════════════════════════════════════════════════
        // Content Area
        // ═══════════════════════════════════════════════════════════════
        let content_y = y + titlebar_h as i32;
        let content_h = h - titlebar_h;
        framebuffer::fill_rect((x + 1) as u32, content_y as u32, w - 2, content_h - 1, colors::MICA_DARKER);
        
        // Resize handle (bottom-right corner dots)
        if !window.maximized {
            let rx = x + w as i32 - 16;
            let ry = y + h as i32 - 16;
            for i in 0..3 {
                for j in 0..3 {
                    if i + j >= 2 {
                        framebuffer::draw_pixel((rx + i * 5) as u32, (ry + j * 5) as u32, colors::BORDER_DEFAULT);
                    }
                }
            }
        }
        
        // Draw window content
        self.draw_window_content(window);
    }
    
    /// Draw a modern minimal button (legacy - kept for compatibility)
    fn draw_modern_button(&self, x: u32, y: u32, size: u32, color: u32, hovered: bool) {
        if hovered {
            // Glow effect on hover
            framebuffer::fill_rect(x.saturating_sub(1), y.saturating_sub(1), size + 2, size + 2, 
                (color & 0x00FFFFFF) | 0x40000000);
        }
        framebuffer::fill_rect(x, y, size, size, color);
    }
    
    /// Blend two colors with a factor (0.0 = first color, 1.0 = second color)
    fn blend_colors(&self, c1: u32, c2: u32, t: f32) -> u32 {
        let r1 = ((c1 >> 16) & 0xFF) as f32;
        let g1 = ((c1 >> 8) & 0xFF) as f32;
        let b1 = (c1 & 0xFF) as f32;
        let r2 = ((c2 >> 16) & 0xFF) as f32;
        let g2 = ((c2 >> 8) & 0xFF) as f32;
        let b2 = (c2 & 0xFF) as f32;
        
        let r = (r1 + (r2 - r1) * t) as u32;
        let g = (g1 + (g2 - g1) * t) as u32;
        let b = (b1 + (b2 - b1) * t) as u32;
        
        0xFF000000 | (r << 16) | (g << 8) | b
    }
    
    fn draw_window_content(&self, window: &Window) {
        let content_x = window.x + 8;
        let content_y = window.y + TITLE_BAR_HEIGHT as i32 + 8;
        
        // Special rendering for 3D demo window
        if window.window_type == WindowType::Demo3D {
            self.draw_3d_demo(window);
            return;
        }
        
        // Special rendering for Snake game
        if window.window_type == WindowType::Game {
            self.draw_snake_game(window);
            return;
        }
        
        // Check if this window type needs selection highlighting
        let needs_selection = matches!(window.window_type, 
            WindowType::FileManager | WindowType::FileAssociations);
        
        // Calculate which lines are selectable (skip headers)
        let (sel_start, sel_end) = match window.window_type {
            WindowType::FileManager => (5, window.content.len().saturating_sub(2)),
            WindowType::FileAssociations => (4, window.content.len().saturating_sub(2)),
            _ => (0, 0),
        };
        
        for (i, line) in window.content.iter().enumerate() {
            let line_y = content_y + (i as i32 * 16);
            if line_y >= window.y + window.height as i32 - 8 {
                break;
            }
            
            // Check if this line is selected
            let is_selected = needs_selection 
                && i >= sel_start 
                && i < sel_end 
                && (i - sel_start) == window.selected_index;
            
            if is_selected {
                // Draw selection highlight
                framebuffer::fill_rect(
                    content_x as u32 - 4, 
                    line_y as u32 - 2, 
                    window.width - 16, 
                    18, 
                    0xFF003300
                );
                self.draw_text(content_x, line_y, line, COLOR_BRIGHT_GREEN);
            } else {
                self.draw_text(content_x, line_y, line, COLOR_GREEN);
            }
        }
    }
    
    /// Draw 3D graphics demo using TrustGL (OpenGL-like API)
    fn draw_3d_demo(&self, window: &Window) {
        use crate::graphics::opengl::*;
        
        let demo_x = window.x as u32 + 10;
        let demo_y = window.y as u32 + TITLE_BAR_HEIGHT + 10;
        let demo_w = window.width.saturating_sub(20);
        let demo_h = window.height.saturating_sub(TITLE_BAR_HEIGHT + 20);
        
        if demo_w < 80 || demo_h < 80 {
            return;
        }
        
        // Initialize TrustGL for this viewport
        gl_init(demo_w, demo_h);
        gl_viewport(demo_x as i32, demo_y as i32, demo_w, demo_h);
        
        // Clear with dark background
        gl_clear_color(0.04, 0.06, 0.04, 1.0);
        gl_clear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);
        
        // Enable depth testing
        gl_enable(GL_DEPTH_TEST);
        
        // Set up projection matrix
        let aspect = demo_w as f32 / demo_h as f32;
        gl_matrix_mode(GL_PROJECTION);
        gl_load_identity();
        glu_perspective(45.0, aspect, 0.1, 100.0);
        
        // Set up modelview matrix
        gl_matrix_mode(GL_MODELVIEW);
        gl_load_identity();
        glu_look_at(
            3.0, 2.0, 4.0,   // eye position
            0.0, 0.0, 0.0,   // look at center
            0.0, 1.0, 0.0    // up vector
        );
        
        // Simple rotation based on a static angle (could be animated with time)
        let angle = (self.frame_count as f32 * 0.5) % 360.0;
        gl_rotatef(angle, 0.0, 1.0, 0.0);
        gl_rotatef(angle * 0.3, 1.0, 0.0, 0.0);
        
        // Draw a colorful cube using immediate mode
        let s = 0.8;
        
        gl_begin(GL_QUADS);
        
        // Front face (red)
        gl_color3f(1.0, 0.2, 0.2);
        gl_normal3f(0.0, 0.0, 1.0);
        gl_vertex3f(-s, -s, s);
        gl_vertex3f(s, -s, s);
        gl_vertex3f(s, s, s);
        gl_vertex3f(-s, s, s);
        
        // Back face (green)
        gl_color3f(0.2, 1.0, 0.2);
        gl_normal3f(0.0, 0.0, -1.0);
        gl_vertex3f(s, -s, -s);
        gl_vertex3f(-s, -s, -s);
        gl_vertex3f(-s, s, -s);
        gl_vertex3f(s, s, -s);
        
        // Top face (blue)
        gl_color3f(0.2, 0.2, 1.0);
        gl_normal3f(0.0, 1.0, 0.0);
        gl_vertex3f(-s, s, s);
        gl_vertex3f(s, s, s);
        gl_vertex3f(s, s, -s);
        gl_vertex3f(-s, s, -s);
        
        // Bottom face (yellow)
        gl_color3f(1.0, 1.0, 0.2);
        gl_normal3f(0.0, -1.0, 0.0);
        gl_vertex3f(-s, -s, -s);
        gl_vertex3f(s, -s, -s);
        gl_vertex3f(s, -s, s);
        gl_vertex3f(-s, -s, s);
        
        // Right face (magenta)
        gl_color3f(1.0, 0.2, 1.0);
        gl_normal3f(1.0, 0.0, 0.0);
        gl_vertex3f(s, -s, s);
        gl_vertex3f(s, -s, -s);
        gl_vertex3f(s, s, -s);
        gl_vertex3f(s, s, s);
        
        // Left face (cyan)
        gl_color3f(0.2, 1.0, 1.0);
        gl_normal3f(-1.0, 0.0, 0.0);
        gl_vertex3f(-s, -s, -s);
        gl_vertex3f(-s, -s, s);
        gl_vertex3f(-s, s, s);
        gl_vertex3f(-s, s, -s);
        
        gl_end();
        
        // Draw coordinate axes with lines
        gl_load_identity();
        glu_look_at(3.0, 2.0, 4.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0);
        
        gl_begin(GL_LINES);
        // X axis (red)
        gl_color3f(1.0, 0.0, 0.0);
        gl_vertex3f(0.0, 0.0, 0.0);
        gl_vertex3f(2.0, 0.0, 0.0);
        // Y axis (green)
        gl_color3f(0.0, 1.0, 0.0);
        gl_vertex3f(0.0, 0.0, 0.0);
        gl_vertex3f(0.0, 2.0, 0.0);
        // Z axis (blue)
        gl_color3f(0.0, 0.0, 1.0);
        gl_vertex3f(0.0, 0.0, 0.0);
        gl_vertex3f(0.0, 0.0, 2.0);
        gl_end();
        
        // Draw text overlay
        self.draw_text(demo_x as i32 + 8, demo_y as i32 + 8, "TrustGL OpenGL Demo", GREEN_SECONDARY);
        self.draw_text(demo_x as i32 + 8, demo_y as i32 + 24, "Software 3D Renderer", GREEN_TERTIARY);
        
        // Stats
        let stats_y = demo_y as i32 + demo_h as i32 - 24;
        self.draw_text(demo_x as i32 + 8, stats_y, "Rotating Cube | Depth Test ON", GREEN_MUTED);
        self.draw_text(demo_x as i32 + 8, stats_y, "Vertices: 8 | Edges: 12 | Faces: 6", GREEN_MUTED);
    }
    
    /// Draw a simple line using Bresenham's algorithm
    fn draw_line_simple(&self, x0: i32, y0: i32, x1: i32, y1: i32, color: u32) {
        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        let mut x = x0;
        let mut y = y0;
        
        loop {
            if x >= 0 && y >= 0 && (x as u32) < self.width && (y as u32) < self.height {
                framebuffer::put_pixel(x as u32, y as u32, color);
            }
            if x == x1 && y == y1 { break; }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x += sx;
            }
            if e2 <= dx {
                err += dx;
                y += sy;
            }
        }
    }
    
    /// Draw Snake game window content
    fn draw_snake_game(&self, window: &Window) {
        let game_x = window.x as u32 + 10;
        let game_y = window.y as u32 + TITLE_BAR_HEIGHT + 10;
        let game_w = window.width.saturating_sub(20);
        let game_h = window.height.saturating_sub(TITLE_BAR_HEIGHT + 20);
        
        if game_w < 80 || game_h < 80 {
            return;
        }
        
        // Draw game area background
        framebuffer::fill_rect(game_x, game_y, game_w, game_h, 0xFF0A0E0B);
        
        // Draw border
        for i in 0..game_w {
            framebuffer::put_pixel(game_x + i, game_y, GREEN_MUTED);
            framebuffer::put_pixel(game_x + i, game_y + game_h - 1, GREEN_MUTED);
        }
        for i in 0..game_h {
            framebuffer::put_pixel(game_x, game_y + i, GREEN_MUTED);
            framebuffer::put_pixel(game_x + game_w - 1, game_y + i, GREEN_MUTED);
        }
        
        // Draw a simple static snake demo
        let cell_size: u32 = 12;
        let grid_offset_x = game_x + 20;
        let grid_offset_y = game_y + 40;
        
        // Snake body segments (demo positions)
        let snake_segments = [
            (8, 5), (7, 5), (6, 5), (5, 5), (4, 5), (3, 5), (2, 5)
        ];
        
        // Draw snake
        for (i, &(sx, sy)) in snake_segments.iter().enumerate() {
            let px = grid_offset_x + sx * cell_size;
            let py = grid_offset_y + sy * cell_size;
            let color = if i == 0 { 
                0xFF00FF00 // Head is bright green
            } else {
                0xFF00CC00 // Body is slightly darker
            };
            
            // Draw filled cell
            for dy in 1..cell_size-1 {
                for dx in 1..cell_size-1 {
                    if px + dx < game_x + game_w && py + dy < game_y + game_h {
                        framebuffer::put_pixel(px + dx, py + dy, color);
                    }
                }
            }
        }
        
        // Draw food (red apple)
        let food_x = grid_offset_x + 12 * cell_size;
        let food_y = grid_offset_y + 5 * cell_size;
        for dy in 2..cell_size-2 {
            for dx in 2..cell_size-2 {
                if food_x + dx < game_x + game_w && food_y + dy < game_y + game_h {
                    framebuffer::put_pixel(food_x + dx, food_y + dy, 0xFFFF4444);
                }
            }
        }
        
        // Title
        self.draw_text(game_x as i32 + 8, game_y as i32 + 8, "SNAKE", COLOR_BRIGHT_GREEN);
        
        // Instructions
        self.draw_text(game_x as i32 + 8, game_y as i32 + game_h as i32 - 20, 
                       "Use Arrow Keys to Play", GREEN_TERTIARY);
        
        // Score display
        self.draw_text(game_x as i32 + game_w as i32 - 80, game_y as i32 + 8, 
                       "Score: 60", GREEN_SECONDARY);
    }
    
    fn draw_cursor(&self) {
        // OPTIMIZED: Simple cursor without expensive glow calculations
        // The glow effect was causing ~256 sqrt calculations per frame!
        
        // Simple 3-pixel drop shadow (very fast)
        let shadow_color = 0x40000000u32;
        for offset in 1..=2 {
            let sx = self.cursor_x + offset;
            let sy = self.cursor_y + offset;
            if sx >= 0 && sy >= 0 && sx < self.width as i32 && sy < self.height as i32 {
                for dy in 0..12 {
                    let py = (sy + dy) as u32;
                    let px = sx as u32;
                    if py < self.height && px < self.width {
                        framebuffer::put_pixel(px, py, shadow_color);
                    }
                }
            }
        }
        
        // Modern arrow cursor with green accent
        let cursor: [[u8; 12]; 16] = [
            [1,0,0,0,0,0,0,0,0,0,0,0],
            [1,1,0,0,0,0,0,0,0,0,0,0],
            [1,2,1,0,0,0,0,0,0,0,0,0],
            [1,2,2,1,0,0,0,0,0,0,0,0],
            [1,2,2,2,1,0,0,0,0,0,0,0],
            [1,2,2,2,2,1,0,0,0,0,0,0],
            [1,2,2,2,2,2,1,0,0,0,0,0],
            [1,2,2,2,2,2,2,1,0,0,0,0],
            [1,2,2,2,2,2,2,2,1,0,0,0],
            [1,2,2,2,2,2,2,2,2,1,0,0],
            [1,2,2,2,2,2,1,1,1,1,1,0],
            [1,2,2,1,2,2,1,0,0,0,0,0],
            [1,2,1,0,1,2,2,1,0,0,0,0],
            [1,1,0,0,1,2,2,1,0,0,0,0],
            [1,0,0,0,0,1,2,2,1,0,0,0],
            [0,0,0,0,0,1,1,1,1,0,0,0],
        ];
        
        for (cy, row) in cursor.iter().enumerate() {
            for (cx, &pixel) in row.iter().enumerate() {
                if pixel == 0 { continue; }
                let px = (self.cursor_x + cx as i32) as u32;
                let py = (self.cursor_y + cy as i32) as u32;
                if px < self.width && py < self.height {
                    let color = match pixel {
                        1 => GREEN_MUTED,      // Outline in muted green
                        2 => GREEN_SECONDARY,  // Fill in bright green
                        _ => continue,
                    };
                    framebuffer::put_pixel(px, py, color);
                }
            }
        }
    }
    
    fn draw_text(&self, x: i32, y: i32, text: &str, color: u32) {
        // Use framebuffer's text drawing
        let old_fg = framebuffer::get_fg_color();
        framebuffer::set_fg_color(color);
        
        // Simple approach: just set cursor and print
        // For now, we'll draw character by character at pixel positions
        for (i, c) in text.chars().enumerate() {
            let px = x + (i as i32 * 8);
            if px >= 0 && px < self.width as i32 && y >= 0 && y < self.height as i32 {
                self.draw_char(px as u32, y as u32, c, color);
            }
        }
        
        framebuffer::set_fg_color(old_fg);
    }
    
    fn draw_char(&self, x: u32, y: u32, c: char, color: u32) {
        // Simple 8x16 font rendering using basic shapes
        // This is a minimal implementation - in a real OS you'd use a proper font
        let glyph = get_char_glyph(c);
        for (row, &bits) in glyph.iter().enumerate() {
            for col in 0..8 {
                if bits & (0x80 >> col) != 0 {
                    let px = x + col;
                    let py = y + row as u32;
                    if px < self.width && py < self.height {
                        framebuffer::put_pixel(px, py, color);
                    }
                }
            }
        }
    }
}

/// Get simple glyph for character (8x16 bitmap)
fn get_char_glyph(c: char) -> [u8; 16] {
    // Very simple glyphs for basic characters
    match c {
        'A' => [0x00,0x18,0x3C,0x66,0x66,0x7E,0x66,0x66,0x66,0x66,0x00,0x00,0x00,0x00,0x00,0x00],
        'B' => [0x00,0x7C,0x66,0x66,0x7C,0x66,0x66,0x66,0x7C,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'C' => [0x00,0x3C,0x66,0x60,0x60,0x60,0x60,0x66,0x3C,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'D' => [0x00,0x78,0x6C,0x66,0x66,0x66,0x66,0x6C,0x78,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'E' => [0x00,0x7E,0x60,0x60,0x7C,0x60,0x60,0x60,0x7E,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'F' => [0x00,0x7E,0x60,0x60,0x7C,0x60,0x60,0x60,0x60,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'G' => [0x00,0x3C,0x66,0x60,0x60,0x6E,0x66,0x66,0x3C,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'H' => [0x00,0x66,0x66,0x66,0x7E,0x66,0x66,0x66,0x66,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'I' => [0x00,0x3C,0x18,0x18,0x18,0x18,0x18,0x18,0x3C,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'J' => [0x00,0x1E,0x0C,0x0C,0x0C,0x0C,0x6C,0x6C,0x38,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'K' => [0x00,0x66,0x6C,0x78,0x70,0x78,0x6C,0x66,0x66,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'L' => [0x00,0x60,0x60,0x60,0x60,0x60,0x60,0x60,0x7E,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'M' => [0x00,0x63,0x77,0x7F,0x6B,0x63,0x63,0x63,0x63,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'N' => [0x00,0x66,0x76,0x7E,0x7E,0x6E,0x66,0x66,0x66,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'O' => [0x00,0x3C,0x66,0x66,0x66,0x66,0x66,0x66,0x3C,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'P' => [0x00,0x7C,0x66,0x66,0x7C,0x60,0x60,0x60,0x60,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'Q' => [0x00,0x3C,0x66,0x66,0x66,0x66,0x6E,0x3C,0x0E,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'R' => [0x00,0x7C,0x66,0x66,0x7C,0x78,0x6C,0x66,0x66,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'S' => [0x00,0x3C,0x66,0x60,0x3C,0x06,0x06,0x66,0x3C,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'T' => [0x00,0x7E,0x18,0x18,0x18,0x18,0x18,0x18,0x18,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'U' => [0x00,0x66,0x66,0x66,0x66,0x66,0x66,0x66,0x3C,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'V' => [0x00,0x66,0x66,0x66,0x66,0x66,0x3C,0x3C,0x18,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'W' => [0x00,0x63,0x63,0x63,0x6B,0x7F,0x77,0x63,0x63,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'X' => [0x00,0x66,0x66,0x3C,0x18,0x3C,0x66,0x66,0x66,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'Y' => [0x00,0x66,0x66,0x66,0x3C,0x18,0x18,0x18,0x18,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'Z' => [0x00,0x7E,0x06,0x0C,0x18,0x30,0x60,0x60,0x7E,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'a' => [0x00,0x00,0x00,0x3C,0x06,0x3E,0x66,0x66,0x3E,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'b' => [0x00,0x60,0x60,0x7C,0x66,0x66,0x66,0x66,0x7C,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'c' => [0x00,0x00,0x00,0x3C,0x66,0x60,0x60,0x66,0x3C,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'd' => [0x00,0x06,0x06,0x3E,0x66,0x66,0x66,0x66,0x3E,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'e' => [0x00,0x00,0x00,0x3C,0x66,0x7E,0x60,0x66,0x3C,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'f' => [0x00,0x1C,0x36,0x30,0x7C,0x30,0x30,0x30,0x30,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'g' => [0x00,0x00,0x00,0x3E,0x66,0x66,0x3E,0x06,0x3C,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'h' => [0x00,0x60,0x60,0x7C,0x66,0x66,0x66,0x66,0x66,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'i' => [0x00,0x18,0x00,0x38,0x18,0x18,0x18,0x18,0x3C,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'j' => [0x00,0x0C,0x00,0x1C,0x0C,0x0C,0x0C,0x6C,0x38,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'k' => [0x00,0x60,0x60,0x66,0x6C,0x78,0x6C,0x66,0x66,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'l' => [0x00,0x38,0x18,0x18,0x18,0x18,0x18,0x18,0x3C,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'm' => [0x00,0x00,0x00,0x76,0x7F,0x6B,0x6B,0x63,0x63,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'n' => [0x00,0x00,0x00,0x7C,0x66,0x66,0x66,0x66,0x66,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'o' => [0x00,0x00,0x00,0x3C,0x66,0x66,0x66,0x66,0x3C,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'p' => [0x00,0x00,0x00,0x7C,0x66,0x66,0x7C,0x60,0x60,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'q' => [0x00,0x00,0x00,0x3E,0x66,0x66,0x3E,0x06,0x06,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'r' => [0x00,0x00,0x00,0x7C,0x66,0x60,0x60,0x60,0x60,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        's' => [0x00,0x00,0x00,0x3E,0x60,0x3C,0x06,0x06,0x7C,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        't' => [0x00,0x30,0x30,0x7C,0x30,0x30,0x30,0x36,0x1C,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'u' => [0x00,0x00,0x00,0x66,0x66,0x66,0x66,0x66,0x3E,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'v' => [0x00,0x00,0x00,0x66,0x66,0x66,0x3C,0x3C,0x18,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'w' => [0x00,0x00,0x00,0x63,0x63,0x6B,0x7F,0x77,0x63,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'x' => [0x00,0x00,0x00,0x66,0x3C,0x18,0x3C,0x66,0x66,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'y' => [0x00,0x00,0x00,0x66,0x66,0x3E,0x06,0x66,0x3C,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'z' => [0x00,0x00,0x00,0x7E,0x0C,0x18,0x30,0x60,0x7E,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        '0' => [0x00,0x3C,0x66,0x6E,0x76,0x66,0x66,0x66,0x3C,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        '1' => [0x00,0x18,0x38,0x18,0x18,0x18,0x18,0x18,0x7E,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        '2' => [0x00,0x3C,0x66,0x06,0x0C,0x18,0x30,0x60,0x7E,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        '3' => [0x00,0x3C,0x66,0x06,0x1C,0x06,0x06,0x66,0x3C,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        '4' => [0x00,0x0C,0x1C,0x3C,0x6C,0x7E,0x0C,0x0C,0x0C,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        '5' => [0x00,0x7E,0x60,0x7C,0x06,0x06,0x06,0x66,0x3C,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        '6' => [0x00,0x3C,0x66,0x60,0x7C,0x66,0x66,0x66,0x3C,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        '7' => [0x00,0x7E,0x06,0x0C,0x18,0x18,0x18,0x18,0x18,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        '8' => [0x00,0x3C,0x66,0x66,0x3C,0x66,0x66,0x66,0x3C,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        '9' => [0x00,0x3C,0x66,0x66,0x3E,0x06,0x06,0x66,0x3C,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        '-' => [0x00,0x00,0x00,0x00,0x7E,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        ' ' => [0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        _ => [0x00,0x3C,0x42,0x42,0x42,0x42,0x42,0x3C,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00], // Box for unknown
    }
}

/// Global desktop
static DESKTOP: Mutex<Desktop> = Mutex::new(Desktop::new());

/// Initialize GUI with double buffering
pub fn init(width: u32, height: u32) {
    DESKTOP.lock().init(width, height);
    crate::serial_println!("[GUI] Desktop initialized: {}x{} (double-buffered)", width, height);
}

/// Create a window (legacy, empty type)
pub fn create_window(title: &str, x: i32, y: i32, width: u32, height: u32) -> u32 {
    DESKTOP.lock().create_window(title, x, y, width, height, WindowType::Empty)
}

/// Create a terminal window
pub fn create_terminal(x: i32, y: i32) -> u32 {
    DESKTOP.lock().create_window("Terminal", x, y, 500, 350, WindowType::Terminal)
}

/// Create a system info window
pub fn create_sysinfo(x: i32, y: i32) -> u32 {
    DESKTOP.lock().create_window("System Info", x, y, 300, 220, WindowType::SystemInfo)
}

/// Close a window
pub fn close_window(id: u32) {
    DESKTOP.lock().close_window(id);
}

/// Update cursor position
pub fn update_cursor(x: i32, y: i32) {
    DESKTOP.lock().handle_move(x, y);
}

/// Handle click
pub fn handle_click(x: i32, y: i32, pressed: bool) {
    DESKTOP.lock().handle_click(x, y, pressed);
}

/// Draw everything
pub fn draw() {
    DESKTOP.lock().draw();
}

/// Handle keyboard input for focused window
pub fn handle_keyboard(key: u8) {
    DESKTOP.lock().handle_keyboard_input(key);
}

/// Handle right-click
pub fn handle_right_click(x: i32, y: i32, pressed: bool) {
    DESKTOP.lock().handle_right_click(x, y, pressed);
}

/// Handle scroll wheel
pub fn handle_scroll(delta: i8) {
    DESKTOP.lock().handle_scroll(delta);
}

/// Run GUI loop with double-buffered rendering and vsync-like timing
pub fn run() -> ! {
    use crate::gui::engine::{self, HotkeyAction};
    
    // Initialize GUI timing
    engine::init_timing();
    
    crate::serial_println!("[GUI] Starting desktop environment...");
    crate::serial_println!("[GUI] Hotkeys: Alt+Tab, Win+Arrows, Alt+F4, Win=Start");
    crate::serial_println!("[GUI] Target: 60 FPS with HLT-based frame limiting");
    
    loop {
        let frame_start = engine::now_us();
        
        // ═══════════════════════════════════════════════════════════════
        // Input Processing
        // ═══════════════════════════════════════════════════════════════
        
        // Process mouse
        let mouse = crate::mouse::get_state();
        update_cursor(mouse.x, mouse.y);
        
        // Handle keyboard with hotkey detection
        while let Some(key) = crate::keyboard::read_char() {
            // Check for hotkeys first
            let scancode = key; // Assuming key is scancode
            let is_release = scancode & 0x80 != 0;
            let raw_code = scancode & 0x7F;
            
            // Update modifier state
            engine::update_modifiers(raw_code, !is_release);
            
            // Check for Start Menu (Win key alone)
            if engine::check_start_menu_trigger(raw_code, !is_release) {
                engine::toggle_start_menu();
            }
            
            // Check hotkeys on key press
            if !is_release {
                match engine::check_hotkey(raw_code) {
                    HotkeyAction::CloseWindow => {
                        // Alt+F4: Close focused window
                        DESKTOP.lock().close_focused_window();
                    }
                    HotkeyAction::SwitchWindow => {
                        // Alt+Tab: Toggle window switcher
                        if !engine::is_alt_tab_active() {
                            engine::start_alt_tab();
                        } else {
                            engine::alt_tab_next();
                        }
                    }
                    HotkeyAction::SnapLeft => {
                        DESKTOP.lock().snap_focused_window(SnapDir::Left);
                    }
                    HotkeyAction::SnapRight => {
                        DESKTOP.lock().snap_focused_window(SnapDir::Right);
                    }
                    HotkeyAction::Maximize => {
                        DESKTOP.lock().toggle_maximize_focused();
                    }
                    HotkeyAction::Minimize => {
                        DESKTOP.lock().minimize_focused_window();
                    }
                    HotkeyAction::ShowDesktop => {
                        DESKTOP.lock().toggle_show_desktop();
                    }
                    HotkeyAction::OpenFileManager => {
                        // Will open file manager when implemented
                        engine::show_toast("Files", "File manager coming soon", engine::NotifyPriority::Info);
                    }
                    HotkeyAction::OpenTerminal => {
                        DESKTOP.lock().open_terminal();
                    }
                    HotkeyAction::None => {
                        // Pass to focused window
                        handle_keyboard(key);
                    }
                    _ => {}
                }
            }
        }
        
        // Handle Alt release to finish Alt+Tab
        if engine::is_alt_tab_active() {
            // Check if Alt is still held - if released, select the window
            if !crate::keyboard::is_key_pressed(0x38) {
                let selected = engine::finish_alt_tab();
                DESKTOP.lock().focus_window_by_index(selected as usize);
            }
        }
        
        // Handle left mouse button
        static mut LAST_LEFT: bool = false;
        let left = mouse.left_button;
        unsafe {
            if left != LAST_LEFT {
                // Close start menu on click outside
                if left && engine::is_start_menu_open() {
                    // Check if click is outside start menu
                    // (Simple: close on any click for now)
                    engine::close_start_menu();
                }
                handle_click(mouse.x, mouse.y, left);
                LAST_LEFT = left;
            }
        }
        
        // Handle right mouse button
        static mut LAST_RIGHT: bool = false;
        let right = mouse.right_button;
        unsafe {
            if right != LAST_RIGHT {
                handle_right_click(mouse.x, mouse.y, right);
                LAST_RIGHT = right;
            }
        }
        
        // Handle scroll wheel
        let scroll = crate::mouse::get_scroll_delta();
        if scroll != 0 {
            handle_scroll(scroll);
        }
        
        // ═══════════════════════════════════════════════════════════════
        // Rendering
        // ═══════════════════════════════════════════════════════════════
        
        // Render main desktop
        draw();
        
        // Render Alt+Tab overlay if active
        if engine::is_alt_tab_active() {
            render_alt_tab_overlay();
        }
        
        // Render Start Menu if open
        if engine::is_start_menu_open() {
            render_start_menu();
        }
        
        // Render notifications
        render_notifications();
        
        // Render FPS counter (debug)
        #[cfg(debug_assertions)]
        {
            let fps = engine::get_fps();
            // Draw FPS in corner (optional)
        }
        
        // ═══════════════════════════════════════════════════════════════
        // Frame Limiting with HLT (saves CPU!)
        // ═══════════════════════════════════════════════════════════════
        engine::wait_for_next_frame(frame_start);
    }
}

/// Snap direction for window snapping
#[derive(Clone, Copy)]
pub enum SnapDir {
    Left,
    Right,
}

/// Render Alt+Tab overlay
fn render_alt_tab_overlay() {
    // Get window list
    let desktop = DESKTOP.lock();
    let windows = desktop.get_window_titles();
    if windows.is_empty() { return; }
    
    let screen_w = desktop.width;
    let screen_h = desktop.height;
    drop(desktop);
    
    let selection = crate::gui::engine::alt_tab_selection();
    let count = windows.len() as i32;
    let idx = ((selection % count) + count) % count; // Wrap around
    
    // Calculate overlay size
    let item_w: u32 = 120;
    let item_h: u32 = 80;
    let padding: u32 = 10;
    let total_w = windows.len() as u32 * (item_w + padding) + padding;
    let total_h = item_h + padding * 2 + 20;
    
    let x = (screen_w as i32 - total_w as i32) / 2;
    let y = (screen_h as i32 - total_h as i32) / 2;
    
    // Draw background
    draw_rounded_rect(x, y, total_w, total_h, 8, 0xE0101520);
    draw_rounded_rect(x + 1, y + 1, total_w - 2, total_h - 2, 7, 0xE0202530);
    
    // Draw window previews
    for (i, title) in windows.iter().enumerate() {
        let ix = x + padding as i32 + i as i32 * (item_w + padding) as i32;
        let iy = y + padding as i32;
        
        // Highlight selected
        let bg = if i as i32 == idx { 0xFF00CC55 } else { 0xFF303540 };
        draw_rounded_rect(ix, iy, item_w, item_h, 4, bg);
        
        // Window title (truncated)
        let short: String = title.chars().take(12).collect();
        draw_text_centered(ix + item_w as i32 / 2, iy + item_h as i32 + 5, &short, 0xFFFFFFFF);
    }
}

/// Render Start Menu
fn render_start_menu() {
    use crate::gui::engine::{get_start_menu_items, StartAction};
    
    let desktop = DESKTOP.lock();
    let screen_h = desktop.height;
    drop(desktop);
    
    let menu_w: u32 = 280;
    let menu_h: u32 = 350;
    let x: i32 = 10;
    let y: i32 = screen_h as i32 - TASKBAR_HEIGHT as i32 - menu_h as i32 - 5;
    
    // Background with blur effect simulation
    draw_rounded_rect(x, y, menu_w, menu_h, 12, 0xF0101520);
    draw_rounded_rect(x + 1, y + 1, menu_w - 2, menu_h - 2, 11, 0xF0181C25);
    
    // TrustOS logo/header
    draw_text(x + 20, y + 15, "TrustOS", 0xFF00FF66);
    draw_text(x + 90, y + 15, "v0.1", 0xFF606060);
    
    // Separator
    draw_line(x + 15, y + 35, x + menu_w as i32 - 15, y + 35, 0xFF303540);
    
    // Menu items
    let items = get_start_menu_items();
    let mut iy = y + 45;
    
    for item in items.iter() {
        if item.icon == 255 {
            // Separator
            draw_line(x + 15, iy + 5, x + menu_w as i32 - 15, iy + 5, 0xFF303540);
            iy += 12;
        } else {
            // Menu item
            draw_text(x + 40, iy, &item.name, 0xFFCCCCCC);
            iy += 28;
        }
    }
    
    // Search bar at bottom
    let search_y = y + menu_h as i32 - 45;
    draw_rounded_rect(x + 15, search_y, menu_w - 30, 30, 6, 0xFF252A35);
    draw_text(x + 25, search_y + 7, "Search apps...", 0xFF606060);
}

/// Render toast notifications
fn render_notifications() {
    use crate::gui::engine::{get_notifications, NotifyPriority};
    
    let desktop = DESKTOP.lock();
    let screen_w = desktop.width;
    drop(desktop);
    
    let notifs = get_notifications();
    let mut y = 60; // Below taskbar
    
    for toast in notifs.iter() {
        let w: u32 = 300;
        let h: u32 = if toast.progress.is_some() { 70 } else { 60 };
        let x = screen_w as i32 - w as i32 - 15;
        
        // Background
        draw_rounded_rect(x, y, w, h, 8, 0xF0181C25);
        
        // Accent bar
        let accent_color = toast.get_color();
        draw_rect(x, y, 4, h, accent_color);
        
        // Title
        draw_text(x + 15, y + 10, &toast.title, accent_color);
        
        // Message
        draw_text(x + 15, y + 30, &toast.message, 0xFFAAAAAA);
        
        // Progress bar if present
        if let Some(percent) = toast.progress {
            let bar_y = y + 50;
            let bar_w = w - 30;
            draw_rounded_rect(x + 15, bar_y, bar_w, 8, 3, 0xFF303540);
            let fill_w = (bar_w as u32 * percent as u32 / 100) as u32;
            if fill_w > 0 {
                draw_rounded_rect(x + 15, bar_y, fill_w.max(6), 8, 3, 0xFF00CC55);
            }
        }
        
        y += h as i32 + 10;
    }
}

/// Helper: Draw text (wrapper)
fn draw_text(x: i32, y: i32, text: &str, color: u32) {
    // Use framebuffer's text rendering
    for (i, c) in text.chars().enumerate() {
        let cx = x + (i * 8) as i32;
        if cx >= 0 {
            crate::framebuffer::draw_char_at(cx as u32, y as u32, c, color);
        }
    }
}

/// Helper: Draw centered text
fn draw_text_centered(cx: i32, y: i32, text: &str, color: u32) {
    let w = text.len() as i32 * 8;
    draw_text(cx - w / 2, y, text, color);
}

/// Helper: Draw line
fn draw_line(x1: i32, y1: i32, x2: i32, y2: i32, color: u32) {
    // Simple horizontal/vertical line
    if y1 == y2 {
        let (x_start, x_end) = if x1 < x2 { (x1, x2) } else { (x2, x1) };
        for x in x_start..=x_end {
            if x >= 0 {
                crate::framebuffer::draw_pixel(x as u32, y1 as u32, color);
            }
        }
    }
}

/// Helper: Draw rect
fn draw_rect(x: i32, y: i32, w: u32, h: u32, color: u32) {
    for dy in 0..h {
        for dx in 0..w {
            crate::framebuffer::draw_pixel((x + dx as i32) as u32, (y + dy as i32) as u32, color);
        }
    }
}

/// Helper: Draw rounded rect
fn draw_rounded_rect(x: i32, y: i32, w: u32, h: u32, radius: u32, color: u32) {
    // Simple implementation - just draw rect for now
    // Alpha blending for transparency
    let alpha = ((color >> 24) & 0xFF) as u32;
    if alpha < 255 {
        // With alpha - blend each pixel
        for dy in 0..h {
            for dx in 0..w {
                let px = x + dx as i32;
                let py = y + dy as i32;
                if px >= 0 && py >= 0 {
                    // Simple alpha blend
                    crate::framebuffer::draw_pixel(px as u32, py as u32, color | 0xFF000000);
                }
            }
        }
    } else {
        draw_rect(x, y, w, h, color);
    }
}

/// Read CPU timestamp counter for timing
#[inline]
fn read_tsc() -> u64 {
    unsafe {
        core::arch::x86_64::_rdtsc()
    }
}

/// Set the render mode (Classic or OpenGL)
pub fn set_render_mode(mode: RenderMode) {
    DESKTOP.lock().set_render_mode(mode);
    let mode_name = match mode {
        RenderMode::Classic => "Classic",
        RenderMode::OpenGL => "OpenGL Compositor",
    };
    crate::serial_println!("[GUI] Render mode: {}", mode_name);
}

/// Set the compositor theme
pub fn set_theme(theme: CompositorTheme) {
    DESKTOP.lock().set_theme(theme);
    let theme_name = match theme {
        CompositorTheme::Flat => "Flat",
        CompositorTheme::Modern => "Modern",
        CompositorTheme::Glass => "Glass",
        CompositorTheme::Neon => "Neon",
        CompositorTheme::Minimal => "Minimal",
    };
    crate::serial_println!("[GUI] Compositor theme: {}", theme_name);
}

/// Get current render mode
pub fn get_render_mode() -> RenderMode {
    DESKTOP.lock().render_mode
}
