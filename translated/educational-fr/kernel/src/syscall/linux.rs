//! Linux-compatible System Calls
//!
//! Implements the most essential Linux syscalls needed to run statically-linked
//! Linux binaries. This is a compatibility layer, not a full Linux implementation.

use crate::memory::{validate_user_pointer, is_user_address};
use crate::syscall::errno;
use alloc::string::String;
use alloc::vec::Vec;

/// Linux syscall numbers (x86_64 ABI)
pub mod nr {
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
const RT_SIGACTION: u64 = 13;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const RT_SIGPROCMASK: u64 = 14;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const RT_SIGRETURN: u64 = 15;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const IOCTL: u64 = 16;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const PREAD64: u64 = 17;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const PWRITE64: u64 = 18;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const READV: u64 = 19;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const WRITEV: u64 = 20;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const ACCESS: u64 = 21;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const PIPE: u64 = 22;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SELECT: u64 = 23;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SCHED_YIELD: u64 = 24;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const MREMAP: u64 = 25;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const MSYNC: u64 = 26;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const MINCORE: u64 = 27;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const MADVISE: u64 = 28;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const DUP: u64 = 32;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const DUP2: u64 = 33;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const PAUSE: u64 = 34;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const NANOSLEEP: u64 = 35;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const GETITIMER: u64 = 36;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const ALARM: u64 = 37;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SETITIMER: u64 = 38;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const GETPID: u64 = 39;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SENDFILE: u64 = 40;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SOCKET: u64 = 41;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const CONNECT: u64 = 42;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const ACCEPT: u64 = 43;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SENDTO: u64 = 44;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const RECVFROM: u64 = 45;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SENDMSG: u64 = 46;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const RECVMSG: u64 = 47;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SHUTDOWN: u64 = 48;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const BIND: u64 = 49;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const LISTEN: u64 = 50;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const GETSOCKNAME: u64 = 51;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const GETPEERNAME: u64 = 52;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SOCKETPAIR: u64 = 53;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SETSOCKOPT: u64 = 54;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const GETSOCKOPT: u64 = 55;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const CLONE: u64 = 56;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const FORK: u64 = 57;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const VFORK: u64 = 58;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const EXECVE: u64 = 59;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const EXIT: u64 = 60;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const WAIT4: u64 = 61;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const KILL: u64 = 62;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const UNAME: u64 = 63;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const FCNTL: u64 = 72;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const FLOCK: u64 = 73;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const FSYNC: u64 = 74;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const FDATASYNC: u64 = 75;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const TRUNCATE: u64 = 76;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const FTRUNCATE: u64 = 77;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const GETDENTS: u64 = 78;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const GETCWD: u64 = 79;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const CHDIR: u64 = 80;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const FCHDIR: u64 = 81;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const RENAME: u64 = 82;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const MKDIR: u64 = 83;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const RMDIR: u64 = 84;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const CREAT: u64 = 85;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const LINK: u64 = 86;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const UNLINK: u64 = 87;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SYMLINK: u64 = 88;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const READLINK: u64 = 89;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const CHMOD: u64 = 90;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const FCHMOD: u64 = 91;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const CHOWN: u64 = 92;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const FCHOWN: u64 = 93;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const LCHOWN: u64 = 94;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const UMASK: u64 = 95;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const GETTIMEOFDAY: u64 = 96;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const GETRLIMIT: u64 = 97;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const GETRUSAGE: u64 = 98;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SYSINFO: u64 = 99;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const TIMES: u64 = 100;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const GETUID: u64 = 102;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SYSLOG: u64 = 103;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const GETGID: u64 = 104;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SETUID: u64 = 105;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SETGID: u64 = 106;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const GETEUID: u64 = 107;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const GETEGID: u64 = 108;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SETPGID: u64 = 109;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const GETPPID: u64 = 110;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const GETPGRP: u64 = 111;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SETSID: u64 = 112;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SETREUID: u64 = 113;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SETREGID: u64 = 114;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const GETGROUPS: u64 = 115;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SETGROUPS: u64 = 116;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SETRESUID: u64 = 117;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const GETRESUID: u64 = 118;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SETRESGID: u64 = 119;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const GETRESGID: u64 = 120;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const GETPGID: u64 = 121;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SETFSUID: u64 = 122;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SETFSGID: u64 = 123;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const GETSID: u64 = 124;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const CAPGET: u64 = 125;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const CAPSET: u64 = 126;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const RT_SIGPENDING: u64 = 127;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const RT_SIGTIMEDWAIT: u64 = 128;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const RT_SIGQUEUEINFO: u64 = 129;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const RT_SIGSUSPEND: u64 = 130;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SIGALTSTACK: u64 = 131;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const UTIME: u64 = 132;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const MKNOD: u64 = 133;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const USELIB: u64 = 134;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const PERSONALITY: u64 = 135;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const USTAT: u64 = 136;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const STATFS: u64 = 137;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const FSTATFS: u64 = 138;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SYSFS: u64 = 139;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const GETPRIORITY: u64 = 140;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SETPRIORITY: u64 = 141;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SCHEDULER_SETPARAM: u64 = 142;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SCHEDULER_GETPARAM: u64 = 143;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SCHEDULER_SETSCHEDULER: u64 = 144;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SCHEDULER_GETSCHEDULER: u64 = 145;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SCHEDULER_GET_PRIORITY_MAXIMUM: u64 = 146;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SCHEDULER_GET_PRIORITY_MINIMUM: u64 = 147;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SCHEDULER_RR_GET_INTERVAL: u64 = 148;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const MLOCK: u64 = 149;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const MUNLOCK: u64 = 150;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const MLOCKALL: u64 = 151;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const MUNLOCKALL: u64 = 152;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const VHANGUP: u64 = 153;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const MODIFY_LDT: u64 = 154;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const PIVOT_ROOT: u64 = 155;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const PRCTL: u64 = 157;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const ARCH_PRCTL: u64 = 158;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const ADJTIMEX: u64 = 159;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SETRLIMIT: u64 = 160;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const CHROOT: u64 = 161;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SYNC: u64 = 162;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const ACCT: u64 = 163;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SETTIMEOFDAY: u64 = 164;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const MOUNT: u64 = 165;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const UMOUNT2: u64 = 166;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SWAPON: u64 = 167;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SWAPOFF: u64 = 168;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const REBOOT: u64 = 169;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SETHOSTNAME: u64 = 170;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SETDOMAINNAME: u64 = 171;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const IOPL: u64 = 172;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const IOPERM: u64 = 173;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const GETTID: u64 = 186;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const READAHEAD: u64 = 187;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SETXATTR: u64 = 188;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const GETXATTR: u64 = 191;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const LISTXATTR: u64 = 194;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const REMOVEXATTR: u64 = 197;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const TKILL: u64 = 200;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const TIME: u64 = 201;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const FUTEX: u64 = 202;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SCHEDULER_SETAFFINITY: u64 = 203;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SCHEDULER_GETAFFINITY: u64 = 204;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SET_THREAD_AREA: u64 = 205;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const IO_SETUP: u64 = 206;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const IO_DESTROY: u64 = 207;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const IO_GETEVENTS: u64 = 208;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const IO_SUBMIT: u64 = 209;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const IO_CANCEL: u64 = 210;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const GET_THREAD_AREA: u64 = 211;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const LOOKUP_DCOOKIE: u64 = 212;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const EPOLL_CREATE: u64 = 213;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const REMAP_FILE_PAGES: u64 = 216;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const GETDENTS64: u64 = 217;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SET_TID_ADDRESS: u64 = 218;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const RESTART_SYSCALL: u64 = 219;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SEMTIMEDOP: u64 = 220;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const FADVISE64: u64 = 221;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const TIMER_CREATE: u64 = 222;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const TIMER_SETTIME: u64 = 223;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const TIMER_GETTIME: u64 = 224;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const TIMER_GETOVERRUN: u64 = 225;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const TIMER_DELETE: u64 = 226;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const CLOCK_SETTIME: u64 = 227;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const CLOCK_GETTIME: u64 = 228;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const CLOCK_GETRES: u64 = 229;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const CLOCK_NANOSLEEP: u64 = 230;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const EXIT_GROUP: u64 = 231;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const EPOLL_WAIT: u64 = 232;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const EPOLL_CONTROLLER: u64 = 233;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const TGKILL: u64 = 234;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const UTIMES: u64 = 235;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const MBIND: u64 = 237;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SET_MEMPOLICY: u64 = 238;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const GET_MEMPOLICY: u64 = 239;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const MQ_OPEN: u64 = 240;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const MQ_UNLINK: u64 = 241;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const MQ_TIMEDSEND: u64 = 242;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const MQ_TIMEDRECEIVE: u64 = 243;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const MQ_NOTIFY: u64 = 244;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const MQ_GETSETATTR: u64 = 245;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const KEXEC_LOAD: u64 = 246;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const WAITID: u64 = 247;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const ADD_KEY: u64 = 248;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const REQUEST_KEY: u64 = 249;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const KEYCTL: u64 = 250;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const IOPRIO_SET: u64 = 251;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const IOPRIO_GET: u64 = 252;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const INOTIFY_INITIALIZE: u64 = 253;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const INOTIFY_ADD_WATCH: u64 = 254;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const INOTIFY_RM_WATCH: u64 = 255;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const MIGRATE_PAGES: u64 = 256;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const OPENAT: u64 = 257;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const MKDIRAT: u64 = 258;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const MKNODAT: u64 = 259;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const FCHOWNAT: u64 = 260;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const FUTIMESAT: u64 = 261;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const NEWFSTATAT: u64 = 262;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const UNLINKAT: u64 = 263;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const RENAMEAT: u64 = 264;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const LINKAT: u64 = 265;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SYMLINKAT: u64 = 266;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const READLINKAT: u64 = 267;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const FCHMODAT: u64 = 268;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const FACCESSAT: u64 = 269;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const PSELECT6: u64 = 270;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const PPOLL: u64 = 271;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const UNSHARE: u64 = 272;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SET_ROBUST_LIST: u64 = 273;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const GET_ROBUST_LIST: u64 = 274;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SPLICE: u64 = 275;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const TEE: u64 = 276;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SYNC_FILE_RANGE: u64 = 277;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const VMSPLICE: u64 = 278;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const MOVE_PAGES: u64 = 279;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const UTIMENSAT: u64 = 280;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const EPOLL_PWAIT: u64 = 281;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SIGNALFD: u64 = 282;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const TIMERFD_CREATE: u64 = 283;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const EVENTFD: u64 = 284;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const FALLOCATE: u64 = 285;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const TIMERFD_SETTIME: u64 = 286;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const TIMERFD_GETTIME: u64 = 287;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const ACCEPT4: u64 = 288;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SIGNALFD4: u64 = 289;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const EVENTFD2: u64 = 290;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const EPOLL_CREATE1: u64 = 291;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const DUP3: u64 = 292;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const PIPE2: u64 = 293;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const INOTIFY_INIT1: u64 = 294;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const PREADV: u64 = 295;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const PWRITEV: u64 = 296;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const RT_TGSIGQUEUEINFO: u64 = 297;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const PERF_EVENT_OPEN: u64 = 298;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const RECVMMSG: u64 = 299;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const FANOTIFY_INITIALIZE: u64 = 300;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const FANOTIFY_MARK: u64 = 301;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const PRLIMIT64: u64 = 302;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const NAME_TO_HANDLE_AT: u64 = 303;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const OPEN_BY_HANDLE_AT: u64 = 304;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const CLOCK_ADJTIME: u64 = 305;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SYNCFS: u64 = 306;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SENDMMSG: u64 = 307;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SETNS: u64 = 308;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const GETCPU: u64 = 309;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const PROCESS_VM_READV: u64 = 310;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const PROCESS_VM_WRITEV: u64 = 311;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const KCMP: u64 = 312;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const FINIT_MODULE: u64 = 313;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SCHEDULER_SETATTR: u64 = 314;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SCHEDULER_GETATTR: u64 = 315;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const RENAMEAT2: u64 = 316;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SECCOMP: u64 = 317;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const GETRANDOM: u64 = 318;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const MEMFD_CREATE: u64 = 319;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const KEXEC_FILE_LOAD: u64 = 320;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const BPF: u64 = 321;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const EXECVEAT: u64 = 322;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const USERFAULTFD: u64 = 323;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const MEMBARRIER: u64 = 324;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const MLOCK2: u64 = 325;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const COPY_FILE_RANGE: u64 = 326;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const PREADV2: u64 = 327;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const PWRITEV2: u64 = 328;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const PKEY_MPROTECT: u64 = 329;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const PKEY_ALLOCATOR: u64 = 330;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const PKEY_FREE: u64 = 331;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const STATX: u64 = 332;
}

// ============================================================================
// Memory Management
// ============================================================================

/// mmap flags
pub mod mmap_flags {
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const MAP_SHARED: u64 = 0x01;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const MAP_PRIVATE: u64 = 0x02;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const MAP_FIXED: u64 = 0x10;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const MAP_ANONYMOUS: u64 = 0x20;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const MAP_GROWSDOWN: u64 = 0x100;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const MAP_DENYWRITE: u64 = 0x800;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const MAP_EXECUTABLE: u64 = 0x1000;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const MAP_LOCKED: u64 = 0x2000;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const MAP_NORESERVE: u64 = 0x4000;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const MAP_POPULATE: u64 = 0x8000;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const MAP_NONBLOCK: u64 = 0x10000;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const MAP_STACK: u64 = 0x20000;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const MAP_HUGETLB: u64 = 0x40000;
}

/// mmap protection flags
pub mod prot_flags {
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const PROT_NONE: u64 = 0x0;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const PROT_READ: u64 = 0x1;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const PROT_WRITE: u64 = 0x2;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const PROT_EXECUTE: u64 = 0x4;
}

use spin::Mutex;
use core::sync::atomic::{AtomicU64, Ordering};

/// Current program break (heap end) — legacy, kept for fallback only
static PROGRAM_BREAK: AtomicU64 = AtomicU64::new(0);

/// Next available mmap address (user region)
static NEXT_MMAP_ADDRESS: AtomicU64 = AtomicU64::new(0x4000_0000); // 1 GB, well inside user space

/// sys_mmap - Map memory (lazy: records VMA, pages faulted in on demand)
/// 
/// For anonymous mappings, only metadata is stored. Physical frames are
/// allocated lazily by the page fault handler on first access.
pub fn system_mmap(addr: u64, length: u64, prot: u64, flags: u64, fd: i64, _offset: u64) -> i64 {
    use mmap_flags::*;
    use prot_flags::*;
    use crate::memory::paging::PageFlags;
    
    if length == 0 {
        return errno::EINVAL;
    }
    
    let page_size = 4096u64;
    let aligned_length = (length + page_size - 1) & !(page_size - 1);
    
    // Determine the mapping address
    let map_address = if addr != 0 && (flags & MAP_FIXED) != 0 {
        let aligned = addr & !(page_size - 1); // page-align
        // Reject MAP_FIXED targeting kernel-space addresses
        if !crate::memory::is_user_address(aligned) {
            return errno::EINVAL;
        }
        aligned
    } else {
        // Kernel chooses address
        NEXT_MMAP_ADDRESS.fetch_add(aligned_length, Ordering::SeqCst)
    };
    
    // Only anonymous mappings for now
    let is_anonymous = (flags & MAP_ANONYMOUS) != 0 || fd < 0;
    if !is_anonymous {
        crate::log_debug!("[MMAP] File-backed mmap not yet implemented");
        return errno::ENOSYS;
    }
    
    // Record the VMA for demand paging
    let cr3: u64;
    #[cfg(target_arch = "x86_64")]
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { core::arch::asm!("mov {}, cr3", out(reg) cr3, options(nostack, preserves_flags)); }
    #[cfg(not(target_arch = "x86_64"))]
    { cr3 = 0; }
    
    let vma_prot = (prot & 0x7) as u32; // PROT_READ | PROT_WRITE | PROT_EXEC
    
    crate::memory::vma::add_vma(cr3, crate::memory::vma::Vma {
        start: map_address,
        end: map_address + aligned_length,
        prot: vma_prot,
        flags: crate::memory::vma::flags::MAP_ANONYMOUS | crate::memory::vma::flags::MAP_PRIVATE,
    });
    
    crate::log_debug!("[MMAP] Lazy VMA {:#x}..{:#x} prot={:#x}", map_address, map_address + aligned_length, prot);
    map_address as i64
}

/// sys_munmap - Unmap memory and free frames
pub fn system_munmap(addr: u64, length: u64) -> i64 {
    if addr == 0 || length == 0 {
        return errno::EINVAL;
    }
    
    let page_size = 4096u64;
    let aligned_length = (length + page_size - 1) & !(page_size - 1);
    let number_pages = (aligned_length / page_size) as usize;
    let start = addr & !(page_size - 1);
    
    // Remove VMA tracking
    let cr3: u64;
    #[cfg(target_arch = "x86_64")]
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { core::arch::asm!("mov {}, cr3", out(reg) cr3, options(nostack, preserves_flags)); }
    #[cfg(not(target_arch = "x86_64"))]
    { cr3 = 0; }
    crate::memory::vma::remove_vma_range(cr3, start, start + aligned_length);
    
    // Free any faulted-in physical frames
    crate::exec::with_current_address_space(|space| {
        for i in 0..number_pages {
            let virt = start + (i as u64 * page_size);
            if let Some(phys) = space.translate(virt) {
                let physical_page = phys & !0xFFF;
                space.unmap_page(virt);
                crate::memory::frame::free_frame(physical_page);
            }
        }
    });
    
    crate::log_debug!("[MUNMAP] Unmapped {} pages at {:#x}", number_pages, addr);
    0
}

/// sys_mprotect - Change memory protection (walks page tables)
pub fn system_mprotect(addr: u64, length: u64, prot: u64) -> i64 {
    use prot_flags::*;
    use crate::memory::paging::{PageFlags, PageTable};
    
    if addr == 0 || addr & 0xFFF != 0 {
        return errno::EINVAL;
    }
    
    let page_size = 4096u64;
    let aligned_length = (length + page_size - 1) & !(page_size - 1);
    let number_pages = (aligned_length / page_size) as usize;
    
    // Build new permission flags
    let mut pf = PageFlags::PRESENT | PageFlags::USER;
    if (prot & PROT_WRITE) != 0 {
        pf |= PageFlags::WRITABLE;
    }
    if (prot & PROT_EXECUTE) == 0 {
        pf |= PageFlags::NO_EXECUTE;
    }
    let new_flags = PageFlags::new(pf);
    
    crate::exec::with_current_address_space(|space| {
        let hhdm = crate::memory::hhdm_offset();
        let cr3 = space.cr3();
        
        for i in 0..number_pages {
            let virt = addr + (i as u64 * page_size);
            let pml4_index = ((virt >> 39) & 0x1FF) as usize;
            let pdpt_index = ((virt >> 30) & 0x1FF) as usize;
            let pd_index   = ((virt >> 21) & 0x1FF) as usize;
            let pt_index   = ((virt >> 12) & 0x1FF) as usize;
            
            let pml4 = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { &*((cr3 + hhdm) as *// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const PageTable) };
            if !pml4.entries[pml4_index].is_present() { continue; }
            let pdpt = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { &*((pml4.entries[pml4_index].phys_addr() + hhdm) as *// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const PageTable) };
            if !pdpt.entries[pdpt_index].is_present() { continue; }
            let pd = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { &*((pdpt.entries[pdpt_index].phys_addr() + hhdm) as *// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const PageTable) };
            if !pd.entries[pd_index].is_present() { continue; }
            let pt = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { &mut *((pd.entries[pd_index].phys_addr() + hhdm) as *mut PageTable) };
            if !pt.entries[pt_index].is_present() { continue; }
            
            let phys = pt.entries[pt_index].phys_addr();
            pt.entries[pt_index].set(phys, new_flags);
            #[cfg(target_arch = "x86_64")]
                        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { core::arch::asm!("invlpg [{}]", in(reg) virt, options(nostack, preserves_flags)); }
        }
    });
    
    crate::log_debug!("[MPROTECT] addr={:#x} len={:#x} prot={:#x}", addr, length, prot);
    0
}

/// sys_brk - Change program break (heap end)
///
/// Eagerly allocates and maps physical frames when the break is extended.
pub fn system_brk(addr: u64) -> i64 {
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
                    let phys = // Correspondance de motifs — branchement exhaustif de Rust.
match crate::memory::frame::allocator_frame_zeroed() {
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
pub fn system_getpid() -> i64 {
    crate::process::current_pid() as i64
}

/// sys_getppid - Get parent process ID
pub fn system_getppid() -> i64 {
    crate::process::with_current(|p| p.ppid as i64)
        .unwrap_or(0)
}

/// sys_gettid - Get thread ID
pub fn system_gettid() -> i64 {
    crate::thread::current_tid() as i64
}

/// sys_getuid - Get user ID
pub fn system_getuid() -> i64 {
    let (uid, _, _, _) = crate::process::current_credentials();
    uid as i64
}

/// sys_getgid - Get group ID
pub fn system_getgid() -> i64 {
    let (_, gid, _, _) = crate::process::current_credentials();
    gid as i64
}

/// sys_geteuid - Get effective user ID  
pub fn system_geteuid() -> i64 {
    let (_, _, euid, _) = crate::process::current_credentials();
    euid as i64
}

/// sys_getegid - Get effective group ID
pub fn system_getegid() -> i64 {
    let (_, _, _, egid) = crate::process::current_credentials();
    egid as i64
}

/// sys_setuid - Set user ID
pub fn system_setuid(uid: u32) -> i64 {
    let pid = crate::process::current_pid();
        // Correspondance de motifs — branchement exhaustif de Rust.
match crate::process::set_uid(pid, uid) {
        Ok(()) => 0,
        Err(_) => -1, // EPERM
    }
}

/// sys_setgid - Set group ID
pub fn system_setgid(gid: u32) -> i64 {
    let pid = crate::process::current_pid();
        // Correspondance de motifs — branchement exhaustif de Rust.
match crate::process::set_gid(pid, gid) {
        Ok(()) => 0,
        Err(_) => -1, // EPERM
    }
}

/// sys_setreuid - Set real and effective user IDs
pub fn system_setreuid(ruid: u32, euid: u32) -> i64 {
    let pid = crate::process::current_pid();
    // If -1 (0xFFFFFFFF), don't change
    if ruid != 0xFFFFFFFF {
        if crate::process::set_uid(pid, ruid).is_err() { return -1; }
    }
    if euid != 0xFFFFFFFF {
        // set only euid
        let mut table = crate::process::PROCESS_TABLE.write();
        if let Some(p) = table.processes.get_mut(&pid) {
            if p.euid == 0 || euid == p.uid || euid == p.euid {
                p.euid = euid;
            } else {
                return -1;
            }
        }
    }
    0
}

/// sys_setregid - Set real and effective group IDs
pub fn system_setregid(rgid: u32, egid: u32) -> i64 {
    let pid = crate::process::current_pid();
    if rgid != 0xFFFFFFFF {
        if crate::process::set_gid(pid, rgid).is_err() { return -1; }
    }
    if egid != 0xFFFFFFFF {
        let mut table = crate::process::PROCESS_TABLE.write();
        if let Some(p) = table.processes.get_mut(&pid) {
            if p.euid == 0 || egid == p.gid || egid == p.egid {
                p.egid = egid;
            } else {
                return -1;
            }
        }
    }
    0
}

/// sys_umask - Set file creation mask
pub fn system_umask(mask: u32) -> i64 {
    let pid = crate::process::current_pid();
    crate::process::set_umask(pid, mask) as i64
}

// ============================================================================
// Process groups, sessions, chroot
// ============================================================================

/// sys_setpgid - Set process group ID
pub fn system_setpgid(pid: u32, pgid: u32) -> i64 {
        // Correspondance de motifs — branchement exhaustif de Rust.
match crate::process::set_pgid(pid, pgid) {
        Ok(()) => 0,
        Err(_) => -1,
    }
}

/// sys_getpgrp - Get process group of calling process
pub fn system_getpgrp() -> i64 {
    crate::process::get_pgid(0) as i64
}

/// sys_setsid - Create a new session
pub fn system_setsid() -> i64 {
        // Correspondance de motifs — branchement exhaustif de Rust.
match crate::process::setsid() {
        Ok(sid) => sid as i64,
        Err(_) => -1,
    }
}

/// sys_getpgid - Get process group of a process
pub fn system_getpgid(pid: u32) -> i64 {
    crate::process::get_pgid(pid) as i64
}

/// sys_getsid - Get session ID
pub fn system_getsid(pid: u32) -> i64 {
    crate::process::get_sid(pid) as i64
}

/// sys_chroot - Change root directory
pub fn system_chroot(path_ptr: u64) -> i64 {
    let path = // Correspondance de motifs — branchement exhaustif de Rust.
match read_user_string(path_ptr, 256) {
        Some(s) => s,
        None => return -14, // EFAULT
    };
    let pid = crate::process::current_pid();
        // Correspondance de motifs — branchement exhaustif de Rust.
match crate::process::chroot(pid, &path) {
        Ok(()) => 0,
        Err(_) => -1, // EPERM
    }
}

/// sys_chmod - Change file mode
pub fn system_chmod(path_ptr: u64, mode: u32) -> i64 {
    let path = // Correspondance de motifs — branchement exhaustif de Rust.
match read_user_string(path_ptr, 256) {
        Some(p) => p,
        None => return -14, // EFAULT
    };
        // Correspondance de motifs — branchement exhaustif de Rust.
match crate::vfs::chmod(&path, mode) {
        Ok(()) => 0,
        Err(_) => -1, // EPERM
    }
}

/// sys_fchmod - Change file mode by fd
pub fn system_fchmod(fd: i32, mode: u32) -> i64 {
        // Correspondance de motifs — branchement exhaustif de Rust.
match crate::vfs::fchmod(fd, mode) {
        Ok(()) => 0,
        Err(_) => -1,
    }
}

/// sys_chown - Change file owner
pub fn system_chown(path_ptr: u64, uid: u32, gid: u32) -> i64 {
    let path = // Correspondance de motifs — branchement exhaustif de Rust.
match read_user_string(path_ptr, 256) {
        Some(p) => p,
        None => return -14,
    };
    // Only root can chown
    let (_, _, euid, _) = crate::process::current_credentials();
    if euid != 0 { return -1; } // EPERM
    match crate::vfs::chown(&path, uid, gid) {
        Ok(()) => 0,
        Err(_) => -1,
    }
}

/// sys_fchown - Change file owner by fd
pub fn system_fchown(fd: i32, uid: u32, gid: u32) -> i64 {
    let (_, _, euid, _) = crate::process::current_credentials();
    if euid != 0 { return -1; }
        // Correspondance de motifs — branchement exhaustif de Rust.
match crate::vfs::fchown(fd, uid, gid) {
        Ok(()) => 0,
        Err(_) => -1,
    }
}

// ============================================================================
// arch_prctl - Architecture-specific thread state
// ============================================================================

/// arch_prctl codes
pub mod arch_prctl_codes {
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const ARCH_SET_GS: u64 = 0x1001;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const ARCH_SET_FILESYSTEM: u64 = 0x1002;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const ARCH_GET_FILESYSTEM: u64 = 0x1003;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const ARCH_GET_GS: u64 = 0x1004;
}

/// Thread-local storage base
static TLS_BASE: AtomicU64 = AtomicU64::new(0);

/// sys_arch_prctl - Set architecture-specific thread state
pub fn system_arch_prctl(code: u64, addr: u64) -> i64 {
    use arch_prctl_codes::*;
    
        // Correspondance de motifs — branchement exhaustif de Rust.
match code {
        ARCH_SET_FILESYSTEM => {
            // Set FS base register (used for TLS)
            TLS_BASE.store(addr, Ordering::SeqCst);
            
            // Actually set the FS base using MSR
            #[cfg(target_arch = "x86_64")]
                        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
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
            #[cfg(target_arch = "x86_64")]
                        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
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
        ARCH_GET_FILESYSTEM => {
            if !is_user_address(addr) {
                return errno::EFAULT;
            }
            let val = TLS_BASE.load(Ordering::SeqCst);
                        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { *(addr as *mut u64) = val; }
            0
        }
        ARCH_GET_GS => {
            if !is_user_address(addr) {
                return errno::EFAULT;
            }
            let val: u64;
            #[cfg(target_arch = "x86_64")]
                        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
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
            #[cfg(not(target_arch = "x86_64"))]
            { val = 0; }
                        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
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
pub fn system_set_tid_address(tidptr: u64) -> i64 {
    CLEAR_CHILD_TID.store(tidptr, Ordering::SeqCst);
    system_gettid()
}

// ============================================================================
// uname - System information
// ============================================================================

/// utsname structure
#[repr(C)]
// Structure publique — visible à l'extérieur de ce module.
pub struct Utsname {
    pub sysname: [u8; 65],
    pub nodename: [u8; 65],
    pub release: [u8; 65],
    pub version: [u8; 65],
    pub machine: [u8; 65],
    pub domainname: [u8; 65],
}

/// sys_uname - Get system information
pub fn system_uname(buf: u64) -> i64 {
    if !validate_user_pointer(buf, core::mem::size_of::<Utsname>(), true) {
        return errno::EFAULT;
    }
    
    let uname = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { &mut *(buf as *mut Utsname) };
    
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
// #[derive] — génère automatiquement les implémentations de traits à la compilation.
#[derive(Clone, Copy, Default)]
// Structure publique — visible à l'extérieur de ce module.
pub struct Timespec {
    pub tv_sec: i64,
    pub tv_nsec: i64,
}

/// timeval structure
#[repr(C)]
// #[derive] — génère automatiquement les implémentations de traits à la compilation.
#[derive(Clone, Copy, Default)]
// Structure publique — visible à l'extérieur de ce module.
pub struct Timeval {
    pub tv_sec: i64,
    pub tv_usec: i64,
}

/// Clock IDs
pub mod clock_ids {
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const CLOCK_REALTIME: u32 = 0;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const CLOCK_MONOTONIC: u32 = 1;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const CLOCK_PROCESS_CPUTIME_ID: u32 = 2;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const CLOCK_THREAD_CPUTIME_ID: u32 = 3;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const CLOCK_MONOTONIC_RAW: u32 = 4;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const CLOCK_REALTIME_COARSE: u32 = 5;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const CLOCK_MONOTONIC_COARSE: u32 = 6;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const CLOCK_BOOTTIME: u32 = 7;
}

/// sys_clock_gettime - Get time from specified clock
pub fn system_clock_gettime(clock_id: u32, tp: u64) -> i64 {
    if !validate_user_pointer(tp, core::mem::size_of::<Timespec>(), true) {
        return errno::EFAULT;
    }
    
    let ticks = crate::time::uptime_ticks();
    let seconds = ticks / 1000;
    let nanos = (ticks % 1000) * 1_000_000;
    
    let ts = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { &mut *(tp as *mut Timespec) };
    ts.tv_sec = seconds as i64;
    ts.tv_nsec = nanos as i64;
    
    0
}

/// sys_gettimeofday - Get current time
pub fn system_gettimeofday(tv: u64, tz: u64) -> i64 {
    if tv != 0 {
        if !validate_user_pointer(tv, core::mem::size_of::<Timeval>(), true) {
            return errno::EFAULT;
        }
        
        let ticks = crate::time::uptime_ticks();
        let seconds = ticks / 1000;
        let usecs = (ticks % 1000) * 1000;
        
        let timeval = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { &mut *(tv as *mut Timeval) };
        timeval.tv_sec = seconds as i64;
        timeval.tv_usec = usecs as i64;
    }
    
    // Timezone is deprecated, ignore
    0
}

/// sys_nanosleep - Sleep for specified time (yield-based)
pub fn system_nanosleep(req: u64, rem: u64) -> i64 {
    if !validate_user_pointer(req, core::mem::size_of::<Timespec>(), false) {
        return errno::EFAULT;
    }
    
    let ts = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { &*(req as *// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const Timespec) };
    let ms = (ts.tv_sec * 1000 + ts.tv_nsec / 1_000_000) as u64;
    
    // Yield-based sleep — gives other threads CPU time
    let start = crate::time::uptime_ticks();
    while crate::time::uptime_ticks().saturating_sub(start) < ms {
        crate::thread::yield_thread();
    }
    
    if rem != 0 && validate_user_pointer(rem, core::mem::size_of::<Timespec>(), true) {
        let rem_ts = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { &mut *(rem as *mut Timespec) };
        rem_ts.tv_sec = 0;
        rem_ts.tv_nsec = 0;
    }
    
    0
}

// ============================================================================
// Random
// ============================================================================

/// sys_getrandom - Get random bytes
pub fn system_getrandom(buf: u64, count: u64, _flags: u64) -> i64 {
    if !validate_user_pointer(buf, count as usize, true) {
        return errno::EFAULT;
    }
    
    let buffer = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { core::slice::from_raw_parts_mut(buf as *mut u8, count as usize) };
    
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
pub fn system_ioctl(fd: i32, request: u64, argument: u64) -> i64 {
    // Common ioctl requests
    const TCGETS: u64 = 0x5401;
        // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const TCSETS: u64 = 0x5402;
        // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const TIOCGWINSZ: u64 = 0x5413;
        // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const TIOCSWINSZ: u64 = 0x5414;
        // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const FIONREAD: u64 = 0x541B;
    
        // Correspondance de motifs — branchement exhaustif de Rust.
match request {
        TCGETS | TCSETS => {
            // Terminal ioctls - pretend success
            0
        }
        TIOCGWINSZ => {
            // Get window size
            if argument != 0 && validate_user_pointer(argument, 8, true) {
                let winsize = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { &mut *(argument as *mut [u16; 4]) };
                winsize[0] = 25;  // rows
                winsize[1] = 80;  // cols
                winsize[2] = 0;   // xpixel
                winsize[3] = 0;   // ypixel
            }
            0
        }
        FIONREAD => {
            // Bytes available for reading
            if argument != 0 && validate_user_pointer(argument, 4, true) {
                                // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { *(argument as *mut i32) = 0; }
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
pub fn system_fcntl(fd: i32, cmd: u32, argument: u64) -> i64 {
    use alloc::collections::BTreeMap;
    use spin::Mutex;

        // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const F_DUPFD: u32 = 0;
        // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const F_GETFD: u32 = 1;
        // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const F_SETFD: u32 = 2;
        // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const F_GETFL: u32 = 3;
        // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const F_SETFL: u32 = 4;
        // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const F_DUPFD_CLOEXEC: u32 = 0x406;

    // Per-(pid,fd) flag storage: (fd_flags, status_flags)
    static FD_FLAGS: Mutex<BTreeMap<(u32, i32), (u32, u32)>> = Mutex::new(BTreeMap::new());

    let pid = crate::process::current_pid();
    let key = (pid, fd);

        // Correspondance de motifs — branchement exhaustif de Rust.
match cmd {
        F_DUPFD | F_DUPFD_CLOEXEC => {
                        // Correspondance de motifs — branchement exhaustif de Rust.
match crate::vfs::dup_fd(fd) {
                Ok(new_fd) => {
                    if cmd == F_DUPFD_CLOEXEC {
                        let mut flags = FD_FLAGS.lock();
                        flags.insert((pid, new_fd), (1, 0)); // FD_CLOEXEC
                    }
                    new_fd as i64
                }
                Err(_) => -9, // EBADF
            }
        }
        F_GETFD => {
            let flags = FD_FLAGS.lock();
            flags.get(&key).map(|f| f.0 as i64).unwrap_or(0)
        }
        F_SETFD => {
            let mut flags = FD_FLAGS.lock();
            let entry = flags.entry(key).or_insert((0, 0));
            entry.0 = argument as u32;
            0
        }
        F_GETFL => {
            let flags = FD_FLAGS.lock();
            flags.get(&key).map(|f| f.1 as i64).unwrap_or(0)
        }
        F_SETFL => {
            // Only allow changing O_APPEND(0x400), O_NONBLOCK(0x800), O_ASYNC(0x2000)
            let allowed = 0x400 | 0x800 | 0x2000;
            let mut flags = FD_FLAGS.lock();
            let entry = flags.entry(key).or_insert((0, 0));
            entry.1 = (entry.1 & !allowed) | (argument as u32 & allowed);
            0
        }
        _ => {
            crate::log_debug!("[FCNTL] fd={} cmd={:#x} arg={}", fd, cmd, argument);
            0
        }
    }
}

/// stat structure (simplified)
#[repr(C)]
// #[derive] — génère automatiquement les implémentations de traits à la compilation.
#[derive(Clone, Copy, Default)]
// Structure publique — visible à l'extérieur de ce module.
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
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const S_IFMT: u32 = 0o170000;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const S_IFREG: u32 = 0o100000;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const S_IFDIR: u32 = 0o040000;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const S_IFCHR: u32 = 0o020000;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const S_IFIFO: u32 = 0o010000;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const S_IFLNK: u32 = 0o120000;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const S_IFSOCK: u32 = 0o140000;
}

/// Convert VFS FileType to Linux stat mode bits
fn filetype_to_mode(ft: crate::vfs::FileType) -> u32 {
        // Correspondance de motifs — branchement exhaustif de Rust.
match ft {
        crate::vfs::FileType::Regular    => stat_mode::S_IFREG,
        crate::vfs::FileType::Directory  => stat_mode::S_IFDIR,
        crate::vfs::FileType::CharDevice => stat_mode::S_IFCHR,
        crate::vfs::FileType::BlockDevice => 0o060000, // S_IFBLK
        crate::vfs::FileType::Symlink    => stat_mode::S_IFLNK,
        crate::vfs::FileType::Pipe       => stat_mode::S_IFIFO,
        crate::vfs::FileType::Socket     => stat_mode::S_IFSOCK,
    }
}

/// Convert a VFS Stat to a Linux Stat structure
fn vfs_to_linux_status(vfs: &crate::vfs::Stat) -> Stat {
    Stat {
        st_dev: 1,
        st_ino: vfs.ino,
        st_nlink: 1,
        st_mode: filetype_to_mode(vfs.file_type) | (vfs.mode & 0o7777),
        st_uid: vfs.uid,
        st_gid: vfs.gid,
        _pad0: 0,
        st_rdev: 0,
        st_size: vfs.size as i64,
        st_blksize: vfs.block_size as i64,
        st_blocks: ((vfs.size + 511) / 512) as i64,
        st_atime: vfs.atime as i64,
        st_atime_nsec: 0,
        st_mtime: vfs.mtime as i64,
        st_mtime_nsec: 0,
        st_ctime: vfs.ctime as i64,
        st_ctime_nsec: 0,
        _unused: [0; 3],
    }
}

/// sys_fstat - Get file status by fd
pub fn system_fstat(fd: i32, statbuf: u64) -> i64 {
    if !validate_user_pointer(statbuf, core::mem::size_of::<Stat>(), true) {
        return errno::EFAULT;
    }
    
    let stat = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { &mut *(statbuf as *mut Stat) };
    
    // stdin/stdout/stderr — character device
    if fd >= 0 && fd <= 2 {
        *stat = Stat::default();
        stat.st_mode = stat_mode::S_IFCHR | 0o666;
        stat.st_rdev = 0x0500; // /dev/tty
        stat.st_blksize = 4096;
        return 0;
    }
    
    // Query VFS for real file info
    match crate::vfs::fstat_fd(fd) {
        Ok(vfs_stat) => {
            *stat = vfs_to_linux_status(&vfs_stat);
            0
        }
        Err(_) => errno::EBADF,
    }
}

/// sys_stat - Get file status by pathname
pub fn system_status(pathname: u64, statbuf: u64) -> i64 {
    let path = // Correspondance de motifs — branchement exhaustif de Rust.
match read_user_string(pathname, 4096) {
        Some(s) => s,
        None => return errno::EFAULT,
    };
    
    if !validate_user_pointer(statbuf, core::mem::size_of::<Stat>(), true) {
        return errno::EFAULT;
    }
    
    let stat = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { &mut *(statbuf as *mut Stat) };
    
        // Correspondance de motifs — branchement exhaustif de Rust.
match crate::vfs::stat(&path) {
        Ok(vfs_stat) => {
            *stat = vfs_to_linux_status(&vfs_stat);
            0
        }
        Err(_) => errno::ENOENT,
    }
}

/// sys_newfstatat - Get file status relative to directory fd
pub fn system_newfstatat(dirfd: i32, pathname: u64, statbuf: u64, _flags: u32) -> i64 {
        // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const AT_FDCWD: i32 = -100;
        // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const AT_EMPTY_PATH: u32 = 0x1000;
    
    // If AT_EMPTY_PATH and pathname is empty, stat the fd itself
    if _flags & AT_EMPTY_PATH != 0 {
        let path = read_user_string(pathname, 4096);
        if path.is_none() || path.as_ref().map_or(false, |s| s.is_empty()) {
            if dirfd >= 0 {
                return system_fstat(dirfd, statbuf);
            }
        }
    }
    
    let path = // Correspondance de motifs — branchement exhaustif de Rust.
match read_user_string(pathname, 4096) {
        Some(s) => s,
        None => return errno::EFAULT,
    };
    
    // If path is absolute or dirfd is AT_FDCWD, use path directly
    if path.starts_with('/') || dirfd == AT_FDCWD {
        return system_status(pathname, statbuf);
    }
    
    // Relative path with dirfd — not fully supported yet, treat as absolute
    system_status(pathname, statbuf)
}

/// sys_access - Check file access
pub fn system_access(pathname: u64, mode: u32) -> i64 {
    let _path = // Correspondance de motifs — branchement exhaustif de Rust.
match read_user_string(pathname, 256) {
        Some(s) => s,
        None => return errno::EFAULT,
    };
    
    // TODO: Actually check file access
    // For now, pretend files exist if they look valid
    0
}

/// sys_readlink - Read symbolic link
pub fn system_readlink(pathname: u64, buf: u64, bufsiz: u64) -> i64 {
    let path = // Correspondance de motifs — branchement exhaustif de Rust.
match read_user_string(pathname, 256) {
        Some(s) => s,
        None => return errno::EFAULT,
    };
    
    // Handle /proc/self/exe
    if path == "/proc/self/exe" {
        let exe = "/bin/program";
        let len = exe.len().min(bufsiz as usize);
        if validate_user_pointer(buf, len, true) {
            let dst = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { core::slice::from_raw_parts_mut(buf as *mut u8, len) };
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
pub fn system_rt_sigaction(sig: u32, act: u64, oldact: u64, sigsetsize: u64) -> i64 {
    let pid = crate::process::current_pid();
    crate::log_debug!("[SIGACTION] pid={} sig={} act={:#x} oldact={:#x}", pid, sig, act, oldact);

    // Return old action if requested
    if oldact != 0 && validate_user_pointer(oldact, core::mem::size_of::<crate::signals::SigAction>(), true) {
        if let Ok(old) = crate::signals::get_action(pid, sig) {
                        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
                core::ptr::write(oldact as *mut crate::signals::SigAction, old);
            }
        }
    }

    // Set new action if provided
    if act != 0 && validate_user_pointer(act, core::mem::size_of::<crate::signals::SigAction>(), false) {
        let new_action = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { core::ptr::read(act as *const crate::signals::SigAction) };
        if let Err(e) = crate::signals::set_action(pid, sig, new_action) {
            return e as i64;
        }
    }

    0
}

/// sys_rt_sigprocmask - Change blocked signals
pub fn system_rt_sigprocmask(how: u32, set: u64, oldset: u64, sigsetsize: u64) -> i64 {
    let pid = crate::process::current_pid();

    let mut old_mask: u64 = 0;
    let new_set = if set != 0 && validate_user_pointer(set, 8, false) {
                // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { core::ptr::read(set as *// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const u64) }
    } else {
        0
    };

    if let Err(e) = crate::signals::set_mask(pid, how, new_set, &mut old_mask) {
        return e as i64;
    }

    // Write old mask if requested
    if oldset != 0 && validate_user_pointer(oldset, sigsetsize as usize, true) {
                // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
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
// #[derive] — génère automatiquement les implémentations de traits à la compilation.
#[derive(Clone, Copy)]
// Structure publique — visible à l'extérieur de ce module.
pub struct Rlimit {
    pub rlim_cur: u64,
    pub rlim_max: u64,
}

/// Resource limit types
pub mod rlimit_resource {
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const RLIMIT_CPU: u32 = 0;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const RLIMIT_FSIZE: u32 = 1;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const RLIMIT_DATA: u32 = 2;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const RLIMIT_STACK: u32 = 3;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const RLIMIT_CORE: u32 = 4;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const RLIMIT_RSS: u32 = 5;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const RLIMIT_NPROC: u32 = 6;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const RLIMIT_NOFILE: u32 = 7;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const RLIMIT_MEMLOCK: u32 = 8;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const RLIMIT_AS: u32 = 9;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const RLIMIT_LOCKS: u32 = 10;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const RLIMIT_SIGPENDING: u32 = 11;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const RLIMIT_MSGQUEUE: u32 = 12;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const RLIMIT_NICE: u32 = 13;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const RLIMIT_RTPRIO: u32 = 14;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const RLIMIT_RTTIME: u32 = 15;
}

// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const RLIM_INFINITY: u64 = !0;

/// sys_getrlimit - Get resource limits
pub fn system_getrlimit(resource: u32, rlim: u64) -> i64 {
    if !validate_user_pointer(rlim, core::mem::size_of::<Rlimit>(), true) {
        return errno::EFAULT;
    }
    
    let limit = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { &mut *(rlim as *mut Rlimit) };
    
    use rlimit_resource::*;
        // Correspondance de motifs — branchement exhaustif de Rust.
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
pub fn system_prlimit64(pid: u32, resource: u32, new_limit: u64, old_limit: u64) -> i64 {
    if old_limit != 0 {
        system_getrlimit(resource, old_limit)
    } else {
        0
    }
}

// ============================================================================
// Misc syscalls
// ============================================================================

/// sys_exit_group - Exit all threads
pub fn system_exit_group(status: i32) -> i64 {
    crate::log!("[EXIT_GROUP] status={}", status);
    crate::process::exit(status);
    0 // Never returns
}

/// sys_set_robust_list - Set robust futex list
pub fn system_set_robust_list(head: u64, len: u64) -> i64 {
    // Just ignore for now
    0
}

/// sys_get_robust_list - Get robust futex list
pub fn system_get_robust_list(pid: u32, head_ptr: u64, len_ptr: u64) -> i64 {
    0
}

/// sys_prctl - Process control
pub fn system_prctl(option: u32, arg2: u64, arg3: u64, arg4: u64, arg5: u64) -> i64 {
        // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const PR_SET_NAME: u32 = 15;
        // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const PR_GET_NAME: u32 = 16;
    
        // Correspondance de motifs — branchement exhaustif de Rust.
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
            if validate_user_pointer(arg2, 16, true) {
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
pub fn system_scheduler_getaffinity(pid: u32, cpusetsize: u64, mask: u64) -> i64 {
    if mask != 0 && validate_user_pointer(mask, cpusetsize as usize, true) {
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

/// Copy a string to user space
fn copy_str_to_user(ptr: u64, s: &str, max: usize) {
    let len = s.len().min(max - 1);
    let dst = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { core::slice::from_raw_parts_mut(ptr as *mut u8, max) };
    dst[..len].copy_from_slice(&s.as_bytes()[..len]);
    dst[len] = 0; // Null terminate
}

// ============================================================================
// Writev/Readv
// ============================================================================

/// iovec structure
#[repr(C)]
// #[derive] — génère automatiquement les implémentations de traits à la compilation.
#[derive(Clone, Copy)]
// Structure publique — visible à l'extérieur de ce module.
pub struct Iovec {
    pub iov_base: u64,
    pub iov_len: u64,
}

/// sys_writev - Write to multiple buffers
pub fn system_writev(fd: i32, iov: u64, iovcnt: u32) -> i64 {
    if !validate_user_pointer(iov, (iovcnt as usize) * core::mem::size_of::<Iovec>(), false) {
        return errno::EFAULT;
    }
    
    let iovecs = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { core::slice::from_raw_parts(iov as *// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const Iovec, iovcnt as usize) };
    let mut total = 0i64;
    
    for iovec in iovecs {
        if iovec.iov_len == 0 {
            continue;
        }
        if !validate_user_pointer(iovec.iov_base, iovec.iov_len as usize, false) {
            return errno::EFAULT;
        }
        
        let buf = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { core::slice::from_raw_parts(iovec.iov_base as *// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const u8, iovec.iov_len as usize) };
        
        // stdout/stderr
        if fd == 1 || fd == 2 {
            for &b in buf {
                crate::serial_print!("{}", b as char);
            }
            total += iovec.iov_len as i64;
        } else {
                        // Correspondance de motifs — branchement exhaustif de Rust.
match crate::vfs::write(fd, buf) {
                Ok(n) => total += n as i64,
                Err(_) => return if total > 0 { total } else { errno::EIO },
            }
        }
    }
    
    total
}

/// sys_readv - Read from multiple buffers
pub fn system_readv(fd: i32, iov: u64, iovcnt: u32) -> i64 {
    if !validate_user_pointer(iov, (iovcnt as usize) * core::mem::size_of::<Iovec>(), false) {
        return errno::EFAULT;
    }
    
    let iovecs = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { core::slice::from_raw_parts(iov as *// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const Iovec, iovcnt as usize) };
    let mut total = 0i64;
    
    for iovec in iovecs {
        if iovec.iov_len == 0 {
            continue;
        }
        if !validate_user_pointer(iovec.iov_base, iovec.iov_len as usize, true) {
            return errno::EFAULT;
        }
        
        let buf = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { core::slice::from_raw_parts_mut(iovec.iov_base as *mut u8, iovec.iov_len as usize) };
        
                // Correspondance de motifs — branchement exhaustif de Rust.
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
pub fn system_dup(old_fd: i32) -> i64 {
    if crate::pipe::is_pipe_fd(old_fd) {
        return old_fd as i64; // simplified pipe dup
    }
        // Correspondance de motifs — branchement exhaustif de Rust.
match crate::vfs::dup_fd(old_fd) {
        Ok(new_fd) => new_fd as i64,
        Err(_) => errno::EBADF,
    }
}

/// sys_dup2 - Duplicate fd to specific target
pub fn system_dup2(old_fd: i32, new_fd: i32) -> i64 {
    if old_fd == new_fd {
        return new_fd as i64;
    }
    if crate::pipe::is_pipe_fd(old_fd) {
        return old_fd as i64; // simplified pipe dup
    }
        // Correspondance de motifs — branchement exhaustif de Rust.
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
// #[derive] — génère automatiquement les implémentations de traits à la compilation.
#[derive(Clone, Copy)]
// Structure publique — visible à l'extérieur de ce module.
pub struct PollFd {
    pub fd: i32,
    pub events: i16,
    pub revents: i16,
}

// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const POLLIN: i16 = 1;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const POLLOUT: i16 = 4;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const POLLERR: i16 = 8;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const POLLHUP: i16 = 16;
#[allow(dead_code)]
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const POLLNVAL: i16 = 32;

/// sys_poll - Check readiness of file descriptors
pub fn system_poll(fds_ptr: u64, nfds: u32, timeout_ms: i32) -> i64 {
    if nfds == 0 { return 0; }
    let size = (nfds as usize) * core::mem::size_of::<PollFd>();
    if !validate_user_pointer(fds_ptr, size, true) {
        return errno::EFAULT;
    }
    
    let fds = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { core::slice::from_raw_parts_mut(fds_ptr as *mut PollFd, nfds as usize) };
    
    // Compute absolute deadline
    let deadline_ns = if timeout_ms < 0 {
        u64::MAX // block forever
    } else if timeout_ms == 0 {
        0 // non-blocking poll
    } else {
        crate::time::now_ns().saturating_add((timeout_ms as u64) * 1_000_000)
    };
    
        // Boucle infinie — tourne jusqu'à un `break` explicite.
loop {
        let mut ready = 0i64;
        for pfd in fds.iter_mut() {
            pfd.revents = 0;
            if pfd.fd < 0 { continue; }
            
            if let Some(status) = crate::vfs::poll_fd(pfd.fd) {
                if (pfd.events & POLLIN)  != 0 && status.readable { pfd.revents |= POLLIN; }
                if (pfd.events & POLLOUT) != 0 && status.writable { pfd.revents |= POLLOUT; }
                if status.error  { pfd.revents |= POLLERR; }
                if status.hangup { pfd.revents |= POLLHUP; }
            } else {
                pfd.revents = POLLNVAL; // invalid fd
            }
            
            if pfd.revents != 0 { ready += 1; }
        }
        
        if ready > 0 { return ready; }
        
        // Non-blocking: return immediately
        if timeout_ms == 0 { return 0; }
        
        // Check timeout
        let now = crate::time::now_ns();
        if now >= deadline_ns { return 0; }
        
        // Sleep up to 10 ms or until deadline, whichever is sooner.
        // The thread will be woken by the timer; if an fd becomes ready
        // in the meantime the next iteration will detect it.
        let sleep_until = deadline_ns.min(now.saturating_add(10_000_000));
        crate::thread::sleep_until(sleep_until);
    }
}

// ============================================================================
// getdents64 - Read directory entries (Linux-compatible)
// ============================================================================

/// sys_getdents64 - Get directory entries in Linux dirent64 format
pub fn system_getdents64(fd: i32, dirp: u64, count: u32) -> i64 {
    if !validate_user_pointer(dirp, count as usize, true) {
        return errno::EFAULT;
    }
    
    // Use CWD as directory path (fd-based dir lookup would need additional VFS
    // infrastructure; this covers the common `getdents64(open("."))` case).
    let path = crate::vfs::getcwd();
    
    let entries = // Correspondance de motifs — branchement exhaustif de Rust.
match crate::vfs::readdir(&path) {
        Ok(e) => e,
        Err(_) => return errno::ENOTDIR,
    };
    
    let buf = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { core::slice::from_raw_parts_mut(dirp as *mut u8, count as usize) };
    let mut offset = 0usize;
    
    for entry in &entries {
        let name_bytes = entry.name.as_bytes();
        // d_ino(8) + d_off(8) + d_reclen(2) + d_type(1) + name + NUL, then 8-byte align
        let reclen = (8 + 8 + 2 + 1 + name_bytes.len() + 1 + 7) & !7;
        
        if offset + reclen > count as usize { break; }
        
        let d_type: u8 = // Correspondance de motifs — branchement exhaustif de Rust.
match entry.file_type {
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
pub fn system_openat(dirfd: i32, pathname: u64, flags: u32) -> i64 {
        // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const AT_FDCWD: i32 = -100;
    
    let path = // Correspondance de motifs — branchement exhaustif de Rust.
match read_user_string(pathname, 256) {
        Some(s) => s,
        None => return errno::EFAULT,
    };
    
    // Absolute path or AT_FDCWD → regular open
    if path.starts_with('/') || dirfd == AT_FDCWD {
        return         // Correspondance de motifs — branchement exhaustif de Rust.
match crate::vfs::open(&path, crate::vfs::OpenFlags(flags)) {
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
    
        // Correspondance de motifs — branchement exhaustif de Rust.
match crate::vfs::open(&full_path, crate::vfs::OpenFlags(flags)) {
        Ok(fd) => fd as i64,
        Err(_) => errno::ENOENT,
    }
}

/// sys_swapon — Enable swap on a file/partition
pub fn system_swapon(path_ptr: u64) -> i64 {
    let (_, _, euid, _) = crate::process::current_credentials();
    if euid != 0 { return -1; } // EPERM: must be root
    let path = // Correspondance de motifs — branchement exhaustif de Rust.
match read_user_string(path_ptr, 256) {
        Some(p) => p,
        None => return errno::EFAULT,
    };
        // Correspondance de motifs — branchement exhaustif de Rust.
match crate::memory::swap::swapon(&path) {
        Ok(()) => 0,
        Err(_) => -1,
    }
}

/// sys_swapoff — Disable swap
pub fn system_swapoff(path_ptr: u64) -> i64 {
    let (_, _, euid, _) = crate::process::current_credentials();
    if euid != 0 { return -1; }
    let path = // Correspondance de motifs — branchement exhaustif de Rust.
match read_user_string(path_ptr, 256) {
        Some(p) => p,
        None => return errno::EFAULT,
    };
        // Correspondance de motifs — branchement exhaustif de Rust.
match crate::memory::swap::swapoff(&path) {
        Ok(()) => 0,
        Err(_) => -1,
    }
}

// ============================================================================
// epoll — I/O event notification facility
// ============================================================================

use alloc::collections::BTreeMap;

/// Epoll event flags (matches Linux)
pub mod epoll_flags {
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const EPOLLIN: u32 = 0x001;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const EPOLLOUT: u32 = 0x004;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const EPOLLERR: u32 = 0x008;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const EPOLLHUP: u32 = 0x010;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const EPOLLRDHUP: u32 = 0x2000;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const EPOLLET: u32 = 0x8000_0000;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const EPOLLONESHOT: u32 = 0x4000_0000;
}

/// epoll_ctl operations
pub mod epoll_op {
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const EPOLL_CONTROLLER_ADD: i32 = 1;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const EPOLL_CONTROLLER_DEL: i32 = 2;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const EPOLL_CONTROLLER_MOD: i32 = 3;
}

/// epoll_event structure (matches Linux layout for x86-64)
#[repr(C, packed)]
// #[derive] — génère automatiquement les implémentations de traits à la compilation.
#[derive(Clone, Copy)]
// Structure publique — visible à l'extérieur de ce module.
pub struct EpollEvent {
    pub events: u32,
    pub data: u64,
}

/// An interest entry for a monitored fd
#[derive(Clone)]
struct EpollInterest {
    fd: i32,
    events: u32,
    data: u64,
}

/// An epoll instance
pub struct EpollInstance {
    /// Map of monitored fds to their interest
    interests: BTreeMap<i32, EpollInterest>,
}

/// Global table of epoll instances, keyed by epoll fd
pub // État global partagé protégé par un Mutex (verrou d'exclusion mutuelle).
static EPOLL_TABLE: Mutex<BTreeMap<i32, EpollInstance>> = Mutex::new(BTreeMap::new());

/// Next epoll fd (using a range unlikely to collide with VFS/socket fds)
static NEXT_EPOLL_FD: AtomicI32 = AtomicI32::new(500);

use core::sync::atomic::AtomicI32;

/// Check if a given fd is an epoll instance
pub fn is_epoll_fd(fd: i32) -> bool {
    EPOLL_TABLE.lock().contains_key(&fd)
}

/// sys_epoll_create1 — Create an epoll instance
pub fn system_epoll_create1(_flags: u32) -> i64 {
    let fd = NEXT_EPOLL_FD.fetch_add(1, Ordering::SeqCst);
    let instance = EpollInstance {
        interests: BTreeMap::new(),
    };
    EPOLL_TABLE.lock().insert(fd, instance);
    crate::log_debug!("[EPOLL] created fd={}", fd);
    fd as i64
}

/// sys_epoll_create — Create an epoll instance (legacy, ignores size)
pub fn system_epoll_create(_size: i32) -> i64 {
    system_epoll_create1(0)
}

/// sys_epoll_ctl — Control an epoll instance (ADD/MOD/DEL)
pub fn system_epoll_controller(epfd: i32, op: i32, fd: i32, event_ptr: u64) -> i64 {
    use epoll_op::*;
    
    let mut table = EPOLL_TABLE.lock();
    let instance = // Correspondance de motifs — branchement exhaustif de Rust.
match table.get_mut(&epfd) {
        Some(i) => i,
        None => return errno::EBADF,
    };
    
        // Correspondance de motifs — branchement exhaustif de Rust.
match op {
        EPOLL_CONTROLLER_ADD => {
            if instance.interests.contains_key(&fd) {
                return errno::EEXIST;
            }
            if event_ptr == 0 || !validate_user_pointer(event_ptr, core::mem::size_of::<EpollEvent>(), false) {
                return errno::EFAULT;
            }
            let ev = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { *(event_ptr as *// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const EpollEvent) };
            instance.interests.insert(fd, EpollInterest {
                fd,
                events: ev.events,
                data: ev.data,
            });
        }
        EPOLL_CONTROLLER_MOD => {
            if !instance.interests.contains_key(&fd) {
                return errno::ENOENT;
            }
            if event_ptr == 0 || !validate_user_pointer(event_ptr, core::mem::size_of::<EpollEvent>(), false) {
                return errno::EFAULT;
            }
            let ev = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { *(event_ptr as *// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const EpollEvent) };
            instance.interests.insert(fd, EpollInterest {
                fd,
                events: ev.events,
                data: ev.data,
            });
        }
        EPOLL_CONTROLLER_DEL => {
            if instance.interests.remove(&fd).is_none() {
                return errno::ENOENT;
            }
        }
        _ => return errno::EINVAL,
    }
    
    0
}

/// sys_epoll_wait — Wait for events on an epoll instance
pub fn system_epoll_wait(epfd: i32, events_ptr: u64, maxevents: i32, timeout_ms: i32) -> i64 {
    if maxevents <= 0 {
        return errno::EINVAL;
    }
    let event_size = core::mem::size_of::<EpollEvent>();
    let buf_size = (maxevents as usize) * event_size;
    if !validate_user_pointer(events_ptr, buf_size, true) {
        return errno::EFAULT;
    }
    
    let events_buffer = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
        core::slice::from_raw_parts_mut(events_ptr as *mut EpollEvent, maxevents as usize)
    };
    
    // Compute deadline
    let deadline_ns = if timeout_ms < 0 {
        u64::MAX
    } else if timeout_ms == 0 {
        0
    } else {
        crate::time::now_ns().saturating_add((timeout_ms as u64) * 1_000_000)
    };
    
        // Boucle infinie — tourne jusqu'à un `break` explicite.
loop {
        // Snapshot interest list under lock, then release
        let interests: Vec<EpollInterest> = {
            let table = EPOLL_TABLE.lock();
                        // Correspondance de motifs — branchement exhaustif de Rust.
match table.get(&epfd) {
                Some(inst) => inst.interests.values().cloned().collect(),
                None => return errno::EBADF,
            }
        };
        
        let mut ready = 0usize;
        for interest in &interests {
            if ready >= maxevents as usize { break; }
            
            let mut revents = 0u32;
            if let Some(status) = crate::vfs::poll_fd(interest.fd) {
                if status.readable { revents |= epoll_flags::EPOLLIN; }
                if status.writable { revents |= epoll_flags::EPOLLOUT; }
                if status.error    { revents |= epoll_flags::EPOLLERR; }
                if status.hangup   { revents |= epoll_flags::EPOLLHUP; }
            }
            
            // Only report events the user is interested in (+ always error/hup)
            let reported = revents & (interest.events | epoll_flags::EPOLLERR | epoll_flags::EPOLLHUP);
            if reported != 0 {
                events_buffer[ready] = EpollEvent {
                    events: reported,
                    data: interest.data,
                };
                ready += 1;
            }
        }
        
        if ready > 0 {
            // Handle EPOLLONESHOT: remove interests that fired
            let mut table = EPOLL_TABLE.lock();
            if let Some(inst) = table.get_mut(&epfd) {
                for i in 0..ready {
                    let data = events_buffer[i].data;
                    // Find and disable ONESHOT interests
                    for interest in inst.interests.values_mut() {
                        if interest.data == data && (interest.events & epoll_flags::EPOLLONESHOT) != 0 {
                            interest.events = 0; // disable
                        }
                    }
                }
            }
            return ready as i64;
        }
        
        // Non-blocking
        if timeout_ms == 0 { return 0; }
        
        // Check deadline
        let now = crate::time::now_ns();
        if now >= deadline_ns { return 0; }
        
        // Sleep briefly, then retry
        let sleep_until = deadline_ns.min(now.saturating_add(10_000_000));
        crate::thread::sleep_until(sleep_until);
    }
}

/// sys_epoll_pwait — epoll_wait with signal mask (signals not yet supported, delegates)
pub fn system_epoll_pwait(epfd: i32, events_ptr: u64, maxevents: i32, timeout_ms: i32, _sigmask: u64, _sigsetsize: u64) -> i64 {
    system_epoll_wait(epfd, events_ptr, maxevents, timeout_ms)
}
