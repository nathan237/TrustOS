





use alloc::vec::Vec;
use alloc::vec;

use crate::audio::synth::{SynthEngine, Envelope, BT_};
use super::track::{Project, Track, Note, fcy};






#[derive(Debug, Clone, Copy)]
pub struct MixerChannel {
    
    pub volume: u8,
    
    pub pan: i8,
    
    pub muted: bool,
    
    pub solo: bool,
}

impl MixerChannel {
    pub fn new() -> Self {
        Self {
            volume: 200,
            pan: 0,
            muted: false,
            solo: false,
        }
    }
}






pub struct Mixer {
    
    pub channels: Vec<MixerChannel>,
    
    pub master_volume: u8,
}

impl Mixer {
    
    pub fn new(num_channels: usize) -> Self {
        Self {
            channels: vec![MixerChannel::new(); num_channels],
            master_volume: 220,
        }
    }

    
    pub fn set_volume(&mut self, ch: usize, volume: u8) -> Result<(), &'static str> {
        let channel = self.channels.get_mut(ch).ok_or("Invalid channel")?;
        channel.volume = volume;
        Ok(())
    }

    
    pub fn set_pan(&mut self, ch: usize, pan: i8) -> Result<(), &'static str> {
        let channel = self.channels.get_mut(ch).ok_or("Invalid channel")?;
        channel.pan = pan.clamp(-100, 100);
        Ok(())
    }

    
    pub fn toggle_mute(&mut self, ch: usize) -> Result<bool, &'static str> {
        let channel = self.channels.get_mut(ch).ok_or("Invalid channel")?;
        channel.muted = !channel.muted;
        Ok(channel.muted)
    }

    
    pub fn toggle_solo(&mut self, ch: usize) -> Result<bool, &'static str> {
        let channel = self.channels.get_mut(ch).ok_or("Invalid channel")?;
        channel.solo = !channel.solo;
        Ok(channel.solo)
    }

    
    pub fn has_solo(&self) -> bool {
        self.channels.iter().any(|c| c.solo)
    }

    
    pub fn is_audible(&self, ch: usize) -> bool {
        if let Some(channel) = self.channels.get(ch) {
            if channel.muted { return false; }
            if self.has_solo() { return channel.solo; }
            true
        } else {
            false
        }
    }

    
    
    pub fn apply_channel(&self, ch: usize, left: i32, right: i32) -> (i32, i32) {
        if let Some(channel) = self.channels.get(ch) {
            let vd = channel.volume as i32;
            
            
            
            
            let pan = channel.pan as i32;
            let mxt = (100 - pan).clamp(0, 200);
            let ogz = (100 + pan).clamp(0, 200);

            let l = left * vd / 255 * mxt / 100;
            let r = right * vd / 255 * ogz / 100;
            (l, r)
        } else {
            (left, right)
        }
    }
}






fn ofu(track: &Track, bpm: u32, start_tick: u32, aai: usize) -> Vec<i32> {
    let mut buffer = vec![0i32; aai];

    if track.notes.is_empty() {
        return buffer;
    }

    
    let mut engine = SynthEngine::new();
    engine.set_waveform(track.waveform);
    engine.envelope = track.envelope;

    
    
    struct Tz {
        sample_pos: usize,
        pitch: u8,
        velocity: u8,
        is_on: bool,
    }

    let mut events: Vec<Tz> = Vec::new();

    for note in &track.notes {
        if note.end_tick() <= start_tick {
            continue; 
        }

        let iqy = if note.start_tick >= start_tick {
            fcy(note.start_tick - start_tick, bpm) as usize
        } else {
            0 
        };

        let iqv = fcy(
            note.end_tick().saturating_sub(start_tick), bpm
        ) as usize;

        if iqy < aai {
            events.push(Tz {
                sample_pos: iqy,
                pitch: note.pitch,
                velocity: note.velocity,
                is_on: true,
            });
        }

        if iqv < aai {
            events.push(Tz {
                sample_pos: iqv,
                pitch: note.pitch,
                velocity: note.velocity,
                is_on: false,
            });
        }
    }

    
    events.sort_by_key(|e| e.sample_pos);

    
    let mut cjb = 0;
    let mut gyf = vec![0i16; 2]; 

    for sample in 0..aai {
        
        while cjb < events.len() && events[cjb].sample_pos <= sample {
            let rt = &events[cjb];
            if rt.is_on {
                engine.note_on(rt.pitch, rt.velocity);
            } else {
                engine.note_off(rt.pitch);
            }
            cjb += 1;
        }

        
        engine.render(&mut gyf, 1);
        
        buffer[sample] = (gyf[0] as i32 + gyf[1] as i32) / 2;
    }

    buffer
}



pub fn izw(project: &Project, mixer: &Mixer, bpm: u32, start_tick: u32) -> Vec<i16> {
    if project.tracks.is_empty() {
        return Vec::new();
    }

    
    let total_ticks = project.length_ticks();
    if total_ticks <= start_tick {
        return Vec::new();
    }

    let oft = total_ticks - start_tick;
    let aai = fcy(oft, bpm) as usize;

    if aai == 0 {
        return Vec::new();
    }

    
    let pmq: Vec<Vec<i32>> = project.tracks.iter()
        .map(|track| ofu(track, bpm, start_tick, aai))
        .collect();

    
    let mut output = vec![0i16; aai * 2];

    for sample in 0..aai {
        let mut dak: i32 = 0;
        let mut ddp: i32 = 0;

        for (cuy, track_buf) in pmq.iter().enumerate() {
            if !mixer.is_audible(cuy) {
                continue;
            }

            let ioi = track_buf[sample];
            let (l, r) = mixer.apply_channel(cuy, ioi, ioi);
            dak += l;
            ddp += r;
        }

        
        dak = dak * mixer.master_volume as i32 / 255;
        ddp = ddp * mixer.master_volume as i32 / 255;

        
        dak = jgz(dak);
        ddp = jgz(ddp);

        output[sample * 2] = dak.clamp(-32767, 32767) as i16;
        output[sample * 2 + 1] = ddp.clamp(-32767, 32767) as i16;
    }

    output
}


fn jgz(sample: i32) -> i32 {
    const Ks: i32 = 24000;
    if sample > Ks {
        let dox = sample - Ks;
        let qv = dox * 8000 / (dox + 8000); 
        Ks + qv
    } else if sample < -Ks {
        let dox = -sample - Ks;
        let qv = dox * 8000 / (dox + 8000);
        -(Ks + qv)
    } else {
        sample
    }
}
