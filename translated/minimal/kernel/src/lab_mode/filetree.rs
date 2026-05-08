




extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use super::{eh, ew, qu,
            P_, F_, M_, AU_, AC_, AK_};


#[derive(Clone)]
struct Jd {
    
    name: String,
    
    path: String,
    
    is_dir: bool,
    
    depth: usize,
    
    expanded: bool,
    
    size: u64,
}


pub struct FileTreeState {
    
    pub nodes: Vec<Jd>,
    
    pub selected: usize,
    
    pub scroll: usize,
    
    pub dirty: bool,
}

impl FileTreeState {
    pub fn new() -> Self {
        let mut j = Self {
            nodes: Vec::new(),
            selected: 0,
            scroll: 0,
            dirty: true,
        };
        j.rebuild_tree();
        j
    }
    
    
    fn rebuild_tree(&mut self) {
        self.nodes.clear();
        
        self.nodes.push(Jd {
            name: String::from("/"),
            path: String::from("/"),
            is_dir: true,
            depth: 0,
            expanded: true,
            size: 0,
        });
        self.add_directory_children("/", 0);
        self.dirty = false;
    }
    
    
    fn add_directory_children(&mut self, path: &str, depth: usize) {
        if depth > 6 { return; }
        
        
        let entries = crate::ramfs::bh(|fs| {
            fs.ls(Some(path)).unwrap_or_default()
        });
        
        if entries.is_empty() { return; }
        
        
        let mut bfy: Vec<_> = entries.iter()
            .filter(|(_, qk, _)| *qk == crate::ramfs::FileType::Directory)
            .collect();
        let mut files: Vec<_> = entries.iter()
            .filter(|(_, qk, _)| *qk != crate::ramfs::FileType::Directory)
            .collect();
        bfy.sort_by(|a, b| a.0.cmp(&b.0));
        files.sort_by(|a, b| a.0.cmp(&b.0));
        
        
        for (name, _, _) in &bfy {
            let bxx = if path == "/" {
                format!("/{}", name)
            } else {
                format!("{}/{}", path, name)
            };
            
            let dsn = depth < 1; 
            self.nodes.push(Jd {
                name: name.clone(),
                path: bxx.clone(),
                is_dir: true,
                depth: depth + 1,
                expanded: dsn,
                size: 0,
            });
            if dsn {
                self.add_directory_children(&bxx, depth + 1);
            }
        }
        
        
        for (name, _, size) in &files {
            let fxn = if path == "/" {
                format!("/{}", name)
            } else {
                format!("{}/{}", path, name)
            };
            self.nodes.push(Jd {
                name: name.clone(),
                path: fxn,
                is_dir: false,
                depth: depth + 1,
                expanded: false,
                size: *size as u64,
            });
        }
    }
    
    pub fn handle_key(&mut self, key: u8) {
        use crate::keyboard::{T_, S_, AM_, AO_};
        match key {
            T_ => {
                self.selected = self.selected.saturating_sub(1);
            }
            S_ => {
                if self.selected + 1 < self.nodes.len() {
                    self.selected += 1;
                }
            }
            AM_ => {
                self.selected = self.selected.saturating_sub(10);
            }
            AO_ => {
                self.selected = (self.selected + 10).min(self.nodes.len().saturating_sub(1));
            }
            
            0x0D | 0x0A => {
                self.toggle_selected();
            }
            
            b'r' | b'R' => {
                self.dirty = true;
            }
            _ => {}
        }
        
        if self.dirty {
            self.rebuild_with_state();
        }
    }
    
    
    pub fn handle_click(&mut self, x: i32, y: i32, w: u32, h: u32) {
        let ee = super::qu() + 1;
        if ee <= 0 { return; }

        
        let gc = ee + 2;
        if y < gc { return; } 

        let row = ((y - gc) / ee) as usize;
        let target = self.scroll + row;
        if target < self.nodes.len() {
            self.selected = target;
            
            if self.nodes[target].is_dir {
                self.toggle_selected();
            }
        }
    }

    
    fn toggle_selected(&mut self) {
        if self.selected >= self.nodes.len() { return; }
        if !self.nodes[self.selected].is_dir { return; }
        
        self.nodes[self.selected].expanded = !self.nodes[self.selected].expanded;
        self.dirty = true;
    }
    
    
    fn rebuild_with_state(&mut self) {
        
        let hxe: Vec<String> = self.nodes.iter()
            .filter(|ae| ae.is_dir && ae.expanded)
            .map(|ae| ae.path.clone())
            .collect();
        
        let qpw = self.selected;
        self.nodes.clear();
        
        
        let jbj = hxe.iter().any(|aa| aa == "/");
        self.nodes.push(Jd {
            name: String::from("/"),
            path: String::from("/"),
            is_dir: true,
            depth: 0,
            expanded: jbj,
            size: 0,
        });
        
        if jbj {
            self.add_directory_children_with_state("/", 0, &hxe);
        }
        
        self.dirty = false;
        
        
        if self.selected >= self.nodes.len() && !self.nodes.is_empty() {
            self.selected = self.nodes.len() - 1;
        }
    }
    
    fn add_directory_children_with_state(&mut self, path: &str, depth: usize, expanded: &[String]) {
        if depth > 6 { return; }
        
        let entries = crate::ramfs::bh(|fs| {
            fs.ls(Some(path)).unwrap_or_default()
        });
        
        let mut bfy: Vec<_> = entries.iter()
            .filter(|(_, qk, _)| *qk == crate::ramfs::FileType::Directory)
            .collect();
        let mut files: Vec<_> = entries.iter()
            .filter(|(_, qk, _)| *qk != crate::ramfs::FileType::Directory)
            .collect();
        bfy.sort_by(|a, b| a.0.cmp(&b.0));
        files.sort_by(|a, b| a.0.cmp(&b.0));
        
        for (name, _, _) in &bfy {
            let bxx = if path == "/" {
                format!("/{}", name)
            } else {
                format!("{}/{}", path, name)
            };
            let dsn = expanded.iter().any(|aa| aa == &bxx);
            self.nodes.push(Jd {
                name: name.clone(),
                path: bxx.clone(),
                is_dir: true,
                depth: depth + 1,
                expanded: dsn,
                size: 0,
            });
            if dsn {
                self.add_directory_children_with_state(&bxx, depth + 1, expanded);
            }
        }
        
        for (name, _, size) in &files {
            let fxn = if path == "/" {
                format!("/{}", name)
            } else {
                format!("{}/{}", path, name)
            };
            self.nodes.push(Jd {
                name: name.clone(),
                path: fxn,
                is_dir: false,
                depth: depth + 1,
                expanded: false,
                size: *size as u64,
            });
        }
    }
}


pub fn draw(state: &FileTreeState, x: i32, y: i32, w: u32, h: u32) {
    let aq = ew();
    let ee = qu() + 1;
    if ee <= 0 || aq <= 0 { return; }
    
    
    let header = format!("/ ({} items)", state.nodes.len());
    eh(x, y, &header, M_);
    
    let gc = y + ee + 2;
    let abc = h as i32 - ee - 2;
    if abc <= 0 { return; }
    
    let visible = (abc / ee) as usize;
    
    if state.nodes.is_empty() {
        eh(x + 4, gc, "Empty filesystem", F_);
        return;
    }
    
    
    let scroll = if state.selected >= state.scroll + visible {
        state.selected - visible + 1
    } else if state.selected < state.scroll {
        state.selected
    } else {
        state.scroll
    };
    
    let end = (scroll + visible).min(state.nodes.len());
    let mut u = gc;
    
    for i in scroll..end {
        let uf = &state.nodes[i];
        let hd = i == state.selected;
        
        
        if hd {
            crate::framebuffer::fill_rect(x as u32, u as u32, w, ee as u32, 0xFF1F2937);
        }
        
        
        let axq = uf.depth as i32 * 2 * aq;
        let nx = x + axq;
        
        
        let (icon, icon_color) = if uf.is_dir {
            if uf.expanded { ("v ", AK_) } else { ("> ", AK_) }
        } else {
            let ltf = hye(&uf.name);
            ("  ", ltf)
        };
        eh(nx, u, icon, icon_color);
        
        
        let bcv = nx + 2 * aq;
        let ayi = if uf.is_dir {
            AU_
        } else {
            hye(&uf.name)
        };
        
        let nde = w as i32 - (bcv - x) - 10 * aq;
        let nd = if aq > 0 { (nde / aq) as usize } else { 20 };
        let name = if uf.name.len() > nd && nd > 3 {
            &uf.name[..nd.saturating_sub(1)]
        } else {
            &uf.name
        };
        eh(bcv, u, name, ayi);
        
        
        if !uf.is_dir && uf.size > 0 {
            let td = aqo(uf.size);
            let am = x + w as i32 - (td.len() as i32 * aq) - 4;
            if am > bcv + (name.len() as i32 * aq) + aq {
                eh(am, u, &td, F_);
            }
        }
        
        u += ee;
        if u > y + h as i32 { break; }
    }
    
    
    if state.nodes.len() > visible {
        let ada = abc;
        let zo = ((visible as i32 * ada) / state.nodes.len() as i32).max(8);
        let ebq = (scroll as i32 * (ada - zo)) / state.nodes.len().saturating_sub(1).max(1) as i32;
        let yc = (x + w as i32 - 3) as u32;
        crate::framebuffer::fill_rect(yc, gc as u32, 2, ada as u32, 0xFF21262D);
        crate::framebuffer::fill_rect(yc, (gc + ebq) as u32, 2, zo as u32, M_);
    }
}


fn hye(name: &str) -> u32 {
    if let Some(ext) = name.rsplit('.').next() {
        match ext {
            "rs" => 0xFFD18616,     
            "toml" | "cfg" | "conf" => AC_,
            "md" | "txt" => P_,
            "sh" | "ps1" | "bat" => AK_,
            "elf" | "bin" => 0xFFF85149,  
            "tl" => 0xFFBC8CFF,     
            _ => F_,
        }
    } else {
        F_
    }
}


fn aqo(bytes: u64) -> String {
    if bytes >= 1024 * 1024 {
        format!("{:.1}M", bytes as f64 / (1024.0 * 1024.0))
    } else if bytes >= 1024 {
        format!("{:.1}K", bytes as f64 / 1024.0)
    } else {
        format!("{}B", bytes)
    }
}
