









use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use alloc::vec;
use alloc::string::String;
use alloc::format;
use super::ir::*;


#[derive(Debug, Clone)]
pub struct RvCpu {
    
    pub regs: [u64; 34],
    
    pub pc: u64,
    
    
    pub cmp_a: i64,
    pub cmp_b: i64,
    
    pub cmp_a_u: u64,
    pub cmp_b_u: u64,
    
    pub inst_count: u64,
    
    pub halted: bool,
}

impl RvCpu {
    pub fn new() -> Self {
        let mut cpu = Self {
            regs: [0; 34],
            pc: 0,
            cmp_a: 0,
            cmp_b: 0,
            cmp_a_u: 0,
            cmp_b_u: 0,
            inst_count: 0,
            halted: false,
        };
        
        cpu.regs[Reg::X2 as usize] = 0x7FFF_FFF0;
        cpu
    }

    
    #[inline]
    pub fn get(&self, reg: Reg) -> u64 {
        if reg as u8 == 0 { 0 } else { self.regs[reg as usize] }
    }

    
    #[inline]
    pub fn set(&mut self, reg: Reg, val: u64) {
        if reg as u8 != 0 {
            self.regs[reg as usize] = val;
        }
    }

    
    #[inline]
    pub fn set_cmp(&mut self, a: u64, b: u64) {
        self.cmp_a = a as i64;
        self.cmp_b = b as i64;
        self.cmp_a_u = a;
        self.cmp_b_u = b;
    }

    
    pub fn eval_cond(&self, fc: FlagCond) -> bool {
        let jr = self.cmp_a.wrapping_sub(self.cmp_b);
        match fc {
            FlagCond::Eq    => self.cmp_a == self.cmp_b,
            FlagCond::Ne    => self.cmp_a != self.cmp_b,
            FlagCond::Lt    => self.cmp_a < self.cmp_b,
            FlagCond::Ge    => self.cmp_a >= self.cmp_b,
            FlagCond::Le    => self.cmp_a <= self.cmp_b,
            FlagCond::Gt    => self.cmp_a > self.cmp_b,
            FlagCond::Ltu   => self.cmp_a_u < self.cmp_b_u,
            FlagCond::Geu   => self.cmp_a_u >= self.cmp_b_u,
            FlagCond::Neg   => jr < 0,
            FlagCond::Pos   => jr >= 0,
            FlagCond::Ovf   => {
                
                (self.cmp_a ^ self.cmp_b) < 0 && (self.cmp_a ^ jr) < 0
            }
            FlagCond::NoOvf => {
                !((self.cmp_a ^ self.cmp_b) < 0 && (self.cmp_a ^ jr) < 0)
            }
        }
    }
}


pub struct RvMemory {
    
    regions: BTreeMap<u64, Vec<u8>>,
    
    pub total_allocated: usize,
}

impl RvMemory {
    pub fn new() -> Self {
        Self {
            regions: BTreeMap::new(),
            total_allocated: 0,
        }
    }

    
    pub fn map(&mut self, addr: u64, size: usize) {
        self.regions.insert(addr, vec![0u8; size]);
        self.total_allocated += size;
    }

    
    pub fn map_with_data(&mut self, addr: u64, data: &[u8]) {
        self.regions.insert(addr, data.to_vec());
        self.total_allocated += data.len();
    }

    
    pub fn read_u8(&self, addr: u64) -> Result<u8, MemError> {
        for (&base, data) in &self.regions {
            if addr >= base && addr < base + data.len() as u64 {
                return Ok(data[(addr - base) as usize]);
            }
        }
        Err(MemError::Unmapped(addr))
    }

    
    pub fn read_u16(&self, addr: u64) -> Result<u16, MemError> {
        Ok(u16::from_le_bytes([
            self.read_u8(addr)?,
            self.read_u8(addr + 1)?,
        ]))
    }

    
    pub fn read_u32(&self, addr: u64) -> Result<u32, MemError> {
        Ok(u32::from_le_bytes([
            self.read_u8(addr)?,
            self.read_u8(addr + 1)?,
            self.read_u8(addr + 2)?,
            self.read_u8(addr + 3)?,
        ]))
    }

    
    pub fn read_u64(&self, addr: u64) -> Result<u64, MemError> {
        Ok(u64::from_le_bytes([
            self.read_u8(addr)?,
            self.read_u8(addr + 1)?,
            self.read_u8(addr + 2)?,
            self.read_u8(addr + 3)?,
            self.read_u8(addr + 4)?,
            self.read_u8(addr + 5)?,
            self.read_u8(addr + 6)?,
            self.read_u8(addr + 7)?,
        ]))
    }

    
    pub fn write_u8(&mut self, addr: u64, val: u8) -> Result<(), MemError> {
        for (&base, data) in self.regions.iter_mut() {
            if addr >= base && addr < base + data.len() as u64 {
                data[(addr - base) as usize] = val;
                return Ok(());
            }
        }
        Err(MemError::Unmapped(addr))
    }

    
    pub fn write_u16(&mut self, addr: u64, val: u16) -> Result<(), MemError> {
        let bytes = val.to_le_bytes();
        self.write_u8(addr, bytes[0])?;
        self.write_u8(addr + 1, bytes[1])
    }

    
    pub fn write_u32(&mut self, addr: u64, val: u32) -> Result<(), MemError> {
        let bytes = val.to_le_bytes();
        for i in 0..4 {
            self.write_u8(addr + i, bytes[i as usize])?;
        }
        Ok(())
    }

    
    pub fn write_u64(&mut self, addr: u64, val: u64) -> Result<(), MemError> {
        let bytes = val.to_le_bytes();
        for i in 0..8 {
            self.write_u8(addr + i, bytes[i as usize])?;
        }
        Ok(())
    }

    
    pub fn qsr(&self, addr: u64, aoo: usize) -> Result<String, MemError> {
        let mut j = String::new();
        for i in 0..aoo {
            let b = self.read_u8(addr + i as u64)?;
            if b == 0 { break; }
            j.push(b as char);
        }
        Ok(j)
    }

    
    pub fn write_string(&mut self, addr: u64, j: &str) -> Result<(), MemError> {
        for (i, b) in j.bytes().enumerate() {
            self.write_u8(addr + i as u64, b)?;
        }
        self.write_u8(addr + j.len() as u64, 0)
    }
}


#[derive(Debug)]
pub enum MemError {
    Unmapped(u64),
}


#[derive(Debug)]
pub enum ExecResult {
    
    Continue,
    
    Syscall {
        number: u64,
        args: [u64; 6],
    },
    
    Breakpoint,
    
    Returned(u64),
    
    MemoryFault(u64),
    
    InstructionLimit,
    
    Halted,
}


pub struct RvInterpreter {
    
    pub cpu: RvCpu,
    
    pub mem: RvMemory,
    
    pub block_cache: BTreeMap<u64, Vec<RvInst>>,
    
    pub max_instructions: u64,
}

impl RvInterpreter {
    pub fn new() -> Self {
        Self {
            cpu: RvCpu::new(),
            mem: RvMemory::new(),
            block_cache: BTreeMap::new(),
            max_instructions: 10_000_000, 
        }
    }

    
    pub fn load_block(&mut self, block: &TranslatedBlock) {
        self.block_cache.insert(block.src_addr, block.instructions.clone());
    }

    
    pub fn load_blocks(&mut self, blocks: &[TranslatedBlock]) {
        for block in blocks {
            self.load_block(block);
        }
    }

    
    pub fn exec_one(&mut self, inst: &RvInst) -> ExecResult {
        self.cpu.inst_count += 1;

        match inst {
            
            RvInst::Add { aj, rs1, rs2 } => {
                self.cpu.set(*aj, self.cpu.get(*rs1).wrapping_add(self.cpu.get(*rs2)));
            }
            RvInst::Sub { aj, rs1, rs2 } => {
                self.cpu.set(*aj, self.cpu.get(*rs1).wrapping_sub(self.cpu.get(*rs2)));
            }
            RvInst::And { aj, rs1, rs2 } => {
                self.cpu.set(*aj, self.cpu.get(*rs1) & self.cpu.get(*rs2));
            }
            RvInst::Or { aj, rs1, rs2 } => {
                self.cpu.set(*aj, self.cpu.get(*rs1) | self.cpu.get(*rs2));
            }
            RvInst::Xor { aj, rs1, rs2 } => {
                self.cpu.set(*aj, self.cpu.get(*rs1) ^ self.cpu.get(*rs2));
            }
            RvInst::Sll { aj, rs1, rs2 } => {
                let acn = self.cpu.get(*rs2) & 63;
                self.cpu.set(*aj, self.cpu.get(*rs1) << acn);
            }
            RvInst::Srl { aj, rs1, rs2 } => {
                let acn = self.cpu.get(*rs2) & 63;
                self.cpu.set(*aj, self.cpu.get(*rs1) >> acn);
            }
            RvInst::Sra { aj, rs1, rs2 } => {
                let acn = self.cpu.get(*rs2) & 63;
                self.cpu.set(*aj, ((self.cpu.get(*rs1) as i64) >> acn) as u64);
            }
            RvInst::Slt { aj, rs1, rs2 } => {
                let v = if (self.cpu.get(*rs1) as i64) < (self.cpu.get(*rs2) as i64) { 1 } else { 0 };
                self.cpu.set(*aj, v);
            }
            RvInst::Sltu { aj, rs1, rs2 } => {
                let v = if self.cpu.get(*rs1) < self.cpu.get(*rs2) { 1 } else { 0 };
                self.cpu.set(*aj, v);
            }

            
            RvInst::Mul { aj, rs1, rs2 } => {
                self.cpu.set(*aj, self.cpu.get(*rs1).wrapping_mul(self.cpu.get(*rs2)));
            }
            RvInst::Mulh { aj, rs1, rs2 } => {
                let a = self.cpu.get(*rs1) as i64 as i128;
                let b = self.cpu.get(*rs2) as i64 as i128;
                self.cpu.set(*aj, ((a * b) >> 64) as u64);
            }
            RvInst::Div { aj, rs1, rs2 } => {
                let b = self.cpu.get(*rs2) as i64;
                if b == 0 {
                    self.cpu.set(*aj, u64::MAX); 
                } else {
                    self.cpu.set(*aj, ((self.cpu.get(*rs1) as i64).wrapping_div(b)) as u64);
                }
            }
            RvInst::Divu { aj, rs1, rs2 } => {
                let b = self.cpu.get(*rs2);
                if b == 0 {
                    self.cpu.set(*aj, u64::MAX);
                } else {
                    self.cpu.set(*aj, self.cpu.get(*rs1) / b);
                }
            }
            RvInst::Rem { aj, rs1, rs2 } => {
                let b = self.cpu.get(*rs2) as i64;
                if b == 0 {
                    self.cpu.set(*aj, self.cpu.get(*rs1));
                } else {
                    self.cpu.set(*aj, ((self.cpu.get(*rs1) as i64).wrapping_rem(b)) as u64);
                }
            }
            RvInst::Remu { aj, rs1, rs2 } => {
                let b = self.cpu.get(*rs2);
                if b == 0 {
                    self.cpu.set(*aj, self.cpu.get(*rs1));
                } else {
                    self.cpu.set(*aj, self.cpu.get(*rs1) % b);
                }
            }

            
            RvInst::Addi { aj, rs1, imm } => {
                self.cpu.set(*aj, self.cpu.get(*rs1).wrapping_add(*imm as u64));
            }
            RvInst::Andi { aj, rs1, imm } => {
                self.cpu.set(*aj, self.cpu.get(*rs1) & (*imm as u64));
            }
            RvInst::Ori { aj, rs1, imm } => {
                self.cpu.set(*aj, self.cpu.get(*rs1) | (*imm as u64));
            }
            RvInst::Xori { aj, rs1, imm } => {
                self.cpu.set(*aj, self.cpu.get(*rs1) ^ (*imm as u64));
            }
            RvInst::Slli { aj, rs1, acn } => {
                self.cpu.set(*aj, self.cpu.get(*rs1) << (*acn & 63));
            }
            RvInst::Srli { aj, rs1, acn } => {
                self.cpu.set(*aj, self.cpu.get(*rs1) >> (*acn & 63));
            }
            RvInst::Srai { aj, rs1, acn } => {
                self.cpu.set(*aj, ((self.cpu.get(*rs1) as i64) >> (*acn & 63)) as u64);
            }
            RvInst::Slti { aj, rs1, imm } => {
                let v = if (self.cpu.get(*rs1) as i64) < *imm { 1 } else { 0 };
                self.cpu.set(*aj, v);
            }
            RvInst::Sltiu { aj, rs1, imm } => {
                let v = if self.cpu.get(*rs1) < (*imm as u64) { 1 } else { 0 };
                self.cpu.set(*aj, v);
            }

            
            RvInst::Lui { aj, imm } => {
                self.cpu.set(*aj, ((*imm as u64) << 12) & 0xFFFF_FFFF_FFFF_F000);
            }
            RvInst::Auipc { aj, imm } => {
                self.cpu.set(*aj, self.cpu.pc.wrapping_add((*imm as u64) << 12));
            }

            
            RvInst::Lb { aj, rs1, offset } => {
                let addr = self.cpu.get(*rs1).wrapping_add(*offset as u64);
                match self.mem.read_u8(addr) {
                    Ok(v) => self.cpu.set(*aj, v as i8 as i64 as u64),
                    Err(_) => return ExecResult::MemoryFault(addr),
                }
            }
            RvInst::Lbu { aj, rs1, offset } => {
                let addr = self.cpu.get(*rs1).wrapping_add(*offset as u64);
                match self.mem.read_u8(addr) {
                    Ok(v) => self.cpu.set(*aj, v as u64),
                    Err(_) => return ExecResult::MemoryFault(addr),
                }
            }
            RvInst::Lh { aj, rs1, offset } => {
                let addr = self.cpu.get(*rs1).wrapping_add(*offset as u64);
                match self.mem.read_u16(addr) {
                    Ok(v) => self.cpu.set(*aj, v as i16 as i64 as u64),
                    Err(_) => return ExecResult::MemoryFault(addr),
                }
            }
            RvInst::Lhu { aj, rs1, offset } => {
                let addr = self.cpu.get(*rs1).wrapping_add(*offset as u64);
                match self.mem.read_u16(addr) {
                    Ok(v) => self.cpu.set(*aj, v as u64),
                    Err(_) => return ExecResult::MemoryFault(addr),
                }
            }
            RvInst::Lw { aj, rs1, offset } => {
                let addr = self.cpu.get(*rs1).wrapping_add(*offset as u64);
                match self.mem.read_u32(addr) {
                    Ok(v) => self.cpu.set(*aj, v as i32 as i64 as u64),
                    Err(_) => return ExecResult::MemoryFault(addr),
                }
            }
            RvInst::Lwu { aj, rs1, offset } => {
                let addr = self.cpu.get(*rs1).wrapping_add(*offset as u64);
                match self.mem.read_u32(addr) {
                    Ok(v) => self.cpu.set(*aj, v as u64),
                    Err(_) => return ExecResult::MemoryFault(addr),
                }
            }
            RvInst::Ld { aj, rs1, offset } => {
                let addr = self.cpu.get(*rs1).wrapping_add(*offset as u64);
                match self.mem.read_u64(addr) {
                    Ok(v) => self.cpu.set(*aj, v),
                    Err(_) => return ExecResult::MemoryFault(addr),
                }
            }

            
            RvInst::Sb { rs2, rs1, offset } => {
                let addr = self.cpu.get(*rs1).wrapping_add(*offset as u64);
                let val = self.cpu.get(*rs2) as u8;
                if self.mem.write_u8(addr, val).is_err() {
                    return ExecResult::MemoryFault(addr);
                }
            }
            RvInst::Sh { rs2, rs1, offset } => {
                let addr = self.cpu.get(*rs1).wrapping_add(*offset as u64);
                let val = self.cpu.get(*rs2) as u16;
                if self.mem.write_u16(addr, val).is_err() {
                    return ExecResult::MemoryFault(addr);
                }
            }
            RvInst::Sw { rs2, rs1, offset } => {
                let addr = self.cpu.get(*rs1).wrapping_add(*offset as u64);
                let val = self.cpu.get(*rs2) as u32;
                if self.mem.write_u32(addr, val).is_err() {
                    return ExecResult::MemoryFault(addr);
                }
            }
            RvInst::Sd { rs2, rs1, offset } => {
                let addr = self.cpu.get(*rs1).wrapping_add(*offset as u64);
                let val = self.cpu.get(*rs2);
                if self.mem.write_u64(addr, val).is_err() {
                    return ExecResult::MemoryFault(addr);
                }
            }

            
            RvInst::Beq { rs1, rs2, offset } => {
                if self.cpu.get(*rs1) == self.cpu.get(*rs2) {
                    self.cpu.pc = *offset as u64;
                    return ExecResult::Continue;
                }
            }
            RvInst::Bne { rs1, rs2, offset } => {
                if self.cpu.get(*rs1) != self.cpu.get(*rs2) {
                    self.cpu.pc = *offset as u64;
                    return ExecResult::Continue;
                }
            }
            RvInst::Blt { rs1, rs2, offset } => {
                if (self.cpu.get(*rs1) as i64) < (self.cpu.get(*rs2) as i64) {
                    self.cpu.pc = *offset as u64;
                    return ExecResult::Continue;
                }
            }
            RvInst::Bge { rs1, rs2, offset } => {
                if (self.cpu.get(*rs1) as i64) >= (self.cpu.get(*rs2) as i64) {
                    self.cpu.pc = *offset as u64;
                    return ExecResult::Continue;
                }
            }
            RvInst::Bltu { rs1, rs2, offset } => {
                if self.cpu.get(*rs1) < self.cpu.get(*rs2) {
                    self.cpu.pc = *offset as u64;
                    return ExecResult::Continue;
                }
            }
            RvInst::Bgeu { rs1, rs2, offset } => {
                if self.cpu.get(*rs1) >= self.cpu.get(*rs2) {
                    self.cpu.pc = *offset as u64;
                    return ExecResult::Continue;
                }
            }

            
            RvInst::Jal { aj, offset } => {
                
                self.cpu.set(*aj, self.cpu.pc);
                self.cpu.pc = *offset as u64;
                return ExecResult::Continue;
            }
            RvInst::Jalr { aj, rs1, offset } => {
                let target = self.cpu.get(*rs1).wrapping_add(*offset as u64) & !1;
                self.cpu.set(*aj, self.cpu.pc);
                self.cpu.pc = target;
                return ExecResult::Continue;
            }

            
            RvInst::Ecall => {
                let number = self.cpu.get(Reg::X17); 
                let args = [
                    self.cpu.get(Reg::X10), 
                    self.cpu.get(Reg::X11), 
                    self.cpu.get(Reg::X12), 
                    self.cpu.get(Reg::X13), 
                    self.cpu.get(Reg::X14), 
                    self.cpu.get(Reg::X15), 
                ];
                return ExecResult::Syscall { number, args };
            }
            RvInst::Ebreak => {
                return ExecResult::Breakpoint;
            }
            RvInst::Fence => {
                
            }

            
            RvInst::AmoswapD { aj, rs2, rs1 } => {
                let addr = self.cpu.get(*rs1);
                match self.mem.read_u64(addr) {
                    Ok(qb) => {
                        self.cpu.set(*aj, qb);
                        let _ = self.mem.write_u64(addr, self.cpu.get(*rs2));
                    }
                    Err(_) => return ExecResult::MemoryFault(addr),
                }
            }
            RvInst::AmoaddD { aj, rs2, rs1 } => {
                let addr = self.cpu.get(*rs1);
                match self.mem.read_u64(addr) {
                    Ok(qb) => {
                        self.cpu.set(*aj, qb);
                        let new = qb.wrapping_add(self.cpu.get(*rs2));
                        let _ = self.mem.write_u64(addr, new);
                    }
                    Err(_) => return ExecResult::MemoryFault(addr),
                }
            }

            
            RvInst::Li { aj, imm } => {
                self.cpu.set(*aj, *imm as u64);
            }
            RvInst::Mv { aj, oc } => {
                self.cpu.set(*aj, self.cpu.get(*oc));
            }
            RvInst::Nop => {}
            RvInst::Ret => {
                let dxe = self.cpu.get(Reg::X1);
                if dxe == 0 {
                    
                    return ExecResult::Returned(self.cpu.get(Reg::X10));
                }
                self.cpu.pc = dxe;
                return ExecResult::Continue;
            }
            RvInst::Call { offset } => {
                self.cpu.set(Reg::X1, self.cpu.pc);
                self.cpu.pc = *offset as u64;
                return ExecResult::Continue;
            }

            
            RvInst::CmpFlags { rs1, rs2 } => {
                self.cpu.set_cmp(self.cpu.get(*rs1), self.cpu.get(*rs2));
            }
            RvInst::BranchCond { fc, offset } => {
                if self.cpu.eval_cond(*fc) {
                    self.cpu.pc = *offset as u64;
                    return ExecResult::Continue;
                }
            }
            RvInst::SrcAnnotation { .. } => {
                
            }
        }

        ExecResult::Continue
    }

    
    pub fn exec_block(&mut self, instructions: &[RvInst]) -> ExecResult {
        let mut ip = 0;
        while ip < instructions.len() {
            if self.cpu.inst_count >= self.max_instructions {
                return ExecResult::InstructionLimit;
            }

            let result = self.exec_one(&instructions[ip]);
            ip += 1;

            match result {
                ExecResult::Continue => {
                    
                    
                    
                }
                other => return other,
            }
        }

        ExecResult::Continue
    }

    
    pub fn qun(&mut self, start_addr: u64) -> ExecResult {
        self.cpu.pc = start_addr;

        loop {
            if self.cpu.inst_count >= self.max_instructions {
                return ExecResult::InstructionLimit;
            }

            
            if let Some(block_insts) = self.block_cache.get(&self.cpu.pc).cloned() {
                let nmt = self.cpu.pc;
                let result = self.exec_block(&block_insts);

                match result {
                    ExecResult::Continue => {
                        
                        if self.cpu.pc == nmt {
                            
                            return ExecResult::Returned(self.cpu.get(Reg::X10));
                        }
                        
                    }
                    other => return other,
                }
            } else {
                
                return ExecResult::Returned(self.cpu.get(Reg::X10));
            }
        }
    }

    
    pub fn dump_state(&self) -> String {
        let mut j = String::from("=== RISC-V IR CPU State ===\n");
        for i in 0..32 {
            let reg = Reg::enm(i);
            let val = self.cpu.get(reg);
            if val != 0 {
                j.push_str(&format!("  {:4} (x{:2}) = 0x{:016X} ({})\n",
                    reg.abi_name(), i, val, val as i64));
            }
        }
        j.push_str(&format!("  pc = 0x{:016X}\n", self.cpu.pc));
        j.push_str(&format!("  instructions executed: {}\n", self.cpu.inst_count));
        j
    }
}
