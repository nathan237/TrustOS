




use alloc::vec::Vec;
use alloc::string::String;
use alloc::format;
use crate::audio::synth::{Waveform, Envelope};

use super::GB_;






#[derive(Debug, Clone, Copy)]
pub struct Note {
    
    pub pitch: u8,
    
    pub velocity: u8,
    
    pub start_tick: u32,
    
    pub duration_ticks: u32,
}

impl Note {
    
    pub fn new(pitch: u8, velocity: u8, start_tick: u32, duration_ticks: u32) -> Self {
        Self {
            pitch: pitch.min(127),
            velocity: velocity.min(127),
            start_tick,
            duration_ticks: duration_ticks.max(1),
        }
    }

    
    pub fn end_tick(&self) -> u32 {
        self.start_tick + self.duration_ticks
    }

    
    pub fn name(&self) -> String {
        let agu = crate::audio::tables::bno(self.pitch);
        let octave = crate::audio::tables::bui(self.pitch);
        format!("{}{}", agu, octave)
    }

    
    pub fn duration_ms(&self, bpm: u32) -> u32 {
        jmk(self.duration_ticks, bpm)
    }

    
    pub fn start_ms(&self, bpm: u32) -> u32 {
        jmk(self.start_tick, bpm)
    }
}






pub struct Track {
    
    name: [u8; 32],
    
    name_len: usize,
    
    pub notes: Vec<Note>,
    
    pub waveform: Waveform,
    
    pub envelope: Envelope,
    
    pub color: u32,
    
    pub armed: bool,
}

impl Track {
    
    pub fn new(name: &str) -> Self {
        let mut bhz = [0u8; 32];
        let bytes = name.as_bytes();
        let len = bytes.len().min(32);
        bhz[..len].copy_from_slice(&bytes[..len]);

        Self {
            name: bhz,
            name_len: len,
            notes: Vec::new(),
            waveform: Waveform::Sine,
            envelope: Envelope::eka(),
            color: 0x4488FF, 
            armed: false,
        }
    }

    
    pub fn name_str(&self) -> &str {
        core::str::from_utf8(&self.name[..self.name_len]).unwrap_or("???")
    }

    
    pub fn add_note(&mut self, note: Note) {
        
        let pos = self.notes.partition_point(|ae| ae.start_tick < note.start_tick);
        self.notes.insert(pos, note);
    }

    
    pub fn remove_note(&mut self, index: usize) -> Option<Note> {
        if index < self.notes.len() {
            Some(self.notes.remove(index))
        } else {
            None
        }
    }

    
    pub fn qtr(&mut self, start: u32, end: u32) {
        self.notes.retain(|ae| ae.start_tick < start || ae.start_tick >= end);
    }

    
    pub fn notes_at_tick(&self, tick: u32) -> Vec<&Note> {
        self.notes.iter()
            .filter(|ae| ae.start_tick <= tick && tick < ae.end_tick())
            .collect()
    }

    
    pub fn qpr(&self, start: u32, end: u32) -> Vec<&Note> {
        self.notes.iter()
            .filter(|ae| ae.start_tick < end && ae.end_tick() > start)
            .collect()
    }

    
    pub fn end_tick(&self) -> u32 {
        self.notes.iter().map(|ae| ae.end_tick()).max().unwrap_or(0)
    }

    
    pub fn nkw(&self) -> usize {
        self.notes.len()
    }

    
    pub fn clear(&mut self) {
        self.notes.clear();
    }

    
    pub fn qrm(&mut self, grid_ticks: u32) {
        if grid_ticks == 0 { return; }
        for note in &mut self.notes {
            let bix = note.start_tick % grid_ticks;
            if bix > grid_ticks / 2 {
                note.start_tick += grid_ticks - bix;
            } else {
                note.start_tick -= bix;
            }
            
            let ekx = note.duration_ticks % grid_ticks;
            if ekx > grid_ticks / 2 {
                note.duration_ticks += grid_ticks - ekx;
            } else if note.duration_ticks > ekx {
                note.duration_ticks -= ekx;
            }
            if note.duration_ticks == 0 {
                note.duration_ticks = grid_ticks;
            }
        }
    }

    
    pub fn transpose(&mut self, semitones: i8) {
        for note in &mut self.notes {
            let njm = note.pitch as i16 + semitones as i16;
            note.pitch = njm.clamp(0, 127) as u8;
        }
    }

    
    pub fn no(&mut self, gx: i32) {
        for note in &mut self.notes {
            let gji = note.start_tick as i64 + gx as i64;
            note.start_tick = gji.max(0) as u32;
        }
        
        self.notes.sort_by_key(|ae| ae.start_tick);
    }
}






pub struct Project {
    
    name: [u8; 64],
    
    name_len: usize,
    
    pub tracks: Vec<Track>,
    
    pub bpm: u16,
    
    pub time_sig_num: u8,
    
    pub time_sig_den: u8,
}


static U_: [u32; 16] = [
    0x4488FF, 
    0xFF4444, 
    0x44FF44, 
    0xFFAA00, 
    0xAA44FF, 
    0x44FFFF, 
    0xFF44AA, 
    0xFFFF44, 
    0x88FF88, 
    0xFF8844, 
    0x8888FF, 
    0xFF88FF, 
    0x44FFAA, 
    0xAAAAFF, 
    0xFFAA88, 
    0x88FFFF, 
];

impl Project {
    
    pub fn new(name: &str, bpm: u16) -> Self {
        let mut bhz = [0u8; 64];
        let bytes = name.as_bytes();
        let len = bytes.len().min(64);
        bhz[..len].copy_from_slice(&bytes[..len]);

        Self {
            name: bhz,
            name_len: len,
            tracks: Vec::new(),
            bpm,
            time_sig_num: 4,
            time_sig_den: 4,
        }
    }

    
    pub fn name_str(&self) -> &str {
        core::str::from_utf8(&self.name[..self.name_len]).unwrap_or("???")
    }

    
    pub fn add_track(&mut self, name: &str) -> Result<usize, &'static str> {
        if self.tracks.len() >= GB_ {
            return Err("Maximum tracks reached");
        }
        let idx = self.tracks.len();
        let mut track = Track::new(name);
        track.color = U_[idx % U_.len()];
        self.tracks.push(track);
        Ok(idx)
    }

    
    pub fn remove_track(&mut self, index: usize) -> Result<(), &'static str> {
        if index >= self.tracks.len() {
            return Err("Invalid track index");
        }
        self.tracks.remove(index);
        Ok(())
    }

    
    pub fn length_ticks(&self) -> u32 {
        self.tracks.iter().map(|t| t.end_tick()).max().unwrap_or(0)
    }

    
    pub fn qni(&self) -> u32 {
        let gx = self.length_ticks();
        let ask = super::AF_ * self.time_sig_num as u32;
        (gx + ask - 1) / ask
    }

    
    pub fn pmt(&self) -> usize {
        self.tracks.len()
    }
}






pub fn jmk(gx: u32, bpm: u32) -> u32 {
    if bpm == 0 { return 0; }
    
    (gx as u64 * 60_000 / (bpm as u64 * super::AF_ as u64)) as u32
}


pub fn duq(dh: u32, bpm: u32) -> u32 {
    if dh == 0 { return 0; }
    
    (dh as u64 * bpm as u64 * super::AF_ as u64 / 60_000) as u32
}


pub fn fcy(gx: u32, bpm: u32) -> u32 {
    if bpm == 0 { return 0; }
    
    (gx as u64 * super::BT_ as u64 * 60 / (bpm as u64 * super::AF_ as u64)) as u32
}


pub fn qup(jo: u32, bpm: u32) -> u32 {
    if jo == 0 { return 0; }
    (jo as u64 * bpm as u64 * super::AF_ as u64 / (super::BT_ as u64 * 60)) as u32
}
