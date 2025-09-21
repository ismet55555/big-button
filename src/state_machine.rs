//! State machine module for the embedded application

#![allow(dead_code)] // only used for development
#![allow(unused_variables)] // only used for development

use defmt::*;
use defmt_rtt as _;
use panic_probe as _;

use embassy_executor::SpawnError;
use embassy_executor::Spawner;
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex; // Ensure thread-safety across tasks
use embassy_sync::pubsub::{Publisher, Subscriber};
use embassy_time::{Duration, Instant, Timer};

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
    // Temporary state machine event (To be replaced later)
    SomethingElse,
    /// Error has occurred
    Error,
    /// Error recovery event
    ErrorRecovery,
}

//////////////////////////////////////////////////////////////////////////////////////////////////

struct StateMachine {
    /// Current state of the state machine: [`State`]
    current_state: State,
    /// Latest event: [`Event`]
    latest_event: Event,
    /// Last recovery attempt time
    last_recovery_attempt: Instant,
}

impl StateMachine {
    /// Constructor
    fn new(current_state: State, latest_event: Event) -> Self {
        StateMachine {
            current_state,
            latest_event,
            last_recovery_attempt: Instant::now(),
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

            (State::Idle, Event::SomethingElse) => {
                info!("[State: Idle - Event: SomethingElse] Some other event happened: Idle -> Processing");
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

            (State::Error, Event::ErrorRecovery) => {
                info!("[State: Processing - Event: ErrorRecovery] Error Recovery: Error -> Idle");
                if self.attempt_error_recovery().await {
                    self.current_state = State::Idle;
                    info!("Successfully recovered from error");
                } else {
                    self::panic!("Failed to recover from error");
                }
            }

            (State::Error, _) => {
                // While state machine is in "Error" state, ignore all events except "ErrorRecovery"
            }

            _ => {} // No state change for unhandled events
        }
    }

    /// Get the current event - based on various conditions and inputs
    /// Return event to the state machine for determining the next state
    async fn get_current_event(&mut self) -> Event {
        // Check for any type of device Error
        if self.check_for_errors().await {
            return Event::Error;
        }

        // Error recovery if in Error state, periodically (every 10 sec) attempt recovery
        if self.current_state == State::Error
            && Instant::now().duration_since(self.last_recovery_attempt) >= Duration::from_secs(10)
        {
            self.last_recovery_attempt = Instant::now();
            return Event::ErrorRecovery;
        }

        // ++ Add event handling here (i.e. monitor button press) ++

        // No events occurred, return a nothing event
        Event::Nothing
    }

    /// Check for any system error conditions
    /// Return `true` if an error is detected
    async fn check_for_errors(&self) -> bool {
        false
    }

    /// Attempt to recover from an error condition
    /// Return `true` if recovered from error
    async fn attempt_error_recovery(&self) -> bool {
        debug!("Attempting device error recovery ...");
        false
    }
}

//////////////////////////////////////////////////////////////////////////////////////////////////

/// State machine task with infinite loop
#[embassy_executor::task]
pub async fn state_machine_task() -> ! {
    info!("Running State Machine async task ...");

    let mut state_machine = StateMachine::new(State::Startup, Event::Nothing);
    state_machine.handle_event(Event::PowerOn).await;

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
