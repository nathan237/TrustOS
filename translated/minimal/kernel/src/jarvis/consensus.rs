





















use alloc::vec::Vec;
use alloc::format;
use alloc::string::String;
use core::sync::atomic::{AtomicU64, AtomicBool, Ordering};
use spin::Mutex;






const BTJ_: u64 = 20_000;


const CEK_: u64 = 5000;


const BTI_: u64 = 30_000;






static EE_: AtomicU64 = AtomicU64::new(0);


static AEE_: AtomicU64 = AtomicU64::new(0);


static AEC_: AtomicU64 = AtomicU64::new(0);


static YM_: AtomicU64 = AtomicU64::new(0);


static FI_: AtomicBool = AtomicBool::new(false);


static UO_: Mutex<Option<[u8; 4]>> = Mutex::new(None);


static AEH_: AtomicU64 = AtomicU64::new(0);






pub fn init() {
    EE_.store(0, Ordering::SeqCst);
    AEE_.store(crate::time::lc(), Ordering::SeqCst);
    AEC_.store(0, Ordering::SeqCst);
    YM_.store(0, Ordering::SeqCst);
    FI_.store(false, Ordering::SeqCst);
    *UO_.lock() = None;
    super::mesh::gsf(super::mesh::NodeRole::Lb);
}


pub fn ylc() -> u64 {
    EE_.load(Ordering::SeqCst)
}


pub fn ogf() -> bool {
    FI_.load(Ordering::SeqCst)
}


pub fn oio() -> Option<[u8; 4]> {
    *UO_.lock()
}


pub fn poll() {
    if !super::mesh::rl() {
        return;
    }

    let iu = crate::time::lc();

    if FI_.load(Ordering::SeqCst) {
        
        let ucn = AEH_.load(Ordering::SeqCst);
        if iu.nj(ucn) >= CEK_ {
            phv();
            AEH_.store(iu, Ordering::SeqCst);
        }
    } else {
        
        let lht = AEE_.load(Ordering::SeqCst);
        let ucf = AEC_.load(Ordering::SeqCst);

        let xhc = iu.nj(lht) >= BTJ_;
        let roq = iu.nj(ucf) >= BTI_;
        let tmx = super::mesh::cti() > 0;

        if xhc && roq && tmx {
            wss();
        }
    }
}


pub fn pos() {
    if FI_.load(Ordering::SeqCst) {
        crate::serial_println!("[RAFT] Stepping down from leader");
        FI_.store(false, Ordering::SeqCst);
        super::mesh::gsf(super::mesh::NodeRole::Lb);
    }
}


pub fn status() -> String {
    let asc = EE_.load(Ordering::SeqCst);
    let bnj = FI_.load(Ordering::SeqCst);
    let oio = UO_.lock();

    let ufl = match *oio {
        Some(ip) => format!("{}.{}.{}.{}", ip[0], ip[1], ip[2], ip[3]),
        None => String::from("unknown"),
    };

    format!("term={} leader={} leader_ip={}", asc, bnj, ufl)
}






fn wss() {
    let jgu = EE_.fetch_add(1, Ordering::SeqCst) + 1;
    AEC_.store(crate::time::lc(), Ordering::SeqCst);

    
    YM_.store(jgu, Ordering::SeqCst);
    super::mesh::gsf(super::mesh::NodeRole::Mu);

    crate::serial_println!("[RAFT] Starting election for term {}", jgu);

    let yp = super::mesh::dhn();
    let pvi = yp.len() + 1; 
    let okw = pvi / 2 + 1;
    let mut mpz: usize = 1; 

    
    let mke = jgu.ft();

    for ko in &yp {
        match super::rpc::bto(
            ko.ip,
            ko.bsb,
            super::rpc::Command::Bao,
            &mke,
        ) {
            Ok((super::rpc::Status::Ok, ew)) => {
                if !ew.is_empty() && ew[0] == 1 {
                    mpz += 1;
                    crate::serial_println!("[RAFT] Vote granted by {}.{}.{}.{}",
                        ko.ip[0], ko.ip[1], ko.ip[2], ko.ip[3]);
                }
            }
            _ => {
                crate::serial_println!("[RAFT] No vote from {}.{}.{}.{}",
                    ko.ip[0], ko.ip[1], ko.ip[2], ko.ip[3]);
            }
        }
    }

    crate::serial_println!("[RAFT] Election result: {}/{} votes (need {})",
        mpz, pvi, okw);

    if mpz >= okw {
        
        qon(jgu);
    } else {
        
        super::mesh::gsf(super::mesh::NodeRole::Lb);
        crate::serial_println!("[RAFT] Election lost, reverting to Worker");
    }
}


fn qon(asc: u64) {
    FI_.store(true, Ordering::SeqCst);
    EE_.store(asc, Ordering::SeqCst);
    super::mesh::gsf(super::mesh::NodeRole::Ni);

    
    if let Some((ip, _, _)) = crate::network::aou() {
        *UO_.lock() = Some(*ip.as_bytes());
    }

    crate::serial_println!("[RAFT] ★ We are now LEADER for term {}", asc);

    
    phv();
    AEH_.store(crate::time::lc(), Ordering::SeqCst);
}






fn phv() {
    let asc = EE_.load(Ordering::SeqCst);
    let mke = asc.ft();

    let yp = super::mesh::dhn();
    for ko in &yp {
        let _ = super::rpc::bto(
            ko.ip,
            ko.bsb,
            super::rpc::Command::Auu,
            &mke,
        );
    }
}








pub fn tlq(ew: &[u8]) -> Vec<u8> {
    if ew.len() < 8 {
        return alloc::vec![0u8];
    }

    let feg = u64::oa([
        ew[0], ew[1], ew[2], ew[3],
        ew[4], ew[5], ew[6], ew[7],
    ]);

    let gop = EE_.load(Ordering::SeqCst);
    let mvf = YM_.load(Ordering::SeqCst);

    
    
    
    if feg >= gop && mvf < feg {
        YM_.store(feg, Ordering::SeqCst);
        EE_.store(feg, Ordering::SeqCst);

        
        if FI_.load(Ordering::SeqCst) && feg > gop {
            pos();
        }

        crate::serial_println!("[RAFT] Granting vote for term {}", feg);
        alloc::vec![1u8]
    } else {
        crate::serial_println!("[RAFT] Denying vote for term {} (our_term={}, voted_in={})",
            feg, gop, mvf);
        alloc::vec![0u8]
    }
}




pub fn tkg(ew: &[u8]) -> Vec<u8> {
    if ew.len() < 8 {
        return alloc::vec![0u8];
    }

    let lif = u64::oa([
        ew[0], ew[1], ew[2], ew[3],
        ew[4], ew[5], ew[6], ew[7],
    ]);

    let gop = EE_.load(Ordering::SeqCst);

    if lif >= gop {
        
        EE_.store(lif, Ordering::SeqCst);
        AEE_.store(crate::time::lc(), Ordering::SeqCst);

        
        if FI_.load(Ordering::SeqCst) && lif > gop {
            pos();
        }

        
        if super::mesh::htw() == super::mesh::NodeRole::Mu {
            super::mesh::gsf(super::mesh::NodeRole::Lb);
        }

        alloc::vec![1u8]
    } else {
        
        alloc::vec![0u8]
    }
}
