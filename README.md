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

## Example
```rust
#[tokio::main]
async fn main() -> Result<()> {
    // 1. Load Config
    let config = config::Config::from_env()?;
    
    // 2. Initialize Client
    let client = client::EvmClient::new(&config).await?;

    // 3. Initialize Token Manager (USDT)
    let usdt_manager = token::TokenManager::new(
        &client.provider, 
        config.usdt_contract, 
        "USDT"
    ).await?;

    // 4. Check Balance
    let balance = usdt_manager.get_balance_human(client.address).await?;
    println!("USDT Balance: {}", balance);

    // 5. Estimate & Send Transfer
    let amount_to_send = 2.5;

    // Estimate transfer cost
    let (wei_amount, fee_readable) = usdt_manager.prepare_transfer(
        config.recipient, 
        amount_to_send
    ).await?;

    println!("Estimated Fee: {}", fee_readable);

    // Uncomment to broadcast:
    /*
    let result = usdt_manager
        .broadcast_transfer(config.recipient, wei_amount)
        .await?;

    println!("Broadcast Result: {:?}", result);
    */

    Ok(())
}
```