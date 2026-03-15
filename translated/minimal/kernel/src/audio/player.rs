




use alloc::vec::Vec;
use alloc::vec;
use alloc::string::String;
use alloc::format;

use super::synth::{SynthEngine, BR_, Dv};
use super::pattern::Pattern;






#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlayerState {
    Af,
    Ce,
    Cl,
}


pub struct PatternPlayer {
    
    pub g: PlayerState,
    
    pub dsx: u32,
    
    pub fgh: u32,
    
    pub aop: usize,
}

impl PatternPlayer {
    pub fn new() -> Self {
        Self {
            g: PlayerState::Af,
            dsx: 1,
            fgh: 0,
            aop: 0,
        }
    }

    
    pub fn zfj(
        &mut self,
        pattern: &Pattern,
        engine: &mut SynthEngine,
        bkh: u32,
    ) -> Result<(), &'static str> {
        self.g = PlayerState::Ce;
        self.fgh = 0;
        self.aop = 0;
        self.dsx = if bkh == 0 { 1 } else { bkh };

        crate::serial_println!("[PLAYER] Playing pattern \"{}\" — {} loops, {} BPM",
            pattern.amj(), self.dsx, pattern.kz);

        for qck in 0..self.dsx {
            if self.g != PlayerState::Ce {
                break;
            }
            self.fgh = qck;

            
            let un = pattern.tj(engine);

            
            let uk = pattern.ief();
            crate::drivers::hda::ele(&un, uk)?;

            self.aop = pattern.len(); 
        }

        self.g = PlayerState::Af;
        self.aop = 0;
        self.fgh = 0;
        Ok(())
    }

    
    pub fn vja(
        &mut self,
        pattern: &Pattern,
        engine: &mut SynthEngine,
        bkh: u32,
    ) -> Result<(), &'static str> {
        self.g = PlayerState::Ce;
        self.fgh = 0;
        self.aop = 0;
        self.dsx = if bkh == 0 { 1 } else { bkh };

        let bop = pattern.dwh();

        crate::serial_println!("[PLAYER] Visual playback \"{}\" — {} loops, {} BPM, {}ms/step",
            pattern.amj(), self.dsx, pattern.kz, bop);

        
        crate::println!();
        crate::gr!(0x00FF88, "▶ ");
        crate::print!("\"{}\" | {} BPM | {} steps | {}",
            pattern.amj(), pattern.kz, pattern.len(), pattern.ve.j());
        crate::println!();

        for okk in 0..self.dsx {
            if self.g != PlayerState::Ce {
                break;
            }
            self.fgh = okk;

            if self.dsx > 1 {
                crate::gr!(0xAAAAFF, "  Loop {}/{}: ", okk + 1, self.dsx);
            } else {
                crate::print!("  ");
            }

            
            let un = pattern.tj(engine);
            let dwk = pattern.pot() as usize;

            
            for (wub, gu) in pattern.au.iter().cf() {
                if self.g != PlayerState::Ce {
                    break;
                }
                self.aop = wub;

                
                if gu.jbs() {
                    crate::gr!(0x666666, "·");
                } else {
                    crate::gr!(0x00FF00, "♪");
                }
            }

            
            let uk = pattern.ief();
            crate::drivers::hda::ele(&un, uk)?;

            crate::println!();
        }

        self.g = PlayerState::Af;
        crate::gr!(0x00FF88, "■ ");
        crate::println!("Stopped");
        Ok(())
    }

    
    pub fn qg(&mut self) {
        self.g = PlayerState::Af;
        let _ = crate::drivers::hda::qg();
    }

    
    pub fn status(&self) -> String {
        match self.g {
            PlayerState::Af => String::from("Player: Stopped\n"),
            PlayerState::Ce => {
                format!("Player: Playing | Step {}/{} | Loop {}/{}\n",
                    self.aop + 1,
                    0, 
                    self.fgh + 1,
                    self.dsx)
            }
            PlayerState::Cl => String::from("Player: Paused\n"),
        }
    }
}
