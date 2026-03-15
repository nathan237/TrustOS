//! NES Cartridge — iNES ROM parsing + Mapper implementations
//! Supports Mapper 0 (NROM), Mapper 1 (MMC1), Mapper 2 (UxROM), Mapper 3 (CNROM)
#![allow(dead_code)]

use alloc::vec;
use alloc::vec::Vec;

const INES_MAGIC: [u8; 4] = [0x4E, 0x45, 0x53, 0x1A];

#[derive(Clone, Copy, PartialEq)]
pub enum Mirror {
    Horizontal,
    Vertical,
    Single0,
    Single1,
    FourScreen,
}

pub struct Cartridge {
    pub prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>,
    pub chr_ram: bool,
    pub prg_ram: [u8; 8192],
    pub mapper_id: u8,
    pub mirror: Mirror,
    // Mapper 1 (MMC1) state
    m1_shift: u8,
    m1_shift_count: u8,
    m1_control: u8,
    m1_chr_bank0: u8,
    m1_chr_bank1: u8,
    m1_prg_bank: u8,
    // Mapper 2/3 state
    m2_prg_bank: u8,
    m3_chr_bank: u8,
}

impl Cartridge {
    pub fn empty() -> Self {
        Self {
            prg_rom: vec![0u8; 32768],
            chr_rom: vec![0u8; 8192],
            chr_ram: true,
            prg_ram: [0; 8192],
            mapper_id: 0,
            mirror: Mirror::Horizontal,
            m1_shift: 0x10,
            m1_shift_count: 0,
            m1_control: 0x0C,
            m1_chr_bank0: 0,
            m1_chr_bank1: 0,
            m1_prg_bank: 0,
            m2_prg_bank: 0,
            m3_chr_bank: 0,
        }
    }

    pub fn from_ines(data: &[u8]) -> Option<Self> {
        if data.len() < 16 { return None; }
        if data[0..4] != INES_MAGIC { return None; }

        let prg_banks = data[4] as usize;
        let chr_banks = data[5] as usize;
        let flags6 = data[6];
        let flags7 = data[7];
        let mapper_id = (flags7 & 0xF0) | (flags6 >> 4);
        let mirror = if flags6 & 0x08 != 0 {
            Mirror::FourScreen
        } else if flags6 & 0x01 != 0 {
            Mirror::Vertical
        } else {
            Mirror::Horizontal
        };
        let has_trainer = flags6 & 0x04 != 0;
        let offset = 16 + if has_trainer { 512 } else { 0 };
        let prg_size = prg_banks * 16384;
        let chr_size = chr_banks * 8192;

        if data.len() < offset + prg_size + chr_size { return None; }

        let prg_rom = data[offset..offset + prg_size].to_vec();
        let (chr_rom, chr_ram) = if chr_size > 0 {
            (data[offset + prg_size..offset + prg_size + chr_size].to_vec(), false)
        } else {
            (vec![0u8; 8192], true)
        };

        crate::serial_println!("[NES] ROM: mapper={} PRG={}KB CHR={}KB mirror={:?}",
            mapper_id, prg_size / 1024, chr_rom.len() / 1024,
            if mirror == Mirror::Vertical { "V" } else { "H" });

        Some(Self {
            prg_rom,
            chr_rom,
            chr_ram,
            prg_ram: [0; 8192],
            mapper_id,
            mirror,
            m1_shift: 0x10,
            m1_shift_count: 0,
            m1_control: 0x0C,
            m1_chr_bank0: 0,
            m1_chr_bank1: 0,
            m1_prg_bank: 0,
            m2_prg_bank: 0,
            m3_chr_bank: 0,
        })
    }

    // ======================== CPU Read ($6000-$FFFF) ========================

    pub fn cpu_read(&self, addr: u16) -> u8 {
        match self.mapper_id {
            0 => self.mapper0_cpu_read(addr),
            1 => self.mapper1_cpu_read(addr),
            2 => self.mapper2_cpu_read(addr),
            3 => self.mapper3_cpu_read(addr),
            _ => self.mapper0_cpu_read(addr),
        }
    }

    pub fn cpu_write(&mut self, addr: u16, val: u8) {
        match self.mapper_id {
            0 => {} // NROM is read-only
            1 => self.mapper1_cpu_write(addr, val),
            2 => self.mapper2_cpu_write(addr, val),
            3 => self.mapper3_cpu_write(addr, val),
            _ => {}
        }
        // PRG RAM ($6000-$7FFF)
        if addr >= 0x6000 && addr < 0x8000 {
            self.prg_ram[(addr - 0x6000) as usize] = val;
        }
    }

    // ======================== PPU Read ($0000-$1FFF) ========================

    pub fn ppu_read(&self, addr: u16) -> u8 {
        match self.mapper_id {
            3 => {
                let bank_offset = (self.m3_chr_bank as usize) * 8192;
                let idx = bank_offset + (addr as usize & 0x1FFF);
                if idx < self.chr_rom.len() { self.chr_rom[idx] } else { 0 }
            }
            1 => self.mapper1_ppu_read(addr),
            _ => {
                let idx = addr as usize & (self.chr_rom.len() - 1).max(0x1FFF);
                if idx < self.chr_rom.len() { self.chr_rom[idx] } else { 0 }
            }
        }
    }

    pub fn ppu_write(&mut self, addr: u16, val: u8) {
        if self.chr_ram {
            let idx = addr as usize & 0x1FFF;
            if idx < self.chr_rom.len() {
                self.chr_rom[idx] = val;
            }
        }
    }

    pub fn mirror_nametable(&self, addr: u16) -> u16 {
        let addr = addr & 0x0FFF;
        match self.mirror {
            Mirror::Horizontal => {
                // $2000=$2400, $2800=$2C00
                let table = (addr >> 11) & 1;
                (table * 0x400) | (addr & 0x03FF)
            }
            Mirror::Vertical => {
                // $2000=$2800, $2400=$2C00
                addr & 0x07FF
            }
            Mirror::Single0 => addr & 0x03FF,
            Mirror::Single1 => 0x400 | (addr & 0x03FF),
            Mirror::FourScreen => addr & 0x0FFF,
        }
    }

    // ======================== Mapper 0 (NROM) ========================

    fn mapper0_cpu_read(&self, addr: u16) -> u8 {
        match addr {
            0x6000..=0x7FFF => self.prg_ram[(addr - 0x6000) as usize],
            0x8000..=0xFFFF => {
                let idx = (addr - 0x8000) as usize;
                if self.prg_rom.len() <= 16384 {
                    self.prg_rom[idx & 0x3FFF] // 16KB mirrored
                } else {
                    self.prg_rom[idx & (self.prg_rom.len() - 1)]
                }
            }
            _ => 0,
        }
    }

    // ======================== Mapper 1 (MMC1) ========================

    fn mapper1_cpu_read(&self, addr: u16) -> u8 {
        match addr {
            0x6000..=0x7FFF => self.prg_ram[(addr - 0x6000) as usize],
            0x8000..=0xFFFF => {
                let prg_mode = (self.m1_control >> 2) & 3;
                let bank = self.m1_prg_bank as usize & 0x0F;
                let prg_banks = self.prg_rom.len() / 16384;
                match prg_mode {
                    0 | 1 => {
                        // 32KB mode: ignore low bit
                        let base = (bank & !1) * 16384;
                        let idx = base + (addr as usize - 0x8000);
                        self.prg_rom[idx % self.prg_rom.len()]
                    }
                    2 => {
                        // Fix first, switch second
                        if addr < 0xC000 {
                            self.prg_rom[(addr as usize - 0x8000) % self.prg_rom.len()]
                        } else {
                            let base = bank * 16384;
                            self.prg_rom[(base + (addr as usize - 0xC000)) % self.prg_rom.len()]
                        }
                    }
                    _ => {
                        // Switch first, fix last
                        if addr < 0xC000 {
                            let base = bank * 16384;
                            self.prg_rom[(base + (addr as usize - 0x8000)) % self.prg_rom.len()]
                        } else {
                            let base = (prg_banks - 1) * 16384;
                            self.prg_rom[(base + (addr as usize - 0xC000)) % self.prg_rom.len()]
                        }
                    }
                }
            }
            _ => 0,
        }
    }

    fn mapper1_cpu_write(&mut self, addr: u16, val: u8) {
        if addr < 0x8000 { return; }

        if val & 0x80 != 0 {
            self.m1_shift = 0x10;
            self.m1_shift_count = 0;
            self.m1_control |= 0x0C;
            return;
        }

        self.m1_shift = (self.m1_shift >> 1) | ((val & 1) << 4);
        self.m1_shift_count += 1;

        if self.m1_shift_count == 5 {
            let value = self.m1_shift;
            match addr {
                0x8000..=0x9FFF => {
                    self.m1_control = value;
                    self.mirror = match value & 3 {
                        0 => Mirror::Single0,
                        1 => Mirror::Single1,
                        2 => Mirror::Vertical,
                        _ => Mirror::Horizontal,
                    };
                }
                0xA000..=0xBFFF => self.m1_chr_bank0 = value,
                0xC000..=0xDFFF => self.m1_chr_bank1 = value,
                0xE000..=0xFFFF => self.m1_prg_bank = value & 0x0F,
                _ => {}
            }
            self.m1_shift = 0x10;
            self.m1_shift_count = 0;
        }
    }

    fn mapper1_ppu_read(&self, addr: u16) -> u8 {
        let chr_mode = (self.m1_control >> 4) & 1;
        let idx = if chr_mode == 0 {
            // 8KB mode
            let bank = (self.m1_chr_bank0 as usize & !1) * 4096;
            bank + (addr as usize & 0x1FFF)
        } else {
            // 4KB mode
            if addr < 0x1000 {
                (self.m1_chr_bank0 as usize) * 4096 + (addr as usize & 0x0FFF)
            } else {
                (self.m1_chr_bank1 as usize) * 4096 + (addr as usize & 0x0FFF)
            }
        };
        if idx < self.chr_rom.len() { self.chr_rom[idx] } else { 0 }
    }

    // ======================== Mapper 2 (UxROM) ========================

    fn mapper2_cpu_read(&self, addr: u16) -> u8 {
        match addr {
            0x6000..=0x7FFF => self.prg_ram[(addr - 0x6000) as usize],
            0x8000..=0xBFFF => {
                let base = (self.m2_prg_bank as usize) * 16384;
                self.prg_rom[(base + (addr as usize - 0x8000)) % self.prg_rom.len()]
            }
            0xC000..=0xFFFF => {
                let last_bank = (self.prg_rom.len() / 16384).saturating_sub(1);
                let base = last_bank * 16384;
                self.prg_rom[(base + (addr as usize - 0xC000)) % self.prg_rom.len()]
            }
            _ => 0,
        }
    }

    fn mapper2_cpu_write(&mut self, addr: u16, val: u8) {
        if addr >= 0x8000 {
            self.m2_prg_bank = val;
        }
    }

    // ======================== Mapper 3 (CNROM) ========================

    fn mapper3_cpu_read(&self, addr: u16) -> u8 {
        self.mapper0_cpu_read(addr) // Same PRG mapping as NROM
    }

    fn mapper3_cpu_write(&mut self, addr: u16, val: u8) {
        if addr >= 0x8000 {
            self.m3_chr_bank = val & 0x03;
        }
    }
}
