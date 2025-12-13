//! Tests for USD_SEND signature type handling
//!
//! This module tests the USD_SEND signature type handling functionality,
//! including action creation, EIP-712 signing, signature validation,
//! and testnet chain ID usage.

use hyperliquid_core::{
    crypto::{sign_user_signed_action, action_types, EIP712Domain},
    types::Wallet,
};
use serde_json::json;
use std::collections::HashMap;

#[test]
fn test_create_usd_send_action() {
    // Test creating a usdSend action with all required fields
    let action = json!({
        "type": "usdSend",
        "destination": "0x1234567890123456789012345678901234567890",
        "amount": "100.0",
        "time": 1234567890
    });

    // Verify action structure
    assert_eq!(action["type"], "usdSend");
    assert_eq!(action["destination"], "0x1234567890123456789012345678901234567890");
    assert_eq!(action["amount"], "100.0");
    assert_eq!(action["time"], 1234567890);
}

#[test]
fn test_usd_send_sign_types_definition() {
    // Test that USD_SEND_SIGN_TYPES is properly defined
    let usd_send_types = action_types::USD_SEND;

    // Verify all required fields are present
    assert_eq!(usd_send_types.len(), 4);

    let fields: Vec<String> = usd_send_types.iter()
        .map(|t| t.name.clone())
        .collect();

    assert!(fields.contains(&"hyperliquidChain".to_string()));
    assert!(fields.contains(&"destination".to_string()));
    assert!(fields.contains(&"amount".to_string()));
    assert!(fields.contains(&"time".to_string()));
}

#[test]
fn test_sign_usd_transfer_with_user_key() {
    // Create a test wallet with a known private key
    let private_key = "0x1234567890123456789012345678901234567890123456789012345678901234";
    let wallet = Wallet::new(private_key, true).expect("Failed to create wallet");

    // Create a usdSend action
    let action = json!({
        "destination": "0x1234567890123456789012345678901234567890",
        "amount": "100.0",
        "time": 1234567890
    });

    // Sign the action using the wallet
    let signature = wallet.sign_usd_transfer(&action)
        .expect("Failed to sign usdSend action");

    // Verify signature is not empty
    assert!(!signature.r.is_empty());
    assert!(!signature.s.is_empty());
    assert!(signature.v == 27 || signature.v == 28);
}

#[test]
fn test_sign_usd_transfer_with_wallet_method() {
    // Create a test wallet
    let private_key = "0x1234567890123456789012345678901234567890123456789012345678901234";
    let wallet = Wallet::new(private_key, true).expect("Failed to create wallet");

    // Test signing usdSend with different parameters
    let destination = "0x1234567890123456789012345678901234567890";
    let amount = "100.0";
    let time = 1234567890;

    let action = json!({
        "destination": destination,
        "amount": amount,
        "time": time
    });

    let signature = wallet.sign_usd_transfer(&action)
        .expect("Failed to sign usdSend");

    // Verify signature format
    assert!(!signature.r.is_empty());
    assert!(!signature.s.is_empty());
    assert!(signature.v == 27 || signature.v == 28);
}

#[test]
fn test_usd_transfer_signature_validation() {
    // Create a test wallet
    let private_key = "0x1234567890123456789012345678901234567890123456789012345678901234";
    let wallet = Wallet::new(private_key, true).expect("Failed to create wallet");

    // Create and sign a usdSend action
    let destination = "0x1234567890123456789012345678901234567890";
    let amount = "100.0";
    let time = 1234567890;

    let action = json!({
        "destination": destination,
        "amount": amount,
        "time": time
    });

    let signature = wallet.sign_usd_transfer(&action)
        .expect("Failed to sign usdSend");

    // Verify signature is valid (non-empty and proper format)
    assert!(!signature.r.is_empty());
    assert!(!signature.s.is_empty());
    assert!(signature.v == 27 || signature.v == 28);

    // Test with different amounts
    let action2 = json!({
        "destination": destination,
        "amount": "50.0",
        "time": time
    });

    let signature2 = wallet.sign_usd_transfer(&action2)
        .expect("Failed to sign second usdSend");

    // Signatures should be different
    assert_ne!(signature.r, signature2.r);
    assert_ne!(signature.s, signature2.s);
}

#[test]
fn test_usd_transfer_with_testnet_chain_id() {
    // Create a test wallet
    let private_key = "0x1234567890123456789012345678901234567890123456789012345678901234";
    let wallet = Wallet::new(private_key, false).expect("Failed to create wallet"); // testnet

    // Test with testnet chain ID (0x66eee)
    let destination = "0x1234567890123456789012345678901234567890";
    let amount = "100.0";
    let time = 1234567890;

    let action = json!({
        "destination": destination,
        "amount": amount,
        "time": time
    });

    // Sign usdSend action (should use testnet chain ID internally)
    let signature = wallet.sign_usd_transfer(&action)
        .expect("Failed to sign usdSend with testnet chain");

    // Verify signature is generated correctly
    assert!(!signature.r.is_empty());
    assert!(!signature.s.is_empty());
    assert!(signature.v == 27 || signature.v == 28);
}

#[test]
fn test_usd_transfer_signature_with_different_destinations() {
    // Create a test wallet
    let private_key = "0x1234567890123456789012345678901234567890123456789012345678901234";
    let wallet = Wallet::new(private_key, true).expect("Failed to create wallet");

    let amount = "100.0";
    let time = 1234567890;

    // Test with different destination addresses
    let destinations = [
        "0x1234567890123456789012345678901234567890",
        "0x0987654321098765432109876543210987654321",
        "0xabcdefabcdefabcdefabcdefabcdefabcdefabcdef",
    ];

    let mut signatures = Vec::new();

    for dest in &destinations {
        let action = json!({
            "destination": dest,
            "amount": amount,
            "time": time
        });

        let signature = wallet.sign_usd_transfer(&action)
            .expect("Failed to sign usdSend");
        signatures.push(signature);
    }

    // Verify all signatures are different
    assert_eq!(signatures.len(), 3);
    assert_ne!(signatures[0].r, signatures[1].r);
    assert_ne!(signatures[1].r, signatures[2].r);
    assert_ne!(signatures[0].r, signatures[2].r);
}

#[test]
fn test_usd_transfer_signature_with_different_amounts() {
    // Create a test wallet
    let private_key = "0x1234567890123456789012345678901234567890123456789012345678901234";
    let wallet = Wallet::new(private_key, true).expect("Failed to create wallet");

    let destination = "0x1234567890123456789012345678901234567890";
    let time = 1234567890;

    // Test with different amounts
    let amounts = ["100.0", "50.5", "25.25", "0.1"];

    let mut signatures = Vec::new();

    for amount in &amounts {
        let action = json!({
            "destination": destination,
            "amount": amount,
            "time": time
        });

        let signature = wallet.sign_usd_transfer(&action)
            .expect("Failed to sign usdSend");
        signatures.push(signature);
    }

    // Verify all signatures are different
    assert_eq!(signatures.len(), 4);
    for i in 0..signatures.len() {
        for j in (i + 1)..signatures.len() {
            assert_ne!(signatures[i].r, signatures[j].r);
        }
    }
}

#[test]
fn test_usd_transfer_signature_with_different_times() {
    // Create a test wallet
    let private_key = "0x1234567890123456789012345678901234567890123456789012345678901234";
    let wallet = Wallet::new(private_key, true).expect("Failed to create wallet");

    let destination = "0x1234567890123456789012345678901234567890";
    let amount = "100.0";

    // Test with different timestamps
    let times = [1234567890, 1234567891, 1234567892, 1234567893];

    let mut signatures = Vec::new();

    for time in × {
        let action = json!({
            "destination": destination,
            "amount": amount,
            "time": *time
        });

        let signature = wallet.sign_usd_transfer(&action)
            .expect("Failed to sign usdSend");
        signatures.push(signature);
    }

    // Verify all signatures are different
    assert_eq!(signatures.len(), 4);
    for i in 0..signatures.len() {
        for j in (i + 1)..signatures.len() {
            assert_ne!(signatures[i].r, signatures[j].r);
        }
    }
}

#[test]
fn test_usd_send_sign_types_fields() {
    // Test that USD_SEND sign types have correct field mappings
    let usd_send_types = action_types::USD_SEND;

    // Create a map of field names to their expected types
    let mut field_types = HashMap::new();
    for field in usd_send_types {
        field_types.insert(field.name.as_str(), field.type_.as_str());
    }

    // Verify field types
    assert_eq!(field_types["hyperliquidChain"], "string");
    assert_eq!(field_types["destination"], "string");
    assert_eq!(field_types["amount"], "string");
    assert_eq!(field_types["time"], "uint64");
}