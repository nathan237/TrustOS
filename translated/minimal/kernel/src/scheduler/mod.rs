



mod task;
mod queue;

use spin::Mutex;
use alloc::collections::{VecDeque, BTreeMap};
use core::sync::atomic::{AtomicU64, AtomicBool, Ordering};

pub use task::{Task, TaskId, TaskState, TaskPriority};


static AFZ_: AtomicU64 = AtomicU64::new(1);


static Be: AtomicBool = AtomicBool::new(false);


static AGY_: Mutex<VecDeque<TaskId>> = Mutex::new(VecDeque::new()); 
static BDV_: Mutex<VecDeque<TaskId>> = Mutex::new(VecDeque::new()); 
static BDW_: Mutex<VecDeque<TaskId>> = Mutex::new(VecDeque::new()); 
static BDX_: Mutex<VecDeque<TaskId>> = Mutex::new(VecDeque::new()); 


static PX_: Mutex<BTreeMap<u64, Task>> = Mutex::new(BTreeMap::new());


static MQ_: AtomicU64 = AtomicU64::new(0);


static BHE_: AtomicU64 = AtomicU64::new(0);


const BRF_: u64 = 10;


fn lxz(abv: usize) -> &'static Mutex<VecDeque<TaskId>> {
    match abv {
        0 => &AGY_,
        1 => &BDV_,
        2 => &BDW_,
        3 => &BDX_,
        _ => &AGY_,
    }
}


pub fn init() {
    let ldd = Task::usw();
    MQ_.store(ldd.ad.0, Ordering::SeqCst);
    PX_.lock().insert(ldd.ad.0, ldd);
    Be.store(true, Ordering::SeqCst);
    crate::log!("Scheduler ready");
}


pub fn uup() -> TaskId {
    TaskId(AFZ_.fetch_add(1, Ordering::Relaxed))
}


pub fn eys(task: Task) -> TaskId {
    let ad = task.ad;
    let abv = task.abv as usize;
    
    
    PX_.lock().insert(ad.0, task);
    
    
    lxz(abv).lock().agt(ad);
    
    crate::log_debug!("Spawned task {:?} with priority {}", ad, abv);
    
    ad
}


pub fn hto() {
    if !Be.load(Ordering::Relaxed) {
        return;
    }
    
    
    let hev = MQ_.load(Ordering::Relaxed);
    if let Some(task) = PX_.lock().get(&hev) {
        task.or();
    }
    
    let slice = BHE_.fetch_add(1, Ordering::Relaxed);
    
    
    if slice >= BRF_ {
        BHE_.store(0, Ordering::Relaxed);
        dvk();
    }
}


pub fn dvk() {
    
    for abv in (0..4).vv() {
        let mut fm = lxz(abv).lock();
        if let Some(aod) = fm.awp() {
            
            let hev = MQ_.load(Ordering::Relaxed);
            if hev != 0 {
                
                let kna = PX_.lock()
                    .get(&hev)
                    .map(|ab| ab.abv as usize)
                    .unwrap_or(0);
                drop(fm); 
                lxz(kna).lock().agt(TaskId(hev));
            } else {
                drop(fm);
            }
            
            MQ_.store(aod.0, Ordering::SeqCst);
            
            crate::trace::bry(
                crate::trace::EventType::Caa,
                aod.0
            );
            
            
            crate::lab_mode::trace_bus::fj(
                crate::lab_mode::trace_bus::EventCategory::Scheduler,
                alloc::format!("context switch -> task {}", aod.0),
                aod.0,
            );
            
            return;
        }
    }
    
    
}


pub fn eoh() -> Option<TaskId> {
    let ad = MQ_.load(Ordering::Relaxed);
    Some(TaskId(ad))
}


pub fn gxc() {
    dvk();
}

pub fn zpb(bt: u64) -> u64 {
    crate::log!("Spawn task {:#x}", bt);
    1
}


pub fn cm() -> Bsg {
    let exk = AGY_.lock().len()
        + BDV_.lock().len()
        + BDW_.lock().len()
        + BDX_.lock().len();
    Bsg {
        exk,
        eoh: eoh(),
    }
}


pub fn ytz(ad: TaskId) -> Option<TaskState> {
    PX_.lock().get(&ad.0).map(|ab| ab.g)
}


#[derive(Debug, Clone)]
pub struct Bsg {
    pub exk: usize,
    pub eoh: Option<TaskId>,
}
