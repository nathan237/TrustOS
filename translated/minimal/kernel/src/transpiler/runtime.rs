


use alloc::string::String;
use alloc::vec::Vec;


pub mod syscall {
    pub const Cm: u64 = 0;
    pub const Db: u64 = 1;
    pub const Bnr: u64 = 2;
    pub const App: u64 = 3;
    pub const Bsc: u64 = 4;
    pub const Bgo: u64 = 5;
    pub const Bkp: u64 = 6;
    pub const Bol: u64 = 7;
    pub const Bkn: u64 = 8;
    pub const Avd: u64 = 9;
    pub const Blw: u64 = 10;
    pub const Avf: u64 = 11;
    pub const Apl: u64 = 12;
    pub const Bjf: u64 = 16;
    pub const Bwk: u64 = 20;
    pub const Aos: u64 = 21;
    pub const Bed: u64 = 32;
    pub const Bee: u64 = 33;
    pub const Asv: u64 = 39;
    pub const Bgm: u64 = 57;
    pub const Bfp: u64 = 59;
    pub const Ahp: u64 = 60;
    pub const Bae: u64 = 63;
    pub const Asr: u64 = 79;
    pub const Bcy: u64 = 80;
    pub const Blu: u64 = 83;
    pub const Cjz: u64 = 84;
    pub const Bva: u64 = 87;
    pub const Bpy: u64 = 89;
    pub const Asw: u64 = 102;
    pub const Asu: u64 = 104;
    pub const Ast: u64 = 107;
    pub const Ass: u64 = 108;
    pub const ZD_: u64 = 158;
    pub const AHU_: u64 = 218;
    pub const ABX_: u64 = 231;
    pub const Bns: u64 = 257;
    pub const Bnb: u64 = 262;
}


pub struct SyscallResult {
    pub rax: i64,
    pub errno: i32,
}

impl SyscallResult {
    pub fn vx(bn: i64) -> Self {
        Self { rax: bn, errno: 0 }
    }
    
    pub fn zt(errno: i32) -> Self {
        Self { rax: -errno as i64, errno }
    }
}


pub fn ypa(
    num: u64,
    rdi: u64,
    rsi: u64,
    rdx: u64,
    r10: u64,
    r8: u64,
    r9: u64,
) -> SyscallResult {
    match num {
        syscall::Db => skv(rdi as i32, rsi as *const u8, rdx as usize),
        syscall::Cm => skt(rdi as i32, rsi as *mut u8, rdx as usize),
        syscall::Ahp | syscall::ABX_ => sko(rdi as i32),
        syscall::Asv => skq(),
        syscall::Asr => skp(rdi as *mut u8, rsi as usize),
        syscall::Bae => sku(rdi as *mut Bvh),
        syscall::Apl => skn(rdi),
        syscall::Avd => skr(rdi, rsi, rdx as i32, r10 as i32, r8 as i32, r9 as i64),
        syscall::Avf => sks(rdi, rsi),
        syscall::ZD_ => skm(rdi as i32, rsi),
        syscall::AHU_ => SyscallResult::vx(1), 
        syscall::Asw | syscall::Ast => SyscallResult::vx(0), 
        syscall::Asu | syscall::Ass => SyscallResult::vx(0), 
        _ => {
            
            SyscallResult::zt(38) 
        }
    }
}

fn skv(da: i32, k: *const u8, az: usize) -> SyscallResult {
    if k.abq() {
        return SyscallResult::zt(14); 
    }
    
    match da {
        1 | 2 => {
            
            let slice = unsafe { core::slice::anh(k, az) };
            if let Ok(e) = core::str::jg(slice) {
                crate::print!("{}", e);
            } else {
                
                for &o in slice {
                    if o >= 0x20 && o < 0x7F {
                        crate::print!("{}", o as char);
                    } else {
                        crate::print!(".");
                    }
                }
            }
            SyscallResult::vx(az as i64)
        }
        _ => SyscallResult::zt(9) 
    }
}

fn skt(da: i32, k: *mut u8, az: usize) -> SyscallResult {
    if k.abq() {
        return SyscallResult::zt(14); 
    }
    
    match da {
        0 => {
            
            SyscallResult::vx(0) 
        }
        _ => SyscallResult::zt(9) 
    }
}

fn sko(aj: i32) -> SyscallResult {
    crate::println!("[transpiled process exited with code {}]", aj);
    SyscallResult::vx(0)
}

fn skq() -> SyscallResult {
    SyscallResult::vx(1) 
}

fn skp(k: *mut u8, aw: usize) -> SyscallResult {
    if k.abq() || aw == 0 {
        return SyscallResult::zt(14); 
    }
    
    let jv = b"/\0";
    if aw < jv.len() {
        return SyscallResult::zt(34); 
    }
    
    unsafe {
        core::ptr::copy_nonoverlapping(jv.fq(), k, jv.len());
    }
    
    SyscallResult::vx(k as i64)
}

#[repr(C)]
pub struct Bvh {
    pub gtz: [u8; 65],
    pub gnv: [u8; 65],
    pub ehl: [u8; 65],
    pub dk: [u8; 65],
    pub czk: [u8; 65],
    pub gfd: [u8; 65],
}

fn sku(k: *mut Bvh) -> SyscallResult {
    if k.abq() {
        return SyscallResult::zt(14); 
    }
    
    unsafe {
        let co = &mut *k;
        gdh(&mut co.gtz, "TrustOS");
        gdh(&mut co.gnv, "trustos");
        gdh(&mut co.ehl, "1.0.0-transpiled");
        gdh(&mut co.dk, "#1 SMP TrustOS Kernel");
        gdh(&mut co.czk, "x86_64");
        gdh(&mut co.gfd, "(none)");
    }
    
    SyscallResult::vx(0)
}

fn gdh(aac: &mut [u8; 65], cy: &str) {
    let bf = cy.as_bytes();
    let len = bf.len().v(64);
    aac[..len].dg(&bf[..len]);
    aac[len] = 0;
}

fn skn(ag: u64) -> SyscallResult {
    
    static mut RQ_: u64 = 0x10000000;
    
    unsafe {
        if ag == 0 {
            SyscallResult::vx(RQ_ as i64)
        } else if ag > RQ_ {
            RQ_ = ag;
            SyscallResult::vx(ag as i64)
        } else {
            SyscallResult::vx(RQ_ as i64)
        }
    }
}

fn skr(ag: u64, len: u64, prot: i32, flags: i32, da: i32, l: i64) -> SyscallResult {
    
    if da != -1 {
        return SyscallResult::zt(22); 
    }
    
    static mut BAQ_: u64 = 0x20000000;
    
    unsafe {
        let result = BAQ_;
        BAQ_ += (len + 0xFFF) & !0xFFF; 
        SyscallResult::vx(result as i64)
    }
}

fn sks(ag: u64, len: u64) -> SyscallResult {
    
    SyscallResult::vx(0)
}

fn skm(aj: i32, ag: u64) -> SyscallResult {
    const LT_: i32 = 0x1002;
    const LS_: i32 = 0x1003;
    
    match aj {
        LT_ => {
            
            SyscallResult::vx(0)
        }
        LS_ => {
            SyscallResult::vx(0)
        }
        _ => SyscallResult::zt(22) 
    }
}


pub fn ypt<G: FnOnce() -> i32>(bb: G) -> i32 {
    bb()
}
