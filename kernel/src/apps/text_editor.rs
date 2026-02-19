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
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            selection_anchor: None,
            find_query: None,
            replace_text: None,
            find_replace_mode: false,
            find_matches: Vec::new(),
            find_match_idx: 0,
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
            for (line_idx, line) in self.lines.iter().enumerate() {
                let mut start = 0;
                while start + q.len() <= line.len() {
                    if &line[start..start + q.len()] == q.as_str() {
                        self.find_matches.push((line_idx, start));
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
    fn find_prev(&mut self) {
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
                let q_len = q.len();
                self.lines[line] = format!("{}{}{}", &self.lines[line][..col], rep, &self.lines[line][col + q_len..]);
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
        
        let shift_held = crate::keyboard::is_key_pressed(0x2A) || crate::keyboard::is_key_pressed(0x36);
        
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
            
            // ── Ctrl+A (select all) ──
            0x01 => {
                self.selection_anchor = Some((0, 0));
                let last_line = self.lines.len().saturating_sub(1);
                let last_col = if last_line < self.lines.len() { self.lines[last_line].len() } else { 0 };
                self.cursor_line = last_line;
                self.cursor_col = last_col;
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
                let line_len = if self.cursor_line < self.lines.len() { self.lines[self.cursor_line].len() } else { 0 };
                if self.cursor_col < line_len {
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
            
            // ── Tab ──
            0x09 => {
                self.save_undo_state();
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
        
        // ── Selection highlight ──
        if let Some((sl, sc, el, ec)) = state.get_selection_range() {
            if line_idx >= sl && line_idx <= el {
                let line_len = state.lines[line_idx].len();
                let sel_start = if line_idx == sl { sc.min(line_len) } else { 0 };
                let sel_end = if line_idx == el { ec.min(line_len) } else { line_len };
                if sel_start < sel_end {
                    let sx = code_x + 4 + (sel_start as i32 * char_w);
                    let sw = ((sel_end - sel_start) as i32 * char_w) as u32;
                    crate::framebuffer::fill_rect(
                        sx as u32, ly as u32,
                        sw, line_h as u32,
                        0xFF264F78, // VSCode-style selection blue
                    );
                }
            }
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
    
    // ── Find/Replace bar ──
    if state.find_query.is_some() {
        let find_bar_h: i32 = if state.replace_text.is_some() { 44 } else { 22 };
        let find_y = code_y; // Draw at top of code area
        crate::framebuffer::fill_rect(code_x as u32, find_y as u32, code_w as u32, find_bar_h as u32, 0xFF252526);
        // Find field
        let find_label = "Find: ";
        let query = state.find_query.as_deref().unwrap_or("");
        let match_info = if state.find_matches.is_empty() {
            if query.is_empty() { String::new() } else { String::from(" (0 results)") }
        } else {
            format!(" ({}/{})", state.find_match_idx + 1, state.find_matches.len())
        };
        let find_indicator = if !state.find_replace_mode { ">" } else { " " };
        let find_text = format!("{}{}{}{}", find_indicator, find_label, query, match_info);
        draw_text_fn(code_x + 4, find_y + 3, &find_text, 0xFFCCCCCC);
        // Replace field
        if let Some(ref replace) = state.replace_text {
            let rep_indicator = if state.find_replace_mode { ">" } else { " " };
            let rep_text = format!("{}Replace: {}  [Enter]=Replace [Ctrl+A]=All", rep_indicator, replace);
            draw_text_fn(code_x + 4, find_y + 22 + 3, &rep_text, 0xFFCCCCCC);
        }
        // Highlight find matches in code area
        for &(ml, mc) in &state.find_matches {
            if ml >= state.scroll_y && ml < state.scroll_y + visible_lines {
                let vi = ml - state.scroll_y;
                let mly = code_y + find_bar_h + (vi as i32 * line_h);
                let q_len = state.find_query.as_ref().map(|q| q.len()).unwrap_or(0);
                if q_len > 0 {
                    let mx = code_x + 4 + (mc as i32 * char_w);
                    let mw = (q_len as i32 * char_w) as u32;
                    crate::framebuffer::fill_rect(mx as u32, mly as u32, mw, line_h as u32, 0xFF613214);
                }
            }
        }
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
