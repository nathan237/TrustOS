






use alloc::vec::Vec;
use alloc::vec;






pub const DHP_: usize = 2;   
pub const DHO_: usize = 4;   
pub const ABL_: usize = 16;    
pub const ABK_: usize = 32;    


const EME_: i32 = 64;



const AFD_: [u8; 64] = [
    255, 252, 248, 244, 240, 236, 231, 226,
    221, 216, 210, 204, 198, 192, 186, 179,
    172, 165, 158, 151, 144, 137, 130, 123,
    116, 109, 102, 96, 90, 84, 78, 72,
    67, 62, 57, 52, 48, 44, 40, 36,
    33, 30, 27, 24, 22, 20, 18, 16,
    14, 13, 12, 11, 10, 9, 8, 7,
    6, 5, 5, 4, 4, 3, 3, 2,  
];




const fn mcg() -> [u32; 256] {
    let mut bhu = [0xFF000000u32; 256];
    let mut i = 1u32;
    while i < 256 {
        let color = if i > 250 {
            
            let t = i - 250; 
            let r = 180 + t * 15; 
            let g = 255;
            let b = 220 + t * 7; 
            (0xFF << 24) | (r << 16) | (g << 8) | b
        } else if i > 235 {
            
            let t = i - 235; 
            let r = 60 + t * 8; 
            let g = 220 + t * 2; 
            let b = 120 + t * 6; 
            (0xFF << 24) | (r << 16) | (g << 8) | b
        } else if i > 200 {
            
            let t = i - 200; 
            let r = t; 
            let g = 180 + t * 2; 
            let b = 30 + t * 2; 
            (0xFF << 24) | (r << 16) | (g << 8) | b
        } else if i > 160 {
            
            let t = i - 160; 
            let g = 140 + t; 
            let r = t / 6; 
            let b = 10 + t / 3; 
            (0xFF << 24) | (r << 16) | (g << 8) | b
        } else if i > 120 {
            
            let t = i - 120; 
            let g = 100 + t; 
            let b = 8 + t / 3; 
            let r = t / 12;
            (0xFF << 24) | (r << 16) | (g << 8) | b
        } else if i > 80 {
            
            let t = i - 80; 
            let g = 60 + t; 
            let b = 4 + t / 5; 
            (0xFF << 24) | (g << 8) | b
        } else if i > 50 {
            
            let t = i - 50; 
            let g = 30 + t; 
            let b = 3 + t / 6; 
            (0xFF << 24) | (g << 8) | b
        } else if i > 25 {
            
            let t = i - 25; 
            let g = 12 + t; 
            let b = 2 + t / 5; 
            (0xFF << 24) | (g << 8) | b
        } else if i > 10 {
            
            let t = i - 10; 
            let g = 5 + t / 2; 
            let b = 1 + t / 8;
            (0xFF << 24) | (g << 8) | b
        } else {
            
            let g = 2 + i / 3;
            let b = 1 + i / 5;
            (0xFF << 24) | (g << 8) | b
        };
        bhu[i as usize] = color;
        i += 1;
    }
    bhu
}


static BPW_: [u32; 256] = mcg();


#[inline(always)]
pub(crate) fn ihb(intensity: u8) -> u32 {
    BPW_[intensity as usize]
}





pub(crate) const CIL_: [[u8; 6]; 64] = [
    
    [0b011110, 0b100001, 0b100101, 0b101001, 0b100001, 0b011110],
    
    [0b001100, 0b010100, 0b000100, 0b000100, 0b000100, 0b011111],
    
    [0b011110, 0b000001, 0b001110, 0b010000, 0b100000, 0b111111],
    
    [0b111110, 0b000001, 0b001110, 0b000001, 0b000001, 0b111110],
    
    [0b100010, 0b100010, 0b100010, 0b111111, 0b000010, 0b000010],
    
    [0b111111, 0b100000, 0b111110, 0b000001, 0b000001, 0b111110],
    
    [0b011110, 0b100000, 0b111110, 0b100001, 0b100001, 0b011110],
    
    [0b111111, 0b000001, 0b000010, 0b000100, 0b001000, 0b001000],
    
    [0b011110, 0b100001, 0b011110, 0b100001, 0b100001, 0b011110],
    
    [0b011110, 0b100001, 0b100001, 0b011111, 0b000001, 0b011110],
    
    [0b111111, 0b000010, 0b000100, 0b001000, 0b010000, 0b100000],
    
    [0b000010, 0b111110, 0b000010, 0b000100, 0b001000, 0b010000],
    
    [0b001100, 0b111111, 0b100001, 0b100001, 0b010010, 0b001100],
    
    [0b111111, 0b000100, 0b000100, 0b000100, 0b000100, 0b111111],
    
    [0b000100, 0b111111, 0b000100, 0b001010, 0b010001, 0b100000],
    
    [0b001000, 0b111111, 0b001001, 0b001010, 0b011100, 0b001000],
    
    [0b000100, 0b111111, 0b000100, 0b111111, 0b000100, 0b000100],
    
    [0b011110, 0b000010, 0b000100, 0b001000, 0b010000, 0b100000],
    
    [0b001000, 0b111111, 0b000100, 0b000010, 0b000001, 0b000000],
    
    [0b111111, 0b000001, 0b000001, 0b000001, 0b000001, 0b111111],
    
    [0b010010, 0b111111, 0b010010, 0b000100, 0b001000, 0b110000],
    
    [0b100000, 0b100100, 0b000010, 0b000001, 0b000010, 0b011100],
    
    [0b111111, 0b000010, 0b000100, 0b001010, 0b010001, 0b100000],
    
    [0b010000, 0b111111, 0b010010, 0b010100, 0b011000, 0b010000],
    
    [0b100010, 0b010010, 0b000100, 0b001000, 0b010000, 0b100000],
    
    [0b001100, 0b111111, 0b000100, 0b111111, 0b001000, 0b110000],
    
    [0b111111, 0b000100, 0b111111, 0b000100, 0b001000, 0b110000],
    
    [0b100010, 0b010010, 0b000100, 0b000100, 0b001000, 0b110000],
    
    [0b111111, 0b000100, 0b000100, 0b000100, 0b001000, 0b110000],  
    
    [0b100000, 0b111100, 0b100000, 0b100000, 0b010000, 0b001111],
    
    [0b000100, 0b111111, 0b000100, 0b001000, 0b010000, 0b100000],
    
    [0b111111, 0b000000, 0b000000, 0b000000, 0b000000, 0b111111],
    
    [0b000100, 0b001010, 0b010001, 0b010001, 0b001010, 0b000100],
    
    [0b000100, 0b001010, 0b010001, 0b100001, 0b111111, 0b000000],
    
    [0b111111, 0b100001, 0b010001, 0b001010, 0b000100, 0b000000],
    
    [0b000100, 0b001110, 0b010101, 0b000100, 0b000100, 0b000100],
    
    [0b000100, 0b000100, 0b000100, 0b010101, 0b001110, 0b000100],
    
    [0b111111, 0b000000, 0b111111, 0b000000, 0b111111, 0b000000],
    
    [0b010010, 0b010010, 0b010010, 0b010010, 0b010010, 0b010010],
    
    [0b000100, 0b000100, 0b111111, 0b000100, 0b000100, 0b000100],
    
    [0b100001, 0b010010, 0b001100, 0b001100, 0b010010, 0b100001],
    
    [0b011110, 0b100001, 0b100001, 0b100001, 0b100001, 0b011110],
    
    [0b111111, 0b100001, 0b100001, 0b100001, 0b100001, 0b111111],
    
    [0b011110, 0b100001, 0b100001, 0b000000, 0b000000, 0b000000],
    
    [0b000000, 0b000000, 0b000000, 0b100001, 0b100001, 0b011110],
    
    [0b000011, 0b001100, 0b010000, 0b010000, 0b001100, 0b000011],
    
    [0b110000, 0b001100, 0b000010, 0b000010, 0b001100, 0b110000],
    
    [0b000000, 0b011000, 0b100100, 0b000010, 0b000001, 0b000000],
    
    [0b011000, 0b100100, 0b011000, 0b000110, 0b001001, 0b000110],
    
    [0b111111, 0b100001, 0b101101, 0b101101, 0b100001, 0b111111],
    
    [0b011110, 0b100001, 0b100001, 0b100001, 0b010010, 0b101101],
    
    [0b100001, 0b010010, 0b001100, 0b001100, 0b000100, 0b000100],
    
    [0b111111, 0b100000, 0b010000, 0b010000, 0b100000, 0b111111],
    
    [0b111111, 0b010010, 0b010010, 0b010010, 0b010010, 0b010010],
    
    [0b011110, 0b100001, 0b111111, 0b100001, 0b100001, 0b011110],
    
    [0b000100, 0b011110, 0b100101, 0b100101, 0b011110, 0b000100],
    
    [0b000000, 0b000000, 0b001100, 0b001100, 0b000000, 0b000000],
    
    [0b000000, 0b001100, 0b001100, 0b000000, 0b001100, 0b001100],
    
    [0b000100, 0b010101, 0b001110, 0b001110, 0b010101, 0b000100],
    
    [0b010010, 0b111111, 0b010010, 0b010010, 0b111111, 0b010010],
    
    [0b110001, 0b110010, 0b000100, 0b001000, 0b010011, 0b100011],
    
    [0b001110, 0b001000, 0b001000, 0b001000, 0b001000, 0b001110],
    
    [0b011100, 0b000100, 0b000100, 0b000100, 0b000100, 0b011100],
    
    [0b111111, 0b111111, 0b111111, 0b111111, 0b111111, 0b111111],
];


const DRG_: usize = 64;


pub(crate) const BBI_: &[u8] = b"0123456789ABCDEF@#$%&*<>[]{}|/\\";







const EO_: usize = 4;

const DRH_: usize = 3;

const TB_: usize = EO_ / 2;


const WP_: usize = 15;
const WK_: usize = 50;


const DP_: usize = 6;


const AGQ_: usize = 512;




const PA_: [[u8; 3]; 64] = [
    [0b111, 0b101, 0b111], 
    [0b010, 0b010, 0b010], 
    [0b111, 0b010, 0b111], 
    [0b110, 0b011, 0b110], 
    [0b101, 0b111, 0b001], 
    [0b011, 0b110, 0b011], 
    [0b011, 0b111, 0b111], 
    [0b111, 0b001, 0b010], 
    [0b111, 0b111, 0b111], 
    [0b111, 0b111, 0b001], 
    [0b111, 0b010, 0b100], 
    [0b001, 0b111, 0b010], 
    [0b111, 0b101, 0b010], 
    [0b111, 0b100, 0b111], 
    [0b010, 0b111, 0b100], 
    [0b100, 0b111, 0b010], 
    [0b010, 0b111, 0b001], 
    [0b110, 0b001, 0b100], 
    [0b100, 0b110, 0b001], 
    [0b111, 0b001, 0b111], 
    [0b101, 0b111, 0b100], 
    [0b100, 0b010, 0b001], 
    [0b111, 0b010, 0b001], 
    [0b100, 0b111, 0b001], 
    [0b101, 0b001, 0b010], 
    [0b011, 0b111, 0b100], 
    [0b111, 0b011, 0b010], 
    [0b101, 0b010, 0b010], 
    [0b111, 0b010, 0b010], 
    [0b100, 0b110, 0b100], 
    [0b010, 0b111, 0b110], 
    [0b111, 0b000, 0b111], 
    [0b010, 0b101, 0b010], 
    [0b010, 0b101, 0b111], 
    [0b111, 0b101, 0b010], 
    [0b010, 0b111, 0b010], 
    [0b101, 0b010, 0b101], 
    [0b111, 0b000, 0b111], 
    [0b101, 0b101, 0b101], 
    [0b110, 0b110, 0b000], 
    [0b011, 0b011, 0b000], 
    [0b000, 0b110, 0b110], 
    [0b000, 0b011, 0b011], 
    [0b010, 0b000, 0b010], 
    [0b000, 0b010, 0b000], 
    [0b010, 0b101, 0b000], 
    [0b000, 0b101, 0b010], 
    [0b100, 0b010, 0b100], 
    [0b001, 0b010, 0b001], 
    [0b110, 0b010, 0b011], 
    [0b011, 0b010, 0b110], 
    [0b010, 0b100, 0b010], 
    [0b010, 0b001, 0b010], 
    [0b101, 0b000, 0b101], 
    [0b001, 0b011, 0b111], 
    [0b100, 0b110, 0b111], 
    [0b111, 0b011, 0b001], 
    [0b111, 0b110, 0b100], 
    [0b011, 0b100, 0b011], 
    [0b110, 0b001, 0b110], 
    [0b101, 0b010, 0b010], 
    [0b010, 0b010, 0b101], 
    [0b110, 0b011, 0b001], 
    [0b011, 0b110, 0b100], 
];


const OC_: usize = 64;


#[derive(Clone, Copy)]
struct RainDrop {
    
    y: i32,
    
    speed: u8,
    
    delay: u8,
    
    trail_len: u8,
    
    glyph_seed: u32,
    
    active: bool,
}

impl RainDrop {
    fn njc() -> Self {
        Self {
            y: -100,
            speed: 1,
            delay: 0,
            trail_len: WP_ as u8,
            glyph_seed: 0,
            active: false,
        }
    }
    
    
    fn tail_y(&self) -> i32 {
        self.y - self.trail_len as i32
    }
}






#[derive(Clone, Copy, PartialEq)]
pub enum ShapeOverlay {
    None,
    Cube,
    Sphere,
    Torus,
    DNA,
}


const WJ_: usize = 144;


const AGS_: usize = 500;  


#[derive(Clone, Copy)]
struct ShapeDrop {
    
    col: i32,
    
    row: i32,
    
    prev_col: i32,
    
    prev_row: i32,
    
    depth: f32,
    
    progress: f32,
    
    edge_idx: u8,
    
    trail_len: u8,
    
    speed: f32,
    
    glyph_seed: u32,
}



#[derive(Clone, Copy)]
struct Rn {
    
    lw: f32,
    
    nn: f32,
    
    vel_x: f32,
    
    vel_y: f32,
    
    trail_len: u8,
    
    active: bool,
    
    glyph_seed: u32,
    
    life: f32,
}

impl ShapeDrop {
    fn new() -> Self {
        Self {
            col: 0,
            row: 0,
            prev_col: 0,
            prev_row: 0,
            depth: 0.5,
            progress: 0.0,
            edge_idx: 0,
            trail_len: 6,
            speed: 0.012,
            glyph_seed: 0,
        }
    }
}






#[repr(C)]
struct Lc {
    buffer_ptr: *mut u32,
    buffer_len: usize,
    fb_width: usize,
    fb_height: usize,
    cell_size: usize,
    cell_rows: usize,
    
    drops_ptr: *const [RainDrop; DP_],
    col_depth_ptr: *const u8,
    num_cols: usize,
    
    cube_active: bool,
    cube_min_x: f32,
    cube_max_x: f32,
    cube_min_y: f32,
    cube_max_y: f32,
    top_edges: [(f32, f32, f32, f32); 4],
    left_edges: [(f32, f32, f32, f32); 4],
    right_edges: [(f32, f32, f32, f32); 4],
    pad_edges: [(f32, f32, f32, f32); 4],
}

unsafe impl Send for Lc {}
unsafe impl Sync for Lc {}



#[inline(always)]
unsafe fn ljb(buffer: *mut u32, fb_width: usize, fb_height: usize,
                              p: usize, o: usize, du: &[u8; 3], color: u32) {
    if o + 2 >= fb_height || p + 2 >= fb_width { return; }
    
    let kl = du[0];
    let aje = o * fb_width + p;
    if kl & 0b001 != 0 { *buffer.add(aje) = color; }
    if kl & 0b010 != 0 { *buffer.add(aje + 1) = color; }
    if kl & 0b100 != 0 { *buffer.add(aje + 2) = color; }
    
    let gf = du[1];
    let ajf = aje + fb_width;
    if gf & 0b001 != 0 { *buffer.add(ajf) = color; }
    if gf & 0b010 != 0 { *buffer.add(ajf + 1) = color; }
    if gf & 0b100 != 0 { *buffer.add(ajf + 2) = color; }
    
    let iq = du[2];
    let bes = ajf + fb_width;
    if iq & 0b001 != 0 { *buffer.add(bes) = color; }
    if iq & 0b010 != 0 { *buffer.add(bes + 1) = color; }
    if iq & 0b100 != 0 { *buffer.add(bes + 2) = color; }
}


#[inline]
unsafe fn qdq(buffer: *mut u32, fb_width: usize, fb_height: usize,
                              p: usize, o: usize, du: &[u8; 6], color: u32) {
    if o >= fb_height || p >= fb_width { return; }
    let buh = (fb_height - o).min(6);
    let agr = (fb_width - p).min(6);
    for row in 0..buh {
        let bits = du[row];
        if bits == 0 { continue; }
        let base = (o + row) * fb_width + p;
        if bits & 0b000001 != 0 && 0 < agr { *buffer.add(base) = color; }
        if bits & 0b000010 != 0 && 1 < agr { *buffer.add(base + 1) = color; }
        if bits & 0b000100 != 0 && 2 < agr { *buffer.add(base + 2) = color; }
        if bits & 0b001000 != 0 && 3 < agr { *buffer.add(base + 3) = color; }
        if bits & 0b010000 != 0 && 4 < agr { *buffer.add(base + 4) = color; }
        if bits & 0b100000 != 0 && 5 < agr { *buffer.add(base + 5) = color; }
    }
}


#[inline(always)]
fn ewv(p: f32, o: f32, edges: &[(f32, f32, f32, f32); 4]) -> bool {
    let mut pos = 0u8;
    let mut neg = 0u8;
    for &(ajq, qz, fh, hk) in edges.iter() {
        let cross = ajq * (o - hk) - qz * (p - fh);
        if cross > 0.0 { pos += 1; }
        else if cross < 0.0 { neg += 1; }
    }
    pos == 0 || neg == 0
}


fn ofg(start: usize, end: usize, data: *mut u8) {
    let aa = unsafe { &*(data as *const Lc) };
    let cell_size = aa.cell_size;
    
    for col in start..end {
        if col >= aa.num_cols { break; }
        
        let depth = unsafe { *aa.col_depth_ptr.add(col) };
        let dms = 100 + (depth as u32 * 155 / 255);
        
        let aqa = (col * cell_size + TB_) as f32;
        let kvh = aa.cube_active && aqa >= aa.cube_min_x && aqa <= aa.cube_max_x;
        
        let drops = unsafe { &*aa.drops_ptr.add(col) };
        
        for drop_idx in 0..DP_ {
            let drop = &drops[drop_idx];
            if !drop.active { continue; }
            
            let head_y = drop.y;
            let ecn = drop.trail_len as usize;
            
            for bab in 0..ecn {
                let aho = head_y - bab as i32;
                if aho < 0 || aho >= aa.cell_rows as i32 { continue; }
                
                let gdc = (bab * 63) / ecn.max(1);
                let din = AFD_[gdc.min(63)] as u32;
                let intensity = ((din * dms) / 255) as u8;
                
                if kvh {
                    let o = (aho as usize * cell_size + TB_) as f32;
                    if o >= aa.cube_min_y && o <= aa.cube_max_y {
                        if ewv(aqa, o, &aa.pad_edges) { continue; }
                        if ewv(aqa, o, &aa.top_edges)
                            || ewv(aqa, o, &aa.left_edges)
                            || ewv(aqa, o, &aa.right_edges) {
                            continue;
                        }
                    }
                }
                
                if intensity < 2 { continue; }
                
                let glyph_seed = drop.glyph_seed.wrapping_add(bab as u32 * 2654435761);
                let axi = (glyph_seed % OC_ as u32) as usize;
                let du = &PA_[axi];
                let color = ihb(intensity);
                
                let p = col * cell_size + 1;
                let o = aho as usize * cell_size + 1;
                unsafe {
                    ljb(aa.buffer_ptr, aa.fb_width, aa.fb_height,
                                       p, o, du, color);
                }
                
                
                if bab == 0 && intensity > 200 {
                    let hc = p + 1; 
                    let jh = o + 1;
                    let agv: [(i32, i32); 4] = [(-1,0),(1,0),(0,-1),(0,1)];
                    for &(fh, hk) in &agv {
                        let bu = hc as i32 + fh;
                        let ty = jh as i32 + hk;
                        if bu >= 0 && bu < aa.fb_width as i32 && ty >= 0 && ty < aa.fb_height as i32 {
                            let idx = ty as usize * aa.fb_width + bu as usize;
                            unsafe {
                                let e = *aa.buffer_ptr.add(idx);
                                let nr = (((e >> 16) & 0xFF) + 10).min(255);
                                let ayn = (((e >> 8) & 0xFF) + 48).min(255);
                                let ayj = ((e & 0xFF) + 32).min(255);
                                *aa.buffer_ptr.add(idx) = 0xFF000000 | (nr << 16) | (ayn << 8) | ayj;
                            }
                        }
                    }
                }
            }
        }
    }
}




pub struct BrailleMatrix {
    
    drops: Vec<[RainDrop; DP_]>,
    
    col_depth: Vec<u8>,
    
    rng: u32,
    
    frame: u32,
    
    num_cols: usize,
    
    num_rows: usize,
    
    shape_mode: ShapeOverlay,
    
    shape_time: f32,
    
    shape_drops: Vec<ShapeDrop>,
    
    shape_drop_count: usize,
    
    cube_flow_drops: Vec<Rn>,
}

impl BrailleMatrix {
    pub fn new() -> Self {
        use alloc::vec::Vec;
        
        let cols = 1280 / EO_; 
        let rows = 800 / EO_;  
        
        
        let mut drops: Vec<[RainDrop; DP_]> = Vec::with_capacity(AGQ_);
        let mut col_depth: Vec<u8> = vec![128u8; AGQ_];
        let mut rng = 0xDEADBEEFu32;
        
        
        for _ in 0..AGQ_ {
            drops.push([RainDrop::njc(); DP_]);
        }
        
        
        
        for col in 0..cols {
            rng = rng.wrapping_mul(1103515245).wrapping_add(12345);
            
            
            let pattern = ((col * 17 + 53) % 97) as i32 - 48; 
            let gpp = (rng % 100) as i32 - 50; 
            let jzy = 145i32 + pattern + gpp;
            col_depth[col] = jzy.clamp(30, 255) as u8;
        }
        
        
        for col in 0..cols {
            let depth = col_depth[col];
            
            let mut iqn: i32 = 0;
            
            for drop_idx in 0..DP_ {
                rng = rng.wrapping_mul(1103515245).wrapping_add(12345);
                
                
                let alh = depth as f32 / 255.0;
                let pxj = WK_ - WP_;
                let dui = (WP_ as f32 * (0.5 + alh * 0.5)) as usize;
                let ghb = (WK_ as f32 * (0.6 + alh * 0.4)) as usize;
                let trail_len = dui + (rng % (ghb - dui + 1) as u32) as usize;
                
                rng = rng.wrapping_mul(1103515245).wrapping_add(12345);
                
                
                
                let fya = ((1.0 - alh) * 2.0) as i32; 
                let dql = (2.0 + (1.0 - alh) * 6.0).max(1.0) as i32; 
                let gap = fya + (rng % dql as u32) as i32;
                let start_y = iqn - (rng % 8) as i32;  
                
                
                iqn = start_y - trail_len as i32 - gap;
                
                
                rng = rng.wrapping_mul(1103515245).wrapping_add(12345);
                let cqx = (1.0 + (1.0 - alh) * 1.5) as u8; 
                let cqy = (2.0 + (1.0 - alh) * 3.0) as u8; 
                let speed = cqx + (rng % cqy as u32) as u8;
                
                rng = rng.wrapping_mul(1103515245).wrapping_add(12345);
                
                drops[col][drop_idx] = RainDrop {
                    y: start_y,
                    speed,
                    delay: (rng % speed as u32) as u8,
                    trail_len: trail_len as u8,
                    glyph_seed: rng,
                    active: true,
                };
            }
        }
        
        
        let mut shape_drops: Vec<ShapeDrop> = Vec::with_capacity(WJ_);
        for _ in 0..WJ_ {
            shape_drops.push(ShapeDrop::new());
        }
        
        
        let mut cube_flow_drops: Vec<Rn> = Vec::with_capacity(AGS_);
        for _ in 0..AGS_ {
            cube_flow_drops.push(Rn {
                lw: 0.0,
                nn: 0.0,
                vel_x: 0.0,
                vel_y: 0.0,
                trail_len: 5,
                active: false,  
                glyph_seed: 0,
                life: 0.0,
            });
        }
        
        Self {
            drops,
            col_depth,
            rng,
            frame: 0,
            num_cols: cols,
            num_rows: rows,
            shape_mode: ShapeOverlay::None,  
            shape_time: 0.0,
            shape_drops,
            shape_drop_count: 0,
            cube_flow_drops,
        }
    }
    
    
    pub fn set_shape(&mut self, mode: ShapeOverlay) {
        self.shape_mode = mode;
        self.shape_time = 0.0;
        
        
        if mode == ShapeOverlay::Cube {
            for i in 0..AGS_ {
                self.cube_flow_drops[i].active = false;
                self.cube_flow_drops[i].life = 0.0;
            }
        }
        
        if mode == ShapeOverlay::None {
            self.shape_drop_count = 0;
            return;
        }
        
        
        let mut rng = 0x12345678u32;
        let lll = match mode {
            ShapeOverlay::Cube => 90,      
            ShapeOverlay::Sphere => 64,    
            ShapeOverlay::Torus => 80,     
            ShapeOverlay::DNA => 64,       
            ShapeOverlay::None => 0,
        };
        
        self.shape_drop_count = lll.min(WJ_);
        
        for i in 0..self.shape_drop_count {
            rng = rng.wrapping_mul(1103515245).wrapping_add(12345);
            self.shape_drops[i] = ShapeDrop {
                col: 0,
                row: 0,
                prev_col: 0,
                prev_row: 0,
                depth: 0.5,
                progress: (i as f32 / self.shape_drop_count as f32),
                edge_idx: (i % 12) as u8,
                trail_len: 5 + (rng % 4) as u8,
                speed: 0.006 + (rng % 50) as f32 / 5000.0,
                glyph_seed: rng,
            };
        }
    }
    
    
    pub fn qim(&self) -> ShapeOverlay {
        self.shape_mode
    }
    
    
    #[inline(always)]
    fn eu(x: f32) -> f32 { crate::math::eu(x) }
    
    
    #[inline(always)]
    fn hr(x: f32) -> f32 { crate::math::hr(x) }
    
    
    #[inline(always)]
    fn ra(x: f32) -> f32 { crate::math::ra(x) }
    
    
    #[inline(always)]
    fn bbp(x: f32) -> f32 {
        let i = x as i32;
        if (i as f32) > x { (i - 1) as f32 } else { i as f32 }
    }

    
    pub fn update(&mut self) {
        self.frame = self.frame.wrapping_add(1);
        
        let aye = (self.num_rows as i32) + WK_ as i32 + 10;
        
        for col in 0..self.num_cols {
            let depth = self.col_depth[col];
            let alh = depth as f32 / 255.0;
            
            
            let mut giv: [bool; DP_] = [false; DP_];
            let mut inv: i32 = 0;
            
            for drop_idx in 0..DP_ {
                let drop = &self.drops[col][drop_idx];
                if drop.active {
                    let ebg = drop.tail_y();
                    if ebg < inv {
                        inv = ebg;
                    }
                }
            }
            
            
            for drop_idx in 0..DP_ {
                let drop = &mut self.drops[col][drop_idx];
                
                if !drop.active {
                    continue;
                }
                
                
                drop.delay = drop.delay.wrapping_add(1);
                if drop.delay >= drop.speed {
                    drop.delay = 0;
                    drop.y += 1;
                    
                    
                    drop.glyph_seed = drop.glyph_seed.wrapping_mul(1103515245).wrapping_add(12345);
                }
                
                
                if drop.y > aye {
                    giv[drop_idx] = true;
                }
            }
            
            
            for drop_idx in 0..DP_ {
                if !giv[drop_idx] {
                    continue;
                }
                
                
                let mut fpt: i32 = 0;
                for other_idx in 0..DP_ {
                    if other_idx != drop_idx && !giv[other_idx] {
                        let drop = &self.drops[col][other_idx];
                        if drop.active {
                            let ebg = drop.tail_y();
                            if ebg < fpt {
                                fpt = ebg;
                            }
                        }
                    }
                }
                
                self.rng = self.rng.wrapping_mul(1103515245).wrapping_add(12345);
                    
                
                let dui = (WP_ as f32 * (0.5 + alh * 0.5)) as usize;
                let ghb = (WK_ as f32 * (0.6 + alh * 0.4)) as usize;
                let iqi = dui + (self.rng % (ghb - dui + 1).max(1) as u32) as usize;
                
                self.rng = self.rng.wrapping_mul(1103515245).wrapping_add(12345);
                    
                
                let fya = ((1.0 - alh) * 2.0) as i32; 
                let dql = (2.0 + (1.0 - alh) * 6.0).max(1.0) as i32; 
                let gap = fya + (self.rng % dql as u32) as i32;
                    
                
                let afk = fpt - iqi as i32 - gap - (self.rng % 5) as i32;
                
                self.rng = self.rng.wrapping_mul(1103515245).wrapping_add(12345);
                    
                
                let cqx = (1.0 + (1.0 - alh) * 1.5) as u8;
                let cqy = (2.0 + (1.0 - alh) * 3.0).max(1.0) as u8;
                let njs = cqx + (self.rng % cqy as u32) as u8;
                
                self.rng = self.rng.wrapping_mul(1103515245).wrapping_add(12345);
                
                let drop = &mut self.drops[col][drop_idx];
                drop.trail_len = iqi as u8;
                drop.y = afk;
                drop.speed = njs;
                drop.delay = 0;
                drop.glyph_seed = self.rng;
            }
        }
        
        
        if self.shape_mode != ShapeOverlay::None && self.shape_drop_count > 0 {
            self.shape_time += 0.016; 
            
            
            let center_x = (self.num_cols * EO_ / 2) as f32;  
            let center_y = (self.num_rows * EO_ / 2) as f32;  
            let scale = ((self.num_rows * EO_) as f32).min((self.num_cols * EO_) as f32) * 0.18;  
            
            let count = self.shape_drop_count.min(WJ_);
            for i in 0..count {
                let drop = &mut self.shape_drops[i];
                
                
                drop.progress += drop.speed;
                if drop.progress >= 1.0 {
                    drop.progress -= 1.0;
                    drop.edge_idx = (drop.edge_idx + 1) % 9;  
                    drop.glyph_seed = drop.glyph_seed.wrapping_mul(1103515245).wrapping_add(12345);
                }
                
                
                let (x, y, z) = match self.shape_mode {
                    ShapeOverlay::Cube => {
                        
                        let angle_y = 0.785398_f32;  
                        let angle_x = 0.523599_f32;  
                        let ahs = Self::hr(angle_y);
                        let air = Self::eu(angle_y);
                        let ahr = Self::hr(angle_x);
                        let aiq = Self::eu(angle_x);
                        
                        
                        let verts: [(f32, f32, f32); 8] = [
                            (-1.0, -1.0, -1.0), (1.0, -1.0, -1.0),
                            (1.0, 1.0, -1.0), (-1.0, 1.0, -1.0),
                            (-1.0, -1.0, 1.0), (1.0, -1.0, 1.0),
                            (1.0, 1.0, 1.0), (-1.0, 1.0, 1.0),
                        ];
                        
                        
                        
                        let edges: [(usize, usize); 9] = [
                            (0,1), (0,4), (0,3),   
                            (1,5),                  
                            (4,5), (4,7),           
                            (5,6),                  
                            (3,7),                  
                            (6,7),                  
                        ];
                        
                        let th = edges[drop.edge_idx as usize % 9];
                        let v1 = verts[th.0];
                        let v2 = verts[th.1];
                        
                        
                        let t = drop.progress;
                        let p = v1.0 + (v2.0 - v1.0) * t;
                        let o = v1.1 + (v2.1 - v1.1) * t;
                        let aos = v1.2 + (v2.2 - v1.2) * t;
                        
                        
                        let bvo = p * ahs - aos * air;
                        let auw = p * air + aos * ahs;
                        
                        
                        let apa = o * ahr - auw * aiq;
                        let auy = o * aiq + auw * ahr;
                        
                        
                        let cam_dist = 5.0;
                        let proj_z = auy + cam_dist;
                        let vq = cam_dist / proj_z.max(0.5);
                        
                        
                        let lw = center_x + bvo * scale * vq;
                        let nn = center_y + apa * scale * vq;
                        
                        
                        let ldh = (auy + 2.0) / 4.0;  
                        
                        (lw as i32, nn as i32, ldh)
                    },
                    ShapeOverlay::Sphere => {
                        
                        let aij = (i as f32 / self.shape_drop_count as f32) * 3.14159 * 2.0;
                        let acz = drop.progress * 3.14159;
                        let cc = self.shape_time * 0.5;
                        
                        let x = Self::eu(acz) * Self::hr(aij + cc);
                        let y = Self::eu(acz) * Self::eu(aij + cc);
                        let z = Self::hr(acz);
                        
                        let vq = 2.0 / (3.0 + z * 0.5);
                        let depth = (z + 1.0) / 2.0;
                        ((center_x + x * scale * vq) as i32,
                         (center_y + y * scale * vq * 0.7) as i32, depth)
                    },
                    ShapeOverlay::Torus => {
                        
                        let iy = drop.progress * 6.28318;
                        let v = (i as f32 / self.shape_drop_count as f32) * 6.28318;
                        let cc = self.shape_time * 0.4;
                        
                        let uh = 1.5;
                        let ju = 0.5;
                        let x = (uh + ju * Self::hr(v)) * Self::hr(iy + cc);
                        let y = (uh + ju * Self::hr(v)) * Self::eu(iy + cc);
                        let z = ju * Self::eu(v);
                        
                        let vq = 1.5 / (2.5 + z * 0.3);
                        let depth = (z + 0.5) / 1.0;
                        ((center_x + x * scale * 0.6 * vq) as i32,
                         (center_y + y * scale * 0.4 * vq) as i32, depth)
                    },
                    ShapeOverlay::DNA => {
                        
                        let t = drop.progress * 10.0 + (i as f32 * 0.1);
                        let cc = self.shape_time * 0.6;
                        let ien = if i % 2 == 0 { 1.0 } else { -1.0 };
                        
                        let x = Self::hr(t + cc) * ien;
                        let y = (t % 6.28318) / 3.14159 - 1.0;
                        let z = Self::eu(t + cc) * ien;
                        
                        let vq = 2.0 / (3.0 + z * 0.5);
                        let depth = (z + 1.0) / 2.0;
                        ((center_x + x * scale * 0.5 * vq) as i32,
                         (center_y + y * scale * 0.8) as i32, depth)
                    },
                    ShapeOverlay::None => (0, 0, 0.5),
                };
                
                
                drop.prev_col = drop.col;
                drop.prev_row = drop.row;
                drop.col = x;
                drop.row = y;
                drop.depth = z.clamp(0.0, 1.0);
            }
        }
        
        
        
    }
    
    
    
    pub fn render(&self, buffer: &mut [u32], fb_width: usize, fb_height: usize) {
        
        let bg_color = 0xFF010203u32;
        #[cfg(target_arch = "x86_64")]
        unsafe {
            crate::graphics::simd::adq(buffer.as_mut_ptr(), buffer.len(), bg_color);
        }
        #[cfg(not(target_arch = "x86_64"))]
        buffer.fill(bg_color);
        
        let cell_size = EO_;
        let khw = fb_width / cell_size;
        let cell_rows = fb_height / cell_size;
        
        
        let cube_active = self.shape_mode == ShapeOverlay::Cube;
        let center_x = (fb_width / 2) as f32;
        let center_y = (fb_height / 2) as f32;
        let scale = (fb_height as f32).min(fb_width as f32) * 0.18;
        
        
        let angle_y = 0.785398_f32; 
        let angle_x = 0.523599_f32; 
        let ahs = Self::hr(angle_y);
        let air = Self::eu(angle_y);
        let ahr = Self::hr(angle_x);
        let aiq = Self::eu(angle_x);
        
        
        let exd = |x3d: f32, z3d: f32| -> (f32, f32) {
            let cfp = -1.0; 
            let bvo = x3d * ahs - z3d * air;
            let auw = x3d * air + z3d * ahs;
            let apa = cfp * ahr - auw * aiq;
            let auy = cfp * aiq + auw * ahr;
            let cam_dist = 5.0;
            let proj_z = auy + cam_dist;
            let vq = cam_dist / proj_z.max(1.0);
            (center_x + bvo * scale * vq, center_y + apa * scale * vq)
        };
        
        
        let (top_back_x, top_back_y) = exd(0.0, -1.0);
        let (top_left_x, top_left_y) = exd(-1.0, 0.0);
        let (top_right_x, top_right_y) = exd(1.0, 0.0);
        let (top_front_x, top_front_y) = exd(0.0, 1.0);
        
        
        
        let coj = |x3d: f32, cfp: f32, z3d: f32| -> (f32, f32) {
            let bvo = x3d * ahs - z3d * air;
            let auw = x3d * air + z3d * ahs;
            let apa = cfp * ahr - auw * aiq;
            let auy = cfp * aiq + auw * ahr;
            let cam_dist = 5.0;
            let proj_z = auy + cam_dist;
            let iul = cam_dist / proj_z.max(1.0);
            (center_x + bvo * scale * iul, center_y + apa * scale * iul)
        };
        
        
        
        let ja = if cube_active { coj(-1.0, -1.0, -1.0) } else { (0.0, 0.0) };
        let brl = if cube_active { coj( 1.0, -1.0, -1.0) } else { (0.0, 0.0) };
        let dmc = if cube_active { coj( 1.0,  1.0, -1.0) } else { (0.0, 0.0) };
        let cwf = if cube_active { coj(-1.0,  1.0, -1.0) } else { (0.0, 0.0) };
        let brm = if cube_active { coj(-1.0, -1.0,  1.0) } else { (0.0, 0.0) };
        let dmd = if cube_active { coj(-1.0,  1.0,  1.0) } else { (0.0, 0.0) };
        
        
        
        let byl = if cube_active { coj( 1.0, -1.0,  1.0) } else { (0.0, 0.0) };
        let gzl = [ja, brl, byl, brm];
        
        let gfg = [ja, brm, dmd, cwf];
        
        let grl = [ja, brl, dmc, cwf];
        
        
        let cube_min_x = if cube_active { ja.0.min(brl.0).min(dmc.0).min(cwf.0).min(brm.0).min(dmd.0).min(byl.0) - 2.0 } else { 0.0 };
        let cube_max_x = if cube_active { ja.0.max(brl.0).max(dmc.0).max(cwf.0).max(brm.0).max(dmd.0).max(byl.0) + 2.0 } else { 0.0 };
        let cube_min_y = if cube_active { ja.1.min(brl.1).min(dmc.1).min(cwf.1).min(brm.1).min(dmd.1).min(byl.1) - 20.0 } else { 0.0 };
        let cube_max_y = if cube_active { ja.1.max(brl.1).max(dmc.1).max(cwf.1).max(brm.1).max(dmd.1).max(byl.1) + 2.0 } else { 0.0 };
        
        
        let lek = if cube_active { (top_right_x - top_left_x) / 2.0 } else { 1.0 };
        let lej = if cube_active { (top_front_y - top_back_y) / 2.0 } else { 1.0 };
        let qcx = (top_left_x + top_right_x) / 2.0;
        let qcy = (top_back_y + top_front_y) / 2.0;
        let qls = 1.0 / lek.max(1.0);
        let qlr = 1.0 / lej.max(1.0);
        
        
        
        let exj = |q: &[(f32, f32); 4]| -> [(f32, f32, f32, f32); 4] {
            let mut edges = [(0.0f32, 0.0f32, 0.0f32, 0.0f32); 4];
            for i in 0..4 {
                let ay = (i + 1) % 4;
                edges[i] = (q[ay].0 - q[i].0, q[ay].1 - q[i].1, q[i].0, q[i].1);
            }
            edges
        };
        let top_edges = if cube_active { exj(&gzl) } else { [(0.0,0.0,0.0,0.0); 4] };
        let left_edges = if cube_active { exj(&gfg) } else { [(0.0,0.0,0.0,0.0); 4] };
        let right_edges = if cube_active { exj(&grl) } else { [(0.0,0.0,0.0,0.0); 4] };
        
        
        
        let pad = 16.0_f32;
        let ple = (ja.0 + brl.0 + byl.0 + brm.0) / 4.0;
        let plf = (ja.1 + brl.1 + byl.1 + brm.1) / 4.0;
        let wd = |aa: (f32, f32)| -> (f32, f32) {
            let dx = aa.0 - ple;
            let ad = aa.1 - plf;
            (aa.0 + dx * 0.12 + 0.0, aa.1 + ad * 0.12 - pad * 0.5)
        };
        let npg = if cube_active {
            [wd(ja), wd(brl), wd(byl), wd(brm)]
        } else {
            [(0.0,0.0); 4]
        };
        let pad_edges = if cube_active { exj(&npg) } else { [(0.0,0.0,0.0,0.0); 4] };
        
        #[inline(always)]
        fn qqq(p: f32, o: f32, edges: &[(f32, f32, f32, f32); 4]) -> bool {
            let mut pos = 0u8;
            let mut neg = 0u8;
            for &(ajq, qz, fh, hk) in edges.iter() {
                let cross = ajq * (o - hk) - qz * (p - fh);
                if cross > 0.0 { pos += 1; } 
                else if cross < 0.0 { neg += 1; }
            }
            pos == 0 || neg == 0
        }
        
        
        let pls = khw.min(self.num_cols);
        let params = Lc {
            buffer_ptr: buffer.as_mut_ptr(),
            buffer_len: buffer.len(),
            fb_width,
            fb_height,
            cell_size,
            cell_rows,
            drops_ptr: self.drops.as_ptr(),
            col_depth_ptr: self.col_depth.as_ptr(),
            num_cols: self.num_cols,
            cube_active,
            cube_min_x,
            cube_max_x,
            cube_min_y,
            cube_max_y,
            top_edges,
            left_edges,
            right_edges,
            pad_edges,
        };
        
        crate::cpu::smp::bcz(
            pls,
            ofg,
            &params as *const Lc as *mut u8,
        );
        
        
        
    }
    
    
    
    pub fn render_entity_layer(&self, buffer: &mut [u32], fb_width: usize, fb_height: usize) {
        if self.shape_mode == ShapeOverlay::None {
            return;
        }
        
        let center_x = (fb_width / 2) as f32;
        let center_y = (fb_height / 2) as f32;
        let scale = (fb_height as f32).min(fb_width as f32) * 0.18;  
        
        
        let time = self.shape_time;
        
        match self.shape_mode {
            ShapeOverlay::Cube => {
                
            },
            ShapeOverlay::Sphere => {
                
                let evl = 24;
                
                
                for i in 0..6 {
                    let aij = (i as f32 / 6.0) * 3.14159;
                    let nwz = 0xFF00AA44;  
                    
                    let mut amh = 0i32;
                    let mut boh = 0i32;
                    let mut first = true;
                    
                    for ay in 0..=evl {
                        let acz = (ay as f32 / evl as f32) * 3.14159 * 2.0;
                        
                        let x = Self::eu(acz) * Self::hr(aij);
                        let z = Self::eu(acz) * Self::eu(aij);
                        let y = Self::hr(acz);
                        
                        
                        let eyy = time * 0.1;
                        let da = x * Self::hr(eyy) - z * Self::eu(eyy);
                        let qp = x * Self::eu(eyy) + z * Self::hr(eyy);
                        
                        let vq = 3.0 / (4.0 + qp);
                        let p = (center_x + da * scale * vq) as i32;
                        let o = (center_y + y * scale * vq * 0.8) as i32;
                        
                        if !first {
                            let depth = (qp + 1.0) / 2.0;
                            let color = if depth > 0.5 { 0xFFCCFFCC } else { nwz };  
                            self.draw_line_thick(buffer, fb_width, fb_height, 
                                               amh, boh, p, o, color, 2);
                        }
                        amh = p;
                        boh = o;
                        first = false;
                    }
                }
                
                
                for i in 1..4 {
                    let cfq = -0.75 + (i as f32 * 0.5);
                    let radius = (1.0 - cfq * cfq).max(0.0);
                    let radius = {
                        let mut x = radius;
                        let mut y = radius * 0.5;
                        for _ in 0..4 { y = (y + x / y) * 0.5; }
                        y
                    };
                    
                    let mut amh = 0i32;
                    let mut boh = 0i32;
                    let mut first = true;
                    
                    for ay in 0..=evl {
                        let cc = (ay as f32 / evl as f32) * 3.14159 * 2.0 + time * 0.1;
                        let x = Self::hr(cc) * radius;
                        let z = Self::eu(cc) * radius;
                        
                        let vq = 3.0 / (4.0 + z);
                        let p = (center_x + x * scale * vq) as i32;
                        let o = (center_y + cfq * scale * vq * 0.8) as i32;
                        
                        if !first {
                            let depth = (z + 1.0) / 2.0;
                            let color = if depth > 0.5 { 0xFFCCFFCC } else { 0xFF00AA44 };  
                            self.draw_line_thick(buffer, fb_width, fb_height, 
                                               amh, boh, p, o, color, 2);
                        }
                        amh = p;
                        boh = o;
                        first = false;
                    }
                }
            },
            ShapeOverlay::Torus => {
                
                let bcm = 0.7;
                let aro = 0.3;
                let segments = 16;
                
                
                for i in 0..8 {
                    let iy = (i as f32 / 8.0) * 3.14159 * 2.0;
                    let cvz = Self::hr(iy);
                    let oyg = Self::eu(iy);
                    
                    let mut amh = 0i32;
                    let mut boh = 0i32;
                    let mut first = true;
                    
                    for ay in 0..=segments {
                        let v = (ay as f32 / segments as f32) * 3.14159 * 2.0;
                        let cwe = Self::hr(v);
                        let amx = Self::eu(v);
                        
                        let x = (bcm + aro * cwe) * cvz;
                        let y = aro * amx;
                        let z = (bcm + aro * cwe) * oyg;
                        
                        
                        let biy = time * 0.2;
                        let da = x * Self::hr(biy) - z * Self::eu(biy);
                        let qp = x * Self::eu(biy) + z * Self::hr(biy);
                        
                        let vq = 3.0 / (4.0 + qp);
                        let p = (center_x + da * scale * vq) as i32;
                        let o = (center_y + y * scale * vq) as i32;
                        
                        if !first {
                            let depth = (qp + 1.0) / 2.0;
                            let color = if depth > 0.5 { 0xFFCCFFCC } else { 0xFF00AA44 };  
                            self.draw_line_thick(buffer, fb_width, fb_height, 
                                               amh, boh, p, o, color, 2);
                        }
                        amh = p;
                        boh = o;
                        first = false;
                    }
                }
            },
            ShapeOverlay::DNA => {
                
                let ieo = scale * 1.5;
                let ckn = scale * 0.3;
                let segments = 40;
                
                let mut iwh = 0i32;
                let mut iwj = 0i32;
                let mut iwi = 0i32;
                let mut iwk = 0i32;
                let mut first = true;
                
                for i in 0..=segments {
                    let t = i as f32 / segments as f32;
                    let y = center_y - ieo / 2.0 + ieo * t;
                    let cc = t * 3.14159 * 4.0 + time * 0.5;
                    
                    
                    let x1 = center_x + Self::hr(cc) * ckn;
                    let x2 = center_x + Self::hr(cc + 3.14159) * ckn;
                    let po = Self::eu(cc);
                    let qt = Self::eu(cc + 3.14159);
                    
                    let ddb = x1 as i32;
                    let bvc = y as i32;
                    let ddc = x2 as i32;
                    let bvd = y as i32;
                    
                    if !first {
                        let agh = if po > 0.0 { 0xFFCCFFCC } else { 0xFF00AA44 };  
                        let ale = if qt > 0.0 { 0xFFCCFFCC } else { 0xFF00AA44 };  
                        self.draw_line_thick(buffer, fb_width, fb_height, 
                                           iwh, iwj, ddb, bvc, agh, 2);
                        self.draw_line_thick(buffer, fb_width, fb_height, 
                                           iwi, iwk, ddc, bvd, ale, 2);
                        
                        
                        if i % 4 == 0 {
                            self.draw_line_thick(buffer, fb_width, fb_height, 
                                               ddb, bvc, ddc, bvd, 0xFF44FF44, 1);
                        }
                    }
                    iwh = ddb;
                    iwj = bvc;
                    iwi = ddc;
                    iwk = bvd;
                    first = false;
                }
            },
            ShapeOverlay::None => {},
        }
    }
    
    
    
    
    
    pub fn render_cube_flow_layer(&self, buffer: &mut [u32], fb_width: usize, fb_height: usize) {
        if self.shape_mode != ShapeOverlay::Cube {
            return;
        }
        
        let cell_size = EO_;
        let screen_width = (self.num_cols * cell_size) as f32;
        let screen_height = (self.num_rows * cell_size) as f32;
        let center_x = screen_width / 2.0;
        let center_y = screen_height / 2.0;
        let scale = screen_height.min(screen_width) * 0.18;
        
        let angle_y = 0.785398_f32;
        let angle_x = 0.523599_f32;
        let ahs = Self::hr(angle_y);
        let air = Self::eu(angle_y);
        let ahr = Self::hr(angle_x);
        let aiq = Self::eu(angle_x);
        let cam_dist = 5.0_f32;
        
        let project = |x3d: f32, cfp: f32, z3d: f32| -> (f32, f32) {
            let bvo = x3d * ahs - z3d * air;
            let auw = x3d * air + z3d * ahs;
            let apa = cfp * ahr - auw * aiq;
            let auy = cfp * aiq + auw * ahr;
            let proj_z = auy + cam_dist;
            let vq = cam_dist / proj_z.max(1.0);
            (center_x + bvo * scale * vq, center_y + apa * scale * vq)
        };
        
        
        let abl = project(-1.0, -1.0, -1.0); 
        let ll = project( 1.0, -1.0, -1.0); 
        let np = project( 1.0, -1.0,  1.0); 
        let acw = project(-1.0, -1.0,  1.0); 
        
        
        let alv = project(-1.0,  1.0,  1.0);  
        let btw = project(-1.0,  1.0, -1.0);  
        let ju = project( 1.0,  1.0, -1.0);  
        
        let gzl = [abl, ll, np, acw];
        let gfg = [abl, acw, alv, btw];
        
        let grl = [abl, ll, ju, btw];
        
        
        
        
        let jno = ll.0 - abl.0;
        let jnp = ll.1 - abl.1;
        let jnq = acw.0 - abl.0;
        let jnr = acw.1 - abl.1;
        let jnk = jno * jnr - jnp * jnq;
        
        
        let dhk = [abl, ll, np, acw, alv, btw, ju];
        let mut fim = dhk[0].0;
        let mut fik = dhk[0].0;
        let mut fin = dhk[0].1;
        let mut fil = dhk[0].1;
        for aa in &dhk[1..] {
            if aa.0 < fim { fim = aa.0; }
            if aa.0 > fik { fik = aa.0; }
            if aa.1 < fin { fin = aa.1; }
            if aa.1 > fil { fil = aa.1; }
        }
        
        let khy = ((fim / cell_size as f32) as i32).max(0) as usize;
        let khz = ((fik / cell_size as f32) as i32 + 1).min(self.num_cols as i32) as usize;
        let kia = ((fin / cell_size as f32) as i32).max(0) as usize;
        let kib = ((fil / cell_size as f32) as i32 + 1).min(self.num_rows as i32) as usize;
        
        let gno = |p: f32, o: f32, q: &[(f32, f32); 4]| -> bool {
            let mut pos = 0i32;
            let mut neg = 0i32;
            for i in 0..4 {
                let ay = (i + 1) % 4;
                let cross = (q[ay].0 - q[i].0) * (o - q[i].1) - (q[ay].1 - q[i].1) * (p - q[i].0);
                if cross > 0.0 { pos += 1; } 
                else if cross < 0.0 { neg += 1; }
            }
            pos == 0 || neg == 0
        };
        
        let time = self.shape_time;
        let irn = 8.0_f32;   
        let ijg = 0.35_f32; 
        
        for u in kia..kib {
            for cx in khy..khz {
                let p = (cx * cell_size + TB_) as f32;
                let o = (u * cell_size + TB_) as f32;
                
                
                if jnk.abs() > 0.01 && gno(p, o, &gzl) {
                    
                    let htf = p - abl.0;
                    let htg = o - abl.1;
                    let ihh = 1.0 / jnk;
                    let iy = (htf * jnr - htg * jnq) * ihh;
                    let v = (jno * htg - jnp * htf) * ihh;
                    
                    
                    let hai = iy * irn;
                    let hbc = v * irn;
                    let ppd = hai - Self::bbp(hai);
                    let pqu = hbc - Self::bbp(hbc);
                    
                    let gkz = ppd < ijg; 
                    let gla = pqu < ijg; 
                    
                    if gkz || gla {
                        let ppe = Self::bbp(hai) as i32;
                        let pqw = Self::bbp(hbc) as i32;
                        
                        
                        let scroll_speed = 2.5;
                        let mut brightness: f32 = 0.0;
                        
                        if gkz {
                            
                            let seed = (pqw as u32).wrapping_mul(2654435761);
                            let phase = (seed % 100) as f32 * 0.04;
                            let su = time * scroll_speed + phase;
                            let wr = 0.4_f32;  
                            let zd = 0.7 + (seed % 3) as f32 * 0.15;
                            let d = iy - su;
                            let aqj = d - Self::bbp(d / zd) * zd;
                            if aqj >= 0.0 && aqj < wr {
                                brightness = (1.0 - aqj / wr).max(0.0);
                            }
                        }
                        
                        if gla {
                            let seed = (ppe as u32).wrapping_mul(340573321);
                            let phase = (seed % 100) as f32 * 0.04;
                            let su = time * scroll_speed * 0.9 + phase;
                            let wr = 0.4_f32;
                            let zd = 0.7 + (seed % 3) as f32 * 0.15;
                            let d = v - su;
                            let aqj = d - Self::bbp(d / zd) * zd;
                            if aqj >= 0.0 && aqj < wr {
                                let iq = (1.0 - aqj / wr).max(0.0);
                                if iq > brightness { brightness = iq; }
                            }
                        }
                        
                        if brightness < 0.08 { brightness = 0.08; }
                        
                        
                        let dkl = (cx as u32).wrapping_mul(2654435761)
                            .wrapping_add((u as u32).wrapping_mul(340573321));
                        let anim_frame = (time * 8.0) as u32;
                        let axi = ((dkl.wrapping_add(anim_frame)) % OC_ as u32) as usize;
                        let du = &PA_[axi];
                        
                        
                        let color = if gkz && gla {
                            
                            let w = (180.0 + brightness * 75.0) as u8;
                            0xFF000000 | ((w as u32) << 16) | ((w as u32) << 8) | (w as u32)
                        } else if brightness > 0.85 {
                            
                            let g = (brightness * 255.0) as u8;
                            0xFF000000 | 0x00300000 | ((g as u32) << 8) | 0x30
                        } else {
                            
                            let g = (30.0 + brightness * 220.0) as u8;
                            0xFF000000 | ((g as u32) << 8) | 0x06
                        };
                        
                        self.draw_glyph_3x3(buffer, fb_width, cx * cell_size + 1, u * cell_size + 1, du, color);
                    }
                    continue;
                }
                
                
                if gno(p, o, &gfg) {
                    let aqa = 10.0_f32;
                    let chk = p / aqa;
                    let fnp = chk - Self::bbp(chk);
                    
                    if fnp < 0.4 {
                        let fnq = Self::bbp(chk) as i32;
                        let seed = (fnq as u32).wrapping_mul(2654435761);
                        let scroll_speed = 2.5 + (seed % 6) as f32 * 0.3;
                        let phase = (seed % 100) as f32 * 0.05;
                        let su = time * scroll_speed + phase;
                        let pos = o / aqa;
                        let wr = 3.0;
                        let zd = 4.0 + (seed % 3) as f32 * 0.5;
                        let d = pos - su;
                        let aqj = d - Self::bbp(d / zd) * zd;
                        
                        let mut brightness: f32 = 0.06;
                        if aqj >= 0.0 && aqj < wr {
                            brightness = (1.0 - aqj / wr).max(0.0);
                            if brightness < 0.06 { brightness = 0.06; }
                        }
                        
                        let dkl = (cx as u32).wrapping_mul(2654435761)
                            .wrapping_add((u as u32).wrapping_mul(340573321));
                        let anim_frame = (time * 6.0) as u32;
                        let axi = ((dkl.wrapping_add(anim_frame)) % OC_ as u32) as usize;
                        let du = &PA_[axi];
                        
                        let color = if brightness > 0.88 {
                            let g = (brightness * 200.0) as u8;
                            0xFF000000 | 0x00200000 | ((g as u32) << 8) | 0x18
                        } else {
                            let g = (12.0 + brightness * 140.0) as u8;
                            0xFF000000 | ((g as u32) << 8) | 0x04
                        };
                        
                        self.draw_glyph_3x3(buffer, fb_width, cx * cell_size + 1, u * cell_size + 1, du, color);
                    }
                    continue;
                }
                
                
                if gno(p, o, &grl) {
                    let aqa = 10.0_f32;
                    let chk = p / aqa;
                    let fnp = chk - Self::bbp(chk);
                    
                    if fnp < 0.4 {
                        let fnq = Self::bbp(chk) as i32;
                        let seed = (fnq as u32).wrapping_mul(340573321);
                        let scroll_speed = 2.8 + (seed % 5) as f32 * 0.25;
                        let phase = (seed % 100) as f32 * 0.05;
                        let su = time * scroll_speed + phase;
                        let pos = o / aqa;
                        let wr = 3.5;
                        let zd = 4.5 + (seed % 3) as f32 * 0.5;
                        let d = pos - su;
                        let aqj = d - Self::bbp(d / zd) * zd;
                        
                        let mut brightness: f32 = 0.06;
                        if aqj >= 0.0 && aqj < wr {
                            brightness = (1.0 - aqj / wr).max(0.0);
                            if brightness < 0.06 { brightness = 0.06; }
                        }
                        
                        let dkl = (cx as u32).wrapping_mul(2654435761)
                            .wrapping_add((u as u32).wrapping_mul(340573321));
                        let anim_frame = (time * 7.0) as u32;
                        let axi = ((dkl.wrapping_add(anim_frame)) % OC_ as u32) as usize;
                        let du = &PA_[axi];
                        
                        let color = if brightness > 0.88 {
                            let g = (brightness * 230.0) as u8;
                            0xFF000000 | 0x00280000 | ((g as u32) << 8) | 0x20
                        } else {
                            let g = (15.0 + brightness * 170.0) as u8;
                            0xFF000000 | ((g as u32) << 8) | 0x06
                        };
                        
                        self.draw_glyph_3x3(buffer, fb_width, cx * cell_size + 1, u * cell_size + 1, du, color);
                    }
                }
            }
        }
    }
    
    
    fn draw_line_thick(&self, buffer: &mut [u32], fb_width: usize, fb_height: usize,
                       bm: i32, az: i32, x1: i32, y1: i32, color: u32, rh: i32) {
        
        let oq = 100i32;
        let w = fb_width as i32;
        let h = fb_height as i32;
        if (bm < -oq && x1 < -oq) || (bm > w + oq && x1 > w + oq) {
            return;
        }
        if (az < -oq && y1 < -oq) || (az > h + oq && y1 > h + oq) {
            return;
        }
        
        let dx = (x1 - bm).abs();
        let ad = -(y1 - az).abs();
        let am = if bm < x1 { 1 } else { -1 };
        let ak = if az < y1 { 1 } else { -1 };
        let mut err = dx + ad;
        
        let mut x = bm;
        let mut y = az;
        
        
        let ayd = (dx.abs() + (-ad).abs() + 10) as usize;
        let mut steps = 0usize;
        
        loop {
            steps += 1;
            if steps > ayd { break; }
            
            
            for ty in -rh/2..=rh/2 {
                for bu in -rh/2..=rh/2 {
                    let p = x + bu;
                    let o = y + ty;
                    if p >= 0 && o >= 0 {
                        let dxa = p as usize;
                        let dxb = o as usize;
                        if dxa < fb_width && dxb < fb_height {
                            buffer[dxb * fb_width + dxa] = color;
                        }
                    }
                }
            }
            
            if x == x1 && y == y1 { break; }
            
            let pg = 2 * err;
            if pg >= ad {
                err += ad;
                x += am;
            }
            if pg <= dx {
                err += dx;
                y += ak;
            }
        }
    }
    
    
    fn qds(&self, buffer: &mut [u32], fb_width: usize, fb_height: usize,
                              bm: i32, az: i32, x1: i32, y1: i32, 
                              qf: u32, depth: f32, edge_idx: usize) {
        
        let oq = 100i32;
        let w = fb_width as i32;
        let h = fb_height as i32;
        if (bm < -oq && x1 < -oq) || (bm > w + oq && x1 > w + oq) {
            return;
        }
        if (az < -oq && y1 < -oq) || (az > h + oq && y1 > h + oq) {
            return;
        }
        
        let time = self.shape_time;
        
        
        let hul = (x1 - bm) as f32;
        let hum = (y1 - az) as f32;
        let wh = (hul * hul + hum * hum).max(1.0);
        let wh = {
            let mut x = wh;
            let mut y = wh * 0.5;
            for _ in 0..4 { y = (y + x / y) * 0.5; }
            y
        };
        
        let dx = (x1 - bm).abs();
        let ad = -(y1 - az).abs();
        let am = if bm < x1 { 1 } else { -1 };
        let ak = if az < y1 { 1 } else { -1 };
        let mut err = dx + ad;
        
        let mut x = bm;
        let mut y = az;
        let mut aza = 0usize;
        
        
        let ayd = (dx.abs() + (-ad).abs() + 10) as usize;
        let mut steps = 0usize;
        
        
        let irr = 3;
        
        let nzp = 2.0 + (edge_idx as f32 * 0.3);
        
        let nzq = 12.0;
        
        loop {
            steps += 1;
            if steps > ayd { break; }
            
            
            let t = aza as f32 / wh.max(1.0);
            
            
            let mut coo = 0.0f32;
            for aa in 0..irr {
                
                let nzn = (time * nzp + aa as f32 / irr as f32) % 1.0;
                let goy = nzn;
                
                
                let lfv = (t - goy).abs();
                let bgb = (t - goy - 1.0).abs();
                let lfw = (t - goy + 1.0).abs();
                let em = lfv.min(bgb).min(lfw);
                
                
                let goz = em * wh / nzq;
                if goz < 1.0 {
                    coo += (1.0 - goz * goz).max(0.0);
                }
            }
            coo = coo.min(1.0);
            
            
            let (adi, agd, apu) = (
                ((qf >> 16) & 0xFF) as f32,
                ((qf >> 8) & 0xFF) as f32,
                (qf & 0xFF) as f32,
            );
            
            
            let nzo = 220.0;
            let nzm = 255.0;
            let nzl = 220.0;
            
            let r = (adi + (nzo - adi) * coo) as u32;
            let g = (agd + (nzm - agd) * coo) as u32;
            let b = (apu + (nzl - apu) * coo) as u32;
            let color = 0xFF000000 | (r << 16) | (g << 8) | b;
            
            
            let rh = if coo > 0.3 { 2 } else { 1 };
            for ty in -rh/2..=rh/2 {
                for bu in -rh/2..=rh/2 {
                    let p = x + bu;
                    let ixe = y + ty;
                    if p >= 0 && ixe >= 0 {
                        let dxa = p as usize;
                        let dxb = ixe as usize;
                        if dxa < fb_width && dxb < fb_height {
                            buffer[dxb * fb_width + dxa] = color;
                        }
                    }
                }
            }
            
            aza += 1;
            
            if x == x1 && y == y1 { break; }
            
            let pg = 2 * err;
            if pg >= ad {
                err += ad;
                x += am;
            }
            if pg <= dx {
                err += dx;
                y += ak;
            }
        }
    }
    
    
    #[inline(always)]
    fn draw_glyph_3x3(&self, buffer: &mut [u32], fb_width: usize,
                       p: usize, o: usize, du: &[u8; 3], color: u32) {
        let fb_height = buffer.len() / fb_width;
        if o + 2 >= fb_height || p + 2 >= fb_width { return; }
        
        let kl = du[0];
        let aje = o * fb_width + p;
        if kl & 0b001 != 0 { buffer[aje] = color; }
        if kl & 0b010 != 0 { buffer[aje + 1] = color; }
        if kl & 0b100 != 0 { buffer[aje + 2] = color; }
        
        let gf = du[1];
        let ajf = aje + fb_width;
        if gf & 0b001 != 0 { buffer[ajf] = color; }
        if gf & 0b010 != 0 { buffer[ajf + 1] = color; }
        if gf & 0b100 != 0 { buffer[ajf + 2] = color; }
        
        let iq = du[2];
        let bes = ajf + fb_width;
        if iq & 0b001 != 0 { buffer[bes] = color; }
        if iq & 0b010 != 0 { buffer[bes + 1] = color; }
        if iq & 0b100 != 0 { buffer[bes + 2] = color; }
    }

    
    #[inline(always)]
    fn ljc(&self, buffer: &mut [u32], fb_width: usize, 
                       p: usize, o: usize, du: &[u8; 6], color: u32) {
        let fb_height = buffer.len() / fb_width;
        if o >= fb_height || p >= fb_width { return; }
        let buh = (fb_height - o).min(6);
        let agr = (fb_width - p).min(6);
        for row in 0..buh {
            let asc = du[row];
            if asc == 0 { continue; }
            let fk = (o + row) * fb_width + p;
            if asc & 0b000001 != 0 && 0 < agr { buffer[fk] = color; }
            if asc & 0b000010 != 0 && 1 < agr { buffer[fk + 1] = color; }
            if asc & 0b000100 != 0 && 2 < agr { buffer[fk + 2] = color; }
            if asc & 0b001000 != 0 && 3 < agr { buffer[fk + 3] = color; }
            if asc & 0b010000 != 0 && 4 < agr { buffer[fk + 4] = color; }
            if asc & 0b100000 != 0 && 5 < agr { buffer[fk + 5] = color; }
        }
    }
}






pub struct FastMatrixRenderer {
    
    heads: Vec<i32>,
    
    speeds: Vec<u8>,
    
    chars: Vec<u8>,
    
    frame: u32,
    
    cols: usize,
    rows: usize,
}

impl FastMatrixRenderer {
    pub fn new() -> Self {
        
        let cols = 1280 / ABL_; 
        let rows = 800 / ABK_;  
        
        let mut heads = vec![0i32; cols];
        let mut speeds = vec![1u8; cols];
        let mut chars = vec![0u8; cols * rows];
        
        
        for i in 0..cols {
            let seed = (i as u32).wrapping_mul(2654435761) ^ 0xDEADBEEF;
            heads[i] = -((seed % (rows as u32 * 2)) as i32);
            speeds[i] = 1 + (seed % 2) as u8;
            
            
            for ay in 0..rows {
                let bfe = seed.wrapping_mul((ay + 1) as u32);
                chars[i * rows + ay] = BBI_[(bfe as usize) % BBI_.len()];
            }
        }
        
        Self { heads, speeds, chars, frame: 0, cols, rows }
    }
    
    
    pub fn update(&mut self) {
        self.frame = self.frame.wrapping_add(1);
        
        for i in 0..self.cols {
            self.heads[i] += self.speeds[i] as i32;
            
            
            if self.heads[i] > (self.rows as i32) + 20 {
                let seed = (i as u32).wrapping_mul(self.frame).wrapping_add(0xBEEF);
                self.heads[i] = -((seed % 30) as i32);
            }
        }
    }
    
    
    pub fn render(&self, buffer: &mut [u32], fb_width: usize, fb_height: usize) {
        let cols = (fb_width / ABL_).min(self.cols);
        let rows = (fb_height / ABK_).min(self.rows);
        
        for col in 0..cols {
            let head_y = self.heads[col];
            
            for row in 0..rows {
                let em = head_y - (row as i32);
                
                if em >= 0 && em < 16 {
                    
                    let intensity = AFD_[(em as usize).min(31)];
                    if intensity > 10 {
                        
                        let c = self.chars[col * self.rows + row];
                        
                        
                        let color = (0xFF << 24) | ((intensity as u32) << 8);
                        
                        
                        self.draw_char(buffer, fb_width, col, row, c, color);
                    }
                }
            }
        }
    }
    
    
    fn draw_char(&self, buffer: &mut [u32], fb_width: usize, 
                 col: usize, row: usize, c: u8, color: u32) {
        let p = col * ABL_;
        let o = row * ABK_;
        
        let du = crate::framebuffer::font::ol(c as char);
        
        for (jh, &bits) in du.iter().enumerate() {
            let y = o + jh;
            if y >= buffer.len() / fb_width { continue; }
            
            for hc in 0..8 {
                if (bits >> (7 - hc)) & 1 != 0 {
                    let x = p + hc;
                    let idx = y * fb_width + x;
                    if idx < buffer.len() {
                        buffer[idx] = color;
                    }
                }
            }
        }
    }
}






const AGT_: usize = 200;


#[derive(Clone, Copy)]
struct Drop3D {
    
    x: f32,
    
    y: f32,
    
    z: f32,
    
    vx: f32,
    
    vy: f32,
    
    vz: f32,
    
    trail_len: u8,
    
    glyph_seed: u32,
    
    on_surface: bool,
    
    flow_time: u8,
}

impl Drop3D {
    fn new() -> Self {
        Self {
            x: 0.0, y: -10.0, z: 0.5,
            vx: 0.0, vy: 0.5, vz: 0.0,
            trail_len: 20,
            glyph_seed: 0,
            on_surface: false,
            flow_time: 0,
        }
    }
}


#[derive(Clone, Copy)]
pub enum Shape3D {
    
    Sphere { cx: f32, u: f32, mj: f32, r: f32 },
    
    Cube { cx: f32, u: f32, mj: f32, cw: f32, biy: f32 },
    
    Torus { cx: f32, u: f32, mj: f32, U: f32, r: f32 },
}


pub struct Matrix3D {
    
    drops: [Drop3D; AGT_],
    
    shapes: [Option<Shape3D>; 4],
    
    rng: u32,
    
    frame: u32,
    
    time: f32,
    
    width: usize,
    height: usize,
}

impl Matrix3D {
    pub fn new() -> Self {
        let mut drops = [Drop3D::new(); AGT_];
        let mut rng = 0xDEADBEEFu32;
        
        
        for drop in drops.iter_mut() {
            rng = rng.wrapping_mul(1103515245).wrapping_add(12345);
            drop.x = (rng % 160) as f32;
            rng = rng.wrapping_mul(1103515245).wrapping_add(12345);
            drop.y = -((rng % 120) as f32);
            rng = rng.wrapping_mul(1103515245).wrapping_add(12345);
            drop.z = 0.2 + (rng % 80) as f32 / 100.0; 
            
            
            drop.vy = 0.3 + drop.z * 0.7; 
            
            rng = rng.wrapping_mul(1103515245).wrapping_add(12345);
            drop.trail_len = 10 + (rng % 30) as u8;
            drop.glyph_seed = rng;
        }
        
        
        let shapes = [
            Some(Shape3D::Sphere { cx: 80.0, u: 50.0, mj: 0.5, r: 15.0 }),
            None,
            None,
            None,
        ];
        
        Self {
            drops,
            shapes,
            rng,
            frame: 0,
            time: 0.0,
            width: 1280,
            height: 800,
        }
    }
    
    
    pub fn pxw(&mut self, shape: Shape3D) {
        for slot in self.shapes.iter_mut() {
            if slot.is_none() {
                *slot = Some(shape);
                return;
            }
        }
    }
    
    
    pub fn qab(&mut self) {
        for slot in self.shapes.iter_mut() {
            *slot = None;
        }
    }
    
    
    #[inline(always)]
    fn eu(x: f32) -> f32 { crate::math::eu(x) }
    
    
    #[inline(always)]
    fn hr(x: f32) -> f32 { crate::math::hr(x) }
    
    
    fn kiu(shapes: &[Option<Shape3D>; 4], x: f32, y: f32, z: f32) -> Option<(f32, f32, f32)> {
        for shape in shapes.iter().filter_map(|j| *j) {
            match shape {
                Shape3D::Sphere { cx, u, mj, r } => {
                    let dx = x - cx;
                    let ad = y - u;
                    let dz = z - mj;
                    let wz = dx * dx + ad * ad + dz * dz;
                    let amn = r * r;
                    
                    if wz < amn {
                        
                        let em = Self::ra(wz).max(0.01);
                        return Some((dx / em, ad / em, dz / em));
                    }
                }
                Shape3D::Cube { cx, u, mj, cw, biy } => {
                    
                    let dx = x - cx;
                    let ad = y - u;
                    let dz = z - mj;
                    
                    let bax = Self::hr(biy);
                    let bds = Self::eu(biy);
                    
                    
                    let da = dx * bax - dz * bds;
                    let cm = ad;
                    let qp = dx * bds + dz * bax;
                    
                    
                    if da.abs() < cw && cm.abs() < cw && qp.abs() < cw {
                        
                        let ax = cw - da.abs();
                        let aet = cw - cm.abs();
                        let did = cw - qp.abs();
                        
                        
                        if ax < aet && ax < did {
                            let nx = if da > 0.0 { 1.0 } else { -1.0 };
                            return Some((nx * bax, 0.0, nx * bds));
                        } else if aet < did {
                            return Some((0.0, if cm > 0.0 { 1.0 } else { -1.0 }, 0.0));
                        } else {
                            let wi = if qp > 0.0 { 1.0 } else { -1.0 };
                            return Some((-wi * bds, 0.0, wi * bax));
                        }
                    }
                }
                Shape3D::Torus { cx, u, mj, U, r } => {
                    let dx = x - cx;
                    let ad = y - u;
                    let dz = z - mj;
                    
                    
                    let dnj = Self::ra(dx * dx + dz * dz);
                    
                    let fdt = dnj - U;
                    let fdr = Self::ra(fdt * fdt + ad * ad);
                    
                    if fdr < r {
                        
                        let lev = if dnj > 0.01 { dx / dnj } else { 1.0 };
                        let lew = if dnj > 0.01 { dz / dnj } else { 0.0 };
                        
                        let nx = fdt * lev / fdr.max(0.01);
                        let re = ad / fdr.max(0.01);
                        let wi = fdt * lew / fdr.max(0.01);
                        
                        return Some((nx, re, wi));
                    }
                }
            }
        }
        None
    }
    
    
    #[inline(always)]
    fn ra(x: f32) -> f32 { crate::math::ra(x) }
    
    
    pub fn update(&mut self) {
        self.frame = self.frame.wrapping_add(1);
        self.time += 0.02; 
        
        
        if let Some(Shape3D::Cube { ref mut biy, .. }) = self.shapes.get_mut(0).and_then(|j| j.as_mut()) {
            *biy = self.time;
        }
        
        let fzj = 0.02f32;
        
        
        let shapes = self.shapes;
        
        for i in 0..AGT_ {
            let drop = &mut self.drops[i];
            
            
            if let Some((nx, re, wi)) = Self::kiu(&shapes, drop.x, drop.y, drop.z) {
                if !drop.on_surface {
                    
                    drop.on_surface = true;
                    drop.flow_time = 0;
                    
                    
                    
                    let dot = drop.vx * nx + drop.vy * re + drop.vz * wi;
                    drop.vx -= dot * nx;
                    drop.vy -= dot * re;
                    drop.vz -= dot * wi;
                    
                    
                    let icq = re; 
                    drop.vy += fzj * (1.0 - icq * icq).max(0.0);
                }
                
                
                drop.x += nx * 0.2;
                drop.y += re * 0.2;
                drop.z += wi * 0.2;
                
                
                drop.vx *= 0.95;
                drop.vy *= 0.95;
                drop.vz *= 0.95;
                
                
                drop.vy += fzj * 0.5;
                
                drop.flow_time += 1;
            } else {
                
                drop.on_surface = false;
                drop.vy += fzj;
            }
            
            
            drop.x += drop.vx;
            drop.y += drop.vy;
            drop.z += drop.vz;
            
            
            drop.z = drop.z.clamp(0.1, 1.0);
            
            
            let reset = drop.y > 110.0 || drop.x < -5.0 || drop.x > 165.0 || drop.flow_time > 100;
            
            if reset {
                self.rng = self.rng.wrapping_mul(1103515245).wrapping_add(12345);
                drop.x = (self.rng % 160) as f32;
                self.rng = self.rng.wrapping_mul(1103515245).wrapping_add(12345);
                drop.y = -((self.rng % 40) as f32);
                self.rng = self.rng.wrapping_mul(1103515245).wrapping_add(12345);
                drop.z = 0.2 + (self.rng % 80) as f32 / 100.0;
                
                drop.vx = 0.0;
                drop.vy = 0.3 + drop.z * 0.7;
                drop.vz = 0.0;
                drop.on_surface = false;
                drop.flow_time = 0;
                drop.glyph_seed = self.rng;
            }
            
            
            drop.glyph_seed = drop.glyph_seed.wrapping_mul(1103515245).wrapping_add(12345);
        }
    }
    
    
    #[inline(always)]
    fn project(&self, x: f32, y: f32, z: f32) -> (i32, i32, f32) {
        
        
        let scale = 0.7 + z * 0.3; 
        let center_x = 80.0;
        let center_y = 50.0;
        
        
        let lw = center_x + (x - center_x) * scale;
        let nn = center_y + (y - center_y) * scale;
        
        (lw as i32, nn as i32, z)
    }
    
    
    pub fn render(&self, buffer: &mut [u32], fb_width: usize, fb_height: usize) {
        
        let bg_color = 0xFF010201u32;
        buffer.fill(bg_color);
        
        let cell_size = EO_;
        
        
        
        
        for drop in self.drops.iter() {
            if drop.y < -5.0 { continue; }
            
            let (lw, nn, depth) = self.project(drop.x, drop.y, drop.z);
            
            
            let dms = (50.0 + depth * 205.0) as u32;
            
            
            let oys = if drop.on_surface { 30u32 } else { 0 };
            
            
            let trail_len = drop.trail_len as usize;
            for bab in 0..trail_len {
                
                let ty = drop.y - bab as f32 * 0.8;
                if ty < 0.0 { continue; }
                
                let (bu, ty_screen, _) = self.project(drop.x, ty, drop.z);
                
                if bu < 0 || bu >= (fb_width / cell_size) as i32 { continue; }
                if ty_screen < 0 || ty_screen >= (fb_height / cell_size) as i32 { continue; }
                
                
                let gdc = (bab * 63) / trail_len.max(1);
                let din = AFD_[gdc.min(63)] as u32;
                let intensity = (((din * dms) / 255) + oys).min(255) as u8;
                
                if intensity < 2 { continue; }
                
                
                let glyph_seed = drop.glyph_seed.wrapping_add(bab as u32 * 2654435761);
                let axi = (glyph_seed % OC_ as u32) as usize;
                let du = &PA_[axi];
                let color = ihb(intensity);
                
                let p = bu as usize * cell_size + 1;
                let o = ty_screen as usize * cell_size + 1;
                self.draw_glyph_3x3(buffer, fb_width, p, o, du, color);
            }
        }
        
        
        
    }
    
    
    #[inline(always)]
    fn draw_glyph_3x3(&self, buffer: &mut [u32], fb_width: usize,
                       p: usize, o: usize, du: &[u8; 3], color: u32) {
        let fb_height = buffer.len() / fb_width;
        if o + 2 >= fb_height || p + 2 >= fb_width { return; }
        let kl = du[0]; let aje = o * fb_width + p;
        if kl & 0b001 != 0 { buffer[aje] = color; }
        if kl & 0b010 != 0 { buffer[aje + 1] = color; }
        if kl & 0b100 != 0 { buffer[aje + 2] = color; }
        let gf = du[1]; let ajf = aje + fb_width;
        if gf & 0b001 != 0 { buffer[ajf] = color; }
        if gf & 0b010 != 0 { buffer[ajf + 1] = color; }
        if gf & 0b100 != 0 { buffer[ajf + 2] = color; }
        let iq = du[2]; let bes = ajf + fb_width;
        if iq & 0b001 != 0 { buffer[bes] = color; }
        if iq & 0b010 != 0 { buffer[bes + 1] = color; }
        if iq & 0b100 != 0 { buffer[bes + 2] = color; }
    }
    
    
    #[inline(always)]
    fn ljc(&self, buffer: &mut [u32], fb_width: usize, 
                       p: usize, o: usize, du: &[u8; 6], color: u32) {
        let fb_height = buffer.len() / fb_width;
        if o >= fb_height || p >= fb_width { return; }
        let buh = (fb_height - o).min(6);
        let agr = (fb_width - p).min(6);
        for row in 0..buh {
            let asc = du[row];
            if asc == 0 { continue; }
            let fk = (o + row) * fb_width + p;
            if asc & 0b000001 != 0 && 0 < agr { buffer[fk] = color; }
            if asc & 0b000010 != 0 && 1 < agr { buffer[fk + 1] = color; }
            if asc & 0b000100 != 0 && 2 < agr { buffer[fk + 2] = color; }
            if asc & 0b001000 != 0 && 3 < agr { buffer[fk + 3] = color; }
            if asc & 0b010000 != 0 && 4 < agr { buffer[fk + 4] = color; }
            if asc & 0b100000 != 0 && 5 < agr { buffer[fk + 5] = color; }
        }
    }
    
    
    pub fn set_demo_shapes(&mut self) {
        self.shapes = [
            
            Some(Shape3D::Sphere { cx: 80.0, u: 50.0, mj: 0.5, r: 18.0 }),
            None,
            None,
            None,
        ];
    }
    
    
    pub fn set_cube(&mut self) {
        self.shapes = [
            Some(Shape3D::Cube { cx: 80.0, u: 50.0, mj: 0.5, cw: 12.0, biy: self.time }),
            None,
            None,
            None,
        ];
    }
    
    
    pub fn set_torus(&mut self) {
        self.shapes = [
            Some(Shape3D::Torus { cx: 80.0, u: 50.0, mj: 0.5, U: 15.0, r: 5.0 }),
            None,
            None,
            None,
        ];
    }
}
