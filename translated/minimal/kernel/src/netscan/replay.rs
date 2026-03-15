











use alloc::vec::Vec;
use alloc::string::String;
use alloc::format;
use core::sync::atomic::{AtomicU64, Ordering};


static NP_: AtomicU64 = AtomicU64::new(0);


#[derive(Debug)]
pub struct Bqy {
    pub egc: u64,
    pub vaq: u64,
    pub co: String,
}




pub fn lep(frame: &[u8]) -> Result<(), &'static str> {
    if frame.len() < 14 {
        return Err("frame too short (min 14 bytes for Ethernet header)");
    }
    crate::network::blc(frame)?;
    NP_.fetch_add(1, Ordering::Relaxed);
    Ok(())
}


pub fn tun(kv: [u8; 4], protocol: u8, ew: &[u8]) -> Result<(), &'static str> {
    crate::netstack::ip::blc(kv, protocol, ew)?;
    NP_.fetch_add(1, Ordering::Relaxed);
    Ok(())
}




pub fn rqg(jh: [u8; 4], pz: [u8; 4], ey: u16, sa: u16, ls: u32) -> Vec<u8> {
    let mut pk = Vec::fc(20);
    pk.bk(&ey.ft());
    pk.bk(&sa.ft());
    pk.bk(&ls.ft());
    pk.bk(&0u32.ft()); 
    pk.push(0x50); 
    pk.push(0x02); 
    pk.bk(&65535u16.ft()); 
    pk.bk(&0u16.ft()); 
    pk.bk(&0u16.ft()); 
    
    let td = crate::netstack::tcp::psc(jh, pz, &pk);
    pk[16] = (td >> 8) as u8;
    pk[17] = (td & 0xFF) as u8;
    pk
}


pub fn ykg(ad: u16, whv: u16, ew: &[u8]) -> Vec<u8> {
    let mut mt = Vec::fc(8 + ew.len());
    mt.push(8); 
    mt.push(0); 
    mt.bk(&0u16.ft()); 
    mt.bk(&ad.ft());
    mt.bk(&whv.ft());
    mt.bk(ew);
    
    let td = lcz(&mt);
    mt[2] = (td >> 8) as u8;
    mt[3] = (td & 0xFF) as u8;
    mt
}


pub fn ykh(ey: u16, sa: u16, ew: &[u8]) -> Vec<u8> {
    let len = 8 + ew.len() as u16;
    let mut mt = Vec::fc(len as usize);
    mt.bk(&ey.ft());
    mt.bk(&sa.ft());
    mt.bk(&len.ft());
    mt.bk(&0u16.ft()); 
    mt.bk(ew);
    mt
}




pub fn zjs() -> Bqy {
    let bjm = super::sniffer::kyk();
    let mut gsb = 0u64;
    let mut ace = 0u64;
    for mt in &bjm {
        if mt.bal.len() >= 14 {
            match lep(&mt.bal) {
                Ok(()) => gsb += 1,
                Err(_) => ace += 1,
            }
        } else {
            ace += 1;
        }
    }
    Bqy {
        egc: gsb,
        vaq: ace,
        co: format!("Replayed {}/{} packets", gsb, gsb + ace),
    }
}


pub fn zjt(index: usize) -> Result<(), &'static str> {
    let bjm = super::sniffer::kyk();
    let mt = bjm.get(index).ok_or("packet index out of range")?;
    if mt.bal.len() < 14 {
        return Err("captured packet too short");
    }
    lep(&mt.bal)
}





pub fn zqm(pz: [u8; 4], sa: u16, az: u32) -> u32 {
    let jh = crate::network::aou()
        .map(|(ip, _, _)| *ip.as_bytes())
        .unwrap_or([10, 0, 2, 15]);
    let mut bq = 0u32;
    for a in 0..az {
        let ey = 1024 + (a as u16 % 64000);
        let ls = crate::rng::wgg();
        let jry = rqg(jh, pz, ey, sa, ls);
        
        if tun(pz, 6, &jry).is_ok() {
            bq += 1;
        }
    }
    bq
}


pub fn yfb(blk: [u8; 4]) -> Result<(), &'static str> {
    crate::netstack::arp::eii(blk)?;
    NP_.fetch_add(1, Ordering::Relaxed);
    Ok(())
}




pub fn xkh() -> u64 {
    NP_.load(Ordering::Relaxed)
}


pub fn pcq() {
    NP_.store(0, Ordering::Relaxed);
}



fn lcz(f: &[u8]) -> u16 {
    let mut sum: u32 = 0;
    let mut a = 0;
    while a + 1 < f.len() {
        sum += ((f[a] as u32) << 8) | (f[a + 1] as u32);
        a += 2;
    }
    if a < f.len() {
        sum += (f[a] as u32) << 8;
    }
    while (sum >> 16) != 0 {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }
    !(sum as u16)
}
