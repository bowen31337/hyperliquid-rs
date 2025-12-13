use thiserror::Error;

#[derive(Error, Debug)]
pub enum WebSocketError {
    #[error("WebSocket connection error: {0}")]
    Connection(String),

    #[error("WebSocket send error: {0}")]
    Send(String),

    #[error("WebSocket receive error: {0}")]
    Receive(String),

    #[error("WebSocket protocol error: {0}")]
    Protocol(String),

    #[error("WebSocket timeout: {0}")]
    Timeout(String),

    #[error("WebSocket authentication error: {0}")]
    Authentication(String),

    #[error("WebSocket subscription error: {0}")]
    Subscription(String),

    #[error("WebSocket serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("WebSocket deserialization error: {0}")]
    Deserialization(String),

    #[error("WebSocket already connected")]
    AlreadyConnected,

    #[error("WebSocket not connected")]
    NotConnected,

    #[error("WebSocket reconnection in progress")]
    Reconnecting,

    #[error("WebSocket channel closed")]
    ChannelClosed,
}