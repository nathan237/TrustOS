//! Scheduler Subsystem
//! 
//! Per-core, NUMA-aware thread scheduler with priority-based scheduling.

mod task;
mod queue;

use spin::Mutex;
use alloc::collections::VecDeque;
use core::sync::atomic::{AtomicU64, AtomicBool, Ordering};

pub use task::{Task, TaskId, TaskState, TaskPriority};

/// Global task ID counter
static NEXT_TASK_ID: AtomicU64 = AtomicU64::new(1);

/// Scheduler initialized flag
static INITIALIZED: AtomicBool = AtomicBool::new(false);

/// Ready queue per priority level
static READY_QUEUES: Mutex<[VecDeque<TaskId>; 4]> = Mutex::new([
    VecDeque::new(),
    VecDeque::new(),
    VecDeque::new(),
    VecDeque::new(),
]);

/// Current running task per CPU
static CURRENT_TASK: Mutex<Option<TaskId>> = Mutex::new(None);

/// Time slice counter
static TIME_SLICE: AtomicU64 = AtomicU64::new(0);

/// Default time quantum (in timer ticks)
const DEFAULT_QUANTUM: u64 = 10;

/// Initialize scheduler
pub fn init() {
    let idle_task = Task::new_idle();
    *CURRENT_TASK.lock() = Some(idle_task.id);
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
    
    // Add to ready queue
    READY_QUEUES.lock()[priority].push_back(id);
    
    crate::log_debug!("Spawned task {:?} with priority {:?}", id, task.priority);
    
    id
}

/// Called on every timer tick
pub fn on_timer_tick() {
    if !INITIALIZED.load(Ordering::Relaxed) {
        return;
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
    let mut queues = READY_QUEUES.lock();
    
    // Priority-based selection (higher priority first)
    for priority in (0..4).rev() {
        if let Some(task_id) = queues[priority].pop_front() {
            // Put current task back in queue
            if let Some(current) = *CURRENT_TASK.lock() {
                if current.0 != 0 {
                    // Don't re-queue idle task
                    queues[priority].push_back(current);
                }
            }
            
            *CURRENT_TASK.lock() = Some(task_id);
            
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
    *CURRENT_TASK.lock()
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
    let queues = READY_QUEUES.lock();
    SchedulerStats {
        ready_count: queues.iter().map(|q| q.len()).sum(),
        current_task: *CURRENT_TASK.lock(),
    }
}

/// Scheduler statistics
#[derive(Debug, Clone)]
pub struct SchedulerStats {
    pub ready_count: usize,
    pub current_task: Option<TaskId>,
}
