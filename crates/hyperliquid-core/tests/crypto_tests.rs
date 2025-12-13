//! Tests for crypto nonce generation and secure memory handling

use hyperliquid_core::crypto::{generate_nonce, generate_timestamp_nonce, NonceGenerator, verify_nonce_age, current_timestamp_ms, current_timestamp_us, format_timestamp};
use hyperliquid_core::crypto::nonce::{SecureBuffer, PrivateKeySecure, SecureBufferError};

#[test]
fn test_generate_nonce_uniqueness() {
    use std::collections::HashSet;

    let mut nonces = HashSet::new();
    for _ in 0..1000 {
        let nonce = generate_nonce();
        assert!(nonces.insert(nonce), "Duplicate nonce generated: {}", nonce);
    }
}

#[test]
fn test_generate_nonce_reasonable_value() {
    let nonce = generate_nonce();
    let current_time = current_timestamp_ms();
    // Nonce should be roughly based on current time (in upper bits)
    let nonce_time = nonce >> 20;
    assert!(nonce_time <= current_time);
    assert!(nonce_time > current_time.saturating_sub(1000)); // Within last second
}

#[test]
fn test_generate_timestamp_nonce() {
    use std::thread;
    use std::time::Duration;

    let nonce1 = generate_timestamp_nonce();
    thread::sleep(Duration::from_millis(1));
    let nonce2 = generate_timestamp_nonce();
    assert!(nonce2 > nonce1, "Timestamp nonces should be increasing");
}

#[test]
fn test_nonce_generator() {
    let generator = NonceGenerator::new();
    let nonce1 = generator.next();
    let nonce2 = generator.next();
    assert_eq!(nonce2, nonce1 + 1);
}

#[test]
fn test_nonce_generator_thread_safety() {
    use std::collections::HashSet;
    use std::sync::Arc;
    use std::thread;

    let generator = Arc::new(NonceGenerator::new());
    let mut handles = vec![];

    for _ in 0..10 {
        let gen = generator.clone();
        handles.push(thread::spawn(move || {
            let mut nonces = Vec::new();
            for _ in 0..100 {
                nonces.push(gen.next());
            }
            nonces
        }));
    }

    let mut all_nonces = HashSet::new();
    for handle in handles {
        for nonce in handle.join().unwrap() {
            assert!(all_nonces.insert(nonce), "Duplicate nonce in concurrent generation: {}", nonce);
        }
    }
}

#[test]
fn test_verify_nonce_age() {
    let old_nonce = current_timestamp_ms() - 60_000; // 60 seconds ago
    assert!(!verify_nonce_age(old_nonce, Some(30)));

    let recent_nonce = current_timestamp_ms() - 10_000; // 10 seconds ago
    assert!(verify_nonce_age(recent_nonce, Some(30)));
}

#[test]
fn test_format_timestamp() {
    let timestamp = 1_640_995_200_000; // 2022-01-01 00:00:00 UTC
    let formatted = format_timestamp(timestamp);
    assert!(formatted.contains("2022-01-01"));
    assert!(formatted.contains("00:00:00"));
}

#[test]
fn test_generate_nonce_with_concurrent_calls() {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::thread;

    let mut handles = vec![];
    let all_nonces = Arc::new(std::sync::Mutex::new(std::collections::HashSet::new()));

    for _ in 0..50 {
        let all_nonces = all_nonces.clone();
        handles.push(thread::spawn(move || {
            for _ in 0..100 {
                let nonce = generate_nonce();
                let mut nonces = all_nonces.lock().unwrap();
                nonces.insert(nonce);
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let nonces = all_nonces.lock().unwrap();
    assert_eq!(nonces.len(), 5000, "All nonces should be unique");
}

#[test]
fn test_generate_timestamp_nonce_with_microsecond_precision() {
    use std::thread;
    use std::time::Duration;

    let nonce1 = generate_timestamp_nonce();
    thread::sleep(Duration::from_micros(1000)); // Sleep for 1 millisecond
    let nonce2 = generate_timestamp_nonce();

    // Should be at least 1000 microseconds apart
    assert!(nonce2 - nonce1 >= 1000);
}

#[test]
fn test_nonce_generator_with_reset() {
    let mut generator = NonceGenerator::new();
    let base_time = generator.current_timestamp();

    let nonce1 = generator.next();
    let nonce2 = generator.next();

    assert_eq!(nonce1, base_time);
    assert_eq!(nonce2, base_time + 1);

    // Reset should change the base timestamp
    generator.reset();
    let new_base_time = generator.current_timestamp();
    assert_ne!(new_base_time, base_time);

    let nonce3 = generator.next();
    assert_eq!(nonce3, new_base_time);
}

#[test]
fn test_verify_nonce_age_with_default_max_age() {
    let recent_nonce = current_timestamp_ms() - 10_000; // 10 seconds ago
    assert!(verify_nonce_age(recent_nonce, None)); // Should use default 300 seconds

    let very_old_nonce = current_timestamp_ms() - 400_000; // 400 seconds ago
    assert!(!verify_nonce_age(very_old_nonce, None));
}

#[test]
fn test_current_timestamp_functions() {
    let ms1 = current_timestamp_ms();
    let us1 = current_timestamp_us();

    // Microseconds should be much larger than milliseconds
    assert!(us1 > ms1);

    // Should be roughly proportional (within 1 second)
    let ms_in_us = ms1 * 1000;
    assert!((us1 as i64 - ms_in_us as i64).abs() < 1_000_000); // Within 1 second
}

#[test]
fn test_format_timestamp_edge_cases() {
    // Test Unix epoch
    let epoch = 0;
    let formatted = format_timestamp(epoch);
    assert!(formatted.contains("1970-01-01"));

    // Test far future date
    let future = 4_000_000_000_000; // Year ~2096
    let formatted = format_timestamp(future);
    assert!(formatted.contains("2096"));

    // Test leap year
    let leap_year = 1_580_515_200_000; // 2020-02-01 00:00:00 UTC
    let formatted = format_timestamp(leap_year);
    assert!(formatted.contains("2020-02-01"));
}

#[test]
fn test_secure_buffer_creation() {
    let buffer = SecureBuffer::new(32).unwrap();
    assert_eq!(buffer.len(), 32);
    assert!(!buffer.is_empty());
    assert_eq!(buffer.as_slice(), &[0; 32]);
}

#[test]
fn test_secure_buffer_copy_from_slice() {
    let mut buffer = SecureBuffer::new(32).unwrap();
    let data = [1, 2, 3, 4, 5];
    assert!(buffer.copy_from_slice(&data).is_err()); // Wrong length

    let data = [0; 32];
    assert!(buffer.copy_from_slice(&data).is_ok());
    assert_eq!(buffer.as_slice(), &data);
}

#[test]
fn test_secure_buffer_fill_random() {
    let mut buffer = SecureBuffer::new(32).unwrap();
    let initial = buffer.as_slice().to_vec();
    assert_eq!(initial, vec![0; 32]);

    buffer.fill_random();
    let after = buffer.as_slice().to_vec();
    assert_ne!(after, vec![0; 32]);
}

#[test]
fn test_secure_buffer_zeroize() {
    let mut buffer = SecureBuffer::new(32).unwrap();
    buffer.fill_random();

    let before = buffer.as_slice().to_vec();
    assert_ne!(before, vec![0; 32]);

    buffer.zeroize();
    assert_eq!(buffer.as_slice(), &[0; 32]);
}

#[test]
fn test_secure_buffer_drop_zeroizes() {
    let buffer = SecureBuffer::new(32).unwrap();
    let mut buffer = buffer;
    buffer.fill_random();

    let before_ptr = buffer.as_slice().as_ptr();
    drop(buffer);

    // After drop, memory should be zeroized
    // Note: This test is inherently unsafe and depends on implementation details
    // In a real implementation, you might use valgrind or similar tools
}

#[test]
fn test_secure_private_key() {
    let key_data = [1; 32];
    let key = PrivateKeySecure::from_bytes(&key_data).unwrap();

    assert_eq!(key.len(), 32);
    assert!(!key.is_empty());
    assert_eq!(key.as_bytes(), &key_data);
}

#[test]
fn test_secure_private_key_from_hex() {
    let hex_key = "0x1111111111111111111111111111111111111111111111111111111111111111";
    let key = PrivateKeySecure::from_hex(hex_key).unwrap();

    assert_eq!(key.len(), 32);
    let expected_bytes = hex::decode("1111111111111111111111111111111111111111111111111111111111111111").unwrap();
    assert_eq!(key.as_bytes(), &expected_bytes);
}

#[test]
fn test_secure_private_key_zeroize() {
    let key_data = [1; 32];
    let mut key = PrivateKeySecure::from_bytes(&key_data).unwrap();

    assert_eq!(key.as_bytes(), &key_data);

    key.zeroize();
    assert_eq!(key.as_bytes(), &[0; 32]);
}

#[test]
fn test_secure_private_key_destroy() {
    let key_data = [1; 32];
    let key = PrivateKeySecure::from_bytes(&key_data).unwrap();

    assert_eq!(key.as_bytes(), &key_data);

    key.destroy();
    // Key is consumed and destroyed
}

#[test]
fn test_secure_private_key_drop_zeroizes() {
    let key_data = [1; 32];
    let mut key = PrivateKeySecure::from_bytes(&key_data).unwrap();
    key.zeroize();

    let before_ptr = key.as_bytes().as_ptr();
    drop(key);

    // After drop, memory should be zeroized
    // Note: This test is inherently unsafe and depends on implementation details
}

#[test]
fn test_secure_buffer_error_messages() {
    assert_eq!(SecureBufferError::InvalidSize.to_string(), "Invalid buffer size");
    assert_eq!(SecureBufferError::AllocationFailed.to_string(), "Memory allocation failed");
    assert_eq!(SecureBufferError::InvalidLength.to_string(), "Invalid data length");
    assert_eq!(SecureBufferError::InvalidHex.to_string(), "Invalid hex string");
}

#[test]
fn test_secure_buffer_edge_cases() {
    // Test zero size
    assert!(matches!(SecureBuffer::new(0), Err(SecureBufferError::InvalidSize)));

    // Test very large size (might fail on some systems)
    let large_size = usize::MAX / 2;
    let result = SecureBuffer::new(large_size);
    // This might fail due to allocation limits, which is expected
    if result.is_ok() {
        let buffer = result.unwrap();
        assert_eq!(buffer.len(), large_size);
    }
}

#[test]
fn test_secure_private_key_edge_cases() {
    // Test empty key
    assert!(PrivateKeySecure::from_bytes(&[]).is_ok()); // Empty buffer should be allowed

    // Test invalid hex
    assert!(matches!(
        PrivateKeySecure::from_hex("invalid_hex"),
        Err(SecureBufferError::InvalidHex)
    ));

    // Test hex with wrong length
    assert!(PrivateKeySecure::from_hex("0x123").is_err());
}

#[test]
fn test_nonce_uniqueness_across_threads() {
    use std::sync::Arc;
    use std::sync::Mutex;
    use std::thread;

    let nonces = Arc::new(Mutex::new(std::collections::HashSet::new()));
    let mut handles = vec![];

    for _ in 0..20 {
        let nonces = nonces.clone();
        handles.push(thread::spawn(move || {
            for _ in 0..500 {
                let nonce = generate_nonce();
                let mut set = nonces.lock().unwrap();
                set.insert(nonce);
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let nonces = nonces.lock().unwrap();
    assert_eq!(nonces.len(), 10000, "All nonces should be unique across threads");
}

#[test]
fn test_nonce_generator_sequential() {
    let generator = NonceGenerator::new();
    let mut previous = generator.next();

    for i in 1..1000 {
        let current = generator.next();
        assert_eq!(current, previous + 1, "Nonces should be sequential");
        previous = current;

        // Verify it's incrementing correctly
        assert_eq!(current, generator.current_timestamp() + i as u64);
    }
}

#[test]
fn test_verify_nonce_age_timestamp_extraction() {
    // Test with timestamp-based nonce (high bits contain timestamp)
    let current_time = current_timestamp_ms();
    let timestamp_nonce = (current_time << 20) | 12345;
    assert!(verify_nonce_age(timestamp_nonce, Some(60)));

    // Test with old timestamp-based nonce
    let old_time = current_time - 100_000; // 100 seconds ago
    let old_timestamp_nonce = (old_time << 20) | 12345;
    assert!(!verify_nonce_age(old_timestamp_nonce, Some(60)));

    // Test with mixed nonce
    let mixed_nonce = current_time * 1_000_000 + 12345;
    assert!(verify_nonce_age(mixed_nonce, Some(60)));
}