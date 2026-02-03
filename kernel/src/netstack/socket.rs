//! BSD-style Socket Interface
//!
//! Provides userspace-accessible socket API via syscalls.
//! Like modern OSes: kernel handles TCP/IP, userspace handles TLS/apps.

use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use alloc::boxed::Box;
use core::sync::atomic::{AtomicI32, AtomicU16, Ordering};
use spin::Mutex;

/// Socket address family
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum AddressFamily {
    Unspec = 0,
    Unix = 1,    // Local communication
    Inet = 2,    // IPv4
    Inet6 = 10,  // IPv6
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

/// Socket type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum SocketType {
    Stream = 1,     // TCP
    Dgram = 2,      // UDP
    Raw = 3,        // Raw IP
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

/// Socket state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SocketState {
    Created,
    Bound,
    Listening,
    Connecting,
    Connected,
    Closed,
}

/// IPv4 socket address (struct sockaddr_in)
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct SockAddrIn {
    pub sin_family: u16,      // AF_INET = 2
    pub sin_port: u16,        // Port (network byte order)
    pub sin_addr: u32,        // IPv4 address (network byte order)
    pub sin_zero: [u8; 8],    // Padding
}

impl SockAddrIn {
    pub const SIZE: usize = 16;
    
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

/// Socket internal data
#[derive(Debug)]
pub struct Socket {
    pub family: AddressFamily,
    pub sock_type: SocketType,
    pub protocol: u32,
    pub state: SocketState,
    
    // Local binding
    pub local_addr: Option<SockAddrIn>,
    pub local_port: u16,
    
    // Remote address (for connected sockets)
    pub remote_addr: Option<SockAddrIn>,
    
    // TCP connection state
    pub tcp_src_port: u16,
    
    // Receive buffer
    pub rx_buffer: Vec<u8>,
    pub rx_closed: bool,
    
    // Send buffer
    pub tx_buffer: Vec<u8>,
    
    // Non-blocking mode
    pub non_blocking: bool,
    
    // Backlog for listening sockets
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

/// Socket file descriptor table
static SOCKET_TABLE: Mutex<BTreeMap<i32, Socket>> = Mutex::new(BTreeMap::new());
static NEXT_SOCKET_FD: AtomicI32 = AtomicI32::new(100); // Start at 100 to avoid conflicts with VFS
static NEXT_EPHEMERAL_PORT: AtomicU16 = AtomicU16::new(49152);

/// Create a new socket
pub fn socket(domain: u16, sock_type: u32, protocol: u32) -> Result<i32, i32> {
    let family = AddressFamily::from(domain);
    let stype = SocketType::from(sock_type & 0xFF); // Mask off SOCK_NONBLOCK etc.
    
    // Validate
    if family != AddressFamily::Inet {
        crate::serial_println!("[SOCKET] Only AF_INET supported");
        return Err(-22); // EINVAL
    }
    
    let sock = Socket::new(family, stype, protocol);
    let fd = NEXT_SOCKET_FD.fetch_add(1, Ordering::Relaxed);
    
    SOCKET_TABLE.lock().insert(fd, sock);
    
    crate::serial_println!("[SOCKET] Created socket fd={} type={:?}", fd, stype);
    Ok(fd)
}

/// Bind socket to local address
pub fn bind(fd: i32, addr: &SockAddrIn) -> Result<(), i32> {
    let mut table = SOCKET_TABLE.lock();
    let sock = table.get_mut(&fd).ok_or(-9)?; // EBADF
    
    if sock.state != SocketState::Created {
        return Err(-22); // EINVAL
    }
    
    sock.local_addr = Some(*addr);
    sock.local_port = addr.port();
    sock.state = SocketState::Bound;
    
    crate::serial_println!("[SOCKET] Bound fd={} to port {}", fd, addr.port());
    Ok(())
}

/// Listen on a socket
pub fn listen(fd: i32, backlog: u32) -> Result<(), i32> {
    let mut table = SOCKET_TABLE.lock();
    let sock = table.get_mut(&fd).ok_or(-9)?;
    
    if sock.sock_type != SocketType::Stream {
        return Err(-95); // EOPNOTSUPP
    }
    
    if sock.state != SocketState::Bound {
        return Err(-22); // EINVAL
    }
    
    sock.state = SocketState::Listening;
    sock.backlog = backlog.max(1);
    
    crate::serial_println!("[SOCKET] Listening fd={} backlog={}", fd, backlog);
    Ok(())
}

/// Connect to remote address
pub fn connect(fd: i32, addr: &SockAddrIn) -> Result<(), i32> {
    // First, initiate the connection
    let (sock_type, local_port) = {
        let mut table = SOCKET_TABLE.lock();
        let sock = table.get_mut(&fd).ok_or(-9)?;
        
        if sock.state != SocketState::Created && sock.state != SocketState::Bound {
            return Err(-106); // EISCONN
        }
        
        // Assign ephemeral port if not bound
        if sock.local_port == 0 {
            sock.local_port = NEXT_EPHEMERAL_PORT.fetch_add(1, Ordering::Relaxed);
        }
        
        sock.remote_addr = Some(*addr);
        sock.state = SocketState::Connecting;
        
        (sock.sock_type, sock.local_port)
    };
    
    let dest_ip = addr.ip();
    let dest_port = addr.port();
    
    crate::serial_println!(
        "[SOCKET] Connecting fd={} to {}.{}.{}.{}:{}",
        fd, dest_ip[0], dest_ip[1], dest_ip[2], dest_ip[3], dest_port
    );
    
    if sock_type == SocketType::Stream {
        // TCP: Send SYN
        match crate::netstack::tcp::send_syn(dest_ip, dest_port) {
            Ok(src_port) => {
                let mut table = SOCKET_TABLE.lock();
                if let Some(sock) = table.get_mut(&fd) {
                    sock.tcp_src_port = src_port;
                }
            }
            Err(e) => {
                crate::serial_println!("[SOCKET] SYN failed: {}", e);
                return Err(-111); // ECONNREFUSED
            }
        }
        
        // Wait for SYN-ACK (poll network)
        for _ in 0..1000 {
            crate::netstack::poll();
            
            // Check if connected
            if crate::netstack::tcp::is_connected(dest_ip, dest_port) {
                let mut table = SOCKET_TABLE.lock();
                if let Some(sock) = table.get_mut(&fd) {
                    sock.state = SocketState::Connected;
                }
                crate::serial_println!("[SOCKET] Connected fd={}", fd);
                return Ok(());
            }
            
            // Small delay
            for _ in 0..10000 { core::hint::spin_loop(); }
        }
        
        Err(-110) // ETIMEDOUT
    } else {
        // UDP: Just mark as "connected" (remembers remote address)
        let mut table = SOCKET_TABLE.lock();
        if let Some(sock) = table.get_mut(&fd) {
            sock.state = SocketState::Connected;
        }
        Ok(())
    }
}

/// Send data on a connected socket
pub fn send(fd: i32, data: &[u8], _flags: u32) -> Result<usize, i32> {
    let (sock_type, remote, tcp_port, local_port) = {
        let table = SOCKET_TABLE.lock();
        let sock = table.get(&fd).ok_or(-9)?;
        
        if sock.state != SocketState::Connected {
            return Err(-107); // ENOTCONN
        }
        
        let remote = sock.remote_addr.ok_or(-89)?; // EDESTADDRREQ
        (sock.sock_type, remote, sock.tcp_src_port, sock.local_port)
    };
    
    // Bounds check
    if data.len() > 65507 { // Max UDP payload
        return Err(-90); // EMSGSIZE
    }
    
    match sock_type {
        SocketType::Stream => {
            // TCP send
            crate::netstack::tcp::send_data(remote.ip(), remote.port(), tcp_port, data)
                .map_err(|_| -104)?; // ECONNRESET
            Ok(data.len())
        }
        SocketType::Dgram => {
            // UDP send
            crate::netstack::udp::send_to(remote.ip(), remote.port(), local_port, data)
                .map_err(|_| -101)?; // ENETUNREACH
            Ok(data.len())
        }
        _ => Err(-95), // EOPNOTSUPP
    }
}

/// Receive data from a connected socket
pub fn recv(fd: i32, buf: &mut [u8], _flags: u32) -> Result<usize, i32> {
    let (sock_type, remote, tcp_port) = {
        let table = SOCKET_TABLE.lock();
        let sock = table.get(&fd).ok_or(-9)?;
        
        if sock.state != SocketState::Connected {
            return Err(-107); // ENOTCONN
        }
        
        let remote = sock.remote_addr.ok_or(-107)?;
        (sock.sock_type, remote, sock.tcp_src_port)
    };
    
    // Poll network first
    crate::netstack::poll();
    
    match sock_type {
        SocketType::Stream => {
            // Check for data in TCP receive buffer
            let data = crate::netstack::tcp::receive_data(remote.ip(), remote.port(), tcp_port);
            
            if let Some(data) = data {
                let len = data.len().min(buf.len());
                buf[..len].copy_from_slice(&data[..len]);
                Ok(len)
            } else {
                // No data available
                Err(-11) // EAGAIN
            }
        }
        SocketType::Dgram => {
            // TODO: UDP receive
            Err(-11) // EAGAIN
        }
        _ => Err(-95),
    }
}

/// Close a socket
pub fn close(fd: i32) -> Result<(), i32> {
    let sock = SOCKET_TABLE.lock().remove(&fd).ok_or(-9)?;
    
    // Send FIN for TCP
    if sock.sock_type == SocketType::Stream && sock.state == SocketState::Connected {
        if let Some(remote) = sock.remote_addr {
            let _ = crate::netstack::tcp::send_fin(remote.ip(), remote.port(), sock.tcp_src_port);
        }
    }
    
    crate::serial_println!("[SOCKET] Closed fd={}", fd);
    Ok(())
}

/// Send to a specific address (for UDP)
pub fn sendto(fd: i32, data: &[u8], _flags: u32, addr: &SockAddrIn) -> Result<usize, i32> {
    let (sock_type, local_port) = {
        let mut table = SOCKET_TABLE.lock();
        let sock = table.get_mut(&fd).ok_or(-9)?;
        
        // Assign ephemeral port if needed
        if sock.local_port == 0 {
            sock.local_port = NEXT_EPHEMERAL_PORT.fetch_add(1, Ordering::Relaxed);
        }
        
        (sock.sock_type, sock.local_port)
    };
    
    if sock_type != SocketType::Dgram {
        return Err(-95); // EOPNOTSUPP
    }
    
    // Bounds check
    if data.len() > 65507 {
        return Err(-90); // EMSGSIZE
    }
    
    crate::netstack::udp::send_to(addr.ip(), addr.port(), local_port, data)
        .map_err(|_| -101)?;
    
    Ok(data.len())
}

/// Receive from any address (for UDP)
pub fn recvfrom(fd: i32, buf: &mut [u8], _flags: u32, addr_out: Option<&mut SockAddrIn>) -> Result<usize, i32> {
    let local_port = {
        let table = SOCKET_TABLE.lock();
        let sock = table.get(&fd).ok_or(-9)?;
        
        if sock.sock_type != SocketType::Dgram {
            return Err(-95);
        }
        
        sock.local_port
    };
    
    // Poll network
    crate::netstack::poll();
    
    // TODO: Check UDP receive buffer
    // For now, return EAGAIN
    Err(-11) // EAGAIN
}

/// Set socket options
pub fn setsockopt(fd: i32, level: i32, optname: i32, optval: &[u8]) -> Result<(), i32> {
    let mut table = SOCKET_TABLE.lock();
    let sock = table.get_mut(&fd).ok_or(-9)?;
    
    // SOL_SOCKET = 1
    // SO_REUSEADDR = 2
    // SO_NONBLOCK = 4 (our extension)
    
    match (level, optname) {
        (1, 2) => {
            // SO_REUSEADDR - just accept
            Ok(())
        }
        (1, 4) => {
            // Non-blocking mode
            sock.non_blocking = optval.first().copied().unwrap_or(0) != 0;
            Ok(())
        }
        _ => {
            crate::serial_println!("[SOCKET] Unknown sockopt level={} name={}", level, optname);
            Ok(()) // Ignore unknown options
        }
    }
}

/// Get socket options
pub fn getsockopt(fd: i32, level: i32, optname: i32, optval: &mut [u8]) -> Result<usize, i32> {
    let table = SOCKET_TABLE.lock();
    let sock = table.get(&fd).ok_or(-9)?;
    
    match (level, optname) {
        (1, 2) => {
            // SO_REUSEADDR
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

/// Check if a socket is valid
pub fn is_socket(fd: i32) -> bool {
    SOCKET_TABLE.lock().contains_key(&fd)
}

/// Get socket state (for debugging)
pub fn get_state(fd: i32) -> Option<SocketState> {
    SOCKET_TABLE.lock().get(&fd).map(|s| s.state)
}
