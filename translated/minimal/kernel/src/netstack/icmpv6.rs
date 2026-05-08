







use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use spin::Mutex;

use super::ipv6::{self, Ipv6Address, next_header};


pub mod icmpv6_type {
    
    pub const DMT_: u8 = 1;
    pub const ECC_: u8 = 2;
    pub const ELR_: u8 = 3;

    
    pub const BVT_: u8 = 128;
    pub const ATC_: u8 = 129;

    
    pub const CSR_: u8 = 133;
    pub const CSQ_: u8 = 134;
    pub const BDJ_: u8 = 135;
    pub const BDH_: u8 = 136;
}


pub mod ndp_option {
    pub const YN_: u8 = 1;
    pub const BJC_: u8 = 2;
    pub const COC_: u8 = 3;
    pub const Amr: u8 = 5;
}


#[derive(Clone)]
struct Abq {
    mac: [u8; 6],
    #[allow(dead_code)]
    timestamp: u64,
}


static BDI_: Mutex<BTreeMap<[u8; 16], Abq>> = Mutex::new(BTreeMap::new());


pub fn nar(addr: Ipv6Address) -> Option<[u8; 6]> {
    BDI_.lock().get(&addr.0).map(|e| e.mac)
}


fn fkn(addr: Ipv6Address, mac: [u8; 6]) {
    let entry = Abq {
        mac,
        timestamp: crate::logger::eg(),
    };
    BDI_.lock().insert(addr.0, entry);
}






pub fn alq(src: Ipv6Address, dst: Ipv6Address, data: &[u8]) {
    if data.len() < 4 { return; } 

    let msg_type = data[0];
    let pwu = data[1];
    

    match msg_type {
        icmpv6_type::BVT_ => {
            mhq(src, dst, data);
        }
        icmpv6_type::ATC_ => {
            mhp(src, data);
        }
        icmpv6_type::BDJ_ => {
            mic(src, data);
        }
        icmpv6_type::BDH_ => {
            mib(src, data);
        }
        icmpv6_type::CSQ_ => {
            mih(src, data);
        }
        _ => {
            crate::serial_println!("[ICMPv6] Unknown type {} from {}", msg_type, src);
        }
    }
}





fn mhq(src: Ipv6Address, _dst: Ipv6Address, data: &[u8]) {
    if data.len() < 8 { return; }

    crate::serial_println!("[ICMPv6] Echo Request from {}", src);

    let amc = ipv6::esz();

    
    let mut bvj = Vec::with_capacity(data.len());
    bvj.push(icmpv6_type::ATC_);
    bvj.push(0); 
    bvj.push(0); bvj.push(0); 
    bvj.extend_from_slice(&data[4..]); 

    
    let ig = ipv6::epz(&amc, &src, &bvj);
    bvj[2] = (ig >> 8) as u8;
    bvj[3] = (ig & 0xFF) as u8;

    let _ = ipv6::aha(src, next_header::Ka, &bvj);
}

fn mhp(src: Ipv6Address, data: &[u8]) {
    if data.len() < 8 { return; }
    let id = u16::from_be_bytes([data[4], data[5]]);
    let seq = u16::from_be_bytes([data[6], data[7]]);
    crate::serial_println!("[ICMPv6] Echo Reply from {}: id={} seq={}", src, id, seq);
}





fn mic(src: Ipv6Address, data: &[u8]) {
    
    if data.len() < 24 { return; }

    let target = Ipv6Address::new([
        data[8], data[9], data[10], data[11],
        data[12], data[13], data[14], data[15],
        data[16], data[17], data[18], data[19],
        data[20], data[21], data[22], data[23],
    ]);

    let amc = ipv6::esz();

    
    gmn(&data[24..], |cno, opt_data| {
        if cno == ndp_option::YN_ && opt_data.len() >= 6 {
            let mac = [opt_data[0], opt_data[1], opt_data[2], opt_data[3], opt_data[4], opt_data[5]];
            fkn(src, mac);
        }
    });

    
    if target != amc { return; }

    crate::serial_println!("[NDP] Neighbor Solicitation for {} from {}", target, src);

    
    let _ = onv(src, amc);
}

fn mib(src: Ipv6Address, data: &[u8]) {
    if data.len() < 24 { return; }

    let target = Ipv6Address::new([
        data[8], data[9], data[10], data[11],
        data[12], data[13], data[14], data[15],
        data[16], data[17], data[18], data[19],
        data[20], data[21], data[22], data[23],
    ]);

    crate::serial_println!("[NDP] Neighbor Advertisement: {} is at {}", target, src);

    
    gmn(&data[24..], |cno, opt_data| {
        if cno == ndp_option::BJC_ && opt_data.len() >= 6 {
            let mac = [opt_data[0], opt_data[1], opt_data[2], opt_data[3], opt_data[4], opt_data[5]];
            fkn(target, mac);
        }
    });
}





fn mih(src: Ipv6Address, data: &[u8]) {
    
    
    if data.len() < 16 { return; }

    let epj = data[4];
    let oii = u16::from_be_bytes([data[6], data[7]]);

    crate::serial_println!("[NDP] Router Advertisement from {}: hop_limit={} lifetime={}s", 
        src, epj, oii);

    
    gmn(&data[16..], |cno, opt_data| {
        match cno {
            ndp_option::COC_ if opt_data.len() >= 30 => {
                let nws = opt_data[0];
                let flags = opt_data[1];
                let nm = &opt_data[14..30];
                crate::serial_println!("[NDP]   Prefix: {:02x}{:02x}:{:02x}{:02x}::/{} flags={:#x}",
                    nm[0], nm[1], nm[2], nm[3], nws, flags);
            }
            ndp_option::YN_ if opt_data.len() >= 6 => {
                let mac = [opt_data[0], opt_data[1], opt_data[2], opt_data[3], opt_data[4], opt_data[5]];
                fkn(src, mac);
                crate::serial_println!("[NDP]   Router MAC: {:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
                    mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]);
            }
            ndp_option::Amr if opt_data.len() >= 6 => {
                let ngx = u32::from_be_bytes([opt_data[2], opt_data[3], opt_data[4], opt_data[5]]);
                crate::serial_println!("[NDP]   MTU: {}", ngx);
            }
            _ => {}
        }
    });
}






pub fn ooa(amc: Ipv6Address) -> Result<(), &'static str> {
    let mac = crate::drivers::net::aqt()
        .or_else(crate::network::aqu)
        .unwrap_or([0; 6]);

    let dst = Ipv6Address::BMM_;

    
    let mut bk = Vec::with_capacity(16);
    bk.push(icmpv6_type::CSR_);
    bk.push(0); 
    bk.push(0); bk.push(0); 
    bk.extend_from_slice(&[0u8; 4]); 

    
    bk.push(ndp_option::YN_);
    bk.push(1); 
    bk.extend_from_slice(&mac);

    
    let ig = ipv6::epz(&amc, &dst, &bk);
    bk[2] = (ig >> 8) as u8;
    bk[3] = (ig & 0xFF) as u8;

    crate::serial_println!("[NDP] Sending Router Solicitation");
    ipv6::fab(amc, dst, next_header::Ka, 255, &bk)
}


fn onv(dst: Ipv6Address, amc: Ipv6Address) -> Result<(), &'static str> {
    let mac = crate::drivers::net::aqt()
        .or_else(crate::network::aqu)
        .unwrap_or([0; 6]);

    
    let mut bk = Vec::with_capacity(32);
    bk.push(icmpv6_type::BDH_);
    bk.push(0); 
    bk.push(0); bk.push(0); 

    
    let flags: u32 = (1 << 30) | (1 << 29);
    bk.extend_from_slice(&flags.to_be_bytes());

    
    bk.extend_from_slice(&amc.0);

    
    bk.push(ndp_option::BJC_);
    bk.push(1); 
    bk.extend_from_slice(&mac);

    
    let ig = ipv6::epz(&amc, &dst, &bk);
    bk[2] = (ig >> 8) as u8;
    bk[3] = (ig & 0xFF) as u8;

    crate::serial_println!("[NDP] Sending Neighbor Advertisement to {}", dst);
    ipv6::fab(amc, dst, next_header::Ka, 255, &bk)
}


pub fn qvh(target: Ipv6Address) -> Result<(), &'static str> {
    let amc = ipv6::esz();
    let mac = crate::drivers::net::aqt()
        .or_else(crate::network::aqu)
        .unwrap_or([0; 6]);

    let dst = target.solicited_node_multicast();

    
    let mut bk = Vec::with_capacity(32);
    bk.push(icmpv6_type::BDJ_);
    bk.push(0); 
    bk.push(0); bk.push(0); 
    bk.extend_from_slice(&[0u8; 4]); 
    bk.extend_from_slice(&target.0);

    
    bk.push(ndp_option::YN_);
    bk.push(1);
    bk.extend_from_slice(&mac);

    
    let ig = ipv6::epz(&amc, &dst, &bk);
    bk[2] = (ig >> 8) as u8;
    bk[3] = (ig & 0xFF) as u8;

    crate::serial_println!("[NDP] Sending Neighbor Solicitation for {}", target);
    ipv6::fab(amc, dst, next_header::Ka, 255, &bk)
}





fn gmn<F: FnMut(u8, &[u8])>(data: &[u8], mut handler: F) {
    let mut i = 0;
    while i + 2 <= data.len() {
        let cno = data[i];
        let evt = data[i + 1] as usize * 8; 
        if evt == 0 || i + evt > data.len() { break; }
        
        handler(cno, &data[i + 2..i + evt]);
        i += evt;
    }
}
