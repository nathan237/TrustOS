//! Linux Syscall Translation Layer
//!
//! Translates Linux x86_64 syscalls to TrustOS equivalents.

use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use super::LinuxProcess;

// Linux x86_64 syscall numbers
pub const SYS_READ: u64 = 0;
pub const SYS_WRITE: u64 = 1;
pub const SYS_OPEN: u64 = 2;
pub const SYS_CLOSE: u64 = 3;
pub const SYS_STAT: u64 = 4;
pub const SYS_FSTAT: u64 = 5;
pub const SYS_LSTAT: u64 = 6;
pub const SYS_POLL: u64 = 7;
pub const SYS_LSEEK: u64 = 8;
pub const SYS_MMAP: u64 = 9;
pub const SYS_MPROTECT: u64 = 10;
pub const SYS_MUNMAP: u64 = 11;
pub const SYS_BRK: u64 = 12;
pub const SYS_RT_SIGACTION: u64 = 13;
pub const SYS_RT_SIGPROCMASK: u64 = 14;
pub const SYS_IOCTL: u64 = 16;
pub const SYS_ACCESS: u64 = 21;
pub const SYS_PIPE: u64 = 22;
pub const SYS_DUP: u64 = 32;
pub const SYS_DUP2: u64 = 33;
pub const SYS_NANOSLEEP: u64 = 35;
pub const SYS_GETPID: u64 = 39;
pub const SYS_FORK: u64 = 57;
pub const SYS_EXECVE: u64 = 59;
pub const SYS_EXIT: u64 = 60;
pub const SYS_WAIT4: u64 = 61;
pub const SYS_UNAME: u64 = 63;
pub const SYS_FCNTL: u64 = 72;
pub const SYS_GETCWD: u64 = 79;
pub const SYS_CHDIR: u64 = 80;
pub const SYS_MKDIR: u64 = 83;
pub const SYS_RMDIR: u64 = 84;
pub const SYS_UNLINK: u64 = 87;
pub const SYS_READLINK: u64 = 89;
pub const SYS_GETUID: u64 = 102;
pub const SYS_GETGID: u64 = 104;
pub const SYS_GETEUID: u64 = 107;
pub const SYS_GETEGID: u64 = 108;
pub const SYS_GETPPID: u64 = 110;
pub const SYS_GETPGRP: u64 = 111;
pub const SYS_SETSID: u64 = 112;
pub const SYS_GETGROUPS: u64 = 115;
pub const SYS_SETGROUPS: u64 = 116;
pub const SYS_ARCH_PRCTL: u64 = 158;
pub const SYS_GETTID: u64 = 186;
pub const SYS_TIME: u64 = 201;
pub const SYS_CLOCK_GETTIME: u64 = 228;
pub const SYS_EXIT_GROUP: u64 = 231;
pub const SYS_OPENAT: u64 = 257;
pub const SYS_MKDIRAT: u64 = 258;
pub const SYS_FSTATAT: u64 = 262;
pub const SYS_UNLINKAT: u64 = 263;
pub const SYS_READLINKAT: u64 = 267;
pub const SYS_FACCESSAT: u64 = 269;
pub const SYS_SET_TID_ADDRESS: u64 = 218;
pub const SYS_SET_ROBUST_LIST: u64 = 273;
pub const SYS_PRLIMIT64: u64 = 302;
pub const SYS_GETRANDOM: u64 = 318;

// Error codes (negative)
pub const ENOSYS: i64 = -38;
pub const ENOENT: i64 = -2;
pub const EBADF: i64 = -9;
pub const EINVAL: i64 = -22;
pub const ENOMEM: i64 = -12;

/// Handle a Linux syscall
/// 
/// Arguments in registers: rdi, rsi, rdx, r10, r8, r9
/// Syscall number in rax
/// Return value in rax
pub fn handle_syscall(
    process: &mut LinuxProcess,
    syscall_num: u64,
    arg1: u64,
    arg2: u64,
    arg3: u64,
    arg4: u64,
    arg5: u64,
    arg6: u64,
) -> i64 {
    match syscall_num {
        SYS_READ => sys_read(process, arg1 as i32, arg2, arg3 as usize),
        SYS_WRITE => sys_write(process, arg1 as i32, arg2, arg3 as usize),
        SYS_OPEN => sys_open(process, arg1, arg2 as i32, arg3 as u32),
        SYS_OPENAT => sys_openat(process, arg1 as i32, arg2, arg3 as i32, arg4 as u32),
        SYS_CLOSE => sys_close(process, arg1 as i32),
        SYS_STAT | SYS_FSTAT | SYS_LSTAT | SYS_FSTATAT => sys_stat(process, arg1, arg2),
        SYS_BRK => sys_brk(process, arg1),
        SYS_MMAP => sys_mmap(process, arg1, arg2, arg3 as i32, arg4 as i32, arg5 as i32, arg6),
        SYS_MUNMAP => sys_munmap(process, arg1, arg2),
        SYS_MPROTECT => 0, // Stub: always succeed
        SYS_EXIT | SYS_EXIT_GROUP => sys_exit(process, arg1 as i32),
        SYS_UNAME => sys_uname(arg1),
        SYS_GETPID => process.pid as i64,
        SYS_GETPPID => 1, // init
        SYS_GETUID | SYS_GETEUID => 0, // root
        SYS_GETGID | SYS_GETEGID => 0, // root
        SYS_GETTID => process.pid as i64,
        SYS_GETCWD => sys_getcwd(process, arg1, arg2 as usize),
        SYS_CHDIR => sys_chdir(process, arg1),
        SYS_ACCESS | SYS_FACCESSAT => sys_access(process, arg1),
        SYS_IOCTL => sys_ioctl(process, arg1 as i32, arg2, arg3),
        SYS_FCNTL => 0, // Stub
        SYS_DUP => sys_dup(process, arg1 as i32),
        SYS_DUP2 => sys_dup2(process, arg1 as i32, arg2 as i32),
        SYS_PIPE => sys_pipe(process, arg1),
        SYS_NANOSLEEP => sys_nanosleep(arg1, arg2),
        SYS_CLOCK_GETTIME => sys_clock_gettime(arg1 as i32, arg2),
        SYS_TIME => sys_time(arg1),
        SYS_GETRANDOM => sys_getrandom(arg1, arg2 as usize, arg3 as u32),
        SYS_ARCH_PRCTL => sys_arch_prctl(arg1, arg2),
        SYS_SET_TID_ADDRESS => process.pid as i64,
        SYS_SET_ROBUST_LIST => 0,
        SYS_PRLIMIT64 => 0,
        SYS_RT_SIGACTION | SYS_RT_SIGPROCMASK => 0, // Stub signals
        SYS_MKDIR | SYS_MKDIRAT => sys_mkdir(process, arg1, arg2),
        SYS_UNLINK | SYS_UNLINKAT | SYS_RMDIR => sys_unlink(process, arg1),
        SYS_READLINK | SYS_READLINKAT => EINVAL,
        SYS_GETGROUPS => 0,
        SYS_SETGROUPS => 0,
        SYS_GETPGRP => process.pid as i64,
        SYS_SETSID => process.pid as i64,
        SYS_FORK => ENOSYS, // Not supported yet
        SYS_EXECVE => ENOSYS, // Not supported yet
        SYS_WAIT4 => ENOSYS,
        SYS_POLL => crate::syscall::linux::sys_poll(arg1, arg2 as u32, arg3 as i32),
        SYS_LSEEK => sys_lseek(process, arg1 as i32, arg2 as i64, arg3 as i32),
        _ => {
            crate::serial_println!("[LINUX] Unhandled syscall: {} (args: {:#x}, {:#x}, {:#x})", 
                syscall_num, arg1, arg2, arg3);
            ENOSYS
        }
    }
}

// ============================================================================
// SYSCALL IMPLEMENTATIONS
// ============================================================================

fn sys_read(process: &mut LinuxProcess, fd: i32, buf: u64, count: usize) -> i64 {
    if fd < 0 || fd >= 256 {
        return EBADF;
    }
    
    match fd {
        0 => {
            // stdin - read from keyboard
            let buf_slice = unsafe { core::slice::from_raw_parts_mut(buf as *mut u8, count) };
            let mut read_count = 0;
            
            // Simple blocking read
            while read_count < count {
                if let Some(c) = crate::keyboard::read_char() {
                    buf_slice[read_count] = c;
                    read_count += 1;
                    if c == b'\n' {
                        break;
                    }
                } else {
                    if read_count > 0 {
                        break;
                    }
                    core::hint::spin_loop();
                }
            }
            read_count as i64
        }
        _ => {
            // File read - not implemented yet
            EBADF
        }
    }
}

fn sys_write(process: &mut LinuxProcess, fd: i32, buf: u64, count: usize) -> i64 {
    if fd < 0 || fd >= 256 {
        return EBADF;
    }
    
    let data = unsafe { core::slice::from_raw_parts(buf as *const u8, count) };
    
    match fd {
        1 | 2 => {
            // stdout/stderr - print to console
            if let Ok(s) = core::str::from_utf8(data) {
                crate::print!("{}", s);
            } else {
                // Binary data, print as hex
                for byte in data {
                    crate::print!("{:02x}", byte);
                }
            }
            count as i64
        }
        _ => {
            // File write - not implemented yet
            EBADF
        }
    }
}

fn sys_open(process: &mut LinuxProcess, path_ptr: u64, flags: i32, mode: u32) -> i64 {
    let path = match read_string_from_user(path_ptr) {
        Ok(s) => s,
        Err(e) => return e,
    };
    crate::serial_println!("[LINUX] open({}, {:#x}, {:#o})", path, flags, mode);
    
    // Find free fd
    let fd = (3..256).find(|&i| process.fds[i].is_none());
    
    match fd {
        Some(fd) => {
            process.fds[fd] = Some(fd as u32);
            fd as i64
        }
        None => ENOMEM
    }
}

fn sys_openat(process: &mut LinuxProcess, dirfd: i32, path_ptr: u64, flags: i32, mode: u32) -> i64 {
    // For now, ignore dirfd and treat as absolute
    sys_open(process, path_ptr, flags, mode)
}

fn sys_close(process: &mut LinuxProcess, fd: i32) -> i64 {
    if fd < 0 || fd >= 256 {
        return EBADF;
    }
    process.fds[fd as usize] = None;
    0
}

fn sys_stat(_process: &mut LinuxProcess, _path_ptr: u64, stat_buf: u64) -> i64 {
    // Return a fake stat structure
    let stat = unsafe { &mut *(stat_buf as *mut LinuxStat) };
    *stat = LinuxStat::default();
    stat.st_mode = 0o100644; // Regular file
    stat.st_size = 0;
    0
}

fn sys_brk(process: &mut LinuxProcess, addr: u64) -> i64 {
    // Delegate to the real sys_brk which allocates physical frames and maps pages
    let result = crate::syscall::linux::sys_brk(addr);
    // Keep process-local tracking in sync
    if result > 0 {
        process.brk = result as u64;
    }
    result
}

fn sys_mmap(process: &mut LinuxProcess, addr: u64, length: u64, prot: i32, flags: i32, fd: i32, offset: u64) -> i64 {
    // Delegate to the real sys_mmap which handles page-level mapping
    crate::syscall::linux::sys_mmap(addr, length, prot as u64, flags as u64, fd as i64, offset)
}

fn sys_munmap(_process: &mut LinuxProcess, addr: u64, length: u64) -> i64 {
    // Delegate to the real sys_munmap
    crate::syscall::linux::sys_munmap(addr, length)
}

fn sys_exit(process: &mut LinuxProcess, code: i32) -> i64 {
    process.exit_code = Some(code);
    crate::serial_println!("[LINUX] Process {} exited with code {}", process.pid, code);
    code as i64
}

fn sys_uname(buf: u64) -> i64 {
    #[repr(C)]
    struct Utsname {
        sysname: [u8; 65],
        nodename: [u8; 65],
        release: [u8; 65],
        version: [u8; 65],
        machine: [u8; 65],
        domainname: [u8; 65],
    }
    
    let uname = unsafe { &mut *(buf as *mut Utsname) };
    
    fn write_field(field: &mut [u8; 65], value: &str) {
        let bytes = value.as_bytes();
        let len = bytes.len().min(64);
        field[..len].copy_from_slice(&bytes[..len]);
        field[len] = 0;
    }
    
    write_field(&mut uname.sysname, "Linux");
    write_field(&mut uname.nodename, "trustos");
    write_field(&mut uname.release, "5.15.0-trustos");
    write_field(&mut uname.version, "#1 SMP TrustOS");
    write_field(&mut uname.machine, "x86_64");
    write_field(&mut uname.domainname, "(none)");
    
    0
}

fn sys_getcwd(process: &mut LinuxProcess, buf: u64, size: usize) -> i64 {
    let cwd = process.cwd.as_bytes();
    if cwd.len() + 1 > size {
        return EINVAL;
    }
    
    let buf_slice = unsafe { core::slice::from_raw_parts_mut(buf as *mut u8, size) };
    buf_slice[..cwd.len()].copy_from_slice(cwd);
    buf_slice[cwd.len()] = 0;
    
    buf as i64
}

fn sys_chdir(process: &mut LinuxProcess, path_ptr: u64) -> i64 {
    let path = match read_string_from_user(path_ptr) {
        Ok(s) => s,
        Err(e) => return e,
    };
    process.cwd = path;
    0
}

fn sys_access(_process: &mut LinuxProcess, path_ptr: u64) -> i64 {
    let path = match read_string_from_user(path_ptr) {
        Ok(s) => s,
        Err(e) => return e,
    };
    if crate::linux::rootfs::exists(&path) {
        0
    } else {
        ENOENT
    }
}

fn sys_ioctl(process: &mut LinuxProcess, fd: i32, request: u64, arg: u64) -> i64 {
    // Terminal ioctls
    const TCGETS: u64 = 0x5401;
    const TIOCGWINSZ: u64 = 0x5413;
    
    match request {
        TCGETS => 0, // Pretend we're a terminal
        TIOCGWINSZ => {
            // Return terminal size
            #[repr(C)]
            struct Winsize {
                ws_row: u16,
                ws_col: u16,
                ws_xpixel: u16,
                ws_ypixel: u16,
            }
            let ws = unsafe { &mut *(arg as *mut Winsize) };
            ws.ws_row = 25;
            ws.ws_col = 80;
            ws.ws_xpixel = 0;
            ws.ws_ypixel = 0;
            0
        }
        _ => 0
    }
}

fn sys_dup(process: &mut LinuxProcess, oldfd: i32) -> i64 {
    if oldfd < 0 || oldfd >= 256 || process.fds[oldfd as usize].is_none() {
        return EBADF;
    }
    
    let newfd = (0..256).find(|&i| process.fds[i].is_none());
    match newfd {
        Some(fd) => {
            process.fds[fd] = process.fds[oldfd as usize];
            fd as i64
        }
        None => ENOMEM
    }
}

fn sys_dup2(process: &mut LinuxProcess, oldfd: i32, newfd: i32) -> i64 {
    if oldfd < 0 || oldfd >= 256 || newfd < 0 || newfd >= 256 {
        return EBADF;
    }
    if process.fds[oldfd as usize].is_none() {
        return EBADF;
    }
    
    process.fds[newfd as usize] = process.fds[oldfd as usize];
    newfd as i64
}

fn sys_pipe(process: &mut LinuxProcess, pipefd: u64) -> i64 {
    // Find two free fds
    let fd1 = (3..256).find(|&i| process.fds[i].is_none());
    let fd2 = fd1.and_then(|f1| (f1+1..256).find(|&i| process.fds[i].is_none()));
    
    match (fd1, fd2) {
        (Some(read_fd), Some(write_fd)) => {
            process.fds[read_fd] = Some(read_fd as u32);
            process.fds[write_fd] = Some(write_fd as u32);
            
            let fds = unsafe { &mut *(pipefd as *mut [i32; 2]) };
            fds[0] = read_fd as i32;
            fds[1] = write_fd as i32;
            0
        }
        _ => ENOMEM
    }
}

fn sys_nanosleep(req: u64, _rem: u64) -> i64 {
    #[repr(C)]
    struct Timespec {
        tv_sec: i64,
        tv_nsec: i64,
    }
    
    let ts = unsafe { &*(req as *const Timespec) };
    let ns = (ts.tv_sec as u64) * 1_000_000_000 + (ts.tv_nsec as u64);
    
    // Use real thread sleep instead of busy-wait
    crate::thread::sleep_ns(ns);
    
    0
}

fn sys_clock_gettime(clk_id: i32, tp: u64) -> i64 {
    #[repr(C)]
    struct Timespec {
        tv_sec: i64,
        tv_nsec: i64,
    }
    
    let ts = unsafe { &mut *(tp as *mut Timespec) };
    let ticks = crate::logger::get_ticks();
    ts.tv_sec = (ticks / 1000) as i64;
    ts.tv_nsec = ((ticks % 1000) * 1_000_000) as i64;
    
    0
}

fn sys_time(tloc: u64) -> i64 {
    let time = (crate::logger::get_ticks() / 1000) as i64;
    if tloc != 0 {
        unsafe { *(tloc as *mut i64) = time; }
    }
    time
}

fn sys_getrandom(buf: u64, buflen: usize, _flags: u32) -> i64 {
    let buffer = unsafe { core::slice::from_raw_parts_mut(buf as *mut u8, buflen) };
    
    // Use our RNG
    for byte in buffer.iter_mut() {
        *byte = crate::rng::random_u8();
    }
    
    buflen as i64
}

fn sys_arch_prctl(code: u64, addr: u64) -> i64 {
    const ARCH_SET_FS: u64 = 0x1002;
    const ARCH_GET_FS: u64 = 0x1003;
    const ARCH_SET_GS: u64 = 0x1001;
    const ARCH_GET_GS: u64 = 0x1004;
    
    match code {
        ARCH_SET_FS => {
            // Set FS base - needed for TLS
            unsafe {
                core::arch::asm!(
                    "wrfsbase {}",
                    in(reg) addr,
                );
            }
            0
        }
        ARCH_SET_GS => {
            unsafe {
                core::arch::asm!(
                    "wrgsbase {}",
                    in(reg) addr,
                );
            }
            0
        }
        ARCH_GET_FS | ARCH_GET_GS => 0,
        _ => EINVAL
    }
}

fn sys_mkdir(process: &mut LinuxProcess, path_ptr: u64, _mode: u64) -> i64 {
    let path = match read_string_from_user(path_ptr) {
        Ok(s) => s,
        Err(e) => return e,
    };
    let full_path = alloc::format!("/linux{}", path);
    
    crate::ramfs::with_fs(|fs| {
        match fs.mkdir(&full_path) {
            Ok(()) => 0,
            Err(_) => ENOENT
        }
    })
}

fn sys_unlink(process: &mut LinuxProcess, path_ptr: u64) -> i64 {
    let path = match read_string_from_user(path_ptr) {
        Ok(s) => s,
        Err(e) => return e,
    };
    let full_path = alloc::format!("/linux{}", path);
    
    crate::ramfs::with_fs(|fs| {
        match fs.rm(&full_path) {
            Ok(()) => 0,
            Err(_) => ENOENT
        }
    })
}

fn sys_lseek(_process: &mut LinuxProcess, fd: i32, offset: i64, whence: i32) -> i64 {
    // Stub
    0
}

// ============================================================================
// HELPER STRUCTURES
// ============================================================================

#[repr(C)]
#[derive(Default)]
struct LinuxStat {
    st_dev: u64,
    st_ino: u64,
    st_nlink: u64,
    st_mode: u32,
    st_uid: u32,
    st_gid: u32,
    __pad0: u32,
    st_rdev: u64,
    st_size: i64,
    st_blksize: i64,
    st_blocks: i64,
    st_atime: i64,
    st_atime_nsec: i64,
    st_mtime: i64,
    st_mtime_nsec: i64,
    st_ctime: i64,
    st_ctime_nsec: i64,
    __unused: [i64; 3],
}

fn read_string_from_user(ptr: u64) -> Result<String, i64> {
    if ptr == 0 {
        return Err(-14); // EFAULT — null pointer
    }
    
    let mut s = String::new();
    let mut p = ptr as *const u8;
    
    unsafe {
        while *p != 0 {
            s.push(*p as char);
            p = p.add(1);
            if s.len() > 4096 {
                break;
            }
        }
    }
    
    Ok(s)
}
