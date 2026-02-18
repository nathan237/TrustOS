//! POSIX Signals
//!
//! Signal handling for process control and inter-process communication.
//! Implements Linux-compatible signal semantics.

use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};
use spin::Mutex;

/// Signal numbers (Linux compatible)
pub mod sig {
    pub const SIGHUP: u32 = 1;
    pub const SIGINT: u32 = 2;
    pub const SIGQUIT: u32 = 3;
    pub const SIGILL: u32 = 4;
    pub const SIGTRAP: u32 = 5;
    pub const SIGABRT: u32 = 6;
    pub const SIGBUS: u32 = 7;
    pub const SIGFPE: u32 = 8;
    pub const SIGKILL: u32 = 9;  // Cannot be caught or ignored
    pub const SIGUSR1: u32 = 10;
    pub const SIGSEGV: u32 = 11;
    pub const SIGUSR2: u32 = 12;
    pub const SIGPIPE: u32 = 13;
    pub const SIGALRM: u32 = 14;
    pub const SIGTERM: u32 = 15;
    pub const SIGSTKFLT: u32 = 16;
    pub const SIGCHLD: u32 = 17;
    pub const SIGCONT: u32 = 18;
    pub const SIGSTOP: u32 = 19; // Cannot be caught or ignored
    pub const SIGTSTP: u32 = 20;
    pub const SIGTTIN: u32 = 21;
    pub const SIGTTOU: u32 = 22;
    pub const SIGURG: u32 = 23;
    pub const SIGXCPU: u32 = 24;
    pub const SIGXFSZ: u32 = 25;
    pub const SIGVTALRM: u32 = 26;
    pub const SIGPROF: u32 = 27;
    pub const SIGWINCH: u32 = 28;
    pub const SIGIO: u32 = 29;
    pub const SIGPWR: u32 = 30;
    pub const SIGSYS: u32 = 31;
    
    // Real-time signals (32-64)
    pub const SIGRTMIN: u32 = 32;
    pub const SIGRTMAX: u32 = 64;
    
    pub const MAX_SIGNALS: usize = 65;
}

/// Signal action flags
pub mod sa_flags {
    pub const SA_NOCLDSTOP: u64 = 0x00000001;
    pub const SA_NOCLDWAIT: u64 = 0x00000002;
    pub const SA_SIGINFO: u64 = 0x00000004;
    pub const SA_ONSTACK: u64 = 0x08000000;
    pub const SA_RESTART: u64 = 0x10000000;
    pub const SA_NODEFER: u64 = 0x40000000;
    pub const SA_RESETHAND: u64 = 0x80000000;
    pub const SA_RESTORER: u64 = 0x04000000;
}

/// Special signal handlers
pub const SIG_DFL: u64 = 0;  // Default action
pub const SIG_IGN: u64 = 1;  // Ignore signal
pub const SIG_ERR: u64 = u64::MAX;

/// Signal action structure (Linux compatible)
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct SigAction {
    /// Handler function pointer or SIG_DFL/SIG_IGN
    pub sa_handler: u64,
    /// Flags
    pub sa_flags: u64,
    /// Signal restorer (for returning from handler)
    pub sa_restorer: u64,
    /// Signal mask during handler
    pub sa_mask: u64,
}

impl Default for SigAction {
    fn default() -> Self {
        Self {
            sa_handler: SIG_DFL,
            sa_flags: 0,
            sa_restorer: 0,
            sa_mask: 0,
        }
    }
}

/// Pending signal info
#[derive(Clone, Debug)]
pub struct PendingSignal {
    pub signo: u32,
    pub sender_pid: u32,
    pub timestamp: u64,
}

/// Per-process signal state
pub struct SignalState {
    /// Signal actions for each signal
    pub actions: [SigAction; sig::MAX_SIGNALS],
    /// Pending signals bitmask
    pub pending: AtomicU64,
    /// Blocked signals bitmask
    pub blocked: AtomicU64,
    /// Pending signal queue (for siginfo)
    pub pending_queue: Vec<PendingSignal>,
}

impl SignalState {
    pub fn new() -> Self {
        Self {
            actions: [SigAction::default(); sig::MAX_SIGNALS],
            pending: AtomicU64::new(0),
            blocked: AtomicU64::new(0),
            pending_queue: Vec::new(),
        }
    }
    
    /// Set signal action
    pub fn set_action(&mut self, signo: u32, action: SigAction) -> Result<SigAction, i32> {
        if signo == 0 || signo as usize >= sig::MAX_SIGNALS {
            return Err(-22); // EINVAL
        }
        
        // Cannot change SIGKILL or SIGSTOP
        if signo == sig::SIGKILL || signo == sig::SIGSTOP {
            return Err(-22); // EINVAL
        }
        
        let old = self.actions[signo as usize];
        self.actions[signo as usize] = action;
        Ok(old)
    }
    
    /// Get signal action
    pub fn get_action(&self, signo: u32) -> Option<&SigAction> {
        if signo as usize >= sig::MAX_SIGNALS {
            return None;
        }
        Some(&self.actions[signo as usize])
    }
    
    /// Post a signal to this process
    pub fn post_signal(&mut self, signo: u32, sender_pid: u32) {
        if signo == 0 || signo as usize >= sig::MAX_SIGNALS {
            return;
        }
        
        // Set pending bit
        self.pending.fetch_or(1 << signo, Ordering::SeqCst);
        
        // Add to queue
        self.pending_queue.push(PendingSignal {
            signo,
            sender_pid,
            timestamp: crate::time::now_ns(),
        });
    }
    
    /// Check if signal is pending
    pub fn is_pending(&self, signo: u32) -> bool {
        if signo as usize >= sig::MAX_SIGNALS {
            return false;
        }
        (self.pending.load(Ordering::Relaxed) & (1 << signo)) != 0
    }
    
    /// Check if signal is blocked
    pub fn is_blocked(&self, signo: u32) -> bool {
        if signo as usize >= sig::MAX_SIGNALS {
            return false;
        }
        // SIGKILL and SIGSTOP cannot be blocked
        if signo == sig::SIGKILL || signo == sig::SIGSTOP {
            return false;
        }
        (self.blocked.load(Ordering::Relaxed) & (1 << signo)) != 0
    }
    
    /// Get next deliverable signal
    pub fn get_deliverable(&self) -> Option<u32> {
        let pending = self.pending.load(Ordering::Relaxed);
        let blocked = self.blocked.load(Ordering::Relaxed);
        let deliverable = pending & !blocked;
        
        if deliverable == 0 {
            return None;
        }
        
        // Find first set bit
        Some(deliverable.trailing_zeros())
    }
    
    /// Clear pending signal
    pub fn clear_pending(&mut self, signo: u32) {
        if signo as usize >= sig::MAX_SIGNALS {
            return;
        }
        self.pending.fetch_and(!(1 << signo), Ordering::SeqCst);
        self.pending_queue.retain(|s| s.signo != signo);
    }
    
    /// Block signals
    pub fn block(&self, mask: u64) {
        self.blocked.fetch_or(mask, Ordering::SeqCst);
    }
    
    /// Unblock signals
    pub fn unblock(&self, mask: u64) {
        self.blocked.fetch_and(!mask, Ordering::SeqCst);
    }
    
    /// Set blocked mask
    pub fn set_blocked(&self, mask: u64) {
        // Cannot block SIGKILL or SIGSTOP
        let mask = mask & !((1 << sig::SIGKILL) | (1 << sig::SIGSTOP));
        self.blocked.store(mask, Ordering::SeqCst);
    }
}

/// Global signal state per process
static SIGNAL_STATES: Mutex<BTreeMap<u32, SignalState>> = Mutex::new(BTreeMap::new());

/// Initialize signal handling for a process
pub fn init_process(pid: u32) {
    SIGNAL_STATES.lock().insert(pid, SignalState::new());
}

/// Clean up signal state for exited process
pub fn cleanup_process(pid: u32) {
    SIGNAL_STATES.lock().remove(&pid);
}

/// Send signal to process
pub fn kill(target_pid: u32, signo: u32, sender_pid: u32) -> Result<(), i32> {
    if signo == 0 {
        // Signal 0 just checks if process exists
        let exists = SIGNAL_STATES.lock().contains_key(&target_pid);
        return if exists { Ok(()) } else { Err(-3) }; // ESRCH
    }
    
    let mut states = SIGNAL_STATES.lock();
    let state = states.get_mut(&target_pid).ok_or(-3)?; // ESRCH
    
    state.post_signal(signo, sender_pid);
    
    // Check if signal should terminate/stop process immediately
    if signo == sig::SIGKILL || signo == sig::SIGSTOP {
        handle_fatal_signal(target_pid, signo);
    }
    
    Ok(())
}

/// Send a signal to all processes in a process group
pub fn kill_process_group(pgid: u32, signo: u32) -> Result<(), i32> {
    let sender = crate::process::current_pid();
    let pids = crate::process::pids_in_group(pgid);
    if pids.is_empty() {
        return Err(-3); // ESRCH
    }
    for pid in pids {
        let _ = kill(pid, signo, sender);
    }
    Ok(())
}

/// Handle fatal signal (SIGKILL, SIGSTOP)
fn handle_fatal_signal(pid: u32, signo: u32) {
    match signo {
        sig::SIGKILL => {
            // Terminate process immediately
            crate::process::terminate(pid);
        }
        sig::SIGSTOP => {
            // Stop process
            crate::process::stop(pid);
        }
        _ => {}
    }
}

/// Set signal action (sigaction syscall)
pub fn set_action(pid: u32, signo: u32, action: SigAction) -> Result<SigAction, i32> {
    let mut states = SIGNAL_STATES.lock();
    let state = states.get_mut(&pid).ok_or(-3)?;
    state.set_action(signo, action)
}

/// Get signal action
pub fn get_action(pid: u32, signo: u32) -> Result<SigAction, i32> {
    let states = SIGNAL_STATES.lock();
    let state = states.get(&pid).ok_or(-3)?;
    state.get_action(signo).copied().ok_or(-22)
}

/// Set signal mask (sigprocmask syscall)
pub fn set_mask(pid: u32, how: u32, set: u64, old_set: &mut u64) -> Result<(), i32> {
    let states = SIGNAL_STATES.lock();
    let state = states.get(&pid).ok_or(-3)?;
    
    *old_set = state.blocked.load(Ordering::Relaxed);
    
    match how {
        0 => state.block(set),        // SIG_BLOCK
        1 => state.unblock(set),      // SIG_UNBLOCK
        2 => state.set_blocked(set),  // SIG_SETMASK
        _ => return Err(-22),
    }
    
    Ok(())
}

/// Check for pending signals and deliver if needed
pub fn check_signals(pid: u32) -> Option<u32> {
    let mut states = SIGNAL_STATES.lock();
    let state = states.get_mut(&pid)?;
    
    if let Some(signo) = state.get_deliverable() {
        let action = &state.actions[signo as usize];
        
        match action.sa_handler {
            SIG_IGN => {
                // Ignore - just clear
                state.clear_pending(signo);
                None
            }
            SIG_DFL => {
                // Default action
                state.clear_pending(signo);
                handle_default_action(pid, signo);
                Some(signo)
            }
            _ => {
                // User handler
                state.clear_pending(signo);
                Some(signo)
            }
        }
    } else {
        None
    }
}

/// Handle default signal action
fn handle_default_action(pid: u32, signo: u32) {
    match signo {
        // Signals that terminate
        sig::SIGHUP | sig::SIGINT | sig::SIGKILL | sig::SIGPIPE |
        sig::SIGALRM | sig::SIGTERM | sig::SIGUSR1 | sig::SIGUSR2 => {
            crate::process::terminate(pid);
        }
        
        // Signals that terminate with core dump
        sig::SIGQUIT | sig::SIGILL | sig::SIGABRT | sig::SIGFPE |
        sig::SIGSEGV | sig::SIGBUS | sig::SIGSYS => {
            // TODO: Generate core dump
            crate::process::terminate(pid);
        }
        
        // Signals that stop
        sig::SIGSTOP | sig::SIGTSTP | sig::SIGTTIN | sig::SIGTTOU => {
            crate::process::stop(pid);
        }
        
        // Signals that continue
        sig::SIGCONT => {
            crate::process::resume(pid);
        }
        
        // Signals ignored by default
        sig::SIGCHLD | sig::SIGURG | sig::SIGWINCH => {
            // Do nothing
        }
        
        _ => {
            // Unknown signal - terminate
            crate::process::terminate(pid);
        }
    }
}

/// Get signal name
pub fn signal_name(signo: u32) -> &'static str {
    match signo {
        sig::SIGHUP => "SIGHUP",
        sig::SIGINT => "SIGINT",
        sig::SIGQUIT => "SIGQUIT",
        sig::SIGILL => "SIGILL",
        sig::SIGTRAP => "SIGTRAP",
        sig::SIGABRT => "SIGABRT",
        sig::SIGBUS => "SIGBUS",
        sig::SIGFPE => "SIGFPE",
        sig::SIGKILL => "SIGKILL",
        sig::SIGUSR1 => "SIGUSR1",
        sig::SIGSEGV => "SIGSEGV",
        sig::SIGUSR2 => "SIGUSR2",
        sig::SIGPIPE => "SIGPIPE",
        sig::SIGALRM => "SIGALRM",
        sig::SIGTERM => "SIGTERM",
        sig::SIGCHLD => "SIGCHLD",
        sig::SIGCONT => "SIGCONT",
        sig::SIGSTOP => "SIGSTOP",
        sig::SIGTSTP => "SIGTSTP",
        _ => "UNKNOWN",
    }
}

// ============================================================================
// Signal Frame — pushed onto user stack for signal delivery
// ============================================================================

/// Signal frame pushed onto the user stack when delivering a signal.
/// The signal handler returns to `pretcode` (sa_restorer) which calls
/// rt_sigreturn to restore the original context.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct SignalFrame {
    /// Return address: sa_restorer (calls rt_sigreturn)
    pub pretcode: u64,
    /// Signal number (for diagnostics, signo is also passed in RDI)
    pub signo: u32,
    /// Padding for alignment
    pub _pad: u32,
    /// Saved user RIP (return address after signal handler)
    pub saved_rip: u64,
    /// Saved user RSP
    pub saved_rsp: u64,
    /// Saved RFLAGS
    pub saved_rflags: u64,
    /// Saved RAX (syscall return value)
    pub saved_rax: u64,
    /// Saved signal mask (to restore on sigreturn)
    pub saved_mask: u64,
}

/// Try to deliver a pending signal to the current process.
///
/// Called from the syscall return path. If a signal with a user handler is
/// pending, this function:
/// 1. Pushes a `SignalFrame` onto the user stack
/// 2. Modifies the return context (RIP, RSP) to jump to the handler
/// 3. Returns `Some((signo, handler))` so the caller can set RDI
///
/// Returns `None` if no signal needs delivery.
pub fn try_deliver_signal(
    user_rip: &mut u64,
    user_rsp: &mut u64,
    user_rflags: &mut u64,
    syscall_rax: u64,
) -> Option<u64> {
    let pid = crate::process::current_pid();
    
    let mut states = SIGNAL_STATES.lock();
    let state = states.get_mut(&pid)?;
    
    let signo = state.get_deliverable()?;
    let action = state.actions[signo as usize];
    
    match action.sa_handler {
        SIG_IGN => {
            state.clear_pending(signo);
            None
        }
        SIG_DFL => {
            state.clear_pending(signo);
            drop(states);
            handle_default_action(pid, signo);
            None
        }
        handler => {
            // User handler — deliver the signal
            state.clear_pending(signo);
            
            // Save the old signal mask and apply sa_mask during handler
            let old_mask = state.blocked.load(Ordering::Relaxed);
            let mut new_mask = old_mask | action.sa_mask;
            // Block the signal itself during handler (unless SA_NODEFER)
            if action.sa_flags & sa_flags::SA_NODEFER == 0 {
                new_mask |= 1 << signo;
            }
            state.blocked.store(new_mask, Ordering::SeqCst);
            
            // SA_RESETHAND: reset handler to SIG_DFL after delivery
            if action.sa_flags & sa_flags::SA_RESETHAND != 0 {
                state.actions[signo as usize].sa_handler = SIG_DFL;
            }
            
            drop(states);
            
            // Build signal frame on the user stack
            let frame_size = core::mem::size_of::<SignalFrame>() as u64;
            // Align to 16 bytes, then subtract 8 for the "return address" slot
            // so that RSP % 16 == 8 when the handler starts (x86_64 ABI)
            let new_rsp = ((*user_rsp - frame_size) & !0xF) - 8;
            
            // Validate user stack pointer
            if !crate::memory::is_user_address(new_rsp) {
                // Stack overflow during signal delivery — terminate
                crate::process::terminate(pid);
                return None;
            }
            
            // Write signal frame
            let frame = unsafe { &mut *(new_rsp as *mut SignalFrame) };
            frame.pretcode = if action.sa_flags & sa_flags::SA_RESTORER != 0 {
                action.sa_restorer
            } else {
                // No restorer — the process should have set one.
                // We can't provide a kernel trampoline in user space easily,
                // so terminate if missing.
                crate::log_debug!("[SIGNAL] No sa_restorer for signal {} — terminating PID {}", signo, pid);
                crate::process::terminate(pid);
                return None;
            };
            frame.signo = signo;
            frame._pad = 0;
            frame.saved_rip = *user_rip;
            frame.saved_rsp = *user_rsp;
            frame.saved_rflags = *user_rflags;
            frame.saved_rax = syscall_rax;
            frame.saved_mask = old_mask;
            
            // Redirect return to the signal handler
            *user_rip = handler;
            *user_rsp = new_rsp;
            // RFLAGS unchanged
            
            crate::log_debug!("[SIGNAL] Delivering signal {} to PID {} at handler {:#x}", signo, pid, handler);
            
            Some(signo as u64)
        }
    }
}

/// Restore context from a signal frame (called by rt_sigreturn).
///
/// Reads the `SignalFrame` from the user stack and restores the saved
/// execution context. Returns the saved RAX (original syscall result).
pub fn sigreturn_restore(
    user_rip: &mut u64,
    user_rsp: &mut u64,
    user_rflags: &mut u64,
) -> i64 {
    let pid = crate::process::current_pid();
    
    // The signal frame is at the current user RSP
    // (the handler did `ret` which consumed pretcode, so RSP might be off)
    // Actually, sa_restorer does `mov rax, 15; syscall` directly, so RSP
    // still points to the signal frame when rt_sigreturn enters.
    // After the `ret` from the handler, RSP points past pretcode.
    // But sa_restorer is called via `ret`, so RSP = &frame + 8 = &frame.signo
    // Wait — the handler RETurns to pretcode value. sa_restorer is a function
    // that calls rt_sigreturn. During rt_sigreturn syscall, the user RSP is
    // whatever sa_restorer left it at. We need the frame address.
    //
    // The frame starts at RSP - 8 (pretcode was the return address, consumed by RET).
    // Actually, let's compute: when signal was delivered:
    //   user_rsp = new_rsp (points to start of frame = pretcode)
    // When handler executes: RSP = new_rsp, first thing is pretcode at [RSP]
    // Handler does `ret` → pops pretcode → RSP = new_rsp + 8
    // sa_restorer runs with RSP = new_rsp + 8
    // sa_restorer does `mov rax, 15; syscall` → syscall_entry saves RSP
    // During rt_sigreturn: user_rsp = new_rsp + 8
    // Frame is at user_rsp - 8
    
    let frame_addr = *user_rsp - 8;
    
    if !crate::memory::is_user_address(frame_addr) {
        return -22; // EINVAL
    }
    
    let frame = unsafe { &*(frame_addr as *const SignalFrame) };
    
    // Restore saved context
    *user_rip = frame.saved_rip;
    *user_rsp = frame.saved_rsp;
    *user_rflags = frame.saved_rflags;
    
    // Restore signal mask
    if let Some(state) = SIGNAL_STATES.lock().get(&pid) {
        state.blocked.store(frame.saved_mask, Ordering::SeqCst);
    }
    
    crate::log_debug!("[SIGNAL] sigreturn: restoring RIP={:#x} RSP={:#x} RAX={}", 
        frame.saved_rip, frame.saved_rsp, frame.saved_rax as i64);
    
    frame.saved_rax as i64
}
