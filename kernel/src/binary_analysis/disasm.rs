//! TrustView — x86_64 Disassembler
//!
//! Decodes x86_64 machine code into human-readable Intel syntax.
//! Covers ~80 common opcodes: arithmetic, logic, control flow, MOV,
//! LEA, PUSH/POP, SYSCALL, string ops, CMOV, SETcc, shifts, etc.
//! Unknown opcodes fall back to `db 0xNN`.

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;

// ──── Instruction ──────────────────────────────────────────────────────────

/// A decoded x86_64 instruction
#[derive(Debug, Clone)]
pub struct Instruction {
    /// Virtual address
    pub address: u64,
    /// Raw bytes
    pub bytes: Vec<u8>,
    /// Mnemonic (e.g. "mov", "call", "jmp")
    pub mnemonic: String,
    /// Formatted operand string (Intel syntax)
    pub operands_str: String,
    /// Optional comment (e.g. syscall name, symbol)
    pub comment: Option<String>,
    /// Target address for CALL/JMP/Jcc (for xref analysis)
    pub branch_target: Option<u64>,
    /// Is this a call instruction?
    pub is_call: bool,
    /// Is this a return?
    pub is_ret: bool,
    /// Is this an unconditional jump?
    pub is_jump: bool,
    /// Is this a conditional jump?
    pub is_cond_jump: bool,
}

// ──── Register Names ───────────────────────────────────────────────────────

const REG64: [&str; 16] = ["rax","rcx","rdx","rbx","rsp","rbp","rsi","rdi",
                            "r8","r9","r10","r11","r12","r13","r14","r15"];
const REG32: [&str; 16] = ["eax","ecx","edx","ebx","esp","ebp","esi","edi",
                            "r8d","r9d","r10d","r11d","r12d","r13d","r14d","r15d"];
const REG16: [&str; 16] = ["ax","cx","dx","bx","sp","bp","si","di",
                            "r8w","r9w","r10w","r11w","r12w","r13w","r14w","r15w"];
const REG8:  [&str; 16] = ["al","cl","dl","bl","spl","bpl","sil","dil",
                            "r8b","r9b","r10b","r11b","r12b","r13b","r14b","r15b"];
const REG8_NOREX: [&str; 8] = ["al","cl","dl","bl","ah","ch","dh","bh"];

fn reg_name(idx: u8, size: u8, has_rex: bool) -> &'static str {
    let i = (idx & 0x0F) as usize;
    match size {
        8 => REG64.get(i).copied().unwrap_or("?"),
        4 => REG32.get(i).copied().unwrap_or("?"),
        2 => REG16.get(i).copied().unwrap_or("?"),
        1 => {
            if has_rex {
                REG8.get(i).copied().unwrap_or("?")
            } else {
                REG8_NOREX.get(i).copied().unwrap_or("?")
            }
        }
        _ => "?",
    }
}

fn size_prefix(size: u8) -> &'static str {
    match size {
        8 => "qword",
        4 => "dword",
        2 => "word",
        1 => "byte",
        _ => "",
    }
}

// ──── Condition Code Names ─────────────────────────────────────────────────

const CC_NAMES: [&str; 16] = [
    "o", "no", "b", "nb", "z", "nz", "be", "a",
    "s", "ns", "p", "np", "l", "nl", "le", "g",
];

// ──── Disassembler ─────────────────────────────────────────────────────────

pub struct Disassembler<'a> {
    code: &'a [u8],
    base_addr: u64,
    pos: usize,
}

impl<'a> Disassembler<'a> {
    pub fn new(code: &'a [u8], base_addr: u64) -> Self {
        Self { code, base_addr, pos: 0 }
    }

    /// Disassemble up to `limit` instructions
    pub fn disassemble(&mut self, limit: usize) -> Vec<Instruction> {
        let mut out = Vec::new();
        while self.pos < self.code.len() && out.len() < limit {
            let inst = self.decode_one();
            out.push(inst);
        }
        out
    }

    /// Disassemble all until end of code or limit (8192)
    pub fn disassemble_all(&mut self) -> Vec<Instruction> {
        self.disassemble(8192)
    }

    fn decode_one(&mut self) -> Instruction {
        let start = self.pos;
        let addr = self.base_addr + start as u64;

        // ── Parse legacy prefixes ──
        let mut has_66 = false;    // operand size override
        let mut has_67 = false;    // address size override
        let mut has_f2 = false;    // REPNE
        let mut has_f3 = false;    // REP
        let mut seg_override = false;

        loop {
            if self.pos >= self.code.len() { break; }
            match self.code[self.pos] {
                0x66 => { has_66 = true; self.pos += 1; }
                0x67 => { has_67 = true; self.pos += 1; }
                0xF2 => { has_f2 = true; self.pos += 1; }
                0xF3 => { has_f3 = true; self.pos += 1; }
                0x26 | 0x2E | 0x36 | 0x3E | 0x64 | 0x65 => { seg_override = true; self.pos += 1; }
                _ => break,
            }
            // Safety: don't consume more than 4 prefixes
            if self.pos - start > 4 { break; }
        }

        if self.pos >= self.code.len() {
            return self.make_db(start, addr);
        }

        // ── REX prefix ──
        let mut rex: u8 = 0;
        let has_rex;
        let b = self.code[self.pos];
        if b >= 0x40 && b <= 0x4F {
            rex = b;
            self.pos += 1;
            has_rex = true;
        } else {
            has_rex = false;
        }

        let rex_w = (rex & 0x08) != 0;
        let rex_r = (rex & 0x04) != 0;
        let rex_x = (rex & 0x02) != 0;
        let rex_b = (rex & 0x01) != 0;

        // Operand size: REX.W → 64, 66h → 16, else 32
        let op_size: u8 = if rex_w { 8 } else if has_66 { 2 } else { 4 };

        if self.pos >= self.code.len() {
            return self.make_db(start, addr);
        }

        let opcode = self.code[self.pos];
        self.pos += 1;

        // ── Two-byte opcode (0F xx) ──
        if opcode == 0x0F {
            return self.decode_0f(start, addr, rex, op_size, has_rex, has_f2, has_f3);
        }

        // ── Single-byte opcodes ──
        let result = match opcode {
            // NOP
            0x90 => Some(("nop", String::new(), None, false, false, false, false)),

            // RET
            0xC3 => Some(("ret", String::new(), None, false, true, false, false)),

            // RET imm16
            0xC2 => {
                let imm = self.read_u16().unwrap_or(0);
                Some(("ret", format!("{:#x}", imm), None, false, true, false, false))
            }

            // INT3
            0xCC => Some(("int3", String::new(), None, false, false, false, false)),

            // INT imm8
            0xCD => {
                let imm = self.read_u8().unwrap_or(0);
                Some(("int", format!("{:#x}", imm), None, false, false, false, false))
            }

            // HLT
            0xF4 => Some(("hlt", String::new(), None, false, false, false, false)),

            // CLC / STC / CLI / STI / CLD / STD
            0xF8 => Some(("clc", String::new(), None, false, false, false, false)),
            0xF9 => Some(("stc", String::new(), None, false, false, false, false)),
            0xFA => Some(("cli", String::new(), None, false, false, false, false)),
            0xFB => Some(("sti", String::new(), None, false, false, false, false)),
            0xFC => Some(("cld", String::new(), None, false, false, false, false)),
            0xFD => Some(("std", String::new(), None, false, false, false, false)),

            // LEAVE
            0xC9 => Some(("leave", String::new(), None, false, false, false, false)),

            // CDQ / CQO
            0x99 => {
                let mn = if rex_w { "cqo" } else { "cdq" };
                Some((mn, String::new(), None, false, false, false, false))
            }

            // CBW / CWDE / CDQE
            0x98 => {
                let mn = if rex_w { "cdqe" } else if has_66 { "cbw" } else { "cwde" };
                Some((mn, String::new(), None, false, false, false, false))
            }

            // PUSH r64 (50+rd)
            0x50..=0x57 => {
                let r = (opcode - 0x50) | if rex_b { 8 } else { 0 };
                Some(("push", String::from(reg_name(r, 8, has_rex)), None, false, false, false, false))
            }

            // POP r64 (58+rd)
            0x58..=0x5F => {
                let r = (opcode - 0x58) | if rex_b { 8 } else { 0 };
                Some(("pop", String::from(reg_name(r, 8, has_rex)), None, false, false, false, false))
            }

            // PUSH imm8
            0x6A => {
                let imm = self.read_i8().unwrap_or(0) as i64;
                Some(("push", format_imm(imm), None, false, false, false, false))
            }

            // PUSH imm32
            0x68 => {
                let imm = self.read_i32().unwrap_or(0) as i64;
                Some(("push", format_imm(imm), None, false, false, false, false))
            }

            // MOV r, imm (B0-BF)
            0xB0..=0xB7 => {
                // 8-bit register, 8-bit immediate
                let r = (opcode - 0xB0) | if rex_b { 8 } else { 0 };
                let imm = self.read_u8().unwrap_or(0);
                Some(("mov", format!("{}, {:#x}", reg_name(r, 1, has_rex), imm), None, false, false, false, false))
            }
            0xB8..=0xBF => {
                let r = (opcode - 0xB8) | if rex_b { 8 } else { 0 };
                let imm = if rex_w {
                    self.read_i64().unwrap_or(0)
                } else {
                    self.read_i32().unwrap_or(0) as i64
                };
                Some(("mov", format!("{}, {}", reg_name(r, op_size, has_rex), format_imm(imm)), None, false, false, false, false))
            }

            // XCHG rAX, r (91-97)
            0x91..=0x97 => {
                let r = (opcode - 0x90) | if rex_b { 8 } else { 0 };
                Some(("xchg", format!("{}, {}", reg_name(0, op_size, has_rex), reg_name(r, op_size, has_rex)), None, false, false, false, false))
            }

            // CALL rel32
            0xE8 => {
                let rel = self.read_i32().unwrap_or(0) as i64;
                let target = (addr as i64 + (self.pos - start) as i64 + rel) as u64;
                Some(("call", format!("{:#x}", target), Some(target), true, false, false, false))
            }

            // JMP rel32
            0xE9 => {
                let rel = self.read_i32().unwrap_or(0) as i64;
                let target = (addr as i64 + (self.pos - start) as i64 + rel) as u64;
                Some(("jmp", format!("{:#x}", target), Some(target), false, false, true, false))
            }

            // JMP rel8
            0xEB => {
                let rel = self.read_i8().unwrap_or(0) as i64;
                let target = (addr as i64 + (self.pos - start) as i64 + rel) as u64;
                Some(("jmp", format!("{:#x}", target), Some(target), false, false, true, false))
            }

            // Jcc rel8 (70-7F)
            0x70..=0x7F => {
                let cc = opcode - 0x70;
                let rel = self.read_i8().unwrap_or(0) as i64;
                let target = (addr as i64 + (self.pos - start) as i64 + rel) as u64;
                let mn = format!("j{}", CC_NAMES[cc as usize]);
                Some((&"jcc_placeholder", format!("{:#x}", target), Some(target), false, false, false, true))
                    .map(|(_, ops, tgt, call, ret, jmp, cj)| (leak_str(&mn), ops, tgt, call, ret, jmp, cj))
            }

            // LOOP / LOOPE / LOOPNE / JCXZ
            0xE0 => {
                let rel = self.read_i8().unwrap_or(0) as i64;
                let target = (addr as i64 + (self.pos - start) as i64 + rel) as u64;
                Some(("loopne", format!("{:#x}", target), Some(target), false, false, false, true))
            }
            0xE1 => {
                let rel = self.read_i8().unwrap_or(0) as i64;
                let target = (addr as i64 + (self.pos - start) as i64 + rel) as u64;
                Some(("loope", format!("{:#x}", target), Some(target), false, false, false, true))
            }
            0xE2 => {
                let rel = self.read_i8().unwrap_or(0) as i64;
                let target = (addr as i64 + (self.pos - start) as i64 + rel) as u64;
                Some(("loop", format!("{:#x}", target), Some(target), false, false, false, true))
            }

            // ── ALU r/m, r (ADD/OR/ADC/SBB/AND/SUB/XOR/CMP) ──
            // opcode = 0x00 + alu_op*8 + direction(0=rm,r 2=r,rm) + width(0=8bit 1=32/64)
            0x00 | 0x01 | 0x02 | 0x03 => self.decode_alu_rm(start, addr, opcode, "add", op_size, has_rex, rex_r, rex_b, rex_x),
            0x08 | 0x09 | 0x0A | 0x0B => self.decode_alu_rm(start, addr, opcode, "or",  op_size, has_rex, rex_r, rex_b, rex_x),
            0x10 | 0x11 | 0x12 | 0x13 => self.decode_alu_rm(start, addr, opcode, "adc", op_size, has_rex, rex_r, rex_b, rex_x),
            0x18 | 0x19 | 0x1A | 0x1B => self.decode_alu_rm(start, addr, opcode, "sbb", op_size, has_rex, rex_r, rex_b, rex_x),
            0x20 | 0x21 | 0x22 | 0x23 => self.decode_alu_rm(start, addr, opcode, "and", op_size, has_rex, rex_r, rex_b, rex_x),
            0x28 | 0x29 | 0x2A | 0x2B => self.decode_alu_rm(start, addr, opcode, "sub", op_size, has_rex, rex_r, rex_b, rex_x),
            0x30 | 0x31 | 0x32 | 0x33 => self.decode_alu_rm(start, addr, opcode, "xor", op_size, has_rex, rex_r, rex_b, rex_x),
            0x38 | 0x39 | 0x3A | 0x3B => self.decode_alu_rm(start, addr, opcode, "cmp", op_size, has_rex, rex_r, rex_b, rex_x),

            // ALU eAX, imm
            0x04 => { let imm = self.read_u8().unwrap_or(0); Some(("add", format!("al, {:#x}", imm), None, false, false, false, false)) }
            0x05 => { let imm = self.read_i32().unwrap_or(0); Some(("add", format!("{}, {}", reg_name(0, op_size, has_rex), format_imm(imm as i64)), None, false, false, false, false)) }
            0x0C => { let imm = self.read_u8().unwrap_or(0); Some(("or",  format!("al, {:#x}", imm), None, false, false, false, false)) }
            0x0D => { let imm = self.read_i32().unwrap_or(0); Some(("or",  format!("{}, {}", reg_name(0, op_size, has_rex), format_imm(imm as i64)), None, false, false, false, false)) }
            0x24 => { let imm = self.read_u8().unwrap_or(0); Some(("and", format!("al, {:#x}", imm), None, false, false, false, false)) }
            0x25 => { let imm = self.read_i32().unwrap_or(0); Some(("and", format!("{}, {}", reg_name(0, op_size, has_rex), format_imm(imm as i64)), None, false, false, false, false)) }
            0x2C => { let imm = self.read_u8().unwrap_or(0); Some(("sub", format!("al, {:#x}", imm), None, false, false, false, false)) }
            0x2D => { let imm = self.read_i32().unwrap_or(0); Some(("sub", format!("{}, {}", reg_name(0, op_size, has_rex), format_imm(imm as i64)), None, false, false, false, false)) }
            0x34 => { let imm = self.read_u8().unwrap_or(0); Some(("xor", format!("al, {:#x}", imm), None, false, false, false, false)) }
            0x35 => { let imm = self.read_i32().unwrap_or(0); Some(("xor", format!("{}, {}", reg_name(0, op_size, has_rex), format_imm(imm as i64)), None, false, false, false, false)) }
            0x3C => { let imm = self.read_u8().unwrap_or(0); Some(("cmp", format!("al, {:#x}", imm), None, false, false, false, false)) }
            0x3D => { let imm = self.read_i32().unwrap_or(0); Some(("cmp", format!("{}, {}", reg_name(0, op_size, has_rex), format_imm(imm as i64)), None, false, false, false, false)) }

            // TEST r/m, r (84/85)
            0x84 => {
                let (rm, reg) = self.decode_modrm_operands(1, has_rex, rex_r, rex_b, rex_x);
                Some(("test", format!("{}, {}", rm, reg), None, false, false, false, false))
            }
            0x85 => {
                let (rm, reg) = self.decode_modrm_operands(op_size, has_rex, rex_r, rex_b, rex_x);
                Some(("test", format!("{}, {}", rm, reg), None, false, false, false, false))
            }

            // TEST eAX, imm
            0xA8 => { let imm = self.read_u8().unwrap_or(0); Some(("test", format!("al, {:#x}", imm), None, false, false, false, false)) }
            0xA9 => { let imm = self.read_i32().unwrap_or(0); Some(("test", format!("{}, {}", reg_name(0, op_size, has_rex), format_imm(imm as i64)), None, false, false, false, false)) }

            // MOV r/m, r (88/89)
            0x88 => {
                let (rm, reg) = self.decode_modrm_operands(1, has_rex, rex_r, rex_b, rex_x);
                Some(("mov", format!("{}, {}", rm, reg), None, false, false, false, false))
            }
            0x89 => {
                let (rm, reg) = self.decode_modrm_operands(op_size, has_rex, rex_r, rex_b, rex_x);
                Some(("mov", format!("{}, {}", rm, reg), None, false, false, false, false))
            }

            // MOV r, r/m (8A/8B)
            0x8A => {
                let (rm, reg) = self.decode_modrm_operands(1, has_rex, rex_r, rex_b, rex_x);
                Some(("mov", format!("{}, {}", reg, rm), None, false, false, false, false))
            }
            0x8B => {
                let (rm, reg) = self.decode_modrm_operands(op_size, has_rex, rex_r, rex_b, rex_x);
                Some(("mov", format!("{}, {}", reg, rm), None, false, false, false, false))
            }

            // LEA r, m (8D)
            0x8D => {
                let (rm, reg) = self.decode_modrm_operands(op_size, has_rex, rex_r, rex_b, rex_x);
                Some(("lea", format!("{}, {}", reg, rm), None, false, false, false, false))
            }

            // XCHG r/m, r (86/87)
            0x86 => {
                let (rm, reg) = self.decode_modrm_operands(1, has_rex, rex_r, rex_b, rex_x);
                Some(("xchg", format!("{}, {}", rm, reg), None, false, false, false, false))
            }
            0x87 => {
                let (rm, reg) = self.decode_modrm_operands(op_size, has_rex, rex_r, rex_b, rex_x);
                Some(("xchg", format!("{}, {}", rm, reg), None, false, false, false, false))
            }

            // MOV r/m, imm (C6/C7)
            0xC6 => {
                let rm = self.decode_modrm_rm_only(1, has_rex, rex_b, rex_x);
                let imm = self.read_u8().unwrap_or(0);
                Some(("mov", format!("{}, {:#x}", rm, imm), None, false, false, false, false))
            }
            0xC7 => {
                let rm = self.decode_modrm_rm_only(op_size, has_rex, rex_b, rex_x);
                let imm = self.read_i32().unwrap_or(0);
                Some(("mov", format!("{}, {}", rm, format_imm(imm as i64)), None, false, false, false, false))
            }

            // Group 1: ALU r/m, imm8/imm32 (80-83)
            0x80 => self.decode_group1(1, has_rex, rex_b, rex_x, true),
            0x81 => self.decode_group1(op_size, has_rex, rex_b, rex_x, false),
            0x83 => self.decode_group1(op_size, has_rex, rex_b, rex_x, true),

            // Group 2: shift/rotate r/m, 1/CL/imm8 (C0/C1/D0-D3)
            0xC0 => self.decode_shift(1, has_rex, rex_b, rex_x, ShiftCount::Imm8),
            0xC1 => self.decode_shift(op_size, has_rex, rex_b, rex_x, ShiftCount::Imm8),
            0xD0 => self.decode_shift(1, has_rex, rex_b, rex_x, ShiftCount::One),
            0xD1 => self.decode_shift(op_size, has_rex, rex_b, rex_x, ShiftCount::One),
            0xD2 => self.decode_shift(1, has_rex, rex_b, rex_x, ShiftCount::CL),
            0xD3 => self.decode_shift(op_size, has_rex, rex_b, rex_x, ShiftCount::CL),

            // INC/DEC (FE/FF)
            0xFE => self.decode_group_fe(1, has_rex, rex_r, rex_b, rex_x, addr, start),
            0xFF => self.decode_group_fe(op_size, has_rex, rex_r, rex_b, rex_x, addr, start),

            // Group 3: TEST/NOT/NEG/MUL/IMUL/DIV/IDIV (F6/F7)
            0xF6 => self.decode_group3(1, has_rex, rex_b, rex_x),
            0xF7 => self.decode_group3(op_size, has_rex, rex_b, rex_x),

            // IMUL r, r/m (sign-extending two-operand form: 0F AF in 2-byte)
            // IMUL r, r/m, imm8 (6B)
            0x6B => {
                let (rm, reg) = self.decode_modrm_operands(op_size, has_rex, rex_r, rex_b, rex_x);
                let imm = self.read_i8().unwrap_or(0);
                Some(("imul", format!("{}, {}, {}", reg, rm, format_imm(imm as i64)), None, false, false, false, false))
            }
            // IMUL r, r/m, imm32 (69)
            0x69 => {
                let (rm, reg) = self.decode_modrm_operands(op_size, has_rex, rex_r, rex_b, rex_x);
                let imm = self.read_i32().unwrap_or(0);
                Some(("imul", format!("{}, {}, {}", reg, rm, format_imm(imm as i64)), None, false, false, false, false))
            }

            // MOVS/STOS/LODS/SCAS/CMPS (string ops)
            0xA4 => Some(( if has_f3 { "rep movsb" } else { "movsb" }, String::new(), None, false, false, false, false)),
            0xA5 => Some(( if has_f3 { "rep movsd" } else { "movsd" }, String::new(), None, false, false, false, false)),
            0xAA => Some(( if has_f3 { "rep stosb" } else { "stosb" }, String::new(), None, false, false, false, false)),
            0xAB => Some(( if has_f3 { "rep stosd" } else { "stosd" }, String::new(), None, false, false, false, false)),
            0xAC => Some(("lodsb", String::new(), None, false, false, false, false)),
            0xAD => Some(("lodsd", String::new(), None, false, false, false, false)),
            0xAE => Some(( if has_f2 { "repne scasb" } else { "scasb" }, String::new(), None, false, false, false, false)),
            0xAF => Some(( if has_f2 { "repne scasd" } else { "scasd" }, String::new(), None, false, false, false, false)),

            // MOV moffs (A0-A3)
            0xA0 => {
                let moff = self.read_u64_or_u32(rex_w);
                Some(("mov", format!("al, [{}]", format_addr(moff)), None, false, false, false, false))
            }
            0xA1 => {
                let moff = self.read_u64_or_u32(rex_w);
                Some(("mov", format!("{}, [{}]", reg_name(0, op_size, has_rex), format_addr(moff)), None, false, false, false, false))
            }
            0xA2 => {
                let moff = self.read_u64_or_u32(rex_w);
                Some(("mov", format!("[{}], al", format_addr(moff)), None, false, false, false, false))
            }
            0xA3 => {
                let moff = self.read_u64_or_u32(rex_w);
                Some(("mov", format!("[{}], {}", format_addr(moff), reg_name(0, op_size, has_rex)), None, false, false, false, false))
            }

            // IN/OUT
            0xE4 => { let p = self.read_u8().unwrap_or(0); Some(("in", format!("al, {:#x}", p), None, false, false, false, false)) }
            0xE5 => { let p = self.read_u8().unwrap_or(0); Some(("in", format!("eax, {:#x}", p), None, false, false, false, false)) }
            0xE6 => { let p = self.read_u8().unwrap_or(0); Some(("out", format!("{:#x}, al", p), None, false, false, false, false)) }
            0xE7 => { let p = self.read_u8().unwrap_or(0); Some(("out", format!("{:#x}, eax", p), None, false, false, false, false)) }
            0xEC => Some(("in", String::from("al, dx"), None, false, false, false, false)),
            0xED => Some(("in", String::from("eax, dx"), None, false, false, false, false)),
            0xEE => Some(("out", String::from("dx, al"), None, false, false, false, false)),
            0xEF => Some(("out", String::from("dx, eax"), None, false, false, false, false)),

            // Unknown
            _ => None,
        };

        match result {
            Some((mn, ops, target, is_call, is_ret, is_jump, is_cond_jump)) => {
                let bytes = self.code[start..self.pos].to_vec();
                Instruction {
                    address: addr,
                    bytes,
                    mnemonic: String::from(mn),
                    operands_str: ops,
                    comment: None,
                    branch_target: target,
                    is_call,
                    is_ret,
                    is_jump,
                    is_cond_jump,
                }
            }
            None => self.make_db(start, addr),
        }
    }

    // ── Two-byte opcode decoder (0F xx) ────────────────────────────────────

    fn decode_0f(&mut self, start: usize, addr: u64, rex: u8, op_size: u8, has_rex: bool, _has_f2: bool, _has_f3: bool) -> Instruction {
        if self.pos >= self.code.len() {
            return self.make_db(start, addr);
        }

        let rex_r = (rex & 0x04) != 0;
        let rex_b = (rex & 0x01) != 0;
        let rex_x = (rex & 0x02) != 0;

        let op2 = self.code[self.pos];
        self.pos += 1;

        let result: Option<(&str, String, Option<u64>, bool, bool, bool, bool)> = match op2 {
            // SYSCALL
            0x05 => Some(("syscall", String::new(), None, false, false, false, false)),

            // SYSRET
            0x07 => Some(("sysret", String::new(), None, false, false, false, false)),

            // CPUID
            0xA2 => Some(("cpuid", String::new(), None, false, false, false, false)),

            // RDTSC
            0x31 => Some(("rdtsc", String::new(), None, false, false, false, false)),

            // RDMSR / WRMSR
            0x32 => Some(("rdmsr", String::new(), None, false, false, false, false)),
            0x30 => Some(("wrmsr", String::new(), None, false, false, false, false)),

            // NOP (0F 1F /0 = multi-byte NOP)
            0x1F => {
                // Consume ModR/M + optional SIB + disp
                let _ = self.decode_modrm_rm_only(op_size, has_rex, rex_b, rex_x);
                Some(("nop", String::new(), None, false, false, false, false))
            }

            // Jcc rel32 (0F 80-8F)
            0x80..=0x8F => {
                let cc = op2 - 0x80;
                let rel = self.read_i32().unwrap_or(0) as i64;
                let target = (addr as i64 + (self.pos - start) as i64 + rel) as u64;
                let mn_str = format!("j{}", CC_NAMES[cc as usize]);
                Some((leak_str(&mn_str), format!("{:#x}", target), Some(target), false, false, false, true))
            }

            // SETcc r/m8 (0F 90-9F)
            0x90..=0x9F => {
                let cc = op2 - 0x90;
                let rm = self.decode_modrm_rm_only(1, has_rex, rex_b, rex_x);
                let mn_str = format!("set{}", CC_NAMES[cc as usize]);
                Some((leak_str(&mn_str), rm, None, false, false, false, false))
            }

            // CMOVcc r, r/m (0F 40-4F)
            0x40..=0x4F => {
                let cc = op2 - 0x40;
                let (rm, reg) = self.decode_modrm_operands(op_size, has_rex, rex_r, rex_b, rex_x);
                let mn_str = format!("cmov{}", CC_NAMES[cc as usize]);
                Some((leak_str(&mn_str), format!("{}, {}", reg, rm), None, false, false, false, false))
            }

            // MOVZX r, r/m8 (0F B6)
            0xB6 => {
                let (rm, reg) = self.decode_modrm_operands(1, has_rex, rex_r, rex_b, rex_x);
                Some(("movzx", format!("{}, {}", reg_name(modrm_reg_idx(rex_r, self.peek_back()), op_size, has_rex), rm), None, false, false, false, false))
            }

            // MOVZX r, r/m16 (0F B7)
            0xB7 => {
                let (rm, reg) = self.decode_modrm_operands(2, has_rex, rex_r, rex_b, rex_x);
                Some(("movzx", format!("{}, {}", reg_name(modrm_reg_idx(rex_r, self.peek_back()), op_size, has_rex), rm), None, false, false, false, false))
            }

            // MOVSX r, r/m8 (0F BE)
            0xBE => {
                let (rm, reg) = self.decode_modrm_operands(1, has_rex, rex_r, rex_b, rex_x);
                Some(("movsx", format!("{}, {}", reg_name(modrm_reg_idx(rex_r, self.peek_back()), op_size, has_rex), rm), None, false, false, false, false))
            }

            // MOVSX r, r/m16 (0F BF)
            0xBF => {
                let (rm, reg) = self.decode_modrm_operands(2, has_rex, rex_r, rex_b, rex_x);
                Some(("movsx", format!("{}, {}", reg_name(modrm_reg_idx(rex_r, self.peek_back()), op_size, has_rex), rm), None, false, false, false, false))
            }

            // IMUL r, r/m (0F AF)
            0xAF => {
                let (rm, reg) = self.decode_modrm_operands(op_size, has_rex, rex_r, rex_b, rex_x);
                Some(("imul", format!("{}, {}", reg, rm), None, false, false, false, false))
            }

            // BSF / BSR (0F BC/BD)
            0xBC => {
                let (rm, reg) = self.decode_modrm_operands(op_size, has_rex, rex_r, rex_b, rex_x);
                Some(("bsf", format!("{}, {}", reg, rm), None, false, false, false, false))
            }
            0xBD => {
                let (rm, reg) = self.decode_modrm_operands(op_size, has_rex, rex_r, rex_b, rex_x);
                Some(("bsr", format!("{}, {}", reg, rm), None, false, false, false, false))
            }

            // BT / BTS / BTR / BTC r/m, r (0F A3/AB/B3/BB)
            0xA3 => {
                let (rm, reg) = self.decode_modrm_operands(op_size, has_rex, rex_r, rex_b, rex_x);
                Some(("bt", format!("{}, {}", rm, reg), None, false, false, false, false))
            }
            0xAB => {
                let (rm, reg) = self.decode_modrm_operands(op_size, has_rex, rex_r, rex_b, rex_x);
                Some(("bts", format!("{}, {}", rm, reg), None, false, false, false, false))
            }
            0xB3 => {
                let (rm, reg) = self.decode_modrm_operands(op_size, has_rex, rex_r, rex_b, rex_x);
                Some(("btr", format!("{}, {}", rm, reg), None, false, false, false, false))
            }
            0xBB => {
                let (rm, reg) = self.decode_modrm_operands(op_size, has_rex, rex_r, rex_b, rex_x);
                Some(("btc", format!("{}, {}", rm, reg), None, false, false, false, false))
            }

            // XADD r/m, r (0F C0/C1)
            0xC0 => {
                let (rm, reg) = self.decode_modrm_operands(1, has_rex, rex_r, rex_b, rex_x);
                Some(("xadd", format!("{}, {}", rm, reg), None, false, false, false, false))
            }
            0xC1 => {
                let (rm, reg) = self.decode_modrm_operands(op_size, has_rex, rex_r, rex_b, rex_x);
                Some(("xadd", format!("{}, {}", rm, reg), None, false, false, false, false))
            }

            // BSWAP r32/64 (0F C8+r)
            0xC8..=0xCF => {
                let r = (op2 - 0xC8) | if rex_b { 8 } else { 0 };
                Some(("bswap", String::from(reg_name(r, op_size, has_rex)), None, false, false, false, false))
            }

            // UD2
            0x0B => Some(("ud2", String::new(), None, false, false, false, false)),

            _ => None,
        };

        match result {
            Some((mn, ops, target, is_call, is_ret, is_jump, is_cond_jump)) => {
                let bytes = self.code[start..self.pos].to_vec();
                Instruction {
                    address: addr,
                    bytes,
                    mnemonic: String::from(mn),
                    operands_str: ops,
                    comment: None,
                    branch_target: target,
                    is_call,
                    is_ret,
                    is_jump,
                    is_cond_jump,
                }
            }
            None => {
                // Unrecognized 0F xx - emit db for both bytes
                self.make_db(start, addr)
            }
        }
    }

    // ── ModR/M decoder ─────────────────────────────────────────────────────

    /// Decode ModR/M and return (rm_string, reg_string)
    fn decode_modrm_operands(&mut self, size: u8, has_rex: bool, rex_r: bool, rex_b: bool, rex_x: bool) -> (String, String) {
        if self.pos >= self.code.len() {
            return (String::from("?"), String::from("?"));
        }

        let modrm = self.code[self.pos];
        self.pos += 1;

        let mod_bits = (modrm >> 6) & 3;
        let reg_field = ((modrm >> 3) & 7) | if rex_r { 8 } else { 0 };
        let rm_field = (modrm & 7) | if rex_b { 8 } else { 0 };

        let reg_str = String::from(reg_name(reg_field, size, has_rex));
        let rm_str = self.decode_rm(mod_bits, rm_field & 7, rex_b, rex_x, size, has_rex);

        (rm_str, reg_str)
    }

    /// Decode ModR/M but only return the r/m operand (for group instructions)
    fn decode_modrm_rm_only(&mut self, size: u8, has_rex: bool, rex_b: bool, rex_x: bool) -> String {
        if self.pos >= self.code.len() {
            return String::from("?");
        }

        let modrm = self.code[self.pos];
        self.pos += 1;

        let mod_bits = (modrm >> 6) & 3;
        let rm_field = (modrm & 7) | if rex_b { 8 } else { 0 };

        self.decode_rm(mod_bits, rm_field & 7, rex_b, rex_x, size, has_rex)
    }

    /// Decode the R/M field of ModR/M
    fn decode_rm(&mut self, mod_bits: u8, rm_low: u8, rex_b: bool, rex_x: bool, size: u8, has_rex: bool) -> String {
        let rm = rm_low | if rex_b { 8 } else { 0 };

        // Direct register
        if mod_bits == 3 {
            return String::from(reg_name(rm, size, has_rex));
        }

        // Memory operand
        let (base_str, needs_sib) = if rm_low == 4 {
            // SIB byte follows
            (String::new(), true)
        } else if rm_low == 5 && mod_bits == 0 {
            // RIP-relative or disp32
            let disp = self.read_i32().unwrap_or(0);
            return format!("{} [rip{:+#x}]", size_prefix(size), disp);
        } else {
            (String::from(reg_name(rm, 8, has_rex)), false)
        };

        if needs_sib {
            return self.decode_sib(mod_bits, rex_b, rex_x, size, has_rex);
        }

        // Add displacement
        match mod_bits {
            0 => format!("{} [{}]", size_prefix(size), base_str),
            1 => {
                let disp = self.read_i8().unwrap_or(0) as i32;
                if disp == 0 {
                    format!("{} [{}]", size_prefix(size), base_str)
                } else {
                    format!("{} [{}{:+#x}]", size_prefix(size), base_str, disp)
                }
            }
            2 => {
                let disp = self.read_i32().unwrap_or(0);
                if disp == 0 {
                    format!("{} [{}]", size_prefix(size), base_str)
                } else {
                    format!("{} [{}{:+#x}]", size_prefix(size), base_str, disp)
                }
            }
            _ => String::from("?"),
        }
    }

    /// Decode SIB byte
    fn decode_sib(&mut self, mod_bits: u8, rex_b: bool, rex_x: bool, size: u8, has_rex: bool) -> String {
        if self.pos >= self.code.len() {
            return String::from("?");
        }

        let sib = self.code[self.pos];
        self.pos += 1;

        let scale = 1u8 << ((sib >> 6) & 3);
        let index = ((sib >> 3) & 7) | if rex_x { 8 } else { 0 };
        let base = (sib & 7) | if rex_b { 8 } else { 0 };

        let has_index = index != 4; // RSP cannot be index
        let base_str = if (base & 7) == 5 && mod_bits == 0 {
            // No base, disp32
            let disp = self.read_i32().unwrap_or(0);
            if has_index {
                if scale > 1 {
                    return format!("{} [{}*{}{:+#x}]", size_prefix(size), reg_name(index, 8, has_rex), scale, disp);
                } else {
                    return format!("{} [{}{:+#x}]", size_prefix(size), reg_name(index, 8, has_rex), disp);
                }
            } else {
                return format!("{} [{:#x}]", size_prefix(size), disp);
            }
        } else {
            String::from(reg_name(base, 8, has_rex))
        };

        // Build address expression
        let addr_expr = if has_index {
            if scale > 1 {
                format!("{}+{}*{}", base_str, reg_name(index, 8, has_rex), scale)
            } else {
                format!("{}+{}", base_str, reg_name(index, 8, has_rex))
            }
        } else {
            base_str
        };

        match mod_bits {
            0 => format!("{} [{}]", size_prefix(size), addr_expr),
            1 => {
                let disp = self.read_i8().unwrap_or(0) as i32;
                if disp == 0 {
                    format!("{} [{}]", size_prefix(size), addr_expr)
                } else {
                    format!("{} [{}{:+#x}]", size_prefix(size), addr_expr, disp)
                }
            }
            2 => {
                let disp = self.read_i32().unwrap_or(0);
                if disp == 0 {
                    format!("{} [{}]", size_prefix(size), addr_expr)
                } else {
                    format!("{} [{}{:+#x}]", size_prefix(size), addr_expr, disp)
                }
            }
            _ => String::from("?"),
        }
    }

    // ── Group decoders ─────────────────────────────────────────────────────

    fn decode_alu_rm(&mut self, _start: usize, _addr: u64, opcode: u8, mnemonic: &str, op_size: u8, has_rex: bool, rex_r: bool, rex_b: bool, rex_x: bool) -> Option<(&'static str, String, Option<u64>, bool, bool, bool, bool)> {
        let is_byte = (opcode & 1) == 0;
        let dir = (opcode & 2) != 0; // 0: r/m, r   2: r, r/m
        let sz = if is_byte { 1 } else { op_size };
        let (rm, reg) = self.decode_modrm_operands(sz, has_rex, rex_r, rex_b, rex_x);
        let ops = if dir {
            format!("{}, {}", reg, rm)
        } else {
            format!("{}, {}", rm, reg)
        };
        let mn: &'static str = match mnemonic {
            "add" => "add", "or" => "or", "adc" => "adc", "sbb" => "sbb",
            "and" => "and", "sub" => "sub", "xor" => "xor", "cmp" => "cmp",
            _ => "?alu",
        };
        Some((mn, ops, None, false, false, false, false))
    }

    fn decode_group1(&mut self, size: u8, has_rex: bool, rex_b: bool, rex_x: bool, imm8: bool) -> Option<(&'static str, String, Option<u64>, bool, bool, bool, bool)> {
        if self.pos >= self.code.len() { return None; }
        let modrm = self.code[self.pos]; // don't advance yet
        let op = (modrm >> 3) & 7;

        let rm = self.decode_modrm_rm_only(size, has_rex, rex_b, rex_x);
        let imm = if imm8 {
            self.read_i8().unwrap_or(0) as i64
        } else {
            self.read_i32().unwrap_or(0) as i64
        };

        let mn: &'static str = match op {
            0 => "add", 1 => "or", 2 => "adc", 3 => "sbb",
            4 => "and", 5 => "sub", 6 => "xor", 7 => "cmp",
            _ => "?",
        };

        Some((mn, format!("{}, {}", rm, format_imm(imm)), None, false, false, false, false))
    }

    fn decode_shift(&mut self, size: u8, has_rex: bool, rex_b: bool, rex_x: bool, count: ShiftCount) -> Option<(&'static str, String, Option<u64>, bool, bool, bool, bool)> {
        if self.pos >= self.code.len() { return None; }
        let modrm = self.code[self.pos];
        let op = (modrm >> 3) & 7;

        let rm = self.decode_modrm_rm_only(size, has_rex, rex_b, rex_x);
        let count_str = match count {
            ShiftCount::One => String::from("1"),
            ShiftCount::CL => String::from("cl"),
            ShiftCount::Imm8 => {
                let imm = self.read_u8().unwrap_or(0);
                format!("{}", imm)
            }
        };

        let mn: &'static str = match op {
            0 => "rol", 1 => "ror", 2 => "rcl", 3 => "rcr",
            4 => "shl", 5 => "shr", 6 => "sal", 7 => "sar",
            _ => "?",
        };

        Some((mn, format!("{}, {}", rm, count_str), None, false, false, false, false))
    }

    fn decode_group_fe(&mut self, size: u8, has_rex: bool, _rex_r: bool, rex_b: bool, rex_x: bool, addr: u64, start: usize) -> Option<(&'static str, String, Option<u64>, bool, bool, bool, bool)> {
        if self.pos >= self.code.len() { return None; }
        let modrm = self.code[self.pos];
        let op = (modrm >> 3) & 7;

        match op {
            0 => {
                let rm = self.decode_modrm_rm_only(size, has_rex, rex_b, rex_x);
                Some(("inc", rm, None, false, false, false, false))
            }
            1 => {
                let rm = self.decode_modrm_rm_only(size, has_rex, rex_b, rex_x);
                Some(("dec", rm, None, false, false, false, false))
            }
            2 if size > 1 => {
                // CALL r/m
                let rm = self.decode_modrm_rm_only(8, has_rex, rex_b, rex_x);
                Some(("call", rm, None, true, false, false, false))
            }
            4 if size > 1 => {
                // JMP r/m
                let rm = self.decode_modrm_rm_only(8, has_rex, rex_b, rex_x);
                Some(("jmp", rm, None, false, false, true, false))
            }
            6 if size > 1 => {
                // PUSH r/m
                let rm = self.decode_modrm_rm_only(8, has_rex, rex_b, rex_x);
                Some(("push", rm, None, false, false, false, false))
            }
            _ => {
                let _ = self.decode_modrm_rm_only(size, has_rex, rex_b, rex_x);
                None
            }
        }
    }

    fn decode_group3(&mut self, size: u8, has_rex: bool, rex_b: bool, rex_x: bool) -> Option<(&'static str, String, Option<u64>, bool, bool, bool, bool)> {
        if self.pos >= self.code.len() { return None; }
        let modrm = self.code[self.pos];
        let op = (modrm >> 3) & 7;

        match op {
            0 | 1 => {
                // TEST r/m, imm
                let rm = self.decode_modrm_rm_only(size, has_rex, rex_b, rex_x);
                let imm = if size == 1 {
                    self.read_u8().unwrap_or(0) as i64
                } else {
                    self.read_i32().unwrap_or(0) as i64
                };
                Some(("test", format!("{}, {}", rm, format_imm(imm)), None, false, false, false, false))
            }
            2 => {
                let rm = self.decode_modrm_rm_only(size, has_rex, rex_b, rex_x);
                Some(("not", rm, None, false, false, false, false))
            }
            3 => {
                let rm = self.decode_modrm_rm_only(size, has_rex, rex_b, rex_x);
                Some(("neg", rm, None, false, false, false, false))
            }
            4 => {
                let rm = self.decode_modrm_rm_only(size, has_rex, rex_b, rex_x);
                Some(("mul", rm, None, false, false, false, false))
            }
            5 => {
                let rm = self.decode_modrm_rm_only(size, has_rex, rex_b, rex_x);
                Some(("imul", rm, None, false, false, false, false))
            }
            6 => {
                let rm = self.decode_modrm_rm_only(size, has_rex, rex_b, rex_x);
                Some(("div", rm, None, false, false, false, false))
            }
            7 => {
                let rm = self.decode_modrm_rm_only(size, has_rex, rex_b, rex_x);
                Some(("idiv", rm, None, false, false, false, false))
            }
            _ => None,
        }
    }

    // ── Byte reading helpers ───────────────────────────────────────────────

    fn read_u8(&mut self) -> Option<u8> {
        if self.pos >= self.code.len() { return None; }
        let v = self.code[self.pos];
        self.pos += 1;
        Some(v)
    }

    fn read_i8(&mut self) -> Option<i8> {
        self.read_u8().map(|v| v as i8)
    }

    fn read_u16(&mut self) -> Option<u16> {
        if self.pos + 2 > self.code.len() { return None; }
        let v = u16::from_le_bytes([self.code[self.pos], self.code[self.pos + 1]]);
        self.pos += 2;
        Some(v)
    }

    fn read_i32(&mut self) -> Option<i32> {
        if self.pos + 4 > self.code.len() { return None; }
        let v = i32::from_le_bytes([
            self.code[self.pos], self.code[self.pos + 1],
            self.code[self.pos + 2], self.code[self.pos + 3],
        ]);
        self.pos += 4;
        Some(v)
    }

    fn read_i64(&mut self) -> Option<i64> {
        if self.pos + 8 > self.code.len() { return None; }
        let v = i64::from_le_bytes([
            self.code[self.pos], self.code[self.pos + 1],
            self.code[self.pos + 2], self.code[self.pos + 3],
            self.code[self.pos + 4], self.code[self.pos + 5],
            self.code[self.pos + 6], self.code[self.pos + 7],
        ]);
        self.pos += 8;
        Some(v)
    }

    fn read_u64_or_u32(&mut self, is_64: bool) -> u64 {
        if is_64 {
            self.read_i64().unwrap_or(0) as u64
        } else {
            self.read_i32().unwrap_or(0) as u64
        }
    }

    /// Peek at last consumed ModR/M byte (for MOVZX/MOVSX reg field extraction)
    fn peek_back(&self) -> u8 {
        if self.pos > 0 {
            // The ModR/M byte was the last thing consumed before SIB/disp
            // We approximate by looking at the byte we know is there
            self.code.get(self.pos.wrapping_sub(1)).copied().unwrap_or(0)
        } else {
            0
        }
    }

    fn make_db(&mut self, start: usize, addr: u64) -> Instruction {
        // Ensure we advance by at least 1
        if self.pos <= start {
            self.pos = start + 1;
        }
        let end = self.pos.min(self.code.len());
        let bytes = self.code[start..end].to_vec();
        let hex: String = bytes.iter().map(|b| format!("{:#04x}", b)).collect::<Vec<_>>().join(", ");
        Instruction {
            address: addr,
            bytes,
            mnemonic: String::from("db"),
            operands_str: hex,
            comment: None,
            branch_target: None,
            is_call: false,
            is_ret: false,
            is_jump: false,
            is_cond_jump: false,
        }
    }
}

// ──── Helpers ──────────────────────────────────────────────────────────────

enum ShiftCount { One, CL, Imm8 }

fn format_imm(v: i64) -> String {
    if v >= 0 && v <= 9 {
        format!("{}", v)
    } else if v >= 0 {
        format!("{:#x}", v)
    } else if v >= -128 {
        format!("-{:#x}", -v)
    } else {
        format!("-{:#x}", (-v) as u64)
    }
}

fn format_addr(v: u64) -> String {
    format!("{:#x}", v)
}

/// Extract reg index from ModR/M reg field (for MOVZX/MOVSX)
fn modrm_reg_idx(rex_r: bool, modrm: u8) -> u8 {
    ((modrm >> 3) & 7) | if rex_r { 8 } else { 0 }
}

/// Leak a string to get a &'static str — used for dynamic mnemonics
/// that need 'static lifetime in return tuples. Leaks ~8 bytes per call.
/// Only used for Jcc/SETcc/CMOVcc which are bounded (48 possible strings).
fn leak_str(s: &str) -> &'static str {
    use alloc::boxed::Box;
    let boxed = String::from(s).into_boxed_str();
    Box::leak(boxed)
}

// ──── Annotation ───────────────────────────────────────────────────────────

/// Annotate instructions with symbol names and syscall info
pub fn annotate_instructions(
    instructions: &mut [Instruction],
    addr_to_symbol: &alloc::collections::BTreeMap<u64, String>,
) {
    // Track last value loaded into RAX for syscall detection
    let mut last_rax_val: Option<i64> = None;

    for inst in instructions.iter_mut() {
        // Label CALL/JMP targets with symbol names
        if let Some(target) = inst.branch_target {
            if let Some(name) = addr_to_symbol.get(&target) {
                inst.comment = Some(format!("<{}>", name));
            }
        }

        // Track MOV EAX/RAX, imm for syscall annotation
        if inst.mnemonic == "mov" && (inst.operands_str.starts_with("eax,") || inst.operands_str.starts_with("rax,")) {
            // Try to extract the immediate value
            if let Some(comma_pos) = inst.operands_str.find(',') {
                let imm_str = inst.operands_str[comma_pos + 1..].trim();
                if let Some(val) = parse_imm_str(imm_str) {
                    last_rax_val = Some(val);
                }
            }
        } else if inst.mnemonic == "xor" && inst.operands_str.contains("eax") && inst.operands_str.matches("eax").count() == 2 {
            last_rax_val = Some(0);
        }

        // Annotate SYSCALL with name
        if inst.mnemonic == "syscall" {
            if let Some(num) = last_rax_val {
                let name = crate::transpiler::syscall_name(num as u64);
                inst.comment = Some(format!("sys_{} ({})", name, num));
            }
        }
    }
}

fn parse_imm_str(s: &str) -> Option<i64> {
    let s = s.trim();
    if s.starts_with("0x") || s.starts_with("0X") {
        i64::from_str_radix(&s[2..], 16).ok()
    } else if s.starts_with("-0x") || s.starts_with("-0X") {
        i64::from_str_radix(&s[3..], 16).ok().map(|v| -v)
    } else {
        s.parse::<i64>().ok()
    }
}
