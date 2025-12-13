//! Multi-sig Envelope Signing Example
//!
//! This example demonstrates how to use the multi-sig envelope signing
//! functionality to create and sign multi-signature transactions for Hyperliquid.

use hyperliquid_core::{
    crypto::{MultiSigEnvelope, sign_multi_sig_envelope, create_multi_sig_envelope, verify_multi_sig_envelope},
    types::{Environment},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Multi-sig Envelope Signing Example");
    println!("==================================");

    // Example 1: Basic multi-sig order placement
    println!("\n1. Multi-sig Order Placement Example:");

    // Create an order action
    let order_action = serde_json::json!({
        "type": "order",
        "coin": "BTC",
        "is_buy": true,
        "sz": "0.1",
        "limit_px": "50000.0",
        "order_type": {
            "limit": {
                "tif": "Gtc"
            }
        }
    });

    // Create multi-sig envelope
    let mut envelope = create_multi_sig_envelope(
        order_action,
        "0x1234567890123456789012345678901234567890", // multi-sig user
        1,                                           // nonce
        None                                         // no vault
    );

    println!("✓ Created multi-sig envelope for order");
    println!("  Multi-sig user: {}", envelope.multi_sig_user);
    println!("  Nonce: {}", envelope.nonce);
    println!("  Signatures: {} (threshold: 2)", envelope.signature_count());

    // Simulate collecting signatures from authorized signers
    let signer1_key = "0x1111111111111111111111111111111111111111111111111111111111111111";
    let signer2_key = "0x2222222222222222222222222222222222222222222222222222222222222222";

    // Signer 1 signs
    let signature1 = sign_multi_sig_envelope(
        signer1_key,
        &envelope,
        Environment::Mainnet
    )?;
    envelope.add_signature(signature1);
    println!("✓ Signer 1 added signature");

    // Signer 2 signs
    let signature2 = sign_multi_sig_envelope(
        signer2_key,
        &envelope,
        Environment::Mainnet
    )?;
    envelope.add_signature(signature2);
    println!("✓ Signer 2 added signature");

    // Verify threshold is met
    let threshold = 2;
    if verify_multi_sig_envelope(&envelope, threshold) {
        println!("✓ Threshold met! Multi-sig envelope is ready for submission");
    } else {
        println!("✗ Threshold not met. Need more signatures.");
    }

    // Example 2: Multi-sig transfer with vault
    println!("\n2. Multi-sig Transfer with Vault Example:");

    let transfer_action = serde_json::json!({
        "type": "usdSend",
        "destination": "0x9876543210987654321098765432109876543210",
        "amount": "1000.0",
        "time": 1234567890
    });

    let mut vault_envelope = create_multi_sig_envelope(
        transfer_action,
        "0x2222222222222222222222222222222222222222",
        42,
        Some("0x3333333333333333333333333333333333333333") // vault address
    );

    println!("✓ Created multi-sig envelope with vault");
    println!("  Multi-sig user: {}", vault_envelope.multi_sig_user);
    println!("  Vault address: {:?}", vault_envelope.vault_address);

    // Add signatures to meet threshold
    let signature3 = sign_multi_sig_envelope(
        signer1_key,
        &vault_envelope,
        Environment::Mainnet
    )?;
    vault_envelope.add_signature(signature3);

    let signature4 = sign_multi_sig_envelope(
        signer2_key,
        &vault_envelope,
        Environment::Mainnet
    )?;
    vault_envelope.add_signature(signature4);

    if verify_multi_sig_envelope(&vault_envelope, 2) {
        println!("✓ Vault transfer envelope ready for submission");
    }

    // Example 3: Serialization and deserialization
    println!("\n3. Serialization Example:");

    let serialized = serde_json::to_string(&envelope)?;
    println!("✓ Serialized envelope: {}", serialized);

    let deserialized: MultiSigEnvelope = serde_json::from_str(&serialized)?;
    println!("✓ Deserialized envelope successfully");
    println!("  Original signatures: {}", envelope.signature_count());
    println!("  Deserialized signatures: {}", deserialized.signature_count());

    println!("\nAll examples completed successfully!");
    println!("\nKey Features Demonstrated:");
    println!("- Creating multi-sig envelopes for different action types");
    println!("- Collecting signatures from multiple authorized signers");
    println!("- Verifying threshold requirements");
    println!("- Vault address support");
    println!("- Serialization/deserialization");
    println!("- EIP-712 compliant signing");

    Ok(())
}