





use alloc::vec::Vec;
use alloc::string::String;
use alloc::format;


#[derive(Debug, Clone)]
pub struct Pp {
    pub ip: [u8; 4],
    pub ed: Option<[u8; 6]>,
    pub ajc: Option<String>,
    pub akv: Option<u8>,
    pub bcj: u64,
    pub fpv: &'static str,
}


fn nzv(akv: u8) -> &'static str {
    match akv {
        
        ab if ab <= 32 => "Unknown (low TTL)",
        ab if ab <= 64 => "Linux/Unix/macOS",
        ab if ab <= 128 => "Windows",
        _ => "Cisco/Network device",
    }
}





pub fn qkp(wvv: [u8; 4], wvu: [u8; 4], sg: u32) -> Vec<Pp> {
    let mut bab = Vec::new();

    let jrp = u32::oa(wvv);
    let ist = u32::oa(wvu);

    if ist < jrp || ist - jrp > 1024 {
        return bab; 
    }

    
    for flx in jrp..=ist {
        let ip = flx.ft();
        let _ = crate::netstack::arp::eii(ip);
        
        for _ in 0..10000 { core::hint::hc(); }
    }

    
    let ay = crate::logger::lh();
    let mut aaf: u32 = 0;
    loop {
        crate::netstack::poll();

        if crate::logger::lh().ao(ay) > sg as u64 {
            break;
        }
        aaf = aaf.cn(1);
        if aaf > 1_000_000 { break; }
        crate::arch::bhd();
    }

    
    let ch = crate::netstack::arp::ch();
    for (flx, ed) in ch {
        let ip = flx.ft();
        
        if flx >= jrp && flx <= ist {
            bab.push(Pp {
                ip,
                ed: Some(ed),
                ajc: None,
                akv: None,
                bcj: 0,
                fpv: "Unknown",
            });
        }
    }

    bab.bxf(|i| u32::oa(i.ip));
    bab
}


pub fn kbb(sg: u32) -> Vec<Pp> {
    let (aro, up, _) = match crate::network::aou() {
        Some((ip, hs, nt)) => (*ip.as_bytes(), *hs.as_bytes(), nt),
        None => return Vec::new(),
    };

    
    let usf = [
        aro[0] & up[0],
        aro[1] & up[1],
        aro[2] & up[2],
        (aro[3] & up[3]).cn(1), 
    ];
    let usd = [
        aro[0] | !up[0],
        aro[1] | !up[1],
        aro[2] | !up[2],
        (aro[3] | !up[3]).nj(1), 
    ];

    qkp(usf, usd, sg)
}




pub fn via(jsp: &[[u8; 4]], sg: u32) -> Vec<Pp> {
    let mut bab = Vec::new();

    crate::netstack::icmp::hcx();

    for (a, &cd) in jsp.iter().cf() {
        let ls = (a + 1) as u16;
        let ay = crate::logger::lh();

        if crate::netstack::icmp::mdr(cd, 0x5CA2, ls).is_err() {
            continue;
        }

        match crate::netstack::icmp::mqe(ls, sg) {
            Some(lj) if lj.vx => {
                let bcj = crate::logger::lh().ao(ay);
                bab.push(Pp {
                    ip: cd,
                    ed: crate::netstack::arp::ayo(cd),
                    ajc: None,
                    akv: Some(lj.akv),
                    bcj,
                    fpv: nzv(lj.akv),
                });
            }
            _ => {} 
        }
    }

    bab
}


pub fn vib(kcf: [u8; 4], xhe: u32) -> Vec<Pp> {
    let mut jsp = Vec::new();
    for a in 1..=254u8 {
        jsp.push([kcf[0], kcf[1], kcf[2], a]);
    }
    via(&jsp, xhe)
}


pub fn syx(sg: u32) -> Vec<Pp> {
    
    let mut bab = kbb(sg);

    
    for kh in &mut bab {
        crate::netstack::icmp::hcx();
        let ay = crate::logger::lh();
        if crate::netstack::icmp::mdr(kh.ip, 0x5CA1, 1).is_ok() {
            if let Some(lj) = crate::netstack::icmp::mqe(1, 500) {
                kh.akv = Some(lj.akv);
                kh.bcj = crate::logger::lh().ao(ay);
                kh.fpv = nzv(lj.akv);
            }
        }
    }

    bab
}
