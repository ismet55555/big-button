//! LED module
//!
//! This module is responsible for handling simple state (on/off) LEDs.

// Map all parts of this module
mod consumer_loop;
mod core;
mod messaging;
mod utility;

// Public re-export of specifics that are available outside
pub use consumer_loop::start_led_consumer;
pub use core::Led;
pub use messaging::{LED_PUBSUB_CHANNEL, LedMessage};
