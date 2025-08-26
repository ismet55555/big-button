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

    // Load configurations from local 'configs.json' file
    let configs_keys = ["number_of_messages"];
    load_configs(out_dir.to_str().unwrap(), configs_keys)
        .unwrap_or_else(|error| panic!("[ERROR] {error:?}"))
}

/// Loads configuration values from configs.json and generates a configs.rs file
/// with Rust constants for use in the embedded application
///
/// # Arguments
/// * `out_dir` - The target build directory path
/// * `config_keys` - Array of configuration keys to extract from configs.json
fn load_configs(out_dir: &str, config_keys: [&str; 1]) -> io::Result<()> {
    println!("[BUILD TASK] LOADING PROGRAM CONFIGURATIONS");
    println!("Configuration output directory: {out_dir:?}");

    println!("Creating new blank 'configs.rs' file within the output directory ...");
    let dest_path = Path::new(&out_dir).join("configs.rs");
    let mut f = fs::File::create(dest_path)?;

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
            f,
            "pub const {}: &str = {:?};",
            config_key.to_uppercase(),
            config_values[config_key].as_str().unwrap()
        )?;
        println!(" ... value loaded!")
    }

    println!("Successfully loaded all program configurations from 'configs.json'!");

    Ok(())
}
