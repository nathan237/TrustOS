//! Demo / Showcase Mode — 60-second cinematic narrator overlay for TrustLab
//!
//! Real-time based (microseconds via TSC, not frame ticks).
//! Giant floating text (scale 3) with drop shadow — NO visible box.
//! Glitch/matrix transition between slides.
//! Text positions itself over the panel being described.
//!
//! Triggered by typing "demo" in the TrustLab shell bar.
//! Controls: [Esc] stop, [Space] skip to next slide.

extern crate alloc;

use alloc::string::String;
use alloc::format;

// ── Timing ─────────────────────────────────────────────────────────────────

/// Get real time in milliseconds (PIT-based, reliable in VBox)
#[inline]
fn now_mouse() -> u64 {
    crate::time::uptime_mouse()
}

// ── Slide positioning ──────────────────────────────────────────────────────

#[derive(Clone, Copy)]
enum Pos {
    Center,
    Panel(usize),   // over panel index 0-6
    BlackScreen,    // full black overlay (intro/outro)
}

// ── Slide definition ───────────────────────────────────────────────────────

struct Slide {
    lines: &'static [&'static str],
    highlights: &'static [&'static str],
    position: Pos,
    focus: Option<usize>,
    /// Duration in MILLISECONDS (real time)
    dur_mouse: u64,
    /// true = scale 3 (giant), false = scale 2
    big: bool,
}

// ── 60-second script (real milliseconds) ───────────────────────────────────
// Total: 60000ms = 60s

const TITLE_BAR_H: u32 = 28;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const SHELL_H: u32 = 28;

// Compile-time constant — evaluated at compilation, zero runtime cost.
const GLITCH_MOUSE: u64 = 350; // glitch transition duration between slides

const SCRIPT: &[Slide] = &[
    // === MORPHEUS INTRO — black screen (0s-3s) ===
    Slide { lines: &[""],
        highlights: &[], position: Pos::BlackScreen, focus: None,
        dur_mouse: 800, big: true },
    Slide { lines: &["Are you ready",  "to see the Matrix, Neo?"],
        highlights: &["Matrix", "Neo"], position: Pos::BlackScreen, focus: None,
        dur_mouse: 2200, big: true },

    // === HOOK (5s-9s) ===
    Slide { lines: &["You don't understand", "how your computer works."],
        highlights: &["don't", "computer"], position: Pos::Center, focus: None,
        dur_mouse: 2000, big: true },
    Slide { lines: &["This is TrustLab."],
        highlights: &["TrustLab"], position: Pos::Center, focus: None,
        dur_mouse: 1500, big: true },

    // === HARDWARE (9s-13s) ===
    Slide { lines: &["HARDWARE STATUS"],
        highlights: &["HARDWARE"], position: Pos::Panel(0), focus: Some(0),
        dur_mouse: 500, big: true },
    Slide { lines: &["Real CPU. Real memory.", "Raw silicon."],
        highlights: &["CPU", "memory", "Raw"], position: Pos::Panel(0), focus: Some(0),
        dur_mouse: 1500, big: true },
    Slide { lines: &["What Task Manager", "will never show you."],
        highlights: &["never"], position: Pos::Panel(0), focus: Some(0),
        dur_mouse: 1200, big: true },

    // === KERNEL TRACE (13s-17s) ===
    Slide { lines: &["LIVE KERNEL TRACE"],
        highlights: &["KERNEL", "TRACE"], position: Pos::Panel(1), focus: Some(1),
        dur_mouse: 500, big: true },
    Slide { lines: &["Every interrupt.", "Every syscall."],
        highlights: &["interrupt", "syscall"], position: Pos::Panel(1), focus: Some(1),
        dur_mouse: 1500, big: true },
    Slide { lines: &["Raw kernel truth."],
        highlights: &["Raw", "truth"], position: Pos::Panel(1), focus: Some(1),
        dur_mouse: 1200, big: true },

    // === PIPELINE (17s-20s) ===
    Slide { lines: &["EXECUTION PIPELINE"],
        highlights: &["PIPELINE"], position: Pos::Panel(5), focus: Some(5),
        dur_mouse: 500, big: true },
    Slide { lines: &["Watch data flow through", "the kernel in real time."],
        highlights: &["flow", "real time"], position: Pos::Panel(5), focus: Some(5),
        dur_mouse: 1500, big: true },

    // === HEX EDITOR (20s-23s) ===
    Slide { lines: &["HEX EDITOR"],
        highlights: &["HEX"], position: Pos::Panel(6), focus: Some(6),
        dur_mouse: 500, big: true },
    Slide { lines: &["Raw bytes. Color-coded."],
        highlights: &["Raw", "bytes"], position: Pos::Panel(6), focus: Some(6),
        dur_mouse: 1300, big: true },

    // === FILE TREE (23s-25s) ===
    Slide { lines: &["FILE SYSTEM"],
        highlights: &["FILE"], position: Pos::Panel(3), focus: Some(3),
        dur_mouse: 500, big: true },
    Slide { lines: &["Live filesystem. In memory."],
        highlights: &["Live", "memory"], position: Pos::Panel(3), focus: Some(3),
        dur_mouse: 1200, big: true },

    // === EDITOR (25s-28s) ===
    Slide { lines: &["TRUSTLANG EDITOR"],
        highlights: &["TRUSTLANG"], position: Pos::Panel(4), focus: Some(4),
        dur_mouse: 500, big: true },
    Slide { lines: &["Write code inside the kernel.", "Execute it."],
        highlights: &["code", "kernel", "Execute"], position: Pos::Panel(4), focus: Some(4),
        dur_mouse: 1500, big: true },

    // === COMMAND GUIDE (28s-30s) ===
    Slide { lines: &["52 COMMANDS"],
        highlights: &["52"], position: Pos::Panel(2), focus: Some(2),
        dur_mouse: 500, big: true },
    Slide { lines: &["Full shell. All built-in."],
        highlights: &["shell", "built-in"], position: Pos::Panel(2), focus: Some(2),
        dur_mouse: 1200, big: true },

    // === CLOSE (30s-35s) ===
    Slide { lines: &["TrustLab is not a tool."],
        highlights: &["not"], position: Pos::Center, focus: None,
        dur_mouse: 1500, big: true },
    Slide { lines: &["Bare metal. Rust. Open source."],
        highlights: &["Rust", "Open source"], position: Pos::Center, focus: None,
        dur_mouse: 1500, big: true },
    Slide { lines: &["Boot it. Break it.", "Understand it."],
        highlights: &["Boot", "Break", "Understand"], position: Pos::Center, focus: None,
        dur_mouse: 2000, big: true },
];

// ── State ──────────────────────────────────────────────────────────────────

pub struct DemoState {
    pub active: bool,
    pub current_slide: usize,
    /// Millisecond timestamp when current slide started
    slide_start_mouse: u64,
    /// Millisecond timestamp when demo started
    demo_start_mouse: u64,
    /// Pseudo-random seed (for glitch)
    seed: u32,
    /// Last slide index (to detect transitions)
    last_slide: usize,
    /// Frame counter for glitch seeding
    pub tick_in_slide: u64,
    pub total_ticks: u64,
}

// Implementation block — defines methods for the type above.
impl DemoState {
        // Public function — callable from other modules.
pub fn new() -> Self {
        Self {
            active: false,
            current_slide: 0,
            slide_start_mouse: 0,
            demo_start_mouse: 0,
            seed: 12345,
            last_slide: usize::MAX,
            tick_in_slide: 0,
            total_ticks: 0,
        }
    }

        // Public function — callable from other modules.
pub fn start(&mut self) {
        self.active = true;
        self.current_slide = 0;
        self.demo_start_mouse = now_mouse();
        self.slide_start_mouse = self.demo_start_mouse;
        self.seed = (self.demo_start_mouse & 0xFFFF) as u32 ^ 0xA5A5;
        self.last_slide = usize::MAX;
        self.tick_in_slide = 0;
        self.total_ticks = 0;
        let total_mouse: u64 = SCRIPT.iter().map(|s| s.dur_mouse).sum();
        crate::serial_println!("[DEMO] Started! now_ms={} total_script={}ms slides={}",
            self.demo_start_mouse, total_mouse, SCRIPT.len());
    }

        // Public function — callable from other modules.
pub fn stop(&mut self) {
        self.active = false;
    }

    /// Advance. Returns panel index to auto-focus (if any).
    pub fn tick(&mut self) -> Option<usize> {
        if !self.active { return None; }
        self.total_ticks += 1;

        let t = now_mouse();
        let total_elapsed_mouse = t - self.demo_start_mouse;

        if self.current_slide >= SCRIPT.len() {
            self.stop();
            return None;
        }

        // Compute cumulative deadline for current slide
        let mut cumulative_mouse: u64 = 0;
        for i in 0..=self.current_slide {
            if i < SCRIPT.len() {
                cumulative_mouse += SCRIPT[i].dur_mouse;
            }
        }

        // Debug every 200 ticks
        if self.total_ticks % 200 == 0 {
            crate::serial_println!("[DEMO] slide={} total={}ms deadline={}ms ticks={}",
                self.current_slide, total_elapsed_mouse, cumulative_mouse, self.total_ticks);
        }

        if total_elapsed_mouse >= cumulative_mouse {
            // Advance to next slide
            crate::serial_println!("[DEMO] -> next slide {} at total={}ms (deadline={}ms)",
                self.current_slide + 1, total_elapsed_mouse, cumulative_mouse);
            self.current_slide += 1;
            self.slide_start_mouse = t;
            self.tick_in_slide = 0;
            if self.current_slide >= SCRIPT.len() {
                self.stop();
                return None;
            }
            return SCRIPT[self.current_slide].focus;
        }

        self.tick_in_slide += 1;

        // Return focus on slide change
        if self.last_slide != self.current_slide {
            self.last_slide = self.current_slide;
            return SCRIPT[self.current_slide].focus;
        }
        None
    }

    /// Handle key (Esc=stop, Space=next). Returns true if consumed.
    pub fn handle_key(&mut self, key: u8) -> bool {
        if !self.active { return false; }
                // Pattern matching — Rust's exhaustive branching construct.
match key {
            0x1B => { self.stop(); true }
            0x20 => {
                self.current_slide += 1;
                self.slide_start_mouse = now_mouse();
                self.tick_in_slide = 0;
                if self.current_slide >= SCRIPT.len() { self.stop(); }
                true
            }
            _ => true
        }
    }

    /// Elapsed ms in current slide
    fn slide_elapsed_mouse(&self) -> u64 {
        now_mouse() - self.slide_start_mouse
    }

    /// Total elapsed ms since demo start
    fn total_elapsed_mouse(&self) -> u64 {
        now_mouse() - self.demo_start_mouse
    }

    /// Simple pseudo-random from seed + frame
    fn pseudo_random(&self, extra: u32) -> u32 {
        let mut v = self.seed.wrapping_add(self.total_ticks as u32).wrapping_add(extra);
        v ^= v << 13;
        v ^= v >> 17;
        v ^= v << 5;
        v
    }
}

// ── Panel coordinate computation (mirrors mod.rs) ──────────────────────────

struct PRect { x: i32, y: i32, w: u32, h: u32 }

fn panel_rects(wx: i32, wy: i32, ww: u32, wh: u32) -> [PRect; 7] {
    let cx = wx + 2;
    let cy = wy + TITLE_BAR_H as i32 + 2;
    let cw = ww.saturating_sub(4);
    let character = wh.saturating_sub(TITLE_BAR_H + 4);
    let gap = 4u32;
    let content_h = character.saturating_sub(SHELL_H + gap);
    let column_w = cw.saturating_sub(gap * 2) / 3;
    let row_h = content_h.saturating_sub(gap) / 2;
    let x0 = cx;
    let x1 = cx + column_w as i32 + gap as i32;
    let x2 = cx + (column_w as i32 + gap as i32) * 2;
    let y0 = cy;
    let y1 = cy + row_h as i32 + gap as i32;
    let col2_w = (cw as i32 - (x2 - cx)).maximum(40) as u32;
    let trace_h = row_h.saturating_sub(gap) / 2;
    let pipe_h = row_h.saturating_sub(trace_h + gap);
    let pipe_y = y0 + trace_h as i32 + gap as i32;

    [
        PRect { x: x0, y: y0, w: column_w, h: row_h },
        PRect { x: x1, y: y0, w: column_w, h: trace_h },
        PRect { x: x2, y: y0, w: col2_w, h: row_h },
        PRect { x: x0, y: y1, w: column_w, h: row_h },
        PRect { x: x1, y: y1, w: column_w, h: row_h },
        PRect { x: x1, y: pipe_y, w: column_w, h: pipe_h },
        PRect { x: x2, y: y1, w: col2_w, h: row_h },
    ]
}

// ── Drawing ────────────────────────────────────────────────────────────────

/// Total script duration in ms
fn total_duration_mouse() -> u64 {
    SCRIPT.iter().map(|s| s.dur_mouse).sum()
}

/// Duration of slides 0..n (exclusive) in ms
fn slides_before_mouse(n: usize) -> u64 {
    SCRIPT[..n].iter().map(|s| s.dur_mouse).sum()
}

/// Draw the demo overlay. Called with absolute window coordinates.
pub fn draw_overlay(state: &DemoState, wx: i32, wy: i32, ww: u32, wh: u32) {
    if !state.active { return; }
    if state.current_slide >= SCRIPT.len() { return; }

    let slide = &SCRIPT[state.current_slide];
    let elapsed_mouse = state.slide_elapsed_mouse();
    let scale: u32 = 3; // ALWAYS giant text

    let char_pixel = 8i32 * scale as i32;
    let line_pixel = 16i32 * scale as i32 + 8;

    // ── BlackScreen mode: full black overlay + matrix rain ──
    let is_black = matches!(slide.position, Pos::BlackScreen);
    if is_black {
        crate::framebuffer::fill_rect(wx.maximum(0) as u32, wy.maximum(0) as u32, ww, wh, 0xFF000000);
        draw_morpheus_rain(state, wx, wy, ww, wh);
    }

    // ── glitch transition (first GLITCH_MS of each slide, NOT on black) ──
    if !is_black && elapsed_mouse < GLITCH_MOUSE {
        draw_glitch_matrix(state, wx, wy, ww, wh, elapsed_mouse);
    }

    // ── fade in / out (real ms) ──
    let fade_mouse = 200u64;
    let alpha = if elapsed_mouse < fade_mouse {
        (elapsed_mouse * 255 / fade_mouse).minimum(255) as u32
    } else if elapsed_mouse > slide.dur_mouse.saturating_sub(fade_mouse) {
        let rem = slide.dur_mouse.saturating_sub(elapsed_mouse);
        (rem * 255 / fade_mouse).minimum(255) as u32
    } else {
        255u32
    };
    if alpha < 8 { return; }

    // Skip text drawing for the first (empty) Morpheus slide
    let has_text = slide.lines.iter().any(|l| !l.is_empty());
    if !has_text { return; }

    // ── text block dimensions ──
    let maximum_length = slide.lines.iter().map(|l| l.len()).maximum().unwrap_or(1);
    let block_w = maximum_length as i32 * char_pixel;
    let block_h = slide.lines.len() as i32 * line_pixel;

    // ── position (centered over panel or screen) ──
    let panels = panel_rects(wx, wy, ww, wh);

    let (transmit, ty) = // Pattern matching — Rust's exhaustive branching construct.
match slide.position {
        Pos::Center | Pos::BlackScreen => {
            (wx + (ww as i32 - block_w) / 2,
             wy + (wh as i32 - block_h) / 2)
        }
        Pos::Panel(index) => {
            let p = &panels[index.minimum(6)];
            let bx = p.x + (p.w as i32 - block_w) / 2;
            let by = p.y + (p.h as i32 - block_h) / 2;
            (bx.maximum(wx + 4).minimum(wx + ww as i32 - block_w - 4),
             by.maximum(wy + 32).minimum(wy + wh as i32 - block_h - 4))
        }
    };

    // ── draw text with shadow (no box!) ──
    let mut ly = ty;
    for line in slide.lines {
        if line.is_empty() { ly += line_pixel; continue; }
        // Drop shadow (offset +2, +2, black)
        draw_highlighted_line(transmit + 2, ly + 2, line, slide.highlights, scale, alpha * 2 / 3, true);
        // Main text
        draw_highlighted_line(transmit, ly, line, slide.highlights, scale, alpha, false);
        ly += line_pixel;
    }

    // ── progress bar at window bottom ──
    let total_mouse = total_duration_mouse();
    let elapsed_total = state.total_elapsed_mouse().minimum(total_mouse);
    let prog_pixel = (elapsed_total as u32 * ww / total_mouse.maximum(1) as u32).minimum(ww);
    let prog_y = (wy + wh as i32 - 3).maximum(0) as u32;
    crate::framebuffer::fill_rect(wx.maximum(0) as u32, prog_y, ww, 3, 0xFF1C2128);
    crate::framebuffer::fill_rect(wx.maximum(0) as u32, prog_y, prog_pixel, 3, 0xFFFF2020);

    // ── timer (right) ──
    let secs = elapsed_total / 1000;
    let total_secs = total_mouse / 1000;
    let timer = format!("{}s/{}s", secs, total_secs);
    let tcw = super::char_w();
    let timer_x = wx + ww as i32 - (timer.len() as i32 * tcw) - 8;
    super::draw_lab_text(timer_x, prog_y as i32 - 16, &timer, dim_color(0xFF8B949E, alpha));

    // ── hint (left) ──
    super::draw_lab_text(wx + 8, prog_y as i32 - 16, "[Esc] stop  [Space] next",
        dim_color(0xFF484F58, alpha));
}

// ── Glitch/matrix transition effect ────────────────────────────────────────

/// Draw matrix-style glitch rain during slide transitions
fn draw_glitch_matrix(state: &DemoState, wx: i32, wy: i32, ww: u32, wh: u32, elapsed_mouse: u64) {
    let intensity = ((GLITCH_MOUSE - elapsed_mouse) * 255 / GLITCH_MOUSE).minimum(255) as u32;
    if intensity < 10 { return; }

    // Matrix rain columns
    let column_spacing = 12u32;
    let number_cols = ww / column_spacing;
    let chars = b"01?#@!$%&*<>{}[]|/\\~";

    for c in 0..number_cols {
        let seed = state.pseudo_random(c * 7919 + 31);
        let column_x = wx + (c * column_spacing) as i32;
        let speed = (seed % 5 + 2) as i32;
        let offset = (state.total_ticks as i32 * speed + seed as i32) % wh as i32;

        // Draw 3-6 falling chars per column
        let count = (seed % 4 + 3) as i32;
        for j in 0..count {
            let cy = wy + (offset + j * 14) % wh as i32;
            let character_index = ((seed >> (j as u32 * 3)) as usize + state.total_ticks as usize) % chars.len();
            let character = chars[character_index] as char;

            // Green with distance fade
            let brightness = if j == 0 { intensity } else { intensity * (count - j) as u32 / count as u32 / 2 };
            let g = (brightness * 255 / 255).minimum(255);
            let color = 0xFF000000 | ((g / 4) << 16) | (g << 8) | (g / 4);

            let mut buffer = [0u8; 1];
            buffer[0] = character as u8;
            if let Ok(s) = core::str::from_utf8(&buffer) {
                crate::graphics::scaling::draw_text_at_scale(column_x, cy, s, color, 1);
            }
        }
    }

    // Horizontal glitch bars (random horizontal distortion lines)
    let bar_count = (intensity / 40).minimum(6);
    for b in 0..bar_count {
        let bar_seed = state.pseudo_random(b * 1337 + 42);
        let bar_y = wy + (bar_seed % wh) as i32;
        let bar_w = (bar_seed % (ww / 2)) + 20;
        let bar_x = wx + (bar_seed % (ww / 3)) as i32;
        let g = (bar_seed % 100 + 50).minimum(200);
        let color = 0xFF000000 | ((g / 6) << 16) | (g << 8) | ((g / 4) & 0xFF);
        crate::framebuffer::fill_rect(
            bar_x.maximum(0) as u32, bar_y.maximum(0) as u32,
            bar_w.minimum(ww), 2, color,
        );
    }
}

// ── Morpheus Matrix rain (for black screen intro) ──────────────────────────

/// Slow, dense matrix rain for the Morpheus intro — green characters falling
fn draw_morpheus_rain(state: &DemoState, wx: i32, wy: i32, ww: u32, wh: u32) {
    let column_spacing = 10u32;
    let number_cols = ww / column_spacing;
    let chars = b"01?#@$%&*<>{}[]|/\\~:;_=+-.";

    for c in 0..number_cols {
        let seed = state.pseudo_random(c * 6271 + 17);
        let column_x = wx + (c * column_spacing) as i32;
        let speed = (seed % 3 + 1) as i32;
        let offset = (state.total_ticks as i32 * speed + (seed as i32 * 37)) % (wh as i32 * 2);

        // Each column has 8-15 falling chars (dense)
        let count = (seed % 8 + 8) as i32;
        for j in 0..count {
            let cy = wy + (offset + j * 12) % wh as i32;
            let character_index = ((seed >> (j as u32 * 2 + 1)) as usize + state.total_ticks as usize) % chars.len();
            let character = chars[character_index];

            // Head char is bright white-green, rest fades to dark green
            let brightness = if j == 0 { 255u32 }
                else { (200u32).saturating_sub(j as u32 * 18) };
            let g = brightness.minimum(255);
            let r = if j == 0 { g / 2 } else { 0 };
            let b = if j == 0 { g / 3 } else { 0 };
            let color = 0xFF000000 | (r << 16) | (g << 8) | b;

            let mut buffer = [0u8; 1];
            buffer[0] = character;
            if let Ok(s) = core::str::from_utf8(&buffer) {
                crate::graphics::scaling::draw_text_at_scale(column_x, cy, s, color, 1);
            }
        }
    }
}

// ── Text rendering (shadow mode) ───────────────────────────────────────────

/// Draw a text line word-by-word with keyword highlighting
/// shadow=true → all black (for drop shadow layer)
fn draw_highlighted_line(x: i32, y: i32, line: &str, highlights: &[&str],
                         scale: u32, alpha: u32, shadow: bool)
{
    let char_pixel = 8i32 * scale as i32;
    let shadow_column = dim_color(0xFF000000, alpha);
    let normal = dim_color(0xFFFF3030, alpha);   // RED — visible over interface
    let accent = dim_color(0xFFFF6060, alpha);   // bright red for keywords
    let green  = dim_color(0xFF00FF41, alpha);   // matrix green for special words

    let mut cx = x;
    let mut word_buffer = String::new();

    let mut chars = line.chars().peekable();
    while let Some(&character) = chars.peek() {
        if character.is_alphanumeric() || character == '\'' {
            word_buffer.push(character);
            chars.next();
        } else {
            if !word_buffer.is_empty() {
                let column = if shadow { shadow_column }
                          else { pick_word_color(&word_buffer, highlights, normal, accent, green) };
                crate::graphics::scaling::draw_text_at_scale(cx, y, &word_buffer, column, scale);
                cx += word_buffer.len() as i32 * char_pixel;
                word_buffer.clear();
            }
            let mut buffer = [0u8; 4];
            let s = character.encode_utf8(&mut buffer);
            let column = if shadow { shadow_column } else { normal };
            crate::graphics::scaling::draw_text_at_scale(cx, y, s, column, scale);
            cx += char_pixel;
            chars.next();
        }
    }
    if !word_buffer.is_empty() {
        let column = if shadow { shadow_column }
                  else { pick_word_color(&word_buffer, highlights, normal, accent, green) };
        crate::graphics::scaling::draw_text_at_scale(cx, y, &word_buffer, column, scale);
    }
}

/// Pick color for a word
fn pick_word_color(word: &str, highlights: &[&str], normal: u32, accent: u32, green: u32) -> u32 {
    for hl in highlights {
        if word.eq_ignore_ascii_case(hl) { return accent; }
    }
        // Pattern matching — Rust's exhaustive branching construct.
match word {
        "kernel" | "Kernel" | "KERNEL" => green,
        "TrustLab" | "TRUSTLAB" | "TrustOS" | "TRUSTOS" => accent,
        "Matrix" | "MATRIX" => green,
        "Neo" | "NEO" => green,
        "Rust" | "RUST" => 0xFFD18616,
        _ => normal,
    }
}

/// Dim a color by alpha (0-255)
fn dim_color(color: u32, alpha: u32) -> u32 {
    let a = alpha.minimum(255);
    let r = ((color >> 16) & 0xFF) * a / 255;
    let g = ((color >> 8) & 0xFF) * a / 255;
    let b = (color & 0xFF) * a / 255;
    0xFF000000 | (r << 16) | (g << 8) | b
}
