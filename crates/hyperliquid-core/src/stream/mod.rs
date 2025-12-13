//! WebSocket streaming client for real-time market data
//!
//! This module provides a WebSocket client for subscribing to real-time market data
//! from the Hyperliquid exchange, including order books, trades, candles, and user events.

mod buffer;
mod client;
mod error;
mod message;
mod router;

pub use buffer::{CircularBuffer, BufferStats};
pub use client::{WebSocketClient, WebSocketClientConfig, WebSocketEvent};
pub use error::WebSocketError;
pub use message::{WebSocketMessage, WebSocketRequest, WebSocketResponse};
pub use router::{MessageRouter, MessageHandler};