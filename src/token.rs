use std::sync::Arc;
use alloy::sol;
use alloy::primitives::{Address, U256};
use alloy::providers::Provider;
use anyhow::{Result, Context};

use crate::client::AppProvider;
use crate::utils;

// Define the interface inline using sol!
// This removes dependencies on external JSON files
sol! {
    #[allow(missing_docs)]
    #[sol(rpc)]
    contract IERC20 {
        function balanceOf(address account) external view returns (uint256);
        function transfer(address to, uint256 amount) external returns (bool);
        function decimals() external view returns (uint8);
        event Transfer(address indexed from, address indexed to, uint256 value);
    }
}

pub struct TokenManager {
    contract: IERC20::IERC20Instance<Arc<AppProvider>>,
    decimals: u8,
    symbol: String,
}

impl TokenManager {
    pub async fn new(provider: Arc<AppProvider>, address: Address, symbol: &str) -> Result<Self> {
        let contract = IERC20::new(address, provider.clone());

        // Cache decimals for parsing
        let decimals_call = contract.decimals().call().await;
        let decimals = match decimals_call {
            Ok(d) => d,
            Err(_) => 18, // Default to 18 if call fails
        };

        Ok(Self { contract, decimals, symbol: symbol.to_string() })
    }

    pub fn get_decimals(&self) -> u8 {
        self.decimals
    }

    pub async fn get_balance_human(&self, address: Address) -> Result<String> {
        let bal = self.contract.balanceOf(address).call().await?;
        Ok(format!("{} {}", utils::to_human(bal, self.decimals), self.symbol))
    }

    /// Prepares, Estimates, and returns Human Readable Fee information
    pub async fn prepare_transfer(&self, to: Address, amount_human: &str) -> Result<(U256, String)> {
        let amount_wei = utils::from_human(amount_human, self.decimals)?;

        // Create the call builder
        let call = self.contract.transfer(to, amount_wei);

        // 1. Estimate Gas Units
        let gas_estimate = call.estimate_gas().await.context("Gas estimation failed")?;

        // 2. Get Gas Price
        let gas_price = self.contract.provider().get_gas_price().await.context("Failed to get gas price")?;

        // 3. Calculate Fee
        let fee_wei = gas_estimate as u128 * gas_price;
        let fee_human = utils::to_human(U256::from(fee_wei), 18); // Native token always 18 usually (BNB/ETH)

        Ok((amount_wei, format!("{} BNB/ETH", fee_human)))
    }

    pub async fn broadcast_transfer(&self, to: Address, amount_wei: U256) -> Result<String> {
        let tx = self.contract.transfer(to, amount_wei).send().await?;
        let hash = tx.tx_hash().clone();

        // Wait for receipt
        let receipt = tx.get_receipt().await?;

        Ok(format!("Tx: {:?} | Status: {:?}", hash, receipt.status()))
    }
}
