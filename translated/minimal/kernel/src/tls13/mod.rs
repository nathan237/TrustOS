






use alloc::vec::Vec;
use alloc::string::String;

pub mod crypto;
pub mod handshake;
pub mod record;
pub mod x509;


pub const EIJ_: u16 = 0x0303; 
pub const EIK_: u16 = 0x0304;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ContentType {
    Csx = 20,
    Bbo = 21,
    Atm = 22,
    Kd = 23,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum HandshakeType {
    Bzk = 1,
    Cmu = 2,
    Dda = 4,
    Cvw = 5,
    Cbq = 8,
    Certificate = 11,
    Csv = 13,
    Bzi = 15,
    Bhf = 20,
    Dat = 24,
    Dco = 254,
}


#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum AlertLevel {
    Oo = 1,
    Nd = 2,
}


#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum AlertDescription {
    Bzm = 0,
    Oj = 10,
    Crx = 20,
    Dft = 22,
    Cyh = 40,
    Crw = 42,
    Csu = 45,
    Csw = 46,
    Czn = 47,
    Cue = 50,
    Cuf = 51,
    Dey = 70,
    Cga = 80,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TlsState {
    Cfx,
    Bzl,
    Bsn,
    Cyg,
    Kd,
    Dk,
    Q,
}


#[derive(Debug, Clone, Copy)]
pub enum TlsError {
    
    Rv,
    
    Atn,
    
    Apv,
    
    Aqq,
    
    Oj,
    
    Fs,
    
    Cga,
    
    Ahe,
    
    Zn,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum CipherSuite {
    AIW_ = 0x1301,
    EIH_ = 0x1302,
    EII_ = 0x1303,
}


pub struct TlsSession {
    
    pub g: TlsState,
    
    
    pub ajc: String,
    
    
    pub ins: Option<CipherSuite>,
    
    
    inx: [u8; 32],
    
    
    pim: [u8; 32],
    
    
    ape: crypto::Sha256,
    
    
    hml: [u8; 32],
    inw: [u8; 32],
    jon: [u8; 32],
    kib: [u8; 32],
    mea: [u8; 32],
    
    
    ioa: [u8; 16],
    inz: [u8; 12],
    gse: [u8; 16],
    hzu: [u8; 12],
    
    
    iny: u64,
    hzt: u64,
    
    
    gfv: [u8; 32],
    kso: [u8; 32],
    
    
    ehx: Vec<u8>,
    
    
    fao: Vec<u8>,
    
    
    pub joo: Vec<u8>,
    
    
    pub pik: Vec<u8>,
}

impl TlsSession {
    
    pub fn new(ajc: &str) -> Self {
        let mut inx = [0u8; 32];
        crate::rng::ntq(&mut inx);
        
        
        let mut gfv = [0u8; 32];
        crate::rng::ntq(&mut gfv);
        let kso = crypto::xwd(&gfv);
        
        Self {
            g: TlsState::Cfx,
            ajc: String::from(ajc),
            ins: None,
            inx,
            pim: [0u8; 32],
            ape: crypto::Sha256::new(),
            hml: [0u8; 32],
            inw: [0u8; 32],
            jon: [0u8; 32],
            kib: [0u8; 32],
            mea: [0u8; 32],
            ioa: [0u8; 16],
            inz: [0u8; 12],
            gse: [0u8; 16],
            hzu: [0u8; 12],
            iny: 0,
            hzt: 0,
            gfv,
            kso,
            ehx: Vec::new(),
            fao: Vec::new(),
            joo: Vec::new(),
            pik: Vec::new(),
        }
    }
    
    
    pub fn kfj(&mut self) -> Vec<u8> {
        handshake::kfj(self)
    }
    
    
    pub fn jkd(&mut self, f: &[u8]) -> Result<Option<Vec<u8>>, TlsError> {
        record::jkd(self, f)
    }
    
    
    pub fn npy(&mut self, ajk: &[u8]) -> Result<Vec<u8>, TlsError> {
        if self.g != TlsState::Kd {
            return Err(TlsError::Fs);
        }
        
        record::kti(self, ContentType::Kd, ajk)
    }
    
    
    pub fn ruw(&mut self, afm: &[u8]) -> Result<Vec<u8>, TlsError> {
        if self.g != TlsState::Kd {
            return Err(TlsError::Fs);
        }
        
        record::kop(self, afm)
    }
    
    
    pub fn uc(&self) -> bool {
        self.g == TlsState::Kd
    }
    
    
    pub fn agj(&mut self) -> Vec<u8> {
        self.g = TlsState::Dk;
        
        let qgb = [AlertLevel::Oo as u8, AlertDescription::Bzm as u8];
        record::kti(self, ContentType::Bbo, &qgb).age()
    }
}


pub fn nmh<G, Aii>(
    he: &mut TlsSession,
    baq: &mut G,
    ehf: &mut Aii,
) -> Result<(), TlsError>
where
    G: FnMut(&[u8]) -> Result<(), TlsError>,
    Aii: FnMut(&mut [u8]) -> Result<usize, TlsError>,
{
    
    let rbq = he.kfj();
    baq(&rbq)?;
    he.g = TlsState::Bzl;
    
    
    let mut ccu: Vec<u8> = Vec::fc(32768);
    let mut paw = [0u8; 4096];
    
    loop {
        
        while ccu.len() >= 5 {
            
            let ahg = ccu[0];
            let dk = u16::oa([ccu[1], ccu[2]]);
            let lyh = u16::oa([ccu[3], ccu[4]]) as usize;
            let mml = 5 + lyh;
            
            crate::serial_println!("[TLS] Header: type={} ver=0x{:04x} len={}", ahg, dk, lyh);
            
            
            if ahg < 20 || ahg > 23 {
                crate::serial_println!("[TLS] Invalid content type {}, first 10 bytes: {:02x?}", 
                    ahg, &ccu[..ccu.len().v(10)]);
                return Err(TlsError::Fs);
            }
            
            if ccu.len() < mml {
                
                crate::serial_println!("[TLS] Need {} bytes, have {}", mml, ccu.len());
                break;
            }
            
            
            let ehd: Vec<u8> = ccu.bbk(..mml).collect();
            crate::serial_println!("[TLS] Processing record: type={} len={}", ehd[0], lyh);
            
            match he.jkd(&ehd) {
                Ok(Some(mk)) => {
                    if !mk.is_empty() {
                        baq(&mk)?;
                    }
                }
                Ok(None) => {}
                Err(aa) => {
                    crate::serial_println!("[TLS] Record error: {:?}", aa);
                    return Err(aa);
                }
            }
            
            if he.g == TlsState::Kd {
                return Ok(());
            }
            
            if he.g == TlsState::Q {
                return Err(TlsError::Atn);
            }
        }
        
        
        match ehf(&mut paw) {
            Ok(0) => {
                if ccu.is_empty() {
                    return Err(TlsError::Ahe);
                }
                
                continue;
            }
            Ok(bo) => {
                crate::serial_println!("[TLS] Received {} bytes, accumulator has {}", bo, ccu.len());
                ccu.bk(&paw[..bo]);
            }
            Err(TlsError::Zn) => {
                
                crate::serial_println!("[TLS] WouldBlock, accumulator has {} bytes", ccu.len());
                continue;
            }
            Err(aa) => {
                crate::serial_println!("[TLS] Recv error: {:?}", aa);
                return Err(aa);
            }
        }
    }
}
