




use alloc::vec::Vec;
use alloc::format;
use alloc::string::String;
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use super::track::{Note, duq};
use super::keyboard_midi;
use super::{Fb, GG_, Df, AF_};






#[derive(Debug, Clone, Copy)]
struct We {
    
    pitch: u8,
    
    velocity: u8,
    
    start_ms: u32,
}


pub struct RecordSession {
    
    pub notes: Vec<Note>,
    
    active_notes: Vec<We>,
    
    start_time_ms: u32,
    
    bpm: u32,
    
    pub quantize_ticks: u32,
    
    pub start_tick_offset: u32,
}

impl RecordSession {
    
    pub fn new(bpm: u32, start_tick: u32) -> Self {
        Self {
            notes: Vec::new(),
            active_notes: Vec::new(),
            start_time_ms: crate::time::uptime_ms() as u32,
            bpm,
            quantize_ticks: AF_ / 4, 
            start_tick_offset: start_tick,
        }
    }

    
    pub fn note_on(&mut self, scancode: u8) {
        if let Some(pitch) = keyboard_midi::dyl(scancode) {
            
            if self.active_notes.iter().any(|ae| ae.pitch == pitch) {
                return;
            }

            let elapsed_ms = self.elapsed_ms();
            let velocity = keyboard_midi::dqs();

            self.active_notes.push(We {
                pitch,
                velocity,
                start_ms: elapsed_ms,
            });

            
            let _ = crate::audio::ive(pitch, velocity, 100);
        }
    }

    
    pub fn note_off(&mut self, scancode: u8) {
        if let Some(pitch) = keyboard_midi::dyl(scancode) {
            let elapsed_ms = self.elapsed_ms();

            
            if let Some(pos) = self.active_notes.iter().position(|ae| ae.pitch == pitch) {
                let active = self.active_notes.remove(pos);
                let duration_ms = elapsed_ms.saturating_sub(active.start_ms).max(10);

                
                let start_tick = self.start_tick_offset + duq(active.start_ms, self.bpm);
                let duration_ticks = duq(duration_ms, self.bpm).max(1);

                
                let (start_tick, duration_ticks) = if self.quantize_ticks > 0 {
                    let q = self.quantize_ticks;
                    let ouh = ((start_tick + q / 2) / q) * q;
                    let ouf = ((duration_ticks + q / 2) / q) * q;
                    (ouh, ouf.max(q))
                } else {
                    (start_tick, duration_ticks)
                };

                self.notes.push(Note::new(pitch, active.velocity, start_tick, duration_ticks));
            }
        }
    }

    
    fn elapsed_ms(&self) -> u32 {
        let cy = crate::time::uptime_ms() as u32;
        cy.saturating_sub(self.start_time_ms)
    }

    
    pub fn finalize(&mut self) -> Vec<Note> {
        let elapsed_ms = self.elapsed_ms();

        
        for active in self.active_notes.drain(..) {
            let duration_ms = elapsed_ms.saturating_sub(active.start_ms).max(10);
            let start_tick = self.start_tick_offset + duq(active.start_ms, self.bpm);
            let duration_ticks = duq(duration_ms, self.bpm).max(1);

            self.notes.push(Note::new(active.pitch, active.velocity, start_tick, duration_ticks));
        }

        
        self.notes.sort_by_key(|ae| ae.start_tick);

        core::mem::take(&mut self.notes)
    }

    
    pub fn nkw(&self) -> usize {
        self.notes.len()
    }

    
    pub fn active_count(&self) -> usize {
        self.active_notes.len()
    }

    
    pub fn duration_ms(&self) -> u32 {
        self.elapsed_ms()
    }

    
    pub fn status(&self) -> String {
        let bb = self.elapsed_ms();
        let im = bb / 1000;
        let dh = bb % 1000;
        format!("REC {:02}:{:02}.{:03} | Notes: {} | Active: {} | Quantize: {}",
            im / 60, im % 60, dh,
            self.notes.len(), self.active_notes.len(),
            if self.quantize_ticks > 0 {
                format!("1/{}", AF_ * 4 / self.quantize_ticks)
            } else {
                String::from("off")
            }
        )
    }
}



pub fn iyu(mp: usize) -> Result<usize, &'static str> {
    super::ensure_init()?;

    let bpm = Df.load(Ordering::Relaxed);
    let start_tick = GG_.load(Ordering::Relaxed);

    Fb.store(true, Ordering::Relaxed);

    crate::println!("Recording on track {}...", mp);
    crate::println!("Play notes on keyboard. Press [Esc] to stop recording.\n");
    crate::println!("{}", keyboard_midi::hsp());

    let mut by = RecordSession::new(bpm, start_tick);

    
    loop {
        if !Fb.load(Ordering::Relaxed) {
            break; 
        }

        
        if let Some(scancode) = crate::keyboard::kr() {
            
            if scancode == 0x01 {
                break;
            }

            
            match scancode {
                0x3B => { 
                    let amb = keyboard_midi::nme();
                    crate::println!("Octave: {:+}", amb);
                    continue;
                }
                0x3C => { 
                    let amb = keyboard_midi::nmf();
                    crate::println!("Octave: {:+}", amb);
                    continue;
                }
                0x3D => { 
                    let v = keyboard_midi::dqs();
                    keyboard_midi::jfn(v.saturating_sub(10));
                    crate::println!("Velocity: {}", keyboard_midi::dqs());
                    continue;
                }
                0x3E => { 
                    let v = keyboard_midi::dqs();
                    keyboard_midi::jfn((v + 10).min(127));
                    crate::println!("Velocity: {}", keyboard_midi::dqs());
                    continue;
                }
                _ => {}
            }

            let adx = scancode & 0x80 != 0;
            let key = scancode & 0x7F;

            if adx {
                by.note_off(key);
            } else {
                by.note_on(key);
            }
        }

        
        
        for _ in 0..1000 {
            core::hint::spin_loop();
        }
    }

    Fb.store(false, Ordering::Relaxed);

    
    let notes = by.finalize();
    let count = notes.len();

    if count > 0 {
        let mut project = super::Ce.lock();
        let project = project.as_mut().ok_or("No project")?;
        let track = project.tracks.get_mut(mp).ok_or("Invalid track index")?;

        for note in notes {
            track.add_note(note);
        }

        crate::println!("\nRecording complete: {} notes added to track {}", count, mp);
    } else {
        crate::println!("\nRecording complete: no notes recorded");
    }

    Ok(count)
}
