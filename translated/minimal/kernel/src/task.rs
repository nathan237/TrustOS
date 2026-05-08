



use alloc::boxed::Box;
use alloc::collections::VecDeque;
use alloc::string::String;
use alloc::vec::Vec;
use spin::Mutex;
use core::sync::atomic::{AtomicU64, Ordering};


static AHT_: AtomicU64 = AtomicU64::new(1);


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TaskState {
    Ready,
    Running,
    Blocked,
    Terminated,
}


#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}


pub struct Task {
    pub id: u64,
    pub name: String,
    pub state: TaskState,
    pub priority: Priority,
    pub cpu_time: u64,  
    func: Option<Box<dyn FnOnce() + Send>>,
}

impl Task {
    
    pub fn new<F>(name: &str, priority: Priority, func: F) -> Self
    where
        F: FnOnce() + Send + 'static,
    {
        Task {
            id: AHT_.fetch_add(1, Ordering::SeqCst),
            name: String::from(name),
            state: TaskState::Ready,
            priority,
            cpu_time: 0,
            func: Some(Box::new(func)),
        }
    }
    
    
    pub fn run(&mut self) {
        if let Some(func) = self.func.take() {
            self.state = TaskState::Running;
            func();
            self.state = TaskState::Terminated;
        }
    }
}


pub struct Scheduler {
    tasks: VecDeque<Task>,
    current_task_id: Option<u64>,
}

impl Scheduler {
    pub const fn new() -> Self {
        Scheduler {
            tasks: VecDeque::new(),
            current_task_id: None,
        }
    }
    
    
    pub fn spawn<F>(&mut self, name: &str, priority: Priority, func: F) -> u64
    where
        F: FnOnce() + Send + 'static,
    {
        let task = Task::new(name, priority, func);
        let id = task.id;
        self.tasks.push_back(task);
        id
    }
    
    
    pub fn task_count(&self) -> usize {
        self.tasks.len()
    }
    
    
    pub fn list_tasks(&self) -> Vec<(u64, String, TaskState, Priority)> {
        self.tasks.iter()
            .map(|t| (t.id, t.name.clone(), t.state, t.priority))
            .collect()
    }
    
    
    pub fn run_one(&mut self) -> bool {
        
        self.tasks.make_contiguous().sort_by(|a, b| b.priority.cmp(&a.priority));
        
        
        if let Some(idx) = self.tasks.iter().position(|t| t.state == TaskState::Ready) {
            if let Some(mut task) = self.tasks.remove(idx) {
                self.current_task_id = Some(task.id);
                task.run();
                self.current_task_id = None;
                
                
                if task.state != TaskState::Terminated {
                    self.tasks.push_back(task);
                }
                return true;
            }
        }
        false
    }
    
    
    pub fn pzv(&mut self) {
        self.tasks.retain(|t| t.state != TaskState::Terminated);
    }
}


static Qc: Mutex<Scheduler> = Mutex::new(Scheduler::new());


pub fn spawn<F>(name: &str, priority: Priority, func: F) -> u64
where
    F: FnOnce() + Send + 'static,
{
    Qc.lock().spawn(name, priority, func)
}


pub fn qxh<F>(name: &str, func: F) -> u64
where
    F: FnOnce() + Send + 'static,
{
    spawn(name, Priority::Normal, func)
}


pub fn list_tasks() -> Vec<(u64, String, TaskState, Priority)> {
    Qc.lock().list_tasks()
}


pub fn task_count() -> usize {
    Qc.lock().task_count()
}


pub fn run() {
    loop {
        if !Qc.lock().run_one() {
            
            core::hint::spin_loop();
        }
    }
}


pub fn dgw() {
    
    core::hint::spin_loop();
}


pub fn init() {
    crate::serial_println!("[TASK] Cooperative scheduler initialized");
}
