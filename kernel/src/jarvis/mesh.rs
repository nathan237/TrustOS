//! JARVIS Mesh — Peer Discovery, Heartbeat & Cluster Management
//!
//! Enables JARVIS instances across multiple TrustOS machines to discover each other
//! on the LAN via UDP broadcast, maintain heartbeats, and form a coordinated cluster.
//!
//! # Protocol
//!
//! ```text
//! Discovery:  UDP broadcast on port 7700 every 5s
//! Heartbeat:  UDP unicast to known peers every 3s
//! Timeout:    Peer removed after 15s without heartbeat
//! ```
//!
//! # Packet Format (all fields big-endian)
//!
//! ```text
//! [0..4]   Magic: b"JMSH"
//! [4]      Message type: 0=Announce, 1=Heartbeat, 2=Leave
//! [5..9]   Sender IP (4 bytes)
//! [9..15]  Sender MAC (6 bytes)
//! [15]     Role: 0=Worker, 1=Leader, 2=Candidate
//! [16..24] Uptime ms (u64)
//! [24..28] Param count (u32) — model size
//! [28..32] Training steps (u32)
//! [32..34] CPU cores (u16)
//! [34..38] RAM MB (u32)
//! [38..40] Mesh port (u16) — RPC port for this node
//! [40]     CPU architecture: 0=x86_64, 1=aarch64, 2=riscv64
//! ```

use alloc::vec::Vec;
use alloc::string::String;
use alloc::format;
use core::sync::atomic::{AtomicBool, AtomicU8, AtomicU64, Ordering};
use spin::Mutex;

// ═══════════════════════════════════════════════════════════════════════════════
// Constants
// ═══════════════════════════════════════════════════════════════════════════════

/// UDP port for mesh discovery broadcasts
pub const MESH_DISCOVERY_PORT: u16 = 7700;

/// TCP port for mesh RPC communication
pub const MESH_RPC_PORT: u16 = 7701;

/// Magic bytes identifying JARVIS mesh packets
const MAGIC: &[u8; 4] = b"JMSH";

/// Packet size for announce/heartbeat (v2: 41 bytes with arch field)
const PACKET_SIZE: usize = 41;

/// Broadcast interval in ms
const ANNOUNCE_INTERVAL_MS: u64 = 5000;

/// Heartbeat interval in ms
const HEARTBEAT_INTERVAL_MS: u64 = 3000;

/// Peer timeout in ms (no heartbeat → dead)
const PEER_TIMEOUT_MS: u64 = 15000;

/// Maximum peers in cluster
const MAX_PEERS: usize = 64;

/// CPU architecture identifier for cross-arch mesh awareness
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CpuArch {
    X86_64 = 0,
    Aarch64 = 1,
    Riscv64 = 2,
    Unknown = 255,
}

impl CpuArch {
    fn from_byte(b: u8) -> Self {
        match b {
            0 => CpuArch::X86_64,
            1 => CpuArch::Aarch64,
            2 => CpuArch::Riscv64,
            _ => CpuArch::Unknown,
        }
    }

    /// Architecture of the currently running kernel
    pub fn current() -> Self {
        #[cfg(target_arch = "x86_64")]
        { CpuArch::X86_64 }
        #[cfg(target_arch = "aarch64")]
        { CpuArch::Aarch64 }
        #[cfg(target_arch = "riscv64")]
        { CpuArch::Riscv64 }
    }

    pub fn name(&self) -> &'static str {
        match self {
            CpuArch::X86_64 => "x86_64",
            CpuArch::Aarch64 => "aarch64",
            CpuArch::Riscv64 => "riscv64",
            CpuArch::Unknown => "unknown",
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Types
// ═══════════════════════════════════════════════════════════════════════════════

/// Message types in the mesh protocol
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MsgType {
    Announce = 0,
    Heartbeat = 1,
    Leave = 2,
}

/// Node role in the cluster
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum NodeRole {
    Worker = 0,
    Leader = 1,
    Candidate = 2,
}

/// Information about a peer node in the mesh
#[derive(Debug, Clone)]
pub struct PeerInfo {
    /// IPv4 address
    pub ip: [u8; 4],
    /// MAC address
    pub mac: [u8; 6],
    /// Current role
    pub role: NodeRole,
    /// Uptime in ms
    pub uptime_ms: u64,
    /// Model parameter count
    pub param_count: u32,
    /// Training steps completed
    pub training_steps: u32,
    /// CPU cores available
    pub cpu_cores: u16,
    /// RAM in MB
    pub ram_mb: u32,
    /// RPC port
    pub rpc_port: u16,
    /// CPU architecture of this peer
    pub arch: CpuArch,
    /// Last time we heard from this peer (our local uptime_ms)
    pub last_seen_ms: u64,
    /// Is this peer alive?
    pub alive: bool,
}

impl PeerInfo {
    /// Format as human-readable string
    pub fn display(&self) -> String {
        let role_str = match self.role {
            NodeRole::Worker => "Worker",
            NodeRole::Leader => "Leader",
            NodeRole::Candidate => "Candidate",
        };
        format!("{}.{}.{}.{}  arch={}  role={}  params={}  steps={}  cores={}  ram={}MB",
            self.ip[0], self.ip[1], self.ip[2], self.ip[3],
            self.arch.name(), role_str, self.param_count, self.training_steps,
            self.cpu_cores, self.ram_mb)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Global State
// ═══════════════════════════════════════════════════════════════════════════════

/// Whether mesh networking is active
static MESH_ACTIVE: AtomicBool = AtomicBool::new(false);

/// Our current role
static OUR_ROLE: AtomicU8 = AtomicU8::new(0); // 0 = Worker

/// Known peers in the cluster
static PEERS: Mutex<Vec<PeerInfo>> = Mutex::new(Vec::new());

/// Last time we sent an announce broadcast
static LAST_ANNOUNCE_MS: AtomicU64 = AtomicU64::new(0);

/// Last time we sent heartbeats
static LAST_HEARTBEAT_MS: AtomicU64 = AtomicU64::new(0);

/// Total peers discovered since start
static PEERS_DISCOVERED: AtomicU64 = AtomicU64::new(0);

// ═══════════════════════════════════════════════════════════════════════════════
// Public API
// ═══════════════════════════════════════════════════════════════════════════════

/// Start the JARVIS mesh network
pub fn start() {
    if MESH_ACTIVE.load(Ordering::SeqCst) {
        crate::serial_println!("[MESH] Already active");
        return;
    }

    if !crate::network::is_available() {
        crate::serial_println!("[MESH] Network not available — cannot start mesh");
        return;
    }

    // Ensure we have an IP (MAC-derived fallback if DHCP hasn't run)
    crate::network::ensure_fallback_ip();

    MESH_ACTIVE.store(true, Ordering::SeqCst);
    PEERS.lock().clear();
    LAST_ANNOUNCE_MS.store(0, Ordering::SeqCst);
    LAST_HEARTBEAT_MS.store(0, Ordering::SeqCst);

    crate::serial_println!("[MESH] JARVIS mesh started on port {}", MESH_DISCOVERY_PORT);

    // Send initial announce immediately
    send_announce();
}

/// Stop the mesh network gracefully
pub fn stop() {
    if !MESH_ACTIVE.load(Ordering::SeqCst) {
        return;
    }

    // Send leave message to all peers
    send_leave();

    MESH_ACTIVE.store(false, Ordering::SeqCst);
    PEERS.lock().clear();
    crate::serial_println!("[MESH] JARVIS mesh stopped");
}

/// Check if mesh is active
pub fn is_active() -> bool {
    MESH_ACTIVE.load(Ordering::SeqCst)
}

/// Get our current role
pub fn our_role() -> NodeRole {
    match OUR_ROLE.load(Ordering::SeqCst) {
        1 => NodeRole::Leader,
        2 => NodeRole::Candidate,
        _ => NodeRole::Worker,
    }
}

/// Set our role
pub fn set_role(role: NodeRole) {
    OUR_ROLE.store(role as u8, Ordering::SeqCst);
}

/// Get list of alive peers
pub fn get_peers() -> Vec<PeerInfo> {
    let peers = PEERS.lock();
    peers.iter().filter(|p| p.alive).cloned().collect()
}

/// Get count of alive peers
pub fn peer_count() -> usize {
    let peers = PEERS.lock();
    peers.iter().filter(|p| p.alive).count()
}

/// Get total peers discovered since start
pub fn total_discovered() -> u64 {
    PEERS_DISCOVERED.load(Ordering::SeqCst)
}

/// Get the leader node (if any)
pub fn get_leader() -> Option<PeerInfo> {
    let peers = PEERS.lock();
    peers.iter().find(|p| p.alive && p.role == NodeRole::Leader).cloned()
}

/// Poll the mesh — call this regularly from the main loop
/// Handles: receiving discovery packets, sending announces/heartbeats, expiring peers
pub fn poll() {
    if !MESH_ACTIVE.load(Ordering::SeqCst) {
        return;
    }

    let now = crate::time::uptime_ms();

    // 1. Receive and process incoming mesh packets
    while let Some(data) = crate::netstack::udp::recv_on(MESH_DISCOVERY_PORT) {
        handle_incoming(&data, now);
    }

    // 2. Send periodic announce broadcast
    let last_announce = LAST_ANNOUNCE_MS.load(Ordering::SeqCst);
    if now.wrapping_sub(last_announce) >= ANNOUNCE_INTERVAL_MS {
        send_announce();
        LAST_ANNOUNCE_MS.store(now, Ordering::SeqCst);
    }

    // 3. Send periodic heartbeats to known peers
    let last_hb = LAST_HEARTBEAT_MS.load(Ordering::SeqCst);
    if now.wrapping_sub(last_hb) >= HEARTBEAT_INTERVAL_MS {
        send_heartbeats();
        LAST_HEARTBEAT_MS.store(now, Ordering::SeqCst);
    }

    // 4. Expire dead peers
    expire_peers(now);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Packet Construction
// ═══════════════════════════════════════════════════════════════════════════════

/// Build a mesh packet with our current info
fn build_packet(msg_type: MsgType) -> [u8; PACKET_SIZE] {
    let mut pkt = [0u8; PACKET_SIZE];

    // Magic
    pkt[0..4].copy_from_slice(MAGIC);

    // Message type
    pkt[4] = msg_type as u8;

    // Our IP
    if let Some((ip, _, _)) = crate::network::get_ipv4_config() {
        pkt[5..9].copy_from_slice(ip.as_bytes());
    }

    // Our MAC
    if let Some(mac) = crate::network::get_mac_address() {
        pkt[9..15].copy_from_slice(&mac);
    }

    // Role
    pkt[15] = OUR_ROLE.load(Ordering::SeqCst);

    // Uptime
    let uptime = crate::time::uptime_ms();
    pkt[16..24].copy_from_slice(&uptime.to_be_bytes());

    // Param count
    let params = if super::is_ready() {
        super::MODEL.lock().as_ref().map(|m| m.param_count() as u32).unwrap_or(0)
    } else {
        0
    };
    pkt[24..28].copy_from_slice(&params.to_be_bytes());

    // Training steps
    let steps = super::TRAINING_STEPS.load(Ordering::SeqCst) as u32;
    pkt[28..32].copy_from_slice(&steps.to_be_bytes());

    // CPU cores (use 1 as minimum)
    let cores: u16 = 1; // TODO: detect from SMP
    pkt[32..34].copy_from_slice(&cores.to_be_bytes());

    // RAM MB (rough estimate from heap)
    let ram_mb: u32 = 256; // TODO: detect from memory subsystem
    pkt[34..38].copy_from_slice(&ram_mb.to_be_bytes());

    // RPC port
    pkt[38..40].copy_from_slice(&MESH_RPC_PORT.to_be_bytes());

    // CPU architecture
    pkt[40] = CpuArch::current() as u8;

    pkt
}

/// Parse an incoming mesh packet
fn parse_packet(data: &[u8]) -> Option<(MsgType, PeerInfo)> {
    if data.len() < PACKET_SIZE {
        return None;
    }

    // Verify magic
    if &data[0..4] != MAGIC {
        return None;
    }

    let msg_type = match data[4] {
        0 => MsgType::Announce,
        1 => MsgType::Heartbeat,
        2 => MsgType::Leave,
        _ => return None,
    };

    let mut ip = [0u8; 4];
    ip.copy_from_slice(&data[5..9]);

    let mut mac = [0u8; 6];
    mac.copy_from_slice(&data[9..15]);

    let role = match data[15] {
        1 => NodeRole::Leader,
        2 => NodeRole::Candidate,
        _ => NodeRole::Worker,
    };

    let uptime = u64::from_be_bytes([
        data[16], data[17], data[18], data[19],
        data[20], data[21], data[22], data[23],
    ]);

    let param_count = u32::from_be_bytes([data[24], data[25], data[26], data[27]]);
    let training_steps = u32::from_be_bytes([data[28], data[29], data[30], data[31]]);
    let cpu_cores = u16::from_be_bytes([data[32], data[33]]);
    let ram_mb = u32::from_be_bytes([data[34], data[35], data[36], data[37]]);
    let rpc_port = u16::from_be_bytes([data[38], data[39]]);

    // CPU architecture (byte 40, optional for backward compat with v1 40-byte packets)
    let arch = if data.len() > 40 {
        CpuArch::from_byte(data[40])
    } else {
        CpuArch::Unknown
    };

    Some((msg_type, PeerInfo {
        ip,
        mac,
        role,
        uptime_ms: uptime,
        param_count,
        training_steps,
        cpu_cores,
        ram_mb,
        rpc_port,
        arch,
        last_seen_ms: 0, // caller fills this
        alive: true,
    }))
}

// ═══════════════════════════════════════════════════════════════════════════════
// Sending
// ═══════════════════════════════════════════════════════════════════════════════

/// Send an announce broadcast to the LAN
fn send_announce() {
    let pkt = build_packet(MsgType::Announce);
    let broadcast_ip = get_broadcast_ip();
    let src_port = MESH_DISCOVERY_PORT;

    if let Err(e) = crate::netstack::udp::send_to(broadcast_ip, MESH_DISCOVERY_PORT, src_port, &pkt) {
        crate::serial_println!("[MESH] Announce send failed: {}", e);
    }
}

/// Send heartbeat to all known alive peers
fn send_heartbeats() {
    let pkt = build_packet(MsgType::Heartbeat);
    let peers = PEERS.lock();

    for peer in peers.iter().filter(|p| p.alive) {
        let _ = crate::netstack::udp::send_to(peer.ip, MESH_DISCOVERY_PORT, MESH_DISCOVERY_PORT, &pkt);
    }
}

/// Send leave notification to all peers and broadcast
fn send_leave() {
    let pkt = build_packet(MsgType::Leave);
    let broadcast_ip = get_broadcast_ip();
    let _ = crate::netstack::udp::send_to(broadcast_ip, MESH_DISCOVERY_PORT, MESH_DISCOVERY_PORT, &pkt);

    let peers = PEERS.lock();
    for peer in peers.iter().filter(|p| p.alive) {
        let _ = crate::netstack::udp::send_to(peer.ip, MESH_DISCOVERY_PORT, MESH_DISCOVERY_PORT, &pkt);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Receiving
// ═══════════════════════════════════════════════════════════════════════════════

/// Handle an incoming mesh packet
fn handle_incoming(data: &[u8], now_ms: u64) {
    let (msg_type, mut peer_info) = match parse_packet(data) {
        Some(v) => v,
        None => return,
    };

    // Ignore packets from ourselves
    if is_our_ip(peer_info.ip) {
        return;
    }

    peer_info.last_seen_ms = now_ms;

    match msg_type {
        MsgType::Announce | MsgType::Heartbeat => {
            let is_new = !peer_known(peer_info.ip);
            update_or_add_peer(peer_info);
            // Respond immediately to new peer announcements so they discover us fast
            if is_new && msg_type == MsgType::Announce {
                send_announce();
            }
        }
        MsgType::Leave => {
            remove_peer(peer_info.ip);
        }
    }
}

/// Check if a peer with this IP is already known
fn peer_known(ip: [u8; 4]) -> bool {
    let peers = PEERS.lock();
    peers.iter().any(|p| p.ip == ip)
}

/// Update an existing peer or add a new one
fn update_or_add_peer(info: PeerInfo) {
    let mut peers = PEERS.lock();

    // Check if peer already known
    for peer in peers.iter_mut() {
        if peer.ip == info.ip {
            peer.role = info.role;
            peer.uptime_ms = info.uptime_ms;
            peer.param_count = info.param_count;
            peer.training_steps = info.training_steps;
            peer.cpu_cores = info.cpu_cores;
            peer.ram_mb = info.ram_mb;
            peer.rpc_port = info.rpc_port;
            peer.arch = info.arch;
            peer.last_seen_ms = info.last_seen_ms;
            peer.alive = true;
            return;
        }
    }

    // New peer
    if peers.len() < MAX_PEERS {
        crate::serial_println!("[MESH] New peer discovered: {}.{}.{}.{} (arch={}, role={:?}, params={})",
            info.ip[0], info.ip[1], info.ip[2], info.ip[3],
            info.arch.name(), info.role, info.param_count);
        PEERS_DISCOVERED.fetch_add(1, Ordering::SeqCst);
        peers.push(info);
    }
}

/// Remove a peer by IP (they sent Leave)
fn remove_peer(ip: [u8; 4]) {
    let mut peers = PEERS.lock();
    if let Some(peer) = peers.iter_mut().find(|p| p.ip == ip) {
        crate::serial_println!("[MESH] Peer left: {}.{}.{}.{}",
            ip[0], ip[1], ip[2], ip[3]);
        peer.alive = false;
    }
}

/// Expire peers that haven't been heard from in PEER_TIMEOUT_MS
fn expire_peers(now_ms: u64) {
    let mut peers = PEERS.lock();
    for peer in peers.iter_mut().filter(|p| p.alive) {
        if now_ms.wrapping_sub(peer.last_seen_ms) > PEER_TIMEOUT_MS {
            crate::serial_println!("[MESH] Peer timed out: {}.{}.{}.{}",
                peer.ip[0], peer.ip[1], peer.ip[2], peer.ip[3]);
            peer.alive = false;
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Helpers
// ═══════════════════════════════════════════════════════════════════════════════

/// Get the LAN broadcast address (ip | ~subnet)
fn get_broadcast_ip() -> [u8; 4] {
    if let Some((ip, subnet, _)) = crate::network::get_ipv4_config() {
        let ip_b = ip.as_bytes();
        let sn_b = subnet.as_bytes();
        [
            ip_b[0] | !sn_b[0],
            ip_b[1] | !sn_b[1],
            ip_b[2] | !sn_b[2],
            ip_b[3] | !sn_b[3],
        ]
    } else {
        [255, 255, 255, 255] // fallback: global broadcast
    }
}

/// Check if an IP is our own
fn is_our_ip(ip: [u8; 4]) -> bool {
    if let Some((our_ip, _, _)) = crate::network::get_ipv4_config() {
        *our_ip.as_bytes() == ip
    } else {
        false
    }
}

/// Get status summary string
pub fn status_summary() -> String {
    let alive = peer_count();
    let role = our_role();
    let role_str = match role {
        NodeRole::Leader => "Leader",
        NodeRole::Worker => "Worker",
        NodeRole::Candidate => "Candidate",
    };

    if !is_active() {
        return String::from("Mesh: inactive");
    }

    format!("Mesh: {} | Role: {} | Peers: {} alive | Discovered: {}",
        if is_active() { "active" } else { "inactive" },
        role_str,
        alive,
        total_discovered())
}
