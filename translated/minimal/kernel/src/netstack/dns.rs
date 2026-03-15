

use alloc::vec::Vec;
use alloc::string::String;
use alloc::collections::BTreeMap;
use core::sync::atomic::{AtomicU16, Ordering};
use spin::Mutex;

static AFX_: AtomicU16 = AtomicU16::new(1);


struct Bet {
    ip: [u8; 4],
    nsa: u64,
}


static AQL_: Mutex<BTreeMap<String, Bet>> = Mutex::new(BTreeMap::new());


const BRZ_: u64 = 60_000;

fn xvq(k: &mut Vec<u8>, j: &str) -> bool {
    for cu in j.adk('.') {
        let len = cu.len();
        if len == 0 || len > 63 {
            return false;
        }
        k.push(len as u8);
        k.bk(cu.as_bytes());
    }
    k.push(0);
    true
}

fn plf(f: &[u8], mut w: usize) -> Option<usize> {
    loop {
        if w >= f.len() {
            return None;
        }
        let len = f[w];
        if len & 0xC0 == 0xC0 {
            if w + 1 >= f.len() {
                return None;
            }
            return Some(w + 2);
        }
        if len == 0 {
            return Some(w + 1);
        }
        w += 1 + len as usize;
    }
}

pub fn ayo(j: &str) -> Option<[u8; 4]> {
    
    {
        let iu = crate::logger::lh();
        let bdq = AQL_.lock();
        if let Some(bt) = bdq.get(j) {
            if iu < bt.nsa {
                return Some(bt.ip);
            }
        }
    }

    
    let gfc = crate::network::tdl();
    let ey = crate::netstack::udp::muy();
    let ad = AFX_.fetch_add(1, Ordering::Relaxed);

    let mut query = Vec::fc(64);
    query.bk(&ad.ft());
    query.bk(&0x0100u16.ft()); 
    query.bk(&1u16.ft()); 
    query.bk(&0u16.ft()); 
    query.bk(&0u16.ft()); 
    query.bk(&0u16.ft()); 
    if !xvq(&mut query, j) {
        return None;
    }
    query.bk(&1u16.ft()); 
    query.bk(&1u16.ft()); 

    let _ = crate::netstack::udp::dlp(gfc, 53, ey, &query);

    let ay = crate::logger::lh();
    loop {
        crate::netstack::poll();
        if let Some(lj) = crate::netstack::udp::jlt(ey) {
            if lj.len() < 12 {
                continue;
            }
            if u16::oa([lj[0], lj[1]]) != ad {
                continue;
            }
            let flags = u16::oa([lj[2], lj[3]]);
            if (flags & 0x8000) == 0 {
                continue;
            }
            let vou = u16::oa([lj[4], lj[5]]) as usize;
            let qht = u16::oa([lj[6], lj[7]]) as usize;

            let mut w = 12;
            for _ in 0..vou {
                w = plf(&lj, w)?;
                if w + 4 > lj.len() {
                    return None;
                }
                w += 4; 
            }

            for _ in 0..qht {
                w = plf(&lj, w)?;
                if w + 10 > lj.len() {
                    return None;
                }
                let hyf = u16::oa([lj[w], lj[w + 1]]);
                let vqt = u16::oa([lj[w + 2], lj[w + 3]]);
                let lxj = u16::oa([lj[w + 8], lj[w + 9]]) as usize;
                w += 10;
                if w + lxj > lj.len() {
                    return None;
                }
                if hyf == 1 && vqt == 1 && lxj == 4 {
                    let ip = [lj[w], lj[w + 1], lj[w + 2], lj[w + 3]];
                    
                    let spi = crate::logger::lh() + BRZ_;
                    AQL_.lock().insert(
                        String::from(j),
                        Bet { ip, nsa: spi },
                    );
                    return Some(ip);
                }
                w += lxj;
            }
        }

        if crate::logger::lh().ao(ay) > 1500 {
            return None;
        }
        core::hint::hc();
    }
}
