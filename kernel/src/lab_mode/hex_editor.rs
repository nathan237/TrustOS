//! Hex Editor Panel — Colorful hex dump viewer
//!
//! Displays file contents (or raw memory) as a hex dump with color-coded
//! byte categories matching the TrustLab theme:
//!   - Green  : printable ASCII (0x20-0x7E)
//!   - Cyan   : whitespace (tab, newline, CR, space)
//!   - Yellow : control characters (0x01-0x1F)
//!   - Purple : high bytes (0x80-0xFE)
//!   - Red    : 0xFF
//!   - Dim    : null bytes (0x00)
//!   - Orange : header/magic bytes
//!
//! Loads data from ramfs files. Select a file in the File Tree then
//! type "hex" in the shell bar to load it.

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use super::{draw_lab_text, char_w, char_h,
            COL_TEXT, COL_DIM, COL_ACCENT, COL_GREEN, COL_YELLOW, COL_RED,
            COL_PURPLE, COL_CYAN, COL_ORANGE};

/// Color for a given byte value
fn byte_color(b: u8) -> u32 {
    match b {
        0x00 => COL_DIM,                   // null
        0x09 | 0x0A | 0x0D | 0x20 => COL_CYAN,  // whitespace
        0x01..=0x1F => COL_YELLOW,         // control
        0x20..=0x7E => COL_GREEN,          // printable ASCII
        0x7F => COL_YELLOW,                // DEL
        0xFF => COL_RED,                   // 0xFF
        0x80..=0xFE => COL_PURPLE,         // high bytes
    }
}

/// ASCII representation char for a byte
fn byte_ascii(b: u8) -> char {
    if b >= 0x20 && b <= 0x7E { b as char } else { '.' }
}

/// Hex editor panel state
pub struct HexEditorState {
    /// Raw data being viewed
    pub data: Vec<u8>,
    /// File path loaded (empty = none)
    pub file_path: String,
    /// Scroll offset in rows (each row = 16 bytes)
    pub scroll: usize,
    /// Cursor offset in bytes
    pub cursor: usize,
    /// Bytes per row
    pub bytes_per_row: usize,
    /// Frame counter
    pub frame: u64,
}

impl HexEditorState {
    pub fn new() -> Self {
        // Default: load welcome text or show sample data
        let mut s = Self {
            data: Vec::new(),
            file_path: String::new(),
            scroll: 0,
            cursor: 0,
            bytes_per_row: 16,
            frame: 0,
        };
        // Try to load /home/welcome.txt
        s.load_file("/home/welcome.txt");
        if s.data.is_empty() {
            s.load_sample();
        }
        s
    }

    /// Load a file from ramfs
    pub fn load_file(&mut self, path: &str) {
        let data = crate::ramfs::with_fs(|fs| {
            fs.read_file(path).ok().map(|bytes| bytes.to_vec())
        });
        if let Some(bytes) = data {
            self.data = bytes;
            self.file_path = String::from(path);
            self.cursor = 0;
            self.scroll = 0;
        }
    }

    /// Load sample data for demonstration
    fn load_sample(&mut self) {
        self.file_path = String::from("<sample>");
        self.data.clear();
        // ELF-like magic header
        self.data.extend_from_slice(&[0x7F, b'E', b'L', b'F', 0x02, 0x01, 0x01, 0x00]);
        self.data.extend_from_slice(&[0x00; 8]); // padding
        // Some mixed content
        for i in 0u8..=255 {
            self.data.push(i);
        }
        // ASCII text section
        self.data.extend_from_slice(b"TrustOS Kernel v0.1\x00");
        self.data.extend_from_slice(b"Built with Rust\x00\xFF\xFF");
        self.cursor = 0;
        self.scroll = 0;
    }

    pub fn handle_key(&mut self, key: u8) {
        use crate::keyboard::{KEY_UP, KEY_DOWN, KEY_LEFT, KEY_RIGHT, KEY_PGUP, KEY_PGDOWN};
        let bpr = self.bytes_per_row;
        match key {
            KEY_UP => {
                self.cursor = self.cursor.saturating_sub(bpr);
            }
            KEY_DOWN => {
                if self.cursor + bpr < self.data.len() {
                    self.cursor += bpr;
                }
            }
            KEY_LEFT => {
                self.cursor = self.cursor.saturating_sub(1);
            }
            KEY_RIGHT => {
                if self.cursor + 1 < self.data.len() {
                    self.cursor += 1;
                }
            }
            KEY_PGUP => {
                self.cursor = self.cursor.saturating_sub(bpr * 8);
            }
            KEY_PGDOWN => {
                self.cursor = (self.cursor + bpr * 8).min(self.data.len().saturating_sub(1));
            }
            _ => {}
        }
    }

    /// Handle mouse click at (x, y) relative to content area
    pub fn handle_click(&mut self, x: i32, y: i32, w: u32, h: u32) {
        let cw = char_w();
        let lh = char_h() + 1;
        if lh <= 0 || cw <= 0 { return; }

        // Skip header line
        let list_y = lh + 2;
        if y < list_y { return; }

        let row = ((y - list_y) / lh) as usize;
        let target_row = self.scroll + row;

        // Hex area starts after "XXXXXXXX  " (10 chars)
        let hex_start_x = 10 * cw;
        // Each hex byte = 3 chars ("XX "), after 8 bytes there's an extra space
        if x >= hex_start_x {
            let rel = (x - hex_start_x) as usize;
            let char_pos = rel / cw as usize;
            // Account for the extra space at position 24 (after 8 bytes * 3 chars)
            let byte_in_row = if char_pos >= 25 {
                (char_pos - 1) / 3
            } else {
                char_pos / 3
            };
            let byte_idx = target_row * self.bytes_per_row + byte_in_row.min(self.bytes_per_row - 1);
            if byte_idx < self.data.len() {
                self.cursor = byte_idx;
            }
        }
    }
}

/// Draw the hex editor panel
pub fn draw(state: &HexEditorState, x: i32, y: i32, w: u32, h: u32) {
    let cw = char_w();
    let lh = char_h() + 1;
    if lh <= 0 || cw <= 0 { return; }

    // Header: file path + size
    let header = if state.file_path.is_empty() {
        String::from("No file loaded — use: hex <path>")
    } else {
        format!("{} ({} bytes)", state.file_path, state.data.len())
    };
    draw_lab_text(x, y, &header, COL_ACCENT);

    // Offset info on right
    let off_info = format!("@{:04X}", state.cursor);
    let off_x = x + w as i32 - (off_info.len() as i32 * cw) - 2;
    draw_lab_text(off_x, y, &off_info, COL_DIM);

    let list_y = y + lh + 2;
    let list_h = h as i32 - lh - 2;
    if list_h <= 0 { return; }

    let visible_rows = (list_h / lh) as usize;
    if state.data.is_empty() {
        draw_lab_text(x + 4, list_y, "Empty — type 'hex <path>' to load a file", COL_DIM);
        return;
    }

    let bpr = state.bytes_per_row;
    let total_rows = (state.data.len() + bpr - 1) / bpr;

    // Auto-scroll to cursor
    let cursor_row = state.cursor / bpr;
    let scroll = if cursor_row >= state.scroll + visible_rows {
        cursor_row - visible_rows + 1
    } else if cursor_row < state.scroll {
        cursor_row
    } else {
        state.scroll
    };

    let end_row = (scroll + visible_rows).min(total_rows);
    let mut cy = list_y;

    // Column header
    let mut hdr = String::from("Offset    ");
    for i in 0..bpr.min(16) {
        if i == 8 { hdr.push(' '); }
        hdr.push_str(&format!("{:02X} ", i));
    }
    hdr.push_str(" ASCII");
    draw_lab_text(x, cy, &hdr, COL_DIM);
    cy += lh;

    for row in scroll..end_row {
        let offset = row * bpr;
        // Offset column
        let off_str = format!("{:08X}  ", offset);
        draw_lab_text(x, cy, &off_str, COL_DIM);

        let hex_x = x + 10 * cw;
        let mut hx = hex_x;

        // Hex bytes
        let row_end = (offset + bpr).min(state.data.len());
        for i in offset..row_end {
            if (i - offset) == 8 { hx += cw; } // extra gap at midpoint

            let b = state.data[i];
            let col = if i == state.cursor { COL_TEXT } else { byte_color(b) };

            // Highlight cursor byte
            if i == state.cursor {
                crate::framebuffer::fill_rect(
                    hx as u32, cy as u32, (2 * cw + 1) as u32, lh as u32,
                    0xFF1F6FEB,
                );
            }

            let hex = format!("{:02X}", b);
            draw_lab_text(hx, cy, &hex, col);
            hx += 3 * cw;
        }

        // ASCII section
        let ascii_x = hex_x + (bpr as i32 * 3 + 2) * cw;

        // Separator bar
        draw_lab_text(ascii_x - 2 * cw, cy, "|", COL_DIM);

        let mut ax = ascii_x;
        for i in offset..row_end {
            let b = state.data[i];
            let ch_str = alloc::format!("{}", byte_ascii(b));
            let col = if i == state.cursor { COL_TEXT } else { byte_color(b) };
            draw_lab_text(ax, cy, &ch_str, col);
            ax += cw;
        }

        cy += lh;
        if cy > y + h as i32 { break; }
    }

    // Scrollbar
    if total_rows > visible_rows {
        let track_h = list_h;
        let thumb_h = ((visible_rows as i32 * track_h) / total_rows as i32).max(8);
        let thumb_pos = (scroll as i32 * (track_h - thumb_h)) / total_rows.saturating_sub(1).max(1) as i32;
        let sb_x = (x + w as i32 - 3) as u32;
        crate::framebuffer::fill_rect(sb_x, (list_y) as u32, 2, track_h as u32, 0xFF21262D);
        crate::framebuffer::fill_rect(sb_x, (list_y + thumb_pos) as u32, 2, thumb_h as u32, COL_ACCENT);
    }

    // Color legend at bottom if space
    let legend_y = cy + 2;
    if legend_y + lh < y + h as i32 {
        crate::framebuffer::fill_rect(x as u32, legend_y as u32, w, 1, 0xFF30363D);
        let mut lx = x;
        let items: &[(&str, u32)] = &[
            ("ASCII", COL_GREEN), ("WS", COL_CYAN), ("CTRL", COL_YELLOW),
            ("HIGH", COL_PURPLE), ("NULL", COL_DIM), ("0xFF", COL_RED),
        ];
        for (label, color) in items {
            // Color dot
            crate::framebuffer::fill_rect(lx as u32, (legend_y + 3) as u32, 6, 6, *color);
            lx += cw;
            draw_lab_text(lx, legend_y + 2, label, *color);
            lx += (label.len() as i32 + 1) * cw;
            if lx > x + w as i32 - 10 { break; }
        }
    }
}
