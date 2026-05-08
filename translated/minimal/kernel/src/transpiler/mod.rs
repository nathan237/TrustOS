









pub mod runtime;
pub mod codegen;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use alloc::format;
use alloc::collections::BTreeMap;


#[derive(Debug, Clone)]
pub struct Bj {
    pub address: u64,
    pub bytes: Vec<u8>,
    pub mnemonic: String,
    pub operands: Vec<Operand>,
    pub comment: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Operand {
    Register(Register),
    Immediate(i64),
    Memory { base: Option<Register>, index: Option<Register>, scale: u8, uv: i64 },
    Br(String),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Register {
    Fa, RBX, RCX, RDX, RSI, RDI, De, RBP,
    R8, R9, R10, R11, R12, R13, R14, R15, Af,
    EAX, EBX, ECX, EDX, ESI, EDI, ESP, EBP,
    AX, BX, CX, DX,
    AL, AH, BL, BH, CL, CH, DL, DH,
}

impl Register {
    pub fn name(&self) -> &'static str {
        match self {
            Register::Fa => "rax", Register::RBX => "rbx", Register::RCX => "rcx", Register::RDX => "rdx",
            Register::RSI => "rsi", Register::RDI => "rdi", Register::De => "rsp", Register::RBP => "rbp",
            Register::R8 => "r8", Register::R9 => "r9", Register::R10 => "r10", Register::R11 => "r11",
            Register::R12 => "r12", Register::R13 => "r13", Register::R14 => "r14", Register::R15 => "r15",
            Register::Af => "rip",
            Register::EAX => "eax", Register::EBX => "ebx", Register::ECX => "ecx", Register::EDX => "edx",
            Register::ESI => "esi", Register::EDI => "edi", Register::ESP => "esp", Register::EBP => "ebp",
            Register::AX => "ax", Register::BX => "bx", Register::CX => "cx", Register::DX => "dx",
            Register::AL => "al", Register::AH => "ah", Register::BL => "bl", Register::BH => "bh",
            Register::CL => "cl", Register::CH => "ch", Register::DL => "dl", Register::DH => "dh",
        }
    }
    
    pub fn enl(code: u8, cq: bool, size: u8) -> Self {
        let idx = if cq { code + 8 } else { code };
        match size {
            8 => match idx {
                0 => Register::Fa, 1 => Register::RCX, 2 => Register::RDX, 3 => Register::RBX,
                4 => Register::De, 5 => Register::RBP, 6 => Register::RSI, 7 => Register::RDI,
                8 => Register::R8, 9 => Register::R9, 10 => Register::R10, 11 => Register::R11,
                12 => Register::R12, 13 => Register::R13, 14 => Register::R14, 15 => Register::R15,
                _ => Register::Fa,
            },
            4 => match idx {
                0 => Register::EAX, 1 => Register::ECX, 2 => Register::EDX, 3 => Register::EBX,
                4 => Register::ESP, 5 => Register::EBP, 6 => Register::ESI, 7 => Register::EDI,
                _ => Register::EAX,
            },
            _ => Register::Fa,
        }
    }
}


#[derive(Debug, Clone)]
pub struct Ql {
    pub address: u64,
    pub number: u64,
    pub name: &'static str,
    pub args: Vec<u64>,
}


#[derive(Debug)]
pub struct Aq {
    pub name: String,
    pub address: u64,
    pub size: usize,
    pub instructions: Vec<Bj>,
    pub syscalls: Vec<Ql>,
    pub fkp: Vec<u64>,
    pub strings_used: Vec<(u64, String)>,
}


#[derive(Debug)]
pub struct Hf {
    pub entry_point: u64,
    pub functions: Vec<Aq>,
    pub strings: Vec<(u64, String)>,
    pub syscalls_used: Vec<&'static str>,
    pub rust_code: String,
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinaryType {
    
    TrueFalse,
    
    Echo,
    
    Pwd,
    
    Hostname,
    
    Uname,
    
    Cat,
    
    Ls,
    
    Unknown,
}


pub fn dfe(num: u64) -> &'static str {
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


pub fn jla(num: u64) -> &'static str {
    match num {
        
        0 | 1 | 2 | 3 | 39 | 60 | 63 | 79 | 102 | 104 | 107 | 108 | 110 | 186 | 231 => "Full",
        
        9 | 10 | 11 | 12 | 21 | 78 | 80 | 83 | 84 | 87 | 96 | 217 | 228 | 257 => "Partial",
        
        4 | 5 | 6 | 8 | 13 | 14 | 16 | 35 | 72 | 158 | 202 | 218 | 273 | 302 | 318 => "Stub",
        
        _ => "None",
    }
}


pub use codegen::Transpiler;


pub fn bks(data: &[u8]) -> Option<Hf> {
    if data.len() < 64 || &data[0..4] != b"\x7FELF" {
        return None;
    }
    
    let entry_point = u64::from_le_bytes(data[24..32].try_into().ok()?);
    let aii = u64::from_le_bytes(data[32..40].try_into().ok()?) as usize;
    let but = u16::from_le_bytes(data[54..56].try_into().ok()?) as usize;
    let bur = u16::from_le_bytes(data[56..58].try_into().ok()?) as usize;
    
    
    let mut crm = 0usize;
    let mut fcv = 0u64;
    let mut crn = 0usize;
    
    for i in 0..bur {
        let qc = aii + i * but;
        if qc + but > data.len() { break; }
        
        let p_type = u32::from_le_bytes(data[qc..qc+4].try_into().ok()?);
        let p_flags = u32::from_le_bytes(data[qc+4..qc+8].try_into().ok()?);
        
        if p_type == 1 && (p_flags & 1) != 0 {
            crm = u64::from_le_bytes(data[qc+8..qc+16].try_into().ok()?) as usize;
            fcv = u64::from_le_bytes(data[qc+16..qc+24].try_into().ok()?);
            crn = u64::from_le_bytes(data[qc+32..qc+40].try_into().ok()?) as usize;
            break;
        }
    }
    
    if crn == 0 {
        crm = 0x1000.min(data.len());
        fcv = entry_point;
        crn = (data.len() - crm).min(0x10000);
    }
    
    let entry_offset = if entry_point >= fcv {
        (entry_point - fcv) as usize
    } else {
        0
    };
    
    let code_start = crm + entry_offset;
    let code = if code_start < data.len() {
        &data[code_start..data.len().min(code_start + crn)]
    } else {
        return None;
    };
    
    
    let mut disasm = Disassembler::new(code, entry_point);
    let instructions = disasm.disassemble_all();
    
    
    let mut transpiler = Transpiler::new(instructions.clone());
    transpiler.analyze_syscalls();
    let fje = transpiler.detect_binary_type();
    
    
    let rust_code = transpiler.generate_functional_rust(fje, data);
    
    let mut syscalls_used: Vec<&'static str> = transpiler.syscalls.iter()
        .map(|j| j.name)
        .collect();
    syscalls_used.sort();
    syscalls_used.dedup();
    
    let strings = fvx(data);
    
    Some(Hf {
        entry_point,
        functions: vec![Aq {
            name: String::from("_start"),
            address: entry_point,
            size: code.len(),
            instructions,
            syscalls: transpiler.syscalls,
            fkp: Vec::new(),
            strings_used: Vec::new(),
        }],
        strings,
        syscalls_used,
        rust_code,
    })
}


fn fvx(data: &[u8]) -> Vec<(u64, String)> {
    let mut strings = Vec::new();
    let mut current = String::new();
    let mut start = 0u64;
    
    for (i, &b) in data.iter().enumerate() {
        if b >= 0x20 && b < 0x7F {
            if current.is_empty() {
                start = i as u64;
            }
            current.push(b as char);
        } else {
            if current.len() >= 4 {
                strings.push((start, current.clone()));
            }
            current.clear();
        }
    }
    
    if current.len() >= 4 {
        strings.push((start, current));
    }
    
    strings
}


pub struct Disassembler<'a> {
    code: &'a [u8],
    base_addr: u64,
    pos: usize,
}

impl<'a> Disassembler<'a> {
    pub fn new(code: &'a [u8], base_addr: u64) -> Self {
        Self { code, base_addr, pos: 0 }
    }
    
    pub fn disassemble_all(&mut self) -> Vec<Bj> {
        let mut instructions = Vec::new();
        let max_instructions = 500; 
        
        while self.pos < self.code.len() && instructions.len() < max_instructions {
            if let Some(inst) = self.next_instruction() {
                let is_ret = inst.mnemonic == "ret";
                instructions.push(inst);
                if is_ret {
                    break;
                }
            } else {
                break;
            }
        }
        
        instructions
    }
    
    fn next_instruction(&mut self) -> Option<Bj> {
        let jij = self.pos;
        let addr = self.base_addr + jij as u64;
        
        if self.pos >= self.code.len() {
            return None;
        }
        
        
        let mut rp = 0u8;
        let mut b = self.code[self.pos];
        
        
        if b >= 0x40 && b <= 0x4F {
            rp = b;
            self.pos += 1;
            if self.pos >= self.code.len() { return None; }
            b = self.code[self.pos];
        }
        
        let rex_w = (rp & 0x08) != 0;
        let gb = (rp & 0x04) != 0;
        let cq = (rp & 0x01) != 0;
        let size = if rex_w { 8 } else { 4 };
        
        
        let (mnemonic, operands) = match b {
            
            0x0F if self.peek(1) == Some(0x05) => {
                self.pos += 2;
                ("syscall", vec![])
            }
            
            
            0xC3 => {
                self.pos += 1;
                ("ret", vec![])
            }
            
            
            0x31 => {
                self.pos += 1;
                if let Some(fi) = self.read_modrm(gb, cq, size) {
                    ("xor", fi)
                } else {
                    return None;
                }
            }
            
            
            0xB8..=0xBF => {
                let oed = b - 0xB8;
                self.pos += 1;
                let imm = if rex_w {
                    self.read_imm64()?
                } else {
                    self.read_imm32()? as i64
                };
                let reg = Register::enl(oed, cq, size);
                ("mov", vec![Operand::Register(reg), Operand::Immediate(imm)])
            }
            
            
            0xC7 => {
                self.pos += 1;
                if let Some(fi) = self.read_modrm_with_imm32(gb, cq, size) {
                    ("mov", fi)
                } else {
                    return None;
                }
            }
            
            
            0x8D => {
                self.pos += 1;
                if let Some(fi) = self.read_modrm(gb, cq, size) {
                    ("lea", fi)
                } else {
                    return None;
                }
            }
            
            
            _ => {
                self.pos += 1;
                ("db", vec![Operand::Immediate(b as i64)])
            }
        };
        
        let bytes = self.code[jij..self.pos].to_vec();
        
        Some(Bj {
            address: addr,
            bytes,
            mnemonic: String::from(mnemonic),
            operands,
            comment: None,
        })
    }
    
    fn peek(&self, offset: usize) -> Option<u8> {
        self.code.get(self.pos + offset).copied()
    }
    
    fn read_modrm(&mut self, gb: bool, cq: bool, size: u8) -> Option<Vec<Operand>> {
        if self.pos >= self.code.len() { return None; }
        let fi = self.code[self.pos];
        self.pos += 1;
        
        let bct = (fi >> 6) & 0x03;
        let reg = (fi >> 3) & 0x07;
        let rm = fi & 0x07;
        
        let oef = Operand::Register(Register::enl(reg, gb, size));
        let grw = Operand::Register(Register::enl(rm, cq, size));
        
        Some(vec![grw, oef])
    }
    
    fn read_modrm_with_imm32(&mut self, gb: bool, cq: bool, size: u8) -> Option<Vec<Operand>> {
        if self.pos >= self.code.len() { return None; }
        let fi = self.code[self.pos];
        self.pos += 1;
        
        let rm = fi & 0x07;
        let grw = Operand::Register(Register::enl(rm, cq, size));
        
        let imm = self.read_imm32()?;
        
        Some(vec![grw, Operand::Immediate(imm as i64)])
    }
    
    fn read_imm32(&mut self) -> Option<i32> {
        if self.pos + 4 > self.code.len() { return None; }
        let val = i32::from_le_bytes([
            self.code[self.pos],
            self.code[self.pos + 1],
            self.code[self.pos + 2],
            self.code[self.pos + 3],
        ]);
        self.pos += 4;
        Some(val)
    }
    
    fn read_imm64(&mut self) -> Option<i64> {
        if self.pos + 8 > self.code.len() { return None; }
        let val = i64::from_le_bytes([
            self.code[self.pos], self.code[self.pos + 1],
            self.code[self.pos + 2], self.code[self.pos + 3],
            self.code[self.pos + 4], self.code[self.pos + 5],
            self.code[self.pos + 6], self.code[self.pos + 7],
        ]);
        self.pos += 8;
        Some(val)
    }
}
