//! TRustOs Shell
//! 
//! Basic interactive shell for TRustOs.
//! Commands: ls, cat, help, exit, ps

#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[path = "../../syscall.rs"]
mod syscall;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    shell_main()
}

fn shell_main() -> ! {
    print_banner();
    
    loop {
        print_prompt();
        
        // TODO: Read command from keyboard
        // let cmd = read_command();
        
        // TODO: Parse and execute
        // execute_command(cmd);
        
        syscall_yield();
    }
}

fn print_banner() {
    // TODO: Print to VGA buffer or serial
}

fn print_prompt() {
    // TODO: Print "trustos> "
}

fn syscall_yield() {
    unsafe {
        core::arch::asm!(
            "syscall",
            in("rax") 6u64,
        );
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
