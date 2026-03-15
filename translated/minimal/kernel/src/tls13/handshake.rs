



use alloc::vec::Vec;
use super::{TlsSession, TlsState, TlsError, ContentType, HandshakeType, CipherSuite};
use super::crypto::{self, Sha256, chw, iyk, gjc, fgo};


pub fn kfj(he: &mut TlsSession) -> Vec<u8> {
    let mut handshake = Vec::new();
    
    
    handshake.push(HandshakeType::Bzk as u8);
    
    
    let lik = handshake.len();
    handshake.bk(&[0, 0, 0]);
    
    
    
    
    handshake.bk(&[0x03, 0x03]);
    
    
    handshake.bk(&he.inx);
    
    
    handshake.push(0); 
    
    
    handshake.bk(&[0x00, 0x02]); 
    handshake.bk(&(CipherSuite::AIW_ as u16).ft());
    
    
    handshake.push(0x01); 
    handshake.push(0x00); 
    
    
    let kuq = handshake.len();
    handshake.bk(&[0x00, 0x00]); 
    
    
    handshake.bk(&[0x00, 0x2b]); 
    handshake.bk(&[0x00, 0x03]); 
    handshake.push(0x02); 
    handshake.bk(&[0x03, 0x04]); 
    
    
    handshake.bk(&[0x00, 0x0a]); 
    handshake.bk(&[0x00, 0x04]); 
    handshake.bk(&[0x00, 0x02]); 
    handshake.bk(&[0x00, 0x1d]); 
    
    
    handshake.bk(&[0x00, 0x0d]); 
    handshake.bk(&[0x00, 0x04]); 
    handshake.bk(&[0x00, 0x02]); 
    handshake.bk(&[0x08, 0x04]); 
    
    
    handshake.bk(&[0x00, 0x33]); 
    let ohv = 2 + 2 + 2 + 32; 
    handshake.bk(&(ohv as u16).ft());
    handshake.bk(&((ohv - 2) as u16).ft()); 
    handshake.bk(&[0x00, 0x1d]); 
    handshake.bk(&[0x00, 0x20]); 
    handshake.bk(&he.kso);
    
    
    if !he.ajc.is_empty() {
        let lcm = he.ajc.as_bytes();
        let plw = 2 + 1 + 2 + lcm.len();
        handshake.bk(&[0x00, 0x00]); 
        handshake.bk(&(plw as u16).ft());
        handshake.bk(&((plw - 2) as u16).ft()); 
        handshake.push(0x00); 
        handshake.bk(&(lcm.len() as u16).ft());
        handshake.bk(lcm);
    }
    
    
    let itl = (handshake.len() - kuq - 2) as u16;
    handshake[kuq] = (itl >> 8) as u8;
    handshake[kuq + 1] = itl as u8;
    
    
    let lbb = (handshake.len() - 4) as u32;
    handshake[lik] = (lbb >> 16) as u8;
    handshake[lik + 1] = (lbb >> 8) as u8;
    handshake[lik + 2] = lbb as u8;
    
    
    he.ape.qs(&handshake);
    
    
    let mut record = Vec::new();
    record.push(ContentType::Atm as u8);
    record.bk(&[0x03, 0x01]); 
    record.bk(&(handshake.len() as u16).ft());
    record.bk(&handshake);
    
    record
}


pub fn vdq(he: &mut TlsSession, f: &[u8]) -> Result<(), TlsError> {
    if f.len() < 38 {
        return Err(TlsError::Fs);
    }
    
    
    if f[0] != HandshakeType::Cmu as u8 {
        return Err(TlsError::Oj);
    }
    
    
    let go = ((f[1] as usize) << 16) | ((f[2] as usize) << 8) | (f[3] as usize);
    if f.len() < 4 + go {
        return Err(TlsError::Fs);
    }
    
    
    he.ape.qs(&f[..4 + go]);
    
    
    let mut u = 4;
    u += 2;
    
    
    if u + 32 > f.len() {
        return Err(TlsError::Fs);
    }
    he.pim.dg(&f[u..u + 32]);
    u += 32;
    
    
    if u >= f.len() {
        return Err(TlsError::Fs);
    }
    let wic = f[u] as usize;
    u += 1 + wic;
    
    
    if u + 2 > f.len() {
        return Err(TlsError::Fs);
    }
    let ins = u16::oa([f[u], f[u + 1]]);
    u += 2;
    
    if ins == CipherSuite::AIW_ as u16 {
        he.ins = Some(CipherSuite::AIW_);
    } else {
        return Err(TlsError::Atn);
    }
    
    
    u += 1;
    
    
    if u + 2 > f.len() {
        return Err(TlsError::Fs);
    }
    let itl = u16::oa([f[u], f[u + 1]]) as usize;
    u += 2;
    
    let kup = u + itl;
    let mut pil: Option<[u8; 32]> = None;
    
    while u + 4 <= kup {
        let spv = u16::oa([f[u], f[u + 1]]);
        let hip = u16::oa([f[u + 2], f[u + 3]]) as usize;
        u += 4;
        
        if u + hip > kup {
            break;
        }
        
        match spv {
            0x0033 => {
                
                if hip >= 36 {
                    let cyi = u16::oa([f[u], f[u + 1]]);
                    let ubh = u16::oa([f[u + 2], f[u + 3]]) as usize;
                    
                    if cyi == 0x001d && ubh == 32 && u + 4 + 32 <= kup {
                        let mut bs = [0u8; 32];
                        bs.dg(&f[u + 4..u + 4 + 32]);
                        pil = Some(bs);
                    }
                }
            }
            0x002b => {
                
            }
            _ => {}
        }
        
        u += hip;
    }
    
    
    let pij = pil.ok_or(TlsError::Atn)?;
    
    crate::serial_println!("[TLS] Server public key: {:02x?}", &pij[..16]);
    crate::serial_println!("[TLS] Client private key: {:02x?}", &he.gfv[..16]);
    
    let jpt = crypto::jxk(&he.gfv, &pij);
    
    crate::serial_println!("[TLS] Shared secret: {:02x?}", &jpt[..16]);
    
    
    rvx(he, &jpt)?;
    
    he.g = TlsState::Bsn;
    Ok(())
}


fn rvx(he: &mut TlsSession, jpt: &[u8; 32]) -> Result<(), TlsError> {
    
    let mrz = [0u8; 32];
    let sie = iyk(&[], &mrz);
    
    
    let ktc = chw(&[]);
    let kpi = fgo(&sie, "derived", &ktc);
    
    
    he.hml = iyk(&kpi, jpt);
    
    
    let mut fxh = he.ape.clone();
    let ape = fxh.bqs();
    
    
    he.inw = fgo(
        &he.hml,
        "c hs traffic",
        &ape,
    );
    
    
    he.jon = fgo(
        &he.hml,
        "s hs traffic",
        &ape,
    );
    
    crate::serial_println!("[TLS] Handshake secrets derived");
    crate::serial_println!("[TLS] Transcript hash: {:02x?}", &ape[..16]);
    
    
    iqz(
        &he.jon,
        &mut he.gse,
        &mut he.hzu,
    );
    
    crate::serial_println!("[TLS] Server write key: {:02x?}", &he.gse[..8]);
    crate::serial_println!("[TLS] Server write IV: {:02x?}", &he.hzu);
    
    iqz(
        &he.inw,
        &mut he.ioa,
        &mut he.inz,
    );
    
    Ok(())
}


pub fn rvw(he: &mut TlsSession, ape: &[u8; 32]) -> Result<(), TlsError> {
    
    let ktc = chw(&[]);
    let kpi = fgo(&he.hml, "derived", &ktc);
    
    
    let mrz = [0u8; 32];
    let ole = iyk(&kpi, &mrz);
    
    
    he.kib = fgo(
        &ole,
        "c ap traffic",
        ape,
    );
    
    he.mea = fgo(
        &ole,
        "s ap traffic",
        ape,
    );
    
    
    iqz(
        &he.mea,
        &mut he.gse,
        &mut he.hzu,
    );
    
    iqz(
        &he.kib,
        &mut he.ioa,
        &mut he.inz,
    );
    
    
    he.iny = 0;
    he.hzt = 0;
    
    Ok(())
}


fn iqz(eig: &[u8; 32], bs: &mut [u8; 16], hph: &mut [u8; 12]) {
    let ubf = gjc(eig, "key", &[], 16);
    let uac = gjc(eig, "iv", &[], 12);
    
    bs.dg(&ubf);
    hph.dg(&uac);
}


pub fn vce(he: &mut TlsSession, f: &[u8]) -> Result<(), TlsError> {
    if f.is_empty() || f[0] != HandshakeType::Cbq as u8 {
        return Err(TlsError::Oj);
    }
    
    
    he.ape.qs(f);
    Ok(())
}


pub fn vby(he: &mut TlsSession, f: &[u8]) -> Result<(), TlsError> {
    if f.is_empty() || f[0] != HandshakeType::Certificate as u8 {
        return Err(TlsError::Oj);
    }
    
    
    he.ape.qs(f);
    
    
    let ncb = super::x509::vbz(f);
    
    if ncb.is_empty() {
        crate::serial_println!("[TLS] No certificates in chain — rejecting");
        return Err(TlsError::Apv);
    }
    
    let awa = &ncb[0];
    
    
    if !he.ajc.is_empty() && !awa.xqk(&he.ajc) {
        crate::serial_println!(
            "[TLS] Certificate hostname mismatch: expected '{}', got CN={:?} SAN={:?}",
            he.ajc,
            awa.icc,
            awa.grg
        );
        return Err(TlsError::Apv);
    }
    
    
    he.joo = awa.cbd.clone();
    he.pik = awa.lwc.clone();
    
    crate::serial_println!(
        "[TLS] Certificate accepted: CN={:?}, issuer={:?}, SAN count={}",
        awa.icc,
        awa.lgr,
        awa.grg.len()
    );
    
    Ok(())
}


pub fn vca(he: &mut TlsSession, f: &[u8]) -> Result<(), TlsError> {
    if f.is_empty() || f[0] != HandshakeType::Bzi as u8 {
        return Err(TlsError::Oj);
    }
    
    
    
    if f.len() < 8 {
        return Err(TlsError::Fs);
    }
    
    let gng = ((f[1] as usize) << 16) | ((f[2] as usize) << 8) | (f[3] as usize);
    if f.len() < 4 + gng || gng < 4 {
        return Err(TlsError::Fs);
    }
    
    let wnx = u16::oa([f[4], f[5]]);
    let mfv = u16::oa([f[6], f[7]]) as usize;
    
    if f.len() < 8 + mfv {
        return Err(TlsError::Fs);
    }
    
    let qdq = &f[8..8 + mfv];
    
    
    
    let ape = he.ape.clone().bqs();
    
    let mut jqi = Vec::fc(64 + 34 + 32);
    jqi.bk(&[0x20u8; 64]); 
    jqi.bk(b"TLS 1.3, server CertificateVerify");
    jqi.push(0x00);
    jqi.bk(&ape);
    
    
    
    
    
    if he.joo.is_empty() {
        crate::serial_println!("[TLS] CertificateVerify: no server pubkey available — rejecting");
        return Err(TlsError::Apv);
    }
    
    crate::serial_println!(
        "[TLS] CertificateVerify: algo=0x{:04X}, sig_len={}, pubkey_len={}",
        wnx, mfv, he.joo.len()
    );
    
    
    he.ape.qs(f);
    
    Ok(())
}


pub fn vch(he: &mut TlsSession, f: &[u8]) -> Result<(), TlsError> {
    if f.is_empty() || f[0] != HandshakeType::Bhf as u8 {
        return Err(TlsError::Oj);
    }
    
    if f.len() < 4 + 32 {
        return Err(TlsError::Fs);
    }
    
    let mpb = &f[4..4 + 32];
    
    
    let mut fxh = he.ape.clone();
    let ape = fxh.bqs();
    
    let kwh = gjc(&he.jon, "finished", &[], 32);
    let mut hjs = [0u8; 32];
    hjs.dg(&kwh);
    
    let qy = crypto::drt(&hjs, &ape);
    
    
    let mut wz = 0u8;
    for a in 0..32 {
        wz |= mpb[a] ^ qy[a];
    }
    
    if wz != 0 {
        return Err(TlsError::Aqq);
    }
    
    
    he.ape.qs(f);
    
    Ok(())
}


pub fn qtb(he: &mut TlsSession) -> Vec<u8> {
    
    let mut fxh = he.ape.clone();
    let ape = fxh.bqs();
    
    
    let kwh = gjc(&he.inw, "finished", &[], 32);
    let mut hjs = [0u8; 32];
    hjs.dg(&kwh);
    
    let mpb = crypto::drt(&hjs, &ape);
    
    
    let mut hjr = Vec::new();
    hjr.push(HandshakeType::Bhf as u8);
    hjr.bk(&[0x00, 0x00, 0x20]); 
    hjr.bk(&mpb);
    
    
    he.ape.qs(&hjr);
    
    hjr
}
