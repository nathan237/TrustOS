//! TLS 1.3 Record Layer
//!
//! Handles encryption/decryption of TLS records.

use alloc::vec::Vec;
use super::{TlsSession, TlsState, TlsError, ContentType, HandshakeType};
use super::crypto::{aes_gcm_encrypt, aes_gcm_decrypt};
use super::handshake;

/// Maximum TLS record size (16KB + overhead)
pub const MAX_RECORD_SIZE: usize = 16384 + 256;

/// Process an incoming TLS record
pub fn process_record(session: &mut TlsSession, data: &[u8]) -> Result<Option<Vec<u8>>, TlsError> {
    if data.len() < 5 {
        return Err(TlsError::ProtocolError);
    }
    
    let content_type = data[0];
    let _legacy_version = u16::from_be_bytes([data[1], data[2]]);
    let length = u16::from_be_bytes([data[3], data[4]]) as usize;
    
    if data.len() < 5 + length {
        return Err(TlsError::ProtocolError);
    }
    
    let record_data = &data[5..5 + length];
    
    match content_type {
        20 => {
            // ChangeCipherSpec (legacy, ignore in TLS 1.3)
            Ok(None)
        }
        21 => {
            // Alert
            if record_data.len() >= 2 {
                let level = record_data[0];
                let desc = record_data[1];
                crate::serial_println!("[TLS] Alert: level={} desc={}", level, desc);
                
                if desc == 0 {
                    // close_notify
                    session.state = TlsState::Closed;
                } else {
                    session.state = TlsState::Error;
                }
            }
            Err(TlsError::ConnectionClosed)
        }
        22 => {
            // Handshake (unencrypted)
            process_handshake(session, record_data)
        }
        23 => {
            // ApplicationData (encrypted)
            if session.state == TlsState::ServerHelloReceived {
                // This is encrypted handshake data
                process_encrypted_handshake(session, record_data)
            } else if session.state == TlsState::ApplicationData {
                // This is application data
                let plaintext = decrypt_record(session, record_data)?;
                Ok(Some(plaintext))
            } else {
                Err(TlsError::UnexpectedMessage)
            }
        }
        _ => Err(TlsError::ProtocolError),
    }
}

/// Process unencrypted handshake message
fn process_handshake(session: &mut TlsSession, data: &[u8]) -> Result<Option<Vec<u8>>, TlsError> {
    if data.is_empty() {
        return Err(TlsError::ProtocolError);
    }
    
    let msg_type = data[0];
    
    match msg_type {
        2 => {
            // ServerHello
            handshake::parse_server_hello(session, data)?;
            Ok(None)
        }
        _ => {
            crate::serial_println!("[TLS] Unexpected handshake type: {}", msg_type);
            Err(TlsError::UnexpectedMessage)
        }
    }
}

/// Process encrypted handshake messages
fn process_encrypted_handshake(session: &mut TlsSession, data: &[u8]) -> Result<Option<Vec<u8>>, TlsError> {
    // Decrypt the record
    let plaintext = decrypt_record(session, data)?;
    
    if plaintext.is_empty() {
        return Err(TlsError::ProtocolError);
    }
    
    // Find the actual content type (last byte before padding)
    let mut actual_content_type = 0u8;
    let mut content_end = plaintext.len();
    for i in (0..plaintext.len()).rev() {
        if plaintext[i] != 0 {
            actual_content_type = plaintext[i];
            content_end = i;
            break;
        }
    }
    
    if actual_content_type != ContentType::Handshake as u8 {
        if actual_content_type == ContentType::Alert as u8 {
            return Err(TlsError::ConnectionClosed);
        }
        return Err(TlsError::UnexpectedMessage);
    }
    
    let handshake_data = &plaintext[..content_end];
    
    // Process handshake messages (may be multiple in one record)
    let mut pos = 0;
    while pos + 4 <= handshake_data.len() {
        let msg_type = handshake_data[pos];
        let msg_len = ((handshake_data[pos + 1] as usize) << 16)
            | ((handshake_data[pos + 2] as usize) << 8)
            | (handshake_data[pos + 3] as usize);
        
        if pos + 4 + msg_len > handshake_data.len() {
            break;
        }
        
        let msg = &handshake_data[pos..pos + 4 + msg_len];
        
        match msg_type {
            8 => {
                // EncryptedExtensions
                handshake::parse_encrypted_extensions(session, msg)?;
            }
            11 => {
                // Certificate
                handshake::parse_certificate(session, msg)?;
            }
            15 => {
                // CertificateVerify
                handshake::parse_certificate_verify(session, msg)?;
            }
            20 => {
                // Finished
                handshake::parse_finished(session, msg)?;
                
                // Derive application secrets
                let mut transcript = session.transcript_hash.clone();
                let transcript_hash = transcript.finalize();
                handshake::derive_application_secrets(session, &transcript_hash)?;
                
                // Send client Finished
                let client_finished = handshake::build_client_finished(session);
                let encrypted = encrypt_record(session, ContentType::Handshake, &client_finished)?;
                
                session.state = TlsState::ApplicationData;
                
                return Ok(Some(encrypted));
            }
            4 => {
                // NewSessionTicket (after handshake, just update transcript)
                session.transcript_hash.update(msg);
            }
            _ => {
                crate::serial_println!("[TLS] Unknown encrypted handshake type: {}", msg_type);
            }
        }
        
        pos += 4 + msg_len;
    }
    
    Ok(None)
}

/// Encrypt a TLS record
pub fn encrypt_record(session: &mut TlsSession, content_type: ContentType, plaintext: &[u8]) -> Result<Vec<u8>, TlsError> {
    // Build inner plaintext: data + content type + padding
    let mut inner = Vec::with_capacity(plaintext.len() + 1);
    inner.extend_from_slice(plaintext);
    inner.push(content_type as u8);
    
    // Build nonce (IV XOR sequence number)
    let mut nonce = [0u8; 12];
    nonce.copy_from_slice(&session.client_write_iv);
    let seq_bytes = session.client_seq.to_be_bytes();
    for i in 0..8 {
        nonce[4 + i] ^= seq_bytes[i];
    }
    session.client_seq += 1;
    
    // Additional authenticated data (record header for TLS 1.3)
    let aad = [
        ContentType::ApplicationData as u8,
        0x03, 0x03, // Legacy version
        ((inner.len() + 16) >> 8) as u8,
        (inner.len() + 16) as u8,
    ];
    
    // Encrypt
    let ciphertext = aes_gcm_encrypt(&session.client_write_key, &nonce, &aad, &inner);
    
    // Build record
    let mut record = Vec::with_capacity(5 + ciphertext.len());
    record.push(ContentType::ApplicationData as u8);
    record.extend_from_slice(&[0x03, 0x03]); // Legacy version
    record.extend_from_slice(&(ciphertext.len() as u16).to_be_bytes());
    record.extend_from_slice(&ciphertext);
    
    Ok(record)
}

/// Decrypt a TLS record
pub fn decrypt_record(session: &mut TlsSession, ciphertext: &[u8]) -> Result<Vec<u8>, TlsError> {
    if ciphertext.len() < 16 {
        return Err(TlsError::DecryptionFailed);
    }
    
    // Build nonce
    let mut nonce = [0u8; 12];
    nonce.copy_from_slice(&session.server_write_iv);
    let seq_bytes = session.server_seq.to_be_bytes();
    for i in 0..8 {
        nonce[4 + i] ^= seq_bytes[i];
    }
    session.server_seq += 1;
    
    // AAD
    let aad = [
        ContentType::ApplicationData as u8,
        0x03, 0x03,
        (ciphertext.len() >> 8) as u8,
        ciphertext.len() as u8,
    ];
    
    // Decrypt
    let plaintext = aes_gcm_decrypt(&session.server_write_key, &nonce, &aad, ciphertext)
        .map_err(|_| TlsError::DecryptionFailed)?;
    
    Ok(plaintext)
}
