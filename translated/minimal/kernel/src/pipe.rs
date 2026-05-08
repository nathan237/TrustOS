




use alloc::collections::{BTreeMap, VecDeque};
use spin::RwLock;

const AIG_: usize = 4096;
const CNB_: i32 = 64; 


struct Acp {
    data: VecDeque<u8>,
    read_open: bool,
    write_open: bool,
}


struct PipeRegistry {
    pipes: BTreeMap<usize, Acp>,
    fd_map: BTreeMap<i32, (usize, bool)>, 
    next_id: usize,
    next_fd: i32,
}

impl PipeRegistry {
    const fn new() -> Self {
        Self {
            pipes: BTreeMap::new(),
            fd_map: BTreeMap::new(),
            next_id: 1,
            next_fd: CNB_,
        }
    }
}

static Ca: RwLock<PipeRegistry> = RwLock::new(PipeRegistry::new());


pub fn create() -> (i32, i32) {
    let mut reg = Ca.write();
    let auo = reg.next_id;
    reg.next_id += 1;

    let aot = reg.next_fd;
    reg.next_fd += 1;
    let asu = reg.next_fd;
    reg.next_fd += 1;

    reg.pipes.insert(auo, Acp {
        data: VecDeque::with_capacity(AIG_),
        read_open: true,
        write_open: true,
    });
    reg.fd_map.insert(aot, (auo, false));   
    reg.fd_map.insert(asu, (auo, true));    

    crate::log_debug!("[PIPE] Created pipe {} (read_fd={}, write_fd={})", auo, aot, asu);
    (aot, asu)
}


pub fn dab(fd: i32) -> bool {
    Ca.read().fd_map.contains_key(&fd)
}



pub fn write(fd: i32, data: &[u8]) -> i64 {
    if data.is_empty() { return 0; }
    
    let mut retries = 0u32;
    loop {
        {
            let mut reg = Ca.write();
            let &(auo, is_write) = match reg.fd_map.get(&fd) {
                Some(info) => info,
                None => return -9, 
            };
            if !is_write {
                return -9; 
            }
            let pipe = match reg.pipes.get_mut(&auo) {
                Some(aa) => aa,
                None => return -9,
            };
            if !pipe.read_open {
                return -32; 
            }
            let space = AIG_ - pipe.data.len();
            if space > 0 {
                let ae = data.len().min(space);
                for &b in &data[..ae] {
                    pipe.data.push_back(b);
                }
                return ae as i64;
            }
            
        }
        
        retries += 1;
        if retries > 10_000 {
            return -11; 
        }
        crate::thread::boq();
    }
}



pub fn read(fd: i32, buf: &mut [u8]) -> i64 {
    if buf.is_empty() { return 0; }
    
    let mut retries = 0u32;
    loop {
        {
            let mut reg = Ca.write();
            let &(auo, is_write) = match reg.fd_map.get(&fd) {
                Some(info) => info,
                None => return -9, 
            };
            if is_write {
                return -9; 
            }
            let pipe = match reg.pipes.get_mut(&auo) {
                Some(aa) => aa,
                None => return -9,
            };
            if !pipe.data.is_empty() {
                let ae = buf.len().min(pipe.data.len());
                for i in 0..ae {
                    buf[i] = pipe.data.pop_front().unwrap();
                }
                return ae as i64;
            }
            if !pipe.write_open {
                return 0; 
            }
            
        }
        
        retries += 1;
        if retries > 10_000 {
            return 0; 
        }
        crate::thread::boq();
    }
}


pub fn close(fd: i32) -> i64 {
    let mut reg = Ca.write();
    let (auo, is_write) = match reg.fd_map.remove(&fd) {
        Some(info) => info,
        None => return -9, 
    };
    if let Some(pipe) = reg.pipes.get_mut(&auo) {
        if is_write {
            pipe.write_open = false;
        } else {
            pipe.read_open = false;
        }
        if !pipe.read_open && !pipe.write_open {
            reg.pipes.remove(&auo);
            crate::log_debug!("[PIPE] Destroyed pipe {}", auo);
        }
    }
    0
}


pub fn active_count() -> usize {
    Ca.read().pipes.len()
}



pub fn poll(fd: i32) -> (bool, bool, bool) {
    let reg = Ca.read();
    let (auo, is_write) = match reg.fd_map.get(&fd) {
        Some(&info) => info,
        None => return (false, false, false),
    };
    let pipe = match reg.pipes.get(&auo) {
        Some(aa) => aa,
        None => return (false, false, false),
    };
    if is_write {
        
        let gab = pipe.data.len() < AIG_;
        (false, gab, !pipe.read_open)
    } else {
        
        let has_data = !pipe.data.is_empty();
        let ewo = !pipe.write_open;
        (has_data || ewo, false, ewo)
    }
}
