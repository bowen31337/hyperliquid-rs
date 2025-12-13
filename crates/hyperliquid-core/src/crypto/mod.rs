//! Cryptographic utilities for Hyperliquid SDK
//!
//! This module provides EIP-712 signing functionality, message hashing,
//! and wallet operations for interacting with the Hyperliquid exchange.

pub mod signing;
pub mod wallet;
pub mod types;
pub mod nonce;

pub use signing::{sign_l1_action, sign_user_signed_action, action_hash, PhantomAgent, recover_address, verify_signature};
pub use wallet::{Wallet, PrivateKey};
pub use types::{
    EIP712Domain, EIP712Type, PhantomAgent, EIP712Message, Signature, Environment,
    action_types, MultiSigEnvelope, MultiSigUser, MultiSigSignature,
};
pub use nonce::{generate_nonce, generate_timestamp_nonce, NonceGenerator, PrivateKeySecure};