extern crate core;

mod game;
mod linestore;
pub mod physics;
pub mod rider;

pub use game::*;

/// Boolean set to print physics information for debugging
pub(crate) static DEBUG_PRINT: bool = false;
