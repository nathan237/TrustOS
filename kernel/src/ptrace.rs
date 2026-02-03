//! ptrace - Process Trace
//!
//! Process debugging and tracing support. Allows a parent process
//! to observe and control the execution of another process.

use alloc::collections::BTreeMap;
use spin::Mutex;
use core::sync::atomic::{AtomicU32, Ordering};

/// ptrace request types (Linux compatible)
pub mod request {
    pub const PTRACE_TRACEME: u32 = 0;
    pub const PTRACE_PEEKTEXT: u32 = 1;
    pub const PTRACE_PEEKDATA: u32 = 2;
    pub const PTRACE_PEEKUSER: u32 = 3;
    pub const PTRACE_POKETEXT: u32 = 4;
    pub const PTRACE_POKEDATA: u32 = 5;
    pub const PTRACE_POKEUSER: u32 = 6;
    pub const PTRACE_CONT: u32 = 7;
    pub const PTRACE_KILL: u32 = 8;
    pub const PTRACE_SINGLESTEP: u32 = 9;
    pub const PTRACE_GETREGS: u32 = 12;
    pub const PTRACE_SETREGS: u32 = 13;
    pub const PTRACE_GETFPREGS: u32 = 14;
    pub const PTRACE_SETFPREGS: u32 = 15;
    pub const PTRACE_ATTACH: u32 = 16;
    pub const PTRACE_DETACH: u32 = 17;
    pub const PTRACE_GETFPXREGS: u32 = 18;
    pub const PTRACE_SETFPXREGS: u32 = 19;
    pub const PTRACE_SYSCALL: u32 = 24;
    pub const PTRACE_SETOPTIONS: u32 = 0x4200;
    pub const PTRACE_GETEVENTMSG: u32 = 0x4201;
    pub const PTRACE_GETSIGINFO: u32 = 0x4202;
    pub const PTRACE_SETSIGINFO: u32 = 0x4203;
    pub const PTRACE_GETREGSET: u32 = 0x4204;
    pub const PTRACE_SETREGSET: u32 = 0x4205;
    pub const PTRACE_SEIZE: u32 = 0x4206;
    pub const PTRACE_INTERRUPT: u32 = 0x4207;
    pub const PTRACE_LISTEN: u32 = 0x4208;
    pub const PTRACE_PEEKSIGINFO: u32 = 0x4209;
}

/// ptrace options
pub mod options {
    pub const PTRACE_O_TRACESYSGOOD: u32 = 0x00000001;
    pub const PTRACE_O_TRACEFORK: u32 = 0x00000002;
    pub const PTRACE_O_TRACEVFORK: u32 = 0x00000004;
    pub const PTRACE_O_TRACECLONE: u32 = 0x00000008;
    pub const PTRACE_O_TRACEEXEC: u32 = 0x00000010;
    pub const PTRACE_O_TRACEVFORKDONE: u32 = 0x00000020;
    pub const PTRACE_O_TRACEEXIT: u32 = 0x00000040;
    pub const PTRACE_O_TRACESECCOMP: u32 = 0x00000080;
    pub const PTRACE_O_EXITKILL: u32 = 0x00100000;
    pub const PTRACE_O_SUSPEND_SECCOMP: u32 = 0x00200000;
}

/// ptrace events
pub mod events {
    pub const PTRACE_EVENT_FORK: u32 = 1;
    pub const PTRACE_EVENT_VFORK: u32 = 2;
    pub const PTRACE_EVENT_CLONE: u32 = 3;
    pub const PTRACE_EVENT_EXEC: u32 = 4;
    pub const PTRACE_EVENT_VFORK_DONE: u32 = 5;
    pub const PTRACE_EVENT_EXIT: u32 = 6;
    pub const PTRACE_EVENT_SECCOMP: u32 = 7;
    pub const PTRACE_EVENT_STOP: u32 = 128;
}

/// Tracee state
#[derive(Clone, Copy, Debug, PartialEq)]
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

impl TraceState {
    pub fn new() -> Self {
        Self {
            tracer_pid: 0,
            state: TraceeState::NotTraced,
            options: 0,
            event_msg: 0,
            pending_signal: 0,
        }
    }
    
    pub fn is_traced(&self) -> bool {
        self.tracer_pid != 0
    }
}

/// x86_64 register set for ptrace
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
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
    pub fs_base: u64,
    pub gs_base: u64,
    pub ds: u64,
    pub es: u64,
    pub fs: u64,
    pub gs: u64,
}

/// FPU register set
#[repr(C)]
#[derive(Clone, Copy, Debug)]
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
pub fn init_process(pid: u32) {
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
        PTRACE_SETREGS => setregs(pid, data as *const UserRegs),
        PTRACE_GETFPREGS => getfpregs(pid, data as *mut FpRegs),
        PTRACE_SETFPREGS => setfpregs(pid, data as *const FpRegs),
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
    
    // Set trap flag in RFLAGS
    // TODO: Actually modify the tracee's RFLAGS
    
    crate::process::resume(pid);
    Ok(0)
}

/// PTRACE_SYSCALL - continue until syscall entry/exit
fn syscall(pid: u32, signal: u32) -> Result<u64, i32> {
    check_tracer(pid)?;
    
    let mut states = TRACE_STATES.lock();
    let state = states.get_mut(&pid).ok_or(-3)?;
    
    // Toggle between entry and exit
    state.state = match state.state {
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
    
    // USER area contains registers at specific offsets
    // TODO: Map offset to register
    Ok(0)
}

/// PTRACE_POKE* - write memory
fn poke(pid: u32, addr: u64, data: u64) -> Result<u64, i32> {
    check_tracer(pid)?;
    
    // Write to tracee's address space
    crate::memory::write_user_u64(pid, addr, data)?;
    Ok(0)
}

/// PTRACE_POKEUSER - write to USER area
fn poke_user(pid: u32, _offset: u64, _data: u64) -> Result<u64, i32> {
    check_tracer(pid)?;
    
    // TODO: Map offset to register and update
    Ok(0)
}

/// PTRACE_GETREGS - get general registers
fn getregs(pid: u32, regs: *mut UserRegs) -> Result<u64, i32> {
    check_tracer(pid)?;
    
    if regs.is_null() {
        return Err(-14); // EFAULT
    }
    
    // TODO: Get registers from saved context
    let user_regs = UserRegs::default();
    unsafe { *regs = user_regs; }
    
    Ok(0)
}

/// PTRACE_SETREGS - set general registers
fn setregs(pid: u32, regs: *const UserRegs) -> Result<u64, i32> {
    check_tracer(pid)?;
    
    if regs.is_null() {
        return Err(-14); // EFAULT
    }
    
    // TODO: Set registers in saved context
    Ok(0)
}

/// PTRACE_GETFPREGS - get FPU registers
fn getfpregs(pid: u32, regs: *mut FpRegs) -> Result<u64, i32> {
    check_tracer(pid)?;
    
    if regs.is_null() {
        return Err(-14); // EFAULT
    }
    
    let fp_regs = FpRegs::default();
    unsafe { *regs = fp_regs; }
    
    Ok(0)
}

/// PTRACE_SETFPREGS - set FPU registers
fn setfpregs(pid: u32, regs: *const FpRegs) -> Result<u64, i32> {
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
    match event {
        events::PTRACE_EVENT_FORK => options::PTRACE_O_TRACEFORK,
        events::PTRACE_EVENT_VFORK => options::PTRACE_O_TRACEVFORK,
        events::PTRACE_EVENT_CLONE => options::PTRACE_O_TRACECLONE,
        events::PTRACE_EVENT_EXEC => options::PTRACE_O_TRACEEXEC,
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
