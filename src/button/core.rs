//! Button module

use defmt::*;
use defmt_rtt as _;
use panic_probe as _;

use embassy_rp::gpio::Input;
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex; // Ensure thread-safety across tasks
use embassy_sync::pubsub::{Error, Publisher};
use embassy_time::Instant;
use embassy_time::Timer;

use super::{BUTTON_PUBSUB_CHANNEL, ButtonMessage, PressType};

// Custom type aliases
type ItemId = u8;
type ItemHandle = Input<'static>;
type ItemInfo = [(ItemId, ItemHandle); 1];

/// Handle for individual Buttons - Button has all provisioned buttons defined
pub struct Button<'a> {
    /// Button definitions - `[(<BUTTON ID>, <`[Input]` HANDLE>), ...]`
    pub item_info: ItemInfo,
    /// Button pub-sub publisher for message publishing
    pub button_pubsub_publisher: Publisher<'a, ThreadModeRawMutex, ButtonMessage, 2, 3, 1>,
}

impl<'a> Button<'a> {
    /// Constructor
    ///
    /// * `item_info` - Button definitions
    pub fn new(item_info: ItemInfo) -> Result<Self, Error> {
        let button_pubsub_publisher = BUTTON_PUBSUB_CHANNEL.publisher()?;
        Ok(Self {
            item_info,
            button_pubsub_publisher,
        })
    }

    /// Get all IDs
    pub fn get_ids(&self) -> [u8; 1] {
        let mut ids = [0; 1];
        for (index, (id, _)) in self.item_info.iter().enumerate() {
            ids[index] = *id;
        }
        ids
    }

    /// Check if specified IDs are valid and within the pre-defined item info
    ///
    /// * `ids` - Item IDs to check. Empty ids array will return `true`
    pub async fn check_ids(&self, ids: &[u8]) -> bool {
        if ids.is_empty() {
            return true;
        }
        for id in ids.iter() {
            if !self.get_ids().contains(id) {
                return false;
            }
        }
        true
    }

    /// Debouncing button press - GPIO Level HIGH to LOW
    /// Debouncing is the process of removing noise from a button press signal.
    /// Returns when the button press signal is stable.
    ///
    /// * `id` - Button ID number
    async fn debounce_high_to_low(&mut self, id: u8) {
        let item = self.item_info.iter_mut().find(|(button_id, _)| *button_id == id);

        if let Some((_, handle)) = item {
            loop {
                let pin_level_1 = handle.get_level();
                handle.wait_for_low().await;
                Timer::after_millis(20).await;
                let pin_level_2 = handle.get_level();
                if pin_level_1 != pin_level_2 && handle.is_low() {
                    break;
                }
            }
        } else {
            warn!("Button with ID {} not found", id);
        }
    }

    /// Debouncing button press - GPIO Level LOW to HIGH
    /// Debouncing is the process of removing noise from a button press signal.
    /// Returns when the button press signal is stable.
    ///
    /// * `id` - Button ID number
    async fn debounce_low_to_high(&mut self, id: u8) {
        let item = self.item_info.iter_mut().find(|(button_id, _)| *button_id == id);

        if let Some((_, handle)) = item {
            loop {
                let pin_level_1 = handle.get_level();
                handle.wait_for_high().await;
                Timer::after_millis(20).await;
                let pin_level_2 = handle.get_level();
                if pin_level_1 != pin_level_2 && handle.is_high() {
                    break;
                }
            }
        } else {
            warn!("Button with ID {} not found", id);
        }
    }

    /// Continuously watch specified button edge state and report button press.
    ///
    /// * `id` - Button ID number
    pub async fn monitor_press(&mut self, id: u8) -> () {
        if !self.check_ids(&[id]).await {
            error!("Failed to find specified Button ID in the pre-defined Button info");
            return;
        }

        let mut button_down_press_timestamp: Instant;
        let mut button_up_release_timestamp: Instant;

        loop {
            // Wait for button down press
            self.debounce_high_to_low(id).await;

            button_down_press_timestamp = Instant::now();
            info!(
                "Button ID {} down pressed! - Timestamp: {:?}ms",
                id,
                button_down_press_timestamp.as_millis()
            );

            // Wait for button up release
            self.debounce_low_to_high(id).await;
            button_up_release_timestamp = Instant::now();
            let release_time = button_up_release_timestamp.duration_since(button_down_press_timestamp);
            info!(
                "Button ID {} up released! - Timestamp: {:?}ms -> Time Difference: {:?}ms",
                id,
                button_up_release_timestamp.as_millis(),
                release_time.as_millis(),
            );

            // Publish the button press event message to channel
            self.button_pubsub_publisher
                .publish(ButtonMessage {
                    id,
                    timestamp_start: button_down_press_timestamp,
                    timestamp_end: button_up_release_timestamp,
                    press_type: PressType::RegularPress,
                })
                .await;
        }
    }
}

/// Implement the `Format` trait for debugging
/// This allows us to print the struct with `defmt::info!` and friends
impl<'a> Format for Button<'a> {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(f, "Button {{");
        defmt::write!(f, " Buttons: ");
        for (id, _) in self.item_info.iter() {
            defmt::write!(f, "{{ID: {}}} ", id);
        }
        defmt::write!(f, "}}");
    }
}
