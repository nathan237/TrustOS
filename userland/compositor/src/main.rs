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
    compositor_main()
}

fn compositor_main() -> ! {
    init_framebuffer();
    
    loop {
        handle_events();
        render_frame();
        syscall::yield_cpu();
    }
}

fn init_framebuffer() {
    // TODO: Map framebuffer via syscall
}

fn handle_events() {
    // TODO: Input events
}

fn render_frame() {
    // TODO: Composite windows
}
