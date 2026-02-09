// TrustVideo Codec — .tv format
// Delta-encoded frames with RLE compression
//
// File layout:
//   [TvHeader]  (24 bytes)
//   [Frame 0]   keyframe — RLE compressed raw pixels
//   [Frame 1..] delta frames — XOR with previous + RLE compressed
//
// Each frame:
//   [u32 compressed_size]
//   [u8  frame_type]  0=keyframe, 1=delta
//   [RLE data...]
//
// RLE encoding:
//   [u8 count] [u32 value]  — repeat value count+1 times (max 256 repeats per run)

use alloc::vec::Vec;
use alloc::vec;

/// Magic: "TrVd" = 0x5476_5264
const TV_MAGIC: u32 = 0x5476_5264;
const FRAME_KEY: u8 = 0;
const FRAME_DELTA: u8 = 1;

/// TrustVideo file header (24 bytes)
#[repr(C)]
#[derive(Clone, Copy)]
pub struct TvHeader {
    pub magic: u32,
    pub version: u16,
    pub width: u16,
    pub height: u16,
    pub fps: u16,
    pub frame_count: u32,
    pub keyframe_interval: u16, // keyframe every N frames
    pub _reserved: [u8; 6],
}

impl TvHeader {
    pub fn new(width: u16, height: u16, fps: u16, frame_count: u32) -> Self {
        Self {
            magic: TV_MAGIC,
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

    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() < 24 { return None; }
        let magic = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        if magic != TV_MAGIC { return None; }
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

// ── RLE compression ──

/// RLE encode: [count-1, value_bytes...] runs
fn rle_encode(pixels: &[u32]) -> Vec<u8> {
    let mut out = Vec::new();
    if pixels.is_empty() { return out; }
    let mut i = 0;
    while i < pixels.len() {
        let val = pixels[i];
        let mut run: u8 = 0; // 0 means 1 pixel
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

/// RLE decode back to pixels
fn rle_decode(data: &[u8], pixel_count: usize) -> Vec<u32> {
    let mut pixels = Vec::with_capacity(pixel_count);
    let mut i = 0;
    while i + 4 < data.len() && pixels.len() < pixel_count {
        let run = data[i] as usize + 1;
        let val = u32::from_le_bytes([data[i + 1], data[i + 2], data[i + 3], data[i + 4]]);
        for _ in 0..run {
            if pixels.len() >= pixel_count { break; }
            pixels.push(val);
        }
        i += 5;
    }
    pixels
}

// ── Encoder ──

pub struct TvEncoder {
    pub header: TvHeader,
    prev_frame: Vec<u32>,
    frames_encoded: u32,
    pub data: Vec<u8>,
}

impl TvEncoder {
    pub fn new(width: u16, height: u16, fps: u16) -> Self {
        let npix = width as usize * height as usize;
        Self {
            header: TvHeader::new(width, height, fps, 0),
            prev_frame: vec![0u32; npix],
            frames_encoded: 0,
            data: Vec::new(),
        }
    }

    /// Add a frame (ARGB u32 pixel buffer, row-major)
    pub fn add_frame(&mut self, pixels: &[u32]) {
        let npix = self.header.width as usize * self.header.height as usize;
        let is_key = self.frames_encoded == 0
            || (self.header.keyframe_interval > 0
                && self.frames_encoded % self.header.keyframe_interval as u32 == 0);

        if is_key {
            // Keyframe: RLE encode raw pixels
            let compressed = rle_encode(&pixels[..npix]);
            let frame_size = 1 + compressed.len(); // type + data
            self.data.extend_from_slice(&(frame_size as u32).to_le_bytes());
            self.data.push(FRAME_KEY);
            self.data.extend_from_slice(&compressed);
            self.prev_frame[..npix].copy_from_slice(&pixels[..npix]);
        } else {
            // Delta frame: XOR with previous, then RLE
            let mut delta = vec![0u32; npix];
            for i in 0..npix {
                delta[i] = pixels[i] ^ self.prev_frame[i];
            }
            let compressed = rle_encode(&delta);
            let frame_size = 1 + compressed.len();
            self.data.extend_from_slice(&(frame_size as u32).to_le_bytes());
            self.data.push(FRAME_DELTA);
            self.data.extend_from_slice(&compressed);
            self.prev_frame[..npix].copy_from_slice(&pixels[..npix]);
        }
        self.frames_encoded += 1;
        self.header.frame_count = self.frames_encoded;
    }

    /// Finalize: produce complete .tv file bytes
    pub fn finalize(&self) -> Vec<u8> {
        let mut out = self.header.to_bytes();
        out.extend_from_slice(&self.data);
        out
    }
}

// ── Decoder ──

pub struct TvDecoder {
    pub header: TvHeader,
    data: Vec<u8>,
    offset: usize,
    current_frame: Vec<u32>,
    pub frames_decoded: u32,
}

impl TvDecoder {
    pub fn new(file_data: Vec<u8>) -> Option<Self> {
        let header = TvHeader::from_bytes(&file_data)?;
        let npix = header.width as usize * header.height as usize;
        Some(Self {
            header,
            data: file_data,
            offset: 24, // skip header
            current_frame: vec![0u32; npix],
            frames_decoded: 0,
        })
    }

    /// Decode next frame. Returns pixel buffer or None if finished.
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

        let frame_type = self.data[self.offset];
        let rle_data = &self.data[self.offset + 1..self.offset + frame_size];
        let npix = self.header.width as usize * self.header.height as usize;

        if frame_type == FRAME_KEY {
            let pixels = rle_decode(rle_data, npix);
            self.current_frame[..npix].copy_from_slice(&pixels[..npix.min(pixels.len())]);
        } else {
            // Delta: XOR decoded delta with current frame
            let delta = rle_decode(rle_data, npix);
            for i in 0..npix.min(delta.len()) {
                self.current_frame[i] ^= delta[i];
            }
        }

        self.offset += frame_size;
        self.frames_decoded += 1;
        Some(&self.current_frame)
    }

    /// Reset to beginning
    pub fn rewind(&mut self) {
        self.offset = 24;
        self.frames_decoded = 0;
        self.current_frame.fill(0);
    }

    /// Check if more frames available
    pub fn has_next(&self) -> bool {
        self.frames_decoded < self.header.frame_count && self.offset + 5 <= self.data.len()
    }
}
