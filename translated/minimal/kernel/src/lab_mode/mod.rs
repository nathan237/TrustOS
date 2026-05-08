















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

use crate::keyboard::{T_, S_, AI_, AJ_, AM_, AO_};


const J_: u32 = 28;
const ECH_: u32 = 1;
const HO_: u32 = 6;
const FC_: u32 = 22;
const QJ_: u32 = 28;


const IK_: u32        = 0xFF0D1117;  
const NF_: u32  = 0xFF161B22;  
const NG_: u32 = 0xFF30363D; 
const KJ_: u32 = 0xFF1C2128;  
const P_: u32       = 0xFFE6EDF3;  
const F_: u32        = 0xFF8B949E;  
const M_: u32     = 0xFF58A6FF;  
const AC_: u32      = 0xFF3FB950;  
const AK_: u32     = 0xFFD29922;  
const AN_: u32        = 0xFFF85149;  
const BG_: u32     = 0xFFBC8CFF;  
const AU_: u32       = 0xFF79C0FF;  
const DN_: u32     = 0xFFD18616;  
const BQS_: u32   = 0xFF0D1117;  
const BQT_: u32 = 0xFF3FB950;
const TH_: u32   = 0xFF1F6FEB;  


pub static EY_: AtomicBool = AtomicBool::new(false);


static DWB_: AtomicU64 = AtomicU64::new(0);


#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PanelId {
    HardwareStatus = 0,
    KernelTrace = 1,
    CommandGuide = 2,
    FileTree = 3,
    TrustLangEditor = 4,
    Pipeline = 5,
    HexEditor = 6,
    VmInspector = 7,
    NetworkDashboard = 8,
}

impl PanelId {
    pub(crate) fn enm(i: usize) -> Self {
        match i {
            0 => PanelId::HardwareStatus,
            1 => PanelId::KernelTrace,
            2 => PanelId::CommandGuide,
            3 => PanelId::FileTree,
            4 => PanelId::TrustLangEditor,
            5 => PanelId::Pipeline,
            6 => PanelId::HexEditor,
            7 => PanelId::VmInspector,
            _ => PanelId::NetworkDashboard,
        }
    }
    
    fn title(&self) -> &'static str {
        match self {
            PanelId::HardwareStatus => "⚙ Hardware Status",
            PanelId::KernelTrace => "◈ Live Kernel Trace",
            PanelId::CommandGuide => "📖 Command Guide",
            PanelId::FileTree => "📁 File System Tree",
            PanelId::TrustLangEditor => "⌨ TrustLang Editor",
            PanelId::Pipeline => "⚙ Pipeline View",
            PanelId::HexEditor => "🔍 Hex Editor",
            PanelId::VmInspector => "🖥 VM Inspector",
            PanelId::NetworkDashboard => "🌐 Network",
        }
    }
    
    fn icon_color(&self) -> u32 {
        match self {
            PanelId::HardwareStatus => AC_,
            PanelId::KernelTrace => DN_,
            PanelId::CommandGuide => M_,
            PanelId::FileTree => AU_,
            PanelId::TrustLangEditor => BG_,
            PanelId::Pipeline => AK_,
            PanelId::HexEditor => AN_,
            PanelId::VmInspector => 0xFFFF6B6B,
            PanelId::NetworkDashboard => 0xFF00CED1,
        }
    }

    
    pub fn all() -> [PanelId; 9] {
        [
            PanelId::HardwareStatus, PanelId::KernelTrace, PanelId::CommandGuide,
            PanelId::FileTree, PanelId::TrustLangEditor, PanelId::Pipeline,
            PanelId::HexEditor, PanelId::VmInspector, PanelId::NetworkDashboard,
        ]
    }

    
    pub fn short_name(&self) -> &'static str {
        match self {
            PanelId::HardwareStatus => "Hardware",
            PanelId::KernelTrace => "Kernel Trace",
            PanelId::CommandGuide => "Cmd Guide",
            PanelId::FileTree => "File Tree",
            PanelId::TrustLangEditor => "TrustLang",
            PanelId::Pipeline => "Pipeline",
            PanelId::HexEditor => "Hex Editor",
            PanelId::VmInspector => "VM Inspector",
            PanelId::NetworkDashboard => "Network",
        }
    }

    
    pub fn category(&self) -> &'static str {
        match self {
            PanelId::VmInspector => "Hypervisor",
            PanelId::NetworkDashboard => "Network",
            _ => "Core",
        }
    }
}




pub struct ModuleSwitcher {
    
    pub open: bool,
    
    pub target_slot: usize,
    
    pub selected: usize,
}

impl ModuleSwitcher {
    pub fn new() -> Self {
        Self { open: false, target_slot: 0, selected: 0 }
    }
}




pub const BAY_: [PanelId; 7] = [
    PanelId::HardwareStatus, PanelId::KernelTrace, PanelId::CommandGuide,
    PanelId::FileTree, PanelId::TrustLangEditor, PanelId::Pipeline, PanelId::HexEditor,
];


const CHN_: [PanelId; 7] = [
    PanelId::KernelTrace, PanelId::TrustLangEditor, PanelId::CommandGuide,
    PanelId::FileTree, PanelId::HexEditor, PanelId::Pipeline, PanelId::HardwareStatus,
];


const CHO_: [PanelId; 7] = [
    PanelId::HardwareStatus, PanelId::KernelTrace, PanelId::Pipeline,
    PanelId::HexEditor, PanelId::FileTree, PanelId::KernelTrace, PanelId::CommandGuide,
];


const AJU_: [&str; 7] = [
    "Top-Left", "Mid-Top", "Top-Right", "Bot-Left", "Mid-Bot", "Mid-Embed", "Bot-Right",
];


pub struct LabState {
    
    pub focused_slot: usize,
    
    pub slot_assignment: [PanelId; 7],
    
    pub switcher: ModuleSwitcher,
    
    pub shell_input: String,
    
    pub shell_cursor: usize,
    
    pub hw_state: hardware::HardwareState,
    pub trace_state: kernel_trace::KernelTraceState,
    pub guide_state: guide::GuideState,
    pub tree_state: filetree::FileTreeState,
    pub editor_state: editor::EditorState,
    pub pipeline_state: pipeline::PipelineState,
    pub hex_state: hex_editor::HexEditorState,
    pub vm_inspector_state: vm_inspector::VmInspectorState,
    pub network_panel_state: network_panel::NetworkPanelState,
    pub demo_state: demo::DemoState,
    
    pub frame: u64,
    
    pub auto_scroll: bool,
}

impl LabState {
    pub fn new() -> Self {
        EY_.store(true, Ordering::SeqCst);
        Self {
            focused_slot: 0,
            slot_assignment: BAY_,
            switcher: ModuleSwitcher::new(),
            shell_input: String::new(),
            shell_cursor: 0,
            hw_state: hardware::HardwareState::new(),
            trace_state: kernel_trace::KernelTraceState::new(),
            guide_state: guide::GuideState::new(),
            tree_state: filetree::FileTreeState::new(),
            editor_state: editor::EditorState::new(),
            pipeline_state: pipeline::PipelineState::new(),
            hex_state: hex_editor::HexEditorState::new(),
            vm_inspector_state: vm_inspector::VmInspectorState::new(),
            network_panel_state: network_panel::NetworkPanelState::new(),
            demo_state: demo::DemoState::new(),
            frame: 0,
            auto_scroll: true,
        }
    }
    
    
    pub fn handle_key(&mut self, key: u8) {
        
        if self.switcher.open {
            self.handle_switcher_key(key);
            return;
        }

        
        if self.demo_state.active {
            self.demo_state.handle_key(key);
            return;
        }

        
        if key == 0x09 {
            self.focused_slot = (self.focused_slot + 1) % 7;
            return;
        }
        
        
        
        if key == 0x0D || key == 0x0A {
            if self.focused_module() == PanelId::TrustLangEditor {
                self.editor_state.handle_key(key);
                return;
            }
            if self.focused_module() == PanelId::FileTree {
                self.tree_state.handle_key(key);
                return;
            }
            if !self.shell_input.is_empty() {
                self.execute_shell_command();
                return;
            }
        }
        
        
        if key == 0x08 {
            
            if self.focused_module() == PanelId::TrustLangEditor {
                self.editor_state.handle_key(key);
                return;
            }
            
            if self.shell_cursor > 0 {
                self.shell_cursor -= 1;
                self.shell_input.remove(self.shell_cursor);
            }
            return;
        }
        
        
        self.dispatch_key(key);
    }

    
    fn dispatch_key(&mut self, key: u8) {
        match self.focused_module() {
            PanelId::HardwareStatus => self.hw_state.handle_key(key),
            PanelId::KernelTrace => self.trace_state.handle_key(key),
            PanelId::CommandGuide => self.guide_state.handle_key(key),
            PanelId::FileTree => self.tree_state.handle_key(key),
            PanelId::TrustLangEditor => self.editor_state.handle_key(key),
            PanelId::Pipeline => self.pipeline_state.handle_key(key),
            PanelId::HexEditor => self.hex_state.handle_key(key),
            PanelId::VmInspector => self.vm_inspector_state.handle_key(key),
            PanelId::NetworkDashboard => self.network_panel_state.handle_key(key),
        }
    }
    
    
    pub fn handle_char(&mut self, ch: char) {
        if self.switcher.open { return; }

        if self.demo_state.active {
            
            if ch == ' ' {
                self.demo_state.handle_key(0x20);
            }
            return;
        }

        match self.focused_module() {
            PanelId::TrustLangEditor => self.editor_state.handle_char(ch),
            PanelId::CommandGuide => self.guide_state.handle_char(ch),
            _ => {
                
                self.shell_input.insert(self.shell_cursor, ch);
                self.shell_cursor += 1;
            }
        }
    }
    
    
    pub(crate) fn execute_shell_command(&mut self) {
        let dm: String = self.shell_input.trim().chars().collect();
        let cmd: String = dm.chars().map(|c| c.to_ascii_lowercase()).collect();
        self.shell_input.clear();
        self.shell_cursor = 0;
        
        match cmd.as_str() {
            "hw" | "hardware" | "cpu" => {
                self.focus_module(PanelId::HardwareStatus);
            }
            "trace" | "log" | "events" => {
                self.focus_module(PanelId::KernelTrace);
            }
            "help" | "guide" | "commands" | "cmd" => {
                self.focus_module(PanelId::CommandGuide);
            }
            "fs" | "files" | "tree" | "ls" => {
                self.focus_module(PanelId::FileTree);
                self.tree_state.dirty = true;
                self.tree_state.handle_key(b'R'); 
            }
            "edit" | "editor" | "code" | "trustlang" => {
                self.focus_module(PanelId::TrustLangEditor);
            }
            "live" | "stream" | "bus" | "pipeline" | "pipe" => {
                self.focus_module(PanelId::Pipeline);
            }
            "hex" | "hexedit" | "hexdump" => {
                self.focus_module(PanelId::HexEditor);
            }
            "vm" | "vmi" | "inspector" | "hypervisor" => {
                self.focus_module(PanelId::VmInspector);
            }
            "net" | "network" | "tcp" | "packets" => {
                self.focus_module(PanelId::NetworkDashboard);
            }
            _ if cmd.starts_with("hex ") => {
                let path = dm[4..].trim();
                if !path.is_empty() {
                    self.hex_state.load_file(path);
                    self.focus_module(PanelId::HexEditor);
                }
            }
            "clear" | "cls" => {
                self.trace_state.events.clear();
                self.pipeline_state.flows.clear();
            }
            "demo" | "showcase" | "present" => {
                self.demo_state.start();
            }
            "refresh" | "r" => {
                self.tree_state.handle_key(b'R');
                self.hw_state.force_refresh();
            }
            "run" | "f5" => {
                self.editor_state.run_code();
                self.focus_module(PanelId::TrustLangEditor);
            }
            "test" | "labtest" | "uxtest" => {
                ux_test::ojh(self);
            }
            
            "swap" | "module" | "switch" => {
                self.open_switcher(self.focused_slot);
            }
            
            _ if cmd.starts_with("layout ") => {
                let preset = cmd[7..].trim();
                match preset {
                    "default" | "reset" => {
                        self.slot_assignment = BAY_;
                        trace_bus::bgi(trace_bus::EventCategory::Custom, "Layout: default", 0);
                    }
                    "dev" | "developer" => {
                        self.slot_assignment = CHN_;
                        trace_bus::bgi(trace_bus::EventCategory::Custom, "Layout: developer", 0);
                    }
                    "monitor" | "mon" => {
                        self.slot_assignment = CHO_;
                        trace_bus::bgi(trace_bus::EventCategory::Custom, "Layout: monitor", 0);
                    }
                    _ => {
                        trace_bus::bgi(trace_bus::EventCategory::Custom, "Unknown layout (try: default, dev, monitor)", 0);
                    }
                }
            }
            "slots" | "layout" => {
                
                for (i, vn) in self.slot_assignment.iter().enumerate() {
                    let bk = format!("Slot {} [{}]: {}", i, AJU_[i], vn.short_name());
                    trace_bus::emit(trace_bus::EventCategory::Custom, bk, i as u64);
                }
                self.focus_module(PanelId::KernelTrace);
            }
            _ => {
                
                trace_bus::bgi(
                    trace_bus::EventCategory::Custom,
                    "lab> unknown command",
                    0,
                );
            }
        }
    }
    
    
    pub fn handle_click(&mut self, da: i32, cm: i32, ca: u32, er: u32) {
        
        if self.switcher.open {
            self.handle_switcher_click(da, cm, ca, er);
            return;
        }

        let cx = 2i32;
        let u = J_ as i32 + 2;
        let aq = ca.saturating_sub(4);
        let ch = er.saturating_sub(J_ + 4);
        if aq < 200 || ch < 100 { return; }

        let aoq = cvn(cx, u, aq, ch);

        
        for (i, ej) in aoq.iter().enumerate() {
            if da >= ej.x && da < ej.x + ej.w as i32
                && cm >= ej.y && cm < ej.y + ej.h as i32
            {
                self.focused_slot = i;
                let pid = self.slot_assignment[i];

                
                let oyx = ej.x + ej.w as i32 - 24;
                if cm < ej.y + FC_ as i32 && da >= oyx {
                    self.open_switcher(i);
                    return;
                }

                
                let ho = ej.x + HO_ as i32;
                let bn = ej.y + FC_ as i32 + HO_ as i32;
                let hy = ej.w.saturating_sub(HO_ * 2);
                let en = ej.h.saturating_sub(FC_ + HO_ * 2);
                let afh = da - ho;
                let ta = cm - bn;

                
                self.dispatch_click(pid, afh, ta, hy, en);
                return;
            }
        }

        
        let gap = 4u32;
        let fat = u + (ch - QJ_) as i32;
        if cm >= fat && cm < fat + QJ_ as i32 {
            let blm = ew();
            if blm > 0 {
                let gos = 5; 
                let aua = cx + 8 + gos * blm;
                let kkz = ((da - aua) / blm).max(0) as usize;
                self.shell_cursor = kkz.min(self.shell_input.len());
            }
        }
    }

    
    fn dispatch_click(&mut self, pid: PanelId, fe: i32, ly: i32, w: u32, h: u32) {
        match pid {
            PanelId::HardwareStatus => self.hw_state.handle_click(fe, ly, w, h),
            PanelId::KernelTrace => self.trace_state.handle_click(fe, ly, w, h),
            PanelId::CommandGuide => self.guide_state.handle_click(fe, ly, w, h),
            PanelId::FileTree => self.tree_state.handle_click(fe, ly, w, h),
            PanelId::TrustLangEditor => self.editor_state.handle_click(fe, ly, w, h),
            PanelId::Pipeline => self.pipeline_state.handle_click(fe, ly, w, h),
            PanelId::HexEditor => self.hex_state.handle_click(fe, ly, w, h),
            PanelId::VmInspector => self.vm_inspector_state.handle_click(fe, ly, w, h),
            PanelId::NetworkDashboard => self.network_panel_state.handle_click(fe, ly, w, h),
        }
    }

    
    pub fn tick(&mut self) {
        self.frame += 1;
        self.hw_state.update();
        self.trace_state.update();
        self.pipeline_state.update();
        self.vm_inspector_state.update();
        
        if let Some(panel_idx) = self.demo_state.tick() {
            
            self.focused_slot = panel_idx.min(6);
        }
    }

    

    
    pub fn focused_module(&self) -> PanelId {
        self.slot_assignment[self.focused_slot]
    }

    
    pub fn slot_of(&self, vn: PanelId) -> Option<usize> {
        self.slot_assignment.iter().position(|m| *m == vn)
    }

    
    pub fn focus_module(&mut self, vn: PanelId) {
        if let Some(slot) = self.slot_of(vn) {
            self.focused_slot = slot;
        }
    }

    
    pub fn open_switcher(&mut self, slot: usize) {
        self.switcher.open = true;
        self.switcher.target_slot = slot;
        let current = self.slot_assignment[slot];
        self.switcher.selected = PanelId::all().iter().position(|aa| *aa == current).unwrap_or(0);
    }

    
    fn handle_switcher_key(&mut self, key: u8) {
        match key {
            0x1B => {
                
                self.switcher.open = false;
            }
            0x0D | 0x0A => {
                
                let all = PanelId::all();
                if self.switcher.selected < all.len() {
                    let vn = all[self.switcher.selected];
                    let slot = self.switcher.target_slot;
                    self.slot_assignment[slot] = vn;
                    let bk = format!("Slot {} [{}] -> {}",
                        slot, AJU_[slot], vn.short_name());
                    trace_bus::emit(trace_bus::EventCategory::Custom, bk, 0);
                }
                self.switcher.open = false;
            }
            k if k == T_ => {
                if self.switcher.selected > 0 {
                    self.switcher.selected -= 1;
                }
            }
            k if k == S_ => {
                let max = PanelId::all().len() - 1;
                if self.switcher.selected < max {
                    self.switcher.selected += 1;
                }
            }
            _ => {}
        }
    }

    
    fn handle_switcher_click(&mut self, da: i32, cm: i32, ca: u32, er: u32) {
        let cx = 2i32;
        let u = J_ as i32 + 2;
        let aq = ca.saturating_sub(4);
        let ch = er.saturating_sub(J_ + 4);
        if aq < 200 || ch < 100 { self.switcher.open = false; return; }

        let aoq = cvn(cx, u, aq, ch);
        let slot = self.switcher.target_slot;
        if slot >= aoq.len() { self.switcher.open = false; return; }
        let ej = &aoq[slot];

        
        let pad = 4i32;
        let fh = ej.x + pad;
        let hk = ej.y + pad;
        let cnp = ej.w.saturating_sub(8);
        let dca = ej.h.saturating_sub(8);

        if da >= fh && da < fh + cnp as i32 && cm >= hk && cm < hk + dca as i32 {
            
            let dto = hk + 24; 
            let ep = qu();
            if ep > 0 && cm >= dto {
                let entry = ((cm - dto) / ep) as usize;
                if entry < PanelId::all().len() {
                    self.switcher.selected = entry;
                    
                    
                    let vn = PanelId::all()[entry];
                    self.slot_assignment[slot] = vn;
                    let bk = format!("Slot {} [{}] -> {}",
                        slot, AJU_[slot], vn.short_name());
                    trace_bus::emit(trace_bus::EventCategory::Custom, bk, 0);
                    self.switcher.open = false;
                }
            }
        } else {
            
            self.switcher.open = false;
        }
    }
}

impl Drop for LabState {
    fn drop(&mut self) {
        
        EY_.store(false, Ordering::SeqCst);
    }
}




pub(crate) struct Cj {
    pub(crate) x: i32,
    pub(crate) y: i32,
    pub(crate) w: u32,
    pub(crate) h: u32,
}

pub(crate) fn cvn(cx: i32, u: i32, aq: u32, ch: u32) -> [Cj; 7] {
    let gap = 4u32;
    
    let en = ch.saturating_sub(QJ_ + gap);
    
    let col_w = (aq.saturating_sub(gap * 2)) / 3;
    let ep = (en.saturating_sub(gap)) / 2;
    
    let bm = cx;
    let x1 = cx + col_w as i32 + gap as i32;
    let x2 = cx + (col_w as i32 + gap as i32) * 2;
    let az = u;
    let y1 = u + ep as i32 + gap as i32;
    
    
    let bye = (aq as i32 - (x2 - cx)).max(40) as u32;
    
    
    let crz = ep.saturating_sub(gap) / 2;
    let gnc = ep.saturating_sub(crz + gap);
    let gnd = az + crz as i32 + gap as i32;
    
    [
        Cj { x: bm, y: az, w: col_w, h: ep },          
        Cj { x: x1, y: az, w: col_w, h: crz },        
        Cj { x: x2, y: az, w: bye, h: ep },         
        Cj { x: bm, y: y1, w: col_w, h: ep },          
        Cj { x: x1, y: y1, w: col_w, h: ep },          
        Cj { x: x1, y: gnd, w: col_w, h: gnc },     
        Cj { x: x2, y: y1, w: bye, h: ep },         
    ]
}


pub fn lji(state: &LabState, wx: i32, wy: i32, ca: u32, er: u32) {
    let cx = wx + 2;
    let u = wy + J_ as i32 + 2;
    let aq = ca.saturating_sub(4);
    let ch = er.saturating_sub(J_ + 4);
    
    if aq < 200 || ch < 100 {
        return;
    }
    
    
    crate::framebuffer::fill_rect(cx as u32, u as u32, aq, ch, IK_);
    
    
    let aoq = cvn(cx, u, aq, ch);
    
    
    for (i, ej) in aoq.iter().enumerate() {
        let pid = state.slot_assignment[i];
        let focused = i == state.focused_slot;
        blx(ej, pid, focused);
        
        
        let ho = ej.x + HO_ as i32;
        let bn = ej.y + FC_ as i32 + HO_ as i32;
        let hy = ej.w.saturating_sub(HO_ * 2);
        let en = ej.h.saturating_sub(FC_ + HO_ * 2);
        
        ljq(state, pid, ho, bn, hy, en);
    }
    
    
    let gap = 4u32;
    let fat = u + (ch - QJ_) as i32;
    lkq(state, cx, fat, aq, QJ_);

    
    if state.switcher.open {
        ljr(state, &aoq);
    }

    
    if state.demo_state.active {
        demo::ljz(&state.demo_state, wx, wy, ca, er);
    }
}


fn blx(ej: &Cj, pid: PanelId, focused: bool) {
    
    crate::framebuffer::fill_rect(ej.x as u32, ej.y as u32, ej.w, ej.h, NF_);
    
    
    let ri = if focused { TH_ } else { NG_ };
    ekq(ej.x, ej.y, ej.w, ej.h, ri);
    
    
    let gaf = if focused { 0xFF1F2937 } else { KJ_ };
    crate::framebuffer::fill_rect(
        (ej.x + 1) as u32, (ej.y + 1) as u32,
        ej.w.saturating_sub(2), FC_ - 1,
        gaf,
    );
    
    
    crate::framebuffer::fill_rect(
        (ej.x + 1) as u32, (ej.y + 1) as u32,
        ej.w.saturating_sub(2), 2,
        pid.icon_color(),
    );
    
    
    let title = pid.title();
    eh(ej.x + 8, ej.y + 6, title, P_);
    
    
    let zs = ej.x + ej.w as i32 - 22;
    let djr = if focused { M_ } else { F_ };
    eh(zs, ej.y + 6, "\u{25BC}", djr);
}


fn ljq(state: &LabState, pid: PanelId, x: i32, y: i32, w: u32, h: u32) {
    match pid {
        PanelId::HardwareStatus => hardware::draw(&state.hw_state, x, y, w, h),
        PanelId::KernelTrace => kernel_trace::draw(&state.trace_state, x, y, w, h),
        PanelId::CommandGuide => guide::draw(&state.guide_state, x, y, w, h),
        PanelId::FileTree => filetree::draw(&state.tree_state, x, y, w, h),
        PanelId::TrustLangEditor => editor::draw(&state.editor_state, x, y, w, h),
        PanelId::Pipeline => pipeline::draw(&state.pipeline_state, x, y, w, h),
        PanelId::HexEditor => hex_editor::draw(&state.hex_state, x, y, w, h),
        PanelId::VmInspector => vm_inspector::draw(&state.vm_inspector_state, x, y, w, h),
        PanelId::NetworkDashboard => network_panel::draw(&state.network_panel_state, x, y, w, h),
    }
}


fn ljr(state: &LabState, aoq: &[Cj; 7]) {
    let slot = state.switcher.target_slot;
    if slot >= aoq.len() { return; }
    let ej = &aoq[slot];

    
    let pad = 2;
    let fh = ej.x + pad;
    let hk = ej.y + pad;
    let cnp = ej.w.saturating_sub(4);
    let dca = ej.h.saturating_sub(4);
    crate::framebuffer::fill_rect(fh as u32, hk as u32, cnp, dca, 0xFF0D1117);
    ekq(fh, hk, cnp, dca, M_);

    
    let title = format!("Select Module (Slot {})", slot);
    eh(fh + 8, hk + 4, &title, M_);

    
    let jeq = hk + 22;
    crate::framebuffer::fill_rect((fh + 1) as u32, jeq as u32, cnp.saturating_sub(2), 1, NG_);

    
    let ch = qu();
    let all = PanelId::all();
    let current = state.slot_assignment[slot];
    let mut mf = jeq + 4;

    for (i, vn) in all.iter().enumerate() {
        if mf + ch > hk + dca as i32 - ch { break; } 

        let hd = i == state.switcher.selected;
        let is_current = *vn == current;

        
        if hd {
            crate::framebuffer::fill_rect(
                (fh + 2) as u32, mf as u32,
                cnp.saturating_sub(4), ch as u32,
                0xFF1F6FEB,
            );
        }

        
        let icon = if is_current { "*" } else { " " };
        let asi = if is_current { " [active]" } else { "" };
        let label = format!("{} {} {}{}", icon, vn.short_name(), vn.category(), asi);
        let color = if hd { P_ } else if is_current { AC_ } else { F_ };
        eh(fh + 8, mf + 2, &label, color);

        
        crate::framebuffer::fill_rect(
            (fh + cnp as i32 - 16) as u32, (mf + 4) as u32,
            8, 8, vn.icon_color(),
        );

        mf += ch;
    }

    
    let epf = hk + dca as i32 - ch;
    eh(fh + 8, epf, "Up/Down Enter Esc", F_);
}


fn lkq(state: &LabState, x: i32, y: i32, w: u32, h: u32) {
    crate::framebuffer::fill_rect(x as u32, y as u32, w, h, BQS_);
    
    crate::framebuffer::fill_rect(x as u32, y as u32, w, 1, NG_);
    
    
    let nh = "lab> ";
    eh(x + 8, y + 7, nh, BQT_);
    
    
    let aua = x + 8 + (nh.len() as i32 * ew());
    if state.shell_input.is_empty() {
        eh(aua, y + 7, "hw|trace|fs|edit|hex|vm|net|swap|layout|run|test", F_);
    } else {
        eh(aua, y + 7, &state.shell_input, P_);
    }
    
    
    if (state.frame / 30) % 2 == 0 {
        let cursor_x = aua + (state.shell_cursor as i32 * ew());
        crate::framebuffer::fill_rect(cursor_x as u32, (y + 6) as u32, 2, 14, M_);
    }
    
    
    let npr = state.focused_module().title();
    let hint = format!("[Tab] cycle | swap | layout | {}", npr);
    let drk = x + w as i32 - (hint.len() as i32 * ew()) - 8;
    eh(drk, y + 7, &hint, F_);
}




pub fn eh(x: i32, y: i32, text: &str, color: u32) {
    crate::graphics::scaling::ekr(x, y, text, color);
}


fn ew() -> i32 {
    crate::graphics::scaling::agg() as i32
}


fn qu() -> i32 {
    16 * crate::graphics::scaling::aqv() as i32
}


fn ekq(x: i32, y: i32, w: u32, h: u32, color: u32) {
    if w == 0 || h == 0 { return; }
    
    crate::framebuffer::fill_rect(x as u32, y as u32, w, 1, color);
    
    crate::framebuffer::fill_rect(x as u32, (y + h as i32 - 1) as u32, w, 1, color);
    
    crate::framebuffer::fill_rect(x as u32, y as u32, 1, h, color);
    
    crate::framebuffer::fill_rect((x + w as i32 - 1) as u32, y as u32, 1, h, color);
}


pub fn bly(x: i32, y: i32, w: u32, h: u32, aed: u32, fg: u32, bg: u32) {
    crate::framebuffer::fill_rect(x as u32, y as u32, w, h, bg);
    let rb = (w as u64 * aed.min(100) as u64 / 100) as u32;
    if rb > 0 {
        crate::framebuffer::fill_rect(x as u32, y as u32, rb, h, fg);
    }
}


pub fn raz(j: &str, max_w: u32) -> &str {
    let aq = ew() as u32;
    if aq == 0 { return j; }
    let nd = (max_w / aq) as usize;
    if j.len() <= nd {
        j
    } else if nd > 3 {
        &j[..nd - 3]
    } else {
        &j[..nd.min(j.len())]
    }
}
