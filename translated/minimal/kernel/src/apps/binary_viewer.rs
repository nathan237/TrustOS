





use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use alloc::collections::BTreeMap;

use crate::binary_analysis::{Hg, Ct, Cy, Bj, Hp, XrefType};


const J_: u32 = 28;



const DW_: u32       = 0xFF0D1117;  
const CJ_: u32      = 0xFF161B22;  
const DX_: u32     = 0xFF21262D;  
const AAW_: u32   = 0xFF1F3A5F;  
const AAV_: u32      = 0xFF1C2333;  
const DHG_: u32       = 0xFF0D1117;  

const IJ_: u32      = 0xFF8B949E;  
const BQN_: u32       = 0xFFC9D1D9;  
const BQH_: u32     = 0xFF7EE787;  
const BQP_: u32  = 0xFF79C0FF;  
const AQP_: u32  = 0xFFD2A8FF;  
const AQN_: u32 = 0xFFFFA657;  
const AQL_: u32   = 0xFF8B949E;  
const BQO_: u32     = 0xFFFF7B72;  
const AQQ_: u32    = 0xFF7EE787;  
const AQM_: u32    = 0xFFFFFFFF;  
const AQT_: u32      = 0xFFC9D1D9;  
const BQV_: u32 = 0xFFFFA657;  
const ABW_: u32    = 0xFF8B949E;  
const ABV_: u32      = 0xFFFF7B72;  
const AQK_: u32      = 0xFF79C0FF;  
const AQO_: u32      = 0xFFFFA657;  
const BQR_: u32 = 0xFF30363D;  
const BRB_: u32      = 0xFFD2A8FF;  




#[derive(Clone, Copy, PartialEq)]
pub enum ActivePanel {
    Navigation,
    HexView,
    Disassembly,
    Info,
}


#[derive(Clone, Copy, PartialEq)]
pub enum NavItem {
    Header,           
    ProgramHeaders,   
    Gz(usize),
    SectionHeaders,   
    Ct(usize),
    Symbols,          
    Cy(usize),
    Functions,        
    Aq(usize),
    Strings,          
    StringItem(usize),
    DynamicInfo,      
    Imports,          
    Relocations,      
}




pub struct BinaryViewerState {
    
    pub analysis: Hg,
    
    pub active_panel: ActivePanel,
    
    
    pub nav_items: Vec<(NavItem, u8, String)>,  
    pub nav_scroll: usize,
    pub nav_selected: usize,
    pub nav_expanded: [bool; 8],  

    
    pub hex_offset: usize,       
    pub hex_selected: usize,     
    pub hex_cursor: usize,       

    
    pub disasm_index: usize,     
    pub disasm_selected: usize,  
    
    
    pub info_scroll: usize,
    pub info_lines: Vec<String>,
    
    
    pub current_addr: u64,
    
    
    pub file_path: String,
    
    
    pub search_active: bool,
    pub search_query: String,
    pub search_results: Vec<u64>,
    pub search_result_idx: usize,
}

impl BinaryViewerState {
    
    pub fn new(analysis: Hg, path: &str) -> Self {
        let mut state = BinaryViewerState {
            analysis,
            active_panel: ActivePanel::Disassembly,
            nav_items: Vec::new(),
            nav_scroll: 0,
            nav_selected: 0,
            nav_expanded: [true, false, true, false, true, false, false, false], 
            hex_offset: 0,
            hex_selected: 0,
            hex_cursor: 0,
            disasm_index: 0,
            disasm_selected: 0,
            info_scroll: 0,
            info_lines: Vec::new(),
            current_addr: 0,
            file_path: String::from(path),
            search_active: false,
            search_query: String::new(),
            search_results: Vec::new(),
            search_result_idx: 0,
        };
        
        
        state.current_addr = state.analysis.elf.info.entry;
        
        
        if let Some(idx) = state.analysis.instructions.iter().position(|i| i.address == state.current_addr) {
            state.disasm_selected = idx;
            state.disasm_index = idx.saturating_sub(5); 
        }
        
        
        if let Some(offset) = state.analysis.vaddr_to_offset(state.current_addr) {
            state.hex_offset = (offset as usize) & !0xF; 
            state.hex_selected = offset as usize;
        }
        
        
        state.rebuild_nav_tree();
        
        
        state.update_info_panel();
        
        state
    }

    
    pub fn rebuild_nav_tree(&mut self) {
        self.nav_items.clear();
        
        
        self.nav_items.push((NavItem::Header, 0, format!(
            "[H] ELF Header — {} {} {}",
            self.analysis.elf.info.elf_type,
            self.analysis.elf.info.machine,
            self.analysis.elf.info.class
        )));
        
        
        let bur = self.analysis.elf.programs.len();
        let nty = if self.nav_expanded[1] { "[-]" } else { "[+]" };
        self.nav_items.push((NavItem::ProgramHeaders, 0, format!(
            "{} Program Headers ({})", nty, bur
        )));
        if self.nav_expanded[1] {
            for (i, qc) in self.analysis.elf.programs.iter().enumerate() {
                self.nav_items.push((NavItem::Gz(i), 1, format!(
                    "  {:6} 0x{:08X} {:>6} {}",
                    qc.type_name(),
                    qc.vaddr,
                    qc.memsz,
                    qc.flags_string(),
                )));
            }
        }
        
        
        let oqp = self.analysis.elf.sections.len();
        let oqq = if self.nav_expanded[2] { "[-]" } else { "[+]" };
        self.nav_items.push((NavItem::SectionHeaders, 0, format!(
            "{} Sections ({})", oqq, oqp
        )));
        if self.nav_expanded[2] {
            for (i, lx) in self.analysis.elf.sections.iter().enumerate() {
                let name = if lx.name.is_empty() { "(null)" } else { &lx.name };
                self.nav_items.push((NavItem::Ct(i), 1, format!(
                    "  {:<16} {:8} 0x{:08X} {:>6}",
                    name,
                    lx.type_name(),
                    lx.addr,
                    lx.size,
                )));
            }
        }
        
        
        let gwv = self.analysis.elf.symbols.len() + self.analysis.elf.dynamic_symbols.len();
        let ozj = if self.nav_expanded[3] { "[-]" } else { "[+]" };
        self.nav_items.push((NavItem::Symbols, 0, format!(
            "{} Symbols ({})", ozj, gwv
        )));
        if self.nav_expanded[3] {
            
            let mut jki: Vec<(usize, &Cy)> = self.analysis.elf.symbols.iter()
                .chain(self.analysis.elf.dynamic_symbols.iter())
                .enumerate()
                .filter(|(_, j)| !j.name.is_empty() && j.value != 0)
                .collect();
            jki.sort_by_key(|(_, j)| j.value);
            for (i, sym) in jki.iter().take(200) {
                let icon = match sym.sym_type {
                    2 => "fn",  
                    1 => "obj", 
                    _ => "  ",
                };
                self.nav_items.push((NavItem::Cy(*i), 1, format!(
                    "  {} {:<24} 0x{:08X} {}",
                    icon, 
                    if sym.name.len() > 24 { &sym.name[..24] } else { &sym.name },
                    sym.value,
                    sym.binding_name(),
                )));
            }
        }
        
        
        let mal = self.analysis.xrefs.functions.len();
        let lxf = if self.nav_expanded[4] { "[-]" } else { "[+]" };
        self.nav_items.push((NavItem::Functions, 0, format!(
            "{} Functions ({})", lxf, mal
        )));
        if self.nav_expanded[4] {
            for (i, func) in self.analysis.xrefs.functions.iter().enumerate().take(200) {
                let name = if func.name.is_empty() {
                    format!("sub_{:X}", func.entry)
                } else {
                    func.name.clone()
                };
                self.nav_items.push((NavItem::Aq(i), 1, format!(
                    "  fn {:<24} 0x{:08X} ({} insns)",
                    if name.len() > 24 { &name[..24] } else { &name },
                    func.entry,
                    func.instruction_count,
                )));
            }
        }
        
        
        let oxv = self.analysis.elf.strings.len();
        let oxw = if self.nav_expanded[5] { "[-]" } else { "[+]" };
        self.nav_items.push((NavItem::Strings, 0, format!(
            "{} Strings ({})", oxw, oxv
        )));
        if self.nav_expanded[5] {
            for (i, j) in self.analysis.elf.strings.iter().enumerate().take(100) {
                let display = if j.content.len() > 30 {
                    format!("\"{}...\"", &j.content[..30])
                } else {
                    format!("\"{}\"", j.content)
                };
                let pqy = j.vaddr.unwrap_or(0);
                self.nav_items.push((NavItem::StringItem(i), 1, format!(
                    "  0x{:08X} {}", pqy, display
                )));
            }
        }
        
        
        if !self.analysis.elf.needed_libs.is_empty() || !self.analysis.elf.dynamic.is_empty() {
            let lmz = if self.nav_expanded[6] { "[-]" } else { "[+]" };
            self.nav_items.push((NavItem::DynamicInfo, 0, format!(
                "{} Dynamic Linking", lmz
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
        
        
        if !self.analysis.elf.relocations.is_empty() {
            let oeo = self.analysis.elf.relocations.len();
            let oep = if self.nav_expanded[7] { "[-]" } else { "[+]" };
            self.nav_items.push((NavItem::Relocations, 0, format!(
                "{} Relocations ({})", oep, oeo
            )));
            if self.nav_expanded[7] {
                for bdg in self.analysis.elf.relocations.iter().take(100) {
                    let sym_name = if bdg.sym_name.is_empty() { "-" } else { &bdg.sym_name };
                    self.nav_items.push((NavItem::Relocations, 1, format!(
                        "  0x{:08X} {} + 0x{:X}",
                        bdg.offset, sym_name, bdg.addend
                    )));
                }
            }
        }
    }

    
    pub fn goto_address(&mut self, addr: u64) {
        self.current_addr = addr;
        
        
        if let Some(idx) = self.analysis.instructions.iter().position(|i| i.address >= addr) {
            self.disasm_selected = idx;
            self.disasm_index = idx.saturating_sub(5);
        }
        
        
        if let Some(offset) = self.analysis.vaddr_to_offset(addr) {
            let off = offset as usize;
            self.hex_selected = off;
            self.hex_offset = off & !0xF;
        }
        
        
        self.update_info_panel();
    }

    
    pub fn update_info_panel(&mut self) {
        self.info_lines.clear();
        let addr = self.current_addr;
        
        
        self.info_lines.push(format!("Address: 0x{:016X}", addr));
        self.info_lines.push(String::new());
        
        
        if let Some(lx) = self.analysis.elf.section_for_addr(addr) {
            self.info_lines.push(format!("Section: {} [{}]", lx.name, lx.type_name()));
            self.info_lines.push(format!("  Range: 0x{:X}..0x{:X}", lx.addr, lx.addr + lx.size));
            self.info_lines.push(format!("  Flags: {}", lx.flags_string()));
        }
        
        
        if let Some(sym_name) = self.analysis.elf.addr_to_symbol.get(&addr) {
            self.info_lines.push(String::new());
            self.info_lines.push(format!("Symbol: {}", sym_name));
        }
        
        
        if let Some(func) = self.analysis.xrefs.function_at(addr) {
            self.info_lines.push(String::new());
            let name = if func.name.is_empty() {
                format!("sub_{:X}", func.entry)
            } else {
                func.name.clone()
            };
            self.info_lines.push(format!("Function: {}", name));
            self.info_lines.push(format!("  Entry: 0x{:X}", func.entry));
            self.info_lines.push(format!("  End:   0x{:X}", func.end));
            self.info_lines.push(format!("  Instructions: {}", func.instruction_count));
            self.info_lines.push(format!("  Basic blocks: {}", func.basic_blocks));
            
            if !func.calls_to.is_empty() {
                self.info_lines.push(String::new());
                self.info_lines.push(String::from("Calls to:"));
                for target in &func.calls_to {
                    let name = self.analysis.elf.addr_to_symbol.get(target)
                        .cloned()
                        .unwrap_or_else(|| format!("0x{:X}", target));
                    self.info_lines.push(format!("  -> {}", name));
                }
            }
            if !func.called_from.is_empty() {
                self.info_lines.push(String::new());
                self.info_lines.push(String::from("Called from:"));
                for caller in &func.called_from {
                    let name = self.analysis.elf.addr_to_symbol.get(caller)
                        .cloned()
                        .unwrap_or_else(|| format!("0x{:X}", caller));
                    self.info_lines.push(format!("  <- {}", name));
                }
            }
        }
        
        
        let xrefs_to = self.analysis.xrefs.xrefs_to(addr);
        if !xrefs_to.is_empty() {
            self.info_lines.push(String::new());
            self.info_lines.push(format!("Xrefs TO 0x{:X} ({}):", addr, xrefs_to.len()));
            for aks in xrefs_to.iter().take(20) {
                let ws = match aks.xref_type {
                    XrefType::Call => "CALL",
                    XrefType::Jump => "JMP ",
                    XrefType::ConditionalJump => "Jcc ",
                    XrefType::DataRef => "DATA",
                };
                self.info_lines.push(format!("  {} from 0x{:X}", ws, aks.from));
            }
        }
        
        
        let xrefs_from = self.analysis.xrefs.xrefs_from(addr);
        if !xrefs_from.is_empty() {
            self.info_lines.push(String::new());
            self.info_lines.push(format!("Xrefs FROM 0x{:X} ({}):", addr, xrefs_from.len()));
            for aks in xrefs_from.iter().take(20) {
                let ws = match aks.xref_type {
                    XrefType::Call => "CALL",
                    XrefType::Jump => "JMP ",
                    XrefType::ConditionalJump => "Jcc ",
                    XrefType::DataRef => "DATA",
                };
                let cri = self.analysis.elf.addr_to_symbol.get(&aks.to)
                    .cloned()
                    .unwrap_or_else(|| format!("0x{:X}", aks.to));
                self.info_lines.push(format!("  {} -> {}", ws, cri));
            }
        }
        
        
        if let Some(inst) = self.analysis.instruction_at(addr) {
            self.info_lines.push(String::new());
            self.info_lines.push(String::from("Instruction:"));
            self.info_lines.push(format!("  {} {}", inst.mnemonic, inst.operands_str));
            let mla: Vec<String> = inst.bytes.iter().map(|b| format!("{:02X}", b)).collect();
            self.info_lines.push(format!("  Bytes: {}", mla.join(" ")));
            self.info_lines.push(format!("  Size: {} bytes", inst.bytes.len()));
            if let Some(ref comment) = inst.comment {
                self.info_lines.push(format!("  Note: {}", comment));
            }
            if let Some(target) = inst.branch_target {
                self.info_lines.push(format!("  Target: 0x{:X}", target));
            }
        }
    }

    
    pub fn handle_key(&mut self, key: char) {
        match key {
            
            '\t' => {
                self.active_panel = match self.active_panel {
                    ActivePanel::Navigation => ActivePanel::HexView,
                    ActivePanel::HexView => ActivePanel::Disassembly,
                    ActivePanel::Disassembly => ActivePanel::Info,
                    ActivePanel::Info => ActivePanel::Navigation,
                };
            },
            
            'U' => self.scroll_up(),   
            'D' => self.scroll_down(), 
            'L' => self.scroll_left(), 
            'R' => self.scroll_right(), 
            
            '\n' | '\r' => self.on_enter(),
            
            'g' | 'G' => {
                self.search_active = !self.search_active;
                if self.search_active {
                    self.search_query.clear();
                }
            },
            
            '0'..='9' | 'a'..='f' | 'A'..='F' if self.search_active => {
                self.search_query.push(key);
            },
            
            '\x08' if self.search_active => {
                self.search_query.pop();
            },
            
            'x' | 'X' => self.follow_xref(),
            _ => {}
        }
    }
    
    
    pub fn handle_scancode(&mut self, scancode: u8) {
        match scancode {
            0x48 => self.scroll_up(),    
            0x50 => self.scroll_down(),  
            0x4B => self.scroll_left(),  
            0x4D => self.scroll_right(), 
            0x49 => {                    
                for _ in 0..20 { self.scroll_up(); }
            },
            0x51 => {                    
                for _ in 0..20 { self.scroll_down(); }
            },
            0x47 => self.go_to_start(), 
            0x4F => self.go_to_end(),   
            0x0F => {                    
                self.active_panel = match self.active_panel {
                    ActivePanel::Navigation => ActivePanel::HexView,
                    ActivePanel::HexView => ActivePanel::Disassembly,
                    ActivePanel::Disassembly => ActivePanel::Info,
                    ActivePanel::Info => ActivePanel::Navigation,
                };
            },
            0x1C => self.on_enter(),     
            _ => {}
        }
    }

    fn scroll_up(&mut self) {
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
                    
                    if let Some(inst) = self.analysis.instructions.get(self.disasm_selected) {
                        self.current_addr = inst.address;
                        self.update_info_panel();
                    }
                }
            },
            ActivePanel::Info => {
                if self.info_scroll > 0 {
                    self.info_scroll -= 1;
                }
            },
        }
    }

    fn scroll_down(&mut self) {
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
                    
                    if let Some(inst) = self.analysis.instructions.get(self.disasm_selected) {
                        self.current_addr = inst.address;
                        self.update_info_panel();
                    }
                }
            },
            ActivePanel::Info => {
                if self.info_scroll + 1 < self.info_lines.len() {
                    self.info_scroll += 1;
                }
            },
        }
    }

    fn scroll_left(&mut self) {
        
        for _ in 0..10 { self.scroll_up(); }
    }

    fn scroll_right(&mut self) {
        
        for _ in 0..10 { self.scroll_down(); }
    }

    fn go_to_start(&mut self) {
        match self.active_panel {
            ActivePanel::Navigation => { self.nav_selected = 0; self.nav_scroll = 0; },
            ActivePanel::HexView => { self.hex_offset = 0; self.hex_selected = 0; },
            ActivePanel::Disassembly => { self.disasm_index = 0; self.disasm_selected = 0; },
            ActivePanel::Info => { self.info_scroll = 0; },
        }
    }

    fn go_to_end(&mut self) {
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
            ActivePanel::Info => {
                self.info_scroll = self.info_lines.len().saturating_sub(1);
            },
        }
    }

    fn on_enter(&mut self) {
        match self.active_panel {
            ActivePanel::Navigation => {
                if let Some((item, _, _)) = self.nav_items.get(self.nav_selected) {
                    match *item {
                        NavItem::Header => {
                            
                            self.goto_address(self.analysis.elf.info.entry);
                        },
                        NavItem::ProgramHeaders => {
                            self.nav_expanded[1] = !self.nav_expanded[1];
                            self.rebuild_nav_tree();
                        },
                        NavItem::Gz(i) => {
                            if let Some(qc) = self.analysis.elf.programs.get(i) {
                                self.goto_address(qc.vaddr);
                            }
                        },
                        NavItem::SectionHeaders => {
                            self.nav_expanded[2] = !self.nav_expanded[2];
                            self.rebuild_nav_tree();
                        },
                        NavItem::Ct(i) => {
                            if let Some(lx) = self.analysis.elf.sections.get(i) {
                                if lx.addr != 0 {
                                    self.goto_address(lx.addr);
                                } else {
                                    
                                    self.hex_offset = lx.offset as usize & !0xF;
                                    self.hex_selected = lx.offset as usize;
                                    self.active_panel = ActivePanel::HexView;
                                }
                            }
                        },
                        NavItem::Symbols => {
                            self.nav_expanded[3] = !self.nav_expanded[3];
                            self.rebuild_nav_tree();
                        },
                        NavItem::Cy(i) => {
                            let jut: Vec<&Cy> = self.analysis.elf.symbols.iter()
                                .chain(self.analysis.elf.dynamic_symbols.iter())
                                .filter(|j| !j.name.is_empty() && j.value != 0)
                                .collect();
                            if let Some(sym) = jut.get(i) {
                                self.goto_address(sym.value);
                            }
                        },
                        NavItem::Functions => {
                            self.nav_expanded[4] = !self.nav_expanded[4];
                            self.rebuild_nav_tree();
                        },
                        NavItem::Aq(i) => {
                            if let Some(func) = self.analysis.xrefs.functions.get(i) {
                                self.goto_address(func.entry);
                            }
                        },
                        NavItem::Strings => {
                            self.nav_expanded[5] = !self.nav_expanded[5];
                            self.rebuild_nav_tree();
                        },
                        NavItem::StringItem(i) => {
                            if let Some(j) = self.analysis.elf.strings.get(i) {
                                if let Some(vaddr) = j.vaddr {
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
                
                if let Some(inst) = self.analysis.instructions.get(self.disasm_selected) {
                    if let Some(target) = inst.branch_target {
                        self.goto_address(target);
                    }
                }
            },
            ActivePanel::HexView => {
                
                if let Some(vaddr) = self.analysis.offset_to_vaddr(self.hex_selected as u64) {
                    self.goto_address(vaddr);
                    self.active_panel = ActivePanel::Disassembly;
                }
            },
            ActivePanel::Info => {},
        }

        
        if self.search_active && !self.search_query.is_empty() {
            if let Ok(addr) = u64::from_str_radix(&self.search_query, 16) {
                self.goto_address(addr);
                self.search_active = false;
            }
        }
    }

    fn follow_xref(&mut self) {
        
        let xrefs = self.analysis.xrefs.xrefs_from(self.current_addr);
        if let Some(aks) = xrefs.first() {
            self.goto_address(aks.to);
        }
    }

    
    pub fn handle_click(&mut self, sk: i32, qn: i32, ul: u32, afy: u32) {
        let en = afy.saturating_sub(J_ + 24) as i32; 
        let cbv = (ul as i32 * 25) / 100; 
        let cko = (ul as i32 * 25) / 100; 
        let cwn = (ul as i32 * 30) / 100; 
        

        let acc = 20i32;
        let aej = 20i32;
        let hnl = (J_ as i32) + acc;

        
        if qn < hnl || qn > afy as i32 - aej {
            return; 
        }

        let bw = 14i32;
        let xf = ((qn - hnl) / bw) as usize;

        if sk < cbv {
            
            self.active_panel = ActivePanel::Navigation;
            let bfi = self.nav_scroll + xf;
            if bfi < self.nav_items.len() {
                self.nav_selected = bfi;
                self.on_enter(); 
            }
        } else if sk < cbv + cko {
            
            self.active_panel = ActivePanel::HexView;
            let flz = self.hex_offset + xf * 16;
            if flz < self.analysis.data.len() {
                self.hex_selected = flz;
                if let Some(vaddr) = self.analysis.offset_to_vaddr(flz as u64) {
                    self.current_addr = vaddr;
                    self.update_info_panel();
                }
            }
        } else if sk < cbv + cko + cwn {
            
            self.active_panel = ActivePanel::Disassembly;
            let fly = self.disasm_index + xf;
            if fly < self.analysis.instructions.len() {
                self.disasm_selected = fly;
                self.current_addr = self.analysis.instructions[fly].address;
                self.update_info_panel();
            }
        } else {
            
            self.active_panel = ActivePanel::Info;
        }
    }
}





pub fn draw_binary_viewer(
    state: &BinaryViewerState,
    wx: i32, wy: i32, ca: u32, er: u32,
    draw_text_fn: &dyn Fn(i32, i32, &str, u32),
) {
    let ana = J_ as i32;
    let aej = 20i32;
    let acc = 20i32;
    
    let ho = wx + 1;
    let bn = wy + ana;
    let hy = ca.saturating_sub(2) as i32;
    let en = (er as i32) - ana - aej;
    
    if hy < 200 || en < 100 {
        return;
    }

    
    let cbv = (hy * 25) / 100;
    let cko = (hy * 25) / 100;
    let cwn = (hy * 30) / 100;
    let eql = hy - cbv - cko - cwn;

    let gis = ho;
    let ckp = gis + cbv;
    let ekf = ckp + cko;
    let czs = ekf + cwn;

    
    crate::framebuffer::fill_rect(wx as u32, (wy + ana) as u32, ca, er - ana as u32, DW_);

    
    let headers = [
        (gis, cbv, "Navigation", ActivePanel::Navigation),
        (ckp, cko, "Hex View", ActivePanel::HexView),
        (ekf, cwn, "Disassembly", ActivePanel::Disassembly),
        (czs, eql, "Info / Xrefs", ActivePanel::Info),
    ];

    for (p, wl, label, panel) in &headers {
        let bg = if *panel == state.active_panel { 0xFF1F6FEB } else { DX_ };
        crate::framebuffer::fill_rect(*p as u32, bn as u32, *wl as u32, acc as u32, bg);
        draw_text_fn(*p + 4, bn + 3, label, AQM_);
    }

    
    for p in &[ckp, ekf, czs] {
        crate::framebuffer::fill_rect(*p as u32, (bn + acc) as u32, 1, en as u32, BQR_);
    }

    let xg = bn + acc;
    let ug = en - acc;
    let bw = 14i32;
    let oe = (ug / bw) as usize;

    
    lju(state, gis, xg, cbv, oe, bw, draw_text_fn);

    
    ljd(state, ckp + 2, xg, cko - 4, oe, bw, draw_text_fn);

    
    lio(state, ekf + 2, xg, cwn - 4, oe, bw, draw_text_fn);

    
    draw_info_panel(state, czs + 2, xg, eql - 4, oe, bw, draw_text_fn);

    
    let status_y = wy + er as i32 - aej;
    crate::framebuffer::fill_rect(wx as u32, status_y as u32, ca, aej as u32, DX_);

    
    let summary = format!(
        " {} | {} | {} insns | {} syms | {} funcs",
        state.file_path,
        state.analysis.elf.info.elf_type,
        state.analysis.instructions.len(),
        state.analysis.elf.symbols.len() + state.analysis.elf.dynamic_symbols.len(),
        state.analysis.xrefs.functions.len(),
    );
    draw_text_fn(wx + 4, status_y + 3, &summary, ABW_);

    
    let bkp = format!("0x{:016X} ", state.current_addr);
    let jua = wx + ca as i32 - (bkp.len() as i32 * 8) - 4;
    draw_text_fn(jua, status_y + 3, &bkp, IJ_);

    
    if state.search_active {
        let agz = wy + ana + 2;
        let dy = 250i32;
        let dw = 20i32;
        let am = wx + ca as i32 / 2 - dy / 2;
        crate::framebuffer::fill_rect(am as u32, agz as u32, dy as u32, dw as u32, 0xFF1F6FEB);
        let nh = format!("Go to: 0x{}_", state.search_query);
        draw_text_fn(am + 4, agz + 3, &nh, 0xFFFFFFFF);
    }
}


fn lju(
    state: &BinaryViewerState,
    x: i32, y: i32, w: i32,
    visible: usize, bw: i32,
    draw_text_fn: &dyn Fn(i32, i32, &str, u32),
) {
    let start = state.nav_scroll;
    let end = (start + visible).min(state.nav_items.len());

    for (pt, idx) in (start..end).enumerate() {
        let ly = y + (pt as i32) * bw;
        let (item, axq, text) = &state.nav_items[idx];

        
        if idx == state.nav_selected {
            let bg = if state.active_panel == ActivePanel::Navigation { AAW_ } else { AAV_ };
            crate::framebuffer::fill_rect(x as u32, ly as u32, w as u32, bw as u32, bg);
        }

        
        let color = match item {
            NavItem::Header | NavItem::ProgramHeaders | NavItem::SectionHeaders |
            NavItem::Symbols | NavItem::Functions | NavItem::Strings |
            NavItem::DynamicInfo | NavItem::Relocations => BQV_,
            NavItem::Aq(_) => ABV_,
            NavItem::Cy(_) => AQP_,
            NavItem::StringItem(_) => AQQ_,
            _ => AQT_,
        };

        
        let nd = (w / 8).max(4) as usize;
        let display = if text.len() > nd {
            &text[..nd]
        } else {
            text
        };

        draw_text_fn(x + 2 + (*axq as i32 * 8), ly + 1, display, color);
    }
}


fn ljd(
    state: &BinaryViewerState,
    x: i32, y: i32, _w: i32,
    visible: usize, bw: i32,
    draw_text_fn: &dyn Fn(i32, i32, &str, u32),
) {
    let data = &state.analysis.data;
    let azv = state.hex_offset;

    for line in 0..visible {
        let offset = azv + line * 16;
        if offset >= data.len() { break; }

        let ly = y + (line as i32) * bw;
        let end = (offset + 16).min(data.len());
        let df = &data[offset..end];

        
        if offset <= state.hex_selected && state.hex_selected < offset + 16 {
            let bg = if state.active_panel == ActivePanel::HexView { AAW_ } else { AAV_ };
            crate::framebuffer::fill_rect(x as u32, ly as u32, 400, bw as u32, bg);
        }

        
        let bqd = format!("{:06X}", offset);
        draw_text_fn(x, ly + 1, &bqd, IJ_);

        
        let mut aib = x + 56;
        for (i, &b) in df.iter().enumerate() {
            if i == 8 { aib += 4; } 
            let kgo = format!("{:02X}", b);
            
            
            let col = if b == 0 {
                0xFF484F58 
            } else if b >= 0x20 && b < 0x7F {
                BQN_ 
            } else {
                AQN_ 
            };
            draw_text_fn(aib, ly + 1, &kgo, col);
            aib += 20;
        }

        
        let efq = x + 56 + 16 * 20 + 12;
        let mut ax = efq;
        for &b in df {
            let ch = if b >= 0x20 && b < 0x7F { b as char } else { '.' };
            let mut buf = [0u8; 4];
            let j = ch.encode_utf8(&mut buf);
            let col = if b >= 0x20 && b < 0x7F { BQH_ } else { 0xFF484F58 };
            draw_text_fn(ax, ly + 1, j, col);
            ax += 8;
        }
    }
}


fn lio(
    state: &BinaryViewerState,
    x: i32, y: i32, w: i32,
    visible: usize, bw: i32,
    draw_text_fn: &dyn Fn(i32, i32, &str, u32),
) {
    let btl = &state.analysis.instructions;
    if btl.is_empty() {
        draw_text_fn(x + 4, y + 20, "No code to display", AQL_);
        return;
    }

    
    let start = state.disasm_index;
    let end = (start + visible).min(btl.len());

    for (pt, idx) in (start..end).enumerate() {
        let inst = &btl[idx];
        let ly = y + (pt as i32) * bw;

        
        if idx == state.disasm_selected {
            let bg = if state.active_panel == ActivePanel::Disassembly { AAW_ } else { AAV_ };
            crate::framebuffer::fill_rect(x as u32, ly as u32, w as u32, bw as u32, bg);
        }

        
        if state.analysis.xrefs.is_function_entry(inst.address) {
            if let Some(name) = state.analysis.elf.addr_to_symbol.get(&inst.address) {
                
                let label = format!("<{}>:", name);
                let nd = (w / 8).max(4) as usize;
                let display = if label.len() > nd { &label[..nd] } else { &label };
                draw_text_fn(x + 2, ly + 1, display, BQO_);
                continue; 
            }
        }

        let mut cx = x + 2;

        
        let bqd = format!("{:08X}", inst.address);
        draw_text_fn(cx, ly + 1, &bqd, IJ_);
        cx += 72;

        
        let nb = inst.bytes.len().min(6);
        let mut dkf = String::new();
        for b in &inst.bytes[..nb] {
            dkf.push_str(&format!("{:02X}", b));
        }
        if inst.bytes.len() > 6 { dkf.push_str(".."); }
        
        while dkf.len() < 14 { dkf.push(' '); }
        draw_text_fn(cx, ly + 1, &dkf, 0xFF484F58);
        cx += 116;

        
        let nfs = if inst.is_call {
            AQK_
        } else if inst.is_jump || inst.is_cond_jump {
            AQO_
        } else if inst.is_ret {
            ABV_
        } else {
            BQP_
        };
        let nfr = format!("{:<7}", inst.mnemonic);
        draw_text_fn(cx, ly + 1, &nfr, nfs);
        cx += 60;

        
        let imp = ((w - (cx - x)) / 8).max(1) as usize;
        let operands = if inst.operands_str.len() > imp {
            &inst.operands_str[..imp]
        } else {
            &inst.operands_str
        };

        
        let nnl = if inst.operands_str.starts_with("0x") || inst.operands_str.contains("0x") {
            AQN_
        } else {
            AQP_
        };
        draw_text_fn(cx, ly + 1, operands, nnl);

        
        if let Some(ref comment) = inst.comment {
            let fnw = format!(" ; {}", comment);
            let izm = w - (cx - x) - (operands.len() as i32 * 8);
            if izm > 24 {
                let imh = (izm / 8) as usize;
                let display = if fnw.len() > imh { &fnw[..imh] } else { &fnw };
                draw_text_fn(cx + (operands.len() as i32 * 8), ly + 1, display, AQL_);
            }
        }
    }
}


fn draw_info_panel(
    state: &BinaryViewerState,
    x: i32, y: i32, _w: i32,
    visible: usize, bw: i32,
    draw_text_fn: &dyn Fn(i32, i32, &str, u32),
) {
    let start = state.info_scroll;
    let end = (start + visible).min(state.info_lines.len());

    for (pt, idx) in (start..end).enumerate() {
        let ly = y + (pt as i32) * bw;
        let line = &state.info_lines[idx];

        
        let col = if line.starts_with("Address:") || line.starts_with("Section:") ||
                     line.starts_with("Symbol:") || line.starts_with("Function:") ||
                     line.starts_with("Instruction:") {
            AQM_
        } else if line.starts_with("Xrefs") {
            BRB_
        } else if line.starts_with("  ->") || line.starts_with("  <-") {
            AQK_
        } else if line.starts_with("  CALL") || line.starts_with("  JMP") || line.starts_with("  Jcc") {
            AQO_
        } else if line.starts_with("  DATA") {
            AQQ_
        } else if line.starts_with("Calls to:") || line.starts_with("Called from:") {
            ABV_
        } else {
            AQT_
        };

        
        let nd = 40usize; 
        let display = if line.len() > nd { &line[..nd] } else { line };
        draw_text_fn(x + 2, ly + 1, display, col);
    }
}
