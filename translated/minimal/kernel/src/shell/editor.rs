















use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use alloc::format;

use crate::framebuffer::{
    R_, NE_, C_, D_, A_,
    B_, K_, G_, CF_, DM_,
};


const CJE_: usize = 64;

struct EditorState {
    lines: Vec<String>,
    cursor_row: usize,    
    cursor_col: usize,    
    scroll_row: usize,    
    scroll_col: usize,    
    filename: String,
    modified: bool,
    status_msg: String,
    status_time: u64,
    clipboard: Vec<String>,    
    undo_stack: Vec<UndoEntry>,
    search_query: String,
    running: bool,
    term_rows: usize,
    term_cols: usize,
}

#[derive(Clone)]
enum UndoEntry {
    InsertChar { row: usize, col: usize },
    DeleteChar { row: usize, col: usize, ch: char },
    InsertLine { row: usize },
    DeleteLine { row: usize, content: String },
    SplitLine { row: usize, col: usize },
    JoinLines { row: usize, col: usize },
}

impl EditorState {
    fn new(filename: &str) -> Self {
        let cols = crate::framebuffer::width() as usize / 8;
        let rows = crate::framebuffer::height() as usize / 16;
        
        Self {
            lines: vec![String::new()],
            cursor_row: 0,
            cursor_col: 0,
            scroll_row: 0,
            scroll_col: 0,
            filename: String::from(filename),
            modified: false,
            status_msg: String::from("Ctrl+S = Save | Ctrl+X = Exit | Ctrl+F = Find | Ctrl+G = Goto"),
            status_time: crate::time::uptime_ms(),
            clipboard: Vec::new(),
            undo_stack: Vec::new(),
            search_query: String::new(),
            running: true,
            term_rows: rows,
            term_cols: cols,
        }
    }

    
    fn edit_rows(&self) -> usize {
        if self.term_rows > 3 { self.term_rows - 3 } else { 1 }
    }

    
    fn gutter_width(&self) -> usize {
        let eub = self.lines.len();
        let eke = if eub < 10 { 1 }
            else if eub < 100 { 2 }
            else if eub < 1000 { 3 }
            else if eub < 10000 { 4 }
            else { 5 };
        eke + 2 
    }

    fn push_undo(&mut self, entry: UndoEntry) {
        if self.undo_stack.len() >= CJE_ {
            self.undo_stack.remove(0);
        }
        self.undo_stack.push(entry);
    }

    fn set_status(&mut self, bk: &str) {
        self.status_msg = String::from(bk);
        self.status_time = crate::time::uptime_ms();
    }
}


pub(super) fn kpw(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: nano <filename>");
        crate::println!("       edit <filename>");
        return;
    }

    let filename = args[0];
    let mut state = EditorState::new(filename);

    
    load_file(&mut state);

    
    crate::framebuffer::clear();
    htq(&state);

    
    while state.running {
        if let Some(key) = crate::keyboard::ya() {
            handle_key(&mut state, key);
            olv(&mut state);
            htq(&state);
        } else {
            core::hint::spin_loop();
        }
    }

    
    crate::framebuffer::clear();
    crate::framebuffer::afr(0, 0);
}

fn load_file(state: &mut EditorState) {
    let path = &state.filename;
    
    
    let content: Option<String> = if path.starts_with("/mnt/") || path.starts_with("/dev/") || path.starts_with("/proc/") {
        crate::vfs::gqh(path).ok()
    } else {
        
        crate::ramfs::bh(|fs| {
            fs.read_file(path)
                .map(|bytes| String::from(core::str::from_utf8(bytes).unwrap_or("")))
                .ok()
        })
    };

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
        crate::vfs::write_file(path, content.as_bytes()).map_err(|_| "VFS write error")
    } else {
        crate::ramfs::bh(|fs| {
            if !fs.exists(path) {
                let _ = fs.touch(path);
            }
            fs.write_file(path, content.as_bytes())
                .map_err(|_| "RamFS write error")
        })
    };

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

fn olv(state: &mut EditorState) {
    let edit_rows = state.edit_rows();
    
    
    if state.cursor_row < state.scroll_row {
        state.scroll_row = state.cursor_row;
    }
    if state.cursor_row >= state.scroll_row + edit_rows {
        state.scroll_row = state.cursor_row - edit_rows + 1;
    }
    
    
    let bgt = state.gutter_width();
    let fel = if state.term_cols > bgt { state.term_cols - bgt } else { 1 };
    if state.cursor_col < state.scroll_col {
        state.scroll_col = state.cursor_col;
    }
    if state.cursor_col >= state.scroll_col + fel {
        state.scroll_col = state.cursor_col - fel + 1;
    }
}

fn htq(state: &EditorState) {
    let cols = state.term_cols;
    let rows = state.term_rows;
    let edit_rows = state.edit_rows();
    let bgt = state.gutter_width();

    
    let title = if state.modified {
        format!(" TrustEdit — {}  [modified]", state.filename)
    } else {
        format!(" TrustEdit — {}", state.filename)
    };
    let header = nph(&title, cols);
    
    crate::framebuffer::fill_rect(0, 0, cols as u32 * 8, 16, 0xFF1A1A2E);
    crate::framebuffer::draw_text(&header, 0, 0, C_);

    
    for bor in 0..edit_rows {
        let eki = state.scroll_row + bor;
        let y = ((bor + 1) * 16) as u32;

        
        crate::framebuffer::fill_rect(0, y, cols as u32 * 8, 16, 0xFF0D0D1A);

        if eki < state.lines.len() {
            
            let axw = format!("{:>width$} ", eki + 1, width = bgt - 2);
            
            crate::framebuffer::fill_rect(0, y, bgt as u32 * 8, 16, 0xFF15152A);
            
            if eki == state.cursor_row {
                crate::framebuffer::fill_rect(bgt as u32 * 8, y, (cols - bgt) as u32 * 8, 16, 0xFF1A1A30);
            }
            crate::framebuffer::draw_text(&axw, 0, y, K_);

            
            let line = &state.lines[eki];
            let fel = cols - bgt;
            let edt: String = if state.scroll_col < line.len() {
                let end = core::cmp::min(line.len(), state.scroll_col + fel);
                String::from(&line[state.scroll_col..end])
            } else {
                String::new()
            };
            
            
            lkx(&edt, (bgt * 8) as u32, y, &state.filename);
        } else {
            
            crate::framebuffer::fill_rect(0, y, bgt as u32 * 8, 16, 0xFF15152A);
            crate::framebuffer::draw_text("~", 0, y, CF_);
        }
    }

    
    let status_y = ((edit_rows + 1) * 16) as u32;
    crate::framebuffer::fill_rect(0, status_y, cols as u32 * 8, 16, 0xFF2A2A4A);
    let left = format!(" Ln {}, Col {} | {} lines | {}", 
        state.cursor_row + 1, 
        state.cursor_col + 1,
        state.lines.len(),
        if state.modified { "MODIFIED" } else { "saved" }
    );
    let right = format!("{} ", ldv(&state.filename));
    let padding = if cols > left.len() + right.len() { cols - left.len() - right.len() } else { 0 };
    let ahd = format!("{}{:>pad$}{}", left, "", right, pad = padding);
    crate::framebuffer::draw_text(&ahd, 0, status_y, B_);

    
    let bnq = ((edit_rows + 2) * 16) as u32;
    crate::framebuffer::fill_rect(0, bnq, cols as u32 * 8, 16, 0xFF0D0D1A);
    let jue = crate::time::uptime_ms().saturating_sub(state.status_time);
    if jue < 8000 {
        crate::framebuffer::draw_text(&state.status_msg, 0, bnq, D_);
    } else {
        crate::framebuffer::draw_text(
            " Ctrl+S=Save  Ctrl+X=Exit  Ctrl+F=Find  Ctrl+G=Goto  Ctrl+K=Cut  Ctrl+U=Paste",
            0, bnq, K_,
        );
    }

    
    let lat = state.cursor_row - state.scroll_row;
    let las = state.cursor_col - state.scroll_col + bgt;
    let cx = (las * 8) as u32;
    let u = ((lat + 1) * 16) as u32;
    
    crate::framebuffer::fill_rect(cx, u, 8, 16, R_);
    
    if state.cursor_row < state.lines.len() {
        let line = &state.lines[state.cursor_row];
        if state.cursor_col < line.len() {
            let ch = &line[state.cursor_col..state.cursor_col + 1];
            crate::framebuffer::draw_text(ch, cx, u, 0xFF0D0D1A);
        }
    }
}


fn lkx(text: &str, x: u32, y: u32, filename: &str) {
    let ext = filename.rsplit('.').next().unwrap_or("");
    let msd = matches!(ext, "rs" | "c" | "h" | "py" | "js" | "ts" | "sh" | "toml" | "json" | "cfg" | "conf");
    
    if !msd || text.is_empty() {
        crate::framebuffer::draw_text(text, x, y, R_);
        return;
    }

    
    let clr = [
        "fn ", "let ", "mut ", "pub ", "use ", "mod ", "struct ", "enum ", "impl ",
        "trait ", "type ", "const ", "static ", "match ", "if ", "else ", "for ",
        "while ", "loop ", "return ", "break ", "continue ", "async ", "await ",
        "unsafe ", "extern ", "crate ", "self ", "super ", "where ",
        "def ", "class ", "import ", "from ", "try ", "except ", "with ",
        "function ", "var ", "const ", "export ", "import ",
    ];

    let jw = text.trim_start();
    
    
    if jw.starts_with("//") || jw.starts_with('#') {
        crate::framebuffer::draw_text(text, x, y, K_);
        return;
    }

    
    for li in &clr {
        if jw.starts_with(li) || jw.starts_with(li.trim_end()) {
            
            crate::framebuffer::draw_text(text, x, y, C_);
            return;
        }
    }

    
    if jw.contains('"') {
        crate::framebuffer::draw_text(text, x, y, D_);
        return;
    }

    
    if jw.contains("!(") || jw.contains("!{") {
        crate::framebuffer::draw_text(text, x, y, DM_);
        return;
    }

    
    crate::framebuffer::draw_text(text, x, y, R_);
}

fn ldv(filename: &str) -> &str {
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

fn nph(j: &str, width: usize) -> String {
    if j.len() >= width {
        String::from(&j[..width])
    } else {
        let mut out = String::from(j);
        for _ in 0..(width - j.len()) {
            out.push(' ');
        }
        out
    }
}

fn handle_key(state: &mut EditorState, key: u8) {
    match key {
        
        0x13 => save_file(state),
        
        
        0x18 => {
            if state.modified {
                state.set_status("File has unsaved changes! Ctrl+S to save, Ctrl+Q to quit without saving");
            } else {
                state.running = false;
            }
        }

        
        0x11 => {
            state.running = false;
        }
        
        
        0x07 => {
            if let Some(gfm) = iww(state, "Go to line: ") {
                if let Ok(line) = gfm.trim().parse::<usize>() {
                    if line > 0 && line <= state.lines.len() {
                        state.cursor_row = line - 1;
                        state.cursor_col = 0;
                        state.set_status(&format!("Jumped to line {}", line));
                    } else {
                        state.set_status("Invalid line number");
                    }
                }
            }
        }
        
        
        0x06 => {
            let default = state.search_query.clone();
            let nyu = if default.is_empty() {
                String::from("Search: ")
            } else {
                format!("Search [{}]: ", default)
            };
            if let Some(query) = iww(state, &nyu) {
                let q = if query.is_empty() { default } else { query };
                if !q.is_empty() {
                    state.search_query = q.clone();
                    find_next(state, &q);
                }
            }
        }
        
        
        0x0B => {
            if state.cursor_row < state.lines.len() {
                let ddj = state.lines.remove(state.cursor_row);
                state.push_undo(UndoEntry::DeleteLine {
                    row: state.cursor_row,
                    content: ddj.clone(),
                });
                state.clipboard = vec![ddj];
                if state.lines.is_empty() {
                    state.lines.push(String::new());
                }
                if state.cursor_row >= state.lines.len() {
                    state.cursor_row = state.lines.len() - 1;
                }
                state.cursor_col = 0;
                state.modified = true;
                state.set_status("Line cut");
            }
        }
        
        
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
        
        
        0x03 => {
            if state.cursor_row < state.lines.len() {
                state.clipboard = vec![state.lines[state.cursor_row].clone()];
                state.set_status("Line copied");
            }
        }
        
        
        0x1A => {
            undo(state);
        }
        
        
        0x0C => {
            crate::framebuffer::clear();
            state.set_status("Screen refreshed");
        }
        
        
        27 => {}
        
        
        crate::keyboard::T_ => {
            if state.cursor_row > 0 {
                state.cursor_row -= 1;
                clamp_cursor_col(state);
            }
        }
        crate::keyboard::S_ => {
            if state.cursor_row + 1 < state.lines.len() {
                state.cursor_row += 1;
                clamp_cursor_col(state);
            }
        }
        crate::keyboard::AI_ => {
            if state.cursor_col > 0 {
                state.cursor_col -= 1;
            } else if state.cursor_row > 0 {
                state.cursor_row -= 1;
                state.cursor_col = state.lines[state.cursor_row].len();
            }
        }
        crate::keyboard::AJ_ => {
            let wh = state.lines.get(state.cursor_row).map(|l| l.len()).unwrap_or(0);
            if state.cursor_col < wh {
                state.cursor_col += 1;
            } else if state.cursor_row + 1 < state.lines.len() {
                state.cursor_row += 1;
                state.cursor_col = 0;
            }
        }
        
        
        crate::keyboard::CW_ => {
            state.cursor_col = 0;
        }
        
        crate::keyboard::CV_ => {
            state.cursor_col = state.lines.get(state.cursor_row).map(|l| l.len()).unwrap_or(0);
        }
        
        
        crate::keyboard::AM_ => {
            let jump = state.edit_rows();
            state.cursor_row = state.cursor_row.saturating_sub(jump);
            clamp_cursor_col(state);
        }
        
        crate::keyboard::AO_ => {
            let jump = state.edit_rows();
            state.cursor_row = core::cmp::min(state.cursor_row + jump, state.lines.len().saturating_sub(1));
            clamp_cursor_col(state);
        }
        
        
        crate::keyboard::DE_ => {
            let wh = state.lines.get(state.cursor_row).map(|l| l.len()).unwrap_or(0);
            if state.cursor_col < wh {
                let ch = state.lines[state.cursor_row].remove(state.cursor_col);
                state.push_undo(UndoEntry::DeleteChar { row: state.cursor_row, col: state.cursor_col, ch });
                state.modified = true;
            } else if state.cursor_row + 1 < state.lines.len() {
                
                let next = state.lines.remove(state.cursor_row + 1);
                state.push_undo(UndoEntry::JoinLines { row: state.cursor_row, col: state.cursor_col });
                state.lines[state.cursor_row].push_str(&next);
                state.modified = true;
            }
        }
        
        
        0x08 => {
            if state.cursor_col > 0 {
                state.cursor_col -= 1;
                let ch = state.lines[state.cursor_row].remove(state.cursor_col);
                state.push_undo(UndoEntry::DeleteChar { row: state.cursor_row, col: state.cursor_col, ch });
                state.modified = true;
            } else if state.cursor_row > 0 {
                
                let current = state.lines.remove(state.cursor_row);
                state.cursor_row -= 1;
                state.cursor_col = state.lines[state.cursor_row].len();
                state.push_undo(UndoEntry::JoinLines { row: state.cursor_row, col: state.cursor_col });
                state.lines[state.cursor_row].push_str(&current);
                state.modified = true;
            }
        }
        
        
        0x0A => {
            let current_line = &state.lines[state.cursor_row];
            let bix = String::from(&current_line[state.cursor_col..]);
            state.lines[state.cursor_row] = String::from(&current_line[..state.cursor_col]);
            state.push_undo(UndoEntry::SplitLine { row: state.cursor_row, col: state.cursor_col });
            state.cursor_row += 1;
            state.lines.insert(state.cursor_row, bix);
            state.cursor_col = 0;
            state.modified = true;
        }
        
        
        0x09 => {
            for _ in 0..4 {
                state.lines[state.cursor_row].insert(state.cursor_col, ' ');
                state.push_undo(UndoEntry::InsertChar { row: state.cursor_row, col: state.cursor_col });
                state.cursor_col += 1;
            }
            state.modified = true;
        }
        
        
        ch if ch >= 32 && ch < 127 => {
            state.lines[state.cursor_row].insert(state.cursor_col, ch as char);
            state.push_undo(UndoEntry::InsertChar { row: state.cursor_row, col: state.cursor_col });
            state.cursor_col += 1;
            state.modified = true;
        }
        
        _ => {}
    }
}

fn clamp_cursor_col(state: &mut EditorState) {
    let wh = state.lines.get(state.cursor_row).map(|l| l.len()).unwrap_or(0);
    if state.cursor_col > wh {
        state.cursor_col = wh;
    }
}

fn undo(state: &mut EditorState) {
    if let Some(entry) = state.undo_stack.pop() {
        match entry {
            UndoEntry::InsertChar { row, col } => {
                if row < state.lines.len() && col < state.lines[row].len() {
                    state.lines[row].remove(col);
                    state.cursor_row = row;
                    state.cursor_col = col;
                }
            }
            UndoEntry::DeleteChar { row, col, ch } => {
                if row < state.lines.len() {
                    state.lines[row].insert(col, ch);
                    state.cursor_row = row;
                    state.cursor_col = col + 1;
                }
            }
            UndoEntry::InsertLine { row } => {
                if row < state.lines.len() {
                    state.lines.remove(row);
                    if state.lines.is_empty() {
                        state.lines.push(String::new());
                    }
                    state.cursor_row = row.min(state.lines.len() - 1);
                    state.cursor_col = 0;
                }
            }
            UndoEntry::DeleteLine { row, content } => {
                state.lines.insert(row, content);
                state.cursor_row = row;
                state.cursor_col = 0;
            }
            UndoEntry::SplitLine { row, col } => {
                if row + 1 < state.lines.len() {
                    let next = state.lines.remove(row + 1);
                    state.lines[row].push_str(&next);
                    state.cursor_row = row;
                    state.cursor_col = col;
                }
            }
            UndoEntry::JoinLines { row, col } => {
                if row < state.lines.len() {
                    let bix = String::from(&state.lines[row][col..]);
                    state.lines[row] = String::from(&state.lines[row][..col]);
                    state.lines.insert(row + 1, bix);
                    state.cursor_row = row + 1;
                    state.cursor_col = 0;
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
    
    let bpd = state.cursor_row;
    let afu = state.cursor_col + 1;
    
    for row in bpd..state.lines.len() {
        let fnt = if row == bpd { afu } else { 0 };
        if fnt < state.lines[row].len() {
            if let Some(pos) = state.lines[row][fnt..].find(query) {
                state.cursor_row = row;
                state.cursor_col = fnt + pos;
                state.set_status(&format!("Found \"{}\" at line {}", query, row + 1));
                return;
            }
        }
    }
    
    
    for row in 0..=bpd {
        let kvf = if row == bpd { afu } else { state.lines[row].len() };
        if let Some(pos) = state.lines[row][..kvf.min(state.lines[row].len())].find(query) {
            state.cursor_row = row;
            state.cursor_col = pos;
            state.set_status(&format!("Found \"{}\" at line {} (wrapped)", query, row + 1));
            return;
        }
    }
    
    state.set_status(&format!("\"{}\" not found", query));
}


fn iww(state: &mut EditorState, nh: &str) -> Option<String> {
    let cols = state.term_cols;
    let bnq = ((state.edit_rows() + 2) * 16) as u32;
    
    let mut input = String::new();
    
    loop {
        
        crate::framebuffer::fill_rect(0, bnq, cols as u32 * 8, 16, 0xFF1A1A2E);
        let display = format!("{}{}", nh, input);
        crate::framebuffer::draw_text(&display, 0, bnq, D_);
        
        let cx = ((nh.len() + input.len()) * 8) as u32;
        crate::framebuffer::fill_rect(cx, bnq, 8, 16, R_);
        
        if let Some(key) = crate::keyboard::ya() {
            match key {
                0x0A => return Some(input), 
                27 => return None,           
                0x08 => { input.pop(); }     
                ch if ch >= 32 && ch < 127 => input.push(ch as char),
                _ => {}
            }
        } else {
            core::hint::spin_loop();
        }
    }
}
