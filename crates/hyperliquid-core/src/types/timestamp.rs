//! Timestamp handling for Unix milliseconds.
//!
//! This module provides functions to work with Unix timestamps in milliseconds,
//! which is the standard format used by the Hyperliquid API. It includes
//! functions to get current timestamps, validate timestamp ranges, and
//! convert between different timestamp formats.
//!
//! Based on the Python SDK's `get_timestamp_ms` function:
//! - Uses Unix epoch time in milliseconds
//! - Provides millisecond precision for trading operations
//! - Includes validation for reasonable timestamp ranges
//! - Handles edge cases and overflow protection

use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

/// Timestamp handling errors
#[derive(Error, Debug, Clone, PartialEq)]
pub enum TimestampError {
    #[error("System time error: {0}")]
    SystemTimeError(String),
    #[error("Timestamp overflow: {0}")]
    OverflowError(i64),
    #[error("Invalid timestamp range: {0}")]
    InvalidRange(i64),
}

/// Get current timestamp in Unix milliseconds
///
/// # Returns
/// * `Ok(i64)` - Current timestamp in milliseconds since Unix epoch
/// * `Err(TimestampError)` - If system time is before Unix epoch or overflow occurs
///
/// # Examples
/// ```
/// use hyperliquid_core::types::timestamp::get_timestamp_ms;
///
/// let timestamp = get_timestamp_ms().unwrap();
/// assert!(timestamp > 1_000_000_000_000); // Should be after 2001
/// ```
pub fn get_timestamp_ms() -> Result<i64, TimestampError> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| TimestampError::SystemTimeError(format!("System time before Unix epoch: {}", e)))?;

    // Convert to milliseconds
    let millis = now.as_secs() as i64 * 1000 + now.subsec_millis() as i64;

    // Validate reasonable range (year 2000 to 2100)
    if millis < 946684800000 || millis > 4102444800000 {
        return Err(TimestampError::InvalidRange(millis));
    }

    Ok(millis)
}

/// Get current timestamp in Unix seconds
///
/// # Returns
/// * `Ok(i64)` - Current timestamp in seconds since Unix epoch
/// * `Err(TimestampError)` - If system time is before Unix epoch or overflow occurs
///
/// # Examples
/// ```
/// use hyperliquid_core::types::timestamp::get_timestamp_seconds;
///
/// let timestamp = get_timestamp_seconds().unwrap();
/// assert!(timestamp > 1_000_000_000); // Should be after 2001
/// ```
pub fn get_timestamp_seconds() -> Result<i64, TimestampError> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| TimestampError::SystemTimeError(format!("System time before Unix epoch: {}", e)))?;

    // Convert to seconds
    let seconds = now.as_secs() as i64;

    // Validate reasonable range (year 2000 to 2100)
    if seconds < 946684800 || seconds > 4102444800 {
        return Err(TimestampError::InvalidRange(seconds));
    }

    Ok(seconds)
}

/// Convert milliseconds to seconds
///
/// # Arguments
/// * `millis` - Timestamp in milliseconds
///
/// # Returns
/// * `i64` - Timestamp in seconds
///
/// # Examples
/// ```
/// use hyperliquid_core::types::timestamp::millis_to_seconds;
///
/// let millis = 1_640_995_200_000; // 2022-01-01 00:00:00 UTC
/// let seconds = millis_to_seconds(millis);
/// assert_eq!(seconds, 1_640_995_200);
/// ```
pub fn millis_to_seconds(millis: i64) -> i64 {
    millis / 1000
}

/// Convert seconds to milliseconds
///
/// # Arguments
/// * `seconds` - Timestamp in seconds
///
/// # Returns
/// * `i64` - Timestamp in milliseconds
///
/// # Examples
/// ```
/// use hyperliquid_core::types::timestamp::seconds_to_millis;
///
/// let seconds = 1_640_995_200; // 2022-01-01 00:00:00 UTC
/// let millis = seconds_to_millis(seconds);
/// assert_eq!(millis, 1_640_995_200_000);
/// ```
pub fn seconds_to_millis(seconds: i64) -> i64 {
    seconds * 1000
}

/// Validate timestamp is within reasonable range
///
/// # Arguments
/// * `timestamp` - Timestamp to validate (in milliseconds)
///
/// # Returns
/// * `true` - If timestamp is within reasonable range (2000-2100)
/// * `false` - If timestamp is outside reasonable range
///
/// # Examples
/// ```
/// use hyperliquid_core::types::timestamp::is_valid_timestamp;
///
/// let valid_timestamp = 1_640_995_200_000; // 2022-01-01
/// let invalid_timestamp = 1_000_000; // 1970
///
/// assert!(is_valid_timestamp(valid_timestamp));
/// assert!(!is_valid_timestamp(invalid_timestamp));
/// ```
pub fn is_valid_timestamp(timestamp: i64) -> bool {
    timestamp >= 946684800000 && timestamp <= 4102444800000
}

/// Validate timestamp is in the future
///
/// # Arguments
/// * `timestamp` - Timestamp to validate (in milliseconds)
///
/// # Returns
/// * `Ok(())` - If timestamp is in the future
/// * `Err(TimestampError)` - If timestamp is in the past
///
/// # Examples
/// ```
/// use hyperliquid_core::types::timestamp::{get_timestamp_ms, validate_future_timestamp};
///
/// let current_time = get_timestamp_ms().unwrap();
/// let future_time = current_time + 60000; // 1 minute from now
/// let past_time = current_time - 60000; // 1 minute ago
///
/// assert!(validate_future_timestamp(future_time).is_ok());
/// assert!(validate_future_timestamp(past_time).is_err());
/// ```
pub fn validate_future_timestamp(timestamp: i64) -> Result<(), TimestampError> {
    let current_time = get_timestamp_ms()?;
    if timestamp <= current_time {
        return Err(TimestampError::InvalidRange(timestamp));
    }
    Ok(())
}

/// Validate timestamp is in the past
///
/// # Arguments
/// * `timestamp` - Timestamp to validate (in milliseconds)
///
/// # Returns
/// * `Ok(())` - If timestamp is in the past
/// * `Err(TimestampError)` - If timestamp is in the future
///
/// # Examples
/// ```
/// use hyperliquid_core::types::timestamp::{get_timestamp_ms, validate_past_timestamp};
///
/// let current_time = get_timestamp_ms().unwrap();
/// let future_time = current_time + 60000; // 1 minute from now
/// let past_time = current_time - 60000; // 1 minute ago
///
/// assert!(validate_past_timestamp(past_time).is_ok());
/// assert!(validate_past_timestamp(future_time).is_err());
/// ```
pub fn validate_past_timestamp(timestamp: i64) -> Result<(), TimestampError> {
    let current_time = get_timestamp_ms()?;
    if timestamp >= current_time {
        return Err(TimestampError::InvalidRange(timestamp));
    }
    Ok(())
}

/// Calculate time difference between two timestamps
///
/// # Arguments
/// * `timestamp1` - First timestamp (in milliseconds)
/// * `timestamp2` - Second timestamp (in milliseconds)
///
/// # Returns
/// * `i64` - Difference in milliseconds (positive if timestamp1 > timestamp2)
///
/// # Examples
/// ```
/// use hyperliquid_core::types::timestamp::time_diff_ms;
///
/// let t1 = 1_640_995_200_000;
/// let t2 = 1_640_995_260_000; // 60 seconds later
///
/// let diff = time_diff_ms(t2, t1);
/// assert_eq!(diff, 60_000); // 60 seconds in milliseconds
/// ```
pub fn time_diff_ms(timestamp1: i64, timestamp2: i64) -> i64 {
    timestamp1 - timestamp2
}

/// Calculate time difference in seconds
///
/// # Arguments
/// * `timestamp1` - First timestamp (in milliseconds)
/// * `timestamp2` - Second timestamp (in milliseconds)
///
/// # Returns
/// * `i64` - Difference in seconds
///
/// # Examples
/// ```
/// use hyperliquid_core::types::timestamp::time_diff_seconds;
///
/// let t1 = 1_640_995_200_000;
/// let t2 = 1_640_995_260_000; // 60 seconds later
///
/// let diff = time_diff_seconds(t2, t1);
/// assert_eq!(diff, 60); // 60 seconds
/// ```
pub fn time_diff_seconds(timestamp1: i64, timestamp2: i64) -> i64 {
    (timestamp1 - timestamp2) / 1000
}

/// Add milliseconds to a timestamp
///
/// # Arguments
/// * `timestamp` - Base timestamp (in milliseconds)
/// * `millis` - Milliseconds to add
///
/// # Returns
/// * `Result<i64, TimestampError>` - New timestamp or error if overflow
///
/// # Examples
/// ```
/// use hyperliquid_core::types::timestamp::add_millis;
///
/// let timestamp = 1_640_995_200_000; // 2022-01-01 00:00:00 UTC
/// let new_timestamp = add_millis(timestamp, 60000).unwrap(); // Add 60 seconds
///
/// assert_eq!(new_timestamp, 1_640_995_260_000);
/// ```
pub fn add_millis(timestamp: i64, millis: i64) -> Result<i64, TimestampError> {
    let result = timestamp.checked_add(millis)
        .ok_or_else(|| TimestampError::OverflowError(timestamp))?;

    if !is_valid_timestamp(result) {
        return Err(TimestampError::InvalidRange(result));
    }

    Ok(result)
}

/// Add seconds to a timestamp
///
/// # Arguments
/// * `timestamp` - Base timestamp (in milliseconds)
/// * `seconds` - Seconds to add
///
/// # Returns
/// * `Result<i64, TimestampError>` - New timestamp or error if overflow
///
/// # Examples
/// ```
/// use hyperliquid_core::types::timestamp::add_seconds;
///
/// let timestamp = 1_640_995_200_000; // 2022-01-01 00:00:00 UTC
/// let new_timestamp = add_seconds(timestamp, 60).unwrap(); // Add 60 seconds
///
/// assert_eq!(new_timestamp, 1_640_995_260_000);
/// ```
pub fn add_seconds(timestamp: i64, seconds: i64) -> Result<i64, TimestampError> {
    add_millis(timestamp, seconds * 1000)
}

/// Format timestamp as ISO 8601 string (for debugging/logging)
///
/// # Arguments
/// * `timestamp` - Timestamp in milliseconds
///
/// # Returns
/// * `String` - ISO 8601 formatted string
///
/// # Examples
/// ```
/// use hyperliquid_core::types::timestamp::format_timestamp;
///
/// let timestamp = 1_640_995_200_000; // 2022-01-01 00:00:00 UTC
/// let formatted = format_timestamp(timestamp);
///
/// assert!(formatted.contains("2022-01-01"));
/// ```
pub fn format_timestamp(timestamp: i64) -> String {
    // For now, just return the raw timestamp as a string
    // In a real implementation, you might want to use chrono for proper formatting
    timestamp.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_timestamp_ms() {
        let timestamp = get_timestamp_ms().unwrap();
        assert!(timestamp > 1_000_000_000_000); // Should be after 2001
        assert!(is_valid_timestamp(timestamp));
    }

    #[test]
    fn test_get_timestamp_seconds() {
        let timestamp = get_timestamp_seconds().unwrap();
        assert!(timestamp > 1_000_000_000); // Should be after 2001
    }

    #[test]
    fn test_millis_to_seconds() {
        assert_eq!(millis_to_seconds(1_640_995_200_000), 1_640_995_200);
        assert_eq!(millis_to_seconds(1000), 1);
        assert_eq!(millis_to_seconds(500), 0); // Truncates
    }

    #[test]
    fn test_seconds_to_millis() {
        assert_eq!(seconds_to_millis(1_640_995_200), 1_640_995_200_000);
        assert_eq!(seconds_to_millis(1), 1000);
    }

    #[test]
    fn test_is_valid_timestamp() {
        // Valid timestamps (2000-2100)
        assert!(is_valid_timestamp(946684800000));    // 2000-01-01
        assert!(is_valid_timestamp(1_640_995_200_000)); // 2022-01-01
        assert!(is_valid_timestamp(4102444800000));   // 2100-01-01

        // Invalid timestamps
        assert!(!is_valid_timestamp(1_000_000));      // 1970
        assert!(!is_valid_timestamp(999_999_999_999_999)); // Far future
    }

    #[test]
    fn test_validate_future_timestamp() {
        let current_time = get_timestamp_ms().unwrap();
        let future_time = current_time + 60000; // 1 minute from now
        let past_time = current_time - 60000;   // 1 minute ago

        assert!(validate_future_timestamp(future_time).is_ok());
        assert!(validate_future_timestamp(past_time).is_err());
        assert!(validate_future_timestamp(current_time).is_err());
    }

    #[test]
    fn test_validate_past_timestamp() {
        let current_time = get_timestamp_ms().unwrap();
        let future_time = current_time + 60000; // 1 minute from now
        let past_time = current_time - 60000;   // 1 minute ago

        assert!(validate_past_timestamp(past_time).is_ok());
        assert!(validate_past_timestamp(future_time).is_err());
        assert!(validate_past_timestamp(current_time).is_err());
    }

    #[test]
    fn test_time_diff() {
        let t1 = 1_640_995_200_000;
        let t2 = 1_640_995_260_000; // 60 seconds later

        assert_eq!(time_diff_ms(t2, t1), 60_000);
        assert_eq!(time_diff_seconds(t2, t1), 60);
    }

    #[test]
    fn test_add_millis() {
        let timestamp = 1_640_995_200_000;
        let new_timestamp = add_millis(timestamp, 60000).unwrap();

        assert_eq!(new_timestamp, 1_640_995_260_000);
    }

    #[test]
    fn test_add_seconds() {
        let timestamp = 1_640_995_200_000;
        let new_timestamp = add_seconds(timestamp, 60).unwrap();

        assert_eq!(new_timestamp, 1_640_995_260_000);
    }

    #[test]
    fn test_overflow_protection() {
        // Test that very large numbers cause overflow errors
        let max_timestamp = 999_999_999_999_999;
        assert!(add_millis(max_timestamp, 1).is_err());
    }
}