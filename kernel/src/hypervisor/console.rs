//! Virtual Console for VMs
//!
//! Console virtuelle pour les machines virtuelles
//! Permet la communication I/O entre le guest et l'host

use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use alloc::collections::VecDeque;
use spin::Mutex;

/// Buffer size for console I/O
const CONSOLE_BUFFER_SIZE: usize = 4096;

/// Virtual console for a VM
pub struct VirtConsole {
    /// VM ID
    vm_id: u64,
    /// Output buffer (guest -> host)
    output_buffer: VecDeque<u8>,
    /// Input buffer (host -> guest)
    input_buffer: VecDeque<u8>,
    /// Echo mode enabled
    echo: bool,
    /// Line buffer for line-oriented input
    line_buffer: String,
    /// Console name
    name: String,
}

impl VirtConsole {
    pub fn new(vm_id: u64, name: &str) -> Self {
        VirtConsole {
            vm_id,
            output_buffer: VecDeque::with_capacity(CONSOLE_BUFFER_SIZE),
            input_buffer: VecDeque::with_capacity(CONSOLE_BUFFER_SIZE),
            echo: true,
            line_buffer: String::new(),
            name: String::from(name),
        }
    }
    
    /// Write a byte from guest to console output
    pub fn write_byte(&mut self, byte: u8) {
        if self.output_buffer.len() < CONSOLE_BUFFER_SIZE {
            self.output_buffer.push_back(byte);
        }
        
        // Also output to serial for debugging
        let c = byte as char;
        crate::serial_print!("{}", c);
    }
    
    /// Write a string from guest
    pub fn write_str(&mut self, s: &str) {
        for byte in s.bytes() {
            self.write_byte(byte);
        }
    }
    
    /// Read a byte for the guest (from input buffer)
    pub fn read_byte(&mut self) -> Option<u8> {
        self.input_buffer.pop_front()
    }
    
    /// Check if input is available
    pub fn has_input(&self) -> bool {
        !self.input_buffer.is_empty()
    }
    
    /// Get input buffer status for guest
    pub fn input_status(&self) -> u8 {
        if self.input_buffer.is_empty() {
            0x00 // No data available
        } else {
            0x01 // Data available
        }
    }
    
    /// Get output buffer status for guest  
    pub fn output_status(&self) -> u8 {
        if self.output_buffer.len() < CONSOLE_BUFFER_SIZE {
            0x20 // TX ready (bit 5)
        } else {
            0x00 // TX busy
        }
    }
    
    /// Inject input from host to guest
    pub fn inject_input(&mut self, data: &[u8]) {
        for &byte in data {
            if self.input_buffer.len() < CONSOLE_BUFFER_SIZE {
                self.input_buffer.push_back(byte);
            }
        }
    }
    
    /// Inject a line from host to guest
    pub fn inject_line(&mut self, line: &str) {
        self.inject_input(line.as_bytes());
        self.inject_input(b"\n");
    }
    
    /// Get and clear output buffer
    pub fn drain_output(&mut self) -> Vec<u8> {
        self.output_buffer.drain(..).collect()
    }
    
    /// Get output as string
    pub fn drain_output_string(&mut self) -> String {
        let bytes: Vec<u8> = self.drain_output();
        String::from_utf8_lossy(&bytes).into_owned()
    }
    
    /// Peek at output buffer without draining
    pub fn peek_output(&self) -> String {
        let bytes: Vec<u8> = self.output_buffer.iter().copied().collect();
        String::from_utf8_lossy(&bytes).into_owned()
    }
}

/// Console manager for all VMs
pub struct ConsoleManager {
    consoles: Vec<VirtConsole>,
}

impl ConsoleManager {
    pub const fn new() -> Self {
        ConsoleManager {
            consoles: Vec::new(),
        }
    }
    
    pub fn create_console(&mut self, vm_id: u64, name: &str) -> usize {
        let console = VirtConsole::new(vm_id, name);
        let idx = self.consoles.len();
        self.consoles.push(console);
        idx
    }
    
    pub fn get_console(&mut self, vm_id: u64) -> Option<&mut VirtConsole> {
        self.consoles.iter_mut().find(|c| c.vm_id == vm_id)
    }
    
    pub fn remove_console(&mut self, vm_id: u64) {
        self.consoles.retain(|c| c.vm_id != vm_id);
    }
}

/// Global console manager
static CONSOLE_MGR: Mutex<ConsoleManager> = Mutex::new(ConsoleManager::new());

/// Get console for a VM
pub fn get_console(vm_id: u64) -> Option<spin::MutexGuard<'static, ConsoleManager>> {
    let mgr = CONSOLE_MGR.lock();
    Some(mgr)
}

/// Create a console for a VM
pub fn create_console(vm_id: u64, name: &str) -> usize {
    CONSOLE_MGR.lock().create_console(vm_id, name)
}

/// Write a character to a VM's console by console index
pub fn write_char(console_id: usize, ch: char) {
    let mut mgr = CONSOLE_MGR.lock();
    if console_id < mgr.consoles.len() {
        mgr.consoles[console_id].write_byte(ch as u8);
    }
}

/// Handle console I/O port access from guest
pub fn handle_console_io(vm_id: u64, port: u16, is_write: bool, value: u8) -> u8 {
    let mut mgr = CONSOLE_MGR.lock();
    
    if let Some(console) = mgr.get_console(vm_id) {
        match port {
            // Data port (COM1 style)
            0x3F8 => {
                if is_write {
                    console.write_byte(value);
                    0
                } else {
                    console.read_byte().unwrap_or(0)
                }
            }
            // Line Status Register
            0x3FD => {
                let mut status = 0x60; // TX empty, TX holding register empty
                if console.has_input() {
                    status |= 0x01; // Data ready
                }
                status
            }
            // Debug port (QEMU bochs-style)
            0xE9 => {
                if is_write {
                    console.write_byte(value);
                }
                0
            }
            _ => 0xFF,
        }
    } else {
        0xFF
    }
}

/// Inject input to a VM's console
pub fn inject_input(vm_id: u64, data: &[u8]) {
    let mut mgr = CONSOLE_MGR.lock();
    if let Some(console) = mgr.get_console(vm_id) {
        console.inject_input(data);
    }
}

/// Get output from a VM's console
pub fn get_output(vm_id: u64) -> String {
    let mut mgr = CONSOLE_MGR.lock();
    if let Some(console) = mgr.get_console(vm_id) {
        console.drain_output_string()
    } else {
        String::new()
    }
}
