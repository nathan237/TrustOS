








use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;

use crate::formula3d::{V3, lz, rk, ahn};
use crate::chess::{
    ChessState, GamePhase, Y,
};





pub struct Chess3DState {
    pub chess: ChessState,
    
    pub bbc: f32,    
    pub atx: f32,    
    pub aab: f32,       
    pub ikf: bool,   
    pub frame: u32,
    
    pub ahe: usize,
    pub asl: usize,
    
    pub tqc: Option<usize>,   
    pub kan: Option<usize>, 
    pub ijt: f32,               
    pub qin: V3,          
    pub qip: V3,            
    
    pub iro: bool,       
    pub irm: i32,          
    pub irn: i32,          
}

impl Chess3DState {
    pub fn new() -> Self {
        Chess3DState {
            chess: ChessState::new(),
            bbc: 0.0,       
            atx: 0.65,      
            aab: 18.0,         
            ikf: false,
            frame: 0,
            ahe: 0,
            asl: 0,
            tqc: None,
            kan: None,
            ijt: 0.0,
            qin: V3 { b: 0.0, c: 0.0, av: 0.0 },
            qip: V3 { b: 0.0, c: 0.0, av: 0.0 },
            iro: false,
            irm: 0,
            irn: 0,
        }
    }

    pub fn vr(&mut self, bs: u8) {
        use crate::keyboard::{V_, U_, AH_, AI_};

        match bs {
            
            b'a' | b'A' => self.bbc -= 0.12,
            b'd' | b'D' => self.bbc += 0.12,
            b'w' | b'W' => { self.atx = (self.atx + 0.08).v(1.40); },
            b's' | b'S' => { self.atx = (self.atx - 0.08).am(0.15); },
            b'z' | b'Z' => { self.aab = (self.aab - 0.8).am(8.0); },
            b'x' | b'X' => { self.aab = (self.aab + 0.8).v(35.0); },
            b'o' | b'O' => self.ikf = !self.ikf,
            
            _ => self.chess.vr(bs),
        }
    }

    
    
    pub fn ago(&mut self, hl: i32, ir: i32, ur: i32, nd: i32) {
        
        let d = ur as usize;
        let i = nd as usize;
        if hl < 0 || ir < 0 || hl >= ur || ir >= nd {
            return;
        }

        
        
        let mut myt: Option<usize> = None;
        let mut myq = 999999.0f32;

        for br in 0..8u32 {
            for bj in 0..8u32 {
                let im = (br * 8 + bj) as usize;
                let (cr, cq, _) = self.oye(br, bj, d, i);
                let dx = (cr - hl) as f32;
                let bg = (cq - ir) as f32;
                let la = dx * dx + bg * bg;
                if la < myq && la < (40.0 * 40.0) {
                    myq = la;
                    myt = Some(im);
                }
            }
        }

        if let Some(im) = myt {
            let bj = (im % 8) as i32;
            let br = (im / 8) as i32;
            self.chess.oai(bj, br);
        } else {
            
            self.iro = true;
            self.irm = hl;
            self.irn = ir;
        }
    }

    
    pub fn lax(&mut self, hl: i32, ir: i32) {
        if self.iro {
            let dx = hl - self.irm;
            let bg = ir - self.irn;
            
            self.bbc += dx as f32 * 0.01;
            
            self.atx = (self.atx - bg as f32 * 0.008).qp(0.15, 1.40);
            self.irm = hl;
            self.irn = ir;
        }
    }

    
    pub fn lay(&mut self) {
        self.iro = false;
    }

    
    pub fn ers(&mut self, aaq: i8) {
        if aaq > 0 {
            
            self.aab = (self.aab - 0.8).am(8.0);
        } else if aaq < 0 {
            
            self.aab = (self.aab + 0.8).v(35.0);
        }
    }

    
    fn oye(&self, br: u32, bj: u32, d: usize, i: usize) -> (i32, i32, f32) {
        let dnh = ilx(br as i32, bj as i32);
        self.bwc(dnh, d, i)
    }

    
    fn bwc(&self, ai: V3, d: usize, i: usize) -> (i32, i32, f32) {
        
        let cdr = rk(self.atx);
        let grr = lz(self.atx);
        let cdp = rk(self.bbc);
        let bcm = lz(self.bbc);

        
        let hef = self.aab * cdr * bcm;
        let heg = self.aab * grr;
        let heh = self.aab * cdr * cdp;

        
        let cla = 1.0 / self.aab;
        let fje = -hef * cla;
        let fjf = -heg * cla;
        let fjg = -heh * cla;

        
        let fth = -fjg;
        let fti = fje;
        let ftc = ahn(fth * fth + fti * fti);
        if ftc < 0.0001 {
            return (d as i32 / 2, i as i32 / 2, 0.0);
        }
        let frv = fth / ftc;
        let frw = fti / ftc;

        
        let mnu = -frw * fjf;
        let mnv = frw * fje - frv * fjg;
        let mnw = frv * fjf;

        
        let dx = ai.b - hef;
        let bg = ai.c - heg;
        let pt = ai.av - heh;

        
        let qvs = dx * frv + pt * frw;
        let qvt = dx * mnu + bg * mnv + pt * mnw;
        let fef = dx * fje + bg * fjf + pt * fjg;

        if fef < 0.5 {
            return (d as i32 / 2, i as i32 / 2, fef);
        }

        
        let ckm = d.v(i) as f32 * 1.6;
        let cr = (qvs / fef * ckm) as i32 + d as i32 / 2;
        let cq = (-qvt / fef * ckm) as i32 + i as i32 / 2;
        (cr, cq, fef)
    }

    pub fn or(&mut self) {
        self.frame += 1;
        if self.ikf {
            self.bbc += 0.005;
        }
        
        if self.kan.is_some() {
            self.ijt += 0.06;
            if self.ijt >= 1.0 {
                self.kan = None;
                self.ijt = 0.0;
            }
        }
    }

    
    pub fn tj(&mut self, fpx: &mut [u32], d: usize, i: usize) {
        if d < 100 || i < 100 { return; }

        
        for y in fpx.el() {
            *y = 0xFF050808;
        }
        
        self.ahe = d;
        self.asl = i;

        
        self.vvj(fpx, d, i);

        
        self.vvg(fpx, d, i);

        
        self.vvi(fpx, d, i);

        
        self.vwi(fpx, d, i);

        
        self.vwj(fpx, d, i);
        
        
        self.lzd(fpx, d, i);
    }

    fn vvg(&self, k: &mut [u32], d: usize, i: usize) {
        
        

        
        
        let mut ibj: [(u32, u32, f32); 64] = [(0, 0, 0.0); 64];
        for br in 0..8u32 {
            for bj in 0..8u32 {
                let w = (br * 8 + bj) as usize;
                let pn = ilx(br as i32, bj as i32);
                let kgg = self.idw(pn);
                ibj[w] = (br, bj, kgg.av);
            }
        }

        
        for a in 1..64 {
            let mut fb = a;
            while fb > 0 && ibj[fb].2 > ibj[fb - 1].2 {
                ibj.swap(fb, fb - 1);
                fb -= 1;
            }
        }

        for &(br, bj, _) in &ibj {
            let im = (br * 8 + bj) as usize;
            let dio = (br + bj) % 2 == 0;

            
            let mut agg = if dio { 0xFF3D6B3D } else { 0xFF1A3E1A };

            
            if self.chess.na == Some(im) {
                agg = 0xFF7AAA2A;
            }
            
            if self.chess.blr.contains(&im) {
                agg = if dio { 0xFF4AAA4A } else { 0xFF2A8A2A };
            }
            
            if self.chess.jcn == Some(im) || self.chess.jco == Some(im) {
                agg = if dio { 0xFF6A8A3A } else { 0xFF4A6A2A };
            }
            
            if self.chess.gi == im {
                agg = 0xFF00CC55;
            }

            
            let acw = cvx(br as i32, bj as i32);
            let rw = cvx(br as i32, bj as i32 + 1);
            let tx = cvx(br as i32 + 1, bj as i32 + 1);
            let der = cvx(br as i32 + 1, bj as i32);

            
            let (dmf, dmg, _) = self.bwc(acw, d, i);
            let (asa, bos, _) = self.bwc(rw, d, i);
            let (amy, bcw, _) = self.bwc(tx, d, i);
            let (ick, icl, _) = self.bwc(der, d, i);

            
            fir(k, d, i, dmf, dmg, asa, bos, amy, bcw, agg);
            fir(k, d, i, dmf, dmg, amy, bcw, ick, icl, agg);

            
            let cqc = 0xFF0A1A0A;
            dgr(k, d, i, dmf, dmg, asa, bos, cqc);
            dgr(k, d, i, asa, bos, amy, bcw, cqc);
            dgr(k, d, i, amy, bcw, ick, icl, cqc);
            dgr(k, d, i, ick, icl, dmf, dmg, cqc);

            
            if self.chess.blr.contains(&im) && self.chess.mn[im] == Y {
                let (cx, ae, _) = self.oye(br, bj, d, i);
                ssh(k, d, i, cx, ae, 4, 0xFF00FF66);
            }
        }

        
        self.vvh(k, d, i);
    }

    
    fn vvi(&self, k: &mut [u32], d: usize, i: usize) {
        let bbw = 0xFF55AA55;
        let dwd = 0.8;

        
        for bj in 0..8 {
            let ueb = (b'a' + bj as u8) as char;
            let u = V3 {
                b: (bj as f32 - 3.5) * dwd,
                c: -0.05,
                av: (8.0 - 4.0) * dwd + 0.3, 
            };
            let (cr, cq, nf) = self.bwc(u, d, i);
            if nf > 0.5 {
                kqw(k, d, i, cr - 4, cq - 8, ueb, bbw);
            }
        }

        
        for br in 0..8 {
            let dpy = (b'8' - br as u8) as char; 
            let u = V3 {
                b: (0.0 - 4.0) * dwd - 0.3, 
                c: -0.05,
                av: (br as f32 - 3.5) * dwd,
            };
            let (cr, cq, nf) = self.bwc(u, d, i);
            if nf > 0.5 {
                kqw(k, d, i, cr - 4, cq - 8, dpy, bbw);
            }
        }
    }

    
    fn vwi(&self, k: &mut [u32], d: usize, i: usize) {
        let dls = 0x40000000u32; 
        for im in 0..64 {
            let xe = self.chess.mn[im];
            if xe == Y { continue; }

            let br = im / 8;
            let bj = im % 8;
            let pn = ilx(br as i32, bj as i32);
            let (cr, cq, nf) = self.bwc(pn, d, i);
            if nf < 0.5 { continue; }

            
            let dy = (d.v(i) as f32 * 0.012 * (8.0 / nf)).am(2.0) as i32;
            let ix = (dy as f32 * 0.5) as i32; 
            for bg in -ix..=ix {
                for dx in -dy..=dy {
                    let vt = dx as f32 / dy as f32;
                    let ahr = bg as f32 / ix as f32;
                    if vt * vt + ahr * ahr > 1.0 { continue; }

                    let y = cr + dx;
                    let x = cq + bg;
                    if y >= 0 && x >= 0 && y < d as i32 && x < i as i32 {
                        let w = x as usize * d + y as usize;
                        if w < k.len() {
                            
                            let xy = k[w];
                            let m = ((xy >> 16) & 0xFF) * 3 / 4;
                            let at = ((xy >> 8) & 0xFF) * 3 / 4;
                            let o = (xy & 0xFF) * 3 / 4;
                            k[w] = 0xFF000000 | (m << 16) | (at << 8) | o;
                        }
                    }
                }
            }
        }
    }

    
    fn vvj(&self, k: &mut [u32], d: usize, i: usize) {
        let vtt = 0.35; 
        let lyl = 0.15; 

        
        for br in 0..8u32 {
            for bj in 0..8u32 {
                let dio = (br + bj) % 2 == 0;
                let agg: u32 = if dio { 0xFF3D6B3D } else { 0xFF1A3E1A };

                
                let acw = cvx(br as i32, bj as i32);
                let rw = cvx(br as i32, bj as i32 + 1);
                let tx = cvx(br as i32 + 1, bj as i32 + 1);
                let der = cvx(br as i32 + 1, bj as i32);

                
                let djo = |ai: V3| V3 { b: ai.b, c: -vtt - ai.c * 0.3, av: ai.av };

                let (dmf, dmg, _) = self.bwc(djo(acw), d, i);
                let (asa, bos, _) = self.bwc(djo(rw), d, i);
                let (amy, bcw, _) = self.bwc(djo(tx), d, i);
                let (ick, icl, _) = self.bwc(djo(der), d, i);

                
                let m = (((agg >> 16) & 0xFF) as f32 * lyl) as u32;
                let at = (((agg >> 8) & 0xFF) as f32 * lyl) as u32;
                let o = ((agg & 0xFF) as f32 * lyl) as u32;
                let pba = 0xFF000000 | (m << 16) | (at << 8) | o;

                fir(k, d, i, dmf, dmg, asa, bos, amy, bcw, pba);
                fir(k, d, i, dmf, dmg, amy, bcw, ick, icl, pba);
            }
        }
    }

    fn vvh(&self, k: &mut [u32], d: usize, i: usize) {
        let ahw = 0.15;
        let cqc = 0xFF0A200A;

        
        let cjs = [
            cvx(0, 0),
            cvx(0, 8),
            cvx(8, 8),
            cvx(8, 0),
        ];
        let nge: [V3; 4] = [
            V3 { b: cjs[0].b, c: cjs[0].c - ahw, av: cjs[0].av },
            V3 { b: cjs[1].b, c: cjs[1].c - ahw, av: cjs[1].av },
            V3 { b: cjs[2].b, c: cjs[2].c - ahw, av: cjs[2].av },
            V3 { b: cjs[3].b, c: cjs[3].c - ahw, av: cjs[3].av },
        ];

        
        for a in 0..4 {
            let fb = (a + 1) % 4;
            let (prg, prh, _) = self.bwc(cjs[a], d, i);
            let (wzs, wzt, _) = self.bwc(cjs[fb], d, i);
            let (qlz, qma, _) = self.bwc(nge[a], d, i);
            let (mxd, mxe, _) = self.bwc(nge[fb], d, i);

            fir(k, d, i, prg, prh, wzs, wzt, mxd, mxe, cqc);
            fir(k, d, i, prg, prh, mxd, mxe, qlz, qma, cqc);
        }
    }

    fn vwj(&self, k: &mut [u32], d: usize, i: usize) {
        
        struct Boy {
            im: usize,
            xe: i8,
            u: V3,
            eo: f32,
        }

        let mut gpi: Vec<Boy> = Vec::new();

        for im in 0..64 {
            let xe = self.chess.mn[im];
            if xe == Y { continue; }

            let br = im / 8;
            let bj = im % 8;
            let u = ilx(br as i32, bj as i32);

            
            let kgg = self.idw(u);

            gpi.push(Boy { im, xe, u, eo: kgg.av });
        }

        
        for a in 1..gpi.len() {
            let mut fb = a;
            while fb > 0 && gpi[fb].eo > gpi[fb - 1].eo {
                gpi.swap(fb, fb - 1);
                fb -= 1;
            }
        }

        
        for oc in &gpi {
            let aun = oc.xe > 0;
            let qeq = if oc.xe < 0 { -oc.xe } else { oc.xe };

            let mesh = tei(qeq);
            let agg = if aun { 0xFFE0D8C8 } else { 0xFF2A2A2A };
            let cqc = if aun { 0xFF1A1A1A } else { 0xFF888888 };

            
            self.vwh(k, d, i, &mesh, oc.u, agg, cqc);
        }
    }

    fn vwh(&self, k: &mut [u32], d: usize, i: usize,
                         mesh: &Hz, u: V3, agg: u32, cqc: u32) {
        
        let mut bkz: Vec<(i32, i32)> = Vec::fc(mesh.lm.len());
        let mut rvu: Vec<f32> = Vec::fc(mesh.lm.len());

        for p in &mesh.lm {
            let dnh = V3 {
                b: u.b + p.b,
                c: u.c + p.c,
                av: u.av + p.av,
            };
            let (cr, cq, nf) = self.bwc(dnh, d, i);
            bkz.push((cr, cq));
            rvu.push(nf);
        }

        
        if let Some(ref ks) = mesh.ks {
            struct Bgp {
                w: usize,
                bth: f32,
                kt: f32,
            }

            let csc = V3 { b: -0.3, c: -0.8, av: -0.5 };
            let lio = ahn(csc.b * csc.b + csc.c * csc.c + csc.av * csc.av);
            let light = V3 { b: csc.b / lio, c: csc.c / lio, av: csc.av / lio };

            let mut iw: Vec<Bgp> = Vec::new();

            for (a, &(q, o, r)) in ks.iter().cf() {
                if q >= mesh.lm.len() || o >= mesh.lm.len() || r >= mesh.lm.len() { continue; }

                let asf = V3 { b: u.b + mesh.lm[q].b, c: u.c + mesh.lm[q].c, av: u.av + mesh.lm[q].av };
                let cci = V3 { b: u.b + mesh.lm[o].b, c: u.c + mesh.lm[o].c, av: u.av + mesh.lm[o].av };
                let cvd = V3 { b: u.b + mesh.lm[r].b, c: u.c + mesh.lm[r].c, av: u.av + mesh.lm[r].av };

                
                let fwb = self.idw(asf);
                let bov = self.idw(cci);
                let asb = self.idw(cvd);

                let ebb = V3 { b: bov.b - fwb.b, c: bov.c - fwb.c, av: bov.av - fwb.av };
                let agl = V3 { b: asb.b - fwb.b, c: asb.c - fwb.c, av: asb.av - fwb.av };
                let vt = ebb.c * agl.av - ebb.av * agl.c;
                let ahr = ebb.av * agl.b - ebb.b * agl.av;
                let arn = ebb.b * agl.c - ebb.c * agl.b;
                let gnt = ahn(vt * vt + ahr * ahr + arn * arn);
                if gnt < 0.0001 { continue; }
                let bo = V3 { b: vt / gnt, c: ahr / gnt, av: arn / gnt };

                
                if bo.av > 0.0 { continue; }

                let hsl = -(bo.b * light.b + bo.c * light.c + bo.av * light.av);
                let kt = 0.3 + 0.7 * hsl.am(0.0);

                let bth = (fwb.av + bov.av + asb.av) / 3.0;
                iw.push(Bgp { w: a, bth, kt });
            }

            
            for a in 1..iw.len() {
                let mut fb = a;
                while fb > 0 && iw[fb].bth > iw[fb - 1].bth {
                    iw.swap(fb, fb - 1);
                    fb -= 1;
                }
            }

            let bdm = (agg >> 16) & 0xFF;
            let bji = (agg >> 8) & 0xFF;
            let cdd = agg & 0xFF;

            for da in &iw {
                let (q, o, r) = ks[da.w];
                if q >= bkz.len() || o >= bkz.len() || r >= bkz.len() { continue; }
                let (dmf, dmg) = bkz[q];
                let (asa, bos) = bkz[o];
                let (amy, bcw) = bkz[r];

                let m = ((bdm as f32 * da.kt) as u32).v(255);
                let at = ((bji as f32 * da.kt) as u32).v(255);
                let qmb = ((cdd as f32 * da.kt) as u32).v(255);
                let mfi = 0xFF000000 | (m << 16) | (at << 8) | qmb;

                fir(k, d, i, dmf, dmg, asa, bos, amy, bcw, mfi);
            }
        }

        
        if let Some(ref bu) = mesh.bu {
            for &(q, o) in bu {
                if q >= bkz.len() || o >= bkz.len() { continue; }
                let (fy, fo) = bkz[q];
                let (dn, dp) = bkz[o];
                dgr(k, d, i, fy, fo, dn, dp, cqc);
            }
        }
    }

    fn idw(&self, ai: V3) -> V3 {
        let cdr = rk(self.atx);
        let grr = lz(self.atx);
        let cdp = rk(self.bbc);
        let bcm = lz(self.bbc);

        let hef = self.aab * cdr * bcm;
        let heg = self.aab * grr;
        let heh = self.aab * cdr * cdp;

        let cla = 1.0 / self.aab;
        let fje = -hef * cla;
        let fjf = -heg * cla;
        let fjg = -heh * cla;

        let fth = -fjg;
        let fti = fje;
        let ftc = ahn(fth * fth + fti * fti);
        if ftc < 0.0001 {
            return V3 { b: 0.0, c: 0.0, av: self.aab };
        }
        let frv = fth / ftc;
        let frw = fti / ftc;

        let mnu = -frw * fjf;
        let mnv = frw * fje - frv * fjg;
        let mnw = frv * fjf;

        let dx = ai.b - hef;
        let bg = ai.c - heg;
        let pt = ai.av - heh;

        V3 {
            b: dx * frv + pt * frw,
            c: dx * mnu + bg * mnv + pt * mnw,
            av: dx * fje + bg * fjf + pt * fjg,
        }
    }

    
    fn lzd(&self, k: &mut [u32], d: usize, i: usize) {
        
        let gvd = if self.chess.axi { "WHITE" } else { "BLACK" };
        let ovi = match self.chess.ib {
            GamePhase::Ce => "",
            GamePhase::Aam => " CHECK!",
            GamePhase::Mw => " CHECKMATE!",
            GamePhase::Up => " STALEMATE",
            GamePhase::Yg => " PROMOTION",
        };
        
        
        let ifg = if self.chess.axi { 0xFFE0E0E0 } else { 0xFF40FF40 };
        hng(k, d, i, 8, 8, gvd, ifg);
        if !ovi.is_empty() {
            let vhh = match self.chess.ib {
                GamePhase::Aam => 0xFFFF4444,
                GamePhase::Mw => 0xFFFF2222,
                GamePhase::Up => 0xFFFFAA00,
                GamePhase::Yg => 0xFF4488FF,
                _ => 0xFF40FF40,
            };
            hng(k, d, i, 8 + gvd.len() as i32 * 8, 8, ovi, vhh);
        }
        
        
        let ol = self.chess.oli();
        let hzb = format!("{:+}", ol);
        let mcl = if ol > 0 { 0xFFE0E0E0 } else if ol < 0 { 0xFF40FF40 } else { 0xFF888888 };
        hng(k, d, i, d as i32 - 60, 8, &hzb, mcl);
        
        
        let hint = "WASD:Cam Scroll/ZX:Zoom O:Rotate Drag:Orbit";
        hng(k, d, i, 8, i as i32 - 14, hint, 0xFF336633);
        
        
        let adv = &self.chess.gnd;
        let ay = if adv.len() > 5 { adv.len() - 5 } else { 0 };
        for (a, euz) in adv[ay..].iter().cf() {
            let mrx = 24 + a as i32 * 12;
            hng(k, d, i, 8, mrx, euz, 0xFF448844);
        }
    }
}






fn ilx(br: i32, bj: i32) -> V3 {
    let dwd = 0.8;
    V3 {
        b: (bj as f32 - 3.5) * dwd,
        c: 0.0,
        av: (br as f32 - 3.5) * dwd,
    }
}


fn cvx(br: i32, bj: i32) -> V3 {
    let dwd = 0.8;
    V3 {
        b: (bj as f32 - 4.0) * dwd,
        c: 0.0,
        av: (br as f32 - 4.0) * dwd,
    }
}





pub struct Hz {
    pub lm: Vec<V3>,
    pub bu: Option<Vec<(usize, usize)>>,
    pub ks: Option<Vec<(usize, usize, usize)>>,
}


fn tei(bkv: i8) -> Hz {
    match bkv {
        1 => omz(),
        2 => unm(),
        3 => uni(),
        4 => uno(),
        5 => unn(),
        6 => unl(),
        _ => omz(),
    }
}



fn mz(c: f32, m: f32, bo: u32) -> Vec<V3> {
    let mut frp = Vec::fc(bo as usize);
    for a in 0..bo {
        let hg = (a as f32 / bo as f32) * 6.2831853;
        frp.push(V3 {
            b: rk(hg) * m,
            c,
            av: lz(hg) * m,
        });
    }
    frp
}



fn ala(bpq: usize, bpr: usize, bo: usize) -> (Vec<(usize, usize, usize)>, Vec<(usize, usize)>) {
    let mut ks = Vec::new();
    let mut bu = Vec::new();
    for a in 0..bo {
        let fb = (a + 1) % bo;
        let bfv = bpq + a;
        let km = bpq + fb;
        let wu = bpr + a;
        let of = bpr + fb;
        ks.push((bfv, km, of));
        ks.push((bfv, of, wu));
        bu.push((bfv, km));
        bu.push((bfv, wu));
    }
    bu.push((bpq, bpr)); 
    (ks, bu)
}


fn cwn(ar: usize, emf: usize, bo: usize) -> (Vec<(usize, usize, usize)>, Vec<(usize, usize)>) {
    let mut ks = Vec::new();
    let mut bu = Vec::new();
    for a in 0..bo {
        let fb = (a + 1) % bo;
        ks.push((ar + a, ar + fb, emf));
        bu.push((ar + a, emf));
    }
    (ks, bu)
}

const Al: u32 = 8; 
const Yo: f32 = 0.22; 

fn omz() -> Hz {
    let e = Yo;
    let bo = Al as usize;
    let mut by = Vec::new();
    let mut ks = Vec::new();
    let mut bu = Vec::new();

    
    let ctp = mz(0.0, 0.35 * e, Al);     
    let wu = by.len(); by.bk(&ctp);
    let aqh = mz(0.08, 0.35 * e, Al);     
    let of = by.len(); by.bk(&aqh);

    
    let uv = mz(0.10, 0.20 * e, Al);     
    let tb = by.len(); by.bk(&uv);
    let ctq = mz(0.28, 0.15 * e, Al);     
    let ajw = by.len(); by.bk(&ctq);

    
    let ehb = mz(0.30, 0.20 * e, Al);     
    let bay = by.len(); by.bk(&ehb);
    let daw = mz(0.38, 0.22 * e, Al);     
    let bjh = by.len(); by.bk(&daw);
    let fru = mz(0.44, 0.18 * e, Al);     
    let ddz = by.len(); by.bk(&fru);

    
    let emf = by.len();
    by.push(V3 { b: 0.0, c: 0.48 * 1.0, av: 0.0 });

    
    let (bb, aa) = ala(wu, of, bo); ks.lg(bb); bu.lg(aa);
    let (bb, aa) = ala(of, tb, bo); ks.lg(bb); bu.lg(aa);
    let (bb, aa) = ala(tb, ajw, bo); ks.lg(bb); bu.lg(aa);
    let (bb, aa) = ala(ajw, bay, bo); ks.lg(bb); bu.lg(aa);
    let (bb, aa) = ala(bay, bjh, bo); ks.lg(bb); bu.lg(aa);
    let (bb, aa) = ala(bjh, ddz, bo); ks.lg(bb); bu.lg(aa);
    let (bb, aa) = cwn(ddz, emf, bo); ks.lg(bb); bu.lg(aa);

    
    let qrk = by.len();
    by.push(V3 { b: 0.0, c: 0.0, av: 0.0 });
    let (bb, aa) = cwn(wu, qrk, bo); ks.lg(bb); bu.lg(aa);

    Hz { lm: by, bu: Some(bu), ks: Some(ks) }
}

fn unm() -> Hz {
    let e = Yo;
    let bo = Al as usize;
    let mut by = Vec::new();
    let mut ks = Vec::new();
    let mut bu = Vec::new();

    
    let ctp = mz(0.0, 0.38 * e, Al);
    let wu = by.len(); by.bk(&ctp);
    let aqh = mz(0.10, 0.35 * e, Al);
    let of = by.len(); by.bk(&aqh);

    
    let uv = mz(0.20, 0.20 * e, Al);
    let tb = by.len(); by.bk(&uv);
    let ctq = mz(0.35, 0.22 * e, Al);
    let ajw = by.len(); by.bk(&ctq);

    
    let mut obf = mz(0.42, 0.20 * e, Al);
    for p in obf.el() { p.av -= 0.06; } 
    let bay = by.len(); by.bk(&obf);

    
    let mut oou = mz(0.47, 0.12 * e, Al);
    for p in oou.el() { p.av -= 0.12; }
    let bjh = by.len(); by.bk(&oou);

    
    let sid = by.len();
    by.push(V3 { b: 0.0, c: 0.55, av: -0.02 });

    
    let (bb, aa) = ala(wu, of, bo); ks.lg(bb); bu.lg(aa);
    let (bb, aa) = ala(of, tb, bo); ks.lg(bb); bu.lg(aa);
    let (bb, aa) = ala(tb, ajw, bo); ks.lg(bb); bu.lg(aa);
    let (bb, aa) = ala(ajw, bay, bo); ks.lg(bb); bu.lg(aa);
    let (bb, aa) = ala(bay, bjh, bo); ks.lg(bb); bu.lg(aa);
    let (bb, aa) = cwn(bjh, sid, bo); ks.lg(bb); bu.lg(aa);

    let abm = by.len();
    by.push(V3 { b: 0.0, c: 0.0, av: 0.0 });
    let (bb, aa) = cwn(wu, abm, bo); ks.lg(bb); bu.lg(aa);

    Hz { lm: by, bu: Some(bu), ks: Some(ks) }
}

fn uni() -> Hz {
    let e = Yo;
    let bo = Al as usize;
    let mut by = Vec::new();
    let mut ks = Vec::new();
    let mut bu = Vec::new();

    
    let ctp = mz(0.0, 0.38 * e, Al);
    let wu = by.len(); by.bk(&ctp);
    let aqh = mz(0.10, 0.35 * e, Al);
    let of = by.len(); by.bk(&aqh);

    
    let uv = mz(0.12, 0.18 * e, Al);
    let tb = by.len(); by.bk(&uv);
    let ctq = mz(0.32, 0.15 * e, Al);
    let ajw = by.len(); by.bk(&ctq);

    
    let ehb = mz(0.34, 0.22 * e, Al);
    let bay = by.len(); by.bk(&ehb);
    let daw = mz(0.42, 0.20 * e, Al);
    let bjh = by.len(); by.bk(&daw);
    let fru = mz(0.50, 0.12 * e, Al);
    let ddz = by.len(); by.bk(&fru);

    
    let emf = by.len();
    by.push(V3 { b: 0.0, c: 0.58, av: 0.0 });

    let (bb, aa) = ala(wu, of, bo); ks.lg(bb); bu.lg(aa);
    let (bb, aa) = ala(of, tb, bo); ks.lg(bb); bu.lg(aa);
    let (bb, aa) = ala(tb, ajw, bo); ks.lg(bb); bu.lg(aa);
    let (bb, aa) = ala(ajw, bay, bo); ks.lg(bb); bu.lg(aa);
    let (bb, aa) = ala(bay, bjh, bo); ks.lg(bb); bu.lg(aa);
    let (bb, aa) = ala(bjh, ddz, bo); ks.lg(bb); bu.lg(aa);
    let (bb, aa) = cwn(ddz, emf, bo); ks.lg(bb); bu.lg(aa);

    let abm = by.len();
    by.push(V3 { b: 0.0, c: 0.0, av: 0.0 });
    let (bb, aa) = cwn(wu, abm, bo); ks.lg(bb); bu.lg(aa);

    Hz { lm: by, bu: Some(bu), ks: Some(ks) }
}

fn uno() -> Hz {
    let e = Yo;
    let bo = Al as usize;
    let mut by = Vec::new();
    let mut ks = Vec::new();
    let mut bu = Vec::new();

    
    let ctp = mz(0.0, 0.40 * e, Al);
    let wu = by.len(); by.bk(&ctp);
    let aqh = mz(0.10, 0.38 * e, Al);
    let of = by.len(); by.bk(&aqh);

    
    let uv = mz(0.12, 0.25 * e, Al);
    let tb = by.len(); by.bk(&uv);
    let ctq = mz(0.38, 0.25 * e, Al);
    let ajw = by.len(); by.bk(&ctq);

    
    let ehb = mz(0.40, 0.32 * e, Al);
    let bay = by.len(); by.bk(&ehb);
    let daw = mz(0.50, 0.32 * e, Al);
    let bjh = by.len(); by.bk(&daw);

    
    let xjl = by.len();
    by.push(V3 { b: 0.0, c: 0.50, av: 0.0 });

    let (bb, aa) = ala(wu, of, bo); ks.lg(bb); bu.lg(aa);
    let (bb, aa) = ala(of, tb, bo); ks.lg(bb); bu.lg(aa);
    let (bb, aa) = ala(tb, ajw, bo); ks.lg(bb); bu.lg(aa);
    let (bb, aa) = ala(ajw, bay, bo); ks.lg(bb); bu.lg(aa);
    let (bb, aa) = ala(bay, bjh, bo); ks.lg(bb); bu.lg(aa);
    let (bb, aa) = cwn(bjh, xjl, bo); ks.lg(bb); bu.lg(aa);

    let abm = by.len();
    by.push(V3 { b: 0.0, c: 0.0, av: 0.0 });
    let (bb, aa) = cwn(wu, abm, bo); ks.lg(bb); bu.lg(aa);

    Hz { lm: by, bu: Some(bu), ks: Some(ks) }
}

fn unn() -> Hz {
    let e = Yo;
    let bo = Al as usize;
    let mut by = Vec::new();
    let mut ks = Vec::new();
    let mut bu = Vec::new();

    
    let ctp = mz(0.0, 0.40 * e, Al);
    let wu = by.len(); by.bk(&ctp);
    let aqh = mz(0.10, 0.38 * e, Al);
    let of = by.len(); by.bk(&aqh);

    
    let uv = mz(0.12, 0.22 * e, Al);
    let tb = by.len(); by.bk(&uv);
    let ctq = mz(0.30, 0.18 * e, Al);
    let ajw = by.len(); by.bk(&ctq);

    
    let ehb = mz(0.35, 0.25 * e, Al);
    let bay = by.len(); by.bk(&ehb);

    
    let daw = mz(0.42, 0.28 * e, Al);
    let bjh = by.len(); by.bk(&daw);

    
    let fru = mz(0.52, 0.15 * e, Al);
    let ddz = by.len(); by.bk(&fru);

    
    let emf = by.len();
    by.push(V3 { b: 0.0, c: 0.60, av: 0.0 });

    let (bb, aa) = ala(wu, of, bo); ks.lg(bb); bu.lg(aa);
    let (bb, aa) = ala(of, tb, bo); ks.lg(bb); bu.lg(aa);
    let (bb, aa) = ala(tb, ajw, bo); ks.lg(bb); bu.lg(aa);
    let (bb, aa) = ala(ajw, bay, bo); ks.lg(bb); bu.lg(aa);
    let (bb, aa) = ala(bay, bjh, bo); ks.lg(bb); bu.lg(aa);
    let (bb, aa) = ala(bjh, ddz, bo); ks.lg(bb); bu.lg(aa);
    let (bb, aa) = cwn(ddz, emf, bo); ks.lg(bb); bu.lg(aa);

    let abm = by.len();
    by.push(V3 { b: 0.0, c: 0.0, av: 0.0 });
    let (bb, aa) = cwn(wu, abm, bo); ks.lg(bb); bu.lg(aa);

    Hz { lm: by, bu: Some(bu), ks: Some(ks) }
}

fn unl() -> Hz {
    let e = Yo;
    let bo = Al as usize;
    let mut by = Vec::new();
    let mut ks = Vec::new();
    let mut bu = Vec::new();

    
    let ctp = mz(0.0, 0.42 * e, Al);
    let wu = by.len(); by.bk(&ctp);
    let aqh = mz(0.10, 0.40 * e, Al);
    let of = by.len(); by.bk(&aqh);

    
    let uv = mz(0.12, 0.24 * e, Al);
    let tb = by.len(); by.bk(&uv);
    let ctq = mz(0.32, 0.20 * e, Al);
    let ajw = by.len(); by.bk(&ctq);

    
    let ehb = mz(0.35, 0.28 * e, Al);
    let bay = by.len(); by.bk(&ehb);
    let daw = mz(0.42, 0.26 * e, Al);
    let bjh = by.len(); by.bk(&daw);

    
    let fru = mz(0.48, 0.15 * e, Al);
    let ddz = by.len(); by.bk(&fru);

    let (bb, aa) = ala(wu, of, bo); ks.lg(bb); bu.lg(aa);
    let (bb, aa) = ala(of, tb, bo); ks.lg(bb); bu.lg(aa);
    let (bb, aa) = ala(tb, ajw, bo); ks.lg(bb); bu.lg(aa);
    let (bb, aa) = ala(ajw, bay, bo); ks.lg(bb); bu.lg(aa);
    let (bb, aa) = ala(bay, bjh, bo); ks.lg(bb); bu.lg(aa);
    let (bb, aa) = ala(bjh, ddz, bo); ks.lg(bb); bu.lg(aa);

    
    
    let azk = 0.04;
    let hek = 0.14;
    let eab = 0.50;
    let to = by.len();
    by.push(V3 { b: -azk, c: eab, av: -azk });
    by.push(V3 { b:  azk, c: eab, av: -azk });
    by.push(V3 { b:  azk, c: eab + hek, av: -azk });
    by.push(V3 { b: -azk, c: eab + hek, av: -azk });
    by.push(V3 { b: -azk, c: eab, av:  azk });
    by.push(V3 { b:  azk, c: eab, av:  azk });
    by.push(V3 { b:  azk, c: eab + hek, av:  azk });
    by.push(V3 { b: -azk, c: eab + hek, av:  azk });

    
    ks.push((to, to+1, to+2)); ks.push((to, to+2, to+3));      
    ks.push((to+4, to+6, to+5)); ks.push((to+4, to+7, to+6));  
    ks.push((to, to+3, to+7)); ks.push((to, to+7, to+4));      
    ks.push((to+1, to+5, to+6)); ks.push((to+1, to+6, to+2));  
    ks.push((to+3, to+2, to+6)); ks.push((to+3, to+6, to+7));  

    
    bu.push((to, to+1)); bu.push((to+1, to+2)); bu.push((to+2, to+3)); bu.push((to+3, to));
    bu.push((to+4, to+5)); bu.push((to+5, to+6)); bu.push((to+6, to+7)); bu.push((to+7, to+4));
    bu.push((to, to+4)); bu.push((to+1, to+5)); bu.push((to+2, to+6)); bu.push((to+3, to+7));

    
    let pl = eab + hek * 0.55;
    let lo = 0.10;
    let ww = by.len();
    by.push(V3 { b: -lo, c: pl, av: -azk });
    by.push(V3 { b:  lo, c: pl, av: -azk });
    by.push(V3 { b:  lo, c: pl + 0.03, av: -azk });
    by.push(V3 { b: -lo, c: pl + 0.03, av: -azk });
    by.push(V3 { b: -lo, c: pl, av:  azk });
    by.push(V3 { b:  lo, c: pl, av:  azk });
    by.push(V3 { b:  lo, c: pl + 0.03, av:  azk });
    by.push(V3 { b: -lo, c: pl + 0.03, av:  azk });

    ks.push((ww, ww+1, ww+2)); ks.push((ww, ww+2, ww+3));
    ks.push((ww+4, ww+6, ww+5)); ks.push((ww+4, ww+7, ww+6));
    ks.push((ww, ww+3, ww+7)); ks.push((ww, ww+7, ww+4));
    ks.push((ww+1, ww+5, ww+6)); ks.push((ww+1, ww+6, ww+2));
    ks.push((ww+3, ww+2, ww+6)); ks.push((ww+3, ww+6, ww+7));

    bu.push((ww, ww+1)); bu.push((ww+1, ww+2)); bu.push((ww+2, ww+3)); bu.push((ww+3, ww));
    bu.push((ww+4, ww+5)); bu.push((ww+5, ww+6)); bu.push((ww+6, ww+7)); bu.push((ww+7, ww+4));

    
    let xjq = by.len();
    by.push(V3 { b: 0.0, c: 0.48, av: 0.0 });
    let (bb, aa) = cwn(ddz, xjq, bo); ks.lg(bb); bu.lg(aa);

    
    let abm = by.len();
    by.push(V3 { b: 0.0, c: 0.0, av: 0.0 });
    let (bb, aa) = cwn(wu, abm, bo); ks.lg(bb); bu.lg(aa);

    Hz { lm: by, bu: Some(bu), ks: Some(ks) }
}





fn fir(k: &mut [u32], d: usize, i: usize,
                       mut fy: i32, mut fo: i32,
                       mut dn: i32, mut dp: i32,
                       mut hy: i32, mut jz: i32,
                       s: u32) {
    
    if fo > dp { core::mem::swap(&mut fy, &mut dn); core::mem::swap(&mut fo, &mut dp); }
    if dp > jz { core::mem::swap(&mut dn, &mut hy); core::mem::swap(&mut dp, &mut jz); }
    if fo > dp { core::mem::swap(&mut fy, &mut dn); core::mem::swap(&mut fo, &mut dp); }

    let aku = jz - fo;
    if aku == 0 { return; }

    let bpl = fo.am(0);
    let dno = jz.v(i as i32 - 1);

    for c in bpl..=dno {
        let hzh = c >= dp;
        let ftz = if hzh { jz - dp } else { dp - fo };

        let mjn = (c - fo) as f32 / aku as f32;
        let bpj = fy as f32 + (hy - fy) as f32 * mjn;

        let dnk = if ftz == 0 {
            bpj
        } else if hzh {
            let fwa = (c - dp) as f32 / ftz as f32;
            dn as f32 + (hy - dn) as f32 * fwa
        } else {
            let fwa = (c - fo) as f32 / ftz as f32;
            fy as f32 + (dn - fy) as f32 * fwa
        };

        let mut fd = bpj as i32;
        let mut hw = dnk as i32;
        if fd > hw { core::mem::swap(&mut fd, &mut hw); }

        fd = fd.am(0);
        hw = hw.v(d as i32 - 1);

        let br = c as usize * d;
        for b in fd..=hw {
            let w = br + b as usize;
            if w < k.len() {
                k[w] = s;
            }
        }
    }
}

use crate::draw_utils::{ahj as dgr, abc as ssh};


fn hng(k: &mut [u32], d: usize, i: usize, b: i32, c: i32, text: &str, s: u32) {
    let mut cx = b;
    for bm in text.bw() {
        kqw(k, d, i, cx, c, bm, s);
        cx += 8;
    }
}


fn kqw(k: &mut [u32], d: usize, i: usize, b: i32, c: i32, r: char, s: u32) {
    let ka = crate::framebuffer::font::ada(r);
    for br in 0..16 {
        let fs = ka[br];
        for bj in 0..8 {
            if fs & (0x80 >> bj) != 0 {
                let y = b + bj as i32;
                let x = c + br as i32;
                if y >= 0 && x >= 0 && y < d as i32 && x < i as i32 {
                    let w = x as usize * d + y as usize;
                    if w < k.len() {
                        k[w] = s;
                    }
                }
            }
        }
    }
}
