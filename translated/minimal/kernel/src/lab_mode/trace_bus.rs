





extern crate alloc;

use alloc::string::String;
use core::sync::atomic::{AtomicU64, Ordering};
use spin::Mutex;


const JY_: usize = 128;


#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum EventCategory {
    Fv = 0,
    Scheduler = 1,
    Cy = 2,
    Cc = 3,
    Hg = 4,
    Hs = 5,
    As = 6,
    De = 7,
    Gv = 8,
    Ee = 9,
}

impl EventCategory {
    pub fn cu(&self) -> &'static str {
        match self {
            Self::Fv  => "IRQ",
            Self::Scheduler  => "SCHED",
            Self::Cy     => "MEM",
            Self::Cc => "VFS",
            Self::Hg    => "SYS",
            Self::Hs   => "KBD",
            Self::As    => "NET",
            Self::De   => "SEC",
            Self::Gv     => "USR",
            Self::Ee => "HV",
        }
    }
    
    pub fn s(&self) -> u32 {
        match self {
            Self::Fv  => 0xFFD18616, 
            Self::Scheduler  => 0xFFD29922, 
            Self::Cy     => 0xFF3FB950, 
            Self::Cc => 0xFF79C0FF, 
            Self::Hg    => 0xFFBC8CFF, 
            Self::Hs   => 0xFF58A6FF, 
            Self::As    => 0xFF79C0FF, 
            Self::De   => 0xFFF85149, 
            Self::Gv     => 0xFFE6EDF3, 
            Self::Ee => 0xFFFF6B6B, 
        }
    }
}


#[derive(Clone)]
pub struct Jq {
    
    pub aet: u64,
    
    pub gb: EventCategory,
    
    pub message: String,
    
    pub ew: u64,
    
    pub fvy: Option<u64>,
    
    pub mjg: Option<[u64; 3]>,
    
    pub mjh: Option<i64>,
}


static GK_: Mutex<EventRing> = Mutex::new(EventRing::new());


static BJC_: AtomicU64 = AtomicU64::new(0);



static ET_: AtomicU64 = AtomicU64::new(0);

struct EventRing {
    bi: [Option<Jq>; JY_],
}


impl EventRing {
    const fn new() -> Self {
        
        const Cq: Option<Jq> = None;
        Self {
            bi: [Cq; JY_],
        }
    }
}





pub fn fj(gb: EventCategory, message: String, ew: u64) {
    if !super::EK_.load(Ordering::Relaxed) {
        return; 
    }
    
    let wi = crate::time::lc();
    let id = Jq {
        aet: wi,
        gb,
        message,
        ew,
        fvy: None,
        mjg: None,
        mjh: None,
    };
    
    let w = BJC_.fetch_add(1, Ordering::Relaxed) as usize;
    let gk = w % JY_;
    
    if let Some(mut mz) = GK_.try_lock() {
        mz.bi[gk] = Some(id);
    }
    ET_.fetch_add(1, Ordering::Relaxed);
}


pub fn ktb(nr: u64, n: [u64; 3], aux: i64) {
    if !super::EK_.load(Ordering::Relaxed) {
        return;
    }
    
    let wi = crate::time::lc();
    let j = gty(nr);
    let message = alloc::format!("{}({:#x}, {:#x}, {:#x}) = {}", j, n[0], n[1], n[2], aux);
    let id = Jq {
        aet: wi,
        gb: EventCategory::Hg,
        message,
        ew: nr,
        fvy: Some(nr),
        mjg: Some(n),
        mjh: Some(aux),
    };
    
    let w = BJC_.fetch_add(1, Ordering::Relaxed) as usize;
    let gk = w % JY_;
    
    if let Some(mut mz) = GK_.try_lock() {
        mz.bi[gk] = Some(id);
    }
    ET_.fetch_add(1, Ordering::Relaxed);
}


pub fn gty(nr: u64) -> &'static str {
    match nr {
        0 => "read",
        1 => "write",
        2 => "open",
        3 => "close",
        4 => "stat",
        5 => "fstat",
        6 => "lstat",
        7 => "poll",
        8 => "lseek",
        9 => "mmap",
        10 => "mprotect",
        11 => "munmap",
        12 => "brk",
        13 => "rt_sigaction",
        14 => "rt_sigprocmask",
        20 => "writev",
        21 => "access",
        24 => "sched_yield",
        35 => "nanosleep",
        39 => "getpid",
        41 => "socket",
        42 => "connect",
        43 => "accept",
        44 => "sendto",
        45 => "recvfrom",
        48 => "shutdown",
        49 => "bind",
        50 => "listen",
        56 => "clone",
        57 => "fork",
        58 => "vfork",
        59 => "execve",
        60 => "exit",
        61 => "wait4",
        62 => "kill",
        63 => "uname",
        72 => "fcntl",
        79 => "getcwd",
        80 => "chdir",
        83 => "mkdir",
        87 => "unlink",
        89 => "readlink",
        96 => "gettid",
        102 => "getuid",
        104 => "getgid",
        107 => "geteuid",
        108 => "getegid",
        110 => "getppid",
        158 => "arch_prctl",
        218 => "set_tid_address",
        228 => "clock_gettime",
        231 => "exit_group",
        273 => "set_robust_list",
        274 => "get_robust_list",
        302 => "prlimit64",
        318 => "getrandom",
        0x1000 => "debug_print",
        0x1001 => "ipc_send",
        0x1002 => "ipc_recv",
        0x1003 => "ipc_create",
        _ => "unknown",
    }
}


#[inline]
pub fn dgy(gb: EventCategory, fr: &'static str, ew: u64) {
    if !super::EK_.load(Ordering::Relaxed) {
        return;
    }
    fj(gb, String::from(fr), ew);
}






pub fn ept(fk: u64, exit_reason: &str, wb: u64, eu: &str) {
    if !super::EK_.load(Ordering::Relaxed) {
        return;
    }
    let fr = alloc::format!("[VM {}] EXIT: {} at RIP=0x{:X} {}", fk, exit_reason, wb, eu);
    fj(EventCategory::Ee, fr, fk);
}


pub fn epu(fk: u64, id: &str) {
    if !super::EK_.load(Ordering::Relaxed) {
        return;
    }
    let fr = alloc::format!("[VM {}] {}", fk, id);
    fj(EventCategory::Ee, fr, fk);
}


pub fn npq(fk: u64, sz: &str, port: u16, bn: u64) {
    if !super::EK_.load(Ordering::Relaxed) {
        return;
    }
    let fr = alloc::format!("[VM {}] IO {} port=0x{:X} val=0x{:X}", fk, sz, port, bn);
    fj(EventCategory::Ee, fr, fk);
}


pub fn npr(fk: u64, bqo: &str, axy: u64, ang: u64) {
    if !super::EK_.load(Ordering::Relaxed) {
        return;
    }
    let fr = alloc::format!("[VM {}] MEM {} GPA=0x{:X} info=0x{:X}", fk, bqo, axy, ang);
    fj(EventCategory::Ee, fr, fk);
}


pub fn nps(fk: u64, rax: u64, rbx: u64, rcx: u64, rdx: u64, pc: u64, rsp: u64) {
    if !super::EK_.load(Ordering::Relaxed) {
        return;
    }
    let fr = alloc::format!(
        "[VM {}] REGS RAX=0x{:X} RBX=0x{:X} RCX=0x{:X} RDX=0x{:X} RIP=0x{:X} RSP=0x{:X}",
        fk, rax, rbx, rcx, rdx, pc, rsp
    );
    fj(EventCategory::Ee, fr, fk);
}


pub fn vsi(az: usize) -> alloc::vec::Vec<Jq> {
    let mut result = alloc::vec::Vec::new();
    let es = ET_.load(Ordering::Relaxed) as usize;
    if es == 0 {
        return result;
    }
    
    let ay = if es > az { es - az } else { 0 };
    let mz = GK_.lock();
    
    for a in ay..es {
        let gk = a % JY_;
        if let Some(ref id) = mz.bi[gk] {
            result.push(id.clone());
        }
    }
    
    result
}


pub fn cus() -> u64 {
    ET_.load(Ordering::Relaxed)
}


pub fn hxa(plb: u64, am: usize) -> (alloc::vec::Vec<Jq>, u64) {
    let es = ET_.load(Ordering::Relaxed);
    if es <= plb {
        return (alloc::vec::Vec::new(), es);
    }
    
    let mut result = alloc::vec::Vec::new();
    let ay = plb as usize;
    let ci = es as usize;
    let mz = GK_.lock();
    
    let qfc = if ci - ay > am { ci - am } else { ay };
    for a in qfc..ci {
        let gk = a % JY_;
        if let Some(ref id) = mz.bi[gk] {
            result.push(id.clone());
        }
    }
    
    (result, es)
}
