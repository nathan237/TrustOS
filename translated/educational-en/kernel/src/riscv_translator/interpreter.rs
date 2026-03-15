// TrustOS Universal Architecture Translation Layer
// RISC-V IR Interpreter
//
// Executes RISC-V IR instructions on any host architecture.
// This is the heart of the universal translation layer:
//   Source binary → Decoder → RISC-V IR → THIS INTERPRETER → execution
//
// The interpreter maintains a virtual RISC-V CPU state and emulated memory,
// allowing binaries from x86_64, ARM64, etc. to run anywhere TrustOS runs.

use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use alloc::vec;
use alloc::string::String;
use alloc::format;
use super::ir::*;

/// Virtual RISC-V CPU state
#[derive(Debug, Clone)]
// Public structure — visible outside this module.
pub struct RvCpu {
    /// General-purpose registers x0-x31 + virtual registers
    pub regs: [u64; 34],
    /// Program counter (index into current block's instruction list)
    pub pc: u64,
    /// Comparison result for flag-based branches
    /// Stores (rs1_val, rs2_val) from last CmpFlags
    pub cmp_a: i64,
    pub cmp_b: i64,
    /// Track unsigned comparison too
    pub cmp_a_u: u64,
    pub cmp_b_u: u64,
    /// Number of instructions executed
    pub inst_count: u64,
    /// Halted flag
    pub halted: bool,
}

// Implementation block — defines methods for the type above.
impl RvCpu {
        // Public function — callable from other modules.
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
        // Set initial stack pointer (high address)
        cpu.regs[Reg::X2 as usize] = 0x7FFF_FFF0;
        cpu
    }

    /// Read register (x0 always returns 0)
    #[inline]
        // Public function — callable from other modules.
pub fn get(&self, reg: Reg) -> u64 {
        if reg as u8 == 0 { 0 } else { self.regs[reg as usize] }
    }

    /// Write register (x0 writes are ignored)
    #[inline]
        // Public function — callable from other modules.
pub fn set(&mut self, reg: Reg, value: u64) {
        if reg as u8 != 0 {
            self.regs[reg as usize] = value;
        }
    }

    /// Set comparison flags from two values
    #[inline]
        // Public function — callable from other modules.
pub fn set_cmp(&mut self, a: u64, b: u64) {
        self.cmp_a = a as i64;
        self.cmp_b = b as i64;
        self.cmp_a_u = a;
        self.cmp_b_u = b;
    }

    /// Evaluate a flag-based condition
    pub fn eval_condition(&self, condition: FlagCond) -> bool {
        let diff = self.cmp_a.wrapping_sub(self.cmp_b);
                // Pattern matching — Rust's exhaustive branching construct.
match condition {
            FlagCond::Eq    => self.cmp_a == self.cmp_b,
            FlagCond::Ne    => self.cmp_a != self.cmp_b,
            FlagCond::Lt    => self.cmp_a < self.cmp_b,
            FlagCond::Ge    => self.cmp_a >= self.cmp_b,
            FlagCond::Le    => self.cmp_a <= self.cmp_b,
            FlagCond::Gt    => self.cmp_a > self.cmp_b,
            FlagCond::Ltu   => self.cmp_a_u < self.cmp_b_u,
            FlagCond::Geu   => self.cmp_a_u >= self.cmp_b_u,
            FlagCond::Neg   => diff < 0,
            FlagCond::Pos   => diff >= 0,
            FlagCond::Ovf   => {
                // Overflow: signs of operands differ and result sign differs from first operand
                (self.cmp_a ^ self.cmp_b) < 0 && (self.cmp_a ^ diff) < 0
            }
            FlagCond::NoOvf => {
                !((self.cmp_a ^ self.cmp_b) < 0 && (self.cmp_a ^ diff) < 0)
            }
        }
    }
}

/// Emulated memory for translated processes
pub struct RvMemory {
    /// Memory regions: base_addr → data
    regions: BTreeMap<u64, Vec<u8>>,
    /// Total allocated bytes
    pub total_allocated: usize,
}

// Implementation block — defines methods for the type above.
impl RvMemory {
        // Public function — callable from other modules.
pub fn new() -> Self {
        Self {
            regions: BTreeMap::new(),
            total_allocated: 0,
        }
    }

    /// Map a memory region
    pub fn map(&mut self, address: u64, size: usize) {
        self.regions.insert(address, vec![0u8; size]);
        self.total_allocated += size;
    }

    /// Map a region with initial data
    pub fn map_with_data(&mut self, address: u64, data: &[u8]) {
        self.regions.insert(address, data.to_vec());
        self.total_allocated += data.len();
    }

    /// Read a byte
    pub fn read_u8(&self, address: u64) -> Result<u8, MemError> {
        for (&base, data) in &self.regions {
            if address >= base && address < base + data.len() as u64 {
                return Ok(data[(address - base) as usize]);
            }
        }
        Err(MemError::Unmapped(address))
    }

    /// Read 2 bytes (little-endian)
    pub fn read_u16(&self, address: u64) -> Result<u16, MemError> {
        Ok(u16::from_le_bytes([
            self.read_u8(address)?,
            self.read_u8(address + 1)?,
        ]))
    }

    /// Read 4 bytes (little-endian)
    pub fn read_u32(&self, address: u64) -> Result<u32, MemError> {
        Ok(u32::from_le_bytes([
            self.read_u8(address)?,
            self.read_u8(address + 1)?,
            self.read_u8(address + 2)?,
            self.read_u8(address + 3)?,
        ]))
    }

    /// Read 8 bytes (little-endian)
    pub fn read_u64(&self, address: u64) -> Result<u64, MemError> {
        Ok(u64::from_le_bytes([
            self.read_u8(address)?,
            self.read_u8(address + 1)?,
            self.read_u8(address + 2)?,
            self.read_u8(address + 3)?,
            self.read_u8(address + 4)?,
            self.read_u8(address + 5)?,
            self.read_u8(address + 6)?,
            self.read_u8(address + 7)?,
        ]))
    }

    /// Write a byte
    pub fn write_u8(&mut self, address: u64, value: u8) -> Result<(), MemError> {
        for (&base, data) in self.regions.iterator_mut() {
            if address >= base && address < base + data.len() as u64 {
                data[(address - base) as usize] = value;
                return Ok(());
            }
        }
        Err(MemError::Unmapped(address))
    }

    /// Write 2 bytes (little-endian)
    pub fn write_u16(&mut self, address: u64, value: u16) -> Result<(), MemError> {
        let bytes = value.to_le_bytes();
        self.write_u8(address, bytes[0])?;
        self.write_u8(address + 1, bytes[1])
    }

    /// Write 4 bytes (little-endian)
    pub fn write_u32(&mut self, address: u64, value: u32) -> Result<(), MemError> {
        let bytes = value.to_le_bytes();
        for i in 0..4 {
            self.write_u8(address + i, bytes[i as usize])?;
        }
        Ok(())
    }

    /// Write 8 bytes (little-endian)
    pub fn write_u64(&mut self, address: u64, value: u64) -> Result<(), MemError> {
        let bytes = value.to_le_bytes();
        for i in 0..8 {
            self.write_u8(address + i, bytes[i as usize])?;
        }
        Ok(())
    }

    /// Read a null-terminated string from memory
    pub fn read_string(&self, address: u64, maximum_length: usize) -> Result<String, MemError> {
        let mut s = String::new();
        for i in 0..maximum_length {
            let b = self.read_u8(address + i as u64)?;
            if b == 0 { break; }
            s.push(b as char);
        }
        Ok(s)
    }

    /// Write a string to memory (with null terminator)
    pub fn write_string(&mut self, address: u64, s: &str) -> Result<(), MemError> {
        for (i, b) in s.bytes().enumerate() {
            self.write_u8(address + i as u64, b)?;
        }
        self.write_u8(address + s.len() as u64, 0)
    }
}

/// Memory access error
#[derive(Debug)]
// Enumeration — a type that can be one of several variants.
pub enum MemError {
    Unmapped(u64),
}

/// Execution result from the interpreter
#[derive(Debug)]
// Enumeration — a type that can be one of several variants.
pub enum ExecResult {
    /// Continued execution normally
    Continue,
    /// Hit an ECALL (syscall) — caller should handle and resume
    Syscall {
        number: u64,
        args: [u64; 6],
    },
    /// Hit a breakpoint
    Breakpoint,
    /// Returned from the top-level function
    Returned(u64),
    /// Memory error
    MemoryFault(u64),
    /// Reached instruction limit
    InstructionLimit,
    /// Halted
    Halted,
}

/// The RISC-V IR interpreter
pub struct RvInterpreter {
    /// Virtual CPU state
    pub cpu: RvCpu,
    /// Virtual memory
    pub mem: RvMemory,
    /// Translation block cache: source_addr → translated block
    pub block_cache: BTreeMap<u64, Vec<RvInst>>,
    /// Maximum instructions to execute before yielding
    pub maximum_instructions: u64,
}

// Implementation block — defines methods for the type above.
impl RvInterpreter {
        // Public function — callable from other modules.
pub fn new() -> Self {
        Self {
            cpu: RvCpu::new(),
            mem: RvMemory::new(),
            block_cache: BTreeMap::new(),
            maximum_instructions: 10_000_000, // 10M instruction limit per run
        }
    }

    /// Load a translated block into the cache
    pub fn load_block(&mut self, block: &TranslatedBlock) {
        self.block_cache.insert(block.source_address, block.instructions.clone());
    }

    /// Load multiple blocks
    pub fn load_blocks(&mut self, blocks: &[TranslatedBlock]) {
        for block in blocks {
            self.load_block(block);
        }
    }

    /// Execute a single RISC-V IR instruction
    pub fn execute_one(&mut self, inst: &RvInst) -> ExecResult {
        self.cpu.inst_count += 1;

                // Pattern matching — Rust's exhaustive branching construct.
match inst {
            // === Register-Register ALU ===
            RvInst::Add { rd, rs1, rs2 } => {
                self.cpu.set(*rd, self.cpu.get(*rs1).wrapping_add(self.cpu.get(*rs2)));
            }
            RvInst::Sub { rd, rs1, rs2 } => {
                self.cpu.set(*rd, self.cpu.get(*rs1).wrapping_sub(self.cpu.get(*rs2)));
            }
            RvInst::And { rd, rs1, rs2 } => {
                self.cpu.set(*rd, self.cpu.get(*rs1) & self.cpu.get(*rs2));
            }
            RvInst::Or { rd, rs1, rs2 } => {
                self.cpu.set(*rd, self.cpu.get(*rs1) | self.cpu.get(*rs2));
            }
            RvInst::Xor { rd, rs1, rs2 } => {
                self.cpu.set(*rd, self.cpu.get(*rs1) ^ self.cpu.get(*rs2));
            }
            RvInst::Sll { rd, rs1, rs2 } => {
                let shamt = self.cpu.get(*rs2) & 63;
                self.cpu.set(*rd, self.cpu.get(*rs1) << shamt);
            }
            RvInst::Srl { rd, rs1, rs2 } => {
                let shamt = self.cpu.get(*rs2) & 63;
                self.cpu.set(*rd, self.cpu.get(*rs1) >> shamt);
            }
            RvInst::Sra { rd, rs1, rs2 } => {
                let shamt = self.cpu.get(*rs2) & 63;
                self.cpu.set(*rd, ((self.cpu.get(*rs1) as i64) >> shamt) as u64);
            }
            RvInst::Slt { rd, rs1, rs2 } => {
                let v = if (self.cpu.get(*rs1) as i64) < (self.cpu.get(*rs2) as i64) { 1 } else { 0 };
                self.cpu.set(*rd, v);
            }
            RvInst::Sltu { rd, rs1, rs2 } => {
                let v = if self.cpu.get(*rs1) < self.cpu.get(*rs2) { 1 } else { 0 };
                self.cpu.set(*rd, v);
            }

            // === M extension ===
            RvInst::Mul { rd, rs1, rs2 } => {
                self.cpu.set(*rd, self.cpu.get(*rs1).wrapping_mul(self.cpu.get(*rs2)));
            }
            RvInst::Mulh { rd, rs1, rs2 } => {
                let a = self.cpu.get(*rs1) as i64 as i128;
                let b = self.cpu.get(*rs2) as i64 as i128;
                self.cpu.set(*rd, ((a * b) >> 64) as u64);
            }
            RvInst::Div { rd, rs1, rs2 } => {
                let b = self.cpu.get(*rs2) as i64;
                if b == 0 {
                    self.cpu.set(*rd, u64::MAX); // Division by zero → -1
                } else {
                    self.cpu.set(*rd, ((self.cpu.get(*rs1) as i64).wrapping_div(b)) as u64);
                }
            }
            RvInst::Divu { rd, rs1, rs2 } => {
                let b = self.cpu.get(*rs2);
                if b == 0 {
                    self.cpu.set(*rd, u64::MAX);
                } else {
                    self.cpu.set(*rd, self.cpu.get(*rs1) / b);
                }
            }
            RvInst::Rem { rd, rs1, rs2 } => {
                let b = self.cpu.get(*rs2) as i64;
                if b == 0 {
                    self.cpu.set(*rd, self.cpu.get(*rs1));
                } else {
                    self.cpu.set(*rd, ((self.cpu.get(*rs1) as i64).wrapping_rem(b)) as u64);
                }
            }
            RvInst::Remu { rd, rs1, rs2 } => {
                let b = self.cpu.get(*rs2);
                if b == 0 {
                    self.cpu.set(*rd, self.cpu.get(*rs1));
                } else {
                    self.cpu.set(*rd, self.cpu.get(*rs1) % b);
                }
            }

            // === Register-Immediate ALU ===
            RvInst::Addi { rd, rs1, imm } => {
                self.cpu.set(*rd, self.cpu.get(*rs1).wrapping_add(*imm as u64));
            }
            RvInst::Andi { rd, rs1, imm } => {
                self.cpu.set(*rd, self.cpu.get(*rs1) & (*imm as u64));
            }
            RvInst::Ori { rd, rs1, imm } => {
                self.cpu.set(*rd, self.cpu.get(*rs1) | (*imm as u64));
            }
            RvInst::Xori { rd, rs1, imm } => {
                self.cpu.set(*rd, self.cpu.get(*rs1) ^ (*imm as u64));
            }
            RvInst::Slli { rd, rs1, shamt } => {
                self.cpu.set(*rd, self.cpu.get(*rs1) << (*shamt & 63));
            }
            RvInst::Srli { rd, rs1, shamt } => {
                self.cpu.set(*rd, self.cpu.get(*rs1) >> (*shamt & 63));
            }
            RvInst::Srai { rd, rs1, shamt } => {
                self.cpu.set(*rd, ((self.cpu.get(*rs1) as i64) >> (*shamt & 63)) as u64);
            }
            RvInst::Slti { rd, rs1, imm } => {
                let v = if (self.cpu.get(*rs1) as i64) < *imm { 1 } else { 0 };
                self.cpu.set(*rd, v);
            }
            RvInst::Sltiu { rd, rs1, imm } => {
                let v = if self.cpu.get(*rs1) < (*imm as u64) { 1 } else { 0 };
                self.cpu.set(*rd, v);
            }

            // === Upper Immediate ===
            RvInst::Lui { rd, imm } => {
                self.cpu.set(*rd, ((*imm as u64) << 12) & 0xFFFF_FFFF_FFFF_F000);
            }
            RvInst::Auipc { rd, imm } => {
                self.cpu.set(*rd, self.cpu.pc.wrapping_add((*imm as u64) << 12));
            }

            // === Load ===
            RvInst::Lb { rd, rs1, offset } => {
                let address = self.cpu.get(*rs1).wrapping_add(*offset as u64);
                                // Pattern matching — Rust's exhaustive branching construct.
match self.mem.read_u8(address) {
                    Ok(v) => self.cpu.set(*rd, v as i8 as i64 as u64),
                    Err(_) => return ExecResult::MemoryFault(address),
                }
            }
            RvInst::Lbu { rd, rs1, offset } => {
                let address = self.cpu.get(*rs1).wrapping_add(*offset as u64);
                                // Pattern matching — Rust's exhaustive branching construct.
match self.mem.read_u8(address) {
                    Ok(v) => self.cpu.set(*rd, v as u64),
                    Err(_) => return ExecResult::MemoryFault(address),
                }
            }
            RvInst::Lh { rd, rs1, offset } => {
                let address = self.cpu.get(*rs1).wrapping_add(*offset as u64);
                                // Pattern matching — Rust's exhaustive branching construct.
match self.mem.read_u16(address) {
                    Ok(v) => self.cpu.set(*rd, v as i16 as i64 as u64),
                    Err(_) => return ExecResult::MemoryFault(address),
                }
            }
            RvInst::Lhu { rd, rs1, offset } => {
                let address = self.cpu.get(*rs1).wrapping_add(*offset as u64);
                                // Pattern matching — Rust's exhaustive branching construct.
match self.mem.read_u16(address) {
                    Ok(v) => self.cpu.set(*rd, v as u64),
                    Err(_) => return ExecResult::MemoryFault(address),
                }
            }
            RvInst::Lw { rd, rs1, offset } => {
                let address = self.cpu.get(*rs1).wrapping_add(*offset as u64);
                                // Pattern matching — Rust's exhaustive branching construct.
match self.mem.read_u32(address) {
                    Ok(v) => self.cpu.set(*rd, v as i32 as i64 as u64),
                    Err(_) => return ExecResult::MemoryFault(address),
                }
            }
            RvInst::Lwu { rd, rs1, offset } => {
                let address = self.cpu.get(*rs1).wrapping_add(*offset as u64);
                                // Pattern matching — Rust's exhaustive branching construct.
match self.mem.read_u32(address) {
                    Ok(v) => self.cpu.set(*rd, v as u64),
                    Err(_) => return ExecResult::MemoryFault(address),
                }
            }
            RvInst::Ld { rd, rs1, offset } => {
                let address = self.cpu.get(*rs1).wrapping_add(*offset as u64);
                                // Pattern matching — Rust's exhaustive branching construct.
match self.mem.read_u64(address) {
                    Ok(v) => self.cpu.set(*rd, v),
                    Err(_) => return ExecResult::MemoryFault(address),
                }
            }

            // === Store ===
            RvInst::Sb { rs2, rs1, offset } => {
                let address = self.cpu.get(*rs1).wrapping_add(*offset as u64);
                let value = self.cpu.get(*rs2) as u8;
                if self.mem.write_u8(address, value).is_err() {
                    return ExecResult::MemoryFault(address);
                }
            }
            RvInst::Sh { rs2, rs1, offset } => {
                let address = self.cpu.get(*rs1).wrapping_add(*offset as u64);
                let value = self.cpu.get(*rs2) as u16;
                if self.mem.write_u16(address, value).is_err() {
                    return ExecResult::MemoryFault(address);
                }
            }
            RvInst::Software { rs2, rs1, offset } => {
                let address = self.cpu.get(*rs1).wrapping_add(*offset as u64);
                let value = self.cpu.get(*rs2) as u32;
                if self.mem.write_u32(address, value).is_err() {
                    return ExecResult::MemoryFault(address);
                }
            }
            RvInst::Sd { rs2, rs1, offset } => {
                let address = self.cpu.get(*rs1).wrapping_add(*offset as u64);
                let value = self.cpu.get(*rs2);
                if self.mem.write_u64(address, value).is_err() {
                    return ExecResult::MemoryFault(address);
                }
            }

            // === Branches ===
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

            // === Jump ===
            RvInst::Jal { rd, offset } => {
                // Store return address
                self.cpu.set(*rd, self.cpu.pc);
                self.cpu.pc = *offset as u64;
                return ExecResult::Continue;
            }
            RvInst::Jalr { rd, rs1, offset } => {
                let target = self.cpu.get(*rs1).wrapping_add(*offset as u64) & !1;
                self.cpu.set(*rd, self.cpu.pc);
                self.cpu.pc = target;
                return ExecResult::Continue;
            }

            // === System ===
            RvInst::Ecall => {
                let number = self.cpu.get(Reg::X17); // a7
                let args = [
                    self.cpu.get(Reg::X10), // a0
                    self.cpu.get(Reg::X11), // a1
                    self.cpu.get(Reg::X12), // a2
                    self.cpu.get(Reg::X13), // a3
                    self.cpu.get(Reg::X14), // a4
                    self.cpu.get(Reg::X15), // a5
                ];
                return ExecResult::Syscall { number, args };
            }
            RvInst::Ebreak => {
                return ExecResult::Breakpoint;
            }
            RvInst::Fence => {
                // Memory fence — no-op in interpreter
            }

            // === Atomics ===
            RvInst::AmoswapD { rd, rs2, rs1 } => {
                let address = self.cpu.get(*rs1);
                                // Pattern matching — Rust's exhaustive branching construct.
match self.mem.read_u64(address) {
                    Ok(old) => {
                        self.cpu.set(*rd, old);
                        let _ = self.mem.write_u64(address, self.cpu.get(*rs2));
                    }
                    Err(_) => return ExecResult::MemoryFault(address),
                }
            }
            RvInst::AmoaddD { rd, rs2, rs1 } => {
                let address = self.cpu.get(*rs1);
                                // Pattern matching — Rust's exhaustive branching construct.
match self.mem.read_u64(address) {
                    Ok(old) => {
                        self.cpu.set(*rd, old);
                        let new = old.wrapping_add(self.cpu.get(*rs2));
                        let _ = self.mem.write_u64(address, new);
                    }
                    Err(_) => return ExecResult::MemoryFault(address),
                }
            }

            // === Pseudo-instructions ===
            RvInst::Li { rd, imm } => {
                self.cpu.set(*rd, *imm as u64);
            }
            RvInst::Mv { rd, rs } => {
                self.cpu.set(*rd, self.cpu.get(*rs));
            }
            RvInst::Nop => {}
            RvInst::Return_value => {
                let ra = self.cpu.get(Reg::X1);
                if ra == 0 {
                    // Return from top-level
                    return ExecResult::Returned(self.cpu.get(Reg::X10));
                }
                self.cpu.pc = ra;
                return ExecResult::Continue;
            }
            RvInst::Call { offset } => {
                self.cpu.set(Reg::X1, self.cpu.pc);
                self.cpu.pc = *offset as u64;
                return ExecResult::Continue;
            }

            // === Translation extensions ===
            RvInst::CmpFlags { rs1, rs2 } => {
                self.cpu.set_cmp(self.cpu.get(*rs1), self.cpu.get(*rs2));
            }
            RvInst::BranchCondition { condition, offset } => {
                if self.cpu.eval_condition(*condition) {
                    self.cpu.pc = *offset as u64;
                    return ExecResult::Continue;
                }
            }
            RvInst::SourceAnnotation { .. } => {
                // Debug annotation — no execution effect
            }
        }

        ExecResult::Continue
    }

    /// Execute a sequence of instructions from a block
    pub fn execute_block(&mut self, instructions: &[RvInst]) -> ExecResult {
        let mut ip = 0;
        while ip < instructions.len() {
            if self.cpu.inst_count >= self.maximum_instructions {
                return ExecResult::InstructionLimit;
            }

            let result = self.execute_one(&instructions[ip]);
            ip += 1;

                        // Pattern matching — Rust's exhaustive branching construct.
match result {
                ExecResult::Continue => {
                    // Check if pc was changed (branch taken)
                    // If so, we need to look up the new block
                    // For now, continue linear execution within the block
                }
                other => return other,
            }
        }

        ExecResult::Continue
    }

    /// Execute starting from a source address, using cached blocks
    pub fn run_from(&mut self, start_address: u64) -> ExecResult {
        self.cpu.pc = start_address;

                // Infinite loop — runs until an explicit `break`.
loop {
            if self.cpu.inst_count >= self.maximum_instructions {
                return ExecResult::InstructionLimit;
            }

            // Look up cached block for current PC
            if let Some(block_insts) = self.block_cache.get(&self.cpu.pc).cloned() {
                let old_pc = self.cpu.pc;
                let result = self.execute_block(&block_insts);

                                // Pattern matching — Rust's exhaustive branching construct.
match result {
                    ExecResult::Continue => {
                        // If PC didn't change, we fell through — advance to next block
                        if self.cpu.pc == old_pc {
                            // No block at this address — execution complete
                            return ExecResult::Returned(self.cpu.get(Reg::X10));
                        }
                        // Otherwise, loop continues with new PC
                    }
                    other => return other,
                }
            } else {
                // No translation available for this address
                return ExecResult::Returned(self.cpu.get(Reg::X10));
            }
        }
    }

    /// Dump CPU state (for debugging)
    pub fn dump_state(&self) -> String {
        let mut s = String::from("=== RISC-V IR CPU State ===\n");
        for i in 0..32 {
            let reg = Reg::from_index(i);
            let value = self.cpu.get(reg);
            if value != 0 {
                s.push_str(&format!("  {:4} (x{:2}) = 0x{:016X} ({})\n",
                    reg.abi_name(), i, value, value as i64));
            }
        }
        s.push_str(&format!("  pc = 0x{:016X}\n", self.cpu.pc));
        s.push_str(&format!("  instructions executed: {}\n", self.cpu.inst_count));
        s
    }
}
