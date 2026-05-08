






use alloc::vec::Vec;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Reg {
    Rax = 0, Rcx = 1, Rdx = 2, Rbx = 3,
    Rsp = 4, Rbp = 5, Rsi = 6, Rdi = 7,
    R8 = 8,  R9 = 9,  R10 = 10, R11 = 11,
    R12 = 12, R13 = 13, R14 = 14, R15 = 15,
}

impl Reg {
    
    #[inline]
    pub fn lo3(self) -> u8 { (self as u8) & 0x07 }
    
    #[inline]
    pub fn needs_rex(self) -> bool { (self as u8) >= 8 }
}


#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum Cc {
    Hq  = 0x04, 
    Ne = 0x05, 
    Th  = 0x0C, 
    Ge = 0x0D, 
    Le = 0x0E, 
    G  = 0x0F, 
    B  = 0x02, 
    Ae = 0x03, 
    Be = 0x06, 
    A  = 0x07, 
}


#[derive(Debug, Clone, Copy)]
pub struct Br(pub usize);


#[derive(Debug)]
pub struct Pw {
    pub offset: usize,  
    pub label: Br,    
}


pub struct X86Asm {
    pub code: Vec<u8>,
    pub labels: Vec<Option<usize>>, 
    pub patches: Vec<Pw>,
}

impl X86Asm {
    pub fn new() -> Self {
        Self {
            code: Vec::with_capacity(4096),
            labels: Vec::new(),
            patches: Vec::new(),
        }
    }

    
    #[inline]
    pub fn offset(&self) -> usize { self.code.len() }

    
    pub fn new_label(&mut self) -> Br {
        let id = self.labels.len();
        self.labels.push(None);
        Br(id)
    }

    
    pub fn bind_label(&mut self, label: Br) {
        self.labels[label.0] = Some(self.code.len());
    }

    
    pub fn resolve_patches(&mut self) -> Result<(), &'static str> {
        for patch in &self.patches {
            let target = self.labels[patch.label.0].ok_or("unresolved label")?;
            let ot = target as i64 - (patch.offset as i64 + 4); 
            let oen = ot as i32;
            let bytes = oen.to_le_bytes();
            self.code[patch.offset..patch.offset + 4].copy_from_slice(&bytes);
        }
        Ok(())
    }

    

    
    fn rex_w(&mut self, rm: Reg, reg: Reg) {
        let mut rp: u8 = 0x48;
        if reg.needs_rex() { rp |= 0x04; } 
        if rm.needs_rex()  { rp |= 0x01; } 
        self.code.push(rp);
    }

    fn rex_w_single(&mut self, rm: Reg) {
        let mut rp: u8 = 0x48;
        if rm.needs_rex() { rp |= 0x01; }
        self.code.push(rp);
    }

    

    pub fn fi(mode: u8, reg: u8, rm: u8) -> u8 {
        (mode << 6) | ((reg & 7) << 3) | (rm & 7)
    }

    

    
    pub fn push_r(&mut self, r: Reg) {
        if r.needs_rex() { self.code.push(0x41); }
        self.code.push(0x50 + r.lo3());
    }

    
    pub fn pop_r(&mut self, r: Reg) {
        if r.needs_rex() { self.code.push(0x41); }
        self.code.push(0x58 + r.lo3());
    }

    
    pub fn mov_r_imm64(&mut self, dst: Reg, imm: i64) {
        self.rex_w_single(dst);
        self.code.push(0xB8 + dst.lo3());
        self.code.extend_from_slice(&imm.to_le_bytes());
    }

    
    pub fn mov_r_imm32(&mut self, dst: Reg, imm: i32) {
        self.rex_w_single(dst);
        self.code.push(0xC7);
        self.code.push(Self::fi(0b11, 0, dst.lo3()));
        self.code.extend_from_slice(&imm.to_le_bytes());
    }

    
    pub fn mov_r_r(&mut self, dst: Reg, src: Reg) {
        self.rex_w(dst, src);
        self.code.push(0x89);
        self.code.push(Self::fi(0b11, src.lo3(), dst.lo3()));
    }

    
    pub fn mov_r_rbp_offset(&mut self, dst: Reg, offset: i32) {
        self.rex_w(Reg::Rbp, dst);
        self.code.push(0x8B);
        if offset >= -128 && offset <= 127 {
            self.code.push(Self::fi(0b01, dst.lo3(), 0x05));
            self.code.push(offset as u8);
        } else {
            self.code.push(Self::fi(0b10, dst.lo3(), 0x05));
            self.code.extend_from_slice(&offset.to_le_bytes());
        }
    }

    
    pub fn mov_rbp_offset_r(&mut self, offset: i32, src: Reg) {
        self.rex_w(Reg::Rbp, src);
        self.code.push(0x89);
        if offset >= -128 && offset <= 127 {
            self.code.push(Self::fi(0b01, src.lo3(), 0x05));
            self.code.push(offset as u8);
        } else {
            self.code.push(Self::fi(0b10, src.lo3(), 0x05));
            self.code.extend_from_slice(&offset.to_le_bytes());
        }
    }

    
    pub fn add_r_r(&mut self, dst: Reg, src: Reg) {
        self.rex_w(dst, src);
        self.code.push(0x01);
        self.code.push(Self::fi(0b11, src.lo3(), dst.lo3()));
    }

    
    pub fn sub_r_r(&mut self, dst: Reg, src: Reg) {
        self.rex_w(dst, src);
        self.code.push(0x29);
        self.code.push(Self::fi(0b11, src.lo3(), dst.lo3()));
    }

    
    pub fn imul_r_r(&mut self, dst: Reg, src: Reg) {
        self.rex_w(src, dst);
        self.code.push(0x0F);
        self.code.push(0xAF);
        self.code.push(Self::fi(0b11, dst.lo3(), src.lo3()));
    }

    
    pub fn cqo(&mut self) {
        self.code.push(0x48);
        self.code.push(0x99);
    }

    
    pub fn idiv_r(&mut self, src: Reg) {
        self.rex_w_single(src);
        self.code.push(0xF7);
        self.code.push(Self::fi(0b11, 7, src.lo3()));
    }

    
    pub fn neg_r(&mut self, r: Reg) {
        self.rex_w_single(r);
        self.code.push(0xF7);
        self.code.push(Self::fi(0b11, 3, r.lo3()));
    }

    
    pub fn and_r_r(&mut self, dst: Reg, src: Reg) {
        self.rex_w(dst, src);
        self.code.push(0x21);
        self.code.push(Self::fi(0b11, src.lo3(), dst.lo3()));
    }

    
    pub fn or_r_r(&mut self, dst: Reg, src: Reg) {
        self.rex_w(dst, src);
        self.code.push(0x09);
        self.code.push(Self::fi(0b11, src.lo3(), dst.lo3()));
    }

    
    pub fn xor_r_r(&mut self, dst: Reg, src: Reg) {
        self.rex_w(dst, src);
        self.code.push(0x31);
        self.code.push(Self::fi(0b11, src.lo3(), dst.lo3()));
    }

    
    pub fn shl_r_cl(&mut self, dst: Reg) {
        self.rex_w_single(dst);
        self.code.push(0xD3);
        self.code.push(Self::fi(0b11, 4, dst.lo3()));
    }

    
    pub fn sar_r_cl(&mut self, dst: Reg) {
        self.rex_w_single(dst);
        self.code.push(0xD3);
        self.code.push(Self::fi(0b11, 7, dst.lo3()));
    }

    
    pub fn cmp_r_r(&mut self, a: Reg, b: Reg) {
        self.rex_w(a, b);
        self.code.push(0x39);
        self.code.push(Self::fi(0b11, b.lo3(), a.lo3()));
    }

    
    pub fn qau(&mut self, r: Reg, imm: i32) {
        self.rex_w_single(r);
        if r == Reg::Rax {
            self.code.push(0x3D);
        } else {
            self.code.push(0x81);
            self.code.push(Self::fi(0b11, 7, r.lo3()));
        }
        self.code.extend_from_slice(&imm.to_le_bytes());
    }

    
    pub fn test_r_r(&mut self, a: Reg, b: Reg) {
        self.rex_w(a, b);
        self.code.push(0x85);
        self.code.push(Self::fi(0b11, b.lo3(), a.lo3()));
    }

    
    pub fn setcc(&mut self, ft: Cc, dst: Reg) {
        if dst.needs_rex() {
            self.code.push(0x41);
        } else {
            
            self.code.push(0x40);
        }
        self.code.push(0x0F);
        self.code.push(0x90 + ft as u8);
        self.code.push(Self::fi(0b11, 0, dst.lo3()));
    }

    
    pub fn movzx_r_r8(&mut self, dst: Reg, src: Reg) {
        self.rex_w(src, dst);
        self.code.push(0x0F);
        self.code.push(0xB6);
        self.code.push(Self::fi(0b11, dst.lo3(), src.lo3()));
    }

    
    pub fn add_r_imm32(&mut self, dst: Reg, imm: i32) {
        self.rex_w_single(dst);
        if dst == Reg::Rax {
            self.code.push(0x05);
        } else {
            self.code.push(0x81);
            self.code.push(Self::fi(0b11, 0, dst.lo3()));
        }
        self.code.extend_from_slice(&imm.to_le_bytes());
    }

    
    pub fn sub_r_imm32(&mut self, dst: Reg, imm: i32) {
        self.rex_w_single(dst);
        if dst == Reg::Rax {
            self.code.push(0x2D);
        } else {
            self.code.push(0x81);
            self.code.push(Self::fi(0b11, 5, dst.lo3()));
        }
        self.code.extend_from_slice(&imm.to_le_bytes());
    }

    
    pub fn jmp_label(&mut self, label: Br) {
        self.code.push(0xE9);
        let off = self.code.len();
        self.code.extend_from_slice(&0i32.to_le_bytes()); 
        self.patches.push(Pw { offset: off, label });
    }

    
    pub fn jcc_label(&mut self, ft: Cc, label: Br) {
        self.code.push(0x0F);
        self.code.push(0x80 + ft as u8);
        let off = self.code.len();
        self.code.extend_from_slice(&0i32.to_le_bytes()); 
        self.patches.push(Pw { offset: off, label });
    }

    
    pub fn call_label(&mut self, label: Br) {
        self.code.push(0xE8);
        let off = self.code.len();
        self.code.extend_from_slice(&0i32.to_le_bytes());
        self.patches.push(Pw { offset: off, label });
    }

    
    pub fn call_r(&mut self, r: Reg) {
        if r.needs_rex() { self.code.push(0x41); }
        self.code.push(0xFF);
        self.code.push(Self::fi(0b11, 2, r.lo3()));
    }

    
    pub fn ret(&mut self) {
        self.code.push(0xC3);
    }

    
    pub fn iqu(&mut self) {
        self.code.push(0x90);
    }

    
    pub fn qnf(&mut self, dst: Reg, offset: i32) {
        self.rex_w(Reg::Rbp, dst);
        self.code.push(0x8D);
        if offset >= -128 && offset <= 127 {
            self.code.push(Self::fi(0b01, dst.lo3(), 0x05));
            self.code.push(offset as u8);
        } else {
            self.code.push(Self::fi(0b10, dst.lo3(), 0x05));
            self.code.extend_from_slice(&offset.to_le_bytes());
        }
    }

    

    
    pub fn prologue(&mut self, frame_size: i32) {
        self.push_r(Reg::Rbp);
        self.mov_r_r(Reg::Rbp, Reg::Rsp);
        if frame_size > 0 {
            self.sub_r_imm32(Reg::Rsp, frame_size);
        }
    }

    
    pub fn epilogue(&mut self) {
        self.mov_r_r(Reg::Rsp, Reg::Rbp);
        self.pop_r(Reg::Rbp);
        self.ret();
    }
}
