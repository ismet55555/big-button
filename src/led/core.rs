//! LED (Simple) module

use core::panic::RefUnwindSafe;

use defmt::*;
use defmt_rtt as _;
use panic_probe as _;

use heapless::Vec;

use embassy_rp::gpio::Output;
use embassy_time::Instant;
use embassy_time::Timer;

//////////////////////////////////////////////////////////////////////////////

// Custom type aliases
type ItemId = u8;
type ItemHandle = Output<'static>;
type ItemInfo = [(ItemId, ItemHandle); 8];

/// Handle for individual simple LEDs - LED has all provisioned LEDS defined
pub struct Led {
    /// Led definitions - `[(<LED ID>, <`[Output]` HANDLE>), ...]`
    pub item_info: ItemInfo,
}

impl Led {
    /// Constructor
    ///
    /// * `item_info` - LED item definitions
    pub fn new(item_info: ItemInfo) -> Self {
        Self { item_info }
    }

    /// Given ID number, return hardware handle
    ///
    /// * `id` - Item ID number
    pub fn get_handle_by_id(&self, id: u8) -> Option<&Output> {
        for (item_id, handle) in self.item_info.iter() {
            if *item_id == id {
                return Some(handle);
            }
        }
        None
    }

    /// Get all IDs
    pub fn get_ids(&self) -> [u8; 8] {
        let mut ids = [0; 8];
        for (index, (id, _)) in self.item_info.iter().enumerate() {
            ids[index] = *id;
        }
        ids
    }

    /// Check if specified IDs are valid and within the pre-defined item info
    ///
    /// * `ids` - Item IDs to check. Eempty ids array will return `true`
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

    /// Set LED status to specified on/off status
    ///
    /// * `ids` - LED IDs to set status. If empty, set all LEDs.
    /// * `status` - Status to be set (`true`: ON, `false`: OFF)
    pub async fn set_status(&mut self, ids: &[u8], status: bool) {
        let led_ids: &[u8] = if ids.is_empty() { &self.get_ids() } else { ids };
        debug!("Setting LED IDs {:?} to status {}", led_ids, status);
        for (item_id, handle) in self.item_info.iter_mut() {
            if led_ids.contains(item_id) {
                if status {
                    handle.set_high();
                } else {
                    handle.set_low();
                }
            }
        }
    }
}

/// Implement the `Format` trait for debugging
/// This allows us to print the struct with `defmt::info!` and friends
impl Format for Led {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(f, "Led {{");
        defmt::write!(f, " LEDs: ");
        for (id, _) in self.item_info.iter() {
            defmt::write!(f, "{{ID: {}}}", id,);
        }
        defmt::write!(f, "}}");
    }
}
