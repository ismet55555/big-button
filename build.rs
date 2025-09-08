//! This build script runs during code compilation
//! Runs on host computer, where `std` Rust tools are available here
//! Reference: https://doc.rust-lang.org/cargo/reference/build-scripts.html

use std::env;
use std::fs;
use std::io;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

/// Main build script entry point
fn main() {
    // Put `memory.x` in our output directory
    let out_dir = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
    fs::File::create(out_dir.join("memory.x"))
        .unwrap()
        .write_all(include_bytes!("memory.x"))
        .unwrap();

    // Ensure 'memory.x' is on the linker search path
    println!("cargo:rustc-link-search={}", out_dir.display());

    // Re-run for any changes to the following files
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=memory.x");
    println!("cargo:rerun-if-changed=configs.json");
    println!("cargo:rerun-if-changed=secrets.json");

    // Load configurations from local 'configs.json' file
    let configs_keys = [
        "clock_system_frequency_mhz",
        "clock_usb_frequency_mhz",
        "clock_peripheral_divider",
        "clock_adc_frequency_mhz",
        "clock_reference_divider",
    ];
    load_configs(out_dir.to_str().unwrap(), configs_keys).unwrap_or_else(|error| panic!("[ERROR] {error:?}"));

    // Load secrets from local 'secrets.json' file with XOR obfuscation
    let secrets_keys = ["super_secret_info"];
    load_secrets(out_dir.to_str().unwrap(), secrets_keys).unwrap_or_else(|error| panic!("[ERROR] {error:?}"));
}

/// Loads configuration values from configs.json and generates a configs.rs file
/// with Rust constants for use in the embedded application
///
/// # Arguments
/// * `out_dir` - The target build directory path
/// * `config_keys` - Array of configuration keys to extract from configs.json
fn load_configs(out_dir: &str, config_keys: [&str; 5]) -> io::Result<()> {
    println!("[BUILD TASK] LOADING PROGRAM CONFIGURATIONS");
    println!("Configuration output directory: {out_dir:?}");

    println!("Creating new blank 'configs.rs' file within the output directory ...");
    let dest_path = Path::new(&out_dir).join("configs.rs");
    let mut output_file = fs::File::create(dest_path)?;

    let project_root_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    println!("Project root directory: {project_root_dir:?}");

    let configs_path = Path::new(&project_root_dir).join("configs.json");
    println!("Looking for configs file at filepath: {configs_path:?}");
    if !configs_path.exists() {
        let error_message = "Failed to find 'configs.json' in the project root directory. 
                             Please see instructions within README.md on creating this file";
        return Err(io::Error::new(io::ErrorKind::NotFound, error_message));
    };

    println!("Reading raw text from 'configs.json' ...");
    let contents_raw_string = fs::read_to_string(configs_path)?;

    println!("Parsing 'configs.json' as a JSON file ...");
    let config_values: serde_json::Value = serde_json::from_str(&contents_raw_string)?;

    // Check all required keys
    for config_key in config_keys.iter() {
        if !config_values.as_object().unwrap().contains_key(*config_key) {
            let error_message = format!("Key '{}' not found in 'configs.json' file", config_key);
            return Err(io::Error::new(io::ErrorKind::InvalidData, error_message));
        }
    }

    println!("Writing from 'configs.json' to 'configs.rs' as UPPERCASE constants ...");
    for config_key in config_keys.iter() {
        print!("  - {}", config_key.to_uppercase());
        writeln!(
            output_file,
            "pub const {}: &str = {:?};",
            config_key.to_uppercase(),
            config_values[config_key].as_str().unwrap()
        )?;
        println!(" ... value loaded!")
    }

    println!("Successfully loaded all program configurations from 'configs.json'!");

    Ok(())
}

/// Loads secret values from secrets.json and generates a secret.rs file
/// with Rust obfuscated constants for use in the embedded application
///
/// # Arguments
/// * `out_dir` - The target build directory path
/// * `secret_keys` - Array of configuration keys to extract from secrets.json
fn load_secrets(out_dir: &str, secret_keys: [&str; 1]) -> io::Result<()> {
    println!("[BUILD TASK] LOADING OBFUSCATED SECRETS");
    println!("Secrets output directory: {out_dir:?}");

    println!("Creating new blank 'secrets.rs' file within the output directory ...");
    let dest_path = Path::new(&out_dir).join("secrets.rs");
    let mut output_file = fs::File::create(dest_path)?;

    let project_root_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let secrets_path = Path::new(&project_root_dir).join("secrets.json");

    println!("Looking for secrets file at filepath: {secrets_path:?}");
    if !secrets_path.exists() {
        let error_message = "Failed to find 'secrets.json' in the project root directory. 
                             Please create a 'secrets.json' file with your secret values.";
        return Err(io::Error::new(io::ErrorKind::NotFound, error_message));
    };

    println!("Reading and parsing 'secrets.json' ...");
    let contents_raw_string = fs::read_to_string(secrets_path)?;
    let secret_values: serde_json::Value = serde_json::from_str(&contents_raw_string)?;

    // Verify all keys exist
    for secret_key in secret_keys.iter() {
        if !secret_values.as_object().unwrap().contains_key(*secret_key) {
            let error_message = format!("Key '{}' not found in 'secrets.json' file", secret_key);
            return Err(io::Error::new(io::ErrorKind::InvalidData, error_message));
        }
    }

    // XOR Key - Alternating bit pattern (01011010) for simple, reversible obfuscation
    let xor_key: u8 = 0xA5;
    println!("Writing XOR key and obfuscated secrets to 'secrets.rs' ...");
    writeln!(output_file, "pub const XOR_KEY: u8 = 0x{:02X};", xor_key)?;

    // Write the obfuscated constants to secrets.rs
    for secret_key in secret_keys.iter() {
        print!("  - {} (obfuscated)", secret_key.to_uppercase());

        let secret_value = secret_values[secret_key].as_str().unwrap();

        // XOR encode the secret
        let obfuscated: Vec<u8> = secret_value.bytes().map(|b| b ^ xor_key).collect();

        // Write the obfuscated constant as a byte array
        writeln!(
            output_file,
            "pub const {}_OBFUSCATED: &[u8] = &{:?};",
            secret_key.to_uppercase(),
            obfuscated
        )?;

        println!(" ... obfuscated and stored!")
    }

    println!("Successfully loaded and obfuscated all secrets from 'secrets.json'!");

    Ok(())
}
