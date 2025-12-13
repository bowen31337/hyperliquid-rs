//! Tests for Exchange API types and client

use crate::{
    types::{
        exchange::{OrderRequest, OrderType, TimeInForce},
        Environment,
    },
    exchange::{ExchangeClient, ExchangeClientConfig},
};
use ethers_core::types::Address;
use serde_json;
use std::str::FromStr;

#[test]
fn test_order_request_creation() {
    let order = OrderRequest {
        coin: "BTC".to_string(),
        is_buy: true,
        sz: "0.001".to_string(),
        limit_px: "50000".to_string(),
        reduce_only: None,
        order_type: Some(OrderType::Limit),
        time_in_force: Some(TimeInForce::GoodTillCanceled),
        trigger_price: None,
        trail_value: None,
        close_on_trigger: None,
    };

    // Test serialization
    let json = serde_json::to_string(&order).unwrap();
    assert!(json.contains("\"coin\":\"BTC\""));
    assert!(json.contains("\"isBuy\":true"));

    // Test deserialization
    let deserialized: OrderRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.coin, "BTC");
    assert_eq!(deserialized.is_buy, true);
}

#[test]
fn test_exchange_client_config() {
    let address = Address::from_str("0x1234567890abcdef1234567890abcdef12345678").unwrap();
    let config = ExchangeClientConfig::mainnet(address);

    assert_eq!(config.base_url, "https://api.hyperliquid.xyz");
    assert_eq!(config.account, address);
}

#[tokio::test]
async fn test_exchange_client_creation() {
    let address = Address::from_str("0x1234567890abcdef1234567890abcdef12345678").unwrap();
    let config = ExchangeClientConfig::testnet(address);
    let client = ExchangeClient::new(config);

    assert_eq!(client.config.base_url, "https://api.hyperliquid-testnet.xyz");
}

#[test]
fn test_ioc_order_request_serialization() {
    let order = OrderRequest {
        coin: "ETH".to_string(),
        is_buy: false,
        sz: "0.5".to_string(),
        limit_px: "3000".to_string(),
        reduce_only: Some(false),
        order_type: Some(OrderType::Limit),
        time_in_force: Some(TimeInForce::ImmediateOrCancel),
        trigger_price: None,
        trail_value: None,
        close_on_trigger: None,
    };

    // Test serialization matches Feature #102 requirements
    let json = serde_json::to_string(&order).unwrap();
    println!("IOC Order JSON: {}", json);

    assert!(json.contains("\"coin\":\"ETH\""));
    assert!(json.contains("\"isBuy\":false"));
    assert!(json.contains("\"sz\":\"0.5\""));
    assert!(json.contains("\"limitPx\":\"3000\""));
    assert!(json.contains("\"tif\":\"Ioc\""));

    // Test deserialization
    let deserialized: OrderRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.coin, "ETH");
    assert_eq!(deserialized.is_buy, false);
    assert_eq!(deserialized.time_in_force, Some(TimeInForce::ImmediateOrCancel));
}

#[test]
fn test_ioc_vs_gtc_order_types() {
    // Test GTC order
    let gtc_order = OrderRequest {
        coin: "BTC".to_string(),
        is_buy: true,
        sz: "0.001".to_string(),
        limit_px: "50000".to_string(),
        reduce_only: None,
        order_type: Some(OrderType::Limit),
        time_in_force: Some(TimeInForce::GoodTillCanceled),
        trigger_price: None,
        trail_value: None,
        close_on_trigger: None,
    };

    // Test IOC order
    let ioc_order = OrderRequest {
        coin: "ETH".to_string(),
        is_buy: false,
        sz: "0.5".to_string(),
        limit_px: "3000".to_string(),
        reduce_only: Some(false),
        order_type: Some(OrderType::Limit),
        time_in_force: Some(TimeInForce::ImmediateOrCancel),
        trigger_price: None,
        trail_value: None,
        close_on_trigger: None,
    };

    let gtc_json = serde_json::to_string(gtc_order).unwrap();
    let ioc_json = serde_json::to_string(&ioc_order).unwrap();

    // Verify different TIF values
    assert!(gtc_json.contains("\"tif\":\"Gtc\""));
    assert!(ioc_json.contains("\"tif\":\"Ioc\""));
    assert_ne!(gtc_json, ioc_json);
}