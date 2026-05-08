





use core::sync::atomic::{AtomicU64, Ordering};
use alloc::string::String;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use spin::Mutex;


#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CapabilityId(pub u64);

impl CapabilityId {
    pub const Aoi: CapabilityId = CapabilityId(0);
}






#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilityType {
    
    
    Memory,
    
    Channel,
    
    Device,
    
    Process,
    
    Filesystem,
    
    Network,
    
    Kernel,
    
    
    
    BlockDeviceRead,
    
    BlockDeviceWrite,
    
    PartitionManagement,
    
    DiskFormat,
    
    
    
    PortIO,
    
    Interrupt,
    
    Timer,
    
    Dma,
    
    PciBus,
    
    Serial,
    
    Usb,
    
    
    
    Framebuffer,
    
    Graphics,
    
    WaylandCompositor,
    
    
    
    Power,
    
    Scheduler,
    
    Debug,
    
    Syscall,
    
    
    
    ShellExec,
    
    ExecBinary,
    
    Crypto,
    
    
    
    Hypervisor,
    
    LinuxCompat,
    
    Media,
    
    
    
    
    
    
    Dynamic(u32),
}

impl CapabilityType {
    
    pub fn danger_level(&self) -> u8 {
        match self {
            
            Self::Memory | Self::Channel | Self::Timer | Self::Serial |
            Self::BlockDeviceRead | Self::Media => 0,
            
            
            Self::Filesystem | Self::PciBus | Self::Framebuffer | Self::Graphics |
            Self::WaylandCompositor | Self::Crypto => 1,
            
            
            Self::Process | Self::Network | Self::Device | Self::Usb |
            Self::Scheduler | Self::Debug | Self::ShellExec | Self::LinuxCompat => 2,
            
            
            Self::PortIO | Self::Interrupt | Self::Dma | Self::BlockDeviceWrite |
            Self::Syscall | Self::ExecBinary | Self::Hypervisor => 3,
            
            
            Self::PartitionManagement | Self::Power => 4,
            
            
            Self::Kernel | Self::DiskFormat => 5,
            
            
            Self::Dynamic(id) => KQ_.lock()
                .get(&(*id))
                .map(|info| info.danger_level)
                .unwrap_or(2),
        }
    }
    
    
    pub fn category(&self) -> &'static str {
        match self {
            Self::Memory | Self::Channel | Self::Process | Self::Kernel => "Core",
            Self::Device | Self::PortIO | Self::Interrupt | Self::Timer |
            Self::Dma | Self::PciBus | Self::Serial | Self::Usb => "Hardware",
            Self::Filesystem | Self::BlockDeviceRead | Self::BlockDeviceWrite |
            Self::PartitionManagement | Self::DiskFormat => "Storage",
            Self::Network | Self::Crypto => "Network",
            Self::Framebuffer | Self::Graphics | Self::WaylandCompositor |
            Self::Media => "Display",
            Self::Power | Self::Scheduler | Self::Debug | Self::Syscall => "System",
            Self::ShellExec | Self::ExecBinary | Self::LinuxCompat |
            Self::Hypervisor => "Execution",
            Self::Dynamic(id) => KQ_.lock()
                .get(&(*id))
                .map(|info| info.category)
                .unwrap_or("Dynamic"),
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CapabilityRights(pub u32);

impl CapabilityRights {
    pub const Bc: Self = Self(0);
    pub const Ba: Self = Self(1 << 0);
    pub const Bh: Self = Self(1 << 1);
    pub const Fm: Self = Self(1 << 2);
    pub const Xl: Self = Self(1 << 3);
    pub const Jq: Self = Self(1 << 4);
    pub const Ze: Self = Self(1 << 5);
    
    pub const Fj: Self = Self(1 << 6);
    
    pub const Aaz: Self = Self(1 << 7);
    
    pub const Uv: Self = Self(1 << 8);
    
    pub const Kh: Self = Self(1 << 9);
    
    pub const Fi: Self = Self(0x3FF); 
    pub const LQ_: Self = Self(0x03); 
    pub const AIU_: Self = Self(0x05); 
    
    
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
    
    
    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }
}


#[derive(Debug)]
pub struct Capability {
    
    pub id: CapabilityId,
    
    pub cap_type: CapabilityType,
    
    pub rights: CapabilityRights,
    
    pub owner: u64,
    
    pub parent: Option<CapabilityId>,
    
    pub created_at: u64,
    
    pub expires_at: u64,
    
    usage_count: AtomicU64,
}

impl Capability {
    
    pub fn new(
        id: CapabilityId,
        cap_type: CapabilityType,
        rights: CapabilityRights,
        owner: u64,
    ) -> Self {
        Self {
            id,
            cap_type,
            rights,
            owner,
            parent: None,
            created_at: crate::logger::ckc(),
            expires_at: 0,
            usage_count: AtomicU64::new(0),
        }
    }
    
    
    pub fn cdl() -> Self {
        Self {
            id: CapabilityId::Aoi,
            cap_type: CapabilityType::Kernel,
            rights: CapabilityRights::Fi,
            owner: 0,
            parent: None,
            created_at: 0,
            expires_at: 0,
            usage_count: AtomicU64::new(0),
        }
    }
    
    
    pub fn has_rights(&self, aov: CapabilityRights) -> bool {
        self.rights.contains(aov)
    }
    
    
    pub fn is_expired(&self) -> bool {
        if self.expires_at == 0 {
            return false;
        }
        crate::logger::ckc() > self.expires_at
    }
    
    
    pub fn use_once(&self) {
        self.usage_count.fetch_add(1, Ordering::Relaxed);
    }
    
    
    pub fn usage(&self) -> u64 {
        self.usage_count.load(Ordering::Relaxed)
    }
}










#[derive(Debug, Clone)]
pub struct Oo {
    
    pub name: String,
    
    pub danger_level: u8,
    
    pub category: &'static str,
    
    pub description: String,
}


static KQ_: Mutex<BTreeMap<u32, Oo>> = Mutex::new(BTreeMap::new());


static CKY_: AtomicU64 = AtomicU64::new(1);



pub fn izk(
    name: &str,
    danger_level: u8,
    category: &'static str,
    description: &str,
) -> u32 {
    let id = CKY_.fetch_add(1, Ordering::Relaxed) as u32;
    let info = Oo {
        name: String::from(name),
        danger_level: danger_level.min(5),
        category,
        description: String::from(description),
    };
    KQ_.lock().insert(id, info);
    crate::log_debug!("Registered dynamic capability type: {} (id={})", name, id);
    id
}


pub fn mda(id: u32) -> Option<Oo> {
    KQ_.lock().get(&id).cloned()
}


pub fn ikq() -> Vec<(u32, Oo)> {
    KQ_.lock()
        .iter()
        .map(|(k, v)| (*k, v.clone()))
        .collect()
}


pub fn huo() -> usize {
    KQ_.lock().len()
}
