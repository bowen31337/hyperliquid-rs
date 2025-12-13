//! Nonce generation utilities for Hyperliquid SDK
//!
//! This module provides secure nonce generation for cryptographic operations,
//! ensuring uniqueness and preventing replay attacks.

#[cfg(unix)]
extern crate libc;

use chrono::{DateTime, Utc};
use rand::Rng;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use std::ptr;
use std::alloc::{alloc, dealloc, Layout};

/// Generate a unique nonce for cryptographic operations
///
/// A nonce is a number used once to ensure that each operation is unique and
/// cannot be replayed. This function combines timestamp and random components
/// to ensure uniqueness even in high-frequency scenarios.
///
/// # Returns
///
/// A unique u64 nonce value
///
/// # Examples
///
/// ```
/// use hyperliquid_core::crypto::generate_nonce;
///
/// let nonce = generate_nonce();
/// println!("Generated nonce: {}", nonce);
/// ```
pub fn generate_nonce() -> u64 {
    static COUNTER: AtomicU64 = AtomicU64::new(0);

    // Get current timestamp in milliseconds
    let timestamp_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("System time is before Unix epoch")
        .as_millis() as u64;

    // Get a small random component for additional uniqueness
    let random_bits = rand::thread_rng().gen::<u16>() as u64;

    // Get an atomic counter to handle concurrent calls
    let counter = COUNTER.fetch_add(1, Ordering::Relaxed);

    // Combine timestamp, random bits, and counter
    // Format: [timestamp_ms (44 bits)] | [counter (10 bits)] | [random (10 bits)]
    (timestamp_ms << 20) | ((counter & 0x3FF) << 10) | (random_bits & 0x3FF)
}

/// Generate a timestamp-based nonce
///
/// This function creates a nonce based primarily on the current timestamp,
/// useful for operations where time-based ordering is important.
///
/// # Returns
///
/// A timestamp-based u64 nonce value
///
/// # Examples
///
/// ```
/// use hyperliquid_core::crypto::generate_timestamp_nonce;
///
/// let nonce = generate_timestamp_nonce();
/// println!("Generated timestamp nonce: {}", nonce);
/// ```
pub fn generate_timestamp_nonce() -> u64 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("System time is before Unix epoch");

    // Use microseconds since epoch for high precision
    now.as_micros() as u64
}

/// Thread-safe nonce generator for high-frequency operations
///
/// This struct provides a thread-safe way to generate sequential nonces
/// with optional timestamp tracking. Useful for applications that need
/// many nonces in a short period.
///
/// # Examples
///
/// ```
/// use hyperliquid_core::crypto::NonceGenerator;
///
/// let mut generator = NonceGenerator::new();
/// let nonce1 = generator.next();
/// let nonce2 = generator.next();
/// assert_ne!(nonce1, nonce2);
/// ```
#[derive(Debug)]
pub struct NonceGenerator {
    base_timestamp: u64,
    counter: AtomicU64,
}

impl NonceGenerator {
    /// Create a new nonce generator
    pub fn new() -> Self {
        Self {
            base_timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("System time is before Unix epoch")
                .as_millis() as u64,
            counter: AtomicU64::new(0),
        }
    }

    /// Generate the next nonce in sequence
    pub fn next(&self) -> u64 {
        let counter = self.counter.fetch_add(1, Ordering::Relaxed);
        self.base_timestamp + counter
    }

    /// Reset the generator with a new base timestamp
    pub fn reset(&mut self) {
        self.base_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("System time is before Unix epoch")
            .as_millis() as u64;
        self.counter.store(0, Ordering::Relaxed);
    }

    /// Get the current timestamp used by the generator
    pub fn current_timestamp(&self) -> u64 {
        self.base_timestamp
    }
}

/// Verify that a nonce is not too old (potential replay attack prevention)
///
/// # Arguments
///
/// * `nonce` - The nonce to check
/// * `max_age_seconds` - Maximum allowed age in seconds (default: 300)
///
/// # Returns
///
/// `true` if the nonce is recent enough, `false` otherwise
///
/// # Examples
///
/// ```
/// use hyperliquid_core::crypto::{generate_nonce, verify_nonce_age};
///
/// let nonce = generate_nonce();
/// let is_recent = verify_nonce_age(nonce, Some(60));
/// assert!(is_recent);
/// ```
pub fn verify_nonce_age(nonce: u64, max_age_seconds: Option<u64>) -> bool {
    let max_age = max_age_seconds.unwrap_or(300); // Default 5 minutes
    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("System time is before Unix epoch")
        .as_secs();

    // Extract timestamp from nonce (assuming it's in the upper bits)
    // For timestamp-based nonces, we can check the age
    let nonce_time = if nonce >> 32 > 1_000_000_000 {
        // Likely a timestamp-based nonce (seconds since epoch in upper bits)
        nonce >> 32
    } else {
        // For mixed nonces, try to extract reasonable timestamp
        // This is a heuristic and may not work for all nonce formats
        nonce / 1_000_000 // Convert approximate milliseconds to seconds
    };

    current_time.saturating_sub(nonce_time) <= max_age
}

/// Get current timestamp in milliseconds since Unix epoch
///
/// # Returns
///
/// Current timestamp as u64 milliseconds
pub fn current_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("System time is before Unix epoch")
        .as_millis() as u64
}

/// Get current timestamp in microseconds since Unix epoch
///
/// # Returns
///
/// Current timestamp as u64 microseconds
pub fn current_timestamp_us() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("System time is before Unix epoch")
        .as_micros() as u64
}

/// Format timestamp as human-readable datetime string
///
/// # Arguments
///
/// * `timestamp_ms` - Timestamp in milliseconds
///
/// # Returns
///
/// Formatted datetime string
pub fn format_timestamp(timestamp_ms: u64) -> String {
    let seconds = timestamp_ms / 1000;
    let nanos = (timestamp_ms % 1000) * 1_000_000;
    let datetime = DateTime::<Utc>::from_naive_utc_and_offset(
        chrono::NaiveDateTime::from_timestamp_opt(seconds as i64, nanos as u32).unwrap_or_default(),
        Utc,
    );
    datetime.format("%Y-%m-%d %H:%M:%S.%3f UTC").to_string()
}

/// Securely allocated memory buffer for sensitive data
///
/// This struct provides secure memory allocation with automatic zeroization
/// on drop. It uses mlock() to prevent swapping to disk and munlock() to
/// release the memory lock when done.
///
/// Note: On some platforms, mlock() may require elevated privileges.
pub struct SecureBuffer {
    ptr: *mut u8,
    len: usize,
    #[cfg(unix)]
    locked: bool,
}

impl SecureBuffer {
    /// Create a new secure buffer with the given size
    pub fn new(size: usize) -> Result<Self, SecureBufferError> {
        if size == 0 {
            return Err(SecureBufferError::InvalidSize);
        }

        let layout = Layout::from_size_align(size, 8).map_err(|_| SecureBufferError::InvalidSize)?;
        let ptr = unsafe { alloc(layout) };

        if ptr.is_null() {
            return Err(SecureBufferError::AllocationFailed);
        }

        // Initialize buffer to zero
        unsafe {
            ptr.write_bytes(0, size);
        }

        #[cfg(unix)]
        let locked = {
            // Try to lock memory to prevent swapping
            let result = unsafe { libc::mlock(ptr as *const libc::c_void, size) };
            result == 0
        };

        #[cfg(not(unix))]
        let locked = false;

        Ok(SecureBuffer {
            ptr,
            len: size,
            #[cfg(unix)]
            locked,
        })
    }

    /// Get a mutable slice to the buffer contents
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        unsafe { std::slice::from_raw_parts_mut(self.ptr, self.len) }
    }

    /// Get a slice to the buffer contents
    pub fn as_slice(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.ptr, self.len) }
    }

    /// Copy data into the secure buffer
    pub fn copy_from_slice(&mut self, data: &[u8]) -> Result<(), SecureBufferError> {
        if data.len() != self.len {
            return Err(SecureBufferError::InvalidLength);
        }
        self.as_mut_slice().copy_from_slice(data);
        Ok(())
    }

    /// Fill the buffer with random data
    pub fn fill_random(&mut self) {
        let slice = self.as_mut_slice();
        for byte in slice.iter_mut() {
            *byte = rand::random();
        }
    }

    /// Zeroize the buffer contents
    pub fn zeroize(&mut self) {
        self.as_mut_slice().fill(0);
    }

    /// Get the length of the buffer
    pub fn len(&self) -> usize {
        self.len
    }

    /// Check if the buffer is empty
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

impl Drop for SecureBuffer {
    fn drop(&mut self) {
        // Zeroize memory before freeing
        self.zeroize();

        #[cfg(unix)]
        {
            if self.locked {
                // Unlock memory
                unsafe {
                    let _ = libc::munlock(self.ptr as *const libc::c_void, self.len);
                }
            }
        }

        // Free memory
        unsafe {
            let layout = Layout::from_size_align(self.len, 8).unwrap();
            dealloc(self.ptr, layout);
        }
    }
}

/// Secure private key wrapper with automatic zeroization
///
/// This struct provides secure storage for private keys with automatic
/// zeroization on drop and optional memory locking to prevent swapping.
#[derive(Debug)]
pub struct PrivateKeySecure {
    buffer: SecureBuffer,
}

impl PrivateKeySecure {
    /// Create a new secure private key from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, SecureBufferError> {
        let mut buffer = SecureBuffer::new(bytes.len())?;
        buffer.copy_from_slice(bytes)?;
        Ok(PrivateKeySecure { buffer })
    }

    /// Create a new secure private key from hex string
    pub fn from_hex(hex_key: &str) -> Result<Self, SecureBufferError> {
        let key_bytes = hex::decode(hex_key.trim_start_matches("0x"))
            .map_err(|_| SecureBufferError::InvalidHex)?;
        Self::from_bytes(&key_bytes)
    }

    /// Get a reference to the key bytes
    pub fn as_bytes(&self) -> &[u8] {
        self.buffer.as_slice()
    }

    /// Get the key length in bytes
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Check if the key is empty
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Zeroize the private key
    pub fn zeroize(&mut self) {
        self.buffer.zeroize();
    }

    /// Wipe and destroy the key
    pub fn destroy(mut self) {
        self.zeroize();
        // Buffer will be zeroized again in drop
    }
}

impl Drop for PrivateKeySecure {
    fn drop(&mut self) {
        // Zeroize key before dropping
        self.zeroize();
    }
}

/// Errors that can occur when working with secure buffers
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SecureBufferError {
    /// Invalid buffer size (zero or too large)
    InvalidSize,
    /// Memory allocation failed
    AllocationFailed,
    /// Invalid data length for operation
    InvalidLength,
    /// Invalid hex string
    InvalidHex,
    /// Memory lock failed (platform-specific)
    MemoryLockFailed,
}

impl std::fmt::Display for SecureBufferError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SecureBufferError::InvalidSize => write!(f, "Invalid buffer size"),
            SecureBufferError::AllocationFailed => write!(f, "Memory allocation failed"),
            SecureBufferError::InvalidLength => write!(f, "Invalid data length"),
            SecureBufferError::InvalidHex => write!(f, "Invalid hex string"),
            SecureBufferError::MemoryLockFailed => write!(f, "Memory lock failed"),
        }
    }
}

impl std::error::Error for SecureBufferError {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_generate_nonce_uniqueness() {
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
        let generator = std::sync::Arc::new(NonceGenerator::new());
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
    fn test_secure_buffer_error_messages() {
        assert_eq!(SecureBufferError::InvalidSize.to_string(), "Invalid buffer size");
        assert_eq!(SecureBufferError::AllocationFailed.to_string(), "Memory allocation failed");
        assert_eq!(SecureBufferError::InvalidLength.to_string(), "Invalid data length");
    }
}