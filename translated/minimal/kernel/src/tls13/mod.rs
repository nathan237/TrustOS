






use alloc::vec::Vec;
use alloc::string::String;

pub mod crypto;
pub mod handshake;
pub mod record;
pub mod x509;


pub const ELY_: u16 = 0x0303; 
pub const ELZ_: u16 = 0x0304;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ContentType {
    ChangeCipherSpec = 20,
    Alert = 21,
    Handshake = 22,
    ApplicationData = 23,
}


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


#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum AlertLevel {
    Warning = 1,
    Fatal = 2,
}


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


#[derive(Debug, Clone, Copy)]
pub enum TlsError {
    
    ConnectionFailed,
    
    HandshakeFailed,
    
    CertificateInvalid,
    
    DecryptionFailed,
    
    UnexpectedMessage,
    
    ProtocolError,
    
    InternalError,
    
    ConnectionClosed,
    
    WouldBlock,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum CipherSuite {
    TLS_AES_128_GCM_SHA256 = 0x1301,
    TLS_AES_256_GCM_SHA384 = 0x1302,
    TLS_CHACHA20_POLY1305_SHA256 = 0x1303,
}


pub struct TlsSession {
    
    pub state: TlsState,
    
    
    pub hostname: String,
    
    
    pub cipher_suite: Option<CipherSuite>,
    
    
    client_random: [u8; 32],
    
    
    server_random: [u8; 32],
    
    
    transcript_hash: crypto::Sha256,
    
    
    handshake_secret: [u8; 32],
    client_handshake_traffic_secret: [u8; 32],
    server_handshake_traffic_secret: [u8; 32],
    client_application_traffic_secret: [u8; 32],
    server_application_traffic_secret: [u8; 32],
    
    
    client_write_key: [u8; 16],
    client_write_iv: [u8; 12],
    server_write_key: [u8; 16],
    server_write_iv: [u8; 12],
    
    
    client_seq: u64,
    server_seq: u64,
    
    
    ecdhe_private: [u8; 32],
    ecdhe_public: [u8; 32],
    
    
    rx_buffer: Vec<u8>,
    
    
    tx_buffer: Vec<u8>,
    
    
    pub server_pubkey: Vec<u8>,
    
    
    pub server_pubkey_algo: Vec<u8>,
}

impl TlsSession {
    
    pub fn new(hostname: &str) -> Self {
        let mut client_random = [0u8; 32];
        crate::rng::hyj(&mut client_random);
        
        
        let mut ecdhe_private = [0u8; 32];
        crate::rng::hyj(&mut ecdhe_private);
        let ecdhe_public = crypto::pvn(&ecdhe_private);
        
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
            server_pubkey: Vec::new(),
            server_pubkey_algo: Vec::new(),
        }
    }
    
    
    pub fn build_client_hello(&mut self) -> Vec<u8> {
        handshake::build_client_hello(self)
    }
    
    
    pub fn process_record(&mut self, data: &[u8]) -> Result<Option<Vec<u8>>, TlsError> {
        record::process_record(self, data)
    }
    
    
    pub fn encrypt(&mut self, ry: &[u8]) -> Result<Vec<u8>, TlsError> {
        if self.state != TlsState::ApplicationData {
            return Err(TlsError::ProtocolError);
        }
        
        record::fuq(self, ContentType::ApplicationData, ry)
    }
    
    
    pub fn lcw(&mut self, pw: &[u8]) -> Result<Vec<u8>, TlsError> {
        if self.state != TlsState::ApplicationData {
            return Err(TlsError::ProtocolError);
        }
        
        record::frc(self, pw)
    }
    
    
    pub fn is_ready(&self) -> bool {
        self.state == TlsState::ApplicationData
    }
    
    
    pub fn close(&mut self) -> Vec<u8> {
        self.state = TlsState::Closed;
        
        let jug = [AlertLevel::Warning as u8, AlertDescription::CloseNotify as u8];
        record::fuq(self, ContentType::Alert, &jug).unwrap_or_default()
    }
}


pub fn hsy<F, G>(
    by: &mut TlsSession,
    send: &mut F,
    recv: &mut G,
) -> Result<(), TlsError>
where
    F: FnMut(&[u8]) -> Result<(), TlsError>,
    G: FnMut(&mut [u8]) -> Result<usize, TlsError>,
{
    
    let klc = by.build_client_hello();
    send(&klc)?;
    by.state = TlsState::ClientHelloSent;
    
    
    let mut apo: Vec<u8> = Vec::with_capacity(32768);
    let mut iyx = [0u8; 4096];
    
    loop {
        
        while apo.len() >= 5 {
            
            let content_type = apo[0];
            let version = u16::from_be_bytes([apo[1], apo[2]]);
            let gqr = u16::from_be_bytes([apo[3], apo[4]]) as usize;
            let gzt = 5 + gqr;
            
            crate::serial_println!("[TLS] Header: type={} ver=0x{:04x} len={}", content_type, version, gqr);
            
            
            if content_type < 20 || content_type > 23 {
                crate::serial_println!("[TLS] Invalid content type {}, first 10 bytes: {:02x?}", 
                    content_type, &apo[..apo.len().min(10)]);
                return Err(TlsError::ProtocolError);
            }
            
            if apo.len() < gzt {
                
                crate::serial_println!("[TLS] Need {} bytes, have {}", gzt, apo.len());
                break;
            }
            
            
            let bvf: Vec<u8> = apo.drain(..gzt).collect();
            crate::serial_println!("[TLS] Processing record: type={} len={}", bvf[0], gqr);
            
            match by.process_record(&bvf) {
                Ok(Some(fa)) => {
                    if !fa.is_empty() {
                        send(&fa)?;
                    }
                }
                Ok(None) => {}
                Err(e) => {
                    crate::serial_println!("[TLS] Record error: {:?}", e);
                    return Err(e);
                }
            }
            
            if by.state == TlsState::ApplicationData {
                return Ok(());
            }
            
            if by.state == TlsState::Error {
                return Err(TlsError::HandshakeFailed);
            }
        }
        
        
        match recv(&mut iyx) {
            Ok(0) => {
                if apo.is_empty() {
                    return Err(TlsError::ConnectionClosed);
                }
                
                continue;
            }
            Ok(ae) => {
                crate::serial_println!("[TLS] Received {} bytes, accumulator has {}", ae, apo.len());
                apo.extend_from_slice(&iyx[..ae]);
            }
            Err(TlsError::WouldBlock) => {
                
                crate::serial_println!("[TLS] WouldBlock, accumulator has {} bytes", apo.len());
                continue;
            }
            Err(e) => {
                crate::serial_println!("[TLS] Recv error: {:?}", e);
                return Err(e);
            }
        }
    }
}
