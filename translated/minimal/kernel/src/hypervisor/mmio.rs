

























#[derive(Debug, Clone)]
pub struct Eh {
    
    pub rm: bool,
    
    pub aqc: u8,
    
    
    pub nw: Option<u8>,
    
    pub cag: Option<u64>,
    
    pub ake: usize,
}


#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum Cyd {
    J = 0, Fe = 1, Axm = 2, Ckf = 3,
    Qo = 4, Qn = 5, Brf = 6, Bql = 7,
    Alo = 8, Alp = 9, Alj = 10, Alk = 11,
    All = 12, Alm = 13, Aln = 14, Aec = 15,
}









pub fn cpw(hod: &[u8], nba: usize, ykx: bool) -> Option<Eh> {
    if nba == 0 || hod.is_empty() {
        return None;
    }
    
    let bf = &hod[..nba.v(hod.len())];
    let mut u: usize = 0;
    
    
    let mut kf = false;
    let mut ako = false;   
    let mut nx = false;   
    let mut ic = false;   
    let mut fkd = false;  
    let mut bus = false;  
    let mut qcd = false;  
    let mut msi = false;  
    let mut msj = false;  
    
    
    while u < bf.len() {
        match bf[u] {
            0x66 => { fkd = true; u += 1; }
            0x67 => { bus = true; u += 1; }
            0xF0 => { qcd = true; u += 1; }
            0xF2 => { msi = true; u += 1; }
            0xF3 => { msj = true; u += 1; }
            
            0x2E | 0x3E | 0x26 | 0x36 | 0x64 | 0x65 => { u += 1; }
            
            o @ 0x40..=0x4F => {
                kf = true;
                ako = (o & 0x08) != 0;
                nx = (o & 0x04) != 0;
                ic = (o & 0x01) != 0;
                u += 1;
            }
            _ => break, 
        }
    }
    
    if u >= bf.len() {
        return None;
    }
    
    
    
    let aqc: u8 = if ako {
        8
    } else if fkd {
        2
    } else {
        4
    };
    
    let opcode = bf[u];
    u += 1;
    
    match opcode {
        
        0x89 => {
            if u >= bf.len() { return None; }
            let ms = bf[u];
            u += 1;
            let (alq, ake) = dpt(ms, nx, ic, bus, bf, u)?;
            Some(Eh {
                rm: true,
                aqc,
                nw: Some(alq),
                cag: None,
                ake,
            })
        }
        
        
        0x88 => {
            if u >= bf.len() { return None; }
            let ms = bf[u];
            u += 1;
            let (alq, ake) = dpt(ms, nx, ic, bus, bf, u)?;
            Some(Eh {
                rm: true,
                aqc: 1,
                nw: Some(alq),
                cag: None,
                ake,
            })
        }
        
        
        0x8B => {
            if u >= bf.len() { return None; }
            let ms = bf[u];
            u += 1;
            let (alq, ake) = dpt(ms, nx, ic, bus, bf, u)?;
            Some(Eh {
                rm: false,
                aqc,
                nw: Some(alq),
                cag: None,
                ake,
            })
        }
        
        
        0x8A => {
            if u >= bf.len() { return None; }
            let ms = bf[u];
            u += 1;
            let (alq, ake) = dpt(ms, nx, ic, bus, bf, u)?;
            Some(Eh {
                rm: false,
                aqc: 1,
                nw: Some(alq),
                cag: None,
                ake,
            })
        }
        
        
        0xC7 => {
            if u >= bf.len() { return None; }
            let ms = bf[u];
            let fsn = (ms >> 3) & 7;
            if fsn != 0 { return None; } 
            u += 1;
            let (_, gzu) = dpt(ms, false, ic, bus, bf, u)?;
            
            let esj = gzu; 
            let ldo = if ako { 4 } else if fkd { 2 } else { 4 }; 
            if esj + ldo > bf.len() { return None; }
            let gf = match ldo {
                2 => u16::dj([bf[esj], bf[esj + 1]]) as u64,
                4 => {
                    let p = u32::dj([
                        bf[esj], bf[esj + 1],
                        bf[esj + 2], bf[esj + 3],
                    ]);
                    if ako {
                        
                        p as i32 as i64 as u64
                    } else {
                        p as u64
                    }
                }
                _ => return None,
            };
            Some(Eh {
                rm: true,
                aqc,
                nw: None,
                cag: Some(gf),
                ake: esj + ldo,
            })
        }
        
        
        0xC6 => {
            if u >= bf.len() { return None; }
            let ms = bf[u];
            let fsn = (ms >> 3) & 7;
            if fsn != 0 { return None; }
            u += 1;
            let (_, gzu) = dpt(ms, false, ic, bus, bf, u)?;
            if gzu >= bf.len() { return None; }
            let gf = bf[gzu] as u64;
            Some(Eh {
                rm: true,
                aqc: 1,
                nw: None,
                cag: Some(gf),
                ake: gzu + 1,
            })
        }
        
        
        0x0F => {
            if u >= bf.len() { return None; }
            let uyu = bf[u];
            u += 1;
            
            match uyu {
                
                0xB6 => {
                    if u >= bf.len() { return None; }
                    let ms = bf[u];
                    u += 1;
                    let (alq, ake) = dpt(ms, nx, ic, bus, bf, u)?;
                    Some(Eh {
                        rm: false,
                        aqc: 1, 
                        nw: Some(alq),
                        cag: None,
                        ake,
                    })
                }
                
                0xB7 => {
                    if u >= bf.len() { return None; }
                    let ms = bf[u];
                    u += 1;
                    let (alq, ake) = dpt(ms, nx, ic, bus, bf, u)?;
                    Some(Eh {
                        rm: false,
                        aqc: 2,
                        nw: Some(alq),
                        cag: None,
                        ake,
                    })
                }
                
                0xBE => {
                    if u >= bf.len() { return None; }
                    let ms = bf[u];
                    u += 1;
                    let (alq, ake) = dpt(ms, nx, ic, bus, bf, u)?;
                    Some(Eh {
                        rm: false,
                        aqc: 1,
                        nw: Some(alq),
                        cag: None,
                        ake,
                    })
                }
                
                0xBF => {
                    if u >= bf.len() { return None; }
                    let ms = bf[u];
                    u += 1;
                    let (alq, ake) = dpt(ms, nx, ic, bus, bf, u)?;
                    Some(Eh {
                        rm: false,
                        aqc: 2,
                        nw: Some(alq),
                        cag: None,
                        ake,
                    })
                }
                _ => None, 
            }
        }
        
        
        0xA1 => {
            
            let fco = if bus { 4 } else { 8 }; 
            let ake = u + fco;
            Some(Eh {
                rm: false,
                aqc,
                nw: Some(0), 
                cag: None,
                ake,
            })
        }
        
        
        0xA3 => {
            let fco = if bus { 4 } else { 8 };
            let ake = u + fco;
            Some(Eh {
                rm: true,
                aqc,
                nw: Some(0), 
                cag: None,
                ake,
            })
        }
        
        
        0xA0 => {
            let fco = if bus { 4 } else { 8 };
            let ake = u + fco;
            Some(Eh {
                rm: false,
                aqc: 1,
                nw: Some(0), 
                cag: None,
                ake,
            })
        }
        
        
        0xA2 => {
            let fco = if bus { 4 } else { 8 };
            let ake = u + fco;
            Some(Eh {
                rm: true,
                aqc: 1,
                nw: Some(0), 
                cag: None,
                ake,
            })
        }
        
        _ => None, 
    }
}





fn dpt(
    ms: u8,
    nx: bool,
    yci: bool,
    xzp: bool,
    bf: &[u8],
    vka: usize,
) -> Option<(u8, usize)> {
    let hrq = (ms >> 6) & 3;
    let fsn = (ms >> 3) & 7;
    let ext = ms & 7;
    
    
    let alq = fsn | (if nx { 8 } else { 0 });
    
    let mut u = vka;
    
    
    match hrq {
        0b00 => {
            
            if ext == 4 {
                
                u += 1; 
                if u > bf.len() { return None; }
                let iam = bf[u - 1];
                let ar = iam & 7;
                if ar == 5 {
                    u += 4; 
                }
            } else if ext == 5 {
                
                u += 4;
            }
        }
        0b01 => {
            
            if ext == 4 {
                u += 1; 
            }
            u += 1; 
        }
        0b10 => {
            
            if ext == 4 {
                u += 1; 
            }
            u += 4; 
        }
        0b11 => {
            
        }
        _ => unreachable!(),
    }
    
    if u > bf.len() {
        return None;
    }
    
    Some((alq, u))
}


pub fn paf(regs: &super::svm_vm::SvmGuestRegs, w: u8) -> u64 {
    match w {
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


pub fn pzy(regs: &mut super::svm_vm::SvmGuestRegs, w: u8, bn: u64) {
    match w {
        0 => regs.rax = bn,
        1 => regs.rcx = bn,
        2 => regs.rdx = bn,
        3 => regs.rbx = bn,
        4 => regs.rsp = bn,
        5 => regs.rbp = bn,
        6 => regs.rsi = bn,
        7 => regs.rdi = bn,
        8 => regs.r8 = bn,
        9 => regs.r9 = bn,
        10 => regs.r10 = bn,
        11 => regs.r11 = bn,
        12 => regs.r12 = bn,
        13 => regs.r13 = bn,
        14 => regs.r14 = bn,
        15 => regs.r15 = bn,
        _ => {}
    }
}


pub fn old(bn: u64, aw: u8) -> u64 {
    match aw {
        1 => bn & 0xFF,
        2 => bn & 0xFFFF,
        4 => bn & 0xFFFF_FFFF,
        8 => bn,
        _ => bn & 0xFFFF_FFFF,
    }
}





#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zrm() {
        
        let bf = [0x89, 0x07];
        let bc = cpw(&bf, 2, true).unwrap();
        assert!(bc.rm);
        assert_eq!(bc.aqc, 4);
        assert_eq!(bc.nw, Some(0)); 
        assert_eq!(bc.ake, 2);
    }

    #[test]
    fn zrl() {
        
        let bf = [0x8B, 0x0F];
        let bc = cpw(&bf, 2, true).unwrap();
        assert!(!bc.rm);
        assert_eq!(bc.aqc, 4);
        assert_eq!(bc.nw, Some(1)); 
        assert_eq!(bc.ake, 2);
    }

    #[test]
    fn zro() {
        
        let bf = [0x48, 0x89, 0x07];
        let bc = cpw(&bf, 3, true).unwrap();
        assert!(bc.rm);
        assert_eq!(bc.aqc, 8);
        assert_eq!(bc.nw, Some(0)); 
        assert_eq!(bc.ake, 3);
    }

    #[test]
    fn zrn() {
        
        let bf = [0x0F, 0xB6, 0x07];
        let bc = cpw(&bf, 3, true).unwrap();
        assert!(!bc.rm);
        assert_eq!(bc.aqc, 1);
        assert_eq!(bc.nw, Some(0)); 
    }

    #[test]  
    fn zrk() {
        
        let bf = [0x8B, 0x87, 0x20, 0x03, 0x00, 0x00];
        let bc = cpw(&bf, 6, true).unwrap();
        assert!(!bc.rm);
        assert_eq!(bc.aqc, 4);
        assert_eq!(bc.nw, Some(0)); 
        assert_eq!(bc.ake, 6);
    }
}
