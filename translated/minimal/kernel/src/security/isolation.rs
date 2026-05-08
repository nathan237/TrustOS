

































use super::{CapabilityId, CapabilityType, CapabilityRights};
use spin::Mutex;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};


#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Subsystem {
    
    Memory,
    
    Storage,
    
    Network,
    
    Graphics,
    
    ProcessMgr,
    
    Hypervisor,
    
    Shell,
    
    Crypto,
    
    LinuxCompat,
    
    SerialDebug,
    
    Power,
    
    Interrupts,
    
    PciBus,
    
    Usb,
}

impl Subsystem {
    
    pub fn required_capability_type(&self) -> CapabilityType {
        match self {
            Self::Memory => CapabilityType::Memory,
            Self::Storage => CapabilityType::BlockDeviceRead,
            Self::Network => CapabilityType::Network,
            Self::Graphics => CapabilityType::Graphics,
            Self::ProcessMgr => CapabilityType::Process,
            Self::Hypervisor => CapabilityType::Hypervisor,
            Self::Shell => CapabilityType::ShellExec,
            Self::Crypto => CapabilityType::Crypto,
            Self::LinuxCompat => CapabilityType::LinuxCompat,
            Self::SerialDebug => CapabilityType::Serial,
            Self::Power => CapabilityType::Power,
            Self::Interrupts => CapabilityType::Interrupt,
            Self::PciBus => CapabilityType::PciBus,
            Self::Usb => CapabilityType::Usb,
        }
    }

    
    pub fn name(&self) -> &'static str {
        match self {
            Self::Memory => "Memory",
            Self::Storage => "Storage",
            Self::Network => "Network",
            Self::Graphics => "Graphics",
            Self::ProcessMgr => "Process",
            Self::Hypervisor => "Hypervisor",
            Self::Shell => "Shell",
            Self::Crypto => "Crypto",
            Self::LinuxCompat => "LinuxCompat",
            Self::SerialDebug => "Serial/Debug",
            Self::Power => "Power",
            Self::Interrupts => "Interrupts",
            Self::PciBus => "PCI Bus",
            Self::Usb => "USB",
        }
    }

    
    fn default_rights(&self) -> CapabilityRights {
        match self {
            
            Self::Memory => CapabilityRights::Fi,
            Self::Interrupts => CapabilityRights::Fi,
            
            Self::Storage => CapabilityRights(
                CapabilityRights::Ba.0 | CapabilityRights::Bh.0 | CapabilityRights::Fj.0
            ),
            
            Self::Network => CapabilityRights(
                CapabilityRights::Ba.0 | CapabilityRights::Bh.0 | CapabilityRights::Jq.0
            ),
            
            Self::Graphics => CapabilityRights(
                CapabilityRights::Ba.0 | CapabilityRights::Bh.0 | CapabilityRights::Aaz.0
            ),
            
            Self::ProcessMgr => CapabilityRights(
                CapabilityRights::Ba.0 | CapabilityRights::Bh.0 |
                CapabilityRights::Jq.0 | CapabilityRights::Fj.0 |
                CapabilityRights::Uv.0
            ),
            
            Self::Hypervisor => CapabilityRights::Fi,
            
            Self::Shell => CapabilityRights(
                CapabilityRights::Ba.0 | CapabilityRights::Fm.0
            ),
            
            Self::Crypto => CapabilityRights::AIU_,
            
            Self::LinuxCompat => CapabilityRights(
                CapabilityRights::Ba.0 | CapabilityRights::Bh.0 | CapabilityRights::Fm.0
            ),
            
            Self::SerialDebug => CapabilityRights::LQ_,
            
            Self::Power => CapabilityRights(
                CapabilityRights::Fj.0 | CapabilityRights::Kh.0
            ),
            
            Self::PciBus => CapabilityRights(
                CapabilityRights::Ba.0 | CapabilityRights::Fj.0
            ),
            
            Self::Usb => CapabilityRights(
                CapabilityRights::Ba.0 | CapabilityRights::Bh.0 | CapabilityRights::Fj.0
            ),
        }
    }

    
    pub fn isolation_level(&self) -> &'static str {
        match self {
            
            Self::Memory | Self::Interrupts => "ring0-tcb",
            
            Self::Network | Self::Graphics | Self::Shell |
            Self::LinuxCompat | Self::Crypto | Self::Usb |
            Self::SerialDebug => "ring3-candidate",
            
            Self::Storage | Self::Hypervisor | Self::ProcessMgr |
            Self::Power | Self::PciBus => "ring0-isolated",
        }
    }

    
    pub fn all() -> &'static [Subsystem] {
        &[
            Self::Memory, Self::Storage, Self::Network, Self::Graphics,
            Self::ProcessMgr, Self::Hypervisor, Self::Shell, Self::Crypto,
            Self::LinuxCompat, Self::SerialDebug, Self::Power, Self::Interrupts,
            Self::PciBus, Self::Usb,
        ]
    }
}


struct Bdm {
    
    capability: CapabilityId,
    
    accesses: AtomicU64,
    
    violations: AtomicU64,
}


static QQ_: Mutex<BTreeMap<Subsystem, CapabilityId>> = Mutex::new(BTreeMap::new());


static EKQ_: Mutex<BTreeMap<u8, (u64, u64)>> = Mutex::new(BTreeMap::new());


static AEH_: AtomicU64 = AtomicU64::new(0);

static AUU_: AtomicU64 = AtomicU64::new(0);



pub fn mpn() {
    for acs in Subsystem::all() {
        let cap_type = acs.required_capability_type();
        let rights = acs.default_rights();
        
        let owner = 0x5500 + (*acs as u64);
        let cap_id = super::fpa(cap_type, rights, owner);
        QQ_.lock().insert(*acs, cap_id);
    }
    
    crate::serial_println!("[ISOLATION] {} subsystem capability tokens created", 
        Subsystem::all().len());
}


pub fn qip(acs: Subsystem) -> Option<CapabilityId> {
    QQ_.lock().get(&acs).copied()
}




pub fn bmj(
    acs: Subsystem,
    abh: CapabilityRights,
) -> Result<(), super::SecurityError> {
    AUU_.fetch_add(1, Ordering::Relaxed);
    
    let cap_id = match QQ_.lock().get(&acs).copied() {
        Some(id) => id,
        None => {
            AEH_.fetch_add(1, Ordering::Relaxed);
            crate::log_warn!("[ISOLATION] Gate denied: subsystem {:?} has no capability token", acs);
            return Err(super::SecurityError::InvalidCapability);
        }
    };
    
    let result = super::prc(
        cap_id,
        acs.required_capability_type(),
        abh,
    );
    
    if result.is_err() {
        AEH_.fetch_add(1, Ordering::Relaxed);
        crate::log_warn!("[ISOLATION] Gate denied: {:?} lacks rights for operation", acs);
    }
    
    result
}


pub fn qgw() -> Result<(), super::SecurityError> {
    bmj(Subsystem::Storage, CapabilityRights::Ba)
}


pub fn qgx() -> Result<(), super::SecurityError> {
    bmj(Subsystem::Storage, CapabilityRights::Bh)
}


pub fn qgs() -> Result<(), super::SecurityError> {
    bmj(Subsystem::Network, CapabilityRights::LQ_)
}


pub fn qgq() -> Result<(), super::SecurityError> {
    bmj(Subsystem::Graphics, CapabilityRights::LQ_)
}


pub fn qgu() -> Result<(), super::SecurityError> {
    bmj(Subsystem::ProcessMgr, CapabilityRights::Jq)
}


pub fn qgr() -> Result<(), super::SecurityError> {
    bmj(Subsystem::Hypervisor, CapabilityRights::Fi)
}


pub fn qgv() -> Result<(), super::SecurityError> {
    bmj(Subsystem::Shell, CapabilityRights::Fm)
}


pub fn qgp() -> Result<(), super::SecurityError> {
    bmj(Subsystem::Crypto, CapabilityRights::AIU_)
}


pub fn qgt() -> Result<(), super::SecurityError> {
    bmj(Subsystem::Power, CapabilityRights::Fj)
}


pub fn jjq() -> usize {
    QQ_.lock().len()
}


pub fn jnu() -> u64 {
    AUU_.load(Ordering::Relaxed)
}


pub fn jnv() -> u64 {
    AEH_.load(Ordering::Relaxed)
}


pub fn mui() -> Vec<String> {
    let mut lines = Vec::new();
    let caps = QQ_.lock();
    
    lines.push(String::from("  Subsystem Isolation Status"));
    lines.push(String::from("  ──────────────────────────────────────────────────────────"));
    lines.push(String::from("  Subsystem       │ Isolation     │ Cap ID │ Rights"));
    lines.push(String::from("  ────────────────┼───────────────┼────────┼─────────────"));
    
    for acs in Subsystem::all() {
        let name = acs.name();
        let level = acs.isolation_level();
        let cap_id = caps.get(acs).map(|id| id.0).unwrap_or(0);
        let rights = acs.default_rights();
        
        let ohb = lxs(rights);
        
        lines.push(alloc::format!(
            "  {:<16}│ {:<14}│ {:>6} │ {}",
            name, level, cap_id, ohb
        ));
    }
    
    lines.push(String::from("  ──────────────────────────────────────────────────────────"));
    lines.push(alloc::format!("  Gate checks: {}  |  Violations: {}",
        jnu(), jnv()));
    
    lines
}


fn lxs(rights: CapabilityRights) -> String {
    let mut j = String::new();
    if rights.contains(CapabilityRights::Ba) { j.push('R'); }
    if rights.contains(CapabilityRights::Bh) { j.push('W'); }
    if rights.contains(CapabilityRights::Fm) { j.push('X'); }
    if rights.contains(CapabilityRights::Xl) { j.push('D'); }
    if rights.contains(CapabilityRights::Jq) { j.push('C'); }
    if rights.contains(CapabilityRights::Ze) { j.push('G'); }
    if rights.contains(CapabilityRights::Fj) { j.push('c'); }
    if rights.contains(CapabilityRights::Aaz) { j.push('M'); }
    if rights.contains(CapabilityRights::Uv) { j.push('S'); }
    if rights.contains(CapabilityRights::Kh) { j.push('P'); }
    if j.is_empty() { j.push_str("none"); }
    j
}
