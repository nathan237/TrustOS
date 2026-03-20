//! WOA — World of Ants game engine
//!
//! Roguelike · 2D Platformer · Turn-Based Combat
//! Internal resolution: 1280×800 (native)

pub mod engine;
pub mod renderer;
pub mod input;
pub mod camera;
pub mod sprites;
pub mod effects;
pub mod militant_sprite_data;

/// Game state machine
#[derive(Clone, Copy, PartialEq)]
pub enum GameState {
    /// Title/menu screen
    Menu,
    /// Exploration / platforming mode
    Playing,
    /// Turn-based combat mode
    Combat,
    /// Pause overlay
    Paused,
}

/// Main entry point — called from shell "woa" command
pub fn run() {
    crate::serial_println!("[WOA] Starting World of Ants...");
    engine::run_game();
}
