







use alloc::vec::Vec;
use alloc::vec;
use alloc::string::String;
use alloc::format;

use super::synth::{Waveform, Envelope, SynthEngine, BT_, Bq};
use super::tables;






pub const BBX_: usize = 16;

pub const GA_: usize = 64;

pub const DMN_: u16 = 120;

pub const BIM_: u32 = 4;






#[derive(Debug, Clone, Copy)]
pub struct Step {
    
    pub note: u8,
    
    pub velocity: u8,
    
    pub waveform: Option<Waveform>,
}

impl Step {
    
    pub fn ef() -> Self {
        Self { note: 255, velocity: 0, waveform: None }
    }

    
    pub fn note(midi_note: u8) -> Self {
        Self { note: midi_note, velocity: 100, waveform: None }
    }

    
    pub fn dvh(midi_note: u8, velocity: u8) -> Self {
        Self { note: midi_note, velocity, waveform: None }
    }

    
    pub fn qpq(midi_note: u8, velocity: u8, aal: Waveform) -> Self {
        Self { note: midi_note, velocity, waveform: Some(aal) }
    }

    
    pub fn is_rest(&self) -> bool {
        self.note == 255 || self.velocity == 0
    }

    
    pub fn display(&self) -> String {
        if self.is_rest() {
            String::from("--")
        } else {
            let name = tables::bno(self.note);
            let amb = tables::bui(self.note);
            format!("{}{}", name, amb)
        }
    }

    
    pub fn wave_display(&self) -> &'static str {
        match self.waveform {
            Some(aal) => aal.short_name(),
            None => "..",
        }
    }
}






#[derive(Clone)]
pub struct Pattern {
    
    pub name: [u8; 16],
    
    pub name_len: usize,
    
    pub steps: Vec<Step>,
    
    pub bpm: u16,
    
    pub waveform: Waveform,
    
    pub envelope: Envelope,
}

impl Pattern {
    
    pub fn new(name: &str, num_steps: usize, bpm: u16) -> Self {
        let ae = num_steps.min(GA_).max(1);
        let mut bhz = [0u8; 16];
        let agt = name.as_bytes();
        let len = agt.len().min(16);
        bhz[..len].copy_from_slice(&agt[..len]);

        Self {
            name: bhz,
            name_len: len,
            steps: vec![Step::ef(); ae],
            bpm,
            waveform: Waveform::Square,
            envelope: Envelope::dwp(),
        }
    }

    
    pub fn name_str(&self) -> &str {
        core::str::from_utf8(&self.name[..self.name_len]).unwrap_or("???")
    }

    
    pub fn len(&self) -> usize {
        self.steps.len()
    }

    
    pub fn qwm(&mut self, idx: usize, step: Step) {
        if idx < self.steps.len() {
            self.steps[idx] = step;
        }
    }

    
    pub fn set_note(&mut self, idx: usize, agu: &str) -> Result<(), &'static str> {
        if idx >= self.steps.len() {
            return Err("Step index out of range");
        }
        if agu == "--" || agu == "." || agu.is_empty() {
            self.steps[idx] = Step::ef();
            return Ok(());
        }
        let aad = tables::cnh(agu)
            .ok_or("Invalid note name")?;
        self.steps[idx] = Step::note(aad);
        Ok(())
    }

    
    pub fn step_duration_samples(&self) -> u32 {
        
        (60 * BT_) / (self.bpm as u32 * BIM_)
    }

    
    pub fn step_duration_ms(&self) -> u32 {
        (60_000) / (self.bpm as u32 * BIM_)
    }

    
    pub fn total_duration_ms(&self) -> u32 {
        self.step_duration_ms() * self.steps.len() as u32
    }

    
    pub fn render(&self, engine: &mut SynthEngine) -> Vec<i16> {
        let bpf = self.step_duration_samples() as usize;
        let aai = bpf * self.steps.len();
        let mut buffer = vec![0i16; aai * Bq as usize];

        
        let oki = engine.waveform;
        let okg = engine.envelope;
        engine.envelope = self.envelope;

        for (i, step) in self.steps.iter().enumerate() {
            if step.is_rest() {
                
                continue;
            }

            
            let aal = step.waveform.unwrap_or(self.waveform);
            engine.set_waveform(aal);

            
            engine.note_on(step.note, step.velocity);

            
            let offset = i * bpf * Bq as usize;
            let cul = &mut buffer[offset..offset + bpf * Bq as usize];
            engine.render(cul, bpf);

            
            engine.note_off(step.note);
        }

        
        engine.set_waveform(oki);
        engine.envelope = okg;

        buffer
    }

    
    pub fn display(&self) -> String {
        let mut j = String::new();
        j.push_str(&format!("Pattern: \"{}\" | {} steps | {} BPM | {} | {}ms/step\n",
            self.name_str(), self.steps.len(), self.bpm,
            self.waveform.name(), self.step_duration_ms()));
        j.push_str(&format!("Total duration: {}ms\n\n", self.total_duration_ms()));

        
        j.push_str(" Step: ");
        for i in 0..self.steps.len() {
            j.push_str(&format!("{:>3}", i + 1));
        }
        j.push('\n');

        
        j.push_str(" Note: ");
        for step in &self.steps {
            j.push_str(&format!("{:>3}", step.display()));
        }
        j.push('\n');

        
        j.push_str("  Vel: ");
        for step in &self.steps {
            if step.is_rest() {
                j.push_str(" --");
            } else {
                j.push_str(&format!("{:>3}", step.velocity));
            }
        }
        j.push('\n');

        
        j.push_str(" Wave: ");
        for step in &self.steps {
            j.push_str(&format!("{:>3}", step.wave_display()));
        }
        j.push('\n');

        j
    }
}






pub struct PatternBank {
    pub patterns: Vec<Pattern>,
}

impl PatternBank {
    pub fn new() -> Self {
        Self { patterns: Vec::new() }
    }

    
    pub fn add(&mut self, pattern: Pattern) -> Result<usize, &'static str> {
        if self.patterns.len() >= BBX_ {
            return Err("Maximum patterns reached (16)");
        }
        
        let name = pattern.name_str();
        for aa in &self.patterns {
            if aa.name_str() == name {
                return Err("Pattern name already exists");
            }
        }
        self.patterns.push(pattern);
        Ok(self.patterns.len() - 1)
    }

    
    pub fn find(&self, name: &str) -> Option<usize> {
        self.patterns.iter().position(|aa| aa.name_str() == name)
    }

    
    pub fn get(&self, idx: usize) -> Option<&Pattern> {
        self.patterns.get(idx)
    }

    
    pub fn get_mut(&mut self, idx: usize) -> Option<&mut Pattern> {
        self.patterns.get_mut(idx)
    }

    
    pub fn get_by_name(&self, name: &str) -> Option<&Pattern> {
        self.find(name).and_then(|i| self.get(i))
    }

    
    pub fn get_by_name_mut(&mut self, name: &str) -> Option<&mut Pattern> {
        let idx = self.find(name)?;
        self.get_mut(idx)
    }

    
    pub fn remove(&mut self, name: &str) -> Result<(), &'static str> {
        let idx = self.find(name).ok_or("Pattern not found")?;
        self.patterns.remove(idx);
        Ok(())
    }

    
    pub fn list(&self) -> String {
        if self.patterns.is_empty() {
            return String::from("No patterns. Use 'synth pattern new <name>' to create one.\n");
        }
        let mut j = String::new();
        j.push_str(&format!("Patterns ({}/{}):\n", self.patterns.len(), BBX_));
        for (i, aa) in self.patterns.iter().enumerate() {
            j.push_str(&format!("  [{}] \"{}\" — {} steps, {} BPM, {}\n",
                i, aa.name_str(), aa.steps.len(), aa.bpm, aa.waveform.name()));
        }
        j
    }

    
    pub fn load_presets(&mut self) {
        
        let mut arp = Pattern::new("arp", 16, 140);
        arp.waveform = Waveform::Sine;
        arp.envelope = Envelope::dwp();
        let jxq = [60, 63, 67, 72, 67, 63, 60, 63, 67, 72, 67, 63, 60, 63, 67, 72]; 
        for (i, &ae) in jxq.iter().enumerate() {
            arp.steps[i] = Step::dvh(ae, 90);
        }
        let _ = self.add(arp);

        
        let mut fco = Pattern::new("techno", 16, 128);
        fco.waveform = Waveform::Sine;
        fco.envelope = Envelope::new(1, 80, 0, 30);
        
        for i in (0..16).step_by(4) {
            fco.steps[i] = Step::dvh(36, 127); 
        }
        let _ = self.add(fco);

        
        let mut bass = Pattern::new("bass", 16, 120);
        bass.waveform = Waveform::Sawtooth;
        bass.envelope = Envelope::new(5, 100, 60, 50);
        let kak: [u8; 16] = [36, 255, 36, 36, 39, 255, 39, 36, 43, 255, 43, 43, 41, 255, 41, 36];
        for (i, &ae) in kak.iter().enumerate() {
            if ae != 255 {
                bass.steps[i] = Step::dvh(ae, 100);
            }
        }
        let _ = self.add(bass);

        
        let mut ehw = Pattern::new("chiptune", 16, 150);
        ehw.waveform = Waveform::Square;
        ehw.envelope = Envelope::new(2, 30, 80, 20);
        let kke: [u8; 16] = [72, 74, 76, 72, 79, 255, 79, 255, 76, 74, 72, 74, 76, 72, 71, 255];
        for (i, &ae) in kke.iter().enumerate() {
            if ae != 255 {
                ehw.steps[i] = Step::dvh(ae, 110);
            }
        }
        let _ = self.add(ehw);

        
        let mut pad = Pattern::new("pad", 8, 80);
        pad.waveform = Waveform::Triangle;
        pad.envelope = Envelope::pad();
        
        let npf: [u8; 8] = [60, 255, 64, 255, 67, 255, 72, 255]; 
        for (i, &ae) in npf.iter().enumerate() {
            if ae != 255 {
                pad.steps[i] = Step::dvh(ae, 80);
            }
        }
        let _ = self.add(pad);
    }
}
