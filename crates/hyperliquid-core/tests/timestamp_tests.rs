//! Tests for timestamp handling

use hyperliquid_core::types::timestamp::*;

#[tokio::test]
async fn test_get_timestamp_ms() {
    let timestamp = get_timestamp_ms().unwrap();
    assert!(timestamp > 1_000_000_000_000); // Should be after 2001
    assert!(is_valid_timestamp(timestamp));
}

#[tokio::test]
async fn test_get_timestamp_seconds() {
    let timestamp = get_timestamp_seconds().unwrap();
    assert!(timestamp > 1_000_000_000); // Should be after 2001
}

#[tokio::test]
async fn test_millis_to_seconds() {
    assert_eq!(millis_to_seconds(1_640_995_200_000), 1_640_995_200);
    assert_eq!(millis_to_seconds(1000), 1);
    assert_eq!(millis_to_seconds(500), 0); // Truncates
}

#[tokio::test]
async fn test_seconds_to_millis() {
    assert_eq!(seconds_to_millis(1_640_995_200), 1_640_995_200_000);
    assert_eq!(seconds_to_millis(1), 1000);
}

#[tokio::test]
async fn test_is_valid_timestamp() {
    // Valid timestamps (2000-2100)
    assert!(is_valid_timestamp(946684800000));    // 2000-01-01
    assert!(is_valid_timestamp(1_640_995_200_000)); // 2022-01-01
    assert!(is_valid_timestamp(4102444800000));   // 2100-01-01

    // Invalid timestamps
    assert!(!is_valid_timestamp(1_000_000));      // 1970
    assert!(!is_valid_timestamp(999_999_999_999_999)); // Far future
}

#[tokio::test]
async fn test_validate_future_timestamp() {
    let current_time = get_timestamp_ms().unwrap();
    let future_time = current_time + 60000; // 1 minute from now
    let past_time = current_time - 60000;   // 1 minute ago

    assert!(validate_future_timestamp(future_time).is_ok());
    assert!(validate_future_timestamp(past_time).is_err());
    assert!(validate_future_timestamp(current_time).is_err());
}

#[tokio::test]
async fn test_validate_past_timestamp() {
    let current_time = get_timestamp_ms().unwrap();
    let future_time = current_time + 60000; // 1 minute from now
    let past_time = current_time - 60000;   // 1 minute ago

    assert!(validate_past_timestamp(past_time).is_ok());
    assert!(validate_past_timestamp(future_time).is_err());
    assert!(validate_past_timestamp(current_time).is_err());
}

#[tokio::test]
async fn test_time_diff() {
    let t1 = 1_640_995_200_000;
    let t2 = 1_640_995_260_000; // 60 seconds later

    assert_eq!(time_diff_ms(t2, t1), 60_000);
    assert_eq!(time_diff_seconds(t2, t1), 60);
}

#[tokio::test]
async fn test_add_millis() {
    let timestamp = 1_640_995_200_000;
    let new_timestamp = add_millis(timestamp, 60000).unwrap();

    assert_eq!(new_timestamp, 1_640_995_260_000);
}

#[tokio::test]
async fn test_add_seconds() {
    let timestamp = 1_640_995_200_000;
    let new_timestamp = add_seconds(timestamp, 60).unwrap();

    assert_eq!(new_timestamp, 1_640_995_260_000);
}

#[tokio::test]
async fn test_overflow_protection() {
    // Test that very large numbers cause overflow errors
    let max_timestamp = 999_999_999_999_999;
    assert!(add_millis(max_timestamp, 1).is_err());
}

#[tokio::test]
async fn test_timestamp_integration_with_trading() {
    // Test timestamp usage in trading scenarios
    use chrono::Utc;

    // Get current timestamp
    let current_time = get_timestamp_ms().unwrap();
    assert!(current_time > 0);

    // Test order expiration timestamp
    let expires_after = add_seconds(current_time, 300).unwrap(); // 5 minutes from now
    assert!(expires_after > current_time);

    // Test time difference calculation
    let time_diff = time_diff_seconds(expires_after, current_time);
    assert_eq!(time_diff, 300);

    // Test that future validation works
    assert!(validate_future_timestamp(expires_after).is_ok());
    assert!(validate_past_timestamp(current_time).is_ok());
}

#[tokio::test]
async fn test_timestamp_edge_cases() {
    // Test boundary conditions
    assert!(is_valid_timestamp(946684800000));    // 2000-01-01 00:00:00 UTC
    assert!(is_valid_timestamp(4102444800000));   // 2100-01-01 00:00:00 UTC

    // Test just outside boundaries
    assert!(!is_valid_timestamp(946684799999));    // Just before 2000
    assert!(!is_valid_timestamp(4102444800001));   // Just after 2100

    // Test zero timestamp
    assert!(!is_valid_timestamp(0));

    // Test negative timestamp
    assert!(!is_valid_timestamp(-1000));
}

#[tokio::test]
async fn test_timestamp_error_types() {
    // Test that we get the right error types
    let result = validate_future_timestamp(0); // Very old timestamp
    assert!(result.is_err());

    if let Err(error) = result {
        match error {
            TimestampError::InvalidRange(_) => {
                // Expected error type
            }
            _ => panic!("Expected InvalidRange error, got: {:?}", error),
        }
    }

    // Test overflow error
    let result = add_millis(999_999_999_999_999, 1);
    assert!(result.is_err());

    if let Err(error) = result {
        match error {
            TimestampError::OverflowError(_) => {
                // Expected error type
            }
            _ => panic!("Expected OverflowError, got: {:?}", error),
        }
    }
}

#[tokio::test]
async fn test_timestamp_consistency() {
    // Test that timestamp functions are consistent with each other
    let timestamp_ms = 1_640_995_200_000;
    let timestamp_seconds = 1_640_995_200;

    // Test conversion consistency
    assert_eq!(millis_to_seconds(timestamp_ms), timestamp_seconds);
    assert_eq!(seconds_to_millis(timestamp_seconds), timestamp_ms);

    // Test that adding and subtracting gives the same result
    let added = add_millis(timestamp_ms, 1000).unwrap();
    let subtracted = added - 1000;
    assert_eq!(subtracted, timestamp_ms);

    // Test that time difference is consistent
    let diff = time_diff_ms(added, timestamp_ms);
    assert_eq!(diff, 1000);
}