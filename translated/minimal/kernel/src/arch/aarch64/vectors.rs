











use core::sync::atomic::{AtomicBool, Ordering};


static BKL_: AtomicBool = AtomicBool::new(false);



#[repr(C)]
pub struct Qq {
    
    pub regs: [u64; 31],
    
    pub sp_el0: u64,
    
    pub elr_el1: u64,
    
    pub spsr_el1: u64,
}





pub fn init() {
    unsafe {
        
        let vectors: u64;
        core::arch::asm!(
            "adrp {0}, __exception_vectors",
            "add {0}, {0}, :lo12:__exception_vectors",
            out(reg) vectors,
            options(nomem, nostack, preserves_flags)
        );

        
        super::cpu::pvg(vectors);

        BKL_.store(true, Ordering::Release);
        crate::serial_println!("[VECTORS] Exception vector table installed at {:#x}", vectors);
    }
}

pub fn cbb() -> bool {
    BKL_.load(Ordering::Acquire)
}















#[no_mangle]
extern "C" fn qen(tf: &Qq) {
    let esr = unsafe { super::cpu::iyj() };
    let far = unsafe { super::cpu::gqc() };
    let ec = (esr >> 26) & 0x3F;
    let xt = esr & 0x1FF_FFFF;

    match ec {
        0x25 => {
            
            let puy = (xt >> 6) & 1; 
            let frz = xt & 0x3F;    
            crate::serial_println!(
                "\n[EXCEPTION] Data Abort (EL1): FAR={:#x} WnR={} DFSC={:#x} ELR={:#x}",
                far, puy, frz, tf.elr_el1
            );

            
            if frz & 0x3C == 0x04 {
                
                crate::serial_println!("  Translation fault level {}", frz & 0x3);
            }

            
            panic!("Unhandled Data Abort at ELR={:#x} FAR={:#x}", tf.elr_el1, far);
        }
        0x21 => {
            
            crate::serial_println!(
                "\n[EXCEPTION] Instruction Abort (EL1): FAR={:#x} ELR={:#x}",
                far, tf.elr_el1
            );
            panic!("Instruction Abort at ELR={:#x} FAR={:#x}", tf.elr_el1, far);
        }
        0x22 => {
            crate::serial_println!("\n[EXCEPTION] PC Alignment Fault: ELR={:#x}", tf.elr_el1);
            panic!("PC Alignment Fault at {:#x}", tf.elr_el1);
        }
        0x26 => {
            crate::serial_println!("\n[EXCEPTION] SP Alignment Fault: ELR={:#x}", tf.elr_el1);
            panic!("SP Alignment Fault at {:#x}", tf.elr_el1);
        }
        0x3C => {
            
            crate::serial_println!("[EXCEPTION] Breakpoint (BRK) at ELR={:#x}", tf.elr_el1);
        }
        _ => {
            crate::serial_println!(
                "\n[EXCEPTION] Unhandled sync exception: EC={:#x} ISS={:#x} ELR={:#x} FAR={:#x}",
                ec, xt, tf.elr_el1, far
            );
            panic!("Unhandled sync exception EC={:#x}", ec);
        }
    }
}





#[no_mangle]
extern "C" fn qem(_tf: &Qq) {
    let irq = super::gic::hdt();

    if irq == super::gic::BIJ_ {
        
        return;
    }

    match irq {
        
        super::gic::MB_ => {
            
            super::gic::gqp(10);

            
            if crate::interrupts::iht() {
                
                crate::logger::tick();
                crate::time::tick();

                
                crate::trace::akj(crate::trace::EventType::TimerTick, 0);

                
                crate::thread::dvv();
            }
        }
        
        _ => {
            crate::serial_println!("[IRQ] Unhandled IRQ {}", irq);
        }
    }

    
    super::gic::fvb(irq);
}



#[no_mangle]
extern "C" fn qel(tf: &Qq) {
    let esr = unsafe { super::cpu::iyj() };
    let ec = (esr >> 26) & 0x3F;

    match ec {
        0x15 => {
            
            
            let jst = tf.regs[8];
            let pwp = tf.regs[0];
            
            crate::serial_println!("[SYSCALL] SVC from user: num={}", jst);
        }
        0x20 => {
            
            let far = unsafe { super::cpu::gqc() };
            crate::serial_println!(
                "[EXCEPTION] User Instruction Abort: FAR={:#x} ELR={:#x}",
                far, tf.elr_el1
            );
        }
        0x24 => {
            
            let far = unsafe { super::cpu::gqc() };
            crate::serial_println!(
                "[EXCEPTION] User Data Abort: FAR={:#x} ELR={:#x}",
                far, tf.elr_el1
            );
        }
        _ => {
            crate::serial_println!(
                "[EXCEPTION] Unhandled user sync exception: EC={:#x} ELR={:#x}",
                ec, tf.elr_el1
            );
        }
    }
}



#[no_mangle]
extern "C" fn qek(_tf: &Qq) {
    
    let irq = super::gic::hdt();

    if irq == super::gic::BIJ_ {
        return;
    }

    match irq {
        super::gic::MB_ => {
            super::gic::gqp(10);
            if crate::interrupts::iht() {
                crate::logger::tick();
                crate::time::tick();
                crate::trace::akj(crate::trace::EventType::TimerTick, 0);
                crate::thread::dvv();
            }
        }
        _ => {
            crate::serial_println!("[IRQ] Unhandled user IRQ {}", irq);
        }
    }

    super::gic::fvb(irq);
}













core::arch::global_asm!(
    
    
    

    
    ".macro SAVE_REGS",
    "    sub sp, sp, #272",          
    "    stp x0,  x1,  [sp, #(0  * 16)]",
    "    stp x2,  x3,  [sp, #(1  * 16)]",
    "    stp x4,  x5,  [sp, #(2  * 16)]",
    "    stp x6,  x7,  [sp, #(3  * 16)]",
    "    stp x8,  x9,  [sp, #(4  * 16)]",
    "    stp x10, x11, [sp, #(5  * 16)]",
    "    stp x12, x13, [sp, #(6  * 16)]",
    "    stp x14, x15, [sp, #(7  * 16)]",
    "    stp x16, x17, [sp, #(8  * 16)]",
    "    stp x18, x19, [sp, #(9  * 16)]",
    "    stp x20, x21, [sp, #(10 * 16)]",
    "    stp x22, x23, [sp, #(11 * 16)]",
    "    stp x24, x25, [sp, #(12 * 16)]",
    "    stp x26, x27, [sp, #(13 * 16)]",
    "    stp x28, x29, [sp, #(14 * 16)]",
    
    "    str x30,       [sp, #240]",
    
    "    mrs x21, sp_el0",
    "    mrs x22, elr_el1",
    "    mrs x23, spsr_el1",
    "    str x21, [sp, #248]",      
    "    stp x22, x23, [sp, #256]", 
    ".endm",

    
    ".macro RESTORE_REGS",
    "    ldp x22, x23, [sp, #256]", 
    "    ldr x21, [sp, #248]",      
    "    msr sp_el0, x21",
    "    msr elr_el1, x22",
    "    msr spsr_el1, x23",
    "    ldr x30,       [sp, #240]",
    "    ldp x28, x29, [sp, #(14 * 16)]",
    "    ldp x26, x27, [sp, #(13 * 16)]",
    "    ldp x24, x25, [sp, #(12 * 16)]",
    "    ldp x22, x23, [sp, #(11 * 16)]",
    "    ldp x20, x21, [sp, #(10 * 16)]",
    "    ldp x18, x19, [sp, #(9  * 16)]",
    "    ldp x16, x17, [sp, #(8  * 16)]",
    "    ldp x14, x15, [sp, #(7  * 16)]",
    "    ldp x12, x13, [sp, #(6  * 16)]",
    "    ldp x10, x11, [sp, #(5  * 16)]",
    "    ldp x8,  x9,  [sp, #(4  * 16)]",
    "    ldp x6,  x7,  [sp, #(3  * 16)]",
    "    ldp x4,  x5,  [sp, #(2  * 16)]",
    "    ldp x2,  x3,  [sp, #(1  * 16)]",
    "    ldp x0,  x1,  [sp, #(0  * 16)]",
    "    add sp, sp, #272",
    "    eret",
    ".endm",

    
    
    
    
    
    
    
    
    ".section .text",
    ".balign 2048",
    ".global __exception_vectors",
    "__exception_vectors:",

    
    
    "    b .",
    ".balign 128",
    
    "    b .",
    ".balign 128",
    
    "    b .",
    ".balign 128",
    
    "    b .",
    ".balign 128",

    
    
    "    b __el1_sync_entry",
    ".balign 128",
    
    "    b __el1_irq_entry",
    ".balign 128",
    
    "    b .",
    ".balign 128",
    
    "    b .",
    ".balign 128",

    
    
    "    b __el0_sync_entry",
    ".balign 128",
    
    "    b __el0_irq_entry",
    ".balign 128",
    
    "    b .",
    ".balign 128",
    
    "    b .",
    ".balign 128",

    
    
    "    b .",
    ".balign 128",
    
    "    b .",
    ".balign 128",
    
    "    b .",
    ".balign 128",
    
    "    b .",
    ".balign 128",

    
    
    

    "__el1_sync_entry:",
    "    SAVE_REGS",
    "    mov x0, sp",
    "    bl el1_sync_handler",
    "    RESTORE_REGS",

    "__el1_irq_entry:",
    "    SAVE_REGS",
    "    mov x0, sp",
    "    bl el1_irq_handler",
    "    RESTORE_REGS",

    "__el0_sync_entry:",
    "    SAVE_REGS",
    "    mov x0, sp",
    "    bl el0_sync_handler",
    "    RESTORE_REGS",

    "__el0_irq_entry:",
    "    SAVE_REGS",
    "    mov x0, sp",
    "    bl el0_irq_handler",
    "    RESTORE_REGS",
);
