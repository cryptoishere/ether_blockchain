use alloy::sol;
use alloy::primitives::{TxHash, U256};
use anyhow::Result;

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

pub struct PreparedTransfer {
    pub gas_estimate: u64,
    pub max_fee_per_gas: U256,
    pub max_priority_fee_per_gas: U256,
}

impl PreparedTransfer {
    /// Returns (fee_in_wei, human_readable)
    pub fn calculate_fee(&self, decimals: Option<u8>) -> Result<(U256, String)> {
        // max_fee_per_gas * gas_units
        let fee_wei = self.max_fee_per_gas.checked_mul(U256::from(self.gas_estimate)).unwrap();

        let fee_human = utils::to_human(fee_wei, decimals.unwrap_or(18))?;

        Ok((fee_wei, fee_human))
    }

    pub fn get_gas_units(&self) -> u64 {
        self.gas_estimate
    }

    pub fn get_max_fee_per_gas(&self) -> U256 {
        self.max_fee_per_gas
    }

    pub fn get_max_priority_fee_per_gas(&self) -> U256 {
        self.max_priority_fee_per_gas
    }
}

pub struct BroadcastedTransaction {
    pub hash: TxHash,
    pub submitted_block: u64,
}
