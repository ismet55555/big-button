#![no_std]
#![no_main]
#![allow(unused_variables)] // only used for development
#![allow(unused_imports)] // only used for development

use defmt::*;
use defmt_rtt as _;
use panic_probe as _;

use embassy_executor::Spawner;
use embassy_rp::config::Config as HalConfig;
use embassy_rp::gpio;
use embassy_time::Timer;

mod clocks_config;
mod utility;
use clocks_config::ClockSettings;

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
        system_frequency_mhz: CLOCK_SYSTEM_FREQEUENCY_MHZ.parse::<u32>().unwrap(),
        usb_frequency_mhz: CLOCK_USB_FREQEUENCY_MHZ.parse::<u32>().unwrap(),
        peripheral_clock_divider: CLOCK_PERIPHERAL_DIVIDER.parse::<u8>().unwrap(),
        adc_frequency_mhz: CLOCK_ADC_FREQUENCY_MHZ.parse::<u32>().unwrap(),
        reference_clock_divider: CLOCK_REFERENCE_DIVIDER.parse::<u8>().unwrap(),
    };
    clocks_config::configure_all_clocks(&mut hal_configuration, clock_settings);
    let peripherals = embassy_rp::init(hal_configuration);

    // Log and verify system clock frequencies
    clocks_config::print_device_frequencies();
    clocks_config::verify_clock_with_timer(500).await;

    info!("All Set! Running async loop ...");

    // General async loop to ensure entire program runs forever while
    // asynchronously working on other tasks
    loop {
        Timer::after_secs(5).await;
    }
}
