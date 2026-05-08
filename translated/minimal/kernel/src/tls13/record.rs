



use alloc::vec::Vec;
use super::{TlsSession, TlsState, TlsError, ContentType, HandshakeType};
use super::crypto::{efc, hee};
use super::handshake;


pub const DXD_: usize = 16384 + 256;


pub fn process_record(by: &mut TlsSession, data: &[u8]) -> Result<Option<Vec<u8>>, TlsError> {
    if data.len() < 5 {
        return Err(TlsError::ProtocolError);
    }
    
    let content_type = data[0];
    let pxb = u16::from_be_bytes([data[1], data[2]]);
    let length = u16::from_be_bytes([data[3], data[4]]) as usize;
    
    if data.len() < 5 + length {
        return Err(TlsError::ProtocolError);
    }
    
    let bvf = &data[5..5 + length];
    
    match content_type {
        20 => {
            
            Ok(None)
        }
        21 => {
            
            if bvf.len() >= 2 {
                let level = bvf[0];
                let desc = bvf[1];
                crate::serial_println!("[TLS] Alert: level={} desc={}", level, desc);
                
                if desc == 0 {
                    
                    by.state = TlsState::Closed;
                } else {
                    by.state = TlsState::Error;
                }
            }
            Err(TlsError::ConnectionClosed)
        }
        22 => {
            
            nyi(by, bvf)
        }
        23 => {
            
            if by.state == TlsState::ServerHelloReceived {
                
                nyh(by, bvf)
            } else if by.state == TlsState::ApplicationData {
                
                let ry = frc(by, bvf)?;
                Ok(Some(ry))
            } else {
                Err(TlsError::UnexpectedMessage)
            }
        }
        _ => Err(TlsError::ProtocolError),
    }
}


fn nyi(by: &mut TlsSession, data: &[u8]) -> Result<Option<Vec<u8>>, TlsError> {
    if data.is_empty() {
        return Err(TlsError::ProtocolError);
    }
    
    let msg_type = data[0];
    
    match msg_type {
        2 => {
            
            handshake::nrd(by, data)?;
            Ok(None)
        }
        _ => {
            crate::serial_println!("[TLS] Unexpected handshake type: {}", msg_type);
            Err(TlsError::UnexpectedMessage)
        }
    }
}


fn nyh(by: &mut TlsSession, data: &[u8]) -> Result<Option<Vec<u8>>, TlsError> {
    
    let ry = frc(by, data)?;
    
    if ry.is_empty() {
        return Err(TlsError::ProtocolError);
    }
    
    
    let mut fgc = 0u8;
    let mut hnk = ry.len();
    for i in (0..ry.len()).rev() {
        if ry[i] != 0 {
            fgc = ry[i];
            hnk = i;
            break;
        }
    }
    
    if fgc != ContentType::Handshake as u8 {
        if fgc == ContentType::Alert as u8 {
            return Err(TlsError::ConnectionClosed);
        }
        return Err(TlsError::UnexpectedMessage);
    }
    
    let ckg = &ry[..hnk];
    
    
    let mut pos = 0;
    while pos + 4 <= ckg.len() {
        let msg_type = ckg[pos];
        let dbl = ((ckg[pos + 1] as usize) << 16)
            | ((ckg[pos + 2] as usize) << 8)
            | (ckg[pos + 3] as usize);
        
        if pos + 4 + dbl > ckg.len() {
            break;
        }
        
        let bk = &ckg[pos..pos + 4 + dbl];
        
        match msg_type {
            8 => {
                
                handshake::nqe(by, bk)?;
            }
            11 => {
                
                handshake::npz(by, bk)?;
            }
            15 => {
                
                handshake::nqb(by, bk)?;
            }
            20 => {
                
                handshake::nqh(by, bk)?;
                
                
                let mut csa = by.transcript_hash.clone();
                let transcript_hash = csa.finalize();
                
                
                let klb = handshake::keu(by);
                let fur = fuq(by, ContentType::Handshake, &klb)?;
                
                
                handshake::ldm(by, &transcript_hash)?;
                
                by.state = TlsState::ApplicationData;
                
                return Ok(Some(fur));
            }
            4 => {
                
                by.transcript_hash.update(bk);
            }
            _ => {
                crate::serial_println!("[TLS] Unknown encrypted handshake type: {}", msg_type);
            }
        }
        
        pos += 4 + dbl;
    }
    
    Ok(None)
}


pub fn fuq(by: &mut TlsSession, content_type: ContentType, ry: &[u8]) -> Result<Vec<u8>, TlsError> {
    
    let mut inner = Vec::with_capacity(ry.len() + 1);
    inner.extend_from_slice(ry);
    inner.push(content_type as u8);
    
    
    let mut akh = [0u8; 12];
    akh.copy_from_slice(&by.client_write_iv);
    let gua = by.client_seq.to_be_bytes();
    for i in 0..8 {
        akh[4 + i] ^= gua[i];
    }
    by.client_seq += 1;
    
    
    let ahh = [
        ContentType::ApplicationData as u8,
        0x03, 0x03, 
        ((inner.len() + 16) >> 8) as u8,
        (inner.len() + 16) as u8,
    ];
    
    
    let pw = efc(&by.client_write_key, &akh, &ahh, &inner);
    
    
    let mut record = Vec::with_capacity(5 + pw.len());
    record.push(ContentType::ApplicationData as u8);
    record.extend_from_slice(&[0x03, 0x03]); 
    record.extend_from_slice(&(pw.len() as u16).to_be_bytes());
    record.extend_from_slice(&pw);
    
    Ok(record)
}


pub fn frc(by: &mut TlsSession, pw: &[u8]) -> Result<Vec<u8>, TlsError> {
    if pw.len() < 16 {
        return Err(TlsError::DecryptionFailed);
    }
    
    crate::serial_println!("[TLS] Decrypting {} bytes, seq={}", pw.len(), by.server_seq);
    
    
    let mut akh = [0u8; 12];
    akh.copy_from_slice(&by.server_write_iv);
    let gua = by.server_seq.to_be_bytes();
    for i in 0..8 {
        akh[4 + i] ^= gua[i];
    }
    by.server_seq += 1;
    
    crate::serial_println!("[TLS] Key={:02x?} Nonce={:02x?}", &by.server_write_key[..8], &akh[..8]);
    
    
    let ahh = [
        ContentType::ApplicationData as u8,
        0x03, 0x03,
        (pw.len() >> 8) as u8,
        pw.len() as u8,
    ];
    
    
    let ry = hee(&by.server_write_key, &akh, &ahh, pw)
        .map_err(|_| TlsError::DecryptionFailed)?;
    
    Ok(ry)
}
