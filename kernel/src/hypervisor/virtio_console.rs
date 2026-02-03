//! Virtio Console Driver for VM Communication
//!
//! Provides bidirectional communication between TrustOS host and guest VMs
//! via virtio-console protocol. Used primarily by the Linux Subsystem.

use alloc::collections::VecDeque;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::boxed::Box;
use spin::Mutex;

/// Maximum buffer size for console I/O
const MAX_BUFFER_SIZE: usize = 4096;

/// Virtio console port state
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PortState {
    Closed,
    Open,
    Ready,
}

/// A single virtio console port
pub struct ConsolePort {
    /// Port ID
    pub id: u32,
    /// Port name
    pub name: String,
    /// Current state
    pub state: PortState,
    /// Input buffer (data from guest)
    input_buffer: VecDeque<u8>,
    /// Output buffer (data to guest)
    output_buffer: VecDeque<u8>,
    /// Callback for data received
    on_data: Option<fn(&[u8])>,
}

impl ConsolePort {
    pub fn new(id: u32, name: &str) -> Self {
        Self {
            id,
            name: String::from(name),
            state: PortState::Closed,
            input_buffer: VecDeque::with_capacity(MAX_BUFFER_SIZE),
            output_buffer: VecDeque::with_capacity(MAX_BUFFER_SIZE),
            on_data: None,
        }
    }

    /// Write data to be sent to guest
    pub fn write(&mut self, data: &[u8]) -> usize {
        let available = MAX_BUFFER_SIZE.saturating_sub(self.output_buffer.len());
        let to_write = data.len().min(available);
        for &byte in &data[..to_write] {
            self.output_buffer.push_back(byte);
        }
        to_write
    }

    /// Write a string to the guest
    pub fn write_str(&mut self, s: &str) -> usize {
        self.write(s.as_bytes())
    }

    /// Read data received from guest
    pub fn read(&mut self, buf: &mut [u8]) -> usize {
        let to_read = buf.len().min(self.input_buffer.len());
        for i in 0..to_read {
            buf[i] = self.input_buffer.pop_front().unwrap_or(0);
        }
        to_read
    }

    /// Read a line from guest (blocking-ish)
    pub fn read_line(&mut self) -> Option<String> {
        // Look for newline in buffer
        let newline_pos = self.input_buffer.iter().position(|&b| b == b'\n');
        
        if let Some(pos) = newline_pos {
            let mut line = String::new();
            for _ in 0..=pos {
                if let Some(byte) = self.input_buffer.pop_front() {
                    if byte != b'\n' && byte != b'\r' {
                        line.push(byte as char);
                    }
                }
            }
            Some(line)
        } else {
            None
        }
    }

    /// Check if there's data available to read
    pub fn has_data(&self) -> bool {
        !self.input_buffer.is_empty()
    }

    /// Receive data from guest (internal use)
    pub(crate) fn receive(&mut self, data: &[u8]) {
        for &byte in data {
            if self.input_buffer.len() < MAX_BUFFER_SIZE {
                self.input_buffer.push_back(byte);
            }
        }
        // Call callback if registered
        if let Some(callback) = self.on_data {
            callback(data);
        }
    }

    /// Get pending output data for guest (internal use)
    pub(crate) fn drain_output(&mut self) -> Vec<u8> {
        self.output_buffer.drain(..).collect()
    }

    /// Set data received callback
    pub fn set_callback(&mut self, callback: fn(&[u8])) {
        self.on_data = Some(callback);
    }
}

/// Virtio Console Device for a VM
pub struct VirtioConsole {
    /// VM ID this console belongs to
    pub vm_id: u64,
    /// Console ports (typically port 0 is the main console)
    ports: Vec<ConsolePort>,
    /// Whether console is multiport capable
    multiport: bool,
}

impl VirtioConsole {
    /// Create a new virtio console for a VM
    pub fn new(vm_id: u64) -> Self {
        let mut console = Self {
            vm_id,
            ports: Vec::new(),
            multiport: true,
        };
        // Add default console port (port 0)
        console.add_port("console");
        console
    }

    /// Add a new console port
    pub fn add_port(&mut self, name: &str) -> u32 {
        let id = self.ports.len() as u32;
        self.ports.push(ConsolePort::new(id, name));
        id
    }

    /// Get a port by ID
    pub fn port(&self, id: u32) -> Option<&ConsolePort> {
        self.ports.get(id as usize)
    }

    /// Get a mutable port by ID
    pub fn port_mut(&mut self, id: u32) -> Option<&mut ConsolePort> {
        self.ports.get_mut(id as usize)
    }

    /// Get the main console port (port 0)
    pub fn main_port(&mut self) -> &mut ConsolePort {
        if self.ports.is_empty() {
            self.add_port("console");
        }
        &mut self.ports[0]
    }

    /// Write to main console
    pub fn write(&mut self, data: &[u8]) -> usize {
        self.main_port().write(data)
    }

    /// Write string to main console
    pub fn print(&mut self, s: &str) -> usize {
        self.main_port().write_str(s)
    }

    /// Read from main console
    pub fn read(&mut self, buf: &mut [u8]) -> usize {
        self.main_port().read(buf)
    }

    /// Read line from main console
    pub fn read_line(&mut self) -> Option<String> {
        self.main_port().read_line()
    }

    /// Check if data is available
    pub fn has_data(&self) -> bool {
        self.ports.first().map_or(false, |p| p.has_data())
    }

    /// Process guest output (called by VM on I/O exit)
    pub fn guest_write(&mut self, port_id: u32, data: &[u8]) {
        if let Some(port) = self.port_mut(port_id) {
            port.receive(data);
        }
    }

    /// Get data to send to guest (called by VM before VMRUN)
    pub fn guest_read(&mut self, port_id: u32) -> Vec<u8> {
        if let Some(port) = self.port_mut(port_id) {
            port.drain_output()
        } else {
            Vec::new()
        }
    }
}

/// Global console manager for all VMs
pub struct ConsoleManager {
    /// Consoles indexed by VM ID
    consoles: Vec<VirtioConsole>,
}

impl ConsoleManager {
    pub const fn new() -> Self {
        Self {
            consoles: Vec::new(),
        }
    }

    /// Create a console for a VM
    pub fn create_console(&mut self, vm_id: u64) -> &mut VirtioConsole {
        // Check if already exists
        if let Some(idx) = self.consoles.iter().position(|c| c.vm_id == vm_id) {
            return &mut self.consoles[idx];
        }
        
        self.consoles.push(VirtioConsole::new(vm_id));
        self.consoles.last_mut().unwrap()
    }

    /// Get console for a VM
    pub fn get_console(&self, vm_id: u64) -> Option<&VirtioConsole> {
        self.consoles.iter().find(|c| c.vm_id == vm_id)
    }

    /// Get mutable console for a VM
    pub fn get_console_mut(&mut self, vm_id: u64) -> Option<&mut VirtioConsole> {
        self.consoles.iter_mut().find(|c| c.vm_id == vm_id)
    }

    /// Remove console for a VM
    pub fn remove_console(&mut self, vm_id: u64) {
        self.consoles.retain(|c| c.vm_id != vm_id);
    }
}

/// Global console manager
static CONSOLE_MANAGER: Mutex<ConsoleManager> = Mutex::new(ConsoleManager::new());

/// Get access to the console manager
pub fn console_manager() -> spin::MutexGuard<'static, ConsoleManager> {
    CONSOLE_MANAGER.lock()
}

/// Create a console for a VM
pub fn create_console(vm_id: u64) {
    console_manager().create_console(vm_id);
}

/// Write to a VM's console
pub fn write_to_vm(vm_id: u64, data: &[u8]) -> usize {
    if let Some(console) = console_manager().get_console_mut(vm_id) {
        console.write(data)
    } else {
        0
    }
}

/// Read from a VM's console
pub fn read_from_vm(vm_id: u64, buf: &mut [u8]) -> usize {
    if let Some(console) = console_manager().get_console_mut(vm_id) {
        console.read(buf)
    } else {
        0
    }
}

/// Check if a VM has console data available
pub fn vm_has_data(vm_id: u64) -> bool {
    if let Some(console) = console_manager().get_console(vm_id) {
        console.has_data()
    } else {
        false
    }
}

/// Read a line from a VM's console
pub fn read_line_from_vm(vm_id: u64) -> Option<String> {
    if let Some(console) = console_manager().get_console_mut(vm_id) {
        console.read_line()
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_console_port_write_read() {
        let mut port = ConsolePort::new(0, "test");
        port.receive(b"Hello, World!\n");
        
        assert!(port.has_data());
        
        let line = port.read_line();
        assert_eq!(line, Some(String::from("Hello, World!")));
    }
}
