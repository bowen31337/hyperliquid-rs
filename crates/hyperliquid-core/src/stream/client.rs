use std::time::Duration;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex, RwLock};
use tokio::time;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures_util::{SinkExt, StreamExt};
use tracing::{debug, error, info, warn};
use serde_json::json;
use rand;

use crate::types::{Environment, Subscription};
use super::error::WebSocketError;
use super::message::{WebSocketMessage, WebSocketRequest, WebSocketResponse};
use super::router::MessageRouter;
use super::buffer::CircularBuffer;

/// Configuration for WebSocket client
#[derive(Clone, Debug)]
pub struct WebSocketClientConfig {
    /// WebSocket URL
    pub url: String,
    /// Connection timeout in seconds
    pub connection_timeout_secs: u64,
    /// Heartbeat interval in seconds
    pub heartbeat_interval_secs: u64,
    /// Reconnection attempts
    pub max_reconnection_attempts: u32,
    /// Reconnection delay base in milliseconds
    pub reconnection_delay_base_ms: u64,
    /// Maximum reconnection delay in milliseconds
    pub max_reconnection_delay_ms: u64,
    /// Enable automatic reconnection
    pub auto_reconnect: bool,
    /// Enable heartbeat
    pub enable_heartbeat: bool,
    /// Circular buffer capacity (0 to disable buffering)
    pub buffer_capacity: usize,
    /// Enable circular buffer for burst handling
    pub enable_buffer: bool,
}

impl Default for WebSocketClientConfig {
    fn default() -> Self {
        Self {
            url: Environment::Mainnet.websocket_url().to_string(),
            connection_timeout_secs: 10,
            heartbeat_interval_secs: 30,
            max_reconnection_attempts: 10,
            reconnection_delay_base_ms: 1000,
            max_reconnection_delay_ms: 30000,
            auto_reconnect: true,
            enable_heartbeat: true,
            buffer_capacity: 1000,
            enable_buffer: true,
        }
    }
}

impl WebSocketClientConfig {
    /// Create configuration for mainnet
    pub fn mainnet() -> Self {
        Self {
            url: Environment::Mainnet.websocket_url().to_string(),
            ..Default::default()
        }
    }

    /// Create configuration for testnet
    pub fn testnet() -> Self {
        Self {
            url: Environment::Testnet.websocket_url().to_string(),
            ..Default::default()
        }
    }

    /// Create configuration for local development
    pub fn local() -> Self {
        Self {
            url: Environment::Local.websocket_url().to_string(),
            ..Default::default()
        }
    }
}

/// WebSocket client state
#[derive(Clone, Debug)]
struct WebSocketState {
    /// Whether the client is connected
    is_connected: bool,
    /// Current reconnection attempt
    reconnection_attempt: u32,
    /// Active subscriptions
    subscriptions: Vec<Subscription>,
    /// Last heartbeat timestamp
    last_heartbeat: Option<std::time::SystemTime>,
}

impl Default for WebSocketState {
    fn default() -> Self {
        Self {
            is_connected: false,
            reconnection_attempt: 0,
            subscriptions: Vec::new(),
            last_heartbeat: None,
        }
    }
}

/// WebSocket event type
#[derive(Debug, Clone)]
pub enum WebSocketEvent {
    /// Connection established
    Connected,
    /// Connection closed
    Disconnected,
    /// Data received
    Data(WebSocketResponse),
    /// Error occurred
    Error(WebSocketError),
    /// Heartbeat received
    Heartbeat,
    /// Reconnection attempt
    Reconnecting(u32),
}

/// WebSocket client for Hyperliquid exchange
pub struct WebSocketClient {
    /// Client configuration
    config: WebSocketClientConfig,
    /// Client state (protected by RwLock for concurrent reads)
    state: Arc<RwLock<WebSocketState>>,
    /// Event sender for broadcasting events to subscribers
    event_tx: mpsc::UnboundedSender<WebSocketEvent>,
    /// Event receiver for internal use
    event_rx: Arc<Mutex<mpsc::UnboundedReceiver<WebSocketEvent>>>,
    /// Message sender for sending requests to WebSocket
    message_tx: mpsc::UnboundedSender<WebSocketRequest>,
    /// Message router for dispatching messages to handlers
    message_router: MessageRouter,
    /// Circular buffer for burst handling
    buffer: Option<Arc<CircularBuffer>>,
    /// Buffer consumer task handle
    buffer_consumer_handle: Option<tokio::task::JoinHandle<()>>,
    /// Shutdown signal
    shutdown_tx: mpsc::Sender<()>,
}

impl WebSocketClient {
    /// Create a new WebSocket client with default configuration
    pub fn new() -> Result<Self, WebSocketError> {
        Self::with_config(WebSocketClientConfig::default())
    }

    /// Create a new WebSocket client with custom configuration
    pub fn with_config(config: WebSocketClientConfig) -> Result<Self, WebSocketError> {
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        let (message_tx, _) = mpsc::unbounded_channel();
        let (shutdown_tx, _) = mpsc::channel(1);

        // Initialize circular buffer if enabled
        let buffer = if config.enable_buffer && config.buffer_capacity > 0 {
            Some(Arc::new(CircularBuffer::new(config.buffer_capacity)))
        } else {
            None
        };

        Ok(Self {
            config,
            state: Arc::new(RwLock::new(WebSocketState::default())),
            event_tx,
            event_rx: Arc::new(Mutex::new(event_rx)),
            message_tx,
            message_router: MessageRouter::new(),
            buffer,
            buffer_consumer_handle: None,
            shutdown_tx,
        })
    }

    /// Connect to the WebSocket server
    pub async fn connect(&mut self) -> Result<(), WebSocketError> {
        let mut state = self.state.write().await;

        if state.is_connected {
            return Err(WebSocketError::AlreadyConnected);
        }

        info!("Connecting to WebSocket: {}", self.config.url);

        // Attempt to establish WebSocket connection
        match time::timeout(
            Duration::from_secs(self.config.connection_timeout_secs),
            connect_async(&self.config.url),
        ).await {
            Ok(Ok((ws_stream, _))) => {
                info!("WebSocket connection established");

                // Update state
                state.is_connected = true;
                state.reconnection_attempt = 0;

                // Send connected event
                let _ = self.event_tx.send(WebSocketEvent::Connected);

                // Start message handling in background task
                self.start_message_handler(ws_stream).await?;

                // Start heartbeat if enabled
                if self.config.enable_heartbeat {
                    self.start_heartbeat().await?;
                }

                // Start reconnection monitor if enabled
                if self.config.auto_reconnect {
                    self.start_reconnection_monitor().await?;
                }

                // Start buffer consumer if buffer is enabled
                if self.buffer.is_some() {
                    self.start_buffer_consumer().await?;
                }

                // Restore previous subscriptions
                self.restore_subscriptions().await?;

                Ok(())
            }
            Ok(Err(e)) => {
                error!("WebSocket connection failed: {}", e);
                Err(WebSocketError::Connection(e.to_string()))
            }
            Err(_) => {
                error!("WebSocket connection timeout");
                Err(WebSocketError::Timeout("Connection timeout".to_string()))
            }
        }
    }

    /// Start the message handler in a background task
    async fn start_message_handler(
        &mut self,
        ws_stream: tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
    ) -> Result<(), WebSocketError> {
        // Split the WebSocket stream
        let (write, read) = ws_stream.split();

        // Create message channel for this connection
        let (message_tx, mut message_rx) = mpsc::unbounded_channel();
        self.message_tx = message_tx;

        // Clone necessary components for background task
        let event_tx = self.event_tx.clone();
        let state = self.state.clone();
        let config = self.config.clone();
        let message_router = self.message_router.clone();
        let buffer = self.buffer.clone();
        let mut shutdown_rx = self.shutdown_tx.subscribe();

        tokio::spawn(async move {
            let (mut write, mut read) = (write, read);

            loop {
                tokio::select! {
                    // Handle incoming messages
                    msg = read.next() => {
                        match msg {
                            Some(Ok(Message::Text(text))) => {
                                debug!("Received WebSocket message: {}", text);

                                // Try to parse as WebSocketResponse
                                match WebSocketResponse::try_from(text.as_str()) {
                                    Ok(response) => {
                                        // If buffer is enabled, insert message into buffer
                                        if let Some(buffer) = &buffer {
                                            let evicted = buffer.insert(response.clone());
                                            if evicted {
                                                debug!("Buffer full, evicted oldest message");
                                            }
                                        } else {
                                            // No buffer, route directly
                                            message_router.route_message(response.clone()).await;
                                            // Also send as event for backward compatibility
                                            let _ = event_tx.send(WebSocketEvent::Data(response));
                                        }
                                    }
                                    Err(e) => {
                                        warn!("Failed to parse WebSocket message: {}", e);
                                        // Check if it's a ping/pong
                                        if text == "ping" {
                                            let _ = event_tx.send(WebSocketEvent::Heartbeat);
                                            // Send pong response
                                            if let Err(e) = write.send(Message::Text("pong".to_string())).await {
                                                error!("Failed to send pong: {}", e);
                                            }
                                        }
                                    }
                                }
                            }
                            Some(Ok(Message::Ping(data))) => {
                                debug!("Received WebSocket ping");
                                let _ = event_tx.send(WebSocketEvent::Heartbeat);

                                // Send pong response (required by WebSocket protocol)
                                if let Err(e) = write.send(Message::Pong(data)).await {
                                    error!("Failed to send pong response: {}", e);
                                }
                            }
                            Some(Ok(Message::Pong(_))) => {
                                debug!("Received WebSocket pong");
                                let _ = event_tx.send(WebSocketEvent::Heartbeat);
                            }
                            Some(Ok(Message::Close(_))) => {
                                info!("WebSocket connection closed by server");
                                let _ = event_tx.send(WebSocketEvent::Disconnected);
                                // Trigger reconnection if enabled
                                if config.auto_reconnect {
                                    let _ = event_tx.send(WebSocketEvent::Reconnecting(1));
                                }
                                break;
                            }
                            Some(Err(e)) => {
                                error!("WebSocket error: {}", e);
                                let _ = event_tx.send(WebSocketEvent::Error(
                                    WebSocketError::Receive(e.to_string())
                                ));
                                // Trigger reconnection if enabled
                                if config.auto_reconnect {
                                    let _ = event_tx.send(WebSocketEvent::Reconnecting(1));
                                }
                                break;
                            }
                            None => {
                                info!("WebSocket stream ended");
                                let _ = event_tx.send(WebSocketEvent::Disconnected);
                                // Trigger reconnection if enabled
                                if config.auto_reconnect {
                                    let _ = event_tx.send(WebSocketEvent::Reconnecting(1));
                                }
                                break;
                            }
                            _ => {
                                // Ignore other message types
                            }
                        }
                    }

                    // Handle outgoing messages
                    msg = message_rx.recv() => {
                        match msg {
                            Some(request) => {
                                // Check if this is a ping request
                                if request.method == "ping" {
                                    // Send WebSocket protocol ping frame (empty payload)
                                    debug!("Sending WebSocket protocol ping");
                                    if let Err(e) = write.send(Message::Ping(vec![])).await {
                                        error!("Failed to send WebSocket ping: {}", e);
                                        let _ = event_tx.send(WebSocketEvent::Error(
                                            WebSocketError::Send(e.to_string())
                                        ));
                                    }
                                } else {
                                    // Serialize and send the request as JSON
                                    match serde_json::to_string(&request) {
                                        Ok(json) => {
                                            debug!("Sending WebSocket request: {}", json);
                                            if let Err(e) = write.send(Message::Text(json)).await {
                                                error!("Failed to send WebSocket message: {}", e);
                                                let _ = event_tx.send(WebSocketEvent::Error(
                                                    WebSocketError::Send(e.to_string())
                                                ));
                                            }
                                        }
                                        Err(e) => {
                                            error!("Failed to serialize WebSocket request: {}", e);
                                            let _ = event_tx.send(WebSocketEvent::Error(
                                                WebSocketError::Serialization(e)
                                            ));
                                        }
                                    }
                                }
                            }
                            None => {
                                // Message channel closed
                                break;
                            }
                        }
                    }

                    // Handle shutdown signal
                    _ = shutdown_rx.recv() => {
                        info!("WebSocket client shutting down");
                        break;
                    }
                }
            }

            // Update state
            if let Ok(mut state) = state.write().await {
                state.is_connected = false;
            }
        });

        Ok(())
    }

    /// Start the buffer consumer task
    async fn start_buffer_consumer(&mut self) -> Result<(), WebSocketError> {
        let buffer = match &self.buffer {
            Some(buffer) => Arc::clone(buffer),
            None => return Ok(()), // No buffer configured
        };

        let message_router = self.message_router.clone();
        let event_tx = self.event_tx.clone();
        let mut shutdown_rx = self.shutdown_tx.subscribe();

        let handle = tokio::spawn(async move {
            loop {
                tokio::select! {
                    // Read message from buffer
                    Some(message) = buffer.read() => {
                        // Route the message to appropriate handlers
                        message_router.route_message(message.clone()).await;
                        // Also send as event for backward compatibility
                        let _ = event_tx.send(WebSocketEvent::Data(message));
                    }
                    // Handle shutdown signal
                    _ = shutdown_rx.recv() => {
                        debug!("Buffer consumer shutting down");
                        break;
                    }
                }
            }
        });

        self.buffer_consumer_handle = Some(handle);
        Ok(())
    }

    /// Get buffer statistics if buffer is enabled
    pub fn buffer_stats(&self) -> Option<super::buffer::BufferStats> {
        self.buffer.as_ref().map(|buffer| buffer.stats())
    }

    /// Get current buffer size (number of messages in buffer)
    pub fn buffer_size(&self) -> usize {
        self.buffer.as_ref().map(|buffer| buffer.len()).unwrap_or(0)
    }

    /// Check if buffer is enabled
    pub fn is_buffer_enabled(&self) -> bool {
        self.buffer.is_some()
    }

    /// Clear the buffer (remove all messages)
    pub fn clear_buffer(&self) {
        if let Some(buffer) = &self.buffer {
            buffer.clear();
        }
    }

    /// Attempt to reconnect with exponential backoff
    async fn attempt_reconnection(&mut self) -> Result<(), WebSocketError> {
        let mut state = self.state.write().await;

        if state.is_connected {
            return Err(WebSocketError::AlreadyConnected);
        }

        if state.reconnection_attempt >= self.config.max_reconnection_attempts {
            return Err(WebSocketError::Connection(
                format!("Max reconnection attempts ({}) exceeded", self.config.max_reconnection_attempts)
            ));
        }

        // Increment reconnection attempt counter
        state.reconnection_attempt += 1;
        let attempt = state.reconnection_attempt;

        // Calculate exponential backoff delay with jitter
        let base_delay_ms = self.config.reconnection_delay_base_ms;
        let max_delay_ms = self.config.max_reconnection_delay_ms;

        // Exponential backoff: base * 2^(attempt-1) because attempt is 1-indexed now
        let mut delay_ms = base_delay_ms * 2u64.pow(attempt.saturating_sub(1));

        // Add jitter: ±25% of delay
        let jitter_range = (delay_ms as f64 * 0.25) as u64;
        let jitter = rand::random::<u64>() % (jitter_range * 2);
        delay_ms = delay_ms.saturating_sub(jitter_range).saturating_add(jitter);

        // Cap at maximum delay
        delay_ms = delay_ms.min(max_delay_ms);

        info!("Reconnection attempt {} in {}ms", attempt, delay_ms);

        // Send reconnecting event
        let _ = self.event_tx.send(WebSocketEvent::Reconnecting(attempt));

        // Release the lock before sleeping
        drop(state);

        // Wait for backoff period
        tokio::time::sleep(Duration::from_millis(delay_ms)).await;

        // Attempt reconnection
        self.connect().await
    }

    /// Start automatic reconnection monitoring
    async fn start_reconnection_monitor(&mut self) -> Result<(), WebSocketError> {
        if !self.config.auto_reconnect {
            return Ok(());
        }

        let event_tx = self.event_tx.clone();
        let state = self.state.clone();
        let config = self.config.clone();
        let mut client = self.clone();
        let mut shutdown_rx = self.shutdown_tx.subscribe();

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    // Monitor for disconnection events
                    _ = tokio::time::sleep(Duration::from_secs(1)) => {
                        // Check connection status periodically
                        let state_guard = state.read().await;
                        if !state_guard.is_connected && config.auto_reconnect {
                            drop(state_guard);

                            // Check if we should attempt reconnection
                            let current_attempt = {
                                let state = state.read().await;
                                state.reconnection_attempt
                            };

                            if current_attempt < config.max_reconnection_attempts {
                                info!("Starting reconnection attempt {}", current_attempt + 1);

                                // Attempt reconnection
                                match client.attempt_reconnection().await {
                                    Ok(_) => {
                                        info!("Reconnection successful");
                                        let _ = event_tx.send(WebSocketEvent::Connected);
                                    }
                                    Err(e) => {
                                        error!("Reconnection failed: {}", e);
                                        let _ = event_tx.send(WebSocketEvent::Error(e));
                                    }
                                }
                            } else {
                                error!("Max reconnection attempts ({}) exceeded", config.max_reconnection_attempts);
                            }
                        }
                    }

                    // Handle shutdown signal
                    _ = shutdown_rx.recv() => {
                        info!("Reconnection monitor shutting down");
                        break;
                    }
                }
            }
        });

        Ok(())
    }

    /// Start heartbeat mechanism
    async fn start_heartbeat(&self) -> Result<(), WebSocketError> {
        let event_tx = self.event_tx.clone();
        let state = self.state.clone();
        let message_tx = self.message_tx.clone();
        let interval = Duration::from_secs(self.config.heartbeat_interval_secs);
        let mut shutdown_rx = self.shutdown_tx.subscribe();

        tokio::spawn(async move {
            let mut interval = time::interval(interval);

            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        // Update last heartbeat timestamp
                        if let Ok(mut state) = state.write().await {
                            state.last_heartbeat = Some(std::time::SystemTime::now());
                        }

                        // Send heartbeat event
                        let _ = event_tx.send(WebSocketEvent::Heartbeat);

                        // Send WebSocket protocol ping for keepalive if connected
                        if let Ok(state) = state.read().await {
                            if state.is_connected {
                                // Send WebSocket protocol ping (empty payload)
                                // Note: We need to send this through the WebSocket connection itself
                                // For now, we'll use the message channel to trigger ping through the main loop
                                // The main loop will handle sending the actual WebSocket Ping frame

                                // Create a special ping request that will be handled by the main loop
                                let ping_request = WebSocketRequest {
                                    method: "ping".to_string(),
                                    subscription: Subscription::AllMids, // Dummy subscription for ping
                                };

                                // Try to send ping request through message channel
                                if let Err(e) = message_tx.send(ping_request) {
                                    warn!("Failed to send ping request: {}", e);
                                }
                            }
                        }
                    }

                    _ = shutdown_rx.recv() => {
                        break;
                    }
                }
            }
        });

        Ok(())
    }

    /// Subscribe to a channel
    pub async fn subscribe(&self, subscription: Subscription) -> Result<(), WebSocketError> {
        // Add to subscriptions list
        {
            let mut state = self.state.write().await;
            if !state.subscriptions.contains(&subscription) {
                state.subscriptions.push(subscription.clone());
            }
        }

        let request = WebSocketRequest::subscribe(subscription);
        self.send_request(request).await
    }

    /// Unsubscribe from a channel
    pub async fn unsubscribe(&self, subscription: Subscription) -> Result<(), WebSocketError> {
        // Remove from subscriptions list
        {
            let mut state = self.state.write().await;
            state.subscriptions.retain(|s| s != &subscription);
        }

        let request = WebSocketRequest::unsubscribe(subscription);
        self.send_request(request).await
    }

    /// Restore all active subscriptions after reconnection
    async fn restore_subscriptions(&self) -> Result<(), WebSocketError> {
        let subscriptions = {
            let state = self.state.read().await;
            state.subscriptions.clone()
        };

        for subscription in subscriptions {
            if let Err(e) = self.subscribe(subscription).await {
                error!("Failed to restore subscription: {}", e);
                // Continue with other subscriptions
            }
        }

        Ok(())
    }

    /// Send a WebSocket request
    async fn send_request(&self, request: WebSocketRequest) -> Result<(), WebSocketError> {
        // Check if connected
        let state = self.state.read().await;
        if !state.is_connected {
            return Err(WebSocketError::NotConnected);
        }

        // Send request through message channel
        self.message_tx.send(request)
            .map_err(|e| WebSocketError::Send(e.to_string()))?;

        Ok(())
    }

    /// Check if client is connected
    pub async fn is_connected(&self) -> bool {
        let state = self.state.read().await;
        state.is_connected
    }

    /// Get active subscriptions
    pub async fn subscriptions(&self) -> Vec<Subscription> {
        let state = self.state.read().await;
        state.subscriptions.clone()
    }

    /// Get next event from the event stream
    pub async fn next_event(&self) -> Option<WebSocketEvent> {
        let mut event_rx = self.event_rx.lock().await;
        event_rx.recv().await
    }

    /// Register a handler for messages from a specific subscription
    pub async fn register_handler<F>(&self, subscription: Subscription, handler: F)
    where
        F: Fn(WebSocketResponse) + Send + Sync + 'static,
    {
        self.message_router.register_handler(subscription, handler).await;
    }

    /// Unregister a handler for a specific subscription
    pub async fn unregister_handler(&self, subscription: &Subscription) {
        self.message_router.unregister_handler(subscription).await;
    }

    /// Get the number of registered handlers
    pub async fn handler_count(&self) -> usize {
        self.message_router.handler_count().await
    }

    /// Check if a handler is registered for a subscription
    pub async fn has_handler(&self, subscription: &Subscription) -> bool {
        self.message_router.has_handler(subscription).await
    }

    /// Shutdown the WebSocket client
    pub async fn shutdown(&self) -> Result<(), WebSocketError> {
        let _ = self.shutdown_tx.send(()).await;
        Ok(())
    }
}

impl Clone for WebSocketClient {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            state: self.state.clone(),
            event_tx: self.event_tx.clone(),
            event_rx: self.event_rx.clone(),
            message_tx: self.message_tx.clone(),
            shutdown_tx: self.shutdown_tx.clone(),
        }
    }
}

impl Default for WebSocketClient {
    fn default() -> Self {
        Self::new().expect("Failed to create WebSocket client")
    }
}