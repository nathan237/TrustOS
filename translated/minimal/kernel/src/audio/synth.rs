










use alloc::vec::Vec;
use alloc::string::String;
use alloc::format;
use super::tables::{YJ_, AHI_};


pub const BT_: u32 = 48000;

pub const Bq: u32 = 2;

pub const DIB_: u32 = 2;

pub const BCG_: usize = 8;

const EU_: u32 = 16;

const JV_: u32 = 256;

const CHV_: u16 = 0xACE1;






#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Waveform {
    Sine,
    Square,
    Sawtooth,
    Triangle,
    Noise,
}

impl Waveform {
    
    pub fn atv(j: &str) -> Option<Self> {
        match j {
            "sine" | "sin" | "s" => Some(Waveform::Sine),
            "square" | "sq" | "q" => Some(Waveform::Square),
            "saw" | "sawtooth" | "w" => Some(Waveform::Sawtooth),
            "triangle" | "tri" | "t" => Some(Waveform::Triangle),
            "noise" | "n" => Some(Waveform::Noise),
            _ => None,
        }
    }

    
    pub fn short_name(&self) -> &'static str {
        match self {
            Waveform::Sine => "Sin",
            Waveform::Square => "Sqr",
            Waveform::Sawtooth => "Saw",
            Waveform::Triangle => "Tri",
            Waveform::Noise => "Noi",
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Waveform::Sine => "Sine",
            Waveform::Square => "Square",
            Waveform::Sawtooth => "Sawtooth",
            Waveform::Triangle => "Triangle",
            Waveform::Noise => "Noise",
        }
    }
}






#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EnvState {
    Idle,
    Attack,
    Decay,
    Sustain,
    Release,
}


#[derive(Debug, Clone, Copy)]
pub struct Envelope {
    
    pub attack_samples: u32,
    
    pub decay_samples: u32,
    
    pub sustain_level: i32,
    
    pub release_samples: u32,
    
    state: EnvState,
    
    level: i32,
    
    counter: u32,
}

impl Envelope {
    
    pub fn new(attack_ms: u32, decay_ms: u32, sustain_pct: u32, release_ms: u32) -> Self {
        let oyu = (sustain_pct.min(100) as i32 * 32767) / 100;
        Self {
            attack_samples: dup(attack_ms),
            decay_samples: dup(decay_ms),
            sustain_level: oyu,
            release_samples: dup(release_ms),
            state: EnvState::Idle,
            level: 0,
            counter: 0,
        }
    }

    
    pub fn eka() -> Self {
        Self::new(10, 50, 70, 100)
    }

    
    pub fn nnt() -> Self {
        Self::new(1, 1, 100, 10)
    }

    
    pub fn dwp() -> Self {
        Self::new(2, 200, 0, 50)
    }

    
    pub fn pad() -> Self {
        Self::new(300, 100, 80, 500)
    }

    
    pub fn note_on(&mut self) {
        self.state = EnvState::Attack;
        self.counter = 0;
        
    }

    
    pub fn note_off(&mut self) {
        if self.state != EnvState::Idle {
            self.state = EnvState::Release;
            self.counter = 0;
        }
    }

    
    pub fn tick(&mut self) -> i32 {
        match self.state {
            EnvState::Idle => {
                self.level = 0;
            }
            EnvState::Attack => {
                if self.attack_samples == 0 {
                    self.level = 32767;
                    self.state = EnvState::Decay;
                    self.counter = 0;
                } else {
                    self.level = ((self.counter as i64 * 32767) / self.attack_samples as i64) as i32;
                    self.counter += 1;
                    if self.counter >= self.attack_samples {
                        self.level = 32767;
                        self.state = EnvState::Decay;
                        self.counter = 0;
                    }
                }
            }
            EnvState::Decay => {
                if self.decay_samples == 0 {
                    self.level = self.sustain_level;
                    self.state = EnvState::Sustain;
                } else {
                    let mk = 32767 - self.sustain_level;
                    self.level = 32767 - ((self.counter as i64 * mk as i64) / self.decay_samples as i64) as i32;
                    self.counter += 1;
                    if self.counter >= self.decay_samples {
                        self.level = self.sustain_level;
                        self.state = EnvState::Sustain;
                        self.counter = 0;
                    }
                }
            }
            EnvState::Sustain => {
                self.level = self.sustain_level;
                
            }
            EnvState::Release => {
                if self.release_samples == 0 {
                    self.level = 0;
                    self.state = EnvState::Idle;
                } else {
                    let jih = if self.counter == 0 { self.level } else {
                        
                        
                        self.sustain_level
                    };
                    self.level = jih - ((self.counter as i64 * jih as i64) / self.release_samples as i64) as i32;
                    if self.level < 0 { self.level = 0; }
                    self.counter += 1;
                    if self.counter >= self.release_samples {
                        self.level = 0;
                        self.state = EnvState::Idle;
                    }
                }
            }
        }
        self.level
    }

    
    pub fn is_idle(&self) -> bool {
        self.state == EnvState::Idle
    }

    
    pub fn state_name(&self) -> &'static str {
        match self.state {
            EnvState::Idle => "Idle",
            EnvState::Attack => "Atk",
            EnvState::Decay => "Dec",
            EnvState::Sustain => "Sus",
            EnvState::Release => "Rel",
        }
    }
}






#[derive(Debug, Clone)]
pub struct Oscillator {
    
    pub waveform: Waveform,
    
    phase: u32,
    
    phase_inc: u32,
    
    pub freq_hz: u32,
    
    lfsr: u16,
}

impl Oscillator {
    
    pub fn new(waveform: Waveform, freq_hz: u32) -> Self {
        let phase_inc = Self::hjr(freq_hz);
        Self {
            waveform,
            phase: 0,
            phase_inc,
            freq_hz,
            lfsr: CHV_,
        }
    }

    
    
    fn hjr(freq_hz: u32) -> u32 {
        
        ((freq_hz as u64 * (JV_ as u64) << EU_) / BT_ as u64) as u32
    }

    
    pub fn set_freq(&mut self, freq_hz: u32) {
        self.freq_hz = freq_hz;
        self.phase_inc = Self::hjr(freq_hz);
    }

    
    pub fn qwg(&mut self, note: u8) {
        let freq = AHI_[note.min(127) as usize];
        self.set_freq(freq);
    }

    
    pub fn quf(&mut self) {
        self.phase = 0;
    }

    
    pub fn tick(&mut self) -> i16 {
        let sample = match self.waveform {
            Waveform::Sine => self.gen_sine(),
            Waveform::Square => self.gen_square(),
            Waveform::Sawtooth => self.gen_sawtooth(),
            Waveform::Triangle => self.gen_triangle(),
            Waveform::Noise => self.gen_noise(),
        };

        
        self.phase = self.phase.wrapping_add(self.phase_inc);

        sample
    }

    
    fn gen_sine(&self) -> i16 {
        let jlg = (self.phase >> EU_) as usize & 0xFF;
        let yt = (self.phase & 0xFFFF) as i32;

        let auz = YJ_[jlg] as i32;
        let afq = YJ_[(jlg + 1) & 0xFF] as i32;

        
        let interp = auz + ((afq - auz) * yt >> 16);
        interp as i16
    }

    
    
    
    
    fn poly_blep(&self, aa: u32) -> i32 {
        let bmy = self.phase_inc;
        if bmy == 0 { return 0; }
        let zd = (JV_ << EU_) as u32;

        
        if aa < bmy {
            let t = ((aa as u64) << 16) / bmy as u64;
            let t = t as i32;
            
            return 2 * t - ((t as i64 * t as i64) >> 16) as i32 - 65536;
        }

        
        if aa > zd.saturating_sub(bmy) {
            let t = (((aa as i64) - zd as i64) << 16) / bmy as i64;
            let t = t as i32;
            
            return ((t as i64 * t as i64) >> 16) as i32 + 2 * t + 65536;
        }

        0
    }

    
    fn gen_square(&self) -> i16 {
        let dco = ((JV_ << EU_) - 1) as u32;
        let cw = (128u32) << EU_;
        let aa = self.phase & dco;

        let gin: i32 = if aa < cw { 24000 } else { -24000 };

        
        let kcl = self.poly_blep(aa);
        let kck = self.poly_blep(aa.wrapping_sub(cw) & dco);

        let sample = gin
            + ((kcl as i64 * 24000) >> 16) as i32
            - ((kck as i64 * 24000) >> 16) as i32;

        sample.clamp(-32767, 32767) as i16
    }

    
    fn gen_sawtooth(&self) -> i16 {
        let dco = ((JV_ << EU_) - 1) as u32;
        let zd = (JV_ << EU_) as u64;
        let aa = self.phase & dco;

        
        let gin = ((aa as i64 * 48000) / zd as i64 - 24000) as i32;

        
        let kcj = self.poly_blep(aa);
        let kyd = ((kcj as i64 * 24000) >> 16) as i32;

        (gin - kyd).clamp(-32767, 32767) as i16
    }

    
    fn gen_triangle(&self) -> i16 {
        let dco = ((JV_ << EU_) - 1) as u32;
        let aa = self.phase & dco;
        let cw = (128u32) << EU_;

        if aa < cw {
            
            ((aa as i64 * 48000 / cw as i64) - 24000) as i16
        } else {
            
            let rev = ((JV_ << EU_) as u32).wrapping_sub(aa);
            ((rev as i64 * 48000 / cw as i64) - 24000) as i16
        }
    }

    
    fn gen_noise(&mut self) -> i16 {
        
        let bf = self.lfsr & 1;
        self.lfsr >>= 1;
        if bf == 1 {
            self.lfsr ^= 0xB400; 
        }
        
        (self.lfsr as i16).wrapping_mul(3) / 4 
    }
}







#[derive(Debug, Clone, Copy)]
struct LowPassFilter {
    y1: i32,    
    y2: i32,    
    alpha: u32, 
}

impl LowPassFilter {
    
    fn hjg() -> Self {
        Self { y1: 0, y2: 0, alpha: 65536 }
    }

    
    fn set_cutoff(&mut self, cutoff_hz: u32) {
        
        let w = (6283u64 * cutoff_hz as u64) / BT_ as u64;
        
        self.alpha = ((w << 16) / (1000 + w)).min(65536) as u32;
    }

    
    fn process(&mut self, input: i32) -> i32 {
        let a = self.alpha as i64;
        
        self.y1 += (((input - self.y1) as i64 * a) >> 16) as i32;
        
        self.y2 += (((self.y1 - self.y2) as i64 * a) >> 16) as i32;
        self.y2
    }

    fn reset(&mut self) {
        self.y1 = 0;
        self.y2 = 0;
    }
}






#[derive(Debug, Clone)]
pub struct Voice {
    pub osc: Oscillator,
    
    osc2: Oscillator,
    pub env: Envelope,
    
    filter: LowPassFilter,
    
    pub note: u8,
    
    pub velocity: u8,
    
    pub active: bool,
    
    drift_phase: u32,
    
    base_inc: u32,
}

impl Voice {
    pub fn new() -> Self {
        Self {
            osc: Oscillator::new(Waveform::Sine, 440),
            osc2: Oscillator::new(Waveform::Sine, 440),
            env: Envelope::eka(),
            filter: LowPassFilter::hjg(),
            note: 0,
            velocity: 0,
            active: false,
            drift_phase: 0,
            base_inc: 0,
        }
    }

    
    pub fn note_on(&mut self, note: u8, velocity: u8, waveform: Waveform, envelope: Envelope) {
        let freq = AHI_[note.min(127) as usize];

        
        self.osc = Oscillator::new(waveform, freq);
        self.base_inc = self.osc.phase_inc;
        self.drift_phase = 0;

        
        let lyr = freq + (freq / 200).max(1);
        self.osc2 = Oscillator::new(waveform, lyr);

        self.env = envelope;
        self.env.note_on();
        self.note = note;
        self.velocity = velocity;
        self.active = true;

        
        self.filter.reset();
        match waveform {
            Waveform::Sine => {
                
                self.filter = LowPassFilter::hjg();
            }
            Waveform::Triangle => {
                
                let fqc = (freq * 12).max(400).min(16000);
                self.filter.set_cutoff(fqc);
            }
            Waveform::Square | Waveform::Sawtooth => {
                
                let fqc = (freq * 8).max(300).min(12000);
                self.filter.set_cutoff(fqc);
            }
            Waveform::Noise => {
                
                self.filter.set_cutoff(6000);
            }
        }
    }

    
    pub fn note_off(&mut self) {
        self.env.note_off();
    }

    
    pub fn tick(&mut self) -> i16 {
        if !self.active {
            return 0;
        }

        let lqx = self.env.tick();
        if self.env.is_idle() {
            self.active = false;
            return 0;
        }

        
        self.drift_phase = self.drift_phase.wrapping_add(19);  
        let llh = (self.drift_phase >> 8) as usize & 0xFF;
        let llj = YJ_[llh] as i32;  
        
        let lli = ((self.base_inc as i64 * llj as i64) / (32767 * 1250)) as i32;
        self.osc.phase_inc = (self.base_inc as i32 + lli).max(1) as u32;

        
        let obl = self.osc.tick() as i32;
        let obm = self.osc2.tick() as i32;
        let dm = (obl + obm) / 2;

        
        let filtered = self.filter.process(dm);

        
        let oke = if filtered > 18000 {
            18000 + (filtered - 18000) / 4
        } else if filtered < -18000 {
            -18000 + (filtered + 18000) / 4
        } else {
            filtered
        };

        let prm = self.velocity as i32;

        
        let sample = (oke * lqx / 32767) * prm / 127;
        sample.clamp(-32767, 32767) as i16
    }
}






pub struct SynthEngine {
    
    pub voices: [Voice; BCG_],
    
    pub waveform: Waveform,
    
    pub envelope: Envelope,
    
    pub master_volume: u8,
}

impl SynthEngine {
    
    pub fn new() -> Self {
        let voices = core::array::from_fn(|_| Voice::new());
        Self {
            voices,
            waveform: Waveform::Sine,
            envelope: Envelope::eka(),
            master_volume: 200,
        }
    }

    
    pub fn set_waveform(&mut self, aal: Waveform) {
        self.waveform = aal;
    }

    
    pub fn set_adsr(&mut self, attack_ms: u32, decay_ms: u32, sustain_pct: u32, release_ms: u32) {
        self.envelope = Envelope::new(attack_ms, decay_ms, sustain_pct, release_ms);
    }

    
    pub fn note_on(&mut self, note: u8, velocity: u8) {
        
        let feq = self.find_free_voice();
        let and = &mut self.voices[feq];
        and.note_on(note, velocity, self.waveform, self.envelope);
    }

    
    pub fn note_off(&mut self, note: u8) {
        for and in &mut self.voices {
            if and.active && and.note == note {
                and.note_off();
                break;
            }
        }
    }

    
    pub fn all_notes_off(&mut self) {
        for and in &mut self.voices {
            and.note_off();
        }
    }

    
    
    pub fn render(&mut self, buffer: &mut [i16], cbz: usize) -> usize {
        let jnb = cbz.min(buffer.len() / 2); 

        for i in 0..jnb {
            
            let mut aif: i32 = 0;
            for and in &mut self.voices {
                if and.active {
                    aif += and.tick() as i32;
                }
            }

            
            aif = aif * self.master_volume as i32 / 255;

            
            let sample = aif.clamp(-32767, 32767) as i16;

            
            buffer[i * 2] = sample;
            buffer[i * 2 + 1] = sample;
        }

        jnb
    }

    
    pub fn render_note(&mut self, note: u8, velocity: u8, duration_ms: u32) -> Vec<i16> {
        let aai = dup(duration_ms) as usize;
        
        let release_samples = self.envelope.release_samples as usize;
        let fxz = aai + release_samples;
        let mut buffer = alloc::vec![0i16; fxz * 2]; 

        
        self.note_on(note, velocity);

        
        self.render(&mut buffer[..aai * 2], aai);

        
        self.note_off(note);

        
        if release_samples > 0 {
            self.render(&mut buffer[aai * 2..], release_samples);
        }

        buffer
    }

    
    pub fn play_note_by_name(&mut self, name: &str, duration_ms: u32) -> Result<Vec<i16>, &'static str> {
        let midi_note = super::tables::cnh(name)
            .ok_or("Invalid note name (use e.g. C4, A#3, Bb5)")?;
        Ok(self.render_note(midi_note, 100, duration_ms))
    }

    
    pub fn render_freq(&mut self, freq_hz: u32, duration_ms: u32) -> Vec<i16> {
        let aai = dup(duration_ms) as usize;
        let release_samples = self.envelope.release_samples as usize;
        let fxz = aai + release_samples;
        let mut buffer = alloc::vec![0i16; fxz * 2]; 

        
        let feq = self.find_free_voice();
        let and = &mut self.voices[feq];
        and.osc = Oscillator::new(self.waveform, freq_hz);
        and.env = self.envelope;
        and.env.note_on();
        and.note = 69; 
        and.velocity = 100;
        and.active = true;

        
        self.render(&mut buffer[..aai * 2], aai);
        
        self.voices[feq].note_off();
        if release_samples > 0 {
            self.render(&mut buffer[aai * 2..], release_samples);
        }

        buffer
    }

    
    pub fn active_voice_count(&self) -> usize {
        self.voices.iter().filter(|v| v.active).count()
    }

    
    pub fn status(&self) -> String {
        let mut j = String::new();
        j.push_str(&format!("TrustSynth Engine\n"));
        j.push_str(&format!("  Waveform: {}\n", self.waveform.name()));
        j.push_str(&format!("  ADSR: A={}ms D={}ms S={}% R={}ms\n",
            gsj(self.envelope.attack_samples),
            gsj(self.envelope.decay_samples),
            self.envelope.sustain_level * 100 / 32767,
            gsj(self.envelope.release_samples)
        ));
        j.push_str(&format!("  Master Volume: {}/255\n", self.master_volume));
        j.push_str(&format!("  Active Voices: {}/{}\n", self.active_voice_count(), BCG_));
        for (i, v) in self.voices.iter().enumerate() {
            if v.active {
                let agu = super::tables::bno(v.note);
                let octave = super::tables::bui(v.note);
                j.push_str(&format!("    Voice {}: {}{} vel={} env={} wf={}\n",
                    i, agu, octave, v.velocity, v.env.state_name(), v.osc.waveform.short_name()));
            }
        }
        j
    }

    

    
    fn find_free_voice(&self) -> usize {
        
        for (i, v) in self.voices.iter().enumerate() {
            if !v.active {
                return i;
            }
        }
        
        let mut adj = 0;
        let mut fiu = i32::MAX;
        for (i, v) in self.voices.iter().enumerate() {
            if v.env.state == EnvState::Release && (v.env.level as i32) < fiu {
                fiu = v.env.level as i32;
                adj = i;
            }
        }
        if fiu < i32::MAX {
            return adj;
        }
        
        0
    }
}






pub fn dup(dh: u32) -> u32 {
    (BT_ * dh) / 1000
}


pub fn gsj(jo: u32) -> u32 {
    (jo * 1000) / BT_
}
