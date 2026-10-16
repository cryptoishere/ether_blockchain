use std::io;

use alloy::signers::local::coins_bip39::{English, Entropy, Mnemonic};
use alloy::signers::local::{LocalSignerError, MnemonicBuilder, PrivateKeySigner};
use zeroize::Zeroizing;

use crate::config::Config;

pub struct Wallet;

impl Wallet {
    pub fn generate_wallet(
        size: usize,
        derivation_index: u32,
        password: Option<&str>,
    ) -> Result<(Zeroizing<String>, PrivateKeySigner), LocalSignerError> {
        let phrase = Self::generate_mnemonic(size)?;

        let mut builder = Self::builder(phrase.as_str(), derivation_index)?;

        if let Some(pwd) = password {
            builder = builder.password(pwd);
        }

        let signer = builder.build()?;

        Ok((phrase, signer))
    }

    fn generate_mnemonic(entropy_size: usize) -> Result<Zeroizing<String>, LocalSignerError> {
        let entropy = match entropy_size {
            12 => Entropy::Sixteen(rand::random()),
            15 => Entropy::Twenty(rand::random()),
            18 => Entropy::TwentyFour(rand::random()),
            21 => Entropy::TwentyEight(rand::random()),
            24 => Entropy::ThirtyTwo(rand::random()),
            other => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!(
                        "invalid mnemonic word count: {other} (expected 12, 15, 18, 21, or 24)"
                    ),
                )
                .into());
            }
        };

        Ok(Zeroizing::new(Mnemonic::<English>::new_from_entropy(entropy).to_phrase()))
    }

    /// Build signer from mnemonic
    pub fn build_signer(
        config: &Config,
        derivation_index: u32,
    ) -> Result<PrivateKeySigner, LocalSignerError> {
        let mut builder = Self::builder(config.phrase.as_str(), derivation_index)?;

        if let Some(ref pwd) = config.password {
            builder = builder.password(pwd.as_str());
        }

        builder.build()
    }

    fn builder(phrase: &str, derivation_index: u32) -> Result<MnemonicBuilder, LocalSignerError> {
        MnemonicBuilder::<English>::default()
            .phrase(phrase)
            .index(derivation_index)
    }
}
