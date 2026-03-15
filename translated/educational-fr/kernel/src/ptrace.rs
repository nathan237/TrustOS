//! ptrace - Process Trace
//!
//! Process debugging and tracing support. Allows a parent process
//! to observe and control the execution of another process.

use alloc::collections::BTreeMap;
use spin::Mutex;
use core::sync::atomic::{AtomicU32, Ordering};

/// ptrace request types (Linux compatible)
pub mod request {
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const PTRACE_TRACEME: u32 = 0;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const PTRACE_PEEKTEXT: u32 = 1;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const PTRACE_PEEKDATA: u32 = 2;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const PTRACE_PEEKUSER: u32 = 3;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const PTRACE_POKETEXT: u32 = 4;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const PTRACE_POKEDATA: u32 = 5;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const PTRACE_POKEUSER: u32 = 6;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const PTRACE_CONT: u32 = 7;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const PTRACE_KILL: u32 = 8;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const PTRACE_SINGLESTEP: u32 = 9;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const PTRACE_GETREGS: u32 = 12;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const PTRACE_SETREGS: u32 = 13;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const PTRACE_GETFPREGS: u32 = 14;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const PTRACE_SETFPREGS: u32 = 15;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const PTRACE_ATTACH: u32 = 16;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const PTRACE_DETACH: u32 = 17;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const PTRACE_GETFPXREGS: u32 = 18;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const PTRACE_SETFPXREGS: u32 = 19;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const PTRACE_SYSCALL: u32 = 24;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const PTRACE_SETOPTIONS: u32 = 0x4200;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const PTRACE_GETEVENTMSG: u32 = 0x4201;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const PTRACE_GETSIGINFO: u32 = 0x4202;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const PTRACE_SETSIGINFO: u32 = 0x4203;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const PTRACE_GETREGSET: u32 = 0x4204;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const PTRACE_SETREGSET: u32 = 0x4205;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const PTRACE_SEIZE: u32 = 0x4206;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const PTRACE_INTERRUPT: u32 = 0x4207;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const PTRACE_LISTEN: u32 = 0x4208;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const PTRACE_PEEKSIGINFO: u32 = 0x4209;
}

/// ptrace options
pub mod options {
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const PTRACE_O_TRACESYSGOOD: u32 = 0x00000001;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const PTRACE_O_TRACEFORK: u32 = 0x00000002;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const PTRACE_O_TRACEVFORK: u32 = 0x00000004;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const PTRACE_O_TRACECLONE: u32 = 0x00000008;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const PTRACE_O_TRACEEXEC: u32 = 0x00000010;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const PTRACE_O_TRACEVFORKDONE: u32 = 0x00000020;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const PTRACE_O_TRACEEXIT: u32 = 0x00000040;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const PTRACE_O_TRACESECCOMP: u32 = 0x00000080;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const PTRACE_O_EXITKILL: u32 = 0x00100000;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const PTRACE_O_SUSPEND_SECCOMP: u32 = 0x00200000;
}

/// ptrace events
pub mod events {
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const PTRACE_EVENT_FORK: u32 = 1;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const PTRACE_EVENT_VFORK: u32 = 2;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const PTRACE_EVENT_CLONE: u32 = 3;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const PTRACE_EVENT_EXECUTE: u32 = 4;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const PTRACE_EVENT_VFORK_DONE: u32 = 5;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const PTRACE_EVENT_EXIT: u32 = 6;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const PTRACE_EVENT_SECCOMP: u32 = 7;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const PTRACE_EVENT_STOP: u32 = 128;
}

/// Tracee state
#[derive(Clone, Copy, Debug, PartialEq)]
// Énumération — un type qui peut être l'une de plusieurs variantes.
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
// Structure publique — visible à l'extérieur de ce module.
pub struct TraceState {
    /// Tracer PID (0 if not traced)
    pub tracer_pid: u32,
    /// Current state
    pub state: TraceeState,
    /// Options set by tracer
    pub options: u32,
    /// Event message
    pub event_message: u64,
    /// Signal to deliver on continue
    pub pending_signal: u32,
}

// Bloc d'implémentation — définit les méthodes du type ci-dessus.
impl TraceState {
        // Fonction publique — appelable depuis d'autres modules.
pub fn new() -> Self {
        Self {
            tracer_pid: 0,
            state: TraceeState::NotTraced,
            options: 0,
            event_message: 0,
            pending_signal: 0,
        }
    }
    
        // Fonction publique — appelable depuis d'autres modules.
pub fn is_traced(&self) -> bool {
        self.tracer_pid != 0
    }
}

/// x86_64 register set for ptrace
#[repr(C)]
// #[derive] — génère automatiquement les implémentations de traits à la compilation.
#[derive(Clone, Copy, Debug, Default)]
// Structure publique — visible à l'extérieur de ce module.
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
// #[derive] — génère automatiquement les implémentations de traits à la compilation.
#[derive(Clone, Copy, Debug)]
// Structure publique — visible à l'extérieur de ce module.
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

// Implémentation de trait — remplit un contrat comportemental.
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
pub fn ptrace(request: u32, pid: u32, address: u64, data: u64) -> Result<u64, i32> {
    use request::*;
    
        // Correspondance de motifs — branchement exhaustif de Rust.
match request {
        PTRACE_TRACEME => traceme(),
        PTRACE_PEEKTEXT | PTRACE_PEEKDATA => peek(pid, address),
        PTRACE_PEEKUSER => peek_user(pid, address),
        PTRACE_POKETEXT | PTRACE_POKEDATA => poke(pid, address, data),
        PTRACE_POKEUSER => poke_user(pid, address, data),
        PTRACE_CONT => cont(pid, data as u32),
        PTRACE_KILL => kill(pid),
        PTRACE_SINGLESTEP => singlestep(pid, data as u32),
        PTRACE_GETREGS => getregs(pid, data as *mut UserRegs),
        PTRACE_SETREGS => setregs(pid, data as *// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const UserRegs),
        PTRACE_GETFPREGS => getfpregs(pid, data as *mut FpRegs),
        PTRACE_SETFPREGS => setfpregs(pid, data as *// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
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
    if let Some(mut context) = crate::process::get_context(pid) {
        context.rflags |= 1 << 8; // TF = Trap Flag
        let _ = crate::process::set_context(pid, &context);
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
    state.state = // Correspondance de motifs — branchement exhaustif de Rust.
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
fn geteventmsg(pid: u32, message_pointer: *mut u64) -> Result<u64, i32> {
    check_tracer(pid)?;
    
    let states = TRACE_STATES.lock();
    let state = states.get(&pid).ok_or(-3)?;
    
    // TODO: Copy to user space safely
    if !message_pointer.is_null() {
                // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { *message_pointer = state.event_message; }
    }
    
    Ok(0)
}

/// PTRACE_PEEK* - read memory
fn peek(pid: u32, address: u64) -> Result<u64, i32> {
    check_tracer(pid)?;
    
    // Read from tracee's address space
    let value = crate::memory::read_user_u64(pid, address)?;
    Ok(value)
}

/// PTRACE_PEEKUSER - read from USER area
fn peek_user(pid: u32, offset: u64) -> Result<u64, i32> {
    check_tracer(pid)?;
    
    let context = crate::process::get_context(pid).ok_or(-3i32)?;
    // Linux USER area register offsets (in bytes, 8-byte aligned)
    let value = // Correspondance de motifs — branchement exhaustif de Rust.
match offset {
        0   => context.r15, 8   => context.r14, 16  => context.r13, 24  => context.r12,
        32  => context.rbp, 40  => context.rbx, 48  => context.r11, 56  => context.r10,
        64  => context.r9,  72  => context.r8,  80  => context.rax, 88  => context.rcx,
        96  => context.rdx, 104 => context.rsi, 112 => context.rdi, 120 => context.rax, // orig_rax
        128 => context.rip, 136 => context.cs,  144 => context.rflags,
        152 => context.rsp, 160 => context.ss,
        _ => return Err(-14), // EFAULT - invalid offset
    };
    Ok(value)
}

/// PTRACE_POKE* - write memory
fn poke(pid: u32, address: u64, data: u64) -> Result<u64, i32> {
    check_tracer(pid)?;
    
    // Write to tracee's address space
    crate::memory::write_user_u64(pid, address, data)?;
    Ok(0)
}

/// PTRACE_POKEUSER - write to USER area
fn poke_user(pid: u32, offset: u64, data: u64) -> Result<u64, i32> {
    check_tracer(pid)?;
    
    let mut context = crate::process::get_context(pid).ok_or(-3i32)?;
        // Correspondance de motifs — branchement exhaustif de Rust.
match offset {
        0   => context.r15 = data, 8   => context.r14 = data,
        16  => context.r13 = data, 24  => context.r12 = data,
        32  => context.rbp = data, 40  => context.rbx = data,
        48  => context.r11 = data, 56  => context.r10 = data,
        64  => context.r9 = data,  72  => context.r8 = data,
        80  => context.rax = data, 88  => context.rcx = data,
        96  => context.rdx = data, 104 => context.rsi = data,
        112 => context.rdi = data, 128 => context.rip = data,
        136 => context.cs = data,  144 => context.rflags = data,
        152 => context.rsp = data, 160 => context.ss = data,
        _ => return Err(-14), // EFAULT
    }
    crate::process::set_context(pid, &context).map_error(|_| -3i32)?;
    Ok(0)
}

/// PTRACE_GETREGS - get general registers
fn getregs(pid: u32, regs: *mut UserRegs) -> Result<u64, i32> {
    check_tracer(pid)?;
    
    if regs.is_null() {
        return Err(-14); // EFAULT
    }
    
    let context = crate::process::get_context(pid).ok_or(-3i32)?;
    let user_regs = UserRegs {
        r15: context.r15, r14: context.r14, r13: context.r13, r12: context.r12,
        rbp: context.rbp, rbx: context.rbx, r11: context.r11, r10: context.r10,
        r9: context.r9, r8: context.r8, rax: context.rax, rcx: context.rcx,
        rdx: context.rdx, rsi: context.rsi, rdi: context.rdi,
        orig_rax: context.rax, // orig_rax = syscall nr, use rax as fallback
        rip: context.rip, cs: context.cs, rflags: context.rflags,
        rsp: context.rsp, ss: context.ss,
        filesystem_base: 0, gs_base: 0, ds: 0, es: 0, fs: 0, gs: 0,
    };
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { *regs = user_regs; }
    
    Ok(0)
}

/// PTRACE_SETREGS - set general registers
fn setregs(pid: u32, regs: *// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const UserRegs) -> Result<u64, i32> {
    check_tracer(pid)?;
    
    if regs.is_null() {
        return Err(-14); // EFAULT
    }
    
    let user_regs = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { &*regs };
    let mut context = crate::process::get_context(pid).ok_or(-3i32)?;
    context.r15 = user_regs.r15; context.r14 = user_regs.r14;
    context.r13 = user_regs.r13; context.r12 = user_regs.r12;
    context.rbp = user_regs.rbp; context.rbx = user_regs.rbx;
    context.r11 = user_regs.r11; context.r10 = user_regs.r10;
    context.r9 = user_regs.r9; context.r8 = user_regs.r8;
    context.rax = user_regs.rax; context.rcx = user_regs.rcx;
    context.rdx = user_regs.rdx; context.rsi = user_regs.rsi;
    context.rdi = user_regs.rdi; context.rip = user_regs.rip;
    context.cs = user_regs.cs; context.rflags = user_regs.rflags;
    context.rsp = user_regs.rsp; context.ss = user_regs.ss;
    crate::process::set_context(pid, &context).map_error(|_| -3i32)?;
    
    Ok(0)
}

/// PTRACE_GETFPREGS - get FPU registers
fn getfpregs(pid: u32, regs: *mut FpRegs) -> Result<u64, i32> {
    check_tracer(pid)?;
    
    if regs.is_null() {
        return Err(-14); // EFAULT
    }
    
    let fp_regs = FpRegs::default();
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { *regs = fp_regs; }
    
    Ok(0)
}

/// PTRACE_SETFPREGS - set FPU registers
fn setfpregs(pid: u32, regs: *// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
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
pub fn notify_event(pid: u32, event: u32, message: u64) {
    let mut states = TRACE_STATES.lock();
    if let Some(state) = states.get_mut(&pid) {
        if state.is_traced() && (state.options & event_to_option(event)) != 0 {
            state.state = TraceeState::EventStop(event);
            state.event_message = message;
            
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
        // Correspondance de motifs — branchement exhaustif de Rust.
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
