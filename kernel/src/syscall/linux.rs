//! Linux-compatible System Calls
//!
//! Implements the most essential Linux syscalls needed to run statically-linked
//! Linux binaries. This is a compatibility layer, not a full Linux implementation.

use crate::memory::{validate_user_ptr, is_user_address};
use crate::syscall::errno;
use alloc::string::String;
use alloc::vec::Vec;

/// Linux syscall numbers (x86_64 ABI)
pub mod nr {
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
    pub const RT_SIGACTION: u64 = 13;
    pub const RT_SIGPROCMASK: u64 = 14;
    pub const RT_SIGRETURN: u64 = 15;
    pub const IOCTL: u64 = 16;
    pub const PREAD64: u64 = 17;
    pub const PWRITE64: u64 = 18;
    pub const READV: u64 = 19;
    pub const WRITEV: u64 = 20;
    pub const ACCESS: u64 = 21;
    pub const PIPE: u64 = 22;
    pub const SELECT: u64 = 23;
    pub const SCHED_YIELD: u64 = 24;
    pub const MREMAP: u64 = 25;
    pub const MSYNC: u64 = 26;
    pub const MINCORE: u64 = 27;
    pub const MADVISE: u64 = 28;
    pub const DUP: u64 = 32;
    pub const DUP2: u64 = 33;
    pub const PAUSE: u64 = 34;
    pub const NANOSLEEP: u64 = 35;
    pub const GETITIMER: u64 = 36;
    pub const ALARM: u64 = 37;
    pub const SETITIMER: u64 = 38;
    pub const GETPID: u64 = 39;
    pub const SENDFILE: u64 = 40;
    pub const SOCKET: u64 = 41;
    pub const CONNECT: u64 = 42;
    pub const ACCEPT: u64 = 43;
    pub const SENDTO: u64 = 44;
    pub const RECVFROM: u64 = 45;
    pub const SENDMSG: u64 = 46;
    pub const RECVMSG: u64 = 47;
    pub const SHUTDOWN: u64 = 48;
    pub const BIND: u64 = 49;
    pub const LISTEN: u64 = 50;
    pub const GETSOCKNAME: u64 = 51;
    pub const GETPEERNAME: u64 = 52;
    pub const SOCKETPAIR: u64 = 53;
    pub const SETSOCKOPT: u64 = 54;
    pub const GETSOCKOPT: u64 = 55;
    pub const CLONE: u64 = 56;
    pub const FORK: u64 = 57;
    pub const VFORK: u64 = 58;
    pub const EXECVE: u64 = 59;
    pub const EXIT: u64 = 60;
    pub const WAIT4: u64 = 61;
    pub const KILL: u64 = 62;
    pub const UNAME: u64 = 63;
    pub const FCNTL: u64 = 72;
    pub const FLOCK: u64 = 73;
    pub const FSYNC: u64 = 74;
    pub const FDATASYNC: u64 = 75;
    pub const TRUNCATE: u64 = 76;
    pub const FTRUNCATE: u64 = 77;
    pub const GETDENTS: u64 = 78;
    pub const GETCWD: u64 = 79;
    pub const CHDIR: u64 = 80;
    pub const FCHDIR: u64 = 81;
    pub const RENAME: u64 = 82;
    pub const MKDIR: u64 = 83;
    pub const RMDIR: u64 = 84;
    pub const CREAT: u64 = 85;
    pub const LINK: u64 = 86;
    pub const UNLINK: u64 = 87;
    pub const SYMLINK: u64 = 88;
    pub const READLINK: u64 = 89;
    pub const CHMOD: u64 = 90;
    pub const FCHMOD: u64 = 91;
    pub const CHOWN: u64 = 92;
    pub const FCHOWN: u64 = 93;
    pub const LCHOWN: u64 = 94;
    pub const UMASK: u64 = 95;
    pub const GETTIMEOFDAY: u64 = 96;
    pub const GETRLIMIT: u64 = 97;
    pub const GETRUSAGE: u64 = 98;
    pub const SYSINFO: u64 = 99;
    pub const TIMES: u64 = 100;
    pub const GETUID: u64 = 102;
    pub const SYSLOG: u64 = 103;
    pub const GETGID: u64 = 104;
    pub const SETUID: u64 = 105;
    pub const SETGID: u64 = 106;
    pub const GETEUID: u64 = 107;
    pub const GETEGID: u64 = 108;
    pub const SETPGID: u64 = 109;
    pub const GETPPID: u64 = 110;
    pub const GETPGRP: u64 = 111;
    pub const SETSID: u64 = 112;
    pub const SETREUID: u64 = 113;
    pub const SETREGID: u64 = 114;
    pub const GETGROUPS: u64 = 115;
    pub const SETGROUPS: u64 = 116;
    pub const SETRESUID: u64 = 117;
    pub const GETRESUID: u64 = 118;
    pub const SETRESGID: u64 = 119;
    pub const GETRESGID: u64 = 120;
    pub const GETPGID: u64 = 121;
    pub const SETFSUID: u64 = 122;
    pub const SETFSGID: u64 = 123;
    pub const GETSID: u64 = 124;
    pub const CAPGET: u64 = 125;
    pub const CAPSET: u64 = 126;
    pub const RT_SIGPENDING: u64 = 127;
    pub const RT_SIGTIMEDWAIT: u64 = 128;
    pub const RT_SIGQUEUEINFO: u64 = 129;
    pub const RT_SIGSUSPEND: u64 = 130;
    pub const SIGALTSTACK: u64 = 131;
    pub const UTIME: u64 = 132;
    pub const MKNOD: u64 = 133;
    pub const USELIB: u64 = 134;
    pub const PERSONALITY: u64 = 135;
    pub const USTAT: u64 = 136;
    pub const STATFS: u64 = 137;
    pub const FSTATFS: u64 = 138;
    pub const SYSFS: u64 = 139;
    pub const GETPRIORITY: u64 = 140;
    pub const SETPRIORITY: u64 = 141;
    pub const SCHED_SETPARAM: u64 = 142;
    pub const SCHED_GETPARAM: u64 = 143;
    pub const SCHED_SETSCHEDULER: u64 = 144;
    pub const SCHED_GETSCHEDULER: u64 = 145;
    pub const SCHED_GET_PRIORITY_MAX: u64 = 146;
    pub const SCHED_GET_PRIORITY_MIN: u64 = 147;
    pub const SCHED_RR_GET_INTERVAL: u64 = 148;
    pub const MLOCK: u64 = 149;
    pub const MUNLOCK: u64 = 150;
    pub const MLOCKALL: u64 = 151;
    pub const MUNLOCKALL: u64 = 152;
    pub const VHANGUP: u64 = 153;
    pub const MODIFY_LDT: u64 = 154;
    pub const PIVOT_ROOT: u64 = 155;
    pub const PRCTL: u64 = 157;
    pub const ARCH_PRCTL: u64 = 158;
    pub const ADJTIMEX: u64 = 159;
    pub const SETRLIMIT: u64 = 160;
    pub const CHROOT: u64 = 161;
    pub const SYNC: u64 = 162;
    pub const ACCT: u64 = 163;
    pub const SETTIMEOFDAY: u64 = 164;
    pub const MOUNT: u64 = 165;
    pub const UMOUNT2: u64 = 166;
    pub const SWAPON: u64 = 167;
    pub const SWAPOFF: u64 = 168;
    pub const REBOOT: u64 = 169;
    pub const SETHOSTNAME: u64 = 170;
    pub const SETDOMAINNAME: u64 = 171;
    pub const IOPL: u64 = 172;
    pub const IOPERM: u64 = 173;
    pub const GETTID: u64 = 186;
    pub const READAHEAD: u64 = 187;
    pub const SETXATTR: u64 = 188;
    pub const GETXATTR: u64 = 191;
    pub const LISTXATTR: u64 = 194;
    pub const REMOVEXATTR: u64 = 197;
    pub const TKILL: u64 = 200;
    pub const TIME: u64 = 201;
    pub const FUTEX: u64 = 202;
    pub const SCHED_SETAFFINITY: u64 = 203;
    pub const SCHED_GETAFFINITY: u64 = 204;
    pub const SET_THREAD_AREA: u64 = 205;
    pub const IO_SETUP: u64 = 206;
    pub const IO_DESTROY: u64 = 207;
    pub const IO_GETEVENTS: u64 = 208;
    pub const IO_SUBMIT: u64 = 209;
    pub const IO_CANCEL: u64 = 210;
    pub const GET_THREAD_AREA: u64 = 211;
    pub const LOOKUP_DCOOKIE: u64 = 212;
    pub const EPOLL_CREATE: u64 = 213;
    pub const REMAP_FILE_PAGES: u64 = 216;
    pub const GETDENTS64: u64 = 217;
    pub const SET_TID_ADDRESS: u64 = 218;
    pub const RESTART_SYSCALL: u64 = 219;
    pub const SEMTIMEDOP: u64 = 220;
    pub const FADVISE64: u64 = 221;
    pub const TIMER_CREATE: u64 = 222;
    pub const TIMER_SETTIME: u64 = 223;
    pub const TIMER_GETTIME: u64 = 224;
    pub const TIMER_GETOVERRUN: u64 = 225;
    pub const TIMER_DELETE: u64 = 226;
    pub const CLOCK_SETTIME: u64 = 227;
    pub const CLOCK_GETTIME: u64 = 228;
    pub const CLOCK_GETRES: u64 = 229;
    pub const CLOCK_NANOSLEEP: u64 = 230;
    pub const EXIT_GROUP: u64 = 231;
    pub const EPOLL_WAIT: u64 = 232;
    pub const EPOLL_CTL: u64 = 233;
    pub const TGKILL: u64 = 234;
    pub const UTIMES: u64 = 235;
    pub const MBIND: u64 = 237;
    pub const SET_MEMPOLICY: u64 = 238;
    pub const GET_MEMPOLICY: u64 = 239;
    pub const MQ_OPEN: u64 = 240;
    pub const MQ_UNLINK: u64 = 241;
    pub const MQ_TIMEDSEND: u64 = 242;
    pub const MQ_TIMEDRECEIVE: u64 = 243;
    pub const MQ_NOTIFY: u64 = 244;
    pub const MQ_GETSETATTR: u64 = 245;
    pub const KEXEC_LOAD: u64 = 246;
    pub const WAITID: u64 = 247;
    pub const ADD_KEY: u64 = 248;
    pub const REQUEST_KEY: u64 = 249;
    pub const KEYCTL: u64 = 250;
    pub const IOPRIO_SET: u64 = 251;
    pub const IOPRIO_GET: u64 = 252;
    pub const INOTIFY_INIT: u64 = 253;
    pub const INOTIFY_ADD_WATCH: u64 = 254;
    pub const INOTIFY_RM_WATCH: u64 = 255;
    pub const MIGRATE_PAGES: u64 = 256;
    pub const OPENAT: u64 = 257;
    pub const MKDIRAT: u64 = 258;
    pub const MKNODAT: u64 = 259;
    pub const FCHOWNAT: u64 = 260;
    pub const FUTIMESAT: u64 = 261;
    pub const NEWFSTATAT: u64 = 262;
    pub const UNLINKAT: u64 = 263;
    pub const RENAMEAT: u64 = 264;
    pub const LINKAT: u64 = 265;
    pub const SYMLINKAT: u64 = 266;
    pub const READLINKAT: u64 = 267;
    pub const FCHMODAT: u64 = 268;
    pub const FACCESSAT: u64 = 269;
    pub const PSELECT6: u64 = 270;
    pub const PPOLL: u64 = 271;
    pub const UNSHARE: u64 = 272;
    pub const SET_ROBUST_LIST: u64 = 273;
    pub const GET_ROBUST_LIST: u64 = 274;
    pub const SPLICE: u64 = 275;
    pub const TEE: u64 = 276;
    pub const SYNC_FILE_RANGE: u64 = 277;
    pub const VMSPLICE: u64 = 278;
    pub const MOVE_PAGES: u64 = 279;
    pub const UTIMENSAT: u64 = 280;
    pub const EPOLL_PWAIT: u64 = 281;
    pub const SIGNALFD: u64 = 282;
    pub const TIMERFD_CREATE: u64 = 283;
    pub const EVENTFD: u64 = 284;
    pub const FALLOCATE: u64 = 285;
    pub const TIMERFD_SETTIME: u64 = 286;
    pub const TIMERFD_GETTIME: u64 = 287;
    pub const ACCEPT4: u64 = 288;
    pub const SIGNALFD4: u64 = 289;
    pub const EVENTFD2: u64 = 290;
    pub const EPOLL_CREATE1: u64 = 291;
    pub const DUP3: u64 = 292;
    pub const PIPE2: u64 = 293;
    pub const INOTIFY_INIT1: u64 = 294;
    pub const PREADV: u64 = 295;
    pub const PWRITEV: u64 = 296;
    pub const RT_TGSIGQUEUEINFO: u64 = 297;
    pub const PERF_EVENT_OPEN: u64 = 298;
    pub const RECVMMSG: u64 = 299;
    pub const FANOTIFY_INIT: u64 = 300;
    pub const FANOTIFY_MARK: u64 = 301;
    pub const PRLIMIT64: u64 = 302;
    pub const NAME_TO_HANDLE_AT: u64 = 303;
    pub const OPEN_BY_HANDLE_AT: u64 = 304;
    pub const CLOCK_ADJTIME: u64 = 305;
    pub const SYNCFS: u64 = 306;
    pub const SENDMMSG: u64 = 307;
    pub const SETNS: u64 = 308;
    pub const GETCPU: u64 = 309;
    pub const PROCESS_VM_READV: u64 = 310;
    pub const PROCESS_VM_WRITEV: u64 = 311;
    pub const KCMP: u64 = 312;
    pub const FINIT_MODULE: u64 = 313;
    pub const SCHED_SETATTR: u64 = 314;
    pub const SCHED_GETATTR: u64 = 315;
    pub const RENAMEAT2: u64 = 316;
    pub const SECCOMP: u64 = 317;
    pub const GETRANDOM: u64 = 318;
    pub const MEMFD_CREATE: u64 = 319;
    pub const KEXEC_FILE_LOAD: u64 = 320;
    pub const BPF: u64 = 321;
    pub const EXECVEAT: u64 = 322;
    pub const USERFAULTFD: u64 = 323;
    pub const MEMBARRIER: u64 = 324;
    pub const MLOCK2: u64 = 325;
    pub const COPY_FILE_RANGE: u64 = 326;
    pub const PREADV2: u64 = 327;
    pub const PWRITEV2: u64 = 328;
    pub const PKEY_MPROTECT: u64 = 329;
    pub const PKEY_ALLOC: u64 = 330;
    pub const PKEY_FREE: u64 = 331;
    pub const STATX: u64 = 332;
}

// ============================================================================
// Memory Management
// ============================================================================

/// mmap flags
pub mod mmap_flags {
    pub const MAP_SHARED: u64 = 0x01;
    pub const MAP_PRIVATE: u64 = 0x02;
    pub const MAP_FIXED: u64 = 0x10;
    pub const MAP_ANONYMOUS: u64 = 0x20;
    pub const MAP_GROWSDOWN: u64 = 0x100;
    pub const MAP_DENYWRITE: u64 = 0x800;
    pub const MAP_EXECUTABLE: u64 = 0x1000;
    pub const MAP_LOCKED: u64 = 0x2000;
    pub const MAP_NORESERVE: u64 = 0x4000;
    pub const MAP_POPULATE: u64 = 0x8000;
    pub const MAP_NONBLOCK: u64 = 0x10000;
    pub const MAP_STACK: u64 = 0x20000;
    pub const MAP_HUGETLB: u64 = 0x40000;
}

/// mmap protection flags
pub mod prot_flags {
    pub const PROT_NONE: u64 = 0x0;
    pub const PROT_READ: u64 = 0x1;
    pub const PROT_WRITE: u64 = 0x2;
    pub const PROT_EXEC: u64 = 0x4;
}

use spin::Mutex;
use core::sync::atomic::{AtomicU64, Ordering};

/// Current program break (heap end) — legacy, kept for fallback only
static PROGRAM_BREAK: AtomicU64 = AtomicU64::new(0);

/// Next available mmap address (user region)
static NEXT_MMAP_ADDR: AtomicU64 = AtomicU64::new(0x4000_0000); // 1 GB, well inside user space

/// sys_mmap - Map memory
/// 
/// Allocates physical frames via the frame allocator and maps them
/// into the current user address space at proper user-space addresses.
pub fn sys_mmap(addr: u64, length: u64, prot: u64, flags: u64, fd: i64, _offset: u64) -> i64 {
    use mmap_flags::*;
    use prot_flags::*;
    use crate::memory::paging::PageFlags;
    
    if length == 0 {
        return errno::EINVAL;
    }
    
    let page_size = 4096u64;
    let aligned_length = (length + page_size - 1) & !(page_size - 1);
    let num_pages = (aligned_length / page_size) as usize;
    
    // Determine the mapping address
    let map_addr = if addr != 0 && (flags & MAP_FIXED) != 0 {
        addr & !(page_size - 1) // page-align
    } else {
        // Kernel chooses address
        NEXT_MMAP_ADDR.fetch_add(aligned_length, Ordering::SeqCst)
    };
    
    // Only anonymous mappings for now
    let is_anonymous = (flags & MAP_ANONYMOUS) != 0 || fd < 0;
    if !is_anonymous {
        crate::log_debug!("[MMAP] File-backed mmap not yet implemented");
        return errno::ENOSYS;
    }
    
    // Determine page flags from protection bits
    let page_flags = {
        let mut f = PageFlags::USER_DATA; // default: present + user + writable + NX
        if (prot & PROT_EXEC) != 0 {
            f = PageFlags::USER_CODE; // present + user (no NX)
        }
        f
    };
    
    // Allocate frames and map them
    let mapped = crate::exec::with_current_address_space(|space| {
        for i in 0..num_pages {
            let virt = map_addr + (i as u64 * page_size);
            let phys = match crate::memory::frame::alloc_frame_zeroed() {
                Some(p) => p,
                None => return false,
            };
            if space.map_page(virt, phys, page_flags).is_none() {
                return false;
            }
        }
        true
    });
    
    match mapped {
        Some(true) => {
            crate::log_debug!("[MMAP] Mapped {} pages at {:#x}", num_pages, map_addr);
            map_addr as i64
        }
        _ => {
            // Fallback for calls without an active user address space
            // (e.g. kernel-internal mmap during init — return heap allocation)
            let layout = core::alloc::Layout::from_size_align(aligned_length as usize, page_size as usize)
                .unwrap_or(core::alloc::Layout::new::<u8>());
            let ptr = unsafe { alloc::alloc::alloc_zeroed(layout) };
            if ptr.is_null() {
                return errno::ENOMEM;
            }
            crate::log_debug!("[MMAP] Fallback heap alloc at {:#x}", ptr as u64);
            ptr as i64
        }
    }
}

/// sys_munmap - Unmap memory and free frames
pub fn sys_munmap(addr: u64, length: u64) -> i64 {
    if addr == 0 || length == 0 {
        return errno::EINVAL;
    }
    
    let page_size = 4096u64;
    let aligned_length = (length + page_size - 1) & !(page_size - 1);
    let num_pages = (aligned_length / page_size) as usize;
    
    crate::exec::with_current_address_space(|space| {
        for i in 0..num_pages {
            let virt = (addr & !(page_size - 1)) + (i as u64 * page_size);
            // Translate to get physical address before unmapping
            if let Some(phys) = space.translate(virt) {
                let phys_page = phys & !0xFFF;
                space.unmap_page(virt);
                crate::memory::frame::free_frame(phys_page);
            }
        }
    });
    
    crate::log_debug!("[MUNMAP] Unmapped {} pages at {:#x}", num_pages, addr);
    0
}

/// sys_mprotect - Change memory protection (walks page tables)
pub fn sys_mprotect(addr: u64, length: u64, prot: u64) -> i64 {
    use prot_flags::*;
    use crate::memory::paging::{PageFlags, PageTable};
    
    if addr == 0 || addr & 0xFFF != 0 {
        return errno::EINVAL;
    }
    
    let page_size = 4096u64;
    let aligned_length = (length + page_size - 1) & !(page_size - 1);
    let num_pages = (aligned_length / page_size) as usize;
    
    // Build new permission flags
    let mut pf = PageFlags::PRESENT | PageFlags::USER;
    if (prot & PROT_WRITE) != 0 {
        pf |= PageFlags::WRITABLE;
    }
    if (prot & PROT_EXEC) == 0 {
        pf |= PageFlags::NO_EXECUTE;
    }
    let new_flags = PageFlags::new(pf);
    
    crate::exec::with_current_address_space(|space| {
        let hhdm = crate::memory::hhdm_offset();
        let cr3 = space.cr3();
        
        for i in 0..num_pages {
            let virt = addr + (i as u64 * page_size);
            let pml4_idx = ((virt >> 39) & 0x1FF) as usize;
            let pdpt_idx = ((virt >> 30) & 0x1FF) as usize;
            let pd_idx   = ((virt >> 21) & 0x1FF) as usize;
            let pt_idx   = ((virt >> 12) & 0x1FF) as usize;
            
            let pml4 = unsafe { &*((cr3 + hhdm) as *const PageTable) };
            if !pml4.entries[pml4_idx].is_present() { continue; }
            let pdpt = unsafe { &*((pml4.entries[pml4_idx].phys_addr() + hhdm) as *const PageTable) };
            if !pdpt.entries[pdpt_idx].is_present() { continue; }
            let pd = unsafe { &*((pdpt.entries[pdpt_idx].phys_addr() + hhdm) as *const PageTable) };
            if !pd.entries[pd_idx].is_present() { continue; }
            let pt = unsafe { &mut *((pd.entries[pd_idx].phys_addr() + hhdm) as *mut PageTable) };
            if !pt.entries[pt_idx].is_present() { continue; }
            
            let phys = pt.entries[pt_idx].phys_addr();
            pt.entries[pt_idx].set(phys, new_flags);
            unsafe { core::arch::asm!("invlpg [{}]", in(reg) virt, options(nostack, preserves_flags)); }
        }
    });
    
    crate::log_debug!("[MPROTECT] addr={:#x} len={:#x} prot={:#x}", addr, length, prot);
    0
}

/// sys_brk - Change program break (heap end)
///
/// Eagerly allocates and maps physical frames when the break is extended.
pub fn sys_brk(addr: u64) -> i64 {
    use crate::memory::paging::{PageFlags, UserMemoryRegion};
    
    let current_brk = crate::exec::current_brk();
    
    if addr == 0 || current_brk == 0 {
        // Query / initialize — return current break
        if current_brk == 0 {
            return UserMemoryRegion::HEAP_START as i64;
        }
        return current_brk as i64;
    }
    
    // Validate range
    if addr < UserMemoryRegion::HEAP_START || addr >= UserMemoryRegion::HEAP_END {
        return current_brk as i64;
    }
    
    let page_size = 4096u64;
    
    if addr > current_brk {
        // Extending the heap — allocate and map new pages
        let old_page = (current_brk + page_size - 1) & !(page_size - 1); // first unmapped page
        let new_page = (addr + page_size - 1) & !(page_size - 1);        // last page to map (exclusive)
        
        if new_page > old_page {
            let pages_needed = ((new_page - old_page) / page_size) as usize;
            
            let ok = crate::exec::with_current_address_space(|space| {
                for i in 0..pages_needed {
                    let virt = old_page + (i as u64 * page_size);
                    let phys = match crate::memory::frame::alloc_frame_zeroed() {
                        Some(p) => p,
                        None => return false,
                    };
                    if space.map_page(virt, phys, PageFlags::USER_DATA).is_none() {
                        return false;
                    }
                }
                true
            });
            
            if ok != Some(true) {
                return current_brk as i64; // OOM — don't move break
            }
        }
    }
    // Note: shrinking the heap (addr < current_brk) — we just move the break
    // without freeing pages for now (matches many real OS behaviours).
    
    crate::exec::set_current_brk(addr);
    crate::log_debug!("[BRK] Set program break to {:#x}", addr);
    addr as i64
}

// ============================================================================
// Process/Thread Identity
// ============================================================================

/// sys_getpid - Get process ID
pub fn sys_getpid() -> i64 {
    crate::process::current_pid() as i64
}

/// sys_getppid - Get parent process ID
pub fn sys_getppid() -> i64 {
    crate::process::current()
        .map(|p| p.ppid as i64)
        .unwrap_or(0)
}

/// sys_gettid - Get thread ID
pub fn sys_gettid() -> i64 {
    crate::thread::current_tid() as i64
}

/// sys_getuid - Get user ID
pub fn sys_getuid() -> i64 {
    0 // Always root for now
}

/// sys_getgid - Get group ID
pub fn sys_getgid() -> i64 {
    0 // Always root for now
}

/// sys_geteuid - Get effective user ID  
pub fn sys_geteuid() -> i64 {
    0
}

/// sys_getegid - Get effective group ID
pub fn sys_getegid() -> i64 {
    0
}

// ============================================================================
// arch_prctl - Architecture-specific thread state
// ============================================================================

/// arch_prctl codes
pub mod arch_prctl_codes {
    pub const ARCH_SET_GS: u64 = 0x1001;
    pub const ARCH_SET_FS: u64 = 0x1002;
    pub const ARCH_GET_FS: u64 = 0x1003;
    pub const ARCH_GET_GS: u64 = 0x1004;
}

/// Thread-local storage base
static TLS_BASE: AtomicU64 = AtomicU64::new(0);

/// sys_arch_prctl - Set architecture-specific thread state
pub fn sys_arch_prctl(code: u64, addr: u64) -> i64 {
    use arch_prctl_codes::*;
    
    match code {
        ARCH_SET_FS => {
            // Set FS base register (used for TLS)
            TLS_BASE.store(addr, Ordering::SeqCst);
            
            // Actually set the FS base using MSR
            unsafe {
                // MSR_FS_BASE = 0xC0000100
                core::arch::asm!(
                    "wrmsr",
                    in("ecx") 0xC0000100u32,
                    in("eax") (addr as u32),
                    in("edx") ((addr >> 32) as u32),
                );
            }
            crate::log_debug!("[ARCH_PRCTL] Set FS base to {:#x}", addr);
            0
        }
        ARCH_SET_GS => {
            // Set GS base register
            unsafe {
                // MSR_GS_BASE = 0xC0000101
                core::arch::asm!(
                    "wrmsr",
                    in("ecx") 0xC0000101u32,
                    in("eax") (addr as u32),
                    in("edx") ((addr >> 32) as u32),
                );
            }
            0
        }
        ARCH_GET_FS => {
            if !is_user_address(addr) {
                return errno::EFAULT;
            }
            let val = TLS_BASE.load(Ordering::SeqCst);
            unsafe { *(addr as *mut u64) = val; }
            0
        }
        ARCH_GET_GS => {
            if !is_user_address(addr) {
                return errno::EFAULT;
            }
            let val: u64;
            unsafe {
                core::arch::asm!(
                    "rdmsr",
                    in("ecx") 0xC0000101u32,
                    out("eax") _,
                    out("edx") _,
                );
                // Simplified - in reality we'd combine eax/edx
                val = 0;
            }
            unsafe { *(addr as *mut u64) = val; }
            0
        }
        _ => errno::EINVAL,
    }
}

// ============================================================================
// set_tid_address - Set clear_child_tid pointer
// ============================================================================

static CLEAR_CHILD_TID: AtomicU64 = AtomicU64::new(0);

/// sys_set_tid_address - Set pointer to thread ID
pub fn sys_set_tid_address(tidptr: u64) -> i64 {
    CLEAR_CHILD_TID.store(tidptr, Ordering::SeqCst);
    sys_gettid()
}

// ============================================================================
// uname - System information
// ============================================================================

/// utsname structure
#[repr(C)]
pub struct Utsname {
    pub sysname: [u8; 65],
    pub nodename: [u8; 65],
    pub release: [u8; 65],
    pub version: [u8; 65],
    pub machine: [u8; 65],
    pub domainname: [u8; 65],
}

/// sys_uname - Get system information
pub fn sys_uname(buf: u64) -> i64 {
    if !validate_user_ptr(buf, core::mem::size_of::<Utsname>(), true) {
        return errno::EFAULT;
    }
    
    let uname = unsafe { &mut *(buf as *mut Utsname) };
    
    // Zero out first
    *uname = Utsname {
        sysname: [0; 65],
        nodename: [0; 65],
        release: [0; 65],
        version: [0; 65],
        machine: [0; 65],
        domainname: [0; 65],
    };
    
    // Fill in values
    copy_str_to_array(&mut uname.sysname, "TrustOS");
    copy_str_to_array(&mut uname.nodename, "trustos");
    copy_str_to_array(&mut uname.release, "1.0.0-trustos");
    copy_str_to_array(&mut uname.version, "#1 SMP PREEMPT TrustOS");
    copy_str_to_array(&mut uname.machine, "x86_64");
    copy_str_to_array(&mut uname.domainname, "(none)");
    
    0
}

fn copy_str_to_array(arr: &mut [u8; 65], s: &str) {
    let bytes = s.as_bytes();
    let len = bytes.len().min(64);
    arr[..len].copy_from_slice(&bytes[..len]);
}

// ============================================================================
// Time-related syscalls
// ============================================================================

/// timespec structure
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct Timespec {
    pub tv_sec: i64,
    pub tv_nsec: i64,
}

/// timeval structure
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct Timeval {
    pub tv_sec: i64,
    pub tv_usec: i64,
}

/// Clock IDs
pub mod clock_ids {
    pub const CLOCK_REALTIME: u32 = 0;
    pub const CLOCK_MONOTONIC: u32 = 1;
    pub const CLOCK_PROCESS_CPUTIME_ID: u32 = 2;
    pub const CLOCK_THREAD_CPUTIME_ID: u32 = 3;
    pub const CLOCK_MONOTONIC_RAW: u32 = 4;
    pub const CLOCK_REALTIME_COARSE: u32 = 5;
    pub const CLOCK_MONOTONIC_COARSE: u32 = 6;
    pub const CLOCK_BOOTTIME: u32 = 7;
}

/// sys_clock_gettime - Get time from specified clock
pub fn sys_clock_gettime(clock_id: u32, tp: u64) -> i64 {
    if !validate_user_ptr(tp, core::mem::size_of::<Timespec>(), true) {
        return errno::EFAULT;
    }
    
    let ticks = crate::time::uptime_ticks();
    let seconds = ticks / 1000;
    let nanos = (ticks % 1000) * 1_000_000;
    
    let ts = unsafe { &mut *(tp as *mut Timespec) };
    ts.tv_sec = seconds as i64;
    ts.tv_nsec = nanos as i64;
    
    0
}

/// sys_gettimeofday - Get current time
pub fn sys_gettimeofday(tv: u64, tz: u64) -> i64 {
    if tv != 0 {
        if !validate_user_ptr(tv, core::mem::size_of::<Timeval>(), true) {
            return errno::EFAULT;
        }
        
        let ticks = crate::time::uptime_ticks();
        let seconds = ticks / 1000;
        let usecs = (ticks % 1000) * 1000;
        
        let timeval = unsafe { &mut *(tv as *mut Timeval) };
        timeval.tv_sec = seconds as i64;
        timeval.tv_usec = usecs as i64;
    }
    
    // Timezone is deprecated, ignore
    0
}

/// sys_nanosleep - Sleep for specified time (yield-based)
pub fn sys_nanosleep(req: u64, rem: u64) -> i64 {
    if !validate_user_ptr(req, core::mem::size_of::<Timespec>(), false) {
        return errno::EFAULT;
    }
    
    let ts = unsafe { &*(req as *const Timespec) };
    let ms = (ts.tv_sec * 1000 + ts.tv_nsec / 1_000_000) as u64;
    
    // Yield-based sleep — gives other threads CPU time
    let start = crate::time::uptime_ticks();
    while crate::time::uptime_ticks().saturating_sub(start) < ms {
        crate::thread::yield_thread();
    }
    
    if rem != 0 && validate_user_ptr(rem, core::mem::size_of::<Timespec>(), true) {
        let rem_ts = unsafe { &mut *(rem as *mut Timespec) };
        rem_ts.tv_sec = 0;
        rem_ts.tv_nsec = 0;
    }
    
    0
}

// ============================================================================
// Random
// ============================================================================

/// sys_getrandom - Get random bytes
pub fn sys_getrandom(buf: u64, count: u64, _flags: u64) -> i64 {
    if !validate_user_ptr(buf, count as usize, true) {
        return errno::EFAULT;
    }
    
    let buffer = unsafe { core::slice::from_raw_parts_mut(buf as *mut u8, count as usize) };
    
    // Use our RNG
    for byte in buffer.iter_mut() {
        *byte = crate::rng::random_u8();
    }
    
    count as i64
}

// ============================================================================
// File-related syscalls
// ============================================================================

/// sys_ioctl - Device control
pub fn sys_ioctl(fd: i32, request: u64, arg: u64) -> i64 {
    // Common ioctl requests
    const TCGETS: u64 = 0x5401;
    const TCSETS: u64 = 0x5402;
    const TIOCGWINSZ: u64 = 0x5413;
    const TIOCSWINSZ: u64 = 0x5414;
    const FIONREAD: u64 = 0x541B;
    
    match request {
        TCGETS | TCSETS => {
            // Terminal ioctls - pretend success
            0
        }
        TIOCGWINSZ => {
            // Get window size
            if arg != 0 && validate_user_ptr(arg, 8, true) {
                let winsize = unsafe { &mut *(arg as *mut [u16; 4]) };
                winsize[0] = 25;  // rows
                winsize[1] = 80;  // cols
                winsize[2] = 0;   // xpixel
                winsize[3] = 0;   // ypixel
            }
            0
        }
        FIONREAD => {
            // Bytes available for reading
            if arg != 0 && validate_user_ptr(arg, 4, true) {
                unsafe { *(arg as *mut i32) = 0; }
            }
            0
        }
        _ => {
            crate::log_debug!("[IOCTL] Unknown ioctl fd={} request={:#x}", fd, request);
            0 // Pretend success for unknown ioctls
        }
    }
}

/// sys_fcntl - File control
pub fn sys_fcntl(fd: i32, cmd: u32, arg: u64) -> i64 {
    const F_DUPFD: u32 = 0;
    const F_GETFD: u32 = 1;
    const F_SETFD: u32 = 2;
    const F_GETFL: u32 = 3;
    const F_SETFL: u32 = 4;
    
    match cmd {
        F_GETFD => 0,  // No close-on-exec
        F_SETFD => 0,
        F_GETFL => 0,  // No flags
        F_SETFL => 0,
        _ => {
            crate::log_debug!("[FCNTL] fd={} cmd={} arg={}", fd, cmd, arg);
            0
        }
    }
}

/// stat structure (simplified)
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct Stat {
    pub st_dev: u64,
    pub st_ino: u64,
    pub st_nlink: u64,
    pub st_mode: u32,
    pub st_uid: u32,
    pub st_gid: u32,
    pub _pad0: u32,
    pub st_rdev: u64,
    pub st_size: i64,
    pub st_blksize: i64,
    pub st_blocks: i64,
    pub st_atime: i64,
    pub st_atime_nsec: i64,
    pub st_mtime: i64,
    pub st_mtime_nsec: i64,
    pub st_ctime: i64,
    pub st_ctime_nsec: i64,
    pub _unused: [i64; 3],
}

/// File type bits
pub mod stat_mode {
    pub const S_IFMT: u32 = 0o170000;
    pub const S_IFREG: u32 = 0o100000;
    pub const S_IFDIR: u32 = 0o040000;
    pub const S_IFCHR: u32 = 0o020000;
    pub const S_IFIFO: u32 = 0o010000;
    pub const S_IFLNK: u32 = 0o120000;
    pub const S_IFSOCK: u32 = 0o140000;
}

/// sys_fstat - Get file status
pub fn sys_fstat(fd: i32, statbuf: u64) -> i64 {
    if !validate_user_ptr(statbuf, core::mem::size_of::<Stat>(), true) {
        return errno::EFAULT;
    }
    
    let stat = unsafe { &mut *(statbuf as *mut Stat) };
    *stat = Stat::default();
    
    // stdin/stdout/stderr
    if fd >= 0 && fd <= 2 {
        stat.st_mode = stat_mode::S_IFCHR | 0o666;
        stat.st_rdev = 0x0500; // /dev/tty
        return 0;
    }
    
    // TODO: Get actual file info from VFS
    stat.st_mode = stat_mode::S_IFREG | 0o644;
    stat.st_size = 0;
    stat.st_blksize = 4096;
    
    0
}

/// sys_access - Check file access
pub fn sys_access(pathname: u64, mode: u32) -> i64 {
    let _path = match read_user_string(pathname, 256) {
        Some(s) => s,
        None => return errno::EFAULT,
    };
    
    // TODO: Actually check file access
    // For now, pretend files exist if they look valid
    0
}

/// sys_readlink - Read symbolic link
pub fn sys_readlink(pathname: u64, buf: u64, bufsiz: u64) -> i64 {
    let path = match read_user_string(pathname, 256) {
        Some(s) => s,
        None => return errno::EFAULT,
    };
    
    // Handle /proc/self/exe
    if path == "/proc/self/exe" {
        let exe = "/bin/program";
        let len = exe.len().min(bufsiz as usize);
        if validate_user_ptr(buf, len, true) {
            let dst = unsafe { core::slice::from_raw_parts_mut(buf as *mut u8, len) };
            dst.copy_from_slice(&exe.as_bytes()[..len]);
            return len as i64;
        }
    }
    
    errno::EINVAL
}

// ============================================================================
// Signal syscalls (stubs)
// ============================================================================

/// sys_rt_sigaction - Set signal handler
pub fn sys_rt_sigaction(sig: u32, act: u64, oldact: u64, sigsetsize: u64) -> i64 {
    let pid = crate::process::current_pid();
    crate::log_debug!("[SIGACTION] pid={} sig={} act={:#x} oldact={:#x}", pid, sig, act, oldact);

    // Return old action if requested
    if oldact != 0 && validate_user_ptr(oldact, core::mem::size_of::<crate::signals::SigAction>(), true) {
        if let Ok(old) = crate::signals::get_action(pid, sig) {
            unsafe {
                core::ptr::write(oldact as *mut crate::signals::SigAction, old);
            }
        }
    }

    // Set new action if provided
    if act != 0 && validate_user_ptr(act, core::mem::size_of::<crate::signals::SigAction>(), false) {
        let new_action = unsafe { core::ptr::read(act as *const crate::signals::SigAction) };
        if let Err(e) = crate::signals::set_action(pid, sig, new_action) {
            return e as i64;
        }
    }

    0
}

/// sys_rt_sigprocmask - Change blocked signals
pub fn sys_rt_sigprocmask(how: u32, set: u64, oldset: u64, sigsetsize: u64) -> i64 {
    let pid = crate::process::current_pid();

    let mut old_mask: u64 = 0;
    let new_set = if set != 0 && validate_user_ptr(set, 8, false) {
        unsafe { core::ptr::read(set as *const u64) }
    } else {
        0
    };

    if let Err(e) = crate::signals::set_mask(pid, how, new_set, &mut old_mask) {
        return e as i64;
    }

    // Write old mask if requested
    if oldset != 0 && validate_user_ptr(oldset, sigsetsize as usize, true) {
        unsafe {
            core::ptr::write(oldset as *mut u64, old_mask);
        }
    }

    0
}

// ============================================================================
// Resource limits
// ============================================================================

/// rlimit structure
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Rlimit {
    pub rlim_cur: u64,
    pub rlim_max: u64,
}

/// Resource limit types
pub mod rlimit_resource {
    pub const RLIMIT_CPU: u32 = 0;
    pub const RLIMIT_FSIZE: u32 = 1;
    pub const RLIMIT_DATA: u32 = 2;
    pub const RLIMIT_STACK: u32 = 3;
    pub const RLIMIT_CORE: u32 = 4;
    pub const RLIMIT_RSS: u32 = 5;
    pub const RLIMIT_NPROC: u32 = 6;
    pub const RLIMIT_NOFILE: u32 = 7;
    pub const RLIMIT_MEMLOCK: u32 = 8;
    pub const RLIMIT_AS: u32 = 9;
    pub const RLIMIT_LOCKS: u32 = 10;
    pub const RLIMIT_SIGPENDING: u32 = 11;
    pub const RLIMIT_MSGQUEUE: u32 = 12;
    pub const RLIMIT_NICE: u32 = 13;
    pub const RLIMIT_RTPRIO: u32 = 14;
    pub const RLIMIT_RTTIME: u32 = 15;
}

const RLIM_INFINITY: u64 = !0;

/// sys_getrlimit - Get resource limits
pub fn sys_getrlimit(resource: u32, rlim: u64) -> i64 {
    if !validate_user_ptr(rlim, core::mem::size_of::<Rlimit>(), true) {
        return errno::EFAULT;
    }
    
    let limit = unsafe { &mut *(rlim as *mut Rlimit) };
    
    use rlimit_resource::*;
    match resource {
        RLIMIT_STACK => {
            limit.rlim_cur = 8 * 1024 * 1024; // 8MB
            limit.rlim_max = RLIM_INFINITY;
        }
        RLIMIT_NOFILE => {
            limit.rlim_cur = 1024;
            limit.rlim_max = 1024 * 1024;
        }
        RLIMIT_AS | RLIMIT_DATA => {
            limit.rlim_cur = RLIM_INFINITY;
            limit.rlim_max = RLIM_INFINITY;
        }
        _ => {
            limit.rlim_cur = RLIM_INFINITY;
            limit.rlim_max = RLIM_INFINITY;
        }
    }
    
    0
}

/// sys_prlimit64 - Get/set resource limits
pub fn sys_prlimit64(pid: u32, resource: u32, new_limit: u64, old_limit: u64) -> i64 {
    if old_limit != 0 {
        sys_getrlimit(resource, old_limit)
    } else {
        0
    }
}

// ============================================================================
// Misc syscalls
// ============================================================================

/// sys_exit_group - Exit all threads
pub fn sys_exit_group(status: i32) -> i64 {
    crate::log!("[EXIT_GROUP] status={}", status);
    crate::process::exit(status);
    0 // Never returns
}

/// sys_set_robust_list - Set robust futex list
pub fn sys_set_robust_list(head: u64, len: u64) -> i64 {
    // Just ignore for now
    0
}

/// sys_get_robust_list - Get robust futex list
pub fn sys_get_robust_list(pid: u32, head_ptr: u64, len_ptr: u64) -> i64 {
    0
}

/// sys_prctl - Process control
pub fn sys_prctl(option: u32, arg2: u64, arg3: u64, arg4: u64, arg5: u64) -> i64 {
    const PR_SET_NAME: u32 = 15;
    const PR_GET_NAME: u32 = 16;
    
    match option {
        PR_SET_NAME => {
            // Set thread name
            if let Some(name) = read_user_string(arg2, 16) {
                crate::log_debug!("[PRCTL] Set thread name: {}", name);
            }
            0
        }
        PR_GET_NAME => {
            // Get thread name
            if validate_user_ptr(arg2, 16, true) {
                copy_str_to_user(arg2, "trustos", 16);
            }
            0
        }
        _ => {
            crate::log_debug!("[PRCTL] Unknown option {}", option);
            0
        }
    }
}

/// sys_sched_yield - Yield CPU
pub fn sys_sched_yield() -> i64 {
    crate::scheduler::yield_now();
    0
}

/// sys_sched_getaffinity - Get CPU affinity
pub fn sys_sched_getaffinity(pid: u32, cpusetsize: u64, mask: u64) -> i64 {
    if mask != 0 && validate_user_ptr(mask, cpusetsize as usize, true) {
        // Return all CPUs (simplified)
        unsafe {
            core::ptr::write_bytes(mask as *mut u8, 0xFF, cpusetsize as usize);
        }
    }
    0
}

// ============================================================================
// Helper functions
// ============================================================================

/// Read a null-terminated string from user space
pub fn read_user_string(ptr: u64, max: usize) -> Option<String> {
    if ptr == 0 || !is_user_address(ptr) {
        return None;
    }
    
    let mut s = String::new();
    for i in 0..max {
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

/// Copy a string to user space
fn copy_str_to_user(ptr: u64, s: &str, max: usize) {
    let len = s.len().min(max - 1);
    let dst = unsafe { core::slice::from_raw_parts_mut(ptr as *mut u8, max) };
    dst[..len].copy_from_slice(&s.as_bytes()[..len]);
    dst[len] = 0; // Null terminate
}

// ============================================================================
// Writev/Readv
// ============================================================================

/// iovec structure
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Iovec {
    pub iov_base: u64,
    pub iov_len: u64,
}

/// sys_writev - Write to multiple buffers
pub fn sys_writev(fd: i32, iov: u64, iovcnt: u32) -> i64 {
    if !validate_user_ptr(iov, (iovcnt as usize) * core::mem::size_of::<Iovec>(), false) {
        return errno::EFAULT;
    }
    
    let iovecs = unsafe { core::slice::from_raw_parts(iov as *const Iovec, iovcnt as usize) };
    let mut total = 0i64;
    
    for iovec in iovecs {
        if iovec.iov_len == 0 {
            continue;
        }
        if !validate_user_ptr(iovec.iov_base, iovec.iov_len as usize, false) {
            return errno::EFAULT;
        }
        
        let buf = unsafe { core::slice::from_raw_parts(iovec.iov_base as *const u8, iovec.iov_len as usize) };
        
        // stdout/stderr
        if fd == 1 || fd == 2 {
            for &b in buf {
                crate::serial_print!("{}", b as char);
            }
            total += iovec.iov_len as i64;
        } else {
            match crate::vfs::write(fd, buf) {
                Ok(n) => total += n as i64,
                Err(_) => return if total > 0 { total } else { errno::EIO },
            }
        }
    }
    
    total
}

/// sys_readv - Read from multiple buffers
pub fn sys_readv(fd: i32, iov: u64, iovcnt: u32) -> i64 {
    if !validate_user_ptr(iov, (iovcnt as usize) * core::mem::size_of::<Iovec>(), false) {
        return errno::EFAULT;
    }
    
    let iovecs = unsafe { core::slice::from_raw_parts(iov as *const Iovec, iovcnt as usize) };
    let mut total = 0i64;
    
    for iovec in iovecs {
        if iovec.iov_len == 0 {
            continue;
        }
        if !validate_user_ptr(iovec.iov_base, iovec.iov_len as usize, true) {
            return errno::EFAULT;
        }
        
        let buf = unsafe { core::slice::from_raw_parts_mut(iovec.iov_base as *mut u8, iovec.iov_len as usize) };
        
        match crate::vfs::read(fd, buf) {
            Ok(n) => {
                total += n as i64;
                if n < iovec.iov_len as usize {
                    break; // Short read
                }
            }
            Err(_) => return if total > 0 { total } else { errno::EIO },
        }
    }
    
    total
}

// ============================================================================
// File Descriptor Duplication
// ============================================================================

/// sys_dup - Duplicate file descriptor to lowest available number
pub fn sys_dup(old_fd: i32) -> i64 {
    if crate::pipe::is_pipe_fd(old_fd) {
        return old_fd as i64; // simplified pipe dup
    }
    match crate::vfs::dup_fd(old_fd) {
        Ok(new_fd) => new_fd as i64,
        Err(_) => errno::EBADF,
    }
}

/// sys_dup2 - Duplicate fd to specific target
pub fn sys_dup2(old_fd: i32, new_fd: i32) -> i64 {
    if old_fd == new_fd {
        return new_fd as i64;
    }
    if crate::pipe::is_pipe_fd(old_fd) {
        return old_fd as i64; // simplified pipe dup
    }
    match crate::vfs::dup2_fd(old_fd, new_fd) {
        Ok(fd) => fd as i64,
        Err(_) => errno::EBADF,
    }
}

// ============================================================================
// poll - Wait for events on file descriptors
// ============================================================================

/// pollfd structure (Linux-compatible)
#[repr(C)]
#[derive(Clone, Copy)]
pub struct PollFd {
    pub fd: i32,
    pub events: i16,
    pub revents: i16,
}

const POLLIN: i16 = 1;
const POLLOUT: i16 = 4;
const POLLERR: i16 = 8;
const POLLHUP: i16 = 16;
#[allow(dead_code)]
const POLLNVAL: i16 = 32;

/// sys_poll - Check readiness of file descriptors
pub fn sys_poll(fds_ptr: u64, nfds: u32, timeout_ms: i32) -> i64 {
    if nfds == 0 { return 0; }
    let size = (nfds as usize) * core::mem::size_of::<PollFd>();
    if !validate_user_ptr(fds_ptr, size, true) {
        return errno::EFAULT;
    }
    
    let fds = unsafe { core::slice::from_raw_parts_mut(fds_ptr as *mut PollFd, nfds as usize) };
    
    let max_tries = if timeout_ms == 0 { 1usize }
                    else if timeout_ms < 0 { 100 }
                    else { (timeout_ms as usize / 10).max(1) };
    
    for _ in 0..max_tries {
        let mut ready = 0i64;
        for pfd in fds.iter_mut() {
            pfd.revents = 0;
            if pfd.fd < 0 { continue; }
            
            // stdin — always readable for now
            if pfd.fd == 0 && (pfd.events & POLLIN) != 0 { pfd.revents |= POLLIN; }
            // stdout/stderr — always writable
            if (pfd.fd == 1 || pfd.fd == 2) && (pfd.events & POLLOUT) != 0 { pfd.revents |= POLLOUT; }
            // Pipes — ready
            if crate::pipe::is_pipe_fd(pfd.fd) {
                if (pfd.events & POLLIN) != 0 { pfd.revents |= POLLIN; }
                if (pfd.events & POLLOUT) != 0 { pfd.revents |= POLLOUT; }
            }
            // Sockets — ready
            if crate::netstack::socket::is_socket(pfd.fd) {
                if (pfd.events & POLLIN) != 0 { pfd.revents |= POLLIN; }
                if (pfd.events & POLLOUT) != 0 { pfd.revents |= POLLOUT; }
            }
            // Regular VFS fds — always ready for regular files
            if pfd.revents == 0 && pfd.fd >= 3 {
                if (pfd.events & POLLIN) != 0 { pfd.revents |= POLLIN; }
                if (pfd.events & POLLOUT) != 0 { pfd.revents |= POLLOUT; }
            }
            
            if pfd.revents != 0 { ready += 1; }
        }
        if ready > 0 { return ready; }
        if timeout_ms == 0 { return 0; }
        crate::thread::yield_thread();
    }
    0
}

// ============================================================================
// getdents64 - Read directory entries (Linux-compatible)
// ============================================================================

/// sys_getdents64 - Get directory entries in Linux dirent64 format
pub fn sys_getdents64(fd: i32, dirp: u64, count: u32) -> i64 {
    if !validate_user_ptr(dirp, count as usize, true) {
        return errno::EFAULT;
    }
    
    // Use CWD as directory path (fd-based dir lookup would need additional VFS
    // infrastructure; this covers the common `getdents64(open("."))` case).
    let path = crate::vfs::getcwd();
    
    let entries = match crate::vfs::readdir(&path) {
        Ok(e) => e,
        Err(_) => return errno::ENOTDIR,
    };
    
    let buf = unsafe { core::slice::from_raw_parts_mut(dirp as *mut u8, count as usize) };
    let mut offset = 0usize;
    
    for entry in &entries {
        let name_bytes = entry.name.as_bytes();
        // d_ino(8) + d_off(8) + d_reclen(2) + d_type(1) + name + NUL, then 8-byte align
        let reclen = (8 + 8 + 2 + 1 + name_bytes.len() + 1 + 7) & !7;
        
        if offset + reclen > count as usize { break; }
        
        let d_type: u8 = match entry.file_type {
            crate::vfs::FileType::Directory  => 4,
            crate::vfs::FileType::Regular    => 8,
            crate::vfs::FileType::CharDevice => 2,
            crate::vfs::FileType::BlockDevice => 6,
            crate::vfs::FileType::Symlink    => 10,
            crate::vfs::FileType::Pipe       => 1,
            crate::vfs::FileType::Socket     => 12,
        };
        
        let ptr = &mut buf[offset..];
        if ptr.len() < reclen { break; }
        
        // d_ino
        ptr[0..8].copy_from_slice(&entry.ino.to_le_bytes());
        // d_off (next offset)
        let next_off = (offset + reclen) as u64;
        ptr[8..16].copy_from_slice(&next_off.to_le_bytes());
        // d_reclen
        ptr[16..18].copy_from_slice(&(reclen as u16).to_le_bytes());
        // d_type
        ptr[18] = d_type;
        // d_name (null-terminated)
        let name_start = 19;
        let name_end = name_start + name_bytes.len();
        if name_end < ptr.len() {
            ptr[name_start..name_end].copy_from_slice(name_bytes);
            ptr[name_end] = 0;
        }
        // Zero padding
        for i in (name_end + 1)..reclen.min(ptr.len()) {
            ptr[i] = 0;
        }
        
        offset += reclen;
    }
    
    offset as i64
}

// ============================================================================
// openat - Open file relative to directory fd
// ============================================================================

/// sys_openat - Open file relative to directory fd
pub fn sys_openat(dirfd: i32, pathname: u64, flags: u32) -> i64 {
    const AT_FDCWD: i32 = -100;
    
    let path = match read_user_string(pathname, 256) {
        Some(s) => s,
        None => return errno::EFAULT,
    };
    
    // Absolute path or AT_FDCWD → regular open
    if path.starts_with('/') || dirfd == AT_FDCWD {
        return match crate::vfs::open(&path, crate::vfs::OpenFlags(flags)) {
            Ok(fd) => fd as i64,
            Err(_) => errno::ENOENT,
        };
    }
    
    // Relative to CWD as fallback
    let full_path = {
        let cwd = crate::vfs::getcwd();
        if cwd == "/" {
            alloc::format!("/{}", path)
        } else {
            alloc::format!("{}/{}", cwd, path)
        }
    };
    
    match crate::vfs::open(&full_path, crate::vfs::OpenFlags(flags)) {
        Ok(fd) => fd as i64,
        Err(_) => errno::ENOENT,
    }
}
