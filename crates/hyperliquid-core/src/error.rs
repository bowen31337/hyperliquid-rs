use reqwest::StatusCode;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum HyperliquidError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("HTTP {status}: {message}")]
    Http {
        status: StatusCode,
        message: String,
        #[source]
        cause: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("Rate limit exceeded: {0}")]
    RateLimit(String),

    #[error("Rate limit exceeded. Retry after {retry_after}s: {message}")]
    RateLimitWithRetry {
        message: String,
        retry_after: u64,
    },

    #[error("Server error: {status} - {message}")]
    Server {
        status: StatusCode,
        message: String,
    },

    #[error("Client error: {code} - {message}")]
    Client {
        code: i32,
        message: String,
        data: Option<serde_json::Value>,
    },

    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Timeout error: {0}")]
    Timeout(String),

    #[error("Retry exhausted after {attempts} attempts")]
    RetryExhausted { attempts: u32 },

    #[error("WebSocket error: {0}")]
    WebSocket(String),

    #[error("Signing error: {0}")]
    Signing(String),

    #[error("Invalid configuration: {0}")]
    Config(String),

    #[error("TLS error: {0}")]
    Tls(String),

    #[error("Authentication error: {0}")]
    Authentication(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl HyperliquidError {
    pub fn is_retryable(&self) -> bool {
        match self {
            HyperliquidError::Network(_) => true,
            HyperliquidError::Timeout(_) => true,
            HyperliquidError::Http { status, .. } => {
                matches!(status.as_u16(), 500..=599 | 429)
            }
            HyperliquidError::RateLimit(_) => true,
            HyperliquidError::RateLimitWithRetry { .. } => true,
            HyperliquidError::Server { .. } => true,
            _ => false,
        }
    }

    pub fn should_retry_immediately(&self) -> bool {
        match self {
            HyperliquidError::Network(_) => true,
            HyperliquidError::Timeout(_) => true,
            HyperliquidError::Http { status, .. } => {
                matches!(status.as_u16(), 502 | 503 | 504)
            }
            _ => false,
        }
    }
}