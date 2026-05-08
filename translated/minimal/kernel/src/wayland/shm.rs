




use alloc::vec::Vec;
use alloc::boxed::Box;


#[derive(Debug)]
pub struct ShmPool {
    
    pub id: u32,
    
    
    pub data: Vec<u8>,
    
    
    pub size: usize,
    
    
    pub buffers: Vec<Qh>,
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
    
    
    pub fn resize(&mut self, akf: usize) {
        if akf > self.size {
            self.data.resize(akf, 0);
            self.size = akf;
        }
    }
    
    
    pub fn create_buffer(
        &mut self,
        cum: u32,
        offset: usize,
        width: u32,
        height: u32,
        stride: u32,
        format: u32,
    ) -> Result<&Qh, &'static str> {
        
        let fkb = (stride * height) as usize;
        if offset + fkb > self.size {
            return Err("Buffer extends beyond pool");
        }
        
        let buffer = Qh {
            id: cum,
            pool_id: self.id,
            offset,
            width,
            height,
            stride,
            format,
            ehi: false,
        };
        
        self.buffers.push(buffer);
        
        Ok(self.buffers.last().unwrap_or_else(|| unreachable!()))
    }
    
    
    pub fn qhf(&self, cum: u32) -> Option<Vec<u32>> {
        let buffer = self.buffers.iter().find(|b| b.id == cum)?;
        
        let mut pixels = Vec::with_capacity((buffer.width * buffer.height) as usize);
        
        for y in 0..buffer.height {
            for x in 0..buffer.width {
                let uo = buffer.offset + (y * buffer.stride + x * 4) as usize;
                if uo + 4 <= self.data.len() {
                    let b = self.data[uo];
                    let g = self.data[uo + 1];
                    let r = self.data[uo + 2];
                    let a = self.data[uo + 3];
                    pixels.push(u32::from_be_bytes([a, r, g, b]));
                } else {
                    pixels.push(0xFF000000); 
                }
            }
        }
        
        Some(pixels)
    }
    
    
    pub fn write(&mut self, offset: usize, data: &[u8]) {
        let end = (offset + data.len()).min(self.size);
        let len = end - offset;
        self.data[offset..end].copy_from_slice(&data[..len]);
    }
}


#[derive(Debug, Clone)]
pub struct Qh {
    
    pub id: u32,
    
    
    pub pool_id: u32,
    
    
    pub offset: usize,
    
    
    pub width: u32,
    
    
    pub height: u32,
    
    
    pub stride: u32,
    
    
    pub format: u32,
    
    
    pub ehi: bool,
}

impl Qh {
    
    pub fn size(&self) -> usize {
        (self.stride * self.height) as usize
    }
}


pub struct Aev {
    pools: Vec<ShmPool>,
    next_pool_id: u32,
    next_buffer_id: u32,
}

impl Aev {
    pub fn new() -> Self {
        Self {
            pools: Vec::new(),
            next_pool_id: 1,
            next_buffer_id: 1,
        }
    }
    
    pub fn qbv(&mut self, size: usize) -> u32 {
        let id = self.next_pool_id;
        self.next_pool_id += 1;
        self.pools.push(ShmPool::new(id, size));
        id
    }
    
    pub fn qcs(&mut self, id: u32) {
        self.pools.retain(|aa| aa.id != id);
    }
    
    pub fn qic(&self, id: u32) -> Option<&ShmPool> {
        self.pools.iter().find(|aa| aa.id == id)
    }
    
    pub fn qie(&mut self, id: u32) -> Option<&mut ShmPool> {
        self.pools.iter_mut().find(|aa| aa.id == id)
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
        let cum = self.next_buffer_id;
        self.next_buffer_id += 1;
        
        let gnr = self.pools.iter_mut()
            .find(|aa| aa.id == pool_id)
            .ok_or("Pool not found")?;
        
        gnr.create_buffer(cum, offset, width, height, stride, format)?;
        Ok(cum)
    }
}

impl Default for Aev {
    fn default() -> Self {
        Self::new()
    }
}
