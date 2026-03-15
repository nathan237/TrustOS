




use alloc::vec::Vec;
use alloc::boxed::Box;
use spin::Mutex;
use micromath::Wo;

use super::render2d::Color2D;






pub const DNZ_: u32 = 0x0DE0;
pub const CW_: u32 = 0x0DE1;


pub const ATM_: u32 = 0x2800;
pub const ATN_: u32 = 0x2801;
pub const BYA_: u32 = 0x2802;
pub const BYB_: u32 = 0x2803;


pub const ATJ_: u32 = 0x2600;
pub const ATI_: u32 = 0x2601;
pub const DNX_: u32 = 0x2700;
pub const DNV_: u32 = 0x2701;
pub const BXX_: u32 = 0x2702;
pub const DNU_: u32 = 0x2703;


pub const ATL_: u32 = 0x2901;
pub const BXP_: u32 = 0x2900;
pub const BXQ_: u32 = 0x812F;
pub const BXW_: u32 = 0x8370;


pub const BXZ_: u32 = 0x1907;
pub const ACZ_: u32 = 0x1908;
pub const BXV_: u32 = 0x1909;
pub const DNW_: u32 = 0x190A;






#[derive(Clone)]
pub struct TextureLevel {
    pub z: u32,
    pub ac: u32,
    pub f: Vec<u32>, 
}

impl TextureLevel {
    pub fn new(z: u32, ac: u32) -> Self {
        Self {
            z,
            ac,
            f: alloc::vec![0u32; (z * ac) as usize],
        }
    }
    
    
    #[inline]
    pub fn ehy(&self, b: u32, c: u32) -> u32 {
        let b = b.v(self.z.ao(1));
        let c = c.v(self.ac.ao(1));
        self.f[(c * self.z + b) as usize]
    }
    
    
    pub fn wck(&self, tm: f32, p: f32) -> u32 {
        let b = tm * (self.z as f32 - 1.0);
        let c = p * (self.ac as f32 - 1.0);
        
        let fy = b.hjw() as u32;
        let fo = c.hjw() as u32;
        let dn = (fy + 1).v(self.z - 1);
        let dp = (fo + 1).v(self.ac - 1);
        
        let jf = b.ivp();
        let sc = c.ivp();
        
        let byn = self.ehy(fy, fo);
        let cdn = self.ehy(dn, fo);
        let byo = self.ehy(fy, dp);
        let cdo = self.ehy(dn, dp);
        
        
        let m = Self::ils(
            ((byn >> 16) & 0xFF) as f32,
            ((cdn >> 16) & 0xFF) as f32,
            ((byo >> 16) & 0xFF) as f32,
            ((cdo >> 16) & 0xFF) as f32,
            jf, sc
        ) as u32;
        let at = Self::ils(
            ((byn >> 8) & 0xFF) as f32,
            ((cdn >> 8) & 0xFF) as f32,
            ((byo >> 8) & 0xFF) as f32,
            ((cdo >> 8) & 0xFF) as f32,
            jf, sc
        ) as u32;
        let o = Self::ils(
            (byn & 0xFF) as f32,
            (cdn & 0xFF) as f32,
            (byo & 0xFF) as f32,
            (cdo & 0xFF) as f32,
            jf, sc
        ) as u32;
        let q = Self::ils(
            ((byn >> 24) & 0xFF) as f32,
            ((cdn >> 24) & 0xFF) as f32,
            ((byo >> 24) & 0xFF) as f32,
            ((cdo >> 24) & 0xFF) as f32,
            jf, sc
        ) as u32;
        
        (q << 24) | (m << 16) | (at << 8) | o
    }
    
    #[inline]
    fn ils(byn: f32, cdn: f32, byo: f32, cdo: f32, jf: f32, sc: f32) -> f32 {
        let acw = byn + (cdn - byn) * jf;
        let rw = byo + (cdo - byo) * jf;
        acw + (rw - acw) * sc
    }
}


pub struct Texture {
    pub ad: u32,
    pub cd: u32,
    pub diw: Vec<TextureLevel>,
    pub lkd: u32,
    pub onh: u32,
    pub mqy: u32,
    pub mqz: u32,
}

impl Texture {
    pub fn new(ad: u32) -> Self {
        Self {
            ad,
            cd: CW_,
            diw: Vec::new(),
            lkd: ATI_,
            onh: BXX_,
            mqy: ATL_,
            mqz: ATL_,
        }
    }
    
    
    pub fn mof(&mut self, z: u32, ac: u32, format: u32, f: &[u8]) {
        let mut jy = TextureLevel::new(z, ac);
        
        match format {
            ACZ_ => {
                for (a, jj) in f.btq(4).cf() {
                    if jj.len() == 4 && a < jy.f.len() {
                        jy.f[a] = ((jj[3] as u32) << 24) 
                                      | ((jj[0] as u32) << 16) 
                                      | ((jj[1] as u32) << 8) 
                                      | (jj[2] as u32);
                    }
                }
            }
            BXZ_ => {
                for (a, jj) in f.btq(3).cf() {
                    if jj.len() == 3 && a < jy.f.len() {
                        jy.f[a] = 0xFF000000 
                                      | ((jj[0] as u32) << 16) 
                                      | ((jj[1] as u32) << 8) 
                                      | (jj[2] as u32);
                    }
                }
            }
            BXV_ => {
                for (a, &fni) in f.iter().cf() {
                    if a < jy.f.len() {
                        jy.f[a] = 0xFF000000 
                                      | ((fni as u32) << 16) 
                                      | ((fni as u32) << 8) 
                                      | (fni as u32);
                    }
                }
            }
            _ => {}
        }
        
        self.diw.clear();
        self.diw.push(jy);
    }
    
    
    pub fn tch(&mut self) {
        if self.diw.is_empty() {
            return;
        }
        
        let mut d = self.diw[0].z / 2;
        let mut i = self.diw[0].ac / 2;
        
        while d >= 1 && i >= 1 {
            let vo = &self.diw[self.diw.len() - 1];
            let mut jy = TextureLevel::new(d, i);
            
            
            for c in 0..i {
                for b in 0..d {
                    let cr = b * 2;
                    let cq = c * 2;
                    
                    let byn = vo.ehy(cr, cq);
                    let cdn = vo.ehy(cr + 1, cq);
                    let byo = vo.ehy(cr, cq + 1);
                    let cdo = vo.ehy(cr + 1, cq + 1);
                    
                    let m = (((byn >> 16) & 0xFF) + ((cdn >> 16) & 0xFF) 
                           + ((byo >> 16) & 0xFF) + ((cdo >> 16) & 0xFF)) / 4;
                    let at = (((byn >> 8) & 0xFF) + ((cdn >> 8) & 0xFF) 
                           + ((byo >> 8) & 0xFF) + ((cdo >> 8) & 0xFF)) / 4;
                    let o = ((byn & 0xFF) + (cdn & 0xFF) 
                           + (byo & 0xFF) + (cdo & 0xFF)) / 4;
                    let q = (((byn >> 24) & 0xFF) + ((cdn >> 24) & 0xFF) 
                           + ((byo >> 24) & 0xFF) + ((cdo >> 24) & 0xFF)) / 4;
                    
                    jy.f[(c * d + b) as usize] = (q << 24) | (m << 16) | (at << 8) | o;
                }
            }
            
            self.diw.push(jy);
            
            if d == 1 && i == 1 { break; }
            d = d.am(1) / 2;
            i = i.am(1) / 2;
            if d == 0 { d = 1; }
            if i == 0 { i = 1; }
        }
    }
    
    
    pub fn yr(&self, mut tm: f32, mut p: f32) -> u32 {
        if self.diw.is_empty() {
            return 0xFFFFFFFF;
        }
        
        
        tm = self.mwc(tm, self.mqy);
        p = self.mwc(p, self.mqz);
        
        let jy = &self.diw[0]; 
        
        match self.lkd {
            ATJ_ => {
                let b = (tm * jy.z as f32) as u32;
                let c = (p * jy.ac as f32) as u32;
                jy.ehy(b, c)
            }
            _ => jy.wck(tm, p),
        }
    }
    
    fn mwc(&self, dff: f32, ev: u32) -> f32 {
        match ev {
            BXP_ | BXQ_ => dff.qp(0.0, 1.0),
            BXW_ => {
                let a = dff.hjw() as i32;
                let bb = dff.ivp();
                if a % 2 == 0 { bb } else { 1.0 - bb }
            }
            _ => { 
                let bb = dff.ivp();
                if bb < 0.0 { 1.0 + bb } else { bb }
            }
        }
    }
}






pub struct TextureState {
    cnd: Vec<Option<Texture>>,
    bcb: u32,
    emy: Option<u32>,
    idi: bool,
}

impl TextureState {
    pub const fn new() -> Self {
        Self {
            cnd: Vec::new(),
            bcb: 1,
            emy: None,
            idi: false,
        }
    }
}

static ES_: Mutex<TextureState> = Mutex::new(TextureState::new());






pub fn tfs(bo: i32, cnd: &mut [u32]) {
    let mut g = ES_.lock();
    for a in 0..(bo as usize) {
        if a < cnd.len() {
            let ad = g.bcb;
            cnd[a] = ad;
            g.cnd.push(Some(Texture::new(ad)));
            g.bcb += 1;
        }
    }
}


pub fn nyv(cd: u32, texture: u32) {
    let mut g = ES_.lock();
    if cd == CW_ {
        g.emy = if texture == 0 { None } else { Some(texture) };
    }
}


pub fn nzb(cd: u32, dkq: u32, evz: u32) {
    let mut g = ES_.lock();
    
    let ezp = match cd {
        CW_ => g.emy,
        _ => None,
    };
    
    if let Some(ad) = ezp {
        if let Some(Some(dco)) = g.cnd.el().du(|ab| ab.as_ref().map(|b| b.ad) == Some(ad)) {
            match dkq {
                ATM_ => dco.lkd = evz,
                ATN_ => dco.onh = evz,
                BYA_ => dco.mqy = evz,
                BYB_ => dco.mqz = evz,
                _ => {}
            }
        }
    }
}


pub fn tfu(
    ydi: u32,
    yal: i32,
    yab: u32,
    z: u32,
    ac: u32,
    xyc: i32,
    format: u32,
    ydp: u32,
    f: &[u8],
) {
    let mut g = ES_.lock();
    
    if let Some(ad) = g.emy {
        if let Some(Some(dco)) = g.cnd.el().du(|ab| ab.as_ref().map(|b| b.ad) == Some(ad)) {
            dco.mof(z, ac, format, f);
        }
    }
}


pub fn yuq(cd: u32) {
    let mut g = ES_.lock();
    
    let ezp = match cd {
        CW_ => g.emy,
        _ => None,
    };
    
    if let Some(ad) = ezp {
        if let Some(Some(dco)) = g.cnd.el().du(|ab| ab.as_ref().map(|b| b.ad) == Some(ad)) {
            dco.tch();
        }
    }
}


pub fn tfq(cd: u32) {
    let mut g = ES_.lock();
    if cd == CW_ {
        g.idi = true;
    }
}


pub fn tfp(cd: u32) {
    let mut g = ES_.lock();
    if cd == CW_ {
        g.idi = false;
    }
}


pub fn pfg(tm: f32, p: f32) -> Option<u32> {
    let g = ES_.lock();
    
    if !g.idi {
        return None;
    }
    
    if let Some(ad) = g.emy {
        if let Some(Some(dco)) = g.cnd.iter().du(|ab| ab.as_ref().map(|b| b.ad) == Some(ad)) {
            return Some(dco.yr(tm, p));
        }
    }
    
    None
}


pub fn tze() -> bool {
    ES_.lock().idi
}


pub fn yun(bo: i32, cnd: &[u32]) {
    let mut g = ES_.lock();
    for a in 0..(bo as usize) {
        if a < cnd.len() {
            let ad = cnd[a];
            if let Some(u) = g.cnd.iter().qf(|ab| ab.as_ref().map(|b| b.ad) == Some(ad)) {
                g.cnd[u] = None;
            }
            if g.emy == Some(ad) {
                g.emy = None;
            }
        }
    }
}






pub fn rqm(aw: u32, bjo: u32, btr: u32) -> Vec<u8> {
    let mut f = Vec::fc((aw * aw * 4) as usize);
    let ncs = aw / 8;
    
    for c in 0..aw {
        for b in 0..aw {
            let cx = b / ncs.am(1);
            let ae = c / ncs.am(1);
            let s = if (cx + ae) % 2 == 0 { bjo } else { btr };
            
            f.push(((s >> 16) & 0xFF) as u8); 
            f.push(((s >> 8) & 0xFF) as u8);  
            f.push((s & 0xFF) as u8);         
            f.push(((s >> 24) & 0xFF) as u8); 
        }
    }
    
    f
}


pub fn yki(aw: u32) -> Vec<u8> {
    let mut f = Vec::fc((aw * aw * 4) as usize);
    let hbg = aw / 4;
    let hbh = aw / 2;
    let hrw = 2u32;
    
    let qry: u32 = 0xFF8B4513; 
    let upq: u32 = 0xFFC0C0C0; 
    
    for c in 0..aw {
        for b in 0..aw {
            let br = c / hbg;
            let l = if br % 2 == 0 { 0 } else { hbh / 2 };
            let bx = (b + l) % hbh;
            let je = c % hbg;
            
            let tyf = je < hrw || bx < hrw;
            let s = if tyf { upq } else { qry };
            
            f.push(((s >> 16) & 0xFF) as u8);
            f.push(((s >> 8) & 0xFF) as u8);
            f.push((s & 0xFF) as u8);
            f.push(0xFF);
        }
    }
    
    f
}


pub fn ykk(aw: u32, bjo: Color2D, btr: Color2D, dic: bool) -> Vec<u8> {
    let mut f = Vec::fc((aw * aw * 4) as usize);
    
    for c in 0..aw {
        for b in 0..aw {
            let ab = if dic {
                b as f32 / aw as f32
            } else {
                c as f32 / aw as f32
            };
            
            let m = (bjo.m as f32 + (btr.m as f32 - bjo.m as f32) * ab) as u8;
            let at = (bjo.at as f32 + (btr.at as f32 - bjo.at as f32) * ab) as u8;
            let o = (bjo.o as f32 + (btr.o as f32 - bjo.o as f32) * ab) as u8;
            
            f.push(m);
            f.push(at);
            f.push(o);
            f.push(0xFF);
        }
    }
    
    f
}
