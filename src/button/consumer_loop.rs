//! Button module - Task manager module

use defmt::*;
use defmt_rtt as _;
use panic_probe as _;

use embassy_rp::gpio::Input;

use super::Button;

/// Async task - Button
/// Continuously monitor and report button press/release events
///
/// * `button_info` - Button information vector
#[embassy_executor::task]
pub async fn start_button_monitor(button_info: [(u8, Input<'static>); 1]) {
    info!("Running Button monitor async task ...");
    let mut button = Button::new(button_info).unwrap();

    button.monitor_press(0).await;
}
