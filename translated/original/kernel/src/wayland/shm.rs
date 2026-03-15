//! Wayland Shared Memory (wl_shm)
//!
//! Provides shared memory buffer support for software rendering.
//! Clients create memory pools and allocate buffers from them.

use alloc::vec::Vec;
use alloc::boxed::Box;

/// A shared memory pool
#[derive(Debug)]
pub struct ShmPool {
    /// Pool ID
    pub id: u32,
    
    /// Memory buffer
    pub data: Vec<u8>,
    
    /// Pool size in bytes
    pub size: usize,
    
    /// Buffers allocated from this pool
    pub buffers: Vec<ShmBuffer>,
}

impl ShmPool {
    pub fn new(id: u32, size: usize) -> Self {
        Self {
            id,
            data: alloc::vec![0u8; size],
            size,
            buffers: Vec::new(),
        }
    }
    
    /// Resize the pool
    pub fn resize(&mut self, new_size: usize) {
        if new_size > self.size {
            self.data.resize(new_size, 0);
            self.size = new_size;
        }
    }
    
    /// Create a buffer from this pool
    pub fn create_buffer(
        &mut self,
        buffer_id: u32,
        offset: usize,
        width: u32,
        height: u32,
        stride: u32,
        format: u32,
    ) -> Result<&ShmBuffer, &'static str> {
        // Validate offset and size
        let buffer_size = (stride * height) as usize;
        if offset + buffer_size > self.size {
            return Err("Buffer extends beyond pool");
        }
        
        let buffer = ShmBuffer {
            id: buffer_id,
            pool_id: self.id,
            offset,
            width,
            height,
            stride,
            format,
            busy: false,
        };
        
        self.buffers.push(buffer);
        Ok(self.buffers.last().unwrap())
    }
    
    /// Get pixel data for a buffer as ARGB8888
    pub fn get_buffer_pixels(&self, buffer_id: u32) -> Option<Vec<u32>> {
        let buffer = self.buffers.iter().find(|b| b.id == buffer_id)?;
        
        let mut pixels = Vec::with_capacity((buffer.width * buffer.height) as usize);
        
        for y in 0..buffer.height {
            for x in 0..buffer.width {
                let byte_offset = buffer.offset + (y * buffer.stride + x * 4) as usize;
                if byte_offset + 4 <= self.data.len() {
                    let b = self.data[byte_offset];
                    let g = self.data[byte_offset + 1];
                    let r = self.data[byte_offset + 2];
                    let a = self.data[byte_offset + 3];
                    pixels.push(u32::from_be_bytes([a, r, g, b]));
                } else {
                    pixels.push(0xFF000000); // Black if out of bounds
                }
            }
        }
        
        Some(pixels)
    }
    
    /// Write directly to pool memory (for testing)
    pub fn write(&mut self, offset: usize, data: &[u8]) {
        let end = (offset + data.len()).min(self.size);
        let len = end - offset;
        self.data[offset..end].copy_from_slice(&data[..len]);
    }
}

/// A buffer allocated from a shared memory pool
#[derive(Debug, Clone)]
pub struct ShmBuffer {
    /// Buffer ID
    pub id: u32,
    
    /// Pool this buffer belongs to
    pub pool_id: u32,
    
    /// Offset in pool
    pub offset: usize,
    
    /// Width in pixels
    pub width: u32,
    
    /// Height in pixels
    pub height: u32,
    
    /// Stride (bytes per row)
    pub stride: u32,
    
    /// Pixel format (WlShmFormat)
    pub format: u32,
    
    /// Is the buffer currently in use by compositor?
    pub busy: bool,
}

impl ShmBuffer {
    /// Calculate size in bytes
    pub fn size(&self) -> usize {
        (self.stride * self.height) as usize
    }
}

/// Shared memory manager
pub struct ShmManager {
    pools: Vec<ShmPool>,
    next_pool_id: u32,
    next_buffer_id: u32,
}

impl ShmManager {
    pub fn new() -> Self {
        Self {
            pools: Vec::new(),
            next_pool_id: 1,
            next_buffer_id: 1,
        }
    }
    
    pub fn create_pool(&mut self, size: usize) -> u32 {
        let id = self.next_pool_id;
        self.next_pool_id += 1;
        self.pools.push(ShmPool::new(id, size));
        id
    }
    
    pub fn destroy_pool(&mut self, id: u32) {
        self.pools.retain(|p| p.id != id);
    }
    
    pub fn get_pool(&self, id: u32) -> Option<&ShmPool> {
        self.pools.iter().find(|p| p.id == id)
    }
    
    pub fn get_pool_mut(&mut self, id: u32) -> Option<&mut ShmPool> {
        self.pools.iter_mut().find(|p| p.id == id)
    }
    
    pub fn create_buffer(
        &mut self,
        pool_id: u32,
        offset: usize,
        width: u32,
        height: u32,
        stride: u32,
        format: u32,
    ) -> Result<u32, &'static str> {
        let buffer_id = self.next_buffer_id;
        self.next_buffer_id += 1;
        
        let pool = self.pools.iter_mut()
            .find(|p| p.id == pool_id)
            .ok_or("Pool not found")?;
        
        pool.create_buffer(buffer_id, offset, width, height, stride, format)?;
        Ok(buffer_id)
    }
}

impl Default for ShmManager {
    fn default() -> Self {
        Self::new()
    }
}
