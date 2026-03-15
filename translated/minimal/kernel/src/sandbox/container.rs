





























extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use alloc::collections::BTreeMap;
use spin::Mutex;

use super::{Ax, SandboxState, SandboxError, BD_};
use super::policy::PolicyPreset;
use super::net_proxy::Fz;




#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IsolationMode {
    
    
    Amu,
    
    
    
    Cew,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthStatus {
    
    Atp,
    
    Bej,
    
    Aj,
    
    Af,
}


#[derive(Debug, Clone)]
pub struct ContainerConfig {
    
    pub j: String,
    
    pub akl: PolicyPreset,
    
    pub isolation: IsolationMode,
    
    pub fob: usize,
    
    pub fsq: usize,
    
    pub efg: usize,
    
    pub hwq: u32,
    
    pub gyk: Vec<String>,
    
    pub hav: Vec<String>,
    
    pub gzi: bool,
    
    pub fyo: u64,
    
    pub gyj: bool,
}

impl Default for ContainerConfig {
    fn default() -> Self {
        Self {
            j: String::from("web-container"),
            akl: PolicyPreset::Ade,
            isolation: IsolationMode::Amu,
            fob: 4 * 1024 * 1024,     
            fsq: 200,
            efg: 1024 * 1024,     
            hwq: 60,
            gyk: Vec::new(),
            hav: Vec::new(),
            gzi: true,
            fyo: 30,
            gyj: false, 
        }
    }
}

impl ContainerConfig {
    
    pub fn hzi() -> Self {
        Self {
            j: String::from("web-secure"),
            akl: PolicyPreset::Aet,
            isolation: IsolationMode::Amu,
            fob: 2 * 1024 * 1024,
            fsq: 50,
            efg: 512 * 1024,
            hwq: 20,
            gyk: Vec::new(),
            hav: Vec::new(),
            gzi: true,
            fyo: 15,
            gyj: false,
        }
    }

    
    pub fn ba() -> Self {
        Self {
            j: String::from("web-dev"),
            akl: PolicyPreset::Ads,
            isolation: IsolationMode::Amu,
            fob: 8 * 1024 * 1024,
            fsq: 500,
            efg: 4 * 1024 * 1024,
            hwq: 120,
            gyk: Vec::new(),
            hav: Vec::new(),
            gzi: false,
            fyo: 0,
            gyj: true,
        }
    }
}




pub struct WebContainer {
    
    pub ad: u32,
    
    pub config: ContainerConfig,
    
    pub afh: Option<Ax>,
    
    pub arh: HealthStatus,
    
    pub jrq: u64,
    
    pub hpq: u64,
    
    pub pzd: u64,
    
    pub lzv: u32,
    
    pub adv: Vec<String>,
    
    pub rzs: Vec<(String, bool)>,
    
    pub xv: usize,
    
    pub dxa: usize,
    
    pub cnt: usize,
}

impl WebContainer {
    fn new(ad: u32, config: ContainerConfig) -> Self {
        let iu = crate::time::lc();
        Self {
            ad,
            config,
            afh: None,
            arh: HealthStatus::Af,
            jrq: iu,
            hpq: iu,
            pzd: iu,
            lzv: 0,
            adv: Vec::new(),
            rzs: Vec::new(),
            xv: 0,
            dxa: 0,
            cnt: 0,
        }
    }

    
    pub fn ay(&mut self) -> Result<(), SandboxError> {
        if self.afh.is_some() {
            return Ok(()); 
        }

        
        let cu = format!("container:{}", self.config.j);
        let afh = {
            let mut aas = BD_.lock();
            let ary = aas.nhh(self.config.akl, Some(&cu));

            
            if let Some(is) = aas.ds(ary) {
                is.cfw.jfj = self.config.fob;
                is.cfw.gmj = self.config.fsq;
                is.cfw.efg = self.config.efg;

                
                for vh in &self.config.gyk {
                    is.policy.kaf(vh);
                }
                for vh in &self.config.hav {
                    is.policy.kpc(vh);
                }

                
                is.policy.gbf = !self.config.gyj;
                is.policy.gqf = self.config.hwq;
            }

            ary
        };

        self.afh = Some(afh);
        self.arh = HealthStatus::Atp;
        self.jrq = crate::time::lc();
        self.hpq = self.jrq;

        crate::serial_println!("[container:{}] Started (sandbox #{}, {:?}, {:?})",
            self.ad, afh.0, self.config.akl, self.config.isolation);

        Ok(())
    }

    
    pub fn qg(&mut self) {
        if let Some(ary) = self.afh.take() {
            let _ = BD_.lock().hfy(ary);
        }
        self.arh = HealthStatus::Af;
        crate::serial_println!("[container:{}] Stopped", self.ad);
    }

    
    pub fn lzu(&mut self) -> Result<(), SandboxError> {
        self.qg();
        self.lzv += 1;
        self.ay()
    }

    
    pub fn bvn(&mut self, url: &str) -> Result<Fz, SandboxError> {
        let ary = self.afh.ok_or(SandboxError::Hh)?;

        let mk = BD_.lock().bvn(ary, url)?;

        self.hpq = crate::time::lc();
        self.xv += mk.gj.len();
        self.dxa += 1;
        self.adv.push(String::from(url));
        if self.adv.len() > 50 {
            self.adv.remove(0);
        }

        
        self.xov();

        Ok(mk)
    }

    
    pub fn hjd(&mut self, url: &str) -> Result<Fz, SandboxError> {
        let ary = self.afh.ok_or(SandboxError::Hh)?;

        let mk = BD_.lock().kvn(ary, url)?;

        self.hpq = crate::time::lc();
        self.xv += mk.gj.len();
        self.dxa += 1;

        Ok(mk)
    }

    
    pub fn xtu(&mut self) -> bool {
        if self.config.fyo == 0 {
            return true; 
        }
        if self.arh == HealthStatus::Af {
            return true;
        }

        let iu = crate::time::lc();
        let sg = self.config.fyo * 1000;

        
        if let Some(ary) = self.afh {
            let aas = BD_.lock();
            if let Some(is) = aas.get(ary) {
                if is.g == SandboxState::Hh {
                    drop(aas);
                    crate::serial_println!("[container:{}] watchdog: sandbox terminated, restarting", self.ad);
                    if self.config.gzi {
                        self.afh = None;
                        let _ = self.lzu();
                    } else {
                        self.arh = HealthStatus::Aj;
                    }
                    return false;
                }
                
                if iu.ao(self.hpq) > sg && self.dxa > 0 {
                    self.arh = HealthStatus::Bej;
                }
            } else {
                drop(aas);
                
                if self.config.gzi {
                    self.afh = None;
                    let _ = self.lzu();
                } else {
                    self.arh = HealthStatus::Aj;
                }
                return false;
            }
        }

        self.pzd = iu;
        true
    }

    
    fn xov(&mut self) {
        if self.afh.is_none() {
            self.arh = HealthStatus::Af;
            return;
        }

        
        let pci = if self.config.fsq > 0 {
            (self.dxa * 100) / self.config.fsq
        } else {
            0
        };

        let omu = if self.config.fob > 0 {
            (self.xv * 100) / self.config.fob
        } else {
            0
        };

        if pci > 90 || omu > 90 || self.cnt > 10 {
            self.arh = HealthStatus::Aj;
        } else if pci > 70 || omu > 70 || self.cnt > 3 {
            self.arh = HealthStatus::Bej;
        } else {
            self.arh = HealthStatus::Atp;
        }
    }

    
    pub fn cnn(&self) -> u64 {
        crate::time::lc().ao(self.jrq) / 1000
    }

    
    pub fn ibt(&self) -> String {
        let tzr = match self.config.isolation {
            IsolationMode::Amu => "software (capabilities)",
            IsolationMode::Cew => "hardware (EPT/VT-x)",
        };

        format!(
            "Container #{} '{}'\n\
             ├─ Health:     {:?}\n\
             ├─ Isolation:  {}\n\
             ├─ Policy:     {:?}\n\
             ├─ Sandbox:    {}\n\
             ├─ Uptime:     {}s\n\
             ├─ Requests:   {}/{}\n\
             ├─ Data:       {} bytes (limit {} MB)\n\
             ├─ Violations: {}\n\
             ├─ Restarts:   {}\n\
             ├─ JS:         {}\n\
             ├─ Rate limit: {}/min\n\
             └─ Watchdog:   {}",
            self.ad, self.config.j,
            self.arh,
            tzr,
            self.config.akl,
            self.afh.map(|e| format!("#{}", e.0)).unwrap_or_else(|| String::from("none")),
            self.cnn(),
            self.dxa, self.config.fsq,
            self.xv, self.config.fob / (1024 * 1024),
            self.cnt,
            self.lzv,
            if self.config.gyj { "allowed" } else { "BLOCKED" },
            self.config.hwq,
            if self.config.fyo > 0 {
                format!("{}s", self.config.fyo)
            } else {
                String::from("disabled")
            },
        )
    }
}




pub struct ContainerDaemon {
    bmm: BTreeMap<u32, WebContainer>,
    bcb: u32,
    
    pub kef: bool,
    
    pub gbh: u64,
    
    pub fgm: Option<u32>,
}

impl ContainerDaemon {
    pub fn new() -> Self {
        Self {
            bmm: BTreeMap::new(),
            bcb: 1,
            kef: false,
            gbh: 0,
            fgm: None,
        }
    }

    
    pub fn boot(&mut self) {
        self.gbh = crate::time::lc();
        self.kef = true;

        
        let ad = self.nha(ContainerConfig::default());
        if let Some(container) = self.bmm.ds(&ad) {
            match container.ay() {
                Ok(()) => {
                    self.fgm = Some(ad);
                    crate::serial_println!("[daemon] Default web container #{} started", ad);
                }
                Err(aa) => {
                    crate::serial_println!("[daemon] Failed to start default container: {:?}", aa);
                }
            }
        }

        crate::serial_println!("[daemon] Container daemon booted at {}ms", self.gbh);
    }

    
    pub fn nha(&mut self, config: ContainerConfig) -> u32 {
        let ad = self.bcb;
        self.bcb += 1;
        let container = WebContainer::new(ad, config);
        self.bmm.insert(ad, container);
        ad
    }

    
    pub fn pnz(&mut self, ad: u32) -> Result<(), SandboxError> {
        let container = self.bmm.ds(&ad)
            .ok_or(SandboxError::N)?;
        container.ay()
    }

    
    pub fn wun(&mut self, ad: u32) -> Result<(), SandboxError> {
        let container = self.bmm.ds(&ad)
            .ok_or(SandboxError::N)?;
        container.qg();
        Ok(())
    }

    
    pub fn vyd(&mut self, ad: u32) -> Result<(), SandboxError> {
        let container = self.bmm.ds(&ad)
            .ok_or(SandboxError::N)?;
        container.lzu()
    }

    
    pub fn rwj(&mut self, ad: u32) -> Result<(), SandboxError> {
        if let Some(mut container) = self.bmm.remove(&ad) {
            container.qg();
            if self.fgm == Some(ad) {
                self.fgm = None;
            }
        }
        Ok(())
    }

    
    pub fn bvn(&mut self, ad: Option<u32>, url: &str) -> Result<Fz, SandboxError> {
        let kkq = ad
            .efx(self.fgm)
            .ok_or(SandboxError::N)?;
        let container = self.bmm.ds(&kkq)
            .ok_or(SandboxError::N)?;
        container.bvn(url)
    }

    
    pub fn get(&self, ad: u32) -> Option<&WebContainer> {
        self.bmm.get(&ad)
    }

    
    pub fn ds(&mut self, ad: u32) -> Option<&mut WebContainer> {
        self.bmm.ds(&ad)
    }

    
    pub fn aoy(&self) -> Vec<(u32, &str, HealthStatus, bool)> {
        self.bmm.alv()
            .map(|r| (
                r.ad,
                r.config.j.as_str(),
                r.arh,
                self.fgm == Some(r.ad),
            ))
            .collect()
    }

    
    pub fn jwm(&mut self) {
        let esg: Vec<u32> = self.bmm.cai().hu().collect();
        for ad in esg {
            if let Some(container) = self.bmm.ds(&ad) {
                container.xtu();
            }
        }
    }

    
    pub fn cnn(&self) -> u64 {
        crate::time::lc().ao(self.gbh) / 1000
    }

    
    pub fn ibt(&self) -> String {
        let es = self.bmm.len();
        let aqk = self.bmm.alv()
            .hi(|r| r.arh != HealthStatus::Af)
            .az();
        let tob = self.bmm.alv()
            .hi(|r| r.arh == HealthStatus::Atp)
            .az();

        format!(
            "Container Daemon\n\
             ├─ Status:     {}\n\
             ├─ Uptime:     {}s\n\
             ├─ Containers: {} total, {} running, {} healthy\n\
             ├─ Default:    {}\n\
             └─ Isolation:  {} (EPT/VT-x: {})",
            if self.kef { "running" } else { "stopped" },
            self.cnn(),
            es, aqk, tob,
            self.fgm.map(|ad| format!("#{}", ad)).unwrap_or_else(|| String::from("none")),
            "software",
            if Self::tlw() { "available" } else { "not available" },
        )
    }

    
    fn tlw() -> bool {
        
        
        crate::hypervisor::zu()
    }
}



lazy_static::lazy_static! {
    pub static ref CJ_: Mutex<ContainerDaemon> = Mutex::new(ContainerDaemon::new());
}


pub fn qqw() {
    let mut bjs = CJ_.lock();
    bjs.boot();
}


pub fn bvn(url: &str) -> Result<Fz, SandboxError> {
    CJ_.lock().bvn(None, url)
}


pub fn ufp() -> Vec<(u32, String, HealthStatus, bool)> {
    CJ_.lock().aoy().dse()
        .map(|(ad, bo, i, bc)| (ad, String::from(bo), i, bc))
        .collect()
}


pub fn rth() -> String {
    CJ_.lock().ibt()
}


pub fn roe(ad: u32) -> Option<String> {
    CJ_.lock().get(ad).map(|r| r.ibt())
}


pub fn jwm() {
    CJ_.lock().jwm();
}
