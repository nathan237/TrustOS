





use alloc::vec::Vec;
use alloc::string::String;
use alloc::format;
use core::sync::atomic::{AtomicU16, Ordering};

static CYI_: AtomicU16 = AtomicU16::new(1000);


#[derive(Debug, Clone)]
pub struct Anq {
    pub gjd: u8,
    pub ip: Option<[u8; 4]>,
    pub ajc: Option<String>,
    pub bcj: [u64; 3],   
    pub gqi: bool,      
}


pub struct TraceConfig {
    pub fnv: u8,
    pub oya: u8,
    pub sg: u32,
    pub cd: [u8; 4],
}

impl Default for TraceConfig {
    fn default() -> Self {
        Self {
            fnv: 30,
            oya: 3,
            sg: 2000,
            cd: [0; 4],
        }
    }
}


pub fn trace(cd: [u8; 4], fnv: u8, sg: u32) -> Vec<Anq> {
    let mut cyn = Vec::new();

    crate::netstack::icmp::hcx();
    crate::netstack::icmp::ndg();

    for akv in 1..=fnv {
        let mut bhe = Anq {
            gjd: akv,
            ip: None,
            ajc: None,
            bcj: [0; 3],
            gqi: false,
        };

        let mut nzk = false;

        for probe in 0..3 {
            let ls = CYI_.fetch_add(1, Ordering::Relaxed);

            crate::netstack::icmp::hcx();
            crate::netstack::icmp::ndg();

            
            let mut cyp = Vec::new();
            cyp.push(8); 
            cyp.push(0); 
            cyp.push(0); cyp.push(0); 
            cyp.bk(&0x5CA1u16.ft()); 
            cyp.bk(&ls.ft());

            
            let wi = crate::time::lc() as u32;
            cyp.bk(&wi.ft());
            for a in 0..20 {
                cyp.push((0x41 + a) as u8);
            }

            
            let td = lcz(&cyp);
            cyp[2] = (td >> 8) as u8;
            cyp[3] = (td & 0xFF) as u8;

            let ay = crate::logger::lh();

            
            if crate::netstack::ip::whj(cd, 1, &cyp, akv).is_err() {
                bhe.bcj[probe] = 0;
                continue;
            }

            
            let result = crate::netstack::icmp::xtk(ls, cd, sg);
            let ez = crate::logger::lh().ao(ay);

            match result {
                crate::netstack::icmp::TracerouteResult::Bqn { ip, .. } => {
                    bhe.ip = Some(ip);
                    bhe.bcj[probe] = ez;
                    bhe.gqi = true;
                    nzk = true;
                }
                crate::netstack::icmp::TracerouteResult::Biu { ip, bcj, .. } => {
                    bhe.ip = Some(ip);
                    bhe.bcj[probe] = bcj;
                    nzk = true;
                }
                crate::netstack::icmp::TracerouteResult::Oi => {
                    bhe.bcj[probe] = 0; 
                }
            }
        }

        cyn.push(bhe.clone());

        if bhe.gqi {
            break;
        }
    }

    cyn
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


pub fn swb(cyn: &[Anq]) -> String {
    let mut an = String::new();
    for bhe in cyn {
        an.t(&format!("{:>2}  ", bhe.gjd));

        if let Some(ip) = bhe.ip {
            an.t(&super::aot(ip));
            an.t("  ");
            for &ehv in &bhe.bcj {
                if ehv == 0 {
                    an.t("*  ");
                } else if ehv < 1 {
                    an.t("<1 ms  ");
                } else {
                    an.t(&format!("{} ms  ", ehv));
                }
            }
        } else {
            an.t("* * *");
        }

        an.push('\n');
    }
    an
}
