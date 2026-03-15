//! TrustEdit — A nano-like terminal text editor for TrustOS
//!
//! Keybindings:
//!   Ctrl+S  — Save
//!   Ctrl+X  — Exit (prompts to save if modified)
//!   Ctrl+Q  — Quit without saving
//!   Ctrl+G  — Go to line
//!   Ctrl+F  — Find text
//!   Ctrl+K  — Cut current line
//!   Ctrl+U  — Paste cut line
//!   Ctrl+C  — Copy current line
//!   Ctrl+Z  — Undo
//!   Ctrl+L  — Refresh screen
//!   Arrows / Home / End / PgUp / PgDn — Navigation
//!   Tab     — Insert 4 spaces

use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use alloc::format;

use crate::framebuffer::{
    COLOR_WHITE, COLOR_BLACK, COLOR_CYAN, COLOR_YELLOW, COLOR_RED,
    COLOR_GREEN, COLOR_GRAY, COLOR_BRIGHT_GREEN, COLOR_BLUE, COLOR_MAGENTA,
};

/// Maximum undo history entries
const MAXIMUM_UNDO: usize = 64;

struct EditorState {
    lines: Vec<String>,
    cursor_row: usize,    // line index in document
    cursor_column: usize,    // column in current line
    scroll_row: usize,    // first visible line
    scroll_column: usize,    // horizontal scroll offset
    filename: String,
    modified: bool,
    status_message: String,
    status_time: u64,
    clipboard: Vec<String>,    // cut/copy buffer
    undo_stack: Vec<UndoEntry>,
    search_query: String,
    running: bool,
    term_rows: usize,
    term_cols: usize,
}

// #[derive] — auto-generates trait implementations at compile time.
#[derive(Clone)]
enum UndoEntry {
    InsertChar { row: usize, column: usize },
    DeleteChar { row: usize, column: usize, character: char },
    InsertLine { row: usize },
    DeleteLine { row: usize, content: String },
    SplitLine { row: usize, column: usize },
    JoinLines { row: usize, column: usize },
}

// Implementation block — defines methods for the type above.
impl EditorState {
    fn new(filename: &str) -> Self {
        let cols = crate::framebuffer::width() as usize / 8;
        let rows = crate::framebuffer::height() as usize / 16;
        
        Self {
            lines: vec![String::new()],
            cursor_row: 0,
            cursor_column: 0,
            scroll_row: 0,
            scroll_column: 0,
            filename: String::from(filename),
            modified: false,
            status_message: String::from("Ctrl+S = Save | Ctrl+X = Exit | Ctrl+F = Find | Ctrl+G = Goto"),
            status_time: crate::time::uptime_mouse(),
            clipboard: Vec::new(),
            undo_stack: Vec::new(),
            search_query: String::new(),
            running: true,
            term_rows: rows,
            term_cols: cols,
        }
    }

    /// Number of rows available for editing (total - 2 for header and status)
    fn edit_rows(&self) -> usize {
        if self.term_rows > 3 { self.term_rows - 3 } else { 1 }
    }

    /// The line-number gutter width
    fn gutter_width(&self) -> usize {
        let maximum_line = self.lines.len();
        let digits = if maximum_line < 10 { 1 }
            else if maximum_line < 100 { 2 }
            else if maximum_line < 1000 { 3 }
            else if maximum_line < 10000 { 4 }
            else { 5 };
        digits + 2 // space + separator + space
    }

    fn push_undo(&mut self, entry: UndoEntry) {
        if self.undo_stack.len() >= MAXIMUM_UNDO {
            self.undo_stack.remove(0);
        }
        self.undo_stack.push(entry);
    }

    fn set_status(&mut self, message: &str) {
        self.status_message = String::from(message);
        self.status_time = crate::time::uptime_mouse();
    }
}

/// Entry point — called from shell
pub(super) fn command_nano(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: nano <filename>");
        crate::println!("       edit <filename>");
        return;
    }

    let filename = args[0];
    let mut state = EditorState::new(filename);

    // Try to load existing file
    load_file(&mut state);

    // Enter raw mode — clear screen and take over
    crate::framebuffer::clear();
    draw_screen(&state);

    // Main editor loop
    while state.running {
        if let Some(key) = crate::keyboard::read_char() {
            handle_key(&mut state, key);
            scroll_into_view(&mut state);
            draw_screen(&state);
        } else {
            core::hint::spin_loop();
        }
    }

    // Restore terminal
    crate::framebuffer::clear();
    crate::framebuffer::set_cursor(0, 0);
}

fn load_file(state: &mut EditorState) {
    let path = &state.filename;
    
    // Try VFS first
    let content: Option<String> = if path.starts_with("/mnt/") || path.starts_with("/dev/") || path.starts_with("/proc/") {
        crate::vfs::read_to_string(path).ok()
    } else {
        // Try ramfs
        crate::ramfs::with_filesystem(|fs| {
            fs.read_file(path)
                .map(|bytes| String::from(core::str::from_utf8(bytes).unwrap_or("")))
                .ok()
        })
    };

        // Pattern matching — Rust's exhaustive branching construct.
match content {
        Some(ref text) => {
            state.lines = text.lines().map(|l| String::from(l)).collect();
            if state.lines.is_empty() {
                state.lines.push(String::new());
            }
            state.set_status(&format!("Opened \"{}\" — {} lines", state.filename, state.lines.len()));
        }
        None => {
            state.lines = vec![String::new()];
            state.set_status(&format!("New file: \"{}\"", state.filename));
        }
    }
}

fn save_file(state: &mut EditorState) {
    let content = state.lines.join("\n");
    let path = &state.filename;
    
    let result = if path.starts_with("/mnt/") || path.starts_with("/dev/") {
        crate::vfs::write_file(path, content.as_bytes()).map_error(|_| "VFS write error")
    } else {
        crate::ramfs::with_filesystem(|fs| {
            if !fs.exists(path) {
                let _ = fs.touch(path);
            }
            fs.write_file(path, content.as_bytes())
                .map_error(|_| "RamFS write error")
        })
    };

        // Pattern matching — Rust's exhaustive branching construct.
match result {
        Ok(()) => {
            state.modified = false;
            state.set_status(&format!("Saved \"{}\" — {} lines, {} bytes", 
                state.filename, state.lines.len(), content.len()));
        }
        Err(e) => {
            state.set_status(&format!("ERROR: Could not save: {}", e));
        }
    }
}

fn scroll_into_view(state: &mut EditorState) {
    let edit_rows = state.edit_rows();
    
    // Vertical scroll
    if state.cursor_row < state.scroll_row {
        state.scroll_row = state.cursor_row;
    }
    if state.cursor_row >= state.scroll_row + edit_rows {
        state.scroll_row = state.cursor_row - edit_rows + 1;
    }
    
    // Horizontal scroll
    let gutter = state.gutter_width();
    let visible_cols = if state.term_cols > gutter { state.term_cols - gutter } else { 1 };
    if state.cursor_column < state.scroll_column {
        state.scroll_column = state.cursor_column;
    }
    if state.cursor_column >= state.scroll_column + visible_cols {
        state.scroll_column = state.cursor_column - visible_cols + 1;
    }
}

fn draw_screen(state: &EditorState) {
    let cols = state.term_cols;
    let rows = state.term_rows;
    let edit_rows = state.edit_rows();
    let gutter = state.gutter_width();

    // ── Header bar ──
    let title = if state.modified {
        format!(" TrustEdit — {}  [modified]", state.filename)
    } else {
        format!(" TrustEdit — {}", state.filename)
    };
    let header = pad_right(&title, cols);
    // Draw header on pixel row 0
    crate::framebuffer::fill_rect(0, 0, cols as u32 * 8, 16, 0xFF1A1A2E);
    crate::framebuffer::draw_text(&header, 0, 0, COLOR_CYAN);

    // ── Content area ──
    for screen_row in 0..edit_rows {
        let doc_row = state.scroll_row + screen_row;
        let y = ((screen_row + 1) * 16) as u32;

        // Clear the row
        crate::framebuffer::fill_rect(0, y, cols as u32 * 8, 16, 0xFF0D0D1A);

        if doc_row < state.lines.len() {
            // Line number gutter
            let line_number = format!("{:>width$} ", doc_row + 1, width = gutter - 2);
            // Gutter bg
            crate::framebuffer::fill_rect(0, y, gutter as u32 * 8, 16, 0xFF15152A);
            // Current line highlight
            if doc_row == state.cursor_row {
                crate::framebuffer::fill_rect(gutter as u32 * 8, y, (cols - gutter) as u32 * 8, 16, 0xFF1A1A30);
            }
            crate::framebuffer::draw_text(&line_number, 0, y, COLOR_GRAY);

            // Line content
            let line = &state.lines[doc_row];
            let visible_cols = cols - gutter;
            let visible_text: String = if state.scroll_column < line.len() {
                let end = core::cmp::minimum(line.len(), state.scroll_column + visible_cols);
                String::from(&line[state.scroll_column..end])
            } else {
                String::new()
            };
            
            // Syntax-aware coloring
            draw_syntax_line(&visible_text, (gutter * 8) as u32, y, &state.filename);
        } else {
            // Empty row — draw tilde
            crate::framebuffer::fill_rect(0, y, gutter as u32 * 8, 16, 0xFF15152A);
            crate::framebuffer::draw_text("~", 0, y, COLOR_BLUE);
        }
    }

    // ── Status bar ──
    let status_y = ((edit_rows + 1) * 16) as u32;
    crate::framebuffer::fill_rect(0, status_y, cols as u32 * 8, 16, 0xFF2A2A4A);
    let left = format!(" Ln {}, Col {} | {} lines | {}", 
        state.cursor_row + 1, 
        state.cursor_column + 1,
        state.lines.len(),
        if state.modified { "MODIFIED" } else { "saved" }
    );
    let right = format!("{} ", detect_filetype(&state.filename));
    let padding = if cols > left.len() + right.len() { cols - left.len() - right.len() } else { 0 };
    let status_line = format!("{}{:>pad$}{}", left, "", right, pad = padding);
    crate::framebuffer::draw_text(&status_line, 0, status_y, COLOR_GREEN);

    // ── Message bar ──
    let message_y = ((edit_rows + 2) * 16) as u32;
    crate::framebuffer::fill_rect(0, message_y, cols as u32 * 8, 16, 0xFF0D0D1A);
    let age = crate::time::uptime_mouse().saturating_sub(state.status_time);
    if age < 8000 {
        crate::framebuffer::draw_text(&state.status_message, 0, message_y, COLOR_YELLOW);
    } else {
        crate::framebuffer::draw_text(
            " Ctrl+S=Save  Ctrl+X=Exit  Ctrl+F=Find  Ctrl+G=Goto  Ctrl+K=Cut  Ctrl+U=Paste",
            0, message_y, COLOR_GRAY,
        );
    }

    // ── Cursor ──
    let cursor_screen_row = state.cursor_row - state.scroll_row;
    let cursor_screen_column = state.cursor_column - state.scroll_column + gutter;
    let cx = (cursor_screen_column * 8) as u32;
    let cy = ((cursor_screen_row + 1) * 16) as u32;
    // Draw blinking cursor (simple block)
    crate::framebuffer::fill_rect(cx, cy, 8, 16, COLOR_WHITE);
    // Draw the character under cursor in inverse
    if state.cursor_row < state.lines.len() {
        let line = &state.lines[state.cursor_row];
        if state.cursor_column < line.len() {
            let character = &line[state.cursor_column..state.cursor_column + 1];
            crate::framebuffer::draw_text(character, cx, cy, 0xFF0D0D1A);
        }
    }
}

/// Simple syntax highlighting based on file extension
fn draw_syntax_line(text: &str, x: u32, y: u32, filename: &str) {
    let ext = filename.rsplit('.').next().unwrap_or("");
    let is_code = matches!(ext, "rs" | "c" | "h" | "py" | "js" | "ts" | "sh" | "toml" | "json" | "cfg" | "conf");
    
    if !is_code || text.is_empty() {
        crate::framebuffer::draw_text(text, x, y, COLOR_WHITE);
        return;
    }

    // Rust/C-like keywords
    let keywords = [
        "fn ", "let ", "mut ", "pub ", "use ", "mod ", "struct ", "enum ", "impl ",
        "trait ", "type ", "const ", "static ", "match ", "if ", "else ", "for ",
        "while ", "loop ", "return ", "break ", "continue ", "async ", "await ",
        "unsafe ", "extern ", "crate ", "self ", "super ", "where ",
        "def ", "class ", "import ", "from ", "try ", "except ", "with ",
        "function ", "var ", "const ", "export ", "import ",
    ];

    let trimmed = text.trim_start();
    
    // Comment line
    if trimmed.starts_with("//") || trimmed.starts_with('#') {
        crate::framebuffer::draw_text(text, x, y, COLOR_GRAY);
        return;
    }

    // Lines starting with keywords
    for kw in &keywords {
        if trimmed.starts_with(kw) || trimmed.starts_with(kw.trim_end()) {
            // Draw with keyword highlighting - simplified approach
            crate::framebuffer::draw_text(text, x, y, COLOR_CYAN);
            return;
        }
    }

    // String literals — simplistic check
    if trimmed.contains('"') {
        crate::framebuffer::draw_text(text, x, y, COLOR_YELLOW);
        return;
    }

    // Macros (Rust)
    if trimmed.contains("!(") || trimmed.contains("!{") {
        crate::framebuffer::draw_text(text, x, y, COLOR_MAGENTA);
        return;
    }

    // Default
    crate::framebuffer::draw_text(text, x, y, COLOR_WHITE);
}

fn detect_filetype(filename: &str) -> &str {
        // Pattern matching — Rust's exhaustive branching construct.
match filename.rsplit('.').next().unwrap_or("") {
        "rs" => "Rust",
        "py" => "Python",
        "js" => "JavaScript",
        "ts" => "TypeScript",
        "c" | "h" => "C",
        "cpp" | "hpp" => "C++",
        "sh" => "Shell",
        "toml" => "TOML",
        "json" => "JSON",
        "md" => "Markdown",
        "txt" => "Text",
        "cfg" | "conf" | "ini" => "Config",
        "html" => "HTML",
        "css" => "CSS",
        _ => "Plain Text",
    }
}

fn pad_right(s: &str, width: usize) -> String {
    if s.len() >= width {
        String::from(&s[..width])
    } else {
        let mut out = String::from(s);
        for _ in 0..(width - s.len()) {
            out.push(' ');
        }
        out
    }
}

fn handle_key(state: &mut EditorState, key: u8) {
        // Pattern matching — Rust's exhaustive branching construct.
match key {
        // Ctrl+S — Save
        0x13 => save_file(state),
        
        // Ctrl+X — Exit
        0x18 => {
            if state.modified {
                state.set_status("File has unsaved changes! Ctrl+S to save, Ctrl+Q to quit without saving");
            } else {
                state.running = false;
            }
        }

        // Ctrl+Q — Quit without saving
        0x11 => {
            state.running = false;
        }
        
        // Ctrl+G — Go to line
        0x07 => {
            if let Some(line_str) = prompt_input(state, "Go to line: ") {
                if let Ok(line) = line_str.trim().parse::<usize>() {
                    if line > 0 && line <= state.lines.len() {
                        state.cursor_row = line - 1;
                        state.cursor_column = 0;
                        state.set_status(&format!("Jumped to line {}", line));
                    } else {
                        state.set_status("Invalid line number");
                    }
                }
            }
        }
        
        // Ctrl+F — Find
        0x06 => {
            let default = state.search_query.clone();
            let prompt_message = if default.is_empty() {
                String::from("Search: ")
            } else {
                format!("Search [{}]: ", default)
            };
            if let Some(query) = prompt_input(state, &prompt_message) {
                let q = if query.is_empty() { default } else { query };
                if !q.is_empty() {
                    state.search_query = q.clone();
                    find_next(state, &q);
                }
            }
        }
        
        // Ctrl+K — Cut line
        0x0B => {
            if state.cursor_row < state.lines.len() {
                let removed = state.lines.remove(state.cursor_row);
                state.push_undo(UndoEntry::DeleteLine {
                    row: state.cursor_row,
                    content: removed.clone(),
                });
                state.clipboard = vec![removed];
                if state.lines.is_empty() {
                    state.lines.push(String::new());
                }
                if state.cursor_row >= state.lines.len() {
                    state.cursor_row = state.lines.len() - 1;
                }
                state.cursor_column = 0;
                state.modified = true;
                state.set_status("Line cut");
            }
        }
        
        // Ctrl+U — Paste
        0x15 => {
            if !state.clipboard.is_empty() {
                for (i, line) in state.clipboard.clone().iter().enumerate() {
                    state.lines.insert(state.cursor_row + i, line.clone());
                    state.push_undo(UndoEntry::InsertLine { row: state.cursor_row + i });
                }
                state.modified = true;
                state.set_status("Pasted");
            }
        }
        
        // Ctrl+C — Copy line
        0x03 => {
            if state.cursor_row < state.lines.len() {
                state.clipboard = vec![state.lines[state.cursor_row].clone()];
                state.set_status("Line copied");
            }
        }
        
        // Ctrl+Z — Undo
        0x1A => {
            undo(state);
        }
        
        // Ctrl+L — Refresh
        0x0C => {
            crate::framebuffer::clear();
            state.set_status("Screen refreshed");
        }
        
        // Escape — cancel / do nothing
        27 => {}
        
        // Arrow keys
        crate::keyboard::KEY_UP => {
            if state.cursor_row > 0 {
                state.cursor_row -= 1;
                clamp_cursor_column(state);
            }
        }
        crate::keyboard::KEY_DOWN => {
            if state.cursor_row + 1 < state.lines.len() {
                state.cursor_row += 1;
                clamp_cursor_column(state);
            }
        }
        crate::keyboard::KEY_LEFT => {
            if state.cursor_column > 0 {
                state.cursor_column -= 1;
            } else if state.cursor_row > 0 {
                state.cursor_row -= 1;
                state.cursor_column = state.lines[state.cursor_row].len();
            }
        }
        crate::keyboard::KEY_RIGHT => {
            let line_length = state.lines.get(state.cursor_row).map(|l| l.len()).unwrap_or(0);
            if state.cursor_column < line_length {
                state.cursor_column += 1;
            } else if state.cursor_row + 1 < state.lines.len() {
                state.cursor_row += 1;
                state.cursor_column = 0;
            }
        }
        
        // Home
        crate::keyboard::KEY_HOME => {
            state.cursor_column = 0;
        }
        // End
        crate::keyboard::KEY_END => {
            state.cursor_column = state.lines.get(state.cursor_row).map(|l| l.len()).unwrap_or(0);
        }
        
        // Page Up
        crate::keyboard::KEY_PGUP => {
            let jump = state.edit_rows();
            state.cursor_row = state.cursor_row.saturating_sub(jump);
            clamp_cursor_column(state);
        }
        // Page Down
        crate::keyboard::KEY_PGDOWN => {
            let jump = state.edit_rows();
            state.cursor_row = core::cmp::minimum(state.cursor_row + jump, state.lines.len().saturating_sub(1));
            clamp_cursor_column(state);
        }
        
        // Delete
        crate::keyboard::KEY_DELETE => {
            let line_length = state.lines.get(state.cursor_row).map(|l| l.len()).unwrap_or(0);
            if state.cursor_column < line_length {
                let character = state.lines[state.cursor_row].remove(state.cursor_column);
                state.push_undo(UndoEntry::DeleteChar { row: state.cursor_row, column: state.cursor_column, character });
                state.modified = true;
            } else if state.cursor_row + 1 < state.lines.len() {
                // Join with next line
                let next = state.lines.remove(state.cursor_row + 1);
                state.push_undo(UndoEntry::JoinLines { row: state.cursor_row, column: state.cursor_column });
                state.lines[state.cursor_row].push_str(&next);
                state.modified = true;
            }
        }
        
        // Backspace
        0x08 => {
            if state.cursor_column > 0 {
                state.cursor_column -= 1;
                let character = state.lines[state.cursor_row].remove(state.cursor_column);
                state.push_undo(UndoEntry::DeleteChar { row: state.cursor_row, column: state.cursor_column, character });
                state.modified = true;
            } else if state.cursor_row > 0 {
                // Join with previous line
                let current = state.lines.remove(state.cursor_row);
                state.cursor_row -= 1;
                state.cursor_column = state.lines[state.cursor_row].len();
                state.push_undo(UndoEntry::JoinLines { row: state.cursor_row, column: state.cursor_column });
                state.lines[state.cursor_row].push_str(&current);
                state.modified = true;
            }
        }
        
        // Enter
        0x0A => {
            let current_line = &state.lines[state.cursor_row];
            let remainder = String::from(&current_line[state.cursor_column..]);
            state.lines[state.cursor_row] = String::from(&current_line[..state.cursor_column]);
            state.push_undo(UndoEntry::SplitLine { row: state.cursor_row, column: state.cursor_column });
            state.cursor_row += 1;
            state.lines.insert(state.cursor_row, remainder);
            state.cursor_column = 0;
            state.modified = true;
        }
        
        // Tab — insert 4 spaces
        0x09 => {
            for _ in 0..4 {
                state.lines[state.cursor_row].insert(state.cursor_column, ' ');
                state.push_undo(UndoEntry::InsertChar { row: state.cursor_row, column: state.cursor_column });
                state.cursor_column += 1;
            }
            state.modified = true;
        }
        
        // Regular printable character
        character if character >= 32 && character < 127 => {
            state.lines[state.cursor_row].insert(state.cursor_column, character as char);
            state.push_undo(UndoEntry::InsertChar { row: state.cursor_row, column: state.cursor_column });
            state.cursor_column += 1;
            state.modified = true;
        }
        
        _ => {}
    }
}

fn clamp_cursor_column(state: &mut EditorState) {
    let line_length = state.lines.get(state.cursor_row).map(|l| l.len()).unwrap_or(0);
    if state.cursor_column > line_length {
        state.cursor_column = line_length;
    }
}

fn undo(state: &mut EditorState) {
    if let Some(entry) = state.undo_stack.pop() {
                // Pattern matching — Rust's exhaustive branching construct.
match entry {
            UndoEntry::InsertChar { row, column } => {
                if row < state.lines.len() && column < state.lines[row].len() {
                    state.lines[row].remove(column);
                    state.cursor_row = row;
                    state.cursor_column = column;
                }
            }
            UndoEntry::DeleteChar { row, column, character } => {
                if row < state.lines.len() {
                    state.lines[row].insert(column, character);
                    state.cursor_row = row;
                    state.cursor_column = column + 1;
                }
            }
            UndoEntry::InsertLine { row } => {
                if row < state.lines.len() {
                    state.lines.remove(row);
                    if state.lines.is_empty() {
                        state.lines.push(String::new());
                    }
                    state.cursor_row = row.minimum(state.lines.len() - 1);
                    state.cursor_column = 0;
                }
            }
            UndoEntry::DeleteLine { row, content } => {
                state.lines.insert(row, content);
                state.cursor_row = row;
                state.cursor_column = 0;
            }
            UndoEntry::SplitLine { row, column } => {
                if row + 1 < state.lines.len() {
                    let next = state.lines.remove(row + 1);
                    state.lines[row].push_str(&next);
                    state.cursor_row = row;
                    state.cursor_column = column;
                }
            }
            UndoEntry::JoinLines { row, column } => {
                if row < state.lines.len() {
                    let remainder = String::from(&state.lines[row][column..]);
                    state.lines[row] = String::from(&state.lines[row][..column]);
                    state.lines.insert(row + 1, remainder);
                    state.cursor_row = row + 1;
                    state.cursor_column = 0;
                }
            }
        }
        state.modified = true;
        state.set_status("Undo");
    } else {
        state.set_status("Nothing to undo");
    }
}

fn find_next(state: &mut EditorState, query: &str) {
    // Search from current position forward
    let start_row = state.cursor_row;
    let start_column = state.cursor_column + 1;
    
    for row in start_row..state.lines.len() {
        let column_start = if row == start_row { start_column } else { 0 };
        if column_start < state.lines[row].len() {
            if let Some(position) = state.lines[row][column_start..].find(query) {
                state.cursor_row = row;
                state.cursor_column = column_start + position;
                state.set_status(&format!("Found \"{}\" at line {}", query, row + 1));
                return;
            }
        }
    }
    
    // Wrap around
    for row in 0..=start_row {
        let column_end = if row == start_row { start_column } else { state.lines[row].len() };
        if let Some(position) = state.lines[row][..column_end.minimum(state.lines[row].len())].find(query) {
            state.cursor_row = row;
            state.cursor_column = position;
            state.set_status(&format!("Found \"{}\" at line {} (wrapped)", query, row + 1));
            return;
        }
    }
    
    state.set_status(&format!("\"{}\" not found", query));
}

/// Show a prompt at the bottom of screen and read user input
fn prompt_input(state: &mut EditorState, prompt: &str) -> Option<String> {
    let cols = state.term_cols;
    let message_y = ((state.edit_rows() + 2) * 16) as u32;
    
    let mut input = String::new();
    
        // Infinite loop — runs until an explicit `break`.
loop {
        // Draw prompt
        crate::framebuffer::fill_rect(0, message_y, cols as u32 * 8, 16, 0xFF1A1A2E);
        let display = format!("{}{}", prompt, input);
        crate::framebuffer::draw_text(&display, 0, message_y, COLOR_YELLOW);
        // Cursor
        let cx = ((prompt.len() + input.len()) * 8) as u32;
        crate::framebuffer::fill_rect(cx, message_y, 8, 16, COLOR_WHITE);
        
        if let Some(key) = crate::keyboard::read_char() {
                        // Pattern matching — Rust's exhaustive branching construct.
match key {
                0x0A => return Some(input), // Enter
                27 => return None,           // Escape — cancel
                0x08 => { input.pop(); }     // Backspace
                character if character >= 32 && character < 127 => input.push(character as char),
                _ => {}
            }
        } else {
            core::hint::spin_loop();
        }
    }
}
