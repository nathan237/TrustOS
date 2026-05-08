




use alloc::vec::Vec;
use alloc::vec;
use alloc::string::String;
use alloc::format;

use super::synth::{SynthEngine, BT_, Bq};
use super::pattern::Pattern;






#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlayerState {
    Stopped,
    Playing,
    Paused,
}


pub struct PatternPlayer {
    
    pub state: PlayerState,
    
    pub loop_count: u32,
    
    pub current_loop: u32,
    
    pub current_step: usize,
}

impl PatternPlayer {
    pub fn new() -> Self {
        Self {
            state: PlayerState::Stopped,
            loop_count: 1,
            current_loop: 0,
            current_step: 0,
        }
    }

    
    pub fn qqo(
        &mut self,
        pattern: &Pattern,
        engine: &mut SynthEngine,
        loops: u32,
    ) -> Result<(), &'static str> {
        self.state = PlayerState::Playing;
        self.current_loop = 0;
        self.current_step = 0;
        self.loop_count = if loops == 0 { 1 } else { loops };

        crate::serial_println!("[PLAYER] Playing pattern \"{}\" — {} loops, {} BPM",
            pattern.name_str(), self.loop_count, pattern.bpm);

        for _loop_i in 0..self.loop_count {
            if self.state != PlayerState::Playing {
                break;
            }
            self.current_loop = _loop_i;

            
            let jo = pattern.render(engine);

            
            let duration_ms = pattern.total_duration_ms();
            crate::drivers::hda::bxb(&jo, duration_ms)?;

            self.current_step = pattern.len(); 
        }

        self.state = PlayerState::Stopped;
        self.current_step = 0;
        self.current_loop = 0;
        Ok(())
    }

    
    pub fn play_pattern_visual(
        &mut self,
        pattern: &Pattern,
        engine: &mut SynthEngine,
        loops: u32,
    ) -> Result<(), &'static str> {
        self.state = PlayerState::Playing;
        self.current_loop = 0;
        self.current_step = 0;
        self.loop_count = if loops == 0 { 1 } else { loops };

        let ait = pattern.step_duration_ms();

        crate::serial_println!("[PLAYER] Visual playback \"{}\" — {} loops, {} BPM, {}ms/step",
            pattern.name_str(), self.loop_count, pattern.bpm, ait);

        
        crate::println!();
        crate::bq!(0x00FF88, "▶ ");
        crate::print!("\"{}\" | {} BPM | {} steps | {}",
            pattern.name_str(), pattern.bpm, pattern.len(), pattern.waveform.name());
        crate::println!();

        for loop_i in 0..self.loop_count {
            if self.state != PlayerState::Playing {
                break;
            }
            self.current_loop = loop_i;

            if self.loop_count > 1 {
                crate::bq!(0xAAAAFF, "  Loop {}/{}: ", loop_i + 1, self.loop_count);
            } else {
                crate::print!("  ");
            }

            
            let jo = pattern.render(engine);
            let bpf = pattern.step_duration_samples() as usize;

            
            for (step_i, step) in pattern.steps.iter().enumerate() {
                if self.state != PlayerState::Playing {
                    break;
                }
                self.current_step = step_i;

                
                if step.is_rest() {
                    crate::bq!(0x666666, "·");
                } else {
                    crate::bq!(0x00FF00, "♪");
                }
            }

            
            let duration_ms = pattern.total_duration_ms();
            crate::drivers::hda::bxb(&jo, duration_ms)?;

            crate::println!();
        }

        self.state = PlayerState::Stopped;
        crate::bq!(0x00FF88, "■ ");
        crate::println!("Stopped");
        Ok(())
    }

    
    pub fn stop(&mut self) {
        self.state = PlayerState::Stopped;
        let _ = crate::drivers::hda::stop();
    }

    
    pub fn status(&self) -> String {
        match self.state {
            PlayerState::Stopped => String::from("Player: Stopped\n"),
            PlayerState::Playing => {
                format!("Player: Playing | Step {}/{} | Loop {}/{}\n",
                    self.current_step + 1,
                    0, 
                    self.current_loop + 1,
                    self.loop_count)
            }
            PlayerState::Paused => String::from("Player: Paused\n"),
        }
    }
}
