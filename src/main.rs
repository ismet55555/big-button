#![no_std]
#![no_main]
#![allow(unused_variables)] // for development
#![allow(unused_imports)] // for development

use defmt::*;
use defmt_rtt as _;
use panic_probe as _;

use embassy_executor::Spawner;
use embassy_rp::config::Config;
use embassy_rp::gpio;
use embassy_time::Timer;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // Initialize system with proper clock configuration
    let config = Config::default();
    let _peripherals = embassy_rp::init(config);

    // Start an infinite loop
    loop {
        info!("Hello World!");
        Timer::after_secs(1).await;

        info!("Goodbye World!");
        Timer::after_secs(1).await;
    }
}
