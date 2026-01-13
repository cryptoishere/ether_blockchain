# Ethereum Blockchain Client

A lightweight Rust client for interacting with the Ethereum blockchain and ERC-20 tokens (example: **USDT**).  
Supports balance queries, gas estimation, and broadcasting transactions through any Ethereum node provider.

---

## Features

- 🔧 Load configuration from `.env`
- 🔗 Connect to any Ethereum RPC provider
- 💰 Query ERC-20 token balances
- 📤 Estimate & broadcast token transfers
- 🪙 Built-in USDT token manager (configurable)

### Used providers
- https://www.quicknode.com/ 

---

## Setup

1. **Create a `.env` file** using the provided example:

   ```bash
   cp .env.example .env

2. If `.env` not supported:

    If your shell supports \ line continuation:
    ```bash
        MAIN_WALLET="your_main_wallet_here" \
        MAIN_PASSPHRASE="your_passphrase_here" \
        MAIN_PASSPHRASE_PASSWORD="optional_password_here" \
        BSC_API="https://ancient-attentive-surf.bsc.quiknode.pro/<API_KEY>" \
        USDT_CONTRACT_BSC="0x55d398326f99059fF775485246999027B3197955" \
        ETH_CONTRACT_BSC="0x2170Ed0880ac9A755fd29B2688956BD959F933F8" \
        XBTS_BSC_WALLET="0x413e8c21dd266ea5f4e7eebcd18498a66ec8dac7" \
        cargo run
    ```

    Alternative: export them first, then run Cargo

    If you want to keep the terminal session “loaded” with these values:

    ```bash
        export MAIN_WALLET="your_main_wallet_here"
        export MAIN_PASSPHRASE="your_passphrase_here"
        export MAIN_PASSPHRASE_PASSWORD="optional_password_here"

        export BSC_API="https://ancient-attentive-surf.bsc.quiknode.pro/<API_KEY>"
        export USDT_CONTRACT_BSC="0x55d398326f99059fF775485246999027B3197955"
        export ETH_CONTRACT_BSC="0x2170Ed0880ac9A755fd29B2688956BD959F933F8"

        export XBTS_BSC_WALLET="0x413e8c21dd266ea5f4e7eebcd18498a66ec8dac7"
    ```

    Then run:

    `cargo run`

    🔒 **Security Reminder**

    If these variables contain sensitive data (wallets, passphrases), avoid storing them in shell history:

    `HISTCONTROL=ignorespace`

    Then start your command with a space:

    ` export MAIN_WALLET="..."`


    Shell history won’t record it.

### Usage `Cargo.toml`

```
tokio = "1.48.0"
ether_alloy = { git = "https://github.com/cryptoishere/ether_blockchain.git" }
```

## Example
```rust
use std::sync::Arc;

use alloy::primitives::U256;
use ether_blockchain::client::EvmClient;
use ether_blockchain::config::Config;
use ether_blockchain::token::TokenManager;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Load Config
    let config = Config::from_env().unwrap();
    
    // 2. Initialize Client
    let client = EvmClient::new(&config).await.unwrap();

    // 3. Initialize Token Manager (USDT)
    let usdt_manager = TokenManager::new(
        Arc::new(client.provider), 
        config.usdt_contract, 
        "USDT"
    ).await.unwrap();

    // 4. Check Balance
    let balance = usdt_manager.get_balance_human(client.address).await.unwrap();
    println!("USDT Balance: {}", balance);

    // 5. Estimate & Send Transfer
    let amount_to_send = U256::from(2_500_000_000_000_000_000u64);

    // Estimate transfer cost
    let estimations = usdt_manager.prepare_transfer(
        config.recipient, 
        amount_to_send
    ).await.unwrap();

    println!("Estimated Fee: {:?}", estimations.calculate_fee(None));

    // Uncomment to broadcast:
    /*
    let result = usdt_manager
        .broadcast_transfer(config.recipient, amount_to_send)
        .await.unwrap();

    println!("Hash: {}", result.hash);
    println!("Status: {}", result.status);
    println!("Cost: {}", result.cost);
    */

    Ok(())
}
```
