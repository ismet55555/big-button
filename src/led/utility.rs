//! LED - Utility functions

use embassy_time::Timer;

/// Create a future that doesn't do anything except wait
/// This can be used for placeholder for idle animations
///
/// Example usage:
///
/// ```rust
/// use embassy_futures::select::{select, Either};
///
/// loop {
///     let message_or_idle = select(
///         pubsub_topic_subscriber.next_message_pure(), // Get message
///         utility::do_nothing_idle(),                  // Do nothing / idle
///     ).await;
///     let message = match buzzer_message_or_idle {
///         Either::First(message) => message, // Extract message
///         Either::Second(_) => continue,     // Continue loop
///     };
/// }
/// ```
pub async fn do_nothing_idle() {
    loop {
        Timer::after_millis(1000).await;
    }
}
