use anyhow::{Context, Result};
use alloy::primitives::Address;
use zeroize::Zeroizing;
use std::env;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub enum UrlWithProtocolType {
    HTTP(String),
    WS(String),
}

/// One ERC-20 (or BEP-20) token the bot cares about.
#[derive(Debug, Clone)]
pub struct TokenConfig {
    pub symbol: String, // "USDT", "USDC", ...
    pub contract: Address,
    pub decimals: Option<u8>, // None = query on-chain
}

#[derive(Debug, Clone)]
pub struct Config {
    pub rpc_url: UrlWithProtocolType,
    pub phrase: Zeroizing<String>,
    pub password: Option<Zeroizing<String>>,
    pub tokens: Vec<TokenConfig>,
    pub recipient: Address,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let rpc_url = if let Ok(key) = env::var("PLATFORM_BSC_API") {
            UrlWithProtocolType::HTTP(key)
        } else if let Ok(key) = env::var("PLATFORM_BSC_WS") {
            UrlWithProtocolType::WS(key)
        } else {
            anyhow::bail!("Cannot build config");
        };

        // Build tokens from env. Add more entries as needed.
        let mut tokens = Vec::new();

        if let Ok(addr) = env::var("USDT_CONTRACT_BSC") {
            tokens.push(TokenConfig {
                symbol: "USDT".into(),
                contract: Address::from_str(&addr).context("bad USDT_CONTRACT_BSC")?,
                decimals: None,
            });
        }
        if let Ok(addr) = env::var("USDC_CONTRACT_BSC") {
            tokens.push(TokenConfig {
                symbol: "USDC".into(),
                contract: Address::from_str(&addr).context("bad USDC_CONTRACT_BSC")?,
                decimals: None,
            });
        }

        if tokens.is_empty() {
            anyhow::bail!("No token contracts configured (USDT_CONTRACT_BSC / USDC_CONTRACT_BSC)");
        }

        Ok(Self {
            rpc_url,
            phrase: Zeroizing::new(
                env::var("PLATFORM_BANK_GENESIS_ADDRESS_PASSPHRASE")
                    .context("PLATFORM_BANK_GENESIS_ADDRESS_PASSPHRASE not set")?,
            ),
            password: env::var("PLATFORM_BANK_GENESIS_ADDRESS_PASSPHRASE_PASSWORD")
                .ok()
                .map(Zeroizing::new),
            tokens,
            recipient: Address::from_str(
                &env::var("PLATFORM_BSC_ADDRESS_OF_XBTS").context("XBTS_BSC_WALLET not set")?,
            )?,
        })
    }
}
