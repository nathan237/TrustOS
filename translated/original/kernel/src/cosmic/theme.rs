//! COSMIC Theme - Pop!_OS / System76 color palette
//!
//! Ported from libcosmic's cosmic-theme crate.
//! https://github.com/pop-os/libcosmic/tree/master/cosmic-theme

use super::Color;

// ═══════════════════════════════════════════════════════════════════════════════
// TRUSTOS MATRIX MODERNE - Official Palette
// ═══════════════════════════════════════════════════════════════════════════════
// Un OS sécurisé ne crie pas. Tout inspire: contrôle, sécurité, calme, maîtrise.
// Règle: 80% vert doux, 15% vert normal, 5% accent
// ═══════════════════════════════════════════════════════════════════════════════

/// TrustOS Matrix Moderne Theme (Primary)
pub mod matrix {
    use super::Color;
    
    // ═══════════════════════════════════════════════════════════════════════════
    // PALETTE OFFICIELLE TRUSTOS
    // ═══════════════════════════════════════════════════════════════════════════
    
    // Background colors - Deep dark with subtle green undertone
    /// #050805 - Deepest background
    pub const BG_MAIN: Color = Color::rgb(0.020, 0.031, 0.020);
    /// #070f07 - Panel/sidebar background  
    pub const BG_PANEL: Color = Color::rgb(0.027, 0.059, 0.027);
    /// rgba(0,255,120,0.04) - Glass/translucent surfaces
    pub const BG_GLASS: Color = Color::new(0.0, 1.0, 0.47, 0.04);
    /// Surface for cards/windows
    pub const BG_SURFACE: Color = Color::new(0.0, 1.0, 0.47, 0.035);
    
    // Green hierarchy (stratified as per spec)
    /// #00ff88 - Soft green (80% usage) - Primary text & elements
    pub const GREEN_SOFT: Color = Color::rgb(0.0, 1.0, 0.533);
    /// #00ff66 - Main green (15% usage) - Important elements
    pub const GREEN_MAIN: Color = Color::rgb(0.0, 1.0, 0.4);
    /// #33ff99 - Accent green (5% usage) - Highlights only
    pub const GREEN_ACCENT: Color = Color::rgb(0.2, 1.0, 0.6);
    /// Muted green for secondary elements
    pub const GREEN_MUTED: Color = Color::rgb(0.0, 0.6, 0.32);
    /// Dim green for disabled/ghost elements
    pub const GREEN_DIM: Color = Color::rgb(0.0, 0.35, 0.18);
    
    // Borders & Glows
    /// rgba(0,255,120,0.22) - Subtle border
    pub const BORDER_SUBTLE: Color = Color::new(0.0, 1.0, 0.47, 0.22);
    /// rgba(0,255,120,0.35) - Focused border
    pub const BORDER_FOCUSED: Color = Color::new(0.0, 1.0, 0.47, 0.35);
    /// rgba(0,255,120,0.12) - Soft glow
    pub const GLOW_SOFT: Color = Color::new(0.0, 1.0, 0.47, 0.12);
    /// rgba(0,255,120,0.28) - Strong glow (hover)
    pub const GLOW_STRONG: Color = Color::new(0.0, 1.0, 0.47, 0.28);
    
    // ═══════════════════════════════════════════════════════════════════════════
    // SEMANTIC ALIASES (for backward compatibility)
    // ═══════════════════════════════════════════════════════════════════════════
    
    // Background
    pub const BG_BASE: Color = BG_MAIN;
    pub const BG_COMPONENT: Color = BG_PANEL;
    pub const BG_CONTAINER: Color = Color::rgb(0.035, 0.075, 0.035);
    pub const BG_DIVIDER: Color = Color::new(0.0, 1.0, 0.47, 0.15);
    
    // Surface
    pub const SURFACE: Color = BG_SURFACE;
    pub const SURFACE_HOVER: Color = Color::new(0.0, 1.0, 0.47, 0.08);
    pub const SURFACE_PRESSED: Color = Color::new(0.0, 1.0, 0.47, 0.12);
    
    // Accent
    pub const ACCENT: Color = GREEN_SOFT;
    pub const ACCENT_HOVER: Color = GREEN_MAIN;
    pub const ACCENT_PRESSED: Color = GREEN_MUTED;
    
    // Matrix legacy
    pub const MATRIX_GREEN: Color = GREEN_MAIN;
    pub const MATRIX_DARK_GREEN: Color = GREEN_MUTED;
    pub const MATRIX_BRIGHT: Color = GREEN_ACCENT;
    pub const MATRIX_DIM: Color = GREEN_DIM;
    
    // Text - Green hierarchy
    pub const TEXT_PRIMARY: Color = GREEN_SOFT;      // 80% - Main text
    pub const TEXT_SECONDARY: Color = GREEN_MUTED;   // Subdued text
    pub const TEXT_DISABLED: Color = GREEN_DIM;      // Disabled
    pub const TEXT_ACCENT: Color = GREEN_ACCENT;     // 5% - Highlights
    
    // Buttons - Subtle glass effect
    pub const BUTTON_BG: Color = Color::new(0.0, 1.0, 0.47, 0.06);
    pub const BUTTON_HOVER: Color = Color::new(0.0, 1.0, 0.47, 0.12);
    pub const BUTTON_PRESSED: Color = Color::new(0.0, 1.0, 0.47, 0.18);
    pub const BUTTON_SUGGESTED: Color = GREEN_SOFT;
    pub const BUTTON_DESTRUCTIVE: Color = Color::rgb(0.6, 0.2, 0.2); // Muted red, not aggressive
    
    // Window - Glassmorphism style
    /// Header gradient start
    pub const HEADER_BG: Color = Color::new(0.0, 1.0, 0.47, 0.06);
    /// Header gradient end  
    pub const HEADER_BG_END: Color = Color::new(0.0, 1.0, 0.47, 0.01);
    pub const HEADER_DIVIDER: Color = BORDER_SUBTLE;
    
    // Window controls - Desaturated, subtle
    pub const CLOSE_BG: Color = Color::rgb(0.45, 0.25, 0.25);      // Muted red
    pub const CLOSE_HOVER: Color = Color::rgb(0.55, 0.30, 0.30);
    pub const MAXIMIZE_BG: Color = Color::rgb(0.25, 0.40, 0.30);   // Muted green
    pub const MAXIMIZE_HOVER: Color = Color::rgb(0.30, 0.50, 0.35);
    pub const MINIMIZE_BG: Color = Color::rgb(0.35, 0.35, 0.25);   // Muted amber
    pub const MINIMIZE_HOVER: Color = Color::rgb(0.45, 0.45, 0.30);
    
    // Panel/dock - Glassmorphism
    /// Panel background with blur
    pub const PANEL_BG: Color = Color::new(0.0, 1.0, 0.47, 0.05);
    pub const PANEL_HOVER: Color = Color::new(0.0, 1.0, 0.47, 0.12);
    pub const PANEL_ACTIVE: Color = Color::new(0.0, 1.0, 0.47, 0.18);
    
    // Borders
    pub const BORDER: Color = BORDER_SUBTLE;
    
    // Status colors - Muted, professional
    pub const SUCCESS: Color = Color::rgb(0.2, 0.65, 0.4);   // Soft green
    pub const WARNING: Color = Color::rgb(0.7, 0.6, 0.25);   // Soft amber
    pub const ERROR: Color = Color::rgb(0.6, 0.3, 0.3);      // Soft red
    pub const INFO: Color = GREEN_SOFT;
    
    // ═══════════════════════════════════════════════════════════════════════════
    // UI CONSTANTS
    // ═══════════════════════════════════════════════════════════════════════════
    
    /// Window border radius
    pub const WINDOW_RADIUS: f32 = 14.0;
    /// Button border radius
    pub const BUTTON_RADIUS: f32 = 8.0;
    /// Panel/dock radius
    pub const PANEL_RADIUS: f32 = 28.0;
    /// Icon container radius
    pub const ICON_RADIUS: f32 = 10.0;
    
    /// Title bar height
    pub const HEADER_HEIGHT: f32 = 28.0;
    /// Dock height
    pub const DOCK_HEIGHT: f32 = 56.0;
    /// Dock icon size
    pub const DOCK_ICON_SIZE: f32 = 44.0;
    
    /// Animation duration (ms) - subtle, not distracting
    pub const ANIM_FAST: u32 = 120;
    pub const ANIM_NORMAL: u32 = 180;
    
    /// Blur radius for glassmorphism
    pub const BLUR_RADIUS: f32 = 18.0;
    
    /// Hover scale factor
    pub const HOVER_SCALE: f32 = 1.04;
    /// Dock hover scale
    pub const DOCK_HOVER_SCALE: f32 = 1.10;
}

// ═══════════════════════════════════════════════════════════════════════════════
// COSMIC PALETTE - Official Pop!_OS colors (backup)
// ═══════════════════════════════════════════════════════════════════════════════

/// COSMIC Dark Theme (fallback)
pub mod dark {
    use super::Color;
    
    // Background colors - Using matrix theme as base
    pub const BG_BASE: Color = super::matrix::BG_MAIN;
    pub const BG_COMPONENT: Color = super::matrix::BG_PANEL;
    pub const BG_CONTAINER: Color = super::matrix::BG_CONTAINER;
    pub const BG_DIVIDER: Color = super::matrix::BG_DIVIDER;
    
    // Surface colors
    pub const SURFACE: Color = super::matrix::SURFACE;
    pub const SURFACE_HOVER: Color = super::matrix::SURFACE_HOVER;
    pub const SURFACE_PRESSED: Color = super::matrix::SURFACE_PRESSED;
    
    // Accent
    pub const ACCENT: Color = super::matrix::ACCENT;
    pub const ACCENT_HOVER: Color = super::matrix::ACCENT_HOVER;
    pub const ACCENT_PRESSED: Color = super::matrix::ACCENT_PRESSED;
    
    // Text
    pub const TEXT_PRIMARY: Color = super::matrix::TEXT_PRIMARY;
    pub const TEXT_SECONDARY: Color = super::matrix::TEXT_SECONDARY;
    pub const TEXT_DISABLED: Color = super::matrix::TEXT_DISABLED;
    
    // Buttons
    pub const BUTTON_BG: Color = super::matrix::BUTTON_BG;
    pub const BUTTON_HOVER: Color = super::matrix::BUTTON_HOVER;
    pub const BUTTON_PRESSED: Color = super::matrix::BUTTON_PRESSED;
    pub const BUTTON_SUGGESTED: Color = super::matrix::BUTTON_SUGGESTED;
    pub const BUTTON_DESTRUCTIVE: Color = super::matrix::BUTTON_DESTRUCTIVE;
    
    // Window
    pub const HEADER_BG: Color = super::matrix::HEADER_BG;
    pub const HEADER_DIVIDER: Color = super::matrix::HEADER_DIVIDER;
    pub const CLOSE_BG: Color = super::matrix::CLOSE_BG;
    pub const MAXIMIZE_BG: Color = super::matrix::MAXIMIZE_BG;
    pub const MINIMIZE_BG: Color = super::matrix::MINIMIZE_BG;
    
    // Panel
    pub const PANEL_BG: Color = super::matrix::PANEL_BG;
    pub const PANEL_HOVER: Color = super::matrix::PANEL_HOVER;
    
    // Borders
    pub const BORDER: Color = super::matrix::BORDER;
    pub const BORDER_FOCUSED: Color = super::matrix::BORDER_FOCUSED;
    
    // Status
    pub const SUCCESS: Color = super::matrix::SUCCESS;
    pub const WARNING: Color = super::matrix::WARNING;
    pub const ERROR: Color = super::matrix::ERROR;
    pub const INFO: Color = super::matrix::INFO;
}

/// COSMIC Light Theme (minimal)
pub mod light {
    use super::Color;
    
    pub const BG_BASE: Color = Color::rgb(0.976, 0.976, 0.976);
    pub const BG_COMPONENT: Color = Color::rgb(0.949, 0.949, 0.949);
    pub const SURFACE: Color = Color::rgb(1.0, 1.0, 1.0);
    pub const ACCENT: Color = Color::rgb(0.0, 0.7, 0.45);
    pub const TEXT_PRIMARY: Color = Color::rgb(0.1, 0.15, 0.1);
    pub const TEXT_SECONDARY: Color = Color::rgb(0.3, 0.35, 0.3);
}

// ═══════════════════════════════════════════════════════════════════════════════
// THEME STRUCT
// ═══════════════════════════════════════════════════════════════════════════════

/// Complete theme definition
#[derive(Clone, Copy)]
pub struct CosmicTheme {
    // Backgrounds
    pub bg_base: Color,
    pub bg_component: Color,
    pub bg_container: Color,
    pub bg_divider: Color,
    
    // Surfaces
    pub surface: Color,
    pub surface_hover: Color,
    pub surface_pressed: Color,
    
    // Accent
    pub accent: Color,
    pub accent_hover: Color,
    pub accent_pressed: Color,
    
    // Text
    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_disabled: Color,
    
    // Buttons
    pub button_bg: Color,
    pub button_hover: Color,
    pub button_pressed: Color,
    pub button_suggested: Color,
    pub button_destructive: Color,
    
    // Window
    pub header_bg: Color,
    pub close_bg: Color,
    pub maximize_bg: Color,
    pub minimize_bg: Color,
    
    // Panel
    pub panel_bg: Color,
    pub panel_hover: Color,
    
    // Misc
    pub border: Color,
    pub border_focused: Color,
    pub corner_radius: f32,
    pub spacing: f32,
    pub padding: f32,
}

impl CosmicTheme {
    /// Create dark theme (default)
    pub const fn dark() -> Self {
        Self {
            bg_base: dark::BG_BASE,
            bg_component: dark::BG_COMPONENT,
            bg_container: dark::BG_CONTAINER,
            bg_divider: dark::BG_DIVIDER,
            surface: dark::SURFACE,
            surface_hover: dark::SURFACE_HOVER,
            surface_pressed: dark::SURFACE_PRESSED,
            accent: dark::ACCENT,
            accent_hover: dark::ACCENT_HOVER,
            accent_pressed: dark::ACCENT_PRESSED,
            text_primary: dark::TEXT_PRIMARY,
            text_secondary: dark::TEXT_SECONDARY,
            text_disabled: dark::TEXT_DISABLED,
            button_bg: dark::BUTTON_BG,
            button_hover: dark::BUTTON_HOVER,
            button_pressed: dark::BUTTON_PRESSED,
            button_suggested: dark::BUTTON_SUGGESTED,
            button_destructive: dark::BUTTON_DESTRUCTIVE,
            header_bg: dark::HEADER_BG,
            close_bg: dark::CLOSE_BG,
            maximize_bg: dark::MAXIMIZE_BG,
            minimize_bg: dark::MINIMIZE_BG,
            panel_bg: dark::PANEL_BG,
            panel_hover: dark::PANEL_HOVER,
            border: dark::BORDER,
            border_focused: dark::BORDER_FOCUSED,
            corner_radius: 8.0,
            spacing: 8.0,
            padding: 12.0,
        }
    }
    
    /// Create light theme
    pub const fn light() -> Self {
        Self {
            bg_base: light::BG_BASE,
            bg_component: light::BG_COMPONENT,
            bg_container: light::BG_COMPONENT,
            bg_divider: light::BG_COMPONENT,
            surface: light::SURFACE,
            surface_hover: light::BG_COMPONENT,
            surface_pressed: light::BG_BASE,
            accent: light::ACCENT,
            accent_hover: light::ACCENT,
            accent_pressed: light::ACCENT,
            text_primary: light::TEXT_PRIMARY,
            text_secondary: light::TEXT_SECONDARY,
            text_disabled: light::TEXT_SECONDARY,
            button_bg: light::BG_COMPONENT,
            button_hover: light::BG_BASE,
            button_pressed: light::BG_COMPONENT,
            button_suggested: light::ACCENT,
            button_destructive: dark::BUTTON_DESTRUCTIVE,
            header_bg: light::SURFACE,
            close_bg: dark::CLOSE_BG,
            maximize_bg: dark::MAXIMIZE_BG,
            minimize_bg: dark::MINIMIZE_BG,
            panel_bg: light::BG_BASE,
            panel_hover: light::BG_COMPONENT,
            border: light::BG_COMPONENT,
            border_focused: light::ACCENT,
            corner_radius: 8.0,
            spacing: 8.0,
            padding: 12.0,
        }
    }
    
    /// Create Matrix theme - Cyberpunk green on black
    pub const fn matrix() -> Self {
        Self {
            bg_base: matrix::BG_BASE,
            bg_component: matrix::BG_COMPONENT,
            bg_container: matrix::BG_CONTAINER,
            bg_divider: matrix::BG_DIVIDER,
            surface: matrix::SURFACE,
            surface_hover: matrix::SURFACE_HOVER,
            surface_pressed: matrix::SURFACE_PRESSED,
            accent: matrix::ACCENT,
            accent_hover: matrix::ACCENT_HOVER,
            accent_pressed: matrix::ACCENT_PRESSED,
            text_primary: matrix::TEXT_PRIMARY,
            text_secondary: matrix::TEXT_SECONDARY,
            text_disabled: matrix::TEXT_DISABLED,
            button_bg: matrix::BUTTON_BG,
            button_hover: matrix::BUTTON_HOVER,
            button_pressed: matrix::BUTTON_PRESSED,
            button_suggested: matrix::BUTTON_SUGGESTED,
            button_destructive: matrix::BUTTON_DESTRUCTIVE,
            header_bg: matrix::HEADER_BG,
            close_bg: matrix::CLOSE_BG,
            maximize_bg: matrix::MAXIMIZE_BG,
            minimize_bg: matrix::MINIMIZE_BG,
            panel_bg: matrix::PANEL_BG,
            panel_hover: matrix::PANEL_HOVER,
            border: matrix::BORDER,
            border_focused: matrix::BORDER_FOCUSED,
            corner_radius: 4.0,  // Sharper corners for Matrix look
            spacing: 6.0,
            padding: 8.0,
        }
    }
}

impl Default for CosmicTheme {
    fn default() -> Self {
        Self::dark()
    }
}

/// Global theme instance
static mut CURRENT_THEME: CosmicTheme = CosmicTheme::dark();

/// Get current theme
pub fn theme() -> &'static CosmicTheme {
    unsafe { &CURRENT_THEME }
}

/// Set current theme
pub fn set_theme(t: CosmicTheme) {
    unsafe { CURRENT_THEME = t; }
}
