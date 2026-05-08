







extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use super::trace_bus::{self, EventCategory, Dw};


const P_: u32    = 0xFFE6EDF3;
const F_: u32     = 0xFF8B949E;
const AC_: u32   = 0xFF3FB950;
const AN_: u32     = 0xFFF85149;
const AK_: u32  = 0xFFD29922;
const AU_: u32    = 0xFF79C0FF;
const BG_: u32  = 0xFFBC8CFF;
const DN_: u32  = 0xFFD18616;
const IL_: u32      = 0xFFFF6B6B; 


pub struct VmInspectorState {
    
    pub events: Vec<Dw>,
    
    pub last_read_idx: u64,
    
    pub active_tab: usize,
    
    pub scroll: usize,
    
    pub frame: u64,
    
    pub vm_count: u32,
    pub total_exits: u64,
    pub last_exit_reason: String,
    pub last_guest_rip: u64,
    
    pub regs_rax: u64,
    pub regs_rbx: u64,
    pub regs_rcx: u64,
    pub regs_rdx: u64,
    pub regs_rip: u64,
    pub regs_rsp: u64,
    
    pub cpuid_exits: u64,
    pub io_exits: u64,
    pub msr_exits: u64,
    pub hlt_exits: u64,
    pub npf_exits: u64,
    pub vmcall_exits: u64,
    
    pub mem_view_addr: u64,
}

impl VmInspectorState {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            last_read_idx: 0,
            active_tab: 0,
            scroll: 0,
            frame: 0,
            vm_count: 0,
            total_exits: 0,
            last_exit_reason: String::from("(none)"),
            last_guest_rip: 0,
            regs_rax: 0,
            regs_rbx: 0,
            regs_rcx: 0,
            regs_rdx: 0,
            regs_rip: 0,
            regs_rsp: 0,
            cpuid_exits: 0,
            io_exits: 0,
            msr_exits: 0,
            hlt_exits: 0,
            npf_exits: 0,
            vmcall_exits: 0,
            mem_view_addr: 0x1000,
        }
    }

    
    pub fn update(&mut self) {
        self.frame += 1;
        if self.frame % 10 != 0 { return; }
        
        let (new_events, new_idx) = trace_bus::dxk(self.last_read_idx, 200);
        self.last_read_idx = new_idx;
        
        for rt in &new_events {
            if rt.category == EventCategory::Hypervisor {
                
                self.parse_hv_event(rt);
                
                
                self.events.push(rt.clone());
                if self.events.len() > 200 {
                    self.events.remove(0);
                }
            }
        }
    }

    
    fn parse_hv_event(&mut self, rt: &Dw) {
        let bk = &rt.message;
        
        
        if bk.contains("EXIT: CPUID") {
            self.cpuid_exits += 1;
            self.total_exits += 1;
            self.last_exit_reason = String::from("CPUID");
        } else if bk.contains("IO ") {
            self.io_exits += 1;
            self.total_exits += 1;
            self.last_exit_reason = String::from("I/O");
        } else if bk.contains("RDMSR") || bk.contains("WRMSR") {
            self.msr_exits += 1;
            self.total_exits += 1;
            self.last_exit_reason = String::from("MSR");
        } else if bk.contains("EXIT: HLT") {
            self.hlt_exits += 1;
            self.total_exits += 1;
            self.last_exit_reason = String::from("HLT");
        } else if bk.contains("NPF_VIOLATION") || bk.contains("EPT_VIOLATION") {
            self.npf_exits += 1;
            self.total_exits += 1;
            self.last_exit_reason = String::from("PAGE_FAULT");
        } else if bk.contains("EXIT: VMMCALL") || bk.contains("EXIT: VMCALL") {
            self.vmcall_exits += 1;
            self.total_exits += 1;
            self.last_exit_reason = String::from("HYPERCALL");
        } else if bk.contains("CREATED") {
            self.vm_count += 1;
        } else if bk.contains("STOPPED") || bk.contains("TRIPLE FAULT") {
            if self.vm_count > 0 {
                self.vm_count -= 1;
            }
        }
        
        
        if let Some(grr) = bk.find("RIP=0x") {
            let grs = &bk[grr + 6..];
            let end = grs.find(|c: char| !c.is_ascii_hexdigit()).unwrap_or(grs.len());
            if let Ok(rip) = u64::from_str_radix(&grs[..end], 16) {
                self.last_guest_rip = rip;
            }
        }
        
        
        if bk.contains("REGS ") {
            self.parse_regs(bk);
        }
    }
    
    
    fn parse_regs(&mut self, bk: &str) {
        fn dcg(bk: &str, nm: &str) -> Option<u64> {
            let pos = bk.find(nm)?;
            let start = pos + nm.len();
            let ef = &bk[start..];
            let end = ef.find(|c: char| !c.is_ascii_hexdigit()).unwrap_or(ef.len());
            u64::from_str_radix(&ef[..end], 16).ok()
        }
        
        if let Some(v) = dcg(bk, "RAX=0x") { self.regs_rax = v; }
        if let Some(v) = dcg(bk, "RBX=0x") { self.regs_rbx = v; }
        if let Some(v) = dcg(bk, "RCX=0x") { self.regs_rcx = v; }
        if let Some(v) = dcg(bk, "RDX=0x") { self.regs_rdx = v; }
        if let Some(v) = dcg(bk, "RIP=0x") { self.regs_rip = v; }
        if let Some(v) = dcg(bk, "RSP=0x") { self.regs_rsp = v; }
    }

    
    pub fn handle_key(&mut self, key: u8) {
        use crate::keyboard::{T_, S_, AI_, AJ_};
        match key {
            k if k == AI_ => {
                if self.active_tab > 0 { self.active_tab -= 1; }
            }
            k if k == AJ_ => {
                if self.active_tab < 3 { self.active_tab += 1; }
            }
            k if k == T_ => {
                if self.scroll > 0 { self.scroll -= 1; }
            }
            k if k == S_ => {
                self.scroll += 1;
            }
            b'1' => self.active_tab = 0, 
            b'2' => self.active_tab = 1, 
            b'3' => self.active_tab = 2, 
            b'4' => self.active_tab = 3, 
            _ => {}
        }
    }

    
    pub fn handle_click(&mut self, fe: i32, ly: i32, w: u32, _h: u32) {
        let ch = crate::graphics::scaling::cgu() as i32;
        let aq = crate::graphics::scaling::agg() as i32;
        
        
        if ly < ch + 4 && aq > 0 {
            let zm = w as i32 / 4;
            let tab = (fe / zm).min(3) as usize;
            self.active_tab = tab;
        }
    }
}




pub fn draw(state: &VmInspectorState, x: i32, y: i32, w: u32, h: u32) {
    let ch = super::qu();
    let aq = super::ew();
    if ch <= 0 || aq <= 0 { return; }
    
    
    let tabs = ["Overview", "VM Exits", "Memory", "Registers"];
    let zm = w as i32 / 4;
    for (i, tab) in tabs.iter().enumerate() {
        let bu = x + i as i32 * zm;
        let color = if i == state.active_tab { IL_ } else { F_ };
        let bg = if i == state.active_tab { 0xFF1F2937 } else { 0xFF0D1117 };
        crate::framebuffer::fill_rect(bu as u32, y as u32, zm as u32, ch as u32, bg);
        super::eh(bu + 4, y + 2, tab, color);
    }
    
    crate::framebuffer::fill_rect(x as u32, (y + ch) as u32, w, 1, IL_);
    
    let bn = y + ch + 4;
    let en = h.saturating_sub(ch as u32 + 4);
    
    match state.active_tab {
        0 => dnq(state, x, bn, w, en),
        1 => lir(state, x, bn, w, en),
        2 => ljl(state, x, bn, w, en),
        3 => lkm(state, x, bn, w, en),
        _ => {}
    }
}


fn dnq(state: &VmInspectorState, x: i32, y: i32, w: u32, h: u32) {
    let ch = super::qu();
    let mut u = y;
    
    
    let bdw = if state.vm_count > 0 { AC_ } else { F_ };
    let status = if state.vm_count > 0 { "RUNNING" } else { "NO VM" };
    super::eh(x, u, &format!("Status: {}", status), bdw);
    u += ch;
    
    super::eh(x, u, &format!("VMs Active: {}", state.vm_count), P_);
    u += ch;
    
    super::eh(x, u, &format!("Total Exits: {}", state.total_exits), AK_);
    u += ch;
    
    super::eh(x, u, &format!("Last Exit: {}", state.last_exit_reason), DN_);
    u += ch;
    
    if state.last_guest_rip != 0 {
        super::eh(x, u, &format!("Guest RIP: 0x{:X}", state.last_guest_rip), AU_);
        u += ch;
    }
    
    
    u += ch / 2;
    let backend = crate::hypervisor::fhy();
    super::eh(x, u, &format!("Backend: {}", backend), BG_);
    u += ch;
    
    
    u += ch / 2;
    crate::framebuffer::fill_rect(x as u32, u as u32, w, 1, 0xFF30363D);
    u += 4;
    
    
    super::eh(x, u, "Recent VM Events:", IL_);
    u += ch;
    
    let imj = ((h as i32 - (u - y)) / ch).max(0) as usize;
    let start = if state.events.len() > imj {
        state.events.len() - imj
    } else {
        0
    };
    
    for rt in state.events[start..].iter() {
        if u + ch > y + h as i32 { break; }
        
        let jy = rt.timestamp_ms;
        let im = jy / 1000;
        let dh = jy % 1000;
        let time_str = format!("{:02}:{:02}.{:03}", im / 60, im % 60, dh);
        
        
        let nd = ((w as i32 - 80) / super::ew()).max(10) as usize;
        let bk = if rt.message.len() > nd {
            &rt.message[..nd]
        } else {
            &rt.message
        };
        
        super::eh(x, u, &time_str, F_);
        super::eh(x + 70, u, bk, P_);
        u += ch;
    }
}


fn lir(state: &VmInspectorState, x: i32, y: i32, w: u32, _h: u32) {
    let ch = super::qu();
    let mut u = y;
    
    super::eh(x, u, "VM Exit Breakdown:", IL_);
    u += ch + 4;
    
    
    let exits = [
        ("CPUID", state.cpuid_exits, AU_),
        ("I/O", state.io_exits, AC_),
        ("MSR", state.msr_exits, BG_),
        ("HLT", state.hlt_exits, AK_),
        ("NPF/EPT", state.npf_exits, AN_),
        ("VMCALL", state.vmcall_exits, DN_),
    ];
    
    let nct = exits.iter().map(|(_, c, _)| *c).max().unwrap_or(1).max(1);
    let ctv = w.saturating_sub(120) as u64;
    
    for (name, count, color) in &exits {
        let label = format!("{:<8} {:>8}", name, count);
        super::eh(x, u, &label, P_);
        
        
        let ek = if *count > 0 {
            ((*count as u64 * ctv) / nct).max(2)
        } else {
            0
        };
        if ek > 0 {
            crate::framebuffer::fill_rect(
                (x + 120) as u32, (u + 2) as u32,
                ek as u32, (ch - 4) as u32,
                *color,
            );
        }
        
        u += ch;
    }
    
    
    u += ch / 2;
    super::eh(x, u, &format!("Total: {}", state.total_exits), P_);
    u += ch;
    
    
    if state.frame > 0 {
        let eps = state.total_exits * 60 / state.frame.max(1);
        super::eh(x, u, &format!("Rate: ~{} exits/sec", eps), F_);
    }
}


fn ljl(_state: &VmInspectorState, x: i32, y: i32, w: u32, h: u32) {
    let ch = super::qu();
    let mut u = y;
    
    super::eh(x, u, "Guest Physical Memory Map:", IL_);
    u += ch + 4;
    
    
    let regions = crate::hypervisor::vmi::fkc(64);
    
    for qd in &regions {
        if u + ch > y + h as i32 { break; }
        let color = match qd.region_type {
            crate::hypervisor::vmi::MemoryRegionType::Ram => AC_,
            crate::hypervisor::vmi::MemoryRegionType::Mmio => AN_,
            crate::hypervisor::vmi::MemoryRegionType::Rom => AK_,
            crate::hypervisor::vmi::MemoryRegionType::Reserved => F_,
            crate::hypervisor::vmi::MemoryRegionType::AcpiReclaimable => BG_,
            crate::hypervisor::vmi::MemoryRegionType::Unmapped => F_,
        };
        
        let size_kb = qd.size / 1024;
        let td = if size_kb >= 1024 {
            format!("{:>4} MB", size_kb / 1024)
        } else {
            format!("{:>4} KB", size_kb)
        };
        
        let label = format!("0x{:09X} {} {}", qd.base, td, qd.label);
        super::eh(x, u, &label, color);
        u += ch;
    }
    
    
    u += ch;
    let pss = if crate::hypervisor::vmi::lq() { "ENABLED" } else { "DISABLED" };
    let psr = if crate::hypervisor::vmi::lq() { AC_ } else { F_ };
    super::eh(x, u, &format!("VMI Engine: {}", pss), psr);
    u += ch;
    
    
    let aen = crate::hypervisor::vmi::ikn();
    if !aen.is_empty() {
        super::eh(x, u, "Live VMs:", IL_);
        u += ch;
        for (id, name, acr) in &aen {
            if u + ch > y + h as i32 { break; }
            let color = match *acr {
                "running" => AC_,
                "created" => AU_,
                "paused" => AK_,
                "stopped" => AN_,
                _ => F_,
            };
            super::eh(x, u, &format!("  VM #{}: {} [{}]", id, name, acr), color);
            u += ch;
        }
    } else {
        super::eh(x, u, "No VMs created yet", F_);
    }
}


fn lkm(state: &VmInspectorState, x: i32, y: i32, _w: u32, _h: u32) {
    let ch = super::qu();
    let mut u = y;
    
    super::eh(x, u, "Guest Registers (last snapshot):", IL_);
    u += ch + 4;
    
    let nk = 200i32;
    
    
    let oel = [
        ("RAX", state.regs_rax),
        ("RBX", state.regs_rbx),
        ("RCX", state.regs_rcx),
        ("RDX", state.regs_rdx),
    ];
    
    let oem = [
        ("RIP", state.regs_rip),
        ("RSP", state.regs_rsp),
        ("Last Exit RIP", state.last_guest_rip),
    ];
    
    for (name, val) in &oel {
        let color = if *val != 0 { AC_ } else { F_ };
        super::eh(x, u, &format!("{:<4} = 0x{:016X}", name, val), color);
        u += ch;
    }
    
    u += ch / 2;
    
    for (name, val) in &oem {
        let color = if *val != 0 { AU_ } else { F_ };
        super::eh(x, u, &format!("{:<14} = 0x{:016X}", name, val), color);
        u += ch;
    }
    
    
    u += ch;
    super::eh(x, u, &format!("Last Exit: {}", state.last_exit_reason), DN_);
    u += ch;
    super::eh(x, u, &format!("Total Exits: {}", state.total_exits), AK_);
}
