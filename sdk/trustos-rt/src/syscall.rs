//! TrustOS System Calls
//!
//! Low-level syscall wrappers using the SYSCALL instruction.
//! Register convention (System V AMD64):
//! - RAX = syscall number
//! - RDI = arg1, RSI = arg2, RDX = arg3
//! - R10 = arg4, R8 = arg5, R9 = arg6
//! - RAX = return value

use core::arch::asm;

// Syscall numbers (Linux-compatible)
pub const SYS_READ: u64 = 0;
pub const SYS_WRITE: u64 = 1;
pub const SYS_OPEN: u64 = 2;
pub const SYS_CLOSE: u64 = 3;
pub const SYS_STAT: u64 = 4;
pub const SYS_LSEEK: u64 = 8;
pub const SYS_GETPID: u64 = 39;
pub const SYS_EXIT: u64 = 60;
pub const SYS_GETCWD: u64 = 79;
pub const SYS_CHDIR: u64 = 80;
pub const SYS_MKDIR: u64 = 83;
pub const SYS_UNLINK: u64 = 87;
pub const SYS_SCHED_YIELD: u64 = 24;
pub const SYS_DEBUG_PRINT: u64 = 0x1000;

// File descriptors
pub const STDIN: i32 = 0;
pub const STDOUT: i32 = 1;
pub const STDERR: i32 = 2;

/// Raw syscall with 0 arguments
#[inline(always)]
pub unsafe fn syscall0(num: u64) -> i64 {
    let ret: i64;
    asm!(
        "syscall",
        in("rax") num,
        lateout("rax") ret,
        out("rcx") _,
        out("r11") _,
        options(nostack)
    );
    ret
}

/// Raw syscall with 1 argument
#[inline(always)]
pub unsafe fn syscall1(num: u64, arg1: u64) -> i64 {
    let ret: i64;
    asm!(
        "syscall",
        in("rax") num,
        in("rdi") arg1,
        lateout("rax") ret,
        out("rcx") _,
        out("r11") _,
        options(nostack)
    );
    ret
}

/// Raw syscall with 2 arguments
#[inline(always)]
pub unsafe fn syscall2(num: u64, arg1: u64, arg2: u64) -> i64 {
    let ret: i64;
    asm!(
        "syscall",
        in("rax") num,
        in("rdi") arg1,
        in("rsi") arg2,
        lateout("rax") ret,
        out("rcx") _,
        out("r11") _,
        options(nostack)
    );
    ret
}

/// Raw syscall with 3 arguments
#[inline(always)]
pub unsafe fn syscall3(num: u64, arg1: u64, arg2: u64, arg3: u64) -> i64 {
    let ret: i64;
    asm!(
        "syscall",
        in("rax") num,
        in("rdi") arg1,
        in("rsi") arg2,
        in("rdx") arg3,
        lateout("rax") ret,
        out("rcx") _,
        out("r11") _,
        options(nostack)
    );
    ret
}

/// Exit the process
pub fn exit(code: i32) -> ! {
    unsafe {
        syscall1(SYS_EXIT, code as u64);
    }
    // Should never reach here
    loop {
        unsafe { asm!("hlt", options(nomem, nostack)); }
    }
}

/// Read from a file descriptor
pub fn read(fd: i32, buf: &mut [u8]) -> Result<usize, i64> {
    let ret = unsafe {
        syscall3(SYS_READ, fd as u64, buf.as_mut_ptr() as u64, buf.len() as u64)
    };
    if ret < 0 {
        Err(ret)
    } else {
        Ok(ret as usize)
    }
}

/// Write to a file descriptor
pub fn write(fd: i32, buf: &[u8]) -> Result<usize, i64> {
    let ret = unsafe {
        syscall3(SYS_WRITE, fd as u64, buf.as_ptr() as u64, buf.len() as u64)
    };
    if ret < 0 {
        Err(ret)
    } else {
        Ok(ret as usize)
    }
}

/// Open a file
pub fn open(path: &str, flags: u32) -> Result<i32, i64> {
    let ret = unsafe {
        syscall2(SYS_OPEN, path.as_ptr() as u64, flags as u64)
    };
    if ret < 0 {
        Err(ret)
    } else {
        Ok(ret as i32)
    }
}

/// Close a file descriptor
pub fn close(fd: i32) -> Result<(), i64> {
    let ret = unsafe {
        syscall1(SYS_CLOSE, fd as u64)
    };
    if ret < 0 {
        Err(ret)
    } else {
        Ok(())
    }
}

/// Get process ID
pub fn getpid() -> u32 {
    unsafe { syscall0(SYS_GETPID) as u32 }
}

/// Yield CPU to other tasks
pub fn sched_yield() {
    unsafe { syscall0(SYS_SCHED_YIELD); }
}

/// Create a directory
pub fn mkdir(path: &str) -> Result<(), i64> {
    let ret = unsafe {
        syscall1(SYS_MKDIR, path.as_ptr() as u64)
    };
    if ret < 0 {
        Err(ret)
    } else {
        Ok(())
    }
}

/// Remove a file
pub fn unlink(path: &str) -> Result<(), i64> {
    let ret = unsafe {
        syscall1(SYS_UNLINK, path.as_ptr() as u64)
    };
    if ret < 0 {
        Err(ret)
    } else {
        Ok(())
    }
}

/// Debug print (writes to serial console)
pub fn debug_print(buf: &[u8]) -> Result<usize, i64> {
    let ret = unsafe {
        syscall2(SYS_DEBUG_PRINT, buf.as_ptr() as u64, buf.len() as u64)
    };
    if ret < 0 {
        Err(ret)
    } else {
        Ok(ret as usize)
    }
}
