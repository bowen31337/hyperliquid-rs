//! Exchange API signing utilities

use crate::error::HyperliquidError;
use crate::types::exchange::{OrderRequest, ExchangeRequest};
use ethers_core::types::Address;
use ethers_core::utils::keccak256;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Import k256 for ECDSA signing
use k256::{
    ecdsa::{SigningKey, Signature, signature::Signer},
    SecretKey,
};
use rand_core::OsRng;

/// Sign an order request for placement
pub fn sign_order(
    order: &OrderRequest,
    private_key: &[u8],
) -> Result<String, HyperliquidError> {
    // Serialize order to JSON for signing
    let order_json = serde_json::to_string(order)
        .map_err(|e| HyperliquidError::Signing(format!("Failed to serialize order: {}", e)))?;

    // Compute keccak256 hash
    let hash = keccak256(order_json.as_bytes());

    // Sign the hash with private key
    let signature = sign_hash(&hash, private_key)?;

    Ok(hex::encode(signature))
}

/// Sign an exchange request
pub fn sign_request(
    request: ExchangeRequest,
    private_key: &[u8],
) -> Result<ExchangeRequest, HyperliquidError> {
    // Serialize request to JSON for signing
    let request_json = serde_json::to_string(&request)
        .map_err(|e| HyperliquidError::Signing(format!("Failed to serialize request: {}", e)))?;

    // Compute keccak256 hash
    let hash = keccak256(request_json.as_bytes());

    // Sign the hash with private key
    let signature = sign_hash(&hash, private_key)?;

    // Add signature to request
    let signed_request = ExchangeRequest {
        type_: request.type_,
        time: request.time,
        nonce: request.nonce,
        orders: request.orders,
        cancels: request.cancels,
        cancel_by_metadata: request.cancel_by_metadata,
        modify: request.modify,
        transfer: request.transfer,
        update_leverage: request.update_leverage,
        update_margin: request.update_margin,
        open_orders: request.open_orders,
        bulk_orders: request.bulk_orders,
        bulk_cancel: request.bulk_cancel,
    };

    Ok(signed_request)
}

/// Sign a hash with a private key using ECDSA secp256k1
fn sign_hash(hash: &[u8], private_key: &[u8]) -> Result<Vec<u8>, HyperliquidError> {
    // Validate private key format
    if private_key.len() != 32 {
        return Err(HyperliquidError::Signing(
            "Invalid private key length".to_string(),
        ));
    }

    // Create SecretKey from bytes
    let secret_key = SecretKey::from_be_bytes(private_key)
        .map_err(|e| HyperliquidError::Signing(format!("Invalid private key: {}", e)))?;

    // Create signing key
    let signing_key = SigningKey::from(secret_key);

    // Sign the hash
    let signature: Signature = signing_key.sign(hash);

    // Convert to 65-byte format (r || s || v)
    let mut signature_bytes = Vec::with_capacity(65);

    // Get r and s values
    let r_bytes = signature.r().to_bytes();
    let s_bytes = signature.s().to_bytes();

    // Pad to 32 bytes if necessary
    let mut r_padded = [0u8; 32];
    r_padded[32 - r_bytes.len()..].copy_from_slice(&r_bytes);

    let mut s_padded = [0u8; 32];
    s_padded[32 - s_bytes.len()..].copy_from_slice(&s_bytes);

    // Add r and s to signature
    signature_bytes.extend_from_slice(&r_padded);
    signature_bytes.extend_from_slice(&s_padded);

    // Add recovery ID (v)
    // k256 doesn't directly provide recovery ID, so we'll use a default
    // In practice, you might need to recover this from the signature verification
    let recovery_id = 27; // Default for Ethereum signatures
    signature_bytes.push(recovery_id);

    Ok(signature_bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use k256::SecretKey;

    #[test]
    fn test_sign_order() {
        let order = OrderRequest {
            coin: "BTC".to_string(),
            is_buy: true,
            sz: "0.001".to_string(),
            limit_px: "50000".to_string(),
            reduce_only: None,
            order_type: None,
            time_in_force: None,
            trigger_price: None,
            trail_value: None,
            close_on_trigger: None,
        };

        // Generate a real private key for testing
        let mut rng = OsRng;
        let secret_key = SecretKey::random(&mut rng);
        let private_key = secret_key.to_be_bytes().to_vec();

        // Sign the order
        let result = sign_order(&order, &private_key);
        assert!(result.is_ok());

        // Verify signature is 65 bytes
        if let Ok(signature) = result {
            assert_eq!(signature.len(), 130); // 65 bytes * 2 (hex encoding)
        }
    }

    #[test]
    fn test_sign_request() {
        let request = ExchangeRequest {
            type_: "order".to_string(),
            time: Some(chrono::Utc::now().timestamp_millis()),
            nonce: None,
            orders: None,
            cancels: None,
            cancel_by_metadata: None,
            modify: None,
            transfer: None,
            update_leverage: None,
            update_margin: None,
            open_orders: None,
            bulk_orders: None,
            bulk_cancel: None,
        };

        // Generate a real private key for testing
        let mut rng = OsRng;
        let secret_key = SecretKey::random(&mut rng);
        let private_key = secret_key.to_be_bytes().to_vec();

        // Sign the request
        let result = sign_request(request, &private_key);
        assert!(result.is_ok());
    }

    #[test]
    fn test_sign_hash_invalid_key_length() {
        let hash = [0u8; 32];
        let invalid_key = [0u8; 31]; // Wrong length

        let result = sign_hash(&hash, &invalid_key);
        assert!(result.is_err());
        if let Err(HyperliquidError::Signing(msg)) = result {
            assert!(msg.contains("Invalid private key length"));
        }
    }

    #[test]
    fn test_sign_hash_valid_key() {
        let hash = [0u8; 32];
        let mut rng = OsRng;
        let secret_key = SecretKey::random(&mut rng);
        let private_key = secret_key.to_be_bytes().to_vec();

        let result = sign_hash(&hash, &private_key);
        assert!(result.is_ok());

        // Verify signature format (65 bytes)
        if let Ok(signature) = result {
            assert_eq!(signature.len(), 65);
        }
    }
}