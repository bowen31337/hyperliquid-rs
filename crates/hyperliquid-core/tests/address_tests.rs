#[test]
fn test_address_validation() {
    // Test valid addresses with 0x prefix
    let addr1 = Address::from_str("0x1234567890123456789012345678901234567890");
    assert!(addr1.is_ok(), "Should accept valid address with 0x prefix");

    let addr1 = addr1.unwrap();
    assert_eq!(addr1.to_hex(), "0x1234567890123456789012345678901234567890");
    assert_eq!(addr1.to_hex_str(), "1234567890123456789012345678901234567890");

    // Test valid addresses without 0x prefix
    let addr2 = Address::from_str("1234567890123456789012345678901234567890");
    assert!(addr2.is_ok(), "Should accept valid address without 0x prefix");

    let addr2 = addr2.unwrap();
    assert_eq!(addr2.to_hex(), "0x1234567890123456789012345678901234567890");
    assert_eq!(addr1, addr2, "Addresses should be equal regardless of prefix in input");

    // Test uppercase hex
    let addr3 = Address::from_str("0xABCDEF1234567890ABCDEF1234567890ABCDEF12");
    assert!(addr3.is_ok(), "Should accept uppercase hex");

    // Test invalid prefix
    let addr_invalid_prefix = Address::from_str("00x1234567890123456789012345678901234567890");
    assert!(addr_invalid_prefix.is_err(), "Should reject invalid prefix");
    assert!(addr_invalid_prefix.unwrap_err().contains("Invalid address length"));

    // Test wrong length - too short
    let addr_short = Address::from_str("0x123456789012345678901234567890123456789"); // 39 chars
    assert!(addr_short.is_err(), "Should reject too short address");
    assert!(addr_short.unwrap_err().contains("Invalid address length"));

    // Test wrong length - too long
    let addr_long = Address::from_str("0x12345678901234567890123456789012345678900"); // 41 chars
    assert!(addr_long.is_err(), "Should reject too long address");
    assert!(addr_long.unwrap_err().contains("Invalid address length"));

    // Test non-hex characters
    let addr_non_hex1 = Address::from_str("0x123456789012345678901234567890123456789g"); // 'g' is not hex
    assert!(addr_non_hex1.is_err(), "Should reject non-hex character 'g'");
    assert!(addr_non_hex1.unwrap_err().contains("non-hexadecimal"));

    let addr_non_hex2 = Address::from_str("0x123456789012345678901234567890123456789Z"); // 'Z' is not hex
    assert!(addr_non_hex2.is_err(), "Should reject non-hex character 'Z'");
    assert!(addr_non_hex2.unwrap_err().contains("non-hexadecimal"));

    // Test empty string
    let addr_empty = Address::from_str("");
    assert!(addr_empty.is_err(), "Should reject empty string");

    // Test just 0x
    let addr_just_prefix = Address::from_str("0x");
    assert!(addr_just_prefix.is_err(), "Should reject just 0x prefix");
    assert!(addr_just_prefix.unwrap_err().contains("Invalid address length"));

    // Test validate_address helper function
    assert!(validate_address("0x1234567890123456789012345678901234567890").is_ok());
    assert!(validate_address("1234567890123456789012345678901234567890").is_ok());
    assert!(validate_address("0x123456789012345678901234567890123456789g").is_err());

    println!("✅ Address validation test passed");
}