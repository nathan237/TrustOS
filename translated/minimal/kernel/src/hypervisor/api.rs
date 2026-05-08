







use alloc::string::String;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use spin::Mutex;
use core::sync::atomic::{AtomicU64, Ordering};






#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u64)]
pub enum Capability {
    
    VmxEnabled = 1 << 0,
    
    EptSupported = 1 << 1,
    
    VpidSupported = 1 << 2,
    
    UnrestrictedGuest = 1 << 3,
    
    VmcsShadowing = 1 << 4,
    
    PostedInterrupts = 1 << 5,
    
    EptAccessDirty = 1 << 6,
    
    VirtualConsole = 1 << 7,
    
    SharedFilesystem = 1 << 8,
    
    NestedVirtualization = 1 << 9,
    
    LiveMigration = 1 << 10,
    
    MemoryBallooning = 1 << 11,
    
    DevicePassthrough = 1 << 12,
}


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
    
    pub fn qki(&self, cap: Capability) -> bool {
        (self.bits & (cap as u64)) != 0
    }
    
    pub fn as_u64(&self) -> u64 {
        self.bits
    }
    
    pub fn qgo(bits: u64) -> Self {
        Capabilities { bits }
    }
}


pub fn enz() -> Capabilities {
    let mut caps = Capabilities::new();
    
    
    if let Ok(vmx_caps) = super::vmx::ehv() {
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
    
    
    caps.set(Capability::VirtualConsole);
    caps.set(Capability::SharedFilesystem);
    
    caps
}






#[derive(Debug, Clone)]
pub struct Bfj {
    pub id: u64,
    pub name: String,
    pub state: Art,
    pub memory_mb: usize,
    pub vcpus: usize,
    pub uptime_ms: u64,
    pub stats: Aru,
    pub isolation: Alx,
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Art {
    Created,
    Initializing,
    Running,
    Paused,
    Stopping,
    Stopped,
    Crashed,
    Migrating,
}


#[derive(Debug, Clone, Default)]
pub struct Aru {
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


#[derive(Debug, Clone)]
pub struct Alx {
    pub vpid: Option<u16>,
    pub ept_enabled: bool,
    pub ept_pages: usize,
    pub execute_only_supported: bool,
    pub supervisor_shadow_stack: bool,
}






#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u32)]
pub enum VmEventType {
    
    Created = 0,
    
    Started = 1,
    
    Paused = 2,
    
    Resumed = 3,
    
    Stopped = 4,
    
    Crashed = 5,
    
    ConsoleOutput = 6,
    
    Hypercall = 7,
    
    Ev = 8,
    
    MemoryWarning = 9,
    
    VpidFlush = 10,
}


#[derive(Debug, Clone)]
pub struct Je {
    pub event_type: VmEventType,
    pub vm_id: u64,
    pub timestamp_ms: u64,
    pub data: VmEventData,
}


#[derive(Debug, Clone)]
pub enum VmEventData {
    None,
    ExitCode(i32),
    Az(String),
    Address(u64),
    HypercallInfo { function: u64, result: i64 },
}


pub type Yl = fn(event: &Je);


struct Ym {
    callback: Yl,
    vm_filter: Option<u64>, 
    event_filter: Option<VmEventType>, 
}

static ATN_: Mutex<Vec<Ym>> = Mutex::new(Vec::new());
static ATM_: Mutex<Vec<Je>> = Mutex::new(Vec::new());
static BXJ_: AtomicU64 = AtomicU64::new(0);


pub fn qxu(
    callback: Yl,
    vm_filter: Option<u64>,
    event_filter: Option<VmEventType>,
) -> u64 {
    let sub = Ym {
        callback,
        vm_filter,
        event_filter,
    };
    
    let mut gwr = ATN_.lock();
    gwr.push(sub);
    BXJ_.fetch_add(1, Ordering::SeqCst)
}


pub fn bzf(event_type: VmEventType, vm_id: u64, data: VmEventData) {
    let event = Je {
        event_type,
        vm_id,
        timestamp_ms: crate::time::uptime_ms(),
        data,
    };
    
    
    {
        let mut log = ATM_.lock();
        if log.len() >= 1000 {
            log.remove(0); 
        }
        log.push(event.clone());
    }
    
    
    let gwr = ATN_.lock();
    for sub in gwr.iter() {
        
        if let Some(filter_vm) = sub.vm_filter {
            if filter_vm != vm_id {
                continue;
            }
        }
        
        
        if let Some(fwx) = sub.event_filter {
            if fwx != event_type {
                continue;
            }
        }
        
        
        (sub.callback)(&event);
    }
}


pub fn mdr(count: usize) -> Vec<Je> {
    let log = ATM_.lock();
    let start = if log.len() > count { log.len() - count } else { 0 };
    log[start..].to_vec()
}






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
    
    
    pub fn send(&mut self, data: &[u8]) -> Result<usize, &'static str> {
        let available = self.max_size - self.tx_buffer.len();
        let gze = data.len().min(available);
        
        if gze == 0 {
            return Err("Channel buffer full");
        }
        
        self.tx_buffer.extend_from_slice(&data[..gze]);
        Ok(gze)
    }
    
    
    pub fn recv(&mut self, buf: &mut [u8]) -> usize {
        let rz = buf.len().min(self.rx_buffer.len());
        buf[..rz].copy_from_slice(&self.rx_buffer[..rz]);
        self.rx_buffer.drain(..rz);
        rz
    }
    
    
    pub fn available(&self) -> usize {
        self.rx_buffer.len()
    }
    
    
    pub fn space(&self) -> usize {
        self.max_size - self.tx_buffer.len()
    }
}


static Bq: Mutex<BTreeMap<u64, VmChannel>> = Mutex::new(BTreeMap::new());
static BPA_: AtomicU64 = AtomicU64::new(0);


pub fn ejc(vm_id: u64, name: &str, max_size: usize) -> u64 {
    let id = BPA_.fetch_add(1, Ordering::SeqCst);
    let channel = VmChannel::new(id, vm_id, name, max_size);
    
    Bq.lock().insert(id, channel);
    
    crate::serial_println!("[API] Created channel {} for VM {}: {}", id, vm_id, name);
    id
}


pub fn hkj(ath: u64, data: &[u8]) -> Result<usize, &'static str> {
    let mut channels = Bq.lock();
    match channels.get_mut(&ath) {
        Some(channel) => channel.send(data),
        None => Err("Channel not found"),
    }
}


pub fn pzj(ath: u64, buf: &mut [u8]) -> Result<usize, &'static str> {
    let mut channels = Bq.lock();
    match channels.get_mut(&ath) {
        Some(channel) => Ok(channel.recv(buf)),
        None => Err("Channel not found"),
    }
}






#[derive(Debug, Clone)]
pub struct ResourceQuota {
    
    pub max_memory: usize,
    
    pub max_vcpus: usize,
    
    pub max_io_bandwidth: usize,
    
    pub max_hypercalls_per_sec: usize,
    
    pub cpu_limit_percent: u8,
}

impl Default for ResourceQuota {
    fn default() -> Self {
        ResourceQuota {
            max_memory: 256 * 1024 * 1024, 
            max_vcpus: 4,
            max_io_bandwidth: 0,
            max_hypercalls_per_sec: 10000,
            cpu_limit_percent: 0,
        }
    }
}

static BKW_: Mutex<BTreeMap<u64, ResourceQuota>> = Mutex::new(BTreeMap::new());


pub fn opj(vm_id: u64, gph: ResourceQuota) {
    BKW_.lock().insert(vm_id, gph);
}


pub fn qig(vm_id: u64) -> Option<ResourceQuota> {
    BKW_.lock().get(&vm_id).cloned()
}






pub mod hypercall {
    
    pub const Bam: u64 = 0x00;
    pub const Oq: u64 = 0x01;
    pub const DQU_: u64 = 0x02;
    pub const DKH_: u64 = 0x03;
    pub const DKG_: u64 = 0x04;
    
    
    pub const ENJ_: u64 = 0x100;
    
    
    pub const BZY_: u64 = 0x200;
    pub const CAI_: u64 = 0x201;
    pub const CAG_: u64 = 0x202;
    pub const BPB_: u64 = 0x210;
    pub const BOZ_: u64 = 0x211;
    pub const DII_: u64 = 0x212;
    pub const DIH_: u64 = 0x213;
    pub const BPC_: u64 = 0x214;
    pub const EJS_: u64 = 0x220;
    pub const EGE_: u64 = 0x221;
    pub const Asg: u64 = 0x222;
    pub const Iu: u64 = 0x223;
    pub const Um: u64 = 0x224;
    
    
    pub const DML_: u64 = 0x300;
    pub const BTW_: u64 = 0x301;
    pub const DMM_: u64 = 0x302;
}


pub fn mhg(vm_id: u64, function: u64, args: &[u64; 4]) -> (i64, u64) {
    use hypercall::*;
    
    match function {
        BZY_ => {
            let caps = enz();
            (0, caps.as_u64())
        }
        
        CAI_ => {
            
            
            (0, vm_id)
        }
        
        CAG_ => {
            
            (0, 0)
        }
        
        BPB_ => {
            
            
            let ath = ejc(vm_id, "guest_channel", 4096);
            (0, ath)
        }
        
        BOZ_ => {
            let ath = args[0];
            let mut channels = Bq.lock();
            if channels.remove(&ath).is_some() {
                (0, 0)
            } else {
                (-1, 0)
            }
        }
        
        BPC_ => {
            let ath = args[0];
            let channels = Bq.lock();
            match channels.get(&ath) {
                Some(ch) => (0, ch.available() as u64),
                None => (-1, 0),
            }
        }
        
        Asg => {
            
            
            (0, 0)
        }
        
        Iu => {
            bzf(VmEventType::Stopped, vm_id, VmEventData::ExitCode(args[0] as i32));
            (-1, 0) 
        }
        
        Um => {
            bzf(VmEventType::Stopped, vm_id, VmEventData::Az(String::from("reboot")));
            (-2, 0) 
        }
        
        BTW_ => {
            crate::serial_println!("[VM {} DEBUG] 0x{:X}", vm_id, args[0]);
            (0, 0)
        }
        
        _ => {
            crate::serial_println!("[API] Unknown hypercall 0x{:X} from VM {}", function, vm_id);
            (-1, 0)
        }
    }
}






pub const DEA_: u32 = 1;
pub const DEB_: u32 = 0;
pub const DEC_: u32 = 0;
pub const DED_: &str = "1.0.0";
pub const DHV_: &str = "2026-01-31";


pub fn qiv() -> u64 {
    ((DEA_ as u64) << 32) | ((DEB_ as u64) << 16) | (DEC_ as u64)
}
