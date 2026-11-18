use std::io;

use alloy::signers::local::coins_bip39::{
    English, Entropy, Mnemonic, WordCount,
};
use alloy::signers::local::{
    LocalSignerError, MnemonicBuilder, PrivateKeySigner,
};
use rand::rngs::SysRng;
use rand::TryRng;
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
        let _word_count = match entropy_size {
            12 => WordCount::Twelve,
            15 => WordCount::Fifteen,
            18 => WordCount::Eighteen,
            21 => WordCount::TwentyOne,
            24 => WordCount::TwentyFour,
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

        let mut entropy_bytes = Zeroizing::new([0u8; 32]);

        let entropy_len = match entropy_size {
            12 => 16,
            15 => 20,
            18 => 24,
            21 => 28,
            24 => 32,
            _ => unreachable!(),
        };

        SysRng
            .try_fill_bytes(&mut entropy_bytes[..entropy_len])
            .map_err(|e| io::Error::other(format!("OS RNG failure: {e}")))?;

        let entropy = Entropy::from_slice(&entropy_bytes[..entropy_len])?;

        Ok(Mnemonic::<English>::new_from_entropy(entropy).to_phrase())
    }

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

    fn builder(
        phrase: &str,
        derivation_index: u32,
    ) -> Result<MnemonicBuilder<English>, LocalSignerError> {
        MnemonicBuilder::<English>::default()
            .phrase(phrase)
            .index(derivation_index)
    }
}
