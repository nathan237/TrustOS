




use alloc::string::String;
use alloc::vec::Vec;


#[derive(Debug, Clone, Copy)]
pub struct Oj {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}


#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum BufferTransform {
    #[default]
    Normal = 0,
    Rotate90 = 1,
    Rotate180 = 2,
    Rotate270 = 3,
    Flipped = 4,
    FlippedRotate90 = 5,
    FlippedRotate180 = 6,
    FlippedRotate270 = 7,
}


#[derive(Debug, Clone)]
pub struct Surface {
    
    pub id: u32,
    
    
    pub x: i32,
    pub y: i32,
    
    
    pub width: u32,
    pub height: u32,
    
    
    pub buffer: Vec<u32>,
    
    
    pub pending_buffer: Option<Vec<u32>>,
    pub pending_width: u32,
    pub pending_height: u32,
    
    
    pub buffer_offset_x: i32,
    pub buffer_offset_y: i32,
    
    
    pub buffer_scale: i32,
    
    
    pub buffer_transform: BufferTransform,
    
    
    pub damage: Vec<Oj>,
    
    
    pub committed: bool,
    
    
    pub visible: bool,
    
    
    pub title: String,
    
    
    pub app_id: String,
    
    
    pub has_decorations: bool,
    
    
    pub is_toplevel: bool,
    
    
    pub parent: Option<u32>,
    
    
    pub opaque_region: Option<Oj>,
    
    
    pub input_region: Option<Oj>,
    
    
    pub frame_callback: Option<u32>,
    
    
    pub state: SurfaceState,
}


#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct SurfaceState {
    pub maximized: bool,
    pub fullscreen: bool,
    pub resizing: bool,
    pub activated: bool,
    pub tiled_left: bool,
    pub tiled_right: bool,
    pub tiled_top: bool,
    pub tiled_bottom: bool,
}

impl Surface {
    pub fn new(id: u32) -> Self {
        Self {
            id,
            x: 100,
            y: 100,
            width: 0,
            height: 0,
            buffer: Vec::new(),
            pending_buffer: None,
            pending_width: 0,
            pending_height: 0,
            buffer_offset_x: 0,
            buffer_offset_y: 0,
            buffer_scale: 1,
            buffer_transform: BufferTransform::Normal,
            damage: Vec::new(),
            committed: false,
            visible: true,
            title: String::new(),
            app_id: String::new(),
            has_decorations: true,
            is_toplevel: false,
            parent: None,
            opaque_region: None,
            input_region: None,
            frame_callback: None,
            state: SurfaceState::default(),
        }
    }
    
    
    pub fn attach(&mut self, buffer: Vec<u32>, width: u32, height: u32) {
        self.pending_buffer = Some(buffer);
        self.pending_width = width;
        self.pending_height = height;
    }
    
    
    pub fn damage(&mut self, x: i32, y: i32, width: i32, height: i32) {
        self.damage.push(Oj { x, y, width, height });
    }
    
    
    pub fn commit(&mut self) {
        
        if let Some(buffer) = self.pending_buffer.take() {
            self.buffer = buffer;
            self.width = self.pending_width;
            self.height = self.pending_height;
        }
        
        
        self.damage.clear();
        
        self.committed = true;
    }
    
    
    pub fn set_title(&mut self, title: &str) {
        self.title = String::from(title);
    }
    
    
    pub fn qvj(&mut self, app_id: &str) {
        self.app_id = String::from(app_id);
    }
    
    
    pub fn set_position(&mut self, x: i32, y: i32) {
        self.x = x;
        self.y = y;
    }
    
    
    pub fn contains(&self, p: i32, o: i32) -> bool {
        let crr = if self.has_decorations { 28 } else { 0 };
        let x1 = self.x;
        let y1 = self.y - crr;
        let x2 = self.x + self.width as i32;
        let y2 = self.y + self.height as i32;
        
        p >= x1 && p < x2 && o >= y1 && o < y2
    }
    
    
    pub fn in_title_bar(&self, p: i32, o: i32) -> bool {
        if !self.has_decorations {
            return false;
        }
        let crr = 28;
        p >= self.x 
            && p < self.x + self.width as i32 
            && o >= self.y - crr 
            && o < self.y
    }
    
    
    pub fn qbf(&self) -> (i32, i32, u32, u32) {
        (self.x, self.y, self.width, self.height)
    }
    
    
    pub fn make_toplevel(&mut self) {
        self.is_toplevel = true;
        self.has_decorations = true;
        self.state.activated = true;
    }
    
    
    pub fn qow(&mut self, screen_width: u32, screen_height: u32) {
        self.state.maximized = true;
        self.x = 0;
        self.y = 28; 
        
    }
    
    
    pub fn rbj(&mut self, amh: i32, boh: i32) {
        self.state.maximized = false;
        self.x = amh;
        self.y = boh;
    }
}


pub struct Afa {
    next_id: u32,
}

impl Afa {
    pub fn new() -> Self {
        Self { next_id: 1 }
    }
    
    pub fn create(&mut self) -> Surface {
        let id = self.next_id;
        self.next_id += 1;
        Surface::new(id)
    }
}

impl Default for Afa {
    fn default() -> Self {
        Self::new()
    }
}
