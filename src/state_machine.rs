//! State machine module for the embedded application

#![allow(dead_code)] // only used for development
#![allow(unused_variables)] // only used for development

use defmt::*;
use defmt_rtt as _;
use panic_probe as _;

use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::pubsub::Subscriber;
use embassy_time::Timer;

use crate::button::{BUTTON_PUBSUB_CHANNEL, ButtonMessage, PressType};
use crate::system_manager::SYSTEM_READY_PUBSUB_CHANNEL;

/// State machine possible states
#[derive(Format, Debug, Clone, Copy, PartialEq)]
enum State {
    /// Startup - Initial state
    Startup,
    /// Device is idle
    Idle,
    /// Device is processing something
    Processing,
    /// Device is in an error state
    Error,
}

/// State machine possible events
#[derive(Format, Debug, Clone, Copy)]
enum Event {
    /// Initial startup
    PowerOn,
    /// System ready
    Ready,
    /// Nothing has happened
    Nothing,
    /// Button press event is a short press
    ButtonPressShortRelease,
    /// Button press event is a long press
    ButtonPressLongRelease,
    /// Button press event is a long hold
    ButtonPressLongHold,
    /// Error has occurred
    Error,
}

struct StateMachine {
    /// Current state of the state machine: [`State`]
    current_state: State,
    /// Latest event: [`Event`]
    latest_event: Event,
    /// Pub-sub subscriber: Listen for Button messages
    button_pubsub_subscriber: Subscriber<'static, ThreadModeRawMutex, ButtonMessage, 2, 3, 1>,
}

impl StateMachine {
    /// Constructor
    fn new(current_state: State, latest_event: Event) -> Self {
        let button_pubsub_subscriber = BUTTON_PUBSUB_CHANNEL.subscriber().unwrap();
        StateMachine {
            current_state,
            latest_event,
            button_pubsub_subscriber,
        }
    }

    /// Handle events and transition between states in state machine
    ///
    /// * `event` - Event to handle
    async fn handle_event(&mut self, event: Event) {
        // Check the current state of the state machine with the passed state machine event
        match (self.current_state, event) {
            (State::Startup, Event::PowerOn) => {
                info!("[State: Startup - Event: PowerOn]");
            }

            (State::Startup, Event::Ready) => {
                info!("[State: Startup - Event: Ready] Startup -> Idle");
                self.current_state = State::Idle;
            }

            (State::Idle, Event::ButtonPressShortRelease) => {
                info!("[State: Idle - Event: ButtonPressShortRelease] Button Press SHORT RELEASE: Idle -> Processing");
                self.current_state = State::Processing;
            }

            (State::Idle, Event::ButtonPressLongRelease) => {
                info!("[State: Idle - Event: ButtonPressLongRelease] Button Press LONG RELEASE: Idle -> Processing");
                self.current_state = State::Processing;
            }

            (State::Idle, Event::ButtonPressLongHold) => {
                info!("[State: Idle - Event: ButtonPressLongHold] Button Press LONG HOLD: Idle -> Processing");
                self.current_state = State::Processing;
            }

            (State::Processing, Event::Nothing) => {
                info!("[State: Processing - Event: Nothing] Nothing: Processing -> Idle");
                self.current_state = State::Idle;
            }

            (_, Event::Error) => {
                info!("[Event: Error] Error: {:?} -> Error", self.current_state);
                self.current_state = State::Error;
            }

            (State::Error, _) => {
                core::panic!("[State: Error] State machine in an error state!");
            }

            _ => {} // No state change for unhandled events
        }
    }

    /// Get the current event - based on various conditions and inputs
    /// Return event to the state machine for determining the next state
    async fn get_current_event(&mut self) -> Event {
        // Check button press event
        match self.button_pubsub_subscriber.try_next_message_pure() {
            None => {} // No message available, do nothing
            Some(button_message) => match button_message.press_type {
                PressType::ShortRelease => {
                    return Event::ButtonPressShortRelease;
                }
                PressType::LongRelease => {
                    return Event::ButtonPressLongRelease;
                }
                PressType::LongHold => {
                    return Event::ButtonPressLongHold;
                }
            },
        }

        // No events occurred, return a nothing event
        Event::Nothing
    }

    /// Check for any system error conditions
    /// Return `true` if an error is detected
    async fn check_for_errors(&self) -> bool {
        false
    }
}

/// State machine task with infinite loop
#[embassy_executor::task]
pub async fn state_machine_task() -> ! {
    info!("Running State Machine async task ...");
    let mut state_machine = StateMachine::new(State::Startup, Event::PowerOn);

    // Send a "PowerOn" event to state machine to handle
    state_machine.handle_event(Event::PowerOn).await;

    // Waiting on system to be ready to proceed
    let mut system_ready_message = SYSTEM_READY_PUBSUB_CHANNEL.subscriber().unwrap();
    system_ready_message.next_message_pure().await;

    // Send a "Ready" event to state machine to handle
    state_machine.handle_event(Event::Ready).await;

    // Main infinite loop for the state machine
    loop {
        // Get the current event
        let current_event = state_machine.get_current_event().await;

        // Handle the event and update the state
        state_machine.handle_event(current_event).await;

        // Add a small delay to prevent tight looping
        Timer::after_millis(10).await;
    }
}
