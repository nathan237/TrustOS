//! Syscall wrapper for userland
pub fn exit(code: u64) -> ! {
    unsafe {
        core::arch::asm!(
            "syscall",
            in("rax") 0u64,
            in("rdi") code,
        );
    }
    loop {}
}

pub fn send(channel: u64) -> Result<(), u64> {
    let result: u64;
    unsafe {
        core::arch::asm!(
            "syscall",
            in("rax") 1u64,
            in("rdi") channel,
            lateout("rax") result,
        );
    }
    if result == 0 { Ok(()) } else { Err(result) }
}

pub fn receive(channel: u64) -> Result<u64, u64> {
    let result: u64;
    unsafe {
        core::arch::asm!(
            "syscall",
            in("rax") 2u64,
            in("rdi") channel,
            lateout("rax") result,
        );
    }
    Ok(result)
}

pub fn create_channel() -> u64 {
    let result: u64;
    unsafe {
        core::arch::asm!(
            "syscall",
            in("rax") 3u64,
            lateout("rax") result,
        );
    }
    result
}

pub fn spawn(entry: u64) -> u64 {
    let result: u64;
    unsafe {
        core::arch::asm!(
            "syscall",
            in("rax") 4u64,
            in("rdi") entry,
            lateout("rax") result,
        );
    }
    result
}

pub fn yield_cpu() {
    unsafe {
        core::arch::asm!(
            "syscall",
            in("rax") 6u64,
        );
    }
}
