//! Button module - Task manager module

use defmt::*;
use defmt_rtt as _;
use panic_probe as _;

use embassy_futures::select::select;
use embassy_rp::gpio::Input;

use super::BUTTON_READY_SIGNAL;
use super::Button;
use super::utility;
use crate::system_manager::SYSTEM_READY_PUBSUB_CHANNEL;

/// Async task - Button
/// Continuously monitor and report button press/release events
///
/// * `button_info` - Button information vector
#[embassy_executor::task]
pub async fn start_button_monitor(button_info: [(u8, Input<'static>); 1]) {
    info!("Running Button monitor async task ...");
    let mut button = Button::new(button_info).unwrap();

    // Signal to system that button is ready to be used
    BUTTON_READY_SIGNAL.signal(true);

    // Wait idle until the system manager sends a ready signal
    let mut system_ready_message = SYSTEM_READY_PUBSUB_CHANNEL.subscriber().unwrap();
    select(system_ready_message.next_message_pure(), utility::do_nothing_idle()).await;

    // Start the continuous button monitor watch
    button.monitor_press(0).await;
}
