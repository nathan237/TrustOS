




use alloc::vec::Vec;
use alloc::format;
use alloc::string::String;
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use super::track::{Note, hse};
use super::keyboard_midi;
use super::{Mf, FR_, Hi, AE_};






#[derive(Debug, Clone, Copy)]
struct Bbl {
    
    jb: u8,
    
    qm: u8,
    
    gtc: u32,
}


pub struct RecordSession {
    
    pub ts: Vec<Note>,
    
    elv: Vec<Bbl>,
    
    poe: u32,
    
    kz: u32,
    
    pub hwh: u32,
    
    pub mhl: u32,
}

impl RecordSession {
    
    pub fn new(kz: u32, vb: u32) -> Self {
        Self {
            ts: Vec::new(),
            elv: Vec::new(),
            poe: crate::time::lc() as u32,
            kz,
            hwh: AE_ / 4, 
            mhl: vb,
        }
    }

    
    pub fn dtq(&mut self, scancode: u8) {
        if let Some(jb) = keyboard_midi::hyv(scancode) {
            
            if self.elv.iter().any(|bo| bo.jb == jb) {
                return;
            }

            let oz = self.oz();
            let qm = keyboard_midi::hlm();

            self.elv.push(Bbl {
                jb,
                qm,
                gtc: oz,
            });

            
            let _ = crate::audio::owb(jb, qm, 100);
        }
    }

    
    pub fn djx(&mut self, scancode: u8) {
        if let Some(jb) = keyboard_midi::hyv(scancode) {
            let oz = self.oz();

            
            if let Some(u) = self.elv.iter().qf(|bo| bo.jb == jb) {
                let gh = self.elv.remove(u);
                let uk = oz.ao(gh.gtc).am(10);

                
                let vb = self.mhl + hse(gh.gtc, self.kz);
                let bbn = hse(uk, self.kz).am(1);

                
                let (vb, bbn) = if self.hwh > 0 {
                    let fm = self.hwh;
                    let wqg = ((vb + fm / 2) / fm) * fm;
                    let wqe = ((bbn + fm / 2) / fm) * fm;
                    (wqg, wqe.am(fm))
                } else {
                    (vb, bbn)
                };

                self.ts.push(Note::new(jb, gh.qm, vb, bbn));
            }
        }
    }

    
    fn oz(&self) -> u32 {
        let iu = crate::time::lc() as u32;
        iu.ao(self.poe)
    }

    
    pub fn bqs(&mut self) -> Vec<Note> {
        let oz = self.oz();

        
        for gh in self.elv.bbk(..) {
            let uk = oz.ao(gh.gtc).am(10);
            let vb = self.mhl + hse(gh.gtc, self.kz);
            let bbn = hse(uk, self.kz).am(1);

            self.ts.push(Note::new(gh.jb, gh.qm, vb, bbn));
        }

        
        self.ts.bxf(|bo| bo.vb);

        core::mem::take(&mut self.ts)
    }

    
    pub fn uve(&self) -> usize {
        self.ts.len()
    }

    
    pub fn gxu(&self) -> usize {
        self.elv.len()
    }

    
    pub fn uk(&self) -> u32 {
        self.oz()
    }

    
    pub fn status(&self) -> String {
        let ez = self.oz();
        let tv = ez / 1000;
        let jn = ez % 1000;
        format!("REC {:02}:{:02}.{:03} | Notes: {} | Active: {} | Quantize: {}",
            tv / 60, tv % 60, jn,
            self.ts.len(), self.elv.len(),
            if self.hwh > 0 {
                format!("1/{}", AE_ * 4 / self.hwh)
            } else {
                String::from("off")
            }
        )
    }
}



pub fn pas(zx: usize) -> Result<usize, &'static str> {
    super::aqz()?;

    let kz = Hi.load(Ordering::Relaxed);
    let vb = FR_.load(Ordering::Relaxed);

    Mf.store(true, Ordering::Relaxed);

    crate::println!("Recording on track {}...", zx);
    crate::println!("Play notes on keyboard. Press [Esc] to stop recording.\n");
    crate::println!("{}", keyboard_midi::nlx());

    let mut he = RecordSession::new(kz, vb);

    
    loop {
        if !Mf.load(Ordering::Relaxed) {
            break; 
        }

        
        if let Some(scancode) = crate::keyboard::xw() {
            
            if scancode == 0x01 {
                break;
            }

            
            match scancode {
                0x3B => { 
                    let bvq = keyboard_midi::uwx();
                    crate::println!("Octave: {:+}", bvq);
                    continue;
                }
                0x3C => { 
                    let bvq = keyboard_midi::uwy();
                    crate::println!("Octave: {:+}", bvq);
                    continue;
                }
                0x3D => { 
                    let p = keyboard_midi::hlm();
                    keyboard_midi::pjj(p.ao(10));
                    crate::println!("Velocity: {}", keyboard_midi::hlm());
                    continue;
                }
                0x3E => { 
                    let p = keyboard_midi::hlm();
                    keyboard_midi::pjj((p + 10).v(127));
                    crate::println!("Velocity: {}", keyboard_midi::hlm());
                    continue;
                }
                _ => {}
            }

            let bep = scancode & 0x80 != 0;
            let bs = scancode & 0x7F;

            if bep {
                he.djx(bs);
            } else {
                he.dtq(bs);
            }
        }

        
        
        for _ in 0..1000 {
            core::hint::hc();
        }
    }

    Mf.store(false, Ordering::Relaxed);

    
    let ts = he.bqs();
    let az = ts.len();

    if az > 0 {
        let mut nv = super::Fc.lock();
        let nv = nv.as_mut().ok_or("No project")?;
        let track = nv.af.ds(zx).ok_or("Invalid track index")?;

        for jp in ts {
            track.axn(jp);
        }

        crate::println!("\nRecording complete: {} notes added to track {}", az, zx);
    } else {
        crate::println!("\nRecording complete: no notes recorded");
    }

    Ok(az)
}
