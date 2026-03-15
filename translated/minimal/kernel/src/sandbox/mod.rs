




pub mod policy;
pub mod net_proxy;
pub mod fs;
pub mod js_sandbox;
pub mod container;

extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use spin::Mutex;

use crate::security;
use crate::security::{CapabilityId, CapabilityType, CapabilityRights};

use self::policy::{SandboxPolicy, PolicyPreset, PolicyVerdict};
use self::net_proxy::{NetProxy, Fz, ProxyError};
use self::fs::SandboxFs;




#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Ax(pub u64);


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SandboxState {
    
    Cv,
    
    Di,
    
    Ky,
    
    Hh,
}


#[derive(Debug, Clone)]
pub struct ResourceLimits {
    
    pub jfj: usize,
    
    pub uky: usize,
    
    pub gmj: usize,
    
    pub efg: usize,
    
    pub ols: usize,
    
    pub olr: usize,
    
    pub uas: u64,
    
    pub uaq: usize,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            jfj: 4 * 1024 * 1024,       
            uky: 4,
            gmj: 100,
            efg: 1024 * 1024,           
            ols: 64,
            olr: 512 * 1024,                 
            uas: 5000,
            uaq: 64,
        }
    }
}


#[derive(Debug, Clone)]
pub struct Ke {
    pub aet: u64,
    pub afh: Ax,
    pub hr: AuditAction,
    pub eu: String,
}

#[derive(Debug, Clone)]
pub enum AuditAction {
    Cu,
    Chv,
    Dcz,
    Dcy,
    Daj,
    Dai,
    Cxh,
    Dep,
    Cav,
}


#[derive(Debug, Clone)]
pub struct SandboxStats {
    pub fsr: usize,
    pub jmd: usize,
    pub cdm: usize,
    pub ohi: usize,
    pub hvq: usize,
    pub nwo: usize,
    pub nwm: usize,
}

impl SandboxStats {
    fn new() -> Self {
        Self {
            fsr: 0,
            jmd: 0,
            cdm: 0,
            ohi: 0,
            hvq: 0,
            nwo: 0,
            nwm: 0,
        }
    }
}


pub struct Sandbox {
    pub ad: Ax,
    pub g: SandboxState,
    
    pub capability: CapabilityId,
    
    pub policy: SandboxPolicy,
    
    pub net_proxy: NetProxy,
    
    pub fio: SandboxFs,
    
    pub bdv: Option<String>,
    
    pub cfw: ResourceLimits,
    
    pub cm: SandboxStats,
    
    pub jij: BTreeMap<String, Vec<u8>>,
    
    pub cu: String,
}

impl Sandbox {
    fn new(ad: Ax, capability: CapabilityId, policy: SandboxPolicy, cfw: ResourceLimits, cu: String) -> Self {
        let net_proxy = NetProxy::new(ad, &policy);
        let fio = SandboxFs::new(ad, cfw.ols, cfw.olr);
        Self {
            ad,
            g: SandboxState::Cv,
            capability,
            policy,
            net_proxy,
            fio,
            bdv: None,
            cfw,
            cm: SandboxStats::new(),
            jij: BTreeMap::new(),
            cu,
        }
    }
}




pub struct SandboxManager {
    bse: BTreeMap<u64, Sandbox>,
    bcb: u64,
    
    nbr: Option<u32>,
    
    lkk: Option<CapabilityId>,
    
    emi: Vec<Ke>,
}

impl SandboxManager {
    pub fn new() -> Self {
        Self {
            bse: BTreeMap::new(),
            bcb: 1,
            nbr: None,
            lkk: None,
            emi: Vec::new(),
        }
    }

    
    pub fn vub(&mut self) {
        let pws = security::pbm(
            "WebSandbox",
            3, 
            "Security",
            "Web sandbox isolation — controls network, JS, filesystem access for untrusted web content"
        );
        self.nbr = Some(pws);

        
        let jez = security::klu(
            CapabilityType::Ari(pws),
            CapabilityRights::Cm
                .far(CapabilityRights::Db)
                .far(CapabilityRights::Vx)
                .far(CapabilityRights::Mt),
            0, 
        );
        self.lkk = Some(jez);
    }

    
    pub fn nhh(&mut self, akl: PolicyPreset, cu: Option<&str>) -> Ax {
        let ad = Ax(self.bcb);
        self.bcb += 1;

        
        let capability = if let Some(jez) = self.lkk {
            security::derive(
                jez,
                CapabilityRights::Cm.far(CapabilityRights::Db),
                ad.0, 
            ).unwrap_or(jez)
        } else {
            CapabilityId(0)
        };

        let policy = SandboxPolicy::sye(akl);
        let cfw = ResourceLimits::default();
        let fms = cu.unwrap_or("sandbox").into();
        let sandbox = Sandbox::new(ad, capability, policy, cfw, fms);
        self.bse.insert(ad.0, sandbox);

        self.ma(ad, AuditAction::Cu, format!("preset={:?}", akl));
        ad
    }

    
    pub fn bvn(&mut self, ad: Ax, url: &str) -> Result<Fz, SandboxError> {
        
        {
            let sandbox = self.bse.get(&ad.0)
                .ok_or(SandboxError::N)?;
            if sandbox.g == SandboxState::Hh {
                return Err(SandboxError::Hh);
            }
            if sandbox.g == SandboxState::Ky {
                return Err(SandboxError::Ky);
            }
            if sandbox.cm.fsr >= sandbox.cfw.gmj {
                
            }
        }

        
        let sandbox = self.bse.ds(&ad.0)
            .ok_or(SandboxError::N)?;

        if sandbox.cm.fsr >= sandbox.cfw.gmj {
            sandbox.cm.hvq += 1;
            return Err(SandboxError::Axx);
        }

        let igj = sandbox.policy.nrf(url);
        match igj {
            PolicyVerdict::Zs => {},
            PolicyVerdict::Pf(ctt) => {
                sandbox.cm.jmd += 1;
                sandbox.cm.hvq += 1;
                return Err(SandboxError::Adv(ctt));
            },
            PolicyVerdict::Nl => {},
        }

        
        sandbox.cm.fsr += 1;
        let ate = sandbox.cfw.efg;
        let mk = sandbox.net_proxy.hjd(url, ate)?;

        sandbox.cm.cdm += mk.gj.len();
        sandbox.bdv = Some(url.into());
        sandbox.g = SandboxState::Di;
        sandbox.jij.insert(url.into(), mk.gj.clone());

        
        self.ma(ad, AuditAction::Chv, url.into());

        Ok(mk)
    }

    
    pub fn kvn(&mut self, ad: Ax, url: &str) -> Result<Fz, SandboxError> {
        let sandbox = self.bse.ds(&ad.0)
            .ok_or(SandboxError::N)?;

        if sandbox.g == SandboxState::Hh {
            return Err(SandboxError::Hh);
        }

        
        if let Some(ene) = sandbox.jij.get(url) {
            return Ok(Fz {
                wt: 200,
                ahg: String::from("text/html"),
                gj: ene.clone(),
                zk: Vec::new(),
                mqh: true,
            });
        }

        if sandbox.cm.fsr >= sandbox.cfw.gmj {
            sandbox.cm.hvq += 1;
            return Err(SandboxError::Axx);
        }

        let igj = sandbox.policy.nrf(url);
        match igj {
            PolicyVerdict::Zs | PolicyVerdict::Nl => {},
            PolicyVerdict::Pf(ctt) => {
                sandbox.cm.jmd += 1;
                return Err(SandboxError::Adv(ctt));
            },
        }

        sandbox.cm.fsr += 1;
        let ate = sandbox.cfw.efg;
        let mk = sandbox.net_proxy.hjd(url, ate)?;
        sandbox.cm.cdm += mk.gj.len();
        sandbox.jij.insert(url.into(), mk.gj.clone());
        Ok(mk)
    }

    
    pub fn fvw(&mut self, ad: Ax) -> Result<(), SandboxError> {
        let sandbox = self.bse.ds(&ad.0)
            .ok_or(SandboxError::N)?;
        sandbox.g = SandboxState::Ky;
        Ok(())
    }

    
    pub fn anu(&mut self, ad: Ax) -> Result<(), SandboxError> {
        let sandbox = self.bse.ds(&ad.0)
            .ok_or(SandboxError::N)?;
        if sandbox.g == SandboxState::Ky {
            sandbox.g = SandboxState::Di;
        }
        Ok(())
    }

    
    pub fn hfy(&mut self, ad: Ax) -> Result<(), SandboxError> {
        if let Some(sandbox) = self.bse.ds(&ad.0) {
            sandbox.g = SandboxState::Hh;
            security::vyp(sandbox.capability);
            self.ma(ad, AuditAction::Cav, String::new());
        }
        self.bse.remove(&ad.0);
        Ok(())
    }

    
    pub fn get(&self, ad: Ax) -> Option<&Sandbox> {
        self.bse.get(&ad.0)
    }

    
    pub fn ds(&mut self, ad: Ax) -> Option<&mut Sandbox> {
        self.bse.ds(&ad.0)
    }

    
    pub fn aoy(&self) -> Vec<(Ax, &str, SandboxState)> {
        self.bse.alv()
            .map(|e| (e.ad, e.cu.as_str(), e.g))
            .collect()
    }

    
    pub fn az(&self) -> usize {
        self.bse.len()
    }

    
    fn ma(&mut self, ad: Ax, hr: AuditAction, eu: String) {
        let wi = crate::time::lc();
        self.emi.push(Ke {
            aet: wi,
            afh: ad,
            hr,
            eu,
        });
        
        if self.emi.len() > 256 {
            self.emi.remove(0);
        }
    }

    
    pub fn emi(&self) -> &[Ke] {
        &self.emi
    }

    
    pub fn qlf(&self, ad: Ax) -> Vec<&Ke> {
        self.emi.iter().hi(|aa| aa.afh == ad).collect()
    }
}



#[derive(Debug)]
pub enum SandboxError {
    N,
    Hh,
    Ky,
    Adv(String),
    Axx,
    Qd(String),
    Cxj,
    Cxi,
    Dal,
    Dak,
    Csr,
}

impl From<ProxyError> for SandboxError {
    fn from(aa: ProxyError) -> Self {
        match aa {
            ProxyError::Beu(bc) => SandboxError::Adv(bc),
            ProxyError::Bqk => SandboxError::Axx,
            ProxyError::Ckr => SandboxError::Adv(String::from("response too large")),
            ProxyError::Qd(e) => SandboxError::Qd(e),
            ProxyError::Wj => SandboxError::Qd(String::from("DNS resolution failed")),
            ProxyError::TlsError => SandboxError::Qd(String::from("TLS handshake failed")),
            ProxyError::Oi => SandboxError::Qd(String::from("request timeout")),
            ProxyError::Bjv => SandboxError::Adv(String::from("invalid URL")),
        }
    }
}



lazy_static::lazy_static! {
    pub static ref BD_: Mutex<SandboxManager> = Mutex::new(SandboxManager::new());
}


pub fn init() {
    let mut aas = BD_.lock();
    aas.vub();
    crate::serial_println!("[sandbox] Web Sandbox subsystem initialized");
    crate::serial_println!("[sandbox] Capability type registered: WebSandbox (danger=3)");
}


pub fn avp(akl: PolicyPreset, cu: Option<&str>) -> Ax {
    BD_.lock().nhh(akl, cu)
}


pub fn bvn(ad: Ax, url: &str) -> Result<Fz, SandboxError> {
    BD_.lock().bvn(ad, url)
}


pub fn kvn(ad: Ax, url: &str) -> Result<Fz, SandboxError> {
    BD_.lock().kvn(ad, url)
}


pub fn hfy(ad: Ax) -> Result<(), SandboxError> {
    BD_.lock().hfy(ad)
}


pub fn aoy() -> Vec<(Ax, String, SandboxState)> {
    BD_.lock().aoy().dse()
        .map(|(ad, e, apc)| (ad, String::from(e), apc))
        .collect()
}


pub fn ibt(ad: Ax) -> Option<String> {
    let aas = BD_.lock();
    let is = aas.get(ad)?;
    Some(format!(
        "Sandbox #{} '{}'\n  State: {:?}\n  URL: {}\n  Requests: {}/{} ({} blocked)\n  Data: {} bytes\n  JS execs: {}\n  Violations: {}\n  FS: {} files, {} bytes\n  Policy: {:?}",
        is.ad.0, is.cu, is.g,
        is.bdv.ahz().unwrap_or("(none)"),
        is.cm.fsr, is.cfw.gmj, is.cm.jmd,
        is.cm.cdm,
        is.cm.ohi,
        is.cm.hvq,
        is.cm.nwo, is.cm.nwm,
        is.policy.akl,
    ))
}
