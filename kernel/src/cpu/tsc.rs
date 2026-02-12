//! TSC - Time Stamp Counter
//! 
//! High-precision timing using CPU's built-in counter.
//! Provides nanosecond resolution instead of PIT's 10ms resolution.

use core::sync::atomic::{AtomicU64, Ordering};

/// TSC frequency in Hz (calibrated at boot)
static TSC_FREQ_HZ: AtomicU64 = AtomicU64::new(0);

/// TSC value at boot (for calculating uptime)
static TSC_BOOT: AtomicU64 = AtomicU64::new(0);

/// Initialize TSC timing
pub fn init(frequency_hz: u64) {
    TSC_FREQ_HZ.store(frequency_hz, Ordering::Release);
    TSC_BOOT.store(read_tsc(), Ordering::Release);
    
    crate::serial_println!("[TSC] Initialized: {} Hz ({} GHz)", 
        frequency_hz, frequency_hz / 1_000_000_000);
}

/// Read raw TSC value
#[inline(always)]
pub fn read_tsc() -> u64 {
    unsafe { core::arch::x86_64::_rdtsc() }
}

/// Read TSC with serialization (more accurate for benchmarking)
#[inline(always)]
pub fn read_tsc_serialized() -> u64 {
    // LFENCE serializes, ensuring all previous instructions complete
    // This is simpler than CPUID and doesn't require clobbering rbx
    unsafe {
        core::arch::asm!("lfence", options(nostack, preserves_flags));
        core::arch::x86_64::_rdtsc()
    }
}

/// Read TSC with RDTSCP (includes processor ID, serializing read)
#[inline(always)]
pub fn read_tscp() -> (u64, u32) {
    let mut aux: u32;
    let tsc: u64;
    
    unsafe {
        let lo: u32;
        let hi: u32;
        core::arch::asm!(
            "rdtscp",
            out("eax") lo,
            out("edx") hi,
            out("ecx") aux,
            options(nostack)
        );
        tsc = ((hi as u64) << 32) | (lo as u64);
    }
    
    (tsc, aux)
}

/// Get TSC frequency in Hz
pub fn frequency_hz() -> u64 {
    TSC_FREQ_HZ.load(Ordering::Acquire)
}

/// Convert TSC cycles to nanoseconds
#[inline]
pub fn cycles_to_nanos(cycles: u64) -> u64 {
    let freq = TSC_FREQ_HZ.load(Ordering::Relaxed);
    if freq == 0 {
        return 0;
    }
    // (cycles * 1_000_000_000) / freq, but avoid overflow
    // Use 128-bit arithmetic
    let nanos = (cycles as u128 * 1_000_000_000u128) / freq as u128;
    nanos as u64
}

/// Convert TSC cycles to microseconds
#[inline]
pub fn cycles_to_micros(cycles: u64) -> u64 {
    let freq = TSC_FREQ_HZ.load(Ordering::Relaxed);
    if freq == 0 {
        return 0;
    }
    (cycles as u128 * 1_000_000u128 / freq as u128) as u64
}

/// Convert TSC cycles to milliseconds
#[inline]
pub fn cycles_to_millis(cycles: u64) -> u64 {
    let freq = TSC_FREQ_HZ.load(Ordering::Relaxed);
    if freq == 0 {
        return 0;
    }
    (cycles as u128 * 1_000u128 / freq as u128) as u64
}

/// Get current time in nanoseconds since boot
pub fn now_nanos() -> u64 {
    let boot = TSC_BOOT.load(Ordering::Relaxed);
    let current = read_tsc();
    let elapsed = current.saturating_sub(boot);
    cycles_to_nanos(elapsed)
}

/// Get current time in microseconds since boot  
pub fn now_micros() -> u64 {
    let boot = TSC_BOOT.load(Ordering::Relaxed);
    let current = read_tsc();
    let elapsed = current.saturating_sub(boot);
    cycles_to_micros(elapsed)
}

/// Get current time in milliseconds since boot
pub fn now_millis() -> u64 {
    let boot = TSC_BOOT.load(Ordering::Relaxed);
    let current = read_tsc();
    let elapsed = current.saturating_sub(boot);
    cycles_to_millis(elapsed)
}

/// High-precision delay in nanoseconds
pub fn delay_nanos(nanos: u64) {
    let freq = TSC_FREQ_HZ.load(Ordering::Relaxed);
    if freq == 0 {
        return;
    }
    
    let cycles_to_wait = (nanos as u128 * freq as u128 / 1_000_000_000u128) as u64;
    let start = read_tsc();
    let target = start + cycles_to_wait;
    
    while read_tsc() < target {
        core::hint::spin_loop();
    }
}

/// High-precision delay in microseconds
pub fn delay_micros(micros: u64) {
    delay_nanos(micros * 1_000);
}

/// High-precision delay in milliseconds
pub fn delay_millis(millis: u64) {
    delay_nanos(millis * 1_000_000);
}

/// PIT-based real-time delay in milliseconds — guaranteed wall-clock time
/// Uses PIT Channel 2 one-shot mode, works reliably in VirtualBox
pub fn pit_delay_ms(millis: u64) {
    const PIT_FREQ: u64 = 1_193_182;
    const PIT_CHANNEL2: u16 = 0x42;
    const PIT_COMMAND: u16 = 0x43;
    // PIT max count = 65535 → max ~54.9ms per shot
    const MAX_MS_PER_SHOT: u64 = 50;

    let mut remaining = millis;
    while remaining > 0 {
        let chunk = remaining.min(MAX_MS_PER_SHOT);
        let pit_count = (PIT_FREQ * chunk / 1000) as u16;
        if pit_count == 0 { break; }

        unsafe {
            use x86_64::instructions::port::Port;
            let mut cmd_port: Port<u8> = Port::new(PIT_COMMAND);
            let mut ch2_port: Port<u8> = Port::new(PIT_CHANNEL2);
            let mut port61: Port<u8> = Port::new(0x61);

            // Save port 0x61
            let val = port61.read();
            // Disable gate first to reset
            port61.write(val & !0x01);

            // Command: Channel 2, lobyte/hibyte, mode 0 (one-shot), binary
            cmd_port.write(0b10110000);
            // Write count
            ch2_port.write((pit_count & 0xFF) as u8);
            ch2_port.write((pit_count >> 8) as u8);

            // Enable gate to start counting
            port61.write((val & !0x20) | 0x01);

            // Wait for bit 5 of port 0x61 to go high (counter reached 0)
            loop {
                let status = port61.read();
                if (status & 0x20) != 0 {
                    break;
                }
                core::hint::spin_loop();
            }

            // Restore
            port61.write(val);
        }
        remaining -= chunk;
    }
}

/// Calibrate TSC frequency using PIT
/// Returns frequency in Hz
pub fn calibrate_tsc() -> u64 {
    // Method 1: Try to read from CPUID (Intel only)
    if let Some(freq) = calibrate_from_cpuid() {
        return freq;
    }
    
    // Method 2: Calibrate against PIT timer
    calibrate_against_pit()
}

/// Try to get TSC frequency from CPUID (works on recent Intel CPUs)
fn calibrate_from_cpuid() -> Option<u64> {
    let cpuid_15 = unsafe { core::arch::x86_64::__cpuid(0x15) };
    
    // CPUID.15H: TSC/Core Crystal Clock Ratio
    // EAX = denominator, EBX = numerator, ECX = crystal frequency (if non-zero)
    if cpuid_15.eax != 0 && cpuid_15.ebx != 0 {
        let crystal_freq = if cpuid_15.ecx != 0 {
            cpuid_15.ecx as u64
        } else {
            // Estimate crystal frequency based on CPU model
            // Most modern Intel: 24 MHz or 25 MHz
            25_000_000u64
        };
        
        let tsc_freq = crystal_freq * cpuid_15.ebx as u64 / cpuid_15.eax as u64;
        if tsc_freq > 100_000_000 { // Sanity check: > 100 MHz
            return Some(tsc_freq);
        }
    }
    
    // CPUID.16H: Processor Frequency Information (newer Intel)
    let cpuid_0 = unsafe { core::arch::x86_64::__cpuid(0) };
    if cpuid_0.eax >= 0x16 {
        let cpuid_16 = unsafe { core::arch::x86_64::__cpuid(0x16) };
        // EAX = base frequency in MHz
        if cpuid_16.eax != 0 {
            let freq_mhz = cpuid_16.eax as u64;
            return Some(freq_mhz * 1_000_000);
        }
    }
    
    None
}

/// Calibrate TSC against PIT (Programmable Interval Timer)
fn calibrate_against_pit() -> u64 {
    const PIT_FREQ: u64 = 1_193_182; // PIT frequency in Hz
    const CALIBRATION_MS: u64 = 50;   // Calibration period
    const PIT_CHANNEL2: u16 = 0x42;
    const PIT_COMMAND: u16 = 0x43;
    
    // Configure PIT Channel 2 for one-shot mode
    let pit_count = (PIT_FREQ * CALIBRATION_MS / 1000) as u16;
    
    unsafe {
        use x86_64::instructions::port::Port;
        
        let mut cmd_port: Port<u8> = Port::new(PIT_COMMAND);
        let mut ch2_port: Port<u8> = Port::new(PIT_CHANNEL2);
        
        // Command: Channel 2, lobyte/hibyte, mode 0 (one-shot), binary
        cmd_port.write(0b10110000);
        
        // Write count
        ch2_port.write((pit_count & 0xFF) as u8);
        ch2_port.write((pit_count >> 8) as u8);
        
        // Enable Channel 2 gate
        let mut port61: Port<u8> = Port::new(0x61);
        let val = port61.read();
        port61.write(val | 0x01); // Enable gate
        
        // Read initial TSC
        let tsc_start = read_tsc();
        
        // Wait for PIT to count down
        // Read port 0x61 bit 5 - goes high when counter reaches 0
        loop {
            let status = port61.read();
            if (status & 0x20) != 0 {
                break;
            }
        }
        
        // Read final TSC
        let tsc_end = read_tsc();
        
        // Calculate frequency
        let tsc_elapsed = tsc_end - tsc_start;
        let freq = tsc_elapsed * 1000 / CALIBRATION_MS;
        
        // Restore port 0x61
        port61.write(val);
        
        freq
    }
}

/// Stopwatch for precise timing
pub struct Stopwatch {
    start: u64,
}

impl Stopwatch {
    /// Start a new stopwatch
    #[inline]
    pub fn start() -> Self {
        Self { start: read_tsc() }
    }
    
    /// Get elapsed time in nanoseconds
    #[inline]
    pub fn elapsed_nanos(&self) -> u64 {
        let elapsed = read_tsc() - self.start;
        cycles_to_nanos(elapsed)
    }
    
    /// Get elapsed time in microseconds
    #[inline]
    pub fn elapsed_micros(&self) -> u64 {
        let elapsed = read_tsc() - self.start;
        cycles_to_micros(elapsed)
    }
    
    /// Get elapsed time in milliseconds
    #[inline]
    pub fn elapsed_millis(&self) -> u64 {
        let elapsed = read_tsc() - self.start;
        cycles_to_millis(elapsed)
    }
    
    /// Get raw elapsed cycles
    #[inline]
    pub fn elapsed_cycles(&self) -> u64 {
        read_tsc() - self.start
    }
    
    /// Reset and get elapsed time in nanoseconds
    pub fn lap_nanos(&mut self) -> u64 {
        let now = read_tsc();
        let elapsed = now - self.start;
        self.start = now;
        cycles_to_nanos(elapsed)
    }
}
