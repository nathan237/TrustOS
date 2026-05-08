


use alloc::string::String;
use alloc::vec::Vec;


pub mod syscall {
    pub const Ba: u64 = 0;
    pub const Bh: u64 = 1;
    pub const Aby: u64 = 2;
    pub const Rf: u64 = 3;
    pub const Ael: u64 = 4;
    pub const Yq: u64 = 5;
    pub const Aas: u64 = 6;
    pub const Ace: u64 = 7;
    pub const Aaq: u64 = 8;
    pub const Tn: u64 = 9;
    pub const Abc: u64 = 10;
    pub const Tp: u64 = 11;
    pub const Rd: u64 = 12;
    pub const Zy: u64 = 16;
    pub const Agm: u64 = 20;
    pub const Qy: u64 = 21;
    pub const Xo: u64 = 32;
    pub const Xp: u64 = 33;
    pub const So: u64 = 39;
    pub const Yo: u64 = 57;
    pub const Yg: u64 = 59;
    pub const Oq: u64 = 60;
    pub const Vq: u64 = 63;
    pub const Sk: u64 = 79;
    pub const Wv: u64 = 80;
    pub const Aba: u64 = 83;
    pub const Aoh: u64 = 84;
    pub const Afq: u64 = 87;
    pub const Acx: u64 = 89;
    pub const Sp: u64 = 102;
    pub const Sn: u64 = 104;
    pub const Sm: u64 = 107;
    pub const Sl: u64 = 108;
    pub const AAK_: u64 = 158;
    pub const AJR_: u64 = 218;
    pub const ADN_: u64 = 231;
    pub const Abz: u64 = 257;
    pub const Abp: u64 = 262;
}


pub struct SyscallResult {
    pub rax: i64,
    pub errno: i32,
}

impl SyscallResult {
    pub fn success(value: i64) -> Self {
        Self { rax: value, errno: 0 }
    }
    
    pub fn error(errno: i32) -> Self {
        Self { rax: -errno as i64, errno }
    }
}


pub fn qes(
    num: u64,
    rdi: u64,
    rsi: u64,
    rdx: u64,
    r10: u64,
    r8: u64,
    r9: u64,
) -> SyscallResult {
    match num {
        syscall::Bh => lpn(rdi as i32, rsi as *const u8, rdx as usize),
        syscall::Ba => lpl(rdi as i32, rsi as *mut u8, rdx as usize),
        syscall::Oq | syscall::ADN_ => lpg(rdi as i32),
        syscall::So => lpi(),
        syscall::Sk => lph(rdi as *mut u8, rsi as usize),
        syscall::Vq => lpm(rdi as *mut Afx),
        syscall::Rd => lpf(rdi),
        syscall::Tn => lpj(rdi, rsi, rdx as i32, r10 as i32, r8 as i32, r9 as i64),
        syscall::Tp => lpk(rdi, rsi),
        syscall::AAK_ => lpe(rdi as i32, rsi),
        syscall::AJR_ => SyscallResult::success(1), 
        syscall::Sp | syscall::Sm => SyscallResult::success(0), 
        syscall::Sn | syscall::Sl => SyscallResult::success(0), 
        _ => {
            
            SyscallResult::error(38) 
        }
    }
}

fn lpn(fd: i32, buf: *const u8, count: usize) -> SyscallResult {
    if buf.is_null() {
        return SyscallResult::error(14); 
    }
    
    match fd {
        1 | 2 => {
            
            let slice = unsafe { core::slice::from_raw_parts(buf, count) };
            if let Ok(j) = core::str::from_utf8(slice) {
                crate::print!("{}", j);
            } else {
                
                for &b in slice {
                    if b >= 0x20 && b < 0x7F {
                        crate::print!("{}", b as char);
                    } else {
                        crate::print!(".");
                    }
                }
            }
            SyscallResult::success(count as i64)
        }
        _ => SyscallResult::error(9) 
    }
}

fn lpl(fd: i32, buf: *mut u8, count: usize) -> SyscallResult {
    if buf.is_null() {
        return SyscallResult::error(14); 
    }
    
    match fd {
        0 => {
            
            SyscallResult::success(0) 
        }
        _ => SyscallResult::error(9) 
    }
}

fn lpg(code: i32) -> SyscallResult {
    crate::println!("[transpiled process exited with code {}]", code);
    SyscallResult::success(0)
}

fn lpi() -> SyscallResult {
    SyscallResult::success(1) 
}

fn lph(buf: *mut u8, size: usize) -> SyscallResult {
    if buf.is_null() || size == 0 {
        return SyscallResult::error(14); 
    }
    
    let cwd = b"/\0";
    if size < cwd.len() {
        return SyscallResult::error(34); 
    }
    
    unsafe {
        core::ptr::copy_nonoverlapping(cwd.as_ptr(), buf, cwd.len());
    }
    
    SyscallResult::success(buf as i64)
}

#[repr(C)]
pub struct Afx {
    pub sysname: [u8; 65],
    pub nodename: [u8; 65],
    pub release: [u8; 65],
    pub version: [u8; 65],
    pub machine: [u8; 65],
    pub domainname: [u8; 65],
}

fn lpm(buf: *mut Afx) -> SyscallResult {
    if buf.is_null() {
        return SyscallResult::error(14); 
    }
    
    unsafe {
        let info = &mut *buf;
        cvp(&mut info.sysname, "TrustOS");
        cvp(&mut info.nodename, "trustos");
        cvp(&mut info.release, "1.0.0-transpiled");
        cvp(&mut info.version, "#1 SMP TrustOS Kernel");
        cvp(&mut info.machine, "x86_64");
        cvp(&mut info.domainname, "(none)");
    }
    
    SyscallResult::success(0)
}

fn cvp(mt: &mut [u8; 65], src: &str) {
    let bytes = src.as_bytes();
    let len = bytes.len().min(64);
    mt[..len].copy_from_slice(&bytes[..len]);
    mt[len] = 0;
}

fn lpf(addr: u64) -> SyscallResult {
    
    static mut SS_: u64 = 0x10000000;
    
    unsafe {
        if addr == 0 {
            SyscallResult::success(SS_ as i64)
        } else if addr > SS_ {
            SS_ = addr;
            SyscallResult::success(addr as i64)
        } else {
            SyscallResult::success(SS_ as i64)
        }
    }
}

fn lpj(addr: u64, len: u64, prot: i32, flags: i32, fd: i32, offset: i64) -> SyscallResult {
    
    if fd != -1 {
        return SyscallResult::error(22); 
    }
    
    static mut BCS_: u64 = 0x20000000;
    
    unsafe {
        let result = BCS_;
        BCS_ += (len + 0xFFF) & !0xFFF; 
        SyscallResult::success(result as i64)
    }
}

fn lpk(addr: u64, len: u64) -> SyscallResult {
    
    SyscallResult::success(0)
}

fn lpe(code: i32, addr: u64) -> SyscallResult {
    const MQ_: i32 = 0x1002;
    const MP_: i32 = 0x1003;
    
    match code {
        MQ_ => {
            
            SyscallResult::success(0)
        }
        MP_ => {
            SyscallResult::success(0)
        }
        _ => SyscallResult::error(22) 
    }
}


pub fn qfi<F: FnOnce() -> i32>(f: F) -> i32 {
    f()
}
