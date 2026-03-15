//! TrustView — Ghidra-style Binary Viewer Desktop App
//!
//! Multi-panel binary analysis viewer for TrustOS desktop.
//! Panels: Navigation Tree | Hex View | Disassembly | Info/Xrefs
//! Fully interactive with keyboard & mouse navigation.

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use alloc::collections::BTreeMap;

use crate::binary_analysis::{BinaryFile, Section, Symbol, Instruction, DetectedFunction, XrefType};

/// Title bar height (mirrors desktop.rs constant)
const TITLE_BAR_HEIGHT: u32 = 28;

// ──── Color Palette (Ghidra-inspired dark theme) ───────────────────────────

const BG_DARK: u32       = 0xFF0D1117;  // Main background
const BG_PANEL: u32      = 0xFF161B22;  // Panel background
const BG_HEADER: u32     = 0xFF21262D;  // Panel header bar
const BG_SELECTED: u32   = 0xFF1F3A5F;  // Selected line
const BG_HOVER: u32      = 0xFF1C2333;  // Hover highlight
const BG_ADDRESS: u32       = 0xFF0D1117;  // Address column

const COLUMN_ADDRESS: u32      = 0xFF8B949E;  // Address text (gray)
const COLUMN_HEX: u32       = 0xFFC9D1D9;  // Hex bytes
const COLUMN_ASCII: u32     = 0xFF7EE787;  // ASCII chars (green)
const COLUMN_MNEMONIC: u32  = 0xFF79C0FF;  // Instruction mnemonic (blue)
const COLUMN_REGISTER: u32  = 0xFFD2A8FF;  // Registers (purple)
const COLUMN_IMMEDIATE: u32 = 0xFFFFA657;  // Immediate values (orange)
const COLUMN_COMMENT: u32   = 0xFF8B949E;  // Comments (gray)
const COLUMN_LABEL: u32     = 0xFFFF7B72;  // Labels/symbols (red)
const COLUMN_STRING: u32    = 0xFF7EE787;  // String references (green)
const COLUMN_HEADER: u32    = 0xFFFFFFFF;  // Panel header text
const COLUMN_TREE: u32      = 0xFFC9D1D9;  // Navigation tree text
const COLUMN_TREE_ICON: u32 = 0xFFFFA657;  // Tree icons (orange)
const COLUMN_STATUS: u32    = 0xFF8B949E;  // Status bar text
const COLUMN_FUNC: u32      = 0xFFFF7B72;  // Function names (red)
const COLUMN_CALL: u32      = 0xFF79C0FF;  // Call targets (blue)
const COLUMN_JUMP: u32      = 0xFFFFA657;  // Jump targets (orange)
const COLUMN_SEPARATOR: u32 = 0xFF30363D;  // Panel separators
const COLUMN_XREF: u32      = 0xFFD2A8FF;  // Cross-references (purple)

// ──── Panel & View ─────────────────────────────────────────────────────────

/// Which panel is currently focused
#[derive(Clone, Copy, PartialEq)]
// Enumeration — a type that can be one of several variants.
pub enum ActivePanel {
    Navigation,
    HexView,
    Disassembly,
    Information,
}

/// Navigation tree item types
#[derive(Clone, Copy, PartialEq)]
// Enumeration — a type that can be one of several variants.
pub enum NavItem {
    Header,           // ELF Header
    ProgramHeaders,   // Program headers group
    ProgramHeader(usize),
    SectionHeaders,   // Sections group
    Section(usize),
    Symbols,          // Symbols group
    Symbol(usize),
    Functions,        // Detected functions group
    Function(usize),
    Strings,          // Strings group
    StringItem(usize),
    DynamicInfo,      // Dynamic linking info
    Imports,          // Imports group
    Relocations,      // Relocations group
}

// ──── State ────────────────────────────────────────────────────────────────

/// Complete state for a binary viewer window
pub struct BinaryViewerState {
    /// The analyzed binary
    pub analysis: BinaryFile,
    /// Active panel
    pub active_panel: ActivePanel,
    
    // Navigation tree
    pub nav_items: Vec<(NavItem, u8, String)>,  // (item, indent_level, display_text)
    pub nav_scroll: usize,
    pub nav_selected: usize,
    pub nav_expanded: [bool; 8],  // Which groups are expanded

    // Hex view
    pub hex_offset: usize,       // Current file offset (top of view)
    pub hex_selected: usize,     // Selected byte offset
    pub hex_cursor: usize,       // Cursor position

    // Disassembly view
    pub disasm_index: usize,     // Index into instructions array (top of view)
    pub disasm_selected: usize,  // Selected instruction index
    
    // Info panel
    pub information_scroll: usize,
    pub information_lines: Vec<String>,
    
    // Current address for syncing panels
    pub current_address: u64,
    
    // File path for title
    pub file_path: String,
    
    // Search
    pub search_active: bool,
    pub search_query: String,
    pub search_results: Vec<u64>,
    pub search_result_index: usize,
}

// Implementation block — defines methods for the type above.
impl BinaryViewerState {
    /// Create a new viewer state from an analyzed binary
    pub fn new(analysis: BinaryFile, path: &str) -> Self {
        let mut state = BinaryViewerState {
            analysis,
            active_panel: ActivePanel::Disassembly,
            nav_items: Vec::new(),
            nav_scroll: 0,
            nav_selected: 0,
            nav_expanded: [true, false, true, false, true, false, false, false], // Header, Programs, Sections, Symbols, Functions, Strings, Dynamic, Relocs
            hex_offset: 0,
            hex_selected: 0,
            hex_cursor: 0,
            disasm_index: 0,
            disasm_selected: 0,
            information_scroll: 0,
            information_lines: Vec::new(),
            current_address: 0,
            file_path: String::from(path),
            search_active: false,
            search_query: String::new(),
            search_results: Vec::new(),
            search_result_index: 0,
        };
        
        // Set initial address to entry point
        state.current_address = state.analysis.elf.information.entry;
        
        // Find the entry point in instructions
        if let Some(index) = state.analysis.instructions.iter().position(|i| i.address == state.current_address) {
            state.disasm_selected = index;
            state.disasm_index = index.saturating_sub(5); // Show a few lines before
        }
        
        // Sync hex to entry point
        if let Some(offset) = state.analysis.vaddr_to_offset(state.current_address) {
            state.hex_offset = (offset as usize) & !0xF; // Align to 16
            state.hex_selected = offset as usize;
        }
        
        // Build navigation tree
        state.rebuild_nav_tree();
        
        // Build initial info for entry point
        state.update_information_panel();
        
        state
    }

    /// Rebuild the navigation tree based on expanded state
    pub fn rebuild_nav_tree(&mut self) {
        self.nav_items.clear();
        
        // ELF Header
        self.nav_items.push((NavItem::Header, 0, format!(
            "[H] ELF Header — {} {} {}",
            self.analysis.elf.information.elf_type,
            self.analysis.elf.information.machine,
            self.analysis.elf.information.class
        )));
        
        // Program Headers
        let ph_count = self.analysis.elf.programs.len();
        let ph_icon = if self.nav_expanded[1] { "[-]" } else { "[+]" };
        self.nav_items.push((NavItem::ProgramHeaders, 0, format!(
            "{} Program Headers ({})", ph_icon, ph_count
        )));
        if self.nav_expanded[1] {
            for (i, ph) in self.analysis.elf.programs.iter().enumerate() {
                self.nav_items.push((NavItem::ProgramHeader(i), 1, format!(
                    "  {:6} 0x{:08X} {:>6} {}",
                    ph.type_name(),
                    ph.vaddr,
                    ph.memsz,
                    ph.flags_string(),
                )));
            }
        }
        
        // Section Headers
        let sh_count = self.analysis.elf.sections.len();
        let sh_icon = if self.nav_expanded[2] { "[-]" } else { "[+]" };
        self.nav_items.push((NavItem::SectionHeaders, 0, format!(
            "{} Sections ({})", sh_icon, sh_count
        )));
        if self.nav_expanded[2] {
            for (i, sector) in self.analysis.elf.sections.iter().enumerate() {
                let name = if sector.name.is_empty() { "(null)" } else { &sector.name };
                self.nav_items.push((NavItem::Section(i), 1, format!(
                    "  {:<16} {:8} 0x{:08X} {:>6}",
                    name,
                    sector.type_name(),
                    sector.address,
                    sector.size,
                )));
            }
        }
        
        // Symbols (merged symtab + dynsym)
        let sym_count = self.analysis.elf.symbols.len() + self.analysis.elf.dynamic_symbols.len();
        let sym_icon = if self.nav_expanded[3] { "[-]" } else { "[+]" };
        self.nav_items.push((NavItem::Symbols, 0, format!(
            "{} Symbols ({})", sym_icon, sym_count
        )));
        if self.nav_expanded[3] {
            // Show functions first, then objects, limit to 200
            let mut syms: Vec<(usize, &Symbol)> = self.analysis.elf.symbols.iter()
                .chain(self.analysis.elf.dynamic_symbols.iter())
                .enumerate()
                .filter(|(_, s)| !s.name.is_empty() && s.value != 0)
                .collect();
            syms.sort_by_key(|(_, s)| s.value);
            for (i, sym) in syms.iter().take(200) {
                let icon = // Pattern matching — Rust's exhaustive branching construct.
match sym.sym_type {
                    2 => "fn",  // STT_FUNC
                    1 => "obj", // STT_OBJECT
                    _ => "  ",
                };
                self.nav_items.push((NavItem::Symbol(*i), 1, format!(
                    "  {} {:<24} 0x{:08X} {}",
                    icon, 
                    if sym.name.len() > 24 { &sym.name[..24] } else { &sym.name },
                    sym.value,
                    sym.binding_name(),
                )));
            }
        }
        
        // Detected Functions
        let func_count = self.analysis.xrefs.functions.len();
        let fn_icon = if self.nav_expanded[4] { "[-]" } else { "[+]" };
        self.nav_items.push((NavItem::Functions, 0, format!(
            "{} Functions ({})", fn_icon, func_count
        )));
        if self.nav_expanded[4] {
            for (i, func) in self.analysis.xrefs.functions.iter().enumerate().take(200) {
                let name = if func.name.is_empty() {
                    format!("sub_{:X}", func.entry)
                } else {
                    func.name.clone()
                };
                self.nav_items.push((NavItem::Function(i), 1, format!(
                    "  fn {:<24} 0x{:08X} ({} insns)",
                    if name.len() > 24 { &name[..24] } else { &name },
                    func.entry,
                    func.instruction_count,
                )));
            }
        }
        
        // Strings
        let str_count = self.analysis.elf.strings.len();
        let str_icon = if self.nav_expanded[5] { "[-]" } else { "[+]" };
        self.nav_items.push((NavItem::Strings, 0, format!(
            "{} Strings ({})", str_icon, str_count
        )));
        if self.nav_expanded[5] {
            for (i, s) in self.analysis.elf.strings.iter().enumerate().take(100) {
                let display = if s.content.len() > 30 {
                    format!("\"{}...\"", &s.content[..30])
                } else {
                    format!("\"{}\"", s.content)
                };
                let vaddr_value = s.vaddr.unwrap_or(0);
                self.nav_items.push((NavItem::StringItem(i), 1, format!(
                    "  0x{:08X} {}", vaddr_value, display
                )));
            }
        }
        
        // Dynamic Info
        if !self.analysis.elf.needed_libs.is_empty() || !self.analysis.elf.dynamic.is_empty() {
            let dyn_icon = if self.nav_expanded[6] { "[-]" } else { "[+]" };
            self.nav_items.push((NavItem::DynamicInfo, 0, format!(
                "{} Dynamic Linking", dyn_icon
            )));
            if self.nav_expanded[6] {
                for lib in &self.analysis.elf.needed_libs {
                    self.nav_items.push((NavItem::Imports, 1, format!("  NEEDED: {}", lib)));
                }
                if let Some(interp) = &self.analysis.elf.interpreter {
                    self.nav_items.push((NavItem::Imports, 1, format!("  INTERP: {}", interp)));
                }
            }
        }
        
        // Relocations
        if !self.analysis.elf.relocations.is_empty() {
            let relative_count = self.analysis.elf.relocations.len();
            let relative_icon = if self.nav_expanded[7] { "[-]" } else { "[+]" };
            self.nav_items.push((NavItem::Relocations, 0, format!(
                "{} Relocations ({})", relative_icon, relative_count
            )));
            if self.nav_expanded[7] {
                for reloc in self.analysis.elf.relocations.iter().take(100) {
                    let sym_name = if reloc.sym_name.is_empty() { "-" } else { &reloc.sym_name };
                    self.nav_items.push((NavItem::Relocations, 1, format!(
                        "  0x{:08X} {} + 0x{:X}",
                        reloc.offset, sym_name, reloc.addend
                    )));
                }
            }
        }
    }

    /// Navigate to a virtual address — syncs all panels
    pub fn goto_address(&mut self, address: u64) {
        self.current_address = address;
        
        // Sync disassembly
        if let Some(index) = self.analysis.instructions.iter().position(|i| i.address >= address) {
            self.disasm_selected = index;
            self.disasm_index = index.saturating_sub(5);
        }
        
        // Sync hex view
        if let Some(offset) = self.analysis.vaddr_to_offset(address) {
            let off = offset as usize;
            self.hex_selected = off;
            self.hex_offset = off & !0xF;
        }
        
        // Update info panel
        self.update_information_panel();
    }

    /// Update the info/xrefs panel for current address
    pub fn update_information_panel(&mut self) {
        self.information_lines.clear();
        let address = self.current_address;
        
        // Address
        self.information_lines.push(format!("Address: 0x{:016X}", address));
        self.information_lines.push(String::new());
        
        // Section info
        if let Some(sector) = self.analysis.elf.section_for_address(address) {
            self.information_lines.push(format!("Section: {} [{}]", sector.name, sector.type_name()));
            self.information_lines.push(format!("  Range: 0x{:X}..0x{:X}", sector.address, sector.address + sector.size));
            self.information_lines.push(format!("  Flags: {}", sector.flags_string()));
        }
        
        // Symbol
        if let Some(sym_name) = self.analysis.elf.address_to_symbol.get(&address) {
            self.information_lines.push(String::new());
            self.information_lines.push(format!("Symbol: {}", sym_name));
        }
        
        // Function info
        if let Some(func) = self.analysis.xrefs.function_at(address) {
            self.information_lines.push(String::new());
            let name = if func.name.is_empty() {
                format!("sub_{:X}", func.entry)
            } else {
                func.name.clone()
            };
            self.information_lines.push(format!("Function: {}", name));
            self.information_lines.push(format!("  Entry: 0x{:X}", func.entry));
            self.information_lines.push(format!("  End:   0x{:X}", func.end));
            self.information_lines.push(format!("  Instructions: {}", func.instruction_count));
            self.information_lines.push(format!("  Basic blocks: {}", func.basic_blocks));
            
            if !func.calls_to.is_empty() {
                self.information_lines.push(String::new());
                self.information_lines.push(String::from("Calls to:"));
                for target in &func.calls_to {
                    let name = self.analysis.elf.address_to_symbol.get(target)
                        .cloned()
                        .unwrap_or_else(|| format!("0x{:X}", target));
                    self.information_lines.push(format!("  -> {}", name));
                }
            }
            if !func.called_from.is_empty() {
                self.information_lines.push(String::new());
                self.information_lines.push(String::from("Called from:"));
                for caller in &func.called_from {
                    let name = self.analysis.elf.address_to_symbol.get(caller)
                        .cloned()
                        .unwrap_or_else(|| format!("0x{:X}", caller));
                    self.information_lines.push(format!("  <- {}", name));
                }
            }
        }
        
        // Cross-references to this address
        let xrefs_to = self.analysis.xrefs.xrefs_to(address);
        if !xrefs_to.is_empty() {
            self.information_lines.push(String::new());
            self.information_lines.push(format!("Xrefs TO 0x{:X} ({}):", address, xrefs_to.len()));
            for xref in xrefs_to.iter().take(20) {
                let type_str = // Pattern matching — Rust's exhaustive branching construct.
match xref.xref_type {
                    XrefType::Call => "CALL",
                    XrefType::Jump => "JMP ",
                    XrefType::ConditionalJump => "Jcc ",
                    XrefType::DataRef => "DATA",
                };
                self.information_lines.push(format!("  {} from 0x{:X}", type_str, xref.from));
            }
        }
        
        // Cross-references from this address
        let xrefs_from = self.analysis.xrefs.xrefs_from(address);
        if !xrefs_from.is_empty() {
            self.information_lines.push(String::new());
            self.information_lines.push(format!("Xrefs FROM 0x{:X} ({}):", address, xrefs_from.len()));
            for xref in xrefs_from.iter().take(20) {
                let type_str = // Pattern matching — Rust's exhaustive branching construct.
match xref.xref_type {
                    XrefType::Call => "CALL",
                    XrefType::Jump => "JMP ",
                    XrefType::ConditionalJump => "Jcc ",
                    XrefType::DataRef => "DATA",
                };
                let target_name = self.analysis.elf.address_to_symbol.get(&xref.to)
                    .cloned()
                    .unwrap_or_else(|| format!("0x{:X}", xref.to));
                self.information_lines.push(format!("  {} -> {}", type_str, target_name));
            }
        }
        
        // Instruction details
        if let Some(inst) = self.analysis.instruction_at(address) {
            self.information_lines.push(String::new());
            self.information_lines.push(String::from("Instruction:"));
            self.information_lines.push(format!("  {} {}", inst.mnemonic, inst.operands_str));
            let hex_bytes: Vec<String> = inst.bytes.iter().map(|b| format!("{:02X}", b)).collect();
            self.information_lines.push(format!("  Bytes: {}", hex_bytes.join(" ")));
            self.information_lines.push(format!("  Size: {} bytes", inst.bytes.len()));
            if let Some(ref comment) = inst.comment {
                self.information_lines.push(format!("  Note: {}", comment));
            }
            if let Some(target) = inst.branch_target {
                self.information_lines.push(format!("  Target: 0x{:X}", target));
            }
        }
    }

    /// Handle keyboard input
    pub fn handle_key(&mut self, key: char) {
                // Pattern matching — Rust's exhaustive branching construct.
match key {
            // Tab: cycle panels
            '\t' => {
                self.active_panel = // Pattern matching — Rust's exhaustive branching construct.
match self.active_panel {
                    ActivePanel::Navigation => ActivePanel::HexView,
                    ActivePanel::HexView => ActivePanel::Disassembly,
                    ActivePanel::Disassembly => ActivePanel::Information,
                    ActivePanel::Information => ActivePanel::Navigation,
                };
            },
            // Arrow keys (mapped from scancode)
            'U' => self.scroll_up(),   // Up
            'D' => self.scroll_down(), // Down
            'L' => self.scroll_left(), // Left (page up)
            'R' => self.scroll_right(), // Right (page down) 
            // Enter: activate nav item / follow branch
            '\n' | '\r' => self.on_enter(),
            // 'g': go to address (toggle search)
            'g' | 'G' => {
                self.search_active = !self.search_active;
                if self.search_active {
                    self.search_query.clear();
                }
            },
            // Number/hex input for search
            '0'..='9' | 'a'..='f' | 'A'..='F' if self.search_active => {
                self.search_query.push(key);
            },
            // Backspace in search
            '\x08' if self.search_active => {
                self.search_query.pop();
            },
            // 'x': follow xref
            'x' | 'X' => self.follow_xref(),
            _ => {}
        }
    }
    
    /// Handle special keys (scancode-based)  
    pub fn handle_scancode(&mut self, scancode: u8) {
                // Pattern matching — Rust's exhaustive branching construct.
match scancode {
            0x48 => self.scroll_up(),    // Up arrow
            0x50 => self.scroll_down(),  // Down arrow
            0x4B => self.scroll_left(),  // Left = page up
            0x4D => self.scroll_right(), // Right = page down
            0x49 => {                    // Page Up
                for _ in 0..20 { self.scroll_up(); }
            },
            0x51 => {                    // Page Down
                for _ in 0..20 { self.scroll_down(); }
            },
            0x47 => self.go_to_start(), // Home
            0x4F => self.go_to_end(),   // End
            0x0F => {                    // Tab
                self.active_panel = // Pattern matching — Rust's exhaustive branching construct.
match self.active_panel {
                    ActivePanel::Navigation => ActivePanel::HexView,
                    ActivePanel::HexView => ActivePanel::Disassembly,
                    ActivePanel::Disassembly => ActivePanel::Information,
                    ActivePanel::Information => ActivePanel::Navigation,
                };
            },
            0x1C => self.on_enter(),     // Enter
            _ => {}
        }
    }

    fn scroll_up(&mut self) {
                // Pattern matching — Rust's exhaustive branching construct.
match self.active_panel {
            ActivePanel::Navigation => {
                if self.nav_selected > 0 {
                    self.nav_selected -= 1;
                    if self.nav_selected < self.nav_scroll {
                        self.nav_scroll = self.nav_selected;
                    }
                }
            },
            ActivePanel::HexView => {
                if self.hex_offset >= 16 {
                    self.hex_offset -= 16;
                }
                if self.hex_selected >= 16 {
                    self.hex_selected -= 16;
                }
            },
            ActivePanel::Disassembly => {
                if self.disasm_selected > 0 {
                    self.disasm_selected -= 1;
                    if self.disasm_selected < self.disasm_index {
                        self.disasm_index = self.disasm_selected;
                    }
                    // Sync current address
                    if let Some(inst) = self.analysis.instructions.get(self.disasm_selected) {
                        self.current_address = inst.address;
                        self.update_information_panel();
                    }
                }
            },
            ActivePanel::Information => {
                if self.information_scroll > 0 {
                    self.information_scroll -= 1;
                }
            },
        }
    }

    fn scroll_down(&mut self) {
                // Pattern matching — Rust's exhaustive branching construct.
match self.active_panel {
            ActivePanel::Navigation => {
                if self.nav_selected + 1 < self.nav_items.len() {
                    self.nav_selected += 1;
                }
            },
            ActivePanel::HexView => {
                if self.hex_offset + 16 < self.analysis.data.len() {
                    self.hex_offset += 16;
                }
                self.hex_selected += 16;
                if self.hex_selected >= self.analysis.data.len() {
                    self.hex_selected = self.analysis.data.len().saturating_sub(1);
                }
            },
            ActivePanel::Disassembly => {
                if self.disasm_selected + 1 < self.analysis.instructions.len() {
                    self.disasm_selected += 1;
                    // Sync current address
                    if let Some(inst) = self.analysis.instructions.get(self.disasm_selected) {
                        self.current_address = inst.address;
                        self.update_information_panel();
                    }
                }
            },
            ActivePanel::Information => {
                if self.information_scroll + 1 < self.information_lines.len() {
                    self.information_scroll += 1;
                }
            },
        }
    }

    fn scroll_left(&mut self) {
        // Page up equivalent
        for _ in 0..10 { self.scroll_up(); }
    }

    fn scroll_right(&mut self) {
        // Page down equivalent
        for _ in 0..10 { self.scroll_down(); }
    }

    fn go_to_start(&mut self) {
                // Pattern matching — Rust's exhaustive branching construct.
match self.active_panel {
            ActivePanel::Navigation => { self.nav_selected = 0; self.nav_scroll = 0; },
            ActivePanel::HexView => { self.hex_offset = 0; self.hex_selected = 0; },
            ActivePanel::Disassembly => { self.disasm_index = 0; self.disasm_selected = 0; },
            ActivePanel::Information => { self.information_scroll = 0; },
        }
    }

    fn go_to_end(&mut self) {
                // Pattern matching — Rust's exhaustive branching construct.
match self.active_panel {
            ActivePanel::Navigation => {
                self.nav_selected = self.nav_items.len().saturating_sub(1);
            },
            ActivePanel::HexView => {
                let last = self.analysis.data.len().saturating_sub(16);
                self.hex_offset = last & !0xF;
                self.hex_selected = last;
            },
            ActivePanel::Disassembly => {
                self.disasm_selected = self.analysis.instructions.len().saturating_sub(1);
            },
            ActivePanel::Information => {
                self.information_scroll = self.information_lines.len().saturating_sub(1);
            },
        }
    }

    fn on_enter(&mut self) {
                // Pattern matching — Rust's exhaustive branching construct.
match self.active_panel {
            ActivePanel::Navigation => {
                if let Some((item, _, _)) = self.nav_items.get(self.nav_selected) {
                                        // Pattern matching — Rust's exhaustive branching construct.
match *item {
                        NavItem::Header => {
                            // Show ELF header info
                            self.goto_address(self.analysis.elf.information.entry);
                        },
                        NavItem::ProgramHeaders => {
                            self.nav_expanded[1] = !self.nav_expanded[1];
                            self.rebuild_nav_tree();
                        },
                        NavItem::ProgramHeader(i) => {
                            if let Some(ph) = self.analysis.elf.programs.get(i) {
                                self.goto_address(ph.vaddr);
                            }
                        },
                        NavItem::SectionHeaders => {
                            self.nav_expanded[2] = !self.nav_expanded[2];
                            self.rebuild_nav_tree();
                        },
                        NavItem::Section(i) => {
                            if let Some(sector) = self.analysis.elf.sections.get(i) {
                                if sector.address != 0 {
                                    self.goto_address(sector.address);
                                } else {
                                    // Non-loaded section, navigate hex to file offset
                                    self.hex_offset = sector.offset as usize & !0xF;
                                    self.hex_selected = sector.offset as usize;
                                    self.active_panel = ActivePanel::HexView;
                                }
                            }
                        },
                        NavItem::Symbols => {
                            self.nav_expanded[3] = !self.nav_expanded[3];
                            self.rebuild_nav_tree();
                        },
                        NavItem::Symbol(i) => {
                            let all_syms: Vec<&Symbol> = self.analysis.elf.symbols.iter()
                                .chain(self.analysis.elf.dynamic_symbols.iter())
                                .filter(|s| !s.name.is_empty() && s.value != 0)
                                .collect();
                            if let Some(sym) = all_syms.get(i) {
                                self.goto_address(sym.value);
                            }
                        },
                        NavItem::Functions => {
                            self.nav_expanded[4] = !self.nav_expanded[4];
                            self.rebuild_nav_tree();
                        },
                        NavItem::Function(i) => {
                            if let Some(func) = self.analysis.xrefs.functions.get(i) {
                                self.goto_address(func.entry);
                            }
                        },
                        NavItem::Strings => {
                            self.nav_expanded[5] = !self.nav_expanded[5];
                            self.rebuild_nav_tree();
                        },
                        NavItem::StringItem(i) => {
                            if let Some(s) = self.analysis.elf.strings.get(i) {
                                if let Some(vaddr) = s.vaddr {
                                    self.goto_address(vaddr);
                                }
                            }
                        },
                        NavItem::DynamicInfo => {
                            self.nav_expanded[6] = !self.nav_expanded[6];
                            self.rebuild_nav_tree();
                        },
                        NavItem::Relocations => {
                            self.nav_expanded[7] = !self.nav_expanded[7];
                            self.rebuild_nav_tree();
                        },
                        NavItem::Imports => {},
                    }
                }
            },
            ActivePanel::Disassembly => {
                // Follow branch/call target
                if let Some(inst) = self.analysis.instructions.get(self.disasm_selected) {
                    if let Some(target) = inst.branch_target {
                        self.goto_address(target);
                    }
                }
            },
            ActivePanel::HexView => {
                // Enter in hex view: try to interpret as address and navigate
                if let Some(vaddr) = self.analysis.offset_to_vaddr(self.hex_selected as u64) {
                    self.goto_address(vaddr);
                    self.active_panel = ActivePanel::Disassembly;
                }
            },
            ActivePanel::Information => {},
        }

        // If search is active and we press enter, execute search
        if self.search_active && !self.search_query.is_empty() {
            if let Ok(address) = u64::from_str_radix(&self.search_query, 16) {
                self.goto_address(address);
                self.search_active = false;
            }
        }
    }

    fn follow_xref(&mut self) {
        // Follow first xref from current address
        let xrefs = self.analysis.xrefs.xrefs_from(self.current_address);
        if let Some(xref) = xrefs.first() {
            self.goto_address(xref.to);
        }
    }

    /// Handle mouse click at window-relative coordinates
    pub fn handle_click(&mut self, relative_x: i32, relative_y: i32, win_w: u32, win_h: u32) {
        let content_h = win_h.saturating_sub(TITLE_BAR_HEIGHT + 24) as i32; // -24 for status bar
        let nav_w = (win_w as i32 * 25) / 100; // 25% for nav
        let hex_w = (win_w as i32 * 25) / 100; // 25% for hex
        let disasm_w = (win_w as i32 * 30) / 100; // 30% for disasm
        // remaining 20% for info

        let header_h = 20i32;
        let status_h = 20i32;
        let content_start_y = (TITLE_BAR_HEIGHT as i32) + header_h;

        // Determine which panel was clicked
        if relative_y < content_start_y || relative_y > win_h as i32 - status_h {
            return; // Header or status bar
        }

        let line_h = 14i32;
        let line_index = ((relative_y - content_start_y) / line_h) as usize;

        if relative_x < nav_w {
            // Navigation panel
            self.active_panel = ActivePanel::Navigation;
            let clicked = self.nav_scroll + line_index;
            if clicked < self.nav_items.len() {
                self.nav_selected = clicked;
                self.on_enter(); // Auto-activate
            }
        } else if relative_x < nav_w + hex_w {
            // Hex panel
            self.active_panel = ActivePanel::HexView;
            let clicked_offset = self.hex_offset + line_index * 16;
            if clicked_offset < self.analysis.data.len() {
                self.hex_selected = clicked_offset;
                if let Some(vaddr) = self.analysis.offset_to_vaddr(clicked_offset as u64) {
                    self.current_address = vaddr;
                    self.update_information_panel();
                }
            }
        } else if relative_x < nav_w + hex_w + disasm_w {
            // Disassembly panel
            self.active_panel = ActivePanel::Disassembly;
            let clicked_inst = self.disasm_index + line_index;
            if clicked_inst < self.analysis.instructions.len() {
                self.disasm_selected = clicked_inst;
                self.current_address = self.analysis.instructions[clicked_inst].address;
                self.update_information_panel();
            }
        } else {
            // Info panel
            self.active_panel = ActivePanel::Information;
        }
    }
}

// ──── Drawing ──────────────────────────────────────────────────────────────

/// Draw the complete binary viewer UI  
/// Called from desktop.rs draw_window_content dispatch
pub fn draw_binary_viewer(
    state: &BinaryViewerState,
    wx: i32, wy: i32, ww: u32, wh: u32,
    draw_text_fn: &dyn Fn(i32, i32, &str, u32),
) {
    let title_h = TITLE_BAR_HEIGHT as i32;
    let status_h = 20i32;
    let header_h = 20i32;
    
    let content_x = wx + 1;
    let content_y = wy + title_h;
    let content_w = ww.saturating_sub(2) as i32;
    let content_h = (wh as i32) - title_h - status_h;
    
    if content_w < 200 || content_h < 100 {
        return;
    }

    // Panel widths (percentage-based)
    let nav_w = (content_w * 25) / 100;
    let hex_w = (content_w * 25) / 100;
    let disasm_w = (content_w * 30) / 100;
    let information_w = content_w - nav_w - hex_w - disasm_w;

    let nav_x = content_x;
    let hex_x = nav_x + nav_w;
    let disasm_x = hex_x + hex_w;
    let information_x = disasm_x + disasm_w;

    // Background
    crate::framebuffer::fill_rect(wx as u32, (wy + title_h) as u32, ww, wh - title_h as u32, BG_DARK);

    // ── Panel Headers ──
    let headers = [
        (nav_x, nav_w, "Navigation", ActivePanel::Navigation),
        (hex_x, hex_w, "Hex View", ActivePanel::HexView),
        (disasm_x, disasm_w, "Disassembly", ActivePanel::Disassembly),
        (information_x, information_w, "Info / Xrefs", ActivePanel::Information),
    ];

    for (pixel, pw, label, panel) in &headers {
        let bg = if *panel == state.active_panel { 0xFF1F6FEB } else { BG_HEADER };
        crate::framebuffer::fill_rect(*pixel as u32, content_y as u32, *pw as u32, header_h as u32, bg);
        draw_text_fn(*pixel + 4, content_y + 3, label, COLUMN_HEADER);
    }

    // ── Panel Separators ──
    for pixel in &[hex_x, disasm_x, information_x] {
        crate::framebuffer::fill_rect(*pixel as u32, (content_y + header_h) as u32, 1, content_h as u32, COLUMN_SEPARATOR);
    }

    let panel_y = content_y + header_h;
    let panel_h = content_h - header_h;
    let line_h = 14i32;
    let visible_lines = (panel_h / line_h) as usize;

    // ── Navigation Panel ──
    draw_nav_panel(state, nav_x, panel_y, nav_w, visible_lines, line_h, draw_text_fn);

    // ── Hex View Panel ──
    draw_hex_panel(state, hex_x + 2, panel_y, hex_w - 4, visible_lines, line_h, draw_text_fn);

    // ── Disassembly Panel ──
    draw_disasm_panel(state, disasm_x + 2, panel_y, disasm_w - 4, visible_lines, line_h, draw_text_fn);

    // ── Info Panel ──
    draw_information_panel(state, information_x + 2, panel_y, information_w - 4, visible_lines, line_h, draw_text_fn);

    // ── Status Bar ──
    let status_y = wy + wh as i32 - status_h;
    crate::framebuffer::fill_rect(wx as u32, status_y as u32, ww, status_h as u32, BG_HEADER);

    // Left: file info
    let summary = format!(
        " {} | {} | {} insns | {} syms | {} funcs",
        state.file_path,
        state.analysis.elf.information.elf_type,
        state.analysis.instructions.len(),
        state.analysis.elf.symbols.len() + state.analysis.elf.dynamic_symbols.len(),
        state.analysis.xrefs.functions.len(),
    );
    draw_text_fn(wx + 4, status_y + 3, &summary, COLUMN_STATUS);

    // Right: current address
    let address_str = format!("0x{:016X} ", state.current_address);
    let address_x = wx + ww as i32 - (address_str.len() as i32 * 8) - 4;
    draw_text_fn(address_x, status_y + 3, &address_str, COLUMN_ADDRESS);

    // Search overlay
    if state.search_active {
        let search_y = wy + title_h + 2;
        let software = 250i32;
        let sh = 20i32;
        let sx = wx + ww as i32 / 2 - software / 2;
        crate::framebuffer::fill_rect(sx as u32, search_y as u32, software as u32, sh as u32, 0xFF1F6FEB);
        let prompt = format!("Go to: 0x{}_", state.search_query);
        draw_text_fn(sx + 4, search_y + 3, &prompt, 0xFFFFFFFF);
    }
}

/// Draw the navigation tree panel
fn draw_nav_panel(
    state: &BinaryViewerState,
    x: i32, y: i32, w: i32,
    visible: usize, line_h: i32,
    draw_text_fn: &dyn Fn(i32, i32, &str, u32),
) {
    let start = state.nav_scroll;
    let end = (start + visible).minimum(state.nav_items.len());

    for (vi, index) in (start..end).enumerate() {
        let ly = y + (vi as i32) * line_h;
        let (item, indent, text) = &state.nav_items[index];

        // Selection highlight
        if index == state.nav_selected {
            let bg = if state.active_panel == ActivePanel::Navigation { BG_SELECTED } else { BG_HOVER };
            crate::framebuffer::fill_rect(x as u32, ly as u32, w as u32, line_h as u32, bg);
        }

        // Determine color
        let color = // Pattern matching — Rust's exhaustive branching construct.
match item {
            NavItem::Header | NavItem::ProgramHeaders | NavItem::SectionHeaders |
            NavItem::Symbols | NavItem::Functions | NavItem::Strings |
            NavItem::DynamicInfo | NavItem::Relocations => COLUMN_TREE_ICON,
            NavItem::Function(_) => COLUMN_FUNC,
            NavItem::Symbol(_) => COLUMN_REGISTER,
            NavItem::StringItem(_) => COLUMN_STRING,
            _ => COLUMN_TREE,
        };

        // Truncate text to panel width
        let maximum_chars = (w / 8).maximum(4) as usize;
        let display = if text.len() > maximum_chars {
            &text[..maximum_chars]
        } else {
            text
        };

        draw_text_fn(x + 2 + (*indent as i32 * 8), ly + 1, display, color);
    }
}

/// Draw the hex view panel
fn draw_hex_panel(
    state: &BinaryViewerState,
    x: i32, y: i32, _w: i32,
    visible: usize, line_h: i32,
    draw_text_fn: &dyn Fn(i32, i32, &str, u32),
) {
    let data = &state.analysis.data;
    let start_offset = state.hex_offset;

    for line in 0..visible {
        let offset = start_offset + line * 16;
        if offset >= data.len() { break; }

        let ly = y + (line as i32) * line_h;
        let end = (offset + 16).minimum(data.len());
        let chunk = &data[offset..end];

        // Highlight selected line
        if offset <= state.hex_selected && state.hex_selected < offset + 16 {
            let bg = if state.active_panel == ActivePanel::HexView { BG_SELECTED } else { BG_HOVER };
            crate::framebuffer::fill_rect(x as u32, ly as u32, 400, line_h as u32, bg);
        }

        // Address
        let address_s = format!("{:06X}", offset);
        draw_text_fn(x, ly + 1, &address_s, COLUMN_ADDRESS);

        // Hex bytes (compact: 2 chars + space per byte)
        let mut hx = x + 56;
        for (i, &b) in chunk.iter().enumerate() {
            if i == 8 { hx += 4; } // Extra gap at midpoint
            let byte_s = format!("{:02X}", b);
            
            // Color based on value
            let column = if b == 0 {
                0xFF484F58 // Dim for zero
            } else if b >= 0x20 && b < 0x7F {
                COLUMN_HEX // Normal printable
            } else {
                COLUMN_IMMEDIATE // Non-printable highlighted
            };
            draw_text_fn(hx, ly + 1, &byte_s, column);
            hx += 20;
        }

        // ASCII
        let ascii_x = x + 56 + 16 * 20 + 12;
        let mut ax = ascii_x;
        for &b in chunk {
            let character = if b >= 0x20 && b < 0x7F { b as char } else { '.' };
            let mut buffer = [0u8; 4];
            let s = character.encode_utf8(&mut buffer);
            let column = if b >= 0x20 && b < 0x7F { COLUMN_ASCII } else { 0xFF484F58 };
            draw_text_fn(ax, ly + 1, s, column);
            ax += 8;
        }
    }
}

/// Draw the disassembly panel
fn draw_disasm_panel(
    state: &BinaryViewerState,
    x: i32, y: i32, w: i32,
    visible: usize, line_h: i32,
    draw_text_fn: &dyn Fn(i32, i32, &str, u32),
) {
    let insts = &state.analysis.instructions;
    if insts.is_empty() {
        draw_text_fn(x + 4, y + 20, "No code to display", COLUMN_COMMENT);
        return;
    }

    // Auto-scroll to keep selected visible
    let start = state.disasm_index;
    let end = (start + visible).minimum(insts.len());

    for (vi, index) in (start..end).enumerate() {
        let inst = &insts[index];
        let ly = y + (vi as i32) * line_h;

        // Selection highlight
        if index == state.disasm_selected {
            let bg = if state.active_panel == ActivePanel::Disassembly { BG_SELECTED } else { BG_HOVER };
            crate::framebuffer::fill_rect(x as u32, ly as u32, w as u32, line_h as u32, bg);
        }

        // Function label (if this is a function entry)
        if state.analysis.xrefs.is_function_entry(inst.address) {
            if let Some(name) = state.analysis.elf.address_to_symbol.get(&inst.address) {
                // Draw function label above instruction  
                let label = format!("<{}>:", name);
                let maximum_chars = (w / 8).maximum(4) as usize;
                let display = if label.len() > maximum_chars { &label[..maximum_chars] } else { &label };
                draw_text_fn(x + 2, ly + 1, display, COLUMN_LABEL);
                continue; // Use this line for the label
            }
        }

        let mut cx = x + 2;

        // Address
        let address_s = format!("{:08X}", inst.address);
        draw_text_fn(cx, ly + 1, &address_s, COLUMN_ADDRESS);
        cx += 72;

        // Bytes (compact, max 6 bytes shown)
        let byte_count = inst.bytes.len().minimum(6);
        let mut bytes_s = String::new();
        for b in &inst.bytes[..byte_count] {
            bytes_s.push_str(&format!("{:02X}", b));
        }
        if inst.bytes.len() > 6 { bytes_s.push_str(".."); }
        // Pad to fixed width
        while bytes_s.len() < 14 { bytes_s.push(' '); }
        draw_text_fn(cx, ly + 1, &bytes_s, 0xFF484F58);
        cx += 116;

        // Mnemonic
        let mnemonic_column = if inst.is_call {
            COLUMN_CALL
        } else if inst.is_jump || inst.is_condition_jump {
            COLUMN_JUMP
        } else if inst.is_return_value {
            COLUMN_FUNC
        } else {
            COLUMN_MNEMONIC
        };
        let mnem = format!("{:<7}", inst.mnemonic);
        draw_text_fn(cx, ly + 1, &mnem, mnemonic_column);
        cx += 60;

        // Operands (color based on content)
        let maximum_operation_chars = ((w - (cx - x)) / 8).maximum(1) as usize;
        let operands = if inst.operands_str.len() > maximum_operation_chars {
            &inst.operands_str[..maximum_operation_chars]
        } else {
            &inst.operands_str
        };

        // Simple operand coloring: detect registers vs immediates
        let operation_column = if inst.operands_str.starts_with("0x") || inst.operands_str.contains("0x") {
            COLUMN_IMMEDIATE
        } else {
            COLUMN_REGISTER
        };
        draw_text_fn(cx, ly + 1, operands, operation_column);

        // Comment (if any) at end of line
        if let Some(ref comment) = inst.comment {
            let comment_s = format!(" ; {}", comment);
            let rem_w = w - (cx - x) - (operands.len() as i32 * 8);
            if rem_w > 24 {
                let maximum_c = (rem_w / 8) as usize;
                let display = if comment_s.len() > maximum_c { &comment_s[..maximum_c] } else { &comment_s };
                draw_text_fn(cx + (operands.len() as i32 * 8), ly + 1, display, COLUMN_COMMENT);
            }
        }
    }
}

/// Draw the info/xrefs panel
fn draw_information_panel(
    state: &BinaryViewerState,
    x: i32, y: i32, _w: i32,
    visible: usize, line_h: i32,
    draw_text_fn: &dyn Fn(i32, i32, &str, u32),
) {
    let start = state.information_scroll;
    let end = (start + visible).minimum(state.information_lines.len());

    for (vi, index) in (start..end).enumerate() {
        let ly = y + (vi as i32) * line_h;
        let line = &state.information_lines[index];

        // Color based on content
        let column = if line.starts_with("Address:") || line.starts_with("Section:") ||
                     line.starts_with("Symbol:") || line.starts_with("Function:") ||
                     line.starts_with("Instruction:") {
            COLUMN_HEADER
        } else if line.starts_with("Xrefs") {
            COLUMN_XREF
        } else if line.starts_with("  ->") || line.starts_with("  <-") {
            COLUMN_CALL
        } else if line.starts_with("  CALL") || line.starts_with("  JMP") || line.starts_with("  Jcc") {
            COLUMN_JUMP
        } else if line.starts_with("  DATA") {
            COLUMN_STRING
        } else if line.starts_with("Calls to:") || line.starts_with("Called from:") {
            COLUMN_FUNC
        } else {
            COLUMN_TREE
        };

        // Truncate if needed
        let maximum_chars = 40usize; // Info panel is narrow
        let display = if line.len() > maximum_chars { &line[..maximum_chars] } else { line };
        draw_text_fn(x + 2, ly + 1, display, column);
    }
}
