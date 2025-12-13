//! Tests for WebSocket circular buffer functionality

use hyperliquid_core::stream::{
    CircularBuffer, WebSocketClient, WebSocketClientConfig, WebSocketResponse,
};
use serde_json::json;
use std::sync::Arc;
use tokio::time::{sleep, Duration};

/// Create a test WebSocket response
fn create_test_response(channel: &str, data: serde_json::Value) -> WebSocketResponse {
    WebSocketResponse {
        channel: channel.to_string(),
        data,
    }
}

#[tokio::test]
async fn test_buffer_creation_and_config() {
    // Test with buffer enabled
    let config = WebSocketClientConfig {
        buffer_capacity: 500,
        enable_buffer: true,
        ..WebSocketClientConfig::default()
    };

    let client = WebSocketClient::with_config(config).unwrap();
    assert!(client.is_buffer_enabled());
    assert_eq!(client.buffer_size(), 0);

    // Test with buffer disabled
    let config = WebSocketClientConfig {
        buffer_capacity: 0,
        enable_buffer: false,
        ..WebSocketClientConfig::default()
    };

    let client = WebSocketClient::with_config(config).unwrap();
    assert!(!client.is_buffer_enabled());
    assert_eq!(client.buffer_size(), 0);
}

#[tokio::test]
async fn test_buffer_insert_and_read() {
    let buffer = CircularBuffer::new(10);

    // Insert messages
    for i in 0..5 {
        let response = create_test_response("test", json!({"index": i}));
        buffer.insert(response);
    }

    assert_eq!(buffer.len(), 5);
    assert!(!buffer.is_full());

    // Read messages
    for i in 0..5 {
        let message = buffer.try_read().unwrap();
        assert_eq!(message.data["index"], i);
    }

    assert!(buffer.is_empty());
}

#[tokio::test]
async fn test_buffer_wrapping_and_eviction() {
    let buffer = CircularBuffer::new(3);

    // Fill buffer to capacity
    for i in 0..3 {
        let response = create_test_response("test", json!({"index": i}));
        buffer.insert(response);
    }

    assert!(buffer.is_full());
    assert_eq!(buffer.len(), 3);

    // Insert one more - should evict oldest (index 0)
    let response = create_test_response("test", json!({"index": 3}));
    let evicted = buffer.insert(response);
    assert!(evicted); // Should have evicted a message
    assert!(buffer.is_full()); // Still full

    // Read all messages
    let mut messages = Vec::new();
    while let Some(msg) = buffer.try_read() {
        messages.push(msg.data["index"].as_i64().unwrap());
    }

    // Should have messages 1, 2, 3 (0 was evicted)
    assert_eq!(messages, vec![1, 2, 3]);
}

#[tokio::test]
async fn test_buffer_statistics() {
    let buffer = CircularBuffer::new(5);

    // Insert some messages
    for i in 0..7 {
        let response = create_test_response("test", json!({"index": i}));
        buffer.insert(response);
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
    assert!(stats.utilization_percent > 0.0);
}

#[tokio::test]
async fn test_buffer_concurrent_access() {
    use tokio::task;

    let buffer = Arc::new(CircularBuffer::new(100));

    // Spawn producer task
    let producer_buffer = Arc::clone(&buffer);
    let producer = task::spawn(async move {
        for i in 0..50 {
            let response = create_test_response("test", json!({"index": i}));
            producer_buffer.insert(response);
            sleep(Duration::from_millis(10)).await;
        }
    });

    // Spawn consumer task
    let consumer_buffer = Arc::clone(&buffer);
    let consumer = task::spawn(async move {
        let mut count = 0;
        for _ in 0..50 {
            if consumer_buffer.try_read().is_some() {
                count += 1;
            }
            sleep(Duration::from_millis(15)).await;
        }
        count
    });

    producer.await.unwrap();
    let consumed = consumer.await.unwrap();

    // Should have consumed all messages (or most of them)
    assert!(consumed > 40);
}

#[tokio::test]
async fn test_buffer_clear() {
    let buffer = CircularBuffer::new(10);

    // Insert messages
    for i in 0..5 {
        let response = create_test_response("test", json!({"index": i}));
        buffer.insert(response);
    }

    assert_eq!(buffer.len(), 5);

    // Clear buffer
    buffer.clear();
    assert!(buffer.is_empty());
    assert_eq!(buffer.len(), 0);

    // Verify we can still insert after clear
    let response = create_test_response("test", json!({"index": 100}));
    buffer.insert(response);
    assert_eq!(buffer.len(), 1);
}

#[tokio::test]
async fn test_buffer_latency_tracking() {
    let buffer = CircularBuffer::new(10);

    // Insert a message
    let response = create_test_response("test", json!({"data": "test"}));
    buffer.insert(response);

    // Wait a bit
    sleep(Duration::from_millis(100)).await;

    // Read the message
    buffer.try_read();

    let stats = buffer.stats();
    assert_eq!(stats.messages_inserted, 1);
    assert_eq!(stats.messages_read, 1);
    assert!(stats.avg_latency_us >= 100_000); // At least 100ms in microseconds
    assert!(stats.max_latency_us >= 100_000);
}

#[tokio::test]
async fn test_buffer_async_read() {
    let buffer = Arc::new(CircularBuffer::new(10));

    // Spawn a consumer that waits for messages
    let consumer_buffer = Arc::clone(&buffer);
    let consumer = tokio::spawn(async move {
        // This will block until a message is available
        let message = consumer_buffer.read().await.unwrap();
        message
    });

    // Wait a bit then insert a message
    sleep(Duration::from_millis(50)).await;

    let response = create_test_response("test", json!({"data": "async_test"}));
    buffer.insert(response);

    // Consumer should receive the message
    let received = consumer.await.unwrap();
    assert_eq!(received.data["data"], "async_test");
}

#[tokio::test]
async fn test_buffer_performance_burst() {
    // Test handling a burst of messages
    let buffer = CircularBuffer::new(1000);

    let start = std::time::Instant::now();

    // Insert 5000 messages rapidly (simulating burst)
    for i in 0..5000 {
        let response = create_test_response("l2Book", json!({"price": i}));
        buffer.insert(response);
    }

    let insert_duration = start.elapsed();
    println!("Inserted 5000 messages in {:?}", insert_duration);

    let stats = buffer.stats();
    assert_eq!(stats.messages_inserted, 5000);
    assert_eq!(stats.messages_dropped, 4000); // 4000 messages evicted (5000 - 1000 capacity)
    assert_eq!(stats.current_size, 1000); // Buffer should be full
    assert!(buffer.is_full());

    // Read all messages
    let start = std::time::Instant::now();
    let mut count = 0;
    while buffer.try_read().is_some() {
        count += 1;
    }

    let read_duration = start.elapsed();
    println!("Read {} messages in {:?}", count, read_duration);

    assert_eq!(count, 1000); // Should have read 1000 messages (buffer capacity)
    assert!(buffer.is_empty());
}

#[test]
fn test_buffer_edge_cases() {
    // Test with capacity 0 (should panic or handle gracefully)
    // For now, we expect capacity > 0

    // Test with capacity 1
    let buffer = CircularBuffer::new(1);
    assert_eq!(buffer.capacity, 1);

    // Insert first message
    let response1 = create_test_response("test", json!({"data": "first"}));
    buffer.insert(response1);
    assert!(buffer.is_full());

    // Insert second message - should evict first
    let response2 = create_test_response("test", json!({"data": "second"}));
    let evicted = buffer.insert(response2);
    assert!(evicted);
    assert!(buffer.is_full());

    // Read - should get second message
    let message = buffer.try_read().unwrap();
    assert_eq!(message.data["data"], "second");
    assert!(buffer.is_empty());
}