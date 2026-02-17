//! x86-64 MMIO Instruction Decoder for TrustVM
//!
//! When a guest triggers a Nested Page Fault (#NPF) on an MMIO region,
//! we need to decode the faulting instruction to determine:
//!   - Is it a read or a write?
//!   - Which guest register is the source/destination?
//!   - What is the operand size (1/2/4/8 bytes)?
//!   - How long is the instruction (to advance RIP)?
//!
//! AMD SVM provides the faulting instruction bytes in the VMCB control area
//! (GUEST_INST_BYTES at offset 0x0D1, up to 15 bytes, count at 0x0D0).
//!
//! We decode the most common MOV instruction patterns:
//!   - MOV r/m, reg  (opcode 0x89)  — write to MMIO
//!   - MOV reg, r/m  (opcode 0x8B)  — read from MMIO
//!   - MOV r/m, imm32 (opcode 0xC7) — write immediate to MMIO
//!   - MOVZX reg, r/m8  (0x0F 0xB6) — byte read from MMIO
//!   - MOVZX reg, r/m16 (0x0F 0xB7) — word read from MMIO
//!
//! We handle x86-64 prefixes: REX, operand-size override (0x66).
//!
//! References:
//!   - Intel SDM Vol.2: Instruction Set Reference
//!   - AMD APM Vol.3: General-Purpose and System Instructions

/// Decoded MMIO instruction
#[derive(Debug, Clone)]
pub struct MmioDecoded {
    /// True = guest writes to MMIO, False = guest reads from MMIO
    pub is_write: bool,
    /// Operand size in bytes (1, 2, 4, or 8)
    pub operand_size: u8,
    /// Register index (0=RAX, 1=RCX, 2=RDX, 3=RBX, 4=RSP, 5=RBP, 6=RSI, 7=RDI,
    /// 8-15 = R8-R15). For immediate writes, this is None.
    pub register: Option<u8>,
    /// Immediate value (for MOV r/m, imm32)
    pub immediate: Option<u64>,
    /// Total instruction length (to advance RIP)
    pub insn_len: usize,
}

/// Register indices used in ModR/M encoding
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum GpReg {
    Rax = 0, Rcx = 1, Rdx = 2, Rbx = 3,
    Rsp = 4, Rbp = 5, Rsi = 6, Rdi = 7,
    R8 = 8, R9 = 9, R10 = 10, R11 = 11,
    R12 = 12, R13 = 13, R14 = 14, R15 = 15,
}

/// Decode an MMIO-faulting instruction from the VMCB guest instruction bytes.
///
/// `insn_bytes`: up to 15 bytes from VMCB GUEST_INST_BYTES
/// `bytes_fetched`: number of valid bytes
/// `cs_d`: true if the CS segment has the D (default operand size 32-bit) bit set
///         (always true in 64-bit long mode for our purposes)
///
/// Returns `Some(MmioDecoded)` on success, `None` if we can't decode.
pub fn decode_mmio_instruction(insn_bytes: &[u8], bytes_fetched: usize, cs_long: bool) -> Option<MmioDecoded> {
    if bytes_fetched == 0 || insn_bytes.is_empty() {
        return None;
    }
    
    let bytes = &insn_bytes[..bytes_fetched.min(insn_bytes.len())];
    let mut pos: usize = 0;
    
    // ── Parse prefixes ──────────────────────────────────────────
    let mut has_rex = false;
    let mut rex_w = false;   // 64-bit operand size
    let mut rex_r = false;   // ModR/M reg extension
    let mut rex_b = false;   // ModR/M rm extension
    let mut has_66 = false;  // Operand-size override (32→16 in long mode)
    let mut has_67 = false;  // Address-size override
    let mut _has_f0 = false;  // LOCK prefix
    let mut _has_f2 = false;  // REPNE
    let mut _has_f3 = false;  // REP
    // Segment override prefixes (2E, 3E, 26, 64, 65, 36) — skip
    
    while pos < bytes.len() {
        match bytes[pos] {
            0x66 => { has_66 = true; pos += 1; }
            0x67 => { has_67 = true; pos += 1; }
            0xF0 => { _has_f0 = true; pos += 1; }
            0xF2 => { _has_f2 = true; pos += 1; }
            0xF3 => { _has_f3 = true; pos += 1; }
            // Segment overrides — skip
            0x2E | 0x3E | 0x26 | 0x36 | 0x64 | 0x65 => { pos += 1; }
            // REX prefix: 0x40-0x4F
            b @ 0x40..=0x4F => {
                has_rex = true;
                rex_w = (b & 0x08) != 0;
                rex_r = (b & 0x04) != 0;
                rex_b = (b & 0x01) != 0;
                pos += 1;
            }
            _ => break, // Not a prefix, start of opcode
        }
    }
    
    if pos >= bytes.len() {
        return None;
    }
    
    // ── Determine default operand size ──────────────────────────
    // In 64-bit long mode: default is 32-bit, REX.W makes 64-bit, 0x66 makes 16-bit
    let operand_size: u8 = if rex_w {
        8
    } else if has_66 {
        2
    } else {
        4
    };
    
    let opcode = bytes[pos];
    pos += 1;
    
    match opcode {
        // ── MOV r/m32, r32 (opcode 0x89) — WRITE to MMIO ───────
        0x89 => {
            if pos >= bytes.len() { return None; }
            let modrm = bytes[pos];
            pos += 1;
            let (reg_idx, insn_len) = decode_modrm_reg_and_skip(modrm, rex_r, rex_b, has_67, bytes, pos)?;
            Some(MmioDecoded {
                is_write: true,
                operand_size,
                register: Some(reg_idx),
                immediate: None,
                insn_len,
            })
        }
        
        // ── MOV r/m8, r8 (opcode 0x88) — WRITE byte to MMIO ───
        0x88 => {
            if pos >= bytes.len() { return None; }
            let modrm = bytes[pos];
            pos += 1;
            let (reg_idx, insn_len) = decode_modrm_reg_and_skip(modrm, rex_r, rex_b, has_67, bytes, pos)?;
            Some(MmioDecoded {
                is_write: true,
                operand_size: 1,
                register: Some(reg_idx),
                immediate: None,
                insn_len,
            })
        }
        
        // ── MOV r32, r/m32 (opcode 0x8B) — READ from MMIO ──────
        0x8B => {
            if pos >= bytes.len() { return None; }
            let modrm = bytes[pos];
            pos += 1;
            let (reg_idx, insn_len) = decode_modrm_reg_and_skip(modrm, rex_r, rex_b, has_67, bytes, pos)?;
            Some(MmioDecoded {
                is_write: false,
                operand_size,
                register: Some(reg_idx),
                immediate: None,
                insn_len,
            })
        }
        
        // ── MOV r8, r/m8 (opcode 0x8A) — READ byte from MMIO ──
        0x8A => {
            if pos >= bytes.len() { return None; }
            let modrm = bytes[pos];
            pos += 1;
            let (reg_idx, insn_len) = decode_modrm_reg_and_skip(modrm, rex_r, rex_b, has_67, bytes, pos)?;
            Some(MmioDecoded {
                is_write: false,
                operand_size: 1,
                register: Some(reg_idx),
                immediate: None,
                insn_len,
            })
        }
        
        // ── MOV r/m32, imm32 (opcode 0xC7 /0) — WRITE immediate ──
        0xC7 => {
            if pos >= bytes.len() { return None; }
            let modrm = bytes[pos];
            let reg_field = (modrm >> 3) & 7;
            if reg_field != 0 { return None; } // Only /0 is MOV
            pos += 1;
            let (_, base_insn_len) = decode_modrm_reg_and_skip(modrm, false, rex_b, has_67, bytes, pos)?;
            // After ModR/M + SIB + displacement, there's the immediate
            let imm_start = base_insn_len; // instruction bytes consumed so far
            let imm_size = if rex_w { 4 } else if has_66 { 2 } else { 4 }; // Note: even with REX.W, MOV r/m64,imm32 sign-extends
            if imm_start + imm_size > bytes.len() { return None; }
            let imm = match imm_size {
                2 => u16::from_le_bytes([bytes[imm_start], bytes[imm_start + 1]]) as u64,
                4 => {
                    let v = u32::from_le_bytes([
                        bytes[imm_start], bytes[imm_start + 1],
                        bytes[imm_start + 2], bytes[imm_start + 3],
                    ]);
                    if rex_w {
                        // Sign-extend to 64-bit
                        v as i32 as i64 as u64
                    } else {
                        v as u64
                    }
                }
                _ => return None,
            };
            Some(MmioDecoded {
                is_write: true,
                operand_size,
                register: None,
                immediate: Some(imm),
                insn_len: imm_start + imm_size,
            })
        }
        
        // ── MOV r/m8, imm8 (opcode 0xC6 /0) — WRITE byte immediate ──
        0xC6 => {
            if pos >= bytes.len() { return None; }
            let modrm = bytes[pos];
            let reg_field = (modrm >> 3) & 7;
            if reg_field != 0 { return None; }
            pos += 1;
            let (_, base_insn_len) = decode_modrm_reg_and_skip(modrm, false, rex_b, has_67, bytes, pos)?;
            if base_insn_len >= bytes.len() { return None; }
            let imm = bytes[base_insn_len] as u64;
            Some(MmioDecoded {
                is_write: true,
                operand_size: 1,
                register: None,
                immediate: Some(imm),
                insn_len: base_insn_len + 1,
            })
        }
        
        // ── Two-byte opcodes (0x0F prefix) ──────────────────────
        0x0F => {
            if pos >= bytes.len() { return None; }
            let opcode2 = bytes[pos];
            pos += 1;
            
            match opcode2 {
                // MOVZX r32, r/m8 (0x0F 0xB6) — byte read, zero-extend
                0xB6 => {
                    if pos >= bytes.len() { return None; }
                    let modrm = bytes[pos];
                    pos += 1;
                    let (reg_idx, insn_len) = decode_modrm_reg_and_skip(modrm, rex_r, rex_b, has_67, bytes, pos)?;
                    Some(MmioDecoded {
                        is_write: false,
                        operand_size: 1, // read 1 byte, but dest register is wider
                        register: Some(reg_idx),
                        immediate: None,
                        insn_len,
                    })
                }
                // MOVZX r32, r/m16 (0x0F 0xB7) — word read, zero-extend
                0xB7 => {
                    if pos >= bytes.len() { return None; }
                    let modrm = bytes[pos];
                    pos += 1;
                    let (reg_idx, insn_len) = decode_modrm_reg_and_skip(modrm, rex_r, rex_b, has_67, bytes, pos)?;
                    Some(MmioDecoded {
                        is_write: false,
                        operand_size: 2,
                        register: Some(reg_idx),
                        immediate: None,
                        insn_len,
                    })
                }
                // MOVSX r32, r/m8 (0x0F 0xBE) — byte read, sign-extend
                0xBE => {
                    if pos >= bytes.len() { return None; }
                    let modrm = bytes[pos];
                    pos += 1;
                    let (reg_idx, insn_len) = decode_modrm_reg_and_skip(modrm, rex_r, rex_b, has_67, bytes, pos)?;
                    Some(MmioDecoded {
                        is_write: false,
                        operand_size: 1,
                        register: Some(reg_idx),
                        immediate: None,
                        insn_len,
                    })
                }
                // MOVSX r32, r/m16 (0x0F 0xBF) — word read, sign-extend
                0xBF => {
                    if pos >= bytes.len() { return None; }
                    let modrm = bytes[pos];
                    pos += 1;
                    let (reg_idx, insn_len) = decode_modrm_reg_and_skip(modrm, rex_r, rex_b, has_67, bytes, pos)?;
                    Some(MmioDecoded {
                        is_write: false,
                        operand_size: 2,
                        register: Some(reg_idx),
                        immediate: None,
                        insn_len,
                    })
                }
                _ => None, // Unknown 2-byte opcode
            }
        }
        
        // ── MOV EAX, moffs32 (opcode 0xA1) — READ from absolute addr ──
        0xA1 => {
            // This loads from an absolute address into RAX
            let addr_size = if has_67 { 4 } else { 8 }; // 8 in 64-bit mode
            let insn_len = pos + addr_size;
            Some(MmioDecoded {
                is_write: false,
                operand_size,
                register: Some(0), // RAX
                immediate: None,
                insn_len,
            })
        }
        
        // ── MOV moffs32, EAX (opcode 0xA3) — WRITE from RAX ──
        0xA3 => {
            let addr_size = if has_67 { 4 } else { 8 };
            let insn_len = pos + addr_size;
            Some(MmioDecoded {
                is_write: true,
                operand_size,
                register: Some(0), // RAX
                immediate: None,
                insn_len,
            })
        }
        
        // ── MOV AL, moffs8 (opcode 0xA0) — READ byte from absolute addr ──
        0xA0 => {
            let addr_size = if has_67 { 4 } else { 8 };
            let insn_len = pos + addr_size;
            Some(MmioDecoded {
                is_write: false,
                operand_size: 1,
                register: Some(0), // AL
                immediate: None,
                insn_len,
            })
        }
        
        // ── MOV moffs8, AL (opcode 0xA2) — WRITE byte from AL ──
        0xA2 => {
            let addr_size = if has_67 { 4 } else { 8 };
            let insn_len = pos + addr_size;
            Some(MmioDecoded {
                is_write: true,
                operand_size: 1,
                register: Some(0), // AL
                immediate: None,
                insn_len,
            })
        }
        
        _ => None, // Unknown opcode
    }
}

/// Decode ModR/M byte: extract the reg field (source/dest register)
/// and compute the total instruction length by skipping the r/m addressing.
///
/// Returns (register_index, total_insn_length_from_start_of_bytes).
fn decode_modrm_reg_and_skip(
    modrm: u8,
    rex_r: bool,
    _rex_b: bool,
    _has_67: bool,
    bytes: &[u8],
    pos_after_modrm: usize,
) -> Option<(u8, usize)> {
    let mod_field = (modrm >> 6) & 3;
    let reg_field = (modrm >> 3) & 7;
    let rm_field = modrm & 7;
    
    // REX.R extends the reg field
    let reg_idx = reg_field | (if rex_r { 8 } else { 0 });
    
    let mut pos = pos_after_modrm;
    
    // Skip the r/m addressing mode (SIB + displacement)
    match mod_field {
        0b00 => {
            // [r/m] — no displacement, unless rm=5 (RIP-relative) or rm=4 (SIB)
            if rm_field == 4 {
                // SIB byte follows
                pos += 1; // skip SIB
                if pos > bytes.len() { return None; }
                let sib = bytes[pos - 1];
                let base = sib & 7;
                if base == 5 {
                    pos += 4; // disp32
                }
            } else if rm_field == 5 {
                // RIP-relative: disp32
                pos += 4;
            }
        }
        0b01 => {
            // [r/m + disp8]
            if rm_field == 4 {
                pos += 1; // SIB byte
            }
            pos += 1; // disp8
        }
        0b10 => {
            // [r/m + disp32]
            if rm_field == 4 {
                pos += 1; // SIB byte
            }
            pos += 4; // disp32
        }
        0b11 => {
            // Register-to-register (not MMIO, but handle gracefully)
        }
        _ => unreachable!(),
    }
    
    if pos > bytes.len() {
        return None;
    }
    
    Some((reg_idx, pos))
}

/// Read a guest register value by index (0=RAX..15=R15)
pub fn read_guest_reg(regs: &super::svm_vm::SvmGuestRegs, idx: u8) -> u64 {
    match idx {
        0 => regs.rax,
        1 => regs.rcx,
        2 => regs.rdx,
        3 => regs.rbx,
        4 => regs.rsp,
        5 => regs.rbp,
        6 => regs.rsi,
        7 => regs.rdi,
        8 => regs.r8,
        9 => regs.r9,
        10 => regs.r10,
        11 => regs.r11,
        12 => regs.r12,
        13 => regs.r13,
        14 => regs.r14,
        15 => regs.r15,
        _ => 0,
    }
}

/// Write a value to a guest register by index (0=RAX..15=R15)
pub fn write_guest_reg(regs: &mut super::svm_vm::SvmGuestRegs, idx: u8, value: u64) {
    match idx {
        0 => regs.rax = value,
        1 => regs.rcx = value,
        2 => regs.rdx = value,
        3 => regs.rbx = value,
        4 => regs.rsp = value,
        5 => regs.rbp = value,
        6 => regs.rsi = value,
        7 => regs.rdi = value,
        8 => regs.r8 = value,
        9 => regs.r9 = value,
        10 => regs.r10 = value,
        11 => regs.r11 = value,
        12 => regs.r12 = value,
        13 => regs.r13 = value,
        14 => regs.r14 = value,
        15 => regs.r15 = value,
        _ => {}
    }
}

/// Mask a value to the operand size
pub fn mask_to_size(value: u64, size: u8) -> u64 {
    match size {
        1 => value & 0xFF,
        2 => value & 0xFFFF,
        4 => value & 0xFFFF_FFFF,
        8 => value,
        _ => value & 0xFFFF_FFFF,
    }
}

// ============================================================================
// UNIT TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_mov_write_rax() {
        // MOV [rdi], eax  →  89 07
        let bytes = [0x89, 0x07];
        let d = decode_mmio_instruction(&bytes, 2, true).unwrap();
        assert!(d.is_write);
        assert_eq!(d.operand_size, 4);
        assert_eq!(d.register, Some(0)); // EAX
        assert_eq!(d.insn_len, 2);
    }

    #[test]
    fn test_decode_mov_read_ecx() {
        // MOV ecx, [rdi]  →  8B 0F
        let bytes = [0x8B, 0x0F];
        let d = decode_mmio_instruction(&bytes, 2, true).unwrap();
        assert!(!d.is_write);
        assert_eq!(d.operand_size, 4);
        assert_eq!(d.register, Some(1)); // ECX
        assert_eq!(d.insn_len, 2);
    }

    #[test]
    fn test_decode_rex_w_mov_write() {
        // REX.W MOV [rdi], rax  →  48 89 07
        let bytes = [0x48, 0x89, 0x07];
        let d = decode_mmio_instruction(&bytes, 3, true).unwrap();
        assert!(d.is_write);
        assert_eq!(d.operand_size, 8);
        assert_eq!(d.register, Some(0)); // RAX
        assert_eq!(d.insn_len, 3);
    }

    #[test]
    fn test_decode_movzx_byte() {
        // MOVZX eax, BYTE PTR [rdi]  →  0F B6 07
        let bytes = [0x0F, 0xB6, 0x07];
        let d = decode_mmio_instruction(&bytes, 3, true).unwrap();
        assert!(!d.is_write);
        assert_eq!(d.operand_size, 1);
        assert_eq!(d.register, Some(0)); // EAX
    }

    #[test]  
    fn test_decode_disp32() {
        // MOV eax, [rdi + 0x320]  →  8B 87 20 03 00 00
        let bytes = [0x8B, 0x87, 0x20, 0x03, 0x00, 0x00];
        let d = decode_mmio_instruction(&bytes, 6, true).unwrap();
        assert!(!d.is_write);
        assert_eq!(d.operand_size, 4);
        assert_eq!(d.register, Some(0)); // EAX
        assert_eq!(d.insn_len, 6);
    }
}
