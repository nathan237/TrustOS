







#[allow(unused_imports)]
use x86_64::registers::model_specific::{Efer, EferFlags, LStar, SFMask, Star};
#[allow(unused_imports)]
use x86_64::registers::rflags::RFlags;
#[allow(unused_imports)]
use x86_64::VirtAddr;








pub fn init() {
    
    
    
    crate::log!("SYSCALL handler ready (entry point set by userland::init)");
}












#[unsafe(naked)]
unsafe extern "C" fn jky() {
    core::arch::naked_asm!(
        
        "push rcx",          
        "push r11",          
        
        
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
        
        
        "pop r11",           
        "pop rcx",           
        
        
        "sysretq",
        
        handler = sym syscall_handler_rust,
    );
}






#[no_mangle]
pub extern "C" fn syscall_handler_rust(
    num: u64,
    arg1: u64,
    arg2: u64,
    aer: u64,
    cfw: u64,
    dhv: u64,
    arg6: u64,
) -> u64 {
    use crate::syscall::linux::nr::AJF_;
    
    
    let ret = crate::syscall::idi(num, arg1, arg2, aer, cfw, dhv, arg6);

    
    crate::lab_mode::trace_bus::fuj(num, [arg1, arg2, aer], ret);

    
    
    if num == AJF_ {
        
        unsafe {
            let result = crate::signals::oss(
                &mut crate::userland::USER_RETURN_RIP,
                &mut crate::userland::USER_RSP_TEMP,
                &mut crate::userland::USER_RETURN_RFLAGS,
            );
            return result as u64;
        }
    }
    
    
    unsafe {
        if let Some(signo) = crate::signals::pnt(
            &mut crate::userland::USER_RETURN_RIP,
            &mut crate::userland::USER_RSP_TEMP,
            &mut crate::userland::USER_RETURN_RFLAGS,
            ret as u64,
        ) {
            
            
            
            crate::userland::SIGNAL_DELIVER_SIGNO = signo;
            
            
        }
    }

    ret as u64
}
