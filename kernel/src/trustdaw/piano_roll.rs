//! Piano Roll — Graphical note editor for TrustDAW
//!
//! Renders a grid-based view where:
//!   - X axis = time (bars/beats)
//!   - Y axis = MIDI pitch (piano keys)
//!   - Notes are colored rectangles
//!
//! Draws directly to the TrustOS framebuffer.

use alloc::format;
use alloc::string::String;
use super::track::{Track, Note};
use super::{TICKS_PER_QUARTER, BPM};
use core::sync::atomic::Ordering;

// ═══════════════════════════════════════════════════════════════════════════════
// Piano Roll Constants  
// ═══════════════════════════════════════════════════════════════════════════════

/// Pixels per tick (horizontal zoom)
const DEFAULT_PIXELS_PER_TICK: u32 = 1;
/// Pixels per semitone (vertical zoom)
const DEFAULT_KEY_HEIGHT: u32 = 8;
/// Width of the piano key labels (left sidebar)
const KEY_LABEL_WIDTH: u32 = 48;
/// Height of the top timeline bar
const TIMELINE_HEIGHT: u32 = 24;
/// Lowest MIDI note displayed
const MIN_NOTE: u8 = 24;  // C1
/// Highest MIDI note displayed
const MAX_NOTE: u8 = 96;  // C7

/// Piano roll colors
mod colors {
    pub const BG: u32 = 0x1A1A2E;
    pub const GRID_LINE: u32 = 0x2A2A3E;
    pub const GRID_BEAT: u32 = 0x3A3A4E;
    pub const GRID_BAR: u32 = 0x5A5A6E;
    pub const WHITE_KEY_BG: u32 = 0x222236;
    pub const BLACK_KEY_BG: u32 = 0x181828;
    pub const KEY_LABEL_BG: u32 = 0x101020;
    pub const KEY_WHITE: u32 = 0xCCCCDD;
    pub const KEY_BLACK: u32 = 0x333344;
    pub const KEY_TEXT: u32 = 0x888899;
    pub const TIMELINE_BG: u32 = 0x151530;
    pub const TIMELINE_TEXT: u32 = 0xAAAABB;
    pub const NOTE_BORDER: u32 = 0xFFFFFF;
    pub const PLAYHEAD: u32 = 0xFF4444;
    pub const SELECTION: u32 = 0x4488FF;
    pub const CURSOR: u32 = 0x44FF44;
}

// ═══════════════════════════════════════════════════════════════════════════════
// Piano Roll State
// ═══════════════════════════════════════════════════════════════════════════════

/// Piano roll view state
pub struct PianoRoll {
    /// Top-left X position on screen
    pub x: u32,
    /// Top-left Y position on screen
    pub y: u32,
    /// Width in pixels
    pub width: u32,
    /// Height in pixels
    pub height: u32,
    /// Horizontal zoom (pixels per tick)
    pub h_zoom: u32,
    /// Vertical zoom (pixels per semitone)
    pub key_height: u32,
    /// Horizontal scroll offset in ticks
    pub scroll_x: u32,
    /// Vertical scroll offset in MIDI notes (bottom note)
    pub scroll_y: u8,
    /// Currently selected note index (None if nothing selected)
    pub selected_note: Option<usize>,
    /// Cursor position (tick, pitch) for keyboard editing
    pub cursor_tick: u32,
    pub cursor_pitch: u8,
    /// Grid snap size in ticks (480 = quarter, 240 = eighth, 120 = sixteenth)
    pub grid_snap: u32,
}

impl PianoRoll {
    /// Create a new piano roll
    pub fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
            h_zoom: DEFAULT_PIXELS_PER_TICK,
            key_height: DEFAULT_KEY_HEIGHT,
            scroll_x: 0,
            scroll_y: MIN_NOTE,
            selected_note: None,
            cursor_tick: 0,
            cursor_pitch: 60, // C4
            grid_snap: TICKS_PER_QUARTER / 4, // Sixteenth note
        }
    }

    /// Draw the complete piano roll for a track
    pub fn draw(&self, track: &Track, playhead_tick: u32) {
        let fb_w = crate::framebuffer::FB_WIDTH.load(Ordering::Relaxed) as u32;
        let fb_h = crate::framebuffer::FB_HEIGHT.load(Ordering::Relaxed) as u32;
        if fb_w == 0 || fb_h == 0 { return; }

        // Clamp dimensions
        let w = self.width.min(fb_w - self.x);
        let h = self.height.min(fb_h - self.y);
        let grid_x = self.x + KEY_LABEL_WIDTH;
        let grid_y = self.y + TIMELINE_HEIGHT;
        let grid_w = w.saturating_sub(KEY_LABEL_WIDTH);
        let grid_h = h.saturating_sub(TIMELINE_HEIGHT);

        // 1. Background
        crate::framebuffer::fill_rect(self.x, self.y, w, h, colors::BG);

        // 2. Draw piano key labels (left sidebar)
        self.draw_key_labels(grid_h);

        // 3. Draw grid (alternating white/black key rows + beat/bar lines)
        self.draw_grid(grid_x, grid_y, grid_w, grid_h);

        // 4. Draw timeline (top bar with bar/beat numbers)
        self.draw_timeline(grid_x, grid_w);

        // 5. Draw notes from the track
        self.draw_notes(track, grid_x, grid_y, grid_w, grid_h);

        // 6. Draw playhead
        self.draw_playhead(playhead_tick, grid_x, grid_y, grid_w, grid_h);

        // 7. Draw cursor
        self.draw_cursor(grid_x, grid_y, grid_w, grid_h);
    }

    /// Draw piano key labels on the left
    fn draw_key_labels(&self, grid_h: u32) {
        let label_x = self.x;
        let label_y = self.y + TIMELINE_HEIGHT;
        let visible_keys = grid_h / self.key_height;

        crate::framebuffer::fill_rect(label_x, label_y, KEY_LABEL_WIDTH, grid_h, colors::KEY_LABEL_BG);

        for i in 0..visible_keys {
            let pitch = self.pitch_from_row(i);
            if pitch > 127 { continue; }

            let row_y = label_y + (visible_keys - 1 - i) * self.key_height;
            let is_black = is_black_key(pitch);

            // Key background
            let key_bg = if is_black { colors::KEY_BLACK } else { colors::KEY_WHITE };
            crate::framebuffer::fill_rect(label_x, row_y, KEY_LABEL_WIDTH - 2, self.key_height, key_bg);

            // Note name label (show only for C notes and current cursor pitch)
            if pitch % 12 == 0 || pitch == self.cursor_pitch {
                let name = crate::audio::tables::midi_to_note_name(pitch);
                let oct = crate::audio::tables::midi_octave(pitch);
                let label = format!("{}{}", name, oct);
                crate::framebuffer::draw_text(&label, label_x + 4, row_y + 1, colors::KEY_TEXT);
            }
        }
    }

    /// Draw the grid background
    fn draw_grid(&self, gx: u32, gy: u32, gw: u32, gh: u32) {
        let visible_keys = gh / self.key_height;

        // Draw horizontal rows (alternating for white/black keys)
        for i in 0..visible_keys {
            let pitch = self.pitch_from_row(i);
            if pitch > 127 { continue; }

            let row_y = gy + (visible_keys - 1 - i) * self.key_height;
            let row_color = if is_black_key(pitch) {
                colors::BLACK_KEY_BG
            } else {
                colors::WHITE_KEY_BG
            };
            crate::framebuffer::fill_rect(gx, row_y, gw, self.key_height, row_color);

            // Horizontal divider line
            crate::framebuffer::draw_hline(gx, row_y, gw, colors::GRID_LINE);
        }

        // Draw vertical lines (beat and bar lines)
        let ticks_per_bar = TICKS_PER_QUARTER * 4; // Assuming 4/4 time
        let ticks_visible = gw / self.h_zoom.max(1);

        let start_tick = self.scroll_x;
        let end_tick = start_tick + ticks_visible;

        // Bar lines
        let first_bar_tick = (start_tick / ticks_per_bar) * ticks_per_bar;
        let mut tick = first_bar_tick;
        while tick <= end_tick {
            let px = self.tick_to_pixel(tick, gx);
            if px >= gx && px < gx + gw {
                crate::framebuffer::draw_vline(px, gy, gh, colors::GRID_BAR);
            }
            tick += ticks_per_bar;
        }

        // Beat lines
        let first_beat_tick = (start_tick / TICKS_PER_QUARTER) * TICKS_PER_QUARTER;
        tick = first_beat_tick;
        while tick <= end_tick {
            if tick % ticks_per_bar != 0 { // Don't overdraw bar lines
                let px = self.tick_to_pixel(tick, gx);
                if px >= gx && px < gx + gw {
                    crate::framebuffer::draw_vline(px, gy, gh, colors::GRID_BEAT);
                }
            }
            tick += TICKS_PER_QUARTER;
        }
    }

    /// Draw the timeline header
    fn draw_timeline(&self, gx: u32, gw: u32) {
        let ty = self.y;
        crate::framebuffer::fill_rect(self.x, ty, self.width, TIMELINE_HEIGHT, colors::TIMELINE_BG);

        let ticks_per_bar = TICKS_PER_QUARTER * 4;
        let ticks_visible = gw / self.h_zoom.max(1);
        let start_tick = self.scroll_x;
        let end_tick = start_tick + ticks_visible;

        let first_bar_tick = (start_tick / ticks_per_bar) * ticks_per_bar;
        let mut tick = first_bar_tick;
        while tick <= end_tick {
            let bar_num = tick / ticks_per_bar + 1;
            let px = self.tick_to_pixel(tick, gx);
            if px >= gx && px < gx + gw {
                let label = format!("{}", bar_num);
                crate::framebuffer::draw_text(&label, px + 2, ty + 4, colors::TIMELINE_TEXT);
                crate::framebuffer::draw_vline(px, ty, TIMELINE_HEIGHT, colors::GRID_BAR);
            }
            tick += ticks_per_bar;
        }
    }

    /// Draw notes from the track
    fn draw_notes(&self, track: &Track, gx: u32, gy: u32, gw: u32, gh: u32) {
        let visible_keys = gh / self.key_height;

        for (i, note) in track.notes.iter().enumerate() {
            // Check if note is visible in our viewport
            let note_px_start = self.tick_to_pixel(note.start_tick, gx);
            let note_px_end = self.tick_to_pixel(note.end_tick(), gx);
            let note_width = (note_px_end.saturating_sub(note_px_start)).max(2);

            // Check if note pitch is visible
            if note.pitch < self.scroll_y || note.pitch >= self.scroll_y + visible_keys as u8 {
                continue;
            }

            // Check if note is horizontally visible
            if note_px_end < gx || note_px_start > gx + gw {
                continue;
            }

            let row_from_bottom = (note.pitch - self.scroll_y) as u32;
            let note_y = gy + (visible_keys - 1 - row_from_bottom) * self.key_height + 1;

            // Clamp horizontal position
            let draw_x = note_px_start.max(gx);
            let draw_w = note_width.min(gx + gw - draw_x);

            // Note color — use track color with velocity brightness
            let brightness = note.velocity as u32 * 100 / 127;
            let note_color = adjust_brightness(track.color, brightness);

            // Draw the note rectangle
            crate::framebuffer::fill_rect(draw_x, note_y, draw_w, self.key_height - 2, note_color);

            // Border highlight for selected note
            if self.selected_note == Some(i) {
                crate::framebuffer::draw_rect(draw_x, note_y, draw_w, self.key_height - 2, colors::SELECTION);
            }

            // Note name text if the note is wide enough
            if draw_w > 24 {
                let name = crate::audio::tables::midi_to_note_name(note.pitch);
                crate::framebuffer::draw_text(name, draw_x + 2, note_y + 1, 0xFFFFFF);
            }
        }
    }

    /// Draw the playhead (vertical red line at current position)
    fn draw_playhead(&self, tick: u32, gx: u32, gy: u32, gw: u32, gh: u32) {
        let px = self.tick_to_pixel(tick, gx);
        if px >= gx && px < gx + gw {
            crate::framebuffer::draw_vline(px, gy, gh, colors::PLAYHEAD);
            // Small triangle at top
            for i in 0..4u32 {
                crate::framebuffer::draw_hline(px.saturating_sub(i), gy.saturating_sub(i + 1), i * 2 + 1, colors::PLAYHEAD);
            }
        }
    }

    /// Draw the edit cursor
    fn draw_cursor(&self, gx: u32, gy: u32, gw: u32, gh: u32) {
        let visible_keys = gh / self.key_height;

        // Vertical line at cursor tick
        let cx = self.tick_to_pixel(self.cursor_tick, gx);
        if cx >= gx && cx < gx + gw {
            crate::framebuffer::draw_vline(cx, gy, gh, colors::CURSOR);
        }

        // Horizontal highlight at cursor pitch
        if self.cursor_pitch >= self.scroll_y && self.cursor_pitch < self.scroll_y + visible_keys as u8 {
            let row = (self.cursor_pitch - self.scroll_y) as u32;
            let cy = gy + (visible_keys - 1 - row) * self.key_height;
            crate::framebuffer::fill_rect_alpha(gx, cy, gw, self.key_height, colors::CURSOR, 40);
        }
    }

    // ─── Coordinate helpers ──────────────────────────────────────────────

    /// Convert tick position to pixel X coordinate
    fn tick_to_pixel(&self, tick: u32, grid_x: u32) -> u32 {
        if tick >= self.scroll_x {
            grid_x + (tick - self.scroll_x) * self.h_zoom
        } else {
            grid_x // Off-screen left
        }
    }

    /// Convert pixel X to tick position
    pub fn pixel_to_tick(&self, px: u32, grid_x: u32) -> u32 {
        if px >= grid_x && self.h_zoom > 0 {
            self.scroll_x + (px - grid_x) / self.h_zoom
        } else {
            self.scroll_x
        }
    }

    /// Get MIDI pitch from a row index (0 = bottom)
    fn pitch_from_row(&self, row: u32) -> u8 {
        let pitch = self.scroll_y as u32 + row;
        if pitch > 127 { 127 } else { pitch as u8 }
    }

    // ─── Navigation ──────────────────────────────────────────────────────

    /// Scroll left by one bar
    pub fn scroll_left(&mut self) {
        let bar_ticks = TICKS_PER_QUARTER * 4;
        self.scroll_x = self.scroll_x.saturating_sub(bar_ticks);
    }

    /// Scroll right by one bar
    pub fn scroll_right(&mut self) {
        let bar_ticks = TICKS_PER_QUARTER * 4;
        self.scroll_x += bar_ticks;
    }

    /// Scroll up (higher pitch)
    pub fn scroll_up(&mut self) {
        if self.scroll_y < MAX_NOTE - 12 {
            self.scroll_y += 12; // One octave
        }
    }

    /// Scroll down (lower pitch)
    pub fn scroll_down(&mut self) {
        if self.scroll_y > MIN_NOTE + 12 {
            self.scroll_y -= 12;
        } else {
            self.scroll_y = MIN_NOTE;
        }
    }

    /// Zoom in horizontally
    pub fn zoom_in(&mut self) {
        if self.h_zoom < 8 {
            self.h_zoom += 1;
        }
    }

    /// Zoom out horizontally
    pub fn zoom_out(&mut self) {
        if self.h_zoom > 1 {
            self.h_zoom -= 1;
        }
    }

    /// Move cursor right by grid snap
    pub fn cursor_right(&mut self) {
        self.cursor_tick += self.grid_snap;
    }

    /// Move cursor left by grid snap
    pub fn cursor_left(&mut self) {
        self.cursor_tick = self.cursor_tick.saturating_sub(self.grid_snap);
    }

    /// Move cursor up (higher pitch)
    pub fn cursor_up(&mut self) {
        if self.cursor_pitch < 127 {
            self.cursor_pitch += 1;
        }
    }

    /// Move cursor down (lower pitch)
    pub fn cursor_down(&mut self) {
        if self.cursor_pitch > 0 {
            self.cursor_pitch -= 1;
        }
    }

    /// Snap cursor tick to grid
    pub fn snap_to_grid(&mut self) {
        if self.grid_snap > 0 {
            let remainder = self.cursor_tick % self.grid_snap;
            if remainder > self.grid_snap / 2 {
                self.cursor_tick += self.grid_snap - remainder;
            } else {
                self.cursor_tick -= remainder;
            }
        }
    }

    /// Set grid snap (in ticks)
    pub fn set_grid(&mut self, division: &str) {
        self.grid_snap = match division {
            "1" | "whole" => TICKS_PER_QUARTER * 4,
            "1/2" | "half" => TICKS_PER_QUARTER * 2,
            "1/4" | "quarter" => TICKS_PER_QUARTER,
            "1/8" | "eighth" => TICKS_PER_QUARTER / 2,
            "1/16" | "sixteenth" => TICKS_PER_QUARTER / 4,
            "1/32" | "thirtysecond" => TICKS_PER_QUARTER / 8,
            "off" | "free" => 1,
            _ => self.grid_snap, // Keep current
        };
    }

    /// Add a note at the cursor position
    pub fn add_note_at_cursor(&self, track: &mut Track, velocity: u8, duration_ticks: u32) {
        let note = Note::new(self.cursor_pitch, velocity, self.cursor_tick, duration_ticks);
        track.add_note(note);
    }

    /// Delete the note under the cursor
    pub fn delete_note_at_cursor(&self, track: &mut Track) -> bool {
        let notes_at = track.notes_at_tick(self.cursor_tick);
        if let Some(note) = notes_at.iter().find(|n| n.pitch == self.cursor_pitch) {
            let idx = track.notes.iter().position(|n|
                n.pitch == note.pitch && n.start_tick == note.start_tick
            );
            if let Some(idx) = idx {
                track.remove_note(idx);
                return true;
            }
        }
        false
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Helper functions
// ═══════════════════════════════════════════════════════════════════════════════

/// Check if a MIDI note is a black key
fn is_black_key(pitch: u8) -> bool {
    matches!(pitch % 12, 1 | 3 | 6 | 8 | 10) // C# D# F# G# A#
}

/// Adjust color brightness (0-100%)
fn adjust_brightness(color: u32, brightness: u32) -> u32 {
    let r = ((color >> 16) & 0xFF) * brightness / 100;
    let g = ((color >> 8) & 0xFF) * brightness / 100;
    let b = (color & 0xFF) * brightness / 100;
    (r.min(255) << 16) | (g.min(255) << 8) | b.min(255)
}

/// Render a text-mode piano roll (for serial/terminal output)
pub fn text_piano_roll(track: &Track, bars: u32) -> String {
    let mut s = String::new();
    let ticks_per_bar = TICKS_PER_QUARTER * 4;
    let total_ticks = ticks_per_bar * bars;
    let cols = (bars * 16) as usize; // 16 columns per bar (sixteenth notes)
    let tick_per_col = ticks_per_bar / 16;

    s.push_str(&format!("Piano Roll: \"{}\" — {} bars, {} notes\n",
        track.name_str(), bars, track.notes.len()));
    s.push_str(&format!("Grid: 1/16 note | {} = {}\n\n", track.waveform.name(),
        if track.armed { "ARMED" } else { "" }));

    // Header: bar numbers
    s.push_str("     │");
    for bar in 0..bars {
        s.push_str(&format!("{:^16}", bar + 1));
    }
    s.push_str("\n     │");
    for _ in 0..bars {
        s.push_str("────────────────");
    }
    s.push('\n');

    // Find pitch range of notes in track
    let (min_pitch, max_pitch) = if track.notes.is_empty() {
        (57, 72) // A3 to C5 default
    } else {
        let min = track.notes.iter().map(|n| n.pitch).min().unwrap_or(60);
        let max = track.notes.iter().map(|n| n.pitch).max().unwrap_or(72);
        (min.saturating_sub(2), (max + 2).min(127))
    };

    // Draw rows from highest to lowest pitch
    for pitch in (min_pitch..=max_pitch).rev() {
        let name = crate::audio::tables::midi_to_note_name(pitch);
        let oct = crate::audio::tables::midi_octave(pitch);
        let is_c = pitch % 12 == 0;
        
        s.push_str(&format!("{}{:<2} {}│", name, oct,
            if is_c { "─" } else { " " }));

        for col in 0..cols {
            let tick = col as u32 * tick_per_col;
            let active = track.notes.iter().any(|n|
                n.pitch == pitch && n.start_tick <= tick && tick < n.end_tick()
            );
            let is_note_start = track.notes.iter().any(|n|
                n.pitch == pitch && n.start_tick == tick
            );

            if is_note_start {
                s.push('█');
            } else if active {
                s.push('▓');
            } else if col % 16 == 0 {
                s.push('│');
            } else if col % 4 == 0 {
                s.push('┊');
            } else {
                s.push('·');
            }
        }
        s.push('\n');
    }

    s
}
