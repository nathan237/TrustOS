//! x86_64 Machine Code Assembler for TrustLang Native Backend
//!
//! Emits raw x86_64 instructions into a byte buffer.
//! Uses System V AMD64 calling convention (rdi, rsi, rdx, rcx, r8, r9).
//! Integer work registers: rax, rbx, rcx, rdx, rsi, rdi.
//! Stack-based evaluation: push/pop for expression results.

use alloc::vec::Vec;

/// x86_64 register encoding (REX.B-aware)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Reg {
    Rax = 0, Rcx = 1, Rdx = 2, Rbx = 3,
    Rsp = 4, Rbp = 5, Rsi = 6, Rdi = 7,
    R8 = 8,  R9 = 9,  R10 = 10, R11 = 11,
    R12 = 12, R13 = 13, R14 = 14, R15 = 15,
}

impl Reg {
    /// Low 3 bits for ModR/M encoding
    #[inline]
    pub fn lo3(self) -> u8 { (self as u8) & 0x07 }
    /// Whether this register needs REX.B or REX.R
    #[inline]
    pub fn needs_rex(self) -> bool { (self as u8) >= 8 }
}

/// Condition codes for Jcc / SETcc
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum Cc {
    E  = 0x04, // equal (ZF=1)
    Ne = 0x05, // not equal
    L  = 0x0C, // less (signed)
    Ge = 0x0D, // greater or equal (signed)
    Le = 0x0E, // less or equal (signed)
    G  = 0x0F, // greater (signed)
    B  = 0x02, // below (unsigned)
    Ae = 0x03, // above or equal (unsigned)
    Be = 0x06, // below or equal (unsigned)
    A  = 0x07, // above (unsigned)
}

/// A relocatable label (forward reference)
#[derive(Debug, Clone, Copy)]
pub struct Label(pub usize);

/// Patch entry for forward jumps
#[derive(Debug)]
pub struct Patch {
    pub offset: usize,  // where the rel32 placeholder is
    pub label: Label,    // target label
}

/// x86_64 machine code emitter
pub struct X86Asm {
    pub code: Vec<u8>,
    pub labels: Vec<Option<usize>>, // label_id → offset
    pub patches: Vec<Patch>,
}

impl X86Asm {
    pub fn new() -> Self {
        Self {
            code: Vec::with_capacity(4096),
            labels: Vec::new(),
            patches: Vec::new(),
        }
    }

    /// Current offset in the code buffer
    #[inline]
    pub fn offset(&self) -> usize { self.code.len() }

    /// Create a new unresolved label
    pub fn new_label(&mut self) -> Label {
        let id = self.labels.len();
        self.labels.push(None);
        Label(id)
    }

    /// Bind a label to the current offset
    pub fn bind_label(&mut self, label: Label) {
        self.labels[label.0] = Some(self.code.len());
    }

    /// Apply all forward-reference patches. Returns Err on unresolved labels.
    pub fn resolve_patches(&mut self) -> Result<(), &'static str> {
        for patch in &self.patches {
            let target = self.labels[patch.label.0].ok_or("unresolved label")?;
            let rel = target as i64 - (patch.offset as i64 + 4); // rel32 is relative to end of instruction
            let rel32 = rel as i32;
            let bytes = rel32.to_le_bytes();
            self.code[patch.offset..patch.offset + 4].copy_from_slice(&bytes);
        }
        Ok(())
    }

    // ─── REX prefix helpers ─────────────────────────────────────────

    /// REX.W prefix (64-bit operand size)
    fn rex_w(&mut self, rm: Reg, reg: Reg) {
        let mut rex: u8 = 0x48;
        if reg.needs_rex() { rex |= 0x04; } // REX.R
        if rm.needs_rex()  { rex |= 0x01; } // REX.B
        self.code.push(rex);
    }

    fn rex_w_single(&mut self, rm: Reg) {
        let mut rex: u8 = 0x48;
        if rm.needs_rex() { rex |= 0x01; }
        self.code.push(rex);
    }

    // ─── Encoding helpers ───────────────────────────────────────────

    pub fn modrm(mode: u8, reg: u8, rm: u8) -> u8 {
        (mode << 6) | ((reg & 7) << 3) | (rm & 7)
    }

    // ─── Instructions ───────────────────────────────────────────────

    /// push reg
    pub fn push_r(&mut self, r: Reg) {
        if r.needs_rex() { self.code.push(0x41); }
        self.code.push(0x50 + r.lo3());
    }

    /// pop reg
    pub fn pop_r(&mut self, r: Reg) {
        if r.needs_rex() { self.code.push(0x41); }
        self.code.push(0x58 + r.lo3());
    }

    /// mov reg, imm64
    pub fn mov_r_imm64(&mut self, dst: Reg, imm: i64) {
        self.rex_w_single(dst);
        self.code.push(0xB8 + dst.lo3());
        self.code.extend_from_slice(&imm.to_le_bytes());
    }

    /// mov reg, imm32 (sign-extended to 64-bit)
    pub fn mov_r_imm32(&mut self, dst: Reg, imm: i32) {
        self.rex_w_single(dst);
        self.code.push(0xC7);
        self.code.push(Self::modrm(0b11, 0, dst.lo3()));
        self.code.extend_from_slice(&imm.to_le_bytes());
    }

    /// mov dst, src (64-bit register to register)
    pub fn mov_r_r(&mut self, dst: Reg, src: Reg) {
        self.rex_w(dst, src);
        self.code.push(0x89);
        self.code.push(Self::modrm(0b11, src.lo3(), dst.lo3()));
    }

    /// mov dst, [rbp + offset] (load local from stack frame)
    pub fn mov_r_rbp_offset(&mut self, dst: Reg, offset: i32) {
        self.rex_w(Reg::Rbp, dst);
        self.code.push(0x8B);
        if offset >= -128 && offset <= 127 {
            self.code.push(Self::modrm(0b01, dst.lo3(), 0x05));
            self.code.push(offset as u8);
        } else {
            self.code.push(Self::modrm(0b10, dst.lo3(), 0x05));
            self.code.extend_from_slice(&offset.to_le_bytes());
        }
    }

    /// mov [rbp + offset], src (store to local in stack frame)
    pub fn mov_rbp_offset_r(&mut self, offset: i32, src: Reg) {
        self.rex_w(Reg::Rbp, src);
        self.code.push(0x89);
        if offset >= -128 && offset <= 127 {
            self.code.push(Self::modrm(0b01, src.lo3(), 0x05));
            self.code.push(offset as u8);
        } else {
            self.code.push(Self::modrm(0b10, src.lo3(), 0x05));
            self.code.extend_from_slice(&offset.to_le_bytes());
        }
    }

    /// add dst, src
    pub fn add_r_r(&mut self, dst: Reg, src: Reg) {
        self.rex_w(dst, src);
        self.code.push(0x01);
        self.code.push(Self::modrm(0b11, src.lo3(), dst.lo3()));
    }

    /// sub dst, src
    pub fn sub_r_r(&mut self, dst: Reg, src: Reg) {
        self.rex_w(dst, src);
        self.code.push(0x29);
        self.code.push(Self::modrm(0b11, src.lo3(), dst.lo3()));
    }

    /// imul dst, src (signed multiply, result in dst)
    pub fn imul_r_r(&mut self, dst: Reg, src: Reg) {
        self.rex_w(src, dst);
        self.code.push(0x0F);
        self.code.push(0xAF);
        self.code.push(Self::modrm(0b11, dst.lo3(), src.lo3()));
    }

    /// cqo (sign-extend rax into rdx:rax for division)
    pub fn cqo(&mut self) {
        self.code.push(0x48);
        self.code.push(0x99);
    }

    /// idiv src (signed divide rdx:rax by src, quotient in rax, remainder in rdx)
    pub fn idiv_r(&mut self, src: Reg) {
        self.rex_w_single(src);
        self.code.push(0xF7);
        self.code.push(Self::modrm(0b11, 7, src.lo3()));
    }

    /// neg reg (two's complement negate)
    pub fn neg_r(&mut self, r: Reg) {
        self.rex_w_single(r);
        self.code.push(0xF7);
        self.code.push(Self::modrm(0b11, 3, r.lo3()));
    }

    /// and dst, src
    pub fn and_r_r(&mut self, dst: Reg, src: Reg) {
        self.rex_w(dst, src);
        self.code.push(0x21);
        self.code.push(Self::modrm(0b11, src.lo3(), dst.lo3()));
    }

    /// or dst, src
    pub fn or_r_r(&mut self, dst: Reg, src: Reg) {
        self.rex_w(dst, src);
        self.code.push(0x09);
        self.code.push(Self::modrm(0b11, src.lo3(), dst.lo3()));
    }

    /// xor dst, src
    pub fn xor_r_r(&mut self, dst: Reg, src: Reg) {
        self.rex_w(dst, src);
        self.code.push(0x31);
        self.code.push(Self::modrm(0b11, src.lo3(), dst.lo3()));
    }

    /// shl dst, cl (shift left by cl)
    pub fn shl_r_cl(&mut self, dst: Reg) {
        self.rex_w_single(dst);
        self.code.push(0xD3);
        self.code.push(Self::modrm(0b11, 4, dst.lo3()));
    }

    /// shr dst, cl (shift right by cl, arithmetic)
    pub fn sar_r_cl(&mut self, dst: Reg) {
        self.rex_w_single(dst);
        self.code.push(0xD3);
        self.code.push(Self::modrm(0b11, 7, dst.lo3()));
    }

    /// cmp a, b
    pub fn cmp_r_r(&mut self, a: Reg, b: Reg) {
        self.rex_w(a, b);
        self.code.push(0x39);
        self.code.push(Self::modrm(0b11, b.lo3(), a.lo3()));
    }

    /// cmp reg, imm32
    pub fn cmp_r_imm32(&mut self, r: Reg, imm: i32) {
        self.rex_w_single(r);
        if r == Reg::Rax {
            self.code.push(0x3D);
        } else {
            self.code.push(0x81);
            self.code.push(Self::modrm(0b11, 7, r.lo3()));
        }
        self.code.extend_from_slice(&imm.to_le_bytes());
    }

    /// test reg, reg (AND without storing — sets flags)
    pub fn test_r_r(&mut self, a: Reg, b: Reg) {
        self.rex_w(a, b);
        self.code.push(0x85);
        self.code.push(Self::modrm(0b11, b.lo3(), a.lo3()));
    }

    /// setcc r8 (set byte based on condition)
    pub fn setcc(&mut self, cc: Cc, dst: Reg) {
        if dst.needs_rex() {
            self.code.push(0x41);
        } else {
            // Need REX prefix to access sil/dil etc.
            self.code.push(0x40);
        }
        self.code.push(0x0F);
        self.code.push(0x90 + cc as u8);
        self.code.push(Self::modrm(0b11, 0, dst.lo3()));
    }

    /// movzx dst(64), src(8) — zero-extend byte to qword
    pub fn movzx_r_r8(&mut self, dst: Reg, src: Reg) {
        self.rex_w(src, dst);
        self.code.push(0x0F);
        self.code.push(0xB6);
        self.code.push(Self::modrm(0b11, dst.lo3(), src.lo3()));
    }

    /// add dst, imm32
    pub fn add_r_imm32(&mut self, dst: Reg, imm: i32) {
        self.rex_w_single(dst);
        if dst == Reg::Rax {
            self.code.push(0x05);
        } else {
            self.code.push(0x81);
            self.code.push(Self::modrm(0b11, 0, dst.lo3()));
        }
        self.code.extend_from_slice(&imm.to_le_bytes());
    }

    /// sub dst, imm32
    pub fn sub_r_imm32(&mut self, dst: Reg, imm: i32) {
        self.rex_w_single(dst);
        if dst == Reg::Rax {
            self.code.push(0x2D);
        } else {
            self.code.push(0x81);
            self.code.push(Self::modrm(0b11, 5, dst.lo3()));
        }
        self.code.extend_from_slice(&imm.to_le_bytes());
    }

    /// jmp rel32 (to a label, patched later)
    pub fn jmp_label(&mut self, label: Label) {
        self.code.push(0xE9);
        let off = self.code.len();
        self.code.extend_from_slice(&0i32.to_le_bytes()); // placeholder
        self.patches.push(Patch { offset: off, label });
    }

    /// jcc rel32 (conditional jump to label)
    pub fn jcc_label(&mut self, cc: Cc, label: Label) {
        self.code.push(0x0F);
        self.code.push(0x80 + cc as u8);
        let off = self.code.len();
        self.code.extend_from_slice(&0i32.to_le_bytes()); // placeholder
        self.patches.push(Patch { offset: off, label });
    }

    /// call rel32 (to a label, patched later)
    pub fn call_label(&mut self, label: Label) {
        self.code.push(0xE8);
        let off = self.code.len();
        self.code.extend_from_slice(&0i32.to_le_bytes());
        self.patches.push(Patch { offset: off, label });
    }

    /// call register (indirect)
    pub fn call_r(&mut self, r: Reg) {
        if r.needs_rex() { self.code.push(0x41); }
        self.code.push(0xFF);
        self.code.push(Self::modrm(0b11, 2, r.lo3()));
    }

    /// ret
    pub fn ret(&mut self) {
        self.code.push(0xC3);
    }

    /// nop
    pub fn nop(&mut self) {
        self.code.push(0x90);
    }

    /// lea dst, [rbp + offset]
    pub fn lea_rbp_offset(&mut self, dst: Reg, offset: i32) {
        self.rex_w(Reg::Rbp, dst);
        self.code.push(0x8D);
        if offset >= -128 && offset <= 127 {
            self.code.push(Self::modrm(0b01, dst.lo3(), 0x05));
            self.code.push(offset as u8);
        } else {
            self.code.push(Self::modrm(0b10, dst.lo3(), 0x05));
            self.code.extend_from_slice(&offset.to_le_bytes());
        }
    }

    // ─── Function prologue / epilogue ───────────────────────────────

    /// Standard function prologue: push rbp; mov rbp, rsp; sub rsp, frame_size
    pub fn prologue(&mut self, frame_size: i32) {
        self.push_r(Reg::Rbp);
        self.mov_r_r(Reg::Rbp, Reg::Rsp);
        if frame_size > 0 {
            self.sub_r_imm32(Reg::Rsp, frame_size);
        }
    }

    /// Standard epilogue: mov rsp, rbp; pop rbp; ret
    pub fn epilogue(&mut self) {
        self.mov_r_r(Reg::Rsp, Reg::Rbp);
        self.pop_r(Reg::Rbp);
        self.ret();
    }
}
