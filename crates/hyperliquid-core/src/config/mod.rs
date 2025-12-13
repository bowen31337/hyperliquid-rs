//! Configuration loading and management for Hyperliquid SDK
//!
//! This module provides TOML-based configuration loading with environment
//! variable overrides and validation. It supports both file-based and
//! programmatic configuration.

use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::Path;

/// Main configuration for the Hyperliquid SDK
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Environment settings
    #[serde(default)]
    pub environment: EnvironmentConfig,

    /// HTTP client configuration
    #[serde(default)]
    pub http: HttpClientConfig,

    /// WebSocket client configuration
    #[serde(default)]
    pub websocket: WebSocketConfig,

    /// Runtime configuration
    #[serde(default)]
    pub runtime: RuntimeConfig,

    /// Logging configuration
    #[serde(default)]
    pub logging: LoggingConfig,

    /// Security settings
    #[serde(default)]
    pub security: SecurityConfig,

    /// Metrics configuration
    #[serde(default)]
    pub metrics: MetricsConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            environment: EnvironmentConfig::default(),
            http: HttpClientConfig::default(),
            websocket: WebSocketConfig::default(),
            runtime: RuntimeConfig::default(),
            logging: LoggingConfig::default(),
            security: SecurityConfig::default(),
            metrics: MetricsConfig::default(),
        }
    }
}

impl Config {
    /// Load configuration from TOML file with environment variable overrides
    ///
    /// # Arguments
    ///
    /// * `path` - Path to TOML configuration file
    ///
    /// # Returns
    ///
    /// * `Result<Self>` - Loaded configuration or error
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hyperliquid_core::config::Config;
    ///
    /// let config = Config::load("config/default.toml")?;
    /// ```
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, crate::error::HyperliquidError> {
        let path = path.as_ref();

        // Load base configuration from file
        let toml_content = fs::read_to_string(path)
            .map_err(|e| crate::error::HyperliquidError::Config(format!(
                "Failed to read config file {}: {}",
                path.display(), e
            )))?;

        let mut config: Config = toml::from_str(&toml_content)
            .map_err(|e| crate::error::HyperliquidError::Config(format!(
                "Failed to parse TOML config: {}",
                e
            )))?;

        // Apply environment variable overrides
        config.apply_env_overrides()?;

        // Validate configuration
        config.validate()?;

        Ok(config)
    }

    /// Load configuration with automatic discovery
    ///
    /// This method tries to find configuration files in the following order:
    /// 1. HYPERLIQUID_CONFIG environment variable
    /// 2. config/default.toml
    /// 3. config/testnet.toml (if HYPERLIQUID_ENV=testnet)
    /// 4. Returns default config if no file found
    ///
    /// # Returns
    ///
    /// * `Result<Self>` - Loaded configuration or error
    pub fn load_auto() -> Result<Self, crate::error::HyperliquidError> {
        // Check for explicit config file
        if let Ok(config_path) = env::var("HYPERLIQUID_CONFIG") {
            return Self::load(config_path);
        }

        // Try default config files
        let default_paths = [
            "config/default.toml",
            "config/testnet.toml",
            "config/local.toml",
        ];

        for path in &default_paths {
            if Path::new(path).exists() {
                return Self::load(path);
            }
        }

        // Return default configuration
        Ok(Config::default())
    }

    /// Apply environment variable overrides
    fn apply_env_overrides(&mut self) -> Result<(), crate::error::HyperliquidError> {
        // Environment overrides
        if let Ok(env_str) = env::var("HYPERLIQUID_ENV") {
            self.environment.env = env_str.parse()?;
        }

        if let Ok(base_url) = env::var("HYPERLIQUID_BASE_URL") {
            self.environment.base_url = Some(base_url);
        }

        if let Ok(ws_url) = env::var("HYPERLIQUID_WS_URL") {
            self.environment.websocket_url = Some(ws_url);
        }

        // HTTP client overrides
        if let Ok(timeout) = env::var("HYPERLIQUID_HTTP_TIMEOUT") {
            self.http.request_timeout_ms = timeout.parse()
                .map_err(|e| crate::error::HyperliquidError::Config(format!(
                    "Invalid HYPERLIQUID_HTTP_TIMEOUT: {}",
                    e
                )))?;
        }

        if let Ok(max_conns) = env::var("HYPERLIQUID_MAX_CONNECTIONS") {
            self.http.max_connections_per_host = max_conns.parse()
                .map_err(|e| crate::error::HyperliquidError::Config(format!(
                    "Invalid HYPERLIQUID_MAX_CONNECTIONS: {}",
                    e
                )))?;
        }

        // Runtime overrides
        if let Ok(threads) = env::var("HYPERLIQUID_WORKER_THREADS") {
            self.runtime.worker_threads = threads.parse()
                .map_err(|e| crate::error::HyperliquidError::Config(format!(
                    "Invalid HYPERLIQUID_WORKER_THREADS: {}",
                    e
                )))?;
        }

        if let Ok(log_level) = env::var("HYPERLIQUID_LOG_LEVEL") {
            self.logging.level = log_level;
        }

        if let Ok(log_file) = env::var("HYPERLIQUID_LOG_FILE") {
            self.logging.file_path = Some(log_file);
        }

        Ok(())
    }

    /// Validate configuration values
    fn validate(&self) -> Result<(), crate::error::HyperliquidError> {
        // Validate HTTP timeout
        if self.http.request_timeout_ms < 1000 {
            return Err(crate::error::HyperliquidError::Config(
                "HTTP request timeout must be at least 1000ms".to_string()
            ));
        }

        // Validate connection pool size
        if self.http.max_connections_per_host < 1 {
            return Err(crate::error::HyperliquidError::Config(
                "Max connections per host must be at least 1".to_string()
            ));
        }

        // Validate worker threads
        if self.runtime.worker_threads < 1 {
            return Err(crate::error::HyperliquidError::Config(
                "Worker threads must be at least 1".to_string()
            ));
        }

        // Validate log level
        match self.logging.level.to_lowercase().as_str() {
            "trace" | "debug" | "info" | "warn" | "error" => {},
            _ => {
                return Err(crate::error::HyperliquidError::Config(
                    format!("Invalid log level: {}", self.logging.level)
                ));
            }
        }

        Ok(())
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

/// Environment configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentConfig {
    /// Target environment
    #[serde(default)]
    pub env: crate::types::Environment,

    /// Override base URL (optional)
    #[serde(default)]
    pub base_url: Option<String>,

    /// Override WebSocket URL (optional)
    #[serde(default)]
    pub websocket_url: Option<String>,
}

impl Default for EnvironmentConfig {
    fn default() -> Self {
        Self {
            env: crate::types::Environment::Mainnet,
            base_url: None,
            websocket_url: None,
        }
    }
}

/// HTTP client configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpClientConfig {
    /// Request timeout in milliseconds
    #[serde(default = "default_http_timeout")]
    pub request_timeout_ms: u64,

    /// Maximum connections per host
    #[serde(default = "default_max_connections")]
    pub max_connections_per_host: usize,

    /// Maximum total connections
    #[serde(default = "default_max_total_connections")]
    pub max_total_connections: usize,

    /// Connection timeout in milliseconds
    #[serde(default = "default_connect_timeout")]
    pub connect_timeout_ms: u64,

    /// Enable certificate pinning
    #[serde(default)]
    pub enable_cert_pinning: bool,

    /// Enable rate limiting
    #[serde(default)]
    pub enable_rate_limiting: bool,
}

fn default_http_timeout() -> u64 { 30000 }
fn default_max_connections() -> usize { 10 }
fn default_max_total_connections() -> usize { 100 }
fn default_connect_timeout() -> u64 { 10000 }

impl Default for HttpClientConfig {
    fn default() -> Self {
        Self {
            request_timeout_ms: default_http_timeout(),
            max_connections_per_host: default_max_connections(),
            max_total_connections: default_max_total_connections(),
            connect_timeout_ms: default_connect_timeout(),
            enable_cert_pinning: false,
            enable_rate_limiting: true,
        }
    }
}

/// WebSocket client configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketConfig {
    /// Connection timeout in milliseconds
    #[serde(default = "default_ws_timeout")]
    pub connect_timeout_ms: u64,

    /// Reconnection delay in milliseconds
    #[serde(default = "default_reconnect_delay")]
    pub reconnect_delay_ms: u64,

    /// Maximum reconnection attempts
    #[serde(default = "default_max_reconnect_attempts")]
    pub max_reconnect_attempts: u32,

    /// Message buffer size
    #[serde(default = "default_buffer_size")]
    pub buffer_size: usize,

    /// Enable compression
    #[serde(default)]
    pub enable_compression: bool,

    /// Ping interval in milliseconds
    #[serde(default = "default_ping_interval")]
    pub ping_interval_ms: u64,
}

fn default_ws_timeout() -> u64 { 10000 }
fn default_reconnect_delay() -> u64 { 1000 }
fn default_max_reconnect_attempts() -> u32 { 5 }
fn default_buffer_size() -> usize { 1000 }
fn default_ping_interval() -> u64 { 30000 }

impl Default for WebSocketConfig {
    fn default() -> Self {
        Self {
            connect_timeout_ms: default_ws_timeout(),
            reconnect_delay_ms: default_reconnect_delay(),
            max_reconnect_attempts: default_max_reconnect_attempts(),
            buffer_size: default_buffer_size(),
            enable_compression: false,
            ping_interval_ms: default_ping_interval(),
        }
    }
}

/// Runtime configuration (mirrors RuntimeConfig)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    /// Number of worker threads (0 = use CPU cores)
    #[serde(default)]
    pub worker_threads: usize,

    /// Maximum number of blocking threads
    #[serde(default = "default_max_blocking_threads")]
    pub max_blocking_threads: usize,

    /// Thread stack size in bytes
    #[serde(default = "default_stack_size")]
    pub thread_stack_size: usize,

    /// Enable I/O driver
    #[serde(default = "default_enable_io")]
    pub enable_io: bool,

    /// Enable time driver
    #[serde(default = "default_enable_time")]
    pub enable_time: bool,

    /// Global queue interval for work-stealing
    #[serde(default = "default_global_queue_interval")]
    pub global_queue_interval: u32,

    /// Shutdown timeout in seconds
    #[serde(default = "default_shutdown_timeout")]
    pub shutdown_timeout_secs: u64,
}

fn default_max_blocking_threads() -> usize { 512 }
fn default_stack_size() -> usize { 2 * 1024 * 1024 } // 2MB
fn default_enable_io() -> bool { true }
fn default_enable_time() -> bool { true }
fn default_global_queue_interval() -> u32 { 61 }
fn default_shutdown_timeout() -> u64 { 30 }

impl Default for RuntimeConfig {
    fn default() -> Self {
        let num_cpus = num_cpus::get();

        Self {
            worker_threads: num_cpus,
            max_blocking_threads: default_max_blocking_threads(),
            thread_stack_size: default_stack_size(),
            enable_io: default_enable_io(),
            enable_time: default_enable_time(),
            global_queue_interval: default_global_queue_interval(),
            shutdown_timeout_secs: default_shutdown_timeout(),
        }
    }
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level (trace, debug, info, warn, error)
    #[serde(default = "default_log_level")]
    pub level: String,

    /// Log format (json, pretty, compact)
    #[serde(default = "default_log_format")]
    pub format: String,

    /// File path for log output (optional)
    #[serde(default)]
    pub file_path: Option<String>,

    /// Maximum log file size in MB
    #[serde(default = "default_max_log_size")]
    pub max_log_size_mb: u64,

    /// Maximum number of log files to keep
    #[serde(default = "default_max_log_files")]
    pub max_log_files: u32,

    /// Enable colored output
    #[serde(default = "default_colored_output")]
    pub colored_output: bool,
}

fn default_log_level() -> String { "info".to_string() }
fn default_log_format() -> String { "json".to_string() }
fn default_max_log_size() -> u64 { 100 }
fn default_max_log_files() -> u32 { 5 }
fn default_colored_output() -> bool { true }

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
            format: default_log_format(),
            file_path: None,
            max_log_size_mb: default_max_log_size(),
            max_log_files: default_max_log_files(),
            colored_output: default_colored_output(),
        }
    }
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Enable certificate pinning
    #[serde(default)]
    pub enable_cert_pinning: bool,

    /// Certificate pinning configuration
    #[serde(default)]
    pub cert_pinning: CertificatePinningConfig,

    /// API key rotation interval in hours
    #[serde(default = "default_rotation_interval")]
    pub key_rotation_hours: u64,

    /// Enable request signing
    #[serde(default)]
    pub enable_request_signing: bool,

    /// Strict mode (fail on security warnings)
    #[serde(default)]
    pub strict_mode: bool,
}

fn default_rotation_interval() -> u64 { 24 }

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            enable_cert_pinning: false,
            cert_pinning: CertificatePinningConfig::default(),
            key_rotation_hours: default_rotation_interval(),
            enable_request_signing: true,
            strict_mode: false,
        }
    }
}

/// Certificate pinning configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificatePinningConfig {
    /// Domain to pin certificate for
    #[serde(default)]
    pub domain: String,

    /// SHA256 hashes of pinned certificates
    #[serde(default)]
    pub pins: Vec<String>,

    /// Enable backup pins
    #[serde(default)]
    pub enable_backup_pins: bool,
}

impl Default for CertificatePinningConfig {
    fn default() -> Self {
        Self {
            domain: String::new(),
            pins: Vec::new(),
            enable_backup_pins: false,
        }
    }
}

/// Metrics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    /// Enable metrics collection
    #[serde(default)]
    pub enabled: bool,

    /// Metrics endpoint path
    #[serde(default = "default_metrics_path")]
    pub endpoint: String,

    /// Metrics collection interval in seconds
    #[serde(default = "default_metrics_interval")]
    pub collection_interval_secs: u64,

    /// Enable Prometheus export
    #[serde(default)]
    pub enable_prometheus: bool,

    /// Custom metrics namespace
    #[serde(default = "default_metrics_namespace")]
    pub namespace: String,
}

fn default_metrics_path() -> String { "/metrics".to_string() }
fn default_metrics_interval() -> u64 { 10 }
fn default_metrics_namespace() -> String { "hyperliquid".to_string() }

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            endpoint: default_metrics_path(),
            collection_interval_secs: default_metrics_interval(),
            enable_prometheus: true,
            namespace: default_metrics_namespace(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.environment.env, crate::types::Environment::Mainnet);
        assert_eq!(config.http.request_timeout_ms, 30000);
        assert_eq!(config.runtime.worker_threads, num_cpus::get());
    }

    #[test]
    fn test_config_validation() {
        let mut config = Config::default();
        config.http.request_timeout_ms = 500; // Too low

        assert!(config.validate().is_err());

        config.http.request_timeout_ms = 5000;
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_env_overrides() {
        env::set_var("HYPERLIQUID_ENV", "testnet");
        env::set_var("HYPERLIQUID_HTTP_TIMEOUT", "60000");
        env::set_var("HYPERLIQUID_WORKER_THREADS", "8");
        env::set_var("HYPERLIQUID_LOG_LEVEL", "debug");

        let mut config = Config::default();
        config.apply_env_overrides().unwrap();

        assert_eq!(config.environment.env, crate::types::Environment::Testnet);
        assert_eq!(config.http.request_timeout_ms, 60000);
        assert_eq!(config.runtime.worker_threads, 8);
        assert_eq!(config.logging.level, "debug");

        // Clean up
        env::remove_var("HYPERLIQUID_ENV");
        env::remove_var("HYPERLIQUID_HTTP_TIMEOUT");
        env::remove_var("HYPERLIQUID_WORKER_THREADS");
        env::remove_var("HYPERLIQUID_LOG_LEVEL");
    }

    #[test]
    fn test_get_base_url() {
        let config = Config::default();
        let url = config.get_base_url();
        assert_eq!(url, "https://api.hyperliquid.xyz");
    }

    #[test]
    fn test_config_file_load() {
        // Create a temporary config file
        let config_content = r#"
            [environment]
            env = "testnet"
            base_url = "https://api.test.hyperliquid.xyz"

            [http]
            request_timeout_ms = 60000
            max_connections_per_host = 20

            [runtime]
            worker_threads = 4
        "#;

        fs::write("test_config.toml", config_content).unwrap();

        let config = Config::load("test_config.toml").unwrap();
        assert_eq!(config.environment.env, crate::types::Environment::Testnet);
        assert_eq!(config.http.request_timeout_ms, 60000);
        assert_eq!(config.runtime.worker_threads, 4);

        // Clean up
        fs::remove_file("test_config.toml").unwrap();
    }
}