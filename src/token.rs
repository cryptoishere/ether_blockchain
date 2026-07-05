use std::convert::TryInto;
use std::sync::Arc;
use alloy::eips::BlockId;
use alloy::network::ReceiptResponse;
use alloy::rpc::types::TransactionReceipt;
use alloy::primitives::{Address, TxHash, U256};
use alloy::providers::Provider;
use anyhow::Result;

use crate::client::AppProvider;
use crate::components::{BroadcastedTransaction, IERC20, PreparedTransfer};
use crate::utils;

pub struct TokenManager {
    contract: IERC20::IERC20Instance<Arc<AppProvider>>,
    decimals: u8,
    symbol: String,
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

                return Err(anyhow::anyhow!("Could not resolve contract decimals"))
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

    /// Helper: compute max_fee and priority_fee (EIP-1559)
    async fn estimate_eip1559_fees(
        provider: &Arc<AppProvider>,
    ) -> Result<(U256, U256)> {
        let block = provider
            .get_block(BlockId::latest())
            .await?
            .ok_or_else(|| anyhow::anyhow!("No latest block"))?;

        let base_fee = block
            .header
            .base_fee_per_gas
            .ok_or_else(|| anyhow::anyhow!("Chain does not support EIP-1559"))?;

        let priority_fee = U256::from(2_000_000_000u64); // 2 gwei

        let base_fee_u256 = U256::from(base_fee);
        let max_fee_per_gas = base_fee_u256.checked_mul(U256::from(2)).unwrap() + priority_fee;

        Ok((max_fee_per_gas, priority_fee))
    }

    /// Cost estimates
    pub async fn prepare_transfer(
        &self,
        to: Address,
        amount_wei: U256,
    ) -> Result<PreparedTransfer> {
        let call = self.contract.transfer(to, amount_wei);

        let gas_estimate = call.estimate_gas().await?;

        let (max_fee_per_gas, max_priority_fee_per_gas) =
            Self::estimate_eip1559_fees(self.contract.provider()).await?;

        Ok(PreparedTransfer {
            gas_estimate,
            max_fee_per_gas,
            max_priority_fee_per_gas,
        })
    }

    pub async fn broadcast_transfer(
        &self,
        to: Address,
        amount_wei: U256,
        prepared: &PreparedTransfer,
    ) -> Result<BroadcastedTransaction> {
        let provider = self.contract.provider();

        let submitted_block = provider.get_block_number().await?;

        let max_fee_u128 = prepared.max_fee_per_gas.try_into()
            .map_err(|_| anyhow::anyhow!("max_fee_per_gas overflowed u128"))?;
        let max_priority_u128 = prepared.max_priority_fee_per_gas.try_into()
            .map_err(|_| anyhow::anyhow!("max_priority_fee_per_gas overflowed u128"))?;

        let tx = self
            .contract
            .transfer(to, amount_wei)
            // .max_fee_per_gas(max_fee.to::<u128>())
            // .max_priority_fee_per_gas(max_priority.to::<u128>())
            .max_fee_per_gas(max_fee_u128)
            .max_priority_fee_per_gas(max_priority_u128)
            .send()
            .await?;

        Ok(BroadcastedTransaction {
            hash: *tx.tx_hash(),
            submitted_block,
        })
    }

    pub async fn wait_for_receipt(
        &self,
        hash: TxHash,
        submitted_block: u64,
        max_blocks_wait: u64,
        wait_time: u64,
    ) -> Result<Option<TransactionReceipt>> {
        let provider = self.contract.provider();

        loop {
            if let Some(receipt) = provider.get_transaction_receipt(hash).await? {
                receipt.ensure_success()?;
                return Ok(Some(receipt));
            }

            let current_block = provider
                .get_block_number()
                .await?;

            if current_block.saturating_sub(submitted_block) > max_blocks_wait {
                // tx is very likely stuck / underpriced
                return Ok(None);
            }

            tokio::time::sleep(tokio::time::Duration::from_secs(wait_time)).await;
        }
    }

    pub async fn get_latest_block(&self) -> Result<u64> {
        let block = self.contract.provider().get_block_number().await?;
        Ok(block)
    }
}
