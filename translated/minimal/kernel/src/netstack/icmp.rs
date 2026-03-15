

use alloc::vec::Vec;
use spin::Mutex;


pub const AWS_: u8 = 0;
pub const AWR_: u8 = 3;
pub const AWU_: u8 = 11;
pub const AWT_: u8 = 8;


static WG_: Mutex<Vec<Awx>> = Mutex::new(Vec::new());


static UA_: Mutex<Vec<Atz>> = Mutex::new(Vec::new());


#[derive(Debug, Clone, Copy)]
pub struct Atz {
    pub hih: u8,        
    pub aj: u8,
    pub bct: [u8; 4],   
    pub lqv: [u8; 4], 
    pub uzi: u8,     
    pub uzg: u16,       
}


#[derive(Debug, Clone, Copy)]
pub struct Awx {
    pub ls: u16,
    pub akv: u8,
    pub vx: bool,
}


#[repr(C, packed)]
struct Czk {
    ifk: u8,
    aj: u8,
    bmj: u16,
    cys: u16,
    eil: u16,
}


fn bmj(f: &[u8]) -> u16 {
    let mut sum: u32 = 0;
    let mut a = 0;
    
    
    while a < f.len() - 1 {
        sum += ((f[a] as u32) << 8) | (f[a + 1] as u32);
        a += 2;
    }
    
    
    if a < f.len() {
        sum += (f[a] as u32) << 8;
    }
    
    
    while sum >> 16 != 0 {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }
    
    !sum as u16
}


pub fn bur(f: &[u8], akv: u8, bct: [u8; 4]) {
    if f.len() < 8 {
        return;
    }
    
    let ifk = f[0];
    let aj = f[1];
    let ad = u16::oa([f[4], f[5]]);
    let ls = u16::oa([f[6], f[7]]);
    
    match ifk {
        AWT_ => {
            crate::serial_println!("[ICMP] Echo request id={} seq={}", ad, ls);
            whc(ad, ls, &f[8..]);
        }
        AWS_ => {
            crate::serial_println!("[ICMP] Echo reply id={} seq={} ttl={}", ad, ls, akv);
            WG_.lock().push(Awx {
                ls,
                akv,
                vx: true,
            });
        }
        AWU_ | AWR_ => {
            
            
            if f.len() >= 8 + 20 {
                let fhn = &f[8..];
                let htv = [fhn[16], fhn[17], fhn[18], fhn[19]];
                let uzd = fhn[9];
                let uzb = u16::oa([fhn[4], fhn[5]]);
                crate::serial_println!("[ICMP] {} from {}.{}.{}.{} code={} orig_dest={}.{}.{}.{}",
                    if ifk == AWU_ { "Time Exceeded" } else { "Dest Unreachable" },
                    bct[0], bct[1], bct[2], bct[3],
                    aj,
                    htv[0], htv[1], htv[2], htv[3]);
                UA_.lock().push(Atz {
                    hih: ifk,
                    aj,
                    bct,
                    lqv: htv,
                    uzi: uzd,
                    uzg: uzb,
                });
            }
        }
        _ => {
            crate::serial_println!("[ICMP] Type {} code {} (unhandled)", ifk, aj);
        }
    }
}


pub fn mdr(kv: [u8; 4], ad: u16, ls: u16) -> Result<(), &'static str> {
    
    let mut ex = Vec::new();
    
    
    ex.push(AWT_); 
    ex.push(0); 
    ex.push(0); ex.push(0); 
    ex.bk(&ad.ft());
    ex.bk(&ls.ft());
    
    
    let aea = crate::time::lc() as u32;
    ex.bk(&aea.ft());
    for a in 0..52 {
        ex.push((0x10 + a) as u8); 
    }
    
    
    let td = bmj(&ex);
    ex[2] = (td >> 8) as u8;
    ex[3] = (td & 0xFF) as u8;
    
    
    crate::netstack::ip::blc(kv, 1, &ex)?;
    
    crate::serial_println!("[ICMP] Sent echo request to {}.{}.{}.{} id={} seq={}", 
        kv[0], kv[1], kv[2], kv[3], ad, ls);
    
    Ok(())
}


fn whc(ad: u16, ls: u16, ew: &[u8]) {
    
    let mut ex = Vec::new();
    
    ex.push(AWS_); 
    ex.push(0); 
    ex.push(0); ex.push(0); 
    ex.bk(&ad.ft());
    ex.bk(&ls.ft());
    ex.bk(ew);
    
    
    let td = bmj(&ex);
    ex[2] = (td >> 8) as u8;
    ex[3] = (td & 0xFF) as u8;
    
    
    crate::serial_println!("[ICMP] Would send echo reply id={} seq={}", ad, ls);
}


pub fn mqe(ls: u16, sg: u32) -> Option<Awx> {
    let ay = crate::logger::lh();
    let mut aaf: u32 = 0;
    
    loop {
        
        crate::netstack::poll();

        
        let mut gqy = WG_.lock();
        if let Some(u) = gqy.iter().qf(|m| m.ls == ls) {
            let mk = gqy.remove(u);
            return Some(mk);
        }
        drop(gqy);
        
        
        if crate::logger::lh() - ay > sg as u64 {
            return None;
        }

        aaf = aaf.cn(1);
        if aaf > 2_000_000 {
            return None;
        }
        
        
        crate::arch::bhd();
    }
}


pub fn hcx() {
    WG_.lock().clear();
}


pub fn xti(kv: [u8; 4], sg: u32) -> Option<Atz> {
    let ay = crate::logger::lh();
    let mut aaf: u32 = 0;
    loop {
        crate::netstack::poll();

        let mut bqn = UA_.lock();
        if let Some(u) = bqn.iter().qf(|aa| aa.lqv == kv) {
            return Some(bqn.remove(u));
        }
        drop(bqn);

        if crate::logger::lh().ao(ay) > sg as u64 {
            return None;
        }
        aaf = aaf.cn(1);
        if aaf > 2_000_000 { return None; }
        crate::arch::bhd();
    }
}


pub fn xtk(ls: u16, kv: [u8; 4], sg: u32) -> TracerouteResult {
    let ay = crate::logger::lh();
    let mut aaf: u32 = 0;
    loop {
        crate::netstack::poll();

        
        {
            let mut gqy = WG_.lock();
            if let Some(u) = gqy.iter().qf(|m| m.ls == ls) {
                let lj = gqy.remove(u);
                let ez = crate::logger::lh().ao(ay);
                return TracerouteResult::Bqn { ip: kv, akv: lj.akv, bcj: ez };
            }
        }

        
        {
            let mut bqn = UA_.lock();
            if let Some(u) = bqn.iter().qf(|aa| aa.lqv == kv) {
                let rq = bqn.remove(u);
                let ez = crate::logger::lh().ao(ay);
                return TracerouteResult::Biu { ip: rq.bct, bcj: ez, hih: rq.hih };
            }
        }

        if crate::logger::lh().ao(ay) > sg as u64 {
            return TracerouteResult::Oi;
        }
        aaf = aaf.cn(1);
        if aaf > 2_000_000 { return TracerouteResult::Oi; }
        crate::arch::bhd();
    }
}


pub fn ndg() {
    UA_.lock().clear();
}


#[derive(Debug, Clone, Copy)]
pub enum TracerouteResult {
    Biu { ip: [u8; 4], bcj: u64, hih: u8 },
    Bqn { ip: [u8; 4], akv: u8, bcj: u64 },
    Oi,
}
