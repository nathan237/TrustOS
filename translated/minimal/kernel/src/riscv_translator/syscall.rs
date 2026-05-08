












use alloc::string::String;
use alloc::format;
use super::ir::SourceArch;


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnifiedSyscall {
    Read,
    Write,
    Open,
    Close,
    Stat,
    Fstat,
    Lseek,
    Mmap,
    Mprotect,
    Munmap,
    Brk,
    Ioctl,
    Access,
    Dup,
    Dup2,
    Getpid,
    Fork,
    Execve,
    Exit,
    Wait4,
    Kill,
    Uname,
    Fcntl,
    Getcwd,
    Chdir,
    Mkdir,
    Rmdir,
    Unlink,
    Readlink,
    Chmod,
    Getuid,
    Getgid,
    Geteuid,
    Getegid,
    Getppid,
    Gettid,
    Clock_gettime,
    Nanosleep,
    ExitGroup,
    Openat,
    Getdents64,
    Set_tid_address,
    Arch_prctl,
    Getrandom,
    Unknown(u64),
}

impl UnifiedSyscall {
    
    pub fn lyz(arch: SourceArch, number: u64) -> Self {
        match arch {
            SourceArch::X86_64 => Self::lzt(number),
            SourceArch::Aarch64 | SourceArch::Riscv64 => Self::lyy(number),
            SourceArch::Mips64 => Self::lzm(number),
            SourceArch::Wasm => UnifiedSyscall::Unknown(number),
        }
    }

    
    fn lzt(num: u64) -> Self {
        match num {
            0   => UnifiedSyscall::Read,
            1   => UnifiedSyscall::Write,
            2   => UnifiedSyscall::Open,
            3   => UnifiedSyscall::Close,
            4   => UnifiedSyscall::Stat,
            5   => UnifiedSyscall::Fstat,
            8   => UnifiedSyscall::Lseek,
            9   => UnifiedSyscall::Mmap,
            10  => UnifiedSyscall::Mprotect,
            11  => UnifiedSyscall::Munmap,
            12  => UnifiedSyscall::Brk,
            16  => UnifiedSyscall::Ioctl,
            21  => UnifiedSyscall::Access,
            32  => UnifiedSyscall::Dup,
            33  => UnifiedSyscall::Dup2,
            35  => UnifiedSyscall::Nanosleep,
            39  => UnifiedSyscall::Getpid,
            57  => UnifiedSyscall::Fork,
            59  => UnifiedSyscall::Execve,
            60  => UnifiedSyscall::Exit,
            61  => UnifiedSyscall::Wait4,
            62  => UnifiedSyscall::Kill,
            63  => UnifiedSyscall::Uname,
            72  => UnifiedSyscall::Fcntl,
            79  => UnifiedSyscall::Getcwd,
            80  => UnifiedSyscall::Chdir,
            83  => UnifiedSyscall::Mkdir,
            84  => UnifiedSyscall::Rmdir,
            87  => UnifiedSyscall::Unlink,
            89  => UnifiedSyscall::Readlink,
            90  => UnifiedSyscall::Chmod,
            102 => UnifiedSyscall::Getuid,
            104 => UnifiedSyscall::Getgid,
            107 => UnifiedSyscall::Geteuid,
            108 => UnifiedSyscall::Getegid,
            110 => UnifiedSyscall::Getppid,
            158 => UnifiedSyscall::Arch_prctl,
            186 => UnifiedSyscall::Gettid,
            217 => UnifiedSyscall::Getdents64,
            218 => UnifiedSyscall::Set_tid_address,
            228 => UnifiedSyscall::Clock_gettime,
            231 => UnifiedSyscall::ExitGroup,
            257 => UnifiedSyscall::Openat,
            318 => UnifiedSyscall::Getrandom,
            _   => UnifiedSyscall::Unknown(num),
        }
    }

    
    fn lyy(num: u64) -> Self {
        match num {
            17  => UnifiedSyscall::Getcwd,
            23  => UnifiedSyscall::Dup,
            24  => UnifiedSyscall::Dup2,  
            25  => UnifiedSyscall::Fcntl,
            29  => UnifiedSyscall::Ioctl,
            34  => UnifiedSyscall::Mkdir,   
            35  => UnifiedSyscall::Unlink,  
            48  => UnifiedSyscall::Access,  
            49  => UnifiedSyscall::Chdir,
            56  => UnifiedSyscall::Openat,
            57  => UnifiedSyscall::Close,
            62  => UnifiedSyscall::Lseek,
            63  => UnifiedSyscall::Read,
            64  => UnifiedSyscall::Write,
            78  => UnifiedSyscall::Readlink, 
            79  => UnifiedSyscall::Fstat,    
            80  => UnifiedSyscall::Fstat,
            93  => UnifiedSyscall::Exit,
            94  => UnifiedSyscall::ExitGroup,
            96  => UnifiedSyscall::Set_tid_address,
            101 => UnifiedSyscall::Nanosleep,
            113 => UnifiedSyscall::Clock_gettime,
            124 => UnifiedSyscall::Kill,
            129 => UnifiedSyscall::Kill,     
            160 => UnifiedSyscall::Uname,
            172 => UnifiedSyscall::Getpid,
            173 => UnifiedSyscall::Getppid,
            174 => UnifiedSyscall::Getuid,
            175 => UnifiedSyscall::Geteuid,
            176 => UnifiedSyscall::Getgid,
            177 => UnifiedSyscall::Getegid,
            178 => UnifiedSyscall::Gettid,
            214 => UnifiedSyscall::Brk,
            215 => UnifiedSyscall::Munmap,
            222 => UnifiedSyscall::Mmap,
            226 => UnifiedSyscall::Mprotect,
            220 => UnifiedSyscall::Fork,     
            221 => UnifiedSyscall::Execve,
            233 => UnifiedSyscall::Wait4,    
            261 => UnifiedSyscall::Getdents64,
            278 => UnifiedSyscall::Getrandom,
            _   => UnifiedSyscall::Unknown(num),
        }
    }

    
    fn lzm(num: u64) -> Self {
        match num {
            5000 => UnifiedSyscall::Read,
            5001 => UnifiedSyscall::Write,
            5002 => UnifiedSyscall::Open,
            5003 => UnifiedSyscall::Close,
            5005 => UnifiedSyscall::Fstat,
            5008 => UnifiedSyscall::Lseek,
            5009 => UnifiedSyscall::Mmap,
            5010 => UnifiedSyscall::Mprotect,
            5011 => UnifiedSyscall::Munmap,
            5012 => UnifiedSyscall::Brk,
            5038 => UnifiedSyscall::Getpid,
            5057 => UnifiedSyscall::Fork,
            5058 => UnifiedSyscall::Execve,
            5059 => UnifiedSyscall::Exit,
            5061 => UnifiedSyscall::Uname,
            5079 => UnifiedSyscall::Getcwd,
            5093 => UnifiedSyscall::Exit,  
            _    => UnifiedSyscall::Unknown(num),
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            UnifiedSyscall::Read => "read",
            UnifiedSyscall::Write => "write",
            UnifiedSyscall::Open => "open",
            UnifiedSyscall::Close => "close",
            UnifiedSyscall::Stat => "stat",
            UnifiedSyscall::Fstat => "fstat",
            UnifiedSyscall::Lseek => "lseek",
            UnifiedSyscall::Mmap => "mmap",
            UnifiedSyscall::Mprotect => "mprotect",
            UnifiedSyscall::Munmap => "munmap",
            UnifiedSyscall::Brk => "brk",
            UnifiedSyscall::Ioctl => "ioctl",
            UnifiedSyscall::Access => "access",
            UnifiedSyscall::Dup => "dup",
            UnifiedSyscall::Dup2 => "dup2",
            UnifiedSyscall::Getpid => "getpid",
            UnifiedSyscall::Fork => "fork",
            UnifiedSyscall::Execve => "execve",
            UnifiedSyscall::Exit => "exit",
            UnifiedSyscall::Wait4 => "wait4",
            UnifiedSyscall::Kill => "kill",
            UnifiedSyscall::Uname => "uname",
            UnifiedSyscall::Fcntl => "fcntl",
            UnifiedSyscall::Getcwd => "getcwd",
            UnifiedSyscall::Chdir => "chdir",
            UnifiedSyscall::Mkdir => "mkdir",
            UnifiedSyscall::Rmdir => "rmdir",
            UnifiedSyscall::Unlink => "unlink",
            UnifiedSyscall::Readlink => "readlink",
            UnifiedSyscall::Chmod => "chmod",
            UnifiedSyscall::Getuid => "getuid",
            UnifiedSyscall::Getgid => "getgid",
            UnifiedSyscall::Geteuid => "geteuid",
            UnifiedSyscall::Getegid => "getegid",
            UnifiedSyscall::Getppid => "getppid",
            UnifiedSyscall::Gettid => "gettid",
            UnifiedSyscall::Clock_gettime => "clock_gettime",
            UnifiedSyscall::Nanosleep => "nanosleep",
            UnifiedSyscall::ExitGroup => "exit_group",
            UnifiedSyscall::Openat => "openat",
            UnifiedSyscall::Getdents64 => "getdents64",
            UnifiedSyscall::Set_tid_address => "set_tid_address",
            UnifiedSyscall::Arch_prctl => "arch_prctl",
            UnifiedSyscall::Getrandom => "getrandom",
            UnifiedSyscall::Unknown(_) => "unknown",
        }
    }
}



pub fn handle_syscall(
    src_arch: SourceArch,
    number: u64,
    args: &[u64; 6],
    mem: &mut super::interpreter::RvMemory,
) -> (i64, bool) {
    let syscall = UnifiedSyscall::lyz(src_arch, number);

    crate::serial_println!("[RV-XLAT] Syscall: {} ({}) from {} [args: 0x{:x}, 0x{:x}, 0x{:x}]",
        syscall.name(), number, src_arch.name(), args[0], args[1], args[2]);

    match syscall {
        UnifiedSyscall::Write => {
            
            let fd = args[0];
            let kem = args[1];
            let count = args[2] as usize;

            if fd == 1 || fd == 2 {
                
                let mut dgu = 0usize;
                for i in 0..count {
                    if let Ok(b) = mem.read_u8(kem + i as u64) {
                        crate::serial_print!("{}", b as char);
                        dgu += 1;
                    } else {
                        break;
                    }
                }
                (dgu as i64, false)
            } else {
                (-9i64, false) 
            }
        }

        UnifiedSyscall::Read => {
            
            (0, false)
        }

        UnifiedSyscall::Exit | UnifiedSyscall::ExitGroup => {
            let code = args[0] as i64;
            crate::serial_println!("[RV-XLAT] Process exited with code {}", code);
            (code, true)
        }

        UnifiedSyscall::Brk => {
            
            (0x1000_0000i64, false)
        }

        UnifiedSyscall::Mmap => {
            
            let addr = args[0];
            let len = args[1] as usize;
            let het = if addr != 0 { addr } else { 0x4000_0000 + mem.total_allocated as u64 };
            if len > 0 && len <= 64 * 1024 * 1024 {
                mem.map(het, len);
                (het as i64, false)
            } else {
                (-12i64, false) 
            }
        }

        UnifiedSyscall::Munmap => {
            
            (0, false)
        }

        UnifiedSyscall::Mprotect => {
            
            (0, false)
        }

        UnifiedSyscall::Getpid => (1000, false),
        UnifiedSyscall::Getppid => (1, false),
        UnifiedSyscall::Getuid | UnifiedSyscall::Geteuid => (0, false),
        UnifiedSyscall::Getgid | UnifiedSyscall::Getegid => (0, false),
        UnifiedSyscall::Gettid => (1000, false),

        UnifiedSyscall::Uname => {
            
            let buf = args[0];
            
            let fields = [
                "TrustOS",                    
                "trustos",                     
                "0.7.0-rv-xlat",              
                "Universal RISC-V Translator", 
                "rv64gc",                      
                "trustos.local",              
            ];
            for (i, field) in fields.iter().enumerate() {
                let _ = mem.write_string(buf + (i * 65) as u64, field);
            }
            (0, false)
        }

        UnifiedSyscall::Getcwd => {
            let buf = args[0];
            let _ = mem.write_string(buf, "/");
            (buf as i64, false)
        }

        UnifiedSyscall::Set_tid_address => (1000, false),
        UnifiedSyscall::Arch_prctl => (0, false),

        UnifiedSyscall::Open | UnifiedSyscall::Openat => {
            
            (-2i64, false)
        }

        UnifiedSyscall::Close => (0, false),

        UnifiedSyscall::Clock_gettime => {
            
            let buf = args[1];
            let _ = mem.write_u64(buf, 1709664000); 
            let _ = mem.write_u64(buf + 8, 0);      
            (0, false)
        }

        UnifiedSyscall::Getrandom => {
            
            let buf = args[0];
            let count = args[1] as usize;
            let mut rng_state: u64 = 0xDEAD_BEEF_CAFE_1234;
            for i in 0..count {
                rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1);
                let _ = mem.write_u8(buf + i as u64, (rng_state >> 33) as u8);
            }
            (count as i64, false)
        }

        UnifiedSyscall::Unknown(num) => {
            crate::serial_println!("[RV-XLAT] WARNING: unhandled syscall {} from {}", num, src_arch.name());
            (-38i64, false) 
        }

        _ => {
            crate::serial_println!("[RV-XLAT] STUB: {} not fully implemented", syscall.name());
            (-38i64, false)
        }
    }
}
