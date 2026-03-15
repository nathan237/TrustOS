







use crate::gdt::{NQ_, NR_, AJK_, AJL_};
use x86_64::registers::model_specific::{Efer, EferFlags, LStar, SFMask, And};
use x86_64::registers::rflags::RFlags;
use x86_64::VirtAddr;


pub const DAD_: u64 = 0x0000_7FFF_FFFF_0000;

pub const AJN_: usize = 1024 * 1024;

pub const DAB_: u64 = 0x0000_0000_0040_0000;





pub fn init() {
    unsafe {
        
        let efer = Efer::read();
        Efer::write(efer | EferFlags::EHE_);
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        let pnx: u64 = (0x10u64 << 48) | (0x08u64 << 32);
        core::arch::asm!(
            "wrmsr",
            in("ecx") 0xC0000081u32, 
            in("eax") pnx as u32,
            in("edx") (pnx >> 32) as u32,
        );
        
        
        LStar::write(VirtAddr::new(prb as *const () as u64));
        
        
        
        SFMask::write(
            RFlags::DRF_ | 
            RFlags::DJN_ | 
            RFlags::EIQ_ |
            RFlags::DCM_
        );
    }
    
    crate::log!("[USERLAND] SYSCALL/SYSRET configured (STAR, LSTAR, SFMASK)");
}
















#[inline(never)]
pub unsafe fn uau(mi: u64, ais: u64) -> ! {
    
    
    const QL_: u64 = 0x20 | 3; 
    const QN_: u64 = 0x18 | 3; 
    
    
    const QM_: u64 = 0x202; 
    
    crate::log_debug!("[USERLAND] Jumping to Ring 3: RIP={:#x}, RSP={:#x}", mi, ais);
    
    core::arch::asm!(
        
        "push {ss}",        
        "push {rsp}",       
        "push {rflags}",    
        "push {cs}",        
        "push {rip}",       
        
        
        "xor rax, rax",
        "xor rbx, rbx",
        "xor rcx, rcx",
        "xor rdx, rdx",
        "xor rsi, rsi",
        "xor rdi, rdi",
        "xor rbp, rbp",
        "xor r8, r8",
        "xor r9, r9",
        "xor r10, r10",
        "xor r11, r11",
        "xor r12, r12",
        "xor r13, r13",
        "xor r14, r14",
        "xor r15, r15",
        
        
        "iretq",
        
        rv = in(reg) QN_,
        rsp = in(reg) ais,
        rflags = in(reg) QM_,
        aap = in(reg) QL_,
        pc = in(reg) mi,
        options(jhe)
    );
}


#[inline(never)]
pub unsafe fn ohk(mi: u64, ais: u64, aai: u64, agf: u64) -> ! {
    const QL_: u64 = 0x20 | 3;
    const QN_: u64 = 0x18 | 3;
    const QM_: u64 = 0x202;
    
    crate::log_debug!("[USERLAND] Jumping to Ring 3: RIP={:#x}, RSP={:#x}, args=({}, {:#x})", 
        mi, ais, aai, agf);
    
    core::arch::asm!(
        
        "push {ss}",
        "push {rsp}",
        "push {rflags}",
        "push {cs}",
        "push {rip}",
        
        
        "mov rdi, {arg1}",
        "mov rsi, {arg2}",
        
        
        "xor rax, rax",
        "xor rbx, rbx",
        "xor rcx, rcx",
        "xor rdx, rdx",
        "xor rbp, rbp",
        "xor r8, r8",
        "xor r9, r9",
        "xor r10, r10",
        "xor r11, r11",
        "xor r12, r12",
        "xor r13, r13",
        "xor r14, r14",
        "xor r15, r15",
        
        "iretq",
        
        rv = in(reg) QN_,
        rsp = in(reg) ais,
        rflags = in(reg) QM_,
        aap = in(reg) QL_,
        pc = in(reg) mi,
        aai = in(reg) aai,
        agf = in(reg) agf,
        options(jhe)
    );
}















#[unsafe(evb)]
extern "C" fn prb() {
    core::arch::evc!(
        
        
        
        
        
        
        
        "mov [rip + {user_rsp_temp}], rsp",
        "mov [rip + {user_return_rip}], rcx",
        "mov [rip + {user_return_rflags}], r11",
        
        
        "mov QWORD PTR [rip + {signal_signo}], 0",
        
        
        "mov rsp, [rip + {kernel_stack}]",
        
        
        "push rbx",
        "push rbp",
        "push r12",
        "push r13",
        "push r14",
        "push r15",
        
        
        
        
        
        
        "push r9",
        
        
        "mov r15, r8",       
        "mov r12, rdi",      
        "mov r13, rsi",      
        "mov r14, rdx",      
        
        "mov rdi, rax",      
        "mov rsi, r12",      
        "mov rdx, r13",      
        "mov rcx, r14",      
        "mov r8,  r10",      
        "mov r9,  r15",      
        
        
        "call {handler}",
        
        
        "add rsp, 8",
        
        
        
        
        "pop r15",
        "pop r14",
        "pop r13",
        "pop r12",
        "pop rbp",
        "pop rbx",
        
        
        "mov r11, [rip + {user_return_rflags}]",
        "mov rcx, [rip + {user_return_rip}]",
        
        
        "mov rdi, [rip + {signal_signo}]",
        
        
        
        
        "mov rsp, [rip + {user_rsp_temp}]",
        
        
        "sysretq",
        
        bhg = aaw NT_,
        zva = aaw YD_,
        zuz = aaw YC_,
        zuy = aaw YB_,
        zoj = aaw AHX_,
        cfd = aaw crate::interrupts::syscall::prc,
    );
}


#[no_mangle]
pub static mut YD_: u64 = 0;


#[no_mangle]
pub static mut YC_: u64 = 0;


#[no_mangle]
pub static mut YB_: u64 = 0;



#[no_mangle]
pub static mut AHX_: u64 = 0;


static mut CDA_: [u8; 65536] = [0; 65536]; 

#[no_mangle]
pub static mut NT_: u64 = 0;


pub fn oen() {
    unsafe {
        let eiy = CDA_.fq() as u64;
        NT_ = eiy + 65536;
        crate::log_debug!("[USERLAND] Syscall stack at {:#x}", NT_);
    }
}









static mut AXT_: u64 = 0;

static mut AXS_: u64 = 0;

static mut QK_: bool = false;















#[inline(never)]
pub unsafe fn eqa(mi: u64, ais: u64) -> i32 {
    const QL_: u64 = 0x20 | 3;  
    const QN_: u64 = 0x18 | 3;  
    const QM_: u64 = 0x202; 

    let nz: i64;
    QK_ = true;

    
    
    let mi = core::hint::mzg(mi);
    let ais = core::hint::mzg(ais);

    core::arch::asm!(
        
        "lea rax, [rip + 2f]",
        "mov [{return_rip}], rax",

        
        "push rbx",
        "push rbp",
        "push r12",
        "push r13",
        "push r14",
        "push r15",

        
        "mov [{return_rsp}], rsp",

        
        "push {ss}",
        "push {user_rsp}",
        "push {rflags}",
        "push {cs}",
        "push {entry}",

        
        "xor rax, rax",
        "xor rbx, rbx",
        "xor rcx, rcx",
        "xor rdx, rdx",
        "xor rsi, rsi",
        "xor rdi, rdi",
        "xor rbp, rbp",
        "xor r8, r8",
        "xor r9, r9",
        "xor r10, r10",
        "xor r11, r11",
        "xor r12, r12",
        "xor r13, r13",
        "xor r14, r14",
        "xor r15, r15",

        
        "iretq",

        
        
        
        
        "2:",

        
        "pop r15",
        "pop r14",
        "pop r13",
        "pop r12",
        "pop rbp",
        "pop rbx",

        

        bt = in(reg) mi,
        dxg = in(reg) ais,
        rv = in(reg) QN_,
        aap = in(reg) QL_,
        rflags = in(reg) QM_,
        vyl = aaw AXT_,
        vyk = aaw AXS_,
        
        bd("rax") nz,
        
        lateout("rcx") _,
        lateout("rdx") _,
        lateout("rsi") _,
        lateout("rdi") _,
        lateout("r8") _,
        lateout("r9") _,
        lateout("r10") _,
        lateout("r11") _,
    );

    QK_ = false;
    nz as i32
}








pub unsafe fn ctw(nz: i32) -> ! {
    
    core::arch::asm!(
        "mov cr3, {cr3}",
        jm = in(reg) crate::memory::paging::ade(),
        options(nostack, preserves_flags)
    );

    
    core::arch::asm!(
        "mov rax, {code}",
        "mov rsp, [{return_rsp}]",
        "sti",          
        "jmp [{return_rip}]",
        aj = in(reg) nz as i64,
        vyl = aaw AXT_,
        vyk = aaw AXS_,
        options(jhe)
    );
}


pub fn jbp() -> bool {
    unsafe { QK_ }
}





use core::sync::atomic::{AtomicI32, AtomicU64, Ordering};


static AJM_: AtomicI32 = AtomicI32::new(0);

pub static LN_: AtomicU64 = AtomicU64::new(0);













pub unsafe fn udc(mi: u64, ais: u64) -> i32 {
    let ce = crate::process::aei();
    let kge = crate::thread::bqd();
    
    
    LN_.store(kge, Ordering::SeqCst);
    AJM_.store(0, Ordering::SeqCst);
    QK_ = true;
    
    
    
    
    
    let qdx = crate::thread::pme(ce, "user_main", mi, ais, 0);
    
    crate::log_debug!("[USERLAND] Launched user thread TID={:#x} for PID {}, waiting on TID={:#x}", 
        qdx, ce, kge);
    
    
    crate::thread::block(kge);
    crate::thread::cix();
    
    
    QK_ = false;
    
    let aj = AJM_.load(Ordering::SeqCst);
    crate::log_debug!("[USERLAND] User process exited with code {}", aj);
    aj
}




pub fn mop(nz: i32) {
    AJM_.store(nz, Ordering::SeqCst);
    
    let cnx = LN_.load(Ordering::SeqCst);
    if cnx != 0 {
        LN_.store(0, Ordering::SeqCst);
        crate::thread::wake(cnx);
    }
    
    
    unsafe {
        core::arch::asm!(
            "mov cr3, {cr3}",
            jm = in(reg) crate::memory::paging::ade(),
            options(nostack, preserves_flags)
        );
    }
    
    
    crate::thread::cxn(nz);
}