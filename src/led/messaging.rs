//! LED (simple) messaging logic

use defmt_rtt as _;
use panic_probe as _;

use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex; // Ensure thread-safety across tasks
use embassy_sync::pubsub::PubSubChannel;
use embassy_sync::signal::Signal;

//////////////////////////////////////////////////////////////////////////////
//                          PUB-SUB TOPIC CHANNEL
//////////////////////////////////////////////////////////////////////////////

/// Led pub-sub message item definition/structure
#[derive(Debug, Copy, Clone)]
pub enum LedMessage {
    /// Set immediate on/off state (`false`: OFF or `true`: ON)
    SetState { ids: &'static [u8], state: bool },
}

/// Led pub-sub topic channel
/// Led listen to this topic to output certain Led statas
/// 2 total capacity/messages, 3 subscribers, and 1 publisher
pub static LED_PUBSUB_CHANNEL: PubSubChannel<ThreadModeRawMutex, LedMessage, 2, 3, 1> = PubSubChannel::new();

/// A signal / flag that signals that the Rest request system ready
pub static LED_READY_SIGNAL: Signal<ThreadModeRawMutex, bool> = Signal::new();
