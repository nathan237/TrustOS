





























use alloc::vec::Vec;
use alloc::string::String;
use alloc::format;
use core::sync::atomic::{AtomicBool, AtomicU8, AtomicU64, Ordering};
use spin::Mutex;






pub const FA_: u16 = 7700;


pub const HM_: u16 = 7701;


const Dp: &[u8; 4] = b"JMSH";


const AIB_: usize = 41;


const BMR_: u64 = 5000;


const CCN_: u64 = 3000;


const CMF_: u64 = 15000;


const CIW_: usize = 64;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CpuArch {
    X86_64 = 0,
    Aarch64 = 1,
    Riscv64 = 2,
    Unknown = 255,
}

impl CpuArch {
    fn fxt(b: u8) -> Self {
        match b {
            0 => CpuArch::X86_64,
            1 => CpuArch::Aarch64,
            2 => CpuArch::Riscv64,
            _ => CpuArch::Unknown,
        }
    }

    
    pub fn current() -> Self {
        #[cfg(target_arch = "x86_64")]
        { CpuArch::X86_64 }
        #[cfg(target_arch = "aarch64")]
        { CpuArch::Aarch64 }
        #[cfg(target_arch = "riscv64")]
        { CpuArch::Riscv64 }
    }

    pub fn name(&self) -> &'static str {
        match self {
            CpuArch::X86_64 => "x86_64",
            CpuArch::Aarch64 => "aarch64",
            CpuArch::Riscv64 => "riscv64",
            CpuArch::Unknown => "unknown",
        }
    }
}






#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MsgType {
    Announce = 0,
    Heartbeat = 1,
    Leave = 2,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum NodeRole {
    Worker = 0,
    Leader = 1,
    Candidate = 2,
}


#[derive(Debug, Clone)]
pub struct Im {
    
    pub ip: [u8; 4],
    
    pub mac: [u8; 6],
    
    pub role: NodeRole,
    
    pub uptime_ms: u64,
    
    pub param_count: u32,
    
    pub training_steps: u32,
    
    pub cpu_cores: u16,
    
    pub ram_mb: u32,
    
    pub rpc_port: u16,
    
    pub arch: CpuArch,
    
    pub last_seen_ms: u64,
    
    pub alive: bool,
}

impl Im {
    
    pub fn display(&self) -> String {
        let dxy = match self.role {
            NodeRole::Worker => "Worker",
            NodeRole::Leader => "Leader",
            NodeRole::Candidate => "Candidate",
        };
        format!("{}.{}.{}.{}  arch={}  role={}  params={}  steps={}  cores={}  ram={}MB",
            self.ip[0], self.ip[1], self.ip[2], self.ip[3],
            self.arch.name(), dxy, self.param_count, self.training_steps,
            self.cpu_cores, self.ram_mb)
    }
}






static LL_: AtomicBool = AtomicBool::new(false);


static AIA_: AtomicU8 = AtomicU8::new(0); 


static Ej: Mutex<Vec<Im>> = Mutex::new(Vec::new());


static AFT_: AtomicU64 = AtomicU64::new(0);


static AFX_: AtomicU64 = AtomicU64::new(0);


static BEY_: AtomicU64 = AtomicU64::new(0);






pub fn start() {
    if LL_.load(Ordering::SeqCst) {
        crate::serial_println!("[MESH] Already active");
        return;
    }

    if !crate::network::sw() {
        crate::serial_println!("[MESH] Network not available — cannot start mesh");
        return;
    }

    
    crate::network::lqi();

    LL_.store(true, Ordering::SeqCst);
    Ej.lock().clear();
    AFT_.store(0, Ordering::SeqCst);
    AFX_.store(0, Ordering::SeqCst);

    crate::serial_println!("[MESH] JARVIS mesh started on port {}", FA_);

    
    gtt();
}


pub fn stop() {
    if !LL_.load(Ordering::SeqCst) {
        return;
    }

    
    ont();

    LL_.store(false, Ordering::SeqCst);
    Ej.lock().clear();
    crate::serial_println!("[MESH] JARVIS mesh stopped");
}


pub fn is_active() -> bool {
    LL_.load(Ordering::SeqCst)
}


pub fn dwa() -> NodeRole {
    match AIA_.load(Ordering::SeqCst) {
        1 => NodeRole::Leader,
        2 => NodeRole::Candidate,
        _ => NodeRole::Worker,
    }
}


pub fn dei(role: NodeRole) {
    AIA_.store(role as u8, Ordering::SeqCst);
}


pub fn bgo() -> Vec<Im> {
    let lj = Ej.lock();
    lj.iter().filter(|aa| aa.alive).cloned().collect()
}


pub fn ayz() -> usize {
    let lj = Ej.lock();
    lj.iter().filter(|aa| aa.alive).count()
}


pub fn plu() -> u64 {
    BEY_.load(Ordering::SeqCst)
}


pub fn fyt() -> Option<Im> {
    let lj = Ej.lock();
    lj.iter().find(|aa| aa.alive && aa.role == NodeRole::Leader).cloned()
}



pub fn poll() {
    if !LL_.load(Ordering::SeqCst) {
        return;
    }

    let cy = crate::time::uptime_ms();

    
    while let Some(data) = crate::netstack::udp::eyc(FA_) {
        mhw(&data, cy);
    }

    
    let mwh = AFT_.load(Ordering::SeqCst);
    if cy.wrapping_sub(mwh) >= BMR_ {
        gtt();
        AFT_.store(cy, Ordering::SeqCst);
    }

    
    let gey = AFX_.load(Ordering::SeqCst);
    if cy.wrapping_sub(gey) >= CCN_ {
        onq();
        AFX_.store(cy, Ordering::SeqCst);
    }

    
    lsu(cy);
}






fn cun(msg_type: MsgType) -> [u8; AIB_] {
    let mut fj = [0u8; AIB_];

    
    fj[0..4].copy_from_slice(Dp);

    
    fj[4] = msg_type as u8;

    
    if let Some((ip, _, _)) = crate::network::rd() {
        fj[5..9].copy_from_slice(ip.as_bytes());
    }

    
    if let Some(mac) = crate::network::aqu() {
        fj[9..15].copy_from_slice(&mac);
    }

    
    fj[15] = AIA_.load(Ordering::SeqCst);

    
    let aiz = crate::time::uptime_ms();
    fj[16..24].copy_from_slice(&aiz.to_be_bytes());

    
    let params = if super::is_ready() {
        super::Ay.lock().as_ref().map(|m| m.param_count() as u32).unwrap_or(0)
    } else {
        0
    };
    fj[24..28].copy_from_slice(&params.to_be_bytes());

    
    let steps = super::BY_.load(Ordering::SeqCst) as u32;
    fj[28..32].copy_from_slice(&steps.to_be_bytes());

    
    let cores: u16 = {
        #[cfg(target_arch = "x86_64")]
        { crate::cpu::smp::cpu_count() as u16 }
        #[cfg(not(target_arch = "x86_64"))]
        { 1 }
    };
    fj[32..34].copy_from_slice(&cores.to_be_bytes());

    
    let ram_mb: u32 = (crate::memory::ceo() / (1024 * 1024)) as u32;
    fj[34..38].copy_from_slice(&ram_mb.to_be_bytes());

    
    fj[38..40].copy_from_slice(&HM_.to_be_bytes());

    
    fj[40] = CpuArch::current() as u8;

    fj
}


fn nqw(data: &[u8]) -> Option<(MsgType, Im)> {
    if data.len() < AIB_ {
        return None;
    }

    
    if &data[0..4] != Dp {
        return None;
    }

    let msg_type = match data[4] {
        0 => MsgType::Announce,
        1 => MsgType::Heartbeat,
        2 => MsgType::Leave,
        _ => return None,
    };

    let mut ip = [0u8; 4];
    ip.copy_from_slice(&data[5..9]);

    let mut mac = [0u8; 6];
    mac.copy_from_slice(&data[9..15]);

    let role = match data[15] {
        1 => NodeRole::Leader,
        2 => NodeRole::Candidate,
        _ => NodeRole::Worker,
    };

    let aiz = u64::from_be_bytes([
        data[16], data[17], data[18], data[19],
        data[20], data[21], data[22], data[23],
    ]);

    let param_count = u32::from_be_bytes([data[24], data[25], data[26], data[27]]);
    let training_steps = u32::from_be_bytes([data[28], data[29], data[30], data[31]]);
    let cpu_cores = u16::from_be_bytes([data[32], data[33]]);
    let ram_mb = u32::from_be_bytes([data[34], data[35], data[36], data[37]]);
    let rpc_port = u16::from_be_bytes([data[38], data[39]]);

    
    let arch = if data.len() > 40 {
        CpuArch::fxt(data[40])
    } else {
        CpuArch::Unknown
    };

    Some((msg_type, Im {
        ip,
        mac,
        role,
        uptime_ms: aiz,
        param_count,
        training_steps,
        cpu_cores,
        ram_mb,
        rpc_port,
        arch,
        last_seen_ms: 0, 
        alive: true,
    }))
}






fn gtt() {
    let fj = cun(MsgType::Announce);
    let fju = ibi();
    let src_port = FA_;

    if let Err(e) = crate::netstack::udp::azq(fju, FA_, src_port, &fj) {
        crate::serial_println!("[MESH] Announce send failed: {}", e);
    }
}


fn onq() {
    let fj = cun(MsgType::Heartbeat);
    let lj = Ej.lock();

    for peer in lj.iter().filter(|aa| aa.alive) {
        let _ = crate::netstack::udp::azq(peer.ip, FA_, FA_, &fj);
    }
}


fn ont() {
    let fj = cun(MsgType::Leave);
    let fju = ibi();
    let _ = crate::netstack::udp::azq(fju, FA_, FA_, &fj);

    let lj = Ej.lock();
    for peer in lj.iter().filter(|aa| aa.alive) {
        let _ = crate::netstack::udp::azq(peer.ip, FA_, FA_, &fj);
    }
}






fn mhw(data: &[u8], bih: u64) {
    let (msg_type, mut peer_info) = match nqw(data) {
        Some(v) => v,
        None => return,
    };

    
    if mti(peer_info.ip) {
        return;
    }

    peer_info.last_seen_ms = bih;

    match msg_type {
        MsgType::Announce | MsgType::Heartbeat => {
            let mtf = !ntm(peer_info.ip);
            ppx(peer_info);
            
            if mtf && msg_type == MsgType::Announce {
                gtt();
            }
        }
        MsgType::Leave => {
            ofa(peer_info.ip);
        }
    }
}


fn ntm(ip: [u8; 4]) -> bool {
    let lj = Ej.lock();
    lj.iter().any(|aa| aa.ip == ip)
}


fn ppx(info: Im) {
    let mut lj = Ej.lock();

    
    for peer in lj.iter_mut() {
        if peer.ip == info.ip {
            peer.role = info.role;
            peer.uptime_ms = info.uptime_ms;
            peer.param_count = info.param_count;
            peer.training_steps = info.training_steps;
            peer.cpu_cores = info.cpu_cores;
            peer.ram_mb = info.ram_mb;
            peer.rpc_port = info.rpc_port;
            peer.arch = info.arch;
            peer.last_seen_ms = info.last_seen_ms;
            peer.alive = true;
            return;
        }
    }

    
    if lj.len() < CIW_ {
        crate::serial_println!("[MESH] New peer discovered: {}.{}.{}.{} (arch={}, role={:?}, params={})",
            info.ip[0], info.ip[1], info.ip[2], info.ip[3],
            info.arch.name(), info.role, info.param_count);
        BEY_.fetch_add(1, Ordering::SeqCst);
        lj.push(info);
    }
}


fn ofa(ip: [u8; 4]) {
    let mut lj = Ej.lock();
    if let Some(peer) = lj.iter_mut().find(|aa| aa.ip == ip) {
        crate::serial_println!("[MESH] Peer left: {}.{}.{}.{}",
            ip[0], ip[1], ip[2], ip[3]);
        peer.alive = false;
    }
}


fn lsu(bih: u64) {
    let mut lj = Ej.lock();
    for peer in lj.iter_mut().filter(|aa| aa.alive) {
        if bih.wrapping_sub(peer.last_seen_ms) > CMF_ {
            crate::serial_println!("[MESH] Peer timed out: {}.{}.{}.{}",
                peer.ip[0], peer.ip[1], peer.ip[2], peer.ip[3]);
            peer.alive = false;
        }
    }
}






fn ibi() -> [u8; 4] {
    if let Some((ip, subnet, _)) = crate::network::rd() {
        let czv = ip.as_bytes();
        let fbd = subnet.as_bytes();
        [
            czv[0] | !fbd[0],
            czv[1] | !fbd[1],
            czv[2] | !fbd[2],
            czv[3] | !fbd[3],
        ]
    } else {
        [255, 255, 255, 255] 
    }
}


fn mti(ip: [u8; 4]) -> bool {
    if let Some((wj, _, _)) = crate::network::rd() {
        *wj.as_bytes() == ip
    } else {
        false
    }
}


pub fn jis() -> String {
    let alive = ayz();
    let role = dwa();
    let dxy = match role {
        NodeRole::Leader => "Leader",
        NodeRole::Worker => "Worker",
        NodeRole::Candidate => "Candidate",
    };

    if !is_active() {
        return String::from("Mesh: inactive");
    }

    format!("Mesh: {} | Role: {} | Peers: {} alive | Discovered: {}",
        if is_active() { "active" } else { "inactive" },
        dxy,
        alive,
        plu())
}
