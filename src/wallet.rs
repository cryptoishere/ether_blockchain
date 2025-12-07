use anyhow::Result;
use alloy::signers::local::{LocalSignerError, MnemonicBuilder, PrivateKeySigner};
use alloy::signers::local::coins_bip39::{English, Mnemonic};
use rand::rngs::OsRng;
use zeroize::Zeroizing;

pub struct Wallet;

impl Wallet {
    /// Generate a secure wallet using OS entropy + in-memory secret zeroization.
    ///
    /// Returns:
    /// - Zeroizing<String>: the mnemonic phrase (auto-zeroized when dropped)
    /// - PrivateKeySigner: alloy signer built from the phrase & password
    pub fn generate_wallet(
        size: usize,
        index: u32,
        password: Option<&str>,
    ) -> Result<(Zeroizing<String>, PrivateKeySigner), LocalSignerError> 
    {
        let phrase = Self::generate_mnemonic(size)?;

        let signer = Self::build_signer(&phrase, password, index)?;

        Ok((phrase, signer))
    }

    fn generate_mnemonic(size: usize) -> Result<Zeroizing<String>, LocalSignerError> 
    {
        // Direct OS entropy (no userland state, no PRNG, no counter limits)
        let mut rng = OsRng;

        // 24-word = 256-bit entropy (strongest BIP-39 option)
        let mnemonic = Mnemonic::<English>::new_with_count(&mut rng, size)?;

        let phrase = Zeroizing::new(mnemonic.to_phrase());

        Ok(phrase)
    }

    pub fn build_signer(phrase: &Zeroizing<String>, password: Option<&str>, derivation_index: u32) -> Result<PrivateKeySigner, LocalSignerError>  {
        let mut builder = MnemonicBuilder::<English>::default()
            .phrase(phrase.as_str())
            .index(derivation_index)?;

        if let Some(pwd) = password {
            builder = builder.password(pwd);
        }

        let signer = builder.build()?;

        Ok(signer)
    }
}
