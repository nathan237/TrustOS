//! TrustOS Applications
//!
//! Built-in graphical applications

pub mod calculator;
pub mod text_editor;

pub use calculator::{Calculator, create_calculator_window};
pub use text_editor::EditorState;
