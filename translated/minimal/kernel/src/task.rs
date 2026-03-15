



use alloc::boxed::Box;
use alloc::collections::VecDeque;
use alloc::string::String;
use alloc::vec::Vec;
use spin::Mutex;
use core::sync::atomic::{AtomicU64, Ordering};


static AFZ_: AtomicU64 = AtomicU64::new(1);


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TaskState {
    At,
    Ai,
    Hj,
    Hh,
}


#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Eg = 0,
    M = 1,
    Ao = 2,
    Aj = 3,
}


pub struct Task {
    pub ad: u64,
    pub j: String,
    pub g: TaskState,
    pub abv: Priority,
    pub cdu: u64,  
    ke: Option<Box<dyn FnOnce() + Send>>,
}

impl Task {
    
    pub fn new<G>(j: &str, abv: Priority, ke: G) -> Self
    where
        G: FnOnce() + Send + 'static,
    {
        Task {
            ad: AFZ_.fetch_add(1, Ordering::SeqCst),
            j: String::from(j),
            g: TaskState::At,
            abv,
            cdu: 0,
            ke: Some(Box::new(ke)),
        }
    }
    
    
    pub fn vw(&mut self) {
        if let Some(ke) = self.ke.take() {
            self.g = TaskState::Ai;
            ke();
            self.g = TaskState::Hh;
        }
    }
}


pub struct Scheduler {
    bcy: VecDeque<Task>,
    knd: Option<u64>,
}

impl Scheduler {
    pub const fn new() -> Self {
        Scheduler {
            bcy: VecDeque::new(),
            knd: None,
        }
    }
    
    
    pub fn eys<G>(&mut self, j: &str, abv: Priority, ke: G) -> u64
    where
        G: FnOnce() + Send + 'static,
    {
        let task = Task::new(j, abv, ke);
        let ad = task.ad;
        self.bcy.agt(task);
        ad
    }
    
    
    pub fn dmj(&self) -> usize {
        self.bcy.len()
    }
    
    
    pub fn liy(&self) -> Vec<(u64, String, TaskState, Priority)> {
        self.bcy.iter()
            .map(|ab| (ab.ad, ab.j.clone(), ab.g, ab.abv))
            .collect()
    }
    
    
    pub fn wbj(&mut self) -> bool {
        
        self.bcy.zbu().bxe(|q, o| o.abv.cmp(&q.abv));
        
        
        if let Some(w) = self.bcy.iter().qf(|ab| ab.g == TaskState::At) {
            if let Some(mut task) = self.bcy.remove(w) {
                self.knd = Some(task.ad);
                task.vw();
                self.knd = None;
                
                
                if task.g != TaskState::Hh {
                    self.bcy.agt(task);
                }
                return true;
            }
        }
        false
    }
    
    
    pub fn yih(&mut self) {
        self.bcy.ajm(|ab| ab.g != TaskState::Hh);
    }
}


static Amd: Mutex<Scheduler> = Mutex::new(Scheduler::new());


pub fn eys<G>(j: &str, abv: Priority, ke: G) -> u64
where
    G: FnOnce() + Send + 'static,
{
    Amd.lock().eys(j, abv, ke)
}


pub fn zpa<G>(j: &str, ke: G) -> u64
where
    G: FnOnce() + Send + 'static,
{
    eys(j, Priority::M, ke)
}


pub fn liy() -> Vec<(u64, String, TaskState, Priority)> {
    Amd.lock().liy()
}


pub fn dmj() -> usize {
    Amd.lock().dmj()
}


pub fn vw() {
    loop {
        if !Amd.lock().wbj() {
            
            core::hint::hc();
        }
    }
}


pub fn gxc() {
    
    core::hint::hc();
}


pub fn init() {
    crate::serial_println!("[TASK] Cooperative scheduler initialized");
}
