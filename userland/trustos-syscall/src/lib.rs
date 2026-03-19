//! TrustOS Syscall Library
//!
//! Linux-compatible syscall wrappers for Ring 3 userland programs.
//! Uses the same syscall numbers as the kernel's handle_full() dispatcher.

#![no_std]
#![allow(dead_code)]

// ─── Linux syscall numbers (x86_64) ───────────────────────────────

pub const SYS_READ: u64 = 0;
pub const SYS_WRITE: u64 = 1;
pub const SYS_OPEN: u64 = 2;
pub const SYS_CLOSE: u64 = 3;
pub const SYS_STAT: u64 = 4;
pub const SYS_FSTAT: u64 = 5;
pub const SYS_LSEEK: u64 = 8;
pub const SYS_MMAP: u64 = 9;
pub const SYS_MPROTECT: u64 = 10;
pub const SYS_MUNMAP: u64 = 11;
pub const SYS_BRK: u64 = 12;
pub const SYS_IOCTL: u64 = 16;
pub const SYS_PIPE2: u64 = 293;
pub const SYS_DUP: u64 = 32;
pub const SYS_DUP2: u64 = 33;
pub const SYS_GETPID: u64 = 39;
pub const SYS_FORK: u64 = 57;
pub const SYS_EXECVE: u64 = 59;
pub const SYS_EXIT: u64 = 60;
pub const SYS_WAIT4: u64 = 61;
pub const SYS_KILL: u64 = 62;
pub const SYS_GETCWD: u64 = 79;
pub const SYS_CHDIR: u64 = 80;
pub const SYS_MKDIR: u64 = 83;
pub const SYS_UNLINK: u64 = 87;
pub const SYS_GETUID: u64 = 102;
pub const SYS_GETGID: u64 = 104;
pub const SYS_GETTID: u64 = 186;
pub const SYS_CLOCK_GETTIME: u64 = 228;
pub const SYS_EXIT_GROUP: u64 = 231;
pub const SYS_SCHED_YIELD: u64 = 24;
pub const SYS_NANOSLEEP: u64 = 35;
pub const SYS_SOCKET: u64 = 41;
pub const SYS_CONNECT: u64 = 42;
pub const SYS_ACCEPT: u64 = 43;
pub const SYS_SENDTO: u64 = 44;
pub const SYS_RECVFROM: u64 = 45;
pub const SYS_BIND: u64 = 49;
pub const SYS_LISTEN: u64 = 50;
pub const SYS_RT_SIGACTION: u64 = 13;
pub const SYS_RT_SIGPROCMASK: u64 = 14;
pub const SYS_ARCH_PRCTL: u64 = 158;

// ─── Raw syscall interface (x86_64) ──────────────────────────────

#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn syscall0(num: u64) -> i64 {
    let ret: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") num as i64 => ret,
        lateout("rcx") _,
        lateout("r11") _,
        options(nostack),
    );
    ret
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn syscall1(num: u64, a1: u64) -> i64 {
    let ret: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") num as i64 => ret,
        in("rdi") a1,
        lateout("rcx") _,
        lateout("r11") _,
        options(nostack),
    );
    ret
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn syscall2(num: u64, a1: u64, a2: u64) -> i64 {
    let ret: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") num as i64 => ret,
        in("rdi") a1,
        in("rsi") a2,
        lateout("rcx") _,
        lateout("r11") _,
        options(nostack),
    );
    ret
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn syscall3(num: u64, a1: u64, a2: u64, a3: u64) -> i64 {
    let ret: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") num as i64 => ret,
        in("rdi") a1,
        in("rsi") a2,
        in("rdx") a3,
        lateout("rcx") _,
        lateout("r11") _,
        options(nostack),
    );
    ret
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn syscall4(num: u64, a1: u64, a2: u64, a3: u64, a4: u64) -> i64 {
    let ret: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") num as i64 => ret,
        in("rdi") a1,
        in("rsi") a2,
        in("rdx") a3,
        in("r10") a4,
        lateout("rcx") _,
        lateout("r11") _,
        options(nostack),
    );
    ret
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn syscall6(num: u64, a1: u64, a2: u64, a3: u64, a4: u64, a5: u64, a6: u64) -> i64 {
    let ret: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") num as i64 => ret,
        in("rdi") a1,
        in("rsi") a2,
        in("rdx") a3,
        in("r10") a4,
        in("r8") a5,
        in("r9") a6,
        lateout("rcx") _,
        lateout("r11") _,
        options(nostack),
    );
    ret
}

// ─── aarch64 stubs ───────────────────────────────────────────────

#[cfg(target_arch = "aarch64")]
#[inline(always)]
unsafe fn syscall0(num: u64) -> i64 {
    let ret: i64;
    core::arch::asm!(
        "svc #0",
        inlateout("x8") num as i64 => _,
        lateout("x0") ret,
        options(nostack),
    );
    ret
}

#[cfg(target_arch = "aarch64")]
#[inline(always)]
unsafe fn syscall1(num: u64, a1: u64) -> i64 {
    let ret: i64;
    core::arch::asm!(
        "svc #0",
        inlateout("x8") num as i64 => _,
        inlateout("x0") a1 as i64 => ret,
        options(nostack),
    );
    ret
}

#[cfg(target_arch = "aarch64")]
#[inline(always)]
unsafe fn syscall2(num: u64, a1: u64, a2: u64) -> i64 {
    let ret: i64;
    core::arch::asm!(
        "svc #0",
        inlateout("x8") num as i64 => _,
        inlateout("x0") a1 as i64 => ret,
        in("x1") a2,
        options(nostack),
    );
    ret
}

#[cfg(target_arch = "aarch64")]
#[inline(always)]
unsafe fn syscall3(num: u64, a1: u64, a2: u64, a3: u64) -> i64 {
    let ret: i64;
    core::arch::asm!(
        "svc #0",
        inlateout("x8") num as i64 => _,
        inlateout("x0") a1 as i64 => ret,
        in("x1") a2,
        in("x2") a3,
        options(nostack),
    );
    ret
}

#[cfg(target_arch = "aarch64")]
#[inline(always)]
unsafe fn syscall4(num: u64, a1: u64, a2: u64, a3: u64, a4: u64) -> i64 {
    let ret: i64;
    core::arch::asm!(
        "svc #0",
        inlateout("x8") num as i64 => _,
        inlateout("x0") a1 as i64 => ret,
        in("x1") a2,
        in("x2") a3,
        in("x3") a4,
        options(nostack),
    );
    ret
}

#[cfg(target_arch = "aarch64")]
#[inline(always)]
unsafe fn syscall6(num: u64, a1: u64, a2: u64, a3: u64, a4: u64, a5: u64, a6: u64) -> i64 {
    let ret: i64;
    core::arch::asm!(
        "svc #0",
        inlateout("x8") num as i64 => _,
        inlateout("x0") a1 as i64 => ret,
        in("x1") a2,
        in("x2") a3,
        in("x3") a4,
        in("x4") a5,
        in("x5") a6,
        options(nostack),
    );
    ret
}

// ─── riscv64 stubs ───────────────────────────────────────────────

#[cfg(target_arch = "riscv64")]
#[inline(always)]
unsafe fn syscall0(num: u64) -> i64 {
    let ret: i64;
    core::arch::asm!(
        "ecall",
        inlateout("a7") num as i64 => _,
        lateout("a0") ret,
        options(nostack),
    );
    ret
}

#[cfg(target_arch = "riscv64")]
#[inline(always)]
unsafe fn syscall1(num: u64, a1: u64) -> i64 {
    let ret: i64;
    core::arch::asm!(
        "ecall",
        inlateout("a7") num as i64 => _,
        inlateout("a0") a1 as i64 => ret,
        options(nostack),
    );
    ret
}

#[cfg(target_arch = "riscv64")]
#[inline(always)]
unsafe fn syscall2(num: u64, a1: u64, a2: u64) -> i64 {
    let ret: i64;
    core::arch::asm!(
        "ecall",
        inlateout("a7") num as i64 => _,
        inlateout("a0") a1 as i64 => ret,
        in("a1") a2,
        options(nostack),
    );
    ret
}

#[cfg(target_arch = "riscv64")]
#[inline(always)]
unsafe fn syscall3(num: u64, a1: u64, a2: u64, a3: u64) -> i64 {
    let ret: i64;
    core::arch::asm!(
        "ecall",
        inlateout("a7") num as i64 => _,
        inlateout("a0") a1 as i64 => ret,
        in("a1") a2,
        in("a2") a3,
        options(nostack),
    );
    ret
}

#[cfg(target_arch = "riscv64")]
#[inline(always)]
unsafe fn syscall4(num: u64, a1: u64, a2: u64, a3: u64, a4: u64) -> i64 {
    let ret: i64;
    core::arch::asm!(
        "ecall",
        inlateout("a7") num as i64 => _,
        inlateout("a0") a1 as i64 => ret,
        in("a1") a2,
        in("a2") a3,
        in("a3") a4,
        options(nostack),
    );
    ret
}

#[cfg(target_arch = "riscv64")]
#[inline(always)]
unsafe fn syscall6(num: u64, a1: u64, a2: u64, a3: u64, a4: u64, a5: u64, a6: u64) -> i64 {
    let ret: i64;
    core::arch::asm!(
        "ecall",
        inlateout("a7") num as i64 => _,
        inlateout("a0") a1 as i64 => ret,
        in("a1") a2,
        in("a2") a3,
        in("a3") a4,
        in("a4") a5,
        in("a5") a6,
        options(nostack),
    );
    ret
}

// ─── High-level wrappers ─────────────────────────────────────────

/// Write bytes to a file descriptor. Returns bytes written or negative error.
pub fn write(fd: i32, buf: &[u8]) -> isize {
    unsafe { syscall3(SYS_WRITE, fd as u64, buf.as_ptr() as u64, buf.len() as u64) as isize }
}

/// Read bytes from a file descriptor. Returns bytes read or negative error.
pub fn read(fd: i32, buf: &mut [u8]) -> isize {
    unsafe { syscall3(SYS_READ, fd as u64, buf.as_mut_ptr() as u64, buf.len() as u64) as isize }
}

/// Write a string to stdout (fd 1).
pub fn print(s: &str) {
    write(1, s.as_bytes());
}

/// Exit the current process.
pub fn exit(code: i32) -> ! {
    unsafe { syscall1(SYS_EXIT, code as u64); }
    loop {}
}

/// Exit all threads in the process group.
pub fn exit_group(code: i32) -> ! {
    unsafe { syscall1(SYS_EXIT_GROUP, code as u64); }
    loop {}
}

/// Get the current process ID.
pub fn getpid() -> i32 {
    unsafe { syscall0(SYS_GETPID) as i32 }
}

/// Get the current thread ID.
pub fn gettid() -> i32 {
    unsafe { syscall0(SYS_GETTID) as i32 }
}

/// Fork the current process.
pub fn fork() -> i32 {
    unsafe { syscall0(SYS_FORK) as i32 }
}

/// Wait for a child process.
pub fn wait4(pid: i32, status: &mut i32, options: i32) -> i32 {
    unsafe {
        syscall4(
            SYS_WAIT4,
            pid as u64,
            status as *mut i32 as u64,
            options as u64,
            0, // rusage = NULL
        ) as i32
    }
}

/// Send a signal to a process.
pub fn kill(pid: i32, sig: i32) -> i32 {
    unsafe { syscall2(SYS_KILL, pid as u64, sig as u64) as i32 }
}

/// Yield the CPU to the scheduler.
pub fn sched_yield() -> i32 {
    unsafe { syscall0(SYS_SCHED_YIELD) as i32 }
}

/// Open a file. Returns fd or negative error.
pub fn open(path: &[u8], flags: i32, mode: u32) -> i32 {
    unsafe { syscall3(SYS_OPEN, path.as_ptr() as u64, flags as u64, mode as u64) as i32 }
}

/// Close a file descriptor.
pub fn close(fd: i32) -> i32 {
    unsafe { syscall1(SYS_CLOSE, fd as u64) as i32 }
}

/// Change heap break. Returns new break or negative error.
pub fn brk(addr: u64) -> i64 {
    unsafe { syscall1(SYS_BRK, addr) }
}

/// Memory map.
pub fn mmap(addr: u64, len: u64, prot: i32, flags: i32, fd: i32, offset: u64) -> i64 {
    unsafe {
        syscall6(
            SYS_MMAP,
            addr,
            len,
            prot as u64,
            flags as u64,
            fd as u64,
            offset,
        )
    }
}

/// Unmap memory.
pub fn munmap(addr: u64, len: u64) -> i32 {
    unsafe { syscall2(SYS_MUNMAP, addr, len) as i32 }
}

/// Get current working directory.
pub fn getcwd(buf: &mut [u8]) -> i32 {
    unsafe { syscall2(SYS_GETCWD, buf.as_mut_ptr() as u64, buf.len() as u64) as i32 }
}

/// Change directory.
pub fn chdir(path: &[u8]) -> i32 {
    unsafe { syscall1(SYS_CHDIR, path.as_ptr() as u64) as i32 }
}

/// Create directory.
pub fn mkdir(path: &[u8], mode: u32) -> i32 {
    unsafe { syscall2(SYS_MKDIR, path.as_ptr() as u64, mode as u64) as i32 }
}

/// Get clock time.
pub fn clock_gettime(clk_id: i32, tp: &mut Timespec) -> i32 {
    unsafe { syscall2(SYS_CLOCK_GETTIME, clk_id as u64, tp as *mut Timespec as u64) as i32 }
}

/// Nanosleep.
pub fn nanosleep(req: &Timespec, rem: Option<&mut Timespec>) -> i32 {
    let rem_ptr = match rem {
        Some(r) => r as *mut Timespec as u64,
        None => 0,
    };
    unsafe { syscall2(SYS_NANOSLEEP, req as *const Timespec as u64, rem_ptr) as i32 }
}

// ─── Types ───────────────────────────────────────────────────────

#[repr(C)]
pub struct Timespec {
    pub tv_sec: i64,
    pub tv_nsec: i64,
}
