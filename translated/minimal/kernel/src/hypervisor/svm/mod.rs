




pub mod vmcb;
pub mod npt;

use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use alloc::boxed::Box;


static AIM_: AtomicBool = AtomicBool::new(false);


static YU_: AtomicU32 = AtomicU32::new(0);


static mut CAD_: Option<Box<HostSaveArea>> = None;


pub mod msr {
    pub const DAY_: u32 = 0xC001_0114;        
    pub const DBH_: u32 = 0xC001_0117;  
    pub const Lh: u32 = 0xC000_0080;         
    pub const BGP_: u32 = 0xC001_0118;     
}


pub mod efer {
    pub const Bsd: u64 = 1 << 12;  
}


pub mod vm_cr {
    pub const CUR_: u64 = 1 << 4;       
    pub const BGP_: u64 = 1 << 3;      
    pub const EDK_: u64 = 1 << 1;        
    pub const DJO_: u64 = 1 << 0;      
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u64)]
pub enum SvmExitCode {
    
    Bqo = 0x00,
    Bqp = 0x03,
    Bqq = 0x04,
    Bwx = 0x10,
    Bwy = 0x13,
    Bwz = 0x14,
    
    
    Cbx = 0x40,  
    Cbw = 0x41,  
    Cwf = 0x42,
    Cbu = 0x43,  
    Cce = 0x44,  
    Cbv = 0x45,  
    Cci = 0x46,  
    Ccc = 0x47,  
    Cby = 0x48,  
    Cch = 0x4A,  
    Ccd = 0x4B,  
    Ccg = 0x4C,  
    Cbz = 0x4D,  
    Ccf = 0x4E,  
    Ccb = 0x50,  
    Cbt = 0x51,  
    Cca = 0x52,  
    Ccj = 0x53,  
    
    
    Bjr = 0x60,
    Chy = 0x61,
    Dit = 0x62,
    Nf = 0x63,
    Bvu = 0x64,
    Bdv = 0x65,
    Czl = 0x66,
    Cya = 0x67,
    Dbf = 0x68,
    Djv = 0x69,
    Czm = 0x6A,
    Cyb = 0x6B,
    Dbg = 0x6C,
    Djw = 0x6D,
    Bqm = 0x6E,
    Dfr = 0x6F,
    Dez = 0x70,
    Deq = 0x71,
    Bdu = 0x72,
    Dgi = 0x73,
    Dae = 0x74,
    Djc = 0x75,
    Bjw = 0x76,
    Bor = 0x77,
    Bit = 0x78,
    Bjx = 0x79,
    Czz = 0x7A,
    Auo = 0x7B,
    Bjz = 0x7C,
    Hx = 0x7D,
    Jr = 0x7E,
    Bub = 0x7F,
    Cxc = 0x80,
    Qt = 0x81,
    Dls = 0x82,
    Bwa = 0x83,
    Dlr = 0x84,
    Dlt = 0x85,
    Dix = 0x86,
    Csy = 0x87,
    Dis = 0x88,
    Uc = 0x89,
    Czj = 0x8A,
    Bwt = 0x8B,
    Bmo = 0x8C,
    Bmz = 0x8D,
    Chq = 0x8E,
    Bxc = 0x8F,
    
    
    Qe = 0x400,  
    
    
    Crn = 0x401,
    Cro = 0x402,
    
    
    Dlq = 0x403,
    
    
    Cgb = 0xFFFF_FFFF_FFFF_FFFF,
}

impl From<u64> for SvmExitCode {
    fn from(aj: u64) -> Self {
        match aj {
            0x60 => SvmExitCode::Bjr,
            0x61 => SvmExitCode::Chy,
            0x72 => SvmExitCode::Bdu,
            0x78 => SvmExitCode::Bit,
            0x7B => SvmExitCode::Auo,
            0x7C => SvmExitCode::Bjz,
            0x7D => SvmExitCode::Hx,
            0x7E => SvmExitCode::Jr,
            0x7F => SvmExitCode::Bub,
            0x81 => SvmExitCode::Qt,
            0x83 => SvmExitCode::Bwa,
            0x76 => SvmExitCode::Bjw,
            0x79 => SvmExitCode::Bjx,
            0x6E => SvmExitCode::Bqm,
            0x89 => SvmExitCode::Uc,
            0x8B => SvmExitCode::Bwt,
            0x8C => SvmExitCode::Bmo,
            0x8D => SvmExitCode::Bmz,
            0x8F => SvmExitCode::Bxc,
            0x77 => SvmExitCode::Bor,
            0x64 => SvmExitCode::Bvu,
            0x65 => SvmExitCode::Bdv,
            0x400 => SvmExitCode::Qe,
            
            0x00 => SvmExitCode::Bqo,
            0x03 => SvmExitCode::Bqp,
            0x04 => SvmExitCode::Bqq,
            
            0x10 => SvmExitCode::Bwx,
            0x13 => SvmExitCode::Bwy,
            0x14 => SvmExitCode::Bwz,
            
            _ if aj >= 0x40 && aj <= 0x5F => unsafe { 
                core::mem::transmute::<u64, SvmExitCode>(aj) 
            },
            _ => SvmExitCode::Cgb,
        }
    }
}


#[repr(C, align(4096))]
pub struct HostSaveArea {
    asi: [u8; 4096],
}

impl HostSaveArea {
    pub const fn new() -> Self {
        Self { asi: [0; 4096] }
    }
}


pub fn gkj() -> bool {
    
    let gdr: u32;
    let fyb: u32;
    let fyc: u32;
    let fyd: u32;
    
    unsafe {
        core::arch::asm!(
            "push rbx",
            "cpuid",
            "mov {0:e}, ebx",
            "pop rbx",
            bd(reg) fyb,
            inout("eax") 0u32 => _,
            lateout("ecx") fyc,
            lateout("edx") fyd,
            options(nostack, preserves_flags)
        );
    }
    
    
    
    let jba = fyb == 0x6874_7541 
        && fyd == 0x6974_6E65 
        && fyc == 0x444D_4163;
    
    
    crate::serial_println!("[SVM] CPUID 0: EBX=0x{:08X} EDX=0x{:08X} ECX=0x{:08X}", 
        fyb, fyd, fyc);
    
    if !jba {
        crate::serial_println!("[SVM] Not an AMD processor, is_amd={}", jba);
        return false;
    }
    
    
    unsafe {
        core::arch::asm!(
            "push rbx",
            "cpuid",
            "pop rbx",
            inout("eax") 0x8000_0001u32 => _,
            lateout("ecx") gdr,
            lateout("edx") _,
            options(nostack, preserves_flags)
        );
    }
    
    crate::serial_println!("[SVM] CPUID 0x80000001 ECX = 0x{:08X}", gdr);
    
    let wwe = (gdr & (1 << 2)) != 0;
    
    if !wwe {
        crate::serial_println!("[SVM] SVM not supported by CPU (bit 2 not set)");
        return false;
    }
    
    
    let vm_cr = bcg(msr::DAY_);
    if (vm_cr & vm_cr::CUR_) != 0 && (vm_cr & vm_cr::BGP_) != 0 {
        crate::serial_println!("[SVM] SVM disabled and locked by BIOS");
        return false;
    }
    
    true
}


pub fn fjn() -> SvmFeatures {
    let mut features = SvmFeatures::default();
    
    
    let eax: u32;
    let ebx: u32;
    let edx: u32;
    
    unsafe {
        core::arch::asm!(
            "push rbx",
            "cpuid",
            "mov {0:e}, ebx",
            "pop rbx",
            bd(reg) ebx,
            inout("eax") 0x8000_000Au32 => eax,
            lateout("ecx") _,
            lateout("edx") edx,
            options(nostack, preserves_flags)
        );
    }
    
    features.afe = (eax & 0xFF) as u8;
    features.fph = ebx;
    features.npt = (edx & (1 << 0)) != 0;
    features.lid = (edx & (1 << 1)) != 0;
    features.mig = (edx & (1 << 2)) != 0;
    features.evl = (edx & (1 << 3)) != 0;
    features.mnj = (edx & (1 << 4)) != 0;
    features.mpr = (edx & (1 << 5)) != 0;
    features.hjy = (edx & (1 << 6)) != 0;
    features.iqs = (edx & (1 << 7)) != 0;
    features.ltc = (edx & (1 << 10)) != 0;
    features.ltd = (edx & (1 << 12)) != 0;
    features.gzk = (edx & (1 << 13)) != 0;
    features.mpt = (edx & (1 << 15)) != 0;
    features.mpf = (edx & (1 << 16)) != 0;
    
    features
}


#[derive(Debug, Default)]
pub struct SvmFeatures {
    pub afe: u8,
    pub fph: u32,
    pub npt: bool,           
    pub lid: bool,      
    pub mig: bool,      
    pub evl: bool,     
    pub mnj: bool,  
    pub mpr: bool,    
    pub hjy: bool, 
    pub iqs: bool,
    pub ltc: bool,  
    pub ltd: bool,
    pub gzk: bool,          
    pub mpt: bool,   
    pub mpf: bool,          
}


pub fn init() -> Result<(), &'static str> {
    if !gkj() {
        return Err("SVM not supported");
    }
    
    if AIM_.load(Ordering::SeqCst) {
        return Ok(());
    }
    
    let features = fjn();
    crate::serial_println!("[SVM] Revision: {}, ASIDs: {}", features.afe, features.fph);
    crate::serial_println!("[SVM] NPT: {}, NRIP: {}, AVIC: {}", 
        features.npt, features.evl, features.gzk);
    
    
    let efer = bcg(msr::Lh);
    if (efer & efer::Bsd) == 0 {
        fbs(msr::Lh, efer | efer::Bsd);
        crate::serial_println!("[SVM] Enabled SVME in EFER");
    }
    
    
    unsafe {
        let wcu = Box::new(HostSaveArea::new());
        let pfk = Box::lfi(wcu);
        let ki = pfk as u64 - crate::memory::lr();
        
        fbs(msr::DBH_, ki);
        CAD_ = Some(Box::nwh(pfk));
        
        crate::serial_println!("[SVM] Host save area at phys {:#x}", ki);
    }
    
    AIM_.store(true, Ordering::SeqCst);
    crate::log!("[SVM] AMD Secure Virtual Machine initialized");
    
    Ok(())
}


pub fn ky() -> bool {
    AIM_.load(Ordering::SeqCst)
}





pub unsafe fn zvq(ekr: u64) {
    core::arch::asm!(
        
        "push rbx",
        "push rcx",
        "push rdx",
        "push rsi",
        "push rdi",
        "push rbp",
        "push r8",
        "push r9",
        "push r10",
        "push r11",
        "push r12",
        "push r13",
        "push r14",
        "push r15",
        
        
        "mov rax, {vmcb}",
        
        
        "vmrun rax",
        
        
        "pop r15",
        "pop r14",
        "pop r13",
        "pop r12",
        "pop r11",
        "pop r10",
        "pop r9",
        "pop r8",
        "pop rbp",
        "pop rdi",
        "pop rsi",
        "pop rdx",
        "pop rcx",
        "pop rbx",
        
        vmcb = in(reg) ekr,
        options(nostack)
    );
}



#[repr(C, align(16))]
pub struct Ban {
    pub rax: u64,
    pub rbx: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub rbp: u64,
    pub r8: u64,
    pub r9: u64,
    pub r10: u64,
    pub r11: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
}





pub unsafe fn xsn(ekr: u64, regs: *mut Ban) {
    core::arch::asm!(
        
        "push rbx",
        "push r12",
        "push r13",
        "push r14",
        "push r15",
        "push rbp",
        
        
        "push r15",
        
        "push {vmcb}",
        
        
        
        
        "mov rax, [r15]",       
        "mov rbx, [r15 + 8]",   
        "mov rcx, [r15 + 16]",  
        "mov rdx, [r15 + 24]",  
        "mov rsi, [r15 + 32]",  
        "mov rdi, [r15 + 40]",  
        "mov rbp, [r15 + 48]",  
        "mov r8,  [r15 + 56]",  
        "mov r9,  [r15 + 64]",  
        "mov r10, [r15 + 72]",  
        "mov r11, [r15 + 80]",  
        "mov r12, [r15 + 88]",  
        "mov r13, [r15 + 96]",  
        "mov r14, [r15 + 104]", 
        "mov r15, [r15 + 112]", 
        
        
        
        "mov rax, [rsp]",          
        
        
        "vmrun rax",
        
        
        
        
        "xchg rax, [rsp + 8]",     
        
        "mov [rax + 8], rbx",      
        "mov [rax + 16], rcx",     
        "mov [rax + 24], rdx",     
        "mov [rax + 32], rsi",     
        "mov [rax + 40], rdi",     
        "mov [rax + 48], rbp",     
        "mov [rax + 56], r8",      
        "mov [rax + 64], r9",      
        "mov [rax + 72], r10",     
        "mov [rax + 80], r11",     
        "mov [rax + 88], r12",     
        "mov [rax + 96], r13",     
        "mov [rax + 104], r14",    
        "mov [rax + 112], r15",    
        
        
        
        
        
        
        
        "add rsp, 16",
        
        
        "pop rbp",
        "pop r15",
        "pop r14",
        "pop r13",
        "pop r12",
        "pop rbx",
        
        vmcb = in(reg) ekr,
        in("r15") regs,
        options(nostack)
    );
}


#[inline]
pub unsafe fn zvr(ekr: u64) {
    core::arch::asm!(
        "vmsave rax",
        in("rax") ekr,
        options(nostack, preserves_flags)
    );
}


#[inline]
pub unsafe fn zvo(ekr: u64) {
    core::arch::asm!(
        "vmload rax",
        in("rax") ekr,
        options(nostack, preserves_flags)
    );
}


#[inline]
pub unsafe fn wug() {
    core::arch::asm!("stgi", options(nostack, preserves_flags));
}


#[inline]
pub unsafe fn rbl() {
    core::arch::asm!("clgi", options(nostack, preserves_flags));
}


#[inline]
pub unsafe fn ofh(uy: u64, ajv: u32) {
    core::arch::asm!(
        "invlpga rax, ecx",
        in("rax") uy,
        in("ecx") ajv,
        options(nostack, preserves_flags)
    );
}


#[inline]
fn bcg(msr: u32) -> u64 {
    let ail: u32;
    let afq: u32;
    unsafe {
        core::arch::asm!(
            "rdmsr",
            in("ecx") msr,
            bd("eax") ail,
            bd("edx") afq,
            options(nostack, preserves_flags)
        );
    }
    ((afq as u64) << 32) | (ail as u64)
}


#[inline]
fn fbs(msr: u32, bn: u64) {
    let ail = bn as u32;
    let afq = (bn >> 32) as u32;
    unsafe {
        core::arch::asm!(
            "wrmsr",
            in("ecx") msr,
            in("eax") ail,
            in("edx") afq,
            options(nostack, preserves_flags)
        );
    }
}


pub fn yei() -> u32 {
    YU_.load(Ordering::SeqCst)
}


pub fn zjc() {
    YU_.fetch_add(1, Ordering::SeqCst);
}


pub fn zui() {
    YU_.fetch_sub(1, Ordering::SeqCst);
}
