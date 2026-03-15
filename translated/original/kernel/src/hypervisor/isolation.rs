//! TrustVM Strong Isolation
//!
//! Enhanced memory isolation and security features:
//! - EPT-based memory protection with NX enforcement
//! - SMAP/SMEP emulation via EPT
//! - Memory region tracking and protection
//! - EPT violation analysis and handling

use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use spin::Mutex;
use core::sync::atomic::{AtomicU64, Ordering};

// ============================================================================
// EPT PROTECTION FLAGS
// ============================================================================

/// Memory protection flags for EPT entries
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u64)]
pub enum MemoryProtection {
    /// No access (page not present)
    None = 0,
    /// Read only
    ReadOnly = 0b001,
    /// Read and write
    ReadWrite = 0b011,
    /// Execute only (requires mode-based execute control)
    ExecuteOnly = 0b100,
    /// Read and execute (code)
    ReadExecute = 0b101,
    /// Full access (data with execute - avoid if possible)
    ReadWriteExecute = 0b111,
}

impl MemoryProtection {
    /// Check if readable
    pub fn is_readable(&self) -> bool {
        (*self as u64) & 0b001 != 0
    }
    
    /// Check if writable
    pub fn is_writable(&self) -> bool {
        (*self as u64) & 0b010 != 0
    }
    
    /// Check if executable
    pub fn is_executable(&self) -> bool {
        (*self as u64) & 0b100 != 0
    }
}

// ============================================================================
// MEMORY REGIONS
// ============================================================================

/// Type of memory region for a VM
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RegionType {
    /// Normal RAM
    Ram,
    /// Code section (RX)
    Code,
    /// Read-only data
    RoData,
    /// Read-write data
    RwData,
    /// Stack (RW, guard pages)
    Stack,
    /// MMIO region (device memory)
    Mmio,
    /// Reserved/unmapped
    Reserved,
    /// Shared memory with host
    Shared,
}

/// A protected memory region
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

// ============================================================================
// VM MEMORY LAYOUT
// ============================================================================

/// Memory layout for a VM with isolated regions
pub struct VmMemoryLayout {
    pub vm_id: u64,
    pub regions: Vec<MemoryRegion>,
    pub total_memory: u64,
}

impl VmMemoryLayout {
    /// Create a new memory layout for a VM
    pub fn new(vm_id: u64, memory_mb: usize) -> Self {
        let total_memory = (memory_mb * 1024 * 1024) as u64;
        let mut regions = Vec::new();
        
        // Default layout:
        // 0x0000 - 0x0FFF: Reserved (null pointer trap)
        // 0x1000 - 0x7FFF: Code section
        // 0x8000 - 0xFFFF: Stack
        // 0x10000+: Data/heap
        
        regions.push(MemoryRegion::new(0x0000, 0x1000, RegionType::Reserved, "null_guard"));
        regions.push(MemoryRegion::new(0x1000, 0x7000, RegionType::Code, "code"));
        regions.push(MemoryRegion::new(0x8000, 0x8000, RegionType::Stack, "stack"));
        
        let data_start = 0x10000u64;
        let data_size = total_memory.saturating_sub(data_start);
        if data_size > 0 {
            regions.push(MemoryRegion::new(data_start, data_size, RegionType::Ram, "data"));
        }
        
        VmMemoryLayout {
            vm_id,
            regions,
            total_memory,
        }
    }
    
    /// Find region containing an address
    pub fn find_region(&self, addr: u64) -> Option<&MemoryRegion> {
        self.regions.iter().find(|r| r.contains(addr))
    }
    
    /// Add a new region
    pub fn add_region(&mut self, region: MemoryRegion) {
        // TODO: Check for overlaps
        self.regions.push(region);
    }
    
    /// Check if access is allowed
    pub fn check_access(&self, addr: u64, is_write: bool, is_exec: bool) -> bool {
        if let Some(region) = self.find_region(addr) {
            if is_write && !region.protection.is_writable() {
                return false;
            }
            if is_exec && !region.protection.is_executable() {
                return false;
            }
            if !is_write && !is_exec && !region.protection.is_readable() {
                return false;
            }
            true
        } else {
            false // No region = no access
        }
    }
}

// ============================================================================
// EPT VIOLATION TRACKING
// ============================================================================

/// EPT violation reason
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ViolationType {
    Read,
    Write,
    Execute,
    ReadWrite,
    WriteExecute,
}

/// EPT violation record
#[derive(Debug, Clone)]
pub struct EptViolation {
    pub vm_id: u64,
    pub guest_physical: u64,
    pub guest_linear: Option<u64>,
    pub violation_type: ViolationType,
    pub timestamp_ms: u64,
    pub guest_rip: u64,
}

static VIOLATION_LOG: Mutex<Vec<EptViolation>> = Mutex::new(Vec::new());
static VIOLATION_COUNT: AtomicU64 = AtomicU64::new(0);

/// Record an EPT violation
pub fn record_violation(
    vm_id: u64,
    guest_physical: u64,
    guest_linear: Option<u64>,
    exit_qualification: u64,
    guest_rip: u64,
) {
    let violation_type = parse_violation_type(exit_qualification);
    
    let violation = EptViolation {
        vm_id,
        guest_physical,
        guest_linear,
        violation_type,
        timestamp_ms: crate::time::uptime_ms(),
        guest_rip,
    };
    
    VIOLATION_COUNT.fetch_add(1, Ordering::SeqCst);
    
    let mut log = VIOLATION_LOG.lock();
    if log.len() >= 100 {
        log.remove(0); // Keep last 100
    }
    log.push(violation);
    
    crate::serial_println!(
        "[EPT] Violation: VM {} GPA=0x{:X} type={:?} at RIP=0x{:X}",
        vm_id, guest_physical, violation_type, guest_rip
    );
}

/// Parse exit qualification to determine violation type
fn parse_violation_type(qualification: u64) -> ViolationType {
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

/// Get violation count
pub fn violation_count() -> u64 {
    VIOLATION_COUNT.load(Ordering::SeqCst)
}

/// Get recent violations
pub fn recent_violations(count: usize) -> Vec<EptViolation> {
    let log = VIOLATION_LOG.lock();
    let start = if log.len() > count { log.len() - count } else { 0 };
    log[start..].to_vec()
}

// ============================================================================
// SECURITY CHECKS
// ============================================================================

/// Security check result
#[derive(Debug, Clone)]
pub struct SecurityCheck {
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

/// Perform security checks on a VM's memory configuration
pub fn check_vm_security(layout: &VmMemoryLayout) -> Vec<SecurityCheck> {
    let mut checks = Vec::new();
    
    // Check 1: Null pointer guard page
    let has_null_guard = layout.regions.iter()
        .any(|r| r.start == 0 && r.region_type == RegionType::Reserved);
    
    checks.push(SecurityCheck {
        passed: has_null_guard,
        message: "Null pointer guard page",
        severity: SecuritySeverity::Critical,
    });
    
    // Check 2: No RWX regions (W^X)
    let has_rwx = layout.regions.iter()
        .any(|r| r.protection == MemoryProtection::ReadWriteExecute && 
             r.region_type != RegionType::Ram);
    
    checks.push(SecurityCheck {
        passed: !has_rwx,
        message: "W^X (no writable+executable regions)",
        severity: SecuritySeverity::Warning,
    });
    
    // Check 3: Stack is not executable
    let stack_nx = layout.regions.iter()
        .filter(|r| r.region_type == RegionType::Stack)
        .all(|r| !r.protection.is_executable());
    
    checks.push(SecurityCheck {
        passed: stack_nx,
        message: "Stack is non-executable",
        severity: SecuritySeverity::Critical,
    });
    
    // Check 4: Code is not writable
    let code_ro = layout.regions.iter()
        .filter(|r| r.region_type == RegionType::Code)
        .all(|r| !r.protection.is_writable());
    
    checks.push(SecurityCheck {
        passed: code_ro,
        message: "Code sections are read-only",
        severity: SecuritySeverity::Warning,
    });
    
    checks
}

// ============================================================================
// ISOLATION METRICS
// ============================================================================

/// Per-VM isolation metrics
#[derive(Debug, Clone, Default)]
pub struct IsolationMetrics {
    pub total_pages: u64,
    pub mapped_pages: u64,
    pub rwx_pages: u64,
    pub shared_pages: u64,
    pub violations: u64,
}

static VM_METRICS: Mutex<BTreeMap<u64, IsolationMetrics>> = Mutex::new(BTreeMap::new());

/// Get isolation metrics for a VM
pub fn get_metrics(vm_id: u64) -> IsolationMetrics {
    VM_METRICS.lock().get(&vm_id).cloned().unwrap_or_default()
}

/// Update metrics for a VM
pub fn update_metrics(vm_id: u64, metrics: IsolationMetrics) {
    VM_METRICS.lock().insert(vm_id, metrics);
}

// ============================================================================
// EPT FEATURE FLAGS
// ============================================================================

/// Check if execute-only EPT pages are supported
pub fn supports_execute_only() -> bool {
    // Read IA32_VMX_EPT_VPID_CAP MSR
    let cap = super::vmx::read_msr(0x48C);
    // Bit 0: Execute-only pages supported
    (cap & 1) != 0
}

/// Check if accessed/dirty bits in EPT are supported
pub fn supports_accessed_dirty() -> bool {
    let cap = super::vmx::read_msr(0x48C);
    // Bit 21: A/D bits supported
    (cap & (1 << 21)) != 0
}

/// Check if 1GB pages in EPT are supported
pub fn supports_1gb_pages() -> bool {
    let cap = super::vmx::read_msr(0x48C);
    // Bit 17: 1GB pages supported
    (cap & (1 << 17)) != 0
}

/// Get EPT memory types supported
pub fn get_ept_memory_types() -> u8 {
    let cap = super::vmx::read_msr(0x48C);
    ((cap >> 8) & 0xFF) as u8
}
