//! TrustLang Editor Panel — Inline code editor with syntax highlighting
//!
//! A mini code editor for writing and running TrustLang scripts
//! directly inside the Lab view. Supports:
//! - Multi-line editing with cursor
//! - Basic syntax highlighting (keywords, strings, numbers, comments)
//! - Run button (Enter in output area to execute)
//! - Output display

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use super::{draw_lab_text, char_w, char_h,
            COL_TEXT, COL_DIM, COL_ACCENT, COL_GREEN, COL_YELLOW, COL_RED, COL_PURPLE, COL_CYAN};

/// Editor panel state
pub struct EditorState {
    /// Source code lines
    pub lines: Vec<String>,
    /// Cursor line
    pub cursor_line: usize,
    /// Cursor column
    pub cursor_col: usize,
    /// Scroll offset
    pub scroll: usize,
    /// Output from last run
    pub output: Vec<String>,
    /// Whether output area is focused (vs editor)
    pub output_focused: bool,
    /// Output scroll
    pub output_scroll: usize,
    /// Frame counter for cursor blink
    pub frame: u64,
}

impl EditorState {
    pub fn new() -> Self {
        Self {
            lines: alloc::vec![
                String::from("// TrustLang — write code here"),
                String::from("fn main() {"),
                String::from("    let x = 42;"),
                String::from("    print(\"Hello from TrustLab!\");"),
                String::from("    print(x * 2);"),
                String::from("}"),
            ],
            cursor_line: 1,
            cursor_col: 0,
            scroll: 0,
            output: alloc::vec![String::from("Press Ctrl+R to run")],
            output_focused: false,
            output_scroll: 0,
            frame: 0,
        }
    }
    
    pub fn handle_key(&mut self, key: u8) {
        use crate::keyboard::{KEY_UP, KEY_DOWN, KEY_LEFT, KEY_RIGHT, KEY_PGUP, KEY_PGDOWN};
        
        self.frame += 1;
        
        match key {
            KEY_UP => {
                if self.output_focused {
                    self.output_scroll = self.output_scroll.saturating_sub(1);
                } else if self.cursor_line > 0 {
                    self.cursor_line -= 1;
                    self.clamp_col();
                }
            }
            KEY_DOWN => {
                if self.output_focused {
                    self.output_scroll += 1;
                } else if self.cursor_line + 1 < self.lines.len() {
                    self.cursor_line += 1;
                    self.clamp_col();
                }
            }
            KEY_LEFT => {
                if !self.output_focused && self.cursor_col > 0 {
                    self.cursor_col -= 1;
                }
            }
            KEY_RIGHT => {
                if !self.output_focused {
                    let line_len = self.lines.get(self.cursor_line)
                        .map(|l| l.len()).unwrap_or(0);
                    if self.cursor_col < line_len {
                        self.cursor_col += 1;
                    }
                }
            }
            KEY_PGUP => {
                if self.output_focused {
                    self.output_scroll = self.output_scroll.saturating_sub(5);
                } else {
                    self.cursor_line = self.cursor_line.saturating_sub(10);
                    self.clamp_col();
                }
            }
            KEY_PGDOWN => {
                if self.output_focused {
                    self.output_scroll += 5;
                } else {
                    self.cursor_line = (self.cursor_line + 10).min(self.lines.len().saturating_sub(1));
                    self.clamp_col();
                }
            }
            // Ctrl+R = run code
            0x12 => {
                self.run_code();
            }
            // Enter
            0x0D | 0x0A => {
                if !self.output_focused {
                    // Split line at cursor
                    if self.cursor_line < self.lines.len() {
                        let rest = self.lines[self.cursor_line].split_off(self.cursor_col);
                        self.cursor_line += 1;
                        self.lines.insert(self.cursor_line, rest);
                        self.cursor_col = 0;
                    } else {
                        self.lines.push(String::new());
                        self.cursor_line = self.lines.len() - 1;
                        self.cursor_col = 0;
                    }
                }
            }
            // Backspace
            0x08 => {
                if !self.output_focused {
                    if self.cursor_col > 0 {
                        self.cursor_col -= 1;
                        if self.cursor_line < self.lines.len() {
                            self.lines[self.cursor_line].remove(self.cursor_col);
                        }
                    } else if self.cursor_line > 0 {
                        // Join with previous line
                        let current = self.lines.remove(self.cursor_line);
                        self.cursor_line -= 1;
                        self.cursor_col = self.lines[self.cursor_line].len();
                        self.lines[self.cursor_line].push_str(&current);
                    }
                }
            }
            // Ctrl+O = toggle output focus
            0x0F => {
                self.output_focused = !self.output_focused;
            }
            _ => {}
        }
    }
    
    pub fn handle_char(&mut self, ch: char) {
        if self.output_focused { return; }
        
        self.frame += 1;
        
        if self.cursor_line >= self.lines.len() {
            self.lines.push(String::new());
            self.cursor_line = self.lines.len() - 1;
        }
        
        self.lines[self.cursor_line].insert(self.cursor_col, ch);
        self.cursor_col += 1;
    }
    
    /// Handle mouse click at (x, y) relative to content area
    pub fn handle_click(&mut self, x: i32, y: i32, w: u32, h: u32) {
        let cw = super::char_w();
        let lh = super::char_h() + 1;
        if lh <= 0 || cw <= 0 { return; }

        // Same layout as draw(): 60% editor, 40% output
        let editor_h = (h as i32 * 60 / 100).max(lh * 3);
        let output_y = editor_h + 2;

        if y >= output_y {
            // Clicked in output area
            self.output_focused = true;
        } else {
            // Clicked in editor area
            self.output_focused = false;

            // Code starts after header line
            let code_y = lh;
            if y < code_y { return; } // header

            let gutter_w = 4 * cw;
            let row = ((y - code_y) / lh) as usize;
            let col = ((x - gutter_w).max(0) / cw) as usize;

            self.cursor_line = (self.scroll + row).min(self.lines.len().saturating_sub(1));
            let line_len = self.lines.get(self.cursor_line).map(|l| l.len()).unwrap_or(0);
            self.cursor_col = col.min(line_len);
            self.frame += 1; // reset blink
        }
    }

    fn clamp_col(&mut self) {
        let line_len = self.lines.get(self.cursor_line)
            .map(|l| l.len()).unwrap_or(0);
        if self.cursor_col > line_len {
            self.cursor_col = line_len;
        }
    }
    
    /// Execute the code (using TrustLang VM if available)
    pub fn run_code(&mut self) {
        self.output.clear();
        self.output.push(String::from("=== Running ==="));
        
        // Concatenate all lines 
        let source: String = self.lines.iter()
            .map(|l| l.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        
        // Try to run through TrustLang
        self.output.push(format!("Source: {} lines, {} bytes", self.lines.len(), source.len()));
        
        // Attempt TrustLang execution
        match crate::trustlang::run(&source) {
            Ok(result) => {
                self.output.push(format!("=> {}", result));
            }
            Err(e) => {
                self.output.push(format!("Error: {}", e));
            }
        }
        
        self.output.push(String::from("=== Done ==="));
        self.output_focused = true;
    }
}

/// Draw the editor panel
pub fn draw(state: &EditorState, x: i32, y: i32, w: u32, h: u32) {
    let cw = char_w();
    let lh = char_h() + 1;
    if lh <= 0 || cw <= 0 { return; }
    
    // Split: 60% editor, 40% output (vertically)
    let editor_h = (h as i32 * 60 / 100).max(lh * 3);
    let output_y = y + editor_h + 2;
    let output_h = h as i32 - editor_h - 4;
    
    // ── Editor area ────────────────────────
    // Header
    let header = if state.output_focused { "Editor (Ctrl+O)" } else { "Editor [active]" };
    let header_color = if !state.output_focused { COL_ACCENT } else { COL_DIM };
    draw_lab_text(x, y, header, header_color);
    
    // F5 hint
    let hint = "[Ctrl+R] run";
    let hint_x = x + w as i32 - (hint.len() as i32 * cw) - 2;
    draw_lab_text(hint_x, y, hint, COL_GREEN);
    
    let code_y = y + lh;
    let code_h = editor_h - lh;
    let visible_code = (code_h / lh) as usize;
    
    // Auto-scroll to cursor
    let scroll = if state.cursor_line >= state.scroll + visible_code {
        state.cursor_line - visible_code + 1
    } else if state.cursor_line < state.scroll {
        state.cursor_line
    } else {
        state.scroll
    };
    
    let gutter_w = 4 * cw; // Line number gutter
    let code_x = x + gutter_w;
    
    let end = (scroll + visible_code).min(state.lines.len());
    let mut cy = code_y;
    
    for i in scroll..end {
        // Line number
        let line_num = format!("{:>3}", i + 1);
        draw_lab_text(x, cy, &line_num, COL_DIM);
        
        // Highlight current line
        if i == state.cursor_line && !state.output_focused {
            crate::framebuffer::fill_rect(
                code_x as u32, cy as u32,
                w.saturating_sub(gutter_w as u32), lh as u32,
                0xFF1C2128,
            );
        }
        
        // Syntax-highlighted line
        draw_highlighted_line(code_x, cy, &state.lines[i], w.saturating_sub(gutter_w as u32));
        
        // Cursor
        if i == state.cursor_line && !state.output_focused {
            if (state.frame / 25) % 2 == 0 {
                let cursor_x = code_x + (state.cursor_col as i32 * cw);
                crate::framebuffer::fill_rect(
                    cursor_x as u32, cy as u32,
                    2, lh as u32,
                    COL_ACCENT,
                );
            }
        }
        
        cy += lh;
    }
    
    // ── Separator ──────────────────────────
    crate::framebuffer::fill_rect(x as u32, (output_y - 1) as u32, w, 1, 0xFF30363D);
    
    // ── Output area ────────────────────────
    if output_h <= 0 { return; }
    
    let out_header = if state.output_focused { "Output [active]" } else { "Output (Ctrl+O)" };
    let out_color = if state.output_focused { COL_GREEN } else { COL_DIM };
    draw_lab_text(x, output_y, out_header, out_color);
    
    let out_list_y = output_y + lh;
    let visible_out = ((output_h - lh) / lh) as usize;
    let out_scroll = state.output_scroll.min(state.output.len().saturating_sub(1));
    let out_end = (out_scroll + visible_out).min(state.output.len());
    
    let mut oy = out_list_y;
    for i in out_scroll..out_end {
        let line = &state.output[i];
        let color = if line.starts_with("Error:") {
            COL_RED
        } else if line.starts_with("=>") {
            COL_GREEN
        } else if line.starts_with("===") {
            COL_YELLOW
        } else {
            COL_TEXT
        };
        
        draw_lab_text(x + 4, oy, line, color);
        oy += lh;
    }
}

/// Draw a line with basic syntax highlighting
fn draw_highlighted_line(x: i32, y: i32, line: &str, _max_w: u32) {
    let cw = char_w();
    let keywords = ["fn", "let", "mut", "if", "else", "for", "while", "return", 
                     "true", "false", "struct", "enum", "match", "pub", "use",
                     "const", "static", "impl", "self", "loop", "break", "continue"];
    
    let mut cx = x;
    let chars: Vec<char> = line.chars().collect();
    let len = chars.len();
    let mut i = 0;
    
    while i < len {
        let ch = chars[i];
        
        // Comment: // to end of line
        if ch == '/' && i + 1 < len && chars[i + 1] == '/' {
            // Draw rest of line as comment
            let rest: String = chars[i..].iter().collect();
            draw_lab_text(cx, y, &rest, COL_DIM);
            return;
        }
        
        // String literal
        if ch == '"' {
            let start = i;
            i += 1;
            while i < len && chars[i] != '"' {
                if chars[i] == '\\' { i += 1; } // skip escape
                i += 1;
            }
            if i < len { i += 1; } // closing quote
            let s: String = chars[start..i].iter().collect();
            draw_lab_text(cx, y, &s, COL_GREEN);
            cx += s.len() as i32 * cw;
            continue;
        }
        
        // Number
        if ch.is_ascii_digit() {
            let start = i;
            while i < len && (chars[i].is_ascii_digit() || chars[i] == '.' || chars[i] == 'x') {
                i += 1;
            }
            let num: String = chars[start..i].iter().collect();
            draw_lab_text(cx, y, &num, COL_CYAN);
            cx += num.len() as i32 * cw;
            continue;
        }
        
        // Identifier / keyword
        if ch.is_ascii_alphanumeric() || ch == '_' {
            let start = i;
            while i < len && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            let word: String = chars[start..i].iter().collect();
            let color = if keywords.contains(&word.as_str()) {
                COL_PURPLE
            } else if word.chars().next().map(|c| c.is_ascii_uppercase()).unwrap_or(false) {
                COL_YELLOW // Type names
            } else {
                COL_TEXT
            };
            draw_lab_text(cx, y, &word, color);
            cx += word.len() as i32 * cw;
            continue;
        }
        
        // Operators and punctuation
        let color = match ch {
            '(' | ')' | '{' | '}' | '[' | ']' => COL_YELLOW,
            '=' | '+' | '-' | '*' | '/' | '<' | '>' | '!' | '&' | '|' => COL_ACCENT,
            ';' | ':' | ',' | '.' => COL_DIM,
            _ => COL_TEXT,
        };
        let s = alloc::format!("{}", ch);
        draw_lab_text(cx, y, &s, color);
        cx += cw;
        i += 1;
    }
}
