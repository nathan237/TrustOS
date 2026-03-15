// TrustOS Universal Architecture Translation Layer
// AArch64 → RISC-V IR Decoder
//
// Translates ARM64 instructions into RISC-V IR basic blocks.
// ARM64 is already RISC-like, so the mapping is more direct than x86.
//
// Register mapping (ARM64 → RISC-V):
//   ARM X0-X7   (args/result)  → RV x10-x17 (a0-a7)
//   ARM X8      (indirect result/syscall#) → RV x17 (a7) for syscall, x28 (t3) otherwise
//   ARM X9-X15  (temp)         → RV x5-x7, x28-x31 (t0-t6)
//   ARM X16-X17 (IP0/IP1)     → RV x6-x7 (t1-t2)
//   ARM X18     (platform)    → RV x4 (tp)
//   ARM X19-X28 (callee-saved)→ RV x18-x27 (s2-s11)
//   ARM X29/FP                → RV x8 (fp/s0)
//   ARM X30/LR                → RV x1 (ra)
//   ARM SP                    → RV x2 (sp)
//   ARM XZR                   → RV x0 (zero)
//
// Syscall ABI translation:
//   ARM: syscall# in X8, args in X0-X5    (SVC #0)
//   RV:  syscall# in a7, args in a0-a5    (ecall)
//   → Nearly 1:1 mapping!

use alloc::vec::Vec;
use alloc::format;
use alloc::string::String;
use super::ir::*;

/// ARM64 register to RISC-V register mapping
fn arm_to_rv(arm_reg: u8) -> Reg {
    match arm_reg {
        0  => Reg::X10, // X0 → a0
        1  => Reg::X11, // X1 → a1
        2  => Reg::X12, // X2 → a2
        3  => Reg::X13, // X3 → a3
        4  => Reg::X14, // X4 → a4
        5  => Reg::X15, // X5 → a5
        6  => Reg::X16, // X6 → a6
        7  => Reg::X17, // X7 → a7
        8  => Reg::X28, // X8 (syscall#) → t3
        9  => Reg::X5,  // X9 → t0
        10 => Reg::X6,  // X10 → t1
        11 => Reg::X7,  // X11 → t2
        12 => Reg::X29, // X12 → t4
        13 => Reg::X30, // X13 → t5
        14 => Reg::X31, // X14 → t6
        15 => Reg::X9,  // X15 → s1
        16 => Reg::X6,  // IP0 → t1
        17 => Reg::X7,  // IP1 → t2
        18 => Reg::X4,  // platform → tp
        19 => Reg::X18, // X19 → s2
        20 => Reg::X19, // X20 → s3
        21 => Reg::X20, // X21 → s4
        22 => Reg::X21, // X22 → s5
        23 => Reg::X22, // X23 → s6
        24 => Reg::X23, // X24 → s7
        25 => Reg::X24, // X25 → s8
        26 => Reg::X25, // X26 → s9
        27 => Reg::X26, // X27 → s10
        28 => Reg::X27, // X28 → s11
        29 => Reg::X8,  // FP  → fp
        30 => Reg::X1,  // LR  → ra
        31 => Reg::X0,  // XZR/SP (context-dependent)
        _  => Reg::X0,
    }
}

/// When register 31 means SP (not XZR)
fn arm_sp_or_zr(arm_reg: u8, is_sp: bool) -> Reg {
    if arm_reg == 31 {
        if is_sp { Reg::X2 } else { Reg::X0 }
    } else {
        arm_to_rv(arm_reg)
    }
}

/// AArch64 → RISC-V IR translator
pub struct ArmDecoder {
    /// Raw binary code (ARM64 is fixed-width 32-bit)
    code: Vec<u8>,
    /// Base virtual address
    base_addr: u64,
    /// Current instruction index (in 4-byte units)
    offset: usize,
    /// Translation statistics
    pub stats: TranslationStats,
}

impl ArmDecoder {
    pub fn new(code: &[u8], base_addr: u64) -> Self {
        Self {
            code: code.to_vec(),
            base_addr,
            offset: 0,
            stats: TranslationStats::default(),
        }
    }

    /// Translate a basic block starting at the given byte offset
    pub fn translate_block(&mut self, start_offset: usize) -> TranslatedBlock {
        self.offset = start_offset;
        let src_addr = self.base_addr + start_offset as u64;
        let mut block = TranslatedBlock::new(src_addr, SourceArch::Aarch64);

        let max_instructions = 256;
        let mut count = 0;

        while self.offset + 4 <= self.code.len() && count < max_instructions {
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

    /// Translate all reachable blocks
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

            for &succ in &block.successors {
                if succ >= self.base_addr {
                    let succ_off = (succ - self.base_addr) as usize;
                    if succ_off + 4 <= self.code.len() && !visited.contains(&succ) {
                        worklist.push(succ_off);
                    }
                }
            }

            blocks.push(block);
        }

        blocks
    }

    /// Read one 32-bit ARM64 instruction
    fn fetch(&mut self) -> u32 {
        if self.offset + 4 > self.code.len() {
            return 0;
        }
        let inst = u32::from_le_bytes([
            self.code[self.offset],
            self.code[self.offset + 1],
            self.code[self.offset + 2],
            self.code[self.offset + 3],
        ]);
        self.offset += 4;
        inst
    }

    /// Decode one AArch64 instruction and emit RISC-V IR.
    /// Returns true if the instruction terminates the basic block.
    fn decode_one(&mut self, block: &mut TranslatedBlock) -> bool {
        let inst_addr = self.base_addr + self.offset as u64;
        let inst = self.fetch();

        if inst == 0 {
            block.emit(RvInst::Nop);
            return true;
        }

        // Top-level decode based on bits [28:25] (the "op0" field)
        let op0 = (inst >> 25) & 0xF;

        match op0 {
            // Data processing — immediate
            0b1000 | 0b1001 => self.decode_dp_imm(inst, inst_addr, block),

            // Branches, exception generation, system
            0b1010 | 0b1011 => return self.decode_branch(inst, inst_addr, block),

            // Loads and stores
            0b0100 | 0b0110 | 0b1100 | 0b1110 => self.decode_ldst(inst, inst_addr, block),

            // Data processing — register
            0b0101 | 0b1101 => self.decode_dp_reg(inst, inst_addr, block),

            _ => {
                self.stats.unsupported_instructions += 1;
                block.emit(RvInst::SrcAnnotation {
                    arch: SourceArch::Aarch64,
                    addr: inst_addr,
                    text: format!("unsupported: 0x{:08X}", inst),
                });
                block.emit(RvInst::Nop);
            }
        }

        false
    }

    /// Decode data processing — immediate group
    fn decode_dp_imm(&mut self, inst: u32, addr: u64, block: &mut TranslatedBlock) {
        let opc = (inst >> 23) & 0x7;
        let sf = (inst >> 31) & 1; // 1=64-bit, 0=32-bit
        let rd = (inst & 0x1F) as u8;
        let rn = ((inst >> 5) & 0x1F) as u8;

        match opc {
            // ADD/SUB immediate — x01 000 1xx
            0b010 | 0b011 => {
                let shift = ((inst >> 22) & 1) as u8;
                let imm12 = ((inst >> 10) & 0xFFF) as i64;
                let imm = if shift == 1 { imm12 << 12 } else { imm12 };
                let is_sub = (inst >> 30) & 1 == 1;
                let sets_flags = (inst >> 29) & 1 == 1;

                let rv_rd = arm_sp_or_zr(rd, !sets_flags);
                let rv_rn = arm_sp_or_zr(rn, true);

                if is_sub {
                    block.emit(RvInst::Addi { rd: rv_rd, rs1: rv_rn, imm: -imm });
                } else {
                    block.emit(RvInst::Addi { rd: rv_rd, rs1: rv_rn, imm });
                }

                if sets_flags {
                    block.emit(RvInst::CmpFlags { rs1: rv_rd, rs2: Reg::X0 });
                }
            }

            // MOV wide immediate (MOVZ/MOVK/MOVN) — x01 001 0x
            0b100 | 0b101 => {
                let hw = ((inst >> 21) & 0x3) as u8;
                let imm16 = ((inst >> 5) & 0xFFFF) as i64;
                let opc2 = (inst >> 29) & 0x3;
                let rv_rd = arm_to_rv(rd);

                let shifted = imm16 << (hw * 16);

                match opc2 {
                    0b00 => { // MOVN — move NOT
                        block.emit(RvInst::Li { rd: rv_rd, imm: !shifted });
                    }
                    0b10 => { // MOVZ — move zero
                        block.emit(RvInst::Li { rd: rv_rd, imm: shifted });
                    }
                    0b11 => { // MOVK — move keep (insert bits)
                        let mask = !(0xFFFF_i64 << (hw * 16));
                        block.emit(RvInst::Li { rd: Reg::X5, imm: mask });
                        block.emit(RvInst::And { rd: rv_rd, rs1: rv_rd, rs2: Reg::X5 });
                        block.emit(RvInst::Li { rd: Reg::X5, imm: shifted });
                        block.emit(RvInst::Or { rd: rv_rd, rs1: rv_rd, rs2: Reg::X5 });
                    }
                    _ => {
                        block.emit(RvInst::Nop);
                    }
                }
            }

            // Logical immediate — x01 001 00
            0b110 => {
                // Simplified: just handle common cases
                let rv_rd = arm_to_rv(rd);
                let rv_rn = arm_to_rv(rn);
                let log_opc = (inst >> 29) & 0x3;
                // Decode bitmask immediate (complex, simplified here)
                let imm_val = decode_bitmask_imm(inst, sf == 1);

                match log_opc {
                    0b00 => block.emit(RvInst::Andi { rd: rv_rd, rs1: rv_rn, imm: imm_val }),
                    0b01 => block.emit(RvInst::Ori { rd: rv_rd, rs1: rv_rn, imm: imm_val }),
                    0b10 => block.emit(RvInst::Xori { rd: rv_rd, rs1: rv_rn, imm: imm_val }),
                    0b11 => { // ANDS
                        block.emit(RvInst::Andi { rd: rv_rd, rs1: rv_rn, imm: imm_val });
                        block.emit(RvInst::CmpFlags { rs1: rv_rd, rs2: Reg::X0 });
                    }
                    _ => {}
                }
            }

            // ADR/ADRP — x00 100 00
            0b000 | 0b001 => {
                let rv_rd = arm_to_rv(rd);
                let immhi = ((inst >> 5) & 0x7FFFF) as i64;
                let immlo = ((inst >> 29) & 0x3) as i64;
                let is_adrp = (inst >> 31) & 1 == 1;
                let mut imm = (immhi << 2) | immlo;
                // Sign extend from 21 bits
                if imm & (1 << 20) != 0 { imm |= !0x1FFFFF; }
                if is_adrp { imm <<= 12; }
                let target = addr as i64 + imm;
                block.emit(RvInst::Li { rd: rv_rd, imm: target });
            }

            _ => {
                self.stats.unsupported_instructions += 1;
                block.emit(RvInst::Nop);
            }
        }
    }

    /// Decode branches, exceptions, system instructions
    fn decode_branch(&mut self, inst: u32, addr: u64, block: &mut TranslatedBlock) -> bool {
        let op1 = (inst >> 29) & 0x7;

        match op1 {
            // B / BL — unconditional branch
            0b000 | 0b100 => {
                let mut imm26 = (inst & 0x03FF_FFFF) as i64;
                if imm26 & (1 << 25) != 0 { imm26 |= !0x03FF_FFFF; }
                let target = addr as i64 + (imm26 << 2);
                let is_bl = (inst >> 31) & 1 == 1;

                if is_bl {
                    // BL — branch and link (call)
                    block.emit(RvInst::Jal { rd: Reg::X1, offset: target });
                } else {
                    // B — unconditional branch
                    block.emit(RvInst::Jal { rd: Reg::X0, offset: target });
                }

                block.successors.push(target as u64);
                if is_bl {
                    block.successors.push(addr + 4);
                }
                true
            }

            // B.cond — conditional branch
            0b010 => {
                let mut imm19 = ((inst >> 5) & 0x7FFFF) as i64;
                if imm19 & (1 << 18) != 0 { imm19 |= !0x7FFFF; }
                let target = addr as i64 + (imm19 << 2);
                let cond = (inst & 0xF) as u8;

                let flag_cond = arm_cc_to_flag(cond);
                block.emit(RvInst::BranchCond { cond: flag_cond, offset: target });
                block.successors.push(target as u64);
                block.successors.push(addr + 4);
                true
            }

            // CBZ/CBNZ — compare and branch
            0b001 | 0b101 => {
                let rt = (inst & 0x1F) as u8;
                let mut imm19 = ((inst >> 5) & 0x7FFFF) as i64;
                if imm19 & (1 << 18) != 0 { imm19 |= !0x7FFFF; }
                let target = addr as i64 + (imm19 << 2);
                let is_cbnz = (inst >> 24) & 1 == 1;
                let rv_rt = arm_to_rv(rt);

                if is_cbnz {
                    block.emit(RvInst::Bne { rs1: rv_rt, rs2: Reg::X0, offset: target });
                } else {
                    block.emit(RvInst::Beq { rs1: rv_rt, rs2: Reg::X0, offset: target });
                }

                block.successors.push(target as u64);
                block.successors.push(addr + 4);
                true
            }

            // SVC / HVC / SMC / system instructions
            0b110 => {
                let op2 = (inst >> 21) & 0x7;
                if op2 == 0 {
                    // Exception generation
                    let opc = (inst >> 21) & 0x7;
                    let ll = inst & 0x3;
                    if ll == 1 {
                        // SVC #imm16 — supervisor call (syscall)
                        block.emit(RvInst::SrcAnnotation {
                            arch: SourceArch::Aarch64,
                            addr,
                            text: String::from("SVC #0 (syscall)"),
                        });
                        // ARM syscall ABI: X8=syscall#, X0-X5=args
                        // RV ABI: a7=syscall#, a0-a5=args
                        // X8 is mapped to x28(t3), need to move to x17(a7)
                        block.emit(RvInst::Mv { rd: Reg::X17, rs: Reg::X28 });
                        // X0-X5 are already mapped to a0-a5 (x10-x15) — perfect!
                        block.emit(RvInst::Ecall);
                        // Return value: a0(x10) = X0 — already correct!
                        return false;
                    } else if ll == 0 && opc == 0 {
                        // SVC with different encoding
                        block.emit(RvInst::Ecall);
                        return false;
                    }
                }

                // RET — encoded as BR X30 (ERET or RET)
                if (inst & 0xFFFFFC1F) == 0xD65F0000 {
                    block.emit(RvInst::Ret);
                    return true;
                }

                // BR Xn — branch to register
                if (inst & 0xFFFFFC00) == 0xD61F0000 {
                    let rn = ((inst >> 5) & 0x1F) as u8;
                    let rv_rn = arm_to_rv(rn);
                    block.emit(RvInst::Jalr { rd: Reg::X0, rs1: rv_rn, offset: 0 });
                    return true;
                }

                // BLR Xn — branch-link to register
                if (inst & 0xFFFFFC00) == 0xD63F0000 {
                    let rn = ((inst >> 5) & 0x1F) as u8;
                    let rv_rn = arm_to_rv(rn);
                    block.emit(RvInst::Jalr { rd: Reg::X1, rs1: rv_rn, offset: 0 });
                    block.successors.push(addr + 4);
                    return true;
                }

                // NOP, WFI, etc.
                if inst == 0xD503201F { // NOP
                    block.emit(RvInst::Nop);
                    return false;
                }
                if inst == 0xD503207F { // WFI
                    block.emit(RvInst::Nop);
                    return false;
                }

                self.stats.unsupported_instructions += 1;
                block.emit(RvInst::Nop);
                false
            }

            // TBZ/TBNZ — test bit and branch
            0b011 | 0b111 => {
                let rt = (inst & 0x1F) as u8;
                let bit = ((inst >> 19) & 0x1F) as u8 | (((inst >> 31) & 1) as u8) << 5;
                let mut imm14 = ((inst >> 5) & 0x3FFF) as i64;
                if imm14 & (1 << 13) != 0 { imm14 |= !0x3FFF; }
                let target = addr as i64 + (imm14 << 2);
                let is_tbnz = (inst >> 24) & 1 == 1;

                let rv_rt = arm_to_rv(rt);
                // Test bit: AND with mask, then branch
                block.emit(RvInst::Li { rd: Reg::X5, imm: 1 << bit });
                block.emit(RvInst::And { rd: Reg::X5, rs1: rv_rt, rs2: Reg::X5 });

                if is_tbnz {
                    block.emit(RvInst::Bne { rs1: Reg::X5, rs2: Reg::X0, offset: target });
                } else {
                    block.emit(RvInst::Beq { rs1: Reg::X5, rs2: Reg::X0, offset: target });
                }

                block.successors.push(target as u64);
                block.successors.push(addr + 4);
                true
            }

            _ => {
                self.stats.unsupported_instructions += 1;
                block.emit(RvInst::Nop);
                false
            }
        }
    }

    /// Decode load/store instructions
    fn decode_ldst(&mut self, inst: u32, addr: u64, block: &mut TranslatedBlock) {
        let size = (inst >> 30) & 0x3;
        let opc = (inst >> 22) & 0x3;
        let rn = ((inst >> 5) & 0x1F) as u8;
        let rt = (inst & 0x1F) as u8;

        let rv_rt = arm_to_rv(rt);
        let rv_rn = arm_sp_or_zr(rn, true);

        // Load/Store with unsigned offset
        if (inst & 0x3B000000) == 0x39000000 {
            let imm12 = ((inst >> 10) & 0xFFF) as i64;
            let scale = size as i64;
            let offset = imm12 << scale;
            let is_load = (opc & 1) == 1;

            if is_load {
                match size {
                    0 => block.emit(RvInst::Lbu { rd: rv_rt, rs1: rv_rn, offset }),
                    1 => block.emit(RvInst::Lhu { rd: rv_rt, rs1: rv_rn, offset }),
                    2 => block.emit(RvInst::Lwu { rd: rv_rt, rs1: rv_rn, offset }),
                    3 => block.emit(RvInst::Ld { rd: rv_rt, rs1: rv_rn, offset }),
                    _ => {}
                }
            } else {
                match size {
                    0 => block.emit(RvInst::Sb { rs2: rv_rt, rs1: rv_rn, offset }),
                    1 => block.emit(RvInst::Sh { rs2: rv_rt, rs1: rv_rn, offset }),
                    2 => block.emit(RvInst::Sw { rs2: rv_rt, rs1: rv_rn, offset }),
                    3 => block.emit(RvInst::Sd { rs2: rv_rt, rs1: rv_rn, offset }),
                    _ => {}
                }
            }
            return;
        }

        // Load/Store with pre/post-index
        if (inst & 0x3B200C00) == 0x38000000 || (inst & 0x3B200C00) == 0x38000400 {
            let mut imm9 = ((inst >> 12) & 0x1FF) as i64;
            if imm9 & (1 << 8) != 0 { imm9 |= !0x1FF; }
            let is_pre = (inst >> 11) & 1 == 1;
            let is_load = (opc & 1) == 1;

            if is_pre {
                // Pre-index: update base first
                block.emit(RvInst::Addi { rd: rv_rn, rs1: rv_rn, imm: imm9 });
            }

            if is_load {
                match size {
                    0 => block.emit(RvInst::Lbu { rd: rv_rt, rs1: rv_rn, offset: if is_pre { 0 } else { imm9 } }),
                    1 => block.emit(RvInst::Lhu { rd: rv_rt, rs1: rv_rn, offset: if is_pre { 0 } else { imm9 } }),
                    2 => block.emit(RvInst::Lwu { rd: rv_rt, rs1: rv_rn, offset: if is_pre { 0 } else { imm9 } }),
                    3 => block.emit(RvInst::Ld { rd: rv_rt, rs1: rv_rn, offset: if is_pre { 0 } else { imm9 } }),
                    _ => {}
                }
            } else {
                match size {
                    0 => block.emit(RvInst::Sb { rs2: rv_rt, rs1: rv_rn, offset: if is_pre { 0 } else { imm9 } }),
                    1 => block.emit(RvInst::Sh { rs2: rv_rt, rs1: rv_rn, offset: if is_pre { 0 } else { imm9 } }),
                    2 => block.emit(RvInst::Sw { rs2: rv_rt, rs1: rv_rn, offset: if is_pre { 0 } else { imm9 } }),
                    3 => block.emit(RvInst::Sd { rs2: rv_rt, rs1: rv_rn, offset: if is_pre { 0 } else { imm9 } }),
                    _ => {}
                }
            }

            if !is_pre {
                // Post-index: update base after
                block.emit(RvInst::Addi { rd: rv_rn, rs1: rv_rn, imm: imm9 });
            }
            return;
        }

        // LDP/STP — load/store pair
        if (inst & 0x3E000000) == 0x28000000 || (inst & 0x3E000000) == 0x2C000000 {
            let rt2 = ((inst >> 10) & 0x1F) as u8;
            let mut imm7 = ((inst >> 15) & 0x7F) as i64;
            if imm7 & (1 << 6) != 0 { imm7 |= !0x7F; }
            let scale = if (inst >> 31) & 1 == 1 { 3 } else { 2 };
            let offset = imm7 << scale;
            let is_load = (inst >> 22) & 1 == 1;

            let rv_rt2 = arm_to_rv(rt2);

            if is_load {
                block.emit(RvInst::Ld { rd: rv_rt, rs1: rv_rn, offset });
                block.emit(RvInst::Ld { rd: rv_rt2, rs1: rv_rn, offset: offset + 8 });
            } else {
                block.emit(RvInst::Sd { rs2: rv_rt, rs1: rv_rn, offset });
                block.emit(RvInst::Sd { rs2: rv_rt2, rs1: rv_rn, offset: offset + 8 });
            }
            return;
        }

        // Fallthrough — unsupported load/store variant
        self.stats.unsupported_instructions += 1;
        block.emit(RvInst::SrcAnnotation {
            arch: SourceArch::Aarch64,
            addr,
            text: format!("unsupported ldst: 0x{:08X}", inst),
        });
        block.emit(RvInst::Nop);
    }

    /// Decode data processing — register group
    fn decode_dp_reg(&mut self, inst: u32, addr: u64, block: &mut TranslatedBlock) {
        let rd = (inst & 0x1F) as u8;
        let rn = ((inst >> 5) & 0x1F) as u8;
        let rm = ((inst >> 16) & 0x1F) as u8;
        let opc = (inst >> 29) & 0x7;

        let rv_rd = arm_to_rv(rd);
        let rv_rn = arm_to_rv(rn);
        let rv_rm = arm_to_rv(rm);

        // Logical (shifted register) — 0xx01010
        if (inst & 0x1F000000) == 0x0A000000 {
            let log_opc = (inst >> 29) & 0x3;
            let n = (inst >> 21) & 1;
            let sets_flags = log_opc == 0b11;

            match (log_opc, n) {
                (0b00, 0) => block.emit(RvInst::And { rd: rv_rd, rs1: rv_rn, rs2: rv_rm }),
                (0b01, 0) => block.emit(RvInst::Or { rd: rv_rd, rs1: rv_rn, rs2: rv_rm }),
                (0b10, 0) => block.emit(RvInst::Xor { rd: rv_rd, rs1: rv_rn, rs2: rv_rm }),
                (0b11, 0) => { // ANDS
                    block.emit(RvInst::And { rd: rv_rd, rs1: rv_rn, rs2: rv_rm });
                }
                // Inverted variants (BIC, ORN, EON, BICS)
                (_, 1) => {
                    block.emit(RvInst::Xori { rd: Reg::X5, rs1: rv_rm, imm: -1 });
                    match log_opc {
                        0b00 => block.emit(RvInst::And { rd: rv_rd, rs1: rv_rn, rs2: Reg::X5 }),
                        0b01 => block.emit(RvInst::Or { rd: rv_rd, rs1: rv_rn, rs2: Reg::X5 }),
                        0b10 => block.emit(RvInst::Xor { rd: rv_rd, rs1: rv_rn, rs2: Reg::X5 }),
                        0b11 => block.emit(RvInst::And { rd: rv_rd, rs1: rv_rn, rs2: Reg::X5 }),
                        _ => {}
                    }
                }
                _ => { block.emit(RvInst::Nop); }
            }

            if sets_flags {
                block.emit(RvInst::CmpFlags { rs1: rv_rd, rs2: Reg::X0 });
            }
            return;
        }

        // Add/Sub (shifted register) — x0x01011
        if (inst & 0x1F000000) == 0x0B000000 {
            let is_sub = (inst >> 30) & 1 == 1;
            let sets_flags = (inst >> 29) & 1 == 1;

            // Handle shift
            let shift_type = ((inst >> 22) & 0x3) as u8;
            let shift_amt = ((inst >> 10) & 0x3F) as u8;

            if shift_amt > 0 {
                match shift_type {
                    0 => block.emit(RvInst::Slli { rd: Reg::X5, rs1: rv_rm, shamt: shift_amt }),
                    1 => block.emit(RvInst::Srli { rd: Reg::X5, rs1: rv_rm, shamt: shift_amt }),
                    2 => block.emit(RvInst::Srai { rd: Reg::X5, rs1: rv_rm, shamt: shift_amt }),
                    _ => block.emit(RvInst::Mv { rd: Reg::X5, rs: rv_rm }),
                }
                if is_sub {
                    block.emit(RvInst::Sub { rd: rv_rd, rs1: rv_rn, rs2: Reg::X5 });
                } else {
                    block.emit(RvInst::Add { rd: rv_rd, rs1: rv_rn, rs2: Reg::X5 });
                }
            } else {
                if is_sub {
                    block.emit(RvInst::Sub { rd: rv_rd, rs1: rv_rn, rs2: rv_rm });
                } else {
                    block.emit(RvInst::Add { rd: rv_rd, rs1: rv_rn, rs2: rv_rm });
                }
            }

            if sets_flags {
                block.emit(RvInst::CmpFlags { rs1: rv_rd, rs2: Reg::X0 });
            }
            return;
        }

        // MUL/MADD/MSUB — x0011011000
        if (inst & 0x7FE00000) == 0x1B000000 {
            let ra = ((inst >> 10) & 0x1F) as u8;
            let is_msub = (inst >> 15) & 1 == 1;

            if ra == 31 {
                // MUL (MADD with XZR)
                block.emit(RvInst::Mul { rd: rv_rd, rs1: rv_rn, rs2: rv_rm });
            } else {
                let rv_ra = arm_to_rv(ra);
                block.emit(RvInst::Mul { rd: Reg::X5, rs1: rv_rn, rs2: rv_rm });
                if is_msub {
                    block.emit(RvInst::Sub { rd: rv_rd, rs1: rv_ra, rs2: Reg::X5 });
                } else {
                    block.emit(RvInst::Add { rd: rv_rd, rs1: rv_ra, rs2: Reg::X5 });
                }
            }
            return;
        }

        // SDIV/UDIV — x0011010110
        if (inst & 0x7FE0FC00) == 0x1AC00800 {
            let is_udiv = (inst >> 10) & 1 == 0;
            if is_udiv {
                block.emit(RvInst::Divu { rd: rv_rd, rs1: rv_rn, rs2: rv_rm });
            } else {
                block.emit(RvInst::Div { rd: rv_rd, rs1: rv_rn, rs2: rv_rm });
            }
            return;
        }

        // Shift register (LSL, LSR, ASR, ROR)
        if (inst & 0x7FE0F000) == 0x1AC02000 {
            let shift_opc = ((inst >> 10) & 0x3) as u8;
            match shift_opc {
                0 => block.emit(RvInst::Sll { rd: rv_rd, rs1: rv_rn, rs2: rv_rm }),
                1 => block.emit(RvInst::Srl { rd: rv_rd, rs1: rv_rn, rs2: rv_rm }),
                2 => block.emit(RvInst::Sra { rd: rv_rd, rs1: rv_rn, rs2: rv_rm }),
                // ROR needs special handling
                3 => {
                    block.emit(RvInst::Srl { rd: Reg::X5, rs1: rv_rn, rs2: rv_rm });
                    block.emit(RvInst::Li { rd: Reg::X6, imm: 64 });
                    block.emit(RvInst::Sub { rd: Reg::X6, rs1: Reg::X6, rs2: rv_rm });
                    block.emit(RvInst::Sll { rd: Reg::X6, rs1: rv_rn, rs2: Reg::X6 });
                    block.emit(RvInst::Or { rd: rv_rd, rs1: Reg::X5, rs2: Reg::X6 });
                }
                _ => {}
            }
            return;
        }

        // Fallthrough — unsupported
        self.stats.unsupported_instructions += 1;
        block.emit(RvInst::SrcAnnotation {
            arch: SourceArch::Aarch64,
            addr,
            text: format!("unsupported dp_reg: 0x{:08X}", inst),
        });
        block.emit(RvInst::Nop);
    }
}

/// Map ARM64 condition codes to flag conditions
fn arm_cc_to_flag(cc: u8) -> FlagCond {
    match cc {
        0x0 => FlagCond::Eq,     // EQ (Z=1)
        0x1 => FlagCond::Ne,     // NE (Z=0)
        0x2 => FlagCond::Geu,    // CS/HS (C=1)
        0x3 => FlagCond::Ltu,    // CC/LO (C=0)
        0x4 => FlagCond::Neg,    // MI (N=1)
        0x5 => FlagCond::Pos,    // PL (N=0)
        0x6 => FlagCond::Ovf,    // VS (V=1)
        0x7 => FlagCond::NoOvf,  // VC (V=0)
        0x8 => FlagCond::Gt,     // HI (C=1 & Z=0)
        0x9 => FlagCond::Le,     // LS (C=0 | Z=1)
        0xA => FlagCond::Ge,     // GE (N=V)
        0xB => FlagCond::Lt,     // LT (N!=V)
        0xC => FlagCond::Gt,     // GT (Z=0 & N=V)
        0xD => FlagCond::Le,     // LE (Z=1 | N!=V)
        0xE => FlagCond::Eq,     // AL (always) — use Eq with tautology
        _   => FlagCond::Eq,
    }
}

/// Decode ARM64 bitmask immediate (simplified — returns common patterns)
fn decode_bitmask_imm(inst: u32, _is_64: bool) -> i64 {
    let n = (inst >> 22) & 1;
    let immr = ((inst >> 16) & 0x3F) as u32;
    let imms = ((inst >> 10) & 0x3F) as u32;

    // Full decode is complex (rotating bitmasks). Handle common patterns:
    let len = if n == 1 { 6 } else {
        // Find highest bit in NOT(imms)
        let not_imms = !imms & 0x3F;
        if not_imms & 0x20 != 0 { 5 }
        else if not_imms & 0x10 != 0 { 4 }
        else if not_imms & 0x08 != 0 { 3 }
        else if not_imms & 0x04 != 0 { 2 }
        else { 1 }
    };

    let size = 1u64 << len;
    let mask = size - 1;
    let s = (imms & mask as u32) as u64;
    let r = (immr & mask as u32) as u64;

    let mut welem: u64 = (1u64 << (s + 1)) - 1;
    // Rotate right by r
    if r > 0 {
        welem = (welem >> r) | (welem << (size - r));
        welem &= (1u64 << size) - 1;
    }

    // Replicate to 64 bits
    let mut result = welem;
    let mut cur_size = size;
    while cur_size < 64 {
        result |= result << cur_size;
        cur_size *= 2;
    }

    result as i64
}
