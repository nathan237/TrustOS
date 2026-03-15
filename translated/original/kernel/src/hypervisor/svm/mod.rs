//! AMD SVM (Secure Virtual Machine) Support
//!
//! AMD-V implementation for TrustVM hypervisor.
//! Uses VMCB (Virtual Machine Control Block) and NPT (Nested Page Tables).

pub mod vmcb;
pub mod npt;

use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use alloc::boxed::Box;

/// SVM initialized flag
static SVM_INITIALIZED: AtomicBool = AtomicBool::new(false);

/// Number of active VMs using SVM
static ACTIVE_VMS: AtomicU32 = AtomicU32::new(0);

/// Host save area physical address (per-CPU in real implementation)
static mut HOST_SAVE_AREA: Option<Box<HostSaveArea>> = None;

/// AMD SVM MSRs
pub mod msr {
    pub const VM_CR: u32 = 0xC001_0114;        // VM Control Register
    pub const VM_HSAVE_PA: u32 = 0xC001_0117;  // Host Save Area Physical Address
    pub const EFER: u32 = 0xC000_0080;         // Extended Feature Enable Register
    pub const SVM_LOCK: u32 = 0xC001_0118;     // SVM Lock Key
}

/// EFER bits
pub mod efer {
    pub const SVME: u64 = 1 << 12;  // SVM Enable
}

/// VM_CR bits
pub mod vm_cr {
    pub const SVM_DIS: u64 = 1 << 4;       // SVM Disable
    pub const SVM_LOCK: u64 = 1 << 3;      // SVM Lock
    pub const R_INIT: u64 = 1 << 1;        // Redirect INIT
    pub const DIS_A20M: u64 = 1 << 0;      // Disable A20 Masking
}

/// SVM Exit Codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u64)]
pub enum SvmExitCode {
    // Intercepts
    ReadCr0 = 0x00,
    ReadCr3 = 0x03,
    ReadCr4 = 0x04,
    WriteCr0 = 0x10,
    WriteCr3 = 0x13,
    WriteCr4 = 0x14,
    
    // Exception intercepts (0x40-0x5F)
    ExceptionDE = 0x40,  // Divide Error
    ExceptionDB = 0x41,  // Debug
    ExceptionNMI = 0x42,
    ExceptionBP = 0x43,  // Breakpoint
    ExceptionOF = 0x44,  // Overflow
    ExceptionBR = 0x45,  // BOUND Range
    ExceptionUD = 0x46,  // Invalid Opcode
    ExceptionNM = 0x47,  // Device Not Available
    ExceptionDF = 0x48,  // Double Fault
    ExceptionTS = 0x4A,  // Invalid TSS
    ExceptionNP = 0x4B,  // Segment Not Present
    ExceptionSS = 0x4C,  // Stack Segment
    ExceptionGP = 0x4D,  // General Protection
    ExceptionPF = 0x4E,  // Page Fault
    ExceptionMF = 0x50,  // x87 FP Exception
    ExceptionAC = 0x51,  // Alignment Check
    ExceptionMC = 0x52,  // Machine Check
    ExceptionXF = 0x53,  // SIMD FP Exception
    
    // Special intercepts
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
    
    // Nested paging
    NpfFault = 0x400,  // Nested Page Fault
    
    // AVIC (Advanced Virtual Interrupt Controller)
    AvicIncomplete = 0x401,
    AvicNoaccel = 0x402,
    
    // Misc
    VmgExit = 0x403,
    
    // Invalid
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
            // CR read intercepts (0x00-0x0F)
            0x00 => SvmExitCode::ReadCr0,
            0x03 => SvmExitCode::ReadCr3,
            0x04 => SvmExitCode::ReadCr4,
            // CR write intercepts (0x10-0x1F)
            0x10 => SvmExitCode::WriteCr0,
            0x13 => SvmExitCode::WriteCr3,
            0x14 => SvmExitCode::WriteCr4,
            // Exception intercepts
            _ if code >= 0x40 && code <= 0x5F => unsafe { 
                core::mem::transmute::<u64, SvmExitCode>(code) 
            },
            _ => SvmExitCode::Invalid,
        }
    }
}

/// Host save area (4KB aligned)
#[repr(C, align(4096))]
pub struct HostSaveArea {
    _reserved: [u8; 4096],
}

impl HostSaveArea {
    pub const fn new() -> Self {
        Self { _reserved: [0; 4096] }
    }
}

/// Check if SVM is supported
pub fn is_supported() -> bool {
    // Check CPUID for AMD processor with SVM
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
    
    // Check for "AuthenticAMD"
    // EBX = "Auth", EDX = "enti", ECX = "cAMD"
    let is_amd = vendor_ebx == 0x6874_7541 
        && vendor_edx == 0x6974_6E65 
        && vendor_ecx == 0x444D_4163;
    
    // Debug: print vendor string
    crate::serial_println!("[SVM] CPUID 0: EBX=0x{:08X} EDX=0x{:08X} ECX=0x{:08X}", 
        vendor_ebx, vendor_edx, vendor_ecx);
    
    if !is_amd {
        crate::serial_println!("[SVM] Not an AMD processor, is_amd={}", is_amd);
        return false;
    }
    
    // Check CPUID.80000001h:ECX.SVM[bit 2]
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
    
    let svm_available = (cpuid_result & (1 << 2)) != 0;
    
    if !svm_available {
        crate::serial_println!("[SVM] SVM not supported by CPU (bit 2 not set)");
        return false;
    }
    
    // Check if SVM is disabled in BIOS (VM_CR.SVMDIS)
    let vm_cr = read_msr(msr::VM_CR);
    if (vm_cr & vm_cr::SVM_DIS) != 0 && (vm_cr & vm_cr::SVM_LOCK) != 0 {
        crate::serial_println!("[SVM] SVM disabled and locked by BIOS");
        return false;
    }
    
    true
}

/// Check SVM features
pub fn get_features() -> SvmFeatures {
    let mut features = SvmFeatures::default();
    
    // CPUID Fn8000_000A
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

/// SVM CPU features
#[derive(Debug, Default)]
pub struct SvmFeatures {
    pub revision: u8,
    pub num_asids: u32,
    pub npt: bool,           // Nested Page Tables
    pub lbr_virt: bool,      // LBR Virtualization
    pub svm_lock: bool,      // SVM Lock
    pub nrip_save: bool,     // Next RIP Save
    pub tsc_rate_msr: bool,  // TSC Rate MSR
    pub vmcb_clean: bool,    // VMCB Clean Bits
    pub flush_by_asid: bool, // TLB Flush by ASID
    pub decode_assists: bool,// Decode Assists
    pub pause_filter: bool,  // Pause Filter
    pub pause_filter_thresh: bool,
    pub avic: bool,          // Advanced Virtual Interrupt Controller
    pub vmsave_virt: bool,   // VMSAVE/VMLOAD Virtualization
    pub vgif: bool,          // Virtual GIF
}

/// Initialize SVM
pub fn init() -> Result<(), &'static str> {
    if !is_supported() {
        return Err("SVM not supported");
    }
    
    if SVM_INITIALIZED.load(Ordering::SeqCst) {
        return Ok(());
    }
    
    let features = get_features();
    crate::serial_println!("[SVM] Revision: {}, ASIDs: {}", features.revision, features.num_asids);
    crate::serial_println!("[SVM] NPT: {}, NRIP: {}, AVIC: {}", 
        features.npt, features.nrip_save, features.avic);
    
    // Enable SVM in EFER
    let efer = read_msr(msr::EFER);
    if (efer & efer::SVME) == 0 {
        write_msr(msr::EFER, efer | efer::SVME);
        crate::serial_println!("[SVM] Enabled SVME in EFER");
    }
    
    // Allocate and set host save area
    unsafe {
        let save_area = Box::new(HostSaveArea::new());
        let save_area_ptr = Box::into_raw(save_area);
        let phys_addr = save_area_ptr as u64 - crate::memory::hhdm_offset();
        
        write_msr(msr::VM_HSAVE_PA, phys_addr);
        HOST_SAVE_AREA = Some(Box::from_raw(save_area_ptr));
        
        crate::serial_println!("[SVM] Host save area at phys {:#x}", phys_addr);
    }
    
    SVM_INITIALIZED.store(true, Ordering::SeqCst);
    crate::log!("[SVM] AMD Secure Virtual Machine initialized");
    
    Ok(())
}

/// Check if SVM is initialized
pub fn is_initialized() -> bool {
    SVM_INITIALIZED.load(Ordering::SeqCst)
}

/// Run a VM (execute VMRUN)
/// 
/// # Safety
/// VMCB must be properly configured
pub unsafe fn vmrun(vmcb_phys: u64) {
    core::arch::asm!(
        // Save host state
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
        
        // Load VMCB address into rax
        "mov rax, {vmcb}",
        
        // Execute VMRUN
        "vmrun rax",
        
        // Restore host state
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

/// Guest registers structure for VMRUN with guest state
/// Must match the layout expected by vmrun_with_regs
#[repr(C, align(16))]
pub struct VmrunGuestRegs {
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

/// Run VM with explicit guest register load/save
/// 
/// # Safety
/// VMCB must be properly configured, regs must point to valid VmrunGuestRegs
pub unsafe fn vmrun_with_regs(vmcb_phys: u64, regs: *mut VmrunGuestRegs) {
    core::arch::asm!(
        // Save host callee-saved registers
        "push rbx",
        "push r12",
        "push r13",
        "push r14",
        "push r15",
        "push rbp",
        
        // Save regs pointer (r15) for after VMEXIT
        "push r15",
        // Save VMCB address
        "push {vmcb}",
        
        // Load guest GPRs from regs structure.
        // r15 is pinned to the regs pointer — load it LAST so the
        // base address stays valid throughout the entire sequence.
        "mov rax, [r15]",       // rax = regs->rax
        "mov rbx, [r15 + 8]",   // rbx = regs->rbx
        "mov rcx, [r15 + 16]",  // rcx = regs->rcx
        "mov rdx, [r15 + 24]",  // rdx = regs->rdx
        "mov rsi, [r15 + 32]",  // rsi = regs->rsi
        "mov rdi, [r15 + 40]",  // rdi = regs->rdi
        "mov rbp, [r15 + 48]",  // rbp = regs->rbp
        "mov r8,  [r15 + 56]",  // r8  = regs->r8
        "mov r9,  [r15 + 64]",  // r9  = regs->r9
        "mov r10, [r15 + 72]",  // r10 = regs->r10
        "mov r11, [r15 + 80]",  // r11 = regs->r11
        "mov r12, [r15 + 88]",  // r12 = regs->r12
        "mov r13, [r15 + 96]",  // r13 = regs->r13
        "mov r14, [r15 + 104]", // r14 = regs->r14
        "mov r15, [r15 + 112]", // r15 = regs->r15 (pointer no longer needed)
        
        // Now load VMCB address into rax for VMRUN
        // (this overwrites guest rax, but hardware saves/restores it via VMCB)
        "mov rax, [rsp]",          // rax = vmcb_phys from stack
        
        // Execute VMRUN - enters guest, returns on #VMEXIT
        "vmrun rax",
        
        // After VMEXIT: save guest GPRs back to regs structure
        // Host rax = vmcb_phys (restored by hardware).
        // Recover the regs pointer from [rsp+8] (pushed before vmcb).
        "xchg rax, [rsp + 8]",     // rax = regs ptr, [rsp+8] = vmcb_phys
        
        "mov [rax + 8], rbx",      // regs->rbx = guest rbx
        "mov [rax + 16], rcx",     // regs->rcx = guest rcx
        "mov [rax + 24], rdx",     // regs->rdx = guest rdx
        "mov [rax + 32], rsi",     // regs->rsi = guest rsi
        "mov [rax + 40], rdi",     // regs->rdi = guest rdi
        "mov [rax + 48], rbp",     // regs->rbp = guest rbp
        "mov [rax + 56], r8",      // regs->r8  = guest r8
        "mov [rax + 64], r9",      // regs->r9  = guest r9
        "mov [rax + 72], r10",     // regs->r10 = guest r10
        "mov [rax + 80], r11",     // regs->r11 = guest r11
        "mov [rax + 88], r12",     // regs->r12 = guest r12
        "mov [rax + 96], r13",     // regs->r13 = guest r13
        "mov [rax + 104], r14",    // regs->r14 = guest r14
        "mov [rax + 112], r15",    // regs->r15 = guest r15
        
        // Guest RAX: hardware saved it in the VMCB state-save area.
        // We cannot read it here (only have physical addr). Caller
        // must read guest RAX from vmcb.read_state(RAX) instead.
        // Leave regs->rax untouched (stale value — caller overwrites it).
        
        // Clean up stack (vmcb + regs)
        "add rsp, 16",
        
        // Restore host callee-saved registers
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

/// Execute VMSAVE instruction
#[inline]
pub unsafe fn vmsave(vmcb_phys: u64) {
    core::arch::asm!(
        "vmsave rax",
        in("rax") vmcb_phys,
        options(nostack, preserves_flags)
    );
}

/// Execute VMLOAD instruction
#[inline]
pub unsafe fn vmload(vmcb_phys: u64) {
    core::arch::asm!(
        "vmload rax",
        in("rax") vmcb_phys,
        options(nostack, preserves_flags)
    );
}

/// Enable global interrupts (STGI)
#[inline]
pub unsafe fn stgi() {
    core::arch::asm!("stgi", options(nostack, preserves_flags));
}

/// Clear global interrupts (CLGI)
#[inline]
pub unsafe fn clgi() {
    core::arch::asm!("clgi", options(nostack, preserves_flags));
}

/// Invalidate TLB entries for an ASID
#[inline]
pub unsafe fn invlpga(vaddr: u64, asid: u32) {
    core::arch::asm!(
        "invlpga rax, ecx",
        in("rax") vaddr,
        in("ecx") asid,
        options(nostack, preserves_flags)
    );
}

/// Read MSR
#[inline]
fn read_msr(msr: u32) -> u64 {
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

/// Write MSR
#[inline]
fn write_msr(msr: u32, value: u64) {
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

/// Get active VM count
pub fn active_vm_count() -> u32 {
    ACTIVE_VMS.load(Ordering::SeqCst)
}

/// Increment active VM count
pub fn register_vm() {
    ACTIVE_VMS.fetch_add(1, Ordering::SeqCst);
}

/// Decrement active VM count
pub fn unregister_vm() {
    ACTIVE_VMS.fetch_sub(1, Ordering::SeqCst);
}
