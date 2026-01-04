use alloy::primitives::U256;
use alloy::primitives::utils::{format_units, parse_units};
use anyhow::Result;

/// Convert human string → U256 using token decimals
pub fn from_human(amount: &str, decimals: u8) -> Result<U256> {
    Ok(parse_units(amount, decimals)?.into())
}

/// Convert U256 → human string using token decimals
pub fn to_human(amount: U256, decimals: u8) -> String {
    format_units(amount, decimals).unwrap_or_else(|_| "0".to_string())
}
