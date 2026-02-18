//! TrustMario64 — TAS Engine (Tool-Assisted Speedrun)
//! Savestates, input recording/replay, rewind, ghost, frame advance
//! This is THE killer feature — no other game has bare-metal TAS with kernel introspection
#![allow(dead_code)]

use alloc::vec::Vec;
use super::physics::Vec3;

// ======================== Input Recording ========================

/// Complete input state for one frame (SM64-style)
#[derive(Clone, Copy, Debug, Default)]
pub struct FrameInput {
    pub frame: u64,
    pub stick_x: i8,       // -128 to +127 (analog stick horizontal)
    pub stick_y: i8,       // -128 to +127 (analog stick vertical)
    pub buttons: u16,      // bitfield of pressed buttons
    pub mouse_dx: i16,     // camera delta X
    pub mouse_dy: i16,     // camera delta Y
}

// Button masks (SM64-inspired)
pub const BTN_A: u16      = 0x0001; // Jump
pub const BTN_B: u16      = 0x0002; // Attack / Dive
pub const BTN_Z: u16      = 0x0004; // Crouch / Ground pound
pub const BTN_START: u16  = 0x0008; // Pause
pub const BTN_L: u16      = 0x0010; // Camera mode
pub const BTN_R: u16      = 0x0020; // Camera mode
pub const BTN_C_UP: u16   = 0x0040; // Camera
pub const BTN_C_DOWN: u16 = 0x0080; // Camera
pub const BTN_C_LEFT: u16 = 0x0100; // Camera
pub const BTN_C_RIGHT: u16= 0x0200; // Camera

// ======================== Savestate ========================

/// Full game state snapshot — everything needed to restore the exact frame
#[derive(Clone)]
pub struct SaveState {
    pub frame: u64,
    // Mario state
    pub mario_pos: Vec3,
    pub mario_vel: Vec3,
    pub mario_facing: f32,
    pub mario_action: u32,
    pub mario_action_timer: u32,
    pub mario_hp: u8,
    pub mario_forward_vel: f32,
    pub mario_peak_height: f32,
    pub mario_on_ground: bool,
    // Game state
    pub coins: u16,
    pub stars: u8,
    pub lives: u8,
    pub rng_state: u64,
    // Enemies (simplified — store positions)
    pub enemy_data: Vec<(Vec3, u8)>, // (position, state_id)
    // Objects
    pub object_data: Vec<(Vec3, bool)>, // (position, collected)
    // Camera
    pub cam_pos: Vec3,
    pub cam_yaw: f32,
    pub cam_pitch: f32,
    pub cam_dist: f32,
}

impl SaveState {
    pub fn empty() -> Self {
        Self {
            frame: 0,
            mario_pos: Vec3::ZERO, mario_vel: Vec3::ZERO,
            mario_facing: 0.0, mario_action: 0, mario_action_timer: 0,
            mario_hp: 8, mario_forward_vel: 0.0, mario_peak_height: 0.0,
            mario_on_ground: true,
            coins: 0, stars: 0, lives: 4, rng_state: 0,
            enemy_data: Vec::new(), object_data: Vec::new(),
            cam_pos: Vec3::ZERO, cam_yaw: 0.0, cam_pitch: 0.0, cam_dist: 10.0,
        }
    }
}

// ======================== Ghost ========================

#[derive(Clone, Copy, Debug, Default)]
pub struct GhostFrame {
    pub pos: Vec3,
    pub facing: f32,
    pub anim_id: u8,
    pub anim_time: f32,
}

// ======================== TAS Mode ========================

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TasMode {
    Normal,       // playing normally
    Recording,    // recording inputs
    Replaying,    // replaying recorded inputs
    FrameAdvance, // pause + step frame by frame
}

// ======================== TAS Engine ========================

pub const NUM_SAVE_SLOTS: usize = 8;
pub const REWIND_BUFFER_SIZE: usize = 300; // 5 seconds at 60fps

pub struct TasEngine {
    pub mode: TasMode,
    pub frame: u64,

    // Input recording (.tm64 format)
    pub input_log: Vec<FrameInput>,
    pub replay_cursor: usize,

    // Savestates
    pub slots: [Option<SaveState>; NUM_SAVE_SLOTS],
    pub active_slot: usize,

    // Rewind buffer (circular)
    pub rewind_buffer: Vec<SaveState>,
    pub rewind_head: usize,
    pub rewind_count: usize,

    // Ghost
    pub ghost_recording: Vec<GhostFrame>,
    pub ghost_replay: Vec<GhostFrame>,
    pub ghost_active: bool,

    // Frame advance
    pub advance_requested: bool,

    // Display
    pub show_input_display: bool,
    pub show_info_panel: bool,
    pub show_hitboxes: bool,

    // Current frame input (for display)
    pub current_input: FrameInput,
}

impl TasEngine {
    pub fn new() -> Self {
        Self {
            mode: TasMode::Normal,
            frame: 0,
            input_log: Vec::new(),
            replay_cursor: 0,
            slots: [
                None, None, None, None,
                None, None, None, None,
            ],
            active_slot: 0,
            rewind_buffer: Vec::new(),
            rewind_head: 0,
            rewind_count: 0,
            ghost_recording: Vec::new(),
            ghost_replay: Vec::new(),
            ghost_active: false,
            advance_requested: false,
            show_input_display: true,
            show_info_panel: false,
            show_hitboxes: false,
            current_input: FrameInput::default(),
        }
    }

    /// Should the game advance this frame?
    pub fn should_tick(&self) -> bool {
        match self.mode {
            TasMode::Normal | TasMode::Recording | TasMode::Replaying => true,
            TasMode::FrameAdvance => self.advance_requested,
        }
    }

    /// Record the current frame's input
    pub fn record_input(&mut self, input: FrameInput) {
        if self.mode == TasMode::Recording {
            let mut inp = input;
            inp.frame = self.frame;
            self.input_log.push(inp);
        }
        self.current_input = input;
    }

    /// Get input for current frame during replay
    pub fn get_replay_input(&mut self) -> Option<FrameInput> {
        if self.mode != TasMode::Replaying { return None; }
        if self.replay_cursor < self.input_log.len() {
            let input = self.input_log[self.replay_cursor];
            self.replay_cursor += 1;
            Some(input)
        } else {
            // Replay finished
            self.mode = TasMode::Normal;
            None
        }
    }

    /// Start recording inputs
    pub fn start_recording(&mut self) {
        self.mode = TasMode::Recording;
        self.input_log.clear();
    }

    /// Stop recording
    pub fn stop_recording(&mut self) {
        self.mode = TasMode::Normal;
    }

    /// Start replaying recorded inputs
    pub fn start_replay(&mut self) {
        if !self.input_log.is_empty() {
            self.mode = TasMode::Replaying;
            self.replay_cursor = 0;
        }
    }

    /// Toggle frame advance mode
    pub fn toggle_frame_advance(&mut self) {
        self.mode = if self.mode == TasMode::FrameAdvance {
            TasMode::Normal
        } else {
            TasMode::FrameAdvance
        };
        self.advance_requested = false;
    }

    /// Step one frame in frame-advance mode
    pub fn step_frame(&mut self) {
        if self.mode == TasMode::FrameAdvance {
            self.advance_requested = true;
        }
    }

    /// Save state to active slot
    pub fn save_state(&mut self, state: SaveState) {
        self.slots[self.active_slot] = Some(state);
    }

    /// Load state from active slot
    pub fn load_state(&self) -> Option<&SaveState> {
        self.slots[self.active_slot].as_ref()
    }

    /// Push state to rewind buffer
    pub fn push_rewind(&mut self, state: SaveState) {
        if self.rewind_buffer.len() < REWIND_BUFFER_SIZE {
            self.rewind_buffer.push(state);
        } else {
            self.rewind_buffer[self.rewind_head] = state;
        }
        self.rewind_head = (self.rewind_head + 1) % REWIND_BUFFER_SIZE;
        if self.rewind_count < REWIND_BUFFER_SIZE {
            self.rewind_count += 1;
        }
    }

    /// Pop most recent state from rewind buffer
    pub fn pop_rewind(&mut self) -> Option<SaveState> {
        if self.rewind_count == 0 { return None; }
        self.rewind_head = if self.rewind_head == 0 { REWIND_BUFFER_SIZE - 1 } else { self.rewind_head - 1 };
        self.rewind_count -= 1;
        if self.rewind_head < self.rewind_buffer.len() {
            Some(self.rewind_buffer[self.rewind_head].clone())
        } else {
            None
        }
    }

    /// Record ghost frame
    pub fn record_ghost(&mut self, pos: Vec3, facing: f32, anim_id: u8, anim_time: f32) {
        self.ghost_recording.push(GhostFrame { pos, facing, anim_id, anim_time });
    }

    /// Finalize ghost recording (swap active ghost)
    pub fn finalize_ghost(&mut self) {
        self.ghost_replay = self.ghost_recording.clone();
        self.ghost_recording.clear();
    }

    /// Get ghost frame for current frame
    pub fn get_ghost_frame(&self) -> Option<GhostFrame> {
        if !self.ghost_active { return None; }
        let idx = self.frame as usize;
        if idx < self.ghost_replay.len() {
            Some(self.ghost_replay[idx])
        } else {
            None
        }
    }

    /// Advance frame counter
    pub fn advance_frame(&mut self) {
        self.frame += 1;
        self.advance_requested = false;
    }

    /// Get recording length in frames
    pub fn recording_length(&self) -> usize {
        self.input_log.len()
    }
}
