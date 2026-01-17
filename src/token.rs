use std::sync::Arc;
use alloy::network::ReceiptResponse;
use alloy::sol;
use alloy::primitives::{Address, TxHash, U256};
use alloy::providers::Provider;
use anyhow::Result;

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

pub struct PreparedTransfer {
    gas_estimate: u64,
    gas_price: u128,
}

impl PreparedTransfer {
    pub fn calculate_fee(&self, decimals: Option<u8>) -> Result<(u128, String)> {
        let fee_wei = self.gas_estimate as u128 * self.gas_price;
        let fee_human = utils::to_human(U256::from(fee_wei), decimals.unwrap_or(18))?;

        Ok((fee_wei, fee_human))
    }

    pub fn get_gas_units(&self) -> u128 {
        self.gas_estimate as u128
    }

    pub fn get_gas_price(&self) -> u128 {
        self.gas_price
    }
}

pub struct BroadcastedTransaction {
    pub hash: TxHash,
    pub status: bool,
    pub cost: u128,
}

impl TokenManager {
    pub async fn new(provider: Arc<AppProvider>, contract_address: Address, symbol: &str) -> Result<Self> {
        let contract = IERC20::new(contract_address, provider.clone());

        // Cache decimals for parsing
        let decimals = match contract.decimals().call().await {
            Ok(d) => {
                log::debug!("decimals_call: {d}");
                d
            },
            Err(e) => {
                log::error!("decimals_call error: {e}");
                18 // Default to 18 if call fails
            }
        };

        Ok(Self { contract, decimals, symbol: symbol.to_string() })
    }

    pub fn get_decimals(&self) -> u8 {
        self.decimals
    }

    pub async fn get_balance_raw(&self, address: Address) -> Result<U256> {
        let bal = self.contract.balanceOf(address).call().await?;

        Ok(bal)
    }

    pub async fn get_chain_balance_raw(&self, address: Address) -> Result<U256> {
        let bal = self.contract.provider().get_balance(address).await?;

        Ok(bal)
    }

    pub async fn get_balance_human(&self, address: Address) -> Result<String> {
        let bal = self.contract.balanceOf(address).call().await?;
        Ok(format!("{} {}", utils::to_human(bal, self.decimals)?, self.symbol))
    }

    /// Cost estimates
    pub async fn prepare_transfer(&self, to: Address, amount_wei: U256) -> Result<PreparedTransfer> {
        // Create the call builder
        let call = self.contract.transfer(to, amount_wei);

        // 1. Estimate Gas Units
        let gas_estimate = call.estimate_gas().await?;

        // 2. Get Gas Price
        let gas_price = self.contract.provider().get_gas_price().await?;

        Ok(PreparedTransfer {
            gas_estimate,
            gas_price,
        })
    }

    pub async fn broadcast_transfer(&self, to: Address, amount_wei: U256) -> Result<BroadcastedTransaction> {
        let tx = self.contract.transfer(to, amount_wei).send().await?;
        let hash = tx.tx_hash().clone();

        let receipt = tx.get_receipt().await?;

        let transaction = BroadcastedTransaction {
            hash,
            status: receipt.status(),
            cost: receipt.cost(),
        };

        Ok(transaction)
    }
}
