




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
    
    
    pub rip: u64,
    
    
    pub rflags: u64,
    
    
    pub cs: u16,
    pub ds: u16,
    pub es: u16,
    pub fs: u64,  
    pub gs: u64,  
    pub ss: u16,
}

impl CpuState {
    pub fn new() -> Self {
        Self {
            rax: 0, rbx: 0, rcx: 0, rdx: 0,
            rsi: 0, rdi: 0, rbp: 0, rsp: 0,
            r8: 0, r9: 0, r10: 0, r11: 0,
            r12: 0, r13: 0, r14: 0, r15: 0,
            rip: 0,
            rflags: 0x202, 
            cs: 0x33, ds: 0x2b, es: 0x2b,
            fs: 0, gs: 0, ss: 0x2b,
        }
    }
    
    
    pub fn get_reg(&self, idx: u8) -> u64 {
        match idx {
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
    
    
    pub fn set_reg(&mut self, idx: u8, val: u64) {
        match idx {
            0 => self.rax = val,
            1 => self.rcx = val,
            2 => self.rdx = val,
            3 => self.rbx = val,
            4 => self.rsp = val,
            5 => self.rbp = val,
            6 => self.rsi = val,
            7 => self.rdi = val,
            8 => self.r8 = val,
            9 => self.r9 = val,
            10 => self.r10 = val,
            11 => self.r11 = val,
            12 => self.r12 = val,
            13 => self.r13 = val,
            14 => self.r14 = val,
            15 => self.r15 = val,
            _ => {}
        }
    }
}


pub const AUG_: u64 = 1 << 0;  
pub const ADW_: u64 = 1 << 2;  
pub const DPU_: u64 = 1 << 4;  
pub const ADY_: u64 = 1 << 6;  
pub const ADX_: u64 = 1 << 7;  
pub const AUI_: u64 = 1 << 11; 


pub struct ProcessMemory {
    
    regions: BTreeMap<u64, MemoryRegion>,
    
    brk: u64,
}

struct MemoryRegion {
    data: Vec<u8>,
    readable: bool,
    writable: bool,
    fvn: bool,
}

impl ProcessMemory {
    pub fn new() -> Self {
        Self {
            regions: BTreeMap::new(),
            brk: 0x1000_0000, 
        }
    }
    
    
    pub fn map(&mut self, addr: u64, size: usize, r: bool, w: bool, x: bool) {
        self.regions.insert(addr, MemoryRegion {
            data: alloc::vec![0u8; size],
            readable: r,
            writable: w,
            fvn: x,
        });
    }
    
    
    pub fn write(&mut self, addr: u64, data: &[u8]) -> Result<(), &'static str> {
        for (region_base, qd) in self.regions.iter_mut() {
            let cdf = *region_base + qd.data.len() as u64;
            if addr >= *region_base && addr < cdf {
                if !qd.writable {
                    return Err("Write to non-writable memory");
                }
                let offset = (addr - *region_base) as usize;
                let mb = core::cmp::min(data.len(), qd.data.len() - offset);
                qd.data[offset..offset + mb].copy_from_slice(&data[..mb]);
                return Ok(());
            }
        }
        Err("Write to unmapped memory")
    }
    
    
    pub fn read(&self, addr: u64, len: usize) -> Result<Vec<u8>, &'static str> {
        for (region_base, qd) in self.regions.iter() {
            let cdf = *region_base + qd.data.len() as u64;
            if addr >= *region_base && addr < cdf {
                if !qd.readable {
                    return Err("Read from non-readable memory");
                }
                let offset = (addr - *region_base) as usize;
                let mb = core::cmp::min(len, qd.data.len() - offset);
                return Ok(qd.data[offset..offset + mb].to_vec());
            }
        }
        Err("Read from unmapped memory")
    }
    
    
    pub fn read_u8(&self, addr: u64) -> Result<u8, &'static str> {
        let data = self.read(addr, 1)?;
        Ok(data[0])
    }
    
    
    pub fn read_u16(&self, addr: u64) -> Result<u16, &'static str> {
        let data = self.read(addr, 2)?;
        Ok(u16::from_le_bytes([data[0], data[1]]))
    }
    
    
    pub fn read_u32(&self, addr: u64) -> Result<u32, &'static str> {
        let data = self.read(addr, 4)?;
        Ok(u32::from_le_bytes([data[0], data[1], data[2], data[3]]))
    }
    
    
    pub fn read_u64(&self, addr: u64) -> Result<u64, &'static str> {
        let data = self.read(addr, 8)?;
        Ok(u64::from_le_bytes([
            data[0], data[1], data[2], data[3],
            data[4], data[5], data[6], data[7],
        ]))
    }
    
    
    pub fn write_u8(&mut self, addr: u64, val: u8) -> Result<(), &'static str> {
        self.write(addr, &[val])
    }
    
    
    pub fn write_u64(&mut self, addr: u64, val: u64) -> Result<(), &'static str> {
        self.write(addr, &val.to_le_bytes())
    }
    
    
    pub fn brk(&self) -> u64 { self.brk }
    pub fn set_brk(&mut self, new_brk: u64) { self.brk = new_brk; }
}


pub enum DecodeResult {
    
    Continue,
    
    Syscall,
    
    Exit(i32),
    
    Error(&'static str),
}


pub struct Interpreter {
    pub cpu: CpuState,
    pub memory: ProcessMemory,
    
    pub fds: BTreeMap<i32, FileDescriptor>,
    
    next_fd: i32,
    
    pub pid: u32,
    
    pub cwd: String,
    
    pub argv: Vec<String>,
    
    pub bzm: Vec<String>,
}

pub enum FileDescriptor {
    Stdin,
    Stdout,
    Stderr,
    File { path: String, position: u64 },
    Pipe { buffer: Vec<u8> },
}

impl Interpreter {
    pub fn new() -> Self {
        let mut fds = BTreeMap::new();
        fds.insert(0, FileDescriptor::Stdin);
        fds.insert(1, FileDescriptor::Stdout);
        fds.insert(2, FileDescriptor::Stderr);
        
        Self {
            cpu: CpuState::new(),
            memory: ProcessMemory::new(),
            fds,
            next_fd: 3,
            pid: 1,
            cwd: String::from("/"),
            argv: Vec::new(),
            bzm: Vec::new(),
        }
    }
    
    
    pub fn load_elf(&mut self, gz: &[u8]) -> Result<u64, &'static str> {
        
        if gz.len() < 64 {
            return Err("ELF too small");
        }
        if &gz[0..4] != b"\x7fELF" {
            return Err("Not an ELF file");
        }
        if gz[4] != 2 {
            return Err("Not 64-bit ELF");
        }
        if gz[5] != 1 {
            return Err("Not little-endian");
        }
        
        
        let e_type = u16::from_le_bytes([gz[16], gz[17]]);
        if e_type != 2 && e_type != 3 {
            return Err("Not executable or shared object");
        }
        
        let e_entry = u64::from_le_bytes([
            gz[24], gz[25], gz[26], gz[27],
            gz[28], gz[29], gz[30], gz[31],
        ]);
        
        let e_phoff = u64::from_le_bytes([
            gz[32], gz[33], gz[34], gz[35],
            gz[36], gz[37], gz[38], gz[39],
        ]) as usize;
        
        let e_phentsize = u16::from_le_bytes([gz[54], gz[55]]) as usize;
        let e_phnum = u16::from_le_bytes([gz[56], gz[57]]) as usize;
        
        
        for i in 0..e_phnum {
            let nv = e_phoff + i * e_phentsize;
            if nv + 56 > gz.len() {
                continue;
            }
            
            let p_type = u32::from_le_bytes([
                gz[nv], gz[nv + 1],
                gz[nv + 2], gz[nv + 3],
            ]);
            
            
            if p_type != 1 {
                continue;
            }
            
            let p_offset = u64::from_le_bytes([
                gz[nv + 8], gz[nv + 9],
                gz[nv + 10], gz[nv + 11],
                gz[nv + 12], gz[nv + 13],
                gz[nv + 14], gz[nv + 15],
            ]) as usize;
            
            let p_vaddr = u64::from_le_bytes([
                gz[nv + 16], gz[nv + 17],
                gz[nv + 18], gz[nv + 19],
                gz[nv + 20], gz[nv + 21],
                gz[nv + 22], gz[nv + 23],
            ]);
            
            let p_filesz = u64::from_le_bytes([
                gz[nv + 32], gz[nv + 33],
                gz[nv + 34], gz[nv + 35],
                gz[nv + 36], gz[nv + 37],
                gz[nv + 38], gz[nv + 39],
            ]) as usize;
            
            let p_memsz = u64::from_le_bytes([
                gz[nv + 40], gz[nv + 41],
                gz[nv + 42], gz[nv + 43],
                gz[nv + 44], gz[nv + 45],
                gz[nv + 46], gz[nv + 47],
            ]) as usize;
            
            let p_flags = u32::from_le_bytes([
                gz[nv + 4], gz[nv + 5],
                gz[nv + 6], gz[nv + 7],
            ]);
            
            
            let readable = (p_flags & 4) != 0;
            let writable = (p_flags & 2) != 0;
            let fvn = (p_flags & 1) != 0;
            
            self.memory.map(p_vaddr, p_memsz, readable, writable, fvn);
            
            
            if p_offset + p_filesz <= gz.len() {
                let _ = self.memory.write(p_vaddr, &gz[p_offset..p_offset + p_filesz]);
            }
        }
        
        
        let bdt = 0x7FFF_0000_0000u64;
        let eah = 8 * 1024 * 1024;
        self.memory.map(bdt - eah as u64, eah, true, true, false);
        self.cpu.rsp = bdt - 8;
        
        
        self.cpu.rip = e_entry;
        
        Ok(e_entry)
    }
    
    
    pub fn setup_stack(&mut self, argv: &[&str], bzm: &[&str]) -> Result<(), &'static str> {
        
        let mut gwm: Vec<u64> = Vec::new();
        let mut fva: Vec<u64> = Vec::new();
        
        
        for db in argv.iter().rev() {
            self.cpu.rsp -= (db.len() + 1) as u64;
            let _ = self.memory.write(self.cpu.rsp, db.as_bytes());
            let _ = self.memory.write_u8(self.cpu.rsp + db.len() as u64, 0);
            gwm.push(self.cpu.rsp);
        }
        gwm.reverse();
        
        for env in bzm.iter().rev() {
            self.cpu.rsp -= (env.len() + 1) as u64;
            let _ = self.memory.write(self.cpu.rsp, env.as_bytes());
            let _ = self.memory.write_u8(self.cpu.rsp + env.len() as u64, 0);
            fva.push(self.cpu.rsp);
        }
        fva.reverse();
        
        
        self.cpu.rsp &= !15;
        
        
        self.cpu.rsp -= 8;
        let _ = self.memory.write_u64(self.cpu.rsp, 0);
        
        
        for ptr in fva.iter().rev() {
            self.cpu.rsp -= 8;
            let _ = self.memory.write_u64(self.cpu.rsp, *ptr);
        }
        
        
        self.cpu.rsp -= 8;
        let _ = self.memory.write_u64(self.cpu.rsp, 0);
        
        
        for ptr in gwm.iter().rev() {
            self.cpu.rsp -= 8;
            let _ = self.memory.write_u64(self.cpu.rsp, *ptr);
        }
        
        
        self.cpu.rsp -= 8;
        let _ = self.memory.write_u64(self.cpu.rsp, argv.len() as u64);
        
        Ok(())
    }
    
    
    pub fn step(&mut self) -> DecodeResult {
        
        let ro = match self.memory.read(self.cpu.rip, 16) {
            Ok(b) => b,
            Err(_) => return DecodeResult::Error("Failed to fetch instruction"),
        };
        
        let mut idx = 0;
        
        
        let mut rp: u8 = 0;
        let mut mja = false;
        
        loop {
            if idx >= ro.len() {
                return DecodeResult::Error("Instruction too long");
            }
            
            match ro[idx] {
                0x40..=0x4F => {
                    rp = ro[idx];
                    idx += 1;
                }
                0x66 => {
                    mja = true;
                    idx += 1;
                }
                0xF0 | 0xF2 | 0xF3 | 0x26 | 0x2E | 0x36 | 0x3E | 0x64 | 0x65 => {
                    
                    idx += 1;
                }
                _ => break,
            }
        }
        
        let rex_w = (rp & 0x08) != 0;
        let gb = (rp & 0x04) != 0;
        let gp = (rp & 0x02) != 0;
        let cq = (rp & 0x01) != 0;
        
        let opcode = ro[idx];
        idx += 1;
        
        match opcode {
            
            0x0F if ro.get(idx) == Some(&0x05) => {
                self.cpu.rip += (idx + 1) as u64;
                return DecodeResult::Syscall;
            }
            
            
            0x90 => {
                self.cpu.rip += idx as u64;
            }
            
            
            0xC3 => {
                let bdk = self.memory.read_u64(self.cpu.rsp).unwrap_or(0);
                self.cpu.rsp += 8;
                self.cpu.rip = bdk;
                if bdk == 0 {
                    return DecodeResult::Exit(self.cpu.rax as i32);
                }
            }
            
            
            0x50..=0x57 => {
                let tb = (opcode - 0x50) + if cq { 8 } else { 0 };
                let val = self.cpu.get_reg(tb);
                self.cpu.rsp -= 8;
                let _ = self.memory.write_u64(self.cpu.rsp, val);
                self.cpu.rip += idx as u64;
            }
            
            
            0x58..=0x5F => {
                let tb = (opcode - 0x58) + if cq { 8 } else { 0 };
                let val = self.memory.read_u64(self.cpu.rsp).unwrap_or(0);
                self.cpu.rsp += 8;
                self.cpu.set_reg(tb, val);
                self.cpu.rip += idx as u64;
            }
            
            
            0xB8..=0xBF if rex_w => {
                let tb = (opcode - 0xB8) + if cq { 8 } else { 0 };
                if idx + 8 <= ro.len() {
                    let imm = u64::from_le_bytes([
                        ro[idx], ro[idx + 1],
                        ro[idx + 2], ro[idx + 3],
                        ro[idx + 4], ro[idx + 5],
                        ro[idx + 6], ro[idx + 7],
                    ]);
                    self.cpu.set_reg(tb, imm);
                    self.cpu.rip += (idx + 8) as u64;
                } else {
                    return DecodeResult::Error("MOV imm64 truncated");
                }
            }
            
            
            0xB8..=0xBF => {
                let tb = (opcode - 0xB8) + if cq { 8 } else { 0 };
                if idx + 4 <= ro.len() {
                    let imm = u32::from_le_bytes([
                        ro[idx], ro[idx + 1],
                        ro[idx + 2], ro[idx + 3],
                    ]);
                    self.cpu.set_reg(tb, imm as u64);
                    self.cpu.rip += (idx + 4) as u64;
                } else {
                    return DecodeResult::Error("MOV imm32 truncated");
                }
            }
            
            
            0x31 if rex_w => {
                if idx >= ro.len() {
                    return DecodeResult::Error("XOR missing modrm");
                }
                let fi = ro[idx];
                idx += 1;
                
                let dul = (fi >> 6) & 3;
                let reg = ((fi >> 3) & 7) + if gb { 8 } else { 0 };
                let rm = (fi & 7) + if cq { 8 } else { 0 };
                
                if dul == 3 {
                    
                    let src = self.cpu.get_reg(reg);
                    let dst = self.cpu.get_reg(rm);
                    let result = dst ^ src;
                    self.cpu.set_reg(rm, result);
                    self.update_flags_logic(result);
                }
                self.cpu.rip += idx as u64;
            }
            
            
            0x31 => {
                if idx >= ro.len() {
                    return DecodeResult::Error("XOR missing modrm");
                }
                let fi = ro[idx];
                idx += 1;
                
                let dul = (fi >> 6) & 3;
                let reg = ((fi >> 3) & 7) + if gb { 8 } else { 0 };
                let rm = (fi & 7) + if cq { 8 } else { 0 };
                
                if dul == 3 {
                    let src = self.cpu.get_reg(reg) as u32;
                    let dst = self.cpu.get_reg(rm) as u32;
                    let result = dst ^ src;
                    self.cpu.set_reg(rm, result as u64);
                    self.update_flags_logic(result as u64);
                }
                self.cpu.rip += idx as u64;
            }
            
            
            0xE8 => {
                if idx + 4 > ro.len() {
                    return DecodeResult::Error("CALL truncated");
                }
                let ot = i32::from_le_bytes([
                    ro[idx], ro[idx + 1],
                    ro[idx + 2], ro[idx + 3],
                ]);
                let vo = self.cpu.rip + (idx + 4) as u64;
                self.cpu.rsp -= 8;
                let _ = self.memory.write_u64(self.cpu.rsp, vo);
                self.cpu.rip = (vo as i64 + ot as i64) as u64;
            }
            
            
            0xE9 => {
                if idx + 4 > ro.len() {
                    return DecodeResult::Error("JMP truncated");
                }
                let ot = i32::from_le_bytes([
                    ro[idx], ro[idx + 1],
                    ro[idx + 2], ro[idx + 3],
                ]);
                let vo = self.cpu.rip + (idx + 4) as u64;
                self.cpu.rip = (vo as i64 + ot as i64) as u64;
            }
            
            
            0xEB => {
                if idx >= ro.len() {
                    return DecodeResult::Error("JMP rel8 truncated");
                }
                let ot = ro[idx] as i8;
                let vo = self.cpu.rip + (idx + 1) as u64;
                self.cpu.rip = (vo as i64 + ot as i64) as u64;
            }
            
            
            0x70..=0x7F => {
                if idx >= ro.len() {
                    return DecodeResult::Error("Jcc truncated");
                }
                let ot = ro[idx] as i8;
                let fc = opcode & 0x0F;
                let vo = self.cpu.rip + (idx + 1) as u64;
                
                if self.check_condition(fc) {
                    self.cpu.rip = (vo as i64 + ot as i64) as u64;
                } else {
                    self.cpu.rip = vo;
                }
            }
            
            
            0xCC => {
                crate::serial_println!("[INTERP] INT3 at 0x{:x}", self.cpu.rip);
                self.cpu.rip += idx as u64;
            }
            
            
            0xF4 => {
                return DecodeResult::Exit(0);
            }
            
            _ => {
                crate::serial_println!(
                    "[INTERP] Unknown opcode 0x{:02x} at RIP=0x{:x}",
                    opcode, self.cpu.rip
                );
                
                self.cpu.rip += idx as u64;
            }
        }
        
        DecodeResult::Continue
    }
    
    
    fn update_flags_logic(&mut self, result: u64) {
        self.cpu.rflags &= !(AUG_ | AUI_ | ADX_ | ADY_ | ADW_);
        
        if result == 0 {
            self.cpu.rflags |= ADY_;
        }
        if (result as i64) < 0 {
            self.cpu.rflags |= ADX_;
        }
        
        let gme = (result as u8).count_ones();
        if gme % 2 == 0 {
            self.cpu.rflags |= ADW_;
        }
    }
    
    
    fn check_condition(&self, fc: u8) -> bool {
        let cf = (self.cpu.rflags & AUG_) != 0;
        let zf = (self.cpu.rflags & ADY_) != 0;
        let bvs = (self.cpu.rflags & ADX_) != 0;
        let dby = (self.cpu.rflags & AUI_) != 0;
        let ccq = (self.cpu.rflags & ADW_) != 0;
        
        match fc {
            0x0 => dby,           
            0x1 => !dby,          
            0x2 => cf,           
            0x3 => !cf,          
            0x4 => zf,           
            0x5 => !zf,          
            0x6 => cf || zf,     
            0x7 => !cf && !zf,   
            0x8 => bvs,           
            0x9 => !bvs,          
            0xA => ccq,           
            0xB => !ccq,          
            0xC => bvs != dby,     
            0xD => bvs == dby,     
            0xE => zf || (bvs != dby), 
            0xF => !zf && (bvs == dby), 
            _ => false,
        }
    }
    
    
    pub fn run(&mut self) -> Result<i32, &'static str> {
        let mut steps = 0u64;
        let ayd = 1_000_000u64; 
        let ogb = 100_000u64;
        
        loop {
            match self.step() {
                DecodeResult::Continue => {
                    steps += 1;
                    if steps % ogb == 0 {
                        crate::print!(".");  
                    }
                    if steps > ayd {
                        crate::println!();
                        crate::println!("Executed {} instructions", steps);
                        return Err("Timeout: binary too complex for interpreter");
                    }
                }
                DecodeResult::Syscall => {
                    
                    let cec = self.cpu.rax;
                    let result = self.handle_syscall();
                    
                    
                    if cec == 60 || cec == 231 {
                        return Ok(result as i32);
                    }
                    
                    self.cpu.rax = result as u64;
                }
                DecodeResult::Exit(code) => {
                    return Ok(code);
                }
                DecodeResult::Error(e) => {
                    crate::println!();
                    crate::println!("After {} instructions:", steps);
                    crate::println!("  RIP: 0x{:x}", self.cpu.rip);
                    return Err(e);
                }
            }
        }
    }
    
    
    fn handle_syscall(&mut self) -> i64 {
        let cec = self.cpu.rax;
        let arg1 = self.cpu.rdi;
        let arg2 = self.cpu.rsi;
        let aer = self.cpu.rdx;
        let cfw = self.cpu.r10;
        let dhv = self.cpu.r8;
        let pwq = self.cpu.r9;
        
        match cec {
            
            1 => {
                let fd = arg1 as i32;
                let buf = arg2;
                let count = aer as usize;
                
                if let Ok(data) = self.memory.read(buf, count) {
                    match self.fds.get(&fd) {
                        Some(FileDescriptor::Stdout) | Some(FileDescriptor::Stderr) => {
                            if let Ok(j) = core::str::from_utf8(&data) {
                                crate::print!("{}", j);
                            }
                            count as i64
                        }
                        _ => -9, 
                    }
                } else {
                    -14 
                }
            }
            
            
            0 => {
                let fd = arg1 as i32;
                let buf = arg2;
                let count = aer as usize;
                
                match self.fds.get(&fd) {
                    Some(FileDescriptor::Stdin) => {
                        
                        let mut biv = 0;
                        while biv < count {
                            if let Some(c) = crate::keyboard::ya() {
                                let _ = self.memory.write_u8(buf + biv as u64, c);
                                biv += 1;
                                if c == b'\n' {
                                    break;
                                }
                            } else if biv > 0 {
                                break;
                            } else {
                                core::hint::spin_loop();
                            }
                        }
                        biv as i64
                    }
                    _ => -9, 
                }
            }
            
            
            60 => {
                -1 
            }
            
            
            12 => {
                let addr = arg1;
                if addr == 0 {
                    self.memory.brk() as i64
                } else {
                    self.memory.set_brk(addr);
                    addr as i64
                }
            }
            
            
            9 => {
                let addr = arg1;
                let length = arg2 as usize;
                let pxf = aer;
                let bej = cfw;
                
                
                let bug = if addr == 0 {
                    self.memory.brk()
                } else {
                    addr
                };
                
                self.memory.map(bug, length, true, true, false);
                if addr == 0 {
                    self.memory.set_brk(bug + length as u64);
                }
                
                bug as i64
            }
            
            
            39 => self.pid as i64,
            
            
            102 => 0, 
            
            
            104 => 0, 
            
            
            107 => 0,
            
            
            108 => 0,
            
            
            63 => {
                
                let buf = arg1;
                let ppl = b"Linux\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
                let _ = self.memory.write(buf, ppl);
                0
            }
            
            
            158 => {
                let code = arg1;
                let addr = arg2;
                
                match code {
                    0x1002 => { 
                        self.cpu.fs = addr;
                        0
                    }
                    0x1003 => { 
                        self.cpu.gs = addr;
                        0
                    }
                    _ => -22, 
                }
            }
            
            _ => {
                crate::serial_println!(
                    "[INTERP] Unknown syscall {} (rdi={:x}, rsi={:x}, rdx={:x})",
                    cec, arg1, arg2, aer
                );
                -38 
            }
        }
    }
}


pub fn jbu(gz: &[u8], argv: &[&str]) -> Result<i32, &'static str> {
    let mut interp = Interpreter::new();
    
    
    interp.load_elf(gz)?;
    
    
    let bzm = ["PATH=/bin:/usr/bin", "HOME=/root", "USER=root"];
    interp.setup_stack(argv, &bzm)?;
    
    crate::println!("Starting {} ...", argv.get(0).unwrap_or(&"<binary>"));
    
    
    interp.run()
}
