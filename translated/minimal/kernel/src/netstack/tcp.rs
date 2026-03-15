

use alloc::collections::{BTreeMap, VecDeque};
use alloc::string::String;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU16, Ordering};
use spin::Mutex;


pub mod flags {
    pub const Bgl: u8 = 0x01;
    pub const Ame: u8 = 0x02;
    pub const Bqg: u8 = 0x04;
    pub const Awp: u8 = 0x08;
    pub const Ie: u8 = 0x10;
    pub const Dkg: u8 = 0x20;
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TcpState {
    Dk,
    Dbi,
    Btv,
    Btu,
    Pi,
    Bhd,
    Bhe,
    Apz,
    Bkq,
    Aey,
    Pb,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Gt {
    jh: u32,
    pz: u32,
    ey: u16,
    sa: u16,
}

#[derive(Debug, Clone)]
struct Azt {
    g: TcpState,
    ls: u32, 
    alx: u32, 
    bqr: bool,
    fis: bool,
    
    egi: u8,
    dit: u64,
    
    jcr: u32,
    lhv: u64,
    lzx: u8,
    
    iay: u32,
}


#[derive(Clone)]
struct Bvg {
    vg: Gt,
    ls: u32,
    f: Vec<u8>,       
    kv: [u8; 4],
    rz: u16,
    ey: u16,
    mdv: u64,
    arv: u8,
}



static PF_: Mutex<VecDeque<Bvg>> = Mutex::new(VecDeque::new());

const CFP_: usize = 32;

static Fh: Mutex<BTreeMap<Gt, Azt>> = Mutex::new(BTreeMap::new());
static LA_: Mutex<BTreeMap<Gt, VecDeque<Vec<u8>>>> = Mutex::new(BTreeMap::new());
static VS_: AtomicU16 = AtomicU16::new(49152);

static AXJ_: Mutex<[u8; 16]> = Mutex::new([0u8; 16]);



fn nxj(jh: [u8; 4], pz: [u8; 4], ey: u16, sa: u16) -> u32 {
    let eig = AXJ_.lock();
    let mut f = [0u8; 28]; 
    f[0..4].dg(&jh);
    f[4..8].dg(&pz);
    f[8..10].dg(&ey.ft());
    f[10..12].dg(&sa.ft());
    f[12..28].dg(&*eig);
    drop(eig);
    let hash = crate::tls13::crypto::chw(&f);
    let i = u32::oa([hash[0], hash[1], hash[2], hash[3]]);
    
    let qb = crate::logger::lh() as u32;
    i.cn(qb)
}


pub fn tts() {
    let mut e = AXJ_.lock();
    crate::rng::phh(&mut *e);
    crate::serial_println!("[TCP] ISN secret initialized (RFC 6528)");
}


struct Blj {
    dea: u32,
    iis: VecDeque<Gt>,
}


static Acn: Mutex<BTreeMap<u16, Blj>> = Mutex::new(BTreeMap::new());


pub fn jdt(port: u16, dea: u32) {
    Acn.lock().insert(port, Blj { dea, iis: VecDeque::new() });
    crate::serial_println!("[TCP] Listening on port {}", port);
}


pub fn mhr(port: u16) {
    Acn.lock().remove(&port);
}



pub fn iir(fnd: u16) -> Option<(u16, [u8; 4], u16)> {
    let mut glp = Acn.lock();
    let glo = glp.ds(&fnd)?;
    let vg = glo.iis.awp()?;
    let ams = [
        ((vg.pz >> 24) & 0xFF) as u8,
        ((vg.pz >> 16) & 0xFF) as u8,
        ((vg.pz >> 8) & 0xFF) as u8,
        (vg.pz & 0xFF) as u8,
    ];
    Some((vg.ey, ams, vg.sa))
}


const BRJ_: u8 = 4;
const BRI_: u64 = 20;


const JD_: u16 = 65535;


const BEL_: u64 = 1000;

const AFE_: u8 = 3;


const AEX_: usize = 512;

const BHF_: u64 = 60_000;

fn bad(ip: [u8; 4]) -> u32 {
    ((ip[0] as u32) << 24) | ((ip[1] as u32) << 16) | ((ip[2] as u32) << 8) | (ip[3] as u32)
}


fn hcs(sum: &mut u32, f: &[u8]) {
    let mut a = 0;
    while a + 1 < f.len() {
        *sum += ((f[a] as u32) << 8) | (f[a + 1] as u32);
        a += 2;
    }
    if a < f.len() {
        *sum += (f[a] as u32) << 8;
    }
}

fn nct(mut sum: u32) -> u16 {
    while (sum >> 16) != 0 {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }
    !(sum as u16)
}

fn bmj(f: &[u8]) -> u16 {
    let mut sum: u32 = 0;
    hcs(&mut sum, f);
    nct(sum)
}


fn ezm(jh: [u8; 4], pz: [u8; 4], ie: &[u8]) -> u16 {
    let mut sum: u32 = 0;
    
    hcs(&mut sum, &jh);
    hcs(&mut sum, &pz);
    let vnk: [u8; 4] = [0, 6, (ie.len() >> 8) as u8, ie.len() as u8];
    hcs(&mut sum, &vnk);
    hcs(&mut sum, ie);
    nct(sum)
}


pub fn psc(jh: [u8; 4], pz: [u8; 4], ie: &[u8]) -> u16 {
    ezm(jh, pz, ie)
}



pub fn ysi() {
    let iu = crate::logger::lh();
    let mut aan = Fh.lock();
    let mut kb = LA_.lock();
    aan.ajm(|ad, ly| {
        let eec = match ly.g {
            TcpState::Aey => iu.nj(ly.dit) < BHF_,
            TcpState::Dk => false,
            _ => true,
        };
        if !eec {
            kb.remove(ad);
        }
        eec
    });
}


pub fn rnz() -> usize {
    Fh.lock().len()
}


pub fn ufo() -> Vec<String> {
    let aan = Fh.lock();
    let mut bd = Vec::fc(aan.len());
    for (ad, ly) in aan.iter() {
        let cs = [
            ((ad.pz >> 24) & 0xFF) as u8,
            ((ad.pz >> 16) & 0xFF) as u8,
            ((ad.pz >> 8) & 0xFF) as u8,
            (ad.pz & 0xFF) as u8,
        ];
        bd.push(alloc::format!(
            "{:<6} {}.{}.{}.{}:{:<5}  {:?}",
            ad.ey, cs[0], cs[1], cs[2], cs[3], ad.sa, ly.g
        ));
    }
    bd
}

fn cqw() -> [u8; 4] {
    crate::network::aou()
        .map(|(ip, _, _)| *ip.as_bytes())
        .unwrap_or([10, 0, 2, 15])
}


pub fn bur(f: &[u8], jh: [u8; 4], pz: [u8; 4]) {
    if f.len() < 20 {
        return;
    }

    let ey = u16::oa([f[0], f[1]]);
    let sa = u16::oa([f[2], f[3]]);
    let ls = u32::oa([f[4], f[5], f[6], f[7]]);
    let gxq = u32::oa([f[8], f[9], f[10], f[11]]);
    let bbj = f[12] >> 4;
    let dcn = f[13];
    let ect = (bbj as usize) * 4;

    if f.len() < ect {
        return;
    }

    let vg = Gt {
        jh: bad(pz),
        pz: bad(jh),
        ey: sa,
        sa: ey,
    };

    let ew = &f[ect..];
    let iup  = (dcn & flags::Bgl) != 0;
    let jry  = (dcn & flags::Ame) != 0;
    let alx  = (dcn & flags::Ie) != 0;
    let waz  = (dcn & flags::Bqg) != 0;
    let vnw  = (dcn & flags::Awp) != 0;
    let bvx = ew.len() as u32;

    let mut aan = Fh.lock();

    if let Some(ly) = aan.ds(&vg) {
        
        if waz {
            ly.g = TcpState::Dk;
            ly.bqr = true;
            ly.fis = true;
            return;
        }

        if ly.bqr && ly.fis && ly.g == TcpState::Dk {
            return;
        }

        match ly.g {
            
            TcpState::Btv => {
                if jry && alx {
                    ly.g = TcpState::Pi;
                    ly.alx = ls.cn(1);
                    ly.egi = 0;
                    ly.dit = crate::logger::lh();
                    drop(aan);
                    let _ = fue(jh, ey, sa);
                }
            }

            
            TcpState::Btu => {
                if alx {
                    ly.g = TcpState::Pi;
                    let fnd = vg.ey;
                    drop(aan);
                    
                    let mut glp = Acn.lock();
                    if let Some(glo) = glp.ds(&fnd) {
                        glo.iis.agt(vg);
                    }
                }
            }

            
            TcpState::Pi => {
                
                if alx && gxq > ly.iay {
                    ly.iay = gxq;
                    ly.lzx = 0;
                    
                    let mut exw = PF_.lock();
                    exw.ajm(|pk| {
                        pk.vg != vg || pk.ls.cn(pk.f.len() as u32) > gxq
                    });
                }

                
                if !ew.is_empty() {
                    let oph = ls.cn(bvx);
                    if oph > ly.alx {
                        ly.alx = oph;
                        let mut kb = LA_.lock();
                        kb.bt(vg).clq(VecDeque::new).agt(ew.ip());
                    }
                }

                if iup {
                    
                    ly.alx = ls.cn(bvx).cn(1);
                    ly.bqr = true;
                    ly.g = TcpState::Apz;
                    drop(aan);
                    let _ = fue(jh, ey, sa);
                    return;
                }

                
                if !ew.is_empty() {
                    ly.egi = ly.egi.akq(1);
                    let iu = crate::logger::lh();
                    let mfo = vnw
                        || ly.egi >= BRJ_
                        || iu.ao(ly.dit) >= BRI_;
                    if mfo {
                        ly.egi = 0;
                        ly.dit = iu;
                        drop(aan);
                        let _ = fue(jh, ey, sa);
                    }
                }
            }

            
            TcpState::Bhd => {
                if iup && alx {
                    
                    ly.alx = ls.cn(1);
                    ly.bqr = true;
                    ly.g = TcpState::Aey;
                    ly.dit = crate::logger::lh();
                    drop(aan);
                    let _ = fue(jh, ey, sa);
                } else if iup {
                    ly.alx = ls.cn(1);
                    ly.bqr = true;
                    ly.g = TcpState::Pb;
                    drop(aan);
                    let _ = fue(jh, ey, sa);
                } else if alx {
                    ly.g = TcpState::Bhe;
                }
            }

            TcpState::Bhe => {
                if iup {
                    ly.alx = ls.cn(1);
                    ly.bqr = true;
                    ly.g = TcpState::Aey;
                    ly.dit = crate::logger::lh();
                    drop(aan);
                    let _ = fue(jh, ey, sa);
                }
            }

            TcpState::Pb => {
                if alx {
                    ly.g = TcpState::Aey;
                    ly.dit = crate::logger::lh(); 
                }
            }

            TcpState::Bkq => {
                if alx {
                    ly.g = TcpState::Dk;
                }
            }

            TcpState::Apz => {
                
                if !ew.is_empty() {
                    ly.alx = ls.cn(bvx);
                    let mut kb = LA_.lock();
                    kb.bt(vg).clq(VecDeque::new).agt(ew.ip());
                }
            }

            _ => {}
        }
    } else {
        
        if jry && !alx {
            
            if aan.len() >= AEX_ {
                let iu = crate::logger::lh();
                let mut kb = LA_.lock();
                aan.ajm(|ad, r| {
                    let eec = match r.g {
                        TcpState::Aey => iu.nj(r.dit) < BHF_,
                        TcpState::Dk => false,
                        _ => true,
                    };
                    if !eec { kb.remove(ad); }
                    eec
                });
            }
            if aan.len() >= AEX_ {
                crate::serial_println!("[TCP] Connection limit reached ({}), dropping SYN", AEX_);
                return;
            }
            let mut glp = Acn.lock();
            if let Some(glo) = glp.ds(&sa) {
                if glo.iis.len() < glo.dea as usize + 16 {
                    let wrs = [
                        ((vg.jh >> 24) & 0xFF) as u8,
                        ((vg.jh >> 16) & 0xFF) as u8,
                        ((vg.jh >> 8) & 0xFF) as u8,
                        (vg.jh & 0xFF) as u8,
                    ];
                    let shb = [
                        ((vg.pz >> 24) & 0xFF) as u8,
                        ((vg.pz >> 16) & 0xFF) as u8,
                        ((vg.pz >> 8) & 0xFF) as u8,
                        (vg.pz & 0xFF) as u8,
                    ];
                    let izx = nxj(wrs, shb, vg.ey, vg.sa);
                    let usp = Azt {
                        g: TcpState::Btu,
                        ls: izx.cn(1),
                        alx: ls.cn(1),
                        bqr: false,
                        fis: false,
                        egi: 0,
                        dit: crate::logger::lh(),
                        jcr: izx,
                        lhv: crate::logger::lh(),
                        lzx: 0,
                        iay: izx.cn(1),
                    };
                    drop(glp);
                    aan.insert(vg, usp);
                    drop(aan);
                    
                    let _ = whp(jh, ey, sa,
                                         ls.cn(1), izx);
                }
            }
        }
    }
}


pub fn cue(kv: [u8; 4], rz: u16) -> Result<u16, &'static str> {
    let jh = cqw();
    
    let ey = VS_.fetch_add(1, Ordering::Relaxed);
    let ls = nxj(jh, kv, ey, rz);

    let mut ie = Vec::fc(20);
    ie.bk(&ey.ft());
    ie.bk(&rz.ft());
    ie.bk(&ls.ft());
    ie.bk(&0u32.ft()); 
    ie.push(0x50); 
    ie.push(flags::Ame);
    ie.bk(&JD_.ft()); 
    ie.bk(&0u16.ft()); 
    ie.bk(&0u16.ft()); 

    let td = ezm(jh, kv, &ie);
    ie[16] = (td >> 8) as u8;
    ie[17] = (td & 0xFF) as u8;
    
    

    let vg = Gt {
        jh: bad(jh),
        pz: bad(kv),
        ey,
        sa: rz,
    };
    Fh.lock().insert(vg, Azt {
        g: TcpState::Btv,
        ls: ls.cn(1),
        alx: 0,
        bqr: false,
        fis: false,
        egi: 0,
        dit: crate::logger::lh(),
        jcr: ls,
        lhv: crate::logger::lh(),
        lzx: 0,
        iay: ls.cn(1),
    });

    

    crate::netstack::ip::blc(kv, 6, &ie)?;
    Ok(ey)
}


fn whp(kv: [u8; 4], rz: u16, ey: u16, gxq: u32, ls: u32) -> Result<(), &'static str> {
    let jh = cqw();

    let mut ie = Vec::fc(20);
    ie.bk(&ey.ft());
    ie.bk(&rz.ft());
    ie.bk(&ls.ft());
    ie.bk(&gxq.ft());
    ie.push(0x50);
    ie.push(flags::Ame | flags::Ie);
    ie.bk(&JD_.ft());
    ie.bk(&0u16.ft());
    ie.bk(&0u16.ft());

    let td = ezm(jh, kv, &ie);
    ie[16] = (td >> 8) as u8;
    ie[17] = (td & 0xFF) as u8;

    crate::netstack::ip::blc(kv, 6, &ie)
}


pub fn fue(kv: [u8; 4], rz: u16, ey: u16) -> Result<(), &'static str> {
    let jh = cqw();
    let vg = Gt {
        jh: bad(jh),
        pz: bad(kv),
        ey,
        sa: rz,
    };

    let (ls, alx) = {
        let aan = Fh.lock();
        let ly = aan.get(&vg).ok_or("Connection not found")?;
        (ly.ls, ly.alx)
    };

    let mut ie = Vec::fc(20);
    ie.bk(&ey.ft());
    ie.bk(&rz.ft());
    ie.bk(&ls.ft());
    ie.bk(&alx.ft());
    ie.push(0x50);
    ie.push(flags::Ie);
    ie.bk(&JD_.ft());
    ie.bk(&0u16.ft());
    ie.bk(&0u16.ft());

    let td = ezm(jh, kv, &ie);
    ie[16] = (td >> 8) as u8;
    ie[17] = (td & 0xFF) as u8;

    crate::netstack::ip::blc(kv, 6, &ie)
}


pub fn dlo(kv: [u8; 4], rz: u16, ey: u16, ew: &[u8]) -> Result<(), &'static str> {
    let jh = cqw();
    let vg = Gt {
        jh: bad(jh),
        pz: bad(kv),
        ey,
        sa: rz,
    };

    let (ls, alx) = {
        let aan = Fh.lock();
        let ly = aan.get(&vg).ok_or("Connection not found")?;
        (ly.ls, ly.alx)
    };

    let mut ie = Vec::fc(20 + ew.len());
    ie.bk(&ey.ft());
    ie.bk(&rz.ft());
    ie.bk(&ls.ft());
    ie.bk(&alx.ft());
    ie.push(0x50);
    ie.push(flags::Awp | flags::Ie);
    ie.bk(&JD_.ft());
    ie.bk(&0u16.ft());
    ie.bk(&0u16.ft());
    ie.bk(ew);

    let td = ezm(jh, kv, &ie);
    ie[16] = (td >> 8) as u8;
    ie[17] = (td & 0xFF) as u8;

    crate::netstack::ip::blc(kv, 6, &ie)?;

    let iu = crate::time::lc();
    let mut aan = Fh.lock();
    if let Some(ly) = aan.ds(&vg) {
        ly.jcr = ls;
        ly.lhv = iu;
        ly.ls = ly.ls.cn(ew.len() as u32);
    }
    drop(aan);

    
    let mut exw = PF_.lock();
    if exw.len() >= CFP_ {
        exw.awp(); 
    }
    exw.agt(Bvg {
        vg,
        ls,
        f: ew.ip(),
        kv,
        rz,
        ey,
        mdv: iu,
        arv: 0,
    });

    Ok(())
}


pub fn bwx(kv: [u8; 4], rz: u16, ey: u16) -> Result<(), &'static str> {
    let jh = cqw();
    let vg = Gt {
        jh: bad(jh),
        pz: bad(kv),
        ey,
        sa: rz,
    };

    let (ls, alx, qhh) = {
        let aan = Fh.lock();
        let ly = aan.get(&vg).ok_or("Connection not found")?;
        (ly.ls, ly.alx, ly.fis)
    };

    if qhh {
        return Ok(());
    }

    let mut ie = Vec::fc(20);
    ie.bk(&ey.ft());
    ie.bk(&rz.ft());
    ie.bk(&ls.ft());
    ie.bk(&alx.ft());
    ie.push(0x50);
    ie.push(flags::Bgl | flags::Ie);
    ie.bk(&JD_.ft());
    ie.bk(&0u16.ft());
    ie.bk(&0u16.ft());

    let td = ezm(jh, kv, &ie);
    ie[16] = (td >> 8) as u8;
    ie[17] = (td & 0xFF) as u8;

    crate::netstack::ip::blc(kv, 6, &ie)?;

    
    PF_.lock().ajm(|pk| pk.vg != vg);

    let mut aan = Fh.lock();
    if let Some(ly) = aan.ds(&vg) {
        ly.ls = ly.ls.cn(1);
        ly.fis = true;
        
        match ly.g {
            TcpState::Pi => ly.g = TcpState::Bhd,
            TcpState::Apz  => ly.g = TcpState::Bkq,
            _ => {}
        }
    }
    Ok(())
}


pub fn fiv(kv: [u8; 4], rz: u16, ey: u16) {
    let jh = cqw();
    let vg = Gt {
        jh: bad(jh),
        pz: bad(kv),
        ey,
        sa: rz,
    };

    let mfo = {
        let mut aan = Fh.lock();
        if let Some(ly) = aan.ds(&vg) {
            if ly.egi > 0 {
                ly.egi = 0;
                ly.dit = crate::logger::lh();
                true
            } else {
                false
            }
        } else {
            false
        }
    };

    if mfo {
        let _ = fue(kv, rz, ey);
    }
}


pub fn cme(kv: [u8; 4], rz: u16, ey: u16) -> Option<Vec<u8>> {
    let jh = cqw();
    let vg = Gt {
        jh: bad(jh),
        pz: bad(kv),
        ey,
        sa: rz,
    };

    {
        let mut kb = LA_.lock();
        if let Some(queue) = kb.ds(&vg) {
            if !queue.is_empty() {
                return queue.awp();
            }
        }
    }

    let mut aan = Fh.lock();
    if let Some(ly) = aan.get(&vg) {
        if ly.bqr && ly.fis {
            aan.remove(&vg);
            LA_.lock().remove(&vg);
        }
    }
    None
}


pub fn bqr(kv: [u8; 4], rz: u16, ey: u16) -> bool {
    let jh = cqw();
    let vg = Gt {
        jh: bad(jh),
        pz: bad(kv),
        ey,
        sa: rz,
    };

    Fh
        .lock()
        .get(&vg)
        .map(|r| r.bqr)
        .unwrap_or(true)
}


pub fn dnd(kv: [u8; 4], rz: u16, ey: u16, sg: u32) -> bool {
    let jh = cqw();
    let vg = Gt {
        jh: bad(jh),
        pz: bad(kv),
        ey,
        sa: rz,
    };

    let ay = crate::logger::lh();
    let mut oig = ay;
    let mut pqn: u8 = 0;
    let mut aaf: u32 = 0;
    
    loop {
        crate::netstack::poll();

        if let Some(ly) = Fh.lock().get(&vg) {
            if ly.g == TcpState::Pi {
                return true;
            }
            
            if ly.g == TcpState::Dk {
                return false;
            }
        }

        let iu = crate::logger::lh();
        
        
        if iu.ao(oig) > BEL_ && pqn < AFE_ {
            pqn += 1;
            oig = iu;
            
            
            
            let ls = {
                let aan = Fh.lock();
                aan.get(&vg).map(|r| r.jcr).unwrap_or(0)
            };
            
            let mut ie = Vec::fc(20);
            ie.bk(&ey.ft());
            ie.bk(&rz.ft());
            ie.bk(&ls.ft());
            ie.bk(&0u32.ft());
            ie.push(0x50);
            ie.push(flags::Ame);
            ie.bk(&JD_.ft());
            ie.bk(&0u16.ft());
            ie.bk(&0u16.ft());
            
            let td = ezm(jh, kv, &ie);
            ie[16] = (td >> 8) as u8;
            ie[17] = (td & 0xFF) as u8;
            
            let _ = crate::netstack::ip::blc(kv, 6, &ie);
        }

        if iu.ao(ay) > sg as u64 {
            return false;
        }
        aaf = aaf.cn(1);
        if aaf > 2_000_000 {
            return false;
        }
        
        crate::thread::cix();
    }
}






pub fn lfz(kv: [u8; 4], rz: u16) -> bool {
    let jh = cqw();
    let aan = Fh.lock();
    
    
    
    for (ad, ly) in aan.iter() {
        if ad.pz == bad(kv) && ad.sa == rz
            && ad.jh == bad(jh)
            && ly.g == TcpState::Pi
        {
            return true;
        }
    }
    false
}


pub fn fuf(kv: [u8; 4], rz: u16, ey: u16, f: &[u8]) -> Result<(), &'static str> {
    if f.is_empty() {
        return Ok(());
    }
    
    
    const Ave: usize = 1400;
    
    const Byd: usize = 4;
    
    let ztj = (f.len() + Ave - 1) / Ave;
    
    for (a, jj) in f.btq(Ave).cf() {
        
        let mut arv = 0u32;
        loop {
            match dlo(kv, rz, ey, jj) {
                Ok(()) => break,
                Err(aa) if arv < 200 => {
                    
                    crate::netstack::poll();
                    crate::thread::cix();
                    arv += 1;
                }
                Err(aa) => return Err(aa),
            }
        }
        
        
        if (a + 1) % Byd == 0 {
            crate::netstack::poll();
            
            crate::thread::cix();
        }
        
    }
    
    Ok(())
}


pub fn pam(kv: [u8; 4], rz: u16, ey: u16) -> Option<alloc::vec::Vec<u8>> {
    cme(kv, rz, ey)
}


pub fn nxy(kv: [u8; 4], rz: u16, ey: u16) -> Option<TcpState> {
    let jh = cqw();
    let vg = Gt {
        jh: bad(jh),
        pz: bad(kv),
        ey,
        sa: rz,
    };
    
    Fh.lock().get(&vg).map(|r| r.g)
}



pub fn qzp() {
    let iu = crate::time::lc();
    let mut exw = PF_.lock();

    for pk in exw.el() {
        if iu.nj(pk.mdv) < BEL_ {
            continue;
        }
        if pk.arv >= AFE_ {
            continue; 
        }

        
        let vg = pk.vg;
        let aan = Fh.lock();
        let wui = aan.get(&vg)
            .map(|r| r.g == TcpState::Pi && r.iay <= pk.ls)
            .unwrap_or(false);
        drop(aan);

        if !wui {
            continue;
        }

        
        let jh = cqw();
        let kv = pk.kv;
        let qeu = {
            let aan = Fh.lock();
            aan.get(&vg).map(|r| r.alx).unwrap_or(0)
        };

        let mut cnb = Vec::fc(20 + pk.f.len());
        cnb.bk(&pk.ey.ft());
        cnb.bk(&pk.rz.ft());
        cnb.bk(&pk.ls.ft());
        cnb.bk(&qeu.ft());
        cnb.push(0x50);
        cnb.push(flags::Awp | flags::Ie);
        cnb.bk(&JD_.ft());
        cnb.bk(&0u16.ft());
        cnb.bk(&0u16.ft());
        cnb.bk(&pk.f);

        let td = ezm(jh, kv, &cnb);
        cnb[16] = (td >> 8) as u8;
        cnb[17] = (td & 0xFF) as u8;

        let _ = crate::netstack::ip::blc(kv, 6, &cnb);
        pk.arv += 1;
        pk.mdv = iu; 
    }

    
    exw.ajm(|pk| pk.arv < AFE_);
}


pub fn yim(kv: [u8; 4], rz: u16, ey: u16) {
    let jh = cqw();
    let vg = Gt {
        jh: bad(jh),
        pz: bad(kv),
        ey,
        sa: rz,
    };
    PF_.lock().ajm(|pk| pk.vg != vg);
}
