//! File System Tree Panel — Live VFS tree browser
//!
//! Shows the virtual file system as an expandable tree.
//! Directories can be expanded/collapsed. Uses ramfs (the actual filesystem).

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
    pub dirty: bool,
}

impl FileTreeState {
    pub fn new() -> Self {
        let mut s = Self {
            nodes: Vec::new(),
            selected: 0,
            scroll: 0,
            dirty: true,
        };
        s.rebuild_tree();
        s
    }
    
    /// Rebuild the flattened tree from ramfs
    fn rebuild_tree(&mut self) {
        self.nodes.clear();
        // Add root as first node
        self.nodes.push(TreeNode {
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
    
    /// Add children of a directory from ramfs
    fn add_directory_children(&mut self, path: &str, depth: usize) {
        if depth > 6 { return; }
        
        // Read from ramfs (the actual filesystem used by TrustOS)
        let entries = crate::ramfs::with_fs(|fs| {
            fs.ls(Some(path)).unwrap_or_default()
        });
        
        if entries.is_empty() { return; }
        
        // Sort: directories first, then alphabetical
        let mut dirs: Vec<_> = entries.iter()
            .filter(|(_, ft, _)| *ft == crate::ramfs::FileType::Directory)
            .collect();
        let mut files: Vec<_> = entries.iter()
            .filter(|(_, ft, _)| *ft != crate::ramfs::FileType::Directory)
            .collect();
        dirs.sort_by(|a, b| a.0.cmp(&b.0));
        files.sort_by(|a, b| a.0.cmp(&b.0));
        
        // Add subdirectories
        for (name, _, _) in &dirs {
            let child_path = if path == "/" {
                format!("/{}", name)
            } else {
                format!("{}/{}", path, name)
            };
            // Check if this dir was expanded
            let is_expanded = depth < 1; // auto-expand first level
            self.nodes.push(TreeNode {
                name: name.clone(),
                path: child_path.clone(),
                is_dir: true,
                depth: depth + 1,
                expanded: is_expanded,
                size: 0,
            });
            if is_expanded {
                self.add_directory_children(&child_path, depth + 1);
            }
        }
        
        // Add files
        for (name, _, size) in &files {
            let fpath = if path == "/" {
                format!("/{}", name)
            } else {
                format!("{}/{}", path, name)
            };
            self.nodes.push(TreeNode {
                name: name.clone(),
                path: fpath,
                is_dir: false,
                depth: depth + 1,
                expanded: false,
                size: *size as u64,
            });
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
                self.toggle_selected();
            }
            // 'r' = refresh
            b'r' | b'R' => {
                self.dirty = true;
            }
            _ => {}
        }
        
        if self.dirty {
            self.rebuild_with_state();
        }
    }
    
    /// Handle mouse click at (x, y) relative to content area
    pub fn handle_click(&mut self, x: i32, y: i32, w: u32, h: u32) {
        let lh = super::char_h() + 1;
        if lh <= 0 { return; }

        // Same layout as draw(): header line then list
        let list_y = lh + 2;
        if y < list_y { return; } // clicked on header

        let row = ((y - list_y) / lh) as usize;
        let target = self.scroll + row;
        if target < self.nodes.len() {
            self.selected = target;
            // If it's a directory, toggle expand/collapse
            if self.nodes[target].is_dir {
                self.toggle_selected();
            }
        }
    }

    /// Toggle expand/collapse on the selected directory
    fn toggle_selected(&mut self) {
        if self.selected >= self.nodes.len() { return; }
        if !self.nodes[self.selected].is_dir { return; }
        
        self.nodes[self.selected].expanded = !self.nodes[self.selected].expanded;
        self.dirty = true;
    }
    
    /// Rebuild tree preserving expanded state
    fn rebuild_with_state(&mut self) {
        // Save expanded paths
        let expanded_paths: Vec<String> = self.nodes.iter()
            .filter(|n| n.is_dir && n.expanded)
            .map(|n| n.path.clone())
            .collect();
        
        let old_selected = self.selected;
        self.nodes.clear();
        
        // Root
        let root_expanded = expanded_paths.iter().any(|p| p == "/");
        self.nodes.push(TreeNode {
            name: String::from("/"),
            path: String::from("/"),
            is_dir: true,
            depth: 0,
            expanded: root_expanded,
            size: 0,
        });
        
        if root_expanded {
            self.add_directory_children_with_state("/", 0, &expanded_paths);
        }
        
        self.dirty = false;
        
        // Clamp selection
        if self.selected >= self.nodes.len() && !self.nodes.is_empty() {
            self.selected = self.nodes.len() - 1;
        }
    }
    
    fn add_directory_children_with_state(&mut self, path: &str, depth: usize, expanded: &[String]) {
        if depth > 6 { return; }
        
        let entries = crate::ramfs::with_fs(|fs| {
            fs.ls(Some(path)).unwrap_or_default()
        });
        
        let mut dirs: Vec<_> = entries.iter()
            .filter(|(_, ft, _)| *ft == crate::ramfs::FileType::Directory)
            .collect();
        let mut files: Vec<_> = entries.iter()
            .filter(|(_, ft, _)| *ft != crate::ramfs::FileType::Directory)
            .collect();
        dirs.sort_by(|a, b| a.0.cmp(&b.0));
        files.sort_by(|a, b| a.0.cmp(&b.0));
        
        for (name, _, _) in &dirs {
            let child_path = if path == "/" {
                format!("/{}", name)
            } else {
                format!("{}/{}", path, name)
            };
            let is_expanded = expanded.iter().any(|p| p == &child_path);
            self.nodes.push(TreeNode {
                name: name.clone(),
                path: child_path.clone(),
                is_dir: true,
                depth: depth + 1,
                expanded: is_expanded,
                size: 0,
            });
            if is_expanded {
                self.add_directory_children_with_state(&child_path, depth + 1, expanded);
            }
        }
        
        for (name, _, size) in &files {
            let fpath = if path == "/" {
                format!("/{}", name)
            } else {
                format!("{}/{}", path, name)
            };
            self.nodes.push(TreeNode {
                name: name.clone(),
                path: fpath,
                is_dir: false,
                depth: depth + 1,
                expanded: false,
                size: *size as u64,
            });
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
