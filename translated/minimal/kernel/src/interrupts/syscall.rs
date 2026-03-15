







#[allow(moc)]
use x86_64::registers::model_specific::{Efer, EferFlags, LStar, SFMask, And};
#[allow(moc)]
use x86_64::registers::rflags::RFlags;
#[allow(moc)]
use x86_64::VirtAddr;








pub fn init() {
    
    
    
    crate::log!("SYSCALL handler ready (entry point set by userland::init)");
}












#[unsafe(evb)]
unsafe extern "C" fn prb() {
    core::arch::evc!(
        
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
        
        cfd = aaw prc,
    );
}






#[no_mangle]
pub extern "C" fn prc(
    num: u64,
    aai: u64,
    agf: u64,
    bfx: u64,
    fcs: u64,
    gyx: u64,
    kax: u64,
) -> u64 {
    use crate::syscall::linux::nr::AHJ_;
    
    
    let aux = crate::syscall::oae(num, aai, agf, bfx, fcs, gyx, kax);

    
    crate::lab_mode::trace_bus::ktb(num, [aai, agf, bfx], aux);

    
    
    if num == AHJ_ {
        
        unsafe {
            let result = crate::signals::wog(
                &mut crate::userland::YC_,
                &mut crate::userland::YD_,
                &mut crate::userland::YB_,
            );
            return result as u64;
        }
    }
    
    
    unsafe {
        if let Some(qk) = crate::signals::xml(
            &mut crate::userland::YC_,
            &mut crate::userland::YD_,
            &mut crate::userland::YB_,
            aux as u64,
        ) {
            
            
            
            crate::userland::AHX_ = qk;
            
            
        }
    }

    aux as u64
}
