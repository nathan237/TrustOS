







use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use spin::Mutex;
use core::sync::atomic::{AtomicU64, Ordering};






#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u64)]
pub enum MemoryProtection {
    
    None = 0,
    
    ReadOnly = 0b001,
    
    ReadWrite = 0b011,
    
    ExecuteOnly = 0b100,
    
    ReadExecute = 0b101,
    
    ReadWriteExecute = 0b111,
}

impl MemoryProtection {
    
    pub fn is_readable(&self) -> bool {
        (*self as u64) & 0b001 != 0
    }
    
    
    pub fn is_writable(&self) -> bool {
        (*self as u64) & 0b010 != 0
    }
    
    
    pub fn is_executable(&self) -> bool {
        (*self as u64) & 0b100 != 0
    }
}






#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RegionType {
    
    Ram,
    
    Code,
    
    RoData,
    
    RwData,
    
    Stack,
    
    Mmio,
    
    Reserved,
    
    Shared,
}


#[derive(Debug, Clone)]
pub struct MemoryRegion {
    pub start: u64,
    pub size: u64,
    pub region_type: RegionType,
    pub protection: MemoryProtection,
    pub name: &'static str,
}

impl MemoryRegion {
    pub fn new(start: u64, size: u64, region_type: RegionType, name: &'static str) -> Self {
        let protection = match region_type {
            RegionType::Ram => MemoryProtection::ReadWriteExecute,
            RegionType::Code => MemoryProtection::ReadExecute,
            RegionType::RoData => MemoryProtection::ReadOnly,
            RegionType::RwData => MemoryProtection::ReadWrite,
            RegionType::Stack => MemoryProtection::ReadWrite,
            RegionType::Mmio => MemoryProtection::ReadWrite,
            RegionType::Reserved => MemoryProtection::None,
            RegionType::Shared => MemoryProtection::ReadWrite,
        };
        
        MemoryRegion {
            start,
            size,
            region_type,
            protection,
            name,
        }
    }
    
    pub fn end(&self) -> u64 {
        self.start + self.size
    }
    
    pub fn contains(&self, addr: u64) -> bool {
        addr >= self.start && addr < self.end()
    }
}






pub struct Vu {
    pub vm_id: u64,
    pub regions: Vec<MemoryRegion>,
    pub gzr: u64,
}

impl Vu {
    
    pub fn new(vm_id: u64, memory_mb: usize) -> Self {
        let gzr = (memory_mb * 1024 * 1024) as u64;
        let mut regions = Vec::new();
        
        
        
        
        
        
        
        regions.push(MemoryRegion::new(0x0000, 0x1000, RegionType::Reserved, "null_guard"));
        regions.push(MemoryRegion::new(0x1000, 0x7000, RegionType::Code, "code"));
        regions.push(MemoryRegion::new(0x8000, 0x8000, RegionType::Stack, "stack"));
        
        let data_start = 0x10000u64;
        let data_size = gzr.saturating_sub(data_start);
        if data_size > 0 {
            regions.push(MemoryRegion::new(data_start, data_size, RegionType::Ram, "data"));
        }
        
        Vu {
            vm_id,
            regions,
            gzr,
        }
    }
    
    
    pub fn find_region(&self, addr: u64) -> Option<&MemoryRegion> {
        self.regions.iter().find(|r| r.contains(addr))
    }
    
    
    pub fn pxv(&mut self, qd: MemoryRegion) {
        
        self.regions.push(qd);
    }
    
    
    pub fn pzk(&self, addr: u64, is_write: bool, erk: bool) -> bool {
        if let Some(qd) = self.find_region(addr) {
            if is_write && !qd.protection.is_writable() {
                return false;
            }
            if erk && !qd.protection.is_executable() {
                return false;
            }
            if !is_write && !erk && !qd.protection.is_readable() {
                return false;
            }
            true
        } else {
            false 
        }
    }
}






#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ViolationType {
    Read,
    Write,
    Execute,
    ReadWrite,
    WriteExecute,
}


#[derive(Debug, Clone)]
pub struct Ev {
    pub vm_id: u64,
    pub guest_physical: u64,
    pub drb: Option<u64>,
    pub violation_type: ViolationType,
    pub timestamp_ms: u64,
    pub guest_rip: u64,
}

static BKQ_: Mutex<Vec<Ev>> = Mutex::new(Vec::new());
static BKP_: AtomicU64 = AtomicU64::new(0);


pub fn iyv(
    vm_id: u64,
    guest_physical: u64,
    drb: Option<u64>,
    exit_qualification: u64,
    guest_rip: u64,
) {
    let violation_type = nrp(exit_qualification);
    
    let psc = Ev {
        vm_id,
        guest_physical,
        drb,
        violation_type,
        timestamp_ms: crate::time::uptime_ms(),
        guest_rip,
    };
    
    BKP_.fetch_add(1, Ordering::SeqCst);
    
    let mut log = BKQ_.lock();
    if log.len() >= 100 {
        log.remove(0); 
    }
    log.push(psc);
    
    crate::serial_println!(
        "[EPT] Violation: VM {} GPA=0x{:X} type={:?} at RIP=0x{:X}",
        vm_id, guest_physical, violation_type, guest_rip
    );
}


fn nrp(qualification: u64) -> ViolationType {
    let read = (qualification & 1) != 0;
    let write = (qualification & 2) != 0;
    let execute = (qualification & 4) != 0;
    
    match (read, write, execute) {
        (true, true, _) => ViolationType::ReadWrite,
        (_, true, true) => ViolationType::WriteExecute,
        (_, true, _) => ViolationType::Write,
        (_, _, true) => ViolationType::Execute,
        _ => ViolationType::Read,
    }
}


pub fn jqd() -> u64 {
    BKP_.load(Ordering::SeqCst)
}


pub fn odq(count: usize) -> Vec<Ev> {
    let log = BKQ_.lock();
    let start = if log.len() > count { log.len() - count } else { 0 };
    log[start..].to_vec()
}






#[derive(Debug, Clone)]
pub struct Ni {
    pub passed: bool,
    pub message: &'static str,
    pub severity: SecuritySeverity,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SecuritySeverity {
    Info,
    Warning,
    Critical,
}


pub fn pzr(layout: &Vu) -> Vec<Ni> {
    let mut cgx = Vec::new();
    
    
    let mjt = layout.regions.iter()
        .any(|r| r.start == 0 && r.region_type == RegionType::Reserved);
    
    cgx.push(Ni {
        passed: mjt,
        message: "Null pointer guard page",
        severity: SecuritySeverity::Critical,
    });
    
    
    let mkb = layout.regions.iter()
        .any(|r| r.protection == MemoryProtection::ReadWriteExecute && 
             r.region_type != RegionType::Ram);
    
    cgx.push(Ni {
        passed: !mkb,
        message: "W^X (no writable+executable regions)",
        severity: SecuritySeverity::Warning,
    });
    
    
    let ovt = layout.regions.iter()
        .filter(|r| r.region_type == RegionType::Stack)
        .all(|r| !r.protection.is_executable());
    
    cgx.push(Ni {
        passed: ovt,
        message: "Stack is non-executable",
        severity: SecuritySeverity::Critical,
    });
    
    
    let kus = layout.regions.iter()
        .filter(|r| r.region_type == RegionType::Code)
        .all(|r| !r.protection.is_writable());
    
    cgx.push(Ni {
        passed: kus,
        message: "Code sections are read-only",
        severity: SecuritySeverity::Warning,
    });
    
    cgx
}






#[derive(Debug, Clone, Default)]
pub struct Tf {
    pub fdh: u64,
    pub mapped_pages: u64,
    pub rwx_pages: u64,
    pub shared_pages: u64,
    pub violations: u64,
}

static BKV_: Mutex<BTreeMap<u64, Tf>> = Mutex::new(BTreeMap::new());


pub fn qhy(vm_id: u64) -> Tf {
    BKV_.lock().get(&vm_id).cloned().unwrap_or_default()
}


pub fn rbq(vm_id: u64, metrics: Tf) {
    BKV_.lock().insert(vm_id, metrics);
}






pub fn jjv() -> bool {
    
    let cap = super::vmx::ach(0x48C);
    
    (cap & 1) != 0
}


pub fn qxy() -> bool {
    let cap = super::vmx::ach(0x48C);
    
    (cap & (1 << 21)) != 0
}


pub fn qxx() -> bool {
    let cap = super::vmx::ach(0x48C);
    
    (cap & (1 << 17)) != 0
}


pub fn qht() -> u8 {
    let cap = super::vmx::ach(0x48C);
    ((cap >> 8) & 0xFF) as u8
}
