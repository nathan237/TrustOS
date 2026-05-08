












use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;






#[derive(Clone)]
struct Nq {
    lines: Vec<String>,
    cursor_line: usize,
    cursor_col: usize,
}


#[derive(Clone)]
pub struct EditorState {
    
    pub lines: Vec<String>,
    
    pub cursor_line: usize,
    
    pub cursor_col: usize,
    
    pub scroll_y: usize,
    
    pub scroll_x: usize,
    
    pub file_path: Option<String>,
    
    pub dirty: bool,
    
    pub language: Language,
    
    pub status_message: Option<String>,
    
    pub blink_counter: u32,
    
    undo_stack: Vec<Nq>,
    
    redo_stack: Vec<Nq>,
    
    pub selection_anchor: Option<(usize, usize)>,
    
    pub find_query: Option<String>,
    
    pub replace_text: Option<String>,
    
    pub find_replace_mode: bool,
    
    pub find_matches: Vec<(usize, usize)>,
    
    pub find_match_idx: usize,
    
    pub goto_line_input: Option<String>,
    
    pub matching_bracket: Option<(usize, usize)>,
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
    
    
    pub fn lze(name: &str) -> Self {
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
            goto_line_input: None,
            matching_bracket: None,
        }
    }
    
    
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
    
    
    pub fn load_file(&mut self, path: &str) {
        self.file_path = Some(String::from(path));
        self.language = Language::lze(path);
        
        let kg = if path.starts_with('/') {
            String::from(path)
        } else {
            format!("/{}", path)
        };
        
        if let Ok(data) = crate::ramfs::bh(|fs| fs.read_file(&kg).map(|d| d.to_vec())) {
            if let Ok(text) = core::str::from_utf8(&data) {
                self.load_text(text);
            } else {
                self.lines = alloc::vec![String::from("(binary file — cannot edit)")];
            }
        } else {
            
            self.lines = alloc::vec![String::new()];
        }
        self.dirty = false;
    }
    
    
    pub fn save(&mut self) -> bool {
        if let Some(ref path) = self.file_path {
            let kg = if path.starts_with('/') {
                String::from(path.as_str())
            } else {
                format!("/{}", path)
            };
            
            
            let mut text = String::new();
            for (i, line) in self.lines.iter().enumerate() {
                text.push_str(line);
                if i + 1 < self.lines.len() {
                    text.push('\n');
                }
            }
            
            let result = crate::ramfs::bh(|fs| {
                
                if fs.read_file(&kg).is_err() {
                    let _ = fs.touch(&kg);
                }
                fs.write_file(&kg, text.as_bytes())
            });
            
            if result.is_ok() {
                self.dirty = false;
                self.status_message = Some(format!("Saved: {}", path));
                crate::serial_println!("[TrustCode] Saved: {}", path);
                
                if let Ok(()) = crate::vfs::write_file(&kg, text.as_bytes()) {
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
    
    
    fn current_line(&self) -> &str {
        if self.cursor_line < self.lines.len() {
            &self.lines[self.cursor_line]
        } else {
            ""
        }
    }
    
    
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

    
    fn save_undo_state(&mut self) {
        self.undo_stack.push(Nq {
            lines: self.lines.clone(),
            cursor_line: self.cursor_line,
            cursor_col: self.cursor_col,
        });
        
        if self.undo_stack.len() > 200 {
            self.undo_stack.remove(0);
        }
        
        self.redo_stack.clear();
    }

    
    fn undo(&mut self) {
        if let Some(snapshot) = self.undo_stack.pop() {
            
            self.redo_stack.push(Nq {
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

    
    fn redo(&mut self) {
        if let Some(snapshot) = self.redo_stack.pop() {
            
            self.undo_stack.push(Nq {
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

    
    
    pub fn get_selection_range(&self) -> Option<(usize, usize, usize, usize)> {
        let (al, ac) = self.selection_anchor?;
        let (bl, bc) = (self.cursor_line, self.cursor_col);
        if (al, ac) <= (bl, bc) {
            Some((al, ac, bl, bc))
        } else {
            Some((bl, bc, al, ac))
        }
    }

    
    fn selected_text(&self) -> Option<String> {
        let (sl, dr, el, ec) = self.get_selection_range()?;
        let mut result = String::new();
        for l in sl..=el {
            if l >= self.lines.len() { break; }
            let line = &self.lines[l];
            let start = if l == sl { dr.min(line.len()) } else { 0 };
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

    
    fn delete_selection(&mut self) {
        if let Some((sl, dr, el, ec)) = self.get_selection_range() {
            self.save_undo_state();
            if sl == el {
                
                if sl < self.lines.len() {
                    let end = ec.min(self.lines[sl].len());
                    let start = dr.min(end);
                    self.lines[sl] = format!("{}{}", &self.lines[sl][..start], &self.lines[sl][end..]);
                }
            } else {
                
                if el < self.lines.len() {
                    let beo = if ec <= self.lines[el].len() {
                        String::from(&self.lines[el][ec..])
                    } else {
                        String::new()
                    };
                    
                    let nm = if dr <= self.lines[sl].len() {
                        String::from(&self.lines[sl][..dr])
                    } else {
                        self.lines[sl].clone()
                    };
                    self.lines[sl] = format!("{}{}", nm, beo);
                    
                    let oex = el - sl;
                    for _ in 0..oex {
                        if sl + 1 < self.lines.len() {
                            self.lines.remove(sl + 1);
                        }
                    }
                }
            }
            self.cursor_line = sl;
            self.cursor_col = dr;
            self.selection_anchor = None;
            self.dirty = true;
        }
    }

    
    fn update_find_matches(&mut self) {
        self.find_matches.clear();
        if let Some(ref query) = self.find_query {
            if query.is_empty() { return; }
            let q = query.clone();
            for (xf, line) in self.lines.iter().enumerate() {
                let mut start = 0;
                while start + q.len() <= line.len() {
                    if &line[start..start + q.len()] == q.as_str() {
                        self.find_matches.push((xf, start));
                        start += q.len().max(1);
                    } else {
                        start += 1;
                    }
                }
            }
        }
    }

    
    fn find_next(&mut self) {
        if self.find_matches.is_empty() { return; }
        self.find_match_idx = (self.find_match_idx + 1) % self.find_matches.len();
        let (line, col) = self.find_matches[self.find_match_idx];
        self.cursor_line = line;
        self.cursor_col = col;
        self.ensure_cursor_visible();
    }

    
    fn qfu(&mut self) {
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

    
    fn replace_current(&mut self) {
        if self.find_matches.is_empty() { return; }
        let idx = self.find_match_idx.min(self.find_matches.len() - 1);
        let (line, col) = self.find_matches[idx];
        let query = self.find_query.clone();
        let gre = self.replace_text.clone();
        if let (Some(q), Some(rep)) = (query, gre) {
            if line < self.lines.len() && col + q.len() <= self.lines[line].len() {
                self.save_undo_state();
                let exi = q.len();
                self.lines[line] = format!("{}{}{}", &self.lines[line][..col], rep, &self.lines[line][col + exi..]);
                self.dirty = true;
                self.update_find_matches();
                self.cursor_line = line;
                self.cursor_col = col + rep.len();
            }
        }
    }

    
    fn replace_all(&mut self) {
        if self.find_matches.is_empty() { return; }
        let query = self.find_query.clone();
        let gre = self.replace_text.clone();
        if let (Some(q), Some(rep)) = (query, gre) {
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
    
    
    pub fn ikd(&self) -> usize {
        self.lines.len()
    }
    
    
    
    
    
    
    pub fn handle_key(&mut self, key: u8) -> bool {
        use crate::keyboard::*;
        
        self.blink_counter = 0; 
        
        
        if self.status_message.is_some() && key != 0x13 { 
            
        }
        
        
        if self.find_query.is_some() {
            match key {
                0x1B => { 
                    self.find_query = None;
                    self.replace_text = None;
                    self.find_replace_mode = false;
                    self.find_matches.clear();
                    return true;
                }
                0x0D | 0x0A => { 
                    if self.find_replace_mode {
                        if let Some(ref _rt) = self.replace_text {
                            self.replace_current();
                        }
                    } else {
                        self.find_next();
                    }
                    return true;
                }
                0x09 => { 
                    if self.replace_text.is_some() {
                        self.find_replace_mode = !self.find_replace_mode;
                    }
                    return true;
                }
                0x08 => { 
                    if self.find_replace_mode {
                        if let Some(ref mut bdm) = self.replace_text {
                            bdm.pop();
                        }
                    } else if let Some(ref mut q) = self.find_query {
                        q.pop();
                    }
                    self.update_find_matches();
                    return true;
                }
                0x01 => { 
                    self.replace_all();
                    return true;
                }
                c if c >= 0x20 && c < 0x7F => {
                    if self.find_replace_mode {
                        if let Some(ref mut bdm) = self.replace_text {
                            bdm.push(c as char);
                        }
                    } else if let Some(ref mut q) = self.find_query {
                        q.push(c as char);
                    }
                    self.update_find_matches();
                    
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
        
        
        if self.goto_line_input.is_some() {
            match key {
                0x1B => { 
                    self.goto_line_input = None;
                    return true;
                }
                0x0D | 0x0A => { 
                    if let Some(ref input) = self.goto_line_input {
                        if let Ok(axw) = input.parse::<usize>() {
                            if axw > 0 && axw <= self.lines.len() {
                                self.cursor_line = axw - 1;
                                self.cursor_col = 0;
                                self.ensure_cursor_visible();
                                self.status_message = Some(format!("Go to line {}", axw));
                            } else {
                                self.status_message = Some(format!("Invalid line (1-{})", self.lines.len()));
                            }
                        }
                    }
                    self.goto_line_input = None;
                    return true;
                }
                0x08 => { 
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
        
        let akm = crate::keyboard::sx(0x2A) || crate::keyboard::sx(0x36);
        
        match key {
            
            0x13 => {
                self.save();
                return true;
            }
            
            
            0x06 => {
                self.find_query = Some(String::new());
                self.replace_text = None;
                self.find_replace_mode = false;
                self.find_matches.clear();
                self.find_match_idx = 0;
                return true;
            }
            
            
            0x12 => {
                self.find_query = Some(String::new());
                self.replace_text = Some(String::new());
                self.find_replace_mode = false;
                self.find_matches.clear();
                self.find_match_idx = 0;
                return true;
            }
            
            
            0x1A => {
                self.undo();
                self.ensure_cursor_visible();
                return true;
            }
            
            
            0x19 => {
                self.redo();
                self.ensure_cursor_visible();
                return true;
            }
            
            
            0x07 => {
                self.goto_line_input = Some(String::new());
                return true;
            }
            
            
            BAB_ => {
                self.toggle_comment();
                return true;
            }
            
            
            BAA_ => {
                self.delete_current_line();
                return true;
            }
            
            
            AZZ_ => {
                self.duplicate_line();
                return true;
            }
            
            
            AZW_ => {
                self.move_line_up();
                return true;
            }
            
            
            AZV_ => {
                self.move_line_down();
                return true;
            }
            
            
            AZX_ => {
                if akm && self.selection_anchor.is_none() {
                    self.selection_anchor = Some((self.cursor_line, self.cursor_col));
                } else if !akm {
                    self.selection_anchor = None;
                }
                self.word_left();
                self.ensure_cursor_visible();
                return true;
            }
            
            
            AZY_ => {
                if akm && self.selection_anchor.is_none() {
                    self.selection_anchor = Some((self.cursor_line, self.cursor_col));
                } else if !akm {
                    self.selection_anchor = None;
                }
                self.word_right();
                self.ensure_cursor_visible();
                return true;
            }
            
            
            0x01 => {
                self.selection_anchor = Some((0, 0));
                let dah = self.lines.len().saturating_sub(1);
                let mwk = if dah < self.lines.len() { self.lines[dah].len() } else { 0 };
                self.cursor_line = dah;
                self.cursor_col = mwk;
                self.status_message = Some(String::from("Select All"));
                return true;
            }
            
            
            0x03 => {
                if let Some(text) = self.selected_text() {
                    crate::keyboard::byb(&text);
                    self.status_message = Some(String::from("Copied"));
                }
                return true;
            }
            
            
            0x18 => {
                if let Some(text) = self.selected_text() {
                    crate::keyboard::byb(&text);
                    self.delete_selection();
                    self.status_message = Some(String::from("Cut"));
                }
                return true;
            }
            
            
            0x16 => {
                if let Some(text) = crate::keyboard::hln() {
                    
                    if self.selection_anchor.is_some() {
                        self.delete_selection();
                    }
                    self.save_undo_state();
                    
                    for ch in text.chars() {
                        if ch == '\n' {
                            
                            if self.cursor_line < self.lines.len() {
                                self.cursor_col = self.cursor_col.min(self.lines[self.cursor_line].len());
                                let ef = self.lines[self.cursor_line].split_off(self.cursor_col);
                                self.cursor_line += 1;
                                self.lines.insert(self.cursor_line, ef);
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
            
            
            T_ => {
                if akm && self.selection_anchor.is_none() {
                    self.selection_anchor = Some((self.cursor_line, self.cursor_col));
                } else if !akm {
                    self.selection_anchor = None;
                }
                if self.cursor_line > 0 {
                    self.cursor_line -= 1;
                    self.clamp_cursor_col();
                }
                self.ensure_cursor_visible();
                return true;
            }
            S_ => {
                if akm && self.selection_anchor.is_none() {
                    self.selection_anchor = Some((self.cursor_line, self.cursor_col));
                } else if !akm {
                    self.selection_anchor = None;
                }
                if self.cursor_line + 1 < self.lines.len() {
                    self.cursor_line += 1;
                    self.clamp_cursor_col();
                }
                self.ensure_cursor_visible();
                return true;
            }
            AI_ => {
                if akm && self.selection_anchor.is_none() {
                    self.selection_anchor = Some((self.cursor_line, self.cursor_col));
                } else if !akm {
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
            AJ_ => {
                if akm && self.selection_anchor.is_none() {
                    self.selection_anchor = Some((self.cursor_line, self.cursor_col));
                } else if !akm {
                    self.selection_anchor = None;
                }
                let wh = if self.cursor_line < self.lines.len() { self.lines[self.cursor_line].len() } else { 0 };
                if self.cursor_col < wh {
                    self.cursor_col += 1;
                } else if self.cursor_line + 1 < self.lines.len() {
                    self.cursor_line += 1;
                    self.cursor_col = 0;
                }
                self.ensure_cursor_visible();
                return true;
            }
            CW_ => {
                if akm && self.selection_anchor.is_none() {
                    self.selection_anchor = Some((self.cursor_line, self.cursor_col));
                } else if !akm {
                    self.selection_anchor = None;
                }
                self.cursor_col = 0;
                return true;
            }
            CV_ => {
                if akm && self.selection_anchor.is_none() {
                    self.selection_anchor = Some((self.cursor_line, self.cursor_col));
                } else if !akm {
                    self.selection_anchor = None;
                }
                let wh = if self.cursor_line < self.lines.len() { self.lines[self.cursor_line].len() } else { 0 };
                self.cursor_col = wh;
                return true;
            }
            AM_ => {
                let jump = 20;
                self.cursor_line = self.cursor_line.saturating_sub(jump);
                self.clamp_cursor_col();
                self.ensure_cursor_visible();
                return true;
            }
            AO_ => {
                let jump = 20;
                self.cursor_line = (self.cursor_line + jump).min(self.lines.len().saturating_sub(1));
                self.clamp_cursor_col();
                self.ensure_cursor_visible();
                return true;
            }
            
            
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
                    
                    let current = self.lines.remove(self.cursor_line);
                    self.cursor_line -= 1;
                    self.cursor_col = self.lines[self.cursor_line].len();
                    self.lines[self.cursor_line].push_str(&current);
                    self.dirty = true;
                }
                self.ensure_cursor_visible();
                return true;
            }
            
            
            DE_ => {
                if self.selection_anchor.is_some() {
                    self.delete_selection();
                    return true;
                }
                self.save_undo_state();
                if self.cursor_line < self.lines.len() {
                    let wh = self.lines[self.cursor_line].len();
                    if self.cursor_col < wh {
                        self.lines[self.cursor_line].remove(self.cursor_col);
                        self.dirty = true;
                    } else if self.cursor_line + 1 < self.lines.len() {
                        
                        let next = self.lines.remove(self.cursor_line + 1);
                        self.lines[self.cursor_line].push_str(&next);
                        self.dirty = true;
                    }
                }
                return true;
            }
            
            
            0x0D | 0x0A => {
                if self.selection_anchor.is_some() {
                    self.delete_selection();
                }
                self.save_undo_state();
                if self.cursor_line < self.lines.len() {
                    
                    self.cursor_col = self.cursor_col.min(self.lines[self.cursor_line].len());
                    
                    let ef = self.lines[self.cursor_line].split_off(self.cursor_col);
                    
                    
                    let axq: String = self.lines[self.cursor_line]
                        .chars()
                        .take_while(|c| *c == ' ' || *c == '\t')
                        .collect();
                    
                    
                    let ua = if self.lines[self.cursor_line].trim_end().ends_with('{')
                        || self.lines[self.cursor_line].trim_end().ends_with('(') {
                        "    "
                    } else {
                        ""
                    };
                    
                    let nje = format!("{}{}{}", axq, ua, ef);
                    self.cursor_line += 1;
                    self.lines.insert(self.cursor_line, nje);
                    self.cursor_col = axq.len() + ua.len();
                    self.dirty = true;
                }
                self.ensure_cursor_visible();
                return true;
            }
            
            
            0x09 => {
                self.save_undo_state();
                if let Some((sl, _sc, el, _ec)) = self.get_selection_range() {
                    
                    if akm {
                        
                        for l in sl..=el.min(self.lines.len().saturating_sub(1)) {
                            let cdt = self.lines[l].chars().take(4).take_while(|c| *c == ' ').count();
                            if cdt > 0 {
                                self.lines[l] = String::from(&self.lines[l][cdt..]);
                            }
                        }
                    } else {
                        
                        for l in sl..=el.min(self.lines.len().saturating_sub(1)) {
                            self.lines[l] = format!("    {}", self.lines[l]);
                        }
                    }
                    self.dirty = true;
                } else if akm {
                    
                    if self.cursor_line < self.lines.len() {
                        let cdt = self.lines[self.cursor_line].chars().take(4).take_while(|c| *c == ' ').count();
                        if cdt > 0 {
                            self.lines[self.cursor_line] = String::from(&self.lines[self.cursor_line][cdt..]);
                            self.cursor_col = self.cursor_col.saturating_sub(cdt);
                            self.dirty = true;
                        }
                    }
                } else {
                    
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
                            
                        }
                    }
                }
                return true;
            }
            
            _ => {}
        }
        false
    }
    
    
    fn ensure_cursor_visible(&mut self) {
        if self.cursor_line < self.scroll_y {
            self.scroll_y = self.cursor_line;
        }
        
        
        let visible = 30usize;
        if self.cursor_line >= self.scroll_y + visible {
            self.scroll_y = self.cursor_line - visible + 1;
        }
    }

    
    
    

    
    fn toggle_comment(&mut self) {
        self.save_undo_state();
        let cvk = match self.language {
            Language::Rust | Language::C | Language::JavaScript => "// ",
            Language::Python => "# ",
            Language::Toml => "# ",
            _ => "// ",
        };

        
        let (start, end) = if let Some((sl, _sc, el, _ec)) = self.get_selection_range() {
            (sl, el)
        } else {
            (self.cursor_line, self.cursor_line)
        };

        
        let heo = (start..=end.min(self.lines.len().saturating_sub(1)))
            .all(|l| self.lines[l].trim_start().starts_with(cvk.trim_end()));

        for l in start..=end.min(self.lines.len().saturating_sub(1)) {
            if heo {
                
                let jw = &self.lines[l];
                if let Some(pos) = jw.find(cvk) {
                    self.lines[l] = format!("{}{}", &jw[..pos], &jw[pos + cvk.len()..]);
                } else if let Some(pos) = jw.find(cvk.trim_end()) {
                    
                    let nwt = cvk.trim_end();
                    self.lines[l] = format!("{}{}", &jw[..pos], &jw[pos + nwt.len()..]);
                }
            } else {
                
                let igi = self.lines[l].chars().take_while(|c| *c == ' ' || *c == '\t').count();
                self.lines[l] = format!("{}{}{}", &self.lines[l][..igi], cvk, &self.lines[l][igi..]);
            }
        }
        self.dirty = true;
        self.status_message = Some(String::from(if heo { "Uncommented" } else { "Commented" }));
    }

    
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

    
    fn duplicate_line(&mut self) {
        if self.cursor_line < self.lines.len() {
            self.save_undo_state();
            let dnx = self.lines[self.cursor_line].clone();
            self.lines.insert(self.cursor_line + 1, dnx);
            self.cursor_line += 1;
            self.dirty = true;
            self.ensure_cursor_visible();
        }
    }

    
    fn move_line_up(&mut self) {
        if self.cursor_line > 0 && self.cursor_line < self.lines.len() {
            self.save_undo_state();
            self.lines.swap(self.cursor_line, self.cursor_line - 1);
            self.cursor_line -= 1;
            self.dirty = true;
            self.ensure_cursor_visible();
        }
    }

    
    fn move_line_down(&mut self) {
        if self.cursor_line + 1 < self.lines.len() {
            self.save_undo_state();
            self.lines.swap(self.cursor_line, self.cursor_line + 1);
            self.cursor_line += 1;
            self.dirty = true;
            self.ensure_cursor_visible();
        }
    }

    
    fn word_left(&mut self) {
        if self.cursor_line >= self.lines.len() { return; }
        if self.cursor_col == 0 {
            
            if self.cursor_line > 0 {
                self.cursor_line -= 1;
                self.cursor_col = self.lines[self.cursor_line].len();
            }
            return;
        }
        let line = &self.lines[self.cursor_line];
        let bytes = line.as_bytes();
        let mut pos = self.cursor_col.min(bytes.len());
        
        while pos > 0 && bytes[pos - 1] == b' ' {
            pos -= 1;
        }
        
        while pos > 0 && bytes[pos - 1] != b' ' {
            pos -= 1;
        }
        self.cursor_col = pos;
    }

    
    fn word_right(&mut self) {
        if self.cursor_line >= self.lines.len() { return; }
        let line = &self.lines[self.cursor_line];
        let bytes = line.as_bytes();
        let len = bytes.len();
        if self.cursor_col >= len {
            
            if self.cursor_line + 1 < self.lines.len() {
                self.cursor_line += 1;
                self.cursor_col = 0;
            }
            return;
        }
        let mut pos = self.cursor_col;
        
        while pos < len && bytes[pos] != b' ' {
            pos += 1;
        }
        
        while pos < len && bytes[pos] == b' ' {
            pos += 1;
        }
        self.cursor_col = pos;
    }

    
    pub fn update_matching_bracket(&mut self) {
        self.matching_bracket = None;
        if self.cursor_line >= self.lines.len() { return; }
        let line = &self.lines[self.cursor_line];
        if self.cursor_col >= line.len() { return; }
        
        let ch = line.as_bytes()[self.cursor_col];
        let (target, forward) = match ch {
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
            loop {
                let bytes = self.lines[l].as_bytes();
                while c >= 0 {
                    let cvz = c as usize;
                    if cvz < bytes.len() {
                        if bytes[cvz] == ch { depth += 1; }
                        else if bytes[cvz] == target {
                            depth -= 1;
                            if depth == 0 {
                                self.matching_bracket = Some((l, cvz));
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






#[derive(Clone, Copy, PartialEq)]
pub enum TokenKind {
    Normal,
    Keyword,
    Type,
    String,
    Comment,
    Number,
    Aq,
    Macro,
    Attribute,
    Lifetime,
    Operator,
    Bracket,
}


pub struct O {
    pub start: usize,
    pub end: usize,
    pub kind: TokenKind,
}


pub const BPU_: u32   = 0xFF569CD6; 
pub const BQG_: u32      = 0xFF4EC9B0; 
pub const BQE_: u32    = 0xFFCE9178; 
pub const BPS_: u32   = 0xFF6A9955; 
pub const BQB_: u32    = 0xFFB5CEA8; 
pub const BPT_: u32  = 0xFFDCDCAA; 
pub const BPX_: u32     = 0xFF4FC1FF; 
pub const BPO_: u32 = 0xFFD7BA7D; 
pub const BPV_: u32  = 0xFF569CD6; 
pub const BQC_: u32  = 0xFFD4D4D4; 
pub const BPP_: u32   = 0xFFFFD700; 
pub const KH_: u32    = 0xFFD4D4D4; 
pub const AQH_: u32  = 0xFF858585; 
pub const BPN_: u32 = 0xFF858585; 
pub const ND_: u32        = 0xFF1E1E2E; 
pub const AQE_: u32 = 0xFF1E1E2E; 
pub const AQB_: u32 = 0xFF2A2D3A; 
pub const ABU_: u32 = 0xFF007ACC; 
pub const GS_: u32 = 0xFFFFFFFF; 
pub const AQC_: u32    = 0xFFAEAFAD; 
pub const BPQ_: u32 = 0xFF252526; 
pub const BQF_: u32 = 0xFF1E1E2E; 
pub const DJK_: u32 = 0xFF2D2D2D; 


const CTH_: &[&str] = &[
    "as", "async", "await", "break", "const", "continue", "crate", "dyn",
    "else", "enum", "extern", "false", "fn", "for", "if", "impl", "in",
    "let", "loop", "match", "mod", "move", "mut", "pub", "ref", "return",
    "self", "Self", "static", "struct", "super", "trait", "true", "type",
    "unsafe", "use", "where", "while", "yield",
];


const CTI_: &[&str] = &[
    "bool", "char", "f32", "f64", "i8", "i16", "i32", "i64", "i128", "isize",
    "u8", "u16", "u32", "u64", "u128", "usize", "str", "String", "Vec",
    "Option", "Result", "Box", "Rc", "Arc", "Cell", "RefCell", "Mutex",
    "HashMap", "HashSet", "BTreeMap", "BTreeSet", "Cow", "Pin",
    "Some", "None", "Ok", "Err",
];


pub fn jnh(line: &str) -> Vec<O> {
    let mut jl = Vec::new();
    let bytes = line.as_bytes();
    let len = bytes.len();
    let mut i = 0;
    
    while i < len {
        let ch = bytes[i];
        
        
        if ch == b'/' && i + 1 < len && bytes[i + 1] == b'/' {
            jl.push(O { start: i, end: len, kind: TokenKind::Comment });
            break; 
        }
        
        
        if ch == b'#' && i + 1 < len && bytes[i + 1] == b'[' {
            let start = i;
            
            while i < len && bytes[i] != b']' { i += 1; }
            if i < len { i += 1; } 
            jl.push(O { start, end: i, kind: TokenKind::Attribute });
            continue;
        }
        
        
        if ch == b'"' {
            let start = i;
            i += 1;
            while i < len {
                if bytes[i] == b'\\' && i + 1 < len {
                    i += 2; 
                } else if bytes[i] == b'"' {
                    i += 1;
                    break;
                } else {
                    i += 1;
                }
            }
            jl.push(O { start, end: i, kind: TokenKind::String });
            continue;
        }
        
        
        if ch == b'\'' {
            
            let start = i;
            i += 1;
            if i < len && bytes[i].is_ascii_alphabetic() {
                
                let rcy = i;
                while i < len && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                    i += 1;
                }
                if i < len && bytes[i] == b'\'' {
                    
                    i += 1;
                    jl.push(O { start, end: i, kind: TokenKind::String });
                } else {
                    
                    jl.push(O { start, end: i, kind: TokenKind::Lifetime });
                }
            } else if i < len && bytes[i] == b'\\' {
                
                while i < len && bytes[i] != b'\'' { i += 1; }
                if i < len { i += 1; }
                jl.push(O { start, end: i, kind: TokenKind::String });
            } else {
                jl.push(O { start, end: start + 1, kind: TokenKind::Normal });
            }
            continue;
        }
        
        
        if ch.is_ascii_digit() || (ch == b'0' && i + 1 < len && (bytes[i+1] == b'x' || bytes[i+1] == b'b' || bytes[i+1] == b'o')) {
            let start = i;
            i += 1;
            while i < len && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_' || bytes[i] == b'.') {
                i += 1;
            }
            jl.push(O { start, end: i, kind: TokenKind::Number });
            continue;
        }
        
        
        if ch.is_ascii_alphabetic() || ch == b'_' {
            let start = i;
            while i < len && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                i += 1;
            }
            let fx = &line[start..i];
            
            
            if i < len && bytes[i] == b'!' {
                jl.push(O { start, end: i + 1, kind: TokenKind::Macro });
                i += 1;
                continue;
            }
            
            
            let gdw = i < len && bytes[i] == b'(';
            
            let kind = if CTH_.contains(&fx) {
                TokenKind::Keyword
            } else if CTI_.contains(&fx) {
                TokenKind::Type
            } else if gdw {
                TokenKind::Aq
            } else {
                TokenKind::Normal
            };
            
            jl.push(O { start, end: i, kind });
            continue;
        }
        
        
        if ch == b'{' || ch == b'}' || ch == b'(' || ch == b')' || ch == b'[' || ch == b']' {
            jl.push(O { start: i, end: i + 1, kind: TokenKind::Bracket });
            i += 1;
            continue;
        }
        
        
        if ch == b'+' || ch == b'-' || ch == b'*' || ch == b'=' || ch == b'!' 
            || ch == b'<' || ch == b'>' || ch == b'&' || ch == b'|' || ch == b'^'
            || ch == b':' || ch == b';' || ch == b',' || ch == b'.' {
            jl.push(O { start: i, end: i + 1, kind: TokenKind::Operator });
            i += 1;
            continue;
        }
        
        
        let start = i;
        i += 1;
        jl.push(O { start, end: i, kind: TokenKind::Normal });
    }
    
    jl
}


pub fn jnf(kind: TokenKind) -> u32 {
    match kind {
        TokenKind::Normal => KH_,
        TokenKind::Keyword => BPU_,
        TokenKind::Type => BQG_,
        TokenKind::String => BQE_,
        TokenKind::Comment => BPS_,
        TokenKind::Number => BQB_,
        TokenKind::Aq => BPT_,
        TokenKind::Macro => BPX_,
        TokenKind::Attribute => BPO_,
        TokenKind::Lifetime => BPV_,
        TokenKind::Operator => BQC_,
        TokenKind::Bracket => BPP_,
    }
}





const CQC_: &[&str] = &[
    "False", "None", "True", "and", "as", "assert", "async", "await",
    "break", "class", "continue", "def", "del", "elif", "else", "except",
    "finally", "for", "from", "global", "if", "import", "in", "is",
    "lambda", "nonlocal", "not", "or", "pass", "raise", "return", "try",
    "while", "with", "yield",
];

const CQB_: &[&str] = &[
    "int", "float", "str", "bool", "list", "dict", "tuple", "set",
    "frozenset", "bytes", "bytearray", "range", "type", "object",
    "print", "len", "input", "open", "super", "self", "cls",
    "Exception", "ValueError", "TypeError", "KeyError", "IndexError",
    "RuntimeError", "StopIteration", "OSError", "IOError",
];

pub fn pkz(line: &str) -> Vec<O> {
    let mut jl = Vec::new();
    let bytes = line.as_bytes();
    let len = bytes.len();
    let mut i = 0;
    
    while i < len {
        let ch = bytes[i];
        
        
        if ch == b'#' {
            jl.push(O { start: i, end: len, kind: TokenKind::Comment });
            break;
        }
        
        
        if ch == b'@' {
            let start = i;
            i += 1;
            while i < len && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_' || bytes[i] == b'.') { i += 1; }
            jl.push(O { start, end: i, kind: TokenKind::Attribute });
            continue;
        }
        
        
        if ch == b'"' || ch == b'\'' {
            let start = i;
            let arw = ch;
            
            if i + 2 < len && bytes[i+1] == arw && bytes[i+2] == arw {
                i += 3;
                while i + 2 < len {
                    if bytes[i] == arw && bytes[i+1] == arw && bytes[i+2] == arw {
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
                    if bytes[i] == arw { i += 1; break; }
                    i += 1;
                }
            }
            jl.push(O { start, end: i, kind: TokenKind::String });
            continue;
        }
        
        
        if ch.is_ascii_digit() {
            let start = i;
            i += 1;
            while i < len && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_' || bytes[i] == b'.') { i += 1; }
            jl.push(O { start, end: i, kind: TokenKind::Number });
            continue;
        }
        
        
        if ch.is_ascii_alphabetic() || ch == b'_' {
            let start = i;
            while i < len && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') { i += 1; }
            let fx = &line[start..i];
            let gdv = i < len && bytes[i] == b'(';
            let kind = if CQC_.contains(&fx) {
                TokenKind::Keyword
            } else if CQB_.contains(&fx) {
                TokenKind::Type
            } else if gdv {
                TokenKind::Aq
            } else {
                TokenKind::Normal
            };
            jl.push(O { start, end: i, kind });
            continue;
        }
        
        
        if ch == b'(' || ch == b')' || ch == b'{' || ch == b'}' || ch == b'[' || ch == b']' {
            jl.push(O { start: i, end: i + 1, kind: TokenKind::Bracket });
            i += 1;
            continue;
        }
        
        
        if ch == b'+' || ch == b'-' || ch == b'*' || ch == b'/' || ch == b'=' || ch == b'!'
            || ch == b'<' || ch == b'>' || ch == b'&' || ch == b'|' || ch == b'^'
            || ch == b':' || ch == b';' || ch == b',' || ch == b'.' || ch == b'%' {
            jl.push(O { start: i, end: i + 1, kind: TokenKind::Operator });
            i += 1;
            continue;
        }
        
        i += 1;
    }
    jl
}





const CGA_: &[&str] = &[
    "break", "case", "catch", "class", "const", "continue", "debugger",
    "default", "delete", "do", "else", "export", "extends", "finally",
    "for", "function", "if", "import", "in", "instanceof", "let", "new",
    "of", "return", "super", "switch", "this", "throw", "try", "typeof",
    "var", "void", "while", "with", "yield", "async", "await", "from",
    "static", "get", "set",
];

const CGB_: &[&str] = &[
    "Array", "Boolean", "Date", "Error", "Function", "JSON", "Map", "Math",
    "Number", "Object", "Promise", "Proxy", "RegExp", "Set", "String",
    "Symbol", "WeakMap", "WeakSet", "console", "document", "window",
    "null", "undefined", "true", "false", "NaN", "Infinity",
];

const BTR_: &[&str] = &[
    "auto", "break", "case", "char", "const", "continue", "default", "do",
    "double", "else", "enum", "extern", "float", "for", "goto", "if",
    "inline", "int", "long", "register", "restrict", "return", "short",
    "signed", "sizeof", "static", "struct", "switch", "typedef", "union",
    "unsigned", "void", "volatile", "while",
    
    "bool", "catch", "class", "constexpr", "delete", "dynamic_cast",
    "explicit", "false", "friend", "mutable", "namespace", "new",
    "noexcept", "nullptr", "operator", "override", "private", "protected",
    "public", "reinterpret_cast", "static_assert", "static_cast",
    "template", "this", "throw", "true", "try", "typeid", "typename",
    "using", "virtual",
];

const BTS_: &[&str] = &[
    "int8_t", "int16_t", "int32_t", "int64_t", "uint8_t", "uint16_t",
    "uint32_t", "uint64_t", "size_t", "ssize_t", "ptrdiff_t", "intptr_t",
    "uintptr_t", "FILE", "NULL", "EOF", "string", "vector", "map",
    "set", "pair", "shared_ptr", "unique_ptr", "weak_ptr",
];

pub fn jng(line: &str, dsl: bool) -> Vec<O> {
    let mut jl = Vec::new();
    let bytes = line.as_bytes();
    let len = bytes.len();
    let mut i = 0;
    let clr: &[&str] = if dsl { BTR_ } else { CGA_ };
    let eda: &[&str] = if dsl { BTS_ } else { CGB_ };
    
    while i < len {
        let ch = bytes[i];
        
        
        if ch == b'/' && i + 1 < len && bytes[i + 1] == b'/' {
            jl.push(O { start: i, end: len, kind: TokenKind::Comment });
            break;
        }
        
        
        if ch == b'/' && i + 1 < len && bytes[i + 1] == b'*' {
            let start = i;
            i += 2;
            while i + 1 < len {
                if bytes[i] == b'*' && bytes[i+1] == b'/' { i += 2; break; }
                i += 1;
            }
            if i >= len { i = len; }
            jl.push(O { start, end: i, kind: TokenKind::Comment });
            continue;
        }
        
        
        if dsl && ch == b'#' {
            jl.push(O { start: i, end: len, kind: TokenKind::Macro });
            break;
        }
        
        
        if ch == b'"' || ch == b'\'' || ch == b'`' {
            let start = i;
            let arw = ch;
            i += 1;
            while i < len {
                if bytes[i] == b'\\' && i + 1 < len { i += 2; continue; }
                if bytes[i] == arw { i += 1; break; }
                i += 1;
            }
            jl.push(O { start, end: i, kind: TokenKind::String });
            continue;
        }
        
        
        if ch.is_ascii_digit() || (ch == b'.' && i + 1 < len && bytes[i+1].is_ascii_digit()) {
            let start = i;
            i += 1;
            while i < len && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_' || bytes[i] == b'.') { i += 1; }
            jl.push(O { start, end: i, kind: TokenKind::Number });
            continue;
        }
        
        
        if ch.is_ascii_alphabetic() || ch == b'_' || ch == b'$' {
            let start = i;
            while i < len && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_' || bytes[i] == b'$') { i += 1; }
            let fx = &line[start..i];
            let gdv = i < len && bytes[i] == b'(';
            let kind = if clr.contains(&fx) {
                TokenKind::Keyword
            } else if eda.contains(&fx) {
                TokenKind::Type
            } else if gdv {
                TokenKind::Aq
            } else {
                TokenKind::Normal
            };
            jl.push(O { start, end: i, kind });
            continue;
        }
        
        
        if ch == b'(' || ch == b')' || ch == b'{' || ch == b'}' || ch == b'[' || ch == b']' {
            jl.push(O { start: i, end: i + 1, kind: TokenKind::Bracket });
            i += 1;
            continue;
        }
        
        
        if ch == b'+' || ch == b'-' || ch == b'*' || ch == b'/' || ch == b'=' || ch == b'!'
            || ch == b'<' || ch == b'>' || ch == b'&' || ch == b'|' || ch == b'^'
            || ch == b':' || ch == b';' || ch == b',' || ch == b'.' || ch == b'?' || ch == b'%' {
            jl.push(O { start: i, end: i + 1, kind: TokenKind::Operator });
            i += 1;
            continue;
        }
        
        i += 1;
    }
    jl
}





pub fn pla(line: &str) -> Vec<O> {
    let mut jl = Vec::new();
    let bytes = line.as_bytes();
    let len = bytes.len();
    let jw = line.trim_start();
    
    
    if jw.starts_with('#') {
        jl.push(O { start: 0, end: len, kind: TokenKind::Comment });
        return jl;
    }
    
    
    if jw.starts_with('[') {
        let offset = len - jw.len();
        jl.push(O { start: offset, end: len, kind: TokenKind::Attribute });
        return jl;
    }
    
    
    let mut i = 0;
    
    while i < len && bytes[i] != b'=' {
        i += 1;
    }
    if i < len {
        
        jl.push(O { start: 0, end: i, kind: TokenKind::Type });
        
        jl.push(O { start: i, end: i + 1, kind: TokenKind::Operator });
        i += 1;
        
        while i < len && bytes[i] == b' ' { i += 1; }
        
        if i < len {
            let bwr = i;
            let cey = bytes[i];
            if cey == b'"' || cey == b'\'' {
                
                jl.push(O { start: bwr, end: len, kind: TokenKind::String });
            } else if cey == b't' || cey == b'f' {
                
                jl.push(O { start: bwr, end: len, kind: TokenKind::Keyword });
            } else if cey.is_ascii_digit() || cey == b'-' || cey == b'+' {
                
                jl.push(O { start: bwr, end: len, kind: TokenKind::Number });
            } else if cey == b'[' {
                
                jl.push(O { start: bwr, end: len, kind: TokenKind::Bracket });
            } else {
                jl.push(O { start: bwr, end: len, kind: TokenKind::Normal });
            }
            
            if let Some(hash_pos) = line[bwr..].find('#') {
                let jtg = bwr + hash_pos;
                jl.push(O { start: jtg, end: len, kind: TokenKind::Comment });
            }
        }
    } else {
        
        jl.push(O { start: 0, end: len, kind: TokenKind::Normal });
    }
    jl
}





pub const DJG_: u32 = 0xFF569CD6;  
pub const DJE_: u32    = 0xFFD7BA7D;   
pub const DJF_: u32    = 0xFFCE9178;    
pub const DJH_: u32    = 0xFF4EC9B0;    
pub const DJI_: u32    = 0xFF569CD6;    

pub fn pky(line: &str) -> Vec<O> {
    let mut jl = Vec::new();
    let bytes = line.as_bytes();
    let len = bytes.len();
    let jw = line.trim_start();
    
    
    if jw.starts_with('#') {
        jl.push(O { start: 0, end: len, kind: TokenKind::Keyword });
        return jl;
    }
    
    
    if jw.starts_with("```") {
        jl.push(O { start: 0, end: len, kind: TokenKind::String });
        return jl;
    }
    
    
    if jw.starts_with("- ") || jw.starts_with("* ") || jw.starts_with("+ ") {
        let offset = len - jw.len();
        jl.push(O { start: offset, end: offset + 2, kind: TokenKind::Macro });
        
        if offset + 2 < len {
            jl.push(O { start: offset + 2, end: len, kind: TokenKind::Normal });
        }
        return jl;
    }
    
    
    let mut i = 0;
    let mut bho = 0;
    
    while i < len {
        
        if bytes[i] == b'`' {
            if bho < i {
                jl.push(O { start: bho, end: i, kind: TokenKind::Normal });
            }
            let start = i;
            i += 1;
            while i < len && bytes[i] != b'`' { i += 1; }
            if i < len { i += 1; }
            jl.push(O { start, end: i, kind: TokenKind::String });
            bho = i;
            continue;
        }
        
        
        if bytes[i] == b'*' && i + 1 < len && bytes[i+1] == b'*' {
            if bho < i {
                jl.push(O { start: bho, end: i, kind: TokenKind::Normal });
            }
            let start = i;
            i += 2;
            while i + 1 < len {
                if bytes[i] == b'*' && bytes[i+1] == b'*' { i += 2; break; }
                i += 1;
            }
            jl.push(O { start, end: i, kind: TokenKind::Attribute });
            bho = i;
            continue;
        }
        
        
        if bytes[i] == b'[' {
            if bho < i {
                jl.push(O { start: bho, end: i, kind: TokenKind::Normal });
            }
            let start = i;
            while i < len && bytes[i] != b')' { i += 1; }
            if i < len { i += 1; }
            jl.push(O { start, end: i, kind: TokenKind::Type });
            bho = i;
            continue;
        }
        
        i += 1;
    }
    
    if bho < len {
        jl.push(O { start: bho, end: len, kind: TokenKind::Normal });
    }
    
    jl
}


pub fn pkx(line: &str, ia: Language) -> Vec<O> {
    match ia {
        Language::Rust => jnh(line),
        Language::Python => pkz(line),
        Language::JavaScript => jng(line, false),
        Language::C => jng(line, true),
        Language::Toml => pla(line),
        Language::Markdown => pky(line),
        Language::Plain => Vec::new(),
    }
}







pub fn bvh(
    state: &mut EditorState,
    x: i32, y: i32, w: u32, h: u32,
    draw_text_fn: &dyn Fn(i32, i32, &str, u32),
    draw_char_fn: &dyn Fn(i32, i32, char, u32),
) {
    let ew: i32 = 8;
    let bw: i32 = 16;
    let aej: i32 = 22;
    let euh: i32 = 22;
    let bpg: i32 = 28;
    let fjt: i32 = 18;
    let fdg = euh + bpg + fjt;
    
    
    let hsg = if state.lines.len() >= 10000 { 5 }
        else if state.lines.len() >= 1000 { 4 }
        else if state.lines.len() >= 100 { 3 }
        else { 2 };
    let ajv = (hsg + 2) * ew;
    
    let adm = x + ajv;
    let adn = y + fdg;
    let ein = w as i32 - ajv;
    let anu = h as i32 - fdg - aej;
    let oe = (anu / bw).max(1) as usize;
    
    
    if state.cursor_line < state.scroll_y {
        state.scroll_y = state.cursor_line;
    }
    if state.cursor_line >= state.scroll_y + oe {
        state.scroll_y = state.cursor_line - oe + 1;
    }
    state.blink_counter += 1;
    state.update_matching_bracket();
    
    
    let ks = y;
    crate::framebuffer::fill_rect(x as u32, ks as u32, w, euh as u32, 0xFF333333);
    
    crate::framebuffer::fill_rect(x as u32, (ks + euh - 1) as u32, w, 1, 0xFF252526);
    let nel = ["File", "Edit", "Selection", "View", "Go", "Run", "Terminal", "Help"];
    let mut cg = x + 8;
    for label in &nel {
        draw_text_fn(cg, ks + 4, label, 0xFFCCCCCC);
        cg += (label.len() as i32 + 2) * ew;
    }
    
    
    let amy = y + euh;
    crate::framebuffer::fill_rect(x as u32, amy as u32, w, bpg as u32, 0xFF252526);
    
    let gxw = state.file_path.as_ref().map(|aa| {
        aa.rsplit('/').next().unwrap_or(aa.as_str())
    }).unwrap_or("untitled");
    let fsi = if state.dirty { " *" } else { "" };
    
    let mwf = match state.language {
        Language::Rust => "RS",
        Language::Python => "PY",
        Language::JavaScript => "JS",
        Language::C => " C",
        Language::Toml => "TL",
        Language::Markdown => "MD",
        Language::Plain => "  ",
    };
    let ebe = format!(" {} {} {}  x", mwf, gxw, fsi);
    let zm = ((ebe.len() as u32) * 8 + 4).min(w);
    
    crate::framebuffer::fill_rect(x as u32, amy as u32, zm, bpg as u32, ND_);
    
    crate::framebuffer::fill_rect(x as u32, amy as u32, zm, 2, 0xFF007ACC);
    
    draw_text_fn(x + 4, amy + 8, &ebe, KH_);
    
    
    let dis = amy + bpg;
    crate::framebuffer::fill_rect(x as u32, dis as u32, w, fjt as u32, 0xFF1E1E1E);
    if let Some(ref path) = state.file_path {
        
        let mut bx = x + ajv + 4;
        let mut start = 0;
        let bytes = path.as_bytes();
        for i in 0..=bytes.len() {
            if i == bytes.len() || bytes[i] == b'/' {
                if i > start {
                    let jn = &path[start..i];
                    if start > 0 {
                        draw_text_fn(bx, dis + 2, " > ", 0xFF666666);
                        bx += 3 * ew;
                    }
                    draw_text_fn(bx, dis + 2, jn, 0xFF858585);
                    bx += jn.len() as i32 * ew;
                }
                start = i + 1;
            }
        }
    } else {
        draw_text_fn(x + ajv + 4, dis + 2, "untitled", 0xFF858585);
    }
    
    crate::framebuffer::fill_rect(x as u32, (dis + fjt - 1) as u32, w, 1, 0xFF333333);
    
    
    crate::framebuffer::fill_rect(x as u32, adn as u32, w, anu as u32, ND_);
    
    
    crate::framebuffer::fill_rect(x as u32, adn as u32, ajv as u32, anu as u32, AQE_);
    crate::framebuffer::fill_rect((x + ajv - 1) as u32, adn as u32, 1, anu as u32, 0xFF333333);
    
    
    for pt in 0..oe {
        let xf = state.scroll_y + pt;
        if xf >= state.lines.len() { break; }
        
        let ly = adn + (pt as i32 * bw);
        if ly + bw > adn + anu { break; }
        
        let eri = xf == state.cursor_line;
        
        
        if eri {
            crate::framebuffer::fill_rect(
                x as u32, ly as u32,
                ajv as u32, bw as u32,
                0xFF1A1D26,
            );
            crate::framebuffer::fill_rect(
                adm as u32, ly as u32,
                ein as u32, bw as u32,
                AQB_,
            );
        }
        
        
        if let Some((sl, dr, el, ec)) = state.get_selection_range() {
            if xf >= sl && xf <= el {
                let wh = state.lines[xf].len();
                let ded = if xf == sl { dr.min(wh) } else { 0 };
                let ezw = if xf == el { ec.min(wh) } else { wh };
                if ded < ezw {
                    let am = adm + 4 + (ded as i32 * ew);
                    let dy = ((ezw - ded) as i32 * ew) as u32;
                    crate::framebuffer::fill_rect(
                        am as u32, ly as u32, dy, bw as u32, 0xFF264F78,
                    );
                }
            }
        }
        
        
        let myo = &state.lines[xf];
        let mut igj = 0usize;
        for b in myo.bytes() {
            if b == b' ' { igj += 1; } else { break; }
        }
        let mop = igj / 4;
        for level in 0..mop {
            let idd = adm + 4 + (level as i32 * 4 * ew);
            if idd < x + w as i32 {
                let mgp = if eri { 0xFF505050 } else { 0xFF404040 };
                crate::framebuffer::fill_rect(idd as u32, ly as u32, 1, bw as u32, mgp);
            }
        }
        
        
        if let Some((ml, mc)) = state.matching_bracket {
            if xf == ml {
                let bx = adm + 4 + (mc as i32 * ew);
                crate::framebuffer::fill_rect(bx as u32, ly as u32, ew as u32, bw as u32, 0xFF3A3D41);
                crate::framebuffer::fill_rect(bx as u32, ly as u32, ew as u32, 1, 0xFF888888);
                crate::framebuffer::fill_rect(bx as u32, (ly + bw - 1) as u32, ew as u32, 1, 0xFF888888);
                crate::framebuffer::fill_rect(bx as u32, ly as u32, 1, bw as u32, 0xFF888888);
                crate::framebuffer::fill_rect((bx + ew - 1) as u32, ly as u32, 1, bw as u32, 0xFF888888);
            }
            if xf == state.cursor_line && state.cursor_col < state.lines[xf].len() {
                let cb = state.lines[xf].as_bytes()[state.cursor_col];
                if matches!(cb, b'(' | b')' | b'{' | b'}' | b'[' | b']') {
                    let dkj = adm + 4 + (state.cursor_col as i32 * ew);
                    crate::framebuffer::fill_rect(dkj as u32, ly as u32, ew as u32, bw as u32, 0xFF3A3D41);
                    crate::framebuffer::fill_rect(dkj as u32, ly as u32, ew as u32, 1, 0xFF888888);
                    crate::framebuffer::fill_rect(dkj as u32, (ly + bw - 1) as u32, ew as u32, 1, 0xFF888888);
                    crate::framebuffer::fill_rect(dkj as u32, ly as u32, 1, bw as u32, 0xFF888888);
                    crate::framebuffer::fill_rect((dkj + ew - 1) as u32, ly as u32, 1, bw as u32, 0xFF888888);
                }
            }
        }
        
        
        let mym = format!("{:>width$} ", xf + 1, width = hsg as usize);
        let dvm = if eri { 0xFFC6C6C6 } else { AQH_ };
        draw_text_fn(x + 2, ly, &mym, dvm);
        
        
        let line = &state.lines[xf];
        let tokens = pkx(line, state.language);
        if !tokens.is_empty() {
            for bjg in &tokens {
                let color = jnf(bjg.kind);
                let pil = &line[bjg.start..bjg.end];
                let am = adm + 4 + (bjg.start as i32 * ew);
                if am < x + w as i32 {
                    draw_text_fn(am, ly, pil, color);
                }
            }
        } else if !line.is_empty() {
            draw_text_fn(adm + 4, ly, line, KH_);
        }
        
        
        if eri {
            let fjk = (state.blink_counter / 30) % 2 == 0;
            if fjk {
                let cx = adm + 4 + (state.cursor_col as i32 * ew);
                crate::framebuffer::fill_rect(cx as u32, ly as u32, 2, bw as u32, AQC_);
            }
        }
    }
    
    
    if state.lines.len() > oe {
        let yc = (x + w as i32 - 10) as u32;
        let bdo = anu as u32;
        let zo = ((oe as u32 * bdo) / state.lines.len() as u32).max(20);
        let akn = (state.scroll_y as u32 * (bdo - zo)) / state.lines.len().saturating_sub(oe) as u32;
        crate::framebuffer::fill_rect(yc + 3, adn as u32, 7, bdo, 0xFF252526);
        crate::framebuffer::fill_rounded_rect(yc + 3, adn as u32 + akn, 7, zo, 3, 0xFF6A6A6A);
    }
    
    
    if state.find_query.is_some() {
        let hyu: i32 = if state.replace_text.is_some() { 56 } else { 32 };
        let cxv: i32 = 370.min(ein);
        let dpq = x + w as i32 - cxv - 20;
        let dpr = adn + 4;
        
        crate::framebuffer::fill_rect((dpq + 2) as u32, (dpr + 2) as u32, cxv as u32, hyu as u32, 0xFF0A0A0A);
        crate::framebuffer::fill_rect(dpq as u32, dpr as u32, cxv as u32, hyu as u32, 0xFF252526);
        crate::framebuffer::fill_rect(dpq as u32, dpr as u32, cxv as u32, 1, 0xFF007ACC);
        
        let query = state.find_query.as_deref().unwrap_or("");
        let nch = if state.find_matches.is_empty() {
            if query.is_empty() { String::new() } else { String::from(" No results") }
        } else {
            format!(" {}/{}", state.find_match_idx + 1, state.find_matches.len())
        };
        let hyt = !state.find_replace_mode;
        let cxs = dpq + 8;
        let emn = dpr + 6;
        let emm = cxv - 100;
        crate::framebuffer::fill_rect(cxs as u32, emn as u32, emm as u32, 18, if hyt { 0xFF3C3C3C } else { 0xFF333333 });
        if hyt {
            crate::framebuffer::fill_rect(cxs as u32, (emn + 17) as u32, emm as u32, 1, 0xFF007ACC);
        }
        draw_text_fn(cxs + 4, emn + 2, query, 0xFFCCCCCC);
        draw_text_fn(dpq + cxv - 90, emn + 2, &nch, 0xFF858585);
        
        if let Some(ref replace) = state.replace_text {
            let grk = dpr + 30;
            let jab = state.find_replace_mode;
            crate::framebuffer::fill_rect(cxs as u32, grk as u32, emm as u32, 18, if jab { 0xFF3C3C3C } else { 0xFF333333 });
            if jab {
                crate::framebuffer::fill_rect(cxs as u32, (grk + 17) as u32, emm as u32, 1, 0xFF007ACC);
            }
            draw_text_fn(cxs + 4, grk + 2, replace, 0xFFCCCCCC);
        }
        
        
        let exi = query.len();
        if exi > 0 {
            for &(ml, mc) in &state.find_matches {
                if ml >= state.scroll_y && ml < state.scroll_y + oe {
                    let pt = ml - state.scroll_y;
                    let ghw = adn + (pt as i32 * bw);
                    let ghx = adm + 4 + (mc as i32 * ew);
                    let buk = (exi as i32 * ew) as u32;
                    crate::framebuffer::fill_rect(ghx as u32, ghw as u32, buk, bw as u32, 0xFF613214);
                    if state.find_match_idx < state.find_matches.len() && state.find_matches[state.find_match_idx] == (ml, mc) {
                        crate::framebuffer::fill_rect(ghx as u32, ghw as u32, buk, 1, 0xFFE8AB53);
                        crate::framebuffer::fill_rect(ghx as u32, (ghw + bw - 1) as u32, buk, 1, 0xFFE8AB53);
                    }
                }
            }
        }
    }
    
    
    if let Some(ref input) = state.goto_line_input {
        let cii: i32 = 320.min(w as i32 - 40);
        let hsb: i32 = 32;
        let cij = x + (w as i32 - cii) / 2;
        let ekc = y + fdg + 2;
        crate::framebuffer::fill_rect((cij + 2) as u32, (ekc + 2) as u32, cii as u32, hsb as u32, 0xFF0A0A0A);
        crate::framebuffer::fill_rect(cij as u32, ekc as u32, cii as u32, hsb as u32, 0xFF252526);
        crate::framebuffer::fill_rect(cij as u32, ekc as u32, cii as u32, 2, 0xFF007ACC);
        let sv = ekc + 6;
        crate::framebuffer::fill_rect((cij + 8) as u32, sv as u32, (cii - 16) as u32, 18, 0xFF3C3C3C);
        crate::framebuffer::fill_rect((cij + 8) as u32, (sv + 17) as u32, (cii - 16) as u32, 1, 0xFF007ACC);
        let mfj = format!(":{}", input);
        draw_text_fn(cij + 12, sv + 2, &mfj, 0xFFCCCCCC);
        let hint = format!("Go to Line (1-{})", state.lines.len());
        let drk = cij + cii - (hint.len() as i32 * ew) - 12;
        draw_text_fn(drk, sv + 2, &hint, 0xFF666666);
    }

    
    let status_y = y + fdg + anu;
    crate::framebuffer::fill_rect(x as u32, status_y as u32, w, aej as u32, ABU_);
    
    
    let mut deo = x + 8;
    draw_text_fn(deo, status_y + 4, "@ main", GS_);
    deo += 7 * ew;
    crate::framebuffer::fill_rect(deo as u32, (status_y + 4) as u32, 1, 14, 0xFF1A6DAA);
    deo += 6;
    if state.dirty {
        draw_text_fn(deo, status_y + 4, "* Modified", 0xFFFFD166);
    } else {
        draw_text_fn(deo, status_y + 4, "Saved", GS_);
    }
    
    
    let bdb = format!("Ln {}, Col {}", state.cursor_line + 1, state.cursor_col + 1);
    let ijh = state.language.name();
    
    let mut amw = x + w as i32 - 8;
    
    amw -= bdb.len() as i32 * ew;
    draw_text_fn(amw, status_y + 4, &bdb, GS_);
    amw -= 10;
    crate::framebuffer::fill_rect(amw as u32, (status_y + 4) as u32, 1, 14, 0xFF1A6DAA);
    amw -= 6;
    
    amw -= ijh.len() as i32 * ew;
    draw_text_fn(amw, status_y + 4, ijh, GS_);
    amw -= 10;
    crate::framebuffer::fill_rect(amw as u32, (status_y + 4) as u32, 1, 14, 0xFF1A6DAA);
    amw -= 6;
    
    amw -= 5 * ew;
    draw_text_fn(amw, status_y + 4, "UTF-8", GS_);
    amw -= 10;
    crate::framebuffer::fill_rect(amw as u32, (status_y + 4) as u32, 1, 14, 0xFF1A6DAA);
    amw -= 6;
    
    amw -= 9 * ew;
    draw_text_fn(amw, status_y + 4, "Spaces: 4", GS_);
}
