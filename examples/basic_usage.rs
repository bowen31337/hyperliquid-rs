//! Basic usage example for Hyperliquid Rust SDK
//!
//! This example demonstrates the core functionality including:
//! - Creating clients
//! - Getting market data
//! - Signing transactions

use hyperliquid_core::{
    client::HttpClient,
    info::InfoClient,
    crypto::{Wallet, action_hash, construct_phantom_agent},
    types::{Environment, EIP712Domain},
    error::HyperliquidError,
};
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};

#[tokio::main]
async fn main() -> Result<(), HyperliquidError> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🚀 Hyperliquid Rust SDK - Basic Usage Example\n");

    // Create HTTP client
    let http_client = HttpClient::with_default_config(Environment::Mainnet.base_url())?;
    println!("✅ HTTP client created");

    // Create Info API client
    let info_client = InfoClient::new(http_client);
    println!("✅ Info API client created");

    // Get exchange metadata
    match info_client.meta_mainnet().await {
        Ok(meta) => {
            println!("✅ Got exchange metadata");
            println!("   Universe contains {} assets", meta.universe.len());

            // Show first few assets
            for (i, asset) in meta.universe.iter().take(5).enumerate() {
                println!("   {}. {} ({} decimals)", i + 1, asset.name, asset.sz_decimals);
            }
        }
        Err(e) => {
            println!("⚠️  Could not fetch metadata (expected in example): {}", e);
        }
    }

    // Create a wallet for signing
    let private_key = "0x1111111111111111111111111111111111111111111111111111111111111111";
    let wallet = Wallet::mainnet(private_key)?;
    println!("✅ Wallet created");
    println!("   Address: {}", wallet.address());

    // Example of creating and signing an order action
    let order_action = json!({
        "type": "order",
        "orders": [{
            "a": 1,  // Asset index for BTC (example)
            "b": true,  // Buy order
            "p": "50000.0",  // Limit price
            "s": "0.1",  // Size
            "r": false,  // Not reduce only
            "t": {
                "limit": {
                    "tif": "Gtc"  // Good til cancelled
                }
            }
        }]
    });

    // Get current timestamp for nonce
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Sign the L1 action
    match wallet.sign_l1_action(&order_action, None, timestamp, None) {
        Ok(signature) => {
            println!("✅ L1 action signed successfully");
            println!("   Signature: r={}, s={}, v={}",
                &signature.r[..10],
                &signature.s[..10],
                signature.v
            );
        }
        Err(e) => {
            println!("⚠️  Signing error: {}", e);
        }
    }

    // Example of action hash computation
    match action_hash(&order_action, None, timestamp, None) {
        Ok(hash) => {
            println!("✅ Action hash computed");
            println!("   Hash: {}", &hash[..20]);

            // Create phantom agent
            let phantom_agent = construct_phantom_agent(&hash, true);
            println!("✅ Phantom agent created");
            println!("   Source: {}, ConnectionID: {}...",
                phantom_agent.source,
                &phantom_agent.connection_id[..20]
            );
        }
        Err(e) => {
            println!("⚠️  Hash computation error: {}", e);
        }
    }

    // Example user state query
    let test_address = "0x742d35bE6C8C2c3c2c2c2c2c2c2c2c2c2c2c2c2c";
    match info_client.user_state_mainnet(test_address).await {
        Ok(user_state) => {
            println!("✅ User state retrieved");
            println!("   Account value: {}", user_state.marginSummary.accountValue);
            println!("   Positions: {}", user_state.positions.len());
        }
        Err(e) => {
            println!("⚠️  Could not fetch user state (expected in example): {}", e);
        }
    }

    println!("\n🎉 Example completed successfully!");
    Ok(())
}