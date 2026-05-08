

#![allow(dead_code)]

use alloc::vec;
use alloc::vec::Vec;
use super::cartridge::Cartridge;


pub const AHP_: [u32; 64] = [
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
    
    pub ctrl: u8,       
    pub mask: u8,       
    pub status: u8,     
    pub oam_addr: u8,   

    
    pub v: u16,         
    pub t: u16,         
    pub fine_x: u8,     
    pub w: bool,        
    pub data_buf: u8,   

    
    pub oam: [u8; 256],        
    pub vram: [u8; 2048],      
    pub palette: [u8; 32],     

    
    pub scanline: i32,
    pub dot: u32,
    pub frame_count: u64,
    pub nmi_triggered: bool,
    pub sprite0_hit_possible: bool,

    
    sprite_indices: [u8; 8],
    sprite_count: u8,

    
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

    

    pub fn read_register(&mut self, addr: u16, cart: &Cartridge) -> u8 {
        match addr & 7 {
            2 => { 
                let val = (self.status & 0xE0) | (self.data_buf & 0x1F);
                self.status &= !0x80; 
                self.w = false;
                val
            }
            4 => { 
                self.oam[self.oam_addr as usize]
            }
            7 => { 
                let addr = self.v & 0x3FFF;
                let val = if addr >= 0x3F00 {
                    self.palette_read(addr)
                } else {
                    let awl = self.data_buf;
                    self.data_buf = self.ppu_read(addr, cart);
                    awl
                };
                self.v = self.v.wrapping_add(if self.ctrl & 0x04 != 0 { 32 } else { 1 });
                val
            }
            _ => 0,
        }
    }

    pub fn write_register(&mut self, addr: u16, val: u8, cart: &mut Cartridge) {
        match addr & 7 {
            0 => { 
                self.ctrl = val;
                self.t = (self.t & 0xF3FF) | (((val as u16) & 3) << 10);
            }
            1 => self.mask = val,
            3 => self.oam_addr = val,
            4 => { 
                self.oam[self.oam_addr as usize] = val;
                self.oam_addr = self.oam_addr.wrapping_add(1);
            }
            5 => { 
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
            6 => { 
                if !self.w {
                    self.t = (self.t & 0x00FF) | (((val as u16) & 0x3F) << 8);
                } else {
                    self.t = (self.t & 0xFF00) | (val as u16);
                    self.v = self.t;
                }
                self.w = !self.w;
            }
            7 => { 
                let a = self.v & 0x3FFF;
                self.ppu_write(a, val, cart);
                self.v = self.v.wrapping_add(if self.ctrl & 0x04 != 0 { 32 } else { 1 });
            }
            _ => {}
        }
    }

    

    fn ppu_read(&self, addr: u16, cart: &Cartridge) -> u8 {
        match addr {
            0x0000..=0x1FFF => cart.ppu_read(addr),
            0x2000..=0x3EFF => {
                let dvk = cart.mirror_nametable(addr - 0x2000);
                self.vram[dvk as usize]
            }
            0x3F00..=0x3FFF => self.palette_read(addr),
            _ => 0,
        }
    }

    fn ppu_write(&mut self, addr: u16, val: u8, cart: &mut Cartridge) {
        match addr {
            0x0000..=0x1FFF => cart.ppu_write(addr, val),
            0x2000..=0x3EFF => {
                let dvk = cart.mirror_nametable(addr - 0x2000);
                self.vram[dvk as usize] = val;
            }
            0x3F00..=0x3FFF => {
                let idx = (addr & 0x1F) as usize;
                self.palette[idx] = val & 0x3F;
                
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

    

    
    pub fn step_scanline(&mut self, cart: &Cartridge) -> bool {
        let mut joq = false;
        let jaa = self.mask & 0x18 != 0;

        match self.scanline {
            0..=239 => {
                
                if jaa {
                    self.evaluate_sprites(cart);
                    self.render_scanline(cart);
                }
            }
            241 => {
                
                self.status |= 0x80;
                if self.ctrl & 0x80 != 0 {
                    joq = true;
                }
            }
            261 => {
                
                self.status &= !0xE0; 
                if jaa {
                    
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

        joq
    }

    fn render_scanline(&mut self, cart: &Cartridge) {
        let y = self.scanline as usize;
        if y >= 240 { return; }

        let kbq = self.mask & 0x08 != 0;
        let ovc = self.mask & 0x10 != 0;
        let kbt = self.mask & 0x02 != 0;
        let ovd = self.mask & 0x04 != 0;

        let bg_pattern = if self.ctrl & 0x10 != 0 { 0x1000u16 } else { 0u16 };
        let gvq = if self.ctrl & 0x08 != 0 { 0x1000u16 } else { 0u16 };
        let pcz = self.ctrl & 0x20 != 0;
        let eac = if pcz { 16 } else { 8 };

        
        let kun = self.v & 0x1F;
        let fnm = (self.v >> 5) & 0x1F;
        let emr = (self.v >> 12) & 7;
        let nlp = (self.v >> 10) & 3;

        for x in 0..256usize {
            let lw = x;
            let ccs = x as u16 + self.fine_x as u16;
            let jmo = (kun as u16 + ccs / 8) as u16;
            let lwd = (ccs % 8) as u8;

            
            let (bg_color, bg_palette) = if kbq && (kbt || x >= 8) {
                let fge = jmo & 0x1F;
                let nln = if jmo >= 32 { 1u16 } else { 0 };
                let nlm = nlp ^ nln;
                let ire = 0x2000 + nlm * 0x400;

                let dvk = ire + (fnm + emr / 8) * 32 + fge;
                let azw = self.ppu_read(dvk, cart) as u16;

                let jyb = ire + 0x03C0 + ((fnm + emr / 8) / 4) * 8 + fge / 4;
                let attr = self.ppu_read(jyb, cart);
                let no = ((((fnm + emr / 8) & 2)) | ((fge & 2) >> 1)) * 2;
                let gly = (attr >> no) & 3;

                let dwi = bg_pattern + azw * 16 + (emr & 7);
                let lo = self.ppu_read(dwi, cart);
                let hi = self.ppu_read(dwi + 8, cart);
                let bf = 7 - lwd;
                let color = ((lo >> bf) & 1) | (((hi >> bf) & 1) << 1);

                (color, gly)
            } else {
                (0, 0)
            };

            
            let (spr_color, spr_palette, spr_priority, is_sprite0) = if ovc && (ovd || x >= 8) {
                self.get_sprite_pixel(x as u8, y as u8, gvq, eac, cart)
            } else {
                (0, 0, false, false)
            };

            
            if is_sprite0 && bg_color != 0 && spr_color != 0 && x < 255 {
                self.status |= 0x40;
            }

            
            let dpo = if spr_color != 0 && (bg_color == 0 || !spr_priority) {
                
                let idx = self.palette[16 + spr_palette as usize * 4 + spr_color as usize] as usize;
                AHP_[idx & 0x3F]
            } else if bg_color != 0 {
                let idx = self.palette[bg_palette as usize * 4 + bg_color as usize] as usize;
                AHP_[idx & 0x3F]
            } else {
                AHP_[self.palette[0] as usize & 0x3F]
            };

            self.framebuffer[y * 256 + lw] = dpo;
        }

        
        if self.scanline < 240 {
            self.increment_y();
            
            self.v = (self.v & !0x041F) | (self.t & 0x041F);
        }
    }

    fn get_sprite_pixel(&self, x: u8, y: u8, gvq: u16, eac: u8, cart: &Cartridge) -> (u8, u8, bool, bool) {
        for i in 0..self.sprite_count as usize {
            let idx = self.sprite_indices[i] as usize * 4;
            let gvs = self.oam[idx] as i16;
            let gvr = self.oam[idx + 1];
            let fbf = self.oam[idx + 2];
            let fbg = self.oam[idx + 3] as i16;

            if (x as i16) < fbg || (x as i16) >= fbg + 8 { continue; }

            let lwx = fbf & 0x40 != 0;
            let lwy = fbf & 0x80 != 0;
            let priority = fbf & 0x20 != 0; 
            let gly = fbf & 3;

            let mut row = y as i16 - gvs - 1;
            if lwy { row = (eac as i16 - 1) - row; }
            if row < 0 || row >= eac as i16 { continue; }

            let (apf, pattern_base) = if eac == 16 {
                let gi = (gvr as u16 & 1) * 0x1000;
                let jmm = gvr & 0xFE;
                if row < 8 {
                    (jmm as u16, gi)
                } else {
                    ((jmm + 1) as u16, gi)
                }
            } else {
                (gvr as u16, gvq)
            };

            let cfs = (row % 8) as u16;
            let dwi = pattern_base + apf * 16 + cfs;
            let lo = self.ppu_read(dwi, cart);
            let hi = self.ppu_read(dwi + 8, cart);

            let col = if lwx { x as i16 - fbg } else { 7 - (x as i16 - fbg) };
            let color = ((lo >> col) & 1) | (((hi >> col) & 1) << 1);

            if color != 0 {
                return (color, gly, priority, self.sprite_indices[i] == 0);
            }
        }
        (0, 0, false, false)
    }

    fn evaluate_sprites(&mut self, _cart: &Cartridge) {
        self.sprite_count = 0;
        let y = self.scanline as u8;
        let h = if self.ctrl & 0x20 != 0 { 16i16 } else { 8i16 };

        for i in 0..64u8 {
            let gvs = self.oam[i as usize * 4] as i16;
            let jr = y as i16 - gvs;
            if jr >= 1 && jr <= h {
                if self.sprite_count < 8 {
                    self.sprite_indices[self.sprite_count as usize] = i;
                    self.sprite_count += 1;
                } else {
                    self.status |= 0x20; 
                    break;
                }
            }
        }
    }

    fn increment_y(&mut self) {
        if (self.v & 0x7000) != 0x7000 {
            self.v += 0x1000; 
        } else {
            self.v &= !0x7000; 
            let mut y = (self.v & 0x03E0) >> 5;
            if y == 29 {
                y = 0;
                self.v ^= 0x0800; 
            } else if y == 31 {
                y = 0;
            } else {
                y += 1;
            }
            self.v = (self.v & !0x03E0) | (y << 5);
        }
    }

    
    pub fn oam_dma(&mut self, data: &[u8; 256]) {
        self.oam.copy_from_slice(data);
    }
}
