




use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use alloc::collections::VecDeque;
use spin::Mutex;


const NL_: usize = 4096;


pub struct VirtConsole {
    
    vm_id: u64,
    
    output_buffer: VecDeque<u8>,
    
    input_buffer: VecDeque<u8>,
    
    cxa: bool,
    
    line_buffer: String,
    
    name: String,
}

impl VirtConsole {
    pub fn new(vm_id: u64, name: &str) -> Self {
        VirtConsole {
            vm_id,
            output_buffer: VecDeque::with_capacity(NL_),
            input_buffer: VecDeque::with_capacity(NL_),
            cxa: true,
            line_buffer: String::new(),
            name: String::from(name),
        }
    }
    
    
    pub fn write_byte(&mut self, byte: u8) {
        if self.output_buffer.len() < NL_ {
            self.output_buffer.push_back(byte);
        }
        
        
        let c = byte as char;
        crate::serial_print!("{}", c);
    }
    
    
    pub fn write_str(&mut self, j: &str) {
        for byte in j.bytes() {
            self.write_byte(byte);
        }
    }
    
    
    pub fn read_byte(&mut self) -> Option<u8> {
        self.input_buffer.pop_front()
    }
    
    
    pub fn has_input(&self) -> bool {
        !self.input_buffer.is_empty()
    }
    
    
    pub fn qlm(&self) -> u8 {
        if self.input_buffer.is_empty() {
            0x00 
        } else {
            0x01 
        }
    }
    
    
    pub fn qqb(&self) -> u8 {
        if self.output_buffer.len() < NL_ {
            0x20 
        } else {
            0x00 
        }
    }
    
    
    pub fn inject_input(&mut self, data: &[u8]) {
        for &byte in data {
            if self.input_buffer.len() < NL_ {
                self.input_buffer.push_back(byte);
            }
        }
    }
    
    
    pub fn qli(&mut self, line: &str) {
        self.inject_input(line.as_bytes());
        self.inject_input(b"\n");
    }
    
    
    pub fn drain_output(&mut self) -> Vec<u8> {
        self.output_buffer.drain(..).collect()
    }
    
    
    pub fn drain_output_string(&mut self) -> String {
        let bytes: Vec<u8> = self.drain_output();
        String::from_utf8_lossy(&bytes).into_owned()
    }
    
    
    pub fn qqg(&self) -> String {
        let bytes: Vec<u8> = self.output_buffer.iter().copied().collect();
        String::from_utf8_lossy(&bytes).into_owned()
    }
}


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
    
    pub fn oew(&mut self, vm_id: u64) {
        self.consoles.retain(|c| c.vm_id != vm_id);
    }
}


static KK_: Mutex<ConsoleManager> = Mutex::new(ConsoleManager::new());


pub fn get_console(vm_id: u64) -> Option<spin::MutexGuard<'static, ConsoleManager>> {
    let ng = KK_.lock();
    Some(ng)
}


pub fn create_console(vm_id: u64, name: &str) -> usize {
    KK_.lock().create_console(vm_id, name)
}


pub fn write_char(console_id: usize, ch: char) {
    let mut ng = KK_.lock();
    if console_id < ng.consoles.len() {
        ng.consoles[console_id].write_byte(ch as u8);
    }
}


pub fn idg(vm_id: u64, port: u16, is_write: bool, value: u8) -> u8 {
    let mut ng = KK_.lock();
    
    if let Some(console) = ng.get_console(vm_id) {
        match port {
            
            0x3F8 => {
                if is_write {
                    console.write_byte(value);
                    0
                } else {
                    console.read_byte().unwrap_or(0)
                }
            }
            
            0x3FD => {
                let mut status = 0x60; 
                if console.has_input() {
                    status |= 0x01; 
                }
                status
            }
            
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


pub fn inject_input(vm_id: u64, data: &[u8]) {
    let mut ng = KK_.lock();
    if let Some(console) = ng.get_console(vm_id) {
        console.inject_input(data);
    }
}


pub fn mdo(vm_id: u64) -> String {
    let mut ng = KK_.lock();
    if let Some(console) = ng.get_console(vm_id) {
        console.drain_output_string()
    } else {
        String::new()
    }
}
