#![no_std]
#![no_main]
#![allow(unused_variables)] // only used for development
#![allow(unused_imports)] // only used for development

use defmt::*;
use defmt_rtt as _;
use panic_probe as _;

use embassy_executor::Spawner;
use embassy_rp::config::Config;
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
    // Initialize system with proper clock configuration
    let config = Config::default();
    let _peripherals = embassy_rp::init(config);

    // Parse and define a configuration value
    let number_of_messages = NUMBER_OF_MESSAGES.parse::<u8>().unwrap();

    info!("Number of Messages: {}", number_of_messages);

    for index in 1..=number_of_messages {
        info!("Hello there - {}", index);
        Timer::after_secs(1).await;
    }

    // Deobfuscate secret (The utility module accesses XOR_KEY)
    let super_secret_info = utility::deobfuscate(SUPER_SECRET_INFO_OBFUSCATED);

    // WARNING: Never log secrets! This is for demonstration only
    info!("Super Secret Info: {}", super_secret_info.as_str());

    // Start an infinite loop
    loop {
        Timer::after_secs(5).await;
    }
}
