






use alloc::vec::Vec;
use alloc::vec;






pub const DDV_: usize = 2;   
pub const DDU_: usize = 4;   
pub const ZZ_: usize = 16;    
pub const ZY_: usize = 32;    


const EIP_: i32 = 64;



const ADN_: [u8; 64] = [
    255, 252, 248, 244, 240, 236, 231, 226,
    221, 216, 210, 204, 198, 192, 186, 179,
    172, 165, 158, 151, 144, 137, 130, 123,
    116, 109, 102, 96, 90, 84, 78, 72,
    67, 62, 57, 52, 48, 44, 40, 36,
    33, 30, 27, 24, 22, 20, 18, 16,
    14, 13, 12, 11, 10, 9, 8, 7,
    6, 5, 5, 4, 4, 3, 3, 2,  
];




const fn tbz() -> [u32; 256] {
    let mut djf = [0xFF000000u32; 256];
    let mut a = 1u32;
    while a < 256 {
        let s = if a > 250 {
            
            let ab = a - 250; 
            let m = 180 + ab * 15; 
            let at = 255;
            let o = 220 + ab * 7; 
            (0xFF << 24) | (m << 16) | (at << 8) | o
        } else if a > 235 {
            
            let ab = a - 235; 
            let m = 60 + ab * 8; 
            let at = 220 + ab * 2; 
            let o = 120 + ab * 6; 
            (0xFF << 24) | (m << 16) | (at << 8) | o
        } else if a > 200 {
            
            let ab = a - 200; 
            let m = ab; 
            let at = 180 + ab * 2; 
            let o = 30 + ab * 2; 
            (0xFF << 24) | (m << 16) | (at << 8) | o
        } else if a > 160 {
            
            let ab = a - 160; 
            let at = 140 + ab; 
            let m = ab / 6; 
            let o = 10 + ab / 3; 
            (0xFF << 24) | (m << 16) | (at << 8) | o
        } else if a > 120 {
            
            let ab = a - 120; 
            let at = 100 + ab; 
            let o = 8 + ab / 3; 
            let m = ab / 12;
            (0xFF << 24) | (m << 16) | (at << 8) | o
        } else if a > 80 {
            
            let ab = a - 80; 
            let at = 60 + ab; 
            let o = 4 + ab / 5; 
            (0xFF << 24) | (at << 8) | o
        } else if a > 50 {
            
            let ab = a - 50; 
            let at = 30 + ab; 
            let o = 3 + ab / 6; 
            (0xFF << 24) | (at << 8) | o
        } else if a > 25 {
            
            let ab = a - 25; 
            let at = 12 + ab; 
            let o = 2 + ab / 5; 
            (0xFF << 24) | (at << 8) | o
        } else if a > 10 {
            
            let ab = a - 10; 
            let at = 5 + ab / 2; 
            let o = 1 + ab / 8;
            (0xFF << 24) | (at << 8) | o
        } else {
            
            let at = 2 + a / 3;
            let o = 1 + a / 5;
            (0xFF << 24) | (at << 8) | o
        };
        djf[a as usize] = s;
        a += 1;
    }
    djf
}


static BNE_: [u32; 256] = tbz();


#[inline(always)]
pub(crate) fn oex(hj: u8) -> u32 {
    BNE_[hj as usize]
}





pub(crate) const CFC_: [[u8; 6]; 64] = [
    
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


const DNM_: usize = 64;


pub(crate) const AZH_: &[u8] = b"0123456789ABCDEF@#$%&*<>[]{}|/\\";







const EB_: usize = 4;

const DNN_: usize = 3;

const RZ_: usize = EB_ / 2;


const VG_: usize = 15;
const VB_: usize = 50;


const DH_: usize = 6;


const AEW_: usize = 512;




const OC_: [[u8; 3]; 64] = [
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


const ND_: usize = 64;


#[derive(Clone, Copy)]
struct RainDrop {
    
    c: i32,
    
    ig: u8,
    
    bmv: u8,
    
    acr: u8,
    
    amg: u32,
    
    gh: bool,
}

impl RainDrop {
    fn usx() -> Self {
        Self {
            c: -100,
            ig: 1,
            bmv: 0,
            acr: VG_ as u8,
            amg: 0,
            gh: false,
        }
    }
    
    
    fn prt(&self) -> i32 {
        self.c - self.acr as i32
    }
}






#[derive(Clone, Copy, PartialEq)]
pub enum ShapeOverlay {
    None,
    Dw,
    Sphere,
    Dr,
    Ij,
}


const VA_: usize = 144;


const AEY_: usize = 500;  


#[derive(Clone, Copy)]
struct ShapeDrop {
    
    bj: i32,
    
    br: i32,
    
    lvi: i32,
    
    frd: i32,
    
    eo: f32,
    
    li: f32,
    
    fhi: u8,
    
    acr: u8,
    
    ig: f32,
    
    amg: u32,
}



#[derive(Clone, Copy)]
struct Aqk {
    
    xu: f32,
    
    abi: f32,
    
    xra: f32,
    
    xrb: f32,
    
    acr: u8,
    
    gh: bool,
    
    amg: u32,
    
    can: f32,
}

impl ShapeDrop {
    fn new() -> Self {
        Self {
            bj: 0,
            br: 0,
            lvi: 0,
            frd: 0,
            eo: 0.5,
            li: 0.0,
            fhi: 0,
            acr: 6,
            ig: 0.012,
            amg: 0,
        }
    }
}






#[repr(C)]
struct Aac {
    doo: *mut u32,
    aic: usize,
    lu: usize,
    qh: usize,
    ny: usize,
    kgx: usize,
    
    nnx: *const [RainDrop; DH_],
    neo: *const u8,
    ajg: usize,
    
    bgo: bool,
    kmj: f32,
    kmh: f32,
    kmk: f32,
    kmi: f32,
    mlv: [(f32, f32, f32, f32); 4],
    lih: [(f32, f32, f32, f32); 4],
    mab: [(f32, f32, f32, f32); 4],
    lro: [(f32, f32, f32, f32); 4],
}

unsafe impl Send for Aac {}
unsafe impl Sync for Aac {}



#[inline(always)]
unsafe fn sdj(bi: *mut u32, lu: usize, qh: usize,
                              y: usize, x: usize, ka: &[u8; 3], s: u32) {
    if x + 2 >= qh || y + 2 >= lu { return; }
    
    let wu = ka[0];
    let bpq = x * lu + y;
    if wu & 0b001 != 0 { *bi.add(bpq) = s; }
    if wu & 0b010 != 0 { *bi.add(bpq + 1) = s; }
    if wu & 0b100 != 0 { *bi.add(bpq + 2) = s; }
    
    let of = ka[1];
    let bpr = bpq + lu;
    if of & 0b001 != 0 { *bi.add(bpr) = s; }
    if of & 0b010 != 0 { *bi.add(bpr + 1) = s; }
    if of & 0b100 != 0 { *bi.add(bpr + 2) = s; }
    
    let tb = ka[2];
    let deb = bpr + lu;
    if tb & 0b001 != 0 { *bi.add(deb) = s; }
    if tb & 0b010 != 0 { *bi.add(deb + 1) = s; }
    if tb & 0b100 != 0 { *bi.add(deb + 2) = s; }
}


#[inline]
unsafe fn ynb(bi: *mut u32, lu: usize, qh: usize,
                              y: usize, x: usize, ka: &[u8; 6], s: u32) {
    if x >= qh || y >= lu { return; }
    let efh = (qh - x).v(6);
    let bkj = (lu - y).v(6);
    for br in 0..efh {
        let fs = ka[br];
        if fs == 0 { continue; }
        let ar = (x + br) * lu + y;
        if fs & 0b000001 != 0 && 0 < bkj { *bi.add(ar) = s; }
        if fs & 0b000010 != 0 && 1 < bkj { *bi.add(ar + 1) = s; }
        if fs & 0b000100 != 0 && 2 < bkj { *bi.add(ar + 2) = s; }
        if fs & 0b001000 != 0 && 3 < bkj { *bi.add(ar + 3) = s; }
        if fs & 0b010000 != 0 && 4 < bkj { *bi.add(ar + 4) = s; }
        if fs & 0b100000 != 0 && 5 < bkj { *bi.add(ar + 5) = s; }
    }
}


#[inline(always)]
fn jjo(y: f32, x: f32, bu: &[(f32, f32, f32, f32); 4]) -> bool {
    let mut u = 0u8;
    let mut neg = 0u8;
    for &(bqp, ahm, mp, qw) in bu.iter() {
        let bjr = bqp * (x - qw) - ahm * (y - mp);
        if bjr > 0.0 { u += 1; }
        else if bjr < 0.0 { neg += 1; }
    }
    u == 0 || neg == 0
}


fn vvl(ay: usize, ci: usize, f: *mut u8) {
    let ai = unsafe { &*(f as *const Aac) };
    let ny = ai.ny;
    
    for bj in ay..ci {
        if bj >= ai.ajg { break; }
        
        let eo = unsafe { *ai.neo.add(bj) };
        let hfu = 100 + (eo as u32 * 155 / 255);
        
        let cds = (bj * ny + RZ_) as f32;
        let rlw = ai.bgo && cds >= ai.kmj && cds <= ai.kmh;
        
        let agk = unsafe { &*ai.nnx.add(bj) };
        
        for cxd in 0..DH_ {
            let drop = &agk[cxd];
            if !drop.gh { continue; }
            
            let buu = drop.c;
            let ies = drop.acr as usize;
            
            for cuv in 0..ies {
                let bmg = buu - cuv as i32;
                if bmg < 0 || bmg >= ai.kgx as i32 { continue; }
                
                let lfa = (cuv * 63) / ies.am(1);
                let gzv = ADN_[lfa.v(63)] as u32;
                let hj = ((gzv * hfu) / 255) as u8;
                
                if rlw {
                    let x = (bmg as usize * ny + RZ_) as f32;
                    if x >= ai.kmk && x <= ai.kmi {
                        if jjo(cds, x, &ai.lro) { continue; }
                        if jjo(cds, x, &ai.mlv)
                            || jjo(cds, x, &ai.lih)
                            || jjo(cds, x, &ai.mab) {
                            continue;
                        }
                    }
                }
                
                if hj < 2 { continue; }
                
                let amg = drop.amg.cn(cuv as u32 * 2654435761);
                let cqy = (amg % ND_ as u32) as usize;
                let ka = &OC_[cqy];
                let s = oex(hj);
                
                let y = bj * ny + 1;
                let x = bmg as usize * ny + 1;
                unsafe {
                    sdj(ai.doo, ai.lu, ai.qh,
                                       y, x, ka, s);
                }
                
                
                if cuv == 0 && hj > 200 {
                    let qz = y + 1; 
                    let ub = x + 1;
                    let bkr: [(i32, i32); 4] = [(-1,0),(1,0),(0,-1),(0,1)];
                    for &(mp, qw) in &bkr {
                        let gx = qz as i32 + mp;
                        let ty = ub as i32 + qw;
                        if gx >= 0 && gx < ai.lu as i32 && ty >= 0 && ty < ai.qh as i32 {
                            let w = ty as usize * ai.lu + gx as usize;
                            unsafe {
                                let aa = *ai.doo.add(w);
                                let nr = (((aa >> 16) & 0xFF) + 10).v(255);
                                let csu = (((aa >> 8) & 0xFF) + 48).v(255);
                                let csq = ((aa & 0xFF) + 32).v(255);
                                *ai.doo.add(w) = 0xFF000000 | (nr << 16) | (csu << 8) | csq;
                            }
                        }
                    }
                }
            }
        }
    }
}




pub struct BrailleMatrix {
    
    agk: Vec<[RainDrop; DH_]>,
    
    cwh: Vec<u8>,
    
    rng: u32,
    
    frame: u32,
    
    ajg: usize,
    
    bnr: usize,
    
    eio: ShapeOverlay,
    
    eip: f32,
    
    iae: Vec<ShapeDrop>,
    
    ein: usize,
    
    heo: Vec<Aqk>,
}

impl BrailleMatrix {
    pub fn new() -> Self {
        use alloc::vec::Vec;
        
        let ec = 1280 / EB_; 
        let lk = 800 / EB_;  
        
        
        let mut agk: Vec<[RainDrop; DH_]> = Vec::fc(AEW_);
        let mut cwh: Vec<u8> = vec![128u8; AEW_];
        let mut rng = 0xDEADBEEFu32;
        
        
        for _ in 0..AEW_ {
            agk.push([RainDrop::usx(); DH_]);
        }
        
        
        
        for bj in 0..ec {
            rng = rng.hx(1103515245).cn(12345);
            
            
            let pattern = ((bj * 17 + 53) % 97) as i32 - 48; 
            let lxa = (rng % 100) as i32 - 50; 
            let qng = 145i32 + pattern + lxa;
            cwh[bj] = qng.qp(30, 255) as u8;
        }
        
        
        for bj in 0..ec {
            let eo = cwh[bj];
            
            let mut oqn: i32 = 0;
            
            for cxd in 0..DH_ {
                rng = rng.hx(1103515245).cn(12345);
                
                
                let btz = eo as f32 / 255.0;
                let ydn = VB_ - VG_;
                let hrn = (VG_ as f32 * (0.5 + btz * 0.5)) as usize;
                let llg = (VB_ as f32 * (0.6 + btz * 0.4)) as usize;
                let acr = hrn + (rng % (llg - hrn + 1) as u32) as usize;
                
                rng = rng.hx(1103515245).cn(12345);
                
                
                
                let kxq = ((1.0 - btz) * 2.0) as i32; 
                let hkw = (2.0 + (1.0 - btz) * 6.0).am(1.0) as i32; 
                let qi = kxq + (rng % hkw as u32) as i32;
                let vc = oqn - (rng % 8) as i32;  
                
                
                oqn = vc - acr as i32 - qi;
                
                
                rng = rng.hx(1103515245).cn(12345);
                let fvc = (1.0 + (1.0 - btz) * 1.5) as u8; 
                let fvd = (2.0 + (1.0 - btz) * 3.0) as u8; 
                let ig = fvc + (rng % fvd as u32) as u8;
                
                rng = rng.hx(1103515245).cn(12345);
                
                agk[bj][cxd] = RainDrop {
                    c: vc,
                    ig,
                    bmv: (rng % ig as u32) as u8,
                    acr: acr as u8,
                    amg: rng,
                    gh: true,
                };
            }
        }
        
        
        let mut iae: Vec<ShapeDrop> = Vec::fc(VA_);
        for _ in 0..VA_ {
            iae.push(ShapeDrop::new());
        }
        
        
        let mut heo: Vec<Aqk> = Vec::fc(AEY_);
        for _ in 0..AEY_ {
            heo.push(Aqk {
                xu: 0.0,
                abi: 0.0,
                xra: 0.0,
                xrb: 0.0,
                acr: 5,
                gh: false,  
                amg: 0,
                can: 0.0,
            });
        }
        
        Self {
            agk,
            cwh,
            rng,
            frame: 0,
            ajg: ec,
            bnr: lk,
            eio: ShapeOverlay::None,  
            eip: 0.0,
            iae,
            ein: 0,
            heo,
        }
    }
    
    
    pub fn gsg(&mut self, ev: ShapeOverlay) {
        self.eio = ev;
        self.eip = 0.0;
        
        
        if ev == ShapeOverlay::Dw {
            for a in 0..AEY_ {
                self.heo[a].gh = false;
                self.heo[a].can = 0.0;
            }
        }
        
        if ev == ShapeOverlay::None {
            self.ein = 0;
            return;
        }
        
        
        let mut rng = 0x12345678u32;
        let sgw = match ev {
            ShapeOverlay::Dw => 90,      
            ShapeOverlay::Sphere => 64,    
            ShapeOverlay::Dr => 80,     
            ShapeOverlay::Ij => 64,       
            ShapeOverlay::None => 0,
        };
        
        self.ein = sgw.v(VA_);
        
        for a in 0..self.ein {
            rng = rng.hx(1103515245).cn(12345);
            self.iae[a] = ShapeDrop {
                bj: 0,
                br: 0,
                lvi: 0,
                frd: 0,
                eo: 0.5,
                li: (a as f32 / self.ein as f32),
                fhi: (a % 12) as u8,
                acr: 5 + (rng % 4) as u8,
                ig: 0.006 + (rng % 50) as f32 / 5000.0,
                amg: rng,
            };
        }
    }
    
    
    pub fn ytv(&self) -> ShapeOverlay {
        self.eio
    }
    
    
    #[inline(always)]
    fn lz(b: f32) -> f32 { crate::math::lz(b) }
    
    
    #[inline(always)]
    fn rk(b: f32) -> f32 { crate::math::rk(b) }
    
    
    #[inline(always)]
    fn ahn(b: f32) -> f32 { crate::math::ahn(b) }
    
    
    #[inline(always)]
    fn cxs(b: f32) -> f32 {
        let a = b as i32;
        if (a as f32) > b { (a - 1) as f32 } else { a as f32 }
    }

    
    pub fn qs(&mut self) {
        self.frame = self.frame.cn(1);
        
        let csl = (self.bnr as i32) + VB_ as i32 + 10;
        
        for bj in 0..self.ajg {
            let eo = self.cwh[bj];
            let btz = eo as f32 / 255.0;
            
            
            let mut lns: [bool; DH_] = [false; DH_];
            let mut onl: i32 = 0;
            
            for cxd in 0..DH_ {
                let drop = &self.agk[bj][cxd];
                if drop.gh {
                    let icx = drop.prt();
                    if icx < onl {
                        onl = icx;
                    }
                }
            }
            
            
            for cxd in 0..DH_ {
                let drop = &mut self.agk[bj][cxd];
                
                if !drop.gh {
                    continue;
                }
                
                
                drop.bmv = drop.bmv.cn(1);
                if drop.bmv >= drop.ig {
                    drop.bmv = 0;
                    drop.c += 1;
                    
                    
                    drop.amg = drop.amg.hx(1103515245).cn(12345);
                }
                
                
                if drop.c > csl {
                    lns[cxd] = true;
                }
            }
            
            
            for cxd in 0..DH_ {
                if !lns[cxd] {
                    continue;
                }
                
                
                let mut kmy: i32 = 0;
                for lqy in 0..DH_ {
                    if lqy != cxd && !lns[lqy] {
                        let drop = &self.agk[bj][lqy];
                        if drop.gh {
                            let icx = drop.prt();
                            if icx < kmy {
                                kmy = icx;
                            }
                        }
                    }
                }
                
                self.rng = self.rng.hx(1103515245).cn(12345);
                    
                
                let hrn = (VG_ as f32 * (0.5 + btz * 0.5)) as usize;
                let llg = (VB_ as f32 * (0.6 + btz * 0.4)) as usize;
                let oqf = hrn + (self.rng % (llg - hrn + 1).am(1) as u32) as usize;
                
                self.rng = self.rng.hx(1103515245).cn(12345);
                    
                
                let kxq = ((1.0 - btz) * 2.0) as i32; 
                let hkw = (2.0 + (1.0 - btz) * 6.0).am(1.0) as i32; 
                let qi = kxq + (self.rng % hkw as u32) as i32;
                    
                
                let bhn = kmy - oqf as i32 - qi - (self.rng % 5) as i32;
                
                self.rng = self.rng.hx(1103515245).cn(12345);
                    
                
                let fvc = (1.0 + (1.0 - btz) * 1.5) as u8;
                let fvd = (2.0 + (1.0 - btz) * 3.0).am(1.0) as u8;
                let utt = fvc + (self.rng % fvd as u32) as u8;
                
                self.rng = self.rng.hx(1103515245).cn(12345);
                
                let drop = &mut self.agk[bj][cxd];
                drop.acr = oqf as u8;
                drop.c = bhn;
                drop.ig = utt;
                drop.bmv = 0;
                drop.amg = self.rng;
            }
        }
        
        
        if self.eio != ShapeOverlay::None && self.ein > 0 {
            self.eip += 0.016; 
            
            
            let yv = (self.ajg * EB_ / 2) as f32;  
            let uq = (self.bnr * EB_ / 2) as f32;  
            let bv = ((self.bnr * EB_) as f32).v((self.ajg * EB_) as f32) * 0.18;  
            
            let az = self.ein.v(VA_);
            for a in 0..az {
                let drop = &mut self.iae[a];
                
                
                drop.li += drop.ig;
                if drop.li >= 1.0 {
                    drop.li -= 1.0;
                    drop.fhi = (drop.fhi + 1) % 9;  
                    drop.amg = drop.amg.hx(1103515245).cn(12345);
                }
                
                
                let (b, c, av) = match self.eio {
                    ShapeOverlay::Dw => {
                        
                        let aev = 0.785398_f32;  
                        let ajt = 0.523599_f32;  
                        let bmo = Self::rk(aev);
                        let bol = Self::lz(aev);
                        let bmn = Self::rk(ajt);
                        let bok = Self::lz(ajt);
                        
                        
                        let by: [(f32, f32, f32); 8] = [
                            (-1.0, -1.0, -1.0), (1.0, -1.0, -1.0),
                            (1.0, 1.0, -1.0), (-1.0, 1.0, -1.0),
                            (-1.0, -1.0, 1.0), (1.0, -1.0, 1.0),
                            (1.0, 1.0, 1.0), (-1.0, 1.0, 1.0),
                        ];
                        
                        
                        
                        let bu: [(usize, usize); 9] = [
                            (0,1), (0,4), (0,3),   
                            (1,5),                  
                            (4,5), (4,7),           
                            (5,6),                  
                            (3,7),                  
                            (6,7),                  
                        ];
                        
                        let amd = bu[drop.fhi as usize % 9];
                        let agy = by[amd.0];
                        let apg = by[amd.1];
                        
                        
                        let ab = drop.li;
                        let y = agy.0 + (apg.0 - agy.0) * ab;
                        let x = agy.1 + (apg.1 - agy.1) * ab;
                        let cbe = agy.2 + (apg.2 - agy.2) * ab;
                        
                        
                        let ehw = y * bmo - cbe * bol;
                        let cmn = y * bol + cbe * bmo;
                        
                        
                        let dbp = x * bmn - cmn * bok;
                        let cmo = x * bok + cmn * bmn;
                        
                        
                        let aab = 5.0;
                        let cgu = cmo + aab;
                        let aqf = aab / cgu.am(0.5);
                        
                        
                        let xu = yv + ehw * bv * aqf;
                        let abi = uq + dbp * bv * aqf;
                        
                        
                        let rvr = (cmo + 2.0) / 4.0;  
                        
                        (xu as i32, abi as i32, rvr)
                    },
                    ShapeOverlay::Sphere => {
                        
                        let bnv = (a as f32 / self.ein as f32) * 3.14159 * 2.0;
                        let bdb = drop.li * 3.14159;
                        let hg = self.eip * 0.5;
                        
                        let b = Self::lz(bdb) * Self::rk(bnv + hg);
                        let c = Self::lz(bdb) * Self::lz(bnv + hg);
                        let av = Self::rk(bdb);
                        
                        let aqf = 2.0 / (3.0 + av * 0.5);
                        let eo = (av + 1.0) / 2.0;
                        ((yv + b * bv * aqf) as i32,
                         (uq + c * bv * aqf * 0.7) as i32, eo)
                    },
                    ShapeOverlay::Dr => {
                        
                        let tm = drop.li * 6.28318;
                        let p = (a as f32 / self.ein as f32) * 6.28318;
                        let hg = self.eip * 0.4;
                        
                        let aqh = 1.5;
                        let uv = 0.5;
                        let b = (aqh + uv * Self::rk(p)) * Self::rk(tm + hg);
                        let c = (aqh + uv * Self::rk(p)) * Self::lz(tm + hg);
                        let av = uv * Self::lz(p);
                        
                        let aqf = 1.5 / (2.5 + av * 0.3);
                        let eo = (av + 0.5) / 1.0;
                        ((yv + b * bv * 0.6 * aqf) as i32,
                         (uq + c * bv * 0.4 * aqf) as i32, eo)
                    },
                    ShapeOverlay::Ij => {
                        
                        let ab = drop.li * 10.0 + (a as f32 * 0.1);
                        let hg = self.eip * 0.6;
                        let obm = if a % 2 == 0 { 1.0 } else { -1.0 };
                        
                        let b = Self::rk(ab + hg) * obm;
                        let c = (ab % 6.28318) / 3.14159 - 1.0;
                        let av = Self::lz(ab + hg) * obm;
                        
                        let aqf = 2.0 / (3.0 + av * 0.5);
                        let eo = (av + 1.0) / 2.0;
                        ((yv + b * bv * 0.5 * aqf) as i32,
                         (uq + c * bv * 0.8) as i32, eo)
                    },
                    ShapeOverlay::None => (0, 0, 0.5),
                };
                
                
                drop.lvi = drop.bj;
                drop.frd = drop.br;
                drop.bj = b;
                drop.br = c;
                drop.eo = av.qp(0.0, 1.0);
            }
        }
        
        
        
    }
    
    
    
    pub fn tj(&self, bi: &mut [u32], lu: usize, qh: usize) {
        
        let vp = 0xFF010203u32;
        #[cfg(target_arch = "x86_64")]
        unsafe {
            crate::graphics::simd::bed(bi.mw(), bi.len(), vp);
        }
        #[cfg(not(target_arch = "x86_64"))]
        bi.vi(vp);
        
        let ny = EB_;
        let qxg = lu / ny;
        let kgx = qh / ny;
        
        
        let bgo = self.eio == ShapeOverlay::Dw;
        let yv = (lu / 2) as f32;
        let uq = (qh / 2) as f32;
        let bv = (qh as f32).v(lu as f32) * 0.18;
        
        
        let aev = 0.785398_f32; 
        let ajt = 0.523599_f32; 
        let bmo = Self::rk(aev);
        let bol = Self::lz(aev);
        let bmn = Self::rk(ajt);
        let bok = Self::lz(ajt);
        
        
        let jkg = |fbv: f32, fbz: f32| -> (f32, f32) {
            let fbx = -1.0; 
            let ehw = fbv * bmo - fbz * bol;
            let cmn = fbv * bol + fbz * bmo;
            let dbp = fbx * bmn - cmn * bok;
            let cmo = fbx * bok + cmn * bmn;
            let aab = 5.0;
            let cgu = cmo + aab;
            let aqf = aab / cgu.am(1.0);
            (yv + ehw * bv * aqf, uq + dbp * bv * aqf)
        };
        
        
        let (ztd, puk) = jkg(0.0, -1.0);
        let (pup, ztf) = jkg(-1.0, 0.0);
        let (pur, ztg) = jkg(1.0, 0.0);
        let (zte, pun) = jkg(0.0, 1.0);
        
        
        
        let frj = |fbv: f32, fbx: f32, fbz: f32| -> (f32, f32) {
            let ehw = fbv * bmo - fbz * bol;
            let cmn = fbv * bol + fbz * bmo;
            let dbp = fbx * bmn - cmn * bok;
            let cmo = fbx * bok + cmn * bmn;
            let aab = 5.0;
            let cgu = cmo + aab;
            let ove = aab / cgu.am(1.0);
            (yv + ehw * bv * ove, uq + dbp * bv * ove)
        };
        
        
        
        let to = if bgo { frj(-1.0, -1.0, -1.0) } else { (0.0, 0.0) };
        let eah = if bgo { frj( 1.0, -1.0, -1.0) } else { (0.0, 0.0) };
        let hez = if bgo { frj( 1.0,  1.0, -1.0) } else { (0.0, 0.0) };
        let ged = if bgo { frj(-1.0,  1.0, -1.0) } else { (0.0, 0.0) };
        let eai = if bgo { frj(-1.0, -1.0,  1.0) } else { (0.0, 0.0) };
        let hfa = if bgo { frj(-1.0,  1.0,  1.0) } else { (0.0, 0.0) };
        
        
        
        let eoi = if bgo { frj( 1.0, -1.0,  1.0) } else { (0.0, 0.0) };
        let mlw = [to, eah, eoi, eai];
        
        let lii = [to, eai, hfa, ged];
        
        let mac = [to, eah, hez, ged];
        
        
        let kmj = if bgo { to.0.v(eah.0).v(hez.0).v(ged.0).v(eai.0).v(hfa.0).v(eoi.0) - 2.0 } else { 0.0 };
        let kmh = if bgo { to.0.am(eah.0).am(hez.0).am(ged.0).am(eai.0).am(hfa.0).am(eoi.0) + 2.0 } else { 0.0 };
        let kmk = if bgo { to.1.v(eah.1).v(hez.1).v(ged.1).v(eai.1).v(hfa.1).v(eoi.1) - 20.0 } else { 0.0 };
        let kmi = if bgo { to.1.am(eah.1).am(hez.1).am(ged.1).am(eai.1).am(hfa.1).am(eoi.1) + 2.0 } else { 0.0 };
        
        
        let rxi = if bgo { (pur - pup) / 2.0 } else { 1.0 };
        let rxh = if bgo { (pun - puk) / 2.0 } else { 1.0 };
        let ymb = (pup + pur) / 2.0;
        let ymc = (puk + pun) / 2.0;
        let yym = 1.0 / rxi.am(1.0);
        let yyl = 1.0 / rxh.am(1.0);
        
        
        
        let jkr = |fm: &[(f32, f32); 4]| -> [(f32, f32, f32, f32); 4] {
            let mut bu = [(0.0f32, 0.0f32, 0.0f32, 0.0f32); 4];
            for a in 0..4 {
                let fb = (a + 1) % 4;
                bu[a] = (fm[fb].0 - fm[a].0, fm[fb].1 - fm[a].1, fm[a].0, fm[a].1);
            }
            bu
        };
        let mlv = if bgo { jkr(&mlw) } else { [(0.0,0.0,0.0,0.0); 4] };
        let lih = if bgo { jkr(&lii) } else { [(0.0,0.0,0.0,0.0); 4] };
        let mab = if bgo { jkr(&mac) } else { [(0.0,0.0,0.0,0.0); 4] };
        
        
        
        let ov = 16.0_f32;
        let xjm = (to.0 + eah.0 + eoi.0 + eai.0) / 4.0;
        let xjn = (to.1 + eah.1 + eoi.1 + eai.1) / 4.0;
        let arb = |ai: (f32, f32)| -> (f32, f32) {
            let dx = ai.0 - xjm;
            let bg = ai.1 - xjn;
            (ai.0 + dx * 0.12 + 0.0, ai.1 + bg * 0.12 - ov * 0.5)
        };
        let vat = if bgo {
            [arb(to), arb(eah), arb(eoi), arb(eai)]
        } else {
            [(0.0,0.0); 4]
        };
        let lro = if bgo { jkr(&vat) } else { [(0.0,0.0,0.0,0.0); 4] };
        
        #[inline(always)]
        fn zfu(y: f32, x: f32, bu: &[(f32, f32, f32, f32); 4]) -> bool {
            let mut u = 0u8;
            let mut neg = 0u8;
            for &(bqp, ahm, mp, qw) in bu.iter() {
                let bjr = bqp * (x - qw) - ahm * (y - mp);
                if bjr > 0.0 { u += 1; } 
                else if bjr < 0.0 { neg += 1; }
            }
            u == 0 || neg == 0
        }
        
        
        let xjz = qxg.v(self.ajg);
        let oi = Aac {
            doo: bi.mw(),
            aic: bi.len(),
            lu,
            qh,
            ny,
            kgx,
            nnx: self.agk.fq(),
            neo: self.cwh.fq(),
            ajg: self.ajg,
            bgo,
            kmj,
            kmh,
            kmk,
            kmi,
            mlv,
            lih,
            mab,
            lro,
        };
        
        crate::cpu::smp::daj(
            xjz,
            vvl,
            &oi as *const Aac as *mut u8,
        );
        
        
        
    }
    
    
    
    pub fn vvs(&self, bi: &mut [u32], lu: usize, qh: usize) {
        if self.eio == ShapeOverlay::None {
            return;
        }
        
        let yv = (lu / 2) as f32;
        let uq = (qh / 2) as f32;
        let bv = (qh as f32).v(lu as f32) * 0.18;  
        
        
        let time = self.eip;
        
        match self.eio {
            ShapeOverlay::Dw => {
                
            },
            ShapeOverlay::Sphere => {
                
                let jho = 24;
                
                
                for a in 0..6 {
                    let bnv = (a as f32 / 6.0) * 3.14159;
                    let vlc = 0xFF00AA44;  
                    
                    let mut bwb = 0i32;
                    let mut dur = 0i32;
                    let mut fv = true;
                    
                    for fb in 0..=jho {
                        let bdb = (fb as f32 / jho as f32) * 3.14159 * 2.0;
                        
                        let b = Self::lz(bdb) * Self::rk(bnv);
                        let av = Self::lz(bdb) * Self::lz(bnv);
                        let c = Self::rk(bdb);
                        
                        
                        let jmu = time * 0.1;
                        let kb = b * Self::rk(jmu) - av * Self::lz(jmu);
                        let agv = b * Self::lz(jmu) + av * Self::rk(jmu);
                        
                        let aqf = 3.0 / (4.0 + agv);
                        let y = (yv + kb * bv * aqf) as i32;
                        let x = (uq + c * bv * aqf * 0.8) as i32;
                        
                        if !fv {
                            let eo = (agv + 1.0) / 2.0;
                            let s = if eo > 0.5 { 0xFFCCFFCC } else { vlc };  
                            self.dqg(bi, lu, qh, 
                                               bwb, dur, y, x, s, 2);
                        }
                        bwb = y;
                        dur = x;
                        fv = false;
                    }
                }
                
                
                for a in 1..4 {
                    let fby = -0.75 + (a as f32 * 0.5);
                    let dy = (1.0 - fby * fby).am(0.0);
                    let dy = {
                        let mut b = dy;
                        let mut c = dy * 0.5;
                        for _ in 0..4 { c = (c + b / c) * 0.5; }
                        c
                    };
                    
                    let mut bwb = 0i32;
                    let mut dur = 0i32;
                    let mut fv = true;
                    
                    for fb in 0..=jho {
                        let hg = (fb as f32 / jho as f32) * 3.14159 * 2.0 + time * 0.1;
                        let b = Self::rk(hg) * dy;
                        let av = Self::lz(hg) * dy;
                        
                        let aqf = 3.0 / (4.0 + av);
                        let y = (yv + b * bv * aqf) as i32;
                        let x = (uq + fby * bv * aqf * 0.8) as i32;
                        
                        if !fv {
                            let eo = (av + 1.0) / 2.0;
                            let s = if eo > 0.5 { 0xFFCCFFCC } else { 0xFF00AA44 };  
                            self.dqg(bi, lu, qh, 
                                               bwb, dur, y, x, s, 2);
                        }
                        bwb = y;
                        dur = x;
                        fv = false;
                    }
                }
            },
            ShapeOverlay::Dr => {
                
                let czl = 0.7;
                let cge = 0.3;
                let jq = 16;
                
                
                for a in 0..8 {
                    let tm = (a as f32 / 8.0) * 3.14159 * 2.0;
                    let gdw = Self::rk(tm);
                    let wvi = Self::lz(tm);
                    
                    let mut bwb = 0i32;
                    let mut dur = 0i32;
                    let mut fv = true;
                    
                    for fb in 0..=jq {
                        let p = (fb as f32 / jq as f32) * 3.14159 * 2.0;
                        let gec = Self::rk(p);
                        let bxk = Self::lz(p);
                        
                        let b = (czl + cge * gec) * gdw;
                        let c = cge * bxk;
                        let av = (czl + cge * gec) * wvi;
                        
                        
                        let dli = time * 0.2;
                        let kb = b * Self::rk(dli) - av * Self::lz(dli);
                        let agv = b * Self::lz(dli) + av * Self::rk(dli);
                        
                        let aqf = 3.0 / (4.0 + agv);
                        let y = (yv + kb * bv * aqf) as i32;
                        let x = (uq + c * bv * aqf) as i32;
                        
                        if !fv {
                            let eo = (agv + 1.0) / 2.0;
                            let s = if eo > 0.5 { 0xFFCCFFCC } else { 0xFF00AA44 };  
                            self.dqg(bi, lu, qh, 
                                               bwb, dur, y, x, s, 2);
                        }
                        bwb = y;
                        dur = x;
                        fv = false;
                    }
                }
            },
            ShapeOverlay::Ij => {
                
                let obn = bv * 1.5;
                let fkn = bv * 0.3;
                let jq = 40;
                
                let mut oxn = 0i32;
                let mut oxp = 0i32;
                let mut oxo = 0i32;
                let mut oxq = 0i32;
                let mut fv = true;
                
                for a in 0..=jq {
                    let ab = a as f32 / jq as f32;
                    let c = uq - obn / 2.0 + obn * ab;
                    let hg = ab * 3.14159 * 4.0 + time * 0.5;
                    
                    
                    let dn = yv + Self::rk(hg) * fkn;
                    let hy = yv + Self::rk(hg + 3.14159) * fkn;
                    let aeu = Self::lz(hg);
                    let ahc = Self::lz(hg + 3.14159);
                    
                    let gqb = dn as i32;
                    let egz = c as i32;
                    let gqc = hy as i32;
                    let eha = c as i32;
                    
                    if !fv {
                        let bjo = if aeu > 0.0 { 0xFFCCFFCC } else { 0xFF00AA44 };  
                        let btr = if ahc > 0.0 { 0xFFCCFFCC } else { 0xFF00AA44 };  
                        self.dqg(bi, lu, qh, 
                                           oxn, oxp, gqb, egz, bjo, 2);
                        self.dqg(bi, lu, qh, 
                                           oxo, oxq, gqc, eha, btr, 2);
                        
                        
                        if a % 4 == 0 {
                            self.dqg(bi, lu, qh, 
                                               gqb, egz, gqc, eha, 0xFF44FF44, 1);
                        }
                    }
                    oxn = gqb;
                    oxp = egz;
                    oxo = gqc;
                    oxq = eha;
                    fv = false;
                }
            },
            ShapeOverlay::None => {},
        }
    }
    
    
    
    
    
    pub fn vvo(&self, bi: &mut [u32], lu: usize, qh: usize) {
        if self.eio != ShapeOverlay::Dw {
            return;
        }
        
        let ny = EB_;
        let anv = (self.ajg * ny) as f32;
        let akr = (self.bnr * ny) as f32;
        let yv = anv / 2.0;
        let uq = akr / 2.0;
        let bv = akr.v(anv) * 0.18;
        
        let aev = 0.785398_f32;
        let ajt = 0.523599_f32;
        let bmo = Self::rk(aev);
        let bol = Self::lz(aev);
        let bmn = Self::rk(ajt);
        let bok = Self::lz(ajt);
        let aab = 5.0_f32;
        
        let nv = |fbv: f32, fbx: f32, fbz: f32| -> (f32, f32) {
            let ehw = fbv * bmo - fbz * bol;
            let cmn = fbv * bol + fbz * bmo;
            let dbp = fbx * bmn - cmn * bok;
            let cmo = fbx * bok + cmn * bmn;
            let cgu = cmo + aab;
            let aqf = aab / cgu.am(1.0);
            (yv + ehw * bv * aqf, uq + dbp * bv * aqf)
        };
        
        
        let bin = nv(-1.0, -1.0, -1.0); 
        let aax = nv( 1.0, -1.0, -1.0); 
        let aco = nv( 1.0, -1.0,  1.0); 
        let bcx = nv(-1.0, -1.0,  1.0); 
        
        
        let bvd = nv(-1.0,  1.0,  1.0);  
        let eei = nv(-1.0,  1.0, -1.0);  
        let uv = nv( 1.0,  1.0, -1.0);  
        
        let mlw = [bin, aax, aco, bcx];
        let lii = [bin, bcx, bvd, eei];
        
        let mac = [bin, aax, uv, eei];
        
        
        
        
        let pus = aax.0 - bin.0;
        let put = aax.1 - bin.1;
        let puu = bcx.0 - bin.0;
        let puv = bcx.1 - bin.1;
        let pul = pus * puv - put * puu;
        
        
        let gyg = [bin, aax, aco, bcx, bvd, eei, uv];
        let mut kcl = gyg[0].0;
        let mut kcj = gyg[0].0;
        let mut kcm = gyg[0].1;
        let mut kck = gyg[0].1;
        for ai in &gyg[1..] {
            if ai.0 < kcl { kcl = ai.0; }
            if ai.0 > kcj { kcj = ai.0; }
            if ai.1 < kcm { kcm = ai.1; }
            if ai.1 > kck { kck = ai.1; }
        }
        
        let qxi = ((kcl / ny as f32) as i32).am(0) as usize;
        let qxj = ((kcj / ny as f32) as i32 + 1).v(self.ajg as i32) as usize;
        let qxk = ((kcm / ny as f32) as i32).am(0) as usize;
        let qxl = ((kck / ny as f32) as i32 + 1).v(self.bnr as i32) as usize;
        
        let luo = |y: f32, x: f32, fm: &[(f32, f32); 4]| -> bool {
            let mut u = 0i32;
            let mut neg = 0i32;
            for a in 0..4 {
                let fb = (a + 1) % 4;
                let bjr = (fm[fb].0 - fm[a].0) * (x - fm[a].1) - (fm[fb].1 - fm[a].1) * (y - fm[a].0);
                if bjr > 0.0 { u += 1; } 
                else if bjr < 0.0 { neg += 1; }
            }
            u == 0 || neg == 0
        };
        
        let time = self.eip;
        let ors = 8.0_f32;   
        let oia = 0.35_f32; 
        
        for ae in qxk..qxl {
            for cx in qxi..qxj {
                let y = (cx * ny + RZ_) as f32;
                let x = (ae * ny + RZ_) as f32;
                
                
                if pul.gp() > 0.01 && luo(y, x, &mlw) {
                    
                    let nmq = y - bin.0;
                    let nmr = x - bin.1;
                    let off = 1.0 / pul;
                    let tm = (nmq * puv - nmr * puu) * off;
                    let p = (pus * nmr - put * nmq) * off;
                    
                    
                    let mnt = tm * ors;
                    let mos = p * ors;
                    let xnv = mnt - Self::cxs(mnt);
                    let xqb = mos - Self::cxs(mos);
                    
                    let lqn = xnv < oia; 
                    let lqo = xqb < oia; 
                    
                    if lqn || lqo {
                        let xnw = Self::cxs(mnt) as i32;
                        let xqd = Self::cxs(mos) as i32;
                        
                        
                        let eie = 2.5;
                        let mut kt: f32 = 0.0;
                        
                        if lqn {
                            
                            let dv = (xqd as u32).hx(2654435761);
                            let ib = (dv % 100) as f32 * 0.04;
                            let ale = time * eie + ib;
                            let ase = 0.4_f32;  
                            let awn = 0.7 + (dv % 3) as f32 * 0.15;
                            let bc = tm - ale;
                            let cee = bc - Self::cxs(bc / awn) * awn;
                            if cee >= 0.0 && cee < ase {
                                kt = (1.0 - cee / ase).am(0.0);
                            }
                        }
                        
                        if lqo {
                            let dv = (xnw as u32).hx(340573321);
                            let ib = (dv % 100) as f32 * 0.04;
                            let ale = time * eie * 0.9 + ib;
                            let ase = 0.4_f32;
                            let awn = 0.7 + (dv % 3) as f32 * 0.15;
                            let bc = p - ale;
                            let cee = bc - Self::cxs(bc / awn) * awn;
                            if cee >= 0.0 && cee < ase {
                                let tb = (1.0 - cee / ase).am(0.0);
                                if tb > kt { kt = tb; }
                            }
                        }
                        
                        if kt < 0.08 { kt = 0.08; }
                        
                        
                        let hcm = (cx as u32).hx(2654435761)
                            .cn((ae as u32).hx(340573321));
                        let dod = (time * 8.0) as u32;
                        let cqy = ((hcm.cn(dod)) % ND_ as u32) as usize;
                        let ka = &OC_[cqy];
                        
                        
                        let s = if lqn && lqo {
                            
                            let d = (180.0 + kt * 75.0) as u8;
                            0xFF000000 | ((d as u32) << 16) | ((d as u32) << 8) | (d as u32)
                        } else if kt > 0.85 {
                            
                            let at = (kt * 255.0) as u8;
                            0xFF000000 | 0x00300000 | ((at as u32) << 8) | 0x30
                        } else {
                            
                            let at = (30.0 + kt * 220.0) as u8;
                            0xFF000000 | ((at as u32) << 8) | 0x06
                        };
                        
                        self.hgu(bi, lu, cx * ny + 1, ae * ny + 1, ka, s);
                    }
                    continue;
                }
                
                
                if luo(y, x, &lii) {
                    let cds = 10.0_f32;
                    let ffj = y / cds;
                    let kjs = ffj - Self::cxs(ffj);
                    
                    if kjs < 0.4 {
                        let kjt = Self::cxs(ffj) as i32;
                        let dv = (kjt as u32).hx(2654435761);
                        let eie = 2.5 + (dv % 6) as f32 * 0.3;
                        let ib = (dv % 100) as f32 * 0.05;
                        let ale = time * eie + ib;
                        let u = x / cds;
                        let ase = 3.0;
                        let awn = 4.0 + (dv % 3) as f32 * 0.5;
                        let bc = u - ale;
                        let cee = bc - Self::cxs(bc / awn) * awn;
                        
                        let mut kt: f32 = 0.06;
                        if cee >= 0.0 && cee < ase {
                            kt = (1.0 - cee / ase).am(0.0);
                            if kt < 0.06 { kt = 0.06; }
                        }
                        
                        let hcm = (cx as u32).hx(2654435761)
                            .cn((ae as u32).hx(340573321));
                        let dod = (time * 6.0) as u32;
                        let cqy = ((hcm.cn(dod)) % ND_ as u32) as usize;
                        let ka = &OC_[cqy];
                        
                        let s = if kt > 0.88 {
                            let at = (kt * 200.0) as u8;
                            0xFF000000 | 0x00200000 | ((at as u32) << 8) | 0x18
                        } else {
                            let at = (12.0 + kt * 140.0) as u8;
                            0xFF000000 | ((at as u32) << 8) | 0x04
                        };
                        
                        self.hgu(bi, lu, cx * ny + 1, ae * ny + 1, ka, s);
                    }
                    continue;
                }
                
                
                if luo(y, x, &mac) {
                    let cds = 10.0_f32;
                    let ffj = y / cds;
                    let kjs = ffj - Self::cxs(ffj);
                    
                    if kjs < 0.4 {
                        let kjt = Self::cxs(ffj) as i32;
                        let dv = (kjt as u32).hx(340573321);
                        let eie = 2.8 + (dv % 5) as f32 * 0.25;
                        let ib = (dv % 100) as f32 * 0.05;
                        let ale = time * eie + ib;
                        let u = x / cds;
                        let ase = 3.5;
                        let awn = 4.5 + (dv % 3) as f32 * 0.5;
                        let bc = u - ale;
                        let cee = bc - Self::cxs(bc / awn) * awn;
                        
                        let mut kt: f32 = 0.06;
                        if cee >= 0.0 && cee < ase {
                            kt = (1.0 - cee / ase).am(0.0);
                            if kt < 0.06 { kt = 0.06; }
                        }
                        
                        let hcm = (cx as u32).hx(2654435761)
                            .cn((ae as u32).hx(340573321));
                        let dod = (time * 7.0) as u32;
                        let cqy = ((hcm.cn(dod)) % ND_ as u32) as usize;
                        let ka = &OC_[cqy];
                        
                        let s = if kt > 0.88 {
                            let at = (kt * 230.0) as u8;
                            0xFF000000 | 0x00280000 | ((at as u32) << 8) | 0x20
                        } else {
                            let at = (15.0 + kt * 170.0) as u8;
                            0xFF000000 | ((at as u32) << 8) | 0x06
                        };
                        
                        self.hgu(bi, lu, cx * ny + 1, ae * ny + 1, ka, s);
                    }
                }
            }
        }
    }
    
    
    fn dqg(&self, bi: &mut [u32], lu: usize, qh: usize,
                       fy: i32, fo: i32, dn: i32, dp: i32, s: u32, ahw: i32) {
        
        let adf = 100i32;
        let d = lu as i32;
        let i = qh as i32;
        if (fy < -adf && dn < -adf) || (fy > d + adf && dn > d + adf) {
            return;
        }
        if (fo < -adf && dp < -adf) || (fo > i + adf && dp > i + adf) {
            return;
        }
        
        let dx = (dn - fy).gp();
        let bg = -(dp - fo).gp();
        let cr = if fy < dn { 1 } else { -1 };
        let cq = if fo < dp { 1 } else { -1 };
        let mut rq = dx + bg;
        
        let mut b = fy;
        let mut c = fo;
        
        
        let csk = (dx.gp() + (-bg).gp() + 10) as usize;
        let mut au = 0usize;
        
        loop {
            au += 1;
            if au > csk { break; }
            
            
            for ty in -ahw/2..=ahw/2 {
                for gx in -ahw/2..=ahw/2 {
                    let y = b + gx;
                    let x = c + ty;
                    if y >= 0 && x >= 0 {
                        let hwf = y as usize;
                        let hwg = x as usize;
                        if hwf < lu && hwg < qh {
                            bi[hwg * lu + hwf] = s;
                        }
                    }
                }
            }
            
            if b == dn && c == dp { break; }
            
            let agl = 2 * rq;
            if agl >= bg {
                rq += bg;
                b += cr;
            }
            if agl <= dx {
                rq += dx;
                c += cq;
            }
        }
    }
    
    
    fn ynd(&self, bi: &mut [u32], lu: usize, qh: usize,
                              fy: i32, fo: i32, dn: i32, dp: i32, 
                              agg: u32, eo: f32, fhi: usize) {
        
        let adf = 100i32;
        let d = lu as i32;
        let i = qh as i32;
        if (fy < -adf && dn < -adf) || (fy > d + adf && dn > d + adf) {
            return;
        }
        if (fo < -adf && dp < -adf) || (fo > i + adf && dp > i + adf) {
            return;
        }
        
        let time = self.eip;
        
        
        let nom = (dn - fy) as f32;
        let non = (dp - fo) as f32;
        let ark = (nom * nom + non * non).am(1.0);
        let ark = {
            let mut b = ark;
            let mut c = ark * 0.5;
            for _ in 0..4 { c = (c + b / c) * 0.5; }
            c
        };
        
        let dx = (dn - fy).gp();
        let bg = -(dp - fo).gp();
        let cr = if fy < dn { 1 } else { -1 };
        let cq = if fo < dp { 1 } else { -1 };
        let mut rq = dx + bg;
        
        let mut b = fy;
        let mut c = fo;
        let mut ctj = 0usize;
        
        
        let csk = (dx.gp() + (-bg).gp() + 10) as usize;
        let mut au = 0usize;
        
        
        let oru = 3;
        
        let vof = 2.0 + (fhi as f32 * 0.3);
        
        let vog = 12.0;
        
        loop {
            au += 1;
            if au > csk { break; }
            
            
            let ab = ctj as f32 / ark.am(1.0);
            
            
            let mut frs = 0.0f32;
            for ai in 0..oru {
                
                let vod = (time * vof + ai as f32 / oru as f32) % 1.0;
                let lwe = vod;
                
                
                let ryy = (ab - lwe).gp();
                let dgk = (ab - lwe - 1.0).gp();
                let ryz = (ab - lwe + 1.0).gp();
                let la = ryy.v(dgk).v(ryz);
                
                
                let lwf = la * ark / vog;
                if lwf < 1.0 {
                    frs += (1.0 - lwf * lwf).am(0.0);
                }
            }
            frs = frs.v(1.0);
            
            
            let (bdm, bji, cdd) = (
                ((agg >> 16) & 0xFF) as f32,
                ((agg >> 8) & 0xFF) as f32,
                (agg & 0xFF) as f32,
            );
            
            
            let voe = 220.0;
            let voc = 255.0;
            let vob = 220.0;
            
            let m = (bdm + (voe - bdm) * frs) as u32;
            let at = (bji + (voc - bji) * frs) as u32;
            let o = (cdd + (vob - cdd) * frs) as u32;
            let s = 0xFF000000 | (m << 16) | (at << 8) | o;
            
            
            let ahw = if frs > 0.3 { 2 } else { 1 };
            for ty in -ahw/2..=ahw/2 {
                for gx in -ahw/2..=ahw/2 {
                    let y = b + gx;
                    let oyp = c + ty;
                    if y >= 0 && oyp >= 0 {
                        let hwf = y as usize;
                        let hwg = oyp as usize;
                        if hwf < lu && hwg < qh {
                            bi[hwg * lu + hwf] = s;
                        }
                    }
                }
            }
            
            ctj += 1;
            
            if b == dn && c == dp { break; }
            
            let agl = 2 * rq;
            if agl >= bg {
                rq += bg;
                b += cr;
            }
            if agl <= dx {
                rq += dx;
                c += cq;
            }
        }
    }
    
    
    #[inline(always)]
    fn hgu(&self, bi: &mut [u32], lu: usize,
                       y: usize, x: usize, ka: &[u8; 3], s: u32) {
        let qh = bi.len() / lu;
        if x + 2 >= qh || y + 2 >= lu { return; }
        
        let wu = ka[0];
        let bpq = x * lu + y;
        if wu & 0b001 != 0 { bi[bpq] = s; }
        if wu & 0b010 != 0 { bi[bpq + 1] = s; }
        if wu & 0b100 != 0 { bi[bpq + 2] = s; }
        
        let of = ka[1];
        let bpr = bpq + lu;
        if of & 0b001 != 0 { bi[bpr] = s; }
        if of & 0b010 != 0 { bi[bpr + 1] = s; }
        if of & 0b100 != 0 { bi[bpr + 2] = s; }
        
        let tb = ka[2];
        let deb = bpr + lu;
        if tb & 0b001 != 0 { bi[deb] = s; }
        if tb & 0b010 != 0 { bi[deb + 1] = s; }
        if tb & 0b100 != 0 { bi[deb + 2] = s; }
    }

    
    #[inline(always)]
    fn sdk(&self, bi: &mut [u32], lu: usize, 
                       y: usize, x: usize, ka: &[u8; 6], s: u32) {
        let qh = bi.len() / lu;
        if x >= qh || y >= lu { return; }
        let efh = (qh - x).v(6);
        let bkj = (lu - y).v(6);
        for br in 0..efh {
            let chk = ka[br];
            if chk == 0 { continue; }
            let mu = (x + br) * lu + y;
            if chk & 0b000001 != 0 && 0 < bkj { bi[mu] = s; }
            if chk & 0b000010 != 0 && 1 < bkj { bi[mu + 1] = s; }
            if chk & 0b000100 != 0 && 2 < bkj { bi[mu + 2] = s; }
            if chk & 0b001000 != 0 && 3 < bkj { bi[mu + 3] = s; }
            if chk & 0b010000 != 0 && 4 < bkj { bi[mu + 4] = s; }
            if chk & 0b100000 != 0 && 5 < bkj { bi[mu + 5] = s; }
        }
    }
}






pub struct FastMatrixRenderer {
    
    ecu: Vec<i32>,
    
    arz: Vec<u8>,
    
    bw: Vec<u8>,
    
    frame: u32,
    
    ec: usize,
    lk: usize,
}

impl FastMatrixRenderer {
    pub fn new() -> Self {
        
        let ec = 1280 / ZZ_; 
        let lk = 800 / ZY_;  
        
        let mut ecu = vec![0i32; ec];
        let mut arz = vec![1u8; ec];
        let mut bw = vec![0u8; ec * lk];
        
        
        for a in 0..ec {
            let dv = (a as u32).hx(2654435761) ^ 0xDEADBEEF;
            ecu[a] = -((dv % (lk as u32 * 2)) as i32);
            arz[a] = 1 + (dv % 2) as u8;
            
            
            for fb in 0..lk {
                let des = dv.hx((fb + 1) as u32);
                bw[a * lk + fb] = AZH_[(des as usize) % AZH_.len()];
            }
        }
        
        Self { ecu, arz, bw, frame: 0, ec, lk }
    }
    
    
    pub fn qs(&mut self) {
        self.frame = self.frame.cn(1);
        
        for a in 0..self.ec {
            self.ecu[a] += self.arz[a] as i32;
            
            
            if self.ecu[a] > (self.lk as i32) + 20 {
                let dv = (a as u32).hx(self.frame).cn(0xBEEF);
                self.ecu[a] = -((dv % 30) as i32);
            }
        }
    }
    
    
    pub fn tj(&self, bi: &mut [u32], lu: usize, qh: usize) {
        let ec = (lu / ZZ_).v(self.ec);
        let lk = (qh / ZY_).v(self.lk);
        
        for bj in 0..ec {
            let buu = self.ecu[bj];
            
            for br in 0..lk {
                let la = buu - (br as i32);
                
                if la >= 0 && la < 16 {
                    
                    let hj = ADN_[(la as usize).v(31)];
                    if hj > 10 {
                        
                        let r = self.bw[bj * self.lk + br];
                        
                        
                        let s = (0xFF << 24) | ((hj as u32) << 8);
                        
                        
                        self.ahi(bi, lu, bj, br, r, s);
                    }
                }
            }
        }
    }
    
    
    fn ahi(&self, bi: &mut [u32], lu: usize, 
                 bj: usize, br: usize, r: u8, s: u32) {
        let y = bj * ZZ_;
        let x = br * ZY_;
        
        let ka = crate::framebuffer::font::ada(r as char);
        
        for (ub, &fs) in ka.iter().cf() {
            let c = x + ub;
            if c >= bi.len() / lu { continue; }
            
            for qz in 0..8 {
                if (fs >> (7 - qz)) & 1 != 0 {
                    let b = y + qz;
                    let w = c * lu + b;
                    if w < bi.len() {
                        bi[w] = s;
                    }
                }
            }
        }
    }
}






const AEZ_: usize = 200;


#[derive(Clone, Copy)]
struct Drop3D {
    
    b: f32,
    
    c: f32,
    
    av: f32,
    
    fp: f32,
    
    iz: f32,
    
    ciq: f32,
    
    acr: u8,
    
    amg: u32,
    
    clp: bool,
    
    ebs: u8,
}

impl Drop3D {
    fn new() -> Self {
        Self {
            b: 0.0, c: -10.0, av: 0.5,
            fp: 0.0, iz: 0.5, ciq: 0.0,
            acr: 20,
            amg: 0,
            clp: false,
            ebs: 0,
        }
    }
}


#[derive(Clone, Copy)]
pub enum Shape3D {
    
    Sphere { cx: f32, ae: f32, zr: f32, m: f32 },
    
    Dw { cx: f32, ae: f32, zr: f32, iv: f32, dli: f32 },
    
    Dr { cx: f32, ae: f32, zr: f32, Ac: f32, m: f32 },
}


pub struct Matrix3D {
    
    agk: [Drop3D; AEZ_],
    
    chy: [Option<Shape3D>; 4],
    
    rng: u32,
    
    frame: u32,
    
    time: f32,
    
    z: usize,
    ac: usize,
}

impl Matrix3D {
    pub fn new() -> Self {
        let mut agk = [Drop3D::new(); AEZ_];
        let mut rng = 0xDEADBEEFu32;
        
        
        for drop in agk.el() {
            rng = rng.hx(1103515245).cn(12345);
            drop.b = (rng % 160) as f32;
            rng = rng.hx(1103515245).cn(12345);
            drop.c = -((rng % 120) as f32);
            rng = rng.hx(1103515245).cn(12345);
            drop.av = 0.2 + (rng % 80) as f32 / 100.0; 
            
            
            drop.iz = 0.3 + drop.av * 0.7; 
            
            rng = rng.hx(1103515245).cn(12345);
            drop.acr = 10 + (rng % 30) as u8;
            drop.amg = rng;
        }
        
        
        let chy = [
            Some(Shape3D::Sphere { cx: 80.0, ae: 50.0, zr: 0.5, m: 15.0 }),
            None,
            None,
            None,
        ];
        
        Self {
            agk,
            chy,
            rng,
            frame: 0,
            time: 0.0,
            z: 1280,
            ac: 800,
        }
    }
    
    
    pub fn yeo(&mut self, chx: Shape3D) {
        for gk in self.chy.el() {
            if gk.is_none() {
                *gk = Some(chx);
                return;
            }
        }
    }
    
    
    pub fn yin(&mut self) {
        for gk in self.chy.el() {
            *gk = None;
        }
    }
    
    
    #[inline(always)]
    fn lz(b: f32) -> f32 { crate::math::lz(b) }
    
    
    #[inline(always)]
    fn rk(b: f32) -> f32 { crate::math::rk(b) }
    
    
    fn qyn(chy: &[Option<Shape3D>; 4], b: f32, c: f32, av: f32) -> Option<(f32, f32, f32)> {
        for chx in chy.iter().kwb(|e| *e) {
            match chx {
                Shape3D::Sphere { cx, ae, zr, m } => {
                    let dx = b - cx;
                    let bg = c - ae;
                    let pt = av - zr;
                    let ass = dx * dx + bg * bg + pt * pt;
                    let bwl = m * m;
                    
                    if ass < bwl {
                        
                        let la = Self::ahn(ass).am(0.01);
                        return Some((dx / la, bg / la, pt / la));
                    }
                }
                Shape3D::Dw { cx, ae, zr, iv, dli } => {
                    
                    let dx = b - cx;
                    let bg = c - ae;
                    let pt = av - zr;
                    
                    let cwr = Self::rk(dli);
                    let dcb = Self::lz(dli);
                    
                    
                    let kb = dx * cwr - pt * dcb;
                    let ix = bg;
                    let agv = dx * dcb + pt * cwr;
                    
                    
                    if kb.gp() < iv && ix.gp() < iv && agv.gp() < iv {
                        
                        let ax = iv - kb.gp();
                        let bga = iv - ix.gp();
                        let gzl = iv - agv.gp();
                        
                        
                        if ax < bga && ax < gzl {
                            let vt = if kb > 0.0 { 1.0 } else { -1.0 };
                            return Some((vt * cwr, 0.0, vt * dcb));
                        } else if bga < gzl {
                            return Some((0.0, if ix > 0.0 { 1.0 } else { -1.0 }, 0.0));
                        } else {
                            let arn = if agv > 0.0 { 1.0 } else { -1.0 };
                            return Some((-arn * dcb, 0.0, arn * cwr));
                        }
                    }
                }
                Shape3D::Dr { cx, ae, zr, Ac, m } => {
                    let dx = b - cx;
                    let bg = c - ae;
                    let pt = av - zr;
                    
                    
                    let hgl = Self::ahn(dx * dx + pt * pt);
                    
                    let juk = hgl - Ac;
                    let juj = Self::ahn(juk * juk + bg * bg);
                    
                    if juj < m {
                        
                        let rxs = if hgl > 0.01 { dx / hgl } else { 1.0 };
                        let rxt = if hgl > 0.01 { pt / hgl } else { 0.0 };
                        
                        let vt = juk * rxs / juj.am(0.01);
                        let ahr = bg / juj.am(0.01);
                        let arn = juk * rxt / juj.am(0.01);
                        
                        return Some((vt, ahr, arn));
                    }
                }
            }
        }
        None
    }
    
    
    #[inline(always)]
    fn ahn(b: f32) -> f32 { crate::math::ahn(b) }
    
    
    pub fn qs(&mut self) {
        self.frame = self.frame.cn(1);
        self.time += 0.02; 
        
        
        if let Some(Shape3D::Dw { ref mut dli, .. }) = self.chy.ds(0).and_then(|e| e.as_mut()) {
            *dli = self.time;
        }
        
        let lai = 0.02f32;
        
        
        let chy = self.chy;
        
        for a in 0..AEZ_ {
            let drop = &mut self.agk[a];
            
            
            if let Some((vt, ahr, arn)) = Self::qyn(&chy, drop.b, drop.c, drop.av) {
                if !drop.clp {
                    
                    drop.clp = true;
                    drop.ebs = 0;
                    
                    
                    
                    let amb = drop.fp * vt + drop.iz * ahr + drop.ciq * arn;
                    drop.fp -= amb * vt;
                    drop.iz -= amb * ahr;
                    drop.ciq -= amb * arn;
                    
                    
                    let nzn = ahr; 
                    drop.iz += lai * (1.0 - nzn * nzn).am(0.0);
                }
                
                
                drop.b += vt * 0.2;
                drop.c += ahr * 0.2;
                drop.av += arn * 0.2;
                
                
                drop.fp *= 0.95;
                drop.iz *= 0.95;
                drop.ciq *= 0.95;
                
                
                drop.iz += lai * 0.5;
                
                drop.ebs += 1;
            } else {
                
                drop.clp = false;
                drop.iz += lai;
            }
            
            
            drop.b += drop.fp;
            drop.c += drop.iz;
            drop.av += drop.ciq;
            
            
            drop.av = drop.av.qp(0.1, 1.0);
            
            
            let apa = drop.c > 110.0 || drop.b < -5.0 || drop.b > 165.0 || drop.ebs > 100;
            
            if apa {
                self.rng = self.rng.hx(1103515245).cn(12345);
                drop.b = (self.rng % 160) as f32;
                self.rng = self.rng.hx(1103515245).cn(12345);
                drop.c = -((self.rng % 40) as f32);
                self.rng = self.rng.hx(1103515245).cn(12345);
                drop.av = 0.2 + (self.rng % 80) as f32 / 100.0;
                
                drop.fp = 0.0;
                drop.iz = 0.3 + drop.av * 0.7;
                drop.ciq = 0.0;
                drop.clp = false;
                drop.ebs = 0;
                drop.amg = self.rng;
            }
            
            
            drop.amg = drop.amg.hx(1103515245).cn(12345);
        }
    }
    
    
    #[inline(always)]
    fn nv(&self, b: f32, c: f32, av: f32) -> (i32, i32, f32) {
        
        
        let bv = 0.7 + av * 0.3; 
        let yv = 80.0;
        let uq = 50.0;
        
        
        let xu = yv + (b - yv) * bv;
        let abi = uq + (c - uq) * bv;
        
        (xu as i32, abi as i32, av)
    }
    
    
    pub fn tj(&self, bi: &mut [u32], lu: usize, qh: usize) {
        
        let vp = 0xFF010201u32;
        bi.vi(vp);
        
        let ny = EB_;
        
        
        
        
        for drop in self.agk.iter() {
            if drop.c < -5.0 { continue; }
            
            let (xu, abi, eo) = self.nv(drop.b, drop.c, drop.av);
            
            
            let hfu = (50.0 + eo * 205.0) as u32;
            
            
            let wwb = if drop.clp { 30u32 } else { 0 };
            
            
            let acr = drop.acr as usize;
            for cuv in 0..acr {
                
                let ty = drop.c - cuv as f32 * 0.8;
                if ty < 0.0 { continue; }
                
                let (gx, mnq, _) = self.nv(drop.b, ty, drop.av);
                
                if gx < 0 || gx >= (lu / ny) as i32 { continue; }
                if mnq < 0 || mnq >= (qh / ny) as i32 { continue; }
                
                
                let lfa = (cuv * 63) / acr.am(1);
                let gzv = ADN_[lfa.v(63)] as u32;
                let hj = (((gzv * hfu) / 255) + wwb).v(255) as u8;
                
                if hj < 2 { continue; }
                
                
                let amg = drop.amg.cn(cuv as u32 * 2654435761);
                let cqy = (amg % ND_ as u32) as usize;
                let ka = &OC_[cqy];
                let s = oex(hj);
                
                let y = gx as usize * ny + 1;
                let x = mnq as usize * ny + 1;
                self.hgu(bi, lu, y, x, ka, s);
            }
        }
        
        
        
    }
    
    
    #[inline(always)]
    fn hgu(&self, bi: &mut [u32], lu: usize,
                       y: usize, x: usize, ka: &[u8; 3], s: u32) {
        let qh = bi.len() / lu;
        if x + 2 >= qh || y + 2 >= lu { return; }
        let wu = ka[0]; let bpq = x * lu + y;
        if wu & 0b001 != 0 { bi[bpq] = s; }
        if wu & 0b010 != 0 { bi[bpq + 1] = s; }
        if wu & 0b100 != 0 { bi[bpq + 2] = s; }
        let of = ka[1]; let bpr = bpq + lu;
        if of & 0b001 != 0 { bi[bpr] = s; }
        if of & 0b010 != 0 { bi[bpr + 1] = s; }
        if of & 0b100 != 0 { bi[bpr + 2] = s; }
        let tb = ka[2]; let deb = bpr + lu;
        if tb & 0b001 != 0 { bi[deb] = s; }
        if tb & 0b010 != 0 { bi[deb + 1] = s; }
        if tb & 0b100 != 0 { bi[deb + 2] = s; }
    }
    
    
    #[inline(always)]
    fn sdk(&self, bi: &mut [u32], lu: usize, 
                       y: usize, x: usize, ka: &[u8; 6], s: u32) {
        let qh = bi.len() / lu;
        if x >= qh || y >= lu { return; }
        let efh = (qh - x).v(6);
        let bkj = (lu - y).v(6);
        for br in 0..efh {
            let chk = ka[br];
            if chk == 0 { continue; }
            let mu = (x + br) * lu + y;
            if chk & 0b000001 != 0 && 0 < bkj { bi[mu] = s; }
            if chk & 0b000010 != 0 && 1 < bkj { bi[mu + 1] = s; }
            if chk & 0b000100 != 0 && 2 < bkj { bi[mu + 2] = s; }
            if chk & 0b001000 != 0 && 3 < bkj { bi[mu + 3] = s; }
            if chk & 0b010000 != 0 && 4 < bkj { bi[mu + 4] = s; }
            if chk & 0b100000 != 0 && 5 < bkj { bi[mu + 5] = s; }
        }
    }
    
    
    pub fn wir(&mut self) {
        self.chy = [
            
            Some(Shape3D::Sphere { cx: 80.0, ae: 50.0, zr: 0.5, m: 18.0 }),
            None,
            None,
            None,
        ];
    }
    
    
    pub fn win(&mut self) {
        self.chy = [
            Some(Shape3D::Dw { cx: 80.0, ae: 50.0, zr: 0.5, iv: 12.0, dli: self.time }),
            None,
            None,
            None,
        ];
    }
    
    
    pub fn wjs(&mut self) {
        self.chy = [
            Some(Shape3D::Dr { cx: 80.0, ae: 50.0, zr: 0.5, Ac: 15.0, m: 5.0 }),
            None,
            None,
            None,
        ];
    }
}
