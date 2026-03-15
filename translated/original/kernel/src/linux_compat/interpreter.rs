//! x86_64 Instruction Interpreter
//!
//! Interprets x86_64 instructions to run Linux binaries without hardware virtualization.
//! This is similar to how QEMU's TCG works, but simpler and focused on userspace code.

use alloc::vec::Vec;
use alloc::string::String;
use alloc::collections::BTreeMap;

/// CPU state for the interpreted process
#[derive(Debug, Clone)]
pub struct CpuState {
    // General purpose registers
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
    
    // Instruction pointer
    pub rip: u64,
    
    // Flags register
    pub rflags: u64,
    
    // Segment registers (simplified)
    pub cs: u16,
    pub ds: u16,
    pub es: u16,
    pub fs: u64,  // FS base for TLS
    pub gs: u64,  // GS base
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
            rflags: 0x202, // IF set
            cs: 0x33, ds: 0x2b, es: 0x2b,
            fs: 0, gs: 0, ss: 0x2b,
        }
    }
    
    /// Get register by index (used for ModR/M decoding)
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
    
    /// Set register by index
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

/// Flag bits
pub const FLAG_CF: u64 = 1 << 0;  // Carry
pub const FLAG_PF: u64 = 1 << 2;  // Parity
pub const FLAG_AF: u64 = 1 << 4;  // Auxiliary
pub const FLAG_ZF: u64 = 1 << 6;  // Zero
pub const FLAG_SF: u64 = 1 << 7;  // Sign
pub const FLAG_OF: u64 = 1 << 11; // Overflow

/// Memory for the interpreted process
pub struct ProcessMemory {
    /// Memory regions: base_addr -> (data, permissions)
    regions: BTreeMap<u64, MemoryRegion>,
    /// Break pointer (heap end)
    brk: u64,
}

struct MemoryRegion {
    data: Vec<u8>,
    readable: bool,
    writable: bool,
    executable: bool,
}

impl ProcessMemory {
    pub fn new() -> Self {
        Self {
            regions: BTreeMap::new(),
            brk: 0x1000_0000, // Initial brk
        }
    }
    
    /// Map a memory region
    pub fn map(&mut self, addr: u64, size: usize, r: bool, w: bool, x: bool) {
        self.regions.insert(addr, MemoryRegion {
            data: alloc::vec![0u8; size],
            readable: r,
            writable: w,
            executable: x,
        });
    }
    
    /// Write data to memory
    pub fn write(&mut self, addr: u64, data: &[u8]) -> Result<(), &'static str> {
        for (region_base, region) in self.regions.iter_mut() {
            let region_end = *region_base + region.data.len() as u64;
            if addr >= *region_base && addr < region_end {
                if !region.writable {
                    return Err("Write to non-writable memory");
                }
                let offset = (addr - *region_base) as usize;
                let copy_len = core::cmp::min(data.len(), region.data.len() - offset);
                region.data[offset..offset + copy_len].copy_from_slice(&data[..copy_len]);
                return Ok(());
            }
        }
        Err("Write to unmapped memory")
    }
    
    /// Read data from memory
    pub fn read(&self, addr: u64, len: usize) -> Result<Vec<u8>, &'static str> {
        for (region_base, region) in self.regions.iter() {
            let region_end = *region_base + region.data.len() as u64;
            if addr >= *region_base && addr < region_end {
                if !region.readable {
                    return Err("Read from non-readable memory");
                }
                let offset = (addr - *region_base) as usize;
                let copy_len = core::cmp::min(len, region.data.len() - offset);
                return Ok(region.data[offset..offset + copy_len].to_vec());
            }
        }
        Err("Read from unmapped memory")
    }
    
    /// Read a single byte
    pub fn read_u8(&self, addr: u64) -> Result<u8, &'static str> {
        let data = self.read(addr, 1)?;
        Ok(data[0])
    }
    
    /// Read u16
    pub fn read_u16(&self, addr: u64) -> Result<u16, &'static str> {
        let data = self.read(addr, 2)?;
        Ok(u16::from_le_bytes([data[0], data[1]]))
    }
    
    /// Read u32
    pub fn read_u32(&self, addr: u64) -> Result<u32, &'static str> {
        let data = self.read(addr, 4)?;
        Ok(u32::from_le_bytes([data[0], data[1], data[2], data[3]]))
    }
    
    /// Read u64
    pub fn read_u64(&self, addr: u64) -> Result<u64, &'static str> {
        let data = self.read(addr, 8)?;
        Ok(u64::from_le_bytes([
            data[0], data[1], data[2], data[3],
            data[4], data[5], data[6], data[7],
        ]))
    }
    
    /// Write u8
    pub fn write_u8(&mut self, addr: u64, val: u8) -> Result<(), &'static str> {
        self.write(addr, &[val])
    }
    
    /// Write u64
    pub fn write_u64(&mut self, addr: u64, val: u64) -> Result<(), &'static str> {
        self.write(addr, &val.to_le_bytes())
    }
    
    /// Get/set brk
    pub fn brk(&self) -> u64 { self.brk }
    pub fn set_brk(&mut self, new_brk: u64) { self.brk = new_brk; }
}

/// Instruction decoder result
pub enum DecodeResult {
    /// Continue execution
    Continue,
    /// Syscall instruction encountered
    Syscall,
    /// Exit requested
    Exit(i32),
    /// Error
    Error(&'static str),
}

/// The instruction interpreter
pub struct Interpreter {
    pub cpu: CpuState,
    pub memory: ProcessMemory,
    /// Open file descriptors
    pub fds: BTreeMap<i32, FileDescriptor>,
    /// Next available fd
    next_fd: i32,
    /// Process ID
    pub pid: u32,
    /// Working directory
    pub cwd: String,
    /// Arguments
    pub argv: Vec<String>,
    /// Environment
    pub envp: Vec<String>,
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
            envp: Vec::new(),
        }
    }
    
    /// Load an ELF binary into memory
    pub fn load_elf(&mut self, elf_data: &[u8]) -> Result<u64, &'static str> {
        // Check ELF magic
        if elf_data.len() < 64 {
            return Err("ELF too small");
        }
        if &elf_data[0..4] != b"\x7fELF" {
            return Err("Not an ELF file");
        }
        if elf_data[4] != 2 {
            return Err("Not 64-bit ELF");
        }
        if elf_data[5] != 1 {
            return Err("Not little-endian");
        }
        
        // Parse ELF header
        let e_type = u16::from_le_bytes([elf_data[16], elf_data[17]]);
        if e_type != 2 && e_type != 3 {
            return Err("Not executable or shared object");
        }
        
        let e_entry = u64::from_le_bytes([
            elf_data[24], elf_data[25], elf_data[26], elf_data[27],
            elf_data[28], elf_data[29], elf_data[30], elf_data[31],
        ]);
        
        let e_phoff = u64::from_le_bytes([
            elf_data[32], elf_data[33], elf_data[34], elf_data[35],
            elf_data[36], elf_data[37], elf_data[38], elf_data[39],
        ]) as usize;
        
        let e_phentsize = u16::from_le_bytes([elf_data[54], elf_data[55]]) as usize;
        let e_phnum = u16::from_le_bytes([elf_data[56], elf_data[57]]) as usize;
        
        // Load program headers
        for i in 0..e_phnum {
            let ph_off = e_phoff + i * e_phentsize;
            if ph_off + 56 > elf_data.len() {
                continue;
            }
            
            let p_type = u32::from_le_bytes([
                elf_data[ph_off], elf_data[ph_off + 1],
                elf_data[ph_off + 2], elf_data[ph_off + 3],
            ]);
            
            // PT_LOAD = 1
            if p_type != 1 {
                continue;
            }
            
            let p_offset = u64::from_le_bytes([
                elf_data[ph_off + 8], elf_data[ph_off + 9],
                elf_data[ph_off + 10], elf_data[ph_off + 11],
                elf_data[ph_off + 12], elf_data[ph_off + 13],
                elf_data[ph_off + 14], elf_data[ph_off + 15],
            ]) as usize;
            
            let p_vaddr = u64::from_le_bytes([
                elf_data[ph_off + 16], elf_data[ph_off + 17],
                elf_data[ph_off + 18], elf_data[ph_off + 19],
                elf_data[ph_off + 20], elf_data[ph_off + 21],
                elf_data[ph_off + 22], elf_data[ph_off + 23],
            ]);
            
            let p_filesz = u64::from_le_bytes([
                elf_data[ph_off + 32], elf_data[ph_off + 33],
                elf_data[ph_off + 34], elf_data[ph_off + 35],
                elf_data[ph_off + 36], elf_data[ph_off + 37],
                elf_data[ph_off + 38], elf_data[ph_off + 39],
            ]) as usize;
            
            let p_memsz = u64::from_le_bytes([
                elf_data[ph_off + 40], elf_data[ph_off + 41],
                elf_data[ph_off + 42], elf_data[ph_off + 43],
                elf_data[ph_off + 44], elf_data[ph_off + 45],
                elf_data[ph_off + 46], elf_data[ph_off + 47],
            ]) as usize;
            
            let p_flags = u32::from_le_bytes([
                elf_data[ph_off + 4], elf_data[ph_off + 5],
                elf_data[ph_off + 6], elf_data[ph_off + 7],
            ]);
            
            // Map segment
            let readable = (p_flags & 4) != 0;
            let writable = (p_flags & 2) != 0;
            let executable = (p_flags & 1) != 0;
            
            self.memory.map(p_vaddr, p_memsz, readable, writable, executable);
            
            // Copy data
            if p_offset + p_filesz <= elf_data.len() {
                let _ = self.memory.write(p_vaddr, &elf_data[p_offset..p_offset + p_filesz]);
            }
        }
        
        // Setup stack (8MB at high address)
        let stack_base = 0x7FFF_0000_0000u64;
        let stack_size = 8 * 1024 * 1024;
        self.memory.map(stack_base - stack_size as u64, stack_size, true, true, false);
        self.cpu.rsp = stack_base - 8;
        
        // Set entry point
        self.cpu.rip = e_entry;
        
        Ok(e_entry)
    }
    
    /// Setup argv and envp on stack
    pub fn setup_stack(&mut self, argv: &[&str], envp: &[&str]) -> Result<(), &'static str> {
        // Store strings and build pointer arrays
        let mut string_ptrs: Vec<u64> = Vec::new();
        let mut env_ptrs: Vec<u64> = Vec::new();
        
        // Push strings onto stack (going down)
        for arg in argv.iter().rev() {
            self.cpu.rsp -= (arg.len() + 1) as u64;
            let _ = self.memory.write(self.cpu.rsp, arg.as_bytes());
            let _ = self.memory.write_u8(self.cpu.rsp + arg.len() as u64, 0);
            string_ptrs.push(self.cpu.rsp);
        }
        string_ptrs.reverse();
        
        for env in envp.iter().rev() {
            self.cpu.rsp -= (env.len() + 1) as u64;
            let _ = self.memory.write(self.cpu.rsp, env.as_bytes());
            let _ = self.memory.write_u8(self.cpu.rsp + env.len() as u64, 0);
            env_ptrs.push(self.cpu.rsp);
        }
        env_ptrs.reverse();
        
        // Align stack to 16 bytes
        self.cpu.rsp &= !15;
        
        // Push null terminator for envp
        self.cpu.rsp -= 8;
        let _ = self.memory.write_u64(self.cpu.rsp, 0);
        
        // Push envp pointers
        for ptr in env_ptrs.iter().rev() {
            self.cpu.rsp -= 8;
            let _ = self.memory.write_u64(self.cpu.rsp, *ptr);
        }
        
        // Push null terminator for argv
        self.cpu.rsp -= 8;
        let _ = self.memory.write_u64(self.cpu.rsp, 0);
        
        // Push argv pointers
        for ptr in string_ptrs.iter().rev() {
            self.cpu.rsp -= 8;
            let _ = self.memory.write_u64(self.cpu.rsp, *ptr);
        }
        
        // Push argc
        self.cpu.rsp -= 8;
        let _ = self.memory.write_u64(self.cpu.rsp, argv.len() as u64);
        
        Ok(())
    }
    
    /// Execute one instruction
    pub fn step(&mut self) -> DecodeResult {
        // Fetch instruction bytes
        let inst_bytes = match self.memory.read(self.cpu.rip, 16) {
            Ok(b) => b,
            Err(_) => return DecodeResult::Error("Failed to fetch instruction"),
        };
        
        let mut idx = 0;
        
        // Parse prefixes
        let mut rex: u8 = 0;
        let mut has_66_prefix = false;
        
        loop {
            if idx >= inst_bytes.len() {
                return DecodeResult::Error("Instruction too long");
            }
            
            match inst_bytes[idx] {
                0x40..=0x4F => {
                    rex = inst_bytes[idx];
                    idx += 1;
                }
                0x66 => {
                    has_66_prefix = true;
                    idx += 1;
                }
                0xF0 | 0xF2 | 0xF3 | 0x26 | 0x2E | 0x36 | 0x3E | 0x64 | 0x65 => {
                    // Other prefixes - skip for now
                    idx += 1;
                }
                _ => break,
            }
        }
        
        let rex_w = (rex & 0x08) != 0;
        let rex_r = (rex & 0x04) != 0;
        let rex_x = (rex & 0x02) != 0;
        let rex_b = (rex & 0x01) != 0;
        
        let opcode = inst_bytes[idx];
        idx += 1;
        
        match opcode {
            // SYSCALL
            0x0F if inst_bytes.get(idx) == Some(&0x05) => {
                self.cpu.rip += (idx + 1) as u64;
                return DecodeResult::Syscall;
            }
            
            // NOP
            0x90 => {
                self.cpu.rip += idx as u64;
            }
            
            // RET
            0xC3 => {
                let ret_addr = self.memory.read_u64(self.cpu.rsp).unwrap_or(0);
                self.cpu.rsp += 8;
                self.cpu.rip = ret_addr;
                if ret_addr == 0 {
                    return DecodeResult::Exit(self.cpu.rax as i32);
                }
            }
            
            // PUSH r64
            0x50..=0x57 => {
                let reg_idx = (opcode - 0x50) + if rex_b { 8 } else { 0 };
                let val = self.cpu.get_reg(reg_idx);
                self.cpu.rsp -= 8;
                let _ = self.memory.write_u64(self.cpu.rsp, val);
                self.cpu.rip += idx as u64;
            }
            
            // POP r64
            0x58..=0x5F => {
                let reg_idx = (opcode - 0x58) + if rex_b { 8 } else { 0 };
                let val = self.memory.read_u64(self.cpu.rsp).unwrap_or(0);
                self.cpu.rsp += 8;
                self.cpu.set_reg(reg_idx, val);
                self.cpu.rip += idx as u64;
            }
            
            // MOV r64, imm64 (REX.W B8+rd)
            0xB8..=0xBF if rex_w => {
                let reg_idx = (opcode - 0xB8) + if rex_b { 8 } else { 0 };
                if idx + 8 <= inst_bytes.len() {
                    let imm = u64::from_le_bytes([
                        inst_bytes[idx], inst_bytes[idx + 1],
                        inst_bytes[idx + 2], inst_bytes[idx + 3],
                        inst_bytes[idx + 4], inst_bytes[idx + 5],
                        inst_bytes[idx + 6], inst_bytes[idx + 7],
                    ]);
                    self.cpu.set_reg(reg_idx, imm);
                    self.cpu.rip += (idx + 8) as u64;
                } else {
                    return DecodeResult::Error("MOV imm64 truncated");
                }
            }
            
            // MOV r32, imm32
            0xB8..=0xBF => {
                let reg_idx = (opcode - 0xB8) + if rex_b { 8 } else { 0 };
                if idx + 4 <= inst_bytes.len() {
                    let imm = u32::from_le_bytes([
                        inst_bytes[idx], inst_bytes[idx + 1],
                        inst_bytes[idx + 2], inst_bytes[idx + 3],
                    ]);
                    self.cpu.set_reg(reg_idx, imm as u64);
                    self.cpu.rip += (idx + 4) as u64;
                } else {
                    return DecodeResult::Error("MOV imm32 truncated");
                }
            }
            
            // XOR r/m64, r64 (31 /r)
            0x31 if rex_w => {
                if idx >= inst_bytes.len() {
                    return DecodeResult::Error("XOR missing modrm");
                }
                let modrm = inst_bytes[idx];
                idx += 1;
                
                let mod_field = (modrm >> 6) & 3;
                let reg = ((modrm >> 3) & 7) + if rex_r { 8 } else { 0 };
                let rm = (modrm & 7) + if rex_b { 8 } else { 0 };
                
                if mod_field == 3 {
                    // Register-register
                    let src = self.cpu.get_reg(reg);
                    let dst = self.cpu.get_reg(rm);
                    let result = dst ^ src;
                    self.cpu.set_reg(rm, result);
                    self.update_flags_logic(result);
                }
                self.cpu.rip += idx as u64;
            }
            
            // XOR r32, r/m32
            0x31 => {
                if idx >= inst_bytes.len() {
                    return DecodeResult::Error("XOR missing modrm");
                }
                let modrm = inst_bytes[idx];
                idx += 1;
                
                let mod_field = (modrm >> 6) & 3;
                let reg = ((modrm >> 3) & 7) + if rex_r { 8 } else { 0 };
                let rm = (modrm & 7) + if rex_b { 8 } else { 0 };
                
                if mod_field == 3 {
                    let src = self.cpu.get_reg(reg) as u32;
                    let dst = self.cpu.get_reg(rm) as u32;
                    let result = dst ^ src;
                    self.cpu.set_reg(rm, result as u64);
                    self.update_flags_logic(result as u64);
                }
                self.cpu.rip += idx as u64;
            }
            
            // CALL rel32
            0xE8 => {
                if idx + 4 > inst_bytes.len() {
                    return DecodeResult::Error("CALL truncated");
                }
                let rel = i32::from_le_bytes([
                    inst_bytes[idx], inst_bytes[idx + 1],
                    inst_bytes[idx + 2], inst_bytes[idx + 3],
                ]);
                let next_rip = self.cpu.rip + (idx + 4) as u64;
                self.cpu.rsp -= 8;
                let _ = self.memory.write_u64(self.cpu.rsp, next_rip);
                self.cpu.rip = (next_rip as i64 + rel as i64) as u64;
            }
            
            // JMP rel32
            0xE9 => {
                if idx + 4 > inst_bytes.len() {
                    return DecodeResult::Error("JMP truncated");
                }
                let rel = i32::from_le_bytes([
                    inst_bytes[idx], inst_bytes[idx + 1],
                    inst_bytes[idx + 2], inst_bytes[idx + 3],
                ]);
                let next_rip = self.cpu.rip + (idx + 4) as u64;
                self.cpu.rip = (next_rip as i64 + rel as i64) as u64;
            }
            
            // JMP rel8
            0xEB => {
                if idx >= inst_bytes.len() {
                    return DecodeResult::Error("JMP rel8 truncated");
                }
                let rel = inst_bytes[idx] as i8;
                let next_rip = self.cpu.rip + (idx + 1) as u64;
                self.cpu.rip = (next_rip as i64 + rel as i64) as u64;
            }
            
            // Conditional jumps (JCC rel8)
            0x70..=0x7F => {
                if idx >= inst_bytes.len() {
                    return DecodeResult::Error("Jcc truncated");
                }
                let rel = inst_bytes[idx] as i8;
                let cond = opcode & 0x0F;
                let next_rip = self.cpu.rip + (idx + 1) as u64;
                
                if self.check_condition(cond) {
                    self.cpu.rip = (next_rip as i64 + rel as i64) as u64;
                } else {
                    self.cpu.rip = next_rip;
                }
            }
            
            // INT3 (breakpoint)
            0xCC => {
                crate::serial_println!("[INTERP] INT3 at 0x{:x}", self.cpu.rip);
                self.cpu.rip += idx as u64;
            }
            
            // HLT
            0xF4 => {
                return DecodeResult::Exit(0);
            }
            
            _ => {
                crate::serial_println!(
                    "[INTERP] Unknown opcode 0x{:02x} at RIP=0x{:x}",
                    opcode, self.cpu.rip
                );
                // Skip unknown instruction
                self.cpu.rip += idx as u64;
            }
        }
        
        DecodeResult::Continue
    }
    
    /// Update flags for logical operations
    fn update_flags_logic(&mut self, result: u64) {
        self.cpu.rflags &= !(FLAG_CF | FLAG_OF | FLAG_SF | FLAG_ZF | FLAG_PF);
        
        if result == 0 {
            self.cpu.rflags |= FLAG_ZF;
        }
        if (result as i64) < 0 {
            self.cpu.rflags |= FLAG_SF;
        }
        // Parity (of low byte)
        let parity = (result as u8).count_ones();
        if parity % 2 == 0 {
            self.cpu.rflags |= FLAG_PF;
        }
    }
    
    /// Check conditional jump condition
    fn check_condition(&self, cond: u8) -> bool {
        let cf = (self.cpu.rflags & FLAG_CF) != 0;
        let zf = (self.cpu.rflags & FLAG_ZF) != 0;
        let sf = (self.cpu.rflags & FLAG_SF) != 0;
        let of = (self.cpu.rflags & FLAG_OF) != 0;
        let pf = (self.cpu.rflags & FLAG_PF) != 0;
        
        match cond {
            0x0 => of,           // JO
            0x1 => !of,          // JNO
            0x2 => cf,           // JB/JNAE/JC
            0x3 => !cf,          // JNB/JAE/JNC
            0x4 => zf,           // JE/JZ
            0x5 => !zf,          // JNE/JNZ
            0x6 => cf || zf,     // JBE/JNA
            0x7 => !cf && !zf,   // JNBE/JA
            0x8 => sf,           // JS
            0x9 => !sf,          // JNS
            0xA => pf,           // JP/JPE
            0xB => !pf,          // JNP/JPO
            0xC => sf != of,     // JL/JNGE
            0xD => sf == of,     // JNL/JGE
            0xE => zf || (sf != of), // JLE/JNG
            0xF => !zf && (sf == of), // JNLE/JG
            _ => false,
        }
    }
    
    /// Run until exit or error
    pub fn run(&mut self) -> Result<i32, &'static str> {
        let mut steps = 0u64;
        let max_steps = 1_000_000u64; // 1M instructions max (reduced for faster feedback)
        let report_interval = 100_000u64;
        
        loop {
            match self.step() {
                DecodeResult::Continue => {
                    steps += 1;
                    if steps % report_interval == 0 {
                        crate::print!(".");  // Progress indicator
                    }
                    if steps > max_steps {
                        crate::println!();
                        crate::println!("Executed {} instructions", steps);
                        return Err("Timeout: binary too complex for interpreter");
                    }
                }
                DecodeResult::Syscall => {
                    // Handle syscall
                    let syscall_num = self.cpu.rax;
                    let result = self.handle_syscall();
                    
                    // Check for exit syscall
                    if syscall_num == 60 || syscall_num == 231 {
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
    
    /// Handle Linux syscall
    fn handle_syscall(&mut self) -> i64 {
        let syscall_num = self.cpu.rax;
        let arg1 = self.cpu.rdi;
        let arg2 = self.cpu.rsi;
        let arg3 = self.cpu.rdx;
        let arg4 = self.cpu.r10;
        let arg5 = self.cpu.r8;
        let _arg6 = self.cpu.r9;
        
        match syscall_num {
            // write(fd, buf, count)
            1 => {
                let fd = arg1 as i32;
                let buf = arg2;
                let count = arg3 as usize;
                
                if let Ok(data) = self.memory.read(buf, count) {
                    match self.fds.get(&fd) {
                        Some(FileDescriptor::Stdout) | Some(FileDescriptor::Stderr) => {
                            if let Ok(s) = core::str::from_utf8(&data) {
                                crate::print!("{}", s);
                            }
                            count as i64
                        }
                        _ => -9, // EBADF
                    }
                } else {
                    -14 // EFAULT
                }
            }
            
            // read(fd, buf, count)
            0 => {
                let fd = arg1 as i32;
                let buf = arg2;
                let count = arg3 as usize;
                
                match self.fds.get(&fd) {
                    Some(FileDescriptor::Stdin) => {
                        // Read from keyboard
                        let mut read_count = 0;
                        while read_count < count {
                            if let Some(c) = crate::keyboard::read_char() {
                                let _ = self.memory.write_u8(buf + read_count as u64, c);
                                read_count += 1;
                                if c == b'\n' {
                                    break;
                                }
                            } else if read_count > 0 {
                                break;
                            } else {
                                core::hint::spin_loop();
                            }
                        }
                        read_count as i64
                    }
                    _ => -9, // EBADF
                }
            }
            
            // exit(code)
            60 => {
                -1 // Signal to exit - handled in run()
            }
            
            // brk(addr)
            12 => {
                let addr = arg1;
                if addr == 0 {
                    self.memory.brk() as i64
                } else {
                    self.memory.set_brk(addr);
                    addr as i64
                }
            }
            
            // mmap
            9 => {
                let addr = arg1;
                let length = arg2 as usize;
                let _prot = arg3;
                let _flags = arg4;
                
                // Simple anonymous mmap
                let map_addr = if addr == 0 {
                    self.memory.brk()
                } else {
                    addr
                };
                
                self.memory.map(map_addr, length, true, true, false);
                if addr == 0 {
                    self.memory.set_brk(map_addr + length as u64);
                }
                
                map_addr as i64
            }
            
            // getpid
            39 => self.pid as i64,
            
            // getuid
            102 => 0, // root
            
            // getgid
            104 => 0, // root
            
            // geteuid
            107 => 0,
            
            // getegid
            108 => 0,
            
            // uname
            63 => {
                // Write uname struct at arg1
                let buf = arg1;
                let uname_data = b"Linux\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
                let _ = self.memory.write(buf, uname_data);
                0
            }
            
            // arch_prctl
            158 => {
                let code = arg1;
                let addr = arg2;
                
                match code {
                    0x1002 => { // ARCH_SET_FS
                        self.cpu.fs = addr;
                        0
                    }
                    0x1003 => { // ARCH_SET_GS
                        self.cpu.gs = addr;
                        0
                    }
                    _ => -22, // EINVAL
                }
            }
            
            _ => {
                crate::serial_println!(
                    "[INTERP] Unknown syscall {} (rdi={:x}, rsi={:x}, rdx={:x})",
                    syscall_num, arg1, arg2, arg3
                );
                -38 // ENOSYS
            }
        }
    }
}

/// Run a Linux binary
pub fn run_binary(elf_data: &[u8], argv: &[&str]) -> Result<i32, &'static str> {
    let mut interp = Interpreter::new();
    
    // Load ELF
    interp.load_elf(elf_data)?;
    
    // Setup stack with arguments
    let envp = ["PATH=/bin:/usr/bin", "HOME=/root", "USER=root"];
    interp.setup_stack(argv, &envp)?;
    
    crate::println!("Starting {} ...", argv.get(0).unwrap_or(&"<binary>"));
    
    // Run
    interp.run()
}
