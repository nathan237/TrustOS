



use super::{CapabilityType, CapabilityRights};


#[derive(Debug, Clone, Copy)]
pub struct Fr {
    
    pub bsa: CapabilityType,
    
    pub bao: CapabilityRights,
    
    pub allow: bool,
}


pub const BRE_: &[Fr] = &[
    
    Fr {
        bsa: CapabilityType::Xj,
        bao: CapabilityRights::Mr,
        allow: true,
    },
    
    Fr {
        bsa: CapabilityType::Cy,
        bao: CapabilityRights::Cm,
        allow: true,
    },
    
    Fr {
        bsa: CapabilityType::Channel,
        bao: CapabilityRights::KZ_,
        allow: true,
    },
    
    Fr {
        bsa: CapabilityType::Asj,
        bao: CapabilityRights::Cm,
        allow: true,
    },
    
    Fr {
        bsa: CapabilityType::As,
        bao: CapabilityRights::KZ_,
        allow: true,
    },
    
    Fr {
        bsa: CapabilityType::Asp,
        bao: CapabilityRights::Cm,
        allow: true,
    },
    
    Fr {
        bsa: CapabilityType::Jm,
        bao: CapabilityRights::Cm,
        allow: true,
    },
    
    Fr {
        bsa: CapabilityType::Timer,
        bao: CapabilityRights::Cm,
        allow: true,
    },
    
    Fr {
        bsa: CapabilityType::Amm,
        bao: CapabilityRights::KZ_,
        allow: true,
    },
    
    Fr {
        bsa: CapabilityType::Process,
        bao: CapabilityRights::Mt,
        allow: true,
    },
    
    Fr {
        bsa: CapabilityType::Debug,
        bao: CapabilityRights::Xy,
        allow: true,
    },
    
    Fr {
        bsa: CapabilityType::Ee,
        bao: CapabilityRights::Mr,
        allow: true,
    },
    
    Fr {
        bsa: CapabilityType::Hb,
        bao: CapabilityRights::Xy,
        allow: true,
    },
    
    Fr {
        bsa: CapabilityType::Awz,
        bao: CapabilityRights::Xy,
        allow: true,
    },
    
    Fr {
        bsa: CapabilityType::Ard,
        bao: CapabilityRights::Xy,
        allow: true,
    },
    
    Fr {
        bsa: CapabilityType::Jh,
        bao: CapabilityRights::AHA_,
        allow: true,
    },
];


pub fn ylm() -> &'static [Fr] {
    BRE_
}


pub mod invariants {
    use super::*;
    
    
    pub fn yia(
        vbp: CapabilityRights,
        rah: CapabilityRights,
    ) -> bool {
        vbp.contains(rah)
    }
    
    
    pub fn yhy(nbq: CapabilityRights) -> bool {
        nbq.contains(CapabilityRights::Cm) || 
        nbq.contains(CapabilityRights::Db)
    }
    
    
    pub fn yhz(cap_type: CapabilityType) -> bool {
        oh!(cap_type, CapabilityType::Xj)
    }
}
