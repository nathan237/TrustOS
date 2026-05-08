















use alloc::vec::Vec;
use alloc::vec;


const BJY_: u32 = 0x5476_5264;
const AUO_: u8 = 0;
const BYR_: u8 = 1;


#[repr(C)]
#[derive(Clone, Copy)]
pub struct TvHeader {
    pub magic: u32,
    pub version: u16,
    pub width: u16,
    pub height: u16,
    pub fps: u16,
    pub frame_count: u32,
    pub keyframe_interval: u16, 
    pub _reserved: [u8; 6],
}

impl TvHeader {
    pub fn new(width: u16, height: u16, fps: u16, frame_count: u32) -> Self {
        Self {
            magic: BJY_,
            version: 1,
            width,
            height,
            fps,
            frame_count,
            keyframe_interval: 30,
            _reserved: [0; 6],
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(24);
        buf.extend_from_slice(&self.magic.to_le_bytes());
        buf.extend_from_slice(&self.version.to_le_bytes());
        buf.extend_from_slice(&self.width.to_le_bytes());
        buf.extend_from_slice(&self.height.to_le_bytes());
        buf.extend_from_slice(&self.fps.to_le_bytes());
        buf.extend_from_slice(&self.frame_count.to_le_bytes());
        buf.extend_from_slice(&self.keyframe_interval.to_le_bytes());
        buf.extend_from_slice(&self._reserved);
        buf
    }

    pub fn bsv(data: &[u8]) -> Option<Self> {
        if data.len() < 24 { return None; }
        let magic = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        if magic != BJY_ { return None; }
        Some(Self {
            magic,
            version: u16::from_le_bytes([data[4], data[5]]),
            width: u16::from_le_bytes([data[6], data[7]]),
            height: u16::from_le_bytes([data[8], data[9]]),
            fps: u16::from_le_bytes([data[10], data[11]]),
            frame_count: u32::from_le_bytes([data[12], data[13], data[14], data[15]]),
            keyframe_interval: u16::from_le_bytes([data[16], data[17]]),
            _reserved: [data[18], data[19], data[20], data[21], data[22], data[23]],
        })
    }
}




fn jbd(pixels: &[u32]) -> Vec<u8> {
    let mut out = Vec::new();
    if pixels.is_empty() { return out; }
    let mut i = 0;
    while i < pixels.len() {
        let val = pixels[i];
        let mut run: u8 = 0; 
        while i + (run as usize) + 1 < pixels.len()
            && pixels[i + (run as usize) + 1] == val
            && run < 255
        {
            run += 1;
        }
        out.push(run);
        out.extend_from_slice(&val.to_le_bytes());
        i += run as usize + 1;
    }
    out
}


fn jbc(data: &[u8], cod: usize) -> Vec<u32> {
    let mut pixels = Vec::with_capacity(cod);
    let mut i = 0;
    while i + 4 < data.len() && pixels.len() < cod {
        let run = data[i] as usize + 1;
        let val = u32::from_le_bytes([data[i + 1], data[i + 2], data[i + 3], data[i + 4]]);
        for _ in 0..run {
            if pixels.len() >= cod { break; }
            pixels.push(val);
        }
        i += 5;
    }
    pixels
}



pub struct TvEncoder {
    pub header: TvHeader,
    prev_frame: Vec<u32>,
    frames_encoded: u32,
    pub data: Vec<u8>,
}

impl TvEncoder {
    pub fn new(width: u16, height: u16, fps: u16) -> Self {
        let yz = width as usize * height as usize;
        Self {
            header: TvHeader::new(width, height, fps, 0),
            prev_frame: vec![0u32; yz],
            frames_encoded: 0,
            data: Vec::new(),
        }
    }

    
    pub fn add_frame(&mut self, pixels: &[u32]) {
        let yz = self.header.width as usize * self.header.height as usize;
        let msw = self.frames_encoded == 0
            || (self.header.keyframe_interval > 0
                && self.frames_encoded % self.header.keyframe_interval as u32 == 0);

        if msw {
            
            let qv = jbd(&pixels[..yz]);
            let frame_size = 1 + qv.len(); 
            self.data.extend_from_slice(&(frame_size as u32).to_le_bytes());
            self.data.push(AUO_);
            self.data.extend_from_slice(&qv);
            self.prev_frame[..yz].copy_from_slice(&pixels[..yz]);
        } else {
            
            let mut mk = vec![0u32; yz];
            for i in 0..yz {
                mk[i] = pixels[i] ^ self.prev_frame[i];
            }
            let qv = jbd(&mk);
            let frame_size = 1 + qv.len();
            self.data.extend_from_slice(&(frame_size as u32).to_le_bytes());
            self.data.push(BYR_);
            self.data.extend_from_slice(&qv);
            self.prev_frame[..yz].copy_from_slice(&pixels[..yz]);
        }
        self.frames_encoded += 1;
        self.header.frame_count = self.frames_encoded;
    }

    
    pub fn finalize(&self) -> Vec<u8> {
        let mut out = self.header.to_bytes();
        out.extend_from_slice(&self.data);
        out
    }
}



pub struct TvDecoder {
    pub header: TvHeader,
    data: Vec<u8>,
    offset: usize,
    current_frame: Vec<u32>,
    pub frames_decoded: u32,
}

impl TvDecoder {
    pub fn new(file_data: Vec<u8>) -> Option<Self> {
        let header = TvHeader::bsv(&file_data)?;
        let yz = header.width as usize * header.height as usize;
        Some(Self {
            header,
            data: file_data,
            offset: 24, 
            current_frame: vec![0u32; yz],
            frames_decoded: 0,
        })
    }

    
    pub fn next_frame(&mut self) -> Option<&[u32]> {
        if self.frames_decoded >= self.header.frame_count { return None; }
        if self.offset + 5 > self.data.len() { return None; }

        let frame_size = u32::from_le_bytes([
            self.data[self.offset],
            self.data[self.offset + 1],
            self.data[self.offset + 2],
            self.data[self.offset + 3],
        ]) as usize;
        self.offset += 4;

        if self.offset + frame_size > self.data.len() { return None; }

        let lyn = self.data[self.offset];
        let jbb = &self.data[self.offset + 1..self.offset + frame_size];
        let yz = self.header.width as usize * self.header.height as usize;

        if lyn == AUO_ {
            let pixels = jbc(jbb, yz);
            self.current_frame[..yz].copy_from_slice(&pixels[..yz.min(pixels.len())]);
        } else {
            
            let mk = jbc(jbb, yz);
            for i in 0..yz.min(mk.len()) {
                self.current_frame[i] ^= mk[i];
            }
        }

        self.offset += frame_size;
        self.frames_decoded += 1;
        Some(&self.current_frame)
    }

    
    pub fn rewind(&mut self) {
        self.offset = 24;
        self.frames_decoded = 0;
        self.current_frame.fill(0);
    }

    
    pub fn qko(&self) -> bool {
        self.frames_decoded < self.header.frame_count && self.offset + 5 <= self.data.len()
    }
}
