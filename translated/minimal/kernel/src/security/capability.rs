





use core::sync::atomic::{AtomicU64, Ordering};
use alloc::string::String;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use spin::Mutex;


#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CapabilityId(pub u64);

impl CapabilityId {
    pub const Cka: CapabilityId = CapabilityId(0);
}






#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilityType {
    
    
    Cy,
    
    Channel,
    
    Wg,
    
    Process,
    
    Asj,
    
    As,
    
    Xj,
    
    
    
    Agr,
    
    Apn,
    
    Awq,
    
    Aqy,
    
    
    
    Awz,
    
    Fv,
    
    Timer,
    
    Ard,
    
    PciBus,
    
    Amm,
    
    Ol,
    
    
    
    Asp,
    
    Jm,
    
    WaylandCompositor,
    
    
    
    Hb,
    
    Scheduler,
    
    Debug,
    
    Hg,
    
    
    
    Ayr,
    
    Bgg,
    
    Jh,
    
    
    
    Ee,
    
    Nj,
    
    Bmb,
    
    
    
    
    
    
    Ari(u32),
}

impl CapabilityType {
    
    pub fn eom(&self) -> u8 {
        match self {
            
            Self::Cy | Self::Channel | Self::Timer | Self::Amm |
            Self::Agr | Self::Bmb => 0,
            
            
            Self::Asj | Self::PciBus | Self::Asp | Self::Jm |
            Self::WaylandCompositor | Self::Jh => 1,
            
            
            Self::Process | Self::As | Self::Wg | Self::Ol |
            Self::Scheduler | Self::Debug | Self::Ayr | Self::Nj => 2,
            
            
            Self::Awz | Self::Fv | Self::Ard | Self::Apn |
            Self::Hg | Self::Bgg | Self::Ee => 3,
            
            
            Self::Awq | Self::Hb => 4,
            
            
            Self::Xj | Self::Aqy => 5,
            
            
            Self::Ari(ad) => JW_.lock()
                .get(&(*ad))
                .map(|co| co.eom)
                .unwrap_or(2),
        }
    }
    
    
    pub fn gb(&self) -> &'static str {
        match self {
            Self::Cy | Self::Channel | Self::Process | Self::Xj => "Core",
            Self::Wg | Self::Awz | Self::Fv | Self::Timer |
            Self::Ard | Self::PciBus | Self::Amm | Self::Ol => "Hardware",
            Self::Asj | Self::Agr | Self::Apn |
            Self::Awq | Self::Aqy => "Storage",
            Self::As | Self::Jh => "Network",
            Self::Asp | Self::Jm | Self::WaylandCompositor |
            Self::Bmb => "Display",
            Self::Hb | Self::Scheduler | Self::Debug | Self::Hg => "System",
            Self::Ayr | Self::Bgg | Self::Nj |
            Self::Ee => "Execution",
            Self::Ari(ad) => JW_.lock()
                .get(&(*ad))
                .map(|co| co.gb)
                .unwrap_or("Dynamic"),
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CapabilityRights(pub u32);

impl CapabilityRights {
    pub const Cq: Self = Self(0);
    pub const Cm: Self = Self(1 << 0);
    pub const Db: Self = Self(1 << 1);
    pub const Mz: Self = Self(1 << 2);
    pub const Bea: Self = Self(1 << 3);
    pub const Vx: Self = Self(1 << 4);
    pub const Bhs: Self = Self(1 << 5);
    
    pub const Mt: Self = Self(1 << 6);
    
    pub const Blt: Self = Self(1 << 7);
    
    pub const Ayg: Self = Self(1 << 8);
    
    pub const Xy: Self = Self(1 << 9);
    
    pub const Mr: Self = Self(0x3FF); 
    pub const KZ_: Self = Self(0x03); 
    pub const AHA_: Self = Self(0x05); 
    
    
    pub const fn far(self, gq: Self) -> Self {
        Self(self.0 | gq.0)
    }
    
    
    pub const fn contains(self, gq: Self) -> bool {
        (self.0 & gq.0) == gq.0
    }
}


#[derive(Debug)]
pub struct Capability {
    
    pub ad: CapabilityId,
    
    pub cap_type: CapabilityType,
    
    pub bap: CapabilityRights,
    
    pub awj: u64,
    
    pub tu: Option<CapabilityId>,
    
    pub cju: u64,
    
    pub itj: u64,
    
    juz: AtomicU64,
}

impl Capability {
    
    pub fn new(
        ad: CapabilityId,
        cap_type: CapabilityType,
        bap: CapabilityRights,
        awj: u64,
    ) -> Self {
        Self {
            ad,
            cap_type,
            bap,
            awj,
            tu: None,
            cju: crate::logger::fjp(),
            itj: 0,
            juz: AtomicU64::new(0),
        }
    }
    
    
    pub fn exv() -> Self {
        Self {
            ad: CapabilityId::Cka,
            cap_type: CapabilityType::Xj,
            bap: CapabilityRights::Mr,
            awj: 0,
            tu: None,
            cju: 0,
            itj: 0,
            juz: AtomicU64::new(0),
        }
    }
    
    
    pub fn lbf(&self, cbj: CapabilityRights) -> bool {
        self.bap.contains(cbj)
    }
    
    
    pub fn hox(&self) -> bool {
        if self.itj == 0 {
            return false;
        }
        crate::logger::fjp() > self.itj
    }
    
    
    pub fn xpm(&self) {
        self.juz.fetch_add(1, Ordering::Relaxed);
    }
    
    
    pub fn pxo(&self) -> u64 {
        self.juz.load(Ordering::Relaxed)
    }
}










#[derive(Debug, Clone)]
pub struct Aho {
    
    pub j: String,
    
    pub eom: u8,
    
    pub gb: &'static str,
    
    pub dc: String,
}


static JW_: Mutex<BTreeMap<u32, Aho>> = Mutex::new(BTreeMap::new());


static CHP_: AtomicU64 = AtomicU64::new(1);



pub fn pbm(
    j: &str,
    eom: u8,
    gb: &'static str,
    dc: &str,
) -> u32 {
    let ad = CHP_.fetch_add(1, Ordering::Relaxed) as u32;
    let co = Aho {
        j: String::from(j),
        eom: eom.v(5),
        gb,
        dc: String::from(dc),
    };
    JW_.lock().insert(ad, co);
    crate::log_debug!("Registered dynamic capability type: {} (id={})", j, ad);
    ad
}


pub fn tdn(ad: u32) -> Option<Aho> {
    JW_.lock().get(&ad).abn()
}


pub fn ojp() -> Vec<(u32, Aho)> {
    JW_.lock()
        .iter()
        .map(|(eh, p)| (*eh, p.clone()))
        .collect()
}


pub fn nop() -> usize {
    JW_.lock().len()
}
