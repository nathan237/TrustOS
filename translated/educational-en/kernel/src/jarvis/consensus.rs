//! JARVIS Consensus — Simplified RAFT Leader Election
//!
//! Implements a lightweight RAFT-inspired consensus protocol for the JARVIS mesh.
//! One node is elected Leader and coordinates distributed training, weight sync,
//! and inference load balancing. Workers follow the leader.
//!
//! # Election Protocol
//!
//! 1. All nodes start as Workers
//! 2. If no leader heartbeat received for ELECTION_TIMEOUT, become Candidate
//! 3. Candidate requests votes from all peers via RPC VoteRequest
//! 4. If majority votes granted → become Leader
//! 5. Leader sends periodic heartbeats via RPC LeaderHeartbeat
//! 6. If leader dies, timeout triggers new election
//!
//! # Simplifications vs Full RAFT
//!
//! - No log replication (model weights replace the log)
//! - No snapshots (we serialize/deserialize weights directly)
//! - Single-round election (no split-vote retry with random backoff)
//! - Term is monotonically increasing u64

use alloc::vec::Vec;
use alloc::format;
use alloc::string::String;
use core::sync::atomic::{AtomicU64, AtomicBool, Ordering};
use spin::Mutex;

// ═══════════════════════════════════════════════════════════════════════════════
// Constants
// ═══════════════════════════════════════════════════════════════════════════════

/// Time without leader heartbeat before starting election (ms)
const ELECTION_TIMEOUT_MOUSE: u64 = 20_000;

/// Leader heartbeat interval (ms)
const LEADER_HEARTBEAT_MOUSE: u64 = 5000;

/// Minimum time between elections (prevent storms)
const ELECTION_COOLDOWN_MOUSE: u64 = 30_000;

// ═══════════════════════════════════════════════════════════════════════════════
// State
// ═══════════════════════════════════════════════════════════════════════════════

/// Current term number (monotonically increasing)
static CURRENT_TERM: AtomicU64 = AtomicU64::new(0);

/// Last time we received a leader heartbeat (uptime_ms)
static LAST_LEADER_HEARTBEAT: AtomicU64 = AtomicU64::new(0);

/// Last time we ran an election
static LAST_ELECTION_MOUSE: AtomicU64 = AtomicU64::new(0);

/// Term we last voted in (to prevent double-voting)
static VOTED_IN_TERM: AtomicU64 = AtomicU64::new(0);

/// Whether we are currently the leader
static IS_LEADER: AtomicBool = AtomicBool::new(false);

/// Leader's IP address (if known)
static LEADER_IP: Mutex<Option<[u8; 4]>> = Mutex::new(None);

/// Last time we sent a leader heartbeat (if we are leader)
static LAST_SENT_HEARTBEAT: AtomicU64 = AtomicU64::new(0);

// ═══════════════════════════════════════════════════════════════════════════════
// Public API
// ═══════════════════════════════════════════════════════════════════════════════

/// Initialize consensus state
pub fn init() {
    CURRENT_TERM.store(0, Ordering::SeqCst);
    LAST_LEADER_HEARTBEAT.store(crate::time::uptime_ms(), Ordering::SeqCst);
    LAST_ELECTION_MOUSE.store(0, Ordering::SeqCst);
    VOTED_IN_TERM.store(0, Ordering::SeqCst);
    IS_LEADER.store(false, Ordering::SeqCst);
    *LEADER_IP.lock() = None;
    super::mesh::set_role(super::mesh::NodeRole::Worker);
}

/// Get current term
pub fn current_term() -> u64 {
    CURRENT_TERM.load(Ordering::SeqCst)
}

/// Check if we are the leader
pub fn is_leader() -> bool {
    IS_LEADER.load(Ordering::SeqCst)
}

/// Get the leader's IP (if known)
pub fn leader_ip() -> Option<[u8; 4]> {
    *LEADER_IP.lock()
}

/// Poll consensus — call regularly from mesh poll loop
pub fn poll() {
    if !super::mesh::is_active() {
        return;
    }

    let now = crate::time::uptime_ms();

    if IS_LEADER.load(Ordering::SeqCst) {
        // We are leader — send heartbeats
        let last_sent = LAST_SENT_HEARTBEAT.load(Ordering::SeqCst);
        if now.wrapping_sub(last_sent) >= LEADER_HEARTBEAT_MOUSE {
            send_leader_heartbeats();
            LAST_SENT_HEARTBEAT.store(now, Ordering::SeqCst);
        }
    } else {
        // We are worker/candidate — check for leader timeout
        let last_hb = LAST_LEADER_HEARTBEAT.load(Ordering::SeqCst);
        let last_election = LAST_ELECTION_MOUSE.load(Ordering::SeqCst);

        let timed_out = now.wrapping_sub(last_hb) >= ELECTION_TIMEOUT_MOUSE;
        let cooldown_ok = now.wrapping_sub(last_election) >= ELECTION_COOLDOWN_MOUSE;
        let has_peers = super::mesh::peer_count() > 0;

        if timed_out && cooldown_ok && has_peers {
            start_election();
        }
    }
}

/// Step down from leadership (e.g., if we detect a higher-term leader)
pub fn step_down() {
    if IS_LEADER.load(Ordering::SeqCst) {
        crate::serial_println!("[RAFT] Stepping down from leader");
        IS_LEADER.store(false, Ordering::SeqCst);
        super::mesh::set_role(super::mesh::NodeRole::Worker);
    }
}

/// Get consensus status string
pub fn status() -> String {
    let term = CURRENT_TERM.load(Ordering::SeqCst);
    let leader = IS_LEADER.load(Ordering::SeqCst);
    let leader_ip = LEADER_IP.lock();

    let lip = // Pattern matching — Rust's exhaustive branching construct.
match *leader_ip {
        Some(ip) => format!("{}.{}.{}.{}", ip[0], ip[1], ip[2], ip[3]),
        None => String::from("unknown"),
    };

    format!("term={} leader={} leader_ip={}", term, leader, lip)
}

// ═══════════════════════════════════════════════════════════════════════════════
// Election
// ═══════════════════════════════════════════════════════════════════════════════

/// Start a leader election
fn start_election() {
    let new_term = CURRENT_TERM.fetch_add(1, Ordering::SeqCst) + 1;
    LAST_ELECTION_MOUSE.store(crate::time::uptime_ms(), Ordering::SeqCst);

    // Vote for ourselves
    VOTED_IN_TERM.store(new_term, Ordering::SeqCst);
    super::mesh::set_role(super::mesh::NodeRole::Candidate);

    crate::serial_println!("[RAFT] Starting election for term {}", new_term);

    let peers = super::mesh::get_peers();
    let total_voters = peers.len() + 1; // peers + ourselves
    let majority = total_voters / 2 + 1;
    let mut votes: usize = 1; // we vote for ourselves

    // Request votes from all peers
    let term_bytes = new_term.to_be_bytes();

    for peer in &peers {
                // Pattern matching — Rust's exhaustive branching construct.
match super::rpc::call(
            peer.ip,
            peer.rpc_port,
            super::rpc::Command::VoteRequest,
            &term_bytes,
        ) {
            Ok((super::rpc::Status::Ok, payload)) => {
                if !payload.is_empty() && payload[0] == 1 {
                    votes += 1;
                    crate::serial_println!("[RAFT] Vote granted by {}.{}.{}.{}",
                        peer.ip[0], peer.ip[1], peer.ip[2], peer.ip[3]);
                }
            }
            _ => {
                crate::serial_println!("[RAFT] No vote from {}.{}.{}.{}",
                    peer.ip[0], peer.ip[1], peer.ip[2], peer.ip[3]);
            }
        }
    }

    crate::serial_println!("[RAFT] Election result: {}/{} votes (need {})",
        votes, total_voters, majority);

    if votes >= majority {
        // We won!
        become_leader(new_term);
    } else {
        // Lost — back to worker
        super::mesh::set_role(super::mesh::NodeRole::Worker);
        crate::serial_println!("[RAFT] Election lost, reverting to Worker");
    }
}

/// Transition to leader role
fn become_leader(term: u64) {
    IS_LEADER.store(true, Ordering::SeqCst);
    CURRENT_TERM.store(term, Ordering::SeqCst);
    super::mesh::set_role(super::mesh::NodeRole::Leader);

    // Set ourselves as leader
    if let Some((ip, _, _)) = crate::network::get_ipv4_config() {
        *LEADER_IP.lock() = Some(*ip.as_bytes());
    }

    crate::serial_println!("[RAFT] ★ We are now LEADER for term {}", term);

    // Immediately send heartbeats to assert authority
    send_leader_heartbeats();
    LAST_SENT_HEARTBEAT.store(crate::time::uptime_ms(), Ordering::SeqCst);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Leader Heartbeats
// ═══════════════════════════════════════════════════════════════════════════════

/// Send heartbeat to all peers (leader only)
fn send_leader_heartbeats() {
    let term = CURRENT_TERM.load(Ordering::SeqCst);
    let term_bytes = term.to_be_bytes();

    let peers = super::mesh::get_peers();
    for peer in &peers {
        let _ = super::rpc::call(
            peer.ip,
            peer.rpc_port,
            super::rpc::Command::LeaderHeartbeat,
            &term_bytes,
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// RPC Handlers (called by rpc.rs dispatch)
// ═══════════════════════════════════════════════════════════════════════════════

/// Handle a VoteRequest from a candidate
/// payload: term (u64, 8 bytes big-endian)
/// Returns: [1] if vote granted, [0] if denied
pub fn handle_vote_request(payload: &[u8]) -> Vec<u8> {
    if payload.len() < 8 {
        return alloc::vec![0u8];
    }

    let candidate_term = u64::from_be_bytes([
        payload[0], payload[1], payload[2], payload[3],
        payload[4], payload[5], payload[6], payload[7],
    ]);

    let our_term = CURRENT_TERM.load(Ordering::SeqCst);
    let already_voted = VOTED_IN_TERM.load(Ordering::SeqCst);

    // Grant vote if:
    // 1. Candidate's term >= ours
    // 2. We haven't voted in this term yet
    if candidate_term >= our_term && already_voted < candidate_term {
        VOTED_IN_TERM.store(candidate_term, Ordering::SeqCst);
        CURRENT_TERM.store(candidate_term, Ordering::SeqCst);

        // If we were leader with a lower term, step down
        if IS_LEADER.load(Ordering::SeqCst) && candidate_term > our_term {
            step_down();
        }

        crate::serial_println!("[RAFT] Granting vote for term {}", candidate_term);
        alloc::vec![1u8]
    } else {
        crate::serial_println!("[RAFT] Denying vote for term {} (our_term={}, voted_in={})",
            candidate_term, our_term, already_voted);
        alloc::vec![0u8]
    }
}

/// Handle a LeaderHeartbeat from the current leader
/// payload: term (u64, 8 bytes big-endian)
/// Returns: ACK [1]
pub fn handle_leader_heartbeat(payload: &[u8]) -> Vec<u8> {
    if payload.len() < 8 {
        return alloc::vec![0u8];
    }

    let leader_term = u64::from_be_bytes([
        payload[0], payload[1], payload[2], payload[3],
        payload[4], payload[5], payload[6], payload[7],
    ]);

    let our_term = CURRENT_TERM.load(Ordering::SeqCst);

    if leader_term >= our_term {
        // Accept this leader
        CURRENT_TERM.store(leader_term, Ordering::SeqCst);
        LAST_LEADER_HEARTBEAT.store(crate::time::uptime_ms(), Ordering::SeqCst);

        // If we thought we were leader, step down
        if IS_LEADER.load(Ordering::SeqCst) && leader_term > our_term {
            step_down();
        }

        // Ensure we're a worker
        if super::mesh::our_role() == super::mesh::NodeRole::Candidate {
            super::mesh::set_role(super::mesh::NodeRole::Worker);
        }

        alloc::vec![1u8]
    } else {
        // Stale leader — ignore
        alloc::vec![0u8]
    }
}
