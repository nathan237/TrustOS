//! TTY Subsystem for TrustOS
//!
//! Provides a POSIX-like TTY layer with line discipline, foreground process
//! groups, session leaders, and terminal window size.

use alloc::collections::VecDeque;
use alloc::string::String;
use alloc::vec::Vec;
use spin::Mutex;
use core::sync::atomic::{AtomicU32, Ordering};

/// Maximum number of TTY devices
const MAX_TTYS: usize = 16;

/// Terminal window size (TIOCGWINSZ / TIOCSWINSZ)
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct WinSize {
    pub ws_row: u16,
    pub ws_col: u16,
    pub ws_xpixel: u16,
    pub ws_ypixel: u16,
}

impl Default for WinSize {
    fn default() -> Self {
        Self { ws_row: 25, ws_col: 80, ws_xpixel: 0, ws_ypixel: 0 }
    }
}

/// Termios-like line discipline flags (simplified)
#[derive(Clone, Debug)]
pub struct Termios {
    /// Input flags
    pub iflag: u32,
    /// Output flags
    pub oflag: u32,
    /// Control flags
    pub cflag: u32,
    /// Local flags
    pub lflag: u32,
}

// Local flag bits
pub const ECHO: u32   = 0x0008;
pub const ICANON: u32 = 0x0002;
pub const ISIG: u32   = 0x0001;

impl Default for Termios {
    fn default() -> Self {
        Self {
            iflag: 0,
            oflag: 0,
            cflag: 0,
            lflag: ECHO | ICANON | ISIG, // canonical mode with echo + signals
        }
    }
}

/// A TTY device
pub struct TtyDevice {
    pub index: u32,
    /// Session ID that owns this TTY
    pub session_id: u32,
    /// Foreground process group ID
    pub foreground_pgid: u32,
    /// Line discipline settings
    pub termios: Termios,
    /// Input buffer (line-edited in canonical mode)
    pub input_buf: VecDeque<u8>,
    /// Canonical line buffer (current line being edited)
    pub line_buf: Vec<u8>,
    /// Output buffer
    pub output_buf: VecDeque<u8>,
    /// Window size
    pub winsize: WinSize,
    /// Whether the TTY is open
    pub active: bool,
}

impl TtyDevice {
    pub fn new(index: u32) -> Self {
        Self {
            index,
            session_id: 0,
            foreground_pgid: 0,
            termios: Termios::default(),
            input_buf: VecDeque::new(),
            line_buf: Vec::new(),
            output_buf: VecDeque::new(),
            winsize: WinSize::default(),
            active: false,
        }
    }

    /// Process input character through line discipline
    pub fn input_char(&mut self, ch: u8) {
        let canonical = (self.termios.lflag & ICANON) != 0;
        let echo = (self.termios.lflag & ECHO) != 0;
        let signals = (self.termios.lflag & ISIG) != 0;

        // Signal characters (Ctrl-C, Ctrl-Z, Ctrl-\)
        if signals {
            match ch {
                3 => {
                    // Ctrl-C → SIGINT to foreground process group
                    if self.foreground_pgid > 0 {
                        let _ = crate::signals::kill_process_group(self.foreground_pgid, 2); // SIGINT
                    }
                    return;
                }
                26 => {
                    // Ctrl-Z → SIGTSTP to foreground process group
                    if self.foreground_pgid > 0 {
                        let _ = crate::signals::kill_process_group(self.foreground_pgid, 20); // SIGTSTP
                    }
                    return;
                }
                28 => {
                    // Ctrl-\ → SIGQUIT to foreground process group
                    if self.foreground_pgid > 0 {
                        let _ = crate::signals::kill_process_group(self.foreground_pgid, 3); // SIGQUIT
                    }
                    return;
                }
                _ => {}
            }
        }

        if canonical {
            // Canonical mode: line editing
            match ch {
                b'\n' | b'\r' => {
                    self.line_buf.push(b'\n');
                    // Transfer completed line to input queue
                    for &b in &self.line_buf {
                        self.input_buf.push_back(b);
                    }
                    self.line_buf.clear();
                    if echo {
                        self.output_buf.push_back(b'\n');
                    }
                }
                0x7F | 8 => {
                    // Backspace / DEL
                    if !self.line_buf.is_empty() {
                        self.line_buf.pop();
                        if echo {
                            self.output_buf.push_back(8);
                            self.output_buf.push_back(b' ');
                            self.output_buf.push_back(8);
                        }
                    }
                }
                _ => {
                    self.line_buf.push(ch);
                    if echo {
                        self.output_buf.push_back(ch);
                    }
                }
            }
        } else {
            // Raw mode: characters go directly to input
            self.input_buf.push_back(ch);
            if echo {
                self.output_buf.push_back(ch);
            }
        }
    }

    /// Read from TTY input buffer
    pub fn read(&mut self, buf: &mut [u8]) -> usize {
        let count = buf.len().min(self.input_buf.len());
        for i in 0..count {
            buf[i] = self.input_buf.pop_front().unwrap_or(0);
        }
        count
    }

    /// Write to TTY output buffer
    pub fn write(&mut self, data: &[u8]) -> usize {
        for &b in data {
            self.output_buf.push_back(b);
        }
        // Flush to serial immediately
        for &b in data {
            crate::serial_print!("{}", b as char);
        }
        data.len()
    }

    /// Drain any pending output
    pub fn flush_output(&mut self) -> Vec<u8> {
        self.output_buf.drain(..).collect()
    }

    /// Check if this TTY is the controlling terminal of a given session
    pub fn is_controlling_for(&self, sid: u32) -> bool {
        self.active && self.session_id == sid
    }
}

/// Global TTY table
static TTY_TABLE: Mutex<Option<Vec<TtyDevice>>> = Mutex::new(None);

/// Next TTY index allocator
static NEXT_TTY: AtomicU32 = AtomicU32::new(0);

/// Initialize the TTY subsystem
pub fn init() {
    let mut table = TTY_TABLE.lock();
    let mut devices = Vec::with_capacity(MAX_TTYS);
    
    // Create TTY0 as the console TTY
    let mut tty0 = TtyDevice::new(0);
    tty0.active = true;
    tty0.session_id = 1; // init session
    tty0.foreground_pgid = 1;
    devices.push(tty0);
    
    NEXT_TTY.store(1, Ordering::SeqCst);
    *table = Some(devices);
    
    crate::log!("[TTY] TTY subsystem initialized (tty0 = console)");
}

/// Allocate a new TTY device, returns its index
pub fn alloc_tty() -> Option<u32> {
    let idx = NEXT_TTY.fetch_add(1, Ordering::SeqCst);
    if idx as usize >= MAX_TTYS {
        NEXT_TTY.fetch_sub(1, Ordering::SeqCst);
        return None;
    }
    
    let mut table = TTY_TABLE.lock();
    if let Some(ref mut devices) = *table {
        let tty = TtyDevice::new(idx);
        devices.push(tty);
        Some(idx)
    } else {
        None
    }
}

/// Access a TTY by index
pub fn with_tty<F, R>(index: u32, f: F) -> Option<R>
where
    F: FnOnce(&mut TtyDevice) -> R,
{
    let mut table = TTY_TABLE.lock();
    if let Some(ref mut devices) = *table {
        for tty in devices.iter_mut() {
            if tty.index == index {
                return Some(f(tty));
            }
        }
    }
    None
}

/// Set the foreground process group for a TTY
pub fn set_foreground_pgid(tty_index: u32, pgid: u32) {
    with_tty(tty_index, |tty| {
        tty.foreground_pgid = pgid;
    });
}

/// Get the foreground process group for a TTY
pub fn get_foreground_pgid(tty_index: u32) -> u32 {
    with_tty(tty_index, |tty| tty.foreground_pgid).unwrap_or(0)
}

/// Set the controlling TTY for a session
pub fn set_controlling_tty(tty_index: u32, session_id: u32) {
    with_tty(tty_index, |tty| {
        tty.active = true;
        tty.session_id = session_id;
    });
}

/// Get the window size of a TTY
pub fn get_winsize(tty_index: u32) -> WinSize {
    with_tty(tty_index, |tty| tty.winsize).unwrap_or_default()
}

/// Set the window size of a TTY
pub fn set_winsize(tty_index: u32, ws: WinSize) {
    with_tty(tty_index, |tty| {
        tty.winsize = ws;
    });
}

// ============================================================================
// ioctl constants for TTY
// ============================================================================

pub const TIOCGPGRP: u64   = 0x540F;
pub const TIOCSPGRP: u64   = 0x5410;
pub const TIOCGSID: u64    = 0x5429;
pub const TIOCGWINSZ: u64  = 0x5413;
pub const TIOCSWINSZ: u64  = 0x5414;
pub const TIOCSCTTY: u64   = 0x540E;
pub const TIOCNOTTY: u64   = 0x5422;
pub const TCGETS: u64      = 0x5401;
pub const TCSETS: u64      = 0x5402;

/// Handle TTY ioctl requests
pub fn handle_ioctl(tty_index: u32, request: u64, arg: u64) -> i64 {
    match request {
        TIOCGPGRP => {
            // Get foreground process group
            let pgid = get_foreground_pgid(tty_index);
            if arg != 0 && crate::memory::validate_user_ptr(arg, 4, true) {
                unsafe { *(arg as *mut u32) = pgid; }
            }
            0
        }
        TIOCSPGRP => {
            // Set foreground process group
            if arg != 0 && crate::memory::validate_user_ptr(arg, 4, false) {
                let pgid = unsafe { *(arg as *const u32) };
                set_foreground_pgid(tty_index, pgid);
            }
            0
        }
        TIOCGSID => {
            // Get session ID
            let sid = with_tty(tty_index, |tty| tty.session_id).unwrap_or(0);
            if arg != 0 && crate::memory::validate_user_ptr(arg, 4, true) {
                unsafe { *(arg as *mut u32) = sid; }
            }
            0
        }
        TIOCGWINSZ => {
            // Get window size
            let ws = get_winsize(tty_index);
            if arg != 0 && crate::memory::validate_user_ptr(arg, 8, true) {
                unsafe { *(arg as *mut WinSize) = ws; }
            }
            0
        }
        TIOCSWINSZ => {
            // Set window size
            if arg != 0 && crate::memory::validate_user_ptr(arg, 8, false) {
                let ws = unsafe { *(arg as *const WinSize) };
                set_winsize(tty_index, ws);
            }
            0
        }
        TIOCSCTTY => {
            // Set controlling terminal
            let pid = crate::process::current_pid();
            let sid = crate::process::get_sid(pid);
            set_controlling_tty(tty_index, sid);
            0
        }
        TCGETS => {
            // Get termios — write 16 bytes (4 u32 flags)
            if arg != 0 && crate::memory::validate_user_ptr(arg, 16, true) {
                if let Some(termios) = with_tty(tty_index, |tty| tty.termios.clone()) {
                    unsafe {
                        let p = arg as *mut u32;
                        *p = termios.iflag;
                        *p.add(1) = termios.oflag;
                        *p.add(2) = termios.cflag;
                        *p.add(3) = termios.lflag;
                    }
                }
            }
            0
        }
        TCSETS => {
            // Set termios — read 16 bytes
            if arg != 0 && crate::memory::validate_user_ptr(arg, 16, false) {
                let (iflag, oflag, cflag, lflag) = unsafe {
                    let p = arg as *const u32;
                    (*p, *p.add(1), *p.add(2), *p.add(3))
                };
                with_tty(tty_index, |tty| {
                    tty.termios.iflag = iflag;
                    tty.termios.oflag = oflag;
                    tty.termios.cflag = cflag;
                    tty.termios.lflag = lflag;
                });
            }
            0
        }
        _ => crate::syscall::errno::ENOTTY,
    }
}
