//! Types for cryptographic operations

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// EIP-712 domain separator for Hyperliquid
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EIP712Domain {
    pub name: String,
    pub version: String,
    pub chain_id: String,
    pub verifying_contract: String,
}

impl EIP712Domain {
    /// Create domain for Hyperliquid mainnet
    pub fn hyperliquid_mainnet() -> Self {
        Self {
            name: "HyperliquidSignTransaction".to_string(),
            version: "1".to_string(),
            chain_id: "0x66eee".to_string(),  // 423664
            verifying_contract: "0x0000000000000000000000000000000000000000".to_string(),
        }
    }

    /// Create domain for Hyperliquid testnet
    pub fn hyperliquid_testnet() -> Self {
        Self {
            name: "HyperliquidSignTransaction".to_string(),
            version: "1".to_string(),
            chain_id: "0x66eee".to_string(),  // Same chain ID for testnet
            verifying_contract: "0x0000000000000000000000000000000000000000".to_string(),
        }
    }

    /// Create domain for L1 actions (Agent signing)
    pub fn l1_agent() -> Self {
        Self {
            name: "Exchange".to_string(),
            version: "1".to_string(),
            chain_id: "1337".to_string(),
            verifying_contract: "0x0000000000000000000000000000000000000000".to_string(),
        }
    }
}

/// EIP-712 type definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EIP712Type {
    pub name: String,
    pub type_: String,
}

/// Phantom agent for L1 signing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhantomAgent {
    pub source: String,  // "a" for mainnet, "b" for testnet
    #[serde(rename = "connectionId")]
    pub connection_id: String,  // 32-byte hash as hex string
}

impl PhantomAgent {
    /// Create phantom agent for mainnet
    pub fn mainnet(hash: impl Into<String>) -> Self {
        Self {
            source: "a".to_string(),
            connection_id: hash.into(),
        }
    }

    /// Create phantom agent for testnet
    pub fn testnet(hash: impl Into<String>) -> Self {
        Self {
            source: "b".to_string(),
            connection_id: hash.into(),
        }
    }
}

/// EIP-712 message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EIP712Message {
    pub domain: EIP712Domain,
    #[serde(rename = "types")]
    pub message_types: HashMap<String, Vec<EIP712Type>>,
    #[serde(rename = "primaryType")]
    pub primary_type: String,
    pub message: serde_json::Value,
}

/// Ethereum signature components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signature {
    pub r: String,
    pub s: String,
    pub v: u8,
}

impl Signature {
    /// Create new signature from components
    pub fn new(r: impl Into<String>, s: impl Into<String>, v: u8) -> Self {
        Self {
            r: r.into(),
            s: s.into(),
            v,
        }
    }

    /// Convert to hex string format used by API
    pub fn to_hex(&self) -> String {
        format!("0x{}{}{:02x}", &self.r[2..], &self.s[2..], self.v)
    }
}

/// Action type definitions for different transaction types
pub mod action_types {
    use super::*;

    pub const USD_SEND: &[EIP712Type] = &[
        EIP712Type { name: "hyperliquidChain".to_string(), type_: "string".to_string() },
        EIP712Type { name: "destination".to_string(), type_: "string".to_string() },
        EIP712Type { name: "amount".to_string(), type_: "string".to_string() },
        EIP712Type { name: "time".to_string(), type_: "uint64".to_string() },
    ];

    pub const SPOT_TRANSFER: &[EIP712Type] = &[
        EIP712Type { name: "hyperliquidChain".to_string(), type_: "string".to_string() },
        EIP712Type { name: "destination".to_string(), type_: "string".to_string() },
        EIP712Type { name: "token".to_string(), type_: "string".to_string() },
        EIP712Type { name: "amount".to_string(), type_: "string".to_string() },
        EIP712Type { name: "time".to_string(), type_: "uint64".to_string() },
    ];

    pub const WITHDRAW: &[EIP712Type] = &[
        EIP712Type { name: "hyperliquidChain".to_string(), type_: "string".to_string() },
        EIP712Type { name: "destination".to_string(), type_: "string".to_string() },
        EIP712Type { name: "amount".to_string(), type_: "string".to_string() },
        EIP712Type { name: "time".to_string(), type_: "uint64".to_string() },
    ];

    pub const USDC_CLASS_TRANSFER: &[EIP712Type] = &[
        EIP712Type { name: "hyperliquidChain".to_string(), type_: "string".to_string() },
        EIP712Type { name: "amount".to_string(), type_: "string".to_string() },
        EIP712Type { name: "toPerp".to_string(), type_: "bool".to_string() },
        EIP712Type { name: "nonce".to_string(), type_: "uint64".to_string() },
    ];

    pub const AGENT: &[EIP712Type] = &[
        EIP712Type { name: "source".to_string(), type_: "string".to_string() },
        EIP712Type { name: "connectionId".to_string(), type_: "bytes32".to_string() },
    ];

    pub const EIP712_DOMAIN: &[EIP712Type] = &[
        EIP712Type { name: "name".to_string(), type_: "string".to_string() },
        EIP712Type { name: "version".to_string(), type_: "string".to_string() },
        EIP712Type { name: "chainId".to_string(), type_: "uint256".to_string() },
        EIP712Type { name: "verifyingContract".to_string(), type_: "address".to_string() },
    ];
}

/// Multi-sig envelope structure for multi-signature transactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiSigEnvelope {
    /// The inner action to be executed
    pub inner: serde_json::Value,
    /// Multi-sig user address
    pub multi_sig_user: String,
    /// Array of signatures from authorized signers
    pub signatures: Vec<String>,
    /// Nonce for the envelope
    pub nonce: u64,
    /// Optional vault address
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vault_address: Option<String>,
}

/// Multi-sig user configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiSigUser {
    /// Multi-sig user address
    pub address: String,
    /// List of authorized signer addresses
    pub authorized_signers: Vec<String>,
    /// Threshold of signatures required
    pub threshold: u32,
}

/// Signature for multi-sig operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiSigSignature {
    /// Signer address
    pub signer: String,
    /// Signature components
    pub signature: Signature,
    /// Timestamp of when signature was created
    pub timestamp: u64,
}

impl MultiSigEnvelope {
    /// Create a new multi-sig envelope
    pub fn new(
        inner: serde_json::Value,
        multi_sig_user: impl Into<String>,
        nonce: u64,
        vault_address: Option<impl Into<String>>,
    ) -> Self {
        Self {
            inner,
            multi_sig_user: multi_sig_user.into(),
            signatures: Vec::new(),
            nonce,
            vault_address: vault_address.map(|v| v.into()),
        }
    }

    /// Add a signature to the envelope
    pub fn add_signature(&mut self, signature: impl Into<String>) {
        self.signatures.push(signature.into());
    }

    /// Get the number of signatures collected
    pub fn signature_count(&self) -> usize {
        self.signatures.len()
    }

    /// Check if enough signatures have been collected
    pub fn has_sufficient_signatures(&self, threshold: u32) -> bool {
        self.signature_count() >= threshold as usize
    }
}

/// Action type definitions for different transaction types
pub mod action_types {
    use super::*;

    pub const USD_SEND: &[EIP712Type] = &[
        EIP712Type { name: "hyperliquidChain".to_string(), type_: "string".to_string() },
        EIP712Type { name: "destination".to_string(), type_: "string".to_string() },
        EIP712Type { name: "amount".to_string(), type_: "string".to_string() },
        EIP712Type { name: "time".to_string(), type_: "uint64".to_string() },
    ];

    pub const SPOT_TRANSFER: &[EIP712Type] = &[
        EIP712Type { name: "hyperliquidChain".to_string(), type_: "string".to_string() },
        EIP712Type { name: "destination".to_string(), type_: "string".to_string() },
        EIP712Type { name: "token".to_string(), type_: "string".to_string() },
        EIP712Type { name: "amount".to_string(), type_: "string".to_string() },
        EIP712Type { name: "time".to_string(), type_: "uint64".to_string() },
    ];

    pub const WITHDRAW: &[EIP712Type] = &[
        EIP712Type { name: "hyperliquidChain".to_string(), type_: "string".to_string() },
        EIP712Type { name: "destination".to_string(), type_: "string".to_string() },
        EIP712Type { name: "amount".to_string(), type_: "string".to_string() },
        EIP712Type { name: "time".to_string(), type_: "uint64".to_string() },
    ];

    pub const USDC_CLASS_TRANSFER: &[EIP712Type] = &[
        EIP712Type { name: "hyperliquidChain".to_string(), type_: "string".to_string() },
        EIP712Type { name: "amount".to_string(), type_: "string".to_string() },
        EIP712Type { name: "toPerp".to_string(), type_: "bool".to_string() },
        EIP712Type { name: "nonce".to_string(), type_: "uint64".to_string() },
    ];

    pub const TOKEN_DELEGATE: &[EIP712Type] = &[
        EIP712Type { name: "hyperliquidChain".to_string(), type_: "string".to_string() },
        EIP712Type { name: "validator".to_string(), type_: "string".to_string() },
        EIP712Type { name: "wei".to_string(), type_: "string".to_string() },
        EIP712Type { name: "isUndelegate".to_string(), type_: "bool".to_string() },
        EIP712Type { name: "time".to_string(), type_: "uint64".to_string() },
    ];

    pub const CONVERT_TO_MULTI_SIG_USER: &[EIP712Type] = &[
        EIP712Type { name: "hyperliquidChain".to_string(), type_: "string".to_string() },
        EIP712Type { name: "authorizedUsers".to_string(), type_: "string[]".to_string() },
        EIP712Type { name: "threshold".to_string(), type_: "uint256".to_string() },
        EIP712Type { name: "time".to_string(), type_: "uint64".to_string() },
    ];

    pub const MULTI_SIG_ENVELOPE: &[EIP712Type] = &[
        EIP712Type { name: "hyperliquidChain".to_string(), type_: "string".to_string() },
        EIP712Type { name: "inner".to_string(), type_: "bytes".to_string() },
        EIP712Type { name: "multiSigUser".to_string(), type_: "string".to_string() },
        EIP712Type { name: "signatures".to_string(), type_: "string[]".to_string() },
        EIP712Type { name: "nonce".to_string(), type_: "uint64".to_string() },
        EIP712Type { name: "vaultAddress".to_string(), type_: "string".to_string() },
    ];

    pub const AGENT: &[EIP712Type] = &[
        EIP712Type { name: "source".to_string(), type_: "string".to_string() },
        EIP712Type { name: "connectionId".to_string(), type_: "bytes32".to_string() },
    ];

    pub const SEND_ASSET: &[EIP712Type] = &[
        EIP712Type { name: "hyperliquidChain".to_string(), type_: "string".to_string() },
        EIP712Type { name: "destination".to_string(), type_: "string".to_string() },
        EIP712Type { name: "sourceDex".to_string(), type_: "string".to_string() },
        EIP712Type { name: "destinationDex".to_string(), type_: "string".to_string() },
        EIP712Type { name: "token".to_string(), type_: "string".to_string() },
        EIP712Type { name: "amount".to_string(), type_: "string".to_string() },
        EIP712Type { name: "fromSubAccount".to_string(), type_: "string".to_string() },
        EIP712Type { name: "nonce".to_string(), type_: "uint64".to_string() },
    ];

    pub const USER_DEX_ABSTRACTION: &[EIP712Type] = &[
        EIP712Type { name: "hyperliquidChain".to_string(), type_: "string".to_string() },
        EIP712Type { name: "user".to_string(), type_: "string".to_string() },
        EIP712Type { name: "enabled".to_string(), type_: "bool".to_string() },
        EIP712Type { name: "nonce".to_string(), type_: "uint64".to_string() },
    ];

    pub const APPROVE_BUILDER_FEE: &[EIP712Type] = &[
        EIP712Type { name: "hyperliquidChain".to_string(), type_: "string".to_string() },
        EIP712Type { name: "maxFeeRate".to_string(), type_: "string".to_string() },
        EIP712Type { name: "builder".to_string(), type_: "string".to_string() },
        EIP712Type { name: "nonce".to_string(), type_: "uint64".to_string() },
    ];

    pub const EIP712_DOMAIN: &[EIP712Type] = &[
        EIP712Type { name: "name".to_string(), type_: "string".to_string() },
        EIP712Type { name: "version".to_string(), type_: "string".to_string() },
        EIP712Type { name: "chainId".to_string(), type_: "uint256".to_string() },
        EIP712Type { name: "verifyingContract".to_string(), type_: "address".to_string() },
    ];
}

/// Environment specification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Environment {
    Mainnet,
    Testnet,
}

impl Environment {
    /// Get the source character for phantom agent
    pub fn source_char(self) -> &'static str {
        match self {
            Environment::Mainnet => "a",
            Environment::Testnet => "b",
        }
    }

    /// Get the chain ID string
    pub fn chain_id(self) -> &'static str {
        "0x66eee"  // Both mainnet and testnet use same chain ID
    }

    /// Get the chain name
    pub fn chain_name(self) -> &'static str {
        match self {
            Environment::Mainnet => "Mainnet",
            Environment::Testnet => "Testnet",
        }
    }
}