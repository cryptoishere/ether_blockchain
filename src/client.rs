use std::sync::Arc;

use alloy::primitives::Address;
use alloy::providers::fillers::FillProvider;
use alloy::providers::fillers::{JoinFill, GasFiller, BlobGasFiller, NonceFiller, ChainIdFiller, WalletFiller};
use alloy::providers::Identity;
use anyhow::Result;
use alloy::providers::{ProviderBuilder, RootProvider};
use alloy::network::EthereumWallet;
// use alloy::transports::http::{Client, Http};
use url::Url;

use crate::config::Config;
use crate::wallet::Wallet;

// Type alias for our specific provider stack
pub type AppProvider = FillProvider<JoinFill<JoinFill<Identity, JoinFill<GasFiller, JoinFill<BlobGasFiller, JoinFill<NonceFiller, ChainIdFiller>>>>, WalletFiller<EthereumWallet>>, RootProvider>;

pub struct EvmClient {
    pub provider: Arc<AppProvider>,
    pub address: Address,
}

impl EvmClient {
    pub async fn new(config: &Config) -> Result<Self> {
        // Build signer from mnemonic
        let wallet = Wallet::build_signer(&config.phrase, config.password.as_deref(), 0)?;
        let address = wallet.address();

        // Connect provider
        let provider: AppProvider = ProviderBuilder::new()
            .wallet(EthereumWallet::from(wallet))
            .connect_http(Url::parse(&config.rpc_url)?);

        Ok(Self { provider: Arc::new(provider), address })
    }
}
