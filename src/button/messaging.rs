//! Button module - Messaging - Pub-Sub

#![allow(dead_code)] // only used for development

use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex; // Ensure thread-safety across tasks
use embassy_sync::pubsub::PubSubChannel;
use embassy_sync::signal::Signal;
use embassy_time::Instant;

/// Button press type
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum PressType {
    /// Short button press type
    ShortRelease,
    /// Long button press type
    LongRelease,
    /// Long hold
    LongHold,
}

/// Button pub-sub message item definition/structure that is passed via channel
#[derive(Debug, Copy, Clone)]
pub struct ButtonMessage {
    /// Button ID
    pub id: u8,
    /// Timestamp of button press start
    pub timestamp_start: Instant,
    /// Timestamp of button press end
    pub timestamp_end: Instant,
    /// The type of button press
    pub press_type: PressType,
}

/// Button pub-sub topic channel
/// Other program parts can listen to this topic to get button press events
/// 3 total capacity/messages, 3 subscribers, and 1 publisher
pub static BUTTON_PUBSUB_CHANNEL: PubSubChannel<ThreadModeRawMutex, ButtonMessage, 2, 3, 1> = PubSubChannel::new();

/// A signal / flag that signals that the button system ready
pub static BUTTON_READY_SIGNAL: Signal<ThreadModeRawMutex, bool> = Signal::new();
