









pub mod runtime;
pub mod codegen;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use alloc::format;
use alloc::collections::BTreeMap;


#[derive(Debug, Clone)]
pub struct Dc {
    pub re: u64,
    pub bf: Vec<u8>,
    pub bes: String,
    pub bvr: Vec<Operand>,
    pub byv: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Operand {
    Register(Register),
    Acf(i64),
    Cy { ar: Option<Register>, index: Option<Register>, bv: u8, aor: i64 },
    Dy(String),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Register {
    Me, Bpw, Bpx, Alr, Alt, Alq, Hc, Bpv,
    Alo, Alp, Alj, Alk, All, Alm, Aln, Aec, Aw,
    Abh, Bfe, Bfg, Bfi, Bfn, Bfh, Bfo, Bfd,
    Bxn, Byj, Bzf, Cat,
    Bxh, Bxg, Byg, Byf, Aag, Byv, Cai, Cag,
}

impl Register {
    pub fn j(&self) -> &'static str {
        match self {
            Register::Me => "rax", Register::Bpw => "rbx", Register::Bpx => "rcx", Register::Alr => "rdx",
            Register::Alt => "rsi", Register::Alq => "rdi", Register::Hc => "rsp", Register::Bpv => "rbp",
            Register::Alo => "r8", Register::Alp => "r9", Register::Alj => "r10", Register::Alk => "r11",
            Register::All => "r12", Register::Alm => "r13", Register::Aln => "r14", Register::Aec => "r15",
            Register::Aw => "rip",
            Register::Abh => "eax", Register::Bfe => "ebx", Register::Bfg => "ecx", Register::Bfi => "edx",
            Register::Bfn => "esi", Register::Bfh => "edi", Register::Bfo => "esp", Register::Bfd => "ebp",
            Register::Bxn => "ax", Register::Byj => "bx", Register::Bzf => "cx", Register::Cat => "dx",
            Register::Bxh => "al", Register::Bxg => "ah", Register::Byg => "bl", Register::Byf => "bh",
            Register::Aag => "cl", Register::Byv => "ch", Register::Cai => "dl", Register::Cag => "dh",
        }
    }
    
    pub fn ivx(aj: u8, ic: bool, aw: u8) -> Self {
        let w = if ic { aj + 8 } else { aj };
        match aw {
            8 => match w {
                0 => Register::Me, 1 => Register::Bpx, 2 => Register::Alr, 3 => Register::Bpw,
                4 => Register::Hc, 5 => Register::Bpv, 6 => Register::Alt, 7 => Register::Alq,
                8 => Register::Alo, 9 => Register::Alp, 10 => Register::Alj, 11 => Register::Alk,
                12 => Register::All, 13 => Register::Alm, 14 => Register::Aln, 15 => Register::Aec,
                _ => Register::Me,
            },
            4 => match w {
                0 => Register::Abh, 1 => Register::Bfg, 2 => Register::Bfi, 3 => Register::Bfe,
                4 => Register::Bfo, 5 => Register::Bfd, 6 => Register::Bfn, 7 => Register::Bfh,
                _ => Register::Abh,
            },
            _ => Register::Me,
        }
    }
}


#[derive(Debug, Clone)]
pub struct Anj {
    pub re: u64,
    pub aqb: u64,
    pub j: &'static str,
    pub n: Vec<u64>,
}


#[derive(Debug)]
pub struct Bs {
    pub j: String,
    pub re: u64,
    pub aw: usize,
    pub instructions: Vec<Dc>,
    pub apd: Vec<Anj>,
    pub kgf: Vec<u64>,
    pub wvd: Vec<(u64, String)>,
}


#[derive(Debug)]
pub struct Rm {
    pub mi: u64,
    pub ajb: Vec<Bs>,
    pub pd: Vec<(u64, String)>,
    pub dck: Vec<&'static str>,
    pub hyh: String,
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinaryType {
    
    Buq,
    
    Bfs,
    
    Bps,
    
    Cfb,
    
    Ra,
    
    Bzh,
    
    Cgx,
    
    F,
}


pub fn gty(num: u64) -> &'static str {
    match num {
        0 => "read",
        1 => "write",
        2 => "open",
        3 => "close",
        4 => "stat",
        5 => "fstat",
        6 => "lstat",
        7 => "poll",
        8 => "lseek",
        9 => "mmap",
        10 => "mprotect",
        11 => "munmap",
        12 => "brk",
        13 => "rt_sigaction",
        14 => "rt_sigprocmask",
        15 => "rt_sigreturn",
        16 => "ioctl",
        17 => "pread64",
        18 => "pwrite64",
        19 => "readv",
        20 => "writev",
        21 => "access",
        22 => "pipe",
        23 => "select",
        24 => "sched_yield",
        25 => "mremap",
        32 => "dup",
        33 => "dup2",
        34 => "pause",
        35 => "nanosleep",
        37 => "alarm",
        38 => "setitimer",
        39 => "getpid",
        41 => "socket",
        42 => "connect",
        43 => "accept",
        44 => "sendto",
        45 => "recvfrom",
        46 => "sendmsg",
        47 => "recvmsg",
        48 => "shutdown",
        49 => "bind",
        50 => "listen",
        51 => "getsockname",
        52 => "getpeername",
        53 => "socketpair",
        54 => "setsockopt",
        55 => "getsockopt",
        56 => "clone",
        57 => "fork",
        58 => "vfork",
        59 => "execve",
        60 => "exit",
        61 => "wait4",
        62 => "kill",
        63 => "uname",
        72 => "fcntl",
        73 => "flock",
        74 => "fsync",
        75 => "fdatasync",
        76 => "truncate",
        77 => "ftruncate",
        78 => "getdents",
        79 => "getcwd",
        80 => "chdir",
        81 => "fchdir",
        82 => "rename",
        83 => "mkdir",
        84 => "rmdir",
        85 => "creat",
        86 => "link",
        87 => "unlink",
        88 => "symlink",
        89 => "readlink",
        90 => "chmod",
        91 => "fchmod",
        92 => "chown",
        93 => "fchown",
        94 => "lchown",
        95 => "umask",
        96 => "gettimeofday",
        97 => "getrlimit",
        99 => "sysinfo",
        100 => "times",
        101 => "ptrace",
        102 => "getuid",
        104 => "getgid",
        105 => "setuid",
        106 => "setgid",
        107 => "geteuid",
        108 => "getegid",
        109 => "setpgid",
        110 => "getppid",
        111 => "getpgrp",
        112 => "setsid",
        113 => "setreuid",
        114 => "setregid",
        115 => "getgroups",
        116 => "setgroups",
        117 => "setresuid",
        118 => "getresuid",
        119 => "setresgid",
        120 => "getresgid",
        121 => "getpgid",
        122 => "setfsuid",
        123 => "setfsgid",
        124 => "getsid",
        125 => "capget",
        126 => "capset",
        131 => "sigaltstack",
        137 => "statfs",
        138 => "fstatfs",
        140 => "getpriority",
        141 => "setpriority",
        142 => "sched_setparam",
        143 => "sched_getparam",
        144 => "sched_setscheduler",
        145 => "sched_getscheduler",
        146 => "sched_get_priority_max",
        147 => "sched_get_priority_min",
        157 => "prctl",
        158 => "arch_prctl",
        186 => "gettid",
        200 => "tkill",
        201 => "time",
        202 => "futex",
        203 => "sched_setaffinity",
        204 => "sched_getaffinity",
        217 => "getdents64",
        218 => "set_tid_address",
        228 => "clock_gettime",
        229 => "clock_getres",
        230 => "clock_nanosleep",
        231 => "exit_group",
        232 => "epoll_wait",
        233 => "epoll_ctl",
        234 => "tgkill",
        257 => "openat",
        258 => "mkdirat",
        259 => "mknodat",
        260 => "fchownat",
        262 => "newfstatat",
        263 => "unlinkat",
        264 => "renameat",
        265 => "linkat",
        266 => "symlinkat",
        267 => "readlinkat",
        268 => "fchmodat",
        269 => "faccessat",
        270 => "pselect6",
        271 => "ppoll",
        273 => "set_robust_list",
        274 => "get_robust_list",
        281 => "epoll_pwait",
        284 => "eventfd",
        288 => "accept4",
        290 => "eventfd2",
        291 => "epoll_create1",
        292 => "dup3",
        293 => "pipe2",
        302 => "prlimit64",
        318 => "getrandom",
        332 => "statx",
        _ => "unknown",
    }
}


pub fn pre(num: u64) -> &'static str {
    match num {
        
        0 | 1 | 2 | 3 | 39 | 60 | 63 | 79 | 102 | 104 | 107 | 108 | 110 | 186 | 231 => "Full",
        
        9 | 10 | 11 | 12 | 21 | 78 | 80 | 83 | 84 | 87 | 96 | 217 | 228 | 257 => "Partial",
        
        4 | 5 | 6 | 8 | 13 | 14 | 16 | 35 | 72 | 158 | 202 | 218 | 273 | 302 | 318 => "Stub",
        
        _ => "None",
    }
}


pub use codegen::Transpiler;


pub fn dob(f: &[u8]) -> Option<Rm> {
    if f.len() < 64 || &f[0..4] != b"\x7FELF" {
        return None;
    }
    
    let mi = u64::dj(f[24..32].try_into().bq()?);
    let bnu = u64::dj(f[32..40].try_into().bq()?) as usize;
    let egq = u16::dj(f[54..56].try_into().bq()?) as usize;
    let egp = u16::dj(f[56..58].try_into().bq()?) as usize;
    
    
    let mut fwo = 0usize;
    let mut jtb = 0u64;
    let mut fwp = 0usize;
    
    for a in 0..egp {
        let afv = bnu + a * egq;
        if afv + egq > f.len() { break; }
        
        let bku = u32::dj(f[afv..afv+4].try_into().bq()?);
        let bvv = u32::dj(f[afv+4..afv+8].try_into().bq()?);
        
        if bku == 1 && (bvv & 1) != 0 {
            fwo = u64::dj(f[afv+8..afv+16].try_into().bq()?) as usize;
            jtb = u64::dj(f[afv+16..afv+24].try_into().bq()?);
            fwp = u64::dj(f[afv+32..afv+40].try_into().bq()?) as usize;
            break;
        }
    }
    
    if fwp == 0 {
        fwo = 0x1000.v(f.len());
        jtb = mi;
        fwp = (f.len() - fwo).v(0x10000);
    }
    
    let bql = if mi >= jtb {
        (mi - jtb) as usize
    } else {
        0
    };
    
    let dez = fwo + bql;
    let aj = if dez < f.len() {
        &f[dez..f.len().v(dez + fwp)]
    } else {
        return None;
    };
    
    
    let mut disasm = Disassembler::new(aj, mi);
    let instructions = disasm.irf();
    
    
    let mut transpiler = Transpiler::new(instructions.clone());
    transpiler.qie();
    let kdi = transpiler.rwn();
    
    
    let hyh = transpiler.tcc(kdi, f);
    
    let mut dck: Vec<&'static str> = transpiler.apd.iter()
        .map(|e| e.j)
        .collect();
    dck.jqs();
    dck.rux();
    
    let pd = kut(f);
    
    Some(Rm {
        mi,
        ajb: vec![Bs {
            j: String::from("_start"),
            re: mi,
            aw: aj.len(),
            instructions,
            apd: transpiler.apd,
            kgf: Vec::new(),
            wvd: Vec::new(),
        }],
        pd,
        dck,
        hyh,
    })
}


fn kut(f: &[u8]) -> Vec<(u64, String)> {
    let mut pd = Vec::new();
    let mut cv = String::new();
    let mut ay = 0u64;
    
    for (a, &o) in f.iter().cf() {
        if o >= 0x20 && o < 0x7F {
            if cv.is_empty() {
                ay = a as u64;
            }
            cv.push(o as char);
        } else {
            if cv.len() >= 4 {
                pd.push((ay, cv.clone()));
            }
            cv.clear();
        }
    }
    
    if cv.len() >= 4 {
        pd.push((ay, cv));
    }
    
    pd
}


pub struct Disassembler<'a> {
    aj: &'a [u8],
    sm: u64,
    u: usize,
}

impl<'a> Disassembler<'a> {
    pub fn new(aj: &'a [u8], sm: u64) -> Self {
        Self { aj, sm, u: 0 }
    }
    
    pub fn irf(&mut self) -> Vec<Dc> {
        let mut instructions = Vec::new();
        let eff = 500; 
        
        while self.u < self.aj.len() && instructions.len() < eff {
            if let Some(fi) = self.uui() {
                let edy = fi.bes == "ret";
                instructions.push(fi);
                if edy {
                    break;
                }
            } else {
                break;
            }
        }
        
        instructions
    }
    
    fn uui(&mut self) -> Option<Dc> {
        let poc = self.u;
        let ag = self.sm + poc as u64;
        
        if self.u >= self.aj.len() {
            return None;
        }
        
        
        let mut aip = 0u8;
        let mut o = self.aj[self.u];
        
        
        if o >= 0x40 && o <= 0x4F {
            aip = o;
            self.u += 1;
            if self.u >= self.aj.len() { return None; }
            o = self.aj[self.u];
        }
        
        let ako = (aip & 0x08) != 0;
        let nx = (aip & 0x04) != 0;
        let ic = (aip & 0x01) != 0;
        let aw = if ako { 8 } else { 4 };
        
        
        let (bes, bvr) = match o {
            
            0x0F if self.amm(1) == Some(0x05) => {
                self.u += 2;
                ("syscall", vec![])
            }
            
            
            0xC3 => {
                self.u += 1;
                ("ret", vec![])
            }
            
            
            0x31 => {
                self.u += 1;
                if let Some(ms) = self.pah(nx, ic, aw) {
                    ("xor", ms)
                } else {
                    return None;
                }
            }
            
            
            0xB8..=0xBF => {
                let vtw = o - 0xB8;
                self.u += 1;
                let gf = if ako {
                    self.vrv()?
                } else {
                    self.pag()? as i64
                };
                let reg = Register::ivx(vtw, ic, aw);
                ("mov", vec![Operand::Register(reg), Operand::Acf(gf)])
            }
            
            
            0xC7 => {
                self.u += 1;
                if let Some(ms) = self.vsc(nx, ic, aw) {
                    ("mov", ms)
                } else {
                    return None;
                }
            }
            
            
            0x8D => {
                self.u += 1;
                if let Some(ms) = self.pah(nx, ic, aw) {
                    ("lea", ms)
                } else {
                    return None;
                }
            }
            
            
            _ => {
                self.u += 1;
                ("db", vec![Operand::Acf(o as i64)])
            }
        };
        
        let bf = self.aj[poc..self.u].ip();
        
        Some(Dc {
            re: ag,
            bf,
            bes: String::from(bes),
            bvr,
            byv: None,
        })
    }
    
    fn amm(&self, l: usize) -> Option<u8> {
        self.aj.get(self.u + l).hu()
    }
    
    fn pah(&mut self, nx: bool, ic: bool, aw: u8) -> Option<Vec<Operand>> {
        if self.u >= self.aj.len() { return None; }
        let ms = self.aj[self.u];
        self.u += 1;
        
        let czy = (ms >> 6) & 0x03;
        let reg = (ms >> 3) & 0x07;
        let hb = ms & 0x07;
        
        let vty = Operand::Register(Register::ivx(reg, nx, aw));
        let mao = Operand::Register(Register::ivx(hb, ic, aw));
        
        Some(vec![mao, vty])
    }
    
    fn vsc(&mut self, nx: bool, ic: bool, aw: u8) -> Option<Vec<Operand>> {
        if self.u >= self.aj.len() { return None; }
        let ms = self.aj[self.u];
        self.u += 1;
        
        let hb = ms & 0x07;
        let mao = Operand::Register(Register::ivx(hb, ic, aw));
        
        let gf = self.pag()?;
        
        Some(vec![mao, Operand::Acf(gf as i64)])
    }
    
    fn pag(&mut self) -> Option<i32> {
        if self.u + 4 > self.aj.len() { return None; }
        let ap = i32::dj([
            self.aj[self.u],
            self.aj[self.u + 1],
            self.aj[self.u + 2],
            self.aj[self.u + 3],
        ]);
        self.u += 4;
        Some(ap)
    }
    
    fn vrv(&mut self) -> Option<i64> {
        if self.u + 8 > self.aj.len() { return None; }
        let ap = i64::dj([
            self.aj[self.u], self.aj[self.u + 1],
            self.aj[self.u + 2], self.aj[self.u + 3],
            self.aj[self.u + 4], self.aj[self.u + 5],
            self.aj[self.u + 6], self.aj[self.u + 7],
        ]);
        self.u += 8;
        Some(ap)
    }
}
