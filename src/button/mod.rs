//! Button module - Module definition - Responsible for handling system buttons

// Map all parts of this module
mod consumer_loop;
mod core;
mod messaging;

// Public re-export of specifics that are available outside of module
pub use consumer_loop::start_button_monitor;
pub use core::Button;
pub use messaging::{BUTTON_PUBSUB_CHANNEL, ButtonMessage, PressType};
