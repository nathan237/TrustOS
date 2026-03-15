



use alloc::vec::Vec;
use super::{TlsSession, TlsState, TlsError, ContentType, HandshakeType};
use super::crypto::{ijd, muf};
use super::handshake;


pub const DTL_: usize = 16384 + 256;


pub fn jkd(he: &mut TlsSession, f: &[u8]) -> Result<Option<Vec<u8>>, TlsError> {
    if f.len() < 5 {
        return Err(TlsError::Fs);
    }
    
    let ahg = f[0];
    let yak = u16::oa([f[1], f[2]]);
    let go = u16::oa([f[3], f[4]]) as usize;
    
    if f.len() < 5 + go {
        return Err(TlsError::Fs);
    }
    
    let ehd = &f[5..5 + go];
    
    match ahg {
        20 => {
            
            Ok(None)
        }
        21 => {
            
            if ehd.len() >= 2 {
                let jy = ehd[0];
                let desc = ehd[1];
                crate::serial_println!("[TLS] Alert: level={} desc={}", jy, desc);
                
                if desc == 0 {
                    
                    he.g = TlsState::Dk;
                } else {
                    he.g = TlsState::Q;
                }
            }
            Err(TlsError::Ahe)
        }
        22 => {
            
            vmo(he, ehd)
        }
        23 => {
            
            if he.g == TlsState::Bsn {
                
                vmn(he, ehd)
            } else if he.g == TlsState::Kd {
                
                let ajk = kop(he, ehd)?;
                Ok(Some(ajk))
            } else {
                Err(TlsError::Oj)
            }
        }
        _ => Err(TlsError::Fs),
    }
}


fn vmo(he: &mut TlsSession, f: &[u8]) -> Result<Option<Vec<u8>>, TlsError> {
    if f.is_empty() {
        return Err(TlsError::Fs);
    }
    
    let msg_type = f[0];
    
    match msg_type {
        2 => {
            
            handshake::vdq(he, f)?;
            Ok(None)
        }
        _ => {
            crate::serial_println!("[TLS] Unexpected handshake type: {}", msg_type);
            Err(TlsError::Oj)
        }
    }
}


fn vmn(he: &mut TlsSession, f: &[u8]) -> Result<Option<Vec<u8>>, TlsError> {
    
    let ajk = kop(he, f)?;
    
    if ajk.is_empty() {
        return Err(TlsError::Fs);
    }
    
    
    let mut jzd = 0u8;
    let mut nfq = ajk.len();
    for a in (0..ajk.len()).vv() {
        if ajk[a] != 0 {
            jzd = ajk[a];
            nfq = a;
            break;
        }
    }
    
    if jzd != ContentType::Atm as u8 {
        if jzd == ContentType::Bbo as u8 {
            return Err(TlsError::Ahe);
        }
        return Err(TlsError::Oj);
    }
    
    let fkb = &ajk[..nfq];
    
    
    let mut u = 0;
    while u + 4 <= fkb.len() {
        let msg_type = fkb[u];
        let gng = ((fkb[u + 1] as usize) << 16)
            | ((fkb[u + 2] as usize) << 8)
            | (fkb[u + 3] as usize);
        
        if u + 4 + gng > fkb.len() {
            break;
        }
        
        let fr = &fkb[u..u + 4 + gng];
        
        match msg_type {
            8 => {
                
                handshake::vce(he, fr)?;
            }
            11 => {
                
                handshake::vby(he, fr)?;
            }
            15 => {
                
                handshake::vca(he, fr)?;
            }
            20 => {
                
                handshake::vch(he, fr)?;
                
                
                let mut fxh = he.ape.clone();
                let ape = fxh.bqs();
                
                
                let rbo = handshake::qtb(he);
                let ktj = kti(he, ContentType::Atm, &rbo)?;
                
                
                handshake::rvw(he, &ape)?;
                
                he.g = TlsState::Kd;
                
                return Ok(Some(ktj));
            }
            4 => {
                
                he.ape.qs(fr);
            }
            _ => {
                crate::serial_println!("[TLS] Unknown encrypted handshake type: {}", msg_type);
            }
        }
        
        u += 4 + gng;
    }
    
    Ok(None)
}


pub fn kti(he: &mut TlsSession, ahg: ContentType, ajk: &[u8]) -> Result<Vec<u8>, TlsError> {
    
    let mut ff = Vec::fc(ajk.len() + 1);
    ff.bk(ajk);
    ff.push(ahg as u8);
    
    
    let mut brn = [0u8; 12];
    brn.dg(&he.inz);
    let mdw = he.iny.ft();
    for a in 0..8 {
        brn[4 + a] ^= mdw[a];
    }
    he.iny += 1;
    
    
    let blv = [
        ContentType::Kd as u8,
        0x03, 0x03, 
        ((ff.len() + 16) >> 8) as u8,
        (ff.len() + 16) as u8,
    ];
    
    
    let afm = ijd(&he.ioa, &brn, &blv, &ff);
    
    
    let mut record = Vec::fc(5 + afm.len());
    record.push(ContentType::Kd as u8);
    record.bk(&[0x03, 0x03]); 
    record.bk(&(afm.len() as u16).ft());
    record.bk(&afm);
    
    Ok(record)
}


pub fn kop(he: &mut TlsSession, afm: &[u8]) -> Result<Vec<u8>, TlsError> {
    if afm.len() < 16 {
        return Err(TlsError::Aqq);
    }
    
    crate::serial_println!("[TLS] Decrypting {} bytes, seq={}", afm.len(), he.hzt);
    
    
    let mut brn = [0u8; 12];
    brn.dg(&he.hzu);
    let mdw = he.hzt.ft();
    for a in 0..8 {
        brn[4 + a] ^= mdw[a];
    }
    he.hzt += 1;
    
    crate::serial_println!("[TLS] Key={:02x?} Nonce={:02x?}", &he.gse[..8], &brn[..8]);
    
    
    let blv = [
        ContentType::Kd as u8,
        0x03, 0x03,
        (afm.len() >> 8) as u8,
        afm.len() as u8,
    ];
    
    
    let ajk = muf(&he.gse, &brn, &blv, afm)
        .jd(|_| TlsError::Aqq)?;
    
    Ok(ajk)
}
