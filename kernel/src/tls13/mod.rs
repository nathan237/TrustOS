//! Pure Rust TLS 1.3 Implementation (no_std compatible)
//!
//! Minimal TLS 1.3 for HTTPS support in TrustOS.
//! Implements only the essential cipher suite: TLS_AES_128_GCM_SHA256
//!
//! Security Note: This runs in userspace via syscalls, not in kernel.

use alloc::vec::Vec;
use alloc::string::String;

pub mod crypto;
pub mod handshake;
pub mod record;
pub mod x509;

/// TLS version
pub const TLS_VERSION_1_3: u16 = 0x0303; // Legacy version in record layer
pub const TLS_VERSION_1_3_DRAFT: u16 = 0x0304;

/// TLS content types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ContentType {
    ChangeCipherSpec = 20,
    Alert = 21,
    Handshake = 22,
    ApplicationData = 23,
}

/// TLS handshake types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum HandshakeType {
    ClientHello = 1,
    ServerHello = 2,
    NewSessionTicket = 4,
    EndOfEarlyData = 5,
    EncryptedExtensions = 8,
    Certificate = 11,
    CertificateRequest = 13,
    CertificateVerify = 15,
    Finished = 20,
    KeyUpdate = 24,
    MessageHash = 254,
}

/// TLS alert levels
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum AlertLevel {
    Warning = 1,
    Fatal = 2,
}

/// TLS alert descriptions
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum AlertDescription {
    CloseNotify = 0,
    UnexpectedMessage = 10,
    BadRecordMac = 20,
    RecordOverflow = 22,
    HandshakeFailure = 40,
    BadCertificate = 42,
    CertificateExpired = 45,
    CertificateUnknown = 46,
    IllegalParameter = 47,
    DecodeError = 50,
    DecryptError = 51,
    ProtocolVersion = 70,
    InternalError = 80,
}

/// TLS connection state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TlsState {
    Initial,
    ClientHelloSent,
    ServerHelloReceived,
    HandshakeComplete,
    ApplicationData,
    Closed,
    Error,
}

/// TLS error types
#[derive(Debug, Clone, Copy)]
pub enum TlsError {
    /// Connection failed
    ConnectionFailed,
    /// Handshake failed
    HandshakeFailed,
    /// Certificate verification failed
    CertificateInvalid,
    /// Decryption failed
    DecryptionFailed,
    /// Unexpected message
    UnexpectedMessage,
    /// Protocol error
    ProtocolError,
    /// Internal error
    InternalError,
    /// Connection closed
    ConnectionClosed,
    /// Would block (non-blocking mode)
    WouldBlock,
}

/// Cipher suite identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum CipherSuite {
    TLS_AES_128_GCM_SHA256 = 0x1301,
    TLS_AES_256_GCM_SHA384 = 0x1302,
    TLS_CHACHA20_POLY1305_SHA256 = 0x1303,
}

/// TLS session for a connection
pub struct TlsSession {
    /// Current state
    pub state: TlsState,
    
    /// Server hostname (for SNI)
    pub hostname: String,
    
    /// Negotiated cipher suite
    pub cipher_suite: Option<CipherSuite>,
    
    /// Client random (32 bytes)
    client_random: [u8; 32],
    
    /// Server random (32 bytes)
    server_random: [u8; 32],
    
    /// Handshake transcript hash
    transcript_hash: crypto::Sha256,
    
    /// Key schedule
    handshake_secret: [u8; 32],
    client_handshake_traffic_secret: [u8; 32],
    server_handshake_traffic_secret: [u8; 32],
    client_application_traffic_secret: [u8; 32],
    server_application_traffic_secret: [u8; 32],
    
    /// Traffic keys
    client_write_key: [u8; 16],
    client_write_iv: [u8; 12],
    server_write_key: [u8; 16],
    server_write_iv: [u8; 12],
    
    /// Sequence numbers for AEAD
    client_seq: u64,
    server_seq: u64,
    
    /// X25519 ephemeral key pair
    ecdhe_private: [u8; 32],
    ecdhe_public: [u8; 32],
    
    /// Receive buffer for fragmented records
    rx_buffer: Vec<u8>,
    
    /// Send buffer
    tx_buffer: Vec<u8>,
}

impl TlsSession {
    /// Create a new TLS session
    pub fn new(hostname: &str) -> Self {
        let mut client_random = [0u8; 32];
        crate::rng::fill_bytes(&mut client_random);
        
        // Generate X25519 key pair
        let mut ecdhe_private = [0u8; 32];
        crate::rng::fill_bytes(&mut ecdhe_private);
        let ecdhe_public = crypto::x25519_base(&ecdhe_private);
        
        Self {
            state: TlsState::Initial,
            hostname: String::from(hostname),
            cipher_suite: None,
            client_random,
            server_random: [0u8; 32],
            transcript_hash: crypto::Sha256::new(),
            handshake_secret: [0u8; 32],
            client_handshake_traffic_secret: [0u8; 32],
            server_handshake_traffic_secret: [0u8; 32],
            client_application_traffic_secret: [0u8; 32],
            server_application_traffic_secret: [0u8; 32],
            client_write_key: [0u8; 16],
            client_write_iv: [0u8; 12],
            server_write_key: [0u8; 16],
            server_write_iv: [0u8; 12],
            client_seq: 0,
            server_seq: 0,
            ecdhe_private,
            ecdhe_public,
            rx_buffer: Vec::new(),
            tx_buffer: Vec::new(),
        }
    }
    
    /// Build ClientHello message
    pub fn build_client_hello(&mut self) -> Vec<u8> {
        handshake::build_client_hello(self)
    }
    
    /// Process incoming TLS record
    pub fn process_record(&mut self, data: &[u8]) -> Result<Option<Vec<u8>>, TlsError> {
        record::process_record(self, data)
    }
    
    /// Encrypt and send application data
    pub fn encrypt(&mut self, plaintext: &[u8]) -> Result<Vec<u8>, TlsError> {
        if self.state != TlsState::ApplicationData {
            return Err(TlsError::ProtocolError);
        }
        
        record::encrypt_record(self, ContentType::ApplicationData, plaintext)
    }
    
    /// Decrypt received application data
    pub fn decrypt(&mut self, ciphertext: &[u8]) -> Result<Vec<u8>, TlsError> {
        if self.state != TlsState::ApplicationData {
            return Err(TlsError::ProtocolError);
        }
        
        record::decrypt_record(self, ciphertext)
    }
    
    /// Check if handshake is complete
    pub fn is_ready(&self) -> bool {
        self.state == TlsState::ApplicationData
    }
    
    /// Close the connection
    pub fn close(&mut self) -> Vec<u8> {
        self.state = TlsState::Closed;
        // Send close_notify alert
        let alert = [AlertLevel::Warning as u8, AlertDescription::CloseNotify as u8];
        record::encrypt_record(self, ContentType::Alert, &alert).unwrap_or_default()
    }
}

/// Perform TLS handshake over a socket
pub fn do_handshake<F, G>(
    session: &mut TlsSession,
    send: &mut F,
    recv: &mut G,
) -> Result<(), TlsError>
where
    F: FnMut(&[u8]) -> Result<(), TlsError>,
    G: FnMut(&mut [u8]) -> Result<usize, TlsError>,
{
    // Send ClientHello
    let client_hello = session.build_client_hello();
    send(&client_hello)?;
    session.state = TlsState::ClientHelloSent;
    
    // Receive and process server messages
    let mut buf = [0u8; 16384];
    loop {
        let n = recv(&mut buf)?;
        if n == 0 {
            return Err(TlsError::ConnectionClosed);
        }
        
        // Process received data
        if let Some(response) = session.process_record(&buf[..n])? {
            if !response.is_empty() {
                send(&response)?;
            }
        }
        
        if session.state == TlsState::ApplicationData {
            return Ok(());
        }
        
        if session.state == TlsState::Error {
            return Err(TlsError::HandshakeFailed);
        }
    }
}
