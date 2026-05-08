












extern crate alloc;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use crate::framebuffer;
use crate::gameboy::GameBoyEmulator;


const FI_: u32 = 28;
const EK_: u32 = 4;
const L_: u32 = 14;
const CE_: u32 = 8;
const JX_: u32 = 24;


const IK_: u32         = 0xFF0A0F14;
const BQQ_: u32      = 0xFF111920;
const KI_: u32     = 0xFF1E2A36;
const KJ_: u32  = 0xFF142028;
const P_: u32       = 0xFF9CD8B0;   
const F_: u32        = 0xFF4A6A54;   
const BF_: u32     = 0xFF00FF88;   
const M_: u32     = 0xFF58A6FF;   
const AG_: u32      = 0xFFE0F8D0;   
const EQ_: u32        = 0xFF80FFAA;   
const DZ_: u32    = 0xFF00FF66;   
const TG_: u32   = 0xFF2A3A30;   
const NI_: u32       = 0xFFD29922;   
const AN_: u32        = 0xFFF85149;   
const AU_: u32       = 0xFF79C0FF;   
const BG_: u32     = 0xFFBC8CFF;   
const IJ_: u32       = 0xFF507060;   
const TF_: u32    = 0xFFFF4444;   
const TJ_: u32    = 0xFF0E1820;

const BCI_: usize = 16;
const CJA_: usize = 256;
const DCB_: usize = 64;
const CIN_: usize = 8;


#[derive(Clone, Copy, PartialEq)]
pub enum LabTab {
    Analyze = 0,
    Search = 1,
    Watch = 2,
    Tiles = 3,
    Trace = 4,
}


#[derive(Clone)]
pub struct Agq {
    pub addr: u16,
    pub label: [u8; 8],    
    pub label_len: u8,
    pub prev_value: u8,
    pub cur_value: u8,
    pub changed: bool,
    pub size: u8,           
}


#[derive(Clone, Copy, PartialEq)]
pub enum SearchMode {
    Exact,      
    Changed,    
    Unchanged,  
    Greater,    
    Less,       
}


#[derive(Clone, Copy)]
pub struct Vl {
    pub pc: u16,
    pub opcode: u8,
    pub a: u8, pub f: u8,
    pub sp: u16,
}


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


pub struct GameLabState {
    
    pub linked_gb_id: Option<u32>,
    
    pub mem_view_addr: u16,
    
    pub selected_panel: u8,
    
    pub frame: u32,
    
    pub mem_mode: u8,
    
    pub scroll: u32,

    
    pub active_tab: LabTab,

    
    pub watches: Vec<Agq>,

    
    pub search_value: u16,         
    pub search_input: [u8; 6],     
    pub search_input_len: u8,
    pub search_results: Vec<u16>,  
    pub search_snapshot: Vec<u8>,  
    pub search_mode: SearchMode,
    pub search_active: bool,       
    pub search_byte_mode: bool,    

    
    pub breakpoints: Vec<u16>,     
    pub bp_input: [u8; 5],        
    pub bp_input_len: u8,
    pub paused: bool,
    pub step_one: bool,            
    pub step_frame: bool,          

    
    
    pub speed_idx: u8,

    
    pub trace: Vec<Vl>,
    pub trace_enabled: bool,

    
    pub tile_page: u8,   
    pub tile_scroll: u32,

    
    pub mem_prev: [u8; 256], 

    
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
            speed_idx: 2, 
            trace: Vec::new(),
            trace_enabled: false,
            tile_page: 0,
            tile_scroll: 0,
            mem_prev: [0; 256],
            save_state: SaveState::empty(),
        }
    }

    pub fn qxj(&self) -> f32 {
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

    
    pub fn qtg(&mut self, an: &GameBoyEmulator) {
        if !self.trace_enabled { return; }
        let pc = an.cpu.pc;
        let opcode = aik(an, pc);
        let entry = Vl {
            pc,
            opcode,
            a: an.cpu.a,
            f: an.cpu.f,
            sp: an.cpu.sp,
        };
        if self.trace.len() >= DCB_ {
            self.trace.remove(0);
        }
        self.trace.push(entry);
    }

    
    pub fn should_break(&self, pc: u16) -> bool {
        self.breakpoints.iter().any(|&bp| bp == pc)
    }

    
    pub fn save_from(&mut self, an: &GameBoyEmulator) {
        let j = &mut self.save_state;
        j.cpu_a = an.cpu.a; j.cpu_f = an.cpu.f;
        j.cpu_b = an.cpu.b; j.cpu_c = an.cpu.c;
        j.cpu_d = an.cpu.d; j.cpu_e = an.cpu.e;
        j.cpu_h = an.cpu.h; j.cpu_l = an.cpu.l;
        j.cpu_sp = an.cpu.sp; j.cpu_pc = an.cpu.pc;
        j.cpu_ime = an.cpu.ime; j.cpu_halted = an.cpu.halted;
        j.ie_reg = an.ie_reg; j.if_reg = an.if_reg;
        j.joypad_reg = an.joypad_reg;
        j.joypad_buttons = an.joypad_buttons;
        j.joypad_dirs = an.joypad_dirs;
        j.serial_data = an.serial_data;
        j.serial_ctrl = an.serial_ctrl;
        j.wram_bank = an.wram_bank; j.key1 = an.key1;
        j.wram = an.wram.clone();
        j.hram = an.hram;
        j.gpu_lcdc = an.gpu.lcdc; j.gpu_stat = an.gpu.stat;
        j.gpu_scy = an.gpu.scy; j.gpu_scx = an.gpu.scx;
        j.gpu_ly = an.gpu.ly; j.gpu_lyc = an.gpu.lyc;
        j.gpu_bgp = an.gpu.bgp; j.gpu_obp0 = an.gpu.obp0; j.gpu_obp1 = an.gpu.obp1;
        j.gpu_wy = an.gpu.wy; j.gpu_wx = an.gpu.wx;
        j.gpu_mode = an.gpu.mode; j.gpu_cycles = an.gpu.cycles;
        j.gpu_vram = an.gpu.vram;
        j.gpu_vram1 = an.gpu.vram1;
        j.gpu_oam = an.gpu.oam;
        j.gpu_bg_palette = an.gpu.bg_palette;
        j.gpu_obj_palette = an.gpu.obj_palette;
        j.gpu_vram_bank = an.gpu.vram_bank;
        j.gpu_bcps = an.gpu.bcps; j.gpu_ocps = an.gpu.ocps;
        j.gpu_window_line = an.gpu.window_line;
        j.timer_div = an.timer.div; j.timer_tima = an.timer.tima;
        j.timer_tma = an.timer.tma; j.timer_tac = an.timer.tac;
        j.cart_rom_bank = an.cart.rom_bank;
        j.cart_ram_bank = an.cart.ram_bank;
        j.cart_ram_enabled = an.cart.ram_enabled;
        j.cart_mode = an.cart.mode;
        j.cart_ram = an.cart.ram.clone();
        j.valid = true;
    }

    
    pub fn load_into(&self, an: &mut GameBoyEmulator) {
        let j = &self.save_state;
        if !j.valid { return; }
        an.cpu.a = j.cpu_a; an.cpu.f = j.cpu_f;
        an.cpu.b = j.cpu_b; an.cpu.c = j.cpu_c;
        an.cpu.d = j.cpu_d; an.cpu.e = j.cpu_e;
        an.cpu.h = j.cpu_h; an.cpu.l = j.cpu_l;
        an.cpu.sp = j.cpu_sp; an.cpu.pc = j.cpu_pc;
        an.cpu.ime = j.cpu_ime; an.cpu.halted = j.cpu_halted;
        an.ie_reg = j.ie_reg; an.if_reg = j.if_reg;
        an.joypad_reg = j.joypad_reg;
        an.joypad_buttons = j.joypad_buttons;
        an.joypad_dirs = j.joypad_dirs;
        an.serial_data = j.serial_data;
        an.serial_ctrl = j.serial_ctrl;
        an.wram_bank = j.wram_bank; an.key1 = j.key1;
        if j.wram.len() == an.wram.len() {
            an.wram.copy_from_slice(&j.wram);
        }
        an.hram = j.hram;
        an.gpu.lcdc = j.gpu_lcdc; an.gpu.stat = j.gpu_stat;
        an.gpu.scy = j.gpu_scy; an.gpu.scx = j.gpu_scx;
        an.gpu.ly = j.gpu_ly; an.gpu.lyc = j.gpu_lyc;
        an.gpu.bgp = j.gpu_bgp; an.gpu.obp0 = j.gpu_obp0; an.gpu.obp1 = j.gpu_obp1;
        an.gpu.wy = j.gpu_wy; an.gpu.wx = j.gpu_wx;
        an.gpu.mode = j.gpu_mode; an.gpu.cycles = j.gpu_cycles;
        an.gpu.vram = j.gpu_vram;
        an.gpu.vram1 = j.gpu_vram1;
        an.gpu.oam = j.gpu_oam;
        an.gpu.bg_palette = j.gpu_bg_palette;
        an.gpu.obj_palette = j.gpu_obj_palette;
        an.gpu.vram_bank = j.gpu_vram_bank;
        an.gpu.bcps = j.gpu_bcps; an.gpu.ocps = j.gpu_ocps;
        an.gpu.window_line = j.gpu_window_line;
        an.timer.div = j.timer_div; an.timer.tima = j.timer_tima;
        an.timer.tma = j.timer_tma; an.timer.tac = j.timer_tac;
        an.cart.rom_bank = j.cart_rom_bank;
        an.cart.ram_bank = j.cart_ram_bank;
        an.cart.ram_enabled = j.cart_ram_enabled;
        an.cart.mode = j.cart_mode;
        if j.cart_ram.len() == an.cart.ram.len() {
            an.cart.ram.copy_from_slice(&j.cart_ram);
        }
    }

    
    pub fn take_search_snapshot(&mut self, an: &GameBoyEmulator) {
        self.search_snapshot.clear();
        self.search_snapshot.reserve(an.wram.len());
        for &b in an.wram.iter() {
            self.search_snapshot.push(b);
        }
    }

    
    pub fn search_initial(&mut self, an: &GameBoyEmulator) {
        self.search_results.clear();
        let val = self.search_value as u8;
        for (i, &b) in an.wram.iter().enumerate() {
            if b == val {
                let addr = if i < 0x1000 { 0xC000 + i as u16 } else { 0xD000 + (i as u16 - 0x1000) };
                self.search_results.push(addr);
                if self.search_results.len() >= CJA_ { break; }
            }
        }
        self.take_search_snapshot(an);
        self.search_active = true;
    }

    
    pub fn search_filter(&mut self, an: &GameBoyEmulator) {
        if !self.search_active { return; }
        let njq: Vec<u16> = self.search_results.iter().copied().filter(|&addr| {
            let agi = aik(an, addr);
            let prev = self.snapshot_byte(addr);
            match self.search_mode {
                SearchMode::Exact => agi == self.search_value as u8,
                SearchMode::Changed => agi != prev,
                SearchMode::Unchanged => agi == prev,
                SearchMode::Greater => agi > prev,
                SearchMode::Less => agi < prev,
            }
        }).collect();
        self.search_results = njq;
        self.take_search_snapshot(an);
    }

    fn snapshot_byte(&self, addr: u16) -> u8 {
        let idx = match addr {
            0xC000..=0xCFFF => (addr - 0xC000) as usize,
            0xD000..=0xDFFF => 0x1000 + (addr - 0xD000) as usize,
            _ => return 0xFF,
        };
        if idx < self.search_snapshot.len() { self.search_snapshot[idx] } else { 0xFF }
    }

    
    pub fn add_watch(&mut self, addr: u16) {
        if self.watches.len() >= BCI_ { return; }
        if self.watches.iter().any(|w| w.addr == addr) { return; }
        let mut label = [0u8; 8];
        let j = format!("{:04X}", addr);
        for (i, b) in j.bytes().enumerate().take(8) { label[i] = b; }
        self.watches.push(Agq {
            addr,
            label,
            label_len: j.len().min(8) as u8,
            prev_value: 0,
            cur_value: 0,
            changed: false,
            size: 1,
        });
    }

    
    pub fn update_watches(&mut self, an: &GameBoyEmulator) {
        for w in self.watches.iter_mut() {
            w.prev_value = w.cur_value;
            w.cur_value = aik(an, w.addr);
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
        
        match key {
            
            b'p' | b'P' => self.paused = !self.paused,
            
            b'n' | b'N' => { self.step_one = true; self.paused = true; }
            
            b'm' | b'M' => { self.step_frame = true; self.paused = true; }
            
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
                    
                    self.search_value = self.parse_search_hex();
                }
            }
            0x08 | 0x7F => { 
                if self.search_input_len > 0 {
                    self.search_input_len -= 1;
                    self.search_value = self.parse_search_hex();
                }
            }
            
            0x09 => {
                self.search_mode = match self.search_mode {
                    SearchMode::Exact => SearchMode::Changed,
                    SearchMode::Changed => SearchMode::Unchanged,
                    SearchMode::Unchanged => SearchMode::Greater,
                    SearchMode::Greater => SearchMode::Less,
                    SearchMode::Less => SearchMode::Exact,
                };
            }
            
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
            let blu = match c {
                b'0'..=b'9' => c - b'0',
                b'a'..=b'f' => c - b'a' + 10,
                b'A'..=b'F' => c - b'A' + 10,
                _ => 0,
            };
            val = (val << 4) | blu as u16;
        }
        val
    }

    pub fn handle_click(&mut self, da: i32, cm: i32, ca: u32, _wh: u32) {
        
        
        let acc = 22i32;
        let jnj = FI_ as i32 + acc;
        let jni = jnj + JX_ as i32;
        if cm >= jnj && cm < jni {
            let zm = 72i32;
            let bu = da - 4;
            if bu >= 0 {
                let pcs = bu / zm;
                match pcs {
                    0 => self.active_tab = LabTab::Analyze,
                    1 => self.active_tab = LabTab::Search,
                    2 => self.active_tab = LabTab::Watch,
                    3 => self.active_tab = LabTab::Tiles,
                    4 => self.active_tab = LabTab::Trace,
                    _ => {}
                }
            }
            
            let cdu = ca as i32 - 200;
            if da >= cdu && da < cdu + 24 {
                if self.speed_idx > 0 { self.speed_idx -= 1; }
            } else if da >= cdu + 28 && da < cdu + 52 {
                if self.speed_idx < 4 { self.speed_idx += 1; }
            }
            
            let dcj = ca as i32 - 130;
            if da >= dcj && da < dcj + 40 {
                self.paused = !self.paused;
            }
            
            let avf = ca as i32 - 86;
            if da >= avf && da < avf + 36 {
                self.step_one = true; self.paused = true;
            }
            
            let hzz = ca as i32 - 46;
            if da >= hzz && da < hzz + 42 {
                self.step_frame = true; self.paused = true;
            }
        }

        
        let bn = jni;

        
        if self.active_tab == LabTab::Search {
            
            let ioh = bn + 6 + L_ as i32 + 4 + L_ as i32 + 2;
            if cm >= ioh && cm < ioh + L_ as i32 {
                let cg = da - 8 - 48;
                if cg >= 0 {
                    
                    let ngc: [i32; 5] = [5*8+10, 7*8+10, 4*8+10, 7*8+10, 4*8+10];
                    let mut fga = 0i32;
                    for (i, &w) in ngc.iter().enumerate() {
                        if cg >= fga && cg < fga + w {
                            self.search_mode = match i {
                                0 => SearchMode::Exact,
                                1 => SearchMode::Changed,
                                2 => SearchMode::Unchanged,
                                3 => SearchMode::Greater,
                                _ => SearchMode::Less,
                            };
                            break;
                        }
                        fga += w;
                    }
                }
            }
            
            let dto = bn + 120;
            if cm >= dto && da >= (ca as i32 - 60) {
                let idx = ((cm - dto) / L_ as i32) as usize;
                if idx < self.search_results.len() {
                    self.add_watch(self.search_results[idx]);
                }
            }
        }

        
        if self.active_tab == LabTab::Tiles {
            let arr = bn + 6 + L_ as i32;
            if cm >= arr && cm < arr + L_ as i32 {
                let p = da - 8;
                if p >= 0 && p < 110 { self.tile_page = 0; }
                else if p >= 110 && p < 220 { self.tile_page = 1; }
                else if p >= 220 && p < 330 { self.tile_page = 2; }
            }
        }
    }
}


pub fn lix(
    state: &GameLabState,
    an: Option<&GameBoyEmulator>,
    wx: i32, wy: i32, ca: u32, er: u32,
) {
    let cx = wx as u32;
    let u = (wy + FI_ as i32) as u32;
    let aq = ca;
    let ch = er.saturating_sub(FI_);

    if aq < 200 || ch < 150 { return; }

    
    framebuffer::fill_rect(cx, u, aq, ch, IK_);

    
    framebuffer::fill_rect(cx, u, aq, 22, KJ_);
    let blink = (state.frame / 15) % 2 == 0;
    let lgz = if blink { BF_ } else { F_ };
    framebuffer::fill_rect(cx + 6, u + 8, 6, 6, lgz);
    bo(cx + 16, u + 4, "GAME LAB", BF_);

    
    if an.is_some() {
        bo(cx + 100, u + 4, "[LINKED]", BF_);
    } else {
        bo(cx + 100, u + 4, "[NO EMU]", AN_);
    }

    
    let bdn = cx + aq - 120;
    afc(bdn, u + 2, 48, 16, "SAVE", state.save_state.valid, M_);
    afc(bdn + 54, u + 2, 48, 16, "LOAD", state.save_state.valid, if state.save_state.valid { BF_ } else { F_ });

    
    let ty = u + 22;
    framebuffer::fill_rect(cx, ty, aq, JX_, TJ_);
    framebuffer::fill_rect(cx, ty + JX_ - 1, aq, 1, KI_);

    
    let tabs: [(&str, LabTab); 5] = [
        ("ANALYZE", LabTab::Analyze),
        ("SEARCH", LabTab::Search),
        ("WATCH", LabTab::Watch),
        ("TILES", LabTab::Tiles),
        ("TRACE", LabTab::Trace),
    ];
    let zm: u32 = 68;
    for (i, (label, tab)) in tabs.iter().enumerate() {
        let bu = cx + 4 + i as u32 * (zm + 4);
        let active = state.active_tab == *tab;
        let bg = if active { 0xFF1A3828 } else { TJ_ };
        framebuffer::fill_rect(bu, ty + 2, zm, JX_ - 4, bg);
        if active {
            framebuffer::fill_rect(bu, ty + JX_ - 3, zm, 2, BF_);
        }
        let col = if active { BF_ } else { F_ };
        bo(bu + 4, ty + 6, label, col);
    }

    
    let cdu = cx + aq - 200;
    afc(cdu, ty + 3, 22, 16, "<", false, M_);
    bo(cdu + 26, ty + 6, state.speed_label(), AG_);
    afc(cdu + 56, ty + 3, 22, 16, ">", false, M_);

    
    let dcj = cx + aq - 130;
    let nsl = if state.paused { AN_ } else { BF_ };
    let nsm = if state.paused { "PLAY" } else { "PAUS" };
    afc(dcj, ty + 3, 38, 16, nsm, state.paused, nsl);
    afc(dcj + 42, ty + 3, 34, 16, "STEP", false, AU_);
    afc(dcj + 80, ty + 3, 42, 16, "FRAME", false, BG_);

    
    let bn = ty + JX_;
    let en = ch.saturating_sub(22 + JX_);

    match state.active_tab {
        LabTab::Analyze => lky(state, an, cx, bn, aq, en),
        LabTab::Search => lkz(state, an, cx, bn, aq, en),
        LabTab::Watch => llc(state, an, cx, bn, aq, en),
        LabTab::Tiles => lla(state, an, cx, bn, aq, en),
        LabTab::Trace => llb(state, an, cx, bn, aq, en),
    }
}





fn lky(state: &GameLabState, an: Option<&GameBoyEmulator>, cx: u32, u: u32, aq: u32, ch: u32) {
    let ebz = ch * 60 / 100;
    let fjq = ch - ebz - 2;
    let col_w = (aq - 4) / 3;

    lke(an, cx + 1, u, col_w, ebz, state);
    lkf(an, cx + col_w + 2, u, col_w, ebz, state);
    lki(an, state, cx + col_w * 2 + 3, u, col_w, ebz);

    let dc = u + ebz + 2;
    lkh(an, cx + 1, dc, col_w, fjq, state);
    lkd(an, cx + col_w + 2, dc, col_w, fjq);
    lkg(an, cx + col_w * 2 + 3, dc, col_w, fjq, state);
}





fn lkz(state: &GameLabState, an: Option<&GameBoyEmulator>, cx: u32, u: u32, aq: u32, ch: u32) {
    let p = cx + 8;
    let mut o = u + 6;

    
    bo(p, o, "MEMORY SEARCH", M_);
    bo(p + 120, o, "(Hex value, Tab=mode, R=reset)", F_);
    o += L_ + 4;

    
    bo(p, o, "VALUE:", EQ_);
    let mut dsa = String::new();
    for i in 0..state.search_input_len as usize {
        dsa.push(state.search_input[i] as char);
    }
    if dsa.is_empty() { dsa.push_str("__"); }
    
    if (state.frame / 20) % 2 == 0 { dsa.push('_'); }
    bo(p + 52, o, &dsa, AG_);

    
    let vs = format!("= {} (0x{:02X})", state.search_value, state.search_value);
    bo(p + 120, o, &vs, F_);
    o += L_ + 2;

    
    bo(p, o, "MODE:", EQ_);
    let modes = [
        ("EXACT", SearchMode::Exact),
        ("CHANGED", SearchMode::Changed),
        ("SAME", SearchMode::Unchanged),
        ("GREATER", SearchMode::Greater),
        ("LESS", SearchMode::Less),
    ];
    let mut cg = p + 48;
    for (label, mode) in &modes {
        let active = state.search_mode == *mode;
        let col = if active { BF_ } else { F_ };
        if active { framebuffer::fill_rect(cg - 2, o - 1, label.len() as u32 * CE_ + 4, L_, 0xFF1A3020); }
        bo(cg, o, label, col);
        cg += label.len() as u32 * CE_ + 10;
    }
    o += L_ + 2;

    
    bo(p, o, "Enter=Scan/Filter", M_);
    bo(p + 152, o, "R=Reset", NI_);
    o += L_ + 6;

    
    let status = if !state.search_active {
        String::from("No scan yet. Type value + Enter to scan WRAM.")
    } else {
        format!("Results: {} addresses", state.search_results.len())
    };
    bo(p, o, &status, P_);
    o += L_ + 4;

    
    if state.search_active {
        framebuffer::fill_rect(cx + 4, o, aq - 8, 1, KI_);
        o += 4;
        bo(p, o, "ADDR", M_);
        bo(p + 60, o, "VALUE", M_);
        bo(p + 110, o, "DEC", M_);
        if aq > 400 { bo(p + 160, o, "PREV", F_); }
        bo(cx + aq - 68, o, "[+WATCH]", BF_);
        o += L_ + 2;

        let xw = ((ch.saturating_sub(o - u)) / L_).min(32) as usize;
        for (i, &addr) in state.search_results.iter().take(xw).enumerate() {
            let val = if let Some(e) = an { aik(e, addr) } else { 0 };
            let prev = state.snapshot_byte(addr);
            let changed = val != prev;

            let bqd = format!("{:04X}", addr);
            bo(p, o, &bqd, IJ_);

            let hbe = format!("{:02X}", val);
            let pqz = if changed { TF_ } else { AG_ };
            bo(p + 60, o, &hbe, pqz);

            let fqz = format!("{:3}", val);
            bo(p + 110, o, &fqz, F_);

            if aq > 400 {
                let gob = format!("{:02X}", prev);
                bo(p + 160, o, &gob, F_);
            }

            
            let hcf = cx + aq - 48;
            bo(hcf, o, "+W", BF_);

            o += L_;
            let _ = i;
        }
        if state.search_results.len() > xw {
            let ngf = format!("... +{} more", state.search_results.len() - xw);
            bo(p, o, &ngf, F_);
        }
    }
}





fn llc(state: &GameLabState, an: Option<&GameBoyEmulator>, cx: u32, u: u32, aq: u32, ch: u32) {
    let p = cx + 8;
    let mut o = u + 6;

    bo(p, o, "WATCH LIST", M_);
    let kye = format!("{}/{}", state.watches.len(), BCI_);
    bo(p + 100, o, &kye, F_);
    bo(p + 160, o, "(Backspace=remove last)", F_);
    o += L_ + 4;

    if state.watches.is_empty() {
        bo(p, o, "No watches. Add from Search tab with [+W] button.", F_);
        return;
    }

    
    framebuffer::fill_rect(cx + 4, o, aq - 8, 1, KI_);
    o += 4;
    bo(p, o, "LABEL", M_);
    bo(p + 72, o, "ADDR", M_);
    bo(p + 120, o, "HEX", M_);
    bo(p + 160, o, "DEC", M_);
    bo(p + 200, o, "PREV", M_);
    if aq > 500 { bo(p + 250, o, "VISUAL", F_); }
    o += L_ + 2;

    for w in state.watches.iter() {
        let cmb: String = w.label[..w.label_len as usize].iter().map(|&b| b as char).collect();
        bo(p, o, &cmb, EQ_);

        let bqd = format!("{:04X}", w.addr);
        bo(p + 72, o, &bqd, IJ_);

        let val = if let Some(e) = an { aik(e, w.addr) } else { w.cur_value };
        let hbe = format!("{:02X}", val);
        let col = if w.changed { TF_ } else { AG_ };
        bo(p + 120, o, &hbe, col);

        let fqz = format!("{:3}", val);
        bo(p + 160, o, &fqz, col);

        let gob = format!("{:02X}", w.prev_value);
        bo(p + 200, o, &gob, F_);

        
        if aq > 500 {
            let ek = 100u32;
            let fill = (val as u32 * ek) / 255;
            framebuffer::fill_rect(p + 250, o + 2, ek, 8, 0xFF0A1A10);
            let fie = if w.changed { TF_ } else { BF_ };
            framebuffer::fill_rect(p + 250, o + 2, fill, 8, fie);
        }

        o += L_;
    }
}





fn lla(state: &GameLabState, an: Option<&GameBoyEmulator>, cx: u32, u: u32, aq: u32, ch: u32) {
    let p = cx + 8;
    let mut o = u + 6;

    let acg = ["TILES $8000", "TILES $8800", "OAM SPRITES"];
    bo(p, o, "TILE VIEWER", M_);
    o += L_;

    
    for (i, label) in acg.iter().enumerate() {
        let active = i as u8 == state.tile_page;
        let col = if active { BF_ } else { F_ };
        let bu = p + i as u32 * 110;
        if active { framebuffer::fill_rect(bu - 2, o - 1, label.len() as u32 * CE_ + 4, L_, 0xFF1A3020); }
        bo(bu, o, label, col);
    }
    bo(p + 340, o, "(Tab=page, Arrows=scroll)", F_);
    o += L_ + 6;

    let an = match an {
        Some(e) => e,
        None => { bo(p, o, "No emulator linked", F_); return; }
    };

    if state.tile_page < 2 {
        
        let base_addr: u16 = if state.tile_page == 0 { 0x8000 } else { 0x8800 };
        let tile_size = 2u32; 
        let ebr = 8 * tile_size;
        let cols = ((aq - 20) / (ebr + 1)).min(16);
        let xw = ((ch - (o - u) - 4) / (ebr + 1)).min(16);
        let scroll = state.tile_scroll.min(16u32.saturating_sub(xw));

        for row in 0..xw {
            for col in 0..cols {
                let jml = (scroll + row) * 16 + col;
                if jml >= 256 { break; }
                let ako = base_addr.wrapping_add(jml as u16 * 16);
                let dx = p + col * (ebr + 1);
                let ad = o + row * (ebr + 1);

                
                for ty_off in 0..8u32 {
                    let lo = aik(an, ako.wrapping_add(ty_off as u16 * 2));
                    let hi = aik(an, ako.wrapping_add(ty_off as u16 * 2 + 1));
                    for tx_off in 0..8u32 {
                        let bf = 7 - tx_off;
                        let alf = ((hi >> bf) & 1) << 1 | ((lo >> bf) & 1);
                        let shade = match alf {
                            0 => 0xFF0A1510,
                            1 => 0xFF346856,
                            2 => 0xFF88C070,
                            3 => 0xFFE0F8D0,
                            _ => 0xFF000000,
                        };
                        framebuffer::fill_rect(
                            dx + tx_off * tile_size,
                            ad + ty_off * tile_size,
                            tile_size, tile_size, shade,
                        );
                    }
                }
            }
        }

        
        let clw = o + xw * (ebr + 1) + 4;
        let obh = format!("Tiles {}-{}", scroll * 16, ((scroll + xw) * 16).min(256) - 1);
        bo(p, clw, &obh, F_);
    } else {
        
        bo(p, o, "#  Y   X   TILE FLAGS", M_);
        o += L_;

        let ndm = ((ch - (o - u)) / L_).min(40);
        for i in 0..ndm {
            let oam_addr = 0xFE00u16 + i as u16 * 4;
            let ak = aik(an, oam_addr);
            let am = aik(an, oam_addr + 1);
            let apf = aik(an, oam_addr + 2);
            let flags = aik(an, oam_addr + 3);

            let visible = ak > 0 && ak < 160 && am > 0 && am < 168;
            let col = if visible { AG_ } else { F_ };

            let j = format!("{:2} {:3} {:3}  {:02X}   {:02X}", i, ak, am, apf, flags);
            bo(p, o, &j, col);

            
            let emv = p + 200;
            if flags & 0x80 != 0 { bo(emv, o, "P", NI_); }
            if flags & 0x40 != 0 { bo(emv + 12, o, "Y", M_); }
            if flags & 0x20 != 0 { bo(emv + 24, o, "X", M_); }
            if an.cgb_mode {
                let ewa = flags & 0x07;
                let gi = (flags >> 3) & 1;
                let exe = format!("P{} B{}", ewa, gi);
                bo(emv + 40, o, &exe, AU_);
            }

            o += L_;
        }
    }
}





fn llb(state: &GameLabState, an: Option<&GameBoyEmulator>, cx: u32, u: u32, aq: u32, ch: u32) {
    let p = cx + 8;
    let mut o = u + 6;

    bo(p, o, "TRACE LOG", M_);
    let lpx = if state.trace_enabled { "[ON]" } else { "[OFF]" };
    let lpw = if state.trace_enabled { BF_ } else { AN_ };
    bo(p + 88, o, lpx, lpw);
    bo(p + 132, o, "(T=toggle, R=clear)", F_);
    o += L_ + 2;

    
    if let Some(an) = an {
        bo(p, o, "DISASSEMBLY @ PC", M_);
        o += L_;
        let pc = an.cpu.pc;
        
        let mut addr = pc.wrapping_sub(8);
        let lfc = 12u32.min((ch / 3) / L_);
        for _ in 0..lfc {
            let opcode = aik(an, addr);
            let is_current = addr == pc;
            let nm = if is_current { ">" } else { " " };
            let (mnemonic, size) = lfb(an, addr);
            let j = format!("{} {:04X}: {:02X}  {}", nm, addr, opcode, mnemonic);
            let col = if is_current { BF_ } else { P_ };
            if is_current {
                framebuffer::fill_rect(p - 2, o - 1, aq - 16, L_, 0xFF1A3020);
            }
            bo(p, o, &j, col);

            
            if state.breakpoints.iter().any(|&bp| bp == addr) {
                framebuffer::fill_rect(p - 6, o + 2, 4, 8, AN_);
            }

            o += L_;
            addr = addr.wrapping_add(size as u16);
        }
    }
    o += 6;
    framebuffer::fill_rect(cx + 4, o, aq - 8, 1, KI_);
    o += 4;

    
    bo(p, o, "BREAKPOINTS", M_);
    let kds = format!("{}/{}", state.breakpoints.len(), CIN_);
    bo(p + 100, o, &kds, F_);
    o += L_;

    if state.breakpoints.is_empty() {
        bo(p, o, "None (type addr in Search, click to add)", F_);
    } else {
        let mut djn = p;
        for &bp in state.breakpoints.iter() {
            let djm = format!("{:04X}", bp);
            framebuffer::fill_rect(djn, o, 40, L_ - 2, 0xFF2A0A0A);
            bo(djn + 2, o, &djm, AN_);
            djn += 48;
            if djn > cx + aq - 60 { o += L_; djn = p; }
        }
    }
    o += L_ + 4;
    framebuffer::fill_rect(cx + 4, o, aq - 8, 1, KI_);
    o += 4;

    
    bo(p, o, "TRACE HISTORY", M_);
    let pmm = format!("({} entries)", state.trace.len());
    bo(p + 120, o, &pmm, F_);
    o += L_;

    bo(p, o, "PC    OP  A  F  SP", F_);
    o += L_;

    let xw = ((ch.saturating_sub(o - u)) / L_) as usize;
    let start = if state.trace.len() > xw { state.trace.len() - xw } else { 0 };
    for entry in state.trace[start..].iter() {
        let j = format!("{:04X}  {:02X}  {:02X} {:02X} {:04X}", entry.pc, entry.opcode, entry.a, entry.f, entry.sp);
        bo(p, o, &j, P_);
        o += L_;
        if o + L_ > u + ch { break; }
    }
}


fn lke(an: Option<&GameBoyEmulator>, x: u32, y: u32, w: u32, h: u32, state: &GameLabState) {
    blx(x, y, w, h, "CPU REGISTERS", state.selected_panel == 0);

    let p = x + EK_ + 2;
    let mut o = y + 20;

    if let Some(an) = an {
        let cpu = &an.cpu;

        
        let regs = [
            ("AF", cpu.a, cpu.f),
            ("BC", cpu.b, cpu.c),
            ("DE", cpu.d, cpu.e),
            ("HL", cpu.h, cpu.l),
        ];
        for (name, hi, lo) in &regs {
            bo(p, o, name, EQ_);
            let j = format!("{:02X}{:02X}", hi, lo);
            bo(p + 28, o, &j, AG_);
            
            let azn = format!("({:3} {:3})", hi, lo);
            bo(p + 72, o, &azn, F_);
            o += L_;
        }

        o += 4;
        
        bo(p, o, "SP", EQ_);
        let ove = format!("{:04X}", cpu.sp);
        bo(p + 28, o, &ove, AG_);
        o += L_;

        bo(p, o, "PC", EQ_);
        let nsv = format!("{:04X}", cpu.pc);
        bo(p + 28, o, &nsv, M_);

        
        if an.rom_loaded {
            let opcode = aik(an, cpu.pc);
            let ops = format!("[{:02X}]", opcode);
            bo(p + 72, o, &ops, AU_);
        }
        o += L_ + 6;

        
        bo(p, o, "FLAGS", F_);
        o += L_;
        let flags = [
            ("Z", cpu.f & 0x80 != 0),
            ("N", cpu.f & 0x40 != 0),
            ("H", cpu.f & 0x20 != 0),
            ("C", cpu.f & 0x10 != 0),
        ];
        let mut dg = p;
        for (name, set) in &flags {
            let color = if *set { DZ_ } else { TG_ };
            framebuffer::fill_rect(dg, o, 24, 14, if *set { 0xFF0A3020 } else { 0xFF0A1510 });
            bo(dg + 4, o + 1, name, color);
            dg += 28;
        }
        o += L_ + 6;

        
        bo(p, o, "IME", F_);
        bo(p + 32, o, if cpu.ime { "ON" } else { "OFF" }, 
            if cpu.ime { DZ_ } else { TG_ });
        bo(p + 64, o, "HALT", F_);
        bo(p + 100, o, if cpu.halted { "YES" } else { "NO" },
            if cpu.halted { NI_ } else { F_ });
        o += L_;

        
        bo(p, o, "CYCLES", F_);
        let cs = format!("{}", cpu.cycles);
        bo(p + 56, o, &cs, AG_);
        o += L_;

        
        if an.cgb_mode {
            bo(p, o, "MODE", F_);
            bo(p + 40, o, "CGB", BF_);
            let dzw = if an.key1 & 0x80 != 0 { "2x" } else { "1x" };
            bo(p + 72, o, dzw, M_);
        }
    } else {
        bo(p, o, "No emulator linked", F_);
    }
}


fn lkf(an: Option<&GameBoyEmulator>, x: u32, y: u32, w: u32, h: u32, state: &GameLabState) {
    blx(x, y, w, h, "GPU / LCD", state.selected_panel == 1);

    let p = x + EK_ + 2;
    let mut o = y + 20;

    if let Some(an) = an {
        let gpu = &an.gpu;

        
        bo(p, o, "LCDC", EQ_);
        let ls = format!("{:02X}", gpu.lcdc);
        bo(p + 40, o, &ls, AG_);
        let ijq = gpu.lcdc & 0x80 != 0;
        bo(p + 64, o, if ijq { "LCD:ON" } else { "LCD:OFF" },
            if ijq { DZ_ } else { AN_ });
        o += L_;

        
        let bits = [
            ("BG", gpu.lcdc & 0x01 != 0),
            ("OBJ", gpu.lcdc & 0x02 != 0),
            ("8x16", gpu.lcdc & 0x04 != 0),
            ("WIN", gpu.lcdc & 0x20 != 0),
        ];
        let mut bx = p;
        for (name, on) in &bits {
            let col = if *on { DZ_ } else { TG_ };
            bo(bx, o, name, col);
            bx += (name.len() as u32 + 1) * CE_;
        }
        o += L_ + 4;

        
        bo(p, o, "LY", EQ_);
        let nbn = format!("{:3} / 153", gpu.ly);
        bo(p + 28, o, &nbn, AG_);
        
        let pv = p + 110;
        let ek = w.saturating_sub(130);
        framebuffer::fill_rect(pv, o + 2, ek, 8, 0xFF0A1A10);
        let progress = (gpu.ly as u32 * ek) / 154;
        let fie = if gpu.ly < 144 { BF_ } else { NI_ };
        framebuffer::fill_rect(pv, o + 2, progress.min(ek), 8, fie);
        o += L_;

        bo(p, o, "LYC", F_);
        let nbm = format!("{:3}", gpu.lyc);
        bo(p + 32, o, &nbm, AG_);
        if gpu.ly == gpu.lyc {
            bo(p + 60, o, "=MATCH", BF_);
        }
        o += L_;

        
        bo(p, o, "MODE", F_);
        let (bcu, mode_col) = match gpu.mode {
            0 => ("HBLANK", F_),
            1 => ("VBLANK", NI_),
            2 => ("OAM", M_),
            3 => ("DRAW", BF_),
            _ => ("???", AN_),
        };
        bo(p + 40, o, bcu, mode_col);
        let lba = format!("({} dots)", gpu.cycles);
        bo(p + 96, o, &lba, F_);
        o += L_ + 4;

        
        bo(p, o, "SCX/Y", F_);
        let ss = format!("{:3},{:3}", gpu.scx, gpu.scy);
        bo(p + 48, o, &ss, AG_);
        o += L_;
        bo(p, o, "WX/Y", F_);
        let asv = format!("{:3},{:3}", gpu.wx, gpu.wy);
        bo(p + 48, o, &asv, AG_);
        o += L_ + 4;

        
        bo(p, o, "BGP", F_);
        fsz(p + 32, o, gpu.bgp);
        o += L_;
        bo(p, o, "OBP0", F_);
        fsz(p + 40, o, gpu.obp0);
        o += L_;
        bo(p, o, "OBP1", F_);
        fsz(p + 40, o, gpu.obp1);
        o += L_ + 4;

        
        if an.cgb_mode {
            bo(p, o, "CGB BG PALETTES", F_);
            o += L_;
            for ewa in 0..8 {
                let nwq = p + ewa * 36;
                for c in 0..4u32 {
                    let offset = (ewa * 8 + c * 2) as usize;
                    if offset + 1 < gpu.bg_palette.len() {
                        let lo = gpu.bg_palette[offset] as u16;
                        let hi = gpu.bg_palette[offset + 1] as u16;
                        let cpj = lo | (hi << 8);
                        let r = (((cpj & 0x1F) as u8) << 3) as u32;
                        let g = ((((cpj >> 5) & 0x1F) as u8) << 3) as u32;
                        let b = ((((cpj >> 10) & 0x1F) as u8) << 3) as u32;
                        let color = 0xFF000000 | (r << 16) | (g << 8) | b;
                        framebuffer::fill_rect(nwq + c * 8, o, 7, 8, color);
                    }
                }
            }
            o += 12;

            
            bo(p, o, "VRAM BANK", F_);
            let prg = format!("{}", gpu.vram_bank);
            bo(p + 80, o, &prg, AG_);
        }
    } else {
        bo(p, o, "No emulator linked", F_);
    }
}


fn lki(an: Option<&GameBoyEmulator>, state: &GameLabState, x: u32, y: u32, w: u32, h: u32) {
    blx(x, y, w, h, "MEMORY", state.selected_panel == 2);

    let p = x + EK_ + 2;
    let mut o = y + 20;

    
    let modes = ["WRAM", "VRAM", "HRAM", "ROM", "OAM"];
    let mut bu = p;
    for (i, name) in modes.iter().enumerate() {
        let selected = i as u8 == state.mem_mode;
        let col = if selected { BF_ } else { F_ };
        if selected {
            framebuffer::fill_rect(bu, o, name.len() as u32 * CE_ + 4, L_, 0xFF1A3020);
        }
        bo(bu + 2, o, name, col);
        bu += name.len() as u32 * CE_ + 8;
    }
    o += L_ + 4;

    
    let juc = format!("ADDR  {:04X}", state.mem_view_addr);
    bo(p, o, &juc, M_);
    o += L_;

    if let Some(an) = an {
        
        let rows = ((h - 60) / L_).min(16) as u16;
        for row in 0..rows {
            let addr = state.mem_view_addr.wrapping_add(row * 16);
            
            let bqd = format!("{:04X}", addr);
            bo(p, o, &bqd, IJ_);

            
            let mut aib = p + 40;
            let mut hfs = [b'.'; 16];
            for col in 0..16u16 {
                let a = addr.wrapping_add(col);
                let byte = aik(an, a);
                let gbf = format!("{:02X}", byte);
                
                let hsd = (row * 16 + col) as usize;
                let prev = if hsd < state.mem_prev.len() { state.mem_prev[hsd] } else { byte };
                let kgk = if byte != prev { TF_ } else if byte == 0 { F_ } else { AG_ };
                bo(aib, o, &gbf, kgk);
                aib += 20;
                if col == 7 { aib += 4; } 

                if byte >= 0x20 && byte < 0x7F {
                    hfs[col as usize] = byte;
                }
            }

            
            if w > 420 {
                let ascii: alloc::string::String = hfs.iter().map(|&b| b as char).collect();
                bo(aib + 8, o, &ascii, F_);
            }

            o += L_;
        }
    } else {
        bo(p, o, "No emulator linked", F_);
    }
}


fn lkh(an: Option<&GameBoyEmulator>, x: u32, y: u32, w: u32, h: u32, state: &GameLabState) {
    blx(x, y, w, h, "I/O REGISTERS", state.selected_panel == 3);

    let p = x + EK_ + 2;
    let mut o = y + 20;

    if let Some(an) = an {
        
        bo(p, o, "INTERRUPTS", M_);
        o += L_;

        bo(p, o, "IE", EQ_);
        let mnz = format!("{:02X}", an.ie_reg);
        bo(p + 24, o, &mnz, AG_);

        bo(p + 50, o, "IF", EQ_);
        let moa = format!("{:02X}", an.if_reg);
        bo(p + 74, o, &moa, AG_);
        o += L_;

        
        let mqt = ["VBL", "STA", "TIM", "SER", "JOY"];
        let mut bi = p;
        for (i, name) in mqt.iter().enumerate() {
            let drt = an.ie_reg & (1 << i) != 0;
            let iflag = an.if_reg & (1 << i) != 0;
            let col = if drt && iflag { AN_ } else if drt { DZ_ } else { TG_ };
            bo(bi, o, name, col);
            bi += 32;
        }
        o += L_ + 6;

        
        bo(p, o, "TIMER", M_);
        o += L_;

        bo(p, o, "DIV", F_);
        let lgd = format!("{:02X}", an.timer.read_div());
        bo(p + 32, o, &lgd, AG_);

        bo(p + 60, o, "TIMA", F_);
        let pjl = format!("{:02X}", an.timer.tima);
        bo(p + 100, o, &pjl, AG_);
        o += L_;

        bo(p, o, "TMA", F_);
        let pki = format!("{:02X}", an.timer.tma);
        bo(p + 32, o, &pki, AG_);

        bo(p + 60, o, "TAC", F_);
        let pcx = format!("{:02X}", an.timer.tac);
        bo(p + 100, o, &pcx, AG_);
        o += L_ + 6;

        
        bo(p, o, "SERIAL", M_);
        o += L_;
        bo(p, o, "SB", F_);
        let oko = format!("{:02X}", an.serial_data);
        bo(p + 24, o, &oko, AG_);
        bo(p + 50, o, "SC", F_);
        let oly = format!("{:02X}", an.serial_ctrl);
        bo(p + 74, o, &oly, AG_);

        if an.cgb_mode {
            o += L_ + 6;
            bo(p, o, "CGB I/O", M_);
            o += L_;
            bo(p, o, "KEY1", F_);
            let mvl = format!("{:02X}", an.key1);
            bo(p + 40, o, &mvl, AG_);
            bo(p + 68, o, "WRAM", F_);
            let pui = format!("BK{}", an.wram_bank);
            bo(p + 104, o, &pui, AG_);
        }
    } else {
        bo(p, o, "No emulator linked", F_);
    }
}


fn lkd(an: Option<&GameBoyEmulator>, x: u32, y: u32, w: u32, h: u32) {
    blx(x, y, w, h, "CARTRIDGE", false);

    let p = x + EK_ + 2;
    let mut o = y + 20;

    if let Some(an) = an {
        let cart = &an.cart;

        
        let pka: alloc::vec::Vec<u8> = cart.title.iter().copied()
            .take_while(|&c| c != 0 && c >= 0x20).collect();
        let title = core::str::from_utf8(&pka).unwrap_or("???");
        bo(p, o, "TITLE", F_);
        bo(p + 48, o, title, BF_);
        o += L_;

        
        let ndt = match cart.mbc_type {
            crate::gameboy::cartridge::MbcType::None => "ROM ONLY",
            crate::gameboy::cartridge::MbcType::Mbc1 => "MBC1",
            crate::gameboy::cartridge::MbcType::Mbc3 => "MBC3",
            crate::gameboy::cartridge::MbcType::Mbc5 => "MBC5",
        };
        bo(p, o, "MBC", F_);
        bo(p + 32, o, ndt, AG_);
        o += L_;

        
        let ohx = cart.rom.len() / 1024;
        let obc = cart.ram.len() / 1024;
        bo(p, o, "ROM", F_);
        let oc = format!("{}KB", ohx);
        bo(p + 32, o, &oc, AG_);
        bo(p + 80, o, "RAM", F_);
        let obk = format!("{}KB", obc);
        bo(p + 112, o, &obk, AG_);
        o += L_;

        
        bo(p, o, "ROM BANK", F_);
        let obw = format!("{:3}", cart.rom_bank);
        bo(p + 72, o, &obw, AG_);
        let pll = cart.rom.len() / 16384;
        let tbs = format!("/ {}", pll);
        bo(p + 96, o, &tbs, F_);
        o += L_;

        bo(p, o, "RAM BANK", F_);
        let ohq = format!("{:3}", cart.ram_bank);
        bo(p + 72, o, &ohq, AG_);
        o += L_;

        
        bo(p, o, "CGB", F_);
        let kij = match cart.cgb_flag {
            0xC0 => "CGB ONLY",
            0x80 => "CGB+DMG",
            _ => "DMG",
        };
        let kii = if cart.cgb_flag >= 0x80 { M_ } else { F_ };
        bo(p + 32, o, kij, kii);
    } else {
        bo(p, o, "No cartridge", F_);
    }
}


fn lkg(an: Option<&GameBoyEmulator>, x: u32, y: u32, w: u32, h: u32, state: &GameLabState) {
    blx(x, y, w, h, "INPUT STATE", state.selected_panel == 5);

    let p = x + EK_ + 2;
    let mut o = y + 20;

    if let Some(an) = an {
        bo(p, o, "JOYPAD $FF00", M_);
        let mve = format!("{:02X}", an.joypad_reg);
        bo(p + 104, o, &mve, AG_);
        o += L_ + 4;

        
        bo(p, o, "D-PAD", F_);
        o += L_;
        let up    = an.joypad_dirs & 0x04 == 0;
        let dno  = an.joypad_dirs & 0x08 == 0;
        let left  = an.joypad_dirs & 0x02 == 0;
        let right = an.joypad_dirs & 0x01 == 0;

        
        let dx = p + 16;
        let ad = o;
        let fq: u32 = 16;
        
        framebuffer::fill_rect(dx + fq, ad, fq, fq, if up { DZ_ } else { 0xFF1A2820 });
        bo(dx + fq + 3, ad + 2, "U", if up { 0xFF000000 } else { F_ });
        
        framebuffer::fill_rect(dx + fq, ad + fq * 2, fq, fq, if dno { DZ_ } else { 0xFF1A2820 });
        bo(dx + fq + 3, ad + fq * 2 + 2, "D", if dno { 0xFF000000 } else { F_ });
        
        framebuffer::fill_rect(dx, ad + fq, fq, fq, if left { DZ_ } else { 0xFF1A2820 });
        bo(dx + 3, ad + fq + 2, "L", if left { 0xFF000000 } else { F_ });
        
        framebuffer::fill_rect(dx + fq * 2, ad + fq, fq, fq, if right { DZ_ } else { 0xFF1A2820 });
        bo(dx + fq * 2 + 3, ad + fq + 2, "R", if right { 0xFF000000 } else { F_ });
        
        framebuffer::fill_rect(dx + fq, ad + fq, fq, fq, 0xFF1A2820);

        
        let bx = p + 100;
        let eer = an.joypad_buttons & 0x01 == 0;
        let efu = an.joypad_buttons & 0x02 == 0;
        let ezx = an.joypad_buttons & 0x04 == 0;
        let fbo = an.joypad_buttons & 0x08 == 0;

        
        framebuffer::fill_rect(bx + 32, ad + 4, 22, 22, if eer { DZ_ } else { 0xFF1A2820 });
        bo(bx + 38, ad + 8, "A", if eer { 0xFF000000 } else { AG_ });

        
        framebuffer::fill_rect(bx, ad + 16, 22, 22, if efu { DZ_ } else { 0xFF1A2820 });
        bo(bx + 6, ad + 20, "B", if efu { 0xFF000000 } else { AG_ });

        o = ad + fq * 3 + 6;

        
        let ezy = p + 20;
        framebuffer::fill_rect(ezy, o, 40, 14, if ezx { M_ } else { 0xFF1A2820 });
        bo(ezy + 4, o + 1, "SEL", if ezx { 0xFF000000 } else { F_ });

        framebuffer::fill_rect(ezy + 48, o, 48, 14, if fbo { M_ } else { 0xFF1A2820 });
        bo(ezy + 52, o + 1, "START", if fbo { 0xFF000000 } else { F_ });
        o += L_ + 8;

        
        bo(p, o, "DIRS", F_);
        let ds = format!("{:02X}", an.joypad_dirs);
        bo(p + 40, o, &ds, AG_);
        bo(p + 68, o, "BTNS", F_);
        let fjv = format!("{:02X}", an.joypad_buttons);
        bo(p + 108, o, &fjv, AG_);
    } else {
        bo(p, o, "No emulator linked", F_);
    }
}



fn blx(x: u32, y: u32, w: u32, h: u32, title: &str, selected: bool) {
    
    framebuffer::fill_rect(x, y, w, h, BQQ_);
    
    let ehe = if selected { BF_ } else { KI_ };
    framebuffer::fill_rect(x, y, w, 1, ehe);
    framebuffer::fill_rect(x, y + h - 1, w, 1, ehe);
    framebuffer::fill_rect(x, y, 1, h, ehe);
    framebuffer::fill_rect(x + w - 1, y, 1, h, ehe);
    
    framebuffer::fill_rect(x + 1, y + 1, w - 2, 16, KJ_);
    bo(x + 6, y + 3, title, if selected { BF_ } else { M_ });
}

fn bo(x: u32, y: u32, text: &str, color: u32) {
    let mut cx = x;
    for ch in text.chars() {
        framebuffer::px(cx, y, ch, color);
        cx += CE_;
    }
}

fn fsz(x: u32, y: u32, palette: u8) {
    
    const BZR_: [u32; 4] = [0xFFE0F8D0, 0xFF88C070, 0xFF346856, 0xFF081820];
    for i in 0..4u32 {
        let shade = (palette >> (i * 2)) & 3;
        framebuffer::fill_rect(x + i * 16, y + 1, 14, 10, BZR_[shade as usize]);
    }
}


pub fn aik(an: &GameBoyEmulator, addr: u16) -> u8 {
    match addr {
        0x0000..=0x7FFF => an.cart.read(addr),
        0x8000..=0x9FFF => an.gpu.read_vram(addr),
        0xA000..=0xBFFF => an.cart.read(addr),
        0xC000..=0xCFFF => {
            let idx = (addr as usize) - 0xC000;
            if idx < an.wram.len() { an.wram[idx] } else { 0xFF }
        }
        0xD000..=0xDFFF => {
            let gi = an.wram_bank.max(1) as usize;
            let offset = gi * 0x1000 + (addr as usize - 0xD000);
            if offset < an.wram.len() { an.wram[offset] } else { 0xFF }
        }
        0xFE00..=0xFE9F => an.gpu.read_oam(addr),
        0xFF80..=0xFFFE => {
            let idx = (addr - 0xFF80) as usize;
            if idx < an.hram.len() { an.hram[idx] } else { 0xFF }
        }
        0xFFFF => an.ie_reg,
        _ => 0xFF,
    }
}


pub fn jpg(state: &mut GameLabState, an: &GameBoyEmulator) {
    for i in 0..256usize {
        let addr = state.mem_view_addr.wrapping_add(i as u16);
        state.mem_prev[i] = aik(an, addr);
    }
}


fn afc(x: u32, y: u32, w: u32, h: u32, label: &str, active: bool, color: u32) {
    let bg = if active { 0xFF1A3020 } else { 0xFF0E1820 };
    framebuffer::fill_rect(x, y, w, h, bg);
    framebuffer::fill_rect(x, y, w, 1, color & 0x40FFFFFF);
    framebuffer::fill_rect(x, y + h - 1, w, 1, color & 0x40FFFFFF);
    framebuffer::fill_rect(x, y, 1, h, color & 0x40FFFFFF);
    framebuffer::fill_rect(x + w - 1, y, 1, h, color & 0x40FFFFFF);
    let acy = label.len() as u32 * CE_;
    bo(x + (w.saturating_sub(acy)) / 2, y + (h.saturating_sub(12)) / 2, label, color);
}


fn lfb(an: &GameBoyEmulator, addr: u16) -> (String, u8) {
    let op = aik(an, addr);
    let gf = aik(an, addr.wrapping_add(1));
    let iq = aik(an, addr.wrapping_add(2));
    let alt = (iq as u16) << 8 | gf as u16;

    match op {
        0x00 => (String::from("NOP"), 1),
        0x01 => (format!("LD BC,${:04X}", alt), 3),
        0x02 => (String::from("LD (BC),A"), 1),
        0x03 => (String::from("INC BC"), 1),
        0x04 => (String::from("INC B"), 1),
        0x05 => (String::from("DEC B"), 1),
        0x06 => (format!("LD B,${:02X}", gf), 2),
        0x07 => (String::from("RLCA"), 1),
        0x08 => (format!("LD (${:04X}),SP", alt), 3),
        0x09 => (String::from("ADD HL,BC"), 1),
        0x0A => (String::from("LD A,(BC)"), 1),
        0x0B => (String::from("DEC BC"), 1),
        0x0C => (String::from("INC C"), 1),
        0x0D => (String::from("DEC C"), 1),
        0x0E => (format!("LD C,${:02X}", gf), 2),
        0x0F => (String::from("RRCA"), 1),
        0x10 => (String::from("STOP"), 2),
        0x11 => (format!("LD DE,${:04X}", alt), 3),
        0x12 => (String::from("LD (DE),A"), 1),
        0x13 => (String::from("INC DE"), 1),
        0x16 => (format!("LD D,${:02X}", gf), 2),
        0x18 => (format!("JR ${:02X}", gf), 2),
        0x1A => (String::from("LD A,(DE)"), 1),
        0x1E => (format!("LD E,${:02X}", gf), 2),
        0x20 => (format!("JR NZ,${:02X}", gf), 2),
        0x21 => (format!("LD HL,${:04X}", alt), 3),
        0x22 => (String::from("LD (HL+),A"), 1),
        0x23 => (String::from("INC HL"), 1),
        0x26 => (format!("LD H,${:02X}", gf), 2),
        0x28 => (format!("JR Z,${:02X}", gf), 2),
        0x2A => (String::from("LD A,(HL+)"), 1),
        0x2E => (format!("LD L,${:02X}", gf), 2),
        0x2F => (String::from("CPL"), 1),
        0x30 => (format!("JR NC,${:02X}", gf), 2),
        0x31 => (format!("LD SP,${:04X}", alt), 3),
        0x32 => (String::from("LD (HL-),A"), 1),
        0x33 => (String::from("INC SP"), 1),
        0x36 => (format!("LD (HL),${:02X}", gf), 2),
        0x38 => (format!("JR C,${:02X}", gf), 2),
        0x3C => (String::from("INC A"), 1),
        0x3D => (String::from("DEC A"), 1),
        0x3E => (format!("LD A,${:02X}", gf), 2),
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
        0xC2 => (format!("JP NZ,${:04X}", alt), 3),
        0xC3 => (format!("JP ${:04X}", alt), 3),
        0xC4 => (format!("CALL NZ,${:04X}", alt), 3),
        0xC5 => (String::from("PUSH BC"), 1),
        0xC6 => (format!("ADD A,${:02X}", gf), 2),
        0xC8 => (String::from("RET Z"), 1),
        0xC9 => (String::from("RET"), 1),
        0xCA => (format!("JP Z,${:04X}", alt), 3),
        0xCB => (format!("CB {:02X}", gf), 2),
        0xCC => (format!("CALL Z,${:04X}", alt), 3),
        0xCD => (format!("CALL ${:04X}", alt), 3),
        0xCE => (format!("ADC A,${:02X}", gf), 2),
        0xD0 => (String::from("RET NC"), 1),
        0xD1 => (String::from("POP DE"), 1),
        0xD2 => (format!("JP NC,${:04X}", alt), 3),
        0xD5 => (String::from("PUSH DE"), 1),
        0xD6 => (format!("SUB ${:02X}", gf), 2),
        0xD8 => (String::from("RET C"), 1),
        0xD9 => (String::from("RETI"), 1),
        0xDA => (format!("JP C,${:04X}", alt), 3),
        0xE0 => (format!("LDH ($FF{:02X}),A", gf), 2),
        0xE1 => (String::from("POP HL"), 1),
        0xE2 => (String::from("LD ($FF00+C),A"), 1),
        0xE5 => (String::from("PUSH HL"), 1),
        0xE6 => (format!("AND ${:02X}", gf), 2),
        0xE9 => (String::from("JP (HL)"), 1),
        0xEA => (format!("LD (${:04X}),A", alt), 3),
        0xEE => (format!("XOR ${:02X}", gf), 2),
        0xF0 => (format!("LDH A,($FF{:02X})", gf), 2),
        0xF1 => (String::from("POP AF"), 1),
        0xF3 => (String::from("DI"), 1),
        0xF5 => (String::from("PUSH AF"), 1),
        0xF6 => (format!("OR ${:02X}", gf), 2),
        0xFA => (format!("LD A,(${:04X})", alt), 3),
        0xFB => (String::from("EI"), 1),
        0xFE => (format!("CP ${:02X}", gf), 2),
        0xFF => (String::from("RST $38"), 1),
        _ => (format!("DB ${:02X}", op), 1),
    }
}




pub fn ljg(
    an: Option<&GameBoyEmulator>,
    cx: u32, u: u32, aq: u32, ch: u32,
) {
    
    framebuffer::fill_rect(cx, u, aq, ch, 0xFF0A0F14);
    
    if aq < 60 || ch < 40 { return; }
    
    let (up, dno, left, right, eer, efu, ezx, fbo, dirs_raw, btns_raw) =
        if let Some(an) = an {
            (
                an.joypad_dirs & 0x04 == 0,
                an.joypad_dirs & 0x08 == 0,
                an.joypad_dirs & 0x02 == 0,
                an.joypad_dirs & 0x01 == 0,
                an.joypad_buttons & 0x01 == 0,
                an.joypad_buttons & 0x02 == 0,
                an.joypad_buttons & 0x04 == 0,
                an.joypad_buttons & 0x08 == 0,
                an.joypad_dirs,
                an.joypad_buttons,
            )
        } else {
            (false, false, false, false, false, false, false, false, 0xFF, 0xFF)
        };
    
    
    bo(cx + 6, u + 4, "GAME BOY INPUT", F_);
    
    
    if aq > 300 {
        bo(cx + aq - 200, u + 4, "WASD=Pad X=A Z=B C=Sel", 0xFF3A5A44);
    }
    
    
    let blv = cx + 40;
    let blw = u + 30;
    let fq: u32 = 26;
    let gap: u32 = 2;
    
    
    ekp(blv, blw - fq - gap, fq, fq, "W", up);
    
    ekp(blv, blw + fq + gap, fq, fq, "S", dno);
    
    ekp(blv - fq - gap, blw, fq, fq, "A", left);
    
    ekp(blv + fq + gap, blw, fq, fq, "D", right);
    
    framebuffer::fill_rect(blv, blw, fq, fq, 0xFF141E1A);
    
    
    let of: u32 = 30;
    let ees = cx + aq - 80;
    let eet = u + 30;
    let efw = cx + aq - 140;
    let efx = u + 48;
    
    htm(ees, eet, of, "A", eer);
    htm(efw, efx, of, "B", efu);
    
    
    bo(ees + of + 4, eet + 8, "(X)", F_);
    bo(efw - 28, efx + 8, "(Z)", F_);
    
    
    let arn = cx + aq / 2;
    let coc = u + ch - 36;
    htn(arn - 70, coc, 56, 20, "SELECT", ezx);
    htn(arn + 14, coc, 56, 20, "START", fbo);
    
    
    bo(arn - 70, coc + 22, "(C)", F_);
    bo(arn + 14, coc + 22, "(Enter)", F_);
    
    
    let btj = u + ch - 16;
    let ds = alloc::format!("DIRS:{:02X}", dirs_raw);
    let fjv = alloc::format!("BTNS:{:02X}", btns_raw);
    bo(cx + 6, btj, &ds, 0xFF3A5A44);
    bo(cx + 80, btj, &fjv, 0xFF3A5A44);
}


pub fn mdf(cx: u32, u: u32, aq: u32, ch: u32) -> [(u32, u32, u32, u32, u8); 8] {
    let blv = cx + 40;
    let blw = u + 30;
    let fq: u32 = 26;
    let gap: u32 = 2;
    
    let of: u32 = 30;
    let ees = cx + aq - 80;
    let eet = u + 30;
    let efw = cx + aq - 140;
    let efx = u + 48;
    
    let arn = cx + aq / 2;
    let coc = u + ch - 36;
    
    [
        (blv, blw - fq - gap, fq, fq, b'w'),           
        (blv, blw + fq + gap, fq, fq, b's'),           
        (blv - fq - gap, blw, fq, fq, b'a'),           
        (blv + fq + gap, blw, fq, fq, b'd'),           
        (ees, eet, of, of, b'x'),                    
        (efw, efx, of, of, b'z'),                    
        (arn - 70, coc, 56, 20, b'c'),                  
        (arn + 14, coc, 56, 20, b'\r'),                 
    ]
}

fn ekp(x: u32, y: u32, w: u32, h: u32, label: &str, pressed: bool) {
    let bg = if pressed { 0xFF00CC66 } else { 0xFF1A2E24 };
    framebuffer::fill_rect(x, y, w, h, bg);
    
    let bc = if pressed { 0xFF00FF88 } else { 0xFF2A4A38 };
    framebuffer::fill_rect(x, y, w, 1, bc);
    framebuffer::fill_rect(x, y + h - 1, w, 1, bc);
    framebuffer::fill_rect(x, y, 1, h, bc);
    framebuffer::fill_rect(x + w - 1, y, 1, h, bc);
    let crl = if pressed { 0xFF000000 } else { 0xFF00FF88 };
    let bu = x + (w / 2).saturating_sub(4);
    let ty = y + (h / 2).saturating_sub(6);
    bo(bu, ty, label, crl);
}

fn htm(x: u32, y: u32, fq: u32, label: &str, pressed: bool) {
    let bg = if pressed { 0xFF00CC66 } else { 0xFF1A2E24 };
    framebuffer::fill_rect(x, y, fq, fq, bg);
    
    framebuffer::fill_rect(x, y, 4, 4, 0xFF0A0F14);
    framebuffer::fill_rect(x + fq - 4, y, 4, 4, 0xFF0A0F14);
    framebuffer::fill_rect(x, y + fq - 4, 4, 4, 0xFF0A0F14);
    framebuffer::fill_rect(x + fq - 4, y + fq - 4, 4, 4, 0xFF0A0F14);
    
    let bc = if pressed { 0xFF00FF88 } else { 0xFF2A4A38 };
    framebuffer::fill_rect(x + 3, y, fq - 6, 1, bc);
    framebuffer::fill_rect(x + 3, y + fq - 1, fq - 6, 1, bc);
    framebuffer::fill_rect(x, y + 3, 1, fq - 6, bc);
    framebuffer::fill_rect(x + fq - 1, y + 3, 1, fq - 6, bc);
    let crl = if pressed { 0xFF000000 } else { 0xFF00FF88 };
    bo(x + fq / 2 - 4, y + fq / 2 - 6, label, crl);
}

fn htn(x: u32, y: u32, w: u32, h: u32, label: &str, pressed: bool) {
    let bg = if pressed { 0xFF00CC66 } else { 0xFF141E1A };
    framebuffer::fill_rect(x, y, w, h, bg);
    let bc = if pressed { 0xFF00FF88 } else { 0xFF2A4A38 };
    framebuffer::fill_rect(x + 2, y, w - 4, 1, bc);
    framebuffer::fill_rect(x + 2, y + h - 1, w - 4, 1, bc);
    framebuffer::fill_rect(x, y + 2, 1, h - 4, bc);
    framebuffer::fill_rect(x + w - 1, y + 2, 1, h - 4, bc);
    let crl = if pressed { 0xFF000000 } else { 0xFF80FFAA };
    let aok = label.len() as u32 * 8;
    bo(x + (w.saturating_sub(aok)) / 2, y + (h / 2).saturating_sub(6), label, crl);
}
