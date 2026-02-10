//! Subsystem Isolation Boundaries
//!
//! Addresses GitHub issue #1: "everything shoved in kernel, consider ring 3"
//!
//! While TrustOS currently runs entirely in ring 0, this module establishes
//! **logical isolation boundaries** between subsystems using capability-based
//! access control. Each subsystem receives dedicated capability tokens at boot
//! with minimal required rights (principle of least privilege).
//!
//! Operations that cross subsystem boundaries must present a valid capability,
//! and all violations are logged and counted. This prepares the architecture
//! for future ring 3 migration: when a subsystem moves to userspace, the
//! capability gate becomes a syscall boundary instead of an inline check.
//!
//! ARCHITECTURE:
//! ┌──────────────────────────────────────────────────┐
//! │ Kernel TCB (ring 0, minimal)                     │
//! │  ├─ Memory management          [Memory cap]      │
//! │  ├─ Interrupt handling         [Interrupt cap]   │
//! │  └─ Capability management      [Kernel cap]      │
//! ├──────────────────────────────────────────────────┤
//! │ Isolated subsystems (ring 0 today, ring 3 later) │
//! │  ├─ Disk/Storage               [BlockDevice cap] │
//! │  ├─ Network stack              [Network cap]     │
//! │  ├─ Graphics/Compositor        [Graphics cap]    │
//! │  ├─ Process management         [Process cap]     │
//! │  ├─ Hypervisor                 [Hypervisor cap]  │
//! │  ├─ Shell / User interface     [ShellExec cap]   │
//! │  ├─ Crypto / TLS               [Crypto cap]      │
//! │  ├─ Linux compat layer         [LinuxCompat cap] │
//! │  ├─ Serial / Debug             [Serial cap]      │
//! │  └─ Power management           [Power cap]       │
//! └──────────────────────────────────────────────────┘

use super::{CapabilityId, CapabilityType, CapabilityRights};
use spin::Mutex;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};

/// Logical kernel subsystem identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Subsystem {
    /// Core memory management (heap, page tables, physical allocator)
    Memory,
    /// Block device and filesystem access
    Storage,
    /// Network stack (TCP/UDP, sockets, DNS)
    Network,
    /// Graphics, framebuffer, GPU, compositor
    Graphics,
    /// Process/task management
    ProcessMgr,
    /// Hypervisor (VMX/SVM)
    Hypervisor,
    /// Shell and user command execution
    Shell,
    /// Cryptographic operations and TLS
    Crypto,
    /// Linux compatibility layer
    LinuxCompat,
    /// Serial port and debug output
    SerialDebug,
    /// Power management (ACPI, shutdown, reboot)
    Power,
    /// Interrupt and timer management
    Interrupts,
    /// PCI bus and device enumeration
    PciBus,
    /// USB / xHCI
    Usb,
}

impl Subsystem {
    /// Get the capability type this subsystem requires
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

    /// Human-readable name
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

    /// Default rights for this subsystem's capability token
    fn default_rights(&self) -> CapabilityRights {
        match self {
            // Core subsystems need more rights
            Self::Memory => CapabilityRights::ALL,
            Self::Interrupts => CapabilityRights::ALL,
            // Storage: read + write + control (no delete/grant)
            Self::Storage => CapabilityRights(
                CapabilityRights::READ.0 | CapabilityRights::WRITE.0 | CapabilityRights::CONTROL.0
            ),
            // Network: read + write + create (sockets)
            Self::Network => CapabilityRights(
                CapabilityRights::READ.0 | CapabilityRights::WRITE.0 | CapabilityRights::CREATE.0
            ),
            // Graphics: read + write + map (framebuffer)
            Self::Graphics => CapabilityRights(
                CapabilityRights::READ.0 | CapabilityRights::WRITE.0 | CapabilityRights::MAP.0
            ),
            // Process: read + write + create + control + signal
            Self::ProcessMgr => CapabilityRights(
                CapabilityRights::READ.0 | CapabilityRights::WRITE.0 |
                CapabilityRights::CREATE.0 | CapabilityRights::CONTROL.0 |
                CapabilityRights::SIGNAL.0
            ),
            // Hypervisor: all (already dangerous)
            Self::Hypervisor => CapabilityRights::ALL,
            // Shell: read + execute
            Self::Shell => CapabilityRights(
                CapabilityRights::READ.0 | CapabilityRights::EXECUTE.0
            ),
            // Crypto: read + execute (no write — key material is immutable)
            Self::Crypto => CapabilityRights::READ_EXECUTE,
            // Linux compat: read + write + execute
            Self::LinuxCompat => CapabilityRights(
                CapabilityRights::READ.0 | CapabilityRights::WRITE.0 | CapabilityRights::EXECUTE.0
            ),
            // Serial: read + write
            Self::SerialDebug => CapabilityRights::READ_WRITE,
            // Power: control + privileged (shutdown is dangerous)
            Self::Power => CapabilityRights(
                CapabilityRights::CONTROL.0 | CapabilityRights::PRIVILEGED.0
            ),
            // PCI: read + control
            Self::PciBus => CapabilityRights(
                CapabilityRights::READ.0 | CapabilityRights::CONTROL.0
            ),
            // USB: read + write + control
            Self::Usb => CapabilityRights(
                CapabilityRights::READ.0 | CapabilityRights::WRITE.0 | CapabilityRights::CONTROL.0
            ),
        }
    }

    /// Isolation level: "ring0-tcb", "ring0-isolated", or "ring3-candidate"
    pub fn isolation_level(&self) -> &'static str {
        match self {
            // These MUST stay in ring 0 (part of TCB)
            Self::Memory | Self::Interrupts => "ring0-tcb",
            // These could potentially move to ring 3 in the future
            Self::Network | Self::Graphics | Self::Shell |
            Self::LinuxCompat | Self::Crypto | Self::Usb |
            Self::SerialDebug => "ring3-candidate",
            // These need ring 0 but are logically isolated
            Self::Storage | Self::Hypervisor | Self::ProcessMgr |
            Self::Power | Self::PciBus => "ring0-isolated",
        }
    }

    /// All subsystems
    pub fn all() -> &'static [Subsystem] {
        &[
            Self::Memory, Self::Storage, Self::Network, Self::Graphics,
            Self::ProcessMgr, Self::Hypervisor, Self::Shell, Self::Crypto,
            Self::LinuxCompat, Self::SerialDebug, Self::Power, Self::Interrupts,
            Self::PciBus, Self::Usb,
        ]
    }
}

/// Per-subsystem capability token and violation tracking
struct SubsystemState {
    /// The capability token assigned to this subsystem
    capability: CapabilityId,
    /// Number of gate checks passed
    accesses: AtomicU64,
    /// Number of gate checks failed (violations)
    violations: AtomicU64,
}

/// Global subsystem capability map
static SUBSYSTEM_CAPS: Mutex<BTreeMap<Subsystem, CapabilityId>> = Mutex::new(BTreeMap::new());

/// Per-subsystem access counters (subsystem ordinal → (accesses, violations))
static SUBSYSTEM_ACCESSES: Mutex<BTreeMap<u8, (u64, u64)>> = Mutex::new(BTreeMap::new());

/// Total gate violations across all subsystems
static GATE_VIOLATIONS: AtomicU64 = AtomicU64::new(0);
/// Total gate checks (passed + failed)
static GATE_CHECKS: AtomicU64 = AtomicU64::new(0);

/// Initialize capability tokens for all subsystems.
/// Called once from `security::init()`.
pub fn init_subsystem_capabilities() {
    for subsystem in Subsystem::all() {
        let cap_type = subsystem.required_capability_type();
        let rights = subsystem.default_rights();
        // Owner ID encodes subsystem identity (0x5500 + ordinal)
        let owner = 0x5500 + (*subsystem as u64);
        let cap_id = super::create_capability(cap_type, rights, owner);
        SUBSYSTEM_CAPS.lock().insert(*subsystem, cap_id);
    }
    
    crate::serial_println!("[ISOLATION] {} subsystem capability tokens created", 
        Subsystem::all().len());
}

/// Get the capability token for a subsystem
pub fn get_subsystem_capability(subsystem: Subsystem) -> Option<CapabilityId> {
    SUBSYSTEM_CAPS.lock().get(&subsystem).copied()
}

/// Check if a subsystem has the required rights for an operation.
/// This is the main "gate" function — call it before any cross-boundary access.
/// Returns Ok(()) if allowed, Err with reason if denied.
pub fn gate_check(
    subsystem: Subsystem,
    required_rights: CapabilityRights,
) -> Result<(), super::SecurityError> {
    GATE_CHECKS.fetch_add(1, Ordering::Relaxed);
    
    let cap_id = match SUBSYSTEM_CAPS.lock().get(&subsystem).copied() {
        Some(id) => id,
        None => {
            GATE_VIOLATIONS.fetch_add(1, Ordering::Relaxed);
            crate::log_warn!("[ISOLATION] Gate denied: subsystem {:?} has no capability token", subsystem);
            return Err(super::SecurityError::InvalidCapability);
        }
    };
    
    let result = super::validate_typed(
        cap_id,
        subsystem.required_capability_type(),
        required_rights,
    );
    
    if result.is_err() {
        GATE_VIOLATIONS.fetch_add(1, Ordering::Relaxed);
        crate::log_warn!("[ISOLATION] Gate denied: {:?} lacks rights for operation", subsystem);
    }
    
    result
}

/// Convenience gate for storage read operations
pub fn gate_storage_read() -> Result<(), super::SecurityError> {
    gate_check(Subsystem::Storage, CapabilityRights::READ)
}

/// Convenience gate for storage write operations
pub fn gate_storage_write() -> Result<(), super::SecurityError> {
    gate_check(Subsystem::Storage, CapabilityRights::WRITE)
}

/// Convenience gate for network operations
pub fn gate_network() -> Result<(), super::SecurityError> {
    gate_check(Subsystem::Network, CapabilityRights::READ_WRITE)
}

/// Convenience gate for graphics/framebuffer operations
pub fn gate_graphics() -> Result<(), super::SecurityError> {
    gate_check(Subsystem::Graphics, CapabilityRights::READ_WRITE)
}

/// Convenience gate for process creation
pub fn gate_process_create() -> Result<(), super::SecurityError> {
    gate_check(Subsystem::ProcessMgr, CapabilityRights::CREATE)
}

/// Convenience gate for hypervisor operations
pub fn gate_hypervisor() -> Result<(), super::SecurityError> {
    gate_check(Subsystem::Hypervisor, CapabilityRights::ALL)
}

/// Convenience gate for shell command execution
pub fn gate_shell_exec() -> Result<(), super::SecurityError> {
    gate_check(Subsystem::Shell, CapabilityRights::EXECUTE)
}

/// Convenience gate for crypto operations
pub fn gate_crypto() -> Result<(), super::SecurityError> {
    gate_check(Subsystem::Crypto, CapabilityRights::READ_EXECUTE)
}

/// Convenience gate for power management (shutdown, reboot)
pub fn gate_power() -> Result<(), super::SecurityError> {
    gate_check(Subsystem::Power, CapabilityRights::CONTROL)
}

/// Number of registered subsystems
pub fn subsystem_count() -> usize {
    SUBSYSTEM_CAPS.lock().len()
}

/// Get total gate checks performed
pub fn total_gate_checks() -> u64 {
    GATE_CHECKS.load(Ordering::Relaxed)
}

/// Get total gate violations
pub fn total_gate_violations() -> u64 {
    GATE_VIOLATIONS.load(Ordering::Relaxed)
}

/// Get isolation status report for all subsystems
pub fn isolation_report() -> Vec<String> {
    let mut lines = Vec::new();
    let caps = SUBSYSTEM_CAPS.lock();
    
    lines.push(String::from("  Subsystem Isolation Status"));
    lines.push(String::from("  ──────────────────────────────────────────────────────────"));
    lines.push(String::from("  Subsystem       │ Isolation     │ Cap ID │ Rights"));
    lines.push(String::from("  ────────────────┼───────────────┼────────┼─────────────"));
    
    for subsystem in Subsystem::all() {
        let name = subsystem.name();
        let level = subsystem.isolation_level();
        let cap_id = caps.get(subsystem).map(|id| id.0).unwrap_or(0);
        let rights = subsystem.default_rights();
        
        let rights_str = format_rights(rights);
        
        lines.push(alloc::format!(
            "  {:<16}│ {:<14}│ {:>6} │ {}",
            name, level, cap_id, rights_str
        ));
    }
    
    lines.push(String::from("  ──────────────────────────────────────────────────────────"));
    lines.push(alloc::format!("  Gate checks: {}  |  Violations: {}",
        total_gate_checks(), total_gate_violations()));
    
    lines
}

/// Format capability rights as a compact string
fn format_rights(rights: CapabilityRights) -> String {
    let mut s = String::new();
    if rights.contains(CapabilityRights::READ) { s.push('R'); }
    if rights.contains(CapabilityRights::WRITE) { s.push('W'); }
    if rights.contains(CapabilityRights::EXECUTE) { s.push('X'); }
    if rights.contains(CapabilityRights::DELETE) { s.push('D'); }
    if rights.contains(CapabilityRights::CREATE) { s.push('C'); }
    if rights.contains(CapabilityRights::GRANT) { s.push('G'); }
    if rights.contains(CapabilityRights::CONTROL) { s.push('c'); }
    if rights.contains(CapabilityRights::MAP) { s.push('M'); }
    if rights.contains(CapabilityRights::SIGNAL) { s.push('S'); }
    if rights.contains(CapabilityRights::PRIVILEGED) { s.push('P'); }
    if s.is_empty() { s.push_str("none"); }
    s
}
