//! Thread Management - Kernel and User Threads
//!
//! Implements multithreading with:
//! - Thread Control Blocks (TCB)
//! - Per-thread kernel stacks
//! - Context switching (save/restore all registers)
//! - Thread states and scheduling integration

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use core::sync::atomic::{AtomicU64, AtomicU32, Ordering};
use spin::{Mutex, RwLock};

/// Thread ID type
pub type Tid = u64;

/// Invalid thread ID
pub const TID_INVALID: Tid = 0;

/// Thread ID counter
static NEXT_TID: AtomicU64 = AtomicU64::new(1);

/// Thread state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThreadState {
    /// Thread is ready to run
    Ready,
    /// Thread is currently running
    Running,
    /// Thread is blocked waiting for something
    Blocked,
    /// Thread is sleeping
    Sleeping,
    /// Thread has exited
    Dead,
}

/// Thread flags
#[derive(Debug, Clone, Copy)]
pub struct ThreadFlags(pub u32);

impl ThreadFlags {
    pub const NONE: u32 = 0;
    pub const KERNEL: u32 = 1 << 0;      // Kernel thread (Ring 0)
    pub const MAIN: u32 = 1 << 1;        // Main thread of process
    pub const DETACHED: u32 = 1 << 2;    // Detached (no join needed)
}

/// CPU context saved during context switch
#[derive(Debug, Clone, Default)]
#[repr(C)]
pub struct ThreadContext {
    // Callee-saved registers (must be preserved across function calls)
    pub rbx: u64,
    pub rbp: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
    
    // Stack pointer
    pub rsp: u64,
    
    // Instruction pointer (return address)
    pub rip: u64,
    
    // For userspace threads
    pub user_rsp: u64,
    pub user_rip: u64,
    
    // Segment selectors (for ring transitions)
    pub cs: u64,
    pub ss: u64,
    
    // Flags
    pub rflags: u64,
}

/// Thread Control Block (TCB)
pub struct Thread {
    /// Unique thread ID
    pub tid: Tid,
    
    /// Process ID this thread belongs to
    pub pid: u32,
    
    /// Thread name (for debugging)
    pub name: String,
    
    /// Current state
    pub state: ThreadState,
    
    /// Thread flags
    pub flags: ThreadFlags,
    
    /// CPU context (saved registers)
    pub context: ThreadContext,
    
    /// Kernel stack (each thread needs its own)
    kernel_stack: Option<Box<[u8; KERNEL_STACK_SIZE]>>,
    
    /// Kernel stack top (RSP value when entering kernel)
    pub kernel_stack_top: u64,
    
    /// User stack top (for user threads)
    pub user_stack_top: u64,
    
    /// Thread entry point
    pub entry_point: u64,
    
    /// Entry argument
    pub entry_arg: u64,
    
    /// Exit code (valid when state is Dead)
    pub exit_code: i32,
    
    /// CPU time consumed (in ticks)
    pub cpu_time: u64,
    
    /// Thread waiting to join on this thread
    pub joiner: Option<Tid>,
}

/// Kernel stack size per thread
const KERNEL_STACK_SIZE: usize = 64 * 1024; // 64 KB

impl Thread {
    /// Create a new kernel thread
    pub fn new_kernel(pid: u32, name: &str, entry: u64, arg: u64) -> Self {
        let tid = NEXT_TID.fetch_add(1, Ordering::SeqCst);
        
        // Allocate kernel stack
        let mut kernel_stack = Box::new([0u8; KERNEL_STACK_SIZE]);
        let stack_top = kernel_stack.as_ptr() as u64 + KERNEL_STACK_SIZE as u64;
        
        // Set up initial context for kernel thread
        let mut context = ThreadContext::default();
        
        // Initial RSP points to top of stack, minus space for return address
        context.rsp = stack_top - 8;
        context.rip = entry;
        context.rflags = 0x202; // IF=1, reserved=1
        context.cs = crate::gdt::KERNEL_CODE_SELECTOR as u64;
        context.ss = crate::gdt::KERNEL_DATA_SELECTOR as u64;
        
        // Set up stack with thread_entry_wrapper as return address
        // This wrapper will call the actual entry point with the argument
        unsafe {
            let ret_addr_ptr = (stack_top - 8) as *mut u64;
            *ret_addr_ptr = thread_entry_wrapper as u64;
        }
        
        // R12 = entry point, R13 = argument (callee-saved, used by wrapper)
        context.r12 = entry;
        context.r13 = arg;
        context.rip = thread_entry_wrapper as u64;
        
        Self {
            tid,
            pid,
            name: String::from(name),
            state: ThreadState::Ready,
            flags: ThreadFlags(ThreadFlags::KERNEL),
            context,
            kernel_stack: Some(kernel_stack),
            kernel_stack_top: stack_top,
            user_stack_top: 0,
            entry_point: entry,
            entry_arg: arg,
            exit_code: 0,
            cpu_time: 0,
            joiner: None,
        }
    }
    
    /// Create main thread for a process (user or kernel)
    pub fn new_main(pid: u32, name: &str, entry: u64, user_stack: u64, is_kernel: bool) -> Self {
        let mut thread = if is_kernel {
            Self::new_kernel(pid, name, entry, 0)
        } else {
            Self::new_user(pid, name, entry, user_stack, 0)
        };
        thread.flags.0 |= ThreadFlags::MAIN;
        thread
    }
    
    /// Create a new user thread
    pub fn new_user(pid: u32, name: &str, entry: u64, user_stack: u64, arg: u64) -> Self {
        let tid = NEXT_TID.fetch_add(1, Ordering::SeqCst);
        
        // Allocate kernel stack (for syscalls/interrupts)
        let kernel_stack = Box::new([0u8; KERNEL_STACK_SIZE]);
        let kernel_stack_top = kernel_stack.as_ptr() as u64 + KERNEL_STACK_SIZE as u64;
        
        // Set up initial context for user thread
        let mut context = ThreadContext::default();
        
        // User thread starts in Ring 3
        context.user_rsp = user_stack;
        context.user_rip = entry;
        context.rflags = 0x202; // IF=1
        context.cs = crate::gdt::USER_CODE_SELECTOR as u64;
        context.ss = crate::gdt::USER_DATA_SELECTOR as u64;
        
        // Kernel context for first entry
        context.rsp = kernel_stack_top;
        context.rip = user_thread_entry as u64;
        
        // Store entry point and arg in callee-saved registers
        context.r12 = entry;
        context.r13 = user_stack;
        context.r14 = arg;
        
        Self {
            tid,
            pid,
            name: String::from(name),
            state: ThreadState::Ready,
            flags: ThreadFlags(ThreadFlags::NONE),
            context,
            kernel_stack: Some(kernel_stack),
            kernel_stack_top,
            user_stack_top: user_stack,
            entry_point: entry,
            entry_arg: arg,
            exit_code: 0,
            cpu_time: 0,
            joiner: None,
        }
    }
    
    /// Check if this is a kernel thread
    pub fn is_kernel(&self) -> bool {
        self.flags.0 & ThreadFlags::KERNEL != 0
    }
    
    /// Check if this is the main thread
    pub fn is_main(&self) -> bool {
        self.flags.0 & ThreadFlags::MAIN != 0
    }
}

/// Kernel thread entry wrapper
/// Called with entry in R12, arg in R13
#[cfg(target_arch = "x86_64")]
#[unsafe(naked)]
extern "C" fn thread_entry_wrapper() {
    core::arch::naked_asm!(
        // New threads start with IF=0 (from context switch inside timer handler).
        // Enable interrupts so the thread can be preempted.
        "sti",
        
        // Entry point is in R12, argument is in R13
        "mov rdi, r13",      // arg -> first parameter
        "call r12",          // Call entry point
        
        // Thread returned, call exit
        "mov rdi, rax",      // Return value as exit code
        "call {exit}",
        
        // Should never reach here
        "ud2",
        
        exit = sym thread_exit,
    );
}

#[cfg(not(target_arch = "x86_64"))]
extern "C" fn thread_entry_wrapper() {
    // Thread entry not implemented for this architecture yet
}

/// User thread entry point (jumps to Ring 3)
/// After context_switch restores callee-saved registers:
///   R12 = user entry point, R13 = user stack, R14 = arg
#[cfg(target_arch = "x86_64")]
#[unsafe(naked)]
extern "C" fn user_thread_entry() {
    core::arch::naked_asm!(
        // Set up args for jump_to_ring3_with_args(entry, stack, arg, 0)
        "mov rdi, r12",          // entry point
        "mov rsi, r13",          // user stack
        "mov rdx, r14",          // arg (argc)
        "xor ecx, ecx",         // 0 (arg2)
        "jmp {jump}",
        jump = sym crate::userland::jump_to_ring3_with_args,
    );
}

#[cfg(not(target_arch = "x86_64"))]
extern "C" fn user_thread_entry() {
    // User thread entry not implemented for this architecture yet
}

/// Thread exit handler
#[no_mangle]
extern "C" fn thread_exit(exit_code: i32) {
    let tid = current_tid();
    
    if let Some(mut thread) = THREADS.write().get_mut(&tid) {
        thread.state = ThreadState::Dead;
        thread.exit_code = exit_code;
        
        // Wake up joiner if any
        if let Some(joiner_tid) = thread.joiner {
            if let Some(joiner) = THREADS.write().get_mut(&joiner_tid) {
                if joiner.state == ThreadState::Blocked {
                    joiner.state = ThreadState::Ready;
                }
            }
        }
    }
    
    crate::log_debug!("[THREAD] Thread {} exited with code {}", tid, exit_code);
    
    // Yield to scheduler
    yield_thread();
}

// ============================================================================
// Thread Table
// ============================================================================

lazy_static::lazy_static! {
    /// Global thread table
    static ref THREADS: RwLock<BTreeMap<Tid, Thread>> = RwLock::new(BTreeMap::new());
}

/// Current thread ID per CPU — SMP: each CPU has its own slot
const MAX_CPUS_SCHED: usize = 64;
static CURRENT_TIDS: [AtomicU64; MAX_CPUS_SCHED] = {
    const INIT: AtomicU64 = AtomicU64::new(TID_INVALID);
    [INIT; MAX_CPUS_SCHED]
};

/// Idle thread TID base for APs (BSP uses TID 0)
const IDLE_TID_AP_BASE: Tid = 0x8000_0000_0000_0000;

/// Get idle thread TID for a given CPU
fn idle_tid_for(cpu_id: usize) -> Tid {
    if cpu_id == 0 { 0 } else { IDLE_TID_AP_BASE + cpu_id as u64 }
}

/// Check if a TID is an idle thread (should never be enqueued)
fn is_idle_tid(tid: Tid) -> bool {
    tid == 0 || tid >= IDLE_TID_AP_BASE
}

/// Get current CPU ID for scheduler (uses CPUID-based lookup)
#[inline]
fn sched_cpu_id() -> usize {
    (crate::cpu::smp::current_cpu_id() as usize).min(MAX_CPUS_SCHED - 1)
}

/// Get current thread ID
pub fn current_tid() -> Tid {
    CURRENT_TIDS[sched_cpu_id()].load(Ordering::Relaxed)
}

/// Set current thread ID
pub fn set_current_tid(tid: Tid) {
    CURRENT_TIDS[sched_cpu_id()].store(tid, Ordering::SeqCst);
}

/// Create a new kernel thread
pub fn spawn_kernel(name: &str, entry: fn(u64) -> i32, arg: u64) -> Tid {
    let pid = crate::process::current_pid();
    let thread = Thread::new_kernel(pid, name, entry as *const () as u64, arg);
    let tid = thread.tid;
    
    THREADS.write().insert(tid, thread);
    enqueue_thread(tid);
    
    crate::log_debug!("[THREAD] Spawned kernel thread {} '{}'", tid, name);
    tid
}

/// Create a new user thread
pub fn spawn_user(pid: u32, name: &str, entry: u64, user_stack: u64, arg: u64) -> Tid {
    let thread = Thread::new_user(pid, name, entry, user_stack, arg);
    let tid = thread.tid;
    
    THREADS.write().insert(tid, thread);
    enqueue_thread(tid);
    
    crate::log_debug!("[THREAD] Spawned user thread {} '{}'", tid, name);
    tid
}

/// Yield current thread
pub fn yield_thread() {
    schedule();
}

/// Exit current thread
pub fn exit(code: i32) {
    thread_exit(code);
}

/// Wake a sleeping thread (make it ready to run)
pub fn wake(tid: Tid) {
    let mut threads = THREADS.write();
    if let Some(thread) = threads.get_mut(&tid) {
        if thread.state == ThreadState::Blocked {
            thread.state = ThreadState::Ready;
            drop(threads);
            enqueue_thread(tid);
        }
    }
}

/// Block current thread (put it to sleep)
pub fn block(tid: Tid) {
    let mut threads = THREADS.write();
    if let Some(thread) = threads.get_mut(&tid) {
        thread.state = ThreadState::Blocked;
    }
}

/// Block current thread AND immediately context-switch away.
/// The thread is marked Blocked and will NOT be put back in the ready queue
/// until another thread calls `wake(tid)`.
/// Returns `true` if the thread was woken normally, `false` on timeout.
pub fn block_current_and_schedule() {
    let tid = current_tid();
    if is_idle_tid(tid) {
        return; // never block idle
    }
    {
        let mut threads = THREADS.write();
        if let Some(thread) = threads.get_mut(&tid) {
            thread.state = ThreadState::Blocked;
        }
    }
    // schedule() will see the thread is Blocked and won't re-enqueue it
    schedule();
}

/// Sleep current thread until `deadline_ns` (nanosecond timestamp).
/// Registers a timer callback that wakes the thread at the deadline.
/// Other threads can also wake this thread early via `wake(tid)`.
pub fn sleep_until(deadline_ns: u64) {
    let tid = current_tid();
    if is_idle_tid(tid) {
        return;
    }

    // Register a timer wake-up so the thread is woken at the deadline
    crate::time::register_wakeup(tid, deadline_ns);

    // Block and yield
    block_current_and_schedule();
}

/// Sleep current thread for `duration_ns` nanoseconds.
pub fn sleep_ns(duration_ns: u64) {
    let deadline = crate::time::now_ns().saturating_add(duration_ns);
    sleep_until(deadline);
}

// ============================================================================
// Scheduler Integration
// ============================================================================

use alloc::collections::VecDeque;

// ============================================================================
// Per-CPU Run Queues with Work Stealing
// ============================================================================

/// Per-CPU ready queue  
struct PerCpuQueue {
    queue: Mutex<VecDeque<Tid>>,
}

impl PerCpuQueue {
    const fn new() -> Self {
        Self { queue: Mutex::new(VecDeque::new()) }
    }
    
    fn push(&self, tid: Tid) {
        self.queue.lock().push_back(tid);
    }
    
    fn pop(&self) -> Option<Tid> {
        self.queue.lock().pop_front()
    }
    
    /// Steal from the back (reduces contention with local dequeue from front)
    fn steal(&self) -> Option<Tid> {
        self.queue.lock().pop_back()
    }
    
    fn len(&self) -> usize {
        self.queue.lock().len()
    }
}

/// Per-CPU run queues (indexed by cpu_id)
static PER_CPU_QUEUES: [PerCpuQueue; MAX_CPUS_SCHED] = {
    const INIT: PerCpuQueue = PerCpuQueue::new();
    [INIT; MAX_CPUS_SCHED]
};

/// Round-robin CPU assignment counter for new threads
static NEXT_CPU: AtomicU64 = AtomicU64::new(0);

/// Legacy global ready queue (kept for compatibility, used as overflow)
lazy_static::lazy_static! {
    static ref READY_QUEUE: Mutex<VecDeque<Tid>> = Mutex::new(VecDeque::new());
}

/// Enqueue a thread on the best CPU's run queue
fn enqueue_thread(tid: Tid) {
    let num_cpus = crate::cpu::smp::ready_cpu_count().max(1) as usize;
    
    // Round-robin assignment across available CPUs
    let target_cpu = (NEXT_CPU.fetch_add(1, Ordering::Relaxed) % num_cpus as u64) as usize;
    PER_CPU_QUEUES[target_cpu].push(tid);
    
    // If the target CPU is not the current one and is idle, wake it
    let current_cpu = sched_cpu_id();
    if target_cpu != current_cpu && target_cpu > 0 {
        crate::cpu::smp::send_reschedule_ipi(target_cpu as u32);
    }
}

/// Try to steal work from the busiest other CPU
fn try_steal_work(my_cpu: usize) -> Option<Tid> {
    let num_cpus = crate::cpu::smp::ready_cpu_count().max(1) as usize;
    
    // Find the busiest CPU (most items in queue) that isn't us
    let mut best_cpu = usize::MAX;
    let mut best_len = 0;
    
    for cpu in 0..num_cpus {
        if cpu == my_cpu { continue; }
        let len = PER_CPU_QUEUES[cpu].len();
        if len > best_len {
            best_len = len;
            best_cpu = cpu;
        }
    }
    
    if best_cpu < MAX_CPUS_SCHED && best_len > 1 {
        return PER_CPU_QUEUES[best_cpu].steal();
    }
    
    // Also check the legacy global queue
    READY_QUEUE.lock().pop_front()
}

/// Initialize thread subsystem (BSP only)
pub fn init() {
    // Create idle thread (TID 0) for BSP
    let idle_thread = Thread {
        tid: 0,
        pid: 0,
        name: String::from("idle"),
        state: ThreadState::Running,
        flags: ThreadFlags(ThreadFlags::KERNEL | ThreadFlags::MAIN),
        context: ThreadContext::default(),
        kernel_stack: None,
        kernel_stack_top: 0,
        user_stack_top: 0,
        entry_point: 0,
        entry_arg: 0,
        exit_code: 0,
        cpu_time: 0,
        joiner: None,
    };
    
    THREADS.write().insert(0, idle_thread);
    CURRENT_TIDS[0].store(0, Ordering::SeqCst);
    
    crate::log!("[THREAD] Thread subsystem initialized");
}

/// Initialize scheduler for an Application Processor.
/// Creates an idle thread so the AP has a valid current_tid.
pub fn init_ap_scheduler(cpu_id: u32) {
    let idx = cpu_id as usize;
    let idle_tid = idle_tid_for(idx);
    
    let idle = Thread {
        tid: idle_tid,
        pid: 0,
        name: String::from("idle-ap"),
        state: ThreadState::Running,
        flags: ThreadFlags(ThreadFlags::KERNEL),
        context: ThreadContext::default(),
        kernel_stack: None,
        kernel_stack_top: 0,
        user_stack_top: 0,
        entry_point: 0,
        entry_arg: 0,
        exit_code: 0,
        cpu_time: 0,
        joiner: None,
    };
    
    THREADS.write().insert(idle_tid, idle);
    CURRENT_TIDS[idx].store(idle_tid, Ordering::SeqCst);
    
    crate::serial_println!("[THREAD] AP {} idle thread created (TID={:#x})", cpu_id, idle_tid);
}

/// Called on timer tick - preemptive scheduling
pub fn on_timer_tick() {
    let tid = current_tid();
    
    // Increment CPU time for current thread
    if let Some(thread) = THREADS.write().get_mut(&tid) {
        thread.cpu_time += 1;
    }
    
    // Check if we should preempt (every 10 ticks = 100ms at 100Hz)
    static TICK_COUNT: AtomicU64 = AtomicU64::new(0);
    let ticks = TICK_COUNT.fetch_add(1, Ordering::Relaxed);
    
    if ticks % 10 == 0 {
        schedule();
    }
}

/// Schedule next thread (SMP-safe: per-CPU run queues with work stealing)
pub fn schedule() {
    let cpu_id = sched_cpu_id();
    let current = current_tid();
    let idle = idle_tid_for(cpu_id);
    
    // Put current thread back on this CPU's queue if still runnable
    if current != TID_INVALID && !is_idle_tid(current) {
        if let Some(thread) = THREADS.read().get(&current) {
            if thread.state == ThreadState::Running {
                if let Some(t) = THREADS.write().get_mut(&current) {
                    t.state = ThreadState::Ready;
                }
                PER_CPU_QUEUES[cpu_id].push(current);
            }
        }
    }
    
    // Try to get next thread from: 1) local queue, 2) work stealing, 3) global queue
    let next_tid = loop {
        // 1. Try local per-CPU queue
        if let Some(tid) = PER_CPU_QUEUES[cpu_id].pop() {
            if let Some(thread) = THREADS.read().get(&tid) {
                if thread.state == ThreadState::Ready || thread.state == ThreadState::Running {
                    break Some(tid);
                }
            }
            continue; // skip non-runnable, try next
        }
        
        // 2. Try work stealing from other CPUs
        if let Some(tid) = try_steal_work(cpu_id) {
            if let Some(thread) = THREADS.read().get(&tid) {
                if thread.state == ThreadState::Ready || thread.state == ThreadState::Running {
                    break Some(tid);
                }
            }
            continue;
        }
        
        // No work available
        break None;
    };
    
    match next_tid {
        Some(next) if next != current => {
            // Mark next thread as running
            if let Some(thread) = THREADS.write().get_mut(&next) {
                thread.state = ThreadState::Running;
            }
            
            // Perform context switch
            context_switch(current, next);
        }
        None if !is_idle_tid(current) && current != TID_INVALID => {
            // Current thread is done/blocked, fall back to this CPU's idle thread
            context_switch(current, idle);
        }
        _ => {
            // Already on idle or same thread, continue
        }
    }
}

/// Context switch from one thread to another
fn context_switch(from: Tid, to: Tid) {
    if from == to {
        return;
    }
    
    // Get contexts
    let from_ctx_ptr: *mut ThreadContext;
    let to_ctx_ptr: *const ThreadContext;
    let to_kernel_stack: u64;
    
    {
        let mut threads = THREADS.write();
        
        let from_thread = match threads.get_mut(&from) {
            Some(t) => t as *mut Thread,
            None => return,
        };
        
        let to_thread = match threads.get(&to) {
            Some(t) => t,
            None => return,
        };
        
        from_ctx_ptr = unsafe { &mut (*from_thread).context as *mut ThreadContext };
        to_ctx_ptr = &to_thread.context as *const ThreadContext;
        to_kernel_stack = to_thread.kernel_stack_top;
    }
    
    // Update TSS with new kernel stack (for Ring 3 -> Ring 0 transitions)
    if to_kernel_stack != 0 {
        crate::gdt::set_kernel_stack(to_kernel_stack);
        
        // Also update the SYSCALL stack pointer so that SYSCALL from Ring 3
        // uses this thread's kernel stack (not the global one).
        unsafe {
            crate::userland::KERNEL_SYSCALL_STACK_TOP = to_kernel_stack;
        }
    }
    
    // Update current thread (per-CPU)
    set_current_tid(to);
    
    // Do the actual context switch
    unsafe {
        switch_context(from_ctx_ptr, to_ctx_ptr);
    }
}

/// Low-level context switch (saves/restores all callee-saved registers)
#[cfg(target_arch = "x86_64")]
#[unsafe(naked)]
extern "C" fn switch_context(from: *mut ThreadContext, to: *const ThreadContext) {
    core::arch::naked_asm!(
        // Save current context to 'from'
        // RDI = from, RSI = to
        
        // Save callee-saved registers
        "mov [rdi + 0x00], rbx",
        "mov [rdi + 0x08], rbp",
        "mov [rdi + 0x10], r12",
        "mov [rdi + 0x18], r13",
        "mov [rdi + 0x20], r14",
        "mov [rdi + 0x28], r15",
        
        // Save RSP
        "mov [rdi + 0x30], rsp",
        
        // Save return address as RIP
        "lea rax, [rip + 2f]",
        "mov [rdi + 0x38], rax",
        
        // Load new context from 'to'
        // RSI = to
        
        // Load callee-saved registers
        "mov rbx, [rsi + 0x00]",
        "mov rbp, [rsi + 0x08]",
        "mov r12, [rsi + 0x10]",
        "mov r13, [rsi + 0x18]",
        "mov r14, [rsi + 0x20]",
        "mov r15, [rsi + 0x28]",
        
        // Load RSP
        "mov rsp, [rsi + 0x30]",
        
        // Jump to saved RIP
        "jmp [rsi + 0x38]",
        
        // Return point for saved context
        "2:",
        "ret",
    );
}

/// Context switch stub for non-x86_64 architectures (no-op)
#[cfg(not(target_arch = "x86_64"))]
extern "C" fn switch_context(_from: *mut ThreadContext, _to: *const ThreadContext) {
    // Context switching not implemented for this architecture yet
}

// ============================================================================
// Thread Synchronization Primitives
// ============================================================================

/// Mutex for thread synchronization
pub struct ThreadMutex {
    locked: AtomicU32,
    owner: AtomicU64,
    waiters: Mutex<VecDeque<Tid>>,
}

impl ThreadMutex {
    pub const fn new() -> Self {
        Self {
            locked: AtomicU32::new(0),
            owner: AtomicU64::new(TID_INVALID),
            waiters: Mutex::new(VecDeque::new()),
        }
    }
    
    pub fn lock(&self) {
        let tid = current_tid();
        
        loop {
            // Try to acquire lock
            if self.locked.compare_exchange(0, 1, Ordering::Acquire, Ordering::Relaxed).is_ok() {
                self.owner.store(tid, Ordering::Relaxed);
                return;
            }
            
            // Block current thread
            {
                self.waiters.lock().push_back(tid);
                if let Some(thread) = THREADS.write().get_mut(&tid) {
                    thread.state = ThreadState::Blocked;
                }
            }
            
            // Yield
            yield_thread();
        }
    }
    
    pub fn unlock(&self) {
        self.owner.store(TID_INVALID, Ordering::Relaxed);
        self.locked.store(0, Ordering::Release);
        
        // Wake up one waiter
        if let Some(waiter) = self.waiters.lock().pop_front() {
            if let Some(thread) = THREADS.write().get_mut(&waiter) {
                thread.state = ThreadState::Ready;
            }
            enqueue_thread(waiter);
        }
    }
    
    pub fn try_lock(&self) -> bool {
        let tid = current_tid();
        
        if self.locked.compare_exchange(0, 1, Ordering::Acquire, Ordering::Relaxed).is_ok() {
            self.owner.store(tid, Ordering::Relaxed);
            true
        } else {
            false
        }
    }
}

/// Semaphore for thread synchronization
pub struct Semaphore {
    count: AtomicU32,
    waiters: Mutex<VecDeque<Tid>>,
}

impl Semaphore {
    pub const fn new(initial: u32) -> Self {
        Self {
            count: AtomicU32::new(initial),
            waiters: Mutex::new(VecDeque::new()),
        }
    }
    
    pub fn wait(&self) {
        loop {
            let count = self.count.load(Ordering::Relaxed);
            
            if count > 0 {
                if self.count.compare_exchange(count, count - 1, Ordering::Acquire, Ordering::Relaxed).is_ok() {
                    return;
                }
            } else {
                // Block
                let tid = current_tid();
                {
                    self.waiters.lock().push_back(tid);
                    if let Some(thread) = THREADS.write().get_mut(&tid) {
                        thread.state = ThreadState::Blocked;
                    }
                }
                yield_thread();
            }
        }
    }
    
    pub fn signal(&self) {
        self.count.fetch_add(1, Ordering::Release);
        
        // Wake up one waiter
        if let Some(waiter) = self.waiters.lock().pop_front() {
            if let Some(thread) = THREADS.write().get_mut(&waiter) {
                thread.state = ThreadState::Ready;
            }
            enqueue_thread(waiter);
        }
    }
}

/// Condition variable
pub struct CondVar {
    waiters: Mutex<VecDeque<Tid>>,
}

impl CondVar {
    pub const fn new() -> Self {
        Self {
            waiters: Mutex::new(VecDeque::new()),
        }
    }
    
    /// Wait on condition (must hold mutex)
    pub fn wait(&self, mutex: &ThreadMutex) {
        let tid = current_tid();
        
        // Add to waiters
        self.waiters.lock().push_back(tid);
        
        // Block current thread
        if let Some(thread) = THREADS.write().get_mut(&tid) {
            thread.state = ThreadState::Blocked;
        }
        
        // Release mutex
        mutex.unlock();
        
        // Yield
        yield_thread();
        
        // Re-acquire mutex
        mutex.lock();
    }
    
    /// Wake one waiting thread
    pub fn signal(&self) {
        if let Some(waiter) = self.waiters.lock().pop_front() {
            if let Some(thread) = THREADS.write().get_mut(&waiter) {
                thread.state = ThreadState::Ready;
            }
            enqueue_thread(waiter);
        }
    }
    
    /// Wake all waiting threads
    pub fn broadcast(&self) {
        let mut waiters = self.waiters.lock();
        while let Some(waiter) = waiters.pop_front() {
            if let Some(thread) = THREADS.write().get_mut(&waiter) {
                thread.state = ThreadState::Ready;
            }
            enqueue_thread(waiter);
        }
    }
}

// ==================== THREAD LISTING ====================

/// List all threads for shell command
/// Returns (tid, pid, state, name)
pub fn list_threads() -> alloc::vec::Vec<(u64, u32, ThreadState, alloc::string::String)> {
    let threads = THREADS.read();
    let mut result = alloc::vec::Vec::new();
    
    for (tid, thread) in threads.iter() {
        result.push((*tid, thread.pid, thread.state, thread.name.clone()));
    }
    
    // Sort by TID
    result.sort_by_key(|(tid, _, _, _)| *tid);
    result
}
