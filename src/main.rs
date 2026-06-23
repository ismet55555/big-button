#![no_std]
#![no_main]
#![allow(unused_variables)] // only used for development
#![allow(unused_imports)] // only used for development
#![allow(dead_code)] // only used for development

use assign_resources::assign_resources; // TODO: Check order of imports
use defmt::*;
use defmt_rtt as _;
use panic_probe as _;

use embassy_executor::Spawner;
use embassy_rp::Peri;
use embassy_rp::config::Config as HalConfig;
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_rp::{bind_interrupts, peripherals};
use embassy_time::Timer;

mod clocks_config;
use clocks_config::ClockSettings;
mod button;
mod led;
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

// Peripherals Definition and Split
// Note: PINs are GPIO pins, not physical pins on device
assign_resources! {
    /// Button hardware resources split
    buttons: ResourcesButtons {
        button0: PIN_15, // Top Button
    }
    /// LED (Simple on/off state) hardware resources split
    leds: ResourcesLed {
        led0: PIN_0,
        led1: PIN_1,
        led2: PIN_2,
        led3: PIN_3,
        led4: PIN_4,
        led5: PIN_5,
        led6: PIN_6,
        led7: PIN_7,
    }
}

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

    info!("Initializing peripherals ...");
    let peripherals_split = split_resources!(peripherals);

    // Task to check if all system components are ready to go
    spawner.spawn(system_manager::wait_for_system_ready()).unwrap();

    //////////////////////////////////////////////////////////////////////////
    
    info!("Initializing resources - Led (Simple) ...");
    // [(<LED ID>, <HANDLE>)]
    let mut led_info = [
        (0_u8, Output::new(peripherals_split.leds.led0, Level::Low)),
        (1_u8, Output::new(peripherals_split.leds.led1, Level::Low)),
        (2_u8, Output::new(peripherals_split.leds.led2, Level::Low)),
        (3_u8, Output::new(peripherals_split.leds.led3, Level::Low)),
        (4_u8, Output::new(peripherals_split.leds.led4, Level::Low)),
        (5_u8, Output::new(peripherals_split.leds.led5, Level::Low)),
        (6_u8, Output::new(peripherals_split.leds.led6, Level::Low)),
        (7_u8, Output::new(peripherals_split.leds.led7, Level::Low)),
    ];
    spawner
        .spawn(led::start_led_consumer(led_info))
        .expect("Failed spawning led_consumer");

    //////////////////////////////////////////////////////////////////////////

    // Button - Define and spawn async task
    // let button_info = [(0_u8, Input::new(peripherals.PIN_15, Pull::Up))];
    let button_info = [(0_u8, Input::new(peripherals_split.buttons.button0, Pull::Up))];
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
