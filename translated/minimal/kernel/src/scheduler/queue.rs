



use alloc::collections::VecDeque;
use spin::Mutex;
use super::TaskId;


pub struct Brg {
    
    queue: Mutex<VecDeque<TaskId>>,
    
    qq: u8,
}

impl Brg {
    
    pub const fn new(qq: u8) -> Self {
        Self {
            queue: Mutex::new(VecDeque::new()),
            qq,
        }
    }
    
    
    pub fn ggg(&self, task: TaskId) {
        self.queue.lock().agt(task);
    }
    
    
    pub fn ylq(&self) -> Option<TaskId> {
        self.queue.lock().awp()
    }
    
    
    pub fn is_empty(&self) -> bool {
        self.queue.lock().is_empty()
    }
    
    
    pub fn len(&self) -> usize {
        self.queue.lock().len()
    }
    
    
    pub fn cpu(&self) -> u8 {
        self.qq
    }
}


pub trait Cqo {
    
    fn por(&self) -> Option<TaskId>;
}

impl Cqo for Brg {
    fn por(&self) -> Option<TaskId> {
        
        self.queue.lock().owo()
    }
}
