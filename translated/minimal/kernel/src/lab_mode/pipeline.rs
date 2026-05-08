










extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use super::{eh, bly, ew, qu,
            P_, F_, M_, AC_, AK_, AN_,
            BG_, AU_, DN_};
use super::trace_bus::{self, EventCategory, dxk};


#[derive(Clone, Copy, PartialEq, Eq)]
enum Stage {
    Input,
    Parser,
    Scheduler,
    Memory,
    Au,
    Interrupts,
    Output,
}

impl Stage {
    fn label(&self) -> &'static str {
        match self {
            Stage::Input       => "USER INPUT",
            Stage::Parser      => "SHELL / PARSER",
            Stage::Scheduler   => "SCHEDULER",
            Stage::Memory      => "MEMORY MGR",
            Stage::Au  => "FILE SYSTEM",
            Stage::Interrupts  => "IRQ / HW",
            Stage::Output      => "DISPLAY OUT",
        }
    }
    
    fn color(&self) -> u32 {
        match self {
            Stage::Input       => M_,
            Stage::Parser      => BG_,
            Stage::Scheduler   => AK_,
            Stage::Memory      => AC_,
            Stage::Au  => AU_,
            Stage::Interrupts  => DN_,
            Stage::Output      => 0xFF3FB950,
        }
    }
    
    fn icon(&self) -> &'static str {
        match self {
            Stage::Input       => ">>",
            Stage::Parser      => "{}", 
            Stage::Scheduler   => "<>",
            Stage::Memory      => "[]",
            Stage::Au  => "//",
            Stage::Interrupts  => "!!",
            Stage::Output      => "<-",
        }
    }
}

const RV_: [Stage; 7] = [
    Stage::Input, Stage::Parser, Stage::Scheduler,
    Stage::Memory, Stage::Au, Stage::Interrupts, Stage::Output,
];


struct StageActivity {
    
    heat: u16,
    
    hits: u64,
    
    last_msg: String,
}

impl StageActivity {
    fn new() -> Self {
        Self { heat: 0, hits: 0, last_msg: String::new() }
    }
}


pub struct PipelineState {
    
    stages: [StageActivity; 7],
    
    pub flows: Vec<Yy>,
    
    max_flows: usize,
    
    last_read_idx: u64,
    
    frame: u64,
    
    pub scroll: usize,
}


pub struct Yy {
    timestamp_ms: u64,
    from_stage: usize,
    to_stage: usize,
    label: String,
}

impl PipelineState {
    pub fn new() -> Self {
        Self {
            stages: [
                StageActivity::new(), StageActivity::new(),
                StageActivity::new(), StageActivity::new(),
                StageActivity::new(), StageActivity::new(),
                StageActivity::new(),
            ],
            flows: Vec::new(),
            max_flows: 50,
            last_read_idx: 0,
            frame: 0,
            scroll: 0,
        }
    }
    
    
    pub fn update(&mut self) {
        self.frame += 1;
        
        
        if self.frame % 3 == 0 {
            for j in &mut self.stages {
                j.heat = j.heat.saturating_sub(3);
            }
        }
        
        
        if self.frame % 5 != 0 { return; }
        
        let (events, new_idx) = dxk(self.last_read_idx, 50);
        if events.is_empty() {
            self.last_read_idx = new_idx;
            return;
        }
        
        for rt in &events {
            
            let (from, to) = match rt.category {
                EventCategory::Keyboard => (0, 1),   
                EventCategory::Syscall  => (1, 2),   
                EventCategory::Scheduler => (2, 3),  
                EventCategory::Memory   => (3, 6),   
                EventCategory::Au => (1, 4), 
                EventCategory::Interrupt => (5, 2),  
                EventCategory::Network  => (4, 6),   
                EventCategory::Security => (1, 5),   
                EventCategory::Custom   => (0, 6),   
                EventCategory::Hypervisor => (5, 6), 
            };
            
            
            self.stages[from].heat = 255;
            self.stages[from].hits += 1;
            self.stages[to].heat = 200;
            self.stages[to].hits += 1;
            
            
            if rt.message.len() < 40 {
                self.stages[to].last_msg = rt.message.clone();
            } else {
                self.stages[to].last_msg = String::from(&rt.message[..37]);
                self.stages[to].last_msg.push_str("...");
            }
            
            
            self.flows.push(Yy {
                timestamp_ms: rt.timestamp_ms,
                from_stage: from,
                to_stage: to,
                label: if rt.message.len() > 25 {
                    let mut j = String::from(&rt.message[..22]);
                    j.push_str("...");
                    j
                } else {
                    rt.message.clone()
                },
            });
        }
        
        
        if self.flows.len() > self.max_flows {
            let drain = self.flows.len() - self.max_flows;
            self.flows.drain(..drain);
        }
        
        self.last_read_idx = new_idx;
    }
    
    pub fn handle_key(&mut self, key: u8) {
        use crate::keyboard::{T_, S_, AM_, AO_};
        match key {
            T_ => self.scroll += 1,
            S_ => { if self.scroll > 0 { self.scroll -= 1; } }
            AM_ => self.scroll += 5,
            AO_ => self.scroll = self.scroll.saturating_sub(5),
            b'c' | b'C' => {
                self.flows.clear();
                self.scroll = 0;
                for j in &mut self.stages {
                    j.hits = 0;
                    j.heat = 0;
                    j.last_msg.clear();
                }
            }
            _ => {}
        }
    }

    
    pub fn handle_click(&mut self, afh: i32, ta: i32, w: u32, _h: u32) {
        let aq = ew();
        let ee = qu() + 1;
        if ee <= 0 || aq <= 0 { return; }

        
        let lei = 3;
        let jie = ee * lei + 4;
        if ta < jie {
            
            let abk = (w as i32 / 4).max(12 * aq);
            let gap = 2i32;
            
            if ta < ee + gap {
                let col = afh / (abk + aq);
                if col < 3 {
                    let idx = col as usize;
                    self.stages[idx].heat = 255; 
                }
            }
            
            else if ta < 2 * (ee + gap) {
                let col = afh / (abk + aq);
                if col < 3 {
                    let idx = 3 + col as usize;
                    if idx < 6 { self.stages[idx].heat = 255; }
                }
            }
            return;
        }

        
        let bpe = jie + 3 + ee + 2;
        if ta >= bpe {
            let row = ((ta - bpe) / ee) as usize;
            
            if row > 5 {
                self.scroll = self.scroll.saturating_sub(row - 5);
            }
        }
    }
}


pub fn draw(state: &PipelineState, x: i32, y: i32, w: u32, h: u32) {
    let aq = ew();
    let ee = qu() + 1;
    if ee <= 0 || aq <= 0 { return; }
    
    
    
    let qcw = ee * 4; 
    let abk = (w as i32 / 4).max(12 * aq); 
    let gap = 2i32;
    
    
    let dxz = y;
    cin(state, 0, x, dxz, abk as u32, ee, aq);
    ekj(x + abk - aq, dxz + ee / 2, aq, F_);
    cin(state, 1, x + abk + aq, dxz, abk as u32, ee, aq);
    ekj(x + 2 * abk, dxz + ee / 2, aq, F_);
    cin(state, 2, x + 2 * (abk + aq), dxz, abk as u32, ee, aq);
    
    
    let dya = y + ee + gap;
    cin(state, 3, x, dya, abk as u32, ee, aq);
    ekj(x + abk - aq, dya + ee / 2, aq, F_);
    cin(state, 4, x + abk + aq, dya, abk as u32, ee, aq);
    ekj(x + 2 * abk, dya + ee / 2, aq, F_);
    cin(state, 5, x + 2 * (abk + aq), dya, abk as u32, ee, aq);
    
    
    let jbl = y + 2 * (ee + gap);
    let non = x + abk + aq;
    cin(state, 6, non, jbl, abk as u32, ee, aq);
    
    
    let bpe = jbl + ee + gap + 2;
    crate::framebuffer::fill_rect(x as u32, bpe as u32, w, 1, 0xFF30363D);
    let jim = bpe + 3;
    
    let mut am = x;
    for (i, cdw) in RV_.iter().enumerate() {
        let hits = state.stages[i].hits;
        let label = format!("{}:{}", cdw.icon(), hits);
        let col = if state.stages[i].heat > 50 { cdw.color() } else { F_ };
        eh(am, jim, &label, col);
        am += (label.len() as i32 + 1) * aq;
        if am > x + w as i32 - 10 { break; }
    }
    
    
    let bub = jim + ee + 2;
    crate::framebuffer::fill_rect(x as u32, (bub - 1) as u32, w, 1, 0xFF30363D);
    
    eh(x, bub, "Pipeline Flow", M_);
    let hoa = format!("{} events", state.flows.len());
    let klj = x + w as i32 - (hoa.len() as i32 * aq) - 2;
    eh(klj, bub, &hoa, F_);
    
    let gc = bub + ee;
    let abc = h as i32 - (gc - y);
    if abc <= 0 { return; }
    
    let visible = (abc / ee) as usize;
    
    if state.flows.is_empty() {
        eh(x + 4, gc, "Waiting for events...", F_);
        return;
    }
    
    
    let av = state.flows.len();
    let end = av.saturating_sub(state.scroll);
    let start = end.saturating_sub(visible);
    
    let mut u = gc;
    for i in start..end {
        let bzu = &state.flows[i];
        let from = &RV_[bzu.from_stage];
        let to = &RV_[bzu.to_stage];
        
        
        let jy = lxv(bzu.timestamp_ms);
        eh(x, u, &jy, F_);
        
        
        let lzj = from.icon();
        let pkm = to.icon();
        let bu = x + 10 * aq;
        eh(bu, u, lzj, from.color());
        eh(bu + 3 * aq, u, ">", F_);
        eh(bu + 4 * aq, u, pkm, to.color());
        
        
        let dmu = bu + 8 * aq;
        let cmu = ((w as i32 - (dmu - x)) / aq) as usize;
        let desc = if bzu.label.len() > cmu && cmu > 3 {
            &bzu.label[..cmu.saturating_sub(1)]
        } else {
            &bzu.label
        };
        eh(dmu, u, desc, P_);
        
        u += ee;
        if u > y + h as i32 { break; }
    }
}


fn cin(state: &PipelineState, idx: usize, x: i32, y: i32, w: u32, h: i32, aq: i32) {
    let cdw = &RV_[idx];
    let cth = &state.stages[idx];
    
    
    let bg = if cth.heat > 150 {
        cue(0xFF161B22, cdw.color(), cth.heat as u32 / 4)
    } else if cth.heat > 50 {
        cue(0xFF161B22, cdw.color(), cth.heat as u32 / 8)
    } else {
        0xFF161B22
    };
    crate::framebuffer::fill_rect(x as u32, y as u32, w, h as u32, bg);
    
    
    let border = if cth.heat > 100 { cdw.color() } else { 0xFF30363D };
    
    crate::framebuffer::fill_rect(x as u32, y as u32, w, 1, border);
    crate::framebuffer::fill_rect(x as u32, (y + h - 1) as u32, w, 1, border);
    
    crate::framebuffer::fill_rect(x as u32, y as u32, 1, h as u32, border);
    crate::framebuffer::fill_rect((x + w as i32 - 1) as u32, y as u32, 1, h as u32, border);
    
    
    let label = cdw.label();
    let text_color = if cth.heat > 100 { cdw.color() } else { F_ };
    
    let bhn = x + 2;
    eh(bhn, y + 2, label, text_color);
}


fn ekj(x: i32, y: i32, _cw: i32, color: u32) {
    eh(x, y, ">", color);
}


fn cue(base: u32, accent: u32, adg: u32) -> u32 {
    let adg = adg.min(63);
    let ki = 63 - adg;
    let r = (((base >> 16) & 0xFF) * ki + ((accent >> 16) & 0xFF) * adg) / 63;
    let g = (((base >> 8) & 0xFF) * ki + ((accent >> 8) & 0xFF) * adg) / 63;
    let b = ((base & 0xFF) * ki + (accent & 0xFF) * adg) / 63;
    0xFF000000 | (r << 16) | (g << 8) | b
}


fn lxv(dh: u64) -> String {
    let j = dh / 1000;
    let m = j / 60;
    let yt = dh % 1000;
    format!("{:02}:{:02}.{:03}", m % 100, j % 60, yt)
}
