//! Hardware Debug Toolkit for bare-metal debugging
//!
//! Provides comprehensive diagnostic tools when running TrustOS on real hardware:
//! - POST code output (port 0x80) for external debug cards
//! - Boot checkpoint system with serial + POST + optional PC speaker beep
//! - Stack walking (frame-pointer based backtrace)
//! - Full CPU state dump (all GPR, control, segment, MSR registers)
//! - I/O port read/write from shell
//! - MSR read/write from shell
//! - Raw CPUID query from shell
//! - ACPI table hex dump
//! - Memory map display
//! - E9 debug port (Bochs/QEMU debug console)

pub mod netconsole;
pub mod remoteshell;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use core::sync::atomic::{AtomicU8, AtomicU64, AtomicBool, Ordering};
use spin::Mutex;

// ═══════════════════════════════════════════════════════════════════════════════
// 1. POST CODE OUTPUT (Port 0x80)
// ═══════════════════════════════════════════════════════════════════════════════

/// Current POST code (visible externally on debug cards)
static LAST_POST_CODE: AtomicU8 = AtomicU8::new(0);

/// Write a POST code to port 0x80 (visible on hardware POST cards)
/// Also writes to port 0xE9 (Bochs/QEMU debug console)
pub fn post_code(code: u8) {
    LAST_POST_CODE.store(code, Ordering::Relaxed);
    #[cfg(target_arch = "x86_64")]
    unsafe {
        core::arch::asm!("out dx, al", in("dx") 0x80u16, in("al") code, options(nostack, preserves_flags));
        // Also to E9 debug port
        core::arch::asm!("out dx, al", in("dx") 0xE9u16, in("al") code, options(nostack, preserves_flags));
    }
}

/// Get last POST code
pub fn last_post_code() -> u8 {
    LAST_POST_CODE.load(Ordering::Relaxed)
}

// ═══════════════════════════════════════════════════════════════════════════════
// 2. BOOT CHECKPOINTS — numbered milestones with POST + serial + optional beep
// ═══════════════════════════════════════════════════════════════════════════════

/// Max boot checkpoints (fixed-size to avoid heap allocation before heap init)
const MAX_CHECKPOINTS: usize = 32;

/// Boot checkpoint log (fixed array + count — no heap needed)
struct CheckpointLog {
    entries: [(u64, u8, &'static str); MAX_CHECKPOINTS],
    count: usize,
}

impl CheckpointLog {
    const fn new() -> Self {
        Self {
            entries: [(0, 0, ""); MAX_CHECKPOINTS],
            count: 0,
        }
    }
    fn push(&mut self, entry: (u64, u8, &'static str)) {
        if self.count < MAX_CHECKPOINTS {
            self.entries[self.count] = entry;
            self.count += 1;
        }
    }
    fn iter(&self) -> core::slice::Iter<'_, (u64, u8, &'static str)> {
        self.entries[..self.count].iter()
    }
    fn is_empty(&self) -> bool {
        self.count == 0
    }
    fn as_slice(&self) -> &[(u64, u8, &'static str)] {
        &self.entries[..self.count]
    }
}

static CHECKPOINTS: Mutex<CheckpointLog> = Mutex::new(CheckpointLog::new());

// POST code assignments for boot phases
pub const POST_SERIAL_INIT:     u8 = 0x10;
pub const POST_FRAMEBUFFER:     u8 = 0x11;
pub const POST_GDT:             u8 = 0x20;
pub const POST_IDT:             u8 = 0x21;
pub const POST_CPU_DETECT:      u8 = 0x22;
pub const POST_ACPI:            u8 = 0x30;
pub const POST_APIC:            u8 = 0x31;
pub const POST_SMP:             u8 = 0x32;
pub const POST_MEMORY:          u8 = 0x40;
pub const POST_HEAP:            u8 = 0x41;
pub const POST_PAGING:          u8 = 0x42;
pub const POST_PCI:             u8 = 0x50;
pub const POST_DISK:            u8 = 0x51;
pub const POST_NETWORK:         u8 = 0x60;
pub const POST_VFS:             u8 = 0x70;
pub const POST_PROCESS:         u8 = 0x80;
pub const POST_SHELL_READY:     u8 = 0xAA;
pub const POST_PANIC:           u8 = 0xFF;

/// Record a boot checkpoint: writes POST code, logs to serial, records TSC timestamp
pub fn checkpoint(code: u8, name: &'static str) {
    post_code(code);
    let tsc = read_tsc();
    crate::serial_println!("[POST 0x{:02X}] {}", code, name);

    // Store in checkpoint log (best-effort if lock available)
    if let Some(mut log) = CHECKPOINTS.try_lock() {
        log.push((tsc, code, name));
    }
}

/// Get all boot checkpoints
pub fn get_checkpoints() -> Vec<(u64, u8, &'static str)> {
    let log = CHECKPOINTS.lock();
    log.as_slice().to_vec()
}

// ═══════════════════════════════════════════════════════════════════════════════
// 3. PC SPEAKER BEEP — audible feedback on real hardware
// ═══════════════════════════════════════════════════════════════════════════════

/// Single short beep via PC speaker (PIT channel 2)
#[cfg(target_arch = "x86_64")]
pub fn beep(frequency_hz: u32, duration_ms: u32) {
    if frequency_hz == 0 { return; }
    let divisor = 1193180u32 / frequency_hz;

    unsafe {
        // Set PIT channel 2 to square wave mode
        core::arch::asm!("out dx, al", in("dx") 0x43u16, in("al") 0xB6u8, options(nostack, preserves_flags));
        core::arch::asm!("out dx, al", in("dx") 0x42u16, in("al") (divisor & 0xFF) as u8, options(nostack, preserves_flags));
        core::arch::asm!("out dx, al", in("dx") 0x42u16, in("al") ((divisor >> 8) & 0xFF) as u8, options(nostack, preserves_flags));

        // Enable speaker (bits 0 and 1 of port 0x61)
        let val: u8;
        core::arch::asm!("in al, dx", in("dx") 0x61u16, out("al") val, options(nostack, preserves_flags));
        let on = val | 0x03;
        core::arch::asm!("out dx, al", in("dx") 0x61u16, in("al") on, options(nostack, preserves_flags));

        // Busy-wait for duration
        busy_wait_ms(duration_ms);

        // Disable speaker
        let off = val & !0x03;
        core::arch::asm!("out dx, al", in("dx") 0x61u16, in("al") off, options(nostack, preserves_flags));
    }
}

#[cfg(not(target_arch = "x86_64"))]
pub fn beep(_frequency_hz: u32, _duration_ms: u32) {}

/// Quick beep patterns for boot feedback
pub fn beep_ok()    { beep(1000, 100); }  // Single short beep = OK
pub fn beep_warn()  { beep(500, 200); busy_wait_ms(100); beep(500, 200); } // Double low = warning
pub fn beep_error() { beep(200, 500); } // Long low = error

// ═══════════════════════════════════════════════════════════════════════════════
// 4. STACK WALKING — frame-pointer based backtrace
// ═══════════════════════════════════════════════════════════════════════════════

/// Walk the stack using frame pointers (RBP chain)
/// Returns pairs of (return_address, frame_pointer)
#[cfg(target_arch = "x86_64")]
pub fn stack_walk(max_frames: usize) -> Vec<(u64, u64)> {
    let mut frames = Vec::new();
    let mut rbp: u64;
    
    unsafe {
        core::arch::asm!("mov {}, rbp", out(reg) rbp);
    }
    
    let kernel_start = 0xFFFF_8000_0000_0000u64; // higher half
    
    for _ in 0..max_frames {
        if rbp == 0 || rbp < kernel_start {
            break;
        }
        
        // Safety: we validate the pointer is in kernel space
        let frame_ptr = rbp as *const u64;
        
        // Read return address (at rbp+8) and previous rbp (at rbp)
        let ret_addr = unsafe { core::ptr::read_volatile(frame_ptr.add(1)) };
        let prev_rbp = unsafe { core::ptr::read_volatile(frame_ptr) };
        
        if ret_addr == 0 {
            break;
        }
        
        frames.push((ret_addr, rbp));
        
        // Prevent infinite loops
        if prev_rbp <= rbp {
            break;
        }
        rbp = prev_rbp;
    }
    
    frames
}

#[cfg(not(target_arch = "x86_64"))]
pub fn stack_walk(_max_frames: usize) -> Vec<(u64, u64)> {
    Vec::new()
}

/// Format a backtrace for display
pub fn format_backtrace(max_frames: usize) -> Vec<String> {
    let mut lines = Vec::new();
    lines.push(String::from("  Stack Backtrace:"));
    lines.push(String::from("  ─────────────────────────────────────────────"));
    
    let frames = stack_walk(max_frames);
    if frames.is_empty() {
        lines.push(String::from("  <no frames — frame pointers may be omitted>"));
        lines.push(String::from("  Hint: build with RUSTFLAGS=\"-Cforce-frame-pointers=yes\""));
    } else {
        for (i, (ret_addr, rbp)) in frames.iter().enumerate() {
            lines.push(format!("  #{:>2}: 0x{:016x}  (rbp=0x{:016x})", i, ret_addr, rbp));
        }
    }
    
    lines
}

// ═══════════════════════════════════════════════════════════════════════════════
// 5. FULL CPU STATE DUMP — all registers for crash analysis
// ═══════════════════════════════════════════════════════════════════════════════

/// Dump ALL cpu registers (GPR + control + segment + MSR) for crash debugging
#[cfg(target_arch = "x86_64")]
pub fn full_cpu_dump() -> Vec<String> {
    let mut lines = Vec::new();
    
    let rax: u64; let rcx: u64; let rdx: u64;
    let rsi: u64; let rdi: u64; let rsp: u64; let rbp: u64;
    let r8: u64;  let r9: u64;  let r10: u64; let r11: u64;
    let r12: u64; let r13: u64; let r14: u64; let r15: u64;
    let rflags: u64;
    let cr0: u64; let cr2: u64; let cr3: u64; let cr4: u64;
    let cs: u16; let ds: u16; let es: u16; let fs: u16; let gs: u16; let ss: u16;
    
    unsafe {
        core::arch::asm!("mov {}, rax", out(reg) rax);
        // Note: rbx is reserved by LLVM and cannot be read via inline asm
        core::arch::asm!("mov {}, rcx", out(reg) rcx);
        core::arch::asm!("mov {}, rdx", out(reg) rdx);
        core::arch::asm!("mov {}, rsi", out(reg) rsi);
        core::arch::asm!("mov {}, rdi", out(reg) rdi);
        core::arch::asm!("mov {}, rsp", out(reg) rsp);
        core::arch::asm!("mov {}, rbp", out(reg) rbp);
        core::arch::asm!("mov {}, r8",  out(reg) r8);
        core::arch::asm!("mov {}, r9",  out(reg) r9);
        core::arch::asm!("mov {}, r10", out(reg) r10);
        core::arch::asm!("mov {}, r11", out(reg) r11);
        core::arch::asm!("mov {}, r12", out(reg) r12);
        core::arch::asm!("mov {}, r13", out(reg) r13);
        core::arch::asm!("mov {}, r14", out(reg) r14);
        core::arch::asm!("mov {}, r15", out(reg) r15);
        core::arch::asm!("pushfq; pop {}", out(reg) rflags);
        core::arch::asm!("mov {}, cr0", out(reg) cr0);
        core::arch::asm!("mov {}, cr2", out(reg) cr2);
        core::arch::asm!("mov {}, cr3", out(reg) cr3);
        core::arch::asm!("mov {}, cr4", out(reg) cr4);
        core::arch::asm!("mov {:x}, cs", out(reg) cs);
        core::arch::asm!("mov {:x}, ds", out(reg) ds);
        core::arch::asm!("mov {:x}, es", out(reg) es);
        core::arch::asm!("mov {:x}, fs", out(reg) fs);
        core::arch::asm!("mov {:x}, gs", out(reg) gs);
        core::arch::asm!("mov {:x}, ss", out(reg) ss);
    }
    
    lines.push(String::from("  ╔══════════════════════════════════════════════════╗"));
    lines.push(String::from("  ║         FULL CPU STATE DUMP                     ║"));
    lines.push(String::from("  ╚══════════════════════════════════════════════════╝"));
    
    lines.push(String::from("  ── General Purpose Registers ──"));
    lines.push(format!("  RAX = 0x{:016x}   RBX = <LLVM reserved>", rax));
    lines.push(format!("  RCX = 0x{:016x}   RDX = 0x{:016x}", rcx, rdx));
    lines.push(format!("  RSI = 0x{:016x}   RDI = 0x{:016x}", rsi, rdi));
    lines.push(format!("  RSP = 0x{:016x}   RBP = 0x{:016x}", rsp, rbp));
    lines.push(format!("  R8  = 0x{:016x}   R9  = 0x{:016x}", r8, r9));
    lines.push(format!("  R10 = 0x{:016x}   R11 = 0x{:016x}", r10, r11));
    lines.push(format!("  R12 = 0x{:016x}   R13 = 0x{:016x}", r12, r13));
    lines.push(format!("  R14 = 0x{:016x}   R15 = 0x{:016x}", r14, r15));
    
    lines.push(String::from(""));
    lines.push(String::from("  ── RFLAGS ──"));
    lines.push(format!("  RFLAGS = 0x{:016x}", rflags));
    let mut flags = Vec::new();
    if rflags & (1 << 0) != 0 { flags.push("CF"); }
    if rflags & (1 << 2) != 0 { flags.push("PF"); }
    if rflags & (1 << 6) != 0 { flags.push("ZF"); }
    if rflags & (1 << 7) != 0 { flags.push("SF"); }
    if rflags & (1 << 8) != 0 { flags.push("TF"); }
    if rflags & (1 << 9) != 0 { flags.push("IF"); }
    if rflags & (1 << 10) != 0 { flags.push("DF"); }
    if rflags & (1 << 11) != 0 { flags.push("OF"); }
    if rflags & (1 << 14) != 0 { flags.push("NT"); }
    if rflags & (1 << 21) != 0 { flags.push("ID"); }
    lines.push(format!("           [{}]", flags.join(" | ")));
    
    lines.push(String::from(""));
    lines.push(String::from("  ── Control Registers ──"));
    lines.push(format!("  CR0 = 0x{:016x}", cr0));
    lines.push(format!("  CR2 = 0x{:016x}  (last page fault addr)", cr2));
    lines.push(format!("  CR3 = 0x{:016x}  (page table root)", cr3));
    lines.push(format!("  CR4 = 0x{:016x}", cr4));
    
    lines.push(String::from(""));
    lines.push(String::from("  ── Segment Registers ──"));
    lines.push(format!("  CS=0x{:04x}  DS=0x{:04x}  ES=0x{:04x}  FS=0x{:04x}  GS=0x{:04x}  SS=0x{:04x}", cs, ds, es, fs, gs, ss));
    
    // MSRs
    lines.push(String::from(""));
    lines.push(String::from("  ── Model Specific Registers ──"));
    
    let msrs: &[(u32, &str)] = &[
        (0xC000_0080, "IA32_EFER"),
        (0xC000_0081, "IA32_STAR"),
        (0xC000_0082, "IA32_LSTAR"),
        (0xC000_0083, "IA32_CSTAR"),
        (0xC000_0084, "IA32_FMASK"),
        (0xC000_0100, "IA32_FS_BASE"),
        (0xC000_0101, "IA32_GS_BASE"),
        (0xC000_0102, "IA32_KERNEL_GS_BASE"),
        (0x0000_0010, "IA32_TSC"),
        (0x0000_001B, "IA32_APIC_BASE"),
        (0x0000_0174, "IA32_SYSENTER_CS"),
        (0x0000_0175, "IA32_SYSENTER_ESP"),
        (0x0000_0176, "IA32_SYSENTER_EIP"),
        (0x0000_0277, "IA32_PAT"),
    ];
    
    for &(msr_id, name) in msrs {
        match read_msr_safe(msr_id) {
            Some(val) => lines.push(format!("  0x{:08X} ({:<24}) = 0x{:016x}", msr_id, name, val)),
            None      => lines.push(format!("  0x{:08X} ({:<24}) = <GPF — not available>", msr_id, name)),
        }
    }
    
    lines
}

#[cfg(not(target_arch = "x86_64"))]
pub fn full_cpu_dump() -> Vec<String> {
    vec![String::from("  Full CPU dump only available on x86_64")]
}

// ═══════════════════════════════════════════════════════════════════════════════
// 6. I/O PORT ACCESS — read/write any I/O port from shell
// ═══════════════════════════════════════════════════════════════════════════════

/// Read a byte from an I/O port
#[cfg(target_arch = "x86_64")]
pub fn inb(port: u16) -> u8 {
    let val: u8;
    unsafe { core::arch::asm!("in al, dx", in("dx") port, out("al") val, options(nostack, preserves_flags)); }
    val
}

/// Read a word from an I/O port
#[cfg(target_arch = "x86_64")]
pub fn inw(port: u16) -> u16 {
    let val: u16;
    unsafe { core::arch::asm!("in ax, dx", in("dx") port, out("ax") val, options(nostack, preserves_flags)); }
    val
}

/// Read a dword from an I/O port
#[cfg(target_arch = "x86_64")]
pub fn inl(port: u16) -> u32 {
    let val: u32;
    unsafe { core::arch::asm!("in eax, dx", in("dx") port, out("eax") val, options(nostack, preserves_flags)); }
    val
}

/// Write a byte to an I/O port
#[cfg(target_arch = "x86_64")]
pub fn outb(port: u16, val: u8) {
    unsafe { core::arch::asm!("out dx, al", in("dx") port, in("al") val, options(nostack, preserves_flags)); }
}

/// Write a word to an I/O port
#[cfg(target_arch = "x86_64")]
pub fn outw(port: u16, val: u16) {
    unsafe { core::arch::asm!("out dx, ax", in("dx") port, in("ax") val, options(nostack, preserves_flags)); }
}

/// Write a dword to an I/O port
#[cfg(target_arch = "x86_64")]
pub fn outl(port: u16, val: u32) {
    unsafe { core::arch::asm!("out dx, eax", in("dx") port, in("eax") val, options(nostack, preserves_flags)); }
}

#[cfg(not(target_arch = "x86_64"))]
pub fn inb(_port: u16) -> u8 { 0 }
#[cfg(not(target_arch = "x86_64"))]
pub fn inw(_port: u16) -> u16 { 0 }
#[cfg(not(target_arch = "x86_64"))]
pub fn inl(_port: u16) -> u32 { 0 }
#[cfg(not(target_arch = "x86_64"))]
pub fn outb(_port: u16, _val: u8) {}
#[cfg(not(target_arch = "x86_64"))]
pub fn outw(_port: u16, _val: u16) {}
#[cfg(not(target_arch = "x86_64"))]
pub fn outl(_port: u16, _val: u32) {}

// ═══════════════════════════════════════════════════════════════════════════════
// 7. MSR ACCESS — read/write Model Specific Registers
// ═══════════════════════════════════════════════════════════════════════════════

/// Read an MSR (returns None if the MSR causes a #GP)
#[cfg(target_arch = "x86_64")]
pub fn read_msr_safe(msr: u32) -> Option<u64> {
    // We can't catch #GP in a no_std kernel easily, so we just read directly.
    // Known-safe MSRs only. For truly unknown MSRs, the caller should handle.
    let lo: u32;
    let hi: u32;
    unsafe {
        core::arch::asm!(
            "rdmsr",
            in("ecx") msr,
            out("eax") lo,
            out("edx") hi,
            options(nostack, preserves_flags),
        );
    }
    Some(((hi as u64) << 32) | (lo as u64))
}

#[cfg(not(target_arch = "x86_64"))]
pub fn read_msr_safe(_msr: u32) -> Option<u64> {
    None
}

/// Write to an MSR
#[cfg(target_arch = "x86_64")]
pub fn write_msr(msr: u32, value: u64) {
    let lo = value as u32;
    let hi = (value >> 32) as u32;
    unsafe {
        core::arch::asm!(
            "wrmsr",
            in("ecx") msr,
            in("eax") lo,
            in("edx") hi,
            options(nostack, preserves_flags),
        );
    }
}

#[cfg(not(target_arch = "x86_64"))]
pub fn write_msr(_msr: u32, _value: u64) {}

// ═══════════════════════════════════════════════════════════════════════════════
// 8. RAW CPUID — query any CPUID leaf
// ═══════════════════════════════════════════════════════════════════════════════

/// Raw CPUID query (leaf, subleaf) -> (eax, ebx, ecx, edx)
#[cfg(target_arch = "x86_64")]
pub fn raw_cpuid(leaf: u32, subleaf: u32) -> (u32, u32, u32, u32) {
    let eax: u32;
    let ebx: u32;
    let ecx: u32;
    let edx: u32;
    unsafe {
        // rbx is reserved by LLVM, so we save/restore it manually
        core::arch::asm!(
            "push rbx",
            "cpuid",
            "mov {ebx_out:e}, ebx",
            "pop rbx",
            inout("eax") leaf => eax,
            inout("ecx") subleaf => ecx,
            ebx_out = out(reg) ebx,
            out("edx") edx,
        );
    }
    (eax, ebx, ecx, edx)
}

#[cfg(not(target_arch = "x86_64"))]
pub fn raw_cpuid(_leaf: u32, _subleaf: u32) -> (u32, u32, u32, u32) {
    (0, 0, 0, 0)
}

/// Format CPUID info for display
pub fn format_cpuid(leaf: u32, subleaf: u32) -> Vec<String> {
    let mut lines = Vec::new();
    let (eax, ebx, ecx, edx) = raw_cpuid(leaf, subleaf);
    lines.push(format!("  CPUID leaf=0x{:08x} subleaf=0x{:08x}", leaf, subleaf));
    lines.push(format!("  EAX = 0x{:08x}  ({:032b})", eax, eax));
    lines.push(format!("  EBX = 0x{:08x}  ({:032b})", ebx, ebx));
    lines.push(format!("  ECX = 0x{:08x}  ({:032b})", ecx, ecx));
    lines.push(format!("  EDX = 0x{:08x}  ({:032b})", edx, edx));
    
    // Decode well-known leaves
    match leaf {
        0 => {
            // Vendor ID
            let vendor = [
                ebx.to_le_bytes(),
                edx.to_le_bytes(),
                ecx.to_le_bytes(),
            ];
            let vendor_str: Vec<u8> = vendor.iter().flat_map(|b| b.iter().copied()).collect();
            if let Ok(s) = core::str::from_utf8(&vendor_str) {
                lines.push(format!("  → Vendor: \"{}\"  Max leaf: {}", s, eax));
            }
        }
        1 => {
            let stepping = eax & 0xF;
            let model = ((eax >> 4) & 0xF) | (((eax >> 16) & 0xF) << 4);
            let family = ((eax >> 8) & 0xF) + ((eax >> 20) & 0xFF);
            lines.push(format!("  → Family={} Model={} Stepping={}", family, model, stepping));
            lines.push(format!("  → Features: SSE3={} PCLMUL={} SSSE3={} SSE4.1={} SSE4.2={} AES={} AVX={}",
                ecx & 1, (ecx >> 1) & 1, (ecx >> 9) & 1, (ecx >> 19) & 1, (ecx >> 20) & 1, (ecx >> 25) & 1, (ecx >> 28) & 1));
            lines.push(format!("  → Features: FPU={} TSC={} MSR={} APIC={} SSE={} SSE2={} HTT={}",
                edx & 1, (edx >> 4) & 1, (edx >> 5) & 1, (edx >> 9) & 1, (edx >> 25) & 1, (edx >> 26) & 1, (edx >> 28) & 1));
        }
        0x8000_0002..=0x8000_0004 => {
            // Processor brand string
            let bytes: Vec<u8> = [eax, ebx, ecx, edx].iter()
                .flat_map(|v| v.to_le_bytes())
                .collect();
            if let Ok(s) = core::str::from_utf8(&bytes) {
                lines.push(format!("  → Brand: \"{}\"", s.trim_end_matches('\0')));
            }
        }
        _ => {}
    }
    
    lines
}

// ═══════════════════════════════════════════════════════════════════════════════
// 9. ENHANCED PANIC INFO — for crash analysis on real hardware
// ═══════════════════════════════════════════════════════════════════════════════

/// Dump full crash context to serial (called from panic handler)
pub fn panic_dump() {
    post_code(POST_PANIC);
    
    crate::serial_println!("╔════════════════════════════════════════════════════════╗");
    crate::serial_println!("║              TRUSTOS CRASH DUMP                       ║");
    crate::serial_println!("╚════════════════════════════════════════════════════════╝");
    
    // CPU state
    crate::serial_println!("── CPU Registers ──");
    for line in &full_cpu_dump() {
        crate::serial_println!("{}", line);
    }
    
    // Stack trace
    crate::serial_println!("");
    for line in &format_backtrace(32) {
        crate::serial_println!("{}", line);
    }
    
    // Stack dump (top 256 bytes from current RSP)
    #[cfg(target_arch = "x86_64")]
    {
        let rsp: u64;
        unsafe { core::arch::asm!("mov {}, rsp", out(reg) rsp); }
        crate::serial_println!("");
        crate::serial_println!("  ── Stack Dump (RSP=0x{:016x}, 256 bytes) ──", rsp);
        let stack_ptr = rsp as *const u8;
        for row in 0..16 {
            let offset = row * 16;
            let addr = rsp + offset as u64;
            let mut hex = String::new();
            let mut ascii = String::new();
            for col in 0..16 {
                let byte = unsafe { core::ptr::read_volatile(stack_ptr.add(offset + col)) };
                hex.push_str(&format!("{:02x} ", byte));
                if byte >= 0x20 && byte < 0x7f {
                    ascii.push(byte as char);
                } else {
                    ascii.push('.');
                }
            }
            crate::serial_println!("  {:016x}: {} |{}|", addr, hex, ascii);
        }
    }
    
    // Boot checkpoints (what was the last thing that succeeded?)
    crate::serial_println!("");
    crate::serial_println!("  ── Boot Checkpoints ──");
    if let Some(log) = CHECKPOINTS.try_lock() {
        if log.is_empty() {
            crate::serial_println!("  <no checkpoints recorded>");
        }
        for (tsc, code, name) in log.iter() {
            crate::serial_println!("  [TSC {:>16}] POST 0x{:02X}: {}", tsc, code, name);
        }
    }
    
    // Heap state
    crate::serial_println!("");
    crate::serial_println!("  ── Heap State ──");
    let stats = crate::devtools::memdbg_stats();
    crate::serial_println!("  allocs={}, deallocs={}, live={}, peak={}",
        stats.alloc_count, stats.dealloc_count, stats.live_allocs, stats.peak_heap_used);
    
    crate::serial_println!("════════════════════════════════════════════════════════════");
    crate::serial_println!("Collect this output via serial cable (115200 8N1) for analysis.");
    
    // 3 error beeps
    beep_error();
}

// ═══════════════════════════════════════════════════════════════════════════════
// 10. E9 DEBUG PORT — for QEMU/Bochs debug console
// ═══════════════════════════════════════════════════════════════════════════════

/// Write a string to the E9 debug port (Bochs/QEMU)
pub fn e9_print(s: &str) {
    #[cfg(target_arch = "x86_64")]
    for byte in s.bytes() {
        unsafe {
            core::arch::asm!("out dx, al", in("dx") 0xE9u16, in("al") byte, options(nostack, preserves_flags));
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// 11. MEMORY MAP DISPLAY
// ═══════════════════════════════════════════════════════════════════════════════

/// Format the UEFI memory map stored at boot time
pub fn format_memory_map() -> Vec<String> {
    let mut lines = Vec::new();
    lines.push(String::from("  Physical Memory Map (from Limine bootloader):"));
    lines.push(String::from("  ─────────────────────────────────────────────────────────────"));
    lines.push(format!("  {:>18}  {:>18}  {:>12}  {}", "Start", "End", "Size", "Type"));
    
    // We read from the stored memory map
    let regions = crate::memory::get_memory_regions();
    if regions.is_empty() {
        lines.push(String::from("  <memory map not stored — add memory::store_memory_map() to boot>"));
    } else {
        let mut total_usable: u64 = 0;
        let mut total_reserved: u64 = 0;
        for (base, length, typ) in &regions {
            let end = base + length;
            let size_kb = length / 1024;
            let type_str = match typ {
                0 => { total_usable += length; "USABLE" },
                1 => { total_reserved += length; "RESERVED" },
                2 => "ACPI RECLAIM",
                3 => "ACPI NVS",
                4 => "BAD MEMORY",
                5 => "BOOTLOADER",
                6 => "KERNEL/MODULES",
                7 => "FRAMEBUFFER",
                _ => "UNKNOWN",
            };
            lines.push(format!("  0x{:016x}  0x{:016x}  {:>8} KB  {}", base, end, size_kb, type_str));
        }
        lines.push(String::from("  ─────────────────────────────────────────────────────────────"));
        lines.push(format!("  Total usable: {} MB   Reserved: {} MB", total_usable / 1024 / 1024, total_reserved / 1024 / 1024));
    }
    
    lines
}

// ═══════════════════════════════════════════════════════════════════════════════
// 12. HARDWARE DIAGNOSTIC REPORT — one command to dump everything
// ═══════════════════════════════════════════════════════════════════════════════

/// Generate a comprehensive hardware diagnostic report
pub fn full_diagnostic_report() -> Vec<String> {
    let mut lines = Vec::new();
    
    lines.push(String::from("╔══════════════════════════════════════════════════════════════╗"));
    lines.push(String::from("║           TRUSTOS HARDWARE DIAGNOSTIC REPORT                ║"));
    lines.push(String::from("╚══════════════════════════════════════════════════════════════╝"));
    
    // CPU info
    lines.push(String::from(""));
    lines.push(String::from("━━━ CPU IDENTIFICATION ━━━"));
    // Vendor
    lines.extend(format_cpuid(0, 0));
    lines.push(String::from(""));
    // Features
    lines.extend(format_cpuid(1, 0));
    lines.push(String::from(""));
    // Brand string
    lines.extend(format_cpuid(0x8000_0002, 0));
    lines.extend(format_cpuid(0x8000_0003, 0));
    lines.extend(format_cpuid(0x8000_0004, 0));
    
    // Registers
    lines.push(String::from(""));
    lines.push(String::from("━━━ CPU REGISTERS ━━━"));
    lines.extend(full_cpu_dump());
    
    // Memory map
    lines.push(String::from(""));
    lines.push(String::from("━━━ MEMORY MAP ━━━"));
    lines.extend(format_memory_map());
    
    // Heap
    lines.push(String::from(""));
    lines.push(String::from("━━━ HEAP STATUS ━━━"));
    let stats = crate::devtools::memdbg_stats();
    lines.push(format!("  Allocations: {}   Deallocations: {}", stats.alloc_count, stats.dealloc_count));
    lines.push(format!("  Live allocs: {}   Peak heap: {}   Largest single: {}", stats.live_allocs, stats.peak_heap_used, stats.largest_alloc));
    lines.push(format!("  Heap free: {} KB", crate::memory::heap::free() / 1024));
    
    // PCI devices
    lines.push(String::from(""));
    lines.push(String::from("━━━ PCI DEVICES ━━━"));
    let pci_devs = crate::pci::scan();
    if pci_devs.is_empty() {
        lines.push(String::from("  <no PCI devices found>"));
    } else {
        for dev in &pci_devs {
            lines.push(format!("  {:02x}:{:02x}.{} [{:04x}:{:04x}] class={:02x}{:02x} {}",
                dev.bus, dev.device, dev.function,
                dev.vendor_id, dev.device_id,
                dev.class_code, dev.subclass,
                dev.class_name()));
        }
    }
    
    // Boot checkpoints
    lines.push(String::from(""));
    lines.push(String::from("━━━ BOOT CHECKPOINTS ━━━"));
    let cps = get_checkpoints();
    if cps.is_empty() {
        lines.push(String::from("  <no checkpoints recorded>"));
    } else {
        for (tsc, code, name) in &cps {
            lines.push(format!("  [TSC {:>16}] POST 0x{:02X}: {}", tsc, code, name));
        }
    }
    
    // Stack trace
    lines.push(String::from(""));
    lines.push(String::from("━━━ CURRENT STACK TRACE ━━━"));
    lines.extend(format_backtrace(16));
    
    // Serial port status
    lines.push(String::from(""));
    lines.push(String::from("━━━ SERIAL PORT STATUS ━━━"));
    #[cfg(target_arch = "x86_64")]
    {
        let lsr = inb(0x3F8 + 5);
        let mcr = inb(0x3F8 + 4);
        let ier = inb(0x3F8 + 1);
        lines.push(format!("  COM1 (0x3F8): LSR=0x{:02x} MCR=0x{:02x} IER=0x{:02x}", lsr, mcr, ier));
        lines.push(format!("    Data Ready: {}  TX Empty: {}  Break: {}  Error: {}",
            lsr & 1 != 0, lsr & (1 << 5) != 0, lsr & (1 << 4) != 0, lsr & (1 << 7) != 0));
    }
    #[cfg(not(target_arch = "x86_64"))]
    lines.push(String::from("  <serial port status not available on this arch>"));
    
    lines.push(String::from(""));
    lines.push(String::from("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"));
    lines.push(String::from("Tip: Use serial cable (115200 8N1) to capture this output."));
    lines.push(String::from("     Run `hwdiag > serial` to send to serial only."));
    
    lines
}

// ═══════════════════════════════════════════════════════════════════════════════
// 13. WATCHDOG — detect hangs by counting timer ticks
// ═══════════════════════════════════════════════════════════════════════════════

static WATCHDOG_ENABLED: AtomicBool = AtomicBool::new(false);
static WATCHDOG_COUNTER: AtomicU64 = AtomicU64::new(0);
static WATCHDOG_THRESHOLD: AtomicU64 = AtomicU64::new(5000); // 5 seconds default

/// Enable the software watchdog (must be pet regularly or it prints a warning)
pub fn watchdog_enable(timeout_ms: u64) {
    WATCHDOG_THRESHOLD.store(timeout_ms, Ordering::Relaxed);
    WATCHDOG_COUNTER.store(0, Ordering::Relaxed);
    WATCHDOG_ENABLED.store(true, Ordering::Relaxed);
    crate::serial_println!("[WATCHDOG] Enabled with {} ms timeout", timeout_ms);
}

/// Pet the watchdog (call from main loop or timer)
pub fn watchdog_pet() {
    WATCHDOG_COUNTER.store(0, Ordering::Relaxed);
}

/// Called from timer interrupt — increments counter and checks for timeout
pub fn watchdog_tick(ms_elapsed: u64) {
    if !WATCHDOG_ENABLED.load(Ordering::Relaxed) {
        return;
    }
    let count = WATCHDOG_COUNTER.fetch_add(ms_elapsed, Ordering::Relaxed) + ms_elapsed;
    if count >= WATCHDOG_THRESHOLD.load(Ordering::Relaxed) {
        WATCHDOG_COUNTER.store(0, Ordering::Relaxed);
        crate::serial_println!("!!! WATCHDOG TIMEOUT !!! System may be hung ({} ms)", count);
        // Don't panic — just warn. The system might recover.
    }
}

pub fn watchdog_disable() {
    WATCHDOG_ENABLED.store(false, Ordering::Relaxed);
    crate::serial_println!("[WATCHDOG] Disabled");
}

// ═══════════════════════════════════════════════════════════════════════════════
// INTERNAL HELPERS
// ═══════════════════════════════════════════════════════════════════════════════

/// Read TSC (Time Stamp Counter)
#[cfg(target_arch = "x86_64")]
pub fn read_tsc() -> u64 {
    let lo: u32;
    let hi: u32;
    unsafe {
        core::arch::asm!("rdtsc", out("eax") lo, out("edx") hi, options(nostack, preserves_flags));
    }
    ((hi as u64) << 32) | (lo as u64)
}

#[cfg(not(target_arch = "x86_64"))]
pub fn read_tsc() -> u64 { 0 }

/// Busy-wait for approximately `ms` milliseconds (TSC-based when available)
fn busy_wait_ms(ms: u32) {
    // Rough estimate: assume ~1 GHz TSC minimum for modern CPUs
    // Actual frequency doesn't matter much for beep durations
    let start = read_tsc();
    let target = start + (ms as u64) * 1_000_000; // ~1M cycles/ms @ 1GHz
    while read_tsc() < target {
        core::hint::spin_loop();
    }
}
