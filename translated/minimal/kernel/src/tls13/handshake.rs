



use alloc::vec::Vec;
use super::{TlsSession, TlsState, TlsError, ContentType, HandshakeType, CipherSuite};
use super::crypto::{self, Sha256, asg, epg, czf, cid};


pub fn build_client_hello(by: &mut TlsSession) -> Vec<u8> {
    let mut handshake = Vec::new();
    
    
    handshake.push(HandshakeType::ClientHello as u8);
    
    
    let gfh = handshake.len();
    handshake.extend_from_slice(&[0, 0, 0]);
    
    
    
    
    handshake.extend_from_slice(&[0x03, 0x03]);
    
    
    handshake.extend_from_slice(&by.client_random);
    
    
    handshake.push(0); 
    
    
    handshake.extend_from_slice(&[0x00, 0x02]); 
    handshake.extend_from_slice(&(CipherSuite::TLS_AES_128_GCM_SHA256 as u16).to_be_bytes());
    
    
    handshake.push(0x01); 
    handshake.push(0x00); 
    
    
    let fvu = handshake.len();
    handshake.extend_from_slice(&[0x00, 0x00]); 
    
    
    handshake.extend_from_slice(&[0x00, 0x2b]); 
    handshake.extend_from_slice(&[0x00, 0x03]); 
    handshake.push(0x02); 
    handshake.extend_from_slice(&[0x03, 0x04]); 
    
    
    handshake.extend_from_slice(&[0x00, 0x0a]); 
    handshake.extend_from_slice(&[0x00, 0x04]); 
    handshake.extend_from_slice(&[0x00, 0x02]); 
    handshake.extend_from_slice(&[0x00, 0x1d]); 
    
    
    handshake.extend_from_slice(&[0x00, 0x0d]); 
    handshake.extend_from_slice(&[0x00, 0x04]); 
    handshake.extend_from_slice(&[0x00, 0x02]); 
    handshake.extend_from_slice(&[0x08, 0x04]); 
    
    
    handshake.extend_from_slice(&[0x00, 0x33]); 
    let ijb = 2 + 2 + 2 + 32; 
    handshake.extend_from_slice(&(ijb as u16).to_be_bytes());
    handshake.extend_from_slice(&((ijb - 2) as u16).to_be_bytes()); 
    handshake.extend_from_slice(&[0x00, 0x1d]); 
    handshake.extend_from_slice(&[0x00, 0x20]); 
    handshake.extend_from_slice(&by.ecdhe_public);
    
    
    if !by.hostname.is_empty() {
        let gba = by.hostname.as_bytes();
        let jgy = 2 + 1 + 2 + gba.len();
        handshake.extend_from_slice(&[0x00, 0x00]); 
        handshake.extend_from_slice(&(jgy as u16).to_be_bytes());
        handshake.extend_from_slice(&((jgy - 2) as u16).to_be_bytes()); 
        handshake.push(0x00); 
        handshake.extend_from_slice(&(gba.len() as u16).to_be_bytes());
        handshake.extend_from_slice(gba);
    }
    
    
    let elv = (handshake.len() - fvu - 2) as u16;
    handshake[fvu] = (elv >> 8) as u8;
    handshake[fvu + 1] = elv as u8;
    
    
    let fzt = (handshake.len() - 4) as u32;
    handshake[gfh] = (fzt >> 16) as u8;
    handshake[gfh + 1] = (fzt >> 8) as u8;
    handshake[gfh + 2] = fzt as u8;
    
    
    by.transcript_hash.update(&handshake);
    
    
    let mut record = Vec::new();
    record.push(ContentType::Handshake as u8);
    record.extend_from_slice(&[0x03, 0x01]); 
    record.extend_from_slice(&(handshake.len() as u16).to_be_bytes());
    record.extend_from_slice(&handshake);
    
    record
}


pub fn nrd(by: &mut TlsSession, data: &[u8]) -> Result<(), TlsError> {
    if data.len() < 38 {
        return Err(TlsError::ProtocolError);
    }
    
    
    if data[0] != HandshakeType::ServerHello as u8 {
        return Err(TlsError::UnexpectedMessage);
    }
    
    
    let length = ((data[1] as usize) << 16) | ((data[2] as usize) << 8) | (data[3] as usize);
    if data.len() < 4 + length {
        return Err(TlsError::ProtocolError);
    }
    
    
    by.transcript_hash.update(&data[..4 + length]);
    
    
    let mut pos = 4;
    pos += 2;
    
    
    if pos + 32 > data.len() {
        return Err(TlsError::ProtocolError);
    }
    by.server_random.copy_from_slice(&data[pos..pos + 32]);
    pos += 32;
    
    
    if pos >= data.len() {
        return Err(TlsError::ProtocolError);
    }
    let ool = data[pos] as usize;
    pos += 1 + ool;
    
    
    if pos + 2 > data.len() {
        return Err(TlsError::ProtocolError);
    }
    let cipher_suite = u16::from_be_bytes([data[pos], data[pos + 1]]);
    pos += 2;
    
    if cipher_suite == CipherSuite::TLS_AES_128_GCM_SHA256 as u16 {
        by.cipher_suite = Some(CipherSuite::TLS_AES_128_GCM_SHA256);
    } else {
        return Err(TlsError::HandshakeFailed);
    }
    
    
    pos += 1;
    
    
    if pos + 2 > data.len() {
        return Err(TlsError::ProtocolError);
    }
    let elv = u16::from_be_bytes([data[pos], data[pos + 1]]) as usize;
    pos += 2;
    
    let fvt = pos + elv;
    let mut jev: Option<[u8; 32]> = None;
    
    while pos + 4 <= fvt {
        let ltj = u16::from_be_bytes([data[pos], data[pos + 1]]);
        let dpc = u16::from_be_bytes([data[pos + 2], data[pos + 3]]) as usize;
        pos += 4;
        
        if pos + dpc > fvt {
            break;
        }
        
        match ltj {
            0x0033 => {
                
                if dpc >= 36 {
                    let bbz = u16::from_be_bytes([data[pos], data[pos + 1]]);
                    let mvr = u16::from_be_bytes([data[pos + 2], data[pos + 3]]) as usize;
                    
                    if bbz == 0x001d && mvr == 32 && pos + 4 + 32 <= fvt {
                        let mut key = [0u8; 32];
                        key.copy_from_slice(&data[pos + 4..pos + 4 + 32]);
                        jev = Some(key);
                    }
                }
            }
            0x002b => {
                
            }
            _ => {}
        }
        
        pos += dpc;
    }
    
    
    let jeu = jev.ok_or(TlsError::HandshakeFailed)?;
    
    crate::serial_println!("[TLS] Server public key: {:02x?}", &jeu[..16]);
    crate::serial_println!("[TLS] Client private key: {:02x?}", &by.ecdhe_private[..16]);
    
    let faq = crypto::ffl(&by.ecdhe_private, &jeu);
    
    crate::serial_println!("[TLS] Shared secret: {:02x?}", &faq[..16]);
    
    
    ldn(by, &faq)?;
    
    by.state = TlsState::ServerHelloReceived;
    Ok(())
}


fn ldn(by: &mut TlsSession, faq: &[u8; 32]) -> Result<(), TlsError> {
    
    let hdg = [0u8; 32];
    let lnf = epg(&[], &hdg);
    
    
    let fuk = asg(&[]);
    let frq = cid(&lnf, "derived", &fuk);
    
    
    by.handshake_secret = epg(&frq, faq);
    
    
    let mut csa = by.transcript_hash.clone();
    let transcript_hash = csa.finalize();
    
    
    by.client_handshake_traffic_secret = cid(
        &by.handshake_secret,
        "c hs traffic",
        &transcript_hash,
    );
    
    
    by.server_handshake_traffic_secret = cid(
        &by.handshake_secret,
        "s hs traffic",
        &transcript_hash,
    );
    
    crate::serial_println!("[TLS] Handshake secrets derived");
    crate::serial_println!("[TLS] Transcript hash: {:02x?}", &transcript_hash[..16]);
    
    
    ekb(
        &by.server_handshake_traffic_secret,
        &mut by.server_write_key,
        &mut by.server_write_iv,
    );
    
    crate::serial_println!("[TLS] Server write key: {:02x?}", &by.server_write_key[..8]);
    crate::serial_println!("[TLS] Server write IV: {:02x?}", &by.server_write_iv);
    
    ekb(
        &by.client_handshake_traffic_secret,
        &mut by.client_write_key,
        &mut by.client_write_iv,
    );
    
    Ok(())
}


pub fn ldm(by: &mut TlsSession, transcript_hash: &[u8; 32]) -> Result<(), TlsError> {
    
    let fuk = asg(&[]);
    let frq = cid(&by.handshake_secret, "derived", &fuk);
    
    
    let hdg = [0u8; 32];
    let ilx = epg(&frq, &hdg);
    
    
    by.client_application_traffic_secret = cid(
        &ilx,
        "c ap traffic",
        transcript_hash,
    );
    
    by.server_application_traffic_secret = cid(
        &ilx,
        "s ap traffic",
        transcript_hash,
    );
    
    
    ekb(
        &by.server_application_traffic_secret,
        &mut by.server_write_key,
        &mut by.server_write_iv,
    );
    
    ekb(
        &by.client_application_traffic_secret,
        &mut by.client_write_key,
        &mut by.client_write_iv,
    );
    
    
    by.client_seq = 0;
    by.server_seq = 0;
    
    Ok(())
}


fn ekb(bvr: &[u8; 32], key: &mut [u8; 16], dsz: &mut [u8; 12]) {
    let mvp = czf(bvr, "key", &[], 16);
    let muu = czf(bvr, "iv", &[], 12);
    
    key.copy_from_slice(&mvp);
    dsz.copy_from_slice(&muu);
}


pub fn nqe(by: &mut TlsSession, data: &[u8]) -> Result<(), TlsError> {
    if data.is_empty() || data[0] != HandshakeType::EncryptedExtensions as u8 {
        return Err(TlsError::UnexpectedMessage);
    }
    
    
    by.transcript_hash.update(data);
    Ok(())
}


pub fn npz(by: &mut TlsSession, data: &[u8]) -> Result<(), TlsError> {
    if data.is_empty() || data[0] != HandshakeType::Certificate as u8 {
        return Err(TlsError::UnexpectedMessage);
    }
    
    
    by.transcript_hash.update(data);
    
    
    let hkc = super::x509::nqa(data);
    
    if hkc.is_empty() {
        crate::serial_println!("[TLS] No certificates in chain — rejecting");
        return Err(TlsError::CertificateInvalid);
    }
    
    let leaf = &hkc[0];
    
    
    if !by.hostname.is_empty() && !leaf.valid_for_hostname(&by.hostname) {
        crate::serial_println!(
            "[TLS] Certificate hostname mismatch: expected '{}', got CN={:?} SAN={:?}",
            by.hostname,
            leaf.subject_cn,
            leaf.san
        );
        return Err(TlsError::CertificateInvalid);
    }
    
    
    by.server_pubkey = leaf.pubkey.clone();
    by.server_pubkey_algo = leaf.pubkey_algo.clone();
    
    crate::serial_println!(
        "[TLS] Certificate accepted: CN={:?}, issuer={:?}, SAN count={}",
        leaf.subject_cn,
        leaf.issuer_cn,
        leaf.san.len()
    );
    
    Ok(())
}


pub fn nqb(by: &mut TlsSession, data: &[u8]) -> Result<(), TlsError> {
    if data.is_empty() || data[0] != HandshakeType::CertificateVerify as u8 {
        return Err(TlsError::UnexpectedMessage);
    }
    
    
    
    if data.len() < 8 {
        return Err(TlsError::ProtocolError);
    }
    
    let dbl = ((data[1] as usize) << 16) | ((data[2] as usize) << 8) | (data[3] as usize);
    if data.len() < 4 + dbl || dbl < 4 {
        return Err(TlsError::ProtocolError);
    }
    
    let osl = u16::from_be_bytes([data[4], data[5]]);
    let gvb = u16::from_be_bytes([data[6], data[7]]) as usize;
    
    if data.len() < 8 + gvb {
        return Err(TlsError::ProtocolError);
    }
    
    let jss = &data[8..8 + gvb];
    
    
    
    let transcript_hash = by.transcript_hash.clone().finalize();
    
    let mut faz = Vec::with_capacity(64 + 34 + 32);
    faz.extend_from_slice(&[0x20u8; 64]); 
    faz.extend_from_slice(b"TLS 1.3, server CertificateVerify");
    faz.push(0x00);
    faz.extend_from_slice(&transcript_hash);
    
    
    
    
    
    if by.server_pubkey.is_empty() {
        crate::serial_println!("[TLS] CertificateVerify: no server pubkey available — rejecting");
        return Err(TlsError::CertificateInvalid);
    }
    
    crate::serial_println!(
        "[TLS] CertificateVerify: algo=0x{:04X}, sig_len={}, pubkey_len={}",
        osl, gvb, by.server_pubkey.len()
    );
    
    
    by.transcript_hash.update(data);
    
    Ok(())
}


pub fn nqh(by: &mut TlsSession, data: &[u8]) -> Result<(), TlsError> {
    if data.is_empty() || data[0] != HandshakeType::Finished as u8 {
        return Err(TlsError::UnexpectedMessage);
    }
    
    if data.len() < 4 + 32 {
        return Err(TlsError::ProtocolError);
    }
    
    let hbl = &data[4..4 + 32];
    
    
    let mut csa = by.transcript_hash.clone();
    let transcript_hash = csa.finalize();
    
    let fxa = czf(&by.server_handshake_traffic_secret, "finished", &[], 32);
    let mut dpt = [0u8; 32];
    dpt.copy_from_slice(&fxa);
    
    let expected = crypto::bmu(&dpt, &transcript_hash);
    
    
    let mut jr = 0u8;
    for i in 0..32 {
        jr |= hbl[i] ^ expected[i];
    }
    
    if jr != 0 {
        return Err(TlsError::DecryptionFailed);
    }
    
    
    by.transcript_hash.update(data);
    
    Ok(())
}


pub fn keu(by: &mut TlsSession) -> Vec<u8> {
    
    let mut csa = by.transcript_hash.clone();
    let transcript_hash = csa.finalize();
    
    
    let fxa = czf(&by.client_handshake_traffic_secret, "finished", &[], 32);
    let mut dpt = [0u8; 32];
    dpt.copy_from_slice(&fxa);
    
    let hbl = crypto::bmu(&dpt, &transcript_hash);
    
    
    let mut dps = Vec::new();
    dps.push(HandshakeType::Finished as u8);
    dps.extend_from_slice(&[0x00, 0x00, 0x20]); 
    dps.extend_from_slice(&hbl);
    
    
    by.transcript_hash.update(&dps);
    
    dps
}
