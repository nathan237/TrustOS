








extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use super::{eh, ew, qu,
            P_, F_, M_, AC_, AK_, AN_, BG_, AU_};


pub struct EditorState {
    
    pub lines: Vec<String>,
    
    pub cursor_line: usize,
    
    pub cursor_col: usize,
    
    pub scroll: usize,
    
    pub output: Vec<String>,
    
    pub output_focused: bool,
    
    pub output_scroll: usize,
    
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
        use crate::keyboard::{T_, S_, AI_, AJ_, AM_, AO_};
        
        self.frame += 1;
        
        match key {
            T_ => {
                if self.output_focused {
                    self.output_scroll = self.output_scroll.saturating_sub(1);
                } else if self.cursor_line > 0 {
                    self.cursor_line -= 1;
                    self.clamp_col();
                }
            }
            S_ => {
                if self.output_focused {
                    self.output_scroll += 1;
                } else if self.cursor_line + 1 < self.lines.len() {
                    self.cursor_line += 1;
                    self.clamp_col();
                }
            }
            AI_ => {
                if !self.output_focused && self.cursor_col > 0 {
                    self.cursor_col -= 1;
                }
            }
            AJ_ => {
                if !self.output_focused {
                    let wh = self.lines.get(self.cursor_line)
                        .map(|l| l.len()).unwrap_or(0);
                    if self.cursor_col < wh {
                        self.cursor_col += 1;
                    }
                }
            }
            AM_ => {
                if self.output_focused {
                    self.output_scroll = self.output_scroll.saturating_sub(5);
                } else {
                    self.cursor_line = self.cursor_line.saturating_sub(10);
                    self.clamp_col();
                }
            }
            AO_ => {
                if self.output_focused {
                    self.output_scroll += 5;
                } else {
                    self.cursor_line = (self.cursor_line + 10).min(self.lines.len().saturating_sub(1));
                    self.clamp_col();
                }
            }
            
            0x12 => {
                self.run_code();
            }
            
            0x13 => {
                self.save_file();
            }
            
            0x0D | 0x0A => {
                if !self.output_focused {
                    
                    if self.cursor_line < self.lines.len() {
                        let col = self.cursor_col.min(self.lines[self.cursor_line].len());
                        let ef = self.lines[self.cursor_line].split_off(col);
                        self.cursor_line += 1;
                        self.lines.insert(self.cursor_line, ef);
                        self.cursor_col = 0;
                    } else {
                        self.lines.push(String::new());
                        self.cursor_line = self.lines.len() - 1;
                        self.cursor_col = 0;
                    }
                }
            }
            
            0x08 => {
                if !self.output_focused {
                    if self.cursor_col > 0 {
                        self.cursor_col -= 1;
                        if self.cursor_line < self.lines.len() {
                            self.lines[self.cursor_line].remove(self.cursor_col);
                        }
                    } else if self.cursor_line > 0 {
                        
                        let current = self.lines.remove(self.cursor_line);
                        self.cursor_line -= 1;
                        self.cursor_col = self.lines[self.cursor_line].len();
                        self.lines[self.cursor_line].push_str(&current);
                    }
                }
            }
            
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
    
    
    pub fn handle_click(&mut self, x: i32, y: i32, w: u32, h: u32) {
        let aq = super::ew();
        let ee = super::qu() + 1;
        if ee <= 0 || aq <= 0 { return; }

        
        let doj = (h as i32 * 60 / 100).max(ee * 3);
        let dwb = doj + 2;

        if y >= dwb {
            
            self.output_focused = true;
        } else {
            
            self.output_focused = false;

            
            let adn = ee;
            if y < adn { return; } 

            let ajv = 4 * aq;
            let row = ((y - adn) / ee) as usize;
            let col = ((x - ajv).max(0) / aq) as usize;

            self.cursor_line = (self.scroll + row).min(self.lines.len().saturating_sub(1));
            let wh = self.lines.get(self.cursor_line).map(|l| l.len()).unwrap_or(0);
            self.cursor_col = col.min(wh);
            self.frame += 1; 
        }
    }

    fn clamp_col(&mut self) {
        let wh = self.lines.get(self.cursor_line)
            .map(|l| l.len()).unwrap_or(0);
        if self.cursor_col > wh {
            self.cursor_col = wh;
        }
    }
    
    
    pub fn save_file(&mut self) {
        let source: String = self.lines.iter()
            .map(|l| l.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        
        match crate::vfs::write_file("/mnt/trustfs/editor.tl", source.as_bytes()) {
            Ok(()) => {
                
                let _ = crate::vfs::jkk();
                self.output.clear();
                self.output.push(String::from("=== Saved ==="));
                self.output.push(format!("Wrote {} bytes to /mnt/trustfs/editor.tl", source.len()));
                self.output.push(String::from("File persisted to disk (WAL protected)"));
            }
            Err(e) => {
                self.output.clear();
                self.output.push(format!("Save error: {:?}", e));
                self.output.push(String::from("Try: file saved to ramfs as fallback"));
                
                let _ = crate::ramfs::bh(|fs| {
                    if !fs.exists("editor.tl") { let _ = fs.touch("editor.tl"); }
                    fs.write_file("editor.tl", source.as_bytes())
                });
            }
        }
    }

    
    pub fn run_code(&mut self) {
        self.output.clear();
        self.output.push(String::from("=== Running ==="));
        
        
        let source: String = self.lines.iter()
            .map(|l| l.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        
        
        self.output.push(format!("Source: {} lines, {} bytes", self.lines.len(), source.len()));
        
        
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


pub fn draw(state: &EditorState, x: i32, y: i32, w: u32, h: u32) {
    let aq = ew();
    let ee = qu() + 1;
    if ee <= 0 || aq <= 0 { return; }
    
    
    let doj = (h as i32 * 60 / 100).max(ee * 3);
    let dwb = y + doj + 2;
    let ita = h as i32 - doj - 4;
    
    
    
    let header = if state.output_focused { "Editor (Ctrl+O)" } else { "Editor [active]" };
    let mko = if !state.output_focused { M_ } else { F_ };
    eh(x, y, header, mko);
    
    
    let hint = "[Ctrl+S] save [Ctrl+R] run";
    let drk = x + w as i32 - (hint.len() as i32 * aq) - 2;
    eh(drk, y, hint, AC_);
    
    let adn = y + ee;
    let anu = doj - ee;
    let hbp = (anu / ee) as usize;
    
    
    let scroll = if state.cursor_line >= state.scroll + hbp {
        state.cursor_line - hbp + 1
    } else if state.cursor_line < state.scroll {
        state.cursor_line
    } else {
        state.scroll
    };
    
    let ajv = 4 * aq; 
    let adm = x + ajv;
    
    let end = (scroll + hbp).min(state.lines.len());
    let mut u = adn;
    
    for i in scroll..end {
        
        let axw = format!("{:>3}", i + 1);
        eh(x, u, &axw, F_);
        
        
        if i == state.cursor_line && !state.output_focused {
            crate::framebuffer::fill_rect(
                adm as u32, u as u32,
                w.saturating_sub(ajv as u32), ee as u32,
                0xFF1C2128,
            );
        }
        
        
        eko(adm, u, &state.lines[i], w.saturating_sub(ajv as u32));
        
        
        if i == state.cursor_line && !state.output_focused {
            if (state.frame / 25) % 2 == 0 {
                let cursor_x = adm + (state.cursor_col as i32 * aq);
                crate::framebuffer::fill_rect(
                    cursor_x as u32, u as u32,
                    2, ee as u32,
                    M_,
                );
            }
        }
        
        u += ee;
    }
    
    
    crate::framebuffer::fill_rect(x as u32, (dwb - 1) as u32, w, 1, 0xFF30363D);
    
    
    if ita <= 0 { return; }
    
    let noi = if state.output_focused { "Output [active]" } else { "Output (Ctrl+O)" };
    let nof = if state.output_focused { AC_ } else { F_ };
    eh(x, dwb, noi, nof);
    
    let nok = dwb + ee;
    let psj = ((ita - ee) / ee) as usize;
    let isx = state.output_scroll.min(state.output.len().saturating_sub(1));
    let noh = (isx + psj).min(state.output.len());
    
    let mut hk = nok;
    for i in isx..noh {
        let line = &state.output[i];
        let color = if line.starts_with("Error:") {
            AN_
        } else if line.starts_with("=>") {
            AC_
        } else if line.starts_with("===") {
            AK_
        } else {
            P_
        };
        
        eh(x + 4, hk, line, color);
        hk += ee;
    }
}


fn eko(x: i32, y: i32, line: &str, _max_w: u32) {
    let aq = ew();
    let clr = ["fn", "let", "mut", "if", "else", "for", "while", "return", 
                     "true", "false", "struct", "enum", "match", "pub", "use",
                     "const", "static", "impl", "self", "loop", "break", "continue"];
    
    let mut cx = x;
    let chars: Vec<char> = line.chars().collect();
    let len = chars.len();
    let mut i = 0;
    
    while i < len {
        let ch = chars[i];
        
        
        if ch == '/' && i + 1 < len && chars[i + 1] == '/' {
            
            let ef: String = chars[i..].iter().collect();
            eh(cx, y, &ef, F_);
            return;
        }
        
        
        if ch == '"' {
            let start = i;
            i += 1;
            while i < len && chars[i] != '"' {
                if chars[i] == '\\' { i += 1; } 
                i += 1;
            }
            if i < len { i += 1; } 
            let j: String = chars[start..i].iter().collect();
            eh(cx, y, &j, AC_);
            cx += j.len() as i32 * aq;
            continue;
        }
        
        
        if ch.is_ascii_digit() {
            let start = i;
            while i < len && (chars[i].is_ascii_digit() || chars[i] == '.' || chars[i] == 'x') {
                i += 1;
            }
            let num: String = chars[start..i].iter().collect();
            eh(cx, y, &num, AU_);
            cx += num.len() as i32 * aq;
            continue;
        }
        
        
        if ch.is_ascii_alphanumeric() || ch == '_' {
            let start = i;
            while i < len && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            let fx: String = chars[start..i].iter().collect();
            let color = if clr.contains(&fx.as_str()) {
                BG_
            } else if fx.chars().next().map(|c| c.is_ascii_uppercase()).unwrap_or(false) {
                AK_ 
            } else {
                P_
            };
            eh(cx, y, &fx, color);
            cx += fx.len() as i32 * aq;
            continue;
        }
        
        
        let color = match ch {
            '(' | ')' | '{' | '}' | '[' | ']' => AK_,
            '=' | '+' | '-' | '*' | '/' | '<' | '>' | '!' | '&' | '|' => M_,
            ';' | ':' | ',' | '.' => F_,
            _ => P_,
        };
        let j = alloc::format!("{}", ch);
        eh(cx, y, &j, color);
        cx += aq;
        i += 1;
    }
}
