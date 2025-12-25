use alloy::primitives::{Address as EtheriumAddress, address};
use std::str::FromStr;

pub struct Address;

const USDT_BSC: EtheriumAddress =
    address!("0x55d398326f99059fF775485246999027B3197955");

impl Address {
    pub fn is_valid_eth_address(addr: &str) -> bool {
        EtheriumAddress::from_str(addr).is_ok()
    }

    pub fn is_strict_checksum(addr: &str) -> bool {
        if let Ok(parsed) = EtheriumAddress::from_str(addr) {
            let checksummed = parsed.to_checksum(None);
            addr == checksummed
        } else {
            false
        }
    }

    pub fn is_usdt_mainnet(addr: &str) -> bool {
        EtheriumAddress::from_str(addr)
            .map(|a| a == USDT_BSC)
            .unwrap_or(false)
    }

    pub fn validate_token(input: &str) -> Result<EtheriumAddress, &'static str> {
        let addr = EtheriumAddress::parse_checksummed(input, None)
            .map_err(|_| "Invalid address or checksum")?;

        if addr == USDT_BSC {
            Ok(addr)
        } else {
            Err("Unsupported token")
        }
    }
}
