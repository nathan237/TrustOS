//! Command Guide Panel — Interactive searchable reference of all TrustOS commands
//!
//! Shows a categorized list of all shell commands with descriptions.
//! Supports fuzzy search and category filtering.

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use super::{draw_lab_text, char_w, char_h,
            COL_TEXT, COL_DIM, COL_ACCENT, COL_GREEN, COL_YELLOW, COL_CYAN, COL_PURPLE};

/// A command entry
struct CmdEntry {
    name: &'static str,
    category: &'static str,
    description: &'static str,
}

const COMMANDS: &[CmdEntry] = &[
    // File System
    CmdEntry { name: "ls", category: "FS", description: "List directory contents" },
    CmdEntry { name: "cd", category: "FS", description: "Change directory" },
    CmdEntry { name: "pwd", category: "FS", description: "Print working directory" },
    CmdEntry { name: "mkdir", category: "FS", description: "Create directory" },
    CmdEntry { name: "rmdir", category: "FS", description: "Remove directory" },
    CmdEntry { name: "touch", category: "FS", description: "Create empty file" },
    CmdEntry { name: "rm", category: "FS", description: "Remove file" },
    CmdEntry { name: "cp", category: "FS", description: "Copy file" },
    CmdEntry { name: "mv", category: "FS", description: "Move or rename file" },
    CmdEntry { name: "cat", category: "FS", description: "Display file contents" },
    CmdEntry { name: "head", category: "FS", description: "Show first lines of file" },
    CmdEntry { name: "tail", category: "FS", description: "Show last lines of file" },
    CmdEntry { name: "tree", category: "FS", description: "Show directory tree" },
    CmdEntry { name: "find", category: "FS", description: "Search for files" },
    CmdEntry { name: "stat", category: "FS", description: "Display file statistics" },
    CmdEntry { name: "hexdump", category: "FS", description: "Hex dump of file" },
    // System
    CmdEntry { name: "help", category: "SYS", description: "Show command help" },
    CmdEntry { name: "clear", category: "SYS", description: "Clear terminal screen" },
    CmdEntry { name: "time", category: "SYS", description: "Show current time" },
    CmdEntry { name: "uptime", category: "SYS", description: "Show system uptime" },
    CmdEntry { name: "date", category: "SYS", description: "Show current date" },
    CmdEntry { name: "whoami", category: "SYS", description: "Show current user" },
    CmdEntry { name: "uname", category: "SYS", description: "System information" },
    CmdEntry { name: "ps", category: "SYS", description: "List running processes" },
    CmdEntry { name: "free", category: "SYS", description: "Display memory usage" },
    CmdEntry { name: "top", category: "SYS", description: "System monitor" },
    CmdEntry { name: "dmesg", category: "SYS", description: "Kernel message buffer" },
    CmdEntry { name: "reboot", category: "SYS", description: "Reboot the system" },
    CmdEntry { name: "shutdown", category: "SYS", description: "Shut down the system" },
    // Network
    CmdEntry { name: "ifconfig", category: "NET", description: "Network interface config" },
    CmdEntry { name: "ping", category: "NET", description: "Send ICMP echo request" },
    CmdEntry { name: "curl", category: "NET", description: "Transfer data from URL" },
    CmdEntry { name: "wget", category: "NET", description: "Download file from URL" },
    CmdEntry { name: "nslookup", category: "NET", description: "DNS lookup" },
    CmdEntry { name: "arp", category: "NET", description: "ARP table" },
    CmdEntry { name: "netstat", category: "NET", description: "Network statistics" },
    // GUI
    CmdEntry { name: "desktop", category: "GUI", description: "Launch graphical desktop" },
    CmdEntry { name: "open", category: "GUI", description: "Open file with GUI app" },
    CmdEntry { name: "trustedit", category: "GUI", description: "3D model editor" },
    // Dev Tools
    CmdEntry { name: "trustview", category: "DEV", description: "Binary analysis viewer" },
    CmdEntry { name: "trustlang", category: "DEV", description: "TrustLang REPL" },
    CmdEntry { name: "transpile", category: "DEV", description: "Binary-to-Rust transpiler" },
    CmdEntry { name: "lab", category: "DEV", description: "TrustLab introspection" },
    // Hardware
    CmdEntry { name: "lspci", category: "HW", description: "List PCI devices" },
    CmdEntry { name: "lshw", category: "HW", description: "List hardware" },
    CmdEntry { name: "disk", category: "HW", description: "Disk information" },
    CmdEntry { name: "fdisk", category: "HW", description: "Partition table" },
    CmdEntry { name: "audio", category: "HW", description: "Audio subsystem" },
    CmdEntry { name: "beep", category: "HW", description: "Play a beep tone" },
    // Fun
    CmdEntry { name: "neofetch", category: "FUN", description: "System info with ASCII art" },
    CmdEntry { name: "matrix", category: "FUN", description: "Matrix rain animation" },
    CmdEntry { name: "cowsay", category: "FUN", description: "ASCII cow says message" },
];

/// Guide panel state
pub struct GuideState {
    /// Search query
    pub search: String,
    /// Search cursor
    pub cursor: usize,
    /// Scroll offset in filtered results
    pub scroll: usize,
    /// Selected entry index
    pub selected: usize,
    /// Active category filter (None = all)
    pub category_filter: Option<&'static str>,
}

impl GuideState {
    pub fn new() -> Self {
        Self {
            search: String::new(),
            cursor: 0,
            scroll: 0,
            selected: 0,
            category_filter: None,
        }
    }
    
    pub fn handle_key(&mut self, key: u8) {
        use crate::keyboard::{KEY_UP, KEY_DOWN, KEY_PGUP, KEY_PGDOWN};
        match key {
            KEY_UP => {
                self.selected = self.selected.saturating_sub(1);
            }
            KEY_DOWN => {
                self.selected += 1;
            }
            KEY_PGUP => {
                self.selected = self.selected.saturating_sub(10);
            }
            KEY_PGDOWN => {
                self.selected += 10;
            }
            // Backspace
            0x08 => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                    self.search.remove(self.cursor);
                    self.selected = 0;
                    self.scroll = 0;
                }
            }
            // Ctrl+U = clear search
            0x15 => {
                self.search.clear();
                self.cursor = 0;
                self.selected = 0;
                self.scroll = 0;
            }
            _ => {}
        }
    }
    
    pub fn handle_char(&mut self, ch: char) {
        if ch.is_ascii_graphic() || ch == ' ' {
            self.search.insert(self.cursor, ch);
            self.cursor += 1;
            self.selected = 0;
            self.scroll = 0;
        }
    }

    /// Handle mouse click inside the guide panel content area
    pub fn handle_click(&mut self, local_x: i32, local_y: i32, w: u32, _h: u32) {
        let cw = char_w();
        let lh = char_h() + 1;
        if lh <= 0 || cw <= 0 { return; }

        // Row 0: search bar (click = focus, we already focus panel)
        // Row 1: category tabs
        let cat_y = lh;
        if local_y >= cat_y && local_y < cat_y + lh {
            let cats: [&str; 8] = ["ALL", "FS", "SYS", "NET", "GUI", "DEV", "HW", "FUN"];
            let cat_vals: [Option<&str>; 8] = [None, Some("FS"), Some("SYS"), Some("NET"), Some("GUI"), Some("DEV"), Some("HW"), Some("FUN")];
            let mut tx = 0i32;
            for (i, cat) in cats.iter().enumerate() {
                let label_end = tx + (cat.len() as i32 + 1) * cw;
                if local_x >= tx && local_x < label_end {
                    self.category_filter = cat_vals[i];
                    self.selected = 0;
                    self.scroll = 0;
                    return;
                }
                tx = label_end;
                if tx > w as i32 - 10 { break; }
            }
            return;
        }

        // Below: command list rows (after search + cats + separator)
        let list_y = cat_y + lh + 5; // lh + 2px separator + 3px gap
        if local_y >= list_y {
            let row = ((local_y - list_y) / lh) as usize;
            let clicked = self.scroll + row;
            let filtered = self.filtered_commands();
            if clicked < filtered.len() {
                self.selected = clicked;
            }
        }
    }
    
    fn filtered_commands(&self) -> Vec<&CmdEntry> {
        COMMANDS.iter()
            .filter(|c| {
                // Category filter
                if let Some(cat) = self.category_filter {
                    if c.category != cat { return false; }
                }
                // Search filter
                if !self.search.is_empty() {
                    let q = self.search.to_ascii_lowercase();
                    let name_match = c.name.to_ascii_lowercase().contains(&q);
                    let desc_match = c.description.to_ascii_lowercase().contains(&q);
                    if !name_match && !desc_match { return false; }
                }
                true
            })
            .collect()
    }
}

/// Draw the command guide panel
pub fn draw(state: &GuideState, x: i32, y: i32, w: u32, h: u32) {
    let cw = char_w();
    let lh = char_h() + 1;
    if lh <= 0 || cw <= 0 { return; }
    
    let mut cy = y;
    
    // Search bar
    let search_label = "/ ";
    draw_lab_text(x, cy, search_label, COL_GREEN);
    let input_x = x + 2 * cw;
    if state.search.is_empty() {
        draw_lab_text(input_x, cy, "type to search...", COL_DIM);
    } else {
        draw_lab_text(input_x, cy, &state.search, COL_TEXT);
    }
    cy += lh;
    
    // Category tabs
    let cats = ["ALL", "FS", "SYS", "NET", "GUI", "DEV", "HW", "FUN"];
    let cat_colors = [COL_ACCENT, COL_CYAN, COL_GREEN, COL_YELLOW, COL_PURPLE, COL_ACCENT, 0xFFD18616, 0xFFBC8CFF];
    let mut tx = x;
    for (i, cat) in cats.iter().enumerate() {
        let active = match state.category_filter {
            None => i == 0,
            Some(c) => *cat == c,
        };
        let color = if active { cat_colors[i] } else { COL_DIM };
        draw_lab_text(tx, cy, cat, color);
        tx += (cat.len() as i32 + 1) * cw;
        if tx > x + w as i32 - 10 { break; }
    }
    cy += lh + 2;
    
    // Separator
    crate::framebuffer::fill_rect(x as u32, cy as u32, w, 1, 0xFF30363D);
    cy += 3;
    
    // Command list
    let filtered = state.filtered_commands();
    let visible = ((h as i32 - (cy - y)) / lh) as usize;
    
    if filtered.is_empty() {
        draw_lab_text(x + 4, cy, "No matching commands", COL_DIM);
        return;
    }
    
    // Clamp selected
    let selected = state.selected.min(filtered.len().saturating_sub(1));
    
    // Auto-scroll to keep selection visible
    let scroll = if selected >= state.scroll + visible {
        selected - visible + 1
    } else if selected < state.scroll {
        selected
    } else {
        state.scroll
    };
    
    let end = (scroll + visible).min(filtered.len());
    
    for i in scroll..end {
        let cmd = filtered[i];
        let is_selected = i == selected;
        
        // Highlight selected row
        if is_selected {
            crate::framebuffer::fill_rect(x as u32, cy as u32, w, lh as u32, 0xFF1F2937);
        }
        
        // Category badge
        let cat_color = match cmd.category {
            "FS" => COL_CYAN,
            "SYS" => COL_GREEN,
            "NET" => COL_YELLOW,
            "GUI" => COL_PURPLE,
            "DEV" => COL_ACCENT,
            "HW" => 0xFFD18616,
            "FUN" => 0xFFBC8CFF,
            _ => COL_DIM,
        };
        draw_lab_text(x + 2, cy, cmd.category, cat_color);
        
        // Command name
        let name_x = x + 6 * cw;
        draw_lab_text(name_x, cy, cmd.name, if is_selected { COL_ACCENT } else { COL_TEXT });
        
        // Description
        let desc_x = name_x + 14 * cw;
        let max_desc = if cw > 0 { ((w as i32 - (desc_x - x)) / cw) as usize } else { 20 };
        let desc = if cmd.description.len() > max_desc && max_desc > 0 {
            &cmd.description[..max_desc]
        } else {
            cmd.description
        };
        draw_lab_text(desc_x, cy, desc, COL_DIM);
        
        cy += lh;
        if cy > y + h as i32 { break; }
    }
    
    // Result count
    let count_str = format!("{}/{}", filtered.len(), COMMANDS.len());
    let count_x = x + w as i32 - (count_str.len() as i32 * cw) - 4;
    draw_lab_text(count_x, y, &count_str, COL_DIM);
}
