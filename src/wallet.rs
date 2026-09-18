use std::io;

use alloy::signers::local::coins_bip39::{English, Entropy, Mnemonic};
use alloy::signers::local::{LocalSignerError, MnemonicBuilder, PrivateKeySigner};
use zeroize::Zeroizing;

pub struct Wallet;

impl Wallet {
    pub fn generate_wallet(
        size: usize,
        index: u32,
        password: Option<&str>,
    ) -> Result<(Zeroizing<String>, PrivateKeySigner), LocalSignerError> {
        let phrase = Self::generate_mnemonic(size)?;
        let signer = Self::build_signer(&phrase, password, index)?;
        Ok((phrase, signer))
    }

    fn generate_mnemonic(size: usize) -> Result<Zeroizing<String>, LocalSignerError> {
        let entropy = match size {
            12 => Entropy::Sixteen(rand::random()),
            15 => Entropy::Twenty(rand::random()),
            18 => Entropy::TwentyFour(rand::random()),
            21 => Entropy::TwentyEight(rand::random()),
            24 => Entropy::ThirtyTwo(rand::random()),
            other => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("invalid mnemonic word count: {other} (expected 12, 15, 18, 21, or 24)"),
                )
                .into());
            }
        };

        let mnemonic = Mnemonic::<English>::new_from_entropy(entropy);
        Ok(Zeroizing::new(mnemonic.to_phrase()))
    }

    pub fn build_signer(
        phrase: &Zeroizing<String>,
        password: Option<&str>,
        derivation_index: u32,
    ) -> Result<PrivateKeySigner, LocalSignerError> {
        let mut builder = MnemonicBuilder::<English>::default()
            .phrase(phrase.as_str())
            .index(derivation_index)?;

        if let Some(pwd) = password {
            builder = builder.password(pwd);
        }

        builder.build()
    }
}
