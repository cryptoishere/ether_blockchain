use alloy::primitives::Address;
use alloy::providers::fillers::FillProvider;
use alloy::providers::fillers::{JoinFill, GasFiller, BlobGasFiller, NonceFiller, ChainIdFiller, WalletFiller};
use anyhow::Result;
use alloy::providers::{ProviderBuilder, RootProvider};
use alloy::network::EthereumWallet;
use alloy::signers::local::{MnemonicBuilder, coins_bip39::English};
// use alloy::transports::http::{Client, Http};
use url::Url;

use crate::config::Config;

// Type alias for our specific provider stack
pub type AppProvider = FillProvider<JoinFill<JoinFill<alloy::providers::Identity, JoinFill<GasFiller, JoinFill<BlobGasFiller, JoinFill<NonceFiller, ChainIdFiller>>>>, WalletFiller<EthereumWallet>>, RootProvider>;

pub struct EvmClient {
    pub provider: AppProvider,
    pub address: Address,
}

impl EvmClient {
    pub async fn new(config: &Config) -> Result<Self> {
        // Build wallet from mnemonic
        let mut builder = MnemonicBuilder::<English>::default()
            .phrase(&config.phrase)
            .index(0)?;

        if let Some(pwd) = &config.password {
            builder = builder.password(pwd);
        }

        let wallet = builder.build()?;
        let address = wallet.address();

        // Connect provider
        let provider: AppProvider = ProviderBuilder::new()
            .wallet(EthereumWallet::from(wallet))
            .connect_http(Url::parse(&config.rpc_url)?);

        Ok(Self { provider, address })
    }
}
