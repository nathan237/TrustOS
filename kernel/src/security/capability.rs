//! Capability types and structures
//! 
//! Unforgeable capability tokens for resource access control.

use core::sync::atomic::{AtomicU64, Ordering};

/// Unique capability identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CapabilityId(pub u64);

impl CapabilityId {
    pub const ROOT: CapabilityId = CapabilityId(0);
}

/// Type of resource this capability grants access to
///
/// Each kernel subsystem has a corresponding capability type.
/// Access to any resource requires holding a capability token with
/// the appropriate type and rights. See GitHub issue #4.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilityType {
    // === Core resources ===
    /// Memory region access (heap, page tables, physical memory)
    Memory,
    /// IPC channel access (message passing, shared memory)
    Channel,
    /// Generic device access (catch-all for unmapped hardware)
    Device,
    /// Process control (create, kill, signal, priority)
    Process,
    /// Filesystem access (files, directories, VFS mounts)
    Filesystem,
    /// Network access (sockets, TCP/UDP, raw packets)
    Network,
    /// Kernel control (privileged operations, module loading)
    Kernel,
    
    // === Storage security levels ===
    /// Block device read access (safe)
    BlockDeviceRead,
    /// Block device write access (dangerous — can corrupt data)
    BlockDeviceWrite,
    /// Partition table management (very dangerous — can lose all data)
    PartitionManagement,
    /// Low-level disk format (destructive — destroys everything)
    DiskFormat,
    
    // === Hardware I/O ===
    /// Raw x86 port I/O access (in/out instructions)
    PortIO,
    /// Interrupt management (IDT, PIC, APIC, IRQ routing)
    Interrupt,
    /// Timer and clock access (PIT, HPET, RTC, TSC)
    Timer,
    /// DMA buffer allocation and mapping
    Dma,
    /// PCI bus enumeration and config space access
    PciBus,
    /// Serial port access (COM1-COM4)
    Serial,
    /// USB/xHCI device access
    Usb,
    
    // === Display and graphics ===
    /// Direct framebuffer access (pixel-level rendering)
    Framebuffer,
    /// GPU and 3D graphics operations (OpenGL, raytracer, VirtIO GPU)
    Graphics,
    /// Wayland compositor protocol access
    WaylandCompositor,
    
    // === System management ===
    /// Power management (ACPI shutdown, reboot, sleep)
    Power,
    /// Scheduler control (priority, CPU pinning, task migration)
    Scheduler,
    /// Process debugging and tracing (ptrace)
    Debug,
    /// System call filtering and interception
    Syscall,
    
    // === Execution and sandboxing ===
    /// Shell command execution
    ShellExec,
    /// ELF binary loading and execution
    ExecBinary,
    /// Cryptographic operations (AES-NI, TLS key material, RNG)
    Crypto,
    
    // === Subsystem access ===
    /// Hypervisor operations (VM creation, VMX/SVM, EPT)
    Hypervisor,
    /// Linux compatibility layer access
    LinuxCompat,
    /// Media operations (video codec, audio, image decode)
    Media,
}

impl CapabilityType {
    /// Returns the danger level of this capability type (0 = safe, 5 = catastrophic)
    pub fn danger_level(&self) -> u8 {
        match self {
            // Safe read-only operations
            Self::Memory | Self::Channel | Self::Timer | Self::Serial |
            Self::BlockDeviceRead | Self::Media => 0,
            
            // Low risk
            Self::Filesystem | Self::PciBus | Self::Framebuffer | Self::Graphics |
            Self::WaylandCompositor | Self::Crypto => 1,
            
            // Moderate risk — can affect other processes or network
            Self::Process | Self::Network | Self::Device | Self::Usb |
            Self::Scheduler | Self::Debug | Self::ShellExec | Self::LinuxCompat => 2,
            
            // High risk — direct hardware access
            Self::PortIO | Self::Interrupt | Self::Dma | Self::BlockDeviceWrite |
            Self::Syscall | Self::ExecBinary | Self::Hypervisor => 3,
            
            // Very dangerous — data destruction possible
            Self::PartitionManagement | Self::Power => 4,
            
            // Catastrophic — full system control
            Self::Kernel | Self::DiskFormat => 5,
        }
    }
    
    /// Returns a human-readable category name
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
        }
    }
}

/// Rights that can be granted
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CapabilityRights(u32);

impl CapabilityRights {
    pub const NONE: Self = Self(0);
    pub const READ: Self = Self(1 << 0);
    pub const WRITE: Self = Self(1 << 1);
    pub const EXECUTE: Self = Self(1 << 2);
    pub const DELETE: Self = Self(1 << 3);
    pub const CREATE: Self = Self(1 << 4);
    pub const GRANT: Self = Self(1 << 5);
    /// Control/configure a resource (e.g., ioctl, set priority)
    pub const CONTROL: Self = Self(1 << 6);
    /// Map/allocate resource into address space
    pub const MAP: Self = Self(1 << 7);
    /// Receive notifications/signals from resource
    pub const SIGNAL: Self = Self(1 << 8);
    /// Access resource in privileged/supervisor mode
    pub const PRIVILEGED: Self = Self(1 << 9);
    
    pub const ALL: Self = Self(0x3FF); // 10 bits
    pub const READ_WRITE: Self = Self(0x03); // READ | WRITE
    pub const READ_EXECUTE: Self = Self(0x05); // READ | EXECUTE
    
    /// Combine rights
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
    
    /// Check if contains rights
    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }
}

/// A capability token
#[derive(Debug)]
pub struct Capability {
    /// Unique ID
    pub id: CapabilityId,
    /// Type of resource
    pub cap_type: CapabilityType,
    /// Granted rights
    pub rights: CapabilityRights,
    /// Owning task ID
    pub owner: u64,
    /// Parent capability (for derivation chain)
    pub parent: Option<CapabilityId>,
    /// Creation timestamp
    pub created_at: u64,
    /// Expiration timestamp (0 = never)
    pub expires_at: u64,
    /// Usage counter
    usage_count: AtomicU64,
}

impl Capability {
    /// Create new capability
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
            created_at: crate::logger::get_timestamp(),
            expires_at: 0,
            usage_count: AtomicU64::new(0),
        }
    }
    
    /// Create root capability with all rights
    pub fn root() -> Self {
        Self {
            id: CapabilityId::ROOT,
            cap_type: CapabilityType::Kernel,
            rights: CapabilityRights::ALL,
            owner: 0,
            parent: None,
            created_at: 0,
            expires_at: 0,
            usage_count: AtomicU64::new(0),
        }
    }
    
    /// Check if capability has required rights
    pub fn has_rights(&self, required: CapabilityRights) -> bool {
        self.rights.contains(required)
    }
    
    /// Check if capability is expired
    pub fn is_expired(&self) -> bool {
        if self.expires_at == 0 {
            return false;
        }
        crate::logger::get_timestamp() > self.expires_at
    }
    
    /// Increment usage counter
    pub fn use_once(&self) {
        self.usage_count.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Get usage count
    pub fn usage(&self) -> u64 {
        self.usage_count.load(Ordering::Relaxed)
    }
}
