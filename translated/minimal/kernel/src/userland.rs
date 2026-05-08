







use crate::gdt::{KERNEL_CODE_SELECTOR, KERNEL_DATA_SELECTOR, ALF_, ALG_};
use x86_64::registers::model_specific::{Efer, EferFlags, LStar, SFMask, Star};
use x86_64::registers::rflags::RFlags;
use x86_64::VirtAddr;


pub const DDV_: u64 = 0x0000_7FFF_FFFF_0000;

pub const ALI_: usize = 1024 * 1024;

pub const DDT_: u64 = 0x0000_0000_0040_0000;





pub fn init() {
    unsafe {
        
        let efer = Efer::read();
        Efer::write(efer | EferFlags::SYSTEM_CALL_EXTENSIONS);
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        let star_value: u64 = (0x10u64 << 48) | (0x08u64 << 32);
        core::arch::asm!(
            "wrmsr",
            in("ecx") 0xC0000081u32, 
            in("eax") star_value as u32,
            in("edx") (star_value >> 32) as u32,
        );
        
        
        LStar::write(VirtAddr::new(jky as *const () as u64));
        
        
        
        SFMask::write(
            RFlags::INTERRUPT_FLAG | 
            RFlags::DIRECTION_FLAG | 
            RFlags::TRAP_FLAG |
            RFlags::ALIGNMENT_CHECK
        );
    }
    
    crate::log!("[USERLAND] SYSCALL/SYSRET configured (STAR, LSTAR, SFMASK)");
}
















#[inline(never)]
pub unsafe fn mvh(entry_point: u64, user_stack: u64) -> ! {
    
    
    const USER_CS: u64 = 0x20 | 3; 
    const USER_SS: u64 = 0x18 | 3; 
    
    
    const USER_RFLAGS: u64 = 0x202; 
    
    crate::log_debug!("[USERLAND] Jumping to Ring 3: RIP={:#x}, RSP={:#x}", entry_point, user_stack);
    
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
        
        ss = in(reg) USER_SS,
        rsp = in(reg) user_stack,
        rflags = in(reg) USER_RFLAGS,
        cs = in(reg) USER_CS,
        rip = in(reg) entry_point,
        options(noreturn)
    );
}


#[inline(never)]
pub unsafe fn jump_to_ring3_with_args(entry_point: u64, user_stack: u64, arg1: u64, arg2: u64) -> ! {
    const USER_CS: u64 = 0x20 | 3;
    const USER_SS: u64 = 0x18 | 3;
    const USER_RFLAGS: u64 = 0x202;
    
    crate::log_debug!("[USERLAND] Jumping to Ring 3: RIP={:#x}, RSP={:#x}, args=({}, {:#x})", 
        entry_point, user_stack, arg1, arg2);
    
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
        
        ss = in(reg) USER_SS,
        rsp = in(reg) user_stack,
        rflags = in(reg) USER_RFLAGS,
        cs = in(reg) USER_CS,
        rip = in(reg) entry_point,
        arg1 = in(reg) arg1,
        arg2 = in(reg) arg2,
        options(noreturn)
    );
}















#[unsafe(naked)]
extern "C" fn jky() {
    core::arch::naked_asm!(
        
        
        
        
        
        
        
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
        
        kernel_stack = sym KERNEL_SYSCALL_STACK_TOP,
        user_rsp_temp = sym USER_RSP_TEMP,
        user_return_rip = sym USER_RETURN_RIP,
        user_return_rflags = sym USER_RETURN_RFLAGS,
        signal_signo = sym SIGNAL_DELIVER_SIGNO,
        handler = sym crate::interrupts::syscall::syscall_handler_rust,
    );
}


#[no_mangle]
pub static mut USER_RSP_TEMP: u64 = 0;


#[no_mangle]
pub static mut USER_RETURN_RIP: u64 = 0;


#[no_mangle]
pub static mut USER_RETURN_RFLAGS: u64 = 0;



#[no_mangle]
pub static mut SIGNAL_DELIVER_SIGNO: u64 = 0;


static mut CGJ_: [u8; 65536] = [0; 65536]; 

#[no_mangle]
pub static mut KERNEL_SYSCALL_STACK_TOP: u64 = 0;


pub fn igu() {
    unsafe {
        let bvx = CGJ_.as_ptr() as u64;
        KERNEL_SYSCALL_STACK_TOP = bvx + 65536;
        crate::log_debug!("[USERLAND] Syscall stack at {:#x}", KERNEL_SYSCALL_STACK_TOP);
    }
}









static mut KERNEL_RETURN_RSP: u64 = 0;

static mut KERNEL_RETURN_RIP: u64 = 0;

static mut RH_: bool = false;















#[inline(never)]
pub unsafe fn bzn(entry_point: u64, user_stack: u64) -> i32 {
    const USER_CS: u64 = 0x20 | 3;  
    const USER_SS: u64 = 0x18 | 3;  
    const USER_RFLAGS: u64 = 0x202; 

    let exit_code: i64;
    RH_ = true;

    
    
    let entry_point = core::hint::black_box(entry_point);
    let user_stack = core::hint::black_box(user_stack);

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

        

        entry = in(reg) entry_point,
        user_rsp = in(reg) user_stack,
        ss = in(reg) USER_SS,
        cs = in(reg) USER_CS,
        rflags = in(reg) USER_RFLAGS,
        return_rsp = sym KERNEL_RETURN_RSP,
        return_rip = sym KERNEL_RETURN_RIP,
        
        out("rax") exit_code,
        
        lateout("rcx") _,
        lateout("rdx") _,
        lateout("rsi") _,
        lateout("rdi") _,
        lateout("r8") _,
        lateout("r9") _,
        lateout("r10") _,
        lateout("r11") _,
    );

    RH_ = false;
    exit_code as i32
}








pub unsafe fn azi(exit_code: i32) -> ! {
    
    core::arch::asm!(
        "mov cr3, {cr3}",
        cr3 = in(reg) crate::memory::paging::kernel_cr3(),
        options(nostack, preserves_flags)
    );

    
    core::arch::asm!(
        "mov rax, {code}",
        "mov rsp, [{return_rsp}]",
        "sti",          
        "jmp [{return_rip}]",
        code = in(reg) exit_code as i64,
        return_rsp = sym KERNEL_RETURN_RSP,
        return_rip = sym KERNEL_RETURN_RIP,
        options(noreturn)
    );
}


pub fn ers() -> bool {
    unsafe { RH_ }
}





use core::sync::atomic::{AtomicI32, AtomicU64, Ordering};


static ALH_: AtomicI32 = AtomicI32::new(0);

pub static MH_: AtomicU64 = AtomicU64::new(0);













pub unsafe fn mxd(entry_point: u64, user_stack: u64) -> i32 {
    let pid = crate::process::pe();
    let fko = crate::thread::current_tid();
    
    
    MH_.store(fko, Ordering::SeqCst);
    ALH_.store(0, Ordering::SeqCst);
    RH_ = true;
    
    
    
    
    
    let jsv = crate::thread::jhc(pid, "user_main", entry_point, user_stack, 0);
    
    crate::log_debug!("[USERLAND] Launched user thread TID={:#x} for PID {}, waiting on TID={:#x}", 
        jsv, pid, fko);
    
    
    crate::thread::block(fko);
    crate::thread::ajc();
    
    
    RH_ = false;
    
    let code = ALH_.load(Ordering::SeqCst);
    crate::log_debug!("[USERLAND] User process exited with code {}", code);
    code
}




pub fn haz(exit_code: i32) {
    ALH_.store(exit_code, Ordering::SeqCst);
    
    let avt = MH_.load(Ordering::SeqCst);
    if avt != 0 {
        MH_.store(0, Ordering::SeqCst);
        crate::thread::wake(avt);
    }
    
    
    unsafe {
        core::arch::asm!(
            "mov cr3, {cr3}",
            cr3 = in(reg) crate::memory::paging::kernel_cr3(),
            options(nostack, preserves_flags)
        );
    }
    
    
    crate::thread::exit(exit_code);
}