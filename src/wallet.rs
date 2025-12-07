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
        password: Option<&str>,
    ) -> Result<(Zeroizing<String>, PrivateKeySigner), LocalSignerError> 
    {
        // ---- HIGH-SECURITY RNG ----
        // Direct OS entropy (no userland state, no PRNG, no counter limits)
        let mut rng = OsRng;

        // ---- GENERATE MNEMONIC ----
        // 24-word = 256-bit entropy (strongest BIP-39 option)
        let mnemonic = Mnemonic::<English>::new_with_count(&mut rng, 24)?;

        // Convert phrase to String, but immediately wrap in Zeroizing<>
        let phrase: Zeroizing<String> = Zeroizing::new(mnemonic.to_phrase());

        // ---- BUILD SIGNER (NO EXTRA COPIES) ----
        // MnemonicBuilder only borrows the phrase, does *not* clone
        let mut signer_builder = MnemonicBuilder::<English>::default()
            .phrase(&*phrase);

        if let Some(pwd) = password {
            signer_builder = signer_builder.password(pwd)   // password is not cloned internally
        } 

        let signer = signer_builder.build()?;

        Ok((phrase, signer))
    }
}
