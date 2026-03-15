







extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use super::trace_bus::{self, EventCategory, Jq};


const T_: u32    = 0xFFE6EDF3;
const F_: u32     = 0xFF8B949E;
const AK_: u32   = 0xFF3FB950;
const AW_: u32     = 0xFFF85149;
const AO_: u32  = 0xFFD29922;
const BB_: u32    = 0xFF79C0FF;
const BO_: u32  = 0xFFBC8CFF;
const EZ_: u32  = 0xFFD18616;
const HS_: u32      = 0xFFFF6B6B; 


pub struct VmInspectorState {
    
    pub events: Vec<Jq>,
    
    pub crz: u64,
    
    pub ahd: usize,
    
    pub jc: usize,
    
    pub frame: u64,
    
    pub dna: u32,
    pub dcw: u64,
    pub eel: String,
    pub hpr: u64,
    
    pub lyq: u64,
    pub lyr: u64,
    pub lys: u64,
    pub lyt: u64,
    pub lyu: u64,
    pub lyv: u64,
    
    pub bmp: u64,
    pub ank: u64,
    pub bkn: u64,
    pub axz: u64,
    pub cay: u64,
    pub mpq: u64,
    
    pub bnn: u64,
}

impl VmInspectorState {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            crz: 0,
            ahd: 0,
            jc: 0,
            frame: 0,
            dna: 0,
            dcw: 0,
            eel: String::from("(none)"),
            hpr: 0,
            lyq: 0,
            lyr: 0,
            lys: 0,
            lyt: 0,
            lyu: 0,
            lyv: 0,
            bmp: 0,
            ank: 0,
            bkn: 0,
            axz: 0,
            cay: 0,
            mpq: 0,
            bnn: 0x1000,
        }
    }

    
    pub fn qs(&mut self) {
        self.frame += 1;
        if self.frame % 10 != 0 { return; }
        
        let (jgs, gnn) = trace_bus::hxa(self.crz, 200);
        self.crz = gnn;
        
        for aiz in &jgs {
            if aiz.gb == EventCategory::Ee {
                
                self.vco(aiz);
                
                
                self.events.push(aiz.clone());
                if self.events.len() > 200 {
                    self.events.remove(0);
                }
            }
        }
    }

    
    fn vco(&mut self, aiz: &Jq) {
        let fr = &aiz.message;
        
        
        if fr.contains("EXIT: CPUID") {
            self.bmp += 1;
            self.dcw += 1;
            self.eel = String::from("CPUID");
        } else if fr.contains("IO ") {
            self.ank += 1;
            self.dcw += 1;
            self.eel = String::from("I/O");
        } else if fr.contains("RDMSR") || fr.contains("WRMSR") {
            self.bkn += 1;
            self.dcw += 1;
            self.eel = String::from("MSR");
        } else if fr.contains("EXIT: HLT") {
            self.axz += 1;
            self.dcw += 1;
            self.eel = String::from("HLT");
        } else if fr.contains("NPF_VIOLATION") || fr.contains("EPT_VIOLATION") {
            self.cay += 1;
            self.dcw += 1;
            self.eel = String::from("PAGE_FAULT");
        } else if fr.contains("EXIT: VMMCALL") || fr.contains("EXIT: VMCALL") {
            self.mpq += 1;
            self.dcw += 1;
            self.eel = String::from("HYPERCALL");
        } else if fr.contains("CREATED") {
            self.dna += 1;
        } else if fr.contains("STOPPED") || fr.contains("TRIPLE FAULT") {
            if self.dna > 0 {
                self.dna -= 1;
            }
        }
        
        
        if let Some(mai) = fr.du("RIP=0x") {
            let maj = &fr[mai + 6..];
            let ci = maj.du(|r: char| !r.ofp()).unwrap_or(maj.len());
            if let Ok(pc) = u64::wa(&maj[..ci], 16) {
                self.hpr = pc;
            }
        }
        
        
        if fr.contains("REGS ") {
            self.vdg(fr);
        }
    }
    
    
    fn vdg(&mut self, fr: &str) {
        fn gou(fr: &str, adx: &str) -> Option<u64> {
            let u = fr.du(adx)?;
            let ay = u + adx.len();
            let kr = &fr[ay..];
            let ci = kr.du(|r: char| !r.ofp()).unwrap_or(kr.len());
            u64::wa(&kr[..ci], 16).bq()
        }
        
        if let Some(p) = gou(fr, "RAX=0x") { self.lyq = p; }
        if let Some(p) = gou(fr, "RBX=0x") { self.lyr = p; }
        if let Some(p) = gou(fr, "RCX=0x") { self.lys = p; }
        if let Some(p) = gou(fr, "RDX=0x") { self.lyt = p; }
        if let Some(p) = gou(fr, "RIP=0x") { self.lyu = p; }
        if let Some(p) = gou(fr, "RSP=0x") { self.lyv = p; }
    }

    
    pub fn vr(&mut self, bs: u8) {
        use crate::keyboard::{V_, U_, AH_, AI_};
        match bs {
            eh if eh == AH_ => {
                if self.ahd > 0 { self.ahd -= 1; }
            }
            eh if eh == AI_ => {
                if self.ahd < 3 { self.ahd += 1; }
            }
            eh if eh == V_ => {
                if self.jc > 0 { self.jc -= 1; }
            }
            eh if eh == U_ => {
                self.jc += 1;
            }
            b'1' => self.ahd = 0, 
            b'2' => self.ahd = 1, 
            b'3' => self.ahd = 2, 
            b'4' => self.ahd = 3, 
            _ => {}
        }
    }

    
    pub fn ago(&mut self, mj: i32, ct: i32, d: u32, dxv: u32) {
        let bm = crate::graphics::scaling::fep() as i32;
        let dt = crate::graphics::scaling::bmi() as i32;
        
        
        if ct < bm + 4 && dt > 0 {
            let axb = d as i32 / 4;
            let acp = (mj / axb).v(3) as usize;
            self.ahd = acp;
        }
    }
}




pub fn po(g: &VmInspectorState, b: i32, c: i32, d: u32, i: u32) {
    let bm = super::apm();
    let dt = super::nk();
    if bm <= 0 || dt <= 0 { return; }
    
    
    let bio = ["Overview", "VM Exits", "Memory", "Registers"];
    let axb = d as i32 / 4;
    for (a, acp) in bio.iter().cf() {
        let gx = b + a as i32 * axb;
        let s = if a == g.ahd { HS_ } else { F_ };
        let ei = if a == g.ahd { 0xFF1F2937 } else { 0xFF0D1117 };
        crate::framebuffer::ah(gx as u32, c as u32, axb as u32, bm as u32, ei);
        super::kw(gx + 4, c + 2, acp, s);
    }
    
    crate::framebuffer::ah(b as u32, (c + bm) as u32, d, 1, HS_);
    
    let gl = c + bm + 4;
    let nd = i.ao(bm as u32 + 4);
    
    match g.ahd {
        0 => krf(g, b, gl, d, nd),
        1 => sct(g, b, gl, d, nd),
        2 => sdv(g, b, gl, d, nd),
        3 => sfa(g, b, gl, d, nd),
        _ => {}
    }
}


fn krf(g: &VmInspectorState, b: i32, c: i32, d: u32, i: u32) {
    let bm = super::apm();
    let mut ae = c;
    
    
    let dch = if g.dna > 0 { AK_ } else { F_ };
    let status = if g.dna > 0 { "RUNNING" } else { "NO VM" };
    super::kw(b, ae, &format!("Status: {}", status), dch);
    ae += bm;
    
    super::kw(b, ae, &format!("VMs Active: {}", g.dna), T_);
    ae += bm;
    
    super::kw(b, ae, &format!("Total Exits: {}", g.dcw), AO_);
    ae += bm;
    
    super::kw(b, ae, &format!("Last Exit: {}", g.eel), EZ_);
    ae += bm;
    
    if g.hpr != 0 {
        super::kw(b, ae, &format!("Guest RIP: 0x{:X}", g.hpr), BB_);
        ae += bm;
    }
    
    
    ae += bm / 2;
    let backend = crate::hypervisor::kbt();
    super::kw(b, ae, &format!("Backend: {}", backend), BO_);
    ae += bm;
    
    
    ae += bm / 2;
    crate::framebuffer::ah(b as u32, ae as u32, d, 1, 0xFF30363D);
    ae += 4;
    
    
    super::kw(b, ae, "Recent VM Events:", HS_);
    ae += bm;
    
    let olq = ((i as i32 - (ae - c)) / bm).am(0) as usize;
    let ay = if g.events.len() > olq {
        g.events.len() - olq
    } else {
        0
    };
    
    for aiz in g.events[ay..].iter() {
        if ae + bm > c + i as i32 { break; }
        
        let wi = aiz.aet;
        let tv = wi / 1000;
        let jn = wi % 1000;
        let bso = format!("{:02}:{:02}.{:03}", tv / 60, tv % 60, jn);
        
        
        let aem = ((d as i32 - 80) / super::nk()).am(10) as usize;
        let fr = if aiz.message.len() > aem {
            &aiz.message[..aem]
        } else {
            &aiz.message
        };
        
        super::kw(b, ae, &bso, F_);
        super::kw(b + 70, ae, fr, T_);
        ae += bm;
    }
}


fn sct(g: &VmInspectorState, b: i32, c: i32, d: u32, dxv: u32) {
    let bm = super::apm();
    let mut ae = c;
    
    super::kw(b, ae, "VM Exit Breakdown:", HS_);
    ae += bm + 4;
    
    
    let ith = [
        ("CPUID", g.bmp, BB_),
        ("I/O", g.ank, AK_),
        ("MSR", g.bkn, BO_),
        ("HLT", g.axz, AO_),
        ("NPF/EPT", g.cay, AW_),
        ("VMCALL", g.mpq, EZ_),
    ];
    
    let ukz = ith.iter().map(|(_, r, _)| *r).am().unwrap_or(1).am(1);
    let gar = d.ao(120) as u64;
    
    for (j, az, s) in &ith {
        let cu = format!("{:<8} {:>8}", j, az);
        super::kw(b, ae, &cu, T_);
        
        
        let lo = if *az > 0 {
            ((*az as u64 * gar) / ukz).am(2)
        } else {
            0
        };
        if lo > 0 {
            crate::framebuffer::ah(
                (b + 120) as u32, (ae + 2) as u32,
                lo as u32, (bm - 4) as u32,
                *s,
            );
        }
        
        ae += bm;
    }
    
    
    ae += bm / 2;
    super::kw(b, ae, &format!("Total: {}", g.dcw), T_);
    ae += bm;
    
    
    if g.frame > 0 {
        let cel = g.dcw * 60 / g.frame.am(1);
        super::kw(b, ae, &format!("Rate: ~{} exits/sec", cel), F_);
    }
}


fn sdv(gxl: &VmInspectorState, b: i32, c: i32, d: u32, i: u32) {
    let bm = super::apm();
    let mut ae = c;
    
    super::kw(b, ae, "Guest Physical Memory Map:", HS_);
    ae += bm + 4;
    
    
    let afx = crate::hypervisor::vmi::kfk(64);
    
    for aoz in &afx {
        if ae + bm > c + i as i32 { break; }
        let s = match aoz.bwo {
            crate::hypervisor::vmi::MemoryRegionType::Jw => AK_,
            crate::hypervisor::vmi::MemoryRegionType::Nn => AW_,
            crate::hypervisor::vmi::MemoryRegionType::Bre => AO_,
            crate::hypervisor::vmi::MemoryRegionType::Nw => F_,
            crate::hypervisor::vmi::MemoryRegionType::Bbk => BO_,
            crate::hypervisor::vmi::MemoryRegionType::Afg => F_,
        };
        
        let gs = aoz.aw / 1024;
        let als = if gs >= 1024 {
            format!("{:>4} MB", gs / 1024)
        } else {
            format!("{:>4} KB", gs)
        };
        
        let cu = format!("0x{:09X} {} {}", aoz.ar, als, aoz.cu);
        super::kw(b, ae, &cu, s);
        ae += bm;
    }
    
    
    ae += bm;
    let xsk = if crate::hypervisor::vmi::zu() { "ENABLED" } else { "DISABLED" };
    let xsj = if crate::hypervisor::vmi::zu() { AK_ } else { F_ };
    super::kw(b, ae, &format!("VMI Engine: {}", xsk), xsj);
    ae += bm;
    
    
    let bfr = crate::hypervisor::vmi::ojm();
    if !bfr.is_empty() {
        super::kw(b, ae, "Live VMs:", HS_);
        ae += bm;
        for (ad, j, boo) in &bfr {
            if ae + bm > c + i as i32 { break; }
            let s = match *boo {
                "running" => AK_,
                "created" => BB_,
                "paused" => AO_,
                "stopped" => AW_,
                _ => F_,
            };
            super::kw(b, ae, &format!("  VM #{}: {} [{}]", ad, j, boo), s);
            ae += bm;
        }
    } else {
        super::kw(b, ae, "No VMs created yet", F_);
    }
}


fn sfa(g: &VmInspectorState, b: i32, c: i32, dxx: u32, dxv: u32) {
    let bm = super::apm();
    let mut ae = c;
    
    super::kw(b, ae, "Guest Registers (last snapshot):", HS_);
    ae += bm + 4;
    
    let abd = 200i32;
    
    
    let vug = [
        ("RAX", g.lyq),
        ("RBX", g.lyr),
        ("RCX", g.lys),
        ("RDX", g.lyt),
    ];
    
    let vuh = [
        ("RIP", g.lyu),
        ("RSP", g.lyv),
        ("Last Exit RIP", g.hpr),
    ];
    
    for (j, ap) in &vug {
        let s = if *ap != 0 { AK_ } else { F_ };
        super::kw(b, ae, &format!("{:<4} = 0x{:016X}", j, ap), s);
        ae += bm;
    }
    
    ae += bm / 2;
    
    for (j, ap) in &vuh {
        let s = if *ap != 0 { BB_ } else { F_ };
        super::kw(b, ae, &format!("{:<14} = 0x{:016X}", j, ap), s);
        ae += bm;
    }
    
    
    ae += bm;
    super::kw(b, ae, &format!("Last Exit: {}", g.eel), EZ_);
    ae += bm;
    super::kw(b, ae, &format!("Total Exits: {}", g.dcw), AO_);
}
