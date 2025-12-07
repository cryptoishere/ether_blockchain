use alloy::primitives::U256;
use alloy::primitives::utils::{format_units, parse_units};
use anyhow::Result;

pub fn to_human_readable(amount: U256, decimals: u8) -> String {
    format_units(amount, decimals).unwrap_or_else(|_| "0.0".to_string())
}

pub fn from_human_readable(amount: f64, decimals: u8) -> Result<U256> {
    // Note: dealing with f64 for crypto amounts can lose precision. 
    // Ideally accept string inputs, but for this example:
    let s = format!("{:.18}", amount); // Format to string to avoid float weirdness
    Ok(parse_units(&s, decimals)?.into())
}
