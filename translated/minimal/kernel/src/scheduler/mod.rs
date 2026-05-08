



mod task;
mod queue;

use spin::Mutex;
use alloc::collections::{VecDeque, BTreeMap};
use core::sync::atomic::{AtomicU64, AtomicBool, Ordering};

pub use task::{Task, TaskId, TaskState, TaskPriority};


static AHT_: AtomicU64 = AtomicU64::new(1);


static Ah: AtomicBool = AtomicBool::new(false);




static AIS_: Mutex<[VecDeque<TaskId>; 4]> = Mutex::new([
    VecDeque::new(), 
    VecDeque::new(), 
    VecDeque::new(), 
    VecDeque::new(), 
]);


static QU_: Mutex<BTreeMap<u64, Task>> = Mutex::new(BTreeMap::new());


static NO_: AtomicU64 = AtomicU64::new(0);


static BJI_: AtomicU64 = AtomicU64::new(0);


const BUB_: u64 = 10;


#[inline(always)]
fn hlc(priority: usize) -> usize {
    if priority > 3 { 0 } else { priority }
}


pub fn init() {
    let gbp = Task::njb();
    NO_.store(gbp.id.0, Ordering::SeqCst);
    QU_.lock().insert(gbp.id.0, gbp);
    Ah.store(true, Ordering::SeqCst);
    crate::log!("Scheduler ready");
}


pub fn nki() -> TaskId {
    TaskId(AHT_.fetch_add(1, Ordering::Relaxed))
}


pub fn spawn(task: Task) -> TaskId {
    let id = task.id;
    let priority = task.priority as usize;
    
    
    QU_.lock().insert(id.0, task);
    
    
    AIS_.lock()[hlc(priority)].push_back(id);
    
    crate::log_debug!("Spawned task {:?} with priority {}", id, priority);
    
    id
}


pub fn dvv() {
    if !Ah.load(Ordering::Relaxed) {
        return;
    }
    
    
    let dly = NO_.load(Ordering::Relaxed);
    if let Some(task) = QU_.lock().get(&dly) {
        task.tick();
    }
    
    let slice = BJI_.fetch_add(1, Ordering::Relaxed);
    
    
    if slice >= BUB_ {
        BJI_.store(0, Ordering::Relaxed);
        boq();
    }
}


pub fn boq() {
    
    let mut zg = AIS_.lock();
    
    
    for priority in (0..4).rev() {
        if let Some(task_id) = zg[priority].pop_front() {
            
            let dly = NO_.load(Ordering::Relaxed);
            if dly != 0 {
                
                let fpv = QU_.lock()
                    .get(&dly)
                    .map(|t| t.priority as usize)
                    .unwrap_or(0);
                zg[hlc(fpv)].push_back(TaskId(dly));
            }
            
            drop(zg); 
            
            NO_.store(task_id.0, Ordering::SeqCst);
            
            crate::trace::akj(
                crate::trace::EventType::ContextSwitch,
                task_id.0
            );
            
            
            crate::lab_mode::trace_bus::emit(
                crate::lab_mode::trace_bus::EventCategory::Scheduler,
                alloc::format!("context switch -> task {}", task_id.0),
                task_id.0,
            );
            
            return;
        }
    }
    
    
}


pub fn byk() -> Option<TaskId> {
    let id = NO_.load(Ordering::Relaxed);
    Some(TaskId(id))
}


pub fn dgw() {
    boq();
}

pub fn qxi(entry: u64) -> u64 {
    crate::log!("Spawn task {:#x}", entry);
    1
}


pub fn stats() -> Aen {
    let zg = AIS_.lock();
    let ready_count = zg[0].len() + zg[1].len()
        + zg[2].len() + zg[3].len();
    Aen {
        ready_count,
        byk: byk(),
    }
}


pub fn qiq(id: TaskId) -> Option<TaskState> {
    QU_.lock().get(&id.0).map(|t| t.state)
}


#[derive(Debug, Clone)]
pub struct Aen {
    pub ready_count: usize,
    pub byk: Option<TaskId>,
}
