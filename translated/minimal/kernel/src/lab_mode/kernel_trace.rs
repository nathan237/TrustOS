











extern crate alloc;

use alloc::vec::Vec;
use alloc::string::String;
use alloc::format;
use super::{eh, ew, qu,
            P_, F_, M_, NF_};
use super::trace_bus::{Dw, EventCategory, ocz, total_count, dxk};


pub struct KernelTraceState {
    
    pub events: Vec<Dw>,
    
    pub scroll: usize,
    
    pub auto_scroll: bool,
    
    pub last_read_idx: u64,
    
    pub filters: [bool; 9],
    
    pub is_live: bool,
    
    refresh_counter: u64,
    
    pub paused: bool,
    
    pub selected_event: Option<usize>,
}

impl KernelTraceState {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            scroll: 0,
            auto_scroll: false,
            last_read_idx: 0,
            filters: [true; 9],
            is_live: false,
            refresh_counter: 0,
            paused: false,
            selected_event: None,
        }
    }
    
    pub fn qpk() -> Self {
        Self {
            events: Vec::new(),
            scroll: 0,
            auto_scroll: true,
            last_read_idx: 0,
            filters: [true; 9],
            is_live: true,
            refresh_counter: 0,
            paused: false,
            selected_event: None,
        }
    }
    
    
    pub fn update(&mut self) {
        self.refresh_counter += 1;
        if self.refresh_counter % 10 != 0 || self.paused {
            return;
        }
        
        
        let (new_events, new_idx) = dxk(self.last_read_idx, 100);
        if !new_events.is_empty() {
            self.events.extend(new_events);
            self.last_read_idx = new_idx;
            
            
            if self.events.len() > 500 {
                let drain = self.events.len() - 500;
                self.events.drain(..drain);
            }
            
            
            if self.auto_scroll {
                self.scroll = 0;
            }
        }
    }
    
    pub fn handle_key(&mut self, key: u8) {
        use crate::keyboard::{T_, S_, AM_, AO_};
        match key {
            T_ => {
                self.scroll += 1;
                self.auto_scroll = false;
            }
            S_ => {
                if self.scroll > 0 {
                    self.scroll -= 1;
                }
                if self.scroll == 0 {
                    self.auto_scroll = true;
                }
            }
            AM_ => {
                self.scroll += 10;
                self.auto_scroll = false;
            }
            AO_ => {
                self.scroll = self.scroll.saturating_sub(10);
                if self.scroll == 0 {
                    self.auto_scroll = true;
                }
            }
            
            b'p' | b'P' => {
                self.paused = !self.paused;
            }
            
            b'c' | b'C' => {
                self.events.clear();
                self.scroll = 0;
            }
            
            b'1'..=b'9' => {
                let idx = (key - b'1') as usize;
                if idx < self.filters.len() {
                    self.filters[idx] = !self.filters[idx];
                }
            }
            _ => {}
        }
    }

    
    pub fn handle_click(&mut self, afh: i32, ta: i32, w: u32, _h: u32) {
        let aq = ew();
        let ee = qu() + 1;
        if ee <= 0 || aq <= 0 { return; }

        let aej = ee;
        let fwy = aej;
        let ikz = fwy + ee + 2;

        
        if ta >= fwy && ta < fwy + ee {
            let cgs = [
                EventCategory::Interrupt,
                EventCategory::Scheduler,
                EventCategory::Memory,
                EventCategory::Au,
                EventCategory::Syscall,
                EventCategory::Keyboard,
            ];
            let mut dg = 0i32;
            for (i, hx) in cgs.iter().enumerate() {
                let label_len = hx.label().len() as i32 + 1;
                let dte = dg + label_len * aq;
                if afh >= dg && afh < dte {
                    let idx = *hx as usize;
                    if idx < self.filters.len() {
                        self.filters[idx] = !self.filters[idx];
                    }
                    return;
                }
                dg = dte;
                if dg > w as i32 - 20 { break; }
            }
            return;
        }

        
        if ta >= ikz {
            let row = ((ta - ikz) / ee) as usize;
            
            let filtered: Vec<usize> = self.events.iter().enumerate()
                .filter(|(_, e)| self.filters[e.category as usize])
                .map(|(i, _)| i)
                .collect();
            let crw = filtered.len();
            let end = crw.saturating_sub(self.scroll);
            let oe = 20usize; 
            let start = end.saturating_sub(oe);
            let hll = start + row;
            if hll < end {
                let cjb = filtered[hll];
                self.selected_event = if self.selected_event == Some(cjb) {
                    None 
                } else {
                    Some(cjb)
                };
            }
            return;
        }
    }
}


pub fn draw(state: &KernelTraceState, x: i32, y: i32, w: u32, h: u32) {
    let aq = ew();
    let ee = qu() + 1;
    
    if ee <= 0 || aq <= 0 { return; }
    
    
    let aej = ee;
    let av = total_count();
    let status = if state.paused {
        format!("PAUSED | {} events", av)
    } else if state.is_live {
        format!("LIVE | {} events", av)
    } else {
        format!("{} events | scroll: {}", av, state.scroll)
    };
    eh(x, y, &status, if state.paused { super::AK_ } else { F_ });
    
    
    let hyp = y + aej;
    let cgs = [
        EventCategory::Interrupt,
        EventCategory::Scheduler,
        EventCategory::Memory,
        EventCategory::Au,
        EventCategory::Syscall,
        EventCategory::Keyboard,
    ];
    let mut dg = x;
    for (i, hx) in cgs.iter().enumerate() {
        let enabled = state.filters[*hx as usize];
        let color = if enabled { hx.color() } else { 0xFF30363D };
        let label = hx.label();
        eh(dg, hyp, label, color);
        dg += (label.len() as i32 + 1) * aq;
        if dg > x + w as i32 - 20 { break; }
    }
    
    
    let bub = hyp + ee + 2;
    let gge = h as i32 - (bub - y);
    if gge <= 0 { return; }
    
    let oe = (gge / ee) as usize;
    
    
    let filtered: Vec<&Dw> = state.events.iter()
        .filter(|e| state.filters[e.category as usize])
        .collect();
    
    if filtered.is_empty() {
        eh(x + 4, bub + ee, "Waiting for events...", F_);
        return;
    }
    
    
    let crw = filtered.len();
    let end = crw.saturating_sub(state.scroll);

    
    let cig = if state.selected_event.is_some() { 4 } else { 0 };
    let nbw = oe.saturating_sub(cig);
    let start = end.saturating_sub(nbw);
    
    
    let hyq: Vec<usize> = state.events.iter().enumerate()
        .filter(|(_, e)| state.filters[e.category as usize])
        .map(|(i, _)| i)
        .collect();
    
    let mut u = bub;
    for i in start..end {
        let event = filtered[i];
        let nnv = if i < hyq.len() { hyq[i] } else { i };
        let hd = state.selected_event == Some(nnv);
        
        
        if hd {
            crate::framebuffer::fill_rect(x as u32, u as u32, w, ee as u32, 0xFF1F2937);
        }
        
        
        let im = event.timestamp_ms / 1000;
        let dh = event.timestamp_ms % 1000;
        let jy = format!("{:02}:{:02}.{:03}", im / 60, im % 60, dh);
        eh(x, u, &jy, F_);
        
        
        let hjx = x + (jy.len() as i32 + 1) * aq;
        let khq = event.category.label();
        eh(hjx, u, khq, event.category.color());
        
        
        let gij = hjx + (6 * aq);
        let eup;
        if let Some(nr) = event.syscall_nr {
            let irc = format!("#{}", nr);
            eh(gij, u, &irc, super::BG_);
            eup = gij + ((irc.len() as i32 + 1) * aq);
        } else {
            eup = gij;
        }
        
        
        let ndd = w as i32 - (eup - x);
        let nd = if aq > 0 { (ndd / aq) as usize } else { 20 };
        let bk = if event.message.len() > nd && nd > 3 {
            &event.message[..nd.saturating_sub(3)]
        } else {
            &event.message
        };
        eh(eup, u, bk, if hd { M_ } else { P_ });
        
        u += ee;
        if u > y + h as i32 { break; }
    }
    
    
    if let Some(sel_idx) = state.selected_event {
        if sel_idx < state.events.len() {
            let event = &state.events[sel_idx];
            let hru = y + h as i32 - (cig as i32 * ee);
            
            crate::framebuffer::fill_rect(x as u32, (hru - 2) as u32, w, 1, super::M_);
            
            let mut ad = hru;
            
            let header = format!("[{}] {}", event.category.label(), event.message);
            eh(x + 2, ad, &header, event.category.color());
            ad += ee;
            
            
            if let Some(nr) = event.syscall_nr {
                let name = super::trace_bus::dfe(nr);
                let detail = if let Some(args) = event.syscall_args {
                    format!("Syscall #{} ({}) args=[{:#x}, {:#x}, {:#x}]",
                        nr, name, args[0], args[1], args[2])
                } else {
                    format!("Syscall #{} ({})", nr, name)
                };
                eh(x + 2, ad, &detail, super::BG_);
                ad += ee;
                
                
                if let Some(ret) = event.syscall_ret {
                    let ogr = if ret < 0 {
                        format!("Return: {} (error)", ret)
                    } else {
                        format!("Return: {} ({:#x})", ret, ret)
                    };
                    eh(x + 2, ad, &ogr, if ret < 0 { super::AN_ } else { super::AC_ });
                }
            } else {
                
                let nso = format!("Payload: {} ({:#x}) | Timestamp: {}ms",
                    event.payload, event.payload, event.timestamp_ms);
                eh(x + 2, ad, &nso, F_);
            }
        }
    }
    
    
    if crw > oe {
        let bwn = bub;
        let ada = gge.max(1);
        let zo = ((oe as i32 * ada) / crw as i32).max(8);
        let ebq = if crw > oe {
            let jdp = crw - oe;
            let pos = jdp.saturating_sub(state.scroll);
            (pos as i32 * (ada - zo)) / jdp.max(1) as i32
        } else { 0 };
        
        let yc = (x + w as i32 - 3) as u32;
        
        crate::framebuffer::fill_rect(yc, bwn as u32, 2, ada as u32, 0xFF21262D);
        
        crate::framebuffer::fill_rect(yc, (bwn + ebq) as u32, 2, zo as u32, M_);
    }
}
