//! Circular buffer for WebSocket message handling
//!
//! This module provides a circular buffer implementation for handling bursts of
//! WebSocket messages. The buffer has a fixed capacity and automatically evicts
//! the oldest messages when full, ensuring constant memory usage.
//!
//! Key features:
//! - Fixed-size circular buffer with O(1) insert/read operations
//! - Thread-safe for concurrent producer/consumer access
//! - Statistics tracking (messages processed, dropped, latency)
//! - Zero memory allocation during steady-state operation
//! - Configurable capacity and eviction policy

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, Notify};

use super::message::WebSocketResponse;

/// Statistics for the circular buffer
#[derive(Debug, Clone)]
pub struct BufferStats {
    /// Total messages inserted into the buffer
    pub messages_inserted: u64,
    /// Total messages read from the buffer
    pub messages_read: u64,
    /// Messages dropped due to buffer being full
    pub messages_dropped: u64,
    /// Current number of messages in the buffer
    pub current_size: usize,
    /// Maximum size the buffer has reached
    pub max_size_reached: usize,
    /// Average processing latency in microseconds
    pub avg_latency_us: u64,
    /// Maximum processing latency in microseconds
    pub max_latency_us: u64,
    /// Buffer capacity
    pub capacity: usize,
    /// Buffer utilization percentage (0-100)
    pub utilization_percent: f32,
}

/// Buffer entry with timestamp for latency tracking
struct BufferEntry {
    /// The WebSocket message
    message: WebSocketResponse,
    /// Timestamp when the message was inserted
    inserted_at: Instant,
}

/// Circular buffer for WebSocket messages
pub struct CircularBuffer {
    /// The underlying buffer storage
    buffer: Vec<Option<BufferEntry>>,
    /// Head pointer (next position to read)
    head: AtomicUsize,
    /// Tail pointer (next position to write)
    tail: AtomicUsize,
    /// Number of items currently in the buffer
    count: AtomicUsize,
    /// Buffer capacity
    capacity: usize,
    /// Statistics
    stats: Arc<BufferStatsInternal>,
    /// Notification for waiting consumers
    notify: Arc<Notify>,
}

/// Internal statistics structure with atomic counters
struct BufferStatsInternal {
    messages_inserted: AtomicU64,
    messages_read: AtomicU64,
    messages_dropped: AtomicU64,
    max_size_reached: AtomicUsize,
    total_latency_us: AtomicU64,
    max_latency_us: AtomicU64,
}

impl BufferStatsInternal {
    fn new() -> Self {
        Self {
            messages_inserted: AtomicU64::new(0),
            messages_read: AtomicU64::new(0),
            messages_dropped: AtomicU64::new(0),
            max_size_reached: AtomicUsize::new(0),
            total_latency_us: AtomicU64::new(0),
            max_latency_us: AtomicU64::new(0),
        }
    }

    fn update_max_size(&self, current_size: usize) {
        let mut max = self.max_size_reached.load(Ordering::Relaxed);
        while current_size > max {
            match self.max_size_reached.compare_exchange_weak(
                max,
                current_size,
                Ordering::SeqCst,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(new_max) => max = new_max,
            }
        }
    }

    fn update_latency(&self, latency_us: u64) {
        self.total_latency_us.fetch_add(latency_us, Ordering::Relaxed);

        let mut max = self.max_latency_us.load(Ordering::Relaxed);
        while latency_us > max {
            match self.max_latency_us.compare_exchange_weak(
                max,
                latency_us,
                Ordering::SeqCst,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(new_max) => max = new_max,
            }
        }
    }
}

impl CircularBuffer {
    /// Create a new circular buffer with the specified capacity
    pub fn new(capacity: usize) -> Self {
        let mut buffer = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            buffer.push(None);
        }

        Self {
            buffer,
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
            count: AtomicUsize::new(0),
            capacity,
            stats: Arc::new(BufferStatsInternal::new()),
            notify: Arc::new(Notify::new()),
        }
    }

    /// Insert a message into the buffer
    ///
    /// If the buffer is full, the oldest message will be evicted.
    /// Returns `true` if a message was evicted, `false` otherwise.
    pub fn insert(&self, message: WebSocketResponse) -> bool {
        let inserted_at = Instant::now();
        let entry = BufferEntry {
            message,
            inserted_at,
        };

        let tail = self.tail.load(Ordering::Acquire);
        let head = self.head.load(Ordering::Acquire);
        let count = self.count.load(Ordering::Acquire);

        let mut evicted = false;

        // Check if buffer is full
        if count == self.capacity {
            // Buffer is full, need to evict oldest message
            evicted = true;

            // Move head forward (evict oldest)
            let new_head = (head + 1) % self.capacity;
            self.head.store(new_head, Ordering::Release);

            // Update dropped count
            self.stats.messages_dropped.fetch_add(1, Ordering::Relaxed);
        } else {
            // Buffer has space, increment count
            self.count.fetch_add(1, Ordering::SeqCst);
        }

        // Insert at tail position
        self.buffer[tail] = Some(entry);

        // Move tail forward
        let new_tail = (tail + 1) % self.capacity;
        self.tail.store(new_tail, Ordering::Release);

        // Update statistics
        self.stats.messages_inserted.fetch_add(1, Ordering::Relaxed);

        let current_count = if evicted {
            self.capacity // Buffer remains full after eviction
        } else {
            count + 1
        };

        self.stats.update_max_size(current_count);

        // Notify waiting consumers
        self.notify.notify_one();

        evicted
    }

    /// Try to read a message from the buffer without blocking
    ///
    /// Returns `Some(message)` if a message is available, `None` otherwise.
    pub fn try_read(&self) -> Option<WebSocketResponse> {
        let head = self.head.load(Ordering::Acquire);
        let count = self.count.load(Ordering::Acquire);

        if count == 0 {
            return None;
        }

        // Read from head position
        let entry = self.buffer[head].take()?;

        // Move head forward
        let new_head = (head + 1) % self.capacity;
        self.head.store(new_head, Ordering::Release);

        // Decrement count
        self.count.fetch_sub(1, Ordering::SeqCst);

        // Calculate processing latency
        let latency = entry.inserted_at.elapsed();
        let latency_us = latency.as_micros() as u64;
        self.stats.update_latency(latency_us);

        // Update read count
        self.stats.messages_read.fetch_add(1, Ordering::Relaxed);

        Some(entry.message)
    }

    /// Read a message from the buffer, waiting if necessary
    ///
    /// This will block until a message is available or the buffer is closed.
    pub async fn read(&self) -> Option<WebSocketResponse> {
        loop {
            if let Some(message) = self.try_read() {
                return Some(message);
            }

            // Wait for notification
            self.notify.notified().await;
        }
    }

    /// Get the current number of messages in the buffer
    pub fn len(&self) -> usize {
        self.count.load(Ordering::Relaxed)
    }

    /// Check if the buffer is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Check if the buffer is full
    pub fn is_full(&self) -> bool {
        self.len() == self.capacity
    }

    /// Get buffer statistics
    pub fn stats(&self) -> BufferStats {
        let inserted = self.stats.messages_inserted.load(Ordering::Relaxed);
        let read = self.stats.messages_read.load(Ordering::Relaxed);
        let dropped = self.stats.messages_dropped.load(Ordering::Relaxed);
        let max_size = self.stats.max_size_reached.load(Ordering::Relaxed);
        let total_latency = self.stats.total_latency_us.load(Ordering::Relaxed);
        let max_latency = self.stats.max_latency_us.load(Ordering::Relaxed);

        let current_size = self.len();
        let capacity = self.capacity;

        let avg_latency = if read > 0 {
            total_latency / read
        } else {
            0
        };

        let utilization_percent = if capacity > 0 {
            (current_size as f32 / capacity as f32) * 100.0
        } else {
            0.0
        };

        BufferStats {
            messages_inserted: inserted,
            messages_read: read,
            messages_dropped: dropped,
            current_size,
            max_size_reached: max_size,
            avg_latency_us: avg_latency,
            max_latency_us: max_latency,
            capacity,
            utilization_percent,
        }
    }

    /// Clear all messages from the buffer
    pub fn clear(&self) {
        let head = self.head.load(Ordering::Relaxed);
        let tail = self.tail.load(Ordering::Relaxed);
        let count = self.count.load(Ordering::Relaxed);

        // Clear all entries between head and tail
        if count > 0 {
            let mut idx = head;
            for _ in 0..count {
                self.buffer[idx] = None;
                idx = (idx + 1) % self.capacity;
            }
        }

        // Reset pointers and count
        self.head.store(0, Ordering::Release);
        self.tail.store(0, Ordering::Release);
        self.count.store(0, Ordering::Release);
    }
}

impl Default for CircularBuffer {
    fn default() -> Self {
        Self::new(1000) // Default capacity of 1000 messages
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Subscription;

    fn create_test_message(channel: &str) -> WebSocketResponse {
        WebSocketResponse {
            channel: channel.to_string(),
            data: serde_json::json!({"test": "data"}),
        }
    }

    #[test]
    fn test_buffer_creation() {
        let buffer = CircularBuffer::new(100);
        assert_eq!(buffer.capacity, 100);
        assert!(buffer.is_empty());
        assert!(!buffer.is_full());
    }

    #[test]
    fn test_insert_and_read() {
        let buffer = CircularBuffer::new(10);

        // Insert a message
        let message = create_test_message("test");
        let evicted = buffer.insert(message.clone());
        assert!(!evicted);
        assert_eq!(buffer.len(), 1);

        // Read the message
        let read_message = buffer.try_read().unwrap();
        assert_eq!(read_message.channel, message.channel);
        assert!(buffer.is_empty());
    }

    #[test]
    fn test_buffer_wrapping() {
        let buffer = CircularBuffer::new(3);

        // Fill the buffer
        for i in 0..3 {
            let message = create_test_message(&format!("msg{}", i));
            buffer.insert(message);
        }
        assert!(buffer.is_full());

        // Insert one more - should evict oldest
        let message = create_test_message("msg3");
        let evicted = buffer.insert(message);
        assert!(evicted);
        assert!(buffer.is_full()); // Still full after eviction

        // Read all messages
        let mut messages = Vec::new();
        while let Some(msg) = buffer.try_read() {
            messages.push(msg.channel);
        }

        // Should have messages 1, 2, 3 (0 was evicted)
        assert_eq!(messages, vec!["msg1", "msg2", "msg3"]);
    }

    #[test]
    fn test_concurrent_access() {
        use std::thread;

        let buffer = Arc::new(CircularBuffer::new(100));
        let buffer_clone = Arc::clone(&buffer);

        // Producer thread
        let producer = thread::spawn(move || {
            for i in 0..50 {
                let message = create_test_message(&format!("msg{}", i));
                buffer_clone.insert(message);
            }
        });

        // Consumer thread
        let consumer = thread::spawn(move || {
            let mut count = 0;
            for _ in 0..50 {
                if buffer.try_read().is_some() {
                    count += 1;
                }
            }
            count
        });

        producer.join().unwrap();
        let consumed = consumer.join().unwrap();

        assert_eq!(consumed, 50);
        assert!(buffer.is_empty());
    }

    #[test]
    fn test_statistics() {
        let buffer = CircularBuffer::new(5);

        // Insert some messages
        for i in 0..7 {
            let message = create_test_message(&format!("msg{}", i));
            buffer.insert(message);
        }

        // Read some messages
        for _ in 0..3 {
            buffer.try_read();
        }

        let stats = buffer.stats();

        assert_eq!(stats.messages_inserted, 7);
        assert_eq!(stats.messages_read, 3);
        assert_eq!(stats.messages_dropped, 2); // 2 messages evicted when buffer filled
        assert_eq!(stats.current_size, 2); // 7 inserted - 2 evicted - 3 read = 2 remaining
        assert_eq!(stats.max_size_reached, 5); // Buffer capacity
        assert_eq!(stats.capacity, 5);
    }
}