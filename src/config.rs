use anyhow::{Context, Result};
use alloy::primitives::Address;
use std::env;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct Config {
    pub rpc_url: String,
    pub phrase: String,
    pub password: Option<String>,
    pub usdt_contract: Address,
    pub recipient: Address,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        // dotenvy::dotenv().ok();

        Ok(Self {
            rpc_url: env::var("BSC_API").context("BSC_API not set")?,
            phrase: env::var("MAIN_PASSPHRASE").context("MAIN_PASSPHRASE not set")?,
            password: env::var("MAIN_PASSPHRASE_PASSWORD").ok(),
            usdt_contract: Address::from_str(&env::var("USDT_CONTRACT_BSC").context("USDT_CONTRACT_BSC not set")?)?,
            recipient: Address::from_str(&env::var("XBTS_BSC_WALLET").context("XBTS_BSC_WALLET not set")?)?,
        })
    }
}
