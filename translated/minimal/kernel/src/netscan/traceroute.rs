





use alloc::vec::Vec;
use alloc::string::String;
use alloc::format;
use core::sync::atomic::{AtomicU16, Ordering};

static DCA_: AtomicU16 = AtomicU16::new(1000);


#[derive(Debug, Clone)]
pub struct Qn {
    pub hop_num: u8,
    pub ip: Option<[u8; 4]>,
    pub hostname: Option<String>,
    pub rtt_ms: [u64; 3],   
    pub reached: bool,      
}


pub struct TraceConfig {
    pub max_hops: u8,
    pub probes_per_hop: u8,
    pub timeout_ms: u32,
    pub target: [u8; 4],
}

impl Default for TraceConfig {
    fn default() -> Self {
        Self {
            max_hops: 30,
            probes_per_hop: 3,
            timeout_ms: 2000,
            target: [0; 4],
        }
    }
}


pub fn trace(target: [u8; 4], max_hops: u8, timeout_ms: u32) -> Vec<Qn> {
    let mut bcb = Vec::new();

    crate::netstack::icmp::dkt();
    crate::netstack::icmp::hlh();

    for ttl in 1..=max_hops {
        let mut afg = Qn {
            hop_num: ttl,
            ip: None,
            hostname: None,
            rtt_ms: [0; 3],
            reached: false,
        };

        let mut icn = false;

        for probe in 0..3 {
            let seq = DCA_.fetch_add(1, Ordering::Relaxed);

            crate::netstack::icmp::dkt();
            crate::netstack::icmp::hlh();

            
            let mut bce = Vec::new();
            bce.push(8); 
            bce.push(0); 
            bce.push(0); bce.push(0); 
            bce.extend_from_slice(&0x5CA1u16.to_be_bytes()); 
            bce.extend_from_slice(&seq.to_be_bytes());

            
            let jy = crate::time::uptime_ms() as u32;
            bce.extend_from_slice(&jy.to_be_bytes());
            for i in 0..20 {
                bce.push((0x41 + i) as u8);
            }

            
            let ig = gbl(&bce);
            bce[2] = (ig >> 8) as u8;
            bce[3] = (ig & 0xFF) as u8;

            let start = crate::logger::eg();

            
            if crate::netstack::ip::onw(target, 1, &bce, ttl).is_err() {
                afg.rtt_ms[probe] = 0;
                continue;
            }

            
            let result = crate::netstack::icmp::ptk(seq, target, timeout_ms);
            let bb = crate::logger::eg().saturating_sub(start);

            match result {
                crate::netstack::icmp::TracerouteResult::Reached { ip, .. } => {
                    afg.ip = Some(ip);
                    afg.rtt_ms[probe] = bb;
                    afg.reached = true;
                    icn = true;
                }
                crate::netstack::icmp::TracerouteResult::Hop { ip, rtt_ms, .. } => {
                    afg.ip = Some(ip);
                    afg.rtt_ms[probe] = rtt_ms;
                    icn = true;
                }
                crate::netstack::icmp::TracerouteResult::Timeout => {
                    afg.rtt_ms[probe] = 0; 
                }
            }
        }

        bcb.push(afg.clone());

        if afg.reached {
            break;
        }
    }

    bcb
}


fn gbl(data: &[u8]) -> u16 {
    let mut sum: u32 = 0;
    let mut i = 0;
    while i + 1 < data.len() {
        sum += ((data[i] as u32) << 8) | (data[i + 1] as u32);
        i += 2;
    }
    if i < data.len() {
        sum += (data[i] as u32) << 8;
    }
    while (sum >> 16) != 0 {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }
    !(sum as u16)
}


pub fn lxu(bcb: &[Qn]) -> String {
    let mut output = String::new();
    for afg in bcb {
        output.push_str(&format!("{:>2}  ", afg.hop_num));

        if let Some(ip) = afg.ip {
            output.push_str(&super::uw(ip));
            output.push_str("  ");
            for &rtt in &afg.rtt_ms {
                if rtt == 0 {
                    output.push_str("*  ");
                } else if rtt < 1 {
                    output.push_str("<1 ms  ");
                } else {
                    output.push_str(&format!("{} ms  ", rtt));
                }
            }
        } else {
            output.push_str("* * *");
        }

        output.push('\n');
    }
    output
}
