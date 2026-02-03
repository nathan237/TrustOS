//! Network Driver
//! 
//! Network driver with real PCI hardware detection.
//! Uses the pci module for device enumeration.

use spin::Mutex;
use alloc::vec::Vec;
use alloc::string::String;
use alloc::collections::VecDeque;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};

/// Maximum packet size (MTU + headers)
pub const MAX_PACKET_SIZE: usize = 1518;

/// Network interface state
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NetworkState {
    Down,
    Up,
    Error,
}

/// MAC address
#[derive(Debug, Clone, Copy)]
pub struct MacAddress([u8; 6]);

impl MacAddress {
    pub const fn new(bytes: [u8; 6]) -> Self {
        Self(bytes)
    }
    
    pub fn as_bytes(&self) -> &[u8; 6] {
        &self.0
    }
}

impl core::fmt::Display for MacAddress {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
            self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5])
    }
}

/// IPv4 address
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ipv4Address([u8; 4]);

impl Ipv4Address {
    pub const fn new(a: u8, b: u8, c: u8, d: u8) -> Self {
        Self([a, b, c, d])
    }
    
    pub fn as_bytes(&self) -> &[u8; 4] {
        &self.0
    }
}

impl core::fmt::Display for Ipv4Address {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}.{}.{}.{}",
            self.0[0], self.0[1], self.0[2], self.0[3])
    }
}

/// Network interface
pub struct NetworkInterface {
    pub mac: MacAddress,
    pub ip: Option<Ipv4Address>,
    pub subnet: Option<Ipv4Address>,
    pub gateway: Option<Ipv4Address>,
    pub state: NetworkState,
}

/// Network statistics
#[derive(Debug, Clone, Copy, Default)]
pub struct NetworkStats {
    pub packets_sent: u64,
    pub packets_received: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub errors: u64,
}

/// Ethernet frame header
#[repr(C, packed)]
pub struct EthernetHeader {
    pub dst_mac: [u8; 6],
    pub src_mac: [u8; 6],
    pub ethertype: u16,
}

/// ARP packet
#[repr(C, packed)]
pub struct ArpPacket {
    pub htype: u16,      // Hardware type (Ethernet = 1)
    pub ptype: u16,      // Protocol type (IPv4 = 0x0800)
    pub hlen: u8,        // Hardware address length (6)
    pub plen: u8,        // Protocol address length (4)
    pub operation: u16,  // 1 = request, 2 = reply
    pub sender_mac: [u8; 6],
    pub sender_ip: [u8; 4],
    pub target_mac: [u8; 6],
    pub target_ip: [u8; 4],
}

/// Global network interface
static INTERFACE: Mutex<Option<NetworkInterface>> = Mutex::new(None);

/// Receive queue
static RX_QUEUE: Mutex<VecDeque<Vec<u8>>> = Mutex::new(VecDeque::new());

/// Transmit queue
static TX_QUEUE: Mutex<VecDeque<Vec<u8>>> = Mutex::new(VecDeque::new());

/// Statistics
static STATS: Mutex<NetworkStats> = Mutex::new(NetworkStats {
    packets_sent: 0,
    packets_received: 0,
    bytes_sent: 0,
    bytes_received: 0,
    errors: 0,
});

/// Initialized flag
static INITIALIZED: AtomicBool = AtomicBool::new(false);

/// Detected NIC info
static NIC_INFO: Mutex<Option<NicInfo>> = Mutex::new(None);

/// NIC hardware info
#[derive(Debug, Clone)]
pub struct NicInfo {
    pub vendor_id: u16,
    pub device_id: u16,
    pub vendor_name: String,
    pub driver: String,
    pub bar0: u64,
    pub irq: u8,
}

/// Get NIC hardware info
pub fn get_nic_info() -> Option<NicInfo> {
    NIC_INFO.lock().clone()
}

/// Identify driver for a network device
fn identify_driver(vendor_id: u16, device_id: u16) -> &'static str {
    match (vendor_id, device_id) {
        // Intel NICs
        (0x8086, 0x100E) => "e1000",      // 82540EM (QEMU default)
        (0x8086, 0x100F) => "e1000",      // 82545EM
        (0x8086, 0x10D3) => "e1000e",     // 82574L
        (0x8086, 0x153A) => "e1000e",     // I217-LM
        (0x8086, 0x15B8) => "e1000e",     // I219-V
        
        // Virtio (QEMU/KVM)
        (0x1AF4, 0x1000) => "virtio-net",
        (0x1AF4, 0x1041) => "virtio-net", // Virtio 1.0
        
        // Realtek
        (0x10EC, 0x8139) => "rtl8139",
        (0x10EC, 0x8168) => "r8168",
        (0x10EC, 0x8169) => "r8169",
        
        // Broadcom
        (0x14E4, _) => "bnx2/tg3",
        
        // VMware
        (0x15AD, 0x0720) => "vmxnet",
        (0x15AD, 0x07B0) => "vmxnet3",
        
        _ => "generic",
    }
}

/// Initialize network driver
pub fn init() {
    // Use PCI module to find network devices
    let network_devices = crate::pci::find_by_class(crate::pci::class::NETWORK);
    
    if network_devices.is_empty() {
        crate::log_warn!("[NET] No network controller found");
        return;
    }
    
    // Take first network device
    let dev = &network_devices[0];
    let driver = identify_driver(dev.vendor_id, dev.device_id);
    
    // Store NIC info
    let nic_info = NicInfo {
        vendor_id: dev.vendor_id,
        device_id: dev.device_id,
        vendor_name: String::from(dev.vendor_name()),
        driver: String::from(driver),
        bar0: dev.bar_address(0).unwrap_or(0),
        irq: dev.interrupt_line,
    };
    
    crate::log!("[NET] Found: {:04X}:{:04X} {} [{}] BAR0={:#X} IRQ={}",
        dev.vendor_id, dev.device_id, 
        driver, dev.vendor_name(),
        nic_info.bar0, nic_info.irq);
    
    *NIC_INFO.lock() = Some(nic_info);
    
    // Enable bus mastering for DMA
    crate::pci::enable_bus_master(dev);
    crate::pci::enable_memory_space(dev);
    
    // If virtio-net driver is initialized, get real MAC, otherwise generate one
    let mac = if crate::virtio_net::is_initialized() {
        if let Some(real_mac) = crate::virtio_net::get_mac() {
            MacAddress::new(real_mac)
        } else {
            // Generate MAC if not available
            let ticks = crate::logger::get_ticks();
            MacAddress::new([
                0x52, 0x54, 0x00,  // QEMU OUI prefix
                ((ticks >> 8) & 0xFF) as u8,
                ((ticks >> 16) & 0xFF) as u8,
                (ticks & 0xFF) as u8,
            ])
        }
    } else {
        // Generate MAC (driver not yet initialized at this point)
        let ticks = crate::logger::get_ticks();
        MacAddress::new([
            0x52, 0x54, 0x00,  // QEMU OUI prefix
            ((ticks >> 8) & 0xFF) as u8,
            ((ticks >> 16) & 0xFF) as u8,
            (ticks & 0xFF) as u8,
        ])
    };
    
    let interface = NetworkInterface {
        mac,
        ip: Some(Ipv4Address::new(192, 168, 56, 100)),  // VirtualBox Host-Only default
        subnet: Some(Ipv4Address::new(255, 255, 255, 0)),
        gateway: Some(Ipv4Address::new(192, 168, 56, 1)), // VirtualBox Host-Only gateway (host)
        state: NetworkState::Up,
    };
    
    *INTERFACE.lock() = Some(interface);
    INITIALIZED.store(true, Ordering::SeqCst);
    
    crate::log!("[NET] Interface up: MAC={}", mac);
}

/// Check if network is available
pub fn is_available() -> bool {
    INITIALIZED.load(Ordering::Relaxed)
}

/// Get interface information
pub fn get_interface() -> Option<(MacAddress, Option<Ipv4Address>, NetworkState)> {
    let iface = INTERFACE.lock();
    iface.as_ref().map(|i| (i.mac, i.ip, i.state))
}

/// Get IPv4 configuration (ip, subnet, gateway)
pub fn get_ipv4_config() -> Option<(Ipv4Address, Ipv4Address, Option<Ipv4Address>)> {
    let iface = INTERFACE.lock();
    iface.as_ref().and_then(|i| {
        let ip = i.ip?;
        let subnet = i.subnet.unwrap_or(Ipv4Address::new(255, 255, 255, 0));
        Some((ip, subnet, i.gateway))
    })
}

/// Get MAC address from configured interface
pub fn get_mac_address() -> Option<[u8; 6]> {
    let iface = INTERFACE.lock();
    iface.as_ref().map(|i| *i.mac.as_bytes())
}

/// Get network statistics
pub fn get_stats() -> NetworkStats {
    *STATS.lock()
}

/// Send a raw ethernet frame
pub fn send_packet(data: &[u8]) -> Result<(), &'static str> {
    if !INITIALIZED.load(Ordering::Relaxed) {
        return Err("Network not initialized");
    }
    
    if data.len() > MAX_PACKET_SIZE {
        return Err("Packet too large");
    }
    
    // Try universal driver system first
    if crate::drivers::net::has_driver() {
        return crate::drivers::net::send(data);
    }
    
    // Fallback to legacy virtio-net driver
    if crate::virtio_net::is_initialized() {
        return crate::virtio_net::send_packet(data);
    }
    
    // Fallback to queue-based simulation
    let mut tx = TX_QUEUE.lock();
    tx.push_back(data.to_vec());
    
    // Update stats
    let mut stats = STATS.lock();
    stats.packets_sent += 1;
    stats.bytes_sent += data.len() as u64;
    
    Ok(())
}

/// Receive a packet (non-blocking)
pub fn receive_packet() -> Option<Vec<u8>> {
    if !INITIALIZED.load(Ordering::Relaxed) {
        return None;
    }
    
    // Try universal driver system first
    if crate::drivers::net::has_driver() {
        if let Some(packet) = crate::drivers::net::receive() {
            let mut stats = STATS.lock();
            stats.packets_received += 1;
            stats.bytes_received += packet.len() as u64;
            return Some(packet);
        }
    }
    
    // Try legacy virtio-net driver
    if crate::virtio_net::is_initialized() {
        if let Some(packet) = crate::virtio_net::receive_packet() {
            let mut stats = STATS.lock();
            stats.packets_received += 1;
            stats.bytes_received += packet.len() as u64;
            return Some(packet);
        }
    }
    
    // Fallback to queue-based
    let mut rx = RX_QUEUE.lock();
    if let Some(packet) = rx.pop_front() {
        let mut stats = STATS.lock();
        stats.packets_received += 1;
        stats.bytes_received += packet.len() as u64;
        Some(packet)
    } else {
        None
    }
}

/// Simulate receiving a packet (for testing)
pub fn inject_packet(data: Vec<u8>) {
    let mut rx = RX_QUEUE.lock();
    rx.push_back(data);
}

/// Send an ARP request
pub fn send_arp_request(target_ip: Ipv4Address) -> Result<(), &'static str> {
    let iface = INTERFACE.lock();
    let interface = iface.as_ref().ok_or("No interface")?;
    
    let mut packet = Vec::with_capacity(42); // Eth header (14) + ARP (28)
    
    // Ethernet header
    packet.extend_from_slice(&[0xFF; 6]); // Broadcast
    packet.extend_from_slice(interface.mac.as_bytes());
    packet.push(0x08); packet.push(0x06); // ARP ethertype
    
    // ARP packet
    packet.push(0x00); packet.push(0x01); // Hardware type: Ethernet
    packet.push(0x08); packet.push(0x00); // Protocol type: IPv4
    packet.push(0x06);                     // Hardware size: 6
    packet.push(0x04);                     // Protocol size: 4
    packet.push(0x00); packet.push(0x01); // Opcode: Request
    packet.extend_from_slice(interface.mac.as_bytes()); // Sender MAC
    if let Some(ip) = interface.ip {
        packet.extend_from_slice(ip.as_bytes()); // Sender IP
    } else {
        packet.extend_from_slice(&[0; 4]);
    }
    packet.extend_from_slice(&[0; 6]);    // Target MAC (unknown)
    packet.extend_from_slice(target_ip.as_bytes()); // Target IP
    
    drop(iface);
    send_packet(&packet)
}

/// Send a ping (ICMP echo request) and simulate reply
pub fn send_ping(target_ip: Ipv4Address) -> Result<PingResult, &'static str> {
    // Sequence number
    static SEQ: AtomicU64 = AtomicU64::new(1);
    let seq = SEQ.fetch_add(1, Ordering::Relaxed) as u16;
    
    // Clear old responses
    crate::netstack::icmp::clear_responses();
    
    // Get start time using high-precision TSC if available
    let start_micros = crate::cpu::tsc::now_micros();
    
    // Send ICMP echo request via network stack
    let ip_bytes = target_ip.as_bytes();
    crate::netstack::icmp::send_echo_request([ip_bytes[0], ip_bytes[1], ip_bytes[2], ip_bytes[3]], 0x1234, seq)?;
    
    // Poll network for responses (timeout 1 second)
    let response = crate::netstack::icmp::wait_for_response(seq, 1000);
    
    // Calculate elapsed time in microseconds
    let elapsed_micros = crate::cpu::tsc::now_micros().saturating_sub(start_micros);
    
    // Convert to milliseconds for display, but show sub-millisecond precision
    let time_ms = if elapsed_micros < 1000 {
        // Less than 1ms - round up to 1ms for display but we have the real value
        1
    } else {
        (elapsed_micros / 1000) as u32
    };
    
    if let Some(resp) = response {
        // Update stats
        let mut stats = STATS.lock();
        stats.packets_received += 1;
        stats.bytes_received += 64; // ICMP reply size
        
        Ok(PingResult {
            seq: resp.seq,
            ttl: resp.ttl,
            time_ms,
            time_us: elapsed_micros,
            success: true,
        })
    } else {
        // Timeout
        Ok(PingResult {
            seq,
            ttl: 0,
            time_ms,
            time_us: elapsed_micros,
            success: false,
        })
    }
}

/// Ping result
#[derive(Debug, Clone)]
pub struct PingResult {
    pub seq: u16,
    pub ttl: u8,
    pub time_ms: u32,
    pub time_us: u64,  // Microsecond precision from TSC
    pub success: bool,
}

/// Simulate ping response based on target IP
fn simulate_ping_response(target: &Ipv4Address) -> u32 {
    let bytes = target.as_bytes();
    
    // Loopback (127.x.x.x) - instant
    if bytes[0] == 127 {
        return 1;
    }
    
    // Gateway (10.0.2.2) - fast
    if bytes[0] == 10 && bytes[1] == 0 && bytes[2] == 2 && bytes[3] == 2 {
        return 2;
    }
    
    // Local network (10.0.2.x) - fast
    if bytes[0] == 10 && bytes[1] == 0 && bytes[2] == 2 {
        return 5;
    }
    
    // Local network (192.168.x.x, 10.x.x.x) - moderate
    if bytes[0] == 192 && bytes[1] == 168 {
        return 10;
    }
    if bytes[0] == 10 {
        return 15;
    }
    
    // Public IPs - simulate internet latency
    // 8.8.8.8 (Google DNS)
    if bytes[0] == 8 && bytes[1] == 8 && bytes[2] == 8 && bytes[3] == 8 {
        return 25;
    }
    
    // 1.1.1.1 (Cloudflare)
    if bytes[0] == 1 && bytes[1] == 1 && bytes[2] == 1 && bytes[3] == 1 {
        return 20;
    }
    
    // Other public IPs - higher latency
    50
}

/// Set IP address
pub fn set_ip(ip: Ipv4Address, subnet: Ipv4Address, gateway: Ipv4Address) {
    let mut iface = INTERFACE.lock();
    if let Some(ref mut interface) = *iface {
        interface.ip = Some(ip);
        interface.subnet = Some(subnet);
        interface.gateway = Some(gateway);
        crate::log!("[NET] IP configured: {}", ip);
    }
}

/// Set IPv4 configuration (called by DHCP)
pub fn set_ipv4_config(ip: Ipv4Address, subnet: Ipv4Address, gateway: Option<Ipv4Address>) {
    let mut iface = INTERFACE.lock();
    if let Some(ref mut interface) = *iface {
        interface.ip = Some(ip);
        interface.subnet = Some(subnet);
        interface.gateway = gateway;
        interface.state = NetworkState::Up;
    }
}

/// Bring interface up
pub fn up() {
    let mut iface = INTERFACE.lock();
    if let Some(ref mut interface) = *iface {
        interface.state = NetworkState::Up;
        crate::log!("[NET] Interface up");
    }
}

/// Bring interface down
pub fn down() {
    let mut iface = INTERFACE.lock();
    if let Some(ref mut interface) = *iface {
        interface.state = NetworkState::Down;
        crate::log!("[NET] Interface down");
    }
}

/// Update MAC address from virtio driver (called after driver init)
pub fn update_mac_from_driver() {
    // Try universal driver system first
    if let Some(real_mac) = crate::drivers::net::get_mac() {
        let mut iface = INTERFACE.lock();
        if let Some(ref mut interface) = *iface {
            interface.mac = MacAddress::new(real_mac);
            crate::log!("[NET] MAC updated: {}", interface.mac);
        }
        return;
    }
    
    // Fallback to legacy virtio-net
    if let Some(real_mac) = crate::virtio_net::get_mac() {
        let mut iface = INTERFACE.lock();
        if let Some(ref mut interface) = *iface {
            interface.mac = MacAddress::new(real_mac);
            crate::log!("[NET] MAC updated: {}", interface.mac);
        }
    }
}

/// Poll network driver (call periodically to process packets)
pub fn poll() {
    if crate::drivers::net::has_driver() {
        crate::netstack::poll(); // Also processes packets through stack
    } else if crate::virtio_net::is_initialized() {
        crate::virtio_net::poll();
    }
}

/// Check if real driver is active (not simulation)
pub fn has_real_driver() -> bool {
    crate::drivers::net::has_driver() || crate::virtio_net::is_initialized()
}

/// Get driver statistics
pub fn get_driver_stats() -> (u64, u64, u64, u64) {
    if crate::drivers::net::has_driver() {
        let stats = crate::drivers::net::stats();
        (stats.tx_packets, stats.rx_packets, stats.tx_bytes, stats.rx_bytes)
    } else if crate::virtio_net::is_initialized() {
        crate::virtio_net::get_stats()
    } else {
        let stats = STATS.lock();
        (stats.packets_sent, stats.packets_received, stats.bytes_sent, stats.bytes_received)
    }
}
