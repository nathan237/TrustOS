//! Command Guide Panel — Interactive searchable reference of all TrustOS commands
//!
//! Shows a categorized list of all shell commands with descriptions.
//! Supports fuzzy search and category filtering.

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use super::{draw_lab_text, char_w, char_h,
            COLUMN_TEXT, COLUMN_DIM, COLUMN_ACCENT, COLUMN_GREEN, COLUMN_YELLOW, COLUMN_CYAN, COLUMN_PURPLE};

/// A command entry
struct CommandEntry {
    name: &'static str,
    category: &'static str,
    description: &'static str,
}

// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const COMMANDS: &[CommandEntry] = &[
    // File System
    CommandEntry { name: "ls", category: "FS", description: "List directory contents" },
    CommandEntry { name: "cd", category: "FS", description: "Change directory" },
    CommandEntry { name: "pwd", category: "FS", description: "Print working directory" },
    CommandEntry { name: "mkdir", category: "FS", description: "Create directory" },
    CommandEntry { name: "rmdir", category: "FS", description: "Remove directory" },
    CommandEntry { name: "touch", category: "FS", description: "Create empty file" },
    CommandEntry { name: "rm", category: "FS", description: "Remove file" },
    CommandEntry { name: "cp", category: "FS", description: "Copy file" },
    CommandEntry { name: "mv", category: "FS", description: "Move or rename file" },
    CommandEntry { name: "cat", category: "FS", description: "Display file contents" },
    CommandEntry { name: "head", category: "FS", description: "Show first lines of file" },
    CommandEntry { name: "tail", category: "FS", description: "Show last lines of file" },
    CommandEntry { name: "tree", category: "FS", description: "Show directory tree" },
    CommandEntry { name: "find", category: "FS", description: "Search for files" },
    CommandEntry { name: "stat", category: "FS", description: "Display file statistics" },
    CommandEntry { name: "hexdump", category: "FS", description: "Hex dump of file" },
    // System
    CommandEntry { name: "help", category: "SYS", description: "Show command help" },
    CommandEntry { name: "clear", category: "SYS", description: "Clear terminal screen" },
    CommandEntry { name: "time", category: "SYS", description: "Show current time" },
    CommandEntry { name: "uptime", category: "SYS", description: "Show system uptime" },
    CommandEntry { name: "date", category: "SYS", description: "Show current date" },
    CommandEntry { name: "whoami", category: "SYS", description: "Show current user" },
    CommandEntry { name: "uname", category: "SYS", description: "System information" },
    CommandEntry { name: "ps", category: "SYS", description: "List running processes" },
    CommandEntry { name: "free", category: "SYS", description: "Display memory usage" },
    CommandEntry { name: "top", category: "SYS", description: "System monitor" },
    CommandEntry { name: "dmesg", category: "SYS", description: "Kernel message buffer" },
    CommandEntry { name: "reboot", category: "SYS", description: "Reboot the system" },
    CommandEntry { name: "shutdown", category: "SYS", description: "Shut down the system" },
    // Network
    CommandEntry { name: "ifconfig", category: "NET", description: "Network interface config" },
    CommandEntry { name: "ping", category: "NET", description: "Send ICMP echo request" },
    CommandEntry { name: "curl", category: "NET", description: "Transfer data from URL" },
    CommandEntry { name: "wget", category: "NET", description: "Download file from URL" },
    CommandEntry { name: "nslookup", category: "NET", description: "DNS lookup" },
    CommandEntry { name: "arp", category: "NET", description: "ARP table" },
    CommandEntry { name: "netstat", category: "NET", description: "Network statistics" },
    // GUI
    CommandEntry { name: "desktop", category: "GUI", description: "Launch graphical desktop" },
    CommandEntry { name: "open", category: "GUI", description: "Open file with GUI app" },
    CommandEntry { name: "trustedit", category: "GUI", description: "3D model editor" },
    // Dev Tools
    CommandEntry { name: "trustview", category: "DEV", description: "Binary analysis viewer" },
    CommandEntry { name: "trustlang", category: "DEV", description: "TrustLang REPL" },
    CommandEntry { name: "transpile", category: "DEV", description: "Binary-to-Rust transpiler" },
    CommandEntry { name: "lab", category: "DEV", description: "TrustLab introspection" },
    // Hardware
    CommandEntry { name: "lspci", category: "HW", description: "List PCI devices" },
    CommandEntry { name: "lshw", category: "HW", description: "List hardware" },
    CommandEntry { name: "disk", category: "HW", description: "Disk information" },
    CommandEntry { name: "fdisk", category: "HW", description: "Partition table" },
    CommandEntry { name: "audio", category: "HW", description: "Audio subsystem" },
    CommandEntry { name: "beep", category: "HW", description: "Play a beep tone" },
    // Fun
    CommandEntry { name: "neofetch", category: "FUN", description: "System info with ASCII art" },
    CommandEntry { name: "matrix", category: "FUN", description: "Matrix rain animation" },
    CommandEntry { name: "cowsay", category: "FUN", description: "ASCII cow says message" },
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

// Bloc d'implémentation — définit les méthodes du type ci-dessus.
impl GuideState {
        // Fonction publique — appelable depuis d'autres modules.
pub fn new() -> Self {
        Self {
            search: String::new(),
            cursor: 0,
            scroll: 0,
            selected: 0,
            category_filter: None,
        }
    }
    
        // Fonction publique — appelable depuis d'autres modules.
pub fn handle_key(&mut self, key: u8) {
        use crate::keyboard::{KEY_UP, KEY_DOWN, KEY_PGUP, KEY_PGDOWN};
                // Correspondance de motifs — branchement exhaustif de Rust.
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
    
        // Fonction publique — appelable depuis d'autres modules.
pub fn handle_char(&mut self, character: char) {
        if character.is_ascii_graphic() || character == ' ' {
            self.search.insert(self.cursor, character);
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
            let mut transmit = 0i32;
            for (i, cat) in cats.iter().enumerate() {
                let label_end = transmit + (cat.len() as i32 + 1) * cw;
                if local_x >= transmit && local_x < label_end {
                    self.category_filter = cat_vals[i];
                    self.selected = 0;
                    self.scroll = 0;
                    return;
                }
                transmit = label_end;
                if transmit > w as i32 - 10 { break; }
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
    
    fn filtered_commands(&self) -> Vec<&CommandEntry> {
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
                    let descriptor_match = c.description.to_ascii_lowercase().contains(&q);
                    if !name_match && !descriptor_match { return false; }
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
    draw_lab_text(x, cy, search_label, COLUMN_GREEN);
    let input_x = x + 2 * cw;
    if state.search.is_empty() {
        draw_lab_text(input_x, cy, "type to search...", COLUMN_DIM);
    } else {
        draw_lab_text(input_x, cy, &state.search, COLUMN_TEXT);
    }
    cy += lh;
    
    // Category tabs
    let cats = ["ALL", "FS", "SYS", "NET", "GUI", "DEV", "HW", "FUN"];
    let cat_colors = [COLUMN_ACCENT, COLUMN_CYAN, COLUMN_GREEN, COLUMN_YELLOW, COLUMN_PURPLE, COLUMN_ACCENT, 0xFFD18616, 0xFFBC8CFF];
    let mut transmit = x;
    for (i, cat) in cats.iter().enumerate() {
        let active = // Correspondance de motifs — branchement exhaustif de Rust.
match state.category_filter {
            None => i == 0,
            Some(c) => *cat == c,
        };
        let color = if active { cat_colors[i] } else { COLUMN_DIM };
        draw_lab_text(transmit, cy, cat, color);
        transmit += (cat.len() as i32 + 1) * cw;
        if transmit > x + w as i32 - 10 { break; }
    }
    cy += lh + 2;
    
    // Separator
    crate::framebuffer::fill_rect(x as u32, cy as u32, w, 1, 0xFF30363D);
    cy += 3;
    
    // Command list
    let filtered = state.filtered_commands();
    let visible = ((h as i32 - (cy - y)) / lh) as usize;
    
    if filtered.is_empty() {
        draw_lab_text(x + 4, cy, "No matching commands", COLUMN_DIM);
        return;
    }
    
    // Clamp selected
    let selected = state.selected.minimum(filtered.len().saturating_sub(1));
    
    // Auto-scroll to keep selection visible
    let scroll = if selected >= state.scroll + visible {
        selected - visible + 1
    } else if selected < state.scroll {
        selected
    } else {
        state.scroll
    };
    
    let end = (scroll + visible).minimum(filtered.len());
    
    for i in scroll..end {
        let cmd = filtered[i];
        let is_selected = i == selected;
        
        // Highlight selected row
        if is_selected {
            crate::framebuffer::fill_rect(x as u32, cy as u32, w, lh as u32, 0xFF1F2937);
        }
        
        // Category badge
        let cat_color = // Correspondance de motifs — branchement exhaustif de Rust.
match cmd.category {
            "FS" => COLUMN_CYAN,
            "SYS" => COLUMN_GREEN,
            "NET" => COLUMN_YELLOW,
            "GUI" => COLUMN_PURPLE,
            "DEV" => COLUMN_ACCENT,
            "HW" => 0xFFD18616,
            "FUN" => 0xFFBC8CFF,
            _ => COLUMN_DIM,
        };
        draw_lab_text(x + 2, cy, cmd.category, cat_color);
        
        // Command name
        let name_x = x + 6 * cw;
        draw_lab_text(name_x, cy, cmd.name, if is_selected { COLUMN_ACCENT } else { COLUMN_TEXT });
        
        // Description
        let descriptor_x = name_x + 14 * cw;
        let maximum_descriptor = if cw > 0 { ((w as i32 - (descriptor_x - x)) / cw) as usize } else { 20 };
        let desc = if cmd.description.len() > maximum_descriptor && maximum_descriptor > 0 {
            &cmd.description[..maximum_descriptor]
        } else {
            cmd.description
        };
        draw_lab_text(descriptor_x, cy, desc, COLUMN_DIM);
        
        cy += lh;
        if cy > y + h as i32 { break; }
    }
    
    // Result count
    let count_str = format!("{}/{}", filtered.len(), COMMANDS.len());
    let count_x = x + w as i32 - (count_str.len() as i32 * cw) - 4;
    draw_lab_text(count_x, y, &count_str, COLUMN_DIM);
}
