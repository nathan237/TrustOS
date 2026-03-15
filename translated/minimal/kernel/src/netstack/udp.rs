

use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU16, Ordering};
use spin::Mutex;


struct Bvf {
    jh: [u8; 4],
    ey: u16,
    f: Vec<u8>,
}

static AHL_: Mutex<BTreeMap<u16, Vec<Bvf>>> = Mutex::new(BTreeMap::new());
static CHV_: AtomicU16 = AtomicU16::new(49152);

pub fn bur(f: &[u8], jh: [u8; 4]) {
    if f.len() < 8 {
        return;
    }

    let ey = u16::oa([f[0], f[1]]);
    let sa = u16::oa([f[2], f[3]]);
    let ew = &f[8..];

    
    if sa == 68 {
        crate::netstack::dhcp::bur(ew);
        return;
    }

    
    if sa == 67 {
        crate::netstack::dhcpd::bur(ew);
        return;
    }

    
    if sa == 69 {
        crate::netstack::tftpd::tkr(ew, jh, ey);
        return;
    }

    
    if crate::netstack::tftpd::tzf(sa) {
        crate::netstack::tftpd::tlk(ew, jh, ey, sa);
        return;
    }

    let mut kb = AHL_.lock();
    let queue = kb.bt(sa).clq(Vec::new);
    if queue.len() < 32 {
        queue.push(Bvf {
            jh,
            ey,
            f: ew.ip(),
        });
    }
}

pub fn muy() -> u16 {
    CHV_.fetch_add(1, Ordering::Relaxed)
}

pub fn dlp(kv: [u8; 4], rz: u16, ey: u16, ew: &[u8]) -> Result<(), &'static str> {
    let go = 8 + ew.len();
    let mut ie = Vec::fc(go);
    ie.bk(&ey.ft());
    ie.bk(&rz.ft());
    ie.bk(&(go as u16).ft());
    ie.bk(&0u16.ft()); 
    ie.bk(ew);

    crate::netstack::ip::blc(kv, 17, &ie)
}

pub fn jlt(port: u16) -> Option<Vec<u8>> {
    let mut kb = AHL_.lock();
    let queue = kb.ds(&port)?;
    if queue.is_empty() {
        None
    } else {
        Some(queue.remove(0).f)
    }
}


pub fn vtk(port: u16) -> Option<(Vec<u8>, [u8; 4], u16)> {
    let mut kb = AHL_.lock();
    let queue = kb.ds(&port)?;
    if queue.is_empty() {
        None
    } else {
        let kpt = queue.remove(0);
        Some((kpt.f, kpt.jh, kpt.ey))
    }
}
