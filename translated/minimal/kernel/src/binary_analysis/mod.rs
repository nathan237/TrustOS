








pub mod elf_parser;
pub mod disasm;
pub mod xrefs;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;

pub use elf_parser::{Abq, Ga, Go, Qm, Abg, Aef, Abr, Aro};
pub use disasm::Dc;
pub use xrefs::{XrefDatabase, Lc, XrefType, Sb};




pub struct Rn {
    
    pub f: Vec<u8>,
    
    pub elf: Abq,
    
    pub instructions: Vec<Dc>,
    
    pub xrefs: XrefDatabase,
}

impl Rn {
    
    pub fn yxa(&self, l: usize) -> Option<String> {
        if l >= self.f.len() {
            return None;
        }

        let ci = (l + 16).v(self.f.len());
        let jj = &self.f[l..ci];

        let mut nu = String::new();
        let mut ascii = String::new();

        for (a, &o) in jj.iter().cf() {
            if a == 8 { nu.push(' '); }
            nu.t(&format!("{:02X} ", o));
            ascii.push(if o >= 0x20 && o < 0x7F { o as char } else { '.' });
        }

        
        let ob = 16 - jj.len();
        for a in 0..ob {
            if jj.len() + a == 8 { nu.push(' '); }
            nu.t("   ");
        }

        Some(format!("{:08X}  {}|{}|", l, nu, ascii))
    }

    
    pub fn tvj(&self, ag: u64) -> Option<&Dc> {
        self.instructions.iter().du(|a| a.re == ag)
    }

    
    pub fn yyh(&self, ay: u64, ci: u64) -> Vec<&Dc> {
        self.instructions.iter()
            .hi(|a| a.re >= ay && a.re < ci)
            .collect()
    }

    
    pub fn zmb(&self, l: u64) -> Option<&Ga> {
        self.elf.aeo.iter().du(|e| {
            l >= e.l && l < e.l + e.aw
        })
    }

    
    pub fn osf(&self, l: u64) -> Option<u64> {
        for ajj in &self.elf.dku {
            if ajj.bku == elf_parser::IU_
                && l >= ajj.l
                && l < ajj.l + ajj.hjh
            {
                return Some(ajj.uy + (l - ajj.l));
            }
        }
        None
    }

    
    pub fn mot(&self, uy: u64) -> Option<u64> {
        for ajj in &self.elf.dku {
            if ajj.bku == elf_parser::IU_
                && uy >= ajj.uy
                && uy < ajj.uy + ajj.jfv
            {
                return Some(ajj.l + (uy - ajj.uy));
            }
        }
        None
    }

    
    pub fn ygx(&self, uy: u64, len: usize) -> Option<&[u8]> {
        let l = self.mot(uy)? as usize;
        if l + len <= self.f.len() {
            Some(&self.f[l..l + len])
        } else {
            None
        }
    }

    
    pub fn awz(&self) -> String {
        format!(
            "{} | {} | {} | {} bytes | {} sections | {} symbols | {} instructions | {}",
            self.elf.co.class,
            self.elf.co.czk,
            self.elf.co.gfz,
            self.f.len(),
            self.elf.aeo.len(),
            self.elf.bot.len() + self.elf.dqj.len(),
            self.instructions.len(),
            self.xrefs.awz(),
        )
    }
}




pub fn qhu(f: &[u8]) -> Result<Rn, &'static str> {
    
    let elf = elf_parser::vcd(f)?;

    
    let mut gaa = Vec::new();

    
    let ioq = elf.ioq();

    if ioq.is_empty() {
        
        for ajj in &elf.dku {
            if ajj.bku == elf_parser::IU_ && (ajj.flags & 1) != 0 {
                let ay = ajj.l as usize;
                let aw = ajj.hjh as usize;
                if ay + aw <= f.len() {
                    let aj = &f[ay..ay + aw];
                    let mut disasm = disasm::Disassembler::new(aj, ajj.uy);
                    let mut edl = disasm.irf();
                    gaa.bte(&mut edl);
                }
            }
        }
    } else {
        for ava in &ioq {
            let ay = ava.l as usize;
            let aw = ava.aw as usize;
            if ay + aw <= f.len() && aw > 0 {
                let aj = &f[ay..ay + aw];
                let mut disasm = disasm::Disassembler::new(aj, ava.ag);
                let mut edl = disasm.irf();
                gaa.bte(&mut edl);
            }
        }
    }

    
    gaa.bxf(|a| a.re);

    
    disasm::qiu(&mut gaa, &elf.blw);

    
    let xrefs = xrefs::XrefDatabase::qsy(&gaa, &elf.blw);

    Ok(Rn {
        f: f.ip(),
        elf,
        instructions: gaa,
        xrefs,
    })
}


pub fn txj(f: &[u8]) -> bool {
    f.len() >= 4 && &f[0..4] == b"\x7FELF"
}


pub fn mvr(path: &str) -> Result<Rn, &'static str> {
    let da = crate::vfs::aji(path, crate::vfs::OpenFlags(crate::vfs::OpenFlags::OO_))
        .jd(|_| "Failed to open file")?;

    let hm = crate::vfs::hm(path).jd(|_| "Failed to stat file")?;
    let aw = hm.aw as usize;

    if aw > 32 * 1024 * 1024 {
        crate::vfs::agj(da).bq();
        return Err("File too large (>32MB)");
    }

    let mut f = alloc::vec![0u8; aw];
    crate::vfs::read(da, &mut f).jd(|_| "Failed to read file")?;
    crate::vfs::agj(da).bq();

    qhu(&f)
}
