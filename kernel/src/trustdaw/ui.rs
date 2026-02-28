//! TrustDAW Main UI — Framebuffer-based DAW Interface
//!
//! Renders the full DAW workspace to the framebuffer:
//!   - Transport bar (play/stop/record, BPM, position)
//!   - Track list with mixer controls
//!   - Piano roll editor for the selected track
//!   - Waveform display for rendered audio
//!
//! Keyboard-driven interaction (no mouse required, but mouse-aware).

use alloc::format;
use alloc::string::String;
use core::sync::atomic::Ordering;

use super::piano_roll::PianoRoll;
use super::{PLAYING, RECORDING, PLAYBACK_POS, BPM, TICKS_PER_QUARTER, MAX_TRACKS};

// ═══════════════════════════════════════════════════════════════════════════════
// UI Constants
// ═══════════════════════════════════════════════════════════════════════════════

/// Colors for the DAW UI
mod colors {
    pub const BG: u32 = 0x0D0D1A;
    pub const HEADER_BG: u32 = 0x1A1A2E;
    pub const TRANSPORT_BG: u32 = 0x151525;
    pub const TRACK_BG: u32 = 0x121222;
    pub const TRACK_SELECTED: u32 = 0x1A1A3A;
    pub const TRACK_BORDER: u32 = 0x2A2A4A;
    pub const TEXT_PRIMARY: u32 = 0xDDDDEE;
    pub const TEXT_SECONDARY: u32 = 0x8888AA;
    pub const TEXT_DIM: u32 = 0x555577;
    pub const PLAY_GREEN: u32 = 0x44DD44;
    pub const STOP_RED: u32 = 0xDD4444;
    pub const REC_RED: u32 = 0xFF2222;
    pub const BUTTON_BG: u32 = 0x2A2A4A;
    pub const BUTTON_ACTIVE: u32 = 0x3A3A6A;
    pub const MUTE_COLOR: u32 = 0xFF8800;
    pub const SOLO_COLOR: u32 = 0xFFDD00;
    pub const METER_GREEN: u32 = 0x44CC44;
    pub const METER_YELLOW: u32 = 0xCCCC44;
    pub const METER_RED: u32 = 0xCC4444;
    pub const METER_BG: u32 = 0x1A1A2A;
}

/// Layout dimensions
const TRANSPORT_HEIGHT: u32 = 40;
const TRACK_LIST_HEIGHT: u32 = 180;
const TRACK_ROW_HEIGHT: u32 = 28;
const TRACK_HEADER_WIDTH: u32 = 200;

// ═══════════════════════════════════════════════════════════════════════════════
// DAW UI State
// ═══════════════════════════════════════════════════════════════════════════════

/// The main DAW UI
pub struct DawUI {
    /// Selected track index
    pub selected_track: usize,
    /// Piano roll view
    pub piano_roll: PianoRoll,
    /// Show piano roll? (vs arrangement view)
    pub show_piano_roll: bool,
    /// UI needs redraw?
    pub dirty: bool,
}

impl DawUI {
    /// Create a new DAW UI filling the screen
    pub fn new() -> Self {
        let fb_w = crate::framebuffer::FB_WIDTH.load(Ordering::Relaxed) as u32;
        let fb_h = crate::framebuffer::FB_HEIGHT.load(Ordering::Relaxed) as u32;

        let piano_y = TRANSPORT_HEIGHT + TRACK_LIST_HEIGHT;
        let piano_h = fb_h.saturating_sub(piano_y);

        Self {
            selected_track: 0,
            piano_roll: PianoRoll::new(0, piano_y, fb_w, piano_h),
            show_piano_roll: true,
            dirty: true,
        }
    }

    /// Draw the entire DAW UI
    pub fn draw(&mut self) {
        let fb_w = crate::framebuffer::FB_WIDTH.load(Ordering::Relaxed) as u32;
        let fb_h = crate::framebuffer::FB_HEIGHT.load(Ordering::Relaxed) as u32;
        if fb_w == 0 || fb_h == 0 { return; }

        // Background
        crate::framebuffer::fill_rect(0, 0, fb_w, fb_h, colors::BG);

        // Get project data
        let project = super::PROJECT.lock();
        let mixer = super::MIXER.lock();

        if let (Some(proj), Some(mix)) = (project.as_ref(), mixer.as_ref()) {
            // 1. Transport bar
            self.draw_transport(fb_w, proj);

            // 2. Track list
            self.draw_track_list(fb_w, proj, mix);

            // 3. Piano roll for selected track
            if self.show_piano_roll {
                if let Some(track) = proj.tracks.get(self.selected_track) {
                    let playhead = PLAYBACK_POS.load(Ordering::Relaxed);
                    self.piano_roll.draw(track, playhead);
                } else {
                    // No tracks — show help text
                    let py = TRANSPORT_HEIGHT + TRACK_LIST_HEIGHT + 20;
                    crate::framebuffer::draw_text("No tracks. Use 'daw track add <name>' to create one.",
                        20, py, colors::TEXT_SECONDARY);
                }
            }
        } else {
            crate::framebuffer::draw_text("TrustDAW not initialized. Run 'daw init'",
                20, 20, colors::TEXT_PRIMARY);
        }

        self.dirty = false;
    }

    /// Draw the transport bar (top)
    fn draw_transport(&self, fb_w: u32, proj: &super::track::Project) {
        crate::framebuffer::fill_rect(0, 0, fb_w, TRANSPORT_HEIGHT, colors::TRANSPORT_BG);

        // Title
        crate::framebuffer::draw_text("TrustDAW", 8, 4, colors::TEXT_PRIMARY);

        // Project name
        let name = proj.name_str();
        crate::framebuffer::draw_text(name, 100, 4, colors::TEXT_SECONDARY);

        // Transport state
        let playing = PLAYING.load(Ordering::Relaxed);
        let recording = RECORDING.load(Ordering::Relaxed);

        // Play/Stop/Rec indicators
        let state_x = 250;
        if recording {
            crate::framebuffer::fill_circle(state_x + 8, 12, 6, colors::REC_RED);
            crate::framebuffer::draw_text("REC", state_x + 20, 4, colors::REC_RED);
        } else if playing {
            // Play triangle
            crate::framebuffer::fill_rect(state_x, 6, 3, 12, colors::PLAY_GREEN);
            crate::framebuffer::draw_text("PLAY", state_x + 20, 4, colors::PLAY_GREEN);
        } else {
            crate::framebuffer::fill_rect(state_x, 6, 12, 12, colors::STOP_RED);
            crate::framebuffer::draw_text("STOP", state_x + 20, 4, colors::TEXT_DIM);
        }

        // BPM
        let bpm = BPM.load(Ordering::Relaxed);
        let bpm_str = format!("BPM: {}", bpm);
        crate::framebuffer::draw_text(&bpm_str, 350, 4, colors::TEXT_PRIMARY);

        // Position (Bar:Beat:Tick)
        let pos = PLAYBACK_POS.load(Ordering::Relaxed);
        let bar = pos / (TICKS_PER_QUARTER * 4);
        let beat = (pos % (TICKS_PER_QUARTER * 4)) / TICKS_PER_QUARTER;
        let tick = pos % TICKS_PER_QUARTER;
        let pos_str = format!("{}:{:02}:{:03}", bar + 1, beat + 1, tick);
        crate::framebuffer::draw_text(&pos_str, 450, 4, colors::TEXT_PRIMARY);

        // Track count
        let track_str = format!("Tracks: {}/{}", proj.tracks.len(), MAX_TRACKS);
        crate::framebuffer::draw_text(&track_str, 560, 4, colors::TEXT_SECONDARY);

        // Bottom border
        crate::framebuffer::draw_hline(0, TRANSPORT_HEIGHT - 1, fb_w, colors::TRACK_BORDER);

        // Second row: keyboard shortcuts
        crate::framebuffer::draw_text(
            "[Space] Play/Stop  [R] Record  [+/-] BPM  [Tab] Track  [F5] Piano Roll  [F8] Export WAV",
            8, 22, colors::TEXT_DIM
        );
    }

    /// Draw the track list (middle section)
    fn draw_track_list(&self, fb_w: u32, proj: &super::track::Project, mixer: &super::mixer::Mixer) {
        let list_y = TRANSPORT_HEIGHT;
        crate::framebuffer::fill_rect(0, list_y, fb_w, TRACK_LIST_HEIGHT, colors::TRACK_BG);

        // Header
        crate::framebuffer::fill_rect(0, list_y, fb_w, TRACK_ROW_HEIGHT, colors::HEADER_BG);
        crate::framebuffer::draw_text(" # ", 4, list_y + 6, colors::TEXT_DIM);
        crate::framebuffer::draw_text("Track", 30, list_y + 6, colors::TEXT_DIM);
        crate::framebuffer::draw_text("Wave", 140, list_y + 6, colors::TEXT_DIM);
        crate::framebuffer::draw_text("Notes", 200, list_y + 6, colors::TEXT_DIM);
        crate::framebuffer::draw_text("Vol", 260, list_y + 6, colors::TEXT_DIM);
        crate::framebuffer::draw_text("Pan", 310, list_y + 6, colors::TEXT_DIM);
        crate::framebuffer::draw_text("M S", 360, list_y + 6, colors::TEXT_DIM);
        crate::framebuffer::draw_text("Meter", 400, list_y + 6, colors::TEXT_DIM);

        // Track rows
        for (i, track) in proj.tracks.iter().enumerate() {
            let row_y = list_y + TRACK_ROW_HEIGHT + (i as u32 * TRACK_ROW_HEIGHT);
            if row_y + TRACK_ROW_HEIGHT > list_y + TRACK_LIST_HEIGHT {
                break; // Out of visible area
            }

            // Selected highlight
            let bg = if i == self.selected_track {
                colors::TRACK_SELECTED
            } else {
                colors::TRACK_BG
            };
            crate::framebuffer::fill_rect(0, row_y, fb_w, TRACK_ROW_HEIGHT, bg);

            // Track color indicator
            crate::framebuffer::fill_rect(0, row_y, 4, TRACK_ROW_HEIGHT, track.color);

            // Track number
            let num_str = format!("{}", i);
            crate::framebuffer::draw_text(&num_str, 8, row_y + 6, colors::TEXT_PRIMARY);

            // Track name
            crate::framebuffer::draw_text(track.name_str(), 30, row_y + 6, colors::TEXT_PRIMARY);

            // Waveform
            crate::framebuffer::draw_text(track.waveform.short_name(), 140, row_y + 6, colors::TEXT_SECONDARY);

            // Note count
            let notes_str = format!("{}", track.notes.len());
            crate::framebuffer::draw_text(&notes_str, 200, row_y + 6, colors::TEXT_SECONDARY);

            // Volume
            if let Some(ch) = mixer.channels.get(i) {
                let vol_str = format!("{}", ch.volume);
                crate::framebuffer::draw_text(&vol_str, 260, row_y + 6, colors::TEXT_PRIMARY);

                // Pan
                let pan_str = if ch.pan == 0 { String::from("C") }
                    else if ch.pan > 0 { format!("R{}", ch.pan) }
                    else { format!("L{}", -ch.pan) };
                crate::framebuffer::draw_text(&pan_str, 310, row_y + 6, colors::TEXT_SECONDARY);

                // Mute indicator
                if ch.muted {
                    crate::framebuffer::draw_text("M", 360, row_y + 6, colors::MUTE_COLOR);
                }

                // Solo indicator
                if ch.solo {
                    crate::framebuffer::draw_text("S", 376, row_y + 6, colors::SOLO_COLOR);
                }

                // Simple volume meter bar
                let meter_x: u32 = 400;
                let meter_w: u32 = (fb_w - meter_x).saturating_sub(20).min(200);
                let filled = (meter_w as u32 * ch.volume as u32) / 255;
                crate::framebuffer::fill_rect(meter_x, row_y + 8, meter_w, 12, colors::METER_BG);
                if filled > 0 {
                    let meter_color = if ch.volume > 230 { colors::METER_RED }
                        else if ch.volume > 180 { colors::METER_YELLOW }
                        else { colors::METER_GREEN };
                    crate::framebuffer::fill_rect(meter_x, row_y + 8, filled, 12, meter_color);
                }
            }

            // Bottom border
            crate::framebuffer::draw_hline(0, row_y + TRACK_ROW_HEIGHT - 1, fb_w, colors::TRACK_BORDER);
        }

        // Bottom border of track list
        crate::framebuffer::draw_hline(0, list_y + TRACK_LIST_HEIGHT - 1, fb_w, colors::TRACK_BORDER);
    }

    // ─── Track selection ─────────────────────────────────────────────────

    /// Select next track
    pub fn next_track(&mut self) {
        let project = super::PROJECT.lock();
        if let Some(proj) = project.as_ref() {
            if !proj.tracks.is_empty() {
                self.selected_track = (self.selected_track + 1) % proj.tracks.len();
                self.dirty = true;
            }
        }
    }

    /// Select previous track
    pub fn prev_track(&mut self) {
        let project = super::PROJECT.lock();
        if let Some(proj) = project.as_ref() {
            if !proj.tracks.is_empty() {
                if self.selected_track == 0 {
                    self.selected_track = proj.tracks.len() - 1;
                } else {
                    self.selected_track -= 1;
                }
                self.dirty = true;
            }
        }
    }

    /// Toggle piano roll visibility
    pub fn toggle_piano_roll(&mut self) {
        self.show_piano_roll = !self.show_piano_roll;
        self.dirty = true;
    }
}

/// Launch the full DAW GUI (blocking, interactive mode)
pub fn launch_gui() -> Result<(), &'static str> {
    super::ensure_init()?;

    let mut ui = DawUI::new();
    ui.draw();

    crate::println!("TrustDAW GUI launched. Press [Esc] to return to shell.");

    // Interactive loop
    loop {
        if let Some(scancode) = crate::keyboard::try_read_key() {
            let is_release = scancode & 0x80 != 0;
            if is_release { continue; } // Only process key presses

            match scancode {
                0x01 => break, // Escape → exit GUI
                0x39 => { // Space → toggle play/stop
                    if PLAYING.load(Ordering::Relaxed) {
                        super::stop();
                    } else {
                        let _ = super::play();
                    }
                    ui.dirty = true;
                }
                0x13 => { // R → toggle record
                    if RECORDING.load(Ordering::Relaxed) {
                        RECORDING.store(false, Ordering::Relaxed);
                    } else {
                        let _ = super::recorder::record_interactive(ui.selected_track);
                    }
                    ui.dirty = true;
                }
                0x0F => { // Tab → next track
                    ui.next_track();
                }
                0x3F => { // F5 → toggle piano roll
                    ui.toggle_piano_roll();
                }
                0x42 => { // F8 → export WAV
                    let _ = super::export_wav("/home/output.wav");
                    crate::println!("Exported to /home/output.wav");
                }
                0x0C => { // - key → BPM down
                    let bpm = BPM.load(Ordering::Relaxed);
                    super::set_bpm(bpm.saturating_sub(5));
                    ui.dirty = true;
                }
                0x0D => { // = key → BPM up
                    let bpm = BPM.load(Ordering::Relaxed);
                    super::set_bpm(bpm + 5);
                    ui.dirty = true;
                }
                // Arrow keys for piano roll navigation
                0x48 => { ui.piano_roll.cursor_up(); ui.dirty = true; }    // Up
                0x50 => { ui.piano_roll.cursor_down(); ui.dirty = true; }  // Down
                0x4B => { ui.piano_roll.cursor_left(); ui.dirty = true; }  // Left
                0x4D => { ui.piano_roll.cursor_right(); ui.dirty = true; } // Right
                _ => {}
            }

            if ui.dirty {
                ui.draw();
            }
        }

        // Brief spin to prevent 100% CPU
        for _ in 0..5000 {
            core::hint::spin_loop();
        }
    }

    Ok(())
}
