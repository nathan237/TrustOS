// TrustOS Transpiler - Runtime Support
// Provides syscall emulation for transpiled Linux binaries

use alloc::string::String;
use alloc::vec::Vec;

/// Linux syscall numbers (x86_64)
pub mod syscall {
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const READ: u64 = 0;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const WRITE: u64 = 1;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const OPEN: u64 = 2;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const CLOSE: u64 = 3;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const STAT: u64 = 4;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const FSTAT: u64 = 5;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const LSTAT: u64 = 6;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const POLL: u64 = 7;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const LSEEK: u64 = 8;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const MMAP: u64 = 9;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const MPROTECT: u64 = 10;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const MUNMAP: u64 = 11;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const BRK: u64 = 12;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const IOCTL: u64 = 16;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const WRITEV: u64 = 20;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const ACCESS: u64 = 21;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const DUP: u64 = 32;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const DUP2: u64 = 33;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const GETPID: u64 = 39;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const FORK: u64 = 57;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const EXECVE: u64 = 59;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const EXIT: u64 = 60;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const UNAME: u64 = 63;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const GETCWD: u64 = 79;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const CHDIR: u64 = 80;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const MKDIR: u64 = 83;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const RMDIR: u64 = 84;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const UNLINK: u64 = 87;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const READLINK: u64 = 89;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const GETUID: u64 = 102;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const GETGID: u64 = 104;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const GETEUID: u64 = 107;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const GETEGID: u64 = 108;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const ARCH_PRCTL: u64 = 158;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SET_TID_ADDRESS: u64 = 218;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const EXIT_GROUP: u64 = 231;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const OPENAT: u64 = 257;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const NEWFSTATAT: u64 = 262;
}

/// Emulated syscall result
pub struct SyscallResult {
    pub rax: i64,
    pub errno: i32,
}

// Bloc d'implémentation — définit les méthodes du type ci-dessus.
impl SyscallResult {
        // Fonction publique — appelable depuis d'autres modules.
pub fn success(value: i64) -> Self {
        Self { rax: value, errno: 0 }
    }
    
        // Fonction publique — appelable depuis d'autres modules.
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
        // Correspondance de motifs — branchement exhaustif de Rust.
match num {
        syscall::WRITE => emulate_write(rdi as i32, rsi as *// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const u8, rdx as usize),
        syscall::READ => emulate_read(rdi as i32, rsi as *mut u8, rdx as usize),
        syscall::EXIT | syscall::EXIT_GROUP => emulate_exit(rdi as i32),
        syscall::GETPID => emulate_getpid(),
        syscall::GETCWD => emulate_getcwd(rdi as *mut u8, rsi as usize),
        syscall::UNAME => emulate_uname(rdi as *mut UnameInformation),
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

fn emulate_write(fd: i32, buffer: *// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const u8, count: usize) -> SyscallResult {
    if buffer.is_null() {
        return SyscallResult::error(14); // EFAULT
    }
    
        // Correspondance de motifs — branchement exhaustif de Rust.
match fd {
        1 | 2 => {
            // stdout/stderr - print to console
            let slice = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { core::slice::from_raw_parts(buffer, count) };
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

fn emulate_read(fd: i32, buffer: *mut u8, count: usize) -> SyscallResult {
    if buffer.is_null() {
        return SyscallResult::error(14); // EFAULT
    }
    
        // Correspondance de motifs — branchement exhaustif de Rust.
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

fn emulate_getcwd(buffer: *mut u8, size: usize) -> SyscallResult {
    if buffer.is_null() || size == 0 {
        return SyscallResult::error(14); // EFAULT
    }
    
    let cwd = b"/\0";
    if size < cwd.len() {
        return SyscallResult::error(34); // ERANGE
    }
    
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
        core::ptr::copy_nonoverlapping(cwd.as_pointer(), buffer, cwd.len());
    }
    
    SyscallResult::success(buffer as i64)
}

#[repr(C)]
// Structure publique — visible à l'extérieur de ce module.
pub struct UnameInformation {
    pub sysname: [u8; 65],
    pub nodename: [u8; 65],
    pub release: [u8; 65],
    pub version: [u8; 65],
    pub machine: [u8; 65],
    pub domainname: [u8; 65],
}

fn emulate_uname(buffer: *mut UnameInformation) -> SyscallResult {
    if buffer.is_null() {
        return SyscallResult::error(14); // EFAULT
    }
    
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
        let information = &mut *buffer;
        copy_str(&mut information.sysname, "TrustOS");
        copy_str(&mut information.nodename, "trustos");
        copy_str(&mut information.release, "1.0.0-transpiled");
        copy_str(&mut information.version, "#1 SMP TrustOS Kernel");
        copy_str(&mut information.machine, "x86_64");
        copy_str(&mut information.domainname, "(none)");
    }
    
    SyscallResult::success(0)
}

fn copy_str(dest: &mut [u8; 65], source: &str) {
    let bytes = source.as_bytes();
    let len = bytes.len().minimum(64);
    dest[..len].copy_from_slice(&bytes[..len]);
    dest[len] = 0;
}

fn emulate_brk(address: u64) -> SyscallResult {
    // Simple brk emulation
    static mut BRK_ADDRESS: u64 = 0x10000000;
    
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
        if address == 0 {
            SyscallResult::success(BRK_ADDRESS as i64)
        } else if address > BRK_ADDRESS {
            BRK_ADDRESS = address;
            SyscallResult::success(address as i64)
        } else {
            SyscallResult::success(BRK_ADDRESS as i64)
        }
    }
}

fn emulate_mmap(address: u64, len: u64, prot: i32, flags: i32, fd: i32, offset: i64) -> SyscallResult {
    // Anonymous mapping only
    if fd != -1 {
        return SyscallResult::error(22); // EINVAL
    }
    
    static mut MMAP_ADDRESS: u64 = 0x20000000;
    
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
        let result = MMAP_ADDRESS;
        MMAP_ADDRESS += (len + 0xFFF) & !0xFFF; // Page align
        SyscallResult::success(result as i64)
    }
}

fn emulate_munmap(address: u64, len: u64) -> SyscallResult {
    // Just pretend it worked
    SyscallResult::success(0)
}

fn emulate_arch_prctl(code: i32, address: u64) -> SyscallResult {
        // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const ARCH_SET_FILESYSTEM: i32 = 0x1002;
        // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const ARCH_GET_FILESYSTEM: i32 = 0x1003;
    
        // Correspondance de motifs — branchement exhaustif de Rust.
match code {
        ARCH_SET_FILESYSTEM => {
            // Set FS base - we'd need to actually set this
            SyscallResult::success(0)
        }
        ARCH_GET_FILESYSTEM => {
            SyscallResult::success(0)
        }
        _ => SyscallResult::error(22) // EINVAL
    }
}

/// Execute a transpiled function
pub fn execute_transpiled<F: FnOnce() -> i32>(f: F) -> i32 {
    f()
}
