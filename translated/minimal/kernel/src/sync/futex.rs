




use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU32, Ordering};
use spin::Mutex;


static KX_: Mutex<BTreeMap<u64, Vec<Ago>>> = Mutex::new(BTreeMap::new());


#[derive(Debug, Clone)]
struct Ago {
    
    tid: u64,
    
    expected: u32,
    
    start_time: u64,
    
    timeout_ns: u64,
}


pub mod op {
    pub const AED_: u32 = 0;
    pub const AEE_: u32 = 1;
    pub const DQB_: u32 = 2;
    pub const BYZ_: u32 = 3;
    pub const BYY_: u32 = 4;
    pub const DQF_: u32 = 5;
    pub const DQC_: u32 = 6;
    pub const DQE_: u32 = 7;
    pub const DQD_: u32 = 8;
    pub const BZA_: u32 = 9;
    pub const BZB_: u32 = 10;
    
    pub const AEC_: u32 = 128;
    pub const BYW_: u32 = 256;
    
    pub const BYX_: u32 = !(AEC_ | BYW_);
}










pub fn futex(
    uaddr: u64,
    futex_op: u32,
    val: u32,
    mz: u64,
    uaddr2: u64,
    val3: u32,
) -> Result<i64, i32> {
    let cmd = futex_op & op::BYX_;
    
    match cmd {
        op::AED_ => iam(uaddr, val, mz),
        op::AEE_ => ian(uaddr, val),
        op::BYZ_ => ial(uaddr, val, uaddr2, val3),
        op::BYY_ => mao(uaddr, val, uaddr2, val3, mz as u32),
        op::BZA_ => maq(uaddr, val, mz, val3),
        op::BZB_ => mar(uaddr, val, val3),
        _ => Err(-38), 
    }
}


fn iam(uaddr: u64, expected: u32, timeout_ns: u64) -> Result<i64, i32> {
    
    if !crate::memory::ux(uaddr) && uaddr != 0 {
        
        if uaddr < 0xFFFF_8000_0000_0000 {
            return Err(-14); 
        }
    }
    
    
    let current = unsafe { 
        let ptr = uaddr as *const AtomicU32;
        (*ptr).load(Ordering::SeqCst)
    };
    
    
    if current != expected {
        return Err(-11); 
    }
    
    
    let tid = crate::thread::current_tid();
    let cy = crate::time::cbx();
    
    
    {
        let mut zg = KX_.lock();
        let queue = zg.entry(uaddr).or_insert_with(Vec::new);
        queue.push(Ago {
            tid,
            expected,
            start_time: cy,
            timeout_ns,
        });
    }
    
    
    
    
    if timeout_ns > 0 {
        let brq = cy.saturating_add(timeout_ns);
        crate::thread::cds(brq);
    } else {
        
        crate::thread::hig();
    }
    
    
    
    {
        let mut zg = KX_.lock();
        if let Some(queue) = zg.get_mut(&uaddr) {
            if queue.iter().any(|e| e.tid == tid) {
                
                queue.retain(|e| e.tid != tid);
                if queue.is_empty() {
                    zg.remove(&uaddr);
                }
                return Err(-110); 
            }
        }
    }
    
    
    Ok(0)
}


fn ian(uaddr: u64, count: u32) -> Result<i64, i32> {
    let mut zg = KX_.lock();
    
    let csw = if let Some(queue) = zg.get_mut(&uaddr) {
        let gzf = (count as usize).min(queue.len());
        
        
        let csw: Vec<_> = queue.drain(..gzf).collect();
        
        
        for entry in &csw {
            crate::thread::wake(entry.tid);
        }
        
        if queue.is_empty() {
            zg.remove(&uaddr);
        }
        
        csw.len() as i64
    } else {
        0
    };
    
    Ok(csw)
}


fn ial(uaddr: u64, wake_count: u32, uaddr2: u64, requeue_count: u32) -> Result<i64, i32> {
    let mut zg = KX_.lock();
    
    let mut jod = 0i64;
    
    if let Some(queue) = zg.get_mut(&uaddr) {
        
        let gzf = (wake_count as usize).min(queue.len());
        let csw: Vec<_> = queue.drain(..gzf).collect();
        
        for entry in &csw {
            crate::thread::wake(entry.tid);
        }
        jod = csw.len() as i64;
        
        
        let pkn = (requeue_count as usize).min(queue.len());
        let ogc: Vec<_> = queue.drain(..pkn).collect();
        
        
        let oap = zg.entry(uaddr2).or_insert_with(Vec::new);
        oap.extend(ogc);
    }
    
    
    if zg.get(&uaddr).map(|q| q.is_empty()).unwrap_or(false) {
        zg.remove(&uaddr);
    }
    
    Ok(jod)
}


fn mao(uaddr: u64, wake_count: u32, uaddr2: u64, requeue_count: u32, expected: u32) -> Result<i64, i32> {
    
    let current = unsafe { 
        let ptr = uaddr as *const AtomicU32;
        (*ptr).load(Ordering::SeqCst)
    };
    if current != expected {
        return Err(-11); 
    }
    
    ial(uaddr, wake_count, uaddr2, requeue_count)
}


fn maq(uaddr: u64, expected: u32, timeout_ns: u64, bitset: u32) -> Result<i64, i32> {
    if bitset == 0 {
        return Err(-22); 
    }
    
    
    
    iam(uaddr, expected, timeout_ns)
}


fn mar(uaddr: u64, count: u32, bitset: u32) -> Result<i64, i32> {
    if bitset == 0 {
        return Err(-22); 
    }
    
    
    ian(uaddr, count)
}


pub fn qiy(uaddr: u64) -> usize {
    KX_.lock()
        .get(&uaddr)
        .map(|q| q.len())
        .unwrap_or(0)
}


pub fn flu(pid: u64) {
    let mut zg = KX_.lock();
    
    
    for queue in zg.values_mut() {
        queue.retain(|e| (e.tid >> 32) != pid);
    }
    
    
    zg.retain(|_, q| !q.is_empty());
}
