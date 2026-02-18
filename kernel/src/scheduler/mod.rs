//! Scheduler Subsystem
//! 
//! Per-core, NUMA-aware thread scheduler with priority-based scheduling.

mod task;
mod queue;

use spin::Mutex;
use alloc::collections::{VecDeque, BTreeMap};
use core::sync::atomic::{AtomicU64, AtomicBool, Ordering};

pub use task::{Task, TaskId, TaskState, TaskPriority};

/// Global task ID counter
static NEXT_TASK_ID: AtomicU64 = AtomicU64::new(1);

/// Scheduler initialized flag
static INITIALIZED: AtomicBool = AtomicBool::new(false);

/// Per-priority ready queues (independent locks for reduced contention)
static READY_Q0: Mutex<VecDeque<TaskId>> = Mutex::new(VecDeque::new()); // Low
static READY_Q1: Mutex<VecDeque<TaskId>> = Mutex::new(VecDeque::new()); // Normal
static READY_Q2: Mutex<VecDeque<TaskId>> = Mutex::new(VecDeque::new()); // High
static READY_Q3: Mutex<VecDeque<TaskId>> = Mutex::new(VecDeque::new()); // RealTime

/// Task registry — stores all known tasks for lookup/management
static TASK_REGISTRY: Mutex<BTreeMap<u64, Task>> = Mutex::new(BTreeMap::new());

/// Current running task per CPU (AtomicU64 avoids mutex overhead on hot path)
static CURRENT_TASK: AtomicU64 = AtomicU64::new(0);

/// Time slice counter
static TIME_SLICE: AtomicU64 = AtomicU64::new(0);

/// Default time quantum (in timer ticks)
const DEFAULT_QUANTUM: u64 = 10;

/// Helper: get the mutex for a given priority level
fn ready_queue(priority: usize) -> &'static Mutex<VecDeque<TaskId>> {
    match priority {
        0 => &READY_Q0,
        1 => &READY_Q1,
        2 => &READY_Q2,
        3 => &READY_Q3,
        _ => &READY_Q0,
    }
}

/// Initialize scheduler
pub fn init() {
    let idle_task = Task::new_idle();
    CURRENT_TASK.store(idle_task.id.0, Ordering::SeqCst);
    TASK_REGISTRY.lock().insert(idle_task.id.0, idle_task);
    INITIALIZED.store(true, Ordering::SeqCst);
    crate::log!("Scheduler ready");
}

/// Generate new unique task ID
pub fn next_task_id() -> TaskId {
    TaskId(NEXT_TASK_ID.fetch_add(1, Ordering::Relaxed))
}

/// Spawn a new task
pub fn spawn(task: Task) -> TaskId {
    let id = task.id;
    let priority = task.priority as usize;
    
    // Register task in the task registry
    TASK_REGISTRY.lock().insert(id.0, task);
    
    // Add to the appropriate priority queue
    ready_queue(priority).lock().push_back(id);
    
    crate::log_debug!("Spawned task {:?} with priority {}", id, priority);
    
    id
}

/// Called on every timer tick
pub fn on_timer_tick() {
    if !INITIALIZED.load(Ordering::Relaxed) {
        return;
    }
    
    // Tick CPU time for current task
    let current_id = CURRENT_TASK.load(Ordering::Relaxed);
    if let Some(task) = TASK_REGISTRY.lock().get(&current_id) {
        task.tick();
    }
    
    let slice = TIME_SLICE.fetch_add(1, Ordering::Relaxed);
    
    // Check if time quantum expired
    if slice >= DEFAULT_QUANTUM {
        TIME_SLICE.store(0, Ordering::Relaxed);
        schedule();
    }
}

/// Run scheduler - select next task to run
pub fn schedule() {
    // Priority-based selection (higher priority = higher index = checked first)
    for priority in (0..4).rev() {
        let mut q = ready_queue(priority).lock();
        if let Some(task_id) = q.pop_front() {
            // Put current task back in its queue
            let current_id = CURRENT_TASK.load(Ordering::Relaxed);
            if current_id != 0 {
                // Don't re-queue idle task; re-queue at same priority
                let current_priority = TASK_REGISTRY.lock()
                    .get(&current_id)
                    .map(|t| t.priority as usize)
                    .unwrap_or(0);
                drop(q); // drop current queue lock before locking another
                ready_queue(current_priority).lock().push_back(TaskId(current_id));
            } else {
                drop(q);
            }
            
            CURRENT_TASK.store(task_id.0, Ordering::SeqCst);
            
            crate::trace::record_event(
                crate::trace::EventType::ContextSwitch,
                task_id.0
            );
            
            // TrustLab trace
            crate::lab_mode::trace_bus::emit(
                crate::lab_mode::trace_bus::EventCategory::Scheduler,
                alloc::format!("context switch -> task {}", task_id.0),
                task_id.0,
            );
            
            return;
        }
    }
    
    // No ready tasks, continue with current or idle
}

/// Get current task ID
pub fn current_task() -> Option<TaskId> {
    let id = CURRENT_TASK.load(Ordering::Relaxed);
    Some(TaskId(id))
}

/// Yield current task
pub fn yield_now() {
    schedule();
}

pub fn spawn_task(entry: u64) -> u64 {
    crate::log!("Spawn task {:#x}", entry);
    1
}

/// Get scheduler statistics
pub fn stats() -> SchedulerStats {
    let ready_count = READY_Q0.lock().len()
        + READY_Q1.lock().len()
        + READY_Q2.lock().len()
        + READY_Q3.lock().len();
    SchedulerStats {
        ready_count,
        current_task: current_task(),
    }
}

/// Look up a task by ID
pub fn get_task(id: TaskId) -> Option<TaskState> {
    TASK_REGISTRY.lock().get(&id.0).map(|t| t.state)
}

/// Scheduler statistics
#[derive(Debug, Clone)]
pub struct SchedulerStats {
    pub ready_count: usize,
    pub current_task: Option<TaskId>,
}
