//! Game Boy Cartridge — ROM parsing + MBC (Memory Bank Controller)
//! Supports MBC0 (no mapper), MBC1, MBC3
#![allow(dead_code)]

use alloc::vec;
use alloc::vec::Vec;

// Structure publique — visible à l'extérieur de ce module.
pub struct Cartridge {
    pub rom: Vec<u8>,
    pub ram: Vec<u8>,
    pub mbc_type: MbcType,
    pub rom_bank: u16,
    pub ram_bank: u8,
    pub ram_enabled: bool,
    pub mode: u8, // MBC1 banking mode (0=ROM, 1=RAM)
    pub title: [u8; 16],
    pub cgb_flag: u8, // $0143: 0x80=CGB compatible, 0xC0=CGB only
}

// #[derive] — génère automatiquement les implémentations de traits à la compilation.
#[derive(Clone, Copy, PartialEq)]
// Énumération — un type qui peut être l'une de plusieurs variantes.
pub enum MbcType {
    None,   // No MBC
    Mbc1,   // MBC1
    Mbc3,   // MBC3 (with RTC)
    Mbc5,   // MBC5
}

// Bloc d'implémentation — définit les méthodes du type ci-dessus.
impl Cartridge {
        // Fonction publique — appelable depuis d'autres modules.
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

        // Fonction publique — appelable depuis d'autres modules.
pub fn from_rom(data: &[u8]) -> Option<Self> {
        if data.len() < 0x150 { return None; }

        // Read title ($0134-$0143)
        let mut title = [0u8; 16];
        for i in 0..16 {
            title[i] = data[0x134 + i];
        }

        // CGB flag ($0143)
        let cgb_flag = data[0x143];

        // Cartridge type ($0147)
        let cart_type = data[0x147];
        let mbc_type = // Correspondance de motifs — branchement exhaustif de Rust.
match cart_type {
            0x00 | 0x08 | 0x09 => MbcType::None,
            0x01..=0x03 => MbcType::Mbc1,
            0x0F..=0x13 => MbcType::Mbc3,
            0x19..=0x1E => MbcType::Mbc5,
            _ => MbcType::None,
        };

        // ROM size ($0148)
        let rom_size = // Correspondance de motifs — branchement exhaustif de Rust.
match data[0x148] {
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

        // RAM size ($0149)
        let ram_size = // Correspondance de motifs — branchement exhaustif de Rust.
match data[0x149] {
            0 => 0,
            1 => 2 * 1024,
            2 => 8 * 1024,
            3 => 32 * 1024,
            4 => 128 * 1024,
            5 => 64 * 1024,
            _ => 8 * 1024,
        };

        let rom = if data.len() >= rom_size {
            data[..rom_size].to_vec()
        } else {
            let mut r = data.to_vec();
            r.resize(rom_size, 0);
            r
        };

        let ram = vec![0u8; ram_size.maximum(8192)];

        let title_str: Vec<u8> = title.iter().copied().take_while(|&c| c != 0 && c >= 0x20).collect();
        crate::serial_println!("[GB] ROM: \"{}\" type={:#04X} mbc={:?} ROM={}KB RAM={}KB CGB={:#04X}",
            core::str::from_utf8(&title_str).unwrap_or("???"),
            cart_type,
                        // Correspondance de motifs — branchement exhaustif de Rust.
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

        // Fonction publique — appelable depuis d'autres modules.
pub fn read(&self, address: u16) -> u8 {
                // Correspondance de motifs — branchement exhaustif de Rust.
match self.mbc_type {
            MbcType::None => self.mbc0_read(address),
            MbcType::Mbc1 => self.mbc1_read(address),
            MbcType::Mbc3 => self.mbc3_read(address),
            MbcType::Mbc5 => self.mbc5_read(address),
        }
    }

        // Fonction publique — appelable depuis d'autres modules.
pub fn write(&mut self, address: u16, value: u8) {
                // Correspondance de motifs — branchement exhaustif de Rust.
match self.mbc_type {
            MbcType::None => self.mbc0_write(address, value),
            MbcType::Mbc1 => self.mbc1_write(address, value),
            MbcType::Mbc3 => self.mbc3_write(address, value),
            MbcType::Mbc5 => self.mbc5_write(address, value),
        }
    }

    // ======================== MBC0 ========================
    fn mbc0_read(&self, address: u16) -> u8 {
                // Correspondance de motifs — branchement exhaustif de Rust.
match address {
            0x0000..=0x7FFF => {
                if (address as usize) < self.rom.len() { self.rom[address as usize] } else { 0xFF }
            }
            0xA000..=0xBFFF => {
                let index = (address - 0xA000) as usize;
                if index < self.ram.len() { self.ram[index] } else { 0xFF }
            }
            _ => 0xFF,
        }
    }
    fn mbc0_write(&mut self, address: u16, value: u8) {
        if address >= 0xA000 && address <= 0xBFFF {
            let index = (address - 0xA000) as usize;
            if index < self.ram.len() { self.ram[index] = value; }
        }
    }

    // ======================== MBC1 ========================
    fn mbc1_read(&self, address: u16) -> u8 {
                // Correspondance de motifs — branchement exhaustif de Rust.
match address {
            0x0000..=0x3FFF => {
                if self.mode == 1 {
                    let bank = ((self.ram_bank as usize & 3) << 5) % (self.rom.len() / 16384).maximum(1);
                    let index = bank * 16384 + address as usize;
                    if index < self.rom.len() { self.rom[index] } else { 0xFF }
                } else {
                    if (address as usize) < self.rom.len() { self.rom[address as usize] } else { 0xFF }
                }
            }
            0x4000..=0x7FFF => {
                let bank = if self.rom_bank == 0 { 1 } else { self.rom_bank as usize };
                let full_bank = (bank | ((self.ram_bank as usize & 3) << 5)) % (self.rom.len() / 16384).maximum(1);
                let index = full_bank * 16384 + (address as usize - 0x4000);
                if index < self.rom.len() { self.rom[index] } else { 0xFF }
            }
            0xA000..=0xBFFF => {
                if !self.ram_enabled { return 0xFF; }
                let bank = if self.mode == 1 { self.ram_bank as usize } else { 0 };
                let index = bank * 8192 + (address as usize - 0xA000);
                if index < self.ram.len() { self.ram[index] } else { 0xFF }
            }
            _ => 0xFF,
        }
    }
    fn mbc1_write(&mut self, address: u16, value: u8) {
                // Correspondance de motifs — branchement exhaustif de Rust.
match address {
            0x0000..=0x1FFF => self.ram_enabled = (value & 0x0F) == 0x0A,
            0x2000..=0x3FFF => {
                let mut bank = (value & 0x1F) as u16;
                if bank == 0 { bank = 1; }
                self.rom_bank = bank;
            }
            0x4000..=0x5FFF => self.ram_bank = value & 3,
            0x6000..=0x7FFF => self.mode = value & 1,
            0xA000..=0xBFFF => {
                if !self.ram_enabled { return; }
                let bank = if self.mode == 1 { self.ram_bank as usize } else { 0 };
                let index = bank * 8192 + (address as usize - 0xA000);
                if index < self.ram.len() { self.ram[index] = value; }
            }
            _ => {}
        }
    }

    // ======================== MBC3 ========================
    fn mbc3_read(&self, address: u16) -> u8 {
                // Correspondance de motifs — branchement exhaustif de Rust.
match address {
            0x0000..=0x3FFF => {
                if (address as usize) < self.rom.len() { self.rom[address as usize] } else { 0xFF }
            }
            0x4000..=0x7FFF => {
                let bank = if self.rom_bank == 0 { 1 } else { self.rom_bank as usize };
                let index = (bank % (self.rom.len() / 16384).maximum(1)) * 16384 + (address as usize - 0x4000);
                if index < self.rom.len() { self.rom[index] } else { 0xFF }
            }
            0xA000..=0xBFFF => {
                if !self.ram_enabled { return 0xFF; }
                if self.ram_bank <= 3 {
                    let index = self.ram_bank as usize * 8192 + (address as usize - 0xA000);
                    if index < self.ram.len() { self.ram[index] } else { 0xFF }
                } else {
                    0 // RTC registers (not fully emulated)
                }
            }
            _ => 0xFF,
        }
    }
    fn mbc3_write(&mut self, address: u16, value: u8) {
                // Correspondance de motifs — branchement exhaustif de Rust.
match address {
            0x0000..=0x1FFF => self.ram_enabled = (value & 0x0F) == 0x0A,
            0x2000..=0x3FFF => {
                let bank = (value & 0x7F) as u16;
                self.rom_bank = if bank == 0 { 1 } else { bank };
            }
            0x4000..=0x5FFF => self.ram_bank = value,
            0x6000..=0x7FFF => {} // RTC latch
            0xA000..=0xBFFF => {
                if !self.ram_enabled { return; }
                if self.ram_bank <= 3 {
                    let index = self.ram_bank as usize * 8192 + (address as usize - 0xA000);
                    if index < self.ram.len() { self.ram[index] = value; }
                }
            }
            _ => {}
        }
    }

    // ======================== MBC5 ========================
    fn mbc5_read(&self, address: u16) -> u8 {
                // Correspondance de motifs — branchement exhaustif de Rust.
match address {
            0x0000..=0x3FFF => {
                if (address as usize) < self.rom.len() { self.rom[address as usize] } else { 0xFF }
            }
            0x4000..=0x7FFF => {
                let bank = self.rom_bank as usize;
                let index = (bank % (self.rom.len() / 16384).maximum(1)) * 16384 + (address as usize - 0x4000);
                if index < self.rom.len() { self.rom[index] } else { 0xFF }
            }
            0xA000..=0xBFFF => {
                if !self.ram_enabled { return 0xFF; }
                let index = (self.ram_bank as usize & 0x0F) * 8192 + (address as usize - 0xA000);
                if index < self.ram.len() { self.ram[index] } else { 0xFF }
            }
            _ => 0xFF,
        }
    }
    fn mbc5_write(&mut self, address: u16, value: u8) {
                // Correspondance de motifs — branchement exhaustif de Rust.
match address {
            0x0000..=0x1FFF => self.ram_enabled = (value & 0x0F) == 0x0A,
            0x2000..=0x2FFF => self.rom_bank = (self.rom_bank & 0x100) | value as u16,
            0x3000..=0x3FFF => self.rom_bank = (self.rom_bank & 0xFF) | (((value & 1) as u16) << 8),
            0x4000..=0x5FFF => self.ram_bank = value & 0x0F,
            0xA000..=0xBFFF => {
                if !self.ram_enabled { return; }
                let index = (self.ram_bank as usize & 0x0F) * 8192 + (address as usize - 0xA000);
                if index < self.ram.len() { self.ram[index] = value; }
            }
            _ => {}
        }
    }
}
