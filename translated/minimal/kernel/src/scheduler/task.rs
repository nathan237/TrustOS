



use core::sync::atomic::{AtomicU64, Ordering};
use alloc::string::String;
use alloc::vec::Vec;


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TaskId(pub u64);

impl TaskId {
    pub const Cfh: TaskId = TaskId(0);
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    
    At,
    
    Ai,
    
    Hj,
    
    Hh,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(usize)]
pub enum TaskPriority {
    
    Eg = 0,
    
    M = 1,
    
    Ao = 2,
    
    Dfs = 3,
}


#[derive(Debug)]
pub struct Task {
    
    pub ad: TaskId,
    
    pub j: String,
    
    pub g: TaskState,
    
    pub abv: TaskPriority,
    
    pub ngl: Option<u8>,
    
    pub bme: Vec<u64>,
    
    pub tu: Option<TaskId>,
    
    pub cdu: AtomicU64,
}

impl Task {
    
    pub fn new(j: String, abv: TaskPriority) -> Self {
        Self {
            ad: super::uup(),
            j,
            g: TaskState::At,
            abv,
            ngl: None,
            bme: Vec::new(),
            tu: super::eoh(),
            cdu: AtomicU64::new(0),
        }
    }
    
    
    pub fn usw() -> Self {
        Self {
            ad: TaskId::Cfh,
            j: String::from("idle"),
            g: TaskState::Ai,
            abv: TaskPriority::Eg,
            ngl: None,
            bme: Vec::new(),
            tu: None,
            cdu: AtomicU64::new(0),
        }
    }
    
    
    pub fn or(&self) {
        self.cdu.fetch_add(1, Ordering::Relaxed);
    }
    
    
    pub fn ysw(&self) -> u64 {
        self.cdu.load(Ordering::Relaxed)
    }
}
