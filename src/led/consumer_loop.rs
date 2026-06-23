//! Led module - consumer task

use defmt::*;
use defmt_rtt as _;
use panic_probe as _;

use embassy_rp::gpio::Output;

use embassy_futures::select::{Either, select};

use super::LED_READY_SIGNAL;
use super::utility;
use super::{LED_PUBSUB_CHANNEL, Led, LedMessage};
use crate::system_manager::SYSTEM_READY_PUBSUB_CHANNEL;

/// Async task - LED consumer
/// Continuously monitor LED status changes messages
///
/// * `led_info` - LED information vector
#[embassy_executor::task]
pub async fn start_led_consumer(led_info: [(u8, Output<'static>); 8]) -> ! {
    info!("Running LED simple consumer async task ...");

    let mut led_pubsub_topic_subscriber = LED_PUBSUB_CHANNEL.subscriber().unwrap();
    let mut led = Led::new(led_info);

    // Set all LEDs to OFF
    led.set_status(&[], false).await;

    // Signal to system that button is ready to be used
    LED_READY_SIGNAL.signal(true);

    // Wait idle until the system manager sends a ready signal
    let mut system_ready_message = SYSTEM_READY_PUBSUB_CHANNEL.subscriber().unwrap();
    select(system_ready_message.next_message_pure(), utility::do_nothing_idle()).await;

    loop {
        debug!("LED simple consumer is waiting for its pubsub message ...");

        // Select between an available message OR idle behavior
        let led_message_or_idle = select(
            led_pubsub_topic_subscriber.next_message_pure(), // Get topic message from channel
            utility::do_nothing_idle(),
        )
        .await;

        // Extract the message
        let led_message = match led_message_or_idle {
            Either::First(message) => message,
            Either::Second(_) => continue, // Continue loop if no message
        };

        // Parse the message and perform the appropriate action
        match led_message {
            LedMessage::SetState { ids, state } => {
                led.set_status(ids, state).await;
            }
        }
    }
}
