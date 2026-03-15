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
const GRID_COLS: u32 = 3;
const ICON_SIZE: u32 = 72;
const ICON_RADIUS: u32 = 18;
const ICON_LABEL_H: u32 = 18;
const ICON_CELL_H: u32 = ICON_SIZE + ICON_LABEL_H + 24; // icon + label + generous vertical gap
const GRID_PADDING_X: u32 = 20; // left/right padding from viewport edge

/// Dock
const DOCK_ICON_COUNT: usize = 5;
const DOCK_ICON_SIZE: u32 = 52;
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
    /// Music widget: visualizer dropdown open
    pub music_dropdown_open: bool,
    /// Music widget: selected visualizer mode (0..7)
    pub music_viz_mode: u8,
    /// Terminal: command history lines
    pub term_lines: Vec<String>,
    /// Terminal: current input buffer
    pub term_input: String,
    /// Calculator: display value
    pub calc_display: String,
    /// Calculator: pending operation (0=none, 1=+, 2=-, 3=*, 4=/)
    pub calc_op: u8,
    /// Calculator: stored operand
    pub calc_operand: i64,
    /// Calculator: just pressed = (reset on next digit)
    pub calc_fresh: bool,
    /// Files: selected entry index (-1 = none)
    pub files_selected: i32,
    /// Files: current path depth (0=home, 1=subdir)
    pub files_depth: u8,
    /// Settings: selected row (-1 = none)
    pub settings_selected: i32,
    /// Settings: toggles [wifi, bt, airplane, dnd, dark_mode, notifications]
    pub settings_toggles: [bool; 6],
    /// Games: selected game (-1 = none)
    pub games_selected: i32,
    /// Browser: current page (0=home, 1=docs, 2=source, 3=downloads)
    pub browser_page: u8,
    /// Editor: cursor line
    pub editor_cursor_line: u32,
    /// Editor: active tab (0 or 1)
    pub editor_tab: u8,
    /// Chess: selected square (-1 = none)
    pub chess_selected: i32,
    /// Chess: whose turn (0=white, 1=black)
    pub chess_turn: u8,
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
            music_dropdown_open: false,
            music_viz_mode: 0,
            term_lines: Vec::new(),
            term_input: String::new(),
            calc_display: String::new(),
            calc_op: 0,
            calc_operand: 0,
            calc_fresh: false,
            files_selected: -1,
            files_depth: 0,
            settings_selected: -1,
            settings_toggles: [true, false, false, false, true, true],
            games_selected: -1,
            browser_page: 0,
            editor_cursor_line: 0,
            editor_tab: 0,
            chess_selected: -1,
            chess_turn: 0,
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

    // Content area: below status bar, above music widget zone
    // Music widget (WIDGET_H=130 + 8 margin) sits above dock
    const MUSIC_WIDGET_ZONE: u32 = 138;
    let content_top = vy + STATUS_BAR_H as i32;
    let content_bottom = vy + vh as i32 - DOCK_H as i32 - GESTURE_BAR_H as i32 - MUSIC_WIDGET_ZONE as i32;
    let content_h = (content_bottom - content_top).max(0) as u32;

    // ── Search bar ──
    let search_h = 36u32;
    let search_margin = 14u32;
    let search_w = vw.saturating_sub(search_margin * 2);
    let search_x = vx + search_margin as i32;
    let search_y = content_top + 10;
    // Frosted glass search background
    draw_rounded_rect(search_x, search_y, search_w, search_h, 12, 0xFF0A120E);
    draw_rounded_rect_border(search_x, search_y, search_w, search_h, 12, CHROME_GHOST);
    // Search icon (magnifier)
    let mag_x = search_x + 12;
    let mag_y = search_y + 8;
    // Circle part
    for dy in -4i32..=4 {
        for dx in -4i32..=4 {
            let d2 = dx * dx + dy * dy;
            if d2 >= 9 && d2 <= 16 {
                put_px(mag_x + dx, mag_y + dy, CHROME_DIM);
            }
        }
    }
    // Handle
    put_px(mag_x + 4, mag_y + 4, CHROME_DIM);
    put_px(mag_x + 5, mag_y + 5, CHROME_DIM);
    put_px(mag_x + 6, mag_y + 6, CHROME_DIM);
    // Placeholder text
    draw_text(search_x + 26, search_y + 10, "Search", TEXT_SECONDARY);

    // Grid starts below the search bar
    let grid_area_top = search_y + search_h as i32 + 10;
    let grid_area_h = (content_bottom - grid_area_top).max(0) as u32;

    // Calculate grid with EQUAL spacing: distribute icons evenly across width
    let grid_inner_w = vw.saturating_sub(GRID_PADDING_X * 2);
    // gap_x = space between icon centers minus icon size, evenly distributed
    let cell_w = grid_inner_w / GRID_COLS; // each cell is this wide
    let grid_start_x = vx + GRID_PADDING_X as i32;

    let total_grid_h = rows * ICON_CELL_H;
    let grid_start_y = grid_area_top + (grid_area_h as i32 - total_grid_h as i32) / 2;

    for i in 0..n {
        let col = i % GRID_COLS;
        let row = i / GRID_COLS;
        let app = &MOBILE_APPS[i as usize];

        // Center each icon within its evenly-sized cell
        let ix = grid_start_x + (col * cell_w) as i32 + (cell_w as i32 - ICON_SIZE as i32) / 2;
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

        // Label below dock icon
        let label_color = if is_hl { GREEN_PRIMARY } else { TEXT_SECONDARY };
        let lx = ix + DOCK_ICON_SIZE as i32 / 2;
        let ly = icon_y + DOCK_ICON_SIZE as i32 + 2;
        draw_text_centered(lx, ly, app.name, label_color);
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
    MusicTogglePlay,
    MusicStop,
    MusicToggleDropdown,
    MusicSetVizMode(u8),
    /// Calculator: button pressed (encoded: 0-9=digits, 10=., 11=+, 12=-, 13=*, 14=/, 15==, 16=C, 17=+/-,18=%)
    CalcButton(u8),
    /// Files: tap on entry
    FilesTap(u8),
    /// Files: back navigation
    FilesBack,
    /// Settings: toggle a setting row
    SettingsTap(u8),
    /// Games: select/launch a game
    GamesTap(u8),
    /// Browser: navigate to page
    BrowserNav(u8),
    /// Editor: tap on line
    EditorTap(u8),
    /// Editor: switch tab
    EditorSwitchTab(u8),
    /// Chess: tap on square (0-63)
    ChessTap(u8),
    /// Music app: toggle play from fullscreen
    MusicAppToggle,
    /// Terminal: submit command
    TermSubmit,
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
            // Check music widget buttons first (above dock)
            let music_btn = hit_test_music_widget(x, y, state.vp_w, state.vp_h, state.music_dropdown_open);
            if music_btn != MobileAction::None {
                return music_btn;
            }
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
            // Route tap to the active app
            if let Some(app_idx) = state.active_app_id {
                let local_y = y - APP_BAR_H as i32;
                return handle_app_tap(state, app_idx, x, local_y);
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

/// Route a tap inside a fullscreen app to the appropriate action.
/// x is viewport-local, y is relative to content area top (below app bar).
fn handle_app_tap(state: &MobileState, app_idx: u32, x: i32, y: i32) -> MobileAction {
    let vw = state.vp_w;
    let vh = state.vp_h;
    let pad = 14i32;

    match app_idx {
        // ── Calculator ──
        3 => {
            let ix = pad;
            let iw = (vw as i32 - pad * 2) as u32;
            let display_h = 60i32;
            let btn_h = 44i32;
            let btn_gap = 6i32;
            let btn_w = ((iw - 6 * 3) / 4) as i32;
            let grid_top = 10 + display_h + 14;
            // Map to button grid
            if y >= grid_top {
                let row = (y - grid_top) / (btn_h + btn_gap);
                let col = (x - ix) / (btn_w + btn_gap);
                if row >= 0 && row < 5 && col >= 0 && col < 4 {
                    // Encode: row*4 + col → map to CalcButton code
                    let btn_map: [[u8; 4]; 5] = [
                        [16, 17, 18, 14], // C, +/-, %, /
                        [7,  8,  9,  13], // 7, 8, 9, x
                        [4,  5,  6,  12], // 4, 5, 6, -
                        [1,  2,  3,  11], // 1, 2, 3, +
                        [0,  10, 15, 255],// 0, ., =, (empty)
                    ];
                    let code = btn_map[row as usize][col as usize];
                    if code != 255 {
                        return MobileAction::CalcButton(code);
                    }
                }
            }
            MobileAction::None
        }
        // ── Files ──
        1 => {
            let row_h = 40i32;
            let first_row_y = 32;
            if y >= first_row_y {
                let idx = (y - first_row_y) / row_h;
                if idx >= 0 && idx < 8 {
                    return MobileAction::FilesTap(idx as u8);
                }
            }
            // Back button in path bar
            if y < 28 && x < 80 && state.files_depth > 0 {
                return MobileAction::FilesBack;
            }
            MobileAction::None
        }
        // ── Settings ──
        8 => {
            let row_h = 52i32;
            let first_row_y = 10;
            if y >= first_row_y {
                let idx = (y - first_row_y) / row_h;
                if idx >= 0 && idx < 6 {
                    return MobileAction::SettingsTap(idx as u8);
                }
            }
            MobileAction::None
        }
        // ── Games ──
        5 => {
            let card_h = 56i32;
            let card_gap = 8i32;
            let first_card_y = 34;
            if y >= first_card_y {
                let idx = (y - first_card_y) / (card_h + card_gap);
                if idx >= 0 && idx < 5 {
                    return MobileAction::GamesTap(idx as u8);
                }
            }
            MobileAction::None
        }
        // ── Browser ──
        6 => {
            let page_y = 40;
            // URL bar tap → go home
            if y >= 4 && y < 34 {
                return MobileAction::BrowserNav(0);
            }
            if state.browser_page == 0 {
                // Home page: links start at page_y + 10 + 24 + 20 + 30 = page_y + 84
                let links_y = page_y + 84;
                let link_h = 20;
                if y >= links_y && y < links_y + link_h * 3 {
                    let idx = (y - links_y) / link_h;
                    return MobileAction::BrowserNav(idx as u8 + 1);
                }
            } else {
                // Sub-page: any tap on content area goes home
                if y >= page_y {
                    return MobileAction::BrowserNav(0);
                }
            }
            MobileAction::None
        }
        // ── Editor ──
        2 => {
            // Tab bar
            if y < 26 {
                if x < 80 {
                    return MobileAction::EditorSwitchTab(0);
                } else {
                    return MobileAction::EditorSwitchTab(1);
                }
            }
            // Code lines
            let line_h = 16;
            let code_start = 30;
            if y >= code_start {
                let line = (y - code_start) / line_h;
                if line >= 0 && line < 12 {
                    return MobileAction::EditorTap(line as u8);
                }
            }
            MobileAction::None
        }
        // ── Chess ──
        11 => {
            let board_size = (vw as i32 - 16).min((vh.saturating_sub(APP_BAR_H + 80)) as i32).min(400);
            let cell = board_size / 8;
            let board_x = (vw as i32 - board_size) / 2;
            let board_y = 10;
            if x >= board_x && x < board_x + board_size && y >= board_y && y < board_y + board_size {
                let col = (x - board_x) / cell;
                let row = (y - board_y) / cell;
                let sq = (row * 8 + col) as u8;
                return MobileAction::ChessTap(sq);
            }
            MobileAction::None
        }
        // ── Music (fullscreen) ──
        10 => {
            let iw = (vw as i32 - pad * 2) as u32;
            let art_size = iw.min(200);
            // Controls area approximately at: art_size + 16 + 20 + 28 + 20 = below art
            let ctrl_y = art_size as i32 + 16 + 20 + 28 + 20 + 14;
            let btn_w = 48i32;
            let btn_h = 36i32;
            let btn_gap = 10i32;
            let total_w = 5 * btn_w + 4 * btn_gap;
            let btn_start = pad + ((iw as i32 - total_w) / 2);
            if y >= ctrl_y && y < ctrl_y + btn_h {
                for i in 0..5 {
                    let bx = btn_start + i * (btn_w + btn_gap);
                    if x >= bx && x < bx + btn_w {
                        return MobileAction::MusicAppToggle;
                    }
                }
            }
            MobileAction::None
        }
        // ── Terminal ──
        0 => {
            // Tap bottom area => submit command
            let content_h = vh.saturating_sub(APP_BAR_H + 20);
            if y > content_h as i32 - 40 {
                return MobileAction::TermSubmit;
            }
            MobileAction::None
        }
        // Other apps: no interaction yet
        _ => MobileAction::None,
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
    const MUSIC_WIDGET_ZONE: u32 = 138;
    let content_top = STATUS_BAR_H as i32;
    let content_bottom = vh as i32 - DOCK_H as i32 - GESTURE_BAR_H as i32 - MUSIC_WIDGET_ZONE as i32;
    // Search bar occupies 36+10+10 = 56px from content_top
    let grid_area_top = content_top + 56;
    let grid_area_h = (content_bottom - grid_area_top).max(0) as u32;

    let grid_inner_w = vw.saturating_sub(GRID_PADDING_X * 2);
    let cell_w = grid_inner_w / GRID_COLS;
    let grid_start_x = GRID_PADDING_X as i32;

    let total_grid_h = rows * ICON_CELL_H;
    let grid_start_y = grid_area_top + (grid_area_h as i32 - total_grid_h as i32) / 2;

    for i in 0..n {
        let col = i % GRID_COLS;
        let row = i / GRID_COLS;
        let ix = grid_start_x + (col * cell_w) as i32 + (cell_w as i32 - ICON_SIZE as i32) / 2;
        let iy = grid_start_y + (row * ICON_CELL_H) as i32;
        if x >= ix && x < ix + ICON_SIZE as i32 && y >= iy && y < iy + ICON_SIZE as i32 {
            return i as i32;
        }
    }
    -1
}

/// Hit test the music widget control buttons and dropdown.
/// Returns MusicTogglePlay for play/pause button, MusicToggleDropdown for arrow, MusicSetVizMode for dropdown items, None otherwise.
fn hit_test_music_widget(x: i32, y: i32, vw: u32, vh: u32, dropdown_open: bool) -> MobileAction {
    const WIDGET_H: u32 = 130;
    const WIDGET_MARGIN: u32 = 14;
    const DROPDOWN_ITEM_H: u32 = 26;
    let widget_w = vw.saturating_sub(WIDGET_MARGIN * 2);
    let widget_x = WIDGET_MARGIN as i32;
    let num_modes = crate::visualizer::NUM_MODES as u32;
    let extra_h = if dropdown_open { num_modes * DROPDOWN_ITEM_H + 8 } else { 0 };
    let widget_y = vh as i32 - DOCK_H as i32 - GESTURE_BAR_H as i32 - WIDGET_H as i32 - 8 - extra_h as i32;

    let total_h = WIDGET_H + extra_h;

    // Check if tap is within widget + dropdown bounds
    if x < widget_x || x > widget_x + widget_w as i32 || y < widget_y || y > widget_y + total_h as i32 {
        return MobileAction::None;
    }

    // Title row: dropdown arrow area (right side of title row, y = widget_y+12, h=~18)
    let title_y = widget_y + 12;
    if y >= title_y && y < title_y + 18 && x > widget_x + widget_w as i32 / 2 {
        return MobileAction::MusicToggleDropdown;
    }

    // Button row controls
    let pad = 14u32;
    let ix = widget_x + pad as i32;
    let iw = widget_w.saturating_sub(pad * 2);
    let ctrl_y = widget_y + 92;
    let btn_w = 40i32;
    let btn_h = 20i32;
    let btn_gap = 6i32;
    let n_btns = 5i32;
    let total_btn_w = n_btns * btn_w + (n_btns - 1) * btn_gap;
    let btn_start_x = ix + (iw as i32 - total_btn_w) / 2;

    if y >= ctrl_y && y <= ctrl_y + btn_h {
        for bi in 0..5 {
            let bx = btn_start_x + bi * (btn_w + btn_gap);
            if x >= bx && x < bx + btn_w {
                return MobileAction::MusicTogglePlay;
            }
        }
    }

    // Dropdown menu items (below the main widget)
    if dropdown_open {
        let dd_y_start = widget_y + WIDGET_H as i32 + 4;
        for mi in 0..num_modes {
            let item_y = dd_y_start + (mi * DROPDOWN_ITEM_H) as i32;
            if y >= item_y && y < item_y + DROPDOWN_ITEM_H as i32 {
                return MobileAction::MusicSetVizMode(mi as u8);
            }
        }
    }

    MobileAction::None
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

// ═══════════════════════════════════════════════════════════════
// Mobile Music Player Widget — compact Now Playing card
// ═══════════════════════════════════════════════════════════════

/// Audio state passed from desktop.rs for the mobile music widget
pub struct MobileAudioInfo {
    pub playing: bool,
    pub beat: f32,
    pub energy: f32,
    pub sub_bass: f32,
    pub bass: f32,
    pub mid: f32,
    pub treble: f32,
    pub frame: u64,
}

/// Draw a compact music player widget above the dock.
/// Styled like an iOS Now Playing card with frosted glass.
/// Includes a dropdown arrow for visualizer mode selection.
pub fn draw_mobile_music_widget(
    vx: i32, vy: i32, vw: u32, vh: u32,
    audio: &MobileAudioInfo,
    dropdown_open: bool,
    viz_mode: u8,
) {
    const WIDGET_H: u32 = 130;
    const WIDGET_MARGIN: u32 = 14;
    const WIDGET_RADIUS: u32 = 16;
    const DROPDOWN_ITEM_H: u32 = 26;

    let widget_w = vw.saturating_sub(WIDGET_MARGIN * 2);
    let widget_x = vx + WIDGET_MARGIN as i32;
    // Position just above the dock (dock_y = vy + vh - DOCK_H - GESTURE_BAR_H)
    let extra_h = if dropdown_open { crate::visualizer::NUM_MODES as u32 * DROPDOWN_ITEM_H + 8 } else { 0 };
    let widget_y = vy + vh as i32 - DOCK_H as i32 - GESTURE_BAR_H as i32 - WIDGET_H as i32 - 8 - extra_h as i32;

    // ── Frosted glass background ──
    draw_glass_panel(widget_x, widget_y, widget_w, WIDGET_H, WIDGET_RADIUS, 210);

    let pad = 14u32;
    let ix = widget_x + pad as i32; // inner x
    let iw = widget_w.saturating_sub(pad * 2); // inner width
    let mut cy = widget_y + 12;

    // ── Title row ──
    let title = if audio.playing { "Now Playing" } else { "Music" };
    let title_color = if audio.playing { GREEN_PRIMARY } else { CHROME_MID };
    draw_text(ix, cy, title, title_color);

    // ── Visualizer mode label + dropdown arrow ──
    let mode_name = if (viz_mode as usize) < crate::visualizer::MODE_NAMES.len() {
        crate::visualizer::MODE_NAMES[viz_mode as usize]
    } else { "Sphere" };
    let mode_label_w = crate::graphics::scaling::measure_text_width(mode_name) as i32;
    let arrow_str = if dropdown_open { "^" } else { "v" };
    let arrow_x = ix + iw as i32 - mode_label_w - 14;
    draw_text(arrow_x, cy, mode_name, GREEN_TERTIARY);
    draw_text(arrow_x + mode_label_w + 4, cy, arrow_str, GREEN_PRIMARY);

    cy += 20;

    // ── Mini frequency bars (horizontal, 4 bands) ──
    let bar_h = 8u32;
    let bar_gap = 4u32;
    let bar_w = (iw - bar_gap * 3) / 4;
    let bands: [(f32, u32, &str); 4] = [
        (audio.sub_bass, 0xFF00FF44, "SB"),
        (audio.bass,     0xFF00CC88, "BA"),
        (audio.mid,      0xFF00AACC, "MD"),
        (audio.treble,   0xFF8866FF, "TR"),
    ];
    for (bi, &(level, color, label)) in bands.iter().enumerate() {
        let bx = ix + (bi as u32 * (bar_w + bar_gap)) as i32;
        // Track background
        framebuffer::fill_rect_alpha(bx.max(0) as u32, cy.max(0) as u32, bar_w, bar_h, 0x112211, 150);
        // Fill level
        let fill = if audio.playing {
            (level.min(1.0) * bar_w as f32) as u32
        } else { 0 };
        if fill > 0 {
            framebuffer::fill_rect(bx.max(0) as u32, cy.max(0) as u32, fill, bar_h, color);
            framebuffer::fill_rect_alpha(bx.max(0) as u32, cy.max(0) as u32, fill, bar_h, 0xFFFFFF, 15);
        }
        // Label (centered in bar)
        let lw = crate::graphics::scaling::measure_text_width(label) as i32;
        draw_text(bx + (bar_w as i32 - lw) / 2, cy - 1, label, 0xFFAABBAA);
    }
    cy += bar_h as i32 + 8;

    // ── Waveform visualizer (compact oscilloscope) ──
    let wave_h = 36u32;
    framebuffer::fill_rect_alpha(ix.max(0) as u32, cy.max(0) as u32, iw, wave_h, 0x030908, 160);
    // Border lines
    framebuffer::fill_rect_alpha(ix.max(0) as u32, cy.max(0) as u32, iw, 1, 0x00FF66, 25);
    framebuffer::fill_rect_alpha(ix.max(0) as u32, (cy + wave_h as i32 - 1).max(0) as u32, iw, 1, 0x00FF66, 15);

    let mid_y = cy + wave_h as i32 / 2;
    let half_h = (wave_h / 2 - 2) as f32;

    if audio.playing {
        // Animated waveform using frame counter + energy/beat
        let n_points = iw.min(256) as usize;
        for i in 0..n_points {
            let t = i as f32 / n_points as f32;
            let phase = audio.frame as f32 * 0.06;
            // Multi-frequency sine wave modulated by audio energy
            let s1 = libm::sinf(t * 12.0 + phase) * audio.energy;
            let s2 = libm::sinf(t * 28.0 + phase * 1.4) * audio.treble * 0.5;
            let s3 = libm::sinf(t * 5.0 + phase * 0.7) * audio.bass * 0.7;
            let beat_amp = 1.0 + audio.beat * 0.6;
            let amp = ((s1 + s2 + s3) * beat_amp).max(-1.0).min(1.0);
            let y_off = (amp * half_h) as i32;

            let px = (ix + i as i32).max(0) as u32;
            let py = ((mid_y + y_off).max(cy + 2).min(cy + wave_h as i32 - 3)) as u32;

            // Green base + beat-reactive cyan
            let g_val = 0xCC;
            let b_val = (audio.beat * 160.0).min(255.0) as u32;
            let r_val = (audio.energy * 50.0).min(50.0) as u32;
            let color = 0xFF000000 | (r_val << 16) | (g_val << 8) | b_val;
            framebuffer::put_pixel(px, py, color);
            // Bright tip
            framebuffer::put_pixel(px, py, 0xFF00FFCC);
        }
        // Beat flash
        if audio.beat > 0.4 {
            let flash = ((audio.beat - 0.4) * 40.0).min(30.0) as u32;
            framebuffer::fill_rect_alpha(ix.max(0) as u32, cy.max(0) as u32, iw, wave_h, 0x00FF88, flash);
        }
    } else {
        // Idle: flat line
        framebuffer::fill_rect((ix + 4).max(0) as u32, mid_y.max(0) as u32, iw.saturating_sub(8), 1, 0xFF334433);
        draw_text_centered(ix + iw as i32 / 2, mid_y - 6, "---", 0xFF445544);
    }
    cy += wave_h as i32 + 8;

    // ── Controls row: |<  <<  PLAY/PAUSE  >>  >| ──
    let ctrl_icons: &[&str] = &["|<", "<<", if audio.playing { "||" } else { ">" }, ">>", ">|"];
    let n_btns = ctrl_icons.len() as u32;
    let btn_w = 40u32;
    let btn_h = 20u32;
    let btn_gap = 6u32;
    let total_btn_w = n_btns * btn_w + (n_btns - 1) * btn_gap;
    let btn_start_x = ix + (iw as i32 - total_btn_w as i32) / 2;

    for (bi, &label) in ctrl_icons.iter().enumerate() {
        let bx = btn_start_x + (bi as u32 * (btn_w + btn_gap)) as i32;
        let is_play = bi == 2;
        let bg = if is_play {
            if audio.playing { 0x00AA55u32 } else { 0x005533u32 }
        } else {
            0x1A2A1Au32
        };
        framebuffer::fill_rect_alpha(bx.max(0) as u32, cy.max(0) as u32, btn_w, btn_h, bg, 190);
        // Top shine
        framebuffer::fill_rect_alpha(bx.max(0) as u32, cy.max(0) as u32, btn_w, 1, 0x00FF88, 30);
        let lw = crate::graphics::scaling::measure_text_width(label) as i32;
        let lcolor = if is_play { 0xFF00FFAA } else { CHROME_BRIGHT };
        draw_text(bx + (btn_w as i32 - lw) / 2, cy + 4, label, lcolor);
    }
    cy += btn_h as i32 + 4;

    // ── Visualizer mode dropdown panel ──
    if dropdown_open {
        let dd_x = widget_x + 8;
        let dd_w = widget_w - 16;
        let num_modes = crate::visualizer::NUM_MODES as u32;
        let dd_h = num_modes * DROPDOWN_ITEM_H + 8;
        // Glass dropdown background
        draw_glass_panel(dd_x, cy, dd_w, dd_h, 12, 230);
        let mut dy = cy + 4;
        for mi in 0..num_modes {
            let name = crate::visualizer::MODE_NAMES[mi as usize];
            let is_selected = mi as u8 == viz_mode;
            if is_selected {
                framebuffer::fill_rect_alpha((dd_x + 4).max(0) as u32, dy.max(0) as u32, dd_w - 8, DROPDOWN_ITEM_H, 0x00FF66, 30);
            }
            let item_color = if is_selected { GREEN_PRIMARY } else { TEXT_SECONDARY };
            let check = if is_selected { "> " } else { "  " };
            use alloc::format;
            let label = format!("{}{}", check, name);
            draw_text(dd_x + 12, dy + 6, &label, item_color);
            dy += DROPDOWN_ITEM_H as i32;
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// Fullscreen Mobile App Rendering
// ═══════════════════════════════════════════════════════════════
// Each app gets a complete mobile-friendly UI within the viewport.

/// Draw a fullscreen mobile app. Called from desktop.rs when view == AppFullscreen.
pub fn draw_mobile_app_content(
    vx: i32, vy: i32, vw: u32, vh: u32,
    app_idx: u32, frame: u64, audio: &MobileAudioInfo,
    state: &MobileState,
) {
    let content_y = vy + APP_BAR_H as i32;
    let content_h = vh.saturating_sub(APP_BAR_H + 20);
    // Dark app background
    framebuffer::fill_rect_alpha(vx.max(0) as u32, content_y.max(0) as u32, vw, content_h, 0x050A06, 220);

    match app_idx {
        0 => draw_app_terminal(vx, content_y, vw, content_h, frame, state),
        1 => draw_app_files(vx, content_y, vw, content_h, state),
        2 => draw_app_editor(vx, content_y, vw, content_h, frame, state),
        3 => draw_app_calculator(vx, content_y, vw, content_h, frame, state),
        4 => draw_app_network(vx, content_y, vw, content_h, frame),
        5 => draw_app_games(vx, content_y, vw, content_h, state),
        6 => draw_app_browser(vx, content_y, vw, content_h, state),
        7 => draw_app_trusted(vx, content_y, vw, content_h, frame),
        8 => draw_app_settings(vx, content_y, vw, content_h, state),
        9 => draw_app_about(vx, content_y, vw, content_h, frame),
        10 => draw_app_music(vx, content_y, vw, content_h, frame, audio),
        11 => draw_app_chess(vx, content_y, vw, content_h, state),
        _ => {
            draw_text_centered(vx + vw as i32 / 2, content_y + content_h as i32 / 2, "Unknown App", TEXT_SECONDARY);
        }
    }
}

// ── Terminal App ──
fn draw_app_terminal(vx: i32, cy: i32, vw: u32, ch: u32, frame: u64, state: &MobileState) {
    let pad = 10i32;
    let ix = vx + pad;

    // Terminal header
    framebuffer::fill_rect_alpha(vx.max(0) as u32, cy.max(0) as u32, vw, 24, 0x0A1A0A, 200);
    draw_text(ix, cy + 4, "trustos@mobile:~$", GREEN_PRIMARY);

    let line_h = 16i32;
    let mut ly = cy + 28;

    // Show initial welcome if no history
    if state.term_lines.is_empty() {
        let welcome = [
            "TrustOS v2.0 — Mobile Shell",
            "Type 'help' for available commands.",
            "",
        ];
        for line in &welcome {
            if ly + line_h > cy + ch as i32 - 40 { break; }
            let color = if line.starts_with("TrustOS") { GREEN_SECONDARY } else { TEXT_SECONDARY };
            draw_text(ix, ly, line, color);
            ly += line_h;
        }
    }

    // Show command history (scroll to bottom — show last lines that fit)
    let max_visible = ((ch as i32 - 68) / line_h).max(1) as usize;
    let start = if state.term_lines.len() > max_visible { state.term_lines.len() - max_visible } else { 0 };
    for line in &state.term_lines[start..] {
        if ly + line_h > cy + ch as i32 - 40 { break; }
        let color = if line.starts_with("$") { GREEN_PRIMARY }
                    else if line.starts_with("TrustOS") { GREEN_SECONDARY }
                    else { TEXT_SECONDARY };
        draw_text(ix, ly, line, color);
        ly += line_h;
    }

    // Input line at bottom
    let input_y = cy + ch as i32 - 36;
    framebuffer::fill_rect_alpha(vx.max(0) as u32, input_y.max(0) as u32, vw, 32, 0x0A1A0A, 200);
    let prompt = alloc::format!("$ {}", state.term_input);
    draw_text(ix, input_y + 8, &prompt, GREEN_PRIMARY);
    // Blinking cursor
    if (frame / 30) % 2 == 0 {
        let cursor_x = ix + crate::graphics::scaling::measure_text_width(&prompt) as i32 + 2;
        framebuffer::fill_rect(cursor_x.max(0) as u32, (input_y + 8).max(0) as u32, 8, 14, GREEN_PRIMARY);
    }
    // Tap hint
    draw_text_centered(vx + vw as i32 / 2, input_y - 14, "Tap here to run a command", CHROME_DIM);
}

// ── Files App ──
fn draw_app_files(vx: i32, cy: i32, vw: u32, ch: u32, state: &MobileState) {
    let pad = 10i32;
    let ix = vx + pad;

    // Path bar
    framebuffer::fill_rect_alpha(vx.max(0) as u32, cy.max(0) as u32, vw, 28, 0x0A120E, 200);
    let path_text = if state.files_depth == 0 { "/home/user/" } else { "/home/user/Documents/" };
    if state.files_depth > 0 {
        draw_text(ix, cy + 6, "< Back", GREEN_PRIMARY);
        let pw = crate::graphics::scaling::measure_text_width(path_text) as i32;
        draw_text(vx + vw as i32 - pad - pw, cy + 6, path_text, GREEN_SECONDARY);
    } else {
        draw_text(ix, cy + 6, path_text, GREEN_SECONDARY);
    }

    // Different entries based on depth
    let entries_root: [(&str, &str, u32); 8] = [
        ("Documents", "DIR", 0xFFDDAA30),
        ("Downloads", "DIR", 0xFFDDAA30),
        ("Pictures",  "DIR", 0xFFDDAA30),
        ("Music",     "DIR", 0xFF4488DD),
        ("readme.md", "4KB", TEXT_SECONDARY),
        ("config.toml","2KB", TEXT_SECONDARY),
        ("notes.txt", "1KB", TEXT_SECONDARY),
        ("photo.png", "3MB", 0xFF9060D0),
    ];
    let entries_sub: [(&str, &str, u32); 6] = [
        ("project.rs", "12KB", 0xFF6688CC),
        ("report.pdf", "2MB", 0xFFCC4444),
        ("budget.csv", "8KB", 0xFF40CC80),
        ("slides.md",  "6KB", TEXT_SECONDARY),
        ("backup.zip", "45MB", 0xFF9060D0),
        ("todo.txt",   "1KB", TEXT_SECONDARY),
    ];

    let row_h = 40u32;
    let mut ey = cy + 32;

    if state.files_depth == 0 {
        for (i, &(name, size, color)) in entries_root.iter().enumerate() {
            if ey + row_h as i32 > cy + ch as i32 { break; }
            let is_selected = state.files_selected == i as i32;
            let bg = if is_selected { 0x0A2A15 } else { 0x060A08 };
            framebuffer::fill_rect_alpha(vx.max(0) as u32, ey.max(0) as u32, vw, row_h, bg, 180);
            framebuffer::fill_rect((vx + 8).max(0) as u32, (ey + row_h as i32 - 1).max(0) as u32, vw.saturating_sub(16), 1, CHROME_GHOST);
            let icon_char = if size == "DIR" { ">" } else { "-" };
            let name_color = if is_selected { GREEN_PRIMARY } else { color };
            draw_text(ix, ey + 12, icon_char, name_color);
            draw_text(ix + 16, ey + 12, name, name_color);
            let sw = crate::graphics::scaling::measure_text_width(size) as i32;
            draw_text(vx + vw as i32 - pad - sw, ey + 12, size, CHROME_DIM);
            ey += row_h as i32;
        }
    } else {
        for (i, &(name, size, color)) in entries_sub.iter().enumerate() {
            if ey + row_h as i32 > cy + ch as i32 { break; }
            let is_selected = state.files_selected == i as i32;
            let bg = if is_selected { 0x0A2A15 } else { 0x060A08 };
            framebuffer::fill_rect_alpha(vx.max(0) as u32, ey.max(0) as u32, vw, row_h, bg, 180);
            framebuffer::fill_rect((vx + 8).max(0) as u32, (ey + row_h as i32 - 1).max(0) as u32, vw.saturating_sub(16), 1, CHROME_GHOST);
            let name_color = if is_selected { GREEN_PRIMARY } else { color };
            draw_text(ix, ey + 12, "-", name_color);
            draw_text(ix + 16, ey + 12, name, name_color);
            let sw = crate::graphics::scaling::measure_text_width(size) as i32;
            draw_text(vx + vw as i32 - pad - sw, ey + 12, size, CHROME_DIM);
            ey += row_h as i32;
        }
    }
}

// ── Editor App ──
fn draw_app_editor(vx: i32, cy: i32, vw: u32, ch: u32, frame: u64, state: &MobileState) {
    let pad = 10i32;
    let ix = vx + pad;

    // Tab bar
    framebuffer::fill_rect_alpha(vx.max(0) as u32, cy.max(0) as u32, vw, 26, 0x0A1A10, 200);
    let tab0_color = if state.editor_tab == 0 { GREEN_PRIMARY } else { CHROME_DIM };
    let tab1_color = if state.editor_tab == 1 { GREEN_PRIMARY } else { CHROME_DIM };
    draw_text(ix, cy + 6, "main.rs", tab0_color);
    draw_text(ix + 80, cy + 6, "lib.rs", tab1_color);
    // Active tab underline
    let ul_x = if state.editor_tab == 0 { ix } else { ix + 80 };
    framebuffer::fill_rect(ul_x.max(0) as u32, (cy + 24).max(0) as u32, 50, 2, GREEN_PRIMARY);

    // Different content per tab
    let code_tab0 = [
        (1, "fn main() {"),
        (2, "    let os = TrustOS::new();"),
        (3, "    os.init_hardware();"),
        (4, "    os.start_desktop();"),
        (5, ""),
        (6, "    // Mobile mode"),
        (7, "    if os.is_mobile() {"),
        (8, "        os.launch_mobile();"),
        (9, "    }"),
        (10, ""),
        (11, "    os.run_forever();"),
        (12, "}"),
    ];
    let code_tab1 = [
        (1, "pub mod kernel;"),
        (2, "pub mod desktop;"),
        (3, "pub mod mobile;"),
        (4, "pub mod audio;"),
        (5, "pub mod visualizer;"),
        (6, "pub mod network;"),
        (7, ""),
        (8, "pub fn init() {"),
        (9, "    kernel::start();"),
        (10, "}"),
        (11, ""),
        (12, ""),
    ];

    let code_lines = if state.editor_tab == 0 { &code_tab0 } else { &code_tab1 };

    let line_h = 16i32;
    let mut ly = cy + 30;
    for &(num, line) in code_lines.iter() {
        if ly + line_h > cy + ch as i32 { break; }
        use alloc::format;
        let num_str = format!("{:3}", num);
        // Highlight current line
        let is_current = (num - 1) as u32 == state.editor_cursor_line;
        if is_current {
            framebuffer::fill_rect_alpha(vx.max(0) as u32, ly.max(0) as u32, vw, line_h as u32, 0x1A2A1A, 120);
        }
        draw_text(ix, ly, &num_str, if is_current { GREEN_PRIMARY } else { CHROME_DIM });
        // Syntax coloring
        let color = if line.contains("fn ") { 0xFF6688CC }
            else if line.contains("let ") || line.contains("pub ") { 0xFF8866FF }
            else if line.contains("//") { 0xFF556655 }
            else if line.contains("TrustOS") || line.contains("mod ") { GREEN_PRIMARY }
            else { TEXT_SECONDARY };
        draw_text(ix + 30, ly, line, color);
        ly += line_h;
    }
    // Cursor on current line
    if (frame / 30) % 2 == 0 {
        let cursor_y = cy + 30 + state.editor_cursor_line as i32 * line_h;
        if cursor_y >= cy + 30 && cursor_y < cy + ch as i32 {
            framebuffer::fill_rect((ix + 30).max(0) as u32, cursor_y.max(0) as u32, 2, 14, GREEN_PRIMARY);
        }
    }
}

// ── Calculator App ──
fn draw_app_calculator(vx: i32, cy: i32, vw: u32, ch: u32, _frame: u64, state: &MobileState) {
    let pad = 14i32;
    let ix = vx + pad;
    let iw = (vw as i32 - pad * 2) as u32;

    // Display
    let display_h = 60u32;
    framebuffer::fill_rect_alpha(ix.max(0) as u32, (cy + 10).max(0) as u32, iw, display_h, 0x0A1A10, 220);
    draw_rounded_rect_border(ix, cy + 10, iw, display_h, 8, CHROME_GHOST);
    let disp_text = if state.calc_display.is_empty() { "0" } else { &state.calc_display };
    let tw = crate::graphics::scaling::measure_text_width(disp_text) as i32;
    draw_text(ix + iw as i32 - tw - 10, cy + 30, disp_text, GREEN_PRIMARY);

    // Button grid: 4 columns x 5 rows
    let btn_labels = [
        ["C", "+/-", "%", "/"],
        ["7", "8", "9", "x"],
        ["4", "5", "6", "-"],
        ["1", "2", "3", "+"],
        ["0", ".", "=", ""],
    ];
    let btn_h = 44u32;
    let btn_gap = 6u32;
    let btn_w = (iw - btn_gap * 3) / 4;
    let mut by = cy + 10 + display_h as i32 + 14;

    for row in &btn_labels {
        let mut bx = ix;
        for &label in row {
            if label.is_empty() { bx += (btn_w + btn_gap) as i32; continue; }
            let is_op = matches!(label, "/" | "x" | "-" | "+" | "=");
            let is_func = matches!(label, "C" | "+/-" | "%");
            let bg = if is_op { 0xFF008844u32 }
                     else if is_func { 0xFF333833 }
                     else { 0xFF1A221A };
            draw_rounded_rect(bx, by, btn_w, btn_h, 8, bg);
            draw_rounded_rect_border(bx, by, btn_w, btn_h, 8, CHROME_GHOST);
            let lcolor = if is_op { GREEN_PRIMARY } else { TEXT_PRIMARY };
            draw_text_centered(bx + btn_w as i32 / 2, by + 14, label, lcolor);
            bx += (btn_w + btn_gap) as i32;
        }
        by += (btn_h + btn_gap) as i32;
    }
}

// ── Network App ──
fn draw_app_network(vx: i32, cy: i32, vw: u32, ch: u32, frame: u64) {
    let pad = 12i32;
    let ix = vx + pad;
    let iw = vw as i32 - pad * 2;

    let mut ly = cy + 10;
    let section_h = 28i32;

    // WiFi section
    draw_text(ix, ly, "WiFi", GREEN_PRIMARY);
    draw_text(ix + iw - 24, ly, "ON", GREEN_SECONDARY);
    ly += section_h;
    framebuffer::fill_rect_alpha(ix.max(0) as u32, ly.max(0) as u32, iw as u32, 40, 0x0A120E, 180);
    draw_text(ix + 8, ly + 12, "TrustNet-5G", TEXT_PRIMARY);
    draw_text(ix + iw - 80, ly + 12, "Connected", GREEN_SECONDARY);
    ly += 48;

    // IP Info
    draw_text(ix, ly, "Network Info", GREEN_PRIMARY);
    ly += 22;
    let info = [
        ("IP Address:", "192.168.1.42"),
        ("Subnet:", "255.255.255.0"),
        ("Gateway:", "192.168.1.1"),
        ("DNS:", "8.8.8.8"),
        ("MAC:", "AA:BB:CC:DD:EE:FF"),
    ];
    for &(label, value) in &info {
        if ly + 18 > cy + ch as i32 { break; }
        draw_text(ix + 8, ly, label, TEXT_SECONDARY);
        let vw2 = crate::graphics::scaling::measure_text_width(value) as i32;
        draw_text(ix + iw - vw2 - 8, ly, value, TEXT_PRIMARY);
        ly += 20;
    }
    ly += 10;

    // Signal strength bar
    draw_text(ix, ly, "Signal", GREEN_PRIMARY);
    ly += 20;
    let bar_w = (iw - 16) as u32;
    framebuffer::fill_rect((ix + 8).max(0) as u32, ly.max(0) as u32, bar_w, 8, CHROME_GHOST);
    let signal = ((frame % 100) as u32 * bar_w / 100).max(bar_w * 7 / 10);
    framebuffer::fill_rect((ix + 8).max(0) as u32, ly.max(0) as u32, signal, 8, GREEN_SECONDARY);
}

// ── Games App ──
fn draw_app_games(vx: i32, cy: i32, vw: u32, ch: u32, state: &MobileState) {
    let pad = 12i32;
    let ix = vx + pad;
    let iw = (vw as i32 - pad * 2) as u32;

    draw_text(ix, cy + 10, "Games Library", GREEN_PRIMARY);

    let games = [
        ("Snake", "Classic arcade", 0xFF44DD44),
        ("Chess", "Strategy board game", 0xFFD4A854),
        ("3D FPS", "Raycasting demo", 0xFF4488DD),
        ("GameBoy", "GB emulator", 0xFF9060D0),
        ("NES", "NES emulator", 0xFFCC4444),
    ];

    let card_h = 56u32;
    let card_gap = 8u32;
    let mut gy = cy + 34;

    for (i, &(name, desc, accent)) in games.iter().enumerate() {
        if gy + card_h as i32 > cy + ch as i32 { break; }
        let is_selected = state.games_selected == i as i32;
        let bg = if is_selected { 0xFF0C1610 } else { 0xFF080C0A };
        let border = if is_selected { GREEN_PRIMARY } else { CHROME_GHOST };
        draw_rounded_rect(ix, gy, iw, card_h, 10, bg);
        draw_rounded_rect_border(ix, gy, iw, card_h, 10, border);
        // Accent bar left
        framebuffer::fill_rect(ix.max(0) as u32, (gy + 8).max(0) as u32, 3, card_h - 16, accent);
        // Title
        draw_text(ix + 14, gy + 10, name, accent);
        // Description
        draw_text(ix + 14, gy + 28, desc, TEXT_SECONDARY);
        // Play/selected indicator
        let btn_text = if is_selected { ">>>" } else { ">" };
        let btn_color = if is_selected { GREEN_PRIMARY } else { CHROME_DIM };
        draw_text(vx + vw as i32 - pad - 30, gy + 16, btn_text, btn_color);
        gy += (card_h + card_gap) as i32;
    }
}

// ── Browser App ──
fn draw_app_browser(vx: i32, cy: i32, vw: u32, ch: u32, state: &MobileState) {
    let pad = 8i32;
    let ix = vx + pad;
    let iw = (vw as i32 - pad * 2) as u32;

    // URL bar
    let url_h = 30u32;
    draw_rounded_rect(ix, cy + 4, iw, url_h, 10, 0xFF0A120E);
    draw_rounded_rect_border(ix, cy + 4, iw, url_h, 10, CHROME_GHOST);
    let url = match state.browser_page {
        0 => "https://trustos.local",
        1 => "https://trustos.local/docs",
        2 => "https://trustos.local/source",
        3 => "https://trustos.local/downloads",
        _ => "https://trustos.local",
    };
    draw_text(ix + 10, cy + 12, url, TEXT_SECONDARY);

    // Page content
    let page_y = cy + 40;
    framebuffer::fill_rect_alpha(vx.max(0) as u32, page_y.max(0) as u32, vw, ch - 40, 0x0C140E, 200);

    let mut ly = page_y + 10;
    match state.browser_page {
        0 => {
            draw_text(ix + 4, ly, "Welcome to TrustOS", GREEN_PRIMARY); ly += 24;
            draw_text(ix + 4, ly, "A secure, minimal operating system", TEXT_SECONDARY); ly += 20;
            draw_text(ix + 4, ly, "built with Rust.", TEXT_SECONDARY); ly += 30;
            draw_text(ix + 4, ly, "> Documentation", 0xFF4488DD); ly += 20;
            draw_text(ix + 4, ly, "> Source Code", 0xFF4488DD); ly += 20;
            draw_text(ix + 4, ly, "> Downloads", 0xFF4488DD);
        }
        1 => {
            draw_text(ix + 4, ly, "Documentation", GREEN_PRIMARY); ly += 24;
            draw_text(ix + 4, ly, "1. Getting Started", 0xFF4488DD); ly += 20;
            draw_text(ix + 4, ly, "   Install TrustOS on bare metal", TEXT_SECONDARY); ly += 16;
            draw_text(ix + 4, ly, "   or run in QEMU/VirtualBox.", TEXT_SECONDARY); ly += 24;
            draw_text(ix + 4, ly, "2. Mobile Mode", 0xFF4488DD); ly += 20;
            draw_text(ix + 4, ly, "   Portrait UI for small screens.", TEXT_SECONDARY); ly += 24;
            draw_text(ix + 4, ly, "3. Desktop Mode", 0xFF4488DD); ly += 20;
            draw_text(ix + 4, ly, "   Full windowed environment.", TEXT_SECONDARY); ly += 24;
            draw_text(ix + 4, ly, "4. Audio System", 0xFF4488DD); ly += 20;
            draw_text(ix + 4, ly, "   HD Audio with DMA.", TEXT_SECONDARY); ly += 30;
            draw_text(ix + 4, ly, "< Back to Home", GREEN_SECONDARY);
        }
        2 => {
            draw_text(ix + 4, ly, "Source Code", GREEN_PRIMARY); ly += 24;
            draw_text(ix + 4, ly, "Repository:", TEXT_SECONDARY); ly += 20;
            draw_text(ix + 4, ly, "  github.com/trustos/kernel", 0xFF4488DD); ly += 24;
            draw_text(ix + 4, ly, "Language: Rust (no_std)", TEXT_SECONDARY); ly += 20;
            draw_text(ix + 4, ly, "LOC: ~25,000", TEXT_SECONDARY); ly += 20;
            draw_text(ix + 4, ly, "License: MIT", TEXT_SECONDARY); ly += 30;
            draw_text(ix + 4, ly, "< Back to Home", GREEN_SECONDARY);
        }
        3 => {
            draw_text(ix + 4, ly, "Downloads", GREEN_PRIMARY); ly += 24;
            draw_text(ix + 4, ly, "TrustOS v2.0 ISO", 0xFF4488DD); ly += 20;
            draw_text(ix + 4, ly, "  Size: 12 MB | x86_64", TEXT_SECONDARY); ly += 24;
            draw_text(ix + 4, ly, "TrustOS v2.0 aarch64", 0xFF4488DD); ly += 20;
            draw_text(ix + 4, ly, "  Size: 14 MB | ARM64", TEXT_SECONDARY); ly += 24;
            draw_text(ix + 4, ly, "VBox Appliance (.ova)", 0xFF4488DD); ly += 20;
            draw_text(ix + 4, ly, "  Size: 50 MB | Pre-configured", TEXT_SECONDARY); ly += 30;
            draw_text(ix + 4, ly, "< Back to Home", GREEN_SECONDARY);
        }
        _ => {
            draw_text(ix + 4, ly, "Page not found", TEXT_SECONDARY);
        }
    }

    // Footer
    ly = cy + ch as i32 - 18;
    framebuffer::fill_rect((ix + 4).max(0) as u32, (ly - 4).max(0) as u32, iw - 8, 1, CHROME_GHOST);
    draw_text(ix + 4, ly, "TrustOS Browser v1.0", CHROME_DIM);
}

// ── TrustEd (3D Model Editor) App ──
fn draw_app_trusted(vx: i32, cy: i32, vw: u32, ch: u32, frame: u64) {
    let pad = 10i32;
    let ix = vx + pad;
    let iw = (vw as i32 - pad * 2) as u32;

    // Toolbar
    framebuffer::fill_rect_alpha(vx.max(0) as u32, cy.max(0) as u32, vw, 28, 0x0A120E, 200);
    let tools = ["Move", "Rot", "Scale", "Add"];
    let mut tx = ix;
    for tool in &tools {
        draw_text(tx, cy + 7, tool, CHROME_MID);
        tx += 50;
    }

    // 3D viewport (wireframe cube)
    let view_y = cy + 32;
    let view_h = ch.saturating_sub(60);
    framebuffer::fill_rect_alpha(vx.max(0) as u32, view_y.max(0) as u32, vw, view_h, 0x030806, 220);

    // Grid lines
    let cx_3d = vx + vw as i32 / 2;
    let cy_3d = view_y + view_h as i32 / 2;
    for i in 0..8u32 {
        let offset = (i as i32 - 4) * 20;
        framebuffer::fill_rect_alpha(vx.max(0) as u32, (cy_3d + offset).max(0) as u32, vw, 1, 0x002A15, 40);
        framebuffer::fill_rect_alpha((cx_3d + offset).max(0) as u32, view_y.max(0) as u32, 1, view_h, 0x002A15, 40);
    }

    // Rotating wireframe cube
    let t = frame as f32 * 0.03;
    let sin_t = libm::sinf(t);
    let cos_t = libm::cosf(t);
    let s = 40.0f32;
    let cube_pts: [(f32, f32, f32); 8] = [
        (-s, -s, -s), (s, -s, -s), (s, s, -s), (-s, s, -s),
        (-s, -s,  s), (s, -s,  s), (s, s,  s), (-s, s,  s),
    ];
    let edges: [(usize, usize); 12] = [
        (0,1),(1,2),(2,3),(3,0), (4,5),(5,6),(6,7),(7,4),
        (0,4),(1,5),(2,6),(3,7),
    ];
    let project = |p: (f32, f32, f32)| -> (i32, i32) {
        let rx = p.0 * cos_t - p.2 * sin_t;
        let rz = p.0 * sin_t + p.2 * cos_t;
        let ry = p.1 * libm::cosf(t * 0.7) - rz * libm::sinf(t * 0.7);
        (cx_3d + rx as i32, cy_3d + ry as i32)
    };
    for &(a, b) in &edges {
        let (x1, y1) = project(cube_pts[a]);
        let (x2, y2) = project(cube_pts[b]);
        draw_line(x1, y1, x2, y2, GREEN_SECONDARY);
    }

    // Bottom info bar
    let info_y = view_y + view_h as i32 - 20;
    draw_text(ix, info_y, "Vertices: 8  Faces: 6  Edges: 12", CHROME_DIM);
}

// ── Settings App ──
fn draw_app_settings(vx: i32, cy: i32, vw: u32, ch: u32, state: &MobileState) {
    let pad = 12i32;
    let ix = vx + pad;
    let iw = (vw as i32 - pad * 2) as u32;

    let settings = [
        ("WiFi", "Wireless connection", 0xFF40CC80),
        ("Bluetooth", "Paired devices", 0xFF4488DD),
        ("Airplane", "Radio off", 0xFFCC8844),
        ("Do Not Disturb", "Silence alerts", 0xFFFF6090),
        ("Dark Mode", "Display theme", 0xFF9988BB),
        ("Notifications", "Push alerts", 0xFF40AADD),
    ];

    let row_h = 52u32;
    let mut ry = cy + 10;
    for (i, &(title, desc, accent)) in settings.iter().enumerate() {
        if ry + row_h as i32 > cy + ch as i32 { break; }
        let is_selected = state.settings_selected == i as i32;
        let bg = if is_selected { 0x0A1A12 } else { 0x080C0A };
        framebuffer::fill_rect_alpha(ix.max(0) as u32, ry.max(0) as u32, iw, row_h, bg, 180);
        framebuffer::fill_rect((ix + 4).max(0) as u32, (ry + row_h as i32 - 1).max(0) as u32, iw - 8, 1, CHROME_GHOST);
        // Accent dot
        framebuffer::fill_rect((ix + 8).max(0) as u32, (ry + 20).max(0) as u32, 4, 4, accent);
        draw_text(ix + 20, ry + 10, title, TEXT_PRIMARY);
        draw_text(ix + 20, ry + 28, desc, TEXT_SECONDARY);
        // Toggle switch on right
        let toggle_on = i < state.settings_toggles.len() && state.settings_toggles[i];
        let toggle_x = vx + vw as i32 - pad - 44;
        let toggle_bg = if toggle_on { 0xFF008844 } else { 0xFF333833 };
        draw_rounded_rect(toggle_x, ry + 14, 40, 22, 11, toggle_bg);
        // Toggle knob
        let knob_x = if toggle_on { toggle_x + 20 } else { toggle_x + 2 };
        draw_rounded_rect(knob_x, ry + 16, 18, 18, 9, if toggle_on { GREEN_PRIMARY } else { CHROME_DIM });
        ry += row_h as i32;
    }
}

// ── About App ──
fn draw_app_about(vx: i32, cy: i32, vw: u32, ch: u32, _frame: u64) {
    let pad = 14i32;
    let ix = vx + pad;

    let center_x = vx + vw as i32 / 2;
    let mut ly = cy + 20;

    // Logo/title
    draw_text_centered(center_x, ly, "TrustOS", GREEN_PRIMARY);
    ly += 24;
    draw_text_centered(center_x, ly, "v2.0.0", GREEN_SECONDARY);
    ly += 30;

    // Divider
    framebuffer::fill_rect((ix + 20).max(0) as u32, ly.max(0) as u32, vw.saturating_sub(68), 1, CHROME_GHOST);
    ly += 16;

    let info = [
        ("Kernel:", "TrustOS Microkernel"),
        ("Arch:", "x86_64 / aarch64"),
        ("License:", "MIT"),
        ("Desktop:", "TrustOS Desktop v2"),
        ("Browser:", "TrustBrowser v1.0"),
        ("Audio:", "HD Audio + DMA"),
        ("Graphics:", "COSMIC Renderer"),
        ("Uptime:", "4h 23m"),
    ];

    for &(label, value) in &info {
        if ly + 20 > cy + ch as i32 { break; }
        draw_text(ix, ly, label, TEXT_SECONDARY);
        let vw2 = crate::graphics::scaling::measure_text_width(value) as i32;
        draw_text(vx + vw as i32 - pad - vw2, ly, value, TEXT_PRIMARY);
        ly += 22;
    }
}

// ── Music App (fullscreen) ──
fn draw_app_music(vx: i32, cy: i32, vw: u32, ch: u32, frame: u64, audio: &MobileAudioInfo) {
    let pad = 14i32;
    let ix = vx + pad;
    let iw = (vw as i32 - pad * 2) as u32;

    // Album art placeholder
    let art_size = iw.min(200);
    let art_x = vx + (vw as i32 - art_size as i32) / 2;
    let mut ly = cy + 14;
    draw_rounded_rect(art_x, ly, art_size, art_size, 14, 0xFF0A1A10);
    draw_rounded_rect_border(art_x, ly, art_size, art_size, 14, CHROME_GHOST);

    // Animated visualizer inside album art
    if audio.playing {
        let mid_x = art_x + art_size as i32 / 2;
        let mid_y = ly + art_size as i32 / 2;
        let n_bars = 16u32;
        let bar_w = art_size / (n_bars * 2);
        for i in 0..n_bars {
            let t = i as f32 / n_bars as f32;
            let phase = frame as f32 * 0.08 + t * 6.28;
            let amp = (libm::sinf(phase) * audio.energy + audio.bass * 0.5).max(0.1).min(1.0);
            let h = (amp * (art_size as f32 * 0.4)) as u32;
            let bx = art_x + 10 + (i * (bar_w * 2)) as i32;
            let by = ly + art_size as i32 / 2 + (art_size as i32 / 4 - h as i32).max(0);
            let g = (128.0 + amp * 127.0).min(255.0) as u32;
            framebuffer::fill_rect(bx.max(0) as u32, by.max(0) as u32, bar_w, h, 0xFF000000 | (g << 8) | 0x40);
        }
    } else {
        draw_text_centered(art_x + art_size as i32 / 2, ly + art_size as i32 / 2 - 6, "No Track", TEXT_SECONDARY);
    }
    ly += art_size as i32 + 16;

    // Track info
    let title = if audio.playing { "Untitled (2) - Lo-Fi" } else { "No Track Playing" };
    draw_text_centered(vx + vw as i32 / 2, ly, title, TEXT_PRIMARY);
    ly += 20;
    draw_text_centered(vx + vw as i32 / 2, ly, "TrustOS Audio", TEXT_SECONDARY);
    ly += 28;

    // Progress bar
    let bar_w = iw - 20;
    framebuffer::fill_rect((ix + 10).max(0) as u32, ly.max(0) as u32, bar_w, 4, CHROME_GHOST);
    if audio.playing {
        let progress = (frame % 300) as u32 * bar_w / 300;
        framebuffer::fill_rect((ix + 10).max(0) as u32, ly.max(0) as u32, progress, 4, GREEN_PRIMARY);
    }
    ly += 20;

    // Large control buttons
    let ctrl_labels = ["|<", "<<", if audio.playing { "||" } else { ">" }, ">>", ">|"];
    let btn_w = 48u32;
    let btn_h = 36u32;
    let btn_gap = 10u32;
    let total_w = 5 * btn_w + 4 * btn_gap;
    let btn_start = ix + (iw as i32 - total_w as i32) / 2;
    for (i, &label) in ctrl_labels.iter().enumerate() {
        let bx = btn_start + (i as u32 * (btn_w + btn_gap)) as i32;
        let is_play = i == 2;
        let bg = if is_play { if audio.playing { 0xFF005533 } else { 0xFF003322 } } else { 0xFF1A2A1A };
        draw_rounded_rect(bx, ly, btn_w, btn_h, 10, bg);
        let lcolor = if is_play { GREEN_PRIMARY } else { CHROME_BRIGHT };
        draw_text_centered(bx + btn_w as i32 / 2, ly + 10, label, lcolor);
    }
    ly += btn_h as i32 + 16;

    // Frequency bands
    let band_names = ["Sub", "Bass", "Mid", "Treble"];
    let band_vals = [audio.sub_bass, audio.bass, audio.mid, audio.treble];
    let band_colors: [u32; 4] = [0xFF00FF44, 0xFF00CC88, 0xFF00AACC, 0xFF8866FF];
    let band_w = (iw - 12) / 4;
    for (i, (&name, &val)) in band_names.iter().zip(band_vals.iter()).enumerate() {
        let bx = ix + (i as u32 * (band_w + 4)) as i32;
        framebuffer::fill_rect_alpha(bx.max(0) as u32, ly.max(0) as u32, band_w, 10, 0x112211, 150);
        let fill = if audio.playing { (val.min(1.0) * band_w as f32) as u32 } else { 0 };
        if fill > 0 {
            framebuffer::fill_rect(bx.max(0) as u32, ly.max(0) as u32, fill, 10, band_colors[i]);
        }
        draw_text_centered(bx + band_w as i32 / 2, ly + 12, name, CHROME_DIM);
    }
}

// ── Chess App ──
fn draw_app_chess(vx: i32, cy: i32, vw: u32, ch: u32, state: &MobileState) {
    let pad = 8i32;

    // Board sizing: fit board in viewport width
    let board_size = (vw as i32 - pad * 2).min(ch as i32 - 60).min(400);
    let cell = board_size / 8;
    let board_x = vx + (vw as i32 - board_size) / 2;
    let board_y = cy + 10;

    let selected_row = if state.chess_selected >= 0 { state.chess_selected / 8 } else { -1 };
    let selected_col = if state.chess_selected >= 0 { state.chess_selected % 8 } else { -1 };

    // Draw board
    for row in 0..8u32 {
        for col in 0..8u32 {
            let is_light = (row + col) % 2 == 0;
            let is_selected = row as i32 == selected_row && col as i32 == selected_col;
            let color = if is_selected { 0xFF2A5A2A }
                       else if is_light { 0xFF2A3A2A }
                       else { 0xFF0A140A };
            let cx = board_x + (col * cell as u32) as i32;
            let ry = board_y + (row * cell as u32) as i32;
            framebuffer::fill_rect(cx.max(0) as u32, ry.max(0) as u32, cell as u32, cell as u32, color);
            // Highlight border for selected square
            if is_selected {
                // Top/bottom borders
                framebuffer::fill_rect(cx.max(0) as u32, ry.max(0) as u32, cell as u32, 2, GREEN_PRIMARY);
                framebuffer::fill_rect(cx.max(0) as u32, (ry + cell - 2).max(0) as u32, cell as u32, 2, GREEN_PRIMARY);
                // Left/right borders
                framebuffer::fill_rect(cx.max(0) as u32, ry.max(0) as u32, 2, cell as u32, GREEN_PRIMARY);
                framebuffer::fill_rect((cx + cell - 2).max(0) as u32, ry.max(0) as u32, 2, cell as u32, GREEN_PRIMARY);
            }
        }
    }

    // Board border
    draw_rounded_rect_border(board_x - 1, board_y - 1, board_size as u32 + 2, board_size as u32 + 2, 2, CHROME_DIM);

    // Place pieces (starting position simplified)
    let pieces_row = ["R", "N", "B", "Q", "K", "B", "N", "R"];
    for col in 0..8 {
        let cx = board_x + col * cell + cell / 2;
        // White pieces (bottom)
        draw_text_centered(cx, board_y + 7 * cell + cell / 2 - 6, pieces_row[col as usize], 0xFFDDDDDD);
        draw_text_centered(cx, board_y + 6 * cell + cell / 2 - 6, "P", 0xFFDDDDDD);
        // Black pieces (top)
        draw_text_centered(cx, board_y + cell / 2 - 6, pieces_row[col as usize], 0xFFD4A854);
        draw_text_centered(cx, board_y + cell + cell / 2 - 6, "P", 0xFFD4A854);
    }

    // Status bar below board
    let status_y = board_y + board_size + 8;
    let turn_text = if state.chess_turn == 0 { "White to move" } else { "Black to move" };
    let turn_color = if state.chess_turn == 0 { 0xFFDDDDDD } else { 0xFFD4A854 };
    draw_text_centered(vx + vw as i32 / 2, status_y, turn_text, turn_color);

    if state.chess_selected >= 0 {
        let sq_text = alloc::format!("Selected: {}{}", 
            (b'a' + (state.chess_selected % 8) as u8) as char,
            8 - state.chess_selected / 8);
        draw_text_centered(vx + vw as i32 / 2, status_y + 18, &sq_text, GREEN_SECONDARY);
    }
}

// ═══════════════════════════════════════════════════════════════
// Bresenham Line Drawing (for wireframe)
// ═══════════════════════════════════════════════════════════════

fn draw_line(x0: i32, y0: i32, x1: i32, y1: i32, color: u32) {
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    let mut cx = x0;
    let mut cy = y0;
    loop {
        put_px(cx, cy, color);
        if cx == x1 && cy == y1 { break; }
        let e2 = 2 * err;
        if e2 >= dy {
            if cx == x1 { break; }
            err += dy;
            cx += sx;
        }
        if e2 <= dx {
            if cy == y1 { break; }
            err += dx;
            cy += sy;
        }
    }
}
