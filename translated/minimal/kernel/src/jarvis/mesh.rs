





























use alloc::vec::Vec;
use alloc::string::String;
use alloc::format;
use core::sync::atomic::{AtomicBool, AtomicU8, AtomicU64, Ordering};
use spin::Mutex;






pub const EM_: u16 = 7700;


pub const GV_: u16 = 7701;


const Iv: &[u8; 4] = b"JMSH";


const AGH_: usize = 41;


const BKH_: u64 = 5000;


const BZC_: u64 = 3000;


const CIW_: u64 = 15000;


const CFN_: usize = 64;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CpuArch {
    BT_ = 0,
    Fg = 1,
    Jy = 2,
    F = 255,
}

impl CpuArch {
    fn kxf(o: u8) -> Self {
        match o {
            0 => CpuArch::BT_,
            1 => CpuArch::Fg,
            2 => CpuArch::Jy,
            _ => CpuArch::F,
        }
    }

    
    pub fn cv() -> Self {
        #[cfg(target_arch = "x86_64")]
        { CpuArch::BT_ }
        #[cfg(target_arch = "aarch64")]
        { CpuArch::Fg }
        #[cfg(target_arch = "riscv64")]
        { CpuArch::Jy }
    }

    pub fn j(&self) -> &'static str {
        match self {
            CpuArch::BT_ => "x86_64",
            CpuArch::Fg => "aarch64",
            CpuArch::Jy => "riscv64",
            CpuArch::F => "unknown",
        }
    }
}






#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MsgType {
    Agi = 0,
    Atq = 1,
    Tf = 2,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum NodeRole {
    Lb = 0,
    Ni = 1,
    Mu = 2,
}


#[derive(Debug, Clone)]
pub struct Tw {
    
    pub ip: [u8; 4],
    
    pub ed: [u8; 6],
    
    pub bwt: NodeRole,
    
    pub lc: u64,
    
    pub vm: u32,
    
    pub fae: u32,
    
    pub azj: u16,
    
    pub amo: u32,
    
    pub bsb: u16,
    
    pub arch: CpuArch,
    
    pub hpt: u64,
    
    pub bje: bool,
}

impl Tw {
    
    pub fn display(&self) -> String {
        let hxw = match self.bwt {
            NodeRole::Lb => "Worker",
            NodeRole::Ni => "Leader",
            NodeRole::Mu => "Candidate",
        };
        format!("{}.{}.{}.{}  arch={}  role={}  params={}  steps={}  cores={}  ram={}MB",
            self.ip[0], self.ip[1], self.ip[2], self.ip[3],
            self.arch.j(), hxw, self.vm, self.fae,
            self.azj, self.amo)
    }
}






static KS_: AtomicBool = AtomicBool::new(false);


static AGG_: AtomicU8 = AtomicU8::new(0); 


static Ku: Mutex<Vec<Tw>> = Mutex::new(Vec::new());


static ADZ_: AtomicU64 = AtomicU64::new(0);


static AED_: AtomicU64 = AtomicU64::new(0);


static BCV_: AtomicU64 = AtomicU64::new(0);






pub fn ay() {
    if KS_.load(Ordering::SeqCst) {
        crate::serial_println!("[MESH] Already active");
        return;
    }

    if !crate::network::anl() {
        crate::serial_println!("[MESH] Network not available — cannot start mesh");
        return;
    }

    
    crate::network::slt();

    KS_.store(true, Ordering::SeqCst);
    Ku.lock().clear();
    ADZ_.store(0, Ordering::SeqCst);
    AED_.store(0, Ordering::SeqCst);

    crate::serial_println!("[MESH] JARVIS mesh started on port {}", EM_);

    
    mdo();
}


pub fn qg() {
    if !KS_.load(Ordering::SeqCst) {
        return;
    }

    
    whh();

    KS_.store(false, Ordering::SeqCst);
    Ku.lock().clear();
    crate::serial_println!("[MESH] JARVIS mesh stopped");
}


pub fn rl() -> bool {
    KS_.load(Ordering::SeqCst)
}


pub fn htw() -> NodeRole {
    match AGG_.load(Ordering::SeqCst) {
        1 => NodeRole::Ni,
        2 => NodeRole::Mu,
        _ => NodeRole::Lb,
    }
}


pub fn gsf(bwt: NodeRole) {
    AGG_.store(bwt as u8, Ordering::SeqCst);
}


pub fn dhn() -> Vec<Tw> {
    let yp = Ku.lock();
    yp.iter().hi(|ai| ai.bje).abn().collect()
}


pub fn cti() -> usize {
    let yp = Ku.lock();
    yp.iter().hi(|ai| ai.bje).az()
}


pub fn xkb() -> u64 {
    BCV_.load(Ordering::SeqCst)
}


pub fn kyu() -> Option<Tw> {
    let yp = Ku.lock();
    yp.iter().du(|ai| ai.bje && ai.bwt == NodeRole::Ni).abn()
}



pub fn poll() {
    if !KS_.load(Ordering::SeqCst) {
        return;
    }

    let iu = crate::time::lc();

    
    while let Some(f) = crate::netstack::udp::jlt(EM_) {
        tjw(&f, iu);
    }

    
    let ubz = ADZ_.load(Ordering::SeqCst);
    if iu.nj(ubz) >= BKH_ {
        mdo();
        ADZ_.store(iu, Ordering::SeqCst);
    }

    
    let lht = AED_.load(Ordering::SeqCst);
    if iu.nj(lht) >= BZC_ {
        whe();
        AED_.store(iu, Ordering::SeqCst);
    }

    
    spj(iu);
}






fn gbt(msg_type: MsgType) -> [u8; AGH_] {
    let mut mt = [0u8; AGH_];

    
    mt[0..4].dg(Iv);

    
    mt[4] = msg_type as u8;

    
    if let Some((ip, _, _)) = crate::network::aou() {
        mt[5..9].dg(ip.as_bytes());
    }

    
    if let Some(ed) = crate::network::ckt() {
        mt[9..15].dg(&ed);
    }

    
    mt[15] = AGG_.load(Ordering::SeqCst);

    
    let bxp = crate::time::lc();
    mt[16..24].dg(&bxp.ft());

    
    let oi = if super::uc() {
        super::Ci.lock().as_ref().map(|ef| ef.vm() as u32).unwrap_or(0)
    } else {
        0
    };
    mt[24..28].dg(&oi.ft());

    
    let au = super::BW_.load(Ordering::SeqCst) as u32;
    mt[28..32].dg(&au.ft());

    
    let ffw: u16 = {
        #[cfg(target_arch = "x86_64")]
        { crate::cpu::smp::aao() as u16 }
        #[cfg(not(target_arch = "x86_64"))]
        { 1 }
    };
    mt[32..34].dg(&ffw.ft());

    
    let amo: u32 = (crate::memory::fxc() / (1024 * 1024)) as u32;
    mt[34..38].dg(&amo.ft());

    
    mt[38..40].dg(&GV_.ft());

    
    mt[40] = CpuArch::cv() as u8;

    mt
}


fn vdd(f: &[u8]) -> Option<(MsgType, Tw)> {
    if f.len() < AGH_ {
        return None;
    }

    
    if &f[0..4] != Iv {
        return None;
    }

    let msg_type = match f[4] {
        0 => MsgType::Agi,
        1 => MsgType::Atq,
        2 => MsgType::Tf,
        _ => return None,
    };

    let mut ip = [0u8; 4];
    ip.dg(&f[5..9]);

    let mut ed = [0u8; 6];
    ed.dg(&f[9..15]);

    let bwt = match f[15] {
        1 => NodeRole::Ni,
        2 => NodeRole::Mu,
        _ => NodeRole::Lb,
    };

    let bxp = u64::oa([
        f[16], f[17], f[18], f[19],
        f[20], f[21], f[22], f[23],
    ]);

    let vm = u32::oa([f[24], f[25], f[26], f[27]]);
    let fae = u32::oa([f[28], f[29], f[30], f[31]]);
    let azj = u16::oa([f[32], f[33]]);
    let amo = u32::oa([f[34], f[35], f[36], f[37]]);
    let bsb = u16::oa([f[38], f[39]]);

    
    let arch = if f.len() > 40 {
        CpuArch::kxf(f[40])
    } else {
        CpuArch::F
    };

    Some((msg_type, Tw {
        ip,
        ed,
        bwt,
        lc: bxp,
        vm,
        fae,
        azj,
        amo,
        bsb,
        arch,
        hpt: 0, 
        bje: true,
    }))
}






fn mdo() {
    let mt = gbt(MsgType::Agi);
    let kew = nxs();
    let ey = EM_;

    if let Err(aa) = crate::netstack::udp::dlp(kew, EM_, ey, &mt) {
        crate::serial_println!("[MESH] Announce send failed: {}", aa);
    }
}


fn whe() {
    let mt = gbt(MsgType::Atq);
    let yp = Ku.lock();

    for ko in yp.iter().hi(|ai| ai.bje) {
        let _ = crate::netstack::udp::dlp(ko.ip, EM_, EM_, &mt);
    }
}


fn whh() {
    let mt = gbt(MsgType::Tf);
    let kew = nxs();
    let _ = crate::netstack::udp::dlp(kew, EM_, EM_, &mt);

    let yp = Ku.lock();
    for ko in yp.iter().hi(|ai| ai.bje) {
        let _ = crate::netstack::udp::dlp(ko.ip, EM_, EM_, &mt);
    }
}






fn tjw(f: &[u8], efu: u64) {
    let (msg_type, mut huy) = match vdd(f) {
        Some(p) => p,
        None => return,
    };

    
    if tyl(huy.ip) {
        return;
    }

    huy.hpt = efu;

    match msg_type {
        MsgType::Agi | MsgType::Atq => {
            let tyi = !vgm(huy.ip);
            xoz(huy);
            
            if tyi && msg_type == MsgType::Agi {
                mdo();
            }
        }
        MsgType::Tf => {
            vuy(huy.ip);
        }
    }
}


fn vgm(ip: [u8; 4]) -> bool {
    let yp = Ku.lock();
    yp.iter().any(|ai| ai.ip == ip)
}


fn xoz(co: Tw) {
    let mut yp = Ku.lock();

    
    for ko in yp.el() {
        if ko.ip == co.ip {
            ko.bwt = co.bwt;
            ko.lc = co.lc;
            ko.vm = co.vm;
            ko.fae = co.fae;
            ko.azj = co.azj;
            ko.amo = co.amo;
            ko.bsb = co.bsb;
            ko.arch = co.arch;
            ko.hpt = co.hpt;
            ko.bje = true;
            return;
        }
    }

    
    if yp.len() < CFN_ {
        crate::serial_println!("[MESH] New peer discovered: {}.{}.{}.{} (arch={}, role={:?}, params={})",
            co.ip[0], co.ip[1], co.ip[2], co.ip[3],
            co.arch.j(), co.bwt, co.vm);
        BCV_.fetch_add(1, Ordering::SeqCst);
        yp.push(co);
    }
}


fn vuy(ip: [u8; 4]) {
    let mut yp = Ku.lock();
    if let Some(ko) = yp.el().du(|ai| ai.ip == ip) {
        crate::serial_println!("[MESH] Peer left: {}.{}.{}.{}",
            ip[0], ip[1], ip[2], ip[3]);
        ko.bje = false;
    }
}


fn spj(efu: u64) {
    let mut yp = Ku.lock();
    for ko in yp.el().hi(|ai| ai.bje) {
        if efu.nj(ko.hpt) > CIW_ {
            crate::serial_println!("[MESH] Peer timed out: {}.{}.{}.{}",
                ko.ip[0], ko.ip[1], ko.ip[2], ko.ip[3]);
            ko.bje = false;
        }
    }
}






fn nxs() -> [u8; 4] {
    if let Some((ip, up, _)) = crate::network::aou() {
        let gkc = ip.as_bytes();
        let jqp = up.as_bytes();
        [
            gkc[0] | !jqp[0],
            gkc[1] | !jqp[1],
            gkc[2] | !jqp[2],
            gkc[3] | !jqp[3],
        ]
    } else {
        [255, 255, 255, 255] 
    }
}


fn tyl(ip: [u8; 4]) -> bool {
    if let Some((aro, _, _)) = crate::network::aou() {
        *aro.as_bytes() == ip
    } else {
        false
    }
}


pub fn poq() -> String {
    let bje = cti();
    let bwt = htw();
    let hxw = match bwt {
        NodeRole::Ni => "Leader",
        NodeRole::Lb => "Worker",
        NodeRole::Mu => "Candidate",
    };

    if !rl() {
        return String::from("Mesh: inactive");
    }

    format!("Mesh: {} | Role: {} | Peers: {} alive | Discovered: {}",
        if rl() { "active" } else { "inactive" },
        hxw,
        bje,
        xkb())
}
