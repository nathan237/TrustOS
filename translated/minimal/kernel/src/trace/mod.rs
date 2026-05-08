




use spin::Mutex;
use core::sync::atomic::{AtomicU64, AtomicBool, Ordering};


const FK_: usize = 1024;


static AKX_: Mutex<[TraceEvent; FK_]> = 
    Mutex::new([TraceEvent::Q; FK_]);


static ZW_: AtomicU64 = AtomicU64::new(0);


static YZ_: AtomicBool = AtomicBool::new(true);


static TW_: AtomicBool = AtomicBool::new(false);


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum EventType {
    None = 0,
    TimerTick = 1,
    ContextSwitch = 2,
    IpcSend = 3,
    IpcReceive = 4,
    PageFault = 5,
    Interrupt = 6,
    SyscallEntry = 7,
    SyscallExit = 8,
    SecurityViolation = 9,
    MemoryAlloc = 10,
    MemoryFree = 11,
    KeyboardInput = 12,
    Custom = 255,
}


#[derive(Debug, Clone, Copy)]
pub struct TraceEvent {
    
    pub timestamp: u64,
    
    pub cpu_id: u8,
    
    pub event_type: EventType,
    
    pub payload: u64,
}

impl TraceEvent {
    pub const Q: Self = Self {
        timestamp: 0,
        cpu_id: 0,
        event_type: EventType::None,
        payload: 0,
    };
}


pub fn init() {
    
    
    #[cfg(feature = "deterministic")]
    {
        TW_.store(true, Ordering::SeqCst);
        crate::log_debug!("Deterministic mode ENABLED");
    }
    
    crate::log_debug!("Trace buffer initialized ({} entries)", FK_);
}


pub fn akj(event_type: EventType, payload: u64) {
    if !YZ_.load(Ordering::Relaxed) {
        return;
    }
    
    let timestamp = crate::logger::ckc();
    let cpu_id = 0; 
    
    let event = TraceEvent {
        timestamp,
        cpu_id,
        event_type,
        payload,
    };
    
    let index = ZW_.fetch_add(1, Ordering::Relaxed) as usize;
    let slot = index % FK_;
    
    AKX_.lock()[slot] = event;
}


pub fn set_enabled(enabled: bool) {
    YZ_.store(enabled, Ordering::SeqCst);
}


pub fn lq() -> bool {
    YZ_.load(Ordering::Relaxed)
}


pub fn qeu() {
    TW_.store(true, Ordering::SeqCst);
    crate::log!("Deterministic mode ENABLED");
}


pub fn qmh() -> bool {
    TW_.load(Ordering::Relaxed)
}


pub fn qeg() {
    let buffer = AKX_.lock();
    let current = ZW_.load(Ordering::Relaxed) as usize;
    
    crate::serial::bxg(format_args!("\n=== TRACE DUMP (last 32 events) ===\n"));
    
    
    let start = current.saturating_sub(32);
    for i in start..current {
        let event = &buffer[i % FK_];
        if event.event_type != EventType::None {
            crate::serial::bxg(format_args!(
                "[{:>10}][CPU{}] {:?} payload={:#x}\n",
                event.timestamp,
                event.cpu_id,
                event.event_type,
                event.payload
            ));
        }
    }
    
    crate::serial::bxg(format_args!("=== END TRACE DUMP ===\n"));
}


pub fn qfj() -> alloc::vec::Vec<TraceEvent> {
    let buffer = AKX_.lock();
    let current = ZW_.load(Ordering::Relaxed) as usize;
    let count = current.min(FK_);
    
    let mut events = alloc::vec::Vec::with_capacity(count);
    let start = if current > FK_ {
        current - FK_
    } else {
        0
    };
    
    for i in start..current {
        events.push(buffer[i % FK_]);
    }
    
    events
}


pub fn stats() -> Afk {
    Afk {
        events_recorded: ZW_.load(Ordering::Relaxed),
        fkb: FK_,
        tracing_enabled: YZ_.load(Ordering::Relaxed),
        deterministic_mode: TW_.load(Ordering::Relaxed),
    }
}


#[derive(Debug, Clone)]
pub struct Afk {
    pub events_recorded: u64,
    pub fkb: usize,
    pub tracing_enabled: bool,
    pub deterministic_mode: bool,
}
