



use super::{CapabilityType, CapabilityRights};


#[derive(Debug, Clone, Copy)]
pub struct Co {
    
    pub akk: CapabilityType,
    
    pub abh: CapabilityRights,
    
    pub allow: bool,
}


pub const BTZ_: &[Co] = &[
    
    Co {
        akk: CapabilityType::Kernel,
        abh: CapabilityRights::Fi,
        allow: true,
    },
    
    Co {
        akk: CapabilityType::Memory,
        abh: CapabilityRights::Ba,
        allow: true,
    },
    
    Co {
        akk: CapabilityType::Channel,
        abh: CapabilityRights::LQ_,
        allow: true,
    },
    
    Co {
        akk: CapabilityType::Filesystem,
        abh: CapabilityRights::Ba,
        allow: true,
    },
    
    Co {
        akk: CapabilityType::Network,
        abh: CapabilityRights::LQ_,
        allow: true,
    },
    
    Co {
        akk: CapabilityType::Framebuffer,
        abh: CapabilityRights::Ba,
        allow: true,
    },
    
    Co {
        akk: CapabilityType::Graphics,
        abh: CapabilityRights::Ba,
        allow: true,
    },
    
    Co {
        akk: CapabilityType::Timer,
        abh: CapabilityRights::Ba,
        allow: true,
    },
    
    Co {
        akk: CapabilityType::Serial,
        abh: CapabilityRights::LQ_,
        allow: true,
    },
    
    Co {
        akk: CapabilityType::Process,
        abh: CapabilityRights::Fj,
        allow: true,
    },
    
    Co {
        akk: CapabilityType::Debug,
        abh: CapabilityRights::Kh,
        allow: true,
    },
    
    Co {
        akk: CapabilityType::Hypervisor,
        abh: CapabilityRights::Fi,
        allow: true,
    },
    
    Co {
        akk: CapabilityType::Power,
        abh: CapabilityRights::Kh,
        allow: true,
    },
    
    Co {
        akk: CapabilityType::PortIO,
        abh: CapabilityRights::Kh,
        allow: true,
    },
    
    Co {
        akk: CapabilityType::Dma,
        abh: CapabilityRights::Kh,
        allow: true,
    },
    
    Co {
        akk: CapabilityType::Crypto,
        abh: CapabilityRights::AIU_,
        allow: true,
    },
];


pub fn qcl() -> &'static [Co] {
    BTZ_
}


pub mod invariants {
    use super::*;
    
    
    pub fn pzp(
        parent_rights: CapabilityRights,
        child_rights: CapabilityRights,
    ) -> bool {
        parent_rights.contains(child_rights)
    }
    
    
    pub fn pzn(cap_rights: CapabilityRights) -> bool {
        cap_rights.contains(CapabilityRights::Ba) || 
        cap_rights.contains(CapabilityRights::Bh)
    }
    
    
    pub fn pzo(cap_type: CapabilityType) -> bool {
        matches!(cap_type, CapabilityType::Kernel)
    }
}
