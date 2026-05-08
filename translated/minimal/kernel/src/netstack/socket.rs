




use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use alloc::boxed::Box;
use core::sync::atomic::{AtomicI32, AtomicU16, Ordering};
use spin::Mutex;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum AddressFamily {
    Unspec = 0,
    Unix = 1,    
    Inet = 2,    
    Inet6 = 10,  
}

impl From<u16> for AddressFamily {
    fn from(v: u16) -> Self {
        match v {
            0 => Self::Unspec,
            1 => Self::Unix,
            2 => Self::Inet,
            10 => Self::Inet6,
            _ => Self::Unspec,
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum SocketType {
    Stream = 1,     
    Dgram = 2,      
    Raw = 3,        
}

impl From<u32> for SocketType {
    fn from(v: u32) -> Self {
        match v {
            1 => Self::Stream,
            2 => Self::Dgram,
            3 => Self::Raw,
            _ => Self::Stream,
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SocketState {
    Created,
    Bound,
    Listening,
    Connecting,
    Connected,
    Closed,
}


#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct SockAddrIn {
    pub sin_family: u16,      
    pub sin_port: u16,        
    pub sin_addr: u32,        
    pub sin_zero: [u8; 8],    
}

impl SockAddrIn {
    pub const Z: usize = 16;
    
    pub fn new(ip: [u8; 4], port: u16) -> Self {
        Self {
            sin_family: AddressFamily::Inet as u16,
            sin_port: port.to_be(),
            sin_addr: u32::from_be_bytes(ip),
            sin_zero: [0; 8],
        }
    }
    
    pub fn ip(&self) -> [u8; 4] {
        self.sin_addr.to_be_bytes()
    }
    
    pub fn port(&self) -> u16 {
        u16::from_be(self.sin_port)
    }
}


#[derive(Debug)]
pub struct Socket {
    pub family: AddressFamily,
    pub sock_type: SocketType,
    pub protocol: u32,
    pub state: SocketState,
    
    
    pub local_addr: Option<SockAddrIn>,
    pub local_port: u16,
    
    
    pub remote_addr: Option<SockAddrIn>,
    
    
    pub tcp_src_port: u16,
    
    
    pub rx_buffer: Vec<u8>,
    pub rx_closed: bool,
    
    
    pub tx_buffer: Vec<u8>,
    
    
    pub non_blocking: bool,
    
    
    pub backlog: u32,
    pub pending_connections: Vec<SockAddrIn>,
}

impl Socket {
    pub fn new(family: AddressFamily, sock_type: SocketType, protocol: u32) -> Self {
        Self {
            family,
            sock_type,
            protocol,
            state: SocketState::Created,
            local_addr: None,
            local_port: 0,
            remote_addr: None,
            tcp_src_port: 0,
            rx_buffer: Vec::new(),
            rx_closed: false,
            tx_buffer: Vec::new(),
            non_blocking: false,
            backlog: 0,
            pending_connections: Vec::new(),
        }
    }
}


pub static BC_: Mutex<BTreeMap<i32, Socket>> = Mutex::new(BTreeMap::new());
static BDN_: AtomicI32 = AtomicI32::new(100); 
static XB_: AtomicU16 = AtomicU16::new(49152);


pub fn socket(domain: u16, sock_type: u32, protocol: u32) -> Result<i32, i32> {
    let family = AddressFamily::from(domain);
    let jjj = SocketType::from(sock_type & 0xFF); 
    
    
    if family != AddressFamily::Inet {
        crate::serial_println!("[SOCKET] Only AF_INET supported");
        return Err(-22); 
    }
    
    let ih = Socket::new(family, jjj, protocol);
    let fd = BDN_.fetch_add(1, Ordering::Relaxed);
    
    BC_.lock().insert(fd, ih);
    
    crate::serial_println!("[SOCKET] Created socket fd={} type={:?}", fd, jjj);
    Ok(fd)
}


pub fn fjf(fd: i32, addr: &SockAddrIn) -> Result<(), i32> {
    let mut bs = BC_.lock();
    let ih = bs.get_mut(&fd).ok_or(-9)?; 
    
    if ih.state != SocketState::Created {
        return Err(-22); 
    }
    
    ih.local_addr = Some(*addr);
    ih.local_port = addr.port();
    ih.state = SocketState::Bound;
    
    crate::serial_println!("[SOCKET] Bound fd={} to port {}", fd, addr.port());
    Ok(())
}


pub fn iks(fd: i32, backlog: u32) -> Result<(), i32> {
    let mut bs = BC_.lock();
    let ih = bs.get_mut(&fd).ok_or(-9)?;
    
    if ih.sock_type != SocketType::Stream {
        return Err(-95); 
    }
    
    if ih.state != SocketState::Bound {
        return Err(-22); 
    }
    
    ih.state = SocketState::Listening;
    ih.backlog = backlog.max(1);
    let port = ih.local_port;
    let bl = ih.backlog;
    drop(bs);

    
    crate::netstack::tcp::etd(port, bl);
    crate::serial_println!("[SOCKET] Listening fd={} port={} backlog={}", fd, port, bl);
    Ok(())
}


pub fn jtj(fd: i32, addr_ptr: u64, addr_len_ptr: u64) -> Result<i32, i32> {
    let cmi = {
        let bs = BC_.lock();
        let ih = bs.get(&fd).ok_or(-9i32)?; 
        if ih.state != SocketState::Listening {
            return Err(-22); 
        }
        ih.local_port
    };

    
    for _ in 0..2000 {
        crate::netstack::poll();

        if let Some((src_port, tn, remote_port)) =
            crate::netstack::tcp::eew(cmi)
        {
            let ue = BDN_.fetch_add(1, Ordering::Relaxed);
            let mut dvd = Socket::new(AddressFamily::Inet, SocketType::Stream, 0);
            dvd.state = SocketState::Connected;
            dvd.local_port = src_port;
            dvd.tcp_src_port = src_port;
            dvd.remote_addr = Some(SockAddrIn::new(tn, remote_port));
            BC_.lock().insert(ue, dvd);

            
            if addr_ptr != 0 && addr_len_ptr != 0 {
                let acl = SockAddrIn::new(tn, remote_port);
                if crate::memory::ij(addr_ptr, core::mem::size_of::<SockAddrIn>(), true) {
                    unsafe { *(addr_ptr as *mut SockAddrIn) = acl; }
                }
                if crate::memory::ij(addr_len_ptr, 4, true) {
                    unsafe { *(addr_len_ptr as *mut u32) = core::mem::size_of::<SockAddrIn>() as u32; }
                }
            }

            crate::serial_println!(
                "[SOCKET] accept fd={} -> new_fd={} remote={}:{}",
                fd, ue,
                tn.iter().map(|b| alloc::format!("{}", b)).collect::<alloc::vec::Vec<_>>().join("."),
                remote_port
            );
            return Ok(ue);
        }

        
        for _ in 0..5000 { core::hint::spin_loop(); }
    }

    Err(-11) 
}


pub fn connect(fd: i32, addr: &SockAddrIn) -> Result<(), i32> {
    
    let (sock_type, local_port) = {
        let mut bs = BC_.lock();
        let ih = bs.get_mut(&fd).ok_or(-9)?;
        
        if ih.state != SocketState::Created && ih.state != SocketState::Bound {
            return Err(-106); 
        }
        
        
        if ih.local_port == 0 {
            ih.local_port = XB_.fetch_add(1, Ordering::Relaxed);
        }
        
        ih.remote_addr = Some(*addr);
        ih.state = SocketState::Connecting;
        
        (ih.sock_type, ih.local_port)
    };
    
    let dest_ip = addr.ip();
    let dest_port = addr.port();
    
    crate::serial_println!(
        "[SOCKET] Connecting fd={} to {}.{}.{}.{}:{}",
        fd, dest_ip[0], dest_ip[1], dest_ip[2], dest_ip[3], dest_port
    );
    
    if sock_type == SocketType::Stream {
        
        match crate::netstack::tcp::azp(dest_ip, dest_port) {
            Ok(src_port) => {
                let mut bs = BC_.lock();
                if let Some(ih) = bs.get_mut(&fd) {
                    ih.tcp_src_port = src_port;
                }
            }
            Err(e) => {
                crate::serial_println!("[SOCKET] SYN failed: {}", e);
                return Err(-111); 
            }
        }
        
        
        for _ in 0..1000 {
            crate::netstack::poll();
            
            
            if crate::netstack::tcp::czx(dest_ip, dest_port) {
                let mut bs = BC_.lock();
                if let Some(ih) = bs.get_mut(&fd) {
                    ih.state = SocketState::Connected;
                }
                crate::serial_println!("[SOCKET] Connected fd={}", fd);
                return Ok(());
            }
            
            
            for _ in 0..10000 { core::hint::spin_loop(); }
        }
        
        Err(-110) 
    } else {
        
        let mut bs = BC_.lock();
        if let Some(ih) = bs.get_mut(&fd) {
            ih.state = SocketState::Connected;
        }
        Ok(())
    }
}


pub fn send(fd: i32, data: &[u8], bej: u32) -> Result<usize, i32> {
    let (sock_type, remote, tcp_port, local_port) = {
        let bs = BC_.lock();
        let ih = bs.get(&fd).ok_or(-9)?;
        
        if ih.state != SocketState::Connected {
            return Err(-107); 
        }
        
        let remote = ih.remote_addr.ok_or(-89)?; 
        (ih.sock_type, remote, ih.tcp_src_port, ih.local_port)
    };
    
    
    if data.len() > 65507 { 
        return Err(-90); 
    }
    
    match sock_type {
        SocketType::Stream => {
            
            crate::netstack::tcp::cqj(remote.ip(), remote.port(), tcp_port, data)
                .map_err(|_| -104)?; 
            Ok(data.len())
        }
        SocketType::Dgram => {
            
            crate::netstack::udp::azq(remote.ip(), remote.port(), local_port, data)
                .map_err(|_| -101)?; 
            Ok(data.len())
        }
        _ => Err(-95), 
    }
}


pub fn recv(fd: i32, buf: &mut [u8], bej: u32) -> Result<usize, i32> {
    let (sock_type, remote, tcp_port) = {
        let bs = BC_.lock();
        let ih = bs.get(&fd).ok_or(-9)?;
        
        if ih.state != SocketState::Connected {
            return Err(-107); 
        }
        
        let remote = ih.remote_addr.ok_or(-107)?;
        (ih.sock_type, remote, ih.tcp_src_port)
    };
    
    
    crate::netstack::poll();
    
    match sock_type {
        SocketType::Stream => {
            
            let data = crate::netstack::tcp::iyp(remote.ip(), remote.port(), tcp_port);
            
            if let Some(data) = data {
                let len = data.len().min(buf.len());
                buf[..len].copy_from_slice(&data[..len]);
                Ok(len)
            } else {
                
                Err(-11) 
            }
        }
        SocketType::Dgram => {
            let local_port = {
                let bs = BC_.lock();
                let ih = bs.get(&fd).ok_or(-9)?;
                ih.local_port
            };
            if let Some(data) = crate::netstack::udp::eyc(local_port) {
                let len = data.len().min(buf.len());
                buf[..len].copy_from_slice(&data[..len]);
                Ok(len)
            } else {
                Err(-11) 
            }
        }
        _ => Err(-95),
    }
}


pub fn close(fd: i32) -> Result<(), i32> {
    let ih = BC_.lock().remove(&fd).ok_or(-9)?;
    
    
    if ih.sock_type == SocketType::Stream && ih.state == SocketState::Connected {
        if let Some(remote) = ih.remote_addr {
            let _ = crate::netstack::tcp::ams(remote.ip(), remote.port(), ih.tcp_src_port);
        }
    }
    
    crate::serial_println!("[SOCKET] Closed fd={}", fd);
    Ok(())
}


pub fn ooe(fd: i32, data: &[u8], bej: u32, addr: &SockAddrIn) -> Result<usize, i32> {
    let (sock_type, local_port) = {
        let mut bs = BC_.lock();
        let ih = bs.get_mut(&fd).ok_or(-9)?;
        
        
        if ih.local_port == 0 {
            ih.local_port = XB_.fetch_add(1, Ordering::Relaxed);
        }
        
        (ih.sock_type, ih.local_port)
    };
    
    if sock_type != SocketType::Dgram {
        return Err(-95); 
    }
    
    
    if data.len() > 65507 {
        return Err(-90); 
    }
    
    crate::netstack::udp::azq(addr.ip(), addr.port(), local_port, data)
        .map_err(|_| -101)?;
    
    Ok(data.len())
}


pub fn qti(fd: i32, buf: &mut [u8], bej: u32, addr_out: Option<&mut SockAddrIn>) -> Result<usize, i32> {
    let local_port = {
        let bs = BC_.lock();
        let ih = bs.get(&fd).ok_or(-9)?;
        
        if ih.sock_type != SocketType::Dgram {
            return Err(-95);
        }
        
        ih.local_port
    };
    
    
    crate::netstack::poll();
    
    if let Some((data, src_ip, src_port)) = crate::netstack::udp::odt(local_port) {
        let len = data.len().min(buf.len());
        buf[..len].copy_from_slice(&data[..len]);
        if let Some(addr_out) = addr_out {
            *addr_out = SockAddrIn::new(src_ip, src_port);
        }
        Ok(len)
    } else {
        Err(-11) 
    }
}


pub fn opz(fd: i32, level: i32, optname: i32, optval: &[u8]) -> Result<(), i32> {
    let mut bs = BC_.lock();
    let ih = bs.get_mut(&fd).ok_or(-9)?;
    
    
    
    
    
    match (level, optname) {
        (1, 2) => {
            
            Ok(())
        }
        (1, 4) => {
            
            ih.non_blocking = optval.first().copied().unwrap_or(0) != 0;
            Ok(())
        }
        _ => {
            crate::serial_println!("[SOCKET] Unknown sockopt level={} name={}", level, optname);
            Ok(()) 
        }
    }
}


pub fn meh(fd: i32, level: i32, optname: i32, optval: &mut [u8]) -> Result<usize, i32> {
    let bs = BC_.lock();
    let ih = bs.get(&fd).ok_or(-9)?;
    
    match (level, optname) {
        (1, 2) => {
            
            if !optval.is_empty() {
                optval[0] = 1;
            }
            Ok(4)
        }
        _ => {
            if !optval.is_empty() {
                optval[0] = 0;
            }
            Ok(4)
        }
    }
}


pub fn mts(fd: i32) -> bool {
    BC_.lock().contains_key(&fd)
}


pub fn get_state(fd: i32) -> Option<SocketState> {
    BC_.lock().get(&fd).map(|j| j.state)
}


pub fn mjy(fd: i32) -> bool {
    let bs = BC_.lock();
    if let Some(ih) = bs.get(&fd) {
        if ih.state == SocketState::Connected {
            if let Some(addr) = &ih.remote_addr {
                return crate::netstack::tcp::iyp(addr.ip(), addr.port(), ih.tcp_src_port).is_some();
            }
        }
    }
    false
}
