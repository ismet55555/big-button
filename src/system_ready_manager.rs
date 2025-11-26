//! System ready manager

use defmt::*;
use defmt_rtt as _;
use panic_probe as _;

use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::pubsub::PubSubChannel;

use crate::button::BUTTON_READY_SIGNAL;
use crate::led::LED_READY_SIGNAL;

//////////////////////////////////////////////////////////////////////////////
//                         SYSTEM READY SIGNAL
//////////////////////////////////////////////////////////////////////////////

/// System ready pub-sub topic channel
/// 1 total capacity/messages, 4 subscribers, and 1 publisher
pub static SYSTEM_READY_PUBSUB_CHANNEL: PubSubChannel<ThreadModeRawMutex, (), 1, 4, 1> = PubSubChannel::new();

/// Async task - Waiting for all sub-systems to report back as ready
#[embassy_executor::task]
pub async fn wait_for_system_ready() {
    info!("Running System ready async task ...");
    info!("Waiting for system to be ready ...");
    BUTTON_READY_SIGNAL.wait().await;
    LED_READY_SIGNAL.wait().await;

    SYSTEM_READY_PUBSUB_CHANNEL.publisher().unwrap().publish(()).await;
    info!(">>>>>>>>>  ALL SYSTEMS GO! BIG BUTTON IS READY!  <<<<<<<<<");
}
