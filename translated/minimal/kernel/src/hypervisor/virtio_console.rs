




use alloc::collections::VecDeque;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::boxed::Box;
use spin::Mutex;


const WG_: usize = 4096;


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PortState {
    Closed,
    Open,
    Ready,
}


pub struct ConsolePort {
    
    pub id: u32,
    
    pub name: String,
    
    pub state: PortState,
    
    input_buffer: VecDeque<u8>,
    
    output_buffer: VecDeque<u8>,
    
    on_data: Option<fn(&[u8])>,
}

impl ConsolePort {
    pub fn new(id: u32, name: &str) -> Self {
        Self {
            id,
            name: String::from(name),
            state: PortState::Closed,
            input_buffer: VecDeque::with_capacity(WG_),
            output_buffer: VecDeque::with_capacity(WG_),
            on_data: None,
        }
    }

    
    pub fn write(&mut self, data: &[u8]) -> usize {
        let available = WG_.saturating_sub(self.output_buffer.len());
        let bpo = data.len().min(available);
        for &byte in &data[..bpo] {
            self.output_buffer.push_back(byte);
        }
        bpo
    }

    
    pub fn write_str(&mut self, j: &str) -> usize {
        self.write(j.as_bytes())
    }

    
    pub fn read(&mut self, buf: &mut [u8]) -> usize {
        let rz = buf.len().min(self.input_buffer.len());
        for i in 0..rz {
            buf[i] = self.input_buffer.pop_front().unwrap_or(0);
        }
        rz
    }

    
    pub fn read_line(&mut self) -> Option<String> {
        
        let njx = self.input_buffer.iter().position(|&b| b == b'\n');
        
        if let Some(pos) = njx {
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

    
    pub fn has_data(&self) -> bool {
        !self.input_buffer.is_empty()
    }

    
    pub(crate) fn receive(&mut self, data: &[u8]) {
        for &byte in data {
            if self.input_buffer.len() < WG_ {
                self.input_buffer.push_back(byte);
            }
        }
        
        if let Some(callback) = self.on_data {
            callback(data);
        }
    }

    
    pub(crate) fn drain_output(&mut self) -> Vec<u8> {
        self.output_buffer.drain(..).collect()
    }

    
    pub fn qvm(&mut self, callback: fn(&[u8])) {
        self.on_data = Some(callback);
    }
}


pub struct VirtioConsole {
    
    pub vm_id: u64,
    
    ports: Vec<ConsolePort>,
    
    multiport: bool,
}

impl VirtioConsole {
    
    pub fn new(vm_id: u64) -> Self {
        let mut console = Self {
            vm_id,
            ports: Vec::new(),
            multiport: true,
        };
        
        console.add_port("console");
        console
    }

    
    pub fn add_port(&mut self, name: &str) -> u32 {
        let id = self.ports.len() as u32;
        self.ports.push(ConsolePort::new(id, name));
        id
    }

    
    pub fn port(&self, id: u32) -> Option<&ConsolePort> {
        self.ports.get(id as usize)
    }

    
    pub fn port_mut(&mut self, id: u32) -> Option<&mut ConsolePort> {
        self.ports.get_mut(id as usize)
    }

    
    pub fn main_port(&mut self) -> &mut ConsolePort {
        if self.ports.is_empty() {
            self.add_port("console");
        }
        &mut self.ports[0]
    }

    
    pub fn write(&mut self, data: &[u8]) -> usize {
        self.main_port().write(data)
    }

    
    pub fn print(&mut self, j: &str) -> usize {
        self.main_port().write_str(j)
    }

    
    pub fn read(&mut self, buf: &mut [u8]) -> usize {
        self.main_port().read(buf)
    }

    
    pub fn read_line(&mut self) -> Option<String> {
        self.main_port().read_line()
    }

    
    pub fn has_data(&self) -> bool {
        self.ports.first().map_or(false, |aa| aa.has_data())
    }

    
    pub fn qjz(&mut self, port_id: u32, data: &[u8]) {
        if let Some(port) = self.port_mut(port_id) {
            port.receive(data);
        }
    }

    
    pub fn qjy(&mut self, port_id: u32) -> Vec<u8> {
        if let Some(port) = self.port_mut(port_id) {
            port.drain_output()
        } else {
            Vec::new()
        }
    }
}


pub struct ConsoleManager {
    
    consoles: Vec<VirtioConsole>,
}

impl ConsoleManager {
    pub const fn new() -> Self {
        Self {
            consoles: Vec::new(),
        }
    }

    
    pub fn create_console(&mut self, vm_id: u64) -> &mut VirtioConsole {
        
        if let Some(idx) = self.consoles.iter().position(|c| c.vm_id == vm_id) {
            return &mut self.consoles[idx];
        }
        
        self.consoles.push(VirtioConsole::new(vm_id));
        self.consoles.last_mut().unwrap()
    }

    
    pub fn get_console(&self, vm_id: u64) -> Option<&VirtioConsole> {
        self.consoles.iter().find(|c| c.vm_id == vm_id)
    }

    
    pub fn get_console_mut(&mut self, vm_id: u64) -> Option<&mut VirtioConsole> {
        self.consoles.iter_mut().find(|c| c.vm_id == vm_id)
    }

    
    pub fn oew(&mut self, vm_id: u64) {
        self.consoles.retain(|c| c.vm_id != vm_id);
    }
}


static BRJ_: Mutex<ConsoleManager> = Mutex::new(ConsoleManager::new());


pub fn dlj() -> spin::MutexGuard<'static, ConsoleManager> {
    BRJ_.lock()
}


pub fn create_console(vm_id: u64) {
    dlj().create_console(vm_id);
}


pub fn rdo(vm_id: u64, data: &[u8]) -> usize {
    if let Some(console) = dlj().get_console_mut(vm_id) {
        console.write(data)
    } else {
        0
    }
}


pub fn qse(vm_id: u64, buf: &mut [u8]) -> usize {
    if let Some(console) = dlj().get_console_mut(vm_id) {
        console.read(buf)
    } else {
        0
    }
}


pub fn rcb(vm_id: u64) -> bool {
    if let Some(console) = dlj().get_console(vm_id) {
        console.has_data()
    } else {
        false
    }
}


pub fn qsj(vm_id: u64) -> Option<String> {
    if let Some(console) = dlj().get_console_mut(vm_id) {
        console.read_line()
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn qzd() {
        let mut port = ConsolePort::new(0, "test");
        port.receive(b"Hello, World!\n");
        
        assert!(port.has_data());
        
        let line = port.read_line();
        assert_eq!(line, Some(String::from("Hello, World!")));
    }
}
