//! Hyperliquid Core - High-performance Rust implementation of Hyperliquid SDK
//!
//! This crate provides the core functionality for interacting with the Hyperliquid
//! exchange API with a focus on performance, reliability, and security.

pub mod client;
pub mod types;
pub mod crypto;
pub mod info;
pub mod exchange;
pub mod stream;
pub mod error;
pub mod runtime;
pub mod logging;
pub mod config;
pub mod memory;

pub use client::{HttpClient, HttpClientConfig, RetryPolicy, StatsSummary};
pub use info::InfoClient;
pub use exchange::ExchangeClient;
pub use exchange::ExchangeClientConfig;
pub use types::{Address, Environment, MarketType, Subscription, BaseResponse, ErrorResponse, ApiResponse, Meta, AssetMeta, ExchangeMeta, VaultMeta, UserState, MarginSummary, CrossMarginSummary, Position, PositionDetails, AssetPosition, BuilderInfo, L2BookSnapshot, OrderLevel, Trade, Bbo, BboLevel, Candle, MidPrice, UserEvent, Cleared, ClosedPnl, Deposit, FundingPayment, Liquidation, NewOrder, OrderStatus, PositionUpdate, PnlAnnihilation, Trigger, FilledOrder, Funding, LedgerUpdate, UserLedgerUpdate, ExchangeFill, Fill, OpenOrder, OrderAction, Cancel, BatchCancel, CancelByCloid, BatchCancelByCloid, Modify, BatchModify, Order, Limit, TriggerType, TpSl, TriggerPx, TriggerPxType, Cloid, WsMsg, AllMidsMsg, L2BookMsg, TradesMsg, BboMsg, CandleMsg, PongMsg, UserEventsMsg, UserFillsMsg, OrderUpdatesMsg, UserFundingsMsg, UserNonFundingLedgerUpdatesMsg, WebData2Msg, ActiveAssetCtxMsg, ActiveSpotAssetCtxMsg, ActiveAssetDataMsg, OtherWsMsg, OtherMsg, PerpDexSchemaInput, FundingHistoryRequest, FundingHistoryResponse, UserFeesResponse, parse_response, parse_success_response, parse_error_response, wrap_success, wrap_error, is_error_response, extract_status, extract_nested_data};
pub use memory::{ArenaAllocator, StringInterner, ZeroCopyValue, ObjectPool, MemoryProfiler, AllocationStats, StringInternStats, PoolStats};
pub use error::HyperliquidError;
pub use runtime::{
    RuntimeConfig, ConfiguredRuntime,
    create_default_runtime, create_high_throughput_runtime,
    create_low_latency_runtime, create_single_threaded_runtime,
};
pub use logging::{
    LoggingConfig, init_tracing, generate_trace_id, request_span,
    log_request, log_response, log_error, log_retry,
};
pub use config::{Config, EnvironmentConfig, HttpClientConfig as ConfiguredHttpClientConfig, WebSocketConfig, RuntimeConfig as ConfiguredRuntimeConfig, LoggingConfig as ConfigLoggingConfig, SecurityConfig, MetricsConfig};
pub use crypto::{MultiSigEnvelope, MultiSigUser, MultiSigSignature, sign_multi_sig_envelope, create_multi_sig_envelope, verify_multi_sig_envelope};

/// Result type alias using HyperliquidError
pub type Result<T> = std::result::Result<T, HyperliquidError>;

/// Main configuration for Hyperliquid SDK with comprehensive settings
#[derive(Clone, Debug)]
pub struct Config {
    /// Environment settings
    pub environment: config::EnvironmentConfig,
    /// HTTP client configuration
    pub http: config::HttpClientConfig,
    /// WebSocket configuration
    pub websocket: config::WebSocketConfig,
    /// Runtime configuration
    pub runtime: config::RuntimeConfig,
    /// Logging configuration
    pub logging: config::LoggingConfig,
    /// Security settings
    pub security: config::SecurityConfig,
    /// Metrics configuration
    pub metrics: config::MetricsConfig,
}

impl Config {
    /// Create a new configuration for mainnet
    pub fn mainnet() -> Self {
        Self {
            environment: config::EnvironmentConfig {
                env: crate::types::Environment::Mainnet,
                base_url: None,
                websocket_url: None,
            },
            http: config::HttpClientConfig::default(),
            websocket: config::WebSocketConfig::default(),
            runtime: config::RuntimeConfig::default(),
            logging: config::LoggingConfig::default(),
            security: config::SecurityConfig::default(),
            metrics: config::MetricsConfig::default(),
        }
    }

    /// Create a new configuration for testnet
    pub fn testnet() -> Self {
        Self {
            environment: config::EnvironmentConfig {
                env: crate::types::Environment::Testnet,
                base_url: None,
                websocket_url: None,
            },
            http: config::HttpClientConfig::default(),
            websocket: config::WebSocketConfig::default(),
            runtime: config::RuntimeConfig::default(),
            logging: config::LoggingConfig::default(),
            security: config::SecurityConfig::default(),
            metrics: config::MetricsConfig::default(),
        }
    }

    /// Create a new configuration for local development
    pub fn local() -> Self {
        Self {
            environment: config::EnvironmentConfig {
                env: crate::types::Environment::Local,
                base_url: None,
                websocket_url: None,
            },
            http: config::HttpClientConfig::default(),
            websocket: config::WebSocketConfig::default(),
            runtime: config::RuntimeConfig::default(),
            logging: config::LoggingConfig::default(),
            security: config::SecurityConfig::default(),
            metrics: config::MetricsConfig::default(),
        }
    }

    /// Load configuration from TOML file with environment variable overrides
    pub fn load<P: AsRef<std::path::Path>>(path: P) -> Result<Self, error::HyperliquidError> {
        config::Config::load(path)
    }

    /// Load configuration with automatic discovery
    pub fn load_auto() -> Result<Self, error::HyperliquidError> {
        config::Config::load_auto()
    }

    /// Get the effective base URL (from env or environment config)
    pub fn get_base_url(&self) -> String {
        self.environment.base_url.clone().unwrap_or_else(|| {
            self.environment.env.base_url().to_string()
        })
    }

    /// Get the effective WebSocket URL (from env or environment config)
    pub fn get_websocket_url(&self) -> String {
        self.environment.websocket_url.clone().unwrap_or_else(|| {
            self.environment.env.websocket_url().to_string()
        })
    }

    /// Get the effective environment
    pub fn get_environment(&self) -> crate::types::Environment {
        self.environment.env
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_mainnet() {
        let config = Config::mainnet();
        assert_eq!(config.get_base_url(), "https://api.hyperliquid.xyz");
        assert_eq!(config.http.max_connections_per_host, 10);
    }

    #[test]
    fn test_config_testnet() {
        let config = Config::testnet();
        assert_eq!(config.get_base_url(), "https://api.hyperliquid-testnet.xyz");
    }

    #[test]
    fn test_config_local() {
        let config = Config::local();
        assert_eq!(config.get_base_url(), "http://localhost:3001");
    }
}