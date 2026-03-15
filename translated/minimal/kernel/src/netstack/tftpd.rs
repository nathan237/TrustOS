








use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use spin::Mutex;


static Ja: AtomicBool = AtomicBool::new(false);


static ACB_: AtomicU64 = AtomicU64::new(0);


static YK_: Mutex<BTreeMap<&'static str, &'static [u8]>> = Mutex::new(BTreeMap::new());


static Ue: Mutex<BTreeMap<u16, Ant>> = Mutex::new(BTreeMap::new());


static VT_: Mutex<u16> = Mutex::new(50000);


mod opcode {
    pub const Ckb: u16 = 1;   
    pub const Cqg: u16 = 2;   
    pub const Bdz: u16 = 3;  
    pub const Ie: u16 = 4;   
    pub const Sf: u16 = 5; 
}


mod error_code {
    pub const BBP_: u16 = 1;
    pub const BJH_: u16 = 2;
    pub const ELF_: u16 = 3;
    pub const AWX_: u16 = 4;
}


const AR_: usize = 512;


struct Ant {
    
    bjn: [u8; 4],
    
    dex: u16,
    
    ahq: u16,
    
    cxw: &'static [u8],
    
    fge: u16,
    
    fxa: u16,
    
    kke: bool,
    
    clg: u64,
    
    arv: u8,
}


pub fn dsi() -> bool {
    Ja.load(Ordering::Relaxed)
}


pub fn kvt() -> u64 {
    ACB_.load(Ordering::Relaxed)
}


pub fn exo(j: &'static str, f: &'static [u8]) {
    let mut sb = YK_.lock();
    sb.insert(j, f);
    crate::serial_println!("[TFTPD] Registered file '{}' ({} bytes)", j, f.len());
}


pub fn ay() {
    if Ja.load(Ordering::Relaxed) {
        crate::serial_println!("[TFTPD] Already running");
        return;
    }

    Ja.store(true, Ordering::Relaxed);
    ACB_.store(0, Ordering::Relaxed);

    let sb = YK_.lock();
    crate::serial_println!("[TFTPD] TFTP server started on port 69 ({} files registered)", sb.len());
    for (j, f) in sb.iter() {
        crate::serial_println!("[TFTPD]   {} ({} bytes, {} blocks)",
            j, f.len(), (f.len() + AR_ - 1) / AR_);
    }
}


pub fn qg() {
    Ja.store(false, Ordering::Relaxed);
    let mut chu = Ue.lock();
    chu.clear();
    crate::serial_println!("[TFTPD] Server stopped");
}



pub fn tkr(f: &[u8], jh: [u8; 4], ey: u16) {
    if !Ja.load(Ordering::Relaxed) || f.len() < 4 {
        return;
    }

    let opcode = u16::oa([f[0], f[1]]);

    match opcode {
        opcode::Ckb => tkq(f, jh, ey),
        opcode::Cqg => {
            
            hzp(jh, ey, 69, error_code::BJH_, "Write not supported");
        }
        _ => {
            hzp(jh, ey, 69, error_code::AWX_, "Invalid opcode");
        }
    }
}


pub fn tlk(f: &[u8], jh: [u8; 4], ey: u16, ahq: u16) {
    if !Ja.load(Ordering::Relaxed) || f.len() < 4 {
        return;
    }

    let opcode = u16::oa([f[0], f[1]]);

    if opcode == opcode::Ie {
        let block = u16::oa([f[2], f[3]]);
        tiw(jh, ey, ahq, block);
    }
}


fn tkq(f: &[u8], bjn: [u8; 4], dex: u16) {
    
    let ew = &f[2..];

    
    let ssf = match ew.iter().qf(|&o| o == 0) {
        Some(u) => u,
        None => {
            hzp(bjn, dex, 69, error_code::AWX_, "Bad request");
            return;
        }
    };

    let it = match core::str::jg(&ew[..ssf]) {
        Ok(e) => e,
        Err(_) => {
            hzp(bjn, dex, 69, error_code::BBP_, "Invalid filename");
            return;
        }
    };

    
    let doy = it.tl('/');

    crate::serial_println!("[TFTPD] RRQ for '{}' from {}.{}.{}.{}:{}",
        doy,
        bjn[0], bjn[1], bjn[2], bjn[3],
        dex);

    
    let sb = YK_.lock();
    let cxw = match sb.get(doy) {
        Some(f) => *f,
        None => {
            
            let qhk = if doy.cj("boot/") {
                &doy[5..]
            } else {
                doy
            };
            match sb.get(qhk) {
                Some(f) => *f,
                None => {
                    drop(sb);
                    crate::serial_println!("[TFTPD] File not found: '{}'", doy);
                    hzp(bjn, dex, 69, error_code::BBP_, "File not found");
                    return;
                }
            }
        }
    };
    drop(sb);

    
    let ahq = {
        let mut ni = VT_.lock();
        let port = *ni;
        *ni = ni.cn(1);
        if *ni < 50000 { *ni = 50000; }
        port
    };

    let fxa = ((cxw.len() + AR_ - 1) / AR_).am(1) as u16;

    crate::serial_println!("[TFTPD] Starting transfer: '{}' ({} bytes, {} blocks) TID={}",
        doy, cxw.len(), fxa, ahq);

    
    let he = Ant {
        bjn,
        dex,
        ahq,
        cxw,
        fge: 1,
        fxa,
        kke: false,
        clg: crate::time::lc(),
        arv: 0,
    };

    
    mdp(&he);

    
    let mut chu = Ue.lock();
    chu.insert(ahq, he);
}


fn tiw(bjn: [u8; 4], dex: u16, ahq: u16, block: u16) {
    let mut chu = Ue.lock();

    let he = match chu.ds(&ahq) {
        Some(e) => e,
        None => return,
    };

    
    if he.bjn != bjn || he.dex != dex {
        return;
    }

    if block == he.fge {
        
        he.fge += 1;
        he.arv = 0;

        if he.fge > he.fxa {
            
            let ucb = he.cxw.len() % AR_;
            if ucb == 0 && !he.cxw.is_empty() {
                
                whd(he);
            }
            crate::serial_println!("[TFTPD] Transfer complete for TID={}", ahq);
            he.kke = true;
            ACB_.fetch_add(1, Ordering::Relaxed);
            let port = ahq;
            drop(chu);
            rbd(port);
            return;
        }

        
        mdp(he);
        he.clg = crate::time::lc();
    }
    
}


fn mdp(he: &Ant) {
    let block = he.fge;
    let l = ((block - 1) as usize) * AR_;
    let ci = (l + AR_).v(he.cxw.len());

    let njq = if l < he.cxw.len() {
        &he.cxw[l..ci]
    } else {
        &[]
    };

    
    let mut ex = Vec::fc(4 + njq.len());
    ex.bk(&opcode::Bdz.ft());
    ex.bk(&block.ft());
    ex.bk(njq);

    let _ = crate::netstack::udp::dlp(
        he.bjn,
        he.dex,
        he.ahq,
        &ex,
    );
}


fn whd(he: &Ant) {
    let block = he.fge;
    let mut ex = Vec::fc(4);
    ex.bk(&opcode::Bdz.ft());
    ex.bk(&block.ft());

    let _ = crate::netstack::udp::dlp(
        he.bjn,
        he.dex,
        he.ahq,
        &ex,
    );
}


fn hzp(kv: [u8; 4], rz: u16, ey: u16, aj: u16, fr: &str) {
    let ooj = fr.as_bytes();
    let mut ex = Vec::fc(5 + ooj.len());
    ex.bk(&opcode::Sf.ft());
    ex.bk(&aj.ft());
    ex.bk(ooj);
    ex.push(0); 

    let _ = crate::netstack::udp::dlp(kv, rz, ey, &ex);
}


fn rbd(port: u16) {
    let mut chu = Ue.lock();
    chu.remove(&port);
}


pub fn poll() {
    if !Ja.load(Ordering::Relaxed) {
        return;
    }

    let iu = crate::time::lc();
    let mut chu = Ue.lock();
    let mut cik = Vec::new();

    for (port, he) in chu.el() {
        if he.kke {
            cik.push(*port);
            continue;
        }

        
        if iu.ao(he.clg) > 3000 {
            if he.arv >= 5 {
                crate::serial_println!("[TFTPD] Transfer timeout for TID={}", port);
                cik.push(*port);
            } else {
                he.arv += 1;
                he.clg = iu;
                mdp(he);
                crate::serial_println!("[TFTPD] Retransmit block {} for TID={} (retry {})",
                    he.fge, port, he.arv);
            }
        }
    }

    for port in cik {
        chu.remove(&port);
    }
}


pub fn jdr() -> Vec<(&'static str, usize)> {
    let sb = YK_.lock();
    sb.iter().map(|(j, f)| (*j, f.len())).collect()
}


pub fn jzc() -> usize {
    let chu = Ue.lock();
    chu.len()
}


pub fn tzf(port: u16) -> bool {
    let chu = Ue.lock();
    chu.bgm(&port)
}
