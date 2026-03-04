// ═══════════════════════════════════════════════════════════════════════════
// TrustOS Mobile Desktop — Same Style, Mobile Layout
// ═══════════════════════════════════════════════════════════════════════════
//
// Keeps the EXACT same visual style as the TrustOS desktop (matrix rain
// background, chrome/silver borders, frosted glass panels, green accents)
// but reorganizes all UI elements for a mobile portrait form factor.
//
// Target: iPhone 16 native — 1179×2556 @ 460 PPI (19.5:9 aspect ratio)
// VBox test: rendered inside a centered portrait viewport on 1920×1080 screen
//
// Layout (portrait):
//   ┌──────────────────────────────────┐
//   │  STATUS BAR (44px) 12:34  🔋 📶 │
//   ├──────────────────────────────────┤
//   │                                  │
//   │     APP GRID (4 cols, scrollable)│
//   │  [icon] [icon] [icon] [icon]     │
//   │  [icon] [icon] [icon] [icon]     │
//   │  [icon] [icon] [icon] [icon]     │
//   │                                  │
//   ├──────────────────────────────────┤
//   │  DOCK (glass, 5 pinned apps)     │
//   │         ─── gesture bar ───      │
//   └──────────────────────────────────┘
//
// ═══════════════════════════════════════════════════════════════════════════

use alloc::string::String;
use alloc::vec::Vec;

use crate::framebuffer;

// ═══════════════════════════════════════════════════════════════
// Desktop-Identical Color Palette (reuse exact same values)
// ═══════════════════════════════════════════════════════════════

const BG_DEEPEST: u32 = 0xFF050606;
const BG_DARK: u32 = 0xFF070B09;
const _BG_MEDIUM: u32 = 0xFF0A0F0C;
const _BG_LIGHT: u32 = 0xFF0D1310;

const GREEN_PRIMARY: u32 = 0xFF00FF66;
const GREEN_SECONDARY: u32 = 0xFF00CC55;
const GREEN_TERTIARY: u32 = 0xFF00AA44;
const GREEN_MUTED: u32 = 0xFF008844;
const GREEN_SUBTLE: u32 = 0xFF006633;
const GREEN_GHOST: u32 = 0xFF003B1A;

const CHROME_BRIGHT: u32 = 0xFFB0B2B0;
const CHROME_MID: u32 = 0xFF8C8E8C;
const CHROME_DIM: u32 = 0xFF606260;
const CHROME_GHOST: u32 = 0xFF3A3C3A;

const _ACCENT_AMBER: u32 = 0xFFFFD166;
const ACCENT_RED: u32 = 0xFFFF5555;
const _ACCENT_BLUE: u32 = 0xFF4ECDC4;

const TEXT_PRIMARY: u32 = 0xFFE0E8E4;
const TEXT_SECONDARY: u32 = 0xFF8A9890;
const _TEXT_ACCENT: u32 = 0xFF00CC55;

// ═══════════════════════════════════════════════════════════════
// iPhone 16 Display Specs
// ═══════════════════════════════════════════════════════════════

/// iPhone 16 native resolution (portrait)
pub const IPHONE16_W: u32 = 1179;
pub const IPHONE16_H: u32 = 2556;
/// iPhone 16 logical points (portrait) at 3x scale
pub const IPHONE16_LOGICAL_W: u32 = 393;
pub const IPHONE16_LOGICAL_H: u32 = 852;
/// Aspect ratio: 19.5:9
pub const IPHONE16_ASPECT_NUM: u32 = 195;
pub const IPHONE16_ASPECT_DEN: u32 = 90;

/// VBox test viewport: fit iPhone aspect ratio onto 1920×1080
/// Height-limited: 1080px height → width = 1080 * 90/195 ≈ 498px
/// We clamp to nice round numbers for pixel alignment
pub const VBOX_VP_W: u32 = 498;
pub const VBOX_VP_H: u32 = 1080;

// ═══════════════════════════════════════════════════════════════
// Mobile Layout Constants (scaled for VBox viewport)
// ═══════════════════════════════════════════════════════════════

const STATUS_BAR_H: u32 = 44;
const DOCK_H: u32 = 90;
const GESTURE_BAR_H: u32 = 5;
const GESTURE_BAR_W: u32 = 134;

/// App grid
const GRID_COLS: u32 = 4;
const ICON_SIZE: u32 = 56;
const ICON_RADIUS: u32 = 14;
const ICON_LABEL_H: u32 = 16;
const ICON_CELL_H: u32 = ICON_SIZE + ICON_LABEL_H + 10;
const ICON_GAP_X: u32 = 6;

/// Dock
const DOCK_ICON_COUNT: usize = 5;
const DOCK_ICON_SIZE: u32 = 48;
const DOCK_RADIUS: u32 = 20;
const DOCK_MARGIN: u32 = 12;

/// App bar (fullscreen app top status)
pub const APP_BAR_H: u32 = 36;

/// Control center
const CC_RADIUS: u32 = 20;

/// App switcher cards
const SWITCHER_CARD_GAP: u32 = 16;
const SWITCHER_CARD_RADIUS: u32 = 14;

// ═══════════════════════════════════════════════════════════════
// App Definitions (same as desktop icons)
// ═══════════════════════════════════════════════════════════════

#[derive(Clone, Copy)]
pub struct MobileApp {
    pub name: &'static str,
    pub icon_idx: u8,
    pub accent: u32,
}

/// All home screen apps — same order/colors as desktop dock sidebar
const MOBILE_APPS: &[MobileApp] = &[
    MobileApp { name: "Terminal",  icon_idx: 0,  accent: 0xFF20CC60 },
    MobileApp { name: "Files",     icon_idx: 1,  accent: 0xFFDDAA30 },
    MobileApp { name: "Editor",    icon_idx: 2,  accent: 0xFF5090E0 },
    MobileApp { name: "Calc",      icon_idx: 3,  accent: 0xFFCC6633 },
    MobileApp { name: "Network",   icon_idx: 4,  accent: 0xFF40AADD },
    MobileApp { name: "Games",     icon_idx: 5,  accent: 0xFFCC4444 },
    MobileApp { name: "Browser",   icon_idx: 6,  accent: 0xFF4488DD },
    MobileApp { name: "TrustEd",   icon_idx: 7,  accent: 0xFF9060D0 },
    MobileApp { name: "Settings",  icon_idx: 8,  accent: 0xFF9988BB },
    MobileApp { name: "About",     icon_idx: 9,  accent: 0xFF40CC80 },
    MobileApp { name: "Music",     icon_idx: 10, accent: 0xFFFF6090 },
    MobileApp { name: "Chess",     icon_idx: 11, accent: 0xFFD4A854 },
];

/// Dock pinned apps: Terminal, Files, Browser, Music, Settings
const DOCK_APPS: [usize; DOCK_ICON_COUNT] = [0, 1, 6, 10, 8];

// ═══════════════════════════════════════════════════════════════
// State
// ═══════════════════════════════════════════════════════════════

#[derive(Clone, Copy, PartialEq)]
pub enum MobileView {
    Home,
    AppFullscreen,
    AppSwitcher,
    ControlCenter,
}

#[derive(Clone)]
pub struct MobileState {
    pub active: bool,
    pub view: MobileView,
    pub active_app_id: Option<u32>,
    pub home_scroll_x: i32,
    pub home_page: i32,
    pub switcher_scroll_x: i32,
    pub cc_progress: u8,
    pub notif_progress: u8,
    pub gesture_start_x: i32,
    pub gesture_start_y: i32,
    pub gesture_active: bool,
    pub gesture_from_bottom: bool,
    pub gesture_from_top: bool,
    pub anim_frame: u64,
    pub highlighted_icon: i32,
    pub search_text: String,
    pub closing_cards: Vec<(u32, u8)>,
    pub time_str: String,
    /// Viewport (set by Desktop based on screen size)
    pub vp_x: i32,
    pub vp_y: i32,
    pub vp_w: u32,
    pub vp_h: u32,
}

impl MobileState {
    pub const fn new() -> Self {
        Self {
            active: false,
            view: MobileView::Home,
            active_app_id: None,
            home_scroll_x: 0,
            home_page: 0,
            switcher_scroll_x: 0,
            cc_progress: 0,
            notif_progress: 0,
            gesture_start_x: 0,
            gesture_start_y: 0,
            gesture_active: false,
            gesture_from_bottom: false,
            gesture_from_top: false,
            anim_frame: 0,
            highlighted_icon: -1,
            search_text: String::new(),
            closing_cards: Vec::new(),
            time_str: String::new(),
            vp_x: 0,
            vp_y: 0,
            vp_w: VBOX_VP_W,
            vp_h: VBOX_VP_H,
        }
    }
}

/// Calculate mobile viewport: fit iPhone 16 aspect ratio centered on screen.
/// On 1920×1080: 498×1080 centered horizontally.
pub fn calculate_viewport(screen_w: u32, screen_h: u32) -> (i32, i32, u32, u32) {
    // Height-limited: use full screen height, calculate width from aspect
    let vp_h = screen_h;
    let vp_w = (screen_h * IPHONE16_ASPECT_DEN / IPHONE16_ASPECT_NUM).min(screen_w);
    let vx = ((screen_w.saturating_sub(vp_w)) / 2) as i32;
    let vy = 0i32;
    (vx, vy, vp_w, vp_h)
}

// ═══════════════════════════════════════════════════════════════
// Drawing Helpers (same style as desktop.rs)
// ═══════════════════════════════════════════════════════════════

fn draw_text(x: i32, y: i32, text: &str, color: u32) {
    crate::graphics::scaling::draw_scaled_text(x, y, text, color);
}

fn draw_text_centered(cx: i32, y: i32, text: &str, color: u32) {
    let w = crate::graphics::scaling::measure_text_width(text) as i32;
    draw_text(cx - w / 2, y, text, color);
}

fn char_width() -> u32 {
    crate::graphics::scaling::char_width()
}

/// Rounded filled rect — identical to desktop.rs implementation
fn draw_rounded_rect(x: i32, y: i32, w: u32, h: u32, radius: u32, color: u32) {
    if w == 0 || h == 0 { return; }
    let r = radius.min(w / 2).min(h / 2);
    if r == 0 {
        if x >= 0 && y >= 0 { framebuffer::fill_rect(x as u32, y as u32, w, h, color); }
        return;
    }
    let wi = w as i32;
    let hi = h as i32;
    let ri = r as i32;
    // Center body
    fill_rect_s(x, y + ri, wi, hi - ri * 2, color);
    fill_rect_s(x + ri, y, wi - ri * 2, ri, color);
    fill_rect_s(x + ri, y + hi - ri, wi - ri * 2, ri, color);
    // Corners
    let r2 = ri * ri;
    for dy in 0..ri {
        let dx = fast_isqrt(r2 - dy * dy);
        fill_rect_s(x + ri - dx, y + ri - dy - 1, dx, 1, color);
        fill_rect_s(x + wi - ri, y + ri - dy - 1, dx, 1, color);
        fill_rect_s(x + ri - dx, y + hi - ri + dy, dx, 1, color);
        fill_rect_s(x + wi - ri, y + hi - ri + dy, dx, 1, color);
    }
}

/// Rounded rect border — identical to desktop.rs
fn draw_rounded_rect_border(x: i32, y: i32, w: u32, h: u32, radius: u32, color: u32) {
    if w == 0 || h == 0 { return; }
    let r = radius.min(w / 2).min(h / 2);
    let wi = w as i32;
    let hi = h as i32;
    let ri = r as i32;
    if r == 0 {
        if x >= 0 && y >= 0 { framebuffer::draw_rect(x as u32, y as u32, w, h, color); }
        return;
    }
    // Straight edges
    for px in (x + ri)..(x + wi - ri) {
        put_px(px, y, color);
        put_px(px, y + hi - 1, color);
    }
    for py in (y + ri)..(y + hi - ri) {
        put_px(x, py, color);
        put_px(x + wi - 1, py, color);
    }
    // Quarter-circle corners
    let r2 = ri * ri;
    let mut last_x = ri;
    for dy in 0..=ri {
        let dx = fast_isqrt(r2 - dy * dy);
        // Draw arc pixels from last_x down to dx
        for ax in dx..=last_x {
            // TL
            put_px(x + ri - ax, y + ri - dy, color);
            // TR
            put_px(x + wi - 1 - ri + ax, y + ri - dy, color);
            // BL
            put_px(x + ri - ax, y + hi - 1 - ri + dy, color);
            // BR
            put_px(x + wi - 1 - ri + ax, y + hi - 1 - ri + dy, color);
        }
        last_x = dx;
    }
}

/// Signed fill_rect helper
fn fill_rect_s(x: i32, y: i32, w: i32, h: i32, color: u32) {
    if w <= 0 || h <= 0 || x + w <= 0 || y + h <= 0 { return; }
    let x0 = x.max(0) as u32;
    let y0 = y.max(0) as u32;
    let x1 = (x + w).max(0) as u32;
    let y1 = (y + h).max(0) as u32;
    if x1 > x0 && y1 > y0 {
        framebuffer::fill_rect(x0, y0, x1 - x0, y1 - y0, color);
    }
}

fn put_px(x: i32, y: i32, color: u32) {
    if x >= 0 && y >= 0 {
        framebuffer::put_pixel(x as u32, y as u32, color);
    }
}

fn fast_isqrt(v: i32) -> i32 {
    if v <= 0 { return 0; }
    let mut x = v;
    let mut y = (x + 1) / 2;
    while y < x { x = y; y = (x + v / x) / 2; }
    x
}

/// Approximate atan2 → octant index (0..7), integer-only for no_std
fn atan2_approx_octant(y: i32, x: i32) -> u32 {
    let ax = if x < 0 { -x } else { x };
    let ay = if y < 0 { -y } else { y };
    // Determine base octant
    let oct = if x >= 0 {
        if y >= 0 {
            if ax >= ay { 0 } else { 1 }
        } else {
            if ax >= ay { 7 } else { 6 }
        }
    } else {
        if y >= 0 {
            if ax >= ay { 3 } else { 2 }
        } else {
            if ax >= ay { 4 } else { 5 }
        }
    };
    oct
}

// ═══════════════════════════════════════════════════════════════
// Glass Panel — identical to desktop frosted glass style
// ═══════════════════════════════════════════════════════════════

/// Frosted glass panel (same as desktop title bars / start menu)
fn draw_glass_panel(x: i32, y: i32, w: u32, h: u32, _radius: u32, opacity: u32) {
    if w == 0 || h == 0 { return; }
    let xu = x.max(0) as u32;
    let yu = y.max(0) as u32;
    // Dark frosted base
    framebuffer::fill_rect_alpha(xu, yu, w, h, 0x000000, (opacity * 7 / 10).min(255));
    // Green tinted overlay
    framebuffer::fill_rect_alpha(xu, yu, w, h, 0x001A0A, (opacity * 2 / 10).min(255));
    // Glass shine gradient (top 25%)
    let shine_h = h / 4;
    for row in 0..shine_h {
        let alpha = ((shine_h - row) * 18 / shine_h).min(255);
        if alpha > 0 {
            framebuffer::fill_rect_alpha(xu, yu + row, w, 1, 0xFFFFFF, alpha);
        }
    }
    // Glass border
    draw_rounded_rect_border(x, y, w, h, _radius, CHROME_GHOST);
}

// ═══════════════════════════════════════════════════════════════
// Status Bar — top of mobile viewport
// ═══════════════════════════════════════════════════════════════

pub fn draw_status_bar(vx: i32, vy: i32, vw: u32, _vh: u32, time_str: &str, _frame: u64) {
    let x = vx;
    let y = vy;
    let w = vw;
    let h = STATUS_BAR_H;

    // Frosted glass background (same as desktop taskbar style)
    framebuffer::fill_rect_alpha(x.max(0) as u32, y.max(0) as u32, w, h, 0x040A06, 180);
    framebuffer::fill_rect_alpha(x.max(0) as u32, y.max(0) as u32, w, h, 0x00AA44, 8);

    // Bottom separator — chrome dim
    framebuffer::fill_rect(x.max(0) as u32, (y + h as i32 - 1).max(0) as u32, w, 1, CHROME_GHOST);

    // Time (centered, bright chrome)
    let cw = char_width();
    draw_text_centered(x + w as i32 / 2, y + 14, time_str, CHROME_BRIGHT);

    // Left: TrustOS label
    draw_text(x + 12, y + 14, "TrustOS", GREEN_SECONDARY);

    // Right: battery + signal indicators
    let rx = x + w as i32 - 12;
    // Battery icon: small rect
    let bx = (rx - 24).max(0) as u32;
    let by = (y + 16).max(0) as u32;
    framebuffer::draw_rect(bx, by, 18, 10, CHROME_DIM);
    framebuffer::fill_rect(bx + 18, by + 3, 2, 4, CHROME_DIM);
    framebuffer::fill_rect(bx + 2, by + 2, 14, 6, GREEN_SECONDARY); // fill

    // WiFi arcs (3 arcs, small)
    let wx = (rx - 50) as i32;
    let wy = y + 24;
    for ring in 0..3u32 {
        let r = 2 + ring * 3;
        let r2 = (r * r) as i32;
        let ri = r.saturating_sub(1);
        let r2i = (ri * ri) as i32;
        for dy in 0..=r as i32 {
            for dx in -(r as i32)..=(r as i32) {
                let d2 = dx * dx + dy * dy;
                if d2 <= r2 && d2 >= r2i && dy <= 0 {
                    let col = if ring == 0 { GREEN_SECONDARY } else { GREEN_MUTED };
                    put_px(wx + dx, wy + dy, col);
                }
            }
        }
    }
    put_px(wx, wy + 1, GREEN_SECONDARY); // center dot
}

// ═══════════════════════════════════════════════════════════════
// Home Screen — App Grid
// ═══════════════════════════════════════════════════════════════

pub fn draw_home_screen(
    vx: i32, vy: i32, vw: u32, vh: u32,
    highlighted: i32, _frame: u64,
) {
    let n = MOBILE_APPS.len() as u32;
    let rows = (n + GRID_COLS - 1) / GRID_COLS;

    // Content area: below status bar, above dock
    let content_top = vy + STATUS_BAR_H as i32;
    let content_bottom = vy + vh as i32 - DOCK_H as i32 - GESTURE_BAR_H as i32;
    let content_h = (content_bottom - content_top).max(0) as u32;

    // Calculate grid dimensions
    let total_icon_w = GRID_COLS * ICON_SIZE + (GRID_COLS - 1) * ICON_GAP_X;
    let grid_start_x = vx + (vw as i32 - total_icon_w as i32) / 2;

    let total_grid_h = rows * ICON_CELL_H;
    let grid_start_y = content_top + (content_h as i32 - total_grid_h as i32) / 2;

    for i in 0..n {
        let col = i % GRID_COLS;
        let row = i / GRID_COLS;
        let app = &MOBILE_APPS[i as usize];

        let ix = grid_start_x + (col * (ICON_SIZE + ICON_GAP_X)) as i32;
        let iy = grid_start_y + (row * ICON_CELL_H) as i32;
        let is_highlighted = highlighted == i as i32;

        // === Icon background — rounded dark square with chrome border ===
        // Same style as desktop dock icons
        draw_rounded_rect(ix, iy, ICON_SIZE, ICON_SIZE, ICON_RADIUS, 0xFF060A06);
        if is_highlighted {
            draw_rounded_rect_border(ix, iy, ICON_SIZE, ICON_SIZE, ICON_RADIUS, app.accent);
            // Glow on highlight
            draw_rounded_rect_border(ix - 1, iy - 1, ICON_SIZE + 2, ICON_SIZE + 2, ICON_RADIUS + 1, GREEN_GHOST);
        } else {
            draw_rounded_rect_border(ix, iy, ICON_SIZE, ICON_SIZE, ICON_RADIUS, CHROME_GHOST);
        }

        // === Pixel art icon (same rendering as desktop sidebar) ===
        let draw_color = if is_highlighted { app.accent } else { GREEN_SUBTLE };
        let cx = ix + ICON_SIZE as i32 / 2;
        let cy = iy + ICON_SIZE as i32 / 2;
        draw_icon_art(cx, cy, app.icon_idx, draw_color, is_highlighted);

        // === Label below icon ===
        let label_color = if is_highlighted { GREEN_PRIMARY } else { TEXT_SECONDARY };
        let lx = ix + ICON_SIZE as i32 / 2;
        let ly = iy + ICON_SIZE as i32 + 2;
        draw_text_centered(lx, ly, app.name, label_color);
    }
}

// ═══════════════════════════════════════════════════════════════
// Dock — Bottom glass bar with pinned apps
// ═══════════════════════════════════════════════════════════════

pub fn draw_dock(vx: i32, vy: i32, vw: u32, vh: u32, highlighted_dock: i32, _frame: u64) {
    let dock_y = vy + vh as i32 - DOCK_H as i32 - GESTURE_BAR_H as i32;
    let dock_x = vx + DOCK_MARGIN as i32;
    let dock_w = vw - DOCK_MARGIN * 2;
    let dock_h = DOCK_H - 10; // inner height

    // Glass panel background — same style as desktop taskbar
    draw_glass_panel(dock_x, dock_y, dock_w, dock_h, DOCK_RADIUS, 200);
    // Top chrome edge
    framebuffer::fill_rect_alpha(
        (dock_x + DOCK_RADIUS as i32).max(0) as u32,
        dock_y.max(0) as u32,
        dock_w.saturating_sub(DOCK_RADIUS * 2), 1,
        CHROME_DIM, 120,
    );

    // Dock icons — evenly spaced
    let total_w = DOCK_ICON_COUNT as u32 * DOCK_ICON_SIZE + (DOCK_ICON_COUNT as u32 - 1) * 12;
    let start_x = dock_x + (dock_w as i32 - total_w as i32) / 2;
    let icon_y = dock_y + (dock_h as i32 - DOCK_ICON_SIZE as i32) / 2;

    for (di, &app_idx) in DOCK_APPS.iter().enumerate() {
        let app = &MOBILE_APPS[app_idx];
        let ix = start_x + (di as u32 * (DOCK_ICON_SIZE + 12)) as i32;
        let is_hl = highlighted_dock == di as i32;

        // Icon background
        draw_rounded_rect(ix, icon_y, DOCK_ICON_SIZE, DOCK_ICON_SIZE, 12, 0xFF060A06);
        if is_hl {
            draw_rounded_rect_border(ix, icon_y, DOCK_ICON_SIZE, DOCK_ICON_SIZE, 12, app.accent);
        } else {
            draw_rounded_rect_border(ix, icon_y, DOCK_ICON_SIZE, DOCK_ICON_SIZE, 12, CHROME_GHOST);
        }

        // Icon art
        let draw_col = if is_hl { app.accent } else { GREEN_SUBTLE };
        let cx = ix + DOCK_ICON_SIZE as i32 / 2;
        let cy = icon_y + DOCK_ICON_SIZE as i32 / 2;
        draw_icon_art(cx, cy, app.icon_idx, draw_col, is_hl);
    }
}

// ═══════════════════════════════════════════════════════════════
// Gesture Bar — bottom pill
// ═══════════════════════════════════════════════════════════════

pub fn draw_gesture_bar(vx: i32, vy: i32, vw: u32, vh: u32) {
    let bar_w = GESTURE_BAR_W;
    let bar_h = GESTURE_BAR_H;
    let bx = vx + (vw as i32 - bar_w as i32) / 2;
    let by = vy + vh as i32 - bar_h as i32 - 4;
    draw_rounded_rect(bx, by, bar_w, bar_h, 3, CHROME_BRIGHT);
}

// ═══════════════════════════════════════════════════════════════
// App Bar — thin glass overlay at top of fullscreen app
// ═══════════════════════════════════════════════════════════════

pub fn draw_app_bar(vx: i32, vy: i32, vw: u32, title: &str, _frame: u64) {
    let h = APP_BAR_H;
    // Frosted glass
    framebuffer::fill_rect_alpha(vx.max(0) as u32, vy.max(0) as u32, vw, h, 0x0A1A0A, 220);
    // Bottom edge
    framebuffer::fill_rect((vx).max(0) as u32, (vy + h as i32 - 1).max(0) as u32, vw, 1, CHROME_DIM);
    // Green accent line at top
    framebuffer::fill_rect_alpha(vx.max(0) as u32, vy.max(0) as u32, vw, 1, 0x00FF66, 15);
    // Title centered
    draw_text_centered(vx + vw as i32 / 2, vy + 10, title, TEXT_PRIMARY);
    // Back arrow (left)
    draw_text(vx + 10, vy + 10, "<", GREEN_SECONDARY);
}

// ═══════════════════════════════════════════════════════════════
// App Switcher — horizontal card carousel
// ═══════════════════════════════════════════════════════════════

pub fn draw_app_switcher(
    vx: i32, vy: i32, vw: u32, vh: u32,
    windows: &[(u32, &str)], // (window_id, title)
    scroll_x: i32, _frame: u64,
) {
    // Dark overlay
    framebuffer::fill_rect_alpha(vx.max(0) as u32, vy.max(0) as u32, vw, vh, 0x000000, 180);

    if windows.is_empty() {
        draw_text_centered(vx + vw as i32 / 2, vy + vh as i32 / 2, "No apps open", TEXT_SECONDARY);
        return;
    }

    // Card dimensions: proportional to viewport
    let card_w = (vw * 7 / 10).min(400);
    let card_h = (vh * 5 / 10).min(600);
    let card_y = vy + (vh as i32 - card_h as i32) / 2;

    let total_cards_w = windows.len() as u32 * card_w + (windows.len() as u32).saturating_sub(1) * SWITCHER_CARD_GAP;
    let start_x = vx + (vw as i32 - total_cards_w as i32) / 2 - scroll_x;

    for (i, &(wid, title)) in windows.iter().enumerate() {
        let cx = start_x + (i as u32 * (card_w + SWITCHER_CARD_GAP)) as i32;
        // Card background — dark rounded rect with chrome border (same as desktop windows)
        draw_rounded_rect(cx, card_y, card_w, card_h, SWITCHER_CARD_RADIUS, 0xFF0A0A0A);
        draw_rounded_rect_border(cx, card_y, card_w, card_h, SWITCHER_CARD_RADIUS, CHROME_DIM);
        // Title bar
        framebuffer::fill_rect_alpha(cx.max(0) as u32, card_y.max(0) as u32, card_w, 28, 0x0A1A0A, 220);
        draw_text(cx + 10, card_y + 6, title, TEXT_PRIMARY);
        // Close X
        let close_x = cx + card_w as i32 - 20;
        draw_text(close_x, card_y + 6, "X", ACCENT_RED);
        // App preview hint
        draw_text_centered(cx + card_w as i32 / 2, card_y + card_h as i32 / 2, title, TEXT_SECONDARY);
    }
}

// ═══════════════════════════════════════════════════════════════
// Control Center — drop-down panel
// ═══════════════════════════════════════════════════════════════

pub fn draw_control_center(vx: i32, vy: i32, vw: u32, _vh: u32, progress: u8, _frame: u64) {
    if progress == 0 { return; }
    let h = (340u32 * progress as u32 / 100).max(1);
    let w = vw.saturating_sub(24);
    let x = vx + 12;
    let y = vy;

    // Glass panel
    draw_glass_panel(x, y, w, h, CC_RADIUS, 230);
    // Bright Chrome top edge
    framebuffer::fill_rect_alpha(
        (x + CC_RADIUS as i32).max(0) as u32, y.max(0) as u32,
        w.saturating_sub(CC_RADIUS * 2), 1, CHROME_BRIGHT, 60,
    );

    if h < 100 { return; } // too small to show controls

    let cx = x + w as i32 / 2;
    let mut ty = y + 20;

    // Brightness bar
    draw_text(x + 16, ty, "Brightness", TEXT_PRIMARY);
    ty += 20;
    let bar_w = w - 40;
    framebuffer::fill_rect((x + 20).max(0) as u32, ty.max(0) as u32, bar_w, 6, CHROME_GHOST);
    framebuffer::fill_rect((x + 20).max(0) as u32, ty.max(0) as u32, bar_w * 7 / 10, 6, GREEN_SECONDARY);
    ty += 24;

    // Volume bar
    draw_text(x + 16, ty, "Volume", TEXT_PRIMARY);
    ty += 20;
    framebuffer::fill_rect((x + 20).max(0) as u32, ty.max(0) as u32, bar_w, 6, CHROME_GHOST);
    framebuffer::fill_rect((x + 20).max(0) as u32, ty.max(0) as u32, bar_w * 5 / 10, 6, GREEN_SECONDARY);
    ty += 24;

    // Toggle tiles (WiFi, Bluetooth, Airplane, DND)
    let tile_size = 50u32;
    let tile_gap = 10u32;
    let tiles = ["WiFi", "BT", "Air", "DND"];
    let tile_states = [true, false, false, false];
    let total_tiles_w = tiles.len() as u32 * tile_size + (tiles.len() as u32 - 1) * tile_gap;
    let tiles_x = x + (w as i32 - total_tiles_w as i32) / 2;

    for (i, &label) in tiles.iter().enumerate() {
        let tx = tiles_x + (i as u32 * (tile_size + tile_gap)) as i32;
        let bg = if tile_states[i] { GREEN_GHOST } else { BG_DEEPEST };
        let border = if tile_states[i] { GREEN_SECONDARY } else { CHROME_GHOST };
        draw_rounded_rect(tx, ty, tile_size, tile_size, 10, bg);
        draw_rounded_rect_border(tx, ty, tile_size, tile_size, 10, border);
        draw_text_centered(tx + tile_size as i32 / 2, ty + tile_size as i32 / 2 - 7, label, TEXT_PRIMARY);
    }
}

// ═══════════════════════════════════════════════════════════════
// Phone Frame — glass border around viewport
// ═══════════════════════════════════════════════════════════════

pub fn draw_phone_frame(vx: i32, vy: i32, vw: u32, vh: u32) {
    // Outer glow (subtle green)
    draw_rounded_rect_border(vx - 3, vy - 3, vw + 6, vh + 6, 18, GREEN_GHOST);
    // Chrome border (bright, like desktop window focused border)
    draw_rounded_rect_border(vx - 2, vy - 2, vw + 4, vh + 4, 16, CHROME_BRIGHT);
    // Inner border (dim)
    draw_rounded_rect_border(vx - 1, vy - 1, vw + 2, vh + 2, 14, CHROME_DIM);
}

// ═══════════════════════════════════════════════════════════════
// Gesture Handling
// ═══════════════════════════════════════════════════════════════

#[derive(Clone, Copy, PartialEq)]
pub enum MobileAction {
    None,
    GoHome,
    OpenSwitcher,
    OpenControlCenter,
    CloseControlCenter,
    LaunchApp(u8),      // app index into MOBILE_APPS
    LaunchDockApp(u8),  // dock slot index
    BackFromApp,
    CloseSwitcherCard(u32),
}

/// Process gesture in viewport-local coordinates.
/// (x, y) should be relative to viewport origin.
pub fn handle_gesture(
    state: &mut MobileState,
    event: GestureEvent,
) -> MobileAction {
    match event {
        GestureEvent::TapDown(x, y) => {
            state.gesture_start_x = x;
            state.gesture_start_y = y;
            state.gesture_active = true;
            state.gesture_from_bottom = y > state.vp_h as i32 - 60;
            state.gesture_from_top = y < 44;
            // Highlight detection (home screen)
            if state.view == MobileView::Home {
                state.highlighted_icon = hit_test_grid(x, y, state.vp_w, state.vp_h);
            }
            MobileAction::None
        }
        GestureEvent::TapUp(x, y) => {
            state.gesture_active = false;
            let dx = x - state.gesture_start_x;
            let dy = y - state.gesture_start_y;
            let dist = fast_isqrt(dx * dx + dy * dy);

            if dist < 15 {
                // Tap — check what was tapped
                return handle_tap(state, x, y);
            }

            // Swipe detection
            if dy.abs() > dx.abs() && dy.abs() > 30 {
                if dy < 0 {
                    // Swipe up
                    if state.gesture_from_bottom {
                        // From bottom edge: go home or open switcher
                        if state.view == MobileView::AppFullscreen {
                            return MobileAction::GoHome;
                        } else if state.view == MobileView::Home {
                            return MobileAction::OpenSwitcher;
                        }
                    }
                    if state.view == MobileView::ControlCenter {
                        return MobileAction::CloseControlCenter;
                    }
                } else {
                    // Swipe down
                    if state.gesture_from_top && state.view == MobileView::Home {
                        return MobileAction::OpenControlCenter;
                    }
                }
            }

            state.highlighted_icon = -1;
            MobileAction::None
        }
        GestureEvent::Move(_x, _y) => {
            MobileAction::None
        }
    }
}

fn handle_tap(state: &mut MobileState, x: i32, y: i32) -> MobileAction {
    match state.view {
        MobileView::Home => {
            // Check grid icons
            let idx = hit_test_grid(x, y, state.vp_w, state.vp_h);
            if idx >= 0 && (idx as usize) < MOBILE_APPS.len() {
                state.highlighted_icon = -1;
                return MobileAction::LaunchApp(idx as u8);
            }
            // Check dock
            let dock_idx = hit_test_dock(x, y, state.vp_w, state.vp_h);
            if dock_idx >= 0 {
                return MobileAction::LaunchDockApp(dock_idx as u8);
            }
            MobileAction::None
        }
        MobileView::AppFullscreen => {
            // Tap back arrow area
            if y < APP_BAR_H as i32 && x < 40 {
                return MobileAction::BackFromApp;
            }
            MobileAction::None
        }
        MobileView::AppSwitcher => {
            // Tap on a card → switch to that app (simplified)
            MobileAction::GoHome
        }
        MobileView::ControlCenter => {
            MobileAction::None
        }
    }
}

#[derive(Clone, Copy)]
pub enum GestureEvent {
    TapDown(i32, i32),
    TapUp(i32, i32),
    Move(i32, i32),
}

// ═══════════════════════════════════════════════════════════════
// Hit Testing
// ═══════════════════════════════════════════════════════════════

fn hit_test_grid(x: i32, y: i32, vw: u32, vh: u32) -> i32 {
    let n = MOBILE_APPS.len() as u32;
    let rows = (n + GRID_COLS - 1) / GRID_COLS;
    let content_top = STATUS_BAR_H as i32;
    let content_bottom = vh as i32 - DOCK_H as i32 - GESTURE_BAR_H as i32;
    let content_h = (content_bottom - content_top).max(0) as u32;
    let total_icon_w = GRID_COLS * ICON_SIZE + (GRID_COLS - 1) * ICON_GAP_X;
    let grid_start_x = (vw as i32 - total_icon_w as i32) / 2;
    let total_grid_h = rows * ICON_CELL_H;
    let grid_start_y = content_top + (content_h as i32 - total_grid_h as i32) / 2;

    for i in 0..n {
        let col = i % GRID_COLS;
        let row = i / GRID_COLS;
        let ix = grid_start_x + (col * (ICON_SIZE + ICON_GAP_X)) as i32;
        let iy = grid_start_y + (row * ICON_CELL_H) as i32;
        if x >= ix && x < ix + ICON_SIZE as i32 && y >= iy && y < iy + ICON_SIZE as i32 {
            return i as i32;
        }
    }
    -1
}

fn hit_test_dock(x: i32, y: i32, vw: u32, vh: u32) -> i32 {
    let dock_y = vh as i32 - DOCK_H as i32 - GESTURE_BAR_H as i32;
    let dock_x = DOCK_MARGIN as i32;
    let dock_w = vw - DOCK_MARGIN * 2;
    let dock_h = DOCK_H - 10;

    if y < dock_y || y > dock_y + dock_h as i32 { return -1; }

    let total_w = DOCK_ICON_COUNT as u32 * DOCK_ICON_SIZE + (DOCK_ICON_COUNT as u32 - 1) * 12;
    let start_x = dock_x + (dock_w as i32 - total_w as i32) / 2;

    for di in 0..DOCK_ICON_COUNT {
        let ix = start_x + (di as u32 * (DOCK_ICON_SIZE + 12)) as i32;
        if x >= ix && x < ix + DOCK_ICON_SIZE as i32 {
            return di as i32;
        }
    }
    -1
}

// ═══════════════════════════════════════════════════════════════
// Animation
// ═══════════════════════════════════════════════════════════════

pub fn tick_animations(state: &mut MobileState) {
    state.anim_frame = state.anim_frame.wrapping_add(1);

    // Control center slide
    if state.view == MobileView::ControlCenter && state.cc_progress < 100 {
        state.cc_progress = (state.cc_progress + 8).min(100);
    }
    if state.view != MobileView::ControlCenter && state.cc_progress > 0 {
        state.cc_progress = state.cc_progress.saturating_sub(8);
    }

    // Closing card fade
    state.closing_cards.retain_mut(|(_id, fade)| {
        *fade = fade.saturating_sub(4);
        *fade > 0
    });
}

// ═══════════════════════════════════════════════════════════════
// Icon Pixel Art — exact same as desktop sidebar icons
// ═══════════════════════════════════════════════════════════════

fn draw_icon_art(cx: i32, cy: i32, icon_idx: u8, color: u32, _highlighted: bool) {
    match icon_idx {
        0 => { // Terminal: rounded rect with >_ prompt
            draw_rounded_rect_border(cx - 14, cy - 10, 28, 20, 3, color);
            framebuffer::fill_rect((cx - 13).max(0) as u32, (cy - 9).max(0) as u32, 26, 3, color);
            // Traffic light dots
            framebuffer::fill_rect((cx - 11).max(0) as u32, (cy - 8).max(0) as u32, 2, 1, 0xFF0A0A0A);
            framebuffer::fill_rect((cx - 8).max(0) as u32, (cy - 8).max(0) as u32, 2, 1, 0xFF0A0A0A);
            framebuffer::fill_rect((cx - 5).max(0) as u32, (cy - 8).max(0) as u32, 2, 1, 0xFF0A0A0A);
            // >_
            draw_text(cx - 8, cy - 2, ">", color);
            framebuffer::fill_rect((cx - 2).max(0) as u32, cy.max(0) as u32, 8, 2, color);
        }
        1 => { // Files: folder
            framebuffer::fill_rect((cx - 14).max(0) as u32, (cy - 8).max(0) as u32, 12, 5, color);
            framebuffer::fill_rect((cx - 14).max(0) as u32, (cy - 3).max(0) as u32, 28, 15, color);
            framebuffer::fill_rect((cx - 12).max(0) as u32, (cy - 1).max(0) as u32, 24, 11, 0xFF0A0A0A);
            framebuffer::fill_rect((cx - 8).max(0) as u32, (cy + 2).max(0) as u32, 16, 1, 0xFF303020);
            framebuffer::fill_rect((cx - 8).max(0) as u32, (cy + 5).max(0) as u32, 12, 1, 0xFF303020);
        }
        2 => { // Editor: document with code
            framebuffer::fill_rect((cx - 10).max(0) as u32, (cy - 12).max(0) as u32, 20, 24, color);
            framebuffer::fill_rect((cx + 4).max(0) as u32, (cy - 12).max(0) as u32, 6, 6, 0xFF0A0A0A);
            framebuffer::fill_rect((cx + 4).max(0) as u32, (cy - 12).max(0) as u32, 1, 6, color);
            framebuffer::fill_rect((cx + 4).max(0) as u32, (cy - 7).max(0) as u32, 6, 1, color);
            framebuffer::fill_rect((cx - 8).max(0) as u32, (cy - 6).max(0) as u32, 16, 16, 0xFF0A0A0A);
            framebuffer::fill_rect((cx - 6).max(0) as u32, (cy - 4).max(0) as u32, 6, 1, 0xFF6688CC);
            framebuffer::fill_rect((cx - 6).max(0) as u32, (cy - 2).max(0) as u32, 10, 1, color);
            framebuffer::fill_rect((cx - 6).max(0) as u32, cy.max(0) as u32, 8, 1, 0xFFCC8844);
            framebuffer::fill_rect((cx - 6).max(0) as u32, (cy + 2).max(0) as u32, 12, 1, color);
            framebuffer::fill_rect((cx - 6).max(0) as u32, (cy + 4).max(0) as u32, 4, 1, 0xFF88BB44);
            framebuffer::fill_rect((cx - 6).max(0) as u32, (cy + 6).max(0) as u32, 9, 1, color);
        }
        3 => { // Calculator
            draw_rounded_rect_border(cx - 10, cy - 12, 20, 24, 2, color);
            framebuffer::fill_rect((cx - 8).max(0) as u32, (cy - 10).max(0) as u32, 16, 6, 0xFF1A3320);
            draw_text(cx - 4, cy - 10, "42", 0xFF40FF40);
            for row in 0..3u32 {
                for col in 0..3u32 {
                    let bx = (cx as u32).wrapping_sub(8) + col * 6;
                    let by = (cy as u32).wrapping_sub(1) + row * 5;
                    let btn = if row == 2 && col == 2 { 0xFFCC6633 } else { color };
                    framebuffer::fill_rect(bx.max(0), by.max(0), 4, 3, btn);
                }
            }
        }
        4 => { // Network: WiFi arcs
            let acx = cx;
            let acy = cy + 4;
            for ring in 0..3u32 {
                let r = 4 + ring * 4;
                let r2 = (r * r) as i32;
                let ri = r.saturating_sub(2);
                let r2i = (ri * ri) as i32;
                for dy in 0..=r as i32 {
                    for dx in -(r as i32)..=(r as i32) {
                        let d2 = dx * dx + dy * dy;
                        if d2 <= r2 && d2 >= r2i && dy <= 0 {
                            let fade = if ring == 0 { color } else { GREEN_GHOST };
                            put_px(acx + dx, acy + dy, fade);
                        }
                    }
                }
            }
            framebuffer::fill_rect((cx - 1).max(0) as u32, (cy + 3).max(0) as u32, 3, 3, color);
        }
        5 => { // Games: controller
            framebuffer::fill_rect((cx - 12).max(0) as u32, (cy - 4).max(0) as u32, 24, 12, color);
            framebuffer::fill_rect((cx - 14).max(0) as u32, (cy - 2).max(0) as u32, 4, 8, color);
            framebuffer::fill_rect((cx + 10).max(0) as u32, (cy - 2).max(0) as u32, 4, 8, color);
            framebuffer::fill_rect((cx - 11).max(0) as u32, (cy - 3).max(0) as u32, 22, 10, 0xFF0A0A0A);
            framebuffer::fill_rect((cx - 9).max(0) as u32, (cy - 1).max(0) as u32, 5, 1, color);
            framebuffer::fill_rect((cx - 7).max(0) as u32, (cy - 3).max(0) as u32, 1, 5, color);
            framebuffer::fill_rect((cx + 5).max(0) as u32, (cy - 2).max(0) as u32, 2, 2, 0xFF4488DD);
            framebuffer::fill_rect((cx + 8).max(0) as u32, (cy - 1).max(0) as u32, 2, 2, ACCENT_RED);
            framebuffer::fill_rect((cx + 5).max(0) as u32, (cy + 1).max(0) as u32, 2, 2, 0xFF44DD44);
            framebuffer::fill_rect((cx + 8).max(0) as u32, (cy + 2).max(0) as u32, 2, 2, 0xFFDDDD44);
        }
        6 => { // Browser: globe
            for dy in -8i32..=8 {
                for dx in -8i32..=8 {
                    let d2 = dx * dx + dy * dy;
                    if d2 <= 64 && d2 >= 49 {
                        put_px(cx + dx, cy + dy, color);
                    }
                }
            }
            // Meridian + equator
            framebuffer::fill_rect((cx - 1).max(0) as u32, (cy - 7).max(0) as u32, 2, 14, color);
            framebuffer::fill_rect((cx - 7).max(0) as u32, (cy - 1).max(0) as u32, 14, 2, color);
        }
        7 => { // TrustEd: 3D cube wireframe
            // Front face
            draw_rounded_rect_border(cx - 8, cy - 6, 16, 12, 1, color);
            // Back face offset
            draw_rounded_rect_border(cx - 4, cy - 10, 16, 12, 1, GREEN_GHOST);
            // Connecting edges
            put_px(cx - 8, cy - 6, color); put_px(cx - 4, cy - 10, color);
            put_px(cx + 7, cy - 6, color); put_px(cx + 11, cy - 10, color);
        }
        8 => { // Settings: gear
            for dy in 0..18u32 {
                for dx in 0..18u32 {
                    let ddx = dx as i32 - 9;
                    let ddy = dy as i32 - 9;
                    let dist2 = ddx * ddx + ddy * ddy;
                    // Outer ring + teeth
                    if dist2 >= 36 && dist2 <= 81 {
                        let angle = atan2_approx_octant(ddy, ddx);
                        if dist2 > 56 {
                            // teeth
                            if angle % 2 == 0 { put_px(cx - 9 + dx as i32, cy - 9 + dy as i32, color); }
                        } else {
                            put_px(cx - 9 + dx as i32, cy - 9 + dy as i32, color);
                        }
                    }
                    // Inner hole
                    if dist2 <= 9 {
                        put_px(cx - 9 + dx as i32, cy - 9 + dy as i32, 0xFF0A0A0A);
                    }
                }
            }
        }
        9 => { // About: info circle
            for dy in -8i32..=8 {
                for dx in -8i32..=8 {
                    let d2 = dx * dx + dy * dy;
                    if d2 <= 64 && d2 >= 49 { put_px(cx + dx, cy + dy, color); }
                }
            }
            draw_text(cx - 2, cy - 6, "i", color);
        }
        10 => { // Music: note
            // Note head
            framebuffer::fill_rect((cx - 3).max(0) as u32, (cy + 2).max(0) as u32, 6, 4, color);
            // Stem
            framebuffer::fill_rect((cx + 2).max(0) as u32, (cy - 8).max(0) as u32, 2, 12, color);
            // Flag
            framebuffer::fill_rect((cx + 3).max(0) as u32, (cy - 8).max(0) as u32, 4, 2, color);
            framebuffer::fill_rect((cx + 5).max(0) as u32, (cy - 6).max(0) as u32, 2, 2, color);
        }
        11 => { // Chess: chess piece (king)
            // Cross on top
            framebuffer::fill_rect((cx - 1).max(0) as u32, (cy - 10).max(0) as u32, 2, 6, color);
            framebuffer::fill_rect((cx - 3).max(0) as u32, (cy - 8).max(0) as u32, 6, 2, color);
            // Body
            framebuffer::fill_rect((cx - 4).max(0) as u32, (cy - 4).max(0) as u32, 8, 10, color);
            framebuffer::fill_rect((cx - 3).max(0) as u32, (cy - 3).max(0) as u32, 6, 8, 0xFF0A0A0A);
            // Base
            framebuffer::fill_rect((cx - 6).max(0) as u32, (cy + 5).max(0) as u32, 12, 3, color);
        }
        _ => {
            // Generic app icon: rounded square with ?
            draw_rounded_rect(cx - 12, cy - 12, 24, 24, 6, color);
            draw_text(cx - 3, cy - 6, "?", 0xFF0A0A0A);
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// Public API — MOBILE_APPS access for desktop.rs
// ═══════════════════════════════════════════════════════════════

pub fn app_count() -> usize { MOBILE_APPS.len() }
pub fn app_name(idx: usize) -> &'static str { MOBILE_APPS[idx].name }
pub fn dock_app_index(slot: usize) -> usize { DOCK_APPS[slot] }
pub fn dock_app_count() -> usize { DOCK_ICON_COUNT }
