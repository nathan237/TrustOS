




















use alloc::vec::Vec;
use alloc::vec;
use alloc::format;
use alloc::string::String;
use core::sync::atomic::Ordering;

use crate::audio::synth::{Waveform, Envelope, SynthEngine, BT_, Bq};





mod colors {
    
    pub const DW_: u32        = 0x0A0A14;
    pub const SO_: u32        = 0x0F0F1E;
    pub const CJ_: u32       = 0x141428;
    pub const DX_: u32      = 0x1A1A30;
    pub const ANL_: u32    = 0x1E1E38;

    
    pub const Bp: u32         = 0x2A2A4A;
    pub const AAY_: u32  = 0x4A4A6A;
    pub const Rp: u32        = 0x222240;

    
    pub const AKP_: u32    = 0xEEEEFF;
    pub const AB_: u32   = 0xCCCCDD;
    pub const O_: u32 = 0x8888AA;
    pub const AY_: u32       = 0x555577;
    pub const QV_: u32    = 0x66BBFF;

    
    pub const PW_: u32     = 0x44DD66;
    pub const AKE_: u32      = 0x666688;
    pub const XW_: u32        = 0xFF3344;
    pub const ZA_: u32   = 0x121228;

    
    pub const CYD_: u32       = 0x1A1A30;
    pub const EKO_: u32        = 0xFF6622;
    pub const EKP_: u32    = 0xFF8844;
    pub const BIN_: u32    = 0x44FF88;
    pub const BIO_: u32  = 0x66FF66;
    pub const CYC_: u32    = 0x333355;
    pub const CYB_: u32  = 0x444466;

    
    pub const AHD_: u32    = 0x44CC44;
    pub const AHF_: u32   = 0xCCCC44;
    pub const AHE_: u32      = 0xCC4444;
    pub const WM_: u32       = 0x0D0D1A;
    pub const DPC_: u32     = 0xBBBBCC;
    pub const CKR_: u32    = 0xFF8800;
    pub const CXI_: u32    = 0xFFDD00;

    
    pub const AFN_: u32      = 0xDDDDEE;
    pub const AFM_: u32      = 0x222233;
    pub const BAC_: u32    = 0xFF6622;
    pub const CGK_: u32      = 0x444455;

    
    pub const CUF_: u32     = 0x44DDFF;
    pub const CUE_: u32       = 0x0A0A18;
    pub const CXJ_: u32     = 0x22CC66;
    pub const CXK_: u32     = 0x66DD44;
    pub const CXL_: u32     = 0xCCCC22;
    pub const CXM_: u32     = 0xDD6622;
    pub const CXN_: u32     = 0xDD2222;

    
    pub const U_: [u32; 8] = [
        0xFF4444, 
        0xFFAA22, 
        0xFFDD44, 
        0x44DD66, 
        0x44AAFF, 
        0x8844FF, 
        0xFF44AA, 
        0x44DDDD, 
    ];
}






const HG_: usize = 8;

const GA_: usize = 32;

const BUD_: usize = 16;


#[derive(Clone, Copy)]
pub struct BeatStep {
    
    pub active: bool,
    
    pub velocity: u8,
    
    pub note_offset: i8,
}

impl BeatStep {
    pub fn off() -> Self {
        Self { active: false, velocity: 100, note_offset: 0 }
    }

    pub fn on(velocity: u8) -> Self {
        Self { active: true, velocity, note_offset: 0 }
    }

    pub fn ah(velocity: u8, offset: i8) -> Self {
        Self { active: true, velocity, note_offset: offset }
    }
}


pub struct BeatTrack {
    
    pub name: [u8; 16],
    pub name_len: usize,
    
    pub steps: [BeatStep; GA_],
    
    pub num_steps: usize,
    
    pub base_note: u8,
    
    pub waveform: Waveform,
    
    pub envelope: Envelope,
    
    pub volume: u8,
    
    pub pan: i8,
    
    pub muted: bool,
    
    pub solo: bool,
    
    pub color: u32,
    
    pub is_drum: bool,
}

impl BeatTrack {
    pub fn new(name: &str, base_note: u8, waveform: Waveform, color: u32, is_drum: bool) -> Self {
        let mut bhz = [0u8; 16];
        let bytes = name.as_bytes();
        let len = bytes.len().min(16);
        bhz[..len].copy_from_slice(&bytes[..len]);

        Self {
            name: bhz,
            name_len: len,
            steps: [BeatStep::off(); GA_],
            num_steps: BUD_,
            base_note,
            waveform,
            envelope: Envelope::dwp(),
            volume: 200,
            pan: 0,
            muted: false,
            solo: false,
            color,
            is_drum,
        }
    }

    pub fn name_str(&self) -> &str {
        core::str::from_utf8(&self.name[..self.name_len]).unwrap_or("???")
    }

    
    pub fn toggle_step(&mut self, step: usize) {
        if step < self.num_steps {
            self.steps[step].active = !self.steps[step].active;
        }
    }

    
    pub fn note_at(&self, step: usize) -> u8 {
        if step < self.num_steps && self.steps[step].active {
            let base = self.base_note as i16;
            let offset = self.steps[step].note_offset as i16;
            (base + offset).clamp(0, 127) as u8
        } else {
            0
        }
    }

    
    pub fn active_count(&self) -> usize {
        self.steps[..self.num_steps].iter().filter(|j| j.active).count()
    }
}






pub struct BeatStudio {
    
    pub tracks: [BeatTrack; HG_],
    
    pub bpm: u16,
    
    pub swing: u8,
    
    pub loop_bars: u8,

    
    pub playing: bool,
    pub recording: bool,
    pub current_step: usize,

    
    pub cursor_track: usize,
    pub cursor_step: usize,

    
    pub scope_buffer: [i16; 256],
    pub scope_pos: usize,
    pub spectrum: [u8; 16],

    
    pub keys_pressed: [bool; 128],
    pub octave: i8,
    pub velocity: u8,

    
    fb_w: u32,
    fb_h: u32,
}

impl BeatStudio {
    
    pub fn new() -> Self {
        let fb_w = crate::framebuffer::X_.load(Ordering::Relaxed) as u32;
        let fb_h = crate::framebuffer::W_.load(Ordering::Relaxed) as u32;

        let tracks = [
            BeatTrack::new("Kick",  36, Waveform::Sine,     colors::U_[0], true),
            BeatTrack::new("Snare", 38, Waveform::Noise,    colors::U_[1], true),
            BeatTrack::new("HiHat", 42, Waveform::Noise,    colors::U_[2], true),
            BeatTrack::new("Bass",  36, Waveform::Square,   colors::U_[3], false),
            BeatTrack::new("Lead",  60, Waveform::Sawtooth, colors::U_[4], false),
            BeatTrack::new("Pad",   60, Waveform::Triangle, colors::U_[5], false),
            BeatTrack::new("FX",    48, Waveform::Sawtooth, colors::U_[6], false),
            BeatTrack::new("Perc",  56, Waveform::Noise,    colors::U_[7], true),
        ];

        let mut ba = Self {
            tracks,
            bpm: 128,
            swing: 50,
            loop_bars: 1,
            playing: false,
            recording: false,
            current_step: 0,
            cursor_track: 0,
            cursor_step: 0,
            scope_buffer: [0i16; 256],
            scope_pos: 0,
            spectrum: [0u8; 16],
            keys_pressed: [false; 128],
            octave: 0,
            velocity: 100,
            fb_w,
            fb_h,
        };

        
        ba.load_demo_beat();

        
        ba.tracks[0].envelope = Envelope::new(2, 80, 0, 50);    
        ba.tracks[1].envelope = Envelope::new(1, 40, 0, 30);    
        ba.tracks[2].envelope = Envelope::new(1, 20, 0, 15);    
        ba.tracks[3].envelope = Envelope::dwp();                
        ba.tracks[4].envelope = Envelope::new(5, 100, 70, 80);  
        ba.tracks[5].envelope = Envelope::pad();                  
        ba.tracks[6].envelope = Envelope::new(1, 200, 0, 100);  
        ba.tracks[7].envelope = Envelope::new(1, 30, 0, 20);    

        ba
    }

    
    fn load_demo_beat(&mut self) {
        
        for i in [0, 4, 8, 12] {
            self.tracks[0].steps[i] = BeatStep::on(127);
        }

        
        for i in [4, 12] {
            self.tracks[1].steps[i] = BeatStep::on(110);
        }

        
        for i in [0, 2, 4, 6, 8, 10, 12, 14] {
            self.tracks[2].steps[i] = BeatStep::on(80);
        }
        
        for i in [1, 3, 5, 7, 9, 11, 13, 15] {
            self.tracks[2].steps[i] = BeatStep::on(40);
        }

        
        self.tracks[3].steps[0] = BeatStep::ah(120, 0);   
        self.tracks[3].steps[3] = BeatStep::ah(100, 0);   
        self.tracks[3].steps[6] = BeatStep::ah(110, 3);   
        self.tracks[3].steps[10] = BeatStep::ah(100, 5);  
        self.tracks[3].steps[13] = BeatStep::ah(90, 3);   

        
        self.tracks[4].steps[0] = BeatStep::ah(100, 0);   
        self.tracks[4].steps[2] = BeatStep::ah(90, 3);    
        self.tracks[4].steps[4] = BeatStep::ah(100, 7);   
        self.tracks[4].steps[8] = BeatStep::ah(110, 5);   
        self.tracks[4].steps[11] = BeatStep::ah(90, 3);   

        
        self.tracks[5].steps[0] = BeatStep::ah(70, 0);    

        
        self.tracks[6].steps[6] = BeatStep::on(60);
        self.tracks[6].steps[14] = BeatStep::on(50);

        
        self.tracks[7].steps[1] = BeatStep::on(80);
        self.tracks[7].steps[5] = BeatStep::on(90);
        self.tracks[7].steps[9] = BeatStep::on(80);
        self.tracks[7].steps[13] = BeatStep::on(70);
    }

    
    
    pub fn load_funky_house(&mut self) {
        
        self.bpm = 100;
        self.swing = 56; 

        
        for t in self.tracks.iter_mut() {
            t.num_steps = 32;
            for j in 0..GA_ {
                t.steps[j] = BeatStep::off();
            }
        }

        
        self.tracks[0] = BeatTrack::new("Kick",    36, Waveform::Sine,     colors::U_[0], true);
        self.tracks[1] = BeatTrack::new("Clap",    39, Waveform::Noise,    colors::U_[1], true);
        self.tracks[2] = BeatTrack::new("HiHat",   42, Waveform::Noise,    colors::U_[2], true);
        self.tracks[3] = BeatTrack::new("Sub Bass", 24, Waveform::Sine,    colors::U_[3], false);
        self.tracks[4] = BeatTrack::new("Mid Bass", 36, Waveform::Square,  colors::U_[4], false);
        self.tracks[5] = BeatTrack::new("Chords",  60, Waveform::Triangle, colors::U_[5], false);
        self.tracks[6] = BeatTrack::new("Lead",    72, Waveform::Sawtooth, colors::U_[6], false);
        self.tracks[7] = BeatTrack::new("Perc",    56, Waveform::Noise,    colors::U_[7], true);

        for t in self.tracks.iter_mut() {
            t.num_steps = 32;
        }

        
        self.tracks[0].envelope = Envelope::new(2, 200, 0, 80);    
        self.tracks[1].envelope = Envelope::new(1, 70, 0, 45);     
        self.tracks[2].envelope = Envelope::new(1, 22, 0, 12);     
        self.tracks[3].envelope = Envelope::new(8, 400, 85, 250);  
        self.tracks[4].envelope = Envelope::new(5, 150, 50, 100);  
        self.tracks[5].envelope = Envelope::pad();                   
        self.tracks[6].envelope = Envelope::new(10, 200, 70, 180); 
        self.tracks[7].envelope = Envelope::new(1, 30, 0, 18);     

        
        
        
        for i in (0..32).step_by(4) {
            self.tracks[0].steps[i] = BeatStep::on(127);
        }
        self.tracks[0].steps[3]  = BeatStep::on(45);  
        self.tracks[0].steps[15] = BeatStep::on(40);  
        self.tracks[0].steps[27] = BeatStep::on(45);  

        
        
        
        self.tracks[1].steps[4]  = BeatStep::on(120);
        self.tracks[1].steps[12] = BeatStep::on(120);
        self.tracks[1].steps[20] = BeatStep::on(120);
        self.tracks[1].steps[28] = BeatStep::on(120);
        self.tracks[1].steps[11] = BeatStep::on(50);  
        self.tracks[1].steps[27] = BeatStep::on(55);  

        
        
        
        for i in 0..16 {
            let anb = match i % 4 {
                0 => 85,
                2 => 105,  
                1 => 35,
                3 => 50,
                _ => 45,
            };
            self.tracks[2].steps[i] = BeatStep::on(anb);
        }
        for i in 16..32 {
            let anb = match i % 4 {
                0 => 80,
                2 => 110,  
                1 => 30,
                3 => 45,
                _ => 40,
            };
            self.tracks[2].steps[i] = BeatStep::on(anb);
        }
        self.tracks[2].steps[23] = BeatStep::off(); 
        self.tracks[2].steps[31] = BeatStep::off();

        
        
        
        
        
        
        self.tracks[3].steps[0]  = BeatStep::ah(127, 0);   
        self.tracks[3].steps[6]  = BeatStep::ah(100, 0);   
        self.tracks[3].steps[8]  = BeatStep::ah(120, 0);   
        self.tracks[3].steps[14] = BeatStep::ah(100, 0);   
        
        self.tracks[3].steps[16] = BeatStep::ah(127, 8);   
        self.tracks[3].steps[20] = BeatStep::ah(110, 8);   
        self.tracks[3].steps[24] = BeatStep::ah(127, 10);  
        self.tracks[3].steps[28] = BeatStep::ah(110, 10);  

        
        
        
        
        
        self.tracks[4].steps[0]  = BeatStep::ah(120, 0);   
        self.tracks[4].steps[3]  = BeatStep::ah(110, 0);   
        self.tracks[4].steps[5]  = BeatStep::ah(100, 3);   
        self.tracks[4].steps[7]  = BeatStep::ah(115, 7);   
        self.tracks[4].steps[10] = BeatStep::ah(105, 5);   
        self.tracks[4].steps[13] = BeatStep::ah(95, 3);    
        
        self.tracks[4].steps[16] = BeatStep::ah(120, 8);   
        self.tracks[4].steps[19] = BeatStep::ah(110, 7);   
        self.tracks[4].steps[21] = BeatStep::ah(100, 5);   
        self.tracks[4].steps[24] = BeatStep::ah(120, 10);  
        self.tracks[4].steps[26] = BeatStep::ah(105, 7);   
        self.tracks[4].steps[29] = BeatStep::ah(100, 5);   
        self.tracks[4].steps[31] = BeatStep::ah(90, 3);    

        
        
        
        
        self.tracks[5].steps[0]  = BeatStep::ah(80, 0);    
        self.tracks[5].steps[4]  = BeatStep::ah(70, 3);    
        self.tracks[5].steps[8]  = BeatStep::ah(75, 7);    
        self.tracks[5].steps[12] = BeatStep::ah(70, 3);    
        
        self.tracks[5].steps[16] = BeatStep::ah(80, 8);    
        self.tracks[5].steps[20] = BeatStep::ah(75, 7);    
        self.tracks[5].steps[24] = BeatStep::ah(80, 10);   
        self.tracks[5].steps[28] = BeatStep::ah(75, 7);    

        
        
        
        
        
        self.tracks[6].steps[0]  = BeatStep::ah(100, 7);   
        self.tracks[6].steps[2]  = BeatStep::ah(105, 10);  
        self.tracks[6].steps[3]  = BeatStep::ah(115, 12);  
        self.tracks[6].steps[5]  = BeatStep::ah(100, 10);  
        self.tracks[6].steps[7]  = BeatStep::ah(90, 7);    
        self.tracks[6].steps[8]  = BeatStep::ah(105, 5);   
        self.tracks[6].steps[10] = BeatStep::ah(100, 3);   
        self.tracks[6].steps[12] = BeatStep::ah(110, 5);   
        self.tracks[6].steps[14] = BeatStep::ah(105, 7);   
        
        self.tracks[6].steps[16] = BeatStep::ah(115, 12);  
        self.tracks[6].steps[17] = BeatStep::ah(100, 10);  
        self.tracks[6].steps[19] = BeatStep::ah(110, 12);  
        self.tracks[6].steps[20] = BeatStep::ah(120, 15);  
        self.tracks[6].steps[22] = BeatStep::ah(105, 12);  
        self.tracks[6].steps[24] = BeatStep::ah(100, 10);  
        self.tracks[6].steps[25] = BeatStep::ah(95, 7);    
        self.tracks[6].steps[27] = BeatStep::ah(90, 5);    
        self.tracks[6].steps[29] = BeatStep::ah(85, 3);    
        self.tracks[6].steps[31] = BeatStep::ah(80, 0);    

        
        
        
        for i in (1..32).step_by(2) {
            self.tracks[7].steps[i] = BeatStep::on(55);
        }
        for i in (2..32).step_by(4) {
            self.tracks[7].steps[i] = BeatStep::on(90);
        }
        
        self.tracks[7].steps[26] = BeatStep::on(90);
        self.tracks[7].steps[27] = BeatStep::on(100);
        self.tracks[7].steps[28] = BeatStep::on(105);
        self.tracks[7].steps[29] = BeatStep::on(110);
        self.tracks[7].steps[30] = BeatStep::on(120);
        self.tracks[7].steps[31] = BeatStep::on(127); 

        
        self.tracks[0].volume = 230; 
        self.tracks[1].volume = 185; 
        self.tracks[2].volume = 140; 
        self.tracks[3].volume = 255; 
        self.tracks[4].volume = 200; 
        self.tracks[5].volume = 110; 
        self.tracks[6].volume = 175; 
        self.tracks[7].volume = 120; 
    }

    
    
    
    
    

    
    fn anthem_init(&mut self) {
        self.bpm = 106;
        self.swing = 50; 

        self.tracks[0] = BeatTrack::new("Kick",  36, Waveform::Sine,     colors::U_[0], true);
        self.tracks[1] = BeatTrack::new("Snare", 38, Waveform::Noise,    colors::U_[1], true);
        self.tracks[2] = BeatTrack::new("HiHat", 42, Waveform::Noise,    colors::U_[2], true);
        self.tracks[3] = BeatTrack::new("Sub",   24, Waveform::Sine,     colors::U_[3], false);
        self.tracks[4] = BeatTrack::new("Bass",  36, Waveform::Square,   colors::U_[4], false);
        self.tracks[5] = BeatTrack::new("Pad",   60, Waveform::Triangle, colors::U_[5], false);
        self.tracks[6] = BeatTrack::new("Lead",  72, Waveform::Sawtooth, colors::U_[6], false);
        self.tracks[7] = BeatTrack::new("Arp",   72, Waveform::Triangle, colors::U_[7], false);

        for t in self.tracks.iter_mut() {
            t.num_steps = 32;
            for j in 0..GA_ { t.steps[j] = BeatStep::off(); }
        }

        
        self.tracks[0].envelope = Envelope::new(2, 200, 0, 80);
        self.tracks[1].envelope = Envelope::new(1, 65, 0, 40);
        self.tracks[2].envelope = Envelope::new(1, 18, 0, 8);
        self.tracks[3].envelope = Envelope::new(15, 600, 90, 350);
        self.tracks[4].envelope = Envelope::new(5, 200, 45, 130);
        self.tracks[5].envelope = Envelope::pad();
        self.tracks[6].envelope = Envelope::new(10, 280, 75, 220);
        self.tracks[7].envelope = Envelope::new(3, 100, 25, 70);

        self.tracks[0].volume = 200;
        self.tracks[1].volume = 175;
        self.tracks[2].volume = 130;
        self.tracks[3].volume = 240;
        self.tracks[4].volume = 190;
        self.tracks[5].volume = 100;
        self.tracks[6].volume = 180;
        self.tracks[7].volume = 140;
    }

    
    fn anthem_intro(&mut self) {
        self.anthem_init();
        
        self.tracks[0].muted = true;  
        self.tracks[1].muted = true;  
        self.tracks[2].muted = true;  
        self.tracks[4].muted = true;  
        self.tracks[6].muted = true;  

        
        self.tracks[3].volume = 180;
        self.tracks[3].steps[0]  = BeatStep::ah(70, 0);   
        self.tracks[3].steps[16] = BeatStep::ah(50, 0);   

        
        self.tracks[5].volume = 75;
        self.tracks[5].steps[0]  = BeatStep::ah(45, 0);   
        self.tracks[5].steps[8]  = BeatStep::ah(40, 3);   
        self.tracks[5].steps[16] = BeatStep::ah(45, 7);   
        self.tracks[5].steps[24] = BeatStep::ah(40, 3);   

        
        self.tracks[7] = BeatTrack::new("Texture", 72, Waveform::Noise, colors::U_[7], true);
        self.tracks[7].num_steps = 32;
        self.tracks[7].envelope = Envelope::new(1, 15, 0, 5);
        self.tracks[7].volume = 50;
        self.tracks[7].steps[4]  = BeatStep::on(25);
        self.tracks[7].steps[5]  = BeatStep::on(20);  
        self.tracks[7].steps[11] = BeatStep::on(30);
        self.tracks[7].steps[18] = BeatStep::on(18);
        self.tracks[7].steps[19] = BeatStep::on(25);
        self.tracks[7].steps[26] = BeatStep::on(28);
    }

    
    fn anthem_build(&mut self) {
        self.anthem_init();
        self.tracks[1].muted = true;  
        self.tracks[6].muted = true;  

        
        self.tracks[0].volume = 160;
        for i in (0..32).step_by(8) {
            self.tracks[0].steps[i] = BeatStep::on(80);
        }

        
        self.tracks[2].volume = 90;
        for i in (0..32).step_by(4) {
            self.tracks[2].steps[i] = BeatStep::on(30);
        }
        for i in (2..32).step_by(4) {
            self.tracks[2].steps[i] = BeatStep::on(55);
        }

        
        self.tracks[3].volume = 220;
        self.tracks[3].steps[0]  = BeatStep::ah(100, 0);  
        self.tracks[3].steps[8]  = BeatStep::ah(90, 0);   
        self.tracks[3].steps[16] = BeatStep::ah(100, 8);  
        self.tracks[3].steps[24] = BeatStep::ah(95, 10);  

        
        self.tracks[4].volume = 150;
        self.tracks[4].steps[0]  = BeatStep::ah(90, 0);   
        self.tracks[4].steps[4]  = BeatStep::ah(75, 0);   
        self.tracks[4].steps[8]  = BeatStep::ah(85, 3);   
        self.tracks[4].steps[12] = BeatStep::ah(80, 7);   
        self.tracks[4].steps[16] = BeatStep::ah(90, 8);   
        self.tracks[4].steps[20] = BeatStep::ah(80, 7);   
        self.tracks[4].steps[24] = BeatStep::ah(95, 10);  
        self.tracks[4].steps[28] = BeatStep::ah(85, 7);   

        
        self.tracks[5].volume = 90;
        self.tracks[5].steps[0]  = BeatStep::ah(55, 0);   
        self.tracks[5].steps[4]  = BeatStep::ah(50, 3);   
        self.tracks[5].steps[8]  = BeatStep::ah(55, 7);   
        self.tracks[5].steps[12] = BeatStep::ah(50, 3);   
        self.tracks[5].steps[16] = BeatStep::ah(55, 8);   
        self.tracks[5].steps[20] = BeatStep::ah(50, 7);   
        self.tracks[5].steps[24] = BeatStep::ah(55, 10);  
        self.tracks[5].steps[28] = BeatStep::ah(50, 7);   

        
        self.tracks[7].volume = 120;
        
        self.tracks[7].steps[0]  = BeatStep::ah(80, 0);   
        self.tracks[7].steps[2]  = BeatStep::ah(85, 3);   
        self.tracks[7].steps[4]  = BeatStep::ah(90, 7);   
        self.tracks[7].steps[6]  = BeatStep::ah(95, 12);  
        self.tracks[7].steps[8]  = BeatStep::ah(80, 0);   
        self.tracks[7].steps[10] = BeatStep::ah(85, 3);   
        self.tracks[7].steps[12] = BeatStep::ah(90, 7);   
        self.tracks[7].steps[14] = BeatStep::ah(100, 12); 
        
        self.tracks[7].steps[16] = BeatStep::ah(80, 8);   
        self.tracks[7].steps[18] = BeatStep::ah(85, 0);   
        self.tracks[7].steps[20] = BeatStep::ah(90, 3);   
        self.tracks[7].steps[22] = BeatStep::ah(85, 8);   
        self.tracks[7].steps[24] = BeatStep::ah(85, 10);  
        self.tracks[7].steps[26] = BeatStep::ah(90, 2);   
        self.tracks[7].steps[28] = BeatStep::ah(95, 5);   
        self.tracks[7].steps[30] = BeatStep::ah(100, 10); 
    }

    
    fn anthem_drop(&mut self) {
        self.anthem_init();
        

        
        self.tracks[0].volume = 225;
        for i in (0..32).step_by(4) {
            self.tracks[0].steps[i] = BeatStep::on(120);
        }
        self.tracks[0].steps[3]  = BeatStep::on(40);
        self.tracks[0].steps[15] = BeatStep::on(35);
        self.tracks[0].steps[19] = BeatStep::on(40);
        self.tracks[0].steps[27] = BeatStep::on(35);

        
        self.tracks[1].volume = 180;
        self.tracks[1].steps[4]  = BeatStep::on(115);
        self.tracks[1].steps[12] = BeatStep::on(115);
        self.tracks[1].steps[20] = BeatStep::on(115);
        self.tracks[1].steps[28] = BeatStep::on(115);
        self.tracks[1].steps[11] = BeatStep::on(45); 

        
        self.tracks[2].volume = 140;
        for i in 0..32 {
            let anb = match i % 4 { 0 => 80, 2 => 100, 1 => 35, _ => 50 };
            self.tracks[2].steps[i] = BeatStep::on(anb);
        }
        self.tracks[2].steps[15] = BeatStep::off();
        self.tracks[2].steps[31] = BeatStep::off();

        
        self.tracks[3].volume = 250;
        self.tracks[3].steps[0]  = BeatStep::ah(127, 0);
        self.tracks[3].steps[6]  = BeatStep::ah(100, 0);
        self.tracks[3].steps[8]  = BeatStep::ah(120, 0);
        self.tracks[3].steps[14] = BeatStep::ah(95, 0);
        self.tracks[3].steps[16] = BeatStep::ah(127, 8);  
        self.tracks[3].steps[20] = BeatStep::ah(105, 8);
        self.tracks[3].steps[24] = BeatStep::ah(127, 10); 
        self.tracks[3].steps[28] = BeatStep::ah(105, 10);

        
        self.tracks[4].volume = 200;
        self.tracks[4].steps[0]  = BeatStep::ah(115, 0);  
        self.tracks[4].steps[3]  = BeatStep::ah(105, 0);  
        self.tracks[4].steps[5]  = BeatStep::ah(95, 3);   
        self.tracks[4].steps[7]  = BeatStep::ah(110, 7);  
        self.tracks[4].steps[10] = BeatStep::ah(100, 5);  
        self.tracks[4].steps[13] = BeatStep::ah(90, 3);   
        self.tracks[4].steps[16] = BeatStep::ah(115, 8);  
        self.tracks[4].steps[19] = BeatStep::ah(105, 7);  
        self.tracks[4].steps[21] = BeatStep::ah(95, 5);   
        self.tracks[4].steps[24] = BeatStep::ah(115, 10); 
        self.tracks[4].steps[26] = BeatStep::ah(100, 7);  
        self.tracks[4].steps[29] = BeatStep::ah(95, 5);   
        self.tracks[4].steps[31] = BeatStep::ah(85, 3);   

        
        self.tracks[5].volume = 110;
        self.tracks[5].steps[0]  = BeatStep::ah(70, 0);   
        self.tracks[5].steps[4]  = BeatStep::ah(65, 3);   
        self.tracks[5].steps[8]  = BeatStep::ah(70, 7);   
        self.tracks[5].steps[12] = BeatStep::ah(65, 3);   
        self.tracks[5].steps[16] = BeatStep::ah(70, 8);   
        self.tracks[5].steps[20] = BeatStep::ah(65, 7);   
        self.tracks[5].steps[24] = BeatStep::ah(70, 10);  
        self.tracks[5].steps[28] = BeatStep::ah(65, 7);   

        
        self.tracks[6].volume = 190;
        
        self.tracks[6].steps[0]  = BeatStep::ah(100, 7);  
        self.tracks[6].steps[3]  = BeatStep::ah(110, 10); 
        self.tracks[6].steps[6]  = BeatStep::ah(120, 12); 
        self.tracks[6].steps[10] = BeatStep::ah(105, 10); 
        self.tracks[6].steps[12] = BeatStep::ah(100, 8);  
        self.tracks[6].steps[14] = BeatStep::ah(95, 7);   
        
        self.tracks[6].steps[16] = BeatStep::ah(105, 3);  
        self.tracks[6].steps[18] = BeatStep::ah(110, 7);  
        self.tracks[6].steps[20] = BeatStep::ah(120, 12); 
        self.tracks[6].steps[22] = BeatStep::ah(127, 15); 
        self.tracks[6].steps[24] = BeatStep::ah(110, 12); 
        self.tracks[6].steps[26] = BeatStep::ah(100, 10); 
        self.tracks[6].steps[28] = BeatStep::ah(90, 7);   
        self.tracks[6].steps[30] = BeatStep::ah(85, 0);   

        
        self.tracks[7].volume = 130;
        self.tracks[7].steps[1]  = BeatStep::ah(70, 12);  
        self.tracks[7].steps[5]  = BeatStep::ah(65, 7);   
        self.tracks[7].steps[9]  = BeatStep::ah(70, 12);  
        self.tracks[7].steps[13] = BeatStep::ah(65, 3);   
        self.tracks[7].steps[17] = BeatStep::ah(70, 8);   
        self.tracks[7].steps[21] = BeatStep::ah(75, 12);  
        self.tracks[7].steps[25] = BeatStep::ah(70, 10);  
        self.tracks[7].steps[29] = BeatStep::ah(65, 7);   
    }

    
    fn anthem_stable(&mut self) {
        self.anthem_init();

        
        self.tracks[0].volume = 210;
        for i in (0..32).step_by(8) {
            self.tracks[0].steps[i] = BeatStep::on(110);
        }

        
        self.tracks[1].volume = 165;
        self.tracks[1].steps[4]  = BeatStep::on(105);
        self.tracks[1].steps[12] = BeatStep::on(105);
        self.tracks[1].steps[20] = BeatStep::on(105);
        self.tracks[1].steps[28] = BeatStep::on(105);

        
        self.tracks[2].volume = 100;
        for i in (2..32).step_by(4) {
            self.tracks[2].steps[i] = BeatStep::on(60);
        }

        
        self.tracks[3].volume = 230;
        self.tracks[3].steps[0]  = BeatStep::ah(110, 0);  
        self.tracks[3].steps[8]  = BeatStep::ah(100, 0);
        self.tracks[3].steps[16] = BeatStep::ah(110, 8);  
        self.tracks[3].steps[24] = BeatStep::ah(105, 10); 

        
        self.tracks[4].volume = 180;
        self.tracks[4].steps[0]  = BeatStep::ah(100, 0);  
        self.tracks[4].steps[4]  = BeatStep::ah(85, 7);   
        self.tracks[4].steps[8]  = BeatStep::ah(95, 4);   
        self.tracks[4].steps[12] = BeatStep::ah(85, 0);   
        self.tracks[4].steps[16] = BeatStep::ah(100, 8);  
        self.tracks[4].steps[20] = BeatStep::ah(85, 7);   
        self.tracks[4].steps[24] = BeatStep::ah(100, 10); 
        self.tracks[4].steps[28] = BeatStep::ah(90, 5);   

        
        self.tracks[5].volume = 105;
        self.tracks[5].steps[0]  = BeatStep::ah(60, 0);   
        self.tracks[5].steps[8]  = BeatStep::ah(55, 4);   
        self.tracks[5].steps[16] = BeatStep::ah(60, 7);   
        self.tracks[5].steps[24] = BeatStep::ah(55, 12);  

        
        
        self.tracks[6].volume = 200;
        
        self.tracks[6].steps[0]  = BeatStep::ah(110, 0);  
        self.tracks[6].steps[4]  = BeatStep::ah(115, 4);  
        self.tracks[6].steps[8]  = BeatStep::ah(120, 7);  
        self.tracks[6].steps[12] = BeatStep::ah(127, 12); 
        
        self.tracks[6].steps[16] = BeatStep::ah(120, 12); 
        self.tracks[6].steps[20] = BeatStep::ah(115, 7);  
        self.tracks[6].steps[24] = BeatStep::ah(110, 4);  
        self.tracks[6].steps[28] = BeatStep::ah(105, 0);  

        
        self.tracks[7].volume = 115;
        self.tracks[7].steps[2]  = BeatStep::ah(75, 12);  
        self.tracks[7].steps[6]  = BeatStep::ah(70, 16);  
        self.tracks[7].steps[10] = BeatStep::ah(75, 19);  
        self.tracks[7].steps[14] = BeatStep::ah(80, 12);  
        self.tracks[7].steps[18] = BeatStep::ah(70, 19);  
        self.tracks[7].steps[22] = BeatStep::ah(75, 16);  
        self.tracks[7].steps[26] = BeatStep::ah(70, 12);  
        self.tracks[7].steps[30] = BeatStep::ah(65, 7);   
    }

    
    fn anthem_outro(&mut self) {
        self.anthem_init();
        
        self.tracks[0].muted = true;  
        self.tracks[1].muted = true;  
        self.tracks[2].muted = true;  
        self.tracks[4].muted = true;  
        self.tracks[7].muted = true;  

        
        self.tracks[3].volume = 150;
        self.tracks[3].steps[0]  = BeatStep::ah(60, 0);   
        self.tracks[3].steps[16] = BeatStep::ah(40, 0);   

        
        self.tracks[5].volume = 80;
        self.tracks[5].steps[0]  = BeatStep::ah(40, 0);   
        self.tracks[5].steps[8]  = BeatStep::ah(35, 7);   
        self.tracks[5].steps[16] = BeatStep::ah(40, 12);  
        self.tracks[5].steps[24] = BeatStep::ah(35, 7);   

        
        self.tracks[6].volume = 140;
        self.tracks[6].steps[0]  = BeatStep::ah(80, 0);   
        self.tracks[6].steps[4]  = BeatStep::ah(75, 4);   
        self.tracks[6].steps[8]  = BeatStep::ah(80, 7);   
        self.tracks[6].steps[12] = BeatStep::ah(85, 12);  
        
    }

    
    
    
    
    
    
    
    
    
    

    
    
    fn trap_base(&mut self) {
        self.bpm = 100;   
        self.swing = 50;

        
        
        self.tracks[0] = BeatTrack::new("Sub",     27, Waveform::Sine,     colors::U_[0], false);  
        self.tracks[1] = BeatTrack::new("Snare",   38, Waveform::Noise,    colors::U_[1], true);
        self.tracks[2] = BeatTrack::new("HiHat",   56, Waveform::Noise,    colors::U_[2], true);
        self.tracks[3] = BeatTrack::new("OpenHat", 53, Waveform::Noise,    colors::U_[3], true);
        self.tracks[4] = BeatTrack::new("Synth",   63, Waveform::Square,   colors::U_[4], false);  
        self.tracks[5] = BeatTrack::new("Pad",     51, Waveform::Sawtooth, colors::U_[5], false);  
        self.tracks[6] = BeatTrack::new("Lead",    75, Waveform::Sawtooth, colors::U_[6], false);  
        self.tracks[7] = BeatTrack::new("Perc",    63, Waveform::Noise,    colors::U_[7], true);

        for t in self.tracks.iter_mut() {
            t.num_steps = 32;
            for j in 0..GA_ { t.steps[j] = BeatStep::off(); }
            t.muted = false;
        }

        
        self.tracks[0].envelope = Envelope::new(1, 1800, 80, 600);  
        self.tracks[1].envelope = Envelope::new(1, 80, 0, 35);      
        self.tracks[2].envelope = Envelope::new(1, 16, 0, 6);       
        self.tracks[3].envelope = Envelope::new(1, 140, 0, 80);     
        self.tracks[4].envelope = Envelope::new(3, 380, 25, 260);   
        self.tracks[5].envelope = Envelope::pad();                   
        self.tracks[6].envelope = Envelope::new(4, 300, 50, 220);   
        self.tracks[7].envelope = Envelope::new(1, 25, 0, 8);       
    }

    
    
    
    
    
    pub fn load_trap_intro(&mut self) {
        self.trap_base();
        self.tracks[1].muted = true;   
        self.tracks[2].muted = true;   
        self.tracks[3].muted = true;   
        self.tracks[7].muted = true;   

        
        self.tracks[0].volume = 120;
        self.tracks[0].steps[0]  = BeatStep::ah(60, 0);     

        
        self.tracks[5].volume = 50;
        self.tracks[5].steps[0]  = BeatStep::ah(35, 0);     
        self.tracks[5].steps[8]  = BeatStep::ah(30, 5);     
        self.tracks[5].steps[16] = BeatStep::ah(35, 3);     
        self.tracks[5].steps[24] = BeatStep::ah(30, -2);    

        
        self.tracks[4].volume = 55;
        self.tracks[4].steps[4]  = BeatStep::ah(40, 0);     
        self.tracks[4].steps[12] = BeatStep::ah(35, -3);    
        self.tracks[4].steps[20] = BeatStep::ah(38, 5);     

        
        self.tracks[6].volume = 35;
        self.tracks[6].steps[16] = BeatStep::ah(30, 7);     
        self.tracks[6].steps[24] = BeatStep::ah(25, 0);     
    }

    
    
    
    
    
    
    pub fn load_trap_hook(&mut self) {
        self.trap_base();

        
        self.tracks[0].volume = 255;
        self.tracks[0].steps[0]  = BeatStep::ah(127, 0);    
        self.tracks[0].steps[6]  = BeatStep::ah(90, 0);     
        self.tracks[0].steps[8]  = BeatStep::ah(118, 0);    
        self.tracks[0].steps[16] = BeatStep::ah(125, 5);    
        self.tracks[0].steps[20] = BeatStep::ah(105, 3);    
        self.tracks[0].steps[24] = BeatStep::ah(120, -2);   
        self.tracks[0].steps[28] = BeatStep::ah(100, 0);    

        
        self.tracks[1].volume = 185;
        self.tracks[1].steps[8]  = BeatStep::on(120);
        self.tracks[1].steps[24] = BeatStep::on(118);
        self.tracks[1].steps[6]  = BeatStep::on(32);             
        self.tracks[1].steps[22] = BeatStep::on(30);             
        self.tracks[1].steps[15] = BeatStep::on(40);             

        
        
        self.tracks[2].volume = 100;
        
        self.tracks[2].steps[0]  = BeatStep::on(90);
        self.tracks[2].steps[1]  = BeatStep::on(38);
        self.tracks[2].steps[2]  = BeatStep::on(72);
        self.tracks[2].steps[3]  = BeatStep::on(35);
        self.tracks[2].steps[4]  = BeatStep::on(85);
        self.tracks[2].steps[5]  = BeatStep::on(32);
        self.tracks[2].steps[6]  = BeatStep::on(68);
        self.tracks[2].steps[7]  = BeatStep::on(30);
        self.tracks[2].steps[8]  = BeatStep::on(88);
        self.tracks[2].steps[9]  = BeatStep::on(36);
        self.tracks[2].steps[10] = BeatStep::on(70);
        self.tracks[2].steps[11] = BeatStep::on(33);
        self.tracks[2].steps[12] = BeatStep::on(82);
        self.tracks[2].steps[13] = BeatStep::on(35);
        self.tracks[2].steps[14] = BeatStep::on(75);
        self.tracks[2].steps[15] = BeatStep::on(40);
        
        self.tracks[2].steps[16] = BeatStep::on(90);
        self.tracks[2].steps[17] = BeatStep::on(38);
        self.tracks[2].steps[18] = BeatStep::on(72);
        self.tracks[2].steps[19] = BeatStep::on(35);
        self.tracks[2].steps[20] = BeatStep::on(85);
        self.tracks[2].steps[21] = BeatStep::on(32);
        self.tracks[2].steps[22] = BeatStep::on(68);
        self.tracks[2].steps[23] = BeatStep::on(30);
        
        self.tracks[2].steps[24] = BeatStep::on(42);
        self.tracks[2].steps[25] = BeatStep::on(52);
        self.tracks[2].steps[26] = BeatStep::on(62);
        self.tracks[2].steps[27] = BeatStep::on(72);
        self.tracks[2].steps[28] = BeatStep::on(82);
        self.tracks[2].steps[29] = BeatStep::on(92);
        self.tracks[2].steps[30] = BeatStep::on(102);
        self.tracks[2].steps[31] = BeatStep::on(115);

        
        self.tracks[3].volume = 70;
        self.tracks[3].steps[4]  = BeatStep::on(65);
        self.tracks[3].steps[12] = BeatStep::on(60);
        self.tracks[3].steps[20] = BeatStep::on(58);

        
        self.tracks[4].volume = 95;
        
        self.tracks[4].steps[0]  = BeatStep::ah(82, 0);     
        self.tracks[4].steps[2]  = BeatStep::ah(60, 3);     
        self.tracks[4].steps[4]  = BeatStep::ah(70, 7);     
        self.tracks[4].steps[6]  = BeatStep::ah(55, 12);    
        self.tracks[4].steps[8]  = BeatStep::ah(78, -4);    
        self.tracks[4].steps[10] = BeatStep::ah(58, 0);     
        self.tracks[4].steps[12] = BeatStep::ah(68, 3);     
        
        self.tracks[4].steps[16] = BeatStep::ah(80, 5);     
        self.tracks[4].steps[18] = BeatStep::ah(58, 9);     
        self.tracks[4].steps[20] = BeatStep::ah(72, 12);    
        self.tracks[4].steps[24] = BeatStep::ah(75, 3);     
        self.tracks[4].steps[26] = BeatStep::ah(55, 7);     
        self.tracks[4].steps[28] = BeatStep::ah(65, -2);    

        
        self.tracks[5].volume = 42;
        self.tracks[5].steps[0]  = BeatStep::ah(32, 0);     
        self.tracks[5].steps[8]  = BeatStep::ah(28, 5);     
        self.tracks[5].steps[16] = BeatStep::ah(32, 3);     
        self.tracks[5].steps[24] = BeatStep::ah(28, -2);    

        
        
        self.tracks[6].volume = 125;
        self.tracks[6].steps[0]  = BeatStep::ah(105, 7);    
        self.tracks[6].steps[2]  = BeatStep::ah(88, 3);     
        self.tracks[6].steps[4]  = BeatStep::ah(95, 7);     
        self.tracks[6].steps[8]  = BeatStep::ah(118, 12);   
        
        self.tracks[6].steps[14] = BeatStep::ah(80, 5);     
        
        self.tracks[6].steps[16] = BeatStep::ah(95, 5);     
        self.tracks[6].steps[19] = BeatStep::ah(85, 3);     
        self.tracks[6].steps[22] = BeatStep::ah(78, -2);    
        self.tracks[6].steps[26] = BeatStep::ah(72, 0);     

        
        self.tracks[7].volume = 55;
        self.tracks[7].steps[3]  = BeatStep::on(42);
        self.tracks[7].steps[7]  = BeatStep::on(38);
        self.tracks[7].steps[11] = BeatStep::on(48);
        self.tracks[7].steps[15] = BeatStep::on(35);
        self.tracks[7].steps[19] = BeatStep::on(45);
        self.tracks[7].steps[23] = BeatStep::on(40);
        self.tracks[7].steps[27] = BeatStep::on(50);
        self.tracks[7].steps[31] = BeatStep::on(55);
    }

    
    
    
    
    
    pub fn load_trap_verse(&mut self) {
        self.trap_base();

        
        self.tracks[0].volume = 160;
        self.tracks[0].steps[0]  = BeatStep::ah(85, 0);     
        self.tracks[0].steps[16] = BeatStep::ah(75, 0);     

        
        self.tracks[1].volume = 140;
        self.tracks[1].steps[8]  = BeatStep::on(90);
        self.tracks[1].steps[24] = BeatStep::on(85);

        
        self.tracks[2].volume = 72;
        self.tracks[2].steps[0]  = BeatStep::on(65);
        self.tracks[2].steps[4]  = BeatStep::on(55);
        self.tracks[2].steps[8]  = BeatStep::on(60);
        self.tracks[2].steps[12] = BeatStep::on(50);
        self.tracks[2].steps[16] = BeatStep::on(65);
        self.tracks[2].steps[20] = BeatStep::on(55);
        self.tracks[2].steps[24] = BeatStep::on(60);
        self.tracks[2].steps[28] = BeatStep::on(50);

        
        self.tracks[3].volume = 55;
        self.tracks[3].steps[14] = BeatStep::on(55);

        
        self.tracks[4].volume = 70;
        self.tracks[4].steps[4]  = BeatStep::ah(55, 0);     
        self.tracks[4].steps[12] = BeatStep::ah(48, -3);    
        self.tracks[4].steps[20] = BeatStep::ah(52, 5);     
        self.tracks[4].steps[28] = BeatStep::ah(45, 3);     

        
        self.tracks[5].volume = 55;
        self.tracks[5].steps[0]  = BeatStep::ah(32, 0);     
        self.tracks[5].steps[16] = BeatStep::ah(28, 7);     

        
        self.tracks[6].volume = 65;
        self.tracks[6].steps[0]  = BeatStep::ah(55, 7);     
        self.tracks[6].steps[12] = BeatStep::ah(48, 0);     

        
        self.tracks[7].volume = 35;
        self.tracks[7].steps[7]  = BeatStep::on(30);
        self.tracks[7].steps[23] = BeatStep::on(28);
    }

    
    
    
    
    
    pub fn load_trap_build(&mut self) {
        self.trap_base();

        
        self.tracks[0].volume = 235;
        self.tracks[0].steps[0]  = BeatStep::ah(118, 0);    
        self.tracks[0].steps[8]  = BeatStep::ah(105, 0);    
        self.tracks[0].steps[16] = BeatStep::ah(115, 5);    
        self.tracks[0].steps[20] = BeatStep::ah(95, 3);     
        self.tracks[0].steps[24] = BeatStep::ah(110, 0);    
        self.tracks[0].steps[28] = BeatStep::ah(88, -2);    

        
        self.tracks[1].volume = 175;
        self.tracks[1].steps[8]  = BeatStep::on(115);
        self.tracks[1].steps[24] = BeatStep::on(112);
        self.tracks[1].steps[4]  = BeatStep::on(28);
        self.tracks[1].steps[12] = BeatStep::on(35);
        self.tracks[1].steps[20] = BeatStep::on(30);

        
        self.tracks[2].volume = 90;
        
        self.tracks[2].steps[0]  = BeatStep::on(82);
        self.tracks[2].steps[2]  = BeatStep::on(40);
        self.tracks[2].steps[4]  = BeatStep::on(75);
        self.tracks[2].steps[6]  = BeatStep::on(38);
        self.tracks[2].steps[8]  = BeatStep::on(80);
        self.tracks[2].steps[10] = BeatStep::on(42);
        self.tracks[2].steps[12] = BeatStep::on(72);
        self.tracks[2].steps[14] = BeatStep::on(35);
        
        self.tracks[2].steps[16] = BeatStep::on(85);
        self.tracks[2].steps[17] = BeatStep::on(35);
        self.tracks[2].steps[18] = BeatStep::on(70);
        self.tracks[2].steps[19] = BeatStep::on(32);
        self.tracks[2].steps[20] = BeatStep::on(80);
        self.tracks[2].steps[21] = BeatStep::on(38);
        self.tracks[2].steps[22] = BeatStep::on(68);
        self.tracks[2].steps[23] = BeatStep::on(30);
        
        self.tracks[2].steps[24] = BeatStep::on(40);
        self.tracks[2].steps[25] = BeatStep::on(48);
        self.tracks[2].steps[26] = BeatStep::on(58);
        self.tracks[2].steps[27] = BeatStep::on(68);
        self.tracks[2].steps[28] = BeatStep::on(78);
        self.tracks[2].steps[29] = BeatStep::on(88);
        self.tracks[2].steps[30] = BeatStep::on(98);
        self.tracks[2].steps[31] = BeatStep::on(110);

        
        self.tracks[3].volume = 60;
        self.tracks[3].steps[6]  = BeatStep::on(60);
        self.tracks[3].steps[14] = BeatStep::on(55);
        self.tracks[3].steps[22] = BeatStep::on(58);

        
        self.tracks[4].volume = 85;
        self.tracks[4].steps[0]  = BeatStep::ah(72, 0);     
        self.tracks[4].steps[4]  = BeatStep::ah(58, 3);     
        self.tracks[4].steps[8]  = BeatStep::ah(68, 7);     
        self.tracks[4].steps[12] = BeatStep::ah(55, 5);     
        self.tracks[4].steps[16] = BeatStep::ah(75, 0);     
        self.tracks[4].steps[20] = BeatStep::ah(62, 3);     
        self.tracks[4].steps[24] = BeatStep::ah(70, -2);    
        self.tracks[4].steps[28] = BeatStep::ah(60, 0);     

        
        self.tracks[5].volume = 48;
        self.tracks[5].steps[0]  = BeatStep::ah(35, 0);     
        self.tracks[5].steps[8]  = BeatStep::ah(30, 5);     
        self.tracks[5].steps[16] = BeatStep::ah(35, 3);     
        self.tracks[5].steps[24] = BeatStep::ah(32, 7);     

        
        self.tracks[6].volume = 100;
        self.tracks[6].steps[0]  = BeatStep::ah(75, 0);     
        self.tracks[6].steps[4]  = BeatStep::ah(82, 3);     
        self.tracks[6].steps[8]  = BeatStep::ah(90, 7);     
        self.tracks[6].steps[16] = BeatStep::ah(80, 5);     
        self.tracks[6].steps[20] = BeatStep::ah(88, 7);     
        self.tracks[6].steps[24] = BeatStep::ah(95, 12);    

        
        self.tracks[7].volume = 48;
        self.tracks[7].steps[3]  = BeatStep::on(35);
        self.tracks[7].steps[7]  = BeatStep::on(32);
        self.tracks[7].steps[11] = BeatStep::on(40);
        self.tracks[7].steps[15] = BeatStep::on(30);
        self.tracks[7].steps[19] = BeatStep::on(38);
        self.tracks[7].steps[23] = BeatStep::on(35);
        self.tracks[7].steps[27] = BeatStep::on(42);
        self.tracks[7].steps[31] = BeatStep::on(48);
    }

    
    
    
    
    
    
    pub fn load_trap_bridge(&mut self) {
        self.trap_base();
        self.tracks[0].muted = true;   
        self.tracks[1].muted = true;   
        self.tracks[2].muted = true;   
        self.tracks[3].muted = true;   
        self.tracks[7].muted = true;   

        
        self.tracks[5].volume = 55;
        self.tracks[5].steps[0]  = BeatStep::ah(35, 0);     
        self.tracks[5].steps[8]  = BeatStep::ah(28, 7);     
        self.tracks[5].steps[16] = BeatStep::ah(32, -2);    
        self.tracks[5].steps[24] = BeatStep::ah(28, 0);     

        
        self.tracks[4].volume = 45;
        self.tracks[4].steps[8]  = BeatStep::ah(35, 0);     
        self.tracks[4].steps[20] = BeatStep::ah(30, -3);    

        
        self.tracks[6].volume = 30;
        self.tracks[6].steps[0]  = BeatStep::ah(28, 7);     
        self.tracks[6].steps[16] = BeatStep::ah(25, 0);     
    }

    
    
    
    
    
    pub fn load_trap_rebuild(&mut self) {
        self.trap_base();

        
        self.tracks[0].volume = 195;
        self.tracks[0].steps[0]  = BeatStep::ah(100, 0);    
        self.tracks[0].steps[8]  = BeatStep::ah(88, 0);     
        self.tracks[0].steps[16] = BeatStep::ah(95, 5);     
        self.tracks[0].steps[24] = BeatStep::ah(85, 0);     

        
        self.tracks[1].volume = 155;
        self.tracks[1].steps[8]  = BeatStep::on(105);
        self.tracks[1].steps[24] = BeatStep::on(100);
        self.tracks[1].steps[7]  = BeatStep::on(25);
        self.tracks[1].steps[23] = BeatStep::on(22);

        
        self.tracks[2].volume = 78;
        self.tracks[2].steps[0]  = BeatStep::on(72);
        self.tracks[2].steps[2]  = BeatStep::on(38);
        self.tracks[2].steps[4]  = BeatStep::on(65);
        self.tracks[2].steps[6]  = BeatStep::on(35);
        self.tracks[2].steps[8]  = BeatStep::on(70);
        self.tracks[2].steps[10] = BeatStep::on(36);
        self.tracks[2].steps[12] = BeatStep::on(62);
        self.tracks[2].steps[14] = BeatStep::on(33);
        self.tracks[2].steps[16] = BeatStep::on(72);
        self.tracks[2].steps[18] = BeatStep::on(38);
        self.tracks[2].steps[20] = BeatStep::on(65);
        self.tracks[2].steps[22] = BeatStep::on(35);
        self.tracks[2].steps[24] = BeatStep::on(70);
        self.tracks[2].steps[26] = BeatStep::on(40);
        self.tracks[2].steps[28] = BeatStep::on(65);
        self.tracks[2].steps[30] = BeatStep::on(50);

        
        self.tracks[3].volume = 55;
        self.tracks[3].steps[6]  = BeatStep::on(52);
        self.tracks[3].steps[22] = BeatStep::on(48);

        
        self.tracks[4].volume = 78;
        self.tracks[4].steps[0]  = BeatStep::ah(65, 0);     
        self.tracks[4].steps[4]  = BeatStep::ah(52, 3);     
        self.tracks[4].steps[8]  = BeatStep::ah(60, 7);     
        self.tracks[4].steps[16] = BeatStep::ah(68, 5);     
        self.tracks[4].steps[20] = BeatStep::ah(55, 0);     
        self.tracks[4].steps[24] = BeatStep::ah(62, 3);     

        
        self.tracks[5].volume = 45;
        self.tracks[5].steps[0]  = BeatStep::ah(30, 0);     
        self.tracks[5].steps[16] = BeatStep::ah(28, 5);     

        
        self.tracks[6].volume = 90;
        self.tracks[6].steps[0]  = BeatStep::ah(78, 7);     
        self.tracks[6].steps[4]  = BeatStep::ah(65, 3);     
        self.tracks[6].steps[12] = BeatStep::ah(72, 0);     
        self.tracks[6].steps[20] = BeatStep::ah(82, 7);     
        self.tracks[6].steps[24] = BeatStep::ah(70, 5);     

        
        self.tracks[7].volume = 40;
        self.tracks[7].steps[3]  = BeatStep::on(28);
        self.tracks[7].steps[11] = BeatStep::on(35);
        self.tracks[7].steps[19] = BeatStep::on(30);
        self.tracks[7].steps[27] = BeatStep::on(38);
    }

    
    
    
    
    
    pub fn load_trap_hook_final(&mut self) {
        self.load_trap_hook();

        
        self.tracks[0].volume = 255;     
        self.tracks[1].volume = 195;     
        self.tracks[2].volume = 115;     
        self.tracks[3].volume = 78;      
        self.tracks[4].volume = 110;     
        self.tracks[5].volume = 52;      
        self.tracks[6].volume = 145;     
        self.tracks[7].volume = 65;      

        
        self.tracks[1].steps[4]  = BeatStep::on(55);
        self.tracks[1].steps[12] = BeatStep::on(50);
        self.tracks[1].steps[20] = BeatStep::on(58);

        
        self.tracks[2].steps[1]  = BeatStep::on(52);
        self.tracks[2].steps[3]  = BeatStep::on(48);

        
        self.tracks[6].steps[8]  = BeatStep::ah(127, 12);   
        self.tracks[6].steps[12] = BeatStep::ah(115, 12);   
    }

    
    
    
    
    
    pub fn load_trap_outro(&mut self) {
        self.trap_base();
        self.tracks[1].muted = true;   
        self.tracks[2].muted = true;   
        self.tracks[3].muted = true;   
        self.tracks[7].muted = true;   

        
        self.tracks[0].volume = 130;
        self.tracks[0].steps[0]  = BeatStep::ah(55, 0);     

        
        self.tracks[5].volume = 35;
        self.tracks[5].steps[0]  = BeatStep::ah(22, 0);     
        self.tracks[5].steps[16] = BeatStep::ah(18, 7);     

        
        self.tracks[4].volume = 48;
        self.tracks[4].steps[4]  = BeatStep::ah(32, 0);     
        self.tracks[4].steps[14] = BeatStep::ah(25, -3);    
        self.tracks[4].steps[22] = BeatStep::ah(28, 0);     

        
        self.tracks[6].volume = 45;
        self.tracks[6].steps[0]  = BeatStep::ah(32, 7);     
        self.tracks[6].steps[10] = BeatStep::ah(28, 0);     
        self.tracks[6].steps[20] = BeatStep::ah(22, 0);     
    }

    
    pub fn qnv(&mut self) {
        self.load_trap_hook();
    }

    
    
    
    
    

    
    fn u2_base(&mut self) {
        self.bpm = 85;
        self.swing = 58; 

        self.tracks[0] = BeatTrack::new("Kick",   36, Waveform::Sine,     colors::U_[0], true);   
        self.tracks[1] = BeatTrack::new("Snare",  38, Waveform::Noise,    colors::U_[1], true);
        self.tracks[2] = BeatTrack::new("HiHat",  54, Waveform::Noise,    colors::U_[2], true);
        self.tracks[3] = BeatTrack::new("Sub",    33, Waveform::Sine,     colors::U_[3], false);  
        self.tracks[4] = BeatTrack::new("Keys",   69, Waveform::Triangle, colors::U_[4], false);  
        self.tracks[5] = BeatTrack::new("Pad",    57, Waveform::Sawtooth, colors::U_[5], false);  
        self.tracks[6] = BeatTrack::new("Lead",   81, Waveform::Square,   colors::U_[6], false);  
        self.tracks[7] = BeatTrack::new("Perc",   60, Waveform::Noise,    colors::U_[7], true);

        for t in self.tracks.iter_mut() {
            t.num_steps = 32;
            for j in 0..GA_ { t.steps[j] = BeatStep::off(); }
            t.muted = false;
        }

        
        self.tracks[0].envelope = Envelope::new(2, 120, 0, 80);     
        self.tracks[1].envelope = Envelope::new(1, 60, 0, 40);      
        self.tracks[2].envelope = Envelope::new(1, 22, 0, 10);      
        self.tracks[3].envelope = Envelope::new(2, 2000, 75, 800);  
        self.tracks[4].envelope = Envelope::new(8, 500, 40, 350);   
        self.tracks[5].envelope = Envelope::pad();                   
        self.tracks[6].envelope = Envelope::new(6, 400, 55, 280);   
        self.tracks[7].envelope = Envelope::new(1, 35, 0, 12);      
    }

    
    
    
    
    pub fn qnz(&mut self) {
        self.u2_base();
        self.tracks[0].muted = true;   
        self.tracks[1].muted = true;   
        self.tracks[2].muted = true;   
        self.tracks[7].muted = true;   

        
        self.tracks[3].volume = 80;
        self.tracks[3].steps[0]  = BeatStep::ah(50, 0);     

        
        self.tracks[5].volume = 55;
        self.tracks[5].steps[0]  = BeatStep::ah(30, 0);     
        self.tracks[5].steps[8]  = BeatStep::ah(25, 4);     
        self.tracks[5].steps[16] = BeatStep::ah(28, 7);     
        self.tracks[5].steps[24] = BeatStep::ah(25, 5);     

        
        self.tracks[4].volume = 40;
        self.tracks[4].steps[4]  = BeatStep::ah(32, 0);     
        self.tracks[4].steps[12] = BeatStep::ah(28, 4);     
        self.tracks[4].steps[20] = BeatStep::ah(30, 7);     
        self.tracks[4].steps[28] = BeatStep::ah(25, 10);    

        
        self.tracks[6].volume = 25;
        self.tracks[6].steps[16] = BeatStep::ah(22, 7);     
    }

    
    
    
    
    
    pub fn qob(&mut self) {
        self.u2_base();
        self.tracks[1].muted = true;    
        self.tracks[7].muted = true;    

        
        self.tracks[0].volume = 145;
        self.tracks[0].steps[0]  = BeatStep::on(100);
        self.tracks[0].steps[8]  = BeatStep::on(85);
        self.tracks[0].steps[16] = BeatStep::on(95);
        self.tracks[0].steps[24] = BeatStep::on(80);

        
        self.tracks[2].volume = 65;
        for i in (0..32).step_by(2) {
            self.tracks[2].steps[i] = BeatStep::on(60 + (i as u8 % 3) * 10);
        }
        
        self.tracks[2].steps[3]  = BeatStep::on(25);
        self.tracks[2].steps[11] = BeatStep::on(22);
        self.tracks[2].steps[19] = BeatStep::on(25);
        self.tracks[2].steps[27] = BeatStep::on(20);

        
        self.tracks[3].volume = 160;
        self.tracks[3].steps[0]  = BeatStep::ah(105, 0);    
        self.tracks[3].steps[8]  = BeatStep::ah(95, -4);    
        self.tracks[3].steps[16] = BeatStep::ah(100, -9);   
        self.tracks[3].steps[24] = BeatStep::ah(90, -2);    

        
        self.tracks[4].volume = 72;
        
        self.tracks[4].steps[0]  = BeatStep::ah(68, 0);     
        self.tracks[4].steps[2]  = BeatStep::ah(55, 4);     
        self.tracks[4].steps[4]  = BeatStep::ah(62, 7);     
        
        self.tracks[4].steps[8]  = BeatStep::ah(65, -4);    
        self.tracks[4].steps[10] = BeatStep::ah(52, 0);     
        self.tracks[4].steps[12] = BeatStep::ah(60, 4);     
        
        self.tracks[4].steps[16] = BeatStep::ah(65, -9);    
        self.tracks[4].steps[18] = BeatStep::ah(50, -5);    
        self.tracks[4].steps[20] = BeatStep::ah(58, -2);    
        
        self.tracks[4].steps[24] = BeatStep::ah(62, -2);    
        self.tracks[4].steps[26] = BeatStep::ah(48, 2);     
        self.tracks[4].steps[28] = BeatStep::ah(55, 5);     

        
        self.tracks[5].volume = 38;
        self.tracks[5].steps[0]  = BeatStep::ah(28, 0);     
        self.tracks[5].steps[16] = BeatStep::ah(26, -9);    

        
        self.tracks[6].volume = 50;
        self.tracks[6].steps[0]  = BeatStep::ah(55, 0);     
        self.tracks[6].steps[6]  = BeatStep::ah(48, 7);     
        self.tracks[6].steps[12] = BeatStep::ah(52, 5);     
        self.tracks[6].steps[18] = BeatStep::ah(50, 4);     
        self.tracks[6].steps[24] = BeatStep::ah(48, 2);     
        self.tracks[6].steps[30] = BeatStep::ah(42, 0);     
    }

    
    
    
    
    
    pub fn qnx(&mut self) {
        self.u2_base();

        
        self.tracks[0].volume = 195;
        self.tracks[0].steps[0]  = BeatStep::on(125);
        self.tracks[0].steps[6]  = BeatStep::on(40);  
        self.tracks[0].steps[8]  = BeatStep::on(120);
        self.tracks[0].steps[16] = BeatStep::on(125);
        self.tracks[0].steps[22] = BeatStep::on(35);  
        self.tracks[0].steps[24] = BeatStep::on(115);

        
        self.tracks[1].volume = 150;
        self.tracks[1].steps[8]  = BeatStep::on(115);
        self.tracks[1].steps[24] = BeatStep::on(110);
        self.tracks[1].steps[5]  = BeatStep::on(28);  
        self.tracks[1].steps[21] = BeatStep::on(25);  

        
        self.tracks[2].volume = 75;
        for i in 0..32 {
            let anb = if i % 4 == 0 { 80 } else if i % 2 == 0 { 55 } else { 30 };
            self.tracks[2].steps[i] = BeatStep::on(anb);
        }

        
        self.tracks[7].volume = 45;
        self.tracks[7].steps[4]  = BeatStep::on(55);
        self.tracks[7].steps[12] = BeatStep::on(50);
        self.tracks[7].steps[20] = BeatStep::on(52);
        self.tracks[7].steps[28] = BeatStep::on(48);

        
        self.tracks[3].volume = 220;
        self.tracks[3].steps[0]  = BeatStep::ah(120, 0);    
        self.tracks[3].steps[4]  = BeatStep::ah(80, 0);     
        self.tracks[3].steps[8]  = BeatStep::ah(115, 5);    
        self.tracks[3].steps[16] = BeatStep::ah(118, -4);   
        self.tracks[3].steps[20] = BeatStep::ah(75, -4);    
        self.tracks[3].steps[24] = BeatStep::ah(110, -5);   
        self.tracks[3].steps[28] = BeatStep::ah(90, -5);    

        
        self.tracks[4].volume = 90;
        
        self.tracks[4].steps[0]  = BeatStep::ah(75, 0);     
        self.tracks[4].steps[2]  = BeatStep::ah(60, 4);     
        self.tracks[4].steps[4]  = BeatStep::ah(68, 7);     
        self.tracks[4].steps[6]  = BeatStep::ah(50, 12);    
        
        self.tracks[4].steps[8]  = BeatStep::ah(72, 5);     
        self.tracks[4].steps[10] = BeatStep::ah(58, 8);     
        self.tracks[4].steps[12] = BeatStep::ah(65, 12);    
        
        self.tracks[4].steps[16] = BeatStep::ah(70, -4);    
        self.tracks[4].steps[18] = BeatStep::ah(55, 0);     
        self.tracks[4].steps[20] = BeatStep::ah(62, 4);     
        
        self.tracks[4].steps[24] = BeatStep::ah(68, -5);    
        self.tracks[4].steps[26] = BeatStep::ah(52, -1);    
        self.tracks[4].steps[28] = BeatStep::ah(60, 2);     
        self.tracks[4].steps[30] = BeatStep::ah(48, 5);     

        
        self.tracks[5].volume = 35;
        self.tracks[5].steps[0]  = BeatStep::ah(30, 0);     
        self.tracks[5].steps[16] = BeatStep::ah(28, -4);    

        
        self.tracks[6].volume = 95;
        self.tracks[6].steps[0]  = BeatStep::ah(85, 7);     
        self.tracks[6].steps[4]  = BeatStep::ah(72, 5);     
        self.tracks[6].steps[8]  = BeatStep::ah(90, 4);     
        self.tracks[6].steps[12] = BeatStep::ah(78, 2);     
        self.tracks[6].steps[16] = BeatStep::ah(95, 0);     
        self.tracks[6].steps[20] = BeatStep::ah(70, 4);     
        self.tracks[6].steps[24] = BeatStep::ah(100, 7);    
        self.tracks[6].steps[28] = BeatStep::ah(65, 5);     
    }

    
    
    
    
    pub fn qnw(&mut self) {
        self.u2_base();
        self.tracks[0].muted = true;   
        self.tracks[1].muted = true;   
        self.tracks[7].muted = true;   

        
        self.tracks[2].volume = 35;
        self.tracks[2].steps[0]  = BeatStep::on(40);
        self.tracks[2].steps[8]  = BeatStep::on(35);
        self.tracks[2].steps[16] = BeatStep::on(38);
        self.tracks[2].steps[24] = BeatStep::on(32);

        
        self.tracks[3].volume = 70;
        self.tracks[3].steps[0]  = BeatStep::ah(55, 0);     
        self.tracks[3].steps[16] = BeatStep::ah(50, 5);     

        
        self.tracks[4].volume = 60;
        self.tracks[4].steps[0]  = BeatStep::ah(55, 5);     
        self.tracks[4].steps[4]  = BeatStep::ah(45, 8);     
        self.tracks[4].steps[8]  = BeatStep::ah(52, 0);     
        self.tracks[4].steps[12] = BeatStep::ah(42, 4);     
        self.tracks[4].steps[16] = BeatStep::ah(50, -5);    
        self.tracks[4].steps[20] = BeatStep::ah(40, -1);    
        self.tracks[4].steps[24] = BeatStep::ah(48, 0);     
        self.tracks[4].steps[28] = BeatStep::ah(38, 7);     

        
        self.tracks[5].volume = 50;
        self.tracks[5].steps[0]  = BeatStep::ah(32, 5);     
        self.tracks[5].steps[16] = BeatStep::ah(30, 0);     

        
        self.tracks[6].volume = 42;
        self.tracks[6].steps[8]  = BeatStep::ah(50, 12);    
        self.tracks[6].steps[16] = BeatStep::ah(42, 7);     
        self.tracks[6].steps[24] = BeatStep::ah(38, 5);     
    }

    
    
    
    
    pub fn qny(&mut self) {
        self.u2_base();

        
        self.tracks[0].volume = 210;
        self.tracks[0].steps[0]  = BeatStep::on(127);
        self.tracks[0].steps[4]  = BeatStep::on(45);  
        self.tracks[0].steps[8]  = BeatStep::on(122);
        self.tracks[0].steps[12] = BeatStep::on(40);  
        self.tracks[0].steps[16] = BeatStep::on(127);
        self.tracks[0].steps[20] = BeatStep::on(42);
        self.tracks[0].steps[24] = BeatStep::on(120);
        self.tracks[0].steps[30] = BeatStep::on(90);  

        
        self.tracks[1].volume = 165;
        self.tracks[1].steps[8]  = BeatStep::on(120);
        self.tracks[1].steps[24] = BeatStep::on(118);
        self.tracks[1].steps[6]  = BeatStep::on(30);
        self.tracks[1].steps[22] = BeatStep::on(28);
        
        self.tracks[1].steps[28] = BeatStep::on(55);
        self.tracks[1].steps[29] = BeatStep::on(65);
        self.tracks[1].steps[30] = BeatStep::on(80);
        self.tracks[1].steps[31] = BeatStep::on(100);

        
        self.tracks[2].volume = 82;
        for i in 0..32 {
            let anb = if i % 4 == 0 { 85 } else if i % 2 == 0 { 60 } else { 35 };
            self.tracks[2].steps[i] = BeatStep::on(anb);
        }

        
        self.tracks[7].volume = 55;
        self.tracks[7].steps[2]  = BeatStep::on(45);
        self.tracks[7].steps[6]  = BeatStep::on(55);
        self.tracks[7].steps[14] = BeatStep::on(50);
        self.tracks[7].steps[18] = BeatStep::on(42);
        self.tracks[7].steps[26] = BeatStep::on(52);
        self.tracks[7].steps[30] = BeatStep::on(60);

        
        self.tracks[3].volume = 245;
        self.tracks[3].steps[0]  = BeatStep::ah(127, 0);    
        self.tracks[3].steps[4]  = BeatStep::ah(85, 0);     
        self.tracks[3].steps[8]  = BeatStep::ah(120, -4);   
        self.tracks[3].steps[16] = BeatStep::ah(125, -2);   
        self.tracks[3].steps[20] = BeatStep::ah(80, -2);    
        self.tracks[3].steps[24] = BeatStep::ah(118, -5);   
        self.tracks[3].steps[28] = BeatStep::ah(95, -5);    

        
        self.tracks[4].volume = 100;
        
        self.tracks[4].steps[0]  = BeatStep::ah(80, 0);     
        self.tracks[4].steps[1]  = BeatStep::ah(62, 4);     
        self.tracks[4].steps[2]  = BeatStep::ah(72, 7);     
        self.tracks[4].steps[4]  = BeatStep::ah(65, 12);    
        
        self.tracks[4].steps[8]  = BeatStep::ah(78, -4);    
        self.tracks[4].steps[9]  = BeatStep::ah(60, 0);     
        self.tracks[4].steps[10] = BeatStep::ah(70, 4);     
        self.tracks[4].steps[12] = BeatStep::ah(58, 8);     
        
        self.tracks[4].steps[16] = BeatStep::ah(75, -2);    
        self.tracks[4].steps[17] = BeatStep::ah(58, 2);     
        self.tracks[4].steps[18] = BeatStep::ah(68, 5);     
        self.tracks[4].steps[20] = BeatStep::ah(55, 10);    
        
        self.tracks[4].steps[24] = BeatStep::ah(72, -5);    
        self.tracks[4].steps[25] = BeatStep::ah(55, -1);    
        self.tracks[4].steps[26] = BeatStep::ah(65, 2);     
        self.tracks[4].steps[28] = BeatStep::ah(52, 5);     

        
        self.tracks[5].volume = 42;
        self.tracks[5].steps[0]  = BeatStep::ah(32, 0);     
        self.tracks[5].steps[8]  = BeatStep::ah(28, -4);    
        self.tracks[5].steps[16] = BeatStep::ah(30, -2);    
        self.tracks[5].steps[24] = BeatStep::ah(28, -5);    

        
        self.tracks[6].volume = 110;
        self.tracks[6].steps[0]  = BeatStep::ah(95, 12);    
        self.tracks[6].steps[3]  = BeatStep::ah(80, 7);     
        self.tracks[6].steps[6]  = BeatStep::ah(88, 5);     
        self.tracks[6].steps[8]  = BeatStep::ah(100, 4);    
        self.tracks[6].steps[12] = BeatStep::ah(92, 0);     
        self.tracks[6].steps[16] = BeatStep::ah(105, 7);    
        self.tracks[6].steps[18] = BeatStep::ah(85, 5);     
        self.tracks[6].steps[20] = BeatStep::ah(98, 4);     
        self.tracks[6].steps[24] = BeatStep::ah(110, 12);   
        self.tracks[6].steps[28] = BeatStep::ah(75, 7);     
    }

    
    
    
    
    pub fn qoa(&mut self) {
        self.u2_base();
        self.tracks[0].muted = true;   
        self.tracks[1].muted = true;   
        self.tracks[2].muted = true;   
        self.tracks[7].muted = true;   

        
        self.tracks[3].volume = 60;
        self.tracks[3].steps[0]  = BeatStep::ah(40, 0);     

        
        self.tracks[4].volume = 35;
        self.tracks[4].steps[0]  = BeatStep::ah(35, 0);     
        self.tracks[4].steps[8]  = BeatStep::ah(28, 4);     
        self.tracks[4].steps[20] = BeatStep::ah(30, 7);     

        
        self.tracks[5].volume = 30;
        self.tracks[5].steps[0]  = BeatStep::ah(22, 0);     
        self.tracks[5].steps[16] = BeatStep::ah(18, 7);     

        
        self.tracks[6].volume = 40;
        self.tracks[6].steps[0]  = BeatStep::ah(30, 7);     
        self.tracks[6].steps[8]  = BeatStep::ah(25, 5);     
        self.tracks[6].steps[16] = BeatStep::ah(22, 4);     
        self.tracks[6].steps[26] = BeatStep::ah(18, 0);     
    }

    
    
    

    fn transport_h(&self) -> u32 { 48 }

    fn track_label_w(&self) -> u32 { 120 }

    fn scope_w(&self) -> u32 { self.fb_w.saturating_sub(self.track_label_w() + self.grid_w()).max(120) }

    fn grid_w(&self) -> u32 {
        
        let step_w = ((self.fb_w - self.track_label_w() - 120) / self.tracks[0].num_steps as u32).max(20).min(44);
        step_w * self.tracks[0].num_steps as u32
    }

    fn step_w(&self) -> u32 {
        self.grid_w() / self.tracks[0].num_steps as u32
    }

    fn track_row_h(&self) -> u32 { 32 }

    fn grid_h(&self) -> u32 {
        
        24 + HG_ as u32 * self.track_row_h()
    }

    fn seq_y(&self) -> u32 { self.transport_h() }
    fn seq_grid_x(&self) -> u32 { self.track_label_w() }
    fn scope_x(&self) -> u32 { self.track_label_w() + self.grid_w() }

    fn bottom_y(&self) -> u32 { self.seq_y() + self.grid_h() + 2 }
    fn bottom_h(&self) -> u32 { self.fb_h.saturating_sub(self.bottom_y() + 48) }
    fn status_y(&self) -> u32 { self.fb_h.saturating_sub(48) }

    
    
    

    
    pub fn draw(&self) {
        if self.fb_w == 0 || self.fb_h == 0 { return; }

        
        crate::framebuffer::fill_rect(0, 0, self.fb_w, self.fb_h, colors::DW_);

        self.draw_transport();
        self.draw_track_labels();
        self.draw_step_grid();
        self.draw_scope();
        self.draw_bottom_panel();
        self.draw_status_bar();
    }

    

    fn draw_transport(&self) {
        let h = self.transport_h();
        crate::framebuffer::fill_rect(0, 0, self.fb_w, h, colors::ZA_);

        
        crate::framebuffer::draw_text("TrustDAW Beat Studio", 8, 4, colors::QV_);

        
        let bx = 220;
        
        let oxl = if !self.playing { colors::AKP_ } else { colors::AKE_ };
        crate::framebuffer::fill_rect(bx, 4, 14, 14, oxl);

        
        let gng = if self.playing { colors::PW_ } else { colors::AKE_ };
        crate::framebuffer::fill_rect(bx + 22, 4, 4, 14, gng);
        crate::framebuffer::fill_rect(bx + 26, 6, 3, 10, gng);
        crate::framebuffer::fill_rect(bx + 29, 8, 2, 6, gng);

        
        let odn = if self.recording { colors::XW_ } else { colors::AKE_ };
        crate::framebuffer::fill_circle(bx + 52, 11, 6, odn);

        
        let cuh = format!("BPM:{}", self.bpm);
        crate::framebuffer::draw_text(&cuh, bx + 80, 4, colors::AKP_);

        
        let bar = self.current_step / 16 + 1;
        let beat = (self.current_step % 16) / 4 + 1;
        let sub = self.current_step % 4 + 1;
        let bdb = format!("{}:{}.{}", bar, beat, sub);
        crate::framebuffer::draw_text(&bdb, bx + 160, 4, colors::PW_);

        
        let ozg = format!("Swing:{}%", self.swing);
        crate::framebuffer::draw_text(&ozg, bx + 240, 4, colors::O_);

        
        let oxg = format!("{} steps", self.tracks[0].num_steps);
        crate::framebuffer::draw_text(&oxg, bx + 340, 4, colors::O_);

        
        crate::framebuffer::draw_text("Key: C# minor", 8, 24, colors::AY_);

        let pmw = self.tracks[self.cursor_track].name_str();
        let ong = format!("Track: {} [{}]", pmw, self.cursor_track);
        crate::framebuffer::draw_text(&ong, 140, 24, colors::O_);

        let gkh = format!("Oct:{}", 4i8 + self.octave);
        crate::framebuffer::draw_text(&gkh, 340, 24, colors::O_);

        let hbh = format!("Vel:{}", self.velocity);
        crate::framebuffer::draw_text(&hbh, 420, 24, colors::O_);

        
        crate::framebuffer::mn(0, h - 1, self.fb_w, colors::AAY_);
    }

    

    fn draw_track_labels(&self) {
        let x = 0;
        let y = self.seq_y();
        let w = self.track_label_w();
        let ep = self.track_row_h();

        
        crate::framebuffer::fill_rect(x, y, w, 24, colors::DX_);
        crate::framebuffer::draw_text("TRACKS", 8, y + 4, colors::AY_);
        crate::framebuffer::mn(x, y + 23, w, colors::Bp);

        for i in 0..HG_ {
            let cm = y + 24 + i as u32 * ep;
            let hd = i == self.cursor_track;

            
            let bg = if hd { colors::ANL_ } else { colors::CJ_ };
            crate::framebuffer::fill_rect(x, cm, w, ep, bg);

            
            crate::framebuffer::fill_rect(x, cm, 4, ep, self.tracks[i].color);

            
            if hd {
                crate::framebuffer::draw_text(">", 6, cm + 8, colors::BIN_);
            }

            
            let name = self.tracks[i].name_str();
            let nhm = if self.tracks[i].muted { colors::AY_ }
                         else { colors::AB_ };
            crate::framebuffer::draw_text(name, 18, cm + 8, nhm);

            
            if self.tracks[i].muted {
                crate::framebuffer::draw_text("M", 82, cm + 8, colors::CKR_);
            }
            if self.tracks[i].solo {
                crate::framebuffer::draw_text("S", 96, cm + 8, colors::CXI_);
            }

            
            crate::framebuffer::mn(x, cm + ep - 1, w, colors::Rp);
        }

        
        crate::framebuffer::zv(w - 1, y, self.grid_h(), colors::Bp);
    }

    

    fn draw_step_grid(&self) {
        let hc = self.seq_grid_x();
        let jh = self.seq_y();
        let dy = self.step_w();
        let ep = self.track_row_h();
        let irt = self.tracks[0].num_steps;

        
        crate::framebuffer::fill_rect(hc, jh, self.grid_w(), 24, colors::DX_);
        for j in 0..irt {
            let am = hc + j as u32 * dy;
            let rw = format!("{}", j + 1);
            
            let color = if j % 4 == 0 { colors::AB_ } else { colors::AY_ };
            crate::framebuffer::draw_text(&rw, am + 2, jh + 4, color);

            
            if j % 4 == 0 && j > 0 {
                crate::framebuffer::zv(am, jh, self.grid_h(), colors::CYB_);
            }
        }
        crate::framebuffer::mn(hc, jh + 23, self.grid_w(), colors::Bp);

        
        for t in 0..HG_ {
            let cm = jh + 24 + t as u32 * ep;

            for j in 0..irt {
                let am = hc + j as u32 * dy;
                let step = &self.tracks[t].steps[j];

                
                let gls = am + 2;
                let glt = cm + 2;
                let glr = dy.saturating_sub(4).max(4);
                let glq = ep.saturating_sub(4).max(4);

                
                let msg = t == self.cursor_track && j == self.cursor_step;
                let iig = self.playing && j == self.current_step;

                let npe = if step.active {
                    if iig {
                        colors::BIO_  
                    } else {
                        
                        let brightness = step.velocity as u32 * 100 / 127;
                        fgh(self.tracks[t].color, brightness.max(40))
                    }
                } else if iig {
                    colors::ANL_  
                } else {
                    colors::CYD_
                };

                
                crate::framebuffer::fill_rect(gls, glt, glr, glq, npe);

                
                if msg {
                    crate::framebuffer::draw_rect(gls.saturating_sub(1), glt.saturating_sub(1),
                        glr + 2, glq + 2, colors::BIN_);
                }

                
                crate::framebuffer::draw_rect(gls, glt, glr, glq, colors::CYC_);
            }

            
            crate::framebuffer::mn(hc, cm + ep - 1, self.grid_w(), colors::Rp);
        }

        
        if self.playing {
            let p = hc + self.current_step as u32 * dy + dy / 2;
            crate::framebuffer::zv(p, jh, self.grid_h(), colors::BIO_);
        }
    }

    

    fn draw_scope(&self) {
        let am = self.scope_x();
        let ak = self.seq_y();
        let dy = self.scope_w();
        let dw = self.grid_h();

        crate::framebuffer::fill_rect(am, ak, dy, dw, colors::CUE_);
        crate::framebuffer::zv(am, ak, dw, colors::Bp);

        
        crate::framebuffer::fill_rect(am, ak, dy, 24, colors::DX_);
        crate::framebuffer::draw_text("SCOPE", am + 8, ak + 4, colors::AY_);
        crate::framebuffer::mn(am, ak + 23, dy, colors::Bp);

        
        let dyp = ak + 24;
        let dyo = dw / 2 - 24;
        let center_y = dyp + dyo / 2;

        
        crate::framebuffer::mn(am + 4, center_y, dy - 8, colors::Rp);

        
        let dnr = (dy - 8).min(256) as usize;
        for i in 1..dnr {
            let mnw = (self.scope_pos + i - 1) % 256;
            let mnx = (self.scope_pos + i) % 256;

            let y1 = center_y as i32 - (self.scope_buffer[mnw] as i32 * dyo as i32 / 2) / 32768;
            let y2 = center_y as i32 - (self.scope_buffer[mnx] as i32 * dyo as i32 / 2) / 32768;

            let bvc = (y1.max(dyp as i32) as u32).min(dyp + dyo);
            let bvd = (y2.max(dyp as i32) as u32).min(dyp + dyo);

            
            let inw = bvc.min(bvd);
            let aye = bvc.max(bvd);
            let bw = (aye - inw).max(1);
            crate::framebuffer::fill_rect(am + 4 + i as u32, inw, 1, bw, colors::CUF_);
        }

        
        let spec_y = ak + dw / 2;
        let oun = dw / 2;

        crate::framebuffer::mn(am, spec_y, dy, colors::Bp);
        crate::framebuffer::draw_text("SPECTRUM", am + 8, spec_y + 4, colors::AY_);

        let dii = spec_y + 20;
        let bxp = oun - 24;
        let irh = 16usize;
        let ek = ((dy - 16) / irh as u32).max(4);

        for i in 0..irh {
            let bx = am + 8 + i as u32 * ek;
            let level = self.spectrum[i].min(100) as u32;
            let fwv = bxp * level / 100;
            let gk = dii + bxp - fwv;

            
            let bxq = if level > 85 { colors::CXN_ }
                           else if level > 70 { colors::CXM_ }
                           else if level > 50 { colors::CXL_ }
                           else if level > 30 { colors::CXK_ }
                           else { colors::CXJ_ };

            
            crate::framebuffer::fill_rect(bx, dii, ek - 2, bxp, colors::WM_);

            
            if fwv > 0 {
                crate::framebuffer::fill_rect(bx, gk, ek - 2, fwv, bxq);
            }
        }
    }

    

    fn draw_bottom_panel(&self) {
        let dc = self.bottom_y();
        let ov = self.bottom_h();

        crate::framebuffer::mn(0, dc, self.fb_w, colors::AAY_);

        
        self.draw_mixer(0, dc + 1, self.track_label_w(), ov);
        self.draw_visual_keyboard(self.track_label_w(), dc + 1, self.grid_w(), ov);
        self.draw_info_panel(self.scope_x(), dc + 1, self.scope_w(), ov);
    }

    

    fn draw_mixer(&self, x: u32, y: u32, w: u32, h: u32) {
        crate::framebuffer::fill_rect(x, y, w, h, colors::CJ_);

        
        crate::framebuffer::draw_text("MIXER", x + 8, y + 4, colors::AY_);

        let lua = h.saturating_sub(40);
        let num_tracks = HG_;
        let hxt = ((w - 8) / num_tracks as u32).max(8);

        for i in 0..num_tracks {
            let dg = x + 4 + i as u32 * hxt;
            let hj = y + 24;

            
            let are = self.tracks[i].name_str().chars().next().unwrap_or('?');
            let mpu = format!("{}", are);
            crate::framebuffer::draw_text(&mpu, dg + 2, hj, self.tracks[i].color);

            
            let fwb = dg + 2;
            let fwc = hj + 18;
            let fwa = hxt.saturating_sub(6).max(3);
            let cxq = lua.saturating_sub(30);

            crate::framebuffer::fill_rect(fwb, fwc, fwa, cxq, colors::WM_);

            
            let level = self.tracks[i].volume as u32 * cxq / 255;
            if level > 0 {
                let mye = fwc + cxq - level;
                let myc = if self.tracks[i].muted { colors::AY_ }
                    else if level > cxq * 90 / 100 { colors::AHE_ }
                    else if level > cxq * 70 / 100 { colors::AHF_ }
                    else { colors::AHD_ };
                crate::framebuffer::fill_rect(fwb, mye, fwa, level, myc);
            }

            
            crate::framebuffer::draw_rect(fwb, fwc, fwa, cxq, colors::Bp);
        }

        
        crate::framebuffer::zv(x + w - 1, y, h, colors::Bp);
    }

    

    fn draw_visual_keyboard(&self, x: u32, y: u32, w: u32, h: u32) {
        crate::framebuffer::fill_rect(x, y, w, h, colors::SO_);

        
        crate::framebuffer::draw_text("KEYBOARD", x + 8, y + 4, colors::AY_);

        
        let clp = y + 22;
        let iiw = h.saturating_sub(26);
        let ffc = iiw;
        let hhx = iiw * 60 / 100;

        
        let gkf = 14u32;
        let bhl = (w - 16) / gkf;
        let iix = x + 8;

        let fii = (4 + self.octave) as u8;

        
        for i in 0..gkf {
            let esc = iix + i * bhl;

            
            let evm = i / 7;
            let daf = i % 7;
            let gts = match daf {
                0 => 0,  
                1 => 2,  
                2 => 4,  
                3 => 5,  
                4 => 7,  
                5 => 9,  
                6 => 11, 
                _ => 0,
            };
            let midi_note = (fii + evm as u8) * 12 + gts as u8;

            let cbd = midi_note < 128 && self.keys_pressed[midi_note as usize];
            let gem = if cbd { colors::BAC_ } else { colors::AFN_ };

            crate::framebuffer::fill_rect(esc, clp, bhl - 2, ffc, gem);
            crate::framebuffer::draw_rect(esc, clp, bhl - 2, ffc, colors::Bp);

            
            let nlb = ["C", "D", "E", "F", "G", "A", "B"];
            if daf < 7 {
                let label = nlb[daf as usize];
                let ace = if cbd { colors::AKP_ } else { colors::CGK_ };
                crate::framebuffer::draw_text(label, esc + bhl / 2 - 4, clp + ffc - 18, ace);
            }

            
            if daf == 0 {
                let gkh = format!("{}", fii + evm as u8);
                crate::framebuffer::draw_text(&gkh, esc + 2, clp + 2, colors::AY_);
            }
        }

        
        for i in 0..gkf {
            let evm = i / 7;
            let daf = i % 7;

            
            let gts = match daf {
                0 => Some(1),  
                1 => Some(3),  
                
                3 => Some(6),  
                4 => Some(8),  
                5 => Some(10), 
                _ => None,
            };

            if let Some(dee) = gts {
                let midi_note = (fii + evm as u8) * 12 + dee as u8;
                let cbd = midi_note < 128 && self.keys_pressed[midi_note as usize];

                let bx = iix + i * bhl + bhl * 2 / 3;
                let fv = bhl * 2 / 3;
                let gem = if cbd { colors::BAC_ } else { colors::AFM_ };

                crate::framebuffer::fill_rect(bx, clp, fv, hhx, gem);
                crate::framebuffer::draw_rect(bx, clp, fv, hhx, colors::Bp);
            }
        }

        
        let clw = clp + ffc + 2;
        if clw + 16 < y + h {
            crate::framebuffer::draw_text("[Z X C V B N M] Low  [Q W E R T Y U] High", x + 8, clw, colors::AY_);
        }
    }

    

    fn draw_info_panel(&self, x: u32, y: u32, w: u32, h: u32) {
        crate::framebuffer::fill_rect(x, y, w, h, colors::CJ_);
        crate::framebuffer::zv(x, y, h, colors::Bp);

        crate::framebuffer::draw_text("INFO", x + 8, y + 4, colors::AY_);

        let mut ly = y + 24;
        let bw = 18u32;

        
        let t = &self.tracks[self.cursor_track];
        let name_str = format!("Track: {}", t.name_str());
        crate::framebuffer::draw_text(&name_str, x + 8, ly, colors::AB_);
        ly += bw;

        let pue = format!("Wave: {}", t.waveform.name());
        crate::framebuffer::draw_text(&pue, x + 8, ly, colors::O_);
        ly += bw;

        let ws = if t.is_drum { "Type: Drum" } else { "Type: Melodic" };
        crate::framebuffer::draw_text(ws, x + 8, ly, colors::O_);
        ly += bw;

        let agu = crate::audio::tables::bno(t.base_note);
        let nlc = crate::audio::tables::bui(t.base_note);
        let nlf = format!("Note: {}{}", agu, nlc);
        crate::framebuffer::draw_text(&nlf, x + 8, ly, colors::O_);
        ly += bw;

        let avy = format!("Steps: {}/{}", t.active_count(), t.num_steps);
        crate::framebuffer::draw_text(&avy, x + 8, ly, colors::O_);
        ly += bw;

        let edy = format!("Vol: {}", t.volume);
        crate::framebuffer::draw_text(&edy, x + 8, ly, colors::O_);
        ly += bw;

        let glz = if t.pan == 0 { String::from("Pan: C") }
                     else if t.pan > 0 { format!("Pan: R{}", t.pan) }
                     else { format!("Pan: L{}", -t.pan) };
        crate::framebuffer::draw_text(&glz, x + 8, ly, colors::O_);
        ly += bw + 8;

        
        let lai = format!("Step: {}/{}", self.cursor_step + 1, t.num_steps);
        crate::framebuffer::draw_text(&lai, x + 8, ly, colors::QV_);
        ly += bw;

        
        let step = &t.steps[self.cursor_step];
        if step.active {
            let hbh = format!("Hit Vel: {}", step.velocity);
            crate::framebuffer::draw_text(&hbh, x + 8, ly, colors::AB_);
        } else {
            crate::framebuffer::draw_text("Hit: ---", x + 8, ly, colors::AY_);
        }
    }

    

    fn draw_status_bar(&self) {
        let ak = self.status_y();
        crate::framebuffer::fill_rect(0, ak, self.fb_w, 48, colors::ZA_);
        crate::framebuffer::mn(0, ak, self.fb_w, colors::AAY_);

        crate::framebuffer::draw_text(
            "[Space] Play/Stop  [Enter] Toggle Step  [R] Record  [Tab] Track  [+/-] BPM",
            8, ak + 6, colors::AY_
        );
        crate::framebuffer::draw_text(
            "[Arrows] Navigate  [Z-M] Low Piano  [Q-P] High Piano  [F8] Export  [Esc] Exit",
            8, ak + 24, colors::AY_
        );
    }

    
    
    

    
    
    
    pub fn render_loop(&self) -> Vec<i16> {
        let bpf = (60 * BT_) / (self.bpm as u32 * 4); 
        let ix = self.tracks[0].num_steps;
        let total_frames = bpf as usize * ix;
        let aai = total_frames * Bq as usize;

        let mut aif = vec![0i32; aai];

        
        let mut rng: u32 = 0xCAFE_B0BA;

        
        let jwj = self.tracks.iter().any(|t| t.solo);

        for (mp, t) in self.tracks.iter().enumerate() {
            if t.muted { continue; }
            if jwj && !t.solo { continue; }

            let mut engine = SynthEngine::new();
            engine.set_waveform(t.waveform);
            engine.envelope = t.envelope;

            let vd = t.volume as i32;

            
            let mut j = 0usize;
            while j < ix {
                let step = &t.steps[j];
                if !step.active {
                    j += 1;
                    continue;
                }

                let aad = t.note_at(j);
                if aad == 0 { j += 1; continue; }

                
                let iqz = if t.is_drum {
                    1usize
                } else {
                    let mut blz = 1usize;
                    while j + blz < ix && !t.steps[j + blz].active {
                        blz += 1;
                    }
                    blz
                };

                
                let kag = j * bpf as usize;
                rng ^= rng << 13; rng ^= rng >> 17; rng ^= rng << 5;
                let muw = ((rng % 769) as i32 - 384) as isize; 
                let nle = (kag as isize + muw).max(0) as usize;

                let nky = bpf as usize * iqz;

                let anb = step.velocity;
                let nld = engine.render_note(aad, anb,
                    (nky as u32 * 1000) / BT_);

                for (ay, &sample) in nld.iter().enumerate() {
                    let idx = nle * Bq as usize + ay;
                    if idx < aif.len() {
                        aif[idx] += (sample as i32 * vd) / 255;
                    }
                }

                j += iqz;
            }
        }

        
        let dmr = (bpf as usize * 3) * Bq as usize;
        let hxy = 50i32;

        if dmr > 0 && dmr < aif.len() {
            for i in dmr..aif.len() {
                let frg = aif[i - dmr];
                aif[i] += (frg * hxy) / 100;
            }
            let gxz = dmr * 2;
            if gxz < aif.len() {
                for i in gxz..aif.len() {
                    let frg = aif[i - gxz];
                    aif[i] += (frg * hxy / 3) / 100;
                }
            }
        }

        
        let mut evd: u16 = 0xACE1;
        for sample in aif.iter_mut() {
            let bf = evd & 1;
            evd >>= 1;
            if bf == 1 { evd ^= 0xB400; }
            let aig = (evd as i16 as i32) / 180; 
            *sample += aig;
        }

        
        aif.iter().map(|&j| {
            let j = j.clamp(-48000, 48000);
            if j > 24000 {
                (24000 + (j - 24000) / 3) as i16
            } else if j < -24000 {
                (-24000 + (j + 24000) / 3) as i16
            } else {
                j as i16
            }
        }).collect()
    }

    
    pub fn step_duration_ms(&self) -> u32 {
        60_000 / (self.bpm as u32 * 4) 
    }

    
    pub fn update_scope(&mut self, jo: &[i16]) {
        
        let step = (jo.len() / 256).max(1);
        for i in 0..256 {
            let idx = (i * step).min(jo.len().saturating_sub(1));
            self.scope_buffer[i] = jo[idx];
        }
        self.scope_pos = 0;
    }

    
    pub fn update_spectrum(&mut self) {
        
        for i in 0..16 {
            let mut level: u32 = 0;
            for t in &self.tracks {
                if !t.muted && self.current_step < t.num_steps {
                    let step = &t.steps[self.current_step];
                    if step.active {
                        
                        let akx = (t.base_note as u32 / 8).min(15);
                        let byu = (akx as i32 - i as i32).unsigned_abs();
                        if byu < 4 {
                            level += step.velocity as u32 * (4 - byu) / 4;
                        }
                    }
                }
            }
            self.spectrum[i] = level.min(100) as u8;
        }
    }

    
    pub fn trigger_note(&mut self, midi_note: u8, velocity: u8) {
        if midi_note < 128 {
            self.keys_pressed[midi_note as usize] = true;
        }
        
        let aal = self.tracks[self.cursor_track].waveform;
        let _ = crate::audio::set_waveform(aal);
        let _ = crate::audio::ive(midi_note, velocity, 150);
    }

    
    pub fn release_note(&mut self, midi_note: u8) {
        if midi_note < 128 {
            self.keys_pressed[midi_note as usize] = false;
        }
    }
}






pub fn mwv() -> Result<(), &'static str> {
    
    crate::audio::init().ok(); 

    let mut ba = BeatStudio::new();
    ba.draw();

    crate::serial_println!("[BEAT_STUDIO] Launched — press Esc to exit");

    ijm(&mut ba)
}


pub fn mwx() -> Result<(), &'static str> {
    crate::audio::init().ok();

    let mut ba = BeatStudio::new();
    ba.load_funky_house();
    ba.draw();

    crate::serial_println!("[BEAT_STUDIO] Funky House loaded — press Esc to exit");

    ijm(&mut ba)
}


fn ijm(ba: &mut BeatStudio) -> Result<(), &'static str> {
    loop {
        if let Some(scancode) = crate::keyboard::kr() {
            let adx = scancode & 0x80 != 0;
            let nwy = scancode & 0x7F;

            
            if adx {
                if let Some(aad) = super::keyboard_midi::dyl(nwy) {
                    ba.release_note(aad);
                    ba.draw();
                }
                continue;
            }

            let mut iza = true;

            match scancode {
                
                0x01 => break,

                
                0x39 => {
                    if ba.playing {
                        ba.playing = false;
                        ba.current_step = 0;
                        let _ = crate::audio::stop();
                    } else {
                        ba.playing = true;
                        
                        let audio = ba.render_loop();
                        ba.update_scope(&audio);
                        let _ = crate::drivers::hda::bdu(&audio);
                        
                        jwg(ba);
                    }
                }

                
                0x1C => {
                    ba.tracks[ba.cursor_track].toggle_step(ba.cursor_step);
                }

                
                0x0F => {
                    ba.cursor_track = (ba.cursor_track + 1) % HG_;
                }

                
                0x4D => { 
                    let max = ba.tracks[ba.cursor_track].num_steps;
                    ba.cursor_step = (ba.cursor_step + 1) % max;
                }
                0x4B => { 
                    let max = ba.tracks[ba.cursor_track].num_steps;
                    if ba.cursor_step == 0 {
                        ba.cursor_step = max - 1;
                    } else {
                        ba.cursor_step -= 1;
                    }
                }
                0x50 => { 
                    ba.cursor_track = (ba.cursor_track + 1) % HG_;
                }
                0x48 => { 
                    if ba.cursor_track == 0 {
                        ba.cursor_track = HG_ - 1;
                    } else {
                        ba.cursor_track -= 1;
                    }
                }

                
                0x0D => { 
                    ba.bpm = (ba.bpm + 5).min(300);
                }
                0x0C => { 
                    ba.bpm = ba.bpm.saturating_sub(5).max(40);
                }

                
                0x49 => { 
                    ba.octave = (ba.octave + 1).min(4);
                }
                0x51 => { 
                    ba.octave = (ba.octave - 1).max(-4);
                }

                
                0x32 => {
                    let wb = ba.cursor_track;
                    ba.tracks[wb].muted = !ba.tracks[wb].muted;
                }

                
                0x3B => {
                    let wb = ba.cursor_track;
                    ba.tracks[wb].waveform = match ba.tracks[wb].waveform {
                        Waveform::Sine => Waveform::Square,
                        Waveform::Square => Waveform::Sawtooth,
                        Waveform::Sawtooth => Waveform::Triangle,
                        Waveform::Triangle => Waveform::Noise,
                        Waveform::Noise => Waveform::Sine,
                    };
                }

                
                0x3C => {
                    for t in ba.tracks.iter_mut() {
                        t.num_steps = if t.num_steps == 16 { 32 } else { 16 };
                    }
                    if ba.cursor_step >= ba.tracks[0].num_steps {
                        ba.cursor_step = 0;
                    }
                }

                
                0x42 => {
                    let audio = ba.render_loop();
                    let _ = super::wav_export::dpb(
                        "/home/beat.wav", &audio, BT_, Bq as u16
                    );
                    crate::serial_println!("[BEAT_STUDIO] Exported to /home/beat.wav");
                }

                
                0x13 => {
                    ba.recording = !ba.recording;
                }

                
                0x0E => {
                    let wb = ba.cursor_track;
                    for j in 0..ba.tracks[wb].num_steps {
                        ba.tracks[wb].steps[j] = BeatStep::off();
                    }
                }

                
                _ => {
                    if let Some(aad) = super::keyboard_midi::dyl(scancode) {
                        ba.trigger_note(aad, ba.velocity);

                        
                        if ba.recording {
                            let wb = ba.cursor_track;
                            let cs = ba.cursor_step;
                            let base = ba.tracks[wb].base_note;
                            let offset = aad as i8 - base as i8;
                            ba.tracks[wb].steps[cs] = BeatStep::ah(ba.velocity, offset);
                            
                            let max = ba.tracks[wb].num_steps;
                            ba.cursor_step = (ba.cursor_step + 1) % max;
                        }
                    } else {
                        iza = false;
                    }
                }
            }

            if iza {
                ba.update_spectrum();
                ba.draw();
            }
        }

        
        for _ in 0..3000 {
            core::hint::spin_loop();
        }
    }

    
    let _ = crate::audio::stop();
    crate::serial_println!("[BEAT_STUDIO] Exited");
    Ok(())
}


fn jwg(ba: &mut BeatStudio) {
    let ix = ba.tracks[0].num_steps;
    let ait = ba.step_duration_ms();

    for j in 0..ix {
        ba.current_step = j;
        ba.update_spectrum();
        ba.draw();

        
        match bwv(ait as u64) {
            1 | 2 => {  
                ba.playing = false;
                ba.current_step = 0;
                let _ = crate::audio::stop();
                return;
            }
            _ => {}
        }
    }

    if ba.playing {
        
        ba.playing = false;
        ba.current_step = 0;
    }
}






fn fgh(color: u32, brightness: u32) -> u32 {
    let r = ((color >> 16) & 0xFF) * brightness / 100;
    let g = ((color >> 8) & 0xFF) * brightness / 100;
    let b = (color & 0xFF) * brightness / 100;
    (r.min(255) << 16) | (g.min(255) << 8) | b.min(255)
}













struct Abf {
    
    head_y: i32,
    
    speed: u8,
    
    trail_len: u8,
    
    active: bool,
    
    char_offset: u8,
    
    flash: u8,
}


struct MatrixState {
    columns: Vec<Abf>,
    num_cols: usize,
    num_rows: usize,
    fb_w: u32,
    fb_h: u32,
    
    frame: u32,
    
    lfsr: u32,
}

impl MatrixState {
    
    
    fn new() -> Self {
        let fb_w = crate::framebuffer::X_.load(Ordering::Relaxed) as u32;
        let fb_h = crate::framebuffer::W_.load(Ordering::Relaxed) as u32;

        const AD_: usize = 160;
        let num_rows = (fb_h / 16) as usize;

        let mut columns = Vec::with_capacity(AD_);
        let mut lfsr: u32 = 0xDEAD_BEEF;

        for i in 0..AD_ {
            
            let seed = (i as u32).wrapping_mul(2654435761) ^ 0xDEADBEEF;
            lfsr = gfj(lfsr);
            let speed = (seed % 3) as u8 + 1;        
            let wr = 30u8;                         
            let start_y = -((seed % (num_rows as u32 / 2)) as i32);
            let kio = (seed.wrapping_mul(7919) % 94) as u8;

            columns.push(Abf {
                head_y: start_y,
                speed,
                trail_len: wr,
                active: true,
                char_offset: kio,
                flash: 100,
            });
        }

        Self {
            columns,
            num_cols: AD_,
            num_rows,
            fb_w,
            fb_h,
            frame: 0,
            lfsr,
        }
    }

    
    fn tick(&mut self) {
        self.frame += 1;

        for (i, col) in self.columns.iter_mut().enumerate() {
            col.head_y += col.speed as i32;

            
            let pmh = col.trail_len as i32 * 16;
            if col.head_y > (self.fb_h as i32 + pmh) {
                let seed = (i as u32).wrapping_mul(1103515245).wrapping_add(self.frame);
                col.head_y = -((seed % (self.fb_h / 2)) as i32);
                col.speed = (seed % 3) as u8 + 1;
                col.char_offset = ((seed.wrapping_mul(7919)) % 94) as u8;
            }
        }
    }

    
    fn flash_beat(&mut self, intensity: u8) {
        
        let count = (self.num_cols * intensity as usize / 255).max(3);
        for _ in 0..count {
            self.lfsr = gfj(self.lfsr);
            let ow = (self.lfsr as usize) % self.num_cols;
            self.columns[ow].flash = 255;
            self.columns[ow].active = true;
            self.columns[ow].head_y = 0;
            self.lfsr = gfj(self.lfsr);
            self.columns[ow].speed = (self.lfsr % 3) as u8 + 3; 
        }
    }

    
    fn draw(&self, step: usize, ix: usize, track_info: &str, bpm: u16, bar_beat: &str) {
        
        crate::framebuffer::fill_rect(0, 0, self.fb_w, self.fb_h, 0x000000);

        
        const Gm: &[u8] = b"@#$%&*0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz!?<>{}[]|/\\~^";

        
        for (ow, col) in self.columns.iter().enumerate() {
            if !col.active { continue; }

            let x = ow as u32 * 8;

            for pq in 0..(col.trail_len as i32 + 1) {
                let row = col.head_y - pq;
                if row < 0 || row >= self.num_rows as i32 { continue; }

                let y = row as u32 * 16;

                
                let cuz = if pq == 0 {
                    
                    ((col.char_offset as u32 + self.frame * 3 + ow as u32) % Gm.len() as u32) as usize
                } else {
                    
                    ((col.char_offset as u32 + row as u32 * 7 + ow as u32 * 13) % Gm.len() as u32) as usize
                };
                let ch = Gm[cuz] as char;

                
                let brightness = if pq == 0 {
                    
                    255u32
                } else {
                    
                    let ln = 255u32.saturating_sub(pq as u32 * 255 / col.trail_len as u32);
                    ln.max(20)
                };

                
                let lwt = col.flash as u32;
                let dom = (brightness * lwt / 100).min(255);

                
                let r = if pq == 0 { dom * 80 / 100 } else { dom * 10 / 100 };
                let g = dom;
                let b = if pq == 0 { dom * 60 / 100 } else { dom * 20 / 100 };
                let color = ((r.min(255)) << 16) | ((g.min(255)) << 8) | b.min(255);

                crate::framebuffer::px(x, y, ch, color);
            }
        }

        
        let gk = self.fb_h - 32;
        let hs = 8;
        let ek = self.fb_w - 40;
        let pv = 20;

        
        crate::framebuffer::fill_rect(pv, gk, ek, hs, 0x002200);
        
        crate::framebuffer::draw_rect(pv, gk, ek, hs, 0x00AA00);

        
        if ix > 0 {
            let oz = ek * step as u32 / ix as u32;
            crate::framebuffer::fill_rect(pv + 1, gk + 1, oz, hs - 2, 0x00FF44);

            
            for i in 1..ix {
                if i % 4 == 0 {
                    let cg = pv + ek * i as u32 / ix as u32;
                    crate::framebuffer::zv(cg, gk, hs, 0x00CC00);
                }
            }
        }

        
        let title = "TRUSTDAW // BEAT MATRIX";
        let bea = title.len() as u32 * 8 + 16;
        let avk = (self.fb_w - bea) / 2;
        crate::framebuffer::fill_rect(avk, 8, bea, 24, 0x001100);
        crate::framebuffer::draw_rect(avk, 8, bea, 24, 0x00CC00);
        crate::framebuffer::draw_text(title, avk + 8, 12, 0x00FF66);

        
        let btj = 40;
        let eql = track_info.len() as u32 * 8 + 16;
        crate::framebuffer::fill_rect(8, btj, eql.min(self.fb_w - 16), 20, 0x000800);
        crate::framebuffer::draw_text(track_info, 16, btj + 2, 0x00AA44);

        
        let cuh = format!("{} BPM  {}", bpm, bar_beat);
        let hin = cuh.len() as u32 * 8 + 16;
        let hio = self.fb_w - hin - 8;
        crate::framebuffer::fill_rect(hio, btj, hin, 20, 0x000800);
        crate::framebuffer::draw_text(&cuh, hio + 8, btj + 2, 0x00CC66);

        
        let jiw = format!("{:02}/{:02}", step + 1, ix);
        let step_w = jiw.len() as u32 * 8 + 12;
        let avf = (self.fb_w - step_w) / 2;
        let bwa = gk - 24;
        crate::framebuffer::fill_rect(avf, bwa, step_w, 20, 0x001100);
        crate::framebuffer::draw_text(&jiw, avf + 6, bwa + 2, 0x44FF88);

        
        let czr = 70;
        let track_names = ["Ki", "Cl", "HH", "SB", "MB", "Ch", "Ld", "Pc"];
        for (i, name) in track_names.iter().enumerate() {
            let ty = czr + i as u32 * 20;
            let color = if i < 8 { colors::U_[i] } else { 0x00FF00 };
            crate::framebuffer::draw_text(name, 8, ty, color);
        }
    }

    
    
    fn draw_rain(&self) {
        crate::framebuffer::fill_rect(0, 0, self.fb_w, self.fb_h, 0xFF000000);

        const Gm: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ@#$%&*+=<>[]{}|";
        const DCD_: i32 = 30;
        const CK_: i32 = 16;

        let ati = self.fb_w / self.num_cols as u32;

        for (ow, col) in self.columns.iter().enumerate() {
            let x = (ow as u32 * ati) + ati / 2;
            let head_y = col.head_y;

            
            let hrk = ((col.speed as u32).saturating_sub(1)) * 50; 
            let cgm = 30 + hrk * 70 / 100; 
            let jcm = 20 + hrk * 80 / 100;      

            for i in 0..DCD_ {
                let yl = head_y - (i * CK_);
                if yl < 0 || yl >= self.fb_h as i32 { continue; }

                
                let base: u32 = if i == 0 { 255 }
                    else if i == 1 { 200 }
                    else { 160u32.saturating_sub(i as u32 * 7) };
                if base < 15 { continue; }

                let brightness = base * cgm / 100;

                
                let (r, g, b) = if i == 0 {
                    
                    let w = 140 * cgm / 100;
                    (w, brightness.max(w), w)
                } else {
                    
                    let mfz = (15 * (100 - jcm) / 100).min(40);
                    let kcw = (30 * (100 - jcm) / 100).min(50);
                    (mfz, brightness, kcw)
                };

                let color = ((r.min(255)) << 16) | ((g.min(255)) << 8) | b.min(255);

                
                let bfe = col.char_offset as u32
                    + (i as u32 * 7919)
                    ^ (self.frame / 12);
                let ch = Gm[(bfe as usize) % Gm.len()] as char;

                crate::framebuffer::px(x, yl as u32, ch, color);
            }
        }
    }
}


pub fn mwz() -> Result<(), &'static str> {
    crate::audio::init().ok();

    let mut ba = BeatStudio::new();
    ba.load_funky_house();

    let mut matrix = MatrixState::new();

    
    matrix.draw(0, ba.tracks[0].num_steps, "> INITIALIZING BEAT MATRIX...", ba.bpm, "1:1.1");

    
    let audio = ba.render_loop();
    ba.update_scope(&audio);

    let ix = ba.tracks[0].num_steps;
    let ait = ba.step_duration_ms();
    let rar = ait * ix as u32;

    crate::serial_println!("[MATRIX] Funky House: {} BPM, {} steps, {}ms per step", ba.bpm, ix, ait);

    
    for f in 0..30 {
        matrix.tick();
        let gdf = match f {
            0..=5   => "> LOADING BEAT DATA...",
            6..=12  => "> DECODING FREQUENCY MATRIX...",
            13..=20 => "> SYNTH ENGINES ONLINE...",
            _       => "> READY. ENTERING THE BEAT.",
        };
        matrix.draw(0, ix, gdf, ba.bpm, "---");

        if avs(100) { return Ok(()); } 
    }

    
    let nda = 4u32;

    
    let _ = crate::drivers::hda::bdu(&audio);

    'outer: for _loop_count in 0..nda {

        
        for j in 0..ix {
            ba.current_step = j;

            
            for t in 0..8 {
                if ba.tracks[t].steps[j].active && !ba.tracks[t].muted {
                    let anb = ba.tracks[t].steps[j].velocity;
                    matrix.flash_beat(anb);
                }
            }

            
            let mut avy = String::from("> ");
            for t in 0..8 {
                if ba.tracks[t].steps[j].active && !ba.tracks[t].muted {
                    avy.push_str(ba.tracks[t].name_str());
                    avy.push(' ');
                }
            }
            if avy.len() <= 2 {
                avy.push_str("...");
            }

            
            let bar = j / 16 + 1;
            let beat = (j % 16) / 4 + 1;
            let sub = j % 4 + 1;
            let bdb = format!("{}:{}.{}", bar, beat, sub);

            
            matrix.tick();
            matrix.draw(j, ix, &avy, ba.bpm, &bdb);

            
            match bwv(ait as u64) {
                1 | 2 => { break 'outer; } 
                _ => {}
            }
        }
    }

    
    let _ = crate::audio::stop();

    for f in 0..40 {
        matrix.tick();
        let glj = match f {
            0..=10  => "> DISCONNECTING...",
            11..=25 => "> SIGNAL LOST",
            _       => "> TRUSTDAW // SYSTEM OFFLINE",
        };
        matrix.draw(0, ix, glj, ba.bpm, "---");

        
        let dmm = matrix.num_cols / 40;
        for c in 0..dmm {
            let idx = (f as usize * dmm + c) % matrix.num_cols;
            matrix.columns[idx].active = false;
        }

        crate::cpu::tsc::ww(80); 
    }

    
    crate::framebuffer::fill_rect(0, 0, matrix.fb_w, matrix.fb_h, 0x000000);
    let hyr = "TRUSTDAW BEAT MATRIX // BUILT ON TRUSTOS";
    let fo = hyr.len() as u32 * 8;
    let dg = (matrix.fb_w - fo) / 2;
    let hj = matrix.fb_h / 2 - 8;
    crate::framebuffer::draw_text(hyr, dg, hj, 0x00FF44);

    let jjm = "Bare-metal. No OS. Pure Rust.";
    let dy = jjm.len() as u32 * 8;
    let am = (matrix.fb_w - dy) / 2;
    crate::framebuffer::draw_text(jjm, am, hj + 24, 0x008822);

    
    loop {
        if let Some(dr) = crate::keyboard::kr() {
            if dr & 0x80 == 0 { break; }
        }
        crate::cpu::tsc::ww(20);
    }

    crate::serial_println!("[MATRIX] Showcase complete");
    Ok(())
}




















struct Dc {
    title: &'static str,
    subtitle: &'static str,
    detail: &'static str,
    frames: u32,
}


fn byx(alc: &Dc, fb_w: u32, fb_h: u32, phase: &str, progress: u32, av: u32) {
    
    let apw = 200u32;
    let agf = fb_h.saturating_sub(apw + 52); 
    let ala = 16u32;
    let bkx = fb_w.saturating_sub(32);

    crate::framebuffer::co(ala, agf, bkx, apw, 0x000000, 230);

    
    crate::framebuffer::draw_rect(ala, agf, bkx, apw, 0x00EEFF);
    crate::framebuffer::draw_rect(ala + 1, agf + 1, bkx - 2, apw - 2, 0x00EEFF);

    
    let scale = 2u32;
    let bi = (ala + 16) as i32;

    
    crate::graphics::scaling::aat(bi, (agf + 12) as i32, phase, 0x00DDFF, scale);

    
    crate::graphics::scaling::aat(bi, (agf + 48) as i32, alc.title, 0xFFFFFF, scale);

    
    crate::graphics::scaling::aat(bi, (agf + 88) as i32, alc.subtitle, 0x55FF99, scale);

    
    crate::graphics::scaling::aat(bi, (agf + 128) as i32, alc.detail, 0xAADDFF, scale);

    
    let ccl = ala + 16;
    let ccm = agf + apw - 20;
    let cny = bkx - 32;
    let cck = 8u32;
    crate::framebuffer::fill_rect(ccl, ccm, cny, cck, 0x112233);
    if av > 0 {
        let oz = cny * progress / av;
        crate::framebuffer::fill_rect(ccl, ccm, oz, cck, 0x00EEFF);
    }
}


fn bgd(fb_w: u32, fb_h: u32, line1: &str, line2: &str, line3: &str, accent: u32) {
    crate::framebuffer::fill_rect(0, 0, fb_w, fb_h, 0x050510);

    let scale = 2u32;
    let ew = 8 * scale; 

    
    let ags = fb_h / 2;
    crate::framebuffer::fill_rect(0, ags - 80, fb_w, 2, accent);
    crate::framebuffer::fill_rect(0, ags + 80, fb_w, 2, accent);

    
    let ahg = line1.len() as u32 * ew;
    crate::graphics::scaling::aat(
        ((fb_w.saturating_sub(ahg)) / 2) as i32, (ags - 52) as i32,
        line1, 0xFFFFFF, scale);

    
    let aeo = line2.len() as u32 * ew;
    crate::graphics::scaling::aat(
        ((fb_w.saturating_sub(aeo)) / 2) as i32, (ags - 10) as i32,
        line2, accent, scale);

    
    let ane = line3.len() as u32 * ew;
    crate::graphics::scaling::aat(
        ((fb_w.saturating_sub(ane)) / 2) as i32, (ags + 36) as i32,
        line3, 0x99AABB, scale);
}



fn avs(total_ms: u64) -> bool {
    
    let df = 50u64; 
    let mut ck = total_ms;
    while ck > 0 {
        let delay = ck.min(df);
        crate::cpu::tsc::ww(delay);
        ck -= delay;
        
        while let Some(dr) = crate::keyboard::kr() {
            if dr & 0x80 != 0 { continue; }
            if dr == 0x01 { return true; } 
        }
    }
    false
}



fn bwv(total_ms: u64) -> u8 {
    let df = 50u64;
    let mut ck = total_ms;
    while ck > 0 {
        let delay = ck.min(df);
        crate::cpu::tsc::ww(delay);
        ck -= delay;
        while let Some(dr) = crate::keyboard::kr() {
            if dr & 0x80 != 0 { continue; }
            if dr == 0x01 { return 1; } 
            if dr == 0x39 { return 2; } 
        }
    }
    0
}






fn kwl(audio: &[i16], start: usize, end: usize) -> u32 {
    let j = start.min(audio.len());
    let e = end.min(audio.len());
    if e <= j { return 0; }
    let slice = &audio[j..e];
    let oyn: u64 = slice.iter().map(|v| v.unsigned_abs() as u64).sum();
    let ns = (oyn / slice.len().max(1) as u64) as u32;
    (ns * 100 / 8000).min(100)
}






fn lja(
    fb_w: u32, fb_h: u32,
    scope: &[i16; 256],
    energy: u32, 
    frame: u32,
) {
    let dgl = fb_w * 72 / 100;
    let beh = fb_h * 34 / 100;
    let dgm = (fb_w - dgl) / 2;
    let center_y = fb_h / 2;

    
    let kq = 35 + energy * 65 / 100;

    
    let fjr = (frame % 40) as u32;
    let kdv = if fjr < 20 { fjr } else { 40 - fjr }; 
    let kdw = 92 + kdv * 16 / 20; 

    let jvq = kq * kdw / 100;

    let gkc: usize = 256;
    let ffp = (dgl / gkc as u32).max(1);

    
    let mut ys = [0i32; 256];
    for i in 0..256 {
        let sample = scope[i] as i32;
        let hdc = sample * (beh as i32 / 2) * jvq as i32 / (32768 * 100);
        ys[i] = center_y as i32 - hdc;
    }

    
    for i in 0..gkc {
        let x = dgm + i as u32 * ffp;
        let y = ys[i];
        let u = center_y as i32;
        let (top, h) = if y < u {
            (y.max(0) as u32, (u - y).max(1) as u32)
        } else {
            (u.max(0) as u32, (y - u).max(1) as u32)
        };
        crate::framebuffer::co(x, top, ffp, h, 0x00DDCC, 18);
    }

    
    let glow: [(i32, u32, u32); 5] = [
        (14, 10,  0x6622FF), 
        (8,  20,  0x4444FF), 
        (4,  45,  0x00AAEE), 
        (2,  100, 0x00DDCC), 
        (1,  220, 0x44FFDD), 
    ];

    for &(kh, alpha, color) in &glow {
        for i in 0..gkc {
            let x = dgm + i as u32 * ffp;
            let y = ys[i];
            let top = (y - kh).max(0) as u32;
            let age = (y + kh).min(fb_h as i32) as u32;
            let h = age.saturating_sub(top).max(1);
            crate::framebuffer::co(x, top, ffp, h, color, alpha);
        }
    }

    
    crate::framebuffer::co(dgm, center_y.saturating_sub(1), dgl, 2, 0x00FFCC, 12);

    
    let hih = center_y.saturating_sub(beh / 2);
    let kcy = center_y + beh / 2;
    crate::framebuffer::co(dgm, hih, dgl, 1, 0x00FFCC, 10);
    crate::framebuffer::co(dgm, kcy, dgl, 1, 0x00FFCC, 10);

    
    let olg = hih + (frame % beh.max(1));
    crate::framebuffer::co(dgm, olg, dgl, 2, 0x00FFCC, 18);
}


fn lim(
    fb_w: u32, fb_h: u32,
    section_name: &str,
    sec_idx: usize,
    loop_num: u32, total_loops: u32,
    step: usize, ix: usize,
    bpm: u16,
) {
    let scale = 2u32;
    let ew = 8 * scale;

    
    let title = "NEON PROTOCOL";
    let gr = title.len() as u32 * ew;
    let bu = (fb_w.saturating_sub(gr)) / 2;
    crate::framebuffer::co(bu.saturating_sub(12), 10, gr + 24, 36, 0x000000, 160);
    crate::graphics::scaling::aat(bu as i32, 14, title, 0x00FFCC, scale);

    
    let jgv = section_name.len() as u32 * 8 + 16;
    let jgw = (fb_w.saturating_sub(jgv)) / 2;
    crate::framebuffer::co(jgw.saturating_sub(4), 50, jgv + 8, 20, 0x000000, 140);
    crate::framebuffer::draw_text(section_name, jgw, 52, 0xBB44FF);

    
    let him = format!("{} BPM", bpm);
    let fv = him.len() as u32 * 8 + 16;
    crate::framebuffer::co(6, 8, fv, 20, 0x000000, 140);
    crate::framebuffer::draw_text(&him, 14, 12, 0x00AA88);

    
    let jdx = format!("{}/8 L{}/{}", sec_idx + 1, loop_num + 1, total_loops);
    let dy = jdx.len() as u32 * 8 + 16;
    let am = fb_w.saturating_sub(dy + 8);
    crate::framebuffer::co(am, 8, dy, 20, 0x000000, 140);
    crate::framebuffer::draw_text(&jdx, am + 8, 12, 0x00AA88);

    
    let ccm = fb_h.saturating_sub(28);
    let cck = 4u32;
    let cny = fb_w.saturating_sub(60);
    let ccl = 30u32;
    
    crate::framebuffer::co(ccl.saturating_sub(6), ccm.saturating_sub(6), cny + 12, cck + 12, 0x00FFCC, 6);
    
    crate::framebuffer::fill_rect(ccl, ccm, cny, cck, 0x001111);
    if ix > 0 {
        let oz = cny * step as u32 / ix as u32;
        crate::framebuffer::fill_rect(ccl, ccm, oz, cck, 0x00FFCC);
        
        crate::framebuffer::co(ccl, ccm.saturating_sub(2), oz, cck + 4, 0x00FFCC, 30);
    }
    
    for i in 1..ix {
        if i % 8 == 0 {
            let cg = ccl + cny * i as u32 / ix as u32;
            crate::framebuffer::zv(cg, ccm, cck, 0x005555);
        }
    }

    
    let ija = "Eb minor";
    let li = ija.len() as u32 * 8 + 8;
    crate::framebuffer::co(fb_w.saturating_sub(li + 8), fb_h.saturating_sub(48), li, 16, 0x000000, 120);
    crate::framebuffer::draw_text(ija, fb_w.saturating_sub(li + 4), fb_h.saturating_sub(46), 0x665588);
}


pub fn mxb() -> Result<(), &'static str> {
    crate::audio::init().ok();
    crate::serial_println!("[SHOWCASE] Starting narrated showcase...");

    let fb_w = crate::framebuffer::X_.load(Ordering::Relaxed) as u32;
    let fb_h = crate::framebuffer::W_.load(Ordering::Relaxed) as u32;

    
    crate::framebuffer::adw();
    crate::framebuffer::pr(true);

    
    
    

    bgd(fb_w, fb_h,
        "T R U S T D A W",
        "Building a Funky House Track from Scratch",
        "Bare-Metal  //  No OS  //  Pure Rust  //  Real-Time Audio",
        0x00CCFF,
    );
    crate::framebuffer::ii();
    if avs(6500) { aco(); return Ok(()); }

    bgd(fb_w, fb_h,
        "PHASE 1: BUILDING THE BEAT",
        "Watch each layer come to life, one track at a time",
        "100 BPM  //  C Minor  //  32 Steps (2 Bars)  //  Echo FX",
        0x44FF88,
    );
    crate::framebuffer::ii();
    if avs(5500) { aco(); return Ok(()); }

    
    
    
    crate::serial_println!("[SHOWCASE] Phase 1: Building the beat");

    
    let mut cpf = BeatStudio::new();
    cpf.load_funky_house();

    
    let mut ba = BeatStudio::new();
    ba.bpm = cpf.bpm;
    ba.swing = cpf.swing;
    
    for t in ba.tracks.iter_mut() {
        t.num_steps = 32;
        for j in 0..GA_ {
            t.steps[j] = BeatStep::off();
        }
    }
    ba.tracks[0] = BeatTrack::new("Kick",     36, Waveform::Sine,     colors::U_[0], true);
    ba.tracks[1] = BeatTrack::new("Clap",     39, Waveform::Noise,    colors::U_[1], true);
    ba.tracks[2] = BeatTrack::new("HiHat",    42, Waveform::Noise,    colors::U_[2], true);
    ba.tracks[3] = BeatTrack::new("Sub Bass", 24, Waveform::Sine,     colors::U_[3], false);
    ba.tracks[4] = BeatTrack::new("Mid Bass", 36, Waveform::Square,   colors::U_[4], false);
    ba.tracks[5] = BeatTrack::new("Chords",   60, Waveform::Triangle, colors::U_[5], false);
    ba.tracks[6] = BeatTrack::new("Lead",     72, Waveform::Sawtooth, colors::U_[6], false);
    ba.tracks[7] = BeatTrack::new("Perc",     56, Waveform::Noise,    colors::U_[7], true);
    for t in ba.tracks.iter_mut() {
        t.num_steps = 32;
    }
    
    for i in 0..8 {
        ba.tracks[i].envelope = cpf.tracks[i].envelope;
        ba.tracks[i].volume = cpf.tracks[i].volume;
        ba.tracks[i].muted = false;
    }

    let ait = ba.step_duration_ms();
    let ix = 32usize;
    let qof = ait * ix as u32;

    
    let pmr: [Dc; 8] = [
        Dc {
            title: "KICK -- The Foundation",
            subtitle: "Four-on-the-floor kicks + ghost notes",
            detail: "Sine wave @ C2  |  Deep 808 thump  |  150ms decay",
            frames: 0,
        },
        Dc {
            title: "CLAP -- The Backbeat",
            subtitle: "Beats 2 & 4 with ghost flams",
            detail: "Noise burst  |  Tight snap  |  55ms decay",
            frames: 0,
        },
        Dc {
            title: "HI-HAT -- The Groove Engine",
            subtitle: "16th note groove with velocity dynamics",
            detail: "Noise  |  Crispy short  |  Off-beat accents for the funk",
            frames: 0,
        },
        Dc {
            title: "SUB BASS -- The Rumble",
            subtitle: "Deep sine sub following Cm -> Ab -> Bb",
            detail: "Sine wave @ C1 (33Hz!)  |  Long sustain  |  Feel it in your chest",
            frames: 0,
        },
        Dc {
            title: "MID BASS -- The Funk",
            subtitle: "Syncopated pluck riding on top of the sub",
            detail: "Square wave @ C2  |  Punchy pluck  |  Funky syncopation",
            frames: 0,
        },
        Dc {
            title: "CHORDS -- The Atmosphere",
            subtitle: "Lush pads: Cm -> Ab -> Bb progression",
            detail: "Triangle wave @ C4  |  Pad envelope  |  Harmonic movement",
            frames: 0,
        },
        Dc {
            title: "LEAD -- The Hook",
            subtitle: "Catchy melody: G5-Bb5-C6 rising, Eb6 peak!",
            detail: "Sawtooth @ C5  |  Singing melody  |  Call & response over 2 bars",
            frames: 0,
        },
        Dc {
            title: "PERCUSSION -- The Energy",
            subtitle: "Shakers + fill buildup into the drop",
            detail: "Noise burst  |  Snap envelope  |  Crescendo at bar 2 end",
            frames: 0,
        },
    ];

    
    for mp in 0..8 {
        ba.cursor_track = mp;

        
        let alc = &pmr[mp];
        bgd(fb_w, fb_h,
            &format!("TRACK {}/8", mp + 1),
            alc.title,
            alc.detail,
            colors::U_[mp],
        );
        crate::framebuffer::ii();
        if avs(5000) { aco(); return Ok(()); }

        
        let mut fbt: Vec<usize> = Vec::new();
        for j in 0..ix {
            if cpf.tracks[mp].steps[j].active {
                fbt.push(j);
            }
        }

        
        let dwl = format!("PHASE 1  //  TRACK {}/8  //  PLACING PATTERN", mp + 1);

        
        ba.draw();
        byx(alc, fb_w, fb_h, &dwl, 0, fbt.len() as u32);
        crate::framebuffer::ii();
        crate::cpu::tsc::ww(1200);

        
        for (place_idx, &step_pos) in fbt.iter().enumerate() {
            
            ba.cursor_step = step_pos;

            
            ba.draw();
            let progress = place_idx as u32;
            let av = fbt.len() as u32;
            byx(alc, fb_w, fb_h, &dwl, progress, av);
            crate::framebuffer::ii();

            
            crate::cpu::tsc::ww(200);

            
            ba.tracks[mp].steps[step_pos] = cpf.tracks[mp].steps[step_pos];

            
            ba.draw();
            byx(alc, fb_w, fb_h, &dwl, progress + 1, av);
            crate::framebuffer::ii();

            
            crate::cpu::tsc::ww(280);

            
            while let Some(dr) = crate::keyboard::kr() {
                if dr & 0x80 != 0 { continue; }
                if dr == 0x01 { aco(); return Ok(()); }
                if dr == 0x39 { break; } 
            }
        }

        
        let mzn = format!("PHASE 1  //  TRACK {}/8  //  LISTEN", mp + 1);

        let mzl = Dc {
            title: alc.title,
            subtitle: if mp == 0 {
                "Listening to the kick pattern..."
            } else {
                "Hear how this layer adds to the mix..."
            },
            detail: alc.detail,
            frames: 0,
        };

        
        let audio = ba.render_loop();
        ba.update_scope(&audio);

        
        let _ = crate::drivers::hda::bdu(&audio);
        ba.playing = true;

        
        let mut wc = false;
        for _loop_num in 0..3u32 {
            for j in 0..ix {
                ba.current_step = j;
                ba.update_spectrum();
                ba.draw();
                let progress = (j as u32 * 100) / ix as u32;
                byx(&mzl, fb_w, fb_h, &mzn, progress, 100);
                crate::framebuffer::ii();

                match bwv(ait as u64) {
                    1 => { wc = true; break; }
                    2 => { break; } 
                    _ => {}
                }
            }
            if wc { break; }
        }

        
        let _ = crate::drivers::hda::stop();
        ba.playing = false;
        ba.current_step = 0;

        if wc { aco(); return Ok(()); }

        
        crate::cpu::tsc::ww(1200);
    }

    
    
    
    crate::serial_println!("[SHOWCASE] Phase 2: Full mix playback");

    bgd(fb_w, fb_h,
        "PHASE 2: THE FULL MIX",
        "All 8 tracks together -- the complete Deep House groove",
        "Listen to how the layers combine with echo and sustain",
        0xFF6622,
    );
    crate::framebuffer::ii();
    if avs(5500) { aco(); return Ok(()); }

    
    let bmh = ba.render_loop();
    ba.update_scope(&bmh);

    let nfk = Dc {
        title: "FULL MIX -- All 8 Tracks",
        subtitle: "Kick + Clap + HiHat + Sub + Bass + Chords + Lead + Perc",
        detail: "100 BPM  |  C Minor  |  Deep House  |  Echo FX  |  Bare-Metal Audio",
        frames: 0,
    };

    
    let _ = crate::drivers::hda::bdu(&bmh);
    ba.playing = true;

    let mut wc = false;
    for loop_num in 0..3u32 {
        for j in 0..ix {
            ba.current_step = j;
            ba.update_spectrum();
            ba.draw();
            let nat = format!("PHASE 2  //  LOOP {}/3", loop_num + 1);
            let progress = (j as u32 * 100) / ix as u32;
            byx(&nfk, fb_w, fb_h, &nat, progress, 100);
            crate::framebuffer::ii();

            match bwv(ait as u64) {
                1 => { wc = true; break; }
                2 => { break; }
                _ => {}
            }
        }
        if wc { break; }
    }

    let _ = crate::drivers::hda::stop();
    ba.playing = false;
    ba.current_step = 0;
    if wc { aco(); return Ok(()); }

    
    
    
    crate::serial_println!("[SHOWCASE] Phase 3: Matrix visualizer");

    bgd(fb_w, fb_h,
        "PHASE 3: ENTER THE MATRIX",
        "The same beat, visualized as a living data stream",
        "Matrix rain  //  Beat-reactive  //  Pure framebuffer rendering",
        0x00FF44,
    );
    crate::framebuffer::ii();
    if avs(5500) { aco(); return Ok(()); }

    let mut matrix = MatrixState::new();

    
    for f in 0..25 {
        matrix.tick();
        let gdf = match f {
            0..=6   => "> LOADING BEAT DATA...",
            7..=14  => "> DECODING FREQUENCY MATRIX...",
            15..=20 => "> SYNTH ENGINES ONLINE...",
            _       => "> ENTERING THE BEAT MATRIX...",
        };
        matrix.draw(0, ix, gdf, ba.bpm, "---");
        crate::framebuffer::ii();
        if avs(150) { aco(); return Ok(()); }
    }

    
    let _ = crate::drivers::hda::bdu(&bmh);

    let imc = 3u32;
    wc = false;
    for loop_num in 0..imc {
        for j in 0..ix {
            ba.current_step = j;

            
            for t in 0..8 {
                if ba.tracks[t].steps[j].active && !ba.tracks[t].muted {
                    matrix.flash_beat(ba.tracks[t].steps[j].velocity);
                }
            }

            
            let mut avy = format!("LOOP {}/{}  > ", loop_num + 1, imc);
            for t in 0..8 {
                if ba.tracks[t].steps[j].active && !ba.tracks[t].muted {
                    avy.push_str(ba.tracks[t].name_str());
                    avy.push(' ');
                }
            }
            if avy.ends_with("> ") { avy.push_str("..."); }

            let bar = j / 16 + 1;
            let beat = (j % 16) / 4 + 1;
            let sub = j % 4 + 1;
            let bdb = format!("{}:{}.{}", bar, beat, sub);

            matrix.tick();
            matrix.draw(j, ix, &avy, ba.bpm, &bdb);
            crate::framebuffer::ii();

            match bwv(ait as u64) {
                1 => { wc = true; break; }
                2 => { break; }
                _ => {}
            }
        }
        if wc { break; }
    }

    let _ = crate::drivers::hda::stop();

    
    
    
    crate::serial_println!("[SHOWCASE] Outro");

    
    for f in 0..35 {
        matrix.tick();
        let glj = match f {
            0..=8   => "> SIGNAL FADING...",
            9..=20  => "> DISCONNECTING FROM THE MATRIX...",
            _       => "> TRUSTDAW // SYSTEM OFFLINE",
        };
        matrix.draw(0, ix, glj, ba.bpm, "---");
        crate::framebuffer::ii();

        
        let jna = matrix.num_cols / 30;
        for c in 0..jna {
            let idx = (f as usize * jna + c) % matrix.num_cols;
            matrix.columns[idx].active = false;
        }

        crate::cpu::tsc::ww(100); 
    }

    
    crate::framebuffer::fill_rect(0, 0, fb_w, fb_h, 0x020208);

    let mid = fb_h / 2;

    
    crate::framebuffer::fill_rect(fb_w / 4, mid - 80, fb_w / 2, 1, 0x00CCFF);
    crate::framebuffer::fill_rect(fb_w / 4, mid + 80, fb_w / 2, 1, 0x00CCFF);

    let kzp: [(&str, u32); 8] = [
        ("T R U S T D A W",                          0x00FF66),
        ("",                                          0x000000),
        ("A bare-metal beat production studio",       0xCCCCDD),
        ("running on TrustOS -- written in Rust",     0xCCCCDD),
        ("",                                          0x000000),
        ("No operating system. No libraries.",         0x88AACC),
        ("Just raw hardware, a framebuffer, and HDA audio.", 0x88AACC),
        ("",                                          0x000000),
    ];

    let start_y = mid - 60;
    for (i, (text, color)) in kzp.iter().enumerate() {
        if text.is_empty() { continue; }
        let gr = text.len() as u32 * 8;
        let bu = (fb_w - gr) / 2;
        crate::framebuffer::draw_text(text, bu, start_y + i as u32 * 20, *color);
    }

    
    let tag = "Press any key to exit";
    let gr = tag.len() as u32 * 8;
    crate::framebuffer::draw_text(tag, (fb_w - gr) / 2, mid + 60, 0x556677);
    crate::framebuffer::ii();

    
    loop {
        if let Some(dr) = crate::keyboard::kr() {
            if dr & 0x80 == 0 { break; }
        }
        crate::cpu::tsc::ww(20);
    }

    aco();
    crate::serial_println!("[SHOWCASE] Narrated showcase complete");
    Ok(())
}


fn aco() {
    crate::framebuffer::pr(false);
}


fn gfj(state: u32) -> u32 {
    let mut j = state;
    if j == 0 { j = 0xDEAD_BEEF; }
    j ^= j << 13;
    j ^= j >> 17;
    j ^= j << 5;
    j
}






pub fn mww() -> Result<(), &'static str> {
    crate::audio::init().ok();
    crate::serial_println!("[ANTHEM] Starting TrustOS Anthem...");

    let fb_w = crate::framebuffer::X_.load(Ordering::Relaxed) as u32;
    let fb_h = crate::framebuffer::W_.load(Ordering::Relaxed) as u32;

    crate::framebuffer::adw();
    crate::framebuffer::pr(true);

    
    
    
    bgd(fb_w, fb_h,
        "T R U S T O S    A N T H E M",
        "Renaissance Numerique",
        "Cm -> C Major  //  106 BPM  //  Tension -> Revelation -> Maitrise",
        0x00CCFF,
    );
    crate::framebuffer::ii();
    if avs(6000) { aco(); return Ok(()); }

    
    
    
    struct La {
        title: &'static str,
        subtitle: &'static str,
        detail: &'static str,
        color: u32,
        loops: u32,
    }

    let sections = [
        La {
            title: "INTRO -- L'EVEIL",
            subtitle: "Quelque chose s'eveille...",
            detail: "Pad drone  |  Heartbeat sub  |  Texture digitale",
            color: 0x4466CC, loops: 6,
        },
        La {
            title: "BUILD -- L'ESPOIR",
            subtitle: "L'espoir nait, le rythme s'installe",
            detail: "Kick doux  |  Arpege montant  |  Basse chaude",
            color: 0x44AAFF, loops: 8,
        },
        La {
            title: "DROP -- LA REVELATION",
            subtitle: "Explosion controlee. Le controle est repris.",
            detail: "Full mix  |  Lead lumineux  |  Groove electro-funk",
            color: 0xFF6622, loops: 10,
        },
        La {
            title: "STABLE -- LA MAITRISE",
            subtitle: "Le theme TrustOS. Souverain. Reconnaissable.",
            detail: "Motif C-E-G-C  |  Cm -> C Major!  |  Identite sonore",
            color: 0x00FF66, loops: 10,
        },
        La {
            title: "OUTRO -- FUTUR SOUVERAIN",
            subtitle: "Le signal s'estompe... le motif reste.",
            detail: "Pad + motif  |  Serenite  |  Un futur souverain",
            color: 0x8844FF, loops: 6,
        },
    ];

    
    
    
    for (sec_idx, lx) in sections.iter().enumerate() {
        
        bgd(fb_w, fb_h,
            &format!("SECTION {}/5", sec_idx + 1),
            lx.title,
            lx.detail,
            lx.color,
        );
        crate::framebuffer::ii();
        if avs(4500) { aco(); return Ok(()); }

        
        let mut ba = BeatStudio::new();
        match sec_idx {
            0 => ba.anthem_intro(),
            1 => ba.anthem_build(),
            2 => ba.anthem_drop(),
            3 => ba.anthem_stable(),
            _ => ba.anthem_outro(),
        }

        
        let audio = ba.render_loop();
        ba.update_scope(&audio);
        let _ = crate::drivers::hda::bdu(&audio);
        ba.playing = true;

        let ait = ba.step_duration_ms();
        let ix = 32usize;

        let alc = Dc {
            title: lx.title,
            subtitle: lx.subtitle,
            detail: lx.detail,
            frames: 0,
        };

        
        let mut wc = false;
        for loop_num in 0..lx.loops {
            for j in 0..ix {
                ba.current_step = j;
                ba.update_spectrum();
                ba.draw();
                let dwl = format!(
                    "SECTION {}/5  //  LOOP {}/{}",
                    sec_idx + 1, loop_num + 1, lx.loops
                );
                let progress = (j as u32 * 100) / ix as u32;
                byx(&alc, fb_w, fb_h, &dwl, progress, 100);
                crate::framebuffer::ii();

                match bwv(ait as u64) {
                    1 => { wc = true; break; } 
                    2 => { break; }                  
                    _ => {}
                }
            }
            if wc { break; }
        }

        let _ = crate::drivers::hda::stop();
        ba.playing = false;
        if wc { aco(); return Ok(()); }

        
        crate::cpu::tsc::ww(800);
    }

    
    
    
    crate::framebuffer::fill_rect(0, 0, fb_w, fb_h, 0x020208);
    let mid = fb_h / 2;
    crate::framebuffer::fill_rect(fb_w / 4, mid - 80, fb_w / 2, 2, 0x00CCFF);
    crate::framebuffer::fill_rect(fb_w / 4, mid + 80, fb_w / 2, 2, 0x00CCFF);

    let scale = 2u32;
    let ew = 8 * scale;

    let title = "T R U S T O S   A N T H E M";
    let ahg = title.len() as u32 * ew;
    crate::graphics::scaling::aat(
        ((fb_w.saturating_sub(ahg)) / 2) as i32, (mid - 55) as i32,
        title, 0x00FF66, scale);

    let sub = "Renaissance Numerique  --  Un futur souverain.";
    let aeo = sub.len() as u32 * ew;
    crate::graphics::scaling::aat(
        ((fb_w.saturating_sub(aeo)) / 2) as i32, (mid - 10) as i32,
        sub, 0xCCCCDD, scale);

    let info = "Composed on TrustOS  //  Bare-metal Rust  //  Native HDA Audio";
    let ane = info.len() as u32 * ew;
    crate::graphics::scaling::aat(
        ((fb_w.saturating_sub(ane)) / 2) as i32, (mid + 30) as i32,
        info, 0x88AACC, scale);

    let tag = "Press any key to exit";
    let gr = tag.len() as u32 * 8;
    crate::framebuffer::draw_text(tag, (fb_w - gr) / 2, mid + 65, 0x556677);
    crate::framebuffer::ii();

    loop {
        if let Some(dr) = crate::keyboard::kr() {
            if dr & 0x80 == 0 { break; }
        }
        crate::cpu::tsc::ww(20);
    }

    aco();
    crate::serial_println!("[ANTHEM] TrustOS Anthem complete");
    Ok(())
}







pub fn mxc() -> Result<(), &'static str> {
    crate::audio::init().ok();
    crate::serial_println!("[CYBER] 'Neon Protocol' — Creative Process + Full Song");

    let fb_w = crate::framebuffer::X_.load(Ordering::Relaxed) as u32;
    let fb_h = crate::framebuffer::W_.load(Ordering::Relaxed) as u32;

    crate::framebuffer::adw();
    crate::framebuffer::pr(true);

    let ix = 32usize;
    let mut ba = BeatStudio::new();

    
    
    
    

    bgd(fb_w, fb_h,
        "T R U S T D A W",
        "\"NEON PROTOCOL\" — Creative Process",
        "Watch the beat come alive  |  100 BPM  |  Eb minor",
        0x00FFCC,
    );
    crate::framebuffer::ii();
    if avs(5000) { aco(); return Ok(()); }

    
    ba.load_trap_hook();
    let ait = ba.step_duration_ms();
    for t in 0..8 { ba.tracks[t].muted = true; }

    
    let layers: [(usize, &str, &str, u32, u32); 8] = [
        (0, "+ SUB BASS",  "The foundation — 43 Hz Eb1 rumble",    3, 0xFF4444),
        (1, "+ SNARE",     "Hard mechanical crack — beats 3 & 7",  2, 0xFFAA22),
        (2, "+ HI-HAT",    "Aggressive 16th-note machine gun",     2, 0xFFFF44),
        (3, "+ OPEN HAT",  "Digital sizzle — off-beat accents",    1, 0x88FF44),
        (4, "+ SYNTH",     "Neon arpeggio: Eb > B > Ab > Gb",      2, 0x44DDFF),
        (5, "+ PAD",       "Cold digital atmosphere",               2, 0x8844FF),
        (6, "+ LEAD",      "The hook — cuts through the noise",     2, 0xFF44CC),
        (7, "+ PERC",      "Glitch percussion accents",             1, 0xCCCCCC),
    ];

    for &(mp, name, desc, loops, accent) in &layers {
        
        ba.tracks[mp].muted = false;

        
        bgd(fb_w, fb_h, name, desc, "", accent);
        crate::framebuffer::ii();
        if avs(1800) { aco(); return Ok(()); }

        
        let audio = ba.render_loop();
        ba.update_scope(&audio);
        let _ = crate::drivers::hda::bdu(&audio);
        ba.playing = true;

        let alc = Dc {
            title: name,
            subtitle: desc,
            detail: "Building the beat...",
            frames: 0,
        };

        let mut wc = false;
        for loop_num in 0..loops {
            for j in 0..ix {
                ba.current_step = j;
                ba.update_spectrum();

                for note in 0..128 { ba.keys_pressed[note] = false; }
                for t_idx in 0..8 {
                    if ba.tracks[t_idx].muted { continue; }
                    if ba.tracks[t_idx].steps[j].active {
                        let aad = ba.tracks[t_idx].note_at(j);
                        if aad > 0 && aad < 128 {
                            ba.keys_pressed[aad as usize] = true;
                        }
                    }
                }

                ba.draw();
                let label = format!("{} — Loop {}/{}", name, loop_num + 1, loops);
                let progress = (j as u32 * 100) / ix as u32;
                byx(&alc, fb_w, fb_h, &label, progress, 100);
                crate::framebuffer::ii();

                match bwv(ait as u64) {
                    1 => { wc = true; break; }
                    2 => { break; }
                    _ => {}
                }
            }
            if wc { break; }
        }

        let _ = crate::drivers::hda::stop();
        ba.playing = false;
        ba.current_step = 0;
        for note in 0..128 { ba.keys_pressed[note] = false; }
        if wc { aco(); return Ok(()); }
    }

    
    bgd(fb_w, fb_h,
        "ALL LAYERS ACTIVE",
        "The complete beat — \"Neon Protocol\"",
        "8 tracks  |  100 BPM  |  Eb minor",
        0x00FFCC,
    );
    crate::framebuffer::ii();
    if avs(2500) { aco(); return Ok(()); }

    {
        let audio = ba.render_loop();
        ba.update_scope(&audio);
        let _ = crate::drivers::hda::bdu(&audio);
        ba.playing = true;

        let alc = Dc {
            title: "FULL MIX",
            subtitle: "All 8 layers combined",
            detail: "Neon Protocol — complete beat",
            frames: 0,
        };

        let mut wc = false;
        for loop_num in 0..3u32 {
            for j in 0..ix {
                ba.current_step = j;
                ba.update_spectrum();
                for note in 0..128 { ba.keys_pressed[note] = false; }
                for t_idx in 0..8 {
                    if !ba.tracks[t_idx].muted && ba.tracks[t_idx].steps[j].active {
                        let aad = ba.tracks[t_idx].note_at(j);
                        if aad > 0 && aad < 128 { ba.keys_pressed[aad as usize] = true; }
                    }
                }
                ba.draw();
                let label = format!("FULL MIX — Loop {}/3", loop_num + 1);
                byx(&alc, fb_w, fb_h, &label, (j as u32 * 100) / ix as u32, 100);
                crate::framebuffer::ii();
                match bwv(ait as u64) {
                    1 => { wc = true; break; }
                    2 => { break; }
                    _ => {}
                }
            }
            if wc { break; }
        }

        let _ = crate::drivers::hda::stop();
        ba.playing = false;
        ba.current_step = 0;
        if wc { aco(); return Ok(()); }
    }

    
    
    
    
    
    

    bgd(fb_w, fb_h,
        "ENTERING THE MATRIX",
        "\"NEON PROTOCOL\" — Full Song",
        "8 sections  |  Pulsing waveform  |  [Esc] Exit",
        0x00FFCC,
    );
    crate::framebuffer::ii();
    if avs(4000) { aco(); return Ok(()); }

    let mut matrix = MatrixState::new();

    let omo: [&str; 8] = [
        "INTRO — System Boot",
        "DROP — Neon Protocol",
        "BREAKDOWN — Signal Lost",
        "BUILD — Recompile",
        "BRIDGE — Blackout",
        "REBUILD — Reboot Sequence",
        "FINAL DROP — Full Override",
        "OUTRO — Shutdown",
    ];
    let jdw: [u32; 8] = [3, 5, 1, 4, 1, 3, 3, 1];

    
    for f in 0..30u32 {
        matrix.tick();
        matrix.draw_rain();
        let bk = match f {
            0..=8   => "INITIALIZING NEON PROTOCOL...",
            9..=18  => "LOADING WAVEFORM ENGINE...",
            _       => "READY.",
        };
        let dur = bk.len() as u32 * 16 + 32;
        let cg = (fb_w.saturating_sub(dur)) / 2;
        let cr = fb_h / 2 - 16;
        crate::framebuffer::co(cg.saturating_sub(8), cr.saturating_sub(8), dur + 16, 48, 0x000000, 180);
        crate::graphics::scaling::aat(cg as i32, cr as i32, bk, 0x00FFCC, 2);
        crate::framebuffer::ii();
        if avs(80) { aco(); return Ok(()); }
    }

    
    
    let mut bmh: Vec<i16> = Vec::new();
    
    let mut gtg: Vec<(usize, usize)> = Vec::new(); 
    let mut fzd: usize = 0;

    for lx in 0..8usize {
        match lx {
            0 => ba.load_trap_intro(),
            1 => ba.load_trap_hook(),
            2 => ba.load_trap_verse(),
            3 => ba.load_trap_build(),
            4 => ba.load_trap_bridge(),
            5 => ba.load_trap_rebuild(),
            6 => ba.load_trap_hook_final(),
            _ => ba.load_trap_outro(),
        }
        let omq = ba.render_loop();
        for _lp in 0..jdw[lx] {
            gtg.push((lx, fzd));
            bmh.extend_from_slice(&omq);
            fzd += ix;
        }
    }

    let ait = ba.step_duration_ms();
    let oxe = (60u32 * 48000) / (ba.bpm as u32 * 4);
    let jci = oxe as usize * 2; 
    let ply = fzd;

    
    let _ = crate::drivers::hda::bdu(&bmh);

    
    let mut wc = false;
    let mut ejj = 0usize;
    let mut hpl = 0u32;

    for g in 0..ply {
        
        
        for (bal, &(si, gs)) in gtg.iter().enumerate() {
            if gs <= g {
                ejj = si;
                
                hpl = gtg[..=bal].iter().filter(|&&(j, _)| j == si).count() as u32 - 1;
            }
        }
        let j = g % ix; 

        
        let acl = g * jci;
        let cv = ((g + 1) * jci).min(bmh.len());
        if acl < bmh.len() {
            ba.update_scope(&bmh[acl..cv]);
        }
        let energy = kwl(&bmh, acl, cv);

        
        matrix.tick();

        
        matrix.draw_rain();
        lja(fb_w, fb_h, &ba.scope_buffer, energy, matrix.frame);
        lim(fb_w, fb_h, omo[ejj], ejj,
            hpl, jdw[ejj], j, ix, ba.bpm);

        crate::framebuffer::ii();

        match bwv(ait as u64) {
            1 => { wc = true; break; }
            2 => {} 
            _ => {}
        }
    }

    let _ = crate::drivers::hda::stop();
    if wc { aco(); return Ok(()); }

    
    for f in 0..50u32 {
        matrix.tick();
        let dmm = matrix.num_cols / 50;
        for c in 0..dmm {
            let idx = (f as usize * dmm + c) % matrix.num_cols;
            matrix.columns[idx].active = false;
        }
        matrix.draw_rain();

        let bk = match f {
            0..=15  => "DISCONNECTING...",
            16..=30 => "SIGNAL LOST",
            _       => "NEON PROTOCOL // OFFLINE",
        };
        let dur = bk.len() as u32 * 16 + 32;
        let cg = (fb_w.saturating_sub(dur)) / 2;
        let cr = fb_h / 2 - 16;
        crate::framebuffer::co(cg.saturating_sub(8), cr.saturating_sub(8), dur + 16, 48, 0x000000, 180);
        crate::graphics::scaling::aat(cg as i32, cr as i32, bk, 0x00FFCC, 2);
        crate::framebuffer::ii();
        crate::cpu::tsc::ww(80);
    }

    
    crate::framebuffer::fill_rect(0, 0, fb_w, fb_h, 0x050510);
    let mid = fb_h / 2;
    crate::framebuffer::fill_rect(fb_w / 4, mid - 80, fb_w / 2, 2, 0x00FFCC);
    crate::framebuffer::fill_rect(fb_w / 4, mid + 80, fb_w / 2, 2, 0x00FFCC);

    let scale = 2u32;
    let ew = 8 * scale;

    let ll = "\"NEON PROTOCOL\"";
    let ahg = ll.len() as u32 * ew;
    crate::graphics::scaling::aat(
        ((fb_w.saturating_sub(ahg)) / 2) as i32, (mid - 55) as i32,
        ll, 0x00FFCC, scale);

    let np = "Cyberpunk Trap — 100 BPM — Eb minor";
    let aeo = np.len() as u32 * ew;
    crate::graphics::scaling::aat(
        ((fb_w.saturating_sub(aeo)) / 2) as i32, (mid - 10) as i32,
        np, 0xCCCCDD, scale);

    let acw = "Creative Process + Full Song — Bare Metal Rust";
    let ane = acw.len() as u32 * ew;
    crate::graphics::scaling::aat(
        ((fb_w.saturating_sub(ane)) / 2) as i32, (mid + 30) as i32,
        acw, 0x8844CC, scale);

    let tag = "Press any key to exit";
    let gr = tag.len() as u32 * 8;
    crate::framebuffer::draw_text(tag, (fb_w - gr) / 2, mid + 65, 0x446688);
    crate::framebuffer::ii();

    loop {
        if let Some(dr) = crate::keyboard::kr() {
            if dr & 0x80 == 0 { break; }
        }
        crate::cpu::tsc::ww(20);
    }

    aco();
    crate::serial_println!("[CYBER] 'Neon Protocol' complete");
    Ok(())
}
