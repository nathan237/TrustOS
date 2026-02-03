// TrustOS Transpiler - Runtime Support
// Provides syscall emulation for transpiled Linux binaries

use alloc::string::String;
use alloc::vec::Vec;

/// Linux syscall numbers (x86_64)
pub mod syscall {
    pub const READ: u64 = 0;
    pub const WRITE: u64 = 1;
    pub const OPEN: u64 = 2;
    pub const CLOSE: u64 = 3;
    pub const STAT: u64 = 4;
    pub const FSTAT: u64 = 5;
    pub const LSTAT: u64 = 6;
    pub const POLL: u64 = 7;
    pub const LSEEK: u64 = 8;
    pub const MMAP: u64 = 9;
    pub const MPROTECT: u64 = 10;
    pub const MUNMAP: u64 = 11;
    pub const BRK: u64 = 12;
    pub const IOCTL: u64 = 16;
    pub const WRITEV: u64 = 20;
    pub const ACCESS: u64 = 21;
    pub const DUP: u64 = 32;
    pub const DUP2: u64 = 33;
    pub const GETPID: u64 = 39;
    pub const FORK: u64 = 57;
    pub const EXECVE: u64 = 59;
    pub const EXIT: u64 = 60;
    pub const UNAME: u64 = 63;
    pub const GETCWD: u64 = 79;
    pub const CHDIR: u64 = 80;
    pub const MKDIR: u64 = 83;
    pub const RMDIR: u64 = 84;
    pub const UNLINK: u64 = 87;
    pub const READLINK: u64 = 89;
    pub const GETUID: u64 = 102;
    pub const GETGID: u64 = 104;
    pub const GETEUID: u64 = 107;
    pub const GETEGID: u64 = 108;
    pub const ARCH_PRCTL: u64 = 158;
    pub const SET_TID_ADDRESS: u64 = 218;
    pub const EXIT_GROUP: u64 = 231;
    pub const OPENAT: u64 = 257;
    pub const NEWFSTATAT: u64 = 262;
}

/// Emulated syscall result
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

/// Emulate a Linux syscall in TrustOS context
pub fn emulate_syscall(
    num: u64,
    rdi: u64,
    rsi: u64,
    rdx: u64,
    r10: u64,
    r8: u64,
    r9: u64,
) -> SyscallResult {
    match num {
        syscall::WRITE => emulate_write(rdi as i32, rsi as *const u8, rdx as usize),
        syscall::READ => emulate_read(rdi as i32, rsi as *mut u8, rdx as usize),
        syscall::EXIT | syscall::EXIT_GROUP => emulate_exit(rdi as i32),
        syscall::GETPID => emulate_getpid(),
        syscall::GETCWD => emulate_getcwd(rdi as *mut u8, rsi as usize),
        syscall::UNAME => emulate_uname(rdi as *mut UnameInfo),
        syscall::BRK => emulate_brk(rdi),
        syscall::MMAP => emulate_mmap(rdi, rsi, rdx as i32, r10 as i32, r8 as i32, r9 as i64),
        syscall::MUNMAP => emulate_munmap(rdi, rsi),
        syscall::ARCH_PRCTL => emulate_arch_prctl(rdi as i32, rsi),
        syscall::SET_TID_ADDRESS => SyscallResult::success(1), // Return TID 1
        syscall::GETUID | syscall::GETEUID => SyscallResult::success(0), // root
        syscall::GETGID | syscall::GETEGID => SyscallResult::success(0), // root
        _ => {
            // Unsupported syscall
            SyscallResult::error(38) // ENOSYS
        }
    }
}

fn emulate_write(fd: i32, buf: *const u8, count: usize) -> SyscallResult {
    if buf.is_null() {
        return SyscallResult::error(14); // EFAULT
    }
    
    match fd {
        1 | 2 => {
            // stdout/stderr - print to console
            let slice = unsafe { core::slice::from_raw_parts(buf, count) };
            if let Ok(s) = core::str::from_utf8(slice) {
                crate::print!("{}", s);
            } else {
                // Binary data
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
        _ => SyscallResult::error(9) // EBADF
    }
}

fn emulate_read(fd: i32, buf: *mut u8, count: usize) -> SyscallResult {
    if buf.is_null() {
        return SyscallResult::error(14); // EFAULT
    }
    
    match fd {
        0 => {
            // stdin - not supported yet
            SyscallResult::success(0) // EOF
        }
        _ => SyscallResult::error(9) // EBADF
    }
}

fn emulate_exit(code: i32) -> SyscallResult {
    crate::println!("[transpiled process exited with code {}]", code);
    SyscallResult::success(0)
}

fn emulate_getpid() -> SyscallResult {
    SyscallResult::success(1) // PID 1
}

fn emulate_getcwd(buf: *mut u8, size: usize) -> SyscallResult {
    if buf.is_null() || size == 0 {
        return SyscallResult::error(14); // EFAULT
    }
    
    let cwd = b"/\0";
    if size < cwd.len() {
        return SyscallResult::error(34); // ERANGE
    }
    
    unsafe {
        core::ptr::copy_nonoverlapping(cwd.as_ptr(), buf, cwd.len());
    }
    
    SyscallResult::success(buf as i64)
}

#[repr(C)]
pub struct UnameInfo {
    pub sysname: [u8; 65],
    pub nodename: [u8; 65],
    pub release: [u8; 65],
    pub version: [u8; 65],
    pub machine: [u8; 65],
    pub domainname: [u8; 65],
}

fn emulate_uname(buf: *mut UnameInfo) -> SyscallResult {
    if buf.is_null() {
        return SyscallResult::error(14); // EFAULT
    }
    
    unsafe {
        let info = &mut *buf;
        copy_str(&mut info.sysname, "TrustOS");
        copy_str(&mut info.nodename, "trustos");
        copy_str(&mut info.release, "1.0.0-transpiled");
        copy_str(&mut info.version, "#1 SMP TrustOS Kernel");
        copy_str(&mut info.machine, "x86_64");
        copy_str(&mut info.domainname, "(none)");
    }
    
    SyscallResult::success(0)
}

fn copy_str(dest: &mut [u8; 65], src: &str) {
    let bytes = src.as_bytes();
    let len = bytes.len().min(64);
    dest[..len].copy_from_slice(&bytes[..len]);
    dest[len] = 0;
}

fn emulate_brk(addr: u64) -> SyscallResult {
    // Simple brk emulation
    static mut BRK_ADDR: u64 = 0x10000000;
    
    unsafe {
        if addr == 0 {
            SyscallResult::success(BRK_ADDR as i64)
        } else if addr > BRK_ADDR {
            BRK_ADDR = addr;
            SyscallResult::success(addr as i64)
        } else {
            SyscallResult::success(BRK_ADDR as i64)
        }
    }
}

fn emulate_mmap(addr: u64, len: u64, prot: i32, flags: i32, fd: i32, offset: i64) -> SyscallResult {
    // Anonymous mapping only
    if fd != -1 {
        return SyscallResult::error(22); // EINVAL
    }
    
    static mut MMAP_ADDR: u64 = 0x20000000;
    
    unsafe {
        let result = MMAP_ADDR;
        MMAP_ADDR += (len + 0xFFF) & !0xFFF; // Page align
        SyscallResult::success(result as i64)
    }
}

fn emulate_munmap(addr: u64, len: u64) -> SyscallResult {
    // Just pretend it worked
    SyscallResult::success(0)
}

fn emulate_arch_prctl(code: i32, addr: u64) -> SyscallResult {
    const ARCH_SET_FS: i32 = 0x1002;
    const ARCH_GET_FS: i32 = 0x1003;
    
    match code {
        ARCH_SET_FS => {
            // Set FS base - we'd need to actually set this
            SyscallResult::success(0)
        }
        ARCH_GET_FS => {
            SyscallResult::success(0)
        }
        _ => SyscallResult::error(22) // EINVAL
    }
}

/// Execute a transpiled function
pub fn execute_transpiled<F: FnOnce() -> i32>(f: F) -> i32 {
    f()
}
