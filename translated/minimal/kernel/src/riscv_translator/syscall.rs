












use alloc::string::String;
use alloc::format;
use super::ir::SourceArch;


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnifiedSyscall {
    Read,
    Write,
    Ck,
    Mx,
    Stat,
    Abw,
    Ajx,
    Adb,
    Adf,
    Adi,
    Aae,
    Aun,
    Aou,
    Abf,
    Arh,
    Abz,
    Aih,
    Ahx,
    Lk,
    Bar,
    Ajm,
    Ra,
    Ase,
    Aby,
    Apx,
    Ada,
    Brd,
    Bag,
    Axp,
    Bdn,
    Aiq,
    Aim,
    Ail,
    Aik,
    Ain,
    Aip,
    Aha,
    Avx,
    Ahy,
    Aku,
    Ata,
    Amn,
    Apc,
    Aio,
    F(u64),
}

impl UnifiedSyscall {
    
    pub fn sxp(arch: SourceArch, aqb: u64) -> Self {
        match arch {
            SourceArch::BT_ => Self::syk(aqb),
            SourceArch::Fg | SourceArch::Jy => Self::sxo(aqb),
            SourceArch::Acz => Self::syc(aqb),
            SourceArch::Aod => UnifiedSyscall::F(aqb),
        }
    }

    
    fn syk(num: u64) -> Self {
        match num {
            0   => UnifiedSyscall::Read,
            1   => UnifiedSyscall::Write,
            2   => UnifiedSyscall::Ck,
            3   => UnifiedSyscall::Mx,
            4   => UnifiedSyscall::Stat,
            5   => UnifiedSyscall::Abw,
            8   => UnifiedSyscall::Ajx,
            9   => UnifiedSyscall::Adb,
            10  => UnifiedSyscall::Adf,
            11  => UnifiedSyscall::Adi,
            12  => UnifiedSyscall::Aae,
            16  => UnifiedSyscall::Aun,
            21  => UnifiedSyscall::Aou,
            32  => UnifiedSyscall::Abf,
            33  => UnifiedSyscall::Arh,
            35  => UnifiedSyscall::Avx,
            39  => UnifiedSyscall::Abz,
            57  => UnifiedSyscall::Aih,
            59  => UnifiedSyscall::Ahx,
            60  => UnifiedSyscall::Lk,
            61  => UnifiedSyscall::Bar,
            62  => UnifiedSyscall::Ajm,
            63  => UnifiedSyscall::Ra,
            72  => UnifiedSyscall::Ase,
            79  => UnifiedSyscall::Aby,
            80  => UnifiedSyscall::Apx,
            83  => UnifiedSyscall::Ada,
            84  => UnifiedSyscall::Brd,
            87  => UnifiedSyscall::Bag,
            89  => UnifiedSyscall::Axp,
            90  => UnifiedSyscall::Bdn,
            102 => UnifiedSyscall::Aiq,
            104 => UnifiedSyscall::Aim,
            107 => UnifiedSyscall::Ail,
            108 => UnifiedSyscall::Aik,
            110 => UnifiedSyscall::Ain,
            158 => UnifiedSyscall::Apc,
            186 => UnifiedSyscall::Aip,
            217 => UnifiedSyscall::Ata,
            218 => UnifiedSyscall::Amn,
            228 => UnifiedSyscall::Aha,
            231 => UnifiedSyscall::Ahy,
            257 => UnifiedSyscall::Aku,
            318 => UnifiedSyscall::Aio,
            _   => UnifiedSyscall::F(num),
        }
    }

    
    fn sxo(num: u64) -> Self {
        match num {
            17  => UnifiedSyscall::Aby,
            23  => UnifiedSyscall::Abf,
            24  => UnifiedSyscall::Arh,  
            25  => UnifiedSyscall::Ase,
            29  => UnifiedSyscall::Aun,
            34  => UnifiedSyscall::Ada,   
            35  => UnifiedSyscall::Bag,  
            48  => UnifiedSyscall::Aou,  
            49  => UnifiedSyscall::Apx,
            56  => UnifiedSyscall::Aku,
            57  => UnifiedSyscall::Mx,
            62  => UnifiedSyscall::Ajx,
            63  => UnifiedSyscall::Read,
            64  => UnifiedSyscall::Write,
            78  => UnifiedSyscall::Axp, 
            79  => UnifiedSyscall::Abw,    
            80  => UnifiedSyscall::Abw,
            93  => UnifiedSyscall::Lk,
            94  => UnifiedSyscall::Ahy,
            96  => UnifiedSyscall::Amn,
            101 => UnifiedSyscall::Avx,
            113 => UnifiedSyscall::Aha,
            124 => UnifiedSyscall::Ajm,
            129 => UnifiedSyscall::Ajm,     
            160 => UnifiedSyscall::Ra,
            172 => UnifiedSyscall::Abz,
            173 => UnifiedSyscall::Ain,
            174 => UnifiedSyscall::Aiq,
            175 => UnifiedSyscall::Ail,
            176 => UnifiedSyscall::Aim,
            177 => UnifiedSyscall::Aik,
            178 => UnifiedSyscall::Aip,
            214 => UnifiedSyscall::Aae,
            215 => UnifiedSyscall::Adi,
            222 => UnifiedSyscall::Adb,
            226 => UnifiedSyscall::Adf,
            220 => UnifiedSyscall::Aih,     
            221 => UnifiedSyscall::Ahx,
            233 => UnifiedSyscall::Bar,    
            261 => UnifiedSyscall::Ata,
            278 => UnifiedSyscall::Aio,
            _   => UnifiedSyscall::F(num),
        }
    }

    
    fn syc(num: u64) -> Self {
        match num {
            5000 => UnifiedSyscall::Read,
            5001 => UnifiedSyscall::Write,
            5002 => UnifiedSyscall::Ck,
            5003 => UnifiedSyscall::Mx,
            5005 => UnifiedSyscall::Abw,
            5008 => UnifiedSyscall::Ajx,
            5009 => UnifiedSyscall::Adb,
            5010 => UnifiedSyscall::Adf,
            5011 => UnifiedSyscall::Adi,
            5012 => UnifiedSyscall::Aae,
            5038 => UnifiedSyscall::Abz,
            5057 => UnifiedSyscall::Aih,
            5058 => UnifiedSyscall::Ahx,
            5059 => UnifiedSyscall::Lk,
            5061 => UnifiedSyscall::Ra,
            5079 => UnifiedSyscall::Aby,
            5093 => UnifiedSyscall::Lk,  
            _    => UnifiedSyscall::F(num),
        }
    }

    pub fn j(&self) -> &'static str {
        match self {
            UnifiedSyscall::Read => "read",
            UnifiedSyscall::Write => "write",
            UnifiedSyscall::Ck => "open",
            UnifiedSyscall::Mx => "close",
            UnifiedSyscall::Stat => "stat",
            UnifiedSyscall::Abw => "fstat",
            UnifiedSyscall::Ajx => "lseek",
            UnifiedSyscall::Adb => "mmap",
            UnifiedSyscall::Adf => "mprotect",
            UnifiedSyscall::Adi => "munmap",
            UnifiedSyscall::Aae => "brk",
            UnifiedSyscall::Aun => "ioctl",
            UnifiedSyscall::Aou => "access",
            UnifiedSyscall::Abf => "dup",
            UnifiedSyscall::Arh => "dup2",
            UnifiedSyscall::Abz => "getpid",
            UnifiedSyscall::Aih => "fork",
            UnifiedSyscall::Ahx => "execve",
            UnifiedSyscall::Lk => "exit",
            UnifiedSyscall::Bar => "wait4",
            UnifiedSyscall::Ajm => "kill",
            UnifiedSyscall::Ra => "uname",
            UnifiedSyscall::Ase => "fcntl",
            UnifiedSyscall::Aby => "getcwd",
            UnifiedSyscall::Apx => "chdir",
            UnifiedSyscall::Ada => "mkdir",
            UnifiedSyscall::Brd => "rmdir",
            UnifiedSyscall::Bag => "unlink",
            UnifiedSyscall::Axp => "readlink",
            UnifiedSyscall::Bdn => "chmod",
            UnifiedSyscall::Aiq => "getuid",
            UnifiedSyscall::Aim => "getgid",
            UnifiedSyscall::Ail => "geteuid",
            UnifiedSyscall::Aik => "getegid",
            UnifiedSyscall::Ain => "getppid",
            UnifiedSyscall::Aip => "gettid",
            UnifiedSyscall::Aha => "clock_gettime",
            UnifiedSyscall::Avx => "nanosleep",
            UnifiedSyscall::Ahy => "exit_group",
            UnifiedSyscall::Aku => "openat",
            UnifiedSyscall::Ata => "getdents64",
            UnifiedSyscall::Amn => "set_tid_address",
            UnifiedSyscall::Apc => "arch_prctl",
            UnifiedSyscall::Aio => "getrandom",
            UnifiedSyscall::F(_) => "unknown",
        }
    }
}



pub fn ixo(
    gsy: SourceArch,
    aqb: u64,
    n: &[u64; 6],
    mem: &mut super::interpreter::RvMemory,
) -> (i64, bool) {
    let syscall = UnifiedSyscall::sxp(gsy, aqb);

    crate::serial_println!("[RV-XLAT] Syscall: {} ({}) from {} [args: 0x{:x}, 0x{:x}, 0x{:x}]",
        syscall.j(), aqb, gsy.j(), n[0], n[1], n[2]);

    match syscall {
        UnifiedSyscall::Write => {
            
            let da = n[0];
            let qso = n[1];
            let az = n[2] as usize;

            if da == 1 || da == 2 {
                
                let mut gwz = 0usize;
                for a in 0..az {
                    if let Ok(o) = mem.ady(qso + a as u64) {
                        crate::serial_print!("{}", o as char);
                        gwz += 1;
                    } else {
                        break;
                    }
                }
                (gwz as i64, false)
            } else {
                (-9i64, false) 
            }
        }

        UnifiedSyscall::Read => {
            
            (0, false)
        }

        UnifiedSyscall::Lk | UnifiedSyscall::Ahy => {
            let aj = n[0] as i64;
            crate::serial_println!("[RV-XLAT] Process exited with code {}", aj);
            (aj, true)
        }

        UnifiedSyscall::Aae => {
            
            (0x1000_0000i64, false)
        }

        UnifiedSyscall::Adb => {
            
            let ag = n[0];
            let len = n[1] as usize;
            let muw = if ag != 0 { ag } else { 0x4000_0000 + mem.jto as u64 };
            if len > 0 && len <= 64 * 1024 * 1024 {
                mem.map(muw, len);
                (muw as i64, false)
            } else {
                (-12i64, false) 
            }
        }

        UnifiedSyscall::Adi => {
            
            (0, false)
        }

        UnifiedSyscall::Adf => {
            
            (0, false)
        }

        UnifiedSyscall::Abz => (1000, false),
        UnifiedSyscall::Ain => (1, false),
        UnifiedSyscall::Aiq | UnifiedSyscall::Ail => (0, false),
        UnifiedSyscall::Aim | UnifiedSyscall::Aik => (0, false),
        UnifiedSyscall::Aip => (1000, false),

        UnifiedSyscall::Ra => {
            
            let k = n[0];
            
            let fields = [
                "TrustOS",                    
                "trustos",                     
                "0.7.0-rv-xlat",              
                "Universal RISC-V Translator", 
                "rv64gc",                      
                "trustos.local",              
            ];
            for (a, buj) in fields.iter().cf() {
                let _ = mem.qad(k + (a * 65) as u64, buj);
            }
            (0, false)
        }

        UnifiedSyscall::Aby => {
            let k = n[0];
            let _ = mem.qad(k, "/");
            (k as i64, false)
        }

        UnifiedSyscall::Amn => (1000, false),
        UnifiedSyscall::Apc => (0, false),

        UnifiedSyscall::Ck | UnifiedSyscall::Aku => {
            
            (-2i64, false)
        }

        UnifiedSyscall::Mx => (0, false),

        UnifiedSyscall::Aha => {
            
            let k = n[1];
            let _ = mem.tw(k, 1709664000); 
            let _ = mem.tw(k + 8, 0);      
            (0, false)
        }

        UnifiedSyscall::Aio => {
            
            let k = n[0];
            let az = n[1] as usize;
            let mut ajn: u64 = 0xDEAD_BEEF_CAFE_1234;
            for a in 0..az {
                ajn = ajn.hx(6364136223846793005).cn(1);
                let _ = mem.cvj(k + a as u64, (ajn >> 33) as u8);
            }
            (az as i64, false)
        }

        UnifiedSyscall::F(num) => {
            crate::serial_println!("[RV-XLAT] WARNING: unhandled syscall {} from {}", num, gsy.j());
            (-38i64, false) 
        }

        _ => {
            crate::serial_println!("[RV-XLAT] STUB: {} not fully implemented", syscall.j());
            (-38i64, false)
        }
    }
}
