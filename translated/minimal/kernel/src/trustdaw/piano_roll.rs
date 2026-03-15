








use alloc::format;
use alloc::string::String;
use super::track::{Track, Note};
use super::{AE_, Hi};
use core::sync::atomic::Ordering;






const BRD_: u32 = 1;

const BRC_: u32 = 8;

const UK_: u32 = 48;

const QE_: u32 = 24;

const AFP_: u8 = 24;  

const CFK_: u8 = 96;  


mod colors {
    pub const Apk: u32 = 0x1A1A2E;
    pub const BYG_: u32 = 0x2A2A3E;
    pub const BYF_: u32 = 0x3A3A4E;
    pub const ATV_: u32 = 0x5A5A6E;
    pub const DBL_: u32 = 0x222236;
    pub const BLE_: u32 = 0x181828;
    pub const CDC_: u32 = 0x101020;
    pub const ADT_: u32 = 0xCCCCDD;
    pub const ADS_: u32 = 0x333344;
    pub const CDD_: u32 = 0x888899;
    pub const CYD_: u32 = 0x151530;
    pub const CYE_: u32 = 0xAAAABB;
    pub const DVE_: u32 = 0xFFFFFF;
    pub const Bok: u32 = 0xFF4444;
    pub const Cle: u32 = 0x4488FF;
    pub const Bdg: u32 = 0x44FF44;
}






pub struct PianoRoll {
    
    pub b: u32,
    
    pub c: u32,
    
    pub z: u32,
    
    pub ac: u32,
    
    pub drl: u32,
    
    pub clf: u32,
    
    pub cms: u32,
    
    pub ug: u8,
    
    pub phr: Option<usize>,
    
    pub dpl: u32,
    pub dfm: u8,
    
    pub ecg: u32,
}

impl PianoRoll {
    
    pub fn new(b: u32, c: u32, z: u32, ac: u32) -> Self {
        Self {
            b,
            c,
            z,
            ac,
            drl: BRD_,
            clf: BRC_,
            cms: 0,
            ug: AFP_,
            phr: None,
            dpl: 0,
            dfm: 60, 
            ecg: AE_ / 4, 
        }
    }

    
    pub fn po(&self, track: &Track, vje: u32) {
        let gz = crate::framebuffer::AB_.load(Ordering::Relaxed) as u32;
        let kc = crate::framebuffer::Z_.load(Ordering::Relaxed) as u32;
        if gz == 0 || kc == 0 { return; }

        
        let d = self.z.v(gz - self.b);
        let i = self.ac.v(kc - self.c);
        let bqw = self.b + UK_;
        let gip = self.c + QE_;
        let auk = d.ao(UK_);
        let bhc = i.ao(QE_);

        
        crate::framebuffer::ah(self.b, self.c, d, i, colors::Apk);

        
        self.sdr(bhc);

        
        self.fgx(bqw, gip, auk, bhc);

        
        self.sgf(bqw, auk);

        
        self.sel(track, bqw, gip, auk, bhc);

        
        self.sey(vje, bqw, gip, auk, bhc);

        
        self.dqf(bqw, gip, auk, bhc);
    }

    
    fn sdr(&self, bhc: u32) {
        let dis = self.b;
        let fmn = self.c + QE_;
        let dmz = bhc / self.clf;

        crate::framebuffer::ah(dis, fmn, UK_, bhc, colors::CDC_);

        for a in 0..dmz {
            let jb = self.ovx(a);
            if jb > 127 { continue; }

            let afy = fmn + (dmz - 1 - a) * self.clf;
            let din = ofr(jb);

            
            let ube = if din { colors::ADS_ } else { colors::ADT_ };
            crate::framebuffer::ah(dis, afy, UK_ - 2, self.clf, ube);

            
            if jb % 12 == 0 || jb == self.dfm {
                let j = crate::audio::tables::dtf(jb);
                let bvq = crate::audio::tables::efk(jb);
                let cu = format!("{}{}", j, bvq);
                crate::framebuffer::cb(&cu, dis + 4, afy + 1, colors::CDD_);
            }
        }
    }

    
    fn fgx(&self, qz: u32, ub: u32, nt: u32, bjz: u32) {
        let dmz = bjz / self.clf;

        
        for a in 0..dmz {
            let jb = self.ovx(a);
            if jb > 127 { continue; }

            let afy = ub + (dmz - 1 - a) * self.clf;
            let wak = if ofr(jb) {
                colors::BLE_
            } else {
                colors::DBL_
            };
            crate::framebuffer::ah(qz, afy, nt, self.clf, wak);

            
            crate::framebuffer::zs(qz, afy, nt, colors::BYG_);
        }

        
        let cij = AE_ * 4; 
        let mkr = nt / self.drl.am(1);

        let vb = self.cms;
        let ckg = vb + mkr;

        
        let kwj = (vb / cij) * cij;
        let mut or = kwj;
        while or <= ckg {
            let y = self.fwq(or, qz);
            if y >= qz && y < qz + nt {
                crate::framebuffer::axt(y, ub, bjz, colors::ATV_);
            }
            or += cij;
        }

        
        let sty = (vb / AE_) * AE_;
        or = sty;
        while or <= ckg {
            if or % cij != 0 { 
                let y = self.fwq(or, qz);
                if y >= qz && y < qz + nt {
                    crate::framebuffer::axt(y, ub, bjz, colors::BYF_);
                }
            }
            or += AE_;
        }
    }

    
    fn sgf(&self, qz: u32, nt: u32) {
        let ty = self.c;
        crate::framebuffer::ah(self.b, ty, self.z, QE_, colors::CYD_);

        let cij = AE_ * 4;
        let mkr = nt / self.drl.am(1);
        let vb = self.cms;
        let ckg = vb + mkr;

        let kwj = (vb / cij) * cij;
        let mut or = kwj;
        while or <= ckg {
            let qmx = or / cij + 1;
            let y = self.fwq(or, qz);
            if y >= qz && y < qz + nt {
                let cu = format!("{}", qmx);
                crate::framebuffer::cb(&cu, y + 2, ty + 4, colors::CYE_);
                crate::framebuffer::axt(y, ty, QE_, colors::ATV_);
            }
            or += cij;
        }
    }

    
    fn sel(&self, track: &Track, qz: u32, ub: u32, nt: u32, bjz: u32) {
        let dmz = bjz / self.clf;

        for (a, jp) in track.ts.iter().cf() {
            
            let low = self.fwq(jp.vb, qz);
            let oqz = self.fwq(jp.ckg(), qz);
            let uvo = (oqz.ao(low)).am(2);

            
            if jp.jb < self.ug || jp.jb >= self.ug + dmz as u8 {
                continue;
            }

            
            if oqz < qz || low > qz + nt {
                continue;
            }

            let wam = (jp.jb - self.ug) as u32;
            let lox = ub + (dmz - 1 - wam) * self.clf + 1;

            
            let irx = low.am(qz);
            let hgy = uvo.v(qz + nt - irx);

            
            let kt = jp.qm as u32 * 100 / 127;
            let uvd = jzn(track.s, kt);

            
            crate::framebuffer::ah(irx, lox, hgy, self.clf - 2, uvd);

            
            if self.phr == Some(a) {
                crate::framebuffer::lx(irx, lox, hgy, self.clf - 2, colors::Cle);
            }

            
            if hgy > 24 {
                let j = crate::audio::tables::dtf(jp.jb);
                crate::framebuffer::cb(j, irx + 2, lox + 1, 0xFFFFFF);
            }
        }
    }

    
    fn sey(&self, or: u32, qz: u32, ub: u32, nt: u32, bjz: u32) {
        let y = self.fwq(or, qz);
        if y >= qz && y < qz + nt {
            crate::framebuffer::axt(y, ub, bjz, colors::Bok);
            
            for a in 0..4u32 {
                crate::framebuffer::zs(y.ao(a), ub.ao(a + 1), a * 2 + 1, colors::Bok);
            }
        }
    }

    
    fn dqf(&self, qz: u32, ub: u32, nt: u32, bjz: u32) {
        let dmz = bjz / self.clf;

        
        let cx = self.fwq(self.dpl, qz);
        if cx >= qz && cx < qz + nt {
            crate::framebuffer::axt(cx, ub, bjz, colors::Bdg);
        }

        
        if self.dfm >= self.ug && self.dfm < self.ug + dmz as u8 {
            let br = (self.dfm - self.ug) as u32;
            let ae = ub + (dmz - 1 - br) * self.clf;
            crate::framebuffer::ih(qz, ae, nt, self.clf, colors::Bdg, 40);
        }
    }

    

    
    fn fwq(&self, or: u32, bqw: u32) -> u32 {
        if or >= self.cms {
            bqw + (or - self.cms) * self.drl
        } else {
            bqw 
        }
    }

    
    pub fn zfh(&self, y: u32, bqw: u32) -> u32 {
        if y >= bqw && self.drl > 0 {
            self.cms + (y - bqw) / self.drl
        } else {
            self.cms
        }
    }

    
    fn ovx(&self, br: u32) -> u8 {
        let jb = self.ug as u32 + br;
        if jb > 127 { 127 } else { jb as u8 }
    }

    

    
    pub fn mco(&mut self) {
        let kcd = AE_ * 4;
        self.cms = self.cms.ao(kcd);
    }

    
    pub fn mcq(&mut self) {
        let kcd = AE_ * 4;
        self.cms += kcd;
    }

    
    pub fn dlm(&mut self) {
        if self.ug < CFK_ - 12 {
            self.ug += 12; 
        }
    }

    
    pub fn eid(&mut self) {
        if self.ug > AFP_ + 12 {
            self.ug -= 12;
        } else {
            self.ug = AFP_;
        }
    }

    
    pub fn zxx(&mut self) {
        if self.drl < 8 {
            self.drl += 1;
        }
    }

    
    pub fn zxy(&mut self) {
        if self.drl > 1 {
            self.drl -= 1;
        }
    }

    
    pub fn rsg(&mut self) {
        self.dpl += self.ecg;
    }

    
    pub fn rse(&mut self) {
        self.dpl = self.dpl.ao(self.ecg);
    }

    
    pub fn rsk(&mut self) {
        if self.dfm < 127 {
            self.dfm += 1;
        }
    }

    
    pub fn rsd(&mut self) {
        if self.dfm > 0 {
            self.dfm -= 1;
        }
    }

    
    pub fn zou(&mut self) {
        if self.ecg > 0 {
            let dlf = self.dpl % self.ecg;
            if dlf > self.ecg / 2 {
                self.dpl += self.ecg - dlf;
            } else {
                self.dpl -= dlf;
            }
        }
    }

    
    pub fn znb(&mut self, rzg: &str) {
        self.ecg = match rzg {
            "1" | "whole" => AE_ * 4,
            "1/2" | "half" => AE_ * 2,
            "1/4" | "quarter" => AE_,
            "1/8" | "eighth" => AE_ / 2,
            "1/16" | "sixteenth" => AE_ / 4,
            "1/32" | "thirtysecond" => AE_ / 8,
            "off" | "free" => 1,
            _ => self.ecg, 
        };
    }

    
    pub fn yem(&self, track: &mut Track, qm: u8, bbn: u32) {
        let jp = Note::new(self.dfm, qm, self.dpl, bbn);
        track.axn(jp);
    }

    
    pub fn ylp(&self, track: &mut Track) -> bool {
        let uvp = track.uvq(self.dpl);
        if let Some(jp) = uvp.iter().du(|bo| bo.jb == self.dfm) {
            let w = track.ts.iter().qf(|bo|
                bo.jb == jp.jb && bo.vb == jp.vb
            );
            if let Some(w) = w {
                track.pbr(w);
                return true;
            }
        }
        false
    }
}






fn ofr(jb: u8) -> bool {
    oh!(jb % 12, 1 | 3 | 6 | 8 | 10) 
}


fn jzn(s: u32, kt: u32) -> u32 {
    let m = ((s >> 16) & 0xFF) * kt / 100;
    let at = ((s >> 8) & 0xFF) * kt / 100;
    let o = (s & 0xFF) * kt / 100;
    (m.v(255) << 16) | (at.v(255) << 8) | o.v(255)
}


pub fn xfw(track: &Track, cjf: u32) -> String {
    let mut e = String::new();
    let cij = AE_ * 4;
    let cng = cij * cjf;
    let ec = (cjf * 16) as usize; 
    let xgs = cij / 16;

    e.t(&format!("Piano Roll: \"{}\" — {} bars, {} notes\n",
        track.amj(), cjf, track.ts.len()));
    e.t(&format!("Grid: 1/16 note | {} = {}\n\n", track.ve.j(),
        if track.mwg { "ARMED" } else { "" }));

    
    e.t("     │");
    for bar in 0..cjf {
        e.t(&format!("{:^16}", bar + 1));
    }
    e.t("\n     │");
    for _ in 0..cjf {
        e.t("────────────────");
    }
    e.push('\n');

    
    let (uom, ulu) = if track.ts.is_empty() {
        (57, 72) 
    } else {
        let v = track.ts.iter().map(|bo| bo.jb).v().unwrap_or(60);
        let am = track.ts.iter().map(|bo| bo.jb).am().unwrap_or(72);
        (v.ao(2), (am + 2).v(127))
    };

    
    for jb in (uom..=ulu).vv() {
        let j = crate::audio::tables::dtf(jb);
        let bvq = crate::audio::tables::efk(jb);
        let hou = jb % 12 == 0;
        
        e.t(&format!("{}{:<2} {}│", j, bvq,
            if hou { "─" } else { " " }));

        for bj in 0..ec {
            let or = bj as u32 * xgs;
            let gh = track.ts.iter().any(|bo|
                bo.jb == jb && bo.vb <= or && or < bo.ckg()
            );
            let tyj = track.ts.iter().any(|bo|
                bo.jb == jb && bo.vb == or
            );

            if tyj {
                e.push('█');
            } else if gh {
                e.push('▓');
            } else if bj % 16 == 0 {
                e.push('│');
            } else if bj % 4 == 0 {
                e.push('┊');
            } else {
                e.push('·');
            }
        }
        e.push('\n');
    }

    e
}
