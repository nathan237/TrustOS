




use alloc::vec::Vec;
use alloc::string::String;
use alloc::collections::BTreeMap;


#[derive(Debug, Clone)]
pub struct CpuState {
    
    pub rax: u64,
    pub rbx: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub rbp: u64,
    pub rsp: u64,
    pub r8: u64,
    pub r9: u64,
    pub r10: u64,
    pub r11: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
    
    
    pub pc: u64,
    
    
    pub rflags: u64,
    
    
    pub aap: u16,
    pub bjw: u16,
    pub cqf: u16,
    pub fs: u64,  
    pub ckx: u64,  
    pub rv: u16,
}

impl CpuState {
    pub fn new() -> Self {
        Self {
            rax: 0, rbx: 0, rcx: 0, rdx: 0,
            rsi: 0, rdi: 0, rbp: 0, rsp: 0,
            r8: 0, r9: 0, r10: 0, r11: 0,
            r12: 0, r13: 0, r14: 0, r15: 0,
            pc: 0,
            rflags: 0x202, 
            aap: 0x33, bjw: 0x2b, cqf: 0x2b,
            fs: 0, ckx: 0, rv: 0x2b,
        }
    }
    
    
    pub fn bzu(&self, w: u8) -> u64 {
        match w {
            0 => self.rax,
            1 => self.rcx,
            2 => self.rdx,
            3 => self.rbx,
            4 => self.rsp,
            5 => self.rbp,
            6 => self.rsi,
            7 => self.rdi,
            8 => self.r8,
            9 => self.r9,
            10 => self.r10,
            11 => self.r11,
            12 => self.r12,
            13 => self.r13,
            14 => self.r14,
            15 => self.r15,
            _ => 0,
        }
    }
    
    
    pub fn bxa(&mut self, w: u8, ap: u64) {
        match w {
            0 => self.rax = ap,
            1 => self.rcx = ap,
            2 => self.rdx = ap,
            3 => self.rbx = ap,
            4 => self.rsp = ap,
            5 => self.rbp = ap,
            6 => self.rsi = ap,
            7 => self.rdi = ap,
            8 => self.r8 = ap,
            9 => self.r9 = ap,
            10 => self.r10 = ap,
            11 => self.r11 = ap,
            12 => self.r12 = ap,
            13 => self.r13 = ap,
            14 => self.r14 = ap,
            15 => self.r15 = ap,
            _ => {}
        }
    }
}


pub const ASC_: u64 = 1 << 0;  
pub const ACF_: u64 = 1 << 2;  
pub const DLY_: u64 = 1 << 4;  
pub const ACH_: u64 = 1 << 6;  
pub const ACG_: u64 = 1 << 7;  
pub const ASE_: u64 = 1 << 11; 


pub struct ProcessMemory {
    
    afx: BTreeMap<u64, MemoryRegion>,
    
    den: u64,
}

struct MemoryRegion {
    f: Vec<u8>,
    bob: bool,
    bjb: bool,
    kuk: bool,
}

impl ProcessMemory {
    pub fn new() -> Self {
        Self {
            afx: BTreeMap::new(),
            den: 0x1000_0000, 
        }
    }
    
    
    pub fn map(&mut self, ag: u64, aw: usize, m: bool, d: bool, b: bool) {
        self.afx.insert(ag, MemoryRegion {
            f: alloc::vec![0u8; aw],
            bob: m,
            bjb: d,
            kuk: b,
        });
    }
    
    
    pub fn write(&mut self, ag: u64, f: &[u8]) -> Result<(), &'static str> {
        for (fso, aoz) in self.afx.el() {
            let exn = *fso + aoz.f.len() as u64;
            if ag >= *fso && ag < exn {
                if !aoz.bjb {
                    return Err("Write to non-writable memory");
                }
                let l = (ag - *fso) as usize;
                let zg = core::cmp::v(f.len(), aoz.f.len() - l);
                aoz.f[l..l + zg].dg(&f[..zg]);
                return Ok(());
            }
        }
        Err("Write to unmapped memory")
    }
    
    
    pub fn read(&self, ag: u64, len: usize) -> Result<Vec<u8>, &'static str> {
        for (fso, aoz) in self.afx.iter() {
            let exn = *fso + aoz.f.len() as u64;
            if ag >= *fso && ag < exn {
                if !aoz.bob {
                    return Err("Read from non-readable memory");
                }
                let l = (ag - *fso) as usize;
                let zg = core::cmp::v(len, aoz.f.len() - l);
                return Ok(aoz.f[l..l + zg].ip());
            }
        }
        Err("Read from unmapped memory")
    }
    
    
    pub fn ady(&self, ag: u64) -> Result<u8, &'static str> {
        let f = self.read(ag, 1)?;
        Ok(f[0])
    }
    
    
    pub fn alp(&self, ag: u64) -> Result<u16, &'static str> {
        let f = self.read(ag, 2)?;
        Ok(u16::dj([f[0], f[1]]))
    }
    
    
    pub fn za(&self, ag: u64) -> Result<u32, &'static str> {
        let f = self.read(ag, 4)?;
        Ok(u32::dj([f[0], f[1], f[2], f[3]]))
    }
    
    
    pub fn aqi(&self, ag: u64) -> Result<u64, &'static str> {
        let f = self.read(ag, 8)?;
        Ok(u64::dj([
            f[0], f[1], f[2], f[3],
            f[4], f[5], f[6], f[7],
        ]))
    }
    
    
    pub fn cvj(&mut self, ag: u64, ap: u8) -> Result<(), &'static str> {
        self.write(ag, &[ap])
    }
    
    
    pub fn tw(&mut self, ag: u64, ap: u64) -> Result<(), &'static str> {
        self.write(ag, &ap.ho())
    }
    
    
    pub fn den(&self) -> u64 { self.den }
    pub fn pip(&mut self, usn: u64) { self.den = usn; }
}


pub enum DecodeResult {
    
    Cg,
    
    Hg,
    
    Lk(i32),
    
    Q(&'static str),
}


pub struct Interpreter {
    pub cpu: CpuState,
    pub memory: ProcessMemory,
    
    pub aho: BTreeMap<i32, FileDescriptor>,
    
    bca: i32,
    
    pub ce: u32,
    
    pub jv: String,
    
    pub cjc: Vec<String>,
    
    pub epy: Vec<String>,
}

pub enum FileDescriptor {
    Btl,
    Btm,
    Btk,
    Es { path: String, qf: u64 },
    Yc { bi: Vec<u8> },
}

impl Interpreter {
    pub fn new() -> Self {
        let mut aho = BTreeMap::new();
        aho.insert(0, FileDescriptor::Btl);
        aho.insert(1, FileDescriptor::Btm);
        aho.insert(2, FileDescriptor::Btk);
        
        Self {
            cpu: CpuState::new(),
            memory: ProcessMemory::new(),
            aho,
            bca: 3,
            ce: 1,
            jv: String::from("/"),
            cjc: Vec::new(),
            epy: Vec::new(),
        }
    }
    
    
    pub fn ugu(&mut self, pu: &[u8]) -> Result<u64, &'static str> {
        
        if pu.len() < 64 {
            return Err("ELF too small");
        }
        if &pu[0..4] != b"\x7fELF" {
            return Err("Not an ELF file");
        }
        if pu[4] != 2 {
            return Err("Not 64-bit ELF");
        }
        if pu[5] != 1 {
            return Err("Not little-endian");
        }
        
        
        let ceh = u16::dj([pu[16], pu[17]]);
        if ceh != 2 && ceh != 3 {
            return Err("Not executable or shared object");
        }
        
        let cxe = u64::dj([
            pu[24], pu[25], pu[26], pu[27],
            pu[28], pu[29], pu[30], pu[31],
        ]);
        
        let epo = u64::dj([
            pu[32], pu[33], pu[34], pu[35],
            pu[36], pu[37], pu[38], pu[39],
        ]) as usize;
        
        let fhh = u16::dj([pu[54], pu[55]]) as usize;
        let dqk = u16::dj([pu[56], pu[57]]) as usize;
        
        
        for a in 0..dqk {
            let abt = epo + a * fhh;
            if abt + 56 > pu.len() {
                continue;
            }
            
            let bku = u32::dj([
                pu[abt], pu[abt + 1],
                pu[abt + 2], pu[abt + 3],
            ]);
            
            
            if bku != 1 {
                continue;
            }
            
            let caz = u64::dj([
                pu[abt + 8], pu[abt + 9],
                pu[abt + 10], pu[abt + 11],
                pu[abt + 12], pu[abt + 13],
                pu[abt + 14], pu[abt + 15],
            ]) as usize;
            
            let ctg = u64::dj([
                pu[abt + 16], pu[abt + 17],
                pu[abt + 18], pu[abt + 19],
                pu[abt + 20], pu[abt + 21],
                pu[abt + 22], pu[abt + 23],
            ]);
            
            let cgh = u64::dj([
                pu[abt + 32], pu[abt + 33],
                pu[abt + 34], pu[abt + 35],
                pu[abt + 36], pu[abt + 37],
                pu[abt + 38], pu[abt + 39],
            ]) as usize;
            
            let ctf = u64::dj([
                pu[abt + 40], pu[abt + 41],
                pu[abt + 42], pu[abt + 43],
                pu[abt + 44], pu[abt + 45],
                pu[abt + 46], pu[abt + 47],
            ]) as usize;
            
            let bvv = u32::dj([
                pu[abt + 4], pu[abt + 5],
                pu[abt + 6], pu[abt + 7],
            ]);
            
            
            let bob = (bvv & 4) != 0;
            let bjb = (bvv & 2) != 0;
            let kuk = (bvv & 1) != 0;
            
            self.memory.map(ctg, ctf, bob, bjb, kuk);
            
            
            if caz + cgh <= pu.len() {
                let _ = self.memory.write(ctg, &pu[caz..caz + cgh]);
            }
        }
        
        
        let dce = 0x7FFF_0000_0000u64;
        let ibn = 8 * 1024 * 1024;
        self.memory.map(dce - ibn as u64, ibn, true, true, false);
        self.cpu.rsp = dce - 8;
        
        
        self.cpu.pc = cxe;
        
        Ok(cxe)
    }
    
    
    pub fn wll(&mut self, cjc: &[&str], epy: &[&str]) -> Result<(), &'static str> {
        
        let mut mht: Vec<u64> = Vec::new();
        let mut ktt: Vec<u64> = Vec::new();
        
        
        for ji in cjc.iter().vv() {
            self.cpu.rsp -= (ji.len() + 1) as u64;
            let _ = self.memory.write(self.cpu.rsp, ji.as_bytes());
            let _ = self.memory.cvj(self.cpu.rsp + ji.len() as u64, 0);
            mht.push(self.cpu.rsp);
        }
        mht.dbh();
        
        for env in epy.iter().vv() {
            self.cpu.rsp -= (env.len() + 1) as u64;
            let _ = self.memory.write(self.cpu.rsp, env.as_bytes());
            let _ = self.memory.cvj(self.cpu.rsp + env.len() as u64, 0);
            ktt.push(self.cpu.rsp);
        }
        ktt.dbh();
        
        
        self.cpu.rsp &= !15;
        
        
        self.cpu.rsp -= 8;
        let _ = self.memory.tw(self.cpu.rsp, 0);
        
        
        for ptr in ktt.iter().vv() {
            self.cpu.rsp -= 8;
            let _ = self.memory.tw(self.cpu.rsp, *ptr);
        }
        
        
        self.cpu.rsp -= 8;
        let _ = self.memory.tw(self.cpu.rsp, 0);
        
        
        for ptr in mht.iter().vv() {
            self.cpu.rsp -= 8;
            let _ = self.memory.tw(self.cpu.rsp, *ptr);
        }
        
        
        self.cpu.rsp -= 8;
        let _ = self.memory.tw(self.cpu.rsp, cjc.len() as u64);
        
        Ok(())
    }
    
    
    pub fn gu(&mut self) -> DecodeResult {
        
        let aij = match self.memory.read(self.cpu.pc, 16) {
            Ok(o) => o,
            Err(_) => return DecodeResult::Q("Failed to fetch instruction"),
        };
        
        let mut w = 0;
        
        
        let mut aip: u8 = 0;
        let mut tlz = false;
        
        loop {
            if w >= aij.len() {
                return DecodeResult::Q("Instruction too long");
            }
            
            match aij[w] {
                0x40..=0x4F => {
                    aip = aij[w];
                    w += 1;
                }
                0x66 => {
                    tlz = true;
                    w += 1;
                }
                0xF0 | 0xF2 | 0xF3 | 0x26 | 0x2E | 0x36 | 0x3E | 0x64 | 0x65 => {
                    
                    w += 1;
                }
                _ => break,
            }
        }
        
        let ako = (aip & 0x08) != 0;
        let nx = (aip & 0x04) != 0;
        let pg = (aip & 0x02) != 0;
        let ic = (aip & 0x01) != 0;
        
        let opcode = aij[w];
        w += 1;
        
        match opcode {
            
            0x0F if aij.get(w) == Some(&0x05) => {
                self.cpu.pc += (w + 1) as u64;
                return DecodeResult::Hg;
            }
            
            
            0x90 => {
                self.cpu.pc += w as u64;
            }
            
            
            0xC3 => {
                let dbg = self.memory.aqi(self.cpu.rsp).unwrap_or(0);
                self.cpu.rsp += 8;
                self.cpu.pc = dbg;
                if dbg == 0 {
                    return DecodeResult::Lk(self.cpu.rax as i32);
                }
            }
            
            
            0x50..=0x57 => {
                let alq = (opcode - 0x50) + if ic { 8 } else { 0 };
                let ap = self.cpu.bzu(alq);
                self.cpu.rsp -= 8;
                let _ = self.memory.tw(self.cpu.rsp, ap);
                self.cpu.pc += w as u64;
            }
            
            
            0x58..=0x5F => {
                let alq = (opcode - 0x58) + if ic { 8 } else { 0 };
                let ap = self.memory.aqi(self.cpu.rsp).unwrap_or(0);
                self.cpu.rsp += 8;
                self.cpu.bxa(alq, ap);
                self.cpu.pc += w as u64;
            }
            
            
            0xB8..=0xBF if ako => {
                let alq = (opcode - 0xB8) + if ic { 8 } else { 0 };
                if w + 8 <= aij.len() {
                    let gf = u64::dj([
                        aij[w], aij[w + 1],
                        aij[w + 2], aij[w + 3],
                        aij[w + 4], aij[w + 5],
                        aij[w + 6], aij[w + 7],
                    ]);
                    self.cpu.bxa(alq, gf);
                    self.cpu.pc += (w + 8) as u64;
                } else {
                    return DecodeResult::Q("MOV imm64 truncated");
                }
            }
            
            
            0xB8..=0xBF => {
                let alq = (opcode - 0xB8) + if ic { 8 } else { 0 };
                if w + 4 <= aij.len() {
                    let gf = u32::dj([
                        aij[w], aij[w + 1],
                        aij[w + 2], aij[w + 3],
                    ]);
                    self.cpu.bxa(alq, gf as u64);
                    self.cpu.pc += (w + 4) as u64;
                } else {
                    return DecodeResult::Q("MOV imm32 truncated");
                }
            }
            
            
            0x31 if ako => {
                if w >= aij.len() {
                    return DecodeResult::Q("XOR missing modrm");
                }
                let ms = aij[w];
                w += 1;
                
                let hrq = (ms >> 6) & 3;
                let reg = ((ms >> 3) & 7) + if nx { 8 } else { 0 };
                let hb = (ms & 7) + if ic { 8 } else { 0 };
                
                if hrq == 3 {
                    
                    let cy = self.cpu.bzu(reg);
                    let cs = self.cpu.bzu(hb);
                    let result = cs ^ cy;
                    self.cpu.bxa(hb, result);
                    self.pxd(result);
                }
                self.cpu.pc += w as u64;
            }
            
            
            0x31 => {
                if w >= aij.len() {
                    return DecodeResult::Q("XOR missing modrm");
                }
                let ms = aij[w];
                w += 1;
                
                let hrq = (ms >> 6) & 3;
                let reg = ((ms >> 3) & 7) + if nx { 8 } else { 0 };
                let hb = (ms & 7) + if ic { 8 } else { 0 };
                
                if hrq == 3 {
                    let cy = self.cpu.bzu(reg) as u32;
                    let cs = self.cpu.bzu(hb) as u32;
                    let result = cs ^ cy;
                    self.cpu.bxa(hb, result as u64);
                    self.pxd(result as u64);
                }
                self.cpu.pc += w as u64;
            }
            
            
            0xE8 => {
                if w + 4 > aij.len() {
                    return DecodeResult::Q("CALL truncated");
                }
                let adj = i32::dj([
                    aij[w], aij[w + 1],
                    aij[w + 2], aij[w + 3],
                ]);
                let aqa = self.cpu.pc + (w + 4) as u64;
                self.cpu.rsp -= 8;
                let _ = self.memory.tw(self.cpu.rsp, aqa);
                self.cpu.pc = (aqa as i64 + adj as i64) as u64;
            }
            
            
            0xE9 => {
                if w + 4 > aij.len() {
                    return DecodeResult::Q("JMP truncated");
                }
                let adj = i32::dj([
                    aij[w], aij[w + 1],
                    aij[w + 2], aij[w + 3],
                ]);
                let aqa = self.cpu.pc + (w + 4) as u64;
                self.cpu.pc = (aqa as i64 + adj as i64) as u64;
            }
            
            
            0xEB => {
                if w >= aij.len() {
                    return DecodeResult::Q("JMP rel8 truncated");
                }
                let adj = aij[w] as i8;
                let aqa = self.cpu.pc + (w + 1) as u64;
                self.cpu.pc = (aqa as i64 + adj as i64) as u64;
            }
            
            
            0x70..=0x7F => {
                if w >= aij.len() {
                    return DecodeResult::Q("Jcc truncated");
                }
                let adj = aij[w] as i8;
                let mo = opcode & 0x0F;
                let aqa = self.cpu.pc + (w + 1) as u64;
                
                if self.qyp(mo) {
                    self.cpu.pc = (aqa as i64 + adj as i64) as u64;
                } else {
                    self.cpu.pc = aqa;
                }
            }
            
            
            0xCC => {
                crate::serial_println!("[INTERP] INT3 at 0x{:x}", self.cpu.pc);
                self.cpu.pc += w as u64;
            }
            
            
            0xF4 => {
                return DecodeResult::Lk(0);
            }
            
            _ => {
                crate::serial_println!(
                    "[INTERP] Unknown opcode 0x{:02x} at RIP=0x{:x}",
                    opcode, self.cpu.pc
                );
                
                self.cpu.pc += w as u64;
            }
        }
        
        DecodeResult::Cg
    }
    
    
    fn pxd(&mut self, result: u64) {
        self.cpu.rflags &= !(ASC_ | ASE_ | ACG_ | ACH_ | ACF_);
        
        if result == 0 {
            self.cpu.rflags |= ACH_;
        }
        if (result as i64) < 0 {
            self.cpu.rflags |= ACG_;
        }
        
        let vbq = (result as u8).ipi();
        if vbq % 2 == 0 {
            self.cpu.rflags |= ACF_;
        }
    }
    
    
    fn qyp(&self, mo: u8) -> bool {
        let vq = (self.cpu.rflags & ASC_) != 0;
        let aca = (self.cpu.rflags & ACH_) != 0;
        let eim = (self.cpu.rflags & ACG_) != 0;
        let gog = (self.cpu.rflags & ASE_) != 0;
        let ewp = (self.cpu.rflags & ACF_) != 0;
        
        match mo {
            0x0 => gog,           
            0x1 => !gog,          
            0x2 => vq,           
            0x3 => !vq,          
            0x4 => aca,           
            0x5 => !aca,          
            0x6 => vq || aca,     
            0x7 => !vq && !aca,   
            0x8 => eim,           
            0x9 => !eim,          
            0xA => ewp,           
            0xB => !ewp,          
            0xC => eim != gog,     
            0xD => eim == gog,     
            0xE => aca || (eim != gog), 
            0xF => !aca && (eim == gog), 
            _ => false,
        }
    }
    
    
    pub fn vw(&mut self) -> Result<i32, &'static str> {
        let mut au = 0u64;
        let csk = 1_000_000u64; 
        let vxl = 100_000u64;
        
        loop {
            match self.gu() {
                DecodeResult::Cg => {
                    au += 1;
                    if au % vxl == 0 {
                        crate::print!(".");  
                    }
                    if au > csk {
                        crate::println!();
                        crate::println!("Executed {} instructions", au);
                        return Err("Timeout: binary too complex for interpreter");
                    }
                }
                DecodeResult::Hg => {
                    
                    let ezk = self.cpu.rax;
                    let result = self.ixo();
                    
                    
                    if ezk == 60 || ezk == 231 {
                        return Ok(result as i32);
                    }
                    
                    self.cpu.rax = result as u64;
                }
                DecodeResult::Lk(aj) => {
                    return Ok(aj);
                }
                DecodeResult::Q(aa) => {
                    crate::println!();
                    crate::println!("After {} instructions:", au);
                    crate::println!("  RIP: 0x{:x}", self.cpu.pc);
                    return Err(aa);
                }
            }
        }
    }
    
    
    fn ixo(&mut self) -> i64 {
        let ezk = self.cpu.rax;
        let aai = self.cpu.rdi;
        let agf = self.cpu.rsi;
        let bfx = self.cpu.rdx;
        let fcs = self.cpu.r10;
        let gyx = self.cpu.r8;
        let xxw = self.cpu.r9;
        
        match ezk {
            
            1 => {
                let da = aai as i32;
                let k = agf;
                let az = bfx as usize;
                
                if let Ok(f) = self.memory.read(k, az) {
                    match self.aho.get(&da) {
                        Some(FileDescriptor::Btm) | Some(FileDescriptor::Btk) => {
                            if let Ok(e) = core::str::jg(&f) {
                                crate::print!("{}", e);
                            }
                            az as i64
                        }
                        _ => -9, 
                    }
                } else {
                    -14 
                }
            }
            
            
            0 => {
                let da = aai as i32;
                let k = agf;
                let az = bfx as usize;
                
                match self.aho.get(&da) {
                    Some(FileDescriptor::Btl) => {
                        
                        let mut dlc = 0;
                        while dlc < az {
                            if let Some(r) = crate::keyboard::auw() {
                                let _ = self.memory.cvj(k + dlc as u64, r);
                                dlc += 1;
                                if r == b'\n' {
                                    break;
                                }
                            } else if dlc > 0 {
                                break;
                            } else {
                                core::hint::hc();
                            }
                        }
                        dlc as i64
                    }
                    _ => -9, 
                }
            }
            
            
            60 => {
                -1 
            }
            
            
            12 => {
                let ag = aai;
                if ag == 0 {
                    self.memory.den() as i64
                } else {
                    self.memory.pip(ag);
                    ag as i64
                }
            }
            
            
            9 => {
                let ag = aai;
                let go = agf as usize;
                let ybz = bfx;
                let ddp = fcs;
                
                
                let efc = if ag == 0 {
                    self.memory.den()
                } else {
                    ag
                };
                
                self.memory.map(efc, go, true, true, false);
                if ag == 0 {
                    self.memory.pip(efc + go as u64);
                }
                
                efc as i64
            }
            
            
            39 => self.ce as i64,
            
            
            102 => 0, 
            
            
            104 => 0, 
            
            
            107 => 0,
            
            
            108 => 0,
            
            
            63 => {
                
                let k = aai;
                let xoc = b"Linux\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
                let _ = self.memory.write(k, xoc);
                0
            }
            
            
            158 => {
                let aj = aai;
                let ag = agf;
                
                match aj {
                    0x1002 => { 
                        self.cpu.fs = ag;
                        0
                    }
                    0x1003 => { 
                        self.cpu.ckx = ag;
                        0
                    }
                    _ => -22, 
                }
            }
            
            _ => {
                crate::serial_println!(
                    "[INTERP] Unknown syscall {} (rdi={:x}, rsi={:x}, rdx={:x})",
                    ezk, aai, agf, bfx
                );
                -38 
            }
        }
    }
}


pub fn peo(pu: &[u8], cjc: &[&str]) -> Result<i32, &'static str> {
    let mut ahp = Interpreter::new();
    
    
    ahp.ugu(pu)?;
    
    
    let epy = ["PATH=/bin:/usr/bin", "HOME=/root", "USER=root"];
    ahp.wll(cjc, &epy)?;
    
    crate::println!("Starting {} ...", cjc.get(0).unwrap_or(&"<binary>"));
    
    
    ahp.vw()
}
