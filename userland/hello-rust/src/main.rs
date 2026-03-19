//! Hello World — TrustOS Ring 3 userland program
//!
//! Demonstrates a real Rust program running in user space with proper
//! syscall-based I/O. No kernel code, no Ring 0 privileges.

#![no_std]
#![no_main]

use core::panic::PanicInfo;
use trustos_syscall as sys;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    sys::print("Hello from TrustOS Ring 3 userland!\n");
    sys::print("  PID: ");

    let pid = sys::getpid();
    let mut buf = [0u8; 16];
    let n = format_i32(pid, &mut buf);
    sys::write(1, &buf[..n]);
    sys::print("\n");

    // Test memory via brk
    let base = sys::brk(0);
    let grown = sys::brk(base as u64 + 0x1000);
    if grown > base {
        sys::print("  Heap: OK (brk works)\n");
    }

    // Test clock
    let mut ts = sys::Timespec { tv_sec: 0, tv_nsec: 0 };
    if sys::clock_gettime(0, &mut ts) == 0 {
        sys::print("  Clock: OK\n");
    }

    sys::print("Goodbye from Ring 3!\n");
    sys::exit(0);
}

fn format_i32(mut val: i32, buf: &mut [u8]) -> usize {
    if val == 0 {
        buf[0] = b'0';
        return 1;
    }
    let neg = val < 0;
    if neg { val = -val; }
    let mut i = 0;
    let mut tmp = [0u8; 12];
    while val > 0 {
        tmp[i] = b'0' + (val % 10) as u8;
        val /= 10;
        i += 1;
    }
    let mut out = 0;
    if neg { buf[out] = b'-'; out += 1; }
    while i > 0 {
        i -= 1;
        buf[out] = tmp[i];
        out += 1;
    }
    out
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    sys::print("PANIC in userland!\n");
    sys::exit(127);
}
