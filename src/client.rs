use std::sync::Arc;

use alloy::network::EthereumWallet;
use alloy::primitives::Address;
use alloy::providers::fillers::FillProvider;
use alloy::providers::fillers::{
    BlobGasFiller, ChainIdFiller, GasFiller, JoinFill, NonceFiller, WalletFiller,
};
use alloy::providers::{Identity, WsConnect};
use alloy::providers::{ProviderBuilder, RootProvider};
use anyhow::Result;
use url::Url;

use crate::config::{Config, UrlWithProtocolType};
use crate::wallet::Wallet;

// Type alias for our specific provider stack
pub type AppProvider = FillProvider<
    JoinFill<
        JoinFill<
            Identity,
            JoinFill<GasFiller, JoinFill<BlobGasFiller, JoinFill<NonceFiller, ChainIdFiller>>>,
        >,
        WalletFiller<EthereumWallet>,
    >,
    RootProvider,
>;

pub struct EvmClient {
    pub provider: Arc<AppProvider>,
    pub address: Address,
}

impl EvmClient {
    pub async fn new(config: &Config) -> Result<Self> {
        let wallet = Wallet::build_signer(config, 0)?;
        let address = wallet.address();

        let builder = ProviderBuilder::new().wallet(EthereumWallet::from(wallet));

        let provider = match config.rpc_url {
            UrlWithProtocolType::HTTP(ref url) => {
                log::debug!("Start HTTP EvmClient connection");
                builder.connect_http(Url::parse(url)?)
            }
            UrlWithProtocolType::WS(ref url) => {
                log::debug!("Start WS EvmClient connection");
                builder.connect_ws(WsConnect::new(url)).await?
            }
        };

        Ok(Self {
            provider: Arc::new(provider),
            address,
        })
    }
}
