//! CoreMark EEMBC benchmark integration
//!
//! Compiles official CoreMark C sources via cc crate and links them into the kernel.
//! Provides FFI glue: TSC timer, serial output, and memory stubs.
//! Output is captured in a buffer and displayed on the framebuffer after completion.

extern "C" {
    fn main();
}

/// Entry point — called from kmain when feature "coremark" is active.
pub fn run() -> ! {
    // Clear screen and show banner on framebuffer
    crate::println!("");
    crate::println!("========================================");
    crate::println!("  CoreMark 1.0 - TrustOS Edition");
    crate::println!("========================================");

    let freq = crate::cpu::tsc_frequency();
    crate::println!("[CoreMark] TSC frequency: {} MHz", freq / 1_000_000);
    crate::println!("[CoreMark] Running benchmark...");
    crate::println!("[CoreMark] Please wait ~30-60 seconds...");

    crate::serial_println!("[CoreMark] TSC={} MHz, calling main()", freq / 1_000_000);

    unsafe { main(); }

    // Display results on framebuffer
    crate::println!("");
    crate::println!("========== RESULTS ==========");
    let guard = OUTPUT_BUF.lock();
    let (ref buf, pos) = *guard;
    if let Ok(s) = core::str::from_utf8(&buf[..pos]) {
        for line in s.lines() {
            crate::println!("{}", line);
            crate::serial_println!("{}", line);
        }
    }
    drop(guard);
    crate::println!("=============================");
    crate::println!("[CoreMark] Done.");

    loop { core::hint::spin_loop(); }
}

// ── Static buffer for capturing all ee_printf output ──

const OUTPUT_BUF_SIZE: usize = 8192;
static OUTPUT_BUF: spin::Mutex<([u8; OUTPUT_BUF_SIZE], usize)> =
    spin::Mutex::new(([0u8; OUTPUT_BUF_SIZE], 0));

// ── FFI exports called by CoreMark C code ──

#[no_mangle]
pub extern "C" fn trustos_read_tsc() -> u64 {
    crate::cpu::tsc::read_tsc_serialized()
}

#[no_mangle]
pub extern "C" fn trustos_tsc_freq() -> u64 {
    let freq = crate::cpu::tsc::frequency_hz();
    if freq == 0 { 3_300_000_000 } else { freq }
}

#[no_mangle]
pub extern "C" fn trustos_serial_putchar(c: u8) {
    crate::arch::serial::write_byte(c);
    let mut guard = OUTPUT_BUF.lock();
    let (ref mut buf, ref mut pos) = *guard;
    if *pos < buf.len() {
        buf[*pos] = c;
        *pos += 1;
    }
}
