







use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use spin::Mutex;

use super::ipv6::{self, Ipv6Address, next_header};


pub mod icmpv6_type {
    
    pub const DJE_: u8 = 1;
    pub const DYL_: u8 = 2;
    pub const EIA_: u8 = 3;

    
    pub const BSX_: u8 = 128;
    pub const AQZ_: u8 = 129;

    
    pub const CPC_: u8 = 133;
    pub const CPB_: u8 = 134;
    pub const BBG_: u8 = 135;
    pub const BBE_: u8 = 136;
}


pub mod ndp_option {
    pub const XG_: u8 = 1;
    pub const BGY_: u8 = 2;
    pub const CKT_: u8 = 3;
    pub const Che: u8 = 5;
}


#[derive(Clone)]
struct Bne {
    ed: [u8; 6],
    #[allow(bgr)]
    aea: u64,
}


static BBF_: Mutex<BTreeMap<[u8; 16], Bne>> = Mutex::new(BTreeMap::new());


pub fn uih(ag: Ipv6Address) -> Option<[u8; 6]> {
    BBF_.lock().get(&ag.0).map(|aa| aa.ed)
}


fn kfy(ag: Ipv6Address, ed: [u8; 6]) {
    let bt = Bne {
        ed,
        aea: crate::logger::lh(),
    };
    BBF_.lock().insert(ag.0, bt);
}






pub fn bur(cy: Ipv6Address, cs: Ipv6Address, f: &[u8]) {
    if f.len() < 4 { return; } 

    let msg_type = f[0];
    let xyi = f[1];
    

    match msg_type {
        icmpv6_type::BSX_ => {
            tjk(cy, cs, f);
        }
        icmpv6_type::AQZ_ => {
            tjj(cy, f);
        }
        icmpv6_type::BBG_ => {
            tkl(cy, f);
        }
        icmpv6_type::BBE_ => {
            tkk(cy, f);
        }
        icmpv6_type::CPB_ => {
            tkt(cy, f);
        }
        _ => {
            crate::serial_println!("[ICMPv6] Unknown type {} from {}", msg_type, cy);
        }
    }
}





fn tjk(cy: Ipv6Address, xyw: Ipv6Address, f: &[u8]) {
    if f.len() < 8 { return; }

    crate::serial_println!("[ICMPv6] Echo Request from {}", cy);

    let bvt = ipv6::jdo();

    
    let mut ehp = Vec::fc(f.len());
    ehp.push(icmpv6_type::AQZ_);
    ehp.push(0); 
    ehp.push(0); ehp.push(0); 
    ehp.bk(&f[4..]); 

    
    let td = ipv6::izf(&bvt, &cy, &ehp);
    ehp[2] = (td >> 8) as u8;
    ehp[3] = (td & 0xFF) as u8;

    let _ = ipv6::blc(cy, next_header::Xb, &ehp);
}

fn tjj(cy: Ipv6Address, f: &[u8]) {
    if f.len() < 8 { return; }
    let ad = u16::oa([f[4], f[5]]);
    let ls = u16::oa([f[6], f[7]]);
    crate::serial_println!("[ICMPv6] Echo Reply from {}: id={} seq={}", cy, ad, ls);
}





fn tkl(cy: Ipv6Address, f: &[u8]) {
    
    if f.len() < 24 { return; }

    let cd = Ipv6Address::new([
        f[8], f[9], f[10], f[11],
        f[12], f[13], f[14], f[15],
        f[16], f[17], f[18], f[19],
        f[20], f[21], f[22], f[23],
    ]);

    let bvt = ipv6::jdo();

    
    lsq(&f[24..], |fpu, anr| {
        if fpu == ndp_option::XG_ && anr.len() >= 6 {
            let ed = [anr[0], anr[1], anr[2], anr[3], anr[4], anr[5]];
            kfy(cy, ed);
        }
    });

    
    if cd != bvt { return; }

    crate::serial_println!("[NDP] Neighbor Solicitation for {} from {}", cd, cy);

    
    let _ = whi(cy, bvt);
}

fn tkk(cy: Ipv6Address, f: &[u8]) {
    if f.len() < 24 { return; }

    let cd = Ipv6Address::new([
        f[8], f[9], f[10], f[11],
        f[12], f[13], f[14], f[15],
        f[16], f[17], f[18], f[19],
        f[20], f[21], f[22], f[23],
    ]);

    crate::serial_println!("[NDP] Neighbor Advertisement: {} is at {}", cd, cy);

    
    lsq(&f[24..], |fpu, anr| {
        if fpu == ndp_option::BGY_ && anr.len() >= 6 {
            let ed = [anr[0], anr[1], anr[2], anr[3], anr[4], anr[5]];
            kfy(cd, ed);
        }
    });
}





fn tkt(cy: Ipv6Address, f: &[u8]) {
    
    
    if f.len() < 16 { return; }

    let iyn = f[4];
    let waj = u16::oa([f[6], f[7]]);

    crate::serial_println!("[NDP] Router Advertisement from {}: hop_limit={} lifetime={}s", 
        cy, iyn, waj);

    
    lsq(&f[16..], |fpu, anr| {
        match fpu {
            ndp_option::CKT_ if anr.len() >= 30 => {
                let vkp = anr[0];
                let flags = anr[1];
                let adx = &anr[14..30];
                crate::serial_println!("[NDP]   Prefix: {:02x}{:02x}:{:02x}{:02x}::/{} flags={:#x}",
                    adx[0], adx[1], adx[2], adx[3], vkp, flags);
            }
            ndp_option::XG_ if anr.len() >= 6 => {
                let ed = [anr[0], anr[1], anr[2], anr[3], anr[4], anr[5]];
                kfy(cy, ed);
                crate::serial_println!("[NDP]   Router MAC: {:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
                    ed[0], ed[1], ed[2], ed[3], ed[4], ed[5]);
            }
            ndp_option::Che if anr.len() >= 6 => {
                let uql = u32::oa([anr[2], anr[3], anr[4], anr[5]]);
                crate::serial_println!("[NDP]   MTU: {}", uql);
            }
            _ => {}
        }
    });
}






pub fn whn(bvt: Ipv6Address) -> Result<(), &'static str> {
    let ed = crate::drivers::net::cez()
        .or_else(crate::network::ckt)
        .unwrap_or([0; 6]);

    let cs = Ipv6Address::BKC_;

    
    let mut fr = Vec::fc(16);
    fr.push(icmpv6_type::CPC_);
    fr.push(0); 
    fr.push(0); fr.push(0); 
    fr.bk(&[0u8; 4]); 

    
    fr.push(ndp_option::XG_);
    fr.push(1); 
    fr.bk(&ed);

    
    let td = ipv6::izf(&bvt, &cs, &fr);
    fr[2] = (td >> 8) as u8;
    fr[3] = (td & 0xFF) as u8;

    crate::serial_println!("[NDP] Sending Router Solicitation");
    ipv6::joj(bvt, cs, next_header::Xb, 255, &fr)
}


fn whi(cs: Ipv6Address, bvt: Ipv6Address) -> Result<(), &'static str> {
    let ed = crate::drivers::net::cez()
        .or_else(crate::network::ckt)
        .unwrap_or([0; 6]);

    
    let mut fr = Vec::fc(32);
    fr.push(icmpv6_type::BBE_);
    fr.push(0); 
    fr.push(0); fr.push(0); 

    
    let flags: u32 = (1 << 30) | (1 << 29);
    fr.bk(&flags.ft());

    
    fr.bk(&bvt.0);

    
    fr.push(ndp_option::BGY_);
    fr.push(1); 
    fr.bk(&ed);

    
    let td = ipv6::izf(&bvt, &cs, &fr);
    fr[2] = (td >> 8) as u8;
    fr[3] = (td & 0xFF) as u8;

    crate::serial_println!("[NDP] Sending Neighbor Advertisement to {}", cs);
    ipv6::joj(bvt, cs, next_header::Xb, 255, &fr)
}


pub fn zmi(cd: Ipv6Address) -> Result<(), &'static str> {
    let bvt = ipv6::jdo();
    let ed = crate::drivers::net::cez()
        .or_else(crate::network::ckt)
        .unwrap_or([0; 6]);

    let cs = cd.pma();

    
    let mut fr = Vec::fc(32);
    fr.push(icmpv6_type::BBG_);
    fr.push(0); 
    fr.push(0); fr.push(0); 
    fr.bk(&[0u8; 4]); 
    fr.bk(&cd.0);

    
    fr.push(ndp_option::XG_);
    fr.push(1);
    fr.bk(&ed);

    
    let td = ipv6::izf(&bvt, &cs, &fr);
    fr[2] = (td >> 8) as u8;
    fr[3] = (td & 0xFF) as u8;

    crate::serial_println!("[NDP] Sending Neighbor Solicitation for {}", cd);
    ipv6::joj(bvt, cs, next_header::Xb, 255, &fr)
}





fn lsq<G: FnMut(u8, &[u8])>(f: &[u8], mut cfd: G) {
    let mut a = 0;
    while a + 2 <= f.len() {
        let fpu = f[a];
        let jhw = f[a + 1] as usize * 8; 
        if jhw == 0 || a + jhw > f.len() { break; }
        
        cfd(fpu, &f[a + 2..a + jhw]);
        a += jhw;
    }
}
