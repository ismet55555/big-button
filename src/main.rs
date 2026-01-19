#![no_std]
#![no_main]
#![allow(unused_variables)] // only used for development
#![allow(unused_imports)] // only used for development

use defmt::*;
use defmt_rtt as _;
use panic_probe as _;

use embassy_executor::Spawner;
use embassy_rp::config::Config as HalConfig;
use embassy_rp::gpio::{Input, Pull};
use embassy_time::Timer;

mod clocks_config;
use clocks_config::ClockSettings;
mod button;
mod state_machine;
mod system_manager;
mod utility;

// Loading configurations
// Note: This comes from the 'configs.rs' file created in 'build.rs' from 'configs.json'
// Note: All loaded configurations will be UPPERCASE string slice (&str) constants
include!(concat!(env!("OUT_DIR"), "/configs.rs"));

// Loading obfuscated secrets (XOR_KEY and _OBFUSCATED constants)
// Note: Similar considerations as configurations
// Note: Load via utility.deobfuscate()
include!(concat!(env!("OUT_DIR"), "/secrets.rs"));

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("==================================");
    info!("    Package: {} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    info!("==================================");

    // Apply default clock configuration
    let mut hal_configuration = HalConfig::default();

    info!("Configuring all system clocks ...");
    let clock_settings = ClockSettings {
        xosc_crystal_hz: 12_000_000, // RP2350 uses 12MHz crystal
        system_frequency_mhz: CLOCK_SYSTEM_FREQUENCY_MHZ.parse::<u32>().unwrap(),
        usb_frequency_mhz: CLOCK_USB_FREQUENCY_MHZ.parse::<u32>().unwrap(),
        peripheral_clock_divider: CLOCK_PERIPHERAL_DIVIDER.parse::<u8>().unwrap(),
        adc_frequency_mhz: CLOCK_ADC_FREQUENCY_MHZ.parse::<u32>().unwrap(),
        reference_clock_divider: CLOCK_REFERENCE_DIVIDER.parse::<u8>().unwrap(),
    };
    clocks_config::configure_all_clocks(&mut hal_configuration, clock_settings);
    let peripherals = embassy_rp::init(hal_configuration);

    // Log and verify system clock frequencies
    clocks_config::print_device_frequencies();
    clocks_config::verify_clock_with_timer(500).await;

    //////////////////////////////////////////////////////////////////////////

    // Task to check if all system components are ready to go
    spawner.spawn(system_manager::wait_for_system_ready()).unwrap();

    //////////////////////////////////////////////////////////////////////////

    // Button - Define and spawn async task
    let button_info = [(0_u8, Input::new(peripherals.PIN_15, Pull::Up))];
    spawner
        .spawn(button::start_button_monitor(button_info))
        .expect("Failed spawning button_consumer");

    //////////////////////////////////////////////////////////////////////////

    // Spawn state machine async task
    spawner
        .spawn(state_machine::state_machine_task())
        .expect("Failed spawning state machine");

    info!("Running async loop ...");

    // General async loop to ensure entire program runs forever while
    // asynchronously working on other tasks
    loop {
        Timer::after_secs(5).await;
    }
}
