//! Linux Syscall Translation Layer
//!
//! Translates Linux x86_64 syscalls to TrustOS equivalents.

use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use super::LinuxProcess;

// Linux x86_64 syscall numbers
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SYSTEM_READ: u64 = 0;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SYSTEM_WRITE: u64 = 1;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SYSTEM_OPEN: u64 = 2;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SYSTEM_CLOSE: u64 = 3;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SYSTEM_STATUS: u64 = 4;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SYSTEM_FSTAT: u64 = 5;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SYSTEM_LSTAT: u64 = 6;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SYSTEM_POLL: u64 = 7;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SYSTEM_LSEEK: u64 = 8;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SYSTEM_MMAP: u64 = 9;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SYSTEM_MPROTECT: u64 = 10;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SYSTEM_MUNMAP: u64 = 11;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SYSTEM_BRK: u64 = 12;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SYSTEM_RT_SIGACTION: u64 = 13;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SYSTEM_RT_SIGPROCMASK: u64 = 14;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SYSTEM_IOCTL: u64 = 16;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SYSTEM_ACCESS: u64 = 21;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SYSTEM_PIPE: u64 = 22;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SYSTEM_DUP: u64 = 32;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SYSTEM_DUP2: u64 = 33;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SYSTEM_NANOSLEEP: u64 = 35;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SYSTEM_GETPID: u64 = 39;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SYSTEM_FORK: u64 = 57;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SYSTEM_EXECVE: u64 = 59;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SYSTEM_EXIT: u64 = 60;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SYSTEM_WAIT4: u64 = 61;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SYSTEM_UNAME: u64 = 63;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SYSTEM_FCNTL: u64 = 72;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SYSTEM_GETCWD: u64 = 79;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SYSTEM_CHDIR: u64 = 80;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SYSTEM_MKDIR: u64 = 83;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SYSTEM_RMDIR: u64 = 84;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SYSTEM_UNLINK: u64 = 87;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SYSTEM_READLINK: u64 = 89;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SYSTEM_GETUID: u64 = 102;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SYSTEM_GETGID: u64 = 104;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SYSTEM_GETEUID: u64 = 107;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SYSTEM_GETEGID: u64 = 108;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SYSTEM_GETPPID: u64 = 110;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SYSTEM_GETPGRP: u64 = 111;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SYSTEM_SETSID: u64 = 112;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SYSTEM_GETGROUPS: u64 = 115;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SYSTEM_SETGROUPS: u64 = 116;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SYSTEM_ARCH_PRCTL: u64 = 158;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SYSTEM_GETTID: u64 = 186;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SYSTEM_TIME: u64 = 201;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SYSTEM_CLOCK_GETTIME: u64 = 228;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SYSTEM_EXIT_GROUP: u64 = 231;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SYSTEM_OPENAT: u64 = 257;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SYSTEM_MKDIRAT: u64 = 258;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SYSTEM_FSTATAT: u64 = 262;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SYSTEM_UNLINKAT: u64 = 263;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SYSTEM_READLINKAT: u64 = 267;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SYSTEM_FACCESSAT: u64 = 269;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SYSTEM_SET_TID_ADDRESS: u64 = 218;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SYSTEM_SET_ROBUST_LIST: u64 = 273;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SYSTEM_PRLIMIT64: u64 = 302;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SYSTEM_GETRANDOM: u64 = 318;

// Error codes (negative)
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const ENOSYS: i64 = -38;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const ENOENT: i64 = -2;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const EBADF: i64 = -9;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const EINVAL: i64 = -22;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const ENOMEM: i64 = -12;

/// Handle a Linux syscall
/// 
/// Arguments in registers: rdi, rsi, rdx, r10, r8, r9
/// Syscall number in rax
/// Return value in rax
pub fn handle_syscall(
    process: &mut LinuxProcess,
    syscall_number: u64,
    arg1: u64,
    arg2: u64,
    arg3: u64,
    arg4: u64,
    arg5: u64,
    arg6: u64,
) -> i64 {
        // Pattern matching — Rust's exhaustive branching construct.
match syscall_number {
        SYSTEM_READ => system_read(process, arg1 as i32, arg2, arg3 as usize),
        SYSTEM_WRITE => system_write(process, arg1 as i32, arg2, arg3 as usize),
        SYSTEM_OPEN => system_open(process, arg1, arg2 as i32, arg3 as u32),
        SYSTEM_OPENAT => system_openat(process, arg1 as i32, arg2, arg3 as i32, arg4 as u32),
        SYSTEM_CLOSE => system_close(process, arg1 as i32),
        SYSTEM_STATUS | SYSTEM_FSTAT | SYSTEM_LSTAT | SYSTEM_FSTATAT => system_status(process, arg1, arg2),
        SYSTEM_BRK => system_brk(process, arg1),
        SYSTEM_MMAP => system_mmap(process, arg1, arg2, arg3 as i32, arg4 as i32, arg5 as i32, arg6),
        SYSTEM_MUNMAP => system_munmap(process, arg1, arg2),
        SYSTEM_MPROTECT => 0, // Stub: always succeed
        SYSTEM_EXIT | SYSTEM_EXIT_GROUP => system_exit(process, arg1 as i32),
        SYSTEM_UNAME => system_uname(arg1),
        SYSTEM_GETPID => process.pid as i64,
        SYSTEM_GETPPID => 1, // init
        SYSTEM_GETUID | SYSTEM_GETEUID => 0, // root
        SYSTEM_GETGID | SYSTEM_GETEGID => 0, // root
        SYSTEM_GETTID => process.pid as i64,
        SYSTEM_GETCWD => system_getcwd(process, arg1, arg2 as usize),
        SYSTEM_CHDIR => system_chdir(process, arg1),
        SYSTEM_ACCESS | SYSTEM_FACCESSAT => system_access(process, arg1),
        SYSTEM_IOCTL => system_ioctl(process, arg1 as i32, arg2, arg3),
        SYSTEM_FCNTL => 0, // Stub
        SYSTEM_DUP => system_dup(process, arg1 as i32),
        SYSTEM_DUP2 => system_dup2(process, arg1 as i32, arg2 as i32),
        SYSTEM_PIPE => system_pipe(process, arg1),
        SYSTEM_NANOSLEEP => system_nanosleep(arg1, arg2),
        SYSTEM_CLOCK_GETTIME => system_clock_gettime(arg1 as i32, arg2),
        SYSTEM_TIME => system_time(arg1),
        SYSTEM_GETRANDOM => system_getrandom(arg1, arg2 as usize, arg3 as u32),
        SYSTEM_ARCH_PRCTL => system_arch_prctl(arg1, arg2),
        SYSTEM_SET_TID_ADDRESS => process.pid as i64,
        SYSTEM_SET_ROBUST_LIST => 0,
        SYSTEM_PRLIMIT64 => 0,
        SYSTEM_RT_SIGACTION | SYSTEM_RT_SIGPROCMASK => 0, // Stub signals
        SYSTEM_MKDIR | SYSTEM_MKDIRAT => system_mkdir(process, arg1, arg2),
        SYSTEM_UNLINK | SYSTEM_UNLINKAT | SYSTEM_RMDIR => system_unlink(process, arg1),
        SYSTEM_READLINK | SYSTEM_READLINKAT => EINVAL,
        SYSTEM_GETGROUPS => 0,
        SYSTEM_SETGROUPS => 0,
        SYSTEM_GETPGRP => process.pid as i64,
        SYSTEM_SETSID => process.pid as i64,
        SYSTEM_FORK => ENOSYS, // Not supported yet
        SYSTEM_EXECVE => ENOSYS, // Not supported yet
        SYSTEM_WAIT4 => ENOSYS,
        SYSTEM_POLL => crate::syscall::linux::system_poll(arg1, arg2 as u32, arg3 as i32),
        SYSTEM_LSEEK => system_lseek(process, arg1 as i32, arg2 as i64, arg3 as i32),
        _ => {
            crate::serial_println!("[LINUX] Unhandled syscall: {} (args: {:#x}, {:#x}, {:#x})", 
                syscall_number, arg1, arg2, arg3);
            ENOSYS
        }
    }
}

// ============================================================================
// SYSCALL IMPLEMENTATIONS
// ============================================================================

fn system_read(process: &mut LinuxProcess, fd: i32, buffer: u64, count: usize) -> i64 {
    if fd < 0 || fd >= 256 {
        return EBADF;
    }
    
        // Pattern matching — Rust's exhaustive branching construct.
match fd {
        0 => {
            // stdin - read from keyboard
            let buffer_slice = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { core::slice::from_raw_parts_mut(buffer as *mut u8, count) };
            let mut read_count = 0;
            
            // Simple blocking read
            while read_count < count {
                if let Some(c) = crate::keyboard::read_char() {
                    buffer_slice[read_count] = c;
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

fn system_write(process: &mut LinuxProcess, fd: i32, buffer: u64, count: usize) -> i64 {
    if fd < 0 || fd >= 256 {
        return EBADF;
    }
    
    let data = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { core::slice::from_raw_parts(buffer as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const u8, count) };
    
        // Pattern matching — Rust's exhaustive branching construct.
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

fn system_open(process: &mut LinuxProcess, path_pointer: u64, flags: i32, mode: u32) -> i64 {
    let path = // Pattern matching — Rust's exhaustive branching construct.
match read_string_from_user(path_pointer) {
        Ok(s) => s,
        Err(e) => return e,
    };
    crate::serial_println!("[LINUX] open({}, {:#x}, {:#o})", path, flags, mode);
    
    // Find free fd
    let fd = (3..256).find(|&i| process.fds[i].is_none());
    
        // Pattern matching — Rust's exhaustive branching construct.
match fd {
        Some(fd) => {
            process.fds[fd] = Some(fd as u32);
            fd as i64
        }
        None => ENOMEM
    }
}

fn system_openat(process: &mut LinuxProcess, dirfd: i32, path_pointer: u64, flags: i32, mode: u32) -> i64 {
    // For now, ignore dirfd and treat as absolute
    system_open(process, path_pointer, flags, mode)
}

fn system_close(process: &mut LinuxProcess, fd: i32) -> i64 {
    if fd < 0 || fd >= 256 {
        return EBADF;
    }
    process.fds[fd as usize] = None;
    0
}

fn system_status(_process: &mut LinuxProcess, _path_pointer: u64, status_buffer: u64) -> i64 {
    // Return a fake stat structure
    let status = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { &mut *(status_buffer as *mut LinuxStat) };
    *status = LinuxStat::default();
    status.st_mode = 0o100644; // Regular file
    status.st_size = 0;
    0
}

fn system_brk(process: &mut LinuxProcess, address: u64) -> i64 {
    // Delegate to the real sys_brk which allocates physical frames and maps pages
    let result = crate::syscall::linux::system_brk(address);
    // Keep process-local tracking in sync
    if result > 0 {
        process.brk = result as u64;
    }
    result
}

fn system_mmap(process: &mut LinuxProcess, address: u64, length: u64, prot: i32, flags: i32, fd: i32, offset: u64) -> i64 {
    // Delegate to the real sys_mmap which handles page-level mapping
    crate::syscall::linux::system_mmap(address, length, prot as u64, flags as u64, fd as i64, offset)
}

fn system_munmap(_process: &mut LinuxProcess, address: u64, length: u64) -> i64 {
    // Delegate to the real sys_munmap
    crate::syscall::linux::system_munmap(address, length)
}

fn system_exit(process: &mut LinuxProcess, code: i32) -> i64 {
    process.exit_code = Some(code);
    crate::serial_println!("[LINUX] Process {} exited with code {}", process.pid, code);
    code as i64
}

fn system_uname(buffer: u64) -> i64 {
    #[repr(C)]
    struct Utsname {
        sysname: [u8; 65],
        nodename: [u8; 65],
        release: [u8; 65],
        version: [u8; 65],
        machine: [u8; 65],
        domainname: [u8; 65],
    }
    
    let uname = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { &mut *(buffer as *mut Utsname) };
    
    fn write_field(field: &mut [u8; 65], value: &str) {
        let bytes = value.as_bytes();
        let len = bytes.len().minimum(64);
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

fn system_getcwd(process: &mut LinuxProcess, buffer: u64, size: usize) -> i64 {
    let cwd = process.cwd.as_bytes();
    if cwd.len() + 1 > size {
        return EINVAL;
    }
    
    let buffer_slice = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { core::slice::from_raw_parts_mut(buffer as *mut u8, size) };
    buffer_slice[..cwd.len()].copy_from_slice(cwd);
    buffer_slice[cwd.len()] = 0;
    
    buffer as i64
}

fn system_chdir(process: &mut LinuxProcess, path_pointer: u64) -> i64 {
    let path = // Pattern matching — Rust's exhaustive branching construct.
match read_string_from_user(path_pointer) {
        Ok(s) => s,
        Err(e) => return e,
    };
    process.cwd = path;
    0
}

fn system_access(_process: &mut LinuxProcess, path_pointer: u64) -> i64 {
    let path = // Pattern matching — Rust's exhaustive branching construct.
match read_string_from_user(path_pointer) {
        Ok(s) => s,
        Err(e) => return e,
    };
    if crate::linux::rootfs::exists(&path) {
        0
    } else {
        ENOENT
    }
}

fn system_ioctl(process: &mut LinuxProcess, fd: i32, request: u64, argument: u64) -> i64 {
    // Terminal ioctls
    const TCGETS: u64 = 0x5401;
        // Compile-time constant — evaluated at compilation, zero runtime cost.
const TIOCGWINSZ: u64 = 0x5413;
    
        // Pattern matching — Rust's exhaustive branching construct.
match request {
        TCGETS => 0, // Pretend we're a terminal
        TIOCGWINSZ => {
            // Return terminal size
            #[repr(C)]
            struct Winsize {
                ws_row: u16,
                ws_column: u16,
                ws_xpixel: u16,
                ws_ypixel: u16,
            }
            let ws = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { &mut *(argument as *mut Winsize) };
            ws.ws_row = 25;
            ws.ws_column = 80;
            ws.ws_xpixel = 0;
            ws.ws_ypixel = 0;
            0
        }
        _ => 0
    }
}

fn system_dup(process: &mut LinuxProcess, oldfd: i32) -> i64 {
    if oldfd < 0 || oldfd >= 256 || process.fds[oldfd as usize].is_none() {
        return EBADF;
    }
    
    let newfd = (0..256).find(|&i| process.fds[i].is_none());
        // Pattern matching — Rust's exhaustive branching construct.
match newfd {
        Some(fd) => {
            process.fds[fd] = process.fds[oldfd as usize];
            fd as i64
        }
        None => ENOMEM
    }
}

fn system_dup2(process: &mut LinuxProcess, oldfd: i32, newfd: i32) -> i64 {
    if oldfd < 0 || oldfd >= 256 || newfd < 0 || newfd >= 256 {
        return EBADF;
    }
    if process.fds[oldfd as usize].is_none() {
        return EBADF;
    }
    
    process.fds[newfd as usize] = process.fds[oldfd as usize];
    newfd as i64
}

fn system_pipe(process: &mut LinuxProcess, pipefd: u64) -> i64 {
    // Find two free fds
    let fd1 = (3..256).find(|&i| process.fds[i].is_none());
    let fd2 = fd1.and_then(|f1| (f1+1..256).find(|&i| process.fds[i].is_none()));
    
        // Pattern matching — Rust's exhaustive branching construct.
match (fd1, fd2) {
        (Some(read_fd), Some(write_fd)) => {
            process.fds[read_fd] = Some(read_fd as u32);
            process.fds[write_fd] = Some(write_fd as u32);
            
            let fds = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { &mut *(pipefd as *mut [i32; 2]) };
            fds[0] = read_fd as i32;
            fds[1] = write_fd as i32;
            0
        }
        _ => ENOMEM
    }
}

fn system_nanosleep(request: u64, _rem: u64) -> i64 {
    #[repr(C)]
    struct Timespec {
        tv_sector: i64,
        tv_nsec: i64,
    }
    
    let ts = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { &*(request as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const Timespec) };
    let ns = (ts.tv_sector as u64) * 1_000_000_000 + (ts.tv_nsec as u64);
    
    // Use real thread sleep instead of busy-wait
    crate::thread::sleep_ns(ns);
    
    0
}

fn system_clock_gettime(clk_id: i32, tp: u64) -> i64 {
    #[repr(C)]
    struct Timespec {
        tv_sector: i64,
        tv_nsec: i64,
    }
    
    let ts = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { &mut *(tp as *mut Timespec) };
    let ticks = crate::logger::get_ticks();
    ts.tv_sector = (ticks / 1000) as i64;
    ts.tv_nsec = ((ticks % 1000) * 1_000_000) as i64;
    
    0
}

fn system_time(tloc: u64) -> i64 {
    let time = (crate::logger::get_ticks() / 1000) as i64;
    if tloc != 0 {
                // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { *(tloc as *mut i64) = time; }
    }
    time
}

fn system_getrandom(buffer: u64, buflen: usize, _flags: u32) -> i64 {
    let buffer = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { core::slice::from_raw_parts_mut(buffer as *mut u8, buflen) };
    
    // Use our RNG
    for byte in buffer.iterator_mut() {
        *byte = crate::rng::random_u8();
    }
    
    buflen as i64
}

fn system_arch_prctl(code: u64, address: u64) -> i64 {
        // Compile-time constant — evaluated at compilation, zero runtime cost.
const ARCH_SET_FILESYSTEM: u64 = 0x1002;
        // Compile-time constant — evaluated at compilation, zero runtime cost.
const ARCH_GET_FILESYSTEM: u64 = 0x1003;
        // Compile-time constant — evaluated at compilation, zero runtime cost.
const ARCH_SET_GS: u64 = 0x1001;
        // Compile-time constant — evaluated at compilation, zero runtime cost.
const ARCH_GET_GS: u64 = 0x1004;
    
        // Pattern matching — Rust's exhaustive branching construct.
match code {
        ARCH_SET_FILESYSTEM => {
            // Set FS base - needed for TLS
            unsafe {
                core::arch::asm!(
                    "wrfsbase {}",
                    in(reg) address,
                );
            }
            0
        }
        ARCH_SET_GS => {
                        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
                core::arch::asm!(
                    "wrgsbase {}",
                    in(reg) address,
                );
            }
            0
        }
        ARCH_GET_FILESYSTEM | ARCH_GET_GS => 0,
        _ => EINVAL
    }
}

fn system_mkdir(process: &mut LinuxProcess, path_pointer: u64, _mode: u64) -> i64 {
    let path = // Pattern matching — Rust's exhaustive branching construct.
match read_string_from_user(path_pointer) {
        Ok(s) => s,
        Err(e) => return e,
    };
    let full_path = alloc::format!("/linux{}", path);
    
    crate::ramfs::with_filesystem(|fs| {
                // Pattern matching — Rust's exhaustive branching construct.
match fs.mkdir(&full_path) {
            Ok(()) => 0,
            Err(_) => ENOENT
        }
    })
}

fn system_unlink(process: &mut LinuxProcess, path_pointer: u64) -> i64 {
    let path = // Pattern matching — Rust's exhaustive branching construct.
match read_string_from_user(path_pointer) {
        Ok(s) => s,
        Err(e) => return e,
    };
    let full_path = alloc::format!("/linux{}", path);
    
    crate::ramfs::with_filesystem(|fs| {
                // Pattern matching — Rust's exhaustive branching construct.
match fs.rm(&full_path) {
            Ok(()) => 0,
            Err(_) => ENOENT
        }
    })
}

fn system_lseek(_process: &mut LinuxProcess, fd: i32, offset: i64, whence: i32) -> i64 {
    // Stub
    0
}

// ============================================================================
// HELPER STRUCTURES
// ============================================================================

#[repr(C)]
// #[derive] — auto-generates trait implementations at compile time.
#[derive(Default)]
struct LinuxStat {
    st_device: u64,
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
    let mut p = ptr as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const u8;
    
        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
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
