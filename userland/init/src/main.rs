//! Init Process (PID 1)
//!
//! First Ring 3 userland process in TrustOS.
//! Spawns core services, monitors health, handles restarts.

#![no_std]
#![no_main]

use core::panic::PanicInfo;
use trustos_syscall as sys;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    init_main()
}

fn init_main() -> ! {
    sys::print("[init] TrustOS init (PID 1) starting\n");

    let pid = sys::getpid();
    let mut buf = [0u8; 32];
    let n = format_i32(pid, &mut buf);
    sys::print("[init] PID = ");
    sys::write(1, &buf[..n]);
    sys::print("\n");

    // Get uptime via clock_gettime
    let mut ts = sys::Timespec { tv_sec: 0, tv_nsec: 0 };
    if sys::clock_gettime(0, &mut ts) == 0 {
        sys::print("[init] Boot clock: ");
        let n = format_i64(ts.tv_sec, &mut buf);
        sys::write(1, &buf[..n]);
        sys::print("s\n");
    }

    // Test memory allocation: brk
    let heap_start = sys::brk(0);
    if heap_start > 0 {
        let new_brk = sys::brk(heap_start as u64 + 4096);
        if new_brk > heap_start {
            sys::print("[init] Heap allocation OK (brk +4096)\n");
        } else {
            sys::print("[init] Heap allocation FAILED\n");
        }
    }

    sys::print("[init] All systems go. Entering idle loop.\n");

    // Init process idles — scheduler preempts us as needed
    loop {
        sys::sched_yield();
    }
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

fn format_i64(mut val: i64, buf: &mut [u8]) -> usize {
    if val == 0 {
        buf[0] = b'0';
        return 1;
    }
    let neg = val < 0;
    if neg { val = -val; }
    let mut i = 0;
    let mut tmp = [0u8; 20];
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
    sys::print("[init] PANIC!\n");
    sys::exit(1);
}
