




const Eu: f32 = 3.14159265;
const Yy: f32 = 6.28318530;


#[inline(always)]
pub fn pzv(b: f32) -> f32 {
    let mut q = b % Yy;
    while q > Eu { q -= Yy; }
    while q < -Eu { q += Yy; }
    q
}


#[inline(always)]
pub fn lz(b: f32) -> f32 {
    let b = pzv(b);
    let hy = b * b;
    let ajr = b * hy;
    let fbw = ajr * hy;
    let fyz = fbw * hy;
    b - ajr / 6.0 + fbw / 120.0 - fyz / 5040.0
}


#[inline(always)]
pub fn rk(b: f32) -> f32 {
    lz(b + 1.5707963)
}


#[inline(always)]
pub fn nsw(b: f32) -> f32 {
    let r = rk(b);
    if r.gp() < 0.0001 { return 99999.0; }
    lz(b) / r
}


#[inline(always)]
pub fn ahn(b: f32) -> f32 {
    if b <= 0.0 { return 0.0; }
    let mut at = b * 0.5;
    at = 0.5 * (at + b / at);
    at = 0.5 * (at + b / at);
    0.5 * (at + b / at)
}


#[inline(always)]
pub fn iua(b: f32) -> f32 {
    if b <= 0.0 { return 0.0; }
    let qar = 0.5 * b;
    let a = unsafe { core::mem::transmute::<f32, u32>(b) };
    let a = 0x5f375a86u32.nj(a >> 1);
    let c = unsafe { core::mem::transmute::<u32, f32>(a) };
    let c = c * (1.5 - qar * c * c);
    c * (1.5 - qar * c * c)
}


#[inline(always)]
pub fn itz(c: f32, b: f32) -> f32 {
    let gxm = axv(b);
    let gxn = axv(c);
    let aki = if gxm > gxn { gxm } else { gxn };
    let hro = if gxm < gxn { gxm } else { gxn };

    if aki < 0.0001 { return 0.0; }

    let q = hro / aki;
    let e = q * q;
    let m = ((-0.0464964749 * e + 0.15931422) * e - 0.327622764) * e * q + q;

    let m = if gxn > gxm { 1.5707963 - m } else { m };
    let m = if b < 0.0 { Eu - m } else { m };
    if c < 0.0 { -m } else { m }
}


#[inline(always)]
pub fn cxr(b: f32) -> f32 {
    if b < -6.0 { return 0.0; }
    if b > 0.0 { return 1.0; }
    let ab = 1.0 + b * 0.125;
    let ab = if ab < 0.0 { 0.0 } else { ab };
    let ab = ab * ab; 
    let ab = ab * ab; 
    ab * ab           
}


#[inline(always)]
pub fn axv(b: f32) -> f32 {
    if b < 0.0 { -b } else { b }
}


#[inline(always)]
pub fn qp(b: f32, v: f32, am: f32) -> f32 {
    if b < v { v } else if b > am { am } else { b }
}


#[inline(always)]
pub fn yif(b: i32, v: i32, am: i32) -> i32 {
    if b < v { v } else if b > am { am } else { b }
}


#[inline(always)]
pub fn csb(q: f32, o: f32, ab: f32) -> f32 {
    q + (o - q) * ab
}


#[inline(always)]
pub fn jdi(q: u8, o: u8, ab: f32) -> u8 {
    ((q as f32) * (1.0 - ab) + (o as f32) * ab) as u8
}


#[inline(always)]
pub fn suw(b: f32) -> i32 {
    let a = b as i32;
    if (a as f32) > b { a - 1 } else { a }
}


#[inline(always)]
pub fn ivp(b: f32) -> f32 {
    b - (suw(b) as f32)
}
