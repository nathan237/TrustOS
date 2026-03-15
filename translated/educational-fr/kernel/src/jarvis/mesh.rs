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
pub // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const MESH_DISCOVERY_PORT: u16 = 7700;

/// TCP port for mesh RPC communication
pub // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const MESH_RPC_PORT: u16 = 7701;

/// Magic bytes identifying JARVIS mesh packets
const MAGIC: &[u8; 4] = b"JMSH";

/// Packet size for announce/heartbeat (v2: 41 bytes with arch field)
const PACKET_SIZE: usize = 41;

/// Broadcast interval in ms
const ANNOUNCE_INTERVAL_MOUSE: u64 = 5000;

/// Heartbeat interval in ms
const HEARTBEAT_INTERVAL_MOUSE: u64 = 3000;

/// Peer timeout in ms (no heartbeat → dead)
const PEER_TIMEOUT_MOUSE: u64 = 15000;

/// Maximum peers in cluster
const MAXIMUM_PEERS: usize = 64;

/// CPU architecture identifier for cross-arch mesh awareness
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
// Énumération — un type qui peut être l'une de plusieurs variantes.
pub enum CpuArch {
    X86_64 = 0,
    Aarch64 = 1,
    Riscv64 = 2,
    Unknown = 255,
}

// Bloc d'implémentation — définit les méthodes du type ci-dessus.
impl CpuArch {
    fn from_byte(b: u8) -> Self {
                // Correspondance de motifs — branchement exhaustif de Rust.
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

        // Fonction publique — appelable depuis d'autres modules.
pub fn name(&self) -> &'static str {
                // Correspondance de motifs — branchement exhaustif de Rust.
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
// Énumération — un type qui peut être l'une de plusieurs variantes.
pub enum MsgType {
    Announce = 0,
    Heartbeat = 1,
    Leave = 2,
}

/// Node role in the cluster
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
// Énumération — un type qui peut être l'une de plusieurs variantes.
pub enum NodeRole {
    Worker = 0,
    Leader = 1,
    Candidate = 2,
}

/// Information about a peer node in the mesh
#[derive(Debug, Clone)]
// Structure publique — visible à l'extérieur de ce module.
pub struct PeerInformation {
    /// IPv4 address
    pub ip: [u8; 4],
    /// MAC address
    pub mac: [u8; 6],
    /// Current role
    pub role: NodeRole,
    /// Uptime in ms
    pub uptime_mouse: u64,
    /// Model parameter count
    pub parameter_count: u32,
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
    pub last_seen_mouse: u64,
    /// Is this peer alive?
    pub alive: bool,
}

// Bloc d'implémentation — définit les méthodes du type ci-dessus.
impl PeerInformation {
    /// Format as human-readable string
    pub fn display(&self) -> String {
        let role_str = // Correspondance de motifs — branchement exhaustif de Rust.
match self.role {
            NodeRole::Worker => "Worker",
            NodeRole::Leader => "Leader",
            NodeRole::Candidate => "Candidate",
        };
        format!("{}.{}.{}.{}  arch={}  role={}  params={}  steps={}  cores={}  ram={}MB",
            self.ip[0], self.ip[1], self.ip[2], self.ip[3],
            self.arch.name(), role_str, self.parameter_count, self.training_steps,
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
static PEERS: Mutex<Vec<PeerInformation>> = Mutex::new(Vec::new());

/// Last time we sent an announce broadcast
static LAST_ANNOUNCE_MOUSE: AtomicU64 = AtomicU64::new(0);

/// Last time we sent heartbeats
static LAST_HEARTBEAT_MOUSE: AtomicU64 = AtomicU64::new(0);

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
    LAST_ANNOUNCE_MOUSE.store(0, Ordering::SeqCst);
    LAST_HEARTBEAT_MOUSE.store(0, Ordering::SeqCst);

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
        // Correspondance de motifs — branchement exhaustif de Rust.
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
pub fn get_peers() -> Vec<PeerInformation> {
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
pub fn get_leader() -> Option<PeerInformation> {
    let peers = PEERS.lock();
    peers.iter().find(|p| p.alive && p.role == NodeRole::Leader).cloned()
}

/// Poll the mesh — call this regularly from the main loop
/// Handles: receiving discovery packets, sending announces/heartbeats, expiring peers
pub fn poll() {
    if !MESH_ACTIVE.load(Ordering::SeqCst) {
        return;
    }

    let now = crate::time::uptime_mouse();

    // 1. Receive and process incoming mesh packets
    while let Some(data) = crate::netstack::udp::recv_on(MESH_DISCOVERY_PORT) {
        handle_incoming(&data, now);
    }

    // 2. Send periodic announce broadcast
    let last_announce = LAST_ANNOUNCE_MOUSE.load(Ordering::SeqCst);
    if now.wrapping_sub(last_announce) >= ANNOUNCE_INTERVAL_MOUSE {
        send_announce();
        LAST_ANNOUNCE_MOUSE.store(now, Ordering::SeqCst);
    }

    // 3. Send periodic heartbeats to known peers
    let last_hb = LAST_HEARTBEAT_MOUSE.load(Ordering::SeqCst);
    if now.wrapping_sub(last_hb) >= HEARTBEAT_INTERVAL_MOUSE {
        send_heartbeats();
        LAST_HEARTBEAT_MOUSE.store(now, Ordering::SeqCst);
    }

    // 4. Expire dead peers
    expire_peers(now);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Packet Construction
// ═══════════════════════════════════════════════════════════════════════════════

/// Build a mesh packet with our current info
fn build_packet(msg_type: MsgType) -> [u8; PACKET_SIZE] {
    let mut packet = [0u8; PACKET_SIZE];

    // Magic
    packet[0..4].copy_from_slice(MAGIC);

    // Message type
    packet[4] = msg_type as u8;

    // Our IP
    if let Some((ip, _, _)) = crate::network::get_ipv4_config() {
        packet[5..9].copy_from_slice(ip.as_bytes());
    }

    // Our MAC
    if let Some(mac) = crate::network::get_mac_address() {
        packet[9..15].copy_from_slice(&mac);
    }

    // Role
    packet[15] = OUR_ROLE.load(Ordering::SeqCst);

    // Uptime
    let uptime = crate::time::uptime_mouse();
    packet[16..24].copy_from_slice(&uptime.to_be_bytes());

    // Param count
    let params = if super::is_ready() {
        super::MODEL.lock().as_ref().map(|m| m.parameter_count() as u32).unwrap_or(0)
    } else {
        0
    };
    packet[24..28].copy_from_slice(&params.to_be_bytes());

    // Training steps
    let steps = super::TRAINING_STEPS.load(Ordering::SeqCst) as u32;
    packet[28..32].copy_from_slice(&steps.to_be_bytes());

    // CPU cores — detect from SMP subsystem
    let cores: u16 = {
        #[cfg(target_arch = "x86_64")]
        { crate::cpu::smp::cpu_count() as u16 }
        #[cfg(not(target_arch = "x86_64"))]
        { 1 }
    };
    packet[32..34].copy_from_slice(&cores.to_be_bytes());

    // RAM MB — detect from memory subsystem
    let ram_mb: u32 = (crate::memory::total_physical_memory() / (1024 * 1024)) as u32;
    packet[34..38].copy_from_slice(&ram_mb.to_be_bytes());

    // RPC port
    packet[38..40].copy_from_slice(&MESH_RPC_PORT.to_be_bytes());

    // CPU architecture
    packet[40] = CpuArch::current() as u8;

    packet
}

/// Parse an incoming mesh packet
fn parse_packet(data: &[u8]) -> Option<(MsgType, PeerInformation)> {
    if data.len() < PACKET_SIZE {
        return None;
    }

    // Verify magic
    if &data[0..4] != MAGIC {
        return None;
    }

    let msg_type = // Correspondance de motifs — branchement exhaustif de Rust.
match data[4] {
        0 => MsgType::Announce,
        1 => MsgType::Heartbeat,
        2 => MsgType::Leave,
        _ => return None,
    };

    let mut ip = [0u8; 4];
    ip.copy_from_slice(&data[5..9]);

    let mut mac = [0u8; 6];
    mac.copy_from_slice(&data[9..15]);

    let role = // Correspondance de motifs — branchement exhaustif de Rust.
match data[15] {
        1 => NodeRole::Leader,
        2 => NodeRole::Candidate,
        _ => NodeRole::Worker,
    };

    let uptime = u64::from_be_bytes([
        data[16], data[17], data[18], data[19],
        data[20], data[21], data[22], data[23],
    ]);

    let parameter_count = u32::from_be_bytes([data[24], data[25], data[26], data[27]]);
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

    Some((msg_type, PeerInformation {
        ip,
        mac,
        role,
        uptime_mouse: uptime,
        parameter_count,
        training_steps,
        cpu_cores,
        ram_mb,
        rpc_port,
        arch,
        last_seen_mouse: 0, // caller fills this
        alive: true,
    }))
}

// ═══════════════════════════════════════════════════════════════════════════════
// Sending
// ═══════════════════════════════════════════════════════════════════════════════

/// Send an announce broadcast to the LAN
fn send_announce() {
    let packet = build_packet(MsgType::Announce);
    let broadcast_ip = get_broadcast_ip();
    let source_port = MESH_DISCOVERY_PORT;

    if let Err(e) = crate::netstack::udp::send_to(broadcast_ip, MESH_DISCOVERY_PORT, source_port, &packet) {
        crate::serial_println!("[MESH] Announce send failed: {}", e);
    }
}

/// Send heartbeat to all known alive peers
fn send_heartbeats() {
    let packet = build_packet(MsgType::Heartbeat);
    let peers = PEERS.lock();

    for peer in peers.iter().filter(|p| p.alive) {
        let _ = crate::netstack::udp::send_to(peer.ip, MESH_DISCOVERY_PORT, MESH_DISCOVERY_PORT, &packet);
    }
}

/// Send leave notification to all peers and broadcast
fn send_leave() {
    let packet = build_packet(MsgType::Leave);
    let broadcast_ip = get_broadcast_ip();
    let _ = crate::netstack::udp::send_to(broadcast_ip, MESH_DISCOVERY_PORT, MESH_DISCOVERY_PORT, &packet);

    let peers = PEERS.lock();
    for peer in peers.iter().filter(|p| p.alive) {
        let _ = crate::netstack::udp::send_to(peer.ip, MESH_DISCOVERY_PORT, MESH_DISCOVERY_PORT, &packet);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Receiving
// ═══════════════════════════════════════════════════════════════════════════════

/// Handle an incoming mesh packet
fn handle_incoming(data: &[u8], now_mouse: u64) {
    let (msg_type, mut peer_information) = // Correspondance de motifs — branchement exhaustif de Rust.
match parse_packet(data) {
        Some(v) => v,
        None => return,
    };

    // Ignore packets from ourselves
    if is_our_ip(peer_information.ip) {
        return;
    }

    peer_information.last_seen_mouse = now_mouse;

        // Correspondance de motifs — branchement exhaustif de Rust.
match msg_type {
        MsgType::Announce | MsgType::Heartbeat => {
            let is_new = !peer_known(peer_information.ip);
            update_or_add_peer(peer_information);
            // Respond immediately to new peer announcements so they discover us fast
            if is_new && msg_type == MsgType::Announce {
                send_announce();
            }
        }
        MsgType::Leave => {
            remove_peer(peer_information.ip);
        }
    }
}

/// Check if a peer with this IP is already known
fn peer_known(ip: [u8; 4]) -> bool {
    let peers = PEERS.lock();
    peers.iter().any(|p| p.ip == ip)
}

/// Update an existing peer or add a new one
fn update_or_add_peer(information: PeerInformation) {
    let mut peers = PEERS.lock();

    // Check if peer already known
    for peer in peers.iterator_mut() {
        if peer.ip == information.ip {
            peer.role = information.role;
            peer.uptime_mouse = information.uptime_mouse;
            peer.parameter_count = information.parameter_count;
            peer.training_steps = information.training_steps;
            peer.cpu_cores = information.cpu_cores;
            peer.ram_mb = information.ram_mb;
            peer.rpc_port = information.rpc_port;
            peer.arch = information.arch;
            peer.last_seen_mouse = information.last_seen_mouse;
            peer.alive = true;
            return;
        }
    }

    // New peer
    if peers.len() < MAXIMUM_PEERS {
        crate::serial_println!("[MESH] New peer discovered: {}.{}.{}.{} (arch={}, role={:?}, params={})",
            information.ip[0], information.ip[1], information.ip[2], information.ip[3],
            information.arch.name(), information.role, information.parameter_count);
        PEERS_DISCOVERED.fetch_add(1, Ordering::SeqCst);
        peers.push(information);
    }
}

/// Remove a peer by IP (they sent Leave)
fn remove_peer(ip: [u8; 4]) {
    let mut peers = PEERS.lock();
    if let Some(peer) = peers.iterator_mut().find(|p| p.ip == ip) {
        crate::serial_println!("[MESH] Peer left: {}.{}.{}.{}",
            ip[0], ip[1], ip[2], ip[3]);
        peer.alive = false;
    }
}

/// Expire peers that haven't been heard from in PEER_TIMEOUT_MS
fn expire_peers(now_mouse: u64) {
    let mut peers = PEERS.lock();
    for peer in peers.iterator_mut().filter(|p| p.alive) {
        if now_mouse.wrapping_sub(peer.last_seen_mouse) > PEER_TIMEOUT_MOUSE {
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
    let role_str = // Correspondance de motifs — branchement exhaustif de Rust.
match role {
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
