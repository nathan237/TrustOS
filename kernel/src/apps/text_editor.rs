//! TrustCode — VSCode-inspired Text Editor
//!
//! A native bare-metal code editor with:
//! - Cursor navigation (arrows, Home/End, PgUp/PgDn)
//! - Line editing (insert, delete, backspace)
//! - Syntax highlighting for Rust
//! - Line numbers with gutter
//! - Status bar (filename, line:col, mode, language)
//! - File save/load (Ctrl+S)
//! - Scroll support
//! - Selection (Shift+arrows) — future
//! - Minimap sidebar — future

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;

// ═══════════════════════════════════════════════════════════════════════════════
// EDITOR STATE
// ═══════════════════════════════════════════════════════════════════════════════

/// Editor state per-window (stored outside Window struct to avoid bloat)
#[derive(Clone)]
pub struct EditorState {
    /// All document lines (the real content)
    pub lines: Vec<String>,
    /// Cursor line (0-indexed)
    pub cursor_line: usize,
    /// Cursor column (0-indexed, byte position)
    pub cursor_col: usize,
    /// Scroll offset (first visible line)
    pub scroll_y: usize,
    /// Scroll horizontal offset
    pub scroll_x: usize,
    /// File path if loaded from file
    pub file_path: Option<String>,
    /// Dirty flag (unsaved changes)
    pub dirty: bool,
    /// Language mode for syntax highlighting
    pub language: Language,
    /// Whether editor is in command mode (Ctrl pressed)
    pub status_message: Option<String>,
    /// Frame counter for cursor blink
    pub blink_counter: u32,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Language {
    Plain,
    Rust,
    Toml,
    Markdown,
    C,
    Python,
    JavaScript,
}

impl Language {
    pub fn name(&self) -> &'static str {
        match self {
            Language::Plain => "Plain Text",
            Language::Rust => "Rust",
            Language::Toml => "TOML",
            Language::Markdown => "Markdown",
            Language::C => "C/C++",
            Language::Python => "Python",
            Language::JavaScript => "JavaScript",
        }
    }
    
    /// Detect language from file extension
    pub fn from_filename(name: &str) -> Self {
        if name.ends_with(".rs") { Language::Rust }
        else if name.ends_with(".toml") { Language::Toml }
        else if name.ends_with(".md") { Language::Markdown }
        else if name.ends_with(".c") || name.ends_with(".h") || name.ends_with(".cpp") { Language::C }
        else if name.ends_with(".py") { Language::Python }
        else if name.ends_with(".js") || name.ends_with(".ts") { Language::JavaScript }
        else { Language::Plain }
    }
}

impl EditorState {
    pub fn new() -> Self {
        Self {
            lines: alloc::vec![String::new()],
            cursor_line: 0,
            cursor_col: 0,
            scroll_y: 0,
            scroll_x: 0,
            file_path: None,
            dirty: false,
            language: Language::Plain,
            status_message: None,
            blink_counter: 0,
        }
    }
    
    /// Load content from string
    pub fn load_text(&mut self, text: &str) {
        self.lines.clear();
        for line in text.lines() {
            self.lines.push(String::from(line));
        }
        if self.lines.is_empty() {
            self.lines.push(String::new());
        }
        self.cursor_line = 0;
        self.cursor_col = 0;
        self.scroll_y = 0;
        self.dirty = false;
    }
    
    /// Load from file path
    pub fn load_file(&mut self, path: &str) {
        self.file_path = Some(String::from(path));
        self.language = Language::from_filename(path);
        
        let full_path = if path.starts_with('/') {
            String::from(path)
        } else {
            format!("/{}", path)
        };
        
        if let Ok(data) = crate::ramfs::with_fs(|fs| fs.read_file(&full_path).map(|d| d.to_vec())) {
            if let Ok(text) = core::str::from_utf8(&data) {
                self.load_text(text);
            } else {
                self.lines = alloc::vec![String::from("(binary file — cannot edit)")];
            }
        } else {
            // New file
            self.lines = alloc::vec![String::new()];
        }
        self.dirty = false;
    }
    
    /// Save to file
    pub fn save(&mut self) -> bool {
        if let Some(ref path) = self.file_path {
            let full_path = if path.starts_with('/') {
                String::from(path.as_str())
            } else {
                format!("/{}", path)
            };
            
            // Build full text
            let mut text = String::new();
            for (i, line) in self.lines.iter().enumerate() {
                text.push_str(line);
                if i + 1 < self.lines.len() {
                    text.push('\n');
                }
            }
            
            let result = crate::ramfs::with_fs(|fs| {
                // Create the file if it doesn't exist yet
                if fs.read_file(&full_path).is_err() {
                    let _ = fs.touch(&full_path);
                }
                fs.write_file(&full_path, text.as_bytes())
            });
            
            if result.is_ok() {
                self.dirty = false;
                self.status_message = Some(format!("Saved: {}", path));
                crate::serial_println!("[TrustCode] Saved: {}", path);
                // Also persist to disk via VFS (TrustFS) if available
                if let Ok(()) = crate::vfs::write_file(&full_path, text.as_bytes()) {
                    crate::serial_println!("[TrustCode] Persisted to disk: {}", path);
                }
                return true;
            } else {
                self.status_message = Some(format!("ERROR: Could not save {}", path));
            }
        } else {
            self.status_message = Some(String::from("No file path — use Ctrl+Shift+S"));
        }
        false
    }
    
    /// Get current line content
    fn current_line(&self) -> &str {
        if self.cursor_line < self.lines.len() {
            &self.lines[self.cursor_line]
        } else {
            ""
        }
    }
    
    /// Clamp cursor col to current line length
    fn clamp_cursor_col(&mut self) {
        let len = if self.cursor_line < self.lines.len() {
            self.lines[self.cursor_line].len()
        } else {
            0
        };
        if self.cursor_col > len {
            self.cursor_col = len;
        }
    }
    
    /// Total line count
    pub fn line_count(&self) -> usize {
        self.lines.len()
    }
    
    // ═══════════════════════════════════════════════════════════════════════
    // INPUT HANDLING
    // ═══════════════════════════════════════════════════════════════════════
    
    /// Handle a keypress. Returns true if handled.
    pub fn handle_key(&mut self, key: u8) -> bool {
        use crate::keyboard::*;
        
        self.blink_counter = 0; // Reset blink on any key
        
        // Clear status message on any key after showing it
        if self.status_message.is_some() && key != 0x13 { // not Ctrl+S
            // Keep message for a few more keypresses
        }
        
        match key {
            // ── Ctrl+S (save) ──
            0x13 => {
                self.save();
                return true;
            }
            
            // ── Arrow keys ──
            KEY_UP => {
                if self.cursor_line > 0 {
                    self.cursor_line -= 1;
                    self.clamp_cursor_col();
                }
                self.ensure_cursor_visible();
                return true;
            }
            KEY_DOWN => {
                if self.cursor_line + 1 < self.lines.len() {
                    self.cursor_line += 1;
                    self.clamp_cursor_col();
                }
                self.ensure_cursor_visible();
                return true;
            }
            KEY_LEFT => {
                if self.cursor_col > 0 {
                    self.cursor_col -= 1;
                } else if self.cursor_line > 0 {
                    // Wrap to end of previous line
                    self.cursor_line -= 1;
                    self.cursor_col = self.lines[self.cursor_line].len();
                }
                self.ensure_cursor_visible();
                return true;
            }
            KEY_RIGHT => {
                let line_len = if self.cursor_line < self.lines.len() { self.lines[self.cursor_line].len() } else { 0 };
                if self.cursor_col < line_len {
                    self.cursor_col += 1;
                } else if self.cursor_line + 1 < self.lines.len() {
                    // Wrap to start of next line
                    self.cursor_line += 1;
                    self.cursor_col = 0;
                }
                self.ensure_cursor_visible();
                return true;
            }
            KEY_HOME => {
                self.cursor_col = 0;
                return true;
            }
            KEY_END => {
                let line_len = if self.cursor_line < self.lines.len() { self.lines[self.cursor_line].len() } else { 0 };
                self.cursor_col = line_len;
                return true;
            }
            KEY_PGUP => {
                let jump = 20;
                self.cursor_line = self.cursor_line.saturating_sub(jump);
                self.clamp_cursor_col();
                self.ensure_cursor_visible();
                return true;
            }
            KEY_PGDOWN => {
                let jump = 20;
                self.cursor_line = (self.cursor_line + jump).min(self.lines.len().saturating_sub(1));
                self.clamp_cursor_col();
                self.ensure_cursor_visible();
                return true;
            }
            
            // ── Backspace ──
            0x08 => {
                if self.cursor_col > 0 && self.cursor_line < self.lines.len() {
                    self.lines[self.cursor_line].remove(self.cursor_col - 1);
                    self.cursor_col -= 1;
                    self.dirty = true;
                } else if self.cursor_line > 0 {
                    // Join with previous line
                    let current = self.lines.remove(self.cursor_line);
                    self.cursor_line -= 1;
                    self.cursor_col = self.lines[self.cursor_line].len();
                    self.lines[self.cursor_line].push_str(&current);
                    self.dirty = true;
                }
                self.ensure_cursor_visible();
                return true;
            }
            
            // ── Delete ──
            KEY_DELETE => {
                if self.cursor_line < self.lines.len() {
                    let line_len = self.lines[self.cursor_line].len();
                    if self.cursor_col < line_len {
                        self.lines[self.cursor_line].remove(self.cursor_col);
                        self.dirty = true;
                    } else if self.cursor_line + 1 < self.lines.len() {
                        // Join next line
                        let next = self.lines.remove(self.cursor_line + 1);
                        self.lines[self.cursor_line].push_str(&next);
                        self.dirty = true;
                    }
                }
                return true;
            }
            
            // ── Enter ──
            0x0D | 0x0A => {
                if self.cursor_line < self.lines.len() {
                    // Split line at cursor
                    let rest = self.lines[self.cursor_line].split_off(self.cursor_col);
                    
                    // Auto-indent: copy leading whitespace from current line
                    let indent: String = self.lines[self.cursor_line]
                        .chars()
                        .take_while(|c| *c == ' ' || *c == '\t')
                        .collect();
                    
                    // Extra indent after { or (
                    let extra = if self.lines[self.cursor_line].trim_end().ends_with('{')
                        || self.lines[self.cursor_line].trim_end().ends_with('(') {
                        "    "
                    } else {
                        ""
                    };
                    
                    let new_line = format!("{}{}{}", indent, extra, rest);
                    self.cursor_line += 1;
                    self.lines.insert(self.cursor_line, new_line);
                    self.cursor_col = indent.len() + extra.len();
                    self.dirty = true;
                }
                self.ensure_cursor_visible();
                return true;
            }
            
            // ── Tab ──
            0x09 => {
                // Insert 4 spaces
                if self.cursor_line < self.lines.len() {
                    for _ in 0..4 {
                        self.lines[self.cursor_line].insert(self.cursor_col, ' ');
                        self.cursor_col += 1;
                    }
                    self.dirty = true;
                }
                return true;
            }
            
            // ── Printable characters ──
            c if c >= 0x20 && c < 0x7F => {
                if self.cursor_line < self.lines.len() {
                    let ch = c as char;
                    if self.cursor_col <= self.lines[self.cursor_line].len() {
                        self.lines[self.cursor_line].insert(self.cursor_col, ch);
                        self.cursor_col += 1;
                        self.dirty = true;
                        
                        // Auto-close brackets
                        let close = match ch {
                            '{' => Some('}'),
                            '(' => Some(')'),
                            '[' => Some(']'),
                            '"' => Some('"'),
                            '\'' => Some('\''),
                            _ => None,
                        };
                        if let Some(closing) = close {
                            self.lines[self.cursor_line].insert(self.cursor_col, closing);
                            // Don't advance cursor — keep between brackets
                        }
                    }
                }
                return true;
            }
            
            _ => {}
        }
        false
    }
    
    /// Ensure cursor is visible by adjusting scroll
    fn ensure_cursor_visible(&mut self) {
        if self.cursor_line < self.scroll_y {
            self.scroll_y = self.cursor_line;
        }
        // We'll calculate visible_lines in the render pass;
        // for now assume ~30 lines visible
        let visible = 30usize;
        if self.cursor_line >= self.scroll_y + visible {
            self.scroll_y = self.cursor_line - visible + 1;
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SYNTAX HIGHLIGHTING
// ═══════════════════════════════════════════════════════════════════════════════

/// Token type for syntax coloring
#[derive(Clone, Copy, PartialEq)]
pub enum TokenKind {
    Normal,
    Keyword,
    Type,
    String,
    Comment,
    Number,
    Function,
    Macro,
    Attribute,
    Lifetime,
    Operator,
    Bracket,
}

/// A colored span within a line
pub struct ColorSpan {
    pub start: usize,
    pub end: usize,
    pub kind: TokenKind,
}

// VSCode-like Dark+ color palette
pub const COLOR_KEYWORD: u32   = 0xFF569CD6; // Blue (let, fn, if, use...)
pub const COLOR_TYPE: u32      = 0xFF4EC9B0; // Teal (u32, String, Self...)
pub const COLOR_STRING: u32    = 0xFFCE9178; // Orange (string literals)
pub const COLOR_COMMENT: u32   = 0xFF6A9955; // Green (comments)
pub const COLOR_NUMBER: u32    = 0xFFB5CEA8; // Light green (numbers)
pub const COLOR_FUNCTION: u32  = 0xFFDCDCAA; // Yellow (function names)
pub const COLOR_MACRO: u32     = 0xFF4FC1FF; // Bright blue (macros!)
pub const COLOR_ATTRIBUTE: u32 = 0xFFD7BA7D; // Gold (#[derive(...)])
pub const COLOR_LIFETIME: u32  = 0xFF569CD6; // Blue ('a, 'static)
pub const COLOR_OPERATOR: u32  = 0xFFD4D4D4; // Light gray
pub const COLOR_BRACKET: u32   = 0xFFFFD700; // Gold/yellow brackets
pub const COLOR_NORMAL: u32    = 0xFFD4D4D4; // Default text
pub const COLOR_LINE_NUM: u32  = 0xFF858585; // Gutter line numbers
pub const COLOR_ACTIVE_LINE: u32 = 0xFF858585; // Active line number
pub const COLOR_BG: u32        = 0xFF1E1E2E; // Editor background (dark)
pub const COLOR_GUTTER_BG: u32 = 0xFF1E1E2E; // Gutter background
pub const COLOR_ACTIVE_LINE_BG: u32 = 0xFF2A2D3A; // Current line highlight
pub const COLOR_STATUS_BG: u32 = 0xFF007ACC; // Status bar (blue)
pub const COLOR_STATUS_FG: u32 = 0xFFFFFFFF; // Status bar text
pub const COLOR_CURSOR: u32    = 0xFFAEAFAD; // Cursor color
pub const COLOR_BREADCRUMB_BG: u32 = 0xFF252526; // Breadcrumb/tab bar
pub const COLOR_TAB_ACTIVE: u32 = 0xFF1E1E2E; // Active tab background
pub const COLOR_TAB_INACTIVE: u32 = 0xFF2D2D2D; // Inactive tab

/// Rust keywords for highlighting
const RUST_KEYWORDS: &[&str] = &[
    "as", "async", "await", "break", "const", "continue", "crate", "dyn",
    "else", "enum", "extern", "false", "fn", "for", "if", "impl", "in",
    "let", "loop", "match", "mod", "move", "mut", "pub", "ref", "return",
    "self", "Self", "static", "struct", "super", "trait", "true", "type",
    "unsafe", "use", "where", "while", "yield",
];

/// Rust built-in types
const RUST_TYPES: &[&str] = &[
    "bool", "char", "f32", "f64", "i8", "i16", "i32", "i64", "i128", "isize",
    "u8", "u16", "u32", "u64", "u128", "usize", "str", "String", "Vec",
    "Option", "Result", "Box", "Rc", "Arc", "Cell", "RefCell", "Mutex",
    "HashMap", "HashSet", "BTreeMap", "BTreeSet", "Cow", "Pin",
    "Some", "None", "Ok", "Err",
];

/// Tokenize a single line of Rust code into colored spans
pub fn tokenize_rust_line(line: &str) -> Vec<ColorSpan> {
    let mut spans = Vec::new();
    let bytes = line.as_bytes();
    let len = bytes.len();
    let mut i = 0;
    
    while i < len {
        let ch = bytes[i];
        
        // ── Line comment ──
        if ch == b'/' && i + 1 < len && bytes[i + 1] == b'/' {
            spans.push(ColorSpan { start: i, end: len, kind: TokenKind::Comment });
            break; // Rest of line is comment
        }
        
        // ── Attribute ──
        if ch == b'#' && i + 1 < len && bytes[i + 1] == b'[' {
            let start = i;
            // Find closing ]
            while i < len && bytes[i] != b']' { i += 1; }
            if i < len { i += 1; } // include ]
            spans.push(ColorSpan { start, end: i, kind: TokenKind::Attribute });
            continue;
        }
        
        // ── String literal ──
        if ch == b'"' {
            let start = i;
            i += 1;
            while i < len {
                if bytes[i] == b'\\' && i + 1 < len {
                    i += 2; // skip escape
                } else if bytes[i] == b'"' {
                    i += 1;
                    break;
                } else {
                    i += 1;
                }
            }
            spans.push(ColorSpan { start, end: i, kind: TokenKind::String });
            continue;
        }
        
        // ── Char literal ──
        if ch == b'\'' {
            // Check if it's a lifetime ('a, 'static) or char literal
            let start = i;
            i += 1;
            if i < len && bytes[i].is_ascii_alphabetic() {
                // Could be lifetime or char
                let word_start = i;
                while i < len && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                    i += 1;
                }
                if i < len && bytes[i] == b'\'' {
                    // Char literal like 'a' or '\n'
                    i += 1;
                    spans.push(ColorSpan { start, end: i, kind: TokenKind::String });
                } else {
                    // Lifetime like 'a or 'static
                    spans.push(ColorSpan { start, end: i, kind: TokenKind::Lifetime });
                }
            } else if i < len && bytes[i] == b'\\' {
                // Escape char like '\n'
                while i < len && bytes[i] != b'\'' { i += 1; }
                if i < len { i += 1; }
                spans.push(ColorSpan { start, end: i, kind: TokenKind::String });
            } else {
                spans.push(ColorSpan { start, end: start + 1, kind: TokenKind::Normal });
            }
            continue;
        }
        
        // ── Number ──
        if ch.is_ascii_digit() || (ch == b'0' && i + 1 < len && (bytes[i+1] == b'x' || bytes[i+1] == b'b' || bytes[i+1] == b'o')) {
            let start = i;
            i += 1;
            while i < len && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_' || bytes[i] == b'.') {
                i += 1;
            }
            spans.push(ColorSpan { start, end: i, kind: TokenKind::Number });
            continue;
        }
        
        // ── Identifiers / keywords ──
        if ch.is_ascii_alphabetic() || ch == b'_' {
            let start = i;
            while i < len && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                i += 1;
            }
            let word = &line[start..i];
            
            // Check if followed by ! (macro)
            if i < len && bytes[i] == b'!' {
                spans.push(ColorSpan { start, end: i + 1, kind: TokenKind::Macro });
                i += 1;
                continue;
            }
            
            // Check if followed by ( — function call
            let is_fn_call = i < len && bytes[i] == b'(';
            
            let kind = if RUST_KEYWORDS.contains(&word) {
                TokenKind::Keyword
            } else if RUST_TYPES.contains(&word) {
                TokenKind::Type
            } else if is_fn_call {
                TokenKind::Function
            } else {
                TokenKind::Normal
            };
            
            spans.push(ColorSpan { start, end: i, kind });
            continue;
        }
        
        // ── Brackets ──
        if ch == b'{' || ch == b'}' || ch == b'(' || ch == b')' || ch == b'[' || ch == b']' {
            spans.push(ColorSpan { start: i, end: i + 1, kind: TokenKind::Bracket });
            i += 1;
            continue;
        }
        
        // ── Operators ──
        if ch == b'+' || ch == b'-' || ch == b'*' || ch == b'=' || ch == b'!' 
            || ch == b'<' || ch == b'>' || ch == b'&' || ch == b'|' || ch == b'^'
            || ch == b':' || ch == b';' || ch == b',' || ch == b'.' {
            spans.push(ColorSpan { start: i, end: i + 1, kind: TokenKind::Operator });
            i += 1;
            continue;
        }
        
        // ── Whitespace or other ──
        let start = i;
        i += 1;
        spans.push(ColorSpan { start, end: i, kind: TokenKind::Normal });
    }
    
    spans
}

/// Get the color for a token kind
pub fn token_color(kind: TokenKind) -> u32 {
    match kind {
        TokenKind::Normal => COLOR_NORMAL,
        TokenKind::Keyword => COLOR_KEYWORD,
        TokenKind::Type => COLOR_TYPE,
        TokenKind::String => COLOR_STRING,
        TokenKind::Comment => COLOR_COMMENT,
        TokenKind::Number => COLOR_NUMBER,
        TokenKind::Function => COLOR_FUNCTION,
        TokenKind::Macro => COLOR_MACRO,
        TokenKind::Attribute => COLOR_ATTRIBUTE,
        TokenKind::Lifetime => COLOR_LIFETIME,
        TokenKind::Operator => COLOR_OPERATOR,
        TokenKind::Bracket => COLOR_BRACKET,
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// EDITOR RENDERING (called from desktop.rs draw_window_content)
// ═══════════════════════════════════════════════════════════════════════════════

/// Render the editor inside a window frame.
/// `x, y, w, h` = content area (inside window chrome).
pub fn render_editor(
    state: &mut EditorState,
    x: i32, y: i32, w: u32, h: u32,
    draw_text_fn: &dyn Fn(i32, i32, &str, u32),
    draw_char_fn: &dyn Fn(i32, i32, char, u32),
) {
    let char_w: i32 = 8;
    let line_h: i32 = 16;
    let gutter_chars = 5; // "nnnn " — 4 digits + 1 space
    let gutter_w = gutter_chars * char_w;
    let status_h: i32 = 20;
    let tab_bar_h: i32 = 24;
    
    let code_x = x + gutter_w;
    let code_y = y + tab_bar_h;
    let code_w = w as i32 - gutter_w;
    let code_h = h as i32 - tab_bar_h - status_h;
    let visible_lines = (code_h / line_h).max(1) as usize;
    
    // Update scroll based on actual visible lines
    if state.cursor_line < state.scroll_y {
        state.scroll_y = state.cursor_line;
    }
    if state.cursor_line >= state.scroll_y + visible_lines {
        state.scroll_y = state.cursor_line - visible_lines + 1;
    }
    
    state.blink_counter += 1;
    
    // ── Tab bar ──
    crate::framebuffer::fill_rect(x as u32, y as u32, w, tab_bar_h as u32, COLOR_BREADCRUMB_BG);
    // Active tab
    let tab_name = state.file_path.as_ref().map(|p| {
        // Extract filename from path
        p.rsplit('/').next().unwrap_or(p.as_str())
    }).unwrap_or("untitled");
    let dirty_marker = if state.dirty { " ●" } else { "" };
    let tab_label = format!("  {}{}  ", tab_name, dirty_marker);
    crate::framebuffer::fill_rect(x as u32, y as u32, (tab_label.len() as u32 * 8).min(w), tab_bar_h as u32, COLOR_TAB_ACTIVE);
    // Tab bottom border (accent)
    crate::framebuffer::fill_rect(x as u32, (y + tab_bar_h - 2) as u32, (tab_label.len() as u32 * 8).min(w), 2, COLOR_STATUS_BG);
    draw_text_fn(x + 4, y + 4, &tab_label, COLOR_NORMAL);
    
    // ── Editor background ──
    crate::framebuffer::fill_rect(x as u32, (y + tab_bar_h) as u32, w, code_h as u32, COLOR_BG);
    
    // ── Gutter background ──
    crate::framebuffer::fill_rect(x as u32, (y + tab_bar_h) as u32, gutter_w as u32, code_h as u32, COLOR_GUTTER_BG);
    // Gutter right border
    crate::framebuffer::fill_rect((x + gutter_w - 1) as u32, (y + tab_bar_h) as u32, 1, code_h as u32, 0xFF333333);
    
    // ── Render lines ──
    for vi in 0..visible_lines {
        let line_idx = state.scroll_y + vi;
        if line_idx >= state.lines.len() { break; }
        
        let ly = code_y + (vi as i32 * line_h);
        if ly + line_h > y + tab_bar_h + code_h { break; }
        
        let is_current_line = line_idx == state.cursor_line;
        
        // ── Current line highlight ──
        if is_current_line {
            crate::framebuffer::fill_rect(
                code_x as u32, ly as u32,
                code_w as u32, line_h as u32,
                COLOR_ACTIVE_LINE_BG,
            );
        }
        
        // ── Line number ──
        let line_num_str = format!("{:>4} ", line_idx + 1);
        let num_color = if is_current_line { COLOR_ACTIVE_LINE } else { COLOR_LINE_NUM };
        draw_text_fn(x + 2, ly, &line_num_str, num_color);
        
        // ── Code with syntax highlighting ──
        let line = &state.lines[line_idx];
        
        if state.language == Language::Rust {
            let tokens = tokenize_rust_line(line);
            for span in &tokens {
                let color = token_color(span.kind);
                let text_segment = &line[span.start..span.end];
                let sx = code_x + 4 + (span.start as i32 * char_w);
                if sx < x + w as i32 {
                    draw_text_fn(sx, ly, text_segment, color);
                }
            }
            // If no tokens, draw empty
            if tokens.is_empty() && !line.is_empty() {
                draw_text_fn(code_x + 4, ly, line, COLOR_NORMAL);
            }
        } else {
            // Plain text — no highlighting
            draw_text_fn(code_x + 4, ly, line, COLOR_NORMAL);
        }
        
        // ── Cursor ──
        if is_current_line {
            let blink_on = (state.blink_counter / 30) % 2 == 0;
            if blink_on {
                let cx = code_x + 4 + (state.cursor_col as i32 * char_w);
                // Draw thin cursor line (2px wide)
                crate::framebuffer::fill_rect(
                    cx as u32, ly as u32,
                    2, line_h as u32,
                    COLOR_CURSOR,
                );
            }
        }
    }
    
    // ── Scrollbar ──
    if state.lines.len() > visible_lines {
        let sb_x = (x + w as i32 - 8) as u32;
        let sb_h = code_h as u32;
        let thumb_h = ((visible_lines as u32 * sb_h) / state.lines.len() as u32).max(20);
        let thumb_y = (state.scroll_y as u32 * (sb_h - thumb_h)) / state.lines.len().saturating_sub(visible_lines) as u32;
        // Track
        crate::framebuffer::fill_rect(sb_x, code_y as u32, 8, sb_h, 0xFF252526);
        // Thumb
        crate::framebuffer::fill_rounded_rect(sb_x + 1, code_y as u32 + thumb_y, 6, thumb_h, 3, 0xFF555555);
    }
    
    // ── Status bar ──
    let status_y = y + tab_bar_h + code_h;
    crate::framebuffer::fill_rect(x as u32, status_y as u32, w, status_h as u32, COLOR_STATUS_BG);
    
    // Left: file info
    let status_left = if let Some(ref msg) = state.status_message {
        format!("  {}", msg)
    } else {
        let dirty_str = if state.dirty { " [Modified]" } else { "" };
        let fname = state.file_path.as_deref().unwrap_or("untitled");
        format!("  {}{}", fname, dirty_str)
    };
    draw_text_fn(x + 4, status_y + 2, &status_left, COLOR_STATUS_FG);
    
    // Right: position and language
    let status_right = format!(
        "Ln {}, Col {}   {}   UTF-8   TrustCode  ",
        state.cursor_line + 1,
        state.cursor_col + 1,
        state.language.name(),
    );
    let right_x = x + w as i32 - (status_right.len() as i32 * char_w) - 4;
    draw_text_fn(right_x, status_y + 2, &status_right, COLOR_STATUS_FG);
}
