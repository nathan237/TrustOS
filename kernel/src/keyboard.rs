//! Keyboard driver
//! 
//! Converts PS/2 scancodes to ASCII characters and manages input buffer.
//! Supports special keys (arrows) and command history.

use spin::Mutex;
use alloc::string::String;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicBool, Ordering};
use crate::serial;

/// Keyboard input buffer size
const BUFFER_SIZE: usize = 256;

/// Special key codes (virtual, not actual scancodes)
pub const KEY_UP: u8 = 0xF0;
pub const KEY_DOWN: u8 = 0xF1;
pub const KEY_LEFT: u8 = 0xF2;
pub const KEY_RIGHT: u8 = 0xF3;
pub const KEY_HOME: u8 = 0xF4;
pub const KEY_END: u8 = 0xF5;
pub const KEY_DELETE: u8 = 0xF6;
pub const KEY_PGUP: u8 = 0xF7;
pub const KEY_PGDOWN: u8 = 0xF8;

/// Ring buffer for keyboard input
struct KeyboardBuffer {
    buffer: [u8; BUFFER_SIZE],
    read_pos: usize,
    write_pos: usize,
}

impl KeyboardBuffer {
    const fn new() -> Self {
        Self {
            buffer: [0; BUFFER_SIZE],
            read_pos: 0,
            write_pos: 0,
        }
    }

    fn push(&mut self, byte: u8) {
        let next_write = (self.write_pos + 1) % BUFFER_SIZE;
        if next_write != self.read_pos {
            self.buffer[self.write_pos] = byte;
            self.write_pos = next_write;
        }
    }

    fn pop(&mut self) -> Option<u8> {
        if self.read_pos == self.write_pos {
            None
        } else {
            let byte = self.buffer[self.read_pos];
            self.read_pos = (self.read_pos + 1) % BUFFER_SIZE;
            Some(byte)
        }
    }

    fn is_empty(&self) -> bool {
        self.read_pos == self.write_pos
    }
}

/// Command history
const HISTORY_SIZE: usize = 32;

struct CommandHistory {
    entries: [Option<String>; HISTORY_SIZE],
    write_pos: usize,
    browse_pos: usize,
    count: usize,
}

impl CommandHistory {
    const fn new() -> Self {
        // Can't use array initialization with Option<String> in const
        Self {
            entries: [
                None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None,
            ],
            write_pos: 0,
            browse_pos: 0,
            count: 0,
        }
    }
    
    fn add(&mut self, cmd: &str) {
        if cmd.is_empty() {
            return;
        }
        // Don't add duplicates of last command
        if self.count > 0 {
            let last_pos = if self.write_pos == 0 { HISTORY_SIZE - 1 } else { self.write_pos - 1 };
            if let Some(ref last) = self.entries[last_pos] {
                if last == cmd {
                    self.browse_pos = self.write_pos;
                    return;
                }
            }
        }
        
        self.entries[self.write_pos] = Some(String::from(cmd));
        self.write_pos = (self.write_pos + 1) % HISTORY_SIZE;
        if self.count < HISTORY_SIZE {
            self.count += 1;
        }
        self.browse_pos = self.write_pos;
    }
    
    fn get_prev(&mut self) -> Option<&str> {
        if self.count == 0 {
            return None;
        }
        
        let new_pos = if self.browse_pos == 0 { 
            HISTORY_SIZE - 1 
        } else { 
            self.browse_pos - 1 
        };
        
        // Don't go past oldest entry
        let oldest = if self.count < HISTORY_SIZE {
            0
        } else {
            self.write_pos
        };
        
        if new_pos == oldest && self.browse_pos == oldest {
            // Already at oldest
            return self.entries[self.browse_pos].as_deref();
        }
        
        if self.entries[new_pos].is_some() {
            self.browse_pos = new_pos;
            self.entries[self.browse_pos].as_deref()
        } else {
            None
        }
    }
    
    fn get_next(&mut self) -> Option<&str> {
        if self.browse_pos == self.write_pos {
            return None; // Already at newest
        }
        
        self.browse_pos = (self.browse_pos + 1) % HISTORY_SIZE;
        
        if self.browse_pos == self.write_pos {
            None // Reached end (current input)
        } else {
            self.entries[self.browse_pos].as_deref()
        }
    }
    
    fn reset_browse(&mut self) {
        self.browse_pos = self.write_pos;
    }
    
    fn iter(&self) -> impl Iterator<Item = (usize, &str)> {
        let count = self.count;
        let start = if count < HISTORY_SIZE { 0 } else { self.write_pos };
        
        (0..count).map(move |i| {
            let idx = (start + i) % HISTORY_SIZE;
            (i + 1, self.entries[idx].as_deref().unwrap_or(""))
        })
    }
}

/// Global keyboard buffer
static KEYBOARD_BUFFER: Mutex<KeyboardBuffer> = Mutex::new(KeyboardBuffer::new());

/// Global command history
static COMMAND_HISTORY: Mutex<CommandHistory> = Mutex::new(CommandHistory::new());
/// Internal clipboard for Ctrl+C/Ctrl+V
static CLIPBOARD: Mutex<Option<String>> = Mutex::new(None);

/// Extended scancode flag (0xE0 prefix)
static EXTENDED_KEY: AtomicBool = AtomicBool::new(false);

/// Alt key state for hotkeys
static ALT_PRESSED: AtomicBool = AtomicBool::new(false);

/// Shift key state
static SHIFT_PRESSED: AtomicBool = AtomicBool::new(false);
/// Caps lock state
static CAPS_LOCK: AtomicBool = AtomicBool::new(false);
/// Num lock state (default OFF - numpad keys are navigation)
static NUM_LOCK: AtomicBool = AtomicBool::new(false);
/// Ctrl key state  
static CTRL_PRESSED: AtomicBool = AtomicBool::new(false);

/// Key state bitmap (256 bits = 32 bytes, one bit per scancode)
static KEY_STATE: Mutex<[u8; 32]> = Mutex::new([0u8; 32]);

/// Last scancode for debouncing
static LAST_SCANCODE: core::sync::atomic::AtomicU8 = core::sync::atomic::AtomicU8::new(0xFF);
/// Scancode repeat counter for debouncing
static REPEAT_COUNT: core::sync::atomic::AtomicU8 = core::sync::atomic::AtomicU8::new(0);

/// US keyboard scancode set 1 to ASCII (lowercase)
const SCANCODE_TO_ASCII: [u8; 128] = [
    0, 27, // 0x00, 0x01 - ESC
    b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'0', b'-', b'=', 
    0x08, // 0x0E Backspace (ASCII 8)
    b'\t', // 0x0F Tab
    b'q', b'w', b'e', b'r', b't', b'y', b'u', b'i', b'o', b'p', b'[', b']', 
    b'\n', // 0x1C Enter
    0, // 0x1D Ctrl
    b'a', b's', b'd', b'f', b'g', b'h', b'j', b'k', b'l', b';', b'\'', b'`', // 0x1E-0x29
    0, // 0x2A Left Shift
    b'\\', b'z', b'x', b'c', b'v', b'b', b'n', b'm', b',', b'.', b'/', // 0x2B-0x35
    0, // 0x36 Right Shift
    b'*', // 0x37 Keypad *
    0, // 0x38 Alt
    b' ', // 0x39 Space (ASCII 32)
    0, // 0x3A Caps Lock
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // 0x3B-0x44 F1-F10
    0, // 0x45 Num Lock
    0, // 0x46 Scroll Lock
    b'7', b'8', b'9', b'-', // 0x47-0x4A Keypad
    b'4', b'5', b'6', b'+', // 0x4B-0x4E Keypad
    b'1', b'2', b'3', // 0x4F-0x51 Keypad
    b'0', b'.', // 0x52-0x53 Keypad
    0, 0, 0, // 0x54-0x56 Unused
    0, 0, // 0x57-0x58 F11, F12
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // 0x59-0x68
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // 0x69-0x78
    0, 0, 0, 0, 0, 0, 0, // 0x79-0x7F
];

/// US keyboard scancode set 1 to ASCII (uppercase/shifted)
const SCANCODE_TO_ASCII_SHIFT: [u8; 128] = [
    0, 27, // 0x00, 0x01 - ESC
    b'!', b'@', b'#', b'$', b'%', b'^', b'&', b'*', b'(', b')', b'_', b'+', 
    0x08, // 0x0E Backspace (ASCII 8)
    b'\t', // 0x0F Tab
    b'Q', b'W', b'E', b'R', b'T', b'Y', b'U', b'I', b'O', b'P', b'{', b'}', 
    b'\n', // 0x1C Enter
    0, // 0x1D Ctrl
    b'A', b'S', b'D', b'F', b'G', b'H', b'J', b'K', b'L', b':', b'"', b'~', // 0x1E-0x29
    0, // 0x2A Left Shift
    b'|', b'Z', b'X', b'C', b'V', b'B', b'N', b'M', b'<', b'>', b'?', // 0x2B-0x35
    0, // 0x36 Right Shift
    b'*', // 0x37 Keypad *
    0, // 0x38 Alt
    b' ', // 0x39 Space (ASCII 32)
    0, // 0x3A Caps Lock
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // 0x3B-0x44 F1-F10
    0, // 0x45 Num Lock
    0, // 0x46 Scroll Lock
    b'7', b'8', b'9', b'-', // 0x47-0x4A Keypad
    b'4', b'5', b'6', b'+', // 0x4B-0x4E Keypad
    b'1', b'2', b'3', // 0x4F-0x51 Keypad
    b'0', b'.', // 0x52-0x53 Keypad
    0, 0, 0, // 0x54-0x56 Unused
    0, 0, // 0x57-0x58 F11, F12
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // 0x59-0x68
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // 0x69-0x78
    0, 0, 0, 0, 0, 0, 0, // 0x79-0x7F
];

/// Process a scancode from the keyboard interrupt
pub fn handle_scancode(scancode: u8) {
    // Debug: log all scancodes
    crate::serial_println!("[KB-IRQ] scancode=0x{:02X}", scancode);
    
    // Ignore invalid/spurious scancodes and PS/2 controller responses
    // 0xFA = ACK, 0xFE = Resend, 0xFC = Error, 0xEE = Echo
    // NOTE: 0xAA (BAT OK) is NOT filtered because it's also Left Shift release
    // (scancode 0x2A | 0x80 = 0xAA). Filtering it caused Shift to get "stuck".
    if scancode == 0x00 || scancode == 0xFF || scancode == 0xFA 
        || scancode == 0xFE || scancode == 0xFC
        || scancode == 0xEE {
        return;
    }
    
    // Ignore numpad keys (0x47-0x53) when not extended and NumLock is off
    // This prevents spurious "7" appearing at boot in VirtualBox
    // These scancodes are: 7,8,9,-,4,5,6,+,1,2,3,0,.
    let key_without_release = scancode & 0x7F;
    if key_without_release >= 0x47 && key_without_release <= 0x53 {
        if !EXTENDED_KEY.load(Ordering::SeqCst) && !NUM_LOCK.load(Ordering::SeqCst) {
            // NumLock off: treat numpad keys as navigation, not numbers
            // 0x47=Home, 0x48=Up, 0x49=PgUp, 0x4B=Left, 0x4D=Right,
            // 0x4F=End, 0x50=Down, 0x51=PgDn, 0x52=Ins, 0x53=Del
            let is_release = scancode & 0x80 != 0;
            if !is_release {
                let special = match key_without_release {
                    0x47 => Some(KEY_HOME),
                    0x48 => Some(KEY_UP),
                    0x49 => Some(KEY_PGUP),
                    0x4B => Some(KEY_LEFT),
                    0x4D => Some(KEY_RIGHT),
                    0x4F => Some(KEY_END),
                    0x50 => Some(KEY_DOWN),
                    0x51 => Some(KEY_PGDOWN),
                    0x53 => Some(KEY_DELETE),
                    _ => None,
                };
                if let Some(k) = special {
                    KEYBOARD_BUFFER.lock().push(k);
                }
            }
            return;
        }
    }
    
    // Handle extended scancode prefix
    if scancode == 0xE0 {
        EXTENDED_KEY.store(true, Ordering::SeqCst);
        return;
    }
    
    let is_extended = EXTENDED_KEY.load(Ordering::SeqCst);
    EXTENDED_KEY.store(false, Ordering::SeqCst);
    
    // Check for key release (high bit set)
    let is_release = scancode & 0x80 != 0;
    let key = scancode & 0x7F;
    
    // Update key state tracking for Alt+Tab and hotkeys
    update_key_state(key, !is_release);
    
    // Handle extended keys (arrows, etc.)
    if is_extended && !is_release {
        let special = match key {
            0x48 => Some(KEY_UP),
            0x50 => Some(KEY_DOWN),
            0x4B => Some(KEY_LEFT),
            0x4D => Some(KEY_RIGHT),
            0x47 => Some(KEY_HOME),
            0x4F => Some(KEY_END),
            0x53 => Some(KEY_DELETE),
            0x71 => Some(KEY_DELETE), // Set 2 Delete
            0x75 => Some(KEY_UP),      // Set 2 Up
            0x72 => Some(KEY_DOWN),    // Set 2 Down
            0x6B => Some(KEY_LEFT),    // Set 2 Left
            0x74 => Some(KEY_RIGHT),   // Set 2 Right
            0x6C => Some(KEY_HOME),    // Set 2 Home
            0x69 => Some(KEY_END),     // Set 2 End
            0x49 => Some(KEY_PGUP),
            0x51 => Some(KEY_PGDOWN),
            _ => None,
        };
        if let Some(k) = special {
            KEYBOARD_BUFFER.lock().push(k);
        }
        return;
    }

    // Fallback for space on set 2 keyboards
    if !is_extended && !is_release && key == 0x29 {
        KEYBOARD_BUFFER.lock().push(b' ');
        return;
    }
    
    // Handle Ctrl key
    if key == 0x1D {
        CTRL_PRESSED.store(!is_release, Ordering::SeqCst);
        return;
    }
    
    // Handle shift keys
    if key == 0x2A || key == 0x36 {
        // Left or Right Shift
        SHIFT_PRESSED.store(!is_release, Ordering::SeqCst);
        return;
    }
    
    // Handle Caps Lock toggle
    if key == 0x3A && !is_release {
        let current = CAPS_LOCK.load(Ordering::SeqCst);
        CAPS_LOCK.store(!current, Ordering::SeqCst);
        return;
    }
    
    // Handle Num Lock toggle (scancode 0x45)
    if key == 0x45 && !is_release {
        let current = NUM_LOCK.load(Ordering::SeqCst);
        NUM_LOCK.store(!current, Ordering::SeqCst);
        return;
    }
    
    // Only process key presses, not releases
    if is_release {
        return;
    }
    
    // Handle Ctrl+A (select all)
    if CTRL_PRESSED.load(Ordering::SeqCst) && key == 0x1E {
        KEYBOARD_BUFFER.lock().push(1); // ASCII SOH
        return;
    }

    // Handle Ctrl+C
    if CTRL_PRESSED.load(Ordering::SeqCst) && key == 0x2E {
        // Ctrl+C - could be used for interrupt
        KEYBOARD_BUFFER.lock().push(3); // ASCII ETX
        return;
    }

    // Handle Ctrl+V (paste)
    if CTRL_PRESSED.load(Ordering::SeqCst) && key == 0x2F {
        KEYBOARD_BUFFER.lock().push(0x16); // ASCII SYN
        return;
    }

    // Handle Ctrl+L (clear)
    if CTRL_PRESSED.load(Ordering::SeqCst) && key == 0x26 {
        KEYBOARD_BUFFER.lock().push(12); // ASCII FF (form feed)
        return;
    }

    // Handle Ctrl+S (save)
    if CTRL_PRESSED.load(Ordering::SeqCst) && key == 0x1F {
        KEYBOARD_BUFFER.lock().push(0x13); // ASCII DC3 (save)
        return;
    }

    // Handle Ctrl+G (goto line)
    if CTRL_PRESSED.load(Ordering::SeqCst) && key == 0x22 {
        KEYBOARD_BUFFER.lock().push(0x07); // ASCII BEL (goto)
        return;
    }

    // Handle Ctrl+Z (undo)
    if CTRL_PRESSED.load(Ordering::SeqCst) && key == 0x2C {
        KEYBOARD_BUFFER.lock().push(0x1A); // ASCII SUB (undo)
        return;
    }
    
    // Convert scancode to ASCII
    let shift = SHIFT_PRESSED.load(Ordering::SeqCst);
    let caps = CAPS_LOCK.load(Ordering::SeqCst);
    
    let ascii = if key < 128 {
        let base = if shift {
            SCANCODE_TO_ASCII_SHIFT[key as usize]
        } else {
            SCANCODE_TO_ASCII[key as usize]
        };
        
        // Apply caps lock only to letters
        if caps && base >= b'a' && base <= b'z' {
            base - 32 // To uppercase
        } else if caps && base >= b'A' && base <= b'Z' {
            base + 32 // To lowercase (caps inverts shift)
        } else {
            base
        }
    } else {
        0
    };
    
    // If valid ASCII, add to buffer
    if ascii != 0 {
        crate::serial_println!("[KB-BUF] push ascii={} (0x{:02X}) char='{}'", ascii, ascii, ascii as char);
        KEYBOARD_BUFFER.lock().push(ascii);
    }
}

/// Read a character from the keyboard buffer (non-blocking)
pub fn read_char() -> Option<u8> {
    if let Some(b) = KEYBOARD_BUFFER.lock().pop() {
        return Some(b);
    }
    serial::read_byte()
}

/// Push a key from USB HID keyboard (or any external source)
pub fn push_key(ascii: u8) {
    if ascii != 0 {
        KEYBOARD_BUFFER.lock().push(ascii);
    }
}

/// Check if there's input available
pub fn has_input() -> bool {
    !KEYBOARD_BUFFER.lock().is_empty()
}

/// Check if a specific key (by scancode) is currently pressed
/// Used for checking modifier keys like Alt during Alt+Tab
pub fn is_key_pressed(scancode: u8) -> bool {
    // Special handling for modifier keys
    match scancode {
        0x38 => ALT_PRESSED.load(Ordering::Relaxed),   // Alt
        0x1D => CTRL_PRESSED.load(Ordering::Relaxed),  // Ctrl
        0x2A | 0x36 => SHIFT_PRESSED.load(Ordering::Relaxed), // Shift
        _ => {
            // Check key state bitmap
            let state = KEY_STATE.lock();
            let byte_idx = (scancode / 8) as usize;
            let bit_idx = scancode % 8;
            if byte_idx < 32 {
                (state[byte_idx] & (1 << bit_idx)) != 0
            } else {
                false
            }
        }
    }
}

/// Update key state when a key is pressed or released
fn update_key_state(scancode: u8, pressed: bool) {
    // Update modifier key atomics
    match scancode {
        0x38 => ALT_PRESSED.store(pressed, Ordering::Relaxed),
        0x1D => CTRL_PRESSED.store(pressed, Ordering::Relaxed),
        0x2A | 0x36 => SHIFT_PRESSED.store(pressed, Ordering::Relaxed),
        _ => {}
    }
    
    // Update key state bitmap
    let mut state = KEY_STATE.lock();
    let byte_idx = (scancode / 8) as usize;
    let bit_idx = scancode % 8;
    if byte_idx < 32 {
        if pressed {
            state[byte_idx] |= 1 << bit_idx;
        } else {
            state[byte_idx] &= !(1 << bit_idx);
        }
    }
}

/// Add command to history
pub fn add_to_history(cmd: &str) {
    COMMAND_HISTORY.lock().add(cmd);
}

/// Get previous command from history
pub fn history_prev() -> Option<String> {
    COMMAND_HISTORY.lock().get_prev().map(String::from)
}

/// Get next command from history
pub fn history_next() -> Option<String> {
    COMMAND_HISTORY.lock().get_next().map(String::from)
}

/// Reset history browsing position
pub fn history_reset() {
    COMMAND_HISTORY.lock().reset_browse();
}

/// Get all history entries
pub fn history_list() -> Vec<(usize, String)> {
    COMMAND_HISTORY.lock().iter().map(|(i, s)| (i, String::from(s))).collect()
}

fn clipboard_set(text: &str) {
    *CLIPBOARD.lock() = Some(String::from(text));
}

fn clipboard_get() -> Option<String> {
    CLIPBOARD.lock().as_ref().map(|s| s.clone())
}

/// Read a line from keyboard with history support
pub fn read_line_with_history(buffer: &mut [u8]) -> usize {
    let mut pos = 0;
    let mut cursor = 0; // Cursor position (can differ from pos when editing mid-line)
    let mut current_input = String::new(); // Save current input when browsing history
    let mut select_all = false;
    
    // Reset history browsing
    history_reset();
    
    loop {
        if let Some(c) = read_char() {
            match c {
                b'\n' | b'\r' => {
                    crate::println!();
                    // Add to history if non-empty
                    let cmd = core::str::from_utf8(&buffer[..pos]).unwrap_or("");
                    if !cmd.trim().is_empty() {
                        add_to_history(cmd);
                    }
                    break;
                }
                0x01 => {
                    // Ctrl+A - select all
                    select_all = true;
                }
                0x08 => {
                    // Backspace - delete character before cursor
                    if select_all {
                        // Clear whole line
                        while cursor > 0 {
                            crate::print!("\x08");
                            cursor -= 1;
                        }
                        for _ in 0..pos {
                            crate::print!(" ");
                        }
                        for _ in 0..pos {
                            crate::print!("\x08");
                        }
                        pos = 0;
                        cursor = 0;
                        select_all = false;
                    } else if cursor > 0 {
                        // Shift everything after cursor left
                        for i in cursor..pos {
                            buffer[i - 1] = buffer[i];
                        }
                        pos = pos.saturating_sub(1);
                        cursor = cursor.saturating_sub(1);
                        
                        // Redraw: move cursor back, print rest of line, space, move back again
                        crate::print!("\x08");
                        for i in cursor..pos {
                            crate::print!("{}", buffer[i] as char);
                        }
                        crate::print!(" ");
                        for _ in cursor..=pos {
                            crate::print!("\x08");
                        }
                    }
                }
                KEY_UP => {
                    // Previous command in history
                    if let Some(prev) = history_prev() {
                        select_all = false;
                        // Clear current line
                        while cursor > 0 {
                            crate::print!("\x08");
                            cursor -= 1;
                        }
                        for _ in 0..pos {
                            crate::print!(" ");
                        }
                        for _ in 0..pos {
                            crate::print!("\x08");
                        }
                        // Display history entry
                        let bytes = prev.as_bytes();
                        let len = bytes.len().min(buffer.len() - 1);
                        buffer[..len].copy_from_slice(&bytes[..len]);
                        pos = len;
                        cursor = len;
                        crate::print!("{}", &prev[..len]);
                    }
                }
                KEY_DOWN => {
                    // Next command in history
                    let next = history_next();
                    select_all = false;
                    // Clear current line
                    while cursor > 0 {
                        crate::print!("\x08");
                        cursor -= 1;
                    }
                    for _ in 0..pos {
                        crate::print!(" ");
                    }
                    for _ in 0..pos {
                        crate::print!("\x08");
                    }
                    
                    if let Some(next_cmd) = next {
                        let bytes = next_cmd.as_bytes();
                        let len = bytes.len().min(buffer.len() - 1);
                        buffer[..len].copy_from_slice(&bytes[..len]);
                        pos = len;
                        cursor = len;
                        crate::print!("{}", &next_cmd[..len]);
                    } else {
                        pos = 0;
                        cursor = 0;
                    }
                }
                KEY_LEFT => {
                    select_all = false;
                    if cursor > 0 {
                        cursor -= 1;
                        crate::print!("\x08");
                    }
                }
                KEY_RIGHT => {
                    select_all = false;
                    if cursor < pos {
                        crate::print!("{}", buffer[cursor] as char);
                        cursor += 1;
                    }
                }
                KEY_HOME => {
                    select_all = false;
                    while cursor > 0 {
                        crate::print!("\x08");
                        cursor -= 1;
                    }
                }
                KEY_END => {
                    select_all = false;
                    while cursor < pos {
                        crate::print!("{}", buffer[cursor] as char);
                        cursor += 1;
                    }
                }
                KEY_DELETE => {
                    // Delete - remove character at cursor position
                    if select_all {
                        // Clear whole line
                        while cursor > 0 {
                            crate::print!("\x08");
                            cursor -= 1;
                        }
                        for _ in 0..pos {
                            crate::print!(" ");
                        }
                        for _ in 0..pos {
                            crate::print!("\x08");
                        }
                        pos = 0;
                        cursor = 0;
                        select_all = false;
                    } else if cursor < pos {
                        // Shift everything after cursor left
                        for i in cursor..pos.saturating_sub(1) {
                            buffer[i] = buffer[i + 1];
                        }
                        pos = pos.saturating_sub(1);
                        
                        // Redraw from cursor to end
                        for i in cursor..pos {
                            crate::print!("{}", buffer[i] as char);
                        }
                        crate::print!(" ");
                        // Move cursor back to original position
                        for _ in cursor..=pos {
                            crate::print!("\x08");
                        }
                    }
                }
                12 => {
                    // Ctrl+L - clear screen
                    crate::framebuffer::clear();
                    // Redraw prompt and current input
                    crate::print_color!(crate::framebuffer::COLOR_BRIGHT_GREEN, "trustos");
                    crate::print_color!(crate::framebuffer::COLOR_GREEN, "> ");
                    for i in 0..pos {
                        crate::print!("{}", buffer[i] as char);
                    }
                    // Move cursor to correct position
                    for _ in cursor..pos {
                        crate::print!("\x08");
                    }
                    select_all = false;
                }
                3 => {
                    // Ctrl+C - copy current line to clipboard
                    if let Ok(text) = core::str::from_utf8(&buffer[..pos]) {
                        clipboard_set(text);
                    }
                    select_all = false;
                }
                0x16 => {
                    // Ctrl+V - paste clipboard
                    if let Some(text) = clipboard_get() {
                        if select_all {
                            // Clear whole line before paste
                            while cursor > 0 {
                                crate::print!("\x08");
                                cursor -= 1;
                            }
                            for _ in 0..pos {
                                crate::print!(" ");
                            }
                            for _ in 0..pos {
                                crate::print!("\x08");
                            }
                            pos = 0;
                            cursor = 0;
                            select_all = false;
                        }
                        for b in text.bytes() {
                            if b < 0x20 || b >= 0x7F || pos >= buffer.len() - 1 {
                                continue;
                            }
                            if cursor < pos {
                                for i in (cursor..pos).rev() {
                                    buffer[i + 1] = buffer[i];
                                }
                            }
                            buffer[cursor] = b;
                            pos += 1;
                            cursor += 1;

                            for i in cursor - 1..pos {
                                crate::print!("{}", buffer[i] as char);
                            }
                            for _ in cursor..pos {
                                crate::print!("\x08");
                            }
                        }
                    }
                }
                _ if c >= 0x20 && c < 0x7F && pos < buffer.len() - 1 => {
                    // Printable character
                    if select_all {
                        // Clear whole line before inserting
                        while cursor > 0 {
                            crate::print!("\x08");
                            cursor -= 1;
                        }
                        for _ in 0..pos {
                            crate::print!(" ");
                        }
                        for _ in 0..pos {
                            crate::print!("\x08");
                        }
                        pos = 0;
                        cursor = 0;
                        select_all = false;
                    }
                    if cursor < pos {
                        // Insert at cursor position
                        for i in (cursor..pos).rev() {
                            buffer[i + 1] = buffer[i];
                        }
                    }
                    buffer[cursor] = c;
                    pos += 1;
                    cursor += 1;
                    
                    // Redraw from cursor
                    for i in cursor - 1..pos {
                        crate::print!("{}", buffer[i] as char);
                    }
                    // Move cursor back if we inserted
                    for _ in cursor..pos {
                        crate::print!("\x08");
                    }
                }
                _ => {}
            }
        } else {
            // Yield CPU while waiting
            x86_64::instructions::hlt();
        }
    }
    
    buffer[pos] = 0; // Null terminate
    pos
}

/// Read a line from keyboard (blocking, with echo) - legacy version
pub fn read_line(buffer: &mut [u8]) -> usize {
    read_line_with_history(buffer)
}

/// Read a line from keyboard with hidden input (for passwords)
pub fn read_line_hidden(buffer: &mut [u8]) -> usize {
    let mut pos = 0;
    
    loop {
        if let Some(c) = read_char() {
            match c {
                b'\n' | b'\r' => {
                    // Don't print newline here - caller will do it
                    break;
                }
                0x08 => {
                    // Backspace
                    if pos > 0 {
                        pos -= 1;
                        buffer[pos] = 0;
                        // Print asterisk backspace (optional visual feedback)
                        crate::print!("\x08 \x08");
                    }
                }
                0x03 => {
                    // Ctrl+C - cancel
                    pos = 0;
                    break;
                }
                0x15 => {
                    // Ctrl+U - clear line
                    for _ in 0..pos {
                        crate::print!("\x08 \x08");
                    }
                    pos = 0;
                }
                c if c >= 0x20 && c < 0x7F => {
                    // Printable character
                    if pos < buffer.len() - 1 {
                        buffer[pos] = c;
                        pos += 1;
                        // Show asterisk instead of actual character
                        crate::print!("*");
                    }
                }
                _ => {}
            }
        }
    }
    
    pos
}

// ═══════════════════════════════════════════════════════════════════════════════
// BLOCKING KEY INPUT for GUI
// ═══════════════════════════════════════════════════════════════════════════════

/// Try to read a key without blocking (returns scancode as ASCII equivalent)
/// Returns None if no key available
pub fn try_read_key() -> Option<u8> {
    read_char()
}

/// Wait for any key press (blocking)
pub fn wait_for_key() -> u8 {
    loop {
        if let Some(key) = read_char() {
            return key;
        }
        // Small delay to avoid burning CPU
        for _ in 0..1000 {
            core::hint::spin_loop();
        }
    }
}
