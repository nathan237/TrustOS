



use core::sync::atomic::{AtomicU64, Ordering};
use alloc::string::String;
use alloc::vec::Vec;


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TaskId(pub u64);

impl TaskId {
    pub const Alg: TaskId = TaskId(0);
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    
    Ready,
    
    Running,
    
    Blocked,
    
    Terminated,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(usize)]
pub enum TaskPriority {
    
    Low = 0,
    
    Normal = 1,
    
    High = 2,
    
    RealTime = 3,
}


#[derive(Debug)]
pub struct Task {
    
    pub id: TaskId,
    
    pub name: String,
    
    pub state: TaskState,
    
    pub priority: TaskPriority,
    
    pub cpu_affinity: Option<u8>,
    
    pub capabilities: Vec<u64>,
    
    pub parent: Option<TaskId>,
    
    pub cpu_time: AtomicU64,
}

impl Task {
    
    pub fn new(name: String, priority: TaskPriority) -> Self {
        Self {
            id: super::nki(),
            name,
            state: TaskState::Ready,
            priority,
            cpu_affinity: None,
            capabilities: Vec::new(),
            parent: super::byk(),
            cpu_time: AtomicU64::new(0),
        }
    }
    
    
    pub fn njb() -> Self {
        Self {
            id: TaskId::Alg,
            name: String::from("idle"),
            state: TaskState::Running,
            priority: TaskPriority::Low,
            cpu_affinity: None,
            capabilities: Vec::new(),
            parent: None,
            cpu_time: AtomicU64::new(0),
        }
    }
    
    
    pub fn tick(&self) {
        self.cpu_time.fetch_add(1, Ordering::Relaxed);
    }
    
    
    pub fn qhl(&self) -> u64 {
        self.cpu_time.load(Ordering::Relaxed)
    }
}
