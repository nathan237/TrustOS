//! TLS 1.3 Handshake Protocol
//!
//! Implements ClientHello, ServerHello parsing, and key derivation.

use alloc::vec::Vec;
use super::{TlsSession, TlsState, TlsError, ContentType, HandshakeType, CipherSuite};
use super::crypto::{self, Sha256, sha256, hkdf_extract, hkdf_expand_label, derive_secret};

/// Build ClientHello message
pub fn build_client_hello(session: &mut TlsSession) -> Vec<u8> {
    let mut handshake = Vec::new();
    
    // ===== Handshake Header =====
    handshake.push(HandshakeType::ClientHello as u8);
    
    // Length placeholder (3 bytes)
    let length_pos = handshake.len();
    handshake.extend_from_slice(&[0, 0, 0]);
    
    // ===== ClientHello Body =====
    
    // Legacy version (TLS 1.2)
    handshake.extend_from_slice(&[0x03, 0x03]);
    
    // Random (32 bytes)
    handshake.extend_from_slice(&session.client_random);
    
    // Legacy session ID (empty for TLS 1.3)
    handshake.push(0); // length = 0
    
    // Cipher suites (2 bytes length + suites)
    handshake.extend_from_slice(&[0x00, 0x02]); // 2 bytes
    handshake.extend_from_slice(&(CipherSuite::TLS_AES_128_GCM_SHA256 as u16).to_be_bytes());
    
    // Compression methods (must include null)
    handshake.push(0x01); // 1 method
    handshake.push(0x00); // null compression
    
    // ===== Extensions =====
    let extensions_start = handshake.len();
    handshake.extend_from_slice(&[0x00, 0x00]); // Extensions length placeholder
    
    // Extension: supported_versions (mandatory for TLS 1.3)
    handshake.extend_from_slice(&[0x00, 0x2b]); // extension type
    handshake.extend_from_slice(&[0x00, 0x03]); // extension length
    handshake.push(0x02); // versions length
    handshake.extend_from_slice(&[0x03, 0x04]); // TLS 1.3
    
    // Extension: supported_groups (for key exchange)
    handshake.extend_from_slice(&[0x00, 0x0a]); // extension type
    handshake.extend_from_slice(&[0x00, 0x04]); // extension length
    handshake.extend_from_slice(&[0x00, 0x02]); // groups length
    handshake.extend_from_slice(&[0x00, 0x1d]); // x25519
    
    // Extension: signature_algorithms
    handshake.extend_from_slice(&[0x00, 0x0d]); // extension type
    handshake.extend_from_slice(&[0x00, 0x04]); // extension length
    handshake.extend_from_slice(&[0x00, 0x02]); // algos length
    handshake.extend_from_slice(&[0x08, 0x04]); // rsa_pss_rsae_sha256
    
    // Extension: key_share (client's X25519 public key)
    handshake.extend_from_slice(&[0x00, 0x33]); // extension type
    let key_share_len = 2 + 2 + 2 + 32; // group + len + key
    handshake.extend_from_slice(&(key_share_len as u16).to_be_bytes());
    handshake.extend_from_slice(&((key_share_len - 2) as u16).to_be_bytes()); // client_shares length
    handshake.extend_from_slice(&[0x00, 0x1d]); // x25519 group
    handshake.extend_from_slice(&[0x00, 0x20]); // key length (32)
    handshake.extend_from_slice(&session.ecdhe_public);
    
    // Extension: server_name (SNI)
    if !session.hostname.is_empty() {
        let hostname_bytes = session.hostname.as_bytes();
        let sni_len = 2 + 1 + 2 + hostname_bytes.len();
        handshake.extend_from_slice(&[0x00, 0x00]); // extension type
        handshake.extend_from_slice(&(sni_len as u16).to_be_bytes());
        handshake.extend_from_slice(&((sni_len - 2) as u16).to_be_bytes()); // server_name_list length
        handshake.push(0x00); // host_name type
        handshake.extend_from_slice(&(hostname_bytes.len() as u16).to_be_bytes());
        handshake.extend_from_slice(hostname_bytes);
    }
    
    // Fix extensions length
    let extensions_len = (handshake.len() - extensions_start - 2) as u16;
    handshake[extensions_start] = (extensions_len >> 8) as u8;
    handshake[extensions_start + 1] = extensions_len as u8;
    
    // Fix handshake length
    let handshake_len = (handshake.len() - 4) as u32;
    handshake[length_pos] = (handshake_len >> 16) as u8;
    handshake[length_pos + 1] = (handshake_len >> 8) as u8;
    handshake[length_pos + 2] = handshake_len as u8;
    
    // Update transcript hash
    session.transcript_hash.update(&handshake);
    
    // Wrap in TLS record
    let mut record = Vec::new();
    record.push(ContentType::Handshake as u8);
    record.extend_from_slice(&[0x03, 0x01]); // Legacy version
    record.extend_from_slice(&(handshake.len() as u16).to_be_bytes());
    record.extend_from_slice(&handshake);
    
    record
}

/// Parse ServerHello and extract key share
pub fn parse_server_hello(session: &mut TlsSession, data: &[u8]) -> Result<(), TlsError> {
    if data.len() < 38 {
        return Err(TlsError::ProtocolError);
    }
    
    // Check handshake type
    if data[0] != HandshakeType::ServerHello as u8 {
        return Err(TlsError::UnexpectedMessage);
    }
    
    // Handshake length
    let length = ((data[1] as usize) << 16) | ((data[2] as usize) << 8) | (data[3] as usize);
    if data.len() < 4 + length {
        return Err(TlsError::ProtocolError);
    }
    
    // Update transcript hash with full ServerHello
    session.transcript_hash.update(&data[..4 + length]);
    
    // Legacy version (skip)
    let mut pos = 4;
    pos += 2;
    
    // Server random
    if pos + 32 > data.len() {
        return Err(TlsError::ProtocolError);
    }
    session.server_random.copy_from_slice(&data[pos..pos + 32]);
    pos += 32;
    
    // Legacy session ID
    if pos >= data.len() {
        return Err(TlsError::ProtocolError);
    }
    let session_id_len = data[pos] as usize;
    pos += 1 + session_id_len;
    
    // Cipher suite
    if pos + 2 > data.len() {
        return Err(TlsError::ProtocolError);
    }
    let cipher_suite = u16::from_be_bytes([data[pos], data[pos + 1]]);
    pos += 2;
    
    if cipher_suite == CipherSuite::TLS_AES_128_GCM_SHA256 as u16 {
        session.cipher_suite = Some(CipherSuite::TLS_AES_128_GCM_SHA256);
    } else {
        return Err(TlsError::HandshakeFailed);
    }
    
    // Compression (skip, must be 0)
    pos += 1;
    
    // Extensions
    if pos + 2 > data.len() {
        return Err(TlsError::ProtocolError);
    }
    let extensions_len = u16::from_be_bytes([data[pos], data[pos + 1]]) as usize;
    pos += 2;
    
    let extensions_end = pos + extensions_len;
    let mut server_public_key: Option<[u8; 32]> = None;
    
    while pos + 4 <= extensions_end {
        let ext_type = u16::from_be_bytes([data[pos], data[pos + 1]]);
        let ext_len = u16::from_be_bytes([data[pos + 2], data[pos + 3]]) as usize;
        pos += 4;
        
        if pos + ext_len > extensions_end {
            break;
        }
        
        match ext_type {
            0x0033 => {
                // key_share
                if ext_len >= 36 {
                    let group = u16::from_be_bytes([data[pos], data[pos + 1]]);
                    let key_len = u16::from_be_bytes([data[pos + 2], data[pos + 3]]) as usize;
                    
                    if group == 0x001d && key_len == 32 && pos + 4 + 32 <= extensions_end {
                        let mut key = [0u8; 32];
                        key.copy_from_slice(&data[pos + 4..pos + 4 + 32]);
                        server_public_key = Some(key);
                    }
                }
            }
            0x002b => {
                // supported_versions (should indicate TLS 1.3)
            }
            _ => {}
        }
        
        pos += ext_len;
    }
    
    // Compute shared secret
    let server_key = server_public_key.ok_or(TlsError::HandshakeFailed)?;
    
    crate::serial_println!("[TLS] Server public key: {:02x?}", &server_key[..16]);
    crate::serial_println!("[TLS] Client private key: {:02x?}", &session.ecdhe_private[..16]);
    
    let shared_secret = crypto::x25519(&session.ecdhe_private, &server_key);
    
    crate::serial_println!("[TLS] Shared secret: {:02x?}", &shared_secret[..16]);
    
    // Derive handshake secrets using HKDF
    derive_handshake_secrets(session, &shared_secret)?;
    
    session.state = TlsState::ServerHelloReceived;
    Ok(())
}

/// Derive handshake traffic secrets
fn derive_handshake_secrets(session: &mut TlsSession, shared_secret: &[u8; 32]) -> Result<(), TlsError> {
    // Early Secret = HKDF-Extract(salt=0, PSK=0)
    let zero_key = [0u8; 32];
    let early_secret = hkdf_extract(&[], &zero_key);
    
    // Derive-Secret(Early Secret, "derived", "")
    let empty_hash = sha256(&[]);
    let derived = derive_secret(&early_secret, "derived", &empty_hash);
    
    // Handshake Secret = HKDF-Extract(derived, shared_secret)
    session.handshake_secret = hkdf_extract(&derived, shared_secret);
    
    // Get transcript hash up to ServerHello
    let mut transcript = session.transcript_hash.clone();
    let transcript_hash = transcript.finalize();
    
    // Client Handshake Traffic Secret
    session.client_handshake_traffic_secret = derive_secret(
        &session.handshake_secret,
        "c hs traffic",
        &transcript_hash,
    );
    
    // Server Handshake Traffic Secret
    session.server_handshake_traffic_secret = derive_secret(
        &session.handshake_secret,
        "s hs traffic",
        &transcript_hash,
    );
    
    crate::serial_println!("[TLS] Handshake secrets derived");
    crate::serial_println!("[TLS] Transcript hash: {:02x?}", &transcript_hash[..16]);
    
    // Derive traffic keys for handshake
    derive_traffic_keys(
        &session.server_handshake_traffic_secret,
        &mut session.server_write_key,
        &mut session.server_write_iv,
    );
    
    crate::serial_println!("[TLS] Server write key: {:02x?}", &session.server_write_key[..8]);
    crate::serial_println!("[TLS] Server write IV: {:02x?}", &session.server_write_iv);
    
    derive_traffic_keys(
        &session.client_handshake_traffic_secret,
        &mut session.client_write_key,
        &mut session.client_write_iv,
    );
    
    Ok(())
}

/// Derive application traffic secrets (after handshake complete)
pub fn derive_application_secrets(session: &mut TlsSession, transcript_hash: &[u8; 32]) -> Result<(), TlsError> {
    // Derive-Secret(Handshake Secret, "derived", "")
    let empty_hash = sha256(&[]);
    let derived = derive_secret(&session.handshake_secret, "derived", &empty_hash);
    
    // Master Secret = HKDF-Extract(derived, 0)
    let zero_key = [0u8; 32];
    let master_secret = hkdf_extract(&derived, &zero_key);
    
    // Client/Server Application Traffic Secrets
    session.client_application_traffic_secret = derive_secret(
        &master_secret,
        "c ap traffic",
        transcript_hash,
    );
    
    session.server_application_traffic_secret = derive_secret(
        &master_secret,
        "s ap traffic",
        transcript_hash,
    );
    
    // Derive traffic keys for application data
    derive_traffic_keys(
        &session.server_application_traffic_secret,
        &mut session.server_write_key,
        &mut session.server_write_iv,
    );
    
    derive_traffic_keys(
        &session.client_application_traffic_secret,
        &mut session.client_write_key,
        &mut session.client_write_iv,
    );
    
    // Reset sequence numbers
    session.client_seq = 0;
    session.server_seq = 0;
    
    Ok(())
}

/// Derive traffic keys from a traffic secret
fn derive_traffic_keys(secret: &[u8; 32], key: &mut [u8; 16], iv: &mut [u8; 12]) {
    let key_bytes = hkdf_expand_label(secret, "key", &[], 16);
    let iv_bytes = hkdf_expand_label(secret, "iv", &[], 12);
    
    key.copy_from_slice(&key_bytes);
    iv.copy_from_slice(&iv_bytes);
}

/// Parse EncryptedExtensions message
pub fn parse_encrypted_extensions(session: &mut TlsSession, data: &[u8]) -> Result<(), TlsError> {
    if data.is_empty() || data[0] != HandshakeType::EncryptedExtensions as u8 {
        return Err(TlsError::UnexpectedMessage);
    }
    
    // Just update transcript hash, we don't need to process extensions
    session.transcript_hash.update(data);
    Ok(())
}

/// Parse Certificate message (minimal validation)
pub fn parse_certificate(session: &mut TlsSession, data: &[u8]) -> Result<(), TlsError> {
    if data.is_empty() || data[0] != HandshakeType::Certificate as u8 {
        return Err(TlsError::UnexpectedMessage);
    }
    
    // Update transcript hash
    session.transcript_hash.update(data);
    
    // TODO: Actually parse and validate the certificate chain
    // For now, we accept any certificate (INSECURE but functional)
    
    Ok(())
}

/// Parse CertificateVerify message
pub fn parse_certificate_verify(session: &mut TlsSession, data: &[u8]) -> Result<(), TlsError> {
    if data.is_empty() || data[0] != HandshakeType::CertificateVerify as u8 {
        return Err(TlsError::UnexpectedMessage);
    }
    
    // Update transcript hash
    session.transcript_hash.update(data);
    
    // TODO: Verify the signature
    // For now, we accept any signature (INSECURE but functional)
    
    Ok(())
}

/// Parse Finished message and verify
pub fn parse_finished(session: &mut TlsSession, data: &[u8]) -> Result<(), TlsError> {
    if data.is_empty() || data[0] != HandshakeType::Finished as u8 {
        return Err(TlsError::UnexpectedMessage);
    }
    
    if data.len() < 4 + 32 {
        return Err(TlsError::ProtocolError);
    }
    
    let verify_data = &data[4..4 + 32];
    
    // Compute expected verify_data
    let mut transcript = session.transcript_hash.clone();
    let transcript_hash = transcript.finalize();
    
    let finished_key = hkdf_expand_label(&session.server_handshake_traffic_secret, "finished", &[], 32);
    let mut finished_key_arr = [0u8; 32];
    finished_key_arr.copy_from_slice(&finished_key);
    
    let expected = crypto::hmac_sha256(&finished_key_arr, &transcript_hash);
    
    // Constant-time comparison
    let mut diff = 0u8;
    for i in 0..32 {
        diff |= verify_data[i] ^ expected[i];
    }
    
    if diff != 0 {
        return Err(TlsError::DecryptionFailed);
    }
    
    // Update transcript with Finished message
    session.transcript_hash.update(data);
    
    Ok(())
}

/// Build client Finished message
pub fn build_client_finished(session: &mut TlsSession) -> Vec<u8> {
    // Get current transcript hash
    let mut transcript = session.transcript_hash.clone();
    let transcript_hash = transcript.finalize();
    
    // Compute verify_data
    let finished_key = hkdf_expand_label(&session.client_handshake_traffic_secret, "finished", &[], 32);
    let mut finished_key_arr = [0u8; 32];
    finished_key_arr.copy_from_slice(&finished_key);
    
    let verify_data = crypto::hmac_sha256(&finished_key_arr, &transcript_hash);
    
    // Build Finished message
    let mut finished = Vec::new();
    finished.push(HandshakeType::Finished as u8);
    finished.extend_from_slice(&[0x00, 0x00, 0x20]); // length = 32
    finished.extend_from_slice(&verify_data);
    
    // Update transcript
    session.transcript_hash.update(&finished);
    
    finished
}
