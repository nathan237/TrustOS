




const By: f32 = 3.14159265;
const Kr: f32 = 6.28318530;


#[inline(always)]
pub fn jrl(x: f32) -> f32 {
    let mut a = x % Kr;
    while a > By { a -= Kr; }
    while a < -By { a += Kr; }
    a
}


#[inline(always)]
pub fn eu(x: f32) -> f32 {
    let x = jrl(x);
    let x2 = x * x;
    let x3 = x * x2;
    let cfo = x3 * x2;
    let csy = cfo * x2;
    x - x3 / 6.0 + cfo / 120.0 - csy / 5040.0
}


#[inline(always)]
pub fn hr(x: f32) -> f32 {
    eu(x + 1.5707963)
}


#[inline(always)]
pub fn hxv(x: f32) -> f32 {
    let c = hr(x);
    if c.abs() < 0.0001 { return 99999.0; }
    eu(x) / c
}


#[inline(always)]
pub fn ra(x: f32) -> f32 {
    if x <= 0.0 { return 0.0; }
    let mut g = x * 0.5;
    g = 0.5 * (g + x / g);
    g = 0.5 * (g + x / g);
    0.5 * (g + x / g)
}


#[inline(always)]
pub fn emg(x: f32) -> f32 {
    if x <= 0.0 { return 0.0; }
    let jrz = 0.5 * x;
    let i = unsafe { core::mem::transmute::<f32, u32>(x) };
    let i = 0x5f375a86u32.wrapping_sub(i >> 1);
    let y = unsafe { core::mem::transmute::<u32, f32>(i) };
    let y = y * (1.5 - jrz * y * y);
    y * (1.5 - jrz * y * y)
}


#[inline(always)]
pub fn emf(y: f32, x: f32) -> f32 {
    let dgz = zx(x);
    let dha = zx(y);
    let sh = if dgz > dha { dgz } else { dha };
    let duj = if dgz < dha { dgz } else { dha };

    if sh < 0.0001 { return 0.0; }

    let a = duj / sh;
    let j = a * a;
    let r = ((-0.0464964749 * j + 0.15931422) * j - 0.327622764) * j * a + a;

    let r = if dha > dgz { 1.5707963 - r } else { r };
    let r = if x < 0.0 { By - r } else { r };
    if y < 0.0 { -r } else { r }
}


#[inline(always)]
pub fn bbo(x: f32) -> f32 {
    if x < -6.0 { return 0.0; }
    if x > 0.0 { return 1.0; }
    let t = 1.0 + x * 0.125;
    let t = if t < 0.0 { 0.0 } else { t };
    let t = t * t; 
    let t = t * t; 
    t * t           
}


#[inline(always)]
pub fn zx(x: f32) -> f32 {
    if x < 0.0 { -x } else { x }
}


#[inline(always)]
pub fn clamp(x: f32, min: f32, max: f32) -> f32 {
    if x < min { min } else if x > max { max } else { x }
}


#[inline(always)]
pub fn pzt(x: i32, min: i32, max: i32) -> i32 {
    if x < min { min } else if x > max { max } else { x }
}


#[inline(always)]
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}


#[inline(always)]
pub fn esw(a: u8, b: u8, t: f32) -> u8 {
    ((a as f32) * (1.0 - t) + (b as f32) * t) as u8
}


#[inline(always)]
pub fn lxa(x: f32) -> i32 {
    let i = x as i32;
    if (i as f32) > x { i - 1 } else { i }
}


#[inline(always)]
pub fn fract(x: f32) -> f32 {
    x - (lxa(x) as f32)
}
