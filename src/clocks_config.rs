//! Raspberry Pi Pico 2 W RP2350 clock/PLL setup
//! Functions to set, verify, and log effective system clock frequencies

use embassy_rp::clocks::{self, ClockConfig, PllConfig, SysClkConfig, SysClkSrc, UsbClkConfig, UsbClkSrc, XoscConfig};
use embassy_rp::clocks::{AdcClkConfig, AdcClkSrc, PeriClkSrc, RefClkConfig, RefClkSrc};
use embassy_rp::config::Config as HalConfig;
use embassy_time::{Instant, Timer};

use defmt::*;
use defmt_rtt as _;
use panic_probe as _;

/// Clock configuration structure to hold all clock settings for RP2350
pub struct ClockSettings {
    pub xosc_crystal_hz: u32,
    pub system_frequency_mhz: u32,
    pub usb_frequency_mhz: u32,
    pub peripheral_clock_divider: u8,
    pub adc_frequency_mhz: u32,
    pub reference_clock_divider: u8,
}

/// Calculate the PLL (Phase-Locked Loop) configuration for the system clock and USB clock
///
/// * `desired_freq_mhz` - Desired frequency in MHz
/// * `xosc_freq_hz` - Crystal oscillator frequency in Hz
/// * `post_div1` - Post divider 1 value
/// * `post_div2` - Post divider 2 value
///
/// The formula for calculating the PLL configuration is:
///    `fbdiv = (desired_freq_mhz * post_div1 * post_div2) / xosc_freq_mhz`
pub fn calculate_pll_config(desired_freq_mhz: u32, xosc_freq_hz: u32, post_div1: u8, post_div2: u8) -> PllConfig {
    const HZ_PER_MHZ: u32 = 1_000_000;

    let refdiv = 1u8; // Fixed reference divider
    let xosc_freq_mhz = xosc_freq_hz / HZ_PER_MHZ;

    // VCO target frequency in MHz (desired output * post dividers)
    let vco_target_mhz = desired_freq_mhz * post_div1 as u32 * post_div2 as u32;

    // Feedback divider: how many times to multiply the reference to hit the VCO target
    let fbdiv = vco_target_mhz / xosc_freq_mhz;

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

/// Configure all system clocks for RP2350
///
/// This function configures:
///   - System PLL and clock
///   - USB PLL and clock
///   - Peripheral clock source
///   - ADC clock
///   - Reference clock
pub fn configure_all_clocks(hal_configuration: &mut HalConfig, settings: ClockSettings) -> &mut HalConfig {
    // Calculate the PLL configuration for the system clock and USB clock
    let sys_pll_config = calculate_pll_config(settings.system_frequency_mhz, settings.xosc_crystal_hz, 6, 2);
    let usb_pll_config = calculate_pll_config(settings.usb_frequency_mhz, settings.xosc_crystal_hz, 6, 5);

    // Create and configure the external oscillator (XOSC) settings
    let mut clocks = ClockConfig::crystal(settings.xosc_crystal_hz);
    clocks.xosc = Some(XoscConfig {
        hz: settings.xosc_crystal_hz,  // Set XOSC frequency
        sys_pll: Some(sys_pll_config), // System PLL configuration
        usb_pll: Some(usb_pll_config), // USB PLL configuration
        delay_multiplier: 128,         // Used to calculate the startup delay (in cycles) for the crystal oscillator
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

    // Configure the peripheral clock source
    // RP2350 uses peri_clk_src field
    clocks.peri_clk_src = Some(match settings.peripheral_clock_divider {
        1 => PeriClkSrc::Sys, // Use system clock directly
        _ => {
            debug!("[NOTE] Peripheral clock divider is applied at runtime, not in clock config");
            PeriClkSrc::Sys // Still use system clock
        }
    });

    // Configure the ADC clock
    // ADC clock can be sourced from USB PLL for stable 48MHz operation
    clocks.adc_clk = Some(AdcClkConfig {
        src: AdcClkSrc::PllUsb, // Source from USB PLL (48MHz)
        div: (settings.usb_frequency_mhz / settings.adc_frequency_mhz) as u8, // Calculate divider
        phase: 0,               // No phase shift
    });

    // Configure the reference clock
    // Reference clock is typically used for watchdog and timers
    clocks.ref_clk = RefClkConfig {
        src: RefClkSrc::Xosc,                  // Source from crystal oscillator
        div: settings.reference_clock_divider, // Apply configured divider
    };

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
    let measured_diff = (measured_elapsed as i64 - expected_elapsed as i64).abs();

    // Use u64 to prevent overflow in calculation
    let accuracy_percent_int = ((measured_diff as u64 * 100) / expected_elapsed as u64) as u32;
    let accuracy_percent_frac = ((measured_diff as u64 * 10000) / expected_elapsed as u64 % 100) as u32;

    debug!(
        "[System clock frequency test] Clock accuracy: {}.{:02}% off",
        accuracy_percent_int, accuracy_percent_frac
    );
}

/// Display device clock frequencies with enhanced detail
pub fn print_device_frequencies() {
    debug!("Log Device Clock Frequencies:");
    debug!("[Oscillators]");
    debug!("  - ROSC (Ring Oscillator): {} Hz", clocks::rosc_freq());
    debug!("  - XOSC (Crystal Oscillator): {} Hz", clocks::xosc_freq());

    debug!("[PLLs (Phase-Locked Loops)]");
    debug!("  - SYS PLL: {} Hz", clocks::pll_sys_freq());
    debug!("  - USB PLL: {} Hz", clocks::pll_usb_freq());

    debug!("[System Clocks]");
    debug!("  - SYS CLK (System Clock): {} Hz", clocks::clk_sys_freq());
    debug!("  - REF CLK (Reference Clock): {} Hz", clocks::clk_ref_freq());
    debug!("  - PERI CLK (Peripheral Clock): {} Hz", clocks::clk_peri_freq());

    debug!("[Specialized Clocks]");
    debug!("  - USB CLK (USB Clock): {} Hz", clocks::clk_usb_freq());
    debug!("  - ADC CLK (ADC Clock): {} Hz", clocks::clk_adc_freq());
}
