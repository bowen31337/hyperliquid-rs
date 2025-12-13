//! Wallet functionality for Hyperliquid SDK

use crate::crypto::signing::{sign_l1_action, sign_user_signed_action};
use crate::crypto::types::*;
use crate::crypto::nonce::{generate_nonce, generate_timestamp_nonce};
use crate::error::HyperliquidError;
use k256::ecdsa::{SigningKey, VerifyingKey};
use serde_json::Value;
use std::str::FromStr;

/// Private key wrapper for secure key handling
#[derive(Debug, Clone)]
pub struct PrivateKey {
    inner: SigningKey,
}

impl PrivateKey {
    /// Generate a new random private key
    pub fn generate() -> Result<Self, HyperliquidError> {
        // Generate 32 random bytes using the secure random number generator
        let mut rng = rand::thread_rng();
        let mut key_bytes = [0u8; 32];
        rng.fill_bytes(&mut key_bytes);

        // Create signing key from the random bytes
        let signing_key = SigningKey::from_bytes(&key_bytes.into())
            .map_err(|e| HyperliquidError::Signing(format!("Failed to create private key: {}", e)))?;

        Ok(Self { inner: signing_key })
    }
}

impl PrivateKey {
    /// Create a new private key from hex string
    pub fn from_hex(hex_key: &str) -> Result<Self, HyperliquidError> {
        let key_bytes = hex::decode(hex_key.trim_start_matches("0x"))
            .map_err(|e| HyperliquidError::Signing(format!("Invalid private key hex: {}", e)))?;

        if key_bytes.len() != 32 {
            return Err(HyperliquidError::Signing(
                "Private key must be exactly 32 bytes".to_string(),
            ));
        }

        let signing_key = SigningKey::from_bytes(&key_bytes.into())
            .map_err(|e| HyperliquidError::Signing(format!("Invalid private key: {}", e)))?;

        Ok(Self { inner: signing_key })
    }

    /// Get the private key as hex string
    pub fn to_hex(&self) -> String {
        format!("0x{}", hex::encode(self.inner.to_bytes()))
    }

    /// Get the public address (20 bytes)
    pub fn address(&self) -> String {
        let verifying_key = VerifyingKey::from(&self.inner);
        let pub_key_bytes = verifying_key.to_sec1_bytes();

        // Take the keccak256 hash of the public key (excluding the 0x04 prefix)
        let mut hasher = k256::sha3::Keccak256::new();
        hasher.update(&pub_key_bytes[1..]); // Skip the 0x04 prefix
        let hash = hasher.finalize();

        // Take the last 20 bytes
        let address_bytes = &hash[12..];
        format!("0x{}", hex::encode(address_bytes))
    }

    /// Get the inner signing key (for advanced usage)
    pub fn inner(&self) -> &SigningKey {
        &self.inner
    }
}

impl FromStr for PrivateKey {
    type Err = HyperliquidError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_hex(s)
    }
}

/// Ethereum wallet for signing transactions
#[derive(Debug, Clone)]
pub struct Wallet {
    private_key: PrivateKey,
    is_mainnet: bool,
}

impl Wallet {
    /// Generate a new random wallet for mainnet
    pub fn generate_mainnet() -> Result<Self, HyperliquidError> {
        let private_key = PrivateKey::generate()?;
        Ok(Self {
            private_key,
            is_mainnet: true,
        })
    }

    /// Generate a new random wallet for testnet
    pub fn generate_testnet() -> Result<Self, HyperliquidError> {
        let private_key = PrivateKey::generate()?;
        Ok(Self {
            private_key,
            is_mainnet: false,
        })
    }

    /// Create a new wallet from private key
    pub fn new(private_key: &str, is_mainnet: bool) -> Result<Self, HyperliquidError> {
        let private_key = PrivateKey::from_hex(private_key)?;
        Ok(Self {
            private_key,
            is_mainnet,
        })
    }

    /// Create a wallet for mainnet
    pub fn mainnet(private_key: &str) -> Result<Self, HyperliquidError> {
        Self::new(private_key, true)
    }

    /// Create a wallet for testnet
    pub fn testnet(private_key: &str) -> Result<Self, HyperliquidError> {
        Self::new(private_key, false)
    }

    /// Get the wallet address
    pub fn address(&self) -> String {
        self.private_key.address()
    }

    /// Get the private key
    pub fn private_key(&self) -> &PrivateKey {
        &self.private_key
    }

    /// Check if this is a mainnet wallet
    pub fn is_mainnet(&self) -> bool {
        self.is_mainnet
    }

    /// Sign an L1 action (order placement, cancellation, etc.)
    pub fn sign_l1_action(
        &self,
        action: &Value,
        vault_address: Option<&str>,
        nonce: u64,
        expires_after: Option<u64>,
    ) -> Result<Signature, HyperliquidError> {
        sign_l1_action(
            &self.private_key.to_hex(),
            action,
            vault_address,
            nonce,
            expires_after,
            self.is_mainnet,
        )
    }

    /// Sign an L1 action with auto-generated nonce
    pub fn sign_l1_action_with_nonce(
        &self,
        action: &Value,
        vault_address: Option<&str>,
        expires_after: Option<u64>,
    ) -> Result<Signature, HyperliquidError> {
        let nonce = generate_nonce();
        self.sign_l1_action(action, vault_address, nonce, expires_after)
    }

    /// Sign an L1 action with timestamp-based nonce
    pub fn sign_l1_action_with_timestamp_nonce(
        &self,
        action: &Value,
        vault_address: Option<&str>,
        expires_after: Option<u64>,
    ) -> Result<Signature, HyperliquidError> {
        let nonce = generate_timestamp_nonce();
        self.sign_l1_action(action, vault_address, nonce, expires_after)
    }

    /// Sign a user-signed action (USDC transfer, withdrawal, etc.)
    pub fn sign_user_signed_action(
        &self,
        action: &Value,
        payload_types: &[EIP712Type],
        primary_type: &str,
    ) -> Result<Signature, HyperliquidError> {
        sign_user_signed_action(
            &self.private_key.to_hex(),
            action,
            payload_types,
            primary_type,
            self.is_mainnet,
        )
    }

    /// Sign a USDC transfer action
    pub fn sign_usd_transfer(&self, action: &Value) -> Result<Signature, HyperliquidError> {
        self.sign_user_signed_action(action, action_types::USD_SEND, "HyperliquidTransaction:UsdSend")
    }

    /// Sign a spot transfer action
    pub fn sign_spot_transfer(&self, action: &Value) -> Result<Signature, HyperliquidError> {
        self.sign_user_signed_action(action, action_types::SPOT_TRANSFER, "HyperliquidTransaction:SpotSend")
    }

    /// Sign a withdrawal action
    pub fn sign_withdraw(&self, action: &Value) -> Result<Signature, HyperliquidError> {
        self.sign_user_signed_action(action, action_types::WITHDRAW, "HyperliquidTransaction:Withdraw")
    }

    /// Sign a USDC class transfer action
    pub fn sign_usdc_class_transfer(&self, action: &Value) -> Result<Signature, HyperliquidError> {
        self.sign_user_signed_action(
            action,
            action_types::USDC_CLASS_TRANSFER,
            "HyperliquidTransaction:UsdcClassTransfer",
        )
    }

    /// Sign a token delegate action (staking)
    pub fn sign_token_delegate(&self, action: &Value) -> Result<Signature, HyperliquidError> {
        self.sign_user_signed_action(
            action,
            action_types::TOKEN_DELEGATE,
            "HyperliquidTransaction:TokenDelegate"
        )
    }

    /// Sign a send asset action (cross-DEX transfer)
    pub fn sign_send_asset(&self, action: &Value) -> Result<Signature, HyperliquidError> {
        self.sign_user_signed_action(
            action,
            action_types::SEND_ASSET,
            "HyperliquidTransaction:SendAsset"
        )
    }

    /// Sign an approve builder fee action
    pub fn sign_approve_builder_fee(&self, action: &Value) -> Result<Signature, HyperliquidError> {
        self.sign_user_signed_action(
            action,
            action_types::APPROVE_BUILDER_FEE,
            "HyperliquidTransaction:ApproveBuilderFee"
        )
    }

    /// Sign a user DEX abstraction action
    pub fn sign_user_dex_abstraction(&self, action: &Value) -> Result<Signature, HyperliquidError> {
        self.sign_user_signed_action(
            action,
            action_types::USER_DEX_ABSTRACTION,
            "HyperliquidTransaction:UserDexAbstraction"
        )
    }

    /// Sign a convert to multi-sig user action
    pub fn sign_convert_to_multi_sig_user(&self, action: &Value) -> Result<Signature, HyperliquidError> {
        self.sign_user_signed_action(
            action,
            action_types::CONVERT_TO_MULTI_SIG_USER,
            "HyperliquidTransaction:ConvertToMultiSigUser"
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_private_key_creation() {
        let key_hex = "0x1111111111111111111111111111111111111111111111111111111111111111";
        let private_key = PrivateKey::from_hex(key_hex).unwrap();

        assert_eq!(private_key.to_hex(), key_hex);
    }

    #[test]
    fn test_private_key_invalid_hex() {
        let result = PrivateKey::from_hex("invalid_hex");
        assert!(result.is_err());
    }

    #[test]
    fn test_private_key_wrong_length() {
        let short_key = "0x1111";
        let result = PrivateKey::from_hex(short_key);
        assert!(result.is_err());
    }

    #[test]
    fn test_private_key_from_str() {
        let key_hex = "0x1111111111111111111111111111111111111111111111111111111111111111";
        let private_key: PrivateKey = key_hex.parse().unwrap();

        assert_eq!(private_key.to_hex(), key_hex);
    }

    #[test]
    fn test_private_key_generate() {
        let private_key = PrivateKey::generate().unwrap();

        // Verify the key was generated
        assert!(!private_key.to_hex().is_empty());
        assert!(private_key.to_hex().starts_with("0x"));
        assert_eq!(private_key.to_hex().len(), 66); // 0x + 64 hex chars

        // Verify address can be derived
        let address = private_key.address();
        assert!(address.starts_with("0x"));
        assert_eq!(address.len(), 42); // 0x + 40 hex chars
    }

    #[test]
    fn test_private_key_generate_unique() {
        let key1 = PrivateKey::generate().unwrap();
        let key2 = PrivateKey::generate().unwrap();

        // Keys should be different (extremely unlikely to be the same)
        assert_ne!(key1.to_hex(), key2.to_hex());
    }

    #[test]
    fn test_wallet_creation() {
        let key_hex = "0x1111111111111111111111111111111111111111111111111111111111111111";

        let wallet_mainnet = Wallet::mainnet(key_hex).unwrap();
        assert!(wallet_mainnet.is_mainnet());
        assert_eq!(wallet_mainnet.private_key().to_hex(), key_hex);

        let wallet_testnet = Wallet::testnet(key_hex).unwrap();
        assert!(!wallet_testnet.is_mainnet());
        assert_eq!(wallet_testnet.private_key().to_hex(), key_hex);
    }

    #[test]
    fn test_wallet_address() {
        let key_hex = "0x1111111111111111111111111111111111111111111111111111111111111111";
        let wallet = Wallet::mainnet(key_hex).unwrap();

        let address = wallet.address();
        assert!(address.starts_with("0x"));
        assert_eq!(address.len(), 42); // 0x + 40 hex chars
    }

    #[test]
    fn test_sign_usd_transfer() {
        let key_hex = "0x1111111111111111111111111111111111111111111111111111111111111111";
        let wallet = Wallet::mainnet(key_hex).unwrap();

        let action = json!({
            "destination": "0x742d35bE6C8C2c3c2c2c2c2c2c2c2c2c2c2c2c2c",
            "amount": "100.0",
            "time": 1234567890
        });

        let signature = wallet.sign_usd_transfer(&action).unwrap();

        // Verify signature format
        assert!(signature.v == 27 || signature.v == 28);
        assert!(signature.r.starts_with("0x"));
        assert!(signature.s.starts_with("0x"));
        assert_eq!(signature.r.len(), 66);
        assert_eq!(signature.s.len(), 66);
    }

    #[test]
    fn test_sign_l1_action() {
        let key_hex = "0x1111111111111111111111111111111111111111111111111111111111111111";
        let wallet = Wallet::mainnet(key_hex).unwrap();

        let action = json!({
            "type": "order",
            "orders": [{
                "a": 1,
                "b": true,
                "p": "50000.0",
                "s": "0.1",
                "r": false,
                "t": {"limit": {"tif": "Gtc"}}
            }]
        });

        let signature = wallet.sign_l1_action(&action, None, 12345678, None).unwrap();

        // Verify signature format
        assert!(signature.v == 27 || signature.v == 28);
        assert!(signature.r.starts_with("0x"));
        assert!(signature.s.starts_with("0x"));
        assert_eq!(signature.r.len(), 66);
        assert_eq!(signature.s.len(), 66);
    }

    #[test]
    fn test_sign_l1_action_with_auto_nonce() {
        let key_hex = "0x1111111111111111111111111111111111111111111111111111111111111111";
        let wallet = Wallet::mainnet(key_hex).unwrap();

        let action = json!({
            "type": "order",
            "orders": [{
                "a": 1,
                "b": true,
                "p": "50000.0",
                "s": "0.1",
                "r": false,
                "t": {"limit": {"tif": "Gtc"}}
            }]
        });

        let signature = wallet.sign_l1_action_with_nonce(&action, None, None).unwrap();

        // Verify signature format
        assert!(signature.v == 27 || signature.v == 28);
        assert!(signature.r.starts_with("0x"));
        assert!(signature.s.starts_with("0x"));
        assert_eq!(signature.r.len(), 66);
        assert_eq!(signature.s.len(), 66);
    }

    #[test]
    fn test_sign_l1_action_with_timestamp_nonce() {
        let key_hex = "0x1111111111111111111111111111111111111111111111111111111111111111";
        let wallet = Wallet::mainnet(key_hex).unwrap();

        let action = json!({
            "type": "order",
            "orders": [{
                "a": 1,
                "b": true,
                "p": "50000.0",
                "s": "0.1",
                "r": false,
                "t": {"limit": {"tif": "Gtc"}}
            }]
        });

        let signature = wallet.sign_l1_action_with_timestamp_nonce(&action, None, None).unwrap();

        // Verify signature format
        assert!(signature.v == 27 || signature.v == 28);
        assert!(signature.r.starts_with("0x"));
        assert!(signature.s.starts_with("0x"));
        assert_eq!(signature.r.len(), 66);
        assert_eq!(signature.s.len(), 66);
    }

    #[test]
    fn test_deterministic_address() {
        let key_hex = "0x1111111111111111111111111111111111111111111111111111111111111111";
        let wallet1 = Wallet::mainnet(key_hex).unwrap();
        let wallet2 = Wallet::testnet(key_hex).unwrap();

        // Same private key should generate same address regardless of network
        assert_eq!(wallet1.address(), wallet2.address());
    }

    #[test]
    fn test_sign_token_delegate() {
        let key_hex = "0x1111111111111111111111111111111111111111111111111111111111111111";
        let wallet = Wallet::mainnet(key_hex).unwrap();

        let action = json!({
            "validator": "0x742d35bE6C8C2c3c2c2c2c2c2c2c2c2c2c2c2c2c",
            "wei": "1000000000000000000",
            "isUndelegate": false,
            "time": 1234567890
        });

        let signature = wallet.sign_token_delegate(&action).unwrap();

        // Verify signature format
        assert!(signature.v == 27 || signature.v == 28);
        assert!(signature.r.starts_with("0x"));
        assert!(signature.s.starts_with("0x"));
        assert_eq!(signature.r.len(), 66);
        assert_eq!(signature.s.len(), 66);
    }

    #[test]
    fn test_sign_send_asset() {
        let key_hex = "0x1111111111111111111111111111111111111111111111111111111111111111";
        let wallet = Wallet::mainnet(key_hex).unwrap();

        let action = json!({
            "destination": "0x742d35bE6C8C2c3c2c2c2c2c2c2c2c2c2c2c2c2c",
            "sourceDex": "hyperliquid",
            "destinationDex": "another_dex",
            "token": "USDC",
            "amount": "100.0",
            "fromSubAccount": "0x1234567890123456789012345678901234567890",
            "nonce": 12345678
        });

        let signature = wallet.sign_send_asset(&action).unwrap();

        // Verify signature format
        assert!(signature.v == 27 || signature.v == 28);
        assert!(signature.r.starts_with("0x"));
        assert!(signature.s.starts_with("0x"));
        assert_eq!(signature.r.len(), 66);
        assert_eq!(signature.s.len(), 66);
    }

    #[test]
    fn test_sign_approve_builder_fee() {
        let key_hex = "0x1111111111111111111111111111111111111111111111111111111111111111";
        let wallet = Wallet::mainnet(key_hex).unwrap();

        let action = json!({
            "maxFeeRate": "100",
            "builder": "0x742d35bE6C8C2c3c2c2c2c2c2c2c2c2c2c2c2c2c",
            "nonce": 12345678
        });

        let signature = wallet.sign_approve_builder_fee(&action).unwrap();

        // Verify signature format
        assert!(signature.v == 27 || signature.v == 28);
        assert!(signature.r.starts_with("0x"));
        assert!(signature.s.starts_with("0x"));
        assert_eq!(signature.r.len(), 66);
        assert_eq!(signature.s.len(), 66);
    }

    #[test]
    fn test_sign_user_dex_abstraction() {
        let key_hex = "0x1111111111111111111111111111111111111111111111111111111111111111";
        let wallet = Wallet::mainnet(key_hex).unwrap();

        let action = json!({
            "user": "0x742d35bE6C8C2c3c2c2c2c2c2c2c2c2c2c2c2c2c",
            "enabled": true,
            "nonce": 12345678
        });

        let signature = wallet.sign_user_dex_abstraction(&action).unwrap();

        // Verify signature format
        assert!(signature.v == 27 || signature.v == 28);
        assert!(signature.r.starts_with("0x"));
        assert!(signature.s.starts_with("0x"));
        assert_eq!(signature.r.len(), 66);
        assert_eq!(signature.s.len(), 66);
    }

    #[test]
    fn test_sign_convert_to_multi_sig_user() {
        let key_hex = "0x1111111111111111111111111111111111111111111111111111111111111111";
        let wallet = Wallet::mainnet(key_hex).unwrap();

        let action = json!({
            "authorizedUsers": ["0x742d35bE6C8C2c3c2c2c2c2c2c2c2c2c2c2c2c2c", "0x9876543210987654321098765432109876543210"],
            "threshold": 1,
            "time": 1234567890
        });

        let signature = wallet.sign_convert_to_multi_sig_user(&action).unwrap();

        // Verify signature format
        assert!(signature.v == 27 || signature.v == 28);
        assert!(signature.r.starts_with("0x"));
        assert!(signature.s.starts_with("0x"));
        assert_eq!(signature.r.len(), 66);
        assert_eq!(signature.s.len(), 66);
    }

    #[test]
    fn test_wallet_generate_mainnet() {
        let wallet = Wallet::generate_mainnet().unwrap();

        // Verify it's a mainnet wallet
        assert!(wallet.is_mainnet());

        // Verify address was generated
        let address = wallet.address();
        assert!(address.starts_with("0x"));
        assert_eq!(address.len(), 42);

        // Verify private key exists
        let private_key_hex = wallet.private_key().to_hex();
        assert!(private_key_hex.starts_with("0x"));
        assert_eq!(private_key_hex.len(), 66);
    }

    #[test]
    fn test_wallet_generate_testnet() {
        let wallet = Wallet::generate_testnet().unwrap();

        // Verify it's a testnet wallet
        assert!(!wallet.is_mainnet());

        // Verify address was generated
        let address = wallet.address();
        assert!(address.starts_with("0x"));
        assert_eq!(address.len(), 42);

        // Verify private key exists
        let private_key_hex = wallet.private_key().to_hex();
        assert!(private_key_hex.starts_with("0x"));
        assert_eq!(private_key_hex.len(), 66);
    }

    #[test]
    fn test_wallet_generate_unique() {
        let wallet1 = Wallet::generate_mainnet().unwrap();
        let wallet2 = Wallet::generate_mainnet().unwrap();

        // Wallets should be different (extremely unlikely to be the same)
        assert_ne!(wallet1.address(), wallet2.address());
        assert_ne!(wallet1.private_key().to_hex(), wallet2.private_key().to_hex());
    }

    #[test]
    fn test_wallet_generate_can_sign() {
        let wallet = Wallet::generate_mainnet().unwrap();

        // Test that the generated wallet can sign transactions
        let action = json!({
            "type": "order",
            "orders": [{
                "a": 1,
                "b": true,
                "p": "50000.0",
                "s": "0.1",
                "r": false,
                "t": {"limit": {"tif": "Gtc"}}
            }]
        });

        let signature = wallet.sign_l1_action(&action, None, 12345678, None).unwrap();

        // Verify signature format
        assert!(signature.v == 27 || signature.v == 28);
        assert!(signature.r.starts_with("0x"));
        assert!(signature.s.starts_with("0x"));
        assert_eq!(signature.r.len(), 66);
        assert_eq!(signature.s.len(), 66);
    }
}