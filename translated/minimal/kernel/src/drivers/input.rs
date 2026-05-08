



use spin::Mutex;


#[derive(Clone, Copy, Debug, PartialEq)]
pub enum InputDeviceType {
    PS2Keyboard,
    PS2Mouse,
    USBKeyboard,
    USBMouse,
    USBGamepad,
}


#[derive(Clone, Debug)]
pub struct Axz {
    pub device_type: InputDeviceType,
    pub name: &'static str,
    pub available: bool,
}


static LF_: Mutex<InputState> = Mutex::new(InputState::new());

struct InputState {
    keyboard_available: bool,
    mouse_available: bool,
    keyboard_type: Option<InputDeviceType>,
    mouse_type: Option<InputDeviceType>,
}

impl InputState {
    const fn new() -> Self {
        InputState {
            keyboard_available: false,
            mouse_available: false,
            keyboard_type: None,
            mouse_type: None,
        }
    }
}


pub fn init() {
    let mut state = LF_.lock();
    
    
    
    state.keyboard_available = true;
    state.keyboard_type = Some(InputDeviceType::PS2Keyboard);
    
    
    if crate::mouse::is_initialized() {
        state.mouse_available = true;
        state.mouse_type = Some(InputDeviceType::PS2Mouse);
    }
    
    crate::serial_println!("[INPUT] Keyboard: {:?}, Mouse: {:?}",
        state.keyboard_type, state.mouse_type);
}


pub fn idr() -> bool {
    LF_.lock().keyboard_available
}


pub fn mjr() -> bool {
    LF_.lock().mouse_available
}


pub fn keyboard_type() -> Option<InputDeviceType> {
    LF_.lock().keyboard_type
}


pub fn mouse_type() -> Option<InputDeviceType> {
    LF_.lock().mouse_type
}


pub fn qyh(_keyboard: bool, _mouse: bool) {
    let mut state = LF_.lock();
    
    
    
    
    
    
    
    
    
    
    
    
    let _ = state;  
}
