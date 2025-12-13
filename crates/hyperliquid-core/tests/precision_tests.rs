//! Tests for precision handling

use hyperliquid_core::types::precision::*;

#[tokio::test]
async fn test_precision_basic_functionality() {
    // Test basic float_to_wire functionality
    assert_eq!(float_to_wire(50000.12345678).unwrap(), "50000.12345678");
    assert_eq!(float_to_wire(0.00000001).unwrap(), "0.00000001");
    assert_eq!(float_to_wire(100.0).unwrap(), "100");
    assert_eq!(float_to_wire(0.0).unwrap(), "0");
    assert_eq!(float_to_wire(1234.56789).unwrap(), "1234.56789");

    // Test negative zero handling
    assert_eq!(float_to_wire(-0.0).unwrap(), "0");
}

#[tokio::test]
async fn test_precision_rounding_errors() {
    // Test that values that would cause rounding are rejected
    assert!(float_to_wire(0.123456789).is_err()); // 9 decimal places
    assert!(float_to_wire(0.000000001).is_err()); // Too small precision
}

#[tokio::test]
async fn test_precision_int_conversion() {
    // Test float_to_int with different decimal places
    assert_eq!(float_to_int(1000.12345678, 8).unwrap(), 100012345678);
    assert_eq!(float_to_int(100.123456, 6).unwrap(), 100123456);

    // Test specific USD conversion
    assert_eq!(float_to_usd_int(100.123456).unwrap(), 100123456);

    // Test hashing conversion
    assert_eq!(float_to_int_for_hashing(1000.12345678).unwrap(), 100012345678);
}

#[tokio::test]
async fn test_precision_validation() {
    // Test validation functions
    assert!(can_convert_without_precision_loss(0.12345678, 8));
    assert!(!can_convert_without_precision_loss(0.123456789, 8));

    assert!(validate_price_precision(50000.12345678));
    assert!(!validate_price_precision(0.123456789));

    assert!(validate_quantity_precision(1.23456789));
    assert!(!validate_quantity_precision(1.234567891));

    assert!(validate_usd_precision(100.123456));
    assert!(!validate_usd_precision(100.1234567));
}

#[tokio::test]
async fn test_precision_edge_cases() {
    // Test edge cases
    assert_eq!(float_to_wire(1e8).unwrap(), "100000000");
    assert_eq!(float_to_wire(1e-8).unwrap(), "0.00000001");

    // Test very small values
    assert!(float_to_wire(1e-9).is_err()); // Too small for 8 decimal places
}

#[tokio::test]
async fn test_precision_integration_with_order_builder() {
    // Test the OrderWireBuilder with precision validation
    use hyperliquid_core::types::precision::OrderWireBuilder;

    // Valid order creation
    let order = OrderWireBuilder::new("BTC")
        .buy()
        .size(1.5).unwrap()
        .limit_price(50000.0).unwrap()
        .build()
        .unwrap();

    assert_eq!(order.coin, "BTC");
    assert_eq!(order.sz, "1.5");
    assert_eq!(order.limit_price, "50000");
    assert!(order.is_buy);
    assert_eq!(order.order_type, hyperliquid_core::types::OrderType::Limit);

    // Test with reduce only
    let order = OrderWireBuilder::new("ETH")
        .sell()
        .size(0.5).unwrap()
        .limit_price(3000.0).unwrap()
        .reduce_only(true)
        .build()
        .unwrap();

    assert_eq!(order.coin, "ETH");
    assert_eq!(order.sz, "0.5");
    assert_eq!(order.limit_price, "3000");
    assert!(!order.is_buy);
    assert!(order.reduce_only);

    // Test pegged order
    let order = OrderWireBuilder::new("SOL")
        .buy()
        .size(10.0).unwrap()
        .limit_price(100.0).unwrap()
        .peg_offset(0.01)
        .unwrap()
        .peg_price_type(hyperliquid_core::types::precision::PegPriceType::Mid)
        .build()
        .unwrap();

    assert_eq!(order.coin, "SOL");
    assert_eq!(order.peg_offset_value, Some("0.01".to_string()));
    assert_eq!(order.peg_price_type, Some(hyperliquid_core::types::precision::PegPriceType::Mid));
}

#[tokio::test]
async fn test_precision_order_builder_errors() {
    use hyperliquid_core::types::precision::OrderWireBuilder;

    // Test invalid precision for size
    let result = OrderWireBuilder::new("BTC")
        .buy()
        .size(1.234567891) // Invalid precision (9 decimal places)
        .limit_price(50000.0)
        .build();

    assert!(result.is_err());

    // Test invalid precision for price
    let result = OrderWireBuilder::new("ETH")
        .sell()
        .size(1.0)
        .limit_price(3000.123456789) // Invalid precision
        .build();

    assert!(result.is_err());
}

#[tokio::test]
async fn test_precision_compatibility_with_python_sdk() {
    // Test that our precision handling matches the Python SDK behavior
    // These are the exact values from the Python SDK tests

    // Test cases from Python SDK
    let test_cases = vec![
        (50000.12345678, "50000.12345678"),
        (0.00000001, "0.00000001"),
        (100.0, "100"),
        (0.0, "0"),
        (-0.0, "0"),
        (1.0, "1"),
        (1000.0, "1000"),
    ];

    for (input, expected) in test_cases {
        let result = float_to_wire(input).unwrap();
        assert_eq!(result, expected, "Failed for input: {}", input);
    }

    // Test that values requiring rounding are rejected
    let invalid_cases = vec![
        0.123456789,  // 9 decimal places
        0.000000001,  // Too small
        1.234567891,  // 10 decimal places
    ];

    for case in invalid_cases {
        assert!(float_to_wire(case).is_err(), "Should reject rounding for: {}", case);
    }
}

#[tokio::test]
async fn test_precision_error_types() {
    // Test that we get the right error types
    let result = float_to_wire(0.123456789);
    assert!(result.is_err());

    if let Err(error) = result {
        match error {
            PrecisionError::RoundingError { value } => {
                assert_eq!(value, 0.123456789);
            }
            _ => panic!("Expected RoundingError, got: {:?}", error),
        }
    }
}