//! Game Boy Emulator — Orchestrator with desktop integration
//! Ties CPU, GPU, Timer, Cartridge together via BusAdapter pattern
#![allow(dead_code)]

use alloc::vec;
use alloc::vec::Vec;
pub mod cartridge;
pub mod gpu;
pub mod timer;
pub mod cpu;

use cpu::GbBus;

/// M-cycles per frame (~59.73 fps, 17556 M-cycles @ 4.194304 MHz)
const FRAME_MCYCLES: u32 = 17556;

pub struct GameBoyEmulator {
    pub cpu: cpu::Cpu,
    pub gpu: gpu::Gpu,
    pub timer: timer::Timer,
    pub cart: cartridge::Cartridge,

    // Memory — CGB: 8 banks of 4KB WRAM ($C000-$CFFF = bank 0, $D000-$DFFF = switchable)
    pub wram: Vec<u8>,       // 32KB for CGB, 8KB for DMG
    pub hram: [u8; 127],    // $FF80-$FFFE

    // I/O
    pub ie_reg: u8,          // $FFFF Interrupt Enable
    pub if_reg: u8,          // $FF0F Interrupt Flag
    pub joypad_reg: u8,      // $FF00 Joypad select
    pub joypad_buttons: u8,  // A, B, Select, Start (active-low bits)
    pub joypad_dirs: u8,     // Right, Left, Up, Down (active-low bits)
    pub serial_data: u8,     // $FF01
    pub serial_ctrl: u8,     // $FF02

    pub rom_loaded: bool,
    pub key_state: u32,

    // CGB registers
    pub cgb_mode: bool,
    pub wram_bank: u8,       // $FF70: WRAM bank (1-7, 0 maps to 1)
    pub key1: u8,            // $FF4D: Speed switch
    pub hdma1: u8,           // $FF51: HDMA Source High
    pub hdma2: u8,           // $FF52: HDMA Source Low
    pub hdma3: u8,           // $FF53: HDMA Dest High
    pub hdma4: u8,           // $FF54: HDMA Dest Low
    pub hdma5: u8,           // $FF55: HDMA Length/Mode/Start
    pub hdma_active: bool,
}

impl GameBoyEmulator {
    pub fn new() -> Self {
        Self {
            cpu: cpu::Cpu::new(),
            gpu: gpu::Gpu::new(),
            timer: timer::Timer::new(),
            cart: cartridge::Cartridge::empty(),
            wram: vec![0u8; 32768], // 32KB for CGB support
            hram: [0; 127],
            ie_reg: 0,
            if_reg: 0,
            joypad_reg: 0xCF,
            joypad_buttons: 0x0F,
            joypad_dirs: 0x0F,
            serial_data: 0,
            serial_ctrl: 0,
            rom_loaded: false,
            key_state: 0,
            cgb_mode: false,
            wram_bank: 1,
            key1: 0,
            hdma1: 0xFF,
            hdma2: 0xFF,
            hdma3: 0xFF,
            hdma4: 0xFF,
            hdma5: 0xFF,
            hdma_active: false,
        }
    }

    pub fn load_rom(&mut self, data: &[u8]) -> bool {
        if let Some(cart) = cartridge::Cartridge::from_rom(data) {
            // Detect CGB mode from ROM header
            let is_cgb = cart.cgb_flag == 0x80 || cart.cgb_flag == 0xC0;
            self.cgb_mode = is_cgb;
            self.cart = cart;
            self.cpu = cpu::Cpu::new();
            if is_cgb {
                // CGB boot state: A=0x11 signals CGB hardware
                self.cpu.a = 0x11;
                self.cpu.f = 0x80; // Z flag set
                self.cpu.b = 0x00;
                self.cpu.c = 0x00;
                self.cpu.d = 0xFF;
                self.cpu.e = 0x56;
                self.cpu.h = 0x00;
                self.cpu.l = 0x0D;
                crate::serial_println!("[GB] CGB mode enabled (A=0x11)");
            }
            self.gpu = gpu::Gpu::new();
            self.gpu.cgb_mode = is_cgb;
            if is_cgb {
                // Initialize CGB BG palette 0 to a reasonable default (white gradient)
                // Palette 0 color 0 = white
                self.gpu.bg_palette[0] = 0xFF; self.gpu.bg_palette[1] = 0x7F;
                // Palette 0 color 1 = light gray
                self.gpu.bg_palette[2] = 0xB5; self.gpu.bg_palette[3] = 0x56;
                // Palette 0 color 2 = dark gray
                self.gpu.bg_palette[4] = 0x4A; self.gpu.bg_palette[5] = 0x29;
                // Palette 0 color 3 = black
                self.gpu.bg_palette[6] = 0x00; self.gpu.bg_palette[7] = 0x00;
                // OBJ palettes similar defaults
                for i in 0..8 {
                    self.gpu.obj_palette[i] = self.gpu.bg_palette[i];
                }
            }
            self.timer = timer::Timer::new();
            for b in self.wram.iter_mut() { *b = 0; }
            self.hram = [0; 127];
            self.ie_reg = 0;
            self.if_reg = 0;
            self.wram_bank = 1;
            self.key1 = 0;
            self.hdma_active = false;
            self.rom_loaded = true;
            crate::serial_println!("[GB] ROM loaded successfully (CGB={})", is_cgb);
            true
        } else {
            crate::serial_println!("[GB] Failed to load ROM");
            false
        }
    }

    fn make_bus(&mut self) -> BusAdapter<'_> {
        BusAdapter {
            wram: &mut self.wram,
            hram: &mut self.hram,
            gpu: &mut self.gpu,
            timer: &mut self.timer,
            cart: &mut self.cart,
            ie_reg: &mut self.ie_reg,
            if_reg: &mut self.if_reg,
            joypad_reg: &mut self.joypad_reg,
            joypad_buttons: &self.joypad_buttons,
            joypad_dirs: &self.joypad_dirs,
            serial_data: &mut self.serial_data,
            serial_ctrl: &mut self.serial_ctrl,
            cgb_mode: self.cgb_mode,
            wram_bank: &mut self.wram_bank,
            key1: &mut self.key1,
            hdma1: &mut self.hdma1,
            hdma2: &mut self.hdma2,
            hdma3: &mut self.hdma3,
            hdma4: &mut self.hdma4,
            hdma5: &mut self.hdma5,
            hdma_active: &mut self.hdma_active,
        }
    }

    // Key handling — active-low (0 = pressed)
    // Joypad buttons: bit3=Start, bit2=Select, bit1=B, bit0=A
    // Joypad dirs:    bit3=Down,  bit2=Up,     bit1=Left, bit0=Right
    pub fn handle_key(&mut self, key: u8) {
        match key {
            b'd' | b'D' | 0xF3 => self.joypad_dirs &= !0x01, // Right
            b'a' | b'A' | 0xF2 => self.joypad_dirs &= !0x02, // Left
            b'w' | b'W' | 0xF0 => self.joypad_dirs &= !0x04, // Up
            b's' | b'S' | 0xF1 => self.joypad_dirs &= !0x08, // Down
            b'x' | b'X' | b' ' => self.joypad_buttons &= !0x01, // A button
            b'z' | b'Z'        => self.joypad_buttons &= !0x02, // B button
            b'\r' | 10         => self.joypad_buttons &= !0x08, // Start (Enter)
            b'c' | b'C'        => self.joypad_buttons &= !0x04, // Select
            _ => {}
        }
        self.if_reg |= 0x10; // Joypad interrupt
    }

    pub fn handle_key_release(&mut self, key: u8) {
        match key {
            b'd' | b'D' | 0xF3 => self.joypad_dirs |= 0x01,
            b'a' | b'A' | 0xF2 => self.joypad_dirs |= 0x02,
            b'w' | b'W' | 0xF0 => self.joypad_dirs |= 0x04,
            b's' | b'S' | 0xF1 => self.joypad_dirs |= 0x08,
            b'x' | b'X' | b' ' => self.joypad_buttons |= 0x01,
            b'z' | b'Z'        => self.joypad_buttons |= 0x02,
            b'\r' | 10         => self.joypad_buttons |= 0x08,
            b'c' | b'C'        => self.joypad_buttons |= 0x04,
            _ => {}
        }
    }

    /// Run one frame (~17556 M-cycles)
    pub fn tick(&mut self) {
        if !self.rom_loaded { return; }

        self.gpu.frame_ready = false;
        let mut frame_cycles: u32 = 0;
        let mut safety_counter: u32 = 0;
        const MAX_INSTRUCTIONS: u32 = 200_000; // Safety limit

        while frame_cycles < FRAME_MCYCLES {
            safety_counter += 1;
            if safety_counter > MAX_INSTRUCTIONS {
                // Prevent infinite loops from crashing the OS
                break;
            }

            let m = {
                let mut bus = BusAdapter {
                    wram: &mut self.wram,
                    hram: &mut self.hram,
                    gpu: &mut self.gpu,
                    timer: &mut self.timer,
                    cart: &mut self.cart,
                    ie_reg: &mut self.ie_reg,
                    if_reg: &mut self.if_reg,
                    joypad_reg: &mut self.joypad_reg,
                    joypad_buttons: &self.joypad_buttons,
                    joypad_dirs: &self.joypad_dirs,
                    serial_data: &mut self.serial_data,
                    serial_ctrl: &mut self.serial_ctrl,
                    cgb_mode: self.cgb_mode,
                    wram_bank: &mut self.wram_bank,
                    key1: &mut self.key1,
                    hdma1: &mut self.hdma1,
                    hdma2: &mut self.hdma2,
                    hdma3: &mut self.hdma3,
                    hdma4: &mut self.hdma4,
                    hdma5: &mut self.hdma5,
                    hdma_active: &mut self.hdma_active,
                };
                self.cpu.step(&mut bus)
            };

            // Step GPU and Timer with M-cycles
            self.gpu.step(m);
            self.timer.step(m);

            // Collect interrupt requests
            if self.gpu.vblank_irq {
                self.if_reg |= 0x01;
                self.gpu.vblank_irq = false;
            }
            if self.gpu.stat_irq {
                self.if_reg |= 0x02;
                self.gpu.stat_irq = false;
            }
            if self.timer.interrupt {
                self.if_reg |= 0x04;
                self.timer.interrupt = false;
            }

            frame_cycles += m;
        }
    }

    /// Render to output buffer (upscale 160×144 → output dimensions)
    pub fn render(&self, out: &mut [u32], out_w: usize, out_h: usize) {
        if !self.rom_loaded {
            self.render_no_rom(out, out_w, out_h);
            return;
        }

        let src_w = gpu::SCREEN_W;
        let src_h = gpu::SCREEN_H;

        for y in 0..out_h {
            let sy = y * src_h / out_h;
            for x in 0..out_w {
                let sx = x * src_w / out_w;
                let si = sy * src_w + sx;
                out[y * out_w + x] = if si < self.gpu.framebuffer.len() {
                    self.gpu.framebuffer[si]
                } else {
                    0xFF081820
                };
            }
        }
    }

    fn render_no_rom(&self, out: &mut [u32], w: usize, h: usize) {
        let bg = 0xFF081820u32;  // Dark green-black
        let fg = 0xFFE0F8D0u32;  // Light green
        let mg = 0xFF346856u32;  // Medium green

        for p in out.iter_mut() { *p = bg; }

        // Title
        draw_text(out, w, h, "GAME BOY", w / 2 - 32, h / 6, fg, 2);
        draw_text(out, w, h, "EMULATOR", w / 2 - 32, h / 6 + 20, mg, 2);

        // Insert ROM message
        draw_text(out, w, h, "INSERT ROM", w / 2 - 40, h / 2 - 10, fg, 2);

        // Draw Game Boy outline
        let cx = w / 2;
        let by = h * 5 / 8;
        let bw = 60usize;
        let bh = 80usize;
        for x in (cx - bw/2)..=(cx + bw/2) {
            if x < w {
                if by < h { out[by * w + x] = mg; }
                if by + bh < h { out[(by + bh) * w + x] = mg; }
            }
        }
        for y in by..=(by + bh) {
            if y < h {
                if cx - bw/2 < w { out[y * w + (cx - bw/2)] = mg; }
                if cx + bw/2 < w { out[y * w + (cx + bw/2)] = mg; }
            }
        }
        // Screen area
        let sw = 40usize;
        let sh = 36usize;
        let sx = cx - sw / 2;
        let sy = by + 8;
        for y in sy..(sy + sh).min(h) {
            for x in sx..(sx + sw).min(w) {
                out[y * w + x] = 0xFF88C070;
            }
        }

        // Controls
        draw_text(out, w, h, "WASD:DPAD", w / 2 - 36, h - 50, mg, 1);
        draw_text(out, w, h, "X:A Z:B ENTER:START", w / 2 - 72, h - 38, mg, 1);
    }
}

// BusAdapter: borrows components separately to satisfy borrow checker
struct BusAdapter<'a> {
    wram: &'a mut Vec<u8>,
    hram: &'a mut [u8; 127],
    gpu: &'a mut gpu::Gpu,
    timer: &'a mut timer::Timer,
    cart: &'a mut cartridge::Cartridge,
    ie_reg: &'a mut u8,
    if_reg: &'a mut u8,
    joypad_reg: &'a mut u8,
    joypad_buttons: &'a u8,
    joypad_dirs: &'a u8,
    serial_data: &'a mut u8,
    serial_ctrl: &'a mut u8,
    // CGB
    cgb_mode: bool,
    wram_bank: &'a mut u8,
    key1: &'a mut u8,
    hdma1: &'a mut u8,
    hdma2: &'a mut u8,
    hdma3: &'a mut u8,
    hdma4: &'a mut u8,
    hdma5: &'a mut u8,
    hdma_active: &'a mut bool,
}

impl GbBus for BusAdapter<'_> {
    fn read(&mut self, addr: u16) -> u8 {
        match addr {
            // ROM
            0x0000..=0x7FFF => self.cart.read(addr),
            // VRAM
            0x8000..=0x9FFF => self.gpu.read_vram(addr),
            // External RAM
            0xA000..=0xBFFF => self.cart.read(addr),
            // WRAM bank 0 ($C000-$CFFF)
            0xC000..=0xCFFF => self.wram[(addr as usize - 0xC000)],
            // WRAM bank 1-7 ($D000-$DFFF) — CGB switchable
            0xD000..=0xDFFF => {
                let bank = if self.cgb_mode { (*self.wram_bank).max(1) as usize } else { 1 };
                let offset = bank * 0x1000 + (addr as usize - 0xD000);
                if offset < self.wram.len() { self.wram[offset] } else { 0xFF }
            },
            // Echo RAM
            0xE000..=0xEFFF => self.wram[(addr as usize - 0xE000)],
            0xF000..=0xFDFF => {
                let bank = if self.cgb_mode { (*self.wram_bank).max(1) as usize } else { 1 };
                let offset = bank * 0x1000 + (addr as usize - 0xF000);
                if offset < self.wram.len() { self.wram[offset] } else { 0xFF }
            },
            // OAM
            0xFE00..=0xFE9F => self.gpu.read_oam(addr),
            // Not usable
            0xFEA0..=0xFEFF => 0xFF,
            // I/O registers
            0xFF00 => {
                let mut val = *self.joypad_reg & 0x30;
                if val & 0x10 == 0 { val |= *self.joypad_dirs; }
                if val & 0x20 == 0 { val |= *self.joypad_buttons; }
                val | 0xC0
            }
            0xFF01 => *self.serial_data,
            0xFF02 => *self.serial_ctrl,
            0xFF04 => self.timer.read_div(),
            0xFF05 => self.timer.tima,
            0xFF06 => self.timer.tma,
            0xFF07 => self.timer.tac,
            0xFF0F => *self.if_reg,
            // Audio (not emulated)
            0xFF10..=0xFF3F => 0xFF,
            // LCD
            0xFF40 => self.gpu.lcdc,
            0xFF41 => self.gpu.read_stat(),
            0xFF42 => self.gpu.scy,
            0xFF43 => self.gpu.scx,
            0xFF44 => if self.gpu.lcdc & 0x80 != 0 { self.gpu.ly } else { 0 },
            0xFF45 => self.gpu.lyc,
            0xFF46 => 0, // DMA — write-only
            0xFF47 => self.gpu.bgp,
            0xFF48 => self.gpu.obp0,
            0xFF49 => self.gpu.obp1,
            0xFF4A => self.gpu.wy,
            0xFF4B => self.gpu.wx,
            // CGB registers
            0xFF4D => *self.key1,                      // KEY1 speed switch
            0xFF4F => self.gpu.vram_bank | 0xFE,       // VBK VRAM bank (only bit 0)
            0xFF51 => *self.hdma1,
            0xFF52 => *self.hdma2,
            0xFF53 => *self.hdma3,
            0xFF54 => *self.hdma4,
            0xFF55 => *self.hdma5,
            0xFF68 => self.gpu.bcps,                   // BCPS
            0xFF69 => self.gpu.read_bcpd(),            // BCPD
            0xFF6A => self.gpu.ocps,                   // OCPS
            0xFF6B => self.gpu.read_ocpd(),            // OCPD
            0xFF70 => *self.wram_bank,                 // SVBK WRAM bank
            // HRAM
            0xFF80..=0xFFFE => self.hram[(addr - 0xFF80) as usize],
            // IE
            0xFFFF => *self.ie_reg,
            _ => 0xFF,
        }
    }

    fn write(&mut self, addr: u16, val: u8) {
        match addr {
            // ROM area — cartridge handles mapper writes
            0x0000..=0x7FFF => self.cart.write(addr, val),
            // VRAM
            0x8000..=0x9FFF => self.gpu.write_vram(addr, val),
            // External RAM
            0xA000..=0xBFFF => self.cart.write(addr, val),
            // WRAM bank 0
            0xC000..=0xCFFF => self.wram[(addr as usize - 0xC000)] = val,
            // WRAM bank 1-7 (CGB switchable)
            0xD000..=0xDFFF => {
                let bank = if self.cgb_mode { (*self.wram_bank).max(1) as usize } else { 1 };
                let offset = bank * 0x1000 + (addr as usize - 0xD000);
                if offset < self.wram.len() { self.wram[offset] = val; }
            },
            // Echo RAM
            0xE000..=0xEFFF => self.wram[(addr as usize - 0xE000)] = val,
            0xF000..=0xFDFF => {
                let bank = if self.cgb_mode { (*self.wram_bank).max(1) as usize } else { 1 };
                let offset = bank * 0x1000 + (addr as usize - 0xF000);
                if offset < self.wram.len() { self.wram[offset] = val; }
            },
            // OAM
            0xFE00..=0xFE9F => self.gpu.write_oam(addr, val),
            // Not usable
            0xFEA0..=0xFEFF => {}
            // I/O
            0xFF00 => *self.joypad_reg = val & 0x30,
            0xFF01 => *self.serial_data = val,
            0xFF02 => *self.serial_ctrl = val,
            0xFF04 => self.timer.write_div(),
            0xFF05 => self.timer.tima = val,
            0xFF06 => self.timer.tma = val,
            0xFF07 => self.timer.tac = val,
            0xFF0F => *self.if_reg = val,
            // Audio (not emulated)
            0xFF10..=0xFF3F => {}
            // LCD
            0xFF40 => {
                let old = self.gpu.lcdc;
                self.gpu.lcdc = val;
                // LCD just turned on — reset GPU state
                if val & 0x80 != 0 && old & 0x80 == 0 {
                    self.gpu.ly = 0;
                    self.gpu.cycles = 0;
                    self.gpu.mode = 2;
                    self.gpu.window_line = 0;
                }
            }
            0xFF41 => self.gpu.stat = (self.gpu.stat & 0x07) | (val & 0xF8),
            0xFF42 => self.gpu.scy = val,
            0xFF43 => self.gpu.scx = val,
            0xFF44 => {} // LY is read-only
            0xFF45 => self.gpu.lyc = val,
            0xFF46 => {
                // OAM DMA transfer — copy 160 bytes from val*$100
                let base = (val as u16) << 8;
                for i in 0..160u16 {
                    let byte = match base + i {
                        a @ 0x0000..=0x7FFF => self.cart.read(a),
                        a @ 0x8000..=0x9FFF => self.gpu.read_vram(a),
                        a @ 0xA000..=0xBFFF => self.cart.read(a),
                        a @ 0xC000..=0xCFFF => self.wram[(a as usize - 0xC000)],
                        a @ 0xD000..=0xDFFF => {
                            let bank = if self.cgb_mode { (*self.wram_bank).max(1) as usize } else { 1 };
                            let offset = bank * 0x1000 + (a as usize - 0xD000);
                            if offset < self.wram.len() { self.wram[offset] } else { 0 }
                        },
                        _ => 0,
                    };
                    self.gpu.write_oam(0xFE00 + i, byte);
                }
            }
            0xFF47 => self.gpu.bgp = val,
            0xFF48 => self.gpu.obp0 = val,
            0xFF49 => self.gpu.obp1 = val,
            0xFF4A => self.gpu.wy = val,
            0xFF4B => self.gpu.wx = val,
            // CGB registers
            0xFF4D => *self.key1 = (*self.key1 & 0x80) | (val & 0x01), // KEY1: only bit 0 writable
            0xFF4F => self.gpu.vram_bank = val & 0x01,                  // VBK
            0xFF51 => *self.hdma1 = val,
            0xFF52 => *self.hdma2 = val & 0xF0,
            0xFF53 => *self.hdma3 = val & 0x1F,
            0xFF54 => *self.hdma4 = val & 0xF0,
            0xFF55 => {
                // HDMA5: Start HDMA transfer (General-Purpose or HBlank)
                if self.cgb_mode {
                    let src = ((*self.hdma1 as u16) << 8) | (*self.hdma2 as u16);
                    let dst = 0x8000 | (((*self.hdma3 as u16) << 8) | (*self.hdma4 as u16));
                    let len = ((val as u16 & 0x7F) + 1) * 16;
                    
                    if val & 0x80 == 0 {
                        // General-purpose DMA: transfer all at once
                        for i in 0..len {
                            let byte = match src.wrapping_add(i) {
                                a @ 0x0000..=0x7FFF => self.cart.read(a),
                                a @ 0x8000..=0x9FFF => self.gpu.read_vram(a),
                                a @ 0xA000..=0xBFFF => self.cart.read(a),
                                a @ 0xC000..=0xCFFF => self.wram[(a as usize - 0xC000)],
                                a @ 0xD000..=0xDFFF => {
                                    let bank = (*self.wram_bank).max(1) as usize;
                                    let offset = bank * 0x1000 + (a as usize - 0xD000);
                                    if offset < self.wram.len() { self.wram[offset] } else { 0 }
                                },
                                _ => 0xFF,
                            };
                            self.gpu.write_vram(dst.wrapping_add(i), byte);
                        }
                        *self.hdma5 = 0xFF; // Transfer complete
                    } else {
                        // HBlank DMA: TODO for now do immediate transfer too
                        for i in 0..len {
                            let byte = match src.wrapping_add(i) {
                                a @ 0x0000..=0x7FFF => self.cart.read(a),
                                a @ 0xA000..=0xBFFF => self.cart.read(a),
                                a @ 0xC000..=0xCFFF => self.wram[(a as usize - 0xC000)],
                                a @ 0xD000..=0xDFFF => {
                                    let bank = (*self.wram_bank).max(1) as usize;
                                    let offset = bank * 0x1000 + (a as usize - 0xD000);
                                    if offset < self.wram.len() { self.wram[offset] } else { 0 }
                                },
                                _ => 0xFF,
                            };
                            self.gpu.write_vram(dst.wrapping_add(i), byte);
                        }
                        *self.hdma5 = 0xFF;
                    }
                }
            }
            0xFF68 => self.gpu.bcps = val,               // BCPS
            0xFF69 => self.gpu.write_bcpd(val),           // BCPD
            0xFF6A => self.gpu.ocps = val,               // OCPS
            0xFF6B => self.gpu.write_ocpd(val),           // OCPD
            0xFF70 => {
                // SVBK: WRAM bank select (1-7, writing 0 selects bank 1)
                *self.wram_bank = val & 0x07;
                if *self.wram_bank == 0 { *self.wram_bank = 1; }
            }
            // HRAM
            0xFF80..=0xFFFE => self.hram[(addr - 0xFF80) as usize] = val,
            // IE
            0xFFFF => *self.ie_reg = val,
            _ => {}
        }
    }
}

// ======== Simple text rendering (3×5 bitmap font) ========
fn draw_text(out: &mut [u32], w: usize, h: usize, text: &str, x: usize, y: usize, color: u32, scale: usize) {
    let mut cx = x;
    for ch in text.bytes() {
        let glyph = get_glyph(ch);
        for row in 0..5usize {
            for col in 0..3usize {
                if glyph[row] & (1 << (2 - col)) != 0 {
                    for sy in 0..scale {
                        for sx in 0..scale {
                            let px = cx + col * scale + sx;
                            let py = y + row * scale + sy;
                            if px < w && py < h { out[py * w + px] = color; }
                        }
                    }
                }
            }
        }
        cx += (3 + 1) * scale;
    }
}

fn get_glyph(ch: u8) -> [u8; 5] {
    match ch {
        b'A' => [0b111, 0b101, 0b111, 0b101, 0b101],
        b'B' => [0b110, 0b101, 0b110, 0b101, 0b110],
        b'C' => [0b111, 0b100, 0b100, 0b100, 0b111],
        b'D' => [0b110, 0b101, 0b101, 0b101, 0b110],
        b'E' => [0b111, 0b100, 0b110, 0b100, 0b111],
        b'F' => [0b111, 0b100, 0b110, 0b100, 0b100],
        b'G' => [0b111, 0b100, 0b101, 0b101, 0b111],
        b'H' => [0b101, 0b101, 0b111, 0b101, 0b101],
        b'I' => [0b111, 0b010, 0b010, 0b010, 0b111],
        b'J' => [0b001, 0b001, 0b001, 0b101, 0b010],
        b'K' => [0b101, 0b101, 0b110, 0b101, 0b101],
        b'L' => [0b100, 0b100, 0b100, 0b100, 0b111],
        b'M' => [0b101, 0b111, 0b111, 0b101, 0b101],
        b'N' => [0b101, 0b111, 0b111, 0b111, 0b101],
        b'O' => [0b111, 0b101, 0b101, 0b101, 0b111],
        b'P' => [0b111, 0b101, 0b111, 0b100, 0b100],
        b'Q' => [0b111, 0b101, 0b101, 0b111, 0b001],
        b'R' => [0b111, 0b101, 0b111, 0b110, 0b101],
        b'S' => [0b111, 0b100, 0b111, 0b001, 0b111],
        b'T' => [0b111, 0b010, 0b010, 0b010, 0b010],
        b'U' => [0b101, 0b101, 0b101, 0b101, 0b111],
        b'V' => [0b101, 0b101, 0b101, 0b101, 0b010],
        b'W' => [0b101, 0b101, 0b111, 0b111, 0b101],
        b'X' => [0b101, 0b101, 0b010, 0b101, 0b101],
        b'Y' => [0b101, 0b101, 0b010, 0b010, 0b010],
        b'Z' => [0b111, 0b001, 0b010, 0b100, 0b111],
        b'0' => [0b111, 0b101, 0b101, 0b101, 0b111],
        b'1' => [0b010, 0b110, 0b010, 0b010, 0b111],
        b'2' => [0b111, 0b001, 0b111, 0b100, 0b111],
        b'3' => [0b111, 0b001, 0b111, 0b001, 0b111],
        b'4' => [0b101, 0b101, 0b111, 0b001, 0b001],
        b'5' => [0b111, 0b100, 0b111, 0b001, 0b111],
        b'6' => [0b111, 0b100, 0b111, 0b101, 0b111],
        b'7' => [0b111, 0b001, 0b001, 0b001, 0b001],
        b'8' => [0b111, 0b101, 0b111, 0b101, 0b111],
        b'9' => [0b111, 0b101, 0b111, 0b001, 0b111],
        b':' => [0b000, 0b010, 0b000, 0b010, 0b000],
        b' ' => [0b000, 0b000, 0b000, 0b000, 0b000],
        _ => [0b111, 0b111, 0b111, 0b111, 0b111],
    }
}
