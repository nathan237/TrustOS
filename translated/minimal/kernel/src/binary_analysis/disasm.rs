






use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;




#[derive(Debug, Clone)]
pub struct Bj {
    
    pub address: u64,
    
    pub bytes: Vec<u8>,
    
    pub mnemonic: String,
    
    pub operands_str: String,
    
    pub comment: Option<String>,
    
    pub branch_target: Option<u64>,
    
    pub is_call: bool,
    
    pub is_ret: bool,
    
    pub is_jump: bool,
    
    pub is_cond_jump: bool,
}



const Anw: [&str; 16] = ["rax","rcx","rdx","rbx","rsp","rbp","rsi","rdi",
                            "r8","r9","r10","r11","r12","r13","r14","r15"];
const Anv: [&str; 16] = ["eax","ecx","edx","ebx","esp","ebp","esi","edi",
                            "r8d","r9d","r10d","r11d","r12d","r13d","r14d","r15d"];
const Anu: [&str; 16] = ["ax","cx","dx","bx","sp","bp","si","di",
                            "r8w","r9w","r10w","r11w","r12w","r13w","r14w","r15w"];
const Anx:  [&str; 16] = ["al","cl","dl","bl","spl","bpl","sil","dil",
                            "r8b","r9b","r10b","r11b","r12b","r13b","r14b","r15b"];
const CQX_: [&str; 8] = ["al","cl","dl","bl","ah","ch","dh","bh"];

fn xi(idx: u8, size: u8, dv: bool) -> &'static str {
    let i = (idx & 0x0F) as usize;
    match size {
        8 => Anw.get(i).copied().unwrap_or("?"),
        4 => Anv.get(i).copied().unwrap_or("?"),
        2 => Anu.get(i).copied().unwrap_or("?"),
        1 => {
            if dv {
                Anx.get(i).copied().unwrap_or("?")
            } else {
                CQX_.get(i).copied().unwrap_or("?")
            }
        }
        _ => "?",
    }
}

fn avd(size: u8) -> &'static str {
    match size {
        8 => "qword",
        4 => "dword",
        2 => "word",
        1 => "byte",
        _ => "",
    }
}



const TA_: [&str; 16] = [
    "o", "no", "b", "nb", "z", "nz", "be", "a",
    "s", "ns", "p", "np", "l", "nl", "le", "g",
];



pub struct Disassembler<'a> {
    code: &'a [u8],
    base_addr: u64,
    pos: usize,
}

impl<'a> Disassembler<'a> {
    pub fn new(code: &'a [u8], base_addr: u64) -> Self {
        Self { code, base_addr, pos: 0 }
    }

    
    pub fn disassemble(&mut self, jm: usize) -> Vec<Bj> {
        let mut out = Vec::new();
        while self.pos < self.code.len() && out.len() < jm {
            let inst = self.decode_one();
            out.push(inst);
        }
        out
    }

    
    pub fn disassemble_all(&mut self) -> Vec<Bj> {
        self.disassemble(8192)
    }

    fn decode_one(&mut self) -> Bj {
        let start = self.pos;
        let addr = self.base_addr + start as u64;

        
        let mut ckh = false;    
        let mut alr = false;    
        let mut eot = false;    
        let mut czb = false;    
        let mut ond = false;

        loop {
            if self.pos >= self.code.len() { break; }
            match self.code[self.pos] {
                0x66 => { ckh = true; self.pos += 1; }
                0x67 => { alr = true; self.pos += 1; }
                0xF2 => { eot = true; self.pos += 1; }
                0xF3 => { czb = true; self.pos += 1; }
                0x26 | 0x2E | 0x36 | 0x3E | 0x64 | 0x65 => { ond = true; self.pos += 1; }
                _ => break,
            }
            
            if self.pos - start > 4 { break; }
        }

        if self.pos >= self.code.len() {
            return self.make_db(start, addr);
        }

        
        let mut rp: u8 = 0;
        let dv;
        let b = self.code[self.pos];
        if b >= 0x40 && b <= 0x4F {
            rp = b;
            self.pos += 1;
            dv = true;
        } else {
            dv = false;
        }

        let rex_w = (rp & 0x08) != 0;
        let gb = (rp & 0x04) != 0;
        let gp = (rp & 0x02) != 0;
        let cq = (rp & 0x01) != 0;

        
        let kz: u8 = if rex_w { 8 } else if ckh { 2 } else { 4 };

        if self.pos >= self.code.len() {
            return self.make_db(start, addr);
        }

        let opcode = self.code[self.pos];
        self.pos += 1;

        
        if opcode == 0x0F {
            return self.decode_0f(start, addr, rp, kz, dv, eot, czb);
        }

        
        let result = match opcode {
            
            0x90 => Some(("nop", String::new(), None, false, false, false, false)),

            
            0xC3 => Some(("ret", String::new(), None, false, true, false, false)),

            
            0xC2 => {
                let imm = self.read_u16().unwrap_or(0);
                Some(("ret", format!("{:#x}", imm), None, false, true, false, false))
            }

            
            0xCC => Some(("int3", String::new(), None, false, false, false, false)),

            
            0xCD => {
                let imm = self.read_u8().unwrap_or(0);
                Some(("int", format!("{:#x}", imm), None, false, false, false, false))
            }

            
            0xF4 => Some(("hlt", String::new(), None, false, false, false, false)),

            
            0xF8 => Some(("clc", String::new(), None, false, false, false, false)),
            0xF9 => Some(("stc", String::new(), None, false, false, false, false)),
            0xFA => Some(("cli", String::new(), None, false, false, false, false)),
            0xFB => Some(("sti", String::new(), None, false, false, false, false)),
            0xFC => Some(("cld", String::new(), None, false, false, false, false)),
            0xFD => Some(("std", String::new(), None, false, false, false, false)),

            
            0xC9 => Some(("leave", String::new(), None, false, false, false, false)),

            
            0x99 => {
                let akc = if rex_w { "cqo" } else { "cdq" };
                Some((akc, String::new(), None, false, false, false, false))
            }

            
            0x98 => {
                let akc = if rex_w { "cdqe" } else if ckh { "cbw" } else { "cwde" };
                Some((akc, String::new(), None, false, false, false, false))
            }

            
            0x50..=0x57 => {
                let r = (opcode - 0x50) | if cq { 8 } else { 0 };
                Some(("push", String::from(xi(r, 8, dv)), None, false, false, false, false))
            }

            
            0x58..=0x5F => {
                let r = (opcode - 0x58) | if cq { 8 } else { 0 };
                Some(("pop", String::from(xi(r, 8, dv)), None, false, false, false, false))
            }

            
            0x6A => {
                let imm = self.read_i8().unwrap_or(0) as i64;
                Some(("push", aqn(imm), None, false, false, false, false))
            }

            
            0x68 => {
                let imm = self.read_i32().unwrap_or(0) as i64;
                Some(("push", aqn(imm), None, false, false, false, false))
            }

            
            0xB0..=0xB7 => {
                
                let r = (opcode - 0xB0) | if cq { 8 } else { 0 };
                let imm = self.read_u8().unwrap_or(0);
                Some(("mov", format!("{}, {:#x}", xi(r, 1, dv), imm), None, false, false, false, false))
            }
            0xB8..=0xBF => {
                let r = (opcode - 0xB8) | if cq { 8 } else { 0 };
                let imm = if rex_w {
                    self.read_i64().unwrap_or(0)
                } else {
                    self.read_i32().unwrap_or(0) as i64
                };
                Some(("mov", format!("{}, {}", xi(r, kz, dv), aqn(imm)), None, false, false, false, false))
            }

            
            0x91..=0x97 => {
                let r = (opcode - 0x90) | if cq { 8 } else { 0 };
                Some(("xchg", format!("{}, {}", xi(0, kz, dv), xi(r, kz, dv)), None, false, false, false, false))
            }

            
            0xE8 => {
                let ot = self.read_i32().unwrap_or(0) as i64;
                let target = (addr as i64 + (self.pos - start) as i64 + ot) as u64;
                Some(("call", format!("{:#x}", target), Some(target), true, false, false, false))
            }

            
            0xE9 => {
                let ot = self.read_i32().unwrap_or(0) as i64;
                let target = (addr as i64 + (self.pos - start) as i64 + ot) as u64;
                Some(("jmp", format!("{:#x}", target), Some(target), false, false, true, false))
            }

            
            0xEB => {
                let ot = self.read_i8().unwrap_or(0) as i64;
                let target = (addr as i64 + (self.pos - start) as i64 + ot) as u64;
                Some(("jmp", format!("{:#x}", target), Some(target), false, false, true, false))
            }

            
            0x70..=0x7F => {
                let ft = opcode - 0x70;
                let ot = self.read_i8().unwrap_or(0) as i64;
                let target = (addr as i64 + (self.pos - start) as i64 + ot) as u64;
                let akc = format!("j{}", TA_[ft as usize]);
                Some((&"jcc_placeholder", format!("{:#x}", target), Some(target), false, false, false, true))
                    .map(|(_, ops, tgt, alb, ret, jmp, cj)| (esm(&akc), ops, tgt, alb, ret, jmp, cj))
            }

            
            0xE0 => {
                let ot = self.read_i8().unwrap_or(0) as i64;
                let target = (addr as i64 + (self.pos - start) as i64 + ot) as u64;
                Some(("loopne", format!("{:#x}", target), Some(target), false, false, false, true))
            }
            0xE1 => {
                let ot = self.read_i8().unwrap_or(0) as i64;
                let target = (addr as i64 + (self.pos - start) as i64 + ot) as u64;
                Some(("loope", format!("{:#x}", target), Some(target), false, false, false, true))
            }
            0xE2 => {
                let ot = self.read_i8().unwrap_or(0) as i64;
                let target = (addr as i64 + (self.pos - start) as i64 + ot) as u64;
                Some(("loop", format!("{:#x}", target), Some(target), false, false, false, true))
            }

            
            
            0x00 | 0x01 | 0x02 | 0x03 => self.decode_alu_rm(start, addr, opcode, "add", kz, dv, gb, cq, gp),
            0x08 | 0x09 | 0x0A | 0x0B => self.decode_alu_rm(start, addr, opcode, "or",  kz, dv, gb, cq, gp),
            0x10 | 0x11 | 0x12 | 0x13 => self.decode_alu_rm(start, addr, opcode, "adc", kz, dv, gb, cq, gp),
            0x18 | 0x19 | 0x1A | 0x1B => self.decode_alu_rm(start, addr, opcode, "sbb", kz, dv, gb, cq, gp),
            0x20 | 0x21 | 0x22 | 0x23 => self.decode_alu_rm(start, addr, opcode, "and", kz, dv, gb, cq, gp),
            0x28 | 0x29 | 0x2A | 0x2B => self.decode_alu_rm(start, addr, opcode, "sub", kz, dv, gb, cq, gp),
            0x30 | 0x31 | 0x32 | 0x33 => self.decode_alu_rm(start, addr, opcode, "xor", kz, dv, gb, cq, gp),
            0x38 | 0x39 | 0x3A | 0x3B => self.decode_alu_rm(start, addr, opcode, "cmp", kz, dv, gb, cq, gp),

            
            0x04 => { let imm = self.read_u8().unwrap_or(0); Some(("add", format!("al, {:#x}", imm), None, false, false, false, false)) }
            0x05 => { let imm = self.read_i32().unwrap_or(0); Some(("add", format!("{}, {}", xi(0, kz, dv), aqn(imm as i64)), None, false, false, false, false)) }
            0x0C => { let imm = self.read_u8().unwrap_or(0); Some(("or",  format!("al, {:#x}", imm), None, false, false, false, false)) }
            0x0D => { let imm = self.read_i32().unwrap_or(0); Some(("or",  format!("{}, {}", xi(0, kz, dv), aqn(imm as i64)), None, false, false, false, false)) }
            0x24 => { let imm = self.read_u8().unwrap_or(0); Some(("and", format!("al, {:#x}", imm), None, false, false, false, false)) }
            0x25 => { let imm = self.read_i32().unwrap_or(0); Some(("and", format!("{}, {}", xi(0, kz, dv), aqn(imm as i64)), None, false, false, false, false)) }
            0x2C => { let imm = self.read_u8().unwrap_or(0); Some(("sub", format!("al, {:#x}", imm), None, false, false, false, false)) }
            0x2D => { let imm = self.read_i32().unwrap_or(0); Some(("sub", format!("{}, {}", xi(0, kz, dv), aqn(imm as i64)), None, false, false, false, false)) }
            0x34 => { let imm = self.read_u8().unwrap_or(0); Some(("xor", format!("al, {:#x}", imm), None, false, false, false, false)) }
            0x35 => { let imm = self.read_i32().unwrap_or(0); Some(("xor", format!("{}, {}", xi(0, kz, dv), aqn(imm as i64)), None, false, false, false, false)) }
            0x3C => { let imm = self.read_u8().unwrap_or(0); Some(("cmp", format!("al, {:#x}", imm), None, false, false, false, false)) }
            0x3D => { let imm = self.read_i32().unwrap_or(0); Some(("cmp", format!("{}, {}", xi(0, kz, dv), aqn(imm as i64)), None, false, false, false, false)) }

            
            0x84 => {
                let (rm, reg) = self.decode_modrm_operands(1, dv, gb, cq, gp);
                Some(("test", format!("{}, {}", rm, reg), None, false, false, false, false))
            }
            0x85 => {
                let (rm, reg) = self.decode_modrm_operands(kz, dv, gb, cq, gp);
                Some(("test", format!("{}, {}", rm, reg), None, false, false, false, false))
            }

            
            0xA8 => { let imm = self.read_u8().unwrap_or(0); Some(("test", format!("al, {:#x}", imm), None, false, false, false, false)) }
            0xA9 => { let imm = self.read_i32().unwrap_or(0); Some(("test", format!("{}, {}", xi(0, kz, dv), aqn(imm as i64)), None, false, false, false, false)) }

            
            0x88 => {
                let (rm, reg) = self.decode_modrm_operands(1, dv, gb, cq, gp);
                Some(("mov", format!("{}, {}", rm, reg), None, false, false, false, false))
            }
            0x89 => {
                let (rm, reg) = self.decode_modrm_operands(kz, dv, gb, cq, gp);
                Some(("mov", format!("{}, {}", rm, reg), None, false, false, false, false))
            }

            
            0x8A => {
                let (rm, reg) = self.decode_modrm_operands(1, dv, gb, cq, gp);
                Some(("mov", format!("{}, {}", reg, rm), None, false, false, false, false))
            }
            0x8B => {
                let (rm, reg) = self.decode_modrm_operands(kz, dv, gb, cq, gp);
                Some(("mov", format!("{}, {}", reg, rm), None, false, false, false, false))
            }

            
            0x8D => {
                let (rm, reg) = self.decode_modrm_operands(kz, dv, gb, cq, gp);
                Some(("lea", format!("{}, {}", reg, rm), None, false, false, false, false))
            }

            
            0x86 => {
                let (rm, reg) = self.decode_modrm_operands(1, dv, gb, cq, gp);
                Some(("xchg", format!("{}, {}", rm, reg), None, false, false, false, false))
            }
            0x87 => {
                let (rm, reg) = self.decode_modrm_operands(kz, dv, gb, cq, gp);
                Some(("xchg", format!("{}, {}", rm, reg), None, false, false, false, false))
            }

            
            0xC6 => {
                let rm = self.decode_modrm_rm_only(1, dv, cq, gp);
                let imm = self.read_u8().unwrap_or(0);
                Some(("mov", format!("{}, {:#x}", rm, imm), None, false, false, false, false))
            }
            0xC7 => {
                let rm = self.decode_modrm_rm_only(kz, dv, cq, gp);
                let imm = self.read_i32().unwrap_or(0);
                Some(("mov", format!("{}, {}", rm, aqn(imm as i64)), None, false, false, false, false))
            }

            
            0x80 => self.decode_group1(1, dv, cq, gp, true),
            0x81 => self.decode_group1(kz, dv, cq, gp, false),
            0x83 => self.decode_group1(kz, dv, cq, gp, true),

            
            0xC0 => self.decode_shift(1, dv, cq, gp, ShiftCount::Imm8),
            0xC1 => self.decode_shift(kz, dv, cq, gp, ShiftCount::Imm8),
            0xD0 => self.decode_shift(1, dv, cq, gp, ShiftCount::One),
            0xD1 => self.decode_shift(kz, dv, cq, gp, ShiftCount::One),
            0xD2 => self.decode_shift(1, dv, cq, gp, ShiftCount::CL),
            0xD3 => self.decode_shift(kz, dv, cq, gp, ShiftCount::CL),

            
            0xFE => self.decode_group_fe(1, dv, gb, cq, gp, addr, start),
            0xFF => self.decode_group_fe(kz, dv, gb, cq, gp, addr, start),

            
            0xF6 => self.decode_group3(1, dv, cq, gp),
            0xF7 => self.decode_group3(kz, dv, cq, gp),

            
            
            0x6B => {
                let (rm, reg) = self.decode_modrm_operands(kz, dv, gb, cq, gp);
                let imm = self.read_i8().unwrap_or(0);
                Some(("imul", format!("{}, {}, {}", reg, rm, aqn(imm as i64)), None, false, false, false, false))
            }
            
            0x69 => {
                let (rm, reg) = self.decode_modrm_operands(kz, dv, gb, cq, gp);
                let imm = self.read_i32().unwrap_or(0);
                Some(("imul", format!("{}, {}, {}", reg, rm, aqn(imm as i64)), None, false, false, false, false))
            }

            
            0xA4 => Some(( if czb { "rep movsb" } else { "movsb" }, String::new(), None, false, false, false, false)),
            0xA5 => Some(( if czb { "rep movsd" } else { "movsd" }, String::new(), None, false, false, false, false)),
            0xAA => Some(( if czb { "rep stosb" } else { "stosb" }, String::new(), None, false, false, false, false)),
            0xAB => Some(( if czb { "rep stosd" } else { "stosd" }, String::new(), None, false, false, false, false)),
            0xAC => Some(("lodsb", String::new(), None, false, false, false, false)),
            0xAD => Some(("lodsd", String::new(), None, false, false, false, false)),
            0xAE => Some(( if eot { "repne scasb" } else { "scasb" }, String::new(), None, false, false, false, false)),
            0xAF => Some(( if eot { "repne scasd" } else { "scasd" }, String::new(), None, false, false, false, false)),

            
            0xA0 => {
                let cmy = self.read_u64_or_u32(rex_w);
                Some(("mov", format!("al, [{}]", ene(cmy)), None, false, false, false, false))
            }
            0xA1 => {
                let cmy = self.read_u64_or_u32(rex_w);
                Some(("mov", format!("{}, [{}]", xi(0, kz, dv), ene(cmy)), None, false, false, false, false))
            }
            0xA2 => {
                let cmy = self.read_u64_or_u32(rex_w);
                Some(("mov", format!("[{}], al", ene(cmy)), None, false, false, false, false))
            }
            0xA3 => {
                let cmy = self.read_u64_or_u32(rex_w);
                Some(("mov", format!("[{}], {}", ene(cmy), xi(0, kz, dv)), None, false, false, false, false))
            }

            
            0xE4 => { let aa = self.read_u8().unwrap_or(0); Some(("in", format!("al, {:#x}", aa), None, false, false, false, false)) }
            0xE5 => { let aa = self.read_u8().unwrap_or(0); Some(("in", format!("eax, {:#x}", aa), None, false, false, false, false)) }
            0xE6 => { let aa = self.read_u8().unwrap_or(0); Some(("out", format!("{:#x}, al", aa), None, false, false, false, false)) }
            0xE7 => { let aa = self.read_u8().unwrap_or(0); Some(("out", format!("{:#x}, eax", aa), None, false, false, false, false)) }
            0xEC => Some(("in", String::from("al, dx"), None, false, false, false, false)),
            0xED => Some(("in", String::from("eax, dx"), None, false, false, false, false)),
            0xEE => Some(("out", String::from("dx, al"), None, false, false, false, false)),
            0xEF => Some(("out", String::from("dx, eax"), None, false, false, false, false)),

            
            _ => None,
        };

        match result {
            Some((akc, ops, target, is_call, is_ret, is_jump, is_cond_jump)) => {
                let bytes = self.code[start..self.pos].to_vec();
                Bj {
                    address: addr,
                    bytes,
                    mnemonic: String::from(akc),
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

    

    fn decode_0f(&mut self, start: usize, addr: u64, rp: u8, kz: u8, dv: bool, hdi: bool, hdj: bool) -> Bj {
        if self.pos >= self.code.len() {
            return self.make_db(start, addr);
        }

        let gb = (rp & 0x04) != 0;
        let cq = (rp & 0x01) != 0;
        let gp = (rp & 0x02) != 0;

        let cnk = self.code[self.pos];
        self.pos += 1;

        let result: Option<(&str, String, Option<u64>, bool, bool, bool, bool)> = match cnk {
            
            0x05 => Some(("syscall", String::new(), None, false, false, false, false)),

            
            0x07 => Some(("sysret", String::new(), None, false, false, false, false)),

            
            0xA2 => Some(("cpuid", String::new(), None, false, false, false, false)),

            
            0x31 => Some(("rdtsc", String::new(), None, false, false, false, false)),

            
            0x32 => Some(("rdmsr", String::new(), None, false, false, false, false)),
            0x30 => Some(("wrmsr", String::new(), None, false, false, false, false)),

            
            0x1F => {
                
                let _ = self.decode_modrm_rm_only(kz, dv, cq, gp);
                Some(("nop", String::new(), None, false, false, false, false))
            }

            
            0x80..=0x8F => {
                let ft = cnk - 0x80;
                let ot = self.read_i32().unwrap_or(0) as i64;
                let target = (addr as i64 + (self.pos - start) as i64 + ot) as u64;
                let duk = format!("j{}", TA_[ft as usize]);
                Some((esm(&duk), format!("{:#x}", target), Some(target), false, false, false, true))
            }

            
            0x90..=0x9F => {
                let ft = cnk - 0x90;
                let rm = self.decode_modrm_rm_only(1, dv, cq, gp);
                let duk = format!("set{}", TA_[ft as usize]);
                Some((esm(&duk), rm, None, false, false, false, false))
            }

            
            0x40..=0x4F => {
                let ft = cnk - 0x40;
                let (rm, reg) = self.decode_modrm_operands(kz, dv, gb, cq, gp);
                let duk = format!("cmov{}", TA_[ft as usize]);
                Some((esm(&duk), format!("{}, {}", reg, rm), None, false, false, false, false))
            }

            
            0xB6 => {
                let (rm, reg) = self.decode_modrm_operands(1, dv, gb, cq, gp);
                Some(("movzx", format!("{}, {}", xi(eum(gb, self.peek_back()), kz, dv), rm), None, false, false, false, false))
            }

            
            0xB7 => {
                let (rm, reg) = self.decode_modrm_operands(2, dv, gb, cq, gp);
                Some(("movzx", format!("{}, {}", xi(eum(gb, self.peek_back()), kz, dv), rm), None, false, false, false, false))
            }

            
            0xBE => {
                let (rm, reg) = self.decode_modrm_operands(1, dv, gb, cq, gp);
                Some(("movsx", format!("{}, {}", xi(eum(gb, self.peek_back()), kz, dv), rm), None, false, false, false, false))
            }

            
            0xBF => {
                let (rm, reg) = self.decode_modrm_operands(2, dv, gb, cq, gp);
                Some(("movsx", format!("{}, {}", xi(eum(gb, self.peek_back()), kz, dv), rm), None, false, false, false, false))
            }

            
            0xAF => {
                let (rm, reg) = self.decode_modrm_operands(kz, dv, gb, cq, gp);
                Some(("imul", format!("{}, {}", reg, rm), None, false, false, false, false))
            }

            
            0xBC => {
                let (rm, reg) = self.decode_modrm_operands(kz, dv, gb, cq, gp);
                Some(("bsf", format!("{}, {}", reg, rm), None, false, false, false, false))
            }
            0xBD => {
                let (rm, reg) = self.decode_modrm_operands(kz, dv, gb, cq, gp);
                Some(("bsr", format!("{}, {}", reg, rm), None, false, false, false, false))
            }

            
            0xA3 => {
                let (rm, reg) = self.decode_modrm_operands(kz, dv, gb, cq, gp);
                Some(("bt", format!("{}, {}", rm, reg), None, false, false, false, false))
            }
            0xAB => {
                let (rm, reg) = self.decode_modrm_operands(kz, dv, gb, cq, gp);
                Some(("bts", format!("{}, {}", rm, reg), None, false, false, false, false))
            }
            0xB3 => {
                let (rm, reg) = self.decode_modrm_operands(kz, dv, gb, cq, gp);
                Some(("btr", format!("{}, {}", rm, reg), None, false, false, false, false))
            }
            0xBB => {
                let (rm, reg) = self.decode_modrm_operands(kz, dv, gb, cq, gp);
                Some(("btc", format!("{}, {}", rm, reg), None, false, false, false, false))
            }

            
            0xC0 => {
                let (rm, reg) = self.decode_modrm_operands(1, dv, gb, cq, gp);
                Some(("xadd", format!("{}, {}", rm, reg), None, false, false, false, false))
            }
            0xC1 => {
                let (rm, reg) = self.decode_modrm_operands(kz, dv, gb, cq, gp);
                Some(("xadd", format!("{}, {}", rm, reg), None, false, false, false, false))
            }

            
            0xC8..=0xCF => {
                let r = (cnk - 0xC8) | if cq { 8 } else { 0 };
                Some(("bswap", String::from(xi(r, kz, dv)), None, false, false, false, false))
            }

            
            0x0B => Some(("ud2", String::new(), None, false, false, false, false)),

            _ => None,
        };

        match result {
            Some((akc, ops, target, is_call, is_ret, is_jump, is_cond_jump)) => {
                let bytes = self.code[start..self.pos].to_vec();
                Bj {
                    address: addr,
                    bytes,
                    mnemonic: String::from(akc),
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
                
                self.make_db(start, addr)
            }
        }
    }

    

    
    fn decode_modrm_operands(&mut self, size: u8, dv: bool, gb: bool, cq: bool, gp: bool) -> (String, String) {
        if self.pos >= self.code.len() {
            return (String::from("?"), String::from("?"));
        }

        let fi = self.code[self.pos];
        self.pos += 1;

        let bct = (fi >> 6) & 3;
        let cpg = ((fi >> 3) & 7) | if gb { 8 } else { 0 };
        let cdj = (fi & 7) | if cq { 8 } else { 0 };

        let oeg = String::from(xi(cpg, size, dv));
        let ohp = self.decode_rm(bct, cdj & 7, cq, gp, size, dv);

        (ohp, oeg)
    }

    
    fn decode_modrm_rm_only(&mut self, size: u8, dv: bool, cq: bool, gp: bool) -> String {
        if self.pos >= self.code.len() {
            return String::from("?");
        }

        let fi = self.code[self.pos];
        self.pos += 1;

        let bct = (fi >> 6) & 3;
        let cdj = (fi & 7) | if cq { 8 } else { 0 };

        self.decode_rm(bct, cdj & 7, cq, gp, size, dv)
    }

    
    fn decode_rm(&mut self, bct: u8, rm_low: u8, cq: bool, gp: bool, size: u8, dv: bool) -> String {
        let rm = rm_low | if cq { 8 } else { 0 };

        
        if bct == 3 {
            return String::from(xi(rm, size, dv));
        }

        
        let (bql, needs_sib) = if rm_low == 4 {
            
            (String::new(), true)
        } else if rm_low == 5 && bct == 0 {
            
            let uv = self.read_i32().unwrap_or(0);
            return format!("{} [rip{:+#x}]", avd(size), uv);
        } else {
            (String::from(xi(rm, 8, dv)), false)
        };

        if needs_sib {
            return self.decode_sib(bct, cq, gp, size, dv);
        }

        
        match bct {
            0 => format!("{} [{}]", avd(size), bql),
            1 => {
                let uv = self.read_i8().unwrap_or(0) as i32;
                if uv == 0 {
                    format!("{} [{}]", avd(size), bql)
                } else {
                    format!("{} [{}{:+#x}]", avd(size), bql, uv)
                }
            }
            2 => {
                let uv = self.read_i32().unwrap_or(0);
                if uv == 0 {
                    format!("{} [{}]", avd(size), bql)
                } else {
                    format!("{} [{}{:+#x}]", avd(size), bql, uv)
                }
            }
            _ => String::from("?"),
        }
    }

    
    fn decode_sib(&mut self, bct: u8, cq: bool, gp: bool, size: u8, dv: bool) -> String {
        if self.pos >= self.code.len() {
            return String::from("?");
        }

        let dzk = self.code[self.pos];
        self.pos += 1;

        let scale = 1u8 << ((dzk >> 6) & 3);
        let index = ((dzk >> 3) & 7) | if gp { 8 } else { 0 };
        let base = (dzk & 7) | if cq { 8 } else { 0 };

        let idp = index != 4; 
        let bql = if (base & 7) == 5 && bct == 0 {
            
            let uv = self.read_i32().unwrap_or(0);
            if idp {
                if scale > 1 {
                    return format!("{} [{}*{}{:+#x}]", avd(size), xi(index, 8, dv), scale, uv);
                } else {
                    return format!("{} [{}{:+#x}]", avd(size), xi(index, 8, dv), uv);
                }
            } else {
                return format!("{} [{:#x}]", avd(size), uv);
            }
        } else {
            String::from(xi(base, 8, dv))
        };

        
        let dhd = if idp {
            if scale > 1 {
                format!("{}+{}*{}", bql, xi(index, 8, dv), scale)
            } else {
                format!("{}+{}", bql, xi(index, 8, dv))
            }
        } else {
            bql
        };

        match bct {
            0 => format!("{} [{}]", avd(size), dhd),
            1 => {
                let uv = self.read_i8().unwrap_or(0) as i32;
                if uv == 0 {
                    format!("{} [{}]", avd(size), dhd)
                } else {
                    format!("{} [{}{:+#x}]", avd(size), dhd, uv)
                }
            }
            2 => {
                let uv = self.read_i32().unwrap_or(0);
                if uv == 0 {
                    format!("{} [{}]", avd(size), dhd)
                } else {
                    format!("{} [{}{:+#x}]", avd(size), dhd, uv)
                }
            }
            _ => String::from("?"),
        }
    }

    

    fn decode_alu_rm(&mut self, _start: usize, _addr: u64, opcode: u8, mnemonic: &str, kz: u8, dv: bool, gb: bool, cq: bool, gp: bool) -> Option<(&'static str, String, Option<u64>, bool, bool, bool, bool)> {
        let mrz = (opcode & 1) == 0;
        let it = (opcode & 2) != 0; 
        let fq = if mrz { 1 } else { kz };
        let (rm, reg) = self.decode_modrm_operands(fq, dv, gb, cq, gp);
        let ops = if it {
            format!("{}, {}", reg, rm)
        } else {
            format!("{}, {}", rm, reg)
        };
        let akc: &'static str = match mnemonic {
            "add" => "add", "or" => "or", "adc" => "adc", "sbb" => "sbb",
            "and" => "and", "sub" => "sub", "xor" => "xor", "cmp" => "cmp",
            _ => "?alu",
        };
        Some((akc, ops, None, false, false, false, false))
    }

    fn decode_group1(&mut self, size: u8, dv: bool, cq: bool, gp: bool, imm8: bool) -> Option<(&'static str, String, Option<u64>, bool, bool, bool, bool)> {
        if self.pos >= self.code.len() { return None; }
        let fi = self.code[self.pos]; 
        let op = (fi >> 3) & 7;

        let rm = self.decode_modrm_rm_only(size, dv, cq, gp);
        let imm = if imm8 {
            self.read_i8().unwrap_or(0) as i64
        } else {
            self.read_i32().unwrap_or(0) as i64
        };

        let akc: &'static str = match op {
            0 => "add", 1 => "or", 2 => "adc", 3 => "sbb",
            4 => "and", 5 => "sub", 6 => "xor", 7 => "cmp",
            _ => "?",
        };

        Some((akc, format!("{}, {}", rm, aqn(imm)), None, false, false, false, false))
    }

    fn decode_shift(&mut self, size: u8, dv: bool, cq: bool, gp: bool, count: ShiftCount) -> Option<(&'static str, String, Option<u64>, bool, bool, bool, bool)> {
        if self.pos >= self.code.len() { return None; }
        let fi = self.code[self.pos];
        let op = (fi >> 3) & 7;

        let rm = self.decode_modrm_rm_only(size, dv, cq, gp);
        let cht = match count {
            ShiftCount::One => String::from("1"),
            ShiftCount::CL => String::from("cl"),
            ShiftCount::Imm8 => {
                let imm = self.read_u8().unwrap_or(0);
                format!("{}", imm)
            }
        };

        let akc: &'static str = match op {
            0 => "rol", 1 => "ror", 2 => "rcl", 3 => "rcr",
            4 => "shl", 5 => "shr", 6 => "sal", 7 => "sar",
            _ => "?",
        };

        Some((akc, format!("{}, {}", rm, cht), None, false, false, false, false))
    }

    fn decode_group_fe(&mut self, size: u8, dv: bool, _rex_r: bool, cq: bool, gp: bool, addr: u64, start: usize) -> Option<(&'static str, String, Option<u64>, bool, bool, bool, bool)> {
        if self.pos >= self.code.len() { return None; }
        let fi = self.code[self.pos];
        let op = (fi >> 3) & 7;

        match op {
            0 => {
                let rm = self.decode_modrm_rm_only(size, dv, cq, gp);
                Some(("inc", rm, None, false, false, false, false))
            }
            1 => {
                let rm = self.decode_modrm_rm_only(size, dv, cq, gp);
                Some(("dec", rm, None, false, false, false, false))
            }
            2 if size > 1 => {
                
                let rm = self.decode_modrm_rm_only(8, dv, cq, gp);
                Some(("call", rm, None, true, false, false, false))
            }
            4 if size > 1 => {
                
                let rm = self.decode_modrm_rm_only(8, dv, cq, gp);
                Some(("jmp", rm, None, false, false, true, false))
            }
            6 if size > 1 => {
                
                let rm = self.decode_modrm_rm_only(8, dv, cq, gp);
                Some(("push", rm, None, false, false, false, false))
            }
            _ => {
                let _ = self.decode_modrm_rm_only(size, dv, cq, gp);
                None
            }
        }
    }

    fn decode_group3(&mut self, size: u8, dv: bool, cq: bool, gp: bool) -> Option<(&'static str, String, Option<u64>, bool, bool, bool, bool)> {
        if self.pos >= self.code.len() { return None; }
        let fi = self.code[self.pos];
        let op = (fi >> 3) & 7;

        match op {
            0 | 1 => {
                
                let rm = self.decode_modrm_rm_only(size, dv, cq, gp);
                let imm = if size == 1 {
                    self.read_u8().unwrap_or(0) as i64
                } else {
                    self.read_i32().unwrap_or(0) as i64
                };
                Some(("test", format!("{}, {}", rm, aqn(imm)), None, false, false, false, false))
            }
            2 => {
                let rm = self.decode_modrm_rm_only(size, dv, cq, gp);
                Some(("not", rm, None, false, false, false, false))
            }
            3 => {
                let rm = self.decode_modrm_rm_only(size, dv, cq, gp);
                Some(("neg", rm, None, false, false, false, false))
            }
            4 => {
                let rm = self.decode_modrm_rm_only(size, dv, cq, gp);
                Some(("mul", rm, None, false, false, false, false))
            }
            5 => {
                let rm = self.decode_modrm_rm_only(size, dv, cq, gp);
                Some(("imul", rm, None, false, false, false, false))
            }
            6 => {
                let rm = self.decode_modrm_rm_only(size, dv, cq, gp);
                Some(("div", rm, None, false, false, false, false))
            }
            7 => {
                let rm = self.decode_modrm_rm_only(size, dv, cq, gp);
                Some(("idiv", rm, None, false, false, false, false))
            }
            _ => None,
        }
    }

    

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

    fn read_u64_or_u32(&mut self, cba: bool) -> u64 {
        if cba {
            self.read_i64().unwrap_or(0) as u64
        } else {
            self.read_i32().unwrap_or(0) as u64
        }
    }

    
    fn peek_back(&self) -> u8 {
        if self.pos > 0 {
            
            
            self.code.get(self.pos.wrapping_sub(1)).copied().unwrap_or(0)
        } else {
            0
        }
    }

    fn make_db(&mut self, start: usize, addr: u64) -> Bj {
        
        if self.pos <= start {
            self.pos = start + 1;
        }
        let end = self.pos.min(self.code.len());
        let bytes = self.code[start..end].to_vec();
        let ga: String = bytes.iter().map(|b| format!("{:#04x}", b)).collect::<Vec<_>>().join(", ");
        Bj {
            address: addr,
            bytes,
            mnemonic: String::from("db"),
            operands_str: ga,
            comment: None,
            branch_target: None,
            is_call: false,
            is_ret: false,
            is_jump: false,
            is_cond_jump: false,
        }
    }
}



enum ShiftCount { One, CL, Imm8 }

fn aqn(v: i64) -> String {
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

fn ene(v: u64) -> String {
    format!("{:#x}", v)
}


fn eum(gb: bool, fi: u8) -> u8 {
    ((fi >> 3) & 7) | if gb { 8 } else { 0 }
}




fn esm(j: &str) -> &'static str {
    use alloc::boxed::Box;
    let boxed = String::from(j).into_boxed_str();
    Box::leak(boxed)
}




pub fn jwh(
    instructions: &mut [Bj],
    addr_to_symbol: &alloc::collections::BTreeMap<u64, String>,
) {
    
    let mut gez: Option<i64> = None;

    for inst in instructions.iter_mut() {
        
        if let Some(target) = inst.branch_target {
            if let Some(name) = addr_to_symbol.get(&target) {
                inst.comment = Some(format!("<{}>", name));
            }
        }

        
        if inst.mnemonic == "mov" && (inst.operands_str.starts_with("eax,") || inst.operands_str.starts_with("rax,")) {
            
            if let Some(comma_pos) = inst.operands_str.find(',') {
                let moe = inst.operands_str[comma_pos + 1..].trim();
                if let Some(val) = nqp(moe) {
                    gez = Some(val);
                }
            }
        } else if inst.mnemonic == "xor" && inst.operands_str.contains("eax") && inst.operands_str.matches("eax").count() == 2 {
            gez = Some(0);
        }

        
        if inst.mnemonic == "syscall" {
            if let Some(num) = gez {
                let name = crate::transpiler::dfe(num as u64);
                inst.comment = Some(format!("sys_{} ({})", name, num));
            }
        }
    }
}

fn nqp(j: &str) -> Option<i64> {
    let j = j.trim();
    if j.starts_with("0x") || j.starts_with("0X") {
        i64::from_str_radix(&j[2..], 16).ok()
    } else if j.starts_with("-0x") || j.starts_with("-0X") {
        i64::from_str_radix(&j[3..], 16).ok().map(|v| -v)
    } else {
        j.parse::<i64>().ok()
    }
}
