









use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use alloc::vec;
use alloc::string::String;
use alloc::format;
use super::ir::*;


#[derive(Debug, Clone)]
pub struct RvCpu {
    
    pub regs: [u64; 34],
    
    pub fz: u64,
    
    
    pub cpf: i64,
    pub dey: i64,
    
    pub ioo: u64,
    pub iop: u64,
    
    pub flq: u64,
    
    pub dhv: bool,
}

impl RvCpu {
    pub fn new() -> Self {
        let mut cpu = Self {
            regs: [0; 34],
            fz: 0,
            cpf: 0,
            dey: 0,
            ioo: 0,
            iop: 0,
            flq: 0,
            dhv: false,
        };
        
        cpu.regs[Reg::Ds as usize] = 0x7FFF_FFF0;
        cpu
    }

    
    #[inline]
    pub fn get(&self, reg: Reg) -> u64 {
        if reg as u8 == 0 { 0 } else { self.regs[reg as usize] }
    }

    
    #[inline]
    pub fn oj(&mut self, reg: Reg, ap: u64) {
        if reg as u8 != 0 {
            self.regs[reg as usize] = ap;
        }
    }

    
    #[inline]
    pub fn wil(&mut self, q: u64, o: u64) {
        self.cpf = q as i64;
        self.dey = o as i64;
        self.ioo = q;
        self.iop = o;
    }

    
    pub fn sno(&self, mo: FlagCond) -> bool {
        let wz = self.cpf.nj(self.dey);
        match mo {
            FlagCond::Eq    => self.cpf == self.dey,
            FlagCond::Adl    => self.cpf != self.dey,
            FlagCond::Lt    => self.cpf < self.dey,
            FlagCond::Wr    => self.cpf >= self.dey,
            FlagCond::Te    => self.cpf <= self.dey,
            FlagCond::Jn    => self.cpf > self.dey,
            FlagCond::Auz   => self.ioo < self.iop,
            FlagCond::Atb   => self.ioo >= self.iop,
            FlagCond::Neg   => wz < 0,
            FlagCond::Pos   => wz >= 0,
            FlagCond::Awn   => {
                
                (self.cpf ^ self.dey) < 0 && (self.cpf ^ wz) < 0
            }
            FlagCond::Awc => {
                !((self.cpf ^ self.dey) < 0 && (self.cpf ^ wz) < 0)
            }
        }
    }
}


pub struct RvMemory {
    
    afx: BTreeMap<u64, Vec<u8>>,
    
    pub jto: usize,
}

impl RvMemory {
    pub fn new() -> Self {
        Self {
            afx: BTreeMap::new(),
            jto: 0,
        }
    }

    
    pub fn map(&mut self, ag: u64, aw: usize) {
        self.afx.insert(ag, vec![0u8; aw]);
        self.jto += aw;
    }

    
    pub fn ujt(&mut self, ag: u64, f: &[u8]) {
        self.afx.insert(ag, f.ip());
        self.jto += f.len();
    }

    
    pub fn ady(&self, ag: u64) -> Result<u8, MemError> {
        for (&ar, f) in &self.afx {
            if ag >= ar && ag < ar + f.len() as u64 {
                return Ok(f[(ag - ar) as usize]);
            }
        }
        Err(MemError::Afg(ag))
    }

    
    pub fn alp(&self, ag: u64) -> Result<u16, MemError> {
        Ok(u16::dj([
            self.ady(ag)?,
            self.ady(ag + 1)?,
        ]))
    }

    
    pub fn za(&self, ag: u64) -> Result<u32, MemError> {
        Ok(u32::dj([
            self.ady(ag)?,
            self.ady(ag + 1)?,
            self.ady(ag + 2)?,
            self.ady(ag + 3)?,
        ]))
    }

    
    pub fn aqi(&self, ag: u64) -> Result<u64, MemError> {
        Ok(u64::dj([
            self.ady(ag)?,
            self.ady(ag + 1)?,
            self.ady(ag + 2)?,
            self.ady(ag + 3)?,
            self.ady(ag + 4)?,
            self.ady(ag + 5)?,
            self.ady(ag + 6)?,
            self.ady(ag + 7)?,
        ]))
    }

    
    pub fn cvj(&mut self, ag: u64, ap: u8) -> Result<(), MemError> {
        for (&ar, f) in self.afx.el() {
            if ag >= ar && ag < ar + f.len() as u64 {
                f[(ag - ar) as usize] = ap;
                return Ok(());
            }
        }
        Err(MemError::Afg(ag))
    }

    
    pub fn aqr(&mut self, ag: u64, ap: u16) -> Result<(), MemError> {
        let bf = ap.ho();
        self.cvj(ag, bf[0])?;
        self.cvj(ag + 1, bf[1])
    }

    
    pub fn sx(&mut self, ag: u64, ap: u32) -> Result<(), MemError> {
        let bf = ap.ho();
        for a in 0..4 {
            self.cvj(ag + a, bf[a as usize])?;
        }
        Ok(())
    }

    
    pub fn tw(&mut self, ag: u64, ap: u64) -> Result<(), MemError> {
        let bf = ap.ho();
        for a in 0..8 {
            self.cvj(ag + a, bf[a as usize])?;
        }
        Ok(())
    }

    
    pub fn zif(&self, ag: u64, cat: usize) -> Result<String, MemError> {
        let mut e = String::new();
        for a in 0..cat {
            let o = self.ady(ag + a as u64)?;
            if o == 0 { break; }
            e.push(o as char);
        }
        Ok(e)
    }

    
    pub fn qad(&mut self, ag: u64, e: &str) -> Result<(), MemError> {
        for (a, o) in e.bf().cf() {
            self.cvj(ag + a as u64, o)?;
        }
        self.cvj(ag + e.len() as u64, 0)
    }
}


#[derive(Debug)]
pub enum MemError {
    Afg(u64),
}


#[derive(Debug)]
pub enum ExecResult {
    
    Cg,
    
    Hg {
        aqb: u64,
        n: [u64; 6],
    },
    
    Bcu,
    
    Amb(u64),
    
    Hw(u64),
    
    Auj,
    
    Ceu,
}


pub struct RvInterpreter {
    
    pub cpu: RvCpu,
    
    pub mem: RvMemory,
    
    pub block_cache: BTreeMap<u64, Vec<RvInst>>,
    
    pub eff: u64,
}

impl RvInterpreter {
    pub fn new() -> Self {
        Self {
            cpu: RvCpu::new(),
            mem: RvMemory::new(),
            block_cache: BTreeMap::new(),
            eff: 10_000_000, 
        }
    }

    
    pub fn ugk(&mut self, block: &TranslatedBlock) {
        self.block_cache.insert(block.cbz, block.instructions.clone());
    }

    
    pub fn ugl(&mut self, xk: &[TranslatedBlock]) {
        for block in xk {
            self.ugk(block);
        }
    }

    
    pub fn soe(&mut self, fi: &RvInst) -> ExecResult {
        self.cpu.flq += 1;

        match fi {
            
            RvInst::Add { ck, cp, et } => {
                self.cpu.oj(*ck, self.cpu.get(*cp).cn(self.cpu.get(*et)));
            }
            RvInst::Sub { ck, cp, et } => {
                self.cpu.oj(*ck, self.cpu.get(*cp).nj(self.cpu.get(*et)));
            }
            RvInst::Ex { ck, cp, et } => {
                self.cpu.oj(*ck, self.cpu.get(*cp) & self.cpu.get(*et));
            }
            RvInst::Fx { ck, cp, et } => {
                self.cpu.oj(*ck, self.cpu.get(*cp) | self.cpu.get(*et));
            }
            RvInst::Aga { ck, cp, et } => {
                self.cpu.oj(*ck, self.cpu.get(*cp) ^ self.cpu.get(*et));
            }
            RvInst::Amt { ck, cp, et } => {
                let bcp = self.cpu.get(*et) & 63;
                self.cpu.oj(*ck, self.cpu.get(*cp) << bcp);
            }
            RvInst::Amx { ck, cp, et } => {
                let bcp = self.cpu.get(*et) & 63;
                self.cpu.oj(*ck, self.cpu.get(*cp) >> bcp);
            }
            RvInst::Azc { ck, cp, et } => {
                let bcp = self.cpu.get(*et) & 63;
                self.cpu.oj(*ck, ((self.cpu.get(*cp) as i64) >> bcp) as u64);
            }
            RvInst::Btb { ck, cp, et } => {
                let p = if (self.cpu.get(*cp) as i64) < (self.cpu.get(*et) as i64) { 1 } else { 0 };
                self.cpu.oj(*ck, p);
            }
            RvInst::Bte { ck, cp, et } => {
                let p = if self.cpu.get(*cp) < self.cpu.get(*et) { 1 } else { 0 };
                self.cpu.oj(*ck, p);
            }

            
            RvInst::Mul { ck, cp, et } => {
                self.cpu.oj(*ck, self.cpu.get(*cp).hx(self.cpu.get(*et)));
            }
            RvInst::Bms { ck, cp, et } => {
                let q = self.cpu.get(*cp) as i64 as i128;
                let o = self.cpu.get(*et) as i64 as i128;
                self.cpu.oj(*ck, ((q * o) >> 64) as u64);
            }
            RvInst::Div { ck, cp, et } => {
                let o = self.cpu.get(*et) as i64;
                if o == 0 {
                    self.cpu.oj(*ck, u64::O); 
                } else {
                    self.cpu.oj(*ck, ((self.cpu.get(*cp) as i64).zwn(o)) as u64);
                }
            }
            RvInst::Arb { ck, cp, et } => {
                let o = self.cpu.get(*et);
                if o == 0 {
                    self.cpu.oj(*ck, u64::O);
                } else {
                    self.cpu.oj(*ck, self.cpu.get(*cp) / o);
                }
            }
            RvInst::Rem { ck, cp, et } => {
                let o = self.cpu.get(*et) as i64;
                if o == 0 {
                    self.cpu.oj(*ck, self.cpu.get(*cp));
                } else {
                    self.cpu.oj(*ck, ((self.cpu.get(*cp) as i64).zwo(o)) as u64);
                }
            }
            RvInst::Bqx { ck, cp, et } => {
                let o = self.cpu.get(*et);
                if o == 0 {
                    self.cpu.oj(*ck, self.cpu.get(*cp));
                } else {
                    self.cpu.oj(*ck, self.cpu.get(*cp) % o);
                }
            }

            
            RvInst::Gf { ck, cp, gf } => {
                self.cpu.oj(*ck, self.cpu.get(*cp).cn(*gf as u64));
            }
            RvInst::Ou { ck, cp, gf } => {
                self.cpu.oj(*ck, self.cpu.get(*cp) & (*gf as u64));
            }
            RvInst::Akw { ck, cp, gf } => {
                self.cpu.oj(*ck, self.cpu.get(*cp) | (*gf as u64));
            }
            RvInst::Aoq { ck, cp, gf } => {
                self.cpu.oj(*ck, self.cpu.get(*cp) ^ (*gf as u64));
            }
            RvInst::Ayv { ck, cp, bcp } => {
                self.cpu.oj(*ck, self.cpu.get(*cp) << (*bcp & 63));
            }
            RvInst::Aze { ck, cp, bcp } => {
                self.cpu.oj(*ck, self.cpu.get(*cp) >> (*bcp & 63));
            }
            RvInst::Azd { ck, cp, bcp } => {
                self.cpu.oj(*ck, ((self.cpu.get(*cp) as i64) >> (*bcp & 63)) as u64);
            }
            RvInst::Btc { ck, cp, gf } => {
                let p = if (self.cpu.get(*cp) as i64) < *gf { 1 } else { 0 };
                self.cpu.oj(*ck, p);
            }
            RvInst::Btd { ck, cp, gf } => {
                let p = if self.cpu.get(*cp) < (*gf as u64) { 1 } else { 0 };
                self.cpu.oj(*ck, p);
            }

            
            RvInst::Blq { ck, gf } => {
                self.cpu.oj(*ck, ((*gf as u64) << 12) & 0xFFFF_FFFF_FFFF_F000);
            }
            RvInst::Bce { ck, gf } => {
                self.cpu.oj(*ck, self.cpu.fz.cn((*gf as u64) << 12));
            }

            
            RvInst::Bky { ck, cp, l } => {
                let ag = self.cpu.get(*cp).cn(*l as u64);
                match self.mem.ady(ag) {
                    Ok(p) => self.cpu.oj(*ck, p as i8 as i64 as u64),
                    Err(_) => return ExecResult::Hw(ag),
                }
            }
            RvInst::Ajr { ck, cp, l } => {
                let ag = self.cpu.get(*cp).cn(*l as u64);
                match self.mem.ady(ag) {
                    Ok(p) => self.cpu.oj(*ck, p as u64),
                    Err(_) => return ExecResult::Hw(ag),
                }
            }
            RvInst::Bla { ck, cp, l } => {
                let ag = self.cpu.get(*cp).cn(*l as u64);
                match self.mem.alp(ag) {
                    Ok(p) => self.cpu.oj(*ck, p as i16 as i64 as u64),
                    Err(_) => return ExecResult::Hw(ag),
                }
            }
            RvInst::Ajs { ck, cp, l } => {
                let ag = self.cpu.get(*cp).cn(*l as u64);
                match self.mem.alp(ag) {
                    Ok(p) => self.cpu.oj(*ck, p as u64),
                    Err(_) => return ExecResult::Hw(ag),
                }
            }
            RvInst::Blr { ck, cp, l } => {
                let ag = self.cpu.get(*cp).cn(*l as u64);
                match self.mem.za(ag) {
                    Ok(p) => self.cpu.oj(*ck, p as i32 as i64 as u64),
                    Err(_) => return ExecResult::Hw(ag),
                }
            }
            RvInst::Aka { ck, cp, l } => {
                let ag = self.cpu.get(*cp).cn(*l as u64);
                match self.mem.za(ag) {
                    Ok(p) => self.cpu.oj(*ck, p as u64),
                    Err(_) => return ExecResult::Hw(ag),
                }
            }
            RvInst::Pt { ck, cp, l } => {
                let ag = self.cpu.get(*cp).cn(*l as u64);
                match self.mem.aqi(ag) {
                    Ok(p) => self.cpu.oj(*ck, p),
                    Err(_) => return ExecResult::Hw(ag),
                }
            }

            
            RvInst::Amf { et, cp, l } => {
                let ag = self.cpu.get(*cp).cn(*l as u64);
                let ap = self.cpu.get(*et) as u8;
                if self.mem.cvj(ag, ap).is_err() {
                    return ExecResult::Hw(ag);
                }
            }
            RvInst::Amo { et, cp, l } => {
                let ag = self.cpu.get(*cp).cn(*l as u64);
                let ap = self.cpu.get(*et) as u16;
                if self.mem.aqr(ag, ap).is_err() {
                    return ExecResult::Hw(ag);
                }
            }
            RvInst::Ang { et, cp, l } => {
                let ag = self.cpu.get(*cp).cn(*l as u64);
                let ap = self.cpu.get(*et) as u32;
                if self.mem.sx(ag, ap).is_err() {
                    return ExecResult::Hw(ag);
                }
            }
            RvInst::Mi { et, cp, l } => {
                let ag = self.cpu.get(*cp).cn(*l as u64);
                let ap = self.cpu.get(*et);
                if self.mem.tw(ag, ap).is_err() {
                    return ExecResult::Hw(ag);
                }
            }

            
            RvInst::Agp { cp, et, l } => {
                if self.cpu.get(*cp) == self.cpu.get(*et) {
                    self.cpu.fz = *l as u64;
                    return ExecResult::Cg;
                }
            }
            RvInst::Ags { cp, et, l } => {
                if self.cpu.get(*cp) != self.cpu.get(*et) {
                    self.cpu.fz = *l as u64;
                    return ExecResult::Cg;
                }
            }
            RvInst::Bcr { cp, et, l } => {
                if (self.cpu.get(*cp) as i64) < (self.cpu.get(*et) as i64) {
                    self.cpu.fz = *l as u64;
                    return ExecResult::Cg;
                }
            }
            RvInst::Bcp { cp, et, l } => {
                if (self.cpu.get(*cp) as i64) >= (self.cpu.get(*et) as i64) {
                    self.cpu.fz = *l as u64;
                    return ExecResult::Cg;
                }
            }
            RvInst::Bcs { cp, et, l } => {
                if self.cpu.get(*cp) < self.cpu.get(*et) {
                    self.cpu.fz = *l as u64;
                    return ExecResult::Cg;
                }
            }
            RvInst::Bcq { cp, et, l } => {
                if self.cpu.get(*cp) >= self.cpu.get(*et) {
                    self.cpu.fz = *l as u64;
                    return ExecResult::Cg;
                }
            }

            
            RvInst::Xh { ck, l } => {
                
                self.cpu.oj(*ck, self.cpu.fz);
                self.cpu.fz = *l as u64;
                return ExecResult::Cg;
            }
            RvInst::Xi { ck, cp, l } => {
                let cd = self.cpu.get(*cp).cn(*l as u64) & !1;
                self.cpu.oj(*ck, self.cpu.fz);
                self.cpu.fz = cd;
                return ExecResult::Cg;
            }

            
            RvInst::Wk => {
                let aqb = self.cpu.get(Reg::Vd); 
                let n = [
                    self.cpu.get(Reg::Je), 
                    self.cpu.get(Reg::Zo), 
                    self.cpu.get(Reg::Afw), 
                    self.cpu.get(Reg::Va), 
                    self.cpu.get(Reg::Vb), 
                    self.cpu.get(Reg::Vc), 
                ];
                return ExecResult::Hg { aqb, n };
            }
            RvInst::Bfr => {
                return ExecResult::Bcu;
            }
            RvInst::Bgw => {
                
            }

            
            RvInst::Bbr { ck, et, cp } => {
                let ag = self.cpu.get(*cp);
                match self.mem.aqi(ag) {
                    Ok(aft) => {
                        self.cpu.oj(*ck, aft);
                        let _ = self.mem.tw(ag, self.cpu.get(*et));
                    }
                    Err(_) => return ExecResult::Hw(ag),
                }
            }
            RvInst::Bbq { ck, et, cp } => {
                let ag = self.cpu.get(*cp);
                match self.mem.aqi(ag) {
                    Ok(aft) => {
                        self.cpu.oj(*ck, aft);
                        let new = aft.cn(self.cpu.get(*et));
                        let _ = self.mem.tw(ag, new);
                    }
                    Err(_) => return ExecResult::Hw(ag),
                }
            }

            
            RvInst::Hu { ck, gf } => {
                self.cpu.oj(*ck, *gf as u64);
            }
            RvInst::Gl { ck, acl } => {
                self.cpu.oj(*ck, self.cpu.get(*acl));
            }
            RvInst::Fq => {}
            RvInst::Ama => {
                let hwl = self.cpu.get(Reg::Oq);
                if hwl == 0 {
                    
                    return ExecResult::Amb(self.cpu.get(Reg::Je));
                }
                self.cpu.fz = hwl;
                return ExecResult::Cg;
            }
            RvInst::En { l } => {
                self.cpu.oj(Reg::Oq, self.cpu.fz);
                self.cpu.fz = *l as u64;
                return ExecResult::Cg;
            }

            
            RvInst::Ed { cp, et } => {
                self.cpu.wil(self.cpu.get(*cp), self.cpu.get(*et));
            }
            RvInst::Aad { mo, l } => {
                if self.cpu.sno(*mo) {
                    self.cpu.fz = *l as u64;
                    return ExecResult::Cg;
                }
            }
            RvInst::Od { .. } => {
                
            }
        }

        ExecResult::Cg
    }

    
    pub fn nrj(&mut self, instructions: &[RvInst]) -> ExecResult {
        let mut ip = 0;
        while ip < instructions.len() {
            if self.cpu.flq >= self.eff {
                return ExecResult::Auj;
            }

            let result = self.soe(&instructions[ip]);
            ip += 1;

            match result {
                ExecResult::Cg => {
                    
                    
                    
                }
                gq => return gq,
            }
        }

        ExecResult::Cg
    }

    
    pub fn zkm(&mut self, wsn: u64) -> ExecResult {
        self.cpu.fz = wsn;

        loop {
            if self.cpu.flq >= self.eff {
                return ExecResult::Auj;
            }

            
            if let Some(kea) = self.block_cache.get(&self.cpu.fz).abn() {
                let uxs = self.cpu.fz;
                let result = self.nrj(&kea);

                match result {
                    ExecResult::Cg => {
                        
                        if self.cpu.fz == uxs {
                            
                            return ExecResult::Amb(self.cpu.get(Reg::Je));
                        }
                        
                    }
                    gq => return gq,
                }
            } else {
                
                return ExecResult::Amb(self.cpu.get(Reg::Je));
            }
        }
    }

    
    pub fn noi(&self) -> String {
        let mut e = String::from("=== RISC-V IR CPU State ===\n");
        for a in 0..32 {
            let reg = Reg::ivy(a);
            let ap = self.cpu.get(reg);
            if ap != 0 {
                e.t(&format!("  {:4} (x{:2}) = 0x{:016X} ({})\n",
                    reg.kj(), a, ap, ap as i64));
            }
        }
        e.t(&format!("  pc = 0x{:016X}\n", self.cpu.fz));
        e.t(&format!("  instructions executed: {}\n", self.cpu.flq));
        e
    }
}
