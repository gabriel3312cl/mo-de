//! Game module - Core game engine and state machine

pub mod board;
mod engine;
mod events;
pub mod state;

pub use board::BOARD;
pub use engine::GameEngine;
pub use events::{ClientEvent, ServerEvent};
pub use state::*;
