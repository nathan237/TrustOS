//! POSIX Pipes — kernel ring buffers for IPC
//!
//! Provides unidirectional byte streams between file descriptors.
//! Used by Ring 3 processes via pipe2() syscall.

use alloc::collections::{BTreeMap, VecDeque};
use spin::RwLock;

const PIPE_BUF_SIZE: usize = 4096;
const PIPE_FD_BASE: i32 = 64; // Pipe fds start here to avoid VFS collision

/// Internal pipe buffer
struct PipeBuffer {
    data: VecDeque<u8>,
    read_open: bool,
    write_open: bool,
}

/// Global pipe registry — maps fd numbers to pipe buffers
struct PipeRegistry {
    pipes: BTreeMap<usize, PipeBuffer>,
    fd_map: BTreeMap<i32, (usize, bool)>, // fd → (pipe_id, is_write_end)
    next_id: usize,
    next_fd: i32,
}

impl PipeRegistry {
    const fn new() -> Self {
        Self {
            pipes: BTreeMap::new(),
            fd_map: BTreeMap::new(),
            next_id: 1,
            next_fd: PIPE_FD_BASE,
        }
    }
}

static REGISTRY: RwLock<PipeRegistry> = RwLock::new(PipeRegistry::new());

/// Create a new pipe. Returns (read_fd, write_fd).
pub fn create() -> (i32, i32) {
    let mut reg = REGISTRY.write();
    let pipe_id = reg.next_id;
    reg.next_id += 1;

    let read_fd = reg.next_fd;
    reg.next_fd += 1;
    let write_fd = reg.next_fd;
    reg.next_fd += 1;

    reg.pipes.insert(pipe_id, PipeBuffer {
        data: VecDeque::with_capacity(PIPE_BUF_SIZE),
        read_open: true,
        write_open: true,
    });
    reg.fd_map.insert(read_fd, (pipe_id, false));   // read end
    reg.fd_map.insert(write_fd, (pipe_id, true));    // write end

    crate::log_debug!("[PIPE] Created pipe {} (read_fd={}, write_fd={})", pipe_id, read_fd, write_fd);
    (read_fd, write_fd)
}

/// Check if an fd belongs to a pipe
pub fn is_pipe_fd(fd: i32) -> bool {
    REGISTRY.read().fd_map.contains_key(&fd)
}

/// Write bytes into a pipe (via write-end fd). Returns bytes written or negative errno.
/// Blocks (yields) if the pipe buffer is full and the read end is still open.
pub fn write(fd: i32, data: &[u8]) -> i64 {
    if data.is_empty() { return 0; }
    
    let mut retries = 0u32;
    loop {
        {
            let mut reg = REGISTRY.write();
            let &(pipe_id, is_write) = match reg.fd_map.get(&fd) {
                Some(info) => info,
                None => return -9, // EBADF
            };
            if !is_write {
                return -9; // Can't write to read end
            }
            let pipe = match reg.pipes.get_mut(&pipe_id) {
                Some(p) => p,
                None => return -9,
            };
            if !pipe.read_open {
                return -32; // EPIPE — no readers left
            }
            let space = PIPE_BUF_SIZE - pipe.data.len();
            if space > 0 {
                let n = data.len().min(space);
                for &b in &data[..n] {
                    pipe.data.push_back(b);
                }
                return n as i64;
            }
            // Buffer full — drop lock and yield
        }
        
        retries += 1;
        if retries > 10_000 {
            return -11; // EAGAIN — too many retries
        }
        crate::thread::schedule();
    }
}

/// Read bytes from a pipe (via read-end fd). Returns bytes read or negative errno.
/// Blocks (yields) if the pipe is empty and the write end is still open.
pub fn read(fd: i32, buf: &mut [u8]) -> i64 {
    if buf.is_empty() { return 0; }
    
    let mut retries = 0u32;
    loop {
        {
            let mut reg = REGISTRY.write();
            let &(pipe_id, is_write) = match reg.fd_map.get(&fd) {
                Some(info) => info,
                None => return -9, // EBADF
            };
            if is_write {
                return -9; // Can't read from write end
            }
            let pipe = match reg.pipes.get_mut(&pipe_id) {
                Some(p) => p,
                None => return -9,
            };
            if !pipe.data.is_empty() {
                let n = buf.len().min(pipe.data.len());
                for i in 0..n {
                    buf[i] = pipe.data.pop_front().unwrap();
                }
                return n as i64;
            }
            if !pipe.write_open {
                return 0; // EOF — write end closed, no more data coming
            }
            // Empty but write end open — drop lock and yield
        }
        
        retries += 1;
        if retries > 10_000 {
            return 0; // Treat as EOF after too many retries
        }
        crate::thread::schedule();
    }
}

/// Close a pipe fd (either end). Destroys pipe when both ends are closed.
pub fn close(fd: i32) -> i64 {
    let mut reg = REGISTRY.write();
    let (pipe_id, is_write) = match reg.fd_map.remove(&fd) {
        Some(info) => info,
        None => return -9, // EBADF
    };
    if let Some(pipe) = reg.pipes.get_mut(&pipe_id) {
        if is_write {
            pipe.write_open = false;
        } else {
            pipe.read_open = false;
        }
        if !pipe.read_open && !pipe.write_open {
            reg.pipes.remove(&pipe_id);
            crate::log_debug!("[PIPE] Destroyed pipe {}", pipe_id);
        }
    }
    0
}

/// Number of active pipes
pub fn active_count() -> usize {
    REGISTRY.read().pipes.len()
}
