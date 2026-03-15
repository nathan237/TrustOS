//! NES PPU (Picture Processing Unit) — 2C02 emulation
//! Scanline-accurate rendering: background tiles, sprites, scrolling, palettes
#![allow(dead_code)]

use alloc::vec;
use alloc::vec::Vec;
use super::cartridge::Cartridge;

// NES system palette — 64 RGB colors (2C02)
pub const NES_PALETTE: [u32; 64] = [
    0xFF666666, 0xFF002A88, 0xFF1412A7, 0xFF3B00A4, 0xFF5C007E, 0xFF6E0040, 0xFF6C0600, 0xFF561D00,
    0xFF333500, 0xFF0B4800, 0xFF005200, 0xFF004F08, 0xFF00404D, 0xFF000000, 0xFF000000, 0xFF000000,
    0xFFADADAD, 0xFF155FD9, 0xFF4240FF, 0xFF7527FE, 0xFFA01ACC, 0xFFB71E7B, 0xFFB53120, 0xFF994E00,
    0xFF6B6D00, 0xFF388700, 0xFF0C9300, 0xFF008F32, 0xFF007C8D, 0xFF000000, 0xFF000000, 0xFF000000,
    0xFFFFFEFF, 0xFF64B0FF, 0xFF9290FF, 0xFFC676FF, 0xFFF36AFF, 0xFFFE6ECC, 0xFFFE8170, 0xFFEA9E22,
    0xFFBCBE00, 0xFF88D800, 0xFF5CE430, 0xFF45E082, 0xFF48CDDE, 0xFF4F4F4F, 0xFF000000, 0xFF000000,
    0xFFFFFEFF, 0xFFC0DFFF, 0xFFD3D2FF, 0xFFE8C8FF, 0xFFFBC2FF, 0xFFFEC4EA, 0xFFFECCC5, 0xFFF7D8A5,
    0xFFE4E594, 0xFFCFEF96, 0xFFBDF4AB, 0xFFB3F3CC, 0xFFB5EBF2, 0xFFB8B8B8, 0xFF000000, 0xFF000000,
];

pub struct Ppu {
    // Registers
    pub ctrl: u8,       // $2000 PPUCTRL
    pub mask: u8,       // $2001 PPUMASK
    pub status: u8,     // $2002 PPUSTATUS
    pub oam_addr: u8,   // $2003 OAMADDR

    // Internal registers
    pub v: u16,         // Current VRAM address (15-bit)
    pub t: u16,         // Temporary VRAM address (15-bit)
    pub fine_x: u8,     // Fine X scroll (3-bit)
    pub w: bool,        // Write toggle for $2005/$2006
    pub data_buf: u8,   // PPUDATA read buffer

    // Memory
    pub oam: [u8; 256],        // Object Attribute Memory (64 sprites × 4 bytes)
    pub vram: [u8; 2048],      // 2KB nametable RAM
    pub palette: [u8; 32],     // Palette RAM

    // Rendering state
    pub scanline: i32,
    pub dot: u32,
    pub frame_count: u64,
    pub nmi_triggered: bool,
    pub sprite0_hit_possible: bool,

    // Secondary OAM for current scanline
    sprite_indices: [u8; 8],
    sprite_count: u8,

    // Frame buffer (256×240)
    pub framebuffer: Vec<u32>,
}

impl Ppu {
    pub fn new() -> Self {
        Self {
            ctrl: 0, mask: 0, status: 0, oam_addr: 0,
            v: 0, t: 0, fine_x: 0, w: false, data_buf: 0,
            oam: [0; 256],
            vram: [0; 2048],
            palette: [0; 32],
            scanline: -1,
            dot: 0,
            frame_count: 0,
            nmi_triggered: false,
            sprite0_hit_possible: false,
            sprite_indices: [0xFF; 8],
            sprite_count: 0,
            framebuffer: vec![0u32; 256 * 240],
        }
    }

    // ======================== Register Access ========================

    pub fn read_register(&mut self, addr: u16, cart: &Cartridge) -> u8 {
        match addr & 7 {
            2 => { // PPUSTATUS
                let val = (self.status & 0xE0) | (self.data_buf & 0x1F);
                self.status &= !0x80; // Clear VBlank
                self.w = false;
                val
            }
            4 => { // OAMDATA
                self.oam[self.oam_addr as usize]
            }
            7 => { // PPUDATA
                let addr = self.v & 0x3FFF;
                let val = if addr >= 0x3F00 {
                    self.palette_read(addr)
                } else {
                    let buffered = self.data_buf;
                    self.data_buf = self.ppu_read(addr, cart);
                    buffered
                };
                self.v = self.v.wrapping_add(if self.ctrl & 0x04 != 0 { 32 } else { 1 });
                val
            }
            _ => 0,
        }
    }

    pub fn write_register(&mut self, addr: u16, val: u8, cart: &mut Cartridge) {
        match addr & 7 {
            0 => { // PPUCTRL
                self.ctrl = val;
                self.t = (self.t & 0xF3FF) | (((val as u16) & 3) << 10);
            }
            1 => self.mask = val,
            3 => self.oam_addr = val,
            4 => { // OAMDATA
                self.oam[self.oam_addr as usize] = val;
                self.oam_addr = self.oam_addr.wrapping_add(1);
            }
            5 => { // PPUSCROLL
                if !self.w {
                    self.t = (self.t & 0xFFE0) | ((val as u16) >> 3);
                    self.fine_x = val & 7;
                } else {
                    self.t = (self.t & 0x8C1F)
                        | (((val as u16) & 0xF8) << 2)
                        | (((val as u16) & 7) << 12);
                }
                self.w = !self.w;
            }
            6 => { // PPUADDR
                if !self.w {
                    self.t = (self.t & 0x00FF) | (((val as u16) & 0x3F) << 8);
                } else {
                    self.t = (self.t & 0xFF00) | (val as u16);
                    self.v = self.t;
                }
                self.w = !self.w;
            }
            7 => { // PPUDATA
                let a = self.v & 0x3FFF;
                self.ppu_write(a, val, cart);
                self.v = self.v.wrapping_add(if self.ctrl & 0x04 != 0 { 32 } else { 1 });
            }
            _ => {}
        }
    }

    // ======================== PPU Memory Access ========================

    fn ppu_read(&self, addr: u16, cart: &Cartridge) -> u8 {
        match addr {
            0x0000..=0x1FFF => cart.ppu_read(addr),
            0x2000..=0x3EFF => {
                let nt_addr = cart.mirror_nametable(addr - 0x2000);
                self.vram[nt_addr as usize]
            }
            0x3F00..=0x3FFF => self.palette_read(addr),
            _ => 0,
        }
    }

    fn ppu_write(&mut self, addr: u16, val: u8, cart: &mut Cartridge) {
        match addr {
            0x0000..=0x1FFF => cart.ppu_write(addr, val),
            0x2000..=0x3EFF => {
                let nt_addr = cart.mirror_nametable(addr - 0x2000);
                self.vram[nt_addr as usize] = val;
            }
            0x3F00..=0x3FFF => {
                let idx = (addr & 0x1F) as usize;
                self.palette[idx] = val & 0x3F;
                // Mirror background color
                if idx & 3 == 0 {
                    self.palette[idx ^ 0x10] = val & 0x3F;
                }
            }
            _ => {}
        }
    }

    fn palette_read(&self, addr: u16) -> u8 {
        let mut idx = (addr & 0x1F) as usize;
        if idx >= 16 && idx & 3 == 0 { idx -= 16; }
        self.palette[idx] & 0x3F
    }

    // ======================== Scanline Rendering ========================

    /// Advance PPU by one scanline. Returns true if NMI should fire.
    pub fn step_scanline(&mut self, cart: &Cartridge) -> bool {
        let mut trigger_nmi = false;
        let rendering = self.mask & 0x18 != 0;

        match self.scanline {
            0..=239 => {
                // Visible scanline — render
                if rendering {
                    self.evaluate_sprites(cart);
                    self.render_scanline(cart);
                }
            }
            241 => {
                // VBlank start
                self.status |= 0x80;
                if self.ctrl & 0x80 != 0 {
                    trigger_nmi = true;
                }
            }
            261 => {
                // Pre-render scanline
                self.status &= !0xE0; // Clear VBlank, sprite 0, overflow
                if rendering {
                    // Copy vertical bits from t to v
                    self.v = (self.v & 0x041F) | (self.t & 0x7BE0);
                }
            }
            _ => {}
        }

        self.scanline += 1;
        if self.scanline > 261 {
            self.scanline = 0;
            self.frame_count += 1;
        }

        trigger_nmi
    }

    fn render_scanline(&mut self, cart: &Cartridge) {
        let y = self.scanline as usize;
        if y >= 240 { return; }

        let bg_enabled = self.mask & 0x08 != 0;
        let spr_enabled = self.mask & 0x10 != 0;
        let bg_left = self.mask & 0x02 != 0;
        let spr_left = self.mask & 0x04 != 0;

        let bg_pattern = if self.ctrl & 0x10 != 0 { 0x1000u16 } else { 0u16 };
        let spr_pattern = if self.ctrl & 0x08 != 0 { 0x1000u16 } else { 0u16 };
        let tall_sprites = self.ctrl & 0x20 != 0;
        let spr_h = if tall_sprites { 16 } else { 8 };

        // Scroll values from v register
        let coarse_x = self.v & 0x1F;
        let coarse_y = (self.v >> 5) & 0x1F;
        let fine_y = (self.v >> 12) & 7;
        let nt_select = (self.v >> 10) & 3;

        for x in 0..256usize {
            let screen_x = x;
            let pixel_x = x as u16 + self.fine_x as u16;
            let tile_x = (coarse_x as u16 + pixel_x / 8) as u16;
            let fine_x_pixel = (pixel_x % 8) as u8;

            // Background pixel
            let (bg_color, bg_palette) = if bg_enabled && (bg_left || x >= 8) {
                let actual_tile_x = tile_x & 0x1F;
                let nt_bit = if tile_x >= 32 { 1u16 } else { 0 };
                let nt = nt_select ^ nt_bit;
                let nt_base = 0x2000 + nt * 0x400;

                let nt_addr = nt_base + (coarse_y + fine_y / 8) * 32 + actual_tile_x;
                let tile_id = self.ppu_read(nt_addr, cart) as u16;

                let attr_addr = nt_base + 0x03C0 + ((coarse_y + fine_y / 8) / 4) * 8 + actual_tile_x / 4;
                let attr = self.ppu_read(attr_addr, cart);
                let shift = ((((coarse_y + fine_y / 8) & 2)) | ((actual_tile_x & 2) >> 1)) * 2;
                let palette_id = (attr >> shift) & 3;

                let pattern_addr = bg_pattern + tile_id * 16 + (fine_y & 7);
                let lo = self.ppu_read(pattern_addr, cart);
                let hi = self.ppu_read(pattern_addr + 8, cart);
                let bit = 7 - fine_x_pixel;
                let color = ((lo >> bit) & 1) | (((hi >> bit) & 1) << 1);

                (color, palette_id)
            } else {
                (0, 0)
            };

            // Sprite pixel
            let (spr_color, spr_palette, spr_priority, is_sprite0) = if spr_enabled && (spr_left || x >= 8) {
                self.get_sprite_pixel(x as u8, y as u8, spr_pattern, spr_h, cart)
            } else {
                (0, 0, false, false)
            };

            // Sprite 0 hit detection
            if is_sprite0 && bg_color != 0 && spr_color != 0 && x < 255 {
                self.status |= 0x40;
            }

            // Compose final pixel
            let final_color = if spr_color != 0 && (bg_color == 0 || !spr_priority) {
                // Sprite wins
                let idx = self.palette[16 + spr_palette as usize * 4 + spr_color as usize] as usize;
                NES_PALETTE[idx & 0x3F]
            } else if bg_color != 0 {
                let idx = self.palette[bg_palette as usize * 4 + bg_color as usize] as usize;
                NES_PALETTE[idx & 0x3F]
            } else {
                NES_PALETTE[self.palette[0] as usize & 0x3F]
            };

            self.framebuffer[y * 256 + screen_x] = final_color;
        }

        // Increment scroll Y at end of visible scanline
        if self.scanline < 240 {
            self.increment_y();
            // Copy horizontal bits from t to v
            self.v = (self.v & !0x041F) | (self.t & 0x041F);
        }
    }

    fn get_sprite_pixel(&self, x: u8, y: u8, spr_pattern: u16, spr_h: u8, cart: &Cartridge) -> (u8, u8, bool, bool) {
        for i in 0..self.sprite_count as usize {
            let idx = self.sprite_indices[i] as usize * 4;
            let spr_y = self.oam[idx] as i16;
            let spr_tile = self.oam[idx + 1];
            let spr_attr = self.oam[idx + 2];
            let spr_x = self.oam[idx + 3] as i16;

            if (x as i16) < spr_x || (x as i16) >= spr_x + 8 { continue; }

            let flip_h = spr_attr & 0x40 != 0;
            let flip_v = spr_attr & 0x80 != 0;
            let priority = spr_attr & 0x20 != 0; // behind background
            let palette_id = spr_attr & 3;

            let mut row = y as i16 - spr_y - 1;
            if flip_v { row = (spr_h as i16 - 1) - row; }
            if row < 0 || row >= spr_h as i16 { continue; }

            let (tile, pattern_base) = if spr_h == 16 {
                let bank = (spr_tile as u16 & 1) * 0x1000;
                let tile_num = spr_tile & 0xFE;
                if row < 8 {
                    (tile_num as u16, bank)
                } else {
                    ((tile_num + 1) as u16, bank)
                }
            } else {
                (spr_tile as u16, spr_pattern)
            };

            let actual_row = (row % 8) as u16;
            let pattern_addr = pattern_base + tile * 16 + actual_row;
            let lo = self.ppu_read(pattern_addr, cart);
            let hi = self.ppu_read(pattern_addr + 8, cart);

            let col = if flip_h { x as i16 - spr_x } else { 7 - (x as i16 - spr_x) };
            let color = ((lo >> col) & 1) | (((hi >> col) & 1) << 1);

            if color != 0 {
                return (color, palette_id, priority, self.sprite_indices[i] == 0);
            }
        }
        (0, 0, false, false)
    }

    fn evaluate_sprites(&mut self, _cart: &Cartridge) {
        self.sprite_count = 0;
        let y = self.scanline as u8;
        let h = if self.ctrl & 0x20 != 0 { 16i16 } else { 8i16 };

        for i in 0..64u8 {
            let spr_y = self.oam[i as usize * 4] as i16;
            let diff = y as i16 - spr_y;
            if diff >= 1 && diff <= h {
                if self.sprite_count < 8 {
                    self.sprite_indices[self.sprite_count as usize] = i;
                    self.sprite_count += 1;
                } else {
                    self.status |= 0x20; // Sprite overflow
                    break;
                }
            }
        }
    }

    fn increment_y(&mut self) {
        if (self.v & 0x7000) != 0x7000 {
            self.v += 0x1000; // Increment fine Y
        } else {
            self.v &= !0x7000; // Fine Y = 0
            let mut y = (self.v & 0x03E0) >> 5;
            if y == 29 {
                y = 0;
                self.v ^= 0x0800; // Switch vertical nametable
            } else if y == 31 {
                y = 0;
            } else {
                y += 1;
            }
            self.v = (self.v & !0x03E0) | (y << 5);
        }
    }

    /// OAM DMA — copy 256 bytes from CPU memory to OAM
    pub fn oam_dma(&mut self, data: &[u8; 256]) {
        self.oam.copy_from_slice(data);
    }
}
