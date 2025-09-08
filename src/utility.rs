//! Utility functions for the embedded application
//! This module contains helper functions that can be used throughout the project

#![allow(dead_code)] // only used for development

use heapless::String;

use embassy_rp::clocks::{self, ClockConfig, PllConfig, SysClkConfig, SysClkSrc, UsbClkConfig, UsbClkSrc, XoscConfig};
use embassy_rp::config::Config as HalConfig;
use embassy_time::{Instant, Timer};

use defmt::*;
use defmt_rtt as _;
use panic_probe as _;

use crate::XOR_KEY;

/// Deobfuscates a byte array that was XOR encoded with XOR_KEY
///
/// This function takes the obfuscated bytes and XORs each one with
/// the XOR_KEY to recover the original secret string.
///
/// # Arguments
/// * `obfuscated` - A byte slice containing XOR-obfuscated data
///
/// # Returns
/// A heapless String containing the deobfuscated text (max 256 chars)
pub fn deobfuscate(obfuscated: &[u8]) -> String<256> {
    let mut result = String::new();

    for &byte in obfuscated {
        // XOR the byte with our key to get original string constant
        let original_char = (byte ^ XOR_KEY) as char;
        let _ = result.push(original_char);
    }
    result
}

/// Calculate the PLL (Phase-Locked Loop) configuration for the system clock and USB clock
///
/// * `desired_freq_mhz` - Desired frequency in MHz
/// * `xosc_freq_mhz` - Crystal oscillator frequency in Hz
/// * `post_div1` - Post divider 1 value
/// * `post_div2` - Post divider 2 value
///
/// The formula for calculating the PLL configuration is:
///    `fbdiv = (desired_freq_mhz * post_div1 * post_div2) / xosc_freq_mhz`
pub fn calculate_pll_config(desired_freq_mhz: u32, xosc_freq_hz: u32, post_div1: u8, post_div2: u8) -> PllConfig {
    let refdiv = 1u8; // Constant as 1
    let fbdiv = (desired_freq_mhz * post_div1 as u32 * post_div2 as u32) / (xosc_freq_hz / 1_000_000);
    debug!(
        "PLL Config: {} MHz -> refdiv={}, fbdiv={}, post_div1={}, post_div2={}",
        desired_freq_mhz, refdiv, fbdiv, post_div1, post_div2
    );
    PllConfig {
        refdiv,
        fbdiv: fbdiv as u16,
        post_div1,
        post_div2,
    }
}

/// Set the system and USB clock frequency
///
/// * `xosc_crystal_hz` - Crystal oscillator frequency in Hz
/// * `system_frequency_mhz` - System clock frequency in MHz
/// * `usb_frequency_mhz` - USB frequency in MHz
pub fn set_system_clock_frequency(
    hal_configuration: &mut HalConfig,
    xosc_crystal_hz: u32,
    system_frequency_mhz: u32,
    usb_frequency_mhz: u32,
) -> &mut HalConfig {
    // Calculate the PLL configuration for the system clock and USB clock
    let sys_pll_config = calculate_pll_config(system_frequency_mhz, xosc_crystal_hz, 6, 2);
    let usb_pll_config = calculate_pll_config(usb_frequency_mhz, xosc_crystal_hz, 6, 5);

    // Create and configure the external oscillator (XOSC) settings
    let mut clocks = ClockConfig::crystal(xosc_crystal_hz);
    clocks.xosc = Some(XoscConfig {
        hz: xosc_crystal_hz,           // Set XOSC frequency
        sys_pll: Some(sys_pll_config), // System PLL configuration
        usb_pll: Some(usb_pll_config), // USB PLL configuration
        delay_multiplier: 128,         // Used to calculate the startup delay (in cycles) for the crystal oscillator
                                       // startup_delay = ((crystal_freq / 1000) * delay_multiplier + 128) / 256
    });

    // Configure the system clock settings
    clocks.sys_clk = SysClkConfig {
        src: SysClkSrc::PllSys, // Set system clock source to system PLL
        div_int: 1,             // No division
        div_frac: 0,            // No fractional division
    };

    // Configure the USB clock settings
    clocks.usb_clk = Some(UsbClkConfig {
        src: UsbClkSrc::PllUsb, // Set USB clock source to USB PLL
        div: 1,                 // No division
        phase: 0,               // No phase shift
    });

    hal_configuration.clocks = clocks;
    hal_configuration
}

/// Verify the system clock frequency using a timer
///
/// * `milliseconds` - Duration in milliseconds to wait before measuring the system clock
pub async fn verify_clock_with_timer(milliseconds: u64) {
    let start = Instant::now(); // Start timer
    Timer::after_millis(milliseconds).await; // Wait for the specified duration
    let measured_elapsed = start.elapsed().as_micros(); // Stop timer
    let expected_elapsed = milliseconds * 1000;

    debug!(
        "[System clock frequency test] Actual: {} microseconds -> Measured: {} microseconds",
        expected_elapsed, measured_elapsed
    );

    // Calculate and print the clock accuracy percentage
    // -> (measured - expecsted) / expected * 100
    let accuracy_percent =
        (measured_elapsed as i32 - expected_elapsed as i32) as f32 / (expected_elapsed as i32) as f32 * 100.0;
    debug!(
        "[System clock frequency test] Clock accuracy: {}% off",
        accuracy_percent
    );
}

/// Display device clock frequencies
pub fn print_device_frequencies() {
    debug!("Device Clock Frequencies:");
    debug!("  - ROSC (Ring Oscil.): {} Hz", clocks::rosc_freq());
    debug!("  - XOSC (Crystal Oscil.): {} Hz", clocks::xosc_freq());
    debug!("  - SYS PLL (Phase-Lock Loop): {} Hz", clocks::pll_sys_freq());
    debug!("  - USB PLL (USB Phase-Lock Loop): {} Hz", clocks::pll_usb_freq());
    debug!("  - SYS CLK (System Clock): {} Hz", clocks::clk_sys_freq());
    debug!("  - REF CLK (Reference Clock): {} Hz", clocks::clk_ref_freq());
    debug!("  - PERI CLK (Peripheral Clock): {} Hz", clocks::clk_peri_freq());
    debug!("  - USB CLK (USB Clock): {} Hz", clocks::clk_usb_freq());
    debug!("  - ADC CLK (Analog-Digital Converter): {} Hz", clocks::clk_adc_freq());
}
