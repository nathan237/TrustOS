//! TrustVM API - Complete Host-Guest Communication API
//!
//! Provides a comprehensive API for:
//! - VM introspection and capabilities
//! - Event notifications and subscriptions
//! - Secure communication channels
//! - Resource management and quotas

use alloc::string::String;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use spin::Mutex;
use core::sync::atomic::{AtomicU64, Ordering};

// ============================================================================
// CAPABILITIES
// ============================================================================

/// TrustVM capabilities flags
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u64)]
pub enum Capability {
    /// VMX root mode is active
    VmxEnabled = 1 << 0,
    /// EPT (Extended Page Tables) supported
    EptSupported = 1 << 1,
    /// VPID (Virtual Processor ID) supported
    VpidSupported = 1 << 2,
    /// Unrestricted guest mode
    UnrestrictedGuest = 1 << 3,
    /// VMCS shadowing
    VmcsShadowing = 1 << 4,
    /// Posted interrupts
    PostedInterrupts = 1 << 5,
    /// EPT accessed/dirty bits
    EptAccessDirty = 1 << 6,
    /// Virtual console available
    VirtualConsole = 1 << 7,
    /// Shared filesystem (VirtFS)
    SharedFilesystem = 1 << 8,
    /// Nested virtualization
    NestedVirtualization = 1 << 9,
    /// Live migration support
    LiveMigration = 1 << 10,
    /// Memory ballooning
    MemoryBallooning = 1 << 11,
    /// Device passthrough
    DevicePassthrough = 1 << 12,
}

/// Collected capabilities as a bitfield
#[derive(Debug, Clone, Copy, Default)]
pub struct Capabilities {
    bits: u64,
}

impl Capabilities {
    pub fn new() -> Self {
        Capabilities { bits: 0 }
    }
    
    pub fn set(&mut self, cap: Capability) {
        self.bits |= cap as u64;
    }
    
    pub fn has(&self, cap: Capability) -> bool {
        (self.bits & (cap as u64)) != 0
    }
    
    pub fn as_u64(&self) -> u64 {
        self.bits
    }
    
    pub fn from_u64(bits: u64) -> Self {
        Capabilities { bits }
    }
}

/// Get current TrustVM capabilities
pub fn get_capabilities() -> Capabilities {
    let mut caps = Capabilities::new();
    
    // Check VMX support
    if let Ok(vmx_caps) = super::vmx::check_vmx_support() {
        if vmx_caps.supported {
            caps.set(Capability::VmxEnabled);
        }
        if vmx_caps.ept_supported {
            caps.set(Capability::EptSupported);
        }
        if vmx_caps.vpid_supported {
            caps.set(Capability::VpidSupported);
        }
        if vmx_caps.unrestricted_guest {
            caps.set(Capability::UnrestrictedGuest);
        }
    }
    
    // Always available features
    caps.set(Capability::VirtualConsole);
    caps.set(Capability::SharedFilesystem);
    
    caps
}

// ============================================================================
// VM INFORMATION / INTROSPECTION
// ============================================================================

/// Extended VM information
#[derive(Debug, Clone)]
pub struct VmInfo {
    pub id: u64,
    pub name: String,
    pub state: VmStateInfo,
    pub memory_mb: usize,
    pub vcpus: usize,
    pub uptime_ms: u64,
    pub stats: VmStatistics,
    pub isolation: IsolationInfo,
}

/// VM state information
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VmStateInfo {
    Created,
    Initializing,
    Running,
    Paused,
    Stopping,
    Stopped,
    Crashed,
    Migrating,
}

/// VM statistics
#[derive(Debug, Clone, Default)]
pub struct VmStatistics {
    pub total_exits: u64,
    pub cpuid_exits: u64,
    pub io_exits: u64,
    pub msr_exits: u64,
    pub hlt_exits: u64,
    pub ept_violations: u64,
    pub hypercalls: u64,
    pub interrupts_injected: u64,
    pub cpu_time_ns: u64,
}

/// Isolation status information
#[derive(Debug, Clone)]
pub struct IsolationInfo {
    pub vpid: Option<u16>,
    pub ept_enabled: bool,
    pub ept_pages: usize,
    pub execute_only_supported: bool,
    pub supervisor_shadow_stack: bool,
}

// ============================================================================
// EVENTS
// ============================================================================

/// VM event types
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u32)]
pub enum VmEventType {
    /// VM was created
    Created = 0,
    /// VM started running
    Started = 1,
    /// VM was paused
    Paused = 2,
    /// VM was resumed
    Resumed = 3,
    /// VM stopped normally
    Stopped = 4,
    /// VM crashed
    Crashed = 5,
    /// Console output available
    ConsoleOutput = 6,
    /// Hypercall received
    Hypercall = 7,
    /// EPT violation occurred
    EptViolation = 8,
    /// Memory limit reached
    MemoryWarning = 9,
    /// VPID flush requested
    VpidFlush = 10,
}

/// VM event with details
#[derive(Debug, Clone)]
pub struct VmEvent {
    pub event_type: VmEventType,
    pub vm_id: u64,
    pub timestamp_ms: u64,
    pub data: VmEventData,
}

/// Event-specific data
#[derive(Debug, Clone)]
pub enum VmEventData {
    None,
    ExitCode(i32),
    Message(String),
    Address(u64),
    HypercallInfo { function: u64, result: i64 },
}

/// Event callback type
pub type EventCallback = fn(event: &VmEvent);

/// Event subscriptions
struct EventSubscription {
    callback: EventCallback,
    vm_filter: Option<u64>, // None = all VMs
    event_filter: Option<VmEventType>, // None = all events
}

static EVENT_SUBSCRIPTIONS: Mutex<Vec<EventSubscription>> = Mutex::new(Vec::new());
static EVENT_LOG: Mutex<Vec<VmEvent>> = Mutex::new(Vec::new());
static EVENT_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Subscribe to VM events
pub fn subscribe_events(
    callback: EventCallback,
    vm_filter: Option<u64>,
    event_filter: Option<VmEventType>,
) -> u64 {
    let sub = EventSubscription {
        callback,
        vm_filter,
        event_filter,
    };
    
    let mut subs = EVENT_SUBSCRIPTIONS.lock();
    subs.push(sub);
    EVENT_COUNTER.fetch_add(1, Ordering::SeqCst)
}

/// Emit a VM event
pub fn emit_event(event_type: VmEventType, vm_id: u64, data: VmEventData) {
    let event = VmEvent {
        event_type,
        vm_id,
        timestamp_ms: crate::time::uptime_ms(),
        data,
    };
    
    // Log the event
    {
        let mut log = EVENT_LOG.lock();
        if log.len() >= 1000 {
            log.remove(0); // Keep last 1000 events
        }
        log.push(event.clone());
    }
    
    // Notify subscribers
    let subs = EVENT_SUBSCRIPTIONS.lock();
    for sub in subs.iter() {
        // Check VM filter
        if let Some(filter_vm) = sub.vm_filter {
            if filter_vm != vm_id {
                continue;
            }
        }
        
        // Check event type filter
        if let Some(filter_type) = sub.event_filter {
            if filter_type != event_type {
                continue;
            }
        }
        
        // Call the callback
        (sub.callback)(&event);
    }
}

/// Get recent events (up to count)
pub fn get_recent_events(count: usize) -> Vec<VmEvent> {
    let log = EVENT_LOG.lock();
    let start = if log.len() > count { log.len() - count } else { 0 };
    log[start..].to_vec()
}

// ============================================================================
// COMMUNICATION CHANNELS
// ============================================================================

/// Channel for host-guest communication
#[derive(Debug)]
pub struct VmChannel {
    id: u64,
    vm_id: u64,
    name: String,
    tx_buffer: Vec<u8>,
    rx_buffer: Vec<u8>,
    max_size: usize,
}

impl VmChannel {
    pub fn new(id: u64, vm_id: u64, name: &str, max_size: usize) -> Self {
        VmChannel {
            id,
            vm_id,
            name: String::from(name),
            tx_buffer: Vec::with_capacity(max_size),
            rx_buffer: Vec::with_capacity(max_size),
            max_size,
        }
    }
    
    /// Send data to guest
    pub fn send(&mut self, data: &[u8]) -> Result<usize, &'static str> {
        let available = self.max_size - self.tx_buffer.len();
        let to_send = data.len().min(available);
        
        if to_send == 0 {
            return Err("Channel buffer full");
        }
        
        self.tx_buffer.extend_from_slice(&data[..to_send]);
        Ok(to_send)
    }
    
    /// Receive data from guest
    pub fn recv(&mut self, buf: &mut [u8]) -> usize {
        let to_read = buf.len().min(self.rx_buffer.len());
        buf[..to_read].copy_from_slice(&self.rx_buffer[..to_read]);
        self.rx_buffer.drain(..to_read);
        to_read
    }
    
    /// Data available to read
    pub fn available(&self) -> usize {
        self.rx_buffer.len()
    }
    
    /// Space available to write
    pub fn space(&self) -> usize {
        self.max_size - self.tx_buffer.len()
    }
}

/// Channel manager
static CHANNELS: Mutex<BTreeMap<u64, VmChannel>> = Mutex::new(BTreeMap::new());
static CHANNEL_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Create a communication channel with a VM
pub fn create_channel(vm_id: u64, name: &str, max_size: usize) -> u64 {
    let id = CHANNEL_COUNTER.fetch_add(1, Ordering::SeqCst);
    let channel = VmChannel::new(id, vm_id, name, max_size);
    
    CHANNELS.lock().insert(id, channel);
    
    crate::serial_println!("[API] Created channel {} for VM {}: {}", id, vm_id, name);
    id
}

/// Send data on a channel
pub fn channel_send(channel_id: u64, data: &[u8]) -> Result<usize, &'static str> {
    let mut channels = CHANNELS.lock();
    match channels.get_mut(&channel_id) {
        Some(channel) => channel.send(data),
        None => Err("Channel not found"),
    }
}

/// Receive data from a channel
pub fn channel_recv(channel_id: u64, buf: &mut [u8]) -> Result<usize, &'static str> {
    let mut channels = CHANNELS.lock();
    match channels.get_mut(&channel_id) {
        Some(channel) => Ok(channel.recv(buf)),
        None => Err("Channel not found"),
    }
}

// ============================================================================
// RESOURCE QUOTAS
// ============================================================================

/// Resource quota configuration
#[derive(Debug, Clone)]
pub struct ResourceQuota {
    /// Maximum memory in bytes
    pub max_memory: usize,
    /// Maximum vCPUs
    pub max_vcpus: usize,
    /// Maximum I/O bandwidth (bytes/sec, 0 = unlimited)
    pub max_io_bandwidth: usize,
    /// Maximum hypercalls per second
    pub max_hypercalls_per_sec: usize,
    /// CPU time limit (percentage, 0-100, 0 = unlimited)
    pub cpu_limit_percent: u8,
}

impl Default for ResourceQuota {
    fn default() -> Self {
        ResourceQuota {
            max_memory: 256 * 1024 * 1024, // 256MB default
            max_vcpus: 4,
            max_io_bandwidth: 0,
            max_hypercalls_per_sec: 10000,
            cpu_limit_percent: 0,
        }
    }
}

static VM_QUOTAS: Mutex<BTreeMap<u64, ResourceQuota>> = Mutex::new(BTreeMap::new());

/// Set resource quota for a VM
pub fn set_quota(vm_id: u64, quota: ResourceQuota) {
    VM_QUOTAS.lock().insert(vm_id, quota);
}

/// Get resource quota for a VM
pub fn get_quota(vm_id: u64) -> Option<ResourceQuota> {
    VM_QUOTAS.lock().get(&vm_id).cloned()
}

// ============================================================================
// HYPERCALL INTERFACE EXTENSIONS
// ============================================================================

/// Extended hypercall numbers for Phase 3 API
pub mod hypercall {
    // Basic hypercalls (0x00 - 0xFF) - Phase 2
    pub const PRINT: u64 = 0x00;
    pub const EXIT: u64 = 0x01;
    pub const GET_TIME: u64 = 0x02;
    pub const CONSOLE_WRITE: u64 = 0x03;
    pub const CONSOLE_READ: u64 = 0x04;
    
    // VirtFS (0x100 - 0x1FF) - Phase 2
    pub const VIRTFS_BASE: u64 = 0x100;
    
    // TrustVM API (0x200 - 0x2FF) - Phase 3
    pub const GET_CAPABILITIES: u64 = 0x200;
    pub const GET_VM_INFO: u64 = 0x201;
    pub const GET_STATS: u64 = 0x202;
    pub const CHANNEL_OPEN: u64 = 0x210;
    pub const CHANNEL_CLOSE: u64 = 0x211;
    pub const CHANNEL_SEND: u64 = 0x212;
    pub const CHANNEL_RECV: u64 = 0x213;
    pub const CHANNEL_POLL: u64 = 0x214;
    pub const SET_NAME: u64 = 0x220;
    pub const REQUEST_MEMORY: u64 = 0x221;
    pub const YIELD: u64 = 0x222;
    pub const SHUTDOWN: u64 = 0x223;
    pub const REBOOT: u64 = 0x224;
    
    // Debugging (0x300 - 0x3FF)
    pub const DEBUG_BREAK: u64 = 0x300;
    pub const DEBUG_PRINT: u64 = 0x301;
    pub const DEBUG_TRACE: u64 = 0x302;
}

/// Handle extended hypercalls (0x200+)
pub fn handle_api_hypercall(vm_id: u64, function: u64, args: &[u64; 4]) -> (i64, u64) {
    use hypercall::*;
    
    match function {
        GET_CAPABILITIES => {
            let caps = get_capabilities();
            (0, caps.as_u64())
        }
        
        GET_VM_INFO => {
            // Return VM ID and state in response
            // Full info would need shared memory
            (0, vm_id)
        }
        
        GET_STATS => {
            // Would need access to VM stats
            (0, 0)
        }
        
        CHANNEL_OPEN => {
            // args[0] = name pointer (in guest memory)
            // For now, create with default name
            let channel_id = create_channel(vm_id, "guest_channel", 4096);
            (0, channel_id)
        }
        
        CHANNEL_CLOSE => {
            let channel_id = args[0];
            let mut channels = CHANNELS.lock();
            if channels.remove(&channel_id).is_some() {
                (0, 0)
            } else {
                (-1, 0)
            }
        }
        
        CHANNEL_POLL => {
            let channel_id = args[0];
            let channels = CHANNELS.lock();
            match channels.get(&channel_id) {
                Some(ch) => (0, ch.available() as u64),
                None => (-1, 0),
            }
        }
        
        YIELD => {
            // Guest yields its time slice
            // In a real implementation, this would schedule another VM
            (0, 0)
        }
        
        SHUTDOWN => {
            emit_event(VmEventType::Stopped, vm_id, VmEventData::ExitCode(args[0] as i32));
            (-1, 0) // Signal VM should stop
        }
        
        REBOOT => {
            emit_event(VmEventType::Stopped, vm_id, VmEventData::Message(String::from("reboot")));
            (-2, 0) // Signal VM should reboot
        }
        
        DEBUG_PRINT => {
            crate::serial_println!("[VM {} DEBUG] 0x{:X}", vm_id, args[0]);
            (0, 0)
        }
        
        _ => {
            crate::serial_println!("[API] Unknown hypercall 0x{:X} from VM {}", function, vm_id);
            (-1, 0)
        }
    }
}

// ============================================================================
// VERSION AND INFO
// ============================================================================

/// TrustVM version information
pub const VERSION_MAJOR: u32 = 1;
pub const VERSION_MINOR: u32 = 0;
pub const VERSION_PATCH: u32 = 0;
pub const VERSION_STRING: &str = "1.0.0";
pub const BUILD_DATE: &str = "2026-01-31";

/// Get TrustVM version as packed u64: (major << 32) | (minor << 16) | patch
pub fn get_version() -> u64 {
    ((VERSION_MAJOR as u64) << 32) | ((VERSION_MINOR as u64) << 16) | (VERSION_PATCH as u64)
}
