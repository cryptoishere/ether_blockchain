use alloy::primitives::{Address, FixedBytes, keccak256};
use alloy::rpc::types::Filter;
use alloy::providers::Provider;
use alloy::sol_types::SolEvent;
use alloy::sol;
use futures::StreamExt;
use anyhow::Result;

use crate::client::AppProvider;
use crate::utils::to_human_readable;

// Re-declare event for decoding
sol! {
    event Transfer(address indexed from, address indexed to, uint256 value);
}

pub async fn monitor(
    provider: &AppProvider,
    contract_addr: Address,
    destination_wallet: Address,
    decimals: u8
) -> Result<()> {
    log::debug!("--- Starting Monitor for {:?} ---", destination_wallet);

    let sig_hash: FixedBytes<32> = keccak256(Transfer::SIGNATURE);

    // Filter: Contract = USDT, Topic2 (to) = My Wallet
    let filter = Filter::new()
        .event_signature(sig_hash)
        .address(contract_addr)
        .topic2(destination_wallet);

    // Create stream
    let sub = provider.watch_logs(&filter).await?;
    let mut stream = sub.into_stream();

    while let Some(logs) = stream.next().await {
        for log in logs {
            match Transfer::decode_log(&log.into()) {
                Ok(event) => {
                    let readable = to_human_readable(event.value, decimals);
                    log::debug!(
                        "🚨 INCOMING TRANSACTION! From: {:?} | Amount: {} USDT",
                        event.from, readable
                    );
                }
                Err(e) => log::error!("Error decoding log: {:?}", e),
            }
        }

        log::debug!("break");
        break;
    }

    Ok(())
}
