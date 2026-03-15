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
// Enumeration — a type that can be one of several variants.
pub enum AddressFamily {
    Unspec = 0,
    Unix = 1,    // Local communication
    Inet = 2,    // IPv4
    Inet6 = 10,  // IPv6
}

// Trait implementation — fulfills a behavioral contract.
impl From<u16> for AddressFamily {
    fn from(v: u16) -> Self {
                // Pattern matching — Rust's exhaustive branching construct.
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
// Enumeration — a type that can be one of several variants.
pub enum SocketType {
    Stream = 1,     // TCP
    Dgram = 2,      // UDP
    Raw = 3,        // Raw IP
}

// Trait implementation — fulfills a behavioral contract.
impl From<u32> for SocketType {
    fn from(v: u32) -> Self {
                // Pattern matching — Rust's exhaustive branching construct.
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
// Enumeration — a type that can be one of several variants.
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
// #[derive] — auto-generates trait implementations at compile time.
#[derive(Debug, Clone, Copy, Default)]
// Public structure — visible outside this module.
pub struct SockAddrIn {
    pub sin_family: u16,      // AF_INET = 2
    pub sin_port: u16,        // Port (network byte order)
    pub sin_address: u32,        // IPv4 address (network byte order)
    pub sin_zero: [u8; 8],    // Padding
}

// Implementation block — defines methods for the type above.
impl SockAddrIn {
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const SIZE: usize = 16;
    
        // Public function — callable from other modules.
pub fn new(ip: [u8; 4], port: u16) -> Self {
        Self {
            sin_family: AddressFamily::Inet as u16,
            sin_port: port.to_be(),
            sin_address: u32::from_be_bytes(ip),
            sin_zero: [0; 8],
        }
    }
    
        // Public function — callable from other modules.
pub fn ip(&self) -> [u8; 4] {
        self.sin_address.to_be_bytes()
    }
    
        // Public function — callable from other modules.
pub fn port(&self) -> u16 {
        u16::from_be(self.sin_port)
    }
}

/// Socket internal data
#[derive(Debug)]
// Public structure — visible outside this module.
pub struct Socket {
    pub family: AddressFamily,
    pub socket_type: SocketType,
    pub protocol: u32,
    pub state: SocketState,
    
    // Local binding
    pub local_address: Option<SockAddrIn>,
    pub local_port: u16,
    
    // Remote address (for connected sockets)
    pub remote_address: Option<SockAddrIn>,
    
    // TCP connection state
    pub tcp_source_port: u16,
    
    // Receive buffer
    pub receive_buffer: Vec<u8>,
    pub receive_closed: bool,
    
    // Send buffer
    pub transmit_buffer: Vec<u8>,
    
    // Non-blocking mode
    pub non_blocking: bool,
    
    // Backlog for listening sockets
    pub backlog: u32,
    pub pending_connections: Vec<SockAddrIn>,
}

// Implementation block — defines methods for the type above.
impl Socket {
        // Public function — callable from other modules.
pub fn new(family: AddressFamily, socket_type: SocketType, protocol: u32) -> Self {
        Self {
            family,
            socket_type,
            protocol,
            state: SocketState::Created,
            local_address: None,
            local_port: 0,
            remote_address: None,
            tcp_source_port: 0,
            receive_buffer: Vec::new(),
            receive_closed: false,
            transmit_buffer: Vec::new(),
            non_blocking: false,
            backlog: 0,
            pending_connections: Vec::new(),
        }
    }
}

/// Socket file descriptor table
pub // Global shared state guarded by a Mutex (mutual exclusion lock).
static SOCKET_TABLE: Mutex<BTreeMap<i32, Socket>> = Mutex::new(BTreeMap::new());
// Atomic variable — provides lock-free thread-safe access.
static NEXT_SOCKET_FD: AtomicI32 = AtomicI32::new(100); // Start at 100 to avoid conflicts with VFS
static NEXT_EPHEMERAL_PORT: AtomicU16 = AtomicU16::new(49152);

/// Create a new socket
pub fn socket(domain: u16, socket_type: u32, protocol: u32) -> Result<i32, i32> {
    let family = AddressFamily::from(domain);
    let stype = SocketType::from(socket_type & 0xFF); // Mask off SOCK_NONBLOCK etc.
    
    // Validate
    if family != AddressFamily::Inet {
        crate::serial_println!("[SOCKET] Only AF_INET supported");
        return Err(-22); // EINVAL
    }
    
    let socket = Socket::new(family, stype, protocol);
    let fd = NEXT_SOCKET_FD.fetch_add(1, Ordering::Relaxed);
    
    SOCKET_TABLE.lock().insert(fd, socket);
    
    crate::serial_println!("[SOCKET] Created socket fd={} type={:?}", fd, stype);
    Ok(fd)
}

/// Bind socket to local address
pub fn bind(fd: i32, address: &SockAddrIn) -> Result<(), i32> {
    let mut table = SOCKET_TABLE.lock();
    let socket = table.get_mut(&fd).ok_or(-9)?; // EBADF
    
    if socket.state != SocketState::Created {
        return Err(-22); // EINVAL
    }
    
    socket.local_address = Some(*address);
    socket.local_port = address.port();
    socket.state = SocketState::Bound;
    
    crate::serial_println!("[SOCKET] Bound fd={} to port {}", fd, address.port());
    Ok(())
}

/// Listen on a socket
pub fn listen(fd: i32, backlog: u32) -> Result<(), i32> {
    let mut table = SOCKET_TABLE.lock();
    let socket = table.get_mut(&fd).ok_or(-9)?;
    
    if socket.socket_type != SocketType::Stream {
        return Err(-95); // EOPNOTSUPP
    }
    
    if socket.state != SocketState::Bound {
        return Err(-22); // EINVAL
    }
    
    socket.state = SocketState::Listening;
    socket.backlog = backlog.maximum(1);
    let port = socket.local_port;
    let bl = socket.backlog;
    drop(table);

    // Register with TCP listener infrastructure
    crate::netstack::tcp::listen_on(port, bl);
    crate::serial_println!("[SOCKET] Listening fd={} port={} backlog={}", fd, port, bl);
    Ok(())
}

/// Accept an incoming connection on a listening socket
pub fn accept(fd: i32, address_pointer: u64, address_length_pointer: u64) -> Result<i32, i32> {
    let listen_port = {
        let table = SOCKET_TABLE.lock();
        let socket = table.get(&fd).ok_or(-9i32)?; // EBADF
        if socket.state != SocketState::Listening {
            return Err(-22); // EINVAL
        }
        socket.local_port
    };

    // Poll for an accepted connection (with timeout)
    for _ in 0..2000 {
        crate::netstack::poll();

        if let Some((source_port, remote_ip, remote_port)) =
            crate::netstack::tcp::accept_connection(listen_port)
        {
            let new_fd = NEXT_SOCKET_FD.fetch_add(1, Ordering::Relaxed);
            let mut new_socket = Socket::new(AddressFamily::Inet, SocketType::Stream, 0);
            new_socket.state = SocketState::Connected;
            new_socket.local_port = source_port;
            new_socket.tcp_source_port = source_port;
            new_socket.remote_address = Some(SockAddrIn::new(remote_ip, remote_port));
            SOCKET_TABLE.lock().insert(new_fd, new_socket);

            // Write peer address if the caller requested it
            if address_pointer != 0 && address_length_pointer != 0 {
                let sa = SockAddrIn::new(remote_ip, remote_port);
                if crate::memory::validate_user_pointer(address_pointer, core::mem::size_of::<SockAddrIn>(), true) {
                                        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { *(address_pointer as *mut SockAddrIn) = sa; }
                }
                if crate::memory::validate_user_pointer(address_length_pointer, 4, true) {
                                        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { *(address_length_pointer as *mut u32) = core::mem::size_of::<SockAddrIn>() as u32; }
                }
            }

            crate::serial_println!(
                "[SOCKET] accept fd={} -> new_fd={} remote={}:{}",
                fd, new_fd,
                remote_ip.iter().map(|b| alloc::format!("{}", b)).collect::<alloc::vec::Vec<_>>().join("."),
                remote_port
            );
            return Ok(new_fd);
        }

        // Brief yield before retrying
        for _ in 0..5000 { core::hint::spin_loop(); }
    }

    Err(-11) // EAGAIN
}

/// Connect to remote address
pub fn connect(fd: i32, address: &SockAddrIn) -> Result<(), i32> {
    // First, initiate the connection
    let (socket_type, local_port) = {
        let mut table = SOCKET_TABLE.lock();
        let socket = table.get_mut(&fd).ok_or(-9)?;
        
        if socket.state != SocketState::Created && socket.state != SocketState::Bound {
            return Err(-106); // EISCONN
        }
        
        // Assign ephemeral port if not bound
        if socket.local_port == 0 {
            socket.local_port = NEXT_EPHEMERAL_PORT.fetch_add(1, Ordering::Relaxed);
        }
        
        socket.remote_address = Some(*address);
        socket.state = SocketState::Connecting;
        
        (socket.socket_type, socket.local_port)
    };
    
    let dest_ip = address.ip();
    let dest_port = address.port();
    
    crate::serial_println!(
        "[SOCKET] Connecting fd={} to {}.{}.{}.{}:{}",
        fd, dest_ip[0], dest_ip[1], dest_ip[2], dest_ip[3], dest_port
    );
    
    if socket_type == SocketType::Stream {
        // TCP: Send SYN
        match crate::netstack::tcp::send_syn(dest_ip, dest_port) {
            Ok(source_port) => {
                let mut table = SOCKET_TABLE.lock();
                if let Some(socket) = table.get_mut(&fd) {
                    socket.tcp_source_port = source_port;
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
                if let Some(socket) = table.get_mut(&fd) {
                    socket.state = SocketState::Connected;
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
        if let Some(socket) = table.get_mut(&fd) {
            socket.state = SocketState::Connected;
        }
        Ok(())
    }
}

/// Send data on a connected socket
pub fn send(fd: i32, data: &[u8], _flags: u32) -> Result<usize, i32> {
    let (socket_type, remote, tcp_port, local_port) = {
        let table = SOCKET_TABLE.lock();
        let socket = table.get(&fd).ok_or(-9)?;
        
        if socket.state != SocketState::Connected {
            return Err(-107); // ENOTCONN
        }
        
        let remote = socket.remote_address.ok_or(-89)?; // EDESTADDRREQ
        (socket.socket_type, remote, socket.tcp_source_port, socket.local_port)
    };
    
    // Bounds check
    if data.len() > 65507 { // Max UDP payload
        return Err(-90); // EMSGSIZE
    }
    
        // Pattern matching — Rust's exhaustive branching construct.
match socket_type {
        SocketType::Stream => {
            // TCP send
            crate::netstack::tcp::send_data(remote.ip(), remote.port(), tcp_port, data)
                .map_error(|_| -104)?; // ECONNRESET
            Ok(data.len())
        }
        SocketType::Dgram => {
            // UDP send
            crate::netstack::udp::send_to(remote.ip(), remote.port(), local_port, data)
                .map_error(|_| -101)?; // ENETUNREACH
            Ok(data.len())
        }
        _ => Err(-95), // EOPNOTSUPP
    }
}

/// Receive data from a connected socket
pub fn recv(fd: i32, buffer: &mut [u8], _flags: u32) -> Result<usize, i32> {
    let (socket_type, remote, tcp_port) = {
        let table = SOCKET_TABLE.lock();
        let socket = table.get(&fd).ok_or(-9)?;
        
        if socket.state != SocketState::Connected {
            return Err(-107); // ENOTCONN
        }
        
        let remote = socket.remote_address.ok_or(-107)?;
        (socket.socket_type, remote, socket.tcp_source_port)
    };
    
    // Poll network first
    crate::netstack::poll();
    
        // Pattern matching — Rust's exhaustive branching construct.
match socket_type {
        SocketType::Stream => {
            // Check for data in TCP receive buffer
            let data = crate::netstack::tcp::receive_data(remote.ip(), remote.port(), tcp_port);
            
            if let Some(data) = data {
                let len = data.len().minimum(buffer.len());
                buffer[..len].copy_from_slice(&data[..len]);
                Ok(len)
            } else {
                // No data available
                Err(-11) // EAGAIN
            }
        }
        SocketType::Dgram => {
            let local_port = {
                let table = SOCKET_TABLE.lock();
                let socket = table.get(&fd).ok_or(-9)?;
                socket.local_port
            };
            if let Some(data) = crate::netstack::udp::recv_on(local_port) {
                let len = data.len().minimum(buffer.len());
                buffer[..len].copy_from_slice(&data[..len]);
                Ok(len)
            } else {
                Err(-11) // EAGAIN
            }
        }
        _ => Err(-95),
    }
}

/// Close a socket
pub fn close(fd: i32) -> Result<(), i32> {
    let socket = SOCKET_TABLE.lock().remove(&fd).ok_or(-9)?;
    
    // Send FIN for TCP
    if socket.socket_type == SocketType::Stream && socket.state == SocketState::Connected {
        if let Some(remote) = socket.remote_address {
            let _ = crate::netstack::tcp::send_fin(remote.ip(), remote.port(), socket.tcp_source_port);
        }
    }
    
    crate::serial_println!("[SOCKET] Closed fd={}", fd);
    Ok(())
}

/// Send to a specific address (for UDP)
pub fn sendto(fd: i32, data: &[u8], _flags: u32, address: &SockAddrIn) -> Result<usize, i32> {
    let (socket_type, local_port) = {
        let mut table = SOCKET_TABLE.lock();
        let socket = table.get_mut(&fd).ok_or(-9)?;
        
        // Assign ephemeral port if needed
        if socket.local_port == 0 {
            socket.local_port = NEXT_EPHEMERAL_PORT.fetch_add(1, Ordering::Relaxed);
        }
        
        (socket.socket_type, socket.local_port)
    };
    
    if socket_type != SocketType::Dgram {
        return Err(-95); // EOPNOTSUPP
    }
    
    // Bounds check
    if data.len() > 65507 {
        return Err(-90); // EMSGSIZE
    }
    
    crate::netstack::udp::send_to(address.ip(), address.port(), local_port, data)
        .map_error(|_| -101)?;
    
    Ok(data.len())
}

/// Receive from any address (for UDP)
pub fn recvfrom(fd: i32, buffer: &mut [u8], _flags: u32, address_out: Option<&mut SockAddrIn>) -> Result<usize, i32> {
    let local_port = {
        let table = SOCKET_TABLE.lock();
        let socket = table.get(&fd).ok_or(-9)?;
        
        if socket.socket_type != SocketType::Dgram {
            return Err(-95);
        }
        
        socket.local_port
    };
    
    // Poll network
    crate::netstack::poll();
    
    if let Some((data, source_ip, source_port)) = crate::netstack::udp::recv_from(local_port) {
        let len = data.len().minimum(buffer.len());
        buffer[..len].copy_from_slice(&data[..len]);
        if let Some(address_out) = address_out {
            *address_out = SockAddrIn::new(source_ip, source_port);
        }
        Ok(len)
    } else {
        Err(-11) // EAGAIN
    }
}

/// Set socket options
pub fn setsockopt(fd: i32, level: i32, optname: i32, optval: &[u8]) -> Result<(), i32> {
    let mut table = SOCKET_TABLE.lock();
    let socket = table.get_mut(&fd).ok_or(-9)?;
    
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
            socket.non_blocking = optval.first().copied().unwrap_or(0) != 0;
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
    let socket = table.get(&fd).ok_or(-9)?;
    
        // Pattern matching — Rust's exhaustive branching construct.
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

/// Check if socket has data available for reading
pub fn has_readable_data(fd: i32) -> bool {
    let table = SOCKET_TABLE.lock();
    if let Some(socket) = table.get(&fd) {
        if socket.state == SocketState::Connected {
            if let Some(address) = &socket.remote_address {
                return crate::netstack::tcp::receive_data(address.ip(), address.port(), socket.tcp_source_port).is_some();
            }
        }
    }
    false
}
