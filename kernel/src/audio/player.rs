//! Pattern Player — real-time loop playback engine
//!
//! Renders patterns step by step into the HDA DMA buffer.
//! Supports multi-loop playback and pattern chaining.

use alloc::vec::Vec;
use alloc::vec;
use alloc::string::String;
use alloc::format;

use super::synth::{SynthEngine, SAMPLE_RATE, CHANNELS};
use super::pattern::Pattern;

// ═══════════════════════════════════════════════════════════════════════════════
// Player State
// ═══════════════════════════════════════════════════════════════════════════════

/// Player playback state
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlayerState {
    Stopped,
    Playing,
    Paused,
}

/// The pattern player — manages loop playback of patterns
pub struct PatternPlayer {
    /// Current playback state
    pub state: PlayerState,
    /// Number of loops to play (0 = infinite, N = play N times)
    pub loop_count: u32,
    /// Current loop iteration
    pub current_loop: u32,
    /// Current step index within the pattern
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

    /// Play a pattern for N loops (0 = once for safety in shell blocking mode)
    pub fn play_pattern(
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

            // Render the full pattern as one buffer
            let samples = pattern.render(engine);

            // Play it through HDA
            let duration_ms = pattern.total_duration_ms();
            crate::drivers::hda::write_samples_and_play(&samples, duration_ms)?;

            self.current_step = pattern.len(); // finished this loop
        }

        self.state = PlayerState::Stopped;
        self.current_step = 0;
        self.current_loop = 0;
        Ok(())
    }

    /// Play a pattern with step-by-step visual feedback
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

        let step_ms = pattern.step_duration_ms();

        crate::serial_println!("[PLAYER] Visual playback \"{}\" — {} loops, {} BPM, {}ms/step",
            pattern.name_str(), self.loop_count, pattern.bpm, step_ms);

        // Print header
        crate::println!();
        crate::print_color!(0x00FF88, "▶ ");
        crate::print!("\"{}\" | {} BPM | {} steps | {}",
            pattern.name_str(), pattern.bpm, pattern.len(), pattern.waveform.name());
        crate::println!();

        for loop_i in 0..self.loop_count {
            if self.state != PlayerState::Playing {
                break;
            }
            self.current_loop = loop_i;

            if self.loop_count > 1 {
                crate::print_color!(0xAAAAFF, "  Loop {}/{}: ", loop_i + 1, self.loop_count);
            } else {
                crate::print!("  ");
            }

            // Render the full pattern
            let samples = pattern.render(engine);
            let step_samples = pattern.step_duration_samples() as usize;

            // Play each step with visual indicator
            for (step_i, step) in pattern.steps.iter().enumerate() {
                if self.state != PlayerState::Playing {
                    break;
                }
                self.current_step = step_i;

                // Display current step
                if step.is_rest() {
                    crate::print_color!(0x666666, "·");
                } else {
                    crate::print_color!(0x00FF00, "♪");
                }
            }

            // Play the entire loop audio
            let duration_ms = pattern.total_duration_ms();
            crate::drivers::hda::write_samples_and_play(&samples, duration_ms)?;

            crate::println!();
        }

        self.state = PlayerState::Stopped;
        crate::print_color!(0x00FF88, "■ ");
        crate::println!("Stopped");
        Ok(())
    }

    /// Stop playback
    pub fn stop(&mut self) {
        self.state = PlayerState::Stopped;
        let _ = crate::drivers::hda::stop();
    }

    /// Get player status
    pub fn status(&self) -> String {
        match self.state {
            PlayerState::Stopped => String::from("Player: Stopped\n"),
            PlayerState::Playing => {
                format!("Player: Playing | Step {}/{} | Loop {}/{}\n",
                    self.current_step + 1,
                    0, // we don't store pattern len here
                    self.current_loop + 1,
                    self.loop_count)
            }
            PlayerState::Paused => String::from("Player: Paused\n"),
        }
    }
}
