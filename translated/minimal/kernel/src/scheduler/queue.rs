



use alloc::collections::VecDeque;
use spin::Mutex;
use super::TaskId;


pub struct Adp {
    
    queue: Mutex<VecDeque<TaskId>>,
    
    cpu_id: u8,
}

impl Adp {
    
    pub const fn new(cpu_id: u8) -> Self {
        Self {
            queue: Mutex::new(VecDeque::new()),
            cpu_id,
        }
    }
    
    
    pub fn enqueue(&self, task: TaskId) {
        self.queue.lock().push_back(task);
    }
    
    
    pub fn qcp(&self) -> Option<TaskId> {
        self.queue.lock().pop_front()
    }
    
    
    pub fn is_empty(&self) -> bool {
        self.queue.lock().is_empty()
    }
    
    
    pub fn len(&self) -> usize {
        self.queue.lock().len()
    }
    
    
    pub fn cpu(&self) -> u8 {
        self.cpu_id
    }
}


pub trait Asc {
    
    fn jit(&self) -> Option<TaskId>;
}

impl Asc for Adp {
    fn jit(&self) -> Option<TaskId> {
        
        self.queue.lock().pop_back()
    }
}
