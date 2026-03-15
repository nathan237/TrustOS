




use spin::Mutex;
use core::sync::atomic::{AtomicU64, AtomicBool, Ordering};


const EU_: usize = 1024;


static AJB_: Mutex<[TraceEvent; EU_]> = 
    Mutex::new([TraceEvent::Y; EU_]);


static YR_: AtomicU64 = AtomicU64::new(0);


static XS_: AtomicBool = AtomicBool::new(true);


static SP_: AtomicBool = AtomicBool::new(false);


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum EventType {
    None = 0,
    Ano = 1,
    Caa = 2,
    Dac = 3,
    Dab = 4,
    Mb = 5,
    Fv = 6,
    Ani = 7,
    Azp = 8,
    Cms = 9,
    Dck = 10,
    Dcm = 11,
    Dau = 12,
    Gv = 255,
}


#[derive(Debug, Clone, Copy)]
pub struct TraceEvent {
    
    pub aea: u64,
    
    pub qq: u8,
    
    pub bqo: EventType,
    
    pub ew: u64,
}

impl TraceEvent {
    pub const Y: Self = Self {
        aea: 0,
        qq: 0,
        bqo: EventType::None,
        ew: 0,
    };
}


pub fn init() {
    
    
    #[cfg(feature = "deterministic")]
    {
        SP_.store(true, Ordering::SeqCst);
        crate::log_debug!("Deterministic mode ENABLED");
    }
    
    crate::log_debug!("Trace buffer initialized ({} entries)", EU_);
}


pub fn bry(bqo: EventType, ew: u64) {
    if !XS_.load(Ordering::Relaxed) {
        return;
    }
    
    let aea = crate::logger::fjp();
    let qq = 0; 
    
    let id = TraceEvent {
        aea,
        qq,
        bqo,
        ew,
    };
    
    let index = YR_.fetch_add(1, Ordering::Relaxed) as usize;
    let gk = index % EU_;
    
    AJB_.lock()[gk] = id;
}


pub fn cuf(iq: bool) {
    XS_.store(iq, Ordering::SeqCst);
}


pub fn zu() -> bool {
    XS_.load(Ordering::Relaxed)
}


pub fn ypc() {
    SP_.store(true, Ordering::SeqCst);
    crate::log!("Deterministic mode ENABLED");
}


pub fn yzh() -> bool {
    SP_.load(Ordering::Relaxed)
}


pub fn ynw() {
    let bi = AJB_.lock();
    let cv = YR_.load(Ordering::Relaxed) as usize;
    
    crate::serial::elt(format_args!("\n=== TRACE DUMP (last 32 events) ===\n"));
    
    
    let ay = cv.ao(32);
    for a in ay..cv {
        let id = &bi[a % EU_];
        if id.bqo != EventType::None {
            crate::serial::elt(format_args!(
                "[{:>10}][CPU{}] {:?} payload={:#x}\n",
                id.aea,
                id.qq,
                id.bqo,
                id.ew
            ));
        }
    }
    
    crate::serial::elt(format_args!("=== END TRACE DUMP ===\n"));
}


pub fn ypu() -> alloc::vec::Vec<TraceEvent> {
    let bi = AJB_.lock();
    let cv = YR_.load(Ordering::Relaxed) as usize;
    let az = cv.v(EU_);
    
    let mut events = alloc::vec::Vec::fc(az);
    let ay = if cv > EU_ {
        cv - EU_
    } else {
        0
    };
    
    for a in ay..cv {
        events.push(bi[a % EU_]);
    }
    
    events
}


pub fn cm() -> Buo {
    Buo {
        nrh: YR_.load(Ordering::Relaxed),
        kfi: EU_,
        xlc: XS_.load(Ordering::Relaxed),
        rwx: SP_.load(Ordering::Relaxed),
    }
}


#[derive(Debug, Clone)]
pub struct Buo {
    pub nrh: u64,
    pub kfi: usize,
    pub xlc: bool,
    pub rwx: bool,
}
