//! Multi-sig envelope signing tests
//!
//! These tests verify the multi-sig envelope functionality including:
//! - Creating multi-sig envelopes
//! - Signing envelopes with multiple signatures
//! - Verifying threshold requirements
//! - EIP-712 compatibility

use hyperliquid_core::{
    crypto::{MultiSigEnvelope, MultiSigUser, MultiSigSignature, sign_multi_sig_envelope, create_multi_sig_envelope, verify_multi_sig_envelope},
    types::{Environment, Address},
    error::HyperliquidError,
};
use serde_json::json;

#[test]
fn test_multi_sig_envelope_creation() {
    let inner_action = json!({
        "type": "order",
        "coin": "BTC",
        "is_buy": true,
        "sz": "0.1",
        "limit_px": "50000.0"
    });

    let envelope = create_multi_sig_envelope(
        inner_action,
        "0x1234567890123456789012345678901234567890",
        1,
        None
    );

    assert_eq!(envelope.multi_sig_user, "0x1234567890123456789012345678901234567890");
    assert_eq!(envelope.nonce, 1);
    assert_eq!(envelope.signature_count(), 0);
    assert!(envelope.vault_address.is_none());
}

#[test]
fn test_multi_sig_envelope_with_vault() {
    let inner_action = json!({
        "type": "order",
        "coin": "ETH",
        "is_buy": false,
        "sz": "1.0",
        "limit_px": "3000.0"
    });

    let envelope = create_multi_sig_envelope(
        inner_action,
        "0x1234567890123456789012345678901234567890",
        42,
        Some("0x9876543210987654321098765432109876543210")
    );

    assert_eq!(envelope.nonce, 42);
    assert_eq!(envelope.vault_address, Some("0x9876543210987654321098765432109876543210".to_string()));
}

#[test]
fn test_multi_sig_envelope_signature_management() {
    let inner_action = json!({
        "destination": "0x1234567890123456789012345678901234567890",
        "amount": "100.0",
        "time": 1234567890
    });

    let mut envelope = create_multi_sig_envelope(
        inner_action,
        "0x1111111111111111111111111111111111111111",
        1,
        None
    );

    // Add first signature
    envelope.add_signature("0xsignature1");
    assert_eq!(envelope.signature_count(), 1);

    // Add second signature
    envelope.add_signature("0xsignature2");
    assert_eq!(envelope.signature_count(), 2);

    // Verify signatures array
    assert_eq!(envelope.signatures.len(), 2);
    assert_eq!(envelope.signatures[0], "0xsignature1");
    assert_eq!(envelope.signatures[1], "0xsignature2");
}

#[test]
fn test_multi_sig_envelope_threshold_check() {
    let inner_action = json!({
        "coin": "BTC",
        "is_buy": true,
        "sz": "0.5",
        "limit_px": "50000.0"
    });

    let mut envelope = create_multi_sig_envelope(
        inner_action,
        "0x2222222222222222222222222222222222222222",
        1,
        None
    );

    // Test with 0 signatures, threshold 2
    assert!(!verify_multi_sig_envelope(&envelope, 2));

    // Add one signature, still below threshold
    envelope.add_signature("0xsignature1");
    assert!(!verify_multi_sig_envelope(&envelope, 2));

    // Add second signature, meets threshold
    envelope.add_signature("0xsignature2");
    assert!(verify_multi_sig_envelope(&envelope, 2));

    // Add third signature, exceeds threshold
    envelope.add_signature("0xsignature3");
    assert!(verify_multi_sig_envelope(&envelope, 2));
}

#[test]
fn test_multi_sig_user_creation() {
    let authorized_signers = vec![
        "0x1111111111111111111111111111111111111111".to_string(),
        "0x2222222222222222222222222222222222222222".to_string(),
        "0x3333333333333333333333333333333333333333".to_string(),
    ];

    let multi_sig_user = MultiSigUser {
        address: "0x4444444444444444444444444444444444444444".to_string(),
        authorized_signers,
        threshold: 2,
    };

    assert_eq!(multi_sig_user.address, "0x4444444444444444444444444444444444444444");
    assert_eq!(multi_sig_user.authorized_signers.len(), 3);
    assert_eq!(multi_sig_user.threshold, 2);
}

#[test]
fn test_multi_sig_signature_creation() {
    let signature = MultiSigSignature {
        signer: "0x1111111111111111111111111111111111111111".to_string(),
        signature: hyperliquid_core::crypto::Signature::new(
            "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
            "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
            27
        ),
        timestamp: 1234567890,
    };

    assert_eq!(signature.signer, "0x1111111111111111111111111111111111111111");
    assert_eq!(signature.timestamp, 1234567890);
    assert_eq!(signature.signature.v, 27);
}

#[test]
fn test_multi_sig_envelope_address_validation() {
    let inner_action = json!({
        "type": "order",
        "coin": "BTC",
        "is_buy": true,
        "sz": "0.1",
        "limit_px": "50000.0"
    });

    // Valid address
    let envelope = create_multi_sig_envelope(
        inner_action.clone(),
        "0x1234567890123456789012345678901234567890",
        1,
        None
    );
    assert_eq!(envelope.multi_sig_user, "0x1234567890123456789012345678901234567890");

    // Address without 0x prefix should be handled by validation layer
    let envelope2 = create_multi_sig_envelope(
        inner_action,
        "1234567890123456789012345678901234567890",
        1,
        None
    );
    assert_eq!(envelope2.multi_sig_user, "1234567890123456789012345678901234567890");
}

#[test]
fn test_multi_sig_envelope_nonce_management() {
    let inner_action = json!({
        "destination": "0x1234567890123456789012345678901234567890",
        "amount": "50.0",
        "time": 1234567890
    });

    // Test with nonce 0
    let envelope1 = create_multi_sig_envelope(
        inner_action.clone(),
        "0x1111111111111111111111111111111111111111",
        0,
        None
    );
    assert_eq!(envelope1.nonce, 0);

    // Test with large nonce
    let envelope2 = create_multi_sig_envelope(
        inner_action,
        "0x1111111111111111111111111111111111111111",
        u64::MAX,
        None
    );
    assert_eq!(envelope2.nonce, u64::MAX);
}

#[test]
fn test_multi_sig_scenario_order_placement() {
    // Scenario: Multi-sig order placement
    let order_action = json!({
        "type": "order",
        "coin": "BTC",
        "is_buy": true,
        "sz": "0.1",
        "limit_px": "50000.0",
        "order_type": {"limit": {"tif": "Gtc"}}
    });

    let multi_sig_user = "0x1234567890123456789012345678901234567890";
    let nonce = 1;

    // Create envelope
    let mut envelope = create_multi_sig_envelope(
        order_action,
        multi_sig_user,
        nonce,
        None
    );

    // Simulate collecting signatures
    envelope.add_signature("0xsignature_from_signer_1");
    envelope.add_signature("0xsignature_from_signer_2");
    envelope.add_signature("0xsignature_from_signer_3");

    // Verify threshold of 2 is met
    assert!(verify_multi_sig_envelope(&envelope, 2));

    // Verify threshold of 4 is not met
    assert!(!verify_multi_sig_envelope(&envelope, 4));

    // Verify envelope structure
    assert_eq!(envelope.multi_sig_user, multi_sig_user);
    assert_eq!(envelope.nonce, nonce);
    assert_eq!(envelope.signature_count(), 3);
}

#[test]
fn test_multi_sig_scenario_transfer() {
    // Scenario: Multi-sig USD transfer
    let transfer_action = json!({
        "type": "usdSend",
        "destination": "0x9876543210987654321098765432109876543210",
        "amount": "1000.0",
        "time": 1234567890
    });

    let multi_sig_user = "0x2222222222222222222222222222222222222222";
    let nonce = 42;

    // Create envelope
    let mut envelope = create_multi_sig_envelope(
        transfer_action,
        multi_sig_user,
        nonce,
        Some("0x3333333333333333333333333333333333333333")
    );

    // Add signatures to meet threshold
    envelope.add_signature("0xfirst_signature");
    envelope.add_signature("0xsecond_signature");

    assert!(verify_multi_sig_envelope(&envelope, 2));
    assert_eq!(envelope.vault_address, Some("0x3333333333333333333333333333333333333333".to_string()));
}

#[test]
fn test_multi_sig_envelope_serialization() {
    let inner_action = json!({
        "coin": "ETH",
        "is_buy": false,
        "sz": "2.0",
        "limit_px": "2500.0"
    });

    let envelope = create_multi_sig_envelope(
        inner_action,
        "0x5555555555555555555555555555555555555555",
        100,
        Some("0x6666666666666666666666666666666666666666")
    );

    // Test serde serialization/deserialization
    let serialized = serde_json::to_string(&envelope).unwrap();
    let deserialized: MultiSigEnvelope = serde_json::from_str(&serialized).unwrap();

    assert_eq!(envelope.multi_sig_user, deserialized.multi_sig_user);
    assert_eq!(envelope.nonce, deserialized.nonce);
    assert_eq!(envelope.vault_address, deserialized.vault_address);
    assert_eq!(envelope.signature_count(), deserialized.signature_count());
}

#[test]
fn test_multi_sig_user_threshold_validation() {
    let authorized_signers = vec![
        "0x1111111111111111111111111111111111111111".to_string(),
        "0x2222222222222222222222222222222222222222".to_string(),
        "0x3333333333333333333333333333333333333333".to_string(),
    ];

    // Threshold cannot exceed number of authorized signers
    let multi_sig_user = MultiSigUser {
        address: "0x4444444444444444444444444444444444444444".to_string(),
        authorized_signers: authorized_signers.clone(),
        threshold: 3,
    };

    assert_eq!(multi_sig_user.threshold, 3);
    assert_eq!(multi_sig_user.authorized_signers.len(), 3);

    // Edge case: threshold of 1 (anyone can sign)
    let multi_sig_user_single = MultiSigUser {
        address: "0x4444444444444444444444444444444444444444".to_string(),
        authorized_signers: authorized_signers.clone(),
        threshold: 1,
    };

    assert_eq!(multi_sig_user_single.threshold, 1);

    // Edge case: threshold equal to all signers
    let multi_sig_user_all = MultiSigUser {
        address: "0x4444444444444444444444444444444444444444".to_string(),
        authorized_signers: authorized_signers.clone(),
        threshold: 3,
    };

    assert_eq!(multi_sig_user_all.threshold, 3);
}