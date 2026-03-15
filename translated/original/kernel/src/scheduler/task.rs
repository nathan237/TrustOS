//! Task structures and management
//! 
//! Defines task control blocks and related types.

use core::sync::atomic::{AtomicU64, Ordering};
use alloc::string::String;
use alloc::vec::Vec;

/// Unique task identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TaskId(pub u64);

impl TaskId {
    pub const IDLE: TaskId = TaskId(0);
}

/// Task execution state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    /// Ready to run
    Ready,
    /// Currently executing
    Running,
    /// Waiting for event
    Blocked,
    /// Terminated
    Terminated,
}

/// Task priority level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(usize)]
pub enum TaskPriority {
    /// Lowest priority (background tasks)
    Low = 0,
    /// Normal priority
    Normal = 1,
    /// High priority (interactive)
    High = 2,
    /// Real-time priority
    RealTime = 3,
}

/// Task control block
#[derive(Debug)]
pub struct Task {
    /// Unique task ID
    pub id: TaskId,
    /// Task name for debugging
    pub name: String,
    /// Current state
    pub state: TaskState,
    /// Priority level
    pub priority: TaskPriority,
    /// CPU affinity (NUMA-aware)
    pub cpu_affinity: Option<u8>,
    /// Capability tokens held by this task
    pub capabilities: Vec<u64>,
    /// Parent task ID
    pub parent: Option<TaskId>,
    /// Total CPU time consumed (in ticks)
    pub cpu_time: AtomicU64,
}

impl Task {
    /// Create new task
    pub fn new(name: String, priority: TaskPriority) -> Self {
        Self {
            id: super::next_task_id(),
            name,
            state: TaskState::Ready,
            priority,
            cpu_affinity: None,
            capabilities: Vec::new(),
            parent: super::current_task(),
            cpu_time: AtomicU64::new(0),
        }
    }
    
    /// Create idle task (task 0)
    pub fn new_idle() -> Self {
        Self {
            id: TaskId::IDLE,
            name: String::from("idle"),
            state: TaskState::Running,
            priority: TaskPriority::Low,
            cpu_affinity: None,
            capabilities: Vec::new(),
            parent: None,
            cpu_time: AtomicU64::new(0),
        }
    }
    
    /// Increment CPU time
    pub fn tick(&self) {
        self.cpu_time.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Get total CPU time
    pub fn get_cpu_time(&self) -> u64 {
        self.cpu_time.load(Ordering::Relaxed)
    }
}
