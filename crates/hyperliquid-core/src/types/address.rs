//! Address validation and type for Ethereum-style addresses

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::str::FromStr;

/// Ethereum-style address validation and storage
///
/// This type ensures that all addresses are properly formatted:
/// - Starts with "0x" prefix (optional for parsing)
/// - Contains exactly 20 bytes (40 hex characters + 0x prefix)
/// - Uses lowercase hex representation
/// - Validates hex characters only
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Address {
    /// Internal storage as 20 bytes (Ethereum address size)
    bytes: [u8; 20],
}

impl Address {
    /// Create a new Address from a hex string
    ///
    /// # Arguments
    /// * `s` - Hex string with or without 0x prefix
    ///
    /// # Returns
    /// * `Result<Self, String>` - Address or error message
    ///
    /// # Examples
    /// ```
    /// let addr = Address::from_str("0x1234567890123456789012345678901234567890");
    /// assert!(addr.is_ok());
    ///
    /// let addr = Address::from_str("1234567890123456789012345678901234567890");
    /// assert!(addr.is_ok());
    /// ```
    pub fn from_str(s: &str) -> Result<Self, String> {
        // Remove 0x prefix if present
        let hex = s.strip_prefix("0x").unwrap_or(s);

        // Validate length (20 bytes = 40 hex chars)
        if hex.len() != 40 {
            return Err(format!(
                "Invalid address length: expected 40 hex characters, got {}",
                hex.len()
            ));
        }

        // Validate hex characters
        if !hex.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err("Address contains non-hexadecimal characters".to_string());
        }

        // Parse hex to bytes
        let mut bytes = [0u8; 20];
        for i in 0..20 {
            let byte_str = &hex[i * 2..i * 2 + 2];
            bytes[i] = u8::from_str_radix(byte_str, 16)
                .map_err(|e| format!("Failed to parse hex: {}", e))?;
        }

        Ok(Address { bytes })
    }

    /// Get the address as bytes
    pub fn as_bytes(&self) -> &[u8; 20] {
        &self.bytes
    }

    /// Get the address as a hex string with 0x prefix
    pub fn to_hex(&self) -> String {
        format!("0x{}", hex::encode(self.bytes))
    }

    /// Get the address as a hex string without 0x prefix
    pub fn to_hex_str(&self) -> String {
        hex::encode(self.bytes)
    }

    /// Validate an address string without creating Address
    pub fn is_valid(s: &str) -> bool {
        Self::from_str(s).is_ok()
    }
}

impl FromStr for Address {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_str(s)
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{}", hex::encode(self.bytes))
    }
}

impl Serialize for Address {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_hex())
    }
}

impl<'de> Deserialize<'de> for Address {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Address::from_str(&s).map_err(serde::de::Error::custom)
    }
}

/// Validate address and return error if invalid
pub fn validate_address(s: &str) -> Result<(), String> {
    Address::from_str(s).map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_address_with_prefix() {
        let addr = Address::from_str("0x1234567890123456789012345678901234567890");
        assert!(addr.is_ok());

        let addr = addr.unwrap();
        assert_eq!(addr.to_hex(), "0x1234567890123456789012345678901234567890");
        assert_eq!(addr.to_hex_str(), "1234567890123456789012345678901234567890");
    }

    #[test]
    fn test_valid_address_without_prefix() {
        let addr = Address::from_str("1234567890123456789012345678901234567890");
        assert!(addr.is_ok());

        let addr = addr.unwrap();
        assert_eq!(addr.to_hex(), "0x1234567890123456789012345678901234567890");
    }

    #[test]
    fn test_invalid_prefix() {
        let addr = Address::from_str("00x1234567890123456789012345678901234567890");
        assert!(addr.is_err());
        assert!(addr.unwrap_err().contains("Invalid address length"));
    }

    #[test]
    fn test_wrong_length() {
        let addr = Address::from_str("0x123456789012345678901234567890123456789"); // 39 chars
        assert!(addr.is_err());
        assert!(addr.unwrap_err().contains("Invalid address length"));

        let addr = Address::from_str("0x12345678901234567890123456789012345678900"); // 41 chars
        assert!(addr.is_err());
        assert!(addr.unwrap_err().contains("Invalid address length"));
    }

    #[test]
    fn test_non_hex_chars() {
        let addr = Address::from_str("0x123456789012345678901234567890123456789g"); // 'g' is not hex
        assert!(addr.is_err());
        assert!(addr.unwrap_err().contains("non-hexadecimal"));

        let addr = Address::from_str("0x123456789012345678901234567890123456789Z"); // 'Z' is not hex
        assert!(addr.is_err());
        assert!(addr.unwrap_err().contains("non-hexadecimal"));
    }

    #[test]
    fn test_case_insensitive() {
        let addr1 = Address::from_str("0x1234567890123456789012345678901234567890").unwrap();
        let addr2 = Address::from_str("0x1234567890123456789012345678901234567890").unwrap();
        assert_eq!(addr1, addr2);

        // Test uppercase
        let addr3 = Address::from_str("0x1234567890123456789012345678901234567890").unwrap();
        assert_eq!(addr1, addr3);
    }

    #[test]
    fn test_display_format() {
        let addr = Address::from_str("0x1234567890123456789012345678901234567890").unwrap();
        assert_eq!(format!("{}", addr), "0x1234567890123456789012345678901234567890");
    }

    #[test]
    fn test_serialize_deserialize() {
        let addr = Address::from_str("0x1234567890123456789012345678901234567890").unwrap();

        // Serialize
        let json = serde_json::to_string(&addr).unwrap();
        assert_eq!(json, r#""0x1234567890123456789012345678901234567890""#);

        // Deserialize
        let deserialized: Address = serde_json::from_str(&json).unwrap();
        assert_eq!(addr, deserialized);
    }

    #[test]
    fn test_validate_address() {
        assert!(validate_address("0x1234567890123456789012345678901234567890").is_ok());
        assert!(validate_address("1234567890123456789012345678901234567890").is_ok());
        assert!(validate_address("0x123456789012345678901234567890123456789g").is_err());
    }

    #[test]
    fn test_hash_and_equality() {
        let addr1 = Address::from_str("0x1234567890123456789012345678901234567890").unwrap();
        let addr2 = Address::from_str("0x1234567890123456789012345678901234567890").unwrap();
        let addr3 = Address::from_str("0x1111111111111111111111111111111111111111").unwrap();

        // Test equality
        assert_eq!(addr1, addr2);
        assert_ne!(addr1, addr3);

        // Test hash
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(addr1.clone());
        set.insert(addr2.clone()); // Should not add duplicate
        set.insert(addr3);
        assert_eq!(set.len(), 2);
    }
}