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

    info!("Setting system clock frequency ...");
    let mut hal_configuration = HalConfig::default();
    utility::set_system_clock_frequency(
        &mut hal_configuration,
        12_000_000,
        SYSTEM_CLOCK_FREQEUENCY_MHZ.parse::<u32>().unwrap(),
        USB_CLOCK_FREQEUENCY_MHZ.parse::<u32>().unwrap(),
    );
    let peripherals = embassy_rp::init(hal_configuration);

    // Log and verify system clock frequencies
    utility::print_device_frequencies();
    utility::verify_clock_with_timer(500).await;

    info!("All Set! Running async loop ...");

    // General async loop to ensure entire program runs forever while
    // asynchronously working on other tasks
    loop {
        Timer::after_secs(5).await;
    }
}
