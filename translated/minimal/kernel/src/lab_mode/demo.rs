









extern crate alloc;

use alloc::string::String;
use alloc::format;




#[inline]
fn efu() -> u64 {
    crate::time::lc()
}



#[derive(Clone, Copy)]
enum Pos {
    Eo,
    Dz(usize),   
    Agq,    
}



struct Dn {
    ak: &'static [&'static str],
    apv: &'static [&'static str],
    u: Pos,
    arc: Option<usize>,
    
    aie: u64,
    
    bba: bool,
}




const LI_: u32 = 28;
const CSR_: u32 = 28;

const ACT_: u64 = 350; 

const Jc: &[Dn] = &[
    
    Dn { ak: &[""],
        apv: &[], u: Pos::Agq, arc: None,
        aie: 800, bba: true },
    Dn { ak: &["Are you ready",  "to see the Matrix, Neo?"],
        apv: &["Matrix", "Neo"], u: Pos::Agq, arc: None,
        aie: 2200, bba: true },

    
    Dn { ak: &["You don't understand", "how your computer works."],
        apv: &["don't", "computer"], u: Pos::Eo, arc: None,
        aie: 2000, bba: true },
    Dn { ak: &["This is TrustLab."],
        apv: &["TrustLab"], u: Pos::Eo, arc: None,
        aie: 1500, bba: true },

    
    Dn { ak: &["HARDWARE STATUS"],
        apv: &["HARDWARE"], u: Pos::Dz(0), arc: Some(0),
        aie: 500, bba: true },
    Dn { ak: &["Real CPU. Real memory.", "Raw silicon."],
        apv: &["CPU", "memory", "Raw"], u: Pos::Dz(0), arc: Some(0),
        aie: 1500, bba: true },
    Dn { ak: &["What Task Manager", "will never show you."],
        apv: &["never"], u: Pos::Dz(0), arc: Some(0),
        aie: 1200, bba: true },

    
    Dn { ak: &["LIVE KERNEL TRACE"],
        apv: &["KERNEL", "TRACE"], u: Pos::Dz(1), arc: Some(1),
        aie: 500, bba: true },
    Dn { ak: &["Every interrupt.", "Every syscall."],
        apv: &["interrupt", "syscall"], u: Pos::Dz(1), arc: Some(1),
        aie: 1500, bba: true },
    Dn { ak: &["Raw kernel truth."],
        apv: &["Raw", "truth"], u: Pos::Dz(1), arc: Some(1),
        aie: 1200, bba: true },

    
    Dn { ak: &["EXECUTION PIPELINE"],
        apv: &["PIPELINE"], u: Pos::Dz(5), arc: Some(5),
        aie: 500, bba: true },
    Dn { ak: &["Watch data flow through", "the kernel in real time."],
        apv: &["flow", "real time"], u: Pos::Dz(5), arc: Some(5),
        aie: 1500, bba: true },

    
    Dn { ak: &["HEX EDITOR"],
        apv: &["HEX"], u: Pos::Dz(6), arc: Some(6),
        aie: 500, bba: true },
    Dn { ak: &["Raw bytes. Color-coded."],
        apv: &["Raw", "bytes"], u: Pos::Dz(6), arc: Some(6),
        aie: 1300, bba: true },

    
    Dn { ak: &["FILE SYSTEM"],
        apv: &["FILE"], u: Pos::Dz(3), arc: Some(3),
        aie: 500, bba: true },
    Dn { ak: &["Live filesystem. In memory."],
        apv: &["Live", "memory"], u: Pos::Dz(3), arc: Some(3),
        aie: 1200, bba: true },

    
    Dn { ak: &["TRUSTLANG EDITOR"],
        apv: &["TRUSTLANG"], u: Pos::Dz(4), arc: Some(4),
        aie: 500, bba: true },
    Dn { ak: &["Write code inside the kernel.", "Execute it."],
        apv: &["code", "kernel", "Execute"], u: Pos::Dz(4), arc: Some(4),
        aie: 1500, bba: true },

    
    Dn { ak: &["52 COMMANDS"],
        apv: &["52"], u: Pos::Dz(2), arc: Some(2),
        aie: 500, bba: true },
    Dn { ak: &["Full shell. All built-in."],
        apv: &["shell", "built-in"], u: Pos::Dz(2), arc: Some(2),
        aie: 1200, bba: true },

    
    Dn { ak: &["TrustLab is not a tool."],
        apv: &["not"], u: Pos::Eo, arc: None,
        aie: 1500, bba: true },
    Dn { ak: &["Bare metal. Rust. Open source."],
        apv: &["Rust", "Open source"], u: Pos::Eo, arc: None,
        aie: 1500, bba: true },
    Dn { ak: &["Boot it. Break it.", "Understand it."],
        apv: &["Boot", "Break", "Understand"], u: Pos::Eo, arc: None,
        aie: 2000, bba: true },
];



pub struct DemoState {
    pub gh: bool,
    pub bzb: usize,
    
    iav: u64,
    
    fgn: u64,
    
    dv: u32,
    
    jcs: usize,
    
    pub idl: u64,
    pub cng: u64,
}

impl DemoState {
    pub fn new() -> Self {
        Self {
            gh: false,
            bzb: 0,
            iav: 0,
            fgn: 0,
            dv: 12345,
            jcs: usize::O,
            idl: 0,
            cng: 0,
        }
    }

    pub fn ay(&mut self) {
        self.gh = true;
        self.bzb = 0;
        self.fgn = efu();
        self.iav = self.fgn;
        self.dv = (self.fgn & 0xFFFF) as u32 ^ 0xA5A5;
        self.jcs = usize::O;
        self.idl = 0;
        self.cng = 0;
        let alu: u64 = Jc.iter().map(|e| e.aie).sum();
        crate::serial_println!("[DEMO] Started! now_ms={} total_script={}ms slides={}",
            self.fgn, alu, Jc.len());
    }

    pub fn qg(&mut self) {
        self.gh = false;
    }

    
    pub fn or(&mut self) -> Option<usize> {
        if !self.gh { return None; }
        self.cng += 1;

        let ab = efu();
        let ieg = ab - self.fgn;

        if self.bzb >= Jc.len() {
            self.qg();
            return None;
        }

        
        let mut ipu: u64 = 0;
        for a in 0..=self.bzb {
            if a < Jc.len() {
                ipu += Jc[a].aie;
            }
        }

        
        if self.cng % 200 == 0 {
            crate::serial_println!("[DEMO] slide={} total={}ms deadline={}ms ticks={}",
                self.bzb, ieg, ipu, self.cng);
        }

        if ieg >= ipu {
            
            crate::serial_println!("[DEMO] -> next slide {} at total={}ms (deadline={}ms)",
                self.bzb + 1, ieg, ipu);
            self.bzb += 1;
            self.iav = ab;
            self.idl = 0;
            if self.bzb >= Jc.len() {
                self.qg();
                return None;
            }
            return Jc[self.bzb].arc;
        }

        self.idl += 1;

        
        if self.jcs != self.bzb {
            self.jcs = self.bzb;
            return Jc[self.bzb].arc;
        }
        None
    }

    
    pub fn vr(&mut self, bs: u8) -> bool {
        if !self.gh { return false; }
        match bs {
            0x1B => { self.qg(); true }
            0x20 => {
                self.bzb += 1;
                self.iav = efu();
                self.idl = 0;
                if self.bzb >= Jc.len() { self.qg(); }
                true
            }
            _ => true
        }
    }

    
    fn wpn(&self) -> u64 {
        efu() - self.iav
    }

    
    fn ieg(&self) -> u64 {
        efu() - self.fgn
    }

    
    fn das(&self, ang: u32) -> u32 {
        let mut p = self.dv.cn(self.cng as u32).cn(ang);
        p ^= p << 13;
        p ^= p >> 17;
        p ^= p << 5;
        p
    }
}



struct Qj { b: i32, c: i32, d: u32, i: u32 }

fn vbl(fx: i32, lw: i32, hk: u32, mg: u32) -> [Qj; 7] {
    let cx = fx + 2;
    let ae = lw + LI_ as i32 + 2;
    let dt = hk.ao(4);
    let bm = mg.ao(LI_ + 4);
    let qi = 4u32;
    let nd = bm.ao(CSR_ + qi);
    let oy = dt.ao(qi * 2) / 3;
    let ph = nd.ao(qi) / 2;
    let fy = cx;
    let dn = cx + oy as i32 + qi as i32;
    let hy = cx + (oy as i32 + qi as i32) * 2;
    let fo = ae;
    let dp = ae + ph as i32 + qi as i32;
    let hdn = (dt as i32 - (hy - cx)).am(40) as u32;
    let fxf = ph.ao(qi) / 2;
    let ltz = ph.ao(fxf + qi);
    let lua = fo + fxf as i32 + qi as i32;

    [
        Qj { b: fy, c: fo, d: oy, i: ph },
        Qj { b: dn, c: fo, d: oy, i: fxf },
        Qj { b: hy, c: fo, d: hdn, i: ph },
        Qj { b: fy, c: dp, d: oy, i: ph },
        Qj { b: dn, c: dp, d: oy, i: ph },
        Qj { b: dn, c: lua, d: oy, i: ltz },
        Qj { b: hy, c: dp, d: hdn, i: ph },
    ]
}




fn ief() -> u64 {
    Jc.iter().map(|e| e.aie).sum()
}


fn zor(bo: usize) -> u64 {
    Jc[..bo].iter().map(|e| e.aie).sum()
}


pub fn seo(g: &DemoState, fx: i32, lw: i32, hk: u32, mg: u32) {
    if !g.gh { return; }
    if g.bzb >= Jc.len() { return; }

    let dwa = &Jc[g.bzb];
    let oz = g.wpn();
    let bv: u32 = 3; 

    let ing = 8i32 * bv as i32;
    let lir = 16i32 * bv as i32 + 8;

    
    let din = oh!(dwa.u, Pos::Agq);
    if din {
        crate::framebuffer::ah(fx.am(0) as u32, lw.am(0) as u32, hk, mg, 0xFF000000);
        sed(g, fx, lw, hk, mg);
    }

    
    if !din && oz < ACT_ {
        sdg(g, fx, lw, hk, mg, oz);
    }

    
    let ggs = 200u64;
    let dw = if oz < ggs {
        (oz * 255 / ggs).v(255) as u32
    } else if oz > dwa.aie.ao(ggs) {
        let rem = dwa.aie.ao(oz);
        (rem * 255 / ggs).v(255) as u32
    } else {
        255u32
    };
    if dw < 8 { return; }

    
    let tnj = dwa.ak.iter().any(|dm| !dm.is_empty());
    if !tnj { return; }

    
    let cat = dwa.ak.iter().map(|dm| dm.len()).am().unwrap_or(1);
    let keb = cat as i32 * ing;
    let kdy = dwa.ak.len() as i32 * lir;

    
    let cls = vbl(fx, lw, hk, mg);

    let (gx, ty) = match dwa.u {
        Pos::Eo | Pos::Agq => {
            (fx + (hk as i32 - keb) / 2,
             lw + (mg as i32 - kdy) / 2)
        }
        Pos::Dz(w) => {
            let ai = &cls[w.v(6)];
            let bx = ai.b + (ai.d as i32 - keb) / 2;
            let je = ai.c + (ai.i as i32 - kdy) / 2;
            (bx.am(fx + 4).v(fx + hk as i32 - keb - 4),
             je.am(lw + 32).v(lw + mg as i32 - kdy - 4))
        }
    };

    
    let mut ct = ty;
    for line in dwa.ak {
        if line.is_empty() { ct += lir; continue; }
        
        irv(gx + 2, ct + 2, line, dwa.apv, bv, dw * 2 / 3, true);
        
        irv(gx, ct, line, dwa.apv, bv, dw, false);
        ct += lir;
    }

    
    let alu = ief();
    let npl = g.ieg().v(alu);
    let vmz = (npl as u32 * hk / alu.am(1) as u32).v(hk);
    let ctm = (lw + mg as i32 - 3).am(0) as u32;
    crate::framebuffer::ah(fx.am(0) as u32, ctm, hk, 3, 0xFF1C2128);
    crate::framebuffer::ah(fx.am(0) as u32, ctm, vmz, 3, 0xFFFF2020);

    
    let tv = npl / 1000;
    let jtw = alu / 1000;
    let timer = format!("{}s/{}s", tv, jtw);
    let jsr = super::nk();
    let xhn = fx + hk as i32 - (timer.len() as i32 * jsr) - 8;
    super::kw(xhn, ctm as i32 - 16, &timer, eot(0xFF8B949E, dw));

    
    super::kw(fx + 8, ctm as i32 - 16, "[Esc] stop  [Space] next",
        eot(0xFF484F58, dw));
}




fn sdg(g: &DemoState, fx: i32, lw: i32, hk: u32, mg: u32, oz: u64) {
    let hj = ((ACT_ - oz) * 255 / ACT_).v(255) as u32;
    if hj < 10 { return; }

    
    let hdp = 12u32;
    let ajg = hk / hdp;
    let bw = b"01?#@!$%&*<>{}[]|/\\~";

    for r in 0..ajg {
        let dv = g.das(r * 7919 + 31);
        let ffl = fx + (r * hdp) as i32;
        let ig = (dv % 5 + 2) as i32;
        let l = (g.cng as i32 * ig + dv as i32) % mg as i32;

        
        let az = (dv % 4 + 3) as i32;
        for fb in 0..az {
            let ae = lw + (l + fb * 14) % mg as i32;
            let gck = ((dv >> (fb as u32 * 3)) as usize + g.cng as usize) % bw.len();
            let bm = bw[gck] as char;

            
            let kt = if fb == 0 { hj } else { hj * (az - fb) as u32 / az as u32 / 2 };
            let at = (kt * 255 / 255).v(255);
            let s = 0xFF000000 | ((at / 4) << 16) | (at << 8) | (at / 4);

            let mut k = [0u8; 1];
            k[0] = bm as u8;
            if let Ok(e) = core::str::jg(&k) {
                crate::graphics::scaling::azp(ffl, ae, e, s, 1);
            }
        }
    }

    
    let iko = (hj / 40).v(6);
    for o in 0..iko {
        let iks = g.das(o * 1337 + 42);
        let pl = lw + (iks % mg) as i32;
        let lo = (iks % (hk / 2)) + 20;
        let ajx = fx + (iks % (hk / 3)) as i32;
        let at = (iks % 100 + 50).v(200);
        let s = 0xFF000000 | ((at / 6) << 16) | (at << 8) | ((at / 4) & 0xFF);
        crate::framebuffer::ah(
            ajx.am(0) as u32, pl.am(0) as u32,
            lo.v(hk), 2, s,
        );
    }
}




fn sed(g: &DemoState, fx: i32, lw: i32, hk: u32, mg: u32) {
    let hdp = 10u32;
    let ajg = hk / hdp;
    let bw = b"01?#@$%&*<>{}[]|/\\~:;_=+-.";

    for r in 0..ajg {
        let dv = g.das(r * 6271 + 17);
        let ffl = fx + (r * hdp) as i32;
        let ig = (dv % 3 + 1) as i32;
        let l = (g.cng as i32 * ig + (dv as i32 * 37)) % (mg as i32 * 2);

        
        let az = (dv % 8 + 8) as i32;
        for fb in 0..az {
            let ae = lw + (l + fb * 12) % mg as i32;
            let gck = ((dv >> (fb as u32 * 2 + 1)) as usize + g.cng as usize) % bw.len();
            let bm = bw[gck];

            
            let kt = if fb == 0 { 255u32 }
                else { (200u32).ao(fb as u32 * 18) };
            let at = kt.v(255);
            let m = if fb == 0 { at / 2 } else { 0 };
            let o = if fb == 0 { at / 3 } else { 0 };
            let s = 0xFF000000 | (m << 16) | (at << 8) | o;

            let mut k = [0u8; 1];
            k[0] = bm;
            if let Ok(e) = core::str::jg(&k) {
                crate::graphics::scaling::azp(ffl, ae, e, s, 1);
            }
        }
    }
}





fn irv(b: i32, c: i32, line: &str, apv: &[&str],
                         bv: u32, dw: u32, zc: bool)
{
    let ing = 8i32 * bv as i32;
    let mfj = eot(0xFF000000, dw);
    let adg = eot(0xFFFF3030, dw);   
    let mm = eot(0xFFFF6060, dw);   
    let gio  = eot(0xFF00FF41, dw);   

    let mut cx = b;
    let mut elb = String::new();

    let mut bw = line.bw().ltk();
    while let Some(&bm) = bw.amm() {
        if bm.etb() || bm == '\'' {
            elb.push(bm);
            bw.next();
        } else {
            if !elb.is_empty() {
                let bj = if zc { mfj }
                          else { ovm(&elb, apv, adg, mm, gio) };
                crate::graphics::scaling::azp(cx, c, &elb, bj, bv);
                cx += elb.len() as i32 * ing;
                elb.clear();
            }
            let mut k = [0u8; 4];
            let e = bm.hia(&mut k);
            let bj = if zc { mfj } else { adg };
            crate::graphics::scaling::azp(cx, c, e, bj, bv);
            cx += ing;
            bw.next();
        }
    }
    if !elb.is_empty() {
        let bj = if zc { mfj }
                  else { ovm(&elb, apv, adg, mm, gio) };
        crate::graphics::scaling::azp(cx, c, &elb, bj, bv);
    }
}


fn ovm(od: &str, apv: &[&str], adg: u32, mm: u32, gio: u32) -> u32 {
    for abe in apv {
        if od.dha(abe) { return mm; }
    }
    match od {
        "kernel" | "Kernel" | "KERNEL" => gio,
        "TrustLab" | "TRUSTLAB" | "TrustOS" | "TRUSTOS" => mm,
        "Matrix" | "MATRIX" => gio,
        "Neo" | "NEO" => gio,
        "Rust" | "RUST" => 0xFFD18616,
        _ => adg,
    }
}


fn eot(s: u32, dw: u32) -> u32 {
    let q = dw.v(255);
    let m = ((s >> 16) & 0xFF) * q / 255;
    let at = ((s >> 8) & 0xFF) * q / 255;
    let o = (s & 0xFF) * q / 255;
    0xFF000000 | (m << 16) | (at << 8) | o
}
