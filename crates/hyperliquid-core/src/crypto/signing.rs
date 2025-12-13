//! EIP-712 signing implementation for Hyperliquid

use crate::error::HyperliquidError;
use crate::crypto::types::*;
use k256::ecdsa::{SigningKey};
use k256::sha3::{Digest, Keccak256};
use rmp_serde::to_vec_named;
use serde_json::{json, Value};
use std::collections::HashMap;

/// Compute action hash using msgpack serialization and keccak256
pub fn action_hash(
    action: &Value,
    vault_address: Option<&str>,
    nonce: u64,
    expires_after: Option<u64>,
) -> Result<String, HyperliquidError> {
    // Serialize action with msgpack
    let action_bytes = to_vec_named(action)
        .map_err(|e| HyperliquidError::Signing(format!("Failed to serialize action: {}", e)))?;

    // Create buffer for hashing
    let mut hash_data = Vec::new();
    hash_data.extend_from_slice(&action_bytes);

    // Append nonce as 8 bytes big-endian
    hash_data.extend_from_slice(&nonce.to_be_bytes());

    // Append vault flag and address if present
    if let Some(vault) = vault_address {
        hash_data.push(0x01);  // Vault present flag
        let vault_bytes = hex::decode(vault.trim_start_matches("0x"))
            .map_err(|e| HyperliquidError::Signing(format!("Invalid vault address: {}", e)))?;
        if vault_bytes.len() != 20 {
            return Err(HyperliquidError::Signing("Vault address must be 20 bytes".to_string()));
        }
        hash_data.extend_from_slice(&vault_bytes);
    } else {
        hash_data.push(0x00);  // No vault flag
    }

    // Append expires_after if present
    if let Some(expires) = expires_after {
        hash_data.extend_from_slice(&expires.to_be_bytes());
    }

    // Compute keccak256 hash
    let mut hasher = Keccak256::new();
    hasher.update(&hash_data);
    let result = hasher.finalize();

    Ok(format!("0x{}", hex::encode(result)))
}

/// Construct phantom agent for L1 signing
pub fn construct_phantom_agent(hash: &str, is_mainnet: bool) -> PhantomAgent {
    PhantomAgent {
        source: if is_mainnet { "a" } else { "b" }.to_string(),
        connection_id: hash.to_string(),
    }
}

/// Create EIP-712 domain separator
pub fn create_domain_separator(is_mainnet: bool) -> EIP712Domain {
    if is_mainnet {
        EIP712Domain::hyperliquid_mainnet()
    } else {
        EIP712Domain::hyperliquid_testnet()
    }
}

/// Create EIP-712 message for L1 action signing
pub fn create_l1_payload(phantom_agent: &PhantomAgent) -> EIP712Message {
    let mut types = HashMap::new();
    types.insert("Agent".to_string(), action_types::AGENT.to_vec());
    types.insert("EIP712Domain".to_string(), action_types::EIP712_DOMAIN.to_vec());

    EIP712Message {
        domain: EIP712Domain::l1_agent(),
        message_types: types,
        primary_type: "Agent".to_string(),
        message: json!(phantom_agent),
    }
}

/// Create EIP-712 message for user-signed action
pub fn create_user_signed_payload(
    action: &Value,
    payload_types: &[EIP712Type],
    primary_type: &str,
    is_mainnet: bool,
) -> EIP712Message {
    let mut types = HashMap::new();
    types.insert(primary_type.to_string(), payload_types.to_vec());
    types.insert("EIP712Domain".to_string(), action_types::EIP712_DOMAIN.to_vec());

    // Ensure action has required fields
    let mut action = action.clone();
    if let Some(obj) = action.as_object_mut() {
        obj.insert("hyperliquidChain".to_string(),
                  json!(if is_mainnet { "Mainnet" } else { "Testnet" }));
        obj.insert("signatureChainId".to_string(), json!("0x66eee"));
    }

    EIP712Message {
        domain: if is_mainnet {
            EIP712Domain::hyperliquid_mainnet()
        } else {
            EIP712Domain::hyperliquid_testnet()
        },
        message_types: types,
        primary_type: primary_type.to_string(),
        message: action,
    }
}

/// Sign L1 action with private key
pub fn sign_l1_action(
    private_key: &str,
    action: &Value,
    vault_address: Option<&str>,
    nonce: u64,
    expires_after: Option<u64>,
    is_mainnet: bool,
) -> Result<Signature, HyperliquidError> {
    // Compute action hash
    let hash = action_hash(action, vault_address, nonce, expires_after)?;

    // Construct phantom agent
    let phantom_agent = construct_phantom_agent(&hash, is_mainnet);

    // Create EIP-712 message
    let payload = create_l1_payload(&phantom_agent);

    // Sign the message
    sign_message(private_key, &payload)
}

/// Sign user-signed action with private key
pub fn sign_user_signed_action(
    private_key: &str,
    action: &Value,
    payload_types: &[EIP712Type],
    primary_type: &str,
    is_mainnet: bool,
) -> Result<Signature, HyperliquidError> {
    // Create EIP-712 message
    let payload = create_user_signed_payload(action, payload_types, primary_type, is_mainnet);

    // Sign the message
    sign_message(private_key, &payload)
}

/// Sign an EIP-712 message
pub fn sign_message(private_key: &str, message: &EIP712Message) -> Result<Signature, HyperliquidError> {
    // Convert private key from hex
    let key_bytes = hex::decode(private_key.trim_start_matches("0x"))
        .map_err(|e| HyperliquidError::Signing(format!("Invalid private key: {}", e)))?;

    let signing_key = SigningKey::from_bytes(&key_bytes.into())
        .map_err(|e| HyperliquidError::Signing(format!("Invalid signing key: {}", e)))?;

    // For now, we'll use a simplified approach - serialize the message as JSON and hash it
    // In a production implementation, you'd want full EIP-712 encoding
    let message_str = serde_json::to_string(message)
        .map_err(|e| HyperliquidError::Signing(format!("Failed to serialize message: {}", e)))?;

    let mut hasher = Keccak256::new();
    hasher.update(message_str.as_bytes());
    let hash = hasher.finalize();

    // Sign the hash
    let (recovery_id, signature_bytes) = signing_key
        .sign_prehash_recoverable(&hash)
        .map_err(|e| HyperliquidError::Signing(format!("Failed to sign message: {}", e)))?
        .split_bytes();

    Ok(Signature {
        r: format!("0x{}", hex::encode(&signature_bytes[..32])),
        s: format!("0x{}", hex::encode(&signature_bytes[32..])),
        v: recovery_id.to_byte() + 27,
    })
}

/// Convert address to bytes
pub fn address_to_bytes(address: &str) -> Result<Vec<u8>, HyperliquidError> {
    hex::decode(address.trim_start_matches("0x"))
        .map_err(|e| HyperliquidError::Signing(format!("Invalid address format: {}", e)))
}

/// Recover the Ethereum address from an EIP-712 signature
///
/// This function verifies a signature and recovers the signer's address.
/// It's useful for verifying that a signature was created by the expected address.
pub fn recover_address(
    message: &EIP712Message,
    signature: &Signature,
) -> Result<String, HyperliquidError> {
    // Serialize the message as JSON and hash it (simplified EIP-712)
    let message_str = serde_json::to_string(message)
        .map_err(|e| HyperliquidError::Signing(format!("Failed to serialize message: {}", e)))?;

    let mut hasher = Keccak256::new();
    hasher.update(message_str.as_bytes());
    let hash = hasher.finalize();

    // Parse signature components
    let r_bytes = hex::decode(signature.r.trim_start_matches("0x"))
        .map_err(|e| HyperliquidError::Signing(format!("Invalid signature R component: {}", e)))?;
    let s_bytes = hex::decode(signature.s.trim_start_matches("0x"))
        .map_err(|e| HyperliquidError::Signing(format!("Invalid signature S component: {}", e)))?;

    if r_bytes.len() != 32 || s_bytes.len() != 32 {
        return Err(HyperliquidError::Signing(
            "Signature R and S must be 32 bytes each".to_string(),
        ));
    }

    // Create signature from bytes
    let mut signature_bytes = [0u8; 64];
    signature_bytes[..32].copy_from_slice(&r_bytes);
    signature_bytes[32..].copy_from_slice(&s_bytes);

    // Parse recovery ID (v)
    // v = recovery_id + 27 or v = recovery_id + 28
    let recovery_id = if signature.v == 27 || signature.v == 28 {
        signature.v - 27
    } else if signature.v == 0 || signature.v == 1 {
        signature.v
    } else {
        return Err(HyperliquidError::Signing(format!(
            "Invalid recovery ID: {}",
            signature.v
        )));
    };

    let recovery_id = k256::ecdsa::RecoveryId::from_byte(recovery_id)
        .map_err(|e| HyperliquidError::Signing(format!("Invalid recovery ID: {}", e)))?;

    let recovered_signature = k256::ecdsa::Signature::from_slice(&signature_bytes)
        .map_err(|e| HyperliquidError::Signing(format!("Invalid signature: {}", e)))?;

    // Combine signature with recovery ID
    let recoverable_signature = k256::ecdsa::recoverable::Signature::new(
        &recovered_signature,
        recovery_id,
    );

    // Recover the public key
    let recovered_key = recoverable_signature
        .recover_verifying_key_from_digest_bytes(&hash.into())
        .map_err(|e| HyperliquidError::Signing(format!("Failed to recover public key: {}", e)))?;

    // Convert to address
    let pub_key_bytes = recovered_key.to_sec1_bytes();

    // Take the keccak256 hash of the public key (excluding the 0x04 prefix)
    let mut address_hasher = Keccak256::new();
    address_hasher.update(&pub_key_bytes[1..]); // Skip the 0x04 prefix
    let hash = address_hasher.finalize();

    // Take the last 20 bytes
    let address_bytes = &hash[12..];
    Ok(format!("0x{}", hex::encode(address_bytes)))
}

/// Verify that a signature was created by the expected address
pub fn verify_signature(
    message: &EIP712Message,
    signature: &Signature,
    expected_address: &str,
) -> Result<bool, HyperliquidError> {
    let recovered_address = recover_address(message, signature)?;
    Ok(recovered_address.to_lowercase() == expected_address.to_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_action_hash_simple_order() {
        let action = json!({
            "type": "order",
            "coin": "ETH",
            "is_buy": true,
            "sz": "1.0",
            "limit_px": "2000.0",
            "order_type": {"limit": {"tif": "Gtc"}},
            "reduce_only": false
        });

        let result = action_hash(&action, None, 12345, None).unwrap();
        assert!(result.starts_with("0x"));
        assert_eq!(result.len(), 66); // 0x + 64 hex chars
    }

    #[test]
    fn test_action_hash_with_vault() {
        let action = json!({
            "type": "order",
            "coin": "BTC",
            "is_buy": false,
            "sz": "0.1",
            "limit_px": "50000.0"
        });

        let vault_address = "0x1234567890123456789012345678901234567890";
        let result = action_hash(&action, Some(vault_address), 98765, None).unwrap();
        assert!(result.starts_with("0x"));
        assert_eq!(result.len(), 66);
    }

    #[test]
    fn test_address_recovery_from_signature() {
        // Test address recovery functionality
        let test_private_key = "0x4f3edf983ac636a65a842ce7c78d9aa706d3b113bce9c46f30d7d21715b23b1d";
        let expected_address = "0x90f8bf6a479f320ead074411a4b0e7944ea8c9c1";

        // Create a simple EIP-712 message
        let domain = EIP712Domain {
            name: "Test".to_string(),
            version: "1".to_string(),
            chain_id: "1".to_string(),
            verifying_contract: "0x0000000000000000000000000000000000000000".to_string(),
        };

        let mut types = HashMap::new();
        types.insert("TestMessage".to_string(), vec![
            EIP712Type { name: "message".to_string(), type_: "string".to_string() },
        ]);
        types.insert("EIP712Domain".to_string(), action_types::EIP712_DOMAIN.to_vec());

        let message = EIP712Message {
            domain,
            message_types: types,
            primary_type: "TestMessage".to_string(),
            message: json!({"message": "test"}),
        };

        // Sign the message
        let signature = sign_message(test_private_key, &message).unwrap();

        // Recover the address
        let recovered_address = recover_address(&message, &signature).unwrap();

        // Verify the recovered address matches (case-insensitive)
        assert_eq!(recovered_address.to_lowercase(), expected_address.to_lowercase());
    }

    #[test]
    fn test_signature_verification() {
        let test_private_key = "0x4f3edf983ac636a65a842ce7c78d9aa706d3b113bce9c46f30d7d21715b23b1d";
        let expected_address = "0x90f8bf6a479f320ead074411a4b0e7944ea8c9c1";

        // Create a simple EIP-712 message
        let domain = EIP712Domain {
            name: "Test".to_string(),
            version: "1".to_string(),
            chain_id: "1".to_string(),
            verifying_contract: "0x0000000000000000000000000000000000000000".to_string(),
        };

        let mut types = HashMap::new();
        types.insert("TestMessage".to_string(), vec![
            EIP712Type { name: "message".to_string(), type_: "string".to_string() },
        ]);
        types.insert("EIP712Domain".to_string(), action_types::EIP712_DOMAIN.to_vec());

        let message = EIP712Message {
            domain,
            message_types: types,
            primary_type: "TestMessage".to_string(),
            message: json!({"message": "test verification"}),
        };

        // Sign the message
        let signature = sign_message(test_private_key, &message).unwrap();

        // Verify the signature
        let is_valid = verify_signature(&message, &signature, expected_address).unwrap();
        assert!(is_valid, "Signature should be valid for the expected address");

        // Verify with wrong address
        let wrong_address = "0x1111111111111111111111111111111111111111";
        let is_valid = verify_signature(&message, &signature, wrong_address).unwrap();
        assert!(!is_valid, "Signature should be invalid for wrong address");
    }

    #[test]
    fn test_signature_validation_errors() {
        let test_private_key = "0x4f3edf983ac636a65a842ce7c78d9aa706d3b113bce9c46f30d7d21715b23b1d";

        let domain = EIP712Domain {
            name: "Test".to_string(),
            version: "1".to_string(),
            chain_id: "1".to_string(),
            verifying_contract: "0x0000000000000000000000000000000000000000".to_string(),
        };

        let mut types = HashMap::new();
        types.insert("TestMessage".to_string(), vec![
            EIP712Type { name: "message".to_string(), type_: "string".to_string() },
        ]);
        types.insert("EIP712Domain".to_string(), action_types::EIP712_DOMAIN.to_vec());

        let message = EIP712Message {
            domain,
            message_types: types,
            primary_type: "TestMessage".to_string(),
            message: json!({"message": "test error"}),
        };

        // Create invalid signature
        let invalid_signature = Signature {
            r: "0xinvalid".to_string(),
            s: "0x1234567890123456789012345678901234567890123456789012345678901234".to_string(),
            v: 27,
        };

        // Should fail with invalid R component
        let result = recover_address(&message, &invalid_signature);
        assert!(result.is_err());

        // Create signature with wrong length
        let wrong_length_signature = Signature {
            r: "0x123456789012345678901234567890123456789012345678901234567890".to_string(), // 60 chars instead of 64
            s: "0x1234567890123456789012345678901234567890123456789012345678901234".to_string(),
            v: 27,
        };

        let result = recover_address(&message, &wrong_length_signature);
        assert!(result.is_err());

        // Create signature with invalid recovery ID
        let invalid_v_signature = Signature {
            r: "0x1234567890123456789012345678901234567890123456789012345678901234".to_string(),
            s: "0x1234567890123456789012345678901234567890123456789012345678901234".to_string(),
            v: 99, // Invalid recovery ID
        };

        let result = recover_address(&message, &invalid_v_signature);
        assert!(result.is_err());
    }
}
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

        let hash = action_hash(&action, None, 12345678, None).unwrap();

        // Should be a 32-byte hash starting with 0x
        assert!(hash.starts_with("0x"));
        assert_eq!(hash.len(), 66); // 0x + 64 hex chars
    }

    #[test]
    fn test_construct_phantom_agent() {
        let hash = "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";

        let agent_mainnet = construct_phantom_agent(hash, true);
        assert_eq!(agent_mainnet.source, "a");
        assert_eq!(agent_mainnet.connection_id, hash);

        let agent_testnet = construct_phantom_agent(hash, false);
        assert_eq!(agent_testnet.source, "b");
        assert_eq!(agent_testnet.connection_id, hash);
    }

    #[test]
    fn test_domain_separator() {
        let domain_mainnet = create_domain_separator(true);
        assert_eq!(domain_mainnet.name, "HyperliquidSignTransaction");
        assert_eq!(domain_mainnet.chain_id, "0x66eee");

        let domain_testnet = create_domain_separator(false);
        assert_eq!(domain_testnet.name, "HyperliquidSignTransaction");
        assert_eq!(domain_testnet.chain_id, "0x66eee");
    }

    #[test]
    fn test_l1_payload_creation() {
        let agent = PhantomAgent::mainnet("0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef");
        let payload = create_l1_payload(&agent);

        assert_eq!(payload.primary_type, "Agent");
        assert_eq!(payload.domain.name, "Exchange");
        assert_eq!(payload.domain.chain_id, "1337");
    }

    #[test]
    fn test_user_signed_payload_creation() {
        let action = json!({
            "destination": "0x1234567890123456789012345678901234567890",
            "amount": "100.0",
            "time": 1234567890
        });

        let payload = create_user_signed_payload(
            &action,
            action_types::USD_SEND,
            "HyperliquidTransaction:UsdSend",
            true
        );

        assert_eq!(payload.primary_type, "HyperliquidTransaction:UsdSend");
        assert_eq!(payload.domain.name, "HyperliquidSignTransaction");

        // Check that required fields were added
        let message = payload.message.as_object().unwrap();
        assert_eq!(message.get("hyperliquidChain").unwrap(), "Mainnet");
        assert_eq!(message.get("signatureChainId").unwrap(), "0x66eee");
    }

    #[test]
    fn test_address_to_bytes() {
        let address = "0x1234567890123456789012345678901234567890";
        let bytes = address_to_bytes(address).unwrap();
        assert_eq!(bytes.len(), 20);

        // Test without 0x prefix
        let address_no_prefix = "1234567890123456789012345678901234567890";
        let bytes2 = address_to_bytes(address_no_prefix).unwrap();
        assert_eq!(bytes, bytes2);
    }

    #[test]
    fn test_signature_creation() {
        let sig = Signature::new(
            "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
            "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
            27
        );

        assert_eq!(sig.v, 27);
        assert_eq!(sig.r.len(), 66); // 0x + 64 hex chars
        assert_eq!(sig.s.len(), 66);
    }

    #[test]
    fn test_sign_message() {
        // This test uses a known private key for deterministic testing
        let private_key = "0x1111111111111111111111111111111111111111111111111111111111111111";

        let agent = PhantomAgent::mainnet("0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef");
        let payload = create_l1_payload(&agent);

        let signature = sign_message(private_key, &payload).unwrap();

        // Verify signature format
        assert_eq!(signature.v, 27 || signature.v == 28);
        assert!(signature.r.starts_with("0x"));
        assert!(signature.s.starts_with("0x"));
        assert_eq!(signature.r.len(), 66);
        assert_eq!(signature.s.len(), 66);
    }
}

/// Multi-sig envelope signing functionality
///
/// This module provides functions for creating and signing multi-signature
/// envelopes for Hyperliquid transactions.

/// Sign a multi-sig envelope with a single signature
///
/// # Arguments
///
/// * `private_key` - The private key of the signer
/// * `envelope` - The multi-sig envelope to sign
/// * `environment` - The network environment (mainnet/testnet)
///
/// # Returns
///
/// A signature string that can be added to the envelope's signatures array
///
/// # Example
///
/// ```
/// use hyperliquid_core::crypto::{MultiSigEnvelope, sign_multi_sig_envelope};
/// use serde_json::json;
///
/// let inner_action = json!({
///     "type": "order",
///     "coin": "BTC",
///     "is_buy": true,
///     "sz": "0.1",
///     "limit_px": "50000.0"
/// });
///
/// let mut envelope = MultiSigEnvelope::new(inner_action, "0xmultiSigUser", 1, None);
/// let signature = sign_multi_sig_envelope(
///     "0xprivateKey",
///     &envelope,
///     crate::types::Environment::Mainnet
/// ).unwrap();
///
/// envelope.add_signature(signature);
/// ```
pub fn sign_multi_sig_envelope(
    private_key: &str,
    envelope: &MultiSigEnvelope,
    environment: crate::types::Environment,
) -> Result<String, HyperliquidError> {
    // Serialize the inner action to bytes for the envelope
    let inner_bytes = rmp_serde::to_vec_named(&envelope.inner)
        .map_err(|e| HyperliquidError::Signing(format!("Failed to serialize inner action: {}", e)))?;

    // Create envelope message with all required fields
    let envelope_message = json!({
        "hyperliquidChain": environment.chain_name(),
        "inner": format!("0x{}", hex::encode(inner_bytes)),
        "multiSigUser": envelope.multi_sig_user,
        "signatures": envelope.signatures,
        "nonce": envelope.nonce,
        "vaultAddress": envelope.vault_address.clone().unwrap_or_default()
    });

    // Create EIP-712 message for envelope signing
    let mut message_types = HashMap::new();
    message_types.insert("EIP712Domain".to_string(), action_types::EIP712_DOMAIN.to_vec());
    message_types.insert("MultiSigEnvelope".to_string(), action_types::MULTI_SIG_ENVELOPE.to_vec());

    let message = EIP712Message {
        domain: EIP712Domain::hyperliquid_mainnet(),
        message_types,
        primary_type: "MultiSigEnvelope".to_string(),
        message: envelope_message,
    };

    // Sign the message
    let signature = sign_message(private_key, &message)?;

    // Return signature in the format expected by the API
    Ok(signature.to_hex())
}

/// Helper function to create a multi-sig envelope from an inner action
///
/// # Arguments
///
/// * `inner_action` - The inner action to execute (order, transfer, etc.)
/// * `multi_sig_user` - The multi-sig user address
/// * `nonce` - Nonce for the envelope
/// * `vault_address` - Optional vault address
///
/// # Returns
///
/// A new MultiSigEnvelope ready for signing
pub fn create_multi_sig_envelope(
    inner_action: serde_json::Value,
    multi_sig_user: impl Into<String>,
    nonce: u64,
    vault_address: Option<impl Into<String>>,
) -> MultiSigEnvelope {
    MultiSigEnvelope::new(inner_action, multi_sig_user, nonce, vault_address)
}

/// Verify that a multi-sig envelope has sufficient signatures
///
/// # Arguments
///
/// * `envelope` - The multi-sig envelope to check
/// * `threshold` - The minimum number of signatures required
///
/// # Returns
///
/// True if the envelope has enough signatures, false otherwise
pub fn verify_multi_sig_envelope(envelope: &MultiSigEnvelope, threshold: u32) -> bool {
    envelope.has_sufficient_signatures(threshold)
}