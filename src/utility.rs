//! Utility functions for the embedded application
//! This module contains helper functions that can be used throughout the project

#![allow(dead_code)] // only used for development

use heapless::String;

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
