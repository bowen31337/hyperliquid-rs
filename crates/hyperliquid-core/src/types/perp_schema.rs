//! Perpetual deployment schema types.
//!
//! This module provides types for perpetual deployment schemas, which are used
//! when deploying new perpetual contracts on the Hyperliquid platform. These
//! schemas define the configuration and metadata for perpetual contracts.

use serde::{Deserialize, Serialize};
use crate::types::Address;

/// Schema input for perpetual deployment
///
/// This type defines the configuration for deploying new perpetual contracts.
/// It includes the full name, collateral token, and optional oracle updater.
///
/// # Examples
/// ```
/// use hyperliquid_core::types::perp_schema::PerpDexSchemaInput;
///
/// let schema = PerpDexSchemaInput {
///     full_name: "BTC/USD Perpetual".to_string(),
///     collateral_token: 0, // USDC
///     oracle_updater: Some("0x1234567890abcdef1234567890abcdef12345678".to_string()),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PerpDexSchemaInput {
    /// Full name of the perpetual contract
    #[serde(rename = "fullName")]
    pub full_name: String,

    /// Collateral token ID
    #[serde(rename = "collateralToken")]
    pub collateral_token: i32,

    /// Optional oracle updater address
    #[serde(rename = "oracleUpdater", skip_serializing_if = "Option::is_none")]
    pub oracle_updater: Option<String>,
}

/// Schema input with address type for oracle updater
///
/// This is an alternative version that uses the Address type for better
/// type safety when working with Ethereum addresses.
///
/// # Examples
/// ```
/// use hyperliquid_core::types::{perp_schema::PerpDexSchemaInputWithAddress, Address};
///
/// let address = Address::from_str("0x1234567890abcdef1234567890abcdef12345678").unwrap();
/// let schema = PerpDexSchemaInputWithAddress {
///     full_name: "BTC/USD Perpetual".to_string(),
///     collateral_token: 0, // USDC
///     oracle_updater: Some(address),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PerpDexSchemaInputWithAddress {
    /// Full name of the perpetual contract
    #[serde(rename = "fullName")]
    pub full_name: String,

    /// Collateral token ID
    #[serde(rename = "collateralToken")]
    pub collateral_token: i32,

    /// Optional oracle updater address
    #[serde(rename = "oracleUpdater", skip_serializing_if = "Option::is_none")]
    pub oracle_updater: Option<Address>,
}

/// Builder for PerpDexSchemaInput
///
/// This provides a fluent interface for creating PerpDexSchemaInput instances.
///
/// # Examples
/// ```
/// use hyperliquid_core::types::perp_schema::PerpDexSchemaInputBuilder;
///
/// let schema = PerpDexSchemaInputBuilder::new("BTC/USD Perpetual", 0)
///     .oracle_updater("0x1234567890abcdef1234567890abcdef12345678")
///     .build();
///
/// assert_eq!(schema.full_name, "BTC/USD Perpetual");
/// assert_eq!(schema.collateral_token, 0);
/// assert!(schema.oracle_updater.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct PerpDexSchemaInputBuilder {
    full_name: String,
    collateral_token: i32,
    oracle_updater: Option<String>,
}

impl PerpDexSchemaInputBuilder {
    /// Create a new schema input builder
    ///
    /// # Arguments
    /// * `full_name` - Full name of the perpetual contract
    /// * `collateral_token` - Collateral token ID
    ///
    /// # Returns
    /// A new builder instance
    ///
    /// # Examples
    /// ```
    /// use hyperliquid_core::types::perp_schema::PerpDexSchemaInputBuilder;
    ///
    /// let builder = PerpDexSchemaInputBuilder::new("BTC/USD Perpetual", 0);
    /// ```
    pub fn new(full_name: impl Into<String>, collateral_token: i32) -> Self {
        Self {
            full_name: full_name.into(),
            collateral_token,
            oracle_updater: None,
        }
    }

    /// Set the oracle updater address
    ///
    /// # Arguments
    /// * `oracle_updater` - Oracle updater address
    ///
    /// # Returns
    /// Self for method chaining
    ///
    /// # Examples
    /// ```
    /// use hyperliquid_core::types::perp_schema::PerpDexSchemaInputBuilder;
    ///
    /// let schema = PerpDexSchemaInputBuilder::new("BTC/USD Perpetual", 0)
    ///     .oracle_updater("0x1234567890abcdef1234567890abcdef12345678")
    ///     .build();
    /// ```
    pub fn oracle_updater(mut self, oracle_updater: impl Into<String>) -> Self {
        self.oracle_updater = Some(oracle_updater.into());
        self
    }

    /// Set the oracle updater address from Address type
    ///
    /// # Arguments
    /// * `oracle_updater` - Oracle updater address
    ///
    /// # Returns
    /// Self for method chaining
    ///
    /// # Examples
    /// ```
    /// use hyperliquid_core::types::{perp_schema::PerpDexSchemaInputBuilder, Address};
    ///
    /// let address = Address::from_str("0x1234567890abcdef1234567890abcdef12345678").unwrap();
    /// let schema = PerpDexSchemaInputBuilder::new("BTC/USD Perpetual", 0)
    ///     .oracle_updater_from_address(address)
    ///     .build();
    /// ```
    pub fn oracle_updater_from_address(mut self, oracle_updater: Address) -> Self {
        self.oracle_updater = Some(oracle_updater.to_string());
        self
    }

    /// Build the PerpDexSchemaInput
    ///
    /// # Returns
    /// A new PerpDexSchemaInput instance
    ///
    /// # Examples
    /// ```
    /// use hyperliquid_core::types::perp_schema::PerpDexSchemaInputBuilder;
    ///
    /// let schema = PerpDexSchemaInputBuilder::new("BTC/USD Perpetual", 0)
    ///     .oracle_updater("0x1234567890abcdef1234567890abcdef12345678")
    ///     .build();
    /// ```
    pub fn build(self) -> PerpDexSchemaInput {
        PerpDexSchemaInput {
            full_name: self.full_name,
            collateral_token: self.collateral_token,
            oracle_updater: self.oracle_updater,
        }
    }
}

impl PerpDexSchemaInput {
    /// Create a new PerpDexSchemaInput
    ///
    /// # Arguments
    /// * `full_name` - Full name of the perpetual contract
    /// * `collateral_token` - Collateral token ID
    /// * `oracle_updater` - Optional oracle updater address
    ///
    /// # Returns
    /// A new PerpDexSchemaInput instance
    ///
    /// # Examples
    /// ```
    /// use hyperliquid_core::types::perp_schema::PerpDexSchemaInput;
    ///
    /// let schema = PerpDexSchemaInput::new(
    ///     "BTC/USD Perpetual".to_string(),
    ///     0,
    ///     Some("0x1234567890abcdef1234567890abcdef12345678".to_string()),
    /// );
    /// ```
    pub fn new(
        full_name: String,
        collateral_token: i32,
        oracle_updater: Option<String>,
    ) -> Self {
        Self {
            full_name,
            collateral_token,
            oracle_updater,
        }
    }

    /// Create a new PerpDexSchemaInput without oracle updater
    ///
    /// # Arguments
    /// * `full_name` - Full name of the perpetual contract
    /// * `collateral_token` - Collateral token ID
    ///
    /// # Returns
    /// A new PerpDexSchemaInput instance without oracle updater
    ///
    /// # Examples
    /// ```
    /// use hyperliquid_core::types::perp_schema::PerpDexSchemaInput;
    ///
    /// let schema = PerpDexSchemaInput::new_without_oracle(
    ///     "BTC/USD Perpetual".to_string(),
    ///     0,
    /// );
    /// assert!(schema.oracle_updater.is_none());
    /// ```
    pub fn new_without_oracle(full_name: String, collateral_token: i32) -> Self {
        Self {
            full_name,
            collateral_token,
            oracle_updater: None,
        }
    }

    /// Check if the schema has an oracle updater
    ///
    /// # Returns
    /// * `true` - If oracle updater is set
    /// * `false` - If oracle updater is not set
    ///
    /// # Examples
    /// ```
    /// use hyperliquid_core::types::perp_schema::PerpDexSchemaInput;
    ///
    /// let schema = PerpDexSchemaInput::new_without_oracle(
    ///     "BTC/USD Perpetual".to_string(),
    ///     0,
    /// );
    /// assert!(!schema.has_oracle_updater());
    /// ```
    pub fn has_oracle_updater(&self) -> bool {
        self.oracle_updater.is_some()
    }

    /// Get the oracle updater address if available
    ///
    /// # Returns
    /// * `Some(String)` - Oracle updater address if available
    /// * `None` - If oracle updater is not set
    ///
    /// # Examples
    /// ```
    /// use hyperliquid_core::types::perp_schema::PerpDexSchemaInput;
    ///
    /// let schema = PerpDexSchemaInput::new(
    ///     "BTC/USD Perpetual".to_string(),
    ///     0,
    ///     Some("0x1234567890abcdef1234567890abcdef12345678".to_string()),
    /// );
    /// assert_eq!(schema.get_oracle_updater(), Some("0x1234567890abcdef1234567890abcdef12345678"));
    /// ```
    pub fn get_oracle_updater(&self) -> Option<&str> {
        self.oracle_updater.as_deref()
    }
}

impl PerpDexSchemaInputWithAddress {
    /// Create a new PerpDexSchemaInputWithAddress
    ///
    /// # Arguments
    /// * `full_name` - Full name of the perpetual contract
    /// * `collateral_token` - Collateral token ID
    /// * `oracle_updater` - Optional oracle updater address
    ///
    /// # Returns
    /// A new PerpDexSchemaInputWithAddress instance
    ///
    /// # Examples
    /// ```
    /// use hyperliquid_core::types::{perp_schema::PerpDexSchemaInputWithAddress, Address};
    ///
    /// let address = Address::from_str("0x1234567890abcdef1234567890abcdef12345678").unwrap();
    /// let schema = PerpDexSchemaInputWithAddress::new(
    ///     "BTC/USD Perpetual".to_string(),
    ///     0,
    ///     Some(address),
    /// );
    /// ```
    pub fn new(
        full_name: String,
        collateral_token: i32,
        oracle_updater: Option<Address>,
    ) -> Self {
        Self {
            full_name,
            collateral_token,
            oracle_updater,
        }
    }

    /// Create a new PerpDexSchemaInputWithAddress without oracle updater
    ///
    /// # Arguments
    /// * `full_name` - Full name of the perpetual contract
    /// * `collateral_token` - Collateral token ID
    ///
    /// # Returns
    /// A new PerpDexSchemaInputWithAddress instance without oracle updater
    ///
    /// # Examples
    /// ```
    /// use hyperliquid_core::types::perp_schema::PerpDexSchemaInputWithAddress;
    ///
    /// let schema = PerpDexSchemaInputWithAddress::new_without_oracle(
    ///     "BTC/USD Perpetual".to_string(),
    ///     0,
    /// );
    /// assert!(schema.oracle_updater.is_none());
    /// ```
    pub fn new_without_oracle(full_name: String, collateral_token: i32) -> Self {
        Self {
            full_name,
            collateral_token,
            oracle_updater: None,
        }
    }

    /// Check if the schema has an oracle updater
    ///
    /// # Returns
    /// * `true` - If oracle updater is set
    /// * `false` - If oracle updater is not set
    ///
    /// # Examples
    /// ```
    /// use hyperliquid_core::types::perp_schema::PerpDexSchemaInputWithAddress;
    ///
    /// let schema = PerpDexSchemaInputWithAddress::new_without_oracle(
    ///     "BTC/USD Perpetual".to_string(),
    ///     0,
    /// );
    /// assert!(!schema.has_oracle_updater());
    /// ```
    pub fn has_oracle_updater(&self) -> bool {
        self.oracle_updater.is_some()
    }

    /// Get the oracle updater address if available
    ///
    /// # Returns
    /// * `Some(&Address)` - Oracle updater address if available
    /// * `None` - If oracle updater is not set
    ///
    /// # Examples
    /// ```
    /// use hyperliquid_core::types::{perp_schema::PerpDexSchemaInputWithAddress, Address};
    ///
    /// let address = Address::from_str("0x1234567890abcdef1234567890abcdef12345678").unwrap();
    /// let schema = PerpDexSchemaInputWithAddress::new(
    ///     "BTC/USD Perpetual".to_string(),
    ///     0,
    ///     Some(address.clone()),
    /// );
    /// assert_eq!(schema.get_oracle_updater().map(|a| a.as_str()), Some("0x1234567890abcdef1234567890abcdef12345678"));
    /// ```
    pub fn get_oracle_updater(&self) -> Option<&Address> {
        self.oracle_updater.as_ref()
    }

    /// Convert to PerpDexSchemaInput (with String oracle updater)
    ///
    /// # Returns
    /// A new PerpDexSchemaInput instance
    ///
    /// # Examples
    /// ```
    /// use hyperliquid_core::types::{perp_schema::PerpDexSchemaInputWithAddress, Address};
    ///
    /// let address = Address::from_str("0x1234567890abcdef1234567890abcdef12345678").unwrap();
    /// let schema_with_address = PerpDexSchemaInputWithAddress::new(
    ///     "BTC/USD Perpetual".to_string(),
    ///     0,
    ///     Some(address),
    /// );
    /// let schema = schema_with_address.to_string_schema();
    /// assert_eq!(schema.get_oracle_updater(), Some("0x1234567890abcdef1234567890abcdef12345678"));
    /// ```
    pub fn to_string_schema(&self) -> PerpDexSchemaInput {
        PerpDexSchemaInput {
            full_name: self.full_name.clone(),
            collateral_token: self.collateral_token,
            oracle_updater: self.oracle_updater.as_ref().map(|addr| addr.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Address;

    #[test]
    fn test_perp_dex_schema_input_basic() {
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

    #[test]
    fn test_perp_dex_schema_input_without_oracle() {
        let schema = PerpDexSchemaInput::new_without_oracle(
            "BTC/USD Perpetual".to_string(),
            0,
        );

        assert_eq!(schema.full_name, "BTC/USD Perpetual");
        assert_eq!(schema.collateral_token, 0);
        assert!(schema.get_oracle_updater().is_none());
        assert!(!schema.has_oracle_updater());
    }

    #[test]
    fn test_perp_dex_schema_input_builder() {
        let schema = PerpDexSchemaInputBuilder::new("BTC/USD Perpetual", 0)
            .oracle_updater("0x1234567890abcdef1234567890abcdef12345678")
            .build();

        assert_eq!(schema.full_name, "BTC/USD Perpetual");
        assert_eq!(schema.collateral_token, 0);
        assert_eq!(schema.get_oracle_updater(), Some("0x1234567890abcdef1234567890abcdef12345678"));
    }

    #[test]
    fn test_perp_dex_schema_input_with_address() {
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

    #[test]
    fn test_perp_dex_schema_input_serialization() {
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

    #[test]
    fn test_perp_dex_schema_input_serialization_without_oracle() {
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

    #[test]
    fn test_perp_dex_schema_input_with_address_serialization() {
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
}