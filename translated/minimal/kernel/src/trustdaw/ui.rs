









use alloc::format;
use alloc::string::String;
use core::sync::atomic::Ordering;

use super::piano_roll::PianoRoll;
use super::{Gw, Fb, GG_, Df, AF_, GB_};






mod colors {
    pub const Rb: u32 = 0x0D0D1A;
    pub const HF_: u32 = 0x1A1A2E;
    pub const ZA_: u32 = 0x151525;
    pub const BJS_: u32 = 0x121222;
    pub const DCC_: u32 = 0x1A1A3A;
    pub const AKY_: u32 = 0x2A2A4A;
    pub const AB_: u32 = 0xDDDDEE;
    pub const O_: u32 = 0x8888AA;
    pub const AY_: u32 = 0x555577;
    pub const PW_: u32 = 0x44DD44;
    pub const CYE_: u32 = 0xDD4444;
    pub const XW_: u32 = 0xFF2222;
    pub const MY_: u32 = 0x2A2A4A;
    pub const DHW_: u32 = 0x3A3A6A;
    pub const CKQ_: u32 = 0xFF8800;
    pub const CXH_: u32 = 0xFFDD00;
    pub const AHD_: u32 = 0x44CC44;
    pub const AHF_: u32 = 0xCCCC44;
    pub const AHE_: u32 = 0xCC4444;
    pub const WM_: u32 = 0x1A1A2A;
}


const RD_: u32 = 40;
const RC_: u32 = 180;
const JY_: u32 = 28;
const EMD_: u32 = 200;






pub struct DawUI {
    
    pub selected_track: usize,
    
    pub piano_roll: PianoRoll,
    
    pub show_piano_roll: bool,
    
    pub dirty: bool,
}

impl DawUI {
    
    pub fn new() -> Self {
        let fb_w = crate::framebuffer::X_.load(Ordering::Relaxed) as u32;
        let fb_h = crate::framebuffer::W_.load(Ordering::Relaxed) as u32;

        let iuq = RD_ + RC_;
        let nui = fb_h.saturating_sub(iuq);

        Self {
            selected_track: 0,
            piano_roll: PianoRoll::new(0, iuq, fb_w, nui),
            show_piano_roll: true,
            dirty: true,
        }
    }

    
    pub fn draw(&mut self) {
        let fb_w = crate::framebuffer::X_.load(Ordering::Relaxed) as u32;
        let fb_h = crate::framebuffer::W_.load(Ordering::Relaxed) as u32;
        if fb_w == 0 || fb_h == 0 { return; }

        
        crate::framebuffer::fill_rect(0, 0, fb_w, fb_h, colors::Rb);

        
        let project = super::Ce.lock();
        let mixer = super::Ex.lock();

        if let (Some(oa), Some(aif)) = (project.as_ref(), mixer.as_ref()) {
            
            self.draw_transport(fb_w, oa);

            
            self.draw_track_list(fb_w, oa, aif);

            
            if self.show_piano_roll {
                if let Some(track) = oa.tracks.get(self.selected_track) {
                    let nvo = GG_.load(Ordering::Relaxed);
                    self.piano_roll.draw(track, nvo);
                } else {
                    
                    let o = RD_ + RC_ + 20;
                    crate::framebuffer::draw_text("No tracks. Use 'daw track add <name>' to create one.",
                        20, o, colors::O_);
                }
            }
        } else {
            crate::framebuffer::draw_text("TrustDAW not initialized. Run 'daw init'",
                20, 20, colors::AB_);
        }

        self.dirty = false;
    }

    
    fn draw_transport(&self, fb_w: u32, oa: &super::track::Project) {
        crate::framebuffer::fill_rect(0, 0, fb_w, RD_, colors::ZA_);

        
        crate::framebuffer::draw_text("TrustDAW", 8, 4, colors::AB_);

        
        let name = oa.name_str();
        crate::framebuffer::draw_text(name, 100, 4, colors::O_);

        
        let playing = Gw.load(Ordering::Relaxed);
        let recording = Fb.load(Ordering::Relaxed);

        
        let dew = 250;
        if recording {
            crate::framebuffer::fill_circle(dew + 8, 12, 6, colors::XW_);
            crate::framebuffer::draw_text("REC", dew + 20, 4, colors::XW_);
        } else if playing {
            
            crate::framebuffer::fill_rect(dew, 6, 3, 12, colors::PW_);
            crate::framebuffer::draw_text("PLAY", dew + 20, 4, colors::PW_);
        } else {
            crate::framebuffer::fill_rect(dew, 6, 12, 12, colors::CYE_);
            crate::framebuffer::draw_text("STOP", dew + 20, 4, colors::AY_);
        }

        
        let bpm = Df.load(Ordering::Relaxed);
        let cuh = format!("BPM: {}", bpm);
        crate::framebuffer::draw_text(&cuh, 350, 4, colors::AB_);

        
        let pos = GG_.load(Ordering::Relaxed);
        let bar = pos / (AF_ * 4);
        let beat = (pos % (AF_ * 4)) / AF_;
        let tick = pos % AF_;
        let bdb = format!("{}:{:02}:{:03}", bar + 1, beat + 1, tick);
        crate::framebuffer::draw_text(&bdb, 450, 4, colors::AB_);

        
        let pmy = format!("Tracks: {}/{}", oa.tracks.len(), GB_);
        crate::framebuffer::draw_text(&pmy, 560, 4, colors::O_);

        
        crate::framebuffer::mn(0, RD_ - 1, fb_w, colors::AKY_);

        
        crate::framebuffer::draw_text(
            "[Space] Play/Stop  [R] Record  [+/-] BPM  [Tab] Track  [F5] Piano Roll  [F8] Export WAV",
            8, 22, colors::AY_
        );
    }

    
    fn draw_track_list(&self, fb_w: u32, oa: &super::track::Project, mixer: &super::mixer::Mixer) {
        let gc = RD_;
        crate::framebuffer::fill_rect(0, gc, fb_w, RC_, colors::BJS_);

        
        crate::framebuffer::fill_rect(0, gc, fb_w, JY_, colors::HF_);
        crate::framebuffer::draw_text(" # ", 4, gc + 6, colors::AY_);
        crate::framebuffer::draw_text("Track", 30, gc + 6, colors::AY_);
        crate::framebuffer::draw_text("Wave", 140, gc + 6, colors::AY_);
        crate::framebuffer::draw_text("Notes", 200, gc + 6, colors::AY_);
        crate::framebuffer::draw_text("Vol", 260, gc + 6, colors::AY_);
        crate::framebuffer::draw_text("Pan", 310, gc + 6, colors::AY_);
        crate::framebuffer::draw_text("M S", 360, gc + 6, colors::AY_);
        crate::framebuffer::draw_text("Meter", 400, gc + 6, colors::AY_);

        
        for (i, track) in oa.tracks.iter().enumerate() {
            let mf = gc + JY_ + (i as u32 * JY_);
            if mf + JY_ > gc + RC_ {
                break; 
            }

            
            let bg = if i == self.selected_track {
                colors::DCC_
            } else {
                colors::BJS_
            };
            crate::framebuffer::fill_rect(0, mf, fb_w, JY_, bg);

            
            crate::framebuffer::fill_rect(0, mf, 4, JY_, track.color);

            
            let rw = format!("{}", i);
            crate::framebuffer::draw_text(&rw, 8, mf + 6, colors::AB_);

            
            crate::framebuffer::draw_text(track.name_str(), 30, mf + 6, colors::AB_);

            
            crate::framebuffer::draw_text(track.waveform.short_name(), 140, mf + 6, colors::O_);

            
            let nli = format!("{}", track.notes.len());
            crate::framebuffer::draw_text(&nli, 200, mf + 6, colors::O_);

            
            if let Some(ch) = mixer.channels.get(i) {
                let edy = format!("{}", ch.volume);
                crate::framebuffer::draw_text(&edy, 260, mf + 6, colors::AB_);

                
                let glz = if ch.pan == 0 { String::from("C") }
                    else if ch.pan > 0 { format!("R{}", ch.pan) }
                    else { format!("L{}", -ch.pan) };
                crate::framebuffer::draw_text(&glz, 310, mf + 6, colors::O_);

                
                if ch.muted {
                    crate::framebuffer::draw_text("M", 360, mf + 6, colors::CKQ_);
                }

                
                if ch.solo {
                    crate::framebuffer::draw_text("S", 376, mf + 6, colors::CXH_);
                }

                
                let gho: u32 = 400;
                let inn: u32 = (fb_w - gho).saturating_sub(20).min(200);
                let oz = (inn as u32 * ch.volume as u32) / 255;
                crate::framebuffer::fill_rect(gho, mf + 8, inn, 12, colors::WM_);
                if oz > 0 {
                    let nfd = if ch.volume > 230 { colors::AHE_ }
                        else if ch.volume > 180 { colors::AHF_ }
                        else { colors::AHD_ };
                    crate::framebuffer::fill_rect(gho, mf + 8, oz, 12, nfd);
                }
            }

            
            crate::framebuffer::mn(0, mf + JY_ - 1, fb_w, colors::AKY_);
        }

        
        crate::framebuffer::mn(0, gc + RC_ - 1, fb_w, colors::AKY_);
    }

    

    
    pub fn next_track(&mut self) {
        let project = super::Ce.lock();
        if let Some(oa) = project.as_ref() {
            if !oa.tracks.is_empty() {
                self.selected_track = (self.selected_track + 1) % oa.tracks.len();
                self.dirty = true;
            }
        }
    }

    
    pub fn prev_track(&mut self) {
        let project = super::Ce.lock();
        if let Some(oa) = project.as_ref() {
            if !oa.tracks.is_empty() {
                if self.selected_track == 0 {
                    self.selected_track = oa.tracks.len() - 1;
                } else {
                    self.selected_track -= 1;
                }
                self.dirty = true;
            }
        }
    }

    
    pub fn toggle_piano_roll(&mut self) {
        self.show_piano_roll = !self.show_piano_roll;
        self.dirty = true;
    }
}


pub fn mwy() -> Result<(), &'static str> {
    super::ensure_init()?;

    let mut ui = DawUI::new();
    ui.draw();

    crate::println!("TrustDAW GUI launched. Press [Esc] to return to shell.");

    
    loop {
        if let Some(scancode) = crate::keyboard::kr() {
            let adx = scancode & 0x80 != 0;
            if adx { continue; } 

            match scancode {
                0x01 => break, 
                0x39 => { 
                    if Gw.load(Ordering::Relaxed) {
                        super::stop();
                    } else {
                        let _ = super::play();
                    }
                    ui.dirty = true;
                }
                0x13 => { 
                    if Fb.load(Ordering::Relaxed) {
                        Fb.store(false, Ordering::Relaxed);
                    } else {
                        let _ = super::recorder::iyu(ui.selected_track);
                    }
                    ui.dirty = true;
                }
                0x0F => { 
                    ui.next_track();
                }
                0x3F => { 
                    ui.toggle_piano_roll();
                }
                0x42 => { 
                    let _ = super::dpb("/home/output.wav");
                    crate::println!("Exported to /home/output.wav");
                }
                0x0C => { 
                    let bpm = Df.load(Ordering::Relaxed);
                    super::guf(bpm.saturating_sub(5));
                    ui.dirty = true;
                }
                0x0D => { 
                    let bpm = Df.load(Ordering::Relaxed);
                    super::guf(bpm + 5);
                    ui.dirty = true;
                }
                
                0x48 => { ui.piano_roll.cursor_up(); ui.dirty = true; }    
                0x50 => { ui.piano_roll.cursor_down(); ui.dirty = true; }  
                0x4B => { ui.piano_roll.cursor_left(); ui.dirty = true; }  
                0x4D => { ui.piano_roll.cursor_right(); ui.dirty = true; } 
                _ => {}
            }

            if ui.dirty {
                ui.draw();
            }
        }

        
        for _ in 0..5000 {
            core::hint::spin_loop();
        }
    }

    Ok(())
}
