//! Wayland Protocol Definitions
//!
//! Core Wayland protocol message types and opcodes.
//! Based on the official Wayland protocol specification.

use alloc::string::String;
use alloc::vec::Vec;

// ═══════════════════════════════════════════════════════════════════════════════
// PROTOCOL OPCODES
// ═══════════════════════════════════════════════════════════════════════════════

/// wl_display opcodes
pub mod wl_display {
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const SYNC: u16 = 0;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const GET_REGISTRY: u16 = 1;
    
    // Events
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const ERROR: u16 = 0;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const DELETE_ID: u16 = 1;
}

/// wl_registry opcodes
pub mod wl_registry {
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const BIND: u16 = 0;
    
    // Events
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const GLOBAL: u16 = 0;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const GLOBAL_REMOVE: u16 = 1;
}

/// wl_compositor opcodes
pub mod wl_compositor {
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const CREATE_SURFACE: u16 = 0;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const CREATE_REGION: u16 = 1;
}

/// wl_surface opcodes
pub mod wl_surface {
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const DESTROY: u16 = 0;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const ATTACH: u16 = 1;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const DAMAGE: u16 = 2;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const FRAME: u16 = 3;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const SET_OPAQUE_REGION: u16 = 4;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const SET_INPUT_REGION: u16 = 5;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const COMMIT: u16 = 6;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const SET_BUFFER_TRANSFORM: u16 = 7;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const SET_BUFFER_SCALE: u16 = 8;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const DAMAGE_BUFFER: u16 = 9;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const OFFSET: u16 = 10;
    
    // Events
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const ENTER: u16 = 0;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const LEAVE: u16 = 1;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const PREFERRED_BUFFER_SCALE: u16 = 2;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const PREFERRED_BUFFER_TRANSFORM: u16 = 3;
}

/// wl_shm opcodes
pub mod wl_shm {
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const CREATE_POOL: u16 = 0;
    
    // Events
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const FORMAT: u16 = 0;
}

/// wl_shm_pool opcodes
pub mod wl_shm_pool {
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const CREATE_BUFFER: u16 = 0;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const DESTROY: u16 = 1;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const RESIZE: u16 = 2;
}

/// wl_buffer opcodes
pub mod wl_buffer {
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const DESTROY: u16 = 0;
    
    // Events
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const RELEASE: u16 = 0;
}

/// wl_seat opcodes
pub mod wl_seat {
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const GET_POINTER: u16 = 0;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const GET_KEYBOARD: u16 = 1;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const GET_TOUCH: u16 = 2;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const RELEASE: u16 = 3;
    
    // Events
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const CAPABILITIES: u16 = 0;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const NAME: u16 = 1;
}

/// wl_pointer opcodes
pub mod wl_pointer {
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const SET_CURSOR: u16 = 0;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const RELEASE: u16 = 1;
    
    // Events
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const ENTER: u16 = 0;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const LEAVE: u16 = 1;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const MOTION: u16 = 2;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const BUTTON: u16 = 3;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const AXIS: u16 = 4;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const FRAME: u16 = 5;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const AXIS_SOURCE: u16 = 6;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const AXIS_STOP: u16 = 7;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const AXIS_DISCRETE: u16 = 8;
}

/// wl_keyboard opcodes
pub mod wl_keyboard {
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const RELEASE: u16 = 0;
    
    // Events
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const KEYMAP: u16 = 0;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const ENTER: u16 = 1;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const LEAVE: u16 = 2;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const KEY: u16 = 3;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const MODIFIERS: u16 = 4;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const REPEAT_INFORMATION: u16 = 5;
}

/// xdg_wm_base opcodes (xdg-shell)
pub mod xdg_wm_base {
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const DESTROY: u16 = 0;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const CREATE_POSITIONER: u16 = 1;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const GET_XDG_SURFACE: u16 = 2;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const PONG: u16 = 3;
    
    // Events
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const PING: u16 = 0;
}

/// xdg_surface opcodes
pub mod xdg_surface {
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const DESTROY: u16 = 0;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const GET_TOPLEVEL: u16 = 1;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const GET_POPUP: u16 = 2;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const SET_WINDOW_GEOMETRY: u16 = 3;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const ACKNOWLEDGE_CONFIGURE: u16 = 4;
    
    // Events
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const CONFIGURE: u16 = 0;
}

/// xdg_toplevel opcodes
pub mod xdg_toplevel {
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const DESTROY: u16 = 0;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const SET_PARENT: u16 = 1;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const SET_TITLE: u16 = 2;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const SET_APP_ID: u16 = 3;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const SHOW_WINDOW_MENU: u16 = 4;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const MOVE: u16 = 5;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const RESIZE: u16 = 6;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const SET_MAXIMUM_SIZE: u16 = 7;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const SET_MINIMUM_SIZE: u16 = 8;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const SET_MAXIMIZED: u16 = 9;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const UNSET_MAXIMIZED: u16 = 10;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const SET_FULLSCREEN: u16 = 11;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const UNSET_FULLSCREEN: u16 = 12;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const SET_MINIMIZED: u16 = 13;
    
    // Events
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const CONFIGURE: u16 = 0;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const CLOSE: u16 = 1;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const CONFIGURE_BOUNDS: u16 = 2;
}

// ═══════════════════════════════════════════════════════════════════════════════
// WIRE FORMAT
// ═══════════════════════════════════════════════════════════════════════════════

/// Wayland message header (8 bytes)
#[repr(C)]
// #[derive] — auto-generates trait implementations at compile time.
#[derive(Debug, Clone, Copy)]
// Public structure — visible outside this module.
pub struct WlMessageHeader {
    /// Object ID this message is for
    pub object_id: u32,
    /// Opcode (16-bit) and message size (16-bit)
    pub opcode_size: u32,
}

// Implementation block — defines methods for the type above.
impl WlMessageHeader {
        // Public function — callable from other modules.
pub fn new(object_id: u32, opcode: u16, size: u16) -> Self {
        Self {
            object_id,
            opcode_size: (size as u32) << 16 | opcode as u32,
        }
    }
    
        // Public function — callable from other modules.
pub fn opcode(&self) -> u16 {
        (self.opcode_size & 0xFFFF) as u16
    }
    
        // Public function — callable from other modules.
pub fn size(&self) -> u16 {
        (self.opcode_size >> 16) as u16
    }
}

/// A Wayland protocol message
#[derive(Debug, Clone)]
// Public structure — visible outside this module.
pub struct WlMessage {
    pub header: WlMessageHeader,
    pub payload: Vec<u8>,
}

// Implementation block — defines methods for the type above.
impl WlMessage {
        // Public function — callable from other modules.
pub fn new(object_id: u32, opcode: u16) -> Self {
        Self {
            header: WlMessageHeader::new(object_id, opcode, 8),
            payload: Vec::new(),
        }
    }
    
        // Public function — callable from other modules.
pub fn with_payload(object_id: u32, opcode: u16, payload: Vec<u8>) -> Self {
        let size = 8 + payload.len() as u16;
        Self {
            header: WlMessageHeader::new(object_id, opcode, size),
            payload,
        }
    }
    
    /// Add a u32 argument
    pub fn push_u32(&mut self, value: u32) {
        self.payload.extend_from_slice(&value.to_ne_bytes());
        self.update_size();
    }
    
    /// Add an i32 argument
    pub fn push_i32(&mut self, value: i32) {
        self.payload.extend_from_slice(&value.to_ne_bytes());
        self.update_size();
    }
    
    /// Add a string argument
    pub fn push_string(&mut self, s: &str) {
        let len = s.len() as u32 + 1; // Include null terminator
        self.push_u32(len);
        self.payload.extend_from_slice(s.as_bytes());
        self.payload.push(0); // Null terminator
        // Pad to 4-byte boundary
        while self.payload.len() % 4 != 0 {
            self.payload.push(0);
        }
        self.update_size();
    }
    
    /// Add a new_id (object creation)
    pub fn push_new_id(&mut self, id: u32) {
        self.push_u32(id);
    }
    
    fn update_size(&mut self) {
        let size = 8 + self.payload.len() as u16;
        self.header.opcode_size = (size as u32) << 16 | (self.header.opcode_size & 0xFFFF);
    }
    
    /// Serialize to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(8 + self.payload.len());
        bytes.extend_from_slice(&self.header.object_id.to_ne_bytes());
        bytes.extend_from_slice(&self.header.opcode_size.to_ne_bytes());
        bytes.extend_from_slice(&self.payload);
        bytes
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SHARED MEMORY FORMATS
// ═══════════════════════════════════════════════════════════════════════════════

/// Pixel formats supported by wl_shm
#[repr(u32)]
// #[derive] — auto-generates trait implementations at compile time.
#[derive(Debug, Clone, Copy, PartialEq)]
// Enumeration — a type that can be one of several variants.
pub enum WlShmFormat {
    Argb8888 = 0,
    Xrgb8888 = 1,
    // Many more formats exist, these are the essentials
}

// ═══════════════════════════════════════════════════════════════════════════════
// SEAT CAPABILITIES
// ═══════════════════════════════════════════════════════════════════════════════

/// wl_seat capabilities (bitmask)
pub mod SeatCapability {
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const POINTER: u32 = 1;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const KEYBOARD: u32 = 2;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const TOUCH: u32 = 4;
}

// ═══════════════════════════════════════════════════════════════════════════════
// GLOBAL INTERFACES
// ═══════════════════════════════════════════════════════════════════════════════

/// Known global interfaces
#[derive(Debug, Clone)]
// Public structure — visible outside this module.
pub struct WlGlobal {
    pub name: u32,
    pub interface: String,
    pub version: u32,
}

/// Standard globals that TrustOS Wayland compositor advertises
pub fn get_globals() -> Vec<WlGlobal> {
    alloc::vec![
        WlGlobal { name: 1, interface: String::from("wl_compositor"), version: 5 },
        WlGlobal { name: 2, interface: String::from("wl_shm"), version: 1 },
        WlGlobal { name: 3, interface: String::from("wl_seat"), version: 8 },
        WlGlobal { name: 4, interface: String::from("wl_output"), version: 4 },
        WlGlobal { name: 5, interface: String::from("xdg_wm_base"), version: 5 },
    ]
}
