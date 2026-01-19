//! Button - Utility functions

use embassy_time::Timer;

/// Create a future that doesn't do anything except wait
/// This can be used for placeholder for anything that idles
pub async fn do_nothing_idle() {
    loop {
        Timer::after_millis(1000).await;
    }
}
