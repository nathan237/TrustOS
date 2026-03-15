//! VPID (Virtual Processor ID) Management
//!
//! Implements VPID allocation and TLB isolation for VMs:
//! - Each VM gets a unique VPID (1-65535, 0 is reserved for host)
//! - VPIDs enable TLB tagging to avoid flushes on VM exit
//! - Provides INVVPID operations for selective TLB invalidation

use alloc::collections::BTreeSet;
use spin::Mutex;
use core::sync::atomic::{AtomicBool, AtomicU16, Ordering};

/// Maximum VPID value (16-bit, 0 reserved)
pub const MAX_VPID: u16 = 65535;

/// VPID 0 is reserved for the host
pub const HOST_VPID: u16 = 0;

/// Whether VPID is supported and enabled
static VPID_ENABLED: AtomicBool = AtomicBool::new(false);

/// Next VPID to try allocating
static NEXT_VPID: AtomicU16 = AtomicU16::new(1);

/// Set of allocated VPIDs
static ALLOCATED_VPIDS: Mutex<BTreeSet<u16>> = Mutex::new(BTreeSet::new());

// ============================================================================
// VPID TYPES AND OPERATIONS
// ============================================================================

/// INVVPID type operand
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u64)]
pub enum InvvpidType {
    /// Invalidate specific address in specific VPID
    IndividualAddress = 0,
    /// Invalidate all mappings for a specific VPID
    SingleContext = 1,
    /// Invalidate all VPIDs except global pages
    AllContext = 2,
    /// Invalidate all VPIDs except global, for specific address
    SingleContextRetainGlobal = 3,
}

/// INVVPID descriptor structure
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct InvvpidDescriptor {
    pub vpid: u16,
    pub reserved: [u8; 6],
    pub linear_address: u64,
}

impl InvvpidDescriptor {
    pub fn new(vpid: u16, addr: u64) -> Self {
        InvvpidDescriptor {
            vpid,
            reserved: [0; 6],
            linear_address: addr,
        }
    }
    
    pub fn for_vpid(vpid: u16) -> Self {
        Self::new(vpid, 0)
    }
}

// ============================================================================
// VPID ALLOCATOR
// ============================================================================

/// Initialize VPID support
pub fn init() -> bool {
    // Check if VPID is supported via IA32_VMX_PROCBASED_CTLS2
    let supported = check_vpid_support();
    
    if supported {
        VPID_ENABLED.store(true, Ordering::SeqCst);
        crate::serial_println!("[VPID] VPID support enabled - TLB isolation active");
    } else {
        crate::serial_println!("[VPID] VPID not supported - TLB flush on every exit");
    }
    
    supported
}

/// Check if VPID is supported by the CPU
fn check_vpid_support() -> bool {
    // Read IA32_VMX_PROCBASED_CTLS2 to check VPID support
    let ctls2 = super::vmx::read_msr(0x48B); // IA32_VMX_PROCBASED_CTLS2
    let allowed_1_settings = (ctls2 >> 32) as u32;
    
    // Bit 5 = Enable VPID
    (allowed_1_settings & (1 << 5)) != 0
}

/// Check if VPID is currently enabled
pub fn is_enabled() -> bool {
    VPID_ENABLED.load(Ordering::SeqCst)
}

/// Allocate a new VPID for a VM
pub fn allocate() -> Option<u16> {
    if !is_enabled() {
        return None;
    }
    
    let mut allocated = ALLOCATED_VPIDS.lock();
    
    // Try from next_vpid onwards
    let start = NEXT_VPID.load(Ordering::SeqCst);
    
    for offset in 0..MAX_VPID {
        let vpid = ((start as u32 + offset as u32) % (MAX_VPID as u32)) as u16;
        
        // Skip VPID 0 (reserved for host)
        if vpid == 0 {
            continue;
        }
        
        if !allocated.contains(&vpid) {
            allocated.insert(vpid);
            NEXT_VPID.store(vpid.wrapping_add(1).max(1), Ordering::SeqCst);
            
            crate::serial_println!("[VPID] Allocated VPID {} for new VM", vpid);
            return Some(vpid);
        }
    }
    
    // All VPIDs exhausted
    crate::serial_println!("[VPID] ERROR: All VPIDs exhausted!");
    None
}

/// Free a VPID when a VM is destroyed
pub fn free(vpid: u16) {
    if vpid == 0 {
        return; // Never free the host VPID
    }
    
    let mut allocated = ALLOCATED_VPIDS.lock();
    if allocated.remove(&vpid) {
        crate::serial_println!("[VPID] Freed VPID {}", vpid);
        
        // Invalidate all TLB entries for this VPID
        invalidate_vpid(vpid);
    }
}

/// Get number of allocated VPIDs
pub fn allocated_count() -> usize {
    ALLOCATED_VPIDS.lock().len()
}

// ============================================================================
// TLB INVALIDATION
// ============================================================================

/// Invalidate all TLB entries for a specific VPID
pub fn invalidate_vpid(vpid: u16) {
    if !is_enabled() {
        return;
    }
    
    let desc = InvvpidDescriptor::for_vpid(vpid);
    
    unsafe {
        invvpid(InvvpidType::SingleContext, &desc);
    }
}

/// Invalidate a specific address in a VPID's TLB
pub fn invalidate_address(vpid: u16, addr: u64) {
    if !is_enabled() {
        return;
    }
    
    let desc = InvvpidDescriptor::new(vpid, addr);
    
    unsafe {
        invvpid(InvvpidType::IndividualAddress, &desc);
    }
}

/// Invalidate all VPID contexts (global TLB flush)
pub fn invalidate_all() {
    if !is_enabled() {
        return;
    }
    
    let desc = InvvpidDescriptor::for_vpid(0);
    
    unsafe {
        invvpid(InvvpidType::AllContext, &desc);
    }
}

/// Execute INVVPID instruction
/// 
/// # Safety
/// This directly executes the INVVPID instruction which modifies TLB state.
/// Must only be called when in VMX root mode.
#[inline]
unsafe fn invvpid(inv_type: InvvpidType, desc: &InvvpidDescriptor) {
    let result: u8;
    
    core::arch::asm!(
        "invvpid {0}, [{1}]",
        "setc {2}",
        in(reg) inv_type as u64,
        in(reg) desc as *const InvvpidDescriptor,
        out(reg_byte) result,
        options(nostack)
    );
    
    if result != 0 {
        crate::serial_println!("[VPID] INVVPID failed! type={:?}", inv_type);
    }
}

// ============================================================================
// VMCS INTEGRATION HELPERS
// ============================================================================

/// Get the VPID value to write to VMCS for a VM
pub fn get_vmcs_vpid(vpid: Option<u16>) -> u64 {
    match vpid {
        Some(v) if is_enabled() => v as u64,
        _ => 0, // VPID 0 disables VPID tagging
    }
}

/// Get the secondary processor controls bit for VPID
pub fn get_secondary_controls_vpid() -> u64 {
    if is_enabled() {
        1 << 5 // Enable VPID bit in secondary controls
    } else {
        0
    }
}

// ============================================================================
// STATISTICS
// ============================================================================

/// VPID statistics
#[derive(Debug, Clone, Default)]
pub struct VpidStats {
    pub allocated: usize,
    pub freed: usize,
    pub invalidations: usize,
}

static VPID_STATS: Mutex<VpidStats> = Mutex::new(VpidStats {
    allocated: 0,
    freed: 0,
    invalidations: 0,
});

/// Get VPID statistics
pub fn get_stats() -> VpidStats {
    VPID_STATS.lock().clone()
}
