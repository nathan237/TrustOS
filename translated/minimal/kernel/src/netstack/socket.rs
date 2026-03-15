




use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use alloc::boxed::Box;
use core::sync::atomic::{AtomicI32, AtomicU16, Ordering};
use spin::Mutex;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum AddressFamily {
    Bvk = 0,
    Cow = 1,    
    Aje = 2,    
    Cfv = 10,  
}

impl From<u16> for AddressFamily {
    fn from(p: u16) -> Self {
        match p {
            0 => Self::Bvk,
            1 => Self::Cow,
            2 => Self::Aje,
            10 => Self::Cfv,
            _ => Self::Bvk,
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum SocketType {
    Qw = 1,     
    Abb = 2,      
    Axl = 3,        
}

impl From<u32> for SocketType {
    fn from(p: u32) -> Self {
        match p {
            1 => Self::Qw,
            2 => Self::Abb,
            3 => Self::Axl,
            _ => Self::Qw,
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SocketState {
    Cu,
    Vq,
    Blk,
    Aas,
    Dl,
    Dk,
}


#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct SockAddrIn {
    pub wot: u16,      
    pub pla: u16,        
    pub pky: u32,        
    pub wou: [u8; 8],    
}

impl SockAddrIn {
    pub const Am: usize = 16;
    
    pub fn new(ip: [u8; 4], port: u16) -> Self {
        Self {
            wot: AddressFamily::Aje as u16,
            pla: port.zsu(),
            pky: u32::oa(ip),
            wou: [0; 8],
        }
    }
    
    pub fn ip(&self) -> [u8; 4] {
        self.pky.ft()
    }
    
    pub fn port(&self) -> u16 {
        u16::eqv(self.pla)
    }
}


#[derive(Debug)]
pub struct Socket {
    pub family: AddressFamily,
    pub bif: SocketType,
    pub protocol: u32,
    pub g: SocketState,
    
    
    pub ljn: Option<SockAddrIn>,
    pub ahq: u16,
    
    
    pub exp: Option<SockAddrIn>,
    
    
    pub fwi: u16,
    
    
    pub ehx: Vec<u8>,
    pub wbs: bool,
    
    
    pub fao: Vec<u8>,
    
    
    pub oqv: bool,
    
    
    pub dea: u32,
    pub vgq: Vec<SockAddrIn>,
}

impl Socket {
    pub fn new(family: AddressFamily, bif: SocketType, protocol: u32) -> Self {
        Self {
            family,
            bif,
            protocol,
            g: SocketState::Cu,
            ljn: None,
            ahq: 0,
            exp: None,
            fwi: 0,
            ehx: Vec::new(),
            wbs: false,
            fao: Vec::new(),
            oqv: false,
            dea: 0,
            vgq: Vec::new(),
        }
    }
}


pub static BA_: Mutex<BTreeMap<i32, Socket>> = Mutex::new(BTreeMap::new());
static BBK_: AtomicI32 = AtomicI32::new(100); 
static VS_: AtomicU16 = AtomicU16::new(49152);


pub fn socket(vh: u16, bif: u32, protocol: u32) -> Result<i32, i32> {
    let family = AddressFamily::from(vh);
    let pph = SocketType::from(bif & 0xFF); 
    
    
    if family != AddressFamily::Aje {
        crate::serial_println!("[SOCKET] Only AF_INET supported");
        return Err(-22); 
    }
    
    let su = Socket::new(family, pph, protocol);
    let da = BBK_.fetch_add(1, Ordering::Relaxed);
    
    BA_.lock().insert(da, su);
    
    crate::serial_println!("[SOCKET] Created socket fd={} type={:?}", da, pph);
    Ok(da)
}


pub fn kdj(da: i32, ag: &SockAddrIn) -> Result<(), i32> {
    let mut gg = BA_.lock();
    let su = gg.ds(&da).ok_or(-9)?; 
    
    if su.g != SocketState::Cu {
        return Err(-22); 
    }
    
    su.ljn = Some(*ag);
    su.ahq = ag.port();
    su.g = SocketState::Vq;
    
    crate::serial_println!("[SOCKET] Bound fd={} to port {}", da, ag.port());
    Ok(())
}


pub fn ojr(da: i32, dea: u32) -> Result<(), i32> {
    let mut gg = BA_.lock();
    let su = gg.ds(&da).ok_or(-9)?;
    
    if su.bif != SocketType::Qw {
        return Err(-95); 
    }
    
    if su.g != SocketState::Vq {
        return Err(-22); 
    }
    
    su.g = SocketState::Blk;
    su.dea = dea.am(1);
    let port = su.ahq;
    let bl = su.dea;
    drop(gg);

    
    crate::netstack::tcp::jdt(port, bl);
    crate::serial_println!("[SOCKET] Listening fd={} port={} backlog={}", da, port, bl);
    Ok(())
}


pub fn qes(da: i32, azf: u64, bye: u64) -> Result<i32, i32> {
    let fnd = {
        let gg = BA_.lock();
        let su = gg.get(&da).ok_or(-9i32)?; 
        if su.g != SocketState::Blk {
            return Err(-22); 
        }
        su.ahq
    };

    
    for _ in 0..2000 {
        crate::netstack::poll();

        if let Some((ey, ams, bci)) =
            crate::netstack::tcp::iir(fnd)
        {
            let anp = BBK_.fetch_add(1, Ordering::Relaxed);
            let mut hss = Socket::new(AddressFamily::Aje, SocketType::Qw, 0);
            hss.g = SocketState::Dl;
            hss.ahq = ey;
            hss.fwi = ey;
            hss.exp = Some(SockAddrIn::new(ams, bci));
            BA_.lock().insert(anp, hss);

            
            if azf != 0 && bye != 0 {
                let bcm = SockAddrIn::new(ams, bci);
                if crate::memory::sw(azf, core::mem::size_of::<SockAddrIn>(), true) {
                    unsafe { *(azf as *mut SockAddrIn) = bcm; }
                }
                if crate::memory::sw(bye, 4, true) {
                    unsafe { *(bye as *mut u32) = core::mem::size_of::<SockAddrIn>() as u32; }
                }
            }

            crate::serial_println!(
                "[SOCKET] accept fd={} -> new_fd={} remote={}:{}",
                da, anp,
                ams.iter().map(|o| alloc::format!("{}", o)).collect::<alloc::vec::Vec<_>>().rr("."),
                bci
            );
            return Ok(anp);
        }

        
        for _ in 0..5000 { core::hint::hc(); }
    }

    Err(-11) 
}


pub fn ipa(da: i32, ag: &SockAddrIn) -> Result<(), i32> {
    
    let (bif, ahq) = {
        let mut gg = BA_.lock();
        let su = gg.ds(&da).ok_or(-9)?;
        
        if su.g != SocketState::Cu && su.g != SocketState::Vq {
            return Err(-106); 
        }
        
        
        if su.ahq == 0 {
            su.ahq = VS_.fetch_add(1, Ordering::Relaxed);
        }
        
        su.exp = Some(*ag);
        su.g = SocketState::Aas;
        
        (su.bif, su.ahq)
    };
    
    let kv = ag.ip();
    let rz = ag.port();
    
    crate::serial_println!(
        "[SOCKET] Connecting fd={} to {}.{}.{}.{}:{}",
        da, kv[0], kv[1], kv[2], kv[3], rz
    );
    
    if bif == SocketType::Qw {
        
        match crate::netstack::tcp::cue(kv, rz) {
            Ok(ey) => {
                let mut gg = BA_.lock();
                if let Some(su) = gg.ds(&da) {
                    su.fwi = ey;
                }
            }
            Err(aa) => {
                crate::serial_println!("[SOCKET] SYN failed: {}", aa);
                return Err(-111); 
            }
        }
        
        
        for _ in 0..1000 {
            crate::netstack::poll();
            
            
            if crate::netstack::tcp::lfz(kv, rz) {
                let mut gg = BA_.lock();
                if let Some(su) = gg.ds(&da) {
                    su.g = SocketState::Dl;
                }
                crate::serial_println!("[SOCKET] Connected fd={}", da);
                return Ok(());
            }
            
            
            for _ in 0..10000 { core::hint::hc(); }
        }
        
        Err(-110) 
    } else {
        
        let mut gg = BA_.lock();
        if let Some(su) = gg.ds(&da) {
            su.g = SocketState::Dl;
        }
        Ok(())
    }
}


pub fn baq(da: i32, f: &[u8], ddp: u32) -> Result<usize, i32> {
    let (bif, bwq, mkc, ahq) = {
        let gg = BA_.lock();
        let su = gg.get(&da).ok_or(-9)?;
        
        if su.g != SocketState::Dl {
            return Err(-107); 
        }
        
        let bwq = su.exp.ok_or(-89)?; 
        (su.bif, bwq, su.fwi, su.ahq)
    };
    
    
    if f.len() > 65507 { 
        return Err(-90); 
    }
    
    match bif {
        SocketType::Qw => {
            
            crate::netstack::tcp::fuf(bwq.ip(), bwq.port(), mkc, f)
                .jd(|_| -104)?; 
            Ok(f.len())
        }
        SocketType::Abb => {
            
            crate::netstack::udp::dlp(bwq.ip(), bwq.port(), ahq, f)
                .jd(|_| -101)?; 
            Ok(f.len())
        }
        _ => Err(-95), 
    }
}


pub fn ehf(da: i32, k: &mut [u8], ddp: u32) -> Result<usize, i32> {
    let (bif, bwq, mkc) = {
        let gg = BA_.lock();
        let su = gg.get(&da).ok_or(-9)?;
        
        if su.g != SocketState::Dl {
            return Err(-107); 
        }
        
        let bwq = su.exp.ok_or(-107)?;
        (su.bif, bwq, su.fwi)
    };
    
    
    crate::netstack::poll();
    
    match bif {
        SocketType::Qw => {
            
            let f = crate::netstack::tcp::pam(bwq.ip(), bwq.port(), mkc);
            
            if let Some(f) = f {
                let len = f.len().v(k.len());
                k[..len].dg(&f[..len]);
                Ok(len)
            } else {
                
                Err(-11) 
            }
        }
        SocketType::Abb => {
            let ahq = {
                let gg = BA_.lock();
                let su = gg.get(&da).ok_or(-9)?;
                su.ahq
            };
            if let Some(f) = crate::netstack::udp::jlt(ahq) {
                let len = f.len().v(k.len());
                k[..len].dg(&f[..len]);
                Ok(len)
            } else {
                Err(-11) 
            }
        }
        _ => Err(-95),
    }
}


pub fn agj(da: i32) -> Result<(), i32> {
    let su = BA_.lock().remove(&da).ok_or(-9)?;
    
    
    if su.bif == SocketType::Qw && su.g == SocketState::Dl {
        if let Some(bwq) = su.exp {
            let _ = crate::netstack::tcp::bwx(bwq.ip(), bwq.port(), su.fwi);
        }
    }
    
    crate::serial_println!("[SOCKET] Closed fd={}", da);
    Ok(())
}


pub fn whr(da: i32, f: &[u8], ddp: u32, ag: &SockAddrIn) -> Result<usize, i32> {
    let (bif, ahq) = {
        let mut gg = BA_.lock();
        let su = gg.ds(&da).ok_or(-9)?;
        
        
        if su.ahq == 0 {
            su.ahq = VS_.fetch_add(1, Ordering::Relaxed);
        }
        
        (su.bif, su.ahq)
    };
    
    if bif != SocketType::Abb {
        return Err(-95); 
    }
    
    
    if f.len() > 65507 {
        return Err(-90); 
    }
    
    crate::netstack::udp::dlp(ag.ip(), ag.port(), ahq, f)
        .jd(|_| -101)?;
    
    Ok(f.len())
}


pub fn zix(da: i32, k: &mut [u8], ddp: u32, jzm: Option<&mut SockAddrIn>) -> Result<usize, i32> {
    let ahq = {
        let gg = BA_.lock();
        let su = gg.get(&da).ok_or(-9)?;
        
        if su.bif != SocketType::Abb {
            return Err(-95);
        }
        
        su.ahq
    };
    
    
    crate::netstack::poll();
    
    if let Some((f, jh, ey)) = crate::netstack::udp::vtk(ahq) {
        let len = f.len().v(k.len());
        k[..len].dg(&f[..len]);
        if let Some(jzm) = jzm {
            *jzm = SockAddrIn::new(jh, ey);
        }
        Ok(len)
    } else {
        Err(-11) 
    }
}


pub fn wkd(da: i32, jy: i32, evr: i32, ctc: &[u8]) -> Result<(), i32> {
    let mut gg = BA_.lock();
    let su = gg.ds(&da).ok_or(-9)?;
    
    
    
    
    
    match (jy, evr) {
        (1, 2) => {
            
            Ok(())
        }
        (1, 4) => {
            
            su.oqv = ctc.fv().hu().unwrap_or(0) != 0;
            Ok(())
        }
        _ => {
            crate::serial_println!("[SOCKET] Unknown sockopt level={} name={}", jy, evr);
            Ok(()) 
        }
    }
}


pub fn tfj(da: i32, jy: i32, evr: i32, ctc: &mut [u8]) -> Result<usize, i32> {
    let gg = BA_.lock();
    let su = gg.get(&da).ok_or(-9)?;
    
    match (jy, evr) {
        (1, 2) => {
            
            if !ctc.is_empty() {
                ctc[0] = 1;
            }
            Ok(4)
        }
        _ => {
            if !ctc.is_empty() {
                ctc[0] = 0;
            }
            Ok(4)
        }
    }
}


pub fn tyx(da: i32) -> bool {
    BA_.lock().bgm(&da)
}


pub fn drd(da: i32) -> Option<SocketState> {
    BA_.lock().get(&da).map(|e| e.g)
}


pub fn tna(da: i32) -> bool {
    let gg = BA_.lock();
    if let Some(su) = gg.get(&da) {
        if su.g == SocketState::Dl {
            if let Some(ag) = &su.exp {
                return crate::netstack::tcp::pam(ag.ip(), ag.port(), su.fwi).is_some();
            }
        }
    }
    false
}
