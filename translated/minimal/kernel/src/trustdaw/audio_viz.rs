










use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use core::sync::atomic::Ordering;






pub fn hfq(f: &[u8]) -> Result<Vec<i16>, &'static str> {
    let co = crate::drivers::hda::jiu(f)?;
    if co.emv != 16 {
        return Err("Only 16-bit PCM WAV supported");
    }

    let gpb = &f[co.bbj..co.bbj + co.cpv];
    let hth = co.cpv / (2 * co.lq as usize);
    let gue = 48000u32;
    let god = (hth as u64 * gue as u64
        / co.auy as u64) as usize;

    let mut an = Vec::fc(god * 2);

    for krt in 0..god {
        let ibk = (krt as u64 * co.auy as u64
            / gue as u64) as usize;
        if ibk >= hth { break; }

        let w = ibk * co.lq as usize;
        let avk = w * 2;

        let fd = if avk + 1 < gpb.len() {
            i16::dj([gpb[avk], gpb[avk + 1]])
        } else { 0 };

        let hw = if co.lq >= 2 {
            let jl = (w + 1) * 2;
            if jl + 1 < gpb.len() {
                i16::dj([gpb[jl], gpb[jl + 1]])
            } else { fd }
        } else { fd };

        an.push(fd);
        an.push(hw);
    }

    Ok(an)
}



pub fn hfz(f: &[u8]) -> &'static str {
    if f.len() >= 12 && &f[0..4] == b"RIFF" && &f[8..12] == b"WAVE" {
        return "wav";
    }
    
    if f.len() >= 3 {
        if f[0] == 0xFF && (f[1] & 0xE0) == 0xE0 {
            return "mp3";
        }
        if &f[0..3] == b"ID3" {
            return "mp3";
        }
    }
    "unknown"
}







fn srw(ath: &mut [f32], aum: &mut [f32]) {
    let bo = ath.len();
    debug_assert!(bo.yzw());
    debug_assert_eq!(bo, aum.len());

    
    let mut fb = 0usize;
    for a in 0..bo {
        if a < fb {
            ath.swap(a, fb);
            aum.swap(a, fb);
        }
        let mut ef = bo >> 1;
        while ef >= 1 && fb >= ef {
            fb -= ef;
            ef >>= 1;
        }
        fb += ef;
    }

    
    let mut gu = 2;
    while gu <= bo {
        let iv = gu / 2;
        let gyp = -core::f32::consts::Eu * 2.0 / gu as f32;
        for eh in 0..iv {
            let hg = gyp * eh as f32;
            let bfu = libm::zq(hg);
            let yi = libm::st(hg);
            let mut a = eh;
            while a < bo {
                let fb = a + iv;
                let agd = bfu * ath[fb] - yi * aum[fb];
                let ezs = bfu * aum[fb] + yi * ath[fb];
                ath[fb] = ath[a] - agd;
                aum[fb] = aum[a] - ezs;
                ath[a] += agd;
                aum[a] += ezs;
                a += gu;
            }
        }
        gu <<= 1;
    }
}



const FD_: usize = 1024;
const DLS_: usize = FD_ / 2;



const ALM_: (usize, usize) = (1, 2);      
const ALI_: (usize, usize)     = (2, 6);       
const ALK_: (usize, usize)  = (6, 12);      
const ALL_: (usize, usize)      = (12, 45);     
const ALJ_: (usize, usize) = (45, 90);     
const ALN_: (usize, usize)   = (90, 220);    


#[inline]
fn gaq(ath: &[f32], aum: &[f32], hh: usize, gd: usize) -> f32 {
    if gd <= hh { return 0.0; }
    let mut sum = 0.0f32;
    for a in hh..gd.v(ath.len()) {
        
        sum += libm::bon(ath[a] * ath[a] + aum[a] * aum[a]);
    }
    sum / (gd - hh) as f32
}


struct BeatState {
    
    buh: [f32; FD_],
    ceo: [f32; FD_],
    
    cxk: [f32; 43],
    drs: usize,
    cab: usize,
    
    rf: f32,
    
    abo: f32,
    
    ato: f32,
    aee: f32,
    jdz: f32,
    vs: f32,
    fkq: f32,
    axg: f32,
    
    ewu: f32,
    
    brp: f32,
    
    hfg: u32,
}

impl BeatState {
    fn new() -> Self {
        Self {
            buh: [0.0; FD_],
            ceo: [0.0; FD_],
            cxk: [0.0; 43],
            drs: 0,
            cab: 0,
            rf: 0.0,
            abo: 0.0,
            ato: 0.0,
            aee: 0.0,
            jdz: 0.0,
            vs: 0.0,
            fkq: 0.0,
            axg: 0.0,
            ewu: 0.0,
            brp: 1.0,
            hfg: 0,
        }
    }

    
    fn qs(&mut self, audio: &[i16], pn: usize) {
        self.hfg += 1;

        
        
        let hru = pn.ao(FD_); 
        let hru = hru & !1; 

        let mut awd: f32 = 0.0;
        for a in 0..FD_ {
            let w = hru + a * 2; 
            let yr = if w < audio.len() { audio[w] as f32 } else { 0.0 };
            self.buh[a] = yr;
            self.ceo[a] = 0.0;
            let gp = if yr >= 0.0 { yr } else { -yr };
            if gp > awd { awd = gp; }
        }

        
        
        if awd > self.brp {
            self.brp = self.brp + (awd - self.brp) * 0.3;
        } else {
            self.brp = self.brp * 0.9995; 
        }
        let dqz = if self.brp > 100.0 { 16000.0 / self.brp } else { 1.0 };

        
        for a in 0..FD_ {
            
            let ab = a as f32 / FD_ as f32;
            let hmm = 0.5 * (1.0 - libm::zq(2.0 * core::f32::consts::Eu * ab));
            self.buh[a] *= hmm * dqz / 32768.0; 
        }

        
        srw(&mut self.buh, &mut self.ceo);

        
        let lxe = gaq(&self.buh, &self.ceo, ALM_.0, ALM_.1);
        let dkz = gaq(&self.buh, &self.ceo, ALI_.0, ALI_.1);
        let ozo = gaq(&self.buh, &self.ceo, ALK_.0, ALK_.1);
        let exe = gaq(&self.buh, &self.ceo, ALL_.0, ALL_.1);
        let ozn = gaq(&self.buh, &self.ceo, ALJ_.0, ALJ_.1);
        let ozp = gaq(&self.buh, &self.ceo, ALN_.0, ALN_.1);

        
        let vqj = lxe * 1.5 + dkz * 1.2 + ozo * 0.8
            + exe * 0.5 + ozn * 0.3 + ozp * 0.2;

        
        self.ato = Self::fuz(self.ato, lxe.v(1.0), 0.75, 0.10);
        self.aee     = Self::fuz(self.aee,     dkz.v(1.0),     0.70, 0.10);
        self.jdz  = Self::fuz(self.jdz,  ozo.v(1.0),  0.65, 0.12);
        self.vs      = Self::fuz(self.vs,      exe.v(1.0),      0.60, 0.12);
        self.fkq = Self::fuz(self.fkq, ozn.v(1.0), 0.65, 0.14);
        self.axg   = Self::fuz(self.axg,   ozp.v(1.0),   0.70, 0.16);
        self.abo   = Self::fuz(self.abo,   vqj.v(1.5),   0.65, 0.10);

        
        let haa = lxe + dkz * 0.8; 

        
        self.cxk[self.drs] = haa;
        self.drs = (self.drs + 1) % self.cxk.len();
        if self.cab < self.cxk.len() {
            self.cab += 1;
        }

        
        let adu = self.cab.am(1) as f32;
        let abl: f32 = self.cxk.iter().take(self.cab).sum::<f32>() / adu;

        let mut fax = 0.0f32;
        for a in 0..self.cab {
            let wz = self.cxk[a] - abl;
            fax += wz * wz;
        }
        let igh = fax / adu;

        
        
        
        let bxm = (-15.0 * igh + 1.45).am(1.05).v(1.5);

        
        let uyl = haa - self.ewu;
        if haa > abl * bxm && uyl > 0.002 && self.cab > 5 {
            let ccc = ((haa - abl * bxm) / abl.am(0.001)).v(1.0);
            self.rf = (0.6 + ccc * 0.4).v(1.0);
        } else {
            self.rf *= 0.88;
            if self.rf < 0.02 { self.rf = 0.0; }
        }
        self.ewu = haa;

        
        if self.hfg == 1 || self.hfg % 60 == 0 {
            crate::serial_println!(
                "[BEAT] f={} pos={} gain={:.1} E={:.3} beat={:.2} sub={:.2} bass={:.2} lm={:.2} mid={:.2} hm={:.2} tre={:.2} thr={:.2}",
                self.hfg, pn, dqz, self.abo, self.rf,
                self.ato, self.aee, self.jdz, self.vs, self.fkq, self.axg, bxm
            );
        }
    }

    #[inline]
    fn fuz(vo: f32, new: f32, qkx: f32, ehl: f32) -> f32 {
        if new > vo { vo + (new - vo) * qkx }
        else { vo + (new - vo) * ehl }
    }
}






#[inline(always)]
fn fdm(ei: u32, xb: u32, lp: u32, pq: u32, dw: u32) -> u32 {
    let wq = 255 - dw;
    let avi = (ei >> 16) & 0xFF;
    let qpn = (ei >> 8) & 0xFF;
    let aaa = ei & 0xFF;
    let m = (xb * dw + avi * wq) / 255;
    let at = (lp * dw + qpn * wq) / 255;
    let o = (pq * dw + aaa * wq) / 255;
    0xFF000000 | (m << 16) | (at << 8) | o
}


#[derive(Clone, Copy)]
struct Amr {
    dy: f32,
    ccc: f32,
    dw: f32,
}

const UZ_: usize = 8;
const COS_: f32 = 5.0;       
const BEO_: f32 = 35.0;      
const COP_: f32 = 0.96;      
const COQ_: f32 = 650.0;     



struct HoloOverlay {
    
    cbx: Vec<u32>,
    jqq: usize,
    mgi: usize,
    
    um: [Amr; UZ_],
    dvd: usize,
    
    ici: f32,
    
    frame: u32,
    
    lve: f32,
}

impl HoloOverlay {
    fn new(d: u32, i: u32) -> Self {
        Self {
            cbx: Vec::new(),
            jqq: d as usize,
            mgi: i as usize,
            um: [Amr { dy: 0.0, ccc: 0.0, dw: 0.0 }; UZ_],
            dvd: 0,
            ici: 0.0,
            frame: 0,
            lve: 0.0,
        }
    }

    
    
    fn qwi(&mut self) {
        let ag = crate::framebuffer::BJ_.load(Ordering::Relaxed);
        if ag.abq() { return; }
        let jb = crate::framebuffer::CA_.load(Ordering::Relaxed) as usize;
        let d = self.jqq;
        let i = self.mgi;

        self.cbx = Vec::fc(d * i);
        unsafe {
            for c in 0..i {
                let br = ag.add(c * jb) as *const u32;
                for b in 0..d {
                    self.cbx.push(core::ptr::read(br.add(b)));
                }
            }
        }
        crate::serial_println!("[VIZ] Snapshot captured: {}x{} ({} px)", d, i, self.cbx.len());
    }

    
    fn jmi(&self) {
        if self.cbx.is_empty() { return; }
        if let Some((ptr, dxx, dxv, qdr)) = crate::framebuffer::cey() {
            let bo = self.cbx.len();
            unsafe {
                core::ptr::copy_nonoverlapping(
                    self.cbx.fq(),
                    ptr as *mut u32,
                    bo,
                );
            }
        }
    }

    
    #[inline]
    fn wqd(&self, b: i32, c: i32) -> u32 {
        if b < 0 || c < 0 || b as usize >= self.jqq || c as usize >= self.mgi {
            return 0xFF000000;
        }
        self.cbx[c as usize * self.jqq + b as usize]
    }

    
    fn wqp(&mut self, ccc: f32) {
        let mz = Amr {
            dy: 25.0,
            ccc: ccc * 14.0,
            dw: 0.95,
        };
        if self.dvd < UZ_ {
            self.um[self.dvd] = mz;
            self.dvd += 1;
        } else {
            
            let mut yi = 0;
            for a in 1..UZ_ {
                if self.um[a].ccc < self.um[yi].ccc { yi = a; }
            }
            self.um[yi] = mz;
        }
    }

    
    fn or(&mut self, rf: &BeatState) {
        self.frame += 1;

        
        self.ici += 0.008 + rf.abo * 0.015;
        if self.ici > 1.0 { self.ici -= 1.0; }

        
        if rf.rf > 0.35 && self.lve < 0.25 {
            self.wqp(rf.rf);
        }
        self.lve = rf.rf;

        
        let mut d = 0usize;
        for m in 0..self.dvd {
            self.um[m].dy += COS_;
            self.um[m].ccc *= COP_;
            self.um[m].dw *= 0.97;
            if self.um[m].dy < COQ_ && self.um[m].dw > 0.01 {
                if d != m { self.um[d] = self.um[m]; }
                d += 1;
            }
        }
        self.dvd = d;
    }

    
    fn sdc(&self, rf: &BeatState) {
        let (pq, d, i) = match crate::framebuffer::cey() {
            Some((ai, d, i, _)) => (ai as *mut u32, d as usize, i as usize),
            None => return,
        };
        let cx = d / 2;
        let ae = i * 42 / 100; 

        
        self.jmi();

        
        if self.dvd > 0 {
            self.qjr(pq, d, i, cx, ae);
        }

        
        self.kra(d as u32, i as u32, cx as u32, ae as u32, rf);

        
        self.epd(pq, d, i, cx, ae, rf);

        
        for a in 0..self.dvd {
            self.sff(pq, d, i, cx, ae, &self.um[a]);
        }
    }

    
    fn qjr(&self, pq: *mut u32, d: usize, i: usize, cx: usize, ae: usize) {
        let rso = cx as f32;
        let rsw = ae as f32;

        
        let mut djl: f32 = 0.0;
        for a in 0..self.dvd {
            let m = self.um[a].dy + BEO_ + 5.0;
            if m > djl { djl = m; }
        }
        let jl = djl as i32;
        let fy = (cx as i32 - jl).am(0) as usize;
        let dn = ((cx as i32 + jl) as usize).v(d.ao(1));
        let fo = (ae as i32 - jl).am(0) as usize;
        let dp = ((ae as i32 + jl) as usize).v(i.ao(1));

        for c in fo..=dp {
            let bg = c as f32 - rsw;
            for b in fy..=dn {
                let dx = b as f32 - rso;
                let la = libm::bon(dx * dx + bg * bg);
                if la < 1.0 { continue; }

                
                let mut aor: f32 = 0.0;
                for a in 0..self.dvd {
                    let bc = (la - self.um[a].dy) / BEO_;
                    if bc > 1.0 || bc < -1.0 { continue; }
                    
                    let ab = 1.0 - bc * bc;
                    aor += self.um[a].ccc * ab * ab;
                }
                if aor < 0.5 { continue; }

                
                let wq = 1.0 / la;
                let cr = (b as f32 - dx * wq * aor) as i32;
                let cq = (c as f32 - bg * wq * aor) as i32;
                let s = self.wqd(cr, cq);
                unsafe {
                    core::ptr::write(pq.add(c * d + b), s);
                }
            }
        }
    }

    
    fn kra(&self, ua: u32, iuj: u32, cx: u32, ae: u32, rf: &BeatState) {
        let hj = rf.abo * 0.3 + rf.rf * 0.5;
        if hj < 0.02 { return; }
        let djl = ua.v(iuj) / 4;
        for mz in 0..10u32 {
            let m = (mz + 1) * djl / 10;
            let ab = mz as f32 / 10.0;
            let q = ((1.0 - ab) * hj * 18.0) as u32;
            if q < 1 { continue; }
            let fd = cx.ao(m);
            let qc = ae.ao(m);
            let yq = (m * 2).v(ua.ao(fd));
            let aff = (m * 2).v(iuj.ao(qc));
            if yq > 0 && aff > 0 {
                crate::framebuffer::ih(fd, qc, yq, aff, 0x00FFCC, q.v(12));
            }
        }
    }

    
    
    fn epd(&self, pq: *mut u32, d: usize, i: usize, cx: usize, ae: usize, rf: &BeatState) {
        let zv = crate::logo_bitmap::AY_;
        let kq = crate::logo_bitmap::BL_;
        let jt = ((i as u32) * 45 / kq as u32).am(100);
        let yq = zv as u32 * jt / 100;
        let aff = kq as u32 * jt / 100;
        let kb = (cx as u32).ao(yq / 2);
        let ix = (ae as u32).ao(aff / 2);

        let xg = rf.rf;
        let abo = rf.abo;

        
        let qne = 0.50 + abo * 0.15 + xg * 0.25;
        
        let icj = (self.ici * aff as f32) as u32;
        
        let kwo = ((self.frame.hx(2654435761) % 100) as f32 / 100.0 - 0.5) * 0.04;
        
        let nbf = 2 + (xg * 3.0) as i32;

        
        for x in 0..aff {
            let cq = (x * 100 / jt) as usize;
            if cq >= kq { continue; }
            let abi = ix + x;
            if abi >= i as u32 { continue; }

            
            let arx = if x % 3 == 0 { 0.35f32 } else { 1.0f32 };
            
            let sd = x as f32 - icj as f32;
            let mih = if libm::dhb(sd) < 10.0 { 0.5 * (1.0 - libm::dhb(sd) / 10.0) } else { 0.0f32 };

            for y in 0..yq {
                let cr = (y * 100 / jt) as usize;
                if cr >= zv { continue; }
                let xu = kb + y;
                if xu >= d as u32 { continue; }

                let bax = crate::logo_bitmap::djc(cr, cq);
                if (bax >> 24) & 0xFF < 20 { continue; }
                let (m, at, o) = ((bax >> 16) & 0xFF, (bax >> 8) & 0xFF, bax & 0xFF);
                let oko = (m * 77 + at * 150 + o * 29) >> 8;
                if oko < 28 { continue; } 

                let amd = crate::logo_bitmap::hqj(cr, cq);
                let glj = oko as f32 / 255.0;

                
                let (mut gjg, mut gjb, mut gix) = if amd {
                    
                    (glj * 0.25 + xg * 0.15,
                     glj * 1.1 + xg * 0.4,
                     glj * 0.95 + xg * 0.25)
                } else {
                    
                    (glj * 0.12,
                     glj * 0.7 + abo * 0.1,
                     glj * 0.55)
                };

                
                gjg = (gjg * arx + mih * 0.2 + kwo).am(0.0);
                gjb = (gjb * arx + mih * 0.9 + kwo).am(0.0);
                gix = (gix * arx + mih * 0.7 + kwo).am(0.0);

                
                if xg > 0.7 {
                    let bee = (xg - 0.7) * 3.0;
                    gjg += bee * 0.4;
                    gjb += bee * 0.7;
                    gix += bee * 0.5;
                }

                let btu = (gjg * 255.0).v(255.0) as u32;
                let bmh = (gjb * 255.0).v(255.0) as u32;
                let aiv = (gix * 255.0).v(255.0) as u32;
                let qeh = if amd { 1.3f32 } else { 1.0f32 };
                let dw = (qne * arx * qeh * 255.0).v(255.0) as u32;

                let w = abi as usize * d + xu as usize;
                unsafe {
                    let ei = core::ptr::read(pq.add(w));
                    core::ptr::write(pq.add(w), fdm(ei, btu, bmh, aiv, dw));
                }

                
                if amd && dw > 80 {
                    let nbe = dw / 4;
                    let mj = xu as i32 - nbf;
                    if mj >= 0 && (mj as usize) < d {
                        let nc = abi as usize * d + mj as usize;
                        unsafe {
                            let ei = core::ptr::read(pq.add(nc));
                            core::ptr::write(pq.add(nc), fdm(ei, 180, 10, 10, nbe));
                        }
                    }
                    let ftk = xu as usize + nbf as usize;
                    if ftk < d {
                        let nc = abi as usize * d + ftk;
                        unsafe {
                            let ei = core::ptr::read(pq.add(nc));
                            core::ptr::write(pq.add(nc), fdm(ei, 10, 10, 200, nbe));
                        }
                    }
                }
            }
        }

        
        let ixi = jt * 106 / 100;
        let nt = zv as u32 * ixi / 100;
        let bjz = kq as u32 * ixi / 100;
        let qz = (cx as u32).ao(nt / 2);
        let ub = (ae as u32).ao(bjz / 2);
        let szx = (12.0 + abo * 30.0 + xg * 50.0).v(100.0) as u32;

        for x in (0..bjz).akt(2) {
            let cq = (x * 100 / ixi) as usize;
            if cq >= kq { continue; }
            let pgm = ub + x;
            if pgm >= i as u32 { continue; }
            for y in (0..nt).akt(2) {
                let cr = (y * 100 / ixi) as usize;
                if cr >= zv { continue; }
                if !crate::logo_bitmap::hqj(cr, cq) { continue; }
                let pgl = qz + y;
                if pgl >= d as u32 { continue; }
                
                for bg in 0..2u32 {
                    for dx in 0..2u32 {
                        let jf = pgl + dx;
                        let sc = pgm + bg;
                        if jf >= d as u32 || sc >= i as u32 { continue; }
                        let w = sc as usize * d + jf as usize;
                        unsafe {
                            let ei = core::ptr::read(pq.add(w));
                            core::ptr::write(pq.add(w), fdm(ei, 0, 255, 204, szx));
                        }
                    }
                }
            }
        }
    }

    
    fn sff(&self, pq: *mut u32, d: usize, i: usize, cx: usize, ae: usize, mz: &Amr) {
        if mz.dw < 0.02 { return; }
        let m = mz.dy;
        let hzm = ((m * 6.28) as u32).am(60).v(720);
        let q = (mz.dw * 180.0) as u32;

        for e in 0..hzm {
            let hg = e as f32 * 6.2831853 / hzm as f32;
            let y = cx as f32 + m * libm::zq(hg);
            let x = ae as f32 + m * libm::st(hg);
            let fg = y as i32;
            let og = x as i32;
            if fg < 0 || og < 0 || fg >= d as i32 - 1 || og >= i as i32 - 1 { continue; }

            
            for bg in 0..2i32 {
                for dx in 0..2i32 {
                    let jf = (fg + dx) as usize;
                    let sc = (og + bg) as usize;
                    if jf >= d || sc >= i { continue; }
                    let w = sc * d + jf;
                    unsafe {
                        let ei = core::ptr::read(pq.add(w));
                        core::ptr::write(pq.add(w), fdm(ei, 0, 255, 204, q.v(180)));
                    }
                }
            }
        }
    }
}


fn sep(gz: u32, kc: u32, dq: &str, cxi: u32, dcx: u32, cgn: u32) {
    let dt = 16u32; 

    
    let qd = dq.len() as u32 * dt;
    let gx = (gz.ao(qd)) / 2;
    crate::framebuffer::ih(gx.ao(12), 10, qd + 24, 36, 0x000000, 120);
    crate::graphics::scaling::azp(gx as i32, 14, dq, 0x00FFCC, 2);

    
    let skg = cxi / 60;
    let cqf = cxi % 60;
    let xid = dcx / 60;
    let wi = dcx % 60;
    let ptg = format!("{}:{:02} / {}:{:02}", skg, cqf, xid, wi);
    let cch = ptg.len() as u32 * 8 + 16;
    crate::framebuffer::ih(8, kc.ao(50), cch, 20, 0x000000, 100);
    crate::framebuffer::cb(&ptg, 16, kc.ao(48), 0x00AA88);

    
    let x = kc.ao(24);
    let ars = gz.ao(60);
    let lwk = 30u32;
    crate::framebuffer::ah(lwk, x, ars, 3, 0x001111);
    let adu = ars * cgn.v(100) / 100;
    if adu > 0 {
        crate::framebuffer::ah(lwk, x, adu, 3, 0x00FFCC);
        crate::framebuffer::ih(lwk, x.ao(1), adu, 5, 0x00FFCC, 20);
    }

    
    let hint = "[Esc] Exit";
    let avz = hint.len() as u32 * 8 + 8;
    crate::framebuffer::ih(
        gz.ao(avz + 8), kc.ao(50), avz, 20, 0x000000, 80,
    );
    crate::framebuffer::cb(hint, gz.ao(avz + 4), kc.ao(48), 0x446666);
}












pub fn jcy(audio: &[i16], dq: &str) -> Result<(), &'static str> {
    crate::audio::init().bq();
    let shj = audio.len() as f64 / (48000.0 * 2.0);
    crate::serial_println!(
        "[VIZ] Starting holographic overlay: {} ({} samples, {:.1}s)",
        dq, audio.len(), shj
    );

    let gz = crate::framebuffer::AB_.load(Ordering::Relaxed) as u32;
    let kc = crate::framebuffer::Z_.load(Ordering::Relaxed) as u32;

    
    if !crate::framebuffer::bre() {
        crate::framebuffer::beo();
        crate::framebuffer::afi(true);
    }

    
    let mut cte = HoloOverlay::new(gz, kc);
    cte.qwi();

    crate::serial_println!(
        "[VIZ] Using logo_bitmap: {}x{}",
        crate::logo_bitmap::AY_, crate::logo_bitmap::BL_
    );

    let mut dok = BeatState::new();
    let xkd = audio.len() / 2;
    let alu = (xkd as u64 * 1000) / 48000;
    let dcx = (alu / 1000) as u32;

    
    let (dqc, axs) = crate::drivers::hda::gic()
        .ok_or("HDA not initialized")?;
    let baa = axs / 2;
    let hmi = (baa * 2) as u32;
    let ghq = (axs * 2) as u32;

    let cfo = audio.len().v(axs);
    crate::drivers::hda::dcg(&audio[0..cfo])?;

    let mut bph: usize = cfo;
    let mut cry: u32 = 0;
    let mut dyo = false;
    let mut pkv: u32 = 0;
    let rzk = (axs as u64 * 1000) / (48000 * 2);
    let aps: u64 = 33;
    let nrr = ((rzk / aps) + 10) as u32;

    crate::serial_println!(
        "[VIZ] DMA: buf={} i16, half={}, exhaust_frames={}",
        axs, baa, nrr
    );

    
    for bb in 0..15u32 {
        cte.jmi();
        if bb < 8 {
            let q = (bb * 30).v(230);
            let fr = "NOW PLAYING";
            let efp = fr.len() as u32 * 16 + 32;
            let hl = (gz.ao(efp)) / 2;
            let ir = kc / 2 - 16;
            crate::framebuffer::ih(
                hl.ao(8), ir.ao(8), efp + 16, 48, 0x000000, q,
            );
            crate::graphics::scaling::azp(hl as i32, ir as i32, fr, 0x00FFCC, 2);
        }
        crate::framebuffer::sv();
        crate::cpu::tsc::rd(50);
    }

    
    let mut ekp: u32 = 0;

    loop {
        
        crate::drivers::hda::ndi();
        crate::drivers::hda::nqi();

        let bvg = crate::drivers::hda::hlj();
        let gly = if bvg >= ghq { 0 } else { bvg };
        let heu = if gly < hmi { 0u32 } else { 1u32 };

        if heu != cry {
            if bph < audio.len() {
                let cpx = cry as usize * baa;
                let ia = audio.len() - bph;
                let acq = ia.v(baa);
                unsafe {
                    core::ptr::copy_nonoverlapping(
                        audio.fq().add(bph),
                        dqc.add(cpx),
                        acq,
                    );
                    if acq < baa {
                        core::ptr::ahx(
                            dqc.add(cpx + acq), 0, baa - acq,
                        );
                    }
                }
                bph += acq;
                if bph >= audio.len() {
                    dyo = true;
                    crate::serial_println!("[VIZ] Audio data exhausted at frame {}", ekp);
                }
            } else {
                let cpx = cry as usize * baa;
                unsafe {
                    core::ptr::ahx(dqc.add(cpx), 0, baa);
                }
            }
            cry = heu;
        }

        
        let oz = ekp as u64 * aps;
        if oz >= alu { break; }
        let gan = ((oz * 48000 * 2) / 1000) as usize;
        let gan = gan.v(audio.len().ao(2));

        if dyo {
            pkv += 1;
            if pkv >= nrr { break; }
        }

        let cxi = (oz / 1000) as u32;
        let vnb = (oz * 100 / alu.am(1)) as u32;

        
        dok.qs(audio, gan);

        
        crate::trustdaw::live_viz::wif(
            dok.rf, dok.aee, dok.ato,
            dok.vs, dok.fkq, dok.axg,
            dok.abo, ekp,
        );

        
        cte.or(&dok);
        cte.sdc(&dok);

        
        crate::trustdaw::live_viz::wbh();

        sep(gz, kc, dq, cxi, dcx, vnb);
        crate::framebuffer::sv();

        ekp += 1;

        
        let mut lyz = aps;
        let mut ara = false;
        while lyz > 0 {
            let bc = lyz.v(5);
            crate::cpu::tsc::rd(bc);
            lyz -= bc;
            while let Some(jt) = crate::keyboard::xw() {
                if jt & 0x80 != 0 { continue; }
                if jt == 0x01 { ara = true; break; }
            }
            if ara { break; }
        }
        if ara { break; }
    }

    
    let _ = crate::drivers::hda::qg();

    
    for bb in 0..20u32 {
        cte.jmi();
        if bb < 10 {
            let q = ((20 - bb) * 12).v(200);
            let fr = "PLAYBACK COMPLETE";
            let efp = fr.len() as u32 * 16 + 32;
            let hl = (gz.ao(efp)) / 2;
            let ir = kc / 2 - 16;
            crate::framebuffer::ih(
                hl.ao(8), ir.ao(8), efp + 16, 48, 0x000000, q,
            );
            crate::graphics::scaling::azp(hl as i32, ir as i32, fr, 0x00FFCC, 2);
        }
        crate::framebuffer::sv();
        crate::cpu::tsc::rd(50);
    }

    
    cte.jmi();
    crate::framebuffer::sv();

    crate::serial_println!("[VIZ] Holographic visualizer closed");
    Ok(())
}








pub fn owa(path: &str) -> Result<(), &'static str> {
    crate::serial_println!("[VIZ] Loading file: {}", path);

    
    let f: Vec<u8> = if crate::vfs::hm(path).is_ok() {
        
        crate::vfs::mq(path).jd(|_| "Failed to read file from VFS")?
    } else {
        
        crate::ramfs::fh(|fs| {
            fs.mq(path).map(|r| r.ip())
        }).jd(|_| "Failed to read file from VFS or ramfs")?
    };

    if f.is_empty() {
        return Err("File is empty");
    }

    let format = hfz(&f);
    crate::serial_println!("[VIZ] Detected format: {}, size: {} bytes", format, f.len());

    let audio = match format {
        "wav" => hfq(&f)?,
        "mp3" => return Err("MP3 not yet supported — convert to WAV first (ffmpeg -i song.mp3 song.wav)"),
        _ => return Err("Unknown audio format — only WAV (16-bit PCM) supported"),
    };

    if audio.is_empty() {
        return Err("Decoded audio is empty");
    }

    let fhe = audio.len() as f64 / (48000.0 * 2.0);
    crate::serial_println!("[VIZ] Decoded: {} samples, {:.1}s", audio.len(), fhe);

    
    let dq = path.cmm('/').next().unwrap_or(path);
    jcy(&audio, dq)
}






#[cfg(feature = "daw")]
pub static XY_: &[u8] = include_bytes!("untitled2.wav");
#[cfg(not(feature = "daw"))]
pub static XY_: &[u8] = &[];


pub const CZP_: &[&str] = &[
    "/music/untitled2.wav",
    "/mnt/fat32/music/untitled2.wav",
    "/mnt/sda1/music/untitled2.wav",
    "/home/music/untitled2.wav",
];


pub fn luh() -> Result<(), &'static str> {
    
    for path in CZP_ {
        if crate::vfs::hm(path).is_ok() {
            crate::serial_println!("[VIZ] Found '{}' on VFS, loading...", path);
            if let Ok(f) = crate::vfs::mq(path) {
                if !f.is_empty() {
                    crate::serial_println!("[VIZ] Loaded {} bytes from VFS", f.len());
                    let audio = hfq(&f)?;
                    let fhe = audio.len() as f64 / (48000.0 * 2.0);
                    crate::serial_println!("[VIZ] Decoded: {} samples, {:.1}s", audio.len(), fhe);
                    return jcy(&audio, "Untitled (2)");
                }
            }
        }
    }

    
    if let Ok(f) = crate::trustdaw::disk_audio::uhr() {
        crate::serial_println!("[VIZ] Loaded {} bytes from data disk", f.len());
        let audio = hfq(&f)?;
        let fhe = audio.len() as f64 / (48000.0 * 2.0);
        crate::serial_println!("[VIZ] Decoded: {} samples, {:.1}s", audio.len(), fhe);
        return jcy(&audio, "Untitled (2)");
    }

    
    if !XY_.is_empty() {
        crate::serial_println!("[VIZ] Using embedded WAV ({} bytes)", XY_.len());
        let audio = hfq(XY_)?;
        let fhe = audio.len() as f64 / (48000.0 * 2.0);
        crate::serial_println!("[VIZ] Decoded: {} samples, {:.1}s", audio.len(), fhe);
        return jcy(&audio, "Untitled (2)");
    }

    Err("Audio not found — place untitled2.wav in /music/ or build with --features daw")
}
