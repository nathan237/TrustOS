

#![allow(dead_code)]

use alloc::vec;
use alloc::vec::Vec;


pub const KY_: [u32; 4] = [
    0xFFE0F8D0, 
    0xFF88C070, 
    0xFF346856, 
    0xFF081820, 
];

pub const FF_: usize = 160;
pub const AJJ_: usize = 144;

pub struct Gpu {
    pub vram: [u8; 8192],     
    pub vram1: [u8; 8192],    
    pub oam: [u8; 160],       
    pub framebuffer: Vec<u32>, 

    
    pub lcdc: u8,   
    pub stat: u8,   
    pub scy: u8,    
    pub scx: u8,    
    pub ly: u8,     
    pub lyc: u8,    
    pub bgp: u8,    
    pub obp0: u8,   
    pub obp1: u8,   
    pub wy: u8,     
    pub wx: u8,     

    pub mode: u8,       
    pub cycles: u32,    
    pub frame_ready: bool,
    pub stat_irq: bool,  
    pub vblank_irq: bool, 

    
    pub window_line: u8,     

    
    pub cgb_mode: bool,        
    pub vram_bank: u8,         
    pub bg_palette: [u8; 64],  
    pub obj_palette: [u8; 64], 
    pub bcps: u8,              
    pub ocps: u8,              
    
    bg_priority: [u8; 160],    
}

impl Gpu {
    pub fn new() -> Self {
        Self {
            vram: [0; 8192],
            vram1: [0; 8192],
            oam: [0; 160],
            framebuffer: vec![KY_[0]; FF_ * AJJ_],
            lcdc: 0x91,
            stat: 0x00,
            scy: 0,
            scx: 0,
            ly: 0,
            lyc: 0,
            bgp: 0xFC,
            obp0: 0xFF,
            obp1: 0xFF,
            wy: 0,
            wx: 0,
            mode: 2,
            cycles: 0,
            frame_ready: false,
            stat_irq: false,
            vblank_irq: false,
            window_line: 0,
            cgb_mode: false,
            vram_bank: 0,
            bg_palette: [0xFF; 64],
            obj_palette: [0xFF; 64],
            bcps: 0,
            ocps: 0,
            bg_priority: [0; 160],
        }
    }

    
    pub fn step(&mut self, cpu_cycles: u32) {
        if self.lcdc & 0x80 == 0 {
            
            return;
        }

        self.cycles += cpu_cycles * 4; 

        match self.mode {
            2 => { 
                if self.cycles >= 80 {
                    self.cycles -= 80;
                    self.mode = 3;
                }
            }
            3 => { 
                if self.cycles >= 172 {
                    self.cycles -= 172;
                    self.mode = 0;

                    
                    self.render_scanline();

                    
                    if self.stat & 0x08 != 0 {
                        self.stat_irq = true;
                    }
                }
            }
            0 => { 
                if self.cycles >= 204 {
                    self.cycles -= 204;
                    self.ly += 1;

                    if self.ly == 144 {
                        
                        self.mode = 1;
                        self.vblank_irq = true;
                        self.frame_ready = true;
                        self.window_line = 0;

                        
                        if self.stat & 0x10 != 0 {
                            self.stat_irq = true;
                        }
                    } else {
                        self.mode = 2;
                        
                        if self.stat & 0x20 != 0 {
                            self.stat_irq = true;
                        }
                    }

                    self.check_lyc();
                }
            }
            1 => { 
                if self.cycles >= 456 {
                    self.cycles -= 456;
                    self.ly += 1;

                    if self.ly >= 154 {
                        self.ly = 0;
                        self.mode = 2;

                        
                        if self.stat & 0x20 != 0 {
                            self.stat_irq = true;
                        }
                    }

                    self.check_lyc();
                }
            }
            _ => {}
        }
    }

    fn check_lyc(&mut self) {
        if self.ly == self.lyc {
            self.stat |= 0x04; 
            if self.stat & 0x40 != 0 {
                self.stat_irq = true;
            }
        } else {
            self.stat &= !0x04;
        }
    }

    pub fn read_stat(&self) -> u8 {
        (self.stat & 0xF8) | (if self.ly == self.lyc { 0x04 } else { 0 }) | self.mode
    }

    
    fn render_scanline(&mut self) {
        let ly = self.ly as usize;
        if ly >= AJJ_ { return; }

        let offset = ly * FF_;

        
        for x in 0..FF_ {
            self.framebuffer[offset + x] = if self.cgb_mode {
                Self::eho(&self.bg_palette, 0, 0)
            } else {
                KY_[0]
            };
            self.bg_priority[x] = 0;
        }

        
        if self.cgb_mode || self.lcdc & 0x01 != 0 {
            if self.cgb_mode {
                self.render_bg_scanline_cgb(ly, offset);
            } else {
                self.render_bg_scanline(ly, offset);
            }
        }

        
        if self.lcdc & 0x20 != 0 && (self.cgb_mode || self.lcdc & 0x01 != 0) {
            if self.cgb_mode {
                self.render_window_scanline_cgb(ly, offset);
            } else {
                self.render_window_scanline(ly, offset);
            }
        }

        
        if self.lcdc & 0x02 != 0 {
            if self.cgb_mode {
                self.render_sprite_scanline_cgb(ly, offset);
            } else {
                self.render_sprite_scanline(ly, offset);
            }
        }
    }

    fn render_bg_scanline(&mut self, ly: usize, offset: usize) {
        let bjo = if self.lcdc & 0x10 != 0 { 0x0000usize } else { 0x0800 };
        let bwk = if self.lcdc & 0x08 != 0 { 0x1C00usize } else { 0x1800 };
        let cqp = self.lcdc & 0x10 == 0;

        let y = (self.scy as usize + ly) & 0xFF;
        let crp = y / 8;
        let buy = y % 8;

        for x in 0..FF_ {
            let dxm = (self.scx as usize + x) & 0xFF;
            let cro = dxm / 8;
            let buw = dxm % 8;

            let bhw = crp * 32 + cro;
            let azw = self.vram[bwk + bhw];

            let ako = if cqp {
                let cqq = azw as i8 as i32;
                (bjo as i32 + (cqq + 128) * 16) as usize
            } else {
                bjo + azw as usize * 16
            };

            let asb = ako + buy * 2;
            if asb + 1 >= self.vram.len() { continue; }

            let lo = self.vram[asb];
            let hi = self.vram[asb + 1];
            let bf = 7 - buw;
            let alf = ((hi >> bf) & 1) << 1 | ((lo >> bf) & 1);

            let auk = (self.bgp >> (alf * 2)) & 3;
            self.framebuffer[offset + x] = KY_[auk as usize];
        }
    }

    fn render_window_scanline(&mut self, ly: usize, offset: usize) {
        if ly < self.wy as usize { return; }
        let wx = self.wx as i32 - 7;

        let bjo = if self.lcdc & 0x10 != 0 { 0x0000usize } else { 0x0800 };
        let bwk = if self.lcdc & 0x40 != 0 { 0x1C00usize } else { 0x1800 };
        let cqp = self.lcdc & 0x10 == 0;

        let qr = self.window_line as usize;
        let crp = qr / 8;
        let buy = qr % 8;

        let mut dxq = false;

        for x in 0..FF_ {
            let nw = x as i32 - wx;
            if nw < 0 { continue; }
            dxq = true;

            let cro = nw as usize / 8;
            let buw = nw as usize % 8;
            let bhw = crp * 32 + cro;
            if bhw >= 1024 { continue; }

            let azw = self.vram[bwk + bhw];

            let ako = if cqp {
                let cqq = azw as i8 as i32;
                (bjo as i32 + (cqq + 128) * 16) as usize
            } else {
                bjo + azw as usize * 16
            };

            let asb = ako + buy * 2;
            if asb + 1 >= self.vram.len() { continue; }

            let lo = self.vram[asb];
            let hi = self.vram[asb + 1];
            let bf = 7 - buw;
            let alf = ((hi >> bf) & 1) << 1 | ((lo >> bf) & 1);

            let auk = (self.bgp >> (alf * 2)) & 3;
            self.framebuffer[offset + x] = KY_[auk as usize];
        }

        if dxq {
            self.window_line += 1;
        }
    }

    fn render_sprite_scanline(&mut self, ly: usize, offset: usize) {
        let ape = if self.lcdc & 0x04 != 0 { 16 } else { 8 };

        
        let mut ead: [(u8, u8, u8, u8, usize); 10] = [(0, 0, 0, 0, 0); 10];
        let mut count = 0usize;

        for i in 0..40 {
            let ak = self.oam[i * 4] as i32 - 16;
            let am = self.oam[i * 4 + 1] as i32 - 8;
            let apf = self.oam[i * 4 + 2];
            let flags = self.oam[i * 4 + 3];

            if ly as i32 >= ak && (ly as i32) < ak + ape as i32 {
                if count < 10 {
                    ead[count] = (
                        self.oam[i * 4],
                        self.oam[i * 4 + 1],
                        apf,
                        flags,
                        i,
                    );
                    count += 1;
                }
            }
        }

        
        for i in (0..count).rev() {
            let (sy_raw, sx_raw, mut apf, flags, _oam_idx) = ead[i];
            let ak = sy_raw as i32 - 16;
            let am = sx_raw as i32 - 8;
            let cjp = flags & 0x20 != 0;
            let cjq = flags & 0x40 != 0;
            let fir = flags & 0x80 != 0;
            let palette = if flags & 0x10 != 0 { self.obp1 } else { self.obp0 };

            let mut row = ly as i32 - ak;
            if cjq { row = ape as i32 - 1 - row; }

            if ape == 16 {
                apf &= 0xFE; 
                if row >= 8 {
                    apf += 1;
                    row -= 8;
                }
            }

            let ako = apf as usize * 16 + row as usize * 2;
            if ako + 1 >= self.vram.len() { continue; }

            let lo = self.vram[ako];
            let hi = self.vram[ako + 1];

            for p in 0..8i32 {
                let lw = am + p;
                if lw < 0 || lw >= FF_ as i32 { continue; }

                let bf = if cjp { p } else { 7 - p } as u8;
                let alf = ((hi >> bf) & 1) << 1 | ((lo >> bf) & 1);
                if alf == 0 { continue; } 

                let ezp = offset + lw as usize;

                
                if fir {
                    let bg_color = self.framebuffer[ezp];
                    if bg_color != KY_[0] { continue; }
                }

                let auk = (palette >> (alf * 2)) & 3;
                self.framebuffer[ezp] = KY_[auk as usize];
            }
        }
    }

    

    fn render_bg_scanline_cgb(&mut self, ly: usize, offset: usize) {
        let bjo = if self.lcdc & 0x10 != 0 { 0x0000usize } else { 0x0800 };
        let bwk = if self.lcdc & 0x08 != 0 { 0x1C00usize } else { 0x1800 };
        let cqp = self.lcdc & 0x10 == 0;

        let y = (self.scy as usize + ly) & 0xFF;
        let crp = y / 8;
        let buy = y % 8;

        for x in 0..FF_ {
            let dxm = (self.scx as usize + x) & 0xFF;
            let cro = dxm / 8;
            let buw = dxm % 8;

            let bhw = crp * 32 + cro;
            let azw = self.vram[bwk + bhw];
            
            let beq = self.vram1[bwk + bhw];
            let dkm = (beq & 0x07) as usize;
            let ebs = (beq >> 3) & 1;
            let cjp = beq & 0x20 != 0;
            let cjq = beq & 0x40 != 0;
            let bg_priority = beq & 0x80 != 0;

            let ako = if cqp {
                let cqq = azw as i8 as i32;
                (bjo as i32 + (cqq + 128) * 16) as usize
            } else {
                bjo + azw as usize * 16
            };

            let cfs = if cjq { 7 - buy } else { buy };
            let asb = ako + cfs * 2;
            let bjx = if ebs == 1 { &self.vram1 } else { &self.vram };
            if asb + 1 >= bjx.len() { continue; }

            let lo = bjx[asb];
            let hi = bjx[asb + 1];
            let bf = if cjp { buw } else { 7 - buw };
            let alf = ((hi >> bf) & 1) << 1 | ((lo >> bf) & 1);

            let color = Self::eho(&self.bg_palette, dkm, alf as usize);
            self.framebuffer[offset + x] = color;
            
            self.bg_priority[x] = (if alf != 0 { 1 } else { 0 })
                | (if bg_priority { 2 } else { 0 });
        }
    }

    fn render_window_scanline_cgb(&mut self, ly: usize, offset: usize) {
        if ly < self.wy as usize { return; }
        let wx = self.wx as i32 - 7;

        let bjo = if self.lcdc & 0x10 != 0 { 0x0000usize } else { 0x0800 };
        let bwk = if self.lcdc & 0x40 != 0 { 0x1C00usize } else { 0x1800 };
        let cqp = self.lcdc & 0x10 == 0;

        let qr = self.window_line as usize;
        let crp = qr / 8;
        let buy = qr % 8;

        let mut dxq = false;

        for x in 0..FF_ {
            let nw = x as i32 - wx;
            if nw < 0 { continue; }
            dxq = true;

            let cro = nw as usize / 8;
            let buw = nw as usize % 8;
            let bhw = crp * 32 + cro;
            if bhw >= 1024 { continue; }

            let azw = self.vram[bwk + bhw];
            let beq = self.vram1[bwk + bhw];
            let dkm = (beq & 0x07) as usize;
            let ebs = (beq >> 3) & 1;
            let cjp = beq & 0x20 != 0;
            let cjq = beq & 0x40 != 0;
            let bg_priority = beq & 0x80 != 0;

            let ako = if cqp {
                let cqq = azw as i8 as i32;
                (bjo as i32 + (cqq + 128) * 16) as usize
            } else {
                bjo + azw as usize * 16
            };

            let cfs = if cjq { 7 - buy } else { buy };
            let asb = ako + cfs * 2;
            let bjx = if ebs == 1 { &self.vram1 } else { &self.vram };
            if asb + 1 >= bjx.len() { continue; }

            let lo = bjx[asb];
            let hi = bjx[asb + 1];
            let bf = if cjp { buw } else { 7 - buw };
            let alf = ((hi >> bf) & 1) << 1 | ((lo >> bf) & 1);

            let color = Self::eho(&self.bg_palette, dkm, alf as usize);
            self.framebuffer[offset + x] = color;
            self.bg_priority[x] = (if alf != 0 { 1 } else { 0 })
                | (if bg_priority { 2 } else { 0 });
        }

        if dxq {
            self.window_line += 1;
        }
    }

    fn render_sprite_scanline_cgb(&mut self, ly: usize, offset: usize) {
        let ape = if self.lcdc & 0x04 != 0 { 16 } else { 8 };

        let mut ead: [(u8, u8, u8, u8, usize); 10] = [(0, 0, 0, 0, 0); 10];
        let mut count = 0usize;

        for i in 0..40 {
            let ak = self.oam[i * 4] as i32 - 16;
            if ly as i32 >= ak && (ly as i32) < ak + ape as i32 {
                if count < 10 {
                    ead[count] = (
                        self.oam[i * 4],
                        self.oam[i * 4 + 1],
                        self.oam[i * 4 + 2],
                        self.oam[i * 4 + 3],
                        i,
                    );
                    count += 1;
                }
            }
        }

        
        for i in (0..count).rev() {
            let (sy_raw, sx_raw, mut apf, flags, _) = ead[i];
            let ak = sy_raw as i32 - 16;
            let am = sx_raw as i32 - 8;
            let cjp = flags & 0x20 != 0;
            let cjq = flags & 0x40 != 0;
            let fir = flags & 0x80 != 0;
            let dkm = (flags & 0x07) as usize;
            let ebs = (flags >> 3) & 1;

            let mut row = ly as i32 - ak;
            if cjq { row = ape as i32 - 1 - row; }

            if ape == 16 {
                apf &= 0xFE;
                if row >= 8 { apf += 1; row -= 8; }
            }

            let ako = apf as usize * 16 + row as usize * 2;
            let bjx = if ebs == 1 { &self.vram1 } else { &self.vram };
            if ako + 1 >= bjx.len() { continue; }

            let lo = bjx[ako];
            let hi = bjx[ako + 1];

            for p in 0..8i32 {
                let lw = am + p;
                if lw < 0 || lw >= FF_ as i32 { continue; }

                let bf = if cjp { p } else { 7 - p } as u8;
                let alf = ((hi >> bf) & 1) << 1 | ((lo >> bf) & 1);
                if alf == 0 { continue; } 

                let jkf = lw as usize;
                let ezp = offset + jkf;

                
                
                if self.lcdc & 0x01 != 0 {
                    let hhr = self.bg_priority[jkf];
                    if (fir || hhr & 2 != 0) && hhr & 1 != 0 {
                        continue;
                    }
                }

                let color = Self::eho(&self.obj_palette, dkm, alf as usize);
                self.framebuffer[ezp] = color;
            }
        }
    }

    
    pub fn read_vram(&self, addr: u16) -> u8 {
        let idx = (addr & 0x1FFF) as usize;
        if self.vram_bank == 1 { self.vram1[idx] } else { self.vram[idx] }
    }
    pub fn write_vram(&mut self, addr: u16, val: u8) {
        let idx = (addr & 0x1FFF) as usize;
        if self.vram_bank == 1 { self.vram1[idx] = val; } else { self.vram[idx] = val; }
    }
    
    pub fn qsy(&self, addr: u16, gi: u8) -> u8 {
        let idx = (addr & 0x1FFF) as usize;
        if gi == 1 { self.vram1[idx] } else { self.vram[idx] }
    }

    
    pub fn read_bcpd(&self) -> u8 {
        let idx = (self.bcps & 0x3F) as usize;
        self.bg_palette[idx]
    }
    pub fn write_bcpd(&mut self, val: u8) {
        let idx = (self.bcps & 0x3F) as usize;
        self.bg_palette[idx] = val;
        if self.bcps & 0x80 != 0 {
            self.bcps = 0x80 | ((self.bcps + 1) & 0x3F);
        }
    }
    pub fn read_ocpd(&self) -> u8 {
        let idx = (self.ocps & 0x3F) as usize;
        self.obj_palette[idx]
    }
    pub fn write_ocpd(&mut self, val: u8) {
        let idx = (self.ocps & 0x3F) as usize;
        self.obj_palette[idx] = val;
        if self.ocps & 0x80 != 0 {
            self.ocps = 0x80 | ((self.ocps + 1) & 0x3F);
        }
    }

    
    fn eho(palette_data: &[u8], palette_num: usize, color_idx: usize) -> u32 {
        let offset = palette_num * 8 + color_idx * 2;
        if offset + 1 >= palette_data.len() { return 0xFF000000; }
        let lo = palette_data[offset] as u16;
        let hi = palette_data[offset + 1] as u16;
        let cpj = lo | (hi << 8);
        let bde = (cpj & 0x1F) as u8;
        let iap = ((cpj >> 5) & 0x1F) as u8;
        let agb = ((cpj >> 10) & 0x1F) as u8;
        
        let r = (bde << 3) | (bde >> 2);
        let g = (iap << 3) | (iap >> 2);
        let b = (agb << 3) | (agb >> 2);
        0xFF000000 | (r as u32) << 16 | (g as u32) << 8 | b as u32
    }

    
    pub fn read_oam(&self, addr: u16) -> u8 {
        let idx = (addr - 0xFE00) as usize;
        if idx < 160 { self.oam[idx] } else { 0xFF }
    }
    pub fn write_oam(&mut self, addr: u16, val: u8) {
        let idx = (addr - 0xFE00) as usize;
        if idx < 160 { self.oam[idx] = val; }
    }
}
