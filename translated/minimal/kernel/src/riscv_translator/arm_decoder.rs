






















use alloc::vec::Vec;
use alloc::format;
use alloc::string::String;
use super::ir::*;


fn apr(arm_reg: u8) -> Reg {
    match arm_reg {
        0  => Reg::X10, 
        1  => Reg::X11, 
        2  => Reg::X12, 
        3  => Reg::X13, 
        4  => Reg::X14, 
        5  => Reg::X15, 
        6  => Reg::X16, 
        7  => Reg::X17, 
        8  => Reg::X28, 
        9  => Reg::X5,  
        10 => Reg::X6,  
        11 => Reg::X7,  
        12 => Reg::X29, 
        13 => Reg::X30, 
        14 => Reg::X31, 
        15 => Reg::X9,  
        16 => Reg::X6,  
        17 => Reg::X7,  
        18 => Reg::X4,  
        19 => Reg::X18, 
        20 => Reg::X19, 
        21 => Reg::X20, 
        22 => Reg::X21, 
        23 => Reg::X22, 
        24 => Reg::X23, 
        25 => Reg::X24, 
        26 => Reg::X25, 
        27 => Reg::X26, 
        28 => Reg::X27, 
        29 => Reg::X8,  
        30 => Reg::X1,  
        31 => Reg::X0,  
        _  => Reg::X0,
    }
}


fn fhk(arm_reg: u8, is_sp: bool) -> Reg {
    if arm_reg == 31 {
        if is_sp { Reg::X2 } else { Reg::X0 }
    } else {
        apr(arm_reg)
    }
}


pub struct ArmDecoder {
    
    code: Vec<u8>,
    
    base_addr: u64,
    
    offset: usize,
    
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

    
    pub fn translate_block(&mut self, azv: usize) -> TranslatedBlock {
        self.offset = azv;
        let src_addr = self.base_addr + azv as u64;
        let mut block = TranslatedBlock::new(src_addr, SourceArch::Aarch64);

        let max_instructions = 256;
        let mut count = 0;

        while self.offset + 4 <= self.code.len() && count < max_instructions {
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
                    if eaq + 4 <= self.code.len() && !anc.contains(&succ) {
                        csx.push(eaq);
                    }
                }
            }

            blocks.push(block);
        }

        blocks
    }

    
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

    
    
    fn decode_one(&mut self, block: &mut TranslatedBlock) -> bool {
        let bhf = self.base_addr + self.offset as u64;
        let inst = self.fetch();

        if inst == 0 {
            block.emit(RvInst::Nop);
            return true;
        }

        
        let nnh = (inst >> 25) & 0xF;

        match nnh {
            
            0b1000 | 0b1001 => self.decode_dp_imm(inst, bhf, block),

            
            0b1010 | 0b1011 => return self.decode_branch(inst, bhf, block),

            
            0b0100 | 0b0110 | 0b1100 | 0b1110 => self.decode_ldst(inst, bhf, block),

            
            0b0101 | 0b1101 => self.decode_dp_reg(inst, bhf, block),

            _ => {
                self.stats.unsupported_instructions += 1;
                block.emit(RvInst::SrcAnnotation {
                    arch: SourceArch::Aarch64,
                    addr: bhf,
                    text: format!("unsupported: 0x{:08X}", inst),
                });
                block.emit(RvInst::Nop);
            }
        }

        false
    }

    
    fn decode_dp_imm(&mut self, inst: u32, addr: u64, block: &mut TranslatedBlock) {
        let cnm = (inst >> 23) & 0x7;
        let bvs = (inst >> 31) & 1; 
        let aj = (inst & 0x1F) as u8;
        let boo = ((inst >> 5) & 0x1F) as u8;

        match cnm {
            
            0b010 | 0b011 => {
                let no = ((inst >> 22) & 1) as u8;
                let eqf = ((inst >> 10) & 0xFFF) as i64;
                let imm = if no == 1 { eqf << 12 } else { eqf };
                let erw = (inst >> 30) & 1 == 1;
                let dej = (inst >> 29) & 1 == 1;

                let pj = fhk(aj, !dej);
                let kc = fhk(boo, true);

                if erw {
                    block.emit(RvInst::Addi { aj: pj, rs1: kc, imm: -imm });
                } else {
                    block.emit(RvInst::Addi { aj: pj, rs1: kc, imm });
                }

                if dej {
                    block.emit(RvInst::CmpFlags { rs1: pj, rs2: Reg::X0 });
                }
            }

            
            0b100 | 0b101 => {
                let xc = ((inst >> 21) & 0x3) as u8;
                let alt = ((inst >> 5) & 0xFFFF) as i64;
                let nnn = (inst >> 29) & 0x3;
                let pj = apr(aj);

                let bow = alt << (xc * 16);

                match nnn {
                    0b00 => { 
                        block.emit(RvInst::Li { aj: pj, imm: !bow });
                    }
                    0b10 => { 
                        block.emit(RvInst::Li { aj: pj, imm: bow });
                    }
                    0b11 => { 
                        let mask = !(0xFFFF_i64 << (xc * 16));
                        block.emit(RvInst::Li { aj: Reg::X5, imm: mask });
                        block.emit(RvInst::And { aj: pj, rs1: pj, rs2: Reg::X5 });
                        block.emit(RvInst::Li { aj: Reg::X5, imm: bow });
                        block.emit(RvInst::Or { aj: pj, rs1: pj, rs2: Reg::X5 });
                    }
                    _ => {
                        block.emit(RvInst::Nop);
                    }
                }
            }

            
            0b110 => {
                
                let pj = apr(aj);
                let kc = apr(boo);
                let dtt = (inst >> 29) & 0x3;
                
                let eqg = lci(inst, bvs == 1);

                match dtt {
                    0b00 => block.emit(RvInst::Andi { aj: pj, rs1: kc, imm: eqg }),
                    0b01 => block.emit(RvInst::Ori { aj: pj, rs1: kc, imm: eqg }),
                    0b10 => block.emit(RvInst::Xori { aj: pj, rs1: kc, imm: eqg }),
                    0b11 => { 
                        block.emit(RvInst::Andi { aj: pj, rs1: kc, imm: eqg });
                        block.emit(RvInst::CmpFlags { rs1: pj, rs2: Reg::X0 });
                    }
                    _ => {}
                }
            }

            
            0b000 | 0b001 => {
                let pj = apr(aj);
                let mof = ((inst >> 5) & 0x7FFFF) as i64;
                let mog = ((inst >> 29) & 0x3) as i64;
                let mrw = (inst >> 31) & 1 == 1;
                let mut imm = (mof << 2) | mog;
                
                if imm & (1 << 20) != 0 { imm |= !0x1FFFFF; }
                if mrw { imm <<= 12; }
                let target = addr as i64 + imm;
                block.emit(RvInst::Li { aj: pj, imm: target });
            }

            _ => {
                self.stats.unsupported_instructions += 1;
                block.emit(RvInst::Nop);
            }
        }
    }

    
    fn decode_branch(&mut self, inst: u32, addr: u64, block: &mut TranslatedBlock) -> bool {
        let nni = (inst >> 29) & 0x7;

        match nni {
            
            0b000 | 0b100 => {
                let mut gbx = (inst & 0x03FF_FFFF) as i64;
                if gbx & (1 << 25) != 0 { gbx |= !0x03FF_FFFF; }
                let target = addr as i64 + (gbx << 2);
                let ihr = (inst >> 31) & 1 == 1;

                if ihr {
                    
                    block.emit(RvInst::Jal { aj: Reg::X1, offset: target });
                } else {
                    
                    block.emit(RvInst::Jal { aj: Reg::X0, offset: target });
                }

                block.successors.push(target as u64);
                if ihr {
                    block.successors.push(addr + 4);
                }
                true
            }

            
            0b010 => {
                let mut cky = ((inst >> 5) & 0x7FFFF) as i64;
                if cky & (1 << 18) != 0 { cky |= !0x7FFFF; }
                let target = addr as i64 + (cky << 2);
                let fc = (inst & 0xF) as u8;

                let lwp = jxp(fc);
                block.emit(RvInst::BranchCond { fc: lwp, offset: target });
                block.successors.push(target as u64);
                block.successors.push(addr + 4);
                true
            }

            
            0b001 | 0b101 => {
                let bdm = (inst & 0x1F) as u8;
                let mut cky = ((inst >> 5) & 0x7FFFF) as i64;
                if cky & (1 << 18) != 0 { cky |= !0x7FFFF; }
                let target = addr as i64 + (cky << 2);
                let msa = (inst >> 24) & 1 == 1;
                let acj = apr(bdm);

                if msa {
                    block.emit(RvInst::Bne { rs1: acj, rs2: Reg::X0, offset: target });
                } else {
                    block.emit(RvInst::Beq { rs1: acj, rs2: Reg::X0, offset: target });
                }

                block.successors.push(target as u64);
                block.successors.push(addr + 4);
                true
            }

            
            0b110 => {
                let cnk = (inst >> 21) & 0x7;
                if cnk == 0 {
                    
                    let cnm = (inst >> 21) & 0x7;
                    let das = inst & 0x3;
                    if das == 1 {
                        
                        block.emit(RvInst::SrcAnnotation {
                            arch: SourceArch::Aarch64,
                            addr,
                            text: String::from("SVC #0 (syscall)"),
                        });
                        
                        
                        
                        block.emit(RvInst::Mv { aj: Reg::X17, oc: Reg::X28 });
                        
                        block.emit(RvInst::Ecall);
                        
                        return false;
                    } else if das == 0 && cnm == 0 {
                        
                        block.emit(RvInst::Ecall);
                        return false;
                    }
                }

                
                if (inst & 0xFFFFFC1F) == 0xD65F0000 {
                    block.emit(RvInst::Ret);
                    return true;
                }

                
                if (inst & 0xFFFFFC00) == 0xD61F0000 {
                    let boo = ((inst >> 5) & 0x1F) as u8;
                    let kc = apr(boo);
                    block.emit(RvInst::Jalr { aj: Reg::X0, rs1: kc, offset: 0 });
                    return true;
                }

                
                if (inst & 0xFFFFFC00) == 0xD63F0000 {
                    let boo = ((inst >> 5) & 0x1F) as u8;
                    let kc = apr(boo);
                    block.emit(RvInst::Jalr { aj: Reg::X1, rs1: kc, offset: 0 });
                    block.successors.push(addr + 4);
                    return true;
                }

                
                if inst == 0xD503201F { 
                    block.emit(RvInst::Nop);
                    return false;
                }
                if inst == 0xD503207F { 
                    block.emit(RvInst::Nop);
                    return false;
                }

                self.stats.unsupported_instructions += 1;
                block.emit(RvInst::Nop);
                false
            }

            
            0b011 | 0b111 => {
                let bdm = (inst & 0x1F) as u8;
                let bf = ((inst >> 19) & 0x1F) as u8 | (((inst >> 31) & 1) as u8) << 5;
                let mut gbw = ((inst >> 5) & 0x3FFF) as i64;
                if gbw & (1 << 13) != 0 { gbw |= !0x3FFF; }
                let target = addr as i64 + (gbw << 2);
                let mtv = (inst >> 24) & 1 == 1;

                let acj = apr(bdm);
                
                block.emit(RvInst::Li { aj: Reg::X5, imm: 1 << bf });
                block.emit(RvInst::And { aj: Reg::X5, rs1: acj, rs2: Reg::X5 });

                if mtv {
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

    
    fn decode_ldst(&mut self, inst: u32, addr: u64, block: &mut TranslatedBlock) {
        let size = (inst >> 30) & 0x3;
        let cnm = (inst >> 22) & 0x3;
        let boo = ((inst >> 5) & 0x1F) as u8;
        let bdm = (inst & 0x1F) as u8;

        let acj = apr(bdm);
        let kc = fhk(boo, true);

        
        if (inst & 0x3B000000) == 0x39000000 {
            let eqf = ((inst >> 10) & 0xFFF) as i64;
            let scale = size as i64;
            let offset = eqf << scale;
            let czz = (cnm & 1) == 1;

            if czz {
                match size {
                    0 => block.emit(RvInst::Lbu { aj: acj, rs1: kc, offset }),
                    1 => block.emit(RvInst::Lhu { aj: acj, rs1: kc, offset }),
                    2 => block.emit(RvInst::Lwu { aj: acj, rs1: kc, offset }),
                    3 => block.emit(RvInst::Ld { aj: acj, rs1: kc, offset }),
                    _ => {}
                }
            } else {
                match size {
                    0 => block.emit(RvInst::Sb { rs2: acj, rs1: kc, offset }),
                    1 => block.emit(RvInst::Sh { rs2: acj, rs1: kc, offset }),
                    2 => block.emit(RvInst::Sw { rs2: acj, rs1: kc, offset }),
                    3 => block.emit(RvInst::Sd { rs2: acj, rs1: kc, offset }),
                    _ => {}
                }
            }
            return;
        }

        
        if (inst & 0x3B200C00) == 0x38000000 || (inst & 0x3B200C00) == 0x38000400 {
            let mut bci = ((inst >> 12) & 0x1FF) as i64;
            if bci & (1 << 8) != 0 { bci |= !0x1FF; }
            let bnb = (inst >> 11) & 1 == 1;
            let czz = (cnm & 1) == 1;

            if bnb {
                
                block.emit(RvInst::Addi { aj: kc, rs1: kc, imm: bci });
            }

            if czz {
                match size {
                    0 => block.emit(RvInst::Lbu { aj: acj, rs1: kc, offset: if bnb { 0 } else { bci } }),
                    1 => block.emit(RvInst::Lhu { aj: acj, rs1: kc, offset: if bnb { 0 } else { bci } }),
                    2 => block.emit(RvInst::Lwu { aj: acj, rs1: kc, offset: if bnb { 0 } else { bci } }),
                    3 => block.emit(RvInst::Ld { aj: acj, rs1: kc, offset: if bnb { 0 } else { bci } }),
                    _ => {}
                }
            } else {
                match size {
                    0 => block.emit(RvInst::Sb { rs2: acj, rs1: kc, offset: if bnb { 0 } else { bci } }),
                    1 => block.emit(RvInst::Sh { rs2: acj, rs1: kc, offset: if bnb { 0 } else { bci } }),
                    2 => block.emit(RvInst::Sw { rs2: acj, rs1: kc, offset: if bnb { 0 } else { bci } }),
                    3 => block.emit(RvInst::Sd { rs2: acj, rs1: kc, offset: if bnb { 0 } else { bci } }),
                    _ => {}
                }
            }

            if !bnb {
                
                block.emit(RvInst::Addi { aj: kc, rs1: kc, imm: bci });
            }
            return;
        }

        
        if (inst & 0x3E000000) == 0x28000000 || (inst & 0x3E000000) == 0x2C000000 {
            let oiu = ((inst >> 10) & 0x1F) as u8;
            let mut gby = ((inst >> 15) & 0x7F) as i64;
            if gby & (1 << 6) != 0 { gby |= !0x7F; }
            let scale = if (inst >> 31) & 1 == 1 { 3 } else { 2 };
            let offset = gby << scale;
            let czz = (inst >> 22) & 1 == 1;

            let jby = apr(oiu);

            if czz {
                block.emit(RvInst::Ld { aj: acj, rs1: kc, offset });
                block.emit(RvInst::Ld { aj: jby, rs1: kc, offset: offset + 8 });
            } else {
                block.emit(RvInst::Sd { rs2: acj, rs1: kc, offset });
                block.emit(RvInst::Sd { rs2: jby, rs1: kc, offset: offset + 8 });
            }
            return;
        }

        
        self.stats.unsupported_instructions += 1;
        block.emit(RvInst::SrcAnnotation {
            arch: SourceArch::Aarch64,
            addr,
            text: format!("unsupported ldst: 0x{:08X}", inst),
        });
        block.emit(RvInst::Nop);
    }

    
    fn decode_dp_reg(&mut self, inst: u32, addr: u64, block: &mut TranslatedBlock) {
        let aj = (inst & 0x1F) as u8;
        let boo = ((inst >> 5) & 0x1F) as u8;
        let rm = ((inst >> 16) & 0x1F) as u8;
        let cnm = (inst >> 29) & 0x7;

        let pj = apr(aj);
        let kc = apr(boo);
        let zk = apr(rm);

        
        if (inst & 0x1F000000) == 0x0A000000 {
            let dtt = (inst >> 29) & 0x3;
            let ae = (inst >> 21) & 1;
            let dej = dtt == 0b11;

            match (dtt, ae) {
                (0b00, 0) => block.emit(RvInst::And { aj: pj, rs1: kc, rs2: zk }),
                (0b01, 0) => block.emit(RvInst::Or { aj: pj, rs1: kc, rs2: zk }),
                (0b10, 0) => block.emit(RvInst::Xor { aj: pj, rs1: kc, rs2: zk }),
                (0b11, 0) => { 
                    block.emit(RvInst::And { aj: pj, rs1: kc, rs2: zk });
                }
                
                (_, 1) => {
                    block.emit(RvInst::Xori { aj: Reg::X5, rs1: zk, imm: -1 });
                    match dtt {
                        0b00 => block.emit(RvInst::And { aj: pj, rs1: kc, rs2: Reg::X5 }),
                        0b01 => block.emit(RvInst::Or { aj: pj, rs1: kc, rs2: Reg::X5 }),
                        0b10 => block.emit(RvInst::Xor { aj: pj, rs1: kc, rs2: Reg::X5 }),
                        0b11 => block.emit(RvInst::And { aj: pj, rs1: kc, rs2: Reg::X5 }),
                        _ => {}
                    }
                }
                _ => { block.emit(RvInst::Nop); }
            }

            if dej {
                block.emit(RvInst::CmpFlags { rs1: pj, rs2: Reg::X0 });
            }
            return;
        }

        
        if (inst & 0x1F000000) == 0x0B000000 {
            let erw = (inst >> 30) & 1 == 1;
            let dej = (inst >> 29) & 1 == 1;

            
            let ort = ((inst >> 22) & 0x3) as u8;
            let fau = ((inst >> 10) & 0x3F) as u8;

            if fau > 0 {
                match ort {
                    0 => block.emit(RvInst::Slli { aj: Reg::X5, rs1: zk, acn: fau }),
                    1 => block.emit(RvInst::Srli { aj: Reg::X5, rs1: zk, acn: fau }),
                    2 => block.emit(RvInst::Srai { aj: Reg::X5, rs1: zk, acn: fau }),
                    _ => block.emit(RvInst::Mv { aj: Reg::X5, oc: zk }),
                }
                if erw {
                    block.emit(RvInst::Sub { aj: pj, rs1: kc, rs2: Reg::X5 });
                } else {
                    block.emit(RvInst::Add { aj: pj, rs1: kc, rs2: Reg::X5 });
                }
            } else {
                if erw {
                    block.emit(RvInst::Sub { aj: pj, rs1: kc, rs2: zk });
                } else {
                    block.emit(RvInst::Add { aj: pj, rs1: kc, rs2: zk });
                }
            }

            if dej {
                block.emit(RvInst::CmpFlags { rs1: pj, rs2: Reg::X0 });
            }
            return;
        }

        
        if (inst & 0x7FE00000) == 0x1B000000 {
            let dxe = ((inst >> 10) & 0x1F) as u8;
            let mte = (inst >> 15) & 1 == 1;

            if dxe == 31 {
                
                block.emit(RvInst::Mul { aj: pj, rs1: kc, rs2: zk });
            } else {
                let jbx = apr(dxe);
                block.emit(RvInst::Mul { aj: Reg::X5, rs1: kc, rs2: zk });
                if mte {
                    block.emit(RvInst::Sub { aj: pj, rs1: jbx, rs2: Reg::X5 });
                } else {
                    block.emit(RvInst::Add { aj: pj, rs1: jbx, rs2: Reg::X5 });
                }
            }
            return;
        }

        
        if (inst & 0x7FE0FC00) == 0x1AC00800 {
            let mty = (inst >> 10) & 1 == 0;
            if mty {
                block.emit(RvInst::Divu { aj: pj, rs1: kc, rs2: zk });
            } else {
                block.emit(RvInst::Div { aj: pj, rs1: kc, rs2: zk });
            }
            return;
        }

        
        if (inst & 0x7FE0F000) == 0x1AC02000 {
            let ors = ((inst >> 10) & 0x3) as u8;
            match ors {
                0 => block.emit(RvInst::Sll { aj: pj, rs1: kc, rs2: zk }),
                1 => block.emit(RvInst::Srl { aj: pj, rs1: kc, rs2: zk }),
                2 => block.emit(RvInst::Sra { aj: pj, rs1: kc, rs2: zk }),
                
                3 => {
                    block.emit(RvInst::Srl { aj: Reg::X5, rs1: kc, rs2: zk });
                    block.emit(RvInst::Li { aj: Reg::X6, imm: 64 });
                    block.emit(RvInst::Sub { aj: Reg::X6, rs1: Reg::X6, rs2: zk });
                    block.emit(RvInst::Sll { aj: Reg::X6, rs1: kc, rs2: Reg::X6 });
                    block.emit(RvInst::Or { aj: pj, rs1: Reg::X5, rs2: Reg::X6 });
                }
                _ => {}
            }
            return;
        }

        
        self.stats.unsupported_instructions += 1;
        block.emit(RvInst::SrcAnnotation {
            arch: SourceArch::Aarch64,
            addr,
            text: format!("unsupported dp_reg: 0x{:08X}", inst),
        });
        block.emit(RvInst::Nop);
    }
}


fn jxp(ft: u8) -> FlagCond {
    match ft {
        0x0 => FlagCond::Eq,     
        0x1 => FlagCond::Ne,     
        0x2 => FlagCond::Geu,    
        0x3 => FlagCond::Ltu,    
        0x4 => FlagCond::Neg,    
        0x5 => FlagCond::Pos,    
        0x6 => FlagCond::Ovf,    
        0x7 => FlagCond::NoOvf,  
        0x8 => FlagCond::Gt,     
        0x9 => FlagCond::Le,     
        0xA => FlagCond::Ge,     
        0xB => FlagCond::Lt,     
        0xC => FlagCond::Gt,     
        0xD => FlagCond::Le,     
        0xE => FlagCond::Eq,     
        _   => FlagCond::Eq,
    }
}


fn lci(inst: u32, _is_64: bool) -> i64 {
    let ae = (inst >> 22) & 1;
    let moh = ((inst >> 16) & 0x3F) as u32;
    let ifw = ((inst >> 10) & 0x3F) as u32;

    
    let len = if ae == 1 { 6 } else {
        
        let evh = !ifw & 0x3F;
        if evh & 0x20 != 0 { 5 }
        else if evh & 0x10 != 0 { 4 }
        else if evh & 0x08 != 0 { 3 }
        else if evh & 0x04 != 0 { 2 }
        else { 1 }
    };

    let size = 1u64 << len;
    let mask = size - 1;
    let j = (ifw & mask as u32) as u64;
    let r = (moh & mask as u32) as u64;

    let mut eee: u64 = (1u64 << (j + 1)) - 1;
    
    if r > 0 {
        eee = (eee >> r) | (eee << (size - r));
        eee &= (1u64 << size) - 1;
    }

    
    let mut result = eee;
    let mut fpk = size;
    while fpk < 64 {
        result |= result << fpk;
        fpk *= 2;
    }

    result as i64
}
