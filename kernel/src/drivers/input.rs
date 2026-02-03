//! Input Device Detection and Management
//!
//! Handles keyboard and mouse detection across PS/2 and USB.

use spin::Mutex;

/// Input device type
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum InputDeviceType {
    PS2Keyboard,
    PS2Mouse,
    USBKeyboard,
    USBMouse,
    USBGamepad,
}

/// Input device info
#[derive(Clone, Debug)]
pub struct InputDevice {
    pub device_type: InputDeviceType,
    pub name: &'static str,
    pub available: bool,
}

/// Input subsystem state
static INPUT_STATE: Mutex<InputState> = Mutex::new(InputState::new());

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

/// Initialize input subsystem
pub fn init() {
    let mut state = INPUT_STATE.lock();
    
    // PS/2 keyboard is typically always available
    // (initialized by keyboard::init)
    state.keyboard_available = true;
    state.keyboard_type = Some(InputDeviceType::PS2Keyboard);
    
    // PS/2 mouse depends on mouse::init success
    if crate::mouse::is_initialized() {
        state.mouse_available = true;
        state.mouse_type = Some(InputDeviceType::PS2Mouse);
    }
    
    crate::serial_println!("[INPUT] Keyboard: {:?}, Mouse: {:?}",
        state.keyboard_type, state.mouse_type);
}

/// Check if keyboard is available
pub fn has_keyboard() -> bool {
    INPUT_STATE.lock().keyboard_available
}

/// Check if mouse is available
pub fn has_mouse() -> bool {
    INPUT_STATE.lock().mouse_available
}

/// Get keyboard device type
pub fn keyboard_type() -> Option<InputDeviceType> {
    INPUT_STATE.lock().keyboard_type
}

/// Get mouse device type
pub fn mouse_type() -> Option<InputDeviceType> {
    INPUT_STATE.lock().mouse_type
}

/// Switch to USB HID if detected
pub fn switch_to_usb_hid(_keyboard: bool, _mouse: bool) {
    let mut state = INPUT_STATE.lock();
    
    // This would be called when USB HID devices are enumerated
    // For now, we always use PS/2
    
    // TODO: When USB HID is implemented:
    // if keyboard {
    //     state.keyboard_type = Some(InputDeviceType::USBKeyboard);
    // }
    // if mouse {
    //     state.mouse_type = Some(InputDeviceType::USBMouse);
    // }
    
    let _ = state;  // Silence unused warning
}
