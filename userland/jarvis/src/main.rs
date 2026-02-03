#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[path = "../../syscall.rs"]
mod syscall;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    jarvis_main()
}

fn jarvis_main() -> ! {
    loop {
        let ch = syscall::receive(1).unwrap_or(0);
        if ch != 0 {
            process_request(ch);
        }
        syscall::yield_cpu();
    }
}

fn process_request(req: u64) {
    match req & 0xFF {
        1 => nlu_parse(req >> 8),
        2 => ml_infer(req >> 8),
        _ => {}
    }
}

fn nlu_parse(_text: u64) {
    // TODO: NLU processing
}

fn ml_infer(_input: u64) {
    // TODO: ML inference
}
