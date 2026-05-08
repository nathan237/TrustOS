

























#[derive(Debug, Clone)]
pub struct Bt {
    
    pub is_write: bool,
    
    pub operand_size: u8,
    
    
    pub register: Option<u8>,
    
    pub immediate: Option<u64>,
    
    pub insn_len: usize,
}


#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum Aww {
    Rax = 0, Rcx = 1, Rdx = 2, Rbx = 3,
    Rsp = 4, Rbp = 5, Rsi = 6, Rdi = 7,
    R8 = 8, R9 = 9, R10 = 10, R11 = 11,
    R12 = 12, R13 = 13, R14 = 14, R15 = 15,
}









pub fn awu(insn_bytes: &[u8], bytes_fetched: usize, cs_long: bool) -> Option<Bt> {
    if bytes_fetched == 0 || insn_bytes.is_empty() {
        return None;
    }
    
    let bytes = &insn_bytes[..bytes_fetched.min(insn_bytes.len())];
    let mut pos: usize = 0;
    
    
    let mut dv = false;
    let mut rex_w = false;   
    let mut gb = false;   
    let mut cq = false;   
    let mut ckh = false;  
    let mut alr = false;  
    let mut jsn = false;  
    let mut hdi = false;  
    let mut hdj = false;  
    
    
    while pos < bytes.len() {
        match bytes[pos] {
            0x66 => { ckh = true; pos += 1; }
            0x67 => { alr = true; pos += 1; }
            0xF0 => { jsn = true; pos += 1; }
            0xF2 => { hdi = true; pos += 1; }
            0xF3 => { hdj = true; pos += 1; }
            
            0x2E | 0x3E | 0x26 | 0x36 | 0x64 | 0x65 => { pos += 1; }
            
            b @ 0x40..=0x4F => {
                dv = true;
                rex_w = (b & 0x08) != 0;
                gb = (b & 0x04) != 0;
                cq = (b & 0x01) != 0;
                pos += 1;
            }
            _ => break, 
        }
    }
    
    if pos >= bytes.len() {
        return None;
    }
    
    
    
    let operand_size: u8 = if rex_w {
        8
    } else if ckh {
        2
    } else {
        4
    };
    
    let opcode = bytes[pos];
    pos += 1;
    
    match opcode {
        
        0x89 => {
            if pos >= bytes.len() { return None; }
            let fi = bytes[pos];
            pos += 1;
            let (tb, insn_len) = blq(fi, gb, cq, alr, bytes, pos)?;
            Some(Bt {
                is_write: true,
                operand_size,
                register: Some(tb),
                immediate: None,
                insn_len,
            })
        }
        
        
        0x88 => {
            if pos >= bytes.len() { return None; }
            let fi = bytes[pos];
            pos += 1;
            let (tb, insn_len) = blq(fi, gb, cq, alr, bytes, pos)?;
            Some(Bt {
                is_write: true,
                operand_size: 1,
                register: Some(tb),
                immediate: None,
                insn_len,
            })
        }
        
        
        0x8B => {
            if pos >= bytes.len() { return None; }
            let fi = bytes[pos];
            pos += 1;
            let (tb, insn_len) = blq(fi, gb, cq, alr, bytes, pos)?;
            Some(Bt {
                is_write: false,
                operand_size,
                register: Some(tb),
                immediate: None,
                insn_len,
            })
        }
        
        
        0x8A => {
            if pos >= bytes.len() { return None; }
            let fi = bytes[pos];
            pos += 1;
            let (tb, insn_len) = blq(fi, gb, cq, alr, bytes, pos)?;
            Some(Bt {
                is_write: false,
                operand_size: 1,
                register: Some(tb),
                immediate: None,
                insn_len,
            })
        }
        
        
        0xC7 => {
            if pos >= bytes.len() { return None; }
            let fi = bytes[pos];
            let cpg = (fi >> 3) & 7;
            if cpg != 0 { return None; } 
            pos += 1;
            let (_, base_insn_len) = blq(fi, false, cq, alr, bytes, pos)?;
            
            let car = base_insn_len; 
            let gbz = if rex_w { 4 } else if ckh { 2 } else { 4 }; 
            if car + gbz > bytes.len() { return None; }
            let imm = match gbz {
                2 => u16::from_le_bytes([bytes[car], bytes[car + 1]]) as u64,
                4 => {
                    let v = u32::from_le_bytes([
                        bytes[car], bytes[car + 1],
                        bytes[car + 2], bytes[car + 3],
                    ]);
                    if rex_w {
                        
                        v as i32 as i64 as u64
                    } else {
                        v as u64
                    }
                }
                _ => return None,
            };
            Some(Bt {
                is_write: true,
                operand_size,
                register: None,
                immediate: Some(imm),
                insn_len: car + gbz,
            })
        }
        
        
        0xC6 => {
            if pos >= bytes.len() { return None; }
            let fi = bytes[pos];
            let cpg = (fi >> 3) & 7;
            if cpg != 0 { return None; }
            pos += 1;
            let (_, base_insn_len) = blq(fi, false, cq, alr, bytes, pos)?;
            if base_insn_len >= bytes.len() { return None; }
            let imm = bytes[base_insn_len] as u64;
            Some(Bt {
                is_write: true,
                operand_size: 1,
                register: None,
                immediate: Some(imm),
                insn_len: base_insn_len + 1,
            })
        }
        
        
        0x0F => {
            if pos >= bytes.len() { return None; }
            let nno = bytes[pos];
            pos += 1;
            
            match nno {
                
                0xB6 => {
                    if pos >= bytes.len() { return None; }
                    let fi = bytes[pos];
                    pos += 1;
                    let (tb, insn_len) = blq(fi, gb, cq, alr, bytes, pos)?;
                    Some(Bt {
                        is_write: false,
                        operand_size: 1, 
                        register: Some(tb),
                        immediate: None,
                        insn_len,
                    })
                }
                
                0xB7 => {
                    if pos >= bytes.len() { return None; }
                    let fi = bytes[pos];
                    pos += 1;
                    let (tb, insn_len) = blq(fi, gb, cq, alr, bytes, pos)?;
                    Some(Bt {
                        is_write: false,
                        operand_size: 2,
                        register: Some(tb),
                        immediate: None,
                        insn_len,
                    })
                }
                
                0xBE => {
                    if pos >= bytes.len() { return None; }
                    let fi = bytes[pos];
                    pos += 1;
                    let (tb, insn_len) = blq(fi, gb, cq, alr, bytes, pos)?;
                    Some(Bt {
                        is_write: false,
                        operand_size: 1,
                        register: Some(tb),
                        immediate: None,
                        insn_len,
                    })
                }
                
                0xBF => {
                    if pos >= bytes.len() { return None; }
                    let fi = bytes[pos];
                    pos += 1;
                    let (tb, insn_len) = blq(fi, gb, cq, alr, bytes, pos)?;
                    Some(Bt {
                        is_write: false,
                        operand_size: 2,
                        register: Some(tb),
                        immediate: None,
                        insn_len,
                    })
                }
                _ => None, 
            }
        }
        
        
        0xA1 => {
            
            let cft = if alr { 4 } else { 8 }; 
            let insn_len = pos + cft;
            Some(Bt {
                is_write: false,
                operand_size,
                register: Some(0), 
                immediate: None,
                insn_len,
            })
        }
        
        
        0xA3 => {
            let cft = if alr { 4 } else { 8 };
            let insn_len = pos + cft;
            Some(Bt {
                is_write: true,
                operand_size,
                register: Some(0), 
                immediate: None,
                insn_len,
            })
        }
        
        
        0xA0 => {
            let cft = if alr { 4 } else { 8 };
            let insn_len = pos + cft;
            Some(Bt {
                is_write: false,
                operand_size: 1,
                register: Some(0), 
                immediate: None,
                insn_len,
            })
        }
        
        
        0xA2 => {
            let cft = if alr { 4 } else { 8 };
            let insn_len = pos + cft;
            Some(Bt {
                is_write: true,
                operand_size: 1,
                register: Some(0), 
                immediate: None,
                insn_len,
            })
        }
        
        _ => None, 
    }
}





fn blq(
    fi: u8,
    gb: bool,
    _rex_b: bool,
    _has_67: bool,
    bytes: &[u8],
    pos_after_modrm: usize,
) -> Option<(u8, usize)> {
    let dul = (fi >> 6) & 3;
    let cpg = (fi >> 3) & 7;
    let cdj = fi & 7;
    
    
    let tb = cpg | (if gb { 8 } else { 0 });
    
    let mut pos = pos_after_modrm;
    
    
    match dul {
        0b00 => {
            
            if cdj == 4 {
                
                pos += 1; 
                if pos > bytes.len() { return None; }
                let dzk = bytes[pos - 1];
                let base = dzk & 7;
                if base == 5 {
                    pos += 4; 
                }
            } else if cdj == 5 {
                
                pos += 4;
            }
        }
        0b01 => {
            
            if cdj == 4 {
                pos += 1; 
            }
            pos += 1; 
        }
        0b10 => {
            
            if cdj == 4 {
                pos += 1; 
            }
            pos += 4; 
        }
        0b11 => {
            
        }
        _ => unreachable!(),
    }
    
    if pos > bytes.len() {
        return None;
    }
    
    Some((tb, pos))
}


pub fn iyl(regs: &super::svm_vm::SvmGuestRegs, idx: u8) -> u64 {
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


pub fn jro(regs: &mut super::svm_vm::SvmGuestRegs, idx: u8, value: u64) {
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


pub fn ilw(value: u64, size: u8) -> u64 {
    match size {
        1 => value & 0xFF,
        2 => value & 0xFFFF,
        4 => value & 0xFFFF_FFFF,
        8 => value,
        _ => value & 0xFFFF_FFFF,
    }
}





#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn qzg() {
        
        let bytes = [0x89, 0x07];
        let d = awu(&bytes, 2, true).unwrap();
        assert!(d.is_write);
        assert_eq!(d.operand_size, 4);
        assert_eq!(d.register, Some(0)); 
        assert_eq!(d.insn_len, 2);
    }

    #[test]
    fn qzf() {
        
        let bytes = [0x8B, 0x0F];
        let d = awu(&bytes, 2, true).unwrap();
        assert!(!d.is_write);
        assert_eq!(d.operand_size, 4);
        assert_eq!(d.register, Some(1)); 
        assert_eq!(d.insn_len, 2);
    }

    #[test]
    fn qzi() {
        
        let bytes = [0x48, 0x89, 0x07];
        let d = awu(&bytes, 3, true).unwrap();
        assert!(d.is_write);
        assert_eq!(d.operand_size, 8);
        assert_eq!(d.register, Some(0)); 
        assert_eq!(d.insn_len, 3);
    }

    #[test]
    fn qzh() {
        
        let bytes = [0x0F, 0xB6, 0x07];
        let d = awu(&bytes, 3, true).unwrap();
        assert!(!d.is_write);
        assert_eq!(d.operand_size, 1);
        assert_eq!(d.register, Some(0)); 
    }

    #[test]  
    fn qze() {
        
        let bytes = [0x8B, 0x87, 0x20, 0x03, 0x00, 0x00];
        let d = awu(&bytes, 6, true).unwrap();
        assert!(!d.is_write);
        assert_eq!(d.operand_size, 4);
        assert_eq!(d.register, Some(0)); 
        assert_eq!(d.insn_len, 6);
    }
}
