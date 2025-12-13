//! Precision handling for prices, quantities, and other numeric values.
//!
//! This module provides functions to convert floating point numbers to strings
//! with proper precision handling, ensuring no rounding errors occur during
//! conversion. This is critical for financial operations where precision matters.
//!
//! Based on the Python SDK's `float_to_wire` function:
//! - Prices and quantities are rounded to 8 decimal places
//! - USD values are rounded to 6 decimal places for hashing
//! - The conversion validates that no precision is lost
//! - Results are normalized to remove trailing zeros

use std::str::FromStr;
use rust_decimal::Decimal;
use thiserror::Error;

/// Precision handling errors
#[derive(Error, Debug, Clone, PartialEq)]
pub enum PrecisionError {
    #[error("Precision conversion would cause rounding: {value}")]
    RoundingError { value: f64 },
    #[error("Invalid decimal conversion: {source}")]
    DecimalError { source: rust_decimal::Error },
    #[error("Overflow during integer conversion: {value}")]
    OverflowError { value: f64 },
}

/// Convert a float to wire format string with precision handling
///
/// This function:
/// 1. Rounds to 8 decimal places (standard for prices and quantities)
/// 2. Validates that no precision loss occurred
/// 3. Normalizes the decimal to remove trailing zeros
/// 4. Returns as a string
///
/// # Arguments
/// * `value` - The floating point value to convert
///
/// # Returns
/// * `Ok(String)` - The converted string if no precision loss
/// * `Err(PrecisionError)` - If conversion would cause rounding
///
/// # Examples
/// ```
/// use hyperliquid_core::types::precision::float_to_wire;
///
/// let price = float_to_wire(50000.12345678).unwrap();
/// assert_eq!(price, "50000.12345678");
///
/// let small_price = float_to_wire(0.00000001).unwrap();
/// assert_eq!(small_price, "0.00000001");
/// ```
pub fn float_to_wire(value: f64) -> Result<String, PrecisionError> {
    // Round to 8 decimal places
    let rounded = (value * 1e8).round() / 1e8;

    // Check if rounding caused precision loss
    if (rounded - value).abs() >= 1e-12 {
        return Err(PrecisionError::RoundingError { value });
    }

    // Handle negative zero
    let rounded = if rounded == -0.0 { 0.0 } else { rounded };

    // Convert to decimal for normalization
    let decimal = Decimal::from_f64(rounded)
        .ok_or_else(|| PrecisionError::RoundingError { value })?;

    // Normalize and convert to string
    let normalized = decimal.normalize();
    Ok(normalized.to_string())
}

/// Convert a float to integer for hashing with specified decimal places
///
/// # Arguments
/// * `value` - The floating point value to convert
/// * `decimal_places` - Number of decimal places to preserve
///
/// # Returns
/// * `Ok(i64)` - The converted integer
/// * `Err(PrecisionError)` - If conversion would cause rounding or overflow
///
/// # Examples
/// ```
/// use hyperliquid_core::types::precision::float_to_int;
///
/// // 8 decimal places for standard hashing
/// let result = float_to_int(1000.12345678, 8).unwrap();
/// assert_eq!(result, 100012345678);
///
/// // 6 decimal places for USD values
/// let usd_result = float_to_int(100.123456, 6).unwrap();
/// assert_eq!(usd_result, 100123456);
/// ```
pub fn float_to_int(value: f64, decimal_places: u32) -> Result<i64, PrecisionError> {
    let power = 10_f64.powi(decimal_places as i32);
    let with_decimals = value * power;

    // Check if rounding would cause precision loss
    if (with_decimals.round() - with_decimals).abs() >= 1e-3 {
        return Err(PrecisionError::RoundingError { value });
    }

    let result = with_decimals.round() as i64;

    // Verify no overflow occurred
    if (result as f64 / power - value).abs() >= 1e-3 {
        return Err(PrecisionError::OverflowError { value });
    }

    Ok(result)
}

/// Convert a float to integer for USD hashing (6 decimal places)
///
/// # Arguments
/// * `value` - The USD value to convert
///
/// # Returns
/// * `Ok(i64)` - The converted integer
/// * `Err(PrecisionError)` - If conversion would cause rounding or overflow
///
/// # Examples
/// ```
/// use hyperliquid_core::types::precision::float_to_usd_int;
///
/// let result = float_to_usd_int(100.123456).unwrap();
/// assert_eq!(result, 100123456);
/// ```
pub fn float_to_usd_int(value: f64) -> Result<i64, PrecisionError> {
    float_to_int(value, 6)
}

/// Convert a float to integer for standard hashing (8 decimal places)
///
/// # Arguments
/// * `value` - The value to convert
///
/// # Returns
/// * `Ok(i64)` - The converted integer
/// * `Err(PrecisionError)` - If conversion would cause rounding or overflow
///
/// # Examples
/// ```
/// use hyperliquid_core::types::precision::float_to_int_for_hashing;
///
/// let result = float_to_int_for_hashing(1000.12345678).unwrap();
/// assert_eq!(result, 100012345678);
/// ```
pub fn float_to_int_for_hashing(value: f64) -> Result<i64, PrecisionError> {
    float_to_int(value, 8)
}

/// Validate that a float can be converted without precision loss
///
/// # Arguments
/// * `value` - The value to validate
/// * `decimal_places` - Number of decimal places to check
///
/// # Returns
/// * `true` - If conversion would not cause precision loss
/// * `false` - If conversion would cause precision loss
///
/// # Examples
/// ```
/// use hyperliquid_core::types::precision::can_convert_without_precision_loss;
///
/// assert!(can_convert_without_precision_loss(0.12345678, 8));
/// assert!(!can_convert_without_precision_loss(0.123456789, 8));
/// ```
pub fn can_convert_without_precision_loss(value: f64, decimal_places: u32) -> bool {
    let power = 10_f64.powi(decimal_places as i32);
    let rounded = (value * power).round() / power;
    (rounded - value).abs() < 1e-12
}

/// Validate price precision (8 decimal places)
///
/// # Arguments
/// * `price` - The price to validate
///
/// # Returns
/// * `true` - If price has valid precision
/// * `false` - If price would cause precision loss
///
/// # Examples
/// ```
/// use hyperliquid_core::types::precision::validate_price_precision;
///
/// assert!(validate_price_precision(50000.12345678));
/// assert!(!validate_price_precision(0.123456789));
/// ```
pub fn validate_price_precision(price: f64) -> bool {
    can_convert_without_precision_loss(price, 8)
}

/// Validate quantity precision (8 decimal places)
///
/// # Arguments
/// * `quantity` - The quantity to validate
///
/// # Returns
/// * `true` - If quantity has valid precision
/// * `false` - If quantity would cause precision loss
///
/// # Examples
/// ```
/// use hyperliquid_core::types::precision::validate_quantity_precision;
///
/// assert!(validate_quantity_precision(1.23456789));
/// assert!(!validate_quantity_precision(1.234567891));
/// ```
pub fn validate_quantity_precision(quantity: f64) -> bool {
    can_convert_without_precision_loss(quantity, 8)
}

/// Validate USD precision (6 decimal places)
///
/// # Arguments
/// * `usd_value` - The USD value to validate
///
/// # Returns
/// * `true` - If USD value has valid precision
/// * `false` - If USD value would cause precision loss
///
/// # Examples
/// ```
/// use hyperliquid_core::types::precision::validate_usd_precision;
///
/// assert!(validate_usd_precision(100.123456));
/// assert!(!validate_usd_precision(100.1234567));
/// ```
pub fn validate_usd_precision(usd_value: f64) -> bool {
    can_convert_without_precision_loss(usd_value, 6)
}

/// Builder for OrderWire with precision validation
///
/// This helper struct provides a fluent interface for creating OrderWire
/// structs with automatic precision validation and conversion.
///
/// # Examples
/// ```
/// use hyperliquid_core::types::precision::OrderWireBuilder;
///
/// let order_wire = OrderWireBuilder::new("BTC")
///     .buy()
///     .size(1.5)
///     .limit_price(50000.0)
///     .build()
///     .unwrap();
///
/// assert_eq!(order_wire.coin, "BTC");
/// assert_eq!(order_wire.sz, "1.5");
/// assert_eq!(order_wire.limit_price, "50000.0");
/// assert!(order_wire.is_buy);
/// ```
#[derive(Debug, Clone)]
pub struct OrderWireBuilder {
    coin: String,
    is_buy: bool,
    size: Option<f64>,
    limit_price: Option<f64>,
    reduce_only: bool,
    order_type: Option<OrderType>,
    peg_offset_value: Option<f64>,
    peg_price_type: Option<PegPriceType>,
    is_trigger: bool,
    trigger_condition: Option<TriggerCondition>,
    trigger_px: Option<f64>,
    cloid: Option<String>,
    order_id: Option<i64>,
}

impl OrderWireBuilder {
    /// Create a new OrderWireBuilder
    pub fn new(coin: impl Into<String>) -> Self {
        Self {
            coin: coin.into(),
            is_buy: true,
            size: None,
            limit_price: None,
            reduce_only: false,
            order_type: None,
            peg_offset_value: None,
            peg_price_type: None,
            is_trigger: false,
            trigger_condition: None,
            trigger_px: None,
            cloid: None,
            order_id: None,
        }
    }

    /// Set buy order (default)
    pub fn buy(mut self) -> Self {
        self.is_buy = true;
        self
    }

    /// Set sell order
    pub fn sell(mut self) -> Self {
        self.is_buy = false;
        self
    }

    /// Set order size (quantity)
    pub fn size(mut self, size: f64) -> Result<Self, PrecisionError> {
        if !validate_quantity_precision(size) {
            return Err(PrecisionError::RoundingError { value: size });
        }
        self.size = Some(size);
        Ok(self)
    }

    /// Set limit price
    pub fn limit_price(mut self, price: f64) -> Result<Self, PrecisionError> {
        if !validate_price_precision(price) {
            return Err(PrecisionError::RoundingError { value: price });
        }
        self.limit_price = Some(price);
        Ok(self)
    }

    /// Set reduce only flag
    pub fn reduce_only(mut self, reduce_only: bool) -> Self {
        self.reduce_only = reduce_only;
        self
    }

    /// Set order type
    pub fn order_type(mut self, order_type: OrderType) -> Self {
        self.order_type = Some(order_type);
        self
    }

    /// Set peg offset value
    pub fn peg_offset(mut self, offset: f64) -> Result<Self, PrecisionError> {
        if !validate_price_precision(offset) {
            return Err(PrecisionError::RoundingError { value: offset });
        }
        self.peg_offset_value = Some(offset);
        Ok(self)
    }

    /// Set peg price type
    pub fn peg_price_type(mut self, peg_type: PegPriceType) -> Self {
        self.peg_price_type = Some(peg_type);
        self
    }

    /// Set trigger order
    pub fn trigger(mut self, condition: TriggerCondition, price: f64) -> Result<Self, PrecisionError> {
        if !validate_price_precision(price) {
            return Err(PrecisionError::RoundingError { value: price });
        }
        self.is_trigger = true;
        self.trigger_condition = Some(condition);
        self.trigger_px = Some(price);
        Ok(self)
    }

    /// Set client order ID
    pub fn cloid(mut self, cloid: impl Into<String>) -> Self {
        self.cloid = Some(cloid.into());
        self
    }

    /// Set order ID
    pub fn order_id(mut self, order_id: i64) -> Self {
        self.order_id = Some(order_id);
        self
    }

    /// Build the OrderWire struct
    pub fn build(self) -> Result<OrderWire, PrecisionError> {
        let size = self.size.ok_or_else(|| PrecisionError::RoundingError { value: 0.0 })?;
        let limit_price = self.limit_price.ok_or_else(|| PrecisionError::RoundingError { value: 0.0 })?;
        let order_type = self.order_type.unwrap_or(OrderType::Limit);

        Ok(OrderWire {
            coin: self.coin,
            cloid: self.cloid,
            order_id: self.order_id,
            limit_price: float_to_wire(limit_price)?,
            sz: float_to_wire(size)?,
            is_buy: self.is_buy,
            reduce_only: self.reduce_only,
            order_type,
            peg_offset_value: self.peg_offset_value.map(float_to_wire).transpose()?,
            peg_price_type: self.peg_price_type,
            is_trigger: if self.is_trigger { Some(true) } else { None },
            trigger_condition: self.trigger_condition,
            trigger_px: self.trigger_px.map(float_to_wire).transpose()?,
        })
    }
}

/// Order types for OrderWireBuilder
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum OrderType {
    Limit,
    Trigger,
}

/// Peg price types for OrderWireBuilder
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum PegPriceType {
    Mid,
    Oracle,
    Last,
    Opposite,
    OracleWithFallback,
}

/// Trigger conditions for OrderWireBuilder
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TriggerCondition {
    #[serde(rename = "mark")]
    Mark,
    #[serde(rename = "index")]
    Index,
    #[serde(rename = "last")]
    Last,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_float_to_wire_basic() {
        assert_eq!(float_to_wire(50000.12345678).unwrap(), "50000.12345678");
        assert_eq!(float_to_wire(0.00000001).unwrap(), "0.00000001");
        assert_eq!(float_to_wire(100.0).unwrap(), "100");
        assert_eq!(float_to_wire(0.0).unwrap(), "0");
    }

    #[test]
    fn test_float_to_wire_negative_zero() {
        assert_eq!(float_to_wire(-0.0).unwrap(), "0");
    }

    #[test]
    fn test_float_to_wire_rounding_error() {
        // This should fail because it would cause rounding
        assert!(float_to_wire(0.123456789).is_err());
    }

    #[test]
    fn test_float_to_int() {
        assert_eq!(float_to_int(1000.12345678, 8).unwrap(), 100012345678);
        assert_eq!(float_to_int(100.123456, 6).unwrap(), 100123456);
    }

    #[test]
    fn test_float_to_usd_int() {
        assert_eq!(float_to_usd_int(100.123456).unwrap(), 100123456);
    }

    #[test]
    fn test_float_to_int_for_hashing() {
        assert_eq!(float_to_int_for_hashing(1000.12345678).unwrap(), 100012345678);
    }

    #[test]
    fn test_can_convert_without_precision_loss() {
        assert!(can_convert_without_precision_loss(0.12345678, 8));
        assert!(!can_convert_without_precision_loss(0.123456789, 8));
    }

    #[test]
    fn test_validate_price_precision() {
        assert!(validate_price_precision(50000.12345678));
        assert!(!validate_price_precision(0.123456789));
    }

    #[test]
    fn test_validate_quantity_precision() {
        assert!(validate_quantity_precision(1.23456789));
        assert!(!validate_quantity_precision(1.234567891));
    }

    #[test]
    fn test_validate_usd_precision() {
        assert!(validate_usd_precision(100.123456));
        assert!(!validate_usd_precision(100.1234567));
    }

    #[test]
    fn test_order_wire_builder() {
        let order_wire = OrderWireBuilder::new("BTC")
            .buy()
            .size(1.5).unwrap()
            .limit_price(50000.0).unwrap()
            .build()
            .unwrap();

        assert_eq!(order_wire.coin, "BTC");
        assert_eq!(order_wire.sz, "1.5");
        assert_eq!(order_wire.limit_price, "50000");
        assert!(order_wire.is_buy);
        assert_eq!(order_wire.order_type, OrderType::Limit);
    }

    #[test]
    fn test_order_wire_builder_invalid_precision() {
        let result = OrderWireBuilder::new("BTC")
            .buy()
            .size(1.234567891) // Invalid precision
            .limit_price(50000.0)
            .build();

        assert!(result.is_err());
    }
}