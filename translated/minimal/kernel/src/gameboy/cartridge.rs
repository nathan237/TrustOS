

#![allow(dead_code)]

use alloc::vec;
use alloc::vec::Vec;

pub struct Cartridge {
    pub rom: Vec<u8>,
    pub ram: Vec<u8>,
    pub mbc_type: MbcType,
    pub rom_bank: u16,
    pub ram_bank: u8,
    pub ram_enabled: bool,
    pub mode: u8, 
    pub title: [u8; 16],
    pub cgb_flag: u8, 
}

#[derive(Clone, Copy, PartialEq)]
pub enum MbcType {
    None,   
    Mbc1,   
    Mbc3,   
    Mbc5,   
}

impl Cartridge {
    pub fn empty() -> Self {
        Self {
            rom: vec![0u8; 32768],
            ram: vec![0u8; 8192],
            mbc_type: MbcType::None,
            rom_bank: 1,
            ram_bank: 0,
            ram_enabled: false,
            mode: 0,
            title: [0; 16],
            cgb_flag: 0,
        }
    }

    pub fn lzq(data: &[u8]) -> Option<Self> {
        if data.len() < 0x150 { return None; }

        
        let mut title = [0u8; 16];
        for i in 0..16 {
            title[i] = data[0x134 + i];
        }

        
        let cgb_flag = data[0x143];

        
        let hjw = data[0x147];
        let mbc_type = match hjw {
            0x00 | 0x08 | 0x09 => MbcType::None,
            0x01..=0x03 => MbcType::Mbc1,
            0x0F..=0x13 => MbcType::Mbc3,
            0x19..=0x1E => MbcType::Mbc5,
            _ => MbcType::None,
        };

        
        let grx = match data[0x148] {
            0 => 32 * 1024,
            1 => 64 * 1024,
            2 => 128 * 1024,
            3 => 256 * 1024,
            4 => 512 * 1024,
            5 => 1024 * 1024,
            6 => 2048 * 1024,
            7 => 4096 * 1024,
            _ => data.len(),
        };

        
        let ram_size = match data[0x149] {
            0 => 0,
            1 => 2 * 1024,
            2 => 8 * 1024,
            3 => 32 * 1024,
            4 => 128 * 1024,
            5 => 64 * 1024,
            _ => 8 * 1024,
        };

        let rom = if data.len() >= grx {
            data[..grx].to_vec()
        } else {
            let mut r = data.to_vec();
            r.resize(grx, 0);
            r
        };

        let ram = vec![0u8; ram_size.max(8192)];

        let ebu: Vec<u8> = title.iter().copied().take_while(|&c| c != 0 && c >= 0x20).collect();
        crate::serial_println!("[GB] ROM: \"{}\" type={:#04X} mbc={:?} ROM={}KB RAM={}KB CGB={:#04X}",
            core::str::from_utf8(&ebu).unwrap_or("???"),
            hjw,
            match mbc_type { MbcType::None => "None", MbcType::Mbc1 => "MBC1", MbcType::Mbc3 => "MBC3", MbcType::Mbc5 => "MBC5" },
            rom.len() / 1024,
            ram_size / 1024,
            cgb_flag);

        Some(Self {
            rom,
            ram,
            mbc_type,
            rom_bank: 1,
            ram_bank: 0,
            ram_enabled: false,
            mode: 0,
            title,
            cgb_flag,
        })
    }

    pub fn read(&self, addr: u16) -> u8 {
        match self.mbc_type {
            MbcType::None => self.mbc0_read(addr),
            MbcType::Mbc1 => self.mbc1_read(addr),
            MbcType::Mbc3 => self.mbc3_read(addr),
            MbcType::Mbc5 => self.mbc5_read(addr),
        }
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        match self.mbc_type {
            MbcType::None => self.mbc0_write(addr, val),
            MbcType::Mbc1 => self.mbc1_write(addr, val),
            MbcType::Mbc3 => self.mbc3_write(addr, val),
            MbcType::Mbc5 => self.mbc5_write(addr, val),
        }
    }

    
    fn mbc0_read(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x7FFF => {
                if (addr as usize) < self.rom.len() { self.rom[addr as usize] } else { 0xFF }
            }
            0xA000..=0xBFFF => {
                let idx = (addr - 0xA000) as usize;
                if idx < self.ram.len() { self.ram[idx] } else { 0xFF }
            }
            _ => 0xFF,
        }
    }
    fn mbc0_write(&mut self, addr: u16, val: u8) {
        if addr >= 0xA000 && addr <= 0xBFFF {
            let idx = (addr - 0xA000) as usize;
            if idx < self.ram.len() { self.ram[idx] = val; }
        }
    }

    
    fn mbc1_read(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x3FFF => {
                if self.mode == 1 {
                    let gi = ((self.ram_bank as usize & 3) << 5) % (self.rom.len() / 16384).max(1);
                    let idx = gi * 16384 + addr as usize;
                    if idx < self.rom.len() { self.rom[idx] } else { 0xFF }
                } else {
                    if (addr as usize) < self.rom.len() { self.rom[addr as usize] } else { 0xFF }
                }
            }
            0x4000..=0x7FFF => {
                let gi = if self.rom_bank == 0 { 1 } else { self.rom_bank as usize };
                let lzz = (gi | ((self.ram_bank as usize & 3) << 5)) % (self.rom.len() / 16384).max(1);
                let idx = lzz * 16384 + (addr as usize - 0x4000);
                if idx < self.rom.len() { self.rom[idx] } else { 0xFF }
            }
            0xA000..=0xBFFF => {
                if !self.ram_enabled { return 0xFF; }
                let gi = if self.mode == 1 { self.ram_bank as usize } else { 0 };
                let idx = gi * 8192 + (addr as usize - 0xA000);
                if idx < self.ram.len() { self.ram[idx] } else { 0xFF }
            }
            _ => 0xFF,
        }
    }
    fn mbc1_write(&mut self, addr: u16, val: u8) {
        match addr {
            0x0000..=0x1FFF => self.ram_enabled = (val & 0x0F) == 0x0A,
            0x2000..=0x3FFF => {
                let mut gi = (val & 0x1F) as u16;
                if gi == 0 { gi = 1; }
                self.rom_bank = gi;
            }
            0x4000..=0x5FFF => self.ram_bank = val & 3,
            0x6000..=0x7FFF => self.mode = val & 1,
            0xA000..=0xBFFF => {
                if !self.ram_enabled { return; }
                let gi = if self.mode == 1 { self.ram_bank as usize } else { 0 };
                let idx = gi * 8192 + (addr as usize - 0xA000);
                if idx < self.ram.len() { self.ram[idx] = val; }
            }
            _ => {}
        }
    }

    
    fn mbc3_read(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x3FFF => {
                if (addr as usize) < self.rom.len() { self.rom[addr as usize] } else { 0xFF }
            }
            0x4000..=0x7FFF => {
                let gi = if self.rom_bank == 0 { 1 } else { self.rom_bank as usize };
                let idx = (gi % (self.rom.len() / 16384).max(1)) * 16384 + (addr as usize - 0x4000);
                if idx < self.rom.len() { self.rom[idx] } else { 0xFF }
            }
            0xA000..=0xBFFF => {
                if !self.ram_enabled { return 0xFF; }
                if self.ram_bank <= 3 {
                    let idx = self.ram_bank as usize * 8192 + (addr as usize - 0xA000);
                    if idx < self.ram.len() { self.ram[idx] } else { 0xFF }
                } else {
                    0 
                }
            }
            _ => 0xFF,
        }
    }
    fn mbc3_write(&mut self, addr: u16, val: u8) {
        match addr {
            0x0000..=0x1FFF => self.ram_enabled = (val & 0x0F) == 0x0A,
            0x2000..=0x3FFF => {
                let gi = (val & 0x7F) as u16;
                self.rom_bank = if gi == 0 { 1 } else { gi };
            }
            0x4000..=0x5FFF => self.ram_bank = val,
            0x6000..=0x7FFF => {} 
            0xA000..=0xBFFF => {
                if !self.ram_enabled { return; }
                if self.ram_bank <= 3 {
                    let idx = self.ram_bank as usize * 8192 + (addr as usize - 0xA000);
                    if idx < self.ram.len() { self.ram[idx] = val; }
                }
            }
            _ => {}
        }
    }

    
    fn mbc5_read(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x3FFF => {
                if (addr as usize) < self.rom.len() { self.rom[addr as usize] } else { 0xFF }
            }
            0x4000..=0x7FFF => {
                let gi = self.rom_bank as usize;
                let idx = (gi % (self.rom.len() / 16384).max(1)) * 16384 + (addr as usize - 0x4000);
                if idx < self.rom.len() { self.rom[idx] } else { 0xFF }
            }
            0xA000..=0xBFFF => {
                if !self.ram_enabled { return 0xFF; }
                let idx = (self.ram_bank as usize & 0x0F) * 8192 + (addr as usize - 0xA000);
                if idx < self.ram.len() { self.ram[idx] } else { 0xFF }
            }
            _ => 0xFF,
        }
    }
    fn mbc5_write(&mut self, addr: u16, val: u8) {
        match addr {
            0x0000..=0x1FFF => self.ram_enabled = (val & 0x0F) == 0x0A,
            0x2000..=0x2FFF => self.rom_bank = (self.rom_bank & 0x100) | val as u16,
            0x3000..=0x3FFF => self.rom_bank = (self.rom_bank & 0xFF) | (((val & 1) as u16) << 8),
            0x4000..=0x5FFF => self.ram_bank = val & 0x0F,
            0xA000..=0xBFFF => {
                if !self.ram_enabled { return; }
                let idx = (self.ram_bank as usize & 0x0F) * 8192 + (addr as usize - 0xA000);
                if idx < self.ram.len() { self.ram[idx] = val; }
            }
            _ => {}
        }
    }
}
