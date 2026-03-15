















extern crate alloc;

pub mod trace_bus;
pub mod hardware;
pub mod guide;
pub mod filetree;
pub mod editor;
pub mod kernel_trace;
pub mod pipeline;
pub mod hex_editor;
pub mod demo;
pub mod ux_test;
pub mod vm_inspector;
pub mod network_panel;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use crate::keyboard::{V_, U_, AH_, AI_, AM_, AQ_};


const J_: u32 = 28;
const DYQ_: u32 = 1;
const GX_: u32 = 6;
const KW_: u32 = 22;
const PM_: u32 = 28;


const MH_: u32        = 0xFF0D1117;  
const AAH_: u32  = 0xFF161B22;  
const AAI_: u32 = 0xFF30363D; 
const SE_: u32 = 0xFF1C2128;  
const T_: u32       = 0xFFE6EDF3;  
const F_: u32        = 0xFF8B949E;  
const O_: u32     = 0xFF58A6FF;  
const AK_: u32      = 0xFF3FB950;  
const AO_: u32     = 0xFFD29922;  
const AW_: u32        = 0xFFF85149;  
const BO_: u32     = 0xFFBC8CFF;  
const BB_: u32       = 0xFF79C0FF;  
const EZ_: u32     = 0xFFD18616;  
const BOB_: u32   = 0xFF0D1117;  
const BOC_: u32 = 0xFF3FB950;
const BNZ_: u32   = 0xFF1F6FEB;  


pub static EK_: AtomicBool = AtomicBool::new(false);


static DSI_: AtomicU64 = AtomicU64::new(0);


#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PanelId {
    Iq = 0,
    Gk = 1,
    Hm = 2,
    Hp = 3,
    Gp = 4,
    Iz = 5,
    Hr = 6,
    Mn = 7,
    Ly = 8,
}

impl PanelId {
    pub(crate) fn ivy(a: usize) -> Self {
        match a {
            0 => PanelId::Iq,
            1 => PanelId::Gk,
            2 => PanelId::Hm,
            3 => PanelId::Hp,
            4 => PanelId::Gp,
            5 => PanelId::Iz,
            6 => PanelId::Hr,
            7 => PanelId::Mn,
            _ => PanelId::Ly,
        }
    }
    
    fn dq(&self) -> &'static str {
        match self {
            PanelId::Iq => "⚙ Hardware Status",
            PanelId::Gk => "◈ Live Kernel Trace",
            PanelId::Hm => "📖 Command Guide",
            PanelId::Hp => "📁 File System Tree",
            PanelId::Gp => "⌨ TrustLang Editor",
            PanelId::Iz => "⚙ Pipeline View",
            PanelId::Hr => "🔍 Hex Editor",
            PanelId::Mn => "🖥 VM Inspector",
            PanelId::Ly => "🌐 Network",
        }
    }
    
    fn xd(&self) -> u32 {
        match self {
            PanelId::Iq => AK_,
            PanelId::Gk => EZ_,
            PanelId::Hm => O_,
            PanelId::Hp => BB_,
            PanelId::Gp => BO_,
            PanelId::Iz => AO_,
            PanelId::Hr => AW_,
            PanelId::Mn => 0xFFFF6B6B,
            PanelId::Ly => 0xFF00CED1,
        }
    }

    
    pub fn xx() -> [PanelId; 9] {
        [
            PanelId::Iq, PanelId::Gk, PanelId::Hm,
            PanelId::Hp, PanelId::Gp, PanelId::Iz,
            PanelId::Hr, PanelId::Mn, PanelId::Ly,
        ]
    }

    
    pub fn dbz(&self) -> &'static str {
        match self {
            PanelId::Iq => "Hardware",
            PanelId::Gk => "Kernel Trace",
            PanelId::Hm => "Cmd Guide",
            PanelId::Hp => "File Tree",
            PanelId::Gp => "TrustLang",
            PanelId::Iz => "Pipeline",
            PanelId::Hr => "Hex Editor",
            PanelId::Mn => "VM Inspector",
            PanelId::Ly => "Network",
        }
    }

    
    pub fn gb(&self) -> &'static str {
        match self {
            PanelId::Mn => "Hypervisor",
            PanelId::Ly => "Network",
            _ => "Core",
        }
    }
}




pub struct ModuleSwitcher {
    
    pub aji: bool,
    
    pub icz: usize,
    
    pub na: usize,
}

impl ModuleSwitcher {
    pub fn new() -> Self {
        Self { aji: false, icz: 0, na: 0 }
    }
}




pub const AYX_: [PanelId; 7] = [
    PanelId::Iq, PanelId::Gk, PanelId::Hm,
    PanelId::Hp, PanelId::Gp, PanelId::Iz, PanelId::Hr,
];


const CEE_: [PanelId; 7] = [
    PanelId::Gk, PanelId::Gp, PanelId::Hm,
    PanelId::Hp, PanelId::Hr, PanelId::Iz, PanelId::Iq,
];


const CEF_: [PanelId; 7] = [
    PanelId::Iq, PanelId::Gk, PanelId::Iz,
    PanelId::Hr, PanelId::Hp, PanelId::Gk, PanelId::Hm,
];


const AHY_: [&str; 7] = [
    "Top-Left", "Mid-Top", "Top-Right", "Bot-Left", "Mid-Bot", "Mid-Embed", "Bot-Right",
];


pub struct LabState {
    
    pub ckl: usize,
    
    pub cui: [PanelId; 7],
    
    pub ayv: ModuleSwitcher,
    
    pub bfh: String,
    
    pub dvx: usize,
    
    pub cri: hardware::HardwareState,
    pub ccg: kernel_trace::KernelTraceState,
    pub crb: guide::GuideState,
    pub faj: filetree::FileTreeState,
    pub aey: editor::EditorState,
    pub egs: pipeline::PipelineState,
    pub hmw: hex_editor::HexEditorState,
    pub igp: vm_inspector::VmInspectorState,
    pub jgq: network_panel::NetworkPanelState,
    pub ear: demo::DemoState,
    
    pub frame: u64,
    
    pub dyp: bool,
}

impl LabState {
    pub fn new() -> Self {
        EK_.store(true, Ordering::SeqCst);
        Self {
            ckl: 0,
            cui: AYX_,
            ayv: ModuleSwitcher::new(),
            bfh: String::new(),
            dvx: 0,
            cri: hardware::HardwareState::new(),
            ccg: kernel_trace::KernelTraceState::new(),
            crb: guide::GuideState::new(),
            faj: filetree::FileTreeState::new(),
            aey: editor::EditorState::new(),
            egs: pipeline::PipelineState::new(),
            hmw: hex_editor::HexEditorState::new(),
            igp: vm_inspector::VmInspectorState::new(),
            jgq: network_panel::NetworkPanelState::new(),
            ear: demo::DemoState::new(),
            frame: 0,
            dyp: true,
        }
    }
    
    
    pub fn vr(&mut self, bs: u8) {
        
        if self.ayv.aji {
            self.tlc(bs);
            return;
        }

        
        if self.ear.gh {
            self.ear.vr(bs);
            return;
        }

        
        if bs == 0x09 {
            self.ckl = (self.ckl + 1) % 7;
            return;
        }
        
        
        
        if bs == 0x0D || bs == 0x0A {
            if self.eqp() == PanelId::Gp {
                self.aey.vr(bs);
                return;
            }
            if self.eqp() == PanelId::Hp {
                self.faj.vr(bs);
                return;
            }
            if !self.bfh.is_empty() {
                self.nrq();
                return;
            }
        }
        
        
        if bs == 0x08 {
            
            if self.eqp() == PanelId::Gp {
                self.aey.vr(bs);
                return;
            }
            
            if self.dvx > 0 {
                self.dvx -= 1;
                self.bfh.remove(self.dvx);
            }
            return;
        }
        
        
        self.ryj(bs);
    }

    
    fn ryj(&mut self, bs: u8) {
        match self.eqp() {
            PanelId::Iq => self.cri.vr(bs),
            PanelId::Gk => self.ccg.vr(bs),
            PanelId::Hm => self.crb.vr(bs),
            PanelId::Hp => self.faj.vr(bs),
            PanelId::Gp => self.aey.vr(bs),
            PanelId::Iz => self.egs.vr(bs),
            PanelId::Hr => self.hmw.vr(bs),
            PanelId::Mn => self.igp.vr(bs),
            PanelId::Ly => self.jgq.vr(bs),
        }
    }
    
    
    pub fn fka(&mut self, bm: char) {
        if self.ayv.aji { return; }

        if self.ear.gh {
            
            if bm == ' ' {
                self.ear.vr(0x20);
            }
            return;
        }

        match self.eqp() {
            PanelId::Gp => self.aey.fka(bm),
            PanelId::Hm => self.crb.fka(bm),
            _ => {
                
                self.bfh.insert(self.dvx, bm);
                self.dvx += 1;
            }
        }
    }
    
    
    pub(crate) fn nrq(&mut self) {
        let js: String = self.bfh.em().bw().collect();
        let cmd: String = js.bw().map(|r| r.avd()).collect();
        self.bfh.clear();
        self.dvx = 0;
        
        match cmd.as_str() {
            "hw" | "hardware" | "cpu" => {
                self.cqr(PanelId::Iq);
            }
            "trace" | "log" | "events" => {
                self.cqr(PanelId::Gk);
            }
            "help" | "guide" | "commands" | "cmd" => {
                self.cqr(PanelId::Hm);
            }
            "fs" | "files" | "tree" | "ls" => {
                self.cqr(PanelId::Hp);
                self.faj.no = true;
                self.faj.vr(b'R'); 
            }
            "edit" | "editor" | "code" | "trustlang" => {
                self.cqr(PanelId::Gp);
            }
            "live" | "stream" | "bus" | "pipeline" | "pipe" => {
                self.cqr(PanelId::Iz);
            }
            "hex" | "hexedit" | "hexdump" => {
                self.cqr(PanelId::Hr);
            }
            "vm" | "vmi" | "inspector" | "hypervisor" => {
                self.cqr(PanelId::Mn);
            }
            "net" | "network" | "tcp" | "packets" => {
                self.cqr(PanelId::Ly);
            }
            _ if cmd.cj("hex ") => {
                let path = js[4..].em();
                if !path.is_empty() {
                    self.hmw.dsu(path);
                    self.cqr(PanelId::Hr);
                }
            }
            "clear" | "cls" => {
                self.ccg.events.clear();
                self.egs.cqq.clear();
            }
            "demo" | "showcase" | "present" => {
                self.ear.ay();
            }
            "refresh" | "r" => {
                self.faj.vr(b'R');
                self.cri.nvo();
            }
            "run" | "f5" => {
                self.aey.pep();
                self.cqr(PanelId::Gp);
            }
            "test" | "labtest" | "uxtest" => {
                ux_test::wbn(self);
            }
            
            "swap" | "module" | "switch" => {
                self.oss(self.ckl);
            }
            
            _ if cmd.cj("layout ") => {
                let akl = cmd[7..].em();
                match akl {
                    "default" | "reset" => {
                        self.cui = AYX_;
                        trace_bus::dgy(trace_bus::EventCategory::Gv, "Layout: default", 0);
                    }
                    "dev" | "developer" => {
                        self.cui = CEE_;
                        trace_bus::dgy(trace_bus::EventCategory::Gv, "Layout: developer", 0);
                    }
                    "monitor" | "mon" => {
                        self.cui = CEF_;
                        trace_bus::dgy(trace_bus::EventCategory::Gv, "Layout: monitor", 0);
                    }
                    _ => {
                        trace_bus::dgy(trace_bus::EventCategory::Gv, "Unknown layout (try: default, dev, monitor)", 0);
                    }
                }
            }
            "slots" | "layout" => {
                
                for (a, apz) in self.cui.iter().cf() {
                    let fr = format!("Slot {} [{}]: {}", a, AHY_[a], apz.dbz());
                    trace_bus::fj(trace_bus::EventCategory::Gv, fr, a as u64);
                }
                self.cqr(PanelId::Gk);
            }
            _ => {
                
                trace_bus::dgy(
                    trace_bus::EventCategory::Gv,
                    "lab> unknown command",
                    0,
                );
            }
        }
    }
    
    
    pub fn ago(&mut self, kb: i32, ix: i32, hk: u32, mg: u32) {
        
        if self.ayv.aji {
            self.tlb(kb, ix, hk, mg);
            return;
        }

        let cx = 2i32;
        let ae = J_ as i32 + 2;
        let dt = hk.ao(4);
        let bm = mg.ao(J_ + 4);
        if dt < 200 || bm < 100 { return; }

        let cls = ioy(cx, ae, dt, bm);

        
        for (a, oc) in cls.iter().cf() {
            if kb >= oc.b && kb < oc.b + oc.d as i32
                && ix >= oc.c && ix < oc.c + oc.i as i32
            {
                self.ckl = a;
                let ce = self.cui[a];

                
                let wwg = oc.b + oc.d as i32 - 24;
                if ix < oc.c + KW_ as i32 && kb >= wwg {
                    self.oss(a);
                    return;
                }

                
                let tc = oc.b + GX_ as i32;
                let gl = oc.c + KW_ as i32 + GX_ as i32;
                let ur = oc.d.ao(GX_ * 2);
                let nd = oc.i.ao(KW_ + GX_ * 2);
                let bhi = kb - tc;
                let alk = ix - gl;

                
                self.ryf(ce, bhi, alk, ur, nd);
                return;
            }
        }

        
        let qi = 4u32;
        let jpv = ae + (bm - PM_) as i32;
        if ix >= jpv && ix < jpv + PM_ as i32 {
            let dpm = nk();
            if dpm > 0 {
                let lvx = 5; 
                let cky = cx + 8 + lvx * dpm;
                let rbm = ((kb - cky) / dpm).am(0) as usize;
                self.dvx = rbm.v(self.bfh.len());
            }
        }
    }

    
    fn ryf(&mut self, ce: PanelId, mj: i32, ct: i32, d: u32, i: u32) {
        match ce {
            PanelId::Iq => self.cri.ago(mj, ct, d, i),
            PanelId::Gk => self.ccg.ago(mj, ct, d, i),
            PanelId::Hm => self.crb.ago(mj, ct, d, i),
            PanelId::Hp => self.faj.ago(mj, ct, d, i),
            PanelId::Gp => self.aey.ago(mj, ct, d, i),
            PanelId::Iz => self.egs.ago(mj, ct, d, i),
            PanelId::Hr => self.hmw.ago(mj, ct, d, i),
            PanelId::Mn => self.igp.ago(mj, ct, d, i),
            PanelId::Ly => self.jgq.ago(mj, ct, d, i),
        }
    }

    
    pub fn or(&mut self) {
        self.frame += 1;
        self.cri.qs();
        self.ccg.qs();
        self.egs.qs();
        self.igp.qs();
        
        if let Some(vbi) = self.ear.or() {
            
            self.ckl = vbi.v(6);
        }
    }

    

    
    pub fn eqp(&self) -> PanelId {
        self.cui[self.ckl]
    }

    
    pub fn wps(&self, apz: PanelId) -> Option<usize> {
        self.cui.iter().qf(|ef| *ef == apz)
    }

    
    pub fn cqr(&mut self, apz: PanelId) {
        if let Some(gk) = self.wps(apz) {
            self.ckl = gk;
        }
    }

    
    pub fn oss(&mut self, gk: usize) {
        self.ayv.aji = true;
        self.ayv.icz = gk;
        let cv = self.cui[gk];
        self.ayv.na = PanelId::xx().iter().qf(|ai| *ai == cv).unwrap_or(0);
    }

    
    fn tlc(&mut self, bs: u8) {
        match bs {
            0x1B => {
                
                self.ayv.aji = false;
            }
            0x0D | 0x0A => {
                
                let xx = PanelId::xx();
                if self.ayv.na < xx.len() {
                    let apz = xx[self.ayv.na];
                    let gk = self.ayv.icz;
                    self.cui[gk] = apz;
                    let fr = format!("Slot {} [{}] -> {}",
                        gk, AHY_[gk], apz.dbz());
                    trace_bus::fj(trace_bus::EventCategory::Gv, fr, 0);
                }
                self.ayv.aji = false;
            }
            eh if eh == V_ => {
                if self.ayv.na > 0 {
                    self.ayv.na -= 1;
                }
            }
            eh if eh == U_ => {
                let am = PanelId::xx().len() - 1;
                if self.ayv.na < am {
                    self.ayv.na += 1;
                }
            }
            _ => {}
        }
    }

    
    fn tlb(&mut self, kb: i32, ix: i32, hk: u32, mg: u32) {
        let cx = 2i32;
        let ae = J_ as i32 + 2;
        let dt = hk.ao(4);
        let bm = mg.ao(J_ + 4);
        if dt < 200 || bm < 100 { self.ayv.aji = false; return; }

        let cls = ioy(cx, ae, dt, bm);
        let gk = self.ayv.icz;
        if gk >= cls.len() { self.ayv.aji = false; return; }
        let oc = &cls[gk];

        
        let ov = 4i32;
        let mp = oc.b + ov;
        let qw = oc.c + ov;
        let fqa = oc.d.ao(8);
        let goh = oc.i.ao(8);

        if kb >= mp && kb < mp + fqa as i32 && ix >= qw && ix < qw + goh as i32 {
            
            let hqd = qw + 24; 
            let ph = apm();
            if ph > 0 && ix >= hqd {
                let bt = ((ix - hqd) / ph) as usize;
                if bt < PanelId::xx().len() {
                    self.ayv.na = bt;
                    
                    
                    let apz = PanelId::xx()[bt];
                    self.cui[gk] = apz;
                    let fr = format!("Slot {} [{}] -> {}",
                        gk, AHY_[gk], apz.dbz());
                    trace_bus::fj(trace_bus::EventCategory::Gv, fr, 0);
                    self.ayv.aji = false;
                }
            }
        } else {
            
            self.ayv.aji = false;
        }
    }
}

impl Drop for LabState {
    fn drop(&mut self) {
        
        EK_.store(false, Ordering::SeqCst);
    }
}




pub(crate) struct Mc {
    pub(crate) b: i32,
    pub(crate) c: i32,
    pub(crate) d: u32,
    pub(crate) i: u32,
}

pub(crate) fn ioy(cx: i32, ae: i32, dt: u32, bm: u32) -> [Mc; 7] {
    let qi = 4u32;
    
    let nd = bm.ao(PM_ + qi);
    
    let oy = (dt.ao(qi * 2)) / 3;
    let ph = (nd.ao(qi)) / 2;
    
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
        Mc { b: fy, c: fo, d: oy, i: ph },          
        Mc { b: dn, c: fo, d: oy, i: fxf },        
        Mc { b: hy, c: fo, d: hdn, i: ph },         
        Mc { b: fy, c: dp, d: oy, i: ph },          
        Mc { b: dn, c: dp, d: oy, i: ph },          
        Mc { b: dn, c: lua, d: oy, i: ltz },     
        Mc { b: hy, c: dp, d: hdn, i: ph },         
    ]
}


pub fn sds(g: &LabState, fx: i32, lw: i32, hk: u32, mg: u32) {
    let cx = fx + 2;
    let ae = lw + J_ as i32 + 2;
    let dt = hk.ao(4);
    let bm = mg.ao(J_ + 4);
    
    if dt < 200 || bm < 100 {
        return;
    }
    
    
    crate::framebuffer::ah(cx as u32, ae as u32, dt, bm, MH_);
    
    
    let cls = ioy(cx, ae, dt, bm);
    
    
    for (a, oc) in cls.iter().cf() {
        let ce = g.cui[a];
        let ja = a == g.ckl;
        epf(oc, ce, ja);
        
        
        let tc = oc.b + GX_ as i32;
        let gl = oc.c + KW_ as i32 + GX_ as i32;
        let ur = oc.d.ao(GX_ * 2);
        let nd = oc.i.ao(KW_ + GX_ * 2);
        
        seb(g, ce, tc, gl, ur, nd);
    }
    
    
    let qi = 4u32;
    let jpv = ae + (bm - PM_) as i32;
    sfm(g, cx, jpv, dt, PM_);

    
    if g.ayv.aji {
        sec(g, &cls);
    }

    
    if g.ear.gh {
        demo::seo(&g.ear, fx, lw, hk, mg);
    }
}


fn epf(oc: &Mc, ce: PanelId, ja: bool) {
    
    crate::framebuffer::ah(oc.b as u32, oc.c as u32, oc.d, oc.i, AAH_);
    
    
    let aia = if ja { BNZ_ } else { AAI_ };
    nno(oc.b, oc.c, oc.d, oc.i, aia);
    
    
    let tnt = if ja { 0xFF1F2937 } else { SE_ };
    crate::framebuffer::ah(
        (oc.b + 1) as u32, (oc.c + 1) as u32,
        oc.d.ao(2), KW_ - 1,
        tnt,
    );
    
    
    crate::framebuffer::ah(
        (oc.b + 1) as u32, (oc.c + 1) as u32,
        oc.d.ao(2), 2,
        ce.xd(),
    );
    
    
    let dq = ce.dq();
    kw(oc.b + 8, oc.c + 6, dq, T_);
    
    
    let axp = oc.b + oc.d as i32 - 22;
    let hbk = if ja { O_ } else { F_ };
    kw(axp, oc.c + 6, "\u{25BC}", hbk);
}


fn seb(g: &LabState, ce: PanelId, b: i32, c: i32, d: u32, i: u32) {
    match ce {
        PanelId::Iq => hardware::po(&g.cri, b, c, d, i),
        PanelId::Gk => kernel_trace::po(&g.ccg, b, c, d, i),
        PanelId::Hm => guide::po(&g.crb, b, c, d, i),
        PanelId::Hp => filetree::po(&g.faj, b, c, d, i),
        PanelId::Gp => editor::po(&g.aey, b, c, d, i),
        PanelId::Iz => pipeline::po(&g.egs, b, c, d, i),
        PanelId::Hr => hex_editor::po(&g.hmw, b, c, d, i),
        PanelId::Mn => vm_inspector::po(&g.igp, b, c, d, i),
        PanelId::Ly => network_panel::po(&g.jgq, b, c, d, i),
    }
}


fn sec(g: &LabState, cls: &[Mc; 7]) {
    let gk = g.ayv.icz;
    if gk >= cls.len() { return; }
    let oc = &cls[gk];

    
    let ov = 2;
    let mp = oc.b + ov;
    let qw = oc.c + ov;
    let fqa = oc.d.ao(4);
    let goh = oc.i.ao(4);
    crate::framebuffer::ah(mp as u32, qw as u32, fqa, goh, 0xFF0D1117);
    nno(mp, qw, fqa, goh, O_);

    
    let dq = format!("Select Module (Slot {})", gk);
    kw(mp + 8, qw + 4, &dq, O_);

    
    let pib = qw + 22;
    crate::framebuffer::ah((mp + 1) as u32, pib as u32, fqa.ao(2), 1, AAI_);

    
    let bm = apm();
    let xx = PanelId::xx();
    let cv = g.cui[gk];
    let mut afy = pib + 4;

    for (a, apz) in xx.iter().cf() {
        if afy + bm > qw + goh as i32 - bm { break; } 

        let qe = a == g.ayv.na;
        let afb = *apz == cv;

        
        if qe {
            crate::framebuffer::ah(
                (mp + 2) as u32, afy as u32,
                fqa.ao(4), bm as u32,
                0xFF1F6FEB,
            );
        }

        
        let pa = if afb { "*" } else { " " };
        let cif = if afb { " [active]" } else { "" };
        let cu = format!("{} {} {}{}", pa, apz.dbz(), apz.gb(), cif);
        let s = if qe { T_ } else if afb { AK_ } else { F_ };
        kw(mp + 8, afy + 2, &cu, s);

        
        crate::framebuffer::ah(
            (mp + fqa as i32 - 16) as u32, (afy + 4) as u32,
            8, 8, apz.xd(),
        );

        afy += bm;
    }

    
    let iyj = qw + goh as i32 - bm;
    kw(mp + 8, iyj, "Up/Down Enter Esc", F_);
}


fn sfm(g: &LabState, b: i32, c: i32, d: u32, i: u32) {
    crate::framebuffer::ah(b as u32, c as u32, d, i, BOB_);
    
    crate::framebuffer::ah(b as u32, c as u32, d, 1, AAI_);
    
    
    let aau = "lab> ";
    kw(b + 8, c + 7, aau, BOC_);
    
    
    let cky = b + 8 + (aau.len() as i32 * nk());
    if g.bfh.is_empty() {
        kw(cky, c + 7, "hw|trace|fs|edit|hex|vm|net|swap|layout|run|test", F_);
    } else {
        kw(cky, c + 7, &g.bfh, T_);
    }
    
    
    if (g.frame / 30) % 2 == 0 {
        let lf = cky + (g.dvx as i32 * nk());
        crate::framebuffer::ah(lf as u32, (c + 6) as u32, 2, 14, O_);
    }
    
    
    let vbj = g.eqp().dq();
    let hint = format!("[Tab] cycle | swap | layout | {}", vbj);
    let hmy = b + d as i32 - (hint.len() as i32 * nk()) - 8;
    kw(hmy, c + 7, &hint, F_);
}




pub fn kw(b: i32, c: i32, text: &str, s: u32) {
    crate::graphics::scaling::kri(b, c, text, s);
}


fn nk() -> i32 {
    crate::graphics::scaling::bmi() as i32
}


fn apm() -> i32 {
    16 * crate::graphics::scaling::ckv() as i32
}


fn nno(b: i32, c: i32, d: u32, i: u32, s: u32) {
    if d == 0 || i == 0 { return; }
    
    crate::framebuffer::ah(b as u32, c as u32, d, 1, s);
    
    crate::framebuffer::ah(b as u32, (c + i as i32 - 1) as u32, d, 1, s);
    
    crate::framebuffer::ah(b as u32, c as u32, 1, i, s);
    
    crate::framebuffer::ah((b + d as i32 - 1) as u32, c as u32, 1, i, s);
}


pub fn gfh(b: i32, c: i32, d: u32, i: u32, cgn: u32, lp: u32, ei: u32) {
    crate::framebuffer::ah(b as u32, c as u32, d, i, ei);
    let akd = (d as u64 * cgn.v(100) as u64 / 100) as u32;
    if akd > 0 {
        crate::framebuffer::ah(b as u32, c as u32, akd, i, lp);
    }
}


pub fn ztq(e: &str, hrh: u32) -> &str {
    let dt = nk() as u32;
    if dt == 0 { return e; }
    let aem = (hrh / dt) as usize;
    if e.len() <= aem {
        e
    } else if aem > 3 {
        &e[..aem - 3]
    } else {
        &e[..aem.v(e.len())]
    }
}
