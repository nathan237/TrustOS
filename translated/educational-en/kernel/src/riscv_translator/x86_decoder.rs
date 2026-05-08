// TrustOS Universal Architecture Translation Layer
// x86_64 → RISC-V IR Decoder
//
// Translates x86_64 instructions into RISC-V IR basic blocks.
// Register mapping:
//   x86 RAX → RV x10 (a0)    x86 RCX → RV x11 (a1)
//   x86 RDX → RV x12 (a2)    x86 RBX → RV x13 (a3)
//   x86 RSP → RV x2  (sp)    x86 RBP → RV x8  (fp)
//   x86 RSI → RV x14 (a4)    x86 RDI → RV x15 (a5)
//   x86 R8  → RV x18 (s2)    x86 R9  → RV x19 (s3)
//   x86 R10 → RV x20 (s4)    x86 R11 → RV x21 (s5)
//   x86 R12 → RV x22 (s6)    x86 R13 → RV x23 (s7)
//   x86 R14 → RV x24 (s8)    x86 R15 → RV x25 (s9)
//
// Syscall ABI translation:
//   x86: syscall# in RAX, args in RDI,RSI,RDX,R10,R8,R9
//   RV:  syscall# in a7,  args in a0,a1,a2,a3,a4,a5

use alloc::vec::Vec;
use alloc::format;
use alloc::string::String;
use super::ir::*;

/// x86_64 register to RISC-V register mapping
fn x86_to_rv(x86_reg: u8) -> Reg {
        // Pattern matching — Rust's exhaustive branching construct.
match x86_reg {
        0  => Reg::X10,  // RAX → a0
        1  => Reg::X11,  // RCX → a1
        2  => Reg::X12,  // RDX → a2
        3  => Reg::X13,  // RBX → a3
        4  => Reg::X2,   // RSP → sp
        5  => Reg::X8,   // RBP → fp
        6  => Reg::X14,  // RSI → a4
        7  => Reg::X15,  // RDI → a5
        8  => Reg::X18,  // R8  → s2
        9  => Reg::X19,  // R9  → s3
        10 => Reg::X20,  // R10 → s4
        11 => Reg::X21,  // R11 → s5
        12 => Reg::X22,  // R12 → s6
        13 => Reg::X23,  // R13 → s7
        14 => Reg::X24,  // R14 → s8
        15 => Reg::X25,  // R15 → s9
        _  => Reg::X5,   // temp
    }
}

/// x86_64 → RISC-V IR translator
pub struct X86Decoder {
    /// Raw binary code bytes
    code: Vec<u8>,
    /// Base virtual address
    base_addr: u64,
    /// Current decode offset
    offset: usize,
    /// Translation statistics
    pub stats: TranslationStats,
}

// Implementation block — defines methods for the type above.
impl X86Decoder {
        // Public function — callable from other modules.
pub fn new(code: &[u8], base_addr: u64) -> Self {
        Self {
            code: code.to_vec(),
            base_addr,
            offset: 0,
            stats: TranslationStats::default(),
        }
    }

    /// Translate a basic block starting at the given offset
    pub fn translate_block(&mut self, start_offset: usize) -> TranslatedBlock {
        self.offset = start_offset;
        let src_addr = self.base_addr + start_offset as u64;
        let mut block = TranslatedBlock::new(src_addr, SourceArch::X86_64);

        let max_instructions = 256;
        let mut count = 0;

        while self.offset < self.code.len() && count < max_instructions {
            let inst_start = self.offset;
            let terminated = self.decode_one(&mut block);
            block.src_inst_count += 1;
            self.stats.instructions_translated += 1;
            count += 1;

            if terminated {
                break;
            }
        }

        self.stats.blocks_translated += 1;
        self.stats.rv_instructions_emitted += block.instructions.len() as u64;
        block
    }

    /// Translate all reachable blocks from the entry point
    pub fn translate_all(&mut self) -> Vec<TranslatedBlock> {
        let mut blocks = Vec::new();
        let mut worklist: Vec<usize> = Vec::new();
        let mut visited: Vec<u64> = Vec::new();

        worklist.push(0);

        while let Some(offset) = worklist.pop() {
            let addr = self.base_addr + offset as u64;
            if visited.contains(&addr) {
                continue;
            }
            visited.push(addr);

            let block = self.translate_block(offset);

            // Add successor addresses to worklist
            for &succ in &block.successors {
                if succ >= self.base_addr {
                    let succ_off = (succ - self.base_addr) as usize;
                    if succ_off < self.code.len() && !visited.contains(&succ) {
                        worklist.push(succ_off);
                    }
                }
            }

            blocks.push(block);
        }

        blocks
    }

    /// Decode one x86_64 instruction and emit RISC-V IR.
    /// Returns true if the instruction terminates the basic block.
    fn decode_one(&mut self, block: &mut TranslatedBlock) -> bool {
        if self.offset >= self.code.len() {
            return true;
        }

        let inst_address = self.base_addr + self.offset as u64;

        // Parse prefixes
        let mut rex: u8 = 0;
        let mut has_rex = false;
        let mut op_size_override = false;

                // Infinite loop — runs until an explicit `break`.
loop {
            if self.offset >= self.code.len() { return true; }
            let b = self.code[self.offset];
                        // Pattern matching — Rust's exhaustive branching construct.
match b {
                0x66 => { op_size_override = true; self.offset += 1; }
                0x40..=0x4F => { rex = b; has_rex = true; self.offset += 1; }
                0xF0 | 0xF2 | 0xF3 | 0x2E | 0x3E | 0x26 | 0x64 | 0x65 | 0x36 => {
                    self.offset += 1; // Skip LOCK, REP, segment overrides
                }
                _ => break,
            }
        }

        if self.offset >= self.code.len() { return true; }

        let rex_w = has_rex && (rex & 0x08) != 0;
        let rex_r = has_rex && (rex & 0x04) != 0;
        let rex_x = has_rex && (rex & 0x02) != 0;
        let rex_b = has_rex && (rex & 0x01) != 0;

        let opcode = self.code[self.offset];
        self.offset += 1;

                // Pattern matching — Rust's exhaustive branching construct.
match opcode {
            // NOP
            0x90 => {
                block.emit(RvInst::Nop);
                false
            }

            // MOV r/m64, imm32 (sign-extended) — C7 /0
            0xC7 => {
                let (rm, _) = self.decode_modrm(rex_b, rex_r);
                let imm = self.read_i32() as i64;
                let rd = x86_to_rv(rm);
                block.emit(RvInst::Li { rd, imm });
                false
            }

            // MOV r64, imm64 — REX.W + B8+rd
            0xB8..=0xBF => {
                let reg = (opcode - 0xB8) + if rex_b { 8 } else { 0 };
                let rd = x86_to_rv(reg);
                let imm = if rex_w {
                    self.read_i64()
                } else {
                    self.read_i32() as i64
                };
                block.emit(RvInst::Li { rd, imm });
                false
            }

            // MOV r8, imm8 — B0+rb
            0xB0..=0xB7 => {
                let reg = (opcode - 0xB0) + if rex_b { 8 } else { 0 };
                let rd = x86_to_rv(reg);
                let imm = self.read_u8() as i64;
                // Byte move — mask to 8 bits and merge
                block.emit(RvInst::Andi { rd, rs1: rd, imm: !0xFF });
                block.emit(RvInst::Ori { rd, rs1: rd, imm: imm & 0xFF });
                false
            }

            // MOV r/m, r or MOV r, r/m — 89, 8B
            0x89 => {
                let (rm, reg) = self.decode_modrm(rex_b, rex_r);
                let rs = x86_to_rv(reg);
                let rd = x86_to_rv(rm);
                block.emit(RvInst::Mv { rd, rs });
                false
            }
            0x8B => {
                let (rm, reg) = self.decode_modrm(rex_b, rex_r);
                let rs = x86_to_rv(rm);
                let rd = x86_to_rv(reg);
                block.emit(RvInst::Mv { rd, rs });
                false
            }

            // PUSH r64 — 50+rd
            0x50..=0x57 => {
                let reg = (opcode - 0x50) + if rex_b { 8 } else { 0 };
                let rs = x86_to_rv(reg);
                // sp -= 8; store [sp] = reg
                block.emit(RvInst::Addi { rd: Reg::X2, rs1: Reg::X2, imm: -8 });
                block.emit(RvInst::Sd { rs2: rs, rs1: Reg::X2, offset: 0 });
                false
            }

            // POP r64 — 58+rd
            0x58..=0x5F => {
                let reg = (opcode - 0x58) + if rex_b { 8 } else { 0 };
                let rd = x86_to_rv(reg);
                // reg = load [sp]; sp += 8
                block.emit(RvInst::Ld { rd, rs1: Reg::X2, offset: 0 });
                block.emit(RvInst::Addi { rd: Reg::X2, rs1: Reg::X2, imm: 8 });
                false
            }

            // ADD r/m, r — 01
            0x01 => {
                let (rm, reg) = self.decode_modrm(rex_b, rex_r);
                let rd = x86_to_rv(rm);
                let rs = x86_to_rv(reg);
                block.emit(RvInst::Add { rd, rs1: rd, rs2: rs });
                block.emit(RvInst::CmpFlags { rs1: rd, rs2: Reg::X0 });
                false
            }

            // ADD r, r/m — 03
            0x03 => {
                let (rm, reg) = self.decode_modrm(rex_b, rex_r);
                let rd = x86_to_rv(reg);
                let rs = x86_to_rv(rm);
                block.emit(RvInst::Add { rd, rs1: rd, rs2: rs });
                block.emit(RvInst::CmpFlags { rs1: rd, rs2: Reg::X0 });
                false
            }

            // SUB r/m, r — 29
            0x29 => {
                let (rm, reg) = self.decode_modrm(rex_b, rex_r);
                let rd = x86_to_rv(rm);
                let rs = x86_to_rv(reg);
                block.emit(RvInst::Sub { rd, rs1: rd, rs2: rs });
                block.emit(RvInst::CmpFlags { rs1: rd, rs2: Reg::X0 });
                false
            }

            // SUB r, r/m — 2B
            0x2B => {
                let (rm, reg) = self.decode_modrm(rex_b, rex_r);
                let rd = x86_to_rv(reg);
                let rs = x86_to_rv(rm);
                block.emit(RvInst::Sub { rd, rs1: rd, rs2: rs });
                block.emit(RvInst::CmpFlags { rs1: rd, rs2: Reg::X0 });
                false
            }

            // AND r/m, r — 21
            0x21 => {
                let (rm, reg) = self.decode_modrm(rex_b, rex_r);
                let rd = x86_to_rv(rm);
                let rs = x86_to_rv(reg);
                block.emit(RvInst::And { rd, rs1: rd, rs2: rs });
                block.emit(RvInst::CmpFlags { rs1: rd, rs2: Reg::X0 });
                false
            }

            // OR r/m, r — 09
            0x09 => {
                let (rm, reg) = self.decode_modrm(rex_b, rex_r);
                let rd = x86_to_rv(rm);
                let rs = x86_to_rv(reg);
                block.emit(RvInst::Or { rd, rs1: rd, rs2: rs });
                block.emit(RvInst::CmpFlags { rs1: rd, rs2: Reg::X0 });
                false
            }

            // XOR r/m, r — 31
            0x31 => {
                let (rm, reg) = self.decode_modrm(rex_b, rex_r);
                let rd = x86_to_rv(rm);
                let rs = x86_to_rv(reg);
                block.emit(RvInst::Xor { rd, rs1: rd, rs2: rs });
                block.emit(RvInst::CmpFlags { rs1: rd, rs2: Reg::X0 });
                false
            }

            // CMP r/m, r — 39
            0x39 => {
                let (rm, reg) = self.decode_modrm(rex_b, rex_r);
                let rv_rm = x86_to_rv(rm);
                let rv_register = x86_to_rv(reg);
                // CMP doesn't store result, only sets flags
                block.emit(RvInst::CmpFlags { rs1: rv_rm, rs2: rv_register });
                false
            }

            // CMP r, r/m — 3B
            0x3B => {
                let (rm, reg) = self.decode_modrm(rex_b, rex_r);
                let rv_rm = x86_to_rv(rm);
                let rv_register = x86_to_rv(reg);
                block.emit(RvInst::CmpFlags { rs1: rv_register, rs2: rv_rm });
                false
            }

            // CMP r/m, imm8 — 83 /7
            0x83 => {
                let (rm, op_ext) = self.decode_modrm(rex_b, rex_r);
                let imm = self.read_i8() as i64;
                let rd = x86_to_rv(rm);
                                // Pattern matching — Rust's exhaustive branching construct.
match op_ext {
                    0 => { // ADD r/m, imm8
                        block.emit(RvInst::Addi { rd, rs1: rd, imm });
                        block.emit(RvInst::CmpFlags { rs1: rd, rs2: Reg::X0 });
                    }
                    4 => { // AND r/m, imm8
                        block.emit(RvInst::Andi { rd, rs1: rd, imm });
                        block.emit(RvInst::CmpFlags { rs1: rd, rs2: Reg::X0 });
                    }
                    5 => { // SUB r/m, imm8
                        block.emit(RvInst::Addi { rd, rs1: rd, imm: -imm });
                        block.emit(RvInst::CmpFlags { rs1: rd, rs2: Reg::X0 });
                    }
                    7 => { // CMP r/m, imm8
                        block.emit(RvInst::Li { rd: Reg::X5, imm });
                        block.emit(RvInst::CmpFlags { rs1: rd, rs2: Reg::X5 });
                    }
                    _ => {
                        self.stats.unsupported_instructions += 1;
                        block.emit(RvInst::Nop);
                    }
                }
                false
            }

            // TEST r/m, r — 85
            0x85 => {
                let (rm, reg) = self.decode_modrm(rex_b, rex_r);
                let rv_rm = x86_to_rv(rm);
                let rv_register = x86_to_rv(reg);
                // TEST = AND but don't store result, just flags
                block.emit(RvInst::And { rd: Reg::X5, rs1: rv_rm, rs2: rv_register });
                block.emit(RvInst::CmpFlags { rs1: Reg::X5, rs2: Reg::X0 });
                false
            }

            // LEA r, [r/m] — 8D
            0x8D => {
                let (rm, reg) = self.decode_modrm(rex_b, rex_r);
                let rd = x86_to_rv(reg);
                let rs = x86_to_rv(rm);
                // Simplified LEA: just move the address (real impl needs SIB decode)
                block.emit(RvInst::Mv { rd, rs });
                false
            }

            // INC r/m — FF /0, DEC r/m — FF /1
            0xFF => {
                let (rm, op_ext) = self.decode_modrm(rex_b, rex_r);
                let rd = x86_to_rv(rm);
                                // Pattern matching — Rust's exhaustive branching construct.
match op_ext {
                    0 => { // INC
                        block.emit(RvInst::Addi { rd, rs1: rd, imm: 1 });
                        block.emit(RvInst::CmpFlags { rs1: rd, rs2: Reg::X0 });
                    }
                    1 => { // DEC
                        block.emit(RvInst::Addi { rd, rs1: rd, imm: -1 });
                        block.emit(RvInst::CmpFlags { rs1: rd, rs2: Reg::X0 });
                    }
                    2 => { // CALL r/m
                        // Push return address, jump to target
                        block.emit(RvInst::Addi { rd: Reg::X2, rs1: Reg::X2, imm: -8 });
                        let next_address = self.base_addr + self.offset as u64;
                        block.emit(RvInst::Li { rd: Reg::X5, imm: next_address as i64 });
                        block.emit(RvInst::Sd { rs2: Reg::X5, rs1: Reg::X2, offset: 0 });
                        block.emit(RvInst::Jalr { rd: Reg::X1, rs1: rd, offset: 0 });
                        block.successors.push(0); // indirect
                        return true;
                    }
                    4 => { // JMP r/m
                        block.emit(RvInst::Jalr { rd: Reg::X0, rs1: rd, offset: 0 });
                        return true;
                    }
                    6 => { // PUSH r/m
                        block.emit(RvInst::Addi { rd: Reg::X2, rs1: Reg::X2, imm: -8 });
                        block.emit(RvInst::Sd { rs2: rd, rs1: Reg::X2, offset: 0 });
                    }
                    _ => {
                        self.stats.unsupported_instructions += 1;
                        block.emit(RvInst::Nop);
                    }
                }
                false
            }

            // Jcc short — 7x
            0x70..=0x7F => {
                let rel = self.read_i8() as i64;
                let target = self.base_addr as i64 + self.offset as i64 + rel;
                let condition = x86_cc_to_flag(opcode & 0x0F);
                block.emit(RvInst::BranchCond { condition, offset: target });
                let fallthrough = self.base_addr + self.offset as u64;
                block.successors.push(target as u64);
                block.successors.push(fallthrough);
                true
            }

            // JMP rel8 — EB
            0xEB => {
                let rel = self.read_i8() as i64;
                let target = self.base_addr as i64 + self.offset as i64 + rel;
                block.emit(RvInst::Jal { rd: Reg::X0, offset: target });
                block.successors.push(target as u64);
                true
            }

            // JMP rel32 — E9
            0xE9 => {
                let rel = self.read_i32() as i64;
                let target = self.base_addr as i64 + self.offset as i64 + rel;
                block.emit(RvInst::Jal { rd: Reg::X0, offset: target });
                block.successors.push(target as u64);
                true
            }

            // CALL rel32 — E8
            0xE8 => {
                let rel = self.read_i32() as i64;
                let target = self.base_addr as i64 + self.offset as i64 + rel;
                // Push return address
                block.emit(RvInst::Addi { rd: Reg::X2, rs1: Reg::X2, imm: -8 });
                let return_value_address = self.base_addr + self.offset as u64;
                block.emit(RvInst::Li { rd: Reg::X5, imm: return_value_address as i64 });
                block.emit(RvInst::Sd { rs2: Reg::X5, rs1: Reg::X2, offset: 0 });
                block.emit(RvInst::Call { offset: target });
                block.successors.push(target as u64);
                block.successors.push(return_value_address);
                true
            }

            // RET — C3
            0xC3 => {
                // Pop return address and jump
                block.emit(RvInst::Ld { rd: Reg::X1, rs1: Reg::X2, offset: 0 });
                block.emit(RvInst::Addi { rd: Reg::X2, rs1: Reg::X2, imm: 8 });
                block.emit(RvInst::Ret);
                true
            }

            // SYSCALL — 0F 05
            0x0F => {
                if self.offset < self.code.len() {
                    let byte2 = self.code[self.offset];
                    self.offset += 1;

                                        // Pattern matching — Rust's exhaustive branching construct.
match byte2 {
                        0x05 => {
                            // SYSCALL: translate x86 ABI → RISC-V ABI
                            // x86: RAX=syscall#, RDI=arg0, RSI=arg1, RDX=arg2, R10=arg3, R8=arg4, R9=arg5
                            // RV:  a7=syscall#,  a0=arg0,  a1=arg1,  a2=arg2,  a3=arg3,  a4=arg4, a5=arg5
                            block.emit(RvInst::SrcAnnotation {
                                arch: SourceArch::X86_64,
                                addr: inst_address,
                                text: String::from("syscall"),
                            });
                            // Move syscall number: RAX(x10) → a7(x17)
                            block.emit(RvInst::Mv { rd: Reg::X17, rs: Reg::X10 });
                            // Remap arguments:
                            // RDI(x15) → a0(x10) — but x10 is RAX, so save it first
                            block.emit(RvInst::Mv { rd: Reg::X5, rs: Reg::X15 });  // save RDI to temp
                            block.emit(RvInst::Mv { rd: Reg::X10, rs: Reg::X5 });  // a0 = RDI
                            // RSI(x14) → a1(x11)
                            block.emit(RvInst::Mv { rd: Reg::X11, rs: Reg::X14 });
                            // RDX(x12) → a2(x12) — already correct!
                            // R10(x20) → a3(x13)
                            block.emit(RvInst::Mv { rd: Reg::X13, rs: Reg::X20 });
                            // R8(x18) → a4(x14) — but x14 was RSI
                            block.emit(RvInst::Mv { rd: Reg::X14, rs: Reg::X18 });
                            // R9(x19) → a5(x15)
                            block.emit(RvInst::Mv { rd: Reg::X15, rs: Reg::X19 });
                            block.emit(RvInst::Ecall);
                            // Result: a0(x10) → RAX(x10) — already correct!
                            false
                        }

                        // Jcc near — 0F 8x
                        0x80..=0x8F => {
                            let rel = self.read_i32() as i64;
                            let target = self.base_addr as i64 + self.offset as i64 + rel;
                            let condition = x86_cc_to_flag(byte2 & 0x0F);
                            block.emit(RvInst::BranchCond { condition, offset: target });
                            let fallthrough = self.base_addr + self.offset as u64;
                            block.successors.push(target as u64);
                            block.successors.push(fallthrough);
                            true
                        }

                        // MOVZX — 0F B6 (byte), 0F B7 (word)
                        0xB6 => {
                            let (rm, reg) = self.decode_modrm(rex_b, rex_r);
                            let rd = x86_to_rv(reg);
                            let rs = x86_to_rv(rm);
                            block.emit(RvInst::Andi { rd, rs1: rs, imm: 0xFF });
                            false
                        }
                        0xB7 => {
                            let (rm, reg) = self.decode_modrm(rex_b, rex_r);
                            let rd = x86_to_rv(reg);
                            let rs = x86_to_rv(rm);
                            block.emit(RvInst::Andi { rd, rs1: rs, imm: 0xFFFF });
                            false
                        }

                        // IMUL r, r/m — 0F AF
                        0xAF => {
                            let (rm, reg) = self.decode_modrm(rex_b, rex_r);
                            let rd = x86_to_rv(reg);
                            let rs = x86_to_rv(rm);
                            block.emit(RvInst::Mul { rd, rs1: rd, rs2: rs });
                            false
                        }

                        _ => {
                            self.stats.unsupported_instructions += 1;
                            block.emit(RvInst::Nop);
                            false
                        }
                    }
                } else {
                    true
                }
            }

            // INT 0x80 (legacy 32-bit Linux syscall) — CD 80
            0xCD => {
                let int_number = self.read_u8();
                if int_number == 0x80 {
                    // Legacy 32-bit syscall (EAX=syscall#, EBX/ECX/EDX/ESI/EDI/EBP)
                    block.emit(RvInst::SrcAnnotation {
                        arch: SourceArch::X86_64,
                        addr: inst_address,
                        text: String::from("int 0x80 (legacy syscall)"),
                    });
                    block.emit(RvInst::Mv { rd: Reg::X17, rs: Reg::X10 }); // EAX → a7
                    block.emit(RvInst::Mv { rd: Reg::X10, rs: Reg::X13 }); // EBX → a0
                    // ECX(x11) → a1(x11) — already correct
                    // EDX(x12) → a2(x12) — already correct
                    block.emit(RvInst::Ecall);
                }
                false
            }

            // Unrecognized instruction — emit NOP and continue
            _ => {
                self.stats.unsupported_instructions += 1;
                block.emit(RvInst::SrcAnnotation {
                    arch: SourceArch::X86_64,
                    addr: inst_address,
                    text: format!("unsupported opcode: 0x{:02X}", opcode),
                });
                block.emit(RvInst::Nop);
                false
            }
        }
    }

    // === Helper methods for decoding x86_64 instruction fields ===

    fn decode_modrm(&mut self, rex_b: bool, rex_r: bool) -> (u8, u8) {
        if self.offset >= self.code.len() {
            return (0, 0);
        }
        let modrm = self.code[self.offset];
        self.offset += 1;

        let md = (modrm >> 6) & 3;
        let mut reg = (modrm >> 3) & 7;
        let mut rm = modrm & 7;

        if rex_r { reg += 8; }
        if rex_b { rm += 8; }

        // Handle SIB byte
        if md != 3 && rm == 4 {
            if self.offset < self.code.len() {
                self.offset += 1; // skip SIB
            }
        }

        // Handle displacement
        match md {
            0 => {
                if rm == 5 {
                    self.offset += 4; // disp32
                }
            }
            1 => { self.offset += 1; } // disp8
            2 => { self.offset += 4; } // disp32
            _ => {} // register direct
        }

        (rm, reg)
    }

    fn read_u8(&mut self) -> u8 {
        if self.offset >= self.code.len() { return 0; }
        let v = self.code[self.offset];
        self.offset += 1;
        v
    }

    fn read_i8(&mut self) -> i8 {
        self.read_u8() as i8
    }

    fn read_i32(&mut self) -> i32 {
        if self.offset + 4 > self.code.len() { return 0; }
        let v = i32::from_le_bytes([
            self.code[self.offset],
            self.code[self.offset + 1],
            self.code[self.offset + 2],
            self.code[self.offset + 3],
        ]);
        self.offset += 4;
        v
    }

    fn read_i64(&mut self) -> i64 {
        if self.offset + 8 > self.code.len() { return 0; }
        let v = i64::from_le_bytes([
            self.code[self.offset], self.code[self.offset + 1],
            self.code[self.offset + 2], self.code[self.offset + 3],
            self.code[self.offset + 4], self.code[self.offset + 5],
            self.code[self.offset + 6], self.code[self.offset + 7],
        ]);
        self.offset += 8;
        v
    }
}

/// Map x86_64 condition codes (0-15) to flag conditions
fn x86_cc_to_flag(cc: u8) -> FlagCond {
        // Pattern matching — Rust's exhaustive branching construct.
match cc {
        0x0 => FlagCond::Ovf,    // JO
        0x1 => FlagCond::NoOvf,  // JNO
        0x2 => FlagCond::Ltu,    // JB/JC
        0x3 => FlagCond::Geu,    // JAE/JNC
        0x4 => FlagCond::Eq,     // JE/JZ
        0x5 => FlagCond::Ne,     // JNE/JNZ
        0x6 => FlagCond::Le,     // JBE  (CF=1 or ZF=1, approx)
        0x7 => FlagCond::Gt,     // JA   (CF=0 and ZF=0, approx)
        0x8 => FlagCond::Neg,    // JS
        0x9 => FlagCond::Pos,    // JNS
        0xC => FlagCond::Lt,     // JL
        0xD => FlagCond::Ge,     // JGE
        0xE => FlagCond::Le,     // JLE
        0xF => FlagCond::Gt,     // JG
        _   => FlagCond::Eq,     // default
    }
}
