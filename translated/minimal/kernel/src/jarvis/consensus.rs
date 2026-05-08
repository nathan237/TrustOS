





















use alloc::vec::Vec;
use alloc::format;
use alloc::string::String;
use core::sync::atomic::{AtomicU64, AtomicBool, Ordering};
use spin::Mutex;






const BWF_: u64 = 20_000;


const CHT_: u64 = 5000;


const BWE_: u64 = 30_000;






static ER_: AtomicU64 = AtomicU64::new(0);


static AFY_: AtomicU64 = AtomicU64::new(0);


static AFW_: AtomicU64 = AtomicU64::new(0);


static ZP_: AtomicU64 = AtomicU64::new(0);


static FX_: AtomicBool = AtomicBool::new(false);


static VX_: Mutex<Option<[u8; 4]>> = Mutex::new(None);


static AGB_: AtomicU64 = AtomicU64::new(0);






pub fn init() {
    ER_.store(0, Ordering::SeqCst);
    AFY_.store(crate::time::uptime_ms(), Ordering::SeqCst);
    AFW_.store(0, Ordering::SeqCst);
    ZP_.store(0, Ordering::SeqCst);
    FX_.store(false, Ordering::SeqCst);
    *VX_.lock() = None;
    super::mesh::dei(super::mesh::NodeRole::Worker);
}


pub fn qcd() -> u64 {
    ER_.load(Ordering::SeqCst)
}


pub fn iia() -> bool {
    FX_.load(Ordering::SeqCst)
}


pub fn ijs() -> Option<[u8; 4]> {
    *VX_.lock()
}


pub fn poll() {
    if !super::mesh::is_active() {
        return;
    }

    let cy = crate::time::uptime_ms();

    if FX_.load(Ordering::SeqCst) {
        
        let mwr = AGB_.load(Ordering::SeqCst);
        if cy.wrapping_sub(mwr) >= CHT_ {
            jem();
            AGB_.store(cy, Ordering::SeqCst);
        }
    } else {
        
        let gey = AFY_.load(Ordering::SeqCst);
        let mwl = AFW_.load(Ordering::SeqCst);

        let pjn = cy.wrapping_sub(gey) >= BWF_;
        let kxp = cy.wrapping_sub(mwl) >= BWE_;
        let mjv = super::mesh::ayz() > 0;

        if pjn && kxp && mjv {
            owf();
        }
    }
}


pub fn jiu() {
    if FX_.load(Ordering::SeqCst) {
        crate::serial_println!("[RAFT] Stepping down from leader");
        FX_.store(false, Ordering::SeqCst);
        super::mesh::dei(super::mesh::NodeRole::Worker);
    }
}


pub fn status() -> String {
    let wp = ER_.load(Ordering::SeqCst);
    let aid = FX_.load(Ordering::SeqCst);
    let ijs = VX_.lock();

    let myy = match *ijs {
        Some(ip) => format!("{}.{}.{}.{}", ip[0], ip[1], ip[2], ip[3]),
        None => String::from("unknown"),
    };

    format!("term={} leader={} leader_ip={}", wp, aid, myy)
}






fn owf() {
    let eux = ER_.fetch_add(1, Ordering::SeqCst) + 1;
    AFW_.store(crate::time::uptime_ms(), Ordering::SeqCst);

    
    ZP_.store(eux, Ordering::SeqCst);
    super::mesh::dei(super::mesh::NodeRole::Candidate);

    crate::serial_println!("[RAFT] Starting election for term {}", eux);

    let lj = super::mesh::bgo();
    let joc = lj.len() + 1; 
    let ilr = joc / 2 + 1;
    let mut hby: usize = 1; 

    
    let gyg = eux.to_be_bytes();

    for peer in &lj {
        match super::rpc::alb(
            peer.ip,
            peer.rpc_port,
            super::rpc::Command::VoteRequest,
            &gyg,
        ) {
            Ok((super::rpc::Status::Ok, payload)) => {
                if !payload.is_empty() && payload[0] == 1 {
                    hby += 1;
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
        hby, joc, ilr);

    if hby >= ilr {
        
        kba(eux);
    } else {
        
        super::mesh::dei(super::mesh::NodeRole::Worker);
        crate::serial_println!("[RAFT] Election lost, reverting to Worker");
    }
}


fn kba(wp: u64) {
    FX_.store(true, Ordering::SeqCst);
    ER_.store(wp, Ordering::SeqCst);
    super::mesh::dei(super::mesh::NodeRole::Leader);

    
    if let Some((ip, _, _)) = crate::network::rd() {
        *VX_.lock() = Some(*ip.as_bytes());
    }

    crate::serial_println!("[RAFT] ★ We are now LEADER for term {}", wp);

    
    jem();
    AGB_.store(crate::time::uptime_ms(), Ordering::SeqCst);
}






fn jem() {
    let wp = ER_.load(Ordering::SeqCst);
    let gyg = wp.to_be_bytes();

    let lj = super::mesh::bgo();
    for peer in &lj {
        let _ = super::rpc::alb(
            peer.ip,
            peer.rpc_port,
            super::rpc::Command::LeaderHeartbeat,
            &gyg,
        );
    }
}








pub fn miu(payload: &[u8]) -> Vec<u8> {
    if payload.len() < 8 {
        return alloc::vec![0u8];
    }

    let cgq = u64::from_be_bytes([
        payload[0], payload[1], payload[2], payload[3],
        payload[4], payload[5], payload[6], payload[7],
    ]);

    let dcb = ER_.load(Ordering::SeqCst);
    let hfa = ZP_.load(Ordering::SeqCst);

    
    
    
    if cgq >= dcb && hfa < cgq {
        ZP_.store(cgq, Ordering::SeqCst);
        ER_.store(cgq, Ordering::SeqCst);

        
        if FX_.load(Ordering::SeqCst) && cgq > dcb {
            jiu();
        }

        crate::serial_println!("[RAFT] Granting vote for term {}", cgq);
        alloc::vec![1u8]
    } else {
        crate::serial_println!("[RAFT] Denying vote for term {} (our_term={}, voted_in={})",
            cgq, dcb, hfa);
        alloc::vec![0u8]
    }
}




pub fn mhz(payload: &[u8]) -> Vec<u8> {
    if payload.len() < 8 {
        return alloc::vec![0u8];
    }

    let gff = u64::from_be_bytes([
        payload[0], payload[1], payload[2], payload[3],
        payload[4], payload[5], payload[6], payload[7],
    ]);

    let dcb = ER_.load(Ordering::SeqCst);

    if gff >= dcb {
        
        ER_.store(gff, Ordering::SeqCst);
        AFY_.store(crate::time::uptime_ms(), Ordering::SeqCst);

        
        if FX_.load(Ordering::SeqCst) && gff > dcb {
            jiu();
        }

        
        if super::mesh::dwa() == super::mesh::NodeRole::Candidate {
            super::mesh::dei(super::mesh::NodeRole::Worker);
        }

        alloc::vec![1u8]
    } else {
        
        alloc::vec![0u8]
    }
}
