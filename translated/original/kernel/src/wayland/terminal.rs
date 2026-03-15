//! TrustOS Graphical Terminal
//!
//! A native graphical terminal emulator using the Wayland compositor.
//! Inspired by Smithay's design but built from scratch for TrustOS.
//!
//! ## Design: Matrix-Style Terminal
//! - Green text on deep black background
//! - Phosphor glow effects
//! - VT100/ANSI escape code support
//! - Scrollback buffer
//! - Unicode support (basic)
//!
//! ## Architecture
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    GraphicsTerminal                         │
//! ├─────────────────────────────────────────────────────────────┤
//! │  Character Grid (cols × rows)                               │
//! │  Cursor position, blink state                               │
//! │  Scrollback buffer                                          │
//! ├─────────────────────────────────────────────────────────────┤
//! │  VT100 Parser (escape sequences)                            │
//! ├─────────────────────────────────────────────────────────────┤
//! │  Wayland Surface (buffer rendering)                         │
//! └─────────────────────────────────────────────────────────────┘
//! ```

use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use alloc::format;
use alloc::collections::VecDeque;

// ═══════════════════════════════════════════════════════════════════════════════
// 🎨 MATRIX TERMINAL PALETTE
// ═══════════════════════════════════════════════════════════════════════════════

/// Deep black background (RGB)
const BG_BLACK: u32 = 0xFF050606;
/// Bright green text (main)
const FG_GREEN_BRIGHT: u32 = 0xFF00FF66;
/// Normal green text
const FG_GREEN: u32 = 0xFF00CC55;
/// Dim green (secondary)
const FG_GREEN_DIM: u32 = 0xFF00AA44;
/// Muted green (subtle)
const FG_GREEN_MUTED: u32 = 0xFF008844;
/// Ghost green (very faint - for glow effect)
const FG_GREEN_GHOST: u32 = 0xFF003B1A;

/// Cursor color (bright phosphor)
const CURSOR_COLOR: u32 = 0xFF00FF88;
/// Selection highlight
const SELECTION_BG: u32 = 0xFF1A3A2A;

/// ANSI color palette (Matrix-ified)
const ANSI_COLORS: [u32; 16] = [
    0xFF050606, // 0: Black
    0xFF882222, // 1: Red (dim)
    0xFF00CC55, // 2: Green (Matrix)
    0xFF888822, // 3: Yellow (dim amber)
    0xFF4466AA, // 4: Blue (muted)
    0xFF884488, // 5: Magenta (dim)
    0xFF448888, // 6: Cyan (muted)
    0xFFAAAAAA, // 7: White (gray)
    0xFF666666, // 8: Bright black (gray)
    0xFFFF5555, // 9: Bright red
    0xFF00FF66, // 10: Bright green (Matrix bright)
    0xFFFFFF00, // 11: Bright yellow
    0xFF6688CC, // 12: Bright blue
    0xFFCC66CC, // 13: Bright magenta
    0xFF66CCCC, // 14: Bright cyan
    0xFFE0E8E4, // 15: Bright white (off-white)
];

// ═══════════════════════════════════════════════════════════════════════════════
// TERMINAL CHARACTER CELL
// ═══════════════════════════════════════════════════════════════════════════════

/// A single character cell in the terminal grid
#[derive(Debug, Clone, Copy)]
pub struct Cell {
    /// Unicode character
    pub ch: char,
    /// Foreground color (ARGB)
    pub fg: u32,
    /// Background color (ARGB)
    pub bg: u32,
    /// Cell attributes
    pub attr: CellAttr,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            ch: ' ',
            fg: FG_GREEN,
            bg: BG_BLACK,
            attr: CellAttr::default(),
        }
    }
}

/// Cell display attributes
#[derive(Debug, Clone, Copy, Default)]
pub struct CellAttr {
    pub bold: bool,
    pub dim: bool,
    pub italic: bool,
    pub underline: bool,
    pub blink: bool,
    pub reverse: bool,
    pub hidden: bool,
    pub strikethrough: bool,
}

// ═══════════════════════════════════════════════════════════════════════════════
// VT100/ANSI ESCAPE SEQUENCE PARSER
// ═══════════════════════════════════════════════════════════════════════════════

/// Parser state for escape sequences
#[derive(Debug, Clone, Copy, PartialEq)]
enum ParseState {
    /// Normal character input
    Normal,
    /// Received ESC (0x1B)
    Escape,
    /// CSI sequence: ESC [
    Csi,
    /// OSC sequence: ESC ]
    Osc,
    /// DCS sequence: ESC P
    Dcs,
}

/// Parsed ANSI command
#[derive(Debug, Clone)]
pub enum AnsiCommand {
    /// Print a character
    Print(char),
    /// Move cursor up N rows
    CursorUp(u16),
    /// Move cursor down N rows
    CursorDown(u16),
    /// Move cursor right N columns
    CursorRight(u16),
    /// Move cursor left N columns
    CursorLeft(u16),
    /// Move cursor to row, column (1-indexed)
    CursorPosition(u16, u16),
    /// Save cursor position
    SaveCursor,
    /// Restore cursor position
    RestoreCursor,
    /// Erase from cursor to end of screen
    EraseToEnd,
    /// Erase from start to cursor
    EraseToStart,
    /// Erase entire screen
    EraseScreen,
    /// Erase from cursor to end of line
    EraseLineToEnd,
    /// Erase from start to cursor on line
    EraseLineToStart,
    /// Erase entire line
    EraseLine,
    /// Set graphics rendition (colors, attributes)
    Sgr(Vec<u16>),
    /// Scroll up N lines
    ScrollUp(u16),
    /// Scroll down N lines
    ScrollDown(u16),
    /// Set window title
    SetTitle(String),
    /// Bell (beep)
    Bell,
    /// Backspace
    Backspace,
    /// Tab
    Tab,
    /// Newline
    Newline,
    /// Carriage return
    CarriageReturn,
    /// Show cursor
    ShowCursor,
    /// Hide cursor
    HideCursor,
    /// Unknown/ignored sequence
    Unknown,
}

// ═══════════════════════════════════════════════════════════════════════════════
// GRAPHICS TERMINAL
// ═══════════════════════════════════════════════════════════════════════════════

/// The graphical terminal emulator
pub struct GraphicsTerminal {
    /// Terminal dimensions in characters
    pub cols: u16,
    pub rows: u16,
    
    /// Character cell size in pixels
    pub cell_width: u16,
    pub cell_height: u16,
    
    /// Current cursor position
    pub cursor_x: u16,
    pub cursor_y: u16,
    
    /// Saved cursor position
    saved_cursor_x: u16,
    saved_cursor_y: u16,
    
    /// Is cursor visible?
    pub cursor_visible: bool,
    
    /// Cursor blink state
    cursor_blink: bool,
    blink_counter: u32,
    
    /// Character grid (current screen)
    grid: Vec<Cell>,
    
    /// Scrollback buffer (previous lines)
    scrollback: VecDeque<Vec<Cell>>,
    scrollback_max: usize,
    
    /// Current scroll offset (0 = bottom, showing current screen)
    scroll_offset: usize,
    
    /// Current text attributes
    current_attr: CellAttr,
    current_fg: u32,
    current_bg: u32,
    
    /// Parse state for escape sequences
    parse_state: ParseState,
    csi_params: Vec<u16>,
    csi_buffer: String,
    osc_buffer: String,
    
    /// Window title
    pub title: String,
    
    /// Wayland surface ID
    pub surface_id: Option<u32>,
    
    /// Dirty flag (needs redraw)
    dirty: bool,
    
    /// Enable phosphor glow effect
    pub glow_enabled: bool,
    
    /// Scanline effect intensity (0-255)
    pub scanline_intensity: u8,
}

impl GraphicsTerminal {
    /// Create a new graphical terminal
    pub fn new(width: u32, height: u32) -> Self {
        // Calculate dimensions based on 8x16 font
        let cell_width = 8u16;
        let cell_height = 16u16;
        let cols = (width / cell_width as u32) as u16;
        let rows = (height / cell_height as u32) as u16;
        
        let grid_size = (cols as usize) * (rows as usize);
        let grid = vec![Cell::default(); grid_size];
        
        let mut term = Self {
            cols,
            rows,
            cell_width,
            cell_height,
            cursor_x: 0,
            cursor_y: 0,
            saved_cursor_x: 0,
            saved_cursor_y: 0,
            cursor_visible: true,
            cursor_blink: true,
            blink_counter: 0,
            grid,
            scrollback: VecDeque::new(),
            scrollback_max: 1000,
            scroll_offset: 0,
            current_attr: CellAttr::default(),
            current_fg: FG_GREEN,
            current_bg: BG_BLACK,
            parse_state: ParseState::Normal,
            csi_params: Vec::new(),
            csi_buffer: String::new(),
            osc_buffer: String::new(),
            title: String::from("TrustOS Terminal"),
            surface_id: None,
            dirty: true,
            glow_enabled: true,
            scanline_intensity: 20,
        };
        
        // Print welcome message
        term.write_str("\x1b[1;32m"); // Bold green
        term.write_str("╔══════════════════════════════════════════════════════════╗\r\n");
        term.write_str("║  \x1b[1;97mTrustOS\x1b[1;32m Graphical Terminal                             ║\r\n");
        term.write_str("║  Matrix Edition v1.0                                     ║\r\n");
        term.write_str("╚══════════════════════════════════════════════════════════╝\r\n");
        term.write_str("\x1b[0;32m"); // Normal green
        term.write_str("\r\n");
        
        term
    }
    
    /// Write a string to the terminal
    pub fn write_str(&mut self, s: &str) {
        for c in s.chars() {
            self.write_char(c);
        }
    }
    
    /// Write a single character, handling escape sequences
    pub fn write_char(&mut self, c: char) {
        match self.parse_state {
            ParseState::Normal => self.handle_normal(c),
            ParseState::Escape => self.handle_escape(c),
            ParseState::Csi => self.handle_csi(c),
            ParseState::Osc => self.handle_osc(c),
            ParseState::Dcs => self.handle_dcs(c),
        }
        self.dirty = true;
    }
    
    fn handle_normal(&mut self, c: char) {
        match c {
            '\x1b' => {
                self.parse_state = ParseState::Escape;
            }
            '\n' => {
                self.newline();
            }
            '\r' => {
                self.cursor_x = 0;
            }
            '\x08' => {
                // Backspace
                if self.cursor_x > 0 {
                    self.cursor_x -= 1;
                }
            }
            '\t' => {
                // Tab: move to next 8-column boundary
                let next_tab = ((self.cursor_x / 8) + 1) * 8;
                self.cursor_x = next_tab.min(self.cols - 1);
            }
            '\x07' => {
                // Bell - could trigger visual bell
            }
            _ if c >= ' ' => {
                self.put_char(c);
            }
            _ => {
                // Ignore other control characters
            }
        }
    }
    
    fn handle_escape(&mut self, c: char) {
        match c {
            '[' => {
                self.parse_state = ParseState::Csi;
                self.csi_params.clear();
                self.csi_buffer.clear();
            }
            ']' => {
                self.parse_state = ParseState::Osc;
                self.osc_buffer.clear();
            }
            'P' => {
                self.parse_state = ParseState::Dcs;
            }
            'c' => {
                // Full reset
                self.reset();
                self.parse_state = ParseState::Normal;
            }
            '7' => {
                // Save cursor
                self.saved_cursor_x = self.cursor_x;
                self.saved_cursor_y = self.cursor_y;
                self.parse_state = ParseState::Normal;
            }
            '8' => {
                // Restore cursor
                self.cursor_x = self.saved_cursor_x;
                self.cursor_y = self.saved_cursor_y;
                self.parse_state = ParseState::Normal;
            }
            'D' => {
                // Index (move down one line, scroll if needed)
                self.newline();
                self.parse_state = ParseState::Normal;
            }
            'M' => {
                // Reverse index (move up one line)
                if self.cursor_y > 0 {
                    self.cursor_y -= 1;
                }
                self.parse_state = ParseState::Normal;
            }
            'E' => {
                // Next line
                self.cursor_x = 0;
                self.newline();
                self.parse_state = ParseState::Normal;
            }
            _ => {
                // Unknown escape, go back to normal
                self.parse_state = ParseState::Normal;
            }
        }
    }
    
    fn handle_csi(&mut self, c: char) {
        if c.is_ascii_digit() || c == ';' {
            self.csi_buffer.push(c);
        } else {
            // Parse parameters
            self.csi_params.clear();
            for part in self.csi_buffer.split(';') {
                if let Ok(n) = part.parse::<u16>() {
                    self.csi_params.push(n);
                } else {
                    self.csi_params.push(0);
                }
            }
            
            // Execute CSI command
            self.execute_csi(c);
            self.parse_state = ParseState::Normal;
        }
    }
    
    fn execute_csi(&mut self, cmd: char) {
        let params = &self.csi_params;
        let p0 = params.first().copied().unwrap_or(1).max(1);
        let p1 = params.get(1).copied().unwrap_or(1).max(1);
        
        match cmd {
            'A' => {
                // Cursor up
                self.cursor_y = self.cursor_y.saturating_sub(p0);
            }
            'B' => {
                // Cursor down
                self.cursor_y = (self.cursor_y + p0).min(self.rows - 1);
            }
            'C' => {
                // Cursor right
                self.cursor_x = (self.cursor_x + p0).min(self.cols - 1);
            }
            'D' => {
                // Cursor left
                self.cursor_x = self.cursor_x.saturating_sub(p0);
            }
            'E' => {
                // Cursor next line
                self.cursor_x = 0;
                self.cursor_y = (self.cursor_y + p0).min(self.rows - 1);
            }
            'F' => {
                // Cursor previous line
                self.cursor_x = 0;
                self.cursor_y = self.cursor_y.saturating_sub(p0);
            }
            'G' => {
                // Cursor horizontal absolute
                self.cursor_x = (p0 - 1).min(self.cols - 1);
            }
            'H' | 'f' => {
                // Cursor position (row, col)
                let row = params.first().copied().unwrap_or(1).max(1);
                let col = params.get(1).copied().unwrap_or(1).max(1);
                self.cursor_y = (row - 1).min(self.rows - 1);
                self.cursor_x = (col - 1).min(self.cols - 1);
            }
            'J' => {
                // Erase in display
                let mode = params.first().copied().unwrap_or(0);
                match mode {
                    0 => self.erase_to_end_of_screen(),
                    1 => self.erase_to_start_of_screen(),
                    2 | 3 => self.erase_screen(),
                    _ => {}
                }
            }
            'K' => {
                // Erase in line
                let mode = params.first().copied().unwrap_or(0);
                match mode {
                    0 => self.erase_to_end_of_line(),
                    1 => self.erase_to_start_of_line(),
                    2 => self.erase_line(),
                    _ => {}
                }
            }
            'S' => {
                // Scroll up
                for _ in 0..p0 {
                    self.scroll_up();
                }
            }
            'T' => {
                // Scroll down
                for _ in 0..p0 {
                    self.scroll_down();
                }
            }
            'm' => {
                // SGR - Select Graphic Rendition
                self.execute_sgr();
            }
            's' => {
                // Save cursor position
                self.saved_cursor_x = self.cursor_x;
                self.saved_cursor_y = self.cursor_y;
            }
            'u' => {
                // Restore cursor position
                self.cursor_x = self.saved_cursor_x;
                self.cursor_y = self.saved_cursor_y;
            }
            '?' if !params.is_empty() => {
                // Private mode (handled in csi_buffer)
            }
            'h' => {
                // Set mode
                if self.csi_buffer.starts_with('?') {
                    let mode = params.first().copied().unwrap_or(0);
                    if mode == 25 {
                        self.cursor_visible = true;
                    }
                }
            }
            'l' => {
                // Reset mode
                if self.csi_buffer.starts_with('?') {
                    let mode = params.first().copied().unwrap_or(0);
                    if mode == 25 {
                        self.cursor_visible = false;
                    }
                }
            }
            _ => {
                // Unknown CSI command
            }
        }
    }
    
    fn execute_sgr(&mut self) {
        let params = if self.csi_params.is_empty() {
            vec![0] // Default to reset
        } else {
            self.csi_params.clone()
        };
        
        let mut i = 0;
        while i < params.len() {
            let p = params[i];
            match p {
                0 => {
                    // Reset all attributes
                    self.current_attr = CellAttr::default();
                    self.current_fg = FG_GREEN;
                    self.current_bg = BG_BLACK;
                }
                1 => self.current_attr.bold = true,
                2 => self.current_attr.dim = true,
                3 => self.current_attr.italic = true,
                4 => self.current_attr.underline = true,
                5 => self.current_attr.blink = true,
                7 => self.current_attr.reverse = true,
                8 => self.current_attr.hidden = true,
                9 => self.current_attr.strikethrough = true,
                21 => self.current_attr.bold = false,
                22 => {
                    self.current_attr.bold = false;
                    self.current_attr.dim = false;
                }
                23 => self.current_attr.italic = false,
                24 => self.current_attr.underline = false,
                25 => self.current_attr.blink = false,
                27 => self.current_attr.reverse = false,
                28 => self.current_attr.hidden = false,
                29 => self.current_attr.strikethrough = false,
                // Foreground colors
                30..=37 => {
                    let idx = (p - 30) as usize;
                    self.current_fg = ANSI_COLORS[idx];
                }
                38 => {
                    // Extended foreground
                    if i + 2 < params.len() && params[i + 1] == 5 {
                        // 256-color mode
                        let idx = params[i + 2] as usize;
                        self.current_fg = self.color_256(idx);
                        i += 2;
                    } else if i + 4 < params.len() && params[i + 1] == 2 {
                        // 24-bit RGB
                        let r = params[i + 2] as u32;
                        let g = params[i + 3] as u32;
                        let b = params[i + 4] as u32;
                        self.current_fg = 0xFF000000 | (r << 16) | (g << 8) | b;
                        i += 4;
                    }
                }
                39 => self.current_fg = FG_GREEN, // Default foreground
                // Background colors
                40..=47 => {
                    let idx = (p - 40) as usize;
                    self.current_bg = ANSI_COLORS[idx];
                }
                48 => {
                    // Extended background
                    if i + 2 < params.len() && params[i + 1] == 5 {
                        let idx = params[i + 2] as usize;
                        self.current_bg = self.color_256(idx);
                        i += 2;
                    } else if i + 4 < params.len() && params[i + 1] == 2 {
                        let r = params[i + 2] as u32;
                        let g = params[i + 3] as u32;
                        let b = params[i + 4] as u32;
                        self.current_bg = 0xFF000000 | (r << 16) | (g << 8) | b;
                        i += 4;
                    }
                }
                49 => self.current_bg = BG_BLACK, // Default background
                // Bright foreground colors
                90..=97 => {
                    let idx = (p - 90 + 8) as usize;
                    self.current_fg = ANSI_COLORS[idx];
                }
                // Bright background colors
                100..=107 => {
                    let idx = (p - 100 + 8) as usize;
                    self.current_bg = ANSI_COLORS[idx];
                }
                _ => {}
            }
            i += 1;
        }
    }
    
    /// Convert 256-color index to RGB
    fn color_256(&self, idx: usize) -> u32 {
        if idx < 16 {
            ANSI_COLORS[idx]
        } else if idx < 232 {
            // 6x6x6 color cube
            let idx = idx - 16;
            let r = (idx / 36) % 6;
            let g = (idx / 6) % 6;
            let b = idx % 6;
            let r = if r > 0 { r * 40 + 55 } else { 0 };
            let g = if g > 0 { g * 40 + 55 } else { 0 };
            let b = if b > 0 { b * 40 + 55 } else { 0 };
            0xFF000000 | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
        } else {
            // Grayscale ramp
            let gray = (idx - 232) * 10 + 8;
            0xFF000000 | ((gray as u32) << 16) | ((gray as u32) << 8) | (gray as u32)
        }
    }
    
    fn handle_osc(&mut self, c: char) {
        if c == '\x07' || c == '\x1b' {
            // End of OSC sequence
            self.execute_osc();
            self.parse_state = ParseState::Normal;
        } else {
            self.osc_buffer.push(c);
        }
    }
    
    fn execute_osc(&mut self) {
        // Parse OSC command
        if let Some(idx) = self.osc_buffer.find(';') {
            let cmd = &self.osc_buffer[..idx];
            let data = &self.osc_buffer[idx + 1..];
            
            match cmd {
                "0" | "2" => {
                    // Set window title
                    self.title = String::from(data);
                }
                _ => {}
            }
        }
    }
    
    fn handle_dcs(&mut self, c: char) {
        // DCS sequences - ignore for now
        if c == '\x1b' || c == '\\' {
            self.parse_state = ParseState::Normal;
        }
    }
    
    /// Put a character at current cursor position
    fn put_char(&mut self, c: char) {
        if self.cursor_x >= self.cols {
            self.cursor_x = 0;
            self.newline();
        }
        
        let idx = self.cursor_y as usize * self.cols as usize + self.cursor_x as usize;
        if idx < self.grid.len() {
            // Apply current attributes
            let (fg, bg) = if self.current_attr.reverse {
                (self.current_bg, self.current_fg)
            } else {
                (self.current_fg, self.current_bg)
            };
            
            let fg = if self.current_attr.bold {
                self.brighten(fg)
            } else if self.current_attr.dim {
                self.dim_color(fg)
            } else {
                fg
            };
            
            self.grid[idx] = Cell {
                ch: c,
                fg,
                bg,
                attr: self.current_attr,
            };
        }
        
        self.cursor_x += 1;
    }
    
    /// Brighten a color (for bold text)
    fn brighten(&self, color: u32) -> u32 {
        let r = ((color >> 16) & 0xFF).min(255);
        let g = ((color >> 8) & 0xFF).min(255);
        let b = (color & 0xFF).min(255);
        
        let r = (r + (255 - r) / 3).min(255);
        let g = (g + (255 - g) / 3).min(255);
        let b = (b + (255 - b) / 3).min(255);
        
        0xFF000000 | (r << 16) | (g << 8) | b
    }
    
    /// Dim a color
    fn dim_color(&self, color: u32) -> u32 {
        let r = ((color >> 16) & 0xFF) * 2 / 3;
        let g = ((color >> 8) & 0xFF) * 2 / 3;
        let b = (color & 0xFF) * 2 / 3;
        
        0xFF000000 | (r << 16) | (g << 8) | b
    }
    
    /// Handle newline
    fn newline(&mut self) {
        if self.cursor_y >= self.rows - 1 {
            self.scroll_up();
        } else {
            self.cursor_y += 1;
        }
    }
    
    /// Scroll the screen up by one line
    fn scroll_up(&mut self) {
        // Save top line to scrollback
        let top_line: Vec<Cell> = self.grid[..self.cols as usize].to_vec();
        self.scrollback.push_back(top_line);
        
        // Trim scrollback if needed
        while self.scrollback.len() > self.scrollback_max {
            self.scrollback.pop_front();
        }
        
        // Shift all lines up
        let cols = self.cols as usize;
        for y in 0..self.rows as usize - 1 {
            let src_start = (y + 1) * cols;
            let dst_start = y * cols;
            for x in 0..cols {
                self.grid[dst_start + x] = self.grid[src_start + x];
            }
        }
        
        // Clear bottom line
        let last_row_start = (self.rows as usize - 1) * cols;
        for x in 0..cols {
            self.grid[last_row_start + x] = Cell::default();
        }
    }
    
    /// Scroll the screen down by one line
    fn scroll_down(&mut self) {
        let cols = self.cols as usize;
        
        // Shift all lines down
        for y in (1..self.rows as usize).rev() {
            let src_start = (y - 1) * cols;
            let dst_start = y * cols;
            for x in 0..cols {
                self.grid[dst_start + x] = self.grid[src_start + x];
            }
        }
        
        // Clear top line
        for x in 0..cols {
            self.grid[x] = Cell::default();
        }
    }
    
    /// Erase from cursor to end of screen
    fn erase_to_end_of_screen(&mut self) {
        // Erase rest of current line
        self.erase_to_end_of_line();
        
        // Erase all lines below
        let cols = self.cols as usize;
        for y in (self.cursor_y + 1) as usize..self.rows as usize {
            let row_start = y * cols;
            for x in 0..cols {
                self.grid[row_start + x] = Cell::default();
            }
        }
    }
    
    /// Erase from start of screen to cursor
    fn erase_to_start_of_screen(&mut self) {
        // Erase start of current line
        self.erase_to_start_of_line();
        
        // Erase all lines above
        let cols = self.cols as usize;
        for y in 0..self.cursor_y as usize {
            let row_start = y * cols;
            for x in 0..cols {
                self.grid[row_start + x] = Cell::default();
            }
        }
    }
    
    /// Erase entire screen
    fn erase_screen(&mut self) {
        for cell in &mut self.grid {
            *cell = Cell::default();
        }
    }
    
    /// Erase from cursor to end of line
    fn erase_to_end_of_line(&mut self) {
        let row_start = self.cursor_y as usize * self.cols as usize;
        for x in self.cursor_x as usize..self.cols as usize {
            self.grid[row_start + x] = Cell::default();
        }
    }
    
    /// Erase from start of line to cursor
    fn erase_to_start_of_line(&mut self) {
        let row_start = self.cursor_y as usize * self.cols as usize;
        for x in 0..=self.cursor_x as usize {
            if row_start + x < self.grid.len() {
                self.grid[row_start + x] = Cell::default();
            }
        }
    }
    
    /// Erase entire current line
    fn erase_line(&mut self) {
        let row_start = self.cursor_y as usize * self.cols as usize;
        for x in 0..self.cols as usize {
            self.grid[row_start + x] = Cell::default();
        }
    }
    
    /// Reset terminal to initial state
    pub fn reset(&mut self) {
        self.cursor_x = 0;
        self.cursor_y = 0;
        self.current_attr = CellAttr::default();
        self.current_fg = FG_GREEN;
        self.current_bg = BG_BLACK;
        self.cursor_visible = true;
        self.erase_screen();
    }
    
    /// Handle keyboard input
    pub fn handle_key(&mut self, c: char) {
        // Echo character and add to input buffer
        // In a real terminal, this would be sent to the shell
        if c == '\n' {
            self.write_str("\r\n");
        } else {
            self.write_char(c);
        }
    }
    
    /// Render terminal to a pixel buffer (ARGB) - SSE2 optimized
    pub fn render(&mut self) -> Vec<u32> {
        let width = self.cols as usize * self.cell_width as usize;
        let height = self.rows as usize * self.cell_height as usize;
        
        // SSE2 optimized buffer clear
        let mut buffer = vec![0u32; width * height];
        crate::graphics::simd::fill_buffer_fast(&mut buffer, BG_BLACK);
        
        // Update cursor blink
        self.blink_counter = self.blink_counter.wrapping_add(1);
        if self.blink_counter % 30 == 0 {
            self.cursor_blink = !self.cursor_blink;
        }
        
        // Draw all cells with SSE2 row fills
        for y in 0..self.rows as usize {
            for x in 0..self.cols as usize {
                let idx = y * self.cols as usize + x;
                let cell = &self.grid[idx];
                
                // Draw cell background with SSE2
                self.draw_cell_bg_fast(&mut buffer, width, x as u32, y as u32, cell.bg);
                
                // Draw character
                if cell.ch != ' ' {
                    self.draw_char(&mut buffer, width as u32, x as u32, y as u32, cell.ch, cell.fg);
                }
                
                // Draw underline if needed
                if cell.attr.underline {
                    self.draw_underline_fast(&mut buffer, width, x as u32, y as u32, cell.fg);
                }
            }
        }
        
        // Draw cursor
        if self.cursor_visible && self.cursor_blink {
            self.draw_cursor_fast(&mut buffer, width);
        }
        
        // Apply phosphor glow effect
        if self.glow_enabled {
            self.apply_glow(&mut buffer, width as u32, height as u32);
        }
        
        // Apply scanline effect
        if self.scanline_intensity > 0 {
            self.apply_scanlines(&mut buffer, width as u32, height as u32);
        }
        
        self.dirty = false;
        buffer
    }
    
    /// Fast cell background using SSE2 row fills
    fn draw_cell_bg_fast(&self, buffer: &mut [u32], width: usize, cx: u32, cy: u32, bg: u32) {
        let px_x = cx as usize * self.cell_width as usize;
        let px_y = cy as usize * self.cell_height as usize;
        let cell_w = self.cell_width as usize;
        
        // Fill each row of the cell with SSE2
        for dy in 0..self.cell_height as usize {
            let row_start = (px_y + dy) * width + px_x;
            if row_start + cell_w <= buffer.len() {
                #[cfg(target_arch = "x86_64")]
                unsafe {
                    crate::graphics::simd::fill_row_sse2(
                        buffer.as_mut_ptr().add(row_start),
                        cell_w,
                        bg
                    );
                }
                #[cfg(not(target_arch = "x86_64"))]
                {
                    buffer[row_start..row_start + cell_w].fill(bg);
                }
            }
        }
    }

    fn draw_cell_bg(&self, buffer: &mut [u32], width: u32, cx: u32, cy: u32, bg: u32) {
        let px_x = cx * self.cell_width as u32;
        let px_y = cy * self.cell_height as u32;
        
        for dy in 0..self.cell_height as u32 {
            for dx in 0..self.cell_width as u32 {
                let idx = ((px_y + dy) * width + px_x + dx) as usize;
                if idx < buffer.len() {
                    buffer[idx] = bg;
                }
            }
        }
    }
    
    fn draw_char(&self, buffer: &mut [u32], width: u32, cx: u32, cy: u32, c: char, fg: u32) {
        let glyph = crate::framebuffer::font::get_glyph(c);
        let px_x = cx * self.cell_width as u32;
        let px_y = cy * self.cell_height as u32;
        
        for (row_idx, &row) in glyph.iter().enumerate() {
            for bit in 0..8 {
                if (row >> (7 - bit)) & 1 == 1 {
                    let x = px_x + bit;
                    let y = px_y + row_idx as u32;
                    let idx = (y * width + x) as usize;
                    if idx < buffer.len() {
                        buffer[idx] = fg;
                    }
                }
            }
        }
    }
    
    fn draw_underline(&self, buffer: &mut [u32], width: u32, cx: u32, cy: u32, fg: u32) {
        let px_x = cx * self.cell_width as u32;
        let px_y = cy * self.cell_height as u32 + self.cell_height as u32 - 2;
        
        for dx in 0..self.cell_width as u32 {
            let idx = (px_y * width + px_x + dx) as usize;
            if idx < buffer.len() {
                buffer[idx] = fg;
            }
        }
    }
    
    /// Fast underline using SSE2
    fn draw_underline_fast(&self, buffer: &mut [u32], width: usize, cx: u32, cy: u32, fg: u32) {
        let px_x = cx as usize * self.cell_width as usize;
        let px_y = cy as usize * self.cell_height as usize + self.cell_height as usize - 2;
        let cell_w = self.cell_width as usize;
        
        let start = px_y * width + px_x;
        if start + cell_w <= buffer.len() {
            #[cfg(target_arch = "x86_64")]
            unsafe {
                crate::graphics::simd::fill_row_sse2(
                    buffer.as_mut_ptr().add(start),
                    cell_w,
                    fg
                );
            }
            #[cfg(not(target_arch = "x86_64"))]
            {
                buffer[start..start + cell_w].fill(fg);
            }
        }
    }
    
    fn draw_cursor(&self, buffer: &mut [u32], width: u32) {
        let px_x = self.cursor_x as u32 * self.cell_width as u32;
        let px_y = self.cursor_y as u32 * self.cell_height as u32;
        
        // Block cursor
        for dy in 0..self.cell_height as u32 {
            for dx in 0..self.cell_width as u32 {
                let idx = ((px_y + dy) * width + px_x + dx) as usize;
                if idx < buffer.len() {
                    // XOR with cursor color for visibility
                    let existing = buffer[idx];
                    buffer[idx] = existing ^ CURSOR_COLOR;
                }
            }
        }
    }
    
    /// Fast cursor drawing using SSE2
    fn draw_cursor_fast(&self, buffer: &mut [u32], width: usize) {
        let px_x = self.cursor_x as usize * self.cell_width as usize;
        let px_y = self.cursor_y as usize * self.cell_height as usize;
        let cell_w = self.cell_width as usize;
        
        // Block cursor - XOR with cursor color
        for dy in 0..self.cell_height as usize {
            let row_start = (px_y + dy) * width + px_x;
            if row_start + cell_w <= buffer.len() {
                for dx in 0..cell_w {
                    let existing = buffer[row_start + dx];
                    buffer[row_start + dx] = existing ^ CURSOR_COLOR;
                }
            }
        }
    }
    
    /// Apply a subtle phosphor glow effect (Matrix style)
    fn apply_glow(&self, buffer: &mut [u32], width: u32, height: u32) {
        // Simple horizontal blur for glow effect
        // This is a simplified version - a real implementation would use
        // a proper bloom shader
        let mut temp = buffer.to_vec();
        
        for y in 0..height {
            for x in 1..(width - 1) {
                let idx = (y * width + x) as usize;
                let left = buffer[(y * width + x - 1) as usize];
                let center = buffer[idx];
                let right = buffer[(y * width + x + 1) as usize];
                
                // Only apply glow to green pixels
                let g_left = (left >> 8) & 0xFF;
                let g_center = (center >> 8) & 0xFF;
                let g_right = (right >> 8) & 0xFF;
                
                if g_center > 100 || g_left > 100 || g_right > 100 {
                    let glow = ((g_left + g_center * 2 + g_right) / 4).min(255);
                    let r = (center >> 16) & 0xFF;
                    let b = center & 0xFF;
                    temp[idx] = 0xFF000000 | (r << 16) | (glow << 8) | b;
                }
            }
        }
        
        buffer.copy_from_slice(&temp);
    }
    
    /// Apply CRT scanline effect (optimized)
    fn apply_scanlines(&self, buffer: &mut [u32], width: u32, height: u32) {
        let intensity = 255 - self.scanline_intensity as u32;
        
        for y in (1..height).step_by(2) {
            let row_start = (y * width) as usize;
            let row_end = ((y + 1) * width) as usize;
            
            if row_end <= buffer.len() {
                for idx in row_start..row_end.min(row_start + width as usize) {
                    let pixel = buffer[idx];
                    let r = ((pixel >> 16) & 0xFF) * intensity / 255;
                    let g = ((pixel >> 8) & 0xFF) * intensity / 255;
                    let b = (pixel & 0xFF) * intensity / 255;
                    buffer[idx] = 0xFF000000 | (r << 16) | (g << 8) | b;
                }
            }
        }
    }
    
    /// Check if terminal needs redraw
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }
    
    /// Get terminal dimensions in pixels
    pub fn pixel_size(&self) -> (u32, u32) {
        (
            self.cols as u32 * self.cell_width as u32,
            self.rows as u32 * self.cell_height as u32,
        )
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// GLOBAL TERMINAL INSTANCE
// ═══════════════════════════════════════════════════════════════════════════════

use spin::Mutex;

static GRAPHICS_TERMINAL: Mutex<Option<GraphicsTerminal>> = Mutex::new(None);

/// Initialize the graphical terminal
pub fn init(width: u32, height: u32) -> Result<(), &'static str> {
    let mut term = GRAPHICS_TERMINAL.lock();
    if term.is_some() {
        return Err("Graphics terminal already initialized");
    }
    
    *term = Some(GraphicsTerminal::new(width, height));
    crate::serial_println!("[GTERM] Graphics terminal initialized ({}x{})", width, height);
    Ok(())
}

/// Write to the graphical terminal
pub fn write(s: &str) {
    if let Some(term) = GRAPHICS_TERMINAL.lock().as_mut() {
        term.write_str(s);
    }
}

/// Render the terminal and get the pixel buffer
pub fn render() -> Option<Vec<u32>> {
    GRAPHICS_TERMINAL.lock().as_mut().map(|term| term.render())
}

/// Handle keyboard input
pub fn handle_key(c: char) {
    if let Some(term) = GRAPHICS_TERMINAL.lock().as_mut() {
        term.handle_key(c);
    }
}

/// Get terminal pixel dimensions
pub fn get_size() -> Option<(u32, u32)> {
    GRAPHICS_TERMINAL.lock().as_ref().map(|term| term.pixel_size())
}
