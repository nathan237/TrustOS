




pub mod vmcb;
pub mod npt;

use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use alloc::boxed::Box;


static AKI_: AtomicBool = AtomicBool::new(false);


static ZZ_: AtomicU32 = AtomicU32::new(0);


static mut CDO_: Option<Box<HostSaveArea>> = None;


pub mod msr {
    pub const DEQ_: u32 = 0xC001_0114;        
    pub const DEZ_: u32 = 0xC001_0117;  
    pub const Eu: u32 = 0xC000_0080;         
    pub const BIT_: u32 = 0xC001_0118;     
}


pub mod efer {
    pub const Aem: u64 = 1 << 12;  
}


pub mod vm_cr {
    pub const CYJ_: u64 = 1 << 4;       
    pub const BIT_: u64 = 1 << 3;      
    pub const EHC_: u64 = 1 << 1;        
    pub const DNC_: u64 = 1 << 0;      
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u64)]
pub enum SvmExitCode {
    
    ReadCr0 = 0x00,
    ReadCr3 = 0x03,
    ReadCr4 = 0x04,
    WriteCr0 = 0x10,
    WriteCr3 = 0x13,
    WriteCr4 = 0x14,
    
    
    ExceptionDE = 0x40,  
    ExceptionDB = 0x41,  
    ExceptionNMI = 0x42,
    ExceptionBP = 0x43,  
    ExceptionOF = 0x44,  
    ExceptionBR = 0x45,  
    ExceptionUD = 0x46,  
    ExceptionNM = 0x47,  
    ExceptionDF = 0x48,  
    ExceptionTS = 0x4A,  
    ExceptionNP = 0x4B,  
    ExceptionSS = 0x4C,  
    ExceptionGP = 0x4D,  
    ExceptionPF = 0x4E,  
    ExceptionMF = 0x50,  
    ExceptionAC = 0x51,  
    ExceptionMC = 0x52,  
    ExceptionXF = 0x53,  
    
    
    Intr = 0x60,
    Nmi = 0x61,
    Smi = 0x62,
    Init = 0x63,
    Vintr = 0x64,
    Cr0SelWrite = 0x65,
    IdtrRead = 0x66,
    GdtrRead = 0x67,
    LdtrRead = 0x68,
    TrRead = 0x69,
    IdtrWrite = 0x6A,
    GdtrWrite = 0x6B,
    LdtrWrite = 0x6C,
    TrWrite = 0x6D,
    Rdtsc = 0x6E,
    Rdpmc = 0x6F,
    Pushf = 0x70,
    Popf = 0x71,
    Cpuid = 0x72,
    Rsm = 0x73,
    Iret = 0x74,
    Swint = 0x75,
    Invd = 0x76,
    Pause = 0x77,
    Hlt = 0x78,
    Invlpg = 0x79,
    Invlpga = 0x7A,
    IoioIn = 0x7B,
    IoioOut = 0x7C,
    MsrRead = 0x7D,
    MsrWrite = 0x7E,
    TaskSwitch = 0x7F,
    FerrFreeze = 0x80,
    Shutdown = 0x81,
    Vmrun = 0x82,
    Vmmcall = 0x83,
    Vmload = 0x84,
    Vmsave = 0x85,
    Stgi = 0x86,
    Clgi = 0x87,
    Skinit = 0x88,
    Rdtscp = 0x89,
    Icebp = 0x8A,
    Wbinvd = 0x8B,
    Monitor = 0x8C,
    Mwait = 0x8D,
    MwaitConditional = 0x8E,
    Xsetbv = 0x8F,
    
    
    NpfFault = 0x400,  
    
    
    AvicIncomplete = 0x401,
    AvicNoaccel = 0x402,
    
    
    VmgExit = 0x403,
    
    
    Invalid = 0xFFFF_FFFF_FFFF_FFFF,
}

impl From<u64> for SvmExitCode {
    fn from(code: u64) -> Self {
        match code {
            0x60 => SvmExitCode::Intr,
            0x61 => SvmExitCode::Nmi,
            0x72 => SvmExitCode::Cpuid,
            0x78 => SvmExitCode::Hlt,
            0x7B => SvmExitCode::IoioIn,
            0x7C => SvmExitCode::IoioOut,
            0x7D => SvmExitCode::MsrRead,
            0x7E => SvmExitCode::MsrWrite,
            0x7F => SvmExitCode::TaskSwitch,
            0x81 => SvmExitCode::Shutdown,
            0x83 => SvmExitCode::Vmmcall,
            0x76 => SvmExitCode::Invd,
            0x79 => SvmExitCode::Invlpg,
            0x6E => SvmExitCode::Rdtsc,
            0x89 => SvmExitCode::Rdtscp,
            0x8B => SvmExitCode::Wbinvd,
            0x8C => SvmExitCode::Monitor,
            0x8D => SvmExitCode::Mwait,
            0x8F => SvmExitCode::Xsetbv,
            0x77 => SvmExitCode::Pause,
            0x64 => SvmExitCode::Vintr,
            0x65 => SvmExitCode::Cr0SelWrite,
            0x400 => SvmExitCode::NpfFault,
            
            0x00 => SvmExitCode::ReadCr0,
            0x03 => SvmExitCode::ReadCr3,
            0x04 => SvmExitCode::ReadCr4,
            
            0x10 => SvmExitCode::WriteCr0,
            0x13 => SvmExitCode::WriteCr3,
            0x14 => SvmExitCode::WriteCr4,
            
            _ if code >= 0x40 && code <= 0x5F => unsafe { 
                core::mem::transmute::<u64, SvmExitCode>(code) 
            },
            _ => SvmExitCode::Invalid,
        }
    }
}


#[repr(C, align(4096))]
pub struct HostSaveArea {
    _reserved: [u8; 4096],
}

impl HostSaveArea {
    pub const fn new() -> Self {
        Self { _reserved: [0; 4096] }
    }
}


pub fn is_supported() -> bool {
    
    let cpuid_result: u32;
    let vendor_ebx: u32;
    let vendor_ecx: u32;
    let vendor_edx: u32;
    
    unsafe {
        core::arch::asm!(
            "push rbx",
            "cpuid",
            "mov {0:e}, ebx",
            "pop rbx",
            out(reg) vendor_ebx,
            inout("eax") 0u32 => _,
            lateout("ecx") vendor_ecx,
            lateout("edx") vendor_edx,
            options(nostack, preserves_flags)
        );
    }
    
    
    
    let erg = vendor_ebx == 0x6874_7541 
        && vendor_edx == 0x6974_6E65 
        && vendor_ecx == 0x444D_4163;
    
    
    crate::serial_println!("[SVM] CPUID 0: EBX=0x{:08X} EDX=0x{:08X} ECX=0x{:08X}", 
        vendor_ebx, vendor_edx, vendor_ecx);
    
    if !erg {
        crate::serial_println!("[SVM] Not an AMD processor, is_amd={}", erg);
        return false;
    }
    
    
    unsafe {
        core::arch::asm!(
            "push rbx",
            "cpuid",
            "pop rbx",
            inout("eax") 0x8000_0001u32 => _,
            lateout("ecx") cpuid_result,
            lateout("edx") _,
            options(nostack, preserves_flags)
        );
    }
    
    crate::serial_println!("[SVM] CPUID 0x80000001 ECX = 0x{:08X}", cpuid_result);
    
    let oyv = (cpuid_result & (1 << 2)) != 0;
    
    if !oyv {
        crate::serial_println!("[SVM] SVM not supported by CPU (bit 2 not set)");
        return false;
    }
    
    
    let vm_cr = ach(msr::DEQ_);
    if (vm_cr & vm_cr::CYJ_) != 0 && (vm_cr & vm_cr::BIT_) != 0 {
        crate::serial_println!("[SVM] SVM disabled and locked by BIOS");
        return false;
    }
    
    true
}


pub fn ckb() -> SvmFeatures {
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
            out(reg) ebx,
            inout("eax") 0x8000_000Au32 => eax,
            lateout("ecx") _,
            lateout("edx") edx,
            options(nostack, preserves_flags)
        );
    }
    
    features.revision = (eax & 0xFF) as u8;
    features.num_asids = ebx;
    features.npt = (edx & (1 << 0)) != 0;
    features.lbr_virt = (edx & (1 << 1)) != 0;
    features.svm_lock = (edx & (1 << 2)) != 0;
    features.nrip_save = (edx & (1 << 3)) != 0;
    features.tsc_rate_msr = (edx & (1 << 4)) != 0;
    features.vmcb_clean = (edx & (1 << 5)) != 0;
    features.flush_by_asid = (edx & (1 << 6)) != 0;
    features.decode_assists = (edx & (1 << 7)) != 0;
    features.pause_filter = (edx & (1 << 10)) != 0;
    features.pause_filter_thresh = (edx & (1 << 12)) != 0;
    features.avic = (edx & (1 << 13)) != 0;
    features.vmsave_virt = (edx & (1 << 15)) != 0;
    features.vgif = (edx & (1 << 16)) != 0;
    
    features
}


#[derive(Debug, Default)]
pub struct SvmFeatures {
    pub revision: u8,
    pub num_asids: u32,
    pub npt: bool,           
    pub lbr_virt: bool,      
    pub svm_lock: bool,      
    pub nrip_save: bool,     
    pub tsc_rate_msr: bool,  
    pub vmcb_clean: bool,    
    pub flush_by_asid: bool, 
    pub decode_assists: bool,
    pub pause_filter: bool,  
    pub pause_filter_thresh: bool,
    pub avic: bool,          
    pub vmsave_virt: bool,   
    pub vgif: bool,          
}


pub fn init() -> Result<(), &'static str> {
    if !is_supported() {
        return Err("SVM not supported");
    }
    
    if AKI_.load(Ordering::SeqCst) {
        return Ok(());
    }
    
    let features = ckb();
    crate::serial_println!("[SVM] Revision: {}, ASIDs: {}", features.revision, features.num_asids);
    crate::serial_println!("[SVM] NPT: {}, NRIP: {}, AVIC: {}", 
        features.npt, features.nrip_save, features.avic);
    
    
    let efer = ach(msr::Eu);
    if (efer & efer::Aem) == 0 {
        cfm(msr::Eu, efer | efer::Aem);
        crate::serial_println!("[SVM] Enabled SVME in EFER");
    }
    
    
    unsafe {
        let okf = Box::new(HostSaveArea::new());
        let jcn = Box::into_raw(okf);
        let phys_addr = jcn as u64 - crate::memory::hhdm_offset();
        
        cfm(msr::DEZ_, phys_addr);
        CDO_ = Some(Box::from_raw(jcn));
        
        crate::serial_println!("[SVM] Host save area at phys {:#x}", phys_addr);
    }
    
    AKI_.store(true, Ordering::SeqCst);
    crate::log!("[SVM] AMD Secure Virtual Machine initialized");
    
    Ok(())
}


pub fn is_initialized() -> bool {
    AKI_.load(Ordering::SeqCst)
}





pub unsafe fn rcf(vmcb_phys: u64) {
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
        
        vmcb = in(reg) vmcb_phys,
        options(nostack)
    );
}



#[repr(C, align(16))]
pub struct Vv {
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





pub unsafe fn psu(vmcb_phys: u64, regs: *mut Vv) {
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
        
        vmcb = in(reg) vmcb_phys,
        in("r15") regs,
        options(nostack)
    );
}


#[inline]
pub unsafe fn rcg(vmcb_phys: u64) {
    core::arch::asm!(
        "vmsave rax",
        in("rax") vmcb_phys,
        options(nostack, preserves_flags)
    );
}


#[inline]
pub unsafe fn rcd(vmcb_phys: u64) {
    core::arch::asm!(
        "vmload rax",
        in("rax") vmcb_phys,
        options(nostack, preserves_flags)
    );
}


#[inline]
pub unsafe fn oxh() {
    core::arch::asm!("stgi", options(nostack, preserves_flags));
}


#[inline]
pub unsafe fn kky() {
    core::arch::asm!("clgi", options(nostack, preserves_flags));
}


#[inline]
pub unsafe fn ihj(vaddr: u64, asid: u32) {
    core::arch::asm!(
        "invlpga rax, ecx",
        in("rax") vaddr,
        in("ecx") asid,
        options(nostack, preserves_flags)
    );
}


#[inline]
fn ach(msr: u32) -> u64 {
    let low: u32;
    let high: u32;
    unsafe {
        core::arch::asm!(
            "rdmsr",
            in("ecx") msr,
            out("eax") low,
            out("edx") high,
            options(nostack, preserves_flags)
        );
    }
    ((high as u64) << 32) | (low as u64)
}


#[inline]
fn cfm(msr: u32, value: u64) {
    let low = value as u32;
    let high = (value >> 32) as u32;
    unsafe {
        core::arch::asm!(
            "wrmsr",
            in("ecx") msr,
            in("eax") low,
            in("edx") high,
            options(nostack, preserves_flags)
        );
    }
}


pub fn pxr() -> u32 {
    ZZ_.load(Ordering::SeqCst)
}


pub fn qtn() {
    ZZ_.fetch_add(1, Ordering::SeqCst);
}


pub fn rbl() {
    ZZ_.fetch_sub(1, Ordering::SeqCst);
}
