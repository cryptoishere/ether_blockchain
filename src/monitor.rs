use alloy::primitives::{Address, B256, FixedBytes, U256, keccak256};
use alloy::rpc::types::Filter;
use alloy::providers::Provider;
use alloy::sol_types::SolEvent;
use alloy::sol;
use futures::StreamExt;
use anyhow::Result;
use tokio::sync::oneshot;
use tokio::sync::mpsc;
use tokio::select;

use crate::client::AppProvider;
use crate::utils::to_human;

// Re-declare event for decoding
sol! {
    event Transfer(address indexed from, address indexed to, uint256 value);
}

pub struct IncomingTransfer {
    pub tx_hash: B256,
    pub log_index: u64,
    pub block_number: u64,
    pub from: Address,
    pub to: Address,
    pub amount: U256,
}

pub async fn monitor(
    provider: &AppProvider,
    contract_addr: Address,
    destination_wallet: Address,
    decimals: u8,
    poll_interval: u64,
    mut shutdown: oneshot::Receiver<()>,
    tx: mpsc::Sender<IncomingTransfer>,
) -> Result<()> {
    log::debug!("--- Starting Monitor for {:?} ---", destination_wallet);

    let sig_hash: FixedBytes<32> = keccak256(Transfer::SIGNATURE);

    let filter = Filter::new()
        .event_signature(sig_hash)
        .address(contract_addr)
        .topic2(destination_wallet);

    // Create stream
    let mut sub = provider.watch_logs(&filter).await?;
    sub.set_poll_interval(std::time::Duration::from_secs(poll_interval));
    let mut stream = sub.into_stream();

    loop {
        select! {
            _ = &mut shutdown => {
                log::info!("Monitor shutting down...");
                break;
            }

            maybe_logs = stream.next() => {
                match maybe_logs {
                    Some(logs) => {
                        for log in logs {
                            let tx_hash = log.transaction_hash;
                            // let block_hash = log.block_hash;
                            let block_number = log.block_number;
                            let log_index = log.log_index;
                            let removed = log.removed;

                            match Transfer::decode_log(&log.into()) {
                                Ok(event) => {
                                    log::debug!("tx: {:?}", tx_hash);
                                    // log::debug!("block_hash: {:?}", block_hash);
                                    log::debug!("block_number: {:?}", block_number);
                                    log::debug!("log_index: {:?}", log_index);
                                    log::debug!("removed: {:?}", removed);

                                    let readable = to_human(event.value, decimals)?;
                                    log::debug!(
                                        "🚨 Incoming transfer from From: {:?} | Amount: {} USDT | Amount raw: {} USDT",
                                        event.from,
                                        readable,
                                        event.value.to_string()
                                    );

                                    match tx.send(IncomingTransfer {
                                        tx_hash: tx_hash.unwrap(),
                                        log_index: log_index.unwrap(),
                                        block_number: block_number.unwrap(),
                                        from: event.from,
                                        to: event.to,
                                        amount: event.value,
                                    }).await {
                                        Ok(_) => {
                                            log::debug!("Blockchain transfer data is sent");
                                        }
                                        Err(e) => {
                                            log::error!("Failed to send transfer data: {e}");

                                            break;
                                        }
                                    };
                                }
                                Err(e) => {
                                    log::error!("Error decoding log: {e}");
                                }
                            }
                        }
                    }

                    None => {
                        log::info!("Log stream ended.");
                        break;
                    }
                };
            }
        }
    }

    Ok(())
}

pub async fn monitor_ws(
    provider: &AppProvider,
    contract_addr: Address,
    destination_wallet: Address,
    decimals: u8,
    mut shutdown: oneshot::Receiver<()>,
    tx: mpsc::Sender<IncomingTransfer>,
) -> Result<()> {
    log::debug!("--- Starting Monitor for {:?} ---", destination_wallet);

    let sig_hash: FixedBytes<32> = keccak256(Transfer::SIGNATURE);

    let filter = Filter::new()
        .event_signature(sig_hash)
        .address(contract_addr)
        .topic2(destination_wallet);

    // Create stream
    let sub = provider.subscribe_logs(&filter).await?;
    let mut stream = sub.into_stream();

    loop {
        select! {
            _ = &mut shutdown => {
                log::info!("Monitor shutting down...");
                break;
            }

            maybe_log = stream.next() => {
                match maybe_log {
                    Some(log) => {
                        let tx_hash = log.transaction_hash;
                        // let block_hash = log.block_hash;
                        let block_number = log.block_number;
                        let log_index = log.log_index;
                        let removed = log.removed;

                        match Transfer::decode_log(&log.into()) {
                            Ok(event) => {
                                log::debug!("tx: {:?}", tx_hash);
                                // log::debug!("block_hash: {:?}", block_hash);
                                log::debug!("block_number: {:?}", block_number);
                                log::debug!("log_index: {:?}", log_index);
                                log::debug!("removed: {:?}", removed);

                                let readable = to_human(event.value, decimals)?;
                                log::debug!(
                                    "🚨 Incoming transfer from From: {:?} | Amount: {} USDT | Amount raw: {} USDT",
                                    event.from,
                                    readable,
                                    event.value.to_string()
                                );

                                match tx.send(IncomingTransfer {
                                    tx_hash: tx_hash.unwrap(),
                                    log_index: log_index.unwrap(),
                                    block_number: block_number.unwrap(),
                                    from: event.from,
                                    to: event.to,
                                    amount: event.value,
                                }).await {
                                    Ok(_) => {
                                        log::debug!("Blockchain transfer data is sent");
                                    }
                                    Err(e) => {
                                        log::error!("Failed to send transfer data: {e}");

                                        break;
                                    }
                                };
                            }
                            Err(e) => {
                                log::error!("Error decoding log: {e}");
                            }
                        };
                    }

                    None => {
                        log::info!("Log stream ended.");
                        break;
                    }
                }
            }
        }
    }

    Ok(())
}
