

use core::sync::atomic::{AtomicU64, Ordering};
use alloc::vec::Vec;
use spin::Mutex;


static AJH_: AtomicU64 = AtomicU64::new(0);


static BIU_: Mutex<Vec<(u64, u64)>> = Mutex::new(Vec::new());


pub fn init() {
    
}


pub fn lc() -> u64 {
    AJH_.load(Ordering::Relaxed)
}


pub fn evk() -> u64 {
    lc() * 1_000_000
}


pub fn ave() -> u64 {
    AJH_.load(Ordering::Relaxed)
}


pub fn or() {
    AJH_.fetch_add(10, Ordering::Relaxed); 

    
    vmw();
}



pub fn vuf(ni: u64, eao: u64) {
    let mut fm = BIU_.lock();
    fm.push((eao, ni));
}


fn vmw() {
    let iu = evk();
    let mut fm = BIU_.lock();
    
    let mut a = 0;
    while a < fm.len() {
        if iu >= fm[a].0 {
            let ni = fm[a].1;
            fm.zqh(a);
            
            
            crate::thread::wake(ni);
        } else {
            a += 1;
        }
    }
}


pub fn cnn() -> u64 {
    lc() / 1000
}