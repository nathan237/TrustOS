








use alloc::vec::Vec;
use super::Uy;


#[repr(C, packed)]
struct Csb {
    signature: [u8; 2],    
    yy: u32,
    awt: u32,
    bbj: u32,
}


#[repr(C, packed)]
struct Csc {
    drp: u32,
    z: i32,
    ac: i32,
    hvf: u16,
    cjh: u16,
    compression: u32,
    gjn: u32,
    zxp: u32,
    zxw: u32,
    yjn: u32,
    yjm: u32,
}


pub fn jdu(path: &str) -> Option<Uy> {
    let f = match crate::vfs::mq(path) {
        Ok(bc) => bc,
        Err(_) => {
            crate::serial_println!("[BMP] Cannot read file: {}", path);
            return None;
        }
    };
    
    ouc(&f)
}


pub fn hqf(f: &[u8]) -> Option<Uy> {
    ouc(f)
}


fn ouc(f: &[u8]) -> Option<Uy> {
    
    if f.len() < 54 {
        crate::serial_println!("[BMP] File too small");
        return None;
    }
    
    
    if f[0] != b'B' || f[1] != b'M' {
        crate::serial_println!("[BMP] Invalid signature");
        return None;
    }
    
    
    let bbj = za(&f[10..14]) as usize;
    let drp = za(&f[14..18]);
    let z = amq(&f[18..22]);
    let ac = amq(&f[22..26]);
    let cjh = alp(&f[28..30]);
    let compression = za(&f[30..34]);
    
    
    if z <= 0 || z > 8192 {
        crate::serial_println!("[BMP] Invalid width: {}", z);
        return None;
    }
    
    let fkl = ac.gp();
    if fkl <= 0 || fkl > 8192 {
        crate::serial_println!("[BMP] Invalid height: {}", ac);
        return None;
    }
    
    
    if compression != 0 && compression != 3 {
        crate::serial_println!("[BMP] Unsupported compression: {}", compression);
        return None;
    }
    
    
    if cjh != 24 && cjh != 32 {
        crate::serial_println!("[BMP] Unsupported bit depth: {}", cjh);
        return None;
    }
    
    let z = z as u32;
    let dia = fkl as u32;
    let lgn = ac < 0;
    
    
    let coy = cjh as usize / 8;
    let ehu = ((z as usize * coy + 3) / 4) * 4;
    
    
    let fqv = (z * dia) as usize;
    let mut hz = Vec::fc(fqv);
    
    
    let amn = &f[bbj..];
    
    for br in 0..dia {
        
        let bxg = if lgn { br } else { dia - 1 - br };
        let mu = bxg as usize * ehu;
        
        for bj in 0..z {
            let bfa = mu + bj as usize * coy;
            
            if bfa + coy <= amn.len() {
                
                let o = amn[bfa] as u32;
                let at = amn[bfa + 1] as u32;
                let m = amn[bfa + 2] as u32;
                let q = if cjh == 32 {
                    amn[bfa + 3] as u32
                } else {
                    255
                };
                
                
                let il = (q << 24) | (m << 16) | (at << 8) | o;
                hz.push(il);
            } else {
                hz.push(0xFF000000); 
            }
        }
    }
    
    crate::serial_println!("[BMP] Loaded {}x{} image ({}-bit)", z, dia, cjh);
    
    Some(Uy {
        z,
        ac: dia,
        hz,
    })
}


fn za(f: &[u8]) -> u32 {
    u32::dj([f[0], f[1], f[2], f[3]])
}


fn amq(f: &[u8]) -> i32 {
    i32::dj([f[0], f[1], f[2], f[3]])
}


fn alp(f: &[u8]) -> u16 {
    u16::dj([f[0], f[1]])
}






pub fn ngy(z: u32, ac: u32, hz: &[u32]) -> Vec<u8> {
    let coy = 3; 
    let ehu = ((z as usize * coy + 3) / 4) * 4;
    let hve = ehu * ac as usize;
    let yy = 54 + hve;
    
    let mut bmp = Vec::fc(yy);
    
    
    bmp.bk(&[b'B', b'M']);
    bmp.bk(&(yy as u32).ho());
    bmp.bk(&[0u8; 4]); 
    bmp.bk(&54u32.ho()); 
    
    
    bmp.bk(&40u32.ho()); 
    bmp.bk(&(z as i32).ho());
    bmp.bk(&(ac as i32).ho()); 
    bmp.bk(&1u16.ho()); 
    bmp.bk(&24u16.ho()); 
    bmp.bk(&0u32.ho()); 
    bmp.bk(&(hve as u32).ho());
    bmp.bk(&[0u8; 16]); 
    
    
    for br in (0..ac).vv() {
        for bj in 0..z {
            let il = hz[(br * z + bj) as usize];
            let m = ((il >> 16) & 0xFF) as u8;
            let at = ((il >> 8) & 0xFF) as u8;
            let o = (il & 0xFF) as u8;
            bmp.push(o);
            bmp.push(at);
            bmp.push(m);
        }
        
        let ob = ehu - (z as usize * 3);
        for _ in 0..ob {
            bmp.push(0);
        }
    }
    
    bmp
}
