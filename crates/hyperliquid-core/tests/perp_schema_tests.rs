//! Tests for PerpDexSchemaInput

use hyperliquid_core::types::perp_schema::*;
use hyperliquid_core::types::Address;

#[tokio::test]
async fn test_perp_dex_schema_input_basic() {
    let schema = PerpDexSchemaInput::new(
        "BTC/USD Perpetual".to_string(),
        0,
        Some("0x1234567890abcdef1234567890abcdef12345678".to_string()),
    );

    assert_eq!(schema.full_name, "BTC/USD Perpetual");
    assert_eq!(schema.collateral_token, 0);
    assert_eq!(schema.get_oracle_updater(), Some("0x1234567890abcdef1234567890abcdef12345678"));
    assert!(schema.has_oracle_updater());
}

#[tokio::test]
async fn test_perp_dex_schema_input_without_oracle() {
    let schema = PerpDexSchemaInput::new_without_oracle(
        "BTC/USD Perpetual".to_string(),
        0,
    );

    assert_eq!(schema.full_name, "BTC/USD Perpetual");
    assert_eq!(schema.collateral_token, 0);
    assert!(schema.get_oracle_updater().is_none());
    assert!(!schema.has_oracle_updater());
}

#[tokio::test]
async fn test_perp_dex_schema_input_builder() {
    let schema = PerpDexSchemaInputBuilder::new("BTC/USD Perpetual", 0)
        .oracle_updater("0x1234567890abcdef1234567890abcdef12345678")
        .build();

    assert_eq!(schema.full_name, "BTC/USD Perpetual");
    assert_eq!(schema.collateral_token, 0);
    assert_eq!(schema.get_oracle_updater(), Some("0x1234567890abcdef1234567890abcdef12345678"));
}

#[tokio::test]
async fn test_perp_dex_schema_input_with_address() {
    let address = Address::from_str("0x1234567890abcdef1234567890abcdef12345678").unwrap();
    let schema = PerpDexSchemaInputWithAddress::new(
        "BTC/USD Perpetual".to_string(),
        0,
        Some(address.clone()),
    );

    assert_eq!(schema.full_name, "BTC/USD Perpetual");
    assert_eq!(schema.collateral_token, 0);
    assert_eq!(schema.get_oracle_updater(), Some(&address));
    assert!(schema.has_oracle_updater());

    // Test conversion to string schema
    let string_schema = schema.to_string_schema();
    assert_eq!(string_schema.get_oracle_updater(), Some("0x1234567890abcdef1234567890abcdef12345678"));
}

#[tokio::test]
async fn test_perp_dex_schema_input_builder_with_address() {
    let address = Address::from_str("0x1234567890abcdef1234567890abcdef12345678").unwrap();
    let schema = PerpDexSchemaInputBuilder::new("BTC/USD Perpetual", 0)
        .oracle_updater_from_address(address)
        .build();

    assert_eq!(schema.full_name, "BTC/USD Perpetual");
    assert_eq!(schema.collateral_token, 0);
    assert_eq!(schema.get_oracle_updater(), Some("0x1234567890abcdef1234567890abcdef12345678"));
}

#[tokio::test]
async fn test_perp_dex_schema_input_serialization() {
    let schema = PerpDexSchemaInput::new(
        "BTC/USD Perpetual".to_string(),
        0,
        Some("0x1234567890abcdef1234567890abcdef12345678".to_string()),
    );

    let json = serde_json::to_string(&schema).unwrap();
    let expected = r#"{"fullName":"BTC/USD Perpetual","collateralToken":0,"oracleUpdater":"0x1234567890abcdef1234567890abcdef12345678"}"#;
    assert_eq!(json, expected);

    // Test deserialization
    let deserialized: PerpDexSchemaInput = serde_json::from_str(expected).unwrap();
    assert_eq!(deserialized, schema);
}

#[tokio::test]
async fn test_perp_dex_schema_input_serialization_without_oracle() {
    let schema = PerpDexSchemaInput::new_without_oracle(
        "BTC/USD Perpetual".to_string(),
        0,
    );

    let json = serde_json::to_string(&schema).unwrap();
    // Oracle updater should be omitted due to skip_serializing_if
    let expected = r#"{"fullName":"BTC/USD Perpetual","collateralToken":0}"#;
    assert_eq!(json, expected);

    // Test deserialization
    let deserialized: PerpDexSchemaInput = serde_json::from_str(expected).unwrap();
    assert_eq!(deserialized, schema);
}

#[tokio::test]
async fn test_perp_dex_schema_input_with_address_serialization() {
    let address = Address::from_str("0x1234567890abcdef1234567890abcdef12345678").unwrap();
    let schema = PerpDexSchemaInputWithAddress::new(
        "BTC/USD Perpetual".to_string(),
        0,
        Some(address),
    );

    let json = serde_json::to_string(&schema).unwrap();
    let expected = r#"{"fullName":"BTC/USD Perpetual","collateralToken":0,"oracleUpdater":"0x1234567890abcdef1234567890abcdef12345678"}"#;
    assert_eq!(json, expected);

    // Test deserialization
    let deserialized: PerpDexSchemaInputWithAddress = serde_json::from_str(expected).unwrap();
    assert_eq!(deserialized.full_name, "BTC/USD Perpetual");
    assert_eq!(deserialized.collateral_token, 0);
    assert!(deserialized.has_oracle_updater());
}

#[tokio::test]
async fn test_perp_dex_schema_input_with_realistic_data() {
    // Test with realistic perpetual deployment data
    let schema = PerpDexSchemaInput::new(
        "Ethereum USD Perpetual".to_string(),
        1, // USDC
        Some("0x0000000000000000000000000000000000000345".to_string()),
    );

    assert_eq!(schema.full_name, "Ethereum USD Perpetual");
    assert_eq!(schema.collateral_token, 1);
    assert!(schema.has_oracle_updater());

    // Test serialization
    let json = serde_json::to_string(&schema).unwrap();
    assert!(json.contains("Ethereum USD Perpetual"));
    assert!(json.contains("1"));
    assert!(json.contains("0x0000000000000000000000000000000000000345"));
}

#[tokio::test]
async fn test_perp_dex_schema_input_edge_cases() {
    // Test with empty name
    let schema = PerpDexSchemaInput::new(
        "".to_string(),
        0,
        None,
    );
    assert_eq!(schema.full_name, "");

    // Test with maximum collateral token ID
    let schema = PerpDexSchemaInput::new(
        "Test".to_string(),
        i32::MAX,
        None,
    );
    assert_eq!(schema.collateral_token, i32::MAX);

    // Test serialization with empty fields
    let json = serde_json::to_string(&schema).unwrap();
    assert!(json.contains("Test"));
    assert!(json.contains(&i32::MAX.to_string()));
}