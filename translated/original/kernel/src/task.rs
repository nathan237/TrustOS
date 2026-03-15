//! Simple Cooperative Multitasking
//! 
//! Basic task management with cooperative scheduling.

use alloc::boxed::Box;
use alloc::collections::VecDeque;
use alloc::string::String;
use alloc::vec::Vec;
use spin::Mutex;
use core::sync::atomic::{AtomicU64, Ordering};

/// Task ID counter
static NEXT_TASK_ID: AtomicU64 = AtomicU64::new(1);

/// Task states
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TaskState {
    Ready,
    Running,
    Blocked,
    Terminated,
}

/// Task priority
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

/// Task structure
pub struct Task {
    pub id: u64,
    pub name: String,
    pub state: TaskState,
    pub priority: Priority,
    pub cpu_time: u64,  // Ticks used
    func: Option<Box<dyn FnOnce() + Send>>,
}

impl Task {
    /// Create a new task
    pub fn new<F>(name: &str, priority: Priority, func: F) -> Self
    where
        F: FnOnce() + Send + 'static,
    {
        Task {
            id: NEXT_TASK_ID.fetch_add(1, Ordering::SeqCst),
            name: String::from(name),
            state: TaskState::Ready,
            priority,
            cpu_time: 0,
            func: Some(Box::new(func)),
        }
    }
    
    /// Run the task
    pub fn run(&mut self) {
        if let Some(func) = self.func.take() {
            self.state = TaskState::Running;
            func();
            self.state = TaskState::Terminated;
        }
    }
}

/// Task scheduler
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
    
    /// Add a task to the scheduler
    pub fn spawn<F>(&mut self, name: &str, priority: Priority, func: F) -> u64
    where
        F: FnOnce() + Send + 'static,
    {
        let task = Task::new(name, priority, func);
        let id = task.id;
        self.tasks.push_back(task);
        id
    }
    
    /// Get number of tasks
    pub fn task_count(&self) -> usize {
        self.tasks.len()
    }
    
    /// List all tasks
    pub fn list_tasks(&self) -> Vec<(u64, String, TaskState, Priority)> {
        self.tasks.iter()
            .map(|t| (t.id, t.name.clone(), t.state, t.priority))
            .collect()
    }
    
    /// Run one task (cooperative)
    pub fn run_one(&mut self) -> bool {
        // Sort by priority (higher first)
        self.tasks.make_contiguous().sort_by(|a, b| b.priority.cmp(&a.priority));
        
        // Find first ready task
        if let Some(idx) = self.tasks.iter().position(|t| t.state == TaskState::Ready) {
            if let Some(mut task) = self.tasks.remove(idx) {
                self.current_task_id = Some(task.id);
                task.run();
                self.current_task_id = None;
                
                // Re-add if not terminated
                if task.state != TaskState::Terminated {
                    self.tasks.push_back(task);
                }
                return true;
            }
        }
        false
    }
    
    /// Remove terminated tasks
    pub fn cleanup(&mut self) {
        self.tasks.retain(|t| t.state != TaskState::Terminated);
    }
}

/// Global scheduler
static SCHEDULER: Mutex<Scheduler> = Mutex::new(Scheduler::new());

/// Spawn a new task
pub fn spawn<F>(name: &str, priority: Priority, func: F) -> u64
where
    F: FnOnce() + Send + 'static,
{
    SCHEDULER.lock().spawn(name, priority, func)
}

/// Spawn a normal priority task
pub fn spawn_normal<F>(name: &str, func: F) -> u64
where
    F: FnOnce() + Send + 'static,
{
    spawn(name, Priority::Normal, func)
}

/// List all tasks
pub fn list_tasks() -> Vec<(u64, String, TaskState, Priority)> {
    SCHEDULER.lock().list_tasks()
}

/// Get task count
pub fn task_count() -> usize {
    SCHEDULER.lock().task_count()
}

/// Run scheduler (cooperative)
pub fn run() {
    loop {
        if !SCHEDULER.lock().run_one() {
            // No tasks ready, yield CPU
            core::hint::spin_loop();
        }
    }
}

/// Yield to scheduler (cooperative multitasking)
pub fn yield_now() {
    // In cooperative scheduling, this is a hint to switch tasks
    core::hint::spin_loop();
}

/// Initialize scheduler with kernel task
pub fn init() {
    crate::serial_println!("[TASK] Cooperative scheduler initialized");
}
