//! System Call Interface
//!
//! Provides the syscall interface between userspace and kernel.
//! Implements Linux-compatible syscalls for binary compatibility.

pub mod linux;

use crate::memory::{validate_user_ptr, is_user_address};

/// Syscall numbers - re-export Linux numbers
pub mod nr {
    // Re-export Linux syscall numbers
    pub use super::linux::nr::*;
    
    // TrustOS-specific syscalls (high numbers to avoid conflicts)
    pub const SYS_DEBUG_PRINT: u64 = 0x1000;
    pub const SYS_TRUSTOS_IPC_SEND: u64 = 0x1001;
    pub const SYS_TRUSTOS_IPC_RECV: u64 = 0x1002;
    pub const SYS_TRUSTOS_IPC_CREATE: u64 = 0x1003;
}

/// Error codes (Linux-compatible, negative values)
pub mod errno {
    pub const EPERM: i64 = -1;
    pub const ENOENT: i64 = -2;
    pub const ESRCH: i64 = -3;
    pub const EINTR: i64 = -4;
    pub const EIO: i64 = -5;
    pub const ENXIO: i64 = -6;
    pub const E2BIG: i64 = -7;
    pub const ENOEXEC: i64 = -8;
    pub const EBADF: i64 = -9;
    pub const ECHILD: i64 = -10;
    pub const EAGAIN: i64 = -11;
    pub const ENOMEM: i64 = -12;
    pub const EACCES: i64 = -13;
    pub const EFAULT: i64 = -14;
    pub const ENOTBLK: i64 = -15;
    pub const EBUSY: i64 = -16;
    pub const EEXIST: i64 = -17;
    pub const EXDEV: i64 = -18;
    pub const ENODEV: i64 = -19;
    pub const ENOTDIR: i64 = -20;
    pub const EISDIR: i64 = -21;
    pub const EINVAL: i64 = -22;
    pub const ENFILE: i64 = -23;
    pub const EMFILE: i64 = -24;
    pub const ENOTTY: i64 = -25;
    pub const ETXTBSY: i64 = -26;
    pub const EFBIG: i64 = -27;
    pub const ENOSPC: i64 = -28;
    pub const ESPIPE: i64 = -29;
    pub const EROFS: i64 = -30;
    pub const EMLINK: i64 = -31;
    pub const EPIPE: i64 = -32;
    pub const EDOM: i64 = -33;
    pub const ERANGE: i64 = -34;
    pub const ENOSYS: i64 = -38;
    pub const EWOULDBLOCK: i64 = EAGAIN;
}

/// Initialize syscall interface
pub fn init() {
    crate::log!("[SYSCALL] Linux-compatible syscall interface initialized");
}

/// Handle a syscall (called from assembly handler or interrupt)
/// 
/// Linux x86_64 syscall ABI:
/// - rax = syscall number
/// - rdi = arg1, rsi = arg2, rdx = arg3, r10 = arg4, r8 = arg5, r9 = arg6
pub fn handle(num: u64, a1: u64, a2: u64, a3: u64) -> u64 {
    // Extended handle with all 6 args
    let ret = handle_full(num, a1, a2, a3, 0, 0, 0);

    // Emit structured syscall event to TrustLab trace bus
    crate::lab_mode::trace_bus::emit_syscall(num, [a1, a2, a3], ret);

    // Check for pending signals before returning to userspace
    let pid = crate::process::current_pid();
    if pid > 0 {
        if let Some(signo) = crate::signals::check_signals(pid) {
            // Signal with default terminate action was handled inside check_signals.
            // If process was terminated, return_from_ring3 if still active.
            if !crate::process::is_running(pid) && crate::userland::is_process_active() {
                unsafe { crate::userland::return_from_ring3(-(signo as i32)); }
            }
        }
    }

    ret as u64
}

/// Full syscall handler with all 6 arguments
pub fn handle_full(num: u64, a1: u64, a2: u64, a3: u64, a4: u64, a5: u64, a6: u64) -> i64 {
    use linux::nr::*;
    
    match num {
        // ====== File I/O ======
        READ => sys_read(a1 as i32, a2, a3 as usize),
        WRITE => sys_write(a1 as i32, a2, a3 as usize),
        OPEN => sys_open(a1, a2 as u32),
        CLOSE => sys_close(a1 as i32),
        STAT | LSTAT => linux::sys_fstat(a1 as i32, a2),
        FSTAT => linux::sys_fstat(a1 as i32, a2),
        LSEEK => sys_lseek(a1 as i32, a2 as i64, a3 as u32),
        READV => linux::sys_readv(a1 as i32, a2, a3 as u32),
        WRITEV => linux::sys_writev(a1 as i32, a2, a3 as u32),
        ACCESS => linux::sys_access(a1, a2 as u32),
        READLINK => linux::sys_readlink(a1, a2, a3),
        IOCTL => linux::sys_ioctl(a1 as i32, a2, a3),
        FCNTL => linux::sys_fcntl(a1 as i32, a2 as u32, a3),
        GETCWD => sys_getcwd(a1, a2 as usize),
        CHDIR => sys_chdir(a1),
        MKDIR => sys_mkdir(a1),
        UNLINK => sys_unlink(a1),
        DUP => linux::sys_dup(a1 as i32),
        DUP2 => linux::sys_dup2(a1 as i32, a2 as i32),
        DUP3 => linux::sys_dup2(a1 as i32, a2 as i32),
        POLL => linux::sys_poll(a1, a2 as u32, a3 as i32),
        GETDENTS64 => linux::sys_getdents64(a1 as i32, a2, a3 as u32),
        OPENAT => linux::sys_openat(a1 as i32, a2, a3 as u32),
        
        // ====== Memory Management ======
        MMAP => linux::sys_mmap(a1, a2, a3, a4, a5 as i64, a6),
        MUNMAP => linux::sys_munmap(a1, a2),
        MPROTECT => linux::sys_mprotect(a1, a2, a3),
        BRK => linux::sys_brk(a1),
        
        // ====== Process/Thread ======
        GETPID => linux::sys_getpid(),
        GETPPID => linux::sys_getppid(),
        GETTID => linux::sys_gettid(),
        GETUID | GETEUID => linux::sys_getuid(),
        GETGID | GETEGID => linux::sys_getgid(),
        SETUID => linux::sys_setuid(a1 as u32),
        SETGID => linux::sys_setgid(a1 as u32),
        SETREUID => linux::sys_setreuid(a1 as u32, a2 as u32),
        SETREGID => linux::sys_setregid(a1 as u32, a2 as u32),
        SETPGID => linux::sys_setpgid(a1 as u32, a2 as u32),
        GETPGRP => linux::sys_getpgrp(),
        SETSID => linux::sys_setsid(),
        GETPGID => linux::sys_getpgid(a1 as u32),
        GETSID => linux::sys_getsid(a1 as u32),
        CHROOT => linux::sys_chroot(a1),
        UMASK => linux::sys_umask(a1 as u32),
        CHMOD => linux::sys_chmod(a1, a2 as u32),
        FCHMOD => linux::sys_fchmod(a1 as i32, a2 as u32),
        CHOWN => linux::sys_chown(a1, a2 as u32, a3 as u32),
        FCHOWN => linux::sys_fchown(a1 as i32, a2 as u32, a3 as u32),
        LCHOWN => linux::sys_chown(a1, a2 as u32, a3 as u32),
        FORK => sys_fork(),
        VFORK => sys_fork(),
        CLONE => sys_clone(a1, a2, a3),
        EXECVE => sys_execve(a1, a2, a3),
        EXIT => { crate::process::exit(a1 as i32); if crate::userland::is_process_active() { unsafe { crate::userland::return_from_ring3(a1 as i32); } } 0 }
        EXIT_GROUP => { crate::process::exit(a1 as i32); if crate::userland::is_process_active() { unsafe { crate::userland::return_from_ring3(a1 as i32); } } 0 }
        WAIT4 => sys_wait4(a1 as i32, a2, a3 as u32),
        KILL => sys_kill(a1 as i32, a2 as i32),
        SET_TID_ADDRESS => linux::sys_set_tid_address(a1),
        
        // ====== Architecture ======
        ARCH_PRCTL => linux::sys_arch_prctl(a1, a2),
        
        // ====== System Info ======
        UNAME => linux::sys_uname(a1),
        
        // ====== Time ======
        CLOCK_GETTIME => linux::sys_clock_gettime(a1 as u32, a2),
        GETTIMEOFDAY => linux::sys_gettimeofday(a1, a2),
        NANOSLEEP => linux::sys_nanosleep(a1, a2),
        
        // ====== Signals ======
        RT_SIGACTION => linux::sys_rt_sigaction(a1 as u32, a2, a3, a4),
        RT_SIGPROCMASK => linux::sys_rt_sigprocmask(a1 as u32, a2, a3, a4),
        RT_SIGRETURN => 0,
        SIGALTSTACK => 0,
        
        // ====== Scheduling ======
        SCHED_YIELD => linux::sys_sched_yield(),
        SCHED_GETAFFINITY => linux::sys_sched_getaffinity(a1 as u32, a2, a3),
        SCHED_SETAFFINITY => 0,
        
        // ====== Synchronization ======
        FUTEX => sys_futex(a1, a2 as u32, a3 as u32),
        SET_ROBUST_LIST => linux::sys_set_robust_list(a1, a2),
        GET_ROBUST_LIST => linux::sys_get_robust_list(a1 as u32, a2, a3),
        
        // ====== Resources ======
        GETRLIMIT => linux::sys_getrlimit(a1 as u32, a2),
        SETRLIMIT => 0,
        PRLIMIT64 => linux::sys_prlimit64(a1 as u32, a2 as u32, a3, a4),
        
        // ====== Random ======
        GETRANDOM => linux::sys_getrandom(a1, a2, a3),
        
        // ====== Networking (BSD Sockets) ======
        SOCKET => sys_socket(a1 as u16, a2 as u32, a3 as u32),
        CONNECT => sys_connect(a1 as i32, a2, a3 as usize),
        ACCEPT => sys_accept(a1 as i32, a2, a3),
        SENDTO => sys_sendto(a1 as i32, a2, a3 as usize, a4 as u32, a5, a6 as usize),
        RECVFROM => sys_recvfrom(a1 as i32, a2, a3 as usize, a4 as u32, a5, a6),
        SHUTDOWN => sys_shutdown(a1 as i32, a2 as i32),
        BIND => sys_bind(a1 as i32, a2, a3 as usize),
        LISTEN => sys_listen(a1 as i32, a2 as u32),
        GETSOCKNAME => 0, // TODO
        GETPEERNAME => 0, // TODO
        SETSOCKOPT => sys_setsockopt(a1 as i32, a2 as i32, a3 as i32, a4, a5 as usize),
        GETSOCKOPT => sys_getsockopt(a1 as i32, a2 as i32, a3 as i32, a4, a5),
        SENDMSG => errno::ENOSYS, // TODO
        RECVMSG => errno::ENOSYS, // TODO
        
        // ====== Pipes ======
        PIPE2 => sys_pipe2(a1, a2 as u32),
        
        // ====== Misc ======
        PRCTL => linux::sys_prctl(a1 as u32, a2, a3, a4, a5),
        SWAPON => linux::sys_swapon(a1),
        SWAPOFF => linux::sys_swapoff(a1),
        
        // ====== TrustOS-specific ======
        nr::SYS_DEBUG_PRINT => sys_debug_print(a1, a2 as usize),
        nr::SYS_TRUSTOS_IPC_SEND => { crate::ipc::send_raw(a1); 0 },
        nr::SYS_TRUSTOS_IPC_RECV => crate::ipc::receive_raw(a1) as i64,
        nr::SYS_TRUSTOS_IPC_CREATE => crate::ipc::create_channel_raw() as i64,
        
        _ => {
            crate::log_debug!("[SYSCALL] Unknown: {} (0x{:x})", num, num);
            errno::ENOSYS
        }
    }
}

fn sys_read(fd: i32, buf_ptr: u64, count: usize) -> i64 {
    if buf_ptr == 0 || count == 0 {
        return errno::EINVAL;
    }
    
    // Validate user pointer - must be in user space and writable
    if !validate_user_ptr(buf_ptr, count, true) {
        crate::log_warn!("[SYSCALL] read: invalid user pointer {:#x}", buf_ptr);
        return errno::EFAULT;
    }
    
    // Pipe fd?
    if crate::pipe::is_pipe_fd(fd) {
        let buffer = unsafe { core::slice::from_raw_parts_mut(buf_ptr as *mut u8, count) };
        return crate::pipe::read(fd, buffer);
    }
    
    let buffer = unsafe { core::slice::from_raw_parts_mut(buf_ptr as *mut u8, count) };
    match crate::vfs::read(fd, buffer) {
        Ok(n) => n as i64,
        Err(_) => errno::EIO,
    }
}

fn sys_write(fd: i32, buf_ptr: u64, count: usize) -> i64 {
    if buf_ptr == 0 || count == 0 {
        return errno::EINVAL;
    }
    
    // Validate user pointer - must be in user space and readable
    if !validate_user_ptr(buf_ptr, count, false) {
        crate::log_warn!("[SYSCALL] write: invalid user pointer {:#x}", buf_ptr);
        return errno::EFAULT;
    }
    
    let buffer = unsafe { core::slice::from_raw_parts(buf_ptr as *const u8, count) };
    
    // stdout/stderr go to serial
    if fd == 1 || fd == 2 {
        for &b in buffer { crate::serial_print!("{}", b as char); }
        return count as i64;
    }
    
    // Pipe fd?
    if crate::pipe::is_pipe_fd(fd) {
        return crate::pipe::write(fd, buffer);
    }
    
    match crate::vfs::write(fd, buffer) {
        Ok(n) => n as i64,
        Err(_) => errno::EIO,
    }
}

fn sys_open(path_ptr: u64, flags: u32) -> i64 {
    let path = match read_cstring(path_ptr, 256) {
        Some(s) => s,
        None => return errno::EFAULT,
    };
    match crate::vfs::open(&path, crate::vfs::OpenFlags(flags)) {
        Ok(fd) => fd as i64,
        Err(_) => errno::ENOENT,
    }
}

fn sys_close(fd: i32) -> i64 {
    // Pipe fd?
    if crate::pipe::is_pipe_fd(fd) {
        return crate::pipe::close(fd);
    }
    match crate::vfs::close(fd) {
        Ok(()) => 0,
        Err(_) => errno::EBADF,
    }
}

/// pipe2(pipefd, flags) — create a pipe, write [read_fd, write_fd] to user buffer
fn sys_pipe2(pipefd_ptr: u64, _flags: u32) -> i64 {
    if !validate_user_ptr(pipefd_ptr, 8, true) {
        return errno::EFAULT;
    }
    let (read_fd, write_fd) = crate::pipe::create();
    unsafe {
        let ptr = pipefd_ptr as *mut i32;
        *ptr = read_fd;
        *ptr.add(1) = write_fd;
    }
    crate::log_debug!("[SYSCALL] pipe2: read_fd={}, write_fd={}", read_fd, write_fd);
    0
}

fn sys_mkdir(path_ptr: u64) -> i64 {
    let path = match read_cstring(path_ptr, 256) {
        Some(s) => s,
        None => return errno::EFAULT,
    };
    match crate::vfs::mkdir(&path) {
        Ok(()) => 0,
        Err(_) => errno::EIO,
    }
}

fn sys_unlink(path_ptr: u64) -> i64 {
    let path = match read_cstring(path_ptr, 256) {
        Some(s) => s,
        None => return errno::EFAULT,
    };
    match crate::vfs::unlink(&path) {
        Ok(()) => 0,
        Err(_) => errno::ENOENT,
    }
}

fn sys_debug_print(buf_ptr: u64, len: usize) -> i64 {
    if buf_ptr == 0 { return errno::EFAULT; }
    
    // Validate user pointer
    if !validate_user_ptr(buf_ptr, len, false) {
        return errno::EFAULT;
    }
    
    let buffer = unsafe { core::slice::from_raw_parts(buf_ptr as *const u8, len) };
    for &b in buffer { crate::serial_print!("{}", b as char); }
    len as i64
}

/// Safely read a C string from user space
fn read_cstring(ptr: u64, max: usize) -> Option<alloc::string::String> {
    if ptr == 0 { return None; }
    
    // Validate the pointer is in user space
    if !is_user_address(ptr) {
        crate::log_warn!("[SYSCALL] read_cstring: kernel address {:#x}", ptr);
        return None;
    }
    
    let mut s = alloc::string::String::new();
    for i in 0..max {
        // Check each byte access is valid
        let byte_addr = ptr + i as u64;
        if !is_user_address(byte_addr) {
            return None;
        }
        
        let b = unsafe { *(byte_addr as *const u8) };
        if b == 0 { break; }
        s.push(b as char);
    }
    if s.is_empty() { None } else { Some(s) }
}

/// Clone syscall - create a new thread
/// 
/// Arguments:
/// - flags: Clone flags (CLONE_THREAD, etc.)
/// - stack: User stack pointer for new thread
/// - entry: Entry point for new thread (or parent_tid ptr)
/// 
/// Returns: Thread ID on success, negative errno on failure
fn sys_clone(flags: u64, stack: u64, entry: u64) -> i64 {
    // For now, simple thread creation
    // flags are ignored, we always create a new thread
    
    if stack == 0 || entry == 0 {
        return errno::EINVAL;
    }
    
    // Validate user pointers
    if !is_user_address(stack) || !is_user_address(entry) {
        return errno::EFAULT;
    }
    
    let pid = crate::process::current_pid();
    let tid = crate::thread::spawn_user(pid, "user_thread", entry, stack, 0);
    
    tid as i64
}

/// Futex syscall - fast userspace mutex
/// 
/// Operations:
/// - FUTEX_WAIT (0): Wait if *addr == val
/// - FUTEX_WAKE (1): Wake up to val waiters
fn sys_futex(addr: u64, op: u32, val: u32) -> i64 {
    const FUTEX_WAIT: u32 = 0;
    const FUTEX_WAKE: u32 = 1;
    const FUTEX_PRIVATE_FLAG: u32 = 128;
    
    let op = op & !FUTEX_PRIVATE_FLAG;
    
    if !is_user_address(addr) {
        return errno::EFAULT;
    }
    
    match op {
        FUTEX_WAIT => {
            // Check if value matches
            let current = unsafe { *(addr as *const u32) };
            if current != val {
                return errno::EAGAIN;
            }
            
            // Block current thread
            let _tid = crate::thread::current_tid();
            // In a real implementation, we'd add to a hash table of waiters
            // For now, just yield
            crate::thread::yield_thread();
            0
        }
        FUTEX_WAKE => {
            // Wake up to 'val' waiters
            // In a real implementation, we'd look up waiters in hash table
            // For now, just return 0 (no one woken)
            0
        }
        _ => errno::ENOSYS,
    }
}

// ============================================================================
// Additional syscall implementations for Linux compatibility
// ============================================================================

fn sys_lseek(fd: i32, offset: i64, whence: u32) -> i64 {
    match crate::vfs::lseek(fd, offset, whence) {
        Ok(pos) => pos as i64,
        Err(_) => errno::EINVAL,
    }
}

fn sys_getcwd(buf: u64, size: usize) -> i64 {
    if !validate_user_ptr(buf, size, true) {
        return errno::EFAULT;
    }
    
    let cwd = crate::vfs::getcwd();
    let bytes = cwd.as_bytes();
    let len = bytes.len().min(size - 1);
    
    unsafe {
        let dst = core::slice::from_raw_parts_mut(buf as *mut u8, size);
        dst[..len].copy_from_slice(&bytes[..len]);
        dst[len] = 0;
    }
    
    buf as i64
}

fn sys_chdir(path_ptr: u64) -> i64 {
    let path = match read_cstring(path_ptr, 256) {
        Some(s) => s,
        None => return errno::EFAULT,
    };
    
    match crate::vfs::chdir(&path) {
        Ok(()) => 0,
        Err(_) => errno::ENOENT,
    }
}

fn sys_fork() -> i64 {
    match crate::process::fork() {
        Ok(pid) => pid as i64,
        Err(_) => errno::ENOMEM,
    }
}

fn sys_execve(pathname: u64, _argv: u64, _envp: u64) -> i64 {
    let path = match read_cstring(pathname, 256) {
        Some(s) => s,
        None => return errno::EFAULT,
    };
    
    // Parse argv from user memory (null-terminated array of char*)
    let mut argv_strs: alloc::vec::Vec<alloc::string::String> = alloc::vec::Vec::new();
    if _argv != 0 && is_user_address(_argv) {
        for i in 0..256u64 {
            let ptr_addr = _argv + i * 8;
            if !is_user_address(ptr_addr) { break; }
            let ptr = unsafe { *(ptr_addr as *const u64) };
            if ptr == 0 { break; }
            if let Some(s) = read_cstring(ptr, 256) {
                argv_strs.push(s);
            } else {
                break;
            }
        }
    }
    if argv_strs.is_empty() {
        argv_strs.push(path.clone());
    }
    let argv_refs: alloc::vec::Vec<&str> = argv_strs.iter().map(|s| s.as_str()).collect();
    
    match crate::exec::execve(&path, &argv_refs, &[]) {
        Ok(()) => 0, // Never returns on success
        Err(_) => errno::ENOENT,
    }
}

fn sys_wait4(pid: i32, wstatus: u64, options: u32) -> i64 {
    let target_pid = if pid > 0 { pid as u32 } else { 0 };
    let wnohang = options & 1 != 0; // WNOHANG
    
    // Blocking wait: retry with yields (up to ~5 s)
    let max_tries: u32 = if wnohang { 1 } else { 5000 };
    
    for _ in 0..max_tries {
        match crate::process::wait(target_pid) {
            Ok(status) => {
                if wstatus != 0 && validate_user_ptr(wstatus, 4, true) {
                    // Linux encodes exit status as (code << 8)
                    unsafe { *(wstatus as *mut i32) = (status & 0xFF) << 8; }
                }
                return target_pid as i64;
            }
            Err(_) => {
                if wnohang { return 0; }
                crate::thread::yield_thread();
            }
        }
    }
    errno::ECHILD
}

fn sys_kill(pid: i32, sig: i32) -> i64 {
    if pid <= 0 {
        return errno::EINVAL;
    }
    
    let sender_pid = crate::process::current_pid();
    
    match crate::signals::kill(pid as u32, sig as u32, sender_pid) {
        Ok(()) => 0,
        Err(e) => e as i64,
    }
}

// ============================================================================
// Network Syscalls (BSD Socket API)
// ============================================================================

/// Create a socket
fn sys_socket(domain: u16, sock_type: u32, protocol: u32) -> i64 {
    match crate::netstack::socket::socket(domain, sock_type, protocol) {
        Ok(fd) => fd as i64,
        Err(e) => e as i64,
    }
}

/// Connect to a remote address
fn sys_connect(fd: i32, addr_ptr: u64, addr_len: usize) -> i64 {
    use crate::netstack::socket::SockAddrIn;
    
    if addr_len < SockAddrIn::SIZE {
        return errno::EINVAL;
    }
    
    if !validate_user_ptr(addr_ptr, addr_len, false) {
        return errno::EFAULT;
    }
    
    let addr = unsafe { *(addr_ptr as *const SockAddrIn) };
    
    match crate::netstack::socket::connect(fd, &addr) {
        Ok(()) => 0,
        Err(e) => e as i64,
    }
}

/// Bind socket to local address
fn sys_bind(fd: i32, addr_ptr: u64, addr_len: usize) -> i64 {
    use crate::netstack::socket::SockAddrIn;
    
    if addr_len < SockAddrIn::SIZE {
        return errno::EINVAL;
    }
    
    if !validate_user_ptr(addr_ptr, addr_len, false) {
        return errno::EFAULT;
    }
    
    let addr = unsafe { *(addr_ptr as *const SockAddrIn) };
    
    match crate::netstack::socket::bind(fd, &addr) {
        Ok(()) => 0,
        Err(e) => e as i64,
    }
}

/// Listen for connections
fn sys_listen(fd: i32, backlog: u32) -> i64 {
    match crate::netstack::socket::listen(fd, backlog) {
        Ok(()) => 0,
        Err(e) => e as i64,
    }
}

/// Accept a connection
fn sys_accept(fd: i32, addr_ptr: u64, addr_len_ptr: u64) -> i64 {
    match crate::netstack::socket::accept(fd, addr_ptr, addr_len_ptr) {
        Ok(new_fd) => new_fd as i64,
        Err(e) => e as i64,
    }
}

/// Send data to connected socket
fn sys_sendto(fd: i32, buf_ptr: u64, len: usize, flags: u32, addr_ptr: u64, addr_len: usize) -> i64 {
    use crate::netstack::socket::SockAddrIn;
    
    if !validate_user_ptr(buf_ptr, len, false) {
        return errno::EFAULT;
    }
    
    let data = unsafe { core::slice::from_raw_parts(buf_ptr as *const u8, len) };
    
    if addr_ptr != 0 && addr_len >= SockAddrIn::SIZE {
        // sendto with address
        if !validate_user_ptr(addr_ptr, addr_len, false) {
            return errno::EFAULT;
        }
        let addr = unsafe { *(addr_ptr as *const SockAddrIn) };
        match crate::netstack::socket::sendto(fd, data, flags, &addr) {
            Ok(n) => n as i64,
            Err(e) => e as i64,
        }
    } else {
        // send (no address, use connected socket)
        match crate::netstack::socket::send(fd, data, flags) {
            Ok(n) => n as i64,
            Err(e) => e as i64,
        }
    }
}

/// Receive data from socket
fn sys_recvfrom(fd: i32, buf_ptr: u64, len: usize, flags: u32, addr_ptr: u64, addr_len_ptr: u64) -> i64 {
    if !validate_user_ptr(buf_ptr, len, true) {
        return errno::EFAULT;
    }
    
    let buf = unsafe { core::slice::from_raw_parts_mut(buf_ptr as *mut u8, len) };
    
    // For now, just use recv (ignore address output)
    match crate::netstack::socket::recv(fd, buf, flags) {
        Ok(n) => n as i64,
        Err(e) => e as i64,
    }
}

/// Shutdown socket
fn sys_shutdown(fd: i32, _how: i32) -> i64 {
    match crate::netstack::socket::close(fd) {
        Ok(()) => 0,
        Err(e) => e as i64,
    }
}

/// Set socket option
fn sys_setsockopt(fd: i32, level: i32, optname: i32, optval: u64, optlen: usize) -> i64 {
    if optlen > 0 && !validate_user_ptr(optval, optlen, false) {
        return errno::EFAULT;
    }
    
    let data = if optlen > 0 {
        unsafe { core::slice::from_raw_parts(optval as *const u8, optlen) }
    } else {
        &[]
    };
    
    match crate::netstack::socket::setsockopt(fd, level, optname, data) {
        Ok(()) => 0,
        Err(e) => e as i64,
    }
}

/// Get socket option
fn sys_getsockopt(fd: i32, level: i32, optname: i32, optval: u64, optlen_ptr: u64) -> i64 {
    if optval == 0 || optlen_ptr == 0 {
        return errno::EFAULT;
    }
    
    if !validate_user_ptr(optlen_ptr, 4, true) {
        return errno::EFAULT;
    }
    
    let optlen = unsafe { *(optlen_ptr as *const u32) } as usize;
    
    if optlen > 0 && !validate_user_ptr(optval, optlen, true) {
        return errno::EFAULT;
    }
    
    let buf = unsafe { core::slice::from_raw_parts_mut(optval as *mut u8, optlen) };
    
    match crate::netstack::socket::getsockopt(fd, level, optname, buf) {
        Ok(len) => {
            unsafe { *(optlen_ptr as *mut u32) = len as u32; }
            0
        }
        Err(e) => e as i64,
    }
}
