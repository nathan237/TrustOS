

















use alloc::vec::Vec;
use alloc::format;
use alloc::string::String;
use super::ir::*;


fn qs(x86_reg: u8) -> Reg {
    match x86_reg {
        0  => Reg::X10,  
        1  => Reg::X11,  
        2  => Reg::X12,  
        3  => Reg::X13,  
        4  => Reg::X2,   
        5  => Reg::X8,   
        6  => Reg::X14,  
        7  => Reg::X15,  
        8  => Reg::X18,  
        9  => Reg::X19,  
        10 => Reg::X20,  
        11 => Reg::X21,  
        12 => Reg::X22,  
        13 => Reg::X23,  
        14 => Reg::X24,  
        15 => Reg::X25,  
        _  => Reg::X5,   
    }
}


pub struct X86Decoder {
    
    code: Vec<u8>,
    
    base_addr: u64,
    
    offset: usize,
    
    pub stats: TranslationStats,
}

impl X86Decoder {
    pub fn new(code: &[u8], base_addr: u64) -> Self {
        Self {
            code: code.to_vec(),
            base_addr,
            offset: 0,
            stats: TranslationStats::default(),
        }
    }

    
    pub fn translate_block(&mut self, azv: usize) -> TranslatedBlock {
        self.offset = azv;
        let src_addr = self.base_addr + azv as u64;
        let mut block = TranslatedBlock::new(src_addr, SourceArch::X86_64);

        let max_instructions = 256;
        let mut count = 0;

        while self.offset < self.code.len() && count < max_instructions {
            let qlo = self.offset;
            let gyh = self.decode_one(&mut block);
            block.src_inst_count += 1;
            self.stats.instructions_translated += 1;
            count += 1;

            if gyh {
                break;
            }
        }

        self.stats.blocks_translated += 1;
        self.stats.rv_instructions_emitted += block.instructions.len() as u64;
        block
    }

    
    pub fn translate_all(&mut self) -> Vec<TranslatedBlock> {
        let mut blocks = Vec::new();
        let mut csx: Vec<usize> = Vec::new();
        let mut anc: Vec<u64> = Vec::new();

        csx.push(0);

        while let Some(offset) = csx.pop() {
            let addr = self.base_addr + offset as u64;
            if anc.contains(&addr) {
                continue;
            }
            anc.push(addr);

            let block = self.translate_block(offset);

            
            for &succ in &block.successors {
                if succ >= self.base_addr {
                    let eaq = (succ - self.base_addr) as usize;
                    if eaq < self.code.len() && !anc.contains(&succ) {
                        csx.push(eaq);
                    }
                }
            }

            blocks.push(block);
        }

        blocks
    }

    
    
    fn decode_one(&mut self, block: &mut TranslatedBlock) -> bool {
        if self.offset >= self.code.len() {
            return true;
        }

        let bhf = self.base_addr + self.offset as u64;

        
        let mut rp: u8 = 0;
        let mut dv = false;
        let mut nnm = false;

        loop {
            if self.offset >= self.code.len() { return true; }
            let b = self.code[self.offset];
            match b {
                0x66 => { nnm = true; self.offset += 1; }
                0x40..=0x4F => { rp = b; dv = true; self.offset += 1; }
                0xF0 | 0xF2 | 0xF3 | 0x2E | 0x3E | 0x26 | 0x64 | 0x65 | 0x36 => {
                    self.offset += 1; 
                }
                _ => break,
            }
        }

        if self.offset >= self.code.len() { return true; }

        let rex_w = dv && (rp & 0x08) != 0;
        let gb = dv && (rp & 0x04) != 0;
        let gp = dv && (rp & 0x02) != 0;
        let cq = dv && (rp & 0x01) != 0;

        let opcode = self.code[self.offset];
        self.offset += 1;

        match opcode {
            
            0x90 => {
                block.emit(RvInst::Nop);
                false
            }

            
            0xC7 => {
                let (rm, _) = self.decode_modrm(cq, gb);
                let imm = self.read_i32() as i64;
                let aj = qs(rm);
                block.emit(RvInst::Li { aj, imm });
                false
            }

            
            0xB8..=0xBF => {
                let reg = (opcode - 0xB8) + if cq { 8 } else { 0 };
                let aj = qs(reg);
                let imm = if rex_w {
                    self.read_i64()
                } else {
                    self.read_i32() as i64
                };
                block.emit(RvInst::Li { aj, imm });
                false
            }

            
            0xB0..=0xB7 => {
                let reg = (opcode - 0xB0) + if cq { 8 } else { 0 };
                let aj = qs(reg);
                let imm = self.read_u8() as i64;
                
                block.emit(RvInst::Andi { aj, rs1: aj, imm: !0xFF });
                block.emit(RvInst::Ori { aj, rs1: aj, imm: imm & 0xFF });
                false
            }

            
            0x89 => {
                let (rm, reg) = self.decode_modrm(cq, gb);
                let oc = qs(reg);
                let aj = qs(rm);
                block.emit(RvInst::Mv { aj, oc });
                false
            }
            0x8B => {
                let (rm, reg) = self.decode_modrm(cq, gb);
                let oc = qs(rm);
                let aj = qs(reg);
                block.emit(RvInst::Mv { aj, oc });
                false
            }

            
            0x50..=0x57 => {
                let reg = (opcode - 0x50) + if cq { 8 } else { 0 };
                let oc = qs(reg);
                
                block.emit(RvInst::Addi { aj: Reg::X2, rs1: Reg::X2, imm: -8 });
                block.emit(RvInst::Sd { rs2: oc, rs1: Reg::X2, offset: 0 });
                false
            }

            
            0x58..=0x5F => {
                let reg = (opcode - 0x58) + if cq { 8 } else { 0 };
                let aj = qs(reg);
                
                block.emit(RvInst::Ld { aj, rs1: Reg::X2, offset: 0 });
                block.emit(RvInst::Addi { aj: Reg::X2, rs1: Reg::X2, imm: 8 });
                false
            }

            
            0x01 => {
                let (rm, reg) = self.decode_modrm(cq, gb);
                let aj = qs(rm);
                let oc = qs(reg);
                block.emit(RvInst::Add { aj, rs1: aj, rs2: oc });
                block.emit(RvInst::CmpFlags { rs1: aj, rs2: Reg::X0 });
                false
            }

            
            0x03 => {
                let (rm, reg) = self.decode_modrm(cq, gb);
                let aj = qs(reg);
                let oc = qs(rm);
                block.emit(RvInst::Add { aj, rs1: aj, rs2: oc });
                block.emit(RvInst::CmpFlags { rs1: aj, rs2: Reg::X0 });
                false
            }

            
            0x29 => {
                let (rm, reg) = self.decode_modrm(cq, gb);
                let aj = qs(rm);
                let oc = qs(reg);
                block.emit(RvInst::Sub { aj, rs1: aj, rs2: oc });
                block.emit(RvInst::CmpFlags { rs1: aj, rs2: Reg::X0 });
                false
            }

            
            0x2B => {
                let (rm, reg) = self.decode_modrm(cq, gb);
                let aj = qs(reg);
                let oc = qs(rm);
                block.emit(RvInst::Sub { aj, rs1: aj, rs2: oc });
                block.emit(RvInst::CmpFlags { rs1: aj, rs2: Reg::X0 });
                false
            }

            
            0x21 => {
                let (rm, reg) = self.decode_modrm(cq, gb);
                let aj = qs(rm);
                let oc = qs(reg);
                block.emit(RvInst::And { aj, rs1: aj, rs2: oc });
                block.emit(RvInst::CmpFlags { rs1: aj, rs2: Reg::X0 });
                false
            }

            
            0x09 => {
                let (rm, reg) = self.decode_modrm(cq, gb);
                let aj = qs(rm);
                let oc = qs(reg);
                block.emit(RvInst::Or { aj, rs1: aj, rs2: oc });
                block.emit(RvInst::CmpFlags { rs1: aj, rs2: Reg::X0 });
                false
            }

            
            0x31 => {
                let (rm, reg) = self.decode_modrm(cq, gb);
                let aj = qs(rm);
                let oc = qs(reg);
                block.emit(RvInst::Xor { aj, rs1: aj, rs2: oc });
                block.emit(RvInst::CmpFlags { rs1: aj, rs2: Reg::X0 });
                false
            }

            
            0x39 => {
                let (rm, reg) = self.decode_modrm(cq, gb);
                let zk = qs(rm);
                let dyd = qs(reg);
                
                block.emit(RvInst::CmpFlags { rs1: zk, rs2: dyd });
                false
            }

            
            0x3B => {
                let (rm, reg) = self.decode_modrm(cq, gb);
                let zk = qs(rm);
                let dyd = qs(reg);
                block.emit(RvInst::CmpFlags { rs1: dyd, rs2: zk });
                false
            }

            
            0x83 => {
                let (rm, op_ext) = self.decode_modrm(cq, gb);
                let imm = self.read_i8() as i64;
                let aj = qs(rm);
                match op_ext {
                    0 => { 
                        block.emit(RvInst::Addi { aj, rs1: aj, imm });
                        block.emit(RvInst::CmpFlags { rs1: aj, rs2: Reg::X0 });
                    }
                    4 => { 
                        block.emit(RvInst::Andi { aj, rs1: aj, imm });
                        block.emit(RvInst::CmpFlags { rs1: aj, rs2: Reg::X0 });
                    }
                    5 => { 
                        block.emit(RvInst::Addi { aj, rs1: aj, imm: -imm });
                        block.emit(RvInst::CmpFlags { rs1: aj, rs2: Reg::X0 });
                    }
                    7 => { 
                        block.emit(RvInst::Li { aj: Reg::X5, imm });
                        block.emit(RvInst::CmpFlags { rs1: aj, rs2: Reg::X5 });
                    }
                    _ => {
                        self.stats.unsupported_instructions += 1;
                        block.emit(RvInst::Nop);
                    }
                }
                false
            }

            
            0x85 => {
                let (rm, reg) = self.decode_modrm(cq, gb);
                let zk = qs(rm);
                let dyd = qs(reg);
                
                block.emit(RvInst::And { aj: Reg::X5, rs1: zk, rs2: dyd });
                block.emit(RvInst::CmpFlags { rs1: Reg::X5, rs2: Reg::X0 });
                false
            }

            
            0x8D => {
                let (rm, reg) = self.decode_modrm(cq, gb);
                let aj = qs(reg);
                let oc = qs(rm);
                
                block.emit(RvInst::Mv { aj, oc });
                false
            }

            
            0xFF => {
                let (rm, op_ext) = self.decode_modrm(cq, gb);
                let aj = qs(rm);
                match op_ext {
                    0 => { 
                        block.emit(RvInst::Addi { aj, rs1: aj, imm: 1 });
                        block.emit(RvInst::CmpFlags { rs1: aj, rs2: Reg::X0 });
                    }
                    1 => { 
                        block.emit(RvInst::Addi { aj, rs1: aj, imm: -1 });
                        block.emit(RvInst::CmpFlags { rs1: aj, rs2: Reg::X0 });
                    }
                    2 => { 
                        
                        block.emit(RvInst::Addi { aj: Reg::X2, rs1: Reg::X2, imm: -8 });
                        let dve = self.base_addr + self.offset as u64;
                        block.emit(RvInst::Li { aj: Reg::X5, imm: dve as i64 });
                        block.emit(RvInst::Sd { rs2: Reg::X5, rs1: Reg::X2, offset: 0 });
                        block.emit(RvInst::Jalr { aj: Reg::X1, rs1: aj, offset: 0 });
                        block.successors.push(0); 
                        return true;
                    }
                    4 => { 
                        block.emit(RvInst::Jalr { aj: Reg::X0, rs1: aj, offset: 0 });
                        return true;
                    }
                    6 => { 
                        block.emit(RvInst::Addi { aj: Reg::X2, rs1: Reg::X2, imm: -8 });
                        block.emit(RvInst::Sd { rs2: aj, rs1: Reg::X2, offset: 0 });
                    }
                    _ => {
                        self.stats.unsupported_instructions += 1;
                        block.emit(RvInst::Nop);
                    }
                }
                false
            }

            
            0x70..=0x7F => {
                let ot = self.read_i8() as i64;
                let target = self.base_addr as i64 + self.offset as i64 + ot;
                let fc = jru(opcode & 0x0F);
                block.emit(RvInst::BranchCond { fc, offset: target });
                let fwe = self.base_addr + self.offset as u64;
                block.successors.push(target as u64);
                block.successors.push(fwe);
                true
            }

            
            0xEB => {
                let ot = self.read_i8() as i64;
                let target = self.base_addr as i64 + self.offset as i64 + ot;
                block.emit(RvInst::Jal { aj: Reg::X0, offset: target });
                block.successors.push(target as u64);
                true
            }

            
            0xE9 => {
                let ot = self.read_i32() as i64;
                let target = self.base_addr as i64 + self.offset as i64 + ot;
                block.emit(RvInst::Jal { aj: Reg::X0, offset: target });
                block.successors.push(target as u64);
                true
            }

            
            0xE8 => {
                let ot = self.read_i32() as i64;
                let target = self.base_addr as i64 + self.offset as i64 + ot;
                
                block.emit(RvInst::Addi { aj: Reg::X2, rs1: Reg::X2, imm: -8 });
                let bdk = self.base_addr + self.offset as u64;
                block.emit(RvInst::Li { aj: Reg::X5, imm: bdk as i64 });
                block.emit(RvInst::Sd { rs2: Reg::X5, rs1: Reg::X2, offset: 0 });
                block.emit(RvInst::Call { offset: target });
                block.successors.push(target as u64);
                block.successors.push(bdk);
                true
            }

            
            0xC3 => {
                
                block.emit(RvInst::Ld { aj: Reg::X1, rs1: Reg::X2, offset: 0 });
                block.emit(RvInst::Addi { aj: Reg::X2, rs1: Reg::X2, imm: 8 });
                block.emit(RvInst::Ret);
                true
            }

            
            0x0F => {
                if self.offset < self.code.len() {
                    let hjh = self.code[self.offset];
                    self.offset += 1;

                    match hjh {
                        0x05 => {
                            
                            
                            
                            block.emit(RvInst::SrcAnnotation {
                                arch: SourceArch::X86_64,
                                addr: bhf,
                                text: String::from("syscall"),
                            });
                            
                            block.emit(RvInst::Mv { aj: Reg::X17, oc: Reg::X10 });
                            
                            
                            block.emit(RvInst::Mv { aj: Reg::X5, oc: Reg::X15 });  
                            block.emit(RvInst::Mv { aj: Reg::X10, oc: Reg::X5 });  
                            
                            block.emit(RvInst::Mv { aj: Reg::X11, oc: Reg::X14 });
                            
                            
                            block.emit(RvInst::Mv { aj: Reg::X13, oc: Reg::X20 });
                            
                            block.emit(RvInst::Mv { aj: Reg::X14, oc: Reg::X18 });
                            
                            block.emit(RvInst::Mv { aj: Reg::X15, oc: Reg::X19 });
                            block.emit(RvInst::Ecall);
                            
                            false
                        }

                        
                        0x80..=0x8F => {
                            let ot = self.read_i32() as i64;
                            let target = self.base_addr as i64 + self.offset as i64 + ot;
                            let fc = jru(hjh & 0x0F);
                            block.emit(RvInst::BranchCond { fc, offset: target });
                            let fwe = self.base_addr + self.offset as u64;
                            block.successors.push(target as u64);
                            block.successors.push(fwe);
                            true
                        }

                        
                        0xB6 => {
                            let (rm, reg) = self.decode_modrm(cq, gb);
                            let aj = qs(reg);
                            let oc = qs(rm);
                            block.emit(RvInst::Andi { aj, rs1: oc, imm: 0xFF });
                            false
                        }
                        0xB7 => {
                            let (rm, reg) = self.decode_modrm(cq, gb);
                            let aj = qs(reg);
                            let oc = qs(rm);
                            block.emit(RvInst::Andi { aj, rs1: oc, imm: 0xFFFF });
                            false
                        }

                        
                        0xAF => {
                            let (rm, reg) = self.decode_modrm(cq, gb);
                            let aj = qs(reg);
                            let oc = qs(rm);
                            block.emit(RvInst::Mul { aj, rs1: aj, rs2: oc });
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

            
            0xCD => {
                let mqu = self.read_u8();
                if mqu == 0x80 {
                    
                    block.emit(RvInst::SrcAnnotation {
                        arch: SourceArch::X86_64,
                        addr: bhf,
                        text: String::from("int 0x80 (legacy syscall)"),
                    });
                    block.emit(RvInst::Mv { aj: Reg::X17, oc: Reg::X10 }); 
                    block.emit(RvInst::Mv { aj: Reg::X10, oc: Reg::X13 }); 
                    
                    
                    block.emit(RvInst::Ecall);
                }
                false
            }

            
            _ => {
                self.stats.unsupported_instructions += 1;
                block.emit(RvInst::SrcAnnotation {
                    arch: SourceArch::X86_64,
                    addr: bhf,
                    text: format!("unsupported opcode: 0x{:02X}", opcode),
                });
                block.emit(RvInst::Nop);
                false
            }
        }
    }

    

    fn decode_modrm(&mut self, cq: bool, gb: bool) -> (u8, u8) {
        if self.offset >= self.code.len() {
            return (0, 0);
        }
        let fi = self.code[self.offset];
        self.offset += 1;

        let imz = (fi >> 6) & 3;
        let mut reg = (fi >> 3) & 7;
        let mut rm = fi & 7;

        if gb { reg += 8; }
        if cq { rm += 8; }

        
        if imz != 3 && rm == 4 {
            if self.offset < self.code.len() {
                self.offset += 1; 
            }
        }

        
        match imz {
            0 => {
                if rm == 5 {
                    self.offset += 4; 
                }
            }
            1 => { self.offset += 1; } 
            2 => { self.offset += 4; } 
            _ => {} 
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


fn jru(ft: u8) -> FlagCond {
    match ft {
        0x0 => FlagCond::Ovf,    
        0x1 => FlagCond::NoOvf,  
        0x2 => FlagCond::Ltu,    
        0x3 => FlagCond::Geu,    
        0x4 => FlagCond::Eq,     
        0x5 => FlagCond::Ne,     
        0x6 => FlagCond::Le,     
        0x7 => FlagCond::Gt,     
        0x8 => FlagCond::Neg,    
        0x9 => FlagCond::Pos,    
        0xC => FlagCond::Lt,     
        0xD => FlagCond::Ge,     
        0xE => FlagCond::Le,     
        0xF => FlagCond::Gt,     
        _   => FlagCond::Eq,     
    }
}
