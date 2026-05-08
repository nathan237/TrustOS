

use core::sync::atomic::{AtomicU64, Ordering};
use alloc::vec::Vec;
use spin::Mutex;


static ALC_: AtomicU64 = AtomicU64::new(0);


static BLA_: Mutex<Vec<(u64, u64)>> = Mutex::new(Vec::new());


pub fn init() {
    
}


pub fn uptime_ms() -> u64 {
    ALC_.load(Ordering::Relaxed)
}


pub fn cbx() -> u64 {
    uptime_ms() * 1_000_000
}


pub fn yf() -> u64 {
    ALC_.load(Ordering::Relaxed)
}


pub fn tick() {
    ALC_.fetch_add(10, Ordering::Relaxed); 

    
    nyn();
}



pub fn oek(tid: u64, brr: u64) {
    let mut q = BLA_.lock();
    q.push((brr, tid));
}


fn nyn() {
    let cy = cbx();
    let mut q = BLA_.lock();
    
    let mut i = 0;
    while i < q.len() {
        if cy >= q[i].0 {
            let tid = q[i].1;
            q.swap_remove(i);
            
            
            crate::thread::wake(tid);
        } else {
            i += 1;
        }
    }
}


pub fn uptime_secs() -> u64 {
    uptime_ms() / 1000
}