//! PTY (Pseudo-Terminal) Subsystem for TrustOS
//!
//! Implements POSIX-style pseudo-terminals. Opening /dev/ptmx allocates a
//! master/slave pair.  The master side is returned to the opener; the slave
//! side is accessible via /dev/pts/N and behaves as a regular TTY.
//!
//! This enables programs like `screen`, `tmux`, ssh, and terminal emulators
//! to operate on virtual terminals.

use alloc::vec::Vec;
use spin::Mutex;
use core::sync::atomic::{AtomicU32, Ordering};

/// Maximum number of PTY pairs
const MAX_PTYS: usize = 64;

/// A PTY master/slave pair
pub struct PtyPair {
    /// PTY index (the N in /dev/pts/N)
    pub index: u32,
    /// Underlying TTY device index in the tty subsystem
    pub tty_index: u32,
    /// Whether the master side is open
    pub master_open: bool,
    /// Whether the slave side is open
    pub slave_open: bool,
    /// Master-side read buffer (data written by slave → read by master)
    pub master_buf: Vec<u8>,
    /// Slave-side read buffer (data written by master → read by slave)
    pub slave_buf: Vec<u8>,
}

/// Global PTY table
static PTY_TABLE: Mutex<Option<Vec<PtyPair>>> = Mutex::new(None);

/// Next PTY index
static NEXT_PTY: AtomicU32 = AtomicU32::new(0);

/// Initialize PTY subsystem
pub fn init() {
    let mut table = PTY_TABLE.lock();
    *table = Some(Vec::with_capacity(MAX_PTYS));
    crate::log!("[PTY] Pseudo-terminal subsystem initialized");
}

/// Allocate a new PTY pair (/dev/ptmx). Returns (pty_index, tty_index).
pub fn alloc_pty() -> Option<(u32, u32)> {
    let pty_idx = NEXT_PTY.fetch_add(1, Ordering::SeqCst);
    if pty_idx as usize >= MAX_PTYS {
        NEXT_PTY.fetch_sub(1, Ordering::SeqCst);
        return None;
    }
    
    // Allocate a TTY for the slave side
    let tty_idx = crate::tty::alloc_tty()?;
    
    let pair = PtyPair {
        index: pty_idx,
        tty_index: tty_idx,
        master_open: true,
        slave_open: false,
        master_buf: Vec::new(),
        slave_buf: Vec::new(),
    };
    
    let mut table = PTY_TABLE.lock();
    if let Some(ref mut ptys) = *table {
        ptys.push(pair);
        crate::log_debug!("[PTY] Allocated pty{} (tty{})", pty_idx, tty_idx);
        Some((pty_idx, tty_idx))
    } else {
        None
    }
}

/// Open the slave side of a PTY
pub fn open_slave(pty_index: u32) -> bool {
    let mut table = PTY_TABLE.lock();
    if let Some(ref mut ptys) = *table {
        for pty in ptys.iter_mut() {
            if pty.index == pty_index {
                pty.slave_open = true;
                return true;
            }
        }
    }
    false
}

/// Write to the master side (data goes to slave's input)
pub fn master_write(pty_index: u32, data: &[u8]) -> usize {
    let mut table = PTY_TABLE.lock();
    if let Some(ref mut ptys) = *table {
        for pty in ptys.iter_mut() {
            if pty.index == pty_index && pty.master_open {
                // Data written to master appears as input on slave TTY
                pty.slave_buf.extend_from_slice(data);
                return data.len();
            }
        }
    }
    0
}

/// Read from the master side (data that slave wrote)
pub fn master_read(pty_index: u32, buf: &mut [u8]) -> usize {
    let mut table = PTY_TABLE.lock();
    if let Some(ref mut ptys) = *table {
        for pty in ptys.iter_mut() {
            if pty.index == pty_index && pty.master_open {
                let count = buf.len().min(pty.master_buf.len());
                for i in 0..count {
                    buf[i] = pty.master_buf.remove(0);
                }
                return count;
            }
        }
    }
    0
}

/// Write to the slave side (data goes to master's read buffer)
pub fn slave_write(pty_index: u32, data: &[u8]) -> usize {
    let mut table = PTY_TABLE.lock();
    if let Some(ref mut ptys) = *table {
        for pty in ptys.iter_mut() {
            if pty.index == pty_index && pty.slave_open {
                pty.master_buf.extend_from_slice(data);
                return data.len();
            }
        }
    }
    0
}

/// Read from the slave side (data that master wrote)
pub fn slave_read(pty_index: u32, buf: &mut [u8]) -> usize {
    let mut table = PTY_TABLE.lock();
    if let Some(ref mut ptys) = *table {
        for pty in ptys.iter_mut() {
            if pty.index == pty_index && pty.slave_open {
                let count = buf.len().min(pty.slave_buf.len());
                for i in 0..count {
                    buf[i] = pty.slave_buf.remove(0);
                }
                return count;
            }
        }
    }
    0
}

/// Close the master side
pub fn close_master(pty_index: u32) {
    let mut table = PTY_TABLE.lock();
    if let Some(ref mut ptys) = *table {
        for pty in ptys.iter_mut() {
            if pty.index == pty_index {
                pty.master_open = false;
                break;
            }
        }
    }
}

/// Close the slave side
pub fn close_slave(pty_index: u32) {
    let mut table = PTY_TABLE.lock();
    if let Some(ref mut ptys) = *table {
        for pty in ptys.iter_mut() {
            if pty.index == pty_index {
                pty.slave_open = false;
                break;
            }
        }
    }
}

/// Get the TTY index for a PTY's slave side
pub fn get_slave_tty(pty_index: u32) -> Option<u32> {
    let table = PTY_TABLE.lock();
    if let Some(ref ptys) = *table {
        for pty in ptys.iter() {
            if pty.index == pty_index {
                return Some(pty.tty_index);
            }
        }
    }
    None
}

/// Get PTS name for a PTY index
pub fn pts_name(pty_index: u32) -> alloc::string::String {
    alloc::format!("/dev/pts/{}", pty_index)
}
