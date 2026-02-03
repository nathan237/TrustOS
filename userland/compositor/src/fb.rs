//! Framebuffer management

pub struct Framebuffer {
    pub addr: u64,
    pub width: u32,
    pub height: u32,
    pub stride: u32,
}

impl Framebuffer {
    pub fn init() -> Self {
        Self {
            addr: 0,
            width: 1920,
            height: 1080,
            stride: 1920 * 4,
        }
    }
    
    pub fn put_pixel(&mut self, x: u32, y: u32, color: u32) {
        if x >= self.width || y >= self.height {
            return;
        }
        let offset = (y * self.stride + x * 4) as usize;
        unsafe {
            let ptr = (self.addr + offset as u64) as *mut u32;
            *ptr = color;
        }
    }
}
