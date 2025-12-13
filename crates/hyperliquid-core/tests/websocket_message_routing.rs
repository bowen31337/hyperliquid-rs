//! Tests for WebSocket message parsing and routing

use hyperliquid_core::stream::{
    WebSocketClient, WebSocketClientConfig, WebSocketResponse,
    MessageRouter, Subscription
};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Test basic message parsing from JSON string
#[tokio::test]
async fn test_message_parsing() {
    // Create a valid WebSocket response JSON
    let json_data = json!({
        "channel": "allMids",
        "data": {
            "BTC": "50000.5",
            "ETH": "3000.25"
        },
        "time": 1234567890
    });

    let json_string = json_data.to_string();

    // Test parsing from &str
    let response = WebSocketResponse::try_from(json_string.as_str());
    assert!(response.is_ok(), "Failed to parse WebSocket response from &str");

    let response = response.unwrap();
    assert_eq!(response.channel, "allMids");
    assert!(response.data.is_object());
    assert_eq!(response.time, Some(1234567890));

    // Test parsing from String
    let response = WebSocketResponse::try_from(json_string.clone());
    assert!(response.is_ok(), "Failed to parse WebSocket response from String");

    let response = response.unwrap();
    assert_eq!(response.channel, "allMids");
}

/// Test message routing to specific handlers
#[tokio::test]
async fn test_message_routing() {
    let router = MessageRouter::new();

    // Create counters to track handler calls
    let all_mids_counter = Arc::new(Mutex::new(0));
    let trades_counter = Arc::new(Mutex::new(0));

    // Clone counters for handlers
    let all_mids_counter_clone = all_mids_counter.clone();
    let trades_counter_clone = trades_counter.clone();

    // Register handler for allMids
    router.register_handler(
        Subscription::AllMids,
        move |response: WebSocketResponse| {
            let counter = all_mids_counter_clone.clone();
            tokio::spawn(async move {
                let mut count = counter.lock().await;
                *count += 1;
            });
        }
    ).await;

    // Register handler for trades
    router.register_handler(
        Subscription::Trades { coin: "BTC".to_string() },
        move |response: WebSocketResponse| {
            let counter = trades_counter_clone.clone();
            tokio::spawn(async move {
                let mut count = counter.lock().await;
                *count += 1;
            });
        }
    ).await;

    // Create test messages
    let all_mids_message = WebSocketResponse {
        channel: "allMids".to_string(),
        data: json!({"BTC": "50000.5"}),
        time: Some(1234567890),
    };

    let trades_message = WebSocketResponse {
        channel: "trades.BTC".to_string(),
        data: json!({"price": "50000.5", "size": "1.0"}),
        time: Some(1234567891),
    };

    let unrelated_message = WebSocketResponse {
        channel: "l2Book.ETH".to_string(),
        data: json!({"bids": [], "asks": []}),
        time: Some(1234567892),
    };

    // Route messages
    router.route_message(all_mids_message.clone()).await;
    router.route_message(trades_message.clone()).await;
    router.route_message(unrelated_message.clone()).await;

    // Give handlers time to execute
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Check counters
    let all_mids_count = all_mids_counter.lock().await;
    let trades_count = trades_counter.lock().await;

    assert_eq!(*all_mids_count, 1, "allMids handler should be called once");
    assert_eq!(*trades_count, 1, "trades handler should be called once");
}

/// Test channel to subscription mapping
#[tokio::test]
async fn test_channel_to_subscription_mapping() {
    use hyperliquid_core::stream::router::MessageRouter;

    // Test allMids channel
    let response = WebSocketResponse {
        channel: "allMids".to_string(),
        data: json!({}),
        time: None,
    };

    let subscription = MessageRouter::channel_to_subscription(&response);
    assert!(subscription.is_some());
    assert!(matches!(subscription.unwrap(), Subscription::AllMids));

    // Test trades channel
    let response = WebSocketResponse {
        channel: "trades.BTC".to_string(),
        data: json!({}),
        time: None,
    };

    let subscription = MessageRouter::channel_to_subscription(&response);
    assert!(subscription.is_some());
    match subscription.unwrap() {
        Subscription::Trades { coin } => assert_eq!(coin, "BTC"),
        _ => panic!("Expected Trades subscription"),
    }

    // Test candle channel with interval
    let response = WebSocketResponse {
        channel: "candle.BTC.1m".to_string(),
        data: json!({}),
        time: None,
    };

    let subscription = MessageRouter::channel_to_subscription(&response);
    assert!(subscription.is_some());
    match subscription.unwrap() {
        Subscription::Candle { coin, interval } => {
            assert_eq!(coin, "BTC");
            assert_eq!(interval, "1m");
        }
        _ => panic!("Expected Candle subscription"),
    }

    // Test user events channel
    let response = WebSocketResponse {
        channel: "userEvents.0x1234".to_string(),
        data: json!({}),
        time: None,
    };

    let subscription = MessageRouter::channel_to_subscription(&response);
    assert!(subscription.is_some());
    match subscription.unwrap() {
        Subscription::UserEvents { user } => assert_eq!(user, "0x1234"),
        _ => panic!("Expected UserEvents subscription"),
    }

    // Test unknown channel
    let response = WebSocketResponse {
        channel: "unknown.123".to_string(),
        data: json!({}),
        time: None,
    };

    let subscription = MessageRouter::channel_to_subscription(&response);
    assert!(subscription.is_none(), "Unknown channel should return None");
}

/// Test handler registration and unregistration
#[tokio::test]
async fn test_handler_registration() {
    let router = MessageRouter::new();

    // Initially no handlers
    assert_eq!(router.handler_count().await, 0);

    // Register a handler
    router.register_handler(
        Subscription::AllMids,
        |_| {}
    ).await;

    assert_eq!(router.handler_count().await, 1);
    assert!(router.has_handler(&Subscription::AllMids).await);

    // Register another handler
    router.register_handler(
        Subscription::Trades { coin: "BTC".to_string() },
        |_| {}
    ).await;

    assert_eq!(router.handler_count().await, 2);
    assert!(router.has_handler(&Subscription::Trades { coin: "BTC".to_string() }).await);

    // Unregister a handler
    router.unregister_handler(&Subscription::AllMids).await;

    assert_eq!(router.handler_count().await, 1);
    assert!(!router.has_handler(&Subscription::AllMids).await);
    assert!(router.has_handler(&Subscription::Trades { coin: "BTC".to_string() }).await);
}

/// Test broadcast fallback when no specific handler
#[tokio::test]
async fn test_broadcast_fallback() {
    let router = MessageRouter::new();

    // Create a counter for broadcast handler
    let broadcast_counter = Arc::new(Mutex::new(0));
    let broadcast_counter_clone = broadcast_counter.clone();

    // Register a generic handler (not subscription-specific)
    // In our implementation, broadcast happens when no specific handler is found
    // So we need to register a handler for a different subscription to test broadcast
    router.register_handler(
        Subscription::AllMids,
        move |response: WebSocketResponse| {
            let counter = broadcast_counter_clone.clone();
            tokio::spawn(async move {
                let mut count = counter.lock().await;
                *count += 1;
            });
        }
    ).await;

    // Create a message for a different subscription
    let message = WebSocketResponse {
        channel: "trades.ETH".to_string(),  // No handler registered for this
        data: json!({"price": "3000.0"}),
        time: None,
    };

    // Route the message
    router.route_message(message).await;

    // Give handler time to execute
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Check counter - should be 1 because message was broadcast to all handlers
    let count = broadcast_counter.lock().await;
    assert_eq!(*count, 1, "Broadcast handler should be called");
}

/// Test WebSocket client integration with message routing
#[tokio::test]
async fn test_websocket_client_routing() {
    // Create WebSocket client with test configuration
    let config = WebSocketClientConfig {
        url: "ws://localhost:9999".to_string(),  // Non-existent URL for testing
        ..Default::default()
    };

    let client = WebSocketClient::with_config(config).unwrap();

    // Initially no handlers
    assert_eq!(client.handler_count().await, 0);

    // Register a handler
    let handler_called = Arc::new(Mutex::new(false));
    let handler_called_clone = handler_called.clone();

    client.register_handler(
        Subscription::AllMids,
        move |_| {
            let flag = handler_called_clone.clone();
            tokio::spawn(async move {
                let mut called = flag.lock().await;
                *called = true;
            });
        }
    ).await;

    // Check handler registration
    assert_eq!(client.handler_count().await, 1);
    assert!(client.has_handler(&Subscription::AllMids).await);

    // Unregister handler
    client.unregister_handler(&Subscription::AllMids).await;

    assert_eq!(client.handler_count().await, 0);
    assert!(!client.has_handler(&Subscription::AllMids).await);
}