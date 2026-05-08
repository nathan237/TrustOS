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

/// Snapshot for undo/redo
#[derive(Clone)]
struct UndoSnapshot {
    lines: Vec<String>,
    cursor_line: usize,
    cursor_col: usize,
}

/// Editor state per-window (stored outside Window struct to avoid bloat)
#[derive(Clone)]
// Public structure — visible outside this module.
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
    /// Undo stack (past snapshots)
    undo_stack: Vec<UndoSnapshot>,
    /// Redo stack (undone snapshots)
    redo_stack: Vec<UndoSnapshot>,
    /// Selection anchor (line, col) — None means no selection
    pub selection_anchor: Option<(usize, usize)>,
    /// Find mode: None=normal, Some(query)=find active
    pub find_query: Option<String>,
    /// Replace text (when in replace mode)
    pub replace_text: Option<String>,
    /// Whether we're editing the replace field (vs find field)
    pub find_replace_mode: bool,
    /// All find match positions [(line, col)]
    pub find_matches: Vec<(usize, usize)>,
    /// Current match index
    pub find_match_idx: usize,
    /// Go-to-line mode: None=normal, Some(input)=dialog active
    pub goto_line_input: Option<String>,
    /// Matching bracket position (line, col) — calculated each frame
    pub matching_bracket: Option<(usize, usize)>,
}

// #[derive] — auto-generates trait implementations at compile time.
#[derive(Clone, Copy, PartialEq)]
// Enumeration — a type that can be one of several variants.
pub enum Language {
    Plain,
    Rust,
    Toml,
    Markdown,
    C,
    Python,
    JavaScript,
}

// Implementation block — defines methods for the type above.
impl Language {
        // Public function — callable from other modules.
pub fn name(&self) -> &'static str {
                // Pattern matching — Rust's exhaustive branching construct.
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

// Implementation block — defines methods for the type above.
impl EditorState {
        // Public function — callable from other modules.
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
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            selection_anchor: None,
            find_query: None,
            replace_text: None,
            find_replace_mode: false,
            find_matches: Vec::new(),
            find_match_idx: 0,
            goto_line_input: None,
            matching_bracket: None,
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
        
        if let Ok(data) = crate::ramfs::with_filesystem(|fs| fs.read_file(&full_path).map(|d| d.to_vec())) {
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
            
            let result = crate::ramfs::with_filesystem(|fs| {
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

    /// Save current state to undo stack (call before every text mutation)
    fn save_undo_state(&mut self) {
        self.undo_stack.push(UndoSnapshot {
            lines: self.lines.clone(),
            cursor_line: self.cursor_line,
            cursor_col: self.cursor_col,
        });
        // Cap at 200 entries
        if self.undo_stack.len() > 200 {
            self.undo_stack.remove(0);
        }
        // Any new edit clears redo stack
        self.redo_stack.clear();
    }

    /// Undo: restore previous state
    fn undo(&mut self) {
        if let Some(snapshot) = self.undo_stack.pop() {
            // Push current state to redo stack
            self.redo_stack.push(UndoSnapshot {
                lines: self.lines.clone(),
                cursor_line: self.cursor_line,
                cursor_col: self.cursor_col,
            });
            self.lines = snapshot.lines;
            self.cursor_line = snapshot.cursor_line;
            self.cursor_col = snapshot.cursor_col;
            self.dirty = true;
            self.status_message = Some(String::from("Undo"));
        }
    }

    /// Redo: restore undone state
    fn redo(&mut self) {
        if let Some(snapshot) = self.redo_stack.pop() {
            // Push current state to undo stack
            self.undo_stack.push(UndoSnapshot {
                lines: self.lines.clone(),
                cursor_line: self.cursor_line,
                cursor_col: self.cursor_col,
            });
            self.lines = snapshot.lines;
            self.cursor_line = snapshot.cursor_line;
            self.cursor_col = snapshot.cursor_col;
            self.dirty = true;
            self.status_message = Some(String::from("Redo"));
        }
    }

    /// Get ordered selection range: (start_line, start_col, end_line, end_col)
    /// Returns None if no selection active
    pub fn get_selection_range(&self) -> Option<(usize, usize, usize, usize)> {
        let (al, ac) = self.selection_anchor?;
        let (bl, bc) = (self.cursor_line, self.cursor_col);
        if (al, ac) <= (bl, bc) {
            Some((al, ac, bl, bc))
        } else {
            Some((bl, bc, al, ac))
        }
    }

    /// Extract selected text as a String
    fn selected_text(&self) -> Option<String> {
        let (sl, sc, el, ec) = self.get_selection_range()?;
        let mut result = String::new();
        for l in sl..=el {
            if l >= self.lines.len() { break; }
            let line = &self.lines[l];
            let start = if l == sl { sc.min(line.len()) } else { 0 };
            let end = if l == el { ec.min(line.len()) } else { line.len() };
            if start <= end {
                result.push_str(&line[start..end]);
            }
            if l < el {
                result.push('\n');
            }
        }
        Some(result)
    }

    /// Delete the selected text and collapse cursor to selection start
    fn delete_selection(&mut self) {
        if let Some((sl, sc, el, ec)) = self.get_selection_range() {
            self.save_undo_state();
            if sl == el {
                // Single line selection
                if sl < self.lines.len() {
                    let end = ec.min(self.lines[sl].len());
                    let start = sc.min(end);
                    self.lines[sl] = format!("{}{}", &self.lines[sl][..start], &self.lines[sl][end..]);
                }
            } else {
                // Multi-line selection
                if el < self.lines.len() {
                    let after = if ec <= self.lines[el].len() {
                        String::from(&self.lines[el][ec..])
                    } else {
                        String::new()
                    };
                    // Merge start line prefix + end line suffix
                    let prefix = if sc <= self.lines[sl].len() {
                        String::from(&self.lines[sl][..sc])
                    } else {
                        self.lines[sl].clone()
                    };
                    self.lines[sl] = format!("{}{}", prefix, after);
                    // Remove lines between
                    let remove_count = el - sl;
                    for _ in 0..remove_count {
                        if sl + 1 < self.lines.len() {
                            self.lines.remove(sl + 1);
                        }
                    }
                }
            }
            self.cursor_line = sl;
            self.cursor_col = sc;
            self.selection_anchor = None;
            self.dirty = true;
        }
    }

    /// Update find matches from current query
    fn update_find_matches(&mut self) {
        self.find_matches.clear();
        if let Some(ref query) = self.find_query {
            if query.is_empty() { return; }
            let q = query.clone();
            for (line_index, line) in self.lines.iter().enumerate() {
                let mut start = 0;
                while start + q.len() <= line.len() {
                    if &line[start..start + q.len()] == q.as_str() {
                        self.find_matches.push((line_index, start));
                        start += q.len().max(1);
                    } else {
                        start += 1;
                    }
                }
            }
        }
    }

    /// Jump to next find match
    fn find_next(&mut self) {
        if self.find_matches.is_empty() { return; }
        self.find_match_idx = (self.find_match_idx + 1) % self.find_matches.len();
        let (line, col) = self.find_matches[self.find_match_idx];
        self.cursor_line = line;
        self.cursor_col = col;
        self.ensure_cursor_visible();
    }

    /// Jump to previous find match
    fn find_previous(&mut self) {
        if self.find_matches.is_empty() { return; }
        if self.find_match_idx == 0 {
            self.find_match_idx = self.find_matches.len() - 1;
        } else {
            self.find_match_idx -= 1;
        }
        let (line, col) = self.find_matches[self.find_match_idx];
        self.cursor_line = line;
        self.cursor_col = col;
        self.ensure_cursor_visible();
    }

    /// Replace current match and find next
    fn replace_current(&mut self) {
        if self.find_matches.is_empty() { return; }
        let idx = self.find_match_idx.min(self.find_matches.len() - 1);
        let (line, col) = self.find_matches[idx];
        let query = self.find_query.clone();
        let replacement = self.replace_text.clone();
        if let (Some(q), Some(rep)) = (query, replacement) {
            if line < self.lines.len() && col + q.len() <= self.lines[line].len() {
                self.save_undo_state();
                let q_length = q.len();
                self.lines[line] = format!("{}{}{}", &self.lines[line][..col], rep, &self.lines[line][col + q_length..]);
                self.dirty = true;
                self.update_find_matches();
                self.cursor_line = line;
                self.cursor_col = col + rep.len();
            }
        }
    }

    /// Replace all matches
    fn replace_all(&mut self) {
        if self.find_matches.is_empty() { return; }
        let query = self.find_query.clone();
        let replacement = self.replace_text.clone();
        if let (Some(q), Some(rep)) = (query, replacement) {
            self.save_undo_state();
            for line in self.lines.iter_mut() {
                let mut result = String::new();
                let mut start = 0;
                while start < line.len() {
                    if start + q.len() <= line.len() && &line[start..start + q.len()] == q.as_str() {
                        result.push_str(&rep);
                        start += q.len();
                    } else {
                        if let Some(ch) = line.as_bytes().get(start) {
                            result.push(*ch as char);
                        }
                        start += 1;
                    }
                }
                *line = result;
            }
            self.dirty = true;
            self.update_find_matches();
            self.status_message = Some(String::from("Replaced all"));
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
        
        // ── Find mode input handling ──
        if self.find_query.is_some() {
                        // Pattern matching — Rust's exhaustive branching construct.
match key {
                0x1B => { // Escape — close find
                    self.find_query = None;
                    self.replace_text = None;
                    self.find_replace_mode = false;
                    self.find_matches.clear();
                    return true;
                }
                0x0D | 0x0A => { // Enter — find next / replace
                    if self.find_replace_mode {
                        if let Some(ref _rt) = self.replace_text {
                            self.replace_current();
                        }
                    } else {
                        self.find_next();
                    }
                    return true;
                }
                0x09 => { // Tab — toggle between find and replace fields
                    if self.replace_text.is_some() {
                        self.find_replace_mode = !self.find_replace_mode;
                    }
                    return true;
                }
                0x08 => { // Backspace
                    if self.find_replace_mode {
                        if let Some(ref mut rt) = self.replace_text {
                            rt.pop();
                        }
                    } else if let Some(ref mut q) = self.find_query {
                        q.pop();
                    }
                    self.update_find_matches();
                    return true;
                }
                0x01 => { // Ctrl+A in find mode — replace all
                    self.replace_all();
                    return true;
                }
                c if c >= 0x20 && c < 0x7F => {
                    if self.find_replace_mode {
                        if let Some(ref mut rt) = self.replace_text {
                            rt.push(c as char);
                        }
                    } else if let Some(ref mut q) = self.find_query {
                        q.push(c as char);
                    }
                    self.update_find_matches();
                    // Auto-jump to first match
                    if !self.find_matches.is_empty() {
                        self.find_match_idx = 0;
                        let (line, col) = self.find_matches[0];
                        self.cursor_line = line;
                        self.cursor_col = col;
                        self.ensure_cursor_visible();
                    }
                    return true;
                }
                _ => { return true; }
            }
        }
        
        // ── Go-to-line mode input handling ──
        if self.goto_line_input.is_some() {
                        // Pattern matching — Rust's exhaustive branching construct.
match key {
                0x1B => { // Escape — close dialog
                    self.goto_line_input = None;
                    return true;
                }
                0x0D | 0x0A => { // Enter — jump to line
                    if let Some(ref input) = self.goto_line_input {
                        if let Ok(line_number) = input.parse::<usize>() {
                            if line_number > 0 && line_number <= self.lines.len() {
                                self.cursor_line = line_number - 1;
                                self.cursor_col = 0;
                                self.ensure_cursor_visible();
                                self.status_message = Some(format!("Go to line {}", line_number));
                            } else {
                                self.status_message = Some(format!("Invalid line (1-{})", self.lines.len()));
                            }
                        }
                    }
                    self.goto_line_input = None;
                    return true;
                }
                0x08 => { // Backspace
                    if let Some(ref mut input) = self.goto_line_input {
                        input.pop();
                    }
                    return true;
                }
                c if c >= b'0' && c <= b'9' => {
                    if let Some(ref mut input) = self.goto_line_input {
                        input.push(c as char);
                    }
                    return true;
                }
                _ => { return true; }
            }
        }
        
        let shift_held = crate::keyboard::is_key_pressed(0x2A) || crate::keyboard::is_key_pressed(0x36);
        
                // Pattern matching — Rust's exhaustive branching construct.
match key {
            // ── Ctrl+S (save) ──
            0x13 => {
                self.save();
                return true;
            }
            
            // ── Ctrl+F (find) ──
            0x06 => {
                self.find_query = Some(String::new());
                self.replace_text = None;
                self.find_replace_mode = false;
                self.find_matches.clear();
                self.find_match_idx = 0;
                return true;
            }
            
            // ── Ctrl+H (find & replace) ──
            0x12 => {
                self.find_query = Some(String::new());
                self.replace_text = Some(String::new());
                self.find_replace_mode = false;
                self.find_matches.clear();
                self.find_match_idx = 0;
                return true;
            }
            
            // ── Ctrl+Z (undo) ──
            0x1A => {
                self.undo();
                self.ensure_cursor_visible();
                return true;
            }
            
            // ── Ctrl+Y (redo) ──
            0x19 => {
                self.redo();
                self.ensure_cursor_visible();
                return true;
            }
            
            // ── Ctrl+G (go to line) ──
            0x07 => {
                self.goto_line_input = Some(String::new());
                return true;
            }
            
            // ── Ctrl+/ (toggle comment) ──
            KEY_CONTROLLER_SLASH => {
                self.toggle_comment();
                return true;
            }
            
            // ── Ctrl+Shift+K (delete line) ──
            KEY_CONTROLLER_SHIFT_K => {
                self.delete_current_line();
                return true;
            }
            
            // ── Ctrl+Shift+D (duplicate line) ──
            KEY_CONTROLLER_SHIFT_D => {
                self.duplicate_line();
                return true;
            }
            
            // ── Alt+Up (move line up) ──
            KEY_ALT_UP => {
                self.move_line_up();
                return true;
            }
            
            // ── Alt+Down (move line down) ──
            KEY_ALT_DOWN => {
                self.move_line_down();
                return true;
            }
            
            // ── Ctrl+Left (word left) ──
            KEY_CONTROLLER_LEFT => {
                if shift_held && self.selection_anchor.is_none() {
                    self.selection_anchor = Some((self.cursor_line, self.cursor_col));
                } else if !shift_held {
                    self.selection_anchor = None;
                }
                self.word_left();
                self.ensure_cursor_visible();
                return true;
            }
            
            // ── Ctrl+Right (word right) ──
            KEY_CONTROLLER_RIGHT => {
                if shift_held && self.selection_anchor.is_none() {
                    self.selection_anchor = Some((self.cursor_line, self.cursor_col));
                } else if !shift_held {
                    self.selection_anchor = None;
                }
                self.word_right();
                self.ensure_cursor_visible();
                return true;
            }
            
            // ── Ctrl+A (select all) ──
            0x01 => {
                self.selection_anchor = Some((0, 0));
                let last_line = self.lines.len().saturating_sub(1);
                let last_column = if last_line < self.lines.len() { self.lines[last_line].len() } else { 0 };
                self.cursor_line = last_line;
                self.cursor_col = last_column;
                self.status_message = Some(String::from("Select All"));
                return true;
            }
            
            // ── Ctrl+C (copy) ──
            0x03 => {
                if let Some(text) = self.selected_text() {
                    crate::keyboard::clipboard_set(&text);
                    self.status_message = Some(String::from("Copied"));
                }
                return true;
            }
            
            // ── Ctrl+X (cut) ──
            0x18 => {
                if let Some(text) = self.selected_text() {
                    crate::keyboard::clipboard_set(&text);
                    self.delete_selection();
                    self.status_message = Some(String::from("Cut"));
                }
                return true;
            }
            
            // ── Ctrl+V (paste) ──
            0x16 => {
                if let Some(text) = crate::keyboard::clipboard_get() {
                    // Delete selection first if any
                    if self.selection_anchor.is_some() {
                        self.delete_selection();
                    }
                    self.save_undo_state();
                    // Insert text at cursor
                    for ch in text.chars() {
                        if ch == '\n' {
                            // Split line
                            if self.cursor_line < self.lines.len() {
                                self.cursor_col = self.cursor_col.min(self.lines[self.cursor_line].len());
                                let rest = self.lines[self.cursor_line].split_off(self.cursor_col);
                                self.cursor_line += 1;
                                self.lines.insert(self.cursor_line, rest);
                                self.cursor_col = 0;
                            }
                        } else if ch >= ' ' && ch as u32 <= 0x7E {
                            if self.cursor_line < self.lines.len() && self.cursor_col <= self.lines[self.cursor_line].len() {
                                self.lines[self.cursor_line].insert(self.cursor_col, ch);
                                self.cursor_col += 1;
                            }
                        }
                    }
                    self.dirty = true;
                    self.status_message = Some(String::from("Pasted"));
                }
                self.ensure_cursor_visible();
                return true;
            }
            
            // ── Arrow keys (Shift = extend selection) ──
            KEY_UP => {
                if shift_held && self.selection_anchor.is_none() {
                    self.selection_anchor = Some((self.cursor_line, self.cursor_col));
                } else if !shift_held {
                    self.selection_anchor = None;
                }
                if self.cursor_line > 0 {
                    self.cursor_line -= 1;
                    self.clamp_cursor_col();
                }
                self.ensure_cursor_visible();
                return true;
            }
            KEY_DOWN => {
                if shift_held && self.selection_anchor.is_none() {
                    self.selection_anchor = Some((self.cursor_line, self.cursor_col));
                } else if !shift_held {
                    self.selection_anchor = None;
                }
                if self.cursor_line + 1 < self.lines.len() {
                    self.cursor_line += 1;
                    self.clamp_cursor_col();
                }
                self.ensure_cursor_visible();
                return true;
            }
            KEY_LEFT => {
                if shift_held && self.selection_anchor.is_none() {
                    self.selection_anchor = Some((self.cursor_line, self.cursor_col));
                } else if !shift_held {
                    self.selection_anchor = None;
                }
                if self.cursor_col > 0 {
                    self.cursor_col -= 1;
                } else if self.cursor_line > 0 {
                    self.cursor_line -= 1;
                    self.cursor_col = self.lines[self.cursor_line].len();
                }
                self.ensure_cursor_visible();
                return true;
            }
            KEY_RIGHT => {
                if shift_held && self.selection_anchor.is_none() {
                    self.selection_anchor = Some((self.cursor_line, self.cursor_col));
                } else if !shift_held {
                    self.selection_anchor = None;
                }
                let line_length = if self.cursor_line < self.lines.len() { self.lines[self.cursor_line].len() } else { 0 };
                if self.cursor_col < line_length {
                    self.cursor_col += 1;
                } else if self.cursor_line + 1 < self.lines.len() {
                    self.cursor_line += 1;
                    self.cursor_col = 0;
                }
                self.ensure_cursor_visible();
                return true;
            }
            KEY_HOME => {
                if shift_held && self.selection_anchor.is_none() {
                    self.selection_anchor = Some((self.cursor_line, self.cursor_col));
                } else if !shift_held {
                    self.selection_anchor = None;
                }
                self.cursor_col = 0;
                return true;
            }
            KEY_END => {
                if shift_held && self.selection_anchor.is_none() {
                    self.selection_anchor = Some((self.cursor_line, self.cursor_col));
                } else if !shift_held {
                    self.selection_anchor = None;
                }
                let line_length = if self.cursor_line < self.lines.len() { self.lines[self.cursor_line].len() } else { 0 };
                self.cursor_col = line_length;
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
                if self.selection_anchor.is_some() {
                    self.delete_selection();
                    self.ensure_cursor_visible();
                    return true;
                }
                self.save_undo_state();
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
                if self.selection_anchor.is_some() {
                    self.delete_selection();
                    return true;
                }
                self.save_undo_state();
                if self.cursor_line < self.lines.len() {
                    let line_length = self.lines[self.cursor_line].len();
                    if self.cursor_col < line_length {
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
                if self.selection_anchor.is_some() {
                    self.delete_selection();
                }
                self.save_undo_state();
                if self.cursor_line < self.lines.len() {
                    // Clamp cursor_col to line length to prevent split_off panic
                    self.cursor_col = self.cursor_col.min(self.lines[self.cursor_line].len());
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
            
            // ── Tab / Shift+Tab (indent/outdent) ──
            0x09 => {
                self.save_undo_state();
                if let Some((sl, _sc, el, _ec)) = self.get_selection_range() {
                    // Block indent/outdent: operate on all selected lines
                    if shift_held {
                        // Shift+Tab: outdent selected lines by up to 4 spaces
                        for l in sl..=el.min(self.lines.len().saturating_sub(1)) {
                            let spaces = self.lines[l].chars().take(4).take_while(|c| *c == ' ').count();
                            if spaces > 0 {
                                self.lines[l] = String::from(&self.lines[l][spaces..]);
                            }
                        }
                    } else {
                        // Tab: indent selected lines by 4 spaces
                        for l in sl..=el.min(self.lines.len().saturating_sub(1)) {
                            self.lines[l] = format!("    {}", self.lines[l]);
                        }
                    }
                    self.dirty = true;
                } else if shift_held {
                    // Shift+Tab on single line: outdent
                    if self.cursor_line < self.lines.len() {
                        let spaces = self.lines[self.cursor_line].chars().take(4).take_while(|c| *c == ' ').count();
                        if spaces > 0 {
                            self.lines[self.cursor_line] = String::from(&self.lines[self.cursor_line][spaces..]);
                            self.cursor_col = self.cursor_col.saturating_sub(spaces);
                            self.dirty = true;
                        }
                    }
                } else {
                    // Plain Tab: insert 4 spaces
                    if self.cursor_line < self.lines.len() {
                        for _ in 0..4 {
                            self.lines[self.cursor_line].insert(self.cursor_col, ' ');
                            self.cursor_col += 1;
                        }
                        self.dirty = true;
                    }
                }
                return true;
            }
            
            // ── Printable characters ──
            c if c >= 0x20 && c < 0x7F => {
                if self.selection_anchor.is_some() {
                    self.delete_selection();
                }
                self.save_undo_state();
                if self.cursor_line < self.lines.len() {
                    let ch = c as char;
                    if self.cursor_col <= self.lines[self.cursor_line].len() {
                        self.lines[self.cursor_line].insert(self.cursor_col, ch);
                        self.cursor_col += 1;
                        self.dirty = true;
                        
                        // Auto-close brackets
                        let close = // Pattern matching — Rust's exhaustive branching construct.
match ch {
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

    // ═══════════════════════════════════════════════════════════════════════
    // PHASE 1: EDITOR PRODUCTIVITY FEATURES
    // ═══════════════════════════════════════════════════════════════════════

    /// Toggle line comment (Ctrl+/) — adds or removes "//" prefix
    fn toggle_comment(&mut self) {
        self.save_undo_state();
        let comment_prefix = // Pattern matching — Rust's exhaustive branching construct.
match self.language {
            Language::Rust | Language::C | Language::JavaScript => "// ",
            Language::Python => "# ",
            Language::Toml => "# ",
            _ => "// ",
        };

        // Determine range of lines to toggle
        let (start, end) = if let Some((sl, _sc, el, _ec)) = self.get_selection_range() {
            (sl, el)
        } else {
            (self.cursor_line, self.cursor_line)
        };

        // Check if ALL lines in range are already commented
        let all_commented = (start..=end.min(self.lines.len().saturating_sub(1)))
            .all(|l| self.lines[l].trim_start().starts_with(comment_prefix.trim_end()));

        for l in start..=end.min(self.lines.len().saturating_sub(1)) {
            if all_commented {
                // Uncomment: remove first occurrence of comment prefix
                let trimmed = &self.lines[l];
                if let Some(pos) = trimmed.find(comment_prefix) {
                    self.lines[l] = format!("{}{}", &trimmed[..pos], &trimmed[pos + comment_prefix.len()..]);
                } else if let Some(pos) = trimmed.find(comment_prefix.trim_end()) {
                    // Handle "//text" without space after
                    let prefix_no_space = comment_prefix.trim_end();
                    self.lines[l] = format!("{}{}", &trimmed[..pos], &trimmed[pos + prefix_no_space.len()..]);
                }
            } else {
                // Comment: find indentation level and insert comment prefix after it
                let indent_length = self.lines[l].chars().take_while(|c| *c == ' ' || *c == '\t').count();
                self.lines[l] = format!("{}{}{}", &self.lines[l][..indent_length], comment_prefix, &self.lines[l][indent_length..]);
            }
        }
        self.dirty = true;
        self.status_message = Some(String::from(if all_commented { "Uncommented" } else { "Commented" }));
    }

    /// Delete current line (Ctrl+Shift+K)
    fn delete_current_line(&mut self) {
        if self.lines.len() <= 1 {
            self.save_undo_state();
            self.lines[0] = String::new();
            self.cursor_col = 0;
            self.dirty = true;
            return;
        }
        self.save_undo_state();
        self.lines.remove(self.cursor_line);
        if self.cursor_line >= self.lines.len() {
            self.cursor_line = self.lines.len().saturating_sub(1);
        }
        self.clamp_cursor_col();
        self.dirty = true;
        self.ensure_cursor_visible();
    }

    /// Duplicate current line (Ctrl+Shift+D)
    fn duplicate_line(&mut self) {
        if self.cursor_line < self.lines.len() {
            self.save_undo_state();
            let dup = self.lines[self.cursor_line].clone();
            self.lines.insert(self.cursor_line + 1, dup);
            self.cursor_line += 1;
            self.dirty = true;
            self.ensure_cursor_visible();
        }
    }

    /// Move current line up (Alt+Up)
    fn move_line_up(&mut self) {
        if self.cursor_line > 0 && self.cursor_line < self.lines.len() {
            self.save_undo_state();
            self.lines.swap(self.cursor_line, self.cursor_line - 1);
            self.cursor_line -= 1;
            self.dirty = true;
            self.ensure_cursor_visible();
        }
    }

    /// Move current line down (Alt+Down)
    fn move_line_down(&mut self) {
        if self.cursor_line + 1 < self.lines.len() {
            self.save_undo_state();
            self.lines.swap(self.cursor_line, self.cursor_line + 1);
            self.cursor_line += 1;
            self.dirty = true;
            self.ensure_cursor_visible();
        }
    }

    /// Move cursor one word to the left (Ctrl+Left)
    fn word_left(&mut self) {
        if self.cursor_line >= self.lines.len() { return; }
        if self.cursor_col == 0 {
            // Jump to end of previous line
            if self.cursor_line > 0 {
                self.cursor_line -= 1;
                self.cursor_col = self.lines[self.cursor_line].len();
            }
            return;
        }
        let line = &self.lines[self.cursor_line];
        let bytes = line.as_bytes();
        let mut pos = self.cursor_col.min(bytes.len());
        // Skip whitespace backwards
        while pos > 0 && bytes[pos - 1] == b' ' {
            pos -= 1;
        }
        // Skip word characters backwards
        while pos > 0 && bytes[pos - 1] != b' ' {
            pos -= 1;
        }
        self.cursor_col = pos;
    }

    /// Move cursor one word to the right (Ctrl+Right)
    fn word_right(&mut self) {
        if self.cursor_line >= self.lines.len() { return; }
        let line = &self.lines[self.cursor_line];
        let bytes = line.as_bytes();
        let len = bytes.len();
        if self.cursor_col >= len {
            // Jump to start of next line
            if self.cursor_line + 1 < self.lines.len() {
                self.cursor_line += 1;
                self.cursor_col = 0;
            }
            return;
        }
        let mut pos = self.cursor_col;
        // Skip word characters forward
        while pos < len && bytes[pos] != b' ' {
            pos += 1;
        }
        // Skip whitespace forward
        while pos < len && bytes[pos] == b' ' {
            pos += 1;
        }
        self.cursor_col = pos;
    }

    /// Find the matching bracket for the character at cursor position
    pub fn update_matching_bracket(&mut self) {
        self.matching_bracket = None;
        if self.cursor_line >= self.lines.len() { return; }
        let line = &self.lines[self.cursor_line];
        if self.cursor_col >= line.len() { return; }
        
        let ch = line.as_bytes()[self.cursor_col];
        let (target, forward) = // Pattern matching — Rust's exhaustive branching construct.
match ch {
            b'(' => (b')', true),
            b')' => (b'(', false),
            b'{' => (b'}', true),
            b'}' => (b'{', false),
            b'[' => (b']', true),
            b']' => (b'[', false),
            _ => return,
        };
        
        let mut depth: i32 = 0;
        if forward {
            let mut l = self.cursor_line;
            let mut c = self.cursor_col;
            while l < self.lines.len() {
                let bytes = self.lines[l].as_bytes();
                while c < bytes.len() {
                    if bytes[c] == ch { depth += 1; }
                    else if bytes[c] == target {
                        depth -= 1;
                        if depth == 0 {
                            self.matching_bracket = Some((l, c));
                            return;
                        }
                    }
                    c += 1;
                }
                l += 1;
                c = 0;
            }
        } else {
            let mut l = self.cursor_line;
            let mut c = self.cursor_col as i32;
                        // Infinite loop — runs until an explicit `break`.
loop {
                let bytes = self.lines[l].as_bytes();
                while c >= 0 {
                    let cu = c as usize;
                    if cu < bytes.len() {
                        if bytes[cu] == ch { depth += 1; }
                        else if bytes[cu] == target {
                            depth -= 1;
                            if depth == 0 {
                                self.matching_bracket = Some((l, cu));
                                return;
                            }
                        }
                    }
                    c -= 1;
                }
                if l == 0 { break; }
                l -= 1;
                c = self.lines[l].len() as i32 - 1;
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SYNTAX HIGHLIGHTING
// ═══════════════════════════════════════════════════════════════════════════════

/// Token type for syntax coloring
#[derive(Clone, Copy, PartialEq)]
// Enumeration — a type that can be one of several variants.
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
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const COLOR_KEYWORD: u32   = 0xFF569CD6; // Blue (let, fn, if, use...)
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const COLOR_TYPE: u32      = 0xFF4EC9B0; // Teal (u32, String, Self...)
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const COLOR_STRING: u32    = 0xFFCE9178; // Orange (string literals)
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const COLOR_COMMENT: u32   = 0xFF6A9955; // Green (comments)
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const COLOR_NUMBER: u32    = 0xFFB5CEA8; // Light green (numbers)
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const COLOR_FUNCTION: u32  = 0xFFDCDCAA; // Yellow (function names)
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const COLOR_MACRO: u32     = 0xFF4FC1FF; // Bright blue (macros!)
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const COLOR_ATTRIBUTE: u32 = 0xFFD7BA7D; // Gold (#[derive(...)])
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const COLOR_LIFETIME: u32  = 0xFF569CD6; // Blue ('a, 'static)
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const COLOR_OPERATOR: u32  = 0xFFD4D4D4; // Light gray
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const COLOR_BRACKET: u32   = 0xFFFFD700; // Gold/yellow brackets
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const COLOR_NORMAL: u32    = 0xFFD4D4D4; // Default text
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const COLOR_LINE_NUMBER: u32  = 0xFF858585; // Gutter line numbers
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const COLOR_ACTIVE_LINE: u32 = 0xFF858585; // Active line number
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const COLOR_BG: u32        = 0xFF1E1E2E; // Editor background (dark)
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const COLOR_GUTTER_BG: u32 = 0xFF1E1E2E; // Gutter background
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const COLOR_ACTIVE_LINE_BG: u32 = 0xFF2A2D3A; // Current line highlight
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const COLOR_STATUS_BG: u32 = 0xFF007ACC; // Status bar (blue)
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const COLOR_STATUS_FG: u32 = 0xFFFFFFFF; // Status bar text
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const COLOR_CURSOR: u32    = 0xFFAEAFAD; // Cursor color
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const COLOR_BREADCRUMB_BG: u32 = 0xFF252526; // Breadcrumb/tab bar
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const COLOR_TAB_ACTIVE: u32 = 0xFF1E1E2E; // Active tab background
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const COLOR_TAB_INACTIVE: u32 = 0xFF2D2D2D; // Inactive tab

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
        // Pattern matching — Rust's exhaustive branching construct.
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
// PYTHON TOKENIZER
// ═══════════════════════════════════════════════════════════════════════════════

const PYTHON_KEYWORDS: &[&str] = &[
    "False", "None", "True", "and", "as", "assert", "async", "await",
    "break", "class", "continue", "def", "del", "elif", "else", "except",
    "finally", "for", "from", "global", "if", "import", "in", "is",
    "lambda", "nonlocal", "not", "or", "pass", "raise", "return", "try",
    "while", "with", "yield",
];

// Compile-time constant — evaluated at compilation, zero runtime cost.
const PYTHON_BUILTINS: &[&str] = &[
    "int", "float", "str", "bool", "list", "dict", "tuple", "set",
    "frozenset", "bytes", "bytearray", "range", "type", "object",
    "print", "len", "input", "open", "super", "self", "cls",
    "Exception", "ValueError", "TypeError", "KeyError", "IndexError",
    "RuntimeError", "StopIteration", "OSError", "IOError",
];

// Public function — callable from other modules.
pub fn tokenize_python_line(line: &str) -> Vec<ColorSpan> {
    let mut spans = Vec::new();
    let bytes = line.as_bytes();
    let len = bytes.len();
    let mut i = 0;
    
    while i < len {
        let ch = bytes[i];
        
        // Comment
        if ch == b'#' {
            spans.push(ColorSpan { start: i, end: len, kind: TokenKind::Comment });
            break;
        }
        
        // Decorator
        if ch == b'@' {
            let start = i;
            i += 1;
            while i < len && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_' || bytes[i] == b'.') { i += 1; }
            spans.push(ColorSpan { start, end: i, kind: TokenKind::Attribute });
            continue;
        }
        
        // String (single, double, triple-quoted)
        if ch == b'"' || ch == b'\'' {
            let start = i;
            let quote = ch;
            // Check for triple quote
            if i + 2 < len && bytes[i+1] == quote && bytes[i+2] == quote {
                i += 3;
                while i + 2 < len {
                    if bytes[i] == quote && bytes[i+1] == quote && bytes[i+2] == quote {
                        i += 3;
                        break;
                    }
                    if bytes[i] == b'\\' { i += 1; }
                    i += 1;
                }
                if i >= len { i = len; }
            } else {
                i += 1;
                while i < len {
                    if bytes[i] == b'\\' && i + 1 < len { i += 2; continue; }
                    if bytes[i] == quote { i += 1; break; }
                    i += 1;
                }
            }
            spans.push(ColorSpan { start, end: i, kind: TokenKind::String });
            continue;
        }
        
        // Number
        if ch.is_ascii_digit() {
            let start = i;
            i += 1;
            while i < len && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_' || bytes[i] == b'.') { i += 1; }
            spans.push(ColorSpan { start, end: i, kind: TokenKind::Number });
            continue;
        }
        
        // Identifier / keyword
        if ch.is_ascii_alphabetic() || ch == b'_' {
            let start = i;
            while i < len && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') { i += 1; }
            let word = &line[start..i];
            let is_fn = i < len && bytes[i] == b'(';
            let kind = if PYTHON_KEYWORDS.contains(&word) {
                TokenKind::Keyword
            } else if PYTHON_BUILTINS.contains(&word) {
                TokenKind::Type
            } else if is_fn {
                TokenKind::Function
            } else {
                TokenKind::Normal
            };
            spans.push(ColorSpan { start, end: i, kind });
            continue;
        }
        
        // Brackets
        if ch == b'(' || ch == b')' || ch == b'{' || ch == b'}' || ch == b'[' || ch == b']' {
            spans.push(ColorSpan { start: i, end: i + 1, kind: TokenKind::Bracket });
            i += 1;
            continue;
        }
        
        // Operators
        if ch == b'+' || ch == b'-' || ch == b'*' || ch == b'/' || ch == b'=' || ch == b'!'
            || ch == b'<' || ch == b'>' || ch == b'&' || ch == b'|' || ch == b'^'
            || ch == b':' || ch == b';' || ch == b',' || ch == b'.' || ch == b'%' {
            spans.push(ColorSpan { start: i, end: i + 1, kind: TokenKind::Operator });
            i += 1;
            continue;
        }
        
        i += 1;
    }
    spans
}

// ═══════════════════════════════════════════════════════════════════════════════
// JAVASCRIPT / C / C++ TOKENIZER
// ═══════════════════════════════════════════════════════════════════════════════

const JS_KEYWORDS: &[&str] = &[
    "break", "case", "catch", "class", "const", "continue", "debugger",
    "default", "delete", "do", "else", "export", "extends", "finally",
    "for", "function", "if", "import", "in", "instanceof", "let", "new",
    "of", "return", "super", "switch", "this", "throw", "try", "typeof",
    "var", "void", "while", "with", "yield", "async", "await", "from",
    "static", "get", "set",
];

// Compile-time constant — evaluated at compilation, zero runtime cost.
const JS_TYPES: &[&str] = &[
    "Array", "Boolean", "Date", "Error", "Function", "JSON", "Map", "Math",
    "Number", "Object", "Promise", "Proxy", "RegExp", "Set", "String",
    "Symbol", "WeakMap", "WeakSet", "console", "document", "window",
    "null", "undefined", "true", "false", "NaN", "Infinity",
];

// Compile-time constant — evaluated at compilation, zero runtime cost.
const C_KEYWORDS: &[&str] = &[
    "auto", "break", "case", "char", "const", "continue", "default", "do",
    "double", "else", "enum", "extern", "float", "for", "goto", "if",
    "inline", "int", "long", "register", "restrict", "return", "short",
    "signed", "sizeof", "static", "struct", "switch", "typedef", "union",
    "unsigned", "void", "volatile", "while",
    // C++ extras
    "bool", "catch", "class", "constexpr", "delete", "dynamic_cast",
    "explicit", "false", "friend", "mutable", "namespace", "new",
    "noexcept", "nullptr", "operator", "override", "private", "protected",
    "public", "reinterpret_cast", "static_assert", "static_cast",
    "template", "this", "throw", "true", "try", "typeid", "typename",
    "using", "virtual",
];

// Compile-time constant — evaluated at compilation, zero runtime cost.
const C_TYPES: &[&str] = &[
    "int8_t", "int16_t", "int32_t", "int64_t", "uint8_t", "uint16_t",
    "uint32_t", "uint64_t", "size_t", "ssize_t", "ptrdiff_t", "intptr_t",
    "uintptr_t", "FILE", "NULL", "EOF", "string", "vector", "map",
    "set", "pair", "shared_ptr", "unique_ptr", "weak_ptr",
];

// Public function — callable from other modules.
pub fn tokenize_js_c_line(line: &str, is_c: bool) -> Vec<ColorSpan> {
    let mut spans = Vec::new();
    let bytes = line.as_bytes();
    let len = bytes.len();
    let mut i = 0;
    let keywords: &[&str] = if is_c { C_KEYWORDS } else { JS_KEYWORDS };
    let types: &[&str] = if is_c { C_TYPES } else { JS_TYPES };
    
    while i < len {
        let ch = bytes[i];
        
        // Line comment
        if ch == b'/' && i + 1 < len && bytes[i + 1] == b'/' {
            spans.push(ColorSpan { start: i, end: len, kind: TokenKind::Comment });
            break;
        }
        
        // Block comment (single-line portion)
        if ch == b'/' && i + 1 < len && bytes[i + 1] == b'*' {
            let start = i;
            i += 2;
            while i + 1 < len {
                if bytes[i] == b'*' && bytes[i+1] == b'/' { i += 2; break; }
                i += 1;
            }
            if i >= len { i = len; }
            spans.push(ColorSpan { start, end: i, kind: TokenKind::Comment });
            continue;
        }
        
        // Preprocessor directives (C/C++)
        if is_c && ch == b'#' {
            spans.push(ColorSpan { start: i, end: len, kind: TokenKind::Macro });
            break;
        }
        
        // String
        if ch == b'"' || ch == b'\'' || ch == b'`' {
            let start = i;
            let quote = ch;
            i += 1;
            while i < len {
                if bytes[i] == b'\\' && i + 1 < len { i += 2; continue; }
                if bytes[i] == quote { i += 1; break; }
                i += 1;
            }
            spans.push(ColorSpan { start, end: i, kind: TokenKind::String });
            continue;
        }
        
        // Number
        if ch.is_ascii_digit() || (ch == b'.' && i + 1 < len && bytes[i+1].is_ascii_digit()) {
            let start = i;
            i += 1;
            while i < len && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_' || bytes[i] == b'.') { i += 1; }
            spans.push(ColorSpan { start, end: i, kind: TokenKind::Number });
            continue;
        }
        
        // Identifier / keyword
        if ch.is_ascii_alphabetic() || ch == b'_' || ch == b'$' {
            let start = i;
            while i < len && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_' || bytes[i] == b'$') { i += 1; }
            let word = &line[start..i];
            let is_fn = i < len && bytes[i] == b'(';
            let kind = if keywords.contains(&word) {
                TokenKind::Keyword
            } else if types.contains(&word) {
                TokenKind::Type
            } else if is_fn {
                TokenKind::Function
            } else {
                TokenKind::Normal
            };
            spans.push(ColorSpan { start, end: i, kind });
            continue;
        }
        
        // Brackets
        if ch == b'(' || ch == b')' || ch == b'{' || ch == b'}' || ch == b'[' || ch == b']' {
            spans.push(ColorSpan { start: i, end: i + 1, kind: TokenKind::Bracket });
            i += 1;
            continue;
        }
        
        // Operators
        if ch == b'+' || ch == b'-' || ch == b'*' || ch == b'/' || ch == b'=' || ch == b'!'
            || ch == b'<' || ch == b'>' || ch == b'&' || ch == b'|' || ch == b'^'
            || ch == b':' || ch == b';' || ch == b',' || ch == b'.' || ch == b'?' || ch == b'%' {
            spans.push(ColorSpan { start: i, end: i + 1, kind: TokenKind::Operator });
            i += 1;
            continue;
        }
        
        i += 1;
    }
    spans
}

// ═══════════════════════════════════════════════════════════════════════════════
// TOML TOKENIZER
// ═══════════════════════════════════════════════════════════════════════════════

pub fn tokenize_toml_line(line: &str) -> Vec<ColorSpan> {
    let mut spans = Vec::new();
    let bytes = line.as_bytes();
    let len = bytes.len();
    let trimmed = line.trim_start();
    
    // Comment lines
    if trimmed.starts_with('#') {
        spans.push(ColorSpan { start: 0, end: len, kind: TokenKind::Comment });
        return spans;
    }
    
    // Section headers [section] or [[array]]
    if trimmed.starts_with('[') {
        let offset = len - trimmed.len();
        spans.push(ColorSpan { start: offset, end: len, kind: TokenKind::Attribute });
        return spans;
    }
    
    // Key = Value
    let mut i = 0;
    // Key part (before =)
    while i < len && bytes[i] != b'=' {
        i += 1;
    }
    if i < len {
        // Key
        spans.push(ColorSpan { start: 0, end: i, kind: TokenKind::Type });
        // Equals sign
        spans.push(ColorSpan { start: i, end: i + 1, kind: TokenKind::Operator });
        i += 1;
        // Skip whitespace
        while i < len && bytes[i] == b' ' { i += 1; }
        
        if i < len {
            let value_start = i;
            let vch = bytes[i];
            if vch == b'"' || vch == b'\'' {
                // String value
                spans.push(ColorSpan { start: value_start, end: len, kind: TokenKind::String });
            } else if vch == b't' || vch == b'f' {
                // Boolean
                spans.push(ColorSpan { start: value_start, end: len, kind: TokenKind::Keyword });
            } else if vch.is_ascii_digit() || vch == b'-' || vch == b'+' {
                // Number
                spans.push(ColorSpan { start: value_start, end: len, kind: TokenKind::Number });
            } else if vch == b'[' {
                // Array
                spans.push(ColorSpan { start: value_start, end: len, kind: TokenKind::Bracket });
            } else {
                spans.push(ColorSpan { start: value_start, end: len, kind: TokenKind::Normal });
            }
            // Inline comment
            if let Some(hash_pos) = line[value_start..].find('#') {
                let absolute_position = value_start + hash_pos;
                spans.push(ColorSpan { start: absolute_position, end: len, kind: TokenKind::Comment });
            }
        }
    } else {
        // No '=' — just content
        spans.push(ColorSpan { start: 0, end: len, kind: TokenKind::Normal });
    }
    spans
}

// ═══════════════════════════════════════════════════════════════════════════════
// MARKDOWN TOKENIZER
// ═══════════════════════════════════════════════════════════════════════════════

pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const COLOR_MD_HEADING: u32 = 0xFF569CD6;  // Blue headings
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const COLOR_MD_BOLD: u32    = 0xFFD7BA7D;   // Gold bold
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const COLOR_MD_CODE: u32    = 0xFFCE9178;    // Orange inline code
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const COLOR_MD_LINK: u32    = 0xFF4EC9B0;    // Teal links
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const COLOR_MD_LIST: u32    = 0xFF569CD6;    // Blue list markers

pub fn tokenize_markdown_line(line: &str) -> Vec<ColorSpan> {
    let mut spans = Vec::new();
    let bytes = line.as_bytes();
    let len = bytes.len();
    let trimmed = line.trim_start();
    
    // Headings: # ## ### etc
    if trimmed.starts_with('#') {
        spans.push(ColorSpan { start: 0, end: len, kind: TokenKind::Keyword });
        return spans;
    }
    
    // Code block fence ```
    if trimmed.starts_with("```") {
        spans.push(ColorSpan { start: 0, end: len, kind: TokenKind::String });
        return spans;
    }
    
    // List items: - * + or 1.
    if trimmed.starts_with("- ") || trimmed.starts_with("* ") || trimmed.starts_with("+ ") {
        let offset = len - trimmed.len();
        spans.push(ColorSpan { start: offset, end: offset + 2, kind: TokenKind::Macro });
        // Rest is normal
        if offset + 2 < len {
            spans.push(ColorSpan { start: offset + 2, end: len, kind: TokenKind::Normal });
        }
        return spans;
    }
    
    // Scan for inline elements
    let mut i = 0;
    let mut last_end = 0;
    
    while i < len {
        // Inline code `...`
        if bytes[i] == b'`' {
            if last_end < i {
                spans.push(ColorSpan { start: last_end, end: i, kind: TokenKind::Normal });
            }
            let start = i;
            i += 1;
            while i < len && bytes[i] != b'`' { i += 1; }
            if i < len { i += 1; }
            spans.push(ColorSpan { start, end: i, kind: TokenKind::String });
            last_end = i;
            continue;
        }
        
        // Bold **...**
        if bytes[i] == b'*' && i + 1 < len && bytes[i+1] == b'*' {
            if last_end < i {
                spans.push(ColorSpan { start: last_end, end: i, kind: TokenKind::Normal });
            }
            let start = i;
            i += 2;
            while i + 1 < len {
                if bytes[i] == b'*' && bytes[i+1] == b'*' { i += 2; break; }
                i += 1;
            }
            spans.push(ColorSpan { start, end: i, kind: TokenKind::Attribute });
            last_end = i;
            continue;
        }
        
        // Link [text](url)
        if bytes[i] == b'[' {
            if last_end < i {
                spans.push(ColorSpan { start: last_end, end: i, kind: TokenKind::Normal });
            }
            let start = i;
            while i < len && bytes[i] != b')' { i += 1; }
            if i < len { i += 1; }
            spans.push(ColorSpan { start, end: i, kind: TokenKind::Type });
            last_end = i;
            continue;
        }
        
        i += 1;
    }
    
    if last_end < len {
        spans.push(ColorSpan { start: last_end, end: len, kind: TokenKind::Normal });
    }
    
    spans
}

/// Dispatch tokenizer based on language mode
pub fn tokenize_line(line: &str, lang: Language) -> Vec<ColorSpan> {
        // Pattern matching — Rust's exhaustive branching construct.
match lang {
        Language::Rust => tokenize_rust_line(line),
        Language::Python => tokenize_python_line(line),
        Language::JavaScript => tokenize_js_c_line(line, false),
        Language::C => tokenize_js_c_line(line, true),
        Language::Toml => tokenize_toml_line(line),
        Language::Markdown => tokenize_markdown_line(line),
        Language::Plain => Vec::new(),
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
    let status_h: i32 = 22;
    let menu_bar_h: i32 = 22;
    let tab_bar_h: i32 = 28;
    let breadcrumb_h: i32 = 18;
    let total_header = menu_bar_h + tab_bar_h + breadcrumb_h;
    
    // Dynamic gutter width based on line count
    let digit_count = if state.lines.len() >= 10000 { 5 }
        else if state.lines.len() >= 1000 { 4 }
        else if state.lines.len() >= 100 { 3 }
        else { 2 };
    let gutter_w = (digit_count + 2) * char_w;
    
    let code_x = x + gutter_w;
    let code_y = y + total_header;
    let code_w = w as i32 - gutter_w;
    let code_h = h as i32 - total_header - status_h;
    let visible_lines = (code_h / line_h).max(1) as usize;
    
    // Update scroll + bracket matching
    if state.cursor_line < state.scroll_y {
        state.scroll_y = state.cursor_line;
    }
    if state.cursor_line >= state.scroll_y + visible_lines {
        state.scroll_y = state.cursor_line - visible_lines + 1;
    }
    state.blink_counter += 1;
    state.update_matching_bracket();
    
    // ── Menu bar (File Edit Selection View Go ...) ──
    let menu_y = y;
    crate::framebuffer::fill_rect(x as u32, menu_y as u32, w, menu_bar_h as u32, 0xFF333333);
    // Bottom border
    crate::framebuffer::fill_rect(x as u32, (menu_y + menu_bar_h - 1) as u32, w, 1, 0xFF252526);
    let menus = ["File", "Edit", "Selection", "View", "Go", "Run", "Terminal", "Help"];
    let mut mx = x + 8;
    for label in &menus {
        draw_text_fn(mx, menu_y + 4, label, 0xFFCCCCCC);
        mx += (label.len() as i32 + 2) * char_w;
    }
    
    // ── Tab bar ──
    let tab_y = y + menu_bar_h;
    crate::framebuffer::fill_rect(x as u32, tab_y as u32, w, tab_bar_h as u32, 0xFF252526);
    // Active tab
    let tab_name = state.file_path.as_ref().map(|p| {
        p.rsplit('/').next().unwrap_or(p.as_str())
    }).unwrap_or("untitled");
    let dirty_marker = if state.dirty { " *" } else { "" };
    // Language icon (ASCII only)
    let lang_prefix = // Pattern matching — Rust's exhaustive branching construct.
match state.language {
        Language::Rust => "RS",
        Language::Python => "PY",
        Language::JavaScript => "JS",
        Language::C => " C",
        Language::Toml => "TL",
        Language::Markdown => "MD",
        Language::Plain => "  ",
    };
    let tab_label = format!(" {} {} {}  x", lang_prefix, tab_name, dirty_marker);
    let tab_w = ((tab_label.len() as u32) * 8 + 4).min(w);
    // Active tab background (editor bg color = looks connected to editor)
    crate::framebuffer::fill_rect(x as u32, tab_y as u32, tab_w, tab_bar_h as u32, COLOR_BG);
    // Top accent line (blue)
    crate::framebuffer::fill_rect(x as u32, tab_y as u32, tab_w, 2, 0xFF007ACC);
    // Tab text
    draw_text_fn(x + 4, tab_y + 8, &tab_label, COLOR_NORMAL);
    
    // ── Breadcrumb bar ──
    let bc_y = tab_y + tab_bar_h;
    crate::framebuffer::fill_rect(x as u32, bc_y as u32, w, breadcrumb_h as u32, 0xFF1E1E1E);
    if let Some(ref path) = state.file_path {
        // Build breadcrumb with " > " separators, no allocations for Vec
        let mut bx = x + gutter_w + 4;
        let mut start = 0;
        let bytes = path.as_bytes();
        for i in 0..=bytes.len() {
            if i == bytes.len() || bytes[i] == b'/' {
                if i > start {
                    let part = &path[start..i];
                    if start > 0 {
                        draw_text_fn(bx, bc_y + 2, " > ", 0xFF666666);
                        bx += 3 * char_w;
                    }
                    draw_text_fn(bx, bc_y + 2, part, 0xFF858585);
                    bx += part.len() as i32 * char_w;
                }
                start = i + 1;
            }
        }
    } else {
        draw_text_fn(x + gutter_w + 4, bc_y + 2, "untitled", 0xFF858585);
    }
    // Separator line
    crate::framebuffer::fill_rect(x as u32, (bc_y + breadcrumb_h - 1) as u32, w, 1, 0xFF333333);
    
    // ── Editor background ──
    crate::framebuffer::fill_rect(x as u32, code_y as u32, w, code_h as u32, COLOR_BG);
    
    // ── Gutter background ──
    crate::framebuffer::fill_rect(x as u32, code_y as u32, gutter_w as u32, code_h as u32, COLOR_GUTTER_BG);
    crate::framebuffer::fill_rect((x + gutter_w - 1) as u32, code_y as u32, 1, code_h as u32, 0xFF333333);
    
    // ── Render visible lines ──
    for vi in 0..visible_lines {
        let line_index = state.scroll_y + vi;
        if line_index >= state.lines.len() { break; }
        
        let ly = code_y + (vi as i32 * line_h);
        if ly + line_h > code_y + code_h { break; }
        
        let is_current_line = line_index == state.cursor_line;
        
        // ── Current line highlight ──
        if is_current_line {
            crate::framebuffer::fill_rect(
                x as u32, ly as u32,
                gutter_w as u32, line_h as u32,
                0xFF1A1D26,
            );
            crate::framebuffer::fill_rect(
                code_x as u32, ly as u32,
                code_w as u32, line_h as u32,
                COLOR_ACTIVE_LINE_BG,
            );
        }
        
        // ── Selection highlight ──
        if let Some((sl, sc, el, ec)) = state.get_selection_range() {
            if line_index >= sl && line_index <= el {
                let line_length = state.lines[line_index].len();
                let sel_start = if line_index == sl { sc.min(line_length) } else { 0 };
                let sel_end = if line_index == el { ec.min(line_length) } else { line_length };
                if sel_start < sel_end {
                    let sx = code_x + 4 + (sel_start as i32 * char_w);
                    let software = ((sel_end - sel_start) as i32 * char_w) as u32;
                    crate::framebuffer::fill_rect(
                        sx as u32, ly as u32, software, line_h as u32, 0xFF264F78,
                    );
                }
            }
        }
        
        // ── Indent guides ──
        let line_ref = &state.lines[line_index];
        let mut indent_spaces = 0usize;
        for b in line_ref.bytes() {
            if b == b' ' { indent_spaces += 1; } else { break; }
        }
        let indent_levels = indent_spaces / 4;
        for level in 0..indent_levels {
            let guide_x = code_x + 4 + (level as i32 * 4 * char_w);
            if guide_x < x + w as i32 {
                let guide_color = if is_current_line { 0xFF505050 } else { 0xFF404040 };
                crate::framebuffer::fill_rect(guide_x as u32, ly as u32, 1, line_h as u32, guide_color);
            }
        }
        
        // ── Bracket matching highlight ──
        if let Some((ml, mc)) = state.matching_bracket {
            if line_index == ml {
                let bx = code_x + 4 + (mc as i32 * char_w);
                crate::framebuffer::fill_rect(bx as u32, ly as u32, char_w as u32, line_h as u32, 0xFF3A3D41);
                crate::framebuffer::fill_rect(bx as u32, ly as u32, char_w as u32, 1, 0xFF888888);
                crate::framebuffer::fill_rect(bx as u32, (ly + line_h - 1) as u32, char_w as u32, 1, 0xFF888888);
                crate::framebuffer::fill_rect(bx as u32, ly as u32, 1, line_h as u32, 0xFF888888);
                crate::framebuffer::fill_rect((bx + char_w - 1) as u32, ly as u32, 1, line_h as u32, 0xFF888888);
            }
            if line_index == state.cursor_line && state.cursor_col < state.lines[line_index].len() {
                let cb = state.lines[line_index].as_bytes()[state.cursor_col];
                if matches!(cb, b'(' | b')' | b'{' | b'}' | b'[' | b']') {
                    let cbx = code_x + 4 + (state.cursor_col as i32 * char_w);
                    crate::framebuffer::fill_rect(cbx as u32, ly as u32, char_w as u32, line_h as u32, 0xFF3A3D41);
                    crate::framebuffer::fill_rect(cbx as u32, ly as u32, char_w as u32, 1, 0xFF888888);
                    crate::framebuffer::fill_rect(cbx as u32, (ly + line_h - 1) as u32, char_w as u32, 1, 0xFF888888);
                    crate::framebuffer::fill_rect(cbx as u32, ly as u32, 1, line_h as u32, 0xFF888888);
                    crate::framebuffer::fill_rect((cbx + char_w - 1) as u32, ly as u32, 1, line_h as u32, 0xFF888888);
                }
            }
        }
        
        // ── Line number ──
        let line_number_str = format!("{:>width$} ", line_index + 1, width = digit_count as usize);
        let number_color = if is_current_line { 0xFFC6C6C6 } else { COLOR_LINE_NUMBER };
        draw_text_fn(x + 2, ly, &line_number_str, number_color);
        
        // ── Code with syntax highlighting (all languages) ──
        let line = &state.lines[line_index];
        let tokens = tokenize_line(line, state.language);
        if !tokens.is_empty() {
            for span in &tokens {
                let color = token_color(span.kind);
                let text_segment = &line[span.start..span.end];
                let sx = code_x + 4 + (span.start as i32 * char_w);
                if sx < x + w as i32 {
                    draw_text_fn(sx, ly, text_segment, color);
                }
            }
        } else if !line.is_empty() {
            draw_text_fn(code_x + 4, ly, line, COLOR_NORMAL);
        }
        
        // ── Cursor ──
        if is_current_line {
            let blink_on = (state.blink_counter / 30) % 2 == 0;
            if blink_on {
                let cx = code_x + 4 + (state.cursor_col as i32 * char_w);
                crate::framebuffer::fill_rect(cx as u32, ly as u32, 2, line_h as u32, COLOR_CURSOR);
            }
        }
    }
    
    // ── Scrollbar ──
    if state.lines.len() > visible_lines {
        let sb_x = (x + w as i32 - 10) as u32;
        let sb_h = code_h as u32;
        let thumb_h = ((visible_lines as u32 * sb_h) / state.lines.len() as u32).max(20);
        let thumb_y = (state.scroll_y as u32 * (sb_h - thumb_h)) / state.lines.len().saturating_sub(visible_lines) as u32;
        crate::framebuffer::fill_rect(sb_x + 3, code_y as u32, 7, sb_h, 0xFF252526);
        crate::framebuffer::fill_rounded_rect(sb_x + 3, code_y as u32 + thumb_y, 7, thumb_h, 3, 0xFF6A6A6A);
    }
    
    // ── Find/Replace bar (floating, VSCode-style) ──
    if state.find_query.is_some() {
        let find_bar_h: i32 = if state.replace_text.is_some() { 56 } else { 32 };
        let find_w: i32 = 370.min(code_w);
        let find_x = x + w as i32 - find_w - 20;
        let find_y = code_y + 4;
        // Shadow + background
        crate::framebuffer::fill_rect((find_x + 2) as u32, (find_y + 2) as u32, find_w as u32, find_bar_h as u32, 0xFF0A0A0A);
        crate::framebuffer::fill_rect(find_x as u32, find_y as u32, find_w as u32, find_bar_h as u32, 0xFF252526);
        crate::framebuffer::fill_rect(find_x as u32, find_y as u32, find_w as u32, 1, 0xFF007ACC);
        
        let query = state.find_query.as_deref().unwrap_or("");
        let match_information = if state.find_matches.is_empty() {
            if query.is_empty() { String::new() } else { String::from(" No results") }
        } else {
            format!(" {}/{}", state.find_match_idx + 1, state.find_matches.len())
        };
        let find_active = !state.find_replace_mode;
        let ff_x = find_x + 8;
        let ff_y = find_y + 6;
        let ff_w = find_w - 100;
        crate::framebuffer::fill_rect(ff_x as u32, ff_y as u32, ff_w as u32, 18, if find_active { 0xFF3C3C3C } else { 0xFF333333 });
        if find_active {
            crate::framebuffer::fill_rect(ff_x as u32, (ff_y + 17) as u32, ff_w as u32, 1, 0xFF007ACC);
        }
        draw_text_fn(ff_x + 4, ff_y + 2, query, 0xFFCCCCCC);
        draw_text_fn(find_x + find_w - 90, ff_y + 2, &match_information, 0xFF858585);
        
        if let Some(ref replace) = state.replace_text {
            let rf_y = find_y + 30;
            let rep_active = state.find_replace_mode;
            crate::framebuffer::fill_rect(ff_x as u32, rf_y as u32, ff_w as u32, 18, if rep_active { 0xFF3C3C3C } else { 0xFF333333 });
            if rep_active {
                crate::framebuffer::fill_rect(ff_x as u32, (rf_y + 17) as u32, ff_w as u32, 1, 0xFF007ACC);
            }
            draw_text_fn(ff_x + 4, rf_y + 2, replace, 0xFFCCCCCC);
        }
        
        // Highlight find matches
        let q_length = query.len();
        if q_length > 0 {
            for &(ml, mc) in &state.find_matches {
                if ml >= state.scroll_y && ml < state.scroll_y + visible_lines {
                    let vi = ml - state.scroll_y;
                    let mly = code_y + (vi as i32 * line_h);
                    let mmx = code_x + 4 + (mc as i32 * char_w);
                    let mw = (q_length as i32 * char_w) as u32;
                    crate::framebuffer::fill_rect(mmx as u32, mly as u32, mw, line_h as u32, 0xFF613214);
                    if state.find_match_idx < state.find_matches.len() && state.find_matches[state.find_match_idx] == (ml, mc) {
                        crate::framebuffer::fill_rect(mmx as u32, mly as u32, mw, 1, 0xFFE8AB53);
                        crate::framebuffer::fill_rect(mmx as u32, (mly + line_h - 1) as u32, mw, 1, 0xFFE8AB53);
                    }
                }
            }
        }
    }
    
    // ── Go-to-line dialog ──
    if let Some(ref input) = state.goto_line_input {
        let dialog_w: i32 = 320.min(w as i32 - 40);
        let dialog_h: i32 = 32;
        let dialog_x = x + (w as i32 - dialog_w) / 2;
        let dialog_y = y + total_header + 2;
        crate::framebuffer::fill_rect((dialog_x + 2) as u32, (dialog_y + 2) as u32, dialog_w as u32, dialog_h as u32, 0xFF0A0A0A);
        crate::framebuffer::fill_rect(dialog_x as u32, dialog_y as u32, dialog_w as u32, dialog_h as u32, 0xFF252526);
        crate::framebuffer::fill_rect(dialog_x as u32, dialog_y as u32, dialog_w as u32, 2, 0xFF007ACC);
        let input_y = dialog_y + 6;
        crate::framebuffer::fill_rect((dialog_x + 8) as u32, input_y as u32, (dialog_w - 16) as u32, 18, 0xFF3C3C3C);
        crate::framebuffer::fill_rect((dialog_x + 8) as u32, (input_y + 17) as u32, (dialog_w - 16) as u32, 1, 0xFF007ACC);
        let goto_text = format!(":{}", input);
        draw_text_fn(dialog_x + 12, input_y + 2, &goto_text, 0xFFCCCCCC);
        let hint = format!("Go to Line (1-{})", state.lines.len());
        let hint_x = dialog_x + dialog_w - (hint.len() as i32 * char_w) - 12;
        draw_text_fn(hint_x, input_y + 2, &hint, 0xFF666666);
    }

    // ── Status bar (VSCode-style, blue background) ──
    let status_y = y + total_header + code_h;
    crate::framebuffer::fill_rect(x as u32, status_y as u32, w, status_h as u32, COLOR_STATUS_BG);
    
    // Left: branch + saved/modified
    let mut slx = x + 8;
    draw_text_fn(slx, status_y + 4, "@ main", COLOR_STATUS_FG);
    slx += 7 * char_w;
    crate::framebuffer::fill_rect(slx as u32, (status_y + 4) as u32, 1, 14, 0xFF1A6DAA);
    slx += 6;
    if state.dirty {
        draw_text_fn(slx, status_y + 4, "* Modified", 0xFFFFD166);
    } else {
        draw_text_fn(slx, status_y + 4, "Saved", COLOR_STATUS_FG);
    }
    
    // Right: Spaces, Encoding, Language, Ln/Col
    let position_str = format!("Ln {}, Col {}", state.cursor_line + 1, state.cursor_col + 1);
    let lang_name = state.language.name();
    // Draw from right to left
    let mut srx = x + w as i32 - 8;
    // Position
    srx -= position_str.len() as i32 * char_w;
    draw_text_fn(srx, status_y + 4, &position_str, COLOR_STATUS_FG);
    srx -= 10;
    crate::framebuffer::fill_rect(srx as u32, (status_y + 4) as u32, 1, 14, 0xFF1A6DAA);
    srx -= 6;
    // Language
    srx -= lang_name.len() as i32 * char_w;
    draw_text_fn(srx, status_y + 4, lang_name, COLOR_STATUS_FG);
    srx -= 10;
    crate::framebuffer::fill_rect(srx as u32, (status_y + 4) as u32, 1, 14, 0xFF1A6DAA);
    srx -= 6;
    // Encoding
    srx -= 5 * char_w;
    draw_text_fn(srx, status_y + 4, "UTF-8", COLOR_STATUS_FG);
    srx -= 10;
    crate::framebuffer::fill_rect(srx as u32, (status_y + 4) as u32, 1, 14, 0xFF1A6DAA);
    srx -= 6;
    // Spaces
    srx -= 9 * char_w;
    draw_text_fn(srx, status_y + 4, "Spaces: 4", COLOR_STATUS_FG);
}
