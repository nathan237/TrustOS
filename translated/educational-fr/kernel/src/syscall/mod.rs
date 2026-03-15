//! System Call Interface
//!
//! Provides the syscall interface between userspace and kernel.
//! Implements Linux-compatible syscalls for binary compatibility.

pub mod linux;

use crate::memory::{validate_user_pointer, is_user_address};

/// Syscall numbers - re-export Linux numbers
pub mod nr {
    // Re-export Linux syscall numbers
    pub use super::linux::nr::*;
    
    // TrustOS-specific syscalls (high numbers to avoid conflicts)
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SYSTEM_DEBUG_PRINT: u64 = 0x1000;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SYSTEM_TRUSTOS_IPC_SEND: u64 = 0x1001;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SYSTEM_TRUSTOS_IPC_RECV: u64 = 0x1002;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SYSTEM_TRUSTOS_IPC_CREATE: u64 = 0x1003;
}

/// Error codes (Linux-compatible, negative values)
pub mod errno {
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const EPERM: i64 = -1;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const ENOENT: i64 = -2;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const ESRCH: i64 = -3;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const EINTR: i64 = -4;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const EIO: i64 = -5;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const ENXIO: i64 = -6;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const E2BIG: i64 = -7;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const ENOEXEC: i64 = -8;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const EBADF: i64 = -9;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const ECHILD: i64 = -10;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const EAGAIN: i64 = -11;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const ENOMEM: i64 = -12;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const EACCES: i64 = -13;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const EFAULT: i64 = -14;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const ENOTBLK: i64 = -15;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const EBUSY: i64 = -16;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const EEXIST: i64 = -17;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const EXDEV: i64 = -18;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const ENODEV: i64 = -19;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const ENOTDIR: i64 = -20;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const EISDIR: i64 = -21;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const EINVAL: i64 = -22;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const ENFILE: i64 = -23;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const EMFILE: i64 = -24;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const ENOTTY: i64 = -25;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const ETXTBSY: i64 = -26;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const EFBIG: i64 = -27;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const ENOSPC: i64 = -28;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const ESPIPE: i64 = -29;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const EROFS: i64 = -30;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const EMLINK: i64 = -31;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const EPIPE: i64 = -32;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const EDOM: i64 = -33;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const ERANGE: i64 = -34;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const ENOSYS: i64 = -38;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const EWOULDBLOCK: i64 = EAGAIN;
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
    let return_value = handle_full(num, a1, a2, a3, 0, 0, 0);

    // Emit structured syscall event to TrustLab trace bus
    crate::lab_mode::trace_bus::emit_syscall(num, [a1, a2, a3], return_value);

    // Check for pending signals before returning to userspace
    let pid = crate::process::current_pid();
    if pid > 0 {
        if let Some(signo) = crate::signals::check_signals(pid) {
            // Signal with default terminate action was handled inside check_signals.
            // If process was terminated, return_from_ring3 if still active.
            if !crate::process::is_running(pid) && crate::userland::is_process_active() {
                                // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { crate::userland::return_from_ring3(-(signo as i32)); }
            }
        }
    }

    return_value as u64
}

/// Full syscall handler with all 6 arguments
pub fn handle_full(num: u64, a1: u64, a2: u64, a3: u64, a4: u64, a5: u64, a6: u64) -> i64 {
    use linux::nr::*;
    
        // Correspondance de motifs — branchement exhaustif de Rust.
match num {
        // ====== File I/O ======
        READ => system_read(a1 as i32, a2, a3 as usize),
        WRITE => system_write(a1 as i32, a2, a3 as usize),
        OPEN => system_open(a1, a2 as u32),
        CLOSE => system_close(a1 as i32),
        STAT | LSTAT => linux::system_status(a1, a2),
        FSTAT => linux::system_fstat(a1 as i32, a2),
        LSEEK => system_lseek(a1 as i32, a2 as i64, a3 as u32),
        READV => linux::system_readv(a1 as i32, a2, a3 as u32),
        WRITEV => linux::system_writev(a1 as i32, a2, a3 as u32),
        ACCESS => linux::system_access(a1, a2 as u32),
        READLINK => linux::system_readlink(a1, a2, a3),
        IOCTL => linux::system_ioctl(a1 as i32, a2, a3),
        FCNTL => linux::system_fcntl(a1 as i32, a2 as u32, a3),
        GETCWD => system_getcwd(a1, a2 as usize),
        CHDIR => system_chdir(a1),
        MKDIR => system_mkdir(a1),
        UNLINK => system_unlink(a1),
        DUP => linux::system_dup(a1 as i32),
        DUP2 => linux::system_dup2(a1 as i32, a2 as i32),
        DUP3 => linux::system_dup2(a1 as i32, a2 as i32),
        POLL => linux::system_poll(a1, a2 as u32, a3 as i32),
        GETDENTS64 => linux::system_getdents64(a1 as i32, a2, a3 as u32),
        OPENAT => linux::system_openat(a1 as i32, a2, a3 as u32),
        NEWFSTATAT => linux::system_newfstatat(a1 as i32, a2, a3, a4 as u32),
        
        // ====== Memory Management ======
        MMAP => linux::system_mmap(a1, a2, a3, a4, a5 as i64, a6),
        MUNMAP => linux::system_munmap(a1, a2),
        MPROTECT => linux::system_mprotect(a1, a2, a3),
        BRK => linux::system_brk(a1),
        
        // ====== Process/Thread ======
        GETPID => linux::system_getpid(),
        GETPPID => linux::system_getppid(),
        GETTID => linux::system_gettid(),
        GETUID | GETEUID => linux::system_getuid(),
        GETGID | GETEGID => linux::system_getgid(),
        SETUID => linux::system_setuid(a1 as u32),
        SETGID => linux::system_setgid(a1 as u32),
        SETREUID => linux::system_setreuid(a1 as u32, a2 as u32),
        SETREGID => linux::system_setregid(a1 as u32, a2 as u32),
        SETPGID => linux::system_setpgid(a1 as u32, a2 as u32),
        GETPGRP => linux::system_getpgrp(),
        SETSID => linux::system_setsid(),
        GETPGID => linux::system_getpgid(a1 as u32),
        GETSID => linux::system_getsid(a1 as u32),
        CHROOT => linux::system_chroot(a1),
        UMASK => linux::system_umask(a1 as u32),
        CHMOD => linux::system_chmod(a1, a2 as u32),
        FCHMOD => linux::system_fchmod(a1 as i32, a2 as u32),
        CHOWN => linux::system_chown(a1, a2 as u32, a3 as u32),
        FCHOWN => linux::system_fchown(a1 as i32, a2 as u32, a3 as u32),
        LCHOWN => linux::system_chown(a1, a2 as u32, a3 as u32),
        FORK => system_fork(),
        VFORK => system_fork(),
        CLONE => system_clone(a1, a2, a3),
        EXECVE => system_execve(a1, a2, a3),
        EXIT => { 
            crate::process::exit(a1 as i32);
            if crate::userland::is_process_active() { 
                // Check if this is a scheduler-managed user thread
                // (has a waiting kernel thread) or a legacy exec_ring3 process
                if crate::userland::WAITING_KERNEL_TID.load(core::sync::atomic::Ordering::SeqCst) != 0 {
                    crate::userland::user_thread_exit(a1 as i32);
                } else {
                                        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { crate::userland::return_from_ring3(a1 as i32); }
                }
            }
            0
        }
        EXIT_GROUP => { 
            crate::process::exit(a1 as i32);
            if crate::userland::is_process_active() { 
                if crate::userland::WAITING_KERNEL_TID.load(core::sync::atomic::Ordering::SeqCst) != 0 {
                    crate::userland::user_thread_exit(a1 as i32);
                } else {
                                        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { crate::userland::return_from_ring3(a1 as i32); }
                }
            }
            0
        }
        WAIT4 => system_wait4(a1 as i32, a2, a3 as u32),
        KILL => system_kill(a1 as i32, a2 as i32),
        SET_TID_ADDRESS => linux::system_set_tid_address(a1),
        
        // ====== Architecture ======
        ARCH_PRCTL => linux::system_arch_prctl(a1, a2),
        
        // ====== System Info ======
        UNAME => linux::system_uname(a1),
        
        // ====== Time ======
        CLOCK_GETTIME => linux::system_clock_gettime(a1 as u32, a2),
        GETTIMEOFDAY => linux::system_gettimeofday(a1, a2),
        NANOSLEEP => linux::system_nanosleep(a1, a2),
        
        // ====== Signals ======
        RT_SIGACTION => linux::system_rt_sigaction(a1 as u32, a2, a3, a4),
        RT_SIGPROCMASK => linux::system_rt_sigprocmask(a1 as u32, a2, a3, a4),
        RT_SIGRETURN => 0,
        SIGALTSTACK => 0,
        
        // ====== Scheduling ======
        SCHED_YIELD => linux::sys_sched_yield(),
        SCHEDULER_GETAFFINITY => linux::system_scheduler_getaffinity(a1 as u32, a2, a3),
        SCHEDULER_SETAFFINITY => 0,
        
        // ====== Synchronization ======
        FUTEX => system_futex(a1, a2 as u32, a3 as u32),
        SET_ROBUST_LIST => linux::system_set_robust_list(a1, a2),
        GET_ROBUST_LIST => linux::system_get_robust_list(a1 as u32, a2, a3),
        
        // ====== Resources ======
        GETRLIMIT => linux::system_getrlimit(a1 as u32, a2),
        SETRLIMIT => 0,
        PRLIMIT64 => linux::system_prlimit64(a1 as u32, a2 as u32, a3, a4),
        
        // ====== Random ======
        GETRANDOM => linux::system_getrandom(a1, a2, a3),
        
        // ====== Networking (BSD Sockets) ======
        SOCKET => system_socket(a1 as u16, a2 as u32, a3 as u32),
        CONNECT => system_connect(a1 as i32, a2, a3 as usize),
        ACCEPT => system_accept(a1 as i32, a2, a3),
        SENDTO => system_sendto(a1 as i32, a2, a3 as usize, a4 as u32, a5, a6 as usize),
        RECVFROM => system_recvfrom(a1 as i32, a2, a3 as usize, a4 as u32, a5, a6),
        SHUTDOWN => system_shutdown(a1 as i32, a2 as i32),
        BIND => system_bind(a1 as i32, a2, a3 as usize),
        LISTEN => system_listen(a1 as i32, a2 as u32),
        GETSOCKNAME => system_getsockname(a1 as i32, a2, a3),
        GETPEERNAME => system_getpeername(a1 as i32, a2, a3),
        SETSOCKOPT => system_setsockopt(a1 as i32, a2 as i32, a3 as i32, a4, a5 as usize),
        GETSOCKOPT => system_getsockopt(a1 as i32, a2 as i32, a3 as i32, a4, a5),
        SENDMSG => system_sendmsg(a1 as i32, a2, a3 as u32),
        RECVMSG => system_recvmsg(a1 as i32, a2, a3 as u32),
        
        // ====== Pipes ======
        PIPE2 => system_pipe2(a1, a2 as u32),
        
        // ====== Epoll ======
        EPOLL_CREATE => linux::system_epoll_create(a1 as i32),
        EPOLL_WAIT => linux::system_epoll_wait(a1 as i32, a2, a3 as i32, a4 as i32),
        EPOLL_CONTROLLER => linux::system_epoll_controller(a1 as i32, a2 as i32, a3 as i32, a4),
        EPOLL_PWAIT => linux::system_epoll_pwait(a1 as i32, a2, a3 as i32, a4 as i32, a5, a6),
        EPOLL_CREATE1 => linux::system_epoll_create1(a1 as u32),
        
        // ====== Misc ======
        PRCTL => linux::system_prctl(a1 as u32, a2, a3, a4, a5),
        SWAPON => linux::system_swapon(a1),
        SWAPOFF => linux::system_swapoff(a1),
        
        // ====== TrustOS-specific ======
        nr::SYSTEM_DEBUG_PRINT => system_debug_print(a1, a2 as usize),
        nr::SYSTEM_TRUSTOS_IPC_SEND => { crate::ipc::send_raw(a1); 0 },
        nr::SYSTEM_TRUSTOS_IPC_RECV => crate::ipc::receive_raw(a1) as i64,
        nr::SYSTEM_TRUSTOS_IPC_CREATE => crate::ipc::create_channel_raw() as i64,
        
        _ => {
            crate::log_debug!("[SYSCALL] Unknown: {} (0x{:x})", num, num);
            errno::ENOSYS
        }
    }
}

fn system_read(fd: i32, buffer_pointer: u64, count: usize) -> i64 {
    if buffer_pointer == 0 || count == 0 {
        return errno::EINVAL;
    }
    
    // Validate user pointer - must be in user space and writable
    if !validate_user_pointer(buffer_pointer, count, true) {
        crate::log_warn!("[SYSCALL] read: invalid user pointer {:#x}", buffer_pointer);
        return errno::EFAULT;
    }
    
    // Pipe fd?
    if crate::pipe::is_pipe_fd(fd) {
        let buffer = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { core::slice::from_raw_parts_mut(buffer_pointer as *mut u8, count) };
        return crate::pipe::read(fd, buffer);
    }
    
    let buffer = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { core::slice::from_raw_parts_mut(buffer_pointer as *mut u8, count) };
        // Correspondance de motifs — branchement exhaustif de Rust.
match crate::vfs::read(fd, buffer) {
        Ok(n) => n as i64,
        Err(_) => errno::EIO,
    }
}

fn system_write(fd: i32, buffer_pointer: u64, count: usize) -> i64 {
    if buffer_pointer == 0 || count == 0 {
        return errno::EINVAL;
    }
    
    // Validate user pointer - must be in user space and readable
    if !validate_user_pointer(buffer_pointer, count, false) {
        crate::log_warn!("[SYSCALL] write: invalid user pointer {:#x}", buffer_pointer);
        return errno::EFAULT;
    }
    
    let buffer = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { core::slice::from_raw_parts(buffer_pointer as *// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const u8, count) };
    
    // stdout/stderr go to serial
    if fd == 1 || fd == 2 {
        for &b in buffer { crate::serial_print!("{}", b as char); }
        return count as i64;
    }
    
    // Pipe fd?
    if crate::pipe::is_pipe_fd(fd) {
        return crate::pipe::write(fd, buffer);
    }
    
        // Correspondance de motifs — branchement exhaustif de Rust.
match crate::vfs::write(fd, buffer) {
        Ok(n) => n as i64,
        Err(_) => errno::EIO,
    }
}

fn system_open(path_pointer: u64, flags: u32) -> i64 {
    let path = // Correspondance de motifs — branchement exhaustif de Rust.
match read_cstring(path_pointer, 256) {
        Some(s) => s,
        None => return errno::EFAULT,
    };
        // Correspondance de motifs — branchement exhaustif de Rust.
match crate::vfs::open(&path, crate::vfs::OpenFlags(flags)) {
        Ok(fd) => fd as i64,
        Err(_) => errno::ENOENT,
    }
}

fn system_close(fd: i32) -> i64 {
    // Epoll fd?
    if linux::is_epoll_fd(fd) {
        crate::syscall::linux::EPOLL_TABLE.lock().remove(&fd);
        return 0;
    }
    // Pipe fd?
    if crate::pipe::is_pipe_fd(fd) {
        return crate::pipe::close(fd);
    }
        // Correspondance de motifs — branchement exhaustif de Rust.
match crate::vfs::close(fd) {
        Ok(()) => 0,
        Err(_) => errno::EBADF,
    }
}

/// pipe2(pipefd, flags) — create a pipe, write [read_fd, write_fd] to user buffer
fn system_pipe2(pipefd_pointer: u64, _flags: u32) -> i64 {
    if !validate_user_pointer(pipefd_pointer, 8, true) {
        return errno::EFAULT;
    }
    let (read_fd, write_fd) = crate::pipe::create();
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
        let ptr = pipefd_pointer as *mut i32;
        *ptr = read_fd;
        *ptr.add(1) = write_fd;
    }
    crate::log_debug!("[SYSCALL] pipe2: read_fd={}, write_fd={}", read_fd, write_fd);
    0
}

fn system_mkdir(path_pointer: u64) -> i64 {
    let path = // Correspondance de motifs — branchement exhaustif de Rust.
match read_cstring(path_pointer, 256) {
        Some(s) => s,
        None => return errno::EFAULT,
    };
        // Correspondance de motifs — branchement exhaustif de Rust.
match crate::vfs::mkdir(&path) {
        Ok(()) => 0,
        Err(_) => errno::EIO,
    }
}

fn system_unlink(path_pointer: u64) -> i64 {
    let path = // Correspondance de motifs — branchement exhaustif de Rust.
match read_cstring(path_pointer, 256) {
        Some(s) => s,
        None => return errno::EFAULT,
    };
        // Correspondance de motifs — branchement exhaustif de Rust.
match crate::vfs::unlink(&path) {
        Ok(()) => 0,
        Err(_) => errno::ENOENT,
    }
}

fn system_debug_print(buffer_pointer: u64, len: usize) -> i64 {
    if buffer_pointer == 0 { return errno::EFAULT; }
    
    // Validate user pointer
    if !validate_user_pointer(buffer_pointer, len, false) {
        return errno::EFAULT;
    }
    
    let buffer = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { core::slice::from_raw_parts(buffer_pointer as *// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const u8, len) };
    for &b in buffer { crate::serial_print!("{}", b as char); }
    len as i64
}

/// Safely read a C string from user space
fn read_cstring(ptr: u64, maximum: usize) -> Option<alloc::string::String> {
    if ptr == 0 { return None; }
    
    // Validate the pointer is in user space
    if !is_user_address(ptr) {
        crate::log_warn!("[SYSCALL] read_cstring: kernel address {:#x}", ptr);
        return None;
    }
    
    let mut s = alloc::string::String::new();
    for i in 0..maximum {
        // Check each byte access is valid
        let byte_address = ptr + i as u64;
        if !is_user_address(byte_address) {
            return None;
        }
        
        let b = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { *(byte_address as *// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const u8) };
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
fn system_clone(flags: u64, stack: u64, entry: u64) -> i64 {
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
fn system_futex(address: u64, op: u32, value: u32) -> i64 {
        // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const FUTEX_WAIT: u32 = 0;
        // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const FUTEX_WAKE: u32 = 1;
        // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const FUTEX_PRIVATE_FLAG: u32 = 128;
    
    let op = op & !FUTEX_PRIVATE_FLAG;
    
    if !is_user_address(address) {
        return errno::EFAULT;
    }
    
        // Correspondance de motifs — branchement exhaustif de Rust.
match op {
        FUTEX_WAIT => {
            // Check if value matches
            let current = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { *(address as *// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const u32) };
            if current != value {
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

fn system_lseek(fd: i32, offset: i64, whence: u32) -> i64 {
        // Correspondance de motifs — branchement exhaustif de Rust.
match crate::vfs::lseek(fd, offset, whence) {
        Ok(position) => position as i64,
        Err(_) => errno::EINVAL,
    }
}

fn system_getcwd(buffer: u64, size: usize) -> i64 {
    if !validate_user_pointer(buffer, size, true) {
        return errno::EFAULT;
    }
    
    let cwd = crate::vfs::getcwd();
    let bytes = cwd.as_bytes();
    let len = bytes.len().minimum(size - 1);
    
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
        let destination = core::slice::from_raw_parts_mut(buffer as *mut u8, size);
        destination[..len].copy_from_slice(&bytes[..len]);
        destination[len] = 0;
    }
    
    buffer as i64
}

fn system_chdir(path_pointer: u64) -> i64 {
    let path = // Correspondance de motifs — branchement exhaustif de Rust.
match read_cstring(path_pointer, 256) {
        Some(s) => s,
        None => return errno::EFAULT,
    };
    
        // Correspondance de motifs — branchement exhaustif de Rust.
match crate::vfs::chdir(&path) {
        Ok(()) => 0,
        Err(_) => errno::ENOENT,
    }
}

fn system_fork() -> i64 {
        // Correspondance de motifs — branchement exhaustif de Rust.
match crate::process::fork() {
        Ok(pid) => pid as i64,
        Err(_) => errno::ENOMEM,
    }
}

fn system_execve(pathname: u64, _argv: u64, _envp: u64) -> i64 {
    let path = // Correspondance de motifs — branchement exhaustif de Rust.
match read_cstring(pathname, 256) {
        Some(s) => s,
        None => return errno::EFAULT,
    };
    
    // Parse argv from user memory (null-terminated array of char*)
    let mut argv_strs: alloc::vec::Vec<alloc::string::String> = alloc::vec::Vec::new();
    if _argv != 0 && is_user_address(_argv) {
        for i in 0..256u64 {
            let pointer_address = _argv + i * 8;
            if !is_user_address(pointer_address) { break; }
            let ptr = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { *(pointer_address as *// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const u64) };
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
    
        // Correspondance de motifs — branchement exhaustif de Rust.
match crate::exec::execve(&path, &argv_refs, &[]) {
        Ok(()) => 0, // Never returns on success
        Err(_) => errno::ENOENT,
    }
}

fn system_wait4(pid: i32, wstatus: u64, options: u32) -> i64 {
    let target_pid = if pid > 0 { pid as u32 } else { 0 };
    let wnohang = options & 1 != 0; // WNOHANG
    
    // Blocking wait: retry with yields (up to ~5 s)
    let maximum_tries: u32 = if wnohang { 1 } else { 5000 };
    
    for _ in 0..maximum_tries {
                // Correspondance de motifs — branchement exhaustif de Rust.
match crate::process::wait(target_pid) {
            Ok(status) => {
                if wstatus != 0 && validate_user_pointer(wstatus, 4, true) {
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

fn system_kill(pid: i32, sig: i32) -> i64 {
    if pid <= 0 {
        return errno::EINVAL;
    }
    
    let sender_pid = crate::process::current_pid();
    
        // Correspondance de motifs — branchement exhaustif de Rust.
match crate::signals::kill(pid as u32, sig as u32, sender_pid) {
        Ok(()) => 0,
        Err(e) => e as i64,
    }
}

// ============================================================================
// Network Syscalls (BSD Socket API)
// ============================================================================

/// Create a socket
fn system_socket(domain: u16, socket_type: u32, protocol: u32) -> i64 {
        // Correspondance de motifs — branchement exhaustif de Rust.
match crate::netstack::socket::socket(domain, socket_type, protocol) {
        Ok(fd) => fd as i64,
        Err(e) => e as i64,
    }
}

/// Connect to a remote address
fn system_connect(fd: i32, address_pointer: u64, address_length: usize) -> i64 {
    use crate::netstack::socket::SockAddrIn;
    
    if address_length < SockAddrIn::SIZE {
        return errno::EINVAL;
    }
    
    if !validate_user_pointer(address_pointer, address_length, false) {
        return errno::EFAULT;
    }
    
    let address = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { *(address_pointer as *// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SockAddrIn) };
    
        // Correspondance de motifs — branchement exhaustif de Rust.
match crate::netstack::socket::connect(fd, &address) {
        Ok(()) => 0,
        Err(e) => e as i64,
    }
}

/// Bind socket to local address
fn system_bind(fd: i32, address_pointer: u64, address_length: usize) -> i64 {
    use crate::netstack::socket::SockAddrIn;
    
    if address_length < SockAddrIn::SIZE {
        return errno::EINVAL;
    }
    
    if !validate_user_pointer(address_pointer, address_length, false) {
        return errno::EFAULT;
    }
    
    let address = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { *(address_pointer as *// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SockAddrIn) };
    
        // Correspondance de motifs — branchement exhaustif de Rust.
match crate::netstack::socket::bind(fd, &address) {
        Ok(()) => 0,
        Err(e) => e as i64,
    }
}

/// Listen for connections
fn system_listen(fd: i32, backlog: u32) -> i64 {
        // Correspondance de motifs — branchement exhaustif de Rust.
match crate::netstack::socket::listen(fd, backlog) {
        Ok(()) => 0,
        Err(e) => e as i64,
    }
}

/// Accept a connection
fn system_accept(fd: i32, address_pointer: u64, address_length_pointer: u64) -> i64 {
        // Correspondance de motifs — branchement exhaustif de Rust.
match crate::netstack::socket::accept(fd, address_pointer, address_length_pointer) {
        Ok(new_fd) => new_fd as i64,
        Err(e) => e as i64,
    }
}

/// Send data to connected socket
fn system_sendto(fd: i32, buffer_pointer: u64, len: usize, flags: u32, address_pointer: u64, address_length: usize) -> i64 {
    use crate::netstack::socket::SockAddrIn;
    
    if !validate_user_pointer(buffer_pointer, len, false) {
        return errno::EFAULT;
    }
    
    let data = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { core::slice::from_raw_parts(buffer_pointer as *// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const u8, len) };
    
    if address_pointer != 0 && address_length >= SockAddrIn::SIZE {
        // sendto with address
        if !validate_user_pointer(address_pointer, address_length, false) {
            return errno::EFAULT;
        }
        let address = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { *(address_pointer as *// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SockAddrIn) };
                // Correspondance de motifs — branchement exhaustif de Rust.
match crate::netstack::socket::sendto(fd, data, flags, &address) {
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
fn system_recvfrom(fd: i32, buffer_pointer: u64, len: usize, flags: u32, address_pointer: u64, address_length_pointer: u64) -> i64 {
    if !validate_user_pointer(buffer_pointer, len, true) {
        return errno::EFAULT;
    }
    
    let buffer = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { core::slice::from_raw_parts_mut(buffer_pointer as *mut u8, len) };
    
    // For now, just use recv (ignore address output)
    match crate::netstack::socket::recv(fd, buffer, flags) {
        Ok(n) => n as i64,
        Err(e) => e as i64,
    }
}

/// Shutdown socket
fn system_shutdown(fd: i32, _how: i32) -> i64 {
        // Correspondance de motifs — branchement exhaustif de Rust.
match crate::netstack::socket::close(fd) {
        Ok(()) => 0,
        Err(e) => e as i64,
    }
}

/// Get local address of a socket
fn system_getsockname(fd: i32, address_pointer: u64, address_length_pointer: u64) -> i64 {
    use crate::netstack::socket::{SockAddrIn, SOCKET_TABLE};

    if address_pointer == 0 || address_length_pointer == 0 {
        return errno::EFAULT;
    }
    if !validate_user_pointer(address_length_pointer, 4, true) {
        return errno::EFAULT;
    }

    let len = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { *(address_length_pointer as *// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const u32) } as usize;
    if len < SockAddrIn::SIZE || !validate_user_pointer(address_pointer, SockAddrIn::SIZE, true) {
        return errno::EINVAL;
    }

    let table = SOCKET_TABLE.lock();
    let socket = // Correspondance de motifs — branchement exhaustif de Rust.
match table.get(&fd) {
        Some(s) => s,
        None => return errno::EBADF,
    };

    let address = socket.local_address.unwrap_or_default();
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
        *(address_pointer as *mut SockAddrIn) = address;
        *(address_length_pointer as *mut u32) = SockAddrIn::SIZE as u32;
    }
    0
}

/// Get remote address of a connected socket
fn system_getpeername(fd: i32, address_pointer: u64, address_length_pointer: u64) -> i64 {
    use crate::netstack::socket::{SockAddrIn, SOCKET_TABLE};

    if address_pointer == 0 || address_length_pointer == 0 {
        return errno::EFAULT;
    }
    if !validate_user_pointer(address_length_pointer, 4, true) {
        return errno::EFAULT;
    }

    let len = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { *(address_length_pointer as *// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const u32) } as usize;
    if len < SockAddrIn::SIZE || !validate_user_pointer(address_pointer, SockAddrIn::SIZE, true) {
        return errno::EINVAL;
    }

    let table = SOCKET_TABLE.lock();
    let socket = // Correspondance de motifs — branchement exhaustif de Rust.
match table.get(&fd) {
        Some(s) => s,
        None => return errno::EBADF,
    };

    let address = // Correspondance de motifs — branchement exhaustif de Rust.
match socket.remote_address {
        Some(a) => a,
        None => return -107, // ENOTCONN
    };
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
        *(address_pointer as *mut SockAddrIn) = address;
        *(address_length_pointer as *mut u32) = SockAddrIn::SIZE as u32;
    }
    0
}

/// sendmsg — extract iov[0] and optional addr from msghdr, delegate to sendto
fn system_sendmsg(fd: i32, message_pointer: u64, flags: u32) -> i64 {
    if message_pointer == 0 || !validate_user_pointer(message_pointer, 56, false) {
        return errno::EFAULT;
    }
    // struct msghdr layout (x86_64): name(8), namelen(4), pad(4), iov(8), iovlen(8), control(8), controllen(8), flags(4)
    let name_pointer = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { *(message_pointer as *// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const u64) };
    let name_length = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { *((message_pointer + 8) as *// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const u32) } as usize;
    let iov_pointer  = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { *((message_pointer + 16) as *// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const u64) };
    let iov_length  = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { *((message_pointer + 24) as *// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const u64) } as usize;

    if iov_length == 0 || iov_pointer == 0 {
        return 0;
    }
    // Read first iovec {base: *u8, len: usize}
    if !validate_user_pointer(iov_pointer, 16, false) {
        return errno::EFAULT;
    }
    let base = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { *(iov_pointer as *// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const u64) };
    let len  = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { *((iov_pointer + 8) as *// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const u64) } as usize;

    system_sendto(fd, base, len, flags, name_pointer, name_length)
}

/// recvmsg — extract iov[0] from msghdr, delegate to recvfrom
fn system_recvmsg(fd: i32, message_pointer: u64, flags: u32) -> i64 {
    if message_pointer == 0 || !validate_user_pointer(message_pointer, 56, true) {
        return errno::EFAULT;
    }
    let name_pointer = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { *(message_pointer as *// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const u64) };
    let name_length_pointer = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { (message_pointer + 8) as u64 };
    let iov_pointer  = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { *((message_pointer + 16) as *// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const u64) };
    let iov_length  = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { *((message_pointer + 24) as *// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const u64) } as usize;

    if iov_length == 0 || iov_pointer == 0 {
        return 0;
    }
    if !validate_user_pointer(iov_pointer, 16, false) {
        return errno::EFAULT;
    }
    let base = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { *(iov_pointer as *// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const u64) };
    let len  = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { *((iov_pointer + 8) as *// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const u64) } as usize;

    system_recvfrom(fd, base, len, flags, name_pointer, name_length_pointer)
}

/// Set socket option
fn system_setsockopt(fd: i32, level: i32, optname: i32, optval: u64, optlen: usize) -> i64 {
    if optlen > 0 && !validate_user_pointer(optval, optlen, false) {
        return errno::EFAULT;
    }
    
    let data = if optlen > 0 {
                // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { core::slice::from_raw_parts(optval as *// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const u8, optlen) }
    } else {
        &[]
    };
    
        // Correspondance de motifs — branchement exhaustif de Rust.
match crate::netstack::socket::setsockopt(fd, level, optname, data) {
        Ok(()) => 0,
        Err(e) => e as i64,
    }
}

/// Get socket option
fn system_getsockopt(fd: i32, level: i32, optname: i32, optval: u64, optlen_pointer: u64) -> i64 {
    if optval == 0 || optlen_pointer == 0 {
        return errno::EFAULT;
    }
    
    if !validate_user_pointer(optlen_pointer, 4, true) {
        return errno::EFAULT;
    }
    
    let optlen = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { *(optlen_pointer as *// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const u32) } as usize;
    
    if optlen > 0 && !validate_user_pointer(optval, optlen, true) {
        return errno::EFAULT;
    }
    
    let buffer = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { core::slice::from_raw_parts_mut(optval as *mut u8, optlen) };
    
        // Correspondance de motifs — branchement exhaustif de Rust.
match crate::netstack::socket::getsockopt(fd, level, optname, buffer) {
        Ok(len) => {
                        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { *(optlen_pointer as *mut u32) = len as u32; }
            0
        }
        Err(e) => e as i64,
    }
}
