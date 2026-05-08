

#![allow(dead_code)]

pub mod cartridge;
pub mod cpu;
pub mod ppu;

use alloc::vec;
use alloc::vec::Vec;
use cartridge::Cartridge;
use cpu::{Cpu, Ax};
use ppu::Ppu;

const BDK_: usize = 256;
const CKU_: usize = 240;
const BTQ_: u32 = 114; 


pub struct NesEmulator {
    cpu: Cpu,
    ppu: Ppu,
    ram: [u8; 2048],
    cart: Cartridge,
    rom_loaded: bool,

    
    controller_state: [u8; 2],
    controller_shift: [u8; 2],
    controller_strobe: bool,

    
    dma_page: u8,
    dma_pending: bool,

    
    frame_ready: bool,
    frame_count: u64,

    
    key_state: u8, 
}

struct Et<'a> {
    ram: &'a mut [u8; 2048],
    ppu: &'a mut Ppu,
    cart: &'a mut Cartridge,
    controller_state: &'a [u8; 2],
    controller_shift: &'a mut [u8; 2],
    controller_strobe: &'a bool,
    dma_page: &'a mut u8,
    dma_pending: &'a mut bool,
}

impl<'a> Ax for Et<'a> {
    fn cpu_read(&mut self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x1FFF => self.ram[(addr & 0x07FF) as usize],
            0x2000..=0x3FFF => self.ppu.read_register(addr, self.cart),
            0x4016 => {
                let val = (self.controller_shift[0] & 0x80) >> 7;
                self.controller_shift[0] <<= 1;
                val | 0x40
            }
            0x4017 => {
                let val = (self.controller_shift[1] & 0x80) >> 7;
                self.controller_shift[1] <<= 1;
                val | 0x40
            }
            0x4000..=0x4015 => 0, 
            0x4018..=0x5FFF => 0, 
            0x6000..=0xFFFF => self.cart.cpu_read(addr),
            _ => 0,
        }
    }

    fn cpu_write(&mut self, addr: u16, val: u8) {
        match addr {
            0x0000..=0x1FFF => self.ram[(addr & 0x07FF) as usize] = val,
            0x2000..=0x3FFF => self.ppu.write_register(addr, val, self.cart),
            0x4014 => {
                
                *self.dma_page = val;
                *self.dma_pending = true;
            }
            0x4016 => {
                if val & 1 != 0 {
                    
                    self.controller_shift[0] = self.controller_state[0];
                    self.controller_shift[1] = self.controller_state[1];
                }
            }
            0x4000..=0x4015 | 0x4017 => {} 
            0x4018..=0x5FFF => {} 
            0x6000..=0xFFFF => self.cart.cpu_write(addr, val),
            _ => {}
        }
    }
}

impl NesEmulator {
    pub fn new() -> Self {
        Self {
            cpu: Cpu::new(),
            ppu: Ppu::new(),
            ram: [0; 2048],
            cart: Cartridge::empty(),
            rom_loaded: false,
            controller_state: [0; 2],
            controller_shift: [0; 2],
            controller_strobe: false,
            dma_page: 0,
            dma_pending: false,
            frame_ready: false,
            frame_count: 0,
            key_state: 0,
        }
    }

    
    pub fn load_rom(&mut self, data: &[u8]) -> bool {
        if let Some(cart) = Cartridge::lzi(data) {
            self.cart = cart;
            self.rom_loaded = true;
            
            self.cpu = Cpu::new();
            self.ppu = Ppu::new();
            self.ram = [0; 2048];
            {
                let mut bus = Et {
                    ram: &mut self.ram,
                    ppu: &mut self.ppu,
                    cart: &mut self.cart,
                    controller_state: &self.controller_state,
                    controller_shift: &mut self.controller_shift,
                    controller_strobe: &self.controller_strobe,
                    dma_page: &mut self.dma_page,
                    dma_pending: &mut self.dma_pending,
                };
                self.cpu.reset(&mut bus);
            }
            crate::serial_println!("[NES] ROM loaded, PC={:#06X}", self.cpu.pc);
            true
        } else {
            false
        }
    }

    fn nby(&mut self) -> Et<'_> {
        Et {
            ram: &mut self.ram,
            ppu: &mut self.ppu,
            cart: &mut self.cart,
            controller_state: &self.controller_state,
            controller_shift: &mut self.controller_shift,
            controller_strobe: &self.controller_strobe,
            dma_page: &mut self.dma_page,
            dma_pending: &mut self.dma_pending,
        }
    }

    

    
    pub fn handle_key(&mut self, key: u8) {
        match key {
            b'x' | b'X' => self.key_state |= 0x80, 
            b'z' | b'Z' => self.key_state |= 0x40, 
            b'c' | b'C' => self.key_state |= 0x20, 
            b'\r' | 10  => self.key_state |= 0x10, 
            b'w' | b'W' | 0xF0 => self.key_state |= 0x08, 
            b's' | b'S' | 0xF1 => self.key_state |= 0x04, 
            b'a' | b'A' | 0xF2 => self.key_state |= 0x02, 
            b'd' | b'D' | 0xF3 => self.key_state |= 0x01, 
            b' '        => self.key_state |= 0x80, 
            _ => {}
        }
        self.controller_state[0] = self.key_state;
    }

    pub fn handle_key_release(&mut self, key: u8) {
        match key {
            b'x' | b'X' => self.key_state &= !0x80,
            b'z' | b'Z' => self.key_state &= !0x40,
            b'c' | b'C' => self.key_state &= !0x20,
            b'\r' | 10  => self.key_state &= !0x10,
            b'w' | b'W' | 0xF0 => self.key_state &= !0x08,
            b's' | b'S' | 0xF1 => self.key_state &= !0x04,
            b'a' | b'A' | 0xF2 => self.key_state &= !0x02,
            b'd' | b'D' | 0xF3 => self.key_state &= !0x01,
            b' '        => self.key_state &= !0x80,
            _ => {}
        }
        self.controller_state[0] = self.key_state;
    }

    

    
    pub fn tick(&mut self) {
        if !self.rom_loaded { return; }

        
        for _ in 0..262 {
            
            let mut gsv: u32 = 0;
            while gsv < BTQ_ {
                
                if self.dma_pending {
                    self.dma_pending = false;
                    let base = (self.dma_page as u16) << 8;
                    let mut hsu = [0u8; 256];
                    for i in 0..256u16 {
                        let addr = base | i;
                        hsu[i as usize] = match addr {
                            0x0000..=0x1FFF => self.ram[(addr & 0x07FF) as usize],
                            0x6000..=0xFFFF => self.cart.cpu_read(addr),
                            _ => 0,
                        };
                    }
                    self.ppu.oam_dma(&hsu);
                    gsv += 513;
                    continue;
                }

                let mut bus = Et {
                    ram: &mut self.ram,
                    ppu: &mut self.ppu,
                    cart: &mut self.cart,
                    controller_state: &self.controller_state,
                    controller_shift: &mut self.controller_shift,
                    controller_strobe: &self.controller_strobe,
                    dma_page: &mut self.dma_page,
                    dma_pending: &mut self.dma_pending,
                };
                let cycles = self.cpu.step(&mut bus);
                gsv += cycles;
            }

            
            let ayo = self.ppu.step_scanline(&self.cart);
            if ayo {
                self.cpu.nmi_pending = true;
            }
        }

        self.frame_count += 1;
    }

    

    
    pub fn render(&self, output: &mut [u32], out_w: usize, out_h: usize) {
        if !self.rom_loaded {
            self.render_no_rom_screen(output, out_w, out_h);
            return;
        }

        
        for hk in 0..out_h {
            let ak = hk * CKU_ / out_h;
            for fh in 0..out_w {
                let am = fh * BDK_ / out_w;
                let color = self.ppu.framebuffer[ak * BDK_ + am];
                output[hk * out_w + fh] = color;
            }
        }
    }

    fn render_no_rom_screen(&self, output: &mut [u32], w: usize, h: usize) {
        
        for i in 0..w * h {
            output[i] = 0xFF0F0F23;
        }

        
        let title = "TrustNES";
        let subtitle = "Insert ROM to play";
        let hnn = "WASD:Dpad X:A Z:B C:Select Enter:Start";

        let cx = w / 2;
        let ty = h / 3;
        self.draw_text(output, w, h, cx - title.len() * 4, ty, title, 0xFFFF4444);
        self.draw_text(output, w, h, cx - subtitle.len() * 4, ty + 30, subtitle, 0xFF888888);
        self.draw_text(output, w, h, cx - hnn.len() * 4, ty + 55, hnn, 0xFF666666);

        
        let u = h / 2 + 30;
        let bx = cx - 30;
        
        for ad in 0..20u32 {
            for dx in 0..60u32 {
                let p = bx + dx as usize;
                let o = u + ad as usize;
                if p < w && o < h {
                    output[o * w + p] = 0xFF333333;
                }
            }
        }
        
        for d in 0..5u32 {
            let p = bx + 12; let o = u + 5 + d as usize;
            if p < w && o < h { output[o * w + p] = 0xFF666666; }
            let p = bx + 10 + d as usize; let o = u + 7;
            if p < w && o < h { output[o * w + p] = 0xFF666666; }
        }
        
        for zs in [bx + 42, bx + 50] {
            for ad in 0..4u32 {
                for dx in 0..4u32 {
                    let p = zs + dx as usize;
                    let o = u + 6 + ad as usize;
                    if p < w && o < h {
                        output[o * w + p] = 0xFFCC2222;
                    }
                }
            }
        }
    }

    fn draw_text(&self, buf: &mut [u32], w: usize, h: usize, x: usize, y: usize, text: &str, color: u32) {
        
        const Ajx: [u64; 128] = {
            let mut f = [0u64; 128];
            f[b'A' as usize] = 0x4A_EA_CE; f[b'B' as usize] = 0xCA_CA_CE;
            f[b'C' as usize] = 0x68_88_6E; f[b'D' as usize] = 0xCA_AA_CE;
            f[b'E' as usize] = 0xE8_C8_EE; f[b'F' as usize] = 0xE8_C8_80;
            f[b'G' as usize] = 0x68_A8_6E; f[b'H' as usize] = 0xAA_EA_AE;
            f[b'I' as usize] = 0xE4_44_EE; f[b'J' as usize] = 0x22_2A_4E;
            f[b'K' as usize] = 0xAA_CA_AE; f[b'L' as usize] = 0x88_88_EE;
            f[b'M' as usize] = 0xAE_EA_AE; f[b'N' as usize] = 0xAE_EA_AE;
            f[b'O' as usize] = 0x4A_AA_4E; f[b'P' as usize] = 0xCA_C8_80;
            f[b'Q' as usize] = 0x4A_AE_6E; f[b'R' as usize] = 0xCA_CA_AE;
            f[b'S' as usize] = 0x68_42_CE; f[b'T' as usize] = 0xE4_44_40;
            f[b'U' as usize] = 0xAA_AA_EE; f[b'V' as usize] = 0xAA_AA_40;
            f[b'W' as usize] = 0xAA_EE_AE; f[b'X' as usize] = 0xAA_4A_AE;
            f[b'Y' as usize] = 0xAA_44_40; f[b'Z' as usize] = 0xE2_48_EE;
            f[b'0' as usize] = 0x4A_AA_4E; f[b'1' as usize] = 0x4C_44_EE;
            f[b'2' as usize] = 0xC2_48_EE; f[b'3' as usize] = 0xC2_42_CE;
            f[b'4' as usize] = 0xAA_E2_2E; f[b'5' as usize] = 0xE8_C2_CE;
            f[b'6' as usize] = 0x68_CA_6E; f[b'7' as usize] = 0xE2_24_40;
            f[b'8' as usize] = 0x6A_4A_6E; f[b'9' as usize] = 0x6A_62_CE;
            f[b':' as usize] = 0x04_04_00; f[b' ' as usize] = 0x00_00_00;
            f[b'.' as usize] = 0x00_00_40;
            f
        };

        let mut cx = x;
        for ch in text.bytes() {
            let idx = ch as usize;
            let du = if idx < 128 { Ajx[idx] } else { 0 };
            if du != 0 || ch == b' ' {
                for row in 0..5 {
                    for col in 0..4 {
                        let bf = (du >> (20 - row * 4 - col)) & 1;
                        if bf != 0 {
                            let p = cx + col as usize;
                            let o = y + row as usize;
                            if p < w && o < h {
                                buf[o * w + p] = color;
                            }
                        }
                    }
                }
            }
            cx += 5;
        }
    }
}
