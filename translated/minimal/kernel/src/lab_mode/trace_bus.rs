





extern crate alloc;

use alloc::string::String;
use core::sync::atomic::{AtomicU64, Ordering};
use spin::Mutex;


const KS_: usize = 128;


#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum EventCategory {
    Interrupt = 0,
    Scheduler = 1,
    Memory = 2,
    Au = 3,
    Syscall = 4,
    Keyboard = 5,
    Network = 6,
    Security = 7,
    Custom = 8,
    Hypervisor = 9,
}

impl EventCategory {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Interrupt  => "IRQ",
            Self::Scheduler  => "SCHED",
            Self::Memory     => "MEM",
            Self::Au => "VFS",
            Self::Syscall    => "SYS",
            Self::Keyboard   => "KBD",
            Self::Network    => "NET",
            Self::Security   => "SEC",
            Self::Custom     => "USR",
            Self::Hypervisor => "HV",
        }
    }
    
    pub fn color(&self) -> u32 {
        match self {
            Self::Interrupt  => 0xFFD18616, 
            Self::Scheduler  => 0xFFD29922, 
            Self::Memory     => 0xFF3FB950, 
            Self::Au => 0xFF79C0FF, 
            Self::Syscall    => 0xFFBC8CFF, 
            Self::Keyboard   => 0xFF58A6FF, 
            Self::Network    => 0xFF79C0FF, 
            Self::Security   => 0xFFF85149, 
            Self::Custom     => 0xFFE6EDF3, 
            Self::Hypervisor => 0xFFFF6B6B, 
        }
    }
}


#[derive(Clone)]
pub struct Dw {
    
    pub timestamp_ms: u64,
    
    pub category: EventCategory,
    
    pub message: String,
    
    pub payload: u64,
    
    pub syscall_nr: Option<u64>,
    
    pub syscall_args: Option<[u64; 3]>,
    
    pub syscall_ret: Option<i64>,
}


static HB_: Mutex<EventRing> = Mutex::new(EventRing::new());


static BLH_: AtomicU64 = AtomicU64::new(0);



static FJ_: AtomicU64 = AtomicU64::new(0);

struct EventRing {
    buffer: [Option<Dw>; KS_],
}


impl EventRing {
    const fn new() -> Self {
        
        const Bc: Option<Dw> = None;
        Self {
            buffer: [Bc; KS_],
        }
    }
}





pub fn emit(category: EventCategory, message: String, payload: u64) {
    if !super::EY_.load(Ordering::Relaxed) {
        return; 
    }
    
    let jy = crate::time::uptime_ms();
    let event = Dw {
        timestamp_ms: jy,
        category,
        message,
        payload,
        syscall_nr: None,
        syscall_args: None,
        syscall_ret: None,
    };
    
    let idx = BLH_.fetch_add(1, Ordering::Relaxed) as usize;
    let slot = idx % KS_;
    
    if let Some(mut dq) = HB_.try_lock() {
        dq.buffer[slot] = Some(event);
    }
    FJ_.fetch_add(1, Ordering::Relaxed);
}


pub fn fuj(nr: u64, args: [u64; 3], ret: i64) {
    if !super::EY_.load(Ordering::Relaxed) {
        return;
    }
    
    let jy = crate::time::uptime_ms();
    let name = dfe(nr);
    let message = alloc::format!("{}({:#x}, {:#x}, {:#x}) = {}", name, args[0], args[1], args[2], ret);
    let event = Dw {
        timestamp_ms: jy,
        category: EventCategory::Syscall,
        message,
        payload: nr,
        syscall_nr: Some(nr),
        syscall_args: Some(args),
        syscall_ret: Some(ret),
    };
    
    let idx = BLH_.fetch_add(1, Ordering::Relaxed) as usize;
    let slot = idx % KS_;
    
    if let Some(mut dq) = HB_.try_lock() {
        dq.buffer[slot] = Some(event);
    }
    FJ_.fetch_add(1, Ordering::Relaxed);
}


pub fn dfe(nr: u64) -> &'static str {
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
pub fn bgi(category: EventCategory, bk: &'static str, payload: u64) {
    if !super::EY_.load(Ordering::Relaxed) {
        return;
    }
    emit(category, String::from(bk), payload);
}






pub fn bzg(vm_id: u64, exit_reason: &str, guest_rip: u64, detail: &str) {
    if !super::EY_.load(Ordering::Relaxed) {
        return;
    }
    let bk = alloc::format!("[VM {}] EXIT: {} at RIP=0x{:X} {}", vm_id, exit_reason, guest_rip, detail);
    emit(EventCategory::Hypervisor, bk, vm_id);
}


pub fn bzh(vm_id: u64, event: &str) {
    if !super::EY_.load(Ordering::Relaxed) {
        return;
    }
    let bk = alloc::format!("[VM {}] {}", vm_id, event);
    emit(EventCategory::Hypervisor, bk, vm_id);
}


pub fn hvk(vm_id: u64, direction: &str, port: u16, value: u64) {
    if !super::EY_.load(Ordering::Relaxed) {
        return;
    }
    let bk = alloc::format!("[VM {}] IO {} port=0x{:X} val=0x{:X}", vm_id, direction, port, value);
    emit(EventCategory::Hypervisor, bk, vm_id);
}


pub fn hvl(vm_id: u64, event_type: &str, zy: u64, ua: u64) {
    if !super::EY_.load(Ordering::Relaxed) {
        return;
    }
    let bk = alloc::format!("[VM {}] MEM {} GPA=0x{:X} info=0x{:X}", vm_id, event_type, zy, ua);
    emit(EventCategory::Hypervisor, bk, vm_id);
}


pub fn hvm(vm_id: u64, rax: u64, rbx: u64, rcx: u64, rdx: u64, rip: u64, rsp: u64) {
    if !super::EY_.load(Ordering::Relaxed) {
        return;
    }
    let bk = alloc::format!(
        "[VM {}] REGS RAX=0x{:X} RBX=0x{:X} RCX=0x{:X} RDX=0x{:X} RIP=0x{:X} RSP=0x{:X}",
        vm_id, rax, rbx, rcx, rdx, rip, rsp
    );
    emit(EventCategory::Hypervisor, bk, vm_id);
}


pub fn ocz(count: usize) -> alloc::vec::Vec<Dw> {
    let mut result = alloc::vec::Vec::new();
    let av = FJ_.load(Ordering::Relaxed) as usize;
    if av == 0 {
        return result;
    }
    
    let start = if av > count { av - count } else { 0 };
    let dq = HB_.lock();
    
    for i in start..av {
        let slot = i % KS_;
        if let Some(ref event) = dq.buffer[slot] {
            result.push(event.clone());
        }
    }
    
    result
}


pub fn total_count() -> u64 {
    FJ_.load(Ordering::Relaxed)
}


pub fn dxk(since_idx: u64, max: usize) -> (alloc::vec::Vec<Dw>, u64) {
    let av = FJ_.load(Ordering::Relaxed);
    if av <= since_idx {
        return (alloc::vec::Vec::new(), av);
    }
    
    let mut result = alloc::vec::Vec::new();
    let start = since_idx as usize;
    let end = av as usize;
    let dq = HB_.lock();
    
    let jtp = if end - start > max { end - max } else { start };
    for i in jtp..end {
        let slot = i % KS_;
        if let Some(ref event) = dq.buffer[slot] {
            result.push(event.clone());
        }
    }
    
    (result, av)
}
