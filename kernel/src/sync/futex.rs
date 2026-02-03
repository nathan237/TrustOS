//! Fast Userspace Mutex (Futex)
//!
//! High-performance synchronization primitive for userspace threads.
//! Compatible with Linux futex syscall interface.

use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU32, Ordering};
use spin::Mutex;

/// Futex wait queue - maps address to waiting threads
static FUTEX_QUEUES: Mutex<BTreeMap<u64, Vec<WaitEntry>>> = Mutex::new(BTreeMap::new());

/// Wait entry in futex queue
#[derive(Debug, Clone)]
struct WaitEntry {
    /// Thread/Task ID
    tid: u64,
    /// Expected value when waiting
    expected: u32,
    /// Timestamp when wait started
    start_time: u64,
    /// Timeout (0 = infinite)
    timeout_ns: u64,
}

/// Futex operation codes (Linux compatible)
pub mod op {
    pub const FUTEX_WAIT: u32 = 0;
    pub const FUTEX_WAKE: u32 = 1;
    pub const FUTEX_FD: u32 = 2;
    pub const FUTEX_REQUEUE: u32 = 3;
    pub const FUTEX_CMP_REQUEUE: u32 = 4;
    pub const FUTEX_WAKE_OP: u32 = 5;
    pub const FUTEX_LOCK_PI: u32 = 6;
    pub const FUTEX_UNLOCK_PI: u32 = 7;
    pub const FUTEX_TRYLOCK_PI: u32 = 8;
    pub const FUTEX_WAIT_BITSET: u32 = 9;
    pub const FUTEX_WAKE_BITSET: u32 = 10;
    
    pub const FUTEX_PRIVATE_FLAG: u32 = 128;
    pub const FUTEX_CLOCK_REALTIME: u32 = 256;
    
    pub const FUTEX_CMD_MASK: u32 = !(FUTEX_PRIVATE_FLAG | FUTEX_CLOCK_REALTIME);
}

/// Futex syscall handler
/// 
/// # Arguments
/// * `uaddr` - User address of the futex word
/// * `futex_op` - Operation to perform
/// * `val` - Value for the operation
/// * `timeout` - Timeout (optional, operation dependent)
/// * `uaddr2` - Second address (for requeue operations)
/// * `val3` - Additional value (for some operations)
pub fn futex(
    uaddr: u64,
    futex_op: u32,
    val: u32,
    timeout: u64,
    uaddr2: u64,
    val3: u32,
) -> Result<i64, i32> {
    let cmd = futex_op & op::FUTEX_CMD_MASK;
    
    match cmd {
        op::FUTEX_WAIT => futex_wait(uaddr, val, timeout),
        op::FUTEX_WAKE => futex_wake(uaddr, val),
        op::FUTEX_REQUEUE => futex_requeue(uaddr, val, uaddr2, val3),
        op::FUTEX_CMP_REQUEUE => futex_cmp_requeue(uaddr, val, uaddr2, val3, timeout as u32),
        op::FUTEX_WAIT_BITSET => futex_wait_bitset(uaddr, val, timeout, val3),
        op::FUTEX_WAKE_BITSET => futex_wake_bitset(uaddr, val, val3),
        _ => Err(-38), // ENOSYS
    }
}

/// Wait on futex if value matches expected
fn futex_wait(uaddr: u64, expected: u32, timeout_ns: u64) -> Result<i64, i32> {
    // Validate user address
    if !crate::memory::is_user_address(uaddr) && uaddr != 0 {
        // Allow kernel addresses for kernel futexes
        if uaddr < 0xFFFF_8000_0000_0000 {
            return Err(-14); // EFAULT
        }
    }
    
    // Read current value atomically
    let current = unsafe { 
        let ptr = uaddr as *const AtomicU32;
        (*ptr).load(Ordering::SeqCst)
    };
    
    // Check if value matches
    if current != expected {
        return Err(-11); // EAGAIN - value changed
    }
    
    // Get current thread ID
    let tid = crate::thread::current_tid();
    let now = crate::time::now_ns();
    
    // Add to wait queue
    {
        let mut queues = FUTEX_QUEUES.lock();
        let queue = queues.entry(uaddr).or_insert_with(Vec::new);
        queue.push(WaitEntry {
            tid,
            expected,
            start_time: now,
            timeout_ns,
        });
    }
    
    // Block thread
    // In a real implementation, this would context switch
    // For now, we spin-wait with yield
    let deadline = if timeout_ns > 0 { now + timeout_ns } else { u64::MAX };
    
    loop {
        // Check if we've been woken
        {
            let queues = FUTEX_QUEUES.lock();
            if let Some(queue) = queues.get(&uaddr) {
                if !queue.iter().any(|e| e.tid == tid) {
                    // We were removed from queue = woken
                    return Ok(0);
                }
            } else {
                // Queue gone = we were woken
                return Ok(0);
            }
        }
        
        // Check timeout
        let now = crate::time::now_ns();
        if now >= deadline {
            // Remove ourselves from queue
            let mut queues = FUTEX_QUEUES.lock();
            if let Some(queue) = queues.get_mut(&uaddr) {
                queue.retain(|e| e.tid != tid);
                if queue.is_empty() {
                    queues.remove(&uaddr);
                }
            }
            return Err(-110); // ETIMEDOUT
        }
        
        // Yield to other tasks
        crate::scheduler::yield_now();
        
        // Also check if value changed
        let current = unsafe { 
            let ptr = uaddr as *const AtomicU32;
            (*ptr).load(Ordering::SeqCst)
        };
        if current != expected {
            // Remove from queue
            let mut queues = FUTEX_QUEUES.lock();
            if let Some(queue) = queues.get_mut(&uaddr) {
                queue.retain(|e| e.tid != tid);
                if queue.is_empty() {
                    queues.remove(&uaddr);
                }
            }
            return Ok(0);
        }
    }
}

/// Wake up to `count` waiters on futex
fn futex_wake(uaddr: u64, count: u32) -> Result<i64, i32> {
    let mut queues = FUTEX_QUEUES.lock();
    
    let woken = if let Some(queue) = queues.get_mut(&uaddr) {
        let to_wake = (count as usize).min(queue.len());
        
        // Remove first `to_wake` entries
        let woken: Vec<_> = queue.drain(..to_wake).collect();
        
        // Wake each thread
        for entry in &woken {
            // In real implementation, this would unblock the thread
            crate::thread::wake(entry.tid);
        }
        
        if queue.is_empty() {
            queues.remove(&uaddr);
        }
        
        woken.len() as i64
    } else {
        0
    };
    
    Ok(woken)
}

/// Requeue waiters from one futex to another
fn futex_requeue(uaddr: u64, wake_count: u32, uaddr2: u64, requeue_count: u32) -> Result<i64, i32> {
    let mut queues = FUTEX_QUEUES.lock();
    
    let mut total_woken = 0i64;
    
    if let Some(queue) = queues.get_mut(&uaddr) {
        // Wake first `wake_count`
        let to_wake = (wake_count as usize).min(queue.len());
        let woken: Vec<_> = queue.drain(..to_wake).collect();
        
        for entry in &woken {
            crate::thread::wake(entry.tid);
        }
        total_woken = woken.len() as i64;
        
        // Requeue next `requeue_count`
        let to_requeue = (requeue_count as usize).min(queue.len());
        let requeued: Vec<_> = queue.drain(..to_requeue).collect();
        
        // Add to second queue
        let queue2 = queues.entry(uaddr2).or_insert_with(Vec::new);
        queue2.extend(requeued);
    }
    
    // Clean up empty queue
    if queues.get(&uaddr).map(|q| q.is_empty()).unwrap_or(false) {
        queues.remove(&uaddr);
    }
    
    Ok(total_woken)
}

/// Compare and requeue
fn futex_cmp_requeue(uaddr: u64, wake_count: u32, uaddr2: u64, requeue_count: u32, expected: u32) -> Result<i64, i32> {
    // First check the value atomically
    let current = unsafe { 
        let ptr = uaddr as *const AtomicU32;
        (*ptr).load(Ordering::SeqCst)
    };
    if current != expected {
        return Err(-11); // EAGAIN
    }
    
    futex_requeue(uaddr, wake_count, uaddr2, requeue_count)
}

/// Wait with bitset (for pthread_cond)
fn futex_wait_bitset(uaddr: u64, expected: u32, timeout_ns: u64, bitset: u32) -> Result<i64, i32> {
    if bitset == 0 {
        return Err(-22); // EINVAL
    }
    
    // For now, ignore bitset and use regular wait
    // Full implementation would filter wakeups by bitset
    futex_wait(uaddr, expected, timeout_ns)
}

/// Wake with bitset
fn futex_wake_bitset(uaddr: u64, count: u32, bitset: u32) -> Result<i64, i32> {
    if bitset == 0 {
        return Err(-22); // EINVAL
    }
    
    // For now, ignore bitset
    futex_wake(uaddr, count)
}

/// Get number of waiters (for debugging)
pub fn get_waiter_count(uaddr: u64) -> usize {
    FUTEX_QUEUES.lock()
        .get(&uaddr)
        .map(|q| q.len())
        .unwrap_or(0)
}

/// Clean up all futexes for a process
pub fn cleanup_process(pid: u64) {
    let mut queues = FUTEX_QUEUES.lock();
    
    // Remove all entries belonging to this process
    for queue in queues.values_mut() {
        queue.retain(|e| (e.tid >> 32) != pid);
    }
    
    // Remove empty queues
    queues.retain(|_, q| !q.is_empty());
}
