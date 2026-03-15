//! Task queues
//! 
//! Lock-free queue implementations for scheduler.

use alloc::collections::VecDeque;
use spin::Mutex;
use super::TaskId;

/// Per-CPU run queue
pub struct RunQueue {
    /// Ready tasks
    queue: Mutex<VecDeque<TaskId>>,
    /// CPU ID this queue belongs to
    cpu_id: u8,
}

impl RunQueue {
    /// Create new run queue for CPU
    pub const fn new(cpu_id: u8) -> Self {
        Self {
            queue: Mutex::new(VecDeque::new()),
            cpu_id,
        }
    }
    
    /// Add task to queue
    pub fn enqueue(&self, task: TaskId) {
        self.queue.lock().push_back(task);
    }
    
    /// Remove and return next task
    pub fn dequeue(&self) -> Option<TaskId> {
        self.queue.lock().pop_front()
    }
    
    /// Check if queue is empty
    pub fn is_empty(&self) -> bool {
        self.queue.lock().is_empty()
    }
    
    /// Get queue length
    pub fn len(&self) -> usize {
        self.queue.lock().len()
    }
    
    /// Get CPU ID
    pub fn cpu(&self) -> u8 {
        self.cpu_id
    }
}

/// NUMA-aware work stealing support
pub trait WorkStealing {
    /// Try to steal work from this queue
    fn steal(&self) -> Option<TaskId>;
}

impl WorkStealing for RunQueue {
    fn steal(&self) -> Option<TaskId> {
        // Steal from back to minimize contention
        self.queue.lock().pop_back()
    }
}
