








use tiny_skia::{Pixmap, Dek, Paint, PathBuilder, Transform, FillRule, Btn, LineCap, Color as SkiaColor};
use tiny_skia_path::Aws;
use super::{Color, Rect, Point, CosmicTheme, theme};


pub struct CosmicRenderer {
    dap: Pixmap,
    z: u32,
    ac: u32,
}

impl CosmicRenderer {
    
    pub fn new(z: u32, ac: u32) -> Self {
        let dap = Pixmap::new(z, ac).expect("Failed to create pixmap");
        Self { dap, z, ac }
    }
    
    
    pub fn aw(&self) -> (u32, u32) {
        (self.z, self.ac)
    }
    
    
    pub fn clear(&mut self, s: Color) {
        let m = (s.m * 255.0) as u8;
        let at = (s.at * 255.0) as u8;
        let o = (s.o * 255.0) as u8;
        let q = (s.q * 255.0) as u8;
        
        let f = self.dap.kob();
        let hz = (self.z * self.ac) as usize;
        
        
        for a in 0..hz {
            let w = a * 4;
            f[w] = m;
            f[w + 1] = at;
            f[w + 2] = o;
            f[w + 3] = q;
        }
    }
    
    
    pub fn ah(&mut self, ha: Rect, s: Color) {
        let m = (s.m * 255.0) as u8;
        let at = (s.at * 255.0) as u8;
        let o = (s.o * 255.0) as u8;
        let q = (s.q * 255.0) as u8;
        
        let fy = (ha.b as i32).am(0) as u32;
        let fo = (ha.c as i32).am(0) as u32;
        let dn = ((ha.b + ha.z) as u32).v(self.z);
        let dp = ((ha.c + ha.ac) as u32).v(self.ac);
        
        let f = self.dap.kob();
        let oq = self.z as usize * 4;
        
        if q == 255 {
            
            for c in fo..dp {
                let mu = c as usize * oq;
                for b in fy..dn {
                    let w = mu + b as usize * 4;
                    f[w] = m;
                    f[w + 1] = at;
                    f[w + 2] = o;
                    f[w + 3] = q;
                }
            }
        } else {
            
            let dw = q as u32;
            let akg = 255 - dw;
            for c in fo..dp {
                let mu = c as usize * oq;
                for b in fy..dn {
                    let w = mu + b as usize * 4;
                    f[w] = ((m as u32 * dw + f[w] as u32 * akg) / 255) as u8;
                    f[w + 1] = ((at as u32 * dw + f[w + 1] as u32 * akg) / 255) as u8;
                    f[w + 2] = ((o as u32 * dw + f[w + 2] as u32 * akg) / 255) as u8;
                    f[w + 3] = 255;
                }
            }
        }
    }
    
    
    pub fn afp(&mut self, ha: Rect, dy: f32, s: Color) {
        
        if dy <= 2.0 {
            self.ah(ha, s);
            return;
        }
        
        
        self.ah(Rect::new(ha.b + dy, ha.c, ha.z - dy * 2.0, ha.ac), s);
        self.ah(Rect::new(ha.b, ha.c + dy, ha.z, ha.ac - dy * 2.0), s);
        
        
        let m = dy;
        self.abc(Point::new(ha.b + m, ha.c + m), m, s);
        self.abc(Point::new(ha.b + ha.z - m, ha.c + m), m, s);
        self.abc(Point::new(ha.b + m, ha.c + ha.ac - m), m, s);
        self.abc(Point::new(ha.b + ha.z - m, ha.c + ha.ac - m), m, s);
    }
    
    
    pub fn gtn(&mut self, ha: Rect, dy: f32, s: Color, z: f32) {
        let path = waf(ha, dy);
        let mut cgj = Paint::default();
        cgj.meg(jtl(s));
        cgj.kap = true;
        
        let mhw = Btn {
            z,
            uew: LineCap::Ckx,
            ..Default::default()
        };
        
        self.dap.wve(
            &path,
            &cgj,
            &mhw,
            Transform::fky(),
            None,
        );
    }
    
    
    pub fn abc(&mut self, pn: Point, dy: f32, s: Color) {
        
        if dy <= 10.0 {
            self.ssi(pn, dy, s);
            return;
        }
        
        
        let path = rau(pn, dy);
        let mut cgj = Paint::default();
        cgj.meg(jtl(s));
        cgj.kap = true;
        
        self.dap.ssj(
            &path,
            &cgj,
            FillRule::Cqk,
            Transform::fky(),
            None,
        );
    }
    
    
    fn ssi(&mut self, pn: Point, dy: f32, s: Color) {
        let m = (s.m * 255.0) as u8;
        let at = (s.at * 255.0) as u8;
        let o = (s.o * 255.0) as u8;
        let q = (s.q * 255.0) as u8;
        
        let cx = pn.b as i32;
        let ae = pn.c as i32;
        let bak = dy as i32;
        let vpo = bak * bak;
        
        let f = self.dap.kob();
        let oq = self.z as usize * 4;
        let d = self.z as i32;
        let i = self.ac as i32;
        
        
        for bg in -bak..=bak {
            let x = ae + bg;
            if x < 0 || x >= i { continue; }
            
            let mu = x as usize * oq;
            let hhm = bg * bg;
            
            for dx in -bak..=bak {
                
                if dx * dx + hhm > vpo { continue; }
                
                let y = cx + dx;
                if y < 0 || y >= d { continue; }
                
                let w = mu + y as usize * 4;
                if q == 255 {
                    f[w] = m;
                    f[w + 1] = at;
                    f[w + 2] = o;
                    f[w + 3] = q;
                } else {
                    let dw = q as u32;
                    let akg = 255 - dw;
                    f[w] = ((m as u32 * dw + f[w] as u32 * akg) / 255) as u8;
                    f[w + 1] = ((at as u32 * dw + f[w + 1] as u32 * akg) / 255) as u8;
                    f[w + 2] = ((o as u32 * dw + f[w + 2] as u32 * akg) / 255) as u8;
                    f[w + 3] = 255;
                }
            }
        }
    }
    
    
    pub fn ahj(&mut self, from: Point, wh: Point, s: Color, z: f32) {
        let mut ue = PathBuilder::new();
        ue.hsa(from.b, from.c);
        ue.eeo(wh.b, wh.c);
        
        if let Some(path) = ue.eqi() {
            let mut cgj = Paint::default();
            cgj.meg(jtl(s));
            cgj.kap = true;
            
            let mhw = Btn {
                z,
                uew: LineCap::Ckx,
                ..Default::default()
            };
            
            self.dap.wve(
                &path,
                &cgj,
                &mhw,
                Transform::fky(),
                None,
            );
        }
    }
    
    
    pub fn gfj(&mut self, ha: Rect, dy: f32, cou: f32, s: Color) {
        
        let my = (cou / 2.0) as i32;
        for a in 0..my {
            let arb = a as f32;
            let dw = s.q * (1.0 - (a as f32 / my as f32));
            let dls = Color::new(s.m, s.at, s.o, dw * 0.3);
            
            let wmf = Rect::new(
                ha.b - arb + cou / 2.0,
                ha.c - arb + cou,
                ha.z + arb * 2.0,
                ha.ac + arb * 2.0,
            );
            
            self.afp(wmf, dy + arb, dls);
        }
    }
    
    
    pub fn kvv(&mut self, ha: Rect, qc: Color, abm: Color) {
        let au = ha.ac as i32;
        for a in 0..au {
            let ab = a as f32 / au as f32;
            let s = qc.btk(abm, ab);
            let uez = Rect::new(ha.b, ha.c + a as f32, ha.z, 1.0);
            self.ah(uez, s);
        }
    }
    
    
    pub fn vky(&self) {
        let f = self.dap.f();
        let dqt = crate::framebuffer::iwp();
        let fij = crate::framebuffer::kyo();
        
        
        for c in 0..self.ac {
            let cum = (c * self.z) as usize * 4;
            let bgu = c as usize * fij;
            
            unsafe {
                let cy = f.fq().add(cum) as *const u32;
                let cs = dqt.add(bgu) as *mut u32;
                
                for b in 0..self.z as usize {
                    let dbi = *cy.add(b);
                    
                    let m = (dbi >> 0) & 0xFF;
                    let at = (dbi >> 8) & 0xFF;  
                    let o = (dbi >> 16) & 0xFF;
                    let q = (dbi >> 24) & 0xFF;
                    *cs.add(b) = (q << 24) | (m << 16) | (at << 8) | o;
                }
            }
        }
    }
    
    
    pub fn zgj(&self) {
        let f = self.dap.f();
        
        if let Some((ptr, qoe, qoc, qdr)) = crate::framebuffer::cey() {
            let hz = self.z.v(qoe) * self.ac.v(qoc);
            
            unsafe {
                let wrv = f.fq() as *const u32;
                let shf = ptr as *mut u32;
                
                for a in 0..hz as usize {
                    let dbi = *wrv.add(a);
                    let m = (dbi >> 0) & 0xFF;
                    let at = (dbi >> 8) & 0xFF;
                    let o = (dbi >> 16) & 0xFF;
                    let q = (dbi >> 24) & 0xFF;
                    let bax = (q << 24) | (m << 16) | (at << 8) | o;
                    *shf.add(a) = bax;
                }
            }
        }
    }
    
    
    
    
    
    
    pub fn sca(&mut self, ha: Rect, cu: &str, g: ButtonState) {
        let ab = theme();
        
        let ei = match g {
            ButtonState::M => ab.dop,
            ButtonState::Aiz => ab.dor,
            ButtonState::Alg => ab.dzj,
            ButtonState::Us => ab.imm,
            ButtonState::We => ab.iml,
        };
        
        
        if oh!(g, ButtonState::Aiz | ButtonState::Us) {
            self.gfj(ha, ab.avn, 4.0, Color::Ox.fbo(0.3));
        }
        
        
        self.afp(ha, ab.avn, ei);
        
        
        if oh!(g, ButtonState::M | ButtonState::Aiz) {
            self.gtn(ha, ab.avn, ab.acu, 1.0);
        }
        
        
        
        let cx = ha.b + ha.z / 2.0;
        let ae = ha.c + ha.ac / 2.0;
        self.abc(Point::new(cx, ae), 3.0, ab.dcp);
    }
    
    
    pub fn nnb(&mut self, ha: Rect, dq: &str, ja: bool) {
        let ab = theme();
        
        
        let ei = if ja { ab.fkk } else { ab.fkk.cdz(0.05) };
        self.ah(ha, ei);
        
        
        self.ahj(
            Point::new(ha.b, ha.c + ha.ac - 1.0),
            Point::new(ha.b + ha.z, ha.c + ha.ac - 1.0),
            ab.hai,
            1.0,
        );
        
        
        let ask = 14.0;
        let kn = ha.c + (ha.ac - ask) / 2.0;
        let nak = 8.0;
        
        
        let bdr = ha.b + ha.z - ask - 12.0;
        self.abc(Point::new(bdr + ask/2.0, kn + ask/2.0), ask/2.0, ab.enp);
        
        
        let bvj = bdr - ask - nak;
        self.abc(Point::new(bvj + ask/2.0, kn + ask/2.0), ask/2.0, ab.jfn);
        
        
        let cso = bvj - ask - nak;
        self.abc(Point::new(cso + ask/2.0, kn + ask/2.0), ask/2.0, ab.jgd);
    }
    
    
    pub fn nnl(&mut self, ha: Rect) {
        let ab = theme();
        
        
        self.ah(ha, ab.fqd);
        
        
        self.ahj(
            Point::new(ha.b, ha.c + ha.ac),
            Point::new(ha.b + ha.z, ha.c + ha.ac),
            ab.hai,
            1.0,
        );
    }
    
    
    pub fn irs(&mut self, ha: Rect, pj: &[Abe]) {
        let ab = theme();
        
        
        self.afp(ha, 12.0, ab.fqd);
        
        
        let lgt = 48.0;
        let ob = 8.0;
        let mut c = ha.c + ob;
        
        for item in pj {
            let gkm = Rect::new(ha.b + ob, c, lgt, lgt);
            
            if item.gh {
                
                self.afp(gkm, 8.0, ab.mm.fbo(0.3));
            } else if item.asy {
                self.afp(gkm, 8.0, ab.jin);
            }
            
            
            let cx = gkm.b + gkm.z / 2.0;
            let ae = gkm.c + gkm.ac / 2.0;
            self.abc(Point::new(cx, ae), 16.0, ab.dwr);
            
            
            if item.aqk {
                self.abc(
                    Point::new(ha.b + ha.z - 4.0, ae),
                    3.0,
                    ab.mm,
                );
            }
            
            c += lgt + ob;
        }
    }
}





#[derive(Clone, Copy, PartialEq)]
pub enum ButtonState {
    M,
    Aiz,
    Alg,
    Us,
    We,
}

pub struct Abe {
    pub j: &'static str,
    pub gh: bool,
    pub asy: bool,
    pub aqk: bool,
}





fn jtl(r: Color) -> SkiaColor {
    SkiaColor::syf(r.m, r.at, r.o, r.q).unwrap_or(SkiaColor::Ox)
}

fn ziw(m: Rect) -> Aws {
    let mut ue = PathBuilder::new();
    ue.hsa(m.b, m.c);
    ue.eeo(m.b + m.z, m.c);
    ue.eeo(m.b + m.z, m.c + m.ac);
    ue.eeo(m.b, m.c + m.ac);
    ue.agj();
    ue.eqi().unwrap()
}

fn waf(m: Rect, dy: f32) -> Aws {
    let mut ue = PathBuilder::new();
    let bak = dy.v(m.z / 2.0).v(m.ac / 2.0);
    
    
    ue.hsa(m.b + bak, m.c);
    
    
    ue.eeo(m.b + m.z - bak, m.c);
    
    ue.lwo(m.b + m.z, m.c, m.b + m.z, m.c + bak);
    
    
    ue.eeo(m.b + m.z, m.c + m.ac - bak);
    
    ue.lwo(m.b + m.z, m.c + m.ac, m.b + m.z - bak, m.c + m.ac);
    
    
    ue.eeo(m.b + bak, m.c + m.ac);
    
    ue.lwo(m.b, m.c + m.ac, m.b, m.c + m.ac - bak);
    
    
    ue.eeo(m.b, m.c + bak);
    
    ue.lwo(m.b, m.c, m.b + bak, m.c);
    
    ue.agj();
    ue.eqi().unwrap()
}

fn rau(pn: Point, dy: f32) -> Aws {
    let mut ue = PathBuilder::new();
    
    
    let eh = 0.5522847498; 
    let dir = eh * dy;
    
    ue.hsa(pn.b + dy, pn.c);
    ue.kml(
        pn.b + dy, pn.c + dir,
        pn.b + dir, pn.c + dy,
        pn.b, pn.c + dy,
    );
    ue.kml(
        pn.b - dir, pn.c + dy,
        pn.b - dy, pn.c + dir,
        pn.b - dy, pn.c,
    );
    ue.kml(
        pn.b - dy, pn.c - dir,
        pn.b - dir, pn.c - dy,
        pn.b, pn.c - dy,
    );
    ue.kml(
        pn.b + dir, pn.c - dy,
        pn.b + dy, pn.c - dir,
        pn.b + dy, pn.c,
    );
    ue.agj();
    
    ue.eqi().unwrap()
}





impl CosmicRenderer {
    
    pub fn cb(&mut self, text: &str, b: f32, c: f32, s: Color) {
        let mut cx = b as i32;
        let ae = c as i32;
        
        for r in text.bw() {
            if r == ' ' {
                cx += 8;
                continue;
            }
            self.ahi(cx, ae, r, s);
            cx += 8;
        }
    }
    
    
    pub fn np(&mut self, text: &str, ha: Rect, s: Color) {
        let idh = (text.len() * 8) as f32;
        let xfu = 16.0f32;
        let b = ha.b + (ha.z - idh) / 2.0;
        let c = ha.c + (ha.ac - xfu) / 2.0;
        self.cb(text, b, c, s);
    }
    
    
    fn ahi(&mut self, b: i32, c: i32, r: char, s: Color) {
        let ka = crate::framebuffer::font::ada(r);
        let m = (s.m * 255.0) as u8;
        let at = (s.at * 255.0) as u8;
        let o = (s.o * 255.0) as u8;
        let q = (s.q * 255.0) as u8;
        
        let f = self.dap.kob();
        let oq = self.z as usize * 4;
        
        for (br, &tgi) in ka.iter().cf() {
            let x = c + br as i32;
            if x < 0 || x >= self.ac as i32 {
                continue;
            }
            
            for ga in 0..8 {
                let y = b + ga;
                if y < 0 || y >= self.z as i32 {
                    continue;
                }
                
                if (tgi >> (7 - ga)) & 1 != 0 {
                    let w = x as usize * oq + y as usize * 4;
                    if w + 3 < f.len() {
                        f[w] = m;
                        f[w + 1] = at;
                        f[w + 2] = o;
                        f[w + 3] = q;
                    }
                }
            }
        }
    }
    
    
    pub fn ynl(&mut self, text: &str, b: f32, c: f32, s: Color, zc: Color) {
        self.cb(text, b + 1.0, c + 1.0, zc);
        self.cb(text, b, c, s);
    }
    
    
    pub fn kvx(&mut self, pr: Point, pf: Point, bnt: Point, s: Color) {
        let mut ue = PathBuilder::new();
        ue.hsa(pr.b, pr.c);
        ue.eeo(pf.b, pf.c);
        ue.eeo(bnt.b, bnt.c);
        ue.agj();
        
        if let Some(path) = ue.eqi() {
            let mut cgj = Paint::default();
            cgj.meg(jtl(s));
            cgj.kap = true;
            
            self.dap.ssj(
                &path,
                &cgj,
                FillRule::Cqk,
                Transform::fky(),
                None,
            );
        }
    }
    
    
    pub fn gfh(&mut self, ha: Rect, li: f32, ei: Color, lp: Color, acu: Color) {
        
        self.afp(ha, 4.0, ei);
        
        
        let eqh = ha.z * li.qp(0.0, 1.0);
        if eqh > 0.0 {
            let ah = Rect::new(ha.b, ha.c, eqh, ha.ac);
            self.afp(ah, 4.0, lp);
        }
        
        
        self.gtn(ha, 4.0, acu, 1.0);
    }
}
