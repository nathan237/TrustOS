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
const PANEL_PAD: u32 = 4;
const LINE_H: u32 = 14;
const CHAR_W: u32 = 8;
const TOOLBAR_H: u32 = 24;

// ── Colors ─────────────────────────────────────────────────────────────────
const COL_BG: u32         = 0xFF0A0F14;
const COL_PANEL: u32      = 0xFF111920;
const COL_BORDER: u32     = 0xFF1E2A36;
const COL_HEADER_BG: u32  = 0xFF142028;
const COL_TEXT: u32       = 0xFF9CD8B0;   // Soft green
const COL_DIM: u32        = 0xFF4A6A54;   // Dim green
const COL_BRIGHT: u32     = 0xFF00FF88;   // Neon green
const COL_ACCENT: u32     = 0xFF58A6FF;   // Blue
const COL_VALUE: u32      = 0xFFE0F8D0;   // Light green (values)
const COL_REG: u32        = 0xFF80FFAA;   // Register names
const COL_FLAG_ON: u32    = 0xFF00FF66;   // Flag active
const COL_FLAG_OFF: u32   = 0xFF2A3A30;   // Flag inactive
const COL_WARN: u32       = 0xFFD29922;   // Yellow/warning
const COL_RED: u32        = 0xFFF85149;   // Red
const COL_CYAN: u32       = 0xFF79C0FF;   // Cyan
const COL_PURPLE: u32     = 0xFFBC8CFF;   // Purple
const COL_ADDR: u32       = 0xFF507060;   // Address color in hex dump
const COL_CHANGED: u32    = 0xFFFF4444;   // Red for changed bytes
const COL_TOOLBAR: u32    = 0xFF0E1820;

const MAX_WATCH: usize = 16;
const MAX_SEARCH_RESULTS: usize = 256;
const TRACE_SIZE: usize = 64;
const MAX_BREAKPOINTS: usize = 8;

// ── Tab modes ──────────────────────────────────────────────────────────────
#[derive(Clone, Copy, PartialEq)]
pub enum LabTab {
    Analyze = 0,
    Search = 1,
    Watch = 2,
    Tiles = 3,
    Trace = 4,
}

// ── Watch entry ────────────────────────────────────────────────────────────
#[derive(Clone)]
pub struct WatchEntry {
    pub addr: u16,
    pub label: [u8; 8],    // short label (ASCII)
    pub label_len: u8,
    pub prev_value: u8,
    pub cur_value: u8,
    pub changed: bool,
    pub size: u8,           // 1=byte, 2=word
}

// ── Search state ───────────────────────────────────────────────────────────
#[derive(Clone, Copy, PartialEq)]
pub enum SearchMode {
    Exact,      // Search for exact value
    Changed,    // Value changed since last scan
    Unchanged,  // Value didn't change
    Greater,    // Value increased
    Less,       // Value decreased
}

// ── Trace entry ────────────────────────────────────────────────────────────
#[derive(Clone, Copy)]
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
    pub ie_reg: u8, pub if_reg: u8,
    pub joypad_reg: u8,
    pub joypad_buttons: u8, pub joypad_dirs: u8,
    pub serial_data: u8, pub serial_ctrl: u8,
    pub wram_bank: u8, pub key1: u8,
    pub wram: Vec<u8>,
    pub hram: [u8; 127],
    pub gpu_lcdc: u8, pub gpu_stat: u8,
    pub gpu_scy: u8, pub gpu_scx: u8,
    pub gpu_ly: u8, pub gpu_lyc: u8,
    pub gpu_bgp: u8, pub gpu_obp0: u8, pub gpu_obp1: u8,
    pub gpu_wy: u8, pub gpu_wx: u8,
    pub gpu_mode: u8, pub gpu_cycles: u32,
    pub gpu_vram: [u8; 8192],
    pub gpu_vram1: [u8; 8192],
    pub gpu_oam: [u8; 160],
    pub gpu_bg_palette: [u8; 64],
    pub gpu_obj_palette: [u8; 64],
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

impl SaveState {
    pub fn empty() -> Self {
        Self {
            cpu_a: 0, cpu_f: 0, cpu_b: 0, cpu_c: 0,
            cpu_d: 0, cpu_e: 0, cpu_h: 0, cpu_l: 0,
            cpu_sp: 0, cpu_pc: 0, cpu_ime: false, cpu_halted: false,
            ie_reg: 0, if_reg: 0, joypad_reg: 0,
            joypad_buttons: 0xF, joypad_dirs: 0xF,
            serial_data: 0, serial_ctrl: 0,
            wram_bank: 1, key1: 0,
            wram: Vec::new(), hram: [0; 127],
            gpu_lcdc: 0, gpu_stat: 0, gpu_scy: 0, gpu_scx: 0,
            gpu_ly: 0, gpu_lyc: 0, gpu_bgp: 0, gpu_obp0: 0, gpu_obp1: 0,
            gpu_wy: 0, gpu_wx: 0, gpu_mode: 0, gpu_cycles: 0,
            gpu_vram: [0; 8192], gpu_vram1: [0; 8192],
            gpu_oam: [0; 160],
            gpu_bg_palette: [0; 64], gpu_obj_palette: [0; 64],
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
    pub mem_view_addr: u16,
    /// Selected panel (0-5) in analyze mode
    pub selected_panel: u8,
    /// Frame counter for blinking
    pub frame: u32,
    /// Memory view mode: 0=WRAM, 1=VRAM, 2=HRAM, 3=ROM, 4=OAM
    pub mem_mode: u8,
    /// Scroll offset for panels
    pub scroll: u32,

    // ── Tab system ─────────────────────────────────────────────────
    pub active_tab: LabTab,

    // ── Watch List ─────────────────────────────────────────────────
    pub watches: Vec<WatchEntry>,

    // ── Memory Search ──────────────────────────────────────────────
    pub search_value: u16,         // Value to search for
    pub search_input: [u8; 6],     // Input buffer for typing
    pub search_input_len: u8,
    pub search_results: Vec<u16>,  // Matching addresses
    pub search_snapshot: Vec<u8>,  // WRAM snapshot for compare scans
    pub search_mode: SearchMode,
    pub search_active: bool,       // Has an initial scan been done?
    pub search_byte_mode: bool,    // true=byte(u8), false=word(u16)

    // ── Breakpoints + Stepping ─────────────────────────────────────
    pub breakpoints: Vec<u16>,     // PC addresses to break on
    pub bp_input: [u8; 5],        // Input buffer for adding BP
    pub bp_input_len: u8,
    pub paused: bool,
    pub step_one: bool,            // Execute exactly one instruction
    pub step_frame: bool,          // Execute one frame then pause

    // ── Speed Control ──────────────────────────────────────────────
    /// Speed multiplier index: 0=0.25x, 1=0.5x, 2=1x, 3=2x, 4=4x
    pub speed_idx: u8,

    // ── Trace Log ──────────────────────────────────────────────────
    pub trace: Vec<TraceEntry>,
    pub trace_enabled: bool,

    // ── Tile Viewer ────────────────────────────────────────────────
    pub tile_page: u8,   // 0=tiles $8000, 1=tiles $8800, 2=OAM sprites
    pub tile_scroll: u32,

    // ── Memory Diff ────────────────────────────────────────────────
    pub mem_prev: [u8; 256], // previous 256 bytes for diff highlighting

    // ── Save State ─────────────────────────────────────────────────
    pub save_state: SaveState,
}

impl GameLabState {
    pub fn new() -> Self {
        Self {
            linked_gb_id: None,
            mem_view_addr: 0xC000,
            selected_panel: 0,
            frame: 0,
            mem_mode: 0,
            scroll: 0,
            active_tab: LabTab::Analyze,
            watches: Vec::new(),
            search_value: 0,
            search_input: [0; 6],
            search_input_len: 0,
            search_results: Vec::new(),
            search_snapshot: Vec::new(),
            search_mode: SearchMode::Exact,
            search_active: false,
            search_byte_mode: true,
            breakpoints: Vec::new(),
            bp_input: [0; 5],
            bp_input_len: 0,
            paused: false,
            step_one: false,
            step_frame: false,
            speed_idx: 2, // 1x default
            trace: Vec::new(),
            trace_enabled: false,
            tile_page: 0,
            tile_scroll: 0,
            mem_prev: [0; 256],
            save_state: SaveState::empty(),
        }
    }

    pub fn speed_multiplier(&self) -> f32 {
        match self.speed_idx {
            0 => 0.25,
            1 => 0.5,
            2 => 1.0,
            3 => 2.0,
            4 => 4.0,
            _ => 1.0,
        }
    }

    pub fn speed_label(&self) -> &'static str {
        match self.speed_idx {
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
        s.ie_reg = emu.ie_reg; s.if_reg = emu.if_reg;
        s.joypad_reg = emu.joypad_reg;
        s.joypad_buttons = emu.joypad_buttons;
        s.joypad_dirs = emu.joypad_dirs;
        s.serial_data = emu.serial_data;
        s.serial_ctrl = emu.serial_ctrl;
        s.wram_bank = emu.wram_bank; s.key1 = emu.key1;
        s.wram = emu.wram.clone();
        s.hram = emu.hram;
        s.gpu_lcdc = emu.gpu.lcdc; s.gpu_stat = emu.gpu.stat;
        s.gpu_scy = emu.gpu.scy; s.gpu_scx = emu.gpu.scx;
        s.gpu_ly = emu.gpu.ly; s.gpu_lyc = emu.gpu.lyc;
        s.gpu_bgp = emu.gpu.bgp; s.gpu_obp0 = emu.gpu.obp0; s.gpu_obp1 = emu.gpu.obp1;
        s.gpu_wy = emu.gpu.wy; s.gpu_wx = emu.gpu.wx;
        s.gpu_mode = emu.gpu.mode; s.gpu_cycles = emu.gpu.cycles;
        s.gpu_vram = emu.gpu.vram;
        s.gpu_vram1 = emu.gpu.vram1;
        s.gpu_oam = emu.gpu.oam;
        s.gpu_bg_palette = emu.gpu.bg_palette;
        s.gpu_obj_palette = emu.gpu.obj_palette;
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
        emu.ie_reg = s.ie_reg; emu.if_reg = s.if_reg;
        emu.joypad_reg = s.joypad_reg;
        emu.joypad_buttons = s.joypad_buttons;
        emu.joypad_dirs = s.joypad_dirs;
        emu.serial_data = s.serial_data;
        emu.serial_ctrl = s.serial_ctrl;
        emu.wram_bank = s.wram_bank; emu.key1 = s.key1;
        if s.wram.len() == emu.wram.len() {
            emu.wram.copy_from_slice(&s.wram);
        }
        emu.hram = s.hram;
        emu.gpu.lcdc = s.gpu_lcdc; emu.gpu.stat = s.gpu_stat;
        emu.gpu.scy = s.gpu_scy; emu.gpu.scx = s.gpu_scx;
        emu.gpu.ly = s.gpu_ly; emu.gpu.lyc = s.gpu_lyc;
        emu.gpu.bgp = s.gpu_bgp; emu.gpu.obp0 = s.gpu_obp0; emu.gpu.obp1 = s.gpu_obp1;
        emu.gpu.wy = s.gpu_wy; emu.gpu.wx = s.gpu_wx;
        emu.gpu.mode = s.gpu_mode; emu.gpu.cycles = s.gpu_cycles;
        emu.gpu.vram = s.gpu_vram;
        emu.gpu.vram1 = s.gpu_vram1;
        emu.gpu.oam = s.gpu_oam;
        emu.gpu.bg_palette = s.gpu_bg_palette;
        emu.gpu.obj_palette = s.gpu_obj_palette;
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
        let val = self.search_value as u8;
        for (i, &b) in emu.wram.iter().enumerate() {
            if b == val {
                let addr = if i < 0x1000 { 0xC000 + i as u16 } else { 0xD000 + (i as u16 - 0x1000) };
                self.search_results.push(addr);
                if self.search_results.len() >= MAX_SEARCH_RESULTS { break; }
            }
        }
        self.take_search_snapshot(emu);
        self.search_active = true;
    }

    /// Filter scan: narrow existing results
    pub fn search_filter(&mut self, emu: &GameBoyEmulator) {
        if !self.search_active { return; }
        let new_results: Vec<u16> = self.search_results.iter().copied().filter(|&addr| {
            let cur = read_emu_byte(emu, addr);
            let prev = self.snapshot_byte(addr);
            match self.search_mode {
                SearchMode::Exact => cur == self.search_value as u8,
                SearchMode::Changed => cur != prev,
                SearchMode::Unchanged => cur == prev,
                SearchMode::Greater => cur > prev,
                SearchMode::Less => cur < prev,
            }
        }).collect();
        self.search_results = new_results;
        self.take_search_snapshot(emu);
    }

    fn snapshot_byte(&self, addr: u16) -> u8 {
        let idx = match addr {
            0xC000..=0xCFFF => (addr - 0xC000) as usize,
            0xD000..=0xDFFF => 0x1000 + (addr - 0xD000) as usize,
            _ => return 0xFF,
        };
        if idx < self.search_snapshot.len() { self.search_snapshot[idx] } else { 0xFF }
    }

    /// Add a watch from a search result address
    pub fn add_watch(&mut self, addr: u16) {
        if self.watches.len() >= MAX_WATCH { return; }
        if self.watches.iter().any(|w| w.addr == addr) { return; }
        let mut label = [0u8; 8];
        let s = format!("{:04X}", addr);
        for (i, b) in s.bytes().enumerate().take(8) { label[i] = b; }
        self.watches.push(WatchEntry {
            addr,
            label,
            label_len: s.len().min(8) as u8,
            prev_value: 0,
            cur_value: 0,
            changed: false,
            size: 1,
        });
    }

    /// Update all watch values
    pub fn update_watches(&mut self, emu: &GameBoyEmulator) {
        for w in self.watches.iter_mut() {
            w.prev_value = w.cur_value;
            w.cur_value = read_emu_byte(emu, w.addr);
            w.changed = w.cur_value != w.prev_value;
        }
    }

    pub fn tick(&mut self) {
        self.frame = self.frame.wrapping_add(1);
    }

    pub fn handle_key(&mut self, key: u8) {
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
            b',' => { if self.speed_idx > 0 { self.speed_idx -= 1; } }
            b'.' => { if self.speed_idx < 4 { self.speed_idx += 1; } }
            _ => {}
        }
    }

    fn handle_key_analyze(&mut self, key: u8) {
        match key {
            0x09 => { self.selected_panel = (self.selected_panel + 1) % 6; }
            0xF0 => { self.mem_view_addr = self.mem_view_addr.wrapping_sub(0x10); }
            0xF1 => { self.mem_view_addr = self.mem_view_addr.wrapping_add(0x10); }
            0xF2 => { self.mem_view_addr = self.mem_view_addr.wrapping_sub(0x100); }
            0xF3 => { self.mem_view_addr = self.mem_view_addr.wrapping_add(0x100); }
            b'1' => { self.mem_mode = 0; self.mem_view_addr = 0xC000; }
            b'2' => { self.mem_mode = 1; self.mem_view_addr = 0x8000; }
            b'3' => { self.mem_mode = 2; self.mem_view_addr = 0xFF80; }
            b'4' => { self.mem_mode = 3; self.mem_view_addr = 0x0000; }
            b'5' => { self.mem_mode = 4; self.mem_view_addr = 0xFE00; }
            _ => {}
        }
    }

    fn handle_key_search(&mut self, key: u8) {
        match key {
            b'0'..=b'9' | b'a'..=b'f' | b'A'..=b'F' => {
                if (self.search_input_len as usize) < 4 {
                    self.search_input[self.search_input_len as usize] = key;
                    self.search_input_len += 1;
                    // Parse hex
                    self.search_value = self.parse_search_hex();
                }
            }
            0x08 | 0x7F => { // Backspace
                if self.search_input_len > 0 {
                    self.search_input_len -= 1;
                    self.search_value = self.parse_search_hex();
                }
            }
            // Tab cycles search mode
            0x09 => {
                self.search_mode = match self.search_mode {
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
                self.search_input_len = 0;
                self.search_snapshot.clear();
            }
            _ => {}
        }
    }

    fn handle_key_watch(&mut self, key: u8) {
        match key {
            // Delete last watch
            0x08 | 0x7F => {
                self.watches.pop();
            }
            _ => {}
        }
    }

    fn handle_key_tiles(&mut self, key: u8) {
        match key {
            0x09 => { self.tile_page = (self.tile_page + 1) % 3; }
            0xF0 => { self.tile_scroll = self.tile_scroll.saturating_sub(1); }
            0xF1 => { self.tile_scroll = self.tile_scroll.saturating_add(1); }
            _ => {}
        }
    }

    fn handle_key_trace(&mut self, key: u8) {
        match key {
            b't' | b'T' => { self.trace_enabled = !self.trace_enabled; }
            b'r' | b'R' => { self.trace.clear(); }
            _ => {}
        }
    }

    fn parse_search_hex(&self) -> u16 {
        let mut val: u16 = 0;
        for i in 0..self.search_input_len as usize {
            let c = self.search_input[i];
            let digit = match c {
                b'0'..=b'9' => c - b'0',
                b'a'..=b'f' => c - b'a' + 10,
                b'A'..=b'F' => c - b'A' + 10,
                _ => 0,
            };
            val = (val << 4) | digit as u16;
        }
        val
    }

    pub fn handle_click(&mut self, rx: i32, ry: i32, ww: u32, _wh: u32) {
        // Header area: TITLE_BAR_H .. TITLE_BAR_H+22  (the "GAME LAB" bar)
        // Toolbar area: TITLE_BAR_H+22 .. TITLE_BAR_H+22+TOOLBAR_H  (tabs + speed + pause)
        let header_h = 22i32;
        let toolbar_y_start = TITLE_BAR_H as i32 + header_h;
        let toolbar_y_end = toolbar_y_start + TOOLBAR_H as i32;
        if ry >= toolbar_y_start && ry < toolbar_y_end {
            let tab_w = 72i32;
            let tx = rx - 4;
            if tx >= 0 {
                let tab_idx = tx / tab_w;
                match tab_idx {
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
            if rx >= speed_x && rx < speed_x + 24 {
                if self.speed_idx > 0 { self.speed_idx -= 1; }
            } else if rx >= speed_x + 28 && rx < speed_x + 52 {
                if self.speed_idx < 4 { self.speed_idx += 1; }
            }
            // Pause button
            let pause_x = ww as i32 - 130;
            if rx >= pause_x && rx < pause_x + 40 {
                self.paused = !self.paused;
            }
            // Step button
            let step_x = ww as i32 - 86;
            if rx >= step_x && rx < step_x + 36 {
                self.step_one = true; self.paused = true;
            }
            // Frame button
            let frame_x = ww as i32 - 46;
            if rx >= frame_x && rx < frame_x + 42 {
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
                let mx = rx - 8 - 48;
                if mx >= 0 {
                    // Each mode button: label_len*8 + 10 spacing
                    let modes_w: [i32; 5] = [5*8+10, 7*8+10, 4*8+10, 7*8+10, 4*8+10];
                    let mut accum = 0i32;
                    for (i, &w) in modes_w.iter().enumerate() {
                        if mx >= accum && mx < accum + w {
                            self.search_mode = match i {
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
            if ry >= list_y_start && rx >= (ww as i32 - 60) {
                let idx = ((ry - list_y_start) / LINE_H as i32) as usize;
                if idx < self.search_results.len() {
                    self.add_watch(self.search_results[idx]);
                }
            }
        }

        // Tiles tab: page selector clicks
        if self.active_tab == LabTab::Tiles {
            let page_y = content_y + 6 + LINE_H as i32;
            if ry >= page_y && ry < page_y + LINE_H as i32 {
                let px = rx - 8;
                if px >= 0 && px < 110 { self.tile_page = 0; }
                else if px >= 110 && px < 220 { self.tile_page = 1; }
                else if px >= 220 && px < 330 { self.tile_page = 2; }
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
    let ch = wh.saturating_sub(TITLE_BAR_H);

    if cw < 200 || ch < 150 { return; }

    // Background
    framebuffer::fill_rect(cx, cy, cw, ch, COL_BG);

    // ── Header bar ─────────────────────────────────────────────────
    framebuffer::fill_rect(cx, cy, cw, 22, COL_HEADER_BG);
    let blink = (state.frame / 15) % 2 == 0;
    let dot_color = if blink { COL_BRIGHT } else { COL_DIM };
    framebuffer::fill_rect(cx + 6, cy + 8, 6, 6, dot_color);
    draw_str(cx + 16, cy + 4, "GAME LAB", COL_BRIGHT);

    // Linked status
    if emu.is_some() {
        draw_str(cx + 100, cy + 4, "[LINKED]", COL_BRIGHT);
    } else {
        draw_str(cx + 100, cy + 4, "[NO EMU]", COL_RED);
    }

    // Save/Load state buttons
    let save_x = cx + cw - 120;
    draw_btn(save_x, cy + 2, 48, 16, "SAVE", state.save_state.valid, COL_ACCENT);
    draw_btn(save_x + 54, cy + 2, 48, 16, "LOAD", state.save_state.valid, if state.save_state.valid { COL_BRIGHT } else { COL_DIM });

    // ── Toolbar with tabs + speed/pause controls ───────────────────
    let ty = cy + 22;
    framebuffer::fill_rect(cx, ty, cw, TOOLBAR_H, COL_TOOLBAR);
    framebuffer::fill_rect(cx, ty + TOOLBAR_H - 1, cw, 1, COL_BORDER);

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
        let tx = cx + 4 + i as u32 * (tab_w + 4);
        let active = state.active_tab == *tab;
        let bg = if active { 0xFF1A3828 } else { COL_TOOLBAR };
        framebuffer::fill_rect(tx, ty + 2, tab_w, TOOLBAR_H - 4, bg);
        if active {
            framebuffer::fill_rect(tx, ty + TOOLBAR_H - 3, tab_w, 2, COL_BRIGHT);
        }
        let col = if active { COL_BRIGHT } else { COL_DIM };
        draw_str(tx + 4, ty + 6, label, col);
    }

    // Speed controls (right side)
    let speed_x = cx + cw - 200;
    draw_btn(speed_x, ty + 3, 22, 16, "<", false, COL_ACCENT);
    draw_str(speed_x + 26, ty + 6, state.speed_label(), COL_VALUE);
    draw_btn(speed_x + 56, ty + 3, 22, 16, ">", false, COL_ACCENT);

    // Pause/Step buttons
    let pause_x = cx + cw - 130;
    let pause_col = if state.paused { COL_RED } else { COL_BRIGHT };
    let pause_label = if state.paused { "PLAY" } else { "PAUS" };
    draw_btn(pause_x, ty + 3, 38, 16, pause_label, state.paused, pause_col);
    draw_btn(pause_x + 42, ty + 3, 34, 16, "STEP", false, COL_CYAN);
    draw_btn(pause_x + 80, ty + 3, 42, 16, "FRAME", false, COL_PURPLE);

    // ── Content area ───────────────────────────────────────────────
    let content_y = ty + TOOLBAR_H;
    let content_h = ch.saturating_sub(22 + TOOLBAR_H);

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

fn draw_tab_analyze(state: &GameLabState, emu: Option<&GameBoyEmulator>, cx: u32, cy: u32, cw: u32, ch: u32) {
    let top_h = ch * 60 / 100;
    let bot_h = ch - top_h - 2;
    let col_w = (cw - 4) / 3;

    draw_panel_cpu(emu, cx + 1, cy, col_w, top_h, state);
    draw_panel_gpu(emu, cx + col_w + 2, cy, col_w, top_h, state);
    draw_panel_memory(emu, state, cx + col_w * 2 + 3, cy, col_w, top_h);

    let by = cy + top_h + 2;
    draw_panel_io(emu, cx + 1, by, col_w, bot_h, state);
    draw_panel_cart(emu, cx + col_w + 2, by, col_w, bot_h);
    draw_panel_input(emu, cx + col_w * 2 + 3, by, col_w, bot_h, state);
}

// ═══════════════════════════════════════════════════════════════════════════
// TAB: SEARCH (Cheat Engine style memory scanner)
// ═══════════════════════════════════════════════════════════════════════════

fn draw_tab_search(state: &GameLabState, emu: Option<&GameBoyEmulator>, cx: u32, cy: u32, cw: u32, ch: u32) {
    let px = cx + 8;
    let mut py = cy + 6;

    // Title
    draw_str(px, py, "MEMORY SEARCH", COL_ACCENT);
    draw_str(px + 120, py, "(Hex value, Tab=mode, R=reset)", COL_DIM);
    py += LINE_H + 4;

    // Search input
    draw_str(px, py, "VALUE:", COL_REG);
    let mut input_str = String::new();
    for i in 0..state.search_input_len as usize {
        input_str.push(state.search_input[i] as char);
    }
    if input_str.is_empty() { input_str.push_str("__"); }
    // Blinking cursor
    if (state.frame / 20) % 2 == 0 { input_str.push('_'); }
    draw_str(px + 52, py, &input_str, COL_VALUE);

    // Parsed value
    let vs = format!("= {} (0x{:02X})", state.search_value, state.search_value);
    draw_str(px + 120, py, &vs, COL_DIM);
    py += LINE_H + 2;

    // Search mode indicator
    draw_str(px, py, "MODE:", COL_REG);
    let modes = [
        ("EXACT", SearchMode::Exact),
        ("CHANGED", SearchMode::Changed),
        ("SAME", SearchMode::Unchanged),
        ("GREATER", SearchMode::Greater),
        ("LESS", SearchMode::Less),
    ];
    let mut mx = px + 48;
    for (label, mode) in &modes {
        let active = state.search_mode == *mode;
        let col = if active { COL_BRIGHT } else { COL_DIM };
        if active { framebuffer::fill_rect(mx - 2, py - 1, label.len() as u32 * CHAR_W + 4, LINE_H, 0xFF1A3020); }
        draw_str(mx, py, label, col);
        mx += label.len() as u32 * CHAR_W + 10;
    }
    py += LINE_H + 2;

    // Action buttons hint
    draw_str(px, py, "Enter=Scan/Filter", COL_ACCENT);
    draw_str(px + 152, py, "R=Reset", COL_WARN);
    py += LINE_H + 6;

    // Status
    let status = if !state.search_active {
        String::from("No scan yet. Type value + Enter to scan WRAM.")
    } else {
        format!("Results: {} addresses", state.search_results.len())
    };
    draw_str(px, py, &status, COL_TEXT);
    py += LINE_H + 4;

    // Results list
    if state.search_active {
        framebuffer::fill_rect(cx + 4, py, cw - 8, 1, COL_BORDER);
        py += 4;
        draw_str(px, py, "ADDR", COL_ACCENT);
        draw_str(px + 60, py, "VALUE", COL_ACCENT);
        draw_str(px + 110, py, "DEC", COL_ACCENT);
        if cw > 400 { draw_str(px + 160, py, "PREV", COL_DIM); }
        draw_str(cx + cw - 68, py, "[+WATCH]", COL_BRIGHT);
        py += LINE_H + 2;

        let max_rows = ((ch.saturating_sub(py - cy)) / LINE_H).min(32) as usize;
        for (i, &addr) in state.search_results.iter().take(max_rows).enumerate() {
            let val = if let Some(e) = emu { read_emu_byte(e, addr) } else { 0 };
            let prev = state.snapshot_byte(addr);
            let changed = val != prev;

            let addr_s = format!("{:04X}", addr);
            draw_str(px, py, &addr_s, COL_ADDR);

            let val_s = format!("{:02X}", val);
            let val_col = if changed { COL_CHANGED } else { COL_VALUE };
            draw_str(px + 60, py, &val_s, val_col);

            let dec_s = format!("{:3}", val);
            draw_str(px + 110, py, &dec_s, COL_DIM);

            if cw > 400 {
                let prev_s = format!("{:02X}", prev);
                draw_str(px + 160, py, &prev_s, COL_DIM);
            }

            // Watch button
            let wb = cx + cw - 48;
            draw_str(wb, py, "+W", COL_BRIGHT);

            py += LINE_H;
            let _ = i;
        }
        if state.search_results.len() > max_rows {
            let more = format!("... +{} more", state.search_results.len() - max_rows);
            draw_str(px, py, &more, COL_DIM);
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TAB: WATCH (pinned memory addresses)
// ═══════════════════════════════════════════════════════════════════════════

fn draw_tab_watch(state: &GameLabState, emu: Option<&GameBoyEmulator>, cx: u32, cy: u32, cw: u32, ch: u32) {
    let px = cx + 8;
    let mut py = cy + 6;

    draw_str(px, py, "WATCH LIST", COL_ACCENT);
    let count_s = format!("{}/{}", state.watches.len(), MAX_WATCH);
    draw_str(px + 100, py, &count_s, COL_DIM);
    draw_str(px + 160, py, "(Backspace=remove last)", COL_DIM);
    py += LINE_H + 4;

    if state.watches.is_empty() {
        draw_str(px, py, "No watches. Add from Search tab with [+W] button.", COL_DIM);
        return;
    }

    // Header
    framebuffer::fill_rect(cx + 4, py, cw - 8, 1, COL_BORDER);
    py += 4;
    draw_str(px, py, "LABEL", COL_ACCENT);
    draw_str(px + 72, py, "ADDR", COL_ACCENT);
    draw_str(px + 120, py, "HEX", COL_ACCENT);
    draw_str(px + 160, py, "DEC", COL_ACCENT);
    draw_str(px + 200, py, "PREV", COL_ACCENT);
    if cw > 500 { draw_str(px + 250, py, "VISUAL", COL_DIM); }
    py += LINE_H + 2;

    for w in state.watches.iter() {
        let lbl: String = w.label[..w.label_len as usize].iter().map(|&b| b as char).collect();
        draw_str(px, py, &lbl, COL_REG);

        let addr_s = format!("{:04X}", w.addr);
        draw_str(px + 72, py, &addr_s, COL_ADDR);

        let val = if let Some(e) = emu { read_emu_byte(e, w.addr) } else { w.cur_value };
        let val_s = format!("{:02X}", val);
        let col = if w.changed { COL_CHANGED } else { COL_VALUE };
        draw_str(px + 120, py, &val_s, col);

        let dec_s = format!("{:3}", val);
        draw_str(px + 160, py, &dec_s, col);

        let prev_s = format!("{:02X}", w.prev_value);
        draw_str(px + 200, py, &prev_s, COL_DIM);

        // Visual bar
        if cw > 500 {
            let bar_w = 100u32;
            let fill = (val as u32 * bar_w) / 255;
            framebuffer::fill_rect(px + 250, py + 2, bar_w, 8, 0xFF0A1A10);
            let bar_col = if w.changed { COL_CHANGED } else { COL_BRIGHT };
            framebuffer::fill_rect(px + 250, py + 2, fill, 8, bar_col);
        }

        py += LINE_H;
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TAB: TILES (VRAM tile viewer + OAM sprites)
// ═══════════════════════════════════════════════════════════════════════════

fn draw_tab_tiles(state: &GameLabState, emu: Option<&GameBoyEmulator>, cx: u32, cy: u32, cw: u32, ch: u32) {
    let px = cx + 8;
    let mut py = cy + 6;

    let pages = ["TILES $8000", "TILES $8800", "OAM SPRITES"];
    draw_str(px, py, "TILE VIEWER", COL_ACCENT);
    py += LINE_H;

    // Page tabs
    for (i, label) in pages.iter().enumerate() {
        let active = i as u8 == state.tile_page;
        let col = if active { COL_BRIGHT } else { COL_DIM };
        let tx = px + i as u32 * 110;
        if active { framebuffer::fill_rect(tx - 2, py - 1, label.len() as u32 * CHAR_W + 4, LINE_H, 0xFF1A3020); }
        draw_str(tx, py, label, col);
    }
    draw_str(px + 340, py, "(Tab=page, Arrows=scroll)", COL_DIM);
    py += LINE_H + 6;

    let emu = match emu {
        Some(e) => e,
        None => { draw_str(px, py, "No emulator linked", COL_DIM); return; }
    };

    if state.tile_page < 2 {
        // Draw tiles as 16x16 grid (each tile = 8x8 pixels, drawn 2x scaled = 16x16)
        let base_addr: u16 = if state.tile_page == 0 { 0x8000 } else { 0x8800 };
        let tile_size = 2u32; // 2x scale
        let tile_px = 8 * tile_size;
        let cols = ((cw - 20) / (tile_px + 1)).min(16);
        let max_rows = ((ch - (py - cy) - 4) / (tile_px + 1)).min(16);
        let scroll = state.tile_scroll.min(16u32.saturating_sub(max_rows));

        for row in 0..max_rows {
            for col in 0..cols {
                let tile_idx = (scroll + row) * 16 + col;
                if tile_idx >= 256 { break; }
                let tile_addr = base_addr.wrapping_add(tile_idx as u16 * 16);
                let dx = px + col * (tile_px + 1);
                let dy = py + row * (tile_px + 1);

                // Draw 8x8 tile
                for ty_off in 0..8u32 {
                    let lo = read_emu_byte(emu, tile_addr.wrapping_add(ty_off as u16 * 2));
                    let hi = read_emu_byte(emu, tile_addr.wrapping_add(ty_off as u16 * 2 + 1));
                    for tx_off in 0..8u32 {
                        let bit = 7 - tx_off;
                        let color_id = ((hi >> bit) & 1) << 1 | ((lo >> bit) & 1);
                        let shade = match color_id {
                            0 => 0xFF0A1510,
                            1 => 0xFF346856,
                            2 => 0xFF88C070,
                            3 => 0xFFE0F8D0,
                            _ => 0xFF000000,
                        };
                        framebuffer::fill_rect(
                            dx + tx_off * tile_size,
                            dy + ty_off * tile_size,
                            tile_size, tile_size, shade,
                        );
                    }
                }
            }
        }

        // Tile index label below
        let label_y = py + max_rows * (tile_px + 1) + 4;
        let range_s = format!("Tiles {}-{}", scroll * 16, ((scroll + max_rows) * 16).min(256) - 1);
        draw_str(px, label_y, &range_s, COL_DIM);
    } else {
        // OAM sprite listing
        draw_str(px, py, "#  Y   X   TILE FLAGS", COL_ACCENT);
        py += LINE_H;

        let max_sprites = ((ch - (py - cy)) / LINE_H).min(40);
        for i in 0..max_sprites {
            let oam_addr = 0xFE00u16 + i as u16 * 4;
            let sy = read_emu_byte(emu, oam_addr);
            let sx = read_emu_byte(emu, oam_addr + 1);
            let tile = read_emu_byte(emu, oam_addr + 2);
            let flags = read_emu_byte(emu, oam_addr + 3);

            let visible = sy > 0 && sy < 160 && sx > 0 && sx < 168;
            let col = if visible { COL_VALUE } else { COL_DIM };

            let s = format!("{:2} {:3} {:3}  {:02X}   {:02X}", i, sy, sx, tile, flags);
            draw_str(px, py, &s, col);

            // Flags breakdown
            let flag_x = px + 200;
            if flags & 0x80 != 0 { draw_str(flag_x, py, "P", COL_WARN); }
            if flags & 0x40 != 0 { draw_str(flag_x + 12, py, "Y", COL_ACCENT); }
            if flags & 0x20 != 0 { draw_str(flag_x + 24, py, "X", COL_ACCENT); }
            if emu.cgb_mode {
                let pal = flags & 0x07;
                let bank = (flags >> 3) & 1;
                let ps = format!("P{} B{}", pal, bank);
                draw_str(flag_x + 40, py, &ps, COL_CYAN);
            }

            py += LINE_H;
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TAB: TRACE (instruction trace log + disassembly)
// ═══════════════════════════════════════════════════════════════════════════

fn draw_tab_trace(state: &GameLabState, emu: Option<&GameBoyEmulator>, cx: u32, cy: u32, cw: u32, ch: u32) {
    let px = cx + 8;
    let mut py = cy + 6;

    draw_str(px, py, "TRACE LOG", COL_ACCENT);
    let enabled_s = if state.trace_enabled { "[ON]" } else { "[OFF]" };
    let enabled_c = if state.trace_enabled { COL_BRIGHT } else { COL_RED };
    draw_str(px + 88, py, enabled_s, enabled_c);
    draw_str(px + 132, py, "(T=toggle, R=clear)", COL_DIM);
    py += LINE_H + 2;

    // Disassembly around current PC (top section)
    if let Some(emu) = emu {
        draw_str(px, py, "DISASSEMBLY @ PC", COL_ACCENT);
        py += LINE_H;
        let pc = emu.cpu.pc;
        // Show a few instructions before and after PC
        let mut addr = pc.wrapping_sub(8);
        let disasm_rows = 12u32.min((ch / 3) / LINE_H);
        for _ in 0..disasm_rows {
            let opcode = read_emu_byte(emu, addr);
            let is_current = addr == pc;
            let prefix = if is_current { ">" } else { " " };
            let (mnemonic, size) = disasm_opcode(emu, addr);
            let s = format!("{} {:04X}: {:02X}  {}", prefix, addr, opcode, mnemonic);
            let col = if is_current { COL_BRIGHT } else { COL_TEXT };
            if is_current {
                framebuffer::fill_rect(px - 2, py - 1, cw - 16, LINE_H, 0xFF1A3020);
            }
            draw_str(px, py, &s, col);

            // Breakpoint indicator
            if state.breakpoints.iter().any(|&bp| bp == addr) {
                framebuffer::fill_rect(px - 6, py + 2, 4, 8, COL_RED);
            }

            py += LINE_H;
            addr = addr.wrapping_add(size as u16);
        }
    }
    py += 6;
    framebuffer::fill_rect(cx + 4, py, cw - 8, 1, COL_BORDER);
    py += 4;

    // Breakpoints section
    draw_str(px, py, "BREAKPOINTS", COL_ACCENT);
    let bp_count = format!("{}/{}", state.breakpoints.len(), MAX_BREAKPOINTS);
    draw_str(px + 100, py, &bp_count, COL_DIM);
    py += LINE_H;

    if state.breakpoints.is_empty() {
        draw_str(px, py, "None (type addr in Search, click to add)", COL_DIM);
    } else {
        let mut bpx = px;
        for &bp in state.breakpoints.iter() {
            let bps = format!("{:04X}", bp);
            framebuffer::fill_rect(bpx, py, 40, LINE_H - 2, 0xFF2A0A0A);
            draw_str(bpx + 2, py, &bps, COL_RED);
            bpx += 48;
            if bpx > cx + cw - 60 { py += LINE_H; bpx = px; }
        }
    }
    py += LINE_H + 4;
    framebuffer::fill_rect(cx + 4, py, cw - 8, 1, COL_BORDER);
    py += 4;

    // Trace entries
    draw_str(px, py, "TRACE HISTORY", COL_ACCENT);
    let trace_count = format!("({} entries)", state.trace.len());
    draw_str(px + 120, py, &trace_count, COL_DIM);
    py += LINE_H;

    draw_str(px, py, "PC    OP  A  F  SP", COL_DIM);
    py += LINE_H;

    let max_rows = ((ch.saturating_sub(py - cy)) / LINE_H) as usize;
    let start = if state.trace.len() > max_rows { state.trace.len() - max_rows } else { 0 };
    for entry in state.trace[start..].iter() {
        let s = format!("{:04X}  {:02X}  {:02X} {:02X} {:04X}", entry.pc, entry.opcode, entry.a, entry.f, entry.sp);
        draw_str(px, py, &s, COL_TEXT);
        py += LINE_H;
        if py + LINE_H > cy + ch { break; }
    }
}

// ── Panel: CPU Registers ───────────────────────────────────────────────────
fn draw_panel_cpu(emu: Option<&GameBoyEmulator>, x: u32, y: u32, w: u32, h: u32, state: &GameLabState) {
    draw_panel_frame(x, y, w, h, "CPU REGISTERS", state.selected_panel == 0);

    let px = x + PANEL_PAD + 2;
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
            draw_str(px, py, name, COL_REG);
            let s = format!("{:02X}{:02X}", hi, lo);
            draw_str(px + 28, py, &s, COL_VALUE);
            // Show individual bytes
            let s2 = format!("({:3} {:3})", hi, lo);
            draw_str(px + 72, py, &s2, COL_DIM);
            py += LINE_H;
        }

        py += 4;
        // SP & PC
        draw_str(px, py, "SP", COL_REG);
        let sps = format!("{:04X}", cpu.sp);
        draw_str(px + 28, py, &sps, COL_VALUE);
        py += LINE_H;

        draw_str(px, py, "PC", COL_REG);
        let pcs = format!("{:04X}", cpu.pc);
        draw_str(px + 28, py, &pcs, COL_ACCENT);

        // Show opcode at PC
        if emu.rom_loaded {
            let opcode = read_emu_byte(emu, cpu.pc);
            let ops = format!("[{:02X}]", opcode);
            draw_str(px + 72, py, &ops, COL_CYAN);
        }
        py += LINE_H + 6;

        // Flags
        draw_str(px, py, "FLAGS", COL_DIM);
        py += LINE_H;
        let flags = [
            ("Z", cpu.f & 0x80 != 0),
            ("N", cpu.f & 0x40 != 0),
            ("H", cpu.f & 0x20 != 0),
            ("C", cpu.f & 0x10 != 0),
        ];
        let mut fx = px;
        for (name, set) in &flags {
            let color = if *set { COL_FLAG_ON } else { COL_FLAG_OFF };
            framebuffer::fill_rect(fx, py, 24, 14, if *set { 0xFF0A3020 } else { 0xFF0A1510 });
            draw_str(fx + 4, py + 1, name, color);
            fx += 28;
        }
        py += LINE_H + 6;

        // IME, HALT state
        draw_str(px, py, "IME", COL_DIM);
        draw_str(px + 32, py, if cpu.ime { "ON" } else { "OFF" }, 
            if cpu.ime { COL_FLAG_ON } else { COL_FLAG_OFF });
        draw_str(px + 64, py, "HALT", COL_DIM);
        draw_str(px + 100, py, if cpu.halted { "YES" } else { "NO" },
            if cpu.halted { COL_WARN } else { COL_DIM });
        py += LINE_H;

        // Cycle counter
        draw_str(px, py, "CYCLES", COL_DIM);
        let cs = format!("{}", cpu.cycles);
        draw_str(px + 56, py, &cs, COL_VALUE);
        py += LINE_H;

        // CGB mode
        if emu.cgb_mode {
            draw_str(px, py, "MODE", COL_DIM);
            draw_str(px + 40, py, "CGB", COL_BRIGHT);
            let spd = if emu.key1 & 0x80 != 0 { "2x" } else { "1x" };
            draw_str(px + 72, py, spd, COL_ACCENT);
        }
    } else {
        draw_str(px, py, "No emulator linked", COL_DIM);
    }
}

// ── Panel: GPU / LCD State ─────────────────────────────────────────────────
fn draw_panel_gpu(emu: Option<&GameBoyEmulator>, x: u32, y: u32, w: u32, h: u32, state: &GameLabState) {
    draw_panel_frame(x, y, w, h, "GPU / LCD", state.selected_panel == 1);

    let px = x + PANEL_PAD + 2;
    let mut py = y + 20;

    if let Some(emu) = emu {
        let gpu = &emu.gpu;

        // LCDC register breakdown
        draw_str(px, py, "LCDC", COL_REG);
        let ls = format!("{:02X}", gpu.lcdc);
        draw_str(px + 40, py, &ls, COL_VALUE);
        let lcd_on = gpu.lcdc & 0x80 != 0;
        draw_str(px + 64, py, if lcd_on { "LCD:ON" } else { "LCD:OFF" },
            if lcd_on { COL_FLAG_ON } else { COL_RED });
        py += LINE_H;

        // LCDC bits
        let bits = [
            ("BG", gpu.lcdc & 0x01 != 0),
            ("OBJ", gpu.lcdc & 0x02 != 0),
            ("8x16", gpu.lcdc & 0x04 != 0),
            ("WIN", gpu.lcdc & 0x20 != 0),
        ];
        let mut bx = px;
        for (name, on) in &bits {
            let col = if *on { COL_FLAG_ON } else { COL_FLAG_OFF };
            draw_str(bx, py, name, col);
            bx += (name.len() as u32 + 1) * CHAR_W;
        }
        py += LINE_H + 4;

        // Scanline info
        draw_str(px, py, "LY", COL_REG);
        let lys = format!("{:3} / 153", gpu.ly);
        draw_str(px + 28, py, &lys, COL_VALUE);
        // Scanline progress bar
        let bar_x = px + 110;
        let bar_w = w.saturating_sub(130);
        framebuffer::fill_rect(bar_x, py + 2, bar_w, 8, 0xFF0A1A10);
        let progress = (gpu.ly as u32 * bar_w) / 154;
        let bar_col = if gpu.ly < 144 { COL_BRIGHT } else { COL_WARN };
        framebuffer::fill_rect(bar_x, py + 2, progress.min(bar_w), 8, bar_col);
        py += LINE_H;

        draw_str(px, py, "LYC", COL_DIM);
        let lycs = format!("{:3}", gpu.lyc);
        draw_str(px + 32, py, &lycs, COL_VALUE);
        if gpu.ly == gpu.lyc {
            draw_str(px + 60, py, "=MATCH", COL_BRIGHT);
        }
        py += LINE_H;

        // Mode
        draw_str(px, py, "MODE", COL_DIM);
        let (mode_name, mode_col) = match gpu.mode {
            0 => ("HBLANK", COL_DIM),
            1 => ("VBLANK", COL_WARN),
            2 => ("OAM", COL_ACCENT),
            3 => ("DRAW", COL_BRIGHT),
            _ => ("???", COL_RED),
        };
        draw_str(px + 40, py, mode_name, mode_col);
        let cycs = format!("({} dots)", gpu.cycles);
        draw_str(px + 96, py, &cycs, COL_DIM);
        py += LINE_H + 4;

        // Scroll
        draw_str(px, py, "SCX/Y", COL_DIM);
        let ss = format!("{:3},{:3}", gpu.scx, gpu.scy);
        draw_str(px + 48, py, &ss, COL_VALUE);
        py += LINE_H;
        draw_str(px, py, "WX/Y", COL_DIM);
        let ws = format!("{:3},{:3}", gpu.wx, gpu.wy);
        draw_str(px + 48, py, &ws, COL_VALUE);
        py += LINE_H + 4;

        // DMG Palettes
        draw_str(px, py, "BGP", COL_DIM);
        draw_palette_bar(px + 32, py, gpu.bgp);
        py += LINE_H;
        draw_str(px, py, "OBP0", COL_DIM);
        draw_palette_bar(px + 40, py, gpu.obp0);
        py += LINE_H;
        draw_str(px, py, "OBP1", COL_DIM);
        draw_palette_bar(px + 40, py, gpu.obp1);
        py += LINE_H + 4;

        // CGB palettes (show first BG palette colors if CGB)
        if emu.cgb_mode {
            draw_str(px, py, "CGB BG PALETTES", COL_DIM);
            py += LINE_H;
            for pal in 0..8 {
                let ppx = px + pal * 36;
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
            draw_str(px, py, "VRAM BANK", COL_DIM);
            let vbs = format!("{}", gpu.vram_bank);
            draw_str(px + 80, py, &vbs, COL_VALUE);
        }
    } else {
        draw_str(px, py, "No emulator linked", COL_DIM);
    }
}

// ── Panel: Memory Viewer ───────────────────────────────────────────────────
fn draw_panel_memory(emu: Option<&GameBoyEmulator>, state: &GameLabState, x: u32, y: u32, w: u32, h: u32) {
    draw_panel_frame(x, y, w, h, "MEMORY", state.selected_panel == 2);

    let px = x + PANEL_PAD + 2;
    let mut py = y + 20;

    // Memory mode tabs
    let modes = ["WRAM", "VRAM", "HRAM", "ROM", "OAM"];
    let mut tx = px;
    for (i, name) in modes.iter().enumerate() {
        let selected = i as u8 == state.mem_mode;
        let col = if selected { COL_BRIGHT } else { COL_DIM };
        if selected {
            framebuffer::fill_rect(tx, py, name.len() as u32 * CHAR_W + 4, LINE_H, 0xFF1A3020);
        }
        draw_str(tx + 2, py, name, col);
        tx += name.len() as u32 * CHAR_W + 8;
    }
    py += LINE_H + 4;

    // Address header
    let adr_s = format!("ADDR  {:04X}", state.mem_view_addr);
    draw_str(px, py, &adr_s, COL_ACCENT);
    py += LINE_H;

    if let Some(emu) = emu {
        // Hex dump rows
        let rows = ((h - 60) / LINE_H).min(16) as u16;
        for row in 0..rows {
            let addr = state.mem_view_addr.wrapping_add(row * 16);
            // Address
            let addr_s = format!("{:04X}", addr);
            draw_str(px, py, &addr_s, COL_ADDR);

            // Hex bytes
            let mut hx = px + 40;
            let mut ascii_buf = [b'.'; 16];
            for col in 0..16u16 {
                let a = addr.wrapping_add(col);
                let byte = read_emu_byte(emu, a);
                let hs = format!("{:02X}", byte);
                // Memory diff: highlight changed bytes
                let diff_idx = (row * 16 + col) as usize;
                let prev = if diff_idx < state.mem_prev.len() { state.mem_prev[diff_idx] } else { byte };
                let byte_col = if byte != prev { COL_CHANGED } else if byte == 0 { COL_DIM } else { COL_VALUE };
                draw_str(hx, py, &hs, byte_col);
                hx += 20;
                if col == 7 { hx += 4; } // gap between 8-byte groups

                if byte >= 0x20 && byte < 0x7F {
                    ascii_buf[col as usize] = byte;
                }
            }

            // ASCII column (if space)
            if w > 420 {
                let ascii: alloc::string::String = ascii_buf.iter().map(|&b| b as char).collect();
                draw_str(hx + 8, py, &ascii, COL_DIM);
            }

            py += LINE_H;
        }
    } else {
        draw_str(px, py, "No emulator linked", COL_DIM);
    }
}

// ── Panel: I/O Registers ──────────────────────────────────────────────────
fn draw_panel_io(emu: Option<&GameBoyEmulator>, x: u32, y: u32, w: u32, h: u32, state: &GameLabState) {
    draw_panel_frame(x, y, w, h, "I/O REGISTERS", state.selected_panel == 3);

    let px = x + PANEL_PAD + 2;
    let mut py = y + 20;

    if let Some(emu) = emu {
        // Interrupts
        draw_str(px, py, "INTERRUPTS", COL_ACCENT);
        py += LINE_H;

        draw_str(px, py, "IE", COL_REG);
        let ies = format!("{:02X}", emu.ie_reg);
        draw_str(px + 24, py, &ies, COL_VALUE);

        draw_str(px + 50, py, "IF", COL_REG);
        let ifs = format!("{:02X}", emu.if_reg);
        draw_str(px + 74, py, &ifs, COL_VALUE);
        py += LINE_H;

        // Individual interrupt flags
        let int_names = ["VBL", "STA", "TIM", "SER", "JOY"];
        let mut ix = px;
        for (i, name) in int_names.iter().enumerate() {
            let ie = emu.ie_reg & (1 << i) != 0;
            let iflag = emu.if_reg & (1 << i) != 0;
            let col = if ie && iflag { COL_RED } else if ie { COL_FLAG_ON } else { COL_FLAG_OFF };
            draw_str(ix, py, name, col);
            ix += 32;
        }
        py += LINE_H + 6;

        // Timer
        draw_str(px, py, "TIMER", COL_ACCENT);
        py += LINE_H;

        draw_str(px, py, "DIV", COL_DIM);
        let divs = format!("{:02X}", emu.timer.read_div());
        draw_str(px + 32, py, &divs, COL_VALUE);

        draw_str(px + 60, py, "TIMA", COL_DIM);
        let timas = format!("{:02X}", emu.timer.tima);
        draw_str(px + 100, py, &timas, COL_VALUE);
        py += LINE_H;

        draw_str(px, py, "TMA", COL_DIM);
        let tmas = format!("{:02X}", emu.timer.tma);
        draw_str(px + 32, py, &tmas, COL_VALUE);

        draw_str(px + 60, py, "TAC", COL_DIM);
        let tacs = format!("{:02X}", emu.timer.tac);
        draw_str(px + 100, py, &tacs, COL_VALUE);
        py += LINE_H + 6;

        // Serial
        draw_str(px, py, "SERIAL", COL_ACCENT);
        py += LINE_H;
        draw_str(px, py, "SB", COL_DIM);
        let sbs = format!("{:02X}", emu.serial_data);
        draw_str(px + 24, py, &sbs, COL_VALUE);
        draw_str(px + 50, py, "SC", COL_DIM);
        let scs = format!("{:02X}", emu.serial_ctrl);
        draw_str(px + 74, py, &scs, COL_VALUE);

        if emu.cgb_mode {
            py += LINE_H + 6;
            draw_str(px, py, "CGB I/O", COL_ACCENT);
            py += LINE_H;
            draw_str(px, py, "KEY1", COL_DIM);
            let k1s = format!("{:02X}", emu.key1);
            draw_str(px + 40, py, &k1s, COL_VALUE);
            draw_str(px + 68, py, "WRAM", COL_DIM);
            let wbs = format!("BK{}", emu.wram_bank);
            draw_str(px + 104, py, &wbs, COL_VALUE);
        }
    } else {
        draw_str(px, py, "No emulator linked", COL_DIM);
    }
}

// ── Panel: Cartridge Info ──────────────────────────────────────────────────
fn draw_panel_cart(emu: Option<&GameBoyEmulator>, x: u32, y: u32, w: u32, h: u32) {
    draw_panel_frame(x, y, w, h, "CARTRIDGE", false);

    let px = x + PANEL_PAD + 2;
    let mut py = y + 20;

    if let Some(emu) = emu {
        let cart = &emu.cart;

        // Title
        let title_bytes: alloc::vec::Vec<u8> = cart.title.iter().copied()
            .take_while(|&c| c != 0 && c >= 0x20).collect();
        let title = core::str::from_utf8(&title_bytes).unwrap_or("???");
        draw_str(px, py, "TITLE", COL_DIM);
        draw_str(px + 48, py, title, COL_BRIGHT);
        py += LINE_H;

        // Type
        let mbc = match cart.mbc_type {
            crate::gameboy::cartridge::MbcType::None => "ROM ONLY",
            crate::gameboy::cartridge::MbcType::Mbc1 => "MBC1",
            crate::gameboy::cartridge::MbcType::Mbc3 => "MBC3",
            crate::gameboy::cartridge::MbcType::Mbc5 => "MBC5",
        };
        draw_str(px, py, "MBC", COL_DIM);
        draw_str(px + 32, py, mbc, COL_VALUE);
        py += LINE_H;

        // ROM/RAM size
        let rom_kb = cart.rom.len() / 1024;
        let ram_kb = cart.ram.len() / 1024;
        draw_str(px, py, "ROM", COL_DIM);
        let rs = format!("{}KB", rom_kb);
        draw_str(px + 32, py, &rs, COL_VALUE);
        draw_str(px + 80, py, "RAM", COL_DIM);
        let ras = format!("{}KB", ram_kb);
        draw_str(px + 112, py, &ras, COL_VALUE);
        py += LINE_H;

        // Banks
        draw_str(px, py, "ROM BANK", COL_DIM);
        let rbs = format!("{:3}", cart.rom_bank);
        draw_str(px + 72, py, &rbs, COL_VALUE);
        let total_banks = cart.rom.len() / 16384;
        let tbs = format!("/ {}", total_banks);
        draw_str(px + 96, py, &tbs, COL_DIM);
        py += LINE_H;

        draw_str(px, py, "RAM BANK", COL_DIM);
        let rmbs = format!("{:3}", cart.ram_bank);
        draw_str(px + 72, py, &rmbs, COL_VALUE);
        py += LINE_H;

        // CGB flag
        draw_str(px, py, "CGB", COL_DIM);
        let cgb_s = match cart.cgb_flag {
            0xC0 => "CGB ONLY",
            0x80 => "CGB+DMG",
            _ => "DMG",
        };
        let cgb_c = if cart.cgb_flag >= 0x80 { COL_ACCENT } else { COL_DIM };
        draw_str(px + 32, py, cgb_s, cgb_c);
    } else {
        draw_str(px, py, "No cartridge", COL_DIM);
    }
}

// ── Panel: Input State ─────────────────────────────────────────────────────
fn draw_panel_input(emu: Option<&GameBoyEmulator>, x: u32, y: u32, w: u32, h: u32, state: &GameLabState) {
    draw_panel_frame(x, y, w, h, "INPUT STATE", state.selected_panel == 5);

    let px = x + PANEL_PAD + 2;
    let mut py = y + 20;

    if let Some(emu) = emu {
        draw_str(px, py, "JOYPAD $FF00", COL_ACCENT);
        let jps = format!("{:02X}", emu.joypad_reg);
        draw_str(px + 104, py, &jps, COL_VALUE);
        py += LINE_H + 4;

        // D-Pad visualization
        draw_str(px, py, "D-PAD", COL_DIM);
        py += LINE_H;
        let up    = emu.joypad_dirs & 0x04 == 0;
        let down  = emu.joypad_dirs & 0x08 == 0;
        let left  = emu.joypad_dirs & 0x02 == 0;
        let right = emu.joypad_dirs & 0x01 == 0;

        // Draw d-pad shape
        let dx = px + 16;
        let dy = py;
        let sz: u32 = 16;
        // Up
        framebuffer::fill_rect(dx + sz, dy, sz, sz, if up { COL_FLAG_ON } else { 0xFF1A2820 });
        draw_str(dx + sz + 3, dy + 2, "U", if up { 0xFF000000 } else { COL_DIM });
        // Down
        framebuffer::fill_rect(dx + sz, dy + sz * 2, sz, sz, if down { COL_FLAG_ON } else { 0xFF1A2820 });
        draw_str(dx + sz + 3, dy + sz * 2 + 2, "D", if down { 0xFF000000 } else { COL_DIM });
        // Left
        framebuffer::fill_rect(dx, dy + sz, sz, sz, if left { COL_FLAG_ON } else { 0xFF1A2820 });
        draw_str(dx + 3, dy + sz + 2, "L", if left { 0xFF000000 } else { COL_DIM });
        // Right
        framebuffer::fill_rect(dx + sz * 2, dy + sz, sz, sz, if right { COL_FLAG_ON } else { 0xFF1A2820 });
        draw_str(dx + sz * 2 + 3, dy + sz + 2, "R", if right { 0xFF000000 } else { COL_DIM });
        // Center
        framebuffer::fill_rect(dx + sz, dy + sz, sz, sz, 0xFF1A2820);

        // Action buttons
        let bx = px + 100;
        let a_pressed = emu.joypad_buttons & 0x01 == 0;
        let b_pressed = emu.joypad_buttons & 0x02 == 0;
        let sel_pressed = emu.joypad_buttons & 0x04 == 0;
        let start_pressed = emu.joypad_buttons & 0x08 == 0;

        // A button (circle feel)
        framebuffer::fill_rect(bx + 32, dy + 4, 22, 22, if a_pressed { COL_FLAG_ON } else { 0xFF1A2820 });
        draw_str(bx + 38, dy + 8, "A", if a_pressed { 0xFF000000 } else { COL_VALUE });

        // B button
        framebuffer::fill_rect(bx, dy + 16, 22, 22, if b_pressed { COL_FLAG_ON } else { 0xFF1A2820 });
        draw_str(bx + 6, dy + 20, "B", if b_pressed { 0xFF000000 } else { COL_VALUE });

        py = dy + sz * 3 + 6;

        // Select / Start
        let sel_x = px + 20;
        framebuffer::fill_rect(sel_x, py, 40, 14, if sel_pressed { COL_ACCENT } else { 0xFF1A2820 });
        draw_str(sel_x + 4, py + 1, "SEL", if sel_pressed { 0xFF000000 } else { COL_DIM });

        framebuffer::fill_rect(sel_x + 48, py, 48, 14, if start_pressed { COL_ACCENT } else { 0xFF1A2820 });
        draw_str(sel_x + 52, py + 1, "START", if start_pressed { 0xFF000000 } else { COL_DIM });
        py += LINE_H + 8;

        // Raw button bytes
        draw_str(px, py, "DIRS", COL_DIM);
        let ds = format!("{:02X}", emu.joypad_dirs);
        draw_str(px + 40, py, &ds, COL_VALUE);
        draw_str(px + 68, py, "BTNS", COL_DIM);
        let bs = format!("{:02X}", emu.joypad_buttons);
        draw_str(px + 108, py, &bs, COL_VALUE);
    } else {
        draw_str(px, py, "No emulator linked", COL_DIM);
    }
}

// ── Helpers ────────────────────────────────────────────────────────────────

fn draw_panel_frame(x: u32, y: u32, w: u32, h: u32, title: &str, selected: bool) {
    // Background
    framebuffer::fill_rect(x, y, w, h, COL_PANEL);
    // Border
    let border_col = if selected { COL_BRIGHT } else { COL_BORDER };
    framebuffer::fill_rect(x, y, w, 1, border_col);
    framebuffer::fill_rect(x, y + h - 1, w, 1, border_col);
    framebuffer::fill_rect(x, y, 1, h, border_col);
    framebuffer::fill_rect(x + w - 1, y, 1, h, border_col);
    // Header
    framebuffer::fill_rect(x + 1, y + 1, w - 2, 16, COL_HEADER_BG);
    draw_str(x + 6, y + 3, title, if selected { COL_BRIGHT } else { COL_ACCENT });
}

fn draw_str(x: u32, y: u32, text: &str, color: u32) {
    let mut cx = x;
    for ch in text.chars() {
        framebuffer::draw_char_at(cx, y, ch, color);
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
pub fn read_emu_byte(emu: &GameBoyEmulator, addr: u16) -> u8 {
    match addr {
        0x0000..=0x7FFF => emu.cart.read(addr),
        0x8000..=0x9FFF => emu.gpu.read_vram(addr),
        0xA000..=0xBFFF => emu.cart.read(addr),
        0xC000..=0xCFFF => {
            let idx = (addr as usize) - 0xC000;
            if idx < emu.wram.len() { emu.wram[idx] } else { 0xFF }
        }
        0xD000..=0xDFFF => {
            let bank = emu.wram_bank.max(1) as usize;
            let offset = bank * 0x1000 + (addr as usize - 0xD000);
            if offset < emu.wram.len() { emu.wram[offset] } else { 0xFF }
        }
        0xFE00..=0xFE9F => emu.gpu.read_oam(addr),
        0xFF80..=0xFFFE => {
            let idx = (addr - 0xFF80) as usize;
            if idx < emu.hram.len() { emu.hram[idx] } else { 0xFF }
        }
        0xFFFF => emu.ie_reg,
        _ => 0xFF,
    }
}

/// Update memory diff snapshot (called each frame from desktop)
pub fn update_mem_diff(state: &mut GameLabState, emu: &GameBoyEmulator) {
    for i in 0..256usize {
        let addr = state.mem_view_addr.wrapping_add(i as u16);
        state.mem_prev[i] = read_emu_byte(emu, addr);
    }
}

/// Draw a small button
fn draw_btn(x: u32, y: u32, w: u32, h: u32, label: &str, active: bool, color: u32) {
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
fn disasm_opcode(emu: &GameBoyEmulator, addr: u16) -> (String, u8) {
    let op = read_emu_byte(emu, addr);
    let b1 = read_emu_byte(emu, addr.wrapping_add(1));
    let b2 = read_emu_byte(emu, addr.wrapping_add(2));
    let imm16 = (b2 as u16) << 8 | b1 as u16;

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
            let dst = ["B","C","D","E","H","L","(HL)","A"][(op as usize >> 3) & 7];
            let src = ["B","C","D","E","H","L","(HL)","A"][op as usize & 7];
            (format!("LD {},{}", dst, src), 1)
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
    cx: u32, cy: u32, cw: u32, ch: u32,
) {
    // Background
    framebuffer::fill_rect(cx, cy, cw, ch, 0xFF0A0F14);
    
    if cw < 60 || ch < 40 { return; }
    
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
    draw_str(cx + 6, cy + 4, "GAME BOY INPUT", COL_DIM);
    
    // Key mapping hint (right side)
    if cw > 300 {
        draw_str(cx + cw - 200, cy + 4, "WASD=Pad X=A Z=B C=Sel", 0xFF3A5A44);
    }
    
    // ── D-Pad ──────────────────────────────────────────────────────
    let dpad_x = cx + 40;
    let dpad_y = cy + 30;
    let sz: u32 = 26;
    let gap: u32 = 2;
    
    // Up
    draw_input_btn(dpad_x, dpad_y - sz - gap, sz, sz, "W", up);
    // Down
    draw_input_btn(dpad_x, dpad_y + sz + gap, sz, sz, "S", down);
    // Left
    draw_input_btn(dpad_x - sz - gap, dpad_y, sz, sz, "A", left);
    // Right
    draw_input_btn(dpad_x + sz + gap, dpad_y, sz, sz, "D", right);
    // Center
    framebuffer::fill_rect(dpad_x, dpad_y, sz, sz, 0xFF141E1A);
    
    // ── A / B Buttons ──────────────────────────────────────────────
    let btn_sz: u32 = 30;
    let a_x = cx + cw - 80;
    let a_y = cy + 30;
    let b_x = cx + cw - 140;
    let b_y = cy + 48;
    
    draw_input_circle(a_x, a_y, btn_sz, "A", a_pressed);
    draw_input_circle(b_x, b_y, btn_sz, "B", b_pressed);
    
    // Labels
    draw_str(a_x + btn_sz + 4, a_y + 8, "(X)", COL_DIM);
    draw_str(b_x - 28, b_y + 8, "(Z)", COL_DIM);
    
    // ── Select / Start ─────────────────────────────────────────────
    let mid_x = cx + cw / 2;
    let pill_y = cy + ch - 36;
    draw_input_pill(mid_x - 70, pill_y, 56, 20, "SELECT", sel_pressed);
    draw_input_pill(mid_x + 14, pill_y, 56, 20, "START", start_pressed);
    
    // Labels
    draw_str(mid_x - 70, pill_y + 22, "(C)", COL_DIM);
    draw_str(mid_x + 14, pill_y + 22, "(Enter)", COL_DIM);
    
    // ── Raw joypad bytes ───────────────────────────────────────────
    let info_y = cy + ch - 16;
    let ds = alloc::format!("DIRS:{:02X}", dirs_raw);
    let bs = alloc::format!("BTNS:{:02X}", btns_raw);
    draw_str(cx + 6, info_y, &ds, 0xFF3A5A44);
    draw_str(cx + 80, info_y, &bs, 0xFF3A5A44);
}

/// Get input button bounds for click hit-testing
pub fn get_input_buttons(cx: u32, cy: u32, cw: u32, ch: u32) -> [(u32, u32, u32, u32, u8); 8] {
    let dpad_x = cx + 40;
    let dpad_y = cy + 30;
    let sz: u32 = 26;
    let gap: u32 = 2;
    
    let btn_sz: u32 = 30;
    let a_x = cx + cw - 80;
    let a_y = cy + 30;
    let b_x = cx + cw - 140;
    let b_y = cy + 48;
    
    let mid_x = cx + cw / 2;
    let pill_y = cy + ch - 36;
    
    [
        (dpad_x, dpad_y - sz - gap, sz, sz, b'w'),           // Up
        (dpad_x, dpad_y + sz + gap, sz, sz, b's'),           // Down
        (dpad_x - sz - gap, dpad_y, sz, sz, b'a'),           // Left
        (dpad_x + sz + gap, dpad_y, sz, sz, b'd'),           // Right
        (a_x, a_y, btn_sz, btn_sz, b'x'),                    // A
        (b_x, b_y, btn_sz, btn_sz, b'z'),                    // B
        (mid_x - 70, pill_y, 56, 20, b'c'),                  // Select
        (mid_x + 14, pill_y, 56, 20, b'\r'),                 // Start
    ]
}

fn draw_input_btn(x: u32, y: u32, w: u32, h: u32, label: &str, pressed: bool) {
    let bg = if pressed { 0xFF00CC66 } else { 0xFF1A2E24 };
    framebuffer::fill_rect(x, y, w, h, bg);
    // Border
    let bc = if pressed { 0xFF00FF88 } else { 0xFF2A4A38 };
    framebuffer::fill_rect(x, y, w, 1, bc);
    framebuffer::fill_rect(x, y + h - 1, w, 1, bc);
    framebuffer::fill_rect(x, y, 1, h, bc);
    framebuffer::fill_rect(x + w - 1, y, 1, h, bc);
    let text_col = if pressed { 0xFF000000 } else { 0xFF00FF88 };
    let tx = x + (w / 2).saturating_sub(4);
    let ty = y + (h / 2).saturating_sub(6);
    draw_str(tx, ty, label, text_col);
}

fn draw_input_circle(x: u32, y: u32, sz: u32, label: &str, pressed: bool) {
    let bg = if pressed { 0xFF00CC66 } else { 0xFF1A2E24 };
    framebuffer::fill_rect(x, y, sz, sz, bg);
    // Rounded corners
    framebuffer::fill_rect(x, y, 4, 4, 0xFF0A0F14);
    framebuffer::fill_rect(x + sz - 4, y, 4, 4, 0xFF0A0F14);
    framebuffer::fill_rect(x, y + sz - 4, 4, 4, 0xFF0A0F14);
    framebuffer::fill_rect(x + sz - 4, y + sz - 4, 4, 4, 0xFF0A0F14);
    // Border
    let bc = if pressed { 0xFF00FF88 } else { 0xFF2A4A38 };
    framebuffer::fill_rect(x + 3, y, sz - 6, 1, bc);
    framebuffer::fill_rect(x + 3, y + sz - 1, sz - 6, 1, bc);
    framebuffer::fill_rect(x, y + 3, 1, sz - 6, bc);
    framebuffer::fill_rect(x + sz - 1, y + 3, 1, sz - 6, bc);
    let text_col = if pressed { 0xFF000000 } else { 0xFF00FF88 };
    draw_str(x + sz / 2 - 4, y + sz / 2 - 6, label, text_col);
}

fn draw_input_pill(x: u32, y: u32, w: u32, h: u32, label: &str, pressed: bool) {
    let bg = if pressed { 0xFF00CC66 } else { 0xFF141E1A };
    framebuffer::fill_rect(x, y, w, h, bg);
    let bc = if pressed { 0xFF00FF88 } else { 0xFF2A4A38 };
    framebuffer::fill_rect(x + 2, y, w - 4, 1, bc);
    framebuffer::fill_rect(x + 2, y + h - 1, w - 4, 1, bc);
    framebuffer::fill_rect(x, y + 2, 1, h - 4, bc);
    framebuffer::fill_rect(x + w - 1, y + 2, 1, h - 4, bc);
    let text_col = if pressed { 0xFF000000 } else { 0xFF80FFAA };
    let label_w = label.len() as u32 * 8;
    draw_str(x + (w.saturating_sub(label_w)) / 2, y + (h / 2).saturating_sub(6), label, text_col);
}
