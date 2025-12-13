//! Tests for Serde skip_serializing_if functionality

use hyperliquid_core::types::{AssetMeta, VaultMeta, UserState, Position, PositionDetails, AssetPosition};
use hyperliquid_core::types::{MarginSummary, CrossMarginSummary};

#[tokio::test]
async fn test_asset_meta_skip_serializing_if() {
    // Create AssetMeta with some None fields
    let asset_meta = AssetMeta {
        name: "BTC".to_string(),
        onlyIsolated: false,
        szDecimals: 8,
        maxLeverage: 20,
        maxDynamicLeverage: None,
        type_: None,
        tokens: None,
        maxOi: None,
        underlying: None,
        isInverse: None,
    };

    let json = serde_json::to_string(&asset_meta).unwrap();
    let expected = r#"{"name":"BTC","onlyIsolated":false,"szDecimals":8,"maxLeverage":20}"#;
    assert_eq!(json, expected);

    // Test deserialization back
    let deserialized: AssetMeta = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.name, "BTC");
    assert_eq!(deserialized.onlyIsolated, false);
    assert_eq!(deserialized.szDecimals, 8);
    assert_eq!(deserialized.maxLeverage, 20);
    assert!(deserialized.maxDynamicLeverage.is_none());
    assert!(deserialized.type_.is_none());
    assert!(deserialized.tokens.is_none());
    assert!(deserialized.maxOi.is_none());
    assert!(deserialized.underlying.is_none());
    assert!(deserialized.isInverse.is_none());
}

#[tokio::test]
async fn test_asset_meta_with_some_fields() {
    // Create AssetMeta with some fields set
    let asset_meta = AssetMeta {
        name: "ETH".to_string(),
        onlyIsolated: true,
        szDecimals: 8,
        maxLeverage: 20,
        maxDynamicLeverage: Some(25),
        type_: Some("perpetual".to_string()),
        tokens: None,
        maxOi: Some("1000000".to_string()),
        underlying: None,
        isInverse: Some(true),
    };

    let json = serde_json::to_string(&asset_meta).unwrap();
    // Only non-None fields should be present
    assert!(json.contains(r#""name":"ETH""#));
    assert!(json.contains(r#""onlyIsolated":true"#));
    assert!(json.contains(r#""szDecimals":8"#));
    assert!(json.contains(r#""maxLeverage":20"#));
    assert!(json.contains(r#""maxDynamicLeverage":25"#));
    assert!(json.contains(r#""type_":"perpetual""#));
    assert!(json.contains(r#""maxOi":"1000000""#));
    assert!(json.contains(r#""isInverse":true"#));

    // None fields should be absent
    assert!(!json.contains("tokens"));
    assert!(!json.contains("underlying"));

    // Test deserialization
    let deserialized: AssetMeta = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.name, "ETH");
    assert_eq!(deserialized.maxDynamicLeverage, Some(25));
    assert_eq!(deserialized.type_, Some("perpetual".to_string()));
    assert_eq!(deserialized.maxOi, Some("1000000".to_string()));
    assert_eq!(deserialized.isInverse, Some(true));
    assert!(deserialized.tokens.is_none());
    assert!(deserialized.underlying.is_none());
}

#[tokio::test]
async fn test_vault_meta_skip_serializing_if() {
    // Create VaultMeta with None price
    let vault_meta = VaultMeta {
        vault: "0x1234567890abcdef".parse().unwrap(),
        name: "Test Vault".to_string(),
        creator: "0xabcdef1234567890".parse().unwrap(),
        creatorLong: "0.1".to_string(),
        creatorShort: "0.9".to_string(),
        price: None,
    };

    let json = serde_json::to_string(&vault_meta).unwrap();
    let expected = r#"{"vault":"0x1234567890abcdef","name":"Test Vault","creator":"0xabcdef1234567890","creatorLong":"0.1","creatorShort":"0.9"}"#;
    assert_eq!(json, expected);

    // Test with price set
    let vault_meta_with_price = VaultMeta {
        vault: "0x1234567890abcdef".parse().unwrap(),
        name: "Test Vault".to_string(),
        creator: "0xabcdef1234567890".parse().unwrap(),
        creatorLong: "0.1".to_string(),
        creatorShort: "0.9".to_string(),
        price: Some("10000.0".to_string()),
    };

    let json_with_price = serde_json::to_string(&vault_meta_with_price).unwrap();
    assert!(json_with_price.contains(r#""price":"10000.0""#));
}

#[tokio::test]
async fn test_user_state_skip_serializing_if() {
    // Create UserState without crossMarginSummary
    let user_state = UserState {
        marginSummary: MarginSummary::new(
            "10000.0".to_string(),
            "2000.0".to_string(),
            "5000.0".to_string(),
            "8000.0".to_string(),
        ),
        crossMarginSummary: None,
        positions: vec![],
        withdrawable: "8000.0".to_string(),
        assetPositions: vec![],
    };

    let json = serde_json::to_string(&user_state).unwrap();
    // crossMarginSummary should be omitted
    assert!(!json.contains("crossMarginSummary"));

    // Test with crossMarginSummary
    let user_state_with_cross = UserState {
        marginSummary: MarginSummary::new(
            "10000.0".to_string(),
            "2000.0".to_string(),
            "5000.0".to_string(),
            "8000.0".to_string(),
        ),
        crossMarginSummary: Some(CrossMarginSummary {
            accountValue: "5000.0".to_string(),
            totalMarginUsed: "1000.0".to_string(),
            totalNtlPos: "2500.0".to_string(),
            totalRawUsd: "4000.0".to_string(),
        }),
        positions: vec![],
        withdrawable: "8000.0".to_string(),
        assetPositions: vec![],
    };

    let json_with_cross = serde_json::to_string(&user_state_with_cross).unwrap();
    assert!(json_with_cross.contains("crossMarginSummary"));
}

#[tokio::test]
async fn test_position_details_skip_serializing_if() {
    // Create PositionDetails with many None fields
    let position_details = PositionDetails {
        szi: "1.0".to_string(),
        entryPx: None,
        leverage: None,
        liquidationPx: None,
        positionValue: "50000.0".to_string(),
        marginUsed: None,
        openSize: "1.0".to_string(),
        rawPNL: None,
        returnOnEquity: None,
        type_: "cross".to_string(),
        userID: "12345".to_string(),
        account: None,
        cumFunding: None,
        maxCost: None,
        maxLeverage: None,
        positionUUID: None,
        pendingFunding: None,
    };

    let json = serde_json::to_string(&position_details).unwrap();
    let expected = r#"{"szi":"1.0","positionValue":"50000.0","openSize":"1.0","type_":"cross","userID":"12345"}"#;
    assert_eq!(json, expected);

    // Test with some fields set
    let position_details_with_fields = PositionDetails {
        szi: "1.0".to_string(),
        entryPx: Some("50000.0".to_string()),
        leverage: Some("10.0".to_string()),
        liquidationPx: None,
        positionValue: "50000.0".to_string(),
        marginUsed: Some("5000.0".to_string()),
        openSize: "1.0".to_string(),
        rawPNL: Some("1000.0".to_string()),
        returnOnEquity: None,
        type_: "cross".to_string(),
        userID: "12345".to_string(),
        account: None,
        cumFunding: None,
        maxCost: None,
        maxLeverage: None,
        positionUUID: Some("uuid-123".to_string()),
        pendingFunding: None,
    };

    let json_with_fields = serde_json::to_string(&position_details_with_fields).unwrap();
    assert!(json_with_fields.contains(r#""entryPx":"50000.0""#));
    assert!(json_with_fields.contains(r#""leverage":"10.0""#));
    assert!(json_with_fields.contains(r#""marginUsed":"5000.0""#));
    assert!(json_with_fields.contains(r#""rawPNL":"1000.0""#));
    assert!(json_with_fields.contains(r#""positionUUID":"uuid-123""#));

    // None fields should be absent
    assert!(!json_with_fields.contains("liquidationPx"));
    assert!(!json_with_fields.contains("returnOnEquity"));
    assert!(!json_with_fields.contains("account"));
}

#[tokio::test]
async fn test_asset_position_skip_serializing_if() {
    // Create AssetPosition with all None fields
    let asset_position = AssetPosition {
        time: 1234567890,
        token: "USDC".to_string(),
        delta: None,
        deltaUsd: None,
        total: None,
        totalUsd: None,
        type_: None,
    };

    let json = serde_json::to_string(&asset_position).unwrap();
    let expected = r#"{"time":1234567890,"token":"USDC"}"#;
    assert_eq!(json, expected);

    // Test with some fields set
    let asset_position_with_fields = AssetPosition {
        time: 1234567890,
        token: "BTC".to_string(),
        delta: Some("1.0".to_string()),
        deltaUsd: Some("50000.0".to_string()),
        total: None,
        totalUsd: Some("100000.0".to_string()),
        type_: Some("trade".to_string()),
    };

    let json_with_fields = serde_json::to_string(&asset_position_with_fields).unwrap();
    assert!(json_with_fields.contains(r#""delta":"1.0""#));
    assert!(json_with_fields.contains(r#""deltaUsd":"50000.0""#));
    assert!(json_with_fields.contains(r#""totalUsd":"100000.0""#));
    assert!(json_with_fields.contains(r#""type_":"trade""#));

    // None fields should be absent
    assert!(!json_with_fields.contains("total"));
}

#[tokio::test]
async fn test_serialization_size_reduction() {
    // Demonstrate the size reduction benefit of skip_serializing_if

    // Struct with all None fields
    let asset_meta_none = AssetMeta {
        name: "BTC".to_string(),
        onlyIsolated: false,
        szDecimals: 8,
        maxLeverage: 20,
        maxDynamicLeverage: None,
        type_: None,
        tokens: None,
        maxOi: None,
        underlying: None,
        isInverse: None,
    };

    let json_none = serde_json::to_string(&asset_meta_none).unwrap();
    println!("JSON with all None fields omitted: {}", json_none);

    // Struct with all Some fields
    let asset_meta_some = AssetMeta {
        name: "BTC".to_string(),
        onlyIsolated: false,
        szDecimals: 8,
        maxLeverage: 20,
        maxDynamicLeverage: Some(25),
        type_: Some("perpetual".to_string()),
        tokens: Some(vec![]),
        maxOi: Some("1000000".to_string()),
        underlying: Some("BTC".to_string()),
        isInverse: Some(true),
    };

    let json_some = serde_json::to_string(&asset_meta_some).unwrap();
    println!("JSON with all fields present: {}", json_some);

    // Calculate size reduction
    let size_none = json_none.len();
    let size_some = json_some.len();
    let reduction = size_some - size_none;
    let percentage = (reduction as f64 / size_some as f64) * 100.0;

    println!("Size reduction: {} bytes ({}%)", reduction, percentage);

    // The JSON with None fields should be significantly smaller
    assert!(size_none < size_some);
    assert!(percentage > 0.0);
}

#[tokio::test]
async fn test_roundtrip_consistency() {
    // Test that structs can be serialized and deserialized consistently
    // regardless of whether fields are None or Some

    let original = AssetMeta {
        name: "ETH".to_string(),
        onlyIsolated: true,
        szDecimals: 8,
        maxLeverage: 20,
        maxDynamicLeverage: Some(25),
        type_: None,
        tokens: Some(vec![]),
        maxOi: None,
        underlying: Some("ETH".to_string()),
        isInverse: None,
    };

    // Serialize
    let json = serde_json::to_string(&original).unwrap();
    println!("Serialized JSON: {}", json);

    // Deserialize
    let deserialized: AssetMeta = serde_json::from_str(&json).unwrap();

    // The deserialized struct should be equivalent to the original
    assert_eq!(deserialized.name, original.name);
    assert_eq!(deserialized.onlyIsolated, original.onlyIsolated);
    assert_eq!(deserialized.szDecimals, original.szDecimals);
    assert_eq!(deserialized.maxLeverage, original.maxLeverage);
    assert_eq!(deserialized.maxDynamicLeverage, original.maxDynamicLeverage);
    assert_eq!(deserialized.type_, original.type_);
    assert_eq!(deserialized.tokens, original.tokens);
    assert_eq!(deserialized.maxOi, original.maxOi);
    assert_eq!(deserialized.underlying, original.underlying);
    assert_eq!(deserialized.isInverse, original.isInverse);

    // Serialize the deserialized struct again
    let json2 = serde_json::to_string(&deserialized).unwrap();

    // Should be identical
    assert_eq!(json, json2);
}