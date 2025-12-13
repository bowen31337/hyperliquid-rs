//! WebSocket message routing system
//!
//! This module provides message routing functionality for WebSocket messages.
//! It routes incoming messages to appropriate handlers based on subscription type.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, error};

use crate::types::Subscription;
use super::message::WebSocketResponse;

/// Handler function type for WebSocket messages
pub type MessageHandler = Box<dyn Fn(WebSocketResponse) + Send + Sync + 'static>;

/// Message router for dispatching WebSocket messages to appropriate handlers
#[derive(Clone)]
pub struct MessageRouter {
    /// Map from subscription to handler
    handlers: Arc<RwLock<HashMap<Subscription, MessageHandler>>>,
    /// Channel for broadcasting messages to all handlers (fallback)
    broadcast_tx: mpsc::UnboundedSender<WebSocketResponse>,
}

impl MessageRouter {
    /// Create a new message router
    pub fn new() -> Self {
        let (broadcast_tx, _) = mpsc::unbounded_channel();

        Self {
            handlers: Arc::new(RwLock::new(HashMap::new())),
            broadcast_tx,
        }
    }

    /// Register a handler for a specific subscription type
    pub async fn register_handler<F>(&self, subscription: Subscription, handler: F)
    where
        F: Fn(WebSocketResponse) + Send + Sync + 'static,
    {
        let mut handlers = self.handlers.write().await;
        handlers.insert(subscription, Box::new(handler));
        debug!("Registered handler for subscription: {:?}", subscription);
    }

    /// Unregister a handler for a specific subscription type
    pub async fn unregister_handler(&self, subscription: &Subscription) {
        let mut handlers = self.handlers.write().await;
        handlers.remove(subscription);
        debug!("Unregistered handler for subscription: {:?}", subscription);
    }

    /// Route a message to the appropriate handler
    pub async fn route_message(&self, response: WebSocketResponse) {
        debug!("Routing WebSocket message: channel={}", response.channel);

        // Try to find a matching subscription based on the channel
        let subscription = match Self::channel_to_subscription(&response) {
            Some(sub) => sub,
            None => {
                // If we can't determine the subscription, broadcast to all
                self.broadcast_message(response).await;
                return;
            }
        };

        // Look for a registered handler
        let handlers = self.handlers.read().await;
        if let Some(handler) = handlers.get(&subscription) {
            debug!("Found handler for subscription: {:?}", subscription);
            handler(response);
        } else {
            // No specific handler, broadcast to all
            drop(handlers);
            self.broadcast_message(response).await;
        }
    }

    /// Broadcast a message to all registered handlers (fallback)
    async fn broadcast_message(&self, response: WebSocketResponse) {
        // Clone the handlers to avoid holding the lock while calling them
        let handlers = {
            let handlers = self.handlers.read().await;
            handlers.clone()
        };

        if handlers.is_empty() {
            debug!("No handlers registered, dropping message");
            return;
        }

        // Call all handlers with the message
        for (subscription, handler) in handlers.iter() {
            debug!("Broadcasting to handler for: {:?}", subscription);
            handler(response.clone());
        }
    }

    /// Get a receiver for broadcast messages
    pub fn broadcast_receiver(&self) -> mpsc::UnboundedReceiver<WebSocketResponse> {
        self.broadcast_tx.subscribe()
    }

    /// Convert a WebSocketResponse channel to a Subscription
    /// This is a simplified mapping - in reality, you'd need to parse the channel
    /// and data to determine the exact subscription
    pub fn channel_to_subscription(response: &WebSocketResponse) -> Option<Subscription> {
        // Parse the channel string to determine subscription type
        // Example channel formats: "allMids", "l2Book.BTC", "trades.ETH", etc.
        let channel = &response.channel;

        if channel == "allMids" {
            return Some(Subscription::AllMids);
        }

        // Parse channel with dot notation: "type.coin" or "type.user"
        let parts: Vec<&str> = channel.split('.').collect();
        if parts.len() >= 2 {
            let subscription_type = parts[0];
            let identifier = parts[1];

            match subscription_type {
                "l2Book" => Some(Subscription::L2Book { coin: identifier.to_string() }),
                "trades" => Some(Subscription::Trades { coin: identifier.to_string() }),
                "bbo" => Some(Subscription::Bbo { coin: identifier.to_string() }),
                "candle" => {
                    if parts.len() >= 3 {
                        Some(Subscription::Candle {
                            coin: identifier.to_string(),
                            interval: parts[2].to_string()
                        })
                    } else {
                        None
                    }
                }
                "userEvents" => Some(Subscription::UserEvents { user: identifier.to_string() }),
                "userFills" => Some(Subscription::UserFills { user: identifier.to_string() }),
                "orderUpdates" => Some(Subscription::OrderUpdates { user: identifier.to_string() }),
                "userFundings" => Some(Subscription::UserFundings { user: identifier.to_string() }),
                "userNonFundingLedgerUpdates" => Some(Subscription::UserNonFundingLedgerUpdates { user: identifier.to_string() }),
                "webData2" => Some(Subscription::WebData2 { user: identifier.to_string() }),
                "activeAssetCtx" => Some(Subscription::ActiveAssetCtx { coin: identifier.to_string() }),
                "activeAssetData" => {
                    if parts.len() >= 3 {
                        Some(Subscription::ActiveAssetData {
                            user: identifier.to_string(),
                            coin: parts[2].to_string()
                        })
                    } else {
                        None
                    }
                }
                _ => None,
            }
        } else {
            None
        }
    }

    /// Get the number of registered handlers
    pub async fn handler_count(&self) -> usize {
        let handlers = self.handlers.read().await;
        handlers.len()
    }

    /// Check if a handler is registered for a subscription
    pub async fn has_handler(&self, subscription: &Subscription) -> bool {
        let handlers = self.handlers.read().await;
        handlers.contains_key(subscription)
    }
}

impl Default for MessageRouter {
    fn default() -> Self {
        Self::new()
    }
}