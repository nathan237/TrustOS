//! ptrace - Process Trace
//!
//! Process debugging and tracing support. Allows a parent process
//! to observe and control the execution of another process.

use alloc::collections::BTreeMap;
use spin::Mutex;
use core::sync::atomic::{AtomicU32, Ordering};

/// ptrace request types (Linux compatible)
pub mod request {
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const PTRACE_TRACEME: u32 = 0;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const PTRACE_PEEKTEXT: u32 = 1;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const PTRACE_PEEKDATA: u32 = 2;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const PTRACE_PEEKUSER: u32 = 3;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const PTRACE_POKETEXT: u32 = 4;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const PTRACE_POKEDATA: u32 = 5;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const PTRACE_POKEUSER: u32 = 6;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const PTRACE_CONT: u32 = 7;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const PTRACE_KILL: u32 = 8;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const PTRACE_SINGLESTEP: u32 = 9;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const PTRACE_GETREGS: u32 = 12;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const PTRACE_SETREGS: u32 = 13;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const PTRACE_GETFPREGS: u32 = 14;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const PTRACE_SETFPREGS: u32 = 15;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const PTRACE_ATTACH: u32 = 16;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const PTRACE_DETACH: u32 = 17;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const PTRACE_GETFPXREGS: u32 = 18;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const PTRACE_SETFPXREGS: u32 = 19;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const PTRACE_SYSCALL: u32 = 24;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const PTRACE_SETOPTIONS: u32 = 0x4200;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const PTRACE_GETEVENTMSG: u32 = 0x4201;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const PTRACE_GETSIGINFO: u32 = 0x4202;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const PTRACE_SETSIGINFO: u32 = 0x4203;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const PTRACE_GETREGSET: u32 = 0x4204;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const PTRACE_SETREGSET: u32 = 0x4205;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const PTRACE_SEIZE: u32 = 0x4206;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const PTRACE_INTERRUPT: u32 = 0x4207;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const PTRACE_LISTEN: u32 = 0x4208;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const PTRACE_PEEKSIGINFO: u32 = 0x4209;
}

/// ptrace options
pub mod options {
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const PTRACE_O_TRACESYSGOOD: u32 = 0x00000001;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const PTRACE_O_TRACEFORK: u32 = 0x00000002;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const PTRACE_O_TRACEVFORK: u32 = 0x00000004;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const PTRACE_O_TRACECLONE: u32 = 0x00000008;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const PTRACE_O_TRACEEXEC: u32 = 0x00000010;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const PTRACE_O_TRACEVFORKDONE: u32 = 0x00000020;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const PTRACE_O_TRACEEXIT: u32 = 0x00000040;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const PTRACE_O_TRACESECCOMP: u32 = 0x00000080;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const PTRACE_O_EXITKILL: u32 = 0x00100000;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const PTRACE_O_SUSPEND_SECCOMP: u32 = 0x00200000;
}

/// ptrace events
pub mod events {
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const PTRACE_EVENT_FORK: u32 = 1;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const PTRACE_EVENT_VFORK: u32 = 2;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const PTRACE_EVENT_CLONE: u32 = 3;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const PTRACE_EVENT_EXECUTE: u32 = 4;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const PTRACE_EVENT_VFORK_DONE: u32 = 5;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const PTRACE_EVENT_EXIT: u32 = 6;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const PTRACE_EVENT_SECCOMP: u32 = 7;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const PTRACE_EVENT_STOP: u32 = 128;
}

/// Tracee state
#[derive(Clone, Copy, Debug, PartialEq)]
// Enumeration — a type that can be one of several variants.
pub enum TraceeState {
    /// Not being traced
    NotTraced,
    /// Running normally
    Running,
    /// Stopped at syscall entry
    SyscallEntry,
    /// Stopped at syscall exit
    SyscallExit,
    /// Stopped by signal
    SignalStop(u32),
    /// Stopped for event
    EventStop(u32),
    /// Single-stepping
    SingleStep,
    /// Seized but not stopped
    Seized,
}

/// Per-process trace state
#[derive(Clone)]
// Public structure — visible outside this module.
pub struct TraceState {
    /// Tracer PID (0 if not traced)
    pub tracer_pid: u32,
    /// Current state
    pub state: TraceeState,
    /// Options set by tracer
    pub options: u32,
    /// Event message
    pub event_msg: u64,
    /// Signal to deliver on continue
    pub pending_signal: u32,
}

// Implementation block — defines methods for the type above.
impl TraceState {
        // Public function — callable from other modules.
pub fn new() -> Self {
        Self {
            tracer_pid: 0,
            state: TraceeState::NotTraced,
            options: 0,
            event_msg: 0,
            pending_signal: 0,
        }
    }
    
        // Public function — callable from other modules.
pub fn is_traced(&self) -> bool {
        self.tracer_pid != 0
    }
}

/// x86_64 register set for ptrace
#[repr(C)]
// #[derive] — auto-generates trait implementations at compile time.
#[derive(Clone, Copy, Debug, Default)]
// Public structure — visible outside this module.
pub struct UserRegs {
    pub r15: u64,
    pub r14: u64,
    pub r13: u64,
    pub r12: u64,
    pub rbp: u64,
    pub rbx: u64,
    pub r11: u64,
    pub r10: u64,
    pub r9: u64,
    pub r8: u64,
    pub rax: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub orig_rax: u64,
    pub rip: u64,
    pub cs: u64,
    pub rflags: u64,
    pub rsp: u64,
    pub ss: u64,
    pub filesystem_base: u64,
    pub gs_base: u64,
    pub ds: u64,
    pub es: u64,
    pub fs: u64,
    pub gs: u64,
}

/// FPU register set
#[repr(C)]
// #[derive] — auto-generates trait implementations at compile time.
#[derive(Clone, Copy, Debug)]
// Public structure — visible outside this module.
pub struct FpRegs {
    pub cwd: u16,
    pub swd: u16,
    pub ftw: u16,
    pub fop: u16,
    pub rip: u64,
    pub rdp: u64,
    pub mxcsr: u32,
    pub mxcr_mask: u32,
    pub st_space: [u32; 32],   // 8 * 16 bytes for x87 registers
    pub xmm_space: [u32; 64],  // 16 * 16 bytes for XMM registers
    pub padding: [u32; 24],
}

// Trait implementation — fulfills a behavioral contract.
impl Default for FpRegs {
    fn default() -> Self {
        Self {
            cwd: 0x37F,  // Default FPU control word
            swd: 0,
            ftw: 0,
            fop: 0,
            rip: 0,
            rdp: 0,
            mxcsr: 0x1F80,  // Default MXCSR
            mxcr_mask: 0,
            st_space: [0; 32],
            xmm_space: [0; 64],
            padding: [0; 24],
        }
    }
}

/// Global trace state per process
static TRACE_STATES: Mutex<BTreeMap<u32, TraceState>> = Mutex::new(BTreeMap::new());

/// Number of active traces (for statistics)
static ACTIVE_TRACES: AtomicU32 = AtomicU32::new(0);

/// Initialize trace state for a new process
pub fn initialize_process(pid: u32) {
    TRACE_STATES.lock().insert(pid, TraceState::new());
}

/// Clean up trace state for exited process
pub fn cleanup_process(pid: u32) {
    let mut states = TRACE_STATES.lock();
    if let Some(state) = states.remove(&pid) {
        if state.is_traced() {
            ACTIVE_TRACES.fetch_sub(1, Ordering::Relaxed);
        }
    }
}

/// Main ptrace syscall handler
pub fn ptrace(request: u32, pid: u32, addr: u64, data: u64) -> Result<u64, i32> {
    use request::*;
    
        // Pattern matching — Rust's exhaustive branching construct.
match request {
        PTRACE_TRACEME => traceme(),
        PTRACE_PEEKTEXT | PTRACE_PEEKDATA => peek(pid, addr),
        PTRACE_PEEKUSER => peek_user(pid, addr),
        PTRACE_POKETEXT | PTRACE_POKEDATA => poke(pid, addr, data),
        PTRACE_POKEUSER => poke_user(pid, addr, data),
        PTRACE_CONT => cont(pid, data as u32),
        PTRACE_KILL => kill(pid),
        PTRACE_SINGLESTEP => singlestep(pid, data as u32),
        PTRACE_GETREGS => getregs(pid, data as *mut UserRegs),
        PTRACE_SETREGS => setregs(pid, data as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const UserRegs),
        PTRACE_GETFPREGS => getfpregs(pid, data as *mut FpRegs),
        PTRACE_SETFPREGS => setfpregs(pid, data as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const FpRegs),
        PTRACE_ATTACH => attach(pid),
        PTRACE_DETACH => detach(pid, data as u32),
        PTRACE_SYSCALL => syscall(pid, data as u32),
        PTRACE_SETOPTIONS => setoptions(pid, data as u32),
        PTRACE_GETEVENTMSG => geteventmsg(pid, data as *mut u64),
        PTRACE_SEIZE => seize(pid, data as u32),
        PTRACE_INTERRUPT => interrupt(pid),
        _ => Err(-22), // EINVAL
    }
}

/// PTRACE_TRACEME - allow parent to trace this process
fn traceme() -> Result<u64, i32> {
    let current_pid = crate::process::current_pid();
    let parent_pid = crate::process::parent_pid(current_pid).ok_or(-3)?; // ESRCH
    
    let mut states = TRACE_STATES.lock();
    let state = states.get_mut(&current_pid).ok_or(-3)?;
    
    if state.is_traced() {
        return Err(-1); // EPERM - already traced
    }
    
    state.tracer_pid = parent_pid;
    state.state = TraceeState::Running;
    ACTIVE_TRACES.fetch_add(1, Ordering::Relaxed);
    
    Ok(0)
}

/// PTRACE_ATTACH - attach to process
fn attach(pid: u32) -> Result<u64, i32> {
    let tracer_pid = crate::process::current_pid();
    
    // Cannot attach to self
    if pid == tracer_pid {
        return Err(-1); // EPERM
    }
    
    let mut states = TRACE_STATES.lock();
    let state = states.get_mut(&pid).ok_or(-3)?; // ESRCH
    
    if state.is_traced() {
        return Err(-1); // EPERM - already traced
    }
    
    // TODO: Check permissions (must be able to send SIGSTOP)
    
    state.tracer_pid = tracer_pid;
    state.state = TraceeState::SignalStop(crate::signals::sig::SIGSTOP);
    ACTIVE_TRACES.fetch_add(1, Ordering::Relaxed);
    
    // Stop the tracee
    crate::process::stop(pid);
    
    Ok(0)
}

/// PTRACE_SEIZE - attach without stopping
fn seize(pid: u32, options: u32) -> Result<u64, i32> {
    let tracer_pid = crate::process::current_pid();
    
    if pid == tracer_pid {
        return Err(-1); // EPERM
    }
    
    let mut states = TRACE_STATES.lock();
    let state = states.get_mut(&pid).ok_or(-3)?;
    
    if state.is_traced() {
        return Err(-1); // EPERM
    }
    
    state.tracer_pid = tracer_pid;
    state.state = TraceeState::Seized;
    state.options = options;
    ACTIVE_TRACES.fetch_add(1, Ordering::Relaxed);
    
    Ok(0)
}

/// PTRACE_DETACH - detach from process
fn detach(pid: u32, signal: u32) -> Result<u64, i32> {
    check_tracer(pid)?;
    
    let mut states = TRACE_STATES.lock();
    let state = states.get_mut(&pid).ok_or(-3)?;
    
    state.tracer_pid = 0;
    state.state = TraceeState::NotTraced;
    state.options = 0;
    ACTIVE_TRACES.fetch_sub(1, Ordering::Relaxed);
    
    drop(states);
    
    // Resume process, optionally delivering signal
    if signal != 0 {
        let _ = crate::signals::kill(pid, signal, crate::process::current_pid());
    }
    crate::process::resume(pid);
    
    Ok(0)
}

/// PTRACE_CONT - continue execution
fn cont(pid: u32, signal: u32) -> Result<u64, i32> {
    check_tracer(pid)?;
    
    let mut states = TRACE_STATES.lock();
    let state = states.get_mut(&pid).ok_or(-3)?;
    state.state = TraceeState::Running;
    state.pending_signal = signal;
    drop(states);
    
    crate::process::resume(pid);
    Ok(0)
}

/// PTRACE_SINGLESTEP - execute single instruction
fn singlestep(pid: u32, signal: u32) -> Result<u64, i32> {
    check_tracer(pid)?;
    
    let mut states = TRACE_STATES.lock();
    let state = states.get_mut(&pid).ok_or(-3)?;
    state.state = TraceeState::SingleStep;
    state.pending_signal = signal;
    drop(states);
    
    // Set trap flag (bit 8) in tracee's RFLAGS for single-step
    if let Some(mut ctx) = crate::process::get_context(pid) {
        ctx.rflags |= 1 << 8; // TF = Trap Flag
        let _ = crate::process::set_context(pid, &ctx);
    }
    
    crate::process::resume(pid);
    Ok(0)
}

/// PTRACE_SYSCALL - continue until syscall entry/exit
fn syscall(pid: u32, signal: u32) -> Result<u64, i32> {
    check_tracer(pid)?;
    
    let mut states = TRACE_STATES.lock();
    let state = states.get_mut(&pid).ok_or(-3)?;
    
    // Toggle between entry and exit
    state.state = // Pattern matching — Rust's exhaustive branching construct.
match state.state {
        TraceeState::SyscallEntry => TraceeState::SyscallExit,
        _ => TraceeState::SyscallEntry,
    };
    state.pending_signal = signal;
    drop(states);
    
    crate::process::resume(pid);
    Ok(0)
}

/// PTRACE_KILL - kill tracee
fn kill(pid: u32) -> Result<u64, i32> {
    check_tracer(pid)?;
    
    let mut states = TRACE_STATES.lock();
    states.remove(&pid);
    ACTIVE_TRACES.fetch_sub(1, Ordering::Relaxed);
    drop(states);
    
    crate::process::terminate(pid);
    Ok(0)
}

/// PTRACE_INTERRUPT - interrupt seized tracee
fn interrupt(pid: u32) -> Result<u64, i32> {
    check_tracer(pid)?;
    
    let mut states = TRACE_STATES.lock();
    let state = states.get_mut(&pid).ok_or(-3)?;
    
    if state.state != TraceeState::Seized {
        return Err(-22); // EINVAL - not seized
    }
    
    state.state = TraceeState::EventStop(events::PTRACE_EVENT_STOP);
    drop(states);
    
    crate::process::stop(pid);
    Ok(0)
}

/// PTRACE_SETOPTIONS - set trace options
fn setoptions(pid: u32, options: u32) -> Result<u64, i32> {
    check_tracer(pid)?;
    
    let mut states = TRACE_STATES.lock();
    let state = states.get_mut(&pid).ok_or(-3)?;
    state.options = options;
    
    Ok(0)
}

/// PTRACE_GETEVENTMSG - get event message
fn geteventmsg(pid: u32, msg_ptr: *mut u64) -> Result<u64, i32> {
    check_tracer(pid)?;
    
    let states = TRACE_STATES.lock();
    let state = states.get(&pid).ok_or(-3)?;
    
    // TODO: Copy to user space safely
    if !msg_ptr.is_null() {
                // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { *msg_ptr = state.event_msg; }
    }
    
    Ok(0)
}

/// PTRACE_PEEK* - read memory
fn peek(pid: u32, addr: u64) -> Result<u64, i32> {
    check_tracer(pid)?;
    
    // Read from tracee's address space
    let value = crate::memory::read_user_u64(pid, addr)?;
    Ok(value)
}

/// PTRACE_PEEKUSER - read from USER area
fn peek_user(pid: u32, offset: u64) -> Result<u64, i32> {
    check_tracer(pid)?;
    
    let ctx = crate::process::get_context(pid).ok_or(-3i32)?;
    // Linux USER area register offsets (in bytes, 8-byte aligned)
    let val = // Pattern matching — Rust's exhaustive branching construct.
match offset {
        0   => ctx.r15, 8   => ctx.r14, 16  => ctx.r13, 24  => ctx.r12,
        32  => ctx.rbp, 40  => ctx.rbx, 48  => ctx.r11, 56  => ctx.r10,
        64  => ctx.r9,  72  => ctx.r8,  80  => ctx.rax, 88  => ctx.rcx,
        96  => ctx.rdx, 104 => ctx.rsi, 112 => ctx.rdi, 120 => ctx.rax, // orig_rax
        128 => ctx.rip, 136 => ctx.cs,  144 => ctx.rflags,
        152 => ctx.rsp, 160 => ctx.ss,
        _ => return Err(-14), // EFAULT - invalid offset
    };
    Ok(val)
}

/// PTRACE_POKE* - write memory
fn poke(pid: u32, addr: u64, data: u64) -> Result<u64, i32> {
    check_tracer(pid)?;
    
    // Write to tracee's address space
    crate::memory::write_user_u64(pid, addr, data)?;
    Ok(0)
}

/// PTRACE_POKEUSER - write to USER area
fn poke_user(pid: u32, offset: u64, data: u64) -> Result<u64, i32> {
    check_tracer(pid)?;
    
    let mut ctx = crate::process::get_context(pid).ok_or(-3i32)?;
        // Pattern matching — Rust's exhaustive branching construct.
match offset {
        0   => ctx.r15 = data, 8   => ctx.r14 = data,
        16  => ctx.r13 = data, 24  => ctx.r12 = data,
        32  => ctx.rbp = data, 40  => ctx.rbx = data,
        48  => ctx.r11 = data, 56  => ctx.r10 = data,
        64  => ctx.r9 = data,  72  => ctx.r8 = data,
        80  => ctx.rax = data, 88  => ctx.rcx = data,
        96  => ctx.rdx = data, 104 => ctx.rsi = data,
        112 => ctx.rdi = data, 128 => ctx.rip = data,
        136 => ctx.cs = data,  144 => ctx.rflags = data,
        152 => ctx.rsp = data, 160 => ctx.ss = data,
        _ => return Err(-14), // EFAULT
    }
    crate::process::set_context(pid, &ctx).map_err(|_| -3i32)?;
    Ok(0)
}

/// PTRACE_GETREGS - get general registers
fn getregs(pid: u32, regs: *mut UserRegs) -> Result<u64, i32> {
    check_tracer(pid)?;
    
    if regs.is_null() {
        return Err(-14); // EFAULT
    }
    
    let ctx = crate::process::get_context(pid).ok_or(-3i32)?;
    let user_regs = UserRegs {
        r15: ctx.r15, r14: ctx.r14, r13: ctx.r13, r12: ctx.r12,
        rbp: ctx.rbp, rbx: ctx.rbx, r11: ctx.r11, r10: ctx.r10,
        r9: ctx.r9, r8: ctx.r8, rax: ctx.rax, rcx: ctx.rcx,
        rdx: ctx.rdx, rsi: ctx.rsi, rdi: ctx.rdi,
        orig_rax: ctx.rax, // orig_rax = syscall nr, use rax as fallback
        rip: ctx.rip, cs: ctx.cs, rflags: ctx.rflags,
        rsp: ctx.rsp, ss: ctx.ss,
        filesystem_base: 0, gs_base: 0, ds: 0, es: 0, fs: 0, gs: 0,
    };
        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { *regs = user_regs; }
    
    Ok(0)
}

/// PTRACE_SETREGS - set general registers
fn setregs(pid: u32, regs: *// Compile-time constant — evaluated at compilation, zero runtime cost.
const UserRegs) -> Result<u64, i32> {
    check_tracer(pid)?;
    
    if regs.is_null() {
        return Err(-14); // EFAULT
    }
    
    let user_regs = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { &*regs };
    let mut ctx = crate::process::get_context(pid).ok_or(-3i32)?;
    ctx.r15 = user_regs.r15; ctx.r14 = user_regs.r14;
    ctx.r13 = user_regs.r13; ctx.r12 = user_regs.r12;
    ctx.rbp = user_regs.rbp; ctx.rbx = user_regs.rbx;
    ctx.r11 = user_regs.r11; ctx.r10 = user_regs.r10;
    ctx.r9 = user_regs.r9; ctx.r8 = user_regs.r8;
    ctx.rax = user_regs.rax; ctx.rcx = user_regs.rcx;
    ctx.rdx = user_regs.rdx; ctx.rsi = user_regs.rsi;
    ctx.rdi = user_regs.rdi; ctx.rip = user_regs.rip;
    ctx.cs = user_regs.cs; ctx.rflags = user_regs.rflags;
    ctx.rsp = user_regs.rsp; ctx.ss = user_regs.ss;
    crate::process::set_context(pid, &ctx).map_err(|_| -3i32)?;
    
    Ok(0)
}

/// PTRACE_GETFPREGS - get FPU registers
fn getfpregs(pid: u32, regs: *mut FpRegs) -> Result<u64, i32> {
    check_tracer(pid)?;
    
    if regs.is_null() {
        return Err(-14); // EFAULT
    }
    
    let fp_regs = FpRegs::default();
        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { *regs = fp_regs; }
    
    Ok(0)
}

/// PTRACE_SETFPREGS - set FPU registers
fn setfpregs(pid: u32, regs: *// Compile-time constant — evaluated at compilation, zero runtime cost.
const FpRegs) -> Result<u64, i32> {
    check_tracer(pid)?;
    
    if regs.is_null() {
        return Err(-14); // EFAULT
    }
    
    // TODO: Set FPU registers in saved context
    Ok(0)
}

/// Verify caller is the tracer of the given process
fn check_tracer(pid: u32) -> Result<(), i32> {
    let current_pid = crate::process::current_pid();
    
    let states = TRACE_STATES.lock();
    let state = states.get(&pid).ok_or(-3)?; // ESRCH
    
    if state.tracer_pid != current_pid {
        return Err(-3); // ESRCH - not our tracee
    }
    
    Ok(())
}

/// Check if process is being traced
pub fn is_traced(pid: u32) -> bool {
    TRACE_STATES.lock()
        .get(&pid)
        .map(|s| s.is_traced())
        .unwrap_or(false)
}

/// Get tracer PID
pub fn get_tracer(pid: u32) -> Option<u32> {
    TRACE_STATES.lock()
        .get(&pid)
        .filter(|s| s.is_traced())
        .map(|s| s.tracer_pid)
}

/// Notify tracer of event
pub fn notify_event(pid: u32, event: u32, msg: u64) {
    let mut states = TRACE_STATES.lock();
    if let Some(state) = states.get_mut(&pid) {
        if state.is_traced() && (state.options & event_to_option(event)) != 0 {
            state.state = TraceeState::EventStop(event);
            state.event_msg = msg;
            
            // Stop tracee and wake tracer
            drop(states);
            crate::process::stop(pid);
            
            // Send SIGCHLD to tracer
            if let Some(tracer) = get_tracer(pid) {
                let _ = crate::signals::kill(tracer, crate::signals::sig::SIGCHLD, pid);
            }
        }
    }
}

/// Convert event to option bit
fn event_to_option(event: u32) -> u32 {
        // Pattern matching — Rust's exhaustive branching construct.
match event {
        events::PTRACE_EVENT_FORK => options::PTRACE_O_TRACEFORK,
        events::PTRACE_EVENT_VFORK => options::PTRACE_O_TRACEVFORK,
        events::PTRACE_EVENT_CLONE => options::PTRACE_O_TRACECLONE,
        events::PTRACE_EVENT_EXECUTE => options::PTRACE_O_TRACEEXEC,
        events::PTRACE_EVENT_EXIT => options::PTRACE_O_TRACEEXIT,
        _ => 0,
    }
}

/// Called at syscall entry
pub fn syscall_enter(pid: u32) -> bool {
    let mut states = TRACE_STATES.lock();
    if let Some(state) = states.get_mut(&pid) {
        if state.state == TraceeState::SyscallEntry {
            state.state = TraceeState::SignalStop(crate::signals::sig::SIGTRAP);
            drop(states);
            crate::process::stop(pid);
            return true;
        }
    }
    false
}

/// Called at syscall exit
pub fn syscall_exit(pid: u32) -> bool {
    let mut states = TRACE_STATES.lock();
    if let Some(state) = states.get_mut(&pid) {
        if state.state == TraceeState::SyscallExit {
            state.state = TraceeState::SignalStop(crate::signals::sig::SIGTRAP);
            drop(states);
            crate::process::stop(pid);
            return true;
        }
    }
    false
}

/// Get number of active traces
pub fn active_count() -> u32 {
    ACTIVE_TRACES.load(Ordering::Relaxed)
}
