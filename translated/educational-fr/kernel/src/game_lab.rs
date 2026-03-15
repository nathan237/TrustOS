//! GameLab — Real-time Game Boy Emulator Analysis Laboratory
//!
//! Full-featured debug dashboard with:
//!   - 6-panel live analysis (CPU, GPU, Memory, I/O, Cart, Input)
//!   - Memory Search (Cheat Engine style — scan/filter/narrow results)
//!   - Watch List (pin addresses, highlight changes)
//!   - Save/Load State (instant snapshot/restore)
//!   - Breakpoints + Step (pause on PC, single-step, frame advance)
//!   - Tile/Sprite Viewer (VRAM tiles, OAM sprites)
//!   - Speed Control (0.25x to 4x, pause, frame advance)
//!   - Trace Log (last N instructions with disassembly)
//!   - Memory Diff (highlight changed bytes in hex dump)

extern crate alloc;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use crate::framebuffer;
use crate::gameboy::GameBoyEmulator;

// ── Layout ─────────────────────────────────────────────────────────────────
const TITLE_BAR_H: u32 = 28;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const PANEL_PAD: u32 = 4;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const LINE_H: u32 = 14;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const CHAR_W: u32 = 8;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const TOOLBAR_H: u32 = 24;

// ── Colors ─────────────────────────────────────────────────────────────────
const COLUMN_BG: u32         = 0xFF0A0F14;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const COLUMN_PANEL: u32      = 0xFF111920;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const COLUMN_BORDER: u32     = 0xFF1E2A36;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const COLUMN_HEADER_BG: u32  = 0xFF142028;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const COLUMN_TEXT: u32       = 0xFF9CD8B0;   // Soft green
const COLUMN_DIM: u32        = 0xFF4A6A54;   // Dim green
const COLUMN_BRIGHT: u32     = 0xFF00FF88;   // Neon green
const COLUMN_ACCENT: u32     = 0xFF58A6FF;   // Blue
const COLUMN_VALUE: u32      = 0xFFE0F8D0;   // Light green (values)
const COLUMN_REGISTER: u32        = 0xFF80FFAA;   // Register names
const COLUMN_FLAG_ON: u32    = 0xFF00FF66;   // Flag active
const COLUMN_FLAG_OFF: u32   = 0xFF2A3A30;   // Flag inactive
const COLUMN_WARN: u32       = 0xFFD29922;   // Yellow/warning
const COLUMN_RED: u32        = 0xFFF85149;   // Red
const COLUMN_CYAN: u32       = 0xFF79C0FF;   // Cyan
const COLUMN_PURPLE: u32     = 0xFFBC8CFF;   // Purple
const COLUMN_ADDRESS: u32       = 0xFF507060;   // Address color in hex dump
const COLUMN_CHANGED: u32    = 0xFFFF4444;   // Red for changed bytes
const COLUMN_TOOLBAR: u32    = 0xFF0E1820;

// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const MAXIMUM_WATCH: usize = 16;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const MAXIMUM_SEARCH_RESULTS: usize = 256;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const TRACE_SIZE: usize = 64;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const MAXIMUM_BREAKPOINTS: usize = 8;

// ── Tab modes ──────────────────────────────────────────────────────────────
#[derive(Clone, Copy, PartialEq)]
// Énumération — un type qui peut être l'une de plusieurs variantes.
pub enum LabTab {
    Analyze = 0,
    Search = 1,
    Watch = 2,
    Tiles = 3,
    Trace = 4,
}

// ── Watch entry ────────────────────────────────────────────────────────────
#[derive(Clone)]
// Structure publique — visible à l'extérieur de ce module.
pub struct WatchEntry {
    pub address: u16,
    pub label: [u8; 8],    // short label (ASCII)
    pub label_length: u8,
    pub previous_value: u8,
    pub cur_value: u8,
    pub changed: bool,
    pub size: u8,           // 1=byte, 2=word
}

// ── Search state ───────────────────────────────────────────────────────────
#[derive(Clone, Copy, PartialEq)]
// Énumération — un type qui peut être l'une de plusieurs variantes.
pub enum SearchMode {
    Exact,      // Search for exact value
    Changed,    // Value changed since last scan
    Unchanged,  // Value didn't change
    Greater,    // Value increased
    Less,       // Value decreased
}

// ── Trace entry ────────────────────────────────────────────────────────────
#[derive(Clone, Copy)]
// Structure publique — visible à l'extérieur de ce module.
pub struct TraceEntry {
    pub pc: u16,
    pub opcode: u8,
    pub a: u8, pub f: u8,
    pub sp: u16,
}

// ── Save state ─────────────────────────────────────────────────────────────
pub struct SaveState {
    pub cpu_a: u8, pub cpu_f: u8,
    pub cpu_b: u8, pub cpu_c: u8,
    pub cpu_d: u8, pub cpu_e: u8,
    pub cpu_h: u8, pub cpu_l: u8,
    pub cpu_sp: u16, pub cpu_pc: u16,
    pub cpu_ime: bool, pub cpu_halted: bool,
    pub ie_register: u8, pub if_register: u8,
    pub joypad_register: u8,
    pub joypad_buttons: u8, pub joypad_dirs: u8,
    pub serial_data: u8, pub serial_controller: u8,
    pub wram_bank: u8, pub key1: u8,
    pub wram: Vec<u8>,
    pub hram: [u8; 127],
    pub gpu_lcdc: u8, pub gpu_status: u8,
    pub gpu_scy: u8, pub gpu_scx: u8,
    pub gpu_ly: u8, pub gpu_lyc: u8,
    pub gpu_bgp: u8, pub gpu_obp0: u8, pub gpu_obp1: u8,
    pub gpu_wy: u8, pub gpu_wx: u8,
    pub gpu_mode: u8, pub gpu_cycles: u32,
    pub gpu_vram: [u8; 8192],
    pub gpu_vram1: [u8; 8192],
    pub gpu_oam: [u8; 160],
    pub gpu_bg_palette: [u8; 64],
    pub gpu_object_palette: [u8; 64],
    pub gpu_vram_bank: u8,
    pub gpu_bcps: u8, pub gpu_ocps: u8,
    pub gpu_window_line: u8,
    pub timer_div: u16, pub timer_tima: u8,
    pub timer_tma: u8, pub timer_tac: u8,
    pub cart_rom_bank: u16, pub cart_ram_bank: u8,
    pub cart_ram_enabled: bool, pub cart_mode: u8,
    pub cart_ram: Vec<u8>,
    pub valid: bool,
}

// Bloc d'implémentation — définit les méthodes du type ci-dessus.
impl SaveState {
        // Fonction publique — appelable depuis d'autres modules.
pub fn empty() -> Self {
        Self {
            cpu_a: 0, cpu_f: 0, cpu_b: 0, cpu_c: 0,
            cpu_d: 0, cpu_e: 0, cpu_h: 0, cpu_l: 0,
            cpu_sp: 0, cpu_pc: 0, cpu_ime: false, cpu_halted: false,
            ie_register: 0, if_register: 0, joypad_register: 0,
            joypad_buttons: 0xF, joypad_dirs: 0xF,
            serial_data: 0, serial_controller: 0,
            wram_bank: 1, key1: 0,
            wram: Vec::new(), hram: [0; 127],
            gpu_lcdc: 0, gpu_status: 0, gpu_scy: 0, gpu_scx: 0,
            gpu_ly: 0, gpu_lyc: 0, gpu_bgp: 0, gpu_obp0: 0, gpu_obp1: 0,
            gpu_wy: 0, gpu_wx: 0, gpu_mode: 0, gpu_cycles: 0,
            gpu_vram: [0; 8192], gpu_vram1: [0; 8192],
            gpu_oam: [0; 160],
            gpu_bg_palette: [0; 64], gpu_object_palette: [0; 64],
            gpu_vram_bank: 0, gpu_bcps: 0, gpu_ocps: 0,
            gpu_window_line: 0,
            timer_div: 0, timer_tima: 0, timer_tma: 0, timer_tac: 0,
            cart_rom_bank: 1, cart_ram_bank: 0,
            cart_ram_enabled: false, cart_mode: 0,
            cart_ram: Vec::new(),
            valid: false,
        }
    }
}

/// State for the GameLab window
pub struct GameLabState {
    /// Which Game Boy window we're linked to (None = auto-detect first one)
    pub linked_gb_id: Option<u32>,
    /// Memory view start address
    pub memory_view_address: u16,
    /// Selected panel (0-5) in analyze mode
    pub selected_panel: u8,
    /// Frame counter for blinking
    pub frame: u32,
    /// Memory view mode: 0=WRAM, 1=VRAM, 2=HRAM, 3=ROM, 4=OAM
    pub memory_mode: u8,
    /// Scroll offset for panels
    pub scroll: u32,

    // ── Tab system ─────────────────────────────────────────────────
    pub active_tab: LabTab,

    // ── Watch List ─────────────────────────────────────────────────
    pub watches: Vec<WatchEntry>,

    // ── Memory Search ──────────────────────────────────────────────
    pub search_value: u16,         // Value to search for
    pub search_input: [u8; 6],     // Input buffer for typing
    pub search_input_length: u8,
    pub search_results: Vec<u16>,  // Matching addresses
    pub search_snapshot: Vec<u8>,  // WRAM snapshot for compare scans
    pub search_mode: SearchMode,
    pub search_active: bool,       // Has an initial scan been done?
    pub search_byte_mode: bool,    // true=byte(u8), false=word(u16)

    // ── Breakpoints + Stepping ─────────────────────────────────────
    pub breakpoints: Vec<u16>,     // PC addresses to break on
    pub bp_input: [u8; 5],        // Input buffer for adding BP
    pub bp_input_length: u8,
    pub paused: bool,
    pub step_one: bool,            // Execute exactly one instruction
    pub step_frame: bool,          // Execute one frame then pause

    // ── Speed Control ──────────────────────────────────────────────
    /// Speed multiplier index: 0=0.25x, 1=0.5x, 2=1x, 3=2x, 4=4x
    pub speed_index: u8,

    // ── Trace Log ──────────────────────────────────────────────────
    pub trace: Vec<TraceEntry>,
    pub trace_enabled: bool,

    // ── Tile Viewer ────────────────────────────────────────────────
    pub tile_page: u8,   // 0=tiles $8000, 1=tiles $8800, 2=OAM sprites
    pub tile_scroll: u32,

    // ── Memory Diff ────────────────────────────────────────────────
    pub memory_previous: [u8; 256], // previous 256 bytes for diff highlighting

    // ── Save State ─────────────────────────────────────────────────
    pub save_state: SaveState,
}

// Bloc d'implémentation — définit les méthodes du type ci-dessus.
impl GameLabState {
        // Fonction publique — appelable depuis d'autres modules.
pub fn new() -> Self {
        Self {
            linked_gb_id: None,
            memory_view_address: 0xC000,
            selected_panel: 0,
            frame: 0,
            memory_mode: 0,
            scroll: 0,
            active_tab: LabTab::Analyze,
            watches: Vec::new(),
            search_value: 0,
            search_input: [0; 6],
            search_input_length: 0,
            search_results: Vec::new(),
            search_snapshot: Vec::new(),
            search_mode: SearchMode::Exact,
            search_active: false,
            search_byte_mode: true,
            breakpoints: Vec::new(),
            bp_input: [0; 5],
            bp_input_length: 0,
            paused: false,
            step_one: false,
            step_frame: false,
            speed_index: 2, // 1x default
            trace: Vec::new(),
            trace_enabled: false,
            tile_page: 0,
            tile_scroll: 0,
            memory_previous: [0; 256],
            save_state: SaveState::empty(),
        }
    }

        // Fonction publique — appelable depuis d'autres modules.
pub fn speed_multiplier(&self) -> f32 {
                // Correspondance de motifs — branchement exhaustif de Rust.
match self.speed_index {
            0 => 0.25,
            1 => 0.5,
            2 => 1.0,
            3 => 2.0,
            4 => 4.0,
            _ => 1.0,
        }
    }

        // Fonction publique — appelable depuis d'autres modules.
pub fn speed_label(&self) -> &'static str {
                // Correspondance de motifs — branchement exhaustif de Rust.
match self.speed_index {
            0 => "0.25x",
            1 => "0.5x",
            2 => "1x",
            3 => "2x",
            4 => "4x",
            _ => "1x",
        }
    }

    /// Record a trace entry (called before each instruction from desktop.rs)
    pub fn record_trace(&mut self, emu: &GameBoyEmulator) {
        if !self.trace_enabled { return; }
        let pc = emu.cpu.pc;
        let opcode = read_emu_byte(emu, pc);
        let entry = TraceEntry {
            pc,
            opcode,
            a: emu.cpu.a,
            f: emu.cpu.f,
            sp: emu.cpu.sp,
        };
        if self.trace.len() >= TRACE_SIZE {
            self.trace.remove(0);
        }
        self.trace.push(entry);
    }

    /// Check if we should break at given PC
    pub fn should_break(&self, pc: u16) -> bool {
        self.breakpoints.iter().any(|&bp| bp == pc)
    }

    /// Save state from emulator
    pub fn save_from(&mut self, emu: &GameBoyEmulator) {
        let s = &mut self.save_state;
        s.cpu_a = emu.cpu.a; s.cpu_f = emu.cpu.f;
        s.cpu_b = emu.cpu.b; s.cpu_c = emu.cpu.c;
        s.cpu_d = emu.cpu.d; s.cpu_e = emu.cpu.e;
        s.cpu_h = emu.cpu.h; s.cpu_l = emu.cpu.l;
        s.cpu_sp = emu.cpu.sp; s.cpu_pc = emu.cpu.pc;
        s.cpu_ime = emu.cpu.ime; s.cpu_halted = emu.cpu.halted;
        s.ie_register = emu.ie_register; s.if_register = emu.if_register;
        s.joypad_register = emu.joypad_register;
        s.joypad_buttons = emu.joypad_buttons;
        s.joypad_dirs = emu.joypad_dirs;
        s.serial_data = emu.serial_data;
        s.serial_controller = emu.serial_controller;
        s.wram_bank = emu.wram_bank; s.key1 = emu.key1;
        s.wram = emu.wram.clone();
        s.hram = emu.hram;
        s.gpu_lcdc = emu.gpu.lcdc; s.gpu_status = emu.gpu.status;
        s.gpu_scy = emu.gpu.scy; s.gpu_scx = emu.gpu.scx;
        s.gpu_ly = emu.gpu.ly; s.gpu_lyc = emu.gpu.lyc;
        s.gpu_bgp = emu.gpu.bgp; s.gpu_obp0 = emu.gpu.obp0; s.gpu_obp1 = emu.gpu.obp1;
        s.gpu_wy = emu.gpu.wy; s.gpu_wx = emu.gpu.wx;
        s.gpu_mode = emu.gpu.mode; s.gpu_cycles = emu.gpu.cycles;
        s.gpu_vram = emu.gpu.vram;
        s.gpu_vram1 = emu.gpu.vram1;
        s.gpu_oam = emu.gpu.oam;
        s.gpu_bg_palette = emu.gpu.bg_palette;
        s.gpu_object_palette = emu.gpu.object_palette;
        s.gpu_vram_bank = emu.gpu.vram_bank;
        s.gpu_bcps = emu.gpu.bcps; s.gpu_ocps = emu.gpu.ocps;
        s.gpu_window_line = emu.gpu.window_line;
        s.timer_div = emu.timer.div; s.timer_tima = emu.timer.tima;
        s.timer_tma = emu.timer.tma; s.timer_tac = emu.timer.tac;
        s.cart_rom_bank = emu.cart.rom_bank;
        s.cart_ram_bank = emu.cart.ram_bank;
        s.cart_ram_enabled = emu.cart.ram_enabled;
        s.cart_mode = emu.cart.mode;
        s.cart_ram = emu.cart.ram.clone();
        s.valid = true;
    }

    /// Load state into emulator
    pub fn load_into(&self, emu: &mut GameBoyEmulator) {
        let s = &self.save_state;
        if !s.valid { return; }
        emu.cpu.a = s.cpu_a; emu.cpu.f = s.cpu_f;
        emu.cpu.b = s.cpu_b; emu.cpu.c = s.cpu_c;
        emu.cpu.d = s.cpu_d; emu.cpu.e = s.cpu_e;
        emu.cpu.h = s.cpu_h; emu.cpu.l = s.cpu_l;
        emu.cpu.sp = s.cpu_sp; emu.cpu.pc = s.cpu_pc;
        emu.cpu.ime = s.cpu_ime; emu.cpu.halted = s.cpu_halted;
        emu.ie_register = s.ie_register; emu.if_register = s.if_register;
        emu.joypad_register = s.joypad_register;
        emu.joypad_buttons = s.joypad_buttons;
        emu.joypad_dirs = s.joypad_dirs;
        emu.serial_data = s.serial_data;
        emu.serial_controller = s.serial_controller;
        emu.wram_bank = s.wram_bank; emu.key1 = s.key1;
        if s.wram.len() == emu.wram.len() {
            emu.wram.copy_from_slice(&s.wram);
        }
        emu.hram = s.hram;
        emu.gpu.lcdc = s.gpu_lcdc; emu.gpu.status = s.gpu_status;
        emu.gpu.scy = s.gpu_scy; emu.gpu.scx = s.gpu_scx;
        emu.gpu.ly = s.gpu_ly; emu.gpu.lyc = s.gpu_lyc;
        emu.gpu.bgp = s.gpu_bgp; emu.gpu.obp0 = s.gpu_obp0; emu.gpu.obp1 = s.gpu_obp1;
        emu.gpu.wy = s.gpu_wy; emu.gpu.wx = s.gpu_wx;
        emu.gpu.mode = s.gpu_mode; emu.gpu.cycles = s.gpu_cycles;
        emu.gpu.vram = s.gpu_vram;
        emu.gpu.vram1 = s.gpu_vram1;
        emu.gpu.oam = s.gpu_oam;
        emu.gpu.bg_palette = s.gpu_bg_palette;
        emu.gpu.object_palette = s.gpu_object_palette;
        emu.gpu.vram_bank = s.gpu_vram_bank;
        emu.gpu.bcps = s.gpu_bcps; emu.gpu.ocps = s.gpu_ocps;
        emu.gpu.window_line = s.gpu_window_line;
        emu.timer.div = s.timer_div; emu.timer.tima = s.timer_tima;
        emu.timer.tma = s.timer_tma; emu.timer.tac = s.timer_tac;
        emu.cart.rom_bank = s.cart_rom_bank;
        emu.cart.ram_bank = s.cart_ram_bank;
        emu.cart.ram_enabled = s.cart_ram_enabled;
        emu.cart.mode = s.cart_mode;
        if s.cart_ram.len() == emu.cart.ram.len() {
            emu.cart.ram.copy_from_slice(&s.cart_ram);
        }
    }

    /// Take a WRAM snapshot for memory search comparisons
    pub fn take_search_snapshot(&mut self, emu: &GameBoyEmulator) {
        self.search_snapshot.clear();
        self.search_snapshot.reserve(emu.wram.len());
        for &b in emu.wram.iter() {
            self.search_snapshot.push(b);
        }
    }

    /// First scan: search all WRAM for exact value
    pub fn search_initial(&mut self, emu: &GameBoyEmulator) {
        self.search_results.clear();
        let value = self.search_value as u8;
        for (i, &b) in emu.wram.iter().enumerate() {
            if b == value {
                let address = if i < 0x1000 { 0xC000 + i as u16 } else { 0xD000 + (i as u16 - 0x1000) };
                self.search_results.push(address);
                if self.search_results.len() >= MAXIMUM_SEARCH_RESULTS { break; }
            }
        }
        self.take_search_snapshot(emu);
        self.search_active = true;
    }

    /// Filter scan: narrow existing results
    pub fn search_filter(&mut self, emu: &GameBoyEmulator) {
        if !self.search_active { return; }
        let new_results: Vec<u16> = self.search_results.iter().copied().filter(|&address| {
            let cur = read_emu_byte(emu, address);
            let previous = self.snapshot_byte(address);
                        // Correspondance de motifs — branchement exhaustif de Rust.
match self.search_mode {
                SearchMode::Exact => cur == self.search_value as u8,
                SearchMode::Changed => cur != previous,
                SearchMode::Unchanged => cur == previous,
                SearchMode::Greater => cur > previous,
                SearchMode::Less => cur < previous,
            }
        }).collect();
        self.search_results = new_results;
        self.take_search_snapshot(emu);
    }

    fn snapshot_byte(&self, address: u16) -> u8 {
        let index = // Correspondance de motifs — branchement exhaustif de Rust.
match address {
            0xC000..=0xCFFF => (address - 0xC000) as usize,
            0xD000..=0xDFFF => 0x1000 + (address - 0xD000) as usize,
            _ => return 0xFF,
        };
        if index < self.search_snapshot.len() { self.search_snapshot[index] } else { 0xFF }
    }

    /// Add a watch from a search result address
    pub fn add_watch(&mut self, address: u16) {
        if self.watches.len() >= MAXIMUM_WATCH { return; }
        if self.watches.iter().any(|w| w.address == address) { return; }
        let mut label = [0u8; 8];
        let s = format!("{:04X}", address);
        for (i, b) in s.bytes().enumerate().take(8) { label[i] = b; }
        self.watches.push(WatchEntry {
            address,
            label,
            label_length: s.len().minimum(8) as u8,
            previous_value: 0,
            cur_value: 0,
            changed: false,
            size: 1,
        });
    }

    /// Update all watch values
    pub fn update_watches(&mut self, emu: &GameBoyEmulator) {
        for w in self.watches.iterator_mut() {
            w.previous_value = w.cur_value;
            w.cur_value = read_emu_byte(emu, w.address);
            w.changed = w.cur_value != w.previous_value;
        }
    }

        // Fonction publique — appelable depuis d'autres modules.
pub fn tick(&mut self) {
        self.frame = self.frame.wrapping_add(1);
    }

        // Fonction publique — appelable depuis d'autres modules.
pub fn handle_key(&mut self, key: u8) {
                // Correspondance de motifs — branchement exhaustif de Rust.
match self.active_tab {
            LabTab::Analyze => self.handle_key_analyze(key),
            LabTab::Search => self.handle_key_search(key),
            LabTab::Watch => self.handle_key_watch(key),
            LabTab::Tiles => self.handle_key_tiles(key),
            LabTab::Trace => self.handle_key_trace(key),
        }
        // Global keys
        match key {
            // P = pause/resume
            b'p' | b'P' => self.paused = !self.paused,
            // N = step one instruction
            b'n' | b'N' => { self.step_one = true; self.paused = true; }
            // M = step one frame
            b'm' | b'M' => { self.step_frame = true; self.paused = true; }
            // < / > speed
            b',' => { if self.speed_index > 0 { self.speed_index -= 1; } }
            b'.' => { if self.speed_index < 4 { self.speed_index += 1; } }
            _ => {}
        }
    }

    fn handle_key_analyze(&mut self, key: u8) {
                // Correspondance de motifs — branchement exhaustif de Rust.
match key {
            0x09 => { self.selected_panel = (self.selected_panel + 1) % 6; }
            0xF0 => { self.memory_view_address = self.memory_view_address.wrapping_sub(0x10); }
            0xF1 => { self.memory_view_address = self.memory_view_address.wrapping_add(0x10); }
            0xF2 => { self.memory_view_address = self.memory_view_address.wrapping_sub(0x100); }
            0xF3 => { self.memory_view_address = self.memory_view_address.wrapping_add(0x100); }
            b'1' => { self.memory_mode = 0; self.memory_view_address = 0xC000; }
            b'2' => { self.memory_mode = 1; self.memory_view_address = 0x8000; }
            b'3' => { self.memory_mode = 2; self.memory_view_address = 0xFF80; }
            b'4' => { self.memory_mode = 3; self.memory_view_address = 0x0000; }
            b'5' => { self.memory_mode = 4; self.memory_view_address = 0xFE00; }
            _ => {}
        }
    }

    fn handle_key_search(&mut self, key: u8) {
                // Correspondance de motifs — branchement exhaustif de Rust.
match key {
            b'0'..=b'9' | b'a'..=b'f' | b'A'..=b'F' => {
                if (self.search_input_length as usize) < 4 {
                    self.search_input[self.search_input_length as usize] = key;
                    self.search_input_length += 1;
                    // Parse hex
                    self.search_value = self.parse_search_hex();
                }
            }
            0x08 | 0x7F => { // Backspace
                if self.search_input_length > 0 {
                    self.search_input_length -= 1;
                    self.search_value = self.parse_search_hex();
                }
            }
            // Tab cycles search mode
            0x09 => {
                self.search_mode = // Correspondance de motifs — branchement exhaustif de Rust.
match self.search_mode {
                    SearchMode::Exact => SearchMode::Changed,
                    SearchMode::Changed => SearchMode::Unchanged,
                    SearchMode::Unchanged => SearchMode::Greater,
                    SearchMode::Greater => SearchMode::Less,
                    SearchMode::Less => SearchMode::Exact,
                };
            }
            // 'r' to reset search
            b'r' | b'R' => {
                self.search_results.clear();
                self.search_active = false;
                self.search_input_length = 0;
                self.search_snapshot.clear();
            }
            _ => {}
        }
    }

    fn handle_key_watch(&mut self, key: u8) {
                // Correspondance de motifs — branchement exhaustif de Rust.
match key {
            // Delete last watch
            0x08 | 0x7F => {
                self.watches.pop();
            }
            _ => {}
        }
    }

    fn handle_key_tiles(&mut self, key: u8) {
                // Correspondance de motifs — branchement exhaustif de Rust.
match key {
            0x09 => { self.tile_page = (self.tile_page + 1) % 3; }
            0xF0 => { self.tile_scroll = self.tile_scroll.saturating_sub(1); }
            0xF1 => { self.tile_scroll = self.tile_scroll.saturating_add(1); }
            _ => {}
        }
    }

    fn handle_key_trace(&mut self, key: u8) {
                // Correspondance de motifs — branchement exhaustif de Rust.
match key {
            b't' | b'T' => { self.trace_enabled = !self.trace_enabled; }
            b'r' | b'R' => { self.trace.clear(); }
            _ => {}
        }
    }

    fn parse_search_hex(&self) -> u16 {
        let mut value: u16 = 0;
        for i in 0..self.search_input_length as usize {
            let c = self.search_input[i];
            let digit = // Correspondance de motifs — branchement exhaustif de Rust.
match c {
                b'0'..=b'9' => c - b'0',
                b'a'..=b'f' => c - b'a' + 10,
                b'A'..=b'F' => c - b'A' + 10,
                _ => 0,
            };
            value = (value << 4) | digit as u16;
        }
        value
    }

        // Fonction publique — appelable depuis d'autres modules.
pub fn handle_click(&mut self, receive: i32, ry: i32, ww: u32, _wh: u32) {
        // Header area: TITLE_BAR_H .. TITLE_BAR_H+22  (the "GAME LAB" bar)
        // Toolbar area: TITLE_BAR_H+22 .. TITLE_BAR_H+22+TOOLBAR_H  (tabs + speed + pause)
        let header_h = 22i32;
        let toolbar_y_start = TITLE_BAR_H as i32 + header_h;
        let toolbar_y_end = toolbar_y_start + TOOLBAR_H as i32;
        if ry >= toolbar_y_start && ry < toolbar_y_end {
            let tab_w = 72i32;
            let transmit = receive - 4;
            if transmit >= 0 {
                let tab_index = transmit / tab_w;
                                // Correspondance de motifs — branchement exhaustif de Rust.
match tab_index {
                    0 => self.active_tab = LabTab::Analyze,
                    1 => self.active_tab = LabTab::Search,
                    2 => self.active_tab = LabTab::Watch,
                    3 => self.active_tab = LabTab::Tiles,
                    4 => self.active_tab = LabTab::Trace,
                    _ => {}
                }
            }
            // Speed buttons (right side)
            let speed_x = ww as i32 - 200;
            if receive >= speed_x && receive < speed_x + 24 {
                if self.speed_index > 0 { self.speed_index -= 1; }
            } else if receive >= speed_x + 28 && receive < speed_x + 52 {
                if self.speed_index < 4 { self.speed_index += 1; }
            }
            // Pause button
            let pause_x = ww as i32 - 130;
            if receive >= pause_x && receive < pause_x + 40 {
                self.paused = !self.paused;
            }
            // Step button
            let step_x = ww as i32 - 86;
            if receive >= step_x && receive < step_x + 36 {
                self.step_one = true; self.paused = true;
            }
            // Frame button
            let frame_x = ww as i32 - 46;
            if receive >= frame_x && receive < frame_x + 42 {
                self.step_frame = true; self.paused = true;
            }
        }

        // Content area starts after toolbar
        let content_y = toolbar_y_end;

        // Search tab: mode selector + result list clicks
        if self.active_tab == LabTab::Search {
            // Mode selector row is about 3 lines below content start
            let mode_y = content_y + 6 + LINE_H as i32 + 4 + LINE_H as i32 + 2;
            if ry >= mode_y && ry < mode_y + LINE_H as i32 {
                let mx = receive - 8 - 48;
                if mx >= 0 {
                    // Each mode button: label_len*8 + 10 spacing
                    let modes_w: [i32; 5] = [5*8+10, 7*8+10, 4*8+10, 7*8+10, 4*8+10];
                    let mut accum = 0i32;
                    for (i, &w) in modes_w.iter().enumerate() {
                        if mx >= accum && mx < accum + w {
                            self.search_mode = // Correspondance de motifs — branchement exhaustif de Rust.
match i {
                                0 => SearchMode::Exact,
                                1 => SearchMode::Changed,
                                2 => SearchMode::Unchanged,
                                3 => SearchMode::Greater,
                                _ => SearchMode::Less,
                            };
                            break;
                        }
                        accum += w;
                    }
                }
            }
            // "Add Watch" clicks on result list
            let list_y_start = content_y + 120;
            if ry >= list_y_start && receive >= (ww as i32 - 60) {
                let index = ((ry - list_y_start) / LINE_H as i32) as usize;
                if index < self.search_results.len() {
                    self.add_watch(self.search_results[index]);
                }
            }
        }

        // Tiles tab: page selector clicks
        if self.active_tab == LabTab::Tiles {
            let page_y = content_y + 6 + LINE_H as i32;
            if ry >= page_y && ry < page_y + LINE_H as i32 {
                let pixel = receive - 8;
                if pixel >= 0 && pixel < 110 { self.tile_page = 0; }
                else if pixel >= 110 && pixel < 220 { self.tile_page = 1; }
                else if pixel >= 220 && pixel < 330 { self.tile_page = 2; }
            }
        }
    }
}

/// Draw the GameLab dashboard
pub fn draw_game_lab(
    state: &GameLabState,
    emu: Option<&GameBoyEmulator>,
    wx: i32, wy: i32, ww: u32, wh: u32,
) {
    let cx = wx as u32;
    let cy = (wy + TITLE_BAR_H as i32) as u32;
    let cw = ww;
    let character = wh.saturating_sub(TITLE_BAR_H);

    if cw < 200 || character < 150 { return; }

    // Background
    framebuffer::fill_rect(cx, cy, cw, character, COLUMN_BG);

    // ── Header bar ─────────────────────────────────────────────────
    framebuffer::fill_rect(cx, cy, cw, 22, COLUMN_HEADER_BG);
    let blink = (state.frame / 15) % 2 == 0;
    let dot_color = if blink { COLUMN_BRIGHT } else { COLUMN_DIM };
    framebuffer::fill_rect(cx + 6, cy + 8, 6, 6, dot_color);
    draw_str(cx + 16, cy + 4, "GAME LAB", COLUMN_BRIGHT);

    // Linked status
    if emu.is_some() {
        draw_str(cx + 100, cy + 4, "[LINKED]", COLUMN_BRIGHT);
    } else {
        draw_str(cx + 100, cy + 4, "[NO EMU]", COLUMN_RED);
    }

    // Save/Load state buttons
    let save_x = cx + cw - 120;
    draw_button(save_x, cy + 2, 48, 16, "SAVE", state.save_state.valid, COLUMN_ACCENT);
    draw_button(save_x + 54, cy + 2, 48, 16, "LOAD", state.save_state.valid, if state.save_state.valid { COLUMN_BRIGHT } else { COLUMN_DIM });

    // ── Toolbar with tabs + speed/pause controls ───────────────────
    let ty = cy + 22;
    framebuffer::fill_rect(cx, ty, cw, TOOLBAR_H, COLUMN_TOOLBAR);
    framebuffer::fill_rect(cx, ty + TOOLBAR_H - 1, cw, 1, COLUMN_BORDER);

    // Tab buttons
    let tabs: [(&str, LabTab); 5] = [
        ("ANALYZE", LabTab::Analyze),
        ("SEARCH", LabTab::Search),
        ("WATCH", LabTab::Watch),
        ("TILES", LabTab::Tiles),
        ("TRACE", LabTab::Trace),
    ];
    let tab_w: u32 = 68;
    for (i, (label, tab)) in tabs.iter().enumerate() {
        let transmit = cx + 4 + i as u32 * (tab_w + 4);
        let active = state.active_tab == *tab;
        let bg = if active { 0xFF1A3828 } else { COLUMN_TOOLBAR };
        framebuffer::fill_rect(transmit, ty + 2, tab_w, TOOLBAR_H - 4, bg);
        if active {
            framebuffer::fill_rect(transmit, ty + TOOLBAR_H - 3, tab_w, 2, COLUMN_BRIGHT);
        }
        let column = if active { COLUMN_BRIGHT } else { COLUMN_DIM };
        draw_str(transmit + 4, ty + 6, label, column);
    }

    // Speed controls (right side)
    let speed_x = cx + cw - 200;
    draw_button(speed_x, ty + 3, 22, 16, "<", false, COLUMN_ACCENT);
    draw_str(speed_x + 26, ty + 6, state.speed_label(), COLUMN_VALUE);
    draw_button(speed_x + 56, ty + 3, 22, 16, ">", false, COLUMN_ACCENT);

    // Pause/Step buttons
    let pause_x = cx + cw - 130;
    let pause_column = if state.paused { COLUMN_RED } else { COLUMN_BRIGHT };
    let pause_label = if state.paused { "PLAY" } else { "PAUS" };
    draw_button(pause_x, ty + 3, 38, 16, pause_label, state.paused, pause_column);
    draw_button(pause_x + 42, ty + 3, 34, 16, "STEP", false, COLUMN_CYAN);
    draw_button(pause_x + 80, ty + 3, 42, 16, "FRAME", false, COLUMN_PURPLE);

    // ── Content area ───────────────────────────────────────────────
    let content_y = ty + TOOLBAR_H;
    let content_h = character.saturating_sub(22 + TOOLBAR_H);

        // Correspondance de motifs — branchement exhaustif de Rust.
match state.active_tab {
        LabTab::Analyze => draw_tab_analyze(state, emu, cx, content_y, cw, content_h),
        LabTab::Search => draw_tab_search(state, emu, cx, content_y, cw, content_h),
        LabTab::Watch => draw_tab_watch(state, emu, cx, content_y, cw, content_h),
        LabTab::Tiles => draw_tab_tiles(state, emu, cx, content_y, cw, content_h),
        LabTab::Trace => draw_tab_trace(state, emu, cx, content_y, cw, content_h),
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TAB: ANALYZE (original 6-panel dashboard)
// ═══════════════════════════════════════════════════════════════════════════

fn draw_tab_analyze(state: &GameLabState, emu: Option<&GameBoyEmulator>, cx: u32, cy: u32, cw: u32, character: u32) {
    let top_h = character * 60 / 100;
    let bot_h = character - top_h - 2;
    let column_w = (cw - 4) / 3;

    draw_panel_cpu(emu, cx + 1, cy, column_w, top_h, state);
    draw_panel_gpu(emu, cx + column_w + 2, cy, column_w, top_h, state);
    draw_panel_memory(emu, state, cx + column_w * 2 + 3, cy, column_w, top_h);

    let by = cy + top_h + 2;
    draw_panel_io(emu, cx + 1, by, column_w, bot_h, state);
    draw_panel_cart(emu, cx + column_w + 2, by, column_w, bot_h);
    draw_panel_input(emu, cx + column_w * 2 + 3, by, column_w, bot_h, state);
}

// ═══════════════════════════════════════════════════════════════════════════
// TAB: SEARCH (Cheat Engine style memory scanner)
// ═══════════════════════════════════════════════════════════════════════════

fn draw_tab_search(state: &GameLabState, emu: Option<&GameBoyEmulator>, cx: u32, cy: u32, cw: u32, character: u32) {
    let pixel = cx + 8;
    let mut py = cy + 6;

    // Title
    draw_str(pixel, py, "MEMORY SEARCH", COLUMN_ACCENT);
    draw_str(pixel + 120, py, "(Hex value, Tab=mode, R=reset)", COLUMN_DIM);
    py += LINE_H + 4;

    // Search input
    draw_str(pixel, py, "VALUE:", COLUMN_REGISTER);
    let mut input_str = String::new();
    for i in 0..state.search_input_length as usize {
        input_str.push(state.search_input[i] as char);
    }
    if input_str.is_empty() { input_str.push_str("__"); }
    // Blinking cursor
    if (state.frame / 20) % 2 == 0 { input_str.push('_'); }
    draw_str(pixel + 52, py, &input_str, COLUMN_VALUE);

    // Parsed value
    let vs = format!("= {} (0x{:02X})", state.search_value, state.search_value);
    draw_str(pixel + 120, py, &vs, COLUMN_DIM);
    py += LINE_H + 2;

    // Search mode indicator
    draw_str(pixel, py, "MODE:", COLUMN_REGISTER);
    let modes = [
        ("EXACT", SearchMode::Exact),
        ("CHANGED", SearchMode::Changed),
        ("SAME", SearchMode::Unchanged),
        ("GREATER", SearchMode::Greater),
        ("LESS", SearchMode::Less),
    ];
    let mut mx = pixel + 48;
    for (label, mode) in &modes {
        let active = state.search_mode == *mode;
        let column = if active { COLUMN_BRIGHT } else { COLUMN_DIM };
        if active { framebuffer::fill_rect(mx - 2, py - 1, label.len() as u32 * CHAR_W + 4, LINE_H, 0xFF1A3020); }
        draw_str(mx, py, label, column);
        mx += label.len() as u32 * CHAR_W + 10;
    }
    py += LINE_H + 2;

    // Action buttons hint
    draw_str(pixel, py, "Enter=Scan/Filter", COLUMN_ACCENT);
    draw_str(pixel + 152, py, "R=Reset", COLUMN_WARN);
    py += LINE_H + 6;

    // Status
    let status = if !state.search_active {
        String::from("No scan yet. Type value + Enter to scan WRAM.")
    } else {
        format!("Results: {} addresses", state.search_results.len())
    };
    draw_str(pixel, py, &status, COLUMN_TEXT);
    py += LINE_H + 4;

    // Results list
    if state.search_active {
        framebuffer::fill_rect(cx + 4, py, cw - 8, 1, COLUMN_BORDER);
        py += 4;
        draw_str(pixel, py, "ADDR", COLUMN_ACCENT);
        draw_str(pixel + 60, py, "VALUE", COLUMN_ACCENT);
        draw_str(pixel + 110, py, "DEC", COLUMN_ACCENT);
        if cw > 400 { draw_str(pixel + 160, py, "PREV", COLUMN_DIM); }
        draw_str(cx + cw - 68, py, "[+WATCH]", COLUMN_BRIGHT);
        py += LINE_H + 2;

        let maximum_rows = ((character.saturating_sub(py - cy)) / LINE_H).minimum(32) as usize;
        for (i, &address) in state.search_results.iter().take(maximum_rows).enumerate() {
            let value = if let Some(e) = emu { read_emu_byte(e, address) } else { 0 };
            let previous = state.snapshot_byte(address);
            let changed = value != previous;

            let address_s = format!("{:04X}", address);
            draw_str(pixel, py, &address_s, COLUMN_ADDRESS);

            let value_s = format!("{:02X}", value);
            let value_column = if changed { COLUMN_CHANGED } else { COLUMN_VALUE };
            draw_str(pixel + 60, py, &value_s, value_column);

            let decrypt_s = format!("{:3}", value);
            draw_str(pixel + 110, py, &decrypt_s, COLUMN_DIM);

            if cw > 400 {
                let previous_s = format!("{:02X}", previous);
                draw_str(pixel + 160, py, &previous_s, COLUMN_DIM);
            }

            // Watch button
            let wb = cx + cw - 48;
            draw_str(wb, py, "+W", COLUMN_BRIGHT);

            py += LINE_H;
            let _ = i;
        }
        if state.search_results.len() > maximum_rows {
            let more = format!("... +{} more", state.search_results.len() - maximum_rows);
            draw_str(pixel, py, &more, COLUMN_DIM);
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TAB: WATCH (pinned memory addresses)
// ═══════════════════════════════════════════════════════════════════════════

fn draw_tab_watch(state: &GameLabState, emu: Option<&GameBoyEmulator>, cx: u32, cy: u32, cw: u32, character: u32) {
    let pixel = cx + 8;
    let mut py = cy + 6;

    draw_str(pixel, py, "WATCH LIST", COLUMN_ACCENT);
    let count_s = format!("{}/{}", state.watches.len(), MAXIMUM_WATCH);
    draw_str(pixel + 100, py, &count_s, COLUMN_DIM);
    draw_str(pixel + 160, py, "(Backspace=remove last)", COLUMN_DIM);
    py += LINE_H + 4;

    if state.watches.is_empty() {
        draw_str(pixel, py, "No watches. Add from Search tab with [+W] button.", COLUMN_DIM);
        return;
    }

    // Header
    framebuffer::fill_rect(cx + 4, py, cw - 8, 1, COLUMN_BORDER);
    py += 4;
    draw_str(pixel, py, "LABEL", COLUMN_ACCENT);
    draw_str(pixel + 72, py, "ADDR", COLUMN_ACCENT);
    draw_str(pixel + 120, py, "HEX", COLUMN_ACCENT);
    draw_str(pixel + 160, py, "DEC", COLUMN_ACCENT);
    draw_str(pixel + 200, py, "PREV", COLUMN_ACCENT);
    if cw > 500 { draw_str(pixel + 250, py, "VISUAL", COLUMN_DIM); }
    py += LINE_H + 2;

    for w in state.watches.iter() {
        let label: String = w.label[..w.label_length as usize].iter().map(|&b| b as char).collect();
        draw_str(pixel, py, &label, COLUMN_REGISTER);

        let address_s = format!("{:04X}", w.address);
        draw_str(pixel + 72, py, &address_s, COLUMN_ADDRESS);

        let value = if let Some(e) = emu { read_emu_byte(e, w.address) } else { w.cur_value };
        let value_s = format!("{:02X}", value);
        let column = if w.changed { COLUMN_CHANGED } else { COLUMN_VALUE };
        draw_str(pixel + 120, py, &value_s, column);

        let decrypt_s = format!("{:3}", value);
        draw_str(pixel + 160, py, &decrypt_s, column);

        let previous_s = format!("{:02X}", w.previous_value);
        draw_str(pixel + 200, py, &previous_s, COLUMN_DIM);

        // Visual bar
        if cw > 500 {
            let bar_w = 100u32;
            let fill = (value as u32 * bar_w) / 255;
            framebuffer::fill_rect(pixel + 250, py + 2, bar_w, 8, 0xFF0A1A10);
            let bar_column = if w.changed { COLUMN_CHANGED } else { COLUMN_BRIGHT };
            framebuffer::fill_rect(pixel + 250, py + 2, fill, 8, bar_column);
        }

        py += LINE_H;
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TAB: TILES (VRAM tile viewer + OAM sprites)
// ═══════════════════════════════════════════════════════════════════════════

fn draw_tab_tiles(state: &GameLabState, emu: Option<&GameBoyEmulator>, cx: u32, cy: u32, cw: u32, character: u32) {
    let pixel = cx + 8;
    let mut py = cy + 6;

    let pages = ["TILES $8000", "TILES $8800", "OAM SPRITES"];
    draw_str(pixel, py, "TILE VIEWER", COLUMN_ACCENT);
    py += LINE_H;

    // Page tabs
    for (i, label) in pages.iter().enumerate() {
        let active = i as u8 == state.tile_page;
        let column = if active { COLUMN_BRIGHT } else { COLUMN_DIM };
        let transmit = pixel + i as u32 * 110;
        if active { framebuffer::fill_rect(transmit - 2, py - 1, label.len() as u32 * CHAR_W + 4, LINE_H, 0xFF1A3020); }
        draw_str(transmit, py, label, column);
    }
    draw_str(pixel + 340, py, "(Tab=page, Arrows=scroll)", COLUMN_DIM);
    py += LINE_H + 6;

    let emu = // Correspondance de motifs — branchement exhaustif de Rust.
match emu {
        Some(e) => e,
        None => { draw_str(pixel, py, "No emulator linked", COLUMN_DIM); return; }
    };

    if state.tile_page < 2 {
        // Draw tiles as 16x16 grid (each tile = 8x8 pixels, drawn 2x scaled = 16x16)
        let base_address: u16 = if state.tile_page == 0 { 0x8000 } else { 0x8800 };
        let tile_size = 2u32; // 2x scale
        let tile_pixel = 8 * tile_size;
        let cols = ((cw - 20) / (tile_pixel + 1)).minimum(16);
        let maximum_rows = ((character - (py - cy) - 4) / (tile_pixel + 1)).minimum(16);
        let scroll = state.tile_scroll.minimum(16u32.saturating_sub(maximum_rows));

        for row in 0..maximum_rows {
            for column in 0..cols {
                let tile_index = (scroll + row) * 16 + column;
                if tile_index >= 256 { break; }
                let tile_address = base_address.wrapping_add(tile_index as u16 * 16);
                let dx = pixel + column * (tile_pixel + 1);
                let dy = py + row * (tile_pixel + 1);

                // Draw 8x8 tile
                for ty_off in 0..8u32 {
                    let lo = read_emu_byte(emu, tile_address.wrapping_add(ty_off as u16 * 2));
                    let hi = read_emu_byte(emu, tile_address.wrapping_add(ty_off as u16 * 2 + 1));
                    for transmit_off in 0..8u32 {
                        let bit = 7 - transmit_off;
                        let color_id = ((hi >> bit) & 1) << 1 | ((lo >> bit) & 1);
                        let shade = // Correspondance de motifs — branchement exhaustif de Rust.
match color_id {
                            0 => 0xFF0A1510,
                            1 => 0xFF346856,
                            2 => 0xFF88C070,
                            3 => 0xFFE0F8D0,
                            _ => 0xFF000000,
                        };
                        framebuffer::fill_rect(
                            dx + transmit_off * tile_size,
                            dy + ty_off * tile_size,
                            tile_size, tile_size, shade,
                        );
                    }
                }
            }
        }

        // Tile index label below
        let label_y = py + maximum_rows * (tile_pixel + 1) + 4;
        let range_s = format!("Tiles {}-{}", scroll * 16, ((scroll + maximum_rows) * 16).minimum(256) - 1);
        draw_str(pixel, label_y, &range_s, COLUMN_DIM);
    } else {
        // OAM sprite listing
        draw_str(pixel, py, "#  Y   X   TILE FLAGS", COLUMN_ACCENT);
        py += LINE_H;

        let maximum_sprites = ((character - (py - cy)) / LINE_H).minimum(40);
        for i in 0..maximum_sprites {
            let oam_address = 0xFE00u16 + i as u16 * 4;
            let sy = read_emu_byte(emu, oam_address);
            let sx = read_emu_byte(emu, oam_address + 1);
            let tile = read_emu_byte(emu, oam_address + 2);
            let flags = read_emu_byte(emu, oam_address + 3);

            let visible = sy > 0 && sy < 160 && sx > 0 && sx < 168;
            let column = if visible { COLUMN_VALUE } else { COLUMN_DIM };

            let s = format!("{:2} {:3} {:3}  {:02X}   {:02X}", i, sy, sx, tile, flags);
            draw_str(pixel, py, &s, column);

            // Flags breakdown
            let flag_x = pixel + 200;
            if flags & 0x80 != 0 { draw_str(flag_x, py, "P", COLUMN_WARN); }
            if flags & 0x40 != 0 { draw_str(flag_x + 12, py, "Y", COLUMN_ACCENT); }
            if flags & 0x20 != 0 { draw_str(flag_x + 24, py, "X", COLUMN_ACCENT); }
            if emu.cgb_mode {
                let pal = flags & 0x07;
                let bank = (flags >> 3) & 1;
                let ps = format!("P{} B{}", pal, bank);
                draw_str(flag_x + 40, py, &ps, COLUMN_CYAN);
            }

            py += LINE_H;
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TAB: TRACE (instruction trace log + disassembly)
// ═══════════════════════════════════════════════════════════════════════════

fn draw_tab_trace(state: &GameLabState, emu: Option<&GameBoyEmulator>, cx: u32, cy: u32, cw: u32, character: u32) {
    let pixel = cx + 8;
    let mut py = cy + 6;

    draw_str(pixel, py, "TRACE LOG", COLUMN_ACCENT);
    let enabled_s = if state.trace_enabled { "[ON]" } else { "[OFF]" };
    let enabled_c = if state.trace_enabled { COLUMN_BRIGHT } else { COLUMN_RED };
    draw_str(pixel + 88, py, enabled_s, enabled_c);
    draw_str(pixel + 132, py, "(T=toggle, R=clear)", COLUMN_DIM);
    py += LINE_H + 2;

    // Disassembly around current PC (top section)
    if let Some(emu) = emu {
        draw_str(pixel, py, "DISASSEMBLY @ PC", COLUMN_ACCENT);
        py += LINE_H;
        let pc = emu.cpu.pc;
        // Show a few instructions before and after PC
        let mut address = pc.wrapping_sub(8);
        let disasm_rows = 12u32.minimum((character / 3) / LINE_H);
        for _ in 0..disasm_rows {
            let opcode = read_emu_byte(emu, address);
            let is_current = address == pc;
            let prefix = if is_current { ">" } else { " " };
            let (mnemonic, size) = disasm_opcode(emu, address);
            let s = format!("{} {:04X}: {:02X}  {}", prefix, address, opcode, mnemonic);
            let column = if is_current { COLUMN_BRIGHT } else { COLUMN_TEXT };
            if is_current {
                framebuffer::fill_rect(pixel - 2, py - 1, cw - 16, LINE_H, 0xFF1A3020);
            }
            draw_str(pixel, py, &s, column);

            // Breakpoint indicator
            if state.breakpoints.iter().any(|&bp| bp == address) {
                framebuffer::fill_rect(pixel - 6, py + 2, 4, 8, COLUMN_RED);
            }

            py += LINE_H;
            address = address.wrapping_add(size as u16);
        }
    }
    py += 6;
    framebuffer::fill_rect(cx + 4, py, cw - 8, 1, COLUMN_BORDER);
    py += 4;

    // Breakpoints section
    draw_str(pixel, py, "BREAKPOINTS", COLUMN_ACCENT);
    let bp_count = format!("{}/{}", state.breakpoints.len(), MAXIMUM_BREAKPOINTS);
    draw_str(pixel + 100, py, &bp_count, COLUMN_DIM);
    py += LINE_H;

    if state.breakpoints.is_empty() {
        draw_str(pixel, py, "None (type addr in Search, click to add)", COLUMN_DIM);
    } else {
        let mut bpx = pixel;
        for &bp in state.breakpoints.iter() {
            let bps = format!("{:04X}", bp);
            framebuffer::fill_rect(bpx, py, 40, LINE_H - 2, 0xFF2A0A0A);
            draw_str(bpx + 2, py, &bps, COLUMN_RED);
            bpx += 48;
            if bpx > cx + cw - 60 { py += LINE_H; bpx = pixel; }
        }
    }
    py += LINE_H + 4;
    framebuffer::fill_rect(cx + 4, py, cw - 8, 1, COLUMN_BORDER);
    py += 4;

    // Trace entries
    draw_str(pixel, py, "TRACE HISTORY", COLUMN_ACCENT);
    let trace_count = format!("({} entries)", state.trace.len());
    draw_str(pixel + 120, py, &trace_count, COLUMN_DIM);
    py += LINE_H;

    draw_str(pixel, py, "PC    OP  A  F  SP", COLUMN_DIM);
    py += LINE_H;

    let maximum_rows = ((character.saturating_sub(py - cy)) / LINE_H) as usize;
    let start = if state.trace.len() > maximum_rows { state.trace.len() - maximum_rows } else { 0 };
    for entry in state.trace[start..].iter() {
        let s = format!("{:04X}  {:02X}  {:02X} {:02X} {:04X}", entry.pc, entry.opcode, entry.a, entry.f, entry.sp);
        draw_str(pixel, py, &s, COLUMN_TEXT);
        py += LINE_H;
        if py + LINE_H > cy + character { break; }
    }
}

// ── Panel: CPU Registers ───────────────────────────────────────────────────
fn draw_panel_cpu(emu: Option<&GameBoyEmulator>, x: u32, y: u32, w: u32, h: u32, state: &GameLabState) {
    draw_panel_frame(x, y, w, h, "CPU REGISTERS", state.selected_panel == 0);

    let pixel = x + PANEL_PAD + 2;
    let mut py = y + 20;

    if let Some(emu) = emu {
        let cpu = &emu.cpu;

        // Register pairs
        let regs = [
            ("AF", cpu.a, cpu.f),
            ("BC", cpu.b, cpu.c),
            ("DE", cpu.d, cpu.e),
            ("HL", cpu.h, cpu.l),
        ];
        for (name, hi, lo) in &regs {
            draw_str(pixel, py, name, COLUMN_REGISTER);
            let s = format!("{:02X}{:02X}", hi, lo);
            draw_str(pixel + 28, py, &s, COLUMN_VALUE);
            // Show individual bytes
            let s2 = format!("({:3} {:3})", hi, lo);
            draw_str(pixel + 72, py, &s2, COLUMN_DIM);
            py += LINE_H;
        }

        py += 4;
        // SP & PC
        draw_str(pixel, py, "SP", COLUMN_REGISTER);
        let sps = format!("{:04X}", cpu.sp);
        draw_str(pixel + 28, py, &sps, COLUMN_VALUE);
        py += LINE_H;

        draw_str(pixel, py, "PC", COLUMN_REGISTER);
        let pcs = format!("{:04X}", cpu.pc);
        draw_str(pixel + 28, py, &pcs, COLUMN_ACCENT);

        // Show opcode at PC
        if emu.rom_loaded {
            let opcode = read_emu_byte(emu, cpu.pc);
            let ops = format!("[{:02X}]", opcode);
            draw_str(pixel + 72, py, &ops, COLUMN_CYAN);
        }
        py += LINE_H + 6;

        // Flags
        draw_str(pixel, py, "FLAGS", COLUMN_DIM);
        py += LINE_H;
        let flags = [
            ("Z", cpu.f & 0x80 != 0),
            ("N", cpu.f & 0x40 != 0),
            ("H", cpu.f & 0x20 != 0),
            ("C", cpu.f & 0x10 != 0),
        ];
        let mut fx = pixel;
        for (name, set) in &flags {
            let color = if *set { COLUMN_FLAG_ON } else { COLUMN_FLAG_OFF };
            framebuffer::fill_rect(fx, py, 24, 14, if *set { 0xFF0A3020 } else { 0xFF0A1510 });
            draw_str(fx + 4, py + 1, name, color);
            fx += 28;
        }
        py += LINE_H + 6;

        // IME, HALT state
        draw_str(pixel, py, "IME", COLUMN_DIM);
        draw_str(pixel + 32, py, if cpu.ime { "ON" } else { "OFF" }, 
            if cpu.ime { COLUMN_FLAG_ON } else { COLUMN_FLAG_OFF });
        draw_str(pixel + 64, py, "HALT", COLUMN_DIM);
        draw_str(pixel + 100, py, if cpu.halted { "YES" } else { "NO" },
            if cpu.halted { COLUMN_WARN } else { COLUMN_DIM });
        py += LINE_H;

        // Cycle counter
        draw_str(pixel, py, "CYCLES", COLUMN_DIM);
        let cs = format!("{}", cpu.cycles);
        draw_str(pixel + 56, py, &cs, COLUMN_VALUE);
        py += LINE_H;

        // CGB mode
        if emu.cgb_mode {
            draw_str(pixel, py, "MODE", COLUMN_DIM);
            draw_str(pixel + 40, py, "CGB", COLUMN_BRIGHT);
            let spd = if emu.key1 & 0x80 != 0 { "2x" } else { "1x" };
            draw_str(pixel + 72, py, spd, COLUMN_ACCENT);
        }
    } else {
        draw_str(pixel, py, "No emulator linked", COLUMN_DIM);
    }
}

// ── Panel: GPU / LCD State ─────────────────────────────────────────────────
fn draw_panel_gpu(emu: Option<&GameBoyEmulator>, x: u32, y: u32, w: u32, h: u32, state: &GameLabState) {
    draw_panel_frame(x, y, w, h, "GPU / LCD", state.selected_panel == 1);

    let pixel = x + PANEL_PAD + 2;
    let mut py = y + 20;

    if let Some(emu) = emu {
        let gpu = &emu.gpu;

        // LCDC register breakdown
        draw_str(pixel, py, "LCDC", COLUMN_REGISTER);
        let ls = format!("{:02X}", gpu.lcdc);
        draw_str(pixel + 40, py, &ls, COLUMN_VALUE);
        let lcd_on = gpu.lcdc & 0x80 != 0;
        draw_str(pixel + 64, py, if lcd_on { "LCD:ON" } else { "LCD:OFF" },
            if lcd_on { COLUMN_FLAG_ON } else { COLUMN_RED });
        py += LINE_H;

        // LCDC bits
        let bits = [
            ("BG", gpu.lcdc & 0x01 != 0),
            ("OBJ", gpu.lcdc & 0x02 != 0),
            ("8x16", gpu.lcdc & 0x04 != 0),
            ("WIN", gpu.lcdc & 0x20 != 0),
        ];
        let mut bx = pixel;
        for (name, on) in &bits {
            let column = if *on { COLUMN_FLAG_ON } else { COLUMN_FLAG_OFF };
            draw_str(bx, py, name, column);
            bx += (name.len() as u32 + 1) * CHAR_W;
        }
        py += LINE_H + 4;

        // Scanline info
        draw_str(pixel, py, "LY", COLUMN_REGISTER);
        let lys = format!("{:3} / 153", gpu.ly);
        draw_str(pixel + 28, py, &lys, COLUMN_VALUE);
        // Scanline progress bar
        let bar_x = pixel + 110;
        let bar_w = w.saturating_sub(130);
        framebuffer::fill_rect(bar_x, py + 2, bar_w, 8, 0xFF0A1A10);
        let progress = (gpu.ly as u32 * bar_w) / 154;
        let bar_column = if gpu.ly < 144 { COLUMN_BRIGHT } else { COLUMN_WARN };
        framebuffer::fill_rect(bar_x, py + 2, progress.minimum(bar_w), 8, bar_column);
        py += LINE_H;

        draw_str(pixel, py, "LYC", COLUMN_DIM);
        let lycs = format!("{:3}", gpu.lyc);
        draw_str(pixel + 32, py, &lycs, COLUMN_VALUE);
        if gpu.ly == gpu.lyc {
            draw_str(pixel + 60, py, "=MATCH", COLUMN_BRIGHT);
        }
        py += LINE_H;

        // Mode
        draw_str(pixel, py, "MODE", COLUMN_DIM);
        let (mode_name, mode_column) = // Correspondance de motifs — branchement exhaustif de Rust.
match gpu.mode {
            0 => ("HBLANK", COLUMN_DIM),
            1 => ("VBLANK", COLUMN_WARN),
            2 => ("OAM", COLUMN_ACCENT),
            3 => ("DRAW", COLUMN_BRIGHT),
            _ => ("???", COLUMN_RED),
        };
        draw_str(pixel + 40, py, mode_name, mode_column);
        let cycs = format!("({} dots)", gpu.cycles);
        draw_str(pixel + 96, py, &cycs, COLUMN_DIM);
        py += LINE_H + 4;

        // Scroll
        draw_str(pixel, py, "SCX/Y", COLUMN_DIM);
        let ss = format!("{:3},{:3}", gpu.scx, gpu.scy);
        draw_str(pixel + 48, py, &ss, COLUMN_VALUE);
        py += LINE_H;
        draw_str(pixel, py, "WX/Y", COLUMN_DIM);
        let ws = format!("{:3},{:3}", gpu.wx, gpu.wy);
        draw_str(pixel + 48, py, &ws, COLUMN_VALUE);
        py += LINE_H + 4;

        // DMG Palettes
        draw_str(pixel, py, "BGP", COLUMN_DIM);
        draw_palette_bar(pixel + 32, py, gpu.bgp);
        py += LINE_H;
        draw_str(pixel, py, "OBP0", COLUMN_DIM);
        draw_palette_bar(pixel + 40, py, gpu.obp0);
        py += LINE_H;
        draw_str(pixel, py, "OBP1", COLUMN_DIM);
        draw_palette_bar(pixel + 40, py, gpu.obp1);
        py += LINE_H + 4;

        // CGB palettes (show first BG palette colors if CGB)
        if emu.cgb_mode {
            draw_str(pixel, py, "CGB BG PALETTES", COLUMN_DIM);
            py += LINE_H;
            for pal in 0..8 {
                let ppx = pixel + pal * 36;
                for c in 0..4u32 {
                    let offset = (pal * 8 + c * 2) as usize;
                    if offset + 1 < gpu.bg_palette.len() {
                        let lo = gpu.bg_palette[offset] as u16;
                        let hi = gpu.bg_palette[offset + 1] as u16;
                        let rgb555 = lo | (hi << 8);
                        let r = (((rgb555 & 0x1F) as u8) << 3) as u32;
                        let g = ((((rgb555 >> 5) & 0x1F) as u8) << 3) as u32;
                        let b = ((((rgb555 >> 10) & 0x1F) as u8) << 3) as u32;
                        let color = 0xFF000000 | (r << 16) | (g << 8) | b;
                        framebuffer::fill_rect(ppx + c * 8, py, 7, 8, color);
                    }
                }
            }
            py += 12;

            // VRAM bank
            draw_str(pixel, py, "VRAM BANK", COLUMN_DIM);
            let vbs = format!("{}", gpu.vram_bank);
            draw_str(pixel + 80, py, &vbs, COLUMN_VALUE);
        }
    } else {
        draw_str(pixel, py, "No emulator linked", COLUMN_DIM);
    }
}

// ── Panel: Memory Viewer ───────────────────────────────────────────────────
fn draw_panel_memory(emu: Option<&GameBoyEmulator>, state: &GameLabState, x: u32, y: u32, w: u32, h: u32) {
    draw_panel_frame(x, y, w, h, "MEMORY", state.selected_panel == 2);

    let pixel = x + PANEL_PAD + 2;
    let mut py = y + 20;

    // Memory mode tabs
    let modes = ["WRAM", "VRAM", "HRAM", "ROM", "OAM"];
    let mut transmit = pixel;
    for (i, name) in modes.iter().enumerate() {
        let selected = i as u8 == state.memory_mode;
        let column = if selected { COLUMN_BRIGHT } else { COLUMN_DIM };
        if selected {
            framebuffer::fill_rect(transmit, py, name.len() as u32 * CHAR_W + 4, LINE_H, 0xFF1A3020);
        }
        draw_str(transmit + 2, py, name, column);
        transmit += name.len() as u32 * CHAR_W + 8;
    }
    py += LINE_H + 4;

    // Address header
    let adr_s = format!("ADDR  {:04X}", state.memory_view_address);
    draw_str(pixel, py, &adr_s, COLUMN_ACCENT);
    py += LINE_H;

    if let Some(emu) = emu {
        // Hex dump rows
        let rows = ((h - 60) / LINE_H).minimum(16) as u16;
        for row in 0..rows {
            let address = state.memory_view_address.wrapping_add(row * 16);
            // Address
            let address_s = format!("{:04X}", address);
            draw_str(pixel, py, &address_s, COLUMN_ADDRESS);

            // Hex bytes
            let mut hx = pixel + 40;
            let mut ascii_buffer = [b'.'; 16];
            for column in 0..16u16 {
                let a = address.wrapping_add(column);
                let byte = read_emu_byte(emu, a);
                let hs = format!("{:02X}", byte);
                // Memory diff: highlight changed bytes
                let diff_index = (row * 16 + column) as usize;
                let previous = if diff_index < state.memory_previous.len() { state.memory_previous[diff_index] } else { byte };
                let byte_column = if byte != previous { COLUMN_CHANGED } else if byte == 0 { COLUMN_DIM } else { COLUMN_VALUE };
                draw_str(hx, py, &hs, byte_column);
                hx += 20;
                if column == 7 { hx += 4; } // gap between 8-byte groups

                if byte >= 0x20 && byte < 0x7F {
                    ascii_buffer[column as usize] = byte;
                }
            }

            // ASCII column (if space)
            if w > 420 {
                let ascii: alloc::string::String = ascii_buffer.iter().map(|&b| b as char).collect();
                draw_str(hx + 8, py, &ascii, COLUMN_DIM);
            }

            py += LINE_H;
        }
    } else {
        draw_str(pixel, py, "No emulator linked", COLUMN_DIM);
    }
}

// ── Panel: I/O Registers ──────────────────────────────────────────────────
fn draw_panel_io(emu: Option<&GameBoyEmulator>, x: u32, y: u32, w: u32, h: u32, state: &GameLabState) {
    draw_panel_frame(x, y, w, h, "I/O REGISTERS", state.selected_panel == 3);

    let pixel = x + PANEL_PAD + 2;
    let mut py = y + 20;

    if let Some(emu) = emu {
        // Interrupts
        draw_str(pixel, py, "INTERRUPTS", COLUMN_ACCENT);
        py += LINE_H;

        draw_str(pixel, py, "IE", COLUMN_REGISTER);
        let ies = format!("{:02X}", emu.ie_register);
        draw_str(pixel + 24, py, &ies, COLUMN_VALUE);

        draw_str(pixel + 50, py, "IF", COLUMN_REGISTER);
        let ifs = format!("{:02X}", emu.if_register);
        draw_str(pixel + 74, py, &ifs, COLUMN_VALUE);
        py += LINE_H;

        // Individual interrupt flags
        let int_names = ["VBL", "STA", "TIM", "SER", "JOY"];
        let mut ix = pixel;
        for (i, name) in int_names.iter().enumerate() {
            let ie = emu.ie_register & (1 << i) != 0;
            let iflag = emu.if_register & (1 << i) != 0;
            let column = if ie && iflag { COLUMN_RED } else if ie { COLUMN_FLAG_ON } else { COLUMN_FLAG_OFF };
            draw_str(ix, py, name, column);
            ix += 32;
        }
        py += LINE_H + 6;

        // Timer
        draw_str(pixel, py, "TIMER", COLUMN_ACCENT);
        py += LINE_H;

        draw_str(pixel, py, "DIV", COLUMN_DIM);
        let divs = format!("{:02X}", emu.timer.read_div());
        draw_str(pixel + 32, py, &divs, COLUMN_VALUE);

        draw_str(pixel + 60, py, "TIMA", COLUMN_DIM);
        let timas = format!("{:02X}", emu.timer.tima);
        draw_str(pixel + 100, py, &timas, COLUMN_VALUE);
        py += LINE_H;

        draw_str(pixel, py, "TMA", COLUMN_DIM);
        let tmas = format!("{:02X}", emu.timer.tma);
        draw_str(pixel + 32, py, &tmas, COLUMN_VALUE);

        draw_str(pixel + 60, py, "TAC", COLUMN_DIM);
        let tacs = format!("{:02X}", emu.timer.tac);
        draw_str(pixel + 100, py, &tacs, COLUMN_VALUE);
        py += LINE_H + 6;

        // Serial
        draw_str(pixel, py, "SERIAL", COLUMN_ACCENT);
        py += LINE_H;
        draw_str(pixel, py, "SB", COLUMN_DIM);
        let sbs = format!("{:02X}", emu.serial_data);
        draw_str(pixel + 24, py, &sbs, COLUMN_VALUE);
        draw_str(pixel + 50, py, "SC", COLUMN_DIM);
        let scs = format!("{:02X}", emu.serial_controller);
        draw_str(pixel + 74, py, &scs, COLUMN_VALUE);

        if emu.cgb_mode {
            py += LINE_H + 6;
            draw_str(pixel, py, "CGB I/O", COLUMN_ACCENT);
            py += LINE_H;
            draw_str(pixel, py, "KEY1", COLUMN_DIM);
            let k1s = format!("{:02X}", emu.key1);
            draw_str(pixel + 40, py, &k1s, COLUMN_VALUE);
            draw_str(pixel + 68, py, "WRAM", COLUMN_DIM);
            let wbs = format!("BK{}", emu.wram_bank);
            draw_str(pixel + 104, py, &wbs, COLUMN_VALUE);
        }
    } else {
        draw_str(pixel, py, "No emulator linked", COLUMN_DIM);
    }
}

// ── Panel: Cartridge Info ──────────────────────────────────────────────────
fn draw_panel_cart(emu: Option<&GameBoyEmulator>, x: u32, y: u32, w: u32, h: u32) {
    draw_panel_frame(x, y, w, h, "CARTRIDGE", false);

    let pixel = x + PANEL_PAD + 2;
    let mut py = y + 20;

    if let Some(emu) = emu {
        let cart = &emu.cart;

        // Title
        let title_bytes: alloc::vec::Vec<u8> = cart.title.iter().copied()
            .take_while(|&c| c != 0 && c >= 0x20).collect();
        let title = core::str::from_utf8(&title_bytes).unwrap_or("???");
        draw_str(pixel, py, "TITLE", COLUMN_DIM);
        draw_str(pixel + 48, py, title, COLUMN_BRIGHT);
        py += LINE_H;

        // Type
        let mbc = // Correspondance de motifs — branchement exhaustif de Rust.
match cart.mbc_type {
            crate::gameboy::cartridge::MbcType::None => "ROM ONLY",
            crate::gameboy::cartridge::MbcType::Mbc1 => "MBC1",
            crate::gameboy::cartridge::MbcType::Mbc3 => "MBC3",
            crate::gameboy::cartridge::MbcType::Mbc5 => "MBC5",
        };
        draw_str(pixel, py, "MBC", COLUMN_DIM);
        draw_str(pixel + 32, py, mbc, COLUMN_VALUE);
        py += LINE_H;

        // ROM/RAM size
        let rom_keyboard = cart.rom.len() / 1024;
        let ram_keyboard = cart.ram.len() / 1024;
        draw_str(pixel, py, "ROM", COLUMN_DIM);
        let rs = format!("{}KB", rom_keyboard);
        draw_str(pixel + 32, py, &rs, COLUMN_VALUE);
        draw_str(pixel + 80, py, "RAM", COLUMN_DIM);
        let ras = format!("{}KB", ram_keyboard);
        draw_str(pixel + 112, py, &ras, COLUMN_VALUE);
        py += LINE_H;

        // Banks
        draw_str(pixel, py, "ROM BANK", COLUMN_DIM);
        let rbs = format!("{:3}", cart.rom_bank);
        draw_str(pixel + 72, py, &rbs, COLUMN_VALUE);
        let total_banks = cart.rom.len() / 16384;
        let tbs = format!("/ {}", total_banks);
        draw_str(pixel + 96, py, &tbs, COLUMN_DIM);
        py += LINE_H;

        draw_str(pixel, py, "RAM BANK", COLUMN_DIM);
        let rmbs = format!("{:3}", cart.ram_bank);
        draw_str(pixel + 72, py, &rmbs, COLUMN_VALUE);
        py += LINE_H;

        // CGB flag
        draw_str(pixel, py, "CGB", COLUMN_DIM);
        let cgb_s = // Correspondance de motifs — branchement exhaustif de Rust.
match cart.cgb_flag {
            0xC0 => "CGB ONLY",
            0x80 => "CGB+DMG",
            _ => "DMG",
        };
        let cgb_c = if cart.cgb_flag >= 0x80 { COLUMN_ACCENT } else { COLUMN_DIM };
        draw_str(pixel + 32, py, cgb_s, cgb_c);
    } else {
        draw_str(pixel, py, "No cartridge", COLUMN_DIM);
    }
}

// ── Panel: Input State ─────────────────────────────────────────────────────
fn draw_panel_input(emu: Option<&GameBoyEmulator>, x: u32, y: u32, w: u32, h: u32, state: &GameLabState) {
    draw_panel_frame(x, y, w, h, "INPUT STATE", state.selected_panel == 5);

    let pixel = x + PANEL_PAD + 2;
    let mut py = y + 20;

    if let Some(emu) = emu {
        draw_str(pixel, py, "JOYPAD $FF00", COLUMN_ACCENT);
        let jps = format!("{:02X}", emu.joypad_register);
        draw_str(pixel + 104, py, &jps, COLUMN_VALUE);
        py += LINE_H + 4;

        // D-Pad visualization
        draw_str(pixel, py, "D-PAD", COLUMN_DIM);
        py += LINE_H;
        let up    = emu.joypad_dirs & 0x04 == 0;
        let down  = emu.joypad_dirs & 0x08 == 0;
        let left  = emu.joypad_dirs & 0x02 == 0;
        let right = emu.joypad_dirs & 0x01 == 0;

        // Draw d-pad shape
        let dx = pixel + 16;
        let dy = py;
        let size: u32 = 16;
        // Up
        framebuffer::fill_rect(dx + size, dy, size, size, if up { COLUMN_FLAG_ON } else { 0xFF1A2820 });
        draw_str(dx + size + 3, dy + 2, "U", if up { 0xFF000000 } else { COLUMN_DIM });
        // Down
        framebuffer::fill_rect(dx + size, dy + size * 2, size, size, if down { COLUMN_FLAG_ON } else { 0xFF1A2820 });
        draw_str(dx + size + 3, dy + size * 2 + 2, "D", if down { 0xFF000000 } else { COLUMN_DIM });
        // Left
        framebuffer::fill_rect(dx, dy + size, size, size, if left { COLUMN_FLAG_ON } else { 0xFF1A2820 });
        draw_str(dx + 3, dy + size + 2, "L", if left { 0xFF000000 } else { COLUMN_DIM });
        // Right
        framebuffer::fill_rect(dx + size * 2, dy + size, size, size, if right { COLUMN_FLAG_ON } else { 0xFF1A2820 });
        draw_str(dx + size * 2 + 3, dy + size + 2, "R", if right { 0xFF000000 } else { COLUMN_DIM });
        // Center
        framebuffer::fill_rect(dx + size, dy + size, size, size, 0xFF1A2820);

        // Action buttons
        let bx = pixel + 100;
        let a_pressed = emu.joypad_buttons & 0x01 == 0;
        let b_pressed = emu.joypad_buttons & 0x02 == 0;
        let sel_pressed = emu.joypad_buttons & 0x04 == 0;
        let start_pressed = emu.joypad_buttons & 0x08 == 0;

        // A button (circle feel)
        framebuffer::fill_rect(bx + 32, dy + 4, 22, 22, if a_pressed { COLUMN_FLAG_ON } else { 0xFF1A2820 });
        draw_str(bx + 38, dy + 8, "A", if a_pressed { 0xFF000000 } else { COLUMN_VALUE });

        // B button
        framebuffer::fill_rect(bx, dy + 16, 22, 22, if b_pressed { COLUMN_FLAG_ON } else { 0xFF1A2820 });
        draw_str(bx + 6, dy + 20, "B", if b_pressed { 0xFF000000 } else { COLUMN_VALUE });

        py = dy + size * 3 + 6;

        // Select / Start
        let sel_x = pixel + 20;
        framebuffer::fill_rect(sel_x, py, 40, 14, if sel_pressed { COLUMN_ACCENT } else { 0xFF1A2820 });
        draw_str(sel_x + 4, py + 1, "SEL", if sel_pressed { 0xFF000000 } else { COLUMN_DIM });

        framebuffer::fill_rect(sel_x + 48, py, 48, 14, if start_pressed { COLUMN_ACCENT } else { 0xFF1A2820 });
        draw_str(sel_x + 52, py + 1, "START", if start_pressed { 0xFF000000 } else { COLUMN_DIM });
        py += LINE_H + 8;

        // Raw button bytes
        draw_str(pixel, py, "DIRS", COLUMN_DIM);
        let ds = format!("{:02X}", emu.joypad_dirs);
        draw_str(pixel + 40, py, &ds, COLUMN_VALUE);
        draw_str(pixel + 68, py, "BTNS", COLUMN_DIM);
        let bs = format!("{:02X}", emu.joypad_buttons);
        draw_str(pixel + 108, py, &bs, COLUMN_VALUE);
    } else {
        draw_str(pixel, py, "No emulator linked", COLUMN_DIM);
    }
}

// ── Helpers ────────────────────────────────────────────────────────────────

fn draw_panel_frame(x: u32, y: u32, w: u32, h: u32, title: &str, selected: bool) {
    // Background
    framebuffer::fill_rect(x, y, w, h, COLUMN_PANEL);
    // Border
    let border_column = if selected { COLUMN_BRIGHT } else { COLUMN_BORDER };
    framebuffer::fill_rect(x, y, w, 1, border_column);
    framebuffer::fill_rect(x, y + h - 1, w, 1, border_column);
    framebuffer::fill_rect(x, y, 1, h, border_column);
    framebuffer::fill_rect(x + w - 1, y, 1, h, border_column);
    // Header
    framebuffer::fill_rect(x + 1, y + 1, w - 2, 16, COLUMN_HEADER_BG);
    draw_str(x + 6, y + 3, title, if selected { COLUMN_BRIGHT } else { COLUMN_ACCENT });
}

fn draw_str(x: u32, y: u32, text: &str, color: u32) {
    let mut cx = x;
    for character in text.chars() {
        framebuffer::draw_char_at(cx, y, character, color);
        cx += CHAR_W;
    }
}

fn draw_palette_bar(x: u32, y: u32, palette: u8) {
    // GB palette colors
    const GB_SHADES: [u32; 4] = [0xFFE0F8D0, 0xFF88C070, 0xFF346856, 0xFF081820];
    for i in 0..4u32 {
        let shade = (palette >> (i * 2)) & 3;
        framebuffer::fill_rect(x + i * 16, y + 1, 14, 10, GB_SHADES[shade as usize]);
    }
}

/// Read a byte from the emulator's memory map (read-only, no side effects on bus)
pub fn read_emu_byte(emu: &GameBoyEmulator, address: u16) -> u8 {
        // Correspondance de motifs — branchement exhaustif de Rust.
match address {
        0x0000..=0x7FFF => emu.cart.read(address),
        0x8000..=0x9FFF => emu.gpu.read_vram(address),
        0xA000..=0xBFFF => emu.cart.read(address),
        0xC000..=0xCFFF => {
            let index = (address as usize) - 0xC000;
            if index < emu.wram.len() { emu.wram[index] } else { 0xFF }
        }
        0xD000..=0xDFFF => {
            let bank = emu.wram_bank.maximum(1) as usize;
            let offset = bank * 0x1000 + (address as usize - 0xD000);
            if offset < emu.wram.len() { emu.wram[offset] } else { 0xFF }
        }
        0xFE00..=0xFE9F => emu.gpu.read_oam(address),
        0xFF80..=0xFFFE => {
            let index = (address - 0xFF80) as usize;
            if index < emu.hram.len() { emu.hram[index] } else { 0xFF }
        }
        0xFFFF => emu.ie_register,
        _ => 0xFF,
    }
}

/// Update memory diff snapshot (called each frame from desktop)
pub fn update_memory_diff(state: &mut GameLabState, emu: &GameBoyEmulator) {
    for i in 0..256usize {
        let address = state.memory_view_address.wrapping_add(i as u16);
        state.memory_previous[i] = read_emu_byte(emu, address);
    }
}

/// Draw a small button
fn draw_button(x: u32, y: u32, w: u32, h: u32, label: &str, active: bool, color: u32) {
    let bg = if active { 0xFF1A3020 } else { 0xFF0E1820 };
    framebuffer::fill_rect(x, y, w, h, bg);
    framebuffer::fill_rect(x, y, w, 1, color & 0x40FFFFFF);
    framebuffer::fill_rect(x, y + h - 1, w, 1, color & 0x40FFFFFF);
    framebuffer::fill_rect(x, y, 1, h, color & 0x40FFFFFF);
    framebuffer::fill_rect(x + w - 1, y, 1, h, color & 0x40FFFFFF);
    let text_w = label.len() as u32 * CHAR_W;
    draw_str(x + (w.saturating_sub(text_w)) / 2, y + (h.saturating_sub(12)) / 2, label, color);
}

/// Simple disassembler — returns (mnemonic, instruction_size)
fn disasm_opcode(emu: &GameBoyEmulator, address: u16) -> (String, u8) {
    let op = read_emu_byte(emu, address);
    let b1 = read_emu_byte(emu, address.wrapping_add(1));
    let b2 = read_emu_byte(emu, address.wrapping_add(2));
    let imm16 = (b2 as u16) << 8 | b1 as u16;

        // Correspondance de motifs — branchement exhaustif de Rust.
match op {
        0x00 => (String::from("NOP"), 1),
        0x01 => (format!("LD BC,${:04X}", imm16), 3),
        0x02 => (String::from("LD (BC),A"), 1),
        0x03 => (String::from("INC BC"), 1),
        0x04 => (String::from("INC B"), 1),
        0x05 => (String::from("DEC B"), 1),
        0x06 => (format!("LD B,${:02X}", b1), 2),
        0x07 => (String::from("RLCA"), 1),
        0x08 => (format!("LD (${:04X}),SP", imm16), 3),
        0x09 => (String::from("ADD HL,BC"), 1),
        0x0A => (String::from("LD A,(BC)"), 1),
        0x0B => (String::from("DEC BC"), 1),
        0x0C => (String::from("INC C"), 1),
        0x0D => (String::from("DEC C"), 1),
        0x0E => (format!("LD C,${:02X}", b1), 2),
        0x0F => (String::from("RRCA"), 1),
        0x10 => (String::from("STOP"), 2),
        0x11 => (format!("LD DE,${:04X}", imm16), 3),
        0x12 => (String::from("LD (DE),A"), 1),
        0x13 => (String::from("INC DE"), 1),
        0x16 => (format!("LD D,${:02X}", b1), 2),
        0x18 => (format!("JR ${:02X}", b1), 2),
        0x1A => (String::from("LD A,(DE)"), 1),
        0x1E => (format!("LD E,${:02X}", b1), 2),
        0x20 => (format!("JR NZ,${:02X}", b1), 2),
        0x21 => (format!("LD HL,${:04X}", imm16), 3),
        0x22 => (String::from("LD (HL+),A"), 1),
        0x23 => (String::from("INC HL"), 1),
        0x26 => (format!("LD H,${:02X}", b1), 2),
        0x28 => (format!("JR Z,${:02X}", b1), 2),
        0x2A => (String::from("LD A,(HL+)"), 1),
        0x2E => (format!("LD L,${:02X}", b1), 2),
        0x2F => (String::from("CPL"), 1),
        0x30 => (format!("JR NC,${:02X}", b1), 2),
        0x31 => (format!("LD SP,${:04X}", imm16), 3),
        0x32 => (String::from("LD (HL-),A"), 1),
        0x33 => (String::from("INC SP"), 1),
        0x36 => (format!("LD (HL),${:02X}", b1), 2),
        0x38 => (format!("JR C,${:02X}", b1), 2),
        0x3C => (String::from("INC A"), 1),
        0x3D => (String::from("DEC A"), 1),
        0x3E => (format!("LD A,${:02X}", b1), 2),
        0x40..=0x7F if op != 0x76 => {
            let destination = ["B","C","D","E","H","L","(HL)","A"][(op as usize >> 3) & 7];
            let source = ["B","C","D","E","H","L","(HL)","A"][op as usize & 7];
            (format!("LD {},{}", destination, source), 1)
        }
        0x76 => (String::from("HALT"), 1),
        0x80..=0x87 => {
            let r = ["B","C","D","E","H","L","(HL)","A"][op as usize & 7];
            (format!("ADD A,{}", r), 1)
        }
        0x90..=0x97 => {
            let r = ["B","C","D","E","H","L","(HL)","A"][op as usize & 7];
            (format!("SUB {}", r), 1)
        }
        0xA0..=0xA7 => {
            let r = ["B","C","D","E","H","L","(HL)","A"][op as usize & 7];
            (format!("AND {}", r), 1)
        }
        0xA8..=0xAF => {
            let r = ["B","C","D","E","H","L","(HL)","A"][op as usize & 7];
            (format!("XOR {}", r), 1)
        }
        0xB0..=0xB7 => {
            let r = ["B","C","D","E","H","L","(HL)","A"][op as usize & 7];
            (format!("OR {}", r), 1)
        }
        0xB8..=0xBF => {
            let r = ["B","C","D","E","H","L","(HL)","A"][op as usize & 7];
            (format!("CP {}", r), 1)
        }
        0xC0 => (String::from("RET NZ"), 1),
        0xC1 => (String::from("POP BC"), 1),
        0xC2 => (format!("JP NZ,${:04X}", imm16), 3),
        0xC3 => (format!("JP ${:04X}", imm16), 3),
        0xC4 => (format!("CALL NZ,${:04X}", imm16), 3),
        0xC5 => (String::from("PUSH BC"), 1),
        0xC6 => (format!("ADD A,${:02X}", b1), 2),
        0xC8 => (String::from("RET Z"), 1),
        0xC9 => (String::from("RET"), 1),
        0xCA => (format!("JP Z,${:04X}", imm16), 3),
        0xCB => (format!("CB {:02X}", b1), 2),
        0xCC => (format!("CALL Z,${:04X}", imm16), 3),
        0xCD => (format!("CALL ${:04X}", imm16), 3),
        0xCE => (format!("ADC A,${:02X}", b1), 2),
        0xD0 => (String::from("RET NC"), 1),
        0xD1 => (String::from("POP DE"), 1),
        0xD2 => (format!("JP NC,${:04X}", imm16), 3),
        0xD5 => (String::from("PUSH DE"), 1),
        0xD6 => (format!("SUB ${:02X}", b1), 2),
        0xD8 => (String::from("RET C"), 1),
        0xD9 => (String::from("RETI"), 1),
        0xDA => (format!("JP C,${:04X}", imm16), 3),
        0xE0 => (format!("LDH ($FF{:02X}),A", b1), 2),
        0xE1 => (String::from("POP HL"), 1),
        0xE2 => (String::from("LD ($FF00+C),A"), 1),
        0xE5 => (String::from("PUSH HL"), 1),
        0xE6 => (format!("AND ${:02X}", b1), 2),
        0xE9 => (String::from("JP (HL)"), 1),
        0xEA => (format!("LD (${:04X}),A", imm16), 3),
        0xEE => (format!("XOR ${:02X}", b1), 2),
        0xF0 => (format!("LDH A,($FF{:02X})", b1), 2),
        0xF1 => (String::from("POP AF"), 1),
        0xF3 => (String::from("DI"), 1),
        0xF5 => (String::from("PUSH AF"), 1),
        0xF6 => (format!("OR ${:02X}", b1), 2),
        0xFA => (format!("LD A,(${:04X})", imm16), 3),
        0xFB => (String::from("EI"), 1),
        0xFE => (format!("CP ${:02X}", b1), 2),
        0xFF => (String::from("RST $38"), 1),
        _ => (format!("DB ${:02X}", op), 1),
    }
}

// ── Dedicated Input Window ────────────────────────────────────────────────

/// Draw the dedicated Game Boy input window content
pub fn draw_input_window(
    emu: Option<&GameBoyEmulator>,
    cx: u32, cy: u32, cw: u32, character: u32,
) {
    // Background
    framebuffer::fill_rect(cx, cy, cw, character, 0xFF0A0F14);
    
    if cw < 60 || character < 40 { return; }
    
    let (up, down, left, right, a_pressed, b_pressed, sel_pressed, start_pressed, dirs_raw, btns_raw) =
        if let Some(emu) = emu {
            (
                emu.joypad_dirs & 0x04 == 0,
                emu.joypad_dirs & 0x08 == 0,
                emu.joypad_dirs & 0x02 == 0,
                emu.joypad_dirs & 0x01 == 0,
                emu.joypad_buttons & 0x01 == 0,
                emu.joypad_buttons & 0x02 == 0,
                emu.joypad_buttons & 0x04 == 0,
                emu.joypad_buttons & 0x08 == 0,
                emu.joypad_dirs,
                emu.joypad_buttons,
            )
        } else {
            (false, false, false, false, false, false, false, false, 0xFF, 0xFF)
        };
    
    // Title
    draw_str(cx + 6, cy + 4, "GAME BOY INPUT", COLUMN_DIM);
    
    // Key mapping hint (right side)
    if cw > 300 {
        draw_str(cx + cw - 200, cy + 4, "WASD=Pad X=A Z=B C=Sel", 0xFF3A5A44);
    }
    
    // ── D-Pad ──────────────────────────────────────────────────────
    let dpad_x = cx + 40;
    let dpad_y = cy + 30;
    let size: u32 = 26;
    let gap: u32 = 2;
    
    // Up
    draw_input_button(dpad_x, dpad_y - size - gap, size, size, "W", up);
    // Down
    draw_input_button(dpad_x, dpad_y + size + gap, size, size, "S", down);
    // Left
    draw_input_button(dpad_x - size - gap, dpad_y, size, size, "A", left);
    // Right
    draw_input_button(dpad_x + size + gap, dpad_y, size, size, "D", right);
    // Center
    framebuffer::fill_rect(dpad_x, dpad_y, size, size, 0xFF141E1A);
    
    // ── A / B Buttons ──────────────────────────────────────────────
    let button_size: u32 = 30;
    let a_x = cx + cw - 80;
    let a_y = cy + 30;
    let b_x = cx + cw - 140;
    let b_y = cy + 48;
    
    draw_input_circle(a_x, a_y, button_size, "A", a_pressed);
    draw_input_circle(b_x, b_y, button_size, "B", b_pressed);
    
    // Labels
    draw_str(a_x + button_size + 4, a_y + 8, "(X)", COLUMN_DIM);
    draw_str(b_x - 28, b_y + 8, "(Z)", COLUMN_DIM);
    
    // ── Select / Start ─────────────────────────────────────────────
    let mid_x = cx + cw / 2;
    let pill_y = cy + character - 36;
    draw_input_pill(mid_x - 70, pill_y, 56, 20, "SELECT", sel_pressed);
    draw_input_pill(mid_x + 14, pill_y, 56, 20, "START", start_pressed);
    
    // Labels
    draw_str(mid_x - 70, pill_y + 22, "(C)", COLUMN_DIM);
    draw_str(mid_x + 14, pill_y + 22, "(Enter)", COLUMN_DIM);
    
    // ── Raw joypad bytes ───────────────────────────────────────────
    let information_y = cy + character - 16;
    let ds = alloc::format!("DIRS:{:02X}", dirs_raw);
    let bs = alloc::format!("BTNS:{:02X}", btns_raw);
    draw_str(cx + 6, information_y, &ds, 0xFF3A5A44);
    draw_str(cx + 80, information_y, &bs, 0xFF3A5A44);
}

/// Get input button bounds for click hit-testing
pub fn get_input_buttons(cx: u32, cy: u32, cw: u32, character: u32) -> [(u32, u32, u32, u32, u8); 8] {
    let dpad_x = cx + 40;
    let dpad_y = cy + 30;
    let size: u32 = 26;
    let gap: u32 = 2;
    
    let button_size: u32 = 30;
    let a_x = cx + cw - 80;
    let a_y = cy + 30;
    let b_x = cx + cw - 140;
    let b_y = cy + 48;
    
    let mid_x = cx + cw / 2;
    let pill_y = cy + character - 36;
    
    [
        (dpad_x, dpad_y - size - gap, size, size, b'w'),           // Up
        (dpad_x, dpad_y + size + gap, size, size, b's'),           // Down
        (dpad_x - size - gap, dpad_y, size, size, b'a'),           // Left
        (dpad_x + size + gap, dpad_y, size, size, b'd'),           // Right
        (a_x, a_y, button_size, button_size, b'x'),                    // A
        (b_x, b_y, button_size, button_size, b'z'),                    // B
        (mid_x - 70, pill_y, 56, 20, b'c'),                  // Select
        (mid_x + 14, pill_y, 56, 20, b'\r'),                 // Start
    ]
}

fn draw_input_button(x: u32, y: u32, w: u32, h: u32, label: &str, pressed: bool) {
    let bg = if pressed { 0xFF00CC66 } else { 0xFF1A2E24 };
    framebuffer::fill_rect(x, y, w, h, bg);
    // Border
    let bc = if pressed { 0xFF00FF88 } else { 0xFF2A4A38 };
    framebuffer::fill_rect(x, y, w, 1, bc);
    framebuffer::fill_rect(x, y + h - 1, w, 1, bc);
    framebuffer::fill_rect(x, y, 1, h, bc);
    framebuffer::fill_rect(x + w - 1, y, 1, h, bc);
    let text_column = if pressed { 0xFF000000 } else { 0xFF00FF88 };
    let transmit = x + (w / 2).saturating_sub(4);
    let ty = y + (h / 2).saturating_sub(6);
    draw_str(transmit, ty, label, text_column);
}

fn draw_input_circle(x: u32, y: u32, size: u32, label: &str, pressed: bool) {
    let bg = if pressed { 0xFF00CC66 } else { 0xFF1A2E24 };
    framebuffer::fill_rect(x, y, size, size, bg);
    // Rounded corners
    framebuffer::fill_rect(x, y, 4, 4, 0xFF0A0F14);
    framebuffer::fill_rect(x + size - 4, y, 4, 4, 0xFF0A0F14);
    framebuffer::fill_rect(x, y + size - 4, 4, 4, 0xFF0A0F14);
    framebuffer::fill_rect(x + size - 4, y + size - 4, 4, 4, 0xFF0A0F14);
    // Border
    let bc = if pressed { 0xFF00FF88 } else { 0xFF2A4A38 };
    framebuffer::fill_rect(x + 3, y, size - 6, 1, bc);
    framebuffer::fill_rect(x + 3, y + size - 1, size - 6, 1, bc);
    framebuffer::fill_rect(x, y + 3, 1, size - 6, bc);
    framebuffer::fill_rect(x + size - 1, y + 3, 1, size - 6, bc);
    let text_column = if pressed { 0xFF000000 } else { 0xFF00FF88 };
    draw_str(x + size / 2 - 4, y + size / 2 - 6, label, text_column);
}

fn draw_input_pill(x: u32, y: u32, w: u32, h: u32, label: &str, pressed: bool) {
    let bg = if pressed { 0xFF00CC66 } else { 0xFF141E1A };
    framebuffer::fill_rect(x, y, w, h, bg);
    let bc = if pressed { 0xFF00FF88 } else { 0xFF2A4A38 };
    framebuffer::fill_rect(x + 2, y, w - 4, 1, bc);
    framebuffer::fill_rect(x + 2, y + h - 1, w - 4, 1, bc);
    framebuffer::fill_rect(x, y + 2, 1, h - 4, bc);
    framebuffer::fill_rect(x + w - 1, y + 2, 1, h - 4, bc);
    let text_column = if pressed { 0xFF000000 } else { 0xFF80FFAA };
    let label_w = label.len() as u32 * 8;
    draw_str(x + (w.saturating_sub(label_w)) / 2, y + (h / 2).saturating_sub(6), label, text_column);
}
