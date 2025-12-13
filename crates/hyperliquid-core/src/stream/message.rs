use serde::{Deserialize, Serialize};
use crate::types::Subscription;

/// WebSocket request message
#[derive(Debug, Clone, Serialize)]
pub struct WebSocketRequest {
    /// Method to call
    pub method: String,
    /// Subscription details
    pub subscription: Subscription,
}

/// WebSocket response message
#[derive(Debug, Clone, Deserialize)]
pub struct WebSocketResponse {
    /// Channel name
    pub channel: String,
    /// Data payload
    pub data: serde_json::Value,
    /// Timestamp
    pub time: Option<i64>,
}

/// WebSocket message type
#[derive(Debug, Clone)]
pub enum WebSocketMessage {
    /// Request to send to server
    Request(WebSocketRequest),
    /// Response received from server
    Response(WebSocketResponse),
    /// Ping message for heartbeat
    Ping,
    /// Pong response to ping
    Pong,
    /// Error message
    Error(String),
    /// Connection established
    Connected,
    /// Connection closed
    Disconnected,
    /// Reconnection attempt
    Reconnecting(u32), // attempt number
}

impl WebSocketRequest {
    /// Create a new subscription request
    pub fn subscribe(subscription: Subscription) -> Self {
        Self {
            method: "subscribe".to_string(),
            subscription,
        }
    }

    /// Create an unsubscribe request
    pub fn unsubscribe(subscription: Subscription) -> Self {
        Self {
            method: "unsubscribe".to_string(),
            subscription,
        }
    }
}

impl TryFrom<&str> for WebSocketResponse {
    type Error = serde_json::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        serde_json::from_str(value)
    }
}

impl TryFrom<String> for WebSocketResponse {
    type Error = serde_json::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        serde_json::from_str(&value)
    }
}