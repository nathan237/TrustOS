//! File System Tree Panel — Live VFS tree browser
//!
//! Shows the virtual file system as an expandable tree.
//! Directories can be expanded/collapsed. Highlights open files.

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use super::{draw_lab_text, char_w, char_h,
            COL_TEXT, COL_DIM, COL_ACCENT, COL_CYAN, COL_GREEN, COL_YELLOW};

/// A node in the tree view
#[derive(Clone)]
struct TreeNode {
    /// Name (just the filename, not full path)
    name: String,
    /// Full path
    path: String,
    /// Is this a directory?
    is_dir: bool,
    /// Depth level (0 = root)
    depth: usize,
    /// Is expanded? (only for dirs)
    expanded: bool,
    /// File size (for files)
    size: u64,
}

/// File tree panel state
pub struct FileTreeState {
    /// Flattened visible tree nodes
    pub nodes: Vec<TreeNode>,
    /// Selected node index
    pub selected: usize,
    /// Scroll offset
    pub scroll: usize,
    /// Whether tree needs rebuild
    dirty: bool,
    /// Refresh counter
    refresh_counter: u64,
}

impl FileTreeState {
    pub fn new() -> Self {
        let mut s = Self {
            nodes: Vec::new(),
            selected: 0,
            scroll: 0,
            dirty: true,
            refresh_counter: 0,
        };
        s.rebuild_tree();
        s
    }
    
    /// Rebuild the flattened tree from VFS
    fn rebuild_tree(&mut self) {
        self.nodes.clear();
        self.add_directory("/", 0);
        self.dirty = false;
    }
    
    /// Recursively add directory contents
    fn add_directory(&mut self, path: &str, depth: usize) {
        // Add the directory node itself (except root)
        if depth > 0 {
            let name = path.rsplit('/').find(|s| !s.is_empty())
                .unwrap_or(path);
            self.nodes.push(TreeNode {
                name: String::from(name),
                path: String::from(path),
                is_dir: true,
                depth,
                expanded: depth <= 1, // Auto-expand first 2 levels
                size: 0,
            });
        }
        
        // Only enumerate if expanded (or root)
        let should_expand = depth == 0 || self.nodes.last()
            .map(|n| n.expanded).unwrap_or(false);
        
        if !should_expand || depth > 6 {
            return;
        }
        
        // Read directory entries from VFS
        if let Ok(entries) = crate::vfs::readdir(path) {
            // Sort: directories first, then alphabetical
            let mut dirs: Vec<_> = entries.iter()
                .filter(|e| e.file_type == crate::vfs::FileType::Directory)
                .collect();
            let mut files: Vec<_> = entries.iter()
                .filter(|e| e.file_type != crate::vfs::FileType::Directory)
                .collect();
            dirs.sort_by(|a, b| a.name.cmp(&b.name));
            files.sort_by(|a, b| a.name.cmp(&b.name));
            
            // Add subdirectories
            for entry in &dirs {
                let child_path = if path == "/" {
                    format!("/{}", entry.name)
                } else {
                    format!("{}/{}", path, entry.name)
                };
                self.add_directory(&child_path, depth + 1);
            }
            
            // Add files
            for entry in &files {
                // Get file size via stat
                let fpath = if path == "/" {
                    format!("/{}", entry.name)
                } else {
                    format!("{}/{}", path, entry.name)
                };
                let fsize = crate::vfs::stat(&fpath).map(|s| s.size).unwrap_or(0);
                self.nodes.push(TreeNode {
                    name: entry.name.clone(),
                    path: fpath,
                    is_dir: false,
                    depth: depth + 1,
                    expanded: false,
                    size: fsize,
                });
            }
        }
    }
    
    pub fn handle_key(&mut self, key: u8) {
        use crate::keyboard::{KEY_UP, KEY_DOWN, KEY_PGUP, KEY_PGDOWN};
        match key {
            KEY_UP => {
                self.selected = self.selected.saturating_sub(1);
            }
            KEY_DOWN => {
                if self.selected + 1 < self.nodes.len() {
                    self.selected += 1;
                }
            }
            KEY_PGUP => {
                self.selected = self.selected.saturating_sub(10);
            }
            KEY_PGDOWN => {
                self.selected = (self.selected + 10).min(self.nodes.len().saturating_sub(1));
            }
            // Enter = toggle expand/collapse
            0x0D | 0x0A => {
                if self.selected < self.nodes.len() && self.nodes[self.selected].is_dir {
                    self.nodes[self.selected].expanded = !self.nodes[self.selected].expanded;
                    self.dirty = true;
                }
            }
            // 'r' = refresh
            b'r' | b'R' => {
                self.dirty = true;
            }
            _ => {}
        }
        
        if self.dirty {
            // Save expanded state
            let expanded_paths: Vec<String> = self.nodes.iter()
                .filter(|n| n.is_dir && n.expanded)
                .map(|n| n.path.clone())
                .collect();
            
            self.nodes.clear();
            self.add_directory_with_state("/", 0, &expanded_paths);
            self.dirty = false;
            
            // Clamp selection
            if self.selected >= self.nodes.len() && !self.nodes.is_empty() {
                self.selected = self.nodes.len() - 1;
            }
        }
    }
    
    /// Rebuild tree preserving expanded state
    fn add_directory_with_state(&mut self, path: &str, depth: usize, expanded: &[String]) {
        if depth > 0 {
            let name = path.rsplit('/').find(|s| !s.is_empty())
                .unwrap_or(path);
            let is_expanded = expanded.iter().any(|p| p == path);
            self.nodes.push(TreeNode {
                name: String::from(name),
                path: String::from(path),
                is_dir: true,
                depth,
                expanded: is_expanded,
                size: 0,
            });
            
            if !is_expanded || depth > 6 {
                return;
            }
        }
        
        let should_expand = depth == 0 || self.nodes.last()
            .map(|n| n.expanded).unwrap_or(true);
        
        if !should_expand {
            return;
        }
        
        if let Ok(entries) = crate::vfs::readdir(path) {
            let mut dirs: Vec<_> = entries.iter()
                .filter(|e| e.file_type == crate::vfs::FileType::Directory)
                .collect();
            let mut files: Vec<_> = entries.iter()
                .filter(|e| e.file_type != crate::vfs::FileType::Directory)
                .collect();
            dirs.sort_by(|a, b| a.name.cmp(&b.name));
            files.sort_by(|a, b| a.name.cmp(&b.name));
            
            for entry in &dirs {
                let child_path = if path == "/" {
                    format!("/{}", entry.name)
                } else {
                    format!("{}/{}", path, entry.name)
                };
                self.add_directory_with_state(&child_path, depth + 1, expanded);
            }
            
            for entry in &files {
                let fpath = if path == "/" {
                    format!("/{}", entry.name)
                } else {
                    format!("{}/{}", path, entry.name)
                };
                let fsize = crate::vfs::stat(&fpath).map(|s| s.size).unwrap_or(0);
                self.nodes.push(TreeNode {
                    name: entry.name.clone(),
                    path: fpath,
                    is_dir: false,
                    depth: depth + 1,
                    expanded: false,
                    size: fsize,
                });
            }
        }
    }
}

/// Draw the file tree panel
pub fn draw(state: &FileTreeState, x: i32, y: i32, w: u32, h: u32) {
    let cw = char_w();
    let lh = char_h() + 1;
    if lh <= 0 || cw <= 0 { return; }
    
    // Header with file count
    let header = format!("/ ({} items)", state.nodes.len());
    draw_lab_text(x, y, &header, COL_ACCENT);
    
    let list_y = y + lh + 2;
    let list_h = h as i32 - lh - 2;
    if list_h <= 0 { return; }
    
    let visible = (list_h / lh) as usize;
    
    if state.nodes.is_empty() {
        draw_lab_text(x + 4, list_y, "Empty filesystem", COL_DIM);
        return;
    }
    
    // Auto-scroll to keep selection visible
    let scroll = if state.selected >= state.scroll + visible {
        state.selected - visible + 1
    } else if state.selected < state.scroll {
        state.selected
    } else {
        state.scroll
    };
    
    let end = (scroll + visible).min(state.nodes.len());
    let mut cy = list_y;
    
    for i in scroll..end {
        let node = &state.nodes[i];
        let is_selected = i == state.selected;
        
        // Highlight selected
        if is_selected {
            crate::framebuffer::fill_rect(x as u32, cy as u32, w, lh as u32, 0xFF1F2937);
        }
        
        // Indent
        let indent = node.depth as i32 * 2 * cw;
        let nx = x + indent;
        
        // Icon
        let (icon, icon_color) = if node.is_dir {
            if node.expanded { ("v ", COL_YELLOW) } else { ("> ", COL_YELLOW) }
        } else {
            let ext_color = file_color(&node.name);
            ("  ", ext_color)
        };
        draw_lab_text(nx, cy, icon, icon_color);
        
        // Name
        let name_x = nx + 2 * cw;
        let name_color = if node.is_dir {
            COL_CYAN
        } else {
            file_color(&node.name)
        };
        
        let max_name_w = w as i32 - (name_x - x) - 10 * cw;
        let max_chars = if cw > 0 { (max_name_w / cw) as usize } else { 20 };
        let name = if node.name.len() > max_chars && max_chars > 3 {
            &node.name[..max_chars.saturating_sub(1)]
        } else {
            &node.name
        };
        draw_lab_text(name_x, cy, name, name_color);
        
        // Size (for files)
        if !node.is_dir && node.size > 0 {
            let size_str = format_size(node.size);
            let sx = x + w as i32 - (size_str.len() as i32 * cw) - 4;
            if sx > name_x + (name.len() as i32 * cw) + cw {
                draw_lab_text(sx, cy, &size_str, COL_DIM);
            }
        }
        
        cy += lh;
        if cy > y + h as i32 { break; }
    }
    
    // Scroll indicator
    if state.nodes.len() > visible {
        let track_h = list_h;
        let thumb_h = ((visible as i32 * track_h) / state.nodes.len() as i32).max(8);
        let thumb_pos = (scroll as i32 * (track_h - thumb_h)) / state.nodes.len().saturating_sub(1).max(1) as i32;
        let sb_x = (x + w as i32 - 3) as u32;
        crate::framebuffer::fill_rect(sb_x, list_y as u32, 2, track_h as u32, 0xFF21262D);
        crate::framebuffer::fill_rect(sb_x, (list_y + thumb_pos) as u32, 2, thumb_h as u32, COL_ACCENT);
    }
}

/// Get color for file extension
fn file_color(name: &str) -> u32 {
    if let Some(ext) = name.rsplit('.').next() {
        match ext {
            "rs" => 0xFFD18616,     // Rust = orange
            "toml" | "cfg" | "conf" => COL_GREEN,
            "md" | "txt" => COL_TEXT,
            "sh" | "ps1" | "bat" => COL_YELLOW,
            "elf" | "bin" => 0xFFF85149,  // Red
            "tl" => 0xFFBC8CFF,     // TrustLang = purple
            _ => COL_DIM,
        }
    } else {
        COL_DIM
    }
}

/// Format file size
fn format_size(bytes: u64) -> String {
    if bytes >= 1024 * 1024 {
        format!("{:.1}M", bytes as f64 / (1024.0 * 1024.0))
    } else if bytes >= 1024 {
        format!("{:.1}K", bytes as f64 / 1024.0)
    } else {
        format!("{}B", bytes)
    }
}
