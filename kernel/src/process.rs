//! Process Manager
//!
//! Manages processes in TrustOS. Provides PID allocation, process states,
//! and the foundation for userspace execution.

use alloc::string::String;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use alloc::boxed::Box;
use alloc::sync::Arc;
use spin::{RwLock, Mutex};
use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use crate::memory::AddressSpace;

/// Process ID type
pub type Pid = u32;

/// Special PIDs
pub const PID_KERNEL: Pid = 0;
pub const PID_INIT: Pid = 1;

/// Process state
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProcessState {
    /// Just created, not yet runnable
    Created,
    /// Ready to run
    Ready,
    /// Currently running
    Running,
    /// Waiting for I/O or event
    Blocked,
    /// Waiting for child to exit
    Waiting,
    /// Stopped by signal (SIGSTOP/SIGTSTP)
    Stopped,
    /// Process has exited but not yet reaped
    Zombie,
    /// Process fully terminated
    Dead,
}

/// Process flags
#[derive(Clone, Copy, Debug)]
pub struct ProcessFlags(pub u32);

impl ProcessFlags {
    pub const NONE: u32 = 0;
    pub const KERNEL: u32 = 1 << 0;      // Kernel process (ring 0)
    pub const DAEMON: u32 = 1 << 1;      // Background daemon
    pub const INIT: u32 = 1 << 2;        // Init process
}

/// File descriptor table entry
#[derive(Clone, Debug)]
pub struct FdEntry {
    pub vfs_fd: i32,    // VFS file descriptor
    pub flags: u32,     // Flags (close-on-exec, etc.)
}

/// Process memory layout
#[derive(Clone, Debug, Default)]
pub struct MemoryLayout {
    pub code_start: u64,
    pub code_end: u64,
    pub data_start: u64,
    pub data_end: u64,
    pub heap_start: u64,
    pub heap_end: u64,
    pub stack_start: u64,
    pub stack_end: u64,
}

/// CPU context for context switching
#[derive(Clone, Debug, Default)]
#[repr(C)]
pub struct CpuContext {
    // General purpose registers
    pub rax: u64,
    pub rbx: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub rbp: u64,
    pub rsp: u64,
    pub r8: u64,
    pub r9: u64,
    pub r10: u64,
    pub r11: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
    // Instruction pointer
    pub rip: u64,
    // Flags
    pub rflags: u64,
    // Segment selectors
    pub cs: u64,
    pub ss: u64,
}

/// Process Control Block (PCB)
#[derive(Clone)]
pub struct Process {
    /// Process ID
    pub pid: Pid,
    /// Parent process ID
    pub ppid: Pid,
    /// Process name
    pub name: String,
    /// Current state
    pub state: ProcessState,
    /// Process flags
    pub flags: ProcessFlags,
    /// Exit code (valid when state is Zombie)
    pub exit_code: i32,
    /// CPU context (saved registers)
    pub context: CpuContext,
    /// Memory layout
    pub memory: MemoryLayout,
    /// File descriptor table
    pub fd_table: BTreeMap<i32, FdEntry>,
    /// Next available fd
    next_fd: i32,
    /// Current working directory
    pub cwd: String,
    /// Environment variables
    pub env: BTreeMap<String, String>,
    /// CPU time used (in ticks)
    pub cpu_time: u64,
    /// Children PIDs
    pub children: Vec<Pid>,
    /// CR3 value for this process (0 = use kernel CR3)
    pub cr3: u64,
    /// Address space (None for kernel processes)
    pub address_space: Option<Arc<Mutex<AddressSpace>>>,
}

impl Process {
    /// Create a new process
    pub fn new(pid: Pid, ppid: Pid, name: &str, flags: ProcessFlags) -> Self {
        let mut fd_table = BTreeMap::new();
        
        // Setup standard file descriptors
        // 0 = stdin, 1 = stdout, 2 = stderr
        fd_table.insert(0, FdEntry { vfs_fd: 0, flags: 0 });
        fd_table.insert(1, FdEntry { vfs_fd: 1, flags: 0 });
        fd_table.insert(2, FdEntry { vfs_fd: 2, flags: 0 });
        
        // Create address space for userspace processes
        let (address_space, cr3) = if flags.0 & ProcessFlags::KERNEL != 0 {
            // Kernel process uses kernel address space
            (None, crate::memory::paging::kernel_cr3())
        } else {
            // User process gets its own address space
            match AddressSpace::new_with_kernel() {
                Some(space) => {
                    let cr3 = space.cr3();
                    (Some(Arc::new(Mutex::new(space))), cr3)
                }
                None => {
                    // Fallback to kernel space if allocation fails
                    (None, crate::memory::paging::kernel_cr3())
                }
            }
        };
        
        Self {
            pid,
            ppid,
            name: String::from(name),
            state: ProcessState::Created,
            flags,
            exit_code: 0,
            context: CpuContext::default(),
            memory: MemoryLayout::default(),
            fd_table,
            next_fd: 3,
            cwd: String::from("/"),
            env: BTreeMap::new(),
            cpu_time: 0,
            children: Vec::new(),
            cr3,
            address_space,
        }
    }
    
    /// Allocate a new file descriptor
    pub fn alloc_fd(&mut self, vfs_fd: i32) -> i32 {
        let fd = self.next_fd;
        self.next_fd += 1;
        self.fd_table.insert(fd, FdEntry { vfs_fd, flags: 0 });
        fd
    }
    
    /// Close a file descriptor
    pub fn close_fd(&mut self, fd: i32) -> Option<FdEntry> {
        self.fd_table.remove(&fd)
    }
    
    /// Get VFS fd for process fd
    pub fn get_vfs_fd(&self, fd: i32) -> Option<i32> {
        self.fd_table.get(&fd).map(|e| e.vfs_fd)
    }
    
    /// Set environment variable
    pub fn setenv(&mut self, key: &str, value: &str) {
        self.env.insert(String::from(key), String::from(value));
    }
    
    /// Get environment variable
    pub fn getenv(&self, key: &str) -> Option<&str> {
        self.env.get(key).map(|s| s.as_str())
    }
    
    /// Duplicate fd to the lowest available number
    pub fn dup_fd(&mut self, old_fd: i32) -> Result<i32, &'static str> {
        let entry = self.fd_table.get(&old_fd).ok_or("Bad fd")?.clone();
        let new_fd = self.next_fd;
        self.next_fd += 1;
        self.fd_table.insert(new_fd, entry);
        Ok(new_fd)
    }
    
    /// Duplicate fd to a specific target number
    pub fn dup2_fd(&mut self, old_fd: i32, new_fd: i32) -> Result<i32, &'static str> {
        if old_fd == new_fd { return Ok(new_fd); }
        let entry = self.fd_table.get(&old_fd).ok_or("Bad fd")?.clone();
        self.fd_table.remove(&new_fd);
        self.fd_table.insert(new_fd, entry);
        Ok(new_fd)
    }
}

/// Process table
struct ProcessTable {
    processes: BTreeMap<Pid, Process>,
    next_pid: AtomicU32,
}

impl ProcessTable {
    const fn new() -> Self {
        Self {
            processes: BTreeMap::new(),
            next_pid: AtomicU32::new(PID_INIT),
        }
    }
    
    fn alloc_pid(&self) -> Pid {
        self.next_pid.fetch_add(1, Ordering::SeqCst)
    }
}

static PROCESS_TABLE: RwLock<ProcessTable> = RwLock::new(ProcessTable::new());
static CURRENT_PID: AtomicU32 = AtomicU32::new(PID_KERNEL);

/// Initialize the process manager
pub fn init() {
    crate::log!("[PROC] Initializing process manager...");
    
    // Create kernel process (PID 0)
    let kernel_proc = Process::new(
        PID_KERNEL,
        PID_KERNEL,
        "kernel",
        ProcessFlags(ProcessFlags::KERNEL)
    );
    
    {
        let mut table = PROCESS_TABLE.write();
        table.processes.insert(PID_KERNEL, kernel_proc);
    }
    
    crate::log_debug!("[PROC] Kernel process created (PID 0)");
    crate::log!("[OK] Process manager ready");
}

/// Create a new process
pub fn create(name: &str, ppid: Pid) -> Result<Pid, &'static str> {
    let mut table = PROCESS_TABLE.write();
    
    let pid = table.alloc_pid();
    let mut proc = Process::new(pid, ppid, name, ProcessFlags(ProcessFlags::NONE));
    
    // Inherit cwd and env from parent
    if let Some(parent) = table.processes.get(&ppid) {
        proc.cwd = parent.cwd.clone();
        proc.env = parent.env.clone();
    }
    
    // Add to parent's children
    if let Some(parent) = table.processes.get_mut(&ppid) {
        parent.children.push(pid);
    }
    
    proc.state = ProcessState::Ready;
    table.processes.insert(pid, proc);
    
    crate::log_debug!("[PROC] Created process {} ({})", pid, name);
    Ok(pid)
}

/// Fork the current process with Copy-on-Write semantics.
///
/// Creates a child process that shares the parent's physical pages.
/// Pages are marked read-only + COW; on first write the page fault handler
/// allocates a private copy (see `memory::cow`).
pub fn fork() -> Result<Pid, &'static str> {
    let current = current_pid();
    
    // Read parent data while holding read lock
    let (name, cwd, env, fd_table, next_fd, parent_cr3, memory) = {
        let table = PROCESS_TABLE.read();
        let parent = table.processes.get(&current)
            .ok_or("Current process not found")?;
        (
            parent.name.clone(),
            parent.cwd.clone(),
            parent.env.clone(),
            parent.fd_table.clone(),
            parent.next_fd,
            parent.cr3,
            parent.memory.clone(),
        )
    };
    
    // Clone address space with COW (drops all locks first)
    let kernel_cr3 = crate::memory::paging::kernel_cr3();
    let (address_space, cr3) = if parent_cr3 != kernel_cr3 {
        match crate::memory::cow::clone_cow(parent_cr3) {
            Some(space) => {
                let c = space.cr3();
                (Some(Arc::new(Mutex::new(space))), c)
            }
            None => {
                // Fallback: fresh address space
                match AddressSpace::new_with_kernel() {
                    Some(space) => {
                        let c = space.cr3();
                        (Some(Arc::new(Mutex::new(space))), c)
                    }
                    None => (None, kernel_cr3)
                }
            }
        }
    } else {
        (None, kernel_cr3)
    };
    
    // Allocate PID and insert child under write lock
    let mut table = PROCESS_TABLE.write();
    let pid = table.alloc_pid();
    
    let child = Process {
        pid,
        ppid: current,
        name,
        state: ProcessState::Ready,
        flags: ProcessFlags(ProcessFlags::NONE),
        exit_code: 0,
        context: CpuContext::default(),
        memory,
        fd_table,   // inherits parent's fd table
        next_fd,
        cwd,
        env,
        cpu_time: 0,
        children: Vec::new(),
        cr3,
        address_space,
    };
    
    if let Some(parent) = table.processes.get_mut(&current) {
        parent.children.push(pid);
    }
    table.processes.insert(pid, child);
    
    // Initialize signal state for child
    crate::signals::init_process(pid);
    
    crate::log_debug!("[PROC] COW-fork: {} -> {}", current, pid);
    Ok(pid)
}

/// Exit the current process
pub fn exit(code: i32) {
    let current = current_pid();
    let mut table = PROCESS_TABLE.write();
    
    if let Some(proc) = table.processes.get_mut(&current) {
        proc.state = ProcessState::Zombie;
        proc.exit_code = code;
        
        // Reparent children to init
        let children: Vec<Pid> = proc.children.drain(..).collect();
        for child_pid in children {
            if let Some(child) = table.processes.get_mut(&child_pid) {
                child.ppid = PID_INIT;
            }
            if let Some(init) = table.processes.get_mut(&PID_INIT) {
                init.children.push(child_pid);
            }
        }
        
        crate::log_debug!("[PROC] Process {} exited with code {}", current, code);
    }
}

/// Wait for a child process to exit
pub fn wait(pid: Pid) -> Result<i32, &'static str> {
    let mut table = PROCESS_TABLE.write();
    
    let proc = table.processes.get(&pid).ok_or("Process not found")?;
    
    if proc.state != ProcessState::Zombie {
        return Err("Process not yet exited");
    }
    
    let exit_code = proc.exit_code;
    
    // Remove from process table
    table.processes.remove(&pid);
    
    Ok(exit_code)
}

/// Get current process PID
pub fn current_pid() -> Pid {
    CURRENT_PID.load(Ordering::Relaxed)
}

/// Set current process PID (called by scheduler)
pub fn set_current(pid: Pid) {
    CURRENT_PID.store(pid, Ordering::SeqCst);
}

/// Get process by PID
pub fn get(pid: Pid) -> Option<Process> {
    PROCESS_TABLE.read().processes.get(&pid).cloned()
}

/// Get current process
pub fn current() -> Option<Process> {
    get(current_pid())
}

/// Check if a process is in Running state
pub fn is_running(pid: Pid) -> bool {
    PROCESS_TABLE.read().processes.get(&pid)
        .map(|p| p.state == ProcessState::Running)
        .unwrap_or(false)
}

/// Set process state
pub fn set_state(pid: Pid, state: ProcessState) {
    if let Some(proc) = PROCESS_TABLE.write().processes.get_mut(&pid) {
        proc.state = state;
    }
}

/// List all processes
pub fn list() -> Vec<(Pid, String, ProcessState)> {
    PROCESS_TABLE.read()
        .processes
        .iter()
        .map(|(pid, proc)| (*pid, proc.name.clone(), proc.state))
        .collect()
}

/// Get process count
pub fn count() -> usize {
    PROCESS_TABLE.read().processes.len()
}

/// Kill a process
pub fn kill(pid: Pid) -> Result<(), &'static str> {
    if pid == PID_KERNEL || pid == PID_INIT {
        return Err("Cannot kill kernel or init");
    }
    
    let mut table = PROCESS_TABLE.write();
    
    if let Some(proc) = table.processes.get_mut(&pid) {
        proc.state = ProcessState::Dead;
        crate::log_debug!("[PROC] Process {} killed", pid);
        Ok(())
    } else {
        Err("Process not found")
    }
}

/// Spawn a new user process, register it in the process table, and return
/// the assigned PID. The process starts in `Ready` state — the caller is
/// responsible for actually running it (see `exec.rs`).
pub fn spawn(name: &str) -> Result<Pid, &'static str> {
    let ppid = current_pid();
    let pid = create(name, ppid)?;
    
    // Initialize signal state for this process
    crate::signals::init_process(pid);
    
    crate::log!("[PROC] Spawned process {} ({}) under parent {}", pid, name, ppid);
    Ok(pid)
}

/// Mark a process as Running and set it as the current PID.
pub fn start_running(pid: Pid) {
    set_state(pid, ProcessState::Running);
    set_current(pid);
}

/// Record that a process has exited.  Marks it Zombie in the table.
pub fn finish(pid: Pid, exit_code: i32) {
    let mut table = PROCESS_TABLE.write();
    if let Some(proc) = table.processes.get_mut(&pid) {
        proc.state = ProcessState::Zombie;
        proc.exit_code = exit_code;
        crate::log_debug!("[PROC] Process {} exited with code {}", pid, exit_code);
    }
}

/// Reap a zombie process — remove it from the table entirely.
pub fn reap(pid: Pid) {
    let mut table = PROCESS_TABLE.write();
    
    // Reparent children to kernel (PID 0)
    if let Some(proc) = table.processes.get(&pid) {
        let children: Vec<Pid> = proc.children.clone();
        for child_pid in children {
            if let Some(child) = table.processes.get_mut(&child_pid) {
                child.ppid = PID_KERNEL;
            }
        }
    }
    
    table.processes.remove(&pid);
    
    // Clean up signal state
    crate::signals::cleanup_process(pid);
    
    crate::log_debug!("[PROC] Reaped process {}", pid);
}

/// Update a process's memory layout in the table.
pub fn set_memory(pid: Pid, memory: MemoryLayout) {
    if let Some(proc) = PROCESS_TABLE.write().processes.get_mut(&pid) {
        proc.memory = memory;
    }
}

/// Print process tree
pub fn print_tree() {
    let table = PROCESS_TABLE.read();
    
    fn print_process(table: &ProcessTable, pid: Pid, depth: usize) {
        if let Some(proc) = table.processes.get(&pid) {
            let indent: String = (0..depth).map(|_| "  ").collect();
            crate::serial_println!("{}[{}] {} ({:?})", indent, pid, proc.name, proc.state);
            for child in &proc.children {
                print_process(table, *child, depth + 1);
            }
        }
    }
    
    crate::serial_println!("Process tree:");
    print_process(&table, PID_KERNEL, 0);
}

/// Terminate a process immediately
pub fn terminate(pid: Pid) {
    let mut table = PROCESS_TABLE.write();
    if let Some(proc) = table.processes.get_mut(&pid) {
        proc.state = ProcessState::Zombie;
        proc.exit_code = -9; // SIGKILL
    }
}

/// Stop a process (SIGSTOP)
pub fn stop(pid: Pid) {
    let mut table = PROCESS_TABLE.write();
    if let Some(proc) = table.processes.get_mut(&pid) {
        proc.state = ProcessState::Stopped;
    }
}

/// Resume a stopped process (SIGCONT)
pub fn resume(pid: Pid) {
    let mut table = PROCESS_TABLE.write();
    if let Some(proc) = table.processes.get_mut(&pid) {
        if proc.state == ProcessState::Stopped {
            proc.state = ProcessState::Ready;
        }
    }
}

/// Get parent PID
pub fn parent_pid(pid: Pid) -> Option<Pid> {
    let table = PROCESS_TABLE.read();
    table.processes.get(&pid).map(|p| p.ppid)
}
