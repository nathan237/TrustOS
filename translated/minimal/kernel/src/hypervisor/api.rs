







use alloc::string::String;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use spin::Mutex;
use core::sync::atomic::{AtomicU64, Ordering};






#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u64)]
pub enum Capability {
    
    Cpu = 1 << 0,
    
    Cbr = 1 << 1,
    
    Cpw = 1 << 2,
    
    Cox = 1 << 3,
    
    Dlp = 1 << 4,
    
    Der = 1 << 5,
    
    Cvx = 1 << 6,
    
    Cpm = 1 << 7,
    
    Cmw = 1 << 8,
    
    Dcx = 1 << 9,
    
    Dbj = 1 << 10,
    
    Dcl = 1 << 11,
    
    Cui = 1 << 12,
}


#[derive(Debug, Clone, Copy, Default)]
pub struct Capabilities {
    fs: u64,
}

impl Capabilities {
    pub fn new() -> Self {
        Capabilities { fs: 0 }
    }
    
    pub fn oj(&mut self, mh: Capability) {
        self.fs |= mh as u64;
    }
    
    pub fn ywe(&self, mh: Capability) -> bool {
        (self.fs & (mh as u64)) != 0
    }
    
    pub fn cvr(&self) -> u64 {
        self.fs
    }
    
    pub fn yru(fs: u64) -> Self {
        Capabilities { fs }
    }
}


pub fn iwn() -> Capabilities {
    let mut dr = Capabilities::new();
    
    
    if let Ok(jvy) = super::vmx::inj() {
        if jvy.dme {
            dr.oj(Capability::Cpu);
        }
        if jvy.fhw {
            dr.oj(Capability::Cbr);
        }
        if jvy.gwj {
            dr.oj(Capability::Cpw);
        }
        if jvy.gvo {
            dr.oj(Capability::Cox);
        }
    }
    
    
    dr.oj(Capability::Cpm);
    dr.oj(Capability::Cmw);
    
    dr
}






#[derive(Debug, Clone)]
pub struct Dlo {
    pub ad: u64,
    pub j: String,
    pub g: Cpo,
    pub afc: usize,
    pub jvj: usize,
    pub lc: u64,
    pub cm: Cpp,
    pub isolation: Cgf,
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Cpo {
    Cu,
    Czs,
    Ai,
    Cl,
    Diz,
    Af,
    Gu,
    Dcp,
}


#[derive(Debug, Clone, Default)]
pub struct Cpp {
    pub dcw: u64,
    pub bmp: u64,
    pub ank: u64,
    pub bkn: u64,
    pub axz: u64,
    pub fhx: u64,
    pub yxd: u64,
    pub yyk: u64,
    pub ykf: u64,
}


#[derive(Debug, Clone)]
pub struct Cgf {
    pub vpid: Option<u16>,
    pub kty: bool,
    pub ypo: usize,
    pub yps: bool,
    pub zpy: bool,
}






#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u32)]
pub enum VmEventType {
    
    Cu = 0,
    
    Diu = 1,
    
    Cl = 2,
    
    Dgg = 3,
    
    Af = 4,
    
    Gu = 5,
    
    Ctm = 6,
    
    Acd = 7,
    
    Lj = 8,
    
    Dcn = 9,
    
    Dlv = 10,
}


#[derive(Debug, Clone)]
pub struct Uw {
    pub bqo: VmEventType,
    pub fk: u64,
    pub aet: u64,
    pub f: VmEventData,
}


#[derive(Debug, Clone)]
pub enum VmEventData {
    None,
    Cck(i32),
    Cj(String),
    Bxs(u64),
    Cfe { gw: u64, result: i64 },
}


pub type Bgd = fn(id: &Uw);


struct Bgf {
    fed: Bgd,
    mpo: Option<u64>, 
    kuf: Option<VmEventType>, 
}

static ARK_: Mutex<Vec<Bgf>> = Mutex::new(Vec::new());
static ARJ_: Mutex<Vec<Uw>> = Mutex::new(Vec::new());
static BUN_: AtomicU64 = AtomicU64::new(0);


pub fn zpu(
    fed: Bgd,
    mpo: Option<u64>,
    kuf: Option<VmEventType>,
) -> u64 {
    let sub = Bgf {
        fed,
        mpo,
        kuf,
    };
    
    let mut mia = ARK_.lock();
    mia.push(sub);
    BUN_.fetch_add(1, Ordering::SeqCst)
}


pub fn eps(bqo: VmEventType, fk: u64, f: VmEventData) {
    let id = Uw {
        bqo,
        fk,
        aet: crate::time::lc(),
        f,
    };
    
    
    {
        let mut log = ARJ_.lock();
        if log.len() >= 1000 {
            log.remove(0); 
        }
        log.push(id.clone());
    }
    
    
    let mia = ARK_.lock();
    for sub in mia.iter() {
        
        if let Some(ssq) = sub.mpo {
            if ssq != fk {
                continue;
            }
        }
        
        
        if let Some(kwc) = sub.kuf {
            if kwc != bqo {
                continue;
            }
        }
        
        
        (sub.fed)(&id);
    }
}


pub fn ten(az: usize) -> Vec<Uw> {
    let log = ARJ_.lock();
    let ay = if log.len() > az { log.len() - az } else { 0 };
    log[ay..].ip()
}






#[derive(Debug)]
pub struct VmChannel {
    ad: u64,
    fk: u64,
    j: String,
    fao: Vec<u8>,
    ehx: Vec<u8>,
    ate: usize,
}

impl VmChannel {
    pub fn new(ad: u64, fk: u64, j: &str, ate: usize) -> Self {
        VmChannel {
            ad,
            fk,
            j: String::from(j),
            fao: Vec::fc(ate),
            ehx: Vec::fc(ate),
            ate,
        }
    }
    
    
    pub fn baq(&mut self, f: &[u8]) -> Result<usize, &'static str> {
        let bfz = self.ate - self.fao.len();
        let mll = f.len().v(bfz);
        
        if mll == 0 {
            return Err("Channel buffer full");
        }
        
        self.fao.bk(&f[..mll]);
        Ok(mll)
    }
    
    
    pub fn ehf(&mut self, k: &mut [u8]) -> usize {
        let ajp = k.len().v(self.ehx.len());
        k[..ajp].dg(&self.ehx[..ajp]);
        self.ehx.bbk(..ajp);
        ajp
    }
    
    
    pub fn bfz(&self) -> usize {
        self.ehx.len()
    }
    
    
    pub fn atm(&self) -> usize {
        self.ate - self.fao.len()
    }
}


static Dv: Mutex<BTreeMap<u64, VmChannel>> = Mutex::new(BTreeMap::new());
static BMH_: AtomicU64 = AtomicU64::new(0);


pub fn ipp(fk: u64, j: &str, ate: usize) -> u64 {
    let ad = BMH_.fetch_add(1, Ordering::SeqCst);
    let channel = VmChannel::new(ad, fk, j, ate);
    
    Dv.lock().insert(ad, channel);
    
    crate::serial_println!("[API] Created channel {} for VM {}: {}", ad, fk, j);
    ad
}


pub fn nci(cjo: u64, f: &[u8]) -> Result<usize, &'static str> {
    let mut lq = Dv.lock();
    match lq.ds(&cjo) {
        Some(channel) => channel.baq(f),
        None => Err("Channel not found"),
    }
}


pub fn yhu(cjo: u64, k: &mut [u8]) -> Result<usize, &'static str> {
    let mut lq = Dv.lock();
    match lq.ds(&cjo) {
        Some(channel) => Ok(channel.ehf(k)),
        None => Err("Channel not found"),
    }
}






#[derive(Debug, Clone)]
pub struct ResourceQuota {
    
    pub lla: usize,
    
    pub llh: usize,
    
    pub ulh: usize,
    
    pub ulg: usize,
    
    pub rpr: u8,
}

impl Default for ResourceQuota {
    fn default() -> Self {
        ResourceQuota {
            lla: 256 * 1024 * 1024, 
            llh: 4,
            ulh: 0,
            ulg: 10000,
            rpr: 0,
        }
    }
}

static BIQ_: Mutex<BTreeMap<u64, ResourceQuota>> = Mutex::new(BTreeMap::new());


pub fn wjm(fk: u64, lws: ResourceQuota) {
    BIQ_.lock().insert(fk, lws);
}


pub fn ytp(fk: u64) -> Option<ResourceQuota> {
    BIQ_.lock().get(&fk).abn()
}






pub mod hypercall {
    
    pub const Dec: u64 = 0x00;
    pub const Ahp: u64 = 0x01;
    pub const DNA_: u64 = 0x02;
    pub const DGO_: u64 = 0x03;
    pub const DGN_: u64 = 0x04;
    
    
    pub const EJV_: u64 = 0x100;
    
    
    pub const BWS_: u64 = 0x200;
    pub const BXC_: u64 = 0x201;
    pub const BXA_: u64 = 0x202;
    pub const BMI_: u64 = 0x210;
    pub const BMG_: u64 = 0x211;
    pub const DEP_: u64 = 0x212;
    pub const DEO_: u64 = 0x213;
    pub const BMJ_: u64 = 0x214;
    pub const EFZ_: u64 = 0x220;
    pub const ECM_: u64 = 0x221;
    pub const Cqt: u64 = 0x222;
    pub const Uf: u64 = 0x223;
    pub const Axh: u64 = 0x224;
    
    
    pub const DIW_: u64 = 0x300;
    pub const BRB_: u64 = 0x301;
    pub const DIX_: u64 = 0x302;
}


pub fn tix(fk: u64, gw: u64, n: &[u64; 4]) -> (i64, u64) {
    use hypercall::*;
    
    match gw {
        BWS_ => {
            let dr = iwn();
            (0, dr.cvr())
        }
        
        BXC_ => {
            
            
            (0, fk)
        }
        
        BXA_ => {
            
            (0, 0)
        }
        
        BMI_ => {
            
            
            let cjo = ipp(fk, "guest_channel", 4096);
            (0, cjo)
        }
        
        BMG_ => {
            let cjo = n[0];
            let mut lq = Dv.lock();
            if lq.remove(&cjo).is_some() {
                (0, 0)
            } else {
                (-1, 0)
            }
        }
        
        BMJ_ => {
            let cjo = n[0];
            let lq = Dv.lock();
            match lq.get(&cjo) {
                Some(bm) => (0, bm.bfz() as u64),
                None => (-1, 0),
            }
        }
        
        Cqt => {
            
            
            (0, 0)
        }
        
        Uf => {
            eps(VmEventType::Af, fk, VmEventData::Cck(n[0] as i32));
            (-1, 0) 
        }
        
        Axh => {
            eps(VmEventType::Af, fk, VmEventData::Cj(String::from("reboot")));
            (-2, 0) 
        }
        
        BRB_ => {
            crate::serial_println!("[VM {} DEBUG] 0x{:X}", fk, n[0]);
            (0, 0)
        }
        
        _ => {
            crate::serial_println!("[API] Unknown hypercall 0x{:X} from VM {}", gw, fk);
            (-1, 0)
        }
    }
}






pub const DAI_: u32 = 1;
pub const DAJ_: u32 = 0;
pub const DAK_: u32 = 0;
pub const DAL_: &str = "1.0.0";
pub const DEB_: &str = "2026-01-31";


pub fn yuf() -> u64 {
    ((DAI_ as u64) << 32) | ((DAJ_ as u64) << 16) | (DAK_ as u64)
}
