

use alloc::vec::Vec;
use alloc::string::String;
use alloc::collections::BTreeMap;
use core::sync::atomic::{AtomicU16, Ordering};
use spin::Mutex;

static AHR_: AtomicU16 = AtomicU16::new(1);


struct Xw {
    ip: [u8; 4],
    expire_tick: u64,
}


static ASO_: Mutex<BTreeMap<String, Xw>> = Mutex::new(BTreeMap::new());


const BUV_: u64 = 60_000;

fn pvb(buf: &mut Vec<u8>, name: &str) -> bool {
    for label in name.split('.') {
        let len = label.len();
        if len == 0 || len > 63 {
            return false;
        }
        buf.push(len as u8);
        buf.extend_from_slice(label.as_bytes());
    }
    buf.push(0);
    true
}

fn jgm(data: &[u8], mut idx: usize) -> Option<usize> {
    loop {
        if idx >= data.len() {
            return None;
        }
        let len = data[idx];
        if len & 0xC0 == 0xC0 {
            if idx + 1 >= data.len() {
                return None;
            }
            return Some(idx + 2);
        }
        if len == 0 {
            return Some(idx + 1);
        }
        idx += 1 + len as usize;
    }
}

pub fn yb(name: &str) -> Option<[u8; 4]> {
    
    {
        let cy = crate::logger::eg();
        let adk = ASO_.lock();
        if let Some(entry) = adk.get(name) {
            if cy < entry.expire_tick {
                return Some(entry.ip);
            }
        }
    }

    
    let dns_server = crate::network::mcy();
    let src_port = crate::netstack::udp::heu();
    let id = AHR_.fetch_add(1, Ordering::Relaxed);

    let mut query = Vec::with_capacity(64);
    query.extend_from_slice(&id.to_be_bytes());
    query.extend_from_slice(&0x0100u16.to_be_bytes()); 
    query.extend_from_slice(&1u16.to_be_bytes()); 
    query.extend_from_slice(&0u16.to_be_bytes()); 
    query.extend_from_slice(&0u16.to_be_bytes()); 
    query.extend_from_slice(&0u16.to_be_bytes()); 
    if !pvb(&mut query, name) {
        return None;
    }
    query.extend_from_slice(&1u16.to_be_bytes()); 
    query.extend_from_slice(&1u16.to_be_bytes()); 

    let _ = crate::netstack::udp::azq(dns_server, 53, src_port, &query);

    let start = crate::logger::eg();
    loop {
        crate::netstack::poll();
        if let Some(eo) = crate::netstack::udp::eyc(src_port) {
            if eo.len() < 12 {
                continue;
            }
            if u16::from_be_bytes([eo[0], eo[1]]) != id {
                continue;
            }
            let flags = u16::from_be_bytes([eo[2], eo[3]]);
            if (flags & 0x8000) == 0 {
                continue;
            }
            let oab = u16::from_be_bytes([eo[4], eo[5]]) as usize;
            let jvr = u16::from_be_bytes([eo[6], eo[7]]) as usize;

            let mut idx = 12;
            for _ in 0..oab {
                idx = jgm(&eo, idx)?;
                if idx + 4 > eo.len() {
                    return None;
                }
                idx += 4; 
            }

            for _ in 0..jvr {
                idx = jgm(&eo, idx)?;
                if idx + 10 > eo.len() {
                    return None;
                }
                let rtype = u16::from_be_bytes([eo[idx], eo[idx + 1]]);
                let oby = u16::from_be_bytes([eo[idx + 2], eo[idx + 3]]);
                let gpz = u16::from_be_bytes([eo[idx + 8], eo[idx + 9]]) as usize;
                idx += 10;
                if idx + gpz > eo.len() {
                    return None;
                }
                if rtype == 1 && oby == 1 && gpz == 4 {
                    let ip = [eo[idx], eo[idx + 1], eo[idx + 2], eo[idx + 3]];
                    
                    let lst = crate::logger::eg() + BUV_;
                    ASO_.lock().insert(
                        String::from(name),
                        Xw { ip, expire_tick: lst },
                    );
                    return Some(ip);
                }
                idx += gpz;
            }
        }

        if crate::logger::eg().saturating_sub(start) > 1500 {
            return None;
        }
        core::hint::spin_loop();
    }
}
