








use alloc::format;
use alloc::string::String;
use super::track::{Track, Note};
use super::{AF_, Df};
use core::sync::atomic::Ordering;






const BTY_: u32 = 1;

const BTX_: u32 = 8;

const VT_: u32 = 48;

const RB_: u32 = 24;

const AHJ_: u8 = 24;  

const CIT_: u8 = 96;  


mod colors {
    pub const Rb: u32 = 0x1A1A2E;
    pub const CBM_: u32 = 0x2A2A3E;
    pub const CBL_: u32 = 0x3A3A4E;
    pub const AVZ_: u32 = 0x5A5A6E;
    pub const DFD_: u32 = 0x222236;
    pub const BNW_: u32 = 0x181828;
    pub const CGL_: u32 = 0x101020;
    pub const AFN_: u32 = 0xCCCCDD;
    pub const AFM_: u32 = 0x333344;
    pub const CGM_: u32 = 0x888899;
    pub const DBV_: u32 = 0x151530;
    pub const DBW_: u32 = 0xAAAABB;
    pub const DYV_: u32 = 0xFFFFFF;
    pub const Acd: u32 = 0xFF4444;
    pub const Aot: u32 = 0x4488FF;
    pub const Xd: u32 = 0x44FF44;
}






pub struct PianoRoll {
    
    pub x: u32,
    
    pub y: u32,
    
    pub width: u32,
    
    pub height: u32,
    
    pub h_zoom: u32,
    
    pub key_height: u32,
    
    pub scroll_x: u32,
    
    pub scroll_y: u8,
    
    pub selected_note: Option<usize>,
    
    pub cursor_tick: u32,
    pub cursor_pitch: u8,
    
    pub grid_snap: u32,
}

impl PianoRoll {
    
    pub fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
            h_zoom: BTY_,
            key_height: BTX_,
            scroll_x: 0,
            scroll_y: AHJ_,
            selected_note: None,
            cursor_tick: 0,
            cursor_pitch: 60, 
            grid_snap: AF_ / 4, 
        }
    }

    
    pub fn draw(&self, track: &Track, playhead_tick: u32) {
        let fb_w = crate::framebuffer::X_.load(Ordering::Relaxed) as u32;
        let fb_h = crate::framebuffer::W_.load(Ordering::Relaxed) as u32;
        if fb_w == 0 || fb_h == 0 { return; }

        
        let w = self.width.min(fb_w - self.x);
        let h = self.height.min(fb_h - self.y);
        let aju = self.x + VT_;
        let cza = self.y + RB_;
        let grid_w = w.saturating_sub(VT_);
        let grid_h = h.saturating_sub(RB_);

        
        crate::framebuffer::fill_rect(self.x, self.y, w, h, colors::Rb);

        
        self.draw_key_labels(grid_h);

        
        self.draw_grid(aju, cza, grid_w, grid_h);

        
        self.draw_timeline(aju, grid_w);

        
        self.draw_notes(track, aju, cza, grid_w, grid_h);

        
        self.draw_playhead(playhead_tick, aju, cza, grid_w, grid_h);

        
        self.draw_cursor(aju, cza, grid_w, grid_h);
    }

    
    fn draw_key_labels(&self, grid_h: u32) {
        let bhn = self.x;
        let clw = self.y + RB_;
        let bjw = grid_h / self.key_height;

        crate::framebuffer::fill_rect(bhn, clw, VT_, grid_h, colors::CGL_);

        for i in 0..bjw {
            let pitch = self.pitch_from_row(i);
            if pitch > 127 { continue; }

            let mf = clw + (bjw - 1 - i) * self.key_height;
            let bhi = ihs(pitch);

            
            let mvo = if bhi { colors::AFM_ } else { colors::AFN_ };
            crate::framebuffer::fill_rect(bhn, mf, VT_ - 2, self.key_height, mvo);

            
            if pitch % 12 == 0 || pitch == self.cursor_pitch {
                let name = crate::audio::tables::bno(pitch);
                let amb = crate::audio::tables::bui(pitch);
                let label = format!("{}{}", name, amb);
                crate::framebuffer::draw_text(&label, bhn + 4, mf + 1, colors::CGM_);
            }
        }
    }

    
    fn draw_grid(&self, hc: u32, jh: u32, fz: u32, agl: u32) {
        let bjw = agl / self.key_height;

        
        for i in 0..bjw {
            let pitch = self.pitch_from_row(i);
            if pitch > 127 { continue; }

            let mf = jh + (bjw - 1 - i) * self.key_height;
            let oij = if ihs(pitch) {
                colors::BNW_
            } else {
                colors::DFD_
            };
            crate::framebuffer::fill_rect(hc, mf, fz, self.key_height, oij);

            
            crate::framebuffer::mn(hc, mf, fz, colors::CBM_);
        }

        
        let ask = AF_ * 4; 
        let gyr = fz / self.h_zoom.max(1);

        let start_tick = self.scroll_x;
        let end_tick = start_tick + gyr;

        
        let fxc = (start_tick / ask) * ask;
        let mut tick = fxc;
        while tick <= end_tick {
            let p = self.tick_to_pixel(tick, hc);
            if p >= hc && p < hc + fz {
                crate::framebuffer::zv(p, jh, agl, colors::AVZ_);
            }
            tick += ask;
        }

        
        let lwf = (start_tick / AF_) * AF_;
        tick = lwf;
        while tick <= end_tick {
            if tick % ask != 0 { 
                let p = self.tick_to_pixel(tick, hc);
                if p >= hc && p < hc + fz {
                    crate::framebuffer::zv(p, jh, agl, colors::CBL_);
                }
            }
            tick += AF_;
        }
    }

    
    fn draw_timeline(&self, hc: u32, fz: u32) {
        let ty = self.y;
        crate::framebuffer::fill_rect(self.x, ty, self.width, RB_, colors::DBV_);

        let ask = AF_ * 4;
        let gyr = fz / self.h_zoom.max(1);
        let start_tick = self.scroll_x;
        let end_tick = start_tick + gyr;

        let fxc = (start_tick / ask) * ask;
        let mut tick = fxc;
        while tick <= end_tick {
            let jzo = tick / ask + 1;
            let p = self.tick_to_pixel(tick, hc);
            if p >= hc && p < hc + fz {
                let label = format!("{}", jzo);
                crate::framebuffer::draw_text(&label, p + 2, ty + 4, colors::DBW_);
                crate::framebuffer::zv(p, ty, RB_, colors::AVZ_);
            }
            tick += ask;
        }
    }

    
    fn draw_notes(&self, track: &Track, hc: u32, jh: u32, fz: u32, agl: u32) {
        let bjw = agl / self.key_height;

        for (i, note) in track.notes.iter().enumerate() {
            
            let gjq = self.tick_to_pixel(note.start_tick, hc);
            let iqw = self.tick_to_pixel(note.end_tick(), hc);
            let nlg = (iqw.saturating_sub(gjq)).max(2);

            
            if note.pitch < self.scroll_y || note.pitch >= self.scroll_y + bjw as u8 {
                continue;
            }

            
            if iqw < hc || gjq > hc + fz {
                continue;
            }

            let oil = (note.pitch - self.scroll_y) as u32;
            let gjr = jh + (bjw - 1 - oil) * self.key_height + 1;

            
            let eks = gjq.max(hc);
            let dnr = nlg.min(hc + fz - eks);

            
            let brightness = note.velocity as u32 * 100 / 127;
            let nkv = fgh(track.color, brightness);

            
            crate::framebuffer::fill_rect(eks, gjr, dnr, self.key_height - 2, nkv);

            
            if self.selected_note == Some(i) {
                crate::framebuffer::draw_rect(eks, gjr, dnr, self.key_height - 2, colors::Aot);
            }

            
            if dnr > 24 {
                let name = crate::audio::tables::bno(note.pitch);
                crate::framebuffer::draw_text(name, eks + 2, gjr + 1, 0xFFFFFF);
            }
        }
    }

    
    fn draw_playhead(&self, tick: u32, hc: u32, jh: u32, fz: u32, agl: u32) {
        let p = self.tick_to_pixel(tick, hc);
        if p >= hc && p < hc + fz {
            crate::framebuffer::zv(p, jh, agl, colors::Acd);
            
            for i in 0..4u32 {
                crate::framebuffer::mn(p.saturating_sub(i), jh.saturating_sub(i + 1), i * 2 + 1, colors::Acd);
            }
        }
    }

    
    fn draw_cursor(&self, hc: u32, jh: u32, fz: u32, agl: u32) {
        let bjw = agl / self.key_height;

        
        let cx = self.tick_to_pixel(self.cursor_tick, hc);
        if cx >= hc && cx < hc + fz {
            crate::framebuffer::zv(cx, jh, agl, colors::Xd);
        }

        
        if self.cursor_pitch >= self.scroll_y && self.cursor_pitch < self.scroll_y + bjw as u8 {
            let row = (self.cursor_pitch - self.scroll_y) as u32;
            let u = jh + (bjw - 1 - row) * self.key_height;
            crate::framebuffer::co(hc, u, fz, self.key_height, colors::Xd, 40);
        }
    }

    

    
    fn tick_to_pixel(&self, tick: u32, aju: u32) -> u32 {
        if tick >= self.scroll_x {
            aju + (tick - self.scroll_x) * self.h_zoom
        } else {
            aju 
        }
    }

    
    pub fn qqm(&self, p: u32, aju: u32) -> u32 {
        if p >= aju && self.h_zoom > 0 {
            self.scroll_x + (p - aju) / self.h_zoom
        } else {
            self.scroll_x
        }
    }

    
    fn pitch_from_row(&self, row: u32) -> u8 {
        let pitch = self.scroll_y as u32 + row;
        if pitch > 127 { 127 } else { pitch as u8 }
    }

    

    
    pub fn scroll_left(&mut self) {
        let fig = AF_ * 4;
        self.scroll_x = self.scroll_x.saturating_sub(fig);
    }

    
    pub fn scroll_right(&mut self) {
        let fig = AF_ * 4;
        self.scroll_x += fig;
    }

    
    pub fn scroll_up(&mut self) {
        if self.scroll_y < CIT_ - 12 {
            self.scroll_y += 12; 
        }
    }

    
    pub fn scroll_down(&mut self) {
        if self.scroll_y > AHJ_ + 12 {
            self.scroll_y -= 12;
        } else {
            self.scroll_y = AHJ_;
        }
    }

    
    pub fn rdq(&mut self) {
        if self.h_zoom < 8 {
            self.h_zoom += 1;
        }
    }

    
    pub fn rdr(&mut self) {
        if self.h_zoom > 1 {
            self.h_zoom -= 1;
        }
    }

    
    pub fn cursor_right(&mut self) {
        self.cursor_tick += self.grid_snap;
    }

    
    pub fn cursor_left(&mut self) {
        self.cursor_tick = self.cursor_tick.saturating_sub(self.grid_snap);
    }

    
    pub fn cursor_up(&mut self) {
        if self.cursor_pitch < 127 {
            self.cursor_pitch += 1;
        }
    }

    
    pub fn cursor_down(&mut self) {
        if self.cursor_pitch > 0 {
            self.cursor_pitch -= 1;
        }
    }

    
    pub fn qxf(&mut self) {
        if self.grid_snap > 0 {
            let bix = self.cursor_tick % self.grid_snap;
            if bix > self.grid_snap / 2 {
                self.cursor_tick += self.grid_snap - bix;
            } else {
                self.cursor_tick -= bix;
            }
        }
    }

    
    pub fn qvy(&mut self, division: &str) {
        self.grid_snap = match division {
            "1" | "whole" => AF_ * 4,
            "1/2" | "half" => AF_ * 2,
            "1/4" | "quarter" => AF_,
            "1/8" | "eighth" => AF_ / 2,
            "1/16" | "sixteenth" => AF_ / 4,
            "1/32" | "thirtysecond" => AF_ / 8,
            "off" | "free" => 1,
            _ => self.grid_snap, 
        };
    }

    
    pub fn pxu(&self, track: &mut Track, velocity: u8, duration_ticks: u32) {
        let note = Note::new(self.cursor_pitch, velocity, self.cursor_tick, duration_ticks);
        track.add_note(note);
    }

    
    pub fn qco(&self, track: &mut Track) -> bool {
        let nlh = track.notes_at_tick(self.cursor_tick);
        if let Some(note) = nlh.iter().find(|ae| ae.pitch == self.cursor_pitch) {
            let idx = track.notes.iter().position(|ae|
                ae.pitch == note.pitch && ae.start_tick == note.start_tick
            );
            if let Some(idx) = idx {
                track.remove_note(idx);
                return true;
            }
        }
        false
    }
}






fn ihs(pitch: u8) -> bool {
    matches!(pitch % 12, 1 | 3 | 6 | 8 | 10) 
}


fn fgh(color: u32, brightness: u32) -> u32 {
    let r = ((color >> 16) & 0xFF) * brightness / 100;
    let g = ((color >> 8) & 0xFF) * brightness / 100;
    let b = (color & 0xFF) * brightness / 100;
    (r.min(255) << 16) | (g.min(255) << 8) | b.min(255)
}


pub fn pij(track: &Track, bars: u32) -> String {
    let mut j = String::new();
    let ask = AF_ * 4;
    let total_ticks = ask * bars;
    let cols = (bars * 16) as usize; 
    let pjg = ask / 16;

    j.push_str(&format!("Piano Roll: \"{}\" — {} bars, {} notes\n",
        track.name_str(), bars, track.notes.len()));
    j.push_str(&format!("Grid: 1/16 note | {} = {}\n\n", track.waveform.name(),
        if track.armed { "ARMED" } else { "" }));

    
    j.push_str("     │");
    for bar in 0..bars {
        j.push_str(&format!("{:^16}", bar + 1));
    }
    j.push_str("\n     │");
    for _ in 0..bars {
        j.push_str("────────────────");
    }
    j.push('\n');

    
    let (min_pitch, max_pitch) = if track.notes.is_empty() {
        (57, 72) 
    } else {
        let min = track.notes.iter().map(|ae| ae.pitch).min().unwrap_or(60);
        let max = track.notes.iter().map(|ae| ae.pitch).max().unwrap_or(72);
        (min.saturating_sub(2), (max + 2).min(127))
    };

    
    for pitch in (min_pitch..=max_pitch).rev() {
        let name = crate::audio::tables::bno(pitch);
        let amb = crate::audio::tables::bui(pitch);
        let dsl = pitch % 12 == 0;
        
        j.push_str(&format!("{}{:<2} {}│", name, amb,
            if dsl { "─" } else { " " }));

        for col in 0..cols {
            let tick = col as u32 * pjg;
            let active = track.notes.iter().any(|ae|
                ae.pitch == pitch && ae.start_tick <= tick && tick < ae.end_tick()
            );
            let mtg = track.notes.iter().any(|ae|
                ae.pitch == pitch && ae.start_tick == tick
            );

            if mtg {
                j.push('█');
            } else if active {
                j.push('▓');
            } else if col % 16 == 0 {
                j.push('│');
            } else if col % 4 == 0 {
                j.push('┊');
            } else {
                j.push('·');
            }
        }
        j.push('\n');
    }

    j
}
